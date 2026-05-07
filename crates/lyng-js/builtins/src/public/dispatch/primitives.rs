mod bigint;
mod math;

use super::{
    format_to_exponential, format_to_precision, is_integral_number, map_completion,
    number_to_i32_after_range_check, primitive_wrapper_constructor, radix_argument, range_error,
    string_value, symbol_descriptive_string, to_integer_or_infinity_for_builtin, type_error,
    BuiltinToPrimitiveBridge, PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use bigint::{bigint_to_number_value, dispatch_bigint_builtin};
use lyng_js_gc::{AllocationLifetime, SymbolFlags};
use lyng_js_objects::PrimitiveWrapperKind;
use lyng_js_ops::{object, read};
use lyng_js_types::{BuiltinFunctionId, Value};
use math::dispatch_math_builtin;

pub(super) fn dispatch_primitive_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_number_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_math_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_bigint_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_boolean_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_symbol_builtin(context, entry, invocation)
}

fn dispatch_number_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::number_builtin() {
        return number_builtin(context, invocation).map(Some);
    }
    if entry == super::number_is_finite_builtin() {
        return Ok(Some(number_is_finite_builtin(invocation)));
    }
    if entry == super::number_is_integer_builtin() {
        return Ok(Some(number_is_integer_builtin(invocation)));
    }
    if entry == super::number_is_nan_builtin() {
        return Ok(Some(number_is_nan_builtin(invocation)));
    }
    if entry == super::number_is_safe_integer_builtin() {
        return Ok(Some(number_is_safe_integer_builtin(invocation)));
    }
    if entry == super::number_to_exponential_builtin() {
        return number_to_exponential_builtin(context, invocation).map(Some);
    }
    if entry == super::number_to_fixed_builtin() {
        return number_to_fixed_builtin(context, invocation).map(Some);
    }
    if entry == super::number_to_locale_string_builtin() {
        return number_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == super::number_to_precision_builtin() {
        return number_to_precision_builtin(context, invocation).map(Some);
    }
    if entry == super::number_to_string_builtin() {
        return number_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::number_value_of_builtin() {
        return number_value_of_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_boolean_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::boolean_builtin() {
        return boolean_builtin(context, invocation).map(Some);
    }
    if entry == super::boolean_to_string_builtin() {
        return boolean_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::boolean_value_of_builtin() {
        return boolean_value_of_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_symbol_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::symbol_builtin() {
        return symbol_builtin(context, invocation).map(Some);
    }
    if entry == super::symbol_for_builtin() {
        return symbol_for_builtin(context, invocation).map(Some);
    }
    if entry == super::symbol_key_for_builtin() {
        return symbol_key_for_builtin(context, invocation).map(Some);
    }
    if entry == super::symbol_to_string_builtin() {
        return symbol_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::symbol_value_of_builtin() {
        return symbol_value_of_builtin(context, invocation).map(Some);
    }
    if entry == super::symbol_to_primitive_builtin() {
        return symbol_to_primitive_builtin(context, invocation).map(Some);
    }
    if entry == super::symbol_description_getter_builtin() {
        return symbol_description_getter_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn number_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let number = if let Some(argument) = invocation.arguments().first().copied() {
        let primitive = {
            let mut bridge = BuiltinToPrimitiveBridge { cx };
            object::to_primitive(&mut bridge, argument, object::ToPrimitiveHint::Number)?
        };
        if primitive.is_bigint() {
            bigint_to_number_value(cx, primitive)?
        } else {
            let number = {
                let agent = cx.agent();
                read::to_number(agent.heap().view(), primitive)
            };
            match number {
                Ok(number) => number,
                Err(_) => return Err(type_error(cx)),
            }
        }
    } else {
        Value::from_smi(0)
    };
    if invocation.new_target().is_none() {
        return Ok(number);
    }
    let realm = cx.builtin_realm();
    let default_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().number_prototype())
        .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, invocation.new_target(), default_prototype)?;
    primitive_wrapper_constructor(cx, realm, prototype, PrimitiveWrapperKind::Number, number)
}

fn number_is_finite_builtin(invocation: BuiltinInvocation<'_>) -> Value {
    let result = invocation
        .arguments()
        .first()
        .copied()
        .and_then(Value::as_f64)
        .is_some_and(f64::is_finite);
    Value::from_bool(result)
}

fn number_is_integer_builtin(invocation: BuiltinInvocation<'_>) -> Value {
    let result = invocation
        .arguments()
        .first()
        .copied()
        .and_then(Value::as_f64)
        .is_some_and(is_integral_number);
    Value::from_bool(result)
}

fn number_is_nan_builtin(invocation: BuiltinInvocation<'_>) -> Value {
    let result = invocation
        .arguments()
        .first()
        .copied()
        .and_then(Value::as_f64)
        .is_some_and(f64::is_nan);
    Value::from_bool(result)
}

fn number_is_safe_integer_builtin(invocation: BuiltinInvocation<'_>) -> Value {
    let result = invocation
        .arguments()
        .first()
        .copied()
        .and_then(Value::as_f64)
        .is_some_and(|number| {
            is_integral_number(number) && number.abs() <= 9_007_199_254_740_991.0
        });
    Value::from_bool(result)
}

fn number_this_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: &BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::Number,
        )
    };
    map_completion(cx, value)
}

fn number_this_f64<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: &BuiltinInvocation<'_>,
) -> Result<(Value, f64), Cx::Error> {
    let value = number_this_value(cx, invocation)?;
    let number = value.as_f64().ok_or_else(|| type_error(cx))?;
    Ok((value, number))
}

