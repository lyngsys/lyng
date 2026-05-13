use super::{
    alloc_string, errors, Agent, AllocationLifetime, FrameRecord, HostHooks,
    NativeFunctionRegistry, PropertyKey, TemplateCacheKey, Value, Vm, VmError, VmResult,
    WellKnownAtom,
};

impl Vm {
    pub(super) fn template_to_string_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: &FrameRecord,
        value: Value,
    ) -> VmResult<Value> {
        if !value.is_object() {
            let text = Self::value_to_string_text(agent, value)?;
            return Ok(Value::from_string_ref(alloc_string(agent, &text, None)));
        }

        let object = value
            .as_object_ref()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let to_string = self.get_property_from_object(
            agent,
            host,
            registry,
            *caller,
            object,
            Value::from_object_ref(object),
            PropertyKey::from_atom(WellKnownAtom::toString.id()),
        )?;
        if let Some(result) = self.call_if_callable_object(
            agent,
            host,
            registry,
            *caller,
            to_string,
            Value::from_object_ref(object),
            &[],
        )? && !result.is_object()
        {
            let text = Self::value_to_string_text(agent, result)?;
            return Ok(Value::from_string_ref(alloc_string(agent, &text, None)));
        }

        let value_of = self.get_property_from_object(
            agent,
            host,
            registry,
            *caller,
            object,
            Value::from_object_ref(object),
            PropertyKey::from_atom(WellKnownAtom::valueOf.id()),
        )?;
        if let Some(result) = self.call_if_callable_object(
            agent,
            host,
            registry,
            *caller,
            value_of,
            Value::from_object_ref(object),
            &[],
        )? && !result.is_object()
        {
            let text = Self::value_to_string_text(agent, result)?;
            return Ok(Value::from_string_ref(alloc_string(agent, &text, None)));
        }

        Err(VmError::Abrupt(errors::throw_type_error(agent)))
    }

    pub(super) fn get_template_object_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: &FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let site = arguments
            .first()
            .copied()
            .and_then(Value::as_smi)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let key = TemplateCacheKey {
            realm: caller.realm(),
            code: caller.code(),
            site,
        };
        if let Some(template) = self.template_cache.get(&key).copied() {
            return Ok(Value::from_object_ref(template));
        }

        let string_count = arguments.len().saturating_sub(1) / 2;
        let cooked = Self::create_array(agent, caller.realm(), string_count)?;
        let raw = Self::create_array(agent, caller.realm(), string_count)?;
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
        Self::sync_engine_array_length(agent, cooked)?;
        Self::sync_engine_array_length(agent, raw)?;
        if !self.set_integrity_level(agent, host, registry, caller, raw, true)? {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }

        let raw_name = agent.atoms_mut().intern_collectible("raw");
        Self::define_data_property_with_attrs(
            agent,
            cooked,
            PropertyKey::from_atom(raw_name),
            Value::from_object_ref(raw),
            false,
            false,
            false,
        )?;
        if !self.set_integrity_level(agent, host, registry, caller, cooked, true)? {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        self.template_cache.insert(key, cooked);
        Ok(Value::from_object_ref(cooked))
    }
}
