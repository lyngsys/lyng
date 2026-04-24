use super::property_access::VmProxyBridge;
use super::runtime_objects::length_value;
use super::values::{alloc_string, to_f64_number};
use super::{
    Agent, AllocationLifetime, FrameRecord, NativeFunctionRegistry, ObjectRef, TemplateCacheKey,
    ThisState, Value, Vm, VmError, VmResult, WellKnownAtom,
};
use crate::extensions::{EmbeddingFunctionContext, EmbeddingInvocation};
use crate::frame::GeneratorResumeKind;
use lyng_js_builtins::{
    builtin_metadata, dispatch_builtin, BuiltinInvocation, DynamicFunctionKind,
    InternalBuiltinDispatchContext, PublicBuiltinDispatchContext,
};
use lyng_js_common::{AtomId, AtomTable, SourceId, Span};
use lyng_js_compiler::compile_script;
use lyng_js_env::{
    EnvironmentBindingLayout, EnvironmentLayout, EnvironmentLayoutKind, EnvironmentSlotFlags,
    PromiseResolvingFunctionKind, ThisBindingStatus,
};
use lyng_js_host::{
    HostErrorKind, HostHooks, ImportMetaValue, ModuleImportAttribute, ModuleKey,
    ModuleSourceRequest, TemporalCivilTime, TemporalCivilToInstantRequest,
    TemporalCurrentInstantRequest, TemporalDefaultTimeZone, TemporalDefaultTimeZoneRequest,
    TemporalInstant, TemporalInstantToCivilRequest, TemporalInstantWithOffset,
};
use lyng_js_objects::{
    ClassPrivateElementKind, FunctionConstructorFlags, FunctionEntryIdentity, FunctionObjectData,
    FunctionThisMode, InternalMethodError, ObjectAllocation, ObjectColdData, OrdinaryObjectData,
    PrimitiveWrapperKind, RegExpPayload,
};
use lyng_js_ops::object::ToPrimitiveHint;
use lyng_js_ops::{errors, names as ops_names, object, proxy, read};
use lyng_js_parser::{parse_script, parse_script_with_initial_strict, validate_regexp_literal};
use lyng_js_sema::{
    analyze_direct_eval_script, analyze_script,
    ClassPrivateElementKind as SemaClassPrivateElementKind, ClassPrivateElementRecord,
    DeclarationKind, DirectEvalScriptAnalysisOptions, ResolutionKind, ScopeId, ScriptSema,
    StorageClass,
};
use lyng_js_types::{
    js3_eval_builtin, js3_internal_dynamic_import_builtin, js3_internal_import_meta_builtin,
    js3_object_to_string_builtin, js3_promise_capability_executor_builtin, AbruptCompletion,
    BuiltinFunctionId, EmbeddingFunctionId, PropertyDescriptor, PropertyKey, RealmRef, StringRef,
    WellKnownSymbolId,
};
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
    fn abrupt_intrinsic_error(
        agent: &mut Agent,
        realm: RealmRef,
        kind: errors::ErrorKind,
    ) -> VmError {
        let thrown = errors::create_intrinsic_error_object(agent, realm, kind, None)
            .map(Value::from_object_ref)
            .unwrap_or(Value::undefined());
        VmError::Abrupt(AbruptCompletion::throw(thrown))
    }

    pub(super) fn builtin_entry(
        agent: &Agent,
        callee_object: ObjectRef,
    ) -> Option<BuiltinFunctionId> {
        let data = agent.objects().function_data(callee_object)?;
        let FunctionEntryIdentity::Native(entry) = data.entry()? else {
            return None;
        };
        entry.builtin_entry()
    }

    pub(super) fn embedding_entry(
        agent: &Agent,
        callee_object: ObjectRef,
    ) -> Option<EmbeddingFunctionId> {
        let data = agent.objects().function_data(callee_object)?;
        let FunctionEntryIdentity::Native(entry) = data.entry()? else {
            return None;
        };
        entry.embedding_entry()
    }

    pub(super) fn allocate_builtin_function_object(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        entry: BuiltinFunctionId,
    ) -> VmResult<ObjectRef> {
        let realm_record = agent
            .realm(realm)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let root_shape = realm_record
            .root_shape()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let intrinsics = realm_record.intrinsics();
        let callable_prototype = intrinsics
            .function_prototype()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let object_prototype = intrinsics
            .object_prototype()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let global_env = realm_record.global_env();
        let metadata = builtin_metadata(entry)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let function_data = FunctionObjectData::native(realm, global_env, entry)
            .with_this_mode(FunctionThisMode::Strict)
            .with_has_prototype_property(metadata.has_prototype_property())
            .with_constructor_flags(if metadata.constructible() {
                FunctionConstructorFlags::constructible()
            } else {
                FunctionConstructorFlags::empty()
            });
        let function = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::function(root_shape)
                    .with_prototype(Some(callable_prototype))
                    .with_cold_data(ObjectColdData::Function(function_data)),
                AllocationLifetime::Default,
            )
        });
        let display_name_atom = agent
            .atoms_mut()
            .intern_collectible(metadata.display_name());
        let display_name = Value::from_string_ref(agent.alloc_runtime_string(
            metadata.display_name(),
            Some(display_name_atom),
            AllocationLifetime::Default,
        ));
        let mut length = PropertyDescriptor::new();
        length.set_value(Value::from_smi(i32::from(metadata.length())));
        length.set_writable(false);
        length.set_enumerable(false);
        length.set_configurable(true);
        let mut name = PropertyDescriptor::new();
        name.set_value(display_name);
        name.set_writable(false);
        name.set_enumerable(false);
        name.set_configurable(true);
        agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let _ = objects.define_own_property(
                &mut mutator,
                function,
                PropertyKey::from_atom(WellKnownAtom::length.id()),
                length,
                AllocationLifetime::Default,
            );
            let _ = objects.define_own_property(
                &mut mutator,
                function,
                PropertyKey::from_atom(WellKnownAtom::name.id()),
                name,
                AllocationLifetime::Default,
            );
            if metadata.has_prototype_property() {
                let mut prototype = PropertyDescriptor::new();
                prototype.set_value(Value::from_object_ref(object_prototype));
                prototype.set_writable(false);
                prototype.set_enumerable(false);
                prototype.set_configurable(false);
                let _ = objects.define_own_property(
                    &mut mutator,
                    function,
                    PropertyKey::from_atom(WellKnownAtom::prototype.id()),
                    prototype,
                    AllocationLifetime::Default,
                );
            }
        });
        Ok(function)
    }

    pub(crate) fn allocate_embedding_function_object(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        entry: EmbeddingFunctionId,
        provider: &crate::SharedRealmExtensionProvider,
    ) -> VmResult<ObjectRef> {
        let realm_record = agent
            .realm(realm)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let root_shape = realm_record
            .root_shape()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let intrinsics = realm_record.intrinsics();
        let callable_prototype = intrinsics
            .function_prototype()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let object_prototype = intrinsics
            .object_prototype()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let global_env = realm_record.global_env();
        let metadata = provider
            .embedding_function_metadata(entry)
            .ok_or(VmError::MissingEmbeddingFunction(entry))?;
        let function_data = FunctionObjectData::embedding(realm, global_env, entry)
            .with_this_mode(FunctionThisMode::Strict)
            .with_has_prototype_property(metadata.has_prototype_property())
            .with_constructor_flags(if metadata.constructible() {
                FunctionConstructorFlags::constructible()
            } else {
                FunctionConstructorFlags::empty()
            });
        let function = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::function(root_shape)
                    .with_prototype(Some(callable_prototype))
                    .with_cold_data(ObjectColdData::Function(function_data)),
                AllocationLifetime::Default,
            )
        });
        let display_name_atom = agent
            .atoms_mut()
            .intern_collectible(metadata.display_name());
        let display_name = Value::from_string_ref(agent.alloc_runtime_string(
            metadata.display_name(),
            Some(display_name_atom),
            AllocationLifetime::Default,
        ));
        let mut length = PropertyDescriptor::new();
        length.set_value(Value::from_smi(i32::from(metadata.length())));
        length.set_writable(false);
        length.set_enumerable(false);
        length.set_configurable(true);
        let mut name = PropertyDescriptor::new();
        name.set_value(display_name);
        name.set_writable(false);
        name.set_enumerable(false);
        name.set_configurable(true);
        agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let _ = objects.define_own_property(
                &mut mutator,
                function,
                PropertyKey::from_atom(WellKnownAtom::length.id()),
                length,
                AllocationLifetime::Default,
            );
            let _ = objects.define_own_property(
                &mut mutator,
                function,
                PropertyKey::from_atom(WellKnownAtom::name.id()),
                name,
                AllocationLifetime::Default,
            );
            if metadata.has_prototype_property() {
                let mut prototype = PropertyDescriptor::new();
                prototype.set_value(Value::from_object_ref(object_prototype));
                prototype.set_writable(false);
                prototype.set_enumerable(false);
                prototype.set_configurable(false);
                let _ = objects.define_own_property(
                    &mut mutator,
                    function,
                    PropertyKey::from_atom(WellKnownAtom::prototype.id()),
                    prototype,
                    AllocationLifetime::Default,
                );
            }
        });
        Ok(function)
    }

    pub(super) fn call_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameRecord,
        callee_object: ObjectRef,
        this_value: Value,
        arguments: &[Value],
        new_target: Option<ObjectRef>,
    ) -> VmResult<Option<Value>> {
        if let Some(entry) = Self::embedding_entry(agent, callee_object) {
            let Some(provider) = self.active_extension_provider.clone() else {
                return Err(VmError::MissingRealmExtensionProvider);
            };
            let mut context = EmbeddingFunctionContext::new(
                self,
                agent,
                host,
                registry,
                &provider,
                caller_frame,
                callee_object,
            );
            let invocation = EmbeddingInvocation::new(this_value, arguments, new_target);
            let value = match new_target {
                Some(new_target) => Value::from_object_ref(provider.construct_embedding_function(
                    &mut context,
                    entry,
                    invocation,
                    new_target,
                )?),
                None => provider.call_embedding_function(&mut context, entry, invocation)?,
            };
            return Ok(Some(value));
        }
        let Some(entry) = Self::builtin_entry(agent, callee_object) else {
            return Ok(None);
        };
        if entry == js3_internal_import_meta_builtin() {
            return self.import_meta_builtin(agent, caller_frame).map(Some);
        }
        if entry == js3_internal_dynamic_import_builtin() {
            return self
                .dynamic_import_builtin(agent, host, registry, caller_frame, arguments)
                .map(Some);
        }
        let mut bridge = VmBuiltinDispatch {
            vm: self,
            agent,
            host,
            registry,
            caller_frame,
            callee_object,
        };
        dispatch_builtin(
            &mut bridge,
            entry,
            BuiltinInvocation::new(this_value, arguments, new_target),
        )
    }

    fn builtin_realm(
        agent: &Agent,
        callee_object: ObjectRef,
        caller_frame: FrameRecord,
    ) -> RealmRef {
        agent
            .objects()
            .function_data(callee_object)
            .and_then(|data| data.realm())
            .unwrap_or(caller_frame.realm())
    }

    fn import_meta_builtin(
        &mut self,
        agent: &mut Agent,
        caller_frame: FrameRecord,
    ) -> VmResult<Value> {
        let module_key = agent
            .module_key_for_environment(caller_frame.lexical_env())
            .ok_or(VmError::MissingModuleRecord)?;
        let (cached_object, host_properties) = {
            let record = agent
                .module_record(&module_key)
                .ok_or(VmError::MissingModuleRecord)?;
            (
                record.import_meta_object(),
                record.import_meta_properties().cloned(),
            )
        };
        if let Some(import_meta) = cached_object {
            return Ok(Value::from_object_ref(import_meta));
        }

        let realm = agent
            .realm(caller_frame.realm())
            .ok_or(VmError::MissingRootShape(caller_frame.realm()))?;
        let root_shape = realm
            .root_shape()
            .ok_or(VmError::MissingRootShape(caller_frame.realm()))?;
        let import_meta = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape),
                AllocationLifetime::Default,
            )
        });
        if let Some(host_properties) = host_properties {
            for property in host_properties.properties {
                let key = agent.atoms_mut().intern_collectible(&property.key);
                let value = match property.value {
                    ImportMetaValue::String(text) => {
                        Value::from_string_ref(alloc_string(agent, &text, None))
                    }
                    ImportMetaValue::Boolean(value) => Value::from_bool(value),
                    ImportMetaValue::Smi(value) => Value::from_smi(value),
                    ImportMetaValue::Null => Value::null(),
                };
                object::create_data_property(
                    agent,
                    import_meta,
                    PropertyKey::from_atom(key),
                    value,
                    AllocationLifetime::Default,
                )
                .map_err(VmError::Abrupt)?;
            }
        } else {
            let url = Value::from_string_ref(alloc_string(agent, module_key.as_str(), None));
            let url_atom = agent.atoms_mut().intern_collectible("url");
            object::create_data_property(
                agent,
                import_meta,
                PropertyKey::from_atom(url_atom),
                url,
                AllocationLifetime::Default,
            )
            .map_err(VmError::Abrupt)?;
        }
        let _ = agent.set_module_record_import_meta_object(&module_key, Some(import_meta));
        Ok(Value::from_object_ref(import_meta))
    }

    fn dynamic_import_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let realm = caller_frame.realm();
        let constructor = agent
            .realm(realm)
            .and_then(|realm| realm.intrinsics().promise())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let capability = self.create_dynamic_import_capability(
            agent,
            host,
            registry,
            caller_frame,
            constructor,
        )?;
        let promise = agent
            .promise_capability(capability)
            .and_then(|record| record.promise())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let specifier = arguments.first().copied().unwrap_or(Value::undefined());
        let options = arguments.get(1).copied().unwrap_or(Value::undefined());
        let realm_record = agent.realm(realm).ok_or(VmError::MissingRootShape(realm))?;

        let outcome = (|| -> Result<Value, Value> {
            let specifier = self
                .to_primitive(
                    agent,
                    host,
                    registry,
                    caller_frame,
                    specifier,
                    ToPrimitiveHint::String,
                )
                .map_err(|error| self.dynamic_import_error_value(agent, error))?;
            let specifier = self
                .value_to_string_text(agent, specifier)
                .map_err(|error| self.dynamic_import_error_value(agent, error))?;
            let attributes = self
                .normalize_dynamic_import_attributes(agent, host, registry, caller_frame, options)
                .map_err(|error| self.dynamic_import_error_value(agent, error))?;
            let request = ModuleSourceRequest {
                specifier,
                referrer: self.active_script_or_module_referrer(agent),
                attributes,
            };
            let loaded = self
                .load_module_graph_from_host(agent, realm_record, host, &request)
                .map_err(|error| self.dynamic_import_module_error_value(agent, error))?;
            let key = loaded.key().clone();
            let module_env = self
                .link_module_graph(agent, realm_record, &key)
                .map_err(|error| self.dynamic_import_error_value(agent, error))?;
            let _ = self
                .evaluate_module_graph(agent, realm_record, &key, module_env, host, registry)
                .map_err(|error| self.dynamic_import_error_value(agent, error))?;
            let namespace = self
                .module_namespace_object(agent, realm_record, &key)
                .map_err(|error| self.dynamic_import_error_value(agent, error))?;
            Ok(Value::from_object_ref(namespace))
        })();

        match outcome {
            Ok(value) => {
                self.enqueue_dynamic_import_settle_job(agent, realm, capability, value, false)
            }
            Err(reason) => {
                self.enqueue_dynamic_import_settle_job(agent, realm, capability, reason, true)
            }
        }
        Ok(Value::from_object_ref(promise))
    }

    fn create_dynamic_import_capability(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameRecord,
        constructor: ObjectRef,
    ) -> VmResult<lyng_js_env::PromiseCapabilityId> {
        let capability = agent.alloc_promise_capability();
        let executor = self.allocate_builtin_function_object(
            agent,
            caller_frame.realm(),
            js3_promise_capability_executor_builtin(),
        )?;
        let _ = agent.alloc_promise_resolving_function(
            executor,
            lyng_js_env::PromiseResolvingFunctionRecord::new(
                PromiseResolvingFunctionKind::CapabilityExecutor,
                capability,
            ),
        );
        let promise = self.construct_to_completion(
            agent,
            host,
            registry,
            caller_frame,
            constructor,
            &[Value::from_object_ref(executor)],
            Some(constructor),
        )?;
        let _ = agent.set_promise_capability_promise(capability, promise);
        if agent
            .promise_capability(capability)
            .is_none_or(|record| record.resolve().is_none() || record.reject().is_none())
        {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(capability)
    }

    fn normalize_dynamic_import_attributes(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameRecord,
        options: Value,
    ) -> VmResult<Vec<ModuleImportAttribute>> {
        if options.is_undefined() {
            return Ok(Vec::new());
        }
        let options_object = self.to_object_for_value(agent, caller_frame.realm(), options)?;
        let with_key = PropertyKey::from_atom(agent.atoms_mut().intern_collectible("with"));
        let with_value = self.get_property_from_object(
            agent,
            host,
            registry,
            caller_frame,
            options_object,
            Value::from_object_ref(options_object),
            with_key,
        )?;
        if with_value.is_undefined() {
            return Ok(Vec::new());
        }
        let attributes_object =
            self.to_object_for_value(agent, caller_frame.realm(), with_value)?;
        let keys = object::own_property_keys(agent, attributes_object).map_err(VmError::Abrupt)?;
        let mut attributes = Vec::new();
        for key in keys {
            let enumerable = object::get_own_property(agent, attributes_object, key)
                .map_err(VmError::Abrupt)?
                .is_some_and(|descriptor| descriptor.enumerable() == Some(true));
            if !enumerable {
                continue;
            }
            let Some(attribute_key) = self.dynamic_import_attribute_key(agent, key) else {
                continue;
            };
            let attribute_value = self.get_property_from_object(
                agent,
                host,
                registry,
                caller_frame,
                attributes_object,
                Value::from_object_ref(attributes_object),
                key,
            )?;
            let attribute_value = self.to_primitive(
                agent,
                host,
                registry,
                caller_frame,
                attribute_value,
                ToPrimitiveHint::String,
            )?;
            let attribute_value = self.value_to_string_text(agent, attribute_value)?;
            attributes.push(ModuleImportAttribute {
                key: attribute_key,
                value: attribute_value,
            });
        }
        Ok(attributes)
    }

    fn dynamic_import_attribute_key(&self, agent: &Agent, key: PropertyKey) -> Option<String> {
        if let Some(index) = key.as_index() {
            return Some(index.to_string());
        }
        key.as_atom()
            .map(|atom| agent.atoms().resolve(atom).to_owned())
    }

    fn active_script_or_module_referrer(&self, agent: &Agent) -> Option<ModuleKey> {
        agent
            .current_execution_context()
            .and_then(|context| context.script_or_module_referrer())
            .map(|atom| ModuleKey::new(agent.atoms().resolve(atom).to_owned().into_boxed_str()))
    }

    fn enqueue_dynamic_import_settle_job(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        capability: lyng_js_env::PromiseCapabilityId,
        value: Value,
        rejected: bool,
    ) {
        let _ = agent.enqueue_job_with_payload(
            lyng_js_host::HostJobKind::Promise,
            lyng_js_env::ExecutableId::Builtin,
            lyng_js_env::RuntimeJobPayload::DynamicImportSettle {
                capability,
                value,
                rejected,
            },
            Some(realm),
            Some("DynamicImportSettle".into()),
        );
    }

    fn dynamic_import_module_error_value(
        &self,
        agent: &mut Agent,
        error: crate::error::ModuleLoadError,
    ) -> Value {
        match error {
            crate::error::ModuleLoadError::Vm(error) => {
                self.dynamic_import_error_value(agent, error)
            }
            crate::error::ModuleLoadError::Host(error) => {
                self.dynamic_import_host_error_value(agent, error)
            }
            crate::error::ModuleLoadError::Parse => {
                Value::from_string_ref(alloc_string(agent, "dynamic import parse failure", None))
            }
            crate::error::ModuleLoadError::Sema => {
                Value::from_string_ref(alloc_string(agent, "dynamic import semantic failure", None))
            }
            crate::error::ModuleLoadError::Lowering => {
                Value::from_string_ref(alloc_string(agent, "dynamic import lowering failure", None))
            }
        }
    }

    fn dynamic_import_error_value(&self, agent: &mut Agent, error: VmError) -> Value {
        match error {
            VmError::Abrupt(completion) => completion.thrown_value().unwrap_or(Value::undefined()),
            VmError::Host(error) => self.dynamic_import_host_error_value(agent, error),
            other => Value::from_string_ref(alloc_string(agent, &format!("{other:?}"), None)),
        }
    }

    fn dynamic_import_host_error_value(
        &self,
        agent: &mut Agent,
        error: lyng_js_host::HostError,
    ) -> Value {
        Value::from_string_ref(alloc_string(agent, &error.to_string(), None))
    }

    fn caller_is_strict(&self, caller: FrameRecord) -> bool {
        self.installed_function(caller.code())
            .map(|function| function.flags().strict())
            .unwrap_or(false)
    }

    fn install_dynamic_function(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        parameters_source: &str,
        body_source: &str,
        strict_caller: bool,
        kind: DynamicFunctionKind,
    ) -> VmResult<crate::InstalledCode> {
        let cache_key = super::DynamicFunctionCacheKey {
            realm,
            kind,
            parameters_source: parameters_source.into(),
            body_source: body_source.into(),
            strict_caller,
        };
        if let Some(installed) = self.dynamic_function_cache.get(&cache_key).copied() {
            return Ok(installed);
        }

        let source_text = match kind {
            DynamicFunctionKind::Ordinary => {
                format!("(function anonymous({parameters_source}) {{\n{body_source}\n}})")
            }
            DynamicFunctionKind::Generator => {
                format!("(function* anonymous({parameters_source}) {{\n{body_source}\n}})")
            }
            DynamicFunctionKind::Async => {
                format!("(async function anonymous({parameters_source}) {{\n{body_source}\n}})")
            }
            DynamicFunctionKind::AsyncGenerator => {
                format!("(async function* anonymous({parameters_source}) {{\n{body_source}\n}})")
            }
        };
        let source_id = self.allocate_dynamic_source_id();
        let parsed = parse_script(agent.atoms_mut(), source_id, &source_text);
        if parsed.diagnostics.has_errors() {
            return Err(Self::syntax_error(
                agent,
                realm,
                "Function constructor parse failure",
            ));
        }
        let sema = analyze_script(&parsed, agent.atoms());
        if sema.diagnostics.has_errors() {
            return Err(Self::syntax_error(
                agent,
                realm,
                "Function constructor semantic failure",
            ));
        }
        let unit = compile_script(&parsed, &sema, agent.atoms_mut()).map_err(|_| {
            Self::syntax_error(agent, realm, "Function constructor compile failure")
        })?;
        let installed = self.install_script(agent, realm, &unit)?;
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

    fn try_evaluate_regexp_literal_eval_string_ref(
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
        let parsed = parse_script(agent.atoms_mut(), source_id, source_text);
        if parsed.diagnostics.has_errors() {
            return Err(Self::syntax_error(agent, realm, "evalScript parse failure"));
        }
        let sema = analyze_script(&parsed, agent.atoms());
        if sema.diagnostics.has_errors() {
            return Err(Self::syntax_error(
                agent,
                realm,
                "evalScript semantic failure",
            ));
        }
        let unit = compile_script(&parsed, &sema, agent.atoms_mut())
            .map_err(|_| Self::syntax_error(agent, realm, "evalScript compile failure"))?;
        let realm_record = agent.realm(realm).ok_or(VmError::MissingRootShape(realm))?;
        let script_referrer = self.active_script_or_module_referrer(agent);
        self.evaluate_script_with_registry_and_host_referrer(
            agent,
            realm_record,
            &unit,
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
        let parsed = parse_script(agent.atoms_mut(), source_id, source_text);
        if parsed.diagnostics.has_errors() {
            return Err(Self::syntax_error(agent, realm, "evalScript parse failure"));
        }

        let mut sema = analyze_script(&parsed, agent.atoms());
        Self::rewrite_direct_eval_root_lexical_uses(&mut sema);
        if sema.diagnostics.has_errors() {
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
        let root_var_names = Self::direct_eval_root_var_names(&sema);
        let root_function_names = Self::direct_eval_root_function_names(&sema);
        if !parsed.strict {
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

        let hosted_names =
            self.rewrite_direct_eval_root_bindings(agent, global_env, parsed.strict, &mut sema)?;
        if sema.diagnostics.has_errors() {
            return Err(Self::syntax_error(
                agent,
                realm,
                "evalScript semantic failure",
            ));
        }

        let unit = compile_script(&parsed, &sema, agent.atoms_mut())
            .map_err(|_| Self::syntax_error(agent, realm, "evalScript compile failure"))?;
        let installed = self.install_script(agent, realm, &unit)?;
        self.install_active_realm_extensions(agent, realm)?;

        let (lexical_env, variable_env) = if parsed.strict {
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

        let source_id = self.allocate_dynamic_source_id();
        let parsed = parse_script_with_initial_strict(
            agent.atoms_mut(),
            source_id,
            source_text,
            self.caller_is_strict(caller),
        );
        if parsed.diagnostics.has_errors() {
            return Err(Self::syntax_error(agent, realm, "evalScript parse failure"));
        }
        let caller_name_env_start = self.dynamic_name_start_environment(caller);
        let (caller_lexical_env, direct_eval_site_flags) =
            self.caller_direct_eval_lexical_environment(agent, caller, caller_name_env_start)?;
        let direct_eval_private_layouts =
            self.direct_eval_ambient_private_layouts(agent, caller, source_id)?;
        let mut sema = analyze_direct_eval_script(
            &parsed,
            agent.atoms(),
            DirectEvalScriptAnalysisOptions::new()
                .with_ambient_private_layouts(direct_eval_private_layouts)
                .with_forbid_arguments_in_class_initializer(
                    direct_eval_site_flags.forbid_arguments_in_class_initializer(),
                ),
        );
        Self::rewrite_direct_eval_use_sites(&mut sema);
        Self::rewrite_direct_eval_root_lexical_uses(&mut sema);
        self.filter_direct_eval_function_code_diagnostics(
            agent,
            caller,
            caller_lexical_env,
            &mut sema,
        );
        let caller_variable_env = caller.variable_env();
        let root_var_names = Self::direct_eval_root_var_names(&sema);
        let root_function_names = Self::direct_eval_root_function_names(&sema);
        if !parsed.strict
            && Self::caller_in_parameter_initializer(caller)
            && Self::direct_eval_declares_root_var_or_function_named_arguments(&sema)
            && self.caller_reserves_arguments_name_during_parameter_initializer(
                agent,
                caller,
                caller_lexical_env,
            )?
        {
            return Err(VmError::Abrupt(errors::throw_syntax_error(agent)));
        }
        if !parsed.strict {
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
            parsed.strict,
            &mut sema,
        )?;
        if sema.diagnostics.has_errors() {
            return Err(Self::syntax_error(
                agent,
                realm,
                "evalScript semantic failure",
            ));
        }
        let unit = compile_script(&parsed, &sema, agent.atoms_mut())
            .map_err(|_| Self::syntax_error(agent, realm, "evalScript compile failure"))?;
        let installed = self.install_script(agent, realm, &unit)?;
        self.install_active_realm_extensions(agent, realm)?;
        let strict_eval = parsed.strict;
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

    fn create_bound_function(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        target: ObjectRef,
        bound_this: Value,
        bound_arguments: &[Value],
    ) -> VmResult<ObjectRef> {
        let target_data = agent
            .objects()
            .function_data(target)
            .cloned()
            .ok_or(VmError::Abrupt(errors::throw_type_error(agent)))?;
        let realm = target_data.realm().unwrap_or(caller.realm());
        let realm_record = agent
            .realm(realm)
            .ok_or(VmError::Abrupt(errors::throw_type_error(agent)))?;
        let root_shape = realm_record
            .root_shape()
            .ok_or(VmError::Abrupt(errors::throw_type_error(agent)))?;
        let function_prototype = realm_record
            .intrinsics()
            .function_prototype()
            .ok_or(VmError::Abrupt(errors::throw_type_error(agent)))?;
        let environment = target_data
            .environment()
            .unwrap_or(realm_record.global_env());
        let function_data = FunctionObjectData::bound(
            realm,
            environment,
            target,
            bound_this,
            bound_arguments.to_vec().into_boxed_slice(),
        )
        .with_has_prototype_property(false)
        .with_constructor_flags(if target_data.is_constructible() {
            FunctionConstructorFlags::constructible()
        } else {
            FunctionConstructorFlags::empty()
        })
        .with_kind_flags(target_data.kind_flags())
        .with_this_mode(FunctionThisMode::Strict);
        let function = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::function(root_shape)
                    .with_prototype(Some(function_prototype))
                    .with_cold_data(ObjectColdData::Function(function_data)),
                AllocationLifetime::Default,
            )
        });

        let length_key = PropertyKey::from_atom(WellKnownAtom::length.id());
        let target_has_own_length = object::get_own_property(agent, target, length_key)
            .map_err(VmError::Abrupt)?
            .is_some();
        let bound_length = if target_has_own_length {
            let target_length = self.get_property_from_object(
                agent,
                host,
                registry,
                caller,
                target,
                Value::from_object_ref(target),
                length_key,
            )?;
            bound_function_length_value(target_length, bound_arguments.len())
        } else {
            Value::from_smi(0)
        };
        let target_name = self.get_property_from_object(
            agent,
            host,
            registry,
            caller,
            target,
            Value::from_object_ref(target),
            PropertyKey::from_atom(WellKnownAtom::name.id()),
        )?;
        let target_name = if target_name.as_string_ref().is_some() {
            self.value_to_string_text(agent, target_name)?
        } else {
            String::new()
        };
        self.define_data_property_with_attrs(
            agent,
            function,
            PropertyKey::from_atom(WellKnownAtom::length.id()),
            bound_length,
            false,
            false,
            true,
        )?;
        let bound_name =
            Value::from_string_ref(alloc_string(agent, &format!("bound {target_name}"), None));
        self.define_data_property_with_attrs(
            agent,
            function,
            PropertyKey::from_atom(WellKnownAtom::name.id()),
            bound_name,
            false,
            false,
            true,
        )?;
        Ok(function)
    }

    fn function_name_text(&mut self, agent: &mut Agent, function: ObjectRef) -> VmResult<String> {
        let name = object::get(
            agent,
            function,
            PropertyKey::from_atom(WellKnownAtom::name.id()),
        )
        .map_err(VmError::Abrupt)?;
        self.value_to_string_text(agent, name)
    }

    fn native_function_source_text(
        &mut self,
        agent: &mut Agent,
        function: ObjectRef,
    ) -> VmResult<String> {
        let name = self.function_name_text(agent, function)?;
        Ok(if name.is_empty() {
            "function () { [native code] }".to_owned()
        } else {
            format!("function {name}() {{ [native code] }}")
        })
    }

    fn source_function_source_text(
        &mut self,
        agent: &mut Agent,
        code: lyng_js_types::CodeRef,
        function: ObjectRef,
    ) -> VmResult<String> {
        let Some(installed) = self.installed_function(code) else {
            return self.native_function_source_text(agent, function);
        };
        if let Some(span) = installed.source_span() {
            if let Some(source_text) = self.source_text(span.source) {
                let start = usize::try_from(span.range.start.raw()).unwrap_or(usize::MAX);
                let end = usize::try_from(span.range.end.raw()).unwrap_or(usize::MAX);
                if start <= end && end <= source_text.len() {
                    let candidate = &source_text[start..end];
                    if let Some(trimmed) = Self::trim_function_source_prefix(span.source, candidate)
                    {
                        return Ok(trimmed);
                    }
                    return Ok(candidate.to_owned());
                }
            }
        }
        let name = self.function_name_text(agent, function)?;
        Ok(if name.is_empty() {
            "function () { [source unavailable] }".to_owned()
        } else {
            format!("function {name}() {{ [source unavailable] }}")
        })
    }

    fn trim_function_source_prefix(
        source: lyng_js_common::SourceId,
        candidate: &str,
    ) -> Option<String> {
        let mut atoms = AtomTable::new();
        for (index, ch) in candidate.char_indices() {
            if ch != '}' {
                continue;
            }
            let end = index + ch.len_utf8();
            let source_text = &candidate[..end];
            if Self::function_source_candidate_parses(&mut atoms, source, source_text) {
                return Some(candidate[..end].to_owned());
            }
        }
        None
    }

    fn function_source_candidate_parses(
        atoms: &mut AtomTable,
        source: lyng_js_common::SourceId,
        source_text: &str,
    ) -> bool {
        if !parse_script(atoms, source, source_text)
            .diagnostics
            .has_errors()
        {
            return true;
        }

        let expression_text = format!("({source_text});");
        if !parse_script(atoms, source, &expression_text)
            .diagnostics
            .has_errors()
        {
            return true;
        }

        let method_text = format!("({{{source_text}}});");
        !parse_script(atoms, source, &method_text)
            .diagnostics
            .has_errors()
    }

    fn collect_array_like_arguments(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameRecord,
        realm: RealmRef,
        value: Value,
    ) -> VmResult<Vec<Value>> {
        if value.is_null() || value.is_undefined() {
            return Ok(Vec::new());
        }
        let object = value
            .as_object_ref()
            .ok_or_else(|| Self::abrupt_intrinsic_error(agent, realm, errors::ErrorKind::Type))?;
        let length = self.get_property_from_object(
            agent,
            host,
            registry,
            caller_frame,
            object,
            Value::from_object_ref(object),
            PropertyKey::from_atom(WellKnownAtom::length.id()),
        )?;
        let length = to_f64_number(agent, length)?.max(0.0) as u32;
        let mut arguments = Vec::with_capacity(usize::try_from(length).unwrap_or(usize::MAX));
        for index in 0..length {
            arguments.push(self.get_property_from_object(
                agent,
                host,
                registry,
                caller_frame,
                object,
                Value::from_object_ref(object),
                PropertyKey::Index(index),
            )?);
        }
        Ok(arguments)
    }

    pub(super) fn allocate_ordinary_object_with_prototype(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        prototype: Option<ObjectRef>,
    ) -> VmResult<ObjectRef> {
        let root_shape = agent
            .realm(realm)
            .and_then(|record| record.root_shape())
            .ok_or(VmError::MissingRootShape(realm))?;
        Ok(agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape).with_prototype(prototype),
                AllocationLifetime::Default,
            )
        }))
    }

    pub(super) fn descriptor_object_from_descriptor(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        descriptor: PropertyDescriptor,
    ) -> VmResult<Value> {
        let object = self.allocate_ordinary_object_with_prototype(
            agent,
            realm,
            agent
                .realm(realm)
                .and_then(|record| record.intrinsics().object_prototype()),
        )?;
        if let Some(value) = descriptor.value() {
            self.define_data_property_with_attrs(
                agent,
                object,
                PropertyKey::from_atom(WellKnownAtom::value.id()),
                value,
                true,
                true,
                true,
            )?;
        }
        if let Some(getter) = descriptor.getter() {
            self.define_data_property_with_attrs(
                agent,
                object,
                PropertyKey::from_atom(WellKnownAtom::get.id()),
                getter,
                true,
                true,
                true,
            )?;
        }
        if let Some(setter) = descriptor.setter() {
            self.define_data_property_with_attrs(
                agent,
                object,
                PropertyKey::from_atom(WellKnownAtom::set.id()),
                setter,
                true,
                true,
                true,
            )?;
        }
        if let Some(writable) = descriptor.writable() {
            self.define_data_property_with_attrs(
                agent,
                object,
                PropertyKey::from_atom(WellKnownAtom::writable.id()),
                Value::from_bool(writable),
                true,
                true,
                true,
            )?;
        }
        if let Some(enumerable) = descriptor.enumerable() {
            self.define_data_property_with_attrs(
                agent,
                object,
                PropertyKey::from_atom(WellKnownAtom::enumerable.id()),
                Value::from_bool(enumerable),
                true,
                true,
                true,
            )?;
        }
        if let Some(configurable) = descriptor.configurable() {
            self.define_data_property_with_attrs(
                agent,
                object,
                PropertyKey::from_atom(WellKnownAtom::configurable.id()),
                Value::from_bool(configurable),
                true,
                true,
                true,
            )?;
        }
        Ok(Value::from_object_ref(object))
    }

    fn set_integrity_level(
        &mut self,
        agent: &mut Agent,
        object_ref: ObjectRef,
        freeze: bool,
    ) -> VmResult<bool> {
        if !object::prevent_extensions(agent, object_ref).map_err(VmError::Abrupt)? {
            return Ok(false);
        }
        let keys = object::own_property_keys(agent, object_ref).map_err(VmError::Abrupt)?;
        for key in keys {
            let Some(mut descriptor) =
                object::get_own_property(agent, object_ref, key).map_err(VmError::Abrupt)?
            else {
                continue;
            };
            descriptor.set_configurable(false);
            let is_data_descriptor = (descriptor.has_value() || descriptor.has_writable())
                && !(descriptor.has_get() || descriptor.has_set());
            if freeze && is_data_descriptor {
                descriptor.set_writable(false);
            }
            let _ = object::define_property(
                agent,
                object_ref,
                key,
                descriptor,
                AllocationLifetime::Default,
            )
            .map_err(VmError::Abrupt)?;
        }
        Ok(true)
    }

    fn test_integrity_level(
        &mut self,
        agent: &mut Agent,
        object_ref: ObjectRef,
        frozen: bool,
    ) -> VmResult<bool> {
        if object::is_extensible(agent, object_ref).map_err(VmError::Abrupt)? {
            return Ok(false);
        }
        let keys = object::own_property_keys(agent, object_ref).map_err(VmError::Abrupt)?;
        for key in keys {
            let Some(descriptor) =
                object::get_own_property(agent, object_ref, key).map_err(VmError::Abrupt)?
            else {
                continue;
            };
            if descriptor.configurable() != Some(false) {
                return Ok(false);
            }
            let is_data_descriptor = (descriptor.has_value() || descriptor.has_writable())
                && !(descriptor.has_get() || descriptor.has_set());
            if frozen && is_data_descriptor && descriptor.writable() != Some(false) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn object_has_own_property_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        this_value: Value,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let object = self.to_object_for_value(agent, caller.realm(), this_value)?;
        let key_value = arguments.first().copied().unwrap_or(Value::undefined());
        let key = self.to_property_key_from_value(agent, host, registry, caller, key_value)?;
        let has_property = object::get_own_property(agent, object, key)
            .map_err(VmError::Abrupt)?
            .is_some();
        Ok(Value::from_bool(has_property))
    }

    fn string_index_of_builtin(
        &mut self,
        agent: &mut Agent,
        this_value: Value,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let haystack = self.value_to_string_text(agent, this_value)?;
        let needle = self.value_to_string_text(
            agent,
            arguments.first().copied().unwrap_or(Value::undefined()),
        )?;
        let start = if let Some(value) = arguments.get(1).copied() {
            let number = to_f64_number(agent, value)?;
            if !number.is_finite() || number <= 0.0 {
                0
            } else {
                number as usize
            }
        } else {
            0
        };
        let position = if start <= haystack.len() {
            haystack[start..]
                .find(&needle)
                .map(|offset| offset + start)
                .map_or(-1, |index| i32::try_from(index).unwrap_or(i32::MAX))
        } else {
            -1
        };
        Ok(Value::from_smi(position))
    }

    fn array_index_of_builtin(
        &mut self,
        agent: &mut Agent,
        this_value: Value,
        arguments: &[Value],
    ) -> VmResult<Value> {
        const MAX_SAFE_INTEGER_U64: u64 = (1_u64 << 53) - 1;

        let object = this_value
            .as_object_ref()
            .ok_or(VmError::Abrupt(errors::throw_type_error(agent)))?;
        let search = arguments.first().copied().unwrap_or(Value::undefined());
        let length = object::get(
            agent,
            object,
            PropertyKey::from_atom(WellKnownAtom::length.id()),
        )
        .map_err(VmError::Abrupt)?;

        let length = {
            let number = to_f64_number(agent, length)?;
            if number <= 0.0 {
                0
            } else if !number.is_finite() {
                MAX_SAFE_INTEGER_U64
            } else {
                number.min(MAX_SAFE_INTEGER_U64 as f64) as u64
            }
        };
        if length == 0 {
            return Ok(Value::from_smi(-1));
        }

        let start = match arguments.get(1).copied() {
            Some(value) => {
                let number = to_f64_number(agent, value)?;
                if number.is_infinite() {
                    if number.is_sign_positive() {
                        return Ok(Value::from_smi(-1));
                    }
                    0
                } else if number >= 0.0 {
                    number.min(length as f64) as u64
                } else {
                    let relative = (length as f64) + number;
                    if relative <= 0.0 {
                        0
                    } else {
                        relative as u64
                    }
                }
            }
            None => 0,
        };

        let mut index = start;
        while index < length {
            let key = PropertyKey::from_array_index(index).unwrap_or_else(|| {
                let atom = agent.atoms_mut().intern_collectible(&index.to_string());
                PropertyKey::from_atom(atom)
            });
            if !object::has_property(agent, object, key).map_err(VmError::Abrupt)? {
                index += 1;
                continue;
            }
            let value = object::get(agent, object, key).map_err(VmError::Abrupt)?;
            if read::is_strictly_equal(agent.heap().view(), value, search)
                .map_err(VmError::Abrupt)?
            {
                return Ok(i32::try_from(index)
                    .map(Value::from_smi)
                    .unwrap_or_else(|_| Value::from_f64(index as f64)));
            }
            index += 1;
        }

        Ok(Value::from_smi(-1))
    }

    fn array_push_builtin(
        &mut self,
        agent: &mut Agent,
        caller: FrameRecord,
        this_value: Value,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let object = self.to_object_for_value(agent, caller.realm(), this_value)?;
        let length = object::get(
            agent,
            object,
            PropertyKey::from_atom(WellKnownAtom::length.id()),
        )
        .map_err(VmError::Abrupt)?;
        let length_number = to_f64_number(agent, length)?;
        let length_uint32 =
            if length_number.is_nan() || length_number == 0.0 || !length_number.is_finite() {
                0
            } else {
                let integer = length_number.trunc();
                let mut modulo = integer % 4_294_967_296.0;
                if modulo < 0.0 {
                    modulo += 4_294_967_296.0;
                }
                modulo as u32
            };
        if f64::from(length_uint32) != length_number {
            return Err(VmError::Abrupt(errors::throw_range_error(agent)));
        }

        let mut next_index = u64::from(length_uint32);

        for argument in arguments {
            let key = if next_index <= u64::from(u32::MAX - 1) {
                PropertyKey::Index(next_index as u32)
            } else {
                let atom = agent
                    .atoms_mut()
                    .intern_collectible(&next_index.to_string());
                PropertyKey::from_atom(atom)
            };
            let stored = object::set(agent, object, key, *argument, AllocationLifetime::Default)
                .map_err(VmError::Abrupt)?;
            if !stored {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            next_index = next_index.saturating_add(1);
        }

        if next_index > u64::from(u32::MAX) {
            return Err(VmError::Abrupt(errors::throw_range_error(agent)));
        }

        let new_length = next_index as u32;
        Self::define_length_property(agent, object, new_length, false)?;
        Ok(length_value(new_length))
    }

    fn array_pop_builtin(
        &mut self,
        agent: &mut Agent,
        caller: FrameRecord,
        this_value: Value,
    ) -> VmResult<Value> {
        let object = self.to_object_for_value(agent, caller.realm(), this_value)?;
        let length = object::get(
            agent,
            object,
            PropertyKey::from_atom(WellKnownAtom::length.id()),
        )
        .map_err(VmError::Abrupt)?;
        let length_number = to_f64_number(agent, length)?;
        let length_uint32 =
            if length_number.is_nan() || length_number == 0.0 || !length_number.is_finite() {
                0
            } else {
                let integer = length_number.trunc();
                let mut modulo = integer % 4_294_967_296.0;
                if modulo < 0.0 {
                    modulo += 4_294_967_296.0;
                }
                modulo as u32
            };
        if f64::from(length_uint32) != length_number {
            return Err(VmError::Abrupt(errors::throw_range_error(agent)));
        }
        if length_uint32 == 0 {
            Self::define_length_property(agent, object, 0, false)?;
            return Ok(Value::undefined());
        }

        let index = length_uint32 - 1;
        let element =
            object::get(agent, object, PropertyKey::Index(index)).map_err(VmError::Abrupt)?;
        let deleted = object::delete_property(agent, object, PropertyKey::Index(index))
            .map_err(VmError::Abrupt)?;
        if !deleted {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Self::define_length_property(agent, object, index, false)?;
        Ok(element)
    }

    fn object_to_string_builtin(
        &mut self,
        agent: &mut Agent,
        caller: FrameRecord,
        this_value: Value,
    ) -> VmResult<Value> {
        if this_value.is_undefined() {
            return Ok(Value::from_string_ref(alloc_string(
                agent,
                "[object Undefined]",
                None,
            )));
        }
        if this_value.is_null() {
            return Ok(Value::from_string_ref(alloc_string(
                agent,
                "[object Null]",
                None,
            )));
        }
        let object = self.to_object_for_value(agent, caller.realm(), this_value)?;
        let default_tag = if agent.objects().is_callable(object) {
            "Function"
        } else if agent
            .objects()
            .object_header(agent.heap().view(), object)
            .is_some_and(|header| header.flags().is_engine_array())
        {
            "Array"
        } else if let Some(kind) = agent.objects().primitive_wrapper_kind(object) {
            match kind {
                PrimitiveWrapperKind::String => "String",
                PrimitiveWrapperKind::Number => "Number",
                PrimitiveWrapperKind::Boolean => "Boolean",
                PrimitiveWrapperKind::Symbol => "Symbol",
                PrimitiveWrapperKind::BigInt => "BigInt",
            }
        } else {
            let intrinsics = agent
                .realm(caller.realm())
                .map(|record| record.intrinsics())
                .ok_or(VmError::Abrupt(errors::throw_type_error(agent)))?;
            let error_prototypes = [
                intrinsics.error_prototype(),
                intrinsics.eval_error_prototype(),
                intrinsics.range_error_prototype(),
                intrinsics.reference_error_prototype(),
                intrinsics.syntax_error_prototype(),
                intrinsics.type_error_prototype(),
                intrinsics.uri_error_prototype(),
            ];
            let mut current = Some(object);
            let mut is_error_object = false;
            while let Some(candidate) = current {
                if error_prototypes
                    .into_iter()
                    .flatten()
                    .any(|prototype| prototype == candidate)
                {
                    is_error_object = true;
                    break;
                }
                current = object::get_prototype_of(agent, candidate).map_err(VmError::Abrupt)?;
            }
            if is_error_object {
                "Error"
            } else {
                "Object"
            }
        };
        let to_string_tag = agent
            .well_known_symbol(WellKnownSymbolId::ToStringTag)
            .map(PropertyKey::from_symbol)
            .and_then(|key| object::get(agent, object, key).ok())
            .filter(|value| value.is_string())
            .map(|value| self.value_to_string_text(agent, value))
            .transpose()?;
        Ok(Value::from_string_ref(alloc_string(
            agent,
            &format!(
                "[object {}]",
                to_string_tag.as_deref().unwrap_or(default_tag)
            ),
            None,
        )))
    }

    fn template_to_string_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        value: Value,
    ) -> VmResult<Value> {
        if !value.is_object() {
            let text = self.value_to_string_text(agent, value)?;
            return Ok(Value::from_string_ref(alloc_string(agent, &text, None)));
        }

        let object = value
            .as_object_ref()
            .ok_or(VmError::Abrupt(errors::throw_type_error(agent)))?;
        let to_string = self.get_property_from_object(
            agent,
            host,
            registry,
            caller,
            object,
            Value::from_object_ref(object),
            PropertyKey::from_atom(WellKnownAtom::toString.id()),
        )?;
        if let Some(result) = self.call_if_callable_object(
            agent,
            host,
            registry,
            caller,
            to_string,
            Value::from_object_ref(object),
            &[],
        )? {
            if !result.is_object() {
                let text = self.value_to_string_text(agent, result)?;
                return Ok(Value::from_string_ref(alloc_string(agent, &text, None)));
            }
        }

        let value_of = self.get_property_from_object(
            agent,
            host,
            registry,
            caller,
            object,
            Value::from_object_ref(object),
            PropertyKey::from_atom(WellKnownAtom::valueOf.id()),
        )?;
        if let Some(result) = self.call_if_callable_object(
            agent,
            host,
            registry,
            caller,
            value_of,
            Value::from_object_ref(object),
            &[],
        )? {
            if !result.is_object() {
                let text = self.value_to_string_text(agent, result)?;
                return Ok(Value::from_string_ref(alloc_string(agent, &text, None)));
            }
        }

        Err(VmError::Abrupt(errors::throw_type_error(agent)))
    }

    fn get_template_object_builtin(
        &mut self,
        agent: &mut Agent,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let site = arguments
            .first()
            .copied()
            .and_then(Value::as_smi)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(VmError::Abrupt(errors::throw_type_error(agent)))?;
        let key = TemplateCacheKey {
            realm: caller.realm(),
            code: caller.code(),
            site,
        };
        if let Some(template) = self.template_cache.get(&key).copied() {
            return Ok(Value::from_object_ref(template));
        }

        let string_count = arguments.len().saturating_sub(1) / 2;
        let cooked = self.create_array(agent, caller.realm(), string_count)?;
        let raw = self.create_array(agent, caller.realm(), string_count)?;
        for index in 0..string_count {
            let cooked_value = arguments
                .get(1 + index * 2)
                .copied()
                .unwrap_or(Value::undefined());
            let raw_value = arguments
                .get(2 + index * 2)
                .copied()
                .unwrap_or(Value::undefined());
            agent.with_heap_and_objects(|heap, objects| {
                let mut mutator = heap.mutator();
                objects.set_element(
                    &mut mutator,
                    cooked,
                    u32::try_from(index).unwrap_or(u32::MAX),
                    cooked_value,
                    AllocationLifetime::Default,
                );
                objects.set_element(
                    &mut mutator,
                    raw,
                    u32::try_from(index).unwrap_or(u32::MAX),
                    raw_value,
                    AllocationLifetime::Default,
                );
            });
        }
        self.sync_engine_array_length(agent, cooked)?;
        self.sync_engine_array_length(agent, raw)?;

        let raw_name = agent.atoms_mut().intern_collectible("raw");
        self.define_data_property_with_attrs(
            agent,
            cooked,
            PropertyKey::from_atom(raw_name),
            Value::from_object_ref(raw),
            false,
            false,
            false,
        )?;
        self.template_cache.insert(key, cooked);
        Ok(Value::from_object_ref(cooked))
    }

    fn instance_of_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let value = arguments.first().copied().unwrap_or(Value::undefined());
        let constructor = arguments.get(1).copied().unwrap_or(Value::undefined());
        let Some(object) = value.as_object_ref() else {
            return Ok(Value::from_bool(false));
        };
        let constructor = Self::require_callable_object(agent, caller, constructor)?;
        let mut bridge = VmProxyBridge {
            vm: self,
            agent,
            host,
            registry,
            frame: caller,
        };
        let prototype = proxy::get(
            &mut bridge,
            constructor,
            PropertyKey::from_atom(WellKnownAtom::prototype.id()),
            Value::from_object_ref(constructor),
        )?
        .as_object_ref()
        .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(bridge.agent)))?;

        let mut current = proxy::get_prototype_of(&mut bridge, object)?;
        while let Some(candidate) = current {
            if candidate == prototype {
                return Ok(Value::from_bool(true));
            }
            current = proxy::get_prototype_of(&mut bridge, candidate)?;
        }

        Ok(Value::from_bool(false))
    }

    fn define_accessor_property_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        arguments: &[Value],
        is_getter: bool,
        enumerable: bool,
    ) -> VmResult<Value> {
        let object = arguments
            .first()
            .copied()
            .and_then(Value::as_object_ref)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let key_value = arguments.get(1).copied().unwrap_or(Value::undefined());
        let accessor = arguments.get(2).copied().unwrap_or(Value::undefined());
        if !accessor.is_undefined() && accessor.as_object_ref().is_none() {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }

        let key = self.to_property_key_from_value(agent, host, registry, caller, key_value)?;
        if let Some(accessor) = accessor.as_object_ref() {
            self.set_function_name_from_property_key(
                agent,
                accessor,
                key,
                Some(if is_getter { "get" } else { "set" }),
            )?;
        }
        let mut descriptor = PropertyDescriptor::new();
        if is_getter {
            descriptor.set_getter(accessor);
        } else {
            descriptor.set_setter(accessor);
        }
        descriptor.set_enumerable(enumerable);
        descriptor.set_configurable(true);
        let defined = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.define_own_property(
                &mut mutator,
                object,
                key,
                descriptor,
                AllocationLifetime::Default,
            )
        });
        let defined = defined.map_err(|_error| VmError::Abrupt(errors::throw_type_error(agent)))?;
        if !defined {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(Value::from_object_ref(object))
    }

    fn define_method_property_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let object = arguments
            .first()
            .copied()
            .and_then(Value::as_object_ref)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let key_value = arguments.get(1).copied().unwrap_or(Value::undefined());
        let value = arguments.get(2).copied().unwrap_or(Value::undefined());
        let key = self.to_property_key_from_value(agent, host, registry, caller, key_value)?;
        if let Some(function) = value.as_object_ref() {
            self.set_function_name_from_property_key(agent, function, key, None)?;
        }

        let mut descriptor = PropertyDescriptor::new();
        descriptor.set_value(value);
        descriptor.set_writable(true);
        descriptor.set_enumerable(false);
        descriptor.set_configurable(true);
        let defined = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.define_own_property(
                &mut mutator,
                object,
                key,
                descriptor,
                AllocationLifetime::Default,
            )
        });
        let defined = defined.map_err(|_error| VmError::Abrupt(errors::throw_type_error(agent)))?;
        if !defined {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(Value::from_object_ref(object))
    }

    fn set_function_home_object_builtin(
        &mut self,
        agent: &mut Agent,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let function = arguments
            .first()
            .copied()
            .and_then(Value::as_object_ref)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let home_object_value = arguments.get(1).copied().unwrap_or(Value::undefined());
        let home_object = if home_object_value.is_undefined() {
            None
        } else {
            Some(
                home_object_value
                    .as_object_ref()
                    .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?,
            )
        };
        let updated = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.set_function_home_object(&mut mutator, function, home_object)
        });
        if !updated {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(Value::from_object_ref(function))
    }

    fn capture_arrow_context_builtin(
        &mut self,
        agent: &mut Agent,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let function = arguments
            .first()
            .copied()
            .and_then(Value::as_object_ref)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let this_value = arguments.get(1).copied().unwrap_or(Value::undefined());
        let explicit_home_object = arguments.get(2).copied().unwrap_or(Value::undefined());
        let home_object = if explicit_home_object.is_undefined() {
            Some(Self::resolve_super_home_object(
                agent,
                caller.lexical_env(),
                caller,
            )?)
        } else {
            Some(
                explicit_home_object
                    .as_object_ref()
                    .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?,
            )
        };
        let outer = agent
            .objects()
            .function_data(function)
            .and_then(|data| data.environment())
            .or_else(|| {
                agent
                    .current_execution_context()
                    .map(|context| context.lexical_env())
            })
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let layout = agent.alloc_environment_layout(EnvironmentLayout::empty(
            EnvironmentLayoutKind::Function,
            true,
        ));
        let new_target = agent
            .current_execution_context()
            .and_then(|context| context.new_target());
        let env = agent
            .alloc_function_environment(
                Some(outer),
                layout,
                function,
                ThisBindingStatus::Initialized,
                this_value,
                new_target,
                home_object,
                AllocationLifetime::Default,
            )
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let updated = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.set_function_environment(&mut mutator, function, Some(env))
        });
        if !updated {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(Value::from_object_ref(function))
    }

    fn object_literal_set_prototype_builtin(
        &mut self,
        agent: &mut Agent,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let object = arguments
            .first()
            .copied()
            .and_then(Value::as_object_ref)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let prototype_value = arguments.get(1).copied().unwrap_or(Value::undefined());
        let prototype = if prototype_value.is_null() {
            None
        } else if let Some(prototype) = prototype_value.as_object_ref() {
            Some(prototype)
        } else {
            return Ok(Value::from_object_ref(object));
        };
        let changed =
            object::set_prototype_of(agent, object, prototype).map_err(VmError::Abrupt)?;
        if !changed {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(Value::from_object_ref(object))
    }

    fn bind_function_private_env_builtin(
        &mut self,
        agent: &mut Agent,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let function = arguments
            .first()
            .copied()
            .and_then(Value::as_object_ref)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let class_object = arguments
            .get(1)
            .copied()
            .and_then(Value::as_object_ref)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let prototype = arguments
            .get(2)
            .copied()
            .and_then(Value::as_object_ref)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let outer = agent
            .objects()
            .function_data(function)
            .and_then(|data| data.private_env())
            .or_else(|| {
                agent
                    .current_execution_context()
                    .and_then(|context| context.private_env())
            });
        let installs_private_names = arguments
            .get(3)
            .copied()
            .and_then(Value::as_bool)
            .unwrap_or(true);
        if !installs_private_names && outer.is_none() {
            return Ok(Value::from_object_ref(function));
        }
        let layout = self.class_private_environment_layout(agent);
        let private_env = agent
            .alloc_private_environment(outer, layout, AllocationLifetime::Default)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        if !agent.init_environment_slot(private_env, 0, Value::from_object_ref(class_object))
            || !agent.init_environment_slot(private_env, 1, Value::from_object_ref(prototype))
        {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        let updated = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.set_function_private_env(&mut mutator, function, Some(private_env))
        });
        if !updated {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(Value::from_object_ref(function))
    }

    fn install_instance_field_key_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let class_object = arguments
            .first()
            .copied()
            .and_then(Value::as_object_ref)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let field_index = arguments
            .get(1)
            .copied()
            .and_then(Value::as_smi)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let key_value = arguments.get(2).copied().unwrap_or(Value::undefined());
        let key = self.to_property_key_from_value(agent, host, registry, caller, key_value)?;
        let canonical_key = self.property_key_to_enumeration_value(agent, key);
        object::install_instance_public_field_key(agent, class_object, field_index, canonical_key)
            .map_err(VmError::Abrupt)
    }

    fn get_instance_field_key_builtin(
        &mut self,
        agent: &mut Agent,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let class_object = arguments
            .first()
            .copied()
            .and_then(Value::as_object_ref)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let field_index = arguments
            .get(1)
            .copied()
            .and_then(Value::as_smi)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        object::instance_public_field_key(agent, class_object, field_index).map_err(VmError::Abrupt)
    }

    fn private_context_class_key(
        &self,
        agent: &Agent,
        caller: FrameRecord,
        receiver: ObjectRef,
        descriptor_index: u32,
        class_depth: u32,
    ) -> ObjectRef {
        if let Some(class_key) =
            Self::private_context_from_private_env(agent, descriptor_index, class_depth)
        {
            return class_key;
        }

        let mut remaining = class_depth;
        let callee = caller.callee();
        if let Some(home_object) = callee.and_then(|callee| {
            agent
                .objects()
                .function_data(callee)
                .and_then(|data| data.home_object())
        }) {
            if remaining == 0 {
                return home_object;
            }
            remaining = remaining.saturating_sub(1);
        }

        let mut current = Some(caller.lexical_env());

        while let Some(environment) = current {
            let Some(record) = agent.environment(environment) else {
                break;
            };
            match record {
                lyng_js_env::EnvironmentRecord::Function(record) => {
                    if callee.is_some_and(|callee| record.function_object() == callee) {
                        current = record.declarative().outer();
                        continue;
                    }
                    if let Some(home_object) = record.home_object() {
                        if remaining == 0 {
                            return home_object;
                        }
                        remaining = remaining.saturating_sub(1);
                    }
                    current = record.declarative().outer();
                }
                lyng_js_env::EnvironmentRecord::Declarative(record) => current = record.outer(),
                lyng_js_env::EnvironmentRecord::Private(record) => current = record.outer(),
                lyng_js_env::EnvironmentRecord::Module(record) => current = record.outer(),
                lyng_js_env::EnvironmentRecord::Global(record) => current = record.outer(),
                lyng_js_env::EnvironmentRecord::Object(record) => current = record.outer(),
            }
        }

        receiver
    }

    fn private_context_from_private_env(
        agent: &Agent,
        descriptor_index: u32,
        class_depth: u32,
    ) -> Option<ObjectRef> {
        let mut current = agent
            .current_execution_context()
            .and_then(|context| context.private_env());
        let mut remaining = class_depth;

        while let Some(environment) = current {
            let record = agent.private_environment(environment)?;
            if remaining == 0 {
                let class_object = agent.environment_slot(environment, 0)?.as_object_ref()?;
                let prototype = agent.environment_slot(environment, 1)?.as_object_ref()?;
                let is_static = agent
                    .objects()
                    .private_descriptor_is_static(class_object, descriptor_index)?;
                return Some(if is_static { class_object } else { prototype });
            }
            remaining = remaining.saturating_sub(1);
            current = record.outer();
        }

        None
    }

    fn define_private_field_builtin(
        &mut self,
        agent: &mut Agent,
        _caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let class_object = arguments
            .first()
            .copied()
            .and_then(Value::as_object_ref)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let prototype = arguments
            .get(1)
            .copied()
            .and_then(Value::as_object_ref)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let name_value = arguments.get(2).copied().unwrap_or(Value::undefined());
        let name_text = self.value_to_string_text(agent, name_value)?;
        let name = agent.atoms_mut().intern_collectible(&name_text);
        let is_static = arguments
            .get(3)
            .copied()
            .and_then(Value::as_bool)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let kind = Self::private_element_kind_from_argument(agent, arguments.get(4).copied())?;
        let descriptor = object::define_private_element_layout(
            agent,
            class_object,
            prototype,
            name,
            is_static,
            kind,
        )
        .map_err(VmError::Abrupt)?;
        if kind != ClassPrivateElementKind::Field {
            let value = arguments.get(5).copied().unwrap_or(Value::undefined());
            let class_key = if is_static { class_object } else { prototype };
            object::install_private_element_value(agent, class_key, descriptor, value)
                .map_err(VmError::Abrupt)?;
        }
        Ok(Value::from_smi(
            i32::try_from(descriptor).unwrap_or(i32::MAX),
        ))
    }

    fn private_field_init_builtin(
        &mut self,
        agent: &mut Agent,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let receiver = arguments
            .first()
            .copied()
            .and_then(Value::as_object_ref)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let descriptor_index = arguments
            .get(1)
            .copied()
            .and_then(Value::as_smi)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let value = arguments.get(2).copied().unwrap_or(Value::undefined());
        let class_depth = arguments
            .get(3)
            .copied()
            .and_then(Value::as_smi)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(0);
        let class_key =
            self.private_context_class_key(agent, caller, receiver, descriptor_index, class_depth);
        object::private_field_init(agent, receiver, class_key, descriptor_index, value)
            .map_err(VmError::Abrupt)
    }

    fn private_field_get_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let receiver = arguments
            .first()
            .copied()
            .and_then(Value::as_object_ref)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let descriptor_index = arguments
            .get(1)
            .copied()
            .and_then(Value::as_smi)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let class_depth = arguments
            .get(2)
            .copied()
            .and_then(Value::as_smi)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(0);
        let class_key =
            self.private_context_class_key(agent, caller, receiver, descriptor_index, class_depth);
        let kind = object::private_element_kind(agent, class_key, descriptor_index)
            .map_err(VmError::Abrupt)?;
        match kind {
            ClassPrivateElementKind::Field => {
                object::private_field_get(agent, receiver, class_key, descriptor_index)
                    .map_err(VmError::Abrupt)
            }
            ClassPrivateElementKind::Method => {
                let has_brand = object::private_has(agent, receiver, class_key, descriptor_index)
                    .map_err(VmError::Abrupt)?;
                if !has_brand {
                    return Err(VmError::Abrupt(errors::throw_type_error(agent)));
                }
                object::private_shared_element_value(agent, class_key, descriptor_index)
                    .map_err(VmError::Abrupt)
            }
            ClassPrivateElementKind::Getter => {
                let has_brand = object::private_has(agent, receiver, class_key, descriptor_index)
                    .map_err(VmError::Abrupt)?;
                if !has_brand {
                    return Err(VmError::Abrupt(errors::throw_type_error(agent)));
                }
                let getter =
                    object::private_shared_element_value(agent, class_key, descriptor_index)
                        .map_err(VmError::Abrupt)?;
                self.call_optional_callback(
                    agent,
                    host,
                    registry,
                    caller,
                    getter,
                    Value::from_object_ref(receiver),
                    &[],
                )?
                .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))
            }
            ClassPrivateElementKind::Setter => {
                Err(VmError::Abrupt(errors::throw_type_error(agent)))
            }
        }
    }

    fn private_field_set_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let receiver = arguments
            .first()
            .copied()
            .and_then(Value::as_object_ref)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let descriptor_index = arguments
            .get(1)
            .copied()
            .and_then(Value::as_smi)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let value = arguments.get(2).copied().unwrap_or(Value::undefined());
        let class_depth = arguments
            .get(3)
            .copied()
            .and_then(Value::as_smi)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(0);
        let class_key =
            self.private_context_class_key(agent, caller, receiver, descriptor_index, class_depth);
        let kind = object::private_element_kind(agent, class_key, descriptor_index)
            .map_err(VmError::Abrupt)?;
        match kind {
            ClassPrivateElementKind::Field => {
                object::private_field_set(agent, receiver, class_key, descriptor_index, value)
                    .map_err(VmError::Abrupt)
            }
            ClassPrivateElementKind::Setter => {
                let has_brand = object::private_has(agent, receiver, class_key, descriptor_index)
                    .map_err(VmError::Abrupt)?;
                if !has_brand {
                    return Err(VmError::Abrupt(errors::throw_type_error(agent)));
                }
                let setter =
                    object::private_shared_element_value(agent, class_key, descriptor_index)
                        .map_err(VmError::Abrupt)?;
                let arguments = [value];
                let _ = self.call_optional_callback(
                    agent,
                    host,
                    registry,
                    caller,
                    setter,
                    Value::from_object_ref(receiver),
                    &arguments,
                )?;
                Ok(value)
            }
            ClassPrivateElementKind::Method | ClassPrivateElementKind::Getter => {
                Err(VmError::Abrupt(errors::throw_type_error(agent)))
            }
        }
    }

    fn private_has_builtin(
        &mut self,
        agent: &mut Agent,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let receiver = arguments
            .first()
            .copied()
            .and_then(Value::as_object_ref)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let descriptor_index = arguments
            .get(1)
            .copied()
            .and_then(Value::as_smi)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let class_depth = arguments
            .get(2)
            .copied()
            .and_then(Value::as_smi)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(0);
        let class_key =
            self.private_context_class_key(agent, caller, receiver, descriptor_index, class_depth);
        let has = object::private_has(agent, receiver, class_key, descriptor_index)
            .map_err(VmError::Abrupt)?;
        Ok(Value::from_bool(has))
    }

    fn private_element_kind_from_argument(
        agent: &mut Agent,
        argument: Option<Value>,
    ) -> VmResult<ClassPrivateElementKind> {
        match argument.and_then(Value::as_smi).unwrap_or(0) {
            0 => Ok(ClassPrivateElementKind::Field),
            1 => Ok(ClassPrivateElementKind::Method),
            2 => Ok(ClassPrivateElementKind::Getter),
            3 => Ok(ClassPrivateElementKind::Setter),
            _ => Err(VmError::Abrupt(errors::throw_type_error(agent))),
        }
    }

    fn super_property_get_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let receiver = arguments.first().copied().unwrap_or(Value::undefined());
        let home_object = arguments
            .get(2)
            .and_then(|value| value.as_object_ref())
            .map(Ok)
            .unwrap_or_else(|| {
                Self::resolve_super_home_object(agent, caller.lexical_env(), caller)
            })?;
        let base = object::super_base(agent, home_object).map_err(VmError::Abrupt)?;
        let key_value = arguments.get(1).copied().unwrap_or(Value::undefined());
        let key = self.to_property_key_from_value(agent, host, registry, caller, key_value)?;
        self.get_property_from_object(agent, host, registry, caller, base, receiver, key)
    }

    fn super_property_set_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let receiver = arguments.first().copied().unwrap_or(Value::undefined());
        let value = arguments.get(2).copied().unwrap_or(Value::undefined());
        let home_object = arguments
            .get(3)
            .and_then(|value| value.as_object_ref())
            .map(Ok)
            .unwrap_or_else(|| {
                Self::resolve_super_home_object(agent, caller.lexical_env(), caller)
            })?;
        let base = object::super_base(agent, home_object).map_err(VmError::Abrupt)?;
        let key_value = arguments.get(1).copied().unwrap_or(Value::undefined());
        let key = self.to_property_key_from_value(agent, host, registry, caller, key_value)?;
        let updated =
            self.set_property_on_object(agent, host, registry, caller, base, receiver, key, value)?;
        if !updated && self.caller_is_strict(caller) {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(value)
    }

    fn construct_super_with_arguments(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let record = Self::this_environment_record(agent, caller.lexical_env())?;
        let function_env = record.map(|record| record.declarative().id());
        let active_function = record
            .map(|record| record.function_object())
            .or_else(|| caller.callee())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let derived_constructor = {
            agent
                .objects()
                .function_data(active_function)
                .and_then(|data| match data.entry() {
                    Some(FunctionEntryIdentity::Bytecode(code)) => Some(code),
                    _ => None,
                })
                .and_then(|code| self.installed_function(code))
                .map(|function| function.flags().derived_class_constructor())
                .unwrap_or(false)
        } || caller
            .flags()
            .contains(super::FrameFlags::derived_construct());
        if !derived_constructor {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        let binding_status = record.map_or_else(
            || {
                if caller.construct_this().is_some()
                    || agent
                        .current_execution_context()
                        .is_some_and(|context| context.this_state() != ThisState::Uninitialized)
                {
                    lyng_js_env::ThisBindingStatus::Initialized
                } else {
                    lyng_js_env::ThisBindingStatus::Uninitialized
                }
            },
            |record| record.this_binding_status(),
        );
        let super_constructor = object::get_prototype_of(agent, active_function)
            .map_err(VmError::Abrupt)?
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let new_target = record
            .and_then(|record| record.new_target())
            .or_else(|| caller.new_target())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let this_object = self.construct_to_completion(
            agent,
            host,
            registry,
            caller,
            super_constructor,
            arguments,
            Some(new_target),
        )?;
        let this_value = Value::from_object_ref(this_object);
        if binding_status != lyng_js_env::ThisBindingStatus::Uninitialized {
            return Err(VmError::Abrupt(errors::throw_reference_error(agent)));
        }
        if let Some(function_env) = function_env {
            let _ = agent.set_function_this_binding(
                function_env,
                lyng_js_env::ThisBindingStatus::Initialized,
                this_value,
            );
            if !agent.set_execution_context_this_state_for_lexical_env(
                function_env,
                ThisState::Value(this_value),
            ) {
                let _ =
                    agent.set_current_execution_context_this_state(ThisState::Value(this_value));
            }
        } else {
            let _ = agent.set_current_execution_context_this_state(ThisState::Value(this_value));
        }
        let frame_index = self
            .frames
            .iter()
            .rposition(|frame| frame.callee() == Some(active_function))
            .or_else(|| {
                function_env.and_then(|function_env| {
                    self.frames.iter().rposition(|frame| {
                        frame.lexical_env() == function_env || frame.variable_env() == function_env
                    })
                })
            })
            .or_else(|| {
                self.frames.iter().rposition(|frame| {
                    frame.code() == caller.code()
                        && frame.registers() == caller.registers()
                        && frame.callee() == caller.callee()
                })
            })
            .or_else(|| self.frames.len().checked_sub(1));
        let Some(frame_index) = frame_index else {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        };
        let Some(frame) = self.frames.get_mut(frame_index) else {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        };
        frame.set_this_value(this_value);
        frame.set_construct_this(Some(this_object));
        Ok(this_value)
    }

    fn construct_super_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        self.construct_super_with_arguments(agent, host, registry, caller, arguments)
    }

    fn construct_super_spread_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let spread_source = arguments.first().copied().unwrap_or(Value::undefined());
        let mut spread_arguments = Vec::new();
        self.append_spread_argument(
            agent,
            host,
            registry,
            caller,
            spread_source,
            &mut spread_arguments,
        )?;
        self.construct_super_with_arguments(agent, host, registry, caller, &spread_arguments)
    }

    fn string_replace_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        this_value: Value,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let source = self.value_to_string_text(agent, this_value)?;
        let search = arguments.first().copied().unwrap_or(Value::undefined());
        let search_text = self.value_to_string_text(agent, search)?;
        let Some(index) = source.find(&search_text) else {
            return Ok(Value::from_string_ref(alloc_string(agent, &source, None)));
        };
        let replacement = arguments.get(1).copied().unwrap_or(Value::undefined());
        let search_value = Value::from_string_ref(alloc_string(agent, &search_text, None));
        let source_value = Value::from_string_ref(alloc_string(agent, &source, None));
        let replacement_text = if let Some(replaced) = self.call_if_callable_object(
            agent,
            host,
            registry,
            caller,
            replacement,
            Value::undefined(),
            &[
                search_value,
                Value::from_smi(i32::try_from(index).unwrap_or(i32::MAX)),
                source_value,
            ],
        )? {
            self.value_to_string_text(agent, replaced)?
        } else {
            self.value_to_string_text(agent, replacement)?
        };

        let mut result = String::with_capacity(source.len() + replacement_text.len());
        result.push_str(&source[..index]);
        result.push_str(&replacement_text);
        result.push_str(&source[index + search_text.len()..]);
        Ok(Value::from_string_ref(alloc_string(agent, &result, None)))
    }
}

