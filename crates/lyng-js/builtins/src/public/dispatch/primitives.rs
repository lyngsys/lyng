use super::{
    argument_to_number, close_iterator_after_error, format_to_exponential, format_to_precision,
    is_integral_number, map_completion, number_value, primitive_wrapper_constructor,
    radix_argument, range_error, string_value, symbol_descriptive_string, to_bigint_for_builtin,
    to_index_for_builtin, to_integer_or_infinity_for_builtin, to_uint32_for_builtin, type_error,
    BuiltinIteratorBridge, BuiltinToPrimitiveBridge, PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_env::Agent;
use lyng_js_gc::{AllocationLifetime, BigIntSign, SymbolFlags};
use lyng_js_objects::PrimitiveWrapperKind;
use lyng_js_ops::{iterator, object, read};
use lyng_js_types::{BuiltinFunctionId, Value};
use std::time::{SystemTime, UNIX_EPOCH};

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

fn dispatch_math_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_math_basic_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_math_advanced_builtin(context, entry, invocation)
}

fn dispatch_math_basic_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::math_abs_builtin() {
        return math_abs_builtin(context, invocation).map(Some);
    }
    if entry == super::math_acos_builtin() {
        return math_acos_builtin(context, invocation).map(Some);
    }
    if entry == super::math_acosh_builtin() {
        return math_acosh_builtin(context, invocation).map(Some);
    }
    if entry == super::math_asin_builtin() {
        return math_asin_builtin(context, invocation).map(Some);
    }
    if entry == super::math_asinh_builtin() {
        return math_asinh_builtin(context, invocation).map(Some);
    }
    if entry == super::math_atan_builtin() {
        return math_atan_builtin(context, invocation).map(Some);
    }
    if entry == super::math_atan2_builtin() {
        return math_atan2_builtin(context, invocation).map(Some);
    }
    if entry == super::math_atanh_builtin() {
        return math_atanh_builtin(context, invocation).map(Some);
    }
    if entry == super::math_cbrt_builtin() {
        return math_cbrt_builtin(context, invocation).map(Some);
    }
    if entry == super::math_ceil_builtin() {
        return math_ceil_builtin(context, invocation).map(Some);
    }
    if entry == super::math_clz32_builtin() {
        return math_clz32_builtin(context, invocation).map(Some);
    }
    if entry == super::math_cos_builtin() {
        return math_cos_builtin(context, invocation).map(Some);
    }
    if entry == super::math_cosh_builtin() {
        return math_cosh_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_math_advanced_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::math_exp_builtin() {
        return math_exp_builtin(context, invocation).map(Some);
    }
    if entry == super::math_expm1_builtin() {
        return math_expm1_builtin(context, invocation).map(Some);
    }
    if entry == super::math_f16round_builtin() {
        return math_f16round_builtin(context, invocation).map(Some);
    }
    if entry == super::math_floor_builtin() {
        return math_floor_builtin(context, invocation).map(Some);
    }
    if entry == super::math_fround_builtin() {
        return math_fround_builtin(context, invocation).map(Some);
    }
    if entry == super::math_hypot_builtin() {
        return math_hypot_builtin(context, invocation).map(Some);
    }
    if entry == super::math_imul_builtin() {
        return math_imul_builtin(context, invocation).map(Some);
    }
    if entry == super::math_log_builtin() {
        return math_log_builtin(context, invocation).map(Some);
    }
    if entry == super::math_log10_builtin() {
        return math_log10_builtin(context, invocation).map(Some);
    }
    if entry == super::math_log1p_builtin() {
        return math_log1p_builtin(context, invocation).map(Some);
    }
    if entry == super::math_log2_builtin() {
        return math_log2_builtin(context, invocation).map(Some);
    }
    if entry == super::math_max_builtin() {
        return math_max_builtin(context, invocation).map(Some);
    }
    if entry == super::math_min_builtin() {
        return math_min_builtin(context, invocation).map(Some);
    }
    if entry == super::math_pow_builtin() {
        return math_pow_builtin(context, invocation).map(Some);
    }
    if entry == super::math_random_builtin() {
        return Ok(Some(math_random_builtin()));
    }
    if entry == super::math_round_builtin() {
        return math_round_builtin(context, invocation).map(Some);
    }
    if entry == super::math_sign_builtin() {
        return math_sign_builtin(context, invocation).map(Some);
    }
    if entry == super::math_sin_builtin() {
        return math_sin_builtin(context, invocation).map(Some);
    }
    if entry == super::math_sinh_builtin() {
        return math_sinh_builtin(context, invocation).map(Some);
    }
    if entry == super::math_sqrt_builtin() {
        return math_sqrt_builtin(context, invocation).map(Some);
    }
    if entry == super::math_sum_precise_builtin() {
        return math_sum_precise_builtin(context, invocation).map(Some);
    }
    if entry == super::math_tan_builtin() {
        return math_tan_builtin(context, invocation).map(Some);
    }
    if entry == super::math_tanh_builtin() {
        return math_tanh_builtin(context, invocation).map(Some);
    }
    if entry == super::math_trunc_builtin() {
        return math_trunc_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_bigint_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::bigint_builtin() {
        return bigint_builtin(context, invocation).map(Some);
    }
    if entry == super::bigint_as_int_n_builtin() {
        return bigint_as_int_n_builtin(context, invocation).map(Some);
    }
    if entry == super::bigint_as_uint_n_builtin() {
        return bigint_as_uint_n_builtin(context, invocation).map(Some);
    }
    if entry == super::bigint_to_string_builtin() {
        return bigint_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::bigint_value_of_builtin() {
        return bigint_value_of_builtin(context, invocation).map(Some);
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

fn bigint_to_number_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Value, Cx::Error> {
    let text = {
        let agent = cx.agent();
        object::bigint_to_string(agent, value, 10)
    };
    let text = map_completion(cx, text)?;
    let number = text.parse::<f64>().unwrap_or_else(|_| {
        if text.starts_with('-') {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        }
    });
    Ok(Value::from_f64(number))
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
    usize::try_from(digits as i32).map_err(|_| range_error(cx))
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
        Some(digits) if digits.is_finite() && (0.0..=100.0).contains(&digits) => {
            Some(usize::try_from(digits as i32).map_err(|_| range_error(cx))?)
        }
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
    let precision = usize::try_from(precision_integer as i32).map_err(|_| range_error(cx))?;
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

fn math_argument(invocation: &BuiltinInvocation<'_>, index: usize) -> Value {
    invocation
        .arguments()
        .get(index)
        .copied()
        .unwrap_or(Value::undefined())
}

fn math_unary_number_builtin<Cx, F>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
    op: F,
) -> Result<Value, Cx::Error>
where
    Cx: PublicBuiltinDispatchContext,
    F: FnOnce(f64) -> f64,
{
    let number = argument_to_number(cx, math_argument(&invocation, 0))?;
    Ok(number_value(op(number)))
}

fn math_abs_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::abs)
}

fn math_acos_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::acos)
}

