use crate::number_to_string;
use lyng_js_env::Agent;
use lyng_js_gc::AllocationLifetime;
use lyng_js_objects::TypedArrayElementKind;
use lyng_js_types::{Completion, ObjectRef, PropertyDescriptor, PropertyKey, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TypedArrayNumericKey {
    Valid(u32),
    Invalid,
}

fn typed_array_biguint64_value(agent: &mut Agent, bits: u64) -> Value {
    let bigint = agent.heap_mut().mutator().alloc_bigint(
        lyng_js_gc::BigIntSign::NonNegative,
        &[bits],
        AllocationLifetime::Default,
    );
    Value::from_bigint_ref(bigint)
}

fn typed_array_bigint64_value(agent: &mut Agent, bits: u64) -> Value {
    let (sign, limbs) = if bits >> 63 == 0 {
        (lyng_js_gc::BigIntSign::NonNegative, [bits])
    } else {
        (lyng_js_gc::BigIntSign::Negative, [bits.wrapping_neg()])
    };
    let bigint = agent
        .heap_mut()
        .mutator()
        .alloc_bigint(sign, &limbs, AllocationLifetime::Default);
    Value::from_bigint_ref(bigint)
}

fn typed_array_storage_bits_to_value(
    agent: &mut Agent,
    kind: TypedArrayElementKind,
    bits: u64,
) -> Value {
    match kind {
        TypedArrayElementKind::BigInt64 => typed_array_bigint64_value(agent, bits),
        TypedArrayElementKind::BigUint64 => typed_array_biguint64_value(agent, bits),
        TypedArrayElementKind::Int8 => Value::from_smi(i32::from((bits as u8) as i8)),
        TypedArrayElementKind::Int16 => Value::from_smi(i32::from((bits as u16) as i16)),
        TypedArrayElementKind::Int32 => Value::from_smi(bits as u32 as i32),
        TypedArrayElementKind::Float32 => Value::from_f64(f64::from(f32::from_bits(bits as u32))),
        TypedArrayElementKind::Float64 => Value::from_f64(f64::from_bits(bits)),
        TypedArrayElementKind::Uint32 => {
            let value = bits as u32;
            i32::try_from(value)
                .map(Value::from_smi)
                .unwrap_or_else(|_| Value::from_f64(f64::from(value)))
        }
        TypedArrayElementKind::Uint16 => Value::from_smi(i32::from(bits as u16)),
        TypedArrayElementKind::Uint8Clamped | TypedArrayElementKind::Uint8 => {
            Value::from_smi(i32::from(bits as u8))
        }
    }
}

fn typed_array_read_storage_bits(
    agent: &Agent,
    object: ObjectRef,
    index: u32,
) -> Completion<Option<(TypedArrayElementKind, u64)>> {
    let Some(record) = agent.objects().typed_array(object) else {
        return Ok(None);
    };
    let index = usize::try_from(index).unwrap_or(usize::MAX);
    if !typed_array_index_is_valid(agent, record, index) {
        return Ok(None);
    }
    let element_size = record.kind().bytes_per_element();
    let Some(start) = index
        .checked_mul(element_size)
        .and_then(|relative| record.byte_offset().checked_add(relative))
    else {
        return Ok(None);
    };
    let mut bits = 0_u64;
    for offset in 0..element_size {
        let Some(byte_index) = start.checked_add(offset) else {
            return Ok(None);
        };
        let Some(byte) = agent.backing_store_get_byte(record.backing_store(), byte_index) else {
            return Ok(None);
        };
        bits |= u64::from(byte) << (offset * 8);
    }
    Ok(Some((record.kind(), bits)))
}

fn typed_array_index_is_valid(
    agent: &Agent,
    record: lyng_js_objects::TypedArrayObjectData,
    index: usize,
) -> bool {
    if index >= record.length() {
        return false;
    }
    if agent
        .backing_store_is_detached(record.backing_store())
        .unwrap_or(false)
    {
        return false;
    }
    if typed_array_is_out_of_bounds(agent, record) {
        return false;
    }
    true
}

fn typed_array_is_out_of_bounds(
    agent: &Agent,
    record: lyng_js_objects::TypedArrayObjectData,
) -> bool {
    let Some(byte_length) = agent.backing_store_byte_length(record.backing_store()) else {
        return true;
    };
    if record.is_length_tracking() {
        return record.byte_offset() > byte_length;
    }
    record.byte_offset().saturating_add(record.byte_length()) > byte_length
}

fn canonical_numeric_index_string(text: &str) -> Option<f64> {
    if text == "-0" {
        return Some(-0.0);
    }
    let number = match text {
        "NaN" => f64::NAN,
        "Infinity" => f64::INFINITY,
        "-Infinity" => f64::NEG_INFINITY,
        _ => text.parse::<f64>().ok()?,
    };
    (number_to_string(number) == text).then_some(number)
}

fn valid_numeric_index(agent: &Agent, object: ObjectRef, index: f64) -> Option<u32> {
    if !index.is_finite()
        || index.fract() != 0.0
        || index < 0.0
        || (index == 0.0 && index.is_sign_negative())
        || index > f64::from(u32::MAX)
    {
        return None;
    }
    let record = agent.objects().typed_array(object)?;
    let index = index as usize;
    if typed_array_index_is_valid(agent, record, index) {
        u32::try_from(index).ok()
    } else {
        None
    }
}

pub(super) fn typed_array_numeric_key(
    agent: &Agent,
    object: ObjectRef,
    key: PropertyKey,
) -> Option<TypedArrayNumericKey> {
    agent.objects().typed_array(object)?;
    let numeric_index = if let Some(index) = key.as_index() {
        f64::from(index)
    } else {
        let atom = key.as_atom()?;
        canonical_numeric_index_string(agent.atoms().resolve(atom))?
    };
    Some(
        valid_numeric_index(agent, object, numeric_index)
            .map_or(TypedArrayNumericKey::Invalid, TypedArrayNumericKey::Valid),
    )
}

pub(crate) fn is_typed_array_numeric_key(
    agent: &Agent,
    object: ObjectRef,
    key: PropertyKey,
) -> bool {
    typed_array_numeric_key(agent, object, key).is_some()
}

pub(super) fn typed_array_index_descriptor(
    agent: &mut Agent,
    object: ObjectRef,
    index: u32,
) -> Completion<Option<PropertyDescriptor>> {
    let Some((kind, bits)) = typed_array_read_storage_bits(agent, object, index)? else {
        return Ok(None);
    };
    let mut descriptor = PropertyDescriptor::new();
    descriptor.set_value(typed_array_storage_bits_to_value(agent, kind, bits));
    descriptor.set_writable(true);
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    Ok(Some(descriptor))
}
