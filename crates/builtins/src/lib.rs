//! Builtin bootstrap and native builtin dispatch for lyng.
//!
//! Ownership: `lyng_builtins` owns builtin registration tables, descriptor
//! table shapes, bootstrap entrypoint surfaces, and builtin call contracts. It
//! owns the public core builtin namespace, while the reserved
//! `internal_*` builtin IDs remain a separate lowering-helper lane bridged
//! through this crate. It does not own VM dispatch, runtime state, or object
//! semantics that belong in `lyng_vm`, `lyng_env`, or `lyng_objects`.

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
    BootstrapArtifacts, BootstrapMode, BootstrapRequest, BuiltinBootstrap, BuiltinBootstrapError,
    BuiltinBootstrapResult, bootstrap_default_realm, bootstrap_realm,
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
    InternalBuiltinCache, InternalBuiltinDispatchContext, InternalRealmBuiltins,
    dispatch_internal_builtin, internal_builtin_metadata,
};
pub use public::{
    BuiltinCache, PublicBuiltinDispatchContext, PublicRealmBuiltins, RealmBuiltins,
    builtin_metadata, dispatch_builtin, public_builtin_metadata,
};
pub use registry::{
    BuiltinEntryMetadata, BuiltinRegistry, BuiltinRegistryEntry, BuiltinRegistryError,
};
