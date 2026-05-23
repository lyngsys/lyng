use super::{
    AtomId, CodeRef, CodeSlotsRef, EnvironmentRef, EnvironmentSlotsRef, FunctionPayloadRef,
    HeapGeneration, ObjectRef, ObjectSlotsRef, PrimitiveHeap, PrimitiveStringRecord,
    PrimitiveSymbolRecord, PrimitiveValueCellRef, RealmRef, RuntimeBoundFunctionRecord,
    RuntimeEnvironmentRecord, RuntimeFunctionRecord, RuntimeObjectRecord, RuntimeRealmRecord,
    RuntimeShapeRecord, RuntimeSuspendedExecutionRecord, ShapeId, StringRef, SuspendedExecutionRef,
    SuspendedRegistersRef, SymbolRef, Value,
};
use crate::{card_table::CardDomain, HeapWriter};

impl PrimitiveHeap {
    pub(crate) fn cache_string_hash(&mut self, id: StringRef) -> Option<u32> {
        if let Some(hash) = self.string(id)?.cached_hash() {
            return Some(hash);
        }

        let hash = {
            let view = self.string_view(id)?;
            view.compute_hash()
        };

        if self.strings.update(id, |record| {
            record.cached_hash = Some(hash);
        }) {
            Some(hash)
        } else {
            None
        }
    }

    pub(crate) fn memoize_string_atom(&mut self, id: StringRef, atom: AtomId) -> bool {
        self.strings.update(id, |record| match record.cached_atom {
            Some(existing) => debug_assert_eq!(
                existing, atom,
                "string atom cache should not change to a different AtomId"
            ),
            None => {
                record.cached_atom = Some(atom);
            }
        })
    }

    #[inline]
    pub(crate) fn set_function_payload_home_object(
        &mut self,
        id: FunctionPayloadRef,
        home_object: Option<ObjectRef>,
    ) -> bool {
        if self.object_ref_points_to_young(home_object) {
            self.mark_function_payload_card_if_old(id);
        }
        let mut writer = HeapWriter::new();
        let wrote = self.function_payloads.update(id, |record| {
            writer.write_ref(&mut record.home_object, home_object);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &home_object);
        }
        wrote
    }

    #[inline]
    pub(crate) fn set_function_payload_environment(
        &mut self,
        id: FunctionPayloadRef,
        environment: Option<EnvironmentRef>,
    ) -> bool {
        if self.environment_ref_points_to_young(environment) {
            self.mark_function_payload_card_if_old(id);
        }
        let mut writer = HeapWriter::new();
        let wrote = self.function_payloads.update(id, |record| {
            writer.write_ref(&mut record.environment, environment);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &environment);
        }
        wrote
    }

    #[inline]
    pub(crate) fn set_function_payload_private_env(
        &mut self,
        id: FunctionPayloadRef,
        private_env: Option<EnvironmentRef>,
    ) -> bool {
        if self.environment_ref_points_to_young(private_env) {
            self.mark_function_payload_card_if_old(id);
        }
        let mut writer = HeapWriter::new();
        let wrote = self.function_payloads.update(id, |record| {
            writer.write_ref(&mut record.private_env, private_env);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &private_env);
        }
        wrote
    }

    pub(crate) fn write_suspended_register(
        &mut self,
        id: SuspendedRegistersRef,
        index: u32,
        value: Value,
    ) -> bool {
        if self.suspended_registers.generation(id) == Some(super::HeapGeneration::Old)
            && self.value_points_to_young(value)
        {
            self.mark_card(
                CardDomain::SuspendedRegisters,
                super::storage::ValueSlotAllocator::<SuspendedRegistersRef>::card_index(id),
            );
        }
        let wrote = self.suspended_registers.write(id, index, value);
        if wrote {
            HeapWriter::incremental_value_barrier(self, id, value);
        }
        wrote
    }

