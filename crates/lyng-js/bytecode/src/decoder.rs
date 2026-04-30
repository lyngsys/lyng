use crate::{Instruction, Opcode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InstructionForm {
    Abc,
    Abx,
    Ax,
}

/// Decoder error for one malformed 4-byte instruction word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    InvalidOpcode(u8),
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

/// Decode one 4-byte instruction word without panicking on malformed opcode bytes.
///
/// # Errors
/// Returns [`DecodeError::InvalidOpcode`] when the opcode byte does not map to a known
/// instruction.
pub fn decode_instruction_word(word: u32) -> Result<Instruction, DecodeError> {
    let [raw_opcode, first, second, third] = word.to_le_bytes();
    let opcode = Opcode::from_byte(raw_opcode).ok_or(DecodeError::InvalidOpcode(raw_opcode))?;

    Ok(match instruction_form(opcode) {
        InstructionForm::Abc => Instruction::abc(opcode, first, second, third),
        InstructionForm::Abx => {
            Instruction::abx(opcode, first, u16::from_le_bytes([second, third]))
        }
        InstructionForm::Ax => Instruction::ax(opcode, sign_extend_i24([first, second, third])),
    })
}

/// Decode as many full 4-byte instruction words as possible from an opaque byte buffer.
///
/// Invalid opcode bytes are recorded and skipped. Trailing bytes that do not form a full
/// instruction word are also reported.
///
/// # Panics
/// Panics if the decoded word index does not fit in `u32`.
pub fn decode_instruction_stream(bytes: &[u8]) -> DecodedInstructionStream {
    let mut instructions = Vec::with_capacity(bytes.len() / 4);
    let mut invalid_words = Vec::new();
    let mut chunks = bytes.chunks_exact(4);

    for (index, chunk) in chunks.by_ref().enumerate() {
        let word = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        match decode_instruction_word(word) {
            Ok(instruction) => instructions.push(instruction),
            Err(DecodeError::InvalidOpcode(raw_opcode)) => {
                invalid_words.push(InvalidInstructionWord::new(
                    u32::try_from(index).expect("decoded instruction index should fit u32"),
                    raw_opcode,
                ));
            }
        }
    }

    DecodedInstructionStream {
        instructions,
        invalid_words,
        trailing_byte_count: chunks.remainder().len(),
    }
}

fn instruction_form(opcode: Opcode) -> InstructionForm {
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
        | Opcode::DeleteName
        | Opcode::CaptureName
        | Opcode::LoadCapturedName
        | Opcode::LoadCapturedNameThis
        | Opcode::AssignCapturedName
        | Opcode::LoadThis
        | Opcode::LoadCallee
        | Opcode::LoadNewTarget
        | Opcode::JumpIfTrue
        | Opcode::JumpIfFalse
        | Opcode::CreateObject
        | Opcode::CreateArray
        | Opcode::CheckObjectCoercible
        | Opcode::ThrowIfUninitialized
        | Opcode::CreateClosure
        | Opcode::CloseForIn
        | Opcode::CloseIterator => InstructionForm::Abx,
        Opcode::Move
        | Opcode::Add
        | Opcode::Sub
        | Opcode::Mul
        | Opcode::Div
        | Opcode::Mod
        | Opcode::Exp
        | Opcode::BitOr
        | Opcode::BitXor
        | Opcode::BitAnd
        | Opcode::BitNot
        | Opcode::ShiftLeft
        | Opcode::ShiftRight
        | Opcode::UnsignedShiftRight
        | Opcode::Negate
        | Opcode::Increment
        | Opcode::Decrement
        | Opcode::Equal
        | Opcode::StrictEqual
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
        | Opcode::GetKeyedProperty
        | Opcode::SetKeyedProperty
        | Opcode::AssignKeyedProperty
        | Opcode::DeleteProperty
        | Opcode::CopyDataProperties
        | Opcode::SetFunctionName
        | Opcode::ToPropertyKey
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

fn sign_extend_i24(bytes: [u8; 3]) -> i32 {
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
        let invalid = [0xff, 9, 8, 7];
        let decoded = decode_instruction_stream(&[
            add[0], add[1], add[2], add[3], invalid[0], invalid[1], invalid[2], invalid[3], 0xaa,
            0xbb,
        ]);

        assert_eq!(
            decoded.instructions(),
            &[Instruction::abc(Opcode::Add, 1, 2, 3)]
        );
        assert_eq!(
            decoded.invalid_words(),
            &[InvalidInstructionWord::new(1, 0xff)]
        );
        assert_eq!(decoded.trailing_byte_count(), 2);
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
            let _ = disassemble_instruction(instruction, &function);
        }
    }
}
