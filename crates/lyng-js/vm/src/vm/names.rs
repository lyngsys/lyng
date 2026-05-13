use super::{
    ActiveEnvScopeRange, Agent, AtomId, CodeRef, EnvironmentRef, FrameRecord, ObjectRef, Value, Vm,
    VmError, VmResult, WellKnownSymbolId,
};
use crate::name_refs::{CapturedNameReference, CapturedNameTarget};
#[cfg(test)]
use crate::vm::call::RejectingNativeRegistry;
use crate::vm::property_access::VmProxyBridge;
use lyng_js_env::{
    EnvironmentRecord, GlobalEnvironmentRecord, GlobalLexicalBindingRecord, ObjectEnvironmentRecord,
};
use lyng_js_host::HostHooks;
#[cfg(test)]
use lyng_js_host::NoopHostHooks;
use lyng_js_objects::{NativeFunctionRegistry, ObjectKind};
use lyng_js_ops::{errors, object, proxy, read};
use lyng_js_types::{FeedbackSlotId, PropertyKey};

impl Vm {
    fn layout_binding_slot(
        &self,
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
            if binding.flags().is_scoped() && !self.active_env_scope_contains(environment, index) {
                continue;
            }
            let value = agent.environment_slot(environment, index)?;
            if binding.flags().is_dynamic() && value == Value::deleted_environment_binding() {
                continue;
            }
            return Some(index);
        }
        None
    }

    fn active_env_scope_contains(&self, environment: EnvironmentRef, slot: u32) -> bool {
        let frame_depth = self.frames.len();
        if self
            .active_env_scopes
            .iter()
            .rev()
            .any(|range| range.frame_depth == frame_depth && range.contains(environment, slot))
        {
            return true;
        }
        self.loop_iteration_envs
            .iter()
            .rev()
            .filter(|record| {
                record.active
                    && record.frame_depth == frame_depth
                    && record.iteration_environment == environment
            })
            .any(|record| {
                self.active_env_scopes.iter().rev().any(|range| {
                    range.frame_depth == frame_depth
                        && range.contains(record.source_environment, slot)
                })
            })
    }

    pub(super) fn enter_env_scope(
        &mut self,
        agent: &Agent,
        frame: &FrameRecord,
        base: u16,
        count: u32,
    ) -> VmResult<()> {
        let environment =
            self.environment_for_slot_access(agent, frame.lexical_env(), 0, u32::from(base))?;
        self.active_env_scopes.push(ActiveEnvScopeRange::new(
            self.frames.len(),
            environment,
            u32::from(base),
            count,
        ));
        Ok(())
    }

    pub(super) fn leave_env_scope(&mut self, frame: &FrameRecord, base: u16, count: u32) {
        let start = u32::from(base);
        let end = start.saturating_add(count);
        let frame_depth = self.frames.len();
        if let Some(index) = self.active_env_scopes.iter().rposition(|range| {
            range.frame_depth == frame_depth
                && range.start == start
                && range.end == end
                && range.environment == frame.lexical_env()
        }) {
            let _ = self.active_env_scopes.remove(index);
        }
    }

    pub(super) fn close_env_scope_frames(&mut self, frame_depth: usize) {
        self.active_env_scopes
            .retain(|range| range.frame_depth <= frame_depth);
    }

    pub(super) fn drain_env_scope_state(&mut self, frame_depth: usize) -> Vec<ActiveEnvScopeRange> {
        let mut states = Vec::new();
        let mut index = 0;
        while index < self.active_env_scopes.len() {
            if self.active_env_scopes[index].frame_depth == frame_depth {
                states.push(self.active_env_scopes.remove(index));
            } else {
                index += 1;
            }
        }
        states
    }

    pub(super) fn restore_env_scope_state(
        &mut self,
        frame_depth: usize,
        mut states: Vec<ActiveEnvScopeRange>,
    ) {
        for state in &mut states {
            state.frame_depth = frame_depth;
        }
        self.active_env_scopes.extend(states);
    }

    fn object_environment_has_binding_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
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
            object::has_property_in_context(&mut bridge, binding_object, key)?
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
        Ok(!read::to_boolean_agent(agent, blocked).map_err(VmError::Abrupt)?)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn object_environment_get_binding_value_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
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
            object::has_property_in_context(&mut bridge, binding_object, key)?
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

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn object_environment_set_mutable_binding_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
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
            object::has_property_in_context(&mut bridge, binding_object, key)?
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
        frame: &FrameRecord,
        record: ObjectEnvironmentRecord,
        name: AtomId,
    ) -> VmResult<bool> {
        let mut bridge = VmProxyBridge {
            vm: self,
            agent,
            host,
            registry,
            frame: frame,
        };
        proxy::delete_property(
            &mut bridge,
            record.binding_object(),
            PropertyKey::from_atom(name),
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn probe_identifier_value_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        start: EnvironmentRef,
        name: AtomId,
        strict: bool,
    ) -> VmResult<Option<Value>> {
        let mut current = Some(start);
        let mut consumed_overlay_source = None;
        while let Some(environment) = current {
            if consumed_overlay_source != Some(environment)
                && let Some(overlay) = self.direct_eval_environment_overlay(environment)
            {
                consumed_overlay_source = Some(environment);
                current = Some(overlay);
                continue;
            }

            let record = agent
                .environment(environment)
                .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
            match record {
                EnvironmentRecord::Declarative(record) => {
                    if let Some(slot) =
                        self.layout_binding_slot(agent, record.id(), record.layout(), name)
                    {
                        let environment =
                            self.environment_for_slot_access(agent, record.id(), 0, slot)?;
                        return Self::read_environment_slot(agent, environment, slot).map(Some);
                    }
                    current = record.outer();
                }
                EnvironmentRecord::Private(record) => current = record.outer(),
                EnvironmentRecord::Function(record) => {
                    let declarative = record.declarative();
                    if let Some(slot) = self.layout_binding_slot(
                        agent,
                        declarative.id(),
                        declarative.layout(),
                        name,
                    ) {
                        let environment =
                            self.environment_for_slot_access(agent, declarative.id(), 0, slot)?;
                        return Self::read_environment_slot(agent, environment, slot).map(Some);
                    }
                    current = declarative.outer();
                }
                EnvironmentRecord::Module(record) => {
                    if let Some(slot) =
                        self.layout_binding_slot(agent, record.id(), record.layout(), name)
                    {
                        let environment =
                            self.environment_for_slot_access(agent, record.id(), 0, slot)?;
                        return Self::read_environment_slot(agent, environment, slot).map(Some);
                    }
                    current = record.outer();
                }
                EnvironmentRecord::Global(record) => {
                    if let Some(binding) = Self::lookup_global_lexical_binding(agent, &record, name)
                    {
                        let environment = self.environment_for_slot_access(
                            agent,
                            binding.environment(),
                            0,
                            binding.slot(),
                        )?;
                        return Self::read_environment_slot(agent, environment, binding.slot())
                            .map(Some);
                    }
                    if let Some(slot) =
                        self.layout_binding_slot(agent, record.id(), record.layout(), name)
                    {
                        let environment =
                            self.environment_for_slot_access(agent, record.id(), 0, slot)?;
                        return Self::read_environment_slot(agent, environment, slot).map(Some);
                    }
                    if let Some(value) = self.get_global_property_binding_with_context(
                        agent,
                        host,
                        registry,
                        frame,
                        record.id(),
                        name,
                    )? {
                        return Ok(Some(value));
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
        frame: &FrameRecord,
        start: EnvironmentRef,
        name: AtomId,
    ) -> VmResult<CapturedNameTarget> {
        let mut current = Some(start);
        let mut consumed_overlay_source = None;
        while let Some(environment) = current {
            if consumed_overlay_source != Some(environment)
                && let Some(overlay) = self.direct_eval_environment_overlay(environment)
            {
                consumed_overlay_source = Some(environment);
                current = Some(overlay);
                continue;
            }

            let Some(record) = agent.environment(environment) else {
                break;
            };
            match record {
                EnvironmentRecord::Declarative(record) => {
                    if let Some(slot) =
                        self.layout_binding_slot(agent, record.id(), record.layout(), name)
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
                    if let Some(slot) = self.layout_binding_slot(
                        agent,
                        declarative.id(),
                        declarative.layout(),
                        name,
                    ) {
                        let environment =
                            self.environment_for_slot_access(agent, declarative.id(), 0, slot)?;
                        return Ok(CapturedNameTarget::EnvironmentSlot { environment, slot });
                    }
                    current = declarative.outer();
                }
                EnvironmentRecord::Module(record) => {
                    if let Some(slot) =
                        self.layout_binding_slot(agent, record.id(), record.layout(), name)
                    {
                        let environment =
                            self.environment_for_slot_access(agent, record.id(), 0, slot)?;
                        return Ok(CapturedNameTarget::EnvironmentSlot { environment, slot });
                    }
                    current = record.outer();
                }
                EnvironmentRecord::Global(record) => {
                    if let Some(binding) = Self::lookup_global_lexical_binding(agent, &record, name)
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
                    let has_global_property = self.global_has_property_with_context(
                        agent,
                        host,
                        registry,
                        frame,
                        record.global_object(),
                        name,
                    )?;
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
        let global = Self::find_global_environment(agent, frame.variable_env())?;
        Ok(CapturedNameTarget::Unresolvable {
            global_environment: global.id(),
        })
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn load_global_with_feedback(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        name: AtomId,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
    ) -> VmResult<Value> {
        let global = Self::find_global_environment_ref(agent, frame.variable_env())?;
        if let Some(binding) = Self::lookup_global_lexical_binding_ref(agent, global, name) {
            let environment =
                self.environment_for_slot_access(agent, binding.environment(), 0, binding.slot())?;
            return Self::read_environment_slot(agent, environment, binding.slot());
        }

        let global_object = agent
            .global_environment_object(global)
            .ok_or(VmError::MissingEnvironment(global))?;
        if let Some(value) =
            self.try_named_property_load_inline_cache_hit(agent, code, feedback_slot, global_object)
        {
            return Ok(value);
        }

        let value = self
            .get_global_property_binding_with_context(
                agent,
                host,
                registry,
                frame,
                frame.variable_env(),
                name,
            )?
            .ok_or_else(|| VmError::Abrupt(errors::throw_reference_error(agent)))?;
        self.observe_named_property_slow_path(
            agent,
            code,
            feedback_slot,
            global_object,
            name,
            lyng_js_objects::NamedPropertyCachePurpose::Load,
        );
        Ok(value)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn store_global_with_feedback(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        name: AtomId,
        value: Value,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
    ) -> VmResult<()> {
        let global = Self::find_global_environment(agent, frame.variable_env())?;
        if let Some(binding) = Self::lookup_global_lexical_binding(agent, &global, name) {
            let environment =
                self.environment_for_slot_access(agent, binding.environment(), 0, binding.slot())?;
            return self.write_environment_slot(agent, environment, binding.slot(), value);
        }
        let global_object = global.global_object();
        if self
            .try_named_property_store_inline_cache(agent, code, feedback_slot, global_object, value)
            .is_some()
        {
            self.record_feedback_slot(code, feedback_slot);
            return Ok(());
        }
        let _ = self.set_global_property_with_context(
            agent,
            host,
            registry,
            frame,
            global_object,
            name,
            value,
        )?;
        self.observe_named_property_slow_path(
            agent,
            code,
            feedback_slot,
            global_object,
            name,
            lyng_js_objects::NamedPropertyCachePurpose::Store,
        );
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn assign_global_with_feedback(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        name: AtomId,
        value: Value,
        code: CodeRef,
        feedback_slot: Option<FeedbackSlotId>,
    ) -> VmResult<()> {
        let global = Self::find_global_environment(agent, frame.variable_env())?;
        if let Some(binding) = Self::lookup_global_lexical_binding(agent, &global, name) {
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
        let global_object = global.global_object();
        if let Some(stored) = self.try_named_property_store_inline_cache(
            agent,
            code,
            feedback_slot,
            global_object,
            value,
        ) {
            if !stored && self.frame_is_strict(frame) {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            self.record_feedback_slot(code, feedback_slot);
            return Ok(());
        }

        let has_property =
            self.global_has_property_with_key(agent, host, registry, frame, global_object, key)?;
        if !has_property && self.frame_is_strict(frame) {
            return Err(VmError::Abrupt(errors::throw_reference_error(agent)));
        }

        let stored = self.set_global_property_with_key(
            agent,
            host,
            registry,
            frame,
            global_object,
            key,
            value,
        )?;
        if !stored && self.frame_is_strict(frame) {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        self.observe_named_property_slow_path(
            agent,
            code,
            feedback_slot,
            global_object,
            name,
            lyng_js_objects::NamedPropertyCachePurpose::Store,
        );
        Ok(())
    }

    fn capture_name_reference_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        name: AtomId,
    ) -> VmResult<CapturedNameReference> {
        let lexical_env = self.dynamic_name_start_environment(agent, frame);
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

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn load_global_property_binding(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
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

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn assign_global_property_binding(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        environment: EnvironmentRef,
        name: AtomId,
        value: Value,
        strict: bool,
    ) -> VmResult<()> {
        let global = Self::find_global_environment(agent, environment)?;
        let key = PropertyKey::from_atom(name);
        let has_property = self.global_has_property_with_key(
            agent,
            host,
            registry,
            frame,
            global.global_object(),
            key,
        )?;
        if !has_property && strict {
            return Err(VmError::Abrupt(errors::throw_reference_error(agent)));
        }
        let stored = self.set_global_property_with_key(
            agent,
            host,
            registry,
            frame,
            global.global_object(),
            key,
            value,
        )?;
        if !stored && strict {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn assign_unresolvable_name(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        global_environment: EnvironmentRef,
        name: AtomId,
        value: Value,
        strict: bool,
    ) -> VmResult<()> {
        if strict {
            return Err(VmError::Abrupt(errors::throw_reference_error(agent)));
        }
        let global = Self::find_global_environment(agent, global_environment)?;
        let _ = self.set_global_property_with_context(
            agent,
            host,
            registry,
            frame,
            global.global_object(),
            name,
            value,
        )?;
        Ok(())
    }

    pub(super) fn load_captured_name_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        reference_register: u16,
    ) -> VmResult<Value> {
        let reference = self
            .captured_name_references
            .get(frame.registers().base(), reference_register)
            .ok_or_else(|| VmError::RegisterOutOfBounds {
                code: frame.code(),
                register: reference_register,
            })?;
        match reference.target() {
            CapturedNameTarget::EnvironmentSlot { environment, slot } => {
                Self::read_environment_slot(agent, environment, slot)
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

    pub(super) fn load_captured_name_this_with_context(
        &self,
        frame: &FrameRecord,
        reference_register: u16,
    ) -> VmResult<Value> {
        let reference = self
            .captured_name_references
            .get(frame.registers().base(), reference_register)
            .ok_or_else(|| VmError::RegisterOutOfBounds {
                code: frame.code(),
                register: reference_register,
            })?;
        match reference.target() {
            CapturedNameTarget::ObjectProperty { record } if record.with_environment() => {
                Ok(Value::from_object_ref(record.binding_object()))
            }
            CapturedNameTarget::EnvironmentSlot { .. }
            | CapturedNameTarget::ObjectProperty { .. }
            | CapturedNameTarget::GlobalProperty { .. }
            | CapturedNameTarget::Unresolvable { .. } => Ok(Value::undefined()),
        }
    }

    pub(super) fn assign_captured_name_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        reference_register: u16,
        value: Value,
    ) -> VmResult<()> {
        let reference = self
            .captured_name_references
            .get(frame.registers().base(), reference_register)
            .ok_or_else(|| VmError::RegisterOutOfBounds {
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
                    host,
                    registry,
                    frame,
                    environment,
                    reference.name(),
                    value,
                    self.frame_is_strict(frame),
                ),
            CapturedNameTarget::Unresolvable { global_environment } => self
                .assign_unresolvable_name(
                    agent,
                    host,
                    registry,
                    frame,
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
        frame: &FrameRecord,
        name: AtomId,
    ) -> VmResult<Value> {
        let lexical_env = self.dynamic_name_start_environment(agent, frame);
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
            .get_global_property_binding_with_context(
                agent,
                host,
                registry,
                frame,
                frame.variable_env(),
                name,
            )?
            .unwrap_or_else(Value::undefined))
    }

    pub(super) fn resolve_global(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        name: AtomId,
    ) -> VmResult<Value> {
        Ok(self
            .get_global_property_binding_with_context(
                agent,
                host,
                registry,
                frame,
                frame.variable_env(),
                name,
            )?
            .unwrap_or_else(Value::undefined))
    }

    pub(super) fn load_name_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        name: AtomId,
    ) -> VmResult<Value> {
        let lexical_env = self.dynamic_name_start_environment(agent, frame);
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

    pub(super) fn capture_name_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
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
        frame: &FrameRecord,
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
                    host,
                    registry,
                    frame,
                    environment,
                    name,
                    value,
                    self.frame_is_strict(frame),
                ),
            CapturedNameTarget::Unresolvable { global_environment } => self
                .assign_unresolvable_name(
                    agent,
                    host,
                    registry,
                    frame,
                    global_environment,
                    name,
                    value,
                    self.frame_is_strict(frame),
                ),
        }
    }

    pub(super) fn assign_variable_name_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        name: AtomId,
        value: Value,
    ) -> VmResult<()> {
        let target = self.resolve_dynamic_name_target_with_context(
            agent,
            host,
            registry,
            frame,
            frame.variable_env(),
            name,
        )?;
        match target {
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
                    host,
                    registry,
                    frame,
                    environment,
                    name,
                    value,
                    self.frame_is_strict(frame),
                ),
            CapturedNameTarget::Unresolvable { global_environment } => self
                .assign_unresolvable_name(
                    agent,
                    host,
                    registry,
                    frame,
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
        frame: &FrameRecord,
        name: AtomId,
    ) -> VmResult<bool> {
        let lexical_env = self.dynamic_name_start_environment(agent, frame);
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
            CapturedNameTarget::GlobalProperty { .. } => Self::delete_global(agent, frame, name),
            CapturedNameTarget::Unresolvable { .. } => Ok(true),
        }
    }

    #[cfg(test)]
    pub(super) fn load_name(
        &mut self,
        agent: &mut Agent,
        frame: &FrameRecord,
        name: AtomId,
    ) -> VmResult<Value> {
        let host = NoopHostHooks;
        let mut registry = RejectingNativeRegistry;
        self.load_name_with_context(agent, &host, &mut registry, frame, name)
    }

    pub(super) fn delete_global(
        agent: &mut Agent,
        frame: &FrameRecord,
        name: AtomId,
    ) -> VmResult<bool> {
        let global = Self::find_global_environment(agent, frame.variable_env())?;
        if Self::global_chain_has_lexical_binding(agent, global.id(), name)
            || Self::global_chain_has_var_name(agent, global.id(), name)
        {
            return Ok(false);
        }
        object::ordinary_delete_property(
            agent,
            global.global_object(),
            PropertyKey::from_atom(name),
        )
        .map_err(VmError::Abrupt)
    }

    pub(super) fn global_has_lexical_binding(
        agent: &Agent,
        global: &GlobalEnvironmentRecord,
        name: AtomId,
    ) -> bool {
        global.has_lexical_name(name)
            || agent
                .environment_layout(global.layout())
                .is_some_and(|layout| {
                    layout
                        .bindings()
                        .iter()
                        .any(|binding| binding.name() == Some(name))
                })
    }

    pub(super) fn lookup_global_lexical_binding(
        agent: &Agent,
        global: &GlobalEnvironmentRecord,
        name: AtomId,
    ) -> Option<GlobalLexicalBindingRecord> {
        global
            .lexical_binding(name)
            .or_else(|| Self::lookup_global_layout_binding(agent, global, name))
    }

    pub(super) fn global_chain_has_lexical_binding(
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
                    if Self::global_has_lexical_binding(agent, &global, name) {
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

    pub(super) fn type_of_value(agent: &mut Agent, value: Value) -> Value {
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
            if agent.objects().is_html_dda_object(object) {
                "undefined"
            } else if agent.objects().is_callable(object) {
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

    fn global_has_property_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        global_object: ObjectRef,
        name: AtomId,
    ) -> VmResult<bool> {
        self.global_has_property_with_key(
            agent,
            host,
            registry,
            frame,
            global_object,
            PropertyKey::from_atom(name),
        )
    }

    fn global_has_property_with_key(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        global_object: ObjectRef,
        key: PropertyKey,
    ) -> VmResult<bool> {
        object::has_property_in_context(
            &mut VmProxyBridge {
                vm: self,
                agent,
                host,
                registry,
                frame,
            },
            global_object,
            key,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn set_global_property_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        global_object: ObjectRef,
        name: AtomId,
        value: Value,
    ) -> VmResult<bool> {
        self.set_global_property_with_key(
            agent,
            host,
            registry,
            frame,
            global_object,
            PropertyKey::from_atom(name),
            value,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn set_global_property_with_key(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        global_object: ObjectRef,
        key: PropertyKey,
        value: Value,
    ) -> VmResult<bool> {
        self.set_property_on_object(
            agent,
            host,
            registry,
            frame,
            global_object,
            Value::from_object_ref(global_object),
            key,
            value,
        )
    }

    fn get_global_property_binding_with_context(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        start: EnvironmentRef,
        name: AtomId,
    ) -> VmResult<Option<Value>> {
        let (_, global_object) = Self::find_global_environment_object(agent, start)?;
        let key = PropertyKey::from_atom(name);
        if let Some(value) = Self::try_get_global_own_data_property(agent, global_object, key)? {
            return Ok(Some(value));
        }
        if !self.global_has_property_with_key(agent, host, registry, frame, global_object, key)? {
            return Ok(None);
        }
        self.get_property_from_object(
            agent,
            host,
            registry,
            frame,
            global_object,
            Value::from_object_ref(global_object),
            key,
        )
        .map(Some)
    }

    fn try_get_global_own_data_property(
        agent: &mut Agent,
        global_object: ObjectRef,
        key: PropertyKey,
    ) -> VmResult<Option<Value>> {
        let Some(header) = agent
            .objects()
            .object_header(agent.heap().view(), global_object)
        else {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        };
        if header.kind() == ObjectKind::Proxy {
            return Ok(None);
        }

        let descriptor = agent
            .objects()
            .get_own_property(agent.heap().view(), global_object, key)
            .map_err(|_| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let Some(descriptor) = descriptor else {
            return Ok(None);
        };
        if descriptor.has_value() && !descriptor.has_get() && !descriptor.has_set() {
            return Ok(descriptor.value());
        }
        Ok(None)
    }

    fn find_global_environment_ref(
        agent: &Agent,
        start: EnvironmentRef,
    ) -> VmResult<EnvironmentRef> {
        let mut current = start;
        loop {
            if agent.environment_is_global(current) {
                return Ok(current);
            }
            current = agent
                .environment_outer(current)
                .ok_or(VmError::MissingEnvironment(current))?
                .ok_or(VmError::MissingEnvironment(current))?;
        }
    }

    fn find_global_environment_object(
        agent: &Agent,
        start: EnvironmentRef,
    ) -> VmResult<(EnvironmentRef, ObjectRef)> {
        let global = Self::find_global_environment_ref(agent, start)?;
        let object = agent
            .global_environment_object(global)
            .ok_or(VmError::MissingEnvironment(global))?;
        Ok((global, object))
    }

    fn lookup_global_lexical_binding_ref(
        agent: &Agent,
        global: EnvironmentRef,
        name: AtomId,
    ) -> Option<GlobalLexicalBindingRecord> {
        agent
            .global_lexical_binding(global, name)
            .or_else(|| Self::lookup_global_layout_binding_ref(agent, global, name))
    }

    fn lookup_global_layout_binding_ref(
        agent: &Agent,
        global: EnvironmentRef,
        name: AtomId,
    ) -> Option<GlobalLexicalBindingRecord> {
        let layout = agent.environment_layout(agent.global_environment_layout(global)?)?;
        let index = layout
            .bindings()
            .iter()
            .position(|binding| binding.name() == Some(name) && binding.flags().is_lexical())?;
        let index = u32::try_from(index).ok()?;
        Some(GlobalLexicalBindingRecord::new(name, global, index))
    }

    fn lookup_global_layout_binding(
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

    pub(super) fn frame_is_strict(&self, frame: &FrameRecord) -> bool {
        self.installed_function(frame.code())
            .is_some_and(|function| function.flags().strict())
    }

    pub(super) fn environment_at_depth(
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
