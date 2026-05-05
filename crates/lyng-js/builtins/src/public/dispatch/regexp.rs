mod accessors;
mod construction;
mod escape;
mod symbols;

use super::{
    allocate_array_like_result, append_string_ref_code_units, builtin_function_entry,
    callable_object_from_value, code_unit_range_value, define_data_property_with_attrs, iterators,
    set_data_property_value, string_from_code_units, string_from_string_ref_range,
    string_ref_code_units, string_value, syntax_error, to_boolean_for_builtin,
    to_integer_or_infinity_for_builtin, to_length_for_builtin, to_string_string_ref, type_error,
    usize_index_value, with_string_ref_code_units, PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use accessors::{
    regexp_flag_getter_builtin, regexp_flags_getter_builtin, regexp_has_indices_getter_builtin,
    regexp_source_getter_builtin,
};
use construction::{
    normalize_regexp_constructor_pattern_text, regexp_builtin, regexp_species_getter_builtin,
};
use escape::regexp_escape_builtin;
use lyng_js_common::WellKnownAtom;
use lyng_js_env::RegExpLegacyStaticText;
use lyng_js_gc::AllocationLifetime;
use lyng_js_objects::{ObjectAllocation, ObjectColdData, OrdinaryObjectData};
use lyng_js_parser::{validate_regexp_constructor_pattern, validate_regexp_literal};
use lyng_js_types::{
    BuiltinFunctionId, ObjectRef, PropertyKey, RealmRef, StringRef, Value, WellKnownSymbolId,
};
use symbols::dispatch_regexp_symbol_builtin;

pub(super) fn dispatch_regexp_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_regexp_constructor_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_regexp_prototype_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if entry == super::regexp_string_iterator_next_builtin() {
        return regexp_string_iterator_next_builtin(context, invocation).map(Some);
    }
    dispatch_regexp_symbol_builtin(context, entry, invocation)
}

fn dispatch_regexp_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::regexp_builtin() {
        return regexp_builtin(context, invocation).map(Some);
    }
    if entry == super::regexp_escape_builtin() {
        return regexp_escape_builtin(context, invocation).map(Some);
    }
    if let Some(property) = legacy_static_property_for_getter(entry) {
        return regexp_legacy_static_getter_builtin(context, invocation, property).map(Some);
    }
    if entry == super::regexp_legacy_input_setter_builtin() {
        return regexp_legacy_input_setter_builtin(context, invocation).map(Some);
    }
    if entry == super::regexp_species_getter_builtin() {
        return Ok(Some(regexp_species_getter_builtin(invocation)));
    }
    Ok(None)
}

fn dispatch_regexp_prototype_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::regexp_to_string_builtin() {
        return regexp_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::regexp_compile_builtin() {
        return regexp_compile_builtin(context, invocation).map(Some);
    }
    if entry == super::regexp_exec_builtin() {
        return regexp_exec_builtin(context, invocation).map(Some);
    }
    if entry == super::regexp_test_builtin() {
        return regexp_test_builtin(context, invocation).map(Some);
    }
    if entry == super::regexp_global_getter_builtin() {
        return regexp_flag_getter_builtin(context, invocation, 'g').map(Some);
    }
    if entry == super::regexp_ignore_case_getter_builtin() {
        return regexp_flag_getter_builtin(context, invocation, 'i').map(Some);
    }
    if entry == super::regexp_multiline_getter_builtin() {
        return regexp_flag_getter_builtin(context, invocation, 'm').map(Some);
    }
    if entry == super::regexp_dot_all_getter_builtin() {
        return regexp_flag_getter_builtin(context, invocation, 's').map(Some);
    }
    if entry == super::regexp_unicode_getter_builtin() {
        return regexp_flag_getter_builtin(context, invocation, 'u').map(Some);
    }
    if entry == super::regexp_unicode_sets_getter_builtin() {
        return regexp_flag_getter_builtin(context, invocation, 'v').map(Some);
    }
    if entry == super::regexp_sticky_getter_builtin() {
        return regexp_flag_getter_builtin(context, invocation, 'y').map(Some);
    }
    if entry == super::regexp_source_getter_builtin() {
        return regexp_source_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::regexp_flags_getter_builtin() {
        return regexp_flags_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::regexp_has_indices_getter_builtin() {
        return regexp_has_indices_getter_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

pub(super) fn is_regexp_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
) -> bool {
    cx.agent().objects().is_regexp_object(object_ref)
}

fn well_known_symbol_key<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    symbol: WellKnownSymbolId,
) -> Result<PropertyKey, Cx::Error> {
    let symbol = {
        let agent = cx.agent();
        agent.well_known_symbol(symbol)
    }
    .ok_or_else(|| type_error(cx))?;
    Ok(PropertyKey::from_symbol(symbol))
}

pub(super) fn get_method_for_well_known_symbol<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    symbol: WellKnownSymbolId,
) -> Result<Option<ObjectRef>, Cx::Error> {
    let key = well_known_symbol_key(cx, symbol)?;
    let method = cx.get_property_value(value, key)?;
    if method.is_undefined() || method.is_null() {
        return Ok(None);
    }
    cx.require_callable_object(method).map(Some)
}

pub(super) fn is_regexp_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<bool, Cx::Error> {
    let Some(object_ref) = value.as_object_ref() else {
        return Ok(false);
    };
    let key = well_known_symbol_key(cx, WellKnownSymbolId::Match)?;
    let matcher = cx.get_property_value(value, key)?;
    if !matcher.is_undefined() {
        return to_boolean_for_builtin(cx, matcher);
    }
    Ok(is_regexp_object(cx, object_ref))
}

fn current_intrinsic_regexp_prototype<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Option<lyng_js_types::ObjectRef> {
    let realm = cx.builtin_realm();
    let agent = cx.agent();
    agent
        .realm(realm)
        .and_then(|record| record.intrinsics().regexp_prototype())
}

fn regexp_matcher_this_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    let object_ref = value.as_object_ref().ok_or_else(|| type_error(cx))?;
    if !is_regexp_object(cx, object_ref) {
        return Err(type_error(cx));
    }
    Ok(object_ref)
}

