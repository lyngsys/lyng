use super::*;

impl ObjectRuntime {
    pub(super) fn engine_array_define_own_property(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        key: PropertyKey,
        descriptor: PropertyDescriptor,
        lifetime: AllocationLifetime,
    ) -> InternalMethodResult<bool> {
        if key.as_atom() == Some(WellKnownAtom::length.id()) {
            return self.engine_array_define_length_property(heap, id, descriptor, lifetime);
        }
        if let Some(index) = key.as_index() {
            return self.engine_array_define_index_property(heap, id, index, descriptor, lifetime);
        }
        self.ordinary_define_own_property(heap, id, key, descriptor, lifetime)
    }

    fn engine_array_define_length_property(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        descriptor: PropertyDescriptor,
        lifetime: AllocationLifetime,
    ) -> InternalMethodResult<bool> {
        let key = PropertyKey::from_atom(WellKnownAtom::length.id());
        let current = self.ordinary_own_named_property(heap.view(), id, key)?;
        if current.is_none() {
            let descriptor = if descriptor.has_value() {
                let mut normalized = descriptor;
                let length = array_length_from_value(
                    descriptor
                        .value()
                        .ok_or(InternalMethodError::InvalidDescriptor)?,
                )?;
                normalized.set_value(length_value(length));
                normalized
            } else {
                descriptor
            };
            if !self.ordinary_is_extensible(id)? {
                return Ok(false);
            }
            return self
                .ordinary_define_own_named_property(heap, id, key, None, descriptor, lifetime);
        }

        let current = current.expect("checked above");
        if !descriptor.has_value() {
            return self.ordinary_define_own_property(heap, id, key, descriptor, lifetime);
        }

        let (old_len, old_writable) = array_length_descriptor_state(current)?;
        let new_len = array_length_from_value(
            descriptor
                .value()
                .ok_or(InternalMethodError::InvalidDescriptor)?,
        )?;
        let mut normalized = descriptor;
        normalized.set_value(length_value(new_len));

        if new_len >= old_len {
            return self.ordinary_define_own_property(heap, id, key, normalized, lifetime);
        }

        if !old_writable {
            return Ok(false);
        }

        let final_writable = normalized.writable() == Some(false);
        if final_writable {
            normalized.set_writable(true);
        }
        if !self.ordinary_define_own_property(heap, id, key, normalized, lifetime)? {
            return Ok(false);
        }

        for index in
            self.collect_own_element_indices_descending_from(heap.view(), id, new_len, old_len)?
        {
            if !self.ordinary_delete(heap, id, PropertyKey::Index(index))? {
                let _ = self.engine_array_set_length(
                    heap,
                    id,
                    index.saturating_add(1),
                    final_writable.then_some(false),
                    lifetime,
                )?;
                return Ok(false);
            }
        }

        if final_writable {
            let _ = self.engine_array_set_length(heap, id, new_len, Some(false), lifetime)?;
        }
        Ok(true)
    }

    fn engine_array_define_index_property(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        index: u32,
        descriptor: PropertyDescriptor,
        lifetime: AllocationLifetime,
    ) -> InternalMethodResult<bool> {
        let (old_len, old_writable) = self.engine_array_length_state(heap.view(), id)?;
        let current = self.ordinary_own_index_property(heap.view(), id, index)?;
        if let Some(current) = current {
            if !validate_descriptor_change(heap.view(), current, descriptor)? {
                return Ok(false);
            }
        } else {
            if index >= old_len && !old_writable {
                return Ok(false);
            }
            if !self.ordinary_is_extensible(id)? {
                return Ok(false);
            }
        }

        if !self
            .ordinary_define_own_index_property(heap, id, index, current, descriptor, lifetime)?
        {
            return Ok(false);
        }

        if index >= old_len {
            let _ =
                self.engine_array_set_length(heap, id, index.saturating_add(1), None, lifetime)?;
        }
        Ok(true)
    }

    fn engine_array_length_state(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> InternalMethodResult<(u32, bool)> {
        let descriptor = self
            .ordinary_own_named_property(
                heap,
                id,
                PropertyKey::from_atom(WellKnownAtom::length.id()),
            )?
            .ok_or(InternalMethodError::CorruptObjectState)?;
        array_length_descriptor_state(descriptor)
    }

    fn engine_array_set_length(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        length: u32,
        writable: Option<bool>,
        lifetime: AllocationLifetime,
    ) -> InternalMethodResult<bool> {
        let key = PropertyKey::from_atom(WellKnownAtom::length.id());
        let current = self
            .ordinary_own_named_property(heap.view(), id, key)?
            .ok_or(InternalMethodError::CorruptObjectState)?;
        let mut descriptor = PropertyDescriptor::new();
        descriptor.set_value(length_value(length));
        if let Some(writable) = writable {
            descriptor.set_writable(writable);
        }
        self.ordinary_define_own_named_property(heap, id, key, Some(current), descriptor, lifetime)
    }
}

fn length_value(length: u32) -> Value {
    if let Ok(length) = i32::try_from(length) {
        Value::from_smi(length)
    } else {
        Value::from_f64(f64::from(length))
    }
}

fn array_length_from_value(value: Value) -> InternalMethodResult<u32> {
    if let Some(length) = value.as_smi().and_then(|value| u32::try_from(value).ok()) {
        return Ok(length);
    }
    let Some(number) = value.as_f64() else {
        return Err(InternalMethodError::RangeError);
    };
    if !number.is_finite() || number < 0.0 || number.trunc() != number {
        return Err(InternalMethodError::RangeError);
    }
    if number > f64::from(u32::MAX) {
        return Err(InternalMethodError::RangeError);
    }
    Ok(number as u32)
}

fn array_length_descriptor_state(
    descriptor: PropertyDescriptor,
) -> InternalMethodResult<(u32, bool)> {
    let length = array_length_from_value(
        descriptor
            .value()
            .ok_or(InternalMethodError::CorruptObjectState)?,
    )?;
    Ok((length, descriptor.writable().unwrap_or(false)))
}
