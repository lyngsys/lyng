use super::*;
use lyng_js_common::{AtomId, AtomLifetime};
use lyng_js_gc::{AllocationLifetime, AtomGcSweep};
use lyng_js_host::NoopHostHooks;
use lyng_js_types::{
    array_buffer_byte_length_getter_builtin, array_buffer_is_view_builtin,
    array_buffer_slice_builtin, array_from_async_builtin, array_iterator_next_builtin,
    array_species_getter_builtin, array_values_builtin,
    async_disposable_stack_dispose_async_builtin, async_iterator_dispose_builtin,
    atomics_add_builtin, bigint_as_int_n_builtin, bigint_to_string_builtin,
    boolean_to_string_builtin, data_view_buffer_getter_builtin, data_view_get_uint8_builtin,
    date_get_time_builtin, date_now_builtin, date_set_full_year_builtin, date_to_primitive_builtin,
    date_to_string_builtin, disposable_stack_dispose_builtin,
    disposable_stack_disposed_getter_builtin, disposable_stack_use_builtin,
    error_to_string_builtin, escape_builtin, iterator_prototype_iterator_builtin,
    json_parse_builtin, json_raw_json_builtin, map_iterator_next_builtin, map_size_getter_builtin,
    math_abs_builtin, number_is_finite_builtin, number_to_string_builtin, promise_resolve_builtin,
    promise_species_getter_builtin, promise_then_builtin, proxy_revocable_builtin,
    reflect_get_builtin, regexp_escape_builtin, regexp_exec_builtin, regexp_global_getter_builtin,
    regexp_species_getter_builtin, regexp_symbol_match_builtin, set_iterator_next_builtin,
    set_values_builtin, string_from_char_code_builtin, string_iterator_builtin,
    string_iterator_next_builtin, string_trim_builtin, symbol_description_getter_builtin,
    symbol_for_builtin, symbol_to_primitive_builtin, typed_array_from_builtin,
    typed_array_to_string_tag_getter_builtin, uint8_array_buffer_getter_builtin,
    uint8_array_values_builtin, unescape_builtin, weak_ref_deref_builtin, PropertyKey, Value,
};

fn assert_reachable_object_atom_keys_are_permanent(
    agent: &Agent,
    roots: &[ObjectRef],
    atoms: &mut Vec<AtomId>,
) {
    let mut stack = roots.to_vec();
    let mut visited = Vec::new();

    while let Some(object) = stack.pop() {
        if visited.contains(&object) {
            continue;
        }
        visited.push(object);

        let keys = agent
            .objects()
            .own_property_keys(agent.heap().view(), object)
            .unwrap_or_else(|err| panic!("{object:?} own keys should be queryable: {err:?}"));

        for key in keys {
            if let PropertyKey::Atom(atom) = key {
                let name = agent.atoms().get(atom).unwrap_or("<utf16>");
                assert_eq!(
                    agent.atoms().lifetime(atom),
                    Some(AtomLifetime::Permanent),
                    "{object:?}.{name} should be a permanent bootstrap atom",
                );
                atoms.push(atom);
            }

            let descriptor = agent
                .objects()
                .get_own_property(agent.heap().view(), object, key)
                .unwrap_or_else(|err| panic!("{object:?} descriptor should be queryable: {err:?}"))
                .unwrap_or_else(|| panic!("{object:?} descriptor should exist for {key:?}"));
            for value in [descriptor.value(), descriptor.getter(), descriptor.setter()]
                .into_iter()
                .flatten()
            {
                if let Some(child) = value.as_object_ref() {
                    stack.push(child);
                }
            }
        }
    }
}

fn own_descriptor(
    agent: &Agent,
    object: ObjectRef,
    key: PropertyKey,
    name: &str,
) -> PropertyDescriptor {
    agent
        .objects()
        .get_own_property(agent.heap().view(), object, key)
        .unwrap()
        .unwrap_or_else(|| panic!("{name} should be installed"))
}