fn math_acosh_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::acosh)
}

fn math_asin_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::asin)
}

fn math_asinh_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::asinh)
}

fn math_atan_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::atan)
}

fn math_atan2_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let y = argument_to_number(cx, math_argument(&invocation, 0))?;
    let x = argument_to_number(cx, math_argument(&invocation, 1))?;
    Ok(number_value(y.atan2(x)))
}

fn math_atanh_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::atanh)
}

fn math_cbrt_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::cbrt)
}

fn math_ceil_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::ceil)
}

fn math_clz32_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = to_uint32_for_builtin(cx, math_argument(&invocation, 0))?;
    Ok(Value::from_smi(
        i32::try_from(value.leading_zeros()).expect("leading zero count should fit in i32"),
    ))
}

fn math_cos_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::cos)
}

fn math_cosh_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::cosh)
}

fn math_exp_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::exp)
}

fn math_expm1_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::exp_m1)
}

fn math_f16round_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let number = argument_to_number(cx, math_argument(&invocation, 0))?;
    Ok(number_value(round_to_float16(number)))
}

fn math_floor_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::floor)
}

fn math_fround_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let number = argument_to_number(cx, math_argument(&invocation, 0))?;
    Ok(number_value(f64::from(number as f32)))
}

fn math_hypot_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let mut values = Vec::with_capacity(invocation.arguments().len());
    let mut saw_infinity = false;
    let mut saw_nan = false;
    for argument in invocation.arguments() {
        let number = argument_to_number(cx, *argument)?;
        if number.is_infinite() {
            saw_infinity = true;
        } else if number.is_nan() {
            saw_nan = true;
        } else {
            values.push(number.abs());
        }
    }
    if saw_infinity {
        return Ok(Value::from_f64(f64::INFINITY));
    }
    if saw_nan {
        return Ok(Value::from_f64(f64::NAN));
    }
    let scale = values.iter().copied().fold(0.0_f64, f64::max);
    if scale == 0.0 {
        return Ok(Value::from_smi(0));
    }
    let scaled_sum = values
        .iter()
        .map(|value| {
            let scaled = *value / scale;
            scaled * scaled
        })
        .sum::<f64>();
    Ok(number_value(scale * scaled_sum.sqrt()))
}

