use super::PublicBuiltinDispatchContext;
use crate::BuiltinInvocation;
use lyng_js_types::{BuiltinFunctionId, Value};

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
    if entry == super::js3_number_builtin() {
        return super::number_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_number_is_finite_builtin() {
        return super::number_is_finite_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_number_is_integer_builtin() {
        return super::number_is_integer_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_number_is_nan_builtin() {
        return super::number_is_nan_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_number_is_safe_integer_builtin() {
        return super::number_is_safe_integer_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_number_to_exponential_builtin() {
        return super::number_to_exponential_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_number_to_fixed_builtin() {
        return super::number_to_fixed_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_number_to_locale_string_builtin() {
        return super::number_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_number_to_precision_builtin() {
        return super::number_to_precision_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_number_to_string_builtin() {
        return super::number_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_number_value_of_builtin() {
        return super::number_value_of_builtin(context, invocation).map(Some);
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
    if entry == super::js3_math_abs_builtin() {
        return super::math_abs_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_acos_builtin() {
        return super::math_acos_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_acosh_builtin() {
        return super::math_acosh_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_asin_builtin() {
        return super::math_asin_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_asinh_builtin() {
        return super::math_asinh_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_atan_builtin() {
        return super::math_atan_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_atan2_builtin() {
        return super::math_atan2_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_atanh_builtin() {
        return super::math_atanh_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_cbrt_builtin() {
        return super::math_cbrt_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_ceil_builtin() {
        return super::math_ceil_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_clz32_builtin() {
        return super::math_clz32_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_cos_builtin() {
        return super::math_cos_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_cosh_builtin() {
        return super::math_cosh_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_math_advanced_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_math_exp_builtin() {
        return super::math_exp_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_expm1_builtin() {
        return super::math_expm1_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_f16round_builtin() {
        return super::math_f16round_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_floor_builtin() {
        return super::math_floor_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_fround_builtin() {
        return super::math_fround_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_hypot_builtin() {
        return super::math_hypot_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_imul_builtin() {
        return super::math_imul_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_log_builtin() {
        return super::math_log_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_log10_builtin() {
        return super::math_log10_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_log1p_builtin() {
        return super::math_log1p_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_log2_builtin() {
        return super::math_log2_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_max_builtin() {
        return super::math_max_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_min_builtin() {
        return super::math_min_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_pow_builtin() {
        return super::math_pow_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_random_builtin() {
        return super::math_random_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_round_builtin() {
        return super::math_round_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_sign_builtin() {
        return super::math_sign_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_sin_builtin() {
        return super::math_sin_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_sinh_builtin() {
        return super::math_sinh_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_sqrt_builtin() {
        return super::math_sqrt_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_sum_precise_builtin() {
        return super::math_sum_precise_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_tan_builtin() {
        return super::math_tan_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_tanh_builtin() {
        return super::math_tanh_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_math_trunc_builtin() {
        return super::math_trunc_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_bigint_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_bigint_builtin() {
        return super::bigint_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_bigint_as_int_n_builtin() {
        return super::bigint_as_int_n_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_bigint_as_uint_n_builtin() {
        return super::bigint_as_uint_n_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_bigint_to_string_builtin() {
        return super::bigint_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_bigint_value_of_builtin() {
        return super::bigint_value_of_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_boolean_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_boolean_builtin() {
        return super::boolean_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_boolean_to_string_builtin() {
        return super::boolean_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_boolean_value_of_builtin() {
        return super::boolean_value_of_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_symbol_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_symbol_builtin() {
        return super::symbol_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_symbol_for_builtin() {
        return super::symbol_for_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_symbol_key_for_builtin() {
        return super::symbol_key_for_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_symbol_to_string_builtin() {
        return super::symbol_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_symbol_value_of_builtin() {
        return super::symbol_value_of_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_symbol_to_primitive_builtin() {
        return super::symbol_to_primitive_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_symbol_description_getter_builtin() {
        return super::symbol_description_getter_builtin(context, invocation).map(Some);
    }
    Ok(None)
}
