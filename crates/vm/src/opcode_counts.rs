use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};

use lyng_bytecode::{OPCODE_COUNT, Opcode};

use crate::slow_path_counts::{SlowPathCounterStore, SlowPathCounts};

const OPCODE_COUNT_LEN: usize = OPCODE_COUNT as usize;

/// All opcode instrumentation state, lifted out of `Vm`.
///
/// Owns the asm-driven dispatch banks (`Box<DispatchCounters>`), the
/// opt-in call-argument-copy store, and the slow-path enable flag. `Vm`
/// holds a single instance of this struct (and exposes only
/// `opcode_counters()` / `opcode_counters_mut()` accessors plus a
/// builder hook); the asm hot path reads the dispatch-bank pointer via
/// `VM_DISPATCH_COUNTERS_PTR_OFFSET`, which composes
/// `offset_of!(Vm, counters) + offset_of!(OpcodeCounters, dispatch)`.
///
/// `dispatch` is intentionally the first field so the compile-time
/// offset stays small and predictable, matching the encoding limits of
/// `AArch64` LDR/STR scaled immediates.
pub struct OpcodeCounters {
    // `pub(crate)` so `dsl::reg_convention` can resolve
    // `offset_of!(OpcodeCounters, dispatch)` for the asm offset binding.
    pub(crate) dispatch: Box<DispatchCounters>,
    call_argument_copy: Option<CallArgumentCopyCounterStore>,
    slow_path: Option<SlowPathCounterStore>,
}

impl OpcodeCounters {
    pub fn new() -> Self {
        Self {
            dispatch: DispatchCounters::new(),
            call_argument_copy: None,
            slow_path: None,
        }
    }

    /// Zero every counter bank and opt-in store.
    pub fn reset(&mut self) {
        self.dispatch.reset();
        if let Some(store) = &self.call_argument_copy {
            store.reset();
        }
        if let Some(store) = &self.slow_path {
            store.reset();
        }
    }

    pub fn dispatch_counts(&self) -> OpcodeDispatchCounts {
        self.dispatch.snapshot_dispatch()
    }

    pub fn reset_dispatch_counts(&mut self) {
        self.dispatch.reset();
    }

    /// Raw `&DispatchCounters` view — exposed for callers (e.g. the
    /// bench tool) that want to read bank-level counts without going
    /// through a snapshot.
    pub fn dispatch_banks(&self) -> &DispatchCounters {
        &self.dispatch
    }

    pub const fn enable_call_argument_copy(&mut self) {
        if self.call_argument_copy.is_none() {
            self.call_argument_copy = Some(CallArgumentCopyCounterStore::new());
        }
    }

    pub const fn disable_call_argument_copy(&mut self) {
        self.call_argument_copy = None;
    }

    pub fn reset_call_argument_copy(&mut self) {
        if let Some(store) = &self.call_argument_copy {
            store.reset();
        }
    }

    pub fn call_argument_copy_counts(&self) -> Option<CallArgumentCopyCounts> {
        self.call_argument_copy
            .as_ref()
            .map(CallArgumentCopyCounterStore::snapshot)
    }

    pub const fn enable_slow_path(&mut self) {
        if self.slow_path.is_none() {
            self.slow_path = Some(SlowPathCounterStore::new());
        }
    }

    pub const fn disable_slow_path(&mut self) {
        self.slow_path = None;
    }

    pub fn reset_slow_path(&mut self) {
        // The asm path is the source of truth for slow-path counts —
        // it writes into the `slow_semantic` / `slow_safepoint` banks
        // of `DispatchCounters`. Reset those alongside the legacy
        // Rust-side store (kept only so `slow_path_counts()` can return
        // `None` when disabled).
        self.dispatch.slow_semantic.fill(0);
        self.dispatch.slow_safepoint.fill(0);
        if let Some(store) = &self.slow_path {
            store.reset();
        }
    }

    pub const fn slow_path_enabled(&self) -> bool {
        self.slow_path.is_some()
    }

    /// Snapshot the asm-driven slow-path banks. Returns `None` only
    /// when slow-path tracking has been disabled via
    /// `disable_slow_path` (the legacy `SlowPathCounterStore` field is
    /// the enable flag; the actual counts live in `dispatch`).
    pub fn slow_path_counts(&self) -> Option<SlowPathCounts> {
        self.slow_path.as_ref()?;
        Some(SlowPathCounts::from_dispatch_arrays(
            &self.dispatch.slow_semantic,
            &self.dispatch.slow_safepoint,
        ))
    }

    #[inline]
    pub(crate) fn record_argument_scratch_pushes(&self, count: u64) {
        if let Some(store) = &self.call_argument_copy {
            store.record_scratch_pushes(count);
        }
    }

    #[inline]
    pub(crate) fn record_argument_frame_copies(&self, count: u64) {
        if let Some(store) = &self.call_argument_copy {
            store.record_frame_copies(count);
        }
    }
}

impl Default for OpcodeCounters {
    fn default() -> Self {
        Self::new()
    }
}

/// Sentinel stored in `DispatchCounters::current_opcode` when no opcode
/// has been dispatched yet (set at construction and on `reset`).
///
/// Samples that observe this value are attributed to the "non-opcode / native"
/// bucket rather than to any opcode.
pub const CURRENT_OPCODE_IDLE: u64 = u64::MAX;

