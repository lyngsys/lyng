use super::{
    iterator, iterator_slot_value_for_builtin, length_value_u64, number_value,
    property_key_from_text, set_iterator_slot_value_for_builtin, type_error, ArrayIterationKind,
    AtomId, ObjectRef, OrdinaryObjectData, PropertyKey, PublicBuiltinDispatchContext, Value,
    WellKnownSymbolId,
};

pub(super) const ITERATOR_HELPER_ITERATED_SLOT: u32 = 0;
pub(super) const ITERATOR_HELPER_NEXT_METHOD_SLOT: u32 = 1;
pub(super) const ITERATOR_HELPER_DONE_SLOT: u32 = 2;
pub(super) const ITERATOR_HELPER_RUNNING_SLOT: u32 = 3;
pub(super) const ITERATOR_HELPER_KIND_SLOT: u32 = 4;
pub(super) const ITERATOR_HELPER_PARAM_SLOT: u32 = 5;
pub(super) const ITERATOR_HELPER_COUNTER_SLOT: u32 = 6;
pub(super) const ITERATOR_HELPER_INNER_ITERATED_SLOT: u32 = 7;
pub(super) const ITERATOR_HELPER_INNER_NEXT_METHOD_SLOT: u32 = 8;
pub(super) const ITERATOR_HELPER_SEQUENCE_BASE_SLOT: u32 = 7;
pub(super) const ITERATOR_ZIP_RECORD_WIDTH: u32 = 6;
pub(super) const ITERATOR_ZIP_KEY_KIND_OFFSET: u32 = 0;
pub(super) const ITERATOR_ZIP_KEY_PAYLOAD_OFFSET: u32 = 1;
pub(super) const ITERATOR_ZIP_ITERATED_OFFSET: u32 = 2;
pub(super) const ITERATOR_ZIP_NEXT_METHOD_OFFSET: u32 = 3;
pub(super) const ITERATOR_ZIP_ALIVE_OFFSET: u32 = 4;
pub(super) const ITERATOR_ZIP_PADDING_OFFSET: u32 = 5;

impl ArrayIterationKind {
    #[inline]
    pub(in crate::public::dispatch) const fn from_value(value: Value) -> Option<Self> {
        match value.as_smi() {
            Some(0) => Some(Self::Key),
            Some(1) => Some(Self::Value),
            Some(2) => Some(Self::Entry),
            _ => None,
        }
    }

    #[inline]
    pub(in crate::public::dispatch) const fn into_value(self) -> Value {
        Value::from_smi(self as i32)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum IteratorHelperKind {
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
    pub(super) const fn from_value(value: Value) -> Option<Self> {
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
    pub(super) const fn into_value(self) -> Value {
        Value::from_smi(self as i32)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum IteratorZipMode {
    Shortest = 0,
    Longest = 1,
    Strict = 2,
}

impl IteratorZipMode {
    #[inline]
    pub(super) const fn from_value(value: Value) -> Option<Self> {
        match value.as_smi() {
            Some(0) => Some(Self::Shortest),
            Some(1) => Some(Self::Longest),
            Some(2) => Some(Self::Strict),
            _ => None,
        }
    }

    #[inline]
    pub(super) const fn into_value(self) -> Value {
        Value::from_smi(self as i32)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum IteratorZipKey {
    None,
    Index(u32),
    Atom(AtomId),
    Symbol(lyng_js_types::SymbolRef),
}

#[derive(Clone, Copy, Debug)]
pub(super) struct IteratorZipCollectedRecord {
    pub(super) key: IteratorZipKey,
    pub(super) iterator: ObjectRef,
    pub(super) next_method: Value,
    pub(super) padding: Value,
}
pub(super) fn get_iterator_flattenable<Cx: PublicBuiltinDispatchContext>(
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

pub(super) fn iterator_helper_this_object<Cx: PublicBuiltinDispatchContext>(
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

pub(super) fn iterator_helper_inner_record<Cx: PublicBuiltinDispatchContext>(
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

pub(super) fn clear_iterator_helper_inner<Cx: PublicBuiltinDispatchContext>(
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

pub(super) fn iterator_helper_record<Cx: PublicBuiltinDispatchContext>(
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

pub(super) fn iterator_helper_active_record<Cx: PublicBuiltinDispatchContext>(
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

pub(super) fn iterator_helper_iterated_object<Cx: PublicBuiltinDispatchContext>(
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

pub(super) fn iterator_helper_done<Cx: PublicBuiltinDispatchContext>(
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

pub(super) fn set_iterator_helper_done<Cx: PublicBuiltinDispatchContext>(
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

pub(super) fn iterator_helper_running<Cx: PublicBuiltinDispatchContext>(
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

pub(super) fn set_iterator_helper_running<Cx: PublicBuiltinDispatchContext>(
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

pub(super) fn iterator_helper_counter<Cx: PublicBuiltinDispatchContext>(
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

pub(super) fn iterator_helper_limit<Cx: PublicBuiltinDispatchContext>(
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

pub(super) fn set_iterator_helper_limit<Cx: PublicBuiltinDispatchContext>(
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

pub(super) fn iterator_helper_sequence_count<Cx: PublicBuiltinDispatchContext>(
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
pub(super) fn u64_to_value(value: u64) -> Value {
    length_value_u64(value)
}