fn math_imul_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let left = to_uint32_for_builtin(cx, math_argument(&invocation, 0))?;
    let right = to_uint32_for_builtin(cx, math_argument(&invocation, 1))?;
    Ok(Value::from_smi(left.wrapping_mul(right) as i32))
}

fn math_log_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::ln)
}

fn math_log10_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::log10)
}

fn math_log1p_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::ln_1p)
}

fn math_log2_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::log2)
}

fn math_max_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    if invocation.arguments().is_empty() {
        return Ok(Value::from_f64(f64::NEG_INFINITY));
    }
    let mut result = f64::NEG_INFINITY;
    let mut saw_nan = false;
    for argument in invocation.arguments() {
        let number = argument_to_number(cx, *argument)?;
        if number.is_nan() {
            saw_nan = true;
            continue;
        }
        if number > result
            || (number == 0.0
                && result == 0.0
                && result.is_sign_negative()
                && number.is_sign_positive())
        {
            result = number;
        }
    }
    if saw_nan {
        return Ok(Value::from_f64(f64::NAN));
    }
    Ok(number_value(result))
}

fn math_min_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    if invocation.arguments().is_empty() {
        return Ok(Value::from_f64(f64::INFINITY));
    }
    let mut result = f64::INFINITY;
    let mut saw_nan = false;
    for argument in invocation.arguments() {
        let number = argument_to_number(cx, *argument)?;
        if number.is_nan() {
            saw_nan = true;
            continue;
        }
        if number < result
            || (number == 0.0
                && result == 0.0
                && result.is_sign_positive()
                && number.is_sign_negative())
        {
            result = number;
        }
    }
    if saw_nan {
        return Ok(Value::from_f64(f64::NAN));
    }
    Ok(number_value(result))
}

#[allow(
    clippy::float_cmp,
    reason = "Math.pow has an exact |base| == 1 infinite-exponent special case."
)]
fn math_pow_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let base = argument_to_number(cx, math_argument(&invocation, 0))?;
    let exponent = argument_to_number(cx, math_argument(&invocation, 1))?;
    if exponent.is_nan() || (base.abs() == 1.0 && exponent.is_infinite()) {
        return Ok(Value::from_f64(f64::NAN));
    }
    Ok(number_value(base.powf(exponent)))
}

fn math_random_builtin() -> Value {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0);
    let mut mixed = seed ^ seed.rotate_left(25) ^ 0x9e37_79b9_7f4a_7c15;
    mixed ^= mixed >> 30;
    mixed = mixed.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    mixed ^= mixed >> 27;
    mixed = mixed.wrapping_mul(0x94d0_49bb_1331_11eb);
    mixed ^= mixed >> 31;
    let mantissa = mixed >> 11;
    Value::from_f64(mantissa as f64 / ((1_u64 << 53) as f64))
}

#[allow(
    clippy::manual_range_contains,
    reason = "Math.round needs the exact [-0.5, 0) interval to preserve negative zero."
)]
fn math_round_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let number = argument_to_number(cx, math_argument(&invocation, 0))?;
    if !number.is_finite() || number == 0.0 {
        return Ok(number_value(number));
    }
    if number < 0.0 && number >= -0.5 {
        return Ok(Value::from_f64(-0.0));
    }
    let floor = number.floor();
    if number - floor < 0.5 {
        Ok(number_value(floor))
    } else {
        Ok(number_value(floor + 1.0))
    }
}