fn regexp_compile_this_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<ObjectRef, Cx::Error> {
    let object_ref = regexp_matcher_this_object(cx, value)?;
    let expected_prototype =
        current_intrinsic_regexp_prototype(cx).ok_or_else(|| type_error(cx))?;
    let actual_prototype = {
        let agent = cx.agent();
        agent
            .objects()
            .get_prototype_of(agent.heap().view(), object_ref)
    }
    .map_err(|_| type_error(cx))?;
    if actual_prototype != Some(expected_prototype) {
        return Err(type_error(cx));
    }
    Ok(object_ref)
}

fn regexp_last_index_key<Cx: PublicBuiltinDispatchContext>(cx: &mut Cx) -> PropertyKey {
    let last_index = {
        let agent = cx.agent();
        agent.bootstrap_atoms().last_index()
    };
    PropertyKey::from_atom(last_index)
}

fn boolean_property_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    receiver: Value,
    key: PropertyKey,
) -> Result<bool, Cx::Error> {
    let value = cx.get_property_value(receiver, key)?;
    to_boolean_for_builtin(cx, value)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RegExpExecState {
    flags: lyng_js_objects::RegExpObjectFlags,
    matched: lyng_js_objects::RegExpMatchRecord,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum RegExpExecResult {
    Builtin(Option<RegExpExecState>),
    Custom(Value),
}

#[derive(Clone, Copy, Debug)]
enum RegExpLegacyStaticProperty {
    Input,
    LastMatch,
    LastParen,
    LeftContext,
    RightContext,
    Paren(usize),
}

fn legacy_static_property_for_getter(
    entry: BuiltinFunctionId,
) -> Option<RegExpLegacyStaticProperty> {
    if entry == super::regexp_legacy_input_getter_builtin() {
        return Some(RegExpLegacyStaticProperty::Input);
    }
    if entry == super::regexp_legacy_last_match_getter_builtin() {
        return Some(RegExpLegacyStaticProperty::LastMatch);
    }
    if entry == super::regexp_legacy_last_paren_getter_builtin() {
        return Some(RegExpLegacyStaticProperty::LastParen);
    }
    if entry == super::regexp_legacy_left_context_getter_builtin() {
        return Some(RegExpLegacyStaticProperty::LeftContext);
    }
    if entry == super::regexp_legacy_right_context_getter_builtin() {
        return Some(RegExpLegacyStaticProperty::RightContext);
    }
    [
        super::regexp_legacy_paren1_getter_builtin(),
        super::regexp_legacy_paren2_getter_builtin(),
        super::regexp_legacy_paren3_getter_builtin(),
        super::regexp_legacy_paren4_getter_builtin(),
        super::regexp_legacy_paren5_getter_builtin(),
        super::regexp_legacy_paren6_getter_builtin(),
        super::regexp_legacy_paren7_getter_builtin(),
        super::regexp_legacy_paren8_getter_builtin(),
        super::regexp_legacy_paren9_getter_builtin(),
    ]
    .iter()
    .position(|candidate| *candidate == entry)
    .map(|zero_based| RegExpLegacyStaticProperty::Paren(zero_based + 1))
}

fn require_legacy_static_regexp_constructor<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    this_value: Value,
) -> Result<RealmRef, Cx::Error> {
    let realm = cx.builtin_realm();
    let constructor = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().regexp())
    }
    .ok_or_else(|| type_error(cx))?;
    if this_value.as_object_ref() != Some(constructor) {
        return Err(type_error(cx));
    }
    Ok(realm)
}

fn regexp_legacy_static_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    property: RegExpLegacyStaticProperty,
) -> Result<Value, Cx::Error> {
    let realm = require_legacy_static_regexp_constructor(cx, invocation.this_value())?;
    let text = {
        let agent = cx.agent();
        agent.regexp_legacy_static_state(realm).map(|state| {
            match property {
                RegExpLegacyStaticProperty::Input => state.input(),
                RegExpLegacyStaticProperty::LastMatch => state.last_match(),
                RegExpLegacyStaticProperty::LastParen => state.last_paren(),
                RegExpLegacyStaticProperty::LeftContext => state.left_context(),
                RegExpLegacyStaticProperty::RightContext => state.right_context(),
                RegExpLegacyStaticProperty::Paren(index) => {
                    state.paren(index).unwrap_or(&RegExpLegacyStaticText::Empty)
                }
            }
            .clone()
        })
    }
    .ok_or_else(|| type_error(cx))?;
    regexp_legacy_static_text_value(cx, text)
}

fn regexp_legacy_static_text_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    text: RegExpLegacyStaticText,
) -> Result<Value, Cx::Error> {
    match text {
        RegExpLegacyStaticText::Empty => Ok(string_from_code_units(cx, &[])),
        RegExpLegacyStaticText::Owned(units) => Ok(string_from_code_units(cx, &units)),
        RegExpLegacyStaticText::SourceSlice { source, range } => {
            string_from_string_ref_range(cx, source, range)
        }
    }
}

