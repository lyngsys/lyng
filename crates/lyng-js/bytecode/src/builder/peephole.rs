use std::collections::{HashSet, VecDeque};

use crate::metadata::{
    DirectEvalLexicalSite, ExceptionHandler, FeedbackSiteDescriptor, LoopIterationEnvironmentSite,
    RuntimeStateCapture, SafepointDescriptor, WideAbxOperands, WideOperand,
};
use crate::{Instruction, Opcode};

use super::{BytecodeBuildError, BytecodeBuildResult, BytecodeBuilder, BytecodeLimitKind};

pub(super) fn optimize(builder: &mut BytecodeBuilder) -> BytecodeBuildResult<()> {
    if builder.instructions.len() < 2 {
        return Ok(());
    }

    loop {
        thread_jump_targets(builder)?;
        if !compact_unreachable_and_noop_jumps(builder)? {
            return Ok(());
        }
        if builder.instructions.len() < 2 {
            return Ok(());
        }
    }
}

fn thread_jump_targets(builder: &mut BytecodeBuilder) -> BytecodeBuildResult<()> {
    let offsets = (0..builder.instructions.len())
        .map(|offset| u32::try_from(offset).expect("builder offsets should fit u32"))
        .collect::<Vec<_>>();

    for offset in offsets {
        let Some(instruction) = builder
            .instructions
            .get(usize::try_from(offset).expect("offset should fit usize"))
            .copied()
        else {
            continue;
        };
        if !instruction.opcode().is_jump() {
            continue;
        }
        let Some(target) = jump_target(builder, offset, instruction) else {
            continue;
        };
        let threaded = threaded_target(builder, target);
        if threaded != target {
            builder.patch_jump_to(offset, threaded)?;
        }
    }

    Ok(())
}

fn compact_unreachable_and_noop_jumps(builder: &mut BytecodeBuilder) -> BytecodeBuildResult<bool> {
    let reachable = reachable_offsets(builder);
    let keep = builder
        .instructions
        .iter()
        .copied()
        .enumerate()
        .map(|(offset, instruction)| {
            reachable[offset]
                && !is_noop_jump(
                    builder,
                    u32::try_from(offset).expect("builder offsets should fit u32"),
                    instruction,
                )
        })
        .collect::<Vec<_>>();

    if keep.iter().all(|keep_instruction| *keep_instruction) {
        return Ok(false);
    }

    let compaction = CompactionMap::new(&keep)?;
    let old_instructions = std::mem::take(&mut builder.instructions);
    let old_wide_operands = std::mem::take(&mut builder.wide_operands);
    let mut new_instructions = Vec::with_capacity(compaction.new_len_usize());
    let mut new_wide_operands = Vec::new();

    for (old_offset, instruction) in old_instructions.iter().copied().enumerate() {
        let old_offset_u32 = u32::try_from(old_offset).expect("builder offsets should fit u32");
        let Some(new_offset) = compaction.kept_offset(old_offset_u32) else {
            continue;
        };
        let mut instruction = instruction;
        if instruction.opcode().is_jump() {
            rewrite_jump_instruction(
                &old_instructions,
                &old_wide_operands,
                &compaction,
                old_offset_u32,
                new_offset,
                &mut instruction,
                &mut new_wide_operands,
            )?;
        } else if let Some(payload) = wide_payload(&old_wide_operands, old_offset_u32) {
            new_wide_operands.push(WideOperand::new(new_offset, payload));
        }
        new_instructions.push(instruction);
    }

    builder.instructions = new_instructions;
    builder.wide_operands = new_wide_operands;
    remap_offset_metadata(builder, &compaction);
    Ok(true)
}

