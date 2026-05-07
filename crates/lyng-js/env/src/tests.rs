#![allow(clippy::too_many_lines)]

use super::*;
use lyng_js_common::{AtomId, SourceId};
use lyng_js_gc::{AllocationLifetime, PrimitiveHeapMarker};
use lyng_js_host::{
    AgentSpawnKind, AgentThreadStartKind, HostCall, HostSharedBufferId, HostThreadId,
    ImportMetaProperties, ImportMetaProperty, ImportMetaValue, ModuleKey, NoopHostHooks, TestHost,
};
use lyng_js_objects::{ObjectAllocation, ObjectFlags, ObjectKind, ObjectSubstrateMarker};
use lyng_js_types::{
    BackingStoreRef, CodeRef, ObjectRef, PropertyKey, StringRef, TypeOwnershipMarker, Value,
    WellKnownSymbolId,
};
use std::mem::size_of;

#[test]
fn environment_layout_id_round_trips_and_stays_compact() {
    let id = EnvironmentLayoutId::from_raw(17).expect("non-zero layout id");

    assert_eq!(id.get(), 17);
    assert_eq!(id.raw().get(), 17);
    assert_eq!(size_of::<EnvironmentLayoutId>(), size_of::<u32>());
    assert_eq!(size_of::<Option<EnvironmentLayoutId>>(), size_of::<u32>());
    assert_eq!(EnvironmentLayoutId::from_raw(0), None);
}

#[test]
fn environment_layout_preserves_binding_order_flags_and_optional_atoms() {
    let layout = EnvironmentLayout::new(
        EnvironmentLayoutKind::Declarative,
        [
            EnvironmentBindingLayout::new(
                Some(AtomId::from_raw(71)),
                EnvironmentSlotFlags::mutable_lexical(),
            ),
            EnvironmentBindingLayout::new(
                Some(AtomId::from_raw(72)),
                EnvironmentSlotFlags::var_like().with_dynamic(true),
            ),
            EnvironmentBindingLayout::new(None, EnvironmentSlotFlags::immutable_lexical()),
        ],
        true,
    );

    assert_eq!(layout.kind(), EnvironmentLayoutKind::Declarative);
    assert_eq!(layout.slot_count(), 3);
    assert!(layout.needs_environment());
    assert_eq!(
        layout.binding(0).unwrap().name(),
        Some(AtomId::from_raw(71))
    );
    assert!(layout.binding(0).unwrap().flags().is_mutable());
    assert!(layout.binding(0).unwrap().flags().is_lexical());
    assert!(layout.binding(0).unwrap().flags().needs_tdz());
    assert_eq!(
        layout.binding(1).unwrap().name(),
        Some(AtomId::from_raw(72))
    );
    assert!(!layout.binding(1).unwrap().flags().is_lexical());
    assert!(layout.binding(1).unwrap().flags().is_dynamic());
    assert_eq!(layout.binding(2).unwrap().name(), None);
    assert!(!layout.binding(2).unwrap().flags().is_mutable());
    assert_eq!(layout.binding(3), None);
}

#[test]
fn declarative_environment_record_stays_within_phase3_size_budget() {
    assert!(size_of::<DeclarativeEnvironmentRecord>() <= 32);
}

#[test]
fn runtime_boots_root_cluster_and_thread_affine_root_agent() {
    let runtime = Runtime::new(NoopHostHooks);
    let default_realm = runtime
        .root_agent()
        .default_realm()
        .expect("root agent should bootstrap one default realm");

    assert_eq!(runtime.root_cluster().agent_count(), 1);
    assert_eq!(runtime.root_agent_id().get(), 1);
    assert_eq!(runtime.root_agent().id(), runtime.root_agent_id());
    assert_eq!(runtime.root_agent().bound_thread(), None);
    assert_eq!(
        runtime.root_agent().queued_job_count(JobQueueKind::Script),
        0
    );
    assert!(runtime.root_agent().execution_contexts().is_empty());
    assert_eq!(runtime.root_agent().realm_refs(), &[default_realm.id()]);
    assert_eq!(
        runtime.root_agent().default_realm_id(),
        Some(default_realm.id())
    );
    assert!(default_realm.is_default());
    assert!(default_realm.global_object().get() > 0);
    assert!(default_realm.global_env().get() > 0);
    assert!(default_realm.root_shape().is_some());
    assert!(runtime.root_agent().heap().view().collection_budget_bytes() > 0);
}

#[test]
fn regexp_legacy_static_state_records_matches_as_lazy_source_ranges() {
    let source = StringRef::from_raw(17).expect("non-zero string ref");
    let mut state = RegExpLegacyStaticState::default();

    state.record_match(
        source,
        10,
        2..7,
        &[Some(3..5), None, Some(6..7), Some(4..6)],
    );

    assert_eq!(
        state.input(),
        &RegExpLegacyStaticText::SourceSlice {
            source,
            range: 0..10,
        }
    );
    assert_eq!(
        state.last_match(),
        &RegExpLegacyStaticText::SourceSlice {
            source,
            range: 2..7,
        }
    );
    assert_eq!(
        state.left_context(),
        &RegExpLegacyStaticText::SourceSlice {
            source,
            range: 0..2,
        }
    );
    assert_eq!(
        state.right_context(),
        &RegExpLegacyStaticText::SourceSlice {
            source,
            range: 7..10,
        }
    );
    assert_eq!(
        state.last_paren(),
        &RegExpLegacyStaticText::SourceSlice {
            source,
            range: 4..6,
        }
    );
    assert_eq!(
        state.paren(1),
        Some(&RegExpLegacyStaticText::SourceSlice {
            source,
            range: 3..5,
        })
    );
    assert_eq!(state.paren(2), Some(&RegExpLegacyStaticText::Empty));
    assert_eq!(
        state.paren(3),
        Some(&RegExpLegacyStaticText::SourceSlice {
            source,
            range: 6..7,
        })
    );

    state.set_input(vec![
        u16::from(b's'),
        u16::from(b'e'),
        u16::from(b'e'),
        u16::from(b'd'),
    ]);

    assert_eq!(
        state.input(),
        &RegExpLegacyStaticText::Owned(vec![
            u16::from(b's'),
            u16::from(b'e'),
            u16::from(b'e'),
            u16::from(b'd')
        ])
    );
    assert_eq!(
        state.last_match(),
        &RegExpLegacyStaticText::SourceSlice {
            source,
            range: 2..7,
        }
    );
}

