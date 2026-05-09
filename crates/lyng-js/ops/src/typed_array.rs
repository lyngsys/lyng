use crate::{
    number_to_string,
    object::{self, ToPrimitiveContext, ToPrimitiveHint},
    read,
};
use lyng_js_env::Agent;
use lyng_js_gc::{AllocationLifetime, BigIntSign};
use lyng_js_objects::{
    f64_to_float16_bits, float16_bits_to_f64, TypedArrayElementKind, TypedArrayObjectData,
};
use lyng_js_types::{ObjectRef, PropertyKey, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumericKey {
    Valid(usize),
    Invalid,
}

pub fn canonical_numeric_index_string(text: &str) -> Option<f64> {
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

pub fn numeric_atom_index(agent: &Agent, key: PropertyKey) -> Option<f64> {
    canonical_numeric_index_string(agent.atoms().resolve(key.as_atom()?))
}

pub fn numeric_property_index(agent: &Agent, key: PropertyKey) -> Option<f64> {
    key.as_index()
        .map(f64::from)
        .or_else(|| numeric_atom_index(agent, key))
}

pub fn numeric_key(agent: &Agent, object: ObjectRef, key: PropertyKey) -> Option<NumericKey> {
    let record = agent.objects().typed_array(object)?;
    let numeric_index = numeric_property_index(agent, key)?;
    Some(
        valid_numeric_index(agent, record, numeric_index)
            .map_or(NumericKey::Invalid, NumericKey::Valid),
    )
}

pub fn is_numeric_key(agent: &Agent, object: ObjectRef, key: PropertyKey) -> bool {
    numeric_key(agent, object, key).is_some()
}

#[allow(
    clippy::cast_precision_loss,
    reason = "upper-bound guard only rejects values beyond the host pointer width"
)]
const fn max_usize_as_f64() -> f64 {
    usize::MAX as f64
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "typed-array numeric indices are finite, integral, non-negative, and usize range checked"
)]
pub fn element_index_from_numeric_index(index: f64) -> Option<usize> {
    if !index.is_finite()
        || index.fract() != 0.0
        || index < 0.0
        || (index == 0.0 && index.is_sign_negative())
        || index > max_usize_as_f64()
    {
        return None;
    }
    Some(index as usize)
}

pub fn valid_numeric_index(
    agent: &Agent,
    record: TypedArrayObjectData,
    index: f64,
) -> Option<usize> {
    let index = element_index_from_numeric_index(index)?;
    is_valid_integer_index(agent, record, index).then_some(index)
}

pub fn current_length(agent: &Agent, record: TypedArrayObjectData) -> Option<usize> {
    if agent.backing_store_is_detached(record.backing_store())? {
        return None;
    }
    let byte_length = agent.backing_store_byte_length(record.backing_store())?;
    if record.is_length_tracking() {
        return byte_length
            .checked_sub(record.byte_offset())
            .map(|remaining| remaining / record.kind().bytes_per_element());
    }
    if record
        .byte_offset()
        .checked_add(record.byte_length())
        .is_none_or(|end| end > byte_length)
    {
        return None;
    }
    Some(record.length())
}

pub fn is_out_of_bounds(agent: &Agent, record: TypedArrayObjectData) -> bool {
    current_length(agent, record).is_none()
}

pub fn is_valid_integer_index(
    agent: &Agent,
    record: TypedArrayObjectData,
    element_index: usize,
) -> bool {
    valid_integer_index_byte_start(agent, record, element_index).is_some()
}

pub fn valid_integer_index_byte_start(
    agent: &Agent,
    record: TypedArrayObjectData,
    element_index: usize,
) -> Option<usize> {
    let element_size = record.kind().bytes_per_element();
    let relative_start = element_index.checked_mul(element_size)?;
    let start = record.byte_offset().checked_add(relative_start)?;
    let element_end = start.checked_add(element_size)?;
    let byte_length = agent.backing_store_byte_length(record.backing_store())?;
    if record.is_length_tracking() {
        if record.byte_offset() > byte_length || element_end > byte_length {
            return None;
        }
        return Some(start);
    }
    if element_index >= record.length() {
        return None;
    }
    let view_end = record.byte_offset().checked_add(record.byte_length())?;
    if view_end > byte_length || element_end > byte_length {
        return None;
    }
    Some(start)
}

pub fn read_storage_bits(
    agent: &Agent,
    record: TypedArrayObjectData,
    element_index: usize,
) -> Option<u64> {
    let element_size = record.kind().bytes_per_element();
    let start = valid_integer_index_byte_start(agent, record, element_index)?;
    agent.backing_store_load_bits(record.backing_store(), start, element_size)
}

