use super::{
    array_like_length, binary_data::typed_array_validated_object_and_record,
    close_iterator_after_error, create_array_result, get_property_from_object, length_value,
    map_completion, number_value, property_key_from_text, range_error, set_property_on_object,
    string_from_code_units, string_ref_code_units, string_this_ref, to_number_for_builtin,
    type_error, BuiltinIteratorBridge, PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_common::WellKnownAtom;
use lyng_js_env::Agent;
use lyng_js_gc::AllocationLifetime;
use lyng_js_objects::{ObjectAllocation, ObjectColdData, OrdinaryObjectData};
use lyng_js_ops::{iterator, read};
use lyng_js_types::{BuiltinFunctionId, ObjectRef, PropertyKey, Value, WellKnownSymbolId};

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
            _ => None,
        }
    }

    #[inline]
    const fn into_value(self) -> Value {
        Value::from_smi(self as i32)
    }
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
    let length = array_like_length(cx, target_object)?;
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
// map/filter/take/drop/flatMap, and Iterator.from when the input already
// inherits from %Iterator.prototype% (i.e. the fast-path branch of the spec's
// GetIteratorFlattenable). Wrapped-iterator support for arbitrary input
// and the static helper constructors are tracked by dcat issue lyng-3k8k.

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
    if iterator_helper_done(cx, helper)? {
        return create_iterator_result_value(cx, Value::undefined(), true);
    }
    if iterator_helper_running(cx, helper)? {
        return Err(type_error(cx));
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
    };
    set_iterator_helper_running(cx, helper, false)?;
    result
}

fn iterator_helper_return_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let helper = iterator_helper_this_object(cx, invocation.this_value())?;
    if iterator_helper_done(cx, helper)? {
        return create_iterator_result_value(cx, Value::undefined(), true);
    }
    if iterator_helper_running(cx, helper)? {
        return Err(type_error(cx));
    }
    set_iterator_helper_running(cx, helper, true)?;
    set_iterator_helper_done(cx, helper)?;
    let kind = IteratorHelperKind::from_value(iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_KIND_SLOT,
    )?)
    .ok_or_else(|| type_error(cx))?;
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
    let iterated = iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_ITERATED_SLOT,
    )?
    .as_object_ref()
    .ok_or_else(|| type_error(cx))?;
    let next_value = iterator_slot_value_for_builtin(
        cx,
        helper,
        OrdinaryObjectData::IteratorHelper,
        ITERATOR_HELPER_NEXT_METHOD_SLOT,
    )?;
    let next_method = cx.require_callable_object(next_value)?;
    Ok(iterator::IteratorRecord::new(iterated, next_method))
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

fn iterator_from_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    // Stage 1 covers the fast path of GetIteratorFlattenable: when O is
    // already an object whose [[Prototype]] chain includes %Iterator.prototype%,
    // return it as-is. The wrapping branch (WrapForValidIteratorPrototype) is
    // deferred to Stage 2 along with the lazy helpers; calling Iterator.from
    // with a non-Iterator-prototype iterable currently throws TypeError so
    // the gap is observable rather than silently incorrect.
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
    let object = argument.as_object_ref().ok_or_else(|| type_error(cx))?;
    if iterator_prototype_in_chain(cx, object, iterator_prototype)? {
        return Ok(Value::from_object_ref(object));
    }
    Err(type_error(cx))
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
