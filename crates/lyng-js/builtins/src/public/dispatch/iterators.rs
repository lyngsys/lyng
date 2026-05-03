use super::{
    array_like_length,
    binary_data::{typed_array_is_out_of_bounds, typed_array_validated_object_and_record},
    close_iterator_after_error, create_array_result, create_data_property_or_throw,
    get_property_from_object, length_value, map_completion, number_value,
    promises::{
        new_promise_capability, perform_promise_then_with_capability, promise_capability_promise,
        promise_capability_reject, promise_capability_resolve, promise_default_constructor,
        promise_resolve_method,
    },
    property_key_from_text, proxy_get_own_property, proxy_own_property_keys, range_error,
    set_property_on_object, string_from_code_units, string_ref_code_units, string_ref_text,
    string_this_ref, to_number_for_builtin, type_error, BuiltinIteratorBridge,
    PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_common::{AtomId, WellKnownAtom};
use lyng_js_env::{Agent, PromiseCapabilityId, PromiseReactionHandler};
use lyng_js_gc::AllocationLifetime;
use lyng_js_objects::{ObjectAllocation, ObjectColdData, OrdinaryObjectData};
use lyng_js_ops::{errors, iterator, read};
use lyng_js_types::{
    AbruptCompletion, BuiltinFunctionId, ObjectRef, PropertyKey, Value, WellKnownSymbolId,
};

pub(super) fn dispatch_iterator_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::iterator_prototype_iterator_builtin() {
        return iterator_prototype_iterator_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_builtin() {
        return iterator_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_from_builtin() {
        return iterator_from_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_concat_builtin() {
        return iterator_concat_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_zip_builtin() {
        return iterator_zip_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_zip_keyed_builtin() {
        return iterator_zip_keyed_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_reduce_builtin() {
        return iterator_reduce_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_for_each_builtin() {
        return iterator_for_each_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_some_builtin() {
        return iterator_some_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_every_builtin() {
        return iterator_every_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_find_builtin() {
        return iterator_find_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_to_array_builtin() {
        return iterator_to_array_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_map_builtin() {
        return iterator_map_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_filter_builtin() {
        return iterator_filter_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_flat_map_builtin() {
        return iterator_flat_map_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_take_builtin() {
        return iterator_take_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_drop_builtin() {
        return iterator_drop_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_dispose_builtin() {
        return iterator_dispose_builtin(context, invocation).map(Some);
    }
    if entry == super::async_iterator_dispose_builtin() {
        return async_iterator_dispose_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_helper_next_builtin() {
        return iterator_helper_next_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_helper_return_builtin() {
        return iterator_helper_return_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_to_string_tag_getter_builtin() {
        return iterator_to_string_tag_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_to_string_tag_setter_builtin() {
        return iterator_to_string_tag_setter_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_constructor_getter_builtin() {
        return iterator_constructor_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::iterator_constructor_setter_builtin() {
        return iterator_constructor_setter_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ArrayIterationKind {
    Key = 0,
    Value = 1,
    Entry = 2,
}

pub(super) const ARRAY_ITERATOR_TARGET_SLOT: u32 = 0;
pub(super) const ARRAY_ITERATOR_INDEX_SLOT: u32 = 1;
pub(super) const ARRAY_ITERATOR_KIND_SLOT: u32 = 2;
pub(super) const MAP_ITERATOR_TARGET_SLOT: u32 = 0;
pub(super) const MAP_ITERATOR_INDEX_SLOT: u32 = 1;
pub(super) const MAP_ITERATOR_KIND_SLOT: u32 = 2;
pub(super) const SET_ITERATOR_TARGET_SLOT: u32 = 0;
pub(super) const SET_ITERATOR_INDEX_SLOT: u32 = 1;
pub(super) const SET_ITERATOR_KIND_SLOT: u32 = 2;
pub(super) const STRING_ITERATOR_STRING_SLOT: u32 = 0;
pub(super) const STRING_ITERATOR_INDEX_SLOT: u32 = 1;
const ITERATOR_HELPER_ITERATED_SLOT: u32 = 0;
const ITERATOR_HELPER_NEXT_METHOD_SLOT: u32 = 1;
const ITERATOR_HELPER_DONE_SLOT: u32 = 2;
const ITERATOR_HELPER_RUNNING_SLOT: u32 = 3;
const ITERATOR_HELPER_KIND_SLOT: u32 = 4;
const ITERATOR_HELPER_PARAM_SLOT: u32 = 5;
const ITERATOR_HELPER_COUNTER_SLOT: u32 = 6;
const ITERATOR_HELPER_INNER_ITERATED_SLOT: u32 = 7;
const ITERATOR_HELPER_INNER_NEXT_METHOD_SLOT: u32 = 8;
const ITERATOR_HELPER_SEQUENCE_BASE_SLOT: u32 = 7;
const ITERATOR_ZIP_RECORD_WIDTH: u32 = 6;
const ITERATOR_ZIP_KEY_KIND_OFFSET: u32 = 0;
const ITERATOR_ZIP_KEY_PAYLOAD_OFFSET: u32 = 1;
const ITERATOR_ZIP_ITERATED_OFFSET: u32 = 2;
const ITERATOR_ZIP_NEXT_METHOD_OFFSET: u32 = 3;
const ITERATOR_ZIP_ALIVE_OFFSET: u32 = 4;
const ITERATOR_ZIP_PADDING_OFFSET: u32 = 5;

impl ArrayIterationKind {
    #[inline]
    pub(super) const fn from_value(value: Value) -> Option<Self> {
        match value.as_smi() {
            Some(0) => Some(Self::Key),
            Some(1) => Some(Self::Value),
            Some(2) => Some(Self::Entry),
            _ => None,
        }
    }

    #[inline]
    pub(super) const fn into_value(self) -> Value {
        Value::from_smi(self as i32)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IteratorHelperKind {
    Map = 0,
    Filter = 1,
    Take = 2,
    Drop = 3,
    FlatMap = 4,
    Wrap = 5,
    Concat = 6,
    Zip = 7,
    ZipKeyed = 8,
}

impl IteratorHelperKind {
    #[inline]
    const fn from_value(value: Value) -> Option<Self> {
        match value.as_smi() {
            Some(0) => Some(Self::Map),
            Some(1) => Some(Self::Filter),
            Some(2) => Some(Self::Take),
            Some(3) => Some(Self::Drop),
            Some(4) => Some(Self::FlatMap),
            Some(5) => Some(Self::Wrap),
            Some(6) => Some(Self::Concat),
            Some(7) => Some(Self::Zip),
            Some(8) => Some(Self::ZipKeyed),
            _ => None,
        }
    }

    #[inline]
    const fn into_value(self) -> Value {
        Value::from_smi(self as i32)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IteratorZipMode {
    Shortest = 0,
    Longest = 1,
    Strict = 2,
}

impl IteratorZipMode {
    #[inline]
    const fn from_value(value: Value) -> Option<Self> {
        match value.as_smi() {
            Some(0) => Some(Self::Shortest),
            Some(1) => Some(Self::Longest),
            Some(2) => Some(Self::Strict),
            _ => None,
        }
    }

    #[inline]
    const fn into_value(self) -> Value {
        Value::from_smi(self as i32)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IteratorZipKey {
    None,
    Index(u32),
    Atom(AtomId),
    Symbol(lyng_js_types::SymbolRef),
}

#[derive(Clone, Copy, Debug)]
struct IteratorZipCollectedRecord {
    key: IteratorZipKey,
    iterator: ObjectRef,
    next_method: Value,
    padding: Value,
}

pub(super) fn create_iterator_result_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    done: bool,
) -> Result<Value, Cx::Error> {
    let result = {
        let realm = cx.builtin_realm();
        let agent = cx.agent();
        iterator::create_iterator_result_object(agent, realm, value, done)
    };
    Ok(Value::from_object_ref(map_completion(cx, result)?))
}

pub(super) fn allocate_iterator_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    prototype: lyng_js_types::ObjectRef,
    cold_data: OrdinaryObjectData,
    slot_values: &[Value],
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    let realm = cx.builtin_realm();
    let root_shape = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(lyng_js_env::RealmRecord::root_shape)
    }
    .ok_or_else(|| type_error(cx))?;
    let iterator_object = cx
        .agent()
        .with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let iterator_object = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape)
                    .with_prototype(Some(prototype))
                    .with_named_slot_count(slot_values.len())
                    .with_cold_data(ObjectColdData::Ordinary(cold_data)),
                AllocationLifetime::Default,
            );
            for (slot_index, slot_value) in slot_values.iter().copied().enumerate() {
                let slot_index =
                    u32::try_from(slot_index).expect("iterator slot index must fit into u32");
                if !objects.init_named_slot(&mut mutator, iterator_object, slot_index, slot_value) {
                    return None;
                }
            }
            Some(iterator_object)
        })
        .ok_or_else(|| type_error(cx))?;
    Ok(iterator_object)
}

fn iterator_slot_value(
    agent: &Agent,
    object_ref: lyng_js_types::ObjectRef,
    expected_kind: OrdinaryObjectData,
    slot_index: u32,
) -> Option<Value> {
    let heap_view = agent.heap().view();
    let matches_kind = matches!(
        agent.objects().object(heap_view, object_ref),
        Some(record)
            if matches!(
                record.cold(),
                ObjectColdData::Ordinary(data) if *data == expected_kind
            )
    );
    if !matches_kind {
        return None;
    }
    let value = agent
        .objects()
        .named_slots(heap_view, object_ref)?
        .get(slot_index as usize)
        .copied()?;
    (value != Value::empty_internal_slot()).then_some(value)
}

pub(super) fn iterator_slot_value_for_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
    expected_kind: OrdinaryObjectData,
    slot_index: u32,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        iterator_slot_value(agent, object_ref, expected_kind, slot_index)
    }
    .ok_or_else(|| type_error(cx))?;
    Ok(value)
}

pub(super) fn set_iterator_slot_value_for_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
    expected_kind: OrdinaryObjectData,
    slot_index: u32,
    value: Value,
) -> Result<(), Cx::Error> {
    let updated = cx.agent().with_heap_and_objects(|heap, objects| {
        let matches_kind = matches!(
            objects.object(heap.view(), object_ref),
            Some(record)
                if matches!(
                    record.cold(),
                    ObjectColdData::Ordinary(data) if *data == expected_kind
                )
        );
        if !matches_kind {
            return false;
        }
        let mut mutator = heap.mutator();
        objects.mut_named_slot(&mut mutator, object_ref, slot_index, value)
    });
    if updated {
        Ok(())
    } else {
        Err(type_error(cx))
    }
}

pub(super) fn array_iterator_factory_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: ArrayIterationKind,
) -> Result<Value, Cx::Error> {
    let object_ref = cx.to_object_for_builtin_value(cx.builtin_realm(), invocation.this_value())?;
    let prototype = {
        let realm = cx.builtin_realm();
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().array_iterator_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let slot_values = [
        Value::from_object_ref(object_ref),
        Value::from_smi(0),
        kind.into_value(),
    ];
    let iterator_object = allocate_iterator_object(
        cx,
        prototype,
        OrdinaryObjectData::ArrayIterator,
        &slot_values,
    )?;
    Ok(Value::from_object_ref(iterator_object))
}

pub(super) fn typed_array_iterator_factory_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: ArrayIterationKind,
) -> Result<Value, Cx::Error> {
    let (object_ref, _) = typed_array_validated_object_and_record(cx, invocation.this_value())?;
    let prototype = {
        let realm = cx.builtin_realm();
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().array_iterator_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let slot_values = [
        Value::from_object_ref(object_ref),
        Value::from_smi(0),
        kind.into_value(),
    ];
    let iterator_object = allocate_iterator_object(
        cx,
        prototype,
        OrdinaryObjectData::ArrayIterator,
        &slot_values,
    )?;
    Ok(Value::from_object_ref(iterator_object))
}

fn iterator_prototype_iterator_builtin<Cx: PublicBuiltinDispatchContext>(
    _cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    Ok(invocation.this_value())
}

fn array_iterator_target_length<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    target_object: ObjectRef,
) -> Result<u32, Cx::Error> {
    let typed_array = cx.agent().objects().typed_array(target_object);
    let Some(record) = typed_array else {
        return array_like_length(cx, target_object);
    };
    if cx
        .agent()
        .backing_store_is_detached(record.backing_store())
        .ok_or_else(|| type_error(cx))?
        || typed_array_is_out_of_bounds(cx.agent(), record)
    {
        return Err(type_error(cx));
    }
    Ok(u32::try_from(record.length()).unwrap_or(u32::MAX))
}

pub(super) fn array_iterator_next_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let iterator_object = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let target = iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::ArrayIterator,
        ARRAY_ITERATOR_TARGET_SLOT,
    )?;
    let Some(target_object) = target.as_object_ref() else {
        return create_iterator_result_value(cx, Value::undefined(), true);
    };
    let index = iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::ArrayIterator,
        ARRAY_ITERATOR_INDEX_SLOT,
    )?
    .as_smi()
    .and_then(|value| u32::try_from(value).ok())
    .ok_or_else(|| type_error(cx))?;
    let kind = ArrayIterationKind::from_value(iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::ArrayIterator,
        ARRAY_ITERATOR_KIND_SLOT,
    )?)
    .ok_or_else(|| type_error(cx))?;
    let length = array_iterator_target_length(cx, target_object)?;
    if index >= length {
        set_iterator_slot_value_for_builtin(
            cx,
            iterator_object,
            OrdinaryObjectData::ArrayIterator,
            ARRAY_ITERATOR_TARGET_SLOT,
            Value::undefined(),
        )?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    }
    set_iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::ArrayIterator,
        ARRAY_ITERATOR_INDEX_SLOT,
        length_value(index.saturating_add(1)),
    )?;
    let value = match kind {
        ArrayIterationKind::Key => length_value(index),
        ArrayIterationKind::Value => {
            get_property_from_object(cx, target_object, PropertyKey::Index(index))?
        }
        ArrayIterationKind::Entry => {
            let pair = create_array_result(cx, 2)?;
            let entry_value =
                get_property_from_object(cx, target_object, PropertyKey::Index(index))?;
            set_property_on_object(cx, pair, PropertyKey::Index(0), length_value(index))?;
            set_property_on_object(cx, pair, PropertyKey::Index(1), entry_value)?;
            Value::from_object_ref(pair)
        }
    };
    create_iterator_result_value(cx, value, false)
}

pub(super) fn string_iterator_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let string_ref = string_this_ref(cx, invocation.this_value())?;
    let prototype = {
        let realm = cx.builtin_realm();
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().string_iterator_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    let slot_values = [Value::from_string_ref(string_ref), Value::from_smi(0)];
    let iterator_object = allocate_iterator_object(
        cx,
        prototype,
        OrdinaryObjectData::StringIterator,
        &slot_values,
    )?;
    Ok(Value::from_object_ref(iterator_object))
}

pub(super) fn string_iterator_next_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let iterator_object = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let source = iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::StringIterator,
        STRING_ITERATOR_STRING_SLOT,
    )?;
    let Some(string_ref) = source.as_string_ref() else {
        return create_iterator_result_value(cx, Value::undefined(), true);
    };
    let index = iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::StringIterator,
        STRING_ITERATOR_INDEX_SLOT,
    )?
    .as_smi()
    .and_then(|value| usize::try_from(value).ok())
    .ok_or_else(|| type_error(cx))?;
    let units = string_ref_code_units(cx, string_ref)?;
    if index >= units.len() {
        set_iterator_slot_value_for_builtin(
            cx,
            iterator_object,
            OrdinaryObjectData::StringIterator,
            STRING_ITERATOR_STRING_SLOT,
            Value::undefined(),
        )?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    }
    let mut next_index = index + 1;
    let first = units[index];
    if (0xD800..=0xDBFF).contains(&first)
        && units
            .get(index + 1)
            .is_some_and(|second| (0xDC00..=0xDFFF).contains(second))
    {
        next_index += 1;
    }
    set_iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::StringIterator,
        STRING_ITERATOR_INDEX_SLOT,
        length_value(u32::try_from(next_index).unwrap_or(u32::MAX)),
    )?;
    let value = string_from_code_units(cx, &units[index..next_index]);
    create_iterator_result_value(cx, value, false)
}

// ====================================================================
// Iterator constructor + Stage-1 helpers (iterator-helpers proposal)
// ====================================================================
//
// Stage 1 here covers: the Iterator constructor (subclass-only), the eager
// helpers reduce/forEach/some/every/find/toArray, the initial lazy helpers
// map/filter/take/drop/flatMap, and Iterator.from including the
// WrapForValidIteratorPrototype branch. The static helper constructors are
// tracked by dcat issue lyng-3k8k.

fn iterator_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    // Step 1: Iterator() called as a function: TypeError.
    let new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    // Step 2: new Iterator() with NewTarget == Iterator (no subclass): TypeError.
    if new_target == cx.callee_object() {
        return Err(type_error(cx));
    }
    // Step 3: Subclass — allocate ordinary object with the subclass's prototype
    // chained through %Iterator.prototype%.
    let realm = cx.builtin_realm();
    let default_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().iterator_prototype())
        .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let object = cx.allocate_ordinary_object_with_prototype(realm, Some(prototype))?;
    Ok(Value::from_object_ref(object))
}

fn iterator_to_string_tag_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    // The default getter returns the literal string "Iterator". Per the
    // spec, custom subclass setters can override this on a per-instance
    // basis via the brand-checked accessor pair below; the getter only
    // observes the brand-installed override.
    let realm = cx.builtin_realm();
    let agent = cx.agent();
    let intrinsics = agent
        .realm(realm)
        .map(lyng_js_env::RealmRecord::intrinsics)
        .unwrap_or_default();
    let _ = intrinsics; // suppress unused warning; reserved for future custom-tag logic
    Ok(super::string_value(cx, "Iterator"))
}

fn iterator_to_string_tag_setter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    // SetterThatIgnoresPrototypeProperties: if `this` is the Iterator.prototype
    // itself, throw TypeError. Otherwise, define the property on `this` as a
    // plain data property (or update an existing data property).
    let this_value = invocation.this_value();
    let this_object = this_value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let realm = cx.builtin_realm();
    let iterator_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().iterator_prototype())
        .ok_or_else(|| type_error(cx))?;
    if this_object == iterator_prototype {
        return Err(type_error(cx));
    }
    let new_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let symbol_ref = cx
        .agent()
        .well_known_symbol(lyng_js_types::WellKnownSymbolId::ToStringTag)
        .ok_or_else(|| type_error(cx))?;
    let symbol_key = PropertyKey::from_symbol(symbol_ref);
    super::define_data_property_with_attrs(
        cx,
        this_object,
        symbol_key,
        new_value,
        true,
        true,
        true,
    )?;
    Ok(Value::undefined())
}

fn iterator_constructor_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    // The default getter returns %Iterator% (the constructor itself).
    let realm = cx.builtin_realm();
    let iterator = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().iterator())
        .ok_or_else(|| type_error(cx))?;
    Ok(Value::from_object_ref(iterator))
}

fn iterator_constructor_setter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    // Mirror image of iterator_to_string_tag_setter_builtin: refuse to set
    // on Iterator.prototype itself, otherwise install a data property on the
    // receiver.
    let this_value = invocation.this_value();
    let this_object = this_value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let realm = cx.builtin_realm();
    let iterator_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().iterator_prototype())
        .ok_or_else(|| type_error(cx))?;
    if this_object == iterator_prototype {
        return Err(type_error(cx));
    }
    let new_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let key = PropertyKey::from_atom(WellKnownAtom::constructor.id());
    super::define_data_property_with_attrs(cx, this_object, key, new_value, true, true, true)?;
    Ok(Value::undefined())
}

// Helper: build an IteratorRecord from an arbitrary `O` whose `next` is the
// only access we need (GetIteratorDirect from the spec).
fn iterator_direct_record<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
) -> Result<iterator::IteratorRecord, Cx::Error> {
    let next_key = property_key_from_text(cx, "next");
    let next_value = cx.get_property_value(Value::from_object_ref(object_ref), next_key)?;
    let next_method = cx.require_callable_object(next_value)?;
    Ok(iterator::IteratorRecord::new(object_ref, next_method))
}

// Helper: call O.return() if it exists, ignoring any errors. Used for the
// argument-validation-failure branch of the eager helpers, where the spec
// asks IteratorClose to run on a record whose [[NextMethod]] hasn't been
// populated yet (so we can't use the regular IteratorRecord-based close).
fn iterator_close_for_validation_failure<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
) {
    let return_key = property_key_from_text(cx, "return");
    let return_value = match cx.get_property_value(Value::from_object_ref(object_ref), return_key) {
        Ok(value) => value,
        Err(_) => return,
    };
    if return_value.is_undefined() || return_value.is_null() {
        return;
    }
    if let Ok(return_method) = cx.require_callable_object(return_value) {
        // Per spec, the original ThrowCompletion is preserved over any
        // completion produced by return(); ignore both Ok and Err here.
        let _ = cx.call_to_completion(return_method, Value::from_object_ref(object_ref), &[]);
    }
}

