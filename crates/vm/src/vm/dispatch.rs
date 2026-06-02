use super::registers::absolute_register;
use super::{
    Agent, CodeRef, HostHooks, NativeFunctionRegistry, Opcode, Value, Vm, VmError, VmResult,
};
use lyng_types::{AbruptCompletion, FeedbackSlotId};

pub(in crate::vm) mod arithmetic;
pub(in crate::vm) mod property;

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
/// essentially zero share on real workloads, and per-handler asm should stay
/// compact without inline wide code competing for L1i.
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
    /// Park `pc` into the overlay `saved_pc` of the frame at `cfr`.
    /// Read back by the slow-path `Refresh` arm and `finish_frame`.
    #[inline]
    pub(in crate::vm) fn park_caller_pc(&mut self, cfr: u32, pc: u32) {
        self.frame_header_mut(cfr).set_saved_pc(pc);
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
        cfr: u32,
        pc: u32,
        result: VmResult<T>,
    ) -> VmResult<Option<T>> {
        match result {
            Ok(value) => Ok(Some(value)),
            Err(VmError::Abrupt(AbruptCompletion::Throw(value))) => {
                // Park the live PC so the handler-covering search uses the correct PC.
                // On a caught throw, `transfer_to_exception_handler` overwrites
                // `saved_pc` with the handler PC; the caller reloads from it.
                if frame_depth != 0 {
                    self.park_caller_pc(cfr, pc);
                }
                if self.transfer_to_exception_handler(agent, value)? {
                    self.request_dispatch_frame_check();
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
        cfr: u32,
        pc: &mut u32,
        code: CodeRef,
        window: crate::frame::RegisterWindow,
        instruction_len: u32,
        feedback_slot: Option<FeedbackSlotId>,
        target_register: u16,
        result: VmResult<Value>,
    ) -> VmResult<()> {
        if let Some(value) = self.handle_dispatch_result(agent, frame_depth, cfr, *pc, result)? {
            self.record_feedback_slot(code, feedback_slot);
            let target = absolute_register(window, target_register);
            self.arena.slots_mut()[target] = value;
            *pc = pc.wrapping_add(instruction_len);
        } else {
            // Same-frame caught throw: handler PC was parked in overlay `saved_pc`.
            // Cross-frame catch is promoted to Refresh, which overwrites this value.
            *pc = self.frame_header(cfr).saved_pc();
        }
        Ok(())
    }

    /// Sole VM dispatch entry point.
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
    /// Dispatch invariant: this file must contain **no** `match` expression
    /// with more than 10 arms.
    ///
    /// The asm-DSL `LLInt` dispatcher uses a handler table + tail dispatch. A
    /// wide opcode-`match` here would re-grow the jump table the handler table
    /// replaced. The only large opcode-`match` lives in
    /// `crate::dsl::handlers::cold::dispatch_wide_form` (codegen-emitted).
    /// "Wide" means more than 10 arms; small matches on e.g. `AbruptCompletion`
    /// or `Opcode::ExtraWide` are fine.
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
            "dispatch.rs contains a match with {max_arms} arms; dispatch invariant is <= 10. \
             A wide opcode match here would re-grow the dispatch jump table that the asm-DSL \
             substrate eliminated. The wide-form opcode match lives in \
             `crate::dsl::handlers::cold::dispatch_wide_form` (codegen-emitted); add new opcodes \
             via the codegen tool, not by hand-rolling a match here.",
        );
    }
}
