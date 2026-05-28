//! Verify `DispatchCounters` layout is stable for asm access.
//!
//! The asm path reads `[VM, #VM_DISPATCH_COUNTERS_PTR_OFFSET]` to get
//! the counter base pointer, then indexes into it with compile-time
//! bank offsets (0, 2048, 4096). These tests pin the layout invariants
//! so a future rustc upgrade or struct re-ordering can't silently
//! break the asm-side reads.

#![cfg(feature = "diagnostic-counters")]

use std::mem::{offset_of, size_of};

use lyng_vm::DispatchCounters;

#[test]
fn dispatch_counters_size_is_expected() {
    // 3 banks * 256 entries * 8 bytes = 6144 bytes.
    assert_eq!(size_of::<DispatchCounters>(), 3 * 256 * 8);
}

#[test]
fn dispatch_counters_field_offsets_are_stable() {
    assert_eq!(offset_of!(DispatchCounters, dispatch), 0);
    assert_eq!(offset_of!(DispatchCounters, slow_semantic), 256 * 8);
    assert_eq!(offset_of!(DispatchCounters, slow_safepoint), 512 * 8);
}