fn regexp_legacy_input_setter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let realm = require_legacy_static_regexp_constructor(cx, invocation.this_value())?;
    let value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let input_ref = to_string_string_ref(cx, value)?;
    let input_units = string_ref_code_units(cx, input_ref)?;
    let updated = {
        let agent = cx.agent();
        agent
            .regexp_legacy_static_state_mut(realm)
            .map(|state| state.set_input(input_units))
            .is_some()
    };
    if !updated {
        return Err(type_error(cx));
    }
    Ok(Value::undefined())
}

fn record_regexp_legacy_static_match<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    input_ref: StringRef,
    units: &[u16],
    state: &RegExpExecState,
) -> Result<(), Cx::Error> {
    let realm = cx.builtin_realm();
    let updated = {
        let agent = cx.agent();
        agent
            .regexp_legacy_static_state_mut(realm)
            .map(|legacy_state| {
                legacy_state.record_match(
                    input_ref,
                    units.len(),
                    state.matched.range(),
                    state.matched.captures(),
                );
            })
            .is_some()
    };
    if updated {
        Ok(())
    } else {
        Err(type_error(cx))
    }
}

const REGEXP_STRING_ITERATOR_REGEXP_SLOT: u32 = 0;
const REGEXP_STRING_ITERATOR_STRING_SLOT: u32 = 1;
const REGEXP_STRING_ITERATOR_GLOBAL_SLOT: u32 = 2;
const REGEXP_STRING_ITERATOR_UNICODE_SLOT: u32 = 3;
const REGEXP_STRING_ITERATOR_DONE_SLOT: u32 = 4;

pub(super) fn allocate_regexp_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    realm: RealmRef,
    prototype: lyng_js_types::ObjectRef,
    pattern: &str,
    flags: &str,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    let root_shape = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(lyng_js_env::RealmRecord::root_shape)
    }
    .ok_or_else(|| type_error(cx))?;
    let payload =
        lyng_js_objects::RegExpPayload::compile(pattern, flags).map_err(|_| syntax_error(cx))?;
    let object = cx.agent().with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let object = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape)
                .with_prototype(Some(prototype))
                .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::RegExp)),
            AllocationLifetime::Default,
        );
        let stored = objects.store_regexp_payload(object, payload);
        debug_assert!(stored, "fresh RegExp objects should accept payload storage");
        object
    });
    let last_index_key = regexp_last_index_key(cx);
    define_data_property_with_attrs(
        cx,
        object,
        last_index_key,
        Value::from_smi(0),
        true,
        false,
        false,
    )?;
    Ok(object)
}

pub(super) fn regexp_literal_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let pattern_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let flags_value = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let pattern_text = cx.value_to_string_text(pattern_value)?;
    let flags_text = cx.value_to_string_text(flags_value)?;
    if validate_regexp_literal(&pattern_text, &flags_text).is_err() {
        return Err(syntax_error(cx));
    }

    let realm = cx.builtin_realm();
    let prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().regexp_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
    allocate_regexp_object(cx, realm, prototype, &pattern_text, &flags_text)
        .map(Value::from_object_ref)
}

fn regexp_object_flags<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
) -> Result<lyng_js_objects::RegExpObjectFlags, Cx::Error> {
    let flags = {
        let agent = cx.agent();
        agent
            .objects()
            .regexp_payload(object_ref)
            .map(lyng_js_objects::RegExpPayload::flags)
    };
    if let Some(flags) = flags {
        return Ok(flags);
    }
    if current_intrinsic_regexp_prototype(cx) == Some(object_ref) {
        return Ok(lyng_js_objects::RegExpObjectFlags::default());
    }
    Err(type_error(cx))
}

fn regexp_object_source_and_flags<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
) -> Result<(String, String), Cx::Error> {
    let values = {
        let agent = cx.agent();
        agent
            .objects()
            .regexp_payload(object_ref)
            .map(|payload| (payload.source().to_owned(), payload.flag_text().to_owned()))
    };
    values.ok_or_else(|| type_error(cx))
}

pub(super) fn regexp_object_source_text<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
) -> Result<String, Cx::Error> {
    let text = {
        let agent = cx.agent();
        agent.objects().regexp_payload(object_ref).map(|payload| {
            if payload.source().is_empty() {
                "(?:)".to_owned()
            } else {
                payload.source().to_owned()
            }
        })
    };
    if let Some(text) = text {
        return Ok(text);
    }
    if current_intrinsic_regexp_prototype(cx) == Some(object_ref) {
        return Ok("(?:)".to_owned());
    }
    Err(type_error(cx))
}

fn advance_string_index(units: &[u16], index: usize, unicode_aware: bool) -> usize {
    if !unicode_aware || index + 1 >= units.len() {
        return index.saturating_add(1);
    }
    let first = units[index];
    let second = units[index + 1];
    if (0xD800..=0xDBFF).contains(&first) && (0xDC00..=0xDFFF).contains(&second) {
        index + 2
    } else {
        index + 1
    }
}

fn allocate_named_capture_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    captures: &[lyng_js_objects::RegExpNamedCapture],
    units: &[u16],
    use_indices: bool,
) -> Result<Option<Value>, Cx::Error> {
    if captures.is_empty() {
        return Ok(None);
    }
    let object = cx.allocate_ordinary_object_with_prototype(cx.builtin_realm(), None)?;
    for capture in captures {
        let atom = {
            let agent = cx.agent();
            agent.atoms_mut().intern_collectible(capture.name())
        };
        let value = match capture.range() {
            Some(range) if use_indices => {
                let pair = allocate_array_like_result(cx, 2)?;
                define_data_property_with_attrs(
                    cx,
                    pair,
                    PropertyKey::Index(0),
                    usize_index_value(range.start),
                    true,
                    true,
                    true,
                )?;
                define_data_property_with_attrs(
                    cx,
                    pair,
                    PropertyKey::Index(1),
                    usize_index_value(range.end),
                    true,
                    true,
                    true,
                )?;
                Value::from_object_ref(pair)
            }
            Some(range) => code_unit_range_value(cx, units, range),
            None => Value::undefined(),
        };
        define_data_property_with_attrs(
            cx,
            object,
            PropertyKey::from_atom(atom),
            value,
            true,
            true,
            true,
        )?;
    }
    Ok(Some(Value::from_object_ref(object)))
}

