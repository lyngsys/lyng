use super::descriptors::{
    accessor_atom_property, builtin_function_atom_property, builtin_function_symbol_property,
    data_atom_property, data_symbol_property, descriptor_tag_with_atom, hidden_builtin_attributes,
    readonly_builtin_attributes, writable_builtin_attributes,
};
use super::{
    install_public_builtin_function, FamilyInstallContext, PrimitiveFamilyBuiltins,
    PrimitiveFamilyObjects, PrimitiveFamilyPrototypes,
};
use crate::bootstrap::{install_descriptor_tables, BuiltinBootstrapError};
use crate::public::{BuiltinCache, PublicRealmBuiltins};
use crate::{
    BuiltinDescriptorTable, BuiltinInstallTarget, BuiltinIntrinsic, BuiltinPropertyDescriptor,
};
use lyng_js_common::{AtomId, WellKnownAtom};
use lyng_js_env::Agent;
use lyng_js_types::{
    array_species_getter_builtin, bigint_as_int_n_builtin, bigint_as_uint_n_builtin,
    bigint_builtin, bigint_to_string_builtin, bigint_value_of_builtin, boolean_builtin,
    boolean_to_string_builtin, boolean_value_of_builtin, math_abs_builtin, math_acos_builtin,
    math_acosh_builtin, math_asin_builtin, math_asinh_builtin, math_atan2_builtin,
    math_atan_builtin, math_atanh_builtin, math_cbrt_builtin, math_ceil_builtin,
    math_clz32_builtin, math_cos_builtin, math_cosh_builtin, math_exp_builtin, math_expm1_builtin,
    math_f16round_builtin, math_floor_builtin, math_fround_builtin, math_hypot_builtin,
    math_imul_builtin, math_log10_builtin, math_log1p_builtin, math_log2_builtin, math_log_builtin,
    math_max_builtin, math_min_builtin, math_pow_builtin, math_random_builtin, math_round_builtin,
    math_sign_builtin, math_sin_builtin, math_sinh_builtin, math_sqrt_builtin,
    math_sum_precise_builtin, math_tan_builtin, math_tanh_builtin, math_trunc_builtin,
    number_builtin, number_is_finite_builtin, number_is_integer_builtin, number_is_nan_builtin,
    number_is_safe_integer_builtin, number_to_exponential_builtin, number_to_fixed_builtin,
    number_to_locale_string_builtin, number_to_precision_builtin, number_to_string_builtin,
    number_value_of_builtin, parse_float_builtin, parse_int_builtin, symbol_builtin,
    symbol_description_getter_builtin, symbol_for_builtin, symbol_key_for_builtin,
    symbol_to_primitive_builtin, symbol_to_string_builtin, symbol_value_of_builtin,
    BuiltinFunctionId, ObjectRef, RealmRef, Value, WellKnownSymbolId,
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
            array_species_getter_builtin(),
            None,
        ),
        symbol_description_getter: install_public_builtin_function(
            agent,
            cx,
            symbol_description_getter_builtin(),
            None,
        ),
    }
}

pub(in crate::public) fn primitive_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    if let Some(object) = number_builtin_object(builtins, entry) {
        return Some(object);
    }
    if let Some(object) = math_builtin_object(builtins, entry) {
        return Some(object);
    }
    if let Some(object) = bigint_builtin_object(builtins, entry) {
        return Some(object);
    }
    if let Some(object) = boolean_builtin_object(builtins, entry) {
        return Some(object);
    }
    if let Some(object) = symbol_builtin_object(builtins, entry) {
        return Some(object);
    }
    primitive_accessor_builtin_object(builtins, entry)
}

fn number_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (number_builtin(), builtins.number),
        (number_is_finite_builtin(), builtins.number_is_finite),
        (number_is_integer_builtin(), builtins.number_is_integer),
        (number_is_nan_builtin(), builtins.number_is_nan),
        (
            number_is_safe_integer_builtin(),
            builtins.number_is_safe_integer,
        ),
        (
            number_to_exponential_builtin(),
            builtins.number_to_exponential,
        ),
        (number_to_fixed_builtin(), builtins.number_to_fixed),
        (
            number_to_locale_string_builtin(),
            builtins.number_to_locale_string,
        ),
        (number_to_precision_builtin(), builtins.number_to_precision),
        (number_to_string_builtin(), builtins.number_to_string),
        (number_value_of_builtin(), builtins.number_value_of),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

