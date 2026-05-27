use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use lyng_builtins::{
    bootstrap_realm, BootstrapArtifacts, BootstrapMode, BootstrapRequest, BuiltinCache,
};
use lyng_bytecode::{
    ArgumentsMode, BytecodeEnvironmentBinding, BytecodeFunction, BytecodeFunctionId, CallRange,
    CompiledAtom, CompiledFunctionUnit, CompiledScriptUnit, ConstantValue, DeoptSnapshot,
    GlobalScriptInstantiationPlan, Opcode, SafepointDescriptor, SourceMapEntry,
};
use lyng_common::{AtomId, SourceId, WellKnownAtom};
use lyng_compiler::dynamic::DynamicFunctionCacheKey;
use lyng_env::{
    Agent, EnvironmentBindingLayout, EnvironmentLayout, EnvironmentLayoutId, EnvironmentLayoutKind,
    EnvironmentSlotFlags, ExecutionContext, ModuleRecord, ModuleStatus, RealmRecord,
    ThisBindingStatus, ThisState,
};
use lyng_gc::{AllocationLifetime, PrimitiveCollectionReport};
use lyng_host::{HostHooks, ModuleKey, NoopHostHooks};
use lyng_objects::{AdaptiveProtoLoadDispatch, NativeFunctionRegistry, ObjectAllocation};
use lyng_types::{
    AbruptCompletion, BuiltinFunctionId, CodeRef, EnvironmentRef, FeedbackSlotId, ObjectRef,
    RealmRef, Value, WellKnownSymbolId,
};

use crate::activation::ActivationSideTables;
use crate::enumeration::{ForInStateTable, IteratorStateTable};
use crate::error::VmResult;
use crate::extensions::{RealmExtensionInstallation, SharedRealmExtensionProvider};
use crate::name_refs::CapturedNameReferenceTable;
#[cfg(feature = "opcode-counters")]
use crate::opcode_counts::OpcodeCounters;
use crate::{FrameFlags, FrameRecord, InstalledCode, RegisterWindow, VmError};

mod activation_objects;
mod async_functions;
mod builtin_dispatch;
mod bytecode_calls;
mod call;
mod debugger;
mod direct_eval_env;
pub mod dispatch;
pub mod dispatch_state;
mod dynamic_compilation;
mod exceptions;
mod feedback;
mod generators;
mod global_script;
pub(crate) mod ic_state;
pub mod install;
mod internal_calls;
mod jobs;
mod loop_iteration;
pub(crate) mod metadata_table;
mod modules;
mod names;
mod property_access;
mod registers;
mod runtime_objects;
pub mod semantics;
mod state;
mod tiering;
mod values;
mod with_env;

use call::RejectingNativeRegistry;
use feedback::{FeedbackVector, PolymorphicChain};
use ic_state::{CallIcState, KeyedPropertyIcState, PropertyIcState};
use install::InstalledFunction;
use metadata_table::MetadataTable;
use state::{
    ActiveEnvScopeRange, ActiveVmRoots, AsyncFrameState, AsyncGeneratorFrameState,
    AsyncGeneratorRequest, DirectEvalEnvironmentState, DynamicImportPhase, DynamicImportRequest,
    EntryExecutionOverride, LoopIterationEnvironment, PendingDynamicImport,
    SuspendedExecutionSideState, TemplateCacheKey, WithEnvironmentState,
};
use values::{bytecode_index, code_index, decode_env_operand, string_text_array_index};
// Re-export `code_index` for the DSL-0b entry shim so it can resolve
// the `feedback_flat_storage` slot for a frame's `CodeRef` without
// re-implementing the (id - 1) → usize indexing.
pub use values::code_index as code_index_for_dsl;

pub use modules::LoadedModuleRoot;

pub use debugger::{
    VmDebugCommand, VmDebugFrame, VmDebugHook, VmDebugPauseContext, VmDebugPauseReason,
    VmDebugSafepoint, VmDebugSafepointKind, VmDebugStepMode, VmDebugger,
};
pub use feedback::{
    CallCacheEntrySnapshot, CallFeedbackSnapshot, ConstructCacheEntrySnapshot,
    ConstructFeedbackSnapshot, FeedbackInlineCacheState, FeedbackKeyedPropertyFamily,
    FeedbackSiteDetail, FeedbackSiteSnapshot, FeedbackVectorSnapshot,
    KeyedNamedPropertyCacheEntrySnapshot, KeyedPropertyFeedbackSnapshot,
    NamedPropertyCacheEntrySnapshot, NamedPropertyFeedbackSnapshot,
};
pub use tiering::{TierStatus, Tiering, TieringSnapshot};

/// Observer for coarse VM evaluation phases around one installed entry execution.
///
/// The observer is intentionally timing-agnostic. Embedders that need diagnostics can
/// record wall-clock data at the phase boundaries without making the VM depend on a
/// clock source.
pub trait VmEvaluationObserver {
    fn before_bytecode_execution(&mut self) {}
    fn after_bytecode_execution(&mut self) {}
    fn before_job_checkpoint(&mut self) {}
    fn after_job_checkpoint(&mut self) {}
}

struct NoopVmEvaluationObserver;

impl VmEvaluationObserver for NoopVmEvaluationObserver {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FeedbackVectorFootprint {
    allocated: bool,
    slot_count: usize,
    live_site_count: usize,
    allocated_bytes: usize,
    warmup_counter: u16,
}

impl FeedbackVectorFootprint {
    #[inline]
    pub const fn allocated(self) -> bool {
        self.allocated
    }

    #[inline]
    pub const fn slot_count(self) -> usize {
        self.slot_count
    }

    #[inline]
    pub const fn live_site_count(self) -> usize {
        self.live_site_count
    }

    #[inline]
    pub const fn allocated_bytes(self) -> usize {
        self.allocated_bytes
    }