fn build_regexp_indices_result<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    units: &[u16],
    state: &RegExpExecState,
) -> Result<Value, Cx::Error> {
    let matched = &state.matched;
    let object = allocate_array_like_result(
        cx,
        u32::try_from(matched.captures().len() + 1).unwrap_or(u32::MAX),
    )?;
    let pair = allocate_array_like_result(cx, 2)?;
    define_data_property_with_attrs(
        cx,
        pair,
        PropertyKey::Index(0),
        usize_index_value(matched.start()),
        true,
        true,
        true,
    )?;
    define_data_property_with_attrs(
        cx,
        pair,
        PropertyKey::Index(1),
        usize_index_value(matched.end()),
        true,
        true,
        true,
    )?;
    define_data_property_with_attrs(
        cx,
        object,
        PropertyKey::Index(0),
        Value::from_object_ref(pair),
        true,
        true,
        true,
    )?;
    for (offset, capture) in matched.captures().iter().enumerate() {
        let value = match capture {
            Some(range) => {
                let pair = allocate_array_like_result(cx, 2)?;
                define_data_property_with_attrs(
                    cx,
                    pair,
                    PropertyKey::Index(0),
                    usize_index_value(range.start),
                    true,
                    true,
                    true,
                )?;
                define_data_property_with_attrs(
                    cx,
                    pair,
                    PropertyKey::Index(1),
                    usize_index_value(range.end),
                    true,
                    true,
                    true,
                )?;
                Value::from_object_ref(pair)
            }
            None => Value::undefined(),
        };
        define_data_property_with_attrs(
            cx,
            object,
            PropertyKey::Index(u32::try_from(offset + 1).unwrap_or(u32::MAX)),
            value,
            true,
            true,
            true,
        )?;
    }
    let groups_atom = {
        let agent = cx.agent();
        agent.atoms_mut().intern_collectible("groups")
    };
    let groups = allocate_named_capture_object(cx, matched.named_captures(), units, true)?
        .unwrap_or(Value::undefined());
    define_data_property_with_attrs(
        cx,
        object,
        PropertyKey::from_atom(groups_atom),
        groups,
        true,
        true,
        true,
    )?;
    Ok(Value::from_object_ref(object))
}

fn build_regexp_match_result<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    units: &[u16],
    input_value: Value,
    state: &RegExpExecState,
) -> Result<Value, Cx::Error> {
    let matched = &state.matched;
    let object = allocate_array_like_result(
        cx,
        u32::try_from(matched.captures().len() + 1).unwrap_or(u32::MAX),
    )?;
    let matched_value = code_unit_range_value(cx, units, matched.range());
    define_data_property_with_attrs(
        cx,
        object,
        PropertyKey::Index(0),
        matched_value,
        true,
        true,
        true,
    )?;
    for (offset, capture) in matched.captures().iter().enumerate() {
        let value = capture.clone().map_or(Value::undefined(), |range| {
            code_unit_range_value(cx, units, range)
        });
        define_data_property_with_attrs(
            cx,
            object,
            PropertyKey::Index(u32::try_from(offset + 1).unwrap_or(u32::MAX)),
            value,
            true,
            true,
            true,
        )?;
    }
    let (index_atom, input_atom, groups_atom, indices_atom) = {
        let agent = cx.agent();
        (
            agent.atoms_mut().intern_collectible("index"),
            agent.atoms_mut().intern_collectible("input"),
            agent.atoms_mut().intern_collectible("groups"),
            agent.atoms_mut().intern_collectible("indices"),
        )
    };
    define_data_property_with_attrs(
        cx,
        object,
        PropertyKey::from_atom(index_atom),
        usize_index_value(matched.start()),
        true,
        true,
        true,
    )?;
    define_data_property_with_attrs(
        cx,
        object,
        PropertyKey::from_atom(input_atom),
        input_value,
        true,
        true,
        true,
    )?;
    let groups = allocate_named_capture_object(cx, matched.named_captures(), units, false)?
        .unwrap_or(Value::undefined());
    define_data_property_with_attrs(
        cx,
        object,
        PropertyKey::from_atom(groups_atom),
        groups,
        true,
        true,
        true,
    )?;
    if state.flags.has_indices() {
        let indices = build_regexp_indices_result(cx, units, state)?;
        define_data_property_with_attrs(
            cx,
            object,
            PropertyKey::from_atom(indices_atom),
            indices,
            true,
            true,
            true,
        )?;
    }
    Ok(Value::from_object_ref(object))
}

