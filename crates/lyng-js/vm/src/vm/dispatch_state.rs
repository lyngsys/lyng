#![allow(
    improper_ctypes_definitions,
    reason = "extern \"C\" handlers carry Rust enums by value as an ABI-stability choice, not as a real FFI boundary"
)]
#![allow(
    dead_code,
    reason = "Phase 1 trampoline spike exercises these via tests and cargo-asm only; production cutover lives in follow-up sub-issues"
)]

//! Phase 1 trampoline spike (Option α).
//!
//! Self-contained prototype of the per-handler dispatch ABI from
//! `reports/js/lyng-js/jsc-aligned-engine-roadmap.md` — `DispatchState`,
//! `Handler`, `Step`, `DISPATCH_TABLE`, and `dispatch_next!`. The shape here
//! is what production Phase 1 will adopt; the bytecode encoding the handlers
//! consume is intentionally minimal (1-byte register IDs, no feedback slots)
//! so the spike measures trampoline asm quality without dragging in the live
//! Vm/Agent/FrameRecord state.
//!
//! The live `run_dispatch_loop` is untouched. `run_trampoline` is reachable
//! only from the trampoline-spike unit test.

use lyng_js_types::Value;

use crate::error::{VmError, VmResult};

use super::dispatch_handlers;

/// Per-frame execution state threaded through every handler call.
///
/// In the spike this is intentionally narrow: `pc` indexes `bytes`, `regs` is
/// the active register window, `feedback_counter` is a stand-in for the real
/// IC machinery so handlers can demonstrate a "record-feedback" side effect
/// without depending on `FeedbackVector`. Phase 1 proper widens this to carry
/// `&mut Agent`, `&mut FrameRecord`, host references, etc.
#[repr(C)]
pub struct DispatchState<'vm> {
    pub(crate) pc: u32,
    pub(crate) bytes: &'vm [u8],
    pub(crate) regs: &'vm mut [Value],
    pub(crate) feedback_counter: u32,
}

impl<'vm> DispatchState<'vm> {
    #[inline]
    pub fn new(bytes: &'vm [u8], regs: &'vm mut [Value]) -> Self {
        Self {
            pc: 0,
            bytes,
            regs,
            feedback_counter: 0,
        }
    }

    #[inline]
    pub(crate) fn current_bytes(&self) -> &[u8] {
        &self.bytes[self.pc as usize..]
    }

    #[inline]
    pub(crate) fn first_opcode_byte(&self) -> u8 {
        self.bytes[self.pc as usize]
    }

    #[inline]
    pub(crate) fn next_opcode_byte(&self) -> u8 {
        self.bytes[self.pc as usize]
    }

    #[inline]
    pub(crate) fn read_register(&self, idx: u16) -> Value {
        self.regs[idx as usize]
    }

    #[inline]
    pub(crate) fn write_register(&mut self, idx: u16, value: Value) {
        self.regs[idx as usize] = value;
    }

    #[inline]
    pub(crate) fn advance(&mut self, n: u32) {
        self.pc += n;
    }

    #[inline]
    pub(crate) fn record_feedback_arithmetic_smi(&mut self, _slot: u16) {
        self.feedback_counter = self.feedback_counter.wrapping_add(1);
    }

    #[inline]
    pub fn feedback_counter(&self) -> u32 {
        self.feedback_counter
    }
}

/// Per-opcode handler ABI. Each handler returns a `Step` describing what the
/// trampoline should do next.
pub type Handler = extern "C" fn(&mut DispatchState) -> Step;

/// Trampoline control-flow value. The trampoline keeps the active handler in a
/// local variable and only inspects this enum's discriminant.
pub enum Step {
    Continue(Handler),
    Done(Value),
    Error(VmError),
}

/// Tail of every fast-path handler: pick the next handler from `DISPATCH_TABLE`
/// indexed by the byte at the current `pc`, and return it inside
/// `Step::Continue`. The trampoline turns this into one indirect call per
/// opcode.
///
/// `dispatch_next!` is the *only* place in any handler body that should
/// reference `DISPATCH_TABLE` — Phase 1's acceptance criteria grep for this
/// invariant.
#[macro_export]
macro_rules! dispatch_next {
    ($state:expr) => {
        return $crate::vm::dispatch_state::Step::Continue(
            $crate::vm::dispatch_state::DISPATCH_TABLE[$state.next_opcode_byte() as usize],
        )
    };
}

/// Sized to cover every byte that could land in `bytes[pc]`. The first
/// `lyng_js_bytecode::OPCODE_COUNT` slots map to real handlers; the rest are
/// `op_stub` so an unknown byte fails cleanly rather than indexing past the
/// table.
pub const DISPATCH_TABLE_LEN: usize = 256;

/// Static dispatch table — one `Handler` per opcode byte value.
///
/// Handlers for the spike's five real opcodes (`op_move`, `op_load_undefined`,
/// `op_add`, `op_jump_back`, `op_return`) are wired at their `Opcode as u8`
/// indices via the `dispatch_handlers` module's `SPIKE_HANDLERS` mapping. All
/// other entries fall through to `op_stub`, which returns
/// `Step::Error(VmError::MissingActiveFrame)`.
pub static DISPATCH_TABLE: [Handler; DISPATCH_TABLE_LEN] =
    dispatch_handlers::build_dispatch_table();

/// Central trampoline. One indirect call per opcode. The hot path is the
/// `Step::Continue(next) => handler = next` arm; `Done` and `Error` are taken
/// once per script.
///
/// `#[inline(never)]` keeps this as a standalone symbol so `cargo asm` can
/// inspect the trampoline shape directly. Once Phase 1 wires it into the live
/// VM, the attribute can be reconsidered; for the spike, keeping the symbol
/// is the whole point.
#[inline(never)]
pub fn run_trampoline(state: &mut DispatchState) -> VmResult<Value> {
    let mut handler = DISPATCH_TABLE[state.first_opcode_byte() as usize];
    loop {
        match (handler)(state) {
            Step::Continue(next) => handler = next,
            Step::Done(value) => return Ok(value),
            Step::Error(error) => return Err(error),
        }
    }
}

#[cfg(test)]
mod trampoline_spike {
    use lyng_js_bytecode::Opcode;
    use lyng_js_types::Value;

    use super::{run_trampoline, DispatchState};

    /// Drive the spike through a hand-rolled 4-opcode program:
    /// `LdaUndefined R2; Move R3, R0; Add R2, R3, R1, slot=0; Return R2`.
    /// With `R0=5`, `R1=7`, `R2=R3=undefined` at entry, the SMI fast path
    /// should produce `Value::from_smi(12)` and bump `feedback_counter` to 1.
    #[test]
    fn smi_add_program_produces_expected_value() {
        let bytes = [
            Opcode::LdaUndefined as u8, 2,
            Opcode::Move as u8, 3, 0,
            Opcode::Add as u8, 2, 3, 1, 0, 0,
            Opcode::Return as u8, 2,
        ];
        let mut regs = [
            Value::from_smi(5),
            Value::from_smi(7),
            Value::undefined(),
            Value::undefined(),
        ];

        let mut state = DispatchState::new(&bytes, &mut regs);
        let result = run_trampoline(&mut state).expect("trampoline should run cleanly");

        assert_eq!(result.as_smi(), Some(12), "Add should sum R3=5 and R1=7");
        assert_eq!(state.feedback_counter(), 1, "Add should record feedback once");
        assert_eq!(
            regs[2].as_smi(),
            Some(12),
            "R2 should hold the Add result after Return reads it"
        );
        assert_eq!(regs[3].as_smi(), Some(5), "R3 should hold the moved R0 value");
    }
}