struct VmBuiltinDispatch<'a, 'agent, 'registry> {
    vm: &'a mut Vm,
    agent: &'agent mut Agent,
    host: &'a dyn HostHooks,
    registry: &'registry mut dyn NativeFunctionRegistry,
    caller_frame: FrameRecord,
    callee_object: ObjectRef,
}

impl object::ToPrimitiveContext for VmBuiltinDispatch<'_, '_, '_> {
    type Error = VmError;

    fn agent(&mut self) -> &mut Agent {
        self.agent
    }

    fn abrupt(&mut self, completion: AbruptCompletion) -> Self::Error {
        VmError::Abrupt(completion)
    }

    fn type_error(&mut self) -> Self::Error {
        VmError::Abrupt(errors::throw_type_error(self.agent))
    }

    fn get_property_value(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
    ) -> Result<Value, Self::Error> {
        self.builtin_get_property_value_from_object(object, key)
    }

    fn require_callable_object(&mut self, value: Value) -> Result<ObjectRef, Self::Error> {
        Vm::require_callable_object(self.agent, self.caller_frame, value)
    }

    fn call_to_completion(
        &mut self,
        callee_object: ObjectRef,
        this_value: Value,
        arguments: &[Value],
    ) -> Result<Value, Self::Error> {
        self.vm.call_to_completion(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            callee_object,
            this_value,
            arguments,
        )
    }

    fn default_to_primitive_result(
        &mut self,
        object: ObjectRef,
        method_name: lyng_js_common::AtomId,
        method_object: ObjectRef,
    ) -> Result<Option<Value>, Self::Error> {
        let Some(entry) = Vm::builtin_entry(self.agent, method_object) else {
            return Ok(None);
        };
        if method_name != WellKnownAtom::toString.id() || entry != js3_object_to_string_builtin() {
            return Ok(None);
        }

        let is_engine_array = self
            .agent
            .objects()
            .object_header(self.agent.heap().view(), object)
            .is_some_and(|header| header.flags().is_engine_array());
        if !is_engine_array {
            return Ok(None);
        }

        let text = self.engine_array_to_string_fallback(object)?;
        Ok(Some(Value::from_string_ref(alloc_string(
            self.agent, &text, None,
        ))))
    }
}

