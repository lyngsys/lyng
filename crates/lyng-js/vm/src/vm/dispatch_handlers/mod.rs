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

use lyng_js_bytecode::{Opcode, OPCODE_COUNT};

use super::dispatch_state::{Handler, DISPATCH_TABLE_LEN};

pub mod arithmetic;
pub mod control_flow;
pub mod loads;
pub mod opcode_stubs;
pub mod stub;

pub use arithmetic::op_add;
pub use control_flow::{op_jump_back, op_return};
pub use loads::{op_load_undefined, op_move};

/// Build the dispatch table at compile time.
///
/// 1. Every byte 0..256 starts as `op_stub` — handles invalid bytes outside
///    the valid `Opcode` range (157..256).
/// 2. Every valid opcode slot (0..OPCODE_COUNT) is overwritten with a
///    `op_unimplemented::<N>` monomorphization, giving 157 distinct symbols
///    (Phase 1's structural invariant from lyng-33i2).
/// 3. The five real handlers from the spike override the slots for opcodes
///    we've already ported. Family-conversion sub-issues (lyng-5zrf,
///    lyng-54em, lyng-5mqv, lyng-1fie, lyng-59e6) overwrite progressively
///    more slots until every valid opcode lands on a real handler.
///
/// Adding a real handler in a family file without registering it here is a
/// silent bug — the handler would be dead code and the opcode would still
/// dispatch through `op_unimplemented`.
pub const fn build_dispatch_table() -> [Handler; DISPATCH_TABLE_LEN] {
    let mut table: [Handler; DISPATCH_TABLE_LEN] = [stub::op_stub; DISPATCH_TABLE_LEN];

    // (2) Populate every valid opcode slot with its distinct stub.
    let mut i: usize = 0;
    while i < OPCODE_COUNT as usize {
        table[i] = opcode_stubs::UNIMPLEMENTED_HANDLERS[i];
        i += 1;
    }

    // (3) Real handlers — overrides the unimplemented stubs above.
    table[Opcode::Move as u8 as usize] = op_move;
    table[Opcode::LdaUndefined as u8 as usize] = op_load_undefined;
    table[Opcode::Add as u8 as usize] = op_add;
    table[Opcode::Jump as u8 as usize] = op_jump_back;
    table[Opcode::Return as u8 as usize] = op_return;

    table
}
