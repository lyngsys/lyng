use super::registers::absolute_register;
use super::{
    Agent, CodeRef, FrameRecord, HostHooks, NativeFunctionRegistry, Opcode, Value, Vm, VmError,
    VmResult,
};
use lyng_types::{AbruptCompletion, FeedbackSlotId};

pub(in crate::vm) mod arithmetic;
pub(in crate::vm) mod property;

#[inline]
pub(in crate::vm) const fn advance_dispatch_frame(frame: &mut FrameRecord, encoded_len: u32) {
    let next = frame
        .instruction_offset()
        .checked_add(encoded_len)
        .expect("instruction offset should stay within u32");
    frame.set_instruction_offset(next);
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
pub fn decode_abc_operands(
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
pub fn decode_abx_operands(
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

    /// Sole VM dispatch entry point.
    ///
    /// DSL-0c (Task C1) flipped this from `run_via_trampoline` (α path
    /// using `DispatchState` + `DISPATCH_TABLE` + `dispatch_handlers/`)
    /// to `run_via_dsl` (asm-DSL trampoline + `DSL_DISPATCH_TABLE`).
    /// Task C5 deleted the α trampoline functions themselves; the
    /// `dispatch_handlers/` modules + `DISPATCH_TABLE` survive only for
    /// the wide-form prefix bridge in `dsl::handlers::warm::op_prefix_via_alpha`.
    pub(super) fn run(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> VmResult<Value> {
        let result = self.run_via_dsl(agent, host, registry);
        self.drain_llint_scalar_feedback();
        result
    }
}

#[cfg(test)]
mod tests {
    /// Phase 1 sub-8 (`lyng-9gyk`) exit invariant: dispatch.rs must contain
    /// **no** `match` expression with more than 10 arms.
    ///
    /// The roadmap (`reports/lyng/jsc-aligned-engine-roadmap.md`) chose
    /// Option α — per-handler `extern "C" fn` table with a central trampoline
    /// — over the legacy single-`match` interpreter. If a regression
    /// reintroduces a wide opcode-match in this file, it would re-grow the
    /// dispatch jump table that Track H + Phase 1 spent two epics deleting.
    /// Catch it at the source level.
    ///
    /// "Wide" here means more than 10 arms in a single `match`. Small matches
    /// (e.g., on `prefix == Opcode::ExtraWide` in wide-decode helpers,
    /// short `match` over `AbruptCompletion`, etc.) are fine.
    ///
    /// DSL-0c C2 update: the only large opcode-match in the workspace now
    /// lives in `crate::dsl::handlers::cold::dispatch_wide_form`, which the
    /// codegen tool emits and which `dsl::handlers::warm` consults for
    /// `Wide` / `ExtraWide` dispatch — it deliberately replaces the deleted
    /// α `DISPATCH_TABLE` indirection with a single, code-generated match.
    /// This file (dispatch.rs) only holds shared decoders + helpers.
    #[test]
    fn dispatch_rs_contains_no_match_over_10_arms() {
        let source = include_str!("dispatch.rs");
        let mut arms_per_match: Vec<usize> = Vec::new();
        let mut depth: usize = 0;
        let mut stack: Vec<(usize, usize)> = Vec::new(); // (open_depth, arm_count)
        let mut chars = source.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '{' => {
                    depth += 1;
                }
                '}' => {
                    if let Some(&(open_depth, count)) = stack.last()
                        && open_depth == depth
                    {
                        arms_per_match.push(count);
                        stack.pop();
                    }
                    depth = depth.saturating_sub(1);
                }
                '=' if chars.peek() == Some(&'>') => {
                    chars.next();
                    if let Some(top) = stack.last_mut() {
                        top.1 += 1;
                    }
                }
                'm' => {
                    let mut buf = String::from("m");
                    for _ in 0..4 {
                        if let Some(&p) = chars.peek() {
                            buf.push(p);
                            chars.next();
                        }
                    }
                    if buf == "match" {
                        // Consume until the '{' that opens the match body.
                        for c in chars.by_ref() {
                            if c == '{' {
                                depth += 1;
                                stack.push((depth, 0));
                                break;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        let max_arms = arms_per_match.iter().copied().max().unwrap_or(0);
        assert!(
            max_arms <= 10,
            "dispatch.rs contains a match with {max_arms} arms; Phase 1 sub-8 invariant is ≤ 10. \
             A wide opcode match here would re-grow the dispatch jump table that Track H + \
             trampoline cutover (`lyng-33i2`) eliminated. The wide-form opcode match lives in \
             `crate::dsl::handlers::cold::dispatch_wide_form` (codegen-emitted); add new opcodes \
             via the codegen tool, not by hand-rolling a match here.",
        );
    }
}
