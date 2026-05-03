//! Interpreter and runtime-installation scaffolding for the lyng-js Phase 4 VM layer.
//!
//! Ownership: `lyng_js_vm` owns runtime installation, frame records, register-window
//! bookkeeping, and bytecode execution entrypoints. It does not own lowering, object
//! semantics, or environment semantics that belong in `lyng_js_compiler`, `lyng_js_ops`,
//! `lyng_js_objects`, or `lyng_js_env`.

#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use
)]

mod activation;
mod enumeration;
mod error;
mod extensions;
mod frame;
mod installed;
mod marker;
mod name_refs;
mod vm;

#[cfg(test)]
mod tests;

pub use error::{ModuleLoadError, VmError};
pub use extensions::{
    EmbeddingFunctionContext, EmbeddingFunctionMetadata, EmbeddingInvocation,
    RealmExtensionInstallation, RealmExtensionProvider, SharedRealmExtensionProvider,
};
pub use frame::{seed_registers, FrameFlags, FrameRecord, RegisterWindow};
pub use installed::InstalledCode;
pub use marker::VmMarker;
pub use vm::{FeedbackVectorFootprint, LoadedModuleRoot, TierStatus, TieringSnapshot, Vm};