fn regexp_exec_state<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
    input_ref: StringRef,
    units: &[u16],
    advance_empty_match: bool,
) -> Result<Option<RegExpExecState>, Cx::Error> {
    let flags = regexp_object_flags(cx, object_ref)?;
    let last_index_key = regexp_last_index_key(cx);
    let last_index = cx.get_property_value(Value::from_object_ref(object_ref), last_index_key)?;
    let last_index = to_length_for_builtin(cx, last_index)?;
    let uses_stateful_last_index = flags.global() || flags.sticky();
    let start_index = if uses_stateful_last_index {
        last_index
    } else {
        0
    };
    if uses_stateful_last_index && start_index > units.len() {
        set_data_property_value(cx, object_ref, last_index_key, Value::from_smi(0))?;
        return Ok(None);
    }

    let matched = {
        let agent = cx.agent();
        agent
            .objects()
            .regexp_payload(object_ref)
            .and_then(|payload| payload.find_from_code_units(units, start_index))
    };
    let matched = matched.filter(|matched| !flags.sticky() || matched.start() == start_index);
    if let Some(matched) = matched {
        if uses_stateful_last_index {
            let next_index = if advance_empty_match && matched.start() == matched.end() {
                advance_string_index(units, matched.end(), flags.unicode_aware())
            } else {
                matched.end()
            };
            set_data_property_value(
                cx,
                object_ref,
                last_index_key,
                usize_index_value(next_index),
            )?;
        }
        let state = RegExpExecState { flags, matched };
        record_regexp_legacy_static_match(cx, input_ref, units, &state)?;
        return Ok(Some(state));
    }

    if uses_stateful_last_index {
        set_data_property_value(cx, object_ref, last_index_key, Value::from_smi(0))?;
    }
    Ok(None)
}

fn regexp_exec<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    input_ref: StringRef,
    input_units: &[u16],
) -> Result<Value, Cx::Error> {
    match regexp_exec_result(cx, object_ref, input_ref, input_units, false)? {
        RegExpExecResult::Builtin(Some(state)) => {
            build_regexp_match_result(cx, input_units, Value::from_string_ref(input_ref), &state)
        }
        RegExpExecResult::Builtin(None) => Ok(Value::null()),
        RegExpExecResult::Custom(result) => Ok(result),
    }
}

fn regexp_exec_result<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    input_ref: StringRef,
    input_units: &[u16],
    advance_empty_match: bool,
) -> Result<RegExpExecResult, Cx::Error> {
    let exec_key = {
        let agent = cx.agent();
        PropertyKey::from_atom(agent.atoms_mut().intern_collectible("exec"))
    };
    let exec = cx.get_property_value(Value::from_object_ref(object_ref), exec_key)?;
    if let Some(exec_object) = callable_object_from_value(cx, exec) {
        if is_current_realm_regexp_exec_builtin(cx, exec_object) {
            return regexp_exec_state(cx, object_ref, input_ref, input_units, advance_empty_match)
                .map(RegExpExecResult::Builtin);
        }
        let input_value = Value::from_string_ref(input_ref);
        let result = cx.call_to_completion(
            exec_object,
            Value::from_object_ref(object_ref),
            &[input_value],
        )?;
        if result.is_null() || result.as_object_ref().is_some() {
            return Ok(RegExpExecResult::Custom(result));
        }
        return Err(type_error(cx));
    }

    regexp_exec_state(cx, object_ref, input_ref, input_units, advance_empty_match)
        .map(RegExpExecResult::Builtin)
}

fn is_current_realm_regexp_exec_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    exec_object: ObjectRef,
) -> bool {
    let builtin_realm = cx.builtin_realm();
    let agent = cx.agent();
    let Some(data) = agent.objects().function_data(exec_object) else {
        return false;
    };
    data.realm() == Some(builtin_realm)
        && builtin_function_entry(agent, exec_object) == Some(super::regexp_exec_builtin())
}

fn regexp_default_constructor<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
) -> Result<ObjectRef, Cx::Error> {
    let realm = cx.builtin_realm();
    cx.agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().regexp())
        .ok_or_else(|| type_error(cx))
}

fn regexp_species_constructor<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
) -> Result<ObjectRef, Cx::Error> {
    let default_constructor = regexp_default_constructor(cx)?;
    let constructor = cx.get_property_value(
        Value::from_object_ref(object_ref),
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

fn set_property_on_object_or_throw<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: ObjectRef,
    key: PropertyKey,
    value: Value,
) -> Result<(), Cx::Error> {
    if !cx.set_property_on_object_with_receiver(
        object_ref,
        key,
        value,
        Value::from_object_ref(object_ref),
    )? {
        return Err(type_error(cx));
    }
    Ok(())
}

fn create_regexp_string_iterator<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    regexp_object: ObjectRef,
    source_ref: StringRef,
    global: bool,
    full_unicode: bool,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().regexp_string_iterator_prototype())
        .ok_or_else(|| type_error(cx))?;
    let slots = [
        Value::from_object_ref(regexp_object),
        Value::from_string_ref(source_ref),
        Value::from_bool(global),
        Value::from_bool(full_unicode),
        Value::from_bool(false),
    ];
    let iterator = iterators::allocate_iterator_object(
        cx,
        prototype,
        OrdinaryObjectData::RegExpStringIterator,
        &slots,
    )?;
    Ok(Value::from_object_ref(iterator))
}

pub(super) fn code_unit_ascii(unit: u16) -> Option<u8> {
    u8::try_from(unit).ok().filter(u8::is_ascii)
}

