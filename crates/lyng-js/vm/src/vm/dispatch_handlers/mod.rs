#![allow(
    improper_ctypes_definitions,
    reason = "extern \"C\" handlers carry Rust enums by value as an ABI-stability choice, not as a real FFI boundary"
)]
#![allow(
    dead_code,
    reason = "Phase 1 trampoline spike exercises these via tests and cargo-asm only; production cutover lives in follow-up sub-issues"
)]

//! Phase 1 trampoline spike — handler family modules.
//!
//! Each family file holds the `extern "C" fn` handlers for a related group of
//! opcodes. The spike implements one representative handler per family so the
//! trampoline shape can be measured; Phase 1 proper scales this to all 157
//! opcodes per the JSC-aligned roadmap.

use lyng_js_bytecode::Opcode;

use super::dispatch_state::{Handler, DISPATCH_TABLE_LEN};

pub mod arithmetic;
pub mod control_flow;
pub mod loads;
pub mod stub;

pub use arithmetic::op_add;
pub use control_flow::{op_jump_back, op_return};
pub use loads::{op_load_undefined, op_move};
pub use stub::op_stub;

/// Build the dispatch table at compile time: every entry starts as `op_stub`,
/// then real handlers overwrite the slots indexed by their `Opcode as u8`.
///
/// All five spike handlers must appear here. Adding a real handler in a
/// family file without registering it here is a silent bug — the handler
/// would be dead code and the opcode would still dispatch through the stub.
pub const fn build_dispatch_table() -> [Handler; DISPATCH_TABLE_LEN] {
    let mut table: [Handler; DISPATCH_TABLE_LEN] = [op_stub; DISPATCH_TABLE_LEN];
    table[Opcode::Move as u8 as usize] = op_move;
    table[Opcode::LdaUndefined as u8 as usize] = op_load_undefined;
    table[Opcode::Add as u8 as usize] = op_add;
    table[Opcode::Jump as u8 as usize] = op_jump_back;
    table[Opcode::Return as u8 as usize] = op_return;
    table
}
