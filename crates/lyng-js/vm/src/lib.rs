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

// `extern crate self as ...` lets the proc-macro lowerer in
// `lyng-js-vm-dsl::lower` emit absolute paths like
// `::lyng_js_vm::dsl::reg_convention::...` that resolve correctly even
// when the macro is invoked from inside `lyng_js_vm` itself. Without
// this, the path can only be found from external test crates that
// have `lyng-js-vm` as a Cargo dep — the proc-macro can't tell the
// difference at lower time.
extern crate self as lyng_js_vm;

mod activation;
pub mod dsl;
mod enumeration;
mod error;
mod extensions;
mod frame;
mod installed;
mod name_refs;
#[cfg(feature = "opcode-counters")]
mod opcode_counts;
#[cfg(feature = "opcode-counters")]
mod slow_path_counts;
pub(crate) mod vm;

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
#[cfg(feature = "opcode-counters")]
pub use opcode_counts::{
    CallArgumentCopyCounts, OpcodeDispatchCount, OpcodeDispatchCounts,
};
#[cfg(feature = "opcode-counters")]
pub use slow_path_counts::{SlowPathCounterStore, SlowPathCounts};
pub use vm::{
    CallCacheEntrySnapshot, CallFeedbackSnapshot, ConstructCacheEntrySnapshot,
    ConstructFeedbackSnapshot, FeedbackInlineCacheState, FeedbackKeyedPropertyFamily,
    FeedbackSiteDetail, FeedbackSiteSnapshot, FeedbackVectorFootprint, FeedbackVectorSnapshot,
    KeyedNamedPropertyCacheEntrySnapshot, KeyedPropertyFeedbackSnapshot, LoadedModuleRoot,
    NamedPropertyCacheEntrySnapshot, NamedPropertyFeedbackSnapshot, TierStatus, TieringSnapshot,
    Vm, VmDebugCommand, VmDebugFrame, VmDebugHook, VmDebugPauseContext, VmDebugPauseReason,
    VmDebugSafepoint, VmDebugSafepointKind, VmDebugStepMode, VmEvaluationObserver,
};
