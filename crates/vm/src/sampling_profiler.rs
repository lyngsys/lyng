//! Statistical sampling profiler for opcode time-attribution.
//!
//! Gated behind `diagnostic-counters`. A background thread reads the
//! `DispatchCounters::current_opcode` atomic at a fixed interval and bins
//! each observation into a fast/slow per-opcode histogram. Combined with
//! the exact dispatch counts, this gives time-share and cost-per-dispatch.
//!
//! ## Lifetime / safety contract
//!
//! `start` captures a raw `*const AtomicU64` into the (boxed, stable-address)
//! `DispatchCounters` allocation. The borrow checker cannot model "asm writes
//! the cell on one thread while the sampler reads it on another", so the
//! caller MUST keep the owning counters alive until `stop` returns. `stop`
//! joins the sampler thread; `Drop` is a safety net that also signals + joins
//! if `stop` was not called. Construct the profiler AFTER the counters and
//! drop it BEFORE them (natural lexical scope order in the bench driver).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use lyng_bytecode::{Opcode, OPCODE_COUNT};

use crate::opcode_counts::decode_current_opcode;

/// Raw pointer to the shared `current_opcode` cell. `Send` because the cell
/// is an `AtomicU64` at a stable heap address that outlives the thread (see
/// the module-level safety contract).
struct CellPtr(*const AtomicU64);

// SAFETY: the pointee is an AtomicU64 (all accesses are atomic) at a stable
// address kept alive by the caller until the thread is joined.
unsafe impl Send for CellPtr {}

/// Per-opcode fast/slow sample tallies plus a non-opcode bucket.
#[derive(Clone, Debug)]
pub struct SampleHistogram {
    fast: Vec<u64>,
    slow: Vec<u64>,
    non_opcode: u64,
    total: u64,
}

impl SampleHistogram {
    fn zeroed() -> Self {
        let len = OPCODE_COUNT as usize;
        Self {
            fast: vec![0; len],
            slow: vec![0; len],
            non_opcode: 0,
            total: 0,
        }
    }

    /// Merge another histogram into this one (used to sum across samples).
    pub fn merge(&mut self, other: &Self) {
        for index in 0..self.fast.len() {
            self.fast[index] = self.fast[index].saturating_add(other.fast[index]);
            self.slow[index] = self.slow[index].saturating_add(other.slow[index]);
        }
        self.non_opcode = self.non_opcode.saturating_add(other.non_opcode);
        self.total = self.total.saturating_add(other.total);
    }

    #[must_use]
    pub fn fast(&self, opcode: Opcode) -> u64 {
        self.fast.get(usize::from(opcode as u8)).copied().unwrap_or(0)
    }

    #[must_use]
    pub fn slow(&self, opcode: Opcode) -> u64 {
        self.slow.get(usize::from(opcode as u8)).copied().unwrap_or(0)
    }

    /// Total samples attributed to an opcode (fast + slow lanes).
    #[must_use]
    pub fn samples(&self, opcode: Opcode) -> u64 {
        self.fast(opcode).saturating_add(self.slow(opcode))
    }

    /// Samples that observed the idle sentinel (time outside any dispatched
    /// opcode — e.g. before the first dispatch or after the run ended).
    #[must_use]
    pub const fn non_opcode(&self) -> u64 {
        self.non_opcode
    }

    /// Total samples taken (sum of all lanes + non_opcode).
    #[must_use]
    pub const fn total(&self) -> u64 {
        self.total
    }

    /// Iterate opcodes that received at least one sample, yielding
    /// `(opcode, fast_samples, slow_samples)`.
    pub fn iter(&self) -> impl Iterator<Item = (Opcode, u64, u64)> + '_ {
        (0..self.fast.len()).filter_map(move |index| {
            let raw = u8::try_from(index).ok()?;
            let opcode = Opcode::from_byte(raw)?;
            let fast = self.fast[index];
            let slow = self.slow[index];
            if fast == 0 && slow == 0 {
                return None;
            }
            Some((opcode, fast, slow))
        })
    }
}

/// A running sampler. Stop it (or drop it) to collect the histogram.
pub struct SamplingProfiler {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<SampleHistogram>>,
}

