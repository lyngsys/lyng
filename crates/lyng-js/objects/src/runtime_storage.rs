use super::object_metadata::ObjectKindPayload;
use super::{
    dense_element_growth_capacity, ordinary_property_attrs, AllocationLifetime, ClassRecord,
    DescriptorAttributes, ElementStorageMetadata, InvalidationCause, InvalidationEvent,
    ModuleNamespaceObject, NamedPropertyDictionary, NamedPropertyDictionaryEntry,
    NamedPropertyValue, ObjectMetadata, ObjectRef, ObjectRuntime, ObjectSlotsHandleStoreTarget,
    PrimitiveHeapView, PrimitiveMutator, RegExpPayload, RuntimeObjectRecord, ShapeId,
    ShapeMetadata, ShapePropertyKind, SlotLocation, SparseElementEntry, TemporalObjectData, Value,
    ValueStoreTarget, DENSE_ELEMENT_SPARSE_GAP_THRESHOLD,
};
use std::collections::HashMap;

impl ObjectRuntime {
    pub(crate) fn store_object_metadata(&mut self, id: ObjectRef, metadata: ObjectMetadata) {
        let index = object_index(id);
        if self.object_metadata.len() <= index {
            self.object_metadata.resize_with(index + 1, || None);
        }
        self.object_metadata[index] = Some(metadata);
    }

    pub(crate) fn store_map_slot(&mut self, id: ObjectRef, map: super::MapObjectData) {
        self.set_payload(id, ObjectKindPayload::Map(Box::new(map)));
    }

    pub(crate) fn store_set_slot(&mut self, id: ObjectRef, set: super::SetObjectData) {
        self.set_payload(id, ObjectKindPayload::Set(Box::new(set)));
    }

    pub(crate) fn store_array_buffer_slot(
        &mut self,
        id: ObjectRef,
        array_buffer: super::ArrayBufferObjectData,
    ) {
        self.set_payload(id, ObjectKindPayload::ArrayBuffer(array_buffer));
    }

    pub(crate) fn store_data_view_slot(
        &mut self,
        id: ObjectRef,
        data_view: super::DataViewObjectData,
    ) {
        self.set_payload(id, ObjectKindPayload::DataView(data_view));
    }

    pub(crate) fn store_typed_array_slot(
        &mut self,
        id: ObjectRef,
        typed_array: super::TypedArrayObjectData,
    ) {
        // TypedArray retains its own contiguous side table; see ObjectKindPayload docs
        // for why this kind opts out of the unified payload.
        let index = object_index(id);
        if self.typed_arrays.len() <= index {
            self.typed_arrays.resize_with(index + 1, || None);
        }
        self.typed_arrays[index] = Some(typed_array);
    }

    pub(crate) fn store_temporal_object_slot(
        &mut self,
        id: ObjectRef,
        payload: TemporalObjectData,
    ) {
        self.set_payload(id, ObjectKindPayload::Temporal(Box::new(payload)));
    }

    pub(crate) fn object_metadata(&self, id: ObjectRef) -> Option<&ObjectMetadata> {
        self.object_metadata.get(object_index(id))?.as_ref()
    }

    pub(crate) fn object_metadata_mut(&mut self, id: ObjectRef) -> Option<&mut ObjectMetadata> {
        self.object_metadata.get_mut(object_index(id))?.as_mut()
    }

    pub(crate) fn take_object_metadata(&mut self, id: ObjectRef) -> Option<ObjectMetadata> {
        self.object_metadata.get_mut(object_index(id))?.take()
    }

    pub(crate) fn store_regexp_payload_slot(&mut self, id: ObjectRef, payload: RegExpPayload) {
        self.set_payload(id, ObjectKindPayload::RegExp(Box::new(payload)));
    }

    pub(crate) fn store_generator_state_slot(
        &mut self,
        id: ObjectRef,
        state: super::GeneratorState,
    ) {
        self.set_payload(id, ObjectKindPayload::Generator(state));
    }

    pub(crate) fn store_class_record_slot(&mut self, id: ObjectRef, record: ClassRecord) {
        self.set_payload(id, ObjectKindPayload::ClassRecord(Box::new(record)));
    }

    pub(crate) fn store_module_namespace_slot(
        &mut self,
        id: ObjectRef,
        namespace: ModuleNamespaceObject,
    ) {
        self.set_payload(id, ObjectKindPayload::ModuleNamespace(Box::new(namespace)));
    }