fn number_format_digits<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
    min: i32,
    max: i32,
) -> Result<usize, Cx::Error> {
    let digits = to_integer_or_infinity_for_builtin(cx, value)?;
    if !digits.is_finite() || digits < f64::from(min) || digits > f64::from(max) {
        return Err(range_error(cx));
    }
    usize::try_from(number_to_i32_after_range_check(digits)).map_err(|_| range_error(cx))
}

fn number_to_exponential_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (number_value, number) = number_this_f64(cx, &invocation)?;
    let fraction_digits = match invocation.arguments().first().copied() {
        None => None,
        Some(value) if value.is_undefined() => None,
        Some(value) => Some(to_integer_or_infinity_for_builtin(cx, value)?),
    };
    if number.is_nan() || number.is_infinite() {
        let text = cx.value_to_string_text(number_value)?;
        return Ok(string_value(cx, &text));
    }
    let fraction_digits = match fraction_digits {
        None => None,
        Some(digits) if digits.is_finite() && (0.0..=100.0).contains(&digits) => Some(
            usize::try_from(number_to_i32_after_range_check(digits))
                .map_err(|_| range_error(cx))?,
        ),
        Some(_) => return Err(range_error(cx)),
    };
    let normalized = if number == 0.0 { 0.0 } else { number };
    let formatted = if let Some(fraction_digits) = fraction_digits {
        format_to_exponential(normalized, fraction_digits).ok_or_else(|| type_error(cx))?
    } else {
        format!("{normalized:e}")
    };
    let (mantissa, exponent) = formatted.split_once('e').ok_or_else(|| type_error(cx))?;
    let exponent = exponent.parse::<i32>().map_err(|_| type_error(cx))?;
    Ok(string_value(cx, &format!("{mantissa}e{exponent:+}")))
}

fn number_to_fixed_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (number_value, number) = number_this_f64(cx, &invocation)?;
    let fraction_digits = match invocation.arguments().first().copied() {
        None => 0,
        Some(value) => number_format_digits(cx, value, 0, 100)?,
    };
    if number.is_nan() || number.is_infinite() || number.abs() >= 1e21 {
        let text = cx.value_to_string_text(number_value)?;
        return Ok(string_value(cx, &text));
    }
    let normalized = if number == 0.0 { 0.0 } else { number };
    Ok(string_value(cx, &format!("{normalized:.fraction_digits$}")))
}

fn number_to_locale_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = number_this_value(cx, &invocation)?;
    let text = cx.value_to_string_text(value)?;
    Ok(string_value(cx, &text))
}

fn number_to_precision_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let (number_value, number) = number_this_f64(cx, &invocation)?;
    let Some(precision_value) = invocation.arguments().first().copied() else {
        let text = cx.value_to_string_text(number_value)?;
        return Ok(string_value(cx, &text));
    };
    if precision_value.is_undefined() {
        let text = cx.value_to_string_text(number_value)?;
        return Ok(string_value(cx, &text));
    }

    let precision_integer = to_integer_or_infinity_for_builtin(cx, precision_value)?;
    if number.is_nan() || number.is_infinite() {
        let text = cx.value_to_string_text(number_value)?;
        return Ok(string_value(cx, &text));
    }
    if !precision_integer.is_finite() || !(1.0..=100.0).contains(&precision_integer) {
        return Err(range_error(cx));
    }
    let precision = usize::try_from(number_to_i32_after_range_check(precision_integer))
        .map_err(|_| range_error(cx))?;
    let text = format_to_precision(number, precision).ok_or_else(|| type_error(cx))?;
    Ok(string_value(cx, &text))
}

