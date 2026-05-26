//! Pinned-register convention for the asm-DSL substrate.
//!
//! Authoritative source: design §5 of
//! docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md
//! and reports/lyng/llint-dsl-abi.md.
//!
//! `AArch64` mapping:
//!
//! | Pin           | Reg     | Type                            |
//! | ------------- | ------- | ------------------------------- |
//! | PC            | x19     | *const u8                       |
//! | REGS          | x20     | *mut Value                      |
//! | FV            | x21     | *mut `FeedbackEntry`              |
//! | VM            | x22     | *mut Vm                         |
//! | TABLE         | x23     | *const `DslHandler`               |
//! | STATE         | x24     | *mut `LlIntState`                 |
//! | t0..t6        | x9..x15 | scratch (caller-saved) — live operand slots |
//! | macro scratch | x16/x17 | AAPCS64 IP0/IP1 — macro-internal only       |
//!
//! Backend macros (`check_smi!`, `tag_smi!`, `dispatch!`,
//! `call_slow!`, `record_smi!`, `poll_safepoint!`, ...) only use
//! `x16`/`x17` (and the call-clobbered `x8`/`w8`) as internal scratch.
//! The proc-macro lowerer's scratch allocator covers `x9..x15`, so the
//! macro-internal `x16`/`x17` never overlap with live operand slots —
//! handlers can `tag_smi!` a result whose destination is still in
//! `x15` without losing operand `a` in `x9`.
//!
//! Refresh discipline (slow-path call):
//!   PRE:   `state.frame_pc_offset` <- PC - `pb_base`
//!   POST:  if Refresh: PC/REGS/FV reloaded from state.frame_*
//!
//! Rust probe hits may use `dispatch_probe_hit_no_refresh!` only when
//! the probe contract guarantees no frame switch, no register-stack
//! relocation, and no feedback-vector relocation. That dispatch form
//! updates PC from the returned payload and leaves pinned REGS/FV intact.
//!
//! Const offsets below are derived from [`LlIntState`] via `offset_of!`
//! and locked in by `tests::ll_int_state_offsets_stable`.

use core::mem::offset_of;

use crate::dsl::llint_state::LlIntState;

pub const LLINT_STATE_FRAME_PC_OFFSET: usize = offset_of!(LlIntState, frame_pc_offset);
pub const LLINT_STATE_FRAME_PB_BASE: usize = offset_of!(LlIntState, frame_pb_base);
pub const LLINT_STATE_FRAME_REGS_BASE: usize = offset_of!(LlIntState, frame_regs_base);
pub const LLINT_STATE_FRAME_FV_BASE: usize = offset_of!(LlIntState, frame_fv_base);
pub const LLINT_STATE_FRAME_METADATA_TABLE_BASE: usize =
    offset_of!(LlIntState, frame_metadata_table_base);

// Phase C Task 4.3: MetadataTable buffer-layout constants re-exported here so
// the proc-macro lowerer (which emits `::lyng_vm::dsl::reg_convention::` paths
// reachable from all crates) can reference them via a fully-public path.
pub use crate::vm::metadata_table::property::PROPERTY_METADATA_STRIDE_SHIFT;
pub use crate::vm::metadata_table::METADATA_TABLE_KIND_OFFSETS_OFFSET;
pub use crate::vm::metadata_table::METADATA_TABLE_SLOT_INDEX_TABLE_OFFSET;
pub const LLINT_STATE_OBJECT_RECORDS_BASE: usize = offset_of!(LlIntState, object_records_base);
pub const LLINT_STATE_OBJECT_SLOTS_BASE: usize = offset_of!(LlIntState, object_slots_base);
// Phase 1.B.1: pre-resolved constants array base + this-mirror.
// Populated at trampoline entry (entry.rs::run_via_dsl) and refreshed
// in the slow-path Refresh arm (slow_path.rs::translate_outcome).
pub const LLINT_STATE_FRAME_CONST_BASE: usize = offset_of!(LlIntState, frame_const_base);
pub const LLINT_STATE_FRAME_THIS_VALUE: usize = offset_of!(LlIntState, frame_this_value);
pub const LLINT_STATE_PREFIX: usize = offset_of!(LlIntState, prefix);

// VM_POLL_PENDING_OFFSET is now derived from `Vm::dsl_poll_pending`,
// added in DSL-0c to give `poll_safepoint!` a known-zero byte to
// dereference on the warm path (`op_loop_header`, conditional
// backward jumps). The field is initialized to 0 in `Vm::new` and
// never written during DSL-0; B41 will give it real semantics.
//
// VM_OPCODE_COUNTER_OFFSET / VM_HEAP_POOL_OFFSET stay as placeholders
// until Tasks B27 / B23 land their respective `Vm` fields. The asm
// bridge never reads through these in DSL-0c; they're declared here
// so backend macros can name them.
pub const VM_POLL_PENDING_OFFSET: usize = offset_of!(crate::vm::Vm, dsl_poll_pending);
pub const VM_OPCODE_COUNTER_OFFSET: usize = 0;
pub const VM_HEAP_POOL_OFFSET: usize = 0;

