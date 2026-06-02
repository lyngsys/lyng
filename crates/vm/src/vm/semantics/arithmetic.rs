//! Arithmetic family semantic bodies.
//!
//! Each `op_xxx_semantic` function implements the semantic effect of one
//! arithmetic-family opcode. The handler decodes operands, constructs
//! `OpXxxArgs`, calls the semantic body, and translates the returned
//! `SemanticOutcome` to `Step`.
//!
//! Family coverage (29 opcodes):
//! - Binary with feedback + SMI cache hit path: `Add`, `Sub`, `Mul`, `Mod`,
//!   `BitAnd`.
//! - SMI-immediate variants (`*Smi`): `AddSmi`, `SubSmi`, `MulSmi`, `ModSmi`,
//!   `BitAndSmi`.
//! - Binary delegating to a Vm helper: `Div`, `DivSmi`, `Exp`,
//!   `BitOr`, `BitXor`, `ShiftLeft`, `ShiftRight`, `UnsignedShiftRight`,
//!   `Equal`, `LessThan`, `LessEqual`, `GreaterThan`, `GreaterEqual`.
//! - `StrictEqual` — like binary-general but the helper does not need host/registry.
//! - `EqualZero` — unary, never throws.
//! - Unary via `handle_dispatch_result`: `Negate`, `BitNot`.
//! - Unary increment/decrement (`Increment`, `Decrement`) — writes a coerced
//!   numeric back to the source register and the post-update value to the destination.
//!
//! On the slow path, `Vm::finish_abc_value_result` advances PC, writes the
//! destination register, and records the feedback slot on success, or leaves PC at
//! the catch target on a caught abrupt completion. Either way the semantic returns
//! `SemanticOutcome::Continue { pc_advance: 0 }`.

use lyng_env::Agent;
use lyng_host::HostHooks;
use lyng_objects::NativeFunctionRegistry;
use lyng_types::{FeedbackSlotId, Value};

use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
use crate::error::VmResult;
use crate::frame::FrameView;
use crate::vm::Vm;
use crate::vm::dispatch::arithmetic::{smi_mod_result, smi_mul_result};
use crate::vm::dispatch_state::DispatchState;

// =====================================================================
// Shared shapes
// =====================================================================

/// Operands for two-register binary opcodes with feedback (Abc layout).
/// Used by Add / Sub / Mul / Div / Mod / Exp / Bit* / Shift* / Equal /
/// Less* / Greater*.
pub struct OpBinaryArgs {
    pub dst: u16,
    pub lhs: u16,
    pub rhs: u16,
    pub feedback_slot: Option<FeedbackSlotId>,
    pub instruction_len: u32,
}

/// Operands for register + i16-immediate binary opcodes (Abc layout, the
/// `c` field is an `i16` immediate decoded via `decode_smi_immediate`).
/// Used by `AddSmi` / `SubSmi` / `MulSmi` / `ModSmi` / `BitAndSmi`. `imm_raw` is the
/// raw `u16` operand value; the slow helper re-decodes it for symmetry.
pub struct OpBinarySmiArgs {
    pub dst: u16,
    pub lhs: u16,
    pub imm_raw: u16,
    pub feedback_slot: Option<FeedbackSlotId>,
    pub instruction_len: u32,
}

/// Operands for unary opcodes that delegate to a Vm helper returning
/// `VmResult<Value>` (Negate, `BitNot`). Only `dst`, `src`, and the
/// feedback slot matter; `c` is unused.
pub struct OpUnaryArgs {
    pub dst: u16,
    pub src: u16,
    pub feedback_slot: Option<FeedbackSlotId>,
    pub instruction_len: u32,
}

/// Operands for increment/decrement — same layout as `OpUnaryArgs` but
/// carries `src` separately because the Vm helper writes back a coerced
/// numeric to the source register.
pub type OpUpdateArgs = OpUnaryArgs;

/// Operands for `EqualZero` — single register input, no possible abrupt
/// completion, no slow path.
pub struct OpEqualZeroArgs {
    pub dst: u16,
    pub src: u16,
    pub feedback_slot: Option<FeedbackSlotId>,
    pub instruction_len: u32,
}