pub fn write_storage_bits(
    agent: &mut Agent,
    record: TypedArrayObjectData,
    element_index: usize,
    bits: u64,
) -> bool {
    let element_size = record.kind().bytes_per_element();
    let Some(start) = valid_integer_index_byte_start(agent, record, element_index) else {
        return false;
    };
    agent.backing_store_store_bits(record.backing_store(), start, element_size, bits)
}

pub fn read_element_value(
    agent: &mut Agent,
    record: TypedArrayObjectData,
    element_index: usize,
) -> Value {
    read_storage_bits(agent, record, element_index).map_or(Value::undefined(), |bits| {
        value_from_storage_bits(agent, record.kind(), bits)
    })
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "typed-array storage masks bits before narrowing to u8"
)]
pub const fn storage_u8_bits(bits: u64) -> u8 {
    (bits & u8::MAX as u64) as u8
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "typed-array storage masks bits before narrowing to u16"
)]
pub const fn storage_u16_bits(bits: u64) -> u16 {
    (bits & u16::MAX as u64) as u16
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "typed-array storage masks bits before narrowing to u32"
)]
pub const fn storage_u32_bits(bits: u64) -> u32 {
    (bits & u32::MAX as u64) as u32
}

pub fn value_from_storage_bits(agent: &mut Agent, kind: TypedArrayElementKind, bits: u64) -> Value {
    match kind {
        TypedArrayElementKind::BigInt64 => bigint64_value_from_bits(agent, bits),
        TypedArrayElementKind::BigUint64 => biguint64_value_from_bits(agent, bits),
        TypedArrayElementKind::Int8 => {
            Value::from_smi(i32::from(storage_u8_bits(bits).cast_signed()))
        }
        TypedArrayElementKind::Int16 => {
            Value::from_smi(i32::from(storage_u16_bits(bits).cast_signed()))
        }
        TypedArrayElementKind::Int32 => Value::from_smi(storage_u32_bits(bits).cast_signed()),
        TypedArrayElementKind::Float16 => {
            Value::from_f64(float16_bits_to_f64(storage_u16_bits(bits)))
        }
        TypedArrayElementKind::Float32 => {
            Value::from_f64(f64::from(f32::from_bits(storage_u32_bits(bits))))
        }
        TypedArrayElementKind::Float64 => Value::from_f64(f64::from_bits(bits)),
        TypedArrayElementKind::Uint32 => {
            let value = storage_u32_bits(bits);
            i32::try_from(value).map_or_else(|_| Value::from_f64(f64::from(value)), Value::from_smi)
        }
        TypedArrayElementKind::Uint16 => Value::from_smi(i32::from(storage_u16_bits(bits))),
        TypedArrayElementKind::Uint8Clamped | TypedArrayElementKind::Uint8 => {
            Value::from_smi(i32::from(storage_u8_bits(bits)))
        }
    }
}

/// Converts one ECMAScript value into typed-array storage bits for `kind`.
///
/// # Errors
/// Returns the caller-provided error when object-to-primitive conversion fails,
/// when a `BigInt` element receives a Number, when a Number element receives a
/// non-number-convertible primitive, or when `BigInt` conversion completes
/// abruptly.
pub fn storage_bits_from_value<Cx: ToPrimitiveContext>(
    cx: &mut Cx,
    kind: TypedArrayElementKind,
    value: Value,
) -> Result<u64, Cx::Error> {
    if matches!(
        kind,
        TypedArrayElementKind::BigInt64 | TypedArrayElementKind::BigUint64
    ) {
        let primitive = object::to_primitive(cx, value, ToPrimitiveHint::Number)?;
        if primitive.is_number() {
            return Err(cx.type_error());
        }
        let bigint = {
            let agent = cx.agent();
            object::primitive_to_bigint(agent, primitive)
        };
        let bigint = match bigint {
            Ok(bigint) => bigint,
            Err(completion) => return Err(cx.abrupt(completion)),
        };
        let bits = {
            let agent = cx.agent();
            bigint_to_uint64_bits(agent, bigint)
        };
        return bits.ok_or_else(|| cx.type_error());
    }

    let primitive = object::to_primitive(cx, value, ToPrimitiveHint::Number)?;
    let number = {
        let agent = cx.agent();
        read::to_number(agent.heap().view(), primitive)
    };
    let Ok(number) = number else {
        return Err(cx.type_error());
    };
    let Some(number) = number.as_f64() else {
        return Err(cx.type_error());
    };
    Ok(storage_bits_from_number(kind, number))
}

