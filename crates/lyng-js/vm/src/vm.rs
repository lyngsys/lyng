use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use lyng_js_builtins::{
    bootstrap_realm, BootstrapArtifacts, BootstrapMode, BootstrapRequest, BuiltinCache,
};
use lyng_js_bytecode::{
    ArgumentsMode, BytecodeEnvironmentBinding, BytecodeFunction, BytecodeFunctionId, CallRange,
    CompiledAtom, CompiledFunctionUnit, CompiledScriptUnit, ConstantValue, DeoptSnapshot,
    GlobalScriptInstantiationPlan, Instruction, Opcode, SafepointDescriptor, SourceMapEntry,
};
use lyng_js_common::{AtomId, Diagnostic, SourceId, WellKnownAtom};
use lyng_js_compiler::{
    compile_module, dynamic::DynamicFunctionCacheKey, CompiledModuleUnit,
    ModuleImportKind as CompiledModuleImportKind, ModuleRequestPhase as CompiledModuleRequestPhase,
};
use lyng_js_env::{
    Agent, EnvironmentBindingLayout, EnvironmentLayout, EnvironmentLayoutId, EnvironmentLayoutKind,
    EnvironmentSlotFlags, ExecutionContext, ModuleBindingAlias, ModuleImportEntry,
    ModuleImportKind, ModuleIndirectExportEntry, ModuleLocalExportEntry, ModuleRecord,
    ModuleRequestPhase, ModuleRequestRecord, ModuleResolvedExport, ModuleResolvedExportTarget,
    ModuleStarExportEntry, ModuleStatus, RealmRecord, ThisBindingStatus, ThisState,
};
use lyng_js_gc::{AllocationLifetime, PrimitiveCollectionReport, PrimitiveTracer, TraceHeapEdges};
use lyng_js_host::{
    DiagnosticReportRequest, HostHooks, ImportMetaRequest, ModuleKey, ModuleSourceRequest,
    NoopHostHooks,
};
use lyng_js_objects::{
    ModuleNamespaceExport, ModuleNamespaceExportTarget, NativeFunctionRegistry, ObjectAllocation,
};
use lyng_js_ops::errors;
use lyng_js_parser::parse_module;
use lyng_js_sema::analyze_module;
use lyng_js_types::{
    AbruptCompletion, BuiltinFunctionId, CodeRef, EnvironmentRef, ObjectRef, PropertyDescriptor,
    PropertyKey, RealmRef, Value, WellKnownSymbolId,
};

use crate::activation::ActivationSideTables;
use crate::enumeration::{ForInStateTable, IteratorStateTable};
use crate::error::{ModuleLoadError, VmResult};
use crate::extensions::{RealmExtensionInstallation, SharedRealmExtensionProvider};
use crate::name_refs::{CapturedNameReference, CapturedNameReferenceTable};
use crate::{FrameFlags, FrameRecord, InstalledCode, RegisterWindow, VmError};

mod activation_objects;
mod async_functions;
mod builtin_dispatch;
mod bytecode_calls;
mod call;
mod direct_eval_env;
mod dispatch;
mod dynamic_compilation;
mod exceptions;
mod feedback;
mod generators;
mod global_script;
mod install;
mod internal_calls;
mod jobs;
mod loop_iteration;
mod names;
mod property_access;
mod registers;
mod runtime_objects;
mod tiering;
mod values;
mod with_env;

use call::RejectingNativeRegistry;
use feedback::FeedbackVector;
use install::InstalledFunction;
use tiering::TieringState;
use values::{bytecode_index, code_index, decode_env_operand, string_text_array_index};

pub use feedback::{
    FeedbackInlineCacheState, FeedbackKeyedPropertyFamily, FeedbackSiteDetail,
    FeedbackSiteSnapshot, FeedbackVectorSnapshot, KeyedNamedPropertyCacheEntrySnapshot,
    KeyedPropertyFeedbackSnapshot, NamedPropertyCacheEntrySnapshot, NamedPropertyFeedbackSnapshot,
};
pub use tiering::{TierStatus, TieringSnapshot};

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedModuleRoot {
    key: ModuleKey,
    display_name: Box<str>,
}

impl LoadedModuleRoot {
    #[inline]
    pub fn new(key: ModuleKey, display_name: impl Into<Box<str>>) -> Self {
        Self {
            key,
            display_name: display_name.into(),
        }
    }

    #[inline]
    pub const fn key(&self) -> &ModuleKey {
        &self.key
    }

