use super::registers::absolute_register;
use super::values::decode_env_operand;
use super::dispatch::arithmetic::{decode_smi_immediate, smi_mod_result, smi_mul_result};
use super::{
    code_index, Agent, CallRange, CodeRef, FrameRecord, HostHooks, NativeFunctionRegistry,
    Opcode, ThisState, Value, Vm, VmDebugSafepointKind, VmError, VmResult,
};
use lyng_js_ops::{errors, read};
use lyng_js_types::{AbruptCompletion, FeedbackSlotId, PropertyKey};

mod arithmetic;
mod property;

#[cfg(test)]
mod tests {
    #[test]
    fn dispatch_loop_does_not_materialize_instructions_on_hot_path() {
        let source = include_str!("dispatch.rs");
        let run_loop = source
            .split("\n    fn run_dispatch_loop")
            .nth(1)
            .expect("dispatch loop should stay in this module");

        assert!(
            !run_loop.contains("instruction_at("),
            "hot dispatch should not decode through InstalledFunction::instruction_at"
        );
        assert!(
            !run_loop.contains("decode_instruction_bytes"),
            "hot dispatch should not construct Instruction through decode_instruction_bytes"
        );
        assert!(
            !run_loop.contains("without_feedback_slot"),
            "hot dispatch should decode profiled envelopes without Instruction helpers"
        );
        assert!(
            !run_loop.contains("instruction.encoded_len()"),
            "hot dispatch should advance from opcode byte layout, not Instruction::encoded_len"
        );
        assert!(
            !run_loop.contains("decode_dispatch_instruction("),
            "hot dispatch should inline opcode + operand decode, not call out to a centralized decoder"
        );
        assert!(
            !run_loop.contains("DispatchDecode"),
            "hot dispatch should not construct a DispatchDecode struct per opcode"
        );
    }

    #[test]
    fn dispatch_loop_keeps_program_counter_in_local_state() {
        let source = include_str!("dispatch.rs");
        let run_loop = source
            .split("\n    fn run_dispatch_loop")
            .nth(1)
            .expect("dispatch loop should stay in this module");

        assert!(
            run_loop.contains("DispatchState"),
            "dispatch should make active frame state explicit"
        );
        assert!(
            !run_loop.contains("current_instruction_len"),
            "dispatch should not share instruction length through Vm state"
        );
        assert!(
            !run_loop.contains("self.advance_instruction("),
            "dispatch should advance the active PC through local frame state"
        );
        assert!(
            !run_loop.contains("self.jump_by("),
            "dispatch should branch through local frame state"
        );
        assert!(
            !run_loop.contains("*self\n                    .frames\n                    .last()"),
            "dispatch should not copy FrameRecord from self.frames.last() per opcode"
        );
    }

    #[test]
    fn dispatch_loop_inlines_hot_abc_handlers() {
        let source = include_str!("dispatch.rs");
        let run_loop = source
            .split("\n    fn run_dispatch_loop")
            .nth(1)
            .expect("dispatch loop should stay in this module");

        assert!(
            !run_loop.contains(".execute_abc_value_opcode("),
            "hot ABC dispatch should not rematch opcode families through execute_abc_value_opcode"
        );
    }

    #[test]
    fn move_arm_uses_direct_register_window_access() {
        let source = include_str!("dispatch.rs");
        let run_loop = source
            .split("\n    fn run_dispatch_loop")
            .nth(1)
            .expect("dispatch loop should stay in this module");
        let move_arm = run_loop
            .split("Opcode::Move => {")
            .nth(1)
            .and_then(|tail| tail.split("Opcode::Add").next())
            .expect("Move arm should stay directly before Add-family arms");

        assert!(
            !move_arm.contains("self.read_register(") && !move_arm.contains("self.write_register("),
            "Move should use direct checked register-window access in the hot dispatch arm"
        );
    }

    #[test]
    fn load_const_arm_uses_direct_register_window_access() {
        let source = include_str!("dispatch.rs");
        let run_loop = source
            .split("\n    fn run_dispatch_loop")
            .nth(1)
            .expect("dispatch loop should stay in this module");
        let load_const_arm = run_loop
            .split("Opcode::LoadConst | Opcode::LoadConst8 => {")
            .nth(1)
            .and_then(|tail| tail.split("Opcode::LoadLocal0").next())
            .expect("LoadConst arm should stay directly before local load arms");

        assert!(
            !load_const_arm.contains("self.write_register("),
            "LoadConst should write through direct checked register-window access"
        );
    }
}

#[derive(Clone, Copy)]
pub(in crate::vm) struct DispatchState {
    frame_depth: usize,
    code: CodeRef,
    frame: FrameRecord,
}

impl DispatchState {
    fn from_active_frame(vm: &Vm) -> Self {
        let frame_depth = vm.frames.len();
        let frame = vm
            .frames
            .last()
            .copied()
            .expect("evaluation should install one active frame");
        Self {
            frame_depth,
            code: frame.code(),
            frame,
        }
    }

    const fn frame_depth(self) -> usize {
        self.frame_depth
    }

    const fn frame(&self) -> &FrameRecord {
        &self.frame
    }

    fn active_in(self, vm: &Vm) -> bool {
        self.frame.code() == self.code
            && vm.frames.len() == self.frame_depth
            && vm
                .frames
                .last()
                .is_some_and(|frame| frame.code() == self.code)
    }
}

#[derive(Clone, Copy)]
enum DispatchOperandForm {
    Abc,
    Abx,
    Abx8,
    Ax,
    Ax8,
    Local,
    Accumulator,
    AccumulatorByte,
    AccumulatorRegister,
    CallRange,
}

const fn sign_extend_i24(bytes: [u8; 3]) -> i32 {
    let sign = if bytes[2] & 0x80 == 0 { 0 } else { 0xff };
    i32::from_le_bytes([bytes[0], bytes[1], bytes[2], sign])
}

#[inline]
pub(in crate::vm) const fn advance_dispatch_frame(frame: &mut FrameRecord, encoded_len: u32) {
    let next = frame
        .instruction_offset()
        .checked_add(encoded_len)
        .expect("instruction offset should stay within u32");
    frame.set_instruction_offset(next);
}

#[inline]
pub(in crate::vm) fn jump_dispatch_frame(
    frame: &mut FrameRecord,
    encoded_len: u32,
    delta: i32,
) -> VmResult<()> {
    let instruction_offset = frame.instruction_offset();
    let next = i64::from(instruction_offset) + i64::from(encoded_len) + i64::from(delta);
    if next < 0 {
        return Err(VmError::InvalidJumpTarget {
            code: frame.code(),
            instruction_offset,
            target_offset: next,
        });
    }
    frame.set_instruction_offset(u32::try_from(next).map_err(|_| VmError::InvalidJumpTarget {
        code: frame.code(),
        instruction_offset,
        target_offset: next,
    })?);
    Ok(())
}

#[inline]
pub(in crate::vm) const fn next_dispatch_instruction_offset(
    frame: &FrameRecord,
    encoded_len: u32,
) -> u32 {
    frame
        .instruction_offset()
        .checked_add(encoded_len)
        .expect("instruction offset should stay within u32")
}

