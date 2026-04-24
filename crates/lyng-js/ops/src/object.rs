use crate::{
    convert::{
        bigint_parts_to_radix_string, bigint_view_to_radix_string, bigint_view_to_string,
        integral_number_to_bigint, lossy_string_from_view, number_to_string,
        parse_string_to_bigint,
    },
    errors::{internal_method_error, throw_range_error, throw_syntax_error, throw_type_error},
    read,
};
use lyng_js_common::WellKnownAtom;
use lyng_js_env::Agent;
use lyng_js_gc::{AllocationLifetime, StringEncoding};
use lyng_js_objects::{
    ClassPrivateElementKind, NativeFunctionRegistry, ObjectAllocation, ObjectColdData,
    OrdinaryObjectData, PrimitiveWrapperKind, TemporalObjectData, TemporalObjectKind,
    TypedArrayElementKind,
};
use lyng_js_types::{
    AbruptCompletion, Completion, ObjectRef, PropertyDescriptor, PropertyKey, RealmRef, ShapeId,
    Value, WellKnownSymbolId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToPrimitiveHint {
    Default,
    String,
    Number,
}

impl ToPrimitiveHint {
    #[inline]
    pub const fn hint_text(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::String => "string",
            Self::Number => "number",
        }
    }

    #[inline]
    pub const fn method_names(self) -> [lyng_js_common::AtomId; 2] {
        match self {
            Self::Default | Self::Number => {
                [WellKnownAtom::valueOf.id(), WellKnownAtom::toString.id()]
            }
            Self::String => [WellKnownAtom::toString.id(), WellKnownAtom::valueOf.id()],
        }
    }
}

pub trait ToPrimitiveContext {
    type Error;

    fn agent(&mut self) -> &mut Agent;

    fn abrupt(&mut self, completion: AbruptCompletion) -> Self::Error;

    fn type_error(&mut self) -> Self::Error;

    fn get_property_value(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
    ) -> Result<Value, Self::Error>;

    fn callable_object(&mut self, value: Value) -> Option<ObjectRef> {
        let object = value.as_object_ref()?;
        self.agent().objects().is_callable(object).then_some(object)
    }

    fn require_callable_object(&mut self, value: Value) -> Result<ObjectRef, Self::Error>;

    fn call_to_completion(
        &mut self,
        callee_object: ObjectRef,
        this_value: Value,
        arguments: &[Value],
    ) -> Result<Value, Self::Error>;

    fn default_to_primitive_result(
        &mut self,
        _object: ObjectRef,
        _method_name: lyng_js_common::AtomId,
        _method_object: ObjectRef,
    ) -> Result<Option<Value>, Self::Error> {
        Ok(None)
    }
}

/// ECMAScript `ToObject` over the shared wrapper substrate.
///
/// # Errors
/// Returns a `TypeError` abrupt completion when the input is nullish or the
/// current realm does not expose the required wrapper prototype.
pub fn to_object(agent: &mut Agent, realm: RealmRef, value: Value) -> Completion<ObjectRef> {
    if let Some(object) = value.as_object_ref() {
        return Ok(object);
    }
    if value.is_null() || value.is_undefined() {
        return Err(throw_type_error(agent));
    }

    wrap_primitive_value(agent, realm, value, AllocationLifetime::Default)
}

/// Allocates one primitive-wrapper object using the shared wrapper substrate.
///
/// # Errors
/// Returns a `TypeError` abrupt completion when wrapper allocation is not
/// available for the provided value in the current realm.
pub fn wrap_primitive_value(
    agent: &mut Agent,
    realm: RealmRef,
    value: Value,
    lifetime: AllocationLifetime,
) -> Completion<ObjectRef> {
    let Some(wrapper_kind) = primitive_wrapper_kind_for_value(value) else {
        return Err(throw_type_error(agent));
    };
    let realm_record = agent.realm(realm).ok_or_else(|| throw_type_error(agent))?;
    let root_shape = realm_record
        .root_shape()
        .ok_or_else(|| throw_type_error(agent))?;
    let prototype =
        wrapper_prototype(agent, realm, wrapper_kind).ok_or_else(|| throw_type_error(agent))?;
    allocate_primitive_wrapper_object(
        agent,
        root_shape,
        Some(prototype),
        wrapper_kind,
        value,
        lifetime,
    )
}

/// Allocates one primitive-wrapper object for a known wrapper family and prototype.
///
/// # Errors
/// Returns a `TypeError` abrupt completion when the provided primitive payload is invalid or when
/// the string-wrapper cache cannot be initialized.
pub fn allocate_primitive_wrapper_object(
    agent: &mut Agent,
    root_shape: ShapeId,
    prototype: Option<ObjectRef>,
    wrapper_kind: PrimitiveWrapperKind,
    value: Value,
    lifetime: AllocationLifetime,
) -> Completion<ObjectRef> {
    let string_plan = if wrapper_kind == PrimitiveWrapperKind::String {
        let string = if let Some(string) = value.as_string_ref() {
            string
        } else {
            return Err(throw_type_error(agent));
        };
        Some(plan_string_wrapper_cache(agent, string)?)
    } else {
        None
    };
    let element_capacity = string_plan.as_ref().map_or(0usize, |plan| {
        usize::try_from(plan.length).unwrap_or(usize::MAX)
    });
    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape)
                .with_prototype(prototype)
                .with_element_capacity(element_capacity)
                .with_cold_data(ObjectColdData::Ordinary(
                    OrdinaryObjectData::PrimitiveWrapper(wrapper_kind),
                ))
                .with_primitive_wrapper_value(value),
            lifetime,
        )
    });
    if let Some(plan) = string_plan {
        install_string_wrapper_elements(agent, object, &plan, lifetime)?;
    }
    Ok(object)
}

/// Returns the primitive payload for a matching wrapper receiver.
///
/// # Errors
/// Returns a `TypeError` abrupt completion when the receiver is neither the
/// primitive itself nor a wrapper object carrying the requested primitive kind.
pub fn require_primitive_wrapper_value(
    agent: &mut Agent,
    value: Value,
    expected: PrimitiveWrapperKind,
) -> Completion<Value> {
    if primitive_matches_kind(expected, value) {
        return Ok(value);
    }

    let Some(object) = value.as_object_ref() else {
        return Err(throw_type_error(agent));
    };
    if agent.objects().primitive_wrapper_kind(object) != Some(expected) {
        return Err(throw_type_error(agent));
    }

    agent
        .objects()
        .primitive_wrapper_value(agent.heap().view(), object)
        .ok_or_else(|| throw_type_error(agent))
}

/// Returns the stored time-value payload for a Date receiver.
///
/// # Errors
/// Returns a `TypeError` abrupt completion when the receiver is not a Date
/// object with an installed ordinary payload value.
pub fn require_date_value(agent: &mut Agent, value: Value) -> Completion<Value> {
    let Some(object) = value.as_object_ref() else {
        return Err(throw_type_error(agent));
    };
    if !agent.objects().is_date_object(object) {
        return Err(throw_type_error(agent));
    }

    agent
        .objects()
        .date_value(agent.heap().view(), object)
        .ok_or_else(|| throw_type_error(agent))
}