#[test]
fn agent_owns_heap_atoms_objects_realms_contexts_and_jobs() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let atom = agent.atoms_mut().intern_collectible("runtime-name");
    let default_realm = agent
        .default_realm()
        .expect("root agent should expose a default realm");

    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let shape = objects.root_shape(&mut mutator, None, AllocationLifetime::Default);
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(shape),
            AllocationLifetime::Default,
        )
    });
    agent.push_script_context(
        default_realm.id(),
        default_realm.global_env(),
        default_realm.global_env(),
    );
    let job = agent.enqueue_job(
        HostJobKind::Script,
        ExecutableId::Script,
        Some(default_realm.id()),
        Some("entry".into()),
    );

    assert_eq!(atom, AtomId::from_raw(atom.raw()));
    assert!(object.get() > default_realm.global_object().get());
    assert_eq!(agent.realm_refs(), &[default_realm.id()]);
    assert_eq!(
        agent.current_execution_context(),
        Some(ExecutionContext::script(
            default_realm.id(),
            default_realm.global_env(),
            default_realm.global_env(),
        ))
    );
    assert_eq!(job.queue_kind(), JobQueueKind::Script);
    assert_eq!(agent.total_queued_jobs(), 1);
    assert_eq!(agent.dequeue_job(JobQueueKind::Script).unwrap(), job);
    assert_eq!(agent.total_queued_jobs(), 0);
}

#[test]
fn default_realm_shell_allocates_placeholders_and_typed_intrinsics() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let default_realm = agent
        .default_realm()
        .expect("default realm should exist after boot");

    assert_eq!(agent.realm(default_realm.id()), Some(default_realm));
    assert_eq!(default_realm.intrinsics(), Intrinsics::new());
    assert_eq!(default_realm.bootstrap_state(), RealmBootstrapState::new());
    assert_eq!(
        agent.realm_bootstrap_state(default_realm.id()),
        Some(RealmBootstrapState::new())
    );
    assert!(agent.heap().view().realm(default_realm.id()).is_some());
    assert!(agent
        .objects()
        .object(agent.heap().view(), default_realm.global_object())
        .is_some());
    assert!(agent
        .heap()
        .view()
        .environment(default_realm.global_env())
        .is_some());

    let global_this = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            default_realm.global_object(),
            PropertyKey::from_atom(agent.bootstrap_atoms().global_this()),
        )
        .unwrap()
        .is_none();
    let undefined = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            default_realm.global_object(),
            PropertyKey::from_atom(agent.bootstrap_atoms().undefined()),
        )
        .unwrap()
        .is_none();
    let nan = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            default_realm.global_object(),
            PropertyKey::from_atom(agent.bootstrap_atoms().nan()),
        )
        .unwrap()
        .is_none();
    let infinity = agent
        .objects()
        .get_own_property(
            agent.heap().view(),
            default_realm.global_object(),
            PropertyKey::from_atom(agent.bootstrap_atoms().infinity()),
        )
        .unwrap()
        .is_none();
    assert!(global_this);
    assert!(undefined);
    assert!(nan);
    assert!(infinity);

    let root_shape = default_realm
        .root_shape()
        .expect("default realm should carry a root shape");
    let throw_type_error = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        )
    });
    let updated_intrinsics = Intrinsics::new()
        .with_object_prototype(Some(default_realm.global_object()))
        .with_throw_type_error(Some(throw_type_error));

    assert!(agent.set_realm_intrinsics(default_realm.id(), updated_intrinsics));
    assert_eq!(
        agent
            .realm(default_realm.id())
            .expect("realm should still exist")
            .intrinsics(),
        updated_intrinsics
    );
    assert!(agent.mark_realm_spec_bootstrapped(default_realm.id()));
    assert_eq!(
        agent
            .realm(default_realm.id())
            .expect("realm should still exist")
            .bootstrap_state(),
        RealmBootstrapState::new().with_spec_ready(true)
    );
    assert!(agent.mark_realm_embedding_bootstrapped(default_realm.id()));
    assert_eq!(
        agent
            .realm(default_realm.id())
            .expect("realm should still exist")
            .bootstrap_state(),
        RealmBootstrapState::new()
            .with_spec_ready(true)
            .with_embedding_ready(true)
    );

    let extra_realm = agent.create_default_realm_shell(AllocationLifetime::Default);
    let extra_record = agent
        .realm(extra_realm)
        .expect("additional realm shell should be queryable");
    assert!(!extra_record.is_default());
    assert_eq!(extra_record.bootstrap_state(), RealmBootstrapState::new());
    assert_eq!(agent.realm_refs(), &[default_realm.id(), extra_realm]);
    let global_env = agent
        .global_environment(default_realm.global_env())
        .expect("default realm should use a global environment family");
    assert_eq!(global_env.global_object(), default_realm.global_object());
    assert!(global_env.lexical_names().is_empty());
    assert!(global_env.var_names().is_empty());
    assert_eq!(
        agent
            .environment_layout(global_env.layout())
            .expect("default global layout should exist")
            .slot_count(),
        0
    );
}

