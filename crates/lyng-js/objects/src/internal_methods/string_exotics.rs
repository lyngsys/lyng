use super::*;

impl ObjectRuntime {
    pub(super) fn is_string_exotic_object(&self, id: ObjectRef) -> bool {
        self.primitive_wrapper_kind(id) == Some(crate::PrimitiveWrapperKind::String)
    }

    pub(super) fn string_exotic_get_own_property(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
        key: PropertyKey,
    ) -> InternalMethodResult<Option<PropertyDescriptor>> {
        if let Some(index) = key.as_index() {
            if let Some(descriptor) = self.string_exotic_index_property(heap, id, index)? {
                return Ok(Some(descriptor));
            }
            return self.ordinary_own_index_property(heap, id, index);
        }
        if key.as_atom() == Some(WellKnownAtom::length.id()) {
            return self.string_exotic_length_property(heap, id).map(Some);
        }
        self.ordinary_own_named_property(heap, id, key)
    }

    pub(super) fn string_exotic_define_own_property(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        key: PropertyKey,
        descriptor: PropertyDescriptor,
        lifetime: AllocationLifetime,
    ) -> InternalMethodResult<bool> {
        if let Some(index) = key.as_index() {
            if let Some(current) = self.string_exotic_index_property(heap.view(), id, index)? {
                return validate_descriptor_change(current, descriptor);
            }
            let current = self.ordinary_own_index_property(heap.view(), id, index)?;
            if let Some(current) = current {
                if !validate_descriptor_change(current, descriptor)? {
                    return Ok(false);
                }
            } else if !self.ordinary_is_extensible(id)? {
                return Ok(false);
            }
            return self.ordinary_define_own_index_property(
                heap, id, index, current, descriptor, lifetime,
            );
        }

        if key.as_atom() == Some(WellKnownAtom::length.id()) {
            let current = self.string_exotic_length_property(heap.view(), id)?;
            return validate_descriptor_change(current, descriptor);
        }

        let current = self.ordinary_own_named_property(heap.view(), id, key)?;
        if let Some(current) = current {
            if !validate_descriptor_change(current, descriptor)? {
                return Ok(false);
            }
        } else if !self.ordinary_is_extensible(id)? {
            return Ok(false);
        }
        self.ordinary_define_own_named_property(heap, id, key, current, descriptor, lifetime)
    }

    pub(super) fn string_exotic_delete(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        key: PropertyKey,
    ) -> InternalMethodResult<bool> {
        if let Some(index) = key.as_index() {
            if self
                .string_exotic_index_property(heap.view(), id, index)?
                .is_some()
            {
                return Ok(false);
            }
            return self.ordinary_delete(heap, id, key);
        }
        if key.as_atom() == Some(WellKnownAtom::length.id()) {
            return Ok(false);
        }
        self.ordinary_delete(heap, id, key)
    }

    pub(super) fn string_exotic_own_property_keys(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> InternalMethodResult<Vec<PropertyKey>> {
        let length = self.string_exotic_code_unit_len(heap, id)?;
        let mut keys = (0..length).map(PropertyKey::Index).collect::<Vec<_>>();
        let mut extra_indices = self.collect_own_element_keys(heap, id)?;
        extra_indices.retain(|key| key.as_index().is_some_and(|index| index >= length));
        keys.append(&mut extra_indices);

        let (mut strings, mut symbols) = self.collect_own_named_keys(heap, id)?;
        strings.retain(|key| key.as_atom() != Some(WellKnownAtom::length.id()));
        keys.push(PropertyKey::from_atom(WellKnownAtom::length.id()));
        keys.append(&mut strings);
        keys.append(&mut symbols);
        Ok(keys)
    }

    fn string_exotic_code_unit_len(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> InternalMethodResult<u32> {
        let value = self
            .primitive_wrapper_value(heap, id)
            .ok_or(InternalMethodError::CorruptObjectState)?;
        let string = value
            .as_string_ref()
            .ok_or(InternalMethodError::CorruptObjectState)?;
        let view = heap
            .string_view(string)
            .ok_or(InternalMethodError::CorruptObjectState)?;
        Ok(view.code_unit_len())
    }

    fn string_exotic_length_property(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> InternalMethodResult<PropertyDescriptor> {
        let length = self.string_exotic_code_unit_len(heap, id)?;
        let mut descriptor = PropertyDescriptor::new();
        descriptor.set_value(if let Ok(length) = i32::try_from(length) {
            Value::from_smi(length)
        } else {
            Value::from_f64(f64::from(length))
        });
        descriptor.set_writable(false);
        descriptor.set_enumerable(false);
        descriptor.set_configurable(false);
        Ok(descriptor)
    }

    fn string_exotic_index_property(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
        index: u32,
    ) -> InternalMethodResult<Option<PropertyDescriptor>> {
        if index >= self.string_exotic_code_unit_len(heap, id)? {
            return Ok(None);
        }
        let descriptor = self
            .ordinary_own_index_property(heap, id, index)?
            .ok_or(InternalMethodError::CorruptObjectState)?;
        let value = descriptor
            .value()
            .ok_or(InternalMethodError::CorruptObjectState)?;
        let mut synthetic = PropertyDescriptor::new();
        synthetic.set_value(value);
        synthetic.set_writable(false);
        synthetic.set_enumerable(true);
        synthetic.set_configurable(false);
        Ok(Some(synthetic))
    }
}