    pub(crate) fn write_object_slot(
        &mut self,
        id: ObjectSlotsRef,
        index: u32,
        value: Value,
    ) -> bool {
        if self.object_slots.generation(id) == Some(super::HeapGeneration::Old)
            && self.value_points_to_young(value)
        {
            self.mark_card(
                CardDomain::ObjectSlots,
                super::storage::ValueSlotAllocator::<ObjectSlotsRef>::card_index(id),
            );
        }
        let wrote = self.object_slots.write(id, index, value);
        if wrote {
            HeapWriter::incremental_value_barrier(self, id, value);
        }
        wrote
    }

    /// Writes one `Value` into `RuntimeObjectRecord.inline_named_slots[index]`. Used by the
    /// runtime's named-property fast path when a shape places its slot inline. Returns
    /// `false` when the index is out of range or the object record has been freed.
    /// Runs the incremental-marking value barrier on the holder so heap references
    /// embedded in inline slots are shaded gray when an incremental mark is in flight.
    pub(crate) fn write_object_inline_named_slot(
        &mut self,
        id: ObjectRef,
        index: u32,
        value: Value,
    ) -> bool {
        let slot_index = index as usize;
        if slot_index >= super::RUNTIME_OBJECT_INLINE_SLOT_COUNT {
            return false;
        }
        if self.value_points_to_young(value) {
            self.mark_object_card_if_old(id);
        }
        let mut writer = HeapWriter::new();
        let wrote = self.objects.update(id, |record| {
            writer.write_value(&mut record.inline_named_slots[slot_index], value);
        });
        if wrote {
            HeapWriter::incremental_value_barrier(self, id, value);
        }
        wrote
    }

    #[inline]
    pub(crate) fn set_object_invalidation_epoch(&mut self, id: ObjectRef, epoch: u64) -> bool {
        self.objects.update(id, |record| {
            record.last_invalidation_epoch = epoch;
        })
    }

    pub(crate) fn write_environment_slot(
        &mut self,
        id: EnvironmentSlotsRef,
        index: u32,
        value: Value,
    ) -> bool {
        if self.environment_slots.generation(id) == Some(super::HeapGeneration::Old)
            && self.value_points_to_young(value)
        {
            self.mark_card(
                CardDomain::EnvironmentSlots,
                super::storage::ValueSlotAllocator::<EnvironmentSlotsRef>::card_index(id),
            );
        }
        let wrote = self.environment_slots.write(id, index, value);
        if wrote {
            HeapWriter::incremental_value_barrier(self, id, value);
        }
        wrote
    }

    pub(crate) fn write_code_slot(&mut self, id: CodeSlotsRef, index: u32, value: Value) -> bool {
        if self.code_slots.generation(id) == Some(super::HeapGeneration::Old)
            && self.value_points_to_young(value)
        {
            self.mark_card(
                CardDomain::CodeSlots,
                super::storage::ValueSlotAllocator::<CodeSlotsRef>::card_index(id),
            );
        }
        let wrote = self.code_slots.write(id, index, value);
        if wrote {
            HeapWriter::incremental_value_barrier(self, id, value);
        }
        wrote
    }

    pub(crate) fn set_symbol_description(
        &mut self,
        id: SymbolRef,
        description: Option<StringRef>,
    ) -> bool {
        if self.string_ref_points_to_young(description) {
            self.mark_symbol_card_if_old(id);
        }
        let mut writer = HeapWriter::new();
        let wrote = self.symbols.update(id, |record| {
            writer.write_ref(&mut record.description, description);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &description);
        }
        wrote
    }

    pub(crate) fn set_value_cell_value(&mut self, id: PrimitiveValueCellRef, value: Value) -> bool {
        if self.value_cells.generation(id) == Some(super::HeapGeneration::Old)
            && self.value_points_to_young(value)
        {
            self.mark_card(
                CardDomain::ValueCell,
                super::storage::SlotArena::<
                    super::PrimitiveValueCellRecord,
                    PrimitiveValueCellRef,
                >::card_index(id, std::mem::size_of::<super::PrimitiveValueCellRecord>()),
            );
        }
        let mut writer = HeapWriter::new();
        let wrote = self.value_cells.update(id, |record| {
            writer.write_value(&mut record.stored_value, value);
        });
        if wrote {
            HeapWriter::incremental_value_barrier(self, id, value);
        }
        wrote
    }

