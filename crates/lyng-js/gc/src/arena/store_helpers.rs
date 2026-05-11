use super::{
    AtomId, CodeRef, CodeSlotsRef, EnvironmentRef, EnvironmentSlotsRef, FunctionPayloadRef,
    ObjectRef, ObjectSlotsRef, PrimitiveHeap, PrimitiveValueCellRef, RealmRef, ShapeId, StringRef,
    SuspendedRegistersRef, SymbolRef, Value,
};
use crate::HeapWriter;

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
        let mut writer = HeapWriter::new();
        self.function_payloads.update(id, |record| {
            writer.write_ref(&mut record.home_object, home_object);
        })
    }

    #[inline]
    pub(crate) fn set_function_payload_environment(
        &mut self,
        id: FunctionPayloadRef,
        environment: Option<EnvironmentRef>,
    ) -> bool {
        let mut writer = HeapWriter::new();
        self.function_payloads.update(id, |record| {
            writer.write_ref(&mut record.environment, environment);
        })
    }

    #[inline]
    pub(crate) fn set_function_payload_private_env(
        &mut self,
        id: FunctionPayloadRef,
        private_env: Option<EnvironmentRef>,
    ) -> bool {
        let mut writer = HeapWriter::new();
        self.function_payloads.update(id, |record| {
            writer.write_ref(&mut record.private_env, private_env);
        })
    }

    pub(crate) fn write_suspended_register(
        &mut self,
        id: SuspendedRegistersRef,
        index: u32,
        value: Value,
    ) -> bool {
        self.suspended_registers.write(id, index, value)
    }

    pub(crate) fn write_object_slot(
        &mut self,
        id: ObjectSlotsRef,
        index: u32,
        value: Value,
    ) -> bool {
        self.object_slots.write(id, index, value)
    }

    pub(crate) fn write_environment_slot(
        &mut self,
        id: EnvironmentSlotsRef,
        index: u32,
        value: Value,
    ) -> bool {
        self.environment_slots.write(id, index, value)
    }

    pub(crate) fn write_code_slot(&mut self, id: CodeSlotsRef, index: u32, value: Value) -> bool {
        self.code_slots.write(id, index, value)
    }

    pub(crate) fn set_symbol_description(
        &mut self,
        id: SymbolRef,
        description: Option<StringRef>,
    ) -> bool {
        let mut writer = HeapWriter::new();
        self.symbols.update(id, |record| {
            writer.write_ref(&mut record.description, description);
        })
    }

    pub(crate) fn set_value_cell_value(&mut self, id: PrimitiveValueCellRef, value: Value) -> bool {
        let mut writer = HeapWriter::new();
        self.value_cells.update(id, |record| {
            writer.write_value(&mut record.stored_value, value);
        })
    }

    pub(crate) fn set_value_cell_linked_string(
        &mut self,
        id: PrimitiveValueCellRef,
        linked_string: Option<StringRef>,
    ) -> bool {
        let mut writer = HeapWriter::new();
        self.value_cells.update(id, |record| {
            writer.write_ref(&mut record.linked_string, linked_string);
        })
    }

    pub(crate) fn set_object_prototype(
        &mut self,
        id: ObjectRef,
        prototype: Option<ObjectRef>,
    ) -> bool {
        let mut writer = HeapWriter::new();
        self.objects.update(id, |record| {
            writer.write_ref(&mut record.prototype, prototype);
        })
    }

    pub(crate) fn set_object_shape(&mut self, id: ObjectRef, shape: Option<ShapeId>) -> bool {
        let mut writer = HeapWriter::new();
        self.objects.update(id, |record| {
            writer.write_ref(&mut record.shape, shape);
        })
    }

    pub(crate) fn set_object_named_slots(
        &mut self,
        id: ObjectRef,
        named_slots: Option<ObjectSlotsRef>,
    ) -> bool {
        let mut writer = HeapWriter::new();
        self.objects.update(id, |record| {
            writer.write_ref(&mut record.named_slots, named_slots);
        })
    }

    pub(crate) fn set_object_elements(
        &mut self,
        id: ObjectRef,
        elements: Option<ObjectSlotsRef>,
    ) -> bool {
        let mut writer = HeapWriter::new();
        self.objects.update(id, |record| {
            writer.write_ref(&mut record.elements, elements);
        })
    }

    pub(crate) fn set_object_private_slots(
        &mut self,
        id: ObjectRef,
        private_slots: Option<ObjectSlotsRef>,
    ) -> bool {
        let mut writer = HeapWriter::new();
        self.objects.update(id, |record| {
            writer.write_ref(&mut record.private_slots, private_slots);
        })
    }

    pub(crate) fn set_environment_outer(
        &mut self,
        id: EnvironmentRef,
        outer: Option<EnvironmentRef>,
    ) -> bool {
        let mut writer = HeapWriter::new();
        self.environments.update(id, |record| {
            writer.write_ref(&mut record.outer, outer);
        })
    }

    pub(crate) fn set_environment_function_object(
        &mut self,
        id: EnvironmentRef,
        function_object: Option<ObjectRef>,
    ) -> bool {
        let mut writer = HeapWriter::new();
        self.environments.update(id, |record| {
            writer.write_ref(&mut record.function_object, function_object);
        })
    }

    pub(crate) fn set_environment_this_value(
        &mut self,
        id: EnvironmentRef,
        this_value: Value,
    ) -> bool {
        let mut writer = HeapWriter::new();
        self.environments.update(id, |record| {
            writer.write_value(&mut record.this_value, this_value);
        })
    }

    pub(crate) fn set_environment_new_target(
        &mut self,
        id: EnvironmentRef,
        new_target: Option<ObjectRef>,
    ) -> bool {
        let mut writer = HeapWriter::new();
        self.environments.update(id, |record| {
            writer.write_ref(&mut record.new_target, new_target);
        })
    }

    pub(crate) fn set_environment_home_object(
        &mut self,
        id: EnvironmentRef,
        home_object: Option<ObjectRef>,
    ) -> bool {
        let mut writer = HeapWriter::new();
        self.environments.update(id, |record| {
            writer.write_ref(&mut record.home_object, home_object);
        })
    }

    pub(crate) fn set_code_parent(&mut self, id: CodeRef, parent: Option<CodeRef>) -> bool {
        let mut writer = HeapWriter::new();
        self.codes.update(id, |record| {
            writer.write_ref(&mut record.parent, parent);
        })
    }

    pub(crate) fn set_code_realm(&mut self, id: CodeRef, realm: Option<RealmRef>) -> bool {
        let mut writer = HeapWriter::new();
        self.codes.update(id, |record| {
            writer.write_ref(&mut record.realm, realm);
        })
    }

    pub(crate) fn set_realm_global_object(
        &mut self,
        id: RealmRef,
        global_object: Option<ObjectRef>,
    ) -> bool {
        let mut writer = HeapWriter::new();
        self.realms.update(id, |record| {
            writer.write_ref(&mut record.global_object, global_object);
        })
    }

    pub(crate) fn set_realm_global_env(
        &mut self,
        id: RealmRef,
        global_env: Option<EnvironmentRef>,
    ) -> bool {
        let mut writer = HeapWriter::new();
        self.realms.update(id, |record| {
            writer.write_ref(&mut record.global_env, global_env);
        })
    }

    pub(crate) fn set_realm_bootstrap_code(
        &mut self,
        id: RealmRef,
        bootstrap_code: Option<CodeRef>,
    ) -> bool {
        let mut writer = HeapWriter::new();
        self.realms.update(id, |record| {
            writer.write_ref(&mut record.bootstrap_code, bootstrap_code);
        })
    }

    pub(crate) fn set_realm_root_shape(
        &mut self,
        id: RealmRef,
        root_shape: Option<ShapeId>,
    ) -> bool {
        let mut writer = HeapWriter::new();
        self.realms.update(id, |record| {
            writer.write_ref(&mut record.root_shape, root_shape);
        })
    }

    pub(crate) fn set_shape_parent(&mut self, id: ShapeId, parent: Option<ShapeId>) -> bool {
        let mut writer = HeapWriter::new();
        self.shapes.update(id, |record| {
            writer.write_ref(&mut record.parent, parent);
        })
    }

    pub(crate) fn set_shape_prototype_guard(
        &mut self,
        id: ShapeId,
        prototype_guard: Option<ObjectRef>,
    ) -> bool {
        let mut writer = HeapWriter::new();
        self.shapes.update(id, |record| {
            writer.write_ref(&mut record.prototype_guard, prototype_guard);
        })
    }
}