fn math_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (math_abs_builtin(), builtins.math_abs),
        (math_acos_builtin(), builtins.math_acos),
        (math_acosh_builtin(), builtins.math_acosh),
        (math_asin_builtin(), builtins.math_asin),
        (math_asinh_builtin(), builtins.math_asinh),
        (math_atan_builtin(), builtins.math_atan),
        (math_atan2_builtin(), builtins.math_atan2),
        (math_atanh_builtin(), builtins.math_atanh),
        (math_cbrt_builtin(), builtins.math_cbrt),
        (math_ceil_builtin(), builtins.math_ceil),
        (math_clz32_builtin(), builtins.math_clz32),
        (math_cos_builtin(), builtins.math_cos),
        (math_cosh_builtin(), builtins.math_cosh),
        (math_exp_builtin(), builtins.math_exp),
        (math_expm1_builtin(), builtins.math_expm1),
        (math_f16round_builtin(), builtins.math_f16round),
        (math_floor_builtin(), builtins.math_floor),
        (math_fround_builtin(), builtins.math_fround),
        (math_hypot_builtin(), builtins.math_hypot),
        (math_imul_builtin(), builtins.math_imul),
        (math_log_builtin(), builtins.math_log),
        (math_log10_builtin(), builtins.math_log10),
        (math_log1p_builtin(), builtins.math_log1p),
        (math_log2_builtin(), builtins.math_log2),
        (math_max_builtin(), builtins.math_max),
        (math_min_builtin(), builtins.math_min),
        (math_pow_builtin(), builtins.math_pow),
        (math_random_builtin(), builtins.math_random),
        (math_round_builtin(), builtins.math_round),
        (math_sign_builtin(), builtins.math_sign),
        (math_sin_builtin(), builtins.math_sin),
        (math_sinh_builtin(), builtins.math_sinh),
        (math_sqrt_builtin(), builtins.math_sqrt),
        (math_sum_precise_builtin(), builtins.math_sum_precise),
        (math_tan_builtin(), builtins.math_tan),
        (math_tanh_builtin(), builtins.math_tanh),
        (math_trunc_builtin(), builtins.math_trunc),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

fn bigint_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (bigint_builtin(), builtins.bigint),
        (bigint_as_int_n_builtin(), builtins.bigint_as_int_n),
        (bigint_as_uint_n_builtin(), builtins.bigint_as_uint_n),
        (bigint_to_string_builtin(), builtins.bigint_to_string),
        (bigint_value_of_builtin(), builtins.bigint_value_of),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

fn boolean_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (boolean_builtin(), builtins.boolean),
        (boolean_to_string_builtin(), builtins.boolean_to_string),
        (boolean_value_of_builtin(), builtins.boolean_value_of),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

fn symbol_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (symbol_builtin(), builtins.symbol),
        (symbol_for_builtin(), builtins.symbol_for),
        (symbol_key_for_builtin(), builtins.symbol_key_for),
        (symbol_to_string_builtin(), builtins.symbol_to_string),
        (symbol_value_of_builtin(), builtins.symbol_value_of),
        (symbol_to_primitive_builtin(), builtins.symbol_to_primitive),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

fn primitive_accessor_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (
            array_species_getter_builtin(),
            builtins.array_species_getter,
        ),
        (
            symbol_description_getter_builtin(),
            builtins.symbol_description_getter,
        ),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

pub(in crate::public) fn install_primitive_family_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
) -> Result<(), BuiltinBootstrapError> {
    let atoms = PrimitiveDescriptorAtoms::new(agent);
    install_number_constructor_descriptors(agent, cache, realm, &atoms)?;
    install_number_prototype_descriptors(agent, cache, realm, builtins.number, &atoms)?;
    install_math_descriptors(agent, cache, realm, &atoms)?;
    install_bigint_constructor_descriptors(agent, cache, realm, &atoms)?;
    install_bigint_prototype_descriptors(agent, cache, realm, builtins.bigint)?;
    install_boolean_prototype_descriptors(agent, cache, realm, builtins.boolean)?;
    install_symbol_constructor_descriptors(agent, cache, realm, &atoms)?;
    install_symbol_prototype_descriptors(agent, cache, realm, builtins.symbol, &atoms)
}

