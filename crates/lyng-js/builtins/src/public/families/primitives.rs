use super::{
    install_public_builtin_function, FamilyInstallContext, PrimitiveFamilyBuiltins,
    PrimitiveFamilyObjects, PrimitiveFamilyPrototypes,
};
use lyng_js_env::Agent;
use lyng_js_types::{
    js3_array_species_getter_builtin, js3_bigint_as_int_n_builtin, js3_bigint_as_uint_n_builtin,
    js3_bigint_builtin, js3_bigint_to_string_builtin, js3_bigint_value_of_builtin,
    js3_boolean_builtin, js3_boolean_to_string_builtin, js3_boolean_value_of_builtin,
    js3_math_abs_builtin, js3_math_acos_builtin, js3_math_acosh_builtin, js3_math_asin_builtin,
    js3_math_asinh_builtin, js3_math_atan2_builtin, js3_math_atan_builtin, js3_math_atanh_builtin,
    js3_math_cbrt_builtin, js3_math_ceil_builtin, js3_math_clz32_builtin, js3_math_cos_builtin,
    js3_math_cosh_builtin, js3_math_exp_builtin, js3_math_expm1_builtin, js3_math_f16round_builtin,
    js3_math_floor_builtin, js3_math_fround_builtin, js3_math_hypot_builtin, js3_math_imul_builtin,
    js3_math_log10_builtin, js3_math_log1p_builtin, js3_math_log2_builtin, js3_math_log_builtin,
    js3_math_max_builtin, js3_math_min_builtin, js3_math_pow_builtin, js3_math_random_builtin,
    js3_math_round_builtin, js3_math_sign_builtin, js3_math_sin_builtin, js3_math_sinh_builtin,
    js3_math_sqrt_builtin, js3_math_sum_precise_builtin, js3_math_tan_builtin,
    js3_math_tanh_builtin, js3_math_trunc_builtin, js3_number_builtin,
    js3_number_is_finite_builtin, js3_number_is_integer_builtin, js3_number_is_nan_builtin,
    js3_number_is_safe_integer_builtin, js3_number_to_exponential_builtin,
    js3_number_to_fixed_builtin, js3_number_to_locale_string_builtin,
    js3_number_to_precision_builtin, js3_number_to_string_builtin, js3_number_value_of_builtin,
    js3_symbol_builtin, js3_symbol_description_getter_builtin, js3_symbol_for_builtin,
    js3_symbol_key_for_builtin, js3_symbol_to_primitive_builtin, js3_symbol_to_string_builtin,
    js3_symbol_value_of_builtin, ObjectRef,
};

