use super::{
    builtin_metadata, errors, Agent, AllocationLifetime, BuiltinFunctionId, EmbeddingFunctionId,
    FunctionConstructorFlags, FunctionObjectData, FunctionThisMode, ObjectAllocation,
    ObjectColdData, ObjectRef, PropertyDescriptor, PropertyKey, RealmRef, Value, Vm, VmError,
    VmResult, WellKnownAtom,
};

impl Vm {
    pub(in crate::vm) fn allocate_builtin_function_object(
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
}