fn install_number_constructor_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    atoms: &PrimitiveDescriptorAtoms,
) -> Result<(), BuiltinBootstrapError> {
    install_builtin_method_group(
        agent,
        cache,
        realm,
        BuiltinIntrinsic::Number,
        number_static_method_specs(atoms),
    )?;

    let descriptors = [
        data_atom_property(
            atoms.nan,
            Value::from_f64(f64::NAN),
            hidden_builtin_attributes(),
        ),
        data_atom_property(
            atoms.positive_infinity,
            Value::from_f64(f64::INFINITY),
            hidden_builtin_attributes(),
        ),
        data_atom_property(
            atoms.negative_infinity,
            Value::from_f64(f64::NEG_INFINITY),
            hidden_builtin_attributes(),
        ),
        data_atom_property(
            atoms.max_value,
            Value::from_f64(f64::MAX),
            hidden_builtin_attributes(),
        ),
        data_atom_property(
            atoms.min_value,
            Value::from_f64(f64::from_bits(1)),
            hidden_builtin_attributes(),
        ),
        data_atom_property(
            atoms.max_safe_integer,
            Value::from_f64(9_007_199_254_740_991.0),
            hidden_builtin_attributes(),
        ),
        data_atom_property(
            atoms.min_safe_integer,
            Value::from_f64(-9_007_199_254_740_991.0),
            hidden_builtin_attributes(),
        ),
        data_atom_property(
            atoms.epsilon,
            Value::from_f64(f64::EPSILON),
            hidden_builtin_attributes(),
        ),
    ];
    install_intrinsic_descriptor_table(agent, cache, realm, BuiltinIntrinsic::Number, &descriptors)
}

fn install_number_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    number: ObjectRef,
    atoms: &PrimitiveDescriptorAtoms,
) -> Result<(), BuiltinBootstrapError> {
    let constructor = [data_atom_property(
        WellKnownAtom::constructor.id(),
        Value::from_object_ref(number),
        writable_builtin_attributes(),
    )];
    install_intrinsic_descriptor_table(
        agent,
        cache,
        realm,
        BuiltinIntrinsic::NumberPrototype,
        &constructor,
    )?;
    install_builtin_method_group(
        agent,
        cache,
        realm,
        BuiltinIntrinsic::NumberPrototype,
        number_prototype_method_specs(atoms),
    )
}

fn install_math_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    atoms: &PrimitiveDescriptorAtoms,
) -> Result<(), BuiltinBootstrapError> {
    install_builtin_method_group(
        agent,
        cache,
        realm,
        BuiltinIntrinsic::Math,
        math_method_specs(atoms),
    )?;
    let math_atom = agent.bootstrap_atoms().math();
    let math_tag = descriptor_tag_with_atom(agent, "Math", math_atom);
    let descriptors = [
        data_atom_property(
            atoms.e,
            Value::from_f64(std::f64::consts::E),
            hidden_builtin_attributes(),
        ),
        data_atom_property(
            atoms.ln10,
            Value::from_f64(std::f64::consts::LN_10),
            hidden_builtin_attributes(),
        ),
        data_atom_property(
            atoms.ln2,
            Value::from_f64(std::f64::consts::LN_2),
            hidden_builtin_attributes(),
        ),
        data_atom_property(
            atoms.log10e,
            Value::from_f64(std::f64::consts::LOG10_E),
            hidden_builtin_attributes(),
        ),
        data_atom_property(
            atoms.log2e,
            Value::from_f64(std::f64::consts::LOG2_E),
            hidden_builtin_attributes(),
        ),
        data_atom_property(
            atoms.pi,
            Value::from_f64(std::f64::consts::PI),
            hidden_builtin_attributes(),
        ),
        data_atom_property(
            atoms.sqrt1_2,
            Value::from_f64(std::f64::consts::FRAC_1_SQRT_2),
            hidden_builtin_attributes(),
        ),
        data_atom_property(
            atoms.sqrt2,
            Value::from_f64(std::f64::consts::SQRT_2),
            hidden_builtin_attributes(),
        ),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            math_tag,
            readonly_builtin_attributes(),
        ),
    ];
    install_intrinsic_descriptor_table(agent, cache, realm, BuiltinIntrinsic::Math, &descriptors)
}

fn install_bigint_constructor_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    atoms: &PrimitiveDescriptorAtoms,
) -> Result<(), BuiltinBootstrapError> {
    install_builtin_method_group(
        agent,
        cache,
        realm,
        BuiltinIntrinsic::BigInt,
        bigint_static_method_specs(atoms),
    )
}