/// Returns the typed Temporal payload for one matching Temporal receiver.
///
/// # Errors
/// Returns a `TypeError` abrupt completion when the receiver is not a Temporal
/// object of the requested kind with an installed typed payload.
pub fn require_temporal_object(
    agent: &mut Agent,
    value: Value,
    expected: TemporalObjectKind,
) -> Completion<TemporalObjectData> {
    let Some(object) = value.as_object_ref() else {
        return Err(throw_type_error(agent));
    };
    if !agent.objects().is_temporal_object_kind(object, expected) {
        return Err(throw_type_error(agent));
    }

    agent
        .objects()
        .temporal_object(object)
        .copied()
        .ok_or_else(|| throw_type_error(agent))
}

pub fn define_private_field_layout(
    agent: &mut Agent,
    class_object: ObjectRef,
    prototype: ObjectRef,
    name: lyng_js_common::AtomId,
    is_static: bool,
) -> Completion<u32> {
    agent
        .with_heap_and_objects(|_heap, objects| {
            objects.define_private_field_layout(class_object, prototype, name, is_static)
        })
        .ok_or_else(|| throw_type_error(agent))
}

pub fn define_private_element_layout(
    agent: &mut Agent,
    class_object: ObjectRef,
    prototype: ObjectRef,
    name: lyng_js_common::AtomId,
    is_static: bool,
    kind: ClassPrivateElementKind,
) -> Completion<u32> {
    agent
        .with_heap_and_objects(|_heap, objects| {
            objects.define_private_element_layout(class_object, prototype, name, is_static, kind)
        })
        .ok_or_else(|| throw_type_error(agent))
}

pub fn install_private_element_value(
    agent: &mut Agent,
    class_key: ObjectRef,
    descriptor_index: u32,
    value: Value,
) -> Completion<Value> {
    let installed = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.install_private_element_value(
            &mut mutator,
            class_key,
            descriptor_index,
            value,
            AllocationLifetime::Default,
        )
    });
    installed
        .map_err(|error| internal_method_error(agent, error))
        .map(|()| value)
}

pub fn install_instance_public_field_key(
    agent: &mut Agent,
    class_object: ObjectRef,
    field_index: u32,
    key_value: Value,
) -> Completion<Value> {
    let installed = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.install_instance_public_field_key(
            &mut mutator,
            class_object,
            field_index,
            key_value,
            AllocationLifetime::Default,
        )
    });
    installed
        .map_err(|error| internal_method_error(agent, error))
        .map(|()| key_value)
}

pub fn instance_public_field_key(
    agent: &mut Agent,
    class_object: ObjectRef,
    field_index: u32,
) -> Completion<Value> {
    agent
        .objects()
        .instance_public_field_key(agent.heap().view(), class_object, field_index)
        .map_err(|error| internal_method_error(agent, error))
}

pub fn private_element_kind(
    agent: &mut Agent,
    class_key: ObjectRef,
    descriptor_index: u32,
) -> Completion<ClassPrivateElementKind> {
    agent
        .objects()
        .private_element_kind(class_key, descriptor_index)
        .map_err(|error| internal_method_error(agent, error))
}

pub fn private_shared_element_value(
    agent: &mut Agent,
    class_key: ObjectRef,
    descriptor_index: u32,
) -> Completion<Value> {
    agent
        .objects()
        .private_shared_element_value(agent.heap().view(), class_key, descriptor_index)
        .map_err(|error| internal_method_error(agent, error))
}

pub fn private_field_init(
    agent: &mut Agent,
    receiver: ObjectRef,
    class_key: ObjectRef,
    descriptor_index: u32,
    value: Value,
) -> Completion<Value> {
    let initialized = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.private_field_init(
            &mut mutator,
            receiver,
            class_key,
            descriptor_index,
            value,
            AllocationLifetime::Default,
        )
    });
    initialized
        .map_err(|error| internal_method_error(agent, error))
        .map(|()| value)
}

pub fn private_field_get(
    agent: &mut Agent,
    receiver: ObjectRef,
    class_key: ObjectRef,
    descriptor_index: u32,
) -> Completion<Value> {
    agent
        .objects()
        .private_field_get(agent.heap().view(), receiver, class_key, descriptor_index)
        .map_err(|error| internal_method_error(agent, error))
}

pub fn private_field_set(
    agent: &mut Agent,
    receiver: ObjectRef,
    class_key: ObjectRef,
    descriptor_index: u32,
    value: Value,
) -> Completion<Value> {
    let updated = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.private_field_set(&mut mutator, receiver, class_key, descriptor_index, value)
    });
    updated
        .map_err(|error| internal_method_error(agent, error))
        .map(|()| value)
}

pub fn private_has(
    agent: &mut Agent,
    receiver: ObjectRef,
    class_key: ObjectRef,
    descriptor_index: u32,
) -> Completion<bool> {
    agent
        .objects()
        .private_has(receiver, class_key, descriptor_index)
        .map_err(|error| internal_method_error(agent, error))
}

/// ECMAScript `ToPrimitive` over the shared wrapper and property substrate.
///
/// # Errors
/// Returns the caller-provided error type when property lookup or method calls
/// fail, or when the conversion cannot produce a primitive.
pub fn to_primitive<Cx: ToPrimitiveContext>(
    cx: &mut Cx,
    value: Value,
    hint: ToPrimitiveHint,
) -> Result<Value, Cx::Error> {
    let Some(object) = value.as_object_ref() else {
        return Ok(value);
    };

    let exotic = {
        let agent = cx.agent();
        agent.well_known_symbol(WellKnownSymbolId::ToPrimitive)
    };
    if let Some(symbol) = exotic {
        if let Some(method) = get_method(cx, object, PropertyKey::from_symbol(symbol))? {
            let hint_value = {
                let agent = cx.agent();
                Value::from_string_ref(agent.alloc_runtime_string(
                    hint.hint_text(),
                    None,
                    AllocationLifetime::Default,
                ))
            };
            let result =
                cx.call_to_completion(method, Value::from_object_ref(object), &[hint_value])?;
            if !result.is_object() {
                return Ok(result);
            }
            return Err(cx.type_error());
        }
    }

    ordinary_to_primitive(cx, object, hint)
}

/// Object-aware ECMAScript `ToNumber`.
///
/// # Errors
/// Returns the caller-provided error type when `ToPrimitive` or the underlying
/// numeric conversion fails.
pub fn to_number<Cx: ToPrimitiveContext>(cx: &mut Cx, value: Value) -> Result<Value, Cx::Error> {
    let primitive = to_primitive(cx, value, ToPrimitiveHint::Number)?;
    let number = {
        let agent = cx.agent();
        read::to_number(agent.heap().view(), primitive)
    };
    map_completion(cx, number)
}

/// Object-aware ECMAScript `ToNumeric`.
///
/// # Errors
/// Returns the caller-provided error type when `ToPrimitive` or the underlying
/// numeric conversion fails.
pub fn to_numeric<Cx: ToPrimitiveContext>(cx: &mut Cx, value: Value) -> Result<Value, Cx::Error> {
    let primitive = to_primitive(cx, value, ToPrimitiveHint::Number)?;
    let numeric = {
        let agent = cx.agent();
        read::to_numeric(agent.heap().view(), primitive)
    };
    map_completion(cx, numeric)
}