fn iterator_this_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    value.as_object_ref().ok_or_else(|| type_error(cx))
}

fn iterator_reduce_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = iterator_this_object(cx, invocation.this_value())?;
    let reducer_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let reducer = match cx.require_callable_object(reducer_value) {
        Ok(reducer) => reducer,
        Err(error) => {
            iterator_close_for_validation_failure(cx, object);
            return Err(error);
        }
    };
    let mut iterator_record = iterator_direct_record(cx, object)?;
    let initial = invocation.arguments().get(1).copied();
    let (mut accumulator, mut counter): (Value, u64) = if let Some(value) = initial {
        (value, 0)
    } else {
        let next = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_step(&mut bridge, &mut iterator_record)
        };
        let next = match next {
            Ok(next) => next,
            Err(error) => {
                iterator_record.set_done(true);
                return Err(error);
            }
        };
        let Some(next) = next else {
            return Err(type_error(cx));
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let value = match value {
            Ok(value) => value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        (value, 1)
    };
    loop {
        let next = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_step(&mut bridge, &mut iterator_record)
        };
        let next = match next {
            Ok(next) => next,
            Err(error) => {
                iterator_record.set_done(true);
                return Err(error);
            }
        };
        let Some(next) = next else {
            return Ok(accumulator);
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let value = match value {
            Ok(value) => value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let counter_value = u64_to_value(counter);
        match cx.call_to_completion(
            reducer,
            Value::undefined(),
            &[accumulator, value, counter_value],
        ) {
            Ok(result) => {
                accumulator = result;
                counter = counter.saturating_add(1);
            }
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        }
    }
}

fn iterator_for_each_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = iterator_this_object(cx, invocation.this_value())?;
    let callback_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let callback = match cx.require_callable_object(callback_value) {
        Ok(callback) => callback,
        Err(error) => {
            iterator_close_for_validation_failure(cx, object);
            return Err(error);
        }
    };
    let mut iterator_record = iterator_direct_record(cx, object)?;
    let mut counter: u64 = 0;
    loop {
        let next = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_step(&mut bridge, &mut iterator_record)
        };
        let next = match next {
            Ok(next) => next,
            Err(error) => {
                iterator_record.set_done(true);
                return Err(error);
            }
        };
        let Some(next) = next else {
            return Ok(Value::undefined());
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let value = match value {
            Ok(value) => value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let counter_value = u64_to_value(counter);
        if let Err(error) =
            cx.call_to_completion(callback, Value::undefined(), &[value, counter_value])
        {
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
        counter = counter.saturating_add(1);
    }
}

fn iterator_some_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    iterator_predicate_helper(cx, invocation, true)
}

fn iterator_every_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    iterator_predicate_helper(cx, invocation, false)
}