fn install_bigint_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    bigint: ObjectRef,
) -> Result<(), BuiltinBootstrapError> {
    let bigint_atom = agent.bootstrap_atoms().bigint();
    let bigint_tag = descriptor_tag_with_atom(agent, "BigInt", bigint_atom);
    let descriptors = [
        data_atom_property(
            WellKnownAtom::constructor.id(),
            Value::from_object_ref(bigint),
            writable_builtin_attributes(),
        ),
        builtin_function_atom_property(WellKnownAtom::toString.id(), bigint_to_string_builtin()),
        builtin_function_atom_property(WellKnownAtom::valueOf.id(), bigint_value_of_builtin()),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            bigint_tag,
            readonly_builtin_attributes(),
        ),
    ];
    install_intrinsic_descriptor_table(
        agent,
        cache,
        realm,
        BuiltinIntrinsic::BigIntPrototype,
        &descriptors,
    )
}

fn install_boolean_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    boolean: ObjectRef,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [
        data_atom_property(
            WellKnownAtom::constructor.id(),
            Value::from_object_ref(boolean),
            writable_builtin_attributes(),
        ),
        builtin_function_atom_property(WellKnownAtom::toString.id(), boolean_to_string_builtin()),
        builtin_function_atom_property(WellKnownAtom::valueOf.id(), boolean_value_of_builtin()),
    ];
    install_intrinsic_descriptor_table(
        agent,
        cache,
        realm,
        BuiltinIntrinsic::BooleanPrototype,
        &descriptors,
    )
}

fn install_symbol_constructor_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    atoms: &PrimitiveDescriptorAtoms,
) -> Result<(), BuiltinBootstrapError> {
    install_builtin_method_group(
        agent,
        cache,
        realm,
        BuiltinIntrinsic::Symbol,
        symbol_static_method_specs(atoms),
    )?;

    let descriptors = symbol_well_known_value_descriptors(agent)?;
    install_intrinsic_descriptor_table(agent, cache, realm, BuiltinIntrinsic::Symbol, &descriptors)
}

fn install_symbol_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    symbol: ObjectRef,
    atoms: &PrimitiveDescriptorAtoms,
) -> Result<(), BuiltinBootstrapError> {
    let symbol_atom = agent.bootstrap_atoms().symbol();
    let symbol_tag = descriptor_tag_with_atom(agent, "Symbol", symbol_atom);
    let descriptors = [
        data_atom_property(
            WellKnownAtom::constructor.id(),
            Value::from_object_ref(symbol),
            writable_builtin_attributes(),
        ),
        builtin_function_atom_property(WellKnownAtom::toString.id(), symbol_to_string_builtin()),
        builtin_function_atom_property(WellKnownAtom::valueOf.id(), symbol_value_of_builtin()),
        builtin_function_symbol_property(
            WellKnownSymbolId::ToPrimitive,
            symbol_to_primitive_builtin(),
            readonly_builtin_attributes(),
        ),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            symbol_tag,
            readonly_builtin_attributes(),
        ),
        accessor_atom_property(
            atoms.description,
            Some(symbol_description_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
    ];
    install_intrinsic_descriptor_table(
        agent,
        cache,
        realm,
        BuiltinIntrinsic::SymbolPrototype,
        &descriptors,
    )
}

fn install_builtin_method_group<const N: usize>(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    target: BuiltinIntrinsic,
    specs: [(AtomId, BuiltinFunctionId); N],
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = specs.map(|(atom, entry)| builtin_function_atom_property(atom, entry));
    install_intrinsic_descriptor_table(agent, cache, realm, target, &descriptors)
}

fn install_intrinsic_descriptor_table(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    target: BuiltinIntrinsic,
    descriptors: &[BuiltinPropertyDescriptor],
) -> Result<(), BuiltinBootstrapError> {
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(target),
            descriptors,
        )],
    )
}

fn well_known_symbol_value(
    agent: &Agent,
    symbol: WellKnownSymbolId,
) -> Result<Value, BuiltinBootstrapError> {
    agent
        .well_known_symbol(symbol)
        .map(Value::from_symbol_ref)
        .ok_or(BuiltinBootstrapError::MissingWellKnownSymbol(symbol))
}

