use crate::{metadata::CallRange, Instruction, Opcode};
use lyng_js_types::FeedbackSlotId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InstructionForm {
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

/// Decoder error for one malformed instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    InvalidOpcode(u8),
    InvalidFeedbackSlot(u16),
    InvalidPrefixStack { prefix: Opcode, next: Opcode },
    UnexpectedPrefix { prefix: Opcode, opcode: Opcode },
    TruncatedInstruction { len: usize },
}

/// One invalid word encountered while decoding an opaque byte stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidInstructionWord {
    word_index: u32,
    raw_opcode: u8,
}

impl InvalidInstructionWord {
    #[inline]
    pub const fn new(word_index: u32, raw_opcode: u8) -> Self {
        Self {
            word_index,
            raw_opcode,
        }
    }

    #[inline]
    pub const fn word_index(self) -> u32 {
        self.word_index
    }

    #[inline]
    pub const fn raw_opcode(self) -> u8 {
        self.raw_opcode
    }
}

/// Best-effort decoded instruction stream built from opaque bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedInstructionStream {
    instructions: Vec<Instruction>,
    invalid_words: Vec<InvalidInstructionWord>,
    trailing_byte_count: usize,
}

impl DecodedInstructionStream {
    #[inline]
    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    #[inline]
    pub fn invalid_words(&self) -> &[InvalidInstructionWord] {
        &self.invalid_words
    }

    #[inline]
    pub const fn trailing_byte_count(&self) -> usize {
        self.trailing_byte_count
    }
}

/// Decode one instruction word without panicking on malformed opcode bytes.
///
/// # Errors
/// Returns [`DecodeError::InvalidOpcode`] when the opcode byte does not map to a known
/// instruction.
pub fn decode_instruction_word(word: u32) -> Result<Instruction, DecodeError> {
    decode_instruction_bytes(&word.to_le_bytes())
}

/// Decode one instruction from raw template bytes.
///
/// # Errors
/// Returns [`DecodeError::TruncatedInstruction`] when fewer bytes than the opcode requires are
/// supplied, [`DecodeError::InvalidOpcode`] when the opcode byte is not recognized, and prefix
/// errors when a `Wide` / `ExtraWide` byte appears outside the durable prefix position.
#[allow(
    clippy::too_many_lines,
    reason = "the decoder keeps the byte layout cases together for prefix auditability"
)]
pub fn decode_instruction_bytes(bytes: &[u8]) -> Result<Instruction, DecodeError> {
    let [raw_opcode, ..] = bytes else {
        return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
    };
    let first = Opcode::from_byte(*raw_opcode).ok_or(DecodeError::InvalidOpcode(*raw_opcode))?;
    let (prefix, opcode_offset, opcode) = if first.is_prefix() {
        let raw_semantic = *bytes
            .get(1)
            .ok_or(DecodeError::TruncatedInstruction { len: bytes.len() })?;
        let semantic =
            Opcode::from_byte(raw_semantic).ok_or(DecodeError::InvalidOpcode(raw_semantic))?;
        if semantic.is_prefix() {
            return Err(DecodeError::InvalidPrefixStack {
                prefix: first,
                next: semantic,
            });
        }
        (Some(first), 1usize, semantic)
    } else {
        (None, 0usize, first)
    };
    if opcode.is_prefix() {
        return Err(DecodeError::InvalidPrefixStack {
            prefix: opcode,
            next: opcode,
        });
    }

    let form = instruction_form(opcode);
    let instruction = match form {
        InstructionForm::Abc => {
            let (a, b, c, slot_offset) = decode_abc(bytes, opcode_offset, prefix, opcode)?;
            if opcode.has_feedback_slot() {
                let slot = decode_feedback_slot(bytes, slot_offset)?;
                Instruction::abc_slot(opcode, a, b, c, slot)
            } else {
                Instruction::abc(opcode, a, b, c)
            }
        }
        InstructionForm::Abx => {
            let (a, bx, slot_offset) = decode_abx(bytes, opcode_offset, prefix, opcode)?;
            if opcode.has_feedback_slot() {
                let slot = decode_feedback_slot(bytes, slot_offset)?;
                Instruction::abx_slot(opcode, a, bx, slot)
            } else {
                Instruction::abx(opcode, a, bx)
            }
        }
        InstructionForm::Abx8 => {
            reject_prefix(prefix, opcode)?;
            let [_, a, bx, ..] = bytes else {
                return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
            };
            Instruction::abx(opcode, u16::from(*a), u32::from(*bx))
        }
        InstructionForm::Ax => {
            reject_prefix(prefix, opcode)?;
            let [_, first, second, third, ..] = bytes else {
                return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
            };
            Instruction::ax(opcode, sign_extend_i24([*first, *second, *third]))
        }
        InstructionForm::Ax8 => {
            reject_prefix(prefix, opcode)?;
            let [_, ax, ..] = bytes else {
                return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
            };
            Instruction::ax(opcode, i32::from(i8::from_le_bytes([*ax])))
        }
        InstructionForm::Local => {
            reject_prefix(prefix, opcode)?;
            let [_, register, ..] = bytes else {
                return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
            };
            Instruction::abx(opcode, u16::from(*register), 0)
        }
        InstructionForm::Accumulator => {
            reject_prefix(prefix, opcode)?;
            Instruction::abc(opcode, 0, 0, 0)
        }
        InstructionForm::AccumulatorByte => {
            reject_prefix(prefix, opcode)?;
            let [_, operand, ..] = bytes else {
                return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
            };
            Instruction::abx(opcode, 0, u32::from(*operand))
        }
        InstructionForm::AccumulatorRegister => {
            reject_prefix(prefix, opcode)?;
            let [_, register, ..] = bytes else {
                return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
            };
            Instruction::abc(opcode, u16::from(*register), 0, 0)
        }
        InstructionForm::CallRange => {
            reject_prefix(prefix, opcode)?;
            let [_, a, b, c, count_low, count_high, base_low, base_high, ..] = bytes else {
                return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
            };
            let range = CallRange::new(
                u16::from_le_bytes([*base_low, *base_high]),
                u16::from_le_bytes([*count_low, *count_high]),
            );
            // After Track H every Call/TailCall/Construct carries a mandatory slot.
            let slot = decode_feedback_slot(bytes, 8)?;
            Instruction::CallRange {
                opcode,
                a: u16::from(*a),
                b: u16::from(*b),
                c: u16::from(*c),
                range,
                slot,
            }
        }
    };
    Ok(instruction)
}

