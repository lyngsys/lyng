#![allow(
    improper_ctypes_definitions,
    reason = "extern \"C\" handlers carry Rust enums by value as an ABI-stability choice, not as a real FFI boundary"
)]

//! Per-opcode "not yet ported" handlers, instantiated as 152 distinct
//! monomorphizations of `op_unimplemented<const OPCODE: u8>`.
//!
//! Phase 1's structural invariant (lyng-33i2) is that every opcode has its
//! own dedicated symbol in `DISPATCH_TABLE` — no shared catch-all. We achieve
//! that here without writing 152 separate `fn` definitions by using a
//! const-generic stub: each `op_unimplemented::<N>` for N in `0..OPCODE_COUNT`
//! is a separate monomorphization with a distinct address.
//!
//! Family-conversion sub-issues (lyng-5zrf loads+control_flow, lyng-54em
//! arithmetic, lyng-5mqv property access, lyng-1fie calls, lyng-59e6
//! scope/generators/exceptions/cold) replace these stubs one family at a
//! time with real handlers.

use lyng_js_bytecode::{Opcode, OPCODE_COUNT};

use crate::error::VmError;
use crate::vm::dispatch_state::{DispatchState, Handler, Step};

/// "Trampoline path: this opcode hasn't been ported yet" handler. The const
/// generic ensures each instantiation is a separate symbol with a distinct
/// function-pointer address, so `DISPATCH_TABLE` ends up with 157 distinct
/// entries even though the source is one function.
#[cold]
#[inline(never)]
pub extern "C" fn op_unimplemented<const OPCODE: u8>(state: &mut DispatchState) -> Step {
    let opcode = Opcode::from_byte(OPCODE)
        .expect("op_unimplemented monomorphized with a byte that doesn't map to an Opcode");
    Step::Error(VmError::UnsupportedOpcode {
        code: state.code(),
        instruction_offset: state.frame.instruction_offset(),
        opcode,
    })
}

/// 152 distinct monomorphizations of `op_unimplemented`, indexed by
/// `Opcode as u8`. Copied into `DISPATCH_TABLE` by `build_dispatch_table`
/// before the real handlers are wired over the slots they cover.
pub const UNIMPLEMENTED_HANDLERS: [Handler; OPCODE_COUNT as usize] = [
    op_unimplemented::<0>,
    op_unimplemented::<1>,
    op_unimplemented::<2>,
    op_unimplemented::<3>,
    op_unimplemented::<4>,
    op_unimplemented::<5>,
    op_unimplemented::<6>,
    op_unimplemented::<7>,
    op_unimplemented::<8>,
    op_unimplemented::<9>,
    op_unimplemented::<10>,
    op_unimplemented::<11>,
    op_unimplemented::<12>,
    op_unimplemented::<13>,
    op_unimplemented::<14>,
    op_unimplemented::<15>,
    op_unimplemented::<16>,
    op_unimplemented::<17>,
    op_unimplemented::<18>,
    op_unimplemented::<19>,
    op_unimplemented::<20>,
    op_unimplemented::<21>,
    op_unimplemented::<22>,
    op_unimplemented::<23>,
    op_unimplemented::<24>,
    op_unimplemented::<25>,
    op_unimplemented::<26>,
    op_unimplemented::<27>,
    op_unimplemented::<28>,
    op_unimplemented::<29>,
    op_unimplemented::<30>,
    op_unimplemented::<31>,
    op_unimplemented::<32>,
    op_unimplemented::<33>,
    op_unimplemented::<34>,
    op_unimplemented::<35>,
    op_unimplemented::<36>,
    op_unimplemented::<37>,
    op_unimplemented::<38>,
    op_unimplemented::<39>,
    op_unimplemented::<40>,
    op_unimplemented::<41>,
    op_unimplemented::<42>,
    op_unimplemented::<43>,
    op_unimplemented::<44>,
    op_unimplemented::<45>,
    op_unimplemented::<46>,
    op_unimplemented::<47>,
    op_unimplemented::<48>,
    op_unimplemented::<49>,
    op_unimplemented::<50>,
    op_unimplemented::<51>,
    op_unimplemented::<52>,
    op_unimplemented::<53>,
    op_unimplemented::<54>,
    op_unimplemented::<55>,
    op_unimplemented::<56>,
    op_unimplemented::<57>,
    op_unimplemented::<58>,
    op_unimplemented::<59>,
    op_unimplemented::<60>,
    op_unimplemented::<61>,
    op_unimplemented::<62>,
    op_unimplemented::<63>,
    op_unimplemented::<64>,
    op_unimplemented::<65>,
    op_unimplemented::<66>,
    op_unimplemented::<67>,
    op_unimplemented::<68>,
    op_unimplemented::<69>,
    op_unimplemented::<70>,
    op_unimplemented::<71>,
    op_unimplemented::<72>,
    op_unimplemented::<73>,
    op_unimplemented::<74>,
    op_unimplemented::<75>,
    op_unimplemented::<76>,
    op_unimplemented::<77>,
    op_unimplemented::<78>,
    op_unimplemented::<79>,
    op_unimplemented::<80>,
    op_unimplemented::<81>,
    op_unimplemented::<82>,
    op_unimplemented::<83>,
    op_unimplemented::<84>,
    op_unimplemented::<85>,
    op_unimplemented::<86>,
    op_unimplemented::<87>,
    op_unimplemented::<88>,
    op_unimplemented::<89>,
    op_unimplemented::<90>,
    op_unimplemented::<91>,
    op_unimplemented::<92>,
    op_unimplemented::<93>,
    op_unimplemented::<94>,
    op_unimplemented::<95>,
    op_unimplemented::<96>,
    op_unimplemented::<97>,
    op_unimplemented::<98>,
    op_unimplemented::<99>,
    op_unimplemented::<100>,
    op_unimplemented::<101>,
    op_unimplemented::<102>,
    op_unimplemented::<103>,
    op_unimplemented::<104>,
    op_unimplemented::<105>,
    op_unimplemented::<106>,
    op_unimplemented::<107>,
    op_unimplemented::<108>,
    op_unimplemented::<109>,
    op_unimplemented::<110>,
    op_unimplemented::<111>,
    op_unimplemented::<112>,
    op_unimplemented::<113>,
    op_unimplemented::<114>,
    op_unimplemented::<115>,
    op_unimplemented::<116>,
    op_unimplemented::<117>,
    op_unimplemented::<118>,
    op_unimplemented::<119>,
    op_unimplemented::<120>,
    op_unimplemented::<121>,
    op_unimplemented::<122>,
    op_unimplemented::<123>,
    op_unimplemented::<124>,
    op_unimplemented::<125>,
    op_unimplemented::<126>,
    op_unimplemented::<127>,
    op_unimplemented::<128>,
    op_unimplemented::<129>,
    op_unimplemented::<130>,
    op_unimplemented::<131>,
    op_unimplemented::<132>,
    op_unimplemented::<133>,
    op_unimplemented::<134>,
    op_unimplemented::<135>,
    op_unimplemented::<136>,
    op_unimplemented::<137>,
    op_unimplemented::<138>,
    op_unimplemented::<139>,
    op_unimplemented::<140>,
    op_unimplemented::<141>,
    op_unimplemented::<142>,
    op_unimplemented::<143>,
    op_unimplemented::<144>,
    op_unimplemented::<145>,
    op_unimplemented::<146>,
    op_unimplemented::<147>,
    op_unimplemented::<148>,
    op_unimplemented::<149>,
    op_unimplemented::<150>,
    op_unimplemented::<151>,
];
