use super::*;

impl ObjectRuntime {
    pub fn note_named_property_addition(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
    ) -> bool {
        let should_transition = {
            let Some(metadata) = self.object_metadata_mut(id) else {
                return false;
            };
            if metadata.named_properties.is_dictionary() {
                return true;
            }
            metadata.named_property_additions = metadata.named_property_additions.saturating_add(1);
            metadata.named_property_additions > NAMED_PROPERTY_ADDITION_CHAIN_DICTIONARY_LIMIT
        };

        if should_transition {
            self.ensure_named_property_dictionary(heap, id)
        } else {
            heap.view().object(id).is_some()
        }
    }

    pub fn init_named_slot(
        &self,
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
        let Some(slots) = record.named_slots() else {
            return false;
        };
        heap.init_store_value(ValueStoreTarget::ObjectSlot(slots, index), value)
    }

    pub fn mut_named_slot(
        &self,
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
        let Some(slots) = record.named_slots() else {
            return false;
        };
        heap.mut_store_value(ValueStoreTarget::ObjectSlot(slots, index), value)
    }

    pub fn note_named_property_churn(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
    ) -> bool {
        let should_transition = {
            let Some(metadata) = self.object_metadata_mut(id) else {
                return false;
            };
            if metadata.named_properties.is_dictionary() {
                return true;
            }
            metadata.named_property_churn = metadata.named_property_churn.saturating_add(1);
            metadata.named_property_churn >= NAMED_PROPERTY_STRUCTURAL_CHURN_DICTIONARY_THRESHOLD
        };

        if should_transition {
            self.ensure_named_property_dictionary(heap, id)
        } else {
            heap.view().object(id).is_some()
        }
    }

    pub fn ensure_named_property_dictionary(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
    ) -> bool {
        let Some(record) = heap.view().object(id) else {
            return false;
        };
        let Some(metadata) = self.object_metadata(id) else {
            return false;
        };
        if metadata.named_properties.is_dictionary() {
            return true;
        }

        let preserve_named_slots = self.has_reserved_named_slots(heap.view(), record);
        let dictionary = self.snapshot_named_property_dictionary(heap.view(), record);
        let Some(metadata) = self.object_metadata_mut(id) else {
            return false;
        };
        metadata.named_properties = NamedPropertyStorage::Dictionary(dictionary);
        metadata.flags = metadata
            .flags
            .union(ObjectFlags::NAMED_PROPERTIES_DICTIONARY);
        if !preserve_named_slots
            && !heap.mut_store_object_slots_handle(
                ObjectSlotsHandleStoreTarget::ObjectNamedSlots(id),
                None,
            )
        {
            return false;
        }
        self.bump_invalidation(id, InvalidationCause::DictionaryTransition)
    }

    pub fn redefine_named_property(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        key: PropertyKey,
        payload: NamedPropertyValue,
        attrs: DescriptorAttributes,
    ) -> bool {
        if !self.ensure_named_property_dictionary(heap, id) {
            return false;
        }

        let Some(metadata) = self.object_metadata_mut(id) else {
            return false;
        };
        let NamedPropertyStorage::Dictionary(dictionary) = &mut metadata.named_properties else {
            return false;
        };
        dictionary.upsert(key, payload, attrs);
        self.bump_invalidation(id, InvalidationCause::PropertyRedefinition)
            && self.refresh_integrity_level_flags(heap.view(), id)
    }

    pub fn delete_named_property(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        key: PropertyKey,
    ) -> bool {
        let Some(record) = heap.view().object(id) else {
            return false;
        };
        let Some(metadata) = self.object_metadata(id) else {
            return false;
        };
        let present = match &metadata.named_properties {
            NamedPropertyStorage::ShapeStable => record
                .shape()
                .and_then(|shape| self.shape_property(shape, key))
                .is_some(),
            NamedPropertyStorage::Dictionary(dictionary) => dictionary.entry(key).is_some(),
        };
        if !present {
            return false;
        }
        if !self.ensure_named_property_dictionary(heap, id) {
            return false;
        }

        let Some(metadata) = self.object_metadata_mut(id) else {
            return false;
        };
        let NamedPropertyStorage::Dictionary(dictionary) = &mut metadata.named_properties else {
            return false;
        };
        if dictionary.remove(key).is_none() {
            return false;
        }
        metadata.named_property_churn = metadata.named_property_churn.saturating_add(1);
        self.bump_invalidation(id, InvalidationCause::PropertyDeletion)
            && self.refresh_integrity_level_flags(heap.view(), id)
    }