#[inline(always)]
#[allow(
    clippy::too_many_lines,
    clippy::inline_always,
    reason = "the dispatch decoder mirrors the bytecode operand layout table; inline(always) is required so the form match folds into the per-opcode dispatch in the hot loop"
)]
const fn dispatch_operand_form(opcode: Opcode) -> DispatchOperandForm {
    match opcode {
        Opcode::Nop
        | Opcode::TypeOf
        | Opcode::Jump
        | Opcode::LoopHeader
        | Opcode::Return
        | Opcode::ReturnUndefined
        | Opcode::PushClosureEnv
        | Opcode::PopClosureEnv
        | Opcode::PushWithEnv
        | Opcode::PopWithEnv
        | Opcode::Throw
        | Opcode::EnterHandler
        | Opcode::LeaveHandler
        | Opcode::LoadException
        | Opcode::SuspendGeneratorStart
        | Opcode::Yield
        | Opcode::Await
        | Opcode::LoadResumeKind
        | Opcode::LoadResumeValue => DispatchOperandForm::Ax,
        Opcode::LoadUndefined
        | Opcode::LoadUninitializedLexical
        | Opcode::LoadNull
        | Opcode::LoadTrue
        | Opcode::LoadFalse
        | Opcode::LoadZero
        | Opcode::LoadOne
        | Opcode::LoadSmi
        | Opcode::LoadConst
        | Opcode::LoadEnvSlot
        | Opcode::StoreEnvSlot
        | Opcode::AssignEnvSlot
        | Opcode::LoadGlobal
        | Opcode::StoreGlobal
        | Opcode::AssignGlobal
        | Opcode::DeleteGlobal
        | Opcode::LoadName
        | Opcode::ResolveName
        | Opcode::ResolveGlobal
        | Opcode::AssignName
        | Opcode::AssignVariableName
        | Opcode::DeleteName
        | Opcode::CaptureName
        | Opcode::LoadCapturedName
        | Opcode::LoadCapturedNameThis
        | Opcode::AssignCapturedName
        | Opcode::LoadThis
        | Opcode::LoadCallee
        | Opcode::LoadNewTarget
        | Opcode::EnterEnvScope
        | Opcode::LeaveEnvScope
        | Opcode::JumpIfTrue
        | Opcode::JumpIfFalse
        | Opcode::CreateObject
        | Opcode::CreateArray
        | Opcode::CheckObjectCoercible
        | Opcode::ThrowIfUninitialized
        | Opcode::CreateClosure
        | Opcode::CloseForIn
        | Opcode::CloseIterator => DispatchOperandForm::Abx,
        Opcode::LoadSmi8 | Opcode::LoadConst8 | Opcode::JumpIfTrue8 | Opcode::JumpIfFalse8 => {
            DispatchOperandForm::Abx8
        }
        Opcode::Jump8 => DispatchOperandForm::Ax8,
        Opcode::LoadLocal0
        | Opcode::LoadLocal1
        | Opcode::LoadLocal2
        | Opcode::LoadLocal3
        | Opcode::StoreLocal0
        | Opcode::StoreLocal1
        | Opcode::StoreLocal2
        | Opcode::StoreLocal3 => DispatchOperandForm::Local,
        Opcode::LdaUndefined
        | Opcode::LdaNull
        | Opcode::LdaTrue
        | Opcode::LdaFalse
        | Opcode::LdaZero
        | Opcode::LdaOne
        | Opcode::Star0
        | Opcode::Star1
        | Opcode::Star2
        | Opcode::Star3
        | Opcode::Star4
        | Opcode::Star5
        | Opcode::Star6
        | Opcode::Star7 => DispatchOperandForm::Accumulator,
        Opcode::LdaSmi8 | Opcode::LdaConst8 => DispatchOperandForm::AccumulatorByte,
        Opcode::Ldar => DispatchOperandForm::AccumulatorRegister,
        Opcode::Call | Opcode::TailCall | Opcode::Construct => DispatchOperandForm::CallRange,
        Opcode::Move
        | Opcode::Add
        | Opcode::AddSmi
        | Opcode::Sub
        | Opcode::SubSmi
        | Opcode::Mul
        | Opcode::MulSmi
        | Opcode::Div
        | Opcode::Mod
        | Opcode::DivSmi
        | Opcode::ModSmi
        | Opcode::Exp
        | Opcode::BitOr
        | Opcode::BitXor
        | Opcode::BitAnd
        | Opcode::BitAndSmi
        | Opcode::BitNot
        | Opcode::ShiftLeft
        | Opcode::ShiftRight
        | Opcode::UnsignedShiftRight
        | Opcode::Negate
        | Opcode::Increment
        | Opcode::Decrement
        | Opcode::Equal
        | Opcode::StrictEqual
        | Opcode::EqualZero
        | Opcode::LessThan
        | Opcode::LessEqual
        | Opcode::GreaterThan
        | Opcode::GreaterEqual
        | Opcode::InstanceOf
        | Opcode::In
        | Opcode::DefineNamedProperty
        | Opcode::DefineKeyedProperty
        | Opcode::StoreDenseElement
        | Opcode::LoadDenseElement
        | Opcode::GetNamedProperty
        | Opcode::SetNamedProperty
        | Opcode::AssignNamedProperty
        | Opcode::StrictAssignNamedProperty
        | Opcode::GetKeyedProperty
        | Opcode::SetKeyedProperty
        | Opcode::AssignKeyedProperty
        | Opcode::StrictAssignKeyedProperty
        | Opcode::DeleteProperty
        | Opcode::CopyDataProperties
        | Opcode::SetFunctionName
        | Opcode::ToPropertyKey
        | Opcode::Call0
        | Opcode::Call1
        | Opcode::Call2
        | Opcode::Call3
        | Opcode::CallMethod
        | Opcode::CreateForIn
        | Opcode::AdvanceForIn
        | Opcode::CreateIterator
        | Opcode::AdvanceIterator
        | Opcode::DelegateYield => DispatchOperandForm::Abc,
        Opcode::Wide | Opcode::ExtraWide => {
            panic!("prefix opcodes should decode their semantic opcode first")
        }
        _ => panic!("profiled opcodes should decode to their base opcode first"),
    }
}

impl Vm {
    #[inline]
    pub(in crate::vm) fn sync_dispatch_frame(&mut self, frame_depth: usize, frame: FrameRecord) {
        let Some(index) = frame_depth.checked_sub(1) else {
            return;
        };
        if let Some(slot) = self.frames.get_mut(index) {
            *slot = frame;
        }
    }

    #[inline]
    pub(in crate::vm) fn refresh_dispatch_frame(
        &self,
        frame_depth: usize,
        frame: &mut FrameRecord,
    ) {
        let Some(index) = frame_depth.checked_sub(1) else {
            return;
        };
        if let Some(stacked) = self.frames.get(index).copied() {
            *frame = stacked;
        }
    }

    pub(in crate::vm) fn handle_dispatch_result<T>(
        &mut self,
        agent: &mut Agent,
        frame_depth: usize,
        frame: &mut FrameRecord,
        result: VmResult<T>,
    ) -> VmResult<Option<T>> {
        match result {
            Ok(value) => Ok(Some(value)),
            Err(VmError::Abrupt(AbruptCompletion::Throw(value))) => {
                self.sync_dispatch_frame(frame_depth, *frame);
                if self.transfer_to_exception_handler(agent, value)? {
                    self.refresh_dispatch_frame(frame_depth, frame);
                    Ok(None)
                } else {
                    Err(VmError::Abrupt(AbruptCompletion::Throw(value)))
                }
            }
            Err(error) => Err(error),
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "hot ABC dispatch keeps frame state, feedback, and target register explicit at call sites"
    )]
    fn finish_abc_value_result(
        &mut self,
        agent: &mut Agent,
        frame_depth: usize,
        frame: &mut FrameRecord,
        instruction_len: u32,
        feedback_slot: Option<FeedbackSlotId>,
        target_register: u16,
        result: VmResult<Value>,
    ) -> VmResult<()> {
        let Some(value) = self.handle_dispatch_result(agent, frame_depth, frame, result)? else {
            return Ok(());
        };
        self.record_feedback_slot(frame.code(), feedback_slot);
        let target = absolute_register(frame.registers(), target_register);
        self.register_stack[target] = value;
        advance_dispatch_frame(frame, instruction_len);
        Ok(())
    }