#[test]
fn bootstrap_atom_property_keys_are_permanent_after_sweep() {
    let mut runtime = lyng_js_env::Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let mut cache = BuiltinCache::new();

    let artifacts = bootstrap_default_realm(
        agent,
        &mut cache,
        BootstrapRequest::new(BootstrapMode::SpecOnly),
    )
    .expect("spec bootstrap should succeed");
    let intrinsics = agent
        .realm(artifacts.realm())
        .expect("default realm should exist")
        .intrinsics();
    let unscopables_symbol = agent
        .well_known_symbol(WellKnownSymbolId::Unscopables)
        .expect("Symbol.unscopables should exist");
    let array_unscopables = own_descriptor(
        agent,
        intrinsics
            .array_prototype()
            .expect("Array.prototype intrinsic should exist"),
        PropertyKey::from_symbol(unscopables_symbol),
        "Array.prototype[Symbol.unscopables]",
    )
    .value()
    .and_then(Value::as_object_ref)
    .expect("Array.prototype[Symbol.unscopables] should be an object");

    let roots = [
        Some(artifacts.global_object()),
        intrinsics.object(),
        intrinsics.object_prototype(),
        intrinsics.function(),
        intrinsics.function_prototype(),
        intrinsics.array(),
        intrinsics.array_prototype(),
        intrinsics.array_iterator_prototype(),
        Some(array_unscopables),
        intrinsics.map(),
        intrinsics.map_prototype(),
        intrinsics.set_prototype(),
        intrinsics.weak_ref_prototype(),
        intrinsics.json(),
        intrinsics.reflect(),
        intrinsics.regexp(),
        intrinsics.regexp_prototype(),
        intrinsics.date(),
        intrinsics.date_prototype(),
        intrinsics.number(),
        intrinsics.math(),
        intrinsics.bigint(),
        intrinsics.symbol_prototype(),
        intrinsics.string(),
        intrinsics.string_prototype(),
        intrinsics.string_iterator_prototype(),
        intrinsics.promise(),
        intrinsics.promise_prototype(),
        intrinsics.disposable_stack_prototype(),
        intrinsics.async_disposable_stack_prototype(),
        intrinsics.array_buffer(),
        intrinsics.array_buffer_prototype(),
        intrinsics.data_view_prototype(),
        intrinsics.atomics(),
        intrinsics.typed_array(),
        intrinsics.typed_array_prototype(),
        intrinsics.uint8_array(),
        intrinsics.uint8_array_prototype(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    let mut retained_atoms = Vec::new();
    assert_reachable_object_atom_keys_are_permanent(agent, &roots, &mut retained_atoms);

    let _ = AtomGcSweep::new(agent.atoms_mut()).sweep();

    for atom in retained_atoms {
        assert_eq!(
            agent.atoms().lifetime(atom),
            Some(AtomLifetime::Permanent),
            "bootstrap atom {atom:?} should stay permanent after sweep",
        );
        assert!(
            agent.atoms().get(atom).is_some(),
            "bootstrap atom {atom:?} should resolve after sweep",
        );
    }
}

#[test]
fn shared_default_realm_bootstrap_installs_typed_global_descriptors() {
    let mut runtime = lyng_js_env::Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let mut cache = BuiltinCache::new();

    let artifacts = bootstrap_default_realm(
        agent,
        &mut cache,
        BootstrapRequest::new(BootstrapMode::SpecOnly),
    )
    .expect("spec bootstrap should succeed");
    let second = bootstrap_default_realm(
        agent,
        &mut cache,
        BootstrapRequest::new(BootstrapMode::SpecOnly),
    )
    .expect("repeated spec bootstrap should stay idempotent");
    let global = agent
        .realm(artifacts.realm())
        .expect("default realm should remain queryable")
        .global_object();
    let atoms = agent.bootstrap_atoms();

    assert_eq!(artifacts, second);
    assert_eq!(
        agent
            .realm(artifacts.realm())
            .expect("default realm should exist")
            .bootstrap_state(),
        RealmBootstrapState::new().with_spec_ready(true)
    );

    let global_this = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.global_this()),
        )
        .unwrap()
        .expect("globalThis should be installed");
    let infinity = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.infinity()),
        )
        .unwrap()
        .expect("Infinity should be installed");
    let nan = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.nan()),
        )
        .unwrap()
        .expect("NaN should be installed");
    let undefined = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.undefined()),
        )
        .unwrap()
        .expect("undefined should be installed");
    let object = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.object()),
        )
        .unwrap()
        .expect("Object should be installed");
    let function = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.function()),
        )
        .unwrap()
        .expect("Function should be installed");
    let string = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.string()),
        )
        .unwrap()
        .expect("String should be installed");
    let regexp = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.regexp()),
        )
        .unwrap()
        .expect("RegExp should be installed");
    let date = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.date()),
        )
        .unwrap()
        .expect("Date should be installed");
    let number = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.number()),
        )
        .unwrap()
        .expect("Number should be installed");
    let boolean = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.boolean()),
        )
        .unwrap()
        .expect("Boolean should be installed");
    let symbol = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.symbol()),
        )
        .unwrap()
        .expect("Symbol should be installed");
    let bigint = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.bigint()),
        )
        .unwrap()
        .expect("BigInt should be installed");
    let error = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.error()),
        )
        .unwrap()
        .expect("Error should be installed");
    let type_error = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.type_error()),
        )
        .unwrap()
        .expect("TypeError should be installed");
    let eval_error = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.eval_error()),
        )
        .unwrap()
        .expect("EvalError should be installed");
    let range_error = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.range_error()),
        )
        .unwrap()
        .expect("RangeError should be installed");
    let reference_error = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.reference_error()),
        )
        .unwrap()
        .expect("ReferenceError should be installed");
    let syntax_error = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.syntax_error()),
        )
        .unwrap()
        .expect("SyntaxError should be installed");
    let uri_error = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.uri_error()),
        )
        .unwrap()
        .expect("URIError should be installed");
    let parse_int = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.parse_int()),
        )
        .unwrap()
        .expect("parseInt should be installed");
    let parse_float = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.parse_float()),
        )
        .unwrap()
        .expect("parseFloat should be installed");
    let is_nan = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.is_nan()),
        )
        .unwrap()
        .expect("isNaN should be installed");
    let is_finite = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.is_finite()),
        )
        .unwrap()
        .expect("isFinite should be installed");
    let decode_uri = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.decode_uri()),
        )
        .unwrap()
        .expect("decodeURI should be installed");
    let decode_uri_component = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.decode_uri_component()),
        )
        .unwrap()
        .expect("decodeURIComponent should be installed");
    let encode_uri = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.encode_uri()),
        )
        .unwrap()
        .expect("encodeURI should be installed");
    let encode_uri_component = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.encode_uri_component()),
        )
        .unwrap()
        .expect("encodeURIComponent should be installed");
    let escape = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.escape()),
        )
        .unwrap()
        .expect("escape should be installed");
    let unescape = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.unescape()),
        )
        .unwrap()
        .expect("unescape should be installed");
    let reflect_atom = agent.atoms_mut().intern_collectible("Reflect");
    let math = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(atoms.math()),
        )
        .unwrap()
        .expect("Math should be installed");
    let reflect = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            global,
            PropertyKey::from_atom(reflect_atom),
        )
        .unwrap();

    assert_eq!(
        global_this.value(),
        Some(Value::from_object_ref(artifacts.global_object()))
    );
    assert_eq!(global_this.writable(), Some(true));
    assert_eq!(global_this.enumerable(), Some(false));
    assert_eq!(global_this.configurable(), Some(true));
    assert_eq!(infinity.value(), Some(Value::from_f64(f64::INFINITY)));
    assert_eq!(infinity.writable(), Some(false));
    assert_eq!(infinity.enumerable(), Some(false));
    assert_eq!(infinity.configurable(), Some(false));
    assert!(nan.value().unwrap().as_f64().unwrap().is_nan());
    assert_eq!(nan.writable(), Some(false));
    assert_eq!(nan.enumerable(), Some(false));
    assert_eq!(nan.configurable(), Some(false));
    assert_eq!(undefined.value(), Some(Value::undefined()));
    assert_eq!(undefined.writable(), Some(false));
    assert_eq!(undefined.enumerable(), Some(false));
    assert_eq!(undefined.configurable(), Some(false));
    assert_eq!(
        object.value(),
        agent
            .realm(artifacts.realm())
            .map(|realm| realm.intrinsics().object())
            .flatten()
            .map(Value::from_object_ref)
    );
    assert_eq!(object.writable(), Some(true));
    assert_eq!(object.enumerable(), Some(false));
    assert_eq!(object.configurable(), Some(true));
    assert_eq!(
        function.value(),
        agent
            .realm(artifacts.realm())
            .map(|realm| realm.intrinsics().function())
            .flatten()
            .map(Value::from_object_ref)
    );
    assert_eq!(function.writable(), Some(true));
    assert_eq!(function.enumerable(), Some(false));
    assert_eq!(function.configurable(), Some(true));
    assert_eq!(
        string.value(),
        agent
            .realm(artifacts.realm())
            .map(|realm| realm.intrinsics().string())
            .flatten()
            .map(Value::from_object_ref)
    );
    assert_eq!(string.writable(), Some(true));
    assert_eq!(string.enumerable(), Some(false));
    assert_eq!(string.configurable(), Some(true));
    assert_eq!(
        regexp.value(),
        agent
            .realm(artifacts.realm())
            .map(|realm| realm.intrinsics().regexp())
            .flatten()
            .map(Value::from_object_ref)
    );
    assert_eq!(regexp.writable(), Some(true));
    assert_eq!(regexp.enumerable(), Some(false));
    assert_eq!(regexp.configurable(), Some(true));
    assert_eq!(
        date.value(),
        agent
            .realm(artifacts.realm())
            .map(|realm| realm.intrinsics().date())
            .flatten()
            .map(Value::from_object_ref)
    );
    assert_eq!(date.writable(), Some(true));
    assert_eq!(date.enumerable(), Some(false));
    assert_eq!(date.configurable(), Some(true));
    assert_eq!(
        number.value(),
        agent
            .realm(artifacts.realm())
            .map(|realm| realm.intrinsics().number())
            .flatten()
            .map(Value::from_object_ref)
    );
    assert_eq!(number.writable(), Some(true));
    assert_eq!(number.enumerable(), Some(false));
    assert_eq!(number.configurable(), Some(true));
    assert_eq!(
        boolean.value(),
        agent
            .realm(artifacts.realm())
            .map(|realm| realm.intrinsics().boolean())
            .flatten()
            .map(Value::from_object_ref)
    );
    assert_eq!(boolean.writable(), Some(true));
    assert_eq!(boolean.enumerable(), Some(false));
    assert_eq!(boolean.configurable(), Some(true));
    assert_eq!(
        symbol.value(),
        agent
            .realm(artifacts.realm())
            .map(|realm| realm.intrinsics().symbol())
            .flatten()
            .map(Value::from_object_ref)
    );
    assert_eq!(symbol.writable(), Some(true));
    assert_eq!(symbol.enumerable(), Some(false));
    assert_eq!(symbol.configurable(), Some(true));
    assert_eq!(
        bigint.value(),
        agent
            .realm(artifacts.realm())
            .map(|realm| realm.intrinsics().bigint())
            .flatten()
            .map(Value::from_object_ref)
    );
    assert_eq!(bigint.writable(), Some(true));
    assert_eq!(bigint.enumerable(), Some(false));
    assert_eq!(bigint.configurable(), Some(true));
    assert_eq!(
        error.value(),
        agent
            .realm(artifacts.realm())
            .map(|realm| realm.intrinsics().error())
            .flatten()
            .map(Value::from_object_ref)
    );
    assert_eq!(error.writable(), Some(true));
    assert_eq!(error.enumerable(), Some(false));
    assert_eq!(error.configurable(), Some(true));
    assert_eq!(
        type_error.value(),
        agent
            .realm(artifacts.realm())
            .map(|realm| realm.intrinsics().type_error())
            .flatten()
            .map(Value::from_object_ref)
    );
    assert_eq!(type_error.writable(), Some(true));
    assert_eq!(type_error.enumerable(), Some(false));
    assert_eq!(type_error.configurable(), Some(true));
    assert_eq!(
        eval_error.value(),
        agent
            .realm(artifacts.realm())
            .map(|realm| realm.intrinsics().eval_error())
            .flatten()
            .map(Value::from_object_ref)
    );
    assert_eq!(eval_error.writable(), Some(true));
    assert_eq!(eval_error.enumerable(), Some(false));
    assert_eq!(eval_error.configurable(), Some(true));
    assert_eq!(
        range_error.value(),
        agent
            .realm(artifacts.realm())
            .map(|realm| realm.intrinsics().range_error())
            .flatten()
            .map(Value::from_object_ref)
    );
    assert_eq!(range_error.writable(), Some(true));
    assert_eq!(range_error.enumerable(), Some(false));
    assert_eq!(range_error.configurable(), Some(true));
    assert_eq!(
        reference_error.value(),
        agent
            .realm(artifacts.realm())
            .map(|realm| realm.intrinsics().reference_error())
            .flatten()
            .map(Value::from_object_ref)
    );
    assert_eq!(reference_error.writable(), Some(true));
    assert_eq!(reference_error.enumerable(), Some(false));
    assert_eq!(reference_error.configurable(), Some(true));
    assert_eq!(
        math.value(),
        agent
            .realm(artifacts.realm())
            .map(|realm| realm.intrinsics().math())
            .flatten()
            .map(Value::from_object_ref)
    );
    assert_eq!(math.writable(), Some(true));
    assert_eq!(math.enumerable(), Some(false));
    assert_eq!(math.configurable(), Some(true));
    assert_eq!(
        syntax_error.value(),
        agent
            .realm(artifacts.realm())
            .map(|realm| realm.intrinsics().syntax_error())
            .flatten()
            .map(Value::from_object_ref)
    );
    assert_eq!(syntax_error.writable(), Some(true));
    assert_eq!(syntax_error.enumerable(), Some(false));
    assert_eq!(syntax_error.configurable(), Some(true));
    assert_eq!(
        uri_error.value(),
        agent
            .realm(artifacts.realm())
            .map(|realm| realm.intrinsics().uri_error())
            .flatten()
            .map(Value::from_object_ref)
    );
    assert_eq!(uri_error.writable(), Some(true));
    assert_eq!(uri_error.enumerable(), Some(false));
    assert_eq!(uri_error.configurable(), Some(true));
    assert!(parse_int.value().and_then(Value::as_object_ref).is_some());
    assert_eq!(parse_int.writable(), Some(true));
    assert_eq!(parse_int.enumerable(), Some(false));
    assert_eq!(parse_int.configurable(), Some(true));
    assert!(parse_float.value().and_then(Value::as_object_ref).is_some());
    assert_eq!(parse_float.writable(), Some(true));
    assert_eq!(parse_float.enumerable(), Some(false));
    assert_eq!(parse_float.configurable(), Some(true));
    assert!(is_nan.value().and_then(Value::as_object_ref).is_some());
    assert_eq!(is_nan.writable(), Some(true));
    assert_eq!(is_nan.enumerable(), Some(false));
    assert_eq!(is_nan.configurable(), Some(true));
    assert!(is_finite.value().and_then(Value::as_object_ref).is_some());
    assert_eq!(is_finite.writable(), Some(true));
    assert_eq!(is_finite.enumerable(), Some(false));
    assert_eq!(is_finite.configurable(), Some(true));
    assert!(decode_uri.value().and_then(Value::as_object_ref).is_some());
    assert_eq!(decode_uri.writable(), Some(true));
    assert_eq!(decode_uri.enumerable(), Some(false));
    assert_eq!(decode_uri.configurable(), Some(true));
    assert!(decode_uri_component
        .value()
        .and_then(Value::as_object_ref)
        .is_some());
    assert_eq!(decode_uri_component.writable(), Some(true));
    assert_eq!(decode_uri_component.enumerable(), Some(false));
    assert_eq!(decode_uri_component.configurable(), Some(true));
    assert!(encode_uri.value().and_then(Value::as_object_ref).is_some());
    assert_eq!(encode_uri.writable(), Some(true));
    assert_eq!(encode_uri.enumerable(), Some(false));
    assert_eq!(encode_uri.configurable(), Some(true));
    assert!(encode_uri_component
        .value()
        .and_then(Value::as_object_ref)
        .is_some());
    assert_eq!(encode_uri_component.writable(), Some(true));
    assert_eq!(encode_uri_component.enumerable(), Some(false));
    assert_eq!(encode_uri_component.configurable(), Some(true));
    assert_eq!(
        escape.value(),
        cache.builtin_constant(agent, artifacts.realm(), escape_builtin())
    );
    assert_eq!(escape.writable(), Some(true));
    assert_eq!(escape.enumerable(), Some(false));
    assert_eq!(escape.configurable(), Some(true));
    assert_eq!(
        unescape.value(),
        cache.builtin_constant(agent, artifacts.realm(), unescape_builtin())
    );
    assert_eq!(unescape.writable(), Some(true));
    assert_eq!(unescape.enumerable(), Some(false));
    assert_eq!(unescape.configurable(), Some(true));
    let reflect = reflect.expect("Reflect should be installed");
    assert!(reflect.value().and_then(Value::as_object_ref).is_some());
    assert_eq!(reflect.writable(), Some(true));
    assert_eq!(reflect.enumerable(), Some(false));
    assert_eq!(reflect.configurable(), Some(true));
}