#[test]
fn intrinsics_expose_phase5_constructor_and_error_slots() {
    let object = ObjectRef::from_raw(1).unwrap();
    let function = ObjectRef::from_raw(2).unwrap();
    let number = ObjectRef::from_raw(3).unwrap();
    let math = ObjectRef::from_raw(4).unwrap();
    let bigint = ObjectRef::from_raw(5).unwrap();
    let boolean = ObjectRef::from_raw(6).unwrap();
    let symbol = ObjectRef::from_raw(7).unwrap();
    let error = ObjectRef::from_raw(8).unwrap();
    let type_error = ObjectRef::from_raw(9).unwrap();
    let uri_error_prototype = ObjectRef::from_raw(10).unwrap();
    let bigint_prototype = ObjectRef::from_raw(11).unwrap();

    let intrinsics = Intrinsics::new()
        .with_object(Some(object))
        .with_function(Some(function))
        .with_number(Some(number))
        .with_math(Some(math))
        .with_bigint(Some(bigint))
        .with_bigint_prototype(Some(bigint_prototype))
        .with_boolean(Some(boolean))
        .with_symbol(Some(symbol))
        .with_error(Some(error))
        .with_type_error(Some(type_error))
        .with_uri_error_prototype(Some(uri_error_prototype));

    assert_eq!(intrinsics.object(), Some(object));
    assert_eq!(intrinsics.function(), Some(function));
    assert_eq!(intrinsics.number(), Some(number));
    assert_eq!(intrinsics.math(), Some(math));
    assert_eq!(intrinsics.bigint(), Some(bigint));
    assert_eq!(intrinsics.bigint_prototype(), Some(bigint_prototype));
    assert_eq!(intrinsics.boolean(), Some(boolean));
    assert_eq!(intrinsics.symbol(), Some(symbol));
    assert_eq!(intrinsics.error(), Some(error));
    assert_eq!(intrinsics.type_error(), Some(type_error));
    assert_eq!(intrinsics.uri_error_prototype(), Some(uri_error_prototype));
}

#[test]
fn agent_bootstraps_phase5_symbol_state_and_global_symbol_registry() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let bootstrap_atoms = agent.bootstrap_atoms();
    let well_known_symbols = agent.well_known_symbols();

    assert_eq!(
        agent.atoms().resolve(bootstrap_atoms.global_this()),
        "globalThis"
    );
    assert_eq!(
        agent.well_known_symbol(WellKnownSymbolId::HasInstance),
        well_known_symbols.has_instance()
    );
    assert_eq!(
        agent.well_known_symbol(WellKnownSymbolId::IsConcatSpreadable),
        well_known_symbols.is_concat_spreadable()
    );
    assert_eq!(
        agent.well_known_symbol(WellKnownSymbolId::Iterator),
        well_known_symbols.iterator()
    );
    assert_eq!(
        agent.well_known_symbol(WellKnownSymbolId::AsyncIterator),
        well_known_symbols.async_iterator()
    );
    assert_eq!(
        agent.well_known_symbol(WellKnownSymbolId::Species),
        well_known_symbols.species()
    );
    assert_eq!(
        agent.well_known_symbol(WellKnownSymbolId::ToPrimitive),
        well_known_symbols.to_primitive()
    );
    assert_eq!(
        agent.well_known_symbol(WellKnownSymbolId::ToStringTag),
        well_known_symbols.to_string_tag()
    );
    assert_eq!(
        agent.well_known_symbol(WellKnownSymbolId::Dispose),
        well_known_symbols.dispose()
    );
    assert_eq!(
        agent.well_known_symbol(WellKnownSymbolId::AsyncDispose),
        well_known_symbols.async_dispose()
    );

    for id in WellKnownSymbolId::ALL {
        let symbol = agent
            .well_known_symbol(id)
            .expect("Phase 5 well-known symbols should allocate during agent bootstrap");
        let symbol_view = agent
            .heap()
            .view()
            .symbol_view(symbol)
            .expect("well-known symbol should be live");
        let description = symbol_view
            .description()
            .expect("well-known symbol should expose a description");
        assert!(symbol_view.is_well_known());
        assert_eq!(
            agent
                .heap()
                .view()
                .string(description)
                .unwrap()
                .cached_atom(),
            Some(bootstrap_atoms.well_known_symbol_description(id))
        );
    }

    let registry_key = agent.atoms_mut().intern_collectible("phase5.registry");
    let first = agent.global_symbol_for(registry_key, AllocationLifetime::Default);
    let second = agent.global_symbol_for(registry_key, AllocationLifetime::Default);
    let description = agent
        .heap()
        .view()
        .symbol_view(first)
        .unwrap()
        .description()
        .unwrap();

    assert_eq!(first, second);
    assert_eq!(agent.global_symbol(registry_key), Some(first));
    assert_eq!(agent.global_symbol_key_for(first), Some(registry_key));
    assert_eq!(agent.global_symbol_registry().len(), 1);
    assert!(agent
        .heap()
        .view()
        .symbol_view(first)
        .unwrap()
        .is_ordinary());
    assert_eq!(
        agent
            .heap()
            .view()
            .string(description)
            .unwrap()
            .cached_atom(),
        Some(registry_key)
    );
}

