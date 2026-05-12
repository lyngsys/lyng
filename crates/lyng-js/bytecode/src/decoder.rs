use crate::{Instruction, Opcode};
use lyng_js_types::FeedbackSlotId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InstructionForm {
    Abc,
    Abx,
    Abx8,
    Ax,
    Ax8,
    Local,
    ProfiledAbc,
    ProfiledAbx,
}

/// Decoder error for one malformed 4-byte instruction word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    InvalidOpcode(u8),
    InvalidFeedbackSlot(u16),
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
/// supplied and
/// [`DecodeError::InvalidOpcode`] when the opcode byte is not recognized.
pub fn decode_instruction_bytes(bytes: &[u8]) -> Result<Instruction, DecodeError> {
    let [raw_opcode, operands @ ..] = bytes else {
        return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
    };
    let opcode = Opcode::from_byte(*raw_opcode).ok_or(DecodeError::InvalidOpcode(*raw_opcode))?;

    Ok(match instruction_form(opcode) {
        InstructionForm::Abc => {
            let [a, b, c, ..] = operands else {
                return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
            };
            Instruction::abc(opcode, *a, *b, *c)
        }
        InstructionForm::Abx => {
            let [a, bx_low, bx_high, ..] = operands else {
                return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
            };
            Instruction::abx(opcode, *a, u16::from_le_bytes([*bx_low, *bx_high]))
        }
        InstructionForm::Abx8 => {
            let [a, bx, ..] = operands else {
                return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
            };
            Instruction::abx(opcode, *a, u16::from(*bx))
        }
        InstructionForm::Ax => {
            let [first, second, third, ..] = operands else {
                return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
            };
            Instruction::ax(opcode, sign_extend_i24([*first, *second, *third]))
        }
        InstructionForm::Ax8 => {
            let [ax, ..] = operands else {
                return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
            };
            Instruction::ax(opcode, i32::from(i8::from_le_bytes([*ax])))
        }
        InstructionForm::Local => {
            let [register, ..] = operands else {
                return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
            };
            Instruction::abx(opcode, *register, 0)
        }
        InstructionForm::ProfiledAbc => {
            let [raw_opcode, a, b, c, slot_low, slot_high, ..] = operands else {
                return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
            };
            let opcode =
                Opcode::from_byte(*raw_opcode).ok_or(DecodeError::InvalidOpcode(*raw_opcode))?;
            let raw_slot = u16::from_le_bytes([*slot_low, *slot_high]);
            let slot = FeedbackSlotId::from_raw(u32::from(raw_slot))
                .ok_or(DecodeError::InvalidFeedbackSlot(raw_slot))?;
            Instruction::profiled_abc(opcode, *a, *b, *c, slot)
        }
        InstructionForm::ProfiledAbx => {
            let [raw_opcode, a, bx_low, bx_high, slot_low, slot_high, ..] = operands else {
                return Err(DecodeError::TruncatedInstruction { len: bytes.len() });
            };
            let opcode =
                Opcode::from_byte(*raw_opcode).ok_or(DecodeError::InvalidOpcode(*raw_opcode))?;
            let raw_slot = u16::from_le_bytes([*slot_low, *slot_high]);
            let slot = FeedbackSlotId::from_raw(u32::from(raw_slot))
                .ok_or(DecodeError::InvalidFeedbackSlot(raw_slot))?;
            Instruction::profiled_abx(opcode, *a, u16::from_le_bytes([*bx_low, *bx_high]), slot)
        }
    })
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
            Err(DecodeError::InvalidFeedbackSlot(_)) => {
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
        Opcode::ProfiledAbc => InstructionForm::ProfiledAbc,
        Opcode::ProfiledAbx => InstructionForm::ProfiledAbx,
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
        | Opcode::Call
        | Opcode::CallMethod
        | Opcode::TailCall
        | Opcode::Construct
        | Opcode::CreateForIn
        | Opcode::AdvanceForIn
        | Opcode::CreateIterator
        | Opcode::AdvanceIterator
        | Opcode::DelegateYield => InstructionForm::Abc,
    }
}

