mod accessors;
mod construction;
mod escape;
mod symbols;

use super::{
    allocate_array_like_result, callable_object_from_value, code_unit_range_value,
    create_array_from_values, define_data_property_with_attrs, iterators, set_data_property_value,
    string_from_code_units, string_ref_code_units, string_value, syntax_error,
    to_boolean_for_builtin, to_length_for_builtin, to_string_string_ref, type_error,
    usize_index_value, PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use accessors::{
    regexp_flag_getter_builtin, regexp_flags_getter_builtin, regexp_has_indices_getter_builtin,
    regexp_source_getter_builtin,
};
use construction::{regexp_builtin, regexp_species_getter_builtin};
use escape::regexp_escape_builtin;
use lyng_js_gc::AllocationLifetime;
use lyng_js_objects::{ObjectAllocation, ObjectColdData, OrdinaryObjectData};
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

fn regexp_object_flag_text<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
) -> Result<String, Cx::Error> {
    let text = {
        let agent = cx.agent();
        agent
            .objects()
            .regexp_payload(object_ref)
            .map(|payload| payload.flag_text().to_owned())
    };
    if let Some(text) = text {
        return Ok(text);
    }
    if current_intrinsic_regexp_prototype(cx) == Some(object_ref) {
        return Ok(String::new());
    }
    Err(type_error(cx))
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

fn build_regexp_global_match_result<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    units: &[u16],
    matches: &[lyng_js_objects::RegExpMatchRecord],
) -> Result<Value, Cx::Error> {
    let object = allocate_array_like_result(cx, u32::try_from(matches.len()).unwrap_or(u32::MAX))?;
    for (index, matched) in matches.iter().enumerate() {
        let matched_value = code_unit_range_value(cx, units, matched.range());
        define_data_property_with_attrs(
            cx,
            object,
            PropertyKey::Index(u32::try_from(index).unwrap_or(u32::MAX)),
            matched_value,
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
    units: &[u16],
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
            let next_index = if matched.start() == matched.end() {
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
        return Ok(Some(RegExpExecState { flags, matched }));
    }

    if uses_stateful_last_index {
        set_data_property_value(cx, object_ref, last_index_key, Value::from_smi(0))?;
    }
    Ok(None)
}

fn capture_range_for_name(
    captures: &[lyng_js_objects::RegExpNamedCapture],
    name: &str,
) -> Option<std::ops::Range<usize>> {
    captures
        .iter()
        .find(|capture| capture.name() == name)
        .and_then(lyng_js_objects::RegExpNamedCapture::range)
}

pub(super) fn code_unit_ascii(unit: u16) -> Option<u8> {
    u8::try_from(unit).ok().filter(u8::is_ascii)
}

fn expand_regexp_replacement_template(
    template_units: &[u16],
    source_units: &[u16],
    state: &RegExpExecState,
) -> Vec<u16> {
    let mut result = Vec::with_capacity(template_units.len());
    let matched = &state.matched;
    let captures = matched.captures();
    let named_captures = matched.named_captures();
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
                result.extend_from_slice(&source_units[matched.start()..matched.end()]);
                index += 2;
            }
            Some('`') => {
                result.extend_from_slice(&source_units[..matched.start()]);
                index += 2;
            }
            Some('\'') => {
                result.extend_from_slice(&source_units[matched.end()..]);
                index += 2;
            }
            Some('<') => {
                let mut end = index + 2;
                while end < template_units.len() && template_units[end] != u16::from(b'>') {
                    end += 1;
                }
                if end == template_units.len() {
                    result.push(u16::from(b'$'));
                    index += 1;
                    continue;
                }
                let name = String::from_utf16_lossy(&template_units[index + 2..end]);
                if let Some(range) = capture_range_for_name(named_captures, &name) {
                    result.extend_from_slice(&source_units[range.start..range.end]);
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
                    if let Some(range) = captures[capture_index - 1].clone() {
                        result.extend_from_slice(&source_units[range.start..range.end]);
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
    result
}

pub(super) fn regexp_search_value_is_rejected<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<bool, Cx::Error> {
    is_regexp_value(cx, value)
}

pub(super) fn regexp_match_with_string<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    regexp_object: ObjectRef,
    source_ref: StringRef,
) -> Result<Value, Cx::Error> {
    let source_units = string_ref_code_units(cx, source_ref)?;
    let source_value = Value::from_string_ref(source_ref);
    let flags = regexp_object_flags(cx, regexp_object)?;
    if flags.global() {
        let last_index_key = regexp_last_index_key(cx);
        set_data_property_value(cx, regexp_object, last_index_key, Value::from_smi(0))?;
        let mut matches = Vec::new();
        while let Some(state) = regexp_exec_state(cx, regexp_object, &source_units)? {
            matches.push(state.matched);
        }
        if matches.is_empty() {
            return Ok(Value::null());
        }
        return build_regexp_global_match_result(cx, &source_units, &matches);
    }

    let Some(state) = regexp_exec_state(cx, regexp_object, &source_units)? else {
        return Ok(Value::null());
    };
    build_regexp_match_result(cx, &source_units, source_value, &state)
}

pub(super) fn regexp_match_all_with_string<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    regexp_object: ObjectRef,
    source_ref: StringRef,
) -> Result<Value, Cx::Error> {
    let source_units = string_ref_code_units(cx, source_ref)?;
    let source_value = Value::from_string_ref(source_ref);
    let flags = regexp_object_flags(cx, regexp_object)?;
    if flags.global() {
        let last_index_key = regexp_last_index_key(cx);
        set_data_property_value(cx, regexp_object, last_index_key, Value::from_smi(0))?;
    }

    let mut matches = Vec::new();
    while let Some(state) = regexp_exec_state(cx, regexp_object, &source_units)? {
        matches.push(build_regexp_match_result(
            cx,
            &source_units,
            source_value,
            &state,
        )?);
        if !flags.global() {
            break;
        }
    }
    let array = create_array_from_values(cx, &matches)?;
    iterators::array_iterator_factory_builtin(
        cx,
        BuiltinInvocation::new(Value::from_object_ref(array), &[], None),
        iterators::ArrayIterationKind::Value,
    )
}

pub(super) fn regexp_search_with_string<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    regexp_object: ObjectRef,
    source_ref: StringRef,
) -> Result<Value, Cx::Error> {
    let source_units = string_ref_code_units(cx, source_ref)?;
    let payload = {
        let agent = cx.agent();
        agent.objects().regexp_payload(regexp_object).cloned()
    }
    .ok_or_else(|| type_error(cx))?;
    Ok(payload.find_from_code_units(&source_units, 0).map_or_else(
        || Value::from_smi(-1),
        |record| usize_index_value(record.range().start),
    ))
}

pub(super) fn regexp_replace_with_string<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    regexp_object: ObjectRef,
    source_ref: StringRef,
    replacement: Value,
) -> Result<Value, Cx::Error> {
    let source_units = string_ref_code_units(cx, source_ref)?;
    let source_value = Value::from_string_ref(source_ref);
    let callable_replacement = callable_object_from_value(cx, replacement);
    let flags = regexp_object_flags(cx, regexp_object)?;
    if flags.global() {
        let last_index_key = regexp_last_index_key(cx);
        set_data_property_value(cx, regexp_object, last_index_key, Value::from_smi(0))?;
    }

    let replacement_template_units = if callable_replacement.is_none() {
        let replacement_ref = to_string_string_ref(cx, replacement)?;
        Some(string_ref_code_units(cx, replacement_ref)?)
    } else {
        None
    };

    let mut match_states = Vec::new();
    while let Some(state) = regexp_exec_state(cx, regexp_object, &source_units)? {
        match_states.push(state);
        if !flags.global() {
            break;
        }
    }
    if match_states.is_empty() {
        return Ok(Value::from_string_ref(source_ref));
    }

    let mut result = Vec::with_capacity(source_units.len());
    let mut cursor = 0;
    for state in match_states {
        let matched = &state.matched;
        result.extend_from_slice(&source_units[cursor..matched.start()]);
        let replacement_units = if let Some(callee) = callable_replacement {
            let mut arguments = Vec::with_capacity(matched.captures().len() + 4);
            arguments.push(code_unit_range_value(cx, &source_units, matched.range()));
            for capture in matched.captures() {
                let value = capture.clone().map_or(Value::undefined(), |range| {
                    code_unit_range_value(cx, &source_units, range)
                });
                arguments.push(value);
            }
            arguments.push(usize_index_value(matched.start()));
            arguments.push(source_value);
            if let Some(groups) =
                allocate_named_capture_object(cx, matched.named_captures(), &source_units, false)?
            {
                arguments.push(groups);
            }
            let replaced = cx.call_to_completion(callee, Value::undefined(), &arguments)?;
            let replaced_ref = to_string_string_ref(cx, replaced)?;
            string_ref_code_units(cx, replaced_ref)?
        } else {
            expand_regexp_replacement_template(
                replacement_template_units
                    .as_deref()
                    .expect("template units should exist for non-callable replacements"),
                &source_units,
                &state,
            )
        };
        result.extend_from_slice(&replacement_units);
        cursor = matched.end();
    }
    result.extend_from_slice(&source_units[cursor..]);
    Ok(string_from_code_units(cx, &result))
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
    let input_units = string_ref_code_units(cx, input_ref)?;
    let input_value = Value::from_string_ref(input_ref);
    let Some(state) = regexp_exec_state(cx, object_ref, &input_units)? else {
        return Ok(Value::null());
    };
    build_regexp_match_result(cx, &input_units, input_value, &state)
}

fn regexp_test_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let matched = regexp_exec_builtin(cx, invocation)?;
    Ok(Value::from_bool(!matched.is_null()))
}