// =====================================================================
// Internal helpers shared between semantics
// =====================================================================

/// Slow-path tail for binary opcodes delegating to a `Vm::execute_*` helper.
/// `finish_abc_value_result` advances PC + writes register + records feedback
/// on success, or rewrites PC to the catch target on caught abrupt completion;
/// the returned `SemanticOutcome` always carries `pc_advance: 0`.
#[inline]
fn route_binary_result(
    state: &mut DispatchState<'_>,
    args: &OpBinaryArgs,
    result: VmResult<Value>,
) -> SemanticOutcome {
    let cfr = state.cfr;
    let code = state.code();
    let window = state.registers();
    let DispatchState {
        vm,
        agent,
        frame_depth,
        pc,
        ..
    } = state;
    let finish = vm.finish_abc_value_result(
        agent,
        *frame_depth,
        cfr,
        pc,
        code,
        window,
        args.instruction_len,
        args.feedback_slot,
        args.dst,
        result,
    );
    match finish {
        Ok(()) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

/// Slow-path tail for SMI-immediate binary opcodes. Identical shape to
/// `route_binary_result` but threads `OpBinarySmiArgs`.
#[inline]
fn route_binary_smi_result(
    state: &mut DispatchState<'_>,
    args: &OpBinarySmiArgs,
    result: VmResult<Value>,
) -> SemanticOutcome {
    let cfr = state.cfr;
    let code = state.code();
    let window = state.registers();
    let DispatchState {
        vm,
        agent,
        frame_depth,
        pc,
        ..
    } = state;
    let finish = vm.finish_abc_value_result(
        agent,
        *frame_depth,
        cfr,
        pc,
        code,
        window,
        args.instruction_len,
        args.feedback_slot,
        args.dst,
        result,
    );
    match finish {
        Ok(()) => SemanticOutcome::Continue { pc_advance: 0 },
        Err(error) => SemanticOutcome::ExitError { error },
    }
}

/// Shared body for binary opcodes that delegate to a Vm helper:
/// Div / Exp / `BitOr` / `BitXor` / Shift* / Equal / Less* / Greater* / `DivSmi`.
type BinaryVmOp = fn(
    &mut Vm,
    &mut Agent,
    &dyn HostHooks,
    &mut dyn NativeFunctionRegistry,
    FrameView,
    u16,
    u16,
) -> VmResult<Value>;

#[inline]
fn op_binary_general(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinaryArgs,
    op: BinaryVmOp,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        op(vm, agent, *host, &mut **registry, view, args.lhs, args.rhs)
    };
    route_binary_result(inner, &args, result)
}

// =====================================================================
// Add / Sub / Mul — two-register Abc with SMI cache hit path and feedback slot
// =====================================================================

pub fn op_add_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinaryArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let registers = inner.registers();
    let left = inner.vm.read_register_unchecked(registers, args.lhs);
    let right = inner.vm.read_register_unchecked(registers, args.rhs);
    if let (Some(l), Some(r)) = (left.as_smi(), right.as_smi())
        && let Some(v) = l.checked_add(r)
    {
        inner.vm.record_feedback_slot(code, args.feedback_slot);
        inner
            .vm
            .write_register_unchecked(registers, args.dst, Value::from_smi(v));
        return SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        };
    }
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_add_opcode(agent, *host, &mut **registry, view, args.lhs, args.rhs)
    };
    route_binary_result(inner, &args, result)
}

pub fn op_sub_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinaryArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let registers = inner.registers();
    let left = inner.vm.read_register_unchecked(registers, args.lhs);
    let right = inner.vm.read_register_unchecked(registers, args.rhs);
    if let (Some(l), Some(r)) = (left.as_smi(), right.as_smi())
        && let Some(v) = l.checked_sub(r)
    {
        inner.vm.record_feedback_slot(code, args.feedback_slot);
        inner
            .vm
            .write_register_unchecked(registers, args.dst, Value::from_smi(v));
        return SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        };
    }
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_sub_opcode(agent, *host, &mut **registry, view, args.lhs, args.rhs)
    };
    route_binary_result(inner, &args, result)
}

