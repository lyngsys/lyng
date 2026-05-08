use super::{
    errors, object, Agent, AllocationLifetime, ClassPrivateElementKind, FrameRecord, HostHooks,
    NativeFunctionRegistry, ObjectRef, Value, Vm, VmError, VmResult,
};

impl Vm {
    pub(in crate::vm::builtin_dispatch) fn bind_function_private_env_builtin(
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
            .and_then(lyng_js_objects::FunctionObjectData::private_env)
            .or_else(|| {
                agent
                    .current_execution_context()
                    .and_then(lyng_js_env::ExecutionContext::private_env)
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

    pub(in crate::vm::builtin_dispatch) fn install_instance_field_key_builtin(
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
        let key = self.property_key_from_value(agent, host, registry, caller, key_value)?;
        let canonical_key = self.property_key_to_enumeration_value(agent, key);
        object::install_instance_public_field_key(agent, class_object, field_index, canonical_key)
            .map_err(VmError::Abrupt)
    }

    pub(in crate::vm::builtin_dispatch) fn get_instance_field_key_builtin(
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

    pub(in crate::vm::builtin_dispatch) fn private_context_class_key(
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
        let callee_object = caller.callee();
        if let Some(home_object) = callee_object.and_then(|callee| {
            agent
                .objects()
                .function_data(callee)
                .and_then(lyng_js_objects::FunctionObjectData::home_object)
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
                    if callee_object.is_some_and(|callee| record.function_object() == callee) {
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

    pub(in crate::vm::builtin_dispatch) fn private_context_from_private_env(
        agent: &Agent,
        descriptor_index: u32,
        class_depth: u32,
    ) -> Option<ObjectRef> {
        let mut current = agent
            .current_execution_context()
            .and_then(lyng_js_env::ExecutionContext::private_env);
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

    pub(in crate::vm::builtin_dispatch) fn define_private_field_builtin(
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
        let name_text = Self::value_to_string_text(agent, name_value)?;
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

    pub(in crate::vm::builtin_dispatch) fn private_field_init_builtin(
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
            Self::private_context_class_key(agent, caller, receiver, descriptor_index, class_depth);
        object::private_field_init(agent, receiver, class_key, descriptor_index, value)
            .map_err(VmError::Abrupt)
    }

    pub(in crate::vm::builtin_dispatch) fn private_field_get_builtin(
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
            Self::private_context_class_key(agent, caller, receiver, descriptor_index, class_depth);
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

    pub(in crate::vm::builtin_dispatch) fn private_field_set_builtin(
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
            Self::private_context_class_key(agent, caller, receiver, descriptor_index, class_depth);
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

    pub(in crate::vm::builtin_dispatch) fn private_has_builtin(
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
            Self::private_context_class_key(agent, caller, receiver, descriptor_index, class_depth);
        let has = object::private_has(agent, receiver, class_key, descriptor_index)
            .map_err(VmError::Abrupt)?;
        Ok(Value::from_bool(has))
    }

    pub(in crate::vm::builtin_dispatch) fn private_element_kind_from_argument(
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
}