impl VmBuiltinDispatch<'_, '_, '_> {
    fn builtin_get_property_value(&mut self, receiver: Value, key: PropertyKey) -> VmResult<Value> {
        self.vm.get_property_from_value(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            receiver,
            key,
        )
    }

    fn builtin_get_property_value_from_object(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
    ) -> VmResult<Value> {
        self.builtin_get_property_value(Value::from_object_ref(object), key)
    }

    fn builtin_constructor_prototype(
        &mut self,
        source_realm: RealmRef,
        default_prototype: ObjectRef,
        new_target: Option<ObjectRef>,
    ) -> VmResult<ObjectRef> {
        let Some(new_target) = new_target else {
            return Ok(default_prototype);
        };
        let prototype = self.builtin_get_property_value_from_object(
            new_target,
            PropertyKey::from_atom(WellKnownAtom::prototype.id()),
        )?;
        if let Some(prototype) = prototype.as_object_ref() {
            return Ok(prototype);
        }
        let function_realm = Vm::function_realm(self.agent, new_target)?;
        if function_realm == source_realm {
            return Ok(default_prototype);
        }
        Ok(self
            .remap_constructor_default_prototype(source_realm, function_realm, default_prototype)
            .unwrap_or(default_prototype))
    }

    fn remap_constructor_default_prototype(
        &mut self,
        source_realm: RealmRef,
        target_realm: RealmRef,
        default_prototype: ObjectRef,
    ) -> Option<ObjectRef> {
        let source_intrinsics = self.agent.realm(source_realm)?.intrinsics();
        let target_intrinsics = self.agent.realm(target_realm)?.intrinsics();

        macro_rules! remap_intrinsic_prototype {
            ($getter:ident) => {
                if source_intrinsics.$getter() == Some(default_prototype) {
                    return target_intrinsics.$getter();
                }
            };
        }

        remap_intrinsic_prototype!(object_prototype);
        remap_intrinsic_prototype!(function_prototype);
        remap_intrinsic_prototype!(async_function_prototype);
        remap_intrinsic_prototype!(async_generator_function_prototype);
        remap_intrinsic_prototype!(async_generator_prototype);
        remap_intrinsic_prototype!(generator_function_prototype);
        remap_intrinsic_prototype!(generator_prototype);
        remap_intrinsic_prototype!(array_prototype);
        remap_intrinsic_prototype!(map_prototype);
        remap_intrinsic_prototype!(map_iterator_prototype);
        remap_intrinsic_prototype!(set_prototype);
        remap_intrinsic_prototype!(set_iterator_prototype);
        remap_intrinsic_prototype!(weak_map_prototype);
        remap_intrinsic_prototype!(weak_set_prototype);
        remap_intrinsic_prototype!(weak_ref_prototype);
        remap_intrinsic_prototype!(finalization_registry_prototype);
        remap_intrinsic_prototype!(array_buffer_prototype);
        remap_intrinsic_prototype!(shared_array_buffer_prototype);
        remap_intrinsic_prototype!(data_view_prototype);
        remap_intrinsic_prototype!(typed_array_prototype);
        remap_intrinsic_prototype!(int8_array_prototype);
        remap_intrinsic_prototype!(int16_array_prototype);
        remap_intrinsic_prototype!(int32_array_prototype);
        remap_intrinsic_prototype!(float32_array_prototype);
        remap_intrinsic_prototype!(float64_array_prototype);
        remap_intrinsic_prototype!(big_int64_array_prototype);
        remap_intrinsic_prototype!(big_uint64_array_prototype);
        remap_intrinsic_prototype!(uint32_array_prototype);
        remap_intrinsic_prototype!(uint16_array_prototype);
        remap_intrinsic_prototype!(uint8_clamped_array_prototype);
        remap_intrinsic_prototype!(uint8_array_prototype);
        remap_intrinsic_prototype!(iterator_prototype);
        remap_intrinsic_prototype!(async_iterator_prototype);
        remap_intrinsic_prototype!(async_from_sync_iterator_prototype);
        remap_intrinsic_prototype!(array_iterator_prototype);
        remap_intrinsic_prototype!(string_prototype);
        remap_intrinsic_prototype!(string_iterator_prototype);
        remap_intrinsic_prototype!(regexp_prototype);
        remap_intrinsic_prototype!(date_prototype);
        remap_intrinsic_prototype!(number_prototype);
        remap_intrinsic_prototype!(bigint_prototype);
        remap_intrinsic_prototype!(boolean_prototype);
        remap_intrinsic_prototype!(symbol_prototype);
        remap_intrinsic_prototype!(error_prototype);
        remap_intrinsic_prototype!(eval_error_prototype);
        remap_intrinsic_prototype!(range_error_prototype);
        remap_intrinsic_prototype!(reference_error_prototype);
        remap_intrinsic_prototype!(syntax_error_prototype);
        remap_intrinsic_prototype!(type_error_prototype);
        remap_intrinsic_prototype!(uri_error_prototype);
        remap_intrinsic_prototype!(aggregate_error_prototype);
        remap_intrinsic_prototype!(suppressed_error_prototype);
        remap_intrinsic_prototype!(promise_prototype);
        remap_intrinsic_prototype!(disposable_stack_prototype);
        remap_intrinsic_prototype!(async_disposable_stack_prototype);

        None
    }

    fn map_temporal_host_result<T>(
        &mut self,
        result: Result<T, lyng_js_host::HostError>,
    ) -> Result<T, VmError> {
        result.map_err(|error| match error.kind() {
            HostErrorKind::InvalidRequest => VmError::Abrupt(errors::throw_range_error(self.agent)),
            _ => VmError::Host(error),
        })
    }

    pub(super) fn builtin_to_property_descriptor(
        &mut self,
        descriptor_object: ObjectRef,
    ) -> VmResult<PropertyDescriptor> {
        let mut descriptor = PropertyDescriptor::new();

        if object::has_property(
            self.agent,
            descriptor_object,
            PropertyKey::from_atom(WellKnownAtom::enumerable.id()),
        )
        .map_err(VmError::Abrupt)?
        {
            let enumerable = self.builtin_get_property_value_from_object(
                descriptor_object,
                PropertyKey::from_atom(WellKnownAtom::enumerable.id()),
            )?;
            descriptor.set_enumerable(
                read::to_boolean(self.agent.heap().view(), enumerable).map_err(VmError::Abrupt)?,
            );
        }
        if object::has_property(
            self.agent,
            descriptor_object,
            PropertyKey::from_atom(WellKnownAtom::configurable.id()),
        )
        .map_err(VmError::Abrupt)?
        {
            let configurable = self.builtin_get_property_value_from_object(
                descriptor_object,
                PropertyKey::from_atom(WellKnownAtom::configurable.id()),
            )?;
            descriptor.set_configurable(
                read::to_boolean(self.agent.heap().view(), configurable)
                    .map_err(VmError::Abrupt)?,
            );
        }
        if object::has_property(
            self.agent,
            descriptor_object,
            PropertyKey::from_atom(WellKnownAtom::value.id()),
        )
        .map_err(VmError::Abrupt)?
        {
            let value = self.builtin_get_property_value_from_object(
                descriptor_object,
                PropertyKey::from_atom(WellKnownAtom::value.id()),
            )?;
            descriptor.set_value(value);
        }
        if object::has_property(
            self.agent,
            descriptor_object,
            PropertyKey::from_atom(WellKnownAtom::writable.id()),
        )
        .map_err(VmError::Abrupt)?
        {
            let writable = self.builtin_get_property_value_from_object(
                descriptor_object,
                PropertyKey::from_atom(WellKnownAtom::writable.id()),
            )?;
            descriptor.set_writable(
                read::to_boolean(self.agent.heap().view(), writable).map_err(VmError::Abrupt)?,
            );
        }
        if object::has_property(
            self.agent,
            descriptor_object,
            PropertyKey::from_atom(WellKnownAtom::get.id()),
        )
        .map_err(VmError::Abrupt)?
        {
            let getter = self.builtin_get_property_value_from_object(
                descriptor_object,
                PropertyKey::from_atom(WellKnownAtom::get.id()),
            )?;
            if !(getter.is_undefined()
                || getter
                    .as_object_ref()
                    .and_then(|object| self.agent.objects().function_data(object))
                    .is_some())
            {
                return Err(VmError::Abrupt(errors::throw_type_error(self.agent)));
            }
            descriptor.set_getter(getter);
        }
        if object::has_property(
            self.agent,
            descriptor_object,
            PropertyKey::from_atom(WellKnownAtom::set.id()),
        )
        .map_err(VmError::Abrupt)?
        {
            let setter = self.builtin_get_property_value_from_object(
                descriptor_object,
                PropertyKey::from_atom(WellKnownAtom::set.id()),
            )?;
            if !(setter.is_undefined()
                || setter
                    .as_object_ref()
                    .and_then(|object| self.agent.objects().function_data(object))
                    .is_some())
            {
                return Err(VmError::Abrupt(errors::throw_type_error(self.agent)));
            }
            descriptor.set_setter(setter);
        }

        if (descriptor.has_get() || descriptor.has_set())
            && (descriptor.has_value() || descriptor.has_writable())
        {
            return Err(VmError::Abrupt(errors::throw_type_error(self.agent)));
        }

        Ok(descriptor)
    }

    fn builtin_value_to_string_text(&mut self, value: Value) -> VmResult<String> {
        let primitive = object::to_primitive(self, value, object::ToPrimitiveHint::String)?;
        self.vm.value_to_string_text(self.agent, primitive)
    }

    fn builtin_to_property_key(&mut self, value: Value) -> VmResult<PropertyKey> {
        if let Some(symbol) = value.as_symbol_ref() {
            return Ok(PropertyKey::from_symbol(symbol));
        }
        let primitive = object::to_primitive(self, value, object::ToPrimitiveHint::String)?;
        self.vm.value_to_property_key(
            self.agent,
            self.caller_frame,
            self.caller_frame.code(),
            self.caller_frame.instruction_offset(),
            primitive,
        )
    }

    fn engine_array_to_string_fallback(&mut self, object: ObjectRef) -> VmResult<String> {
        let length = self.builtin_get_property_value_from_object(
            object,
            PropertyKey::from_atom(WellKnownAtom::length.id()),
        )?;
        let length = to_f64_number(self.agent, length)?.max(0.0) as u32;
        let mut text = String::new();
        for index in 0..length {
            if index != 0 {
                text.push(',');
            }
            let element =
                self.builtin_get_property_value_from_object(object, PropertyKey::Index(index))?;
            if element.is_undefined() || element.is_null() {
                continue;
            }
            text.push_str(&self.builtin_value_to_string_text(element)?);
        }
        Ok(text)
    }
}