const fn sign_extend_i24(bytes: [u8; 3]) -> i32 {
    let sign = if bytes[2] & 0x80 == 0 { 0 } else { 0xff };
    i32::from_le_bytes([bytes[0], bytes[1], bytes[2], sign])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        disassemble, disassemble_instruction, ArgumentsMode, BytecodeFunction, BytecodeFunctionId,
        ConstantValue, WideOperand,
    };

    #[test]
    fn decode_instruction_word_round_trips_abc_abx_and_ax_forms() {
        let abc = Instruction::abc(Opcode::Add, 1, 2, 3);
        let abx = Instruction::abx(Opcode::LoadConst, 7, 0x3412);
        let ax = Instruction::ax(Opcode::Jump, -7);

        assert_eq!(decode_instruction_word(abc.encode_word()), Ok(abc));
        assert_eq!(decode_instruction_word(abx.encode_word()), Ok(abx));
        assert_eq!(decode_instruction_word(ax.encode_word()), Ok(ax));
    }

    #[test]
    fn decode_instruction_stream_tracks_invalid_words_and_trailing_bytes() {
        let add = Instruction::abc(Opcode::Add, 1, 2, 3)
            .encode_word()
            .to_le_bytes();
        let decoded = decode_instruction_stream(&[
            add[0],
            add[1],
            add[2],
            add[3],
            0xff,
            Opcode::LoadConst as u8,
        ]);

        assert_eq!(
            decoded.instructions(),
            &[Instruction::abc(Opcode::Add, 1, 2, 3)]
        );
        assert_eq!(
            decoded.invalid_words(),
            &[InvalidInstructionWord::new(1, 0xff)]
        );
        assert_eq!(decoded.trailing_byte_count(), 1);
    }

    #[test]
    fn decoded_stream_can_be_disassembled_without_panicking() {
        let bytes = [
            Opcode::LoadConst as u8,
            0,
            0,
            0,
            Opcode::Call as u8,
            1,
            2,
            3,
            Opcode::Return as u8,
            1,
            0,
            0,
            0xff,
            0,
            0,
            0,
        ];
        let decoded = decode_instruction_stream(&bytes);
        let function = BytecodeFunction::new(
            BytecodeFunctionId::from_raw(1).expect("non-zero bytecode id"),
            None,
            ArgumentsMode::None,
        )
        .with_instructions(decoded.instructions().to_vec())
        .with_constants(vec![ConstantValue::Smi(7)])
        .with_wide_operands(vec![WideOperand::new(1, 0x0002_0003)]);

        let _ = disassemble(&function);
        for instruction in function.instructions() {
            let _ = disassemble_instruction(&instruction, &function);
        }
    }

    #[test]
    fn specialized_smi_opcodes_decode_and_disassemble_immediates() {
        let function = BytecodeFunction::new(
            BytecodeFunctionId::from_raw(2).expect("non-zero bytecode id"),
            None,
            ArgumentsMode::None,
        )
        .with_instructions(vec![
            Instruction::abx(Opcode::LoadZero, 0, 0),
            Instruction::abx(Opcode::LoadOne, 1, 0),
            Instruction::abc(Opcode::AddSmi, 2, 1, 13),
            Instruction::abc(Opcode::SubSmi, 3, 2, 5),
            Instruction::abc(Opcode::MulSmi, 4, 3, 7),
            Instruction::abc(Opcode::DivSmi, 5, 4, 2),
            Instruction::abc(Opcode::ModSmi, 6, 5, 3),
            Instruction::abc(Opcode::BitAndSmi, 7, 6, 1),
            Instruction::abc(Opcode::EqualZero, 8, 7, 0),
        ]);

        assert_eq!(
            decode_instruction_word(
                function
                    .instructions()
                    .get(2)
                    .expect("third instruction should decode")
                    .encode_word()
            ),
            Ok(Instruction::abc(Opcode::AddSmi, 2, 1, 13))
        );

        let text = disassemble(&function);
        assert!(text.contains("LoadZero"));
        assert!(text.contains("LoadOne"));
        assert!(text.contains("AddSmi"));
        assert!(text.contains("r2, r1, 13"));
        assert!(text.contains("SubSmi"));
        assert!(text.contains("r3, r2, 5"));
        assert!(text.contains("MulSmi"));
        assert!(text.contains("r4, r3, 7"));
        assert!(text.contains("DivSmi"));
        assert!(text.contains("r5, r4, 2"));
        assert!(text.contains("ModSmi"));
        assert!(text.contains("r6, r5, 3"));
        assert!(text.contains("BitAndSmi"));
        assert!(text.contains("r7, r6, 1"));
        assert!(text.contains("EqualZero"));
        assert!(text.contains("r8, r7"));
    }
}
