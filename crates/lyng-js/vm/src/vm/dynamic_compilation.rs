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
use lyng_js_objects::{ClassPrivateElementKind, InternalMethodError, RegExpPayload};
use lyng_js_ops::{errors, names as ops_names, object};
use lyng_js_parser::validate_regexp_literal;
use lyng_js_sema::{
    ClassPrivateElementKind as SemaClassPrivateElementKind, ClassPrivateElementRecord,
    DeclarationKind, DirectEvalScriptAnalysisOptions, ResolutionKind, ScopeId, ScriptSema,
    StorageClass,
};
use lyng_js_types::{AbruptCompletion, PropertyDescriptor, PropertyKey, RealmRef, StringRef};
use std::collections::HashMap;

fn source_is_empty_block_sequence(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut index = 0;
    let mut saw_block = false;

    while index < bytes.len() {
        match bytes[index] {
            b'{' if bytes.get(index + 1) == Some(&b'}') => {
                saw_block = true;
                index += 2;
            }
            b'\t' | b'\n' | b'\r' | b' ' => {
                index += 1;
            }
            _ => return false,
        }
    }

    saw_block
}

fn split_eval_regexp_literal_source(source: &str) -> Option<(&str, &str)> {
    let mut chars = source.char_indices();
    if chars.next()?.1 != '/' {
        return None;
    }
    if matches!(chars.clone().next().map(|(_, ch)| ch), Some('*' | '/')) {
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

const fn is_regexp_literal_flag_char(ch: char) -> bool {
    ch == '$' || ch == '_' || ch.is_ascii_alphanumeric()
}

fn split_eval_regexp_literal_units(units: &[u16]) -> Option<(&[u16], String)> {
    if units.first().copied()? != u16::from(b'/') {
        return None;
    }
    if matches!(units.get(1).copied(), Some(0x002a | 0x002f)) {
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
    const fn compiler_dynamic_function_kind(
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

    const fn dynamic_stage_message<'a>(
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
            .is_some_and(|function| function.flags().strict())
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
        let regexp = Self::allocate_eval_regexp_literal(agent, realm, pattern, flags)?;
        Ok(Some(Value::from_object_ref(regexp)))
    }

    pub(super) fn try_evaluate_regexp_literal_eval_string_ref(
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
        let regexp = Self::allocate_eval_regexp_literal_with_source_units(
            agent,
            realm,
            &pattern,
            pattern_units.to_vec().into_boxed_slice(),
            &flags,
        )?;
        Ok(Some(Value::from_object_ref(regexp)))
    }

    fn allocate_eval_regexp_literal(
        agent: &mut Agent,
        realm: RealmRef,
        pattern: &str,
        flags: &str,
    ) -> VmResult<ObjectRef> {
        let payload = RegExpPayload::compile(pattern, flags)
            .map_err(|_| Self::syntax_error(agent, realm, "evalScript parse failure"))?;
        Self::allocate_eval_regexp_literal_with_payload(agent, realm, payload)
    }

    fn allocate_eval_regexp_literal_with_source_units(
        agent: &mut Agent,
        realm: RealmRef,
        pattern: &str,
        source_units: Box<[u16]>,
        flags: &str,
    ) -> VmResult<ObjectRef> {
        let payload = RegExpPayload::compile_with_source_units(pattern, source_units, flags)
            .map_err(|_| Self::syntax_error(agent, realm, "evalScript parse failure"))?;
        Self::allocate_eval_regexp_literal_with_payload(agent, realm, payload)
    }

    fn allocate_eval_regexp_literal_with_payload(
        agent: &mut Agent,
        realm: RealmRef,
        payload: RegExpPayload,
    ) -> VmResult<ObjectRef> {
        Self::allocate_regexp_object_with_payload(agent, realm, payload)
    }

    pub(crate) fn evaluate_script_source(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        realm: RealmRef,
        source_text: &str,
    ) -> VmResult<Value> {
        if source_is_empty_block_sequence(source_text) {
            return Ok(Value::undefined());
        }

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
        let script_referrer = Self::active_script_or_module_referrer(agent);
        self.evaluate_script_with_registry_and_host_referrer(
            agent,
            realm_record,
            compilation.unit(),
            script_referrer.as_ref(),
            host,
            registry,
        )
    }

    #[expect(
        clippy::too_many_lines,
        reason = "spec-shaped VM routine stays contiguous to preserve completion ordering and cleanup invariants"
    )]
    pub(crate) fn evaluate_indirect_eval_source(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        realm: RealmRef,
        source_text: &str,
    ) -> VmResult<Value> {
        if source_is_empty_block_sequence(source_text) {
            return Ok(Value::undefined());
        }

        if let Some(value) =
            Self::try_evaluate_regexp_literal_eval_source(agent, realm, source_text)?
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
        if !analysis.parsed().strict
            && let Some(lyng_js_env::EnvironmentRecord::Global(record)) =
                agent.environment(global_env)
        {
            Self::validate_direct_eval_global_declarations(
                agent,
                global_env,
                record.global_object(),
                &root_function_names,
                &root_var_names,
            )?;
        }

        let hosted_names = Self::rewrite_direct_eval_root_bindings(
            agent,
            global_env,
            analysis.parsed().strict,
            analysis.sema_mut(),
        );
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
                Self::create_direct_eval_var_environment(agent, global_env, &hosted_names)?;
            indirect_eval_env.map_or((global_env, global_env), |environment| {
                (environment, environment)
            })
        } else {
            if let Some(lyng_js_env::EnvironmentRecord::Global(record)) =
                agent.environment(global_env)
            {
                Self::seed_direct_eval_global_var_bindings(
                    agent,
                    global_env,
                    record.global_object(),
                    &root_var_names,
                    &root_function_names,
                )?;
            }
            (global_env, global_env)
        };
        let script_referrer = Self::active_script_or_module_referrer(agent)
            .map(|key| agent.atoms_mut().intern_collectible(key.as_str()));
        self.push_direct_eval_environment(self.frames.len() + 1, lexical_env);
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

    const fn caller_in_parameter_initializer(caller: FrameRecord) -> bool {
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

    fn direct_eval_declares_root_var_or_function_with_name(
        sema: &ScriptSema,
        name: AtomId,
    ) -> bool {
        let root_scope = ScopeId::new(0);
        sema.scope_table
            .get(root_scope)
            .bindings
            .iter()
            .copied()
            .any(|binding_id| {
                let binding = sema.binding_table.get(binding_id);
                binding.scope == root_scope
                    && binding.name == name
                    && matches!(
                        binding.kind,
                        DeclarationKind::Var | DeclarationKind::Function
                    )
            })
    }

    fn direct_eval_declares_parameter_name(sema: &ScriptSema, parameter_names: &[AtomId]) -> bool {
        parameter_names
            .iter()
            .copied()
            .any(|name| Self::direct_eval_declares_root_var_or_function_with_name(sema, name))
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
                .map(lyng_js_bytecode::BytecodeFunction::kind),
            Some(lyng_js_bytecode::BytecodeFunctionKind::Function)
        ))
    }

    fn caller_allows_direct_eval_function_code(&self, agent: &Agent, caller: FrameRecord) -> bool {
        match self
            .installed_function(caller.code())
            .map(lyng_js_bytecode::BytecodeFunction::kind)
        {
            Some(lyng_js_bytecode::BytecodeFunctionKind::Function) => true,
            Some(lyng_js_bytecode::BytecodeFunctionKind::Arrow) => {
                let global_object = agent
                    .realm(caller.realm())
                    .map(|realm| realm.global_object());
                Self::this_environment_record(agent, caller.lexical_env()).is_ok_and(|record| {
                    record.is_some_and(|record| Some(record.function_object()) != global_object)
                })
            }
            _ => false,
        }
    }

    fn caller_direct_eval_home_object(
        &self,
        agent: &Agent,
        lexical_env: lyng_js_types::EnvironmentRef,
        caller: FrameRecord,
    ) -> Option<ObjectRef> {
        if let Ok(Some(record)) = Self::this_environment_record(agent, lexical_env) {
            if let Some(home_object) = record.home_object() {
                return Some(home_object);
            }
            if let Some(home_object) = agent
                .objects()
                .function_data(record.function_object())
                .and_then(lyng_js_objects::FunctionObjectData::home_object)
            {
                return Some(home_object);
            }
        }

        if !self.caller_allows_direct_eval_function_code(agent, caller) {
            return None;
        }

        caller.callee().and_then(|callee| {
            agent
                .objects()
                .function_data(callee)
                .and_then(lyng_js_objects::FunctionObjectData::home_object)
        })
    }

    fn caller_direct_eval_active_function(
        agent: &Agent,
        lexical_env: lyng_js_types::EnvironmentRef,
        caller: FrameRecord,
    ) -> Option<ObjectRef> {
        Self::this_environment_record(agent, lexical_env)
            .ok()
            .flatten()
            .map(lyng_js_env::FunctionEnvironmentRecord::function_object)
            .or_else(|| caller.callee())
    }

    fn caller_direct_eval_private_env(
        agent: &Agent,
        caller: FrameRecord,
    ) -> Option<lyng_js_types::EnvironmentRef> {
        agent
            .current_execution_context()
            .and_then(lyng_js_env::ExecutionContext::private_env)
            .or_else(|| {
                caller.callee().and_then(|callee| {
                    agent
                        .objects()
                        .function_data(callee)
                        .and_then(lyng_js_objects::FunctionObjectData::private_env)
                })
            })
    }

    fn caller_direct_eval_call_state(
        agent: &Agent,
        lexical_env: lyng_js_types::EnvironmentRef,
        caller: FrameRecord,
    ) -> VmResult<(Value, Option<ObjectRef>)> {
        if let Some(context) = agent.current_execution_context()
            && let ThisState::Value(value) = context.this_state()
        {
            return Ok((value, context.new_target()));
        }
        Self::lexical_call_state(agent, lexical_env, &caller)
    }

    const fn sema_private_element_kind(
        kind: ClassPrivateElementKind,
    ) -> SemaClassPrivateElementKind {
        match kind {
            ClassPrivateElementKind::Field => SemaClassPrivateElementKind::Field,
            ClassPrivateElementKind::Method => SemaClassPrivateElementKind::Method,
            ClassPrivateElementKind::Getter => SemaClassPrivateElementKind::Getter,
            ClassPrivateElementKind::Setter => SemaClassPrivateElementKind::Setter,
        }
    }

    fn direct_eval_ambient_private_layouts(
        agent: &mut Agent,
        caller: FrameRecord,
        source: SourceId,
    ) -> VmResult<Vec<Vec<ClassPrivateElementRecord>>> {
        let mut layouts = Vec::new();
        let mut current = Self::caller_direct_eval_private_env(agent, caller);
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
        allow_new_target: bool,
        allow_super: bool,
        sema: &mut ScriptSema,
    ) {
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
            object::ordinary_get_own_property(agent, global_object, PropertyKey::from_atom(name))
                .map_err(VmError::Abrupt)?;
        if descriptor.is_some() {
            return Ok(true);
        }
        object::ordinary_is_extensible(agent, global_object).map_err(VmError::Abrupt)
    }

    fn can_declare_direct_eval_global_function(
        agent: &mut Agent,
        global_object: ObjectRef,
        name: AtomId,
    ) -> VmResult<bool> {
        let descriptor =
            object::ordinary_get_own_property(agent, global_object, PropertyKey::from_atom(name))
                .map_err(VmError::Abrupt)?;
        let Some(descriptor) = descriptor else {
            return object::ordinary_is_extensible(agent, global_object).map_err(VmError::Abrupt);
        };
        if descriptor.configurable() == Some(true) {
            return Ok(true);
        }
        Ok((descriptor.has_value() || descriptor.has_writable())
            && descriptor.writable() == Some(true)
            && descriptor.enumerable() == Some(true))
    }

    fn environment_has_active_layout_binding(
        agent: &Agent,
        _environment: lyng_js_types::EnvironmentRef,
        layout: lyng_js_env::EnvironmentLayoutId,
        name: AtomId,
    ) -> bool {
        agent.environment_layout(layout).is_some_and(|layout| {
            layout.bindings().iter().any(|binding| {
                if binding.name() != Some(name) {
                    return false;
                }
                !binding.flags().is_dynamic()
            })
        })
    }

    fn layout_binding_is_lexical(
        agent: &Agent,
        layout: lyng_js_env::EnvironmentLayoutId,
        name: AtomId,
    ) -> Option<bool> {
        agent.environment_layout(layout).and_then(|layout| {
            layout
                .bindings()
                .iter()
                .find(|binding| binding.name() == Some(name))
                .map(|binding| binding.flags().is_lexical())
        })
    }

    fn collect_layout_lexical_names(
        agent: &Agent,
        layout: lyng_js_env::EnvironmentLayoutId,
        out: &mut Vec<AtomId>,
    ) {
        let Some(layout) = agent.environment_layout(layout) else {
            return;
        };
        for binding in layout.bindings() {
            if binding.flags().is_lexical()
                && let Some(name) = binding.name()
            {
                Self::push_unique_atom(out, name);
            }
        }
    }

    fn direct_eval_lexical_names_before_var_env(
        agent: &Agent,
        start: lyng_js_types::EnvironmentRef,
        var_env: lyng_js_types::EnvironmentRef,
    ) -> Vec<AtomId> {
        let mut names = Vec::new();
        let mut current = Some(start);
        while let Some(environment) = current {
            let Some(record) = agent.environment(environment) else {
                break;
            };
            let reached_var_env = environment == var_env;
            match record {
                lyng_js_env::EnvironmentRecord::Declarative(record) => {
                    Self::collect_layout_lexical_names(agent, record.layout(), &mut names);
                    current = record.outer();
                }
                lyng_js_env::EnvironmentRecord::Function(record) => {
                    let declarative = record.declarative();
                    Self::collect_layout_lexical_names(agent, declarative.layout(), &mut names);
                    current = declarative.outer();
                }
                lyng_js_env::EnvironmentRecord::Module(record) => {
                    Self::collect_layout_lexical_names(agent, record.layout(), &mut names);
                    current = record.outer();
                }
                lyng_js_env::EnvironmentRecord::Global(record) => {
                    for &name in record.lexical_names() {
                        Self::push_unique_atom(&mut names, name);
                    }
                    current = record.outer();
                }
                lyng_js_env::EnvironmentRecord::Private(record) => current = record.outer(),
                lyng_js_env::EnvironmentRecord::Object(record) => current = record.outer(),
            }
            if reached_var_env {
                break;
            }
        }
        names
    }

    fn direct_eval_chain_lexical_binding_before_var_env(
        agent: &Agent,
        start: lyng_js_types::EnvironmentRef,
        var_env: lyng_js_types::EnvironmentRef,
        name: AtomId,
    ) -> Option<(lyng_js_types::EnvironmentRef, bool)> {
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
                    if let Some(is_lexical) =
                        Self::layout_binding_is_lexical(agent, record.layout(), name)
                    {
                        return Some((record.id(), is_lexical));
                    }
                    current = record.outer();
                }
                lyng_js_env::EnvironmentRecord::Function(record) => {
                    let declarative = record.declarative();
                    if let Some(is_lexical) =
                        Self::layout_binding_is_lexical(agent, declarative.layout(), name)
                    {
                        return Some((declarative.id(), is_lexical));
                    }
                    current = declarative.outer();
                }
                lyng_js_env::EnvironmentRecord::Module(record) => {
                    if let Some(is_lexical) =
                        Self::layout_binding_is_lexical(agent, record.layout(), name)
                    {
                        return Some((record.id(), is_lexical));
                    }
                    current = record.outer();
                }
                lyng_js_env::EnvironmentRecord::Global(record) => {
                    if let Some(binding) = Self::lookup_global_lexical_binding(agent, &record, name)
                    {
                        return Some((binding.environment(), true));
                    }
                    current = record.outer();
                }
                lyng_js_env::EnvironmentRecord::Private(record) => current = record.outer(),
                lyng_js_env::EnvironmentRecord::Object(record) => current = record.outer(),
            }
        }
        None
    }

    fn validate_direct_eval_lower_lexical_conflicts(
        agent: &mut Agent,
        lexical_env: lyng_js_types::EnvironmentRef,
        var_env: lyng_js_types::EnvironmentRef,
        function_names: &[AtomId],
        var_names: &[AtomId],
        annex_b_catch_environments: &[(
            lyng_js_types::EnvironmentRef,
            u32,
            lyng_js_types::EnvironmentRef,
            u32,
            AtomId,
        )],
        annex_b_catch_names: &[AtomId],
    ) -> VmResult<()> {
        if lexical_env == var_env {
            for &name in function_names.iter().chain(var_names) {
                if Self::direct_eval_variable_environment_has_own_lexical_binding(
                    agent, var_env, name,
                ) {
                    return Err(VmError::Abrupt(errors::throw_syntax_error(agent)));
                }
            }
            return Ok(());
        }

        for &name in function_names {
            if let Some((environment, is_lexical)) =
                Self::direct_eval_chain_lexical_binding_before_var_env(
                    agent,
                    lexical_env,
                    var_env,
                    name,
                )
                && !annex_b_catch_environments
                    .iter()
                    .any(|&(_, _, catch_env, _, catch_name)| {
                        catch_env == environment && catch_name == name
                    })
                && (is_lexical || !annex_b_catch_names.contains(&name))
            {
                return Err(VmError::Abrupt(errors::throw_syntax_error(agent)));
            }
        }

        for &name in var_names {
            if let Some((environment, is_lexical)) =
                Self::direct_eval_chain_lexical_binding_before_var_env(
                    agent,
                    lexical_env,
                    var_env,
                    name,
                )
                && !annex_b_catch_environments
                    .iter()
                    .any(|&(_, _, catch_env, _, catch_name)| {
                        catch_env == environment && catch_name == name
                    })
                && (is_lexical || !annex_b_catch_names.contains(&name))
            {
                return Err(VmError::Abrupt(errors::throw_syntax_error(agent)));
            }
        }

        Ok(())
    }

    fn validate_direct_eval_global_declarations(
        agent: &mut Agent,
        global_env: lyng_js_types::EnvironmentRef,
        global_object: ObjectRef,
        function_names: &[AtomId],
        var_names: &[AtomId],
    ) -> VmResult<()> {
        for &name in function_names {
            if Self::global_chain_has_lexical_binding(agent, global_env, name) {
                return Err(VmError::Abrupt(errors::throw_syntax_error(agent)));
            }
            if !Self::can_declare_direct_eval_global_function(agent, global_object, name)? {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
        }

        for &name in var_names {
            if Self::global_chain_has_lexical_binding(agent, global_env, name) {
                return Err(VmError::Abrupt(errors::throw_syntax_error(agent)));
            }
            if !Self::can_declare_direct_eval_global_var(agent, global_object, name)? {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
        }

        Ok(())
    }

    fn direct_eval_variable_environment_has_own_binding(
        agent: &Agent,
        variable_env: lyng_js_types::EnvironmentRef,
        name: AtomId,
    ) -> bool {
        let Some(record) = agent.environment(variable_env) else {
            return false;
        };
        match record {
            lyng_js_env::EnvironmentRecord::Declarative(record) => {
                Self::environment_has_active_layout_binding(
                    agent,
                    variable_env,
                    record.layout(),
                    name,
                )
            }
            lyng_js_env::EnvironmentRecord::Function(record) => {
                Self::environment_has_active_layout_binding(
                    agent,
                    variable_env,
                    record.declarative().layout(),
                    name,
                )
            }
            lyng_js_env::EnvironmentRecord::Module(record) => {
                Self::environment_has_active_layout_binding(
                    agent,
                    variable_env,
                    record.layout(),
                    name,
                )
            }
            lyng_js_env::EnvironmentRecord::Global(record) => record.has_var_name(name),
            lyng_js_env::EnvironmentRecord::Private(_)
            | lyng_js_env::EnvironmentRecord::Object(_) => false,
        }
    }

    fn direct_eval_variable_environment_has_own_lexical_binding(
        agent: &Agent,
        variable_env: lyng_js_types::EnvironmentRef,
        name: AtomId,
    ) -> bool {
        let Some(record) = agent.environment(variable_env) else {
            return false;
        };
        match record {
            lyng_js_env::EnvironmentRecord::Declarative(record) => {
                Self::layout_binding_is_lexical(agent, record.layout(), name) == Some(true)
            }
            lyng_js_env::EnvironmentRecord::Function(record) => {
                Self::layout_binding_is_lexical(agent, record.declarative().layout(), name)
                    == Some(true)
            }
            lyng_js_env::EnvironmentRecord::Module(record) => {
                Self::layout_binding_is_lexical(agent, record.layout(), name) == Some(true)
            }
            lyng_js_env::EnvironmentRecord::Global(record) => {
                Self::lookup_global_lexical_binding(agent, &record, name).is_some()
            }
            lyng_js_env::EnvironmentRecord::Private(_)
            | lyng_js_env::EnvironmentRecord::Object(_) => false,
        }
    }

    fn rewrite_direct_eval_root_bindings(
        agent: &Agent,
        variable_env: lyng_js_types::EnvironmentRef,
        always_host: bool,
        sema: &mut ScriptSema,
    ) -> Vec<AtomId> {
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
                Self::direct_eval_variable_environment_has_own_binding(agent, variable_env, name);
            if (always_host || !binding_exists) && !hosted_names.contains(&name) {
                hosted_names.push(name);
            }

            let binding = sema.binding_table.get_mut(binding_id);
            binding.storage_class = if !always_host && binding_exists {
                StorageClass::DynamicVariableLookup
            } else {
                StorageClass::DynamicLookup
            };
            binding.needs_environment = false;
            binding.slot_index = None;
        }
        hosted_names
    }

    fn force_host_annex_b_direct_eval_catch_bindings(
        sema: &mut ScriptSema,
        annex_b_catch_names: &[AtomId],
        hosted_names: &mut Vec<AtomId>,
    ) {
        if annex_b_catch_names.is_empty() {
            return;
        }
        let root_scope = ScopeId::new(0);
        let bindings = sema.scope_table.get(root_scope).bindings.clone();
        for binding_id in bindings {
            let binding = sema.binding_table.get_mut(binding_id);
            if binding.scope != root_scope
                || !matches!(
                    binding.kind,
                    DeclarationKind::Var | DeclarationKind::Function
                )
                || !annex_b_catch_names.contains(&binding.name)
            {
                continue;
            }
            if !hosted_names.contains(&binding.name) {
                hosted_names.push(binding.name);
            }
            binding.storage_class = StorageClass::DynamicLookup;
            binding.needs_environment = false;
            binding.slot_index = None;
        }
    }

    fn create_direct_eval_var_environment(
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

    fn direct_eval_named_environment_slot(
        agent: &Agent,
        environment: lyng_js_types::EnvironmentRef,
        name: AtomId,
    ) -> Option<u32> {
        let layout = match agent.environment(environment)? {
            lyng_js_env::EnvironmentRecord::Declarative(record) => record.layout(),
            lyng_js_env::EnvironmentRecord::Function(record) => record.declarative().layout(),
            lyng_js_env::EnvironmentRecord::Module(record) => record.layout(),
            lyng_js_env::EnvironmentRecord::Global(record) => record.layout(),
            lyng_js_env::EnvironmentRecord::Private(_)
            | lyng_js_env::EnvironmentRecord::Object(_) => return None,
        };
        agent
            .environment_layout(layout)?
            .bindings()
            .iter()
            .enumerate()
            .find_map(|(index, binding)| {
                (binding.name() == Some(name)).then(|| u32::try_from(index).ok())?
            })
    }

    fn direct_eval_chain_named_environment_slot(
        agent: &Agent,
        start: lyng_js_types::EnvironmentRef,
        stop: lyng_js_types::EnvironmentRef,
        name: AtomId,
    ) -> Option<(lyng_js_types::EnvironmentRef, u32)> {
        let mut current = Some(start);
        while let Some(environment) = current {
            if let Some(slot) = Self::direct_eval_named_environment_slot(agent, environment, name) {
                return Some((environment, slot));
            }
            if environment == stop {
                return None;
            }
            current = match agent.environment(environment)? {
                lyng_js_env::EnvironmentRecord::Declarative(record) => record.outer(),
                lyng_js_env::EnvironmentRecord::Function(record) => record.declarative().outer(),
                lyng_js_env::EnvironmentRecord::Module(record) => record.outer(),
                lyng_js_env::EnvironmentRecord::Global(record) => record.outer(),
                lyng_js_env::EnvironmentRecord::Private(record) => record.outer(),
                lyng_js_env::EnvironmentRecord::Object(record) => record.outer(),
            };
        }
        None
    }

    fn sync_direct_eval_annex_b_catch_bindings(
        agent: &mut Agent,
        annex_b_catch_environments: &[(
            lyng_js_types::EnvironmentRef,
            u32,
            lyng_js_types::EnvironmentRef,
            u32,
            AtomId,
        )],
        sync_names: &[AtomId],
    ) -> VmResult<()> {
        for &(source_environment, source_slot, cloned_environment, cloned_slot, name) in
            annex_b_catch_environments
        {
            if !sync_names.contains(&name) {
                continue;
            }
            let value = agent
                .environment_slot(cloned_environment, cloned_slot)
                .ok_or(VmError::MissingEnvironment(cloned_environment))?;
            if !agent.set_environment_slot(source_environment, source_slot, value) {
                return Err(VmError::MissingEnvironment(source_environment));
            }
        }
        Ok(())
    }

    fn sync_direct_eval_annex_b_catch_var_bindings(
        agent: &mut Agent,
        source_start: lyng_js_types::EnvironmentRef,
        var_environment: lyng_js_types::EnvironmentRef,
        hosted_names: &[AtomId],
        annex_b_catch_names: &[AtomId],
        sync_names: &[AtomId],
    ) -> VmResult<()> {
        for &name in annex_b_catch_names {
            if !hosted_names.contains(&name) || !sync_names.contains(&name) {
                continue;
            }
            let Some((source_environment, source_slot)) =
                Self::direct_eval_chain_named_environment_slot(
                    agent,
                    source_start,
                    var_environment,
                    name,
                )
            else {
                continue;
            };
            let Some(var_slot) =
                Self::direct_eval_named_environment_slot(agent, var_environment, name)
            else {
                continue;
            };
            let value = agent
                .environment_slot(var_environment, var_slot)
                .ok_or(VmError::MissingEnvironment(var_environment))?;
            if !agent.set_environment_slot(source_environment, source_slot, value) {
                return Err(VmError::MissingEnvironment(source_environment));
            }
        }
        Ok(())
    }

    fn seed_direct_eval_annex_b_catch_var_bindings(
        agent: &mut Agent,
        source_start: lyng_js_types::EnvironmentRef,
        var_environment: lyng_js_types::EnvironmentRef,
        hosted_names: &[AtomId],
        annex_b_catch_names: &[AtomId],
    ) -> VmResult<()> {
        for &name in annex_b_catch_names {
            if !hosted_names.contains(&name) {
                continue;
            }
            let Some((source_environment, source_slot)) =
                Self::direct_eval_chain_named_environment_slot(
                    agent,
                    source_start,
                    var_environment,
                    name,
                )
            else {
                continue;
            };
            let Some(var_slot) =
                Self::direct_eval_named_environment_slot(agent, var_environment, name)
            else {
                continue;
            };
            let value = agent
                .environment_slot(source_environment, source_slot)
                .ok_or(VmError::MissingEnvironment(source_environment))?;
            if !agent.set_environment_slot(var_environment, var_slot, value) {
                return Err(VmError::MissingEnvironment(var_environment));
            }
        }
        Ok(())
    }

    fn seed_direct_eval_global_var_bindings(
        agent: &mut Agent,
        global_env: lyng_js_types::EnvironmentRef,
        global_object: ObjectRef,
        var_names: &[AtomId],
        function_names: &[AtomId],
    ) -> VmResult<()> {
        for &name in function_names {
            let key = PropertyKey::from_atom(name);
            let existing = object::ordinary_get_own_property(agent, global_object, key)
                .map_err(VmError::Abrupt)?;
            let mut descriptor = PropertyDescriptor::new();
            descriptor.set_value(Value::undefined());
            if existing.is_none()
                || existing.is_some_and(|descriptor| descriptor.configurable() == Some(true))
            {
                descriptor.set_writable(true);
                descriptor.set_enumerable(true);
                descriptor.set_configurable(true);
            }
            let defined = object::ordinary_define_property(
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

        for &name in var_names {
            if Self::global_chain_has_lexical_binding(agent, global_env, name) {
                return Err(VmError::Abrupt(errors::throw_syntax_error(agent)));
            }

            let key = PropertyKey::from_atom(name);
            let has_property = object::ordinary_get_own_property(agent, global_object, key)
                .map_err(VmError::Abrupt)?
                .is_some();
            if !has_property {
                let extensible = object::ordinary_is_extensible(agent, global_object)
                    .map_err(VmError::Abrupt)?;
                if !extensible {
                    return Err(VmError::Abrupt(errors::throw_type_error(agent)));
                }

                let mut descriptor = PropertyDescriptor::new();
                descriptor.set_value(Value::undefined());
                descriptor.set_writable(true);
                descriptor.set_enumerable(true);
                descriptor.set_configurable(true);
                let defined = object::ordinary_define_property(
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
        }

        Ok(())
    }

    #[expect(
        clippy::too_many_lines,
        reason = "spec-shaped VM routine stays contiguous to preserve completion ordering and cleanup invariants"
    )]
    pub(crate) fn evaluate_direct_eval_source(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        source_text: &str,
        this_override: Option<Value>,
    ) -> VmResult<Value> {
        let realm = caller.realm();
        if source_is_empty_block_sequence(source_text) {
            return Ok(Value::undefined());
        }

        if let Some(value) =
            Self::try_evaluate_regexp_literal_eval_source(agent, realm, source_text)?
        {
            return Ok(value);
        }

        let caller_name_env_start = self.lexical_name_start_environment(&caller);
        let (
            caller_lexical_env,
            direct_eval_site_flags,
            annex_b_catch_environments,
            annex_b_catch_names,
            direct_eval_parameter_names,
        ) = self.caller_direct_eval_lexical_environment(agent, &caller, caller_name_env_start)?;
        let caller_variable_env = caller.variable_env();
        let caller_home_object =
            self.caller_direct_eval_home_object(agent, caller_name_env_start, caller);
        let allow_new_target = direct_eval_site_flags.allow_new_target()
            || self.caller_allows_direct_eval_function_code(agent, caller);
        let allow_super = direct_eval_site_flags.allow_super()
            || (allow_new_target && caller_home_object.is_some());
        let caller_active_function =
            Self::caller_direct_eval_active_function(agent, caller_name_env_start, caller);
        let allow_super_call = allow_super
            && caller_active_function
                .is_some_and(|function| agent.objects().is_constructor(function));
        let mut annex_b_blocked_var_names = Self::direct_eval_lexical_names_before_var_env(
            agent,
            caller_lexical_env,
            caller_variable_env,
        );
        annex_b_blocked_var_names.retain(|name| !annex_b_catch_names.contains(name));
        let source_id = self.allocate_dynamic_source_id();
        let direct_eval_private_layouts =
            Self::direct_eval_ambient_private_layouts(agent, caller, source_id)?;
        let mut analysis = dynamic::analyze_dynamic_script_with_diagnostics(
            agent.atoms_mut(),
            source_id,
            source_text,
            dynamic::DynamicScriptAnalysisMode::DirectEval {
                initial_strict: self.caller_is_strict(caller),
                options: DirectEvalScriptAnalysisOptions::new()
                    .with_ambient_private_layouts(direct_eval_private_layouts)
                    .with_annex_b_blocked_var_names(annex_b_blocked_var_names)
                    .with_forbid_arguments_in_class_initializer(
                        direct_eval_site_flags.forbid_arguments_in_class_initializer(),
                    )
                    .with_forbid_direct_super_call(!allow_super_call)
                    .with_forbid_super_call_in_class_initializer(
                        direct_eval_site_flags.forbid_super_call_in_class_initializer(),
                    )
                    .with_allow_new_target(allow_new_target)
                    .with_allow_super(allow_super),
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
        Self::filter_direct_eval_function_code_diagnostics(
            allow_new_target,
            allow_super,
            analysis.sema_mut(),
        );
        let root_var_names = Self::direct_eval_root_var_names(analysis.sema());
        let root_function_names = Self::direct_eval_root_function_names(analysis.sema());
        let annex_b_catch_sync_names = analysis.root_var_initializer_names();
        if !analysis.parsed().strict
            && Self::caller_in_parameter_initializer(caller)
            && Self::direct_eval_declares_parameter_name(
                analysis.sema(),
                &direct_eval_parameter_names,
            )
        {
            return Err(VmError::Abrupt(errors::throw_syntax_error(agent)));
        }
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
            Self::validate_direct_eval_lower_lexical_conflicts(
                agent,
                caller_lexical_env,
                caller_variable_env,
                &root_function_names,
                &root_var_names,
                &annex_b_catch_environments,
                &annex_b_catch_names,
            )?;
            if let Some(lyng_js_env::EnvironmentRecord::Global(record)) =
                agent.environment(caller_variable_env)
            {
                Self::validate_direct_eval_global_declarations(
                    agent,
                    caller_variable_env,
                    record.global_object(),
                    &root_function_names,
                    &root_var_names,
                )?;
            }
        }
        let caller_is_script = self
            .installed_function(caller.code())
            .is_some_and(|function| {
                function.kind() == lyng_js_bytecode::BytecodeFunctionKind::Script
            });
        let caller_variable_env_is_global = matches!(
            agent.environment(caller_variable_env),
            Some(lyng_js_env::EnvironmentRecord::Global(_))
        );
        let host_root_bindings =
            analysis.parsed().strict || (caller_variable_env_is_global && !caller_is_script);
        let mut hosted_names = Self::rewrite_direct_eval_root_bindings(
            agent,
            caller_variable_env,
            host_root_bindings,
            analysis.sema_mut(),
        );
        Self::force_host_annex_b_direct_eval_catch_bindings(
            analysis.sema_mut(),
            &annex_b_catch_names,
            &mut hosted_names,
        );
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
        let mut persistent_direct_eval_env = None;
        let (lexical_env, variable_env) = if strict_eval {
            let direct_eval_env =
                Self::create_direct_eval_var_environment(agent, caller_lexical_env, &hosted_names)?;
            direct_eval_env.map_or((caller_lexical_env, caller_variable_env), |environment| {
                (environment, environment)
            })
        } else if let Some(record) = caller_is_script.then_some(()).and_then(|()| {
            match agent.environment(caller_variable_env) {
                Some(lyng_js_env::EnvironmentRecord::Global(record)) => Some(record),
                _ => None,
            }
        }) {
            Self::seed_direct_eval_global_var_bindings(
                agent,
                caller_variable_env,
                record.global_object(),
                &root_var_names,
                &root_function_names,
            )?;
            (caller_lexical_env, caller_variable_env)
        } else {
            let inherited_direct_eval_env =
                self.direct_eval_environment_overlay(caller_lexical_env);
            let direct_eval_outer = inherited_direct_eval_env.unwrap_or(caller_lexical_env);
            let direct_eval_env =
                Self::create_direct_eval_var_environment(agent, direct_eval_outer, &hosted_names)?;
            if let Some(environment) = direct_eval_env {
                Self::seed_direct_eval_annex_b_catch_var_bindings(
                    agent,
                    caller_lexical_env,
                    environment,
                    &hosted_names,
                    &annex_b_catch_names,
                )?;
                self.register_direct_eval_environment_overlay(caller_lexical_env, environment);
                persistent_direct_eval_env = Some(environment);
                (environment, caller_variable_env)
            } else {
                (
                    inherited_direct_eval_env.unwrap_or(caller_lexical_env),
                    caller_variable_env,
                )
            }
        };
        let script_referrer = Self::active_script_or_module_referrer(agent)
            .map(|key| agent.atoms_mut().intern_collectible(key.as_str()));
        let (entry_this_value, entry_new_target) = if let Some(this_override) = this_override {
            (this_override, None)
        } else {
            Self::caller_direct_eval_call_state(agent, caller_name_env_start, caller)?
        };
        let entry_home_object = caller_home_object;
        let entry_active_function = allow_super.then_some(caller_active_function).flatten();
        let entry_private_env = Self::caller_direct_eval_private_env(agent, caller);
        let entry_lexical_this = this_override.is_none() && entry_active_function.is_some();
        if let Some(environment) = persistent_direct_eval_env {
            self.push_direct_eval_environment(self.frames.len(), environment);
        }
        self.push_direct_eval_environment(self.frames.len() + 1, lexical_env);
        let result = self.evaluate_installed_with_registry_and_host_with_entry_override(
            agent,
            installed,
            lexical_env,
            variable_env,
            script_referrer,
            entry_this_value,
            entry_new_target,
            entry_home_object,
            entry_active_function,
            entry_private_env,
            entry_lexical_this,
            host,
            registry,
        );
        if let Some(environment) = persistent_direct_eval_env {
            Self::sync_direct_eval_annex_b_catch_var_bindings(
                agent,
                caller_lexical_env,
                environment,
                &hosted_names,
                &annex_b_catch_names,
                &annex_b_catch_sync_names,
            )?;
        }
        Self::sync_direct_eval_annex_b_catch_bindings(
            agent,
            &annex_b_catch_environments,
            &annex_b_catch_sync_names,
        )?;
        result
    }
}