    #[inline]
    pub const fn warmup_counter(self) -> u16 {
        self.warmup_counter
    }
}

#[derive(Default)]
pub struct Vm {
    register_stack: Vec<Value>,
    register_stack_top: usize,
    frames: Vec<FrameRecord>,
    dispatch_frame_check_epoch: u32,
    installed: Vec<Option<Arc<InstalledFunction>>>,
    current_exception: Option<Value>,
    #[cfg(feature = "opcode-counters")]
    pub(crate) counters: OpcodeCounters,
    debugger: VmDebugger,
    atom_texts: HashMap<AtomId, Box<str>>,
    preferred_atoms_by_text: HashMap<Box<str>, AtomId>,
    source_texts: HashMap<SourceId, Arc<str>>,
    /// Per-installed-code feedback storage, keyed by `code_index(code_ref)`. Every entry is a
    /// real `FeedbackVector` rather than `Option<FeedbackVector>` — the default-constructed
    /// value is the "unallocated" sentinel (empty slot storage), so IC-bearing opcodes drop
    /// one Option discriminant on the hot path. The warmup counter lives on `Tiering`
    /// (see `TieringState::warmup_counter`); Spec 2 Phase A lifted it off `FeedbackVector`.
    feedback_vectors: Vec<FeedbackVector>,
    /// Spec 2 Phase B: out-of-line polymorphic IC entries (indices POLY_LIMIT..8).
    /// Keyed by (CodeRef, FeedbackSlotId). Lazy: monomorphic and ≤POLY_LIMIT
    /// polymorphic slots have no entry. Cleared on AdaptiveProtoLoad fire and
    /// on code GC (via prune_dead_code_polymorphic_chains).
    polymorphic_chains: HashMap<(CodeRef, FeedbackSlotId), PolymorphicChain>,
    /// Phase D.1.1: Rust-only IC state machine for `NamedProperty` slots.
    /// Keyed by `(CodeRef, FeedbackSlotId)`. Lazy: created on first slow-path
    /// install. Entries are pruned on code GC via
    /// `prune_dead_code_property_ic_states`. The asm-readable bits (`mode`,
    /// `generation`, `handler_bits`, `aux_bits`, `execution_count`) live on
    /// `PropertyMetadata` inside `MetadataTable`; this map holds the remaining
    /// Rust-only state-machine fields.
    pub(crate) property_ic_states: HashMap<(CodeRef, FeedbackSlotId), PropertyIcState>,
    /// Phase D.1.2: Rust-only IC state machine for `Call` slots.
    /// Keyed by `(CodeRef, FeedbackSlotId)`. Lazy: created on first slow-path
    /// observation. Entries are pruned on code GC via
    /// `prune_dead_code_call_ic_states`. The asm-readable bits (`mode`,
    /// `generation`, `callee_bits`, `execution_count`) live on `CallMetadata`
    /// inside `MetadataTable`; this map holds the Rust-only state.
    pub(crate) call_ic_states: HashMap<(CodeRef, FeedbackSlotId), CallIcState>,
    /// Phase D.1.2: Rust-only IC state machine for `Construct` slots.
    /// Same shape as `call_ic_states`; the kind distinction is implicit in
    /// which map the entry lives in.
    pub(crate) construct_ic_states: HashMap<(CodeRef, FeedbackSlotId), CallIcState>,
    /// Phase D.1.3: Rust-only IC state machine for `KeyedProperty` slots.
    /// Keyed by `(CodeRef, FeedbackSlotId)`. Lazy: created on first slow-path
    /// observation. Entries are pruned on code GC via
    /// `prune_dead_code_keyed_property_ic_states`. The asm-readable bits (`mode`,
    /// `generation`, `handler_bits`, `execution_count`) live on
    /// `KeyedPropertyMetadata` inside `MetadataTable`; this map holds the
    /// Rust-only state (family, entries, sidecars).
    pub(crate) keyed_property_ic_states: HashMap<(CodeRef, FeedbackSlotId), KeyedPropertyIcState>,
    /// Legacy scalar feedback mirror. Phase C.4 status: the asm IC fast path
    /// no longer reads OR writes this storage — both `load_feedback_site!` and
    /// `record_*` macros now source x21 from `Vm::metadata_tables`. This field
    /// survives solely to feed `mirror_flat_slot`, which the Phase C.3 debug
    /// equivalence assertion still compares against. Phase D deletes it
    /// entirely along with `FeedbackEntry` and `mirror_flat_slot`.
    pub(crate) feedback_flat_storage: Vec<Box<[crate::dsl::feedback_flat::FeedbackEntry]>>,
    /// Phase C: per-code-object IC metadata buffer, parallel to
    /// `feedback_flat_storage`, keyed by `code_index(code_ref)`.
    /// `None` for code that has not yet been installed (or was installed
    /// before Phase C landed). Allocated eagerly alongside the flat
    /// storage in `store_installed`; never grown thereafter.
    pub(crate) metadata_tables: Vec<Option<MetadataTable>>,
    /// Safepoint poll-pending byte read by `poll_safepoint!` (warm
    /// `op_loop_header` / backward jumps). The asm reads
    /// `[x22, VM_POLL_PENDING_OFFSET]` where `x22 = *mut Vm`; the offset
    /// is derived from `offset_of!(Vm, dsl_poll_pending)` in
    /// `crate::dsl::reg_convention`. Non-zero means a same-thread
    /// incremental-mark step or debugger pause is pending.
    pub(crate) dsl_poll_pending: u8,
    pub(crate) tiering: Tiering,
    activation_tables: ActivationSideTables,
    for_in_states: ForInStateTable,
    iterator_states: IteratorStateTable,
    captured_name_references: CapturedNameReferenceTable,
    builtin_cache: BuiltinCache,
    template_cache: HashMap<TemplateCacheKey, ObjectRef>,
    dynamic_function_cache: HashMap<DynamicFunctionCacheKey, InstalledCode>,
    suspended_side_states: HashMap<lyng_types::SuspendedExecutionRef, SuspendedExecutionSideState>,
    async_frame_states: HashMap<u32, AsyncFrameState>,
    async_generator_objects: HashSet<ObjectRef>,
    async_generator_frame_states: HashMap<u32, AsyncGeneratorFrameState>,
    async_generator_queues: HashMap<ObjectRef, VecDeque<AsyncGeneratorRequest>>,
    dynamic_import_requests: Vec<Option<DynamicImportRequest>>,
    dynamic_import_evaluate_depth: u32,
    dynamic_import_waiting_modules: HashMap<ModuleKey, Vec<PendingDynamicImport>>,
    deferred_module_namespaces: HashMap<ObjectRef, ModuleKey>,
    async_body_suspended_modules: HashSet<ModuleKey>,
    async_dependency_blocked_modules: HashSet<ModuleKey>,
    async_dependency_blocked_queue: VecDeque<ModuleKey>,
    async_dependency_completed_modules: HashSet<ModuleKey>,
    next_dynamic_source_raw: u32,
    loop_iteration_envs: Vec<LoopIterationEnvironment>,
    with_environment_states: Vec<WithEnvironmentState>,
    direct_eval_environment_states: Vec<DirectEvalEnvironmentState>,
    active_env_scopes: Vec<ActiveEnvScopeRange>,
    direct_eval_environment_overlays: HashMap<EnvironmentRef, EnvironmentRef>,
    direct_eval_lexical_layouts: HashMap<Vec<BytecodeEnvironmentBinding>, EnvironmentLayoutId>,
    loop_iteration_layouts: HashMap<Option<EnvironmentLayoutId>, EnvironmentLayoutId>,
    loop_iteration_source_scratch: Vec<EnvironmentRef>,
    loop_iteration_target_scratch: Vec<EnvironmentRef>,
    class_private_env_layout: Option<EnvironmentLayoutId>,
    internal_completion_targets: Vec<usize>,
    generator_resume_depth: usize,
    argument_scratch: Vec<Value>,
    string_code_units_scratch: Vec<u16>,
    active_extension_provider: Option<SharedRealmExtensionProvider>,
    #[cfg(test)]
    peak_frame_depth: usize,
}

/// Scoped builder for evaluating a `CompiledScriptUnit`. Holds borrows of the VM, agent,
/// and required inputs; consumed by `.run()` or `.run_retaining_installed()`.
#[must_use = "call .run() to execute the script, or .run_retaining_installed() to also keep the InstalledCode"]
pub struct EvaluateScript<'b> {
    vm: &'b mut Vm,
    agent: &'b mut Agent,
    realm: RealmRecord,
    unit: &'b CompiledScriptUnit,
    host: Option<&'b dyn HostHooks>,
    registry: Option<&'b mut dyn NativeFunctionRegistry>,
    referrer: Option<&'b ModuleKey>,
    extensions: Option<&'b SharedRealmExtensionProvider>,
    #[cfg(feature = "opcode-counters")]
    installed_counters: Option<&'b mut OpcodeCounters>,
    installed_debugger: Option<&'b mut VmDebugger>,
    installed_tiering: Option<&'b mut Tiering>,
}

impl<'b> EvaluateScript<'b> {
    pub fn with_host(mut self, host: &'b dyn HostHooks) -> Self {
        self.host = Some(host);
        self
    }

    pub fn with_registry(mut self, registry: &'b mut dyn NativeFunctionRegistry) -> Self {
        self.registry = Some(registry);
        self
    }

    pub fn with_referrer(mut self, key: &'b ModuleKey) -> Self {
        self.referrer = Some(key);
        self
    }

    pub fn with_extensions(mut self, provider: &'b SharedRealmExtensionProvider) -> Self {
        self.extensions = Some(provider);
        self
    }

    /// Redirect opcode-counter writes to an externally-owned
    /// `OpcodeCounters` for the duration of `.run()` /
    /// `.run_retaining_installed()`. The borrow's contents are swapped
    /// with the VM's internal counters at the start of the run, then
    /// swapped back when the run returns — so the caller can read
    /// dispatch / slow-path / call-argument-copy snapshots off their
    /// own `OpcodeCounters` afterwards.
    #[cfg(feature = "opcode-counters")]
    pub fn with_opcode_counters(mut self, counters: &'b mut OpcodeCounters) -> Self {
        self.installed_counters = Some(counters);
        self
    }

    /// Install a caller-owned [`VmDebugger`] for the duration of `.run()` /
    /// `.run_retaining_installed()`. The debugger is swapped into the VM at
    /// run entry and swapped back at run exit, so pause-control mutations
    /// (and step state the hook installed) persist on the caller's struct.
    pub fn with_debugger(mut self, debugger: &'b mut VmDebugger) -> Self {
        self.installed_debugger = Some(debugger);
        self
    }

    /// Install a caller-owned [`Tiering`] for the duration of `.run()` /
    /// `.run_retaining_installed()`. The tiering store is swapped into the
    /// VM at run entry and swapped back at run exit, so per-`CodeRef` tier
    /// state accumulated during the run persists on the caller's struct.
    /// The default `Vm::new()` holds an empty `Tiering`, so without this
    /// hook the VM allocates no per-install tier slots and `observe_*`
    /// calls short-circuit before any `Vec` indexing.
    pub fn with_tiering(mut self, tiering: &'b mut Tiering) -> Self {
        self.installed_tiering = Some(tiering);
        self
    }

