//! Interpreter and runtime installation for the lyng-js VM layer.
//!
//! Ownership: `lyng_js_vm` owns runtime installation, frame records, register-window
//! bookkeeping, and bytecode execution entrypoints. It does not own lowering, object
//! semantics, or environment semantics that belong in `lyng_js_compiler`, `lyng_js_ops`,
//! `lyng_js_objects`, or `lyng_js_env`.

#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    reason = "VM public API keeps execution-domain names and lightweight snapshot accessors explicit for embedders"
)]
#![cfg_attr(
    feature = "trampoline-dispatch",
    allow(
        dead_code,
        reason = "lyng-33i2 cutover window: the legacy run_dispatch_loop, DispatchFrameSnapshot, and decode_* helpers stay compiled (default path) but go unused on the trampoline-dispatch feature build until sub-8 (lyng-9gyk) deletes them."
    )
)]

mod activation;
mod enumeration;
mod error;
mod extensions;
mod frame;
mod installed;
mod name_refs;
mod opcode_counts;
mod vm;

#[cfg(test)]
mod tests;

pub use error::{ModuleLoadError, VmError};
pub use extensions::{
    EmbeddingFunctionContext, EmbeddingFunctionMetadata, EmbeddingInvocation,
    RealmExtensionInstallation, RealmExtensionProvider, SharedRealmExtensionProvider,
};
pub use frame::{
    seed_registers, FrameFlags, FrameMetadata, FrameRecord, FrameState, RegisterWindow,
};
pub use installed::InstalledCode;
pub use opcode_counts::{OpcodeDispatchCount, OpcodeDispatchCounts};
pub use vm::{
    CallCacheEntrySnapshot, CallFeedbackSnapshot, ConstructCacheEntrySnapshot,
    ConstructFeedbackSnapshot, FeedbackInlineCacheState, FeedbackKeyedPropertyFamily,
    FeedbackSiteDetail, FeedbackSiteSnapshot, FeedbackVectorFootprint, FeedbackVectorSnapshot,
    KeyedNamedPropertyCacheEntrySnapshot, KeyedPropertyFeedbackSnapshot, LoadedModuleRoot,
    NamedPropertyCacheEntrySnapshot, NamedPropertyFeedbackSnapshot, TierStatus, TieringSnapshot,
    Vm, VmDebugCommand, VmDebugFrame, VmDebugHook, VmDebugPauseContext, VmDebugPauseReason,
    VmDebugSafepoint, VmDebugSafepointKind, VmDebugStepMode, VmEvaluationObserver,
};