impl InternalBuiltinDispatchContext for VmBuiltinDispatch<'_, '_, '_> {
    type Error = VmError;

    fn function_call_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let target =
            Vm::require_callable_object(self.agent, self.caller_frame, invocation.this_value())?;
        let rebound_this = invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined());
        self.vm.call_to_completion(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            target,
            rebound_this,
            invocation.arguments().get(1..).unwrap_or(&[]),
        )
    }

    fn direct_eval_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let callee = invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined());
        let target = Vm::require_callable_object(self.agent, self.caller_frame, callee)?;
        let builtin_eval = self
            .vm
            .builtin_cache
            .builtin_constant(self.agent, self.caller_frame.realm(), js3_eval_builtin())
            .and_then(Value::as_object_ref)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(self.agent)))?;
        if target != builtin_eval {
            return self.vm.call_to_completion(
                self.agent,
                self.host,
                self.registry,
                self.caller_frame,
                target,
                Value::undefined(),
                invocation.arguments().get(1..).unwrap_or(&[]),
            );
        }

        let source = invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined());
        let Some(source_ref) = source.as_string_ref() else {
            return Ok(source);
        };
        if let Some(value) = self.vm.try_evaluate_regexp_literal_eval_string_ref(
            self.agent,
            self.caller_frame.realm(),
            source_ref,
        )? {
            return Ok(value);
        }
        let source_text = self.builtin_value_to_string_text(Value::from_string_ref(source_ref))?;
        self.vm.evaluate_direct_eval_source(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            &source_text,
        )
    }

    fn string_replace_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm.string_replace_builtin(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            invocation.this_value(),
            invocation.arguments(),
        )
    }

    fn string_index_of_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm
            .string_index_of_builtin(self.agent, invocation.this_value(), invocation.arguments())
    }

    fn array_index_of_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm
            .array_index_of_builtin(self.agent, invocation.this_value(), invocation.arguments())
    }

    fn array_push_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm.array_push_builtin(
            self.agent,
            self.caller_frame,
            invocation.this_value(),
            invocation.arguments(),
        )
    }

    fn array_pop_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm
            .array_pop_builtin(self.agent, self.caller_frame, invocation.this_value())
    }

    fn object_to_string_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm
            .object_to_string_builtin(self.agent, self.caller_frame, invocation.this_value())
    }

    fn template_to_string_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let value = invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined());
        self.vm.template_to_string_builtin(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            value,
        )
    }

    fn get_template_object_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm
            .get_template_object_builtin(self.agent, self.caller_frame, invocation.arguments())
    }

    fn instance_of_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm.instance_of_builtin(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            invocation.arguments(),
        )
    }

    fn define_method_property_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm.define_method_property_builtin(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            invocation.arguments(),
        )
    }

    fn define_getter_property_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm.define_accessor_property_builtin(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            invocation.arguments(),
            true,
            true,
        )
    }

    fn define_setter_property_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm.define_accessor_property_builtin(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            invocation.arguments(),
            false,
            true,
        )
    }

    fn define_class_getter_property_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm.define_accessor_property_builtin(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            invocation.arguments(),
            true,
            false,
        )
    }

    fn define_class_setter_property_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm.define_accessor_property_builtin(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            invocation.arguments(),
            false,
            false,
        )
    }

    fn define_private_field_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm
            .define_private_field_builtin(self.agent, self.caller_frame, invocation.arguments())
    }

    fn private_field_init_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm
            .private_field_init_builtin(self.agent, self.caller_frame, invocation.arguments())
    }

    fn private_field_get_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm.private_field_get_builtin(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            invocation.arguments(),
        )
    }

    fn private_field_set_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm.private_field_set_builtin(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            invocation.arguments(),
        )
    }

    fn private_has_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm
            .private_has_builtin(self.agent, self.caller_frame, invocation.arguments())
    }

    fn super_property_get_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm.super_property_get_builtin(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            invocation.arguments(),
        )
    }

    fn super_property_set_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm.super_property_set_builtin(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            invocation.arguments(),
        )
    }

    fn construct_super_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm.construct_super_builtin(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            invocation.arguments(),
        )
    }

    fn construct_super_spread_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm.construct_super_spread_builtin(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            invocation.arguments(),
        )
    }

    fn object_has_own_property_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm.object_has_own_property_builtin(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            invocation.this_value(),
            invocation.arguments(),
        )
    }

    fn set_function_home_object_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm
            .set_function_home_object_builtin(self.agent, invocation.arguments())
    }

    fn object_literal_set_prototype_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm
            .object_literal_set_prototype_builtin(self.agent, invocation.arguments())
    }

    fn bind_function_private_env_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm
            .bind_function_private_env_builtin(self.agent, invocation.arguments())
    }

    fn capture_arrow_context_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm
            .capture_arrow_context_builtin(self.agent, self.caller_frame, invocation.arguments())
    }

    fn install_instance_field_key_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm.install_instance_field_key_builtin(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            invocation.arguments(),
        )
    }

    fn get_instance_field_key_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm
            .get_instance_field_key_builtin(self.agent, invocation.arguments())
    }

    fn throw_type_error_builtin(
        &mut self,
        _invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let realm = Vm::builtin_realm(self.agent, self.callee_object, self.caller_frame);
        Err(Vm::abrupt_intrinsic_error(
            self.agent,
            realm,
            errors::ErrorKind::Type,
        ))
    }
}

