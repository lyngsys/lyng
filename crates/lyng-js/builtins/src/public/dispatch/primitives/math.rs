use super::super::{
    argument_to_number, close_iterator_after_error, number_value, to_uint32_for_builtin,
    type_error, BuiltinIteratorBridge, PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_ops::iterator;
use lyng_js_types::{BuiltinFunctionId, Value};
use std::f64::consts::LN_2;
use std::time::{SystemTime, UNIX_EPOCH};

const ACOSH_LARGE_THRESHOLD: f64 = 268_435_456.0;
const ATANH_TINY_THRESHOLD: f64 = 3.725_290_298_461_914e-9;

pub(super) fn dispatch_math_builtin<Cx: PublicBuiltinDispatchContext>(
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
    if entry == super::super::math_abs_builtin() {
        return math_abs_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_acos_builtin() {
        return math_acos_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_acosh_builtin() {
        return math_acosh_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_asin_builtin() {
        return math_asin_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_asinh_builtin() {
        return math_asinh_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_atan_builtin() {
        return math_atan_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_atan2_builtin() {
        return math_atan2_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_atanh_builtin() {
        return math_atanh_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_cbrt_builtin() {
        return math_cbrt_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_ceil_builtin() {
        return math_ceil_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_clz32_builtin() {
        return math_clz32_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_cos_builtin() {
        return math_cos_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_cosh_builtin() {
        return math_cosh_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_math_advanced_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::super::math_exp_builtin() {
        return math_exp_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_expm1_builtin() {
        return math_expm1_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_f16round_builtin() {
        return math_f16round_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_floor_builtin() {
        return math_floor_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_fround_builtin() {
        return math_fround_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_hypot_builtin() {
        return math_hypot_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_imul_builtin() {
        return math_imul_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_log_builtin() {
        return math_log_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_log10_builtin() {
        return math_log10_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_log1p_builtin() {
        return math_log1p_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_log2_builtin() {
        return math_log2_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_max_builtin() {
        return math_max_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_min_builtin() {
        return math_min_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_pow_builtin() {
        return math_pow_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_random_builtin() {
        return Ok(Some(math_random_builtin()));
    }
    if entry == super::super::math_round_builtin() {
        return math_round_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_sign_builtin() {
        return math_sign_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_sin_builtin() {
        return math_sin_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_sinh_builtin() {
        return math_sinh_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_sqrt_builtin() {
        return math_sqrt_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_sum_precise_builtin() {
        return math_sum_precise_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_tan_builtin() {
        return math_tan_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_tanh_builtin() {
        return math_tanh_builtin(context, invocation).map(Some);
    }
    if entry == super::super::math_trunc_builtin() {
        return math_trunc_builtin(context, invocation).map(Some);
    }
    Ok(None)
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
    math_unary_number_builtin(cx, invocation, acosh_number)
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
    math_unary_number_builtin(cx, invocation, atanh_number)
}

fn acosh_number(value: f64) -> f64 {
    if value.is_nan() || value < 1.0 {
        return f64::NAN;
    }
    if value == 1.0 {
        return 0.0;
    }
    if value.is_infinite() {
        return value;
    }
    if value >= ACOSH_LARGE_THRESHOLD {
        return value.ln() + LN_2;
    }
    if value > 2.0 {
        return 2.0f64
            .mul_add(value, -(1.0 / (value + value.mul_add(value, -1.0).sqrt())))
            .ln();
    }

    let delta = value - 1.0;
    (delta + 2.0f64.mul_add(delta, delta * delta).sqrt()).ln_1p()
}

fn atanh_number(value: f64) -> f64 {
    let abs = value.abs();
    if value.is_nan() || abs > 1.0 {
        return f64::NAN;
    }
    if abs == 1.0 {
        return value / 0.0;
    }
    if abs < ATANH_TINY_THRESHOLD {
        return value;
    }

    let result = if abs < 0.5 {
        let doubled = abs + abs;
        0.5 * (doubled + doubled * abs / (1.0 - abs)).ln_1p()
    } else {
        0.5 * ((abs + abs) / (1.0 - abs)).ln_1p()
    };
    if value.is_sign_negative() {
        -result
    } else {
        result
    }
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
    if bit_shift != 0
        && let Some(high) = limbs.get(limb_shift + 1).copied()
    {
        result |= high << (64 - bit_shift);
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
