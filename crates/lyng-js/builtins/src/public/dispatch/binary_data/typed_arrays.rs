mod access;
mod base64_hex;
mod construction;
mod iteration;
mod mutation;
mod search;

pub(super) use access::dispatch_typed_array_access_builtin;
pub(super) use base64_hex::dispatch_uint8_array_base64_hex_builtin;
pub(super) use construction::dispatch_typed_array_constructor_builtin;
pub(super) use iteration::dispatch_typed_array_iteration_builtin;
pub(super) use mutation::dispatch_typed_array_mutation_builtin;
pub(super) use search::dispatch_typed_array_search_builtin;

use super::{length_value_u64, range_error, type_error, PublicBuiltinDispatchContext};
use crate::public::dispatch::BuiltinToPrimitiveBridge;
use lyng_js_common::WellKnownAtom;
use lyng_js_env::Agent;
use lyng_js_gc::AllocationLifetime;
use lyng_js_objects::{
    ObjectAllocation, ObjectColdData, OrdinaryObjectData, TypedArrayElementKind,
    TypedArrayObjectData,
};
use lyng_js_ops::typed_array;
use lyng_js_types::{ObjectRef, PropertyKey, RealmRef, Value, WellKnownSymbolId};

pub(super) const fn typed_array_storage_u8_bits(bits: u64) -> u8 {
    typed_array::storage_u8_bits(bits)
}

pub(super) const fn typed_array_storage_u16_bits(bits: u64) -> u16 {
    typed_array::storage_u16_bits(bits)
}

pub(super) const fn typed_array_storage_u32_bits(bits: u64) -> u32 {
    typed_array::storage_u32_bits(bits)
}

pub(super) fn typed_array_storage_bits_to_value(
    agent: &mut Agent,
    kind: TypedArrayElementKind,
    bits: u64,
) -> Value {
    typed_array::value_from_storage_bits(agent, kind, bits)
}

pub(in crate::public::dispatch) fn typed_array_storage_bits_from_builtin_value<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    kind: TypedArrayElementKind,
    value: Value,
) -> Result<u64, Cx::Error> {
    let mut bridge = BuiltinToPrimitiveBridge { cx };
    typed_array::storage_bits_from_value(&mut bridge, kind, value)
}

pub(super) fn typed_array_read_storage_bits(
    agent: &Agent,
    record: TypedArrayObjectData,
    element_index: usize,
) -> Option<u64> {
    typed_array::read_storage_bits(agent, record, element_index)
}

pub(in crate::public::dispatch) fn typed_array_write_storage_bits<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    record: TypedArrayObjectData,
    element_index: usize,
    bits: u64,
) -> Result<(), Cx::Error> {
    if !typed_array::write_storage_bits(cx.agent(), record, element_index, bits) {
        return Err(range_error(cx));
    }
    Ok(())
}

pub(super) fn typed_array_snapshot_storage_bits(
    agent: &Agent,
    record: TypedArrayObjectData,
) -> Vec<u64> {
    (0..typed_array_current_length(agent, record).unwrap_or(0))
        .map(|index| typed_array_read_storage_bits(agent, record, index).unwrap_or(0))
        .collect()
}

pub(super) fn typed_array_read_element_value(
    agent: &mut Agent,
    record: TypedArrayObjectData,
    index: usize,
) -> Value {
    typed_array::read_element_value(agent, record, index)
}

pub(in crate::public::dispatch) fn typed_array_is_out_of_bounds(
    agent: &Agent,
    record: TypedArrayObjectData,
) -> bool {
    typed_array::is_out_of_bounds(agent, record)
}

pub(in crate::public::dispatch) fn typed_array_current_length(
    agent: &Agent,
    record: TypedArrayObjectData,
) -> Option<usize> {
    typed_array::current_length(agent, record)
}