fn math_sign_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let number = argument_to_number(cx, math_argument(&invocation, 0))?;
    if number.is_nan() || number == 0.0 {
        return Ok(number_value(number));
    }
    Ok(number_value(if number.is_sign_negative() {
        -1.0
    } else {
        1.0
    }))
}

fn math_sin_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::sin)
}

fn math_sinh_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::sinh)
}

fn math_sqrt_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::sqrt)
}

fn math_sum_precise_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let iterable = math_argument(&invocation, 0);
    let mut iterator_record = {
        let mut bridge = BuiltinIteratorBridge { cx };
        iterator::get_iterator(&mut bridge, iterable)?
    };
    let mut numbers = Vec::new();
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
            return Ok(number_value(math_sum_precise_numbers(&numbers)));
        };
        let value = {
            let mut bridge = BuiltinIteratorBridge { cx };
            iterator::iterator_value(&mut bridge, next)
        };
        let value = match value {
            Ok(value) => value,
            Err(error) => return close_iterator_after_error(cx, &mut iterator_record, error),
        };
        let Some(number) = value.as_f64() else {
            let error = type_error(cx);
            return close_iterator_after_error(cx, &mut iterator_record, error);
        };
        numbers.push(number);
    }
}

fn math_tan_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::tan)
}

fn math_tanh_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::tanh)
}

fn math_trunc_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    math_unary_number_builtin(cx, invocation, f64::trunc)
}

fn round_to_float16(number: f64) -> f64 {
    const MIN_SUBNORMAL: f64 = 5.960_464_477_539_063e-8;
    const MIN_NORMAL: f64 = 0.000_061_035_156_25;
    const MAX_FINITE: f64 = 65_504.0;
    const INFINITY_THRESHOLD: f64 = 65_520.0;

    if number.is_nan() || number == 0.0 || number.is_infinite() {
        return number;
    }

    let negative = number.is_sign_negative();
    let magnitude = number.abs();
    let rounded = if magnitude >= INFINITY_THRESHOLD {
        f64::INFINITY
    } else if magnitude < MIN_NORMAL {
        round_ties_to_even(magnitude / MIN_SUBNORMAL) * MIN_SUBNORMAL
    } else {
        let exponent = magnitude.log2().floor() as i32;
        let step = 2.0_f64.powi(exponent - 10);
        let candidate = round_ties_to_even(magnitude / step) * step;
        candidate.min(MAX_FINITE)
    };

    if negative {
        -rounded
    } else {
        rounded
    }
}

fn round_ties_to_even(value: f64) -> f64 {
    let floor = value.floor();
    let fraction = value - floor;
    if fraction < 0.5 {
        floor
    } else if fraction > 0.5 || (floor as u64) % 2 == 1 {
        floor + 1.0
    } else {
        floor
    }
}

fn math_sum_precise_numbers(numbers: &[f64]) -> f64 {
    let mut saw_nan = false;
    let mut saw_positive_infinity = false;
    let mut saw_negative_infinity = false;
    let mut saw_positive_zero = false;
    let mut saw_negative_zero = false;
    let mut finite = Vec::new();

    for number in numbers {
        if number.is_nan() {
            saw_nan = true;
        } else if *number == f64::INFINITY {
            saw_positive_infinity = true;
        } else if *number == f64::NEG_INFINITY {
            saw_negative_infinity = true;
        } else if *number == 0.0 {
            if number.is_sign_negative() {
                saw_negative_zero = true;
            } else {
                saw_positive_zero = true;
            }
        } else {
            finite.push(*number);
        }
    }

    if saw_nan || (saw_positive_infinity && saw_negative_infinity) {
        return f64::NAN;
    }
    if saw_positive_infinity {
        return f64::INFINITY;
    }
    if saw_negative_infinity {
        return f64::NEG_INFINITY;
    }
    if finite.is_empty() {
        if saw_positive_zero {
            return 0.0;
        }
        return if saw_negative_zero || numbers.is_empty() {
            -0.0
        } else {
            0.0
        };
    }

    let result = math_precise_finite_sum(&finite);
    if result == 0.0 {
        0.0
    } else {
        result
    }
}

