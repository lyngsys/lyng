use std::cell::Cell;

use lyng_bytecode::{Opcode, OPCODE_COUNT};

const OPCODE_COUNT_LEN: usize = OPCODE_COUNT as usize;

/// Flat counter banks for the asm-driven counter increments.
///
/// Layout is `#[repr(C)]` with three sequential `[u64; 256]` banks:
/// - `dispatch[op]`        — bumped at handler entry by `inc_dispatch_counter!`
/// - `slow_semantic[op]`   — bumped at `call_slow!` invocation site
/// - `slow_safepoint[op]`  — bumped at `poll_safepoint!` pending branch
///
/// Indexed by raw opcode byte (`opcode as u8`). 256 entries reserves
/// space for the full byte range even though Lyng uses ~157 opcodes,
/// to keep offset math cheap (compile-time bank offsets are 0, 2048,
/// 4096 — all encodable as `AArch64` LDR/STR immediates).
///
/// Box-allocated so the Vm pointer stays stable across struct moves
/// (Vm itself isn't pinned; the asm-side `[VM, #offset]` access reads
/// the pointer first, then indexes into the heap-allocated array).
#[repr(C)]
pub struct DispatchCounters {
    pub dispatch: [u64; 256],
    pub slow_semantic: [u64; 256],
    pub slow_safepoint: [u64; 256],
}

impl DispatchCounters {
    pub fn new() -> Box<Self> {
        // `Box::new(Self { ... })` with a 6 KB struct literal would
        // build the struct on the stack first, then copy to the heap.
        // In debug builds that's a 6 KB stack frame which is fine here,
        // but allocate via a zeroed Vec → Box conversion to keep the
        // hot path cheap and predictable across opt levels.
        let zeros: Vec<u64> = vec![0; 3 * 256];
        let boxed_slice: Box<[u64]> = zeros.into_boxed_slice();
        // SAFETY: `Box<[u64; 3 * 256]>` has identical layout to
        // `Box<DispatchCounters>` because both are `#[repr(C)]`-ish
        // contiguous 6144-byte allocations with 8-byte alignment.
        // We verify size + offsets in
        // `tests/dispatch_counters_layout.rs`.
        let raw: *mut u64 = Box::into_raw(boxed_slice).cast::<u64>();
        unsafe { Box::from_raw(raw.cast::<Self>()) }
    }

    pub fn reset(&mut self) {
        self.dispatch.fill(0);
        self.slow_semantic.fill(0);
        self.slow_safepoint.fill(0);
    }

    /// Snapshot the dispatch bank into an `OpcodeDispatchCounts`.
    pub fn snapshot_dispatch(&self) -> OpcodeDispatchCounts {
        OpcodeDispatchCounts::from_dispatch_array(&self.dispatch)
    }

    pub const fn slow_semantic_count(&self, opcode: Opcode) -> u64 {
        self.slow_semantic[opcode as u8 as usize]
    }

    pub const fn slow_safepoint_count(&self, opcode: Opcode) -> u64 {
        self.slow_safepoint[opcode as u8 as usize]
    }
}

impl Default for DispatchCounters {
    fn default() -> Self {
        Self {
            dispatch: [0; 256],
            slow_semantic: [0; 256],
            slow_safepoint: [0; 256],
        }
    }
}

pub struct OpcodeDispatchCounterStore {
    counters: Box<DispatchCounters>,
}

impl OpcodeDispatchCounterStore {
    pub fn new() -> Self {
        Self {
            counters: DispatchCounters::new(),
        }
    }

    pub fn counters(&self) -> &DispatchCounters {
        &self.counters
    }

    pub fn counters_mut(&mut self) -> &mut DispatchCounters {
        &mut self.counters
    }

    /// Raw pointer to the inner `DispatchCounters`, for asm-side reads.
    /// The asm path reads `[VM, #VM_DISPATCH_COUNTERS_PTR_OFFSET]` to
    /// get the `OpcodeDispatchCounterStore` pointer (or its inner Box —
    /// depending on the chosen offset binding strategy).
    pub fn counters_ptr(&self) -> *const DispatchCounters {
        &raw const *self.counters
    }

    #[inline]
    pub fn increment(&self, opcode: Opcode) {
        // SAFETY: single-threaded VM; counter writes are tear-free on
        // aligned u64. The asm path uses non-atomic `add x10, x10, #1`
        // with the same single-threaded guarantee.
        unsafe {
            let counters = &mut *self.counters_ptr().cast_mut();
            counters.dispatch[opcode as u8 as usize] =
                counters.dispatch[opcode as u8 as usize].saturating_add(1);
        }
    }

    pub fn reset(&self) {
        // SAFETY: same as `increment` — single-threaded VM.
        unsafe {
            let counters = &mut *self.counters_ptr().cast_mut();
            counters.reset();
        }
    }