    pub(super) fn ordinary_own_named_property(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
        key: PropertyKey,
    ) -> InternalMethodResult<Option<PropertyDescriptor>> {
        let record = heap.object(id).ok_or(InternalMethodError::MissingObject)?;
        let metadata = self
            .object_metadata(id)
            .ok_or(InternalMethodError::MissingObject)?;
        match &metadata.named_properties {
            NamedPropertyStorage::ShapeStable => {
                let Some(shape) = record.shape() else {
                    return Err(InternalMethodError::CorruptObjectState);
                };
                let Some(property) = self.shape_property(shape, key) else {
                    return Ok(None);
                };
                let descriptor = Self::descriptor_from_shape_property(heap, record, property)?;
                Ok(Some(descriptor))
            }
            NamedPropertyStorage::Dictionary(dictionary) => Ok(dictionary
                .entry(key)
                .map(|entry| descriptor_from_payload(entry.payload(), entry.attrs()))),
        }
    }

    pub(super) fn ordinary_define_own_named_property(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        key: PropertyKey,
        current: Option<PropertyDescriptor>,
        descriptor: PropertyDescriptor,
        lifetime: AllocationLifetime,
    ) -> InternalMethodResult<bool> {
        let payload_and_attrs = complete_descriptor_update(current, descriptor)?;
        let (payload, attrs) = payload_and_attrs;

        let storage_mode = self
            .named_property_storage_mode(id)
            .ok_or(InternalMethodError::MissingObject)?;
        match (storage_mode, current) {
            (NamedPropertyStorageMode::Dictionary, _) => {
                if self.redefine_named_property(heap, id, key, payload, attrs) {
                    Ok(true)
                } else {
                    Err(InternalMethodError::CorruptObjectState)
                }
            }
            (NamedPropertyStorageMode::ShapeStable, None) => self
                .ordinary_define_absent_shaped_named_property(
                    heap, id, key, payload, attrs, lifetime,
                ),
            (NamedPropertyStorageMode::ShapeStable, Some(current)) => self
                .ordinary_update_shaped_named_property(
                    heap, id, key, current, payload, attrs, lifetime,
                ),
        }
    }