fn math_precise_finite_sum(numbers: &[f64]) -> f64 {
    let mut terms = Vec::with_capacity(numbers.len());
    let mut min_exponent = 0;
    for number in numbers {
        if let Some(term) = MathFiniteTerm::from_f64(*number) {
            if terms.is_empty() || term.exponent < min_exponent {
                min_exponent = term.exponent;
            }
            terms.push(term);
        }
    }

    let mut exact = MathExactSum::default();
    for term in terms {
        exact.add_term(
            term.negative,
            term.mantissa,
            usize::try_from(term.exponent - min_exponent)
                .expect("finite term shift should be non-negative"),
        );
    }
    exact.to_f64(min_exponent)
}

struct MathFiniteTerm {
    negative: bool,
    mantissa: u64,
    exponent: i32,
}

impl MathFiniteTerm {
    fn from_f64(number: f64) -> Option<Self> {
        let bits = number.to_bits();
        let negative = (bits >> 63) != 0;
        let exponent_bits = ((bits >> 52) & 0x7ff) as u16;
        let fraction = bits & ((1_u64 << 52) - 1);
        if exponent_bits == 0 {
            if fraction == 0 {
                return None;
            }
            return Some(Self {
                negative,
                mantissa: fraction,
                exponent: -1074,
            });
        }

        Some(Self {
            negative,
            mantissa: (1_u64 << 52) | fraction,
            exponent: i32::from(exponent_bits) - 1075,
        })
    }
}

#[derive(Default)]
struct MathExactSum {
    sign: i8,
    limbs: Vec<u64>,
}

impl MathExactSum {
    fn add_term(&mut self, negative: bool, mantissa: u64, shift: usize) {
        let magnitude = shifted_magnitude(mantissa, shift);
        if magnitude.is_empty() {
            return;
        }
        let term_sign = if negative { -1 } else { 1 };
        if self.sign == 0 {
            self.sign = term_sign;
            self.limbs = magnitude;
            return;
        }
        if self.sign == term_sign {
            add_magnitude(&mut self.limbs, &magnitude);
            return;
        }

        match compare_magnitude(&self.limbs, &magnitude) {
            std::cmp::Ordering::Greater => {
                subtract_magnitude(&mut self.limbs, &magnitude);
            }
            std::cmp::Ordering::Less => {
                let mut replacement = magnitude;
                subtract_magnitude(&mut replacement, &self.limbs);
                self.limbs = replacement;
                self.sign = term_sign;
            }
            std::cmp::Ordering::Equal => {
                self.limbs.clear();
                self.sign = 0;
            }
        }
    }

    fn to_f64(&self, exponent: i32) -> f64 {
        if self.sign == 0 {
            return 0.0;
        }

        let bit_len = magnitude_bit_len(&self.limbs);
        let highest_exponent =
            exponent + i32::try_from(bit_len).expect("bit length should fit") - 1;
        let magnitude = if highest_exponent < -1022 {
            round_subnormal_magnitude_to_f64(&self.limbs, exponent)
        } else {
            round_normal_magnitude_to_f64(&self.limbs, bit_len, highest_exponent)
        };

        if self.sign < 0 {
            -magnitude
        } else {
            magnitude
        }
    }
}

fn shifted_magnitude(mantissa: u64, shift: usize) -> Vec<u64> {
    let limb_shift = shift / 64;
    let bit_shift = shift % 64;
    let mut limbs = vec![0; limb_shift];
    if bit_shift == 0 {
        limbs.push(mantissa);
    } else {
        limbs.push(mantissa << bit_shift);
        let high = mantissa >> (64 - bit_shift);
        if high != 0 {
            limbs.push(high);
        }
    }
    normalize_magnitude(&mut limbs);
    limbs
}

fn normalize_magnitude(limbs: &mut Vec<u64>) {
    while limbs.last().copied() == Some(0) {
        limbs.pop();
    }
}