/// Converts one already-primitive ECMAScript value into a BigInt.
///
/// # Errors
/// Returns `TypeError` for unsupported logical types, `SyntaxError` for invalid
/// BigInt strings, and `RangeError` for non-integral number input.
pub fn primitive_to_bigint(agent: &mut Agent, value: Value) -> Completion<Value> {
    if value.is_bigint() {
        return Ok(value);
    }
    if let Some(boolean) = value.as_bool() {
        let bigint = agent.heap_mut().mutator().alloc_bigint(
            lyng_js_gc::BigIntSign::NonNegative,
            &[u64::from(boolean)],
            AllocationLifetime::Default,
        );
        return Ok(Value::from_bigint_ref(bigint));
    }
    if let Some(number) = value.as_f64() {
        let Some((sign, limbs)) = integral_number_to_bigint(number) else {
            return Err(throw_range_error(agent));
        };
        let bigint =
            agent
                .heap_mut()
                .mutator()
                .alloc_bigint(sign, &limbs, AllocationLifetime::Default);
        return Ok(Value::from_bigint_ref(bigint));
    }
    if let Some(string) = value.as_string_ref() {
        let text = agent
            .heap()
            .view()
            .string_view(string)
            .map(crate::convert::lossy_string_from_view)
            .ok_or_else(|| throw_type_error(agent))?;
        let Some((sign, limbs)) = parse_string_to_bigint(&text) else {
            return Err(throw_syntax_error(agent));
        };
        let bigint =
            agent
                .heap_mut()
                .mutator()
                .alloc_bigint(sign, &limbs, AllocationLifetime::Default);
        return Ok(Value::from_bigint_ref(bigint));
    }
    Err(throw_type_error(agent))
}

/// ECMAScript `StringToBigInt` for relational BigInt/string comparisons.
///
/// # Errors
/// Returns `TypeError` if the input is not a live string handle.
pub fn string_to_bigint_value(agent: &mut Agent, value: Value) -> Completion<Option<Value>> {
    let string = value
        .as_string_ref()
        .ok_or_else(|| throw_type_error(agent))?;
    let text = agent
        .heap()
        .view()
        .string_view(string)
        .map(lossy_string_from_view)
        .ok_or_else(|| throw_type_error(agent))?;
    let Some((sign, limbs)) = parse_string_to_bigint(&text) else {
        return Ok(None);
    };
    let bigint = agent
        .heap_mut()
        .mutator()
        .alloc_bigint(sign, &limbs, AllocationLifetime::Default);
    Ok(Some(Value::from_bigint_ref(bigint)))
}

/// Formats one BigInt value using the selected radix.
///
/// # Errors
/// Returns `TypeError` when the value is not a live BigInt handle.
pub fn bigint_to_string(agent: &mut Agent, value: Value, radix: u32) -> Completion<String> {
    let bigint = value
        .as_bigint_ref()
        .ok_or_else(|| throw_type_error(agent))?;
    let heap_view = agent.heap().view();
    let Some(view) = heap_view.bigint_view(bigint) else {
        return Err(throw_type_error(agent));
    };
    Ok(if radix == 10 {
        bigint_view_to_string(view)
    } else {
        bigint_view_to_radix_string(view, radix)
    })
}

/// Formats one integral ECMAScript number using the selected non-decimal radix.
#[allow(clippy::cast_possible_truncation)]
pub fn integral_number_to_radix_string(number: f64, radix: u32) -> Option<String> {
    let (sign, limbs) = integral_number_to_bigint(number)?;
    Some(bigint_parts_to_radix_string(sign, &limbs, radix))
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
    if index >= record.length() {
        return Ok(None);
    }
    if agent
        .backing_store_is_detached(record.backing_store())
        .unwrap_or(false)
    {
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

fn typed_array_index_descriptor(
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

/// ECMAScript `HasProperty` over the public object substrate.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn has_property(agent: &mut Agent, object: ObjectRef, key: PropertyKey) -> Completion<bool> {
    if let Some(index) = key.as_index() {
        if agent.objects().typed_array(object).is_some() {
            return Ok(typed_array_index_descriptor(agent, object, index)?.is_some());
        }
    }
    agent
        .objects()
        .has_property(agent.heap().view(), object, key)
        .map_err(|error| internal_method_error(agent, error))
}

/// ECMAScript `Get` over the public object substrate.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn get(agent: &mut Agent, object: ObjectRef, key: PropertyKey) -> Completion<Value> {
    get_with_receiver(agent, object, key, Value::from_object_ref(object))
}

/// ECMAScript `Get` over the public object substrate with an explicit receiver.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn get_with_receiver(
    agent: &mut Agent,
    object: ObjectRef,
    key: PropertyKey,
    receiver: Value,
) -> Completion<Value> {
    if let Some(index) = key.as_index() {
        if agent.objects().typed_array(object).is_some() {
            return Ok(typed_array_index_descriptor(agent, object, index)?
                .and_then(|descriptor| descriptor.value())
                .unwrap_or(Value::undefined()));
        }
    }
    agent
        .objects()
        .get(agent.heap().view(), object, key, receiver)
        .map_err(|error| internal_method_error(agent, error))
}

/// Resolves the `super` base object from one `[[HomeObject]]`.
///
/// # Errors
/// Returns an abrupt completion if the home object has a null prototype or the
/// underlying object internal methods fail.
pub fn super_base(agent: &mut Agent, home_object: ObjectRef) -> Completion<ObjectRef> {
    agent
        .objects()
        .get_prototype_of(agent.heap().view(), home_object)
        .map_err(|error| internal_method_error(agent, error))?
        .ok_or_else(|| throw_type_error(agent))
}

/// ECMAScript `GetSuper`-style helper using a pre-resolved home object and receiver.
///
/// # Errors
/// Returns an abrupt completion if the home object has a null prototype or the
/// underlying object internal methods fail.
pub fn super_get(
    agent: &mut Agent,
    home_object: ObjectRef,
    receiver: Value,
    key: PropertyKey,
) -> Completion<Value> {
    let base = super_base(agent, home_object)?;
    get_with_receiver(agent, base, key, receiver)
}

/// ECMAScript `[[GetPrototypeOf]]` over the public object substrate.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn get_prototype_of(agent: &mut Agent, object: ObjectRef) -> Completion<Option<ObjectRef>> {
    agent
        .objects()
        .get_prototype_of(agent.heap().view(), object)
        .map_err(|error| internal_method_error(agent, error))
}

/// ECMAScript `[[SetPrototypeOf]]` over the public object substrate.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn set_prototype_of(
    agent: &mut Agent,
    object: ObjectRef,
    prototype: Option<ObjectRef>,
) -> Completion<bool> {
    let result = agent.with_heap_and_objects(|heap, objects| {
        objects.set_prototype_of(&mut heap.mutator(), object, prototype)
    });
    result.map_err(|error| internal_method_error(agent, error))
}

/// ECMAScript `[[GetOwnProperty]]` over the public object substrate.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn get_own_property(
    agent: &mut Agent,
    object: ObjectRef,
    key: PropertyKey,
) -> Completion<Option<PropertyDescriptor>> {
    if let Some(index) = key.as_index() {
        if agent.objects().typed_array(object).is_some() {
            return typed_array_index_descriptor(agent, object, index);
        }
    }
    agent
        .objects()
        .get_own_property(agent.heap().view(), object, key)
        .map_err(|error| internal_method_error(agent, error))
}