pub(in crate::public) fn install_primitive_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototypes: PrimitiveFamilyPrototypes,
    objects: PrimitiveFamilyObjects,
) -> PrimitiveFamilyBuiltins {
    let number = install_number_family(agent, cx, prototypes.number_prototype);
    let math = install_math_family(agent, cx, objects.math);
    let bigint = install_bigint_family(agent, cx, prototypes.bigint_prototype);
    let boolean = install_boolean_family(agent, cx, prototypes.boolean_prototype);
    let symbol = install_symbol_family(agent, cx, prototypes.symbol_prototype);

    PrimitiveFamilyBuiltins {
        number: number.number,
        number_prototype: number.prototype,
        number_is_finite: number.is_finite,
        number_is_integer: number.is_integer,
        number_is_nan: number.is_nan,
        number_is_safe_integer: number.is_safe_integer,
        number_to_exponential: number.to_exponential,
        number_to_fixed: number.to_fixed,
        number_to_locale_string: number.to_locale_string,
        number_to_precision: number.to_precision,
        number_to_string: number.to_string,
        number_value_of: number.value_of,
        math: math.math,
        math_abs: math.abs,
        math_acos: math.acos,
        math_acosh: math.acosh,
        math_asin: math.asin,
        math_asinh: math.asinh,
        math_atan: math.atan,
        math_atan2: math.atan2,
        math_atanh: math.atanh,
        math_cbrt: math.cbrt,
        math_ceil: math.ceil,
        math_clz32: math.clz32,
        math_cos: math.cos,
        math_cosh: math.cosh,
        math_exp: math.exp,
        math_expm1: math.expm1,
        math_f16round: math.f16round,
        math_floor: math.floor,
        math_fround: math.fround,
        math_hypot: math.hypot,
        math_imul: math.imul,
        math_log: math.log,
        math_log10: math.log10,
        math_log1p: math.log1p,
        math_log2: math.log2,
        math_max: math.max,
        math_min: math.min,
        math_pow: math.pow,
        math_random: math.random,
        math_round: math.round,
        math_sign: math.sign,
        math_sin: math.sin,
        math_sinh: math.sinh,
        math_sqrt: math.sqrt,
        math_sum_precise: math.sum_precise,
        math_tan: math.tan,
        math_tanh: math.tanh,
        math_trunc: math.trunc,
        bigint: bigint.bigint,
        bigint_as_int_n: bigint.as_int_n,
        bigint_as_uint_n: bigint.as_uint_n,
        bigint_prototype: bigint.prototype,
        bigint_to_string: bigint.to_string,
        bigint_value_of: bigint.value_of,
        boolean: boolean.boolean,
        boolean_prototype: boolean.prototype,
        boolean_to_string: boolean.to_string,
        boolean_value_of: boolean.value_of,
        symbol: symbol.symbol,
        symbol_prototype: symbol.prototype,
        symbol_for: symbol.symbol_for,
        symbol_key_for: symbol.key_for,
        symbol_to_string: symbol.to_string,
        symbol_value_of: symbol.value_of,
        symbol_to_primitive: symbol.to_primitive,
        array_species_getter: install_public_builtin_function(
            agent,
            cx,
            js3_array_species_getter_builtin(),
            None,
        ),
        symbol_description_getter: install_public_builtin_function(
            agent,
            cx,
            js3_symbol_description_getter_builtin(),
            None,
        ),
    }
}

#[derive(Clone, Copy, Debug)]
struct NumberBuiltins {
    number: ObjectRef,
    prototype: ObjectRef,
    is_finite: ObjectRef,
    is_integer: ObjectRef,
    is_nan: ObjectRef,
    is_safe_integer: ObjectRef,
    to_exponential: ObjectRef,
    to_fixed: ObjectRef,
    to_locale_string: ObjectRef,
    to_precision: ObjectRef,
    to_string: ObjectRef,
    value_of: ObjectRef,
}

fn install_number_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototype: ObjectRef,
) -> NumberBuiltins {
    NumberBuiltins {
        number: install_public_builtin_function(agent, cx, js3_number_builtin(), Some(prototype)),
        prototype,
        is_finite: install_public_builtin_function(agent, cx, js3_number_is_finite_builtin(), None),
        is_integer: install_public_builtin_function(
            agent,
            cx,
            js3_number_is_integer_builtin(),
            None,
        ),
        is_nan: install_public_builtin_function(agent, cx, js3_number_is_nan_builtin(), None),
        is_safe_integer: install_public_builtin_function(
            agent,
            cx,
            js3_number_is_safe_integer_builtin(),
            None,
        ),
        to_exponential: install_public_builtin_function(
            agent,
            cx,
            js3_number_to_exponential_builtin(),
            None,
        ),
        to_fixed: install_public_builtin_function(agent, cx, js3_number_to_fixed_builtin(), None),
        to_locale_string: install_public_builtin_function(
            agent,
            cx,
            js3_number_to_locale_string_builtin(),
            None,
        ),
        to_precision: install_public_builtin_function(
            agent,
            cx,
            js3_number_to_precision_builtin(),
            None,
        ),
        to_string: install_public_builtin_function(agent, cx, js3_number_to_string_builtin(), None),
        value_of: install_public_builtin_function(agent, cx, js3_number_value_of_builtin(), None),
    }
}

