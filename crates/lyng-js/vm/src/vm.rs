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
use lyng_js_common::{AtomId, SourceId, WellKnownAtom};
use lyng_js_compiler::dynamic::DynamicFunctionCacheKey;
use lyng_js_env::{
    Agent, EnvironmentBindingLayout, EnvironmentLayout, EnvironmentLayoutId, EnvironmentLayoutKind,
    EnvironmentSlotFlags, ExecutionContext, ModuleRecord, ModuleStatus, RealmRecord,
    ThisBindingStatus, ThisState,
};
use lyng_js_gc::{AllocationLifetime, PrimitiveCollectionReport};
use lyng_js_host::{HostHooks, ModuleKey, NoopHostHooks};
use lyng_js_objects::{NativeFunctionRegistry, ObjectAllocation};
use lyng_js_types::{
    AbruptCompletion, BuiltinFunctionId, CodeRef, EnvironmentRef, ObjectRef, RealmRef, Value,
    WellKnownSymbolId,
};

use crate::activation::ActivationSideTables;
use crate::enumeration::{ForInStateTable, IteratorStateTable};
use crate::error::VmResult;
use crate::extensions::{RealmExtensionInstallation, SharedRealmExtensionProvider};
use crate::name_refs::CapturedNameReferenceTable;
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
mod modules;
mod names;
mod property_access;
mod registers;
mod runtime_objects;
mod state;
mod tiering;
mod values;
mod with_env;

use call::RejectingNativeRegistry;
use feedback::FeedbackVector;
use install::InstalledFunction;
use state::{
    ActiveEnvScopeRange, ActiveVmRoots, AsyncFrameState, AsyncGeneratorFrameState,
    AsyncGeneratorRequest, DirectEvalEnvironmentState, DynamicImportPhase, DynamicImportRequest,
    EntryExecutionOverride, LoopIterationEnvironment, PendingDynamicImport,
    SuspendedExecutionSideState, TemplateCacheKey, WithEnvironmentState,
};
use tiering::TieringState;
use values::{bytecode_index, code_index, decode_env_operand, string_text_array_index};

pub use modules::LoadedModuleRoot;

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
        let Ok(start) = usize::try_from(register_base) else {
            debug_assert!(false, "register stack base should fit into usize");
            return;
        };
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