#[test]
fn agent_allocates_declarative_and_function_environments_with_dense_slots_and_this_state() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let decl_layout = agent.alloc_environment_layout(EnvironmentLayout::new(
        EnvironmentLayoutKind::Declarative,
        [
            EnvironmentBindingLayout::new(
                Some(AtomId::from_raw(81)),
                EnvironmentSlotFlags::mutable_lexical(),
            ),
            EnvironmentBindingLayout::new(
                Some(AtomId::from_raw(82)),
                EnvironmentSlotFlags::var_like(),
            ),
            EnvironmentBindingLayout::new(
                Some(AtomId::from_raw(83)),
                EnvironmentSlotFlags::new(false, false, false, false),
            ),
        ],
        true,
    ));
    let decl_env = agent
        .alloc_declarative_environment(None, decl_layout, AllocationLifetime::Default)
        .expect("declarative environment should allocate");

    assert_eq!(
        agent.declarative_environment(decl_env).unwrap().layout(),
        decl_layout
    );
    assert_eq!(
        agent.environment_slot(decl_env, 0),
        Some(Value::uninitialized_lexical())
    );
    assert_eq!(
        agent.environment_slot(decl_env, 1),
        Some(Value::undefined())
    );
    assert_eq!(
        agent.environment_slot(decl_env, 2),
        Some(Value::uninitialized_lexical())
    );
    assert!(agent.set_environment_slot(decl_env, 1, Value::from_smi(7)));
    assert_eq!(
        agent.environment_slot(decl_env, 1),
        Some(Value::from_smi(7))
    );

    let realm = agent
        .default_realm()
        .expect("default realm should be available");
    let root_shape = realm
        .root_shape()
        .expect("default realm should carry a root shape");
    let (function_object, new_target, home_object) =
        agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let function_object = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape),
                AllocationLifetime::Default,
            );
            let new_target = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape),
                AllocationLifetime::Default,
            );
            let home_object = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape),
                AllocationLifetime::Default,
            );
            (function_object, new_target, home_object)
        });
    let function_layout = agent.alloc_environment_layout(EnvironmentLayout::new(
        EnvironmentLayoutKind::Function,
        [EnvironmentBindingLayout::new(
            Some(AtomId::from_raw(83)),
            EnvironmentSlotFlags::mutable_lexical(),
        )],
        true,
    ));
    let function_env = agent
        .alloc_function_environment(
            Some(decl_env),
            function_layout,
            function_object,
            ThisBindingStatus::Uninitialized,
            Value::undefined(),
            Some(new_target),
            None,
            AllocationLifetime::Default,
        )
        .expect("function environment should allocate");

    let initial = agent
        .function_environment(function_env)
        .expect("function environment should be queryable");
    assert_eq!(initial.declarative().outer(), Some(decl_env));
    assert_eq!(initial.function_object(), function_object);
    assert_eq!(
        initial.this_binding_status(),
        ThisBindingStatus::Uninitialized
    );
    assert_eq!(initial.this_value(), Value::undefined());
    assert_eq!(initial.new_target(), Some(new_target));
    assert_eq!(
        agent.environment_slot(function_env, 0),
        Some(Value::uninitialized_lexical())
    );

    assert!(agent.set_function_this_binding(
        function_env,
        ThisBindingStatus::Initialized,
        Value::from_smi(42),
    ));
    assert!(agent.set_function_home_object(function_env, Some(home_object)));
    assert!(agent.set_function_new_target(function_env, None));

    let updated = agent
        .function_environment(function_env)
        .expect("updated function environment should be queryable");
    assert_eq!(
        updated.this_binding_status(),
        ThisBindingStatus::Initialized
    );
    assert_eq!(updated.this_value(), Value::from_smi(42));
    assert_eq!(updated.new_target(), None);
    assert_eq!(updated.home_object(), Some(home_object));
}

#[test]
fn private_environment_family_tracks_distinct_layout_and_slots() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let outer = agent
        .default_realm()
        .expect("default realm should exist after boot")
        .global_env();
    let layout = agent.alloc_environment_layout(EnvironmentLayout::new(
        EnvironmentLayoutKind::Private,
        [EnvironmentBindingLayout::new(
            Some(AtomId::from_raw(95)),
            EnvironmentSlotFlags::mutable_lexical(),
        )],
        true,
    ));
    let private_env = agent
        .alloc_private_environment(Some(outer), layout, AllocationLifetime::Default)
        .expect("private environment should allocate");

    let record = agent
        .private_environment(private_env)
        .expect("private environment should be queryable");
    assert_eq!(record.outer(), Some(outer));
    assert_eq!(record.layout(), layout);
    assert_eq!(
        agent.environment_slot(private_env, 0),
        Some(Value::uninitialized_lexical())
    );
    assert!(matches!(
        agent.environment(private_env),
        Some(EnvironmentRecord::Private(_))
    ));
}

#[test]
fn module_environment_family_and_module_record_cache_track_agent_owned_module_state() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let outer = agent
        .default_realm()
        .expect("default realm should exist after boot")
        .global_env();
    let layout = agent.alloc_environment_layout(EnvironmentLayout::new(
        EnvironmentLayoutKind::Module,
        [EnvironmentBindingLayout::new(
            Some(AtomId::from_raw(96)),
            EnvironmentSlotFlags::immutable_lexical(),
        )],
        true,
    ));
    let module_env = agent
        .alloc_module_environment(Some(outer), layout, AllocationLifetime::Default)
        .expect("module environment should allocate");

    let record = agent
        .module_environment(module_env)
        .expect("module environment should be queryable");
    assert_eq!(record.outer(), Some(outer));
    assert_eq!(record.layout(), layout);
    assert_eq!(
        agent.environment_slot(module_env, 0),
        Some(Value::uninitialized_lexical())
    );
    assert!(matches!(
        agent.environment(module_env),
        Some(EnvironmentRecord::Module(_))
    ));
    assert_eq!(agent.module_binding_alias(module_env, 0), None);

    let key = ModuleKey::new("/tmp/main.mjs");
    let dep = ModuleKey::new("/tmp/dep.mjs");
    let module = ModuleRecord::new(
        key.clone(),
        "/tmp/main.mjs",
        vec![ModuleRequestRecord::new(
            "./dep.mjs",
            Vec::new(),
            ModuleRequestPhase::Evaluation,
        )],
        vec![ModuleImportEntry::new(
            0,
            AtomId::from_raw(201),
            0,
            ModuleImportKind::Named(AtomId::from_raw(202)),
        )],
        vec![ModuleLocalExportEntry::new(
            AtomId::from_raw(203),
            Some(AtomId::from_raw(204)),
            0,
        )],
        vec![ModuleIndirectExportEntry::new(
            AtomId::from_raw(205),
            0,
            ModuleImportKind::NamespaceObject,
        )],
        vec![ModuleStarExportEntry::new(0)],
    );
    assert!(agent.install_module_record(module).is_none());
    assert!(agent.set_module_requested_key(&key, 0, Some(dep.clone())));
    assert!(agent.set_module_record_code(&key, CodeRef::from_raw(7)));
    assert!(agent.set_module_record_environment(&key, Some(module_env)));
    assert!(agent.set_module_record_status(&key, ModuleStatus::Linked));
    assert!(agent.set_module_record_dfs_state(&key, Some(3), Some(2)));
    assert!(agent.set_module_record_evaluation_error(&key, Some(Value::from_smi(7))));
    assert!(agent.set_module_record_import_meta_properties(
        &key,
        ImportMetaProperties::new(vec![ImportMetaProperty {
            key: "url".into(),
            value: ImportMetaValue::String("file:///tmp/main.mjs".into()),
        }]),
    ));
    assert!(agent.set_module_record_resolved_exports(
        &key,
        vec![ModuleResolvedExport::new(
            AtomId::from_raw(203),
            ModuleResolvedExportTarget::Binding {
                environment: module_env,
                slot: 0,
            },
        )],
    ));

    let stored = agent
        .module_record(&key)
        .expect("module record should be stored on the agent");
    assert_eq!(stored.display_name(), "/tmp/main.mjs");
    assert_eq!(stored.requested_modules()[0].resolved_key(), Some(&dep));
    assert_eq!(stored.code(), CodeRef::from_raw(7));
    assert_eq!(stored.environment(), Some(module_env));
    assert_eq!(
        stored.import_meta_properties(),
        Some(&ImportMetaProperties::new(vec![ImportMetaProperty {
            key: "url".into(),
            value: ImportMetaValue::String("file:///tmp/main.mjs".into()),
        }]))
    );
    assert_eq!(
        stored.resolved_export(AtomId::from_raw(203)),
        Some(ModuleResolvedExport::new(
            AtomId::from_raw(203),
            ModuleResolvedExportTarget::Binding {
                environment: module_env,
                slot: 0,
            },
        ))
    );
    assert_eq!(stored.status(), ModuleStatus::Linked);
    assert_eq!(stored.dfs_index(), Some(3));
    assert_eq!(stored.dfs_ancestor_index(), Some(2));
    assert_eq!(stored.evaluation_error(), Some(Value::from_smi(7)));
}