/// ECMAScript `[[DefineOwnProperty]]` over the public object substrate.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn define_property(
    agent: &mut Agent,
    object: ObjectRef,
    key: PropertyKey,
    descriptor: PropertyDescriptor,
    lifetime: AllocationLifetime,
) -> Completion<bool> {
    let result = agent.with_heap_and_objects(|heap, objects| {
        objects.define_own_property(&mut heap.mutator(), object, key, descriptor, lifetime)
    });
    result.map_err(|error| internal_method_error(agent, error))
}

/// ECMAScript `[[IsExtensible]]` over the public object substrate.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn is_extensible(agent: &mut Agent, object: ObjectRef) -> Completion<bool> {
    agent
        .objects()
        .is_extensible(object)
        .map_err(|error| internal_method_error(agent, error))
}

/// ECMAScript `[[PreventExtensions]]` over the public object substrate.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn prevent_extensions(agent: &mut Agent, object: ObjectRef) -> Completion<bool> {
    let result = agent
        .with_heap_and_objects(|heap, objects| objects.prevent_extensions(heap.view(), object));
    result.map_err(|error| internal_method_error(agent, error))
}

/// ECMAScript `[[OwnPropertyKeys]]` over the public object substrate.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn own_property_keys(agent: &mut Agent, object: ObjectRef) -> Completion<Vec<PropertyKey>> {
    agent
        .objects()
        .own_property_keys(agent.heap().view(), object)
        .map_err(|error| internal_method_error(agent, error))
}

/// ECMAScript `Set` over the public object substrate.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn set(
    agent: &mut Agent,
    object: ObjectRef,
    key: PropertyKey,
    value: Value,
    lifetime: AllocationLifetime,
) -> Completion<bool> {
    set_with_receiver(
        agent,
        object,
        key,
        value,
        Value::from_object_ref(object),
        lifetime,
    )
}

/// ECMAScript `Set` over the public object substrate with an explicit receiver.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn set_with_receiver(
    agent: &mut Agent,
    object: ObjectRef,
    key: PropertyKey,
    value: Value,
    receiver: Value,
    lifetime: AllocationLifetime,
) -> Completion<bool> {
    let result = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.set(&mut mutator, object, key, value, receiver, lifetime)
    });
    result.map_err(|error| internal_method_error(agent, error))
}

/// ECMAScript `SetSuper`-style helper using a pre-resolved home object and receiver.
///
/// # Errors
/// Returns an abrupt completion if the home object has a null prototype or the
/// underlying object internal methods fail.
pub fn super_set(
    agent: &mut Agent,
    home_object: ObjectRef,
    receiver: Value,
    key: PropertyKey,
    value: Value,
    lifetime: AllocationLifetime,
) -> Completion<bool> {
    let base = super_base(agent, home_object)?;
    set_with_receiver(agent, base, key, value, receiver, lifetime)
}

/// ECMAScript `DeletePropertyOrThrow`-style primitive over the public object substrate.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn delete_property(agent: &mut Agent, object: ObjectRef, key: PropertyKey) -> Completion<bool> {
    let result = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.delete(&mut mutator, object, key)
    });
    result.map_err(|error| internal_method_error(agent, error))
}

/// ECMAScript `CreateDataProperty` over the public object substrate.
///
/// # Errors
/// Returns an abrupt completion if the underlying object internal methods fail.
pub fn create_data_property(
    agent: &mut Agent,
    object: ObjectRef,
    key: PropertyKey,
    value: Value,
    lifetime: AllocationLifetime,
) -> Completion<bool> {
    let mut descriptor = PropertyDescriptor::new();
    descriptor.set_value(value);
    descriptor.set_writable(true);
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);

    let result = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.define_own_property(&mut mutator, object, key, descriptor, lifetime)
    });
    result.map_err(|error| internal_method_error(agent, error))
}

/// ECMAScript `Call` over the public object substrate.
///
/// # Errors
/// Returns an abrupt completion if the callee is not an object or if the
/// underlying call internal methods fail.
pub fn call(
    agent: &mut Agent,
    callee: ObjectRef,
    this_value: Value,
    arguments: &[Value],
    registry: &mut dyn NativeFunctionRegistry,
) -> Completion<Value> {
    let result = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.call(&mut mutator, callee, this_value, arguments, registry)
    });
    result.map_err(|error| internal_method_error(agent, error))
}

/// ECMAScript `Construct` over the public object substrate.
///
/// # Errors
/// Returns an abrupt completion if the callee is not an object or if the
/// underlying construct internal methods fail.
pub fn construct(
    agent: &mut Agent,
    callee: ObjectRef,
    arguments: &[Value],
    new_target: Option<ObjectRef>,
    registry: &mut dyn NativeFunctionRegistry,
) -> Completion<ObjectRef> {
    let result = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.construct(&mut mutator, callee, arguments, new_target, registry)
    });
    result.map_err(|error| internal_method_error(agent, error))
}

fn primitive_wrapper_kind_for_value(value: Value) -> Option<PrimitiveWrapperKind> {
    if value.is_string() {
        return Some(PrimitiveWrapperKind::String);
    }
    if value.is_number() {
        return Some(PrimitiveWrapperKind::Number);
    }
    if value.is_bool() {
        return Some(PrimitiveWrapperKind::Boolean);
    }
    if value.is_symbol() {
        return Some(PrimitiveWrapperKind::Symbol);
    }
    if value.is_bigint() {
        return Some(PrimitiveWrapperKind::BigInt);
    }
    None
}

fn wrapper_prototype(
    agent: &Agent,
    realm: RealmRef,
    wrapper_kind: PrimitiveWrapperKind,
) -> Option<ObjectRef> {
    let intrinsics = agent.realm(realm)?.intrinsics();
    match wrapper_kind {
        PrimitiveWrapperKind::String => intrinsics.string_prototype(),
        PrimitiveWrapperKind::Number => intrinsics.number_prototype(),
        PrimitiveWrapperKind::Boolean => intrinsics.boolean_prototype(),
        PrimitiveWrapperKind::Symbol => intrinsics.symbol_prototype(),
        PrimitiveWrapperKind::BigInt => intrinsics.bigint_prototype(),
    }
}

fn primitive_matches_kind(expected: PrimitiveWrapperKind, value: Value) -> bool {
    match expected {
        PrimitiveWrapperKind::String => value.is_string(),
        PrimitiveWrapperKind::Number => value.is_number(),
        PrimitiveWrapperKind::Boolean => value.is_bool(),
        PrimitiveWrapperKind::Symbol => value.is_symbol(),
        PrimitiveWrapperKind::BigInt => value.is_bigint(),
    }
}

struct StringWrapperCachePlan {
    length: u32,
    latin1_bytes: Option<Vec<u8>>,
    utf16_units: Option<Vec<u16>>,
}

fn plan_string_wrapper_cache(
    agent: &mut Agent,
    string: lyng_js_types::StringRef,
) -> Completion<StringWrapperCachePlan> {
    let heap_view = agent.heap().view();
    let Some(view) = heap_view.string_view(string) else {
        return Err(throw_type_error(agent));
    };
    let length = view.code_unit_len();
    if let Some(bytes) = view.latin1_bytes() {
        return Ok(StringWrapperCachePlan {
            length,
            latin1_bytes: Some(bytes.to_vec()),
            utf16_units: None,
        });
    }
    let utf16_units = view.utf16_bytes().map(|bytes| {
        bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>()
    });
    Ok(StringWrapperCachePlan {
        length,
        latin1_bytes: None,
        utf16_units,
    })
}

