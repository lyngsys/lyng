//! Runtime-wide environment substrate for lyng-js Phase 3.
//!
//! Ownership: `lyng_js_env` owns the runtime topology above primitive values:
//! runtime, agents, cluster-owned shared coordination, job queues, and the
//! later realm and execution-context owners layered on top of that topology.

#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use
)]

use lyng_js_common::AtomTable;
use lyng_js_gc::{
    AllocationLifetime, EnvironmentSlotsRef, ObjectHandleStoreTarget, PrimitiveHeap,
    PrimitiveRoots, RuntimeEnvironmentRecord, RuntimeRealmRecord, ValueStoreTarget,
};
use lyng_js_host::{
    HostAgentId, HostHooks, HostJobKind, HostJobPhase, HostResult, HostThreadId, JobObservation,
    WaitLocation,
};
use lyng_js_objects::ObjectRuntime;
use lyng_js_types::{CodeRef, EnvironmentRef, RealmRef};

mod accounting;
mod agent;
mod backing_store;
mod cluster;
mod disposal;
mod environment_layouts;
mod environment_records;
mod execution;
mod ids;
mod jobs;
mod module_records;
mod promise;
mod runtime;
mod shared_memory;
mod symbols;

pub use accounting::{AgentPhase6Accounting, RuntimeDomainAccounting, RuntimePhase6Accounting};
pub use agent::Agent;
pub use cluster::AgentCluster;
pub use disposal::{
    AsyncDisposalOperationRecord, AsyncDisposalResumeRecord, DisposableResourceKind,
    DisposableResourceRecord, DisposalCapabilityKind, DisposalCapabilityRecord,
    DisposalCapabilityState, DisposalMethodKind,
};
pub use environment_layouts::{
    EnvironmentBindingLayout, EnvironmentLayout, EnvironmentLayoutId, EnvironmentLayoutKind,
    EnvironmentSlotFlags, ThisBindingStatus,
};
pub use environment_records::{
    DeclarativeEnvironmentRecord, EnvironmentRecord, FunctionEnvironmentRecord,
    GlobalEnvironmentRecord, GlobalLexicalBindingRecord, ModuleBindingAlias,
    ModuleEnvironmentRecord, ObjectEnvironmentRecord, PrivateEnvironmentRecord,
};
pub use execution::{
    ExecutableId, ExecutionContext, ExecutionContextKind, Intrinsics, RealmBootstrapState,
    RealmRecord, RegExpLegacyStaticState, ThisState,
};
pub use ids::{AgentId, JobId};
pub use ids::{
    AsyncDisposalOperationId, AsyncDisposalResumeId, DisposalCapabilityId, PromiseCapabilityId,
    PromiseCombinatorElementId, PromiseCombinatorId, PromiseFinallyFunctionId, PromiseId,
    PromiseReactionId, PromiseResolvingFunctionId,
};
pub use jobs::{JobQueueKind, RuntimeJob, RuntimeJobPayload};
pub use module_records::{
    ModuleImportEntry, ModuleImportKind, ModuleIndirectExportEntry, ModuleLocalExportEntry,
    ModuleRecord, ModuleRequestRecord, ModuleResolvedExport, ModuleResolvedExportTarget,
    ModuleStarExportEntry, ModuleStatus,
};
pub use promise::{
    PromiseCapabilityRecord, PromiseCombinatorElementKind, PromiseCombinatorElementRecord,
    PromiseCombinatorKind, PromiseCombinatorRecord, PromiseFinallyFunctionKind,
    PromiseFinallyFunctionRecord, PromiseReactionHandler, PromiseReactionKind,
    PromiseReactionRecord, PromiseRecord, PromiseResolvingFunctionKind,
    PromiseResolvingFunctionRecord, PromiseState,
};
pub use runtime::{Runtime, RuntimeSubstrateMarker};
pub use shared_memory::{
    AsyncWaiterRecord, ParkedAgentRecord, SharedBackingStoreRecord, WaiterKind, WaiterRecord,
    WaiterToken,
};
pub use symbols::{
    BootstrapAtoms, GlobalSymbolRegistry, GlobalSymbolRegistryEntry, WellKnownSymbols,
};

use disposal::AgentDisposalTables;
use environment_records::EnvironmentMetadata;
use execution::RealmMetadata;
use ids::{agent_index, environment_index, layout_index, realm_index};
use jobs::AgentJobQueues;
use promise::AgentPromiseTables;

pub(crate) use accounting::{merge_primitive_heap_accounting, total_live_bytes};
pub(crate) use agent::{ClusterBackingStoreHandle, ClusterSharedMemoryHandle};
pub(crate) use backing_store::BackingStoreRuntime;
pub(crate) use shared_memory::SharedMemoryRuntime;

#[cfg(test)]
mod tests;