impl PublicBuiltinDispatchContext for VmBuiltinDispatch<'_, '_, '_> {
    fn agent(&mut self) -> &mut Agent {
        self.agent
    }

    fn callee_object(&self) -> ObjectRef {
        self.callee_object
    }

    fn builtin_realm(&self) -> RealmRef {
        Vm::builtin_realm(self.agent, self.callee_object, self.caller_frame)
    }

    fn caller_realm(&self) -> RealmRef {
        self.caller_frame.realm()
    }

    fn caller_is_strict(&self) -> bool {
        self.vm.caller_is_strict(self.caller_frame)
    }

    fn abrupt(&mut self, completion: AbruptCompletion) -> Self::Error {
        VmError::Abrupt(completion)
    }

    fn extract_thrown_value(&mut self, error: Self::Error) -> Result<Option<Value>, Self::Error> {
        match error {
            VmError::Abrupt(completion) => Ok(completion.thrown_value()),
            other => Err(other),
        }
    }

    fn value_to_string_text(&mut self, value: Value) -> Result<String, Self::Error> {
        self.builtin_value_to_string_text(value)
    }

    fn to_property_key(&mut self, value: Value) -> Result<PropertyKey, Self::Error> {
        self.builtin_to_property_key(value)
    }

    fn get_property_value(
        &mut self,
        receiver: Value,
        key: PropertyKey,
    ) -> Result<Value, Self::Error> {
        self.builtin_get_property_value(receiver, key)
    }

    fn get_property_from_object_with_receiver(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
        receiver: Value,
    ) -> Result<Value, Self::Error> {
        self.vm.get_property_from_object(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            object,
            receiver,
            key,
        )
    }

    fn get_own_property_from_object(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
    ) -> Result<Option<PropertyDescriptor>, Self::Error> {
        self.vm
            .get_own_property_from_object(self.agent, object, key)
    }

    fn set_property_on_object_with_receiver(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
        value: Value,
        receiver: Value,
    ) -> Result<bool, Self::Error> {
        self.vm.set_property_on_object(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            object,
            receiver,
            key,
            value,
        )
    }

    fn define_property_on_object(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
        descriptor: PropertyDescriptor,
        lifetime: AllocationLifetime,
    ) -> Result<bool, Self::Error> {
        self.vm
            .define_property_on_object(self.agent, object, key, descriptor, lifetime)
    }

    fn delete_property_from_object(
        &mut self,
        object: ObjectRef,
        key: PropertyKey,
    ) -> Result<bool, Self::Error> {
        self.vm.delete_property_from_object(self.agent, object, key)
    }

    fn to_object_for_builtin_value(
        &mut self,
        realm: RealmRef,
        value: Value,
    ) -> Result<ObjectRef, Self::Error> {
        self.vm.to_object_for_value(self.agent, realm, value)
    }

    fn allocate_ordinary_object_with_prototype(
        &mut self,
        realm: RealmRef,
        prototype: Option<ObjectRef>,
    ) -> Result<ObjectRef, Self::Error> {
        self.vm
            .allocate_ordinary_object_with_prototype(self.agent, realm, prototype)
    }

    fn allocate_builtin_function(
        &mut self,
        entry: BuiltinFunctionId,
    ) -> Result<ObjectRef, Self::Error> {
        self.vm
            .allocate_builtin_function_object(self.agent, self.builtin_realm(), entry)
    }

    fn create_array_object(
        &mut self,
        realm: RealmRef,
        element_capacity: usize,
    ) -> Result<ObjectRef, Self::Error> {
        self.vm.create_array(self.agent, realm, element_capacity)
    }

    fn ordinary_constructor_prototype(
        &mut self,
        realm: RealmRef,
        new_target: Option<ObjectRef>,
        default_prototype: ObjectRef,
    ) -> Result<ObjectRef, Self::Error> {
        self.builtin_constructor_prototype(realm, default_prototype, new_target)
    }

    fn descriptor_object_from_descriptor(
        &mut self,
        realm: RealmRef,
        descriptor: PropertyDescriptor,
    ) -> Result<Value, Self::Error> {
        self.vm
            .descriptor_object_from_descriptor(self.agent, realm, descriptor)
    }

    fn to_property_descriptor(
        &mut self,
        descriptor_object: ObjectRef,
    ) -> Result<PropertyDescriptor, Self::Error> {
        self.builtin_to_property_descriptor(descriptor_object)
    }

    fn set_integrity_level(
        &mut self,
        object: ObjectRef,
        freeze: bool,
    ) -> Result<bool, Self::Error> {
        self.vm.set_integrity_level(self.agent, object, freeze)
    }

    fn test_integrity_level(
        &mut self,
        object: ObjectRef,
        frozen: bool,
    ) -> Result<bool, Self::Error> {
        self.vm.test_integrity_level(self.agent, object, frozen)
    }

    fn park_agent(
        &mut self,
        request: &lyng_js_host::ParkAgentRequest,
    ) -> Result<lyng_js_host::ParkAgentResult, Self::Error> {
        self.host.park_agent(request).map_err(VmError::Host)
    }

    fn unpark_agent(
        &mut self,
        request: &lyng_js_host::UnparkAgentRequest,
    ) -> Result<lyng_js_host::UnparkAgentResult, Self::Error> {
        self.host.unpark_agent(request).map_err(VmError::Host)
    }

    fn temporal_current_instant(
        &mut self,
        request: &TemporalCurrentInstantRequest,
    ) -> Result<TemporalInstant, Self::Error> {
        self.map_temporal_host_result(self.host.temporal_current_instant(request))
    }

    fn temporal_default_time_zone(
        &mut self,
        request: &TemporalDefaultTimeZoneRequest,
    ) -> Result<TemporalDefaultTimeZone, Self::Error> {
        self.map_temporal_host_result(self.host.temporal_default_time_zone(request))
    }

    fn temporal_instant_to_civil_time(
        &mut self,
        request: &TemporalInstantToCivilRequest,
    ) -> Result<TemporalCivilTime, Self::Error> {
        self.map_temporal_host_result(self.host.temporal_instant_to_civil_time(request))
    }

    fn temporal_civil_time_to_instant(
        &mut self,
        request: &TemporalCivilToInstantRequest,
    ) -> Result<TemporalInstantWithOffset, Self::Error> {
        self.map_temporal_host_result(self.host.temporal_civil_time_to_instant(request))
    }

    fn require_callable_object(&mut self, value: Value) -> Result<ObjectRef, Self::Error> {
        Vm::require_callable_object(self.agent, self.caller_frame, value).map_err(|error| {
            match error {
                VmError::Abrupt(abrupt) => self.abrupt(abrupt),
                other => other,
            }
        })
    }

    fn call_to_completion(
        &mut self,
        callee_object: ObjectRef,
        this_value: Value,
        arguments: &[Value],
    ) -> Result<Value, Self::Error> {
        self.vm.call_to_completion(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            callee_object,
            this_value,
            arguments,
        )
    }

    fn construct_to_completion(
        &mut self,
        callee_object: ObjectRef,
        arguments: &[Value],
        new_target: Option<ObjectRef>,
    ) -> Result<ObjectRef, Self::Error> {
        self.vm.construct_to_completion(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            callee_object,
            arguments,
            new_target,
        )
    }

    fn collect_array_like_arguments(
        &mut self,
        realm: RealmRef,
        value: Value,
    ) -> Result<Vec<Value>, Self::Error> {
        self.vm.collect_array_like_arguments(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            realm,
            value,
        )
    }

    fn create_bound_function(
        &mut self,
        target: ObjectRef,
        bound_this: Value,
        bound_arguments: &[Value],
    ) -> Result<ObjectRef, Self::Error> {
        self.vm.create_bound_function(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            target,
            bound_this,
            bound_arguments,
        )
    }

    fn create_dynamic_function(
        &mut self,
        realm: RealmRef,
        parameters_source: &str,
        body_source: &str,
        strict_caller: bool,
        kind: DynamicFunctionKind,
        new_target: Option<ObjectRef>,
    ) -> Result<ObjectRef, Self::Error> {
        let installed = self.vm.install_dynamic_function(
            self.agent,
            realm,
            parameters_source,
            body_source,
            strict_caller,
            kind,
        )?;
        let (lexical_env, variable_env) = if let Some(realm_record) = self.agent.realm(realm) {
            (realm_record.global_env(), realm_record.global_env())
        } else {
            return Err(VmError::Abrupt(errors::throw_type_error(self.agent)));
        };
        let function = self
            .vm
            .evaluate_installed(self.agent, installed, lexical_env, variable_env)?
            .as_object_ref()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(self.agent)))?;
        let default_prototype = self
            .agent
            .realm(realm)
            .and_then(|record| match kind {
                DynamicFunctionKind::Ordinary => record.intrinsics().function_prototype(),
                DynamicFunctionKind::Generator => {
                    record.intrinsics().generator_function_prototype()
                }
                DynamicFunctionKind::Async => record.intrinsics().async_function_prototype(),
                DynamicFunctionKind::AsyncGenerator => {
                    record.intrinsics().async_generator_function_prototype()
                }
            })
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(self.agent)))?;
        let prototype = self.builtin_constructor_prototype(realm, default_prototype, new_target)?;
        let _ = self.agent.with_heap_and_objects(|heap, objects| {
            objects.set_prototype(&mut heap.mutator(), function, Some(prototype))
        });
        Ok(function)
    }

    fn generator_next(&mut self, generator: ObjectRef, value: Value) -> Result<Value, Self::Error> {
        if self.vm.is_async_generator_object(generator) {
            return self.vm.resume_async_generator(
                self.agent,
                self.host,
                self.registry,
                self.caller_frame,
                generator,
                GeneratorResumeKind::Next,
                value,
            );
        }
        self.vm.resume_generator(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            generator,
            GeneratorResumeKind::Next,
            value,
        )
    }

    fn generator_return(
        &mut self,
        generator: ObjectRef,
        value: Value,
    ) -> Result<Value, Self::Error> {
        if self.vm.is_async_generator_object(generator) {
            return self.vm.resume_async_generator(
                self.agent,
                self.host,
                self.registry,
                self.caller_frame,
                generator,
                GeneratorResumeKind::Return,
                value,
            );
        }
        self.vm.resume_generator(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            generator,
            GeneratorResumeKind::Return,
            value,
        )
    }

    fn generator_throw(
        &mut self,
        generator: ObjectRef,
        value: Value,
    ) -> Result<Value, Self::Error> {
        if self.vm.is_async_generator_object(generator) {
            return self.vm.resume_async_generator(
                self.agent,
                self.host,
                self.registry,
                self.caller_frame,
                generator,
                GeneratorResumeKind::Throw,
                value,
            );
        }
        self.vm.resume_generator(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            generator,
            GeneratorResumeKind::Throw,
            value,
        )
    }

    fn async_generator_next(
        &mut self,
        this_value: Value,
        value: Value,
    ) -> Result<Value, Self::Error> {
        self.vm.resume_async_generator_from_value(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            this_value,
            GeneratorResumeKind::Next,
            value,
        )
    }

    fn async_generator_return(
        &mut self,
        this_value: Value,
        value: Value,
    ) -> Result<Value, Self::Error> {
        self.vm.resume_async_generator_from_value(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            this_value,
            GeneratorResumeKind::Return,
            value,
        )
    }

    fn async_generator_throw(
        &mut self,
        this_value: Value,
        value: Value,
    ) -> Result<Value, Self::Error> {
        self.vm.resume_async_generator_from_value(
            self.agent,
            self.host,
            self.registry,
            self.caller_frame,
            this_value,
            GeneratorResumeKind::Throw,
            value,
        )
    }

    fn evaluate_script_in_realm(
        &mut self,
        realm: RealmRef,
        source_text: &str,
    ) -> Result<Value, Self::Error> {
        self.vm.evaluate_indirect_eval_source(
            self.agent,
            self.host,
            self.registry,
            realm,
            source_text,
        )
    }

    fn function_to_string_text(&mut self, function: ObjectRef) -> Result<String, Self::Error> {
        if !self.agent.objects().is_callable(function) {
            return Err(VmError::Abrupt(errors::throw_type_error(self.agent)));
        }
        let Some(entry) = self
            .agent
            .objects()
            .function_data(function)
            .and_then(|data| data.entry())
        else {
            return self.vm.native_function_source_text(self.agent, function);
        };
        match entry {
            lyng_js_objects::FunctionEntryIdentity::Bytecode(code) => self
                .vm
                .source_function_source_text(self.agent, code, function),
            lyng_js_objects::FunctionEntryIdentity::Native(_)
            | lyng_js_objects::FunctionEntryIdentity::Bound => {
                self.vm.native_function_source_text(self.agent, function)
            }
        }
    }
}

fn bound_function_length_value(target_length: Value, bound_argument_count: usize) -> Value {
    let Some(number) = target_length.as_f64() else {
        return Value::from_smi(0);
    };

    let length = if number.is_nan() || number == 0.0 {
        0.0
    } else if number.is_infinite() {
        if number.is_sign_positive() {
            f64::INFINITY
        } else {
            0.0
        }
    } else {
        (number.trunc() - bound_argument_count as f64).max(0.0)
    };

    if length.is_infinite() {
        return Value::from_f64(length);
    }
    if let Ok(integer) = i32::try_from(length as i64) {
        if f64::from(integer) == length {
            return Value::from_smi(integer);
        }
    }
    Value::from_f64(length)
}