#[test]
fn module_environment_slot_aliases_follow_live_import_targets() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let outer = agent
        .default_realm()
        .expect("default realm should exist after boot")
        .global_env();

    let exporter_layout = agent.alloc_environment_layout(EnvironmentLayout::new(
        EnvironmentLayoutKind::Module,
        [EnvironmentBindingLayout::new(
            Some(AtomId::from_raw(301)),
            EnvironmentSlotFlags::mutable_lexical(),
        )],
        true,
    ));
    let importer_layout = agent.alloc_environment_layout(EnvironmentLayout::new(
        EnvironmentLayoutKind::Module,
        [EnvironmentBindingLayout::new(
            Some(AtomId::from_raw(302)),
            EnvironmentSlotFlags::immutable_lexical(),
        )],
        true,
    ));
    let exporter = agent
        .alloc_module_environment(Some(outer), exporter_layout, AllocationLifetime::Default)
        .expect("exporter environment should allocate");
    let importer = agent
        .alloc_module_environment(Some(outer), importer_layout, AllocationLifetime::Default)
        .expect("importer environment should allocate");

    assert!(agent.init_environment_slot(exporter, 0, Value::from_smi(1)));
    assert!(agent.set_module_binding_alias(
        importer,
        0,
        Some(ModuleBindingAlias::new(exporter, 0))
    ));
    assert_eq!(
        agent.module_binding_alias(importer, 0),
        Some(ModuleBindingAlias::new(exporter, 0))
    );
    assert_eq!(
        agent.environment_slot(importer, 0),
        Some(Value::from_smi(1))
    );

    assert!(agent.set_environment_slot(exporter, 0, Value::from_smi(9)));
    assert_eq!(
        agent.environment_slot(importer, 0),
        Some(Value::from_smi(9))
    );

    assert!(agent.set_environment_slot(importer, 0, Value::from_smi(17)));
    assert_eq!(
        agent.environment_slot(exporter, 0),
        Some(Value::from_smi(17))
    );
}

#[test]
fn global_and_object_environment_families_keep_binding_domains_separate() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let default_realm = agent
        .default_realm()
        .expect("default realm should exist after boot");
    let root_shape = default_realm
        .root_shape()
        .expect("default realm should carry a root shape");
    let (global_object, binding_object) = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let global_object = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        );
        let binding_object = objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape),
            AllocationLifetime::Default,
        );
        (global_object, binding_object)
    });

    let global_layout = agent.alloc_environment_layout(EnvironmentLayout::new(
        EnvironmentLayoutKind::Global,
        [EnvironmentBindingLayout::new(
            Some(AtomId::from_raw(91)),
            EnvironmentSlotFlags::mutable_lexical(),
        )],
        true,
    ));
    let global_env = agent
        .alloc_global_environment(
            Some(default_realm.global_env()),
            global_layout,
            global_object,
            AllocationLifetime::Default,
        )
        .expect("global environment should allocate");
    assert_eq!(
        agent.environment_slot(global_env, 0),
        Some(Value::uninitialized_lexical())
    );
    assert!(!agent.global_has_lexical_name(global_env, AtomId::from_raw(91)));
    assert!(agent.global_add_lexical_name(global_env, AtomId::from_raw(91)));
    assert!(agent.global_has_lexical_name(global_env, AtomId::from_raw(91)));
    assert!(agent.global_set_lexical_binding(global_env, AtomId::from_raw(91), global_env, 0));
    let lexical_binding = agent
        .global_lexical_binding(global_env, AtomId::from_raw(91))
        .expect("global lexical binding should be tracked");
    assert_eq!(lexical_binding.name(), AtomId::from_raw(91));
    assert_eq!(lexical_binding.environment(), global_env);
    assert_eq!(lexical_binding.slot(), 0);
    assert!(!agent.global_has_var_name(global_env, AtomId::from_raw(92)));
    assert!(agent.global_add_var_name(global_env, AtomId::from_raw(92)));
    assert!(agent.global_has_var_name(global_env, AtomId::from_raw(92)));

    let global_record = agent
        .global_environment(global_env)
        .expect("global environment should be queryable");
    assert_eq!(global_record.outer(), Some(default_realm.global_env()));
    assert_eq!(global_record.global_object(), global_object);
    assert_eq!(global_record.layout(), global_layout);
    assert!(global_record.has_lexical_name(AtomId::from_raw(91)));
    assert_eq!(global_record.lexical_bindings(), &[lexical_binding]);
    assert!(global_record.has_var_name(AtomId::from_raw(92)));
    assert_eq!(
        agent.environment_outer(global_env),
        Some(Some(default_realm.global_env()))
    );
    assert!(agent.environment_is_global(global_env));
    assert_eq!(
        agent.global_environment_object(global_env),
        Some(global_object)
    );
    assert_eq!(
        agent.global_environment_layout(global_env),
        Some(global_layout)
    );
    assert!(matches!(
        agent.environment(global_env),
        Some(EnvironmentRecord::Global(_))
    ));

    let object_env = agent.alloc_object_environment(
        Some(global_env),
        binding_object,
        true,
        AllocationLifetime::Default,
    );
    let object_record = agent
        .object_environment(object_env)
        .expect("object environment should be queryable");
    assert_eq!(object_record.outer(), Some(global_env));
    assert_eq!(object_record.binding_object(), binding_object);
    assert!(object_record.with_environment());
    assert_eq!(agent.environment_outer(object_env), Some(Some(global_env)));
    assert!(!agent.environment_is_global(object_env));
    assert_eq!(agent.global_environment_object(object_env), None);
    assert_eq!(agent.global_environment_layout(object_env), None);
    assert_eq!(agent.environment_slots(object_env), None);
    assert!(matches!(
        agent.environment(object_env),
        Some(EnvironmentRecord::Object(_))
    ));
}