    /// Internal helper: write the kind payload on an object's metadata. Silently no-ops if
    /// the object has no metadata (no allocation). Preserves the prior `store_*_slot`
    /// semantics of "replace whatever is there with this payload".
    #[inline]
    fn set_payload(&mut self, id: ObjectRef, payload: ObjectKindPayload) {
        if let Some(metadata) = self.object_metadata_mut(id) {
            metadata.payload = Some(payload);
        }
    }

    pub(crate) fn class_record_slot(&self, id: ObjectRef) -> Option<&ClassRecord> {
        match self.object_metadata(id)?.payload.as_ref()? {
            ObjectKindPayload::ClassRecord(record) => Some(record.as_ref()),
            _ => None,
        }
    }

    pub(crate) fn module_namespace_slot(&self, id: ObjectRef) -> Option<&ModuleNamespaceObject> {
        match self.object_metadata(id)?.payload.as_ref()? {
            ObjectKindPayload::ModuleNamespace(namespace) => Some(namespace.as_ref()),
            _ => None,
        }
    }

    pub(crate) fn map_slot(&self, id: ObjectRef) -> Option<&super::MapObjectData> {
        match self.object_metadata(id)?.payload.as_ref()? {
            ObjectKindPayload::Map(data) => Some(data.as_ref()),
            _ => None,
        }
    }

    pub(crate) fn map_slot_mut(&mut self, id: ObjectRef) -> Option<&mut super::MapObjectData> {
        match self.object_metadata_mut(id)?.payload.as_mut()? {
            ObjectKindPayload::Map(data) => Some(data.as_mut()),
            _ => None,
        }
    }

    pub(crate) fn set_slot(&self, id: ObjectRef) -> Option<&super::SetObjectData> {
        match self.object_metadata(id)?.payload.as_ref()? {
            ObjectKindPayload::Set(data) => Some(data.as_ref()),
            _ => None,
        }
    }

    pub(crate) fn set_slot_mut(&mut self, id: ObjectRef) -> Option<&mut super::SetObjectData> {
        match self.object_metadata_mut(id)?.payload.as_mut()? {
            ObjectKindPayload::Set(data) => Some(data.as_mut()),
            _ => None,
        }
    }

    pub(crate) fn array_buffer_slot(&self, id: ObjectRef) -> Option<super::ArrayBufferObjectData> {
        match self.object_metadata(id)?.payload.as_ref()? {
            ObjectKindPayload::ArrayBuffer(data) => Some(*data),
            _ => None,
        }
    }

    pub(crate) fn data_view_slot(&self, id: ObjectRef) -> Option<super::DataViewObjectData> {
        match self.object_metadata(id)?.payload.as_ref()? {
            ObjectKindPayload::DataView(data) => Some(*data),
            _ => None,
        }
    }

    pub(crate) fn typed_array_slot(&self, id: ObjectRef) -> Option<super::TypedArrayObjectData> {
        self.typed_arrays.get(object_index(id)).copied().flatten()
    }

    pub(crate) fn take_typed_array_slot(
        &mut self,
        id: ObjectRef,
    ) -> Option<super::TypedArrayObjectData> {
        self.typed_arrays.get_mut(object_index(id))?.take()
    }

    pub(crate) fn temporal_object_slot(&self, id: ObjectRef) -> Option<&TemporalObjectData> {
        match self.object_metadata(id)?.payload.as_ref()? {
            ObjectKindPayload::Temporal(data) => Some(data.as_ref()),
            _ => None,
        }
    }

    pub(crate) fn regexp_payload_slot(&self, id: ObjectRef) -> Option<&RegExpPayload> {
        match self.object_metadata(id)?.payload.as_ref()? {
            ObjectKindPayload::RegExp(data) => Some(data.as_ref()),
            _ => None,
        }
    }

    pub(crate) fn generator_state_slot(&self, id: ObjectRef) -> Option<super::GeneratorState> {
        match self.object_metadata(id)?.payload.as_ref()? {
            ObjectKindPayload::Generator(state) => Some(*state),
            _ => None,
        }
    }

    pub(crate) fn store_shape_metadata(&mut self, id: ShapeId, metadata: ShapeMetadata) {
        let index = shape_index(id);
        if self.shape_metadata.len() <= index {
            self.shape_metadata.resize_with(index + 1, || None);
        }
        self.shape_metadata[index] = Some(metadata);
    }