/// Decode as many full instructions as possible from an opaque byte buffer.
///
/// Invalid opcode bytes are recorded and skipped. Trailing bytes that do not form a full
/// instruction are also reported.
///
/// # Panics
/// Panics if the decoded word index does not fit in `u32`.
pub fn decode_instruction_stream(bytes: &[u8]) -> DecodedInstructionStream {
    let mut instructions = Vec::new();
    let mut invalid_words = Vec::new();
    let mut trailing_byte_count = 0;
    let mut byte_offset = 0usize;
    let mut word_index = 0u32;

    while byte_offset < bytes.len() {
        match decode_instruction_bytes(&bytes[byte_offset..]) {
            Ok(instruction) => {
                byte_offset += instruction.encoded_len();
                instructions.push(instruction);
            }
            Err(DecodeError::InvalidOpcode(raw_opcode)) => {
                invalid_words.push(InvalidInstructionWord::new(word_index, raw_opcode));
                byte_offset += 1;
            }
            Err(DecodeError::InvalidFeedbackSlot(raw_slot)) => {
                invalid_words.push(InvalidInstructionWord::new(
                    word_index,
                    u8::try_from(raw_slot).unwrap_or(bytes[byte_offset]),
                ));
                byte_offset += 1;
            }
            Err(DecodeError::InvalidPrefixStack { .. } | DecodeError::UnexpectedPrefix { .. }) => {
                invalid_words.push(InvalidInstructionWord::new(word_index, bytes[byte_offset]));
                byte_offset += 1;
            }
            Err(DecodeError::TruncatedInstruction { .. }) => {
                trailing_byte_count = bytes.len() - byte_offset;
                break;
            }
        }
        word_index = word_index
            .checked_add(1)
            .expect("decoded instruction index should fit u32");
    }

    DecodedInstructionStream {
        instructions,
        invalid_words,
        trailing_byte_count,
    }
}

const fn reject_prefix(prefix: Option<Opcode>, opcode: Opcode) -> Result<(), DecodeError> {
    if let Some(prefix) = prefix {
        return Err(DecodeError::UnexpectedPrefix { prefix, opcode });
    }
    Ok(())
}

fn decode_abc(
    bytes: &[u8],
    opcode_offset: usize,
    prefix: Option<Opcode>,
    opcode: Opcode,
) -> Result<(u16, u16, u16, usize), DecodeError> {
    if prefix.is_some() {
        let [_, _, a_low, b_low, c_low, a_high, b_high, c_high, ..] = bytes else {
            return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
        };
        return Ok((
            u16::from_le_bytes([*a_low, *a_high]),
            u16::from_le_bytes([*b_low, *b_high]),
            u16::from_le_bytes([*c_low, *c_high]),
            8,
        ));
    }
    let operands = bytes
        .get(opcode_offset + 1..)
        .ok_or(DecodeError::TruncatedInstruction { len: bytes.len() })?;
    let [a, b, c, ..] = operands else {
        return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
    };
    if opcode.is_prefix() {
        return Err(DecodeError::InvalidPrefixStack {
            prefix: opcode,
            next: opcode,
        });
    }
    Ok((u16::from(*a), u16::from(*b), u16::from(*c), 4))
}

