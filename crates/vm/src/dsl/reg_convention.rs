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

use crate::dsl::llint_state::{LlIntCallTarget, LlIntFrameInfo, LlIntState};

pub const LLINT_STATE_FRAME_PC_OFFSET: usize = offset_of!(LlIntState, frame_pc_offset);
pub const LLINT_STATE_FRAME_PB_BASE: usize = offset_of!(LlIntState, frame_pb_base);
pub const LLINT_STATE_FRAME_REGS_BASE: usize = offset_of!(LlIntState, frame_regs_base);
pub const LLINT_STATE_FRAME_FV_BASE: usize = offset_of!(LlIntState, frame_fv_base);
pub const LLINT_STATE_OBJECT_RECORDS_BASE: usize = offset_of!(LlIntState, object_records_base);
pub const LLINT_STATE_OBJECT_SLOTS_BASE: usize = offset_of!(LlIntState, object_slots_base);
// Phase 1.B.1: pre-resolved constants array base + this-mirror.
// Populated at trampoline entry (entry.rs::run_via_dsl) and refreshed
// in the slow-path Refresh arm (slow_path.rs::translate_outcome).
pub const LLINT_STATE_FRAME_CONST_BASE: usize = offset_of!(LlIntState, frame_const_base);
pub const LLINT_STATE_FRAME_THIS_VALUE: usize = offset_of!(LlIntState, frame_this_value);
pub const LLINT_STATE_FRAME_DEPTH: usize = offset_of!(LlIntState, frame_depth);
pub const LLINT_STATE_FRAME_INFO_BASE: usize = offset_of!(LlIntState, frame_info_base);
pub const LLINT_STATE_FRAME_INFO_LEN: usize = offset_of!(LlIntState, frame_info_len);
pub const LLINT_STATE_REGISTER_STACK_TOP: usize = offset_of!(LlIntState, register_stack_top);
pub const LLINT_STATE_REGISTER_STACK_LEN: usize = offset_of!(LlIntState, register_stack_len);
pub const LLINT_STATE_REGISTER_STACK_BASE: usize = offset_of!(LlIntState, register_stack_base);
pub const LLINT_STATE_CALL_TARGETS_BASE: usize = offset_of!(LlIntState, call_targets_base);
pub const LLINT_STATE_CALL_TARGETS_LEN: usize = offset_of!(LlIntState, call_targets_len);
pub const LLINT_STATE_PREFIX: usize = offset_of!(LlIntState, prefix);

pub const LLINT_FRAME_INFO_PB_BASE: usize = offset_of!(LlIntFrameInfo, pb_base);
pub const LLINT_FRAME_INFO_REGS_BASE: usize = offset_of!(LlIntFrameInfo, regs_base);
pub const LLINT_FRAME_INFO_FV_BASE: usize = offset_of!(LlIntFrameInfo, fv_base);
pub const LLINT_FRAME_INFO_CONST_BASE: usize = offset_of!(LlIntFrameInfo, const_base);
pub const LLINT_FRAME_INFO_THIS_VALUE: usize = offset_of!(LlIntFrameInfo, this_value);
pub const LLINT_FRAME_INFO_PC_OFFSET: usize = offset_of!(LlIntFrameInfo, pc_offset);
pub const LLINT_FRAME_INFO_RETURN_REGISTER: usize = offset_of!(LlIntFrameInfo, return_register);
pub const LLINT_FRAME_INFO_FLAGS: usize = offset_of!(LlIntFrameInfo, flags);
pub const LLINT_FRAME_INFO_REGISTER_BASE: usize = offset_of!(LlIntFrameInfo, register_base);
pub const LLINT_FRAME_INFO_REGISTER_LEN: usize = offset_of!(LlIntFrameInfo, register_len);
pub const LLINT_FRAME_INFO_CODE_RAW: usize = offset_of!(LlIntFrameInfo, code_raw);
pub const LLINT_FRAME_INFO_REALM_RAW: usize = offset_of!(LlIntFrameInfo, realm_raw);
pub const LLINT_FRAME_INFO_LEXICAL_ENV_RAW: usize = offset_of!(LlIntFrameInfo, lexical_env_raw);
pub const LLINT_FRAME_INFO_VARIABLE_ENV_RAW: usize = offset_of!(LlIntFrameInfo, variable_env_raw);
pub const LLINT_FRAME_INFO_PRIVATE_ENV_RAW: usize = offset_of!(LlIntFrameInfo, private_env_raw);
pub const LLINT_FRAME_INFO_CALLEE_RAW: usize = offset_of!(LlIntFrameInfo, callee_raw);
pub const LLINT_FRAME_INFO_PARAMETER_INITIALIZER_END_OFFSET: usize =
    offset_of!(LlIntFrameInfo, parameter_initializer_end_offset);
pub const LLINT_FRAME_INFO_FRAME_FLAGS_RAW: usize = offset_of!(LlIntFrameInfo, frame_flags_raw);
pub const LLINT_FRAME_INFO_TAIL_CALLER_RAW: usize = offset_of!(LlIntFrameInfo, tail_caller_raw);
pub const LLINT_FRAME_INFO_TAIL_CALLER_STRICT: usize =
    offset_of!(LlIntFrameInfo, tail_caller_strict);