pub(super) fn allocate_typed_array_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: ObjectRef,
    typed_array: TypedArrayObjectData,
) -> Result<ObjectRef, Cx::Error> {
    let root_shape = {
        let agent = cx.agent();
        agent.realm(realm).and_then(|realm| realm.root_shape())
    }
    .ok_or_else(|| type_error(cx))?;
    Ok(cx.agent().with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let object = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape)
                .with_prototype(Some(prototype))
                .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::TypedArray(
                    typed_array.kind(),
                ))),
            AllocationLifetime::Default,
        );
        let installed = objects.install_typed_array_object(object, typed_array);
        debug_assert!(
            installed,
            "fresh typed array should install its view record"
        );
        object
    }))
}

pub(super) fn typed_array_this_record<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TypedArrayObjectData, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    cx.agent()
        .objects()
        .typed_array(object)
        .ok_or_else(|| type_error(cx))
}

pub(super) fn typed_array_this_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if cx.agent().objects().typed_array(object).is_none() {
        return Err(type_error(cx));
    }
    Ok(object)
}

pub(super) fn typed_array_validated_record<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<TypedArrayObjectData, Cx::Error> {
    typed_array_validated_record_and_length(cx, value).map(|(record, _)| record)
}

pub(super) fn typed_array_validated_record_and_length<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<(TypedArrayObjectData, usize), Cx::Error> {
    let record = typed_array_this_record(cx, value)?;
    let length = typed_array_current_length(cx.agent(), record).ok_or_else(|| type_error(cx))?;
    Ok((record, length))
}

pub(in crate::public::dispatch) fn typed_array_validated_object_and_record<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    value: Value,
) -> Result<(ObjectRef, TypedArrayObjectData), Cx::Error> {
    let object = typed_array_this_object(cx, value)?;
    let record = typed_array_validated_record(cx, value)?;
    Ok((object, record))
}

pub(in crate::public::dispatch) fn typed_array_validated_object_record_and_length<
    Cx: PublicBuiltinDispatchContext,
>(
    cx: &mut Cx,
    value: Value,
) -> Result<(ObjectRef, TypedArrayObjectData, usize), Cx::Error> {
    let object = typed_array_this_object(cx, value)?;
    let (record, length) = typed_array_validated_record_and_length(cx, value)?;
    Ok((object, record, length))
}

pub(super) fn typed_array_default_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    kind: TypedArrayElementKind,
) -> Result<ObjectRef, Cx::Error> {
    let getter: fn(lyng_js_env::Intrinsics) -> Option<ObjectRef> = match kind {
        TypedArrayElementKind::Int8 => lyng_js_env::Intrinsics::int8_array_prototype,
        TypedArrayElementKind::Int16 => lyng_js_env::Intrinsics::int16_array_prototype,
        TypedArrayElementKind::Int32 => lyng_js_env::Intrinsics::int32_array_prototype,
        TypedArrayElementKind::Float16 => lyng_js_env::Intrinsics::float16_array_prototype,
        TypedArrayElementKind::Float32 => lyng_js_env::Intrinsics::float32_array_prototype,
        TypedArrayElementKind::Float64 => lyng_js_env::Intrinsics::float64_array_prototype,
        TypedArrayElementKind::BigInt64 => lyng_js_env::Intrinsics::big_int64_array_prototype,
        TypedArrayElementKind::BigUint64 => lyng_js_env::Intrinsics::big_uint64_array_prototype,
        TypedArrayElementKind::Uint32 => lyng_js_env::Intrinsics::uint32_array_prototype,
        TypedArrayElementKind::Uint16 => lyng_js_env::Intrinsics::uint16_array_prototype,
        TypedArrayElementKind::Uint8Clamped => {
            lyng_js_env::Intrinsics::uint8_clamped_array_prototype
        }
        TypedArrayElementKind::Uint8 => lyng_js_env::Intrinsics::uint8_array_prototype,
    };
    let prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .map(|realm| realm.intrinsics())
            .and_then(getter)
    };
    prototype.ok_or_else(|| type_error(cx))
}

