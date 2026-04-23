use super::*;
use crate::name_refs::{CapturedNameReference, CapturedNameTarget};
use crate::vm::property_access::VmProxyBridge;
use lyng_js_env::{
    EnvironmentRecord, GlobalEnvironmentRecord, GlobalLexicalBindingRecord, ObjectEnvironmentRecord,
};
use lyng_js_host::HostHooks;
#[cfg(test)]
use lyng_js_host::NoopHostHooks;
use lyng_js_objects::NativeFunctionRegistry;
use lyng_js_ops::{errors, object, proxy, read};
use lyng_js_types::PropertyKey;

fn layout_binding_slot(
    agent: &Agent,
    environment: EnvironmentRef,
    layout: lyng_js_env::EnvironmentLayoutId,
    name: AtomId,
) -> Option<u32> {
    let layout = agent.environment_layout(layout)?;
    for (index, binding) in layout.bindings().iter().enumerate() {
        if binding.name() != Some(name) {
            continue;
        }
        let index = u32::try_from(index).ok()?;
        let value = agent.environment_slot(environment, index)?;
        if binding.flags().is_dynamic() && value == Value::deleted_environment_binding() {
            continue;
        }
        return Some(index);
    }
    None
}

impl Vm {
    fn object_environment_has_binding_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        record: ObjectEnvironmentRecord,
        name: AtomId,
    ) -> VmResult<bool> {
        let key = PropertyKey::from_atom(name);
        let binding_object = record.binding_object();
        let receiver = Value::from_object_ref(binding_object);
        let found = {
            let mut bridge = VmProxyBridge {
                vm: self,
                agent,
                host,
                registry,
                frame,
            };
            proxy::has_property(&mut bridge, binding_object, key)?
        };
        if !found {
            return Ok(false);
        }
        if !record.with_environment() {
            return Ok(true);
        }

        let Some(unscopables_symbol) = agent.well_known_symbol(WellKnownSymbolId::Unscopables)
        else {
            return Ok(true);
        };
        let unscopables = self.get_property_from_object(
            agent,
            host,
            registry,
            frame,
            binding_object,
            receiver,
            PropertyKey::from_symbol(unscopables_symbol),
        )?;
        let Some(unscopables_object) = unscopables.as_object_ref() else {
            return Ok(true);
        };
        let blocked = self.get_property_from_object(
            agent,
            host,
            registry,
            frame,
            unscopables_object,
            Value::from_object_ref(unscopables_object),
            key,
        )?;
        Ok(!read::to_boolean(agent.heap().view(), blocked).map_err(VmError::Abrupt)?)
    }

    fn object_environment_get_binding_value_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        record: ObjectEnvironmentRecord,
        name: AtomId,
        strict: bool,
    ) -> VmResult<Value> {
        let key = PropertyKey::from_atom(name);
        let binding_object = record.binding_object();
        let still_exists = {
            let mut bridge = VmProxyBridge {
                vm: self,
                agent,
                host,
                registry,
                frame,
            };
            proxy::has_property(&mut bridge, binding_object, key)?
        };
        if !still_exists {
            if strict {
                return Err(VmError::Abrupt(errors::throw_reference_error(agent)));
            }
            return Ok(Value::undefined());
        }
        self.get_property_from_object(
            agent,
            host,
            registry,
            frame,
            binding_object,
            Value::from_object_ref(binding_object),
            key,
        )
    }

    fn object_environment_set_mutable_binding_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        record: ObjectEnvironmentRecord,
        name: AtomId,
        value: Value,
        strict: bool,
    ) -> VmResult<()> {
        let key = PropertyKey::from_atom(name);
        let binding_object = record.binding_object();
        let still_exists = {
            let mut bridge = VmProxyBridge {
                vm: self,
                agent,
                host,
                registry,
                frame,
            };
            proxy::has_property(&mut bridge, binding_object, key)?
        };
        if !still_exists && strict {
            return Err(VmError::Abrupt(errors::throw_reference_error(agent)));
        }

        let stored = self.set_property_on_object(
            agent,
            host,
            registry,
            frame,
            binding_object,
            Value::from_object_ref(binding_object),
            key,
            value,
        )?;
        if !stored && strict {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(())
    }

    fn object_environment_delete_binding_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        record: ObjectEnvironmentRecord,
        name: AtomId,
    ) -> VmResult<bool> {
        let mut bridge = VmProxyBridge {
            vm: self,
            agent,
            host,
            registry,
            frame,
        };
        proxy::delete_property(
            &mut bridge,
            record.binding_object(),
            PropertyKey::from_atom(name),
        )
    }

    fn probe_identifier_value_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        start: EnvironmentRef,
        name: AtomId,
        strict: bool,
    ) -> VmResult<Option<Value>> {
        let mut current = Some(start);
        while let Some(environment) = current {
            let record = agent
                .environment(environment)
                .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
            match record {
                EnvironmentRecord::Declarative(record) => {
                    if let Some(slot) =
                        layout_binding_slot(agent, record.id(), record.layout(), name)
                    {
                        let environment =
                            self.environment_for_slot_access(agent, record.id(), 0, slot)?;
                        return self
                            .read_environment_slot(agent, environment, slot)
                            .map(Some);
                    }
                    current = record.outer();
                }
                EnvironmentRecord::Private(record) => current = record.outer(),
                EnvironmentRecord::Function(record) => {
                    let declarative = record.declarative();
                    if let Some(slot) =
                        layout_binding_slot(agent, declarative.id(), declarative.layout(), name)
                    {
                        let environment =
                            self.environment_for_slot_access(agent, declarative.id(), 0, slot)?;
                        return self
                            .read_environment_slot(agent, environment, slot)
                            .map(Some);
                    }
                    current = declarative.outer();
                }
                EnvironmentRecord::Module(record) => {
                    if let Some(slot) =
                        layout_binding_slot(agent, record.id(), record.layout(), name)
                    {
                        let environment =
                            self.environment_for_slot_access(agent, record.id(), 0, slot)?;
                        return self
                            .read_environment_slot(agent, environment, slot)
                            .map(Some);
                    }
                    current = record.outer();
                }
                EnvironmentRecord::Global(record) => {
                    if let Some(binding) = self.lookup_global_lexical_binding(agent, &record, name)
                    {
                        let environment = self.environment_for_slot_access(
                            agent,
                            binding.environment(),
                            0,
                            binding.slot(),
                        )?;
                        return self
                            .read_environment_slot(agent, environment, binding.slot())
                            .map(Some);
                    }
                    if let Some(slot) =
                        layout_binding_slot(agent, record.id(), record.layout(), name)
                    {
                        let environment =
                            self.environment_for_slot_access(agent, record.id(), 0, slot)?;
                        return self
                            .read_environment_slot(agent, environment, slot)
                            .map(Some);
                    }
                    if record.has_var_name(name) {
                        return self
                            .get_global_property_binding_with_context(
                                agent,
                                host,
                                registry,
                                frame,
                                record.id(),
                                name,
                            )
                            .map(|value| Some(value.unwrap_or_else(Value::undefined)));
                    }
                    current = record.outer();
                }
                EnvironmentRecord::Object(record) => {
                    if self.object_environment_has_binding_with_context(
                        agent, host, registry, frame, record, name,
                    )? {
                        let value = self.object_environment_get_binding_value_with_context(
                            agent, host, registry, frame, record, name, strict,
                        )?;
                        return Ok(Some(value));
                    }
                    current = record.outer();
                }
            }
        }
        Ok(None)
    }

    fn resolve_dynamic_name_target_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        start: EnvironmentRef,
        name: AtomId,
    ) -> VmResult<CapturedNameTarget> {
        let mut current = Some(start);
        while let Some(environment) = current {
            let Some(record) = agent.environment(environment) else {
                break;
            };
            match record {
                EnvironmentRecord::Declarative(record) => {
                    if let Some(slot) =
                        layout_binding_slot(agent, record.id(), record.layout(), name)
                    {
                        let environment =
                            self.environment_for_slot_access(agent, record.id(), 0, slot)?;
                        return Ok(CapturedNameTarget::EnvironmentSlot { environment, slot });
                    }
                    current = record.outer();
                }
                EnvironmentRecord::Private(record) => current = record.outer(),
                EnvironmentRecord::Function(record) => {
                    let declarative = record.declarative();
                    if let Some(slot) =
                        layout_binding_slot(agent, declarative.id(), declarative.layout(), name)
                    {
                        let environment =
                            self.environment_for_slot_access(agent, declarative.id(), 0, slot)?;
                        return Ok(CapturedNameTarget::EnvironmentSlot { environment, slot });
                    }
                    current = declarative.outer();
                }
                EnvironmentRecord::Module(record) => {
                    if let Some(slot) =
                        layout_binding_slot(agent, record.id(), record.layout(), name)
                    {
                        let environment =
                            self.environment_for_slot_access(agent, record.id(), 0, slot)?;
                        return Ok(CapturedNameTarget::EnvironmentSlot { environment, slot });
                    }
                    current = record.outer();
                }
                EnvironmentRecord::Global(record) => {
                    if let Some(binding) = self.lookup_global_lexical_binding(agent, &record, name)
                    {
                        let environment = self.environment_for_slot_access(
                            agent,
                            binding.environment(),
                            0,
                            binding.slot(),
                        )?;
                        return Ok(CapturedNameTarget::EnvironmentSlot {
                            environment,
                            slot: binding.slot(),
                        });
                    }
                    let has_global_property = object::has_property(
                        agent,
                        record.global_object(),
                        PropertyKey::from_atom(name),
                    )
                    .map_err(VmError::Abrupt)?;
                    if record.has_var_name(name) || has_global_property {
                        return Ok(CapturedNameTarget::GlobalProperty {
                            environment: record.id(),
                        });
                    }
                    current = record.outer();
                }
                EnvironmentRecord::Object(record) => {
                    if self.object_environment_has_binding_with_context(
                        agent, host, registry, frame, record, name,
                    )? {
                        return Ok(CapturedNameTarget::ObjectProperty { record });
                    }
                    current = record.outer();
                }
            }
        }
        let global = self.find_global_environment(agent, frame.variable_env())?;
        Ok(CapturedNameTarget::Unresolvable {
            global_environment: global.id(),
        })
    }

    pub(super) fn load_global(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        name: AtomId,
    ) -> VmResult<Value> {
        let global = self.find_global_environment(agent, frame.variable_env())?;
        if let Some(binding) = self.lookup_global_lexical_binding(agent, &global, name) {
            let environment =
                self.environment_for_slot_access(agent, binding.environment(), 0, binding.slot())?;
            return self.read_environment_slot(agent, environment, binding.slot());
        }
        self.get_global_property_binding_with_context(
            agent,
            host,
            registry,
            frame,
            frame.variable_env(),
            name,
        )?
        .ok_or_else(|| VmError::Abrupt(errors::throw_reference_error(agent)))
    }

    pub(super) fn store_global(
        &mut self,
        agent: &mut Agent,
        frame: FrameRecord,
        name: AtomId,
        value: Value,
    ) -> VmResult<()> {
        let global = self.find_global_environment(agent, frame.variable_env())?;
        if let Some(binding) = self.lookup_global_lexical_binding(agent, &global, name) {
            let environment =
                self.environment_for_slot_access(agent, binding.environment(), 0, binding.slot())?;
            return self.write_environment_slot(agent, environment, binding.slot(), value);
        }
        let _ = object::set(
            agent,
            global.global_object(),
            PropertyKey::from_atom(name),
            value,
            AllocationLifetime::Default,
        )
        .map_err(VmError::Abrupt)?;
        Ok(())
    }

    pub(super) fn assign_global(
        &mut self,
        agent: &mut Agent,
        frame: FrameRecord,
        name: AtomId,
        value: Value,
    ) -> VmResult<()> {
        let global = self.find_global_environment(agent, frame.variable_env())?;
        if let Some(binding) = self.lookup_global_lexical_binding(agent, &global, name) {
            let environment =
                self.environment_for_slot_access(agent, binding.environment(), 0, binding.slot())?;
            return self.assign_environment_slot(
                agent,
                environment,
                binding.slot(),
                value,
                self.frame_is_strict(frame),
            );
        }

        let key = PropertyKey::from_atom(name);
        let has_property =
            object::has_property(agent, global.global_object(), key).map_err(VmError::Abrupt)?;
        if !has_property && self.frame_is_strict(frame) {
            return Err(VmError::Abrupt(errors::throw_reference_error(agent)));
        }

        let stored = object::set(
            agent,
            global.global_object(),
            key,
            value,
            AllocationLifetime::Default,
        )
        .map_err(VmError::Abrupt)?;
        if !stored && self.frame_is_strict(frame) {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(())
    }

    fn capture_name_reference_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        name: AtomId,
    ) -> VmResult<CapturedNameReference> {
        let lexical_env = self.dynamic_name_start_environment(frame);
        let target = self.resolve_dynamic_name_target_with_context(
            agent,
            host,
            registry,
            frame,
            lexical_env,
            name,
        )?;
        Ok(CapturedNameReference::new(name, target))
    }

    fn load_global_property_binding(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        environment: EnvironmentRef,
        name: AtomId,
        strict: bool,
    ) -> VmResult<Value> {
        match self.get_global_property_binding_with_context(
            agent,
            host,
            registry,
            frame,
            environment,
            name,
        )? {
            Some(value) => Ok(value),
            None if strict => Err(VmError::Abrupt(errors::throw_reference_error(agent))),
            None => Ok(Value::undefined()),
        }
    }

    fn assign_global_property_binding(
        &mut self,
        agent: &mut Agent,
        environment: EnvironmentRef,
        name: AtomId,
        value: Value,
        strict: bool,
    ) -> VmResult<()> {
        let global = self.find_global_environment(agent, environment)?;
        let key = PropertyKey::from_atom(name);
        let has_property =
            object::has_property(agent, global.global_object(), key).map_err(VmError::Abrupt)?;
        if !has_property && strict {
            return Err(VmError::Abrupt(errors::throw_reference_error(agent)));
        }
        let stored = object::set(
            agent,
            global.global_object(),
            key,
            value,
            AllocationLifetime::Default,
        )
        .map_err(VmError::Abrupt)?;
        if !stored && strict {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(())
    }

    fn assign_unresolvable_name(
        &mut self,
        agent: &mut Agent,
        global_environment: EnvironmentRef,
        name: AtomId,
        value: Value,
        strict: bool,
    ) -> VmResult<()> {
        if strict {
            return Err(VmError::Abrupt(errors::throw_reference_error(agent)));
        }
        let global = self.find_global_environment(agent, global_environment)?;
        let _ = object::set(
            agent,
            global.global_object(),
            PropertyKey::from_atom(name),
            value,
            AllocationLifetime::Default,
        )
        .map_err(VmError::Abrupt)?;
        Ok(())
    }

    pub(super) fn load_captured_name_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        reference_register: u16,
    ) -> VmResult<Value> {
        let reference = self
            .captured_name_references
            .get(frame.registers().base(), reference_register)
            .ok_or(VmError::RegisterOutOfBounds {
                code: frame.code(),
                register: reference_register,
            })?;
        match reference.target() {
            CapturedNameTarget::EnvironmentSlot { environment, slot } => {
                self.read_environment_slot(agent, environment, slot)
            }
            CapturedNameTarget::ObjectProperty { record } => self
                .object_environment_get_binding_value_with_context(
                    agent,
                    host,
                    registry,
                    frame,
                    record,
                    reference.name(),
                    self.frame_is_strict(frame),
                ),
            CapturedNameTarget::GlobalProperty { environment } => self
                .load_global_property_binding(
                    agent,
                    host,
                    registry,
                    frame,
                    environment,
                    reference.name(),
                    self.frame_is_strict(frame),
                ),
            CapturedNameTarget::Unresolvable { .. } => {
                Err(VmError::Abrupt(errors::throw_reference_error(agent)))
            }
        }
    }

    pub(super) fn assign_captured_name_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        reference_register: u16,
        value: Value,
    ) -> VmResult<()> {
        let reference = self
            .captured_name_references
            .get(frame.registers().base(), reference_register)
            .ok_or(VmError::RegisterOutOfBounds {
                code: frame.code(),
                register: reference_register,
            })?;
        match reference.target() {
            CapturedNameTarget::EnvironmentSlot { environment, slot } => self
                .assign_environment_slot(
                    agent,
                    environment,
                    slot,
                    value,
                    self.frame_is_strict(frame),
                ),
            CapturedNameTarget::ObjectProperty { record } => self
                .object_environment_set_mutable_binding_with_context(
                    agent,
                    host,
                    registry,
                    frame,
                    record,
                    reference.name(),
                    value,
                    self.frame_is_strict(frame),
                ),
            CapturedNameTarget::GlobalProperty { environment } => self
                .assign_global_property_binding(
                    agent,
                    environment,
                    reference.name(),
                    value,
                    self.frame_is_strict(frame),
                ),
            CapturedNameTarget::Unresolvable { global_environment } => self
                .assign_unresolvable_name(
                    agent,
                    global_environment,
                    reference.name(),
                    value,
                    self.frame_is_strict(frame),
                ),
        }
    }

    pub(super) fn resolve_name_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        name: AtomId,
    ) -> VmResult<Value> {
        let lexical_env = self.dynamic_name_start_environment(frame);
        if let Some(value) = self.probe_identifier_value_with_context(
            agent,
            host,
            registry,
            frame,
            lexical_env,
            name,
            false,
        )? {
            return Ok(value);
        }
        Ok(self
            .lookup_global_property(agent, frame.variable_env(), name)?
            .unwrap_or_else(Value::undefined))
    }

    pub(super) fn load_name_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        name: AtomId,
    ) -> VmResult<Value> {
        let lexical_env = self.dynamic_name_start_environment(frame);
        if let Some(value) = self.probe_identifier_value_with_context(
            agent,
            host,
            registry,
            frame,
            lexical_env,
            name,
            self.frame_is_strict(frame),
        )? {
            return Ok(value);
        }
        self.lookup_global_property(agent, frame.variable_env(), name)?
            .ok_or_else(|| VmError::Abrupt(errors::throw_reference_error(agent)))
    }

    pub(super) fn capture_name_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        reference_register: u16,
        name: AtomId,
    ) -> VmResult<()> {
        let reference =
            self.capture_name_reference_with_context(agent, host, registry, frame, name)?;
        self.captured_name_references.insert(
            frame.registers().base(),
            reference_register,
            reference,
        );
        Ok(())
    }

    pub(super) fn assign_name_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        name: AtomId,
        value: Value,
    ) -> VmResult<()> {
        let reference =
            self.capture_name_reference_with_context(agent, host, registry, frame, name)?;
        match reference.target() {
            CapturedNameTarget::EnvironmentSlot { environment, slot } => self
                .assign_environment_slot(
                    agent,
                    environment,
                    slot,
                    value,
                    self.frame_is_strict(frame),
                ),
            CapturedNameTarget::ObjectProperty { record } => self
                .object_environment_set_mutable_binding_with_context(
                    agent,
                    host,
                    registry,
                    frame,
                    record,
                    name,
                    value,
                    self.frame_is_strict(frame),
                ),
            CapturedNameTarget::GlobalProperty { environment } => self
                .assign_global_property_binding(
                    agent,
                    environment,
                    name,
                    value,
                    self.frame_is_strict(frame),
                ),
            CapturedNameTarget::Unresolvable { global_environment } => self
                .assign_unresolvable_name(
                    agent,
                    global_environment,
                    name,
                    value,
                    self.frame_is_strict(frame),
                ),
        }
    }

    pub(super) fn delete_name_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        name: AtomId,
    ) -> VmResult<bool> {
        let lexical_env = self.dynamic_name_start_environment(frame);
        let target = self.resolve_dynamic_name_target_with_context(
            agent,
            host,
            registry,
            frame,
            lexical_env,
            name,
        )?;
        match target {
            CapturedNameTarget::EnvironmentSlot { environment, slot } => {
                let deletable = agent
                    .environment(environment)
                    .and_then(|record| record.layout())
                    .and_then(|layout| agent.environment_layout(layout))
                    .and_then(|layout| layout.binding(slot))
                    .is_some_and(|binding| binding.flags().is_dynamic());
                if !deletable {
                    return Ok(false);
                }
                if !agent.set_environment_slot(
                    environment,
                    slot,
                    Value::deleted_environment_binding(),
                ) {
                    return Err(VmError::MissingEnvironment(environment));
                }
                Ok(true)
            }
            CapturedNameTarget::ObjectProperty { record } => self
                .object_environment_delete_binding_with_context(
                    agent, host, registry, frame, record, name,
                ),
            CapturedNameTarget::GlobalProperty { .. } => self.delete_global(agent, frame, name),
            CapturedNameTarget::Unresolvable { .. } => Ok(true),
        }
    }

    #[cfg(test)]
    pub(super) fn load_name(
        &mut self,
        agent: &mut Agent,
        frame: FrameRecord,
        name: AtomId,
    ) -> VmResult<Value> {
        let host = NoopHostHooks;
        let mut registry = RejectingNativeRegistry;
        self.load_name_with_context(agent, &host, &mut registry, frame, name)
    }

    pub(super) fn delete_global(
        &self,
        agent: &mut Agent,
        frame: FrameRecord,
        name: AtomId,
    ) -> VmResult<bool> {
        let global = self.find_global_environment(agent, frame.variable_env())?;
        if self.global_chain_has_lexical_binding(agent, global.id(), name)
            || self.global_chain_has_var_name(agent, global.id(), name)
        {
            return Ok(false);
        }
        object::delete_property(agent, global.global_object(), PropertyKey::from_atom(name))
            .map_err(VmError::Abrupt)
    }

    pub(super) fn global_has_lexical_binding(
        &self,
        agent: &Agent,
        global: &GlobalEnvironmentRecord,
        name: AtomId,
    ) -> bool {
        global.has_lexical_name(name)
            || agent
                .environment_layout(global.layout())
                .map(|layout| {
                    layout
                        .bindings()
                        .iter()
                        .any(|binding| binding.name() == Some(name))
                })
                .unwrap_or(false)
    }

    pub(super) fn lookup_global_lexical_binding(
        &self,
        agent: &Agent,
        global: &GlobalEnvironmentRecord,
        name: AtomId,
    ) -> Option<GlobalLexicalBindingRecord> {
        global
            .lexical_binding(name)
            .or_else(|| self.lookup_global_layout_binding(agent, global, name))
    }

    pub(super) fn global_chain_has_lexical_binding(
        &self,
        agent: &Agent,
        start: EnvironmentRef,
        name: AtomId,
    ) -> bool {
        let mut current = Some(start);
        while let Some(environment) = current {
            let Some(record) = agent.environment(environment) else {
                break;
            };
            match record {
                EnvironmentRecord::Global(global) => {
                    if self.global_has_lexical_binding(agent, &global, name) {
                        return true;
                    }
                    current = global.outer();
                }
                EnvironmentRecord::Declarative(record) => current = record.outer(),
                EnvironmentRecord::Private(record) => current = record.outer(),
                EnvironmentRecord::Function(record) => current = record.declarative().outer(),
                EnvironmentRecord::Module(record) => current = record.outer(),
                EnvironmentRecord::Object(record) => current = record.outer(),
            }
        }
        false
    }

    pub(super) fn global_chain_has_var_name(
        &self,
        agent: &Agent,
        start: EnvironmentRef,
        name: AtomId,
    ) -> bool {
        let mut current = Some(start);
        while let Some(environment) = current {
            let Some(record) = agent.environment(environment) else {
                break;
            };
            match record {
                EnvironmentRecord::Global(global) => {
                    if global.has_var_name(name) {
                        return true;
                    }
                    current = global.outer();
                }
                EnvironmentRecord::Declarative(record) => current = record.outer(),
                EnvironmentRecord::Private(record) => current = record.outer(),
                EnvironmentRecord::Function(record) => current = record.declarative().outer(),
                EnvironmentRecord::Module(record) => current = record.outer(),
                EnvironmentRecord::Object(record) => current = record.outer(),
            }
        }
        false
    }

    pub(super) fn type_of_value(&self, agent: &mut Agent, value: Value) -> Value {
        let text = if value.is_undefined() {
            "undefined"
        } else if value.is_null() {
            "object"
        } else if value.is_bool() {
            "boolean"
        } else if value.is_number() {
            "number"
        } else if value.is_string() {
            "string"
        } else if value.is_symbol() {
            "symbol"
        } else if value.is_bigint() {
            "bigint"
        } else if let Some(object) = value.as_object_ref() {
            if agent.objects().is_callable(object) {
                "function"
            } else {
                "object"
            }
        } else {
            "undefined"
        };
        let atom = agent.atoms_mut().intern(text);
        Value::from_string_ref(super::values::alloc_atom_string(agent, atom, text))
    }

    pub(super) fn find_global_environment(
        &self,
        agent: &Agent,
        start: EnvironmentRef,
    ) -> VmResult<GlobalEnvironmentRecord> {
        let mut current = Some(start);
        while let Some(environment) = current {
            match agent
                .environment(environment)
                .ok_or(VmError::MissingEnvironment(environment))?
            {
                EnvironmentRecord::Global(record) => return Ok(record),
                EnvironmentRecord::Declarative(record) => current = record.outer(),
                EnvironmentRecord::Private(record) => current = record.outer(),
                EnvironmentRecord::Function(record) => current = record.declarative().outer(),
                EnvironmentRecord::Module(record) => current = record.outer(),
                EnvironmentRecord::Object(record) => current = record.outer(),
            }
        }
        Err(VmError::MissingEnvironment(start))
    }

    fn lookup_global_property(
        &self,
        agent: &mut Agent,
        start: EnvironmentRef,
        name: AtomId,
    ) -> VmResult<Option<Value>> {
        let global = self.find_global_environment(agent, start)?;
        let key = PropertyKey::from_atom(name);
        if !object::has_property(agent, global.global_object(), key).map_err(VmError::Abrupt)? {
            return Ok(None);
        }
        object::get(agent, global.global_object(), key)
            .map(Some)
            .map_err(VmError::Abrupt)
    }

    fn get_global_property_binding_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        start: EnvironmentRef,
        name: AtomId,
    ) -> VmResult<Option<Value>> {
        let global = self.find_global_environment(agent, start)?;
        let key = PropertyKey::from_atom(name);
        if !object::has_property(agent, global.global_object(), key).map_err(VmError::Abrupt)? {
            return Ok(None);
        }
        self.get_property_from_value(
            agent,
            host,
            registry,
            frame,
            Value::from_object_ref(global.global_object()),
            key,
        )
        .map(Some)
    }

    fn lookup_global_layout_binding(
        &self,
        agent: &Agent,
        global: &GlobalEnvironmentRecord,
        name: AtomId,
    ) -> Option<GlobalLexicalBindingRecord> {
        let layout = agent.environment_layout(global.layout())?;
        let index = layout
            .bindings()
            .iter()
            .position(|binding| binding.name() == Some(name) && binding.flags().is_lexical())?;
        let index = u32::try_from(index).ok()?;
        Some(GlobalLexicalBindingRecord::new(name, global.id(), index))
    }

    pub(super) fn frame_is_strict(&self, frame: FrameRecord) -> bool {
        self.installed_function(frame.code())
            .map(|function| function.flags().strict())
            .unwrap_or(false)
    }

    pub(super) fn environment_at_depth(
        &self,
        agent: &Agent,
        start: EnvironmentRef,
        depth: u8,
    ) -> VmResult<EnvironmentRef> {
        let mut current = start;
        while matches!(
            agent.environment(current),
            Some(EnvironmentRecord::Object(_))
        ) {
            current = match agent
                .environment(current)
                .ok_or(VmError::MissingEnvironment(current))?
            {
                EnvironmentRecord::Object(record) => {
                    record.outer().ok_or(VmError::MissingEnvironment(current))?
                }
                _ => unreachable!("object-environment probe must only inspect object records"),
            };
        }

        let mut remaining = depth;
        while remaining > 0 {
            current = match agent
                .environment(current)
                .ok_or(VmError::MissingEnvironment(current))?
            {
                EnvironmentRecord::Declarative(record) => {
                    record.outer().ok_or(VmError::MissingEnvironment(current))?
                }
                EnvironmentRecord::Private(record) => {
                    record.outer().ok_or(VmError::MissingEnvironment(current))?
                }
                EnvironmentRecord::Function(record) => record
                    .declarative()
                    .outer()
                    .ok_or(VmError::MissingEnvironment(current))?,
                EnvironmentRecord::Module(record) => {
                    record.outer().ok_or(VmError::MissingEnvironment(current))?
                }
                EnvironmentRecord::Global(record) => {
                    record.outer().ok_or(VmError::MissingEnvironment(current))?
                }
                EnvironmentRecord::Object(record) => {
                    record.outer().ok_or(VmError::MissingEnvironment(current))?
                }
            };
            while matches!(
                agent.environment(current),
                Some(EnvironmentRecord::Object(_))
            ) {
                current = match agent
                    .environment(current)
                    .ok_or(VmError::MissingEnvironment(current))?
                {
                    EnvironmentRecord::Object(record) => {
                        record.outer().ok_or(VmError::MissingEnvironment(current))?
                    }
                    _ => unreachable!("object-environment probe must only inspect object records"),
                };
            }
            remaining -= 1;
        }
        Ok(current)
    }
}