fn decode_abx(
    bytes: &[u8],
    opcode_offset: usize,
    prefix: Option<Opcode>,
    _opcode: Opcode,
) -> Result<(u16, u32, usize), DecodeError> {
    if let Some(prefix) = prefix {
        let [_, _, a_low, bx0, bx1, a_high, bx2, bx3, ..] = bytes else {
            return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
        };
        let bx3 = if prefix == Opcode::ExtraWide { *bx3 } else { 0 };
        return Ok((
            u16::from_le_bytes([*a_low, *a_high]),
            u32::from_le_bytes([*bx0, *bx1, *bx2, bx3]),
            8,
        ));
    }
    let operands = bytes
        .get(opcode_offset + 1..)
        .ok_or(DecodeError::TruncatedInstruction { len: bytes.len() })?;
    let [a, bx_low, bx_high, ..] = operands else {
        return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
    };
    Ok((
        u16::from(*a),
        u32::from(u16::from_le_bytes([*bx_low, *bx_high])),
        4,
    ))
}

fn decode_feedback_slot(bytes: &[u8], offset: usize) -> Result<FeedbackSlotId, DecodeError> {
    let [low, high, ..] = bytes
        .get(offset..)
        .ok_or(DecodeError::TruncatedInstruction { len: bytes.len() })?
    else {
        return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
    };
    let raw_slot = u16::from_le_bytes([*low, *high]);
    FeedbackSlotId::from_raw(u32::from(raw_slot)).ok_or(DecodeError::InvalidFeedbackSlot(raw_slot))
}

#[allow(
    clippy::too_many_lines,
    reason = "the decoder keeps one exhaustive opcode-to-encoding table for auditability"
)]
const fn instruction_form(opcode: Opcode) -> InstructionForm {
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
        | Opcode::LoadResumeValue => InstructionForm::Ax,
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
        | Opcode::CloseIterator => InstructionForm::Abx,
        Opcode::LoadSmi8 | Opcode::LoadConst8 | Opcode::JumpIfTrue8 | Opcode::JumpIfFalse8 => {
            InstructionForm::Abx8
        }
        Opcode::Jump8 => InstructionForm::Ax8,
        Opcode::LoadLocal0
        | Opcode::LoadLocal1
        | Opcode::LoadLocal2
        | Opcode::LoadLocal3
        | Opcode::StoreLocal0
        | Opcode::StoreLocal1
        | Opcode::StoreLocal2
        | Opcode::StoreLocal3 => InstructionForm::Local,
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
        | Opcode::Star7 => InstructionForm::Accumulator,
        Opcode::LdaSmi8 | Opcode::LdaConst8 => InstructionForm::AccumulatorByte,
        Opcode::Ldar => InstructionForm::AccumulatorRegister,
        Opcode::Call | Opcode::TailCall | Opcode::Construct => InstructionForm::CallRange,
        _ => InstructionForm::Abc,
    }
}

const fn sign_extend_i24(bytes: [u8; 3]) -> i32 {
    let sign = if bytes[2] & 0x80 == 0 { 0 } else { 0xff };
    i32::from_le_bytes([bytes[0], bytes[1], bytes[2], sign])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_abc_decodes_with_inline_feedback_slot() {
        // Add is IC-shaped, so the wide encoding carries the 2-byte slot inline
        // after the 8-byte wide ABC envelope.
        let decoded = decode_instruction_bytes(&[
            Opcode::Wide as u8,
            Opcode::Add as u8,
            0x23,
            0x45,
            0xab,
            0x01,
            0x00,
            0x01,
            1,
            0,
        ])
        .expect("wide add should decode");
        assert_eq!(
            decoded,
            Instruction::abc_slot(Opcode::Add, 0x0123, 0x0045, 0x01ab, slot(1))
        );
    }

    #[test]
    fn extra_wide_abx_decodes_u32_payload() {
        let decoded = decode_instruction_bytes(&[
            Opcode::ExtraWide as u8,
            Opcode::LoadGlobal as u8,
            0x02,
            0x04,
            0x03,
            0x01,
            0x02,
            0x01,
            1,
            0,
        ])
        .expect("extra-wide global load should decode");
        assert_eq!(
            decoded,
            Instruction::abx_slot(Opcode::LoadGlobal, 0x0102, 0x0102_0304, slot(1))
        );
    }

    #[test]
    fn prefix_stacking_is_rejected() {
        assert!(matches!(
            decode_instruction_bytes(&[Opcode::Wide as u8, Opcode::ExtraWide as u8]),
            Err(DecodeError::InvalidPrefixStack { .. })
        ));
    }

    #[test]
    fn call_decodes_inline_range_and_slot() {
        let decoded = decode_instruction_bytes(&[Opcode::Call as u8, 1, 2, 3, 5, 0, 4, 0, 1, 0])
            .expect("call should decode");
        assert_eq!(
            decoded,
            Instruction::CallRange {
                opcode: Opcode::Call,
                a: 1,
                b: 2,
                c: 3,
                range: CallRange::new(4, 5),
                slot: slot(1)
            }
        );
    }

    fn slot(raw: u32) -> FeedbackSlotId {
        FeedbackSlotId::from_raw(raw).expect("test slot should be non-zero")
    }
}