fn compare_magnitude(left: &[u64], right: &[u64]) -> std::cmp::Ordering {
    if left.len() != right.len() {
        return left.len().cmp(&right.len());
    }
    for (left_limb, right_limb) in left.iter().zip(right.iter()).rev() {
        if left_limb != right_limb {
            return left_limb.cmp(right_limb);
        }
    }
    std::cmp::Ordering::Equal
}

fn add_magnitude(target: &mut Vec<u64>, addend: &[u64]) {
    if target.len() < addend.len() {
        target.resize(addend.len(), 0);
    }
    let mut carry = 0_u128;
    for (index, target_limb) in target.iter_mut().enumerate() {
        let addend_limb = addend.get(index).copied().unwrap_or(0);
        let sum = u128::from(*target_limb) + u128::from(addend_limb) + carry;
        *target_limb = sum as u64;
        carry = sum >> 64;
    }
    if carry != 0 {
        target.push(carry as u64);
    }
}

fn subtract_magnitude(target: &mut Vec<u64>, subtrahend: &[u64]) {
    let mut borrow = 0_i128;
    for (index, target_limb) in target.iter_mut().enumerate() {
        let left = i128::from(*target_limb) - borrow;
        let right = i128::from(subtrahend.get(index).copied().unwrap_or(0));
        if left >= right {
            *target_limb = (left - right) as u64;
            borrow = 0;
        } else {
            *target_limb = ((1_i128 << 64) + left - right) as u64;
            borrow = 1;
        }
    }
    debug_assert_eq!(borrow, 0);
    normalize_magnitude(target);
}

fn magnitude_bit_len(limbs: &[u64]) -> usize {
    let Some(last) = limbs.last() else {
        return 0;
    };
    (limbs.len() - 1) * 64
        + usize::try_from(64 - last.leading_zeros()).expect("bit count should fit in usize")
}

fn round_normal_magnitude_to_f64(limbs: &[u64], bit_len: usize, mut exponent: i32) -> f64 {
    if exponent > 1023 {
        return f64::INFINITY;
    }

    let mut significand = if bit_len > 53 {
        round_shift_to_u64(limbs, bit_len - 53)
    } else {
        magnitude_to_u64(limbs) << (53 - bit_len)
    };
    if significand == (1_u64 << 53) {
        significand >>= 1;
        exponent += 1;
    }
    if exponent > 1023 {
        return f64::INFINITY;
    }

    let exponent_bits = u64::try_from(exponent + 1023).expect("normal exponent should fit");
    let fraction = significand - (1_u64 << 52);
    f64::from_bits((exponent_bits << 52) | fraction)
}

fn round_subnormal_magnitude_to_f64(limbs: &[u64], exponent: i32) -> f64 {
    let scale = exponent + 1074;
    let fraction = if scale >= 0 {
        magnitude_to_u64(limbs) << usize::try_from(scale).expect("scale should fit")
    } else {
        round_shift_to_u64(
            limbs,
            usize::try_from(-scale).expect("right shift should fit"),
        )
    };
    if fraction == 0 {
        return 0.0;
    }
    if fraction >= (1_u64 << 52) {
        return f64::from_bits(1_u64 << 52);
    }
    f64::from_bits(fraction)
}

fn magnitude_to_u64(limbs: &[u64]) -> u64 {
    debug_assert!(limbs.iter().skip(1).all(|limb| *limb == 0));
    limbs.first().copied().unwrap_or(0)
}

fn round_shift_to_u64(limbs: &[u64], shift: usize) -> u64 {
    if shift == 0 {
        return magnitude_to_u64(limbs);
    }

    let mut quotient = magnitude_shr_to_u64(limbs, shift);
    let half = magnitude_bit(limbs, shift - 1);
    let sticky = magnitude_any_bits_below(limbs, shift - 1);
    if half && (sticky || quotient % 2 == 1) {
        quotient += 1;
    }
    quotient
}

fn magnitude_shr_to_u64(limbs: &[u64], shift: usize) -> u64 {
    let limb_shift = shift / 64;
    let bit_shift = shift % 64;
    let Some(low) = limbs.get(limb_shift).copied() else {
        return 0;
    };
    let mut result = low >> bit_shift;
    if bit_shift != 0 {
        if let Some(high) = limbs.get(limb_shift + 1).copied() {
            result |= high << (64 - bit_shift);
        }
    }
    result
}