pub const RUNTIME_OBJECT_SHAPE_OFFSET: usize = lyng_gc::RUNTIME_OBJECT_SHAPE_OFFSET;
pub const RUNTIME_OBJECT_PROTOTYPE_OFFSET: usize = lyng_gc::RUNTIME_OBJECT_PROTOTYPE_OFFSET;
pub const RUNTIME_OBJECT_NAMED_SLOTS_OFFSET: usize = lyng_gc::RUNTIME_OBJECT_NAMED_SLOTS_OFFSET;
pub const RUNTIME_OBJECT_INLINE_NAMED_SLOTS_OFFSET: usize =
    lyng_gc::RUNTIME_OBJECT_INLINE_NAMED_SLOTS_OFFSET;

// =============================================================================
// VM-relative offsets (read from pinned register x22 = VM).
//
// Only valid when the `opcode-counters` feature is on; otherwise the
// `counters` field doesn't exist on `Vm`.
// =============================================================================

/// Byte offset (within `Vm`) of the `Box<DispatchCounters>` that the
/// asm-side counter macros dereference.
///
/// Composes two compile-time offsets:
///   - `offset_of!(Vm, counters)` — `OpcodeCounters` sub-struct on `Vm`.
///   - `offset_of!(OpcodeCounters, dispatch)` — the `Box<DispatchCounters>`
///     field inside it (intentionally the first field, so this offset is
///     0 in practice).
///
/// `Box<DispatchCounters>` is layout-equivalent to a raw pointer, so the
/// asm load `ldr xS, [x22, #VM_DISPATCH_COUNTERS_PTR_OFFSET]` reads the
/// pointer to the heap-allocated `DispatchCounters` directly (one load,
/// no extra dereference). From there, bank offsets (0, 2048, 4096)
/// index into the flat `[u64; 256]` banks.
#[cfg(feature = "opcode-counters")]
pub const VM_DISPATCH_COUNTERS_PTR_OFFSET: usize = ::core::mem::offset_of!(crate::vm::Vm, counters)
    + ::core::mem::offset_of!(crate::opcode_counts::OpcodeCounters, dispatch);

/// Feature-off fallback. The proc-macro lowerer always emits
/// `vm_counter_base = const VM_DISPATCH_COUNTERS_PTR_OFFSET` as a named
/// `naked_asm!` binding (so the leading `/* ... ctr={vm_counter_base} ... */`
/// comment fragment doesn't reference an unbound name). When the feature
/// is off the `inc_dispatch_counter!` / `inc_slow_*_counter!` macros all
/// expand to empty strings and never reference the binding at runtime, so
/// the value is irrelevant — `0` is a safe sentinel.
#[cfg(not(feature = "opcode-counters"))]
pub const VM_DISPATCH_COUNTERS_PTR_OFFSET: usize = 0;

/// Byte offset of the `dispatch` bank within `DispatchCounters`. 0
/// because it's the first field of the `#[repr(C)]` struct.
#[cfg(feature = "opcode-counters")]
pub const DISPATCH_COUNTER_BANK_DISPATCH: usize = 0;

/// Byte offset of the `slow_semantic` bank within `DispatchCounters`.
/// 256 × 8 = 2048 (one full bank past `dispatch`).
#[cfg(feature = "opcode-counters")]
pub const DISPATCH_COUNTER_BANK_SLOW_SEMANTIC: usize = 256 * 8;

/// Byte offset of the `slow_safepoint` bank within `DispatchCounters`.
/// 512 × 8 = 4096 (two full banks past `dispatch`).
#[cfg(feature = "opcode-counters")]
pub const DISPATCH_COUNTER_BANK_SLOW_SAFEPOINT: usize = 512 * 8;

#[cfg(test)]
#[cfg(feature = "opcode-counters")]
mod counter_offset_tests {
    use super::*;

    #[test]
    fn counter_bank_offsets_are_well_aligned() {
        // All bank offsets must be 8-byte-aligned for u64 indexed loads.
        assert_eq!(DISPATCH_COUNTER_BANK_DISPATCH % 8, 0);
        assert_eq!(DISPATCH_COUNTER_BANK_SLOW_SEMANTIC % 8, 0);
        assert_eq!(DISPATCH_COUNTER_BANK_SLOW_SAFEPOINT % 8, 0);
    }

    #[test]
    fn counter_bank_offsets_match_struct_layout() {
        use crate::DispatchCounters;
        use std::mem::offset_of;
        assert_eq!(
            DISPATCH_COUNTER_BANK_DISPATCH,
            offset_of!(DispatchCounters, dispatch)
        );
        assert_eq!(
            DISPATCH_COUNTER_BANK_SLOW_SEMANTIC,
            offset_of!(DispatchCounters, slow_semantic)
        );
        assert_eq!(
            DISPATCH_COUNTER_BANK_SLOW_SAFEPOINT,
            offset_of!(DispatchCounters, slow_safepoint)
        );
    }

    #[test]
    fn vm_dispatch_counters_offset_resolves() {
        // Sanity check that the offset_of!() invocation produces a
        // sensible non-zero value (Vm is large; dispatch_counters is
        // unlikely to be at the very start).
        // Just verifies the const is reachable; the exact value depends
        // on Vm's struct layout which may change.
        std::hint::black_box(VM_DISPATCH_COUNTERS_PTR_OFFSET);
        // No specific assertion — the import resolving is the test.
    }
}