    pub(super) fn ordinary_define_absent_shaped_named_property(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        key: PropertyKey,
        payload: NamedPropertyValue,
        attrs: DescriptorAttributes,
        lifetime: AllocationLifetime,
    ) -> InternalMethodResult<bool> {
        let record = heap
            .view()
            .object(id)
            .ok_or(InternalMethodError::MissingObject)?;
        if !self.note_named_property_addition(heap, id) {
            return Err(InternalMethodError::MissingObject);
        }
        if self.named_property_storage_mode(id) == Some(NamedPropertyStorageMode::Dictionary) {
            if self.redefine_named_property(heap, id, key, payload, attrs) {
                return Ok(true);
            }
            return Err(InternalMethodError::CorruptObjectState);
        }
        if self.has_reserved_named_slots(heap.view(), record) {
            if self.redefine_named_property(heap, id, key, payload, attrs) {
                return Ok(true);
            }
            return Err(InternalMethodError::CorruptObjectState);
        }
        let current_shape = record
            .shape()
            .ok_or(InternalMethodError::CorruptObjectState)?;
        if self.shape(heap.view(), current_shape).is_some_and(|shape| {
            shape.property_count() >= NAMED_PROPERTY_ADDITION_CHAIN_DICTIONARY_LIMIT
        }) {
            if self.redefine_named_property(heap, id, key, payload, attrs) {
                return Ok(true);
            }
            return Err(InternalMethodError::CorruptObjectState);
        }
        let transition = ShapeTransitionKey::new(key, payload.kind(), attrs);
        let Some(next_shape) = self.transition_shape(heap, current_shape, transition, lifetime)
        else {
            if self.redefine_named_property(heap, id, key, payload, attrs) {
                return Ok(true);
            }
            return Err(InternalMethodError::CorruptObjectState);
        };

        let old_slots = record
            .named_slots()
            .and_then(|slots| heap.view().object_slots(slots))
            .map(<[Value]>::to_vec)
            .unwrap_or_default();
        let next_shape_record = self
            .shape(heap.view(), next_shape)
            .ok_or(InternalMethodError::CorruptObjectState)?;
        let new_slots = heap.alloc_object_slots(
            next_shape_record.slot_count() as usize,
            Value::empty_internal_slot(),
            lifetime,
        );
        for (index, slot) in old_slots.into_iter().enumerate() {
            let index = u32::try_from(index).expect("named slot index must fit into u32");
            if !heap.init_store_value(ValueStoreTarget::ObjectSlot(new_slots, index), slot) {
                return Err(InternalMethodError::CorruptObjectState);
            }
        }
        let property = self
            .shape_property(next_shape, key)
            .ok_or(InternalMethodError::CorruptObjectState)?;
        write_named_payload(heap, new_slots, property.slot_offset(), payload, true)?;
        if !heap.mut_store_shape_handle(ShapeHandleStoreTarget::ObjectShape(id), Some(next_shape)) {
            return Err(InternalMethodError::CorruptObjectState);
        }
        if !heap.mut_store_object_slots_handle(
            ObjectSlotsHandleStoreTarget::ObjectNamedSlots(id),
            Some(new_slots),
        ) {
            return Err(InternalMethodError::CorruptObjectState);
        }
        Ok(true)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn ordinary_update_shaped_named_property(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        key: PropertyKey,
        current: PropertyDescriptor,
        payload: NamedPropertyValue,
        attrs: DescriptorAttributes,
        _lifetime: AllocationLifetime,
    ) -> InternalMethodResult<bool> {
        let record = heap
            .view()
            .object(id)
            .ok_or(InternalMethodError::MissingObject)?;
        let shape = record
            .shape()
            .ok_or(InternalMethodError::CorruptObjectState)?;
        let property = self
            .shape_property(shape, key)
            .ok_or(InternalMethodError::CorruptObjectState)?;

        let current_payload = payload_from_complete_descriptor(current)?;
        if current_payload.kind() == payload.kind() && property.attrs() == attrs {
            let Some(slots) = record.named_slots() else {
                return Err(InternalMethodError::CorruptObjectState);
            };
            write_named_payload(heap, slots, property.slot_offset(), payload, false)?;
            return Ok(true);
        }

        if self.redefine_named_property(heap, id, key, payload, attrs) {
            Ok(true)
        } else {
            Err(InternalMethodError::CorruptObjectState)
        }
    }

    pub(super) fn collect_own_named_keys(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> InternalMethodResult<(Vec<PropertyKey>, Vec<PropertyKey>)> {
        let record = heap.object(id).ok_or(InternalMethodError::MissingObject)?;
        let metadata = self
            .object_metadata(id)
            .ok_or(InternalMethodError::MissingObject)?;
        let mut strings = Vec::new();
        let mut symbols = Vec::new();

        match &metadata.named_properties {
            NamedPropertyStorage::ShapeStable => {
                let shape = record
                    .shape()
                    .ok_or(InternalMethodError::CorruptObjectState)?;
                for property in self
                    .shape_properties(shape)
                    .ok_or(InternalMethodError::CorruptObjectState)?
                {
                    if property.key().is_symbol() {
                        symbols.push(property.key());
                    } else {
                        strings.push(property.key());
                    }
                }
            }
            NamedPropertyStorage::Dictionary(dictionary) => {
                for entry in dictionary.ordered_entries() {
                    if entry.key().is_symbol() {
                        symbols.push(entry.key());
                    } else {
                        strings.push(entry.key());
                    }
                }
            }
        }

        Ok((strings, symbols))
    }

    fn descriptor_from_shape_property(
        heap: PrimitiveHeapView<'_>,
        object: RuntimeObjectRecord,
        property: ShapeProperty,
    ) -> InternalMethodResult<PropertyDescriptor> {
        let Some(named_slots) = object.named_slots() else {
            return Err(InternalMethodError::CorruptObjectState);
        };
        let Some(slots) = heap.object_slots(named_slots) else {
            return Err(InternalMethodError::CorruptObjectState);
        };
        let payload = match property.kind() {
            ShapePropertyKind::Data => NamedPropertyValue::data(
                slots
                    .get(property.slot_offset() as usize)
                    .copied()
                    .unwrap_or(Value::empty_internal_slot()),
            ),
            ShapePropertyKind::Accessor => {
                let index = property.slot_offset() as usize;
                NamedPropertyValue::accessor(
                    slots
                        .get(index)
                        .copied()
                        .unwrap_or(Value::empty_internal_slot()),
                    slots
                        .get(index + 1)
                        .copied()
                        .unwrap_or(Value::empty_internal_slot()),
                )
            }
        };
        Ok(descriptor_from_payload(payload, property.attrs()))
    }

    fn has_reserved_named_slots(
        &self,
        heap: PrimitiveHeapView<'_>,
        record: RuntimeObjectRecord,
    ) -> bool {
        let shape_slot_count = record
            .shape()
            .and_then(|shape| self.shape(heap, shape))
            .map_or(0, ShapeRecord::slot_count) as usize;
        let named_slot_count = record
            .named_slots()
            .and_then(|slots| heap.object_slots(slots))
            .map_or(0, <[Value]>::len);
        named_slot_count > shape_slot_count
    }
}