fn install_string_wrapper_elements(
    agent: &mut Agent,
    object: ObjectRef,
    plan: &StringWrapperCachePlan,
    lifetime: AllocationLifetime,
) -> Completion<()> {
    if let Some(bytes) = &plan.latin1_bytes {
        for (index, byte) in bytes.iter().copied().enumerate() {
            let element = agent.heap_mut().mutator().alloc_string(
                StringEncoding::Latin1,
                1,
                &[byte],
                None,
                lifetime,
            );
            let stored = agent.with_heap_and_objects(|heap, objects| {
                objects.init_element(
                    &mut heap.mutator(),
                    object,
                    u32::try_from(index).unwrap_or(u32::MAX),
                    Value::from_string_ref(element),
                )
            });
            if !stored {
                return Err(throw_type_error(agent));
            }
        }
        return Ok(());
    }

    let Some(units) = &plan.utf16_units else {
        return Ok(());
    };
    for (index, unit) in units.iter().copied().enumerate() {
        let element = agent.heap_mut().mutator().alloc_string(
            StringEncoding::Utf16,
            1,
            &unit.to_le_bytes(),
            None,
            lifetime,
        );
        let stored = agent.with_heap_and_objects(|heap, objects| {
            objects.init_element(
                &mut heap.mutator(),
                object,
                u32::try_from(index).unwrap_or(u32::MAX),
                Value::from_string_ref(element),
            )
        });
        if !stored {
            return Err(throw_type_error(agent));
        }
    }
    Ok(())
}

fn map_completion<Cx: ToPrimitiveContext, T>(
    cx: &mut Cx,
    completion: Completion<T>,
) -> Result<T, Cx::Error> {
    completion.map_err(|completion| cx.abrupt(completion))
}

fn get_method<Cx: ToPrimitiveContext>(
    cx: &mut Cx,
    object: ObjectRef,
    key: PropertyKey,
) -> Result<Option<ObjectRef>, Cx::Error> {
    let method = cx.get_property_value(object, key)?;
    if method.is_undefined() || method.is_null() {
        return Ok(None);
    }
    cx.require_callable_object(method).map(Some)
}

/// ECMAScript `OrdinaryToPrimitive` for callers that have already selected the
/// preferred hint and must bypass exotic `@@toPrimitive` dispatch.
///
/// # Errors
/// Returns the caller-provided error type when property lookup, method call, or
/// primitive extraction fails.
pub fn ordinary_to_primitive<Cx: ToPrimitiveContext>(
    cx: &mut Cx,
    object: ObjectRef,
    hint: ToPrimitiveHint,
) -> Result<Value, Cx::Error> {
    for method_name in hint.method_names() {
        let key = PropertyKey::from_atom(method_name);
        let method = cx.get_property_value(object, key)?;
        let Some(method) = cx.callable_object(method) else {
            continue;
        };
        if let Some(result) = default_ordinary_to_primitive_result(cx, object, method_name, method)?
        {
            return Ok(result);
        }
        if let Some(result) = cx.default_to_primitive_result(object, method_name, method)? {
            return Ok(result);
        }
        let result = cx.call_to_completion(method, Value::from_object_ref(object), &[])?;
        if !result.is_object() {
            return Ok(result);
        }
    }
    Err(cx.type_error())
}

fn default_ordinary_to_primitive_result<Cx: ToPrimitiveContext>(
    cx: &mut Cx,
    object: ObjectRef,
    method_name: lyng_js_common::AtomId,
    method_object: ObjectRef,
) -> Result<Option<Value>, Cx::Error> {
    if method_name != WellKnownAtom::toString.id() && method_name != WellKnownAtom::valueOf.id() {
        return Ok(None);
    }

    let wrapper_kind = {
        let agent = cx.agent();
        agent.objects().primitive_wrapper_kind(object)
    };
    let Some(wrapper_kind) = wrapper_kind else {
        return Ok(None);
    };
    if !is_default_object_prototype_method(cx, object, method_name, method_object)? {
        return Ok(None);
    }

    let wrapper_value = {
        let agent = cx.agent();
        agent
            .objects()
            .primitive_wrapper_value(agent.heap().view(), object)
    }
    .ok_or_else(|| cx.type_error())?;

    if method_name == WellKnownAtom::valueOf.id() {
        return match wrapper_kind {
            PrimitiveWrapperKind::String
            | PrimitiveWrapperKind::Number
            | PrimitiveWrapperKind::BigInt => Ok(Some(wrapper_value)),
            PrimitiveWrapperKind::Boolean | PrimitiveWrapperKind::Symbol => Ok(None),
        };
    }

    match wrapper_kind {
        PrimitiveWrapperKind::String => Ok(Some(wrapper_value)),
        PrimitiveWrapperKind::Number => {
            let text = number_to_string(
                wrapper_value
                    .as_f64()
                    .expect("wrapper number payload should expose a numeric value"),
            );
            let value = {
                let agent = cx.agent();
                Value::from_string_ref(agent.alloc_runtime_string(
                    &text,
                    None,
                    AllocationLifetime::Default,
                ))
            };
            Ok(Some(value))
        }
        PrimitiveWrapperKind::BigInt => {
            let text = {
                let agent = cx.agent();
                let bigint = wrapper_value
                    .as_bigint_ref()
                    .expect("wrapper bigint payload should expose a bigint value");
                let view = agent.heap().view().bigint_view(bigint);
                let Some(view) = view else {
                    return Err(cx.type_error());
                };
                bigint_view_to_string(view)
            };
            let value = {
                let agent = cx.agent();
                Value::from_string_ref(agent.alloc_runtime_string(
                    &text,
                    None,
                    AllocationLifetime::Default,
                ))
            };
            Ok(Some(value))
        }
        PrimitiveWrapperKind::Boolean | PrimitiveWrapperKind::Symbol => Ok(None),
    }
}