// some: returns true on first truthy → true; default false.
// every: returns false on first falsy → false; default true.
fn iterator_predicate_helper<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    is_some: bool,
) -> Result<Value, Cx::Error> {
    let object = iterator_this_object(cx, invocation.this_value())?;
    let callback_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let callback = match cx.require_callable_object(callback_value) {
        Ok(callback) => callback,
        Err(error) => {
            iterator_close_for_validation_failure(cx, object);
            return Err(error);
        }
    };
    let mut iterator_record = iterator_direct_record(cx, object)?;
    let mut counter: u64 = 0;
    loop {
        let next = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_step(&mut bridge, &mut iterator_record)
        };
        let next = match next {
            Ok(next) => next,
            Err(error) => {
                iterator_record.set_done(true);
                return Err(error);
            }
        };
        let Some(next) = next else {
            return Ok(Value::from_bool(!is_some));
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let value = match value {
            Ok(value) => value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let counter_value = u64_to_value(counter);
        let result =
            match cx.call_to_completion(callback, Value::undefined(), &[value, counter_value]) {
                Ok(result) => result,
                Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
            };
        let truthy = {
            let completion = {
                let agent = cx.agent();
                read::to_boolean(agent.heap().view(), result)
            };
            map_completion(cx, completion)?
        };
        if is_some && truthy {
            // some short-circuits to true
            let close_result = {
                let mut bridge = BuiltinIteratorBridge { cx };
                iterator::iterator_close(
                    &mut bridge,
                    &mut iterator_record,
                    Ok::<(), lyng_js_types::AbruptCompletion>(()),
                )
            };
            close_result?;
            return Ok(Value::from_bool(true));
        }
        if !is_some && !truthy {
            // every short-circuits to false
            let close_result = {
                let mut bridge = BuiltinIteratorBridge { cx };
                iterator::iterator_close(
                    &mut bridge,
                    &mut iterator_record,
                    Ok::<(), lyng_js_types::AbruptCompletion>(()),
                )
            };
            close_result?;
            return Ok(Value::from_bool(false));
        }
        counter = counter.saturating_add(1);
    }
}

fn iterator_find_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = iterator_this_object(cx, invocation.this_value())?;
    let callback_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let callback = match cx.require_callable_object(callback_value) {
        Ok(callback) => callback,
        Err(error) => {
            iterator_close_for_validation_failure(cx, object);
            return Err(error);
        }
    };
    let mut iterator_record = iterator_direct_record(cx, object)?;
    let mut counter: u64 = 0;
    loop {
        let next = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_step(&mut bridge, &mut iterator_record)
        };
        let next = match next {
            Ok(next) => next,
            Err(error) => {
                iterator_record.set_done(true);
                return Err(error);
            }
        };
        let Some(next) = next else {
            return Ok(Value::undefined());
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let value = match value {
            Ok(value) => value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let counter_value = u64_to_value(counter);
        let result =
            match cx.call_to_completion(callback, Value::undefined(), &[value, counter_value]) {
                Ok(result) => result,
                Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
            };
        let truthy = {
            let completion = {
                let agent = cx.agent();
                read::to_boolean(agent.heap().view(), result)
            };
            map_completion(cx, completion)?
        };
        if truthy {
            let close_result = {
                let mut bridge = BuiltinIteratorBridge { cx };
                iterator::iterator_close(
                    &mut bridge,
                    &mut iterator_record,
                    Ok::<(), lyng_js_types::AbruptCompletion>(()),
                )
            };
            close_result?;
            return Ok(value);
        }
        counter = counter.saturating_add(1);
    }
}

fn iterator_to_array_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object = iterator_this_object(cx, invocation.this_value())?;
    let mut iterator_record = iterator_direct_record(cx, object)?;
    let mut values: Vec<Value> = Vec::new();
    loop {
        let next = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_step(&mut bridge, &mut iterator_record)
        };
        let next = match next {
            Ok(next) => next,
            Err(error) => {
                iterator_record.set_done(true);
                return Err(error);
            }
        };
        let Some(next) = next else {
            break;
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let value = match value {
            Ok(value) => value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        values.push(value);
    }
    let array = super::create_array_from_values(cx, &values)?;
    Ok(Value::from_object_ref(array))
}

fn iterator_concat_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let arguments = invocation.arguments();
    let iterator_symbol = cx
        .agent()
        .well_known_symbol(WellKnownSymbolId::Iterator)
        .ok_or_else(|| type_error(cx))?;
    let count = i32::try_from(arguments.len()).map_err(|_| type_error(cx))?;
    let mut slot_values = Vec::with_capacity(ITERATOR_HELPER_SEQUENCE_BASE_SLOT as usize + 2);
    slot_values.extend_from_slice(&[
        Value::undefined(),
        Value::undefined(),
        Value::from_bool(false),
        Value::from_bool(false),
        IteratorHelperKind::Concat.into_value(),
        Value::from_smi(count),
        Value::from_smi(0),
    ]);
    for item in arguments.iter().copied() {
        let iterable = item.as_object_ref().ok_or_else(|| type_error(cx))?;
        let method = cx.get_property_value(
            Value::from_object_ref(iterable),
            PropertyKey::from_symbol(iterator_symbol),
        )?;
        if method.is_undefined() || method.is_null() {
            return Err(type_error(cx));
        }
        let method = cx.require_callable_object(method)?;
        slot_values.push(Value::from_object_ref(iterable));
        slot_values.push(Value::from_object_ref(method));
    }
    let realm = cx.builtin_realm();
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().iterator_helper_prototype())
        .ok_or_else(|| type_error(cx))?;
    let helper = allocate_iterator_object(
        cx,
        prototype,
        OrdinaryObjectData::IteratorHelper,
        slot_values.as_slice(),
    )?;
    Ok(Value::from_object_ref(helper))
}

fn iterator_zip_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let arguments = invocation.arguments();
    let iterables = arguments.first().copied().unwrap_or(Value::undefined());
    iterables.as_object_ref().ok_or_else(|| type_error(cx))?;
    let (mode, padding) = iterator_zip_options(cx, arguments)?;
    let mut input_record = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::get_iterator(&mut bridge, iterables)?
    };
    let mut records = Vec::new();
    loop {
        let next = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_step(&mut bridge, &mut input_record)
        };
        let next = match next {
            Ok(next) => next,
            Err(error) => return iterator_zip_close_collected_after_error(cx, &records, error),
        };
        let Some(next) = next else {
            break;
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let value = match value {
            Ok(value) => value,
            Err(error) => return iterator_zip_close_collected_after_error(cx, &records, error),
        };
        let (iterator, next_method) = match get_iterator_flattenable(cx, value) {
            Ok(record) => record,
            Err(error) => {
                return iterator_zip_close_collected_and_input_after_error(
                    cx,
                    &records,
                    &mut input_record,
                    error,
                );
            }
        };
        records.push(IteratorZipCollectedRecord {
            key: IteratorZipKey::None,
            iterator,
            next_method,
            padding: Value::undefined(),
        });
    }
    iterator_zip_finish(cx, records, mode, padding, IteratorHelperKind::Zip)
}

