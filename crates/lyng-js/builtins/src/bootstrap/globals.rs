use super::{
    aggregate_error_builtin, array_buffer_builtin, array_builtin, big_int64_array_builtin,
    big_uint64_array_builtin, bigint_builtin, boolean_builtin, data_view_builtin, date_builtin,
    decode_uri_builtin, decode_uri_component_builtin, encode_uri_builtin,
    encode_uri_component_builtin, error_builtin, escape_builtin, eval_builtin, eval_error_builtin,
    finalization_registry_builtin, float16_array_builtin, float32_array_builtin,
    float64_array_builtin, function_builtin, int16_array_builtin, int32_array_builtin,
    int8_array_builtin, is_finite_builtin, is_nan_builtin, map_builtin, number_builtin,
    object_builtin, parse_float_builtin, parse_int_builtin, promise_builtin, range_error_builtin,
    reference_error_builtin, regexp_builtin, set_builtin, shared_array_buffer_builtin,
    string_builtin, symbol_builtin, syntax_error_builtin, type_error_builtin, typed_array_builtin,
    uint16_array_builtin, uint32_array_builtin, uint8_array_builtin, uint8_clamped_array_builtin,
    unescape_builtin, uri_error_builtin, weak_map_builtin, weak_ref_builtin, weak_set_builtin,
    Agent, BootstrapArtifacts, BuiltinAttributes, BuiltinPropertyDescriptor,
    BuiltinPropertyKeySpec, BuiltinPropertyValueSpec, RealmRecord, Value, WellKnownAtom,
};