#[test]
fn execution_context_stack_tracks_script_module_builtin_job_and_bytecode_entries() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent
        .default_realm()
        .expect("default realm should exist after boot");
    let env = realm.global_env();
    let private_layout = agent.alloc_environment_layout(EnvironmentLayout::empty(
        EnvironmentLayoutKind::Private,
        true,
    ));
    let private_env = agent
        .alloc_private_environment(Some(env), private_layout, AllocationLifetime::Default)
        .expect("private environment should allocate");
    let receiver = Value::undefined();
    let new_target = realm.global_object();
    let configured_bytecode =
        ExecutionContext::bytecode(realm.id(), CodeRef::from_raw(11).unwrap(), env, env)
            .with_private_env(Some(private_env))
            .with_this_state(ThisState::Value(receiver))
            .with_new_target(Some(new_target));

    agent.push_script_context(realm.id(), env, env);
    agent.push_module_context(realm.id(), env, env);
    agent.push_builtin_context(realm.id(), env, env);
    agent.push_job_context(realm.id(), ExecutableId::Builtin, env, env);
    agent.push_bytecode_context(realm.id(), CodeRef::from_raw(9).unwrap(), env, env);

    let contexts = agent.execution_contexts().to_vec();
    assert_eq!(contexts.len(), 5);
    assert_eq!(contexts[0].kind(), ExecutionContextKind::Script);
    assert_eq!(contexts[1].kind(), ExecutionContextKind::Module);
    assert_eq!(contexts[2].kind(), ExecutionContextKind::Builtin);
    assert_eq!(contexts[3].kind(), ExecutionContextKind::Job);
    assert_eq!(contexts[4].kind(), ExecutionContextKind::Function);
    assert_eq!(
        contexts[4].executable(),
        ExecutableId::Bytecode(CodeRef::from_raw(9).unwrap())
    );
    assert_eq!(contexts[4].private_env(), None);
    assert_eq!(contexts[4].this_state(), ThisState::Uninitialized);
    assert_eq!(contexts[4].new_target(), None);
    assert_eq!(agent.current_execution_context(), Some(contexts[4]));
    assert_eq!(configured_bytecode.private_env(), Some(private_env));
    assert_eq!(configured_bytecode.this_state(), ThisState::Value(receiver));
    assert_eq!(configured_bytecode.new_target(), Some(new_target));

    assert_eq!(agent.pop_execution_context(), Some(contexts[4]));
    assert_eq!(agent.pop_execution_context(), Some(contexts[3]));
    assert_eq!(agent.pop_execution_context(), Some(contexts[2]));
    assert_eq!(agent.pop_execution_context(), Some(contexts[1]));
    assert_eq!(agent.pop_execution_context(), Some(contexts[0]));
    assert_eq!(agent.pop_execution_context(), None);
}

#[test]
fn eval_execution_context_constructor_and_stack_entry_use_eval_kind() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent
        .default_realm()
        .expect("default realm should exist after boot");
    let env = realm.global_env();

    let eval = ExecutionContext::eval(realm.id(), env, env)
        .with_this_state(ThisState::Lexical)
        .with_private_env(Some(env));
    // The builder assertions exercise the configured local value, while the pushed stack
    // entry intentionally checks the default eval context created by `push_eval_context`.
    agent.push_eval_context(realm.id(), env, env);

    assert_eq!(eval.kind(), ExecutionContextKind::Eval);
    assert_eq!(eval.executable(), ExecutableId::Script);
    assert_eq!(eval.private_env(), Some(env));
    assert_eq!(eval.this_state(), ThisState::Lexical);
    assert_eq!(
        agent.current_execution_context(),
        Some(ExecutionContext::eval(realm.id(), env, env))
    );
}

#[test]
#[should_panic(expected = "runtime job id overflowed supported u32 range")]
fn runtime_job_ids_fail_loudly_on_overflow() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent
        .default_realm()
        .expect("default realm should exist after boot");
    agent.set_next_job_id_for_test(u32::MAX);

    let _ = agent.enqueue_job(
        HostJobKind::Script,
        ExecutableId::Script,
        Some(realm.id()),
        Some("overflow".into()),
    );
}

#[test]
#[should_panic(expected = "agent id overflowed supported u32 range")]
fn cluster_agent_ids_fail_loudly_on_overflow() {
    let mut cluster = AgentCluster::new();
    cluster.set_next_agent_id_for_test(u32::MAX);

    let _ = cluster.add_agent(None, Some("overflow".into()));
}