fn reachable_offsets(builder: &BytecodeBuilder) -> Vec<bool> {
    let len = builder.instructions.len();
    let mut reachable = vec![false; len];
    let mut queue = VecDeque::new();
    if len != 0 {
        queue.push_back(0);
    }
    for handler in &builder.exception_handlers {
        if let Ok(handler_offset) = usize::try_from(handler.handler())
            && handler_offset < len
        {
            queue.push_back(handler_offset);
        }
    }

    while let Some(offset) = queue.pop_front() {
        if offset >= len || reachable[offset] {
            continue;
        }
        reachable[offset] = true;
        let offset_u32 = u32::try_from(offset).expect("builder offsets should fit u32");
        match builder.instructions[offset] {
            Instruction::Ax {
                opcode: Opcode::Jump,
                ..
            } => {
                if let Some(target) = jump_target(builder, offset_u32, builder.instructions[offset])
                {
                    push_target(&mut queue, target, len);
                }
            }
            Instruction::Abx {
                opcode: Opcode::JumpIfTrue | Opcode::JumpIfFalse,
                ..
            } => {
                if let Some(target) = jump_target(builder, offset_u32, builder.instructions[offset])
                {
                    push_target(&mut queue, target, len);
                }
                push_target(&mut queue, offset_u32.saturating_add(1), len);
            }
            Instruction::Abc {
                opcode: Opcode::TailCall,
                ..
            }
            | Instruction::Ax {
                opcode: Opcode::Return | Opcode::ReturnUndefined | Opcode::Throw,
                ..
            } => {}
            _ => push_target(&mut queue, offset_u32.saturating_add(1), len),
        }
    }

    reachable
}

fn push_target(queue: &mut VecDeque<usize>, target: u32, len: usize) {
    let Ok(target) = usize::try_from(target) else {
        return;
    };
    if target < len {
        queue.push_back(target);
    }
}

fn is_noop_jump(builder: &BytecodeBuilder, offset: u32, instruction: Instruction) -> bool {
    instruction.opcode().is_jump()
        && jump_target(builder, offset, instruction) == Some(offset.saturating_add(1))
}

fn jump_target(builder: &BytecodeBuilder, offset: u32, instruction: Instruction) -> Option<u32> {
    let len = u32::try_from(builder.instructions.len()).ok()?;
    match instruction {
        Instruction::Ax {
            opcode: Opcode::Jump,
            ax,
        } => target_from_delta(offset, ax, len),
        Instruction::Abx {
            opcode: Opcode::JumpIfTrue | Opcode::JumpIfFalse,
            a,
            bx,
        } => {
            let operands = builder.decode_abx_operands(offset, a, bx);
            let delta = i32::from_le_bytes(operands.bx().to_le_bytes());
            target_from_delta(offset, delta, len)
        }
        _ => None,
    }
}

fn jump_target_from_parts(
    instructions: &[Instruction],
    wide_operands: &[WideOperand],
    offset: u32,
    instruction: Instruction,
) -> Option<u32> {
    let len = u32::try_from(instructions.len()).ok()?;
    match instruction {
        Instruction::Ax {
            opcode: Opcode::Jump,
            ax,
        } => target_from_delta(offset, ax, len),
        Instruction::Abx {
            opcode: Opcode::JumpIfTrue | Opcode::JumpIfFalse,
            a,
            bx,
        } => {
            let operands = decode_abx_operands(wide_operands, offset, a, bx);
            let delta = i32::from_le_bytes(operands.bx().to_le_bytes());
            target_from_delta(offset, delta, len)
        }
        _ => None,
    }
}

fn target_from_delta(offset: u32, delta: i32, instruction_len: u32) -> Option<u32> {
    let target = i64::from(offset) + 1 + i64::from(delta);
    if (0..=i64::from(instruction_len)).contains(&target) {
        u32::try_from(target).ok()
    } else {
        None
    }
}

fn threaded_target(builder: &BytecodeBuilder, target: u32) -> u32 {
    let mut target = target;
    let mut seen = HashSet::new();
    while seen.insert(target) {
        let Some(Instruction::Ax {
            opcode: Opcode::Jump,
            ..
        }) = builder
            .instructions
            .get(usize::try_from(target).unwrap_or(usize::MAX))
            .copied()
        else {
            break;
        };
        let Some(next) = jump_target(
            builder,
            target,
            builder.instructions[usize::try_from(target).expect("target should fit usize")],
        ) else {
            break;
        };
        if next == target {
            break;
        }
        target = next;
    }
    target
}

fn threaded_target_from_parts(
    instructions: &[Instruction],
    wide_operands: &[WideOperand],
    target: u32,
) -> u32 {
    let mut target = target;
    let mut seen = HashSet::new();
    while seen.insert(target) {
        let Some(instruction) = instructions
            .get(usize::try_from(target).unwrap_or(usize::MAX))
            .copied()
        else {
            break;
        };
        if !matches!(
            instruction,
            Instruction::Ax {
                opcode: Opcode::Jump,
                ..
            }
        ) {
            break;
        }
        let Some(next) = jump_target_from_parts(instructions, wide_operands, target, instruction)
        else {
            break;
        };
        if next == target {
            break;
        }
        target = next;
    }
    target
}