impl SamplingProfiler {
    /// Start sampling `cell` every `interval`.
    ///
    /// See the module-level safety contract: `cell` must outlive the profiler,
    /// and the profiler must be stopped/dropped before the cell is freed.
    #[must_use]
    pub fn start(cell: &AtomicU64, interval: Duration) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let ptr = CellPtr(cell as *const AtomicU64);
        let handle = std::thread::spawn(move || {
            let cell_ptr = ptr; // move the Send wrapper into the thread
            let mut hist = SampleHistogram::zeroed();
            while !thread_stop.load(Ordering::Acquire) {
                // SAFETY: pointee is a live AtomicU64 per the safety contract.
                let raw = unsafe { (*cell_ptr.0).load(Ordering::Relaxed) };
                match decode_current_opcode(raw) {
                    Some((opcode, in_slow)) => {
                        let index = usize::from(opcode as u8);
                        if in_slow {
                            hist.slow[index] = hist.slow[index].saturating_add(1);
                        } else {
                            hist.fast[index] = hist.fast[index].saturating_add(1);
                        }
                    }
                    None => hist.non_opcode = hist.non_opcode.saturating_add(1),
                }
                hist.total = hist.total.saturating_add(1);
                std::thread::sleep(interval);
            }
            hist
        });
        Self {
            stop,
            handle: Some(handle),
        }
    }

    /// Stop sampling, join the thread, and return the collected histogram.
    #[must_use]
    pub fn stop(mut self) -> SampleHistogram {
        self.stop.store(true, Ordering::Release);
        self.handle
            .take()
            .map(|handle| handle.join().unwrap_or_else(|_| SampleHistogram::zeroed()))
            .unwrap_or_else(SampleHistogram::zeroed)
    }
}

impl Drop for SamplingProfiler {
    fn drop(&mut self) {
        // Safety net: if stop() was not called, still signal + join so the
        // thread cannot outlive the cell it reads.
        self.stop.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opcode_counts::{CURRENT_OPCODE_IDLE, CURRENT_OPCODE_SLOW_BIT};

    fn sample_constant_cell(value: u64) -> SampleHistogram {
        let cell = AtomicU64::new(value);
        let profiler = SamplingProfiler::start(&cell, Duration::from_millis(1));
        // Let the sampler take several ticks against a constant cell.
        std::thread::sleep(Duration::from_millis(40));
        let hist = profiler.stop();
        // Keep `cell` alive until after stop() joined the thread.
        std::hint::black_box(&cell);
        hist
    }

    #[test]
    fn constant_fast_cell_attributes_all_samples_to_one_fast_lane() {
        let op = Opcode::from_byte(0).expect("opcode 0 exists");
        let hist = sample_constant_cell(u64::from(op as u8));
        assert!(hist.total() > 0, "sampler should have taken ticks");
        assert_eq!(hist.fast(op), hist.total());
        assert_eq!(hist.slow(op), 0);
        assert_eq!(hist.non_opcode(), 0);
    }

    #[test]
    fn constant_slow_cell_attributes_all_samples_to_slow_lane() {
        let op = Opcode::from_byte(0).expect("opcode 0 exists");
        let hist = sample_constant_cell(u64::from(op as u8) | CURRENT_OPCODE_SLOW_BIT);
        assert!(hist.total() > 0);
        assert_eq!(hist.slow(op), hist.total());
        assert_eq!(hist.fast(op), 0);
    }

    #[test]
    fn idle_cell_attributes_all_samples_to_non_opcode() {
        let hist = sample_constant_cell(CURRENT_OPCODE_IDLE);
        assert!(hist.total() > 0);
        assert_eq!(hist.non_opcode(), hist.total());
    }

    #[test]
    fn merge_sums_lanes_and_totals() {
        let op = Opcode::from_byte(0).expect("opcode 0 exists");
        let mut a = sample_constant_cell(u64::from(op as u8));
        let b = sample_constant_cell(u64::from(op as u8));
        let expected = a.total() + b.total();
        a.merge(&b);
        assert_eq!(a.total(), expected);
        assert_eq!(a.fast(op), expected);
    }
}