#[test]
fn runtime_spawns_agents_starts_threads_and_observes_job_enqueue() {
    let host = TestHost::new();
    let mut runtime = Runtime::new(host.clone());

    let worker = runtime
        .spawn_agent(AgentSpawnKind::Harness, Some("worker".into()))
        .unwrap();
    let thread = runtime
        .start_agent_thread(
            worker,
            AgentThreadStartKind::Harness,
            Some("worker-thread".into()),
        )
        .unwrap();
    let job = runtime
        .enqueue_job(
            worker,
            HostJobKind::Harness,
            ExecutableId::Builtin,
            None,
            Some("harness-job".into()),
        )
        .unwrap();

    assert_eq!(worker.get(), 2);
    assert_eq!(thread, HostThreadId::from_raw(1).unwrap());
    assert_eq!(
        runtime.root_cluster().agent(worker).unwrap().bound_thread(),
        Some(thread)
    );
    assert_eq!(
        runtime
            .root_cluster()
            .agent(worker)
            .unwrap()
            .total_queued_jobs(),
        1
    );
    assert_eq!(job.get(), 1);

    let snapshot = host.snapshot();
    assert!(matches!(snapshot.calls[0], HostCall::CreateAgent(_)));
    assert!(matches!(snapshot.calls[1], HostCall::StartAgentThread(_)));
    assert!(matches!(snapshot.calls[2], HostCall::ObserveJob(_)));
}

#[test]
fn cluster_owns_shared_backing_store_and_wait_queue_coordination() {
    let mut cluster = AgentCluster::new();
    let root = cluster.root_agent_id();
    let worker = cluster.add_agent(None, Some("worker".into()));
    let shared_buffer = HostSharedBufferId::from_raw(7).unwrap();
    let backing_store = cluster
        .register_shared_backing_store(root, 128)
        .expect("shared backing store should allocate");
    let location = WaitLocation::new(backing_store, 16);

    assert!(cluster.cache_shared_backing_store_handle(backing_store, shared_buffer));
    assert!(cluster.share_shared_backing_store(backing_store, worker));
    assert_eq!(
        cluster
            .shared_backing_store(backing_store)
            .unwrap()
            .visible_to(),
        &[root, worker]
    );
    assert_eq!(
        cluster
            .shared_backing_store(backing_store)
            .unwrap()
            .host_shared_buffer(),
        Some(shared_buffer)
    );

    assert!(cluster.park_agent(location, ParkedAgentRecord::new(root, None, false)));
    assert!(cluster.park_agent(
        location,
        ParkedAgentRecord::new(worker, Some(HostThreadId::from_raw(3).unwrap()), true),
    ));
    assert_eq!(cluster.waiter_count(location), 2);

    let woken = cluster.unpark_agents(location, 1);
    assert_eq!(woken, vec![ParkedAgentRecord::new(root, None, false)]);
    assert_eq!(cluster.waiter_count(location), 1);
}

#[test]
fn cluster_owned_backing_store_handles_are_not_agent_local() {
    let mut cluster = AgentCluster::new();
    let worker = cluster.add_agent(None, Some("worker".into()));
    let store = cluster
        .root_agent_mut()
        .allocate_backing_store(4)
        .expect("local backing store should allocate");

    assert_eq!(
        cluster
            .agent(worker)
            .unwrap()
            .backing_store_byte_length(store),
        Some(4)
    );
    assert_eq!(
        cluster
            .agent(worker)
            .unwrap()
            .backing_store_is_shared(store),
        Some(false)
    );
    assert!(cluster.root_agent_mut().backing_store_set_byte(store, 0, 7));
    assert_eq!(
        cluster
            .agent(worker)
            .unwrap()
            .backing_store_get_byte(store, 0),
        Some(7)
    );
    assert!(cluster
        .agent_mut(worker)
        .unwrap()
        .detach_backing_store(store));
    assert_eq!(
        cluster.root_agent().backing_store_is_detached(store),
        Some(true)
    );
    assert_eq!(
        cluster.root_agent().backing_store_byte_length(store),
        Some(0)
    );
}

#[test]
fn cluster_shared_backing_store_metadata_tracks_sharedness_and_refuses_detach() {
    let mut cluster = AgentCluster::new();
    let root = cluster.root_agent_id();
    let shared_buffer = HostSharedBufferId::from_raw(9).unwrap();
    let store = cluster
        .register_shared_backing_store(root, 64)
        .expect("shared backing store should allocate");

    assert!(cluster.cache_shared_backing_store_handle(store, shared_buffer));

    assert_eq!(
        cluster.root_agent().backing_store_byte_length(store),
        Some(64)
    );
    assert_eq!(
        cluster.root_agent().backing_store_is_shared(store),
        Some(true)
    );
    assert_eq!(
        cluster.root_agent().backing_store_is_detached(store),
        Some(false)
    );
    assert!(!cluster.root_agent_mut().detach_backing_store(store));
    assert_eq!(
        cluster.root_agent().backing_store_byte_length(store),
        Some(64)
    );
    assert_eq!(
        cluster
            .shared_backing_store(store)
            .expect("shared backing store record should exist")
            .host_shared_buffer(),
        Some(shared_buffer)
    );
}

#[test]
fn wait_location_uses_backing_store_identity_not_host_shared_buffer_id() {
    let store = BackingStoreRef::from_raw(13).unwrap();
    let location = WaitLocation::new(store, 24);

    assert_eq!(location.backing_store, store);
    assert_eq!(location.byte_offset, 24);
}

#[test]
fn backing_store_bit_helpers_round_trip_local_and_shared_records() {
    let mut cluster = AgentCluster::new();
    let local = cluster
        .root_agent_mut()
        .allocate_backing_store(12)
        .expect("local backing store should allocate");
    let shared = cluster
        .register_shared_backing_store(cluster.root_agent_id(), 12)
        .expect("shared backing store should allocate");
    let agent = cluster.root_agent_mut();

    assert!(agent.backing_store_store_bits(local, 2, 4, 0x0403_0201));
    assert_eq!(
        agent.backing_store_load_bits(local, 2, 4),
        Some(0x0403_0201)
    );
    assert_eq!(agent.backing_store_get_byte(local, 3), Some(0x02));

    assert!(agent.backing_store_store_bits(shared, 4, 8, 0x0807_0605_0403_0201));
    assert_eq!(
        agent.backing_store_load_bits(shared, 4, 8),
        Some(0x0807_0605_0403_0201)
    );
    assert!(!agent.backing_store_store_bits(shared, 10, 4, 0));
    assert_eq!(agent.backing_store_load_bits(shared, 10, 4), None);
}