#[derive(Clone, Copy)]
struct PrimitiveDescriptorAtoms {
    nan: AtomId,
    is_finite: AtomId,
    is_integer: AtomId,
    is_nan: AtomId,
    is_safe_integer: AtomId,
    parse_float: AtomId,
    parse_int: AtomId,
    positive_infinity: AtomId,
    negative_infinity: AtomId,
    max_value: AtomId,
    min_value: AtomId,
    max_safe_integer: AtomId,
    min_safe_integer: AtomId,
    epsilon: AtomId,
    to_exponential: AtomId,
    to_fixed: AtomId,
    to_locale_string: AtomId,
    to_precision: AtomId,
    abs: AtomId,
    acos: AtomId,
    acosh: AtomId,
    asin: AtomId,
    asinh: AtomId,
    atan: AtomId,
    atan2: AtomId,
    atanh: AtomId,
    cbrt: AtomId,
    ceil: AtomId,
    clz32: AtomId,
    cos: AtomId,
    cosh: AtomId,
    e: AtomId,
    exp: AtomId,
    expm1: AtomId,
    f16round: AtomId,
    floor: AtomId,
    fround: AtomId,
    hypot: AtomId,
    imul: AtomId,
    log: AtomId,
    log10: AtomId,
    log1p: AtomId,
    log2: AtomId,
    ln10: AtomId,
    ln2: AtomId,
    log10e: AtomId,
    log2e: AtomId,
    max: AtomId,
    min: AtomId,
    pi: AtomId,
    pow: AtomId,
    random: AtomId,
    round: AtomId,
    sign: AtomId,
    sin: AtomId,
    sinh: AtomId,
    sqrt: AtomId,
    sqrt1_2: AtomId,
    sqrt2: AtomId,
    sum_precise: AtomId,
    tan: AtomId,
    tanh: AtomId,
    trunc: AtomId,
    as_int_n: AtomId,
    as_uint_n: AtomId,
    key_for: AtomId,
    description: AtomId,
}

impl PrimitiveDescriptorAtoms {
    fn new(agent: &mut Agent) -> Self {
        let nan = agent.bootstrap_atoms().nan();
        let key_for = agent.bootstrap_atoms().key_for();
        Self {
            nan,
            is_finite: agent.atoms_mut().intern("isFinite"),
            is_integer: agent.atoms_mut().intern("isInteger"),
            is_nan: agent.atoms_mut().intern("isNaN"),
            is_safe_integer: agent.atoms_mut().intern("isSafeInteger"),
            parse_float: agent.atoms_mut().intern("parseFloat"),
            parse_int: agent.atoms_mut().intern("parseInt"),
            positive_infinity: agent.atoms_mut().intern("POSITIVE_INFINITY"),
            negative_infinity: agent.atoms_mut().intern("NEGATIVE_INFINITY"),
            max_value: agent.atoms_mut().intern("MAX_VALUE"),
            min_value: agent.atoms_mut().intern("MIN_VALUE"),
            max_safe_integer: agent.atoms_mut().intern("MAX_SAFE_INTEGER"),
            min_safe_integer: agent.atoms_mut().intern("MIN_SAFE_INTEGER"),
            epsilon: agent.atoms_mut().intern("EPSILON"),
            to_exponential: agent.atoms_mut().intern("toExponential"),
            to_fixed: agent.atoms_mut().intern("toFixed"),
            to_locale_string: agent.atoms_mut().intern("toLocaleString"),
            to_precision: agent.atoms_mut().intern("toPrecision"),
            abs: agent.atoms_mut().intern("abs"),
            acos: agent.atoms_mut().intern("acos"),
            acosh: agent.atoms_mut().intern("acosh"),
            asin: agent.atoms_mut().intern("asin"),
            asinh: agent.atoms_mut().intern("asinh"),
            atan: agent.atoms_mut().intern("atan"),
            atan2: agent.atoms_mut().intern("atan2"),
            atanh: agent.atoms_mut().intern("atanh"),
            cbrt: agent.atoms_mut().intern("cbrt"),
            ceil: agent.atoms_mut().intern("ceil"),
            clz32: agent.atoms_mut().intern("clz32"),
            cos: agent.atoms_mut().intern("cos"),
            cosh: agent.atoms_mut().intern("cosh"),
            e: agent.atoms_mut().intern("E"),
            exp: agent.atoms_mut().intern("exp"),
            expm1: agent.atoms_mut().intern("expm1"),
            f16round: agent.atoms_mut().intern("f16round"),
            floor: agent.atoms_mut().intern("floor"),
            fround: agent.atoms_mut().intern("fround"),
            hypot: agent.atoms_mut().intern("hypot"),
            imul: agent.atoms_mut().intern("imul"),
            log: agent.atoms_mut().intern("log"),
            log10: agent.atoms_mut().intern("log10"),
            log1p: agent.atoms_mut().intern("log1p"),
            log2: agent.atoms_mut().intern("log2"),
            ln10: agent.atoms_mut().intern("LN10"),
            ln2: agent.atoms_mut().intern("LN2"),
            log10e: agent.atoms_mut().intern("LOG10E"),
            log2e: agent.atoms_mut().intern("LOG2E"),
            max: agent.atoms_mut().intern("max"),
            min: agent.atoms_mut().intern("min"),
            pi: agent.atoms_mut().intern("PI"),
            pow: agent.atoms_mut().intern("pow"),
            random: agent.atoms_mut().intern("random"),
            round: agent.atoms_mut().intern("round"),
            sign: agent.atoms_mut().intern("sign"),
            sin: agent.atoms_mut().intern("sin"),
            sinh: agent.atoms_mut().intern("sinh"),
            sqrt: agent.atoms_mut().intern("sqrt"),
            sqrt1_2: agent.atoms_mut().intern("SQRT1_2"),
            sqrt2: agent.atoms_mut().intern("SQRT2"),
            sum_precise: agent.atoms_mut().intern("sumPrecise"),
            tan: agent.atoms_mut().intern("tan"),
            tanh: agent.atoms_mut().intern("tanh"),
            trunc: agent.atoms_mut().intern("trunc"),
            as_int_n: agent.atoms_mut().intern("asIntN"),
            as_uint_n: agent.atoms_mut().intern("asUintN"),
            key_for,
            description: agent.atoms_mut().intern("description"),
        }
    }
}