pub fn op_mul_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinaryArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let registers = inner.registers();
    let left = inner.vm.read_register_unchecked(registers, args.lhs);
    let right = inner.vm.read_register_unchecked(registers, args.rhs);
    if let (Some(l), Some(r)) = (left.as_smi(), right.as_smi())
        && let Some(v) = smi_mul_result(l, r)
    {
        inner.vm.record_feedback_slot(code, args.feedback_slot);
        inner.vm.write_register_unchecked(registers, args.dst, v);
        return SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        };
    }
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_mul_opcode(agent, *host, &mut **registry, view, args.lhs, args.rhs)
    };
    route_binary_result(inner, &args, result)
}

// =====================================================================
// AddSmi / SubSmi / MulSmi — register + i16 immediate (Abc-encoded)
// =====================================================================

pub fn op_add_smi_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinarySmiArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let registers = inner.registers();
    let left = inner.vm.read_register_unchecked(registers, args.lhs);
    let imm = i32::from(crate::vm::dispatch::arithmetic::decode_smi_immediate(
        args.imm_raw,
    ));
    if let Some(l) = left.as_smi()
        && let Some(v) = l.checked_add(imm)
    {
        inner.vm.record_feedback_slot(code, args.feedback_slot);
        inner
            .vm
            .write_register_unchecked(registers, args.dst, Value::from_smi(v));
        return SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        };
    }
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_add_smi_opcode(agent, *host, &mut **registry, view, args.lhs, args.imm_raw)
    };
    route_binary_smi_result(inner, &args, result)
}

pub fn op_sub_smi_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinarySmiArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let registers = inner.registers();
    let left = inner.vm.read_register_unchecked(registers, args.lhs);
    let imm = i32::from(crate::vm::dispatch::arithmetic::decode_smi_immediate(
        args.imm_raw,
    ));
    if let Some(l) = left.as_smi()
        && let Some(v) = l.checked_sub(imm)
    {
        inner.vm.record_feedback_slot(code, args.feedback_slot);
        inner
            .vm
            .write_register_unchecked(registers, args.dst, Value::from_smi(v));
        return SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        };
    }
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_sub_smi_opcode(agent, *host, &mut **registry, view, args.lhs, args.imm_raw)
    };
    route_binary_smi_result(inner, &args, result)
}

pub fn op_mul_smi_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinarySmiArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let registers = inner.registers();
    let left = inner.vm.read_register_unchecked(registers, args.lhs);
    let imm = i32::from(crate::vm::dispatch::arithmetic::decode_smi_immediate(
        args.imm_raw,
    ));
    if let Some(l) = left.as_smi()
        && let Some(v) = smi_mul_result(l, imm)
    {
        inner.vm.record_feedback_slot(code, args.feedback_slot);
        inner.vm.write_register_unchecked(registers, args.dst, v);
        return SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        };
    }
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_mul_smi_opcode(agent, *host, &mut **registry, view, args.lhs, args.imm_raw)
    };
    route_binary_smi_result(inner, &args, result)
}

// =====================================================================
// Div / DivSmi / Exp — always delegate, no inline SMI cache hit path
// =====================================================================

pub fn op_div_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinaryArgs,
) -> SemanticOutcome {
    op_binary_general(state, args, Vm::execute_div_opcode)
}

pub fn op_div_smi_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinarySmiArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_div_smi_opcode(agent, *host, &mut **registry, view, args.lhs, args.imm_raw)
    };
    route_binary_smi_result(inner, &args, result)
}

pub fn op_exp_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinaryArgs,
) -> SemanticOutcome {
    op_binary_general(state, args, Vm::execute_exp_opcode)
}

// =====================================================================
// Mod / ModSmi — SMI cache hit path via smi_mod_result, then delegate
// =====================================================================