fn is_default_object_prototype_method<Cx: ToPrimitiveContext>(
    cx: &mut Cx,
    object: ObjectRef,
    method_name: lyng_js_common::AtomId,
    method_object: ObjectRef,
) -> Result<bool, Cx::Error> {
    let key = PropertyKey::from_atom(method_name);
    let prototype = {
        let agent = cx.agent();
        get_prototype_of(agent, object)
    };
    let Some(wrapper_prototype) = map_completion(cx, prototype)? else {
        return Ok(false);
    };

    let wrapper_descriptor = {
        let agent = cx.agent();
        get_own_property(agent, wrapper_prototype, key)
    };
    if map_completion(cx, wrapper_descriptor)?.is_some() {
        return Ok(false);
    }

    let object_prototype = {
        let agent = cx.agent();
        get_prototype_of(agent, wrapper_prototype)
    };
    let Some(object_prototype) = map_completion(cx, object_prototype)? else {
        return Ok(false);
    };

    let descriptor = {
        let agent = cx.agent();
        get_own_property(agent, object_prototype, key)
    };
    let Some(descriptor) = map_completion(cx, descriptor)? else {
        return Ok(false);
    };

    Ok(descriptor.value() == Some(Value::from_object_ref(method_object)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_env::Runtime;
    use lyng_js_gc::{AllocationLifetime, BigIntSign, SymbolFlags};
    use lyng_js_host::NoopHostHooks;
    use lyng_js_objects::{
        ArrayBufferObjectData, FunctionConstructorFlags, FunctionObjectData, FunctionThisMode,
        InternalMethodResult, NativeCallRequest, NativeConstructRequest, ObjectAllocation,
        ObjectColdData, ObjectRuntime, OrdinaryObjectData, TemporalInstantObjectData,
        TemporalObjectData, TemporalObjectKind, TypedArrayElementKind, TypedArrayObjectData,
    };
    use lyng_js_types::{
        BuiltinFunctionId, EnvironmentRef, PropertyKey, RealmRef, WellKnownSymbolId,
    };

    fn install_test_wrapper_prototypes(agent: &mut Agent) -> RealmRef {
        let default_realm = agent.default_realm().expect("default realm should exist");
        let root_shape = default_realm
            .root_shape()
            .expect("default realm should expose a root shape");
        let (
            object_prototype,
            string_prototype,
            number_prototype,
            bigint_prototype,
            boolean_prototype,
            symbol_prototype,
        ) = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let object_prototype = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape),
                AllocationLifetime::Default,
            );
            let string_prototype = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape).with_prototype(Some(object_prototype)),
                AllocationLifetime::Default,
            );
            let number_prototype = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape).with_prototype(Some(object_prototype)),
                AllocationLifetime::Default,
            );
            let bigint_prototype = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape).with_prototype(Some(object_prototype)),
                AllocationLifetime::Default,
            );
            let boolean_prototype = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape).with_prototype(Some(object_prototype)),
                AllocationLifetime::Default,
            );
            let symbol_prototype = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape).with_prototype(Some(object_prototype)),
                AllocationLifetime::Default,
            );
            (
                object_prototype,
                string_prototype,
                number_prototype,
                bigint_prototype,
                boolean_prototype,
                symbol_prototype,
            )
        });
        let intrinsics = default_realm
            .intrinsics()
            .with_object_prototype(Some(object_prototype))
            .with_string_prototype(Some(string_prototype))
            .with_number_prototype(Some(number_prototype))
            .with_bigint_prototype(Some(bigint_prototype))
            .with_boolean_prototype(Some(boolean_prototype))
            .with_symbol_prototype(Some(symbol_prototype));
        assert!(agent.set_realm_intrinsics(default_realm.id(), intrinsics));
        default_realm.id()
    }

    fn install_test_uint8_array(agent: &mut Agent, elements: &[u8]) -> ObjectRef {
        let realm = agent.default_realm().expect("default realm should exist");
        let root_shape = realm
            .root_shape()
            .expect("default realm should expose a root shape");
        let backing_store = agent
            .allocate_backing_store(elements.len())
            .expect("typed-array test backing store should allocate");
        for (index, byte) in elements.iter().copied().enumerate() {
            assert!(agent.backing_store_set_byte(backing_store, index, byte));
        }

        agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let buffer = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape)
                    .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::ArrayBuffer)),
                AllocationLifetime::Default,
            );
            assert!(objects
                .install_array_buffer_object(buffer, ArrayBufferObjectData::new(backing_store)));

            let typed_array = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape)
                    .with_element_capacity(elements.len())
                    .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::TypedArray(
                        TypedArrayElementKind::Uint8,
                    ))),
                AllocationLifetime::Default,
            );
            assert!(objects.install_typed_array_object(
                typed_array,
                TypedArrayObjectData::new(
                    buffer,
                    backing_store,
                    0,
                    elements.len(),
                    TypedArrayElementKind::Uint8,
                ),
            ));
            for (index, byte) in elements.iter().copied().enumerate() {
                assert!(objects.init_element(
                    &mut mutator,
                    typed_array,
                    u32::try_from(index).expect("typed-array index should fit into u32"),
                    Value::from_smi(i32::from(byte)),
                ));
            }
            typed_array
        })
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct RecordedCall {
        callee: ObjectRef,
        this_value: Value,
        arguments: Vec<Value>,
        realm: RealmRef,
        environment: EnvironmentRef,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct RecordedConstruct {
        callee: ObjectRef,
        new_target: ObjectRef,
        arguments: Vec<Value>,
        realm: RealmRef,
        environment: EnvironmentRef,
    }

    #[derive(Default)]
    struct RecordingRegistry {
        last_call: Option<RecordedCall>,
        last_construct: Option<RecordedConstruct>,
    }

    impl NativeFunctionRegistry for RecordingRegistry {
        fn call(
            &mut self,
            _runtime: &mut ObjectRuntime,
            _heap: &mut lyng_js_gc::PrimitiveMutator<'_>,
            request: NativeCallRequest<'_>,
        ) -> InternalMethodResult<Value> {
            self.last_call = Some(RecordedCall {
                callee: request.callee(),
                this_value: request.this_value(),
                arguments: request.arguments().to_vec(),
                realm: request.realm(),
                environment: request.environment(),
            });
            Ok(Value::from_smi(77))
        }

        fn construct(
            &mut self,
            runtime: &mut ObjectRuntime,
            heap: &mut lyng_js_gc::PrimitiveMutator<'_>,
            request: NativeConstructRequest<'_>,
        ) -> InternalMethodResult<ObjectRef> {
            self.last_construct = Some(RecordedConstruct {
                callee: request.callee(),
                new_target: request.new_target(),
                arguments: request.arguments().to_vec(),
                realm: request.realm(),
                environment: request.environment(),
            });
            let root = runtime.root_shape(heap, None, AllocationLifetime::Default);
            Ok(runtime.alloc_object(
                heap,
                ObjectAllocation::ordinary(root).with_prototype(Some(request.new_target())),
                AllocationLifetime::Default,
            ))
        }
    }

    #[test]
    fn object_property_wrappers_delegate_to_internal_methods() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let default_realm = agent.default_realm().expect("default realm should exist");
        let key = PropertyKey::from_atom(lyng_js_common::AtomId::from_raw(501));
        let object = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let root_shape = default_realm
                .root_shape()
                .expect("default realm should expose a root shape");
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape),
                AllocationLifetime::Default,
            )
        });

        assert!(create_data_property(
            agent,
            object,
            key,
            Value::from_smi(11),
            AllocationLifetime::Default,
        )
        .unwrap());
        assert!(has_property(agent, object, key).unwrap());
        assert_eq!(get(agent, object, key).unwrap(), Value::from_smi(11));
        assert!(set(
            agent,
            object,
            key,
            Value::from_smi(13),
            AllocationLifetime::Default,
        )
        .unwrap());
        assert_eq!(get(agent, object, key).unwrap(), Value::from_smi(13));
        assert!(delete_property(agent, object, key).unwrap());
        assert!(!has_property(agent, object, key).unwrap());
    }

    #[test]
    fn typed_array_index_reads_observe_live_backing_store_bytes() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let typed_array = install_test_uint8_array(agent, &[1]);
        let typed_array_record = agent
            .objects()
            .typed_array(typed_array)
            .expect("test typed array should install its view record");
        assert!(agent.backing_store_set_byte(typed_array_record.backing_store(), 0, 7));

        let descriptor = get_own_property(agent, typed_array, PropertyKey::Index(0))
            .expect("typed-array descriptor lookup should succeed")
            .expect("typed-array index should still exist");
        let value = get(agent, typed_array, PropertyKey::Index(0))
            .expect("typed-array indexed get should succeed");

        assert_eq!(descriptor.value(), Some(Value::from_smi(7)));
        assert_eq!(value, Value::from_smi(7));
    }

    #[test]
    fn call_and_construct_wrappers_delegate_to_function_runtime() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let default_realm = agent.default_realm().expect("default realm should exist");
        let entry = BuiltinFunctionId::from_raw(7).expect("builtin id");
        let function = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let root_shape = default_realm
                .root_shape()
                .expect("default realm should expose a root shape");
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::function(root_shape).with_cold_data(ObjectColdData::Function(
                    FunctionObjectData::native(
                        default_realm.id(),
                        default_realm.global_env(),
                        entry,
                    )
                    .with_this_mode(FunctionThisMode::Global)
                    .with_constructor_flags(FunctionConstructorFlags::constructible()),
                )),
                AllocationLifetime::Default,
            )
        });
        let mut registry = RecordingRegistry::default();

        let call_result = call(
            agent,
            function,
            Value::from_smi(3),
            &[Value::from_smi(4), Value::from_smi(5)],
            &mut registry,
        )
        .unwrap();
        let constructed =
            construct(agent, function, &[Value::from_smi(9)], None, &mut registry).unwrap();

        assert_eq!(call_result, Value::from_smi(77));
        assert_eq!(
            registry.last_call,
            Some(RecordedCall {
                callee: function,
                this_value: Value::from_smi(3),
                arguments: vec![Value::from_smi(4), Value::from_smi(5)],
                realm: default_realm.id(),
                environment: default_realm.global_env(),
            })
        );
        assert_eq!(
            registry.last_construct,
            Some(RecordedConstruct {
                callee: function,
                new_target: function,
                arguments: vec![Value::from_smi(9)],
                realm: default_realm.id(),
                environment: default_realm.global_env(),
            })
        );
        assert_eq!(
            agent
                .objects()
                .get_prototype_of(agent.heap().view(), constructed)
                .unwrap(),
            Some(function)
        );
    }

    #[test]
    fn to_object_wraps_phase5_and_6a1_primitive_families() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = install_test_wrapper_prototypes(agent);
        let string = agent.alloc_runtime_string("x", None, AllocationLifetime::Default);
        let bigint = agent.heap_mut().mutator().alloc_bigint(
            BigIntSign::NonNegative,
            &[17],
            AllocationLifetime::Default,
        );
        let symbol = agent.heap_mut().mutator().alloc_symbol(
            None,
            SymbolFlags::ordinary(),
            AllocationLifetime::Default,
        );
        let string_wrapper =
            to_object(agent, realm, Value::from_string_ref(string)).expect("String should wrap");
        let number_wrapper =
            to_object(agent, realm, Value::from_smi(41)).expect("Number should wrap");
        let boolean_wrapper =
            to_object(agent, realm, Value::from_bool(true)).expect("Boolean should wrap");
        let symbol_wrapper =
            to_object(agent, realm, Value::from_symbol_ref(symbol)).expect("Symbol should wrap");
        let bigint_wrapper =
            to_object(agent, realm, Value::from_bigint_ref(bigint)).expect("BigInt should wrap");

        assert_eq!(
            agent.objects().primitive_wrapper_kind(string_wrapper),
            Some(PrimitiveWrapperKind::String)
        );
        assert_eq!(
            agent.objects().primitive_wrapper_kind(number_wrapper),
            Some(PrimitiveWrapperKind::Number)
        );
        assert_eq!(
            agent.objects().primitive_wrapper_kind(boolean_wrapper),
            Some(PrimitiveWrapperKind::Boolean)
        );
        assert_eq!(
            agent
                .objects()
                .primitive_wrapper_value(agent.heap().view(), string_wrapper),
            Some(Value::from_string_ref(string))
        );
        assert_eq!(
            agent
                .objects()
                .primitive_wrapper_value(agent.heap().view(), number_wrapper),
            Some(Value::from_smi(41))
        );
        assert_eq!(
            agent
                .objects()
                .primitive_wrapper_value(agent.heap().view(), boolean_wrapper),
            Some(Value::from_bool(true))
        );
        assert_eq!(
            agent.objects().primitive_wrapper_kind(symbol_wrapper),
            Some(PrimitiveWrapperKind::Symbol)
        );
        assert_eq!(
            agent.objects().primitive_wrapper_kind(bigint_wrapper),
            Some(PrimitiveWrapperKind::BigInt)
        );
        assert_eq!(
            agent
                .objects()
                .primitive_wrapper_value(agent.heap().view(), symbol_wrapper),
            Some(Value::from_symbol_ref(symbol))
        );
        assert_eq!(
            agent
                .objects()
                .primitive_wrapper_value(agent.heap().view(), bigint_wrapper),
            Some(Value::from_bigint_ref(bigint))
        );
        assert!(to_object(agent, realm, Value::null()).is_err());
    }

    #[test]
    fn primitive_wrapper_value_accepts_primitive_or_matching_wrapper() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = install_test_wrapper_prototypes(agent);
        let symbol = agent
            .well_known_symbol(WellKnownSymbolId::ToPrimitive)
            .expect("well-known symbol should exist");
        let symbol_wrapper = wrap_primitive_value(
            agent,
            realm,
            Value::from_symbol_ref(symbol),
            AllocationLifetime::Default,
        )
        .expect("Symbol wrapper should allocate");

        assert_eq!(
            require_primitive_wrapper_value(
                agent,
                Value::from_bool(false),
                PrimitiveWrapperKind::Boolean
            ),
            Ok(Value::from_bool(false))
        );
        assert_eq!(
            require_primitive_wrapper_value(
                agent,
                Value::from_symbol_ref(symbol),
                PrimitiveWrapperKind::Symbol
            ),
            Ok(Value::from_symbol_ref(symbol))
        );
        assert_eq!(
            require_primitive_wrapper_value(
                agent,
                Value::from_object_ref(symbol_wrapper),
                PrimitiveWrapperKind::Symbol
            ),
            Ok(Value::from_symbol_ref(symbol))
        );
        assert!(require_primitive_wrapper_value(
            agent,
            Value::from_object_ref(symbol_wrapper),
            PrimitiveWrapperKind::Boolean
        )
        .is_err());
    }

    #[test]
    fn string_wrapper_install_preserves_length_and_elements() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = install_test_wrapper_prototypes(agent);
        let string = agent.alloc_runtime_string("abc", None, AllocationLifetime::Default);
        let wrapper = wrap_primitive_value(
            agent,
            realm,
            Value::from_string_ref(string),
            AllocationLifetime::Default,
        )
        .expect("String wrapper should allocate");

        let length = get(
            agent,
            wrapper,
            PropertyKey::from_atom(WellKnownAtom::length.id()),
        )
        .unwrap();
        let first = get(agent, wrapper, PropertyKey::Index(0)).unwrap();

        assert_eq!(
            agent.objects().primitive_wrapper_kind(wrapper),
            Some(PrimitiveWrapperKind::String)
        );
        assert_eq!(
            agent
                .objects()
                .primitive_wrapper_value(agent.heap().view(), wrapper),
            Some(Value::from_string_ref(string))
        );
        assert_eq!(length, Value::from_smi(3));
        assert_eq!(
            agent
                .heap()
                .view()
                .string(first.as_string_ref().unwrap())
                .unwrap()
                .cached_atom(),
            None
        );
    }

    struct WrapperToPrimitiveContext<'a> {
        agent: &'a mut Agent,
    }

    impl ToPrimitiveContext for WrapperToPrimitiveContext<'_> {
        type Error = AbruptCompletion;

        fn agent(&mut self) -> &mut Agent {
            self.agent
        }

        fn abrupt(&mut self, completion: AbruptCompletion) -> Self::Error {
            completion
        }

        fn type_error(&mut self) -> Self::Error {
            throw_type_error(self.agent)
        }

        fn get_property_value(
            &mut self,
            object: ObjectRef,
            key: PropertyKey,
        ) -> Result<Value, Self::Error> {
            get(self.agent, object, key)
        }

        fn require_callable_object(&mut self, value: Value) -> Result<ObjectRef, Self::Error> {
            let Some(object) = value.as_object_ref() else {
                return Err(throw_type_error(self.agent));
            };
            if self.agent.objects().function_data(object).is_some() {
                Ok(object)
            } else {
                Err(throw_type_error(self.agent))
            }
        }

        fn call_to_completion(
            &mut self,
            _callee_object: ObjectRef,
            _this_value: Value,
            _arguments: &[Value],
        ) -> Result<Value, Self::Error> {
            panic!("wrapper fallback should avoid calling default Object.prototype methods")
        }
    }

    fn install_default_object_to_primitive_methods(
        agent: &mut Agent,
        realm: RealmRef,
    ) -> (ObjectRef, ObjectRef) {
        let realm_record = agent.realm(realm).expect("realm should exist");
        let root_shape = realm_record
            .root_shape()
            .expect("realm should expose a root shape");
        let global_env = realm_record.global_env();
        let object_prototype = realm_record
            .intrinsics()
            .object_prototype()
            .expect("wrapper test realm should expose Object.prototype");
        let (to_string, value_of) = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let to_string = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::function(root_shape).with_cold_data(ObjectColdData::Function(
                    FunctionObjectData::native(
                        realm,
                        global_env,
                        BuiltinFunctionId::from_raw(40).expect("builtin id"),
                    ),
                )),
                AllocationLifetime::Default,
            );
            let value_of = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::function(root_shape).with_cold_data(ObjectColdData::Function(
                    FunctionObjectData::native(
                        realm,
                        global_env,
                        BuiltinFunctionId::from_raw(41).expect("builtin id"),
                    ),
                )),
                AllocationLifetime::Default,
            );
            (to_string, value_of)
        });
        assert!(create_data_property(
            agent,
            object_prototype,
            PropertyKey::from_atom(WellKnownAtom::toString.id()),
            Value::from_object_ref(to_string),
            AllocationLifetime::Default,
        )
        .unwrap());
        assert!(create_data_property(
            agent,
            object_prototype,
            PropertyKey::from_atom(WellKnownAtom::valueOf.id()),
            Value::from_object_ref(value_of),
            AllocationLifetime::Default,
        )
        .unwrap());
        (to_string, value_of)
    }

    #[test]
    fn to_primitive_uses_default_object_prototype_fallbacks_for_6a1_wrappers() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = install_test_wrapper_prototypes(agent);
        let (_to_string, _value_of) = install_default_object_to_primitive_methods(agent, realm);
        let string = agent.alloc_runtime_string("abc", None, AllocationLifetime::Default);
        let bigint = agent.heap_mut().mutator().alloc_bigint(
            BigIntSign::NonNegative,
            &[19],
            AllocationLifetime::Default,
        );
        let number_wrapper = wrap_primitive_value(
            agent,
            realm,
            Value::from_smi(7),
            AllocationLifetime::Default,
        )
        .expect("Number wrapper should allocate");
        let string_wrapper = wrap_primitive_value(
            agent,
            realm,
            Value::from_string_ref(string),
            AllocationLifetime::Default,
        )
        .expect("String wrapper should allocate");
        let bigint_wrapper = wrap_primitive_value(
            agent,
            realm,
            Value::from_bigint_ref(bigint),
            AllocationLifetime::Default,
        )
        .expect("BigInt wrapper should allocate");
        let mut context = WrapperToPrimitiveContext { agent };

        assert_eq!(
            to_primitive(
                &mut context,
                Value::from_object_ref(number_wrapper),
                ToPrimitiveHint::Number,
            ),
            Ok(Value::from_smi(7))
        );
        assert_eq!(
            to_primitive(
                &mut context,
                Value::from_object_ref(string_wrapper),
                ToPrimitiveHint::Number,
            ),
            Ok(Value::from_string_ref(string))
        );
        let number_text = to_primitive(
            &mut context,
            Value::from_object_ref(number_wrapper),
            ToPrimitiveHint::String,
        )
        .expect("number wrapper should stringify");
        let bigint_text = to_primitive(
            &mut context,
            Value::from_object_ref(bigint_wrapper),
            ToPrimitiveHint::String,
        )
        .expect("bigint wrapper should stringify");
        assert_eq!(
            context
                .agent
                .heap()
                .view()
                .string_view(number_text.as_string_ref().unwrap())
                .unwrap()
                .latin1_bytes(),
            Some(&b"7"[..])
        );
        assert_eq!(
            context
                .agent
                .heap()
                .view()
                .string_view(bigint_text.as_string_ref().unwrap())
                .unwrap()
                .latin1_bytes(),
            Some(&b"19"[..])
        );
    }

    #[test]
    fn require_temporal_object_returns_installed_payload_for_matching_kind() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let default_realm = agent.default_realm().expect("default realm should exist");
        let instant = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let root_shape = default_realm
                .root_shape()
                .expect("default realm should expose a root shape");
            let instant = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape).with_cold_data(ObjectColdData::Ordinary(
                    OrdinaryObjectData::Temporal(TemporalObjectKind::Instant),
                )),
                AllocationLifetime::Default,
            );
            assert!(objects.install_temporal_object(
                instant,
                TemporalObjectData::Instant(TemporalInstantObjectData::new(77)),
            ));
            instant
        });

        let payload = require_temporal_object(
            agent,
            Value::from_object_ref(instant),
            TemporalObjectKind::Instant,
        )
        .expect("Temporal.Instant should expose its typed payload");

        assert_eq!(
            payload,
            TemporalObjectData::Instant(TemporalInstantObjectData::new(77))
        );
    }

    #[test]
    fn require_temporal_object_rejects_plain_objects_and_wrong_temporal_kind() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let default_realm = agent.default_realm().expect("default realm should exist");
        let (plain, instant) = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let root_shape = default_realm
                .root_shape()
                .expect("default realm should expose a root shape");
            let plain = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape),
                AllocationLifetime::Default,
            );
            let instant = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape).with_cold_data(ObjectColdData::Ordinary(
                    OrdinaryObjectData::Temporal(TemporalObjectKind::Instant),
                )),
                AllocationLifetime::Default,
            );
            assert!(objects.install_temporal_object(
                instant,
                TemporalObjectData::Instant(TemporalInstantObjectData::new(77)),
            ));
            (plain, instant)
        });

        assert!(require_temporal_object(
            agent,
            Value::from_object_ref(plain),
            TemporalObjectKind::Instant,
        )
        .is_err());
        assert!(require_temporal_object(
            agent,
            Value::from_object_ref(instant),
            TemporalObjectKind::ZonedDateTime,
        )
        .is_err());
    }
}