fn storage_bits_from_number(kind: TypedArrayElementKind, number: f64) -> u64 {
    match kind {
        TypedArrayElementKind::BigInt64 | TypedArrayElementKind::BigUint64 => {
            unreachable!("BigInt typed array elements require BigInt conversion")
        }
        TypedArrayElementKind::Int8 | TypedArrayElementKind::Uint8 => u64::from(to_uint8(number)),
        TypedArrayElementKind::Uint8Clamped => u64::from(to_uint8_clamp(number)),
        TypedArrayElementKind::Int16 | TypedArrayElementKind::Uint16 => {
            u64::from(to_uint16(number))
        }
        TypedArrayElementKind::Float16 => u64::from(f64_to_float16_bits(number)),
        TypedArrayElementKind::Float32 => u64::from(f32::to_bits(number_to_f32_storage(number))),
        TypedArrayElementKind::Float64 => number.to_bits(),
        TypedArrayElementKind::Int32 | TypedArrayElementKind::Uint32 => {
            u64::from(to_uint32(number))
        }
    }
}

pub fn bigint_to_uint64_bits(agent: &Agent, value: Value) -> Option<u64> {
    let bigint = value.as_bigint_ref()?;
    let view = agent.heap().view().bigint_view(bigint)?;
    let low = view.limb_at(0).unwrap_or(0);
    Some(match view.sign() {
        BigIntSign::NonNegative => low,
        BigIntSign::Negative => 0_u64.wrapping_sub(low),
    })
}

fn biguint64_value_from_bits(agent: &mut Agent, bits: u64) -> Value {
    let bigint = agent.heap_mut().mutator().alloc_bigint(
        BigIntSign::NonNegative,
        &[bits],
        AllocationLifetime::Default,
    );
    Value::from_bigint_ref(bigint)
}

fn bigint64_value_from_bits(agent: &mut Agent, bits: u64) -> Value {
    let (sign, limbs) = if bits >> 63 == 0 {
        (BigIntSign::NonNegative, [bits])
    } else {
        (BigIntSign::Negative, [bits.wrapping_neg()])
    };
    let bigint = agent
        .heap_mut()
        .mutator()
        .alloc_bigint(sign, &limbs, AllocationLifetime::Default);
    Value::from_bigint_ref(bigint)
}

fn to_uint8(number: f64) -> u8 {
    if number.is_nan() || number == 0.0 || !number.is_finite() {
        return 0;
    }
    let integer = number.trunc();
    let mut modulo = integer % 256.0;
    if modulo < 0.0 {
        modulo += 256.0;
    }
    number_to_u8_after_range_check(modulo)
}

fn to_uint8_clamp(number: f64) -> u8 {
    if number.is_nan() || number <= 0.0 {
        return 0;
    }
    if number >= 255.0 {
        return 255;
    }
    let floor = number.floor();
    if floor + 0.5 < number {
        return number_to_u8_after_range_check(floor).saturating_add(1);
    }
    if number < floor + 0.5 {
        return number_to_u8_after_range_check(floor);
    }
    let floor_u8 = number_to_u8_after_range_check(floor);
    if floor_u8 % 2 == 1 {
        floor_u8.saturating_add(1)
    } else {
        floor_u8
    }
}

fn to_uint16(number: f64) -> u16 {
    if number.is_nan() || number == 0.0 || !number.is_finite() {
        return 0;
    }
    let integer = number.trunc();
    let mut modulo = integer % 65_536.0;
    if modulo < 0.0 {
        modulo += 65_536.0;
    }
    number_to_u16_after_range_check(modulo)
}

fn to_uint32(number: f64) -> u32 {
    if number.is_nan() || number == 0.0 || !number.is_finite() {
        return 0;
    }
    let integer = number.trunc();
    let mut modulo = integer % 4_294_967_296.0;
    if modulo < 0.0 {
        modulo += 4_294_967_296.0;
    }
    number_to_u32_after_range_check(modulo)
}

const fn number_to_u8_after_range_check(number: f64) -> u8 {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "caller applies the ECMAScript modulo/range rules before narrowing to u8"
    )]
    let integer = number as u8;
    integer
}

const fn number_to_u16_after_range_check(number: f64) -> u16 {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "caller applies the ECMAScript modulo/range rules before narrowing to u16"
    )]
    let integer = number as u16;
    integer
}

const fn number_to_u32_after_range_check(number: f64) -> u32 {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "caller applies the ECMAScript modulo/range rules before narrowing to u32"
    )]
    let integer = number as u32;
    integer
}

const fn number_to_f32_storage(number: f64) -> f32 {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "Float32Array storage uses ECMA-262 NumberToRawBytes f64-to-f32 rounding"
    )]
    let narrowed = number as f32;
    narrowed
}