    /// # Errors
    /// Returns a VM error if script installation, bootstrap, instantiation, execution, or job
    /// checkpointing fails.
    pub fn run(self) -> VmResult<Value> {
        self.run_retaining_installed().map(|(value, _)| value)
    }

    /// # Errors
    /// Returns a VM error if script installation, bootstrap, instantiation, execution, or job
    /// checkpointing fails.
    #[allow(
        clippy::needless_option_as_deref,
        reason = "as_deref_mut produces a reborrow we need at two distinct call sites; the inner Option<&mut> can't be consumed twice"
    )]
    pub fn run_retaining_installed(self) -> VmResult<(Value, InstalledCode)> {
        let EvaluateScript {
            vm,
            agent,
            realm,
            unit,
            host,
            registry,
            referrer,
            extensions,
            #[cfg(feature = "opcode-counters")]
            mut installed_counters,
            mut installed_debugger,
            mut installed_tiering,
        } = self;
        let host = host.unwrap_or(&NoopHostHooks);
        let mut fallback_registry = RejectingNativeRegistry;
        let registry: &mut dyn NativeFunctionRegistry = match registry {
            Some(r) => r,
            None => &mut fallback_registry,
        };

        #[cfg(feature = "opcode-counters")]
        if let Some(external) = installed_counters.as_deref_mut() {
            std::mem::swap(&mut vm.counters, external);
        }
        if let Some(external) = installed_debugger.as_deref_mut() {
            std::mem::swap(&mut vm.debugger, external);
            vm.refresh_dsl_poll_pending();
        }
        if let Some(external) = installed_tiering.as_deref_mut() {
            std::mem::swap(&mut vm.tiering, external);
        }
        let result = match extensions {
            Some(provider) => vm.with_extension_provider(provider, |vm| {
                Self::run_inner(vm, agent, &realm, unit, referrer, host, registry)
            }),
            None => Self::run_inner(vm, agent, &realm, unit, referrer, host, registry),
        };
        if let Some(external) = installed_tiering.as_deref_mut() {
            std::mem::swap(&mut vm.tiering, external);
        }
        if let Some(external) = installed_debugger.as_deref_mut() {
            std::mem::swap(&mut vm.debugger, external);
            vm.refresh_dsl_poll_pending();
        }
        #[cfg(feature = "opcode-counters")]
        if let Some(external) = installed_counters.as_deref_mut() {
            std::mem::swap(&mut vm.counters, external);
        }
        result
    }

    fn run_inner(
        vm: &mut Vm,
        agent: &mut Agent,
        realm: &RealmRecord,
        unit: &CompiledScriptUnit,
        referrer: Option<&ModuleKey>,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> VmResult<(Value, InstalledCode)> {
        let installed = vm.install_script(agent, realm.id(), unit)?;
        let _ = vm.bootstrap_realm(agent, realm.id(), BootstrapMode::SpecOnly)?;
        vm.install_active_realm_extensions(agent, realm.id())?;
        vm.instantiate_global_script(agent, realm, unit.instantiation_plan())?;
        let referrer_atom = referrer.map(|key| agent.atoms_mut().intern_collectible(key.as_str()));
        let mut observer = NoopVmEvaluationObserver;
        let value = vm.evaluate_entry_with_registry_and_checkpoint(
            agent,
            installed,
            realm.global_env(),
            realm.global_env(),
            referrer_atom,
            host,
            registry,
            Some(unit.instantiation_plan()),
            None,
            &mut observer,
        )?;
        Ok((value, installed))
    }
}

/// Scoped builder for evaluating an already-installed code record. Holds borrows of the VM,
/// agent, and required inputs; consumed by `.run()`.
#[must_use = "call .run() to execute the installed code"]
pub struct EvaluateInstalled<'b> {
    vm: &'b mut Vm,
    agent: &'b mut Agent,
    installed: InstalledCode,
    lexical_env: EnvironmentRef,
    variable_env: EnvironmentRef,
    host: Option<&'b dyn HostHooks>,
    registry: Option<&'b mut dyn NativeFunctionRegistry>,
    referrer: Option<AtomId>,
    observer: Option<&'b mut dyn VmEvaluationObserver>,
    entry_override: Option<EntryExecutionOverride>,
    #[cfg(feature = "opcode-counters")]
    installed_counters: Option<&'b mut OpcodeCounters>,
    installed_debugger: Option<&'b mut VmDebugger>,
    installed_tiering: Option<&'b mut Tiering>,
}

impl<'b> EvaluateInstalled<'b> {
    pub fn with_host(mut self, host: &'b dyn HostHooks) -> Self {
        self.host = Some(host);
        self
    }

    pub fn with_registry(mut self, registry: &'b mut dyn NativeFunctionRegistry) -> Self {
        self.registry = Some(registry);
        self
    }

    pub fn with_referrer(mut self, atom: AtomId) -> Self {
        self.referrer = Some(atom);
        self
    }

    pub fn with_observer(mut self, observer: &'b mut dyn VmEvaluationObserver) -> Self {
        self.observer = Some(observer);
        self
    }

    pub(crate) fn with_entry_override(mut self, override_: EntryExecutionOverride) -> Self {
        self.entry_override = Some(override_);
        self
    }

    /// Redirect opcode-counter writes to an externally-owned
    /// `OpcodeCounters` for the duration of `.run()`. See
    /// [`EvaluateScript::with_opcode_counters`] for the full
    /// description.
    #[cfg(feature = "opcode-counters")]
    pub fn with_opcode_counters(mut self, counters: &'b mut OpcodeCounters) -> Self {
        self.installed_counters = Some(counters);
        self
    }

    /// Install a caller-owned [`VmDebugger`] for the duration of `.run()`.
    /// See [`EvaluateScript::with_debugger`] for the full description.
    pub fn with_debugger(mut self, debugger: &'b mut VmDebugger) -> Self {
        self.installed_debugger = Some(debugger);
        self
    }

    /// Install a caller-owned [`Tiering`] for the duration of `.run()`. See
    /// [`EvaluateScript::with_tiering`] for the full description.
    pub fn with_tiering(mut self, tiering: &'b mut Tiering) -> Self {
        self.installed_tiering = Some(tiering);
        self
    }

    /// # Errors
    /// Returns a VM error if entering the installed function, execution, or job checkpointing fails.
    #[allow(
        clippy::needless_option_as_deref,
        reason = "as_deref_mut produces a reborrow we need at two distinct call sites; the inner Option<&mut> can't be consumed twice"
    )]
    pub fn run(self) -> VmResult<Value> {
        let EvaluateInstalled {
            vm,
            agent,
            installed,
            lexical_env,
            variable_env,
            host,
            registry,
            referrer,
            observer,
            entry_override,
            #[cfg(feature = "opcode-counters")]
            mut installed_counters,
            mut installed_debugger,
            mut installed_tiering,
        } = self;
        let host = host.unwrap_or(&NoopHostHooks);
        let mut fallback_registry = RejectingNativeRegistry;
        let registry: &mut dyn NativeFunctionRegistry = match registry {
            Some(r) => r,
            None => &mut fallback_registry,
        };
        let mut fallback_observer = NoopVmEvaluationObserver;
        let observer: &mut dyn VmEvaluationObserver = match observer {
            Some(o) => o,
            None => &mut fallback_observer,
        };

        #[cfg(feature = "opcode-counters")]
        if let Some(external) = installed_counters.as_deref_mut() {
            std::mem::swap(&mut vm.counters, external);
        }
        if let Some(external) = installed_debugger.as_deref_mut() {
            std::mem::swap(&mut vm.debugger, external);
            vm.refresh_dsl_poll_pending();
        }
        if let Some(external) = installed_tiering.as_deref_mut() {
            std::mem::swap(&mut vm.tiering, external);
        }
        let result = vm.evaluate_entry_with_registry_and_checkpoint(
            agent,
            installed,
            lexical_env,
            variable_env,
            referrer,
            host,
            registry,
            None,
            entry_override,
            observer,
        );
        if let Some(external) = installed_tiering.as_deref_mut() {
            std::mem::swap(&mut vm.tiering, external);
        }
        if let Some(external) = installed_debugger.as_deref_mut() {
            std::mem::swap(&mut vm.debugger, external);
            vm.refresh_dsl_poll_pending();
        }
        #[cfg(feature = "opcode-counters")]
        if let Some(external) = installed_counters.as_deref_mut() {
            std::mem::swap(&mut vm.counters, external);
        }
        result
    }
}