fn rewrite_jump_instruction(
    old_instructions: &[Instruction],
    old_wide_operands: &[WideOperand],
    compaction: &CompactionMap,
    old_offset: u32,
    new_offset: u32,
    instruction: &mut Instruction,
    new_wide_operands: &mut Vec<WideOperand>,
) -> BytecodeBuildResult<()> {
    let Some(old_target) = jump_target_from_parts(
        old_instructions,
        old_wide_operands,
        old_offset,
        *instruction,
    ) else {
        return Ok(());
    };
    let threaded = threaded_target_from_parts(old_instructions, old_wide_operands, old_target);
    let Some(new_target) = compaction.remap_boundary(threaded) else {
        return Ok(());
    };
    let delta = i64::from(new_target) - (i64::from(new_offset) + 1);
    match instruction {
        Instruction::Ax {
            opcode: Opcode::Jump,
            ax,
        } => {
            *ax = i32::try_from(delta).map_err(|_| BytecodeBuildError::JumpDeltaOverflow {
                instruction_offset: old_offset,
                target_offset: old_target,
            })?;
        }
        Instruction::Abx {
            opcode: Opcode::JumpIfTrue | Opcode::JumpIfFalse,
            a,
            bx,
        } => {
            let operands = decode_abx_operands(old_wide_operands, old_offset, *a, *bx);
            let delta =
                i32::try_from(delta).map_err(|_| BytecodeBuildError::JumpDeltaOverflow {
                    instruction_offset: old_offset,
                    target_offset: old_target,
                })?;
            let updated =
                WideAbxOperands::new(operands.a(), u32::from_le_bytes(delta.to_le_bytes()));
            *a = updated.narrow_a();
            *bx = updated.narrow_bx();
            if updated.needs_wide() {
                new_wide_operands.push(WideOperand::new(new_offset, updated.encode_payload()));
            }
        }
        _ => {}
    }
    Ok(())
}

fn remap_offset_metadata(builder: &mut BytecodeBuilder, compaction: &CompactionMap) {
    builder.direct_eval_lexical_sites = builder
        .direct_eval_lexical_sites
        .drain(..)
        .filter_map(|site| {
            compaction
                .kept_offset(site.instruction_offset())
                .map(|instruction_offset| {
                    DirectEvalLexicalSite::new(
                        instruction_offset,
                        site.scopes().to_vec(),
                        site.flags(),
                        site.annex_b_catch_names().to_vec(),
                        site.parameter_names().to_vec(),
                    )
                })
        })
        .collect();

    builder.loop_iteration_environment_sites = builder
        .loop_iteration_environment_sites
        .drain(..)
        .filter_map(|site| {
            compaction
                .kept_offset(site.instruction_offset())
                .map(|instruction_offset| {
                    LoopIterationEnvironmentSite::new(
                        instruction_offset,
                        site.iteration_slots().to_vec(),
                        site.shared_slots().to_vec(),
                        site.detached_slots().to_vec(),
                    )
                })
        })
        .collect();

    builder.feedback_sites = builder
        .feedback_sites
        .drain(..)
        .filter_map(|site| {
            compaction
                .kept_offset(site.instruction_offset())
                .map(|instruction_offset| {
                    FeedbackSiteDescriptor::new(site.slot(), instruction_offset, site.kind())
                        .with_metadata(site.metadata())
                })
        })
        .collect();

    builder.source_map = builder
        .source_map
        .drain(..)
        .filter_map(|entry| {
            compaction
                .kept_offset(entry.instruction_offset())
                .map(|instruction_offset| {
                    crate::SourceMapEntry::new(
                        entry.source(),
                        instruction_offset,
                        entry.start(),
                        entry.end(),
                    )
                })
        })
        .collect();

    builder.exception_handlers = builder
        .exception_handlers
        .drain(..)
        .filter_map(|handler| remap_exception_handler(handler, compaction))
        .collect();

    let mut kept_safepoints = HashSet::new();
    builder.safepoints = builder
        .safepoints
        .drain(..)
        .filter_map(|descriptor| {
            compaction
                .kept_offset(descriptor.instruction_offset())
                .map(|instruction_offset| {
                    kept_safepoints.insert(descriptor.id());
                    remap_safepoint(descriptor, instruction_offset)
                })
        })
        .collect();

    builder
        .deopt_snapshots
        .retain(|snapshot| kept_safepoints.contains(&snapshot.safepoint_id()));
}

