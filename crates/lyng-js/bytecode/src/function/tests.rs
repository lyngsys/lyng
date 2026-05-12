use super::{BytecodeFunction, CompiledScriptUnit};
use crate::{ArgumentsMode, BytecodeFunctionId, ConstantValue, Instruction, Opcode};
use lyng_js_common::{AtomId, SourceId};
use std::num::NonZeroU32;

#[test]
fn bytecode_functions_store_encoded_instruction_bytes() {
    let function = BytecodeFunction::new(
        BytecodeFunctionId::from_raw(1).expect("non-zero bytecode id"),
        None,
        ArgumentsMode::None,
    )
    .with_instructions(vec![Instruction::ax(Opcode::Return, -2)]);

    assert_eq!(function.instruction_count(), 1);
    assert_eq!(
        function.instruction_bytes(),
        &[Opcode::Return as u8, 0xfe, 0xff, 0xff]
    );
}

#[test]
fn compiled_units_can_lookup_functions_by_id() {
    let entry = BytecodeFunctionId::new(NonZeroU32::new(1).unwrap());
    let function = BytecodeFunction::new(entry, Some(AtomId::from_raw(7)), ArgumentsMode::None)
        .with_instructions(vec![Instruction::ax(Opcode::Return, 0)])
        .with_constants(vec![ConstantValue::Smi(1)]);

    let unit = CompiledScriptUnit::new(SourceId::new(0), entry, vec![function.clone()]);

    assert_eq!(unit.entry(), entry);
    assert_eq!(unit.function(entry), Some(&function));
}