#[test]
fn shared_backing_store_atomic_helpers_round_trip_integer_bits() {
    let mut cluster = AgentCluster::new();
    let store = cluster
        .register_shared_backing_store(cluster.root_agent_id(), 8)
        .expect("shared backing store should allocate");
    let agent = cluster.root_agent_mut();

    assert!(agent.backing_store_atomic_store_bits(store, 0, 4, 0x0403_0201));
    assert_eq!(
        agent.backing_store_atomic_load_bits(store, 0, 4),
        Some(0x0403_0201)
    );
    assert_eq!(agent.backing_store_get_byte(store, 2), Some(0x03));
    assert_eq!(
        agent.backing_store_atomic_compare_exchange_bits(store, 0, 4, 0x0403_0201, 0x0807_0605),
        Some(0x0403_0201)
    );
    assert_eq!(
        agent.backing_store_atomic_load_bits(store, 0, 4),
        Some(0x0807_0605)
    );
}

#[test]
fn shared_memory_wait_queue_preserves_fifo_after_cancellation_and_async_wake() {
    let mut cluster = AgentCluster::new();
    let root = cluster.root_agent_id();
    let worker = cluster.add_agent(None, Some("worker".into()));
    let store = cluster
        .register_shared_backing_store(root, 16)
        .expect("shared backing store should allocate");
    let location = WaitLocation::new(store, 4);
    let worker_thread = HostThreadId::from_raw(3).unwrap();
    let async_promise = ObjectRef::from_raw(41).unwrap();

    let first = cluster
        .root_agent_mut()
        .park_shared_memory_waiter(location, ParkedAgentRecord::new(root, None, false))
        .expect("blocking waiter should receive a token");
    let _async = cluster
        .root_agent_mut()
        .park_async_shared_memory_waiter(location, AsyncWaiterRecord::new(root, async_promise));
    let _third = cluster.root_agent_mut().park_shared_memory_waiter(
        location,
        ParkedAgentRecord::new(worker, Some(worker_thread), false),
    );

    assert!(cluster
        .root_agent_mut()
        .remove_shared_memory_waiter(location, first));
    let woken = cluster
        .root_agent_mut()
        .wake_shared_memory_waiters(location, 2);

    assert_eq!(woken.len(), 2);
    assert_eq!(
        woken[0].kind(),
        WaiterKind::Async(AsyncWaiterRecord::new(root, async_promise))
    );
    assert_eq!(
        woken[1].kind(),
        WaiterKind::Blocking(ParkedAgentRecord::new(worker, Some(worker_thread), false))
    );
    assert_eq!(cluster.root_agent().shared_memory_waiter_count(location), 0);
}

#[test]
fn shared_backing_store_visibility_crosses_os_threads() {
    let runtime = std::sync::Arc::new(std::sync::Mutex::new(BackingStoreRuntime::new()));
    let store = runtime
        .lock()
        .expect("backing-store mutex should stay healthy")
        .allocate_shared(8)
        .expect("shared backing store should allocate");
    let worker = runtime.clone();

    std::thread::spawn(move || {
        let runtime = worker.lock().expect("worker mutex should stay healthy");
        assert!(runtime.atomic_load_bits(store, 0, 4).is_some());
    })
    .join()
    .expect("worker thread should complete");

    let worker = runtime.clone();
    std::thread::spawn(move || {
        let mut runtime = worker.lock().expect("worker mutex should stay healthy");
        assert!(runtime.atomic_store_bits(store, 0, 4, 0x7856_3412));
    })
    .join()
    .expect("writer thread should complete");

    let runtime = runtime
        .lock()
        .expect("backing-store mutex should stay healthy");
    assert_eq!(runtime.atomic_load_bits(store, 0, 4), Some(0x7856_3412));
}

#[test]
fn shared_wait_queue_wakes_waiters_across_os_threads() {
    let runtime = std::sync::Arc::new(std::sync::Mutex::new(SharedMemoryRuntime::default()));
    let location = WaitLocation::new(BackingStoreRef::from_raw(17).unwrap(), 8);
    let parked = ParkedAgentRecord::new(AgentId::from_raw(1).unwrap(), None, false);
    let token = runtime
        .lock()
        .expect("shared-memory mutex should stay healthy")
        .park_agent(location, parked)
        .expect("waiter should receive a token");
    let worker = runtime.clone();

    let woken = std::thread::spawn(move || {
        loop {
            let count = worker
                .lock()
                .expect("worker mutex should stay healthy")
                .waiter_count(location);
            if count == 1 {
                break;
            }
            std::thread::yield_now();
        }
        worker
            .lock()
            .expect("worker mutex should stay healthy")
            .wake_waiters(location, 1)
    })
    .join()
    .expect("wake thread should complete");

    assert_eq!(
        woken,
        vec![WaiterRecord::new(token, WaiterKind::Blocking(parked))]
    );
    assert_eq!(
        runtime
            .lock()
            .expect("shared-memory mutex should stay healthy")
            .waiter_count(location),
        0
    );
}

#[test]
fn runtime_marker_round_trips_phase3_dependencies() {
    let property_name = AtomId::from_raw(19);
    let type_marker = TypeOwnershipMarker::new(property_name);
    let host = lyng_js_host::HostMarker::new(type_marker, property_name);
    let heap = PrimitiveHeapMarker::new(type_marker, SourceId::new(7));
    let objects = ObjectSubstrateMarker::new(
        heap,
        property_name,
        ObjectKind::Ordinary,
        ObjectFlags::extensible(),
    );
    let layout = EnvironmentLayout::new(
        EnvironmentLayoutKind::Declarative,
        [EnvironmentBindingLayout::new(
            Some(property_name),
            EnvironmentSlotFlags::mutable_lexical(),
        )],
        true,
    );
    let marker = RuntimeSubstrateMarker::new(
        objects,
        host,
        ExecutableId::Bytecode(CodeRef::from_raw(23).unwrap()),
        layout.clone(),
    );

    assert_eq!(marker.objects(), objects);
    assert_eq!(marker.host(), host);
    assert_eq!(
        marker.executable(),
        ExecutableId::Bytecode(CodeRef::from_raw(23).unwrap())
    );
    assert_eq!(marker.layout(), &layout);
}