fn number_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let number_value = number_this_value(cx, &invocation)?;
    let number = number_value.as_f64().ok_or_else(|| type_error(cx))?;
    let radix = radix_argument(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let text = if radix == 10 || !number.is_finite() {
        cx.value_to_string_text(number_value)?
    } else {
        object::integral_number_to_radix_string(number, radix).unwrap_or_else(|| number.to_string())
    };
    Ok(string_value(cx, &text))
}

fn number_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::Number,
        )
    };
    map_completion(cx, value)
}

fn boolean_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let argument = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let boolean = {
        let agent = cx.agent();
        read::to_boolean_agent(agent, argument)
    };
    let boolean = map_completion(cx, boolean)?;
    if invocation.new_target().is_none() {
        return Ok(Value::from_bool(boolean));
    }
    let realm = cx.builtin_realm();
    let default_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().boolean_prototype())
        .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, invocation.new_target(), default_prototype)?;
    primitive_wrapper_constructor(
        cx,
        realm,
        prototype,
        PrimitiveWrapperKind::Boolean,
        Value::from_bool(boolean),
    )
}

fn boolean_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::Boolean,
        )
    };
    let value = map_completion(cx, value)?;
    Ok(string_value(
        cx,
        if value.as_bool() == Some(true) {
            "true"
        } else {
            "false"
        },
    ))
}

fn boolean_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::Boolean,
        )
    };
    map_completion(cx, value)
}

fn symbol_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    if invocation.new_target().is_some() {
        return Err(type_error(cx));
    }
    let description = match invocation.arguments().first().copied() {
        Some(value) if !value.is_undefined() => {
            let text = cx.value_to_string_text(value)?;
            let description = {
                let agent = cx.agent();
                agent.alloc_runtime_string(&text, None, AllocationLifetime::Default)
            };
            Some(description)
        }
        _ => None,
    };
    let symbol = {
        let agent = cx.agent();
        agent.heap_mut().mutator().alloc_symbol(
            description,
            SymbolFlags::ordinary(),
            AllocationLifetime::Default,
        )
    };
    Ok(Value::from_symbol_ref(symbol))
}

fn symbol_for_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let key_text = cx.value_to_string_text(
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let symbol = {
        let agent = cx.agent();
        let key = agent.atoms_mut().intern_collectible(&key_text);
        agent.global_symbol_for(key, AllocationLifetime::Default)
    };
    Ok(Value::from_symbol_ref(symbol))
}

fn symbol_key_for_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let symbol = invocation
        .arguments()
        .first()
        .copied()
        .and_then(Value::as_symbol_ref)
        .ok_or_else(|| type_error(cx))?;
    let Some(key) = ({
        let agent = cx.agent();
        agent.global_symbol_key_for(symbol)
    }) else {
        return Ok(Value::undefined());
    };
    let value = {
        let agent = cx.agent();
        Value::from_string_ref(agent.alloc_runtime_string(
            "",
            Some(key),
            AllocationLifetime::Default,
        ))
    };
    Ok(value)
}

fn symbol_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::Symbol,
        )
    };
    let symbol = map_completion(cx, value)?
        .as_symbol_ref()
        .ok_or_else(|| type_error(cx))?;
    let text = symbol_descriptive_string(cx, symbol)?;
    Ok(string_value(cx, &text))
}

fn symbol_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::Symbol,
        )
    };
    map_completion(cx, value)
}

fn symbol_description_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::Symbol,
        )
    };
    let symbol = map_completion(cx, value)?
        .as_symbol_ref()
        .ok_or_else(|| type_error(cx))?;
    let description = {
        let agent = cx.agent();
        let heap_view = agent.heap().view();
        heap_view
            .symbol_view(symbol)
            .and_then(lyng_js_gc::PrimitiveSymbolView::description)
            .map_or(Value::undefined(), Value::from_string_ref)
    };
    Ok(description)
}

fn symbol_to_primitive_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    symbol_value_of_builtin(cx, invocation)
}