fn iterator_zip_keyed_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let arguments = invocation.arguments();
    let iterables = arguments.first().copied().unwrap_or(Value::undefined());
    let iterables = iterables.as_object_ref().ok_or_else(|| type_error(cx))?;
    let (mode, padding) = iterator_zip_options(cx, arguments)?;
    let keys = proxy_own_property_keys(cx, iterables)?;
    let mut records = Vec::new();
    for key in keys {
        let descriptor = match proxy_get_own_property(cx, iterables, key) {
            Ok(descriptor) => descriptor,
            Err(error) => return iterator_zip_close_collected_after_error(cx, &records, error),
        };
        let Some(descriptor) = descriptor else {
            continue;
        };
        if descriptor.enumerable() != Some(true) {
            continue;
        }
        let value = match cx.get_property_value(Value::from_object_ref(iterables), key) {
            Ok(value) => value,
            Err(error) => return iterator_zip_close_collected_after_error(cx, &records, error),
        };
        if value.is_undefined() {
            continue;
        }
        let (iterator, next_method) = match get_iterator_flattenable(cx, value) {
            Ok(record) => record,
            Err(error) => return iterator_zip_close_collected_after_error(cx, &records, error),
        };
        records.push(IteratorZipCollectedRecord {
            key: iterator_zip_key_from_property_key(key),
            iterator,
            next_method,
            padding: Value::undefined(),
        });
    }
    iterator_zip_finish(cx, records, mode, padding, IteratorHelperKind::ZipKeyed)
}

fn iterator_zip_finish<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    mut records: Vec<IteratorZipCollectedRecord>,
    mode: IteratorZipMode,
    padding: Option<ObjectRef>,
    kind: IteratorHelperKind,
) -> Result<Value, Cx::Error> {
    if mode == IteratorZipMode::Longest {
        let padding_values = if kind == IteratorHelperKind::ZipKeyed {
            iterator_zip_keyed_padding_values(cx, padding, &records)?
        } else {
            iterator_zip_padding_values(cx, padding, records.len(), &records)?
        };
        for (record, padding_value) in records.iter_mut().zip(padding_values) {
            record.padding = padding_value;
        }
    }
    let count = i32::try_from(records.len()).map_err(|_| type_error(cx))?;
    let mut slot_values = Vec::with_capacity(
        ITERATOR_HELPER_SEQUENCE_BASE_SLOT as usize
            + records.len() * ITERATOR_ZIP_RECORD_WIDTH as usize,
    );
    slot_values.extend_from_slice(&[
        Value::from_bool(false),
        Value::undefined(),
        Value::from_bool(false),
        Value::from_bool(false),
        kind.into_value(),
        Value::from_smi(count),
        mode.into_value(),
    ]);
    for record in records {
        let (key_kind, key_payload) = iterator_zip_key_to_slot_values(record.key);
        slot_values.push(key_kind);
        slot_values.push(key_payload);
        slot_values.push(Value::from_object_ref(record.iterator));
        slot_values.push(record.next_method);
        slot_values.push(Value::from_bool(true));
        slot_values.push(record.padding);
    }
    let realm = cx.builtin_realm();
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().iterator_helper_prototype())
        .ok_or_else(|| type_error(cx))?;
    let helper = allocate_iterator_object(
        cx,
        prototype,
        OrdinaryObjectData::IteratorHelper,
        slot_values.as_slice(),
    )?;
    Ok(Value::from_object_ref(helper))
}

fn iterator_zip_options<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    arguments: &[Value],
) -> Result<(IteratorZipMode, Option<ObjectRef>), Cx::Error> {
    let options = arguments.get(1).copied().unwrap_or(Value::undefined());
    let Some(options) = options.as_object_ref() else {
        if options.is_undefined() {
            return Ok((IteratorZipMode::Shortest, None));
        }
        return Err(type_error(cx));
    };
    let mode_key = property_key_from_text(cx, "mode");
    let mode_value = cx.get_property_value(Value::from_object_ref(options), mode_key)?;
    let mode = if mode_value.is_undefined() {
        IteratorZipMode::Shortest
    } else if let Some(string) = mode_value.as_string_ref() {
        match string_ref_text(cx, string)?.as_str() {
            "shortest" => IteratorZipMode::Shortest,
            "longest" => IteratorZipMode::Longest,
            "strict" => IteratorZipMode::Strict,
            _ => return Err(type_error(cx)),
        }
    } else {
        return Err(type_error(cx));
    };
    let padding = if mode == IteratorZipMode::Longest {
        let padding_key = property_key_from_text(cx, "padding");
        let padding = cx.get_property_value(Value::from_object_ref(options), padding_key)?;
        if padding.is_undefined() {
            None
        } else {
            Some(padding.as_object_ref().ok_or_else(|| type_error(cx))?)
        }
    } else {
        None
    };
    Ok((mode, padding))
}

fn iterator_map_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    iterator_lazy_callback_helper(cx, invocation, IteratorHelperKind::Map)
}

fn iterator_filter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    iterator_lazy_callback_helper(cx, invocation, IteratorHelperKind::Filter)
}

fn iterator_flat_map_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    iterator_lazy_callback_helper(cx, invocation, IteratorHelperKind::FlatMap)
}

fn iterator_take_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    iterator_limit_helper(cx, invocation, IteratorHelperKind::Take)
}

fn iterator_drop_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    iterator_limit_helper(cx, invocation, IteratorHelperKind::Drop)
}

fn iterator_dispose_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let this = invocation.this_value();
    let return_key = property_key_from_text(cx, "return");
    let return_value = cx.get_property_value(this, return_key)?;
    if return_value.is_undefined() || return_value.is_null() {
        return Ok(Value::undefined());
    }
    let return_method = cx.require_callable_object(return_value)?;
    cx.call_to_completion(return_method, this, &[])?;
    Ok(Value::undefined())
}

fn reject_async_iterator_dispose_error<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    capability: PromiseCapabilityId,
    error: Cx::Error,
) -> Result<Value, Cx::Error> {
    let Some(thrown) = cx.extract_thrown_value(error)? else {
        unreachable!("non-abrupt builtin error should propagate")
    };
    reject_async_iterator_dispose_value(cx, capability, thrown)
}

fn reject_async_iterator_dispose_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    capability: PromiseCapabilityId,
    reason: Value,
) -> Result<Value, Cx::Error> {
    let reject = promise_capability_reject(cx, capability)?;
    let _ = cx.call_to_completion(reject, Value::undefined(), &[reason])?;
    let promise = promise_capability_promise(cx, capability)?;
    Ok(Value::from_object_ref(promise))
}

fn async_iterator_dispose_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let promise_constructor = promise_default_constructor(cx)?;
    let capability = new_promise_capability(cx, promise_constructor)?;
    let promise = promise_capability_promise(cx, capability)?;
    let receiver = invocation.this_value();
    let return_key = property_key_from_text(cx, "return");
    let return_method = match cx.get_property_value(receiver, return_key) {
        Ok(method) if method.is_undefined() || method.is_null() => None,
        Ok(method) => match cx.require_callable_object(method) {
            Ok(method) => Some(method),
            Err(error) => return reject_async_iterator_dispose_error(cx, capability, error),
        },
        Err(error) => return reject_async_iterator_dispose_error(cx, capability, error),
    };
    let Some(return_method) = return_method else {
        let resolve = promise_capability_resolve(cx, capability)?;
        let _ = cx.call_to_completion(resolve, Value::undefined(), &[Value::undefined()])?;
        return Ok(Value::from_object_ref(promise));
    };
    let result = match cx.call_to_completion(return_method, receiver, &[Value::undefined()]) {
        Ok(result) => result,
        Err(error) => return reject_async_iterator_dispose_error(cx, capability, error),
    };
    let promise_resolve = match promise_resolve_method(cx, promise_constructor) {
        Ok(resolve) => resolve,
        Err(error) => return reject_async_iterator_dispose_error(cx, capability, error),
    };
    let result_wrapper = match cx.call_to_completion(
        promise_resolve,
        Value::from_object_ref(promise_constructor),
        &[result],
    ) {
        Ok(result_wrapper) => result_wrapper,
        Err(error) => return reject_async_iterator_dispose_error(cx, capability, error),
    };
    let Some(result_wrapper) = result_wrapper
        .as_object_ref()
        .filter(|object| cx.agent().promise_record(*object).is_some())
    else {
        let reason = errors::throw_type_error(cx.agent())
            .thrown_value()
            .unwrap_or(Value::undefined());
        return reject_async_iterator_dispose_value(cx, capability, reason);
    };
    perform_promise_then_with_capability(
        cx,
        result_wrapper,
        PromiseReactionHandler::PassThrough(Value::undefined()),
        PromiseReactionHandler::Thrower,
        Some(capability),
    )?;
    Ok(Value::from_object_ref(promise))
}

fn iterator_lazy_callback_helper<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: IteratorHelperKind,
) -> Result<Value, Cx::Error> {
    let object = iterator_this_object(cx, invocation.this_value())?;
    let callback_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let callback = match cx.require_callable_object(callback_value) {
        Ok(callback) => callback,
        Err(error) => {
            iterator_close_for_validation_failure(cx, object);
            return Err(error);
        }
    };
    let next_key = property_key_from_text(cx, "next");
    let next_method = cx.get_property_value(Value::from_object_ref(object), next_key)?;
    let realm = cx.builtin_realm();
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().iterator_helper_prototype())
        .ok_or_else(|| type_error(cx))?;
    let mut slot_values = vec![
        Value::from_object_ref(object),
        next_method,
        Value::from_bool(false),
        Value::from_bool(false),
        kind.into_value(),
        Value::from_object_ref(callback),
        Value::from_smi(0),
    ];
    if kind == IteratorHelperKind::FlatMap {
        slot_values.push(Value::undefined());
        slot_values.push(Value::undefined());
    }
    let helper = allocate_iterator_object(
        cx,
        prototype,
        OrdinaryObjectData::IteratorHelper,
        slot_values.as_slice(),
    )?;
    Ok(Value::from_object_ref(helper))
}

fn iterator_limit_helper<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    kind: IteratorHelperKind,
) -> Result<Value, Cx::Error> {
    let object = iterator_this_object(cx, invocation.this_value())?;
    let limit_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let limit = match iterator_limit_value(cx, limit_value) {
        Ok(limit) => limit,
        Err(error) => {
            iterator_close_for_validation_failure(cx, object);
            return Err(error);
        }
    };
    let next_key = property_key_from_text(cx, "next");
    let next_method = cx.get_property_value(Value::from_object_ref(object), next_key)?;
    let realm = cx.builtin_realm();
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().iterator_helper_prototype())
        .ok_or_else(|| type_error(cx))?;
    let slot_values = [
        Value::from_object_ref(object),
        next_method,
        Value::from_bool(false),
        Value::from_bool(false),
        kind.into_value(),
        number_value(limit),
        Value::from_smi(0),
    ];
    let helper = allocate_iterator_object(
        cx,
        prototype,
        OrdinaryObjectData::IteratorHelper,
        &slot_values,
    )?;
    Ok(Value::from_object_ref(helper))
}

