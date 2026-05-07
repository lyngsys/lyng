use super::{
    errors, object, Agent, AllocationLifetime, ClassPrivateElementKind, EnvironmentLayout,
    EnvironmentLayoutKind, FrameRecord, FunctionEntryIdentity, FunctionThisMode, HostHooks,
    NativeFunctionRegistry, ObjectRef, PropertyDescriptor, ThisBindingStatus, ThisState, Value, Vm,
    VmError, VmResult,
};

mod private_fields;
mod super_ops;

impl Vm {
    pub(super) fn define_accessor_property_builtin(
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

    pub(super) fn define_method_property_builtin(
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

    pub(super) fn set_function_home_object_builtin(
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

    pub(super) fn capture_arrow_context_builtin(
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
            .and_then(lyng_js_objects::FunctionObjectData::environment)
            .or_else(|| {
                agent
                    .current_execution_context()
                    .map(lyng_js_env::ExecutionContext::lexical_env)
            })
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let layout = agent.alloc_environment_layout(EnvironmentLayout::empty(
            EnvironmentLayoutKind::Function,
            true,
        ));
        let new_target = agent
            .current_execution_context()
            .and_then(lyng_js_env::ExecutionContext::new_target);
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

    pub(super) fn object_literal_set_prototype_builtin(
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
            object::ordinary_set_prototype_of(agent, object, prototype).map_err(VmError::Abrupt)?;
        if !changed {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(Value::from_object_ref(object))
    }
}