    #[inline]
    pub fn display_name(&self) -> &str {
        &self.display_name
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct TemplateCacheKey {
    realm: RealmRef,
    code: CodeRef,
    site: u32,
}

/// Runtime state for one compiler-planned loop-iteration lexical environment.
///
/// `iteration_slots` mirror per-iteration bindings while the loop body is
/// active unless they are detached normal-for copies. `shared_slots` continue
/// to alias the source environment after the per-iteration environment is
/// retained by a closure.
#[derive(Clone, Debug, PartialEq, Eq)]
struct LoopIterationEnvironment {
    frame_depth: usize,
    source_environment: EnvironmentRef,
    iteration_environment: EnvironmentRef,
    iteration_slots: Vec<u32>,
    shared_slots: Vec<u32>,
    detached_slots: Vec<u32>,
    active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WithEnvironmentState {
    frame_depth: usize,
    previous_lexical_env: EnvironmentRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DirectEvalEnvironmentState {
    frame_depth: usize,
    environment: EnvironmentRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AsyncFrameState {
    capability: lyng_js_env::PromiseCapabilityId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AsyncGeneratorRequest {
    kind: crate::frame::GeneratorResumeKind,
    value: Value,
    capability: lyng_js_env::PromiseCapabilityId,
    realm: RealmRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AsyncGeneratorFrameState {
    generator: ObjectRef,
    capability: lyng_js_env::PromiseCapabilityId,
    realm: RealmRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EntryExecutionOverride {
    this_value: Value,
    new_target: Option<ObjectRef>,
    home_object: Option<ObjectRef>,
    active_function: Option<ObjectRef>,
    private_env: Option<EnvironmentRef>,
    lexical_this: bool,
}

struct ActiveVmRoots<'a> {
    vm: &'a Vm,
    caller_frame: FrameRecord,
}

impl TraceHeapEdges for TemplateCacheKey {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.realm.trace_heap_edges(tracer);
        self.code.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for LoopIterationEnvironment {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.source_environment.trace_heap_edges(tracer);
        self.iteration_environment.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for WithEnvironmentState {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.previous_lexical_env.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for DirectEvalEnvironmentState {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.environment.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for AsyncGeneratorRequest {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.value.trace_heap_edges(tracer);
        self.realm.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for AsyncGeneratorFrameState {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.generator.trace_heap_edges(tracer);
        self.realm.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for EntryExecutionOverride {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.this_value.trace_heap_edges(tracer);
        self.new_target.trace_heap_edges(tracer);
        self.home_object.trace_heap_edges(tracer);
        self.active_function.trace_heap_edges(tracer);
        self.private_env.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for ActiveVmRoots<'_> {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        trace_frame_record(self.caller_frame, tracer);

        for value in &self.vm.register_stack {
            value.trace_heap_edges(tracer);
        }
        for frame in &self.vm.frames {
            trace_frame_record(*frame, tracer);
        }
        self.vm.current_exception.trace_heap_edges(tracer);

        for installed in self.vm.installed.iter().flatten() {
            for code in &installed.child_codes {
                code.trace_heap_edges(tracer);
            }
        }
        self.vm.builtin_cache.trace_heap_edges(tracer);
        for (key, value) in &self.vm.template_cache {
            key.trace_heap_edges(tracer);
            value.trace_heap_edges(tracer);
        }
        for code in self.vm.dynamic_function_cache.values() {
            code.code().trace_heap_edges(tracer);
        }
        for state in self.vm.suspended_side_states.values() {
            for (_, iterator) in &state.iterator_states {
                iterator.trace_heap_edges(tracer);
            }
            for (_, enumerator) in &state.for_in_states {
                enumerator.trace_heap_edges(tracer);
            }
            for state in &state.loop_iteration_envs {
                state.trace_heap_edges(tracer);
            }
            for state in &state.with_environment_states {
                state.trace_heap_edges(tracer);
            }
            for state in &state.direct_eval_environment_states {
                state.trace_heap_edges(tracer);
            }
            for state in &state.active_env_scopes {
                state.environment.trace_heap_edges(tracer);
            }
            state.async_generator_frame_state.trace_heap_edges(tracer);
        }
        for state in self.vm.async_generator_frame_states.values() {
            state.trace_heap_edges(tracer);
        }
        for object in &self.vm.async_generator_objects {
            object.trace_heap_edges(tracer);
        }
        for (object, queue) in &self.vm.async_generator_queues {
            object.trace_heap_edges(tracer);
            for request in queue {
                request.trace_heap_edges(tracer);
            }
        }
        for object in self.vm.deferred_module_namespaces.keys() {
            object.trace_heap_edges(tracer);
        }
        for state in &self.vm.loop_iteration_envs {
            state.trace_heap_edges(tracer);
        }
        for environment in &self.vm.loop_iteration_source_scratch {
            environment.trace_heap_edges(tracer);
        }
        for environment in &self.vm.loop_iteration_target_scratch {
            environment.trace_heap_edges(tracer);
        }
        for state in &self.vm.with_environment_states {
            state.trace_heap_edges(tracer);
        }
        for state in &self.vm.direct_eval_environment_states {
            state.trace_heap_edges(tracer);
        }
        for state in &self.vm.active_env_scopes {
            state.environment.trace_heap_edges(tracer);
        }
        for (overlay, source) in &self.vm.direct_eval_environment_overlays {
            overlay.trace_heap_edges(tracer);
            source.trace_heap_edges(tracer);
        }
        for value in &self.vm.argument_scratch {
            value.trace_heap_edges(tracer);
        }
    }
}

fn trace_frame_record(frame: FrameRecord, tracer: &mut PrimitiveTracer<'_>) {
    frame.code().trace_heap_edges(tracer);
    frame.realm().trace_heap_edges(tracer);
    frame.lexical_env().trace_heap_edges(tracer);
    frame.variable_env().trace_heap_edges(tracer);
    frame.this_value().trace_heap_edges(tracer);
    frame.construct_this().trace_heap_edges(tracer);
    frame.new_target().trace_heap_edges(tracer);
    frame.callee().trace_heap_edges(tracer);
    frame.tail_caller().trace_heap_edges(tracer);
    frame.resume_value().trace_heap_edges(tracer);
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DynamicImportRequest {
    capability: lyng_js_env::PromiseCapabilityId,
    request: ModuleSourceRequest,
    phase: DynamicImportPhase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::vm) enum DynamicImportPhase {
    Evaluation,
    Source,
    Defer,
}

impl DynamicImportPhase {
    pub(in crate::vm) fn from_value(value: Option<Value>) -> Self {
        match value.and_then(Value::as_smi) {
            Some(1) => Self::Source,
            Some(2) => Self::Defer,
            _ => Self::Evaluation,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PendingDynamicImport {
    capability: lyng_js_env::PromiseCapabilityId,
    realm: RealmRef,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SuspendedExecutionSideState {
    iterator_states: Vec<(u16, lyng_js_ops::iterator::IteratorRecord)>,
    for_in_states: Vec<(u16, lyng_js_ops::enumeration::ForInEnumerator)>,
    captured_name_references: Vec<(u16, CapturedNameReference)>,
    loop_iteration_envs: Vec<LoopIterationEnvironment>,
    with_environment_states: Vec<WithEnvironmentState>,
    direct_eval_environment_states: Vec<DirectEvalEnvironmentState>,
    active_env_scopes: Vec<ActiveEnvScopeRange>,
    async_frame_state: Option<AsyncFrameState>,
    async_generator_frame_state: Option<AsyncGeneratorFrameState>,
    script_or_module_referrer: Option<AtomId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActiveEnvScopeRange {
    frame_depth: usize,
    environment: EnvironmentRef,
    start: u32,
    end: u32,
}

impl ActiveEnvScopeRange {
    const fn new(frame_depth: usize, environment: EnvironmentRef, start: u32, count: u32) -> Self {
        Self {
            frame_depth,
            environment,
            start,
            end: start.saturating_add(count),
        }
    }

    fn contains(self, environment: EnvironmentRef, slot: u32) -> bool {
        self.environment == environment && self.start <= slot && slot < self.end
    }
}

#[derive(Default)]
pub struct Vm {
    register_stack: Vec<Value>,
    frames: Vec<FrameRecord>,
    installed: Vec<Option<Arc<InstalledFunction>>>,
    current_exception: Option<Value>,
    atom_texts: HashMap<AtomId, Box<str>>,
    preferred_atoms_by_text: HashMap<Box<str>, AtomId>,
    source_texts: HashMap<SourceId, Arc<str>>,
    feedback_warmup: Vec<u16>,
    feedback_vectors: Vec<Option<FeedbackVector>>,
    tiering: Vec<Option<TieringState>>,
    activation_tables: ActivationSideTables,
    for_in_states: ForInStateTable,
    iterator_states: IteratorStateTable,
    captured_name_references: CapturedNameReferenceTable,
    builtin_cache: BuiltinCache,
    template_cache: HashMap<TemplateCacheKey, ObjectRef>,
    dynamic_function_cache: HashMap<DynamicFunctionCacheKey, InstalledCode>,
    suspended_side_states:
        HashMap<lyng_js_types::SuspendedExecutionRef, SuspendedExecutionSideState>,
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

impl Vm {
    #[inline]
    pub fn new() -> Self {
        Self {
            register_stack: Vec::new(),
            frames: Vec::new(),
            installed: Vec::new(),
            current_exception: None,
            atom_texts: HashMap::new(),
            preferred_atoms_by_text: HashMap::new(),
            source_texts: HashMap::new(),
            feedback_warmup: Vec::new(),
            feedback_vectors: Vec::new(),
            tiering: Vec::new(),
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

    #[inline]
    pub fn register_stack(&self) -> &[Value] {
        &self.register_stack
    }

    #[inline]
    pub fn frames(&self) -> &[FrameRecord] {
        &self.frames
    }

    #[inline]
    pub fn frame(&self) -> Option<FrameRecord> {
        self.frames.last().copied()
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
        let start =
            usize::try_from(register_base).expect("register stack base should fit into usize");
        debug_assert_eq!(self.register_stack.len(), start);
        self.register_stack
            .resize(start + usize::from(register_len), Value::undefined());
    }

    #[inline]
    pub const fn current_exception(&self) -> Option<Value> {
        self.current_exception
    }

    pub(crate) fn force_collect_with_active_roots(
        &self,
        agent: &mut Agent,
        caller_frame: FrameRecord,
    ) -> PrimitiveCollectionReport {
        agent.force_collect_with_additional_roots(&ActiveVmRoots {
            vm: self,
            caller_frame,
        })
    }

    #[inline]
    #[allow(clippy::needless_pass_by_ref_mut)]
    #[cfg_attr(
        not(test),
        expect(
            clippy::unused_self,
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
        lyng_js_builtins::bootstrap_realm(
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
                    lyng_js_builtins::BuiltinBootstrapError::MissingRealm(realm),
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

    /// # Errors
    ///
    /// Returns a VM error if module function installation or module-record creation fails.
    pub fn install_module(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        key: &ModuleKey,
        display_name: &str,
        unit: &CompiledModuleUnit,
    ) -> VmResult<InstalledCode> {
        self.record_source_text(unit.source(), unit.source_text());
        let installed =
            self.install_functions(agent, realm, unit.entry(), unit.functions(), unit.atoms())?;
        let mut record = compiled_module_record(self, installed, key, display_name, unit);
        record.set_code(Some(installed.code()));
        record.set_status(ModuleStatus::Unlinked);
        let _ = agent.install_module_record(record);
        Ok(installed)
    }

    /// # Errors
    ///
    /// Returns a module-load error if host loading, diagnostics, bootstrap, or VM installation fails.
    pub fn load_module_graph_from_host(
        &mut self,
        agent: &mut Agent,
        realm: &RealmRecord,
        host: &dyn HostHooks,
        request: &ModuleSourceRequest,
    ) -> Result<LoadedModuleRoot, ModuleLoadError> {
        let _ = self
            .bootstrap_realm(agent, realm.id(), BootstrapMode::SpecOnly)
            .map_err(ModuleLoadError::Vm)?;
        self.install_active_realm_extensions(agent, realm.id())
            .map_err(ModuleLoadError::Vm)?;
        let loaded = host
            .load_module_source(request)
            .map_err(ModuleLoadError::Host)?;
        self.ensure_module_loaded_from_host(agent, realm, host, loaded)
    }

    /// # Errors
    ///
    /// Returns a module-load error if host loading, diagnostics, bootstrap, extension installation,
    /// or VM installation fails.
    pub fn load_module_graph_from_host_and_extensions(
        &mut self,
        agent: &mut Agent,
        realm: &RealmRecord,
        host: &dyn HostHooks,
        request: &ModuleSourceRequest,
        provider: Option<&SharedRealmExtensionProvider>,
    ) -> Result<LoadedModuleRoot, ModuleLoadError> {
        match provider {
            Some(provider) => self.with_extension_provider(provider, |vm| {
                vm.load_module_graph_from_host(agent, realm, host, request)
            }),
            None => self.load_module_graph_from_host(agent, realm, host, request),
        }
    }

    /// # Errors
    ///
    /// Returns a VM error if script installation, instantiation, extension setup, evaluation, or job
    /// checkpointing fails.
    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub fn evaluate_script_with_registry_and_host_referrer_and_extensions(
        &mut self,
        agent: &mut Agent,
        realm: RealmRecord,
        unit: &CompiledScriptUnit,
        script_referrer: Option<&ModuleKey>,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        provider: Option<&SharedRealmExtensionProvider>,
    ) -> VmResult<Value> {
        match provider {
            Some(provider) => self.with_extension_provider(provider, |vm| {
                vm.evaluate_script_with_registry_and_host_referrer(
                    agent,
                    realm,
                    unit,
                    script_referrer,
                    host,
                    registry,
                )
            }),
            None => self.evaluate_script_with_registry_and_host_referrer(
                agent,
                realm,
                unit,
                script_referrer,
                host,
                registry,
            ),
        }
    }

    /// # Errors
    ///
    /// Returns a VM error if script installation, instantiation, extension setup, evaluation, or job
    /// checkpointing fails.
    pub fn evaluate_script_with_host_referrer_and_extensions(
        &mut self,
        agent: &mut Agent,
        realm: RealmRecord,
        unit: &CompiledScriptUnit,
        script_referrer: Option<&ModuleKey>,
        host: &dyn HostHooks,
        provider: Option<&SharedRealmExtensionProvider>,
    ) -> VmResult<Value> {
        let mut registry = RejectingNativeRegistry;
        self.evaluate_script_with_registry_and_host_referrer_and_extensions(
            agent,
            realm,
            unit,
            script_referrer,
            host,
            &mut registry,
            provider,
        )
    }

    /// # Errors
    ///
    /// Returns a VM error if script installation, instantiation, extension setup, evaluation, or job
    /// checkpointing fails.
    pub fn evaluate_script_with_host_referrer_and_extensions_retaining_installed(
        &mut self,
        agent: &mut Agent,
        realm: RealmRecord,
        unit: &CompiledScriptUnit,
        script_referrer: Option<&ModuleKey>,
        host: &dyn HostHooks,
        provider: Option<&SharedRealmExtensionProvider>,
    ) -> VmResult<(Value, InstalledCode)> {
        match provider {
            Some(provider) => self.with_extension_provider(provider, |vm| {
                vm.evaluate_script_with_host_referrer_retaining_installed(
                    agent,
                    &realm,
                    unit,
                    script_referrer,
                    host,
                )
            }),
            None => self.evaluate_script_with_host_referrer_retaining_installed(
                agent,
                &realm,
                unit,
                script_referrer,
                host,
            ),
        }
    }

    /// # Errors
    ///
    /// Returns a VM error if module bootstrap, extension setup, installation, linking, evaluation, or
    /// job checkpointing fails.
    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub fn evaluate_module_with_registry_and_host_and_extensions(
        &mut self,
        agent: &mut Agent,
        realm: RealmRecord,
        key: &ModuleKey,
        display_name: &str,
        unit: &CompiledModuleUnit,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        provider: Option<&SharedRealmExtensionProvider>,
    ) -> VmResult<Value> {
        match provider {
            Some(provider) => self.with_extension_provider(provider, |vm| {
                vm.evaluate_module_with_registry_and_host(
                    agent,
                    realm,
                    key,
                    display_name,
                    unit,
                    host,
                    registry,
                )
            }),
            None => self.evaluate_module_with_registry_and_host(
                agent,
                realm,
                key,
                display_name,
                unit,
                host,
                registry,
            ),
        }
    }

    /// # Errors
    ///
    /// Returns a VM error if extension setup, module linking, evaluation, or job checkpointing fails.
    pub fn evaluate_linked_module_with_registry_and_host_and_extensions(
        &mut self,
        agent: &mut Agent,
        realm: RealmRecord,
        key: &ModuleKey,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        provider: Option<&SharedRealmExtensionProvider>,
    ) -> VmResult<Value> {
        match provider {
            Some(provider) => self.with_extension_provider(provider, |vm| {
                vm.evaluate_linked_module_with_registry_and_host(agent, realm, key, host, registry)
            }),
            None => self
                .evaluate_linked_module_with_registry_and_host(agent, realm, key, host, registry),
        }
    }

    /// # Errors
    ///
    /// Returns a VM error if extension setup, module linking, evaluation, or job checkpointing fails.
    pub fn evaluate_linked_module_with_host_and_extensions(
        &mut self,
        agent: &mut Agent,
        realm: RealmRecord,
        key: &ModuleKey,
        host: &dyn HostHooks,
        provider: Option<&SharedRealmExtensionProvider>,
    ) -> VmResult<Value> {
        let mut registry = RejectingNativeRegistry;
        self.evaluate_linked_module_with_registry_and_host_and_extensions(
            agent,
            realm,
            key,
            host,
            &mut registry,
            provider,
        )
    }

    /// # Errors
    ///
    /// Returns a VM error if entering the installed function, execution, or job checkpointing fails.
    pub fn evaluate_installed(
        &mut self,
        agent: &mut Agent,
        installed: InstalledCode,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) -> VmResult<Value> {
        let mut registry = RejectingNativeRegistry;
        self.evaluate_installed_with_registry_and_host(
            agent,
            installed,
            lexical_env,
            variable_env,
            None,
            &NoopHostHooks,
            &mut registry,
        )
    }

    /// # Errors
    ///
    /// Returns a VM error if script installation, bootstrap, instantiation, execution, or job
    /// checkpointing fails.
    pub fn evaluate_script(
        &mut self,
        agent: &mut Agent,
        realm: RealmRecord,
        unit: &CompiledScriptUnit,
    ) -> VmResult<Value> {
        let installed = self.install_script(agent, realm.id(), unit)?;
        let _ = self.bootstrap_realm(agent, realm.id(), BootstrapMode::SpecOnly)?;
        self.install_active_realm_extensions(agent, realm.id())?;
        Self::instantiate_global_script(agent, &realm, unit.instantiation_plan())?;
        let mut registry = RejectingNativeRegistry;
        self.evaluate_entry_with_registry_and_checkpoint(
            agent,
            installed,
            realm.global_env(),
            realm.global_env(),
            None,
            &NoopHostHooks,
            &mut registry,
            Some(unit.instantiation_plan()),
            None,
        )
    }

    /// # Errors
    ///
    /// Returns a VM error if script installation, bootstrap, instantiation, execution, or job
    /// checkpointing fails.
    pub fn evaluate_script_with_host(
        &mut self,
        agent: &mut Agent,
        realm: RealmRecord,
        unit: &CompiledScriptUnit,
        host: &dyn HostHooks,
    ) -> VmResult<Value> {
        let mut registry = RejectingNativeRegistry;
        self.evaluate_script_with_registry_and_host(agent, realm, unit, host, &mut registry)
    }

    /// # Errors
    ///
    /// Returns a VM error if script installation, bootstrap, instantiation, execution, or job
    /// checkpointing fails.
    pub fn evaluate_script_with_host_referrer(
        &mut self,
        agent: &mut Agent,
        realm: RealmRecord,
        unit: &CompiledScriptUnit,
        script_referrer: Option<&ModuleKey>,
        host: &dyn HostHooks,
    ) -> VmResult<Value> {
        let mut registry = RejectingNativeRegistry;
        self.evaluate_script_with_registry_and_host_referrer(
            agent,
            realm,
            unit,
            script_referrer,
            host,
            &mut registry,
        )
    }

    /// # Errors
    ///
    /// Returns a VM error if script installation, bootstrap, instantiation, execution, or job
    /// checkpointing fails.
    pub fn evaluate_script_with_registry(
        &mut self,
        agent: &mut Agent,
        realm: RealmRecord,
        unit: &CompiledScriptUnit,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> VmResult<Value> {
        self.evaluate_script_with_registry_and_host(agent, realm, unit, &NoopHostHooks, registry)
    }

    /// # Errors
    ///
    /// Returns a VM error if script installation, bootstrap, instantiation, execution, or job
    /// checkpointing fails.
    pub fn evaluate_script_with_registry_and_host(
        &mut self,
        agent: &mut Agent,
        realm: RealmRecord,
        unit: &CompiledScriptUnit,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> VmResult<Value> {
        self.evaluate_script_with_registry_and_host_referrer(
            agent, realm, unit, None, host, registry,
        )
    }

    /// # Errors
    ///
    /// Returns a VM error if script installation, bootstrap, instantiation, execution, or job
    /// checkpointing fails.
    pub fn evaluate_script_with_registry_and_host_referrer(
        &mut self,
        agent: &mut Agent,
        realm: RealmRecord,
        unit: &CompiledScriptUnit,
        script_referrer: Option<&ModuleKey>,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> VmResult<Value> {
        let installed = self.install_script(agent, realm.id(), unit)?;
        let _ = self.bootstrap_realm(agent, realm.id(), BootstrapMode::SpecOnly)?;
        self.install_active_realm_extensions(agent, realm.id())?;
        Self::instantiate_global_script(agent, &realm, unit.instantiation_plan())?;
        let script_referrer =
            script_referrer.map(|key| agent.atoms_mut().intern_collectible(key.as_str()));
        self.evaluate_entry_with_registry_and_checkpoint(
            agent,
            installed,
            realm.global_env(),
            realm.global_env(),
            script_referrer,
            host,
            registry,
            Some(unit.instantiation_plan()),
            None,
        )
    }

    fn evaluate_script_with_host_referrer_retaining_installed(
        &mut self,
        agent: &mut Agent,
        realm: &RealmRecord,
        unit: &CompiledScriptUnit,
        script_referrer: Option<&ModuleKey>,
        host: &dyn HostHooks,
    ) -> VmResult<(Value, InstalledCode)> {
        let installed = self.install_script(agent, realm.id(), unit)?;
        let _ = self.bootstrap_realm(agent, realm.id(), BootstrapMode::SpecOnly)?;
        self.install_active_realm_extensions(agent, realm.id())?;
        Self::instantiate_global_script(agent, realm, unit.instantiation_plan())?;
        let script_referrer =
            script_referrer.map(|key| agent.atoms_mut().intern_collectible(key.as_str()));
        let mut registry = RejectingNativeRegistry;
        let value = self.evaluate_entry_with_registry_and_checkpoint(
            agent,
            installed,
            realm.global_env(),
            realm.global_env(),
            script_referrer,
            host,
            &mut registry,
            Some(unit.instantiation_plan()),
            None,
        )?;
        Ok((value, installed))
    }

    /// # Errors
    ///
    /// Returns a VM error if module bootstrap, installation, linking, evaluation, or job
    /// checkpointing fails.
    pub fn evaluate_module(
        &mut self,
        agent: &mut Agent,
        realm: RealmRecord,
        key: &ModuleKey,
        display_name: &str,
        unit: &CompiledModuleUnit,
    ) -> VmResult<Value> {
        let mut registry = RejectingNativeRegistry;
        self.evaluate_module_with_registry(agent, realm, key, display_name, unit, &mut registry)
    }

    /// # Errors
    ///
    /// Returns a VM error if module bootstrap, installation, linking, evaluation, or job
    /// checkpointing fails.
    pub fn evaluate_module_with_registry(
        &mut self,
        agent: &mut Agent,
        realm: RealmRecord,
        key: &ModuleKey,
        display_name: &str,
        unit: &CompiledModuleUnit,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> VmResult<Value> {
        self.evaluate_module_with_registry_and_host(
            agent,
            realm,
            key,
            display_name,
            unit,
            &NoopHostHooks,
            registry,
        )
    }

    /// # Errors
    ///
    /// Returns a VM error if module bootstrap, installation, linking, evaluation, or job
    /// checkpointing fails.
    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub fn evaluate_module_with_registry_and_host(
        &mut self,
        agent: &mut Agent,
        realm: RealmRecord,
        key: &ModuleKey,
        display_name: &str,
        unit: &CompiledModuleUnit,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> VmResult<Value> {
        let _ = self.bootstrap_realm(agent, realm.id(), BootstrapMode::SpecOnly)?;
        self.install_active_realm_extensions(agent, realm.id())?;
        let _ = self.install_module(agent, realm.id(), key, display_name, unit)?;
        self.evaluate_linked_module_with_registry_and_host(agent, realm, key, host, registry)
    }

    /// # Errors
    ///
    /// Returns a VM error if bootstrap, extension setup, or module graph linking fails.
    pub fn link_module(
        &mut self,
        agent: &mut Agent,
        realm: RealmRecord,
        key: &ModuleKey,
    ) -> VmResult<EnvironmentRef> {
        let _ = self.bootstrap_realm(agent, realm.id(), BootstrapMode::SpecOnly)?;
        self.install_active_realm_extensions(agent, realm.id())?;
        self.link_module_graph(agent, &realm, key)
    }

    /// # Errors
    ///
    /// Returns a VM error if module linking, evaluation, or job checkpointing fails.
    pub fn evaluate_linked_module(
        &mut self,
        agent: &mut Agent,
        realm: RealmRecord,
        key: &ModuleKey,
    ) -> VmResult<Value> {
        let mut registry = RejectingNativeRegistry;
        self.evaluate_linked_module_with_registry(agent, realm, key, &mut registry)
    }

    /// # Errors
    ///
    /// Returns a VM error if module linking, evaluation, or job checkpointing fails.
    pub fn evaluate_linked_module_with_host(
        &mut self,
        agent: &mut Agent,
        realm: RealmRecord,
        key: &ModuleKey,
        host: &dyn HostHooks,
    ) -> VmResult<Value> {
        let mut registry = RejectingNativeRegistry;
        self.evaluate_linked_module_with_registry_and_host(agent, realm, key, host, &mut registry)
    }

    /// # Errors
    ///
    /// Returns a VM error if module linking, evaluation, or job checkpointing fails.
    pub fn evaluate_linked_module_with_registry(
        &mut self,
        agent: &mut Agent,
        realm: RealmRecord,
        key: &ModuleKey,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> VmResult<Value> {
        self.evaluate_linked_module_with_registry_and_host(
            agent,
            realm,
            key,
            &NoopHostHooks,
            registry,
        )
    }

    /// # Errors
    ///
    /// Returns a VM error if module linking, evaluation, or job checkpointing fails.
    pub fn evaluate_linked_module_with_registry_and_host(
        &mut self,
        agent: &mut Agent,
        realm: RealmRecord,
        key: &ModuleKey,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> VmResult<Value> {
        let _ = self.bootstrap_realm(agent, realm.id(), BootstrapMode::SpecOnly)?;
        self.install_active_realm_extensions(agent, realm.id())?;
        let module_env = self.link_module_graph(agent, &realm, key)?;
        let result =
            self.evaluate_module_graph(agent, &realm, key, module_env, host, registry, None, true);
        let result = match result {
            Ok(value) => {
                self.checkpoint_promise_jobs(agent, host, registry)?;
                Ok(value)
            }
            Err(error) => Err(error),
        };
        agent.clear_kept_objects();
        result
    }

    /// # Errors
    ///
    /// Returns a VM error if entering the installed function, execution, or job checkpointing fails.
    pub fn evaluate_installed_with_registry(
        &mut self,
        agent: &mut Agent,
        installed: InstalledCode,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> VmResult<Value> {
        self.evaluate_installed_with_registry_and_host(
            agent,
            installed,
            lexical_env,
            variable_env,
            None,
            &NoopHostHooks,
            registry,
        )
    }

    /// # Errors
    ///
    /// Returns a VM error if entering the installed function, execution, or job checkpointing fails.
    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub fn evaluate_installed_with_registry_and_host(
        &mut self,
        agent: &mut Agent,
        installed: InstalledCode,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
        script_or_module_referrer: Option<AtomId>,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> VmResult<Value> {
        self.evaluate_entry_with_registry_and_checkpoint(
            agent,
            installed,
            lexical_env,
            variable_env,
            script_or_module_referrer,
            host,
            registry,
            None,
            None,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(crate) fn evaluate_installed_with_registry_and_host_with_entry_override(
        &mut self,
        agent: &mut Agent,
        installed: InstalledCode,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
        script_or_module_referrer: Option<AtomId>,
        entry_this_value: Value,
        entry_new_target: Option<ObjectRef>,
        entry_home_object: Option<ObjectRef>,
        entry_active_function: Option<ObjectRef>,
        entry_private_env: Option<EnvironmentRef>,
        entry_lexical_this: bool,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> VmResult<Value> {
        self.evaluate_entry_with_registry_and_checkpoint(
            agent,
            installed,
            lexical_env,
            variable_env,
            script_or_module_referrer,
            host,
            registry,
            None,
            Some(EntryExecutionOverride {
                this_value: entry_this_value,
                new_target: entry_new_target,
                home_object: entry_home_object,
                active_function: entry_active_function,
                private_env: entry_private_env,
                lexical_this: entry_lexical_this,
            }),
        )
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
    ) -> VmResult<Value> {
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
        let result = match result {
            Ok(value) => {
                self.checkpoint_promise_jobs(agent, host, registry)?;
                Ok(value)
            }
            Err(error) => Err(error),
        };
        agent.clear_kept_objects();
        result
    }

    fn ensure_module_loaded_from_host(
        &mut self,
        agent: &mut Agent,
        realm: &RealmRecord,
        host: &dyn HostHooks,
        loaded: lyng_js_host::LoadedModuleSource,
    ) -> Result<LoadedModuleRoot, ModuleLoadError> {
        if agent.module_record(&loaded.key).is_none() {
            let source = self.allocate_dynamic_source_id();
            let parsed = parse_module(agent.atoms_mut(), source, &loaded.source_text);
            Self::report_module_diagnostics(host, parsed.diagnostics.as_slice())?;
            if parsed.diagnostics.has_errors() {
                return Err(ModuleLoadError::Parse);
            }

            let sema = analyze_module(&parsed, agent.atoms());
            Self::report_module_diagnostics(host, sema.diagnostics.as_slice())?;
            if sema.diagnostics.has_errors() {
                return Err(ModuleLoadError::Sema);
            }

            let unit = compile_module(&parsed, &sema, agent.atoms_mut())
                .map_err(|_| ModuleLoadError::Lowering)?;
            let _ = self
                .install_module(agent, realm.id(), &loaded.key, &loaded.display_name, &unit)
                .map_err(ModuleLoadError::Vm)?;

            for (index, request) in unit.requested_modules().iter().enumerate() {
                let dependency = host
                    .load_module_source(&ModuleSourceRequest {
                        specifier: request.specifier().to_owned(),
                        referrer: Some(loaded.key.clone()),
                        attributes: request.attributes().to_vec(),
                    })
                    .map_err(ModuleLoadError::Host)?;
                if !agent.set_module_requested_key(
                    &loaded.key,
                    u32::try_from(index).expect("module request index should fit into u32"),
                    Some(dependency.key.clone()),
                ) {
                    return Err(ModuleLoadError::Vm(VmError::MissingModuleRecord));
                }
                let _ = self.ensure_module_loaded_from_host(agent, realm, host, dependency)?;
            }
        }

        let import_meta = host
            .resolve_import_meta(&ImportMetaRequest {
                module: loaded.key.clone(),
            })
            .map_err(ModuleLoadError::Host)?;
        if !agent.set_module_record_import_meta_properties(&loaded.key, import_meta) {
            return Err(ModuleLoadError::Vm(VmError::MissingModuleRecord));
        }

        Ok(LoadedModuleRoot::new(loaded.key, loaded.display_name))
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
        reason = "spec-shaped VM algorithm stays contiguous until the VM module split issue extracts it"
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
            u32::try_from(self.register_stack.len()).expect("register stack length should fit u32");
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
        let context = if function.kind() == lyng_js_bytecode::BytecodeFunctionKind::Module {
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
            self.register_stack.truncate(
                usize::try_from(leaked.registers().base()).expect("base should fit usize"),
            );
        }
        self.register_stack.truncate(prior_register_len);
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
        if function.kind() == lyng_js_bytecode::BytecodeFunctionKind::Module {
            let this_value = Value::undefined();
            if !function.needs_environment() {
                return Ok((lexical_env, lexical_env, this_value, None));
            }
            if agent.module_environment(lexical_env).is_some() {
                return Ok((lexical_env, lexical_env, this_value, None));
            }
            let layout = function
                .environment_layout()
                .and_then(|layout| lyng_js_env::EnvironmentLayoutId::from_raw(layout.get()))
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
            .and_then(|layout| lyng_js_env::EnvironmentLayoutId::from_raw(layout.get()))
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
        if function.kind() == lyng_js_bytecode::BytecodeFunctionKind::Script
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

    fn allocate_module_entry_environment(
        &self,
        agent: &mut Agent,
        realm: &RealmRecord,
        installed: InstalledCode,
    ) -> VmResult<EnvironmentRef> {
        let function = self
            .installed_function(installed.code())
            .cloned()
            .ok_or_else(|| VmError::MissingInstalledCode(installed.code()))?;
        if !function.needs_environment() {
            return Ok(realm.global_env());
        }
        let layout = function
            .environment_layout()
            .and_then(|layout| lyng_js_env::EnvironmentLayoutId::from_raw(layout.get()))
            .ok_or_else(|| VmError::MissingEnvironmentLayout(installed.code()))?;
        agent
            .alloc_module_environment(
                Some(realm.global_env()),
                layout,
                AllocationLifetime::Default,
            )
            .ok_or_else(|| VmError::MissingEnvironmentLayout(installed.code()))
    }

    fn link_module_graph(
        &mut self,
        agent: &mut Agent,
        realm: &RealmRecord,
        key: &ModuleKey,
    ) -> VmResult<EnvironmentRef> {
        let (status, environment, code, requested_modules, import_entries) = {
            let record = agent
                .module_record(key)
                .ok_or(VmError::MissingModuleRecord)?;
            (
                record.status(),
                record.environment(),
                record.code(),
                record.requested_modules().to_vec(),
                record.import_entries().to_vec(),
            )
        };
        match status {
            ModuleStatus::Linked
            | ModuleStatus::Evaluating
            | ModuleStatus::Evaluated
            | ModuleStatus::Errored => {
                return environment.ok_or(VmError::MissingModuleEnvironment);
            }
            ModuleStatus::Linking => return environment.ok_or(VmError::MissingModuleEnvironment),
            ModuleStatus::New | ModuleStatus::Unlinked => {}
        }

        let code = code.ok_or(VmError::MissingModuleCode)?;
        let installed = InstalledCode::new(
            code,
            self.installed_function(code)
                .ok_or(VmError::MissingInstalledCode(code))?
                .id(),
        );
        let module_env = if let Some(environment) = environment {
            environment
        } else {
            let environment = self.allocate_module_entry_environment(agent, realm, installed)?;
            let _ = agent.set_module_record_environment(key, Some(environment));
            environment
        };
        let _ = agent.set_module_record_status(key, ModuleStatus::Linking);

        for request in &requested_modules {
            let resolved_key = request
                .resolved_key()
                .cloned()
                .ok_or(VmError::MissingModuleResolution)?;
            let _ = self.link_module_graph(agent, realm, &resolved_key)?;
        }

        let resolved_exports = self.compute_module_resolved_exports(agent, realm, key)?;
        let _ = agent.set_module_record_resolved_exports(key, resolved_exports);
        self.bind_module_imports(
            agent,
            realm,
            module_env,
            &requested_modules,
            &import_entries,
        )?;
        self.initialize_module_hoisted_functions(agent, realm, code, module_env)?;
        let _ = agent.set_module_record_status(key, ModuleStatus::Linked);
        Ok(module_env)
    }

    fn initialize_module_hoisted_functions(
        &self,
        agent: &mut Agent,
        realm: &RealmRecord,
        code: CodeRef,
        module_env: EnvironmentRef,
    ) -> VmResult<()> {
        let installed = self
            .installed_function_record(code)
            .cloned()
            .ok_or(VmError::MissingInstalledCode(code))?;
        for (slot, child_index) in Self::module_hoisted_function_initializers(&installed) {
            let frame = FrameRecord::new(
                code,
                0,
                RegisterWindow::new(0, 0),
                None,
                realm.id(),
                module_env,
                module_env,
                lyng_js_env::ExecutionContextKind::Module,
            );
            let closure = self.create_closure(agent, frame, child_index)?;
            Self::initialize_environment_slot(
                agent,
                module_env,
                slot,
                Value::from_object_ref(closure),
            )?;
        }
        Ok(())
    }

    fn module_hoisted_function_initializers(installed: &InstalledFunction) -> Vec<(u32, u32)> {
        let instructions = installed.function.instructions();
        let mut offset = Self::module_hoisted_function_prologue_start(instructions);
        let mut initializers = Vec::new();
        while let Some((slot, child_index)) =
            Self::module_hoisted_function_initializer_at(installed, instructions, offset)
        {
            initializers.push((slot, child_index));
            offset += 2;
        }
        initializers
    }

    fn module_hoisted_function_prologue_end(installed: &InstalledFunction) -> u32 {
        let instructions = installed.function.instructions();
        let start = Self::module_hoisted_function_prologue_start(instructions);
        let mut offset = start;
        while Self::module_hoisted_function_initializer_at(installed, instructions, offset)
            .is_some()
        {
            offset += 2;
        }
        if offset == start {
            0
        } else {
            u32::try_from(offset).expect("instruction offset should fit into u32")
        }
    }

    fn module_hoisted_function_prologue_start(instructions: &[Instruction]) -> usize {
        instructions
            .iter()
            .take_while(|instruction| {
                matches!(
                    instruction,
                    Instruction::Abx {
                        opcode: Opcode::LoadUndefined,
                        ..
                    }
                )
            })
            .count()
    }

    fn module_hoisted_function_initializer_at(
        installed: &InstalledFunction,
        instructions: &[Instruction],
        offset: usize,
    ) -> Option<(u32, u32)> {
        let create_offset = u32::try_from(offset).expect("instruction offset should fit into u32");
        let store_offset =
            u32::try_from(offset + 1).expect("instruction offset should fit into u32");
        let Instruction::Abx {
            opcode: Opcode::CreateClosure,
            a: create_register,
            bx: child_index,
        } = instructions.get(offset).copied()?
        else {
            return None;
        };
        let Instruction::Abx {
            opcode: Opcode::StoreEnvSlot,
            a: store_register,
            bx: env_operand,
        } = instructions.get(offset + 1).copied()?
        else {
            return None;
        };
        let create_operands = installed.wide_payload(create_offset).map_or_else(
            || lyng_js_bytecode::WideAbxOperands::narrow(create_register, child_index),
            |payload| {
                lyng_js_bytecode::WideAbxOperands::decode(create_register, child_index, payload)
            },
        );
        let store_operands = installed.wide_payload(store_offset).map_or_else(
            || lyng_js_bytecode::WideAbxOperands::narrow(store_register, env_operand),
            |payload| {
                lyng_js_bytecode::WideAbxOperands::decode(store_register, env_operand, payload)
            },
        );
        if create_operands.a() != store_operands.a() {
            return None;
        }
        let (depth, slot) = decode_env_operand(store_operands.bx());
        (depth == 0).then_some((slot, create_operands.bx()))
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "spec-shaped VM algorithm stays contiguous until the VM module split issue extracts it"
    )]
    fn evaluate_module_graph(
        &mut self,
        agent: &mut Agent,
        realm: &RealmRecord,
        key: &ModuleKey,
        module_env: EnvironmentRef,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        defer_waiter_flush_for: Option<&ModuleKey>,
        checkpoint_on_async_suspend: bool,
    ) -> VmResult<Value> {
        let (status, code, requested_modules, evaluation_error) = {
            let record = agent
                .module_record(key)
                .ok_or(VmError::MissingModuleRecord)?;
            (
                record.status(),
                record.code(),
                record.requested_modules().to_vec(),
                record.evaluation_error(),
            )
        };
        match status {
            ModuleStatus::Evaluated => return Ok(Value::undefined()),
            ModuleStatus::Evaluating if self.async_dependency_blocked_modules.contains(key) => {}
            ModuleStatus::Evaluating if self.async_body_suspended_modules.contains(key) => {
                return Err(VmError::AsyncSuspend);
            }
            ModuleStatus::Evaluating => return Ok(Value::undefined()),
            ModuleStatus::Errored => {
                if let Some(thrown) = evaluation_error {
                    return Err(VmError::Abrupt(lyng_js_types::AbruptCompletion::throw(
                        thrown,
                    )));
                }
            }
            ModuleStatus::New
            | ModuleStatus::Unlinked
            | ModuleStatus::Linking
            | ModuleStatus::Linked => {}
        }

        let code = code.ok_or(VmError::MissingModuleCode)?;
        let installed = InstalledCode::new(
            code,
            self.installed_function(code)
                .ok_or(VmError::MissingInstalledCode(code))?
                .id(),
        );
        let entry_offset = self
            .installed_function_record(code)
            .map(Self::module_hoisted_function_prologue_end)
            .ok_or(VmError::MissingInstalledCode(code))?;
        let _ = agent.set_module_record_status(key, ModuleStatus::Evaluating);
        let mut suspended_dependencies = Vec::new();
        for request in &requested_modules {
            let resolved_key = request
                .resolved_key()
                .cloned()
                .ok_or(VmError::MissingModuleResolution)?;
            let evaluation_keys = if request.phase() == ModuleRequestPhase::Defer {
                self.gather_asynchronous_transitive_dependencies(agent, &resolved_key)?
            } else {
                vec![resolved_key]
            };
            for evaluation_key in evaluation_keys {
                let dependency_env = self.link_module_graph(agent, realm, &evaluation_key)?;
                match self.evaluate_module_graph(
                    agent,
                    realm,
                    &evaluation_key,
                    dependency_env,
                    host,
                    registry,
                    defer_waiter_flush_for,
                    false,
                ) {
                    Ok(_) => {}
                    Err(VmError::AsyncSuspend) => suspended_dependencies.push(evaluation_key),
                    Err(error) => return Err(error),
                }
            }
        }
        if !suspended_dependencies.is_empty() {
            self.queue_async_dependency_blocked_module(key);
            if !checkpoint_on_async_suspend {
                return Err(VmError::AsyncSuspend);
            }
            if let Err(error) = self.checkpoint_promise_jobs(agent, host, registry) {
                self.async_dependency_blocked_modules.remove(key);
                if let VmError::Abrupt(completion) = &error {
                    self.async_dependency_completed_modules.insert(key.clone());
                    let _ = agent.set_module_record_status(key, ModuleStatus::Errored);
                    let _ =
                        agent.set_module_record_evaluation_error(key, completion.thrown_value());
                    if defer_waiter_flush_for.is_none_or(|deferred| deferred != key) {
                        self.settle_waiting_dynamic_imports_for_module(agent, host, registry, key)?;
                    }
                }
                return Err(error);
            }
            self.finish_async_body_suspended_modules_after_checkpoint(
                agent,
                host,
                registry,
                defer_waiter_flush_for,
            )?;
            self.drain_async_dependency_blocked_modules(
                agent,
                realm,
                host,
                registry,
                defer_waiter_flush_for,
            )?;
            match agent
                .module_record(key)
                .ok_or(VmError::MissingModuleRecord)?
                .status()
            {
                ModuleStatus::Evaluated => return Ok(Value::undefined()),
                ModuleStatus::Errored => {
                    let thrown = agent
                        .module_record(key)
                        .and_then(ModuleRecord::evaluation_error)
                        .unwrap_or(Value::undefined());
                    return Err(VmError::Abrupt(lyng_js_types::AbruptCompletion::throw(
                        thrown,
                    )));
                }
                ModuleStatus::Evaluating
                | ModuleStatus::Linked
                | ModuleStatus::Linking
                | ModuleStatus::Unlinked
                | ModuleStatus::New => {}
            }
        }

        let was_dependency_blocked = self.async_dependency_blocked_modules.remove(key);
        let result = self.evaluate_entry_with_registry_from_offset(
            agent,
            installed,
            module_env,
            module_env,
            None,
            host,
            registry,
            None,
            None,
            entry_offset,
        );
        match result {
            Ok(value) => {
                self.async_body_suspended_modules.remove(key);
                self.async_dependency_blocked_modules.remove(key);
                if was_dependency_blocked {
                    self.async_dependency_completed_modules.insert(key.clone());
                }
                let _ = agent.set_module_record_status(key, ModuleStatus::Evaluated);
                let _ = agent.set_module_record_evaluation_error(key, None);
                if defer_waiter_flush_for.is_none_or(|deferred| deferred != key) {
                    self.settle_waiting_dynamic_imports_for_module(agent, host, registry, key)?;
                }
                Ok(value)
            }
            Err(VmError::AsyncSuspend) if !checkpoint_on_async_suspend => {
                self.async_body_suspended_modules.insert(key.clone());
                Err(VmError::AsyncSuspend)
            }
            Err(VmError::AsyncSuspend) => {
                self.async_body_suspended_modules.insert(key.clone());
                match self.checkpoint_promise_jobs(agent, host, registry) {
                    Ok(()) => {
                        self.async_body_suspended_modules.remove(key);
                        self.async_dependency_blocked_modules.remove(key);
                        if was_dependency_blocked {
                            self.async_dependency_completed_modules.insert(key.clone());
                        }
                        let _ = agent.set_module_record_status(key, ModuleStatus::Evaluated);
                        let _ = agent.set_module_record_evaluation_error(key, None);
                        if defer_waiter_flush_for.is_none_or(|deferred| deferred != key) {
                            self.settle_waiting_dynamic_imports_for_module(
                                agent, host, registry, key,
                            )?;
                        }
                        Ok(Value::undefined())
                    }
                    Err(VmError::Abrupt(completion)) => {
                        self.async_body_suspended_modules.remove(key);
                        self.async_dependency_blocked_modules.remove(key);
                        if was_dependency_blocked {
                            self.async_dependency_completed_modules.insert(key.clone());
                        }
                        let _ = agent.set_module_record_status(key, ModuleStatus::Errored);
                        let _ = agent
                            .set_module_record_evaluation_error(key, completion.thrown_value());
                        if defer_waiter_flush_for.is_none_or(|deferred| deferred != key) {
                            self.settle_waiting_dynamic_imports_for_module(
                                agent, host, registry, key,
                            )?;
                        }
                        Err(VmError::Abrupt(completion))
                    }
                    Err(error) => Err(error),
                }
            }
            Err(VmError::Abrupt(completion)) => {
                self.async_body_suspended_modules.remove(key);
                self.async_dependency_blocked_modules.remove(key);
                if was_dependency_blocked {
                    self.async_dependency_completed_modules.insert(key.clone());
                }
                let _ = agent.set_module_record_status(key, ModuleStatus::Errored);
                let _ = agent.set_module_record_evaluation_error(key, completion.thrown_value());
                if defer_waiter_flush_for.is_none_or(|deferred| deferred != key) {
                    self.settle_waiting_dynamic_imports_for_module(agent, host, registry, key)?;
                }
                Err(VmError::Abrupt(completion))
            }
            Err(error) => Err(error),
        }
    }

    fn queue_async_dependency_blocked_module(&mut self, key: &ModuleKey) {
        if self.async_dependency_blocked_modules.insert(key.clone()) {
            self.async_dependency_blocked_queue.push_back(key.clone());
        }
    }

    fn finish_async_body_suspended_modules_after_checkpoint(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        defer_waiter_flush_for: Option<&ModuleKey>,
    ) -> VmResult<()> {
        let suspended = self
            .async_body_suspended_modules
            .drain()
            .collect::<Vec<_>>();
        for key in suspended {
            match agent
                .module_record(&key)
                .ok_or(VmError::MissingModuleRecord)?
                .status()
            {
                ModuleStatus::Evaluating => {
                    let _ = agent.set_module_record_status(&key, ModuleStatus::Evaluated);
                    let _ = agent.set_module_record_evaluation_error(&key, None);
                }
                ModuleStatus::Evaluated => {}
                ModuleStatus::New
                | ModuleStatus::Unlinked
                | ModuleStatus::Linking
                | ModuleStatus::Linked
                | ModuleStatus::Errored => continue,
            }
            if defer_waiter_flush_for.is_none_or(|deferred| deferred != &key) {
                self.settle_waiting_dynamic_imports_for_module(agent, host, registry, &key)?;
            }
        }
        Ok(())
    }

    fn drain_async_dependency_blocked_modules(
        &mut self,
        agent: &mut Agent,
        realm: &RealmRecord,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        defer_waiter_flush_for: Option<&ModuleKey>,
    ) -> VmResult<()> {
        while let Some(key) = self.async_dependency_blocked_queue.pop_front() {
            if !self.async_dependency_blocked_modules.contains(&key) {
                continue;
            }
            let Some(module_env) = agent
                .module_record(&key)
                .and_then(ModuleRecord::environment)
            else {
                continue;
            };
            let _ = self.evaluate_module_graph(
                agent,
                realm,
                &key,
                module_env,
                host,
                registry,
                defer_waiter_flush_for,
                true,
            )?;
        }
        Ok(())
    }

    fn bind_module_imports(
        &mut self,
        agent: &mut Agent,
        realm: &RealmRecord,
        module_env: EnvironmentRef,
        requested_modules: &[ModuleRequestRecord],
        import_entries: &[ModuleImportEntry],
    ) -> VmResult<()> {
        for entry in import_entries {
            let resolved_key = requested_modules
                .get(entry.request_index() as usize)
                .and_then(ModuleRequestRecord::resolved_key)
                .cloned()
                .ok_or(VmError::MissingModuleResolution)?;
            match entry.import_kind() {
                ModuleImportKind::Named(export_name) => {
                    let resolved = self
                        .resolve_module_export(
                            agent,
                            realm,
                            &resolved_key,
                            export_name,
                            &mut Vec::new(),
                        )?
                        .ok_or(VmError::MissingModuleResolution)?;
                    match resolved.target() {
                        ModuleResolvedExportTarget::Binding { environment, slot } => {
                            if !agent.set_module_binding_alias(
                                module_env,
                                entry.local_slot(),
                                Some(ModuleBindingAlias::new(environment, slot)),
                            ) {
                                return Err(VmError::MissingModuleEnvironment);
                            }
                        }
                        ModuleResolvedExportTarget::Value(value) => {
                            if !agent.set_module_binding_alias(module_env, entry.local_slot(), None)
                            {
                                return Err(VmError::MissingModuleEnvironment);
                            }
                            Self::initialize_environment_slot(
                                agent,
                                module_env,
                                entry.local_slot(),
                                value,
                            )?;
                        }
                    }
                }
                ModuleImportKind::NamespaceObject => {
                    let namespace = self.module_namespace_object_for_request(
                        agent,
                        realm,
                        &resolved_key,
                        requested_modules
                            .get(entry.request_index() as usize)
                            .map_or(ModuleRequestPhase::Evaluation, ModuleRequestRecord::phase),
                    )?;
                    if !agent.set_module_binding_alias(module_env, entry.local_slot(), None) {
                        return Err(VmError::MissingModuleEnvironment);
                    }
                    Self::initialize_environment_slot(
                        agent,
                        module_env,
                        entry.local_slot(),
                        Value::from_object_ref(namespace),
                    )?;
                }
                ModuleImportKind::Source => {
                    return Err(VmError::Abrupt(AbruptCompletion::throw(
                        errors::syntax_error_value(agent),
                    )));
                }
            }
        }
        Ok(())
    }

    fn compute_module_resolved_exports(
        &mut self,
        agent: &mut Agent,
        realm: &RealmRecord,
        key: &ModuleKey,
    ) -> VmResult<Vec<ModuleResolvedExport>> {
        let (local_exports, indirect_exports) = {
            let record = agent
                .module_record(key)
                .ok_or(VmError::MissingModuleRecord)?;
            (
                record.local_exports().to_vec(),
                record.indirect_exports().to_vec(),
            )
        };

        let mut explicit_export_names = HashSet::new();
        for entry in &local_exports {
            explicit_export_names.insert(entry.export_name());
        }
        for entry in &indirect_exports {
            explicit_export_names.insert(entry.export_name());
        }
        let export_names =
            self.collect_module_exported_names(agent, realm, key, &mut Vec::new())?;

        let mut resolved_exports = Vec::new();
        for export_name in export_names {
            match self.resolve_module_export(agent, realm, key, export_name, &mut Vec::new()) {
                Ok(Some(export)) => resolved_exports.push(export),
                Ok(None) => {
                    if explicit_export_names.contains(&export_name) {
                        return Err(VmError::MissingModuleResolution);
                    }
                }
                Err(VmError::AmbiguousModuleExport) => {
                    if explicit_export_names.contains(&export_name) {
                        return Err(VmError::AmbiguousModuleExport);
                    }
                }
                Err(error) => return Err(error),
            }
        }
        Ok(resolved_exports)
    }

    fn collect_module_exported_names(
        &mut self,
        agent: &mut Agent,
        realm: &RealmRecord,
        key: &ModuleKey,
        export_star_set: &mut Vec<ModuleKey>,
    ) -> VmResult<Vec<AtomId>> {
        if export_star_set.iter().any(|candidate| candidate == key) {
            return Ok(Vec::new());
        }
        export_star_set.push(key.clone());

        let (local_exports, indirect_exports, star_exports, requested_modules) = {
            let record = agent
                .module_record(key)
                .ok_or(VmError::MissingModuleRecord)?;
            (
                record.local_exports().to_vec(),
                record.indirect_exports().to_vec(),
                record.star_exports().to_vec(),
                record.requested_modules().to_vec(),
            )
        };

        let mut export_names = Vec::new();
        for entry in &local_exports {
            if !export_names.contains(&entry.export_name()) {
                export_names.push(entry.export_name());
            }
        }
        for entry in &indirect_exports {
            if !export_names.contains(&entry.export_name()) {
                export_names.push(entry.export_name());
            }
        }
        for entry in &star_exports {
            let resolved_key = requested_modules
                .get(entry.request_index() as usize)
                .and_then(ModuleRequestRecord::resolved_key)
                .cloned()
                .ok_or(VmError::MissingModuleResolution)?;
            let _ = self.link_module_graph(agent, realm, &resolved_key)?;
            let dependency_export_names =
                self.collect_module_exported_names(agent, realm, &resolved_key, export_star_set)?;
            for export_name in dependency_export_names {
                if export_name == WellKnownAtom::default.id() {
                    continue;
                }
                if !export_names.contains(&export_name) {
                    export_names.push(export_name);
                }
            }
        }

        let _ = export_star_set.pop();
        Ok(export_names)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "spec-shaped VM algorithm stays contiguous until the VM module split issue extracts it"
    )]
    fn resolve_module_export(
        &mut self,
        agent: &mut Agent,
        realm: &RealmRecord,
        key: &ModuleKey,
        export_name: AtomId,
        resolve_set: &mut Vec<(ModuleKey, AtomId)>,
    ) -> VmResult<Option<ModuleResolvedExport>> {
        if resolve_set.iter().any(|(candidate_key, candidate_export)| {
            candidate_key == key && *candidate_export == export_name
        }) {
            return Ok(None);
        }
        if let Some(cached) = agent
            .module_record(key)
            .and_then(|record| record.resolved_export(export_name))
        {
            return Ok(Some(cached));
        }

        let (
            module_env,
            local_exports,
            import_entries,
            indirect_exports,
            star_exports,
            requested_modules,
        ) = {
            let record = agent
                .module_record(key)
                .ok_or(VmError::MissingModuleRecord)?;
            (
                record.environment(),
                record.local_exports().to_vec(),
                record.import_entries().to_vec(),
                record.indirect_exports().to_vec(),
                record.star_exports().to_vec(),
                record.requested_modules().to_vec(),
            )
        };
        let module_env = module_env.ok_or(VmError::MissingModuleEnvironment)?;
        resolve_set.push((key.clone(), export_name));

        let resolved = if let Some(entry) = local_exports
            .iter()
            .copied()
            .find(|entry| entry.export_name() == export_name)
        {
            if let Some(alias) = agent.module_binding_alias(module_env, entry.local_slot()) {
                Some(ModuleResolvedExport::new(
                    export_name,
                    ModuleResolvedExportTarget::Binding {
                        environment: alias.environment(),
                        slot: alias.slot(),
                    },
                ))
            } else if let Some(import_entry) = import_entries
                .iter()
                .copied()
                .find(|import| import.local_slot() == entry.local_slot())
            {
                let resolved_key = requested_modules
                    .get(import_entry.request_index() as usize)
                    .and_then(ModuleRequestRecord::resolved_key)
                    .cloned()
                    .ok_or(VmError::MissingModuleResolution)?;
                match import_entry.import_kind() {
                    ModuleImportKind::Named(import_name) => self
                        .resolve_module_export(
                            agent,
                            realm,
                            &resolved_key,
                            import_name,
                            resolve_set,
                        )?
                        .map(|resolved| ModuleResolvedExport::new(export_name, resolved.target())),
                    ModuleImportKind::NamespaceObject => Some(ModuleResolvedExport::new(
                        export_name,
                        ModuleResolvedExportTarget::Value(Value::from_object_ref(
                            self.module_namespace_object_for_request(
                                agent,
                                realm,
                                &resolved_key,
                                requested_modules
                                    .get(import_entry.request_index() as usize)
                                    .map_or(
                                        ModuleRequestPhase::Evaluation,
                                        ModuleRequestRecord::phase,
                                    ),
                            )?,
                        )),
                    )),
                    ModuleImportKind::Source => {
                        return Err(VmError::Abrupt(AbruptCompletion::throw(
                            errors::syntax_error_value(agent),
                        )));
                    }
                }
            } else {
                Some(ModuleResolvedExport::new(
                    export_name,
                    ModuleResolvedExportTarget::Binding {
                        environment: module_env,
                        slot: entry.local_slot(),
                    },
                ))
            }
        } else if let Some(entry) = indirect_exports
            .iter()
            .copied()
            .find(|entry| entry.export_name() == export_name)
        {
            let resolved_key = requested_modules
                .get(entry.request_index() as usize)
                .and_then(ModuleRequestRecord::resolved_key)
                .cloned()
                .ok_or(VmError::MissingModuleResolution)?;
            match entry.import_kind() {
                ModuleImportKind::Named(import_name) => self
                    .resolve_module_export(agent, realm, &resolved_key, import_name, resolve_set)?
                    .map(|resolved| ModuleResolvedExport::new(export_name, resolved.target())),
                ModuleImportKind::NamespaceObject => Some(ModuleResolvedExport::new(
                    export_name,
                    ModuleResolvedExportTarget::Value(Value::from_object_ref(
                        self.module_namespace_object_for_request(
                            agent,
                            realm,
                            &resolved_key,
                            requested_modules
                                .get(entry.request_index() as usize)
                                .map_or(ModuleRequestPhase::Evaluation, ModuleRequestRecord::phase),
                        )?,
                    )),
                )),
                ModuleImportKind::Source => {
                    return Err(VmError::Abrupt(AbruptCompletion::throw(
                        errors::syntax_error_value(agent),
                    )));
                }
            }
        } else if export_name == WellKnownAtom::default.id() {
            None
        } else {
            let mut resolved = None;
            for entry in &star_exports {
                let resolved_key = requested_modules
                    .get(entry.request_index() as usize)
                    .and_then(ModuleRequestRecord::resolved_key)
                    .cloned()
                    .ok_or(VmError::MissingModuleResolution)?;
                let Some(candidate) = self.resolve_module_export(
                    agent,
                    realm,
                    &resolved_key,
                    export_name,
                    resolve_set,
                )?
                else {
                    continue;
                };
                if let Some(existing) = resolved {
                    if existing != candidate {
                        let _ = resolve_set.pop();
                        return Err(VmError::AmbiguousModuleExport);
                    }
                } else {
                    resolved = Some(candidate);
                }
            }
            resolved
        };

        let _ = resolve_set.pop();
        Ok(resolved)
    }

    fn module_namespace_object(
        &mut self,
        agent: &mut Agent,
        realm: &RealmRecord,
        key: &ModuleKey,
    ) -> VmResult<ObjectRef> {
        self.module_namespace_object_with_phase(agent, realm, key, false)
    }

    fn module_namespace_object_for_request(
        &mut self,
        agent: &mut Agent,
        realm: &RealmRecord,
        key: &ModuleKey,
        phase: ModuleRequestPhase,
    ) -> VmResult<ObjectRef> {
        self.module_namespace_object_with_phase(
            agent,
            realm,
            key,
            phase == ModuleRequestPhase::Defer,
        )
    }

    #[expect(
        clippy::too_many_lines,
        reason = "spec-shaped VM algorithm stays contiguous until the VM module split issue extracts it"
    )]
    fn module_namespace_object_with_phase(
        &mut self,
        agent: &mut Agent,
        realm: &RealmRecord,
        key: &ModuleKey,
        deferred: bool,
    ) -> VmResult<ObjectRef> {
        let _ = self.link_module_graph(agent, realm, key)?;
        let needs_resolved_exports = deferred
            && agent.module_record(key).is_some_and(|record| {
                record.resolved_exports().is_empty()
                    && (!record.local_exports().is_empty()
                        || !record.indirect_exports().is_empty()
                        || !record.star_exports().is_empty())
            });
        if needs_resolved_exports {
            let resolved_exports = self.compute_module_resolved_exports(agent, realm, key)?;
            let _ = agent.set_module_record_resolved_exports(key, resolved_exports);
        }
        let existing_namespace = agent.module_record(key).and_then(|record| {
            if deferred {
                record.deferred_namespace()
            } else {
                record.namespace()
            }
        });
        if let Some(namespace) = existing_namespace {
            if deferred
                && agent
                    .module_record(key)
                    .is_some_and(|record| !matches!(record.status(), ModuleStatus::Evaluated))
            {
                self.deferred_module_namespaces
                    .insert(namespace, key.clone());
            }
            return Ok(namespace);
        }

        let root_shape = realm
            .root_shape()
            .ok_or_else(|| VmError::MissingRootShape(realm.id()))?;
        let mut exports = agent
            .module_record(key)
            .ok_or(VmError::MissingModuleRecord)?
            .resolved_exports()
            .iter()
            .map(|entry| {
                ModuleNamespaceExport::new(
                    entry.export_name(),
                    match entry.target() {
                        ModuleResolvedExportTarget::Binding { environment, slot } => {
                            ModuleNamespaceExportTarget::Binding { environment, slot }
                        }
                        ModuleResolvedExportTarget::Value(value) => {
                            ModuleNamespaceExportTarget::Value(value)
                        }
                    },
                )
                .with_array_index(module_export_array_index(agent, entry.export_name()))
            })
            .collect::<Vec<_>>();
        exports.sort_by(|left, right| {
            let left_text = agent.atoms().get(left.export_name()).unwrap_or("");
            let right_text = agent.atoms().get(right.export_name()).unwrap_or("");
            left_text
                .cmp(right_text)
                .then_with(|| left.export_name().raw().cmp(&right.export_name().raw()))
        });
        let to_string_tag = agent
            .well_known_symbol(WellKnownSymbolId::ToStringTag)
            .expect("default realm should bootstrap Symbol.toStringTag");
        let tag = if deferred {
            "Deferred Module"
        } else {
            "Module"
        };
        let module_tag = Value::from_string_ref(agent.alloc_runtime_string(
            tag,
            None,
            AllocationLifetime::Default,
        ));

        let namespace = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let object = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape),
                AllocationLifetime::Default,
            );
            let mut descriptor = PropertyDescriptor::new();
            descriptor.set_value(module_tag);
            descriptor.set_writable(false);
            descriptor.set_enumerable(false);
            descriptor.set_configurable(false);
            assert!(
                matches!(
                    objects.define_own_property(
                        &mut mutator,
                        object,
                        PropertyKey::from_symbol(to_string_tag),
                        descriptor,
                        AllocationLifetime::Default,
                    ),
                    Ok(true)
                ),
                "module namespace @@toStringTag should install on a fresh namespace object"
            );
            assert!(
                objects.install_module_namespace_object(object, exports),
                "module namespace side table should install on freshly allocated ordinary objects"
            );
            object
        });
        if deferred {
            let _ = agent.set_module_record_deferred_namespace(key, Some(namespace));
            if agent
                .module_record(key)
                .is_some_and(|record| !matches!(record.status(), ModuleStatus::Evaluated))
            {
                self.deferred_module_namespaces
                    .insert(namespace, key.clone());
            }
        } else {
            let _ = agent.set_module_record_namespace(key, Some(namespace));
        }
        Ok(namespace)
    }

    fn report_module_diagnostics(
        host: &dyn HostHooks,
        diagnostics: &[Diagnostic],
    ) -> Result<(), ModuleLoadError> {
        for diagnostic in diagnostics {
            host.report_diagnostic(&DiagnosticReportRequest {
                severity: diagnostic.severity,
                source: Some(diagnostic.span.source),
                span: Some(diagnostic.span),
                message: diagnostic.message.clone(),
            })
            .map_err(ModuleLoadError::Host)?;
        }
        Ok(())
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
}

fn module_export_array_index(agent: &Agent, export_name: AtomId) -> Option<u32> {
    agent
        .atoms()
        .get(export_name)
        .and_then(string_text_array_index)
}

fn compiled_module_record(
    vm: &Vm,
    installed: InstalledCode,
    key: &ModuleKey,
    display_name: &str,
    unit: &CompiledModuleUnit,
) -> ModuleRecord {
    let canonical_atom = |atom| vm.canonical_atom_for_code(installed.code(), atom);
    ModuleRecord::new(
        key.clone(),
        display_name,
        unit.requested_modules()
            .iter()
            .map(|request| {
                ModuleRequestRecord::new(
                    request.specifier(),
                    request.attributes().to_vec(),
                    match request.phase() {
                        CompiledModuleRequestPhase::Evaluation => ModuleRequestPhase::Evaluation,
                        CompiledModuleRequestPhase::Source => ModuleRequestPhase::Source,
                        CompiledModuleRequestPhase::Defer => ModuleRequestPhase::Defer,
                    },
                )
            })
            .collect(),
        unit.import_entries()
            .iter()
            .map(|entry| {
                ModuleImportEntry::new(
                    entry.request_index(),
                    canonical_atom(entry.local_name()),
                    entry.local_slot(),
                    match entry.import_kind() {
                        CompiledModuleImportKind::Named(name) => {
                            ModuleImportKind::Named(canonical_atom(name))
                        }
                        CompiledModuleImportKind::NamespaceObject => {
                            ModuleImportKind::NamespaceObject
                        }
                        CompiledModuleImportKind::Source => ModuleImportKind::Source,
                    },
                )
            })
            .collect(),
        unit.local_exports()
            .iter()
            .map(|entry| {
                ModuleLocalExportEntry::new(
                    canonical_atom(entry.export_name()),
                    entry.local_name().map(canonical_atom),
                    entry.local_slot(),
                )
            })
            .collect(),
        unit.indirect_exports()
            .iter()
            .map(|entry| {
                ModuleIndirectExportEntry::new(
                    canonical_atom(entry.export_name()),
                    entry.request_index(),
                    match entry.import_kind() {
                        CompiledModuleImportKind::Named(name) => {
                            ModuleImportKind::Named(canonical_atom(name))
                        }
                        CompiledModuleImportKind::NamespaceObject => {
                            ModuleImportKind::NamespaceObject
                        }
                        CompiledModuleImportKind::Source => ModuleImportKind::Source,
                    },
                )
            })
            .collect(),
        unit.star_exports()
            .iter()
            .map(|entry| ModuleStarExportEntry::new(entry.request_index()))
            .collect(),
    )
}
