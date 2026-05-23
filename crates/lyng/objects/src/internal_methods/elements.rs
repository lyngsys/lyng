use super::*;

impl ObjectRuntime {
    pub fn init_element(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        index: u32,
        value: Value,
    ) -> bool {
        let Some(record) = heap.view().object(id) else {
            return false;
        };
        if self.object_metadata(id).is_none() {
            return false;
        }
        let Some(elements) = record.elements() else {
            return false;
        };
        let stored = heap.init_store_value(ValueStoreTarget::ObjectSlot(elements, index), value);
        if stored {
            self.update_dense_element_len(id, index.saturating_add(1));
        }
        stored
    }

    pub fn mut_element(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        index: u32,
        value: Value,
    ) -> bool {
        let Some(record) = heap.view().object(id) else {
            return false;
        };
        if self.object_metadata(id).is_none() {
            return false;
        }
        let Some(elements) = record.elements() else {
            return false;
        };
        let stored = heap.mut_store_value(ValueStoreTarget::ObjectSlot(elements, index), value);
        if stored {
            self.update_dense_element_len(id, index.saturating_add(1));
        }
        stored
    }

    pub fn define_element(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        index: u32,
        value: Value,
        attrs: DescriptorAttributes,
        lifetime: AllocationLifetime,
    ) -> bool {
        if value == Value::array_hole() {
            return self.delete_element(heap, id, index);
        }

        let Some(record) = heap.view().object(id) else {
            return false;
        };
        let mode = match self.object_metadata(id) {
            Some(metadata) => metadata.element_storage.mode(),
            None => return false,
        };

        let updated = match mode {
            ElementMode::Empty => {
                self.install_empty_element(heap, id, index, value, attrs, lifetime)
            }
            ElementMode::Dense => {
                self.store_dense_element(heap, id, record, index, value, attrs, lifetime)
            }
            ElementMode::Sparse => self.store_sparse_element(id, index, value, attrs),
        };
        updated && self.refresh_integrity_level_flags(heap.view(), id)
    }

    #[inline]
    pub fn set_element(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        index: u32,
        value: Value,
        lifetime: AllocationLifetime,
    ) -> bool {
        self.define_element(heap, id, index, value, ordinary_property_attrs(), lifetime)
    }

    pub fn delete_element(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        index: u32,
    ) -> bool {
        let Some(record) = heap.view().object(id) else {
            return false;
        };
        let Some(metadata) = self.object_metadata_mut(id) else {
            return false;
        };

        let updated = match &mut metadata.element_storage {
            ElementStorageMetadata::Empty => true,
            ElementStorageMetadata::Dense { logical_len } => {
                let Some(elements) = record.elements() else {
                    return false;
                };
                let current_len = *logical_len;
                if index >= current_len {
                    return true;
                }
                if !heap.mut_store_value(
                    ValueStoreTarget::ObjectSlot(elements, index),
                    Value::array_hole(),
                ) {
                    return false;
                }

                let Some(buffer) = heap.view().object_slots(elements) else {
                    return false;
                };
                let trimmed = trim_dense_logical_len(buffer, current_len);
                if trimmed == 0 {
                    metadata.element_storage = ElementStorageMetadata::Empty;
                    let cleared = heap.mut_store_object_slots_handle(
                        ObjectSlotsHandleStoreTarget::ObjectElements(id),
                        None,
                    );
                    return cleared && self.refresh_integrity_level_flags(heap.view(), id);
                }

                *logical_len = trimmed;
                true
            }
            ElementStorageMetadata::Sparse {
                entries,
                logical_len,
            } => {
                entries.remove(&index);
                if entries.is_empty() {
                    metadata.element_storage = ElementStorageMetadata::Empty;
                    let cleared = heap.mut_store_object_slots_handle(
                        ObjectSlotsHandleStoreTarget::ObjectElements(id),
                        None,
                    );
                    return cleared && self.refresh_integrity_level_flags(heap.view(), id);
                }
                if index.saturating_add(1) >= *logical_len {
                    *logical_len = entries
                        .keys()
                        .copied()
                        .max()
                        .map_or(0, |last| last.saturating_add(1));
                }
                true
            }
        };
        updated && self.refresh_integrity_level_flags(heap.view(), id)
    }

