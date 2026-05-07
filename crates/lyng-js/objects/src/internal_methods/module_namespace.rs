use super::*;

impl ObjectRuntime {
    pub(super) fn module_namespace_get_own_property(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
        key: PropertyKey,
    ) -> InternalMethodResult<Option<PropertyDescriptor>> {
        let Some(export) = self
            .module_namespace_slot(id)
            .and_then(|namespace| namespace.export_for_key(key))
        else {
            return self.ordinary_get_own_property(heap, id, key);
        };
        let mut descriptor = PropertyDescriptor::new();
        descriptor.set_value(self.module_namespace_export_value(heap, export)?);
        descriptor.set_writable(true);
        descriptor.set_enumerable(true);
        descriptor.set_configurable(false);
        Ok(Some(descriptor))
    }

    pub(super) fn module_namespace_define_own_property(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        key: PropertyKey,
        descriptor: PropertyDescriptor,
        lifetime: AllocationLifetime,
    ) -> InternalMethodResult<bool> {
        let Some(export) = self
            .module_namespace_slot(id)
            .and_then(|namespace| namespace.export_for_key(key))
        else {
            return self.ordinary_define_own_property(heap, id, key, descriptor, lifetime);
        };
        if descriptor.getter().is_some() || descriptor.setter().is_some() {
            return Ok(false);
        }
        if descriptor.configurable() == Some(true) {
            return Ok(false);
        }
        if descriptor.enumerable() == Some(false) {
            return Ok(false);
        }
        if descriptor.writable() == Some(false) {
            return Ok(false);
        }
        if let Some(value) = descriptor.value()
            && !descriptor_same_value(
                heap.view(),
                value,
                self.module_namespace_export_value(heap.view(), export)?,
            )?
        {
            return Ok(false);
        }
        Ok(true)
    }

    pub(super) fn module_namespace_has_property(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
        key: PropertyKey,
    ) -> InternalMethodResult<bool> {
        let Some(namespace) = self.module_namespace_slot(id) else {
            return Err(InternalMethodError::MissingObject);
        };
        if namespace.export_for_key(key).is_some() {
            return Ok(true);
        }
        self.ordinary_has_property(heap, id, key)
    }

    pub(super) fn module_namespace_delete(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        key: PropertyKey,
    ) -> InternalMethodResult<bool> {
        if self
            .module_namespace_slot(id)
            .and_then(|namespace| namespace.export_for_key(key))
            .is_some()
        {
            return Ok(false);
        }
        self.ordinary_delete(heap, id, key)
    }

    pub(super) fn module_namespace_own_property_keys(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> InternalMethodResult<Vec<PropertyKey>> {
        let Some(namespace) = self.module_namespace_slot(id) else {
            return Err(InternalMethodError::MissingObject);
        };
        let mut keys = namespace
            .exports()
            .iter()
            .map(|entry| entry.export_key())
            .collect::<Vec<_>>();
        let (_, mut symbols) = self.collect_own_named_keys(heap, id)?;
        keys.append(&mut symbols);
        Ok(keys)
    }

    fn module_namespace_export_value(
        &self,
        heap: PrimitiveHeapView<'_>,
        export: crate::ModuleNamespaceExport,
    ) -> InternalMethodResult<Value> {
        match export.target() {
            crate::ModuleNamespaceExportTarget::Binding { environment, slot } => {
                let slots = heap
                    .environment(environment)
                    .and_then(lyng_js_gc::RuntimeEnvironmentRecord::slots)
                    .and_then(|slots| heap.environment_slots(slots))
                    .ok_or(InternalMethodError::CorruptObjectState)?;
                let value = slots
                    .get(slot as usize)
                    .copied()
                    .ok_or(InternalMethodError::CorruptObjectState)?;
                if value == Value::uninitialized_lexical() {
                    return Err(InternalMethodError::ReferenceError);
                }
                Ok(value)
            }
            crate::ModuleNamespaceExportTarget::Value(value) => Ok(value),
        }
    }
}