fn iterator_limit_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<f64, Cx::Error> {
    let number = to_number_for_builtin(cx, value)?;
    if number.is_nan() {
        return Err(range_error(cx));
    }
    let integer = if number == 0.0 {
        0.0
    } else if number.is_finite() {
        number.trunc()
    } else {
        number
    };
    if integer < 0.0 {
        return Err(range_error(cx));
    }
    Ok(integer)
}

fn iterator_helper_next_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let helper = iterator_helper_this_object(cx, invocation.this_value())?;
    if iterator_helper_running(cx, helper)? {
        return Err(type_error(cx));
    }
    if iterator_helper_done(cx, helper)? {
        return create_iterator_result_value(cx, Value::undefined(), true);
    }
    let kind = IteratorHelperKind::from_value(iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_KIND_SLOT,
    )?)
    .ok_or_else(|| type_error(cx))?;
    set_iterator_helper_running(cx, helper, true)?;
    let result = match kind {
        IteratorHelperKind::Map => iterator_helper_map_next(cx, helper),
        IteratorHelperKind::Filter => iterator_helper_filter_next(cx, helper),
        IteratorHelperKind::Take => iterator_helper_take_next(cx, helper),
        IteratorHelperKind::Drop => iterator_helper_drop_next(cx, helper),
        IteratorHelperKind::FlatMap => iterator_helper_flat_map_next(cx, helper),
        IteratorHelperKind::Wrap => iterator_helper_wrap_next(cx, helper),
        IteratorHelperKind::Concat => iterator_helper_concat_next(cx, helper),
        IteratorHelperKind::Zip | IteratorHelperKind::ZipKeyed => {
            iterator_helper_zip_next(cx, helper, kind)
        }
    };
    set_iterator_helper_running(cx, helper, false)?;
    result
}

fn iterator_helper_return_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let helper = iterator_helper_this_object(cx, invocation.this_value())?;
    if iterator_helper_running(cx, helper)? {
        return Err(type_error(cx));
    }
    if iterator_helper_done(cx, helper)? {
        return create_iterator_result_value(cx, Value::undefined(), true);
    }
    let kind = IteratorHelperKind::from_value(iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_KIND_SLOT,
    )?)
    .ok_or_else(|| type_error(cx))?;
    if matches!(kind, IteratorHelperKind::Zip | IteratorHelperKind::ZipKeyed) {
        set_iterator_helper_done(cx, helper)?;
        let started = iterator_helper_zip_started(cx, helper)?;
        if started {
            set_iterator_helper_running(cx, helper, true)?;
        }
        let result = iterator_helper_zip_return(cx, helper);
        if started {
            set_iterator_helper_running(cx, helper, false)?;
        }
        return result;
    }
    set_iterator_helper_running(cx, helper, true)?;
    set_iterator_helper_done(cx, helper)?;
    if kind == IteratorHelperKind::Wrap {
        let result = iterator_helper_wrap_return(cx, helper);
        set_iterator_helper_running(cx, helper, false)?;
        return result;
    }
    if kind == IteratorHelperKind::Concat {
        let result = iterator_helper_concat_return(cx, helper);
        set_iterator_helper_running(cx, helper, false)?;
        return result;
    }
    if kind == IteratorHelperKind::FlatMap {
        if let Some(mut inner_record) = iterator_helper_inner_record(cx, helper)? {
            clear_iterator_helper_inner(cx, helper)?;
            let inner_close = {
                let mut bridge = BuiltinIteratorBridge { cx };
                iterator::iterator_close(
                    &mut bridge,
                    &mut inner_record,
                    Ok::<(), lyng_js_types::AbruptCompletion>(()),
                )
            };
            if let Err(error) = inner_close {
                set_iterator_helper_running(cx, helper, false)?;
                return Err(error);
            }
        }
    }
    let mut iterator_record = iterator_helper_record(cx, helper)?;
    let close_result = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::iterator_close(
            &mut bridge,
            &mut iterator_record,
            Ok::<(), lyng_js_types::AbruptCompletion>(()),
        )
    };
    let result = match close_result {
        Ok(()) => create_iterator_result_value(cx, Value::undefined(), true),
        Err(error) => Err(error),
    };
    set_iterator_helper_running(cx, helper, false)?;
    result
}

fn iterator_helper_wrap_next<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Value, Cx::Error> {
    let iterator_record = iterator_helper_record(cx, helper)?;
    let result = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::iterator_next(&mut bridge, &iterator_record, None)
    }?;
    Ok(Value::from_object_ref(result))
}

fn iterator_helper_wrap_return<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Value, Cx::Error> {
    let iterated = iterator_helper_iterated_object(cx, helper)?;
    let return_key = property_key_from_text(cx, "return");
    let return_value = cx.get_property_value(Value::from_object_ref(iterated), return_key)?;
    if return_value.is_undefined() || return_value.is_null() {
        return create_iterator_result_value(cx, Value::undefined(), true);
    }
    let return_method = cx.require_callable_object(return_value)?;
    cx.call_to_completion(return_method, Value::from_object_ref(iterated), &[])
}

fn iterator_helper_concat_next<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Value, Cx::Error> {
    loop {
        if let Some(mut iterator_record) = iterator_helper_active_record(cx, helper)? {
            let next = {
                let mut bridge = BuiltinIteratorBridge { cx };
                iterator::iterator_step(&mut bridge, &mut iterator_record)
            };
            let next = match next {
                Ok(next) => next,
                Err(error) => {
                    set_iterator_helper_done(cx, helper)?;
                    return Err(error);
                }
            };
            let Some(next) = next else {
                clear_iterator_helper_current(cx, helper)?;
                continue;
            };
            let value = {
                let mut bridge = BuiltinIteratorBridge { cx };
                iterator::iterator_value(&mut bridge, next)
            };
            let value = match value {
                Ok(value) => value,
                Err(error) => {
                    set_iterator_helper_done(cx, helper)?;
                    return Err(error);
                }
            };
            return create_iterator_result_value(cx, value, false);
        }

        let index = iterator_helper_counter(cx, helper)?;
        let count = iterator_helper_sequence_count(cx, helper)?;
        if index >= count {
            set_iterator_helper_done(cx, helper)?;
            return create_iterator_result_value(cx, Value::undefined(), true);
        }
        if let Err(error) = iterator_helper_concat_open_current(cx, helper, index) {
            set_iterator_helper_done(cx, helper)?;
            return Err(error);
        }
        set_iterator_slot_value_for_builtin(
            cx,
            helper,
            OrdinaryObjectData::IteratorHelper,
            ITERATOR_HELPER_COUNTER_SLOT,
            u64_to_value(index.saturating_add(1)),
        )?;
    }
}

fn iterator_helper_concat_return<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Value, Cx::Error> {
    if let Some(mut iterator_record) = iterator_helper_active_record(cx, helper)? {
        clear_iterator_helper_current(cx, helper)?;
        let close_result = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_close(
                &mut bridge,
                &mut iterator_record,
                Ok::<(), lyng_js_types::AbruptCompletion>(()),
            )
        };
        close_result?;
    }
    create_iterator_result_value(cx, Value::undefined(), true)
}

fn iterator_helper_zip_next<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    kind: IteratorHelperKind,
) -> Result<Value, Cx::Error> {
    let count = iterator_helper_sequence_count(cx, helper)?;
    if count == 0 {
        set_iterator_helper_done(cx, helper)?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    }
    let mode = iterator_helper_zip_mode(cx, helper)?;
    let mut values = Vec::with_capacity(usize::try_from(count).map_err(|_| type_error(cx))?);
    let mut any_done = false;
    let mut any_value = false;

    for index in 0..count {
        if !iterator_helper_zip_alive(cx, helper, index)? {
            values.push(iterator_helper_zip_padding(cx, helper, index)?);
            continue;
        }
        let mut iterator_record = match iterator_helper_zip_step_record(cx, helper, index) {
            Ok(record) => record,
            Err(error) => {
                set_iterator_helper_done(cx, helper)?;
                iterator_helper_zip_mark_dead(cx, helper, index)?;
                return iterator_helper_zip_close_all_after_error(cx, helper, error);
            }
        };
        let next = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_step(&mut bridge, &mut iterator_record)
        };
        let next = match next {
            Ok(next) => next,
            Err(error) => {
                set_iterator_helper_done(cx, helper)?;
                iterator_helper_zip_mark_dead(cx, helper, index)?;
                return iterator_helper_zip_close_all_after_error(cx, helper, error);
            }
        };
        let Some(next) = next else {
            iterator_helper_zip_mark_dead(cx, helper, index)?;
            match mode {
                IteratorZipMode::Shortest => {
                    set_iterator_helper_done(cx, helper)?;
                    iterator_helper_zip_close_all(cx, helper, None)?;
                    return create_iterator_result_value(cx, Value::undefined(), true);
                }
                IteratorZipMode::Longest => {
                    any_done = true;
                    values.push(iterator_helper_zip_padding(cx, helper, index)?);
                    continue;
                }
                IteratorZipMode::Strict => {
                    any_done = true;
                    if any_value {
                        set_iterator_helper_done(cx, helper)?;
                        let error = type_error(cx);
                        return iterator_helper_zip_close_all_after_error(cx, helper, error);
                    }
                    values.push(Value::undefined());
                    continue;
                }
            }
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let value = match value {
            Ok(value) => value,
            Err(error) => {
                set_iterator_helper_done(cx, helper)?;
                iterator_helper_zip_mark_dead(cx, helper, index)?;
                return iterator_helper_zip_close_all_after_error(cx, helper, error);
            }
        };
        if mode == IteratorZipMode::Strict && any_done {
            set_iterator_helper_done(cx, helper)?;
            let error = type_error(cx);
            return iterator_helper_zip_close_all_after_error(cx, helper, error);
        }
        any_value = true;
        values.push(value);
    }

    if mode == IteratorZipMode::Longest && !any_value {
        set_iterator_helper_done(cx, helper)?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    }
    if mode == IteratorZipMode::Strict && any_done {
        set_iterator_helper_done(cx, helper)?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    }

    let value = if kind == IteratorHelperKind::Zip {
        Value::from_object_ref(super::create_array_from_values(cx, &values)?)
    } else {
        Value::from_object_ref(iterator_helper_zip_keyed_result(cx, helper, &values)?)
    };
    iterator_helper_zip_set_started(cx, helper)?;
    create_iterator_result_value(cx, value, false)
}

fn iterator_helper_zip_return<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Value, Cx::Error> {
    iterator_helper_zip_close_all(cx, helper, None)?;
    create_iterator_result_value(cx, Value::undefined(), true)
}