#[allow(
    clippy::too_many_lines,
    reason = "global descriptor installation is a single ordered ECMA bootstrap table"
)]
pub(super) fn default_global_descriptors(
    agent: &mut Agent,
    artifacts: BootstrapArtifacts,
) -> [BuiltinPropertyDescriptor; 65] {
    let atoms = agent.bootstrap_atoms();
    let reflect_atom = agent.atoms_mut().intern("Reflect");
    let proxy_atom = agent.atoms_mut().intern("Proxy");
    let suppressed_error_atom = agent.atoms_mut().intern("SuppressedError");
    let disposable_stack_atom = agent.atoms_mut().intern("DisposableStack");
    let async_disposable_stack_atom = agent.atoms_mut().intern("AsyncDisposableStack");
    let iterator_atom = agent.atoms_mut().intern("Iterator");
    let intrinsics = agent
        .realm(artifacts.realm())
        .map(RealmRecord::intrinsics)
        .unwrap_or_default();

    [
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.global_this()),
            BuiltinPropertyValueSpec::Data(Value::from_object_ref(artifacts.global_object())),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.infinity()),
            BuiltinPropertyValueSpec::Data(Value::from_f64(f64::INFINITY)),
            BuiltinAttributes::new(false, false, false),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.nan()),
            BuiltinPropertyValueSpec::Data(Value::from_f64(f64::NAN)),
            BuiltinAttributes::new(false, false, false),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.undefined()),
            BuiltinPropertyValueSpec::Data(Value::undefined()),
            BuiltinAttributes::new(false, false, false),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.object()),
            BuiltinPropertyValueSpec::BuiltinFunction(object_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.json()),
            BuiltinPropertyValueSpec::Data(
                intrinsics
                    .json()
                    .map_or(Value::undefined(), Value::from_object_ref),
            ),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.function()),
            BuiltinPropertyValueSpec::BuiltinFunction(function_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.array()),
            BuiltinPropertyValueSpec::BuiltinFunction(array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.map()),
            BuiltinPropertyValueSpec::BuiltinFunction(map_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.set()),
            BuiltinPropertyValueSpec::BuiltinFunction(set_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.weak_map()),
            BuiltinPropertyValueSpec::BuiltinFunction(weak_map_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.weak_set()),
            BuiltinPropertyValueSpec::BuiltinFunction(weak_set_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.weak_ref()),
            BuiltinPropertyValueSpec::BuiltinFunction(weak_ref_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.finalization_registry()),
            BuiltinPropertyValueSpec::BuiltinFunction(finalization_registry_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.string()),
            BuiltinPropertyValueSpec::BuiltinFunction(string_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.regexp()),
            BuiltinPropertyValueSpec::BuiltinFunction(regexp_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.date()),
            BuiltinPropertyValueSpec::BuiltinFunction(date_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.array_buffer()),
            BuiltinPropertyValueSpec::BuiltinFunction(array_buffer_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.shared_array_buffer()),
            BuiltinPropertyValueSpec::BuiltinFunction(shared_array_buffer_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.data_view()),
            BuiltinPropertyValueSpec::BuiltinFunction(data_view_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.atomics()),
            BuiltinPropertyValueSpec::Data(Value::from_object_ref(
                intrinsics
                    .atomics()
                    .expect("Atomics intrinsic should be bootstrapped before globals"),
            )),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.typed_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(typed_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.int8_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(int8_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.int16_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(int16_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.int32_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(int32_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.float16_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(float16_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.uint32_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(uint32_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.float32_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(float32_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.float64_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(float64_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.big_int64_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(big_int64_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.big_uint64_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(big_uint64_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.uint16_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(uint16_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.uint8_clamped_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(uint8_clamped_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.uint8_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(uint8_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.number()),
            BuiltinPropertyValueSpec::BuiltinFunction(number_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.math()),
            BuiltinPropertyValueSpec::Data(
                intrinsics
                    .math()
                    .map_or(Value::undefined(), Value::from_object_ref),
            ),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.bigint()),
            BuiltinPropertyValueSpec::BuiltinFunction(bigint_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.boolean()),
            BuiltinPropertyValueSpec::BuiltinFunction(boolean_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.symbol()),
            BuiltinPropertyValueSpec::BuiltinFunction(symbol_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.promise()),
            BuiltinPropertyValueSpec::BuiltinFunction(promise_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(reflect_atom),
            BuiltinPropertyValueSpec::Data(
                intrinsics
                    .reflect()
                    .map_or(Value::undefined(), Value::from_object_ref),
            ),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(proxy_atom),
            BuiltinPropertyValueSpec::BuiltinFunction(lyng_js_types::proxy_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.aggregate_error()),
            BuiltinPropertyValueSpec::BuiltinFunction(aggregate_error_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(suppressed_error_atom),
            BuiltinPropertyValueSpec::BuiltinFunction(lyng_js_types::suppressed_error_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(disposable_stack_atom),
            BuiltinPropertyValueSpec::BuiltinFunction(lyng_js_types::disposable_stack_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(async_disposable_stack_atom),
            BuiltinPropertyValueSpec::BuiltinFunction(
                lyng_js_types::async_disposable_stack_builtin(),
            ),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(iterator_atom),
            BuiltinPropertyValueSpec::BuiltinFunction(lyng_js_types::iterator_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(WellKnownAtom::eval.id()),
            BuiltinPropertyValueSpec::BuiltinFunction(eval_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.error()),
            BuiltinPropertyValueSpec::BuiltinFunction(error_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.eval_error()),
            BuiltinPropertyValueSpec::BuiltinFunction(eval_error_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.range_error()),
            BuiltinPropertyValueSpec::BuiltinFunction(range_error_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.reference_error()),
            BuiltinPropertyValueSpec::BuiltinFunction(reference_error_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.syntax_error()),
            BuiltinPropertyValueSpec::BuiltinFunction(syntax_error_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.type_error()),
            BuiltinPropertyValueSpec::BuiltinFunction(type_error_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.uri_error()),
            BuiltinPropertyValueSpec::BuiltinFunction(uri_error_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.parse_int()),
            BuiltinPropertyValueSpec::BuiltinFunction(parse_int_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.parse_float()),
            BuiltinPropertyValueSpec::BuiltinFunction(parse_float_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.is_nan()),
            BuiltinPropertyValueSpec::BuiltinFunction(is_nan_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.is_finite()),
            BuiltinPropertyValueSpec::BuiltinFunction(is_finite_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.decode_uri()),
            BuiltinPropertyValueSpec::BuiltinFunction(decode_uri_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.decode_uri_component()),
            BuiltinPropertyValueSpec::BuiltinFunction(decode_uri_component_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.encode_uri()),
            BuiltinPropertyValueSpec::BuiltinFunction(encode_uri_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.encode_uri_component()),
            BuiltinPropertyValueSpec::BuiltinFunction(encode_uri_component_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.escape()),
            BuiltinPropertyValueSpec::BuiltinFunction(escape_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.unescape()),
            BuiltinPropertyValueSpec::BuiltinFunction(unescape_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
    ]
}