#[test]
fn shared_bootstrap_installs_array_family_descriptors() {
    let mut runtime = lyng_js_env::Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let mut cache = BuiltinCache::new();

    let artifacts = bootstrap_default_realm(
        agent,
        &mut cache,
        BootstrapRequest::new(BootstrapMode::SpecOnly),
    )
    .expect("spec bootstrap should succeed");
    let intrinsics = agent
        .realm(artifacts.realm())
        .expect("default realm should exist")
        .intrinsics();
    let array = intrinsics.array().expect("Array intrinsic should exist");
    let array_prototype = intrinsics
        .array_prototype()
        .expect("Array.prototype intrinsic should exist");
    let array_iterator_prototype = intrinsics
        .array_iterator_prototype()
        .expect("Array Iterator prototype intrinsic should exist");

    let from_async_atom = agent.atoms_mut().intern_collectible("fromAsync");
    let flat_atom = agent.atoms_mut().intern_collectible("flat");
    let length_atom = WellKnownAtom::length.id();
    let next_atom = agent.atoms_mut().intern_collectible("next");
    let species_symbol = agent
        .well_known_symbol(WellKnownSymbolId::Species)
        .expect("Symbol.species should exist");
    let iterator_symbol = agent
        .well_known_symbol(WellKnownSymbolId::Iterator)
        .expect("Symbol.iterator should exist");
    let unscopables_symbol = agent
        .well_known_symbol(WellKnownSymbolId::Unscopables)
        .expect("Symbol.unscopables should exist");

    let from_async_value = cache
        .builtin_constant(agent, artifacts.realm(), array_from_async_builtin())
        .expect("Array.fromAsync builtin should resolve");
    let species_getter = cache
        .builtin_constant(agent, artifacts.realm(), array_species_getter_builtin())
        .expect("Array @@species getter should resolve");
    let values_value = cache
        .builtin_constant(agent, artifacts.realm(), array_values_builtin())
        .expect("Array.prototype.values builtin should resolve");
    let iterator_next = cache
        .builtin_constant(agent, artifacts.realm(), array_iterator_next_builtin())
        .expect("Array Iterator next builtin should resolve");

    let from_async = own_descriptor(
        agent,
        array,
        PropertyKey::from_atom(from_async_atom),
        "Array.fromAsync",
    );
    assert_eq!(from_async.value(), Some(from_async_value));
    assert_eq!(from_async.writable(), Some(true));
    assert_eq!(from_async.enumerable(), Some(false));
    assert_eq!(from_async.configurable(), Some(true));

    let species = own_descriptor(
        agent,
        array,
        PropertyKey::from_symbol(species_symbol),
        "Array[Symbol.species]",
    );
    assert_eq!(species.getter(), Some(species_getter));
    assert_eq!(species.setter(), Some(Value::undefined()));
    assert_eq!(species.enumerable(), Some(false));
    assert_eq!(species.configurable(), Some(true));

    let length = own_descriptor(
        agent,
        array_prototype,
        PropertyKey::from_atom(length_atom),
        "Array.prototype.length",
    );
    assert_eq!(length.value(), Some(Value::from_smi(0)));
    assert_eq!(length.writable(), Some(true));
    assert_eq!(length.enumerable(), Some(false));
    assert_eq!(length.configurable(), Some(false));

    let unscopables = own_descriptor(
        agent,
        array_prototype,
        PropertyKey::from_symbol(unscopables_symbol),
        "Array.prototype[Symbol.unscopables]",
    );
    let unscopables_object = unscopables
        .value()
        .and_then(Value::as_object_ref)
        .expect("Array unscopables should be an object");
    assert_eq!(unscopables.writable(), Some(false));
    assert_eq!(unscopables.enumerable(), Some(false));
    assert_eq!(unscopables.configurable(), Some(true));

    let unscopables_flat = own_descriptor(
        agent,
        unscopables_object,
        PropertyKey::from_atom(flat_atom),
        "Array.prototype[Symbol.unscopables].flat",
    );
    assert_eq!(unscopables_flat.value(), Some(Value::from_bool(true)));
    assert_eq!(unscopables_flat.writable(), Some(true));
    assert_eq!(unscopables_flat.enumerable(), Some(true));
    assert_eq!(unscopables_flat.configurable(), Some(true));

    let iterator = own_descriptor(
        agent,
        array_prototype,
        PropertyKey::from_symbol(iterator_symbol),
        "Array.prototype[Symbol.iterator]",
    );
    assert_eq!(iterator.value(), Some(values_value));
    assert_eq!(iterator.writable(), Some(true));
    assert_eq!(iterator.enumerable(), Some(false));
    assert_eq!(iterator.configurable(), Some(true));

    let next = own_descriptor(
        agent,
        array_iterator_prototype,
        PropertyKey::from_atom(next_atom),
        "Array Iterator prototype.next",
    );
    assert_eq!(next.value(), Some(iterator_next));
    assert_eq!(next.writable(), Some(true));
    assert_eq!(next.enumerable(), Some(false));
    assert_eq!(next.configurable(), Some(true));
}

#[test]
fn shared_bootstrap_installs_collection_family_descriptors() {
    let mut runtime = lyng_js_env::Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let mut cache = BuiltinCache::new();

    let artifacts = bootstrap_default_realm(
        agent,
        &mut cache,
        BootstrapRequest::new(BootstrapMode::SpecOnly),
    )
    .expect("spec bootstrap should succeed");
    let intrinsics = agent
        .realm(artifacts.realm())
        .expect("default realm should exist")
        .intrinsics();
    let map = intrinsics.map().expect("Map intrinsic should exist");
    let map_prototype = intrinsics
        .map_prototype()
        .expect("Map.prototype intrinsic should exist");
    let set_prototype = intrinsics
        .set_prototype()
        .expect("Set.prototype intrinsic should exist");
    let weak_ref_prototype = intrinsics
        .weak_ref_prototype()
        .expect("WeakRef.prototype intrinsic should exist");

    let size_atom = agent.atoms_mut().intern_collectible("size");
    let keys_atom = agent.atoms_mut().intern_collectible("keys");
    let deref_atom = agent.atoms_mut().intern_collectible("deref");
    let species_symbol = agent
        .well_known_symbol(WellKnownSymbolId::Species)
        .expect("Symbol.species should exist");

    let species_getter = cache
        .builtin_constant(agent, artifacts.realm(), array_species_getter_builtin())
        .expect("collection @@species getter should resolve");
    let map_size_getter = cache
        .builtin_constant(agent, artifacts.realm(), map_size_getter_builtin())
        .expect("Map.prototype.size getter should resolve");
    let set_values = cache
        .builtin_constant(agent, artifacts.realm(), set_values_builtin())
        .expect("Set.prototype.values builtin should resolve");
    let weak_ref_deref = cache
        .builtin_constant(agent, artifacts.realm(), weak_ref_deref_builtin())
        .expect("WeakRef.prototype.deref builtin should resolve");

    let map_species = own_descriptor(
        agent,
        map,
        PropertyKey::from_symbol(species_symbol),
        "Map[Symbol.species]",
    );
    assert_eq!(map_species.getter(), Some(species_getter));
    assert_eq!(map_species.setter(), Some(Value::undefined()));
    assert_eq!(map_species.enumerable(), Some(false));
    assert_eq!(map_species.configurable(), Some(true));

    let map_size = own_descriptor(
        agent,
        map_prototype,
        PropertyKey::from_atom(size_atom),
        "Map.prototype.size",
    );
    assert_eq!(map_size.getter(), Some(map_size_getter));
    assert_eq!(map_size.setter(), Some(Value::undefined()));
    assert_eq!(map_size.enumerable(), Some(false));
    assert_eq!(map_size.configurable(), Some(true));

    let set_keys = own_descriptor(
        agent,
        set_prototype,
        PropertyKey::from_atom(keys_atom),
        "Set.prototype.keys",
    );
    assert_eq!(set_keys.value(), Some(set_values));
    assert_eq!(set_keys.writable(), Some(true));
    assert_eq!(set_keys.enumerable(), Some(false));
    assert_eq!(set_keys.configurable(), Some(true));

    let weak_ref_deref_descriptor = own_descriptor(
        agent,
        weak_ref_prototype,
        PropertyKey::from_atom(deref_atom),
        "WeakRef.prototype.deref",
    );
    assert_eq!(weak_ref_deref_descriptor.value(), Some(weak_ref_deref));
    assert_eq!(weak_ref_deref_descriptor.writable(), Some(true));
    assert_eq!(weak_ref_deref_descriptor.enumerable(), Some(false));
    assert_eq!(weak_ref_deref_descriptor.configurable(), Some(true));
}

#[test]
fn shared_bootstrap_installs_iterator_family_descriptors() {
    let mut runtime = lyng_js_env::Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let mut cache = BuiltinCache::new();

    let artifacts = bootstrap_default_realm(
        agent,
        &mut cache,
        BootstrapRequest::new(BootstrapMode::SpecOnly),
    )
    .expect("spec bootstrap should succeed");
    let intrinsics = agent
        .realm(artifacts.realm())
        .expect("default realm should exist")
        .intrinsics();
    let iterator_prototype = intrinsics
        .iterator_prototype()
        .expect("Iterator.prototype intrinsic should exist");
    let async_iterator_prototype = intrinsics
        .async_iterator_prototype()
        .expect("AsyncIterator.prototype intrinsic should exist");
    let map_iterator_prototype = intrinsics
        .map_iterator_prototype()
        .expect("Map Iterator prototype intrinsic should exist");
    let set_iterator_prototype = intrinsics
        .set_iterator_prototype()
        .expect("Set Iterator prototype intrinsic should exist");

    let next_atom = agent.atoms_mut().intern_collectible("next");
    let iterator_symbol = agent
        .well_known_symbol(WellKnownSymbolId::Iterator)
        .expect("Symbol.iterator should exist");
    let async_iterator_symbol = agent
        .well_known_symbol(WellKnownSymbolId::AsyncIterator)
        .expect("Symbol.asyncIterator should exist");
    let async_dispose_symbol = agent
        .well_known_symbol(WellKnownSymbolId::AsyncDispose)
        .expect("Symbol.asyncDispose should exist");
    let to_string_tag_symbol = agent
        .well_known_symbol(WellKnownSymbolId::ToStringTag)
        .expect("Symbol.toStringTag should exist");

    let iterator_method = cache
        .builtin_constant(
            agent,
            artifacts.realm(),
            iterator_prototype_iterator_builtin(),
        )
        .expect("Iterator.prototype[Symbol.iterator] builtin should resolve");
    let map_iterator_next = cache
        .builtin_constant(agent, artifacts.realm(), map_iterator_next_builtin())
        .expect("Map Iterator next builtin should resolve");
    let set_iterator_next = cache
        .builtin_constant(agent, artifacts.realm(), set_iterator_next_builtin())
        .expect("Set Iterator next builtin should resolve");
    let async_iterator_dispose = cache
        .builtin_constant(agent, artifacts.realm(), async_iterator_dispose_builtin())
        .expect("AsyncIterator.prototype[Symbol.asyncDispose] builtin should resolve");

    let iterator = own_descriptor(
        agent,
        iterator_prototype,
        PropertyKey::from_symbol(iterator_symbol),
        "Iterator.prototype[Symbol.iterator]",
    );
    assert_eq!(iterator.value(), Some(iterator_method));
    assert_eq!(iterator.writable(), Some(true));
    assert_eq!(iterator.enumerable(), Some(false));
    assert_eq!(iterator.configurable(), Some(true));

    let async_iterator = own_descriptor(
        agent,
        async_iterator_prototype,
        PropertyKey::from_symbol(async_iterator_symbol),
        "AsyncIterator.prototype[Symbol.asyncIterator]",
    );
    assert!(async_iterator
        .value()
        .and_then(Value::as_object_ref)
        .is_some());
    assert_eq!(async_iterator.writable(), Some(true));
    assert_eq!(async_iterator.enumerable(), Some(false));
    assert_eq!(async_iterator.configurable(), Some(true));

    let async_dispose = own_descriptor(
        agent,
        async_iterator_prototype,
        PropertyKey::from_symbol(async_dispose_symbol),
        "AsyncIterator.prototype[Symbol.asyncDispose]",
    );
    assert_eq!(async_dispose.value(), Some(async_iterator_dispose));
    assert_eq!(async_dispose.writable(), Some(true));
    assert_eq!(async_dispose.enumerable(), Some(false));
    assert_eq!(async_dispose.configurable(), Some(true));

    let async_iterator_tag = own_descriptor(
        agent,
        async_iterator_prototype,
        PropertyKey::from_symbol(to_string_tag_symbol),
        "AsyncIterator.prototype[Symbol.toStringTag]",
    );
    assert!(async_iterator_tag
        .value()
        .and_then(Value::as_string_ref)
        .is_some());
    assert_eq!(async_iterator_tag.writable(), Some(false));
    assert_eq!(async_iterator_tag.enumerable(), Some(false));
    assert_eq!(async_iterator_tag.configurable(), Some(true));

    let map_next = own_descriptor(
        agent,
        map_iterator_prototype,
        PropertyKey::from_atom(next_atom),
        "Map Iterator prototype.next",
    );
    assert_eq!(map_next.value(), Some(map_iterator_next));
    assert_eq!(map_next.writable(), Some(true));
    assert_eq!(map_next.enumerable(), Some(false));
    assert_eq!(map_next.configurable(), Some(true));

    let map_tag = own_descriptor(
        agent,
        map_iterator_prototype,
        PropertyKey::from_symbol(to_string_tag_symbol),
        "Map Iterator prototype[Symbol.toStringTag]",
    );
    assert!(map_tag.value().and_then(Value::as_string_ref).is_some());
    assert_eq!(map_tag.writable(), Some(false));
    assert_eq!(map_tag.enumerable(), Some(false));
    assert_eq!(map_tag.configurable(), Some(true));

    let set_next = own_descriptor(
        agent,
        set_iterator_prototype,
        PropertyKey::from_atom(next_atom),
        "Set Iterator prototype.next",
    );
    assert_eq!(set_next.value(), Some(set_iterator_next));
    assert_eq!(set_next.writable(), Some(true));
    assert_eq!(set_next.enumerable(), Some(false));
    assert_eq!(set_next.configurable(), Some(true));

    let set_tag = own_descriptor(
        agent,
        set_iterator_prototype,
        PropertyKey::from_symbol(to_string_tag_symbol),
        "Set Iterator prototype[Symbol.toStringTag]",
    );
    assert!(set_tag.value().and_then(Value::as_string_ref).is_some());
    assert_eq!(set_tag.writable(), Some(false));
    assert_eq!(set_tag.enumerable(), Some(false));
    assert_eq!(set_tag.configurable(), Some(true));
}