fn iterator_helper_map_next<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Value, Cx::Error> {
    let mut iterator_record = iterator_helper_record(cx, helper)?;
    let next = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::iterator_step(&mut bridge, &mut iterator_record)
    };
    let next = match next {
        Ok(next) => next,
        Err(error) => {
            set_iterator_helper_done(cx, helper)?;
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    };
    let Some(next) = next else {
        set_iterator_helper_done(cx, helper)?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    };
    let value = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::iterator_value(&mut bridge, next)
    };
    let value = match value {
        Ok(value) => value,
        Err(error) => {
            set_iterator_helper_done(cx, helper)?;
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    };
    let mapper = iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_PARAM_SLOT,
    )?
    .as_object_ref()
    .ok_or_else(|| type_error(cx))?;
    let counter = iterator_helper_counter(cx, helper)?;
    let mapped =
        match cx.call_to_completion(mapper, Value::undefined(), &[value, u64_to_value(counter)]) {
            Ok(mapped) => mapped,
            Err(error) => {
                set_iterator_helper_done(cx, helper)?;
                return close_iterator_after_error(cx, &mut iterator_record, error);
            }
        };
    set_iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_COUNTER_SLOT,
        u64_to_value(counter.saturating_add(1)),
    )?;
    create_iterator_result_value(cx, mapped, false)
}

fn iterator_helper_take_next<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Value, Cx::Error> {
    let mut iterator_record = iterator_helper_record(cx, helper)?;
    let remaining = iterator_helper_limit(cx, helper)?;
    if remaining == 0.0 {
        set_iterator_helper_done(cx, helper)?;
        let close_result = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_close(
                &mut bridge,
                &mut iterator_record,
                Ok::<(), lyng_js_types::AbruptCompletion>(()),
            )
        };
        close_result?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    }
    if remaining.is_finite() {
        set_iterator_helper_limit(cx, helper, remaining - 1.0)?;
    }
    let next = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::iterator_step(&mut bridge, &mut iterator_record)
    };
    let next = match next {
        Ok(next) => next,
        Err(error) => {
            set_iterator_helper_done(cx, helper)?;
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    };
    let Some(next) = next else {
        set_iterator_helper_done(cx, helper)?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    };
    let value = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::iterator_value(&mut bridge, next)
    };
    let value = match value {
        Ok(value) => value,
        Err(error) => {
            set_iterator_helper_done(cx, helper)?;
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    };
    create_iterator_result_value(cx, value, false)
}

fn iterator_helper_drop_next<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Value, Cx::Error> {
    let mut iterator_record = iterator_helper_record(cx, helper)?;
    let mut remaining = iterator_helper_limit(cx, helper)?;
    while remaining > 0.0 {
        let next = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_step(&mut bridge, &mut iterator_record)
        };
        let next = match next {
            Ok(next) => next,
            Err(error) => {
                set_iterator_helper_done(cx, helper)?;
                return close_iterator_after_error(cx, &mut iterator_record, error);
            }
        };
        if next.is_none() {
            set_iterator_helper_done(cx, helper)?;
            return create_iterator_result_value(cx, Value::undefined(), true);
        }
        if remaining.is_finite() {
            remaining -= 1.0;
            set_iterator_helper_limit(cx, helper, remaining)?;
        }
    }
    let next = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::iterator_step(&mut bridge, &mut iterator_record)
    };
    let next = match next {
        Ok(next) => next,
        Err(error) => {
            set_iterator_helper_done(cx, helper)?;
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    };
    let Some(next) = next else {
        set_iterator_helper_done(cx, helper)?;
        return create_iterator_result_value(cx, Value::undefined(), true);
    };
    let value = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::iterator_value(&mut bridge, next)
    };
    let value = match value {
        Ok(value) => value,
        Err(error) => {
            set_iterator_helper_done(cx, helper)?;
            return close_iterator_after_error(cx, &mut iterator_record, error);
        }
    };
    create_iterator_result_value(cx, value, false)
}

fn iterator_helper_flat_map_next<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Value, Cx::Error> {
    let mut outer_record = iterator_helper_record(cx, helper)?;
    loop {
        if let Some(mut inner_record) = iterator_helper_inner_record(cx, helper)? {
            let next = {
                let mut bridge = BuiltinIteratorBridge { cx };
                iterator::iterator_step(&mut bridge, &mut inner_record)
            };
            let next = match next {
                Ok(next) => next,
                Err(error) => {
                    clear_iterator_helper_inner(cx, helper)?;
                    set_iterator_helper_done(cx, helper)?;
                    return close_iterator_after_error(cx, &mut outer_record, error);
                }
            };
            let Some(next) = next else {
                clear_iterator_helper_inner(cx, helper)?;
                continue;
            };
            let value = {
                let mut bridge = BuiltinIteratorBridge { cx };
                iterator::iterator_value(&mut bridge, next)
            };
            let value = match value {
                Ok(value) => value,
                Err(error) => {
                    clear_iterator_helper_inner(cx, helper)?;
                    set_iterator_helper_done(cx, helper)?;
                    return close_iterator_after_error(cx, &mut outer_record, error);
                }
            };
            return create_iterator_result_value(cx, value, false);
        }

        let next = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_step(&mut bridge, &mut outer_record)
        };
        let next = match next {
            Ok(next) => next,
            Err(error) => {
                set_iterator_helper_done(cx, helper)?;
                return close_iterator_after_error(cx, &mut outer_record, error);
            }
        };
        let Some(next) = next else {
            set_iterator_helper_done(cx, helper)?;
            return create_iterator_result_value(cx, Value::undefined(), true);
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let value = match value {
            Ok(value) => value,
            Err(error) => {
                set_iterator_helper_done(cx, helper)?;
                return close_iterator_after_error(cx, &mut outer_record, error);
            }
        };
        let mapper = iterator_slot_value_for_builtin(
            cx,
            helper,
            OrdinaryObjectData::IteratorHelper,
            ITERATOR_HELPER_PARAM_SLOT,
        )?
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
        let counter = iterator_helper_counter(cx, helper)?;
        let mapped = match cx.call_to_completion(
            mapper,
            Value::undefined(),
            &[value, u64_to_value(counter)],
        ) {
            Ok(mapped) => mapped,
            Err(error) => {
                set_iterator_helper_done(cx, helper)?;
                return close_iterator_after_error(cx, &mut outer_record, error);
            }
        };
        set_iterator_slot_value_for_builtin(
            cx,
            helper,
            OrdinaryObjectData::IteratorHelper,
            ITERATOR_HELPER_COUNTER_SLOT,
            u64_to_value(counter.saturating_add(1)),
        )?;
        let (inner, inner_next) = match get_iterator_flattenable(cx, mapped) {
            Ok(record) => record,
            Err(error) => {
                set_iterator_helper_done(cx, helper)?;
                return close_iterator_after_error(cx, &mut outer_record, error);
            }
        };
        set_iterator_slot_value_for_builtin(
            cx,
            helper,
            OrdinaryObjectData::IteratorHelper,
            ITERATOR_HELPER_INNER_ITERATED_SLOT,
            Value::from_object_ref(inner),
        )?;
        set_iterator_slot_value_for_builtin(
            cx,
            helper,
            OrdinaryObjectData::IteratorHelper,
            ITERATOR_HELPER_INNER_NEXT_METHOD_SLOT,
            inner_next,
        )?;
    }
}

fn iterator_helper_filter_next<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Value, Cx::Error> {
    let mut iterator_record = iterator_helper_record(cx, helper)?;
    loop {
        let next = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_step(&mut bridge, &mut iterator_record)
        };
        let next = match next {
            Ok(next) => next,
            Err(error) => {
                set_iterator_helper_done(cx, helper)?;
                return close_iterator_after_error(cx, &mut iterator_record, error);
            }
        };
        let Some(next) = next else {
            set_iterator_helper_done(cx, helper)?;
            return create_iterator_result_value(cx, Value::undefined(), true);
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let value = match value {
            Ok(value) => value,
            Err(error) => {
                set_iterator_helper_done(cx, helper)?;
                return close_iterator_after_error(cx, &mut iterator_record, error);
            }
        };
        let predicate = iterator_slot_value_for_builtin(
            cx,
            helper,
            OrdinaryObjectData::IteratorHelper,
            ITERATOR_HELPER_PARAM_SLOT,
        )?
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
        let counter = iterator_helper_counter(cx, helper)?;
        let selected = match cx.call_to_completion(
            predicate,
            Value::undefined(),
            &[value, u64_to_value(counter)],
        ) {
            Ok(selected) => selected,
            Err(error) => {
                set_iterator_helper_done(cx, helper)?;
                return close_iterator_after_error(cx, &mut iterator_record, error);
            }
        };
        set_iterator_slot_value_for_builtin(
            cx,
            helper,
            OrdinaryObjectData::IteratorHelper,
            ITERATOR_HELPER_COUNTER_SLOT,
            u64_to_value(counter.saturating_add(1)),
        )?;
        let selected = {
            let completion = {
                let agent = cx.agent();
                read::to_boolean(agent.heap().view(), selected)
            };
            map_completion(cx, completion)?
        };
        if selected {
            return create_iterator_result_value(cx, value, false);
        }
    }
}

fn get_iterator_flattenable<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<(ObjectRef, Value), Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let iterator_symbol = cx
        .agent()
        .well_known_symbol(WellKnownSymbolId::Iterator)
        .ok_or_else(|| type_error(cx))?;
    let method = cx.get_property_value(
        Value::from_object_ref(object),
        PropertyKey::from_symbol(iterator_symbol),
    )?;
    let iterator = if method.is_undefined() || method.is_null() {
        object
    } else {
        let method = cx.require_callable_object(method)?;
        let iterator = cx.call_to_completion(method, Value::from_object_ref(object), &[])?;
        iterator.as_object_ref().ok_or_else(|| type_error(cx))?
    };
    let next_key = property_key_from_text(cx, "next");
    let next = cx.get_property_value(Value::from_object_ref(iterator), next_key)?;
    Ok((iterator, next))
}

fn iterator_helper_this_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let object = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let _ = iterator_slot_value_for_builtin(
        cx,
        object,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_KIND_SLOT,
    )?;
    Ok(object)
}

fn iterator_helper_inner_record<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Option<iterator::IteratorRecord>, Cx::Error> {
    let inner = iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_INNER_ITERATED_SLOT,
    )?;
    let Some(inner) = inner.as_object_ref() else {
        return Ok(None);
    };
    let next = iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_INNER_NEXT_METHOD_SLOT,
    )?;
    let next = cx.require_callable_object(next)?;
    Ok(Some(iterator::IteratorRecord::new(inner, next)))
}

