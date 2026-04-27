use super::*;

impl ObjectRuntime {
    pub(super) fn typed_array_define_own_property(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        key: PropertyKey,
        descriptor: PropertyDescriptor,
        lifetime: AllocationLifetime,
    ) -> InternalMethodResult<bool> {
        let Some(index) = key.as_index() else {
            return self.ordinary_define_own_property(heap, id, key, descriptor, lifetime);
        };
        let Some(typed_array) = self.typed_array(id) else {
            return Err(InternalMethodError::CorruptObjectState);
        };
        let length = u32::try_from(typed_array.length()).unwrap_or(u32::MAX);
        if index >= length {
            return Ok(false);
        }
        if descriptor.has_get() || descriptor.has_set() {
            return Ok(false);
        }
        if descriptor.configurable() == Some(false) {
            return Ok(false);
        }
        if descriptor.enumerable() == Some(false) {
            return Ok(false);
        }
        if descriptor.writable() == Some(false) {
            return Ok(false);
        }
        let current = self.ordinary_own_index_property(heap.view(), id, index)?;
        let Some(current) = current else {
            return Err(InternalMethodError::CorruptObjectState);
        };
        if let Some(value) = descriptor.value() {
            let mut normalized = PropertyDescriptor::new();
            normalized.set_value(value);
            normalized.set_writable(true);
            normalized.set_enumerable(true);
            normalized.set_configurable(true);
            return self.ordinary_define_own_index_property(
                heap,
                id,
                index,
                Some(current),
                normalized,
                lifetime,
            );
        }
        Ok(true)
    }

    pub(super) fn typed_array_delete(
        &mut self,
        id: ObjectRef,
        key: PropertyKey,
    ) -> InternalMethodResult<bool> {
        let Some(index) = key.as_index() else {
            return Ok(true);
        };
        let Some(typed_array) = self.typed_array(id) else {
            return Err(InternalMethodError::CorruptObjectState);
        };
        let length = u32::try_from(typed_array.length()).unwrap_or(u32::MAX);
        if index < length {
            return Ok(false);
        }
        Ok(true)
    }

    pub(super) fn typed_array_own_property_keys(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> InternalMethodResult<Vec<PropertyKey>> {
        let Some(typed_array) = self.typed_array(id) else {
            return Err(InternalMethodError::CorruptObjectState);
        };
        let length = u32::try_from(typed_array.length()).unwrap_or(u32::MAX);
        let mut keys = (0..length).map(PropertyKey::Index).collect::<Vec<_>>();
        let (mut strings, mut symbols) = self.collect_own_named_keys(heap, id)?;
        keys.append(&mut strings);
        keys.append(&mut symbols);
        Ok(keys)
    }
}
