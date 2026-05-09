//! Builtin bootstrap and native builtin dispatch for lyng-js.
//!
//! Ownership: `lyng_js_builtins` owns builtin registration tables, descriptor
//! table shapes, bootstrap entrypoint surfaces, and builtin call contracts. It
//! owns the public core builtin namespace, while the reserved
//! `internal_*` builtin IDs remain a separate lowering-helper lane bridged
//! through this crate. It does not own VM dispatch, runtime state, or object
//! semantics that belong in `lyng_js_vm`, `lyng_js_env`, or `lyng_js_objects`.

#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    reason = "builtin bootstrap APIs expose domain-specific descriptors and cheap registry accessors across crates"
)]

mod bootstrap;
mod context;
mod descriptors;
mod internal;
mod public;
mod registry;

pub use bootstrap::{
    bootstrap_default_realm, bootstrap_realm, BootstrapArtifacts, BootstrapMode, BootstrapRequest,
    BuiltinBootstrap, BuiltinBootstrapError, BuiltinBootstrapResult,
};
pub use context::{
    BuiltinCallContext, BuiltinFunctionAllocation, BuiltinHandler, BuiltinInvocation,
    BuiltinResult, DynamicFunctionKind, DynamicFunctionPlan,
};
pub use descriptors::{
    BuiltinAttributes, BuiltinDescriptorTable, BuiltinInstallTarget, BuiltinIntrinsic,
    BuiltinPropertyDescriptor, BuiltinPropertyKeySpec, BuiltinPropertyValueSpec,
};
pub use internal::{
    dispatch_internal_builtin, internal_builtin_metadata, InternalBuiltinCache,
    InternalBuiltinDispatchContext, InternalRealmBuiltins,
};
pub use public::{
    builtin_metadata, dispatch_builtin, public_builtin_metadata, BuiltinCache,
    PublicBuiltinDispatchContext, PublicRealmBuiltins, RealmBuiltins,
};
pub use registry::{
    BuiltinEntryMetadata, BuiltinRegistry, BuiltinRegistryEntry, BuiltinRegistryError,
};