fn clear_iterator_helper_inner<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<(), Cx::Error> {
    set_iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_INNER_ITERATED_SLOT,
        Value::undefined(),
    )?;
    set_iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_INNER_NEXT_METHOD_SLOT,
        Value::undefined(),
    )
}

fn iterator_helper_record<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<iterator::IteratorRecord, Cx::Error> {
    let iterated = iterator_helper_iterated_object(cx, helper)?;
    let next_value = iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_NEXT_METHOD_SLOT,
    )?;
    let next_method = cx.require_callable_object(next_value)?;
    Ok(iterator::IteratorRecord::new(iterated, next_method))
}

fn iterator_helper_active_record<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<Option<iterator::IteratorRecord>, Cx::Error> {
    let iterated = iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_ITERATED_SLOT,
    )?;
    let Some(iterated) = iterated.as_object_ref() else {
        return Ok(None);
    };
    let next_value = iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_NEXT_METHOD_SLOT,
    )?;
    let next_method = cx.require_callable_object(next_value)?;
    Ok(Some(iterator::IteratorRecord::new(iterated, next_method)))
}

fn iterator_helper_iterated_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<ObjectRef, Cx::Error> {
    iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_ITERATED_SLOT,
    )?
    .as_object_ref()
    .ok_or_else(|| type_error(cx))
}

fn iterator_helper_done<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<bool, Cx::Error> {
    iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_DONE_SLOT,
    )?
    .as_bool()
    .ok_or_else(|| type_error(cx))
}

fn set_iterator_helper_done<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<(), Cx::Error> {
    set_iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_DONE_SLOT,
        Value::from_bool(true),
    )
}

fn iterator_helper_running<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<bool, Cx::Error> {
    iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_RUNNING_SLOT,
    )?
    .as_bool()
    .ok_or_else(|| type_error(cx))
}

fn set_iterator_helper_running<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    running: bool,
) -> Result<(), Cx::Error> {
    set_iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_RUNNING_SLOT,
        Value::from_bool(running),
    )
}

fn iterator_helper_counter<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<u64, Cx::Error> {
    iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_COUNTER_SLOT,
    )?
    .as_smi()
    .and_then(|counter| u64::try_from(counter).ok())
    .ok_or_else(|| type_error(cx))
}

fn iterator_helper_limit<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<f64, Cx::Error> {
    iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_PARAM_SLOT,
    )?
    .as_f64()
    .ok_or_else(|| type_error(cx))
}

fn set_iterator_helper_limit<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    value: f64,
) -> Result<(), Cx::Error> {
    set_iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_PARAM_SLOT,
        number_value(value),
    )
}

fn iterator_helper_sequence_count<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<u64, Cx::Error> {
    iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_PARAM_SLOT,
    )?
    .as_smi()
    .and_then(|count| u64::try_from(count).ok())
    .ok_or_else(|| type_error(cx))
}

fn iterator_zip_padding_values<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    padding: Option<ObjectRef>,
    count: usize,
    records: &[IteratorZipCollectedRecord],
) -> Result<Vec<Value>, Cx::Error> {
    let Some(padding) = padding else {
        return Ok(vec![Value::undefined(); count]);
    };
    let mut padding_record = {
        let mut bridge = BuiltinIteratorBridge { cx };
        match iterator::get_iterator(&mut bridge, Value::from_object_ref(padding)) {
            Ok(record) => record,
            Err(error) => return iterator_zip_close_collected_after_error(cx, records, error),
        }
    };
    let mut using_iterator = true;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        if !using_iterator {
            values.push(Value::undefined());
            continue;
        }
        let next = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_step(&mut bridge, &mut padding_record)
        };
        let next = match next {
            Ok(next) => next,
            Err(error) => return iterator_zip_close_collected_after_error(cx, records, error),
        };
        let Some(next) = next else {
            using_iterator = false;
            values.push(Value::undefined());
            continue;
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        match value {
            Ok(value) => values.push(value),
            Err(error) => return iterator_zip_close_collected_after_error(cx, records, error),
        }
    }
    if using_iterator {
        let thrown = iterator_zip_close_iterator_record(cx, &mut padding_record, None)?;
        if let Some(thrown) = thrown {
            let thrown = iterator_zip_close_collected_with_thrown(cx, records, Some(thrown))?
                .expect("throw completion should be preserved");
            return Err(cx.abrupt(AbruptCompletion::throw(thrown)));
        }
    }
    Ok(values)
}

fn iterator_zip_keyed_padding_values<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    padding: Option<ObjectRef>,
    records: &[IteratorZipCollectedRecord],
) -> Result<Vec<Value>, Cx::Error> {
    let Some(padding) = padding else {
        return Ok(vec![Value::undefined(); records.len()]);
    };
    let mut values = Vec::with_capacity(records.len());
    for record in records {
        let key = iterator_zip_collected_key_to_property_key(cx, record.key)?;
        let value = match cx.get_property_value(Value::from_object_ref(padding), key) {
            Ok(value) => value,
            Err(error) => return iterator_zip_close_collected_after_error(cx, records, error),
        };
        values.push(value);
    }
    Ok(values)
}

fn iterator_zip_close_collected_after_error<Cx: PublicBuiltinDispatchContext, T>(
    cx: &mut Cx,
    records: &[IteratorZipCollectedRecord],
    error: Cx::Error,
) -> Result<T, Cx::Error> {
    let thrown = iterator_zip_thrown_value(cx, error)?;
    let thrown = iterator_zip_close_collected_with_thrown(cx, records, Some(thrown))?
        .expect("throw completion should be preserved");
    Err(cx.abrupt(AbruptCompletion::throw(thrown)))
}

fn iterator_zip_close_collected_and_input_after_error<Cx: PublicBuiltinDispatchContext, T>(
    cx: &mut Cx,
    records: &[IteratorZipCollectedRecord],
    input_record: &mut iterator::IteratorRecord,
    error: Cx::Error,
) -> Result<T, Cx::Error> {
    let thrown = iterator_zip_thrown_value(cx, error)?;
    let thrown = iterator_zip_close_collected_with_thrown(cx, records, Some(thrown))?
        .expect("throw completion should be preserved");
    let thrown = iterator_zip_close_iterator_record(cx, input_record, Some(thrown))?
        .expect("throw completion should be preserved");
    Err(cx.abrupt(AbruptCompletion::throw(thrown)))
}

fn iterator_zip_close_collected_with_thrown<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    records: &[IteratorZipCollectedRecord],
    mut thrown: Option<Value>,
) -> Result<Option<Value>, Cx::Error> {
    for record in records.iter().rev() {
        thrown =
            iterator_zip_close_iterator_value(cx, record.iterator, record.next_method, thrown)?;
    }
    Ok(thrown)
}

fn iterator_helper_zip_close_all_after_error<Cx: PublicBuiltinDispatchContext, T>(
    cx: &mut Cx,
    helper: ObjectRef,
    error: Cx::Error,
) -> Result<T, Cx::Error> {
    let thrown = iterator_zip_thrown_value(cx, error)?;
    let thrown = iterator_helper_zip_close_all_with_thrown(cx, helper, Some(thrown))?
        .expect("throw completion should be preserved");
    Err(cx.abrupt(AbruptCompletion::throw(thrown)))
}

fn iterator_helper_zip_close_all<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    thrown: Option<Value>,
) -> Result<(), Cx::Error> {
    if let Some(thrown) = iterator_helper_zip_close_all_with_thrown(cx, helper, thrown)? {
        return Err(cx.abrupt(AbruptCompletion::throw(thrown)));
    }
    Ok(())
}

fn iterator_helper_zip_close_all_with_thrown<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    mut thrown: Option<Value>,
) -> Result<Option<Value>, Cx::Error> {
    let count = iterator_helper_sequence_count(cx, helper)?;
    for index in (0..count).rev() {
        if !iterator_helper_zip_alive(cx, helper, index)? {
            continue;
        }
        let iterator = iterator_helper_zip_iterator(cx, helper, index)?;
        let next_method = iterator_helper_zip_next_method(cx, helper, index)?;
        iterator_helper_zip_mark_dead(cx, helper, index)?;
        thrown = iterator_zip_close_iterator_value(cx, iterator, next_method, thrown)?;
    }
    Ok(thrown)
}

fn iterator_zip_close_iterator_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    iterator: ObjectRef,
    next_method: Value,
    thrown: Option<Value>,
) -> Result<Option<Value>, Cx::Error> {
    let next_method = next_method
        .as_object_ref()
        .unwrap_or_else(|| cx.callee_object());
    let mut record = iterator::IteratorRecord::new(iterator, next_method);
    iterator_zip_close_iterator_record(cx, &mut record, thrown)
}

fn iterator_zip_close_iterator_record<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    record: &mut iterator::IteratorRecord,
    thrown: Option<Value>,
) -> Result<Option<Value>, Cx::Error> {
    let completion = match thrown {
        Some(thrown) => Err(AbruptCompletion::throw(thrown)),
        None => Ok(()),
    };
    let close = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::iterator_close(&mut bridge, record, completion)
    };
    match close {
        Ok(()) => Ok(None),
        Err(error) => iterator_zip_thrown_value(cx, error).map(Some),
    }
}

fn iterator_zip_thrown_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    error: Cx::Error,
) -> Result<Value, Cx::Error> {
    let Some(thrown) = cx.extract_thrown_value(error)? else {
        unreachable!("non-abrupt builtin error should propagate");
    };
    Ok(thrown)
}

fn iterator_helper_zip_mode<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<IteratorZipMode, Cx::Error> {
    IteratorZipMode::from_value(iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_COUNTER_SLOT,
    )?)
    .ok_or_else(|| type_error(cx))
}

fn iterator_helper_zip_started<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<bool, Cx::Error> {
    iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_ITERATED_SLOT,
    )?
    .as_bool()
    .ok_or_else(|| type_error(cx))
}

fn iterator_helper_zip_set_started<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<(), Cx::Error> {
    set_iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_ITERATED_SLOT,
        Value::from_bool(true),
    )
}

fn iterator_helper_zip_record_slot<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    index: u64,
    offset: u32,
) -> Result<u32, Cx::Error> {
    ITERATOR_HELPER_SEQUENCE_BASE_SLOT
        .checked_add(
            u32::try_from(index.saturating_mul(u64::from(ITERATOR_ZIP_RECORD_WIDTH)))
                .map_err(|_| type_error(cx))?,
        )
        .and_then(|base| base.checked_add(offset))
        .ok_or_else(|| type_error(cx))
}