#[test]
fn shared_bootstrap_keeps_generator_iterator_method_inherited() {
    let mut runtime = lyng_js_env::Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let mut cache = BuiltinCache::new();

    let artifacts = bootstrap_default_realm(
        agent,
        &mut cache,
        BootstrapRequest::new(BootstrapMode::SpecOnly),
    )
    .expect("spec bootstrap should succeed");
    let intrinsics = agent
        .realm(artifacts.realm())
        .expect("default realm should exist")
        .intrinsics();
    let generator_prototype = intrinsics
        .generator_prototype()
        .expect("GeneratorPrototype intrinsic should exist");
    let iterator_symbol = agent
        .well_known_symbol(WellKnownSymbolId::Iterator)
        .expect("Symbol.iterator should exist");
    let to_string_tag_symbol = agent
        .well_known_symbol(WellKnownSymbolId::ToStringTag)
        .expect("Symbol.toStringTag should exist");

    let own_iterator = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            generator_prototype,
            PropertyKey::from_symbol(iterator_symbol),
        )
        .expect("GeneratorPrototype should be queryable");
    assert!(
        own_iterator.is_none(),
        "GeneratorPrototype should inherit Symbol.iterator from IteratorPrototype"
    );

    let tag = own_descriptor(
        agent,
        generator_prototype,
        PropertyKey::from_symbol(to_string_tag_symbol),
        "GeneratorPrototype[Symbol.toStringTag]",
    );
    assert!(tag.value().and_then(Value::as_string_ref).is_some());
    assert_eq!(tag.writable(), Some(false));
    assert_eq!(tag.enumerable(), Some(false));
    assert_eq!(tag.configurable(), Some(true));
}

#[test]
fn shared_bootstrap_installs_object_reflection_family_descriptors() {
    let mut runtime = lyng_js_env::Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let mut cache = BuiltinCache::new();

    let artifacts = bootstrap_default_realm(
        agent,
        &mut cache,
        BootstrapRequest::new(BootstrapMode::SpecOnly),
    )
    .expect("spec bootstrap should succeed");
    let intrinsics = agent
        .realm(artifacts.realm())
        .expect("default realm should exist")
        .intrinsics();
    let reflect = intrinsics
        .reflect()
        .expect("Reflect intrinsic should exist");
    let proxy = intrinsics.proxy().expect("Proxy intrinsic should exist");

    let get_atom = agent.atoms_mut().intern_collectible("get");
    let revocable_atom = agent.atoms_mut().intern_collectible("revocable");
    let to_string_tag_symbol = agent
        .well_known_symbol(WellKnownSymbolId::ToStringTag)
        .expect("Symbol.toStringTag should exist");

    let reflect_get = cache
        .builtin_constant(agent, artifacts.realm(), reflect_get_builtin())
        .expect("Reflect.get builtin should resolve");
    let proxy_revocable = cache
        .builtin_constant(agent, artifacts.realm(), proxy_revocable_builtin())
        .expect("Proxy.revocable builtin should resolve");

    let reflect_get_descriptor = own_descriptor(
        agent,
        reflect,
        PropertyKey::from_atom(get_atom),
        "Reflect.get",
    );
    assert_eq!(reflect_get_descriptor.value(), Some(reflect_get));
    assert_eq!(reflect_get_descriptor.writable(), Some(true));
    assert_eq!(reflect_get_descriptor.enumerable(), Some(false));
    assert_eq!(reflect_get_descriptor.configurable(), Some(true));

    let reflect_tag = own_descriptor(
        agent,
        reflect,
        PropertyKey::from_symbol(to_string_tag_symbol),
        "Reflect[Symbol.toStringTag]",
    );
    assert!(reflect_tag.value().and_then(Value::as_string_ref).is_some());
    assert_eq!(reflect_tag.writable(), Some(false));
    assert_eq!(reflect_tag.enumerable(), Some(false));
    assert_eq!(reflect_tag.configurable(), Some(true));

    let proxy_revocable_descriptor = own_descriptor(
        agent,
        proxy,
        PropertyKey::from_atom(revocable_atom),
        "Proxy.revocable",
    );
    assert_eq!(proxy_revocable_descriptor.value(), Some(proxy_revocable));
    assert_eq!(proxy_revocable_descriptor.writable(), Some(true));
    assert_eq!(proxy_revocable_descriptor.enumerable(), Some(false));
    assert_eq!(proxy_revocable_descriptor.configurable(), Some(true));
}

#[test]
fn shared_bootstrap_installs_json_family_descriptors() {
    let mut runtime = lyng_js_env::Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let mut cache = BuiltinCache::new();

    let artifacts = bootstrap_default_realm(
        agent,
        &mut cache,
        BootstrapRequest::new(BootstrapMode::SpecOnly),
    )
    .expect("spec bootstrap should succeed");
    let json = agent
        .realm(artifacts.realm())
        .expect("default realm should exist")
        .intrinsics()
        .json()
        .expect("JSON intrinsic should exist");

    let parse_atom = agent.atoms_mut().intern_collectible("parse");
    let raw_json_atom = agent.atoms_mut().intern_collectible("rawJSON");
    let to_string_tag_symbol = agent
        .well_known_symbol(WellKnownSymbolId::ToStringTag)
        .expect("Symbol.toStringTag should exist");

    let json_parse = cache
        .builtin_constant(agent, artifacts.realm(), json_parse_builtin())
        .expect("JSON.parse builtin should resolve");
    let json_raw_json = cache
        .builtin_constant(agent, artifacts.realm(), json_raw_json_builtin())
        .expect("JSON.rawJSON builtin should resolve");

    let parse = own_descriptor(
        agent,
        json,
        PropertyKey::from_atom(parse_atom),
        "JSON.parse",
    );
    assert_eq!(parse.value(), Some(json_parse));
    assert_eq!(parse.writable(), Some(true));
    assert_eq!(parse.enumerable(), Some(false));
    assert_eq!(parse.configurable(), Some(true));

    let raw_json = own_descriptor(
        agent,
        json,
        PropertyKey::from_atom(raw_json_atom),
        "JSON.rawJSON",
    );
    assert_eq!(raw_json.value(), Some(json_raw_json));
    assert_eq!(raw_json.writable(), Some(true));
    assert_eq!(raw_json.enumerable(), Some(false));
    assert_eq!(raw_json.configurable(), Some(true));

    let json_tag = own_descriptor(
        agent,
        json,
        PropertyKey::from_symbol(to_string_tag_symbol),
        "JSON[Symbol.toStringTag]",
    );
    assert!(json_tag.value().and_then(Value::as_string_ref).is_some());
    assert_eq!(json_tag.writable(), Some(false));
    assert_eq!(json_tag.enumerable(), Some(false));
    assert_eq!(json_tag.configurable(), Some(true));
}

#[test]
fn shared_bootstrap_installs_string_family_descriptors() {
    let mut runtime = lyng_js_env::Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let mut cache = BuiltinCache::new();

    let artifacts = bootstrap_default_realm(
        agent,
        &mut cache,
        BootstrapRequest::new(BootstrapMode::SpecOnly),
    )
    .expect("spec bootstrap should succeed");
    let intrinsics = agent
        .realm(artifacts.realm())
        .expect("default realm should exist")
        .intrinsics();
    let string = intrinsics.string().expect("String intrinsic should exist");
    let string_prototype = intrinsics
        .string_prototype()
        .expect("String.prototype intrinsic should exist");
    let string_iterator_prototype = intrinsics
        .string_iterator_prototype()
        .expect("String Iterator prototype intrinsic should exist");

    let constructor_atom = WellKnownAtom::constructor.id();
    let from_char_code_atom = agent.atoms_mut().intern_collectible("fromCharCode");
    let trim_atom = agent.atoms_mut().intern_collectible("trim");
    let next_atom = agent.atoms_mut().intern_collectible("next");
    let iterator_symbol = agent
        .well_known_symbol(WellKnownSymbolId::Iterator)
        .expect("Symbol.iterator should exist");
    let to_string_tag_symbol = agent
        .well_known_symbol(WellKnownSymbolId::ToStringTag)
        .expect("Symbol.toStringTag should exist");

    let from_char_code = cache
        .builtin_constant(agent, artifacts.realm(), string_from_char_code_builtin())
        .expect("String.fromCharCode builtin should resolve");
    let trim = cache
        .builtin_constant(agent, artifacts.realm(), string_trim_builtin())
        .expect("String.prototype.trim builtin should resolve");
    let iterator = cache
        .builtin_constant(agent, artifacts.realm(), string_iterator_builtin())
        .expect("String.prototype[Symbol.iterator] builtin should resolve");
    let iterator_next = cache
        .builtin_constant(agent, artifacts.realm(), string_iterator_next_builtin())
        .expect("String Iterator next builtin should resolve");

    let from_char_code_descriptor = own_descriptor(
        agent,
        string,
        PropertyKey::from_atom(from_char_code_atom),
        "String.fromCharCode",
    );
    assert_eq!(from_char_code_descriptor.value(), Some(from_char_code));
    assert_eq!(from_char_code_descriptor.writable(), Some(true));
    assert_eq!(from_char_code_descriptor.enumerable(), Some(false));
    assert_eq!(from_char_code_descriptor.configurable(), Some(true));

    let constructor = own_descriptor(
        agent,
        string_prototype,
        PropertyKey::from_atom(constructor_atom),
        "String.prototype.constructor",
    );
    assert_eq!(constructor.value(), Some(Value::from_object_ref(string)));
    assert_eq!(constructor.writable(), Some(true));
    assert_eq!(constructor.enumerable(), Some(false));
    assert_eq!(constructor.configurable(), Some(true));

    let trim_descriptor = own_descriptor(
        agent,
        string_prototype,
        PropertyKey::from_atom(trim_atom),
        "String.prototype.trim",
    );
    assert_eq!(trim_descriptor.value(), Some(trim));
    assert_eq!(trim_descriptor.writable(), Some(true));
    assert_eq!(trim_descriptor.enumerable(), Some(false));
    assert_eq!(trim_descriptor.configurable(), Some(true));

    let iterator_descriptor = own_descriptor(
        agent,
        string_prototype,
        PropertyKey::from_symbol(iterator_symbol),
        "String.prototype[Symbol.iterator]",
    );
    assert_eq!(iterator_descriptor.value(), Some(iterator));
    assert_eq!(iterator_descriptor.writable(), Some(true));
    assert_eq!(iterator_descriptor.enumerable(), Some(false));
    assert_eq!(iterator_descriptor.configurable(), Some(true));

    let next = own_descriptor(
        agent,
        string_iterator_prototype,
        PropertyKey::from_atom(next_atom),
        "String Iterator prototype.next",
    );
    assert_eq!(next.value(), Some(iterator_next));
    assert_eq!(next.writable(), Some(true));
    assert_eq!(next.enumerable(), Some(false));
    assert_eq!(next.configurable(), Some(true));

    let tag = own_descriptor(
        agent,
        string_iterator_prototype,
        PropertyKey::from_symbol(to_string_tag_symbol),
        "String Iterator prototype[Symbol.toStringTag]",
    );
    assert!(tag.value().and_then(Value::as_string_ref).is_some());
    assert_eq!(tag.writable(), Some(false));
    assert_eq!(tag.enumerable(), Some(false));
    assert_eq!(tag.configurable(), Some(true));
}