impl Vm {
    #[inline]
    pub fn new() -> Self {
        Self {
            register_stack: Vec::new(),
            register_stack_top: 0,
            frames: Vec::new(),
            dispatch_frame_check_epoch: 0,
            installed: Vec::new(),
            current_exception: None,
            #[cfg(feature = "opcode-counters")]
            counters: OpcodeCounters::new(),
            debugger: VmDebugger::default(),
            atom_texts: HashMap::new(),
            preferred_atoms_by_text: HashMap::new(),
            source_texts: HashMap::new(),
            feedback_vectors: Vec::new(),
            feedback_flat_storage: Vec::new(),
            metadata_tables: Vec::new(),
            polymorphic_chains: HashMap::new(),
            property_ic_states: HashMap::new(),
            call_ic_states: HashMap::new(),
            construct_ic_states: HashMap::new(),
            keyed_property_ic_states: HashMap::new(),
            dsl_poll_pending: 0,
            tiering: Tiering::disabled(),
            activation_tables: ActivationSideTables::default(),
            for_in_states: ForInStateTable::default(),
            iterator_states: IteratorStateTable::default(),
            captured_name_references: CapturedNameReferenceTable::default(),
            builtin_cache: BuiltinCache::new(),
            template_cache: HashMap::new(),
            dynamic_function_cache: HashMap::new(),
            suspended_side_states: HashMap::new(),
            async_frame_states: HashMap::new(),
            async_generator_objects: HashSet::new(),
            async_generator_frame_states: HashMap::new(),
            async_generator_queues: HashMap::new(),
            dynamic_import_requests: Vec::new(),
            dynamic_import_evaluate_depth: 0,
            dynamic_import_waiting_modules: HashMap::new(),
            deferred_module_namespaces: HashMap::new(),
            async_body_suspended_modules: HashSet::new(),
            async_dependency_blocked_modules: HashSet::new(),
            async_dependency_blocked_queue: VecDeque::new(),
            async_dependency_completed_modules: HashSet::new(),
            next_dynamic_source_raw: 1,
            loop_iteration_envs: Vec::new(),
            with_environment_states: Vec::new(),
            direct_eval_environment_states: Vec::new(),
            active_env_scopes: Vec::new(),
            direct_eval_environment_overlays: HashMap::new(),
            direct_eval_lexical_layouts: HashMap::new(),
            loop_iteration_layouts: HashMap::new(),
            loop_iteration_source_scratch: Vec::new(),
            loop_iteration_target_scratch: Vec::new(),
            class_private_env_layout: None,
            internal_completion_targets: Vec::new(),
            generator_resume_depth: 0,
            argument_scratch: Vec::new(),
            string_code_units_scratch: Vec::new(),
            active_extension_provider: None,
            #[cfg(test)]
            peak_frame_depth: 0,
        }
    }

