use super::*;

impl Vm {
    pub(super) fn create_bound_function(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        target: ObjectRef,
        bound_this: Value,
        bound_arguments: &[Value],
    ) -> VmResult<ObjectRef> {
        if !agent.objects().is_callable(target) {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        let target_data = bound_target_function_data(agent, target)?;
        let realm = target_data
            .as_ref()
            .and_then(FunctionObjectData::realm)
            .unwrap_or(caller.realm());
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
            .as_ref()
            .and_then(FunctionObjectData::environment)
            .unwrap_or(realm_record.global_env());
        let mut function_data = FunctionObjectData::bound(
            realm,
            environment,
            target,
            bound_this,
            bound_arguments.to_vec().into_boxed_slice(),
        )
        .with_has_prototype_property(false)
        .with_constructor_flags(if agent.objects().is_constructor(target) {
            FunctionConstructorFlags::constructible()
        } else {
            FunctionConstructorFlags::empty()
        })
        .with_this_mode(FunctionThisMode::Strict);
        if let Some(target_data) = target_data.as_ref() {
            function_data = function_data.with_kind_flags(target_data.kind_flags());
        }
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
        let target_has_own_length = object::get_own_property_in_context(
            &mut VmProxyBridge {
                vm: self,
                agent,
                host,
                registry,
                frame: caller,
            },
            target,
            length_key,
        )?
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

    pub(super) fn collect_array_like_arguments(
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
        if let Some(arguments) = Self::try_collect_fast_engine_array_arguments(agent, object)? {
            return Ok(arguments);
        }
        let length = self.get_property_from_object(
            agent,
            host,
            registry,
            caller_frame,
            object,
            Value::from_object_ref(object),
            PropertyKey::from_atom(WellKnownAtom::length.id()),
        )?;
        let length =
            self.to_length_for_array_like_arguments(agent, host, registry, caller_frame, length)?;
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

    fn to_length_for_array_like_arguments(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameRecord,
        value: Value,
    ) -> VmResult<u32> {
        const MAX_SAFE_LENGTH: f64 = 9_007_199_254_740_991.0;
        let primitive = self.to_primitive(
            agent,
            host,
            registry,
            caller_frame,
            value,
            ToPrimitiveHint::Number,
        )?;
        let number = read::to_number(agent.heap().view(), primitive)
            .map_err(|_| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let number = number_value_to_f64(number);
        if number.is_nan() || number <= 0.0 {
            return Ok(0);
        }
        if !number.is_finite() {
            return Ok(u32::MAX);
        }
        Ok(number.trunc().min(MAX_SAFE_LENGTH).min(f64::from(u32::MAX)) as u32)
    }

    fn try_collect_fast_engine_array_arguments(
        agent: &mut Agent,
        object: ObjectRef,
    ) -> VmResult<Option<Vec<Value>>> {
        if !agent
            .objects()
            .object_header(agent.heap().view(), object)
            .is_some_and(|header| header.flags().is_engine_array())
            || !Self::engine_array_index_prototype_chain_is_clear(agent, object)?
            || agent.objects().element_mode(object) == Some(lyng_js_objects::ElementMode::Sparse)
        {
            return Ok(None);
        }

        let length_descriptor = agent
            .objects()
            .get_own_property(
                agent.heap().view(),
                object,
                PropertyKey::from_atom(WellKnownAtom::length.id()),
            )
            .map_err(|_error| VmError::Abrupt(errors::throw_type_error(agent)))?
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let Some(length_value) = length_descriptor.value() else {
            return Ok(None);
        };
        let Some(length) = length_value
            .as_smi()
            .and_then(|value| u32::try_from(value).ok())
            .or_else(|| length_value.as_f64().and_then(number_to_u32_length))
        else {
            return Ok(None);
        };
        let capacity = usize::try_from(length).unwrap_or(usize::MAX);
        let mut arguments = Vec::with_capacity(capacity);
        for index in 0..length {
            let value = agent
                .objects()
                .element(agent.heap().view(), object, index)
                .unwrap_or(Value::array_hole());
            arguments.push(if value == Value::array_hole() {
                Value::undefined()
            } else {
                value
            });
        }
        Ok(Some(arguments))
    }

    pub(super) fn instantiate_dynamic_function(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        realm: RealmRef,
        installed: crate::InstalledCode,
        prototype: ObjectRef,
    ) -> VmResult<ObjectRef> {
        let (lexical_env, variable_env) = if let Some(realm_record) = agent.realm(realm) {
            (realm_record.global_env(), realm_record.global_env())
        } else {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        };
        let script_referrer = self
            .active_script_or_module_referrer(agent)
            .map(|key| agent.atoms_mut().intern_collectible(key.as_str()));
        let function = self
            .evaluate_entry_with_registry(
                agent,
                installed,
                lexical_env,
                variable_env,
                script_referrer,
                host,
                registry,
                None,
                None,
            )?
            .as_object_ref()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let _ = agent.with_heap_and_objects(|heap, objects| {
            objects.set_prototype(&mut heap.mutator(), function, Some(prototype))
        });
        Ok(function)
    }

    pub(super) fn native_function_source_text(
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

    pub(super) fn source_function_source_text(
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

    fn function_name_text(&mut self, agent: &mut Agent, function: ObjectRef) -> VmResult<String> {
        let name = object::ordinary_get(
            agent,
            function,
            PropertyKey::from_atom(WellKnownAtom::name.id()),
        )
        .map_err(VmError::Abrupt)?;
        self.value_to_string_text(agent, name)
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
        if !parse_script(atoms, source, &method_text)
            .diagnostics
            .has_errors()
        {
            return true;
        }

        let class_method_text = format!("(class {{{source_text}}});");
        !parse_script(atoms, source, &class_method_text)
            .diagnostics
            .has_errors()
    }
}

fn bound_target_function_data(
    agent: &mut Agent,
    mut target: ObjectRef,
) -> VmResult<Option<FunctionObjectData>> {
    loop {
        if let Some(data) = agent.objects().function_data(target) {
            return Ok(Some(data.clone()));
        }
        if !agent.objects().is_proxy_object(target) {
            return Ok(None);
        }
        let Some(data) = agent.objects().proxy_data(target) else {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        };
        if data.revoked() {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        target = data.target();
    }
}

fn number_to_u32_length(value: f64) -> Option<u32> {
    if !value.is_finite() || value < 0.0 || value.trunc() != value || value > f64::from(u32::MAX) {
        return None;
    }
    Some(value as u32)
}

fn number_value_to_f64(value: Value) -> f64 {
    value
        .as_smi()
        .map(f64::from)
        .or_else(|| value.as_f64())
        .expect("ToNumber must produce a numeric value")
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
