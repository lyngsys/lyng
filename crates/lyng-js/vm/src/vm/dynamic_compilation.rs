use super::values::alloc_string;
use super::{
    Agent, AllocationLifetime, FrameRecord, NativeFunctionRegistry, ObjectRef, Value, Vm, VmError,
    VmResult, WellKnownAtom,
};
use lyng_js_builtins::DynamicFunctionKind as BuiltinDynamicFunctionKind;
use lyng_js_common::{AtomId, SourceId, Span};
use lyng_js_compiler::dynamic;
use lyng_js_env::{
    EnvironmentBindingLayout, EnvironmentLayout, EnvironmentLayoutKind, EnvironmentSlotFlags,
    ThisState,
};
use lyng_js_host::HostHooks;
use lyng_js_objects::{
    ClassPrivateElementKind, InternalMethodError, ObjectAllocation, ObjectColdData,
    OrdinaryObjectData, RegExpPayload,
};
use lyng_js_ops::{errors, names as ops_names, object};
use lyng_js_parser::validate_regexp_literal;
use lyng_js_sema::{
    ClassPrivateElementKind as SemaClassPrivateElementKind, ClassPrivateElementRecord,
    DeclarationKind, DirectEvalScriptAnalysisOptions, ResolutionKind, ScopeId, ScriptSema,
    StorageClass,
};
use lyng_js_types::{AbruptCompletion, PropertyDescriptor, PropertyKey, RealmRef, StringRef};
use std::collections::HashMap;

fn split_eval_regexp_literal_source(source: &str) -> Option<(&str, &str)> {
    let mut chars = source.char_indices();
    if chars.next()?.1 != '/' {
        return None;
    }

    let mut escaped = false;
    let mut in_class = false;
    for (index, ch) in chars {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '[' => in_class = true,
            ']' if in_class => in_class = false,
            '/' if !in_class => {
                let pattern = &source[1..index];
                let flags = &source[index + ch.len_utf8()..];
                if flags.chars().all(is_regexp_literal_flag_char) {
                    return Some((pattern, flags));
                }
                return None;
            }
            _ => {}
        }
    }
    None
}

fn is_regexp_literal_flag_char(ch: char) -> bool {
    ch == '$' || ch == '_' || ch.is_ascii_alphanumeric()
}

fn split_eval_regexp_literal_units(units: &[u16]) -> Option<(&[u16], String)> {
    if units.first().copied()? != u16::from(b'/') {
        return None;
    }
    let mut escaped = false;
    let mut in_class = false;
    for (index, unit) in units.iter().copied().enumerate().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }
        match unit {
            0x005c => escaped = true,
            0x005b => in_class = true,
            0x005d if in_class => in_class = false,
            0x002f if !in_class => {
                let flags = units[index + 1..]
                    .iter()
                    .copied()
                    .map(|unit| u8::try_from(unit).ok().map(char::from))
                    .collect::<Option<String>>()?;
                if flags.chars().all(is_regexp_literal_flag_char) {
                    return Some((&units[1..index], flags));
                }
                return None;
            }
            _ => {}
        }
    }
    None
}

fn string_ref_code_units(agent: &Agent, string: StringRef) -> Option<Vec<u16>> {
    let view = agent.heap().view().string_view(string)?;
    if let Some(bytes) = view.latin1_bytes() {
        return Some(bytes.iter().copied().map(u16::from).collect());
    }
    let bytes = view.utf16_bytes()?;
    Some(
        bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect(),
    )
}

impl Vm {
    fn compiler_dynamic_function_kind(
        kind: BuiltinDynamicFunctionKind,
    ) -> dynamic::DynamicFunctionKind {
        match kind {
            BuiltinDynamicFunctionKind::Ordinary => dynamic::DynamicFunctionKind::Ordinary,
            BuiltinDynamicFunctionKind::Generator => dynamic::DynamicFunctionKind::Generator,
            BuiltinDynamicFunctionKind::Async => dynamic::DynamicFunctionKind::Async,
            BuiltinDynamicFunctionKind::AsyncGenerator => {
                dynamic::DynamicFunctionKind::AsyncGenerator
            }
        }
    }

