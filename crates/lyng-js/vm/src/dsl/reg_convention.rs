//! Pinned-register convention for the asm-DSL substrate.
//!
//! Authoritative source: design §5 of
//! docs/lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md
//! and reports/js/lyng-js/llint-dsl-abi.md.
//!
//! AArch64 mapping:
//!
//! | Pin           | Reg     | Type                            |
//! | ------------- | ------- | ------------------------------- |
//! | PC            | x19     | *const u8                       |
//! | REGS          | x20     | *mut Value                      |
//! | FV            | x21     | *mut FeedbackEntry              |
//! | VM            | x22     | *mut Vm                         |
//! | TABLE         | x23     | *const DslHandler               |
//! | STATE         | x24     | *mut LlIntState                 |
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
//!   PRE:   state.frame_pc_offset <- PC - pb_base
//!   POST:  if Refresh: PC/REGS/FV reloaded from state.frame_*
//!
//! Const offsets below are derived from [`LlIntState`] via `offset_of!`
//! and locked in by `tests::ll_int_state_offsets_stable`.

use core::mem::offset_of;

use crate::dsl::llint_state::LlIntState;

pub const LLINT_STATE_FRAME_PC_OFFSET: usize = offset_of!(LlIntState, frame_pc_offset);
pub const LLINT_STATE_FRAME_PB_BASE: usize = offset_of!(LlIntState, frame_pb_base);
pub const LLINT_STATE_FRAME_REGS_BASE: usize = offset_of!(LlIntState, frame_regs_base);
pub const LLINT_STATE_FRAME_FV_BASE: usize = offset_of!(LlIntState, frame_fv_base);
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