    pub(crate) fn set_value_cell_linked_string(
        &mut self,
        id: PrimitiveValueCellRef,
        linked_string: Option<StringRef>,
    ) -> bool {
        if self.string_ref_points_to_young(linked_string) {
            self.mark_value_cell_card_if_old(id);
        }
        let mut writer = HeapWriter::new();
        let wrote = self.value_cells.update(id, |record| {
            writer.write_ref(&mut record.linked_string, linked_string);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &linked_string);
        }
        wrote
    }

    pub(crate) fn set_object_prototype(
        &mut self,
        id: ObjectRef,
        prototype: Option<ObjectRef>,
    ) -> bool {
        if self.object_ref_points_to_young(prototype) {
            self.mark_object_card_if_old(id);
        }
        let mut writer = HeapWriter::new();
        let wrote = self.objects.update(id, |record| {
            writer.write_ref(&mut record.prototype, prototype);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &prototype);
        }
        wrote
    }

    pub(crate) fn set_object_shape(&mut self, id: ObjectRef, shape: Option<ShapeId>) -> bool {
        let mut writer = HeapWriter::new();
        let wrote = self.objects.update(id, |record| {
            writer.write_ref(&mut record.shape, shape);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &shape);
        }
        wrote
    }

    pub(crate) fn set_object_named_slots(
        &mut self,
        id: ObjectRef,
        named_slots: Option<ObjectSlotsRef>,
    ) -> bool {
        if self.object_slots_ref_points_to_young(named_slots) {
            self.mark_object_card_if_old(id);
        }
        let mut writer = HeapWriter::new();
        let wrote = self.objects.update(id, |record| {
            writer.write_ref(&mut record.named_slots, named_slots);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &named_slots);
        }
        wrote
    }

    pub(crate) fn set_object_elements(
        &mut self,
        id: ObjectRef,
        elements: Option<ObjectSlotsRef>,
    ) -> bool {
        if self.object_slots_ref_points_to_young(elements) {
            self.mark_object_card_if_old(id);
        }
        let mut writer = HeapWriter::new();
        let wrote = self.objects.update(id, |record| {
            writer.write_ref(&mut record.elements, elements);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &elements);
        }
        wrote
    }

    pub(crate) fn set_object_private_slots(
        &mut self,
        id: ObjectRef,
        private_slots: Option<ObjectSlotsRef>,
    ) -> bool {
        if self.object_slots_ref_points_to_young(private_slots) {
            self.mark_object_card_if_old(id);
        }
        let mut writer = HeapWriter::new();
        let wrote = self.objects.update(id, |record| {
            writer.write_ref(&mut record.private_slots, private_slots);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &private_slots);
        }
        wrote
    }

    pub(crate) fn set_environment_outer(
        &mut self,
        id: EnvironmentRef,
        outer: Option<EnvironmentRef>,
    ) -> bool {
        if self.environment_ref_points_to_young(outer) {
            self.mark_environment_card_if_old(id);
        }
        let mut writer = HeapWriter::new();
        let wrote = self.environments.update(id, |record| {
            writer.write_ref(&mut record.outer, outer);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &outer);
        }
        wrote
    }

    pub(crate) fn set_environment_function_object(
        &mut self,
        id: EnvironmentRef,
        function_object: Option<ObjectRef>,
    ) -> bool {
        if self.object_ref_points_to_young(function_object) {
            self.mark_environment_card_if_old(id);
        }
        let mut writer = HeapWriter::new();
        let wrote = self.environments.update(id, |record| {
            writer.write_ref(&mut record.function_object, function_object);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &function_object);
        }
        wrote
    }