/// Bit OR-ed into the published opcode byte to mark "currently in this
/// opcode's slow (semantic) path". Opcodes occupy bits 0..=7 (values
/// 0..=255), so bit 8 is a free lane flag.
pub const CURRENT_OPCODE_SLOW_BIT: u64 = 0x100;

/// Decode a raw `current_opcode` cell value into `(opcode, in_slow_path)`.
///
/// Returns `None` for the idle sentinel or any byte that is not a known
/// opcode discriminant (defensive — a torn or stale read should not panic
/// the sampler).
#[must_use]
pub fn decode_current_opcode(raw: u64) -> Option<(Opcode, bool)> {
    if raw == CURRENT_OPCODE_IDLE {
        return None;
    }
    let in_slow = raw & CURRENT_OPCODE_SLOW_BIT != 0;
    // `raw & 0xFF` is always 0..=255, so this conversion never fails.
    let byte = u8::try_from(raw & 0xFF).ok()?;
    Opcode::from_byte(byte).map(|opcode| (opcode, in_slow))
}

/// Flat counter banks for the asm-driven counter increments.
///
/// Layout is `#[repr(C)]` with three sequential `[u64; 256]` banks:
/// - `dispatch[op]`        — bumped at handler entry by `inc_dispatch_counter!`
/// - `slow_semantic[op]`   — bumped at `call_slow!` invocation site
/// - `slow_safepoint[op]`  — bumped at `poll_safepoint!` pending branch
///
/// Followed by a single `current_opcode` `AtomicU64` cell (offset 6144)
/// publishing the live opcode for the sampling profiler.
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
    /// Live opcode published by the asm dispatch prologue (fast lane) and
    /// the slow-semantic counter site (slow lane, `| CURRENT_OPCODE_SLOW_BIT`).
    /// Read concurrently by the sampling profiler, hence `AtomicU64` rather
    /// than a plain `u64` like the count banks above. At offset 6144.
    pub current_opcode: AtomicU64,
}

impl DispatchCounters {
    pub fn new() -> Box<Self> {
        // 3 * 256 count slots + 1 current_opcode cell. Allocate via a
        // zeroed Vec<u64> -> Box conversion to avoid a 6 KB stack copy.
        // SAFETY: A `Box<[u64]>` of exactly 3 * 256 + 1 elements has identical
        // heap layout to `Box<DispatchCounters>`: both are #[repr(C)] contiguous
        // 6152-byte, 8-byte-aligned allocations (AtomicU64 has the same
        // size/align as u64). Verified in tests/dispatch_counters_layout.rs.
        let zeros: Vec<u64> = vec![0; 3 * 256 + 1];
        let boxed_slice: Box<[u64]> = zeros.into_boxed_slice();
        let raw: *mut u64 = Box::into_raw(boxed_slice).cast::<u64>();
        let mut counters = unsafe { Box::from_raw(raw.cast::<Self>()) };
        counters.current_opcode = AtomicU64::new(CURRENT_OPCODE_IDLE);
        counters
    }

    pub fn reset(&mut self) {
        self.dispatch.fill(0);
        self.slow_semantic.fill(0);
        self.slow_safepoint.fill(0);
        self.current_opcode
            .store(CURRENT_OPCODE_IDLE, Ordering::Relaxed);
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

    /// Borrow the live `current_opcode` cell. The sampling profiler turns
    /// this into a raw pointer; the cell stays at a stable heap address for
    /// the lifetime of the owning `DispatchCounters` box.
    #[must_use]
    pub const fn current_opcode_cell(&self) -> &AtomicU64 {
        &self.current_opcode
    }
}

impl Default for DispatchCounters {
    fn default() -> Self {
        Self {
            dispatch: [0; 256],
            slow_semantic: [0; 256],
            slow_safepoint: [0; 256],
            current_opcode: AtomicU64::new(CURRENT_OPCODE_IDLE),
        }
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

#[cfg(test)]
mod current_opcode_tests {
    use super::{CURRENT_OPCODE_IDLE, CURRENT_OPCODE_SLOW_BIT, decode_current_opcode};
    use lyng_bytecode::Opcode;

    #[test]
    fn idle_sentinel_decodes_to_none() {
        assert_eq!(decode_current_opcode(CURRENT_OPCODE_IDLE), None);
    }

    #[test]
    fn fast_lane_decodes_to_opcode_without_slow_flag() {
        let op = Opcode::from_byte(0).expect("opcode 0 exists");
        let raw = u64::from(op as u8);
        assert_eq!(decode_current_opcode(raw), Some((op, false)));
    }

    #[test]
    fn slow_bit_decodes_to_slow_lane() {
        let op = Opcode::from_byte(0).expect("opcode 0 exists");
        let raw = u64::from(op as u8) | CURRENT_OPCODE_SLOW_BIT;
        assert_eq!(decode_current_opcode(raw), Some((op, true)));
    }

    #[test]
    fn unknown_byte_decodes_to_none() {
        // A byte with no opcode discriminant must not panic the sampler.
        // (Pick a byte that Opcode::from_byte rejects; 0xFF is unused today.)
        assert_eq!(decode_current_opcode(0xFF), None);
    }
}
