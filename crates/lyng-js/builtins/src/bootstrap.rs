use crate::{
    BuiltinAttributes, BuiltinCache, BuiltinCallContext, BuiltinDescriptorTable,
    BuiltinInstallTarget, BuiltinIntrinsic, BuiltinPropertyDescriptor, BuiltinPropertyKeySpec,
    BuiltinPropertyValueSpec,
};
use lyng_js_common::WellKnownAtom;
use lyng_js_env::{Agent, RealmBootstrapState, RealmRecord};
use lyng_js_gc::AllocationLifetime;
use lyng_js_types::{
    js3_aggregate_error_builtin, js3_array_buffer_builtin, js3_array_builtin,
    js3_big_int64_array_builtin, js3_big_uint64_array_builtin, js3_bigint_builtin,
    js3_boolean_builtin, js3_data_view_builtin, js3_date_builtin, js3_decode_uri_builtin,
    js3_decode_uri_component_builtin, js3_encode_uri_builtin, js3_encode_uri_component_builtin,
    js3_error_builtin, js3_eval_builtin, js3_eval_error_builtin, js3_finalization_registry_builtin,
    js3_float32_array_builtin, js3_float64_array_builtin, js3_function_builtin,
    js3_int16_array_builtin, js3_int32_array_builtin, js3_int8_array_builtin,
    js3_is_finite_builtin, js3_is_nan_builtin, js3_map_builtin, js3_number_builtin,
    js3_object_builtin, js3_parse_float_builtin, js3_parse_int_builtin, js3_promise_builtin,
    js3_range_error_builtin, js3_reference_error_builtin, js3_regexp_builtin, js3_set_builtin,
    js3_shared_array_buffer_builtin, js3_string_builtin, js3_symbol_builtin,
    js3_syntax_error_builtin, js3_type_error_builtin, js3_typed_array_builtin,
    js3_uint16_array_builtin, js3_uint32_array_builtin, js3_uint8_array_builtin,
    js3_uint8_clamped_array_builtin, js3_uri_error_builtin, js3_weak_map_builtin,
    js3_weak_ref_builtin, js3_weak_set_builtin, EnvironmentRef, ObjectRef, PropertyDescriptor,
    PropertyKey, RealmRef, Value, WellKnownSymbolId,
};

/// Realm-bootstrap mode for the shared JS3 bootstrap path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootstrapMode {
    SpecOnly,
}

/// Request payload for default-realm bootstrap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BootstrapRequest {
    mode: BootstrapMode,
}

impl BootstrapRequest {
    #[inline]
    pub const fn new(mode: BootstrapMode) -> Self {
        Self { mode }
    }

    #[inline]
    pub const fn mode(self) -> BootstrapMode {
        self.mode
    }
}

/// Typed handles returned by the shared bootstrap entrypoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BootstrapArtifacts {
    realm: RealmRef,
    global_object: ObjectRef,
    global_env: EnvironmentRef,
}

impl BootstrapArtifacts {
    #[inline]
    pub const fn new(
        realm: RealmRef,
        global_object: ObjectRef,
        global_env: EnvironmentRef,
    ) -> Self {
        Self {
            realm,
            global_object,
            global_env,
        }
    }

    #[inline]
    pub const fn realm(self) -> RealmRef {
        self.realm
    }

    #[inline]
    pub const fn global_object(self) -> ObjectRef {
        self.global_object
    }

    #[inline]
    pub const fn global_env(self) -> EnvironmentRef {
        self.global_env
    }
}

/// Errors returned by the shared default-realm bootstrap entrypoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BuiltinBootstrapError {
    MissingDefaultRealm,
    MissingRealm(RealmRef),
    MissingRootShape(RealmRef),
    MissingWellKnownSymbol(WellKnownSymbolId),
    MissingIntrinsic(BuiltinIntrinsic, RealmRef),
    MissingBuiltinFunction(lyng_js_types::BuiltinFunctionId, RealmRef),
    DefinePropertyRejected {
        target: ObjectRef,
        key: BuiltinPropertyKeySpec,
    },
}

