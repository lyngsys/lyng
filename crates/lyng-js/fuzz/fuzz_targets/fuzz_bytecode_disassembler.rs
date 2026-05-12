#![no_main]

use lyng_js_bytecode::{
    decode_instruction_stream, disassemble, disassemble_instruction, ArgumentsMode,
    BytecodeFunction, BytecodeFunctionId, ConstantValue, WideOperand,
};
use lyng_js_common::AtomId;
use libfuzzer_sys::fuzz_target;

const MAX_INPUT_LEN: usize = 4096;
const MAX_CONSTANTS: usize = 64;
const MAX_WIDE_OPERANDS: usize = 64;

fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_INPUT_LEN {
        return;
    }

    let decoded = decode_instruction_stream(data);
    let instructions = decoded.instructions().to_vec();
    let constants = decode_constants(data);
    let wide_operands = decode_wide_operands(data, instructions.len());

    let arguments_mode = match data.first().copied().unwrap_or(0) % 3 {
        0 => ArgumentsMode::None,
        1 => ArgumentsMode::Unmapped,
        _ => ArgumentsMode::Mapped,
    };

    let function = BytecodeFunction::new(
        BytecodeFunctionId::from_raw(1).expect("non-zero bytecode id"),
        Some(AtomId::from_raw(u32::from(data.first().copied().unwrap_or(0)))),
        arguments_mode,
    )
    .with_instructions(instructions)
    .with_constants(constants)
    .with_wide_operands(wide_operands);

    let _ = decoded.invalid_words();
    let _ = decoded.trailing_byte_count();
    let _ = disassemble(&function);
    for instruction in function.instructions() {
        let _ = disassemble_instruction(&instruction, &function);
    }
});

fn decode_constants(bytes: &[u8]) -> Vec<ConstantValue> {
    bytes.chunks(9)
        .take(MAX_CONSTANTS)
        .map(|chunk| {
            let tag = chunk.first().copied().unwrap_or(0) % 3;
            match tag {
                0 => ConstantValue::Smi(i32::from(read_u16(chunk, 1) as i16)),
                1 => ConstantValue::Float64Bits(read_u64(chunk, 1)),
                _ => ConstantValue::Atom(AtomId::from_raw(read_u32(chunk, 1))),
            }
        })
        .collect()
}

fn decode_wide_operands(bytes: &[u8], instruction_count: usize) -> Vec<WideOperand> {
    if instruction_count == 0 {
        return Vec::new();
    }

    bytes.chunks(8)
        .take(MAX_WIDE_OPERANDS)
        .map(|chunk| {
            let offset = usize::try_from(read_u32(chunk, 0)).unwrap_or(usize::MAX) % instruction_count;
            let payload = read_u32(chunk, 4);
            WideOperand::new(
                u32::try_from(offset).expect("fuzz instruction offset should fit u32"),
                payload,
            )
        })
        .collect()
}

fn read_u16(chunk: &[u8], start: usize) -> u16 {
    let low = chunk.get(start).copied().unwrap_or(0);
    let high = chunk.get(start + 1).copied().unwrap_or(0);
    u16::from_le_bytes([low, high])
}

fn read_u32(chunk: &[u8], start: usize) -> u32 {
    let bytes = [
        chunk.get(start).copied().unwrap_or(0),
        chunk.get(start + 1).copied().unwrap_or(0),
        chunk.get(start + 2).copied().unwrap_or(0),
        chunk.get(start + 3).copied().unwrap_or(0),
    ];
    u32::from_le_bytes(bytes)
}

fn read_u64(chunk: &[u8], start: usize) -> u64 {
    let bytes = [
        chunk.get(start).copied().unwrap_or(0),
        chunk.get(start + 1).copied().unwrap_or(0),
        chunk.get(start + 2).copied().unwrap_or(0),
        chunk.get(start + 3).copied().unwrap_or(0),
        chunk.get(start + 4).copied().unwrap_or(0),
        chunk.get(start + 5).copied().unwrap_or(0),
        chunk.get(start + 6).copied().unwrap_or(0),
        chunk.get(start + 7).copied().unwrap_or(0),
    ];
    u64::from_le_bytes(bytes)
}