pub const LLINT_FRAME_INFO_STRIDE_SHIFT: u32 = 7;
pub const LLINT_FRAME_INFO_FAST_RETURN_SAFE_BIT: u32 = 0;
pub const LLINT_FRAME_INFO_STRICT_BIT: u32 = 1;
pub const LLINT_FRAME_INFO_TAIL_CALL_RECYCLE_SAFE_BIT: u32 = 2;

pub const LLINT_CALL_TARGET_CALLEE_BITS: usize = offset_of!(LlIntCallTarget, callee_bits);
pub const LLINT_CALL_TARGET_PB_BASE: usize = offset_of!(LlIntCallTarget, pb_base);
pub const LLINT_CALL_TARGET_FV_BASE: usize = offset_of!(LlIntCallTarget, fv_base);
pub const LLINT_CALL_TARGET_CONST_BASE: usize = offset_of!(LlIntCallTarget, const_base);
pub const LLINT_CALL_TARGET_GLOBAL_THIS: usize = offset_of!(LlIntCallTarget, global_this);
pub const LLINT_CALL_TARGET_CODE_RAW: usize = offset_of!(LlIntCallTarget, code_raw);
pub const LLINT_CALL_TARGET_REGISTER_LEN: usize = offset_of!(LlIntCallTarget, register_len);
pub const LLINT_CALL_TARGET_PARAMETER_COUNT: usize = offset_of!(LlIntCallTarget, parameter_count);
pub const LLINT_CALL_TARGET_FLAGS: usize = offset_of!(LlIntCallTarget, flags);
pub const LLINT_CALL_TARGET_REALM_RAW: usize = offset_of!(LlIntCallTarget, realm_raw);
pub const LLINT_CALL_TARGET_LEXICAL_ENV_RAW: usize = offset_of!(LlIntCallTarget, lexical_env_raw);
pub const LLINT_CALL_TARGET_VARIABLE_ENV_RAW: usize = offset_of!(LlIntCallTarget, variable_env_raw);
pub const LLINT_CALL_TARGET_PRIVATE_ENV_RAW: usize = offset_of!(LlIntCallTarget, private_env_raw);
pub const LLINT_CALL_TARGET_CALLEE_RAW: usize = offset_of!(LlIntCallTarget, callee_raw);
pub const LLINT_CALL_TARGET_PARAMETER_INITIALIZER_END_OFFSET: usize =
    offset_of!(LlIntCallTarget, parameter_initializer_end_offset);
pub const LLINT_CALL_TARGET_STRIDE_SHIFT: u32 = 7;
pub const LLINT_CALL_TARGET_ENABLED_BIT: u32 = 0;
pub const LLINT_CALL_TARGET_FAST_RETURN_SAFE_BIT: u32 = 1;
pub const LLINT_CALL_TARGET_THIS_GLOBAL_BIT: u32 = 2;
pub const LLINT_CALL_TARGET_STRICT_BIT: u32 = 3;
pub const LLINT_CALL_TARGET_TAIL_CALL_RECYCLE_SAFE_BIT: u32 = 4;

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
pub const RUNTIME_OBJECT_LAST_INVALIDATION_EPOCH_OFFSET: usize =
    lyng_gc::RUNTIME_OBJECT_LAST_INVALIDATION_EPOCH_OFFSET;
pub const RUNTIME_OBJECT_INLINE_NAMED_SLOTS_OFFSET: usize =
    lyng_gc::RUNTIME_OBJECT_INLINE_NAMED_SLOTS_OFFSET;

// =============================================================================
// VM-relative offsets (read from pinned register x22 = VM).
//
// Only valid when the `opcode-counters` feature is on; otherwise the
// `dispatch_counters` field doesn't exist on `Vm`.
// =============================================================================

/// Byte offset of `Vm::dispatch_counters` (the `OpcodeDispatchCounterStore`).
///
/// The asm-side counter macros read `[x22, #VM_DISPATCH_COUNTERS_PTR_OFFSET]`
/// to access the counter store. Note: `OpcodeDispatchCounterStore` is a thin
/// wrapper around `Box<DispatchCounters>`; its single field is the Box at
/// offset 0. So the asm path needs TWO loads:
///   1. `ldr x9, [x22, #VM_DISPATCH_COUNTERS_PTR_OFFSET]` — gets the Box's
///      raw pointer (the first u64 of `OpcodeDispatchCounterStore` is the Box).
///   2. `ldr x9, [x9]` (or equivalent indexed load) — dereferences the Box
///      pointer to reach `DispatchCounters`.
///
/// From there, bank offsets (0, 2048, 4096) index into the flat `[u64; 256]`
/// banks.
#[cfg(feature = "opcode-counters")]
pub const VM_DISPATCH_COUNTERS_PTR_OFFSET: usize =
    ::core::mem::offset_of!(crate::vm::Vm, dispatch_counters);

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