    fn dynamic_stage_message<'a>(
        error: &dynamic::DynamicCompilationError,
        parse: &'a str,
        semantic: &'a str,
        compile: &'a str,
    ) -> &'a str {
        match error.stage() {
            dynamic::DynamicCompilationStage::Parse => parse,
            dynamic::DynamicCompilationStage::Semantic => semantic,
            dynamic::DynamicCompilationStage::Compile => compile,
        }
    }

    pub(super) fn caller_is_strict(&self, caller: FrameRecord) -> bool {
        self.installed_function(caller.code())
            .map(|function| function.flags().strict())
            .unwrap_or(false)
    }

    pub(super) fn install_dynamic_function(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        parameters_source: &str,
        body_source: &str,
        strict_caller: bool,
        kind: BuiltinDynamicFunctionKind,
    ) -> VmResult<crate::InstalledCode> {
        let compiler_kind = Self::compiler_dynamic_function_kind(kind);
        let cache_key = dynamic::DynamicFunctionCacheKey::new(
            realm,
            compiler_kind,
            parameters_source,
            body_source,
            strict_caller,
        );
        if let Some(installed) = self.dynamic_function_cache.get(&cache_key).copied() {
            return Ok(installed);
        }

        let source_id = self.allocate_dynamic_source_id();
        let compilation = dynamic::compile_dynamic_function(
            agent.atoms_mut(),
            source_id,
            parameters_source,
            body_source,
            compiler_kind,
        )
        .map_err(|error| {
            Self::syntax_error(
                agent,
                realm,
                Self::dynamic_stage_message(
                    &error,
                    "Function constructor parse failure",
                    "Function constructor semantic failure",
                    "Function constructor compile failure",
                ),
            )
        })?;
        let installed = self.install_script(agent, realm, compilation.unit())?;
        self.dynamic_function_cache.insert(cache_key, installed);
        Ok(installed)
    }

    fn syntax_error(agent: &mut Agent, realm: RealmRef, message: &str) -> VmError {
        let message = Value::from_string_ref(alloc_string(agent, message, None));
        match errors::create_intrinsic_error_object(
            agent,
            realm,
            errors::ErrorKind::Syntax,
            Some(message),
        ) {
            Ok(object) => VmError::Abrupt(AbruptCompletion::throw(Value::from_object_ref(object))),
            Err(completion) => VmError::Abrupt(completion),
        }
    }

    fn try_evaluate_regexp_literal_eval_source(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        source_text: &str,
    ) -> VmResult<Option<Value>> {
        let Some((pattern, flags)) = split_eval_regexp_literal_source(source_text) else {
            return Ok(None);
        };
        if validate_regexp_literal(pattern, flags).is_err() {
            return Err(Self::syntax_error(agent, realm, "evalScript parse failure"));
        }
        let regexp = self.allocate_eval_regexp_literal(agent, realm, pattern, flags)?;
        Ok(Some(Value::from_object_ref(regexp)))
    }

    pub(super) fn try_evaluate_regexp_literal_eval_string_ref(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        source: StringRef,
    ) -> VmResult<Option<Value>> {
        let units = string_ref_code_units(agent, source).ok_or(VmError::MissingRootShape(realm))?;
        let Some((pattern_units, flags)) = split_eval_regexp_literal_units(&units) else {
            return Ok(None);
        };
        let pattern = String::from_utf16_lossy(pattern_units);
        if validate_regexp_literal(&pattern, &flags).is_err() {
            return Err(Self::syntax_error(agent, realm, "evalScript parse failure"));
        }
        let regexp = self.allocate_eval_regexp_literal_with_source_units(
            agent,
            realm,
            &pattern,
            pattern_units.to_vec().into_boxed_slice(),
            &flags,
        )?;
        Ok(Some(Value::from_object_ref(regexp)))
    }

    fn allocate_eval_regexp_literal(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        pattern: &str,
        flags: &str,
    ) -> VmResult<ObjectRef> {
        let payload = RegExpPayload::compile(pattern, flags)
            .map_err(|_| Self::syntax_error(agent, realm, "evalScript parse failure"))?;
        self.allocate_eval_regexp_literal_with_payload(agent, realm, payload)
    }

    fn allocate_eval_regexp_literal_with_source_units(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        pattern: &str,
        source_units: Box<[u16]>,
        flags: &str,
    ) -> VmResult<ObjectRef> {
        let payload = RegExpPayload::compile_with_source_units(pattern, source_units, flags)
            .map_err(|_| Self::syntax_error(agent, realm, "evalScript parse failure"))?;
        self.allocate_eval_regexp_literal_with_payload(agent, realm, payload)
    }

    fn allocate_eval_regexp_literal_with_payload(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        payload: RegExpPayload,
    ) -> VmResult<ObjectRef> {
        let realm_record = agent.realm(realm).ok_or(VmError::MissingRootShape(realm))?;
        let root_shape = realm_record
            .root_shape()
            .ok_or(VmError::MissingRootShape(realm))?;
        let prototype = realm_record
            .intrinsics()
            .regexp_prototype()
            .ok_or(VmError::MissingRootShape(realm))?;
        let object = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let object = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape)
                    .with_prototype(Some(prototype))
                    .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::RegExp)),
                AllocationLifetime::Default,
            );
            let stored = objects.store_regexp_payload(object, payload);
            debug_assert!(stored, "fresh RegExp objects should accept payload storage");
            object
        });
        let key = PropertyKey::from_atom(agent.bootstrap_atoms().last_index());
        let mut descriptor = PropertyDescriptor::new();
        descriptor.set_value(Value::from_smi(0));
        descriptor.set_writable(true);
        descriptor.set_enumerable(false);
        descriptor.set_configurable(false);
        let defined =
            object::define_property(agent, object, key, descriptor, AllocationLifetime::Default)
                .map_err(VmError::Abrupt)?;
        if defined {
            Ok(object)
        } else {
            Err(VmError::Abrupt(errors::throw_type_error(agent)))
        }
    }

    pub(crate) fn evaluate_script_source(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        realm: RealmRef,
        source_text: &str,
    ) -> VmResult<Value> {
        let source_id = self.allocate_dynamic_source_id();
        let compilation = dynamic::compile_dynamic_script_source(
            agent.atoms_mut(),
            source_id,
            source_text,
            dynamic::DynamicScriptAnalysisMode::Script,
        )
        .map_err(|error| {
            Self::syntax_error(
                agent,
                realm,
                Self::dynamic_stage_message(
                    &error,
                    "evalScript parse failure",
                    "evalScript semantic failure",
                    "evalScript compile failure",
                ),
            )
        })?;
        let realm_record = agent.realm(realm).ok_or(VmError::MissingRootShape(realm))?;
        let script_referrer = self.active_script_or_module_referrer(agent);
        self.evaluate_script_with_registry_and_host_referrer(
            agent,
            realm_record,
            compilation.unit(),
            script_referrer.as_ref(),
            host,
            registry,
        )
    }

    pub(crate) fn evaluate_indirect_eval_source(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        realm: RealmRef,
        source_text: &str,
    ) -> VmResult<Value> {
        if let Some(value) =
            self.try_evaluate_regexp_literal_eval_source(agent, realm, source_text)?
        {
            return Ok(value);
        }

        let source_id = self.allocate_dynamic_source_id();
        let mut analysis = dynamic::analyze_dynamic_script_with_diagnostics(
            agent.atoms_mut(),
            source_id,
            source_text,
            dynamic::DynamicScriptAnalysisMode::Script,
        )
        .map_err(|error| {
            Self::syntax_error(
                agent,
                realm,
                Self::dynamic_stage_message(
                    &error,
                    "evalScript parse failure",
                    "evalScript semantic failure",
                    "evalScript compile failure",
                ),
            )
        })?;

        Self::rewrite_direct_eval_root_lexical_uses(analysis.sema_mut());
        if analysis.sema().diagnostics.has_errors() {
            return Err(Self::syntax_error(
                agent,
                realm,
                "evalScript semantic failure",
            ));
        }

        let global_env = agent
            .realm(realm)
            .ok_or(VmError::MissingRootShape(realm))?
            .global_env();
        let root_var_names = Self::direct_eval_root_var_names(analysis.sema());
        let root_function_names = Self::direct_eval_root_function_names(analysis.sema());
        if !analysis.parsed().strict {
            if let Some(lyng_js_env::EnvironmentRecord::Global(record)) =
                agent.environment(global_env)
            {
                self.validate_direct_eval_global_declarations(
                    agent,
                    global_env,
                    record.global_object(),
                    &root_function_names,
                    &root_var_names,
                )?;
            }
        }

        let hosted_names = self.rewrite_direct_eval_root_bindings(
            agent,
            global_env,
            analysis.parsed().strict,
            analysis.sema_mut(),
        )?;
        if analysis.sema().diagnostics.has_errors() {
            return Err(Self::syntax_error(
                agent,
                realm,
                "evalScript semantic failure",
            ));
        }

        let unit = dynamic::compile_analyzed_dynamic_script(&analysis, agent.atoms_mut())
            .map_err(|_| Self::syntax_error(agent, realm, "evalScript compile failure"))?;
        let installed = self.install_script(agent, realm, &unit)?;
        self.install_active_realm_extensions(agent, realm)?;

        let (lexical_env, variable_env) = if analysis.parsed().strict {
            let indirect_eval_env =
                self.create_direct_eval_var_environment(agent, global_env, &hosted_names)?;
            indirect_eval_env
                .map(|environment| (environment, environment))
                .unwrap_or((global_env, global_env))
        } else {
            if let Some(lyng_js_env::EnvironmentRecord::Global(record)) =
                agent.environment(global_env)
            {
                self.seed_direct_eval_global_var_bindings(
                    agent,
                    global_env,
                    record.global_object(),
                    &root_var_names,
                    &root_function_names,
                )?;
            }
            (global_env, global_env)
        };
        let script_referrer = self
            .active_script_or_module_referrer(agent)
            .map(|key| agent.atoms_mut().intern_collectible(key.as_str()));
        self.evaluate_installed_with_registry_and_host(
            agent,
            installed,
            lexical_env,
            variable_env,
            script_referrer,
            host,
            registry,
        )
    }

    fn rewrite_direct_eval_use_sites(sema: &mut ScriptSema) {
        for record in sema.use_sites.as_mut_slice() {
            if matches!(
                record.resolution_kind,
                ResolutionKind::Global | ResolutionKind::Unresolved
            ) {
                record.resolution_kind = ResolutionKind::Dynamic;
            }
        }
    }

    fn rewrite_direct_eval_root_lexical_uses(sema: &mut ScriptSema) {
        let root_scope = ScopeId::new(0);
        let lexical_bindings = sema
            .scope_table
            .get(root_scope)
            .bindings
            .iter()
            .copied()
            .filter_map(|binding_id| {
                let binding = sema.binding_table.get(binding_id);
                (binding.scope == root_scope && binding.kind.is_lexical())
                    .then_some((binding.name, binding_id))
            })
            .collect::<HashMap<_, _>>();

        for record in sema.use_sites.as_mut_slice() {
            if record.scope != root_scope
                || !matches!(
                    record.resolution_kind,
                    ResolutionKind::Dynamic | ResolutionKind::Global | ResolutionKind::Unresolved
                )
            {
                continue;
            }
            let Some(binding) = lexical_bindings.get(&record.name).copied() else {
                continue;
            };
            record.resolved_binding = Some(binding);
            record.resolution_kind = ResolutionKind::Local;
        }
    }

    fn caller_in_parameter_initializer(caller: FrameRecord) -> bool {
        let end_offset = caller.parameter_initializer_end_offset();
        end_offset != 0 && caller.instruction_offset() < end_offset
    }

    fn direct_eval_declares_root_var_or_function_named_arguments(sema: &ScriptSema) -> bool {
        let root_scope = ScopeId::new(0);
        sema.scope_table
            .get(root_scope)
            .bindings
            .iter()
            .copied()
            .any(|binding_id| {
                let binding = sema.binding_table.get(binding_id);
                binding.scope == root_scope
                    && binding.name == WellKnownAtom::arguments.id()
                    && matches!(
                        binding.kind,
                        DeclarationKind::Var | DeclarationKind::Function
                    )
            })
    }

    fn push_unique_atom(names: &mut Vec<AtomId>, name: AtomId) {
        if !names.contains(&name) {
            names.push(name);
        }
    }

    fn direct_eval_root_var_names(sema: &ScriptSema) -> Vec<AtomId> {
        let root_scope = ScopeId::new(0);
        let mut names = Vec::new();
        for binding_id in sema.scope_table.get(root_scope).bindings.iter().copied() {
            let binding = sema.binding_table.get(binding_id);
            if binding.scope == root_scope && binding.kind == DeclarationKind::Var {
                Self::push_unique_atom(&mut names, binding.name);
            }
        }
        names
    }

    fn direct_eval_root_function_names(sema: &ScriptSema) -> Vec<AtomId> {
        let root_scope = ScopeId::new(0);
        let mut names = Vec::new();
        for binding_id in sema.scope_table.get(root_scope).bindings.iter().copied() {
            let binding = sema.binding_table.get(binding_id);
            if binding.scope == root_scope && binding.kind == DeclarationKind::Function {
                Self::push_unique_atom(&mut names, binding.name);
            }
        }
        names
    }

    fn caller_reserves_arguments_name_during_parameter_initializer(
        &self,
        agent: &mut Agent,
        caller: FrameRecord,
        lexical_env: lyng_js_types::EnvironmentRef,
    ) -> VmResult<bool> {
        if ops_names::has_identifier_binding(agent, lexical_env, WellKnownAtom::arguments.id())
            .map_err(VmError::Abrupt)?
        {
            return Ok(true);
        }

        Ok(matches!(
            self.installed_function(caller.code())
                .map(|function| function.kind()),
            Some(lyng_js_bytecode::BytecodeFunctionKind::Function)
        ))
    }

    fn caller_allows_direct_eval_function_code(&self, caller: FrameRecord) -> bool {
        matches!(
            self.installed_function(caller.code())
                .map(|function| function.kind()),
            Some(lyng_js_bytecode::BytecodeFunctionKind::Function)
        )
    }

    fn caller_direct_eval_home_object(
        &self,
        agent: &Agent,
        lexical_env: lyng_js_types::EnvironmentRef,
        caller: FrameRecord,
    ) -> Option<ObjectRef> {
        if !self.caller_allows_direct_eval_function_code(caller) {
            return None;
        }

        if let Ok(Some(record)) = Self::this_environment_record(agent, lexical_env) {
            if let Some(home_object) = record.home_object() {
                return Some(home_object);
            }
            if let Some(home_object) = agent
                .objects()
                .function_data(record.function_object())
                .and_then(|data| data.home_object())
            {
                return Some(home_object);
            }
        }

        caller.callee().and_then(|callee| {
            agent
                .objects()
                .function_data(callee)
                .and_then(|data| data.home_object())
        })
    }

    fn caller_direct_eval_private_env(
        &self,
        agent: &Agent,
        caller: FrameRecord,
    ) -> Option<lyng_js_types::EnvironmentRef> {
        agent
            .current_execution_context()
            .and_then(|context| context.private_env())
            .or_else(|| {
                caller.callee().and_then(|callee| {
                    agent
                        .objects()
                        .function_data(callee)
                        .and_then(|data| data.private_env())
                })
            })
    }

    fn caller_direct_eval_call_state(
        &self,
        agent: &Agent,
        lexical_env: lyng_js_types::EnvironmentRef,
        caller: FrameRecord,
    ) -> VmResult<(Value, Option<ObjectRef>)> {
        if let Some(context) = agent.current_execution_context() {
            if let ThisState::Value(value) = context.this_state() {
                return Ok((value, context.new_target()));
            }
        }
        Self::lexical_call_state(agent, lexical_env, caller)
    }

    fn sema_private_element_kind(kind: ClassPrivateElementKind) -> SemaClassPrivateElementKind {
        match kind {
            ClassPrivateElementKind::Field => SemaClassPrivateElementKind::Field,
            ClassPrivateElementKind::Method => SemaClassPrivateElementKind::Method,
            ClassPrivateElementKind::Getter => SemaClassPrivateElementKind::Getter,
            ClassPrivateElementKind::Setter => SemaClassPrivateElementKind::Setter,
        }
    }

    fn direct_eval_ambient_private_layouts(
        &self,
        agent: &mut Agent,
        caller: FrameRecord,
        source: SourceId,
    ) -> VmResult<Vec<Vec<ClassPrivateElementRecord>>> {
        let mut layouts = Vec::new();
        let mut current = self.caller_direct_eval_private_env(agent, caller);
        let imported_span = Span::from_offsets(source, 0, 0);

        while let Some(environment) = current {
            let record = agent
                .private_environment(environment)
                .ok_or(VmError::MissingEnvironment(environment))?;
            let class_object = agent
                .environment_slot(environment, 0)
                .and_then(Value::as_object_ref)
                .ok_or(VmError::MissingEnvironment(environment))?;
            let entries = match agent.objects().private_descriptor_summaries(class_object) {
                Ok(descriptors) => descriptors
                    .into_iter()
                    .map(|descriptor| {
                        ClassPrivateElementRecord::new(
                            descriptor.name(),
                            descriptor.is_static(),
                            Self::sema_private_element_kind(descriptor.kind()),
                            imported_span,
                        )
                    })
                    .collect(),
                Err(InternalMethodError::MissingClassRecord) => Vec::new(),
                Err(_) => return Err(VmError::Abrupt(errors::throw_type_error(agent))),
            };
            layouts.push(entries);
            current = record.outer();
        }

        layouts.reverse();
        Ok(layouts)
    }

    fn filter_direct_eval_function_code_diagnostics(
        &self,
        agent: &Agent,
        caller: FrameRecord,
        caller_lexical_env: lyng_js_types::EnvironmentRef,
        sema: &mut ScriptSema,
    ) {
        let allow_new_target = self.caller_allows_direct_eval_function_code(caller);
        let allow_super = allow_new_target
            && self
                .caller_direct_eval_home_object(agent, caller_lexical_env, caller)
                .is_some();
        if !allow_new_target && !allow_super {
            return;
        }

        let diagnostics = std::mem::take(&mut sema.diagnostics).into_inner();
        let mut filtered = lyng_js_common::DiagnosticList::new();
        for diagnostic in diagnostics {
            let suppress = (allow_new_target
                && diagnostic.message == "'new.target' outside of a function")
                || (allow_super && diagnostic.message == "'super' keyword outside of a method");
            if !suppress {
                filtered.push(diagnostic);
            }
        }
        sema.diagnostics = filtered;
    }

    fn can_declare_direct_eval_global_var(
        agent: &mut Agent,
        global_object: ObjectRef,
        name: AtomId,
    ) -> VmResult<bool> {
        let descriptor =
            object::get_own_property(agent, global_object, PropertyKey::from_atom(name))
                .map_err(VmError::Abrupt)?;
        if descriptor.is_some() {
            return Ok(true);
        }
        object::is_extensible(agent, global_object).map_err(VmError::Abrupt)
    }

    fn can_declare_direct_eval_global_function(
        agent: &mut Agent,
        global_object: ObjectRef,
        name: AtomId,
    ) -> VmResult<bool> {
        let descriptor =
            object::get_own_property(agent, global_object, PropertyKey::from_atom(name))
                .map_err(VmError::Abrupt)?;
        let Some(descriptor) = descriptor else {
            return object::is_extensible(agent, global_object).map_err(VmError::Abrupt);
        };
        if descriptor.configurable() == Some(true) {
            return Ok(true);
        }
        Ok((descriptor.has_value() || descriptor.has_writable())
            && descriptor.writable() == Some(true)
            && descriptor.enumerable() == Some(true))
    }

    fn layout_has_binding(
        agent: &Agent,
        layout: lyng_js_env::EnvironmentLayoutId,
        name: AtomId,
    ) -> bool {
        agent.environment_layout(layout).is_some_and(|layout| {
            layout
                .bindings()
                .iter()
                .any(|binding| binding.name() == Some(name))
        })
    }

    fn direct_eval_chain_has_lexical_binding_before_var_env(
        &self,
        agent: &Agent,
        start: lyng_js_types::EnvironmentRef,
        var_env: lyng_js_types::EnvironmentRef,
        name: AtomId,
    ) -> bool {
        let mut current = Some(start);
        while let Some(environment) = current {
            if environment == var_env {
                break;
            }
            let Some(record) = agent.environment(environment) else {
                break;
            };
            match record {
                lyng_js_env::EnvironmentRecord::Declarative(record) => {
                    if Self::layout_has_binding(agent, record.layout(), name) {
                        return true;
                    }
                    current = record.outer();
                }
                lyng_js_env::EnvironmentRecord::Function(record) => {
                    let declarative = record.declarative();
                    if Self::layout_has_binding(agent, declarative.layout(), name) {
                        return true;
                    }
                    current = declarative.outer();
                }
                lyng_js_env::EnvironmentRecord::Module(record) => {
                    if Self::layout_has_binding(agent, record.layout(), name) {
                        return true;
                    }
                    current = record.outer();
                }
                lyng_js_env::EnvironmentRecord::Global(record) => {
                    if self.global_has_lexical_binding(agent, &record, name) {
                        return true;
                    }
                    current = record.outer();
                }
                lyng_js_env::EnvironmentRecord::Private(record) => current = record.outer(),
                lyng_js_env::EnvironmentRecord::Object(record) => current = record.outer(),
            }
        }
        false
    }

    fn validate_direct_eval_lower_lexical_conflicts(
        &self,
        agent: &mut Agent,
        lexical_env: lyng_js_types::EnvironmentRef,
        var_env: lyng_js_types::EnvironmentRef,
        function_names: &[AtomId],
        var_names: &[AtomId],
    ) -> VmResult<()> {
        if lexical_env == var_env {
            return Ok(());
        }

        for &name in function_names {
            if self.direct_eval_chain_has_lexical_binding_before_var_env(
                agent,
                lexical_env,
                var_env,
                name,
            ) {
                return Err(VmError::Abrupt(errors::throw_syntax_error(agent)));
            }
        }

        for &name in var_names {
            if self.direct_eval_chain_has_lexical_binding_before_var_env(
                agent,
                lexical_env,
                var_env,
                name,
            ) {
                return Err(VmError::Abrupt(errors::throw_syntax_error(agent)));
            }
        }

        Ok(())
    }

    fn validate_direct_eval_global_declarations(
        &self,
        agent: &mut Agent,
        global_env: lyng_js_types::EnvironmentRef,
        global_object: ObjectRef,
        function_names: &[AtomId],
        var_names: &[AtomId],
    ) -> VmResult<()> {
        for &name in function_names {
            if self.global_chain_has_lexical_binding(agent, global_env, name) {
                return Err(VmError::Abrupt(errors::throw_syntax_error(agent)));
            }
            if !Self::can_declare_direct_eval_global_function(agent, global_object, name)? {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
        }

        for &name in var_names {
            if self.global_chain_has_lexical_binding(agent, global_env, name) {
                return Err(VmError::Abrupt(errors::throw_syntax_error(agent)));
            }
            if !Self::can_declare_direct_eval_global_var(agent, global_object, name)? {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
        }

        Ok(())
    }

    fn direct_eval_variable_environment_has_own_binding(
        &self,
        agent: &Agent,
        variable_env: lyng_js_types::EnvironmentRef,
        name: AtomId,
    ) -> bool {
        let Some(record) = agent.environment(variable_env) else {
            return false;
        };
        match record {
            lyng_js_env::EnvironmentRecord::Declarative(record) => {
                Self::layout_has_binding(agent, record.layout(), name)
            }
            lyng_js_env::EnvironmentRecord::Function(record) => {
                Self::layout_has_binding(agent, record.declarative().layout(), name)
            }
            lyng_js_env::EnvironmentRecord::Module(record) => {
                Self::layout_has_binding(agent, record.layout(), name)
            }
            lyng_js_env::EnvironmentRecord::Global(record) => record.has_var_name(name),
            lyng_js_env::EnvironmentRecord::Private(_)
            | lyng_js_env::EnvironmentRecord::Object(_) => false,
        }
    }

    fn rewrite_direct_eval_root_bindings(
        &self,
        agent: &mut Agent,
        variable_env: lyng_js_types::EnvironmentRef,
        always_host: bool,
        sema: &mut ScriptSema,
    ) -> VmResult<Vec<AtomId>> {
        let root_scope = ScopeId::new(0);
        let bindings = sema.scope_table.get(root_scope).bindings.clone();
        let mut hosted_names = Vec::new();
        for binding_id in bindings {
            let (kind, name, scope) = {
                let binding = sema.binding_table.get(binding_id);
                (binding.kind, binding.name, binding.scope)
            };
            if scope != root_scope
                || !matches!(kind, DeclarationKind::Var | DeclarationKind::Function)
            {
                continue;
            }
            let binding_exists =
                self.direct_eval_variable_environment_has_own_binding(agent, variable_env, name);
            if (always_host || !binding_exists) && !hosted_names.contains(&name) {
                hosted_names.push(name);
            }

            let binding = sema.binding_table.get_mut(binding_id);
            binding.storage_class = StorageClass::DynamicLookup;
            binding.needs_environment = false;
            binding.slot_index = None;
        }
        Ok(hosted_names)
    }

    fn create_direct_eval_var_environment(
        &mut self,
        agent: &mut Agent,
        outer: lyng_js_types::EnvironmentRef,
        hosted_names: &[AtomId],
    ) -> VmResult<Option<lyng_js_types::EnvironmentRef>> {
        if hosted_names.is_empty() {
            return Ok(None);
        }

        let bindings = hosted_names
            .iter()
            .copied()
            .map(|name| {
                EnvironmentBindingLayout::new(
                    Some(name),
                    EnvironmentSlotFlags::var_like().with_dynamic(true),
                )
            })
            .collect::<Vec<_>>();
        let layout = agent.alloc_environment_layout(EnvironmentLayout::new(
            EnvironmentLayoutKind::Declarative,
            bindings,
            true,
        ));
        let environment = agent
            .alloc_declarative_environment(Some(outer), layout, AllocationLifetime::Default)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        for slot in 0..hosted_names.len() {
            if !agent.init_environment_slot(
                environment,
                u32::try_from(slot).unwrap_or(u32::MAX),
                Value::undefined(),
            ) {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
        }
        Ok(Some(environment))
    }

    fn seed_direct_eval_global_var_bindings(
        &self,
        agent: &mut Agent,
        global_env: lyng_js_types::EnvironmentRef,
        global_object: ObjectRef,
        var_names: &[AtomId],
        function_names: &[AtomId],
    ) -> VmResult<()> {
        for &name in function_names {
            let key = PropertyKey::from_atom(name);
            let existing =
                object::get_own_property(agent, global_object, key).map_err(VmError::Abrupt)?;
            let mut descriptor = PropertyDescriptor::new();
            descriptor.set_value(Value::undefined());
            if existing.is_none()
                || existing.is_some_and(|descriptor| descriptor.configurable() == Some(true))
            {
                descriptor.set_writable(true);
                descriptor.set_enumerable(true);
                descriptor.set_configurable(true);
            }
            let defined = object::define_property(
                agent,
                global_object,
                key,
                descriptor,
                AllocationLifetime::Default,
            )
            .map_err(VmError::Abrupt)?;
            if !defined {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            if !agent.global_has_var_name(global_env, name) {
                let _ = agent.global_add_var_name(global_env, name);
            }
        }

        for &name in var_names {
            if self.global_chain_has_lexical_binding(agent, global_env, name) {
                return Err(VmError::Abrupt(errors::throw_syntax_error(agent)));
            }

            let key = PropertyKey::from_atom(name);
            let has_property = object::get_own_property(agent, global_object, key)
                .map_err(VmError::Abrupt)?
                .is_some();
            if !has_property {
                let extensible =
                    object::is_extensible(agent, global_object).map_err(VmError::Abrupt)?;
                if !extensible {
                    return Err(VmError::Abrupt(errors::throw_type_error(agent)));
                }

                let mut descriptor = PropertyDescriptor::new();
                descriptor.set_value(Value::undefined());
                descriptor.set_writable(true);
                descriptor.set_enumerable(true);
                descriptor.set_configurable(true);
                let defined = object::define_property(
                    agent,
                    global_object,
                    key,
                    descriptor,
                    AllocationLifetime::Default,
                )
                .map_err(VmError::Abrupt)?;
                if !defined {
                    return Err(VmError::Abrupt(errors::throw_type_error(agent)));
                }
            }

            if !agent.global_has_var_name(global_env, name) {
                let _ = agent.global_add_var_name(global_env, name);
            }
        }

        Ok(())
    }

    pub(crate) fn evaluate_direct_eval_source(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        source_text: &str,
    ) -> VmResult<Value> {
        let realm = caller.realm();
        if let Some(value) =
            self.try_evaluate_regexp_literal_eval_source(agent, realm, source_text)?
        {
            return Ok(value);
        }

        let caller_name_env_start = self.dynamic_name_start_environment(caller);
        let (caller_lexical_env, direct_eval_site_flags) =
            self.caller_direct_eval_lexical_environment(agent, caller, caller_name_env_start)?;
        let source_id = self.allocate_dynamic_source_id();
        let direct_eval_private_layouts =
            self.direct_eval_ambient_private_layouts(agent, caller, source_id)?;
        let mut analysis = dynamic::analyze_dynamic_script_with_diagnostics(
            agent.atoms_mut(),
            source_id,
            source_text,
            dynamic::DynamicScriptAnalysisMode::DirectEval {
                initial_strict: self.caller_is_strict(caller),
                options: DirectEvalScriptAnalysisOptions::new()
                    .with_ambient_private_layouts(direct_eval_private_layouts)
                    .with_forbid_arguments_in_class_initializer(
                        direct_eval_site_flags.forbid_arguments_in_class_initializer(),
                    ),
            },
        )
        .map_err(|error| {
            Self::syntax_error(
                agent,
                realm,
                Self::dynamic_stage_message(
                    &error,
                    "evalScript parse failure",
                    "evalScript semantic failure",
                    "evalScript compile failure",
                ),
            )
        })?;
        Self::rewrite_direct_eval_use_sites(analysis.sema_mut());
        Self::rewrite_direct_eval_root_lexical_uses(analysis.sema_mut());
        self.filter_direct_eval_function_code_diagnostics(
            agent,
            caller,
            caller_lexical_env,
            analysis.sema_mut(),
        );
        let caller_variable_env = caller.variable_env();
        let root_var_names = Self::direct_eval_root_var_names(analysis.sema());
        let root_function_names = Self::direct_eval_root_function_names(analysis.sema());
        if !analysis.parsed().strict
            && Self::caller_in_parameter_initializer(caller)
            && Self::direct_eval_declares_root_var_or_function_named_arguments(analysis.sema())
            && self.caller_reserves_arguments_name_during_parameter_initializer(
                agent,
                caller,
                caller_lexical_env,
            )?
        {
            return Err(VmError::Abrupt(errors::throw_syntax_error(agent)));
        }
        if !analysis.parsed().strict {
            self.validate_direct_eval_lower_lexical_conflicts(
                agent,
                caller_lexical_env,
                caller_variable_env,
                &root_function_names,
                &root_var_names,
            )?;
            if let Some(lyng_js_env::EnvironmentRecord::Global(record)) =
                agent.environment(caller_variable_env)
            {
                self.validate_direct_eval_global_declarations(
                    agent,
                    caller_variable_env,
                    record.global_object(),
                    &root_function_names,
                    &root_var_names,
                )?;
            }
        }
        let hosted_names = self.rewrite_direct_eval_root_bindings(
            agent,
            caller_variable_env,
            analysis.parsed().strict,
            analysis.sema_mut(),
        )?;
        if analysis.sema().diagnostics.has_errors() {
            return Err(Self::syntax_error(
                agent,
                realm,
                "evalScript semantic failure",
            ));
        }
        let unit = dynamic::compile_analyzed_dynamic_script(&analysis, agent.atoms_mut())
            .map_err(|_| Self::syntax_error(agent, realm, "evalScript compile failure"))?;
        let installed = self.install_script(agent, realm, &unit)?;
        self.install_active_realm_extensions(agent, realm)?;
        let strict_eval = analysis.parsed().strict;
        let (lexical_env, variable_env) = if strict_eval {
            let direct_eval_env =
                self.create_direct_eval_var_environment(agent, caller_lexical_env, &hosted_names)?;
            direct_eval_env
                .map(|environment| (environment, environment))
                .unwrap_or((caller_lexical_env, caller_variable_env))
        } else if let Some(lyng_js_env::EnvironmentRecord::Global(record)) =
            agent.environment(caller_variable_env)
        {
            self.seed_direct_eval_global_var_bindings(
                agent,
                caller_variable_env,
                record.global_object(),
                &root_var_names,
                &root_function_names,
            )?;
            (caller_lexical_env, caller_variable_env)
        } else {
            let direct_eval_env =
                self.create_direct_eval_var_environment(agent, caller_lexical_env, &hosted_names)?;
            if let Some(environment) = direct_eval_env {
                self.push_direct_eval_environment(self.frames.len(), environment);
                (environment, caller_variable_env)
            } else {
                (caller_lexical_env, caller_variable_env)
            }
        };
        let script_referrer = self
            .active_script_or_module_referrer(agent)
            .map(|key| agent.atoms_mut().intern_collectible(key.as_str()));
        let (entry_this_value, entry_new_target) =
            self.caller_direct_eval_call_state(agent, caller_name_env_start, caller)?;
        let entry_home_object =
            self.caller_direct_eval_home_object(agent, caller_name_env_start, caller);
        let entry_private_env = self.caller_direct_eval_private_env(agent, caller);
        self.evaluate_installed_with_registry_and_host_with_entry_override(
            agent,
            installed,
            lexical_env,
            variable_env,
            script_referrer,
            entry_this_value,
            entry_new_target,
            entry_home_object,
            entry_private_env,
            host,
            registry,
        )
    }
}
