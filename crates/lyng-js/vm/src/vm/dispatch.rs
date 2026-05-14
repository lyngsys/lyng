use super::dispatch::arithmetic::{decode_smi_immediate, smi_mod_result, smi_mul_result};
use super::registers::absolute_register;
use super::values::decode_env_operand;
use super::{
    code_index, Agent, CallRange, CodeRef, FrameRecord, HostHooks, NativeFunctionRegistry, Opcode,
    ThisState, Value, Vm, VmDebugSafepointKind, VmError, VmResult,
};
use lyng_js_ops::{errors, read};
use lyng_js_types::{AbruptCompletion, FeedbackSlotId, PropertyKey};

pub(in crate::vm) mod arithmetic;
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
            run_loop.contains("DispatchFrameSnapshot"),
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
    fn dispatch_loop_folds_operand_decode_into_opcode_arms() {
        let source = include_str!("dispatch.rs");
        let run_loop = source
            .split("\n    fn run_dispatch_loop")
            .nth(1)
            .expect("dispatch loop should stay in this module");

        assert!(
            run_loop.contains("match semantic_opcode"),
            "hot dispatch should enter opcode arms with the encoded semantic opcode"
        );
        assert!(
            !run_loop.contains("dispatch_operand_form("),
            "operand layout should be selected inside opcode arms, not through a centralized form dispatch"
        );
    }

    #[test]
    fn dispatch_loop_avoids_unconditional_frame_active_check() {
        let source = include_str!("dispatch.rs");
        let run_loop = source
            .split("\n    fn run_dispatch_loop")
            .nth(1)
            .expect("dispatch loop should stay in this module");

        assert!(
            !run_loop.contains("if !state.active_in(self)"),
            "frame-stack validation should run only after frame-changing boundaries"
        );
    }

    #[test]
    fn dispatch_loop_does_not_call_profiled_metadata_helpers() {
        // Track H removed the *Profiled opcode mirror. The dispatch loop must no
        // longer derive `is_profiled` / base opcode via the 46-arm match jump
        // tables that those helpers compiled into — that jump table was the
        // second indirect branch we're eliminating.
        let source = include_str!("dispatch.rs");
        let run_loop = source
            .split("\n    fn run_dispatch_loop")
            .nth(1)
            .expect("dispatch loop should stay in this module");

        assert!(
            !run_loop.contains("profiled_base_opcode("),
            "hot dispatch should not call profiled_base_opcode (Track H removed *Profiled mirror)"
        );
        assert!(
            !run_loop.contains(".is_profiled()"),
            "hot dispatch should not call Opcode::is_profiled (Track H removed *Profiled mirror)"
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
            .split("Opcode::LoadConst => {")
            .nth(1)
            .and_then(|tail| tail.split("Opcode::LoadConst8").next())
            .expect("LoadConst arm should stay directly before local load arms");

        assert!(
            !load_const_arm.contains("self.write_register("),
            "LoadConst should write through direct checked register-window access"
        );
    }
}

#[derive(Clone, Copy)]
pub(in crate::vm) struct DispatchFrameSnapshot {
    frame_depth: usize,
    code: CodeRef,
    frame: FrameRecord,
    frame_check_epoch: u32,
}