    pub(crate) fn shape_metadata(&self, id: ShapeId) -> Option<&ShapeMetadata> {
        self.shape_metadata.get(shape_index(id))?.as_ref()
    }

    pub(crate) fn shape_metadata_mut(&mut self, id: ShapeId) -> Option<&mut ShapeMetadata> {
        self.shape_metadata.get_mut(shape_index(id))?.as_mut()
    }

    pub(crate) fn take_shape_metadata(&mut self, id: ShapeId) -> Option<ShapeMetadata> {
        self.shape_metadata.get_mut(shape_index(id))?.take()
    }

    pub(crate) fn update_dense_element_len(&mut self, id: ObjectRef, min_len: u32) {
        let Some(metadata) = self.object_metadata_mut(id) else {
            return;
        };
        if let ElementStorageMetadata::Dense { logical_len } = &mut metadata.element_storage {
            *logical_len = (*logical_len).max(min_len);
        }
    }

    pub(crate) fn bump_invalidation(&mut self, id: ObjectRef, cause: InvalidationCause) -> bool {
        self.next_invalidation_epoch = self.next_invalidation_epoch.saturating_add(1);
        let epoch = self.next_invalidation_epoch;
        let Some(metadata) = self.object_metadata_mut(id) else {
            return false;
        };
        metadata.last_invalidation = Some(InvalidationEvent::new(epoch, cause));
        true
    }

