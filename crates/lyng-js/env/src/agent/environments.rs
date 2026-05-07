use super::Agent;
use crate::{
    environment_index, layout_index, DeclarativeEnvironmentRecord, EnvironmentLayout,
    EnvironmentLayoutId, EnvironmentLayoutKind, EnvironmentMetadata, EnvironmentRecord,
    EnvironmentRef, EnvironmentSlotsRef, FunctionEnvironmentRecord, GlobalEnvironmentRecord,
    GlobalLexicalBindingRecord, ModuleBindingAlias, ModuleEnvironmentRecord,
    ObjectEnvironmentRecord, ObjectHandleStoreTarget, PrivateEnvironmentRecord,
    RuntimeEnvironmentRecord, ThisBindingStatus, ValueStoreTarget,
};
use lyng_js_common::AtomId;
use lyng_js_gc::AllocationLifetime;
use lyng_js_types::{ObjectRef, Value};
use std::collections::HashSet;

impl Agent {
    /// Allocates one immutable environment layout record.
    ///
    /// # Panics
    /// Panics if the layout table grows beyond the `u32` id range.
    pub fn alloc_environment_layout(&mut self, layout: EnvironmentLayout) -> EnvironmentLayoutId {
        let raw_id = u32::try_from(self.environment_layouts.len() + 1)
            .expect("environment layout id must fit into u32");
        let id = EnvironmentLayoutId::from_raw(raw_id)
            .expect("environment layout id must stay non-zero");
        self.environment_layouts.push(Some(layout));
        id
    }

    pub fn environment_layout(&self, id: EnvironmentLayoutId) -> Option<&EnvironmentLayout> {
        self.environment_layouts.get(layout_index(id))?.as_ref()
    }

    pub fn alloc_declarative_environment(
        &mut self,
        outer: Option<EnvironmentRef>,
        layout: EnvironmentLayoutId,
        lifetime: AllocationLifetime,
    ) -> Option<EnvironmentRef> {
        if self.environment_layout(layout)?.kind() != EnvironmentLayoutKind::Declarative {
            return None;
        }
        Some(self.alloc_environment_record(
            outer,
            Some(layout),
            None,
            Value::undefined(),
            None,
            None,
            EnvironmentMetadata::Declarative { layout },
            lifetime,
        ))
    }

    pub fn alloc_private_environment(
        &mut self,
        outer: Option<EnvironmentRef>,
        layout: EnvironmentLayoutId,
        lifetime: AllocationLifetime,
    ) -> Option<EnvironmentRef> {
        if self.environment_layout(layout)?.kind() != EnvironmentLayoutKind::Private {
            return None;
        }
        Some(self.alloc_environment_record(
            outer,
            Some(layout),
            None,
            Value::undefined(),
            None,
            None,
            EnvironmentMetadata::Private { layout },
            lifetime,
        ))
    }