#[test]
fn shared_bootstrap_installs_regexp_family_descriptors() {
    let mut runtime = lyng_js_env::Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let mut cache = BuiltinCache::new();

    let artifacts = bootstrap_default_realm(
        agent,
        &mut cache,
        BootstrapRequest::new(BootstrapMode::SpecOnly),
    )
    .expect("spec bootstrap should succeed");
    let intrinsics = agent
        .realm(artifacts.realm())
        .expect("default realm should exist")
        .intrinsics();
    let regexp = intrinsics.regexp().expect("RegExp intrinsic should exist");
    let regexp_prototype = intrinsics
        .regexp_prototype()
        .expect("RegExp.prototype intrinsic should exist");

    let constructor_atom = WellKnownAtom::constructor.id();
    let escape_atom = agent.atoms_mut().intern_collectible("escape");
    let exec_atom = agent.atoms_mut().intern_collectible("exec");
    let global_atom = agent.atoms_mut().intern_collectible("global");
    let species_symbol = agent
        .well_known_symbol(WellKnownSymbolId::Species)
        .expect("Symbol.species should exist");
    let match_symbol = agent
        .well_known_symbol(WellKnownSymbolId::Match)
        .expect("Symbol.match should exist");

    let escape = cache
        .builtin_constant(agent, artifacts.realm(), regexp_escape_builtin())
        .expect("RegExp.escape builtin should resolve");
    let species_getter = cache
        .builtin_constant(agent, artifacts.realm(), regexp_species_getter_builtin())
        .expect("RegExp @@species getter should resolve");
    let exec = cache
        .builtin_constant(agent, artifacts.realm(), regexp_exec_builtin())
        .expect("RegExp.prototype.exec builtin should resolve");
    let symbol_match = cache
        .builtin_constant(agent, artifacts.realm(), regexp_symbol_match_builtin())
        .expect("RegExp.prototype[Symbol.match] builtin should resolve");
    let global_getter = cache
        .builtin_constant(agent, artifacts.realm(), regexp_global_getter_builtin())
        .expect("RegExp.prototype.global getter should resolve");

    let escape_descriptor = own_descriptor(
        agent,
        regexp,
        PropertyKey::from_atom(escape_atom),
        "RegExp.escape",
    );
    assert_eq!(escape_descriptor.value(), Some(escape));
    assert_eq!(escape_descriptor.writable(), Some(true));
    assert_eq!(escape_descriptor.enumerable(), Some(false));
    assert_eq!(escape_descriptor.configurable(), Some(true));

    let species = own_descriptor(
        agent,
        regexp,
        PropertyKey::from_symbol(species_symbol),
        "RegExp[Symbol.species]",
    );
    assert_eq!(species.getter(), Some(species_getter));
    assert_eq!(species.setter(), Some(Value::undefined()));
    assert_eq!(species.enumerable(), Some(false));
    assert_eq!(species.configurable(), Some(true));

    let constructor = own_descriptor(
        agent,
        regexp_prototype,
        PropertyKey::from_atom(constructor_atom),
        "RegExp.prototype.constructor",
    );
    assert_eq!(constructor.value(), Some(Value::from_object_ref(regexp)));
    assert_eq!(constructor.writable(), Some(true));
    assert_eq!(constructor.enumerable(), Some(false));
    assert_eq!(constructor.configurable(), Some(true));

    let exec_descriptor = own_descriptor(
        agent,
        regexp_prototype,
        PropertyKey::from_atom(exec_atom),
        "RegExp.prototype.exec",
    );
    assert_eq!(exec_descriptor.value(), Some(exec));
    assert_eq!(exec_descriptor.writable(), Some(true));
    assert_eq!(exec_descriptor.enumerable(), Some(false));
    assert_eq!(exec_descriptor.configurable(), Some(true));

    let match_descriptor = own_descriptor(
        agent,
        regexp_prototype,
        PropertyKey::from_symbol(match_symbol),
        "RegExp.prototype[Symbol.match]",
    );
    assert_eq!(match_descriptor.value(), Some(symbol_match));
    assert_eq!(match_descriptor.writable(), Some(true));
    assert_eq!(match_descriptor.enumerable(), Some(false));
    assert_eq!(match_descriptor.configurable(), Some(true));

    let global = own_descriptor(
        agent,
        regexp_prototype,
        PropertyKey::from_atom(global_atom),
        "RegExp.prototype.global",
    );
    assert_eq!(global.getter(), Some(global_getter));
    assert_eq!(global.setter(), Some(Value::undefined()));
    assert_eq!(global.enumerable(), Some(false));
    assert_eq!(global.configurable(), Some(true));
}

#[test]
fn shared_bootstrap_installs_date_family_descriptors() {
    let mut runtime = lyng_js_env::Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let mut cache = BuiltinCache::new();

    let artifacts = bootstrap_default_realm(
        agent,
        &mut cache,
        BootstrapRequest::new(BootstrapMode::SpecOnly),
    )
    .expect("spec bootstrap should succeed");
    let intrinsics = agent
        .realm(artifacts.realm())
        .expect("default realm should exist")
        .intrinsics();
    let date = intrinsics.date().expect("Date intrinsic should exist");
    let date_prototype = intrinsics
        .date_prototype()
        .expect("Date.prototype intrinsic should exist");

    let constructor_atom = WellKnownAtom::constructor.id();
    let now_atom = agent.atoms_mut().intern_collectible("now");
    let get_time_atom = agent.atoms_mut().intern_collectible("getTime");
    let set_full_year_atom = agent.atoms_mut().intern_collectible("setFullYear");
    let to_primitive_symbol = agent
        .well_known_symbol(WellKnownSymbolId::ToPrimitive)
        .expect("Symbol.toPrimitive should exist");

    let now = cache
        .builtin_constant(agent, artifacts.realm(), date_now_builtin())
        .expect("Date.now builtin should resolve");
    let to_string = cache
        .builtin_constant(agent, artifacts.realm(), date_to_string_builtin())
        .expect("Date.prototype.toString builtin should resolve");
    let get_time = cache
        .builtin_constant(agent, artifacts.realm(), date_get_time_builtin())
        .expect("Date.prototype.getTime builtin should resolve");
    let set_full_year = cache
        .builtin_constant(agent, artifacts.realm(), date_set_full_year_builtin())
        .expect("Date.prototype.setFullYear builtin should resolve");
    let to_primitive = cache
        .builtin_constant(agent, artifacts.realm(), date_to_primitive_builtin())
        .expect("Date.prototype[Symbol.toPrimitive] builtin should resolve");

    let now_descriptor = own_descriptor(agent, date, PropertyKey::from_atom(now_atom), "Date.now");
    assert_eq!(now_descriptor.value(), Some(now));
    assert_eq!(now_descriptor.writable(), Some(true));
    assert_eq!(now_descriptor.enumerable(), Some(false));
    assert_eq!(now_descriptor.configurable(), Some(true));

    let constructor = own_descriptor(
        agent,
        date_prototype,
        PropertyKey::from_atom(constructor_atom),
        "Date.prototype.constructor",
    );
    assert_eq!(constructor.value(), Some(Value::from_object_ref(date)));
    assert_eq!(constructor.writable(), Some(true));
    assert_eq!(constructor.enumerable(), Some(false));
    assert_eq!(constructor.configurable(), Some(true));

    let to_string_descriptor = own_descriptor(
        agent,
        date_prototype,
        PropertyKey::from_atom(WellKnownAtom::toString.id()),
        "Date.prototype.toString",
    );
    assert_eq!(to_string_descriptor.value(), Some(to_string));
    assert_eq!(to_string_descriptor.writable(), Some(true));
    assert_eq!(to_string_descriptor.enumerable(), Some(false));
    assert_eq!(to_string_descriptor.configurable(), Some(true));

    let get_time_descriptor = own_descriptor(
        agent,
        date_prototype,
        PropertyKey::from_atom(get_time_atom),
        "Date.prototype.getTime",
    );
    assert_eq!(get_time_descriptor.value(), Some(get_time));
    assert_eq!(get_time_descriptor.writable(), Some(true));
    assert_eq!(get_time_descriptor.enumerable(), Some(false));
    assert_eq!(get_time_descriptor.configurable(), Some(true));

    let set_full_year_descriptor = own_descriptor(
        agent,
        date_prototype,
        PropertyKey::from_atom(set_full_year_atom),
        "Date.prototype.setFullYear",
    );
    assert_eq!(set_full_year_descriptor.value(), Some(set_full_year));
    assert_eq!(set_full_year_descriptor.writable(), Some(true));
    assert_eq!(set_full_year_descriptor.enumerable(), Some(false));
    assert_eq!(set_full_year_descriptor.configurable(), Some(true));

    let to_primitive_descriptor = own_descriptor(
        agent,
        date_prototype,
        PropertyKey::from_symbol(to_primitive_symbol),
        "Date.prototype[Symbol.toPrimitive]",
    );
    assert_eq!(to_primitive_descriptor.value(), Some(to_primitive));
    assert_eq!(to_primitive_descriptor.writable(), Some(false));
    assert_eq!(to_primitive_descriptor.enumerable(), Some(false));
    assert_eq!(to_primitive_descriptor.configurable(), Some(true));
}