    pub(super) fn ordinary_own_index_property(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
        index: u32,
    ) -> InternalMethodResult<Option<PropertyDescriptor>> {
        let record = heap.object(id).ok_or(InternalMethodError::MissingObject)?;
        let metadata = self
            .object_metadata(id)
            .ok_or(InternalMethodError::MissingObject)?;
        match &metadata.element_storage {
            ElementStorageMetadata::Empty => Ok(None),
            ElementStorageMetadata::Dense { logical_len } => {
                if index >= *logical_len {
                    return Ok(None);
                }
                let Some(elements) = record.elements() else {
                    return Err(InternalMethodError::CorruptObjectState);
                };
                let Some(buffer) = heap.object_slots(elements) else {
                    return Err(InternalMethodError::CorruptObjectState);
                };
                let value = buffer
                    .get(index as usize)
                    .copied()
                    .unwrap_or(Value::array_hole());
                if value == Value::array_hole() {
                    Ok(None)
                } else {
                    Ok(Some(descriptor_from_payload(
                        NamedPropertyValue::data(value),
                        ordinary_property_attrs(),
                    )))
                }
            }
            ElementStorageMetadata::Sparse { entries, .. } => Ok(entries
                .get(&index)
                .copied()
                .map(|entry| descriptor_from_payload(entry.payload(), entry.attrs()))),
        }
    }

    /// Return a fast-path own data value for an ordinary indexed property when possible.
    ///
    /// # Errors
    /// Returns [`InternalMethodError`] when object metadata or dense element storage is corrupt.
    pub fn fast_own_index_data_value(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
        index: u32,
    ) -> InternalMethodResult<Option<Value>> {
        if !self.can_fast_query_ordinary_index(id)? {
            return Ok(None);
        }
        let record = heap.object(id).ok_or(InternalMethodError::MissingObject)?;
        let metadata = self
            .object_metadata(id)
            .ok_or(InternalMethodError::MissingObject)?;
        match &metadata.element_storage {
            ElementStorageMetadata::Empty => Ok(None),
            ElementStorageMetadata::Dense { logical_len } => {
                if index >= *logical_len {
                    return Ok(None);
                }
                let Some(elements) = record.elements() else {
                    return Err(InternalMethodError::CorruptObjectState);
                };
                let Some(buffer) = heap.object_slots(elements) else {
                    return Err(InternalMethodError::CorruptObjectState);
                };
                let value = buffer
                    .get(index as usize)
                    .copied()
                    .unwrap_or(Value::array_hole());
                Ok((value != Value::array_hole()).then_some(value))
            }
            ElementStorageMetadata::Sparse { entries, .. } => Ok(entries
                .get(&index)
                .copied()
                .and_then(SparseElementEntry::data_value)),
        }
    }

    /// Return whether an ordinary object has an own indexed property when the fast path applies.
    ///
    /// # Errors
    /// Returns [`InternalMethodError`] when object metadata or dense element storage is corrupt.
    pub fn fast_has_own_index_property(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
        index: u32,
    ) -> InternalMethodResult<Option<bool>> {
        if !self.can_fast_query_ordinary_index(id)? {
            return Ok(None);
        }
        let record = heap.object(id).ok_or(InternalMethodError::MissingObject)?;
        let metadata = self
            .object_metadata(id)
            .ok_or(InternalMethodError::MissingObject)?;
        match &metadata.element_storage {
            ElementStorageMetadata::Empty => Ok(Some(false)),
            ElementStorageMetadata::Dense { logical_len } => {
                if index >= *logical_len {
                    return Ok(Some(false));
                }
                let Some(elements) = record.elements() else {
                    return Err(InternalMethodError::CorruptObjectState);
                };
                let Some(buffer) = heap.object_slots(elements) else {
                    return Err(InternalMethodError::CorruptObjectState);
                };
                let value = buffer
                    .get(index as usize)
                    .copied()
                    .unwrap_or(Value::array_hole());
                Ok(Some(value != Value::array_hole()))
            }
            ElementStorageMetadata::Sparse { entries, .. } => {
                Ok(Some(entries.contains_key(&index)))
            }
        }
    }