fn magnitude_bit(limbs: &[u64], bit: usize) -> bool {
    let limb = bit / 64;
    let offset = bit % 64;
    limbs
        .get(limb)
        .is_some_and(|value| ((value >> offset) & 1) != 0)
}

fn magnitude_any_bits_below(limbs: &[u64], bit_count: usize) -> bool {
    let full_limbs = bit_count / 64;
    for limb in limbs.iter().take(full_limbs) {
        if *limb != 0 {
            return true;
        }
    }
    let remaining_bits = bit_count % 64;
    if remaining_bits == 0 {
        return false;
    }
    let mask = (1_u64 << remaining_bits) - 1;
    limbs.get(full_limbs).is_some_and(|limb| (limb & mask) != 0)
}

fn bigint_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    if invocation.new_target().is_some() {
        return Err(type_error(cx));
    }
    let argument = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let primitive = {
        let mut bridge = BuiltinToPrimitiveBridge { cx };
        object::to_primitive(&mut bridge, argument, object::ToPrimitiveHint::Number)?
    };
    let bigint = {
        let agent = cx.agent();
        object::primitive_to_bigint(agent, primitive)
    };
    map_completion(cx, bigint)
}

const BIGINT_WIDTH_EXACT_LIMIT_BITS: u64 = 4_096;

fn bigint_as_int_n_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let bits = to_index_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let bigint = to_bigint_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    if bits == 0 {
        return Ok(bigint_zero_value(cx));
    }
    let (sign, limbs) = bigint_parts(cx.agent(), bigint).ok_or_else(|| type_error(cx))?;
    if bigint_fits_signed_width(sign, &limbs, bits) {
        return Ok(bigint);
    }
    if bits > BIGINT_WIDTH_EXACT_LIMIT_BITS {
        return Err(range_error(cx));
    }
    let unsigned = bigint_to_uint_n_limbs(sign, &limbs, bits);
    let negative = bigint_width_sign_bit(&unsigned, bits);
    if negative {
        let magnitude = twos_complement_width_magnitude(unsigned, bits);
        Ok(bigint_from_parts(cx, BigIntSign::Negative, &magnitude))
    } else {
        Ok(bigint_from_parts(cx, BigIntSign::NonNegative, &unsigned))
    }
}

fn bigint_as_uint_n_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let bits = to_index_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let bigint = to_bigint_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    if bits == 0 {
        return Ok(bigint_zero_value(cx));
    }
    let (sign, limbs) = bigint_parts(cx.agent(), bigint).ok_or_else(|| type_error(cx))?;
    if sign == BigIntSign::NonNegative && bigint_magnitude_bit_length(&limbs) <= bits {
        return Ok(bigint);
    }
    if bits > BIGINT_WIDTH_EXACT_LIMIT_BITS {
        return Err(range_error(cx));
    }
    let unsigned = bigint_to_uint_n_limbs(sign, &limbs, bits);
    Ok(bigint_from_parts(cx, BigIntSign::NonNegative, &unsigned))
}

fn bigint_parts(agent: &Agent, value: Value) -> Option<(BigIntSign, Vec<u64>)> {
    let bigint = value.as_bigint_ref()?;
    let view = agent.heap().view().bigint_view(bigint)?;
    let mut limbs = Vec::with_capacity(view.limb_count() as usize);
    for index in 0..view.limb_count() {
        limbs.push(view.limb_at(index as usize).unwrap_or(0));
    }
    normalize_bigint_limbs(&mut limbs);
    Some((normalize_bigint_sign(view.sign(), &limbs), limbs))
}

fn bigint_zero_value<Cx: PublicBuiltinDispatchContext>(cx: &mut Cx) -> Value {
    bigint_from_parts(cx, BigIntSign::NonNegative, &[])
}

fn bigint_from_parts<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    sign: BigIntSign,
    limbs: &[u64],
) -> Value {
    let mut normalized = limbs.to_vec();
    normalize_bigint_limbs(&mut normalized);
    let sign = normalize_bigint_sign(sign, &normalized);
    let bigint = cx.agent().heap_mut().mutator().alloc_bigint(
        sign,
        &normalized,
        AllocationLifetime::Default,
    );
    Value::from_bigint_ref(bigint)
}