    pub(crate) fn set_environment_this_value(
        &mut self,
        id: EnvironmentRef,
        this_value: Value,
    ) -> bool {
        if self.environments.generation(id) == Some(super::HeapGeneration::Old)
            && self.value_points_to_young(this_value)
        {
            self.mark_card(
                CardDomain::Environment,
                super::storage::SlotArena::<
                    super::RuntimeEnvironmentRecord,
                    EnvironmentRef,
                >::card_index(id, std::mem::size_of::<super::RuntimeEnvironmentRecord>()),
            );
        }
        let mut writer = HeapWriter::new();
        let wrote = self.environments.update(id, |record| {
            writer.write_value(&mut record.this_value, this_value);
        });
        if wrote {
            HeapWriter::incremental_value_barrier(self, id, this_value);
        }
        wrote
    }

    pub(crate) fn set_environment_new_target(
        &mut self,
        id: EnvironmentRef,
        new_target: Option<ObjectRef>,
    ) -> bool {
        if self.object_ref_points_to_young(new_target) {
            self.mark_environment_card_if_old(id);
        }
        let mut writer = HeapWriter::new();
        let wrote = self.environments.update(id, |record| {
            writer.write_ref(&mut record.new_target, new_target);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &new_target);
        }
        wrote
    }

    pub(crate) fn set_environment_home_object(
        &mut self,
        id: EnvironmentRef,
        home_object: Option<ObjectRef>,
    ) -> bool {
        if self.object_ref_points_to_young(home_object) {
            self.mark_environment_card_if_old(id);
        }
        let mut writer = HeapWriter::new();
        let wrote = self.environments.update(id, |record| {
            writer.write_ref(&mut record.home_object, home_object);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &home_object);
        }
        wrote
    }

    pub(crate) fn set_code_parent(&mut self, id: CodeRef, parent: Option<CodeRef>) -> bool {
        let mut writer = HeapWriter::new();
        let wrote = self.codes.update(id, |record| {
            writer.write_ref(&mut record.parent, parent);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &parent);
        }
        wrote
    }

    pub(crate) fn set_code_realm(&mut self, id: CodeRef, realm: Option<RealmRef>) -> bool {
        let mut writer = HeapWriter::new();
        let wrote = self.codes.update(id, |record| {
            writer.write_ref(&mut record.realm, realm);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &realm);
        }
        wrote
    }

    pub(crate) fn set_realm_global_object(
        &mut self,
        id: RealmRef,
        global_object: Option<ObjectRef>,
    ) -> bool {
        if self.object_ref_points_to_young(global_object) {
            self.mark_realm_card_if_old(id);
        }
        let mut writer = HeapWriter::new();
        let wrote = self.realms.update(id, |record| {
            writer.write_ref(&mut record.global_object, global_object);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &global_object);
        }
        wrote
    }

    pub(crate) fn set_realm_global_env(
        &mut self,
        id: RealmRef,
        global_env: Option<EnvironmentRef>,
    ) -> bool {
        if self.environment_ref_points_to_young(global_env) {
            self.mark_realm_card_if_old(id);
        }
        let mut writer = HeapWriter::new();
        let wrote = self.realms.update(id, |record| {
            writer.write_ref(&mut record.global_env, global_env);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &global_env);
        }
        wrote
    }

    pub(crate) fn set_realm_bootstrap_code(
        &mut self,
        id: RealmRef,
        bootstrap_code: Option<CodeRef>,
    ) -> bool {
        let mut writer = HeapWriter::new();
        let wrote = self.realms.update(id, |record| {
            writer.write_ref(&mut record.bootstrap_code, bootstrap_code);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &bootstrap_code);
        }
        wrote
    }

    pub(crate) fn set_realm_root_shape(
        &mut self,
        id: RealmRef,
        root_shape: Option<ShapeId>,
    ) -> bool {
        let mut writer = HeapWriter::new();
        let wrote = self.realms.update(id, |record| {
            writer.write_ref(&mut record.root_shape, root_shape);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &root_shape);
        }
        wrote
    }

    pub(crate) fn set_shape_parent(&mut self, id: ShapeId, parent: Option<ShapeId>) -> bool {
        let mut writer = HeapWriter::new();
        let wrote = self.shapes.update(id, |record| {
            writer.write_ref(&mut record.parent, parent);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &parent);
        }
        wrote
    }