fn remap_exception_handler(
    handler: ExceptionHandler,
    compaction: &CompactionMap,
) -> Option<ExceptionHandler> {
    let start = compaction.remap_boundary(handler.protected_start())?;
    let end = compaction.remap_boundary(handler.protected_end())?;
    let handler_offset = compaction.remap_boundary(handler.handler())?;
    (start < end && handler_offset < compaction.new_len()).then(|| {
        ExceptionHandler::new(
            start,
            end,
            handler_offset,
            handler.kind(),
            handler.stack_depth(),
            handler.target_register(),
        )
    })
}

const fn remap_safepoint(
    descriptor: SafepointDescriptor,
    instruction_offset: u32,
) -> SafepointDescriptor {
    let runtime_state = RuntimeStateCapture::new()
        .with_lexical_env(descriptor.captures_lexical_env())
        .with_variable_env(descriptor.captures_variable_env())
        .with_this_value(descriptor.captures_this())
        .with_new_target(descriptor.captures_new_target())
        .with_callee(descriptor.captures_callee())
        .with_exception_state(descriptor.captures_exception_state())
        .with_completion_state(descriptor.captures_completion_state());
    SafepointDescriptor::new(
        descriptor.id(),
        instruction_offset,
        descriptor.kind(),
        descriptor.register_window_len(),
    )
    .with_environment_layout(descriptor.environment_layout())
    .with_runtime_state(runtime_state)
}

fn decode_abx_operands(
    wide_operands: &[WideOperand],
    instruction_offset: u32,
    a: u8,
    bx: u16,
) -> WideAbxOperands {
    wide_payload(wide_operands, instruction_offset).map_or_else(
        || WideAbxOperands::narrow(a, bx),
        |payload| WideAbxOperands::decode(a, bx, payload),
    )
}

fn wide_payload(wide_operands: &[WideOperand], instruction_offset: u32) -> Option<u32> {
    wide_operands.iter().find_map(|operand| {
        (operand.instruction_offset() == instruction_offset).then_some(operand.payload())
    })
}

struct CompactionMap {
    kept_offsets: Vec<Option<u32>>,
    next_offsets: Vec<Option<u32>>,
    old_len: u32,
    new_len: u32,
}

impl CompactionMap {
    fn new(keep: &[bool]) -> BytecodeBuildResult<Self> {
        let mut kept_offsets = vec![None; keep.len()];
        let mut next_new_offset = 0u32;
        for (old_offset, keep_instruction) in keep.iter().copied().enumerate() {
            if keep_instruction {
                kept_offsets[old_offset] = Some(next_new_offset);
                next_new_offset =
                    next_new_offset
                        .checked_add(1)
                        .ok_or(BytecodeBuildError::LimitExceeded {
                            kind: BytecodeLimitKind::InstructionStream,
                        })?;
            }
        }

        let mut next_offsets = vec![Some(next_new_offset); keep.len() + 1];
        let mut next = Some(next_new_offset);
        for old_offset in (0..keep.len()).rev() {
            if let Some(kept) = kept_offsets[old_offset] {
                next = Some(kept);
            }
            next_offsets[old_offset] = next;
        }

        Ok(Self {
            kept_offsets,
            next_offsets,
            old_len: u32::try_from(keep.len()).map_err(|_| BytecodeBuildError::LimitExceeded {
                kind: BytecodeLimitKind::InstructionStream,
            })?,
            new_len: next_new_offset,
        })
    }

    fn kept_offset(&self, old_offset: u32) -> Option<u32> {
        self.kept_offsets
            .get(usize::try_from(old_offset).ok()?)
            .copied()
            .flatten()
    }

    fn remap_boundary(&self, old_offset: u32) -> Option<u32> {
        if old_offset == self.old_len {
            return Some(self.new_len);
        }
        self.next_offsets
            .get(usize::try_from(old_offset).ok()?)
            .copied()
            .flatten()
    }

    const fn new_len(&self) -> u32 {
        self.new_len
    }

    fn new_len_usize(&self) -> usize {
        usize::try_from(self.new_len).expect("new instruction length should fit usize")
    }
}