    /// Fast-path indexed assignment for ordinary objects with ordinary data elements.
    ///
    /// This intentionally excludes engine arrays, typed arrays, string exotics, module namespace
    /// objects, proxies, and sparse elements because those paths carry additional observable
    /// semantics or descriptor attributes.
    ///
    /// # Errors
    /// Returns [`InternalMethodError`] when object metadata or dense element storage is corrupt.
    pub fn fast_set_ordinary_index_data_property(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        index: u32,
        value: Value,
        lifetime: AllocationLifetime,
    ) -> InternalMethodResult<Option<bool>> {
        if value == Value::array_hole() {
            return Ok(None);
        }
        let record = heap
            .view()
            .object(id)
            .ok_or(InternalMethodError::MissingObject)?;
        let metadata = self
            .object_metadata(id)
            .ok_or(InternalMethodError::MissingObject)?;
        if !matches!(metadata.kind, ObjectKind::Ordinary | ObjectKind::Function)
            || metadata.flags.is_engine_array()
            || self.is_module_namespace_object(id)
            || self.is_string_exotic_object(id)
            || self.is_typed_array_object(id)
        {
            return Ok(None);
        }

        match &metadata.element_storage {
            ElementStorageMetadata::Dense { logical_len } if index < *logical_len => {
                let Some(elements) = record.elements() else {
                    return Err(InternalMethodError::CorruptObjectState);
                };
                let Some(buffer) = heap.view().object_slots(elements) else {
                    return Err(InternalMethodError::CorruptObjectState);
                };
                let current = buffer
                    .get(index as usize)
                    .copied()
                    .unwrap_or(Value::array_hole());
                if current != Value::array_hole() {
                    return if heap
                        .mut_store_value(ValueStoreTarget::ObjectSlot(elements, index), value)
                    {
                        Ok(Some(true))
                    } else {
                        Err(InternalMethodError::CorruptObjectState)
                    };
                }
            }
            ElementStorageMetadata::Sparse { .. } => return Ok(None),
            ElementStorageMetadata::Empty | ElementStorageMetadata::Dense { .. } => {}
        }

        if !metadata.flags.is_extensible() {
            return Ok(Some(false));
        }
        match &metadata.element_storage {
            ElementStorageMetadata::Empty if index > DENSE_ELEMENT_SPARSE_GAP_THRESHOLD => {
                return Ok(None);
            }
            ElementStorageMetadata::Dense { logical_len }
                if index > logical_len.saturating_add(DENSE_ELEMENT_SPARSE_GAP_THRESHOLD) =>
            {
                return Ok(None);
            }
            ElementStorageMetadata::Sparse { .. } => return Ok(None),
            ElementStorageMetadata::Empty | ElementStorageMetadata::Dense { .. } => {}
        }

        if self.set_element(heap, id, index, value, lifetime) {
            Ok(Some(true))
        } else {
            Err(InternalMethodError::CorruptObjectState)
        }
    }

    fn can_fast_query_ordinary_index(&self, id: ObjectRef) -> InternalMethodResult<bool> {
        match self.require_object_kind(id)? {
            ObjectKind::Proxy => Ok(false),
            ObjectKind::Ordinary | ObjectKind::Function => Ok(!self.is_module_namespace_object(id)
                && !self.is_string_exotic_object(id)
                && !self.is_typed_array_object(id)),
        }
    }