fn bigint_to_uint_n_limbs(sign: BigIntSign, limbs: &[u64], bits: u64) -> Vec<u64> {
    let width = usize::try_from(bits.div_ceil(64)).expect("exact BigInt width should fit");
    let mut result = vec![0; width.max(1)];
    let copied = result.len().min(limbs.len());
    result[..copied].copy_from_slice(&limbs[..copied]);
    if normalize_bigint_sign(sign, limbs) == BigIntSign::Negative {
        for limb in &mut result {
            *limb = !*limb;
        }
        add_one_to_limbs(&mut result);
    }
    mask_bigint_width(&mut result, bits);
    normalize_bigint_limbs(&mut result);
    result
}

fn twos_complement_width_magnitude(mut bits: Vec<u64>, width_bits: u64) -> Vec<u64> {
    for limb in &mut bits {
        *limb = !*limb;
    }
    mask_bigint_width(&mut bits, width_bits);
    add_one_to_limbs(&mut bits);
    mask_bigint_width(&mut bits, width_bits);
    normalize_bigint_limbs(&mut bits);
    bits
}

fn bigint_width_sign_bit(limbs: &[u64], bits: u64) -> bool {
    let bit = bits - 1;
    let limb_index = usize::try_from(bit / 64).expect("exact BigInt width should fit");
    let bit_index = bit % 64;
    limbs
        .get(limb_index)
        .is_some_and(|limb| (limb & (1_u64 << bit_index)) != 0)
}

fn mask_bigint_width(limbs: &mut [u64], bits: u64) {
    let remainder = bits % 64;
    if remainder == 0 {
        return;
    }
    if let Some(last) = limbs.last_mut() {
        *last &= (1_u64 << remainder) - 1;
    }
}

fn add_one_to_limbs(limbs: &mut [u64]) {
    let mut carry = true;
    for limb in limbs {
        if !carry {
            return;
        }
        let (next, overflow) = limb.overflowing_add(1);
        *limb = next;
        carry = overflow;
    }
}

fn normalize_bigint_limbs(limbs: &mut Vec<u64>) {
    while limbs.last() == Some(&0) {
        limbs.pop();
    }
}

fn normalize_bigint_sign(sign: BigIntSign, limbs: &[u64]) -> BigIntSign {
    if limbs.is_empty() {
        BigIntSign::NonNegative
    } else {
        sign
    }
}

fn bigint_magnitude_bit_length(limbs: &[u64]) -> u64 {
    let Some(last) = limbs.last().copied() else {
        return 0;
    };
    let high_bits = 64 - u64::from(last.leading_zeros());
    ((limbs.len() as u64 - 1) * 64) + high_bits
}

fn bigint_fits_signed_width(sign: BigIntSign, limbs: &[u64], bits: u64) -> bool {
    match normalize_bigint_sign(sign, limbs) {
        BigIntSign::NonNegative => bigint_magnitude_bit_length(limbs) < bits,
        BigIntSign::Negative => bigint_magnitude_le_power_of_two(limbs, bits - 1),
    }
}

fn bigint_magnitude_le_power_of_two(limbs: &[u64], exponent: u64) -> bool {
    let limb_index = usize::try_from(exponent / 64).expect("BigInt exponent should fit");
    if limbs.len() > limb_index + 1 {
        return false;
    }
    if limbs.len() <= limb_index {
        return true;
    }
    let bit = exponent % 64;
    let limit = 1_u64 << bit;
    let high = limbs[limb_index];
    high < limit || (high == limit && limbs[..limb_index].iter().all(|limb| *limb == 0))
}

fn bigint_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::BigInt,
        )
    };
    let bigint_value = map_completion(cx, value)?;
    let radix = radix_argument(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let text = {
        let agent = cx.agent();
        object::bigint_to_string(agent, bigint_value, radix)
    };
    let text = map_completion(cx, text)?;
    Ok(string_value(cx, &text))
}

fn bigint_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::BigInt,
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
        read::to_boolean(agent.heap().view(), argument)
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
