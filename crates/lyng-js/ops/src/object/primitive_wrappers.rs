use crate::errors::{self, throw_type_error};
use lyng_js_env::Agent;
use lyng_js_gc::{AllocationLifetime, StringEncoding};
use lyng_js_objects::{ObjectAllocation, ObjectColdData, OrdinaryObjectData, PrimitiveWrapperKind};
use lyng_js_types::{AbruptCompletion, Completion, ObjectRef, RealmRef, ShapeId, StringRef, Value};

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
        return Err(throw_type_error_for_realm(agent, realm));
    }

    wrap_primitive_value(agent, realm, value, AllocationLifetime::Default)
}

fn throw_type_error_for_realm(agent: &mut Agent, realm: RealmRef) -> AbruptCompletion {
    errors::create_intrinsic_error_object(agent, realm, errors::ErrorKind::Type, None)
        .map(Value::from_object_ref)
        .map_or_else(|completion| completion, AbruptCompletion::throw)
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
    string: StringRef,
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