fn number_static_method_specs(
    atoms: &PrimitiveDescriptorAtoms,
) -> [(AtomId, BuiltinFunctionId); 6] {
    [
        (atoms.is_finite, number_is_finite_builtin()),
        (atoms.is_integer, number_is_integer_builtin()),
        (atoms.is_nan, number_is_nan_builtin()),
        (atoms.is_safe_integer, number_is_safe_integer_builtin()),
        (atoms.parse_float, parse_float_builtin()),
        (atoms.parse_int, parse_int_builtin()),
    ]
}

fn number_prototype_method_specs(
    atoms: &PrimitiveDescriptorAtoms,
) -> [(AtomId, BuiltinFunctionId); 6] {
    [
        (atoms.to_exponential, number_to_exponential_builtin()),
        (atoms.to_fixed, number_to_fixed_builtin()),
        (atoms.to_locale_string, number_to_locale_string_builtin()),
        (atoms.to_precision, number_to_precision_builtin()),
        (WellKnownAtom::toString.id(), number_to_string_builtin()),
        (WellKnownAtom::valueOf.id(), number_value_of_builtin()),
    ]
}

fn math_method_specs(atoms: &PrimitiveDescriptorAtoms) -> [(AtomId, BuiltinFunctionId); 37] {
    [
        (atoms.abs, math_abs_builtin()),
        (atoms.acos, math_acos_builtin()),
        (atoms.acosh, math_acosh_builtin()),
        (atoms.asin, math_asin_builtin()),
        (atoms.asinh, math_asinh_builtin()),
        (atoms.atan, math_atan_builtin()),
        (atoms.atan2, math_atan2_builtin()),
        (atoms.atanh, math_atanh_builtin()),
        (atoms.cbrt, math_cbrt_builtin()),
        (atoms.ceil, math_ceil_builtin()),
        (atoms.clz32, math_clz32_builtin()),
        (atoms.cos, math_cos_builtin()),
        (atoms.cosh, math_cosh_builtin()),
        (atoms.exp, math_exp_builtin()),
        (atoms.expm1, math_expm1_builtin()),
        (atoms.f16round, math_f16round_builtin()),
        (atoms.floor, math_floor_builtin()),
        (atoms.fround, math_fround_builtin()),
        (atoms.hypot, math_hypot_builtin()),
        (atoms.imul, math_imul_builtin()),
        (atoms.log, math_log_builtin()),
        (atoms.log10, math_log10_builtin()),
        (atoms.log1p, math_log1p_builtin()),
        (atoms.log2, math_log2_builtin()),
        (atoms.max, math_max_builtin()),
        (atoms.min, math_min_builtin()),
        (atoms.pow, math_pow_builtin()),
        (atoms.random, math_random_builtin()),
        (atoms.round, math_round_builtin()),
        (atoms.sign, math_sign_builtin()),
        (atoms.sin, math_sin_builtin()),
        (atoms.sinh, math_sinh_builtin()),
        (atoms.sqrt, math_sqrt_builtin()),
        (atoms.sum_precise, math_sum_precise_builtin()),
        (atoms.tan, math_tan_builtin()),
        (atoms.tanh, math_tanh_builtin()),
        (atoms.trunc, math_trunc_builtin()),
    ]
}