    pub(crate) fn set_shape_prototype_guard(
        &mut self,
        id: ShapeId,
        prototype_guard: Option<ObjectRef>,
    ) -> bool {
        if self.object_ref_points_to_young(prototype_guard) {
            self.mark_shape_card_if_old(id);
        }
        let mut writer = HeapWriter::new();
        let wrote = self.shapes.update(id, |record| {
            writer.write_ref(&mut record.prototype_guard, prototype_guard);
        });
        if wrote {
            HeapWriter::incremental_ref_barrier(self, id, &prototype_guard);
        }
        wrote
    }

    pub(super) fn mark_string_card_if_old_points_to_young(
        &mut self,
        id: StringRef,
        record: PrimitiveStringRecord,
    ) {
        let points_to_young = record
            .cons_children()
            .is_some_and(|(left, right)| self.is_young_string(left) || self.is_young_string(right));
        if points_to_young {
            self.mark_string_card_if_old(id);
        }
    }

    pub(super) fn mark_symbol_card_if_old_points_to_young(
        &mut self,
        id: SymbolRef,
        description: Option<StringRef>,
    ) {
        if self.string_ref_points_to_young(description) {
            self.mark_symbol_card_if_old(id);
        }
    }

    pub(super) fn mark_object_card_if_old_points_to_young(
        &mut self,
        id: ObjectRef,
        record: RuntimeObjectRecord,
    ) {
        if self.object_record_points_to_young(record) {
            self.mark_object_card_if_old(id);
        }
    }

    pub(super) fn mark_function_payload_card_if_old_points_to_young(
        &mut self,
        id: FunctionPayloadRef,
        record: RuntimeFunctionRecord,
    ) {
        if self.function_payload_record_points_to_young(record) {
            self.mark_function_payload_card_if_old(id);
        }
    }

    pub(super) fn mark_suspended_execution_card_if_old_points_to_young(
        &mut self,
        id: SuspendedExecutionRef,
        record: RuntimeSuspendedExecutionRecord,
    ) {
        if self.suspended_execution_record_points_to_young(record) {
            self.mark_suspended_execution_card_if_old(id);
        }
    }

    pub(super) fn mark_environment_card_if_old_points_to_young(
        &mut self,
        id: EnvironmentRef,
        record: RuntimeEnvironmentRecord,
    ) {
        if self.environment_record_points_to_young(record) {
            self.mark_environment_card_if_old(id);
        }
    }

    pub(super) fn mark_value_slot_card_if_old_points_to_young<
        Handle: super::storage::ArenaHandle,
    >(
        &mut self,
        domain: CardDomain,
        id: Handle,
        generation: HeapGeneration,
        fill: Value,
    ) {
        if generation == HeapGeneration::Old && self.value_points_to_young(fill) {
            self.mark_card(
                domain,
                super::storage::ValueSlotAllocator::<Handle>::card_index(id),
            );
        }
    }

    pub(super) fn mark_realm_card_if_old_points_to_young(
        &mut self,
        id: RealmRef,
        record: RuntimeRealmRecord,
    ) {
        if self.realm_record_points_to_young(record) {
            self.mark_realm_card_if_old(id);
        }
    }

    pub(super) fn mark_shape_card_if_old_points_to_young(
        &mut self,
        id: ShapeId,
        record: RuntimeShapeRecord,
    ) {
        if self.shape_record_points_to_young(record) {
            self.mark_shape_card_if_old(id);
        }
    }

    fn mark_string_card_if_old(&mut self, id: StringRef) {
        if self.strings.generation(id) == Some(HeapGeneration::Old) {
            self.mark_card(
                CardDomain::String,
                super::storage::SlotArena::<PrimitiveStringRecord, StringRef>::card_index(
                    id,
                    std::mem::size_of::<PrimitiveStringRecord>(),
                ),
            );
        }
    }