impl DispatchFrameSnapshot {
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
            frame_check_epoch: vm.dispatch_frame_check_epoch,
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

pub(in crate::vm) const fn sign_extend_i24(bytes: [u8; 3]) -> i32 {
    let sign = if bytes[2] & 0x80 == 0 { 0 } else { 0xff };
    i32::from_le_bytes([bytes[0], bytes[1], bytes[2], sign])
}

type DecodedCallRangeOperands = (
    u16,
    u16,
    u16,
    Option<CallRange>,
    Option<FeedbackSlotId>,
    u32,
);

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

#[inline]
pub(in crate::vm) fn decode_feedback_slot_operand(
    bytes: &[u8],
    operand_end: usize,
    is_profiled: bool,
    code: CodeRef,
    instruction_offset: u32,
) -> VmResult<(Option<FeedbackSlotId>, u32)> {
    if is_profiled {
        let [slot_low, slot_high, ..] =
            bytes
                .get(operand_end..)
                .ok_or(VmError::InstructionOutOfBounds {
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
        let len = u32::try_from(operand_end + 2).map_err(|_| VmError::InstructionOutOfBounds {
            code,
            instruction_offset,
        })?;
        Ok((Some(slot), len))
    } else {
        let len = u32::try_from(operand_end).map_err(|_| VmError::InstructionOutOfBounds {
            code,
            instruction_offset,
        })?;
        Ok((None, len))
    }
}

#[inline]
pub(in crate::vm) fn decode_abc_operands(
    bytes: &[u8],
    prefix: Option<Opcode>,
    is_profiled: bool,
    code: CodeRef,
    instruction_offset: u32,
) -> VmResult<(u16, u16, u16, Option<FeedbackSlotId>, u32)> {
    if prefix.is_some() {
        return decode_abc_operands_wide(bytes, is_profiled, code, instruction_offset);
    }
    let [_, ra, rb, rc, ..] = bytes else {
        return Err(VmError::InstructionOutOfBounds {
            code,
            instruction_offset,
        });
    };
    let (feedback_slot, instruction_len) =
        decode_feedback_slot_operand(bytes, 4usize, is_profiled, code, instruction_offset)?;
    Ok((
        u16::from(*ra),
        u16::from(*rb),
        u16::from(*rc),
        feedback_slot,
        instruction_len,
    ))
}

/// Wide / ExtraWide-prefixed Abc operand decoding. Extracted to a `#[cold]`
/// `#[inline(never)]` helper so the narrow path inlines into each handler
/// without dragging the wide decoder bytes along — the wide path is
/// "essentially zero share on real workloads" (Phase 1 spec), and per-handler
/// asm should fit the < 200 byte budget without inline wide code competing
/// for L1i.
#[cold]
#[inline(never)]
fn decode_abc_operands_wide(
    bytes: &[u8],
    is_profiled: bool,
    code: CodeRef,
    instruction_offset: u32,
) -> VmResult<(u16, u16, u16, Option<FeedbackSlotId>, u32)> {
    let [_, _, a_low, b_low, c_low, a_high, b_high, c_high, ..] = bytes else {
        return Err(VmError::InstructionOutOfBounds {
            code,
            instruction_offset,
        });
    };
    let (feedback_slot, instruction_len) =
        decode_feedback_slot_operand(bytes, 8usize, is_profiled, code, instruction_offset)?;
    Ok((
        u16::from_le_bytes([*a_low, *a_high]),
        u16::from_le_bytes([*b_low, *b_high]),
        u16::from_le_bytes([*c_low, *c_high]),
        feedback_slot,
        instruction_len,
    ))
}

#[inline]
pub(in crate::vm) fn decode_abx_operands(
    bytes: &[u8],
    prefix: Option<Opcode>,
    is_profiled: bool,
    code: CodeRef,
    instruction_offset: u32,
) -> VmResult<(u16, u32, Option<FeedbackSlotId>, u32)> {
    if let Some(prefix) = prefix {
        return decode_abx_operands_wide(bytes, prefix, is_profiled, code, instruction_offset);
    }
    let [_, ra, bx_low, bx_high, ..] = bytes else {
        return Err(VmError::InstructionOutOfBounds {
            code,
            instruction_offset,
        });
    };
    let (feedback_slot, instruction_len) =
        decode_feedback_slot_operand(bytes, 4usize, is_profiled, code, instruction_offset)?;
    Ok((
        u16::from(*ra),
        u32::from(u16::from_le_bytes([*bx_low, *bx_high])),
        feedback_slot,
        instruction_len,
    ))
}

/// Wide / ExtraWide-prefixed Abx operand decoding. See
/// `decode_abc_operands_wide` for the rationale on the `#[cold]` /
/// `#[inline(never)]` placement.
#[cold]
#[inline(never)]
fn decode_abx_operands_wide(
    bytes: &[u8],
    prefix: Opcode,
    is_profiled: bool,
    code: CodeRef,
    instruction_offset: u32,
) -> VmResult<(u16, u32, Option<FeedbackSlotId>, u32)> {
    let [_, _, a_low, bx0, bx1, a_high, bx2, bx3, ..] = bytes else {
        return Err(VmError::InstructionOutOfBounds {
            code,
            instruction_offset,
        });
    };
    let bx3 = if prefix == Opcode::ExtraWide { *bx3 } else { 0 };
    let (feedback_slot, instruction_len) =
        decode_feedback_slot_operand(bytes, 8usize, is_profiled, code, instruction_offset)?;
    Ok((
        u16::from_le_bytes([*a_low, *a_high]),
        u32::from_le_bytes([*bx0, *bx1, *bx2, bx3]),
        feedback_slot,
        instruction_len,
    ))
}

#[inline]
pub(in crate::vm) fn decode_abx8_operands(
    bytes: &[u8],
    is_profiled: bool,
    code: CodeRef,
    instruction_offset: u32,
) -> VmResult<(u16, u32, Option<FeedbackSlotId>, u32)> {
    let [_, ra, rbx, ..] = bytes else {
        return Err(VmError::InstructionOutOfBounds {
            code,
            instruction_offset,
        });
    };
    let (feedback_slot, instruction_len) =
        decode_feedback_slot_operand(bytes, 3usize, is_profiled, code, instruction_offset)?;
    Ok((
        u16::from(*ra),
        u32::from(*rbx),
        feedback_slot,
        instruction_len,
    ))
}

#[inline]
pub(in crate::vm) fn decode_ax_operands(
    bytes: &[u8],
    is_profiled: bool,
    code: CodeRef,
    instruction_offset: u32,
) -> VmResult<(i32, Option<FeedbackSlotId>, u32)> {
    let [_, first_byte, second, third, ..] = bytes else {
        return Err(VmError::InstructionOutOfBounds {
            code,
            instruction_offset,
        });
    };
    let (feedback_slot, instruction_len) =
        decode_feedback_slot_operand(bytes, 4usize, is_profiled, code, instruction_offset)?;
    Ok((
        sign_extend_i24([*first_byte, *second, *third]),
        feedback_slot,
        instruction_len,
    ))
}

#[inline]
pub(in crate::vm) fn decode_ax8_operands(
    bytes: &[u8],
    is_profiled: bool,
    code: CodeRef,
    instruction_offset: u32,
) -> VmResult<(i32, Option<FeedbackSlotId>, u32)> {
    let [_, raw_ax, ..] = bytes else {
        return Err(VmError::InstructionOutOfBounds {
            code,
            instruction_offset,
        });
    };
    let (feedback_slot, instruction_len) =
        decode_feedback_slot_operand(bytes, 2usize, is_profiled, code, instruction_offset)?;
    Ok((
        i32::from(i8::from_le_bytes([*raw_ax])),
        feedback_slot,
        instruction_len,
    ))
}

#[inline]
pub(in crate::vm) fn decode_local_operands(
    bytes: &[u8],
    is_profiled: bool,
    code: CodeRef,
    instruction_offset: u32,
) -> VmResult<(u16, Option<FeedbackSlotId>, u32)> {
    let [_, register, ..] = bytes else {
        return Err(VmError::InstructionOutOfBounds {
            code,
            instruction_offset,
        });
    };
    let (feedback_slot, instruction_len) =
        decode_feedback_slot_operand(bytes, 2usize, is_profiled, code, instruction_offset)?;
    Ok((u16::from(*register), feedback_slot, instruction_len))
}

#[inline]
pub(in crate::vm) fn decode_accumulator_operands(
    bytes: &[u8],
    is_profiled: bool,
    code: CodeRef,
    instruction_offset: u32,
) -> VmResult<(Option<FeedbackSlotId>, u32)> {
    decode_feedback_slot_operand(bytes, 1usize, is_profiled, code, instruction_offset)
}

#[inline]
pub(in crate::vm) fn decode_accumulator_byte_operands(
    bytes: &[u8],
    is_profiled: bool,
    code: CodeRef,
    instruction_offset: u32,
) -> VmResult<(u32, Option<FeedbackSlotId>, u32)> {
    let [_, operand, ..] = bytes else {
        return Err(VmError::InstructionOutOfBounds {
            code,
            instruction_offset,
        });
    };
    let (feedback_slot, instruction_len) =
        decode_feedback_slot_operand(bytes, 2usize, is_profiled, code, instruction_offset)?;
    Ok((u32::from(*operand), feedback_slot, instruction_len))
}

#[inline]
pub(in crate::vm) fn decode_accumulator_register_operands(
    bytes: &[u8],
    is_profiled: bool,
    code: CodeRef,
    instruction_offset: u32,
) -> VmResult<(u16, Option<FeedbackSlotId>, u32)> {
    let [_, register, ..] = bytes else {
        return Err(VmError::InstructionOutOfBounds {
            code,
            instruction_offset,
        });
    };
    let (feedback_slot, instruction_len) =
        decode_feedback_slot_operand(bytes, 2usize, is_profiled, code, instruction_offset)?;
    Ok((u16::from(*register), feedback_slot, instruction_len))
}

#[inline]
fn decode_call_range_operands(
    bytes: &[u8],
    is_profiled: bool,
    code: CodeRef,
    instruction_offset: u32,
) -> VmResult<DecodedCallRangeOperands> {
    let [_, ra, rb, rc, count_low, count_high, base_low, base_high, ..] = bytes else {
        return Err(VmError::InstructionOutOfBounds {
            code,
            instruction_offset,
        });
    };
    let (feedback_slot, instruction_len) =
        decode_feedback_slot_operand(bytes, 8usize, is_profiled, code, instruction_offset)?;
    Ok((
        u16::from(*ra),
        u16::from(*rb),
        u16::from(*rc),
        Some(CallRange::new(
            u16::from_le_bytes([*base_low, *base_high]),
            u16::from_le_bytes([*count_low, *count_high]),
        )),
        feedback_slot,
        instruction_len,
    ))
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

    #[inline]
    pub(in crate::vm) const fn request_dispatch_frame_check(&mut self) {
        self.dispatch_frame_check_epoch = self.dispatch_frame_check_epoch.wrapping_add(1);
    }

    #[inline]
    pub(in crate::vm) const fn dispatch_frame_check_epoch(&self) -> u32 {
        self.dispatch_frame_check_epoch
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
                    self.request_dispatch_frame_check();
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
    pub(in crate::vm) fn finish_abc_value_result(
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
        // Phase 1 (lyng-33i2): the `trampoline-dispatch` feature swaps in the
        // per-handler `extern "C" fn` dispatch path. Off by default until
        // sub-3..sub-7 land real handlers for every opcode family; the cutover
        // (sub-8, lyng-9gyk) flips the default and deletes the legacy match.
        #[cfg(feature = "trampoline-dispatch")]
        {
            return self.run_via_trampoline(agent, host, registry);
        }

        // Two const-generic switches at the dispatcher boundary: opcode-counter recording
        // and debugger safepoint polling. Both are zero-overhead when disabled because the
        // const propagates through `run_dispatch_loop` and LLVM strips the dead branches.
        #[cfg(not(feature = "trampoline-dispatch"))]
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
    #[allow(
        unused_variables,
        reason = "per-opcode operand helpers decode the full bytecode form; some opcodes intentionally ignore fields in that form"
    )]
    fn run_dispatch_loop<const COUNT_OPCODES: bool, const DEBUG: bool>(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> VmResult<Value> {
        loop {
            let mut state = DispatchFrameSnapshot::from_active_frame(self);
            let code = state.frame().code();
            let installed = self
                .installed
                .get(code_index(code))
                .and_then(Option::as_ref)
                .cloned()
                .ok_or(VmError::MissingInstalledCode(code))?;

            loop {
                if state.frame_check_epoch != self.dispatch_frame_check_epoch() {
                    state.frame_check_epoch = self.dispatch_frame_check_epoch();
                    let active = state.active_in(self);
                    if !active {
                        break;
                    }
                }
                #[cfg(debug_assertions)]
                debug_assert!(
                    state.active_in(self),
                    "dispatch frame state should stay active until a boundary requests validation"
                );
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
                let first =
                    Opcode::from_byte(raw_first).ok_or(VmError::InstructionOutOfBounds {
                        code,
                        instruction_offset,
                    })?;
                // Hot path: no prefix. Wide/ExtraWide are essentially zero share on real
                // workloads; we still handle them via the same inline path.
                let (prefix, semantic_opcode) = if first.is_prefix() {
                    let raw_semantic = *bytes.get(1).ok_or(VmError::InstructionOutOfBounds {
                        code,
                        instruction_offset,
                    })?;
                    let semantic =
                        Opcode::from_byte(raw_semantic).ok_or(VmError::InstructionOutOfBounds {
                            code,
                            instruction_offset,
                        })?;
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
                // After Track H, the *Profiled opcode mirror is gone; the slot is a
                // mandatory trailing operand of every IC-shaped opcode. `has_feedback_slot()`
                // returns a `bool`, which LLVM should fold to a bitset or to per-arm
                // constant-propagation without producing a separate indirect-branch
                // jump table the way the previous `profiled_base_opcode` did.
                let is_profiled = semantic_opcode.has_feedback_slot();

                if COUNT_OPCODES {
                    self.record_opcode_dispatch(semantic_opcode);
                }
                #[cfg(debug_assertions)]
                self.assert_deopt_safepoint_state(agent, state.frame(), installed.as_ref());

                let frame_depth = state.frame_depth();
                let frame = &mut state.frame;

                // Flat dispatch on the dense `#[repr(u8)]` Opcode enum. Each arm chooses
                // its operand layout directly so there is no pre-dispatch operand-form
                // branch in the hot loop.
                #[allow(
                    clippy::match_same_arms,
                    reason = "opcode families stay grouped even when marker opcodes share dispatch behavior"
                )]
                match semantic_opcode {
                    Opcode::Move => {
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        let registers = frame.registers();
                        let source = absolute_register(registers, b);
                        let target = absolute_register(registers, a);
                        let value = self.register_stack[source];
                        self.register_stack[target] = value;
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::Add => {
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let result = self.execute_add_opcode(agent, host, registry, frame, b, c);
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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let result = self.execute_sub_opcode(agent, host, registry, frame, b, c);
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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let result = self.execute_mul_opcode(agent, host, registry, frame, b, c);
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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let result = self.execute_mod_opcode(agent, host, registry, frame, b, c);
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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let result = self.execute_bitand_opcode(agent, host, registry, frame, b, c);
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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                    Opcode::Increment
                   
                    | Opcode::Decrement
                    => {
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        let update_result = self.update_register_value(
                            agent,
                            host,
                            registry,
                            frame,
                            b,
                            semantic_opcode == Opcode::Increment,
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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                   
                    | Opcode::StrictAssignNamedProperty
                    => {
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        self.execute_set_named_property_opcode(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            semantic_opcode,
                            a,
                            b,
                            c,
                        )?;
                    }
                    Opcode::DefineNamedProperty => {
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                   
                    | Opcode::StrictAssignKeyedProperty
                    => {
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        self.execute_set_keyed_property_opcode(
                            agent,
                            host,
                            registry,
                            frame_depth,
                            frame,
                            instruction_len,
                            feedback_slot,
                            semantic_opcode,
                            a,
                            b,
                            c,
                        )?;
                    }
                    Opcode::DefineKeyedProperty => {
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                            semantic_opcode.small_call_arity().unwrap_or(0),
                        );
                        let Some(()) =
                            self.handle_dispatch_result(agent, frame_depth, frame, call_result)?
                        else {
                            continue;
                        };
                    }
                    Opcode::Call => {
                        let (a, b, c, call_range, feedback_slot, instruction_len) =
                            decode_call_range_operands(
                                bytes,
                                is_profiled,
                                code,
                                instruction_offset,
                            )?;

                        let range = call_range.ok_or_else(|| VmError::MissingInlineCallRange {
                            code: frame.code(),
                            instruction_offset: frame.instruction_offset(),
                            opcode: semantic_opcode,
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
                        let (a, b, c, call_range, feedback_slot, instruction_len) =
                            decode_call_range_operands(
                                bytes,
                                is_profiled,
                                code,
                                instruction_offset,
                            )?;

                        let range = call_range.ok_or_else(|| VmError::MissingInlineCallRange {
                            code: frame.code(),
                            instruction_offset: frame.instruction_offset(),
                            opcode: semantic_opcode,
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
                        let (a, b, c, call_range, feedback_slot, instruction_len) =
                            decode_call_range_operands(
                                bytes,
                                is_profiled,
                                code,
                                instruction_offset,
                            )?;

                        let range = call_range.ok_or_else(|| VmError::MissingInlineCallRange {
                            code: frame.code(),
                            instruction_offset: frame.instruction_offset(),
                            opcode: semantic_opcode,
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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, b, c, feedback_slot, instruction_len) = decode_abc_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (feedback_slot, instruction_len) = decode_accumulator_operands(
                            bytes,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        self.write_register(frame.registers(), 0, Value::undefined());
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LdaNull => {
                        let (feedback_slot, instruction_len) = decode_accumulator_operands(
                            bytes,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        self.write_register(frame.registers(), 0, Value::null());
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LdaTrue => {
                        let (feedback_slot, instruction_len) = decode_accumulator_operands(
                            bytes,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        self.write_register(frame.registers(), 0, Value::from_bool(true));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LdaFalse => {
                        let (feedback_slot, instruction_len) = decode_accumulator_operands(
                            bytes,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        self.write_register(frame.registers(), 0, Value::from_bool(false));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LdaZero => {
                        let (feedback_slot, instruction_len) = decode_accumulator_operands(
                            bytes,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        self.write_register(frame.registers(), 0, Value::from_smi(0));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LdaOne => {
                        let (feedback_slot, instruction_len) = decode_accumulator_operands(
                            bytes,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        self.write_register(frame.registers(), 0, Value::from_smi(1));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LdaSmi8 => {
                        let (bx, feedback_slot, instruction_len) =
                            decode_accumulator_byte_operands(
                                bytes,
                                is_profiled,
                                code,
                                instruction_offset,
                            )?;

                        let value = i8::from_le_bytes([bx.to_le_bytes()[0]]);
                        self.write_register(
                            frame.registers(),
                            0,
                            Value::from_smi(i32::from(value)),
                        );
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LdaConst8 => {
                        let (bx, feedback_slot, instruction_len) =
                            decode_accumulator_byte_operands(
                                bytes,
                                is_profiled,
                                code,
                                instruction_offset,
                            )?;

                        let value = self.read_constant(agent, frame.code(), bx)?;
                        let target = absolute_register(frame.registers(), 0);
                        self.register_stack[target] = value;
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::Ldar => {
                        let (a, feedback_slot, instruction_len) =
                            decode_accumulator_register_operands(
                                bytes,
                                is_profiled,
                                code,
                                instruction_offset,
                            )?;

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
                        let (feedback_slot, instruction_len) = decode_accumulator_operands(
                            bytes,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        let target = semantic_opcode
                            .accumulator_store_index()
                            .expect("store-accumulator opcode should have an index");
                        let value = self.read_register(frame.registers(), 0);
                        self.write_register(frame.registers(), target, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    // === Abx-form arms ===
                    Opcode::LoadUndefined => {
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        self.write_register(frame.registers(), a, Value::undefined());
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadUninitializedLexical => {
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        self.write_register(frame.registers(), a, Value::uninitialized_lexical());
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadNull => {
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        self.write_register(frame.registers(), a, Value::null());
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadTrue => {
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        self.write_register(frame.registers(), a, Value::from_bool(true));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadFalse => {
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        self.write_register(frame.registers(), a, Value::from_bool(false));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadZero => {
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        self.write_register(frame.registers(), a, Value::from_smi(0));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadOne => {
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        self.write_register(frame.registers(), a, Value::from_smi(1));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadSmi => {
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) =
                            decode_abx8_operands(bytes, is_profiled, code, instruction_offset)?;

                        let value = i8::from_le_bytes([bx.to_le_bytes()[0]]);
                        self.write_register(
                            frame.registers(),
                            a,
                            Value::from_smi(i32::from(value)),
                        );
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadConst => {
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        let value = self.read_constant(agent, frame.code(), bx)?;
                        let target = absolute_register(frame.registers(), a);
                        self.register_stack[target] = value;
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadConst8 => {
                        let (a, bx, feedback_slot, instruction_len) =
                            decode_abx8_operands(bytes, is_profiled, code, instruction_offset)?;

                        let value = self.read_constant(agent, frame.code(), bx)?;
                        let target = absolute_register(frame.registers(), a);
                        self.register_stack[target] = value;
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadLocal0
                    | Opcode::LoadLocal1
                    | Opcode::LoadLocal2
                    | Opcode::LoadLocal3 => {
                        let (a, feedback_slot, instruction_len) =
                            decode_local_operands(bytes, is_profiled, code, instruction_offset)?;

                        let local = semantic_opcode
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
                        let (a, feedback_slot, instruction_len) =
                            decode_local_operands(bytes, is_profiled, code, instruction_offset)?;

                        let local = semantic_opcode
                            .local_store_index()
                            .expect("store-local opcode should have an index");
                        let value = self.read_register(frame.registers(), a);
                        self.write_register(frame.registers(), local, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadEnvSlot => {
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        self.enter_env_scope(agent, frame, a, bx)?;
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LeaveEnvScope => {
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        self.leave_env_scope(frame, a, bx);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadGlobal => {
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        let value = frame
                            .callee()
                            .map_or(Value::undefined(), Value::from_object_ref);
                        self.write_register(frame.registers(), a, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadNewTarget => {
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        let value = frame
                            .new_target()
                            .map_or(Value::undefined(), Value::from_object_ref);
                        self.write_register(frame.registers(), a, value);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::JumpIfTrue | Opcode::JumpIfFalse => {
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let delta = i32::from_le_bytes(bx.to_le_bytes());
                        let should_jump = match semantic_opcode {
                            Opcode::JumpIfTrue => truthy,
                            Opcode::JumpIfFalse => !truthy,
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
                    Opcode::JumpIfTrue8 | Opcode::JumpIfFalse8 => {
                        let (a, bx, feedback_slot, instruction_len) =
                            decode_abx8_operands(bytes, is_profiled, code, instruction_offset)?;

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
                        let delta = i32::from(i8::from_le_bytes([bx.to_le_bytes()[0]]));
                        let should_jump = match semantic_opcode {
                            Opcode::JumpIfTrue8 => truthy,
                            Opcode::JumpIfFalse8 => !truthy,
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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        let object = Self::create_object(
                            agent,
                            frame.realm(),
                            usize::try_from(bx).unwrap_or(usize::MAX),
                        )?;
                        self.write_register(frame.registers(), a, Value::from_object_ref(object));
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::CreateArray => {
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

                        let _ = self.for_in_states.remove(frame.registers().base(), a);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::CloseIterator => {
                        let (a, bx, feedback_slot, instruction_len) = decode_abx_operands(
                            bytes,
                            prefix,
                            is_profiled,
                            code,
                            instruction_offset,
                        )?;

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
                    Opcode::Nop => {
                        let (ax, feedback_slot, instruction_len) =
                            decode_ax_operands(bytes, is_profiled, code, instruction_offset)?;
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoopHeader => {
                        let (ax, feedback_slot, instruction_len) =
                            decode_ax_operands(bytes, is_profiled, code, instruction_offset)?;

                        if DEBUG {
                            self.sync_dispatch_frame(frame_depth, *frame);
                            self.poll_debug_safepoint(agent, VmDebugSafepointKind::LoopHeader);
                            self.refresh_dispatch_frame(frame_depth, frame);
                        }
                        self.observe_tier_backedge_event(frame.code());
                        Self::poll_incremental_mark_safepoint(agent);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::Jump => {
                        let (ax, feedback_slot, instruction_len) =
                            decode_ax_operands(bytes, is_profiled, code, instruction_offset)?;

                        if ax < 0 {
                            self.observe_tier_backedge_event(frame.code());
                            Self::poll_incremental_mark_safepoint(agent);
                        }
                        jump_dispatch_frame(frame, instruction_len, ax)?;
                    }
                    Opcode::Jump8 => {
                        let (ax, feedback_slot, instruction_len) =
                            decode_ax8_operands(bytes, is_profiled, code, instruction_offset)?;

                        if ax < 0 {
                            self.observe_tier_backedge_event(frame.code());
                            Self::poll_incremental_mark_safepoint(agent);
                        }
                        jump_dispatch_frame(frame, instruction_len, ax)?;
                    }
                    Opcode::PushClosureEnv => {
                        let (ax, feedback_slot, instruction_len) =
                            decode_ax_operands(bytes, is_profiled, code, instruction_offset)?;

                        let site = installed
                            .loop_iteration_environment_site(frame.instruction_offset())
                            .cloned();
                        let mirrored_slot = if ax > 0 {
                            Some(
                                u32::try_from(ax - 1).map_err(|_| VmError::UnsupportedOpcode {
                                    code: frame.code(),
                                    instruction_offset: frame.instruction_offset(),
                                    opcode: semantic_opcode,
                                })?,
                            )
                        } else {
                            None
                        };
                        self.push_loop_iteration_environment(agent, frame, site, mirrored_slot)?;
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::PopClosureEnv => {
                        let (ax, feedback_slot, instruction_len) =
                            decode_ax_operands(bytes, is_profiled, code, instruction_offset)?;

                        self.pop_loop_iteration_environment();
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::PushWithEnv => {
                        let (ax, feedback_slot, instruction_len) =
                            decode_ax_operands(bytes, is_profiled, code, instruction_offset)?;

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
                        let (ax, feedback_slot, instruction_len) =
                            decode_ax_operands(bytes, is_profiled, code, instruction_offset)?;

                        self.pop_with_environment(frame);
                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::TypeOf => {
                        let (ax, feedback_slot, instruction_len) =
                            decode_ax_operands(bytes, is_profiled, code, instruction_offset)?;

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
                        let (ax, feedback_slot, instruction_len) =
                            decode_ax_operands(bytes, is_profiled, code, instruction_offset)?;

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
                        let (ax, feedback_slot, instruction_len) =
                            decode_ax_operands(bytes, is_profiled, code, instruction_offset)?;

                        self.sync_dispatch_frame(frame_depth, *frame);
                        let _ = agent.pop_execution_context();
                        if let Some(result) = self.finish_frame(agent, Value::undefined())? {
                            return Ok(result);
                        }
                    }
                    Opcode::SuspendGeneratorStart => {
                        let (ax, feedback_slot, instruction_len) =
                            decode_ax_operands(bytes, is_profiled, code, instruction_offset)?;

                        let resume_offset =
                            next_dispatch_instruction_offset(frame, instruction_len);
                        self.sync_dispatch_frame(frame_depth, *frame);
                        self.suspend_generator_start(agent, frame, resume_offset)?;
                    }
                    Opcode::Yield => {
                        let (ax, feedback_slot, instruction_len) =
                            decode_ax_operands(bytes, is_profiled, code, instruction_offset)?;

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
                        let (ax, feedback_slot, instruction_len) =
                            decode_ax_operands(bytes, is_profiled, code, instruction_offset)?;

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
                        let (ax, feedback_slot, instruction_len) =
                            decode_ax_operands(bytes, is_profiled, code, instruction_offset)?;

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
                        let (ax, feedback_slot, instruction_len) =
                            decode_ax_operands(bytes, is_profiled, code, instruction_offset)?;

                        advance_dispatch_frame(frame, instruction_len);
                    }
                    Opcode::LoadException => {
                        let (ax, feedback_slot, instruction_len) =
                            decode_ax_operands(bytes, is_profiled, code, instruction_offset)?;

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
                        let (ax, feedback_slot, instruction_len) =
                            decode_ax_operands(bytes, is_profiled, code, instruction_offset)?;

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
                        let (ax, feedback_slot, instruction_len) =
                            decode_ax_operands(bytes, is_profiled, code, instruction_offset)?;

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
                            opcode: semantic_opcode,
                        });
                    }
                }
            }
        }
    }
}
