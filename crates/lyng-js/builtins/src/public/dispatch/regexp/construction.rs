use super::super::{syntax_error, type_error, PublicBuiltinDispatchContext};
use super::{allocate_regexp_object, is_regexp_value};
use crate::BuiltinInvocation;
use lyng_js_common::WellKnownAtom;
use lyng_js_parser::validate_regexp_constructor_pattern;
use lyng_js_types::{PropertyKey, Value};

pub(super) fn regexp_species_getter_builtin(invocation: BuiltinInvocation<'_>) -> Value {
    invocation.this_value()
}

pub(super) fn normalize_regexp_constructor_pattern_text(pattern: &str) -> String {
    let mut normalized = String::with_capacity(pattern.len());
    let mut trailing_backslashes = 0usize;
    for ch in pattern.chars() {
        match ch {
            '\n' if trailing_backslashes % 2 == 0 => normalized.push_str("\\n"),
            '\r' if trailing_backslashes % 2 == 0 => normalized.push_str("\\r"),
            '\u{2028}' if trailing_backslashes % 2 == 0 => normalized.push_str("\\u2028"),
            '\u{2029}' if trailing_backslashes % 2 == 0 => normalized.push_str("\\u2029"),
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
    if pattern_is_regexp {
        if flags_value.is_undefined() && invocation.new_target().is_none() {
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
    }

    let pattern_text = if pattern_is_regexp {
        let source_key = {
            let agent = cx.agent();
            PropertyKey::from_atom(agent.bootstrap_atoms().source())
        };
        let source_value = cx.get_property_value(pattern_value, source_key)?;
        normalize_regexp_constructor_pattern_text(&cx.value_to_string_text(source_value)?)
    } else if pattern_value.is_undefined() {
        String::new()
    } else {
        normalize_regexp_constructor_pattern_text(&cx.value_to_string_text(pattern_value)?)
    };
    let flags_text = if flags_value.is_undefined() {
        if pattern_is_regexp {
            let flags_key = {
                let agent = cx.agent();
                PropertyKey::from_atom(agent.bootstrap_atoms().flags())
            };
            let flags_value = cx.get_property_value(pattern_value, flags_key)?;
            cx.value_to_string_text(flags_value)?
        } else {
            String::new()
        }
    } else {
        cx.value_to_string_text(flags_value)?
    };
    if validate_regexp_constructor_pattern(&pattern_text, &flags_text).is_err() {
        return Err(syntax_error(cx));
    }

    let prototype = if invocation.new_target().is_some() {
        cx.ordinary_constructor_prototype(realm, invocation.new_target(), default_prototype)?
    } else {
        default_prototype
    };
    let regexp = allocate_regexp_object(cx, realm, prototype, &pattern_text, &flags_text)?;
    Ok(Value::from_object_ref(regexp))
}