#[test]
fn shared_bootstrap_installs_primitive_family_descriptors() {
    let mut runtime = lyng_js_env::Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let mut cache = BuiltinCache::new();

    let artifacts = bootstrap_default_realm(
        agent,
        &mut cache,
        BootstrapRequest::new(BootstrapMode::SpecOnly),
    )
    .expect("spec bootstrap should succeed");
    let intrinsics = agent
        .realm(artifacts.realm())
        .expect("default realm should exist")
        .intrinsics();
    let number = intrinsics.number().expect("Number intrinsic should exist");
    let number_prototype = intrinsics
        .number_prototype()
        .expect("Number.prototype intrinsic should exist");
    let math = intrinsics.math().expect("Math intrinsic should exist");
    let bigint = intrinsics.bigint().expect("BigInt intrinsic should exist");
    let bigint_prototype = intrinsics
        .bigint_prototype()
        .expect("BigInt.prototype intrinsic should exist");
    let boolean_prototype = intrinsics
        .boolean_prototype()
        .expect("Boolean.prototype intrinsic should exist");
    let symbol = intrinsics.symbol().expect("Symbol intrinsic should exist");
    let symbol_prototype = intrinsics
        .symbol_prototype()
        .expect("Symbol.prototype intrinsic should exist");

    let is_finite_atom = agent.atoms_mut().intern_collectible("isFinite");
    let abs_atom = agent.atoms_mut().intern_collectible("abs");
    let as_int_n_atom = agent.atoms_mut().intern_collectible("asIntN");
    let description_atom = agent.atoms_mut().intern_collectible("description");
    let to_string_tag_symbol = agent
        .well_known_symbol(WellKnownSymbolId::ToStringTag)
        .expect("Symbol.toStringTag should exist");
    let to_primitive_symbol = agent
        .well_known_symbol(WellKnownSymbolId::ToPrimitive)
        .expect("Symbol.toPrimitive should exist");
    let has_instance_symbol = agent
        .well_known_symbol(WellKnownSymbolId::HasInstance)
        .expect("Symbol.hasInstance should exist");

    let number_is_finite = cache
        .builtin_constant(agent, artifacts.realm(), number_is_finite_builtin())
        .expect("Number.isFinite builtin should resolve");
    let number_to_string = cache
        .builtin_constant(agent, artifacts.realm(), number_to_string_builtin())
        .expect("Number.prototype.toString builtin should resolve");
    let math_abs = cache
        .builtin_constant(agent, artifacts.realm(), math_abs_builtin())
        .expect("Math.abs builtin should resolve");
    let bigint_as_int_n = cache
        .builtin_constant(agent, artifacts.realm(), bigint_as_int_n_builtin())
        .expect("BigInt.asIntN builtin should resolve");
    let bigint_to_string = cache
        .builtin_constant(agent, artifacts.realm(), bigint_to_string_builtin())
        .expect("BigInt.prototype.toString builtin should resolve");
    let boolean_to_string = cache
        .builtin_constant(agent, artifacts.realm(), boolean_to_string_builtin())
        .expect("Boolean.prototype.toString builtin should resolve");
    let symbol_for = cache
        .builtin_constant(agent, artifacts.realm(), symbol_for_builtin())
        .expect("Symbol.for builtin should resolve");
    let symbol_to_primitive = cache
        .builtin_constant(agent, artifacts.realm(), symbol_to_primitive_builtin())
        .expect("Symbol.prototype[Symbol.toPrimitive] builtin should resolve");
    let symbol_description_getter = cache
        .builtin_constant(
            agent,
            artifacts.realm(),
            symbol_description_getter_builtin(),
        )
        .expect("Symbol.prototype.description getter should resolve");

    let is_finite = own_descriptor(
        agent,
        number,
        PropertyKey::from_atom(is_finite_atom),
        "Number.isFinite",
    );
    assert_eq!(is_finite.value(), Some(number_is_finite));
    assert_eq!(is_finite.writable(), Some(true));
    assert_eq!(is_finite.enumerable(), Some(false));
    assert_eq!(is_finite.configurable(), Some(true));

    let number_nan = own_descriptor(
        agent,
        number,
        PropertyKey::from_atom(agent.bootstrap_atoms().nan()),
        "Number.NaN",
    );
    assert!(number_nan
        .value()
        .and_then(Value::as_f64)
        .is_some_and(f64::is_nan));
    assert_eq!(number_nan.writable(), Some(false));
    assert_eq!(number_nan.enumerable(), Some(false));
    assert_eq!(number_nan.configurable(), Some(false));

    let number_to_string_descriptor = own_descriptor(
        agent,
        number_prototype,
        PropertyKey::from_atom(WellKnownAtom::toString.id()),
        "Number.prototype.toString",
    );
    assert_eq!(number_to_string_descriptor.value(), Some(number_to_string));

    let math_abs_descriptor =
        own_descriptor(agent, math, PropertyKey::from_atom(abs_atom), "Math.abs");
    assert_eq!(math_abs_descriptor.value(), Some(math_abs));
    assert_eq!(math_abs_descriptor.writable(), Some(true));
    assert_eq!(math_abs_descriptor.enumerable(), Some(false));
    assert_eq!(math_abs_descriptor.configurable(), Some(true));

    let math_tag = own_descriptor(
        agent,
        math,
        PropertyKey::from_symbol(to_string_tag_symbol),
        "Math[Symbol.toStringTag]",
    );
    assert!(math_tag.value().and_then(Value::as_string_ref).is_some());
    assert_eq!(math_tag.writable(), Some(false));
    assert_eq!(math_tag.enumerable(), Some(false));
    assert_eq!(math_tag.configurable(), Some(true));

    let as_int_n = own_descriptor(
        agent,
        bigint,
        PropertyKey::from_atom(as_int_n_atom),
        "BigInt.asIntN",
    );
    assert_eq!(as_int_n.value(), Some(bigint_as_int_n));

    let bigint_to_string_descriptor = own_descriptor(
        agent,
        bigint_prototype,
        PropertyKey::from_atom(WellKnownAtom::toString.id()),
        "BigInt.prototype.toString",
    );
    assert_eq!(bigint_to_string_descriptor.value(), Some(bigint_to_string));

    let boolean_to_string_descriptor = own_descriptor(
        agent,
        boolean_prototype,
        PropertyKey::from_atom(WellKnownAtom::toString.id()),
        "Boolean.prototype.toString",
    );
    assert_eq!(
        boolean_to_string_descriptor.value(),
        Some(boolean_to_string)
    );

    let symbol_for_descriptor = own_descriptor(
        agent,
        symbol,
        PropertyKey::from_atom(WellKnownAtom::r#for.id()),
        "Symbol.for",
    );
    assert_eq!(symbol_for_descriptor.value(), Some(symbol_for));

    let has_instance_descriptor = own_descriptor(
        agent,
        symbol,
        PropertyKey::from_atom(agent.bootstrap_atoms().has_instance()),
        "Symbol.hasInstance",
    );
    assert_eq!(
        has_instance_descriptor.value(),
        Some(Value::from_symbol_ref(has_instance_symbol))
    );
    assert_eq!(has_instance_descriptor.writable(), Some(false));
    assert_eq!(has_instance_descriptor.enumerable(), Some(false));
    assert_eq!(has_instance_descriptor.configurable(), Some(false));

    let symbol_to_primitive_descriptor = own_descriptor(
        agent,
        symbol_prototype,
        PropertyKey::from_symbol(to_primitive_symbol),
        "Symbol.prototype[Symbol.toPrimitive]",
    );
    assert_eq!(
        symbol_to_primitive_descriptor.value(),
        Some(symbol_to_primitive)
    );
    assert_eq!(symbol_to_primitive_descriptor.writable(), Some(false));
    assert_eq!(symbol_to_primitive_descriptor.enumerable(), Some(false));
    assert_eq!(symbol_to_primitive_descriptor.configurable(), Some(true));

    let description = own_descriptor(
        agent,
        symbol_prototype,
        PropertyKey::from_atom(description_atom),
        "Symbol.prototype.description",
    );
    assert_eq!(description.getter(), Some(symbol_description_getter));
    assert_eq!(description.setter(), Some(Value::undefined()));
    assert_eq!(description.enumerable(), Some(false));
    assert_eq!(description.configurable(), Some(true));
}

#[test]
fn shared_bootstrap_installs_binary_data_family_descriptors() {
    let mut runtime = lyng_js_env::Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let mut cache = BuiltinCache::new();

    let artifacts = bootstrap_default_realm(
        agent,
        &mut cache,
        BootstrapRequest::new(BootstrapMode::SpecOnly),
    )
    .expect("spec bootstrap should succeed");
    let intrinsics = agent
        .realm(artifacts.realm())
        .expect("default realm should exist")
        .intrinsics();
    let array_buffer = intrinsics
        .array_buffer()
        .expect("ArrayBuffer intrinsic should exist");
    let array_buffer_prototype = intrinsics
        .array_buffer_prototype()
        .expect("ArrayBuffer.prototype intrinsic should exist");
    let atomics = intrinsics
        .atomics()
        .expect("Atomics intrinsic should exist");
    let data_view_prototype = intrinsics
        .data_view_prototype()
        .expect("DataView.prototype intrinsic should exist");
    let typed_array = intrinsics
        .typed_array()
        .expect("%TypedArray% intrinsic should exist");
    let typed_array_prototype = intrinsics
        .typed_array_prototype()
        .expect("%TypedArray%.prototype intrinsic should exist");
    let uint8_array = intrinsics
        .uint8_array()
        .expect("Uint8Array intrinsic should exist");
    let uint8_array_prototype = intrinsics
        .uint8_array_prototype()
        .expect("Uint8Array.prototype intrinsic should exist");

    let is_view_atom = agent.atoms_mut().intern_collectible("isView");
    let byte_length_atom = agent.atoms_mut().intern_collectible("byteLength");
    let slice_atom = agent.atoms_mut().intern_collectible("slice");
    let add_atom = agent.atoms_mut().intern_collectible("add");
    let buffer_atom = agent.atoms_mut().intern_collectible("buffer");
    let get_uint8_atom = agent.atoms_mut().intern_collectible("getUint8");
    let from_atom = agent.atoms_mut().intern_collectible("from");
    let bytes_per_element_atom = agent.atoms_mut().intern_collectible("BYTES_PER_ELEMENT");
    let values_atom = agent.atoms_mut().intern_collectible("values");
    let species_symbol = agent
        .well_known_symbol(WellKnownSymbolId::Species)
        .expect("Symbol.species should exist");
    let iterator_symbol = agent
        .well_known_symbol(WellKnownSymbolId::Iterator)
        .expect("Symbol.iterator should exist");
    let to_string_tag_symbol = agent
        .well_known_symbol(WellKnownSymbolId::ToStringTag)
        .expect("Symbol.toStringTag should exist");

    let array_buffer_is_view = cache
        .builtin_constant(agent, artifacts.realm(), array_buffer_is_view_builtin())
        .expect("ArrayBuffer.isView builtin should resolve");
    let array_buffer_species_getter = cache
        .builtin_constant(agent, artifacts.realm(), array_species_getter_builtin())
        .expect("ArrayBuffer[Symbol.species] getter should resolve");
    let array_buffer_byte_length_getter = cache
        .builtin_constant(
            agent,
            artifacts.realm(),
            array_buffer_byte_length_getter_builtin(),
        )
        .expect("ArrayBuffer.prototype.byteLength getter should resolve");
    let array_buffer_slice = cache
        .builtin_constant(agent, artifacts.realm(), array_buffer_slice_builtin())
        .expect("ArrayBuffer.prototype.slice builtin should resolve");
    let atomics_add = cache
        .builtin_constant(agent, artifacts.realm(), atomics_add_builtin())
        .expect("Atomics.add builtin should resolve");
    let data_view_buffer_getter = cache
        .builtin_constant(agent, artifacts.realm(), data_view_buffer_getter_builtin())
        .expect("DataView.prototype.buffer getter should resolve");
    let data_view_get_uint8 = cache
        .builtin_constant(agent, artifacts.realm(), data_view_get_uint8_builtin())
        .expect("DataView.prototype.getUint8 builtin should resolve");
    let typed_array_from = cache
        .builtin_constant(agent, artifacts.realm(), typed_array_from_builtin())
        .expect("%TypedArray%.from builtin should resolve");
    let typed_array_to_string_tag_getter = cache
        .builtin_constant(
            agent,
            artifacts.realm(),
            typed_array_to_string_tag_getter_builtin(),
        )
        .expect("%TypedArray%.prototype[Symbol.toStringTag] getter should resolve");
    let uint8_array_buffer_getter = cache
        .builtin_constant(
            agent,
            artifacts.realm(),
            uint8_array_buffer_getter_builtin(),
        )
        .expect("%TypedArray%.prototype.buffer getter should resolve");
    let uint8_array_values = cache
        .builtin_constant(agent, artifacts.realm(), uint8_array_values_builtin())
        .expect("%TypedArray%.prototype.values builtin should resolve");

    let is_view = own_descriptor(
        agent,
        array_buffer,
        PropertyKey::from_atom(is_view_atom),
        "ArrayBuffer.isView",
    );
    assert_eq!(is_view.value(), Some(array_buffer_is_view));
    assert_eq!(is_view.writable(), Some(true));
    assert_eq!(is_view.enumerable(), Some(false));
    assert_eq!(is_view.configurable(), Some(true));

    let array_buffer_species = own_descriptor(
        agent,
        array_buffer,
        PropertyKey::from_symbol(species_symbol),
        "ArrayBuffer[Symbol.species]",
    );
    assert_eq!(
        array_buffer_species.getter(),
        Some(array_buffer_species_getter)
    );
    assert_eq!(array_buffer_species.setter(), Some(Value::undefined()));
    assert_eq!(array_buffer_species.enumerable(), Some(false));
    assert_eq!(array_buffer_species.configurable(), Some(true));

    let array_buffer_constructor = own_descriptor(
        agent,
        array_buffer_prototype,
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        "ArrayBuffer.prototype.constructor",
    );
    assert_eq!(
        array_buffer_constructor.value(),
        Some(Value::from_object_ref(array_buffer))
    );

    let array_buffer_byte_length = own_descriptor(
        agent,
        array_buffer_prototype,
        PropertyKey::from_atom(byte_length_atom),
        "ArrayBuffer.prototype.byteLength",
    );
    assert_eq!(
        array_buffer_byte_length.getter(),
        Some(array_buffer_byte_length_getter)
    );
    assert_eq!(array_buffer_byte_length.setter(), Some(Value::undefined()));

    let array_buffer_slice_descriptor = own_descriptor(
        agent,
        array_buffer_prototype,
        PropertyKey::from_atom(slice_atom),
        "ArrayBuffer.prototype.slice",
    );
    assert_eq!(
        array_buffer_slice_descriptor.value(),
        Some(array_buffer_slice)
    );

    let array_buffer_tag = own_descriptor(
        agent,
        array_buffer_prototype,
        PropertyKey::from_symbol(to_string_tag_symbol),
        "ArrayBuffer.prototype[Symbol.toStringTag]",
    );
    assert!(array_buffer_tag
        .value()
        .and_then(Value::as_string_ref)
        .is_some());
    assert_eq!(array_buffer_tag.writable(), Some(false));
    assert_eq!(array_buffer_tag.enumerable(), Some(false));
    assert_eq!(array_buffer_tag.configurable(), Some(true));

    let atomics_add_descriptor = own_descriptor(
        agent,
        atomics,
        PropertyKey::from_atom(add_atom),
        "Atomics.add",
    );
    assert_eq!(atomics_add_descriptor.value(), Some(atomics_add));
    assert_eq!(atomics_add_descriptor.writable(), Some(true));
    assert_eq!(atomics_add_descriptor.enumerable(), Some(false));
    assert_eq!(atomics_add_descriptor.configurable(), Some(true));

    let atomics_tag = own_descriptor(
        agent,
        atomics,
        PropertyKey::from_symbol(to_string_tag_symbol),
        "Atomics[Symbol.toStringTag]",
    );
    assert!(atomics_tag.value().and_then(Value::as_string_ref).is_some());
    assert_eq!(atomics_tag.writable(), Some(false));

    let data_view_buffer = own_descriptor(
        agent,
        data_view_prototype,
        PropertyKey::from_atom(buffer_atom),
        "DataView.prototype.buffer",
    );
    assert_eq!(data_view_buffer.getter(), Some(data_view_buffer_getter));
    assert_eq!(data_view_buffer.setter(), Some(Value::undefined()));

    let data_view_get_uint8_descriptor = own_descriptor(
        agent,
        data_view_prototype,
        PropertyKey::from_atom(get_uint8_atom),
        "DataView.prototype.getUint8",
    );
    assert_eq!(
        data_view_get_uint8_descriptor.value(),
        Some(data_view_get_uint8)
    );

    let typed_array_from_descriptor = own_descriptor(
        agent,
        typed_array,
        PropertyKey::from_atom(from_atom),
        "%TypedArray%.from",
    );
    assert_eq!(typed_array_from_descriptor.value(), Some(typed_array_from));

    let typed_array_species = own_descriptor(
        agent,
        typed_array,
        PropertyKey::from_symbol(species_symbol),
        "%TypedArray%[Symbol.species]",
    );
    assert_eq!(
        typed_array_species.getter(),
        Some(array_buffer_species_getter)
    );
    assert_eq!(typed_array_species.setter(), Some(Value::undefined()));

    let typed_array_tag = own_descriptor(
        agent,
        typed_array_prototype,
        PropertyKey::from_symbol(to_string_tag_symbol),
        "%TypedArray%.prototype[Symbol.toStringTag]",
    );
    assert_eq!(
        typed_array_tag.getter(),
        Some(typed_array_to_string_tag_getter)
    );
    assert_eq!(typed_array_tag.setter(), Some(Value::undefined()));

    let typed_array_buffer = own_descriptor(
        agent,
        typed_array_prototype,
        PropertyKey::from_atom(buffer_atom),
        "%TypedArray%.prototype.buffer",
    );
    assert_eq!(typed_array_buffer.getter(), Some(uint8_array_buffer_getter));
    assert_eq!(typed_array_buffer.setter(), Some(Value::undefined()));

    let typed_array_values_descriptor = own_descriptor(
        agent,
        typed_array_prototype,
        PropertyKey::from_atom(values_atom),
        "%TypedArray%.prototype.values",
    );
    assert_eq!(
        typed_array_values_descriptor.value(),
        Some(uint8_array_values)
    );

    let typed_array_iterator = own_descriptor(
        agent,
        typed_array_prototype,
        PropertyKey::from_symbol(iterator_symbol),
        "%TypedArray%.prototype[Symbol.iterator]",
    );
    assert_eq!(typed_array_iterator.value(), Some(uint8_array_values));

    let bytes_per_element = own_descriptor(
        agent,
        uint8_array,
        PropertyKey::from_atom(bytes_per_element_atom),
        "Uint8Array.BYTES_PER_ELEMENT",
    );
    assert_eq!(bytes_per_element.value(), Some(Value::from_smi(1)));
    assert_eq!(bytes_per_element.writable(), Some(false));
    assert_eq!(bytes_per_element.enumerable(), Some(false));
    assert_eq!(bytes_per_element.configurable(), Some(false));

    let uint8_array_constructor = own_descriptor(
        agent,
        uint8_array_prototype,
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        "Uint8Array.prototype.constructor",
    );
    assert_eq!(
        uint8_array_constructor.value(),
        Some(Value::from_object_ref(uint8_array))
    );

    let uint8_array_prototype_bytes_per_element = own_descriptor(
        agent,
        uint8_array_prototype,
        PropertyKey::from_atom(bytes_per_element_atom),
        "Uint8Array.prototype.BYTES_PER_ELEMENT",
    );
    assert_eq!(
        uint8_array_prototype_bytes_per_element.value(),
        Some(Value::from_smi(1))
    );
    assert_eq!(
        uint8_array_prototype_bytes_per_element.writable(),
        Some(false)
    );
    assert_eq!(
        uint8_array_prototype_bytes_per_element.enumerable(),
        Some(false)
    );
    assert_eq!(
        uint8_array_prototype_bytes_per_element.configurable(),
        Some(false)
    );

    assert!(
        agent
            .objects()
            .get_own_property(
                agent.heap().view(),
                uint8_array_prototype,
                PropertyKey::from_atom(buffer_atom),
            )
            .unwrap()
            .is_none(),
        "Uint8Array.prototype.buffer should be inherited"
    );
    assert!(
        agent
            .objects()
            .get_own_property(
                agent.heap().view(),
                uint8_array_prototype,
                PropertyKey::from_atom(values_atom),
            )
            .unwrap()
            .is_none(),
        "Uint8Array.prototype.values should be inherited"
    );
    assert!(
        agent
            .objects()
            .get_own_property(
                agent.heap().view(),
                uint8_array_prototype,
                PropertyKey::from_symbol(iterator_symbol),
            )
            .unwrap()
            .is_none(),
        "Uint8Array.prototype[Symbol.iterator] should be inherited"
    );
}

#[test]
fn shared_bootstrap_installs_promise_disposal_family_descriptors() {
    let mut runtime = lyng_js_env::Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let mut cache = BuiltinCache::new();

    let artifacts = bootstrap_default_realm(
        agent,
        &mut cache,
        BootstrapRequest::new(BootstrapMode::SpecOnly),
    )
    .expect("spec bootstrap should succeed");
    let intrinsics = agent
        .realm(artifacts.realm())
        .expect("default realm should exist")
        .intrinsics();
    let promise = intrinsics
        .promise()
        .expect("Promise intrinsic should exist");
    let promise_prototype = intrinsics
        .promise_prototype()
        .expect("Promise.prototype intrinsic should exist");
    let disposable_stack = intrinsics
        .disposable_stack()
        .expect("DisposableStack intrinsic should exist");
    let disposable_stack_prototype = intrinsics
        .disposable_stack_prototype()
        .expect("DisposableStack.prototype intrinsic should exist");
    let async_disposable_stack = intrinsics
        .async_disposable_stack()
        .expect("AsyncDisposableStack intrinsic should exist");
    let async_disposable_stack_prototype = intrinsics
        .async_disposable_stack_prototype()
        .expect("AsyncDisposableStack.prototype intrinsic should exist");

    let resolve_atom = agent.atoms_mut().intern_collectible("resolve");
    let then_atom = agent.atoms_mut().intern_collectible("then");
    let use_atom = agent.atoms_mut().intern_collectible("use");
    let disposed_atom = agent.atoms_mut().intern_collectible("disposed");
    let dispose_async_atom = agent.atoms_mut().intern_collectible("disposeAsync");
    let species_symbol = agent
        .well_known_symbol(WellKnownSymbolId::Species)
        .expect("Symbol.species should exist");
    let dispose_symbol = agent
        .well_known_symbol(WellKnownSymbolId::Dispose)
        .expect("Symbol.dispose should exist");
    let async_dispose_symbol = agent
        .well_known_symbol(WellKnownSymbolId::AsyncDispose)
        .expect("Symbol.asyncDispose should exist");
    let to_string_tag_symbol = agent
        .well_known_symbol(WellKnownSymbolId::ToStringTag)
        .expect("Symbol.toStringTag should exist");

    let promise_resolve = cache
        .builtin_constant(agent, artifacts.realm(), promise_resolve_builtin())
        .expect("Promise.resolve builtin should resolve");
    let promise_species_getter = cache
        .builtin_constant(agent, artifacts.realm(), promise_species_getter_builtin())
        .expect("Promise[Symbol.species] getter should resolve");
    let promise_then = cache
        .builtin_constant(agent, artifacts.realm(), promise_then_builtin())
        .expect("Promise.prototype.then builtin should resolve");
    let disposable_use = cache
        .builtin_constant(agent, artifacts.realm(), disposable_stack_use_builtin())
        .expect("DisposableStack.prototype.use builtin should resolve");
    let disposable_disposed_getter = cache
        .builtin_constant(
            agent,
            artifacts.realm(),
            disposable_stack_disposed_getter_builtin(),
        )
        .expect("DisposableStack.prototype.disposed getter should resolve");
    let disposable_dispose = cache
        .builtin_constant(agent, artifacts.realm(), disposable_stack_dispose_builtin())
        .expect("DisposableStack.prototype.dispose builtin should resolve");
    let async_dispose = cache
        .builtin_constant(
            agent,
            artifacts.realm(),
            async_disposable_stack_dispose_async_builtin(),
        )
        .expect("AsyncDisposableStack.prototype.disposeAsync builtin should resolve");

    let resolve = own_descriptor(
        agent,
        promise,
        PropertyKey::from_atom(resolve_atom),
        "Promise.resolve",
    );
    assert_eq!(resolve.value(), Some(promise_resolve));
    assert_eq!(resolve.writable(), Some(true));
    assert_eq!(resolve.enumerable(), Some(false));
    assert_eq!(resolve.configurable(), Some(true));

    let species = own_descriptor(
        agent,
        promise,
        PropertyKey::from_symbol(species_symbol),
        "Promise[Symbol.species]",
    );
    assert_eq!(species.getter(), Some(promise_species_getter));
    assert_eq!(species.setter(), Some(Value::undefined()));
    assert_eq!(species.enumerable(), Some(false));
    assert_eq!(species.configurable(), Some(true));

    let then = own_descriptor(
        agent,
        promise_prototype,
        PropertyKey::from_atom(then_atom),
        "Promise.prototype.then",
    );
    assert_eq!(then.value(), Some(promise_then));

    let promise_tag = own_descriptor(
        agent,
        promise_prototype,
        PropertyKey::from_symbol(to_string_tag_symbol),
        "Promise.prototype[Symbol.toStringTag]",
    );
    assert!(promise_tag.value().and_then(Value::as_string_ref).is_some());
    assert_eq!(promise_tag.writable(), Some(false));
    assert_eq!(promise_tag.enumerable(), Some(false));
    assert_eq!(promise_tag.configurable(), Some(true));

    let stack_constructor = own_descriptor(
        agent,
        disposable_stack_prototype,
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        "DisposableStack.prototype.constructor",
    );
    assert_eq!(
        stack_constructor.value(),
        Some(Value::from_object_ref(disposable_stack))
    );

    let stack_use = own_descriptor(
        agent,
        disposable_stack_prototype,
        PropertyKey::from_atom(use_atom),
        "DisposableStack.prototype.use",
    );
    assert_eq!(stack_use.value(), Some(disposable_use));

    let stack_disposed = own_descriptor(
        agent,
        disposable_stack_prototype,
        PropertyKey::from_atom(disposed_atom),
        "DisposableStack.prototype.disposed",
    );
    assert_eq!(stack_disposed.getter(), Some(disposable_disposed_getter));
    assert_eq!(stack_disposed.setter(), Some(Value::undefined()));
    assert_eq!(stack_disposed.enumerable(), Some(false));
    assert_eq!(stack_disposed.configurable(), Some(true));

    let stack_dispose_symbol = own_descriptor(
        agent,
        disposable_stack_prototype,
        PropertyKey::from_symbol(dispose_symbol),
        "DisposableStack.prototype[Symbol.dispose]",
    );
    assert_eq!(stack_dispose_symbol.value(), Some(disposable_dispose));
    assert_eq!(stack_dispose_symbol.writable(), Some(true));
    assert_eq!(stack_dispose_symbol.enumerable(), Some(false));
    assert_eq!(stack_dispose_symbol.configurable(), Some(true));

    let async_stack_constructor = own_descriptor(
        agent,
        async_disposable_stack_prototype,
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        "AsyncDisposableStack.prototype.constructor",
    );
    assert_eq!(
        async_stack_constructor.value(),
        Some(Value::from_object_ref(async_disposable_stack))
    );

    let async_stack_dispose = own_descriptor(
        agent,
        async_disposable_stack_prototype,
        PropertyKey::from_atom(dispose_async_atom),
        "AsyncDisposableStack.prototype.disposeAsync",
    );
    assert_eq!(async_stack_dispose.value(), Some(async_dispose));

    let async_stack_dispose_symbol = own_descriptor(
        agent,
        async_disposable_stack_prototype,
        PropertyKey::from_symbol(async_dispose_symbol),
        "AsyncDisposableStack.prototype[Symbol.asyncDispose]",
    );
    assert_eq!(async_stack_dispose_symbol.value(), Some(async_dispose));
    assert_eq!(async_stack_dispose_symbol.writable(), Some(true));
    assert_eq!(async_stack_dispose_symbol.enumerable(), Some(false));
    assert_eq!(async_stack_dispose_symbol.configurable(), Some(true));
}

#[test]
fn shared_bootstrap_installs_error_family_descriptors() {
    let mut runtime = lyng_js_env::Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let mut cache = BuiltinCache::new();

    let artifacts = bootstrap_default_realm(
        agent,
        &mut cache,
        BootstrapRequest::new(BootstrapMode::SpecOnly),
    )
    .expect("spec bootstrap should succeed");
    let intrinsics = agent
        .realm(artifacts.realm())
        .expect("default realm should exist")
        .intrinsics();
    let error = intrinsics.error().expect("Error intrinsic should exist");
    let error_prototype = intrinsics
        .error_prototype()
        .expect("Error.prototype intrinsic should exist");
    let type_error = intrinsics
        .type_error()
        .expect("TypeError intrinsic should exist");
    let type_error_prototype = intrinsics
        .type_error_prototype()
        .expect("TypeError.prototype intrinsic should exist");

    let constructor_atom = WellKnownAtom::constructor.id();
    let name_atom = WellKnownAtom::name.id();
    let message_atom = agent.bootstrap_atoms().message();
    let to_string_atom = WellKnownAtom::toString.id();
    let error_to_string = cache
        .builtin_constant(agent, artifacts.realm(), error_to_string_builtin())
        .expect("Error.prototype.toString builtin should resolve");

    let error_constructor = own_descriptor(
        agent,
        error_prototype,
        PropertyKey::from_atom(constructor_atom),
        "Error.prototype.constructor",
    );
    assert_eq!(
        error_constructor.value(),
        Some(Value::from_object_ref(error))
    );
    assert_eq!(error_constructor.writable(), Some(true));
    assert_eq!(error_constructor.enumerable(), Some(false));
    assert_eq!(error_constructor.configurable(), Some(true));

    let error_to_string_descriptor = own_descriptor(
        agent,
        error_prototype,
        PropertyKey::from_atom(to_string_atom),
        "Error.prototype.toString",
    );
    assert_eq!(error_to_string_descriptor.value(), Some(error_to_string));
    assert_eq!(error_to_string_descriptor.writable(), Some(true));
    assert_eq!(error_to_string_descriptor.enumerable(), Some(false));
    assert_eq!(error_to_string_descriptor.configurable(), Some(true));

    let error_name = own_descriptor(
        agent,
        error_prototype,
        PropertyKey::from_atom(name_atom),
        "Error.prototype.name",
    );
    assert!(error_name.value().and_then(Value::as_string_ref).is_some());
    assert_eq!(error_name.writable(), Some(true));
    assert_eq!(error_name.enumerable(), Some(false));
    assert_eq!(error_name.configurable(), Some(true));

    let type_error_constructor = own_descriptor(
        agent,
        type_error_prototype,
        PropertyKey::from_atom(constructor_atom),
        "TypeError.prototype.constructor",
    );
    assert_eq!(
        type_error_constructor.value(),
        Some(Value::from_object_ref(type_error))
    );
    assert_eq!(type_error_constructor.writable(), Some(true));
    assert_eq!(type_error_constructor.enumerable(), Some(false));
    assert_eq!(type_error_constructor.configurable(), Some(true));

    let type_error_message = own_descriptor(
        agent,
        type_error_prototype,
        PropertyKey::from_atom(message_atom),
        "TypeError.prototype.message",
    );
    assert!(type_error_message
        .value()
        .and_then(Value::as_string_ref)
        .is_some());
    assert_eq!(type_error_message.writable(), Some(true));
    assert_eq!(type_error_message.enumerable(), Some(false));
    assert_eq!(type_error_message.configurable(), Some(true));
}

#[test]
fn shared_bootstrap_supports_selected_realm_shells() {
    let mut runtime = lyng_js_env::Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let default_realm = agent.default_realm().expect("default realm should exist");
    let extra_realm = agent.create_default_realm_shell(AllocationLifetime::Default);
    let mut cache = BuiltinCache::new();

    let artifacts = bootstrap_realm(
        agent,
        &mut cache,
        extra_realm,
        BootstrapRequest::new(BootstrapMode::SpecOnly),
    )
    .expect("selected realm bootstrap should succeed");
    let extra_record = agent
        .realm(extra_realm)
        .expect("extra realm should remain queryable");
    let atoms = agent.bootstrap_atoms();
    let global_this = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            extra_record.global_object(),
            PropertyKey::from_atom(atoms.global_this()),
        )
        .unwrap()
        .expect("globalThis should be installed on the extra realm");
    let object = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            extra_record.global_object(),
            PropertyKey::from_atom(atoms.object()),
        )
        .unwrap()
        .expect("Object should be installed on the extra realm");

    assert_eq!(artifacts.realm(), extra_realm);
    assert_ne!(artifacts.realm(), default_realm.id());
    assert_eq!(
        extra_record.bootstrap_state(),
        RealmBootstrapState::new().with_spec_ready(true)
    );
    assert_eq!(
        global_this.value(),
        Some(Value::from_object_ref(extra_record.global_object()))
    );
    assert_eq!(
        object.value(),
        extra_record
            .intrinsics()
            .object()
            .map(Value::from_object_ref)
    );
    assert_eq!(
        agent
            .realm(default_realm.id())
            .expect("default realm should remain queryable")
            .bootstrap_state(),
        RealmBootstrapState::new()
    );
}