fn expand_regexp_replacement_template<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    template_units: &[u16],
    source_units: &[u16],
    matched_units: &[u16],
    position: usize,
    captures: &[Option<Vec<u16>>],
    named_captures: Value,
) -> Result<Vec<u16>, Cx::Error> {
    let mut result = Vec::with_capacity(template_units.len());
    let mut index = 0;
    while index < template_units.len() {
        if template_units[index] != u16::from(b'$') {
            result.push(template_units[index]);
            index += 1;
            continue;
        }
        let Some(next) = template_units.get(index + 1).copied() else {
            result.push(u16::from(b'$'));
            index += 1;
            continue;
        };
        match code_unit_ascii(next).map(char::from) {
            Some('$') => {
                result.push(u16::from(b'$'));
                index += 2;
            }
            Some('&') => {
                result.extend_from_slice(matched_units);
                index += 2;
            }
            Some('`') => {
                result.extend_from_slice(&source_units[..position]);
                index += 2;
            }
            Some('\'') => {
                let end = position
                    .saturating_add(matched_units.len())
                    .min(source_units.len());
                result.extend_from_slice(&source_units[end..]);
                index += 2;
            }
            Some('<') => {
                if named_captures.is_undefined() {
                    result.extend_from_slice(&[u16::from(b'$'), u16::from(b'<')]);
                    index += 2;
                    continue;
                }
                let mut end = index + 2;
                while end < template_units.len() && template_units[end] != u16::from(b'>') {
                    end += 1;
                }
                if end == template_units.len() {
                    result.extend_from_slice(&[u16::from(b'$'), u16::from(b'<')]);
                    index += 2;
                    continue;
                }
                let name = String::from_utf16_lossy(&template_units[index + 2..end]);
                let group_value = if let Some(groups) = named_captures.as_object_ref() {
                    let key = {
                        let agent = cx.agent();
                        PropertyKey::from_atom(agent.atoms_mut().intern_collectible(&name))
                    };
                    cx.get_property_value(Value::from_object_ref(groups), key)?
                } else {
                    Value::undefined()
                };
                if !group_value.is_undefined() {
                    let group_ref = to_string_string_ref(cx, group_value)?;
                    append_string_ref_code_units(cx, group_ref, &mut result)?;
                }
                index = end + 1;
            }
            Some(digit @ '0'..='9') => {
                let first = usize::from((digit as u8) - b'0');
                let mut capture_index = first;
                let mut digit_count = 1;
                if let Some(second) = template_units
                    .get(index + 2)
                    .and_then(|unit| code_unit_ascii(*unit))
                    .filter(u8::is_ascii_digit)
                {
                    let candidate = first * 10 + usize::from(second - b'0');
                    digit_count = 2;
                    capture_index = candidate;
                    if capture_index > captures.len() && first != 0 {
                        digit_count = 1;
                        capture_index = first;
                    }
                }
                if (1..=captures.len()).contains(&capture_index) {
                    if let Some(capture) = &captures[capture_index - 1] {
                        result.extend_from_slice(capture);
                    }
                    index += 1 + digit_count;
                } else {
                    result.push(u16::from(b'$'));
                    result.extend_from_slice(&template_units[index + 1..index + 1 + digit_count]);
                    index += 1 + digit_count;
                }
            }
            _ => {
                result.push(u16::from(b'$'));
                index += 1;
            }
        }
    }
    Ok(result)
}

fn regexp_result_position<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    result: Value,
    source_len: usize,
) -> Result<usize, Cx::Error> {
    let index_key = {
        let agent = cx.agent();
        PropertyKey::from_atom(agent.atoms_mut().intern_collectible("index"))
    };
    let index_value = cx.get_property_value(result, index_key)?;
    let integer = to_integer_or_infinity_for_builtin(cx, index_value)?;
    if integer <= 0.0 {
        return Ok(0);
    }
    if !integer.is_finite() {
        return Ok(source_len);
    }
    Ok((integer as usize).min(source_len))
}

fn regexp_result_capture_count<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    result: Value,
) -> Result<usize, Cx::Error> {
    let length_key = PropertyKey::from_atom(WellKnownAtom::length.id());
    let length_value = cx.get_property_value(result, length_key)?;
    let length = to_length_for_builtin(cx, length_value)?;
    Ok(length.saturating_sub(1))
}

pub(super) fn regexp_search_value_is_rejected<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<bool, Cx::Error> {
    is_regexp_value(cx, value)
}

pub(super) fn regexp_match_all_with_string<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    regexp_value: Value,
    source_ref: StringRef,
) -> Result<Value, Cx::Error> {
    let object_ref = regexp_value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let constructor = regexp_species_constructor(cx, object_ref)?;
    let flags_key = {
        let agent = cx.agent();
        PropertyKey::from_atom(agent.bootstrap_atoms().flags())
    };
    let flags_value = cx.get_property_value(regexp_value, flags_key)?;
    let flags_ref = to_string_string_ref(cx, flags_value)?;
    let flags_units = string_ref_code_units(cx, flags_ref)?;
    let flags_text = String::from_utf16_lossy(&flags_units);
    let matcher = cx.construct_to_completion(
        constructor,
        &[regexp_value, Value::from_string_ref(flags_ref)],
        Some(constructor),
    )?;
    let last_index_key = regexp_last_index_key(cx);
    let last_index = cx.get_property_value(regexp_value, last_index_key)?;
    let last_index = to_length_for_builtin(cx, last_index)?;
    set_property_on_object_or_throw(cx, matcher, last_index_key, usize_index_value(last_index))?;

    create_regexp_string_iterator(
        cx,
        matcher,
        source_ref,
        flags_text.contains('g'),
        flags_text.contains('u') || flags_text.contains('v'),
    )
}