fn typed_array_default_constructor<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    kind: TypedArrayElementKind,
) -> Result<ObjectRef, Cx::Error> {
    let getter: fn(lyng_js_env::Intrinsics) -> Option<ObjectRef> = match kind {
        TypedArrayElementKind::Int8 => lyng_js_env::Intrinsics::int8_array,
        TypedArrayElementKind::Int16 => lyng_js_env::Intrinsics::int16_array,
        TypedArrayElementKind::Int32 => lyng_js_env::Intrinsics::int32_array,
        TypedArrayElementKind::Float16 => lyng_js_env::Intrinsics::float16_array,
        TypedArrayElementKind::Float32 => lyng_js_env::Intrinsics::float32_array,
        TypedArrayElementKind::Float64 => lyng_js_env::Intrinsics::float64_array,
        TypedArrayElementKind::BigInt64 => lyng_js_env::Intrinsics::big_int64_array,
        TypedArrayElementKind::BigUint64 => lyng_js_env::Intrinsics::big_uint64_array,
        TypedArrayElementKind::Uint32 => lyng_js_env::Intrinsics::uint32_array,
        TypedArrayElementKind::Uint16 => lyng_js_env::Intrinsics::uint16_array,
        TypedArrayElementKind::Uint8Clamped => lyng_js_env::Intrinsics::uint8_clamped_array,
        TypedArrayElementKind::Uint8 => lyng_js_env::Intrinsics::uint8_array,
    };
    let constructor = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .map(|realm| realm.intrinsics())
            .and_then(getter)
    };
    constructor.ok_or_else(|| type_error(cx))
}

fn typed_array_species_constructor<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    exemplar: ObjectRef,
    kind: TypedArrayElementKind,
) -> Result<ObjectRef, Cx::Error> {
    let realm = cx.builtin_realm();
    let default_constructor = typed_array_default_constructor(cx, realm, kind)?;
    let constructor = cx.get_property_value(
        Value::from_object_ref(exemplar),
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
    )?;
    if constructor.is_undefined() {
        return Ok(default_constructor);
    }
    let constructor = constructor.as_object_ref().ok_or_else(|| type_error(cx))?;
    let species_symbol = cx
        .agent()
        .well_known_symbol(WellKnownSymbolId::Species)
        .ok_or_else(|| type_error(cx))?;
    let species = cx.get_property_value(
        Value::from_object_ref(constructor),
        PropertyKey::from_symbol(species_symbol),
    )?;
    if species.is_undefined() || species.is_null() {
        return Ok(default_constructor);
    }
    let species = species.as_object_ref().ok_or_else(|| type_error(cx))?;
    if !cx.agent().objects().is_constructor(species) {
        return Err(type_error(cx));
    }
    Ok(species)
}

pub(super) fn typed_array_species_create_with_arguments<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    exemplar: ObjectRef,
    kind: TypedArrayElementKind,
    arguments: &[Value],
    minimum_length: Option<usize>,
) -> Result<(ObjectRef, TypedArrayObjectData), Cx::Error> {
    let constructor = typed_array_species_constructor(cx, exemplar, kind)?;
    let object = cx.construct_to_completion(constructor, arguments, None)?;
    let (record, actual_length) =
        typed_array_validated_record_and_length(cx, Value::from_object_ref(object))?;
    if let Some(length) = minimum_length
        && actual_length < length
    {
        return Err(type_error(cx));
    }
    Ok((object, record))
}

pub(super) fn typed_array_species_create<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    exemplar: ObjectRef,
    kind: TypedArrayElementKind,
    length: usize,
) -> Result<(ObjectRef, TypedArrayObjectData), Cx::Error> {
    let arguments = [length_value_u64(u64::try_from(length).unwrap_or(u64::MAX))];
    typed_array_species_create_with_arguments(cx, exemplar, kind, &arguments, Some(length))
}

pub(super) fn typed_array_same_kind_create<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    kind: TypedArrayElementKind,
    length: usize,
) -> Result<(ObjectRef, TypedArrayObjectData), Cx::Error> {
    let constructor = typed_array_default_constructor(cx, cx.builtin_realm(), kind)?;
    let arguments = [length_value_u64(u64::try_from(length).unwrap_or(u64::MAX))];
    let object = cx.construct_to_completion(constructor, &arguments, None)?;
    let (record, actual_length) =
        typed_array_validated_record_and_length(cx, Value::from_object_ref(object))?;
    if record.kind() != kind || actual_length != length {
        return Err(type_error(cx));
    }
    Ok((object, record))
}