    fn mark_symbol_card_if_old(&mut self, id: SymbolRef) {
        if self.symbols.generation(id) == Some(HeapGeneration::Old) {
            self.mark_card(
                CardDomain::Symbol,
                super::storage::SlotArena::<PrimitiveSymbolRecord, SymbolRef>::card_index(
                    id,
                    std::mem::size_of::<PrimitiveSymbolRecord>(),
                ),
            );
        }
    }

    fn mark_value_cell_card_if_old(&mut self, id: PrimitiveValueCellRef) {
        if self.value_cells.generation(id) == Some(HeapGeneration::Old) {
            self.mark_card(
                CardDomain::ValueCell,
                super::storage::SlotArena::<
                    super::PrimitiveValueCellRecord,
                    PrimitiveValueCellRef,
                >::card_index(id, std::mem::size_of::<super::PrimitiveValueCellRecord>()),
            );
        }
    }

    fn mark_object_card_if_old(&mut self, id: ObjectRef) {
        if self.objects.generation(id) == Some(HeapGeneration::Old) {
            self.mark_card(
                CardDomain::Object,
                super::storage::SlotArena::<RuntimeObjectRecord, ObjectRef>::card_index(
                    id,
                    std::mem::size_of::<RuntimeObjectRecord>(),
                ),
            );
        }
    }

    fn mark_function_payload_card_if_old(&mut self, id: FunctionPayloadRef) {
        if self.function_payloads.generation(id) == Some(HeapGeneration::Old) {
            self.mark_card(
                CardDomain::FunctionPayload,
                super::storage::SlotArena::<RuntimeFunctionRecord, FunctionPayloadRef>::card_index(
                    id,
                    std::mem::size_of::<RuntimeFunctionRecord>(),
                ),
            );
        }
    }

    fn mark_suspended_execution_card_if_old(&mut self, id: SuspendedExecutionRef) {
        if self.suspended_executions.generation(id) == Some(HeapGeneration::Old) {
            self.mark_card(
                CardDomain::SuspendedExecution,
                super::storage::SlotArena::<
                    RuntimeSuspendedExecutionRecord,
                    SuspendedExecutionRef,
                >::card_index(id, std::mem::size_of::<RuntimeSuspendedExecutionRecord>()),
            );
        }
    }

    fn mark_environment_card_if_old(&mut self, id: EnvironmentRef) {
        if self.environments.generation(id) == Some(HeapGeneration::Old) {
            self.mark_card(
                CardDomain::Environment,
                super::storage::SlotArena::<RuntimeEnvironmentRecord, EnvironmentRef>::card_index(
                    id,
                    std::mem::size_of::<RuntimeEnvironmentRecord>(),
                ),
            );
        }
    }

    fn mark_realm_card_if_old(&mut self, id: RealmRef) {
        if self.realms.generation(id) == Some(HeapGeneration::Old) {
            self.mark_card(
                CardDomain::Realm,
                super::storage::SlotArena::<RuntimeRealmRecord, RealmRef>::card_index(
                    id,
                    std::mem::size_of::<RuntimeRealmRecord>(),
                ),
            );
        }
    }

    fn mark_shape_card_if_old(&mut self, id: ShapeId) {
        if self.shapes.generation(id) == Some(HeapGeneration::Old) {
            self.mark_card(
                CardDomain::Shape,
                super::storage::SlotArena::<RuntimeShapeRecord, ShapeId>::card_index(
                    id,
                    std::mem::size_of::<RuntimeShapeRecord>(),
                ),
            );
        }
    }

    fn object_record_points_to_young(&self, record: RuntimeObjectRecord) -> bool {
        self.object_ref_points_to_young(record.prototype())
            || self.object_slots_ref_points_to_young(record.named_slots())
            || self.object_slots_ref_points_to_young(record.elements())
            || self.object_slots_ref_points_to_young(record.private_slots())
            || self.function_payload_ref_points_to_young(record.function_payload())
            || self.value_cell_ref_points_to_young(record.ordinary_payload())
    }