    pub(super) fn run(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> VmResult<Value> {
        // Two const-generic switches at the dispatcher boundary: opcode-counter recording
        // and debugger safepoint polling. Both are zero-overhead when disabled because the
        // const propagates through `run_dispatch_loop` and LLVM strips the dead branches.
        match (
            self.opcode_dispatch_counts_enabled(),
            self.debug_poll_enabled(),
        ) {
            (false, false) => self.run_dispatch_loop::<false, false>(agent, host, registry),
            (false, true) => self.run_dispatch_loop::<false, true>(agent, host, registry),
            (true, false) => self.run_dispatch_loop::<true, false>(agent, host, registry),
            (true, true) => self.run_dispatch_loop::<true, true>(agent, host, registry),
        }
    }

    /// Inner monomorphized dispatch loop.
    ///
    /// Single flat-`match opcode` dispatch over the dense `#[repr(u8)]`
    /// [`Opcode`] enum, dense across 0..=141. LLVM synthesizes one indexed jump table
    /// (verified via `cargo asm`) so each opcode landing pad is reached with one
    /// register-indirect jump from the bottom of the loop. Const generics gate the
    /// counter-recording and debugger-poll work — disabled paths become unconditional
    /// `continue`s.
    #[expect(
        clippy::too_many_lines,
        reason = "main interpreter dispatch keeps opcode fetch and branch handling in one profiler-friendly loop"
    )]
    #[allow(
        clippy::collapsible_if,
        reason = "SMI fast paths keep the outer SMI tag check and the inner overflow check on separate lines so the cold fallthrough is obvious"
    )]
    fn run_dispatch_loop<const COUNT_OPCODES: bool, const DEBUG: bool>(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> VmResult<Value> {
        loop {
            let mut state = DispatchState::from_active_frame(self);
            let code = state.frame().code();
            let installed = self
                .installed
                .get(code_index(code))
                .and_then(Option::as_ref)
                .cloned()
                .ok_or(VmError::MissingInstalledCode(code))?;

            loop {
                if !state.active_in(self) {
                    break;
                }
                let instruction_offset = state.frame().instruction_offset();

                // === Track A: Inline opcode + operand decode ===
                // The previous shape called `decode_dispatch_instruction` per opcode and
                // returned a typed decode struct. On Richards/Crypto that function was
                // 26-40% of total time. We inline it here so LLVM can fuse the decode with
                // the subsequent opcode dispatch into one tight loop body.
                let pc_usize = usize::try_from(instruction_offset).map_err(|_| {
                    VmError::InstructionOutOfBounds {
                        code,
                        instruction_offset,
                    }
                })?;
                let bytes = installed
                    .function
                    .instruction_bytes()
                    .get(pc_usize..)
                    .ok_or(VmError::InstructionOutOfBounds {
                        code,
                        instruction_offset,
                    })?;
                let raw_first = *bytes.first().ok_or(VmError::InstructionOutOfBounds {
                    code,
                    instruction_offset,
                })?;
                let first = Opcode::from_byte(raw_first).ok_or(
                    VmError::InstructionOutOfBounds {
                        code,
                        instruction_offset,
                    },
                )?;
                // Hot path: no prefix. Wide/ExtraWide are essentially zero share on real
                // workloads; we still handle them via the same inline path.
                let (prefix, semantic_opcode) = if first.is_prefix() {
                    let raw_semantic = *bytes.get(1).ok_or(
                        VmError::InstructionOutOfBounds {
                            code,
                            instruction_offset,
                        },
                    )?;
                    let semantic = Opcode::from_byte(raw_semantic).ok_or(
                        VmError::InstructionOutOfBounds {
                            code,
                            instruction_offset,
                        },
                    )?;
                    if semantic.is_prefix() {
                        return Err(VmError::InstructionOutOfBounds {
                            code,
                            instruction_offset,
                        });
                    }
                    (Some(first), semantic)
                } else {
                    (None, first)
                };
                let opcode = semantic_opcode.profiled_base_opcode();
                let is_profiled = semantic_opcode.is_profiled();

                if COUNT_OPCODES {
                    self.record_opcode_dispatch(opcode);
                }
                #[cfg(debug_assertions)]
                self.assert_deopt_safepoint_state(agent, state.frame(), installed.as_ref());

                // Decode operands inline by form. `dispatch_operand_form` is a const fn
                // marked `#[inline(always)]` (see below); LLVM lowers it to a constant per
                // arm of the outer opcode match because the opcode is known at this point.
                let mut a: u16 = 0;
                let mut b: u16 = 0;
                let mut c: u16 = 0;
                let mut bx: u32 = 0;
                let mut ax: i32 = 0;
                let mut call_range: Option<CallRange> = None;
                let operand_end: usize;
                #[allow(
                    clippy::match_same_arms,
                    reason = "operand-form arms stay grouped per semantic form even when bodies are byte-identical"
                )]
                match dispatch_operand_form(opcode) {
                    DispatchOperandForm::Abc => {
                        if prefix.is_some() {
                            let [_, _, a_low, b_low, c_low, a_high, b_high, c_high, ..] =
                                bytes
                            else {
                                return Err(VmError::InstructionOutOfBounds {
                                    code,
                                    instruction_offset,
                                });
                            };
                            a = u16::from_le_bytes([*a_low, *a_high]);
                            b = u16::from_le_bytes([*b_low, *b_high]);
                            c = u16::from_le_bytes([*c_low, *c_high]);
                            operand_end = 8;
                        } else {
                            let [_, ra, rb, rc, ..] = bytes else {
                                return Err(VmError::InstructionOutOfBounds {
                                    code,
                                    instruction_offset,
                                });
                            };
                            a = u16::from(*ra);
                            b = u16::from(*rb);
                            c = u16::from(*rc);
                            operand_end = 4;
                        }
                    }
                    DispatchOperandForm::Abx => {
                        if let Some(prefix) = prefix {
                            let [_, _, a_low, bx0, bx1, a_high, bx2, bx3, ..] = bytes else {
                                return Err(VmError::InstructionOutOfBounds {
                                    code,
                                    instruction_offset,
                                });
                            };
                            let bx3 = if prefix == Opcode::ExtraWide { *bx3 } else { 0 };
                            a = u16::from_le_bytes([*a_low, *a_high]);
                            bx = u32::from_le_bytes([*bx0, *bx1, *bx2, bx3]);
                            operand_end = 8;
                        } else {
                            let [_, ra, bx_low, bx_high, ..] = bytes else {
                                return Err(VmError::InstructionOutOfBounds {
                                    code,
                                    instruction_offset,
                                });
                            };
                            a = u16::from(*ra);
                            bx = u32::from(u16::from_le_bytes([*bx_low, *bx_high]));
                            operand_end = 4;
                        }
                    }
                    DispatchOperandForm::Abx8 => {
                        let [_, ra, rbx, ..] = bytes else {
                            return Err(VmError::InstructionOutOfBounds {
                                code,
                                instruction_offset,
                            });
                        };
                        a = u16::from(*ra);
                        bx = u32::from(*rbx);
                        operand_end = 3;
                    }
                    DispatchOperandForm::Ax => {
                        let [_, first_byte, second, third, ..] = bytes else {
                            return Err(VmError::InstructionOutOfBounds {
                                code,
                                instruction_offset,
                            });
                        };
                        ax = sign_extend_i24([*first_byte, *second, *third]);
                        operand_end = 4;
                    }
                    DispatchOperandForm::Ax8 => {
                        let [_, raw_ax, ..] = bytes else {
                            return Err(VmError::InstructionOutOfBounds {
                                code,
                                instruction_offset,
                            });
                        };
                        ax = i32::from(i8::from_le_bytes([*raw_ax]));
                        operand_end = 2;
                    }
                    DispatchOperandForm::Local => {
                        let [_, register, ..] = bytes else {
                            return Err(VmError::InstructionOutOfBounds {
                                code,
                                instruction_offset,
                            });
                        };
                        a = u16::from(*register);
                        operand_end = 2;
                    }
                    DispatchOperandForm::Accumulator => {
                        operand_end = 1;
                    }
                    DispatchOperandForm::AccumulatorByte => {
                        let [_, operand, ..] = bytes else {
                            return Err(VmError::InstructionOutOfBounds {
                                code,
                                instruction_offset,
                            });
                        };
                        bx = u32::from(*operand);
                        operand_end = 2;
                    }
                    DispatchOperandForm::AccumulatorRegister => {
                        let [_, register, ..] = bytes else {
                            return Err(VmError::InstructionOutOfBounds {
                                code,
                                instruction_offset,
                            });
                        };
                        a = u16::from(*register);
                        operand_end = 2;
                    }
                    DispatchOperandForm::CallRange => {
                        let [_, ra, rb, rc, count_low, count_high, base_low, base_high, ..] =
                            bytes
                        else {
                            return Err(VmError::InstructionOutOfBounds {
                                code,
                                instruction_offset,
                            });
                        };
                        a = u16::from(*ra);
                        b = u16::from(*rb);
                        c = u16::from(*rc);
                        call_range = Some(CallRange::new(
                            u16::from_le_bytes([*base_low, *base_high]),
                            u16::from_le_bytes([*count_low, *count_high]),
                        ));
                        operand_end = 8;
                    }
                }
                let (feedback_slot, instruction_len) = if is_profiled {
                    let [slot_low, slot_high, ..] =
                        bytes.get(operand_end..).ok_or(VmError::InstructionOutOfBounds {
                            code,
                            instruction_offset,
                        })?
                    else {
                        return Err(VmError::InstructionOutOfBounds {
                            code,
                            instruction_offset,
                        });
                    };
                    let raw_slot = u16::from_le_bytes([*slot_low, *slot_high]);
                    let slot = FeedbackSlotId::from_raw(u32::from(raw_slot)).ok_or(
                        VmError::InstructionOutOfBounds {
                            code,
                            instruction_offset,
                        },
                    )?;
                    let len = u32::try_from(operand_end + 2).map_err(|_| {
                        VmError::InstructionOutOfBounds {
                            code,
                            instruction_offset,
                        }
                    })?;
                    (Some(slot), len)
                } else {
                    let len = u32::try_from(operand_end).map_err(|_| {
                        VmError::InstructionOutOfBounds {
                            code,
                            instruction_offset,
                        }
                    })?;
                    (None, len)
                };

                let frame_depth = state.frame_depth();
                let frame = &mut state.frame;

                // Flat dispatch on the dense `#[repr(u8)]` Opcode enum so LLVM can emit
                // one indexed jump table for the entire opcode space. Each arm sees
                // `a/b/c`, `bx`, or `ax` from the outer-scope locals above depending on
                // which form it belongs to.
                #[allow(
                    clippy::match_same_arms,
                    reason = "opcode families stay grouped even when marker opcodes share dispatch behavior"
                )]
                match opcode {
                    Opcode::Move => {
                        let registers = frame.registers();
                        let source = absolute_register(registers, b);
                        let target = absolute_register(registers, a);
                        let value = self.register_stack[source];
                        self.register_stack[target] = value;
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::Add => {
                        let registers = frame.registers();
                        let left = self.register_stack[absolute_register(registers, b)];
                        let right = self.register_stack[absolute_register(registers, c)];
                        if let (Some(l), Some(r)) = (left.as_smi(), right.as_smi()) {
                            if let Some(v) = l.checked_add(r) {
                                self.record_feedback_slot(frame.code(), feedback_slot);
                                self.register_stack[absolute_register(registers, a)] =
                                    Value::from_smi(v);
                                advance_dispatch_frame(frame, instruction_len);
                                continue;
                            }
                        }
                        // Cold path: ToPrimitive, BigInt, f64, etc.
                        let result =
                            self.execute_add_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::AddSmi => {
                        let registers = frame.registers();
                        let left = self.register_stack[absolute_register(registers, b)];
                        let imm = i32::from(decode_smi_immediate(c));
                        if let Some(l) = left.as_smi() {
                            if let Some(v) = l.checked_add(imm) {
                                self.record_feedback_slot(frame.code(), feedback_slot);
                                self.register_stack[absolute_register(registers, a)] =
                                    Value::from_smi(v);
                                advance_dispatch_frame(frame, instruction_len);
                                continue;
                            }
                        }
                        // Cold path: ToPrimitive, BigInt, f64, etc.
                        let result =
                            self.execute_add_smi_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::Sub => {
                        let registers = frame.registers();
                        let left = self.register_stack[absolute_register(registers, b)];
                        let right = self.register_stack[absolute_register(registers, c)];
                        if let (Some(l), Some(r)) = (left.as_smi(), right.as_smi()) {
                            if let Some(v) = l.checked_sub(r) {
                                self.record_feedback_slot(frame.code(), feedback_slot);
                                self.register_stack[absolute_register(registers, a)] =
                                    Value::from_smi(v);
                                advance_dispatch_frame(frame, instruction_len);
                                continue;
                            }
                        }
                        // Cold path: ToPrimitive, BigInt, f64, etc.
                        let result =
                            self.execute_sub_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::SubSmi => {
                        let registers = frame.registers();
                        let left = self.register_stack[absolute_register(registers, b)];
                        let imm = i32::from(decode_smi_immediate(c));
                        if let Some(l) = left.as_smi() {
                            if let Some(v) = l.checked_sub(imm) {
                                self.record_feedback_slot(frame.code(), feedback_slot);
                                self.register_stack[absolute_register(registers, a)] =
                                    Value::from_smi(v);
                                advance_dispatch_frame(frame, instruction_len);
                                continue;
                            }
                        }
                        // Cold path: ToPrimitive, BigInt, f64, etc.
                        let result =
                            self.execute_sub_smi_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::Mul => {
                        let registers = frame.registers();
                        let left = self.register_stack[absolute_register(registers, b)];
                        let right = self.register_stack[absolute_register(registers, c)];
                        if let (Some(l), Some(r)) = (left.as_smi(), right.as_smi()) {
                            if let Some(v) = smi_mul_result(l, r) {
                                self.record_feedback_slot(frame.code(), feedback_slot);
                                self.register_stack[absolute_register(registers, a)] = v;
                                advance_dispatch_frame(frame, instruction_len);
                                continue;
                            }
                        }
                        // Cold path: ToPrimitive, BigInt, f64, etc.
                        let result =
                            self.execute_mul_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::MulSmi => {
                        let registers = frame.registers();
                        let left = self.register_stack[absolute_register(registers, b)];
                        let imm = i32::from(decode_smi_immediate(c));
                        if let Some(l) = left.as_smi() {
                            if let Some(v) = smi_mul_result(l, imm) {
                                self.record_feedback_slot(frame.code(), feedback_slot);
                                self.register_stack[absolute_register(registers, a)] = v;
                                advance_dispatch_frame(frame, instruction_len);
                                continue;
                            }
                        }
                        // Cold path: ToPrimitive, BigInt, f64, etc.
                        let result =
                            self.execute_mul_smi_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::Div => {
                        let result = self.execute_div_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::DivSmi => {
                        let result =
                            self.execute_div_smi_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::Mod => {
                        let registers = frame.registers();
                        let left = self.register_stack[absolute_register(registers, b)];
                        let right = self.register_stack[absolute_register(registers, c)];
                        if let (Some(l), Some(r)) = (left.as_smi(), right.as_smi()) {
                            if let Some(v) = smi_mod_result(l, r) {
                                self.record_feedback_slot(frame.code(), feedback_slot);
                                self.register_stack[absolute_register(registers, a)] = v;
                                advance_dispatch_frame(frame, instruction_len);
                                continue;
                            }
                        }
                        // Cold path: ToPrimitive, BigInt, f64, etc.
                        let result =
                            self.execute_mod_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::ModSmi => {
                        let registers = frame.registers();
                        let left = self.register_stack[absolute_register(registers, b)];
                        let imm = i32::from(decode_smi_immediate(c));
                        if let Some(l) = left.as_smi() {
                            if let Some(v) = smi_mod_result(l, imm) {
                                self.record_feedback_slot(frame.code(), feedback_slot);
                                self.register_stack[absolute_register(registers, a)] = v;
                                advance_dispatch_frame(frame, instruction_len);
                                continue;
                            }
                        }
                        // Cold path: ToPrimitive, BigInt, f64, etc.
                        let result =
                            self.execute_mod_smi_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::Exp => {
                        let result = self.execute_exp_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::BitOr => {
                        let result = self.execute_bitor_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::BitAnd => {
                        let registers = frame.registers();
                        let left = self.register_stack[absolute_register(registers, b)];
                        let right = self.register_stack[absolute_register(registers, c)];
                        if let (Some(l), Some(r)) = (left.as_smi(), right.as_smi()) {
                            let v = l & r;
                            self.record_feedback_slot(frame.code(), feedback_slot);
                            self.register_stack[absolute_register(registers, a)] =
                                Value::from_smi(v);
                            advance_dispatch_frame(frame, instruction_len);
                            continue;
                        }
                        // Cold path: ToPrimitive, BigInt, f64, etc.
                        let result =
                            self.execute_bitand_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::BitAndSmi => {
                        let registers = frame.registers();
                        let left = self.register_stack[absolute_register(registers, b)];
                        let imm = i32::from(decode_smi_immediate(c));
                        if let Some(l) = left.as_smi() {
                            let v = l & imm;
                            self.record_feedback_slot(frame.code(), feedback_slot);
                            self.register_stack[absolute_register(registers, a)] =
                                Value::from_smi(v);
                            advance_dispatch_frame(frame, instruction_len);
                            continue;
                        }
                        // Cold path: ToPrimitive, BigInt, f64, etc.
                        let result =
                            self.execute_bitand_smi_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::BitXor => {
                        let result = self.execute_bitxor_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::ShiftLeft => {
                        let result =
                            self.execute_shift_left_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::ShiftRight => {
                        let result =
                            self.execute_shift_right_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::UnsignedShiftRight => {
                        let result = self.execute_unsigned_shift_right_opcode(
                            agent, host, registry, frame, b, c,
                        );
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::Equal => {
                        let result = self.execute_equal_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::StrictEqual => {
                        let result = self.execute_strict_equal_opcode(agent, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::EqualZero => {
                        let result = Ok(self.execute_equal_zero_opcode(frame, b));
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::LessThan => {
                        let result =
                            self.execute_less_than_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::LessEqual => {
                        let result =
                            self.execute_less_equal_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::GreaterThan => {
                        let result =
                            self.execute_greater_than_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::GreaterEqual => {
                        let result =
                            self.execute_greater_equal_opcode(agent, host, registry, frame, b, c);
                        self.finish_abc_value_result(
                            agent,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            result,
                        )?;
                    }
                    Opcode::In => {
                        self.execute_in_opcode(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            instruction_len,
                            a,
                            b,
                            c,
                        )?;
                    }
                    Opcode::Negate => {
                        let negate_result = self.negate_value(agent, host, registry, frame, b);
                        let Some(value) =
                            self.handle_dispatch_result(agent, frame_depth, frame, negate_result)?
                        else {
                            continue;
                        };
                        self.record_feedback_slot(frame.code(), feedback_slot);
                        self.write_register(frame.registers(), a, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::BitNot => {
                        let bit_not_result =
                            self.bitwise_not_value(agent, host, registry, frame, b);
                        let Some(value) =
                            self.handle_dispatch_result(agent, frame_depth, frame, bit_not_result)?
                        else {
                            continue;
                        };
                        self.record_feedback_slot(frame.code(), feedback_slot);
                        self.write_register(frame.registers(), a, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::Increment | Opcode::Decrement => {
                        let update_result = self.update_register_value(
                            agent,
                            host,
                            registry,
                            frame,
                            b,
                            opcode == Opcode::Increment,
                        );
                        let Some((numeric, value)) =
                            self.handle_dispatch_result(agent, frame_depth, frame, update_result)?
                        else {
                            continue;
                        };
                        self.write_register(frame.registers(), b, numeric);
                        self.record_feedback_slot(frame.code(), feedback_slot);
                        self.write_register(frame.registers(), a, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::GetNamedProperty => {
                        self.execute_get_named_property_opcode(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            b,
                            c,
                        )?;
                    }
                    Opcode::SetNamedProperty
                    | Opcode::AssignNamedProperty
                    | Opcode::StrictAssignNamedProperty => {
                        self.execute_set_named_property_opcode(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            opcode,
                            a,
                            b,
                            c,
                        )?;
                    }
                    Opcode::DefineNamedProperty => {
                        self.execute_define_named_property_opcode(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            instruction_len,
                            a,
                            b,
                            c,
                        )?;
                    }
                    Opcode::GetKeyedProperty => {
                        self.execute_get_keyed_property_opcode(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            b,
                            c,
                        )?;
                    }
                    Opcode::SetKeyedProperty
                    | Opcode::AssignKeyedProperty
                    | Opcode::StrictAssignKeyedProperty => {
                        self.execute_set_keyed_property_opcode(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            opcode,
                            a,
                            b,
                            c,
                        )?;
                    }
                    Opcode::DefineKeyedProperty => {
                        self.execute_define_keyed_property_opcode(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            instruction_len,
                            a,
                            b,
                            c,
                        )?;
                    }
                    Opcode::SetFunctionName => {
                        let function = self.object_register(frame, a)?;
                        let name_value = self.read_register(frame.registers(), b);
                        let set_result = Self::set_function_name(agent, function, name_value);
                        let Some(()) =
                            self.handle_dispatch_result(agent, frame_depth, frame, set_result)?
                        else {
                            continue;
                        };
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::ToPropertyKey => {
                        self.execute_to_property_key_opcode(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            instruction_len,
                            a,
                            b,
                        )?;
                    }
                    Opcode::DeleteProperty => {
                        self.execute_delete_property_opcode(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            instruction_len,
                            a,
                            b,
                            c,
                        )?;
                    }
                    Opcode::CopyDataProperties => {
                        self.execute_copy_data_properties_opcode(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            instruction_len,
                            a,
                            b,
                            c,
                        )?;
                    }
                    Opcode::StoreDenseElement => {
                        self.execute_store_dense_element_opcode(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            instruction_len,
                            a,
                            b,
                            c,
                        )?;
                    }
                    Opcode::LoadDenseElement => {
                        self.execute_load_dense_element_opcode(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            instruction_len,
                            a,
                            b,
                            c,
                        )?;
                    }
                    Opcode::Call0 | Opcode::Call1 | Opcode::Call2 | Opcode::Call3 => {
                        let call_result = self.call_value_small(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            b,
                            c,
                            opcode.small_call_arity().unwrap_or(0),
                        );
                        let Some(()) =
                            self.handle_dispatch_result(agent, frame_depth, frame, call_result)?
                        else {
                            continue;
                        };
                    }
                    Opcode::Call => {
                        let range = call_range.ok_or_else(|| VmError::MissingInlineCallRange {
                            code: frame.code(),
                            instruction_offset: frame.instruction_offset(),
                            opcode,
                        })?;
                        let spread_mask = feedback_slot
                            .and_then(|slot| installed.feedback_descriptor_for_slot(slot))
                            .and_then(|descriptor| descriptor.metadata().spread_mask());
                        let call_result = self.call_value(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            b,
                            c,
                            range,
                            spread_mask,
                        );
                        let Some(()) =
                            self.handle_dispatch_result(agent, frame_depth, frame, call_result)?
                        else {
                            continue;
                        };
                    }
                    Opcode::TailCall => {
                        let range = call_range.ok_or_else(|| VmError::MissingInlineCallRange {
                            code: frame.code(),
                            instruction_offset: frame.instruction_offset(),
                            opcode,
                        })?;
                        let spread_mask = feedback_slot
                            .and_then(|slot| installed.feedback_descriptor_for_slot(slot))
                            .and_then(|descriptor| descriptor.metadata().spread_mask());
                        let tail_result = self.tail_call_value(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            feedback_slot,
                            a,
                            b,
                            range,
                            spread_mask,
                        );
                        let Some(result) =
                            self.handle_dispatch_result(agent, frame_depth, frame, tail_result)?
                        else {
                            continue;
                        };
                        self.record_feedback_slot(frame.code(), feedback_slot);
                        if let Some(result) = result {
                            return Ok(result);
                        }
                        self.refresh_dispatch_frame(frame_depth, frame);
                    }
                    Opcode::Construct => {
                        let range = call_range.ok_or_else(|| VmError::MissingInlineCallRange {
                            code: frame.code(),
                            instruction_offset: frame.instruction_offset(),
                            opcode,
                        })?;
                        let spread_mask = feedback_slot
                            .and_then(|slot| installed.feedback_descriptor_for_slot(slot))
                            .and_then(|descriptor| descriptor.metadata().spread_mask());
                        let construct_result = self.construct_value(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            a,
                            b,
                            range,
                            spread_mask,
                        );
                        let Some(()) = self.handle_dispatch_result(
                            agent,
                            frame_depth,
                            frame,
                            construct_result,
                        )?
                        else {
                            continue;
                        };
                    }
                    Opcode::CreateForIn => {
                        let value = self.read_register(frame.registers(), b);
                        let enumerator_result = self.create_for_in_enumerator_for_value(
                            agent, host, registry, frame, value,
                        );
                        let Some(enumerator) = self.handle_dispatch_result(
                            agent,
                            frame_depth,
                            frame,
                            enumerator_result,
                        )?
                        else {
                            continue;
                        };
                        self.for_in_states
                            .insert(frame.registers().base(), a, enumerator);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::AdvanceForIn => {
                        let next = self
                            .for_in_states
                            .advance(agent, frame.registers().base(), a);
                        let Some(next): Option<Option<PropertyKey>> =
                            self.handle_dispatch_result(agent, frame_depth, frame, next)?
                        else {
                            continue;
                        };
                        let done = next.is_none();
                        if let Some(key) = next {
                            let value = self.property_key_to_enumeration_value(agent, key);
                            self.write_register(frame.registers(), b, value);
                        } else {
                            self.write_register(frame.registers(), b, Value::undefined());
                        }
                        self.write_register(frame.registers(), c, Value::from_bool(done));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::CreateIterator => {
                        let value = self.read_register(frame.registers(), b);
                        let iterator_result = self.create_iterator_for_value(
                            agent,
                            host,
                            registry,
                            frame,
                            value,
                            c != 0,
                        );
                        let Some(iterator) = self.handle_dispatch_result(
                            agent,
                            frame_depth,
                            frame,
                            iterator_result,
                        )?
                        else {
                            continue;
                        };
                        self.iterator_states
                            .insert(frame.registers().base(), a, iterator);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::AdvanceIterator => {
                        self.sync_dispatch_frame(frame_depth, *frame);
                        let next = self.advance_iterator_state(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            a,
                        );
                        let Some(next) =
                            self.handle_dispatch_result(agent, frame_depth, frame, next)?
                        else {
                            continue;
                        };
                        let done = next.is_none();
                        self.write_register(
                            frame.registers(),
                            b,
                            next.unwrap_or(Value::undefined()),
                        );
                        self.write_register(frame.registers(), c, Value::from_bool(done));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::DelegateYield => {
                        let delegate_result = self.delegate_yield(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            instruction_len,
                            a,
                            b,
                            c,
                        );
                        let Some(()) = self.handle_dispatch_result(
                            agent,
                            frame_depth,
                            frame,
                            delegate_result,
                        )?
                        else {
                            continue;
                        };
                    }
                    Opcode::LdaUndefined => {
                        self.write_register(frame.registers(), 0, Value::undefined());
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LdaNull => {
                        self.write_register(frame.registers(), 0, Value::null());
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LdaTrue => {
                        self.write_register(frame.registers(), 0, Value::from_bool(true));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LdaFalse => {
                        self.write_register(frame.registers(), 0, Value::from_bool(false));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LdaZero => {
                        self.write_register(frame.registers(), 0, Value::from_smi(0));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LdaOne => {
                        self.write_register(frame.registers(), 0, Value::from_smi(1));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LdaSmi8 => {
                        let value = i8::from_le_bytes([bx.to_le_bytes()[0]]);
                        self.write_register(
                            frame.registers(),
                            0,
                            Value::from_smi(i32::from(value)),
                        );
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LdaConst8 => {
                        let value = self.read_constant(agent, frame.code(), bx)?;
                        let target = absolute_register(frame.registers(), 0);
                        self.register_stack[target] = value;
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::Ldar => {
                        let value = self.read_register(frame.registers(), a);
                        self.write_register(frame.registers(), 0, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::Star0
                    | Opcode::Star1
                    | Opcode::Star2
                    | Opcode::Star3
                    | Opcode::Star4
                    | Opcode::Star5
                    | Opcode::Star6
                    | Opcode::Star7 => {
                        let target = opcode
                            .accumulator_store_index()
                            .expect("store-accumulator opcode should have an index");
                        let value = self.read_register(frame.registers(), 0);
                        self.write_register(frame.registers(), target, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    // === Abx-form arms ===
                    Opcode::LoadUndefined => {
                        self.write_register(frame.registers(), a, Value::undefined());
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadUninitializedLexical => {
                        self.write_register(frame.registers(), a, Value::uninitialized_lexical());
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadNull => {
                        self.write_register(frame.registers(), a, Value::null());
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadTrue => {
                        self.write_register(frame.registers(), a, Value::from_bool(true));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadFalse => {
                        self.write_register(frame.registers(), a, Value::from_bool(false));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadZero => {
                        self.write_register(frame.registers(), a, Value::from_smi(0));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadOne => {
                        self.write_register(frame.registers(), a, Value::from_smi(1));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadSmi => {
                        let bytes = bx.to_le_bytes();
                        let value = i16::from_le_bytes([bytes[0], bytes[1]]);
                        self.write_register(
                            frame.registers(),
                            a,
                            Value::from_smi(i32::from(value)),
                        );
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadSmi8 => {
                        let value = i8::from_le_bytes([bx.to_le_bytes()[0]]);
                        self.write_register(
                            frame.registers(),
                            a,
                            Value::from_smi(i32::from(value)),
                        );
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadConst | Opcode::LoadConst8 => {
                        let value = self.read_constant(agent, frame.code(), bx)?;
                        let target = absolute_register(frame.registers(), a);
                        self.register_stack[target] = value;
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadLocal0
                    | Opcode::LoadLocal1
                    | Opcode::LoadLocal2
                    | Opcode::LoadLocal3 => {
                        let local = opcode
                            .local_load_index()
                            .expect("load-local opcode should have an index");
                        let value = self.read_register(frame.registers(), local);
                        self.write_register(frame.registers(), a, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::StoreLocal0
                    | Opcode::StoreLocal1
                    | Opcode::StoreLocal2
                    | Opcode::StoreLocal3 => {
                        let local = opcode
                            .local_store_index()
                            .expect("store-local opcode should have an index");
                        let value = self.read_register(frame.registers(), a);
                        self.write_register(frame.registers(), local, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadEnvSlot => {
                        let (depth, slot) = decode_env_operand(bx);
                        let environment = self.environment_for_slot_access(
                            agent,
                            frame.lexical_env(),
                            depth,
                            slot,
                        )?;
                        let slot_value = Self::read_environment_slot(agent, environment, slot);
                        let Some(value) =
                            self.handle_dispatch_result(agent, frame_depth, frame, slot_value)?
                        else {
                            continue;
                        };
                        self.write_register(frame.registers(), a, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::StoreEnvSlot => {
                        let (depth, slot) = decode_env_operand(bx);
                        let environment = self.environment_for_slot_access(
                            agent,
                            frame.lexical_env(),
                            depth,
                            slot,
                        )?;
                        let value = self.read_register(frame.registers(), a);
                        let store_result =
                            self.write_environment_slot(agent, environment, slot, value);
                        let Some(()) =
                            self.handle_dispatch_result(agent, frame_depth, frame, store_result)?
                        else {
                            continue;
                        };
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::AssignEnvSlot => {
                        let (depth, slot) = decode_env_operand(bx);
                        let environment = self.environment_for_slot_access(
                            agent,
                            frame.lexical_env(),
                            depth,
                            slot,
                        )?;
                        let value = self.read_register(frame.registers(), a);
                        let assign_result = self.assign_environment_slot(
                            agent,
                            environment,
                            slot,
                            value,
                            self.frame_is_strict(frame),
                        );
                        let Some(()) =
                            self.handle_dispatch_result(agent, frame_depth, frame, assign_result)?
                        else {
                            continue;
                        };
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::EnterEnvScope => {
                        self.enter_env_scope(agent, frame, a, bx)?;
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LeaveEnvScope => {
                        self.leave_env_scope(frame, a, bx);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadGlobal => {
                        let atom = self.read_atom_constant(frame.code(), bx)?;
                        let load_result = self.load_global_with_feedback(
                            agent,
                            host,
                            registry,
                            frame,
                            atom,
                            frame.code(),
                            feedback_slot,
                        );
                        let Some(value) =
                            self.handle_dispatch_result(agent, frame_depth, frame, load_result)?
                        else {
                            continue;
                        };
                        self.write_register(frame.registers(), a, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadName => {
                        let atom = self.read_atom_constant(frame.code(), bx)?;
                        let load_result =
                            self.load_name_with_context(agent, host, registry, frame, atom);
                        let Some(value) =
                            self.handle_dispatch_result(agent, frame_depth, frame, load_result)?
                        else {
                            continue;
                        };
                        self.write_register(frame.registers(), a, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::ResolveName => {
                        let atom = self.read_atom_constant(frame.code(), bx)?;
                        let resolve_result =
                            self.resolve_name_with_context(agent, host, registry, frame, atom);
                        let Some(value) =
                            self.handle_dispatch_result(agent, frame_depth, frame, resolve_result)?
                        else {
                            continue;
                        };
                        self.write_register(frame.registers(), a, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::ResolveGlobal => {
                        let atom = self.read_atom_constant(frame.code(), bx)?;
                        let resolve_result =
                            self.resolve_global(agent, host, registry, frame, atom);
                        let Some(value) =
                            self.handle_dispatch_result(agent, frame_depth, frame, resolve_result)?
                        else {
                            continue;
                        };
                        self.write_register(frame.registers(), a, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::AssignName => {
                        let atom = self.read_atom_constant(frame.code(), bx)?;
                        let value = self.read_register(frame.registers(), a);
                        let assign_result = self
                            .assign_name_with_context(agent, host, registry, frame, atom, value);
                        let Some(()) =
                            self.handle_dispatch_result(agent, frame_depth, frame, assign_result)?
                        else {
                            continue;
                        };
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::AssignVariableName => {
                        let atom = self.read_atom_constant(frame.code(), bx)?;
                        let value = self.read_register(frame.registers(), a);
                        let assign_result = self.assign_variable_name_with_context(
                            agent, host, registry, frame, atom, value,
                        );
                        let Some(()) =
                            self.handle_dispatch_result(agent, frame_depth, frame, assign_result)?
                        else {
                            continue;
                        };
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::DeleteName => {
                        let atom = self.read_atom_constant(frame.code(), bx)?;
                        let delete_result =
                            self.delete_name_with_context(agent, host, registry, frame, atom);
                        let Some(deleted) =
                            self.handle_dispatch_result(agent, frame_depth, frame, delete_result)?
                        else {
                            continue;
                        };
                        self.write_register(frame.registers(), a, Value::from_bool(deleted));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::CaptureName => {
                        let atom = self.read_atom_constant(frame.code(), bx)?;
                        let capture_result =
                            self.capture_name_with_context(agent, host, registry, frame, a, atom);
                        let Some(()) =
                            self.handle_dispatch_result(agent, frame_depth, frame, capture_result)?
                        else {
                            continue;
                        };
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadCapturedName => {
                        let reference_register =
                            u16::try_from(bx).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: u16::MAX,
                            })?;
                        let load_result = self.load_captured_name_with_context(
                            agent,
                            host,
                            registry,
                            frame,
                            reference_register,
                        );
                        let Some(value) =
                            self.handle_dispatch_result(agent, frame_depth, frame, load_result)?
                        else {
                            continue;
                        };
                        self.write_register(frame.registers(), a, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadCapturedNameThis => {
                        let reference_register =
                            u16::try_from(bx).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: u16::MAX,
                            })?;
                        let load_result =
                            self.load_captured_name_this_with_context(frame, reference_register);
                        let Some(value) =
                            self.handle_dispatch_result(agent, frame_depth, frame, load_result)?
                        else {
                            continue;
                        };
                        self.write_register(frame.registers(), a, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::AssignCapturedName => {
                        let reference_register =
                            u16::try_from(bx).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: u16::MAX,
                            })?;
                        let value = self.read_register(frame.registers(), a);
                        let assign_result = self.assign_captured_name_with_context(
                            agent,
                            host,
                            registry,
                            frame,
                            reference_register,
                            value,
                        );
                        let Some(()) =
                            self.handle_dispatch_result(agent, frame_depth, frame, assign_result)?
                        else {
                            continue;
                        };
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::StoreGlobal => {
                        let atom = self.read_atom_constant(frame.code(), bx)?;
                        let value = self.read_register(frame.registers(), a);
                        let store_result = self.store_global_with_feedback(
                            agent,
                            host,
                            registry,
                            frame,
                            atom,
                            value,
                            frame.code(),
                            feedback_slot,
                        );
                        let Some(()) =
                            self.handle_dispatch_result(agent, frame_depth, frame, store_result)?
                        else {
                            continue;
                        };
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::AssignGlobal => {
                        let atom = self.read_atom_constant(frame.code(), bx)?;
                        let value = self.read_register(frame.registers(), a);
                        let assign_result = self.assign_global_with_feedback(
                            agent,
                            host,
                            registry,
                            frame,
                            atom,
                            value,
                            frame.code(),
                            feedback_slot,
                        );
                        let Some(()) =
                            self.handle_dispatch_result(agent, frame_depth, frame, assign_result)?
                        else {
                            continue;
                        };
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::DeleteGlobal => {
                        let atom = self.read_atom_constant(frame.code(), bx)?;
                        let delete_result = Self::delete_global(agent, frame, atom);
                        let Some(deleted) =
                            self.handle_dispatch_result(agent, frame_depth, frame, delete_result)?
                        else {
                            continue;
                        };
                        self.write_register(frame.registers(), a, Value::from_bool(deleted));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadThis => {
                        let load_this = match agent.current_execution_context().map_or_else(
                            || ThisState::Value(frame.this_value()),
                            lyng_js_env::ExecutionContext::this_state,
                        ) {
                            ThisState::Value(value) => Ok(value),
                            ThisState::Uninitialized => {
                                Err(VmError::Abrupt(errors::throw_reference_error(agent)))
                            }
                            ThisState::Lexical => {
                                Self::resolve_this_binding(agent, frame.lexical_env(), frame)
                            }
                        };
                        let Some(value) =
                            self.handle_dispatch_result(agent, frame_depth, frame, load_this)?
                        else {
                            continue;
                        };
                        self.write_register(frame.registers(), a, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadCallee => {
                        let value = frame
                            .callee()
                            .map_or(Value::undefined(), Value::from_object_ref);
                        self.write_register(frame.registers(), a, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadNewTarget => {
                        let value = frame
                            .new_target()
                            .map_or(Value::undefined(), Value::from_object_ref);
                        self.write_register(frame.registers(), a, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::JumpIfTrue
                    | Opcode::JumpIfFalse
                    | Opcode::JumpIfTrue8
                    | Opcode::JumpIfFalse8 => {
                        let condition = self.read_register(frame.registers(), a);
                        let Some(truthy) = self.handle_dispatch_result(
                            agent,
                            frame_depth,
                            frame,
                            read::to_boolean_agent(agent, condition).map_err(VmError::Abrupt),
                        )?
                        else {
                            continue;
                        };
                        let delta = if matches!(opcode, Opcode::JumpIfTrue8 | Opcode::JumpIfFalse8)
                        {
                            i32::from(i8::from_le_bytes([bx.to_le_bytes()[0]]))
                        } else {
                            i32::from_le_bytes(bx.to_le_bytes())
                        };
                        let should_jump = match opcode {
                            Opcode::JumpIfTrue | Opcode::JumpIfTrue8 => truthy,
                            Opcode::JumpIfFalse | Opcode::JumpIfFalse8 => !truthy,
                            _ => unreachable!("guarded by opcode match"),
                        };
                        if should_jump {
                            if delta < 0 {
                                Self::poll_incremental_mark_safepoint(agent);
                            }
                            jump_dispatch_frame(frame, instruction_len, delta)?;
                        } else {
                            advance_dispatch_frame(frame, instruction_len);
                        }
                    }
                    Opcode::CreateObject => {
                        let object = Self::create_object(
                            agent,
                            frame.realm(),
                            usize::try_from(bx).unwrap_or(usize::MAX),
                        )?;
                        self.write_register(frame.registers(), a, Value::from_object_ref(object));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::CreateArray => {
                        let length = usize::try_from(bx).unwrap_or(usize::MAX);
                        let object = Self::create_array(agent, frame.realm(), length)?;
                        let length = u32::try_from(length).unwrap_or(u32::MAX);
                        if length != 0 {
                            Self::define_length_property(agent, object, length, false)?;
                        }
                        self.write_register(frame.registers(), a, Value::from_object_ref(object));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::CheckObjectCoercible => {
                        let value = self.read_register(frame.registers(), a);
                        let coercible = Self::check_object_coercible(agent, value);
                        let Some(()) =
                            self.handle_dispatch_result(agent, frame_depth, frame, coercible)?
                        else {
                            continue;
                        };
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::ThrowIfUninitialized => {
                        let value = self.read_register(frame.registers(), a);
                        if value == Value::uninitialized_lexical() {
                            let result = Err(VmError::Abrupt(errors::throw_reference_error(agent)));
                            let Some(()) =
                                self.handle_dispatch_result(agent, frame_depth, frame, result)?
                            else {
                                continue;
                            };
                        }
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::CreateClosure => {
                        let closure_result = self.create_closure(agent, frame, bx);
                        let Some(closure) =
                            self.handle_dispatch_result(agent, frame_depth, frame, closure_result)?
                        else {
                            continue;
                        };
                        self.write_register(frame.registers(), a, Value::from_object_ref(closure));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::CloseForIn => {
                        let _ = self.for_in_states.remove(frame.registers().base(), a);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::CloseIterator => {
                        let close_result = self.close_iterator_state(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            a,
                            bx != 0,
                        );
                        let Some(()) =
                            self.handle_dispatch_result(agent, frame_depth, frame, close_result)?
                        else {
                            continue;
                        };
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    // === Ax-form arms ===
                    Opcode::Nop => advance_dispatch_frame(frame, instruction_len),
                    Opcode::LoopHeader => {
                        if DEBUG {
                            self.sync_dispatch_frame(frame_depth, *frame);
                            self.poll_debug_safepoint(agent, VmDebugSafepointKind::LoopHeader);
                            self.refresh_dispatch_frame(frame_depth, frame);
                        }
                        self.observe_tier_backedge_event(frame.code());
                        Self::poll_incremental_mark_safepoint(agent);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::Jump | Opcode::Jump8 => {
                        if ax < 0 {
                            self.observe_tier_backedge_event(frame.code());
                            Self::poll_incremental_mark_safepoint(agent);
                        }
                        jump_dispatch_frame(frame, instruction_len, ax)?;
                    }
                    Opcode::PushClosureEnv => {
                        let site = installed
                            .loop_iteration_environment_site(frame.instruction_offset())
                            .cloned();
                        let mirrored_slot = if ax > 0 {
                            Some(
                                u32::try_from(ax - 1).map_err(|_| VmError::UnsupportedOpcode {
                                    code: frame.code(),
                                    instruction_offset: frame.instruction_offset(),
                                    opcode,
                                })?,
                            )
                        } else {
                            None
                        };
                        self.push_loop_iteration_environment(agent, frame, site, mirrored_slot)?;
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::PopClosureEnv => {
                        self.pop_loop_iteration_environment();
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::PushWithEnv => {
                        let register =
                            u16::try_from(ax).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: 0,
                            })?;
                        let value = self.read_register(frame.registers(), register);
                        let push_result = self.push_with_environment(agent, frame, value);
                        let Some(()) =
                            self.handle_dispatch_result(agent, frame_depth, frame, push_result)?
                        else {
                            continue;
                        };
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::PopWithEnv => {
                        self.pop_with_environment(frame);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::TypeOf => {
                        let register =
                            u16::try_from(ax).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: 0,
                            })?;
                        let value = self.read_register(frame.registers(), register);
                        let type_string = Self::type_of_value(agent, value);
                        self.write_register(frame.registers(), register, type_string);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::Return => {
                        let register =
                            u16::try_from(ax).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: 0,
                            })?;
                        let value = self.read_register(frame.registers(), register);
                        self.sync_dispatch_frame(frame_depth, *frame);
                        let _ = agent.pop_execution_context();
                        if let Some(result) = self.finish_frame(agent, value)? {
                            return Ok(result);
                        }
                    }
                    Opcode::ReturnUndefined => {
                        self.sync_dispatch_frame(frame_depth, *frame);
                        let _ = agent.pop_execution_context();
                        if let Some(result) = self.finish_frame(agent, Value::undefined())? {
                            return Ok(result);
                        }
                    }
                    Opcode::SuspendGeneratorStart => {
                        let resume_offset =
                            next_dispatch_instruction_offset(frame, instruction_len);
                        self.sync_dispatch_frame(frame_depth, *frame);
                        self.suspend_generator_start(agent, frame, resume_offset)?;
                    }
                    Opcode::Yield => {
                        let register =
                            u16::try_from(ax).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: 0,
                            })?;
                        let value = self.read_register(frame.registers(), register);
                        self.sync_dispatch_frame(frame_depth, *frame);
                        self.suspend_current_generator_frame(
                            agent,
                            frame,
                            value,
                            next_dispatch_instruction_offset(frame, instruction_len),
                            false,
                        )?;
                    }
                    Opcode::Await => {
                        let register =
                            u16::try_from(ax).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: 0,
                            })?;
                        self.await_value(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            instruction_len,
                            register,
                        )?;
                    }
                    Opcode::Throw => {
                        let register =
                            u16::try_from(ax).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: 0,
                            })?;
                        let value = self.read_register(frame.registers(), register);
                        self.sync_dispatch_frame(frame_depth, *frame);
                        if self.transfer_to_exception_handler(agent, value)? {
                            self.refresh_dispatch_frame(frame_depth, frame);
                            continue;
                        }
                        return Err(VmError::Abrupt(AbruptCompletion::Throw(value)));
                    }
                    Opcode::EnterHandler | Opcode::LeaveHandler => {
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadException => {
                        let register =
                            u16::try_from(ax).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: 0,
                            })?;
                        let value = self.current_exception.unwrap_or(Value::undefined());
                        self.write_register(frame.registers(), register, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadResumeKind => {
                        let register =
                            u16::try_from(ax).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: 0,
                            })?;
                        self.write_register(
                            frame.registers(),
                            register,
                            Value::from_smi(i32::from(frame.resume_kind().raw())),
                        );
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadResumeValue => {
                        let register =
                            u16::try_from(ax).map_err(|_| VmError::RegisterOutOfBounds {
                                code: frame.code(),
                                register: 0,
                            })?;
                        self.write_register(frame.registers(), register, frame.resume_value());
                        frame.clear_resume();
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    _ => {
                        return Err(VmError::UnsupportedOpcode {
                            code: frame.code(),
                            instruction_offset: frame.instruction_offset(),
                            opcode,
                        });
                    }
                }
            }
        }
    }
}
