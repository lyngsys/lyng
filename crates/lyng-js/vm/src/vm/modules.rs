use std::collections::HashSet;

use lyng_js_builtins::BootstrapMode;
use lyng_js_bytecode::{Instruction, InstructionStream, Opcode};
use lyng_js_common::{AtomId, Diagnostic, WellKnownAtom};
use lyng_js_compiler::{
    compile_module, CompiledModuleUnit, ModuleImportKind as CompiledModuleImportKind,
    ModuleRequestPhase as CompiledModuleRequestPhase,
};
use lyng_js_env::{
    Agent, ModuleBindingAlias, ModuleImportEntry, ModuleImportKind, ModuleIndirectExportEntry,
    ModuleLocalExportEntry, ModuleRecord, ModuleRequestPhase, ModuleRequestRecord,
    ModuleResolvedExport, ModuleResolvedExportTarget, ModuleStarExportEntry, ModuleStatus,
    RealmRecord,
};
use lyng_js_gc::AllocationLifetime;
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
    AbruptCompletion, CodeRef, EnvironmentRef, ObjectRef, PropertyDescriptor, PropertyKey,
    RealmRef, Value, WellKnownSymbolId,
};

use crate::{FrameRecord, InstalledCode, RegisterWindow, VmError};

use super::call::RejectingNativeRegistry;
use super::install::InstalledFunction;
use super::{decode_env_operand, string_text_array_index, Vm};
use crate::error::{ModuleLoadError, VmResult};
use crate::extensions::SharedRealmExtensionProvider;

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

impl Vm {
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

    pub(in crate::vm) fn link_module_graph(
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
            let closure = self.create_closure(agent, &frame, child_index)?;
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
        while let Some((slot, child_index, next_offset)) =
            Self::module_hoisted_function_initializer_at(installed, offset)
        {
            initializers.push((slot, child_index));
            offset = next_offset;
        }
        initializers
    }

    fn module_hoisted_function_prologue_end(installed: &InstalledFunction) -> u32 {
        let instructions = installed.function.instructions();
        let start = Self::module_hoisted_function_prologue_start(instructions);
        let mut offset = start;
        while let Some((_, _, next_offset)) =
            Self::module_hoisted_function_initializer_at(installed, offset)
        {
            offset = next_offset;
        }
        if offset == start {
            0
        } else {
            offset
        }
    }

    fn module_hoisted_function_prologue_start(instructions: InstructionStream<'_>) -> u32 {
        let mut start = 0;
        for (offset, instruction) in instructions.byte_offsets().zip(instructions.iter()) {
            if !matches!(
                instruction,
                Instruction::Abx {
                    opcode: Opcode::LoadUndefined,
                    ..
                }
            ) {
                break;
            }
            let next = offset
                .checked_add(instruction.encoded_len())
                .expect("instruction offset should stay within usize");
            start = u32::try_from(next).expect("instruction offset should fit into u32");
        }
        start
    }

    fn module_hoisted_function_initializer_at(
        installed: &InstalledFunction,
        create_offset: u32,
    ) -> Option<(u32, u32, u32)> {
        let create_instruction = installed.instruction_at(create_offset)?;
        let Instruction::Abx {
            opcode: Opcode::CreateClosure,
            a: create_register,
            bx: child_index,
        } = create_instruction
        else {
            return None;
        };
        let store_offset =
            create_offset.checked_add(u32::try_from(create_instruction.encoded_len()).ok()?)?;
        let store_instruction = installed.instruction_at(store_offset)?;
        let Instruction::Abx {
            opcode: Opcode::StoreEnvSlot,
            a: store_register,
            bx: env_operand,
        } = store_instruction
        else {
            return None;
        };
        let next_offset =
            store_offset.checked_add(u32::try_from(store_instruction.encoded_len()).ok()?)?;
        let create_operands = lyng_js_bytecode::WideAbxOperands::new(create_register, child_index);
        let store_operands = lyng_js_bytecode::WideAbxOperands::new(store_register, env_operand);
        if create_operands.a() != store_operands.a() {
            return None;
        }
        let (depth, slot) = decode_env_operand(store_operands.bx());
        (depth == 0).then_some((slot, create_operands.bx(), next_offset))
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "module graph algorithms stay contiguous enough to preserve ECMA-262 ordering and completion propagation"
    )]
    pub(in crate::vm) fn evaluate_module_graph(
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
            ModuleStatus::Evaluating if self.async_dependency_blocked_modules.contains(key) => {}
            ModuleStatus::Evaluating if self.async_body_suspended_modules.contains(key) => {
                return Err(VmError::AsyncSuspend);
            }
            ModuleStatus::Evaluated | ModuleStatus::Evaluating => return Ok(Value::undefined()),
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
        reason = "module graph algorithms stay contiguous enough to preserve ECMA-262 ordering and completion propagation"
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

    pub(in crate::vm) fn module_namespace_object(
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
        reason = "module graph algorithms stay contiguous enough to preserve ECMA-262 ordering and completion propagation"
    )]
    pub(in crate::vm) fn module_namespace_object_with_phase(
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