fn iterator_helper_zip_slot_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    index: u64,
    offset: u32,
) -> Result<Value, Cx::Error> {
    let slot = iterator_helper_zip_record_slot(cx, index, offset)?;
    iterator_slot_value_for_builtin(cx, helper, OrdinaryObjectData::IteratorHelper, slot)
}

fn iterator_helper_zip_set_slot_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    index: u64,
    offset: u32,
    value: Value,
) -> Result<(), Cx::Error> {
    let slot = iterator_helper_zip_record_slot(cx, index, offset)?;
    set_iterator_slot_value_for_builtin(cx, helper, OrdinaryObjectData::IteratorHelper, slot, value)
}

fn iterator_helper_zip_alive<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    index: u64,
) -> Result<bool, Cx::Error> {
    iterator_helper_zip_slot_value(cx, helper, index, ITERATOR_ZIP_ALIVE_OFFSET)?
        .as_bool()
        .ok_or_else(|| type_error(cx))
}

fn iterator_helper_zip_mark_dead<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    index: u64,
) -> Result<(), Cx::Error> {
    iterator_helper_zip_set_slot_value(
        cx,
        helper,
        index,
        ITERATOR_ZIP_ALIVE_OFFSET,
        Value::from_bool(false),
    )
}

fn iterator_helper_zip_iterator<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    index: u64,
) -> Result<ObjectRef, Cx::Error> {
    iterator_helper_zip_slot_value(cx, helper, index, ITERATOR_ZIP_ITERATED_OFFSET)?
        .as_object_ref()
        .ok_or_else(|| type_error(cx))
}

fn iterator_helper_zip_next_method<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    index: u64,
) -> Result<Value, Cx::Error> {
    iterator_helper_zip_slot_value(cx, helper, index, ITERATOR_ZIP_NEXT_METHOD_OFFSET)
}

fn iterator_helper_zip_step_record<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    index: u64,
) -> Result<iterator::IteratorRecord, Cx::Error> {
    let iterator = iterator_helper_zip_iterator(cx, helper, index)?;
    let next_method = iterator_helper_zip_next_method(cx, helper, index)?;
    let next_method = cx.require_callable_object(next_method)?;
    Ok(iterator::IteratorRecord::new(iterator, next_method))
}

fn iterator_helper_zip_padding<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    index: u64,
) -> Result<Value, Cx::Error> {
    iterator_helper_zip_slot_value(cx, helper, index, ITERATOR_ZIP_PADDING_OFFSET)
}

fn iterator_helper_zip_keyed_result<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    values: &[Value],
) -> Result<ObjectRef, Cx::Error> {
    let object = cx.allocate_ordinary_object_with_prototype(cx.builtin_realm(), None)?;
    for (index, value) in values.iter().copied().enumerate() {
        let key = iterator_helper_zip_key(cx, helper, index as u64)?;
        create_data_property_or_throw(cx, object, key, value)?;
    }
    Ok(object)
}

fn iterator_helper_zip_key<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    index: u64,
) -> Result<PropertyKey, Cx::Error> {
    let kind = iterator_helper_zip_slot_value(cx, helper, index, ITERATOR_ZIP_KEY_KIND_OFFSET)?;
    let payload =
        iterator_helper_zip_slot_value(cx, helper, index, ITERATOR_ZIP_KEY_PAYLOAD_OFFSET)?;
    iterator_zip_key_from_slot_values(cx, kind, payload)
}

fn iterator_zip_key_from_property_key(key: PropertyKey) -> IteratorZipKey {
    match key {
        PropertyKey::Index(index) => IteratorZipKey::Index(index),
        PropertyKey::Atom(atom) => IteratorZipKey::Atom(atom),
        PropertyKey::Symbol(symbol) => IteratorZipKey::Symbol(symbol),
    }
}

fn iterator_zip_collected_key_to_property_key<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    key: IteratorZipKey,
) -> Result<PropertyKey, Cx::Error> {
    match key {
        IteratorZipKey::None => Err(type_error(cx)),
        IteratorZipKey::Index(index) => Ok(PropertyKey::Index(index)),
        IteratorZipKey::Atom(atom) => Ok(PropertyKey::Atom(atom)),
        IteratorZipKey::Symbol(symbol) => Ok(PropertyKey::Symbol(symbol)),
    }
}

fn iterator_zip_key_to_slot_values(key: IteratorZipKey) -> (Value, Value) {
    match key {
        IteratorZipKey::None => (Value::from_smi(0), Value::undefined()),
        IteratorZipKey::Index(index) => (Value::from_smi(1), u64_to_value(u64::from(index))),
        IteratorZipKey::Atom(atom) => (Value::from_smi(2), u64_to_value(u64::from(atom.raw()))),
        IteratorZipKey::Symbol(symbol) => (Value::from_smi(3), Value::from_symbol_ref(symbol)),
    }
}

fn iterator_zip_key_from_slot_values<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    kind: Value,
    payload: Value,
) -> Result<PropertyKey, Cx::Error> {
    match kind.as_smi() {
        Some(1) => Ok(PropertyKey::Index(iterator_zip_u32_payload(cx, payload)?)),
        Some(2) => Ok(PropertyKey::Atom(AtomId::from_raw(
            iterator_zip_u32_payload(cx, payload)?,
        ))),
        Some(3) => payload
            .as_symbol_ref()
            .map(PropertyKey::from_symbol)
            .ok_or_else(|| type_error(cx)),
        _ => Err(type_error(cx)),
    }
}

fn iterator_zip_u32_payload<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<u32, Cx::Error> {
    if let Some(value) = value.as_smi() {
        return u32::try_from(value).map_err(|_| type_error(cx));
    }
    let number = value.as_f64().ok_or_else(|| type_error(cx))?;
    if !number.is_finite() || number < 0.0 || number.trunc() != number || number > u32::MAX as f64 {
        return Err(type_error(cx));
    }
    Ok(number as u32)
}

fn iterator_helper_concat_open_current<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
    index: u64,
) -> Result<(), Cx::Error> {
    let slot_base = ITERATOR_HELPER_SEQUENCE_BASE_SLOT
        .checked_add(u32::try_from(index.saturating_mul(2)).map_err(|_| type_error(cx))?)
        .ok_or_else(|| type_error(cx))?;
    let iterable =
        iterator_slot_value_for_builtin(cx, helper, OrdinaryObjectData::IteratorHelper, slot_base)?
            .as_object_ref()
            .ok_or_else(|| type_error(cx))?;
    let method = iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        slot_base.saturating_add(1),
    )?
    .as_object_ref()
    .ok_or_else(|| type_error(cx))?;
    let iterator = cx.call_to_completion(method, Value::from_object_ref(iterable), &[])?;
    let iterator = iterator.as_object_ref().ok_or_else(|| type_error(cx))?;
    let next_key = property_key_from_text(cx, "next");
    let next = cx.get_property_value(Value::from_object_ref(iterator), next_key)?;
    set_iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_ITERATED_SLOT,
        Value::from_object_ref(iterator),
    )?;
    set_iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_NEXT_METHOD_SLOT,
        next,
    )
}

fn clear_iterator_helper_current<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    helper: ObjectRef,
) -> Result<(), Cx::Error> {
    set_iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_ITERATED_SLOT,
        Value::undefined(),
    )?;
    set_iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_NEXT_METHOD_SLOT,
        Value::undefined(),
    )
}

fn iterator_from_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let argument = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let realm = cx.builtin_realm();
    let iterator_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().iterator_prototype())
        .ok_or_else(|| type_error(cx))?;
    let (iterator, next_method) = get_iterator_flattenable_for_iterator_from(cx, argument)?;
    if iterator_prototype_in_chain(cx, iterator, iterator_prototype)? {
        return Ok(Value::from_object_ref(iterator));
    }
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().iterator_helper_prototype())
        .ok_or_else(|| type_error(cx))?;
    let slot_values = [
        Value::from_object_ref(iterator),
        next_method,
        Value::from_bool(false),
        Value::from_bool(false),
        IteratorHelperKind::Wrap.into_value(),
        Value::undefined(),
        Value::from_smi(0),
    ];
    let wrapper = allocate_iterator_object(
        cx,
        prototype,
        OrdinaryObjectData::IteratorHelper,
        &slot_values,
    )?;
    Ok(Value::from_object_ref(wrapper))
}

fn get_iterator_flattenable_for_iterator_from<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<(ObjectRef, Value), Cx::Error> {
    if value.as_object_ref().is_none() && !value.is_string() {
        return Err(type_error(cx));
    }
    let iterator_symbol = cx
        .agent()
        .well_known_symbol(WellKnownSymbolId::Iterator)
        .ok_or_else(|| type_error(cx))?;
    let method = cx.get_property_value(value, PropertyKey::from_symbol(iterator_symbol))?;
    let iterator = if method.is_undefined() || method.is_null() {
        value.as_object_ref().ok_or_else(|| type_error(cx))?
    } else {
        let method = cx.require_callable_object(method)?;
        let iterator = cx.call_to_completion(method, value, &[])?;
        iterator.as_object_ref().ok_or_else(|| type_error(cx))?
    };
    let next_key = property_key_from_text(cx, "next");
    let next = cx.get_property_value(Value::from_object_ref(iterator), next_key)?;
    Ok((iterator, next))
}

fn iterator_prototype_in_chain<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    start: lyng_js_types::ObjectRef,
    target_prototype: lyng_js_types::ObjectRef,
) -> Result<bool, Cx::Error> {
    let mut current = Some(start);
    let mut steps = 0_u32;
    while let Some(object) = current {
        if object == target_prototype {
            return Ok(true);
        }
        // Cap traversal to prevent runaway proxy traps from misbehaving;
        // the spec never requires more than a finite chain.
        steps = steps.saturating_add(1);
        if steps > 1024 {
            break;
        }
        let parent = {
            let agent = cx.agent();
            agent
                .objects()
                .get_prototype_of(agent.heap().view(), object)
        };
        let next = match parent {
            Ok(Some(parent_object)) => Some(parent_object),
            Ok(None) => None,
            Err(_) => return Err(type_error(cx)),
        };
        current = next;
    }
    Ok(false)
}

#[inline]
fn u64_to_value(value: u64) -> Value {
    if let Ok(small) = i32::try_from(value) {
        Value::from_smi(small)
    } else {
        Value::from_f64(value as f64)
    }
}