pub fn op_mod_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinaryArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let registers = inner.registers();
    let left = inner.vm.read_register_unchecked(registers, args.lhs);
    let right = inner.vm.read_register_unchecked(registers, args.rhs);
    if let (Some(l), Some(r)) = (left.as_smi(), right.as_smi())
        && let Some(v) = smi_mod_result(l, r)
    {
        inner.vm.record_feedback_slot(code, args.feedback_slot);
        inner.vm.write_register_unchecked(registers, args.dst, v);
        return SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        };
    }
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_mod_opcode(agent, *host, &mut **registry, view, args.lhs, args.rhs)
    };
    route_binary_result(inner, &args, result)
}

pub fn op_mod_smi_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinarySmiArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let registers = inner.registers();
    let left = inner.vm.read_register_unchecked(registers, args.lhs);
    let imm = i32::from(crate::vm::dispatch::arithmetic::decode_smi_immediate(
        args.imm_raw,
    ));
    if let Some(l) = left.as_smi()
        && let Some(v) = smi_mod_result(l, imm)
    {
        inner.vm.record_feedback_slot(code, args.feedback_slot);
        inner.vm.write_register_unchecked(registers, args.dst, v);
        return SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        };
    }
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_mod_smi_opcode(agent, *host, &mut **registry, view, args.lhs, args.imm_raw)
    };
    route_binary_smi_result(inner, &args, result)
}

// =====================================================================
// Bitwise — BitAnd / BitAndSmi have inline SMI cache hit paths; the rest always
// delegate.
// =====================================================================

pub fn op_bit_and_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinaryArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let registers = inner.registers();
    let left = inner.vm.read_register_unchecked(registers, args.lhs);
    let right = inner.vm.read_register_unchecked(registers, args.rhs);
    if let (Some(l), Some(r)) = (left.as_smi(), right.as_smi()) {
        inner.vm.record_feedback_slot(code, args.feedback_slot);
        inner
            .vm
            .write_register_unchecked(registers, args.dst, Value::from_smi(l & r));
        return SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        };
    }
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_bitand_opcode(agent, *host, &mut **registry, view, args.lhs, args.rhs)
    };
    route_binary_result(inner, &args, result)
}

pub fn op_bit_and_smi_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinarySmiArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let registers = inner.registers();
    let left = inner.vm.read_register_unchecked(registers, args.lhs);
    let imm = i32::from(crate::vm::dispatch::arithmetic::decode_smi_immediate(
        args.imm_raw,
    ));
    if let Some(l) = left.as_smi() {
        inner.vm.record_feedback_slot(code, args.feedback_slot);
        inner
            .vm
            .write_register_unchecked(registers, args.dst, Value::from_smi(l & imm));
        return SemanticOutcome::Continue {
            pc_advance: args.instruction_len,
        };
    }
    let view = inner.frame_view();
    let result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.execute_bitand_smi_opcode(agent, *host, &mut **registry, view, args.lhs, args.imm_raw)
    };
    route_binary_smi_result(inner, &args, result)
}

pub fn op_bit_or_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinaryArgs,
) -> SemanticOutcome {
    op_binary_general(state, args, Vm::execute_bitor_opcode)
}

pub fn op_bit_xor_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinaryArgs,
) -> SemanticOutcome {
    op_binary_general(state, args, Vm::execute_bitxor_opcode)
}

pub fn op_shift_left_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinaryArgs,
) -> SemanticOutcome {
    op_binary_general(state, args, Vm::execute_shift_left_opcode)
}

pub fn op_shift_right_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinaryArgs,
) -> SemanticOutcome {
    op_binary_general(state, args, Vm::execute_shift_right_opcode)
}

pub fn op_unsigned_shift_right_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinaryArgs,
) -> SemanticOutcome {
    op_binary_general(state, args, Vm::execute_unsigned_shift_right_opcode)
}

// =====================================================================
// Comparisons — Equal / StrictEqual / EqualZero / Less* / Greater*
// =====================================================================

pub fn op_equal_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinaryArgs,
) -> SemanticOutcome {
    op_binary_general(state, args, Vm::execute_equal_opcode)
}

pub fn op_strict_equal_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinaryArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let view = inner.frame_view();
    let result = {
        let DispatchState { vm, agent, .. } = &mut *inner;
        vm.execute_strict_equal_opcode(agent, view, args.lhs, args.rhs)
    };
    route_binary_result(inner, &args, result)
}

