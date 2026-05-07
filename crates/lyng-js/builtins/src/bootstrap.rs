use crate::{
    BuiltinAttributes, BuiltinCache, BuiltinCallContext, BuiltinDescriptorTable,
    BuiltinInstallTarget, BuiltinIntrinsic, BuiltinPropertyDescriptor, BuiltinPropertyKeySpec,
    BuiltinPropertyValueSpec,
};
use lyng_js_common::WellKnownAtom;
use lyng_js_env::{Agent, RealmRecord};
use lyng_js_gc::AllocationLifetime;
use lyng_js_types::{
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
    EnvironmentRef, ObjectRef, PropertyDescriptor, PropertyKey, RealmRef, Value, WellKnownSymbolId,
};

mod descriptors;
mod globals;

pub use descriptors::install_descriptor_tables;

use globals::default_global_descriptors;

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
    let bootstrap_state = agent.realm_bootstrap_state(realm).unwrap_or_default();

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

#[cfg(test)]
mod tests;