#[derive(Clone, Copy, Debug)]
struct MathBuiltins {
    math: ObjectRef,
    abs: ObjectRef,
    acos: ObjectRef,
    acosh: ObjectRef,
    asin: ObjectRef,
    asinh: ObjectRef,
    atan: ObjectRef,
    atan2: ObjectRef,
    atanh: ObjectRef,
    cbrt: ObjectRef,
    ceil: ObjectRef,
    clz32: ObjectRef,
    cos: ObjectRef,
    cosh: ObjectRef,
    exp: ObjectRef,
    expm1: ObjectRef,
    f16round: ObjectRef,
    floor: ObjectRef,
    fround: ObjectRef,
    hypot: ObjectRef,
    imul: ObjectRef,
    log: ObjectRef,
    log10: ObjectRef,
    log1p: ObjectRef,
    log2: ObjectRef,
    max: ObjectRef,
    min: ObjectRef,
    pow: ObjectRef,
    random: ObjectRef,
    round: ObjectRef,
    sign: ObjectRef,
    sin: ObjectRef,
    sinh: ObjectRef,
    sqrt: ObjectRef,
    sum_precise: ObjectRef,
    tan: ObjectRef,
    tanh: ObjectRef,
    trunc: ObjectRef,
}

#[allow(clippy::too_many_lines)]
fn install_math_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    math: ObjectRef,
) -> MathBuiltins {
    MathBuiltins {
        math,
        abs: install_public_builtin_function(agent, cx, js3_math_abs_builtin(), None),
        acos: install_public_builtin_function(agent, cx, js3_math_acos_builtin(), None),
        acosh: install_public_builtin_function(agent, cx, js3_math_acosh_builtin(), None),
        asin: install_public_builtin_function(agent, cx, js3_math_asin_builtin(), None),
        asinh: install_public_builtin_function(agent, cx, js3_math_asinh_builtin(), None),
        atan: install_public_builtin_function(agent, cx, js3_math_atan_builtin(), None),
        atan2: install_public_builtin_function(agent, cx, js3_math_atan2_builtin(), None),
        atanh: install_public_builtin_function(agent, cx, js3_math_atanh_builtin(), None),
        cbrt: install_public_builtin_function(agent, cx, js3_math_cbrt_builtin(), None),
        ceil: install_public_builtin_function(agent, cx, js3_math_ceil_builtin(), None),
        clz32: install_public_builtin_function(agent, cx, js3_math_clz32_builtin(), None),
        cos: install_public_builtin_function(agent, cx, js3_math_cos_builtin(), None),
        cosh: install_public_builtin_function(agent, cx, js3_math_cosh_builtin(), None),
        exp: install_public_builtin_function(agent, cx, js3_math_exp_builtin(), None),
        expm1: install_public_builtin_function(agent, cx, js3_math_expm1_builtin(), None),
        f16round: install_public_builtin_function(agent, cx, js3_math_f16round_builtin(), None),
        floor: install_public_builtin_function(agent, cx, js3_math_floor_builtin(), None),
        fround: install_public_builtin_function(agent, cx, js3_math_fround_builtin(), None),
        hypot: install_public_builtin_function(agent, cx, js3_math_hypot_builtin(), None),
        imul: install_public_builtin_function(agent, cx, js3_math_imul_builtin(), None),
        log: install_public_builtin_function(agent, cx, js3_math_log_builtin(), None),
        log10: install_public_builtin_function(agent, cx, js3_math_log10_builtin(), None),
        log1p: install_public_builtin_function(agent, cx, js3_math_log1p_builtin(), None),
        log2: install_public_builtin_function(agent, cx, js3_math_log2_builtin(), None),
        max: install_public_builtin_function(agent, cx, js3_math_max_builtin(), None),
        min: install_public_builtin_function(agent, cx, js3_math_min_builtin(), None),
        pow: install_public_builtin_function(agent, cx, js3_math_pow_builtin(), None),
        random: install_public_builtin_function(agent, cx, js3_math_random_builtin(), None),
        round: install_public_builtin_function(agent, cx, js3_math_round_builtin(), None),
        sign: install_public_builtin_function(agent, cx, js3_math_sign_builtin(), None),
        sin: install_public_builtin_function(agent, cx, js3_math_sin_builtin(), None),
        sinh: install_public_builtin_function(agent, cx, js3_math_sinh_builtin(), None),
        sqrt: install_public_builtin_function(agent, cx, js3_math_sqrt_builtin(), None),
        sum_precise: install_public_builtin_function(
            agent,
            cx,
            js3_math_sum_precise_builtin(),
            None,
        ),
        tan: install_public_builtin_function(agent, cx, js3_math_tan_builtin(), None),
        tanh: install_public_builtin_function(agent, cx, js3_math_tanh_builtin(), None),
        trunc: install_public_builtin_function(agent, cx, js3_math_trunc_builtin(), None),
    }
}