    pub(super) fn ordinary_define_own_index_property(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        index: u32,
        current: Option<PropertyDescriptor>,
        descriptor: PropertyDescriptor,
        lifetime: AllocationLifetime,
    ) -> InternalMethodResult<bool> {
        let (payload, attrs) = complete_descriptor_update(current, descriptor)?;
        if payload.kind() == ShapePropertyKind::Data && attrs == ordinary_property_attrs() {
            let value = payload
                .data_value()
                .ok_or(InternalMethodError::InvalidDescriptor)?;
            return if self.set_element(heap, id, index, value, lifetime) {
                Ok(true)
            } else {
                Err(InternalMethodError::CorruptObjectState)
            };
        }

        self.ordinary_store_sparse_index_property(heap, id, index, payload, attrs)
    }

    pub(super) fn ordinary_store_sparse_index_property(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        index: u32,
        payload: NamedPropertyValue,
        attrs: DescriptorAttributes,
    ) -> InternalMethodResult<bool> {
        match self
            .element_mode(id)
            .ok_or(InternalMethodError::MissingObject)?
        {
            ElementMode::Sparse => {
                if self.store_sparse_element_payload(id, index, payload, attrs) {
                    Ok(true)
                } else {
                    Err(InternalMethodError::CorruptObjectState)
                }
            }
            ElementMode::Empty | ElementMode::Dense => {
                if self.transition_elements_to_sparse_payload(heap, id, None, index, payload, attrs)
                {
                    Ok(true)
                } else {
                    Err(InternalMethodError::CorruptObjectState)
                }
            }
        }
    }

    pub(super) fn collect_own_element_keys(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> InternalMethodResult<Vec<PropertyKey>> {
        let record = heap.object(id).ok_or(InternalMethodError::MissingObject)?;
        let metadata = self
            .object_metadata(id)
            .ok_or(InternalMethodError::MissingObject)?;
        let mut keys = Vec::new();
        match &metadata.element_storage {
            ElementStorageMetadata::Empty => {}
            ElementStorageMetadata::Dense { logical_len } => {
                let Some(elements) = record.elements() else {
                    return Err(InternalMethodError::CorruptObjectState);
                };
                let Some(buffer) = heap.object_slots(elements) else {
                    return Err(InternalMethodError::CorruptObjectState);
                };
                for index in 0..*logical_len {
                    let value = buffer
                        .get(index as usize)
                        .copied()
                        .unwrap_or(Value::array_hole());
                    if value != Value::array_hole() {
                        keys.push(PropertyKey::Index(index));
                    }
                }
            }
            ElementStorageMetadata::Sparse { entries, .. } => {
                let mut indices = entries.keys().copied().collect::<Vec<_>>();
                indices.sort_unstable();
                keys.extend(indices.into_iter().map(PropertyKey::Index));
            }
        }
        Ok(keys)
    }

    pub(super) fn collect_own_element_indices_descending_from(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
        min_index: u32,
        max_exclusive: u32,
    ) -> InternalMethodResult<Vec<u32>> {
        let record = heap.object(id).ok_or(InternalMethodError::MissingObject)?;
        let metadata = self
            .object_metadata(id)
            .ok_or(InternalMethodError::MissingObject)?;
        let mut indices = Vec::new();
        match &metadata.element_storage {
            ElementStorageMetadata::Empty => {}
            ElementStorageMetadata::Dense { logical_len } => {
                let upper_bound = (*logical_len).min(max_exclusive);
                if min_index >= upper_bound {
                    return Ok(indices);
                }
                let Some(elements) = record.elements() else {
                    return Err(InternalMethodError::CorruptObjectState);
                };
                let Some(buffer) = heap.object_slots(elements) else {
                    return Err(InternalMethodError::CorruptObjectState);
                };
                for index in (min_index..upper_bound).rev() {
                    let value = buffer
                        .get(index as usize)
                        .copied()
                        .unwrap_or(Value::array_hole());
                    if value != Value::array_hole() {
                        indices.push(index);
                    }
                }
            }
            ElementStorageMetadata::Sparse { entries, .. } => {
                indices.extend(
                    entries
                        .keys()
                        .copied()
                        .filter(|index| *index >= min_index && *index < max_exclusive),
                );
                indices.sort_unstable_by(|left, right| right.cmp(left));
            }
        }
        Ok(indices)
    }
}