    pub fn alloc_module_environment(
        &mut self,
        outer: Option<EnvironmentRef>,
        layout: EnvironmentLayoutId,
        lifetime: AllocationLifetime,
    ) -> Option<EnvironmentRef> {
        let layout_record = self.environment_layout(layout)?;
        if layout_record.kind() != EnvironmentLayoutKind::Module {
            return None;
        }
        let import_aliases = vec![None; usize::try_from(layout_record.slot_count()).ok()?];
        Some(self.alloc_environment_record(
            outer,
            Some(layout),
            None,
            Value::undefined(),
            None,
            None,
            EnvironmentMetadata::Module {
                layout,
                import_aliases,
            },
            lifetime,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn alloc_function_environment(
        &mut self,
        outer: Option<EnvironmentRef>,
        layout: EnvironmentLayoutId,
        function_object: ObjectRef,
        this_binding_status: ThisBindingStatus,
        this_value: Value,
        new_target: Option<ObjectRef>,
        home_object: Option<ObjectRef>,
        lifetime: AllocationLifetime,
    ) -> Option<EnvironmentRef> {
        if self.environment_layout(layout)?.kind() != EnvironmentLayoutKind::Function {
            return None;
        }
        Some(self.alloc_environment_record(
            outer,
            Some(layout),
            Some(function_object),
            this_value,
            new_target,
            home_object,
            EnvironmentMetadata::Function {
                layout,
                this_binding_status,
            },
            lifetime,
        ))
    }

    pub fn alloc_global_environment(
        &mut self,
        outer: Option<EnvironmentRef>,
        layout: EnvironmentLayoutId,
        global_object: ObjectRef,
        lifetime: AllocationLifetime,
    ) -> Option<EnvironmentRef> {
        if self.environment_layout(layout)?.kind() != EnvironmentLayoutKind::Global {
            return None;
        }
        Some(self.alloc_environment_record(
            outer,
            Some(layout),
            Some(global_object),
            Value::undefined(),
            None,
            None,
            EnvironmentMetadata::Global {
                layout,
                lexical_names: HashSet::new(),
                lexical_bindings: Vec::new(),
                var_names: HashSet::new(),
            },
            lifetime,
        ))
    }

    /// Allocates one object environment record.
    pub fn alloc_object_environment(
        &mut self,
        outer: Option<EnvironmentRef>,
        binding_object: ObjectRef,
        with_environment: bool,
        lifetime: AllocationLifetime,
    ) -> EnvironmentRef {
        self.alloc_environment_record(
            outer,
            None,
            Some(binding_object),
            Value::undefined(),
            None,
            None,
            EnvironmentMetadata::Object { with_environment },
            lifetime,
        )
    }

    pub fn environment(&self, id: EnvironmentRef) -> Option<EnvironmentRecord> {
        let record = self.heap.view().environment(id)?;
        let metadata = self.environment_metadata(id)?;
        match metadata {
            EnvironmentMetadata::Declarative { layout } => Some(EnvironmentRecord::Declarative(
                DeclarativeEnvironmentRecord {
                    id,
                    outer: record.outer(),
                    layout: *layout,
                    slots: record.slots(),
                },
            )),
            EnvironmentMetadata::Private { layout } => {
                Some(EnvironmentRecord::Private(PrivateEnvironmentRecord {
                    id,
                    outer: record.outer(),
                    layout: *layout,
                    slots: record.slots(),
                }))
            }
            EnvironmentMetadata::Function {
                layout,
                this_binding_status,
            } => Some(EnvironmentRecord::Function(FunctionEnvironmentRecord {
                declarative: DeclarativeEnvironmentRecord {
                    id,
                    outer: record.outer(),
                    layout: *layout,
                    slots: record.slots(),
                },
                function_object: record.function_object()?,
                this_binding_status: *this_binding_status,
                this_value: record.this_value(),
                new_target: record.new_target(),
                home_object: record.home_object(),
            })),
            EnvironmentMetadata::Module { layout, .. } => {
                Some(EnvironmentRecord::Module(ModuleEnvironmentRecord {
                    declarative: DeclarativeEnvironmentRecord {
                        id,
                        outer: record.outer(),
                        layout: *layout,
                        slots: record.slots(),
                    },
                }))
            }
            EnvironmentMetadata::Global {
                layout,
                lexical_names,
                lexical_bindings,
                var_names,
            } => Some(EnvironmentRecord::Global(GlobalEnvironmentRecord {
                id,
                outer: record.outer(),
                layout: *layout,
                lexical_slots: record.slots(),
                global_object: record.function_object()?,
                lexical_names: lexical_names.clone(),
                lexical_bindings: lexical_bindings.clone(),
                var_names: var_names.clone(),
            })),
            EnvironmentMetadata::Object { with_environment } => {
                Some(EnvironmentRecord::Object(ObjectEnvironmentRecord {
                    id,
                    outer: record.outer(),
                    binding_object: record.function_object()?,
                    with_environment: *with_environment,
                }))
            }
        }
    }

    pub fn private_environment(&self, id: EnvironmentRef) -> Option<PrivateEnvironmentRecord> {
        match self.environment(id)? {
            EnvironmentRecord::Private(record) => Some(record),
            _ => None,
        }
    }

    pub fn declarative_environment(
        &self,
        id: EnvironmentRef,
    ) -> Option<DeclarativeEnvironmentRecord> {
        match self.environment(id)? {
            EnvironmentRecord::Declarative(record) => Some(record),
            _ => None,
        }
    }

    pub fn function_environment(&self, id: EnvironmentRef) -> Option<FunctionEnvironmentRecord> {
        match self.environment(id)? {
            EnvironmentRecord::Function(record) => Some(record),
            _ => None,
        }
    }

    pub fn module_environment(&self, id: EnvironmentRef) -> Option<ModuleEnvironmentRecord> {
        match self.environment(id)? {
            EnvironmentRecord::Module(record) => Some(record),
            _ => None,
        }
    }

    pub fn global_environment(&self, id: EnvironmentRef) -> Option<GlobalEnvironmentRecord> {
        match self.environment(id)? {
            EnvironmentRecord::Global(record) => Some(record),
            _ => None,
        }
    }

    pub fn object_environment(&self, id: EnvironmentRef) -> Option<ObjectEnvironmentRecord> {
        match self.environment(id)? {
            EnvironmentRecord::Object(record) => Some(record),
            _ => None,
        }
    }

    pub fn environment_outer(&self, id: EnvironmentRef) -> Option<Option<EnvironmentRef>> {
        self.heap
            .view()
            .environment(id)
            .map(RuntimeEnvironmentRecord::outer)
    }

    pub fn environment_is_global(&self, id: EnvironmentRef) -> bool {
        matches!(
            self.environment_metadata(id),
            Some(EnvironmentMetadata::Global { .. })
        )
    }

    pub fn global_environment_object(&self, id: EnvironmentRef) -> Option<ObjectRef> {
        if !self.environment_is_global(id) {
            return None;
        }
        self.heap
            .view()
            .environment(id)
            .and_then(RuntimeEnvironmentRecord::function_object)
    }

    pub fn global_environment_layout(&self, id: EnvironmentRef) -> Option<EnvironmentLayoutId> {
        match self.environment_metadata(id) {
            Some(EnvironmentMetadata::Global { layout, .. }) => Some(*layout),
            _ => None,
        }
    }

    pub fn environment_slots(&self, id: EnvironmentRef) -> Option<&[Value]> {
        let slots = self.heap.view().environment(id)?.slots()?;
        self.heap.view().environment_slots(slots)
    }

    pub fn module_binding_alias(
        &self,
        id: EnvironmentRef,
        slot: u32,
    ) -> Option<ModuleBindingAlias> {
        let EnvironmentMetadata::Module { import_aliases, .. } = self.environment_metadata(id)?
        else {
            return None;
        };
        import_aliases.get(slot as usize).copied().flatten()
    }

    pub fn set_module_binding_alias(
        &mut self,
        id: EnvironmentRef,
        slot: u32,
        alias: Option<ModuleBindingAlias>,
    ) -> bool {
        let Some(EnvironmentMetadata::Module { import_aliases, .. }) =
            self.environment_metadata_mut(id)
        else {
            return false;
        };
        let Some(target) = import_aliases.get_mut(slot as usize) else {
            return false;
        };
        *target = alias;
        true
    }

    fn resolved_environment_slot_target(
        &self,
        id: EnvironmentRef,
        index: u32,
    ) -> Option<(EnvironmentRef, u32)> {
        let mut environment = id;
        let mut slot = index;
        let mut traversed = 0usize;
        loop {
            if traversed >= self.environment_metadata.len().max(1) {
                return None;
            }
            let Some(alias) = self.module_binding_alias(environment, slot) else {
                return Some((environment, slot));
            };
            environment = alias.environment();
            slot = alias.slot();
            traversed = traversed.saturating_add(1);
        }
    }

    pub fn environment_slot(&self, id: EnvironmentRef, index: u32) -> Option<Value> {
        let (id, index) = self.resolved_environment_slot_target(id, index)?;
        self.environment_slots(id)?.get(index as usize).copied()
    }

    pub fn init_environment_slot(&mut self, id: EnvironmentRef, index: u32, value: Value) -> bool {
        let Some((id, index)) = self.resolved_environment_slot_target(id, index) else {
            return false;
        };
        let Some(slots) = self
            .heap
            .view()
            .environment(id)
            .and_then(RuntimeEnvironmentRecord::slots)
        else {
            return false;
        };
        self.heap
            .mutator()
            .init_store_value(ValueStoreTarget::EnvironmentSlot(slots, index), value)
    }

    pub fn set_environment_slot(&mut self, id: EnvironmentRef, index: u32, value: Value) -> bool {
        let Some((id, index)) = self.resolved_environment_slot_target(id, index) else {
            return false;
        };
        let Some(slots) = self
            .heap
            .view()
            .environment(id)
            .and_then(RuntimeEnvironmentRecord::slots)
        else {
            return false;
        };
        self.heap
            .mutator()
            .mut_store_value(ValueStoreTarget::EnvironmentSlot(slots, index), value)
    }

    pub fn set_function_this_binding(
        &mut self,
        id: EnvironmentRef,
        status: ThisBindingStatus,
        value: Value,
    ) -> bool {
        let Some(EnvironmentMetadata::Function {
            this_binding_status,
            ..
        }) = self.environment_metadata_mut(id)
        else {
            return false;
        };
        *this_binding_status = status;
        let stored_value = match status {
            ThisBindingStatus::Initialized => value,
            ThisBindingStatus::Lexical | ThisBindingStatus::Uninitialized => Value::undefined(),
        };
        self.heap
            .mutator()
            .mut_store_value(ValueStoreTarget::EnvironmentThisValue(id), stored_value)
    }

    pub fn set_function_new_target(
        &mut self,
        id: EnvironmentRef,
        new_target: Option<ObjectRef>,
    ) -> bool {
        matches!(
            self.environment_metadata(id),
            Some(EnvironmentMetadata::Function { .. })
        ) && self.heap.mutator().mut_store_object_handle(
            ObjectHandleStoreTarget::EnvironmentNewTarget(id),
            new_target,
        )
    }

    pub fn set_function_home_object(
        &mut self,
        id: EnvironmentRef,
        home_object: Option<ObjectRef>,
    ) -> bool {
        matches!(
            self.environment_metadata(id),
            Some(EnvironmentMetadata::Function { .. })
        ) && self.heap.mutator().mut_store_object_handle(
            ObjectHandleStoreTarget::EnvironmentHomeObject(id),
            home_object,
        )
    }

    pub fn global_has_lexical_name(&self, id: EnvironmentRef, name: AtomId) -> bool {
        match self.environment_metadata(id) {
            Some(EnvironmentMetadata::Global { lexical_names, .. }) => {
                lexical_names.contains(&name)
            }
            _ => false,
        }
    }

    pub fn global_lexical_binding(
        &self,
        id: EnvironmentRef,
        name: AtomId,
    ) -> Option<GlobalLexicalBindingRecord> {
        match self.environment_metadata(id) {
            Some(EnvironmentMetadata::Global {
                lexical_bindings, ..
            }) => lexical_bindings
                .iter()
                .copied()
                .find(|binding| binding.name() == name),
            _ => None,
        }
    }

    pub fn global_add_lexical_name(&mut self, id: EnvironmentRef, name: AtomId) -> bool {
        let Some(EnvironmentMetadata::Global { lexical_names, .. }) =
            self.environment_metadata_mut(id)
        else {
            return false;
        };
        lexical_names.insert(name)
    }

    pub fn global_set_lexical_binding(
        &mut self,
        id: EnvironmentRef,
        name: AtomId,
        environment: EnvironmentRef,
        slot: u32,
    ) -> bool {
        let Some(EnvironmentMetadata::Global {
            lexical_bindings, ..
        }) = self.environment_metadata_mut(id)
        else {
            return false;
        };

        let binding = GlobalLexicalBindingRecord::new(name, environment, slot);
        if let Some(existing) = lexical_bindings
            .iter_mut()
            .find(|existing| existing.name() == name)
        {
            *existing = binding;
        } else {
            lexical_bindings.push(binding);
        }
        true
    }

    pub fn global_has_var_name(&self, id: EnvironmentRef, name: AtomId) -> bool {
        match self.environment_metadata(id) {
            Some(EnvironmentMetadata::Global { var_names, .. }) => var_names.contains(&name),
            _ => false,
        }
    }

    pub fn global_add_var_name(&mut self, id: EnvironmentRef, name: AtomId) -> bool {
        let Some(EnvironmentMetadata::Global { var_names, .. }) = self.environment_metadata_mut(id)
        else {
            return false;
        };
        var_names.insert(name)
    }

    #[allow(clippy::too_many_arguments)]
    fn alloc_environment_record(
        &mut self,
        outer: Option<EnvironmentRef>,
        layout: Option<EnvironmentLayoutId>,
        function_object: Option<ObjectRef>,
        this_value: Value,
        new_target: Option<ObjectRef>,
        home_object: Option<ObjectRef>,
        metadata: EnvironmentMetadata,
        lifetime: AllocationLifetime,
    ) -> EnvironmentRef {
        let layout = layout.and_then(|id| self.environment_layout(id).cloned());
        let env = {
            let mut mutator = self.heap.mutator();
            let slots = layout.as_ref().and_then(|layout| {
                Self::alloc_environment_slots_for_layout(&mut mutator, layout, lifetime)
            });
            mutator.alloc_environment(
                RuntimeEnvironmentRecord::new(
                    outer,
                    slots,
                    function_object,
                    this_value,
                    new_target,
                    home_object,
                ),
                lifetime,
            )
        };
        self.store_environment_metadata(env, metadata);
        env
    }

    fn alloc_environment_slots_for_layout(
        mutator: &mut lyng_js_gc::PrimitiveMutator<'_>,
        layout: &EnvironmentLayout,
        lifetime: AllocationLifetime,
    ) -> Option<EnvironmentSlotsRef> {
        let slot_count = usize::try_from(layout.slot_count())
            .expect("environment layout slot count must fit into usize");
        if slot_count == 0 {
            return None;
        }

        let slots = mutator.alloc_environment_slots(slot_count, Value::undefined(), lifetime);
        for (index, binding) in layout.bindings().iter().enumerate() {
            if binding.flags().needs_tdz() || !binding.flags().is_mutable() {
                let index = u32::try_from(index).expect("environment slot index must fit into u32");
                assert!(mutator.init_store_value(
                    ValueStoreTarget::EnvironmentSlot(slots, index),
                    Value::uninitialized_lexical(),
                ));
            }
        }
        Some(slots)
    }

    fn store_environment_metadata(&mut self, env: EnvironmentRef, metadata: EnvironmentMetadata) {
        let index = environment_index(env);
        if self.environment_metadata.len() <= index {
            self.environment_metadata.resize_with(index + 1, || None);
        }
        self.environment_metadata[index] = Some(metadata);
    }

    fn environment_metadata(&self, env: EnvironmentRef) -> Option<&EnvironmentMetadata> {
        self.environment_metadata
            .get(environment_index(env))?
            .as_ref()
    }

    fn environment_metadata_mut(
        &mut self,
        env: EnvironmentRef,
    ) -> Option<&mut EnvironmentMetadata> {
        self.environment_metadata
            .get_mut(environment_index(env))?
            .as_mut()
    }
}