fn bigint_static_method_specs(
    atoms: &PrimitiveDescriptorAtoms,
) -> [(AtomId, BuiltinFunctionId); 2] {
    [
        (atoms.as_int_n, bigint_as_int_n_builtin()),
        (atoms.as_uint_n, bigint_as_uint_n_builtin()),
    ]
}

fn symbol_static_method_specs(
    atoms: &PrimitiveDescriptorAtoms,
) -> [(AtomId, BuiltinFunctionId); 2] {
    [
        (WellKnownAtom::r#for.id(), symbol_for_builtin()),
        (atoms.key_for, symbol_key_for_builtin()),
    ]
}

fn symbol_well_known_value_descriptors(
    agent: &Agent,
) -> Result<Vec<BuiltinPropertyDescriptor>, BuiltinBootstrapError> {
    let atoms = agent.bootstrap_atoms();
    let specs = [
        (atoms.has_instance(), WellKnownSymbolId::HasInstance),
        (
            atoms.is_concat_spreadable(),
            WellKnownSymbolId::IsConcatSpreadable,
        ),
        (atoms.iterator(), WellKnownSymbolId::Iterator),
        (atoms.async_iterator(), WellKnownSymbolId::AsyncIterator),
        (atoms.match_(), WellKnownSymbolId::Match),
        (atoms.match_all(), WellKnownSymbolId::MatchAll),
        (atoms.replace(), WellKnownSymbolId::Replace),
        (atoms.search(), WellKnownSymbolId::Search),
        (atoms.species(), WellKnownSymbolId::Species),
        (atoms.split(), WellKnownSymbolId::Split),
        (atoms.to_primitive(), WellKnownSymbolId::ToPrimitive),
        (atoms.to_string_tag(), WellKnownSymbolId::ToStringTag),
        (atoms.unscopables(), WellKnownSymbolId::Unscopables),
        (atoms.dispose(), WellKnownSymbolId::Dispose),
        (atoms.async_dispose(), WellKnownSymbolId::AsyncDispose),
    ];
    let mut descriptors = Vec::with_capacity(specs.len());
    for (atom, symbol) in specs {
        descriptors.push(data_atom_property(
            atom,
            well_known_symbol_value(agent, symbol)?,
            hidden_builtin_attributes(),
        ));
    }
    Ok(descriptors)
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
        number: install_public_builtin_function(agent, cx, number_builtin(), Some(prototype)),
        prototype,
        is_finite: install_public_builtin_function(agent, cx, number_is_finite_builtin(), None),
        is_integer: install_public_builtin_function(agent, cx, number_is_integer_builtin(), None),
        is_nan: install_public_builtin_function(agent, cx, number_is_nan_builtin(), None),
        is_safe_integer: install_public_builtin_function(
            agent,
            cx,
            number_is_safe_integer_builtin(),
            None,
        ),
        to_exponential: install_public_builtin_function(
            agent,
            cx,
            number_to_exponential_builtin(),
            None,
        ),
        to_fixed: install_public_builtin_function(agent, cx, number_to_fixed_builtin(), None),
        to_locale_string: install_public_builtin_function(
            agent,
            cx,
            number_to_locale_string_builtin(),
            None,
        ),
        to_precision: install_public_builtin_function(
            agent,
            cx,
            number_to_precision_builtin(),
            None,
        ),
        to_string: install_public_builtin_function(agent, cx, number_to_string_builtin(), None),
        value_of: install_public_builtin_function(agent, cx, number_value_of_builtin(), None),
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
        abs: install_public_builtin_function(agent, cx, math_abs_builtin(), None),
        acos: install_public_builtin_function(agent, cx, math_acos_builtin(), None),
        acosh: install_public_builtin_function(agent, cx, math_acosh_builtin(), None),
        asin: install_public_builtin_function(agent, cx, math_asin_builtin(), None),
        asinh: install_public_builtin_function(agent, cx, math_asinh_builtin(), None),
        atan: install_public_builtin_function(agent, cx, math_atan_builtin(), None),
        atan2: install_public_builtin_function(agent, cx, math_atan2_builtin(), None),
        atanh: install_public_builtin_function(agent, cx, math_atanh_builtin(), None),
        cbrt: install_public_builtin_function(agent, cx, math_cbrt_builtin(), None),
        ceil: install_public_builtin_function(agent, cx, math_ceil_builtin(), None),
        clz32: install_public_builtin_function(agent, cx, math_clz32_builtin(), None),
        cos: install_public_builtin_function(agent, cx, math_cos_builtin(), None),
        cosh: install_public_builtin_function(agent, cx, math_cosh_builtin(), None),
        exp: install_public_builtin_function(agent, cx, math_exp_builtin(), None),
        expm1: install_public_builtin_function(agent, cx, math_expm1_builtin(), None),
        f16round: install_public_builtin_function(agent, cx, math_f16round_builtin(), None),
        floor: install_public_builtin_function(agent, cx, math_floor_builtin(), None),
        fround: install_public_builtin_function(agent, cx, math_fround_builtin(), None),
        hypot: install_public_builtin_function(agent, cx, math_hypot_builtin(), None),
        imul: install_public_builtin_function(agent, cx, math_imul_builtin(), None),
        log: install_public_builtin_function(agent, cx, math_log_builtin(), None),
        log10: install_public_builtin_function(agent, cx, math_log10_builtin(), None),
        log1p: install_public_builtin_function(agent, cx, math_log1p_builtin(), None),
        log2: install_public_builtin_function(agent, cx, math_log2_builtin(), None),
        max: install_public_builtin_function(agent, cx, math_max_builtin(), None),
        min: install_public_builtin_function(agent, cx, math_min_builtin(), None),
        pow: install_public_builtin_function(agent, cx, math_pow_builtin(), None),
        random: install_public_builtin_function(agent, cx, math_random_builtin(), None),
        round: install_public_builtin_function(agent, cx, math_round_builtin(), None),
        sign: install_public_builtin_function(agent, cx, math_sign_builtin(), None),
        sin: install_public_builtin_function(agent, cx, math_sin_builtin(), None),
        sinh: install_public_builtin_function(agent, cx, math_sinh_builtin(), None),
        sqrt: install_public_builtin_function(agent, cx, math_sqrt_builtin(), None),
        sum_precise: install_public_builtin_function(agent, cx, math_sum_precise_builtin(), None),
        tan: install_public_builtin_function(agent, cx, math_tan_builtin(), None),
        tanh: install_public_builtin_function(agent, cx, math_tanh_builtin(), None),
        trunc: install_public_builtin_function(agent, cx, math_trunc_builtin(), None),
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
        bigint: install_public_builtin_function(agent, cx, bigint_builtin(), Some(prototype)),
        as_int_n: install_public_builtin_function(agent, cx, bigint_as_int_n_builtin(), None),
        as_uint_n: install_public_builtin_function(agent, cx, bigint_as_uint_n_builtin(), None),
        prototype,
        to_string: install_public_builtin_function(agent, cx, bigint_to_string_builtin(), None),
        value_of: install_public_builtin_function(agent, cx, bigint_value_of_builtin(), None),
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
        boolean: install_public_builtin_function(agent, cx, boolean_builtin(), Some(prototype)),
        prototype,
        to_string: install_public_builtin_function(agent, cx, boolean_to_string_builtin(), None),
        value_of: install_public_builtin_function(agent, cx, boolean_value_of_builtin(), None),
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
        symbol: install_public_builtin_function(agent, cx, symbol_builtin(), Some(prototype)),
        prototype,
        symbol_for: install_public_builtin_function(agent, cx, symbol_for_builtin(), None),
        key_for: install_public_builtin_function(agent, cx, symbol_key_for_builtin(), None),
        to_string: install_public_builtin_function(agent, cx, symbol_to_string_builtin(), None),
        value_of: install_public_builtin_function(agent, cx, symbol_value_of_builtin(), None),
        to_primitive: install_public_builtin_function(
            agent,
            cx,
            symbol_to_primitive_builtin(),
            None,
        ),
    }
}