fn regexp_string_iterator_next_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let iterator_object = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let done = iterators::iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::RegExpStringIterator,
        REGEXP_STRING_ITERATOR_DONE_SLOT,
    )?
    .as_bool()
    .ok_or_else(|| type_error(cx))?;
    if done {
        return iterators::create_iterator_result_value(cx, Value::undefined(), true);
    }

    let regexp_object = iterators::iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::RegExpStringIterator,
        REGEXP_STRING_ITERATOR_REGEXP_SLOT,
    )?
    .as_object_ref()
    .ok_or_else(|| type_error(cx))?;
    let source_ref = iterators::iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::RegExpStringIterator,
        REGEXP_STRING_ITERATOR_STRING_SLOT,
    )?
    .as_string_ref()
    .ok_or_else(|| type_error(cx))?;
    let global = iterators::iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::RegExpStringIterator,
        REGEXP_STRING_ITERATOR_GLOBAL_SLOT,
    )?
    .as_bool()
    .ok_or_else(|| type_error(cx))?;
    let full_unicode = iterators::iterator_slot_value_for_builtin(
        cx,
        iterator_object,
        OrdinaryObjectData::RegExpStringIterator,
        REGEXP_STRING_ITERATOR_UNICODE_SLOT,
    )?
    .as_bool()
    .ok_or_else(|| type_error(cx))?;
    with_string_ref_code_units(cx, source_ref, |cx, source_units| {
        let matched = regexp_exec(cx, regexp_object, source_ref, source_units)?;
        if matched.is_null() {
            iterators::set_iterator_slot_value_for_builtin(
                cx,
                iterator_object,
                OrdinaryObjectData::RegExpStringIterator,
                REGEXP_STRING_ITERATOR_DONE_SLOT,
                Value::from_bool(true),
            )?;
            return iterators::create_iterator_result_value(cx, Value::undefined(), true);
        }
        if !global {
            iterators::set_iterator_slot_value_for_builtin(
                cx,
                iterator_object,
                OrdinaryObjectData::RegExpStringIterator,
                REGEXP_STRING_ITERATOR_DONE_SLOT,
                Value::from_bool(true),
            )?;
            return iterators::create_iterator_result_value(cx, matched, false);
        }

        let match_str = {
            let element = cx.get_property_value(matched, PropertyKey::Index(0))?;
            let element_ref = to_string_string_ref(cx, element)?;
            string_ref_code_units(cx, element_ref)?
        };
        if match_str.is_empty() {
            let last_index_key = regexp_last_index_key(cx);
            let this_index =
                cx.get_property_value(Value::from_object_ref(regexp_object), last_index_key)?;
            let this_index = to_length_for_builtin(cx, this_index)?;
            let next_index = advance_string_index(source_units, this_index, full_unicode);
            set_property_on_object_or_throw(
                cx,
                regexp_object,
                last_index_key,
                usize_index_value(next_index),
            )?;
        }
        iterators::create_iterator_result_value(cx, matched, false)
    })
}

pub(super) fn regexp_replace_with_string<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    regexp_value: Value,
    source_ref: StringRef,
    replacement: Value,
) -> Result<Value, Cx::Error> {
    let regexp_object = regexp_value.as_object_ref().ok_or_else(|| type_error(cx))?;
    with_string_ref_code_units(cx, source_ref, |cx, source_units| {
        let source_value = Value::from_string_ref(source_ref);
        let callable_replacement = callable_object_from_value(cx, replacement);
        let replacement_template_units = if callable_replacement.is_none() {
            let replacement_ref = to_string_string_ref(cx, replacement)?;
            Some(string_ref_code_units(cx, replacement_ref)?)
        } else {
            None
        };
        let flags_key = {
            let agent = cx.agent();
            PropertyKey::from_atom(agent.bootstrap_atoms().flags())
        };
        let flags_value = cx.get_property_value(regexp_value, flags_key)?;
        let flags_text = cx.value_to_string_text(flags_value)?;
        let global = flags_text.contains('g');
        let full_unicode = flags_text.contains('u') || flags_text.contains('v');
        let last_index_key = regexp_last_index_key(cx);
        if global {
            set_property_on_object_or_throw(cx, regexp_object, last_index_key, Value::from_smi(0))?;
        }

        let mut results = Vec::new();
        loop {
            let result = regexp_exec(cx, regexp_object, source_ref, source_units)?;
            if result.is_null() {
                break;
            }
            if global {
                let matched = cx.get_property_value(result, PropertyKey::Index(0))?;
                let matched_ref = to_string_string_ref(cx, matched)?;
                let matched_units = string_ref_code_units(cx, matched_ref)?;
                if matched_units.is_empty() {
                    let this_index = cx.get_property_value(regexp_value, last_index_key)?;
                    let this_index = to_length_for_builtin(cx, this_index)?;
                    let next_index = advance_string_index(source_units, this_index, full_unicode);
                    set_property_on_object_or_throw(
                        cx,
                        regexp_object,
                        last_index_key,
                        usize_index_value(next_index),
                    )?;
                }
            }
            results.push(result);
            if !global {
                break;
            }
        }
        if results.is_empty() {
            return Ok(Value::from_string_ref(source_ref));
        }

        let mut result = Vec::with_capacity(source_units.len());
        let mut next_source_position = 0;
        for match_result in results {
            let capture_count = regexp_result_capture_count(cx, match_result)?;
            let matched_value = cx.get_property_value(match_result, PropertyKey::Index(0))?;
            let matched_ref = to_string_string_ref(cx, matched_value)?;
            let matched_units = string_ref_code_units(cx, matched_ref)?;
            let position = regexp_result_position(cx, match_result, source_units.len())?;
            let mut capture_units = Vec::with_capacity(capture_count);
            let mut capture_values = Vec::with_capacity(capture_count);
            for index in 1..=capture_count {
                let capture = cx.get_property_value(
                    match_result,
                    PropertyKey::Index(u32::try_from(index).unwrap_or(u32::MAX)),
                )?;
                if capture.is_undefined() {
                    capture_units.push(None);
                    capture_values.push(Value::undefined());
                } else {
                    let capture_ref = to_string_string_ref(cx, capture)?;
                    capture_units.push(Some(string_ref_code_units(cx, capture_ref)?));
                    capture_values.push(Value::from_string_ref(capture_ref));
                }
            }
            let groups_key = {
                let agent = cx.agent();
                PropertyKey::from_atom(agent.atoms_mut().intern_collectible("groups"))
            };
            let named_captures = cx.get_property_value(match_result, groups_key)?;
            let replacement_units = if let Some(callee) = callable_replacement {
                let mut arguments = Vec::with_capacity(capture_values.len() + 5);
                arguments.push(Value::from_string_ref(matched_ref));
                arguments.extend(capture_values);
                arguments.push(usize_index_value(position));
                arguments.push(source_value);
                if !named_captures.is_undefined() {
                    arguments.push(named_captures);
                }
                let replaced = cx.call_to_completion(callee, Value::undefined(), &arguments)?;
                let replaced_ref = to_string_string_ref(cx, replaced)?;
                string_ref_code_units(cx, replaced_ref)?
            } else {
                let named_captures = if named_captures.is_undefined() {
                    Value::undefined()
                } else {
                    Value::from_object_ref(
                        cx.to_object_for_builtin_value(cx.builtin_realm(), named_captures)?,
                    )
                };
                expand_regexp_replacement_template(
                    cx,
                    replacement_template_units
                        .as_deref()
                        .expect("template units should exist for non-callable replacements"),
                    source_units,
                    &matched_units,
                    position,
                    &capture_units,
                    named_captures,
                )?
            };
            if position >= next_source_position {
                result.extend_from_slice(&source_units[next_source_position..position]);
                result.extend_from_slice(&replacement_units);
                next_source_position = position
                    .saturating_add(matched_units.len())
                    .min(source_units.len());
            }
        }
        result.extend_from_slice(&source_units[next_source_position..]);
        Ok(string_from_code_units(cx, &result))
    })
}

