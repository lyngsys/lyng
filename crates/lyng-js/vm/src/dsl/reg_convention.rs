//! Pinned-register convention for the asm-DSL substrate.
//!
//! Authoritative source: design §5 of
//! docs/lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md
//! and reports/js/lyng-js/llint-dsl-abi.md.
//!
//! AArch64 mapping:
//!
//! | Pin     | Reg | Type                       |
//! | ------- | --- | -------------------------- |
//! | PC      | x19 | *const u8                  |
//! | REGS    | x20 | *mut Value                 |
//! | FV      | x21 | *mut FeedbackEntry         |
//! | VM      | x22 | *mut Vm                    |
//! | TABLE   | x23 | *const DslHandler          |
//! | STATE   | x24 | *mut LlIntState            |
//! | t0..t6  | x9..x15 | scratch (caller-saved) |
//!
//! Refresh discipline (slow-path call):
//!   PRE:   state.frame_pc_offset <- PC - pb_base
//!   POST:  if Refresh: PC/REGS/FV reloaded from state.frame_*
//!
//! Const offsets below are populated by Task B7 using `offset_of!`.

// Placeholders; resolved to concrete values in Task B7.
pub const LLINT_STATE_FRAME_PC_OFFSET: usize = 0;
pub const LLINT_STATE_FRAME_PB_BASE: usize = 0;
pub const LLINT_STATE_FRAME_REGS_BASE: usize = 0;
pub const LLINT_STATE_FRAME_FV_BASE: usize = 0;
pub const LLINT_STATE_PREFIX: usize = 0;
// VM_POLL_PENDING_OFFSET / VM_OPCODE_COUNTER_OFFSET / VM_HEAP_POOL_OFFSET
// stay as placeholders until the `Vm` struct gains explicit fields
// (Tasks B41, B27, B23). The asm bridge never reads through these in
// DSL-0b; they're declared here so backend macros can name them.
pub const VM_POLL_PENDING_OFFSET: usize = 0;
pub const VM_OPCODE_COUNTER_OFFSET: usize = 0;
pub const VM_HEAP_POOL_OFFSET: usize = 0;
