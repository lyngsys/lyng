use lyng_js_env::Agent;
use lyng_js_gc::{AllocationLifetime, BigIntSign};
use lyng_js_host::WaitLocation;
use lyng_js_objects::{float16_bits_to_f64, TypedArrayElementKind, TypedArrayObjectData};
use lyng_js_types::{ObjectRef, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtomicAccessError {
    Type,
    Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AtomicAccessRecord {
    viewed_array_buffer: ObjectRef,
    typed_array: TypedArrayObjectData,
    element_index: usize,
}

impl AtomicAccessRecord {
    #[inline]
    pub const fn viewed_array_buffer(self) -> ObjectRef {
        self.viewed_array_buffer
    }

    #[inline]
    pub const fn typed_array(self) -> TypedArrayObjectData {
        self.typed_array
    }

    #[inline]
    pub const fn element_index(self) -> usize {
        self.element_index
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtomicRmwOp {
    Add,
    Sub,
    And,
    Or,
    Xor,
    Exchange,
}

#[inline]
pub const fn is_atomics_friendly_kind(kind: TypedArrayElementKind) -> bool {
    !matches!(
        kind,
        TypedArrayElementKind::Float32
            | TypedArrayElementKind::Float16
            | TypedArrayElementKind::Float64
            | TypedArrayElementKind::Uint8Clamped
    )
}

#[inline]
pub const fn is_waitable_kind(kind: TypedArrayElementKind) -> bool {
    matches!(
        kind,
        TypedArrayElementKind::Int32 | TypedArrayElementKind::BigInt64
    )
}

/// Validates that an object is usable for Atomics operations.
///
/// # Errors
/// Returns [`AtomicAccessError`] when the object is not a compatible typed array, is detached, or
/// does not satisfy the requested waitable/shared constraints.
pub fn validate_atomic_typed_array(
    agent: &Agent,
    typed_array_object: ObjectRef,
    waitable: bool,
    require_shared: bool,
) -> Result<TypedArrayObjectData, AtomicAccessError> {
    let typed_array = agent
        .objects()
        .typed_array(typed_array_object)
        .ok_or(AtomicAccessError::Type)?;
    if !is_atomics_friendly_kind(typed_array.kind()) {
        return Err(AtomicAccessError::Type);
    }
    if waitable && !is_waitable_kind(typed_array.kind()) {
        return Err(AtomicAccessError::Type);
    }
    if agent
        .backing_store_is_detached(typed_array.backing_store())
        .unwrap_or(false)
    {
        return Err(AtomicAccessError::Type);
    }
    if require_shared
        && !agent
            .backing_store_is_shared(typed_array.backing_store())
            .unwrap_or(false)
    {
        return Err(AtomicAccessError::Type);
    }
    Ok(typed_array)
}

/// Validates an Atomics element index against a typed-array length.
///
/// # Errors
/// Returns [`AtomicAccessError::Range`] when the element index is out of range.
pub fn validate_atomic_index(
    typed_array: TypedArrayObjectData,
    element_index: u64,
) -> Result<usize, AtomicAccessError> {
    let element_index = usize::try_from(element_index).map_err(|_| AtomicAccessError::Range)?;
    if element_index >= typed_array.length() {
        return Err(AtomicAccessError::Range);
    }
    Ok(element_index)
}

#[inline]
pub const fn atomic_access_record(
    typed_array: TypedArrayObjectData,
    element_index: usize,
) -> AtomicAccessRecord {
    AtomicAccessRecord {
        viewed_array_buffer: typed_array.viewed_array_buffer(),
        typed_array,
        element_index,
    }
}

/// Validates a typed array and element index for one Atomics access.
///
/// # Errors
/// Returns [`AtomicAccessError`] when typed-array validation or index validation fails.
pub fn validate_atomic_access(
    agent: &Agent,
    typed_array_object: ObjectRef,
    element_index: u64,
    waitable: bool,
    require_shared: bool,
) -> Result<AtomicAccessRecord, AtomicAccessError> {
    let typed_array =
        validate_atomic_typed_array(agent, typed_array_object, waitable, require_shared)?;
    let element_index = validate_atomic_index(typed_array, element_index)?;
    Ok(AtomicAccessRecord {
        viewed_array_buffer: typed_array.viewed_array_buffer(),
        typed_array,
        element_index,
    })
}

#[inline]
pub fn wait_location(record: AtomicAccessRecord) -> WaitLocation {
    let element_size = record.typed_array.kind().bytes_per_element();
    let byte_offset = record
        .typed_array
        .byte_offset()
        .saturating_add(record.element_index.saturating_mul(element_size));
    WaitLocation::new(
        record.typed_array.backing_store(),
        u64::try_from(byte_offset).unwrap_or(u64::MAX),
    )
}

#[inline]
pub fn read_atomic_bits(agent: &Agent, record: AtomicAccessRecord) -> Option<u64> {
    read_typed_array_bits(agent, record.typed_array, record.element_index)
}

#[inline]
pub fn atomic_store_bits(agent: &mut Agent, record: AtomicAccessRecord, bits: u64) -> Option<u64> {
    let normalized = normalize_integer_bits(record.typed_array.kind(), bits);
    write_typed_array_bits(agent, record.typed_array, record.element_index, normalized)
        .then_some(normalized)
}

pub fn atomic_compare_exchange_bits(
    agent: &mut Agent,
    record: AtomicAccessRecord,
    expected: u64,
    replacement: u64,
) -> Option<u64> {
    let element_size = record.typed_array.kind().bytes_per_element();
    let start = typed_array_start(record.typed_array, record.element_index)?;
    agent.backing_store_atomic_compare_exchange_bits(
        record.typed_array.backing_store(),
        start,
        element_size,
        normalize_integer_bits(record.typed_array.kind(), expected),
        normalize_integer_bits(record.typed_array.kind(), replacement),
    )
}

pub fn atomic_rmw_bits(
    agent: &mut Agent,
    record: AtomicAccessRecord,
    value: u64,
    op: AtomicRmwOp,
) -> Option<u64> {
    let value = normalize_integer_bits(record.typed_array.kind(), value);
    loop {
        let current = read_atomic_bits(agent, record)?;
        let next = match op {
            AtomicRmwOp::Add => current.wrapping_add(value),
            AtomicRmwOp::Sub => current.wrapping_sub(value),
            AtomicRmwOp::And => current & value,
            AtomicRmwOp::Or => current | value,
            AtomicRmwOp::Xor => current ^ value,
            AtomicRmwOp::Exchange => value,
        };
        let next = normalize_integer_bits(record.typed_array.kind(), next);
        let observed = atomic_compare_exchange_bits(agent, record, current, next)?;
        if observed == current {
            return Some(current);
        }
    }
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

pub fn atomic_value_from_bits(agent: &mut Agent, kind: TypedArrayElementKind, bits: u64) -> Value {
    match kind {
        TypedArrayElementKind::BigInt64 => {
            let (sign, limbs) = if bits >> 63 == 0 {
                (BigIntSign::NonNegative, [bits])
            } else {
                (BigIntSign::Negative, [bits.wrapping_neg()])
            };
            let bigint =
                agent
                    .heap_mut()
                    .mutator()
                    .alloc_bigint(sign, &limbs, AllocationLifetime::Default);
            Value::from_bigint_ref(bigint)
        }
        TypedArrayElementKind::BigUint64 => {
            let bigint = agent.heap_mut().mutator().alloc_bigint(
                BigIntSign::NonNegative,
                &[bits],
                AllocationLifetime::Default,
            );
            Value::from_bigint_ref(bigint)
        }
        TypedArrayElementKind::Int8 => Value::from_smi(i32::from(low_u8(bits).cast_signed())),
        TypedArrayElementKind::Int16 => Value::from_smi(i32::from(low_u16(bits).cast_signed())),
        TypedArrayElementKind::Int32 => Value::from_smi(low_u32(bits).cast_signed()),
        TypedArrayElementKind::Uint32 => {
            let value = low_u32(bits);
            i32::try_from(value).map_or_else(|_| Value::from_f64(f64::from(value)), Value::from_smi)
        }
        TypedArrayElementKind::Uint16 => Value::from_smi(i32::from(low_u16(bits))),
        TypedArrayElementKind::Uint8 | TypedArrayElementKind::Uint8Clamped => {
            Value::from_smi(i32::from(low_u8(bits)))
        }
        TypedArrayElementKind::Float16 => Value::from_f64(float16_bits_to_f64(low_u16(bits))),
        TypedArrayElementKind::Float32 => Value::from_f64(f64::from(f32::from_bits(low_u32(bits)))),
        TypedArrayElementKind::Float64 => Value::from_f64(f64::from_bits(bits)),
    }
}

#[inline]
pub const fn atomics_is_lock_free(size: u64) -> bool {
    matches!(size, 1 | 2 | 4 | 8)
}

fn read_typed_array_bits(
    agent: &Agent,
    typed_array: TypedArrayObjectData,
    element_index: usize,
) -> Option<u64> {
    let element_size = typed_array.kind().bytes_per_element();
    let start = typed_array_start(typed_array, element_index)?;
    agent.backing_store_atomic_load_bits(typed_array.backing_store(), start, element_size)
}

fn write_typed_array_bits(
    agent: &mut Agent,
    typed_array: TypedArrayObjectData,
    element_index: usize,
    bits: u64,
) -> bool {
    let element_size = typed_array.kind().bytes_per_element();
    let Some(start) = typed_array_start(typed_array, element_index) else {
        return false;
    };
    agent.backing_store_atomic_store_bits(typed_array.backing_store(), start, element_size, bits)
}

fn typed_array_start(typed_array: TypedArrayObjectData, element_index: usize) -> Option<usize> {
    let element_size = typed_array.kind().bytes_per_element();
    typed_array
        .byte_offset()
        .checked_add(element_index.checked_mul(element_size)?)
}

fn normalize_integer_bits(kind: TypedArrayElementKind, bits: u64) -> u64 {
    match kind {
        TypedArrayElementKind::BigInt64
        | TypedArrayElementKind::BigUint64
        | TypedArrayElementKind::Float16
        | TypedArrayElementKind::Float32
        | TypedArrayElementKind::Float64 => bits,
        TypedArrayElementKind::Int8
        | TypedArrayElementKind::Uint8
        | TypedArrayElementKind::Uint8Clamped => u64::from(low_u8(bits)),
        TypedArrayElementKind::Int16 | TypedArrayElementKind::Uint16 => u64::from(low_u16(bits)),
        TypedArrayElementKind::Int32 | TypedArrayElementKind::Uint32 => u64::from(low_u32(bits)),
    }
}