pub fn op_less_than_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinaryArgs,
) -> SemanticOutcome {
    op_binary_general(state, args, Vm::execute_less_than_opcode)
}

pub fn op_less_equal_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinaryArgs,
) -> SemanticOutcome {
    op_binary_general(state, args, Vm::execute_less_equal_opcode)
}

pub fn op_greater_than_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinaryArgs,
) -> SemanticOutcome {
    op_binary_general(state, args, Vm::execute_greater_than_opcode)
}

pub fn op_greater_equal_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpBinaryArgs,
) -> SemanticOutcome {
    op_binary_general(state, args, Vm::execute_greater_equal_opcode)
}

/// `EqualZero` cannot raise — it returns a Boolean directly from a single
/// register read.
pub fn op_equal_zero_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpEqualZeroArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let value = inner
        .vm
        .execute_equal_zero_opcode(inner.frame_view(), args.src);
    inner.vm.record_feedback_slot(code, args.feedback_slot);
    let registers = inner.registers();
    inner
        .vm
        .write_register_unchecked(registers, args.dst, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

// =====================================================================
// Unary — Negate / BitNot
// =====================================================================

pub fn op_negate_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpUnaryArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let view = inner.frame_view();
    let negate_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.negate_value(agent, *host, &mut **registry, view, args.src)
    };
    let handled = inner.handle_dispatch_result(negate_result);
    let value = match handled {
        Ok(Some(v)) => v,
        Ok(None) => {
            return SemanticOutcome::Continue { pc_advance: 0 };
        }
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    inner.vm.record_feedback_slot(code, args.feedback_slot);
    let registers = inner.registers();
    inner
        .vm
        .write_register_unchecked(registers, args.dst, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub fn op_bit_not_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpUnaryArgs,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let view = inner.frame_view();
    let bit_not_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.bitwise_not_value(agent, *host, &mut **registry, view, args.src)
    };
    let handled = inner.handle_dispatch_result(bit_not_result);
    let value = match handled {
        Ok(Some(v)) => v,
        Ok(None) => {
            return SemanticOutcome::Continue { pc_advance: 0 };
        }
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    inner.vm.record_feedback_slot(code, args.feedback_slot);
    let registers = inner.registers();
    inner
        .vm
        .write_register_unchecked(registers, args.dst, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

/// Shared body for Increment / Decrement. The Vm helper returns a
/// `(numeric, value)` pair: `numeric` is the ToNumeric-coerced input that
/// must be written back to the source register (so subsequent reads see
/// a numeric value, not the original), and `value` is the post-update
/// result written to the destination.
fn op_update_register_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpUpdateArgs,
    increment: bool,
) -> SemanticOutcome {
    let inner = state.dispatch_state();
    let code = inner.code();
    let view = inner.frame_view();
    let update_result = {
        let DispatchState {
            vm,
            agent,
            host,
            registry,
            ..
        } = &mut *inner;
        vm.update_register_value(agent, *host, &mut **registry, view, args.src, increment)
    };
    let handled = inner.handle_dispatch_result(update_result);
    let (numeric, value) = match handled {
        Ok(Some(pair)) => pair,
        Ok(None) => {
            return SemanticOutcome::Continue { pc_advance: 0 };
        }
        Err(error) => return SemanticOutcome::ExitError { error },
    };
    let registers = inner.registers();
    inner
        .vm
        .write_register_unchecked(registers, args.src, numeric);
    inner.vm.record_feedback_slot(code, args.feedback_slot);
    inner
        .vm
        .write_register_unchecked(registers, args.dst, value);
    SemanticOutcome::Continue {
        pc_advance: args.instruction_len,
    }
}

pub fn op_increment_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpUpdateArgs,
) -> SemanticOutcome {
    op_update_register_semantic(state, args, true)
}

pub fn op_decrement_semantic(
    state: &mut LlIntDispatchState<'_, '_>,
    args: OpUpdateArgs,
) -> SemanticOutcome {
    op_update_register_semantic(state, args, false)
}