fn regexp_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let receiver = invocation.this_value();
    if receiver.as_object_ref().is_none() {
        return Err(type_error(cx));
    }
    let (source_key, flags_key) = {
        let agent = cx.agent();
        (
            PropertyKey::from_atom(agent.bootstrap_atoms().source()),
            PropertyKey::from_atom(agent.bootstrap_atoms().flags()),
        )
    };
    let source_value = cx.get_property_value(receiver, source_key)?;
    let source = cx.value_to_string_text(source_value)?;
    let flags_value = cx.get_property_value(receiver, flags_key)?;
    let flags = cx.value_to_string_text(flags_value)?;
    Ok(string_value(cx, &format!("/{source}/{flags}")))
}

fn regexp_compile_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = regexp_compile_this_object(cx, invocation.this_value())?;
    let pattern_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let flags_value = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());

    let (pattern_text, flags_text) = if let Some(pattern_object) = pattern_value.as_object_ref() {
        if is_regexp_object(cx, pattern_object) {
            if !flags_value.is_undefined() {
                return Err(type_error(cx));
            }
            regexp_object_source_and_flags(cx, pattern_object)?
        } else {
            let pattern_text = if pattern_value.is_undefined() {
                String::new()
            } else {
                normalize_regexp_constructor_pattern_text(&cx.value_to_string_text(pattern_value)?)
            };
            let flags_text = if flags_value.is_undefined() {
                String::new()
            } else {
                cx.value_to_string_text(flags_value)?
            };
            (pattern_text, flags_text)
        }
    } else {
        let pattern_text = if pattern_value.is_undefined() {
            String::new()
        } else {
            normalize_regexp_constructor_pattern_text(&cx.value_to_string_text(pattern_value)?)
        };
        let flags_text = if flags_value.is_undefined() {
            String::new()
        } else {
            cx.value_to_string_text(flags_value)?
        };
        (pattern_text, flags_text)
    };

    if validate_regexp_constructor_pattern(&pattern_text, &flags_text).is_err() {
        return Err(syntax_error(cx));
    }
    let payload = lyng_js_objects::RegExpPayload::compile(&pattern_text, &flags_text)
        .map_err(|_| syntax_error(cx))?;
    let stored = cx
        .agent()
        .objects_mut()
        .store_regexp_payload(object_ref, payload);
    if !stored {
        return Err(type_error(cx));
    }
    let last_index_key = regexp_last_index_key(cx);
    set_data_property_value(cx, object_ref, last_index_key, Value::from_smi(0))?;
    Ok(Value::from_object_ref(object_ref))
}

fn regexp_exec_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = regexp_matcher_this_object(cx, invocation.this_value())?;
    let input_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    with_string_ref_code_units(cx, input_ref, |cx, input_units| {
        let input_value = Value::from_string_ref(input_ref);
        let Some(state) = regexp_exec_state(cx, object_ref, input_ref, input_units, false)? else {
            return Ok(Value::null());
        };
        build_regexp_match_result(cx, input_units, input_value, &state)
    })
}

fn regexp_test_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let input_ref = to_string_string_ref(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let exec_key = {
        let agent = cx.agent();
        PropertyKey::from_atom(agent.atoms_mut().intern_collectible("exec"))
    };
    let exec = cx.get_property_value(Value::from_object_ref(object_ref), exec_key)?;
    let default_exec = exec
        .as_object_ref()
        .and_then(|exec| builtin_function_entry(cx.agent(), exec))
        == Some(super::regexp_exec_builtin());
    if default_exec {
        let object_ref = regexp_matcher_this_object(cx, Value::from_object_ref(object_ref))?;
        return with_string_ref_code_units(cx, input_ref, |cx, input_units| {
            Ok(Value::from_bool(
                regexp_exec_state(cx, object_ref, input_ref, input_units, false)?.is_some(),
            ))
        });
    }

    with_string_ref_code_units(cx, input_ref, |cx, input_units| {
        let matched = regexp_exec(cx, object_ref, input_ref, input_units)?;
        Ok(Value::from_bool(!matched.is_null()))
    })
}