#[derive(Clone, Copy, Debug)]
struct BigIntBuiltins {
    bigint: ObjectRef,
    as_int_n: ObjectRef,
    as_uint_n: ObjectRef,
    prototype: ObjectRef,
    to_string: ObjectRef,
    value_of: ObjectRef,
}

fn install_bigint_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototype: ObjectRef,
) -> BigIntBuiltins {
    BigIntBuiltins {
        bigint: install_public_builtin_function(agent, cx, js3_bigint_builtin(), Some(prototype)),
        as_int_n: install_public_builtin_function(agent, cx, js3_bigint_as_int_n_builtin(), None),
        as_uint_n: install_public_builtin_function(agent, cx, js3_bigint_as_uint_n_builtin(), None),
        prototype,
        to_string: install_public_builtin_function(agent, cx, js3_bigint_to_string_builtin(), None),
        value_of: install_public_builtin_function(agent, cx, js3_bigint_value_of_builtin(), None),
    }
}

#[derive(Clone, Copy, Debug)]
struct BooleanBuiltins {
    boolean: ObjectRef,
    prototype: ObjectRef,
    to_string: ObjectRef,
    value_of: ObjectRef,
}

fn install_boolean_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototype: ObjectRef,
) -> BooleanBuiltins {
    BooleanBuiltins {
        boolean: install_public_builtin_function(agent, cx, js3_boolean_builtin(), Some(prototype)),
        prototype,
        to_string: install_public_builtin_function(
            agent,
            cx,
            js3_boolean_to_string_builtin(),
            None,
        ),
        value_of: install_public_builtin_function(agent, cx, js3_boolean_value_of_builtin(), None),
    }
}

#[derive(Clone, Copy, Debug)]
struct SymbolBuiltins {
    symbol: ObjectRef,
    prototype: ObjectRef,
    symbol_for: ObjectRef,
    key_for: ObjectRef,
    to_string: ObjectRef,
    value_of: ObjectRef,
    to_primitive: ObjectRef,
}

fn install_symbol_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototype: ObjectRef,
) -> SymbolBuiltins {
    SymbolBuiltins {
        symbol: install_public_builtin_function(agent, cx, js3_symbol_builtin(), Some(prototype)),
        prototype,
        symbol_for: install_public_builtin_function(agent, cx, js3_symbol_for_builtin(), None),
        key_for: install_public_builtin_function(agent, cx, js3_symbol_key_for_builtin(), None),
        to_string: install_public_builtin_function(agent, cx, js3_symbol_to_string_builtin(), None),
        value_of: install_public_builtin_function(agent, cx, js3_symbol_value_of_builtin(), None),
        to_primitive: install_public_builtin_function(
            agent,
            cx,
            js3_symbol_to_primitive_builtin(),
            None,
        ),
    }
}