/// Result type for the shared default-realm bootstrap entrypoint.
pub type BuiltinBootstrapResult<T, E> = Result<T, E>;

/// Shared bootstrap entrypoint surface implemented by the execution layer.
pub trait BuiltinBootstrap: BuiltinCallContext {
    /// Bootstraps one default realm using the shared JS3 bootstrap path.
    ///
    /// # Errors
    /// Returns an error when the execution layer cannot complete typed bootstrap.
    fn bootstrap_default_realm(
        &mut self,
        request: BootstrapRequest,
    ) -> BuiltinBootstrapResult<BootstrapArtifacts, Self::Error>;
}

/// Bootstraps the agent default realm using the shared JS3 bootstrap path.
///
/// # Errors
/// Returns an error when the default realm shell is missing or builtin descriptor
/// installation cannot complete.
pub fn bootstrap_default_realm(
    agent: &mut Agent,
    builtin_cache: &mut BuiltinCache,
    request: BootstrapRequest,
) -> BuiltinBootstrapResult<BootstrapArtifacts, BuiltinBootstrapError> {
    let realm = agent
        .default_realm_id()
        .ok_or(BuiltinBootstrapError::MissingDefaultRealm)?;
    bootstrap_realm(agent, builtin_cache, realm, request)
}

/// Bootstraps one selected realm using the shared JS3 bootstrap path.
///
/// # Errors
/// Returns an error when the realm shell is missing or builtin descriptor
/// installation cannot complete.
pub fn bootstrap_realm(
    agent: &mut Agent,
    builtin_cache: &mut BuiltinCache,
    realm: RealmRef,
    _request: BootstrapRequest,
) -> BuiltinBootstrapResult<BootstrapArtifacts, BuiltinBootstrapError> {
    let (global_object, global_env) = agent
        .realm(realm)
        .map(|record| {
            let _ = record
                .root_shape()
                .ok_or(BuiltinBootstrapError::MissingRootShape(realm))?;
            Ok((record.global_object(), record.global_env()))
        })
        .ok_or(BuiltinBootstrapError::MissingRealm(realm))??;
    let _ = builtin_cache
        .ensure_realm_builtins(agent, realm)
        .ok_or(BuiltinBootstrapError::MissingRootShape(realm))?;

    let artifacts = BootstrapArtifacts::new(realm, global_object, global_env);
    let bootstrap_state = agent
        .realm_bootstrap_state(realm)
        .unwrap_or_else(RealmBootstrapState::new);

    if !bootstrap_state.spec_ready() {
        install_spec_bootstrap(agent, builtin_cache, artifacts)?;
        if !agent.mark_realm_spec_bootstrapped(realm) {
            return Err(BuiltinBootstrapError::MissingRealm(realm));
        }
    }

    Ok(artifacts)
}

fn install_spec_bootstrap(
    agent: &mut Agent,
    builtin_cache: &mut BuiltinCache,
    artifacts: BootstrapArtifacts,
) -> Result<(), BuiltinBootstrapError> {
    let descriptor_rows = default_global_descriptors(agent, artifacts);
    let descriptor_tables = [BuiltinDescriptorTable::new(
        BuiltinInstallTarget::GlobalObject,
        &descriptor_rows,
    )];
    install_descriptor_tables(agent, builtin_cache, artifacts.realm(), &descriptor_tables)
}

