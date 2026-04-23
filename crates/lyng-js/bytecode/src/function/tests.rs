use super::{BytecodeFunction, BytecodeMarker, CompiledScriptUnit};
use crate::{ArgumentsMode, BytecodeFunctionId, ConstantValue, Instruction, Opcode};
use lyng_js_common::{AtomId, SourceId};
use lyng_js_types::FeedbackSlotId;
use std::num::NonZeroU32;

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

#[test]
fn bytecode_marker_round_trips_identity() {
    let marker = BytecodeMarker::new(
        SourceId::new(3),
        BytecodeFunctionId::from_raw(7).unwrap(),
        FeedbackSlotId::from_raw(11).unwrap(),
    );

    assert_eq!(marker.source(), SourceId::new(3));
    assert_eq!(marker.entry(), BytecodeFunctionId::from_raw(7).unwrap());
    assert_eq!(
        marker.feedback_slot(),
        FeedbackSlotId::from_raw(11).unwrap()
    );
}
