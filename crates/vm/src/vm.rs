use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use lyng_builtins::{
    BootstrapArtifacts, BootstrapMode, BootstrapRequest, BuiltinCache, bootstrap_realm,
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
    EnvironmentSlotFlags, ExecutionContextKind, ModuleRecord, ModuleStatus, RealmRecord,
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
#[cfg(feature = "diagnostic-counters")]
use crate::opcode_counts::OpcodeCounters;
use crate::{CallerContext, FrameFlags, FrameRecord, InstalledCode, RegisterWindow, VmError};

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
#[cfg(feature = "diagnostic-counters")]
mod ic_slow_counters;
pub mod ic_state;
pub mod install;
mod internal_calls;
mod jobs;
mod loop_iteration;
pub mod metadata_table;
mod modules;
mod names;
mod property_access;
mod registers;
mod runtime_objects;
pub mod semantics;
mod state;
pub mod status;
mod tiering;
mod values;
mod with_env;

// Compile-time guard: the arena slack above the soft limit must be large enough
// to accommodate the RangeError throw path's own frame + a minimal window.
// `HEADER_SLOTS` (7) + 256 is a conservative floor; `ARENA_SLACK_SLOTS` (4096)
// comfortably exceeds it. Evaluated at compile time so no runtime cost.
const _: () = assert!(
    crate::frame_arena::ARENA_SLACK_SLOTS >= crate::frame_header::HEADER_SLOTS + 256,
    "arena slack must leave room for the RangeError throw path's own frame + window",
);

use call::RejectingNativeRegistry;
use feedback::{
    CallCacheStorage, ConstructCacheStorage, KeyedPropertyNamedEntries, PolymorphicChain,
};
use ic_state::{CallIcState, GlobalCellIcState, KeyedPropertyIcState, PropertyIcState};
use install::InstalledFunction;
use metadata_table::MetadataTable;
use state::{
    ActiveEnvScopeRange, ActiveVmRoots, AsyncFrameState, AsyncGeneratorFrameState,
    AsyncGeneratorRequest, DirectEvalEnvironmentState, DynamicImportPhase, DynamicImportRequest,
    EntryExecutionOverride, LoopIterationEnvironment, PendingDynamicImport,
    SuspendedExecutionSideState, TemplateCacheKey, WithEnvironmentState,
};
use values::{bytecode_index, code_index, decode_env_operand, string_text_array_index};
// Re-export `code_index` for the DSL entry shim so it can resolve
// the metadata-table slot for a frame's `CodeRef` without
// re-implementing the (id - 1) → usize indexing.
pub use values::code_index as code_index_for_dsl;

pub use modules::LoadedModuleRoot;

pub use debugger::{
    VmDebugCommand, VmDebugFrame, VmDebugHook, VmDebugPauseContext, VmDebugPauseReason,
    VmDebugSafepoint, VmDebugSafepointKind, VmDebugStepMode, VmDebugger,
};
pub use feedback::{FeedbackInlineCacheState, FeedbackKeyedPropertyFamily};
#[cfg(feature = "diagnostic-counters")]
pub use ic_slow_counters::{IcSlowPathCause, IcSlowPathCounters, IcSlowPathKind};
pub use status::{
    ArithStatus, CallStatus, CalleeSummary, ComparisonStatus, ConstructStatus,
    KeyedPropertyDenseStatusEntry, KeyedPropertyNamedStatusEntry, KeyedPropertyStatus,
    MetadataTableFootprint, NamedPropertyEntryKind, NamedPropertyHandlerSummary,
    NamedPropertyStatus, NamedPropertyStatusEntry, ScalarObserved,
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

/// One establishment scope on the `Vm` side-stack. `base_depth` is the frame
/// depth at which the establishing frame sits; the scope covers all frames at
/// depth >= `base_depth` until that frame unwinds. `referrer` is the seed (None
/// is valid — e.g. a script with no host referrer). `realm` is the establishing
/// root's realm; callee-less root frames recover their realm from the covering
/// scope here. Function frames derive their realm from the callee instead.
#[derive(Clone, Copy, Debug)]
struct ReferrerScope {
    base_depth: usize,
    realm: RealmRef,
    referrer: Option<lyng_common::AtomId>,
}

#[derive(Default)]
pub struct Vm {
    /// Never-reallocated value backing for register windows. A window's base offset
    /// is the running register cursor (`arena.top()`); because the storage never
    /// moves, a pointer into it stays valid across every push.
    arena: crate::frame_arena::FrameArena,
    /// Call frame register (cfr) of the active frame: the arena slot offset of its
    /// `[FrameHeader][window]` run. `u32::MAX` means no active frame.
    current_cfr: u32,
    /// Depth-indexed cold per-activation state (handler cursor, tail linkage,
    /// generator resume). Seeded on every frame push; the top frame's slot lives
    /// at `frame_depth - 1`.
    frame_cold: crate::frame_cold::FrameColdTable,
    /// Number of currently-active frames (0 == empty). The per-frame
    /// [`crate::frame_header::FrameHeader`] overlay + the depth-keyed
    /// [`crate::frame_cold::FrameColdTable`] + the `current_cfr`/`caller_cfr`
    /// chain are the sole frame source. Incremented in
    /// [`Self::push_frame_with_header`], decremented in [`Self::pop_frame_depth`].
    /// INVARIANT: after any push/pop, call [`Self::refresh_running_context`] to
    /// keep the Agent's `running_context` in sync with the active frame.
    frame_depth: usize,
    referrer_scopes: Vec<ReferrerScope>,
    dispatch_frame_check_epoch: u32,
    installed: Vec<Option<Arc<InstalledFunction>>>,
    current_exception: Option<Value>,
    #[cfg(feature = "diagnostic-counters")]
    pub(crate) counters: OpcodeCounters,
    debugger: VmDebugger,
    atom_texts: HashMap<AtomId, Box<str>>,
    preferred_atoms_by_text: HashMap<Box<str>, AtomId>,
    source_texts: HashMap<SourceId, Arc<str>>,
    /// Out-of-line polymorphic IC entries (indices `POLY_LIMIT..8`). Outer `Vec`
    /// keyed by `code_index(code)`, inner `Box<[..]>` keyed by zero-based slot.
    /// Lazy; cleared on `AdaptiveProtoLoad` fire and on code GC.
    pub(crate) polymorphic_chains: Vec<Option<Box<[Option<PolymorphicChain>]>>>,
    /// Rust-only IC state machine for `NamedProperty` slots. Outer `Vec` indexed
    /// by `code_index(code)`; inner `Box<[..]>` by zero-based slot. Entries
    /// allocated eagerly at install, populated lazily. The asm-readable bits live
    /// on `PropertyMetadata` inside `MetadataTable`; this table holds Rust-only
    /// fields. Cleared on code GC.
    pub(crate) property_ic_states: Vec<Option<Box<[Option<PropertyIcState>]>>>,
    /// Rust-only IC state machine for `Call` slots. Same shape as
    /// `property_ic_states`. Asm-readable bits live on `CallMetadata`.
    pub(crate) call_ic_states: Vec<Option<Box<[Option<CallIcState>]>>>,
    /// Rust-only IC state machine for `Construct` slots. Same shape as
    /// `call_ic_states`.
    pub(crate) construct_ic_states: Vec<Option<Box<[Option<CallIcState>]>>>,
    /// Per-slot Call cache entries (callee/builtin data). Keyed by
    /// `(CodeRef, FeedbackSlotId)`. Lazy; pruned on code GC.
    pub(in crate::vm) call_cache_entries: HashMap<(CodeRef, FeedbackSlotId), Box<CallCacheStorage>>,
    /// Per-slot Construct cache entries. Keyed by `(CodeRef, FeedbackSlotId)`.
    /// Lazy; pruned on code GC.
    pub(in crate::vm) construct_cache_entries:
        HashMap<(CodeRef, FeedbackSlotId), Box<ConstructCacheStorage>>,
    /// Per-slot `KeyedProperty` named-atom cache entries. Keyed by
    /// `(CodeRef, FeedbackSlotId)`. Lazy; pruned on code GC.
    pub(in crate::vm) keyed_property_named_entries:
        HashMap<(CodeRef, FeedbackSlotId), KeyedPropertyNamedEntries>,
    /// Rust-only IC state machine for `KeyedProperty` slots. Same shape as
    /// `property_ic_states`. Asm-readable bits live on `KeyedPropertyMetadata`.
    pub(crate) keyed_property_ic_states: Vec<Option<Box<[Option<KeyedPropertyIcState>]>>>,
    /// Per-site global cell IC. Caches `LoadGlobal` resolution to skip name
    /// lookup on repeats. Stale entries are inert (they re-resolve on generation
    /// mismatch); lazy; keyed by `(CodeRef, FeedbackSlotId)`.
    pub(crate) global_cell_ic_states: HashMap<(CodeRef, FeedbackSlotId), GlobalCellIcState>,
    /// Per-code-object IC metadata buffer keyed by `code_index(code_ref)`. `None`
    /// for uninstalled code. Allocated eagerly at install; never grown thereafter.
    pub(crate) metadata_tables: Vec<Option<MetadataTable>>,
    /// Safepoint poll-pending byte read by `poll_safepoint!` (warm
    /// `op_loop_header` / backward jumps). The asm reads
    /// `[x22, VM_POLL_PENDING_OFFSET]` where `x22 = *mut Vm`; the offset
    /// is derived from `offset_of!(Vm, dsl_poll_pending)` in
    /// `crate::dsl::reg_convention`. Non-zero means a same-thread
    /// incremental-mark step or debugger pause is pending.
    pub(crate) dsl_poll_pending: u8,
    /// Mirror of the executing realm's `global_structure_generation`, read by the asm
    /// `LoadGlobal` mode-7 hit via `[x22, #VM_GLOBAL_IC_GENERATION_OFFSET]`. MUST equal
    /// the live env generation at every mode-7 hit: refreshed at run entry and at the
    /// slow-path-return choke point (`translate_outcome`), so any structural bump during
    /// a slow path is reflected before asm dispatch resumes.
    pub(crate) dsl_global_ic_generation: u32,
    pub(crate) tiering: Tiering,
    /// `LLInt` feedback drain optimization: codes whose frames were entered
    /// since the last `drain_llint_scalar_feedback`. Step 1 of the drain
    /// scans only these (non-executed code has `execution_count == 0` on all
    /// arith slots, so draining it is a guaranteed no-op). Cleared each drain.
    executed_codes: Vec<CodeRef>,
    /// Parallel to `self.installed` (indexed by `code_index(code)`): dedup
    /// stamp recording the `drain_generation` at which a code was last pushed
    /// to `executed_codes`. Grown lazily by `note_executed_code`.
    code_executed_stamp: Vec<u32>,
    /// Current dedup generation for `code_executed_stamp`; bumped each drain so
    /// stamps from the prior cycle no longer match and codes re-queue.
    drain_generation: u32,
    /// Per-kind, per-cause IC slow-path entry counters. Inert by
    /// default — every counter starts at zero and is bumped exactly
    /// once per slow-path entry from the cold handler bridges in
    /// `crate::dsl::handlers::cold`. See [`IcSlowPathCounters`] for
    /// the table layout. Gated behind the `diagnostic-counters`
    /// feature so production builds carry no per-slow-entry overhead.
    #[cfg(feature = "diagnostic-counters")]
    pub(crate) ic_slow_path_counters: IcSlowPathCounters,
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
    #[cfg(feature = "diagnostic-counters")]
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

    pub const fn with_referrer(mut self, key: &'b ModuleKey) -> Self {
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
    #[cfg(feature = "diagnostic-counters")]
    pub const fn with_opcode_counters(mut self, counters: &'b mut OpcodeCounters) -> Self {
        self.installed_counters = Some(counters);
        self
    }

    /// Install a caller-owned [`VmDebugger`] for the duration of `.run()` /
    /// `.run_retaining_installed()`. The debugger is swapped into the VM at
    /// run entry and swapped back at run exit, so pause-control mutations
    /// (and step state the hook installed) persist on the caller's struct.
    pub const fn with_debugger(mut self, debugger: &'b mut VmDebugger) -> Self {
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
    pub const fn with_tiering(mut self, tiering: &'b mut Tiering) -> Self {
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
            #[cfg(feature = "diagnostic-counters")]
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

        #[cfg(feature = "diagnostic-counters")]
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
        #[cfg(feature = "diagnostic-counters")]
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
    #[cfg(feature = "diagnostic-counters")]
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

    pub const fn with_referrer(mut self, atom: AtomId) -> Self {
        self.referrer = Some(atom);
        self
    }

    pub fn with_observer(mut self, observer: &'b mut dyn VmEvaluationObserver) -> Self {
        self.observer = Some(observer);
        self
    }

    pub(crate) const fn with_entry_override(mut self, override_: EntryExecutionOverride) -> Self {
        self.entry_override = Some(override_);
        self
    }

    /// Redirect opcode-counter writes to an externally-owned
    /// `OpcodeCounters` for the duration of `.run()`. See
    /// [`EvaluateScript::with_opcode_counters`] for the full
    /// description.
    #[cfg(feature = "diagnostic-counters")]
    pub const fn with_opcode_counters(mut self, counters: &'b mut OpcodeCounters) -> Self {
        self.installed_counters = Some(counters);
        self
    }

    /// Install a caller-owned [`VmDebugger`] for the duration of `.run()`.
    /// See [`EvaluateScript::with_debugger`] for the full description.
    pub const fn with_debugger(mut self, debugger: &'b mut VmDebugger) -> Self {
        self.installed_debugger = Some(debugger);
        self
    }

    /// Install a caller-owned [`Tiering`] for the duration of `.run()`. See
    /// [`EvaluateScript::with_tiering`] for the full description.
    pub const fn with_tiering(mut self, tiering: &'b mut Tiering) -> Self {
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
            #[cfg(feature = "diagnostic-counters")]
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

        #[cfg(feature = "diagnostic-counters")]
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
        #[cfg(feature = "diagnostic-counters")]
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
            arena: crate::frame_arena::FrameArena::new(),
            current_cfr: u32::MAX,
            frame_cold: crate::frame_cold::FrameColdTable::new(),
            frame_depth: 0,
            referrer_scopes: Vec::new(),
            dispatch_frame_check_epoch: 0,
            installed: Vec::new(),
            current_exception: None,
            #[cfg(feature = "diagnostic-counters")]
            counters: OpcodeCounters::new(),
            debugger: VmDebugger::default(),
            atom_texts: HashMap::new(),
            preferred_atoms_by_text: HashMap::new(),
            source_texts: HashMap::new(),
            metadata_tables: Vec::new(),
            polymorphic_chains: Vec::new(),
            property_ic_states: Vec::new(),
            call_ic_states: Vec::new(),
            construct_ic_states: Vec::new(),
            call_cache_entries: HashMap::new(),
            construct_cache_entries: HashMap::new(),
            keyed_property_named_entries: HashMap::new(),
            keyed_property_ic_states: Vec::new(),
            global_cell_ic_states: HashMap::new(),
            dsl_poll_pending: 0,
            dsl_global_ic_generation: 0,
            tiering: Tiering::disabled(),
            executed_codes: Vec::new(),
            code_executed_stamp: Vec::new(),
            // Start at 1: lazily-grown `code_executed_stamp` entries default to
            // 0, so a generation of 0 would spuriously match an unqueued code's
            // stamp and skip it on its first frame entry.
            drain_generation: 1,
            #[cfg(feature = "diagnostic-counters")]
            ic_slow_path_counters: IcSlowPathCounters::new(),
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
    /// Hot path: two index dereferences instead of a `HashMap` probe.
    #[allow(dead_code, reason = "used in tests via pub(crate)")]
    #[inline]
    pub(crate) fn polymorphic_chain(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<&PolymorphicChain> {
        let slot_zero = (slot.get() - 1) as usize;
        self.polymorphic_chains
            .get(code_index(code))?
            .as_deref()?
            .get(slot_zero)?
            .as_ref()
    }

    /// Returns a mutable reference to the polymorphic chain for `(code, slot)`,
    /// lazily creating an empty chain on first access. The install path uses a
    /// split-borrow directly; this helper is for callers holding an exclusive
    /// `&mut Vm`.
    #[allow(
        dead_code,
        reason = "install path uses split-borrow; this surface is for non-split callers"
    )]
    #[inline]
    pub(crate) fn polymorphic_chain_mut(
        &mut self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> &mut PolymorphicChain {
        let index = code_index(code);
        let slot_zero = (slot.get() - 1) as usize;
        let slots = self.polymorphic_chains[index]
            .as_deref_mut()
            .expect("polymorphic_chains slab must be allocated at install");
        slots[slot_zero].get_or_insert_with(PolymorphicChain::new)
    }

    /// Removes the polymorphic chain for `(code, slot)`. Called when the IC
    /// transitions to Megamorphic or is cleared by an `AdaptiveProtoLoad` fire.
    #[inline]
    pub(crate) fn drop_polymorphic_chain(&mut self, code: CodeRef, slot: FeedbackSlotId) {
        let slot_zero = (slot.get() - 1) as usize;
        if let Some(Some(slots)) = self.polymorphic_chains.get_mut(code_index(code))
            && let Some(entry) = slots.get_mut(slot_zero)
        {
            *entry = None;
        }
    }

    pub fn metadata_table(&self, code: CodeRef) -> Option<&MetadataTable> {
        let idx = code_index(code);
        self.metadata_tables.get(idx).and_then(|t| t.as_ref())
    }

    pub(crate) fn metadata_table_mut(&mut self, code: CodeRef) -> Option<&mut MetadataTable> {
        let idx = code_index(code);
        self.metadata_tables.get_mut(idx).and_then(|t| t.as_mut())
    }

    /// Post-mark GC sweep. Drops polymorphic chain entries for dead code.
    /// The GC call site uses an inline split-borrow retain in
    /// `force_collect_with_active_roots`; this method is for callers that
    /// already hold `&mut Vm`.
    #[allow(
        dead_code,
        reason = "GC call site uses inline split-borrow; this surface is for other callers"
    )]
    pub(crate) fn prune_dead_code_polymorphic_chains(&mut self, is_live: impl Fn(CodeRef) -> bool) {
        prune_dead_code_ic_slab(&mut self.polymorphic_chains, is_live);
    }

    /// Returns the `PropertyIcState` for `(code, slot)` if any.
    #[allow(dead_code, reason = "consumed from tests and feedback callers")]
    #[inline]
    pub(crate) fn property_ic_state(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<&PropertyIcState> {
        let slot_zero = (slot.get() - 1) as usize;
        self.property_ic_states
            .get(code_index(code))?
            .as_deref()?
            .get(slot_zero)?
            .as_ref()
    }

    /// Returns a mutable reference to the `PropertyIcState` for `(code, slot)`
    /// if any. Returns `None` if the code is uninstalled, the slot is out of
    /// range, or the slot is cold.
    #[inline]
    pub(crate) fn property_ic_state_mut(
        &mut self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<&mut PropertyIcState> {
        let slot_zero = (slot.get() - 1) as usize;
        self.property_ic_states
            .get_mut(code_index(code))?
            .as_deref_mut()?
            .get_mut(slot_zero)?
            .as_mut()
    }

    /// Returns a mutable reference to the `PropertyIcState` for `(code, slot)`,
    /// lazily inserting a default on first access. The outer vec slot must have
    /// been allocated by `store_installed`.
    #[inline]
    pub(crate) fn property_ic_state_or_default_mut(
        &mut self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> &mut PropertyIcState {
        let index = code_index(code);
        let slot_zero = (slot.get() - 1) as usize;
        let slots = self.property_ic_states[index]
            .as_deref_mut()
            .expect("property_ic_states slab must be allocated at install");
        slots[slot_zero].get_or_insert_with(PropertyIcState::default)
    }

    /// Clears the per-slot entry on watchpoint fire or IC invalidation.
    #[inline]
    pub(crate) fn clear_property_ic_state(&mut self, code: CodeRef, slot: FeedbackSlotId) {
        let slot_zero = (slot.get() - 1) as usize;
        if let Some(Some(slots)) = self.property_ic_states.get_mut(code_index(code))
            && let Some(entry) = slots.get_mut(slot_zero)
        {
            *entry = None;
        }
    }

    /// Post-mark GC sweep. Drops the `PropertyIcState` slab for dead code.
    #[allow(
        dead_code,
        reason = "sweep surface; call site wired alongside prune_dead_code_polymorphic_chains"
    )]
    pub(crate) fn prune_dead_code_property_ic_states(&mut self, is_live: impl Fn(CodeRef) -> bool) {
        prune_dead_code_ic_slab(&mut self.property_ic_states, is_live);
    }

    /// Returns the `CallIcState` for a `Call` slot `(code, slot)`.
    #[allow(dead_code, reason = "consumed from tests and feedback callers")]
    #[inline]
    pub(crate) fn call_ic_state(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<&CallIcState> {
        let slot_zero = (slot.get() - 1) as usize;
        self.call_ic_states
            .get(code_index(code))?
            .as_deref()?
            .get(slot_zero)?
            .as_ref()
    }

    /// Returns the `CallIcState` for a `Construct` slot `(code, slot)`.
    #[allow(dead_code, reason = "consumed from tests and feedback callers")]
    #[inline]
    pub(crate) fn construct_ic_state(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<&CallIcState> {
        let slot_zero = (slot.get() - 1) as usize;
        self.construct_ic_states
            .get(code_index(code))?
            .as_deref()?
            .get(slot_zero)?
            .as_ref()
    }

    /// Construct fast-path invalidation (eager clear). Clears the `Construct`
    /// IC slot at `(code, slot)` — its `CallIcState` in `construct_ic_states`,
    /// its `ConstructCacheStorage` entry, and the asm-readable `CallMetadata` —
    /// iff the slot's current generation matches `expected_generation`.
    ///
    /// Mirrors [`Self::clear_ic_slot_if_generation_matches`] but operates on the
    /// construct slabs. The construct slot's generation lives on the
    /// `CallMetadata` entry inside `MetadataTable` (there is no generation field
    /// on `CallIcState`); read it the same way [`Self::construct_status`] does.
    /// A generation mismatch means the watchpoint is stale (the slot was
    /// re-cached since registration) and is silently dropped.
    pub(crate) fn clear_construct_ic_slot_if_generation_matches(
        &mut self,
        code: CodeRef,
        slot: FeedbackSlotId,
        expected_generation: u32,
    ) {
        let current_generation = self
            .metadata_table(code)
            .map(|table| table.call(slot.get()).generation);
        if current_generation != Some(expected_generation) {
            // Either the code is gone, the slot was never installed, or the
            // generation has moved on since registration — stale watchpoint.
            return;
        }
        // Clear the Rust-side state-machine entry; the next slow-path Construct
        // visit re-inserts a fresh default.
        let slot_zero = (slot.get() - 1) as usize;
        if let Some(Some(slab)) = self.construct_ic_states.get_mut(code_index(code))
            && let Some(entry) = slab.get_mut(slot_zero)
        {
            *entry = None;
        }
        // Drop the cached constructor/prototype data for this site.
        self.construct_cache_entries.remove(&(code, slot));
        // Zero the asm-readable CallMetadata entry (mode/generation/callee_bits).
        if let Some(table) = self.metadata_table_mut(code) {
            *table.call_mut(slot.get()) = metadata_table::call::CallMetadata::default();
        }
    }

    /// Post-mark GC sweep. Drops `CallIcState` slabs for dead code (both Call
    /// and Construct tables).
    #[allow(
        dead_code,
        reason = "sweep surface; call site wired alongside prune_dead_code_property_ic_states"
    )]
    pub(crate) fn prune_dead_code_call_ic_states(&mut self, is_live: impl Fn(CodeRef) -> bool) {
        prune_dead_code_ic_slab(&mut self.call_ic_states, &is_live);
        prune_dead_code_ic_slab(&mut self.construct_ic_states, is_live);
    }

    /// Returns the `KeyedPropertyIcState` for `(code, slot)` if any.
    #[allow(dead_code, reason = "consumed from tests and feedback callers")]
    #[inline]
    pub(crate) fn keyed_property_ic_state(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<&KeyedPropertyIcState> {
        let slot_zero = (slot.get() - 1) as usize;
        self.keyed_property_ic_states
            .get(code_index(code))?
            .as_deref()?
            .get(slot_zero)?
            .as_ref()
    }

    /// Lazily inserts and returns a mutable reference to the
    /// `KeyedPropertyIcState` for `(code, slot)`.
    #[inline]
    pub(crate) fn keyed_property_ic_state_or_default_mut(
        &mut self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> &mut KeyedPropertyIcState {
        let index = code_index(code);
        let slot_zero = (slot.get() - 1) as usize;
        let slots = self.keyed_property_ic_states[index]
            .as_deref_mut()
            .expect("keyed_property_ic_states slab must be allocated at install");
        slots[slot_zero].get_or_insert_with(KeyedPropertyIcState::default)
    }

    /// Clears the per-slot keyed entry on watchpoint fire.
    #[inline]
    pub(crate) fn clear_keyed_property_ic_state(&mut self, code: CodeRef, slot: FeedbackSlotId) {
        let slot_zero = (slot.get() - 1) as usize;
        if let Some(Some(slots)) = self.keyed_property_ic_states.get_mut(code_index(code))
            && let Some(entry) = slots.get_mut(slot_zero)
        {
            *entry = None;
        }
    }

    /// Post-mark GC sweep. Drops the `KeyedPropertyIcState` slab for dead code.
    #[allow(
        dead_code,
        reason = "sweep surface; call site wired alongside prune_dead_code_call_ic_states"
    )]
    pub(crate) fn prune_dead_code_keyed_property_ic_states(
        &mut self,
        is_live: impl Fn(CodeRef) -> bool,
    ) {
        prune_dead_code_ic_slab(&mut self.keyed_property_ic_states, is_live);
    }

    /// Post-mark GC sweep. Drops `MetadataTable` entries for dead code.
    /// The vec is indexed by `code_index(code_ref) = code_ref.get() - 1`.
    #[allow(dead_code, reason = "called from tests and the GC sweep site")]
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
    #[cfg(feature = "diagnostic-counters")]
    #[inline]
    pub const fn opcode_counters(&self) -> &OpcodeCounters {
        &self.counters
    }

    #[cfg(feature = "diagnostic-counters")]
    #[inline]
    pub const fn opcode_counters_mut(&mut self) -> &mut OpcodeCounters {
        &mut self.counters
    }

    /// Records `count` argument values pushed into `argument_scratch`. No-op
    /// when the counter is disabled (the default in production builds and
    /// when the `diagnostic-counters` feature is off). Inlined so the disabled
    /// case compiles to a single load+branch.
    #[cfg(feature = "diagnostic-counters")]
    #[inline]
    pub(in crate::vm) fn record_argument_scratch_pushes(&self, count: u64) {
        self.counters.record_argument_scratch_pushes(count);
    }

    #[cfg(not(feature = "diagnostic-counters"))]
    #[inline]
    pub(in crate::vm) fn record_argument_scratch_pushes(&self, _count: u64) {}

    /// Records `count` argument values copied into a callee bytecode frame.
    /// Symmetric with `record_argument_scratch_pushes` — together they let
    /// tests verify that ordinary calls copy each argument exactly once
    /// (`frame_copies` == n, `scratch_pushes` == 0) instead of twice.
    #[cfg(feature = "diagnostic-counters")]
    #[inline]
    pub(in crate::vm) fn record_argument_frame_copies(&self, count: u64) {
        self.counters.record_argument_frame_copies(count);
    }

    #[cfg(not(feature = "diagnostic-counters"))]
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

    /// Current value of the asm-read global-IC generation mirror. Used by
    /// tests to assert coherence with the agent's live generation. (The asm
    /// mode-7 hit reads the field directly via `VM_GLOBAL_IC_GENERATION_OFFSET`,
    /// not through this accessor — hence `cfg(test)` until a non-test reader, if
    /// any, appears.)
    #[cfg(test)]
    #[inline]
    pub(crate) const fn dsl_global_ic_generation(&self) -> u32 {
        self.dsl_global_ic_generation
    }

    /// Prime the baseline generation from the executing realm's global env.
    /// Called at top-level run entry AFTER global declaration instantiation, so any
    /// agent-side bumps from instantiation are already captured in the baseline.
    /// Dispatch-time bumps (delete/defineProperty on the global) are covered by the
    /// `translate_outcome` slow-path choke point.
    #[inline]
    pub(crate) fn prime_global_ic_generation(&mut self, agent: &Agent, global_env: EnvironmentRef) {
        self.dsl_global_ic_generation = agent.global_structure_generation(global_env);
    }

    /// Refresh the mirror from the realm of the currently-executing frame, so a
    /// cross-realm mode-7 hit compares against the correct realm's generation.
    ///
    /// `prime_global_ic_generation` sets the baseline from the realm that was active
    /// at `Vm::run` entry. A cross-realm call (`$262.createRealm` → invoke a function whose
    /// `[[Realm]]` differs) egresses through `translate_outcome` with the callee
    /// frame active; deriving the global env from that frame's realm re-primes the
    /// generation to the executing realm before its first opcode runs.
    /// No-op (mirror left as-is) if the realm has no resolvable global env.
    #[inline]
    pub(crate) fn refresh_global_ic_generation_for_realm(
        &mut self,
        agent: &Agent,
        realm: RealmRef,
    ) {
        if let Some(global_env) = agent
            .heap()
            .view()
            .realm(realm)
            .and_then(lyng_gc::RuntimeRealmRecord::global_env)
        {
            self.dsl_global_ic_generation = agent.global_structure_generation(global_env);
        }
    }

    #[inline]
    pub(crate) fn poll_debug_safepoint(&mut self, agent: &Agent, kind: VmDebugSafepointKind) {
        if !self.debug_poll_enabled() {
            return;
        }
        let Some(frame) = self.frame() else {
            return;
        };
        let safepoint = VmDebugSafepoint::new(kind, &frame, self.frame_depth);
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

    /// The live (used-prefix) register slots.
    ///
    /// Returns only the used prefix of the arena, never the full fixed box.
    ///
    /// NOTE: this is the WHOLESALE prefix — it interleaves the packed-int
    /// [`crate::frame_header::FrameHeader`] slots (6 of the 7 are NOT valid
    /// `Value`s) with each frame's window. It MUST NOT be traced as `Value`s by
    /// GC; the GC uses a per-frame window walk plus a typed header-ref walk.
    /// Remaining callers use it only to test arena emptiness.
    #[inline]
    pub fn live_register_slots(&self) -> &[Value] {
        &self.arena.slots()[..self.arena.top()]
    }

    /// Raw view of the frame arena's value slots, used by the GC per-frame window
    /// walk to trace exactly `[cfr+HEADER_SLOTS .. +frame_window_len(cfr)]` for
    /// each live frame (never the interleaved header slots). Indexing past a live
    /// frame's window or into a header overlay reads packed ints, not `Value`s, so
    /// callers must bound their reads with [`Self::frame_window_len`].
    #[inline]
    pub(crate) fn arena_slots(&self) -> &[Value] {
        self.arena.slots()
    }

    /// Materialize every live frame (outermost-first) by reconstructing each
    /// from its arena header overlay + cold slot + geometry. Walks the
    /// `current_cfr`/`caller_cfr` chain and reverses so index 0 is the root.
    /// Allocates; not on any hot path. The reconstructed `instruction_offset`
    /// comes from `saved_pc` (the parked PC), which for the active top frame
    /// may lag the live dispatch snapshot until the next sync.
    #[inline]
    pub fn frames(&self) -> Vec<FrameRecord> {
        let depth = self.frame_depth;
        let mut frames: Vec<FrameRecord> = self
            .frame_cfrs()
            .enumerate()
            .map(|(rev_index, cfr)| {
                // `frame_cfrs` yields innermost-first; depth-1 is the top frame.
                let frame_index = depth - 1 - rev_index;
                self.reconstruct_frame_from_header(cfr, frame_index)
            })
            .collect();
        frames.reverse();
        frames
    }

    /// The active (top) frame, reconstructed from its header overlay, cold slot,
    /// and geometry, or `None` when no frame is active.
    #[inline]
    pub fn frame(&self) -> Option<FrameRecord> {
        let cfr = self.current_cfr_opt()?;
        Some(self.reconstruct_frame_from_header(cfr, self.frame_depth - 1))
    }

    #[inline]
    pub(super) const fn register_stack_top(&self) -> usize {
        self.arena.top()
    }

    #[inline]
    pub(super) fn release_register_stack_to(&mut self, top: usize) {
        debug_assert!(
            top <= self.arena.top(),
            "register stack cursor should only move back during cleanup"
        );
        let Ok(top) = u32::try_from(top) else {
            debug_assert!(false, "register stack cursor should fit into u32");
            return;
        };
        self.arena.release_to(top);
    }

    #[cfg(test)]
    #[inline]
    pub(crate) fn register_stack_storage_len_for_tests(&self) -> usize {
        self.arena.slots().len()
    }

    /// Test-only: seed the register arena's used prefix with `values` and set
    /// the cursor to `values.len()`.
    #[cfg(test)]
    pub(crate) fn seed_register_stack_for_tests(&mut self, values: &[Value]) {
        self.arena.slots_mut()[..values.len()].copy_from_slice(values);
        self.arena.set_top(values.len());
    }

    /// Test-only: push a synthetic caller/root frame through the real reservation
    /// path so it carries a header and a window base ≥ `HEADER_SLOTS` in the
    /// arena — matching the layout every production push produces. `build`
    /// receives the reserved window and returns the `FrameRecord` to install.
    /// The reserved window slots are zeroed; `seed_window` overwrites the prefix.
    /// Returns the window base.
    #[cfg(test)]
    pub(crate) fn push_test_root_frame(
        &mut self,
        agent: &mut Agent,
        register_len: u16,
        seed_window: &[Value],
        build: impl FnOnce(RegisterWindow) -> FrameRecord,
    ) -> u32 {
        let (cfr, register_base) = self
            .reserve_frame(agent, register_len)
            .expect("test frame reservation should fit the arena");
        let start = register_base as usize;
        self.arena.slots_mut()[start..start + seed_window.len()].copy_from_slice(seed_window);
        let frame = build(RegisterWindow::new(register_base, register_len));
        // Seed an establishment scope so `realm_of` can recover the realm for
        // callee-less roots (the common case for test helpers).
        let depth = self.frame_depth;
        let realm = frame
            .callee()
            .and_then(|callee| agent.objects().function_data(callee))
            .and_then(lyng_objects::FunctionObjectData::realm)
            .or_else(|| agent.default_realm_id())
            .expect("test root frame must resolve a realm");
        self.push_referrer_scope(depth, realm, None);
        self.push_frame_with_header(cfr, frame);
        register_base
    }

    /// Raw mutable pointer to the start of the register-stack storage, used by
    /// [`crate::dsl::entry::run_via_dsl`] to compute the active frame's `REGS`
    /// pin. The arena never reallocates, so this base pointer stays stable
    /// across nested calls within a single trampoline invocation.
    ///
    /// Callers must respect Rust's aliasing rules — the returned pointer
    /// aliases the arena's backing buffer; concurrent reborrows of
    /// `&mut self.arena` would be UB.
    #[inline]
    pub(crate) fn register_stack_storage_mut_ptr(&mut self) -> *mut Value {
        self.arena.base_mut_ptr()
    }

    /// Crate-visible accessor for the dispatch frame-check epoch, used by
    /// [`crate::dsl::entry::run_via_dsl`] when seeding the entry `DispatchState`.
    #[inline]
    pub(crate) const fn dispatch_frame_check_epoch_for_dsl(&self) -> u32 {
        self.dispatch_frame_check_epoch
    }

    #[cfg(test)]
    pub(crate) const fn string_code_units_scratch_capacity(&self) -> usize {
        self.string_code_units_scratch.capacity()
    }

    /// Test-only: overwrite a single raw arena slot with a verbatim bit pattern,
    /// bypassing typed header setters. Used to plant non-Value patterns in header
    /// slots for GC mistrace tests.
    #[cfg(test)]
    pub(crate) fn write_arena_slot_for_tests(&mut self, slot: u32, value: Value) {
        self.arena.slots_mut()[slot as usize] = value;
    }

    /// Test-only: read a single raw arena slot (the verbatim bits, NOT a typed
    /// header field). Pairs with [`Self::write_arena_slot_for_tests`].
    #[cfg(test)]
    pub(crate) fn arena_slot_for_tests(&self, slot: u32) -> Value {
        self.arena.slots()[slot as usize]
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

    /// `cfr` (call frame register) of a frame: the arena slot offset of its
    /// `[FrameHeader][window]` run. The header occupies `[cfr .. cfr +
    /// HEADER_SLOTS)` and the register window starts at `cfr + HEADER_SLOTS`, so
    /// the cfr is the window base shifted back by the header. Release sites
    /// reclaim to this offset (not the window base) so the header slots are
    /// freed too.
    #[inline]
    pub(crate) const fn cfr_of(frame: &FrameRecord) -> u32 {
        frame.registers().base() - crate::frame_header::HEADER_SLOTS as u32
    }

    /// Register-window length for a frame *running this bytecode `code`*
    /// (entry/call/generator/module frames). Returns `register_count() +
    /// hidden_register_count()` of `code`'s installed function — the exact value
    /// `reserve_frame` is handed for those push sites (the generator-restore site
    /// round-trips the saved window, which is that same span). The two counts are
    /// immutable per installed code.
    ///
    /// NOT valid for the synthetic job-root frame: it reserves a 0-width window
    /// (`reserve_frame(agent, 0)`) yet borrows a non-matching, non-zero-register
    /// `CodeRef` from `job_caller_code()`, so `window_len_for(its code) > 0` while
    /// its real window is 0. Callers walking a cfr chain (e.g. the GC trace) must
    /// use [`Self::frame_window_len`], which special-cases the job root.
    ///
    /// Panics if `code` is not installed (a live frame's `code` always is).
    #[inline]
    pub(crate) fn window_len_for(&self, code: lyng_types::CodeRef) -> u16 {
        let function = self
            .installed_function(code)
            .expect("window_len_for: frame code must be installed");
        function
            .register_count()
            .checked_add(function.hidden_register_count())
            .expect("frame register span should fit within u16")
    }

    /// Register-window length of the live frame at `cfr` — exact for every frame,
    /// including the synthetic job root (reserved 0-width despite borrowing a
    /// non-zero-register code). Use this (not `window_len_for(code)` directly) when
    /// bounding a frame's window during a cfr-chain walk (e.g. the GC trace).
    ///
    /// Audit of the 5 push sites confirms Job is the sole special case: entry
    /// (`vm.rs`) and the two bytecode-call installs (`bytecode_calls.rs`) all
    /// reserve `register_count + hidden_register_count`; the generator/async
    /// restore reserves the saved window, which is that same span for the
    /// suspended code. Only the job root reserves a 0-width window under a
    /// mismatched code.
    #[inline]
    pub(crate) fn frame_window_len(&self, cfr: u32) -> u16 {
        let header = self.frame_header(cfr);
        if header.kind() == lyng_env::ExecutionContextKind::Job {
            0
        } else {
            self.window_len_for(header.code())
        }
    }

    /// Shared, immutable view of the header overlay at `cfr`.
    ///
    /// SAFETY: `cfr` is a valid frame base reserved with `HEADER_SLOTS + window`
    /// slots via [`Self::reserve_frame`]; [`crate::frame_header::FrameHeader`] is
    /// `repr(C)` POD sized to `HEADER_SLOTS * size_of::<Value>()`, so the cast
    /// over the arena slots is sound and stays in-bounds. The `&self` receiver
    /// ties the returned reference to the VM borrow, so the borrow checker
    /// guarantees no other live borrow of the arena slots aliases this header.
    #[inline]
    pub(crate) fn frame_header(&self, cfr: u32) -> &crate::frame_header::FrameHeader {
        debug_assert!(
            (cfr as usize) + crate::frame_header::HEADER_SLOTS <= self.arena.slots().len(),
            "frame header overlay must stay within the arena",
        );
        let ptr = self.arena.slots().as_ptr();
        unsafe {
            &*ptr
                .add(cfr as usize)
                .cast::<crate::frame_header::FrameHeader>()
        }
    }

    /// Mutable view of the header overlay at `cfr`. Same safety contract as
    /// [`Self::frame_header`]; the `&mut self` receiver ties the returned
    /// reference to the VM borrow, so the borrow checker guarantees no other
    /// live borrow of the arena slots aliases this header.
    #[inline]
    pub(crate) fn frame_header_mut(&mut self, cfr: u32) -> &mut crate::frame_header::FrameHeader {
        let len = self.arena.slots().len();
        debug_assert!(
            (cfr as usize) + crate::frame_header::HEADER_SLOTS <= len,
            "frame header overlay must stay within the arena",
        );
        let ptr = self.arena.slots_mut().as_mut_ptr();
        unsafe {
            &mut *ptr
                .add(cfr as usize)
                .cast::<crate::frame_header::FrameHeader>()
        }
    }

    /// The active frame's header overlay, or `None` when no frame is active.
    #[allow(dead_code, reason = "used by the header-mirror test")]
    #[inline]
    pub(crate) fn current_frame_header(&self) -> Option<&crate::frame_header::FrameHeader> {
        (self.current_cfr != u32::MAX).then(|| self.frame_header(self.current_cfr))
    }

    /// The active frame's cfr (slot offset of its header), or `None` when no frame is active.
    #[allow(dead_code, reason = "used by the frame_depth_and_caller_walk test")]
    #[inline]
    pub(crate) fn current_cfr_opt(&self) -> Option<u32> {
        (self.current_cfr != u32::MAX).then_some(self.current_cfr)
    }

    /// Returns `(code, saved_pc)` from the active frame's header overlay, or `None`
    /// when no frame is active. Used by the property-access chain to populate
    /// `VmProxyBridge.caller_code`/`caller_pc` from bridge-internal helpers
    /// (e.g. `VmToPrimitiveBridge`) that don't have direct access to the frame params.
    #[inline]
    pub(crate) fn current_code_and_pc(&self) -> Option<(CodeRef, u32)> {
        let cfr = self.current_cfr_opt()?;
        let h = self.frame_header(cfr);
        Some((h.code(), h.saved_pc()))
    }

    /// Depth of the active frame stack (0 == empty).
    #[inline]
    pub(crate) const fn frame_depth(&self) -> usize {
        self.frame_depth
    }

    /// Mutable cold state of the current frame, or `None` when no frame is active.
    #[inline]
    pub(crate) fn current_cold_mut(&mut self) -> Option<&mut crate::frame_cold::FrameColdState> {
        self.current_cfr_opt()?;
        let depth = self.frame_depth() - 1;
        Some(self.frame_cold.get_mut(depth))
    }

    /// The cold-state slots backing every currently-live frame (depth order).
    #[inline]
    pub(crate) fn frame_cold_live_slots(&self) -> &[crate::frame_cold::FrameColdState] {
        self.frame_cold.live_slots(self.frame_depth())
    }

    /// Slot offsets (cfr) of every live frame, innermost first.
    pub(crate) fn frame_cfrs(&self) -> impl Iterator<Item = u32> + '_ {
        let mut next = self.current_cfr_opt();
        std::iter::from_fn(move || {
            let cfr = next?;
            next = self.frame_header(cfr).caller_cfr();
            Some(cfr)
        })
    }

    /// Rebuild a full [`FrameRecord`] for the live frame at `cfr` entirely from
    /// its arena-resident state: the header overlay, the cold side-table slot at
    /// `depth`, and the derived register-window geometry.
    ///
    /// `instruction_offset` is sourced from `saved_pc` (the parked PC). This is
    /// called exclusively at frame-switch boundaries (return / call / catch /
    /// dispatch-entry), never on a same-frame opcode step — the same-frame
    /// `Continue` arm advances the live snapshot PC locally without touching
    /// `saved_pc`. `FrameRecord` does not carry `realm`; derive it via
    /// [`Self::realm_of`] / [`Self::frame_record_realm`].
    pub(crate) fn reconstruct_frame_from_header(&self, cfr: u32, depth: usize) -> FrameRecord {
        let header = *self.frame_header(cfr);
        let window = RegisterWindow::new(
            cfr + crate::frame_header::HEADER_SLOTS as u32,
            self.frame_window_len(cfr),
        );
        let mut frame = FrameRecord::new(
            header.code(),
            header.saved_pc(),
            window,
            header.return_register(),
            header.lexical_env(),
            header.variable_env(),
            header.kind(),
        )
        .with_this_state(header.this_state())
        .with_this_value(header.this_value())
        .with_construct_this(header.construct_this())
        .with_new_target(header.new_target())
        .with_callee(header.callee())
        .with_private_env(header.private_env())
        .with_flags(crate::frame::FrameFlags::from_raw(header.flags_bits()));
        // `handler_cursor`, `tail_caller`(+strict), `resume_*` come from the cold
        // slot; `parameter_initializer_end_offset` is restored separately (metadata,
        // not part of `apply_cold`).
        let cold = self.frame_cold.get(depth);
        frame.apply_cold(cold);
        frame.set_parameter_initializer_end_offset(cold.parameter_initializer_end_offset);
        frame
    }

    /// Reserve `[header][window]` for a new frame at the current arena top.
    /// Returns `(cfr, window_base)` where `window_base = cfr + HEADER_SLOTS`.
    /// A run crossing the soft limit is rejected with a `RangeError` (the slack
    /// above the soft limit is reserved for the throw path). This is the single
    /// rejection point on every reservation path (entry, bytecode calls,
    /// generator resume, job root).
    #[inline]
    fn reserve_frame(&mut self, agent: &mut Agent, register_len: u16) -> VmResult<(u32, u32)> {
        let slots = crate::frame_header::HEADER_SLOTS + usize::from(register_len);
        let cfr = self
            .arena
            .bump(slots)
            .ok_or_else(|| VmError::Abrupt(lyng_ops::errors::throw_range_error(agent)))?;
        // `bump` does not clear slots on reuse; zero the window region so a
        // re-entered frame sees `undefined` for slots its callee may not rewrite.
        // (The header overlay is fully written by `write_header_from_record`.)
        let window_base = cfr + crate::frame_header::HEADER_SLOTS as u32;
        let start = window_base as usize;
        let end = start + usize::from(register_len);
        self.arena.slots_mut()[start..end].fill(lyng_types::Value::undefined());
        Ok((cfr, window_base))
    }

    /// Mirror `record` into the arena header overlay at `cfr` and seed the
    /// cold-table slot at `depth`. Called on every frame push; `caller_cfr` is
    /// the chain link to the frame below (`None` for a root frame).
    fn write_header_from_record(
        &mut self,
        cfr: u32,
        caller_cfr: Option<u32>,
        depth: usize,
        record: &FrameRecord,
    ) {
        self.frame_cold.reset_at(depth);
        {
            let cold = self.frame_cold.get_mut(depth);
            cold.handler_cursor = record.handler_cursor();
            cold.tail_caller = record.tail_caller();
            cold.tail_caller_strict = record.tail_caller_strict();
            cold.resume_kind = record.resume_kind();
            cold.resume_value = record.resume_value();
            cold.resume_active = record.resume_active();
            cold.parameter_initializer_end_offset = record.parameter_initializer_end_offset();
        }
        let h = self.frame_header_mut(cfr);
        *h = crate::frame_header::FrameHeader::zeroed();
        h.set_caller_cfr(caller_cfr);
        h.set_saved_pc(record.instruction_offset());
        h.set_code(record.code());
        h.set_callee(record.callee());
        h.set_this(record.this_state(), record.this_value());
        h.set_return_register(record.return_register());
        h.set_variable_env(record.variable_env());
        h.set_lexical_env(record.lexical_env());
        h.set_private_env(record.private_env());
        h.set_new_target(record.new_target());
        h.set_construct_this(record.construct_this());
        // The arg_count ABI slot is reserved for the asm path's real argument count;
        // seed 0 until the asm path writes it. No reader consumes it yet.
        h.set_arg_count(0);
        h.set_flags_bits(record.flags().raw());
        h.set_kind_raw(record.kind() as u8);
    }

    /// Common push tail: mirror `frame` into the arena header overlay, bump the
    /// frame depth, and advance `current_cfr`. `cfr` must come from the
    /// [`Self::reserve_frame`] that allocated `frame.registers()`.
    #[inline]
    fn push_frame_with_header(&mut self, cfr: u32, frame: FrameRecord) {
        let depth = self.frame_depth;
        let caller_cfr = (self.current_cfr != u32::MAX).then_some(self.current_cfr);
        debug_assert_eq!(
            Self::cfr_of(&frame),
            cfr,
            "frame window base must sit HEADER_SLOTS above its cfr"
        );
        self.write_header_from_record(cfr, caller_cfr, depth, &frame);
        self.frame_depth = depth + 1;
        self.current_cfr = cfr;
        #[cfg(debug_assertions)]
        self.debug_assert_cfr_chain_invariant();
    }

    /// Verify `current_cfr`/`frame_depth` consistency and `caller_cfr` chain
    /// well-formedness after every push or release (debug-only).
    ///
    /// Invariants:
    /// 1. `current_cfr == u32::MAX` iff `frame_depth == 0`.
    /// 2. The caller-chain walks from `current_cfr` to `None` in exactly
    ///    `frame_depth` steps, with each successive cfr strictly less than
    ///    the one above it.
    #[cfg(debug_assertions)]
    fn debug_assert_cfr_chain_invariant(&self) {
        debug_assert_eq!(
            self.current_cfr == u32::MAX,
            self.frame_depth == 0,
            "current_cfr must be u32::MAX iff frame_depth is 0 \
             (current_cfr={}, frame_depth={})",
            self.current_cfr,
            self.frame_depth,
        );
        if self.frame_depth == 0 {
            return;
        }
        let mut steps = 0usize;
        let mut prev_cfr = self.current_cfr;
        let mut next = self.frame_header(self.current_cfr).caller_cfr();
        steps += 1;
        while let Some(caller) = next {
            debug_assert!(
                caller < prev_cfr,
                "caller_cfr must be strictly less than its callee's cfr \
                 (caller={caller}, callee={prev_cfr}): chain is corrupt"
            );
            prev_cfr = caller;
            next = self.frame_header(caller).caller_cfr();
            steps += 1;
            debug_assert!(
                steps <= self.frame_depth,
                "caller_cfr chain is longer than frame_depth={} (cycle or stale link)",
                self.frame_depth
            );
        }
        debug_assert_eq!(
            steps, self.frame_depth,
            "caller_cfr chain length ({steps}) must equal frame_depth ({})",
            self.frame_depth
        );
    }

    /// Decrement the maintained frame depth. Used at pop sites that already hold
    /// the live snapshot (tail-frame teardown, generator/async suspend). Does NOT
    /// release the arena run or restore `current_cfr` — that is done by
    /// [`Self::release_frame_to_caller`] after any reads on the still-mapped run.
    /// Pop sites that must rebuild the record from the arena use
    /// [`Self::pop_current_frame`] instead.
    #[inline]
    fn pop_frame_depth(&mut self) {
        debug_assert!(self.frame_depth > 0, "pop requires one active frame");
        self.frame_depth = self.frame_depth.saturating_sub(1);
    }

    /// Reconstruct the active (top) frame from its header overlay + cold slot +
    /// geometry, then decrement the maintained frame depth. Used at pop sites that
    /// need the popped frame's fields (window cleanup, return-register write,
    /// mapped-arguments finalization). After this returns, `current_cfr` still
    /// points at the popped frame (its run stays mapped) until
    /// [`Self::release_frame_to_caller`] reclaims it.
    #[inline]
    fn pop_current_frame(&mut self) -> FrameRecord {
        debug_assert!(
            self.current_cfr != u32::MAX && self.frame_depth > 0,
            "pop requires one active frame"
        );
        let depth = self.frame_depth - 1;
        let frame = self.reconstruct_frame_from_header(self.current_cfr, depth);
        self.frame_depth = depth;
        frame
    }

    /// Release a just-popped frame's arena run (header + window) and restore
    /// `current_cfr` to the caller's cfr (or `u32::MAX` when the stack empties).
    /// Depth must have been decremented by [`Self::pop_frame_depth`] or
    /// [`Self::pop_current_frame`] before calling this.
    #[inline]
    fn release_frame_to_caller(&mut self, popped_cfr: u32) {
        self.current_cfr = self
            .frame_header(popped_cfr)
            .caller_cfr()
            .unwrap_or(u32::MAX);
        self.arena.release_to(popped_cfr);
        // Note: `referrer_scopes` may still be non-empty at this point (the caller
        // will call `unwind_referrer_scopes_to` separately). Only the cfr/depth
        // chain is checked here; the establishment side-stack is verified by
        // `unwind_referrer_scopes_to` when it unwinds all the way to depth 0.
        #[cfg(debug_assertions)]
        self.debug_assert_cfr_chain_invariant();
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
        // Prune IC side-table slabs for code that is no longer installed. Each
        // slab is a Vec keyed by code_index; the entry is set to None (freeing
        // the inner Box) when the code is dead. A CodeRef is live iff its slot
        // in `self.installed` is `Some(_)`.
        let installed = &self.installed;
        prune_dead_code_ic_slab_by_installed(&mut self.polymorphic_chains, installed);
        prune_dead_code_ic_slab_by_installed(&mut self.property_ic_states, installed);
        prune_dead_code_ic_slab_by_installed(&mut self.call_ic_states, installed);
        prune_dead_code_ic_slab_by_installed(&mut self.construct_ic_states, installed);
        prune_dead_code_ic_slab_by_installed(&mut self.keyed_property_ic_states, installed);
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
            self.peak_frame_depth = self.peak_frame_depth.max(self.frame_depth);
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

    /// Record a new establishment starting at `base_depth` frames deep. `realm`
    /// is the establishing root's realm (consulted only by callee-less roots; see
    /// [`Self::establishment_realm_covering`]).
    pub(crate) fn push_referrer_scope(
        &mut self,
        base_depth: usize,
        realm: RealmRef,
        referrer: Option<AtomId>,
    ) {
        self.referrer_scopes.push(ReferrerScope {
            base_depth,
            realm,
            referrer,
        });
    }

    /// Drop all scopes established at frame depth ≥ `target_frame_depth` (those
    /// frames are being exited).
    pub(crate) fn unwind_referrer_scopes_to(&mut self, target_frame_depth: usize) {
        while self
            .referrer_scopes
            .last()
            .is_some_and(|scope| scope.base_depth >= target_frame_depth)
        {
            self.referrer_scopes.pop();
        }
        // When all frames are gone the establishment side-stack must also be empty.
        // Checked here rather than in the low-level primitives because
        // `release_frame_to_caller` and `unwind_referrer_scopes_to` are called
        // together at the exit of every entry/job root.
        debug_assert!(
            target_frame_depth > 0 || self.referrer_scopes.is_empty(),
            "establishment side-stack must be empty when frame_depth reaches 0 \
             ({} scope(s) remain)",
            self.referrer_scopes.len(),
        );
    }

    /// The referrer of the current establishment (the nearest one toward the
    /// base); the single source of truth for the script/module referrer.
    pub(crate) fn current_referrer(&self) -> Option<AtomId> {
        self.referrer_scopes.last().and_then(|scope| scope.referrer)
    }

    /// Every realm captured on the establishment side-stack (root realms). GC must
    /// trace these so a root realm reachable only through a callee-less root frame
    /// stays rooted; function-frame realms are reachable via the callee instead.
    pub(in crate::vm) fn establishment_realms(&self) -> impl Iterator<Item = RealmRef> + '_ {
        self.referrer_scopes.iter().map(|scope| scope.realm)
    }

    /// The realm of the establishment covering frame `depth`: the top scope whose
    /// `base_depth <= depth`. Mirrors [`Self::current_referrer`]'s nearest-toward-
    /// base walk, but is depth-relative so a callee-less root at any depth recovers
    /// the realm pushed for the establishment that actually owns it (rather than a
    /// deeper scope that has not yet unwound). Returns `None` when no scope covers
    /// the depth (no active establishment).
    pub(crate) fn establishment_realm_covering(&self, depth: usize) -> Option<RealmRef> {
        self.referrer_scopes
            .iter()
            .rev()
            .find(|scope| scope.base_depth <= depth)
            .map(|scope| scope.realm)
    }

    /// 0-based depth of the frame at `cfr` (number of callers below it), found by
    /// walking the caller chain. The top frame is at `frame_depth() - 1`.
    pub(crate) fn depth_of(&self, cfr: u32) -> usize {
        let mut depth = 0usize;
        let mut caller = self.frame_header(cfr).caller_cfr();
        while let Some(c) = caller {
            depth += 1;
            caller = self.frame_header(c).caller_cfr();
        }
        depth
    }

    /// Realm of the frame at `cfr`. Function frames derive from the callee's
    /// `[[Realm]]`; callee-less roots read the covering establishment scope.
    ///
    /// [`Self::frame_record_realm`] is the `FrameRecord`-snapshot variant; the
    /// two differ only in the callee-less fallback source (this reads the
    /// establishment scope; the snapshot variant reads the ambient
    /// running-context realm).
    #[allow(
        clippy::collapsible_if,
        reason = "nested let bindings read clearer than a let-chain given the multi-line .and_then inner; the crate uses no `&& let` chains today"
    )]
    pub(crate) fn realm_of(&self, agent: &Agent, cfr: u32) -> RealmRef {
        let header = self.frame_header(cfr);
        if let Some(callee) = header.callee() {
            if let Some(realm) = agent
                .objects()
                .function_data(callee)
                .and_then(lyng_objects::FunctionObjectData::realm)
            {
                return realm;
            }
        }
        self.establishment_realm_covering(self.depth_of(cfr))
            .expect("a live frame must be covered by an establishment scope")
    }

    /// Realm of the current frame, or `None` when no frame is active.
    pub(crate) fn current_realm_of(&self, agent: &Agent) -> Option<RealmRef> {
        self.current_cfr_opt().map(|cfr| self.realm_of(agent, cfr))
    }

    /// Realm represented by a `FrameRecord` snapshot of the current frame.
    /// Function-frame snapshots derive from the callee's `[[Realm]]`; callee-less
    /// roots fall back to the ambient running-context realm. An associated
    /// function (no `&self`) so static helpers without a `Vm` borrow can call it.
    ///
    /// [`Self::realm_of`] is the live-cfr variant; it uses the covering
    /// establishment scope as its callee-less fallback instead.
    #[allow(
        clippy::collapsible_if,
        reason = "nested let bindings read clearer than a let-chain given the multi-line .and_then inner; the crate uses no `&& let` chains today"
    )]
    pub(crate) fn frame_record_realm(agent: &Agent, frame: &FrameRecord) -> RealmRef {
        if let Some(callee) = frame.callee() {
            if let Some(realm) = agent
                .objects()
                .function_data(callee)
                .and_then(lyng_objects::FunctionObjectData::realm)
            {
                return realm;
            }
        }
        if let Some(running) = agent.running_context() {
            return running.realm();
        }
        // Production never reaches here: a callee-less frame used as a realm source
        // is always the *current* frame, whose running_context was refreshed at the
        // push (or a pushed root, covered by the establishment side-stack). Only
        // unit tests that invoke a helper with a hand-built `FrameRecord` outside an
        // active dispatch (no pushed frame, so no running_context) land here; fall
        // back to the agent's default realm — the realm every such test frame is
        // built against. Gated to test builds so a production miss stays a loud bug.
        #[cfg(test)]
        if let Some(default_realm) = agent.default_realm_id() {
            return default_realm;
        }
        panic!(
            "frame_record_realm needs an active running context or establishment scope for a callee-less frame"
        );
    }

    /// Extract the [`CallerContext`] (`realm/lexical_env/code/pc`) from a caller
    /// `FrameRecord`. Synthetic-safe: reads only struct fields and
    /// [`Self::frame_record_realm`], so it is valid on a synthetic
    /// `RegisterWindow::new(0, 0)` frame. Extract into a local before any
    /// `&mut agent` argument to avoid a double borrow.
    #[inline]
    pub(crate) fn caller_context_from_record(agent: &Agent, frame: &FrameRecord) -> CallerContext {
        CallerContext {
            realm: Self::frame_record_realm(agent, frame),
            lexical_env: frame.lexical_env(),
            code: frame.code(),
            pc: frame.instruction_offset(),
        }
    }

    /// [`CallerContext`] from a real arena-backed caller frame identified by a
    /// [`crate::frame::FrameView`]. MUST NOT be used on a synthetic frame
    /// (`realm_of`/`frame_header` underflow on a zero-cfr). For a real active
    /// frame this agrees with `caller_context_from_record` field-for-field.
    #[inline]
    pub(crate) fn caller_context_from_view(
        &self,
        agent: &Agent,
        view: crate::frame::FrameView,
    ) -> CallerContext {
        CallerContext {
            realm: self.realm_of(agent, view.cfr()),
            lexical_env: self.frame_header(view.cfr()).lexical_env(),
            code: view.code(),
            pc: view.instruction_offset(),
        }
    }

    /// Build a [`FrameRecord`] from the overlay fields for the frame identified by
    /// `view`. Cold fields (`handler_cursor`, `tail_caller`, `resume_*`,
    /// `parameter_initializer_end_offset`) are left at zero defaults, which is safe
    /// because all callee sites that consume this record read only overlay fields.
    #[inline]
    pub(super) fn frame_record_for_view(&self, view: crate::frame::FrameView) -> FrameRecord {
        let h = self.frame_header(view.cfr());
        FrameRecord::new(
            view.code(),
            view.instruction_offset(),
            view.registers(),
            h.return_register(),
            h.lexical_env(),
            h.variable_env(),
            h.kind(),
        )
        .with_callee(h.callee())
        .with_new_target(h.new_target())
        .with_flags(crate::frame::FrameFlags::from_raw(h.flags_bits()))
        .with_this_state(h.this_state())
        .with_this_value(h.this_value())
        .with_construct_this(h.construct_this())
        .with_private_env(h.private_env())
    }

    /// Refresh the Agent's ambient running-context from the active frame. Called
    /// at every frame transition.
    pub(crate) fn refresh_running_context(&self, agent: &mut Agent) {
        let running = self
            .current_realm_of(agent)
            .map(|realm| lyng_env::RunningContext::new(realm, self.current_referrer()));
        agent.set_running_context(running);
        // `current_cfr` is authoritative here (unlike
        // `refresh_running_context_to_caller`, which derives from the caller while
        // `current_cfr` still points at the just-popped frame).
        debug_assert_eq!(
            agent.running_context().map(lyng_env::RunningContext::realm),
            self.current_cfr_opt().map(|cfr| self.realm_of(agent, cfr)),
            "running_context realm must equal the realm derived from the current frame"
        );
    }

    /// Refresh the running-context from the *caller* of `popped`.
    ///
    /// The return path and construct/return finalization must observe the caller's
    /// realm (per spec [[Construct]] step 13c / `GetThisBinding`). At that point
    /// `current_cfr` still points at the just-popped frame (its run stays mapped
    /// until [`Self::release_frame_to_caller`]), so plain
    /// [`Self::refresh_running_context`] would derive the popped frame's realm.
    /// Reading the caller cfr explicitly restores the correct cross-realm behavior
    /// without disturbing `current_cfr`.
    pub(crate) fn refresh_running_context_to_caller(&self, agent: &mut Agent, popped_cfr: u32) {
        let caller_cfr = self.frame_header(popped_cfr).caller_cfr();
        let running = caller_cfr
            .map(|cfr| self.realm_of(agent, cfr))
            .map(|realm| lyng_env::RunningContext::new(realm, self.current_referrer()));
        agent.set_running_context(running);
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
            #[cfg(feature = "diagnostic-counters")]
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
            #[cfg(feature = "diagnostic-counters")]
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
        // Snapshot the arena cursor (== this frame's cfr) before reserving, so
        // the post-run reset reclaims the whole `[header][window]` run.
        let prior_register_top = self.register_stack_top();
        let (cfr, register_base) = self.reserve_frame(agent, register_len)?;

        let entry_this_state = if entry_lexical_this {
            ThisState::Lexical
        } else {
            ThisState::Value(this_value)
        };
        // Module entries advertise the `Module` kind and re-intern their module
        // key as the referrer; every other entry is a `Function` carrying the
        // inherited referrer.
        let (frame_kind, frame_referrer) =
            if function.kind() == lyng_bytecode::BytecodeFunctionKind::Module {
                let module_referrer = agent
                    .module_key_for_environment(lexical_env)
                    .map(|key| agent.atoms_mut().intern_collectible(key.as_str()));
                (ExecutionContextKind::Module, module_referrer)
            } else {
                (ExecutionContextKind::Function, script_or_module_referrer)
            };
        let frame = FrameRecord::new(
            code,
            entry_offset,
            RegisterWindow::new(register_base, register_len),
            None,
            lexical_env,
            variable_env,
            frame_kind,
        )
        .with_this_value(this_value)
        .with_this_state(entry_this_state)
        .with_private_env(entry_private_env)
        .with_new_target(new_target)
        .with_flags(FrameFlags::entry().with_flag(FrameFlags::suspendable(), true));

        let prior_frame_depth = self.frame_depth();
        // Record the entry's realm + referrer on the parallel side-stack. The
        // establishing frame sits at `prior_frame_depth`, so this scope unwinds
        // exactly when that frame does (see the unwind loop below). The derived
        // `frame_referrer` keeps script and module branches in lockstep; `realm`
        // is the root's realm (this entry frame is callee-less, so `realm_of`
        // recovers it from this scope rather than a callee).
        self.push_referrer_scope(prior_frame_depth, realm, frame_referrer);
        self.note_executed_code(frame.code());
        self.push_frame_with_header(cfr, frame);
        self.refresh_running_context(agent);
        self.note_frame_depth();
        self.internal_completion_targets.push(prior_frame_depth);
        self.poll_debug_safepoint(agent, VmDebugSafepointKind::FunctionEntry);

        // Prime the global-IC generation mirror (read by the asm `LoadGlobal`
        // mode-7 hit) before dispatch begins. Cache the executing realm's global
        // env so the slow-path-egress refresh is a Vec index, and set the
        // baseline generation — which already reflects any declaration-time bumps
        // from `instantiate_global_script` above. Cheap, not on the asm hot path.
        if let Some(realm_record) = agent.realm(realm) {
            self.prime_global_ic_generation(agent, realm_record.global_env());
        }

        let result = self.run(agent, host, registry);
        if self.internal_completion_targets.last().copied() == Some(prior_frame_depth) {
            let _ = self.internal_completion_targets.pop();
        }

        while self.frame_depth() > prior_frame_depth {
            // Reconstruct the leaked top frame from its header overlay + cold slot
            // (this also decrements the maintained depth); its run stays mapped at
            // `current_cfr` until `release_frame_to_caller` below.
            let leaked = self.pop_current_frame();
            self.close_loop_iteration_frames(self.frame_depth());
            self.close_with_environment_frames(self.frame_depth());
            self.close_direct_eval_frames(self.frame_depth());
            self.for_in_states.clear_window(leaked.registers());
            self.iterator_states.clear_window(leaked.registers());
            self.captured_name_references
                .clear_window(leaked.registers());
            // `lexical_env` is authoritative in the overlay; read before the
            // arena run is released.
            let lexical_env = self.frame_header(Self::cfr_of(&leaked)).lexical_env();
            self.finalize_mapped_arguments(agent, lexical_env)?;
            self.release_frame_to_caller(Self::cfr_of(&leaked));
        }
        self.unwind_referrer_scopes_to(prior_frame_depth);
        self.release_register_stack_to(prior_register_top);
        self.refresh_running_context(agent);

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
        let mut bound_any = false;
        for binding in plan.lexical_bindings() {
            let name = agent.atoms_mut().intern_collectible(binding.name());
            let _ = agent.global_set_lexical_binding(global_env, name, lexical_env, binding.slot());
            bound_any = true;
        }
        if bound_any {
            // A new global lexical binding may shadow an existing var/builtin on
            // the global object, redirecting future `LoadGlobal`s for that name
            // from the cell to the lexical slot. Conservatively bump the
            // structure generation so any cached site re-resolves. This only
            // runs at script/eval instantiation, so the cost is negligible.
            agent.bump_global_structure_generation(global_env);
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

    /// Sole dispatch entrypoint, routing through the asm-DSL trampoline
    /// `crate::dsl::entry::run_via_dsl`. Called by `Vm::run` in `vm/dispatch.rs`.
    pub(crate) fn run_via_dsl(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> VmResult<Value> {
        self.refresh_dsl_poll_pending_for_agent(agent);
        // Reconstruct the just-pushed active frame (dispatch-entry frame switch;
        // loading the parked `saved_pc` as the entry PC is correct).
        let cfr = self
            .current_cfr_opt()
            .expect("evaluation should install one active frame");
        let frame = self.reconstruct_frame_from_header(cfr, self.frame_depth() - 1);
        let code = self
            .current_frame_header()
            .expect("a live frame must have an overlaid header at current_cfr")
            .code();
        let installed = self
            .installed
            .get(crate::vm::code_index_for_dsl(code))
            .and_then(Option::as_ref)
            .cloned()
            .ok_or(VmError::MissingInstalledCode(code))?;
        crate::dsl::entry::run_via_dsl(self, agent, host, registry, installed, frame)
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
        // Lazily insert a default entry on first visit, then bump the generation.
        let state = self.property_ic_state_or_default_mut(code, slot);
        state.generation = state.generation.wrapping_add(1);
        state.generation
    }

    fn clear_construct_ic_slot_if_generation_matches(
        &mut self,
        code: CodeRef,
        slot: FeedbackSlotId,
        generation: u32,
    ) {
        Self::clear_construct_ic_slot_if_generation_matches(self, code, slot, generation);
    }
}

/// Free `Option<Box<[Option<T>]>>` slabs for dead code. Walks the outer Vec by
/// `code_index` and clears slabs whose `CodeRef` predicate returns `false`.
fn prune_dead_code_ic_slab<T>(
    slab: &mut [Option<Box<[Option<T>]>>],
    is_live: impl Fn(CodeRef) -> bool,
) {
    for (index, slot) in slab.iter_mut().enumerate() {
        if slot.is_none() {
            continue;
        }
        let Some(raw) = u32::try_from(index + 1).ok() else {
            continue;
        };
        let Some(code) = CodeRef::from_raw(raw) else {
            continue;
        };
        if !is_live(code) {
            *slot = None;
        }
    }
}

/// Variant of `prune_dead_code_ic_slab` that checks liveness via the `installed`
/// vector. A code is live iff `installed[code_index]` is `Some(_)`.
fn prune_dead_code_ic_slab_by_installed<T>(
    slab: &mut [Option<Box<[Option<T>]>>],
    installed: &[Option<std::sync::Arc<install::InstalledFunction>>],
) {
    for (index, slot) in slab.iter_mut().enumerate() {
        if slot.is_none() {
            continue;
        }
        let live = installed.get(index).is_some_and(Option::is_some);
        if !live {
            *slot = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_bytecode::{BytecodeBuilder, BytecodeFunctionKind};
    use lyng_env::Runtime;
    use lyng_host::NoopHostHooks;
    use lyng_objects::{FunctionObjectData, ObjectColdData};

    #[test]
    fn referrer_scopes_walk_returns_nearest_establishment() {
        let mut vm = Vm::new();
        let realm_a = RealmRef::from_raw(1).unwrap();
        let realm_b = RealmRef::from_raw(2).unwrap();
        assert_eq!(vm.current_referrer(), None);
        assert_eq!(vm.establishment_realm_covering(0), None);
        let a = AtomId::from_raw(10);
        vm.push_referrer_scope(0, realm_a, Some(a));
        assert_eq!(vm.current_referrer(), Some(a));
        // A scope at base_depth 0 covers every depth at or above it.
        assert_eq!(vm.establishment_realm_covering(0), Some(realm_a));
        assert_eq!(vm.establishment_realm_covering(5), Some(realm_a));
        let b = AtomId::from_raw(20);
        vm.push_referrer_scope(2, realm_b, Some(b));
        assert_eq!(vm.current_referrer(), Some(b));
        // Depth-relative: a frame below the depth-2 establishment still sees the
        // depth-0 scope's realm; one at/above depth 2 sees the newer scope.
        assert_eq!(vm.establishment_realm_covering(1), Some(realm_a));
        assert_eq!(vm.establishment_realm_covering(2), Some(realm_b));
        assert_eq!(vm.establishment_realm_covering(3), Some(realm_b));
        vm.unwind_referrer_scopes_to(1); // drops the depth-2 scope
        assert_eq!(vm.current_referrer(), Some(a));
        assert_eq!(vm.establishment_realm_covering(3), Some(realm_a));
    }

    /// Verify the arena `FrameHeader` overlaid at `current_cfr` mirrors the
    /// `FrameRecord` for a real frame pushed through the install path.
    #[test]
    fn arena_header_overlay_mirrors_the_record_at_entry() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .expect("default realm should exist after boot");
        let global_env = realm.global_env();
        let root_shape = realm
            .root_shape()
            .expect("default realm should expose a root shape");

        // A trivial bytecode function (no parameters, no environment).
        let function = BytecodeBuilder::new(
            BytecodeFunctionId::from_raw(31).unwrap(),
            BytecodeFunctionKind::Function,
        )
        .finish()
        .expect("test bytecode should build");
        let unit = CompiledFunctionUnit::new(SourceId::new(131), function.id(), vec![function]);

        let mut vm = Vm::new();
        let installed = vm
            .install_function(agent, realm.id(), &unit)
            .expect("function unit should install");
        let callee = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::function(root_shape).with_cold_data(ObjectColdData::Function(
                    FunctionObjectData::bytecode(realm.id(), global_env, installed.code()),
                )),
                AllocationLifetime::Default,
            )
        });

        // Seed a caller frame (through the real reservation path) so the caller's
        // lexical_env is available and `current_realm_of` resolves the active realm.
        vm.push_test_root_frame(agent, 1, &[Value::undefined(); 1], |window| {
            FrameRecord::new(
                installed.code(),
                0,
                window,
                None,
                global_env,
                global_env,
                ExecutionContextKind::Function,
            )
        });
        let caller_frame = vm.frame().expect("test root frame should be active");

        let prepared = vm
            .prepare_bytecode_call(
                agent,
                caller_frame.lexical_env(),
                callee,
                Value::undefined(),
                None,
            )
            .expect("bytecode call should prepare");
        vm.install_prepared_bytecode_call(agent, prepared, &[], Some(0), None, false)
            .expect("bytecode call should install");

        let record = vm.frame().expect("install should push a callee frame");
        let header = vm
            .current_frame_header()
            .expect("a live frame must have an overlaid header at current_cfr");

        assert_eq!(header.code(), record.code(), "header code mirrors record");
        // `window_len_for` derives the same register-window length the push path
        // reserved straight from the frame's code (no record needed).
        assert_eq!(
            vm.window_len_for(record.code()),
            record.registers().len(),
            "window_len_for(code) equals the reserved window length",
        );
        assert_eq!(
            header.callee(),
            record.callee(),
            "header callee mirrors record"
        );
        assert_eq!(
            header.variable_env(),
            record.variable_env(),
            "header variable_env mirrors record"
        );
        assert_eq!(
            header.lexical_env(),
            record.lexical_env(),
            "header lexical_env mirrors record"
        );
        assert_eq!(
            header.this_value(),
            record.this_value(),
            "header this_value mirrors record"
        );
        assert_eq!(
            header.this_state(),
            record.this_state(),
            "header this_state mirrors record"
        );
        // The cfr/window invariant: window base sits HEADER_SLOTS above the cfr.
        assert_eq!(
            Vm::cfr_of(&record),
            vm.current_cfr,
            "current_cfr equals the active frame's cfr (window base - HEADER_SLOTS)"
        );

        // Cold-slot depth convention: the top frame's cold state lives at
        // `frame_depth() - 1` and was seeded from the record on push.
        let top_depth = vm.frame_depth() - 1;
        let cold = vm.frame_cold.get(top_depth);
        assert_eq!(
            cold.handler_cursor,
            record.handler_cursor(),
            "cold handler_cursor seeded from record at the top frame's depth"
        );
        assert_eq!(
            cold.parameter_initializer_end_offset,
            record.parameter_initializer_end_offset(),
            "cold parameter_initializer_end_offset seeded from record at the top frame's depth"
        );

        // The callee's caller_cfr chains down to the test root frame's cfr.
        assert_eq!(
            header.caller_cfr(),
            Some(0),
            "callee caller_cfr points at the root frame reserved at arena base 0"
        );
    }

    /// `frame_window_len(cfr)` must be exact for every live frame. For a normal
    /// bytecode frame it equals `window_len_for(code)`. For the synthetic job-root
    /// frame — which reserves a 0-width window yet borrows a non-zero-register
    /// `CodeRef` — it must report 0, not `window_len_for(its code)`. Without the
    /// Job special-case the GC arena walk would over-trace header slots as Values.
    #[test]
    fn frame_window_len_is_zero_for_the_job_root_despite_borrowed_code() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .expect("default realm should exist after boot");
        let global_env = realm.global_env();

        // A function with a non-zero register window (3 visible registers): this is
        // the code the job root will borrow, so `window_len_for(it) > 0`.
        let mut builder = BytecodeBuilder::new(
            BytecodeFunctionId::from_raw(51).unwrap(),
            BytecodeFunctionKind::Function,
        );
        builder
            .try_alloc_registers(3)
            .expect("three registers should allocate");
        let function = builder.finish().expect("test bytecode should build");
        let unit = CompiledFunctionUnit::new(SourceId::new(151), function.id(), vec![function]);

        let mut vm = Vm::new();
        let installed = vm
            .install_function(agent, realm.id(), &unit)
            .expect("function unit should install");
        assert_eq!(
            vm.window_len_for(installed.code()),
            3,
            "the borrowed code has a 3-register window",
        );

        // A normal bytecode frame running that code. `frame_window_len` equals both
        // `window_len_for(code)` and the reserved window length.
        let base_cfr = {
            let base_window =
                vm.push_test_root_frame(agent, 3, &[Value::undefined(); 3], |window| {
                    FrameRecord::new(
                        installed.code(),
                        0,
                        window,
                        None,
                        global_env,
                        global_env,
                        ExecutionContextKind::Function,
                    )
                });
            base_window - crate::frame_header::HEADER_SLOTS as u32
        };
        let base_record = vm.frame().expect("base frame should be active");
        assert_eq!(
            vm.frame_window_len(base_cfr),
            vm.window_len_for(base_record.code()),
            "normal frame: frame_window_len equals window_len_for(code)",
        );
        assert_eq!(
            vm.frame_window_len(base_cfr),
            base_record.registers().len(),
            "normal frame: frame_window_len equals the reserved window",
        );

        // Push a synthetic job-root frame the same way `run_microtask_job` does:
        // reserve a 0-width window, then build the frame via
        // `synthetic_job_caller_frame` (which borrows the live frame's code).
        let (job_cfr, job_window_base) = vm
            .reserve_frame(agent, 0)
            .expect("job-root reservation should fit");
        let job_root = vm
            .synthetic_job_caller_frame(&realm)
            .with_register_window(RegisterWindow::new(job_window_base, 0));
        assert_eq!(
            job_root.kind(),
            ExecutionContextKind::Job,
            "synthetic job caller is a Job-kind frame",
        );
        // The job root borrowed the base frame's non-zero-register code.
        assert_eq!(
            vm.window_len_for(job_root.code()),
            3,
            "job root borrows a 3-register code, so window_len_for(code) is 3, NOT 0",
        );
        vm.push_frame_with_header(job_cfr, job_root);

        // The fix: bounded by the ACTUAL reserved window (0), not the borrowed code.
        assert_eq!(
            vm.frame_window_len(job_cfr),
            0,
            "job-root frame_window_len must be 0 despite its 3-register borrowed code",
        );
        // The header decodes back to the Job kind that drives the special-case.
        assert_eq!(
            vm.frame_header(job_cfr).kind(),
            ExecutionContextKind::Job,
            "job-root header kind decodes to Job",
        );
    }

    /// `frame_depth()` tracks nesting and the `caller_cfr` chain bottoms out at
    /// the root frame.
    #[test]
    fn frame_depth_and_caller_walk_track_nested_calls() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .expect("default realm should exist after boot");
        let global_env = realm.global_env();
        let root_shape = realm
            .root_shape()
            .expect("default realm should expose a root shape");

        let function = BytecodeBuilder::new(
            BytecodeFunctionId::from_raw(41).unwrap(),
            BytecodeFunctionKind::Function,
        )
        .finish()
        .expect("test bytecode should build");
        let unit = CompiledFunctionUnit::new(SourceId::new(141), function.id(), vec![function]);

        let mut vm = Vm::new();
        let installed = vm
            .install_function(agent, realm.id(), &unit)
            .expect("function unit should install");
        let callee = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::function(root_shape).with_cold_data(ObjectColdData::Function(
                    FunctionObjectData::bytecode(realm.id(), global_env, installed.code()),
                )),
                AllocationLifetime::Default,
            )
        });

        // Push root frame (depth 1).
        vm.push_test_root_frame(agent, 1, &[Value::undefined(); 1], |window| {
            FrameRecord::new(
                installed.code(),
                0,
                window,
                None,
                global_env,
                global_env,
                ExecutionContextKind::Function,
            )
        });
        assert_eq!(vm.frame_depth(), 1, "root frame pushed: depth == 1");

        // Push callee frame (depth 2).
        let caller_frame = vm.frame().expect("root frame should be active");
        let prepared = vm
            .prepare_bytecode_call(
                agent,
                caller_frame.lexical_env(),
                callee,
                Value::undefined(),
                None,
            )
            .expect("bytecode call should prepare");
        vm.install_prepared_bytecode_call(agent, prepared, &[], Some(0), None, false)
            .expect("bytecode call should install");

        assert!(vm.frame_depth() >= 2, "callee pushed: depth >= 2");

        // Walk the caller_cfr chain from current_cfr_opt; expect exactly one step to root.
        let mut cfr = vm.current_cfr_opt().expect("a live frame must exist");
        let mut steps = 0usize;
        while let Some(caller) = vm.frame_header(cfr).caller_cfr() {
            cfr = caller;
            steps += 1;
        }
        assert!(
            steps >= 1,
            "caller chain must bottom out at the root (>= 1 step)"
        );
    }
}
