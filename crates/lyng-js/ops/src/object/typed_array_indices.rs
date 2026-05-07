use crate::{errors::internal_method_error, number_to_string};
use lyng_js_env::Agent;
use lyng_js_gc::AllocationLifetime;
use lyng_js_objects::{float16_bits_to_f64, TypedArrayElementKind};
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

fn low_u8(bits: u64) -> u8 {
    u8::try_from(bits & u64::from(u8::MAX)).expect("masked typed-array byte should fit u8")
}

fn low_u16(bits: u64) -> u16 {
    u16::try_from(bits & u64::from(u16::MAX)).expect("masked typed-array bits should fit u16")
}

fn low_u32(bits: u64) -> u32 {
    u32::try_from(bits & u64::from(u32::MAX)).expect("masked typed-array bits should fit u32")
}

fn typed_array_storage_bits_to_value(
    agent: &mut Agent,
    kind: TypedArrayElementKind,
    bits: u64,
) -> Value {
    match kind {
        TypedArrayElementKind::BigInt64 => typed_array_bigint64_value(agent, bits),
        TypedArrayElementKind::BigUint64 => typed_array_biguint64_value(agent, bits),
        TypedArrayElementKind::Int8 => Value::from_smi(i32::from(low_u8(bits).cast_signed())),
        TypedArrayElementKind::Int16 => Value::from_smi(i32::from(low_u16(bits).cast_signed())),
        TypedArrayElementKind::Int32 => Value::from_smi(low_u32(bits).cast_signed()),
        TypedArrayElementKind::Float16 => Value::from_f64(float16_bits_to_f64(low_u16(bits))),
        TypedArrayElementKind::Float32 => Value::from_f64(f64::from(f32::from_bits(low_u32(bits)))),
        TypedArrayElementKind::Float64 => Value::from_f64(f64::from_bits(bits)),
        TypedArrayElementKind::Uint32 => {
            let value = low_u32(bits);
            i32::try_from(value).map_or_else(|_| Value::from_f64(f64::from(value)), Value::from_smi)
        }
        TypedArrayElementKind::Uint16 => Value::from_smi(i32::from(low_u16(bits))),
        TypedArrayElementKind::Uint8Clamped | TypedArrayElementKind::Uint8 => {
            Value::from_smi(i32::from(low_u8(bits)))
        }
    }
}

fn typed_array_read_storage_bits(
    agent: &Agent,
    object: ObjectRef,
    index: u32,
) -> Option<(TypedArrayElementKind, u64)> {
    let record = agent.objects().typed_array(object)?;
    let index = usize::try_from(index).unwrap_or(usize::MAX);
    if !typed_array_index_is_valid(agent, record, index) {
        return None;
    }
    let element_size = record.kind().bytes_per_element();
    let start = index
        .checked_mul(element_size)
        .and_then(|relative| record.byte_offset().checked_add(relative))?;
    let mut bits = 0_u64;
    for offset in 0..element_size {
        let byte_index = start.checked_add(offset)?;
        let byte = agent.backing_store_get_byte(record.backing_store(), byte_index)?;
        bits |= u64::from(byte) << (offset * 8);
    }
    Some((record.kind(), bits))
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

fn typed_array_integer_index_length(
    agent: &Agent,
    record: lyng_js_objects::TypedArrayObjectData,
) -> usize {
    if agent
        .backing_store_is_detached(record.backing_store())
        .unwrap_or(true)
    {
        return 0;
    }
    let Some(byte_length) = agent.backing_store_byte_length(record.backing_store()) else {
        return 0;
    };
    if record.is_length_tracking() {
        return byte_length
            .checked_sub(record.byte_offset())
            .map_or(0, |remaining| remaining / record.kind().bytes_per_element());
    }
    if record
        .byte_offset()
        .checked_add(record.byte_length())
        .is_none_or(|end| end > byte_length)
    {
        return 0;
    }
    record.length()
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

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "typed-array numeric indices are finite, integral, non-negative, and u32 range checked"
)]
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

pub fn is_typed_array_numeric_key(agent: &Agent, object: ObjectRef, key: PropertyKey) -> bool {
    typed_array_numeric_key(agent, object, key).is_some()
}

pub(super) fn typed_array_index_descriptor(
    agent: &mut Agent,
    object: ObjectRef,
    index: u32,
) -> Option<PropertyDescriptor> {
    let (kind, bits) = typed_array_read_storage_bits(agent, object, index)?;
    let mut descriptor = PropertyDescriptor::new();
    descriptor.set_value(typed_array_storage_bits_to_value(agent, kind, bits));
    descriptor.set_writable(true);
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    Some(descriptor)
}

pub(super) fn typed_array_own_property_keys(
    agent: &mut Agent,
    object: ObjectRef,
) -> Completion<Option<Vec<PropertyKey>>> {
    let Some(record) = agent.objects().typed_array(object) else {
        return Ok(None);
    };
    let length = typed_array_integer_index_length(agent, record);
    let mut keys = (0..u32::try_from(length).unwrap_or(u32::MAX))
        .map(PropertyKey::Index)
        .collect::<Vec<_>>();
    let own_keys = match agent
        .objects()
        .own_property_keys(agent.heap().view(), object)
    {
        Ok(keys) => keys,
        Err(error) => return Err(internal_method_error(agent, error)),
    };
    keys.extend(own_keys.into_iter().filter(|key| key.as_index().is_none()));
    Ok(Some(keys))
}