    pub(crate) fn snapshot_named_property_dictionary(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> NamedPropertyDictionary {
        let Some(record) = heap.object(id) else {
            return NamedPropertyDictionary::new(HashMap::new(), 0);
        };
        let shape = record
            .shape()
            .and_then(|shape| self.shape_properties(shape))
            .unwrap_or(&[]);
        // Inline slots are packed directly into the heap-side object record; out-of-line
        // slots live in the heap-allocated `NamedSlotStorage`. Read each property's
        // payload through the encoded `slot_offset` so dictionary entries snapshot from
        // whichever tier the slot currently uses.
        let named_slots = record
            .named_slots()
            .and_then(|slots| heap.object_slots(slots))
            .unwrap_or(&[]);
        let read_slot = |location: SlotLocation| match location {
            SlotLocation::Inline(index) => record
                .inline_named_slot(index as usize)
                .unwrap_or(Value::empty_internal_slot()),
            SlotLocation::OutOfLine(offset) => named_slots
                .get(offset as usize)
                .copied()
                .unwrap_or(Value::empty_internal_slot()),
        };

        let mut entries = HashMap::with_capacity(shape.len());
        let mut next_index = 0;
        for property in shape {
            let primary = property.slot_location();
            let payload = match property.kind() {
                ShapePropertyKind::Data => NamedPropertyValue::data(read_slot(primary)),
                ShapePropertyKind::Accessor => NamedPropertyValue::accessor(
                    read_slot(primary),
                    read_slot(primary.accessor_setter_location()),
                ),
            };
            next_index = next_index.max(property.enumeration_index().saturating_add(1));
            entries.insert(
                property.key(),
                NamedPropertyDictionaryEntry::new(
                    property.key(),
                    property.attrs(),
                    payload,
                    property.enumeration_index(),
                ),
            );
        }

        NamedPropertyDictionary::new(entries, next_index)
    }

    pub(crate) fn install_empty_element(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        index: u32,
        value: Value,
        attrs: DescriptorAttributes,
        lifetime: AllocationLifetime,
    ) -> bool {
        if index > DENSE_ELEMENT_SPARSE_GAP_THRESHOLD {
            return self.transition_elements_to_sparse_payload(
                heap,
                id,
                None,
                index,
                NamedPropertyValue::data(value),
                attrs,
            );
        }

        let capacity = dense_element_growth_capacity(0, index);
        let slots = heap.alloc_object_slots(capacity, Value::array_hole(), lifetime);
        if !heap.init_store_value(ValueStoreTarget::ObjectSlot(slots, index), value) {
            return false;
        }
        if !heap.mut_store_object_slots_handle(
            ObjectSlotsHandleStoreTarget::ObjectElements(id),
            Some(slots),
        ) {
            return false;
        }
        let Some(metadata) = self.object_metadata_mut(id) else {
            return false;
        };
        metadata.element_storage = ElementStorageMetadata::Dense {
            logical_len: index.saturating_add(1),
        };
        true
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn store_dense_element(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        record: RuntimeObjectRecord,
        index: u32,
        value: Value,
        attrs: DescriptorAttributes,
        lifetime: AllocationLifetime,
    ) -> bool {
        let Some(elements) = record.elements() else {
            return false;
        };
        let capacity = heap.view().object_slots(elements).map_or(0, <[Value]>::len);
        let logical_len = self.element_logical_len(id).unwrap_or(0);
        if index > logical_len.saturating_add(DENSE_ELEMENT_SPARSE_GAP_THRESHOLD) {
            return self.transition_elements_to_sparse_payload(
                heap,
                id,
                Some(record),
                index,
                NamedPropertyValue::data(value),
                attrs,
            );
        }

        if (index as usize) < capacity {
            if !heap.mut_store_value(ValueStoreTarget::ObjectSlot(elements, index), value) {
                return false;
            }
            self.update_dense_element_len(id, index.saturating_add(1));
            return true;
        }

        let previous = heap
            .view()
            .object_slots(elements)
            .map(<[Value]>::to_vec)
            .unwrap_or_default();
        let new_capacity = dense_element_growth_capacity(capacity, index);
        let new_slots = heap.alloc_object_slots(new_capacity, Value::array_hole(), lifetime);
        for (slot_index, slot_value) in previous.iter().copied().enumerate() {
            let slot_index = u32::try_from(slot_index).expect("dense slot index must fit into u32");
            if !heap.init_store_value(
                ValueStoreTarget::ObjectSlot(new_slots, slot_index),
                slot_value,
            ) {
                return false;
            }
        }
        if !heap.init_store_value(ValueStoreTarget::ObjectSlot(new_slots, index), value) {
            return false;
        }
        if !heap.mut_store_object_slots_handle(
            ObjectSlotsHandleStoreTarget::ObjectElements(id),
            Some(new_slots),
        ) {
            return false;
        }
        self.update_dense_element_len(id, index.saturating_add(1));
        true
    }

    pub(crate) fn transition_elements_to_sparse_payload(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        record: Option<RuntimeObjectRecord>,
        index: u32,
        payload: NamedPropertyValue,
        attrs: DescriptorAttributes,
    ) -> bool {
        let record = record.or_else(|| heap.view().object(id));
        let dense_slots = record
            .and_then(RuntimeObjectRecord::elements)
            .and_then(|slots| heap.view().object_slots(slots))
            .map(<[Value]>::to_vec)
            .unwrap_or_default();
        let mut entries = HashMap::new();
        for (entry_index, entry_value) in dense_slots.into_iter().enumerate() {
            if entry_value == Value::array_hole() {
                continue;
            }
            let entry_index =
                u32::try_from(entry_index).expect("dense element index must fit into u32");
            entries.insert(
                entry_index,
                SparseElementEntry::new(
                    NamedPropertyValue::data(entry_value),
                    ordinary_property_attrs(),
                ),
            );
        }
        entries.insert(index, SparseElementEntry::new(payload, attrs));
        let logical_len = entries
            .keys()
            .copied()
            .max()
            .map_or(0, |last| last.saturating_add(1));

        let Some(metadata) = self.object_metadata_mut(id) else {
            return false;
        };
        metadata.element_storage = ElementStorageMetadata::Sparse {
            entries,
            logical_len,
        };
        heap.mut_store_object_slots_handle(ObjectSlotsHandleStoreTarget::ObjectElements(id), None)
    }

    pub(crate) fn store_sparse_element(
        &mut self,
        id: ObjectRef,
        index: u32,
        value: Value,
        attrs: DescriptorAttributes,
    ) -> bool {
        self.store_sparse_element_payload(id, index, NamedPropertyValue::data(value), attrs)
    }

    pub(crate) fn store_sparse_element_payload(
        &mut self,
        id: ObjectRef,
        index: u32,
        payload: NamedPropertyValue,
        attrs: DescriptorAttributes,
    ) -> bool {
        let Some(metadata) = self.object_metadata_mut(id) else {
            return false;
        };
        let ElementStorageMetadata::Sparse {
            entries,
            logical_len,
        } = &mut metadata.element_storage
        else {
            return false;
        };
        entries.insert(index, SparseElementEntry::new(payload, attrs));
        *logical_len = (*logical_len).max(index.saturating_add(1));
        true
    }
}

#[inline]
const fn object_index(id: ObjectRef) -> usize {
    (id.get() - 1) as usize
}

#[inline]
const fn shape_index(id: ShapeId) -> usize {
    (id.get() - 1) as usize
}