    pub fn snapshot(&self) -> OpcodeDispatchCounts {
        self.counters.snapshot_dispatch()
    }
}

impl Default for OpcodeDispatchCounterStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpcodeDispatchCounts {
    counts: Vec<u64>,
}

impl OpcodeDispatchCounts {
    /// Snapshot the first `OPCODE_COUNT_LEN` entries of a flat 256-bank
    /// dispatch array (the layout used by `DispatchCounters::dispatch`).
    /// Entries past the variant count are padding.
    pub(crate) fn from_dispatch_array(arr: &[u64; 256]) -> Self {
        Self {
            counts: arr[..OPCODE_COUNT_LEN].to_vec(),
        }
    }

    #[must_use]
    pub fn from_counts<I>(counts: I) -> Self
    where
        I: IntoIterator<Item = (Opcode, u64)>,
    {
        let mut snapshot = Self::zeroed();
        for (opcode, count) in counts {
            snapshot.counts[usize::from(opcode as u8)] =
                snapshot.counts[usize::from(opcode as u8)].saturating_add(count);
        }
        snapshot
    }

    #[must_use]
    pub fn count(&self, opcode: Opcode) -> u64 {
        self.counts
            .get(usize::from(opcode as u8))
            .copied()
            .unwrap_or(0)
    }

    #[must_use]
    pub fn total(&self) -> u64 {
        self.counts
            .iter()
            .fold(0_u64, |total, count| total.saturating_add(*count))
    }

    pub fn iter(&self) -> impl Iterator<Item = OpcodeDispatchCount> + '_ {
        self.counts.iter().enumerate().filter_map(|(index, count)| {
            let raw = u8::try_from(index).ok()?;
            Some(OpcodeDispatchCount {
                opcode: Opcode::from_byte(raw)?,
                count: *count,
            })
        })
    }

    #[must_use]
    pub fn top(&self, limit: usize) -> Vec<OpcodeDispatchCount> {
        let mut counts = self
            .iter()
            .filter(|entry| entry.count() != 0)
            .collect::<Vec<_>>();
        counts.sort_unstable_by(|left, right| {
            right
                .count()
                .cmp(&left.count())
                .then_with(|| left.opcode().name().cmp(right.opcode().name()))
        });
        counts.truncate(limit);
        counts
    }

    fn zeroed() -> Self {
        Self {
            counts: vec![0; OPCODE_COUNT_LEN],
        }
    }
}

impl Default for OpcodeDispatchCounts {
    fn default() -> Self {
        Self::zeroed()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpcodeDispatchCount {
    opcode: Opcode,
    count: u64,
}

impl OpcodeDispatchCount {
    #[inline]
    pub const fn opcode(self) -> Opcode {
        self.opcode
    }

    #[inline]
    pub const fn count(self) -> u64 {
        self.count
    }
}

/// Per-VM counters for argument-vector materialization on the call path.
///
/// `scratch_pushes` increments every time the VM pushes a single argument
/// value into its reusable `argument_scratch` Vec during call setup. A
/// well-shaped ordinary bytecode-to-bytecode call with no spread, no
/// bound chain, and no arguments-object usage should not require any
/// pushes — the value can be copied directly from the caller's register
/// window into the callee's frame.
pub struct CallArgumentCopyCounterStore {
    scratch_pushes: Cell<u64>,
    frame_copies: Cell<u64>,
}

impl CallArgumentCopyCounterStore {
    pub const fn new() -> Self {
        Self {
            scratch_pushes: Cell::new(0),
            frame_copies: Cell::new(0),
        }
    }

    #[inline]
    pub fn record_scratch_pushes(&self, count: u64) {
        self.scratch_pushes
            .set(self.scratch_pushes.get().saturating_add(count));
    }

    #[inline]
    pub fn record_frame_copies(&self, count: u64) {
        self.frame_copies
            .set(self.frame_copies.get().saturating_add(count));
    }

    pub fn reset(&self) {
        self.scratch_pushes.set(0);
        self.frame_copies.set(0);
    }

    pub const fn snapshot(&self) -> CallArgumentCopyCounts {
        CallArgumentCopyCounts {
            scratch_pushes: self.scratch_pushes.get(),
            frame_copies: self.frame_copies.get(),
        }
    }
}

impl Default for CallArgumentCopyCounterStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CallArgumentCopyCounts {
    scratch_pushes: u64,
    frame_copies: u64,
}

impl CallArgumentCopyCounts {
    #[must_use]
    #[inline]
    pub const fn scratch_pushes(self) -> u64 {
        self.scratch_pushes
    }

    #[must_use]
    #[inline]
    pub const fn frame_copies(self) -> u64 {
        self.frame_copies
    }
}