fn default_global_descriptors(
    agent: &mut Agent,
    artifacts: BootstrapArtifacts,
) -> [BuiltinPropertyDescriptor; 61] {
    let atoms = agent.bootstrap_atoms();
    let reflect_atom = agent.atoms_mut().intern_collectible("Reflect");
    let proxy_atom = agent.atoms_mut().intern_collectible("Proxy");
    let suppressed_error_atom = agent.atoms_mut().intern_collectible("SuppressedError");
    let disposable_stack_atom = agent.atoms_mut().intern_collectible("DisposableStack");
    let async_disposable_stack_atom = agent.atoms_mut().intern_collectible("AsyncDisposableStack");
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
            BuiltinPropertyValueSpec::BuiltinFunction(js3_object_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.json()),
            BuiltinPropertyValueSpec::Data(
                intrinsics
                    .json()
                    .map(Value::from_object_ref)
                    .unwrap_or(Value::undefined()),
            ),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.function()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_function_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.array()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.map()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_map_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.set()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_set_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.weak_map()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_weak_map_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.weak_set()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_weak_set_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.weak_ref()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_weak_ref_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.finalization_registry()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_finalization_registry_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.string()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_string_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.regexp()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_regexp_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.date()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_date_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.array_buffer()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_array_buffer_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.shared_array_buffer()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_shared_array_buffer_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.data_view()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_data_view_builtin()),
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
            BuiltinPropertyValueSpec::BuiltinFunction(js3_typed_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.int8_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_int8_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.int16_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_int16_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.int32_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_int32_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.uint32_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_uint32_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.float32_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_float32_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.float64_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_float64_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.big_int64_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_big_int64_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.big_uint64_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_big_uint64_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.uint16_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_uint16_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.uint8_clamped_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_clamped_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.uint8_array()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_uint8_array_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.number()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_number_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.math()),
            BuiltinPropertyValueSpec::Data(
                intrinsics
                    .math()
                    .map(Value::from_object_ref)
                    .unwrap_or(Value::undefined()),
            ),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.bigint()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_bigint_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.boolean()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_boolean_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.symbol()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_symbol_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.promise()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_promise_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(reflect_atom),
            BuiltinPropertyValueSpec::Data(
                intrinsics
                    .reflect()
                    .map(Value::from_object_ref)
                    .unwrap_or(Value::undefined()),
            ),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(proxy_atom),
            BuiltinPropertyValueSpec::BuiltinFunction(lyng_js_types::js3_proxy_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.aggregate_error()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_aggregate_error_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(suppressed_error_atom),
            BuiltinPropertyValueSpec::BuiltinFunction(lyng_js_types::js3_suppressed_error_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(disposable_stack_atom),
            BuiltinPropertyValueSpec::BuiltinFunction(lyng_js_types::js3_disposable_stack_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(async_disposable_stack_atom),
            BuiltinPropertyValueSpec::BuiltinFunction(
                lyng_js_types::js3_async_disposable_stack_builtin(),
            ),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(WellKnownAtom::eval.id()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_eval_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.error()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_error_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.eval_error()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_eval_error_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.range_error()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_range_error_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.reference_error()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_reference_error_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.syntax_error()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_syntax_error_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.type_error()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_type_error_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.uri_error()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_uri_error_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.parse_int()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_parse_int_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.parse_float()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_parse_float_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.is_nan()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_is_nan_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.is_finite()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_is_finite_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.decode_uri()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_decode_uri_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.decode_uri_component()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_decode_uri_component_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.encode_uri()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_encode_uri_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.encode_uri_component()),
            BuiltinPropertyValueSpec::BuiltinFunction(js3_encode_uri_component_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
    ]
}

pub(crate) fn install_descriptor_tables(
    agent: &mut Agent,
    builtin_cache: &mut BuiltinCache,
    realm: RealmRef,
    descriptor_tables: &[BuiltinDescriptorTable<'_>],
) -> Result<(), BuiltinBootstrapError> {
    for table in descriptor_tables {
        let target = resolve_install_target(agent, realm, table.target())?;
        for descriptor in table.descriptors() {
            install_descriptor(agent, builtin_cache, realm, target, *descriptor)?;
        }
    }
    Ok(())
}

fn resolve_install_target(
    agent: &Agent,
    realm: RealmRef,
    target: BuiltinInstallTarget,
) -> Result<ObjectRef, BuiltinBootstrapError> {
    match target {
        BuiltinInstallTarget::GlobalObject => agent
            .realm(realm)
            .map(RealmRecord::global_object)
            .ok_or(BuiltinBootstrapError::MissingRealm(realm)),
        BuiltinInstallTarget::Intrinsic(intrinsic) => resolve_intrinsic(agent, realm, intrinsic),
        BuiltinInstallTarget::Object(object) => Ok(object),
    }
}

fn resolve_intrinsic(
    agent: &Agent,
    realm: RealmRef,
    intrinsic: BuiltinIntrinsic,
) -> Result<ObjectRef, BuiltinBootstrapError> {
    let intrinsics = agent
        .realm(realm)
        .ok_or(BuiltinBootstrapError::MissingRealm(realm))?
        .intrinsics();
    let resolved = match intrinsic {
        BuiltinIntrinsic::Object => intrinsics.object(),
        BuiltinIntrinsic::ObjectPrototype => intrinsics.object_prototype(),
        BuiltinIntrinsic::Function => intrinsics.function(),
        BuiltinIntrinsic::FunctionPrototype => intrinsics.function_prototype(),
        BuiltinIntrinsic::AsyncFunction => intrinsics.async_function(),
        BuiltinIntrinsic::AsyncFunctionPrototype => intrinsics.async_function_prototype(),
        BuiltinIntrinsic::AsyncGeneratorFunction => intrinsics.async_generator_function(),
        BuiltinIntrinsic::AsyncGeneratorFunctionPrototype => {
            intrinsics.async_generator_function_prototype()
        }
        BuiltinIntrinsic::AsyncGeneratorPrototype => intrinsics.async_generator_prototype(),
        BuiltinIntrinsic::GeneratorFunction => intrinsics.generator_function(),
        BuiltinIntrinsic::GeneratorFunctionPrototype => intrinsics.generator_function_prototype(),
        BuiltinIntrinsic::GeneratorPrototype => intrinsics.generator_prototype(),
        BuiltinIntrinsic::Array => intrinsics.array(),
        BuiltinIntrinsic::ArrayPrototype => intrinsics.array_prototype(),
        BuiltinIntrinsic::Map => intrinsics.map(),
        BuiltinIntrinsic::MapPrototype => intrinsics.map_prototype(),
        BuiltinIntrinsic::MapIteratorPrototype => intrinsics.map_iterator_prototype(),
        BuiltinIntrinsic::Set => intrinsics.set(),
        BuiltinIntrinsic::SetPrototype => intrinsics.set_prototype(),
        BuiltinIntrinsic::SetIteratorPrototype => intrinsics.set_iterator_prototype(),
        BuiltinIntrinsic::WeakMap => intrinsics.weak_map(),
        BuiltinIntrinsic::WeakMapPrototype => intrinsics.weak_map_prototype(),
        BuiltinIntrinsic::WeakSet => intrinsics.weak_set(),
        BuiltinIntrinsic::WeakSetPrototype => intrinsics.weak_set_prototype(),
        BuiltinIntrinsic::WeakRef => intrinsics.weak_ref(),
        BuiltinIntrinsic::WeakRefPrototype => intrinsics.weak_ref_prototype(),
        BuiltinIntrinsic::FinalizationRegistry => intrinsics.finalization_registry(),
        BuiltinIntrinsic::FinalizationRegistryPrototype => {
            intrinsics.finalization_registry_prototype()
        }
        BuiltinIntrinsic::ArrayBuffer => intrinsics.array_buffer(),
        BuiltinIntrinsic::ArrayBufferPrototype => intrinsics.array_buffer_prototype(),
        BuiltinIntrinsic::SharedArrayBuffer => intrinsics.shared_array_buffer(),
        BuiltinIntrinsic::SharedArrayBufferPrototype => intrinsics.shared_array_buffer_prototype(),
        BuiltinIntrinsic::DataView => intrinsics.data_view(),
        BuiltinIntrinsic::DataViewPrototype => intrinsics.data_view_prototype(),
        BuiltinIntrinsic::Atomics => intrinsics.atomics(),
        BuiltinIntrinsic::TypedArray => intrinsics.typed_array(),
        BuiltinIntrinsic::TypedArrayPrototype => intrinsics.typed_array_prototype(),
        BuiltinIntrinsic::Int8Array => intrinsics.int8_array(),
        BuiltinIntrinsic::Int8ArrayPrototype => intrinsics.int8_array_prototype(),
        BuiltinIntrinsic::Int16Array => intrinsics.int16_array(),
        BuiltinIntrinsic::Int16ArrayPrototype => intrinsics.int16_array_prototype(),
        BuiltinIntrinsic::Int32Array => intrinsics.int32_array(),
        BuiltinIntrinsic::Int32ArrayPrototype => intrinsics.int32_array_prototype(),
        BuiltinIntrinsic::Float32Array => intrinsics.float32_array(),
        BuiltinIntrinsic::Float32ArrayPrototype => intrinsics.float32_array_prototype(),
        BuiltinIntrinsic::Float64Array => intrinsics.float64_array(),
        BuiltinIntrinsic::Float64ArrayPrototype => intrinsics.float64_array_prototype(),
        BuiltinIntrinsic::BigInt64Array => intrinsics.big_int64_array(),
        BuiltinIntrinsic::BigInt64ArrayPrototype => intrinsics.big_int64_array_prototype(),
        BuiltinIntrinsic::BigUint64Array => intrinsics.big_uint64_array(),
        BuiltinIntrinsic::BigUint64ArrayPrototype => intrinsics.big_uint64_array_prototype(),
        BuiltinIntrinsic::Uint32Array => intrinsics.uint32_array(),
        BuiltinIntrinsic::Uint32ArrayPrototype => intrinsics.uint32_array_prototype(),
        BuiltinIntrinsic::Uint16Array => intrinsics.uint16_array(),
        BuiltinIntrinsic::Uint16ArrayPrototype => intrinsics.uint16_array_prototype(),
        BuiltinIntrinsic::Uint8ClampedArray => intrinsics.uint8_clamped_array(),
        BuiltinIntrinsic::Uint8ClampedArrayPrototype => intrinsics.uint8_clamped_array_prototype(),
        BuiltinIntrinsic::Uint8Array => intrinsics.uint8_array(),
        BuiltinIntrinsic::Uint8ArrayPrototype => intrinsics.uint8_array_prototype(),
        BuiltinIntrinsic::IteratorPrototype => intrinsics.iterator_prototype(),
        BuiltinIntrinsic::AsyncIteratorPrototype => intrinsics.async_iterator_prototype(),
        BuiltinIntrinsic::AsyncFromSyncIteratorPrototype => {
            intrinsics.async_from_sync_iterator_prototype()
        }
        BuiltinIntrinsic::ArrayIteratorPrototype => intrinsics.array_iterator_prototype(),
        BuiltinIntrinsic::String => intrinsics.string(),
        BuiltinIntrinsic::StringPrototype => intrinsics.string_prototype(),
        BuiltinIntrinsic::StringIteratorPrototype => intrinsics.string_iterator_prototype(),
        BuiltinIntrinsic::RegExp => intrinsics.regexp(),
        BuiltinIntrinsic::RegExpPrototype => intrinsics.regexp_prototype(),
        BuiltinIntrinsic::Date => intrinsics.date(),
        BuiltinIntrinsic::DatePrototype => intrinsics.date_prototype(),
        BuiltinIntrinsic::Number => intrinsics.number(),
        BuiltinIntrinsic::NumberPrototype => intrinsics.number_prototype(),
        BuiltinIntrinsic::Math => intrinsics.math(),
        BuiltinIntrinsic::BigInt => intrinsics.bigint(),
        BuiltinIntrinsic::BigIntPrototype => intrinsics.bigint_prototype(),
        BuiltinIntrinsic::Boolean => intrinsics.boolean(),
        BuiltinIntrinsic::BooleanPrototype => intrinsics.boolean_prototype(),
        BuiltinIntrinsic::Symbol => intrinsics.symbol(),
        BuiltinIntrinsic::SymbolPrototype => intrinsics.symbol_prototype(),
        BuiltinIntrinsic::Json => intrinsics.json(),
        BuiltinIntrinsic::Reflect => intrinsics.reflect(),
        BuiltinIntrinsic::Proxy => intrinsics.proxy(),
        BuiltinIntrinsic::Error => intrinsics.error(),
        BuiltinIntrinsic::ErrorPrototype => intrinsics.error_prototype(),
        BuiltinIntrinsic::EvalError => intrinsics.eval_error(),
        BuiltinIntrinsic::EvalErrorPrototype => intrinsics.eval_error_prototype(),
        BuiltinIntrinsic::RangeError => intrinsics.range_error(),
        BuiltinIntrinsic::RangeErrorPrototype => intrinsics.range_error_prototype(),
        BuiltinIntrinsic::ReferenceError => intrinsics.reference_error(),
        BuiltinIntrinsic::ReferenceErrorPrototype => intrinsics.reference_error_prototype(),
        BuiltinIntrinsic::SyntaxError => intrinsics.syntax_error(),
        BuiltinIntrinsic::SyntaxErrorPrototype => intrinsics.syntax_error_prototype(),
        BuiltinIntrinsic::TypeError => intrinsics.type_error(),
        BuiltinIntrinsic::TypeErrorPrototype => intrinsics.type_error_prototype(),
        BuiltinIntrinsic::UriError => intrinsics.uri_error(),
        BuiltinIntrinsic::UriErrorPrototype => intrinsics.uri_error_prototype(),
        BuiltinIntrinsic::AggregateError => intrinsics.aggregate_error(),
        BuiltinIntrinsic::AggregateErrorPrototype => intrinsics.aggregate_error_prototype(),
        BuiltinIntrinsic::SuppressedError => intrinsics.suppressed_error(),
        BuiltinIntrinsic::SuppressedErrorPrototype => intrinsics.suppressed_error_prototype(),
        BuiltinIntrinsic::Promise => intrinsics.promise(),
        BuiltinIntrinsic::PromisePrototype => intrinsics.promise_prototype(),
        BuiltinIntrinsic::DisposableStack => intrinsics.disposable_stack(),
        BuiltinIntrinsic::DisposableStackPrototype => intrinsics.disposable_stack_prototype(),
        BuiltinIntrinsic::AsyncDisposableStack => intrinsics.async_disposable_stack(),
        BuiltinIntrinsic::AsyncDisposableStackPrototype => {
            intrinsics.async_disposable_stack_prototype()
        }
        BuiltinIntrinsic::ThrowTypeError => intrinsics.throw_type_error(),
    };
    resolved.ok_or(BuiltinBootstrapError::MissingIntrinsic(intrinsic, realm))
}

fn install_descriptor(
    agent: &mut Agent,
    builtin_cache: &mut BuiltinCache,
    realm: RealmRef,
    target: ObjectRef,
    descriptor: BuiltinPropertyDescriptor,
) -> Result<(), BuiltinBootstrapError> {
    let key = resolve_property_key(agent, descriptor.key())?;
    let attributes = descriptor.attributes();
    let mut property_descriptor = PropertyDescriptor::new();
    match descriptor.value() {
        BuiltinPropertyValueSpec::Data(value) => {
            property_descriptor.set_value(value);
            property_descriptor.set_writable(attributes.writable());
        }
        BuiltinPropertyValueSpec::BuiltinFunction(entry) => {
            property_descriptor.set_value(resolve_builtin_function_value(
                agent,
                builtin_cache,
                realm,
                entry,
            )?);
            property_descriptor.set_writable(attributes.writable());
        }
        BuiltinPropertyValueSpec::Accessor { get, set } => {
            property_descriptor.set_getter(resolve_accessor_builtin_value(
                agent,
                builtin_cache,
                realm,
                get,
            )?);
            property_descriptor.set_setter(resolve_accessor_builtin_value(
                agent,
                builtin_cache,
                realm,
                set,
            )?);
        }
    }
    property_descriptor.set_enumerable(attributes.enumerable());
    property_descriptor.set_configurable(attributes.configurable());
    let defined = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.define_own_property(
            &mut mutator,
            target,
            key,
            property_descriptor,
            AllocationLifetime::Default,
        )
    });
    if matches!(defined, Ok(true)) {
        return Ok(());
    }

    Err(BuiltinBootstrapError::DefinePropertyRejected {
        target,
        key: descriptor.key(),
    })
}

fn resolve_property_key(
    agent: &Agent,
    key: BuiltinPropertyKeySpec,
) -> Result<PropertyKey, BuiltinBootstrapError> {
    match key {
        BuiltinPropertyKeySpec::Index(index) => Ok(PropertyKey::Index(index)),
        BuiltinPropertyKeySpec::Atom(atom) => Ok(PropertyKey::from_atom(atom)),
        BuiltinPropertyKeySpec::WellKnownSymbol(symbol_id) => agent
            .well_known_symbol(symbol_id)
            .map(PropertyKey::from_symbol)
            .ok_or(BuiltinBootstrapError::MissingWellKnownSymbol(symbol_id)),
    }
}

fn resolve_builtin_function_value(
    agent: &mut Agent,
    builtin_cache: &mut BuiltinCache,
    realm: RealmRef,
    entry: lyng_js_types::BuiltinFunctionId,
) -> Result<Value, BuiltinBootstrapError> {
    builtin_cache
        .builtin_constant(agent, realm, entry)
        .ok_or(BuiltinBootstrapError::MissingBuiltinFunction(entry, realm))
}

fn resolve_accessor_builtin_value(
    agent: &mut Agent,
    builtin_cache: &mut BuiltinCache,
    realm: RealmRef,
    entry: Option<lyng_js_types::BuiltinFunctionId>,
) -> Result<Value, BuiltinBootstrapError> {
    let Some(entry) = entry else {
        return Ok(Value::undefined());
    };
    resolve_builtin_function_value(agent, builtin_cache, realm, entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_gc::AllocationLifetime;
    use lyng_js_host::NoopHostHooks;
    use lyng_js_types::{
        js3_array_from_async_builtin, js3_array_iterator_next_builtin,
        js3_array_species_getter_builtin, js3_array_values_builtin, js3_error_to_string_builtin,
        js3_symbol_to_primitive_builtin, PropertyKey, Value,
    };

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
            .builtin_constant(agent, artifacts.realm(), js3_array_from_async_builtin())
            .expect("Array.fromAsync builtin should resolve");
        let species_getter = cache
            .builtin_constant(agent, artifacts.realm(), js3_array_species_getter_builtin())
            .expect("Array @@species getter should resolve");
        let values_value = cache
            .builtin_constant(agent, artifacts.realm(), js3_array_values_builtin())
            .expect("Array.prototype.values builtin should resolve");
        let iterator_next = cache
            .builtin_constant(agent, artifacts.realm(), js3_array_iterator_next_builtin())
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
                get: Some(js3_symbol_to_primitive_builtin()),
                set: Some(js3_error_to_string_builtin()),
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
            .builtin_constant(agent, artifacts.realm(), js3_symbol_to_primitive_builtin())
            .expect("getter builtin constant should resolve");
        let setter = cache
            .builtin_constant(agent, artifacts.realm(), js3_error_to_string_builtin())
            .expect("setter builtin constant should resolve");

        assert_eq!(property.value(), None);
        assert_eq!(property.getter(), Some(getter));
        assert_eq!(property.setter(), Some(setter));
        assert_eq!(property.writable(), None);
        assert_eq!(property.enumerable(), Some(true));
        assert_eq!(property.configurable(), Some(true));
    }
}
