use super::super::{syntax_error, type_error, PublicBuiltinDispatchContext};
use super::{
    allocate_regexp_object, is_regexp_object, is_regexp_value, regexp_object_source_and_flags,
};
use crate::BuiltinInvocation;
use lyng_js_common::WellKnownAtom;
use lyng_js_parser::validate_regexp_constructor_pattern;
use lyng_js_types::{PropertyKey, Value};

enum RegExpPatternSeed {
    Text(String),
    Value(Value),
}

enum RegExpFlagsSeed {
    Text(String),
    Value(Value),
}

pub(super) const fn regexp_species_getter_builtin(invocation: BuiltinInvocation<'_>) -> Value {
    invocation.this_value()
}

pub(super) fn normalize_regexp_constructor_pattern_text(
    pattern: &str,
    unicode_aware: bool,
) -> String {
    let mut normalized = String::with_capacity(pattern.len());
    let mut trailing_backslashes = 0usize;
    for ch in pattern.chars() {
        match ch {
            '\n' if trailing_backslashes.is_multiple_of(2) => normalized.push_str("\\n"),
            '\n' if !unicode_aware => normalized.push('n'),
            '\n' => normalized.push(ch),
            '\r' if trailing_backslashes.is_multiple_of(2) => normalized.push_str("\\r"),
            '\r' if !unicode_aware => normalized.push('r'),
            '\r' => normalized.push(ch),
            '\u{2028}' if trailing_backslashes.is_multiple_of(2) => normalized.push_str("\\u2028"),
            '\u{2028}' if !unicode_aware => normalized.push_str("u2028"),
            '\u{2028}' => normalized.push(ch),
            '\u{2029}' if trailing_backslashes.is_multiple_of(2) => normalized.push_str("\\u2029"),
            '\u{2029}' if !unicode_aware => normalized.push_str("u2029"),
            '\u{2029}' => normalized.push(ch),
            _ => normalized.push(ch),
        }
        if ch == '\\' {
            trailing_backslashes += 1;
        } else {
            trailing_backslashes = 0;
        }
    }
    normalized
}

fn regexp_pattern_seed_text<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    seed: RegExpPatternSeed,
) -> Result<String, Cx::Error> {
    match seed {
        RegExpPatternSeed::Text(text) => Ok(text),
        RegExpPatternSeed::Value(value) => cx.value_to_string_text(value),
    }
}

fn regexp_flags_seed_text<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    seed: RegExpFlagsSeed,
) -> Result<String, Cx::Error> {
    match seed {
        RegExpFlagsSeed::Text(text) => Ok(text),
        RegExpFlagsSeed::Value(value) => cx.value_to_string_text(value),
    }
}

pub(super) fn regexp_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let realm = cx.builtin_realm();
    let default_prototype = {
        let agent = cx.agent();
        agent
            .realm(realm)
            .and_then(|record| record.intrinsics().regexp_prototype())
    }
    .ok_or_else(|| type_error(cx))?;
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

    let pattern_is_regexp = is_regexp_value(cx, pattern_value)?;
    if pattern_is_regexp && flags_value.is_undefined() && invocation.new_target().is_none() {
        let object_ref = pattern_value
            .as_object_ref()
            .ok_or_else(|| type_error(cx))?;
        let active_constructor = {
            let agent = cx.agent();
            agent
                .realm(realm)
                .and_then(|record| record.intrinsics().regexp())
        };
        let constructor = cx.get_property_value(
            Value::from_object_ref(object_ref),
            PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        )?;
        if constructor.as_object_ref() == active_constructor {
            return Ok(Value::from_object_ref(object_ref));
        }
    }

    let pattern_object = pattern_value.as_object_ref();
    let pattern_has_regexp_slots =
        pattern_object.is_some_and(|object| is_regexp_object(cx, object));
    let (pattern_seed, flags_seed) = if pattern_has_regexp_slots {
        let object = pattern_object.ok_or_else(|| type_error(cx))?;
        let (source, flags) = regexp_object_source_and_flags(cx, object)?;
        let flags_seed = if flags_value.is_undefined() {
            RegExpFlagsSeed::Text(flags)
        } else {
            RegExpFlagsSeed::Value(flags_value)
        };
        (RegExpPatternSeed::Text(source), flags_seed)
    } else {
        let pattern_seed = if pattern_is_regexp {
            let source_key = {
                let agent = cx.agent();
                PropertyKey::from_atom(agent.bootstrap_atoms().source())
            };
            RegExpPatternSeed::Value(cx.get_property_value(pattern_value, source_key)?)
        } else if pattern_value.is_undefined() {
            RegExpPatternSeed::Text(String::new())
        } else {
            RegExpPatternSeed::Value(pattern_value)
        };
        let flags_seed = if flags_value.is_undefined() {
            if pattern_is_regexp {
                let flags_key = {
                    let agent = cx.agent();
                    PropertyKey::from_atom(agent.bootstrap_atoms().flags())
                };
                RegExpFlagsSeed::Value(cx.get_property_value(pattern_value, flags_key)?)
            } else {
                RegExpFlagsSeed::Text(String::new())
            }
        } else {
            RegExpFlagsSeed::Value(flags_value)
        };
        (pattern_seed, flags_seed)
    };

    let prototype = if invocation.new_target().is_some() {
        cx.ordinary_constructor_prototype(realm, invocation.new_target(), default_prototype)?
    } else {
        default_prototype
    };
    let flags_text = regexp_flags_seed_text(cx, flags_seed)?;
    let pattern_text = regexp_pattern_seed_text(cx, pattern_seed)?;
    let pattern_text = normalize_regexp_constructor_pattern_text(
        &pattern_text,
        flags_text.contains('u') || flags_text.contains('v'),
    );
    if validate_regexp_constructor_pattern(&pattern_text, &flags_text).is_err() {
        return Err(syntax_error(cx));
    }

    let regexp = allocate_regexp_object(cx, realm, prototype, &pattern_text, &flags_text)?;
    Ok(Value::from_object_ref(regexp))
}