    /// Returns the polymorphic chain for `(code, slot)` if any.
    pub(crate) fn polymorphic_chain(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<&PolymorphicChain> {
        self.polymorphic_chains.get(&(code, slot))
    }

    /// Returns a mutable reference to the polymorphic chain for `(code, slot)`,
    /// lazily creating an empty chain on first access. The slow-path installer
    /// reaches into `self.polymorphic_chains` directly via a split-borrow
    /// alongside `feedback_vectors`; this helper is the documented public
    /// surface for callers that hold an exclusive `&mut Vm` and don't need
    /// to borrow another field at the same time.
    #[allow(
        dead_code,
        reason = "Spec 2 Phase B accessor surface; install path uses split-borrow"
    )]
    pub(crate) fn polymorphic_chain_mut(
        &mut self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> &mut PolymorphicChain {
        self.polymorphic_chains
            .entry((code, slot))
            .or_insert_with(PolymorphicChain::new)
    }

    /// Removes the polymorphic chain for `(code, slot)`. Called when the IC
    /// transitions to Megamorphic or is cleared by an AdaptiveProtoLoad fire.
    pub(crate) fn drop_polymorphic_chain(&mut self, code: CodeRef, slot: FeedbackSlotId) {
        self.polymorphic_chains.remove(&(code, slot));
    }

    /// Phase C: returns the `MetadataTable` for `code`, or `None` if the
    /// code has not been installed yet.
    #[allow(dead_code, reason = "Phase C accessor surface; consumed from Task 1.4")]
    pub fn metadata_table(&self, code: CodeRef) -> Option<&MetadataTable> {
        let idx = code_index(code);
        self.metadata_tables.get(idx).and_then(|t| t.as_ref())
    }

    /// Phase C: returns a mutable reference to the `MetadataTable` for `code`,
    /// or `None` if the code has not been installed yet.
    #[allow(dead_code, reason = "Phase C accessor surface; consumed from Task 2.x")]
    pub(crate) fn metadata_table_mut(&mut self, code: CodeRef) -> Option<&mut MetadataTable> {
        let idx = code_index(code);
        self.metadata_tables.get_mut(idx).and_then(|t| t.as_mut())
    }

    /// Spec 2 Phase B: post-mark GC sweep. Drops polymorphic chain entries
    /// for code that is no longer live. Mirrors
    /// `ObjectRuntime::prune_dead_prototype_transitions` from Spec 1.
    ///
    /// The actual call site uses an inline split-borrow retain in
    /// `force_collect_with_active_roots`; this method is the documented
    /// accessor surface for future callers that already hold `&mut Vm`.
    #[allow(
        dead_code,
        reason = "Spec 2 Phase B sweep surface; call site uses inline split-borrow retain in force_collect_with_active_roots"
    )]
    pub(crate) fn prune_dead_code_polymorphic_chains(&mut self, is_live: impl Fn(CodeRef) -> bool) {
        self.polymorphic_chains
            .retain(|(code, _slot), _chain| is_live(*code));
    }

    /// Phase D.1.1: returns the `PropertyIcState` for `(code, slot)` if any.
    #[allow(
        dead_code,
        reason = "Phase D.1.1 accessor surface; consumed from tests and future D.2.x callers"
    )]
    pub(crate) fn property_ic_state(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<&PropertyIcState> {
        self.property_ic_states.get(&(code, slot))
    }

    /// Phase D.1.1: post-mark GC sweep. Drops `PropertyIcState` entries for
    /// code that is no longer live. Mirrors `prune_dead_code_polymorphic_chains`.
    #[allow(
        dead_code,
        reason = "Phase D.1.1 sweep surface; call site wired alongside prune_dead_code_polymorphic_chains"
    )]
    pub(crate) fn prune_dead_code_property_ic_states(&mut self, is_live: impl Fn(CodeRef) -> bool) {
        self.property_ic_states
            .retain(|(code, _slot), _state| is_live(*code));
    }

    /// Phase D.1.2: returns the `CallIcState` for a `Call` slot `(code, slot)`.
    #[allow(
        dead_code,
        reason = "Phase D.1.2 accessor surface; consumed from tests and future D.2.x callers"
    )]
    pub(crate) fn call_ic_state(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<&CallIcState> {
        self.call_ic_states.get(&(code, slot))
    }

    /// Phase D.1.2: returns the `CallIcState` for a `Construct` slot `(code, slot)`.
    #[allow(
        dead_code,
        reason = "Phase D.1.2 accessor surface; consumed from tests and future D.2.x callers"
    )]
    pub(crate) fn construct_ic_state(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<&CallIcState> {
        self.construct_ic_states.get(&(code, slot))
    }

    /// Phase D.1.2: post-mark GC sweep. Drops `CallIcState` entries (both Call
    /// and Construct maps) for code that is no longer live. Mirrors
    /// `prune_dead_code_property_ic_states`.
    #[allow(
        dead_code,
        reason = "Phase D.1.2 sweep surface; call site wired alongside prune_dead_code_property_ic_states"
    )]
    pub(crate) fn prune_dead_code_call_ic_states(&mut self, is_live: impl Fn(CodeRef) -> bool) {
        self.call_ic_states
            .retain(|(code, _slot), _state| is_live(*code));
        self.construct_ic_states
            .retain(|(code, _slot), _state| is_live(*code));
    }

    /// Phase D.1.3: returns the `KeyedPropertyIcState` for `(code, slot)` if any.
    #[allow(
        dead_code,
        reason = "Phase D.1.3 accessor surface; consumed from tests and future D.2.x callers"
    )]
    pub(crate) fn keyed_property_ic_state(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<&KeyedPropertyIcState> {
        self.keyed_property_ic_states.get(&(code, slot))
    }

    /// Phase D.1.3: post-mark GC sweep. Drops `KeyedPropertyIcState` entries for
    /// code that is no longer live. Mirrors `prune_dead_code_call_ic_states`.
    #[allow(
        dead_code,
        reason = "Phase D.1.3 sweep surface; call site wired alongside prune_dead_code_call_ic_states"
    )]
    pub(crate) fn prune_dead_code_keyed_property_ic_states(
        &mut self,
        is_live: impl Fn(CodeRef) -> bool,
    ) {
        self.keyed_property_ic_states
            .retain(|(code, _slot), _state| is_live(*code));
    }

    /// Phase C Task 4.5: post-mark GC sweep. Drops `MetadataTable` entries
    /// for code objects that are no longer live. Mirrors
    /// `prune_dead_code_polymorphic_chains` (Phase B) for the metadata_tables
    /// vec. The vec is indexed by `code_index(code_ref) = code_ref.get() - 1`,
    /// so index `i` corresponds to `CodeRef::from_raw(i as u32 + 1)`.
    #[allow(
        dead_code,
        reason = "Phase C Task 4.5 sweep surface; called from tests and GC sweep site"
    )]
    pub(crate) fn prune_dead_code_metadata_tables(&mut self, is_live: impl Fn(CodeRef) -> bool) {
        for (index, slot) in self.metadata_tables.iter_mut().enumerate() {
            if slot.is_none() {
                continue;
            }
            let raw = u32::try_from(index + 1).expect("metadata_tables index should fit u32");
            if let Some(code) = CodeRef::from_raw(raw)
                && !is_live(code)
            {
                *slot = None;
            }
        }
    }

    /// Access the VM's opcode instrumentation. Counters are always
    /// allocated when the feature is on; callers reset/snapshot via the
    /// returned `&OpcodeCounters`. To redirect counter writes to an
    /// externally-owned store for a single evaluation, use
    /// `EvaluateScript::with_opcode_counters` /
    /// `EvaluateInstalled::with_opcode_counters` on the builder.
    #[cfg(feature = "opcode-counters")]
    #[inline]
    pub const fn opcode_counters(&self) -> &OpcodeCounters {
        &self.counters
    }

    #[cfg(feature = "opcode-counters")]
    #[inline]
    pub const fn opcode_counters_mut(&mut self) -> &mut OpcodeCounters {
        &mut self.counters
    }

    /// Records `count` argument values pushed into `argument_scratch`. No-op
    /// when the counter is disabled (the default in production builds and
    /// when the `opcode-counters` feature is off). Inlined so the disabled
    /// case compiles to a single load+branch.
    #[cfg(feature = "opcode-counters")]
    #[inline]
    pub(in crate::vm) fn record_argument_scratch_pushes(&self, count: u64) {
        self.counters.record_argument_scratch_pushes(count);
    }

    #[cfg(not(feature = "opcode-counters"))]
    #[inline]
    pub(in crate::vm) fn record_argument_scratch_pushes(&self, _count: u64) {}

    /// Records `count` argument values copied into a callee bytecode frame.
    /// Symmetric with `record_argument_scratch_pushes` — together they let
    /// tests verify that ordinary calls copy each argument exactly once
    /// (`frame_copies` == n, `scratch_pushes` == 0) instead of twice.
    #[cfg(feature = "opcode-counters")]
    #[inline]
    pub(in crate::vm) fn record_argument_frame_copies(&self, count: u64) {
        self.counters.record_argument_frame_copies(count);
    }

    #[cfg(not(feature = "opcode-counters"))]
    #[inline]
    pub(in crate::vm) fn record_argument_frame_copies(&self, _count: u64) {}

    /// Sync `dsl_poll_pending` with the swapped-in debugger only. The
    /// builder calls this after each `mem::swap` of [`VmDebugger`]; GC
    /// pending work is folded in separately at DSL entry / slow-path
    /// egress via [`Self::refresh_dsl_poll_pending_for_agent`].
    #[inline]
    pub(crate) fn refresh_dsl_poll_pending(&mut self) {
        self.dsl_poll_pending = u8::from(self.debug_poll_enabled());
    }

    #[inline]
    pub(crate) fn refresh_dsl_poll_pending_for_agent(&mut self, agent: &Agent) {
        self.dsl_poll_pending = u8::from(self.dsl_poll_pending_for_agent(agent));
    }

    #[inline]
    pub(crate) fn dsl_poll_pending_for_agent(&self, agent: &Agent) -> bool {
        self.debug_poll_enabled()
            || agent
                .heap()
                .active_incremental_mark_pending_work_items()
                .is_some_and(|pending| pending > 0)
    }

    #[inline]
    pub(crate) const fn debug_poll_enabled(&self) -> bool {
        self.debugger.poll_enabled()
    }

    #[inline]
    pub(crate) fn poll_debug_safepoint(&mut self, agent: &Agent, kind: VmDebugSafepointKind) {
        if !self.debug_poll_enabled() {
            return;
        }
        let Some(frame) = self.frame() else {
            return;
        };
        let safepoint = VmDebugSafepoint::new(kind, &frame, self.frames.len());
        let Some(reason) = self.debugger.consume_pause(safepoint) else {
            return;
        };
        let mut hook = self
            .debugger
            .take_hook()
            .expect("debug polling requires an installed hook");
        let command = hook.on_pause(VmDebugPauseContext::new(self, agent, safepoint, reason));
        self.debugger.restore_hook(hook);
        self.debugger
            .apply_command(command, safepoint.frame_depth());
        self.refresh_dsl_poll_pending();
    }

    #[inline]
    pub fn register_stack(&self) -> &[Value] {
        &self.register_stack[..self.register_stack_top]
    }

    #[inline]
    pub fn frames(&self) -> &[FrameRecord] {
        &self.frames
    }

    #[inline]
    pub fn frame(&self) -> Option<FrameRecord> {
        self.frames.last().copied()
    }

    #[inline]
    pub(super) const fn register_stack_top(&self) -> usize {
        self.register_stack_top
    }

    #[inline]
    pub(super) fn release_register_stack_to(&mut self, top: usize) {
        debug_assert!(
            top <= self.register_stack_top,
            "register stack cursor should only move back during cleanup"
        );
        debug_assert!(
            top <= self.register_stack.len(),
            "register stack cursor should stay inside backing storage"
        );
        self.register_stack_top = top;
    }

    #[inline]
    pub(super) fn release_register_window(&mut self, register_base: u32) {
        let Ok(top) = usize::try_from(register_base) else {
            debug_assert!(false, "register stack base should fit into usize");
            return;
        };
        self.release_register_stack_to(top);
    }

    #[cfg(test)]
    #[inline]
    pub(crate) const fn register_stack_storage_len_for_tests(&self) -> usize {
        self.register_stack.len()
    }

    /// DSL-0c: raw mutable pointer to the start of the register-stack
    /// storage, used by [`crate::dsl::entry::run_via_dsl`] to compute
    /// the active frame's `REGS` pin (`*mut Value` at
    /// `register_stack.as_mut_ptr().add(frame.registers().base())`).
    ///
    /// Callers must respect Rust's aliasing rules — the returned
    /// pointer aliases the `Vec`'s backing buffer; concurrent
    /// reborrows of `&mut self.register_stack` would be UB. The
    /// trampoline's contract is that the pointer is only used while
    /// `run_via_dsl` holds `&mut Vm`, and the buffer is not grown
    /// during a single trampoline invocation (window reservation
    /// happens before entry, release happens after return).
    #[inline]
    pub(crate) const fn register_stack_storage_mut_ptr(&mut self) -> *mut Value {
        self.register_stack.as_mut_ptr()
    }

    /// DSL-0c: crate-visible accessor for the dispatch frame-check
    /// epoch used by [`crate::dsl::entry::run_via_dsl`] when seeding
    /// the entry `DispatchState`. The α path reads the same value
    /// through `Vm::dispatch_frame_check_epoch` which is
    /// `pub(in crate::vm)`-scoped.
    #[inline]
    pub(crate) const fn dispatch_frame_check_epoch_for_dsl(&self) -> u32 {
        self.dispatch_frame_check_epoch
    }

    #[cfg(test)]
    pub(crate) const fn string_code_units_scratch_capacity(&self) -> usize {
        self.string_code_units_scratch.capacity()
    }

    #[cfg(test)]
    pub(crate) const fn loop_iteration_scratch_state_for_tests(
        &self,
    ) -> (usize, usize, usize, usize) {
        (
            self.loop_iteration_source_scratch.len(),
            self.loop_iteration_target_scratch.len(),
            self.loop_iteration_source_scratch.capacity(),
            self.loop_iteration_target_scratch.capacity(),
        )
    }

    pub(super) fn class_private_environment_layout(
        &mut self,
        agent: &mut Agent,
    ) -> EnvironmentLayoutId {
        if let Some(layout) = self.class_private_env_layout {
            return layout;
        }
        let layout = agent.alloc_environment_layout(EnvironmentLayout::new(
            EnvironmentLayoutKind::Private,
            [
                EnvironmentBindingLayout::new(None, EnvironmentSlotFlags::mutable_lexical()),
                EnvironmentBindingLayout::new(None, EnvironmentSlotFlags::mutable_lexical()),
            ],
            true,
        ));
        self.class_private_env_layout = Some(layout);
        layout
    }

    #[inline]
    fn reserve_register_window(&mut self, register_base: u32, register_len: u16) {
        let Ok(start) = usize::try_from(register_base) else {
            debug_assert!(false, "register stack base should fit into usize");
            return;
        };
        debug_assert_eq!(self.register_stack_top, start);
        let Some(end) = start.checked_add(usize::from(register_len)) else {
            debug_assert!(false, "register window end should fit into usize");
            return;
        };
        // `release_register_stack_to` only moves the cursor; it does not
        // truncate the Vec. So `register_stack.len()` can sit anywhere in
        // `[start..]` with stale values from past frames in `[start..len)`.
        // Reset that range to `undefined` before extending so a re-entered
        // window starts clean — callers (especially the direct call path)
        // may not rewrite every slot.
        if self.register_stack.len() > start {
            let reset_end = end.min(self.register_stack.len());
            self.register_stack[start..reset_end].fill(Value::undefined());
        }
        if self.register_stack.len() < end {
            self.register_stack.resize(end, Value::undefined());
        }
        self.register_stack_top = end;
    }

    #[inline]
    pub const fn current_exception(&self) -> Option<Value> {
        self.current_exception
    }

    pub(crate) fn force_collect_with_active_roots(
        &mut self,
        agent: &mut Agent,
        caller_frame: FrameRecord,
    ) -> PrimitiveCollectionReport {
        let report = agent.force_collect_with_additional_roots(&ActiveVmRoots {
            vm: self,
            caller_frame: &caller_frame,
        });
        // Spec 2 Phase B (B.2.2): prune polymorphic chain entries for code
        // that is no longer installed. Mirrors the post-mark sweep in
        // Agent::force_collect_with_additional_roots for ObjectRuntime's
        // prototype-transition table, but lives here because Vm owns
        // polymorphic_chains.
        //
        // Liveness predicate: a CodeRef is live iff its slot in
        // `self.installed` is `Some(Some(_))` — i.e., it was installed and
        // has not been evicted by dynamic_function_cache cleanup or otherwise
        // uninstalled.
        let installed = &self.installed;
        self.polymorphic_chains.retain(|(code, _), _| {
            installed
                .get(code_index(*code))
                .is_some_and(|s| s.is_some())
        });
        // Phase D.1.1: prune PropertyIcState side-table entries for code that
        // is no longer installed, mirroring the polymorphic_chains sweep above.
        self.property_ic_states.retain(|(code, _), _| {
            installed
                .get(code_index(*code))
                .is_some_and(|s| s.is_some())
        });
        // Phase D.1.2: prune CallIcState side-table entries (Call + Construct)
        // for code that is no longer installed.
        self.call_ic_states.retain(|(code, _), _| {
            installed
                .get(code_index(*code))
                .is_some_and(|s| s.is_some())
        });
        self.construct_ic_states.retain(|(code, _), _| {
            installed
                .get(code_index(*code))
                .is_some_and(|s| s.is_some())
        });
        // Phase D.1.3: prune KeyedPropertyIcState side-table entries for code
        // that is no longer installed.
        self.keyed_property_ic_states.retain(|(code, _), _| {
            installed
                .get(code_index(*code))
                .is_some_and(|s| s.is_some())
        });
        report
    }

    #[inline]
    pub(crate) fn poll_incremental_mark_safepoint(agent: &mut Agent) {
        let _ = agent.heap_mut().poll_incremental_mark_step();
    }

    #[inline]
    #[allow(clippy::needless_pass_by_ref_mut)]
    #[cfg_attr(
        not(test),
        expect(
            clippy::unused_self,
            clippy::missing_const_for_fn,
            reason = "non-test builds keep the frame-depth instrumentation hook as a no-op"
        )
    )]
    fn note_frame_depth(&mut self) {
        #[cfg(test)]
        {
            self.peak_frame_depth = self.peak_frame_depth.max(self.frames.len());
        }
    }

    #[cfg(test)]
    #[inline]
    pub(crate) const fn peak_frame_depth(&self) -> usize {
        self.peak_frame_depth
    }

    #[cfg(test)]
    #[inline]
    pub(crate) fn active_for_in_enumerators(&self) -> usize {
        self.for_in_states.len()
    }

    #[inline]
    pub fn installed_function(&self, code: CodeRef) -> Option<&BytecodeFunction> {
        Some(&self.installed.get(code_index(code))?.as_ref()?.function)
    }

    #[inline]
    fn installed_function_record(&self, code: CodeRef) -> Option<&InstalledFunction> {
        self.installed
            .get(code_index(code))?
            .as_ref()
            .map(Arc::as_ref)
    }

    #[inline]
    pub fn installed_child_code(&self, code: CodeRef, child_index: u32) -> Option<CodeRef> {
        self.installed
            .get(code_index(code))?
            .as_ref()?
            .child_codes
            .get(usize::try_from(child_index).ok()?)
            .copied()
    }

    #[inline]
    pub fn source_map_entry(
        &self,
        code: CodeRef,
        instruction_offset: u32,
    ) -> Option<SourceMapEntry> {
        self.installed
            .get(code_index(code))?
            .as_ref()?
            .source_map_entry(instruction_offset)
    }

    #[inline]
    pub fn safepoint_at(
        &self,
        code: CodeRef,
        instruction_offset: u32,
    ) -> Option<SafepointDescriptor> {
        self.installed
            .get(code_index(code))?
            .as_ref()?
            .safepoint(instruction_offset)
    }

    #[inline]
    pub fn safepoint_by_id(&self, code: CodeRef, safepoint_id: u32) -> Option<SafepointDescriptor> {
        self.installed
            .get(code_index(code))?
            .as_ref()?
            .safepoint_by_id(safepoint_id)
    }

    #[inline]
    pub fn deopt_snapshot(&self, code: CodeRef, safepoint_id: u32) -> Option<DeoptSnapshot> {
        self.installed
            .get(code_index(code))?
            .as_ref()?
            .deopt_snapshot(safepoint_id)
            .cloned()
    }

    /// # Errors
    ///
    /// Returns a VM error if builtin bootstrap fails for the requested realm.
    pub fn bootstrap_realm(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        mode: BootstrapMode,
    ) -> Result<BootstrapArtifacts, VmError> {
        lyng_builtins::bootstrap_realm(
            agent,
            &mut self.builtin_cache,
            realm,
            BootstrapRequest::new(mode),
        )
        .map_err(VmError::BuiltinBootstrap)
    }

    pub(crate) fn builtin_constant(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        entry: BuiltinFunctionId,
    ) -> Option<Value> {
        self.builtin_cache.builtin_constant(agent, realm, entry)
    }

    fn with_extension_provider<T>(
        &mut self,
        provider: &SharedRealmExtensionProvider,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let previous = self.active_extension_provider.clone();
        self.active_extension_provider = Some(provider.clone());
        let result = f(self);
        self.active_extension_provider = previous;
        result
    }

    fn install_active_realm_extensions(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
    ) -> VmResult<()> {
        let Some(provider) = self.active_extension_provider.clone() else {
            return Ok(());
        };
        let _ = self.install_realm_extensions(agent, realm, &provider)?;
        Ok(())
    }

    /// # Errors
    ///
    /// Returns a VM error if bootstrap or provider extension installation fails.
    pub fn install_realm_extensions(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        provider: &SharedRealmExtensionProvider,
    ) -> VmResult<BootstrapArtifacts> {
        let artifacts = self.bootstrap_realm(agent, realm, BootstrapMode::SpecOnly)?;
        let bootstrap_state = agent.realm_bootstrap_state(realm).unwrap_or_default();
        if !bootstrap_state.embedding_ready() {
            let mut installation =
                RealmExtensionInstallation::new(self, agent, provider, artifacts);
            provider.install_realm_extensions(&mut installation)?;
            if !agent.mark_realm_embedding_bootstrapped(realm) {
                return Err(VmError::BuiltinBootstrap(
                    lyng_builtins::BuiltinBootstrapError::MissingRealm(realm),
                ));
            }
        }
        Ok(artifacts)
    }

    /// # Errors
    ///
    /// Returns a VM error if realm creation or extension installation fails.
    pub fn create_embedding_realm(
        &mut self,
        agent: &mut Agent,
        provider: &SharedRealmExtensionProvider,
    ) -> VmResult<BootstrapArtifacts> {
        let realm = agent.create_default_realm_shell(AllocationLifetime::Default);
        self.install_realm_extensions(agent, realm, provider)
    }

    /// # Errors
    ///
    /// Returns a VM error if function installation fails for the compiled script unit.
    pub fn install_script(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        unit: &CompiledScriptUnit,
    ) -> VmResult<InstalledCode> {
        self.record_source_text(unit.source(), unit.source_text());
        self.install_functions(agent, realm, unit.entry(), unit.functions(), unit.atoms())
    }

    /// # Errors
    ///
    /// Returns a VM error if function installation fails for the compiled function unit.
    pub fn install_function(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        unit: &CompiledFunctionUnit,
    ) -> VmResult<InstalledCode> {
        self.record_source_text(unit.source(), unit.source_text());
        self.install_functions(agent, realm, unit.entry(), unit.functions(), unit.atoms())
    }

    /// Begin evaluating a compiled script unit. Returns a builder; call `.run()` to execute.
    pub fn evaluate_script<'b>(
        &'b mut self,
        agent: &'b mut Agent,
        realm: RealmRecord,
        unit: &'b CompiledScriptUnit,
    ) -> EvaluateScript<'b> {
        EvaluateScript {
            vm: self,
            agent,
            realm,
            unit,
            host: None,
            registry: None,
            referrer: None,
            extensions: None,
            #[cfg(feature = "opcode-counters")]
            installed_counters: None,
            installed_debugger: None,
            installed_tiering: None,
        }
    }

    /// Begin evaluating an already-installed code record. Returns a builder; call `.run()` to execute.
    pub fn evaluate_installed<'b>(
        &'b mut self,
        agent: &'b mut Agent,
        installed: InstalledCode,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) -> EvaluateInstalled<'b> {
        EvaluateInstalled {
            vm: self,
            agent,
            installed,
            lexical_env,
            variable_env,
            host: None,
            registry: None,
            referrer: None,
            observer: None,
            entry_override: None,
            #[cfg(feature = "opcode-counters")]
            installed_counters: None,
            installed_debugger: None,
            installed_tiering: None,
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn evaluate_entry_with_registry_and_checkpoint(
        &mut self,
        agent: &mut Agent,
        installed: InstalledCode,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
        script_or_module_referrer: Option<AtomId>,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        global_script_plan: Option<&GlobalScriptInstantiationPlan>,
        entry_override: Option<EntryExecutionOverride>,
        observer: &mut dyn VmEvaluationObserver,
    ) -> VmResult<Value> {
        observer.before_bytecode_execution();
        let result = self.evaluate_entry_with_registry(
            agent,
            installed,
            lexical_env,
            variable_env,
            script_or_module_referrer,
            host,
            registry,
            global_script_plan,
            entry_override,
        );
        observer.after_bytecode_execution();
        let result = match result {
            Ok(value) => {
                observer.before_job_checkpoint();
                let checkpoint = self.checkpoint_promise_jobs(agent, host, registry);
                observer.after_job_checkpoint();
                checkpoint.map(|()| value)
            }
            Err(error) => Err(error),
        };
        agent.clear_kept_objects();
        result
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn evaluate_entry_with_registry(
        &mut self,
        agent: &mut Agent,
        installed: InstalledCode,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
        script_or_module_referrer: Option<AtomId>,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        global_script_plan: Option<&GlobalScriptInstantiationPlan>,
        entry_override: Option<EntryExecutionOverride>,
    ) -> VmResult<Value> {
        self.evaluate_entry_with_registry_from_offset(
            agent,
            installed,
            lexical_env,
            variable_env,
            script_or_module_referrer,
            host,
            registry,
            global_script_plan,
            entry_override,
            0,
        )
    }

    #[allow(clippy::too_many_arguments)]
    #[expect(
        clippy::too_many_lines,
        reason = "entry-frame setup and teardown stay contiguous so unwind ordering is auditable"
    )]
    fn evaluate_entry_with_registry_from_offset(
        &mut self,
        agent: &mut Agent,
        installed: InstalledCode,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
        script_or_module_referrer: Option<AtomId>,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        global_script_plan: Option<&GlobalScriptInstantiationPlan>,
        entry_override: Option<EntryExecutionOverride>,
        entry_offset: u32,
    ) -> VmResult<Value> {
        let code = installed.code();
        let code_record = agent
            .heap()
            .view()
            .code(code)
            .ok_or(VmError::MissingCodeRecord(code))?;
        let realm = code_record
            .realm()
            .or_else(|| agent.default_realm_id())
            .ok_or(VmError::MissingDefaultRealm)?;
        let _ = bootstrap_realm(
            agent,
            &mut self.builtin_cache,
            realm,
            BootstrapRequest::new(BootstrapMode::SpecOnly),
        )
        .map_err(VmError::BuiltinBootstrap)?;
        let function = self
            .installed_function(code)
            .cloned()
            .ok_or(VmError::MissingInstalledCode(code))?;
        let entry_private_env =
            entry_override.and_then(|override_state| override_state.private_env);
        let entry_lexical_this = entry_override.is_some_and(|override_state| {
            override_state.active_function.is_some() && override_state.lexical_this
        });
        let (lexical_env, variable_env, this_value, new_target) = Self::prepare_entry_execution(
            agent,
            code,
            realm,
            &function,
            lexical_env,
            variable_env,
            global_script_plan,
            entry_override,
        )?;
        let register_len = function
            .register_count()
            .checked_add(function.hidden_register_count())
            .expect("frame register span should fit within u16");
        let register_base =
            u32::try_from(self.register_stack_top()).expect("register stack length should fit u32");
        self.reserve_register_window(register_base, register_len);

        let context = ExecutionContext::bytecode(realm, code, lexical_env, variable_env)
            .with_private_env(entry_private_env)
            .with_this_state(if entry_lexical_this {
                ThisState::Lexical
            } else {
                ThisState::Value(this_value)
            })
            .with_new_target(new_target)
            .with_script_or_module_referrer(script_or_module_referrer);
        let context = if function.kind() == lyng_bytecode::BytecodeFunctionKind::Module {
            let module_referrer = agent
                .module_key_for_environment(lexical_env)
                .map(|key| agent.atoms_mut().intern_collectible(key.as_str()));
            ExecutionContext::module(realm, lexical_env, variable_env)
                .with_private_env(entry_private_env)
                .with_this_state(ThisState::Value(this_value))
                .with_script_or_module_referrer(module_referrer)
        } else {
            context
        };
        let frame = FrameRecord::new(
            code,
            entry_offset,
            RegisterWindow::new(register_base, register_len),
            None,
            realm,
            lexical_env,
            variable_env,
            context.kind(),
        )
        .with_this_value(this_value)
        .with_new_target(new_target)
        .with_flags(FrameFlags::entry().with_flag(FrameFlags::suspendable(), true));

        let prior_frame_depth = self.frames.len();
        let prior_register_len = usize::try_from(register_base)
            .expect("register stack base should fit into usize for truncation");
        let prior_context_depth = agent.execution_contexts().len();
        agent.push_execution_context(context);
        self.frames.push(frame);
        self.note_frame_depth();
        self.internal_completion_targets.push(prior_frame_depth);
        self.poll_debug_safepoint(agent, VmDebugSafepointKind::FunctionEntry);

        let result = self.run(agent, host, registry);
        if self.internal_completion_targets.last().copied() == Some(prior_frame_depth) {
            let _ = self.internal_completion_targets.pop();
        }

        while self.frames.len() > prior_frame_depth {
            let leaked = self
                .frames
                .pop()
                .expect("frame count should be greater than baseline");
            self.close_loop_iteration_frames(self.frames.len());
            self.close_with_environment_frames(self.frames.len());
            self.close_direct_eval_frames(self.frames.len());
            self.for_in_states.clear_window(leaked.registers());
            self.iterator_states.clear_window(leaked.registers());
            self.captured_name_references
                .clear_window(leaked.registers());
            self.finalize_mapped_arguments(agent, leaked.lexical_env())?;
            self.release_register_window(leaked.registers().base());
        }
        self.release_register_stack_to(prior_register_len);
        while agent.execution_contexts().len() > prior_context_depth {
            let _ = agent.pop_execution_context();
        }

        result
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn prepare_entry_execution(
        agent: &mut Agent,
        code: CodeRef,
        realm: RealmRef,
        function: &BytecodeFunction,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
        global_script_plan: Option<&GlobalScriptInstantiationPlan>,
        entry_override: Option<EntryExecutionOverride>,
    ) -> VmResult<(EnvironmentRef, EnvironmentRef, Value, Option<ObjectRef>)> {
        if function.kind() == lyng_bytecode::BytecodeFunctionKind::Module {
            let this_value = Value::undefined();
            if !function.needs_environment() {
                return Ok((lexical_env, lexical_env, this_value, None));
            }
            if agent.module_environment(lexical_env).is_some() {
                return Ok((lexical_env, lexical_env, this_value, None));
            }
            let layout = function
                .environment_layout()
                .and_then(|layout| lyng_env::EnvironmentLayoutId::from_raw(layout.get()))
                .ok_or(VmError::MissingEnvironmentLayout(code))?;
            let module_env = agent
                .alloc_module_environment(Some(lexical_env), layout, AllocationLifetime::Default)
                .ok_or(VmError::MissingEnvironmentLayout(code))?;
            return Ok((module_env, module_env, this_value, None));
        }

        let (this_value, new_target, home_object, active_function, lexical_this) =
            if let Some(override_state) = entry_override {
                (
                    override_state.this_value,
                    override_state.new_target,
                    override_state.home_object,
                    override_state.active_function,
                    override_state.lexical_this,
                )
            } else {
                (
                    Self::resolve_global_this(agent, realm, Value::undefined())?,
                    None,
                    None,
                    None,
                    false,
                )
            };
        if !function.needs_environment() {
            return Ok((lexical_env, variable_env, this_value, new_target));
        }

        let layout = function
            .environment_layout()
            .and_then(|layout| lyng_env::EnvironmentLayoutId::from_raw(layout.get()))
            .ok_or(VmError::MissingEnvironmentLayout(code))?;
        let global_object = agent
            .realm(realm)
            .ok_or(VmError::MissingRootShape(realm))?
            .global_object();
        let function_object = active_function.unwrap_or(global_object);
        let this_binding_status = if lexical_this && active_function.is_some() {
            ThisBindingStatus::Lexical
        } else {
            ThisBindingStatus::Initialized
        };
        let lexical_env = agent
            .alloc_function_environment(
                Some(lexical_env),
                layout,
                function_object,
                this_binding_status,
                this_value,
                new_target,
                home_object,
                AllocationLifetime::Default,
            )
            .ok_or(VmError::MissingEnvironmentLayout(code))?;
        if function.kind() == lyng_bytecode::BytecodeFunctionKind::Script
            && let Some(global_script_plan) = global_script_plan
        {
            Self::bind_global_script_lexical_bindings(
                agent,
                variable_env,
                lexical_env,
                global_script_plan,
            );
        }
        Ok((lexical_env, variable_env, this_value, new_target))
    }

    fn bind_global_script_lexical_bindings(
        agent: &mut Agent,
        global_env: EnvironmentRef,
        lexical_env: EnvironmentRef,
        plan: &GlobalScriptInstantiationPlan,
    ) {
        for binding in plan.lexical_bindings() {
            let name = agent.atoms_mut().intern_collectible(binding.name());
            let _ = agent.global_set_lexical_binding(global_env, name, lexical_env, binding.slot());
        }
    }

    pub(crate) fn source_text(&self, source: SourceId) -> Option<&str> {
        self.source_texts.get(&source).map(AsRef::as_ref)
    }

    fn record_source_text(&mut self, source: SourceId, source_text: Option<&str>) {
        self.next_dynamic_source_raw = self
            .next_dynamic_source_raw
            .max(source.raw().saturating_add(1));
        if let Some(source_text) = source_text {
            self.source_texts
                .entry(source)
                .or_insert_with(|| Arc::<str>::from(source_text));
        }
    }

    fn allocate_dynamic_source_id(&mut self) -> SourceId {
        loop {
            let source = SourceId::new(self.next_dynamic_source_raw);
            self.next_dynamic_source_raw = self.next_dynamic_source_raw.saturating_add(1);
            if !self.source_texts.contains_key(&source) {
                return source;
            }
        }
    }

    /// DSL-0c: sole dispatch entrypoint, routing through
    /// `crate::dsl::entry::run_via_dsl` (asm-DSL trampoline).
    ///
    /// Pulls the active frame + installed function (sub-8 invariant),
    /// then hands off to the DSL entry shim. `Vm::run` (in
    /// `vm/dispatch.rs`) calls this. DSL-0c (Task C5) deleted the
    /// α trampoline (`run_via_trampoline`, `run_trampoline`,
    /// `still_active`); `DISPATCH_TABLE` + `dispatch_handlers/`
    /// survive specifically for the wide-form prefix bridge in
    /// `crate::dsl::handlers::warm::op_prefix_via_alpha`.
    pub(crate) fn run_via_dsl(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> VmResult<Value> {
        self.refresh_dsl_poll_pending_for_agent(agent);
        let frame = self
            .frames
            .last()
            .copied()
            .expect("evaluation should install one active frame");
        let code = frame.code();
        let installed = self
            .installed
            .get(crate::vm::code_index_for_dsl(code))
            .and_then(Option::as_ref)
            .cloned()
            .ok_or(VmError::MissingInstalledCode(code))?;
        crate::dsl::entry::run_via_dsl(self, agent, host, registry, installed, frame)
    }

    /// Spec 2 Phase A: dispatched from `Agent::fire_watchpoints_for_shape` when
    /// an `AdaptiveProtoLoad` observer fires. Clears the IC slot identified by
    /// `(code, slot)` if its current generation matches `expected_generation`.
    /// Stale watchpoints from prior installs are silently dropped; the slot
    /// keeps whatever it currently holds.
    ///
    /// After `clear_site` the slot is `None`, so the `NamedPropertyFeedback`
    /// (and its `generation`) is dropped. The next install creates a fresh
    /// `NamedPropertyFeedback { generation: 0 }` and the slow path bumps to 1
    /// before registering new watchpoints. Watchpoints from the prior install
    /// era carry the old generation (> 0 after at least one bump) and will
    /// no-op on mismatch — correct staleness behaviour.
    pub(crate) fn clear_ic_slot_if_generation_matches(
        &mut self,
        code: CodeRef,
        slot: FeedbackSlotId,
        expected_generation: u32,
    ) {
        let Some(vector) = self.feedback_vectors.get_mut(code_index(code)) else {
            return;
        };
        if vector.generation(slot) != expected_generation {
            return;
        }
        vector.clear_site(slot);
        // Spec 2 Phase B.1.3: drop any out-of-line polymorphic chain attached
        // to this (code, slot). The site is being reset to `None`, so the
        // chain must follow it; otherwise stale chain entries would be
        // visible on the next install.
        self.drop_polymorphic_chain(code, slot);
        // Phase D.1.1: drop the PropertyIcState for this slot. On the next
        // slow-path install a fresh default entry will be created.
        self.property_ic_states.remove(&(code, slot));
        // Phase D.1.3: drop the KeyedPropertyIcState for this slot. Keyed-atom
        // sites can register AdaptiveProtoLoad watchpoints (via
        // `observe_keyed_atom_slow_path`), so the side-table entry must be
        // cleared when the watchpoint fires just like NamedProperty.
        self.keyed_property_ic_states.remove(&(code, slot));
        // Note: bump_generation after clear_site is a no-op because clear_site
        // drops the NamedPropertyFeedback that holds the generation counter.
        // Generation resets to 0 on the next fresh install; see doc above.
        self.mirror_flat_slot(code, slot);
        self.mirror_metadata_slot(code, slot);
    }
}

impl AdaptiveProtoLoadDispatch for Vm {
    fn clear_ic_slot_if_generation_matches(
        &mut self,
        code: CodeRef,
        slot: FeedbackSlotId,
        generation: u32,
    ) {
        Self::clear_ic_slot_if_generation_matches(self, code, slot, generation);
    }

    fn bump_generation_for_install(&mut self, code: CodeRef, slot: FeedbackSlotId) -> u32 {
        let Some(vector) = self.feedback_vectors.get_mut(code_index(code)) else {
            return 0;
        };
        vector.bump_generation(slot)
    }
}