    fn function_payload_record_points_to_young(&self, record: RuntimeFunctionRecord) -> bool {
        self.environment_ref_points_to_young(record.environment())
            || self.environment_ref_points_to_young(record.private_env())
            || self.object_ref_points_to_young(record.home_object())
            || record
                .bound()
                .is_some_and(|bound| self.bound_function_record_points_to_young(bound))
    }

    fn bound_function_record_points_to_young(&self, record: RuntimeBoundFunctionRecord) -> bool {
        self.is_young_object(record.target())
            || self.value_points_to_young(record.this_value())
            || self.object_slots_ref_points_to_young(record.arguments())
    }

    fn suspended_execution_record_points_to_young(
        &self,
        record: RuntimeSuspendedExecutionRecord,
    ) -> bool {
        self.is_young_environment(record.lexical_env())
            || self.is_young_environment(record.variable_env())
            || self.environment_ref_points_to_young(record.private_env())
            || self.value_points_to_young(record.this_value())
            || self.object_ref_points_to_young(record.construct_this())
            || self.object_ref_points_to_young(record.new_target())
            || self.object_ref_points_to_young(record.callee())
            || self.suspended_registers_ref_points_to_young(record.registers())
    }

    fn environment_record_points_to_young(&self, record: RuntimeEnvironmentRecord) -> bool {
        self.environment_ref_points_to_young(record.outer())
            || self.environment_slots_ref_points_to_young(record.slots())
            || self.object_ref_points_to_young(record.function_object())
            || self.value_points_to_young(record.this_value())
            || self.object_ref_points_to_young(record.new_target())
            || self.object_ref_points_to_young(record.home_object())
    }

    fn realm_record_points_to_young(&self, record: RuntimeRealmRecord) -> bool {
        self.object_ref_points_to_young(record.global_object())
            || self.environment_ref_points_to_young(record.global_env())
    }

    fn shape_record_points_to_young(&self, record: RuntimeShapeRecord) -> bool {
        self.object_ref_points_to_young(record.prototype_guard())
    }

    pub(crate) fn value_points_to_young(&self, value: Value) -> bool {
        value
            .as_object_ref()
            .is_some_and(|id| self.is_young_object(id))
            || value
                .as_string_ref()
                .is_some_and(|id| self.is_young_string(id))
            || value
                .as_symbol_ref()
                .is_some_and(|id| self.is_young_symbol(id))
            || value
                .as_bigint_ref()
                .is_some_and(|id| self.is_young_bigint(id))
            || value
                .as_suspended_execution_ref()
                .is_some_and(|id| self.is_young_suspended_execution(id))
    }

    fn string_ref_points_to_young(&self, id: Option<StringRef>) -> bool {
        id.is_some_and(|id| self.is_young_string(id))
    }

    fn object_ref_points_to_young(&self, id: Option<ObjectRef>) -> bool {
        id.is_some_and(|id| self.is_young_object(id))
    }

    fn function_payload_ref_points_to_young(&self, id: Option<FunctionPayloadRef>) -> bool {
        id.is_some_and(|id| self.is_young_function_payload(id))
    }

    fn value_cell_ref_points_to_young(&self, id: Option<PrimitiveValueCellRef>) -> bool {
        id.is_some_and(|id| self.is_young_value_cell(id))
    }

    fn object_slots_ref_points_to_young(&self, id: Option<ObjectSlotsRef>) -> bool {
        id.is_some_and(|id| self.is_young_object_slots(id))
    }

    fn environment_ref_points_to_young(&self, id: Option<EnvironmentRef>) -> bool {
        id.is_some_and(|id| self.is_young_environment(id))
    }

    fn environment_slots_ref_points_to_young(&self, id: Option<EnvironmentSlotsRef>) -> bool {
        id.is_some_and(|id| self.is_young_environment_slots(id))
    }

    fn suspended_registers_ref_points_to_young(&self, id: Option<SuspendedRegistersRef>) -> bool {
        id.is_some_and(|id| self.is_young_suspended_registers(id))
    }
}