#[test]
fn descriptor_installer_supports_accessor_rows() {
    let mut runtime = lyng_js_env::Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let mut cache = BuiltinCache::new();
    let artifacts = bootstrap_default_realm(
        agent,
        &mut cache,
        BootstrapRequest::new(BootstrapMode::SpecOnly),
    )
    .expect("spec bootstrap should succeed");
    let accessor_name = agent.atoms_mut().intern_collectible("bootstrapAccessor");
    let descriptors = [BuiltinPropertyDescriptor::new(
        BuiltinPropertyKeySpec::from_atom(accessor_name),
        BuiltinPropertyValueSpec::Accessor {
            get: Some(symbol_to_primitive_builtin()),
            set: Some(error_to_string_builtin()),
        },
        BuiltinAttributes::new(false, true, true),
    )];
    let tables = [BuiltinDescriptorTable::new(
        BuiltinInstallTarget::GlobalObject,
        &descriptors,
    )];

    install_descriptor_tables(agent, &mut cache, artifacts.realm(), &tables)
        .expect("accessor descriptor installation should succeed");

    let property = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            artifacts.global_object(),
            PropertyKey::from_atom(accessor_name),
        )
        .unwrap()
        .expect("accessor descriptor should be installed");
    let getter = cache
        .builtin_constant(agent, artifacts.realm(), symbol_to_primitive_builtin())
        .expect("getter builtin constant should resolve");
    let setter = cache
        .builtin_constant(agent, artifacts.realm(), error_to_string_builtin())
        .expect("setter builtin constant should resolve");

    assert_eq!(property.value(), None);
    assert_eq!(property.getter(), Some(getter));
    assert_eq!(property.setter(), Some(setter));
    assert_eq!(property.writable(), None);
    assert_eq!(property.enumerable(), Some(true));
    assert_eq!(property.configurable(), Some(true));
}
