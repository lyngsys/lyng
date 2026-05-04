use super::super::{
    string_from_code_units, string_value, type_error, PublicBuiltinDispatchContext,
};
use super::{boolean_property_value, current_intrinsic_regexp_prototype};
use crate::BuiltinInvocation;
use lyng_js_types::{PropertyKey, Value};

pub(super) fn regexp_flag_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    flag: char,
) -> Result<Value, Cx::Error> {
    let object_ref = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let flags = {
        let agent = cx.agent();
        agent
            .objects()
            .regexp_payload(object_ref)
            .map(lyng_js_objects::RegExpPayload::flags)
    };
    let Some(flags) = flags else {
        if current_intrinsic_regexp_prototype(cx) == Some(object_ref) {
            return Ok(Value::undefined());
        }
        return Err(type_error(cx));
    };
    let value = match flag {
        'd' => flags.has_indices(),
        'g' => flags.global(),
        'i' => flags.ignore_case(),
        'm' => flags.multiline(),
        's' => flags.dot_all(),
        'u' => flags.unicode(),
        'v' => flags.unicode_sets(),
        'y' => flags.sticky(),
        _ => false,
    };
    Ok(Value::from_bool(value))
}

enum RegExpSource {
    Text(String),
    Units(Vec<u16>),
}

fn escape_regexp_pattern_units(units: &[u16]) -> Vec<u16> {
    let mut escaped = Vec::with_capacity(units.len());
    let mut trailing_backslashes = 0usize;
    for unit in units {
        match *unit {
            0x002F if trailing_backslashes % 2 == 0 => {
                escaped.extend_from_slice(&[b'\\' as u16, b'/' as u16]);
                trailing_backslashes = 0;
            }
            0x002F => {
                escaped.push(b'/' as u16);
                trailing_backslashes = 0;
            }
            0x000A => escaped.extend_from_slice(&[b'\\' as u16, b'n' as u16]),
            0x000D => escaped.extend_from_slice(&[b'\\' as u16, b'r' as u16]),
            0x2028 => escaped.extend_from_slice(&[
                b'\\' as u16,
                b'u' as u16,
                b'2' as u16,
                b'0' as u16,
                b'2' as u16,
                b'8' as u16,
            ]),
            0x2029 => escaped.extend_from_slice(&[
                b'\\' as u16,
                b'u' as u16,
                b'2' as u16,
                b'0' as u16,
                b'2' as u16,
                b'9' as u16,
            ]),
            unit => escaped.push(unit),
        }
        if *unit == b'\\' as u16 {
            trailing_backslashes += 1;
        } else if *unit != 0x002F {
            trailing_backslashes = 0;
        }
    }
    escaped
}

fn escape_regexp_pattern_text(source: &str) -> RegExpSource {
    RegExpSource::Units(escape_regexp_pattern_units(
        &source.encode_utf16().collect::<Vec<_>>(),
    ))
}

pub(super) fn regexp_source_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let object_ref = invocation
        .this_value()
        .as_object_ref()
        .ok_or_else(|| type_error(cx))?;
    let source = {
        let agent = cx.agent();
        agent.objects().regexp_payload(object_ref).map(|payload| {
            if let Some(units) = payload.source_units() {
                if units.is_empty() {
                    RegExpSource::Text("(?:)".to_owned())
                } else {
                    RegExpSource::Units(escape_regexp_pattern_units(units))
                }
            } else if payload.source().is_empty() {
                RegExpSource::Text("(?:)".to_owned())
            } else {
                escape_regexp_pattern_text(payload.source())
            }
        })
    };
    if let Some(source) = source {
        return Ok(match source {
            RegExpSource::Text(source) => string_value(cx, &source),
            RegExpSource::Units(units) => string_from_code_units(cx, &units),
        });
    }
    if current_intrinsic_regexp_prototype(cx) == Some(object_ref) {
        return Ok(string_value(cx, "(?:)"));
    }
    Err(type_error(cx))
}

pub(super) fn regexp_flags_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let receiver = Value::from_object_ref(
        invocation
            .this_value()
            .as_object_ref()
            .ok_or_else(|| type_error(cx))?,
    );
    let (
        global_key,
        ignore_case_key,
        multiline_key,
        dot_all_key,
        unicode_key,
        unicode_sets_key,
        sticky_key,
        has_indices_key,
    ) = {
        let agent = cx.agent();
        (
            PropertyKey::from_atom(agent.atoms_mut().intern_collectible("global")),
            PropertyKey::from_atom(agent.atoms_mut().intern_collectible("ignoreCase")),
            PropertyKey::from_atom(agent.atoms_mut().intern_collectible("multiline")),
            PropertyKey::from_atom(agent.atoms_mut().intern_collectible("dotAll")),
            PropertyKey::from_atom(agent.atoms_mut().intern_collectible("unicode")),
            PropertyKey::from_atom(agent.atoms_mut().intern_collectible("unicodeSets")),
            PropertyKey::from_atom(agent.atoms_mut().intern_collectible("sticky")),
            PropertyKey::from_atom(agent.bootstrap_atoms().has_indices()),
        )
    };
    let mut flags = String::with_capacity(8);
    let has_indices = boolean_property_value(cx, receiver, has_indices_key)?;
    if has_indices {
        flags.push('d');
    }
    let global = boolean_property_value(cx, receiver, global_key)?;
    if global {
        flags.push('g');
    }
    let ignore_case = boolean_property_value(cx, receiver, ignore_case_key)?;
    if ignore_case {
        flags.push('i');
    }
    let multiline = boolean_property_value(cx, receiver, multiline_key)?;
    if multiline {
        flags.push('m');
    }
    let dot_all = boolean_property_value(cx, receiver, dot_all_key)?;
    if dot_all {
        flags.push('s');
    }
    let unicode = boolean_property_value(cx, receiver, unicode_key)?;
    if unicode {
        flags.push('u');
    }
    let unicode_sets = if let Some(object_ref) = receiver.as_object_ref() {
        let payload_flags = {
            let agent = cx.agent();
            agent
                .objects()
                .regexp_payload(object_ref)
                .map(lyng_js_objects::RegExpPayload::flags)
        };
        if let Some(payload_flags) = payload_flags {
            payload_flags.unicode_sets()
        } else {
            boolean_property_value(cx, receiver, unicode_sets_key)?
        }
    } else {
        boolean_property_value(cx, receiver, unicode_sets_key)?
    };
    if unicode_sets {
        flags.push('v');
    }
    let sticky = boolean_property_value(cx, receiver, sticky_key)?;
    if sticky {
        flags.push('y');
    }
    Ok(string_value(cx, &flags))
}

pub(super) fn regexp_has_indices_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    regexp_flag_getter_builtin(cx, invocation, 'd')
}
