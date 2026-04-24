use super::{
    AllocationLifetime, ArrayBufferObjectData, DataViewObjectData, ElementMode,
    ElementStorageMetadata, ElementStorageRef, InvalidationEvent, MapObjectData,
    ModuleNamespaceExport, ModuleNamespaceObject, NamedPropertyDictionaryEntry,
    NamedPropertyStorage, NamedPropertyStorageMode, NamedSlotStorageRef, ObjectAllocation,
    ObjectColdData, ObjectHeader, ObjectMetadata, ObjectRecord, ObjectRef, PrimitiveHeapView,
    PrimitiveMutator, PropertyKey, ProxyObjectData, RegExpPayload, RegExpPayloadAccounting,
    RootShapeKey, RuntimeObjectRecord, RuntimeShapeRecord, SetObjectData, ShapeAllocation, ShapeId,
    ShapeMetadata, ShapeProperty, ShapeRecord, ShapeTransitionKey, SparseElementEntry,
    TemporalObjectData, TemporalObjectKind, TypedArrayObjectData, Value,
};
use std::collections::HashMap;

/// Checked-in threshold before stable shapes switch from inline descriptor scan
/// to a flattened lookup table.
pub const SMALL_SHAPE_INLINE_PROPERTY_LIMIT: usize = 4;
/// Explicit fallback threshold for dynamic named-property churn.
pub const NAMED_PROPERTY_CHURN_DICTIONARY_THRESHOLD: u32 = 3;
/// Dense elements switch to sparse mode when a single write would create a gap
/// beyond this checked-in threshold.
pub const DENSE_ELEMENT_SPARSE_GAP_THRESHOLD: u32 = 16;
pub(crate) const MIN_DENSE_ELEMENT_CAPACITY: usize = 4;
const PROXY_TARGET_SLOT_INDEX: u32 = 0;
const PROXY_HANDLER_SLOT_INDEX: u32 = 1;

/// Object and shape allocation owner for the Phase 3 substrate.
#[derive(Default)]
pub struct ObjectRuntime {
    pub(crate) object_metadata: Vec<Option<ObjectMetadata>>,
    pub(crate) class_records: Vec<Option<super::object_metadata::ClassRecord>>,
    pub(crate) module_namespaces: Vec<Option<ModuleNamespaceObject>>,
    pub(crate) maps: Vec<Option<MapObjectData>>,
    pub(crate) sets: Vec<Option<SetObjectData>>,
    pub(crate) array_buffers: Vec<Option<ArrayBufferObjectData>>,
    pub(crate) data_views: Vec<Option<DataViewObjectData>>,
    pub(crate) typed_arrays: Vec<Option<TypedArrayObjectData>>,
    pub(crate) temporal_objects: Vec<Option<TemporalObjectData>>,
    pub(crate) regexp_payloads: Vec<Option<RegExpPayload>>,
    pub(crate) generator_states: Vec<Option<super::GeneratorState>>,
    pub(crate) shape_metadata: Vec<Option<ShapeMetadata>>,
    pub(crate) root_shapes: HashMap<RootShapeKey, ShapeId>,
    pub(crate) next_private_brand_raw: u32,
    pub(crate) next_invalidation_epoch: u64,
}

impl ObjectRuntime {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Low-level bootstrap shape allocation. Canonical root and transition APIs
    /// should be preferred for normal named-property shape families.
    pub fn alloc_shape(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        allocation: ShapeAllocation,
        lifetime: AllocationLifetime,
    ) -> ShapeId {
        let id = heap.alloc_shape(
            RuntimeShapeRecord::new(
                allocation.parent(),
                allocation.prototype_guard(),
                allocation.slot_count(),
            ),
            lifetime,
        );
        self.store_shape_metadata(id, ShapeMetadata::bootstrap());
        id
    }

    /// Returns the canonical empty/root shape for one prototype guard.
    pub fn root_shape(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        prototype_guard: Option<ObjectRef>,
        lifetime: AllocationLifetime,
    ) -> ShapeId {
        let key = RootShapeKey { prototype_guard };
        if let Some(id) = self.root_shapes.get(&key).copied() {
            return id;
        }

        let id = self.alloc_shape(
            heap,
            ShapeAllocation::new(None, prototype_guard, 0),
            lifetime,
        );
        self.root_shapes.insert(key, id);
        id
    }

    /// Follows one canonical named-property transition from `parent`.
    ///
    /// Returns `None` when the parent shape does not exist or when the property
    /// key already exists on the parent shape, since that is a redefine path
    /// rather than an add-property transition.
    ///
    /// # Panics
    /// Panics if the parent shape property count does not fit into `u32`.
    pub fn transition_shape(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        parent: ShapeId,
        transition: ShapeTransitionKey,
        lifetime: AllocationLifetime,
    ) -> Option<ShapeId> {
        {
            let parent_metadata = self.shape_metadata(parent)?;
            if let Some(existing) = parent_metadata.transitions.get(&transition).copied() {
                return Some(existing);
            }
            if parent_metadata
                .property(transition.property_key())
                .is_some()
            {
                return None;
            }
        }

        let parent_record = heap.view().shape(parent)?;
        let parent_properties = self.shape_metadata(parent)?.properties.clone();
        let property = ShapeProperty::from_transition(
            transition,
            parent_record.slot_count(),
            u32::try_from(parent_properties.len()).expect("shape property count must fit into u32"),
        );
        let mut properties = parent_properties;
        properties.push(property);

        let id = heap.alloc_shape(
            RuntimeShapeRecord::new(
                Some(parent),
                parent_record.prototype_guard(),
                parent_record
                    .slot_count()
                    .saturating_add(transition.property_kind().slot_width()),
            ),
            lifetime,
        );
        self.store_shape_metadata(id, ShapeMetadata::derived(transition, properties));
        self.shape_metadata_mut(parent)?
            .transitions
            .insert(transition, id);
        Some(id)
    }

    #[inline]
    /// Returns a compact read-only record for one shape.
    ///
    /// # Panics
    /// Panics if the stored property count does not fit into `u32`.
    pub fn shape(&self, heap: PrimitiveHeapView<'_>, id: ShapeId) -> Option<ShapeRecord> {
        let record = heap.shape(id)?;
        let metadata = self.shape_metadata(id)?;
        Some(ShapeRecord::new(
            id,
            record.parent(),
            record.prototype_guard(),
            record.slot_count(),
            u32::try_from(metadata.properties.len())
                .expect("shape property count must fit into u32"),
            metadata.transition_key,
            metadata.property,
            metadata.property_lookup.is_some(),
        ))
    }

    #[inline]
    pub fn shape_property(&self, id: ShapeId, key: PropertyKey) -> Option<ShapeProperty> {
        self.shape_metadata(id)?.property(key)
    }

    #[inline]
    pub fn shape_properties(&self, id: ShapeId) -> Option<&[ShapeProperty]> {
        Some(&self.shape_metadata(id)?.properties)
    }

    #[inline]
    pub fn free_shape(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ShapeId,
    ) -> Option<ShapeRecord> {
        let shape = self.shape(heap.view(), id)?;
        if !self.shape_metadata(id)?.transitions.is_empty() {
            return None;
        }

        let metadata = self.take_shape_metadata(id)?;
        if let Some(parent) = shape.parent() {
            if let Some(parent_metadata) = self.shape_metadata_mut(parent) {
                if let Some(transition_key) = metadata.transition_key {
                    parent_metadata.transitions.remove(&transition_key);
                }
            }
        } else {
            self.root_shapes.remove(&RootShapeKey {
                prototype_guard: shape.prototype_guard(),
            });
        }

        heap.free_shape(id)?;
        Some(shape)
    }

    /// Allocates one object record together with its requested backing stores.
    ///
    /// # Panics
    /// Panics if a requested slot or element capacity does not fit into the runtime's compact
    /// integer fields.
    pub fn alloc_object(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        mut allocation: ObjectAllocation,
        lifetime: AllocationLifetime,
    ) -> ObjectRef {
        let shape_slot_count = self
            .shape(heap.view(), allocation.shape)
            .map_or(0, ShapeRecord::slot_count) as usize;
        let named_slot_count = allocation.named_slot_count.max(shape_slot_count);

        let named_slots = if named_slot_count == 0 {
            None
        } else {
            Some(heap.alloc_object_slots(named_slot_count, allocation.named_slot_fill, lifetime))
        };
        let elements = if allocation.element_capacity == 0 {
            None
        } else {
            Some(heap.alloc_object_slots(
                allocation.element_capacity,
                allocation.element_fill,
                lifetime,
            ))
        };
        let function_payload = match &allocation.cold {
            ObjectColdData::Function(data) => {
                let mut record = data.runtime_record();
                if let (Some(bound_init), Some(function_record)) =
                    (data.bound_init(), record.as_mut())
                {
                    let argument_slots = if bound_init.arguments().is_empty() {
                        None
                    } else {
                        let slots = heap.alloc_object_slots(
                            bound_init.arguments().len(),
                            Value::undefined(),
                            lifetime,
                        );
                        for (index, argument) in bound_init.arguments().iter().copied().enumerate()
                        {
                            assert!(
                                heap.init_store_value(
                                    lyng_js_gc::ValueStoreTarget::ObjectSlot(
                                        slots,
                                        u32::try_from(index)
                                            .expect("bound argument index should fit u32"),
                                    ),
                                    argument,
                                ),
                                "bound function argument slots should initialize exactly once"
                            );
                        }
                        Some(slots)
                    };
                    *function_record = function_record.with_bound(Some(
                        function_record
                            .bound()
                            .expect("bound function init should seed one bound record")
                            .with_arguments(argument_slots),
                    ));
                }
                record.map(|record| heap.alloc_function_payload(record, lifetime))
            }
            ObjectColdData::Ordinary(_) | ObjectColdData::Proxy(_) => None,
        };
        let ordinary_payload = allocation.ordinary_payload_value.map(|value| {
            let cell = heap.alloc_value_cell(lifetime);
            assert!(
                heap.init_store_value(lyng_js_gc::ValueStoreTarget::ValueCell(cell), value),
                "ordinary payload cell should initialize exactly once"
            );
            cell
        });
        if let ObjectColdData::Function(data) = allocation.cold {
            allocation.cold = ObjectColdData::Function(
                data.with_gc_payload(function_payload).without_bound_init(),
            );
        }
        let object = heap.alloc_object(
            RuntimeObjectRecord::new(
                allocation.prototype,
                Some(allocation.shape),
                named_slots,
                elements,
                function_payload,
            )
            .with_ordinary_payload(ordinary_payload),
            lifetime,
        );
        let cold_for_init = allocation.cold.clone();
        self.store_object_metadata(
            object,
            ObjectMetadata {
                kind: allocation.kind,
                flags: allocation.flags,
                cold: allocation.cold,
                private_brands: Vec::new(),
                named_properties: NamedPropertyStorage::ShapeStable,
                named_property_churn: 0,
                element_storage: if allocation.element_capacity == 0 {
                    ElementStorageMetadata::Empty
                } else {
                    ElementStorageMetadata::Dense {
                        logical_len: if allocation.element_fill == Value::array_hole() {
                            0
                        } else {
                            u32::try_from(allocation.element_capacity)
                                .expect("element capacity must fit into u32")
                        },
                    }
                },
                last_invalidation: None,
            },
        );
        if let ObjectColdData::Proxy(data) = cold_for_init {
            let initialized_target = self.init_named_slot(
                heap,
                object,
                PROXY_TARGET_SLOT_INDEX,
                Value::from_object_ref(data.target()),
            );
            debug_assert!(
                initialized_target,
                "proxy target slot should initialize during allocation"
            );
            let initialized_handler = self.init_named_slot(
                heap,
                object,
                PROXY_HANDLER_SLOT_INDEX,
                data.handler()
                    .map_or(Value::undefined(), Value::from_object_ref),
            );
            debug_assert!(
                initialized_handler,
                "proxy handler slot should initialize during allocation"
            );
        }
        let _ = self.refresh_integrity_level_flags(heap.view(), object);
        object
    }

    pub fn primitive_wrapper_kind(&self, id: ObjectRef) -> Option<crate::PrimitiveWrapperKind> {
        match self.object_metadata(id)?.cold {
            ObjectColdData::Ordinary(data) => data.wrapper_kind(),
            ObjectColdData::Function(_) | ObjectColdData::Proxy(_) => None,
        }
    }

    #[inline]
    pub fn is_proxy_object(&self, id: ObjectRef) -> bool {
        matches!(
            self.object_metadata(id)
                .map(|metadata| metadata.cold.clone()),
            Some(ObjectColdData::Proxy(_))
        )
    }

    pub fn proxy_data(&self, id: ObjectRef) -> Option<ProxyObjectData> {
        match self.object_metadata(id)?.cold {
            ObjectColdData::Proxy(data) => Some(data),
            ObjectColdData::Ordinary(_) | ObjectColdData::Function(_) => None,
        }
    }

    #[inline]
    pub fn proxy_target(&self, id: ObjectRef) -> Option<ObjectRef> {
        Some(self.proxy_data(id)?.target())
    }

    #[inline]
    pub fn proxy_handler(&self, id: ObjectRef) -> Option<ObjectRef> {
        self.proxy_data(id)?.handler()
    }

    #[inline]
    pub fn is_proxy_revoked(&self, id: ObjectRef) -> Option<bool> {
        Some(self.proxy_data(id)?.revoked())
    }

    pub fn revoke_proxy(&mut self, heap: &mut PrimitiveMutator<'_>, id: ObjectRef) -> bool {
        let Some(metadata) = self.object_metadata_mut(id) else {
            return false;
        };
        let ObjectColdData::Proxy(data) = &mut metadata.cold else {
            return false;
        };
        if data.revoked() {
            return true;
        }
        *data = data.with_handler(None).with_revoked(true);
        let record = match heap.view().object(id) {
            Some(record) => record,
            None => return false,
        };
        let Some(named_slots) = record.named_slots() else {
            return false;
        };
        heap.mut_store_value(
            lyng_js_gc::ValueStoreTarget::ObjectSlot(named_slots, PROXY_HANDLER_SLOT_INDEX),
            Value::undefined(),
        )
    }

    #[inline]
    pub fn is_callable(&self, id: ObjectRef) -> bool {
        match self
            .object_metadata(id)
            .map(|metadata| metadata.cold.clone())
        {
            Some(ObjectColdData::Ordinary(_)) | None => false,
            Some(ObjectColdData::Function(_)) => true,
            Some(ObjectColdData::Proxy(data)) => data.callable(),
        }
    }

    #[inline]
    pub fn is_constructor(&self, id: ObjectRef) -> bool {
        match self
            .object_metadata(id)
            .map(|metadata| metadata.cold.clone())
        {
            Some(ObjectColdData::Ordinary(_)) | None => false,
            Some(ObjectColdData::Function(data)) => data.is_constructible(),
            Some(ObjectColdData::Proxy(data)) => data.constructible(),
        }
    }

    pub fn install_module_namespace_object(
        &mut self,
        id: ObjectRef,
        exports: Vec<ModuleNamespaceExport>,
    ) -> bool {
        let Some(metadata) = self.object_metadata_mut(id) else {
            return false;
        };
        metadata.flags = metadata
            .flags
            .without(super::ObjectFlags::EXTENSIBLE)
            .union(super::ObjectFlags::SEALED)
            .without(super::ObjectFlags::FROZEN);
        self.store_module_namespace_slot(id, ModuleNamespaceObject::new(exports));
        true
    }

    #[inline]
    pub fn is_module_namespace_object(&self, id: ObjectRef) -> bool {
        self.module_namespace_slot(id).is_some()
    }

    #[inline]
    pub fn module_namespace_exports(&self, id: ObjectRef) -> Option<&[ModuleNamespaceExport]> {
        Some(self.module_namespace_slot(id)?.exports())
    }

    pub fn primitive_wrapper_value(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> Option<Value> {
        let wrapper_kind = self.primitive_wrapper_kind(id)?;
        let payload = heap.object(id)?.ordinary_payload()?;
        let value = heap.value_cell(payload)?.stored_value();
        debug_assert!(match wrapper_kind {
            crate::PrimitiveWrapperKind::String => value.is_string(),
            crate::PrimitiveWrapperKind::Number => value.is_number(),
            crate::PrimitiveWrapperKind::Boolean => value.is_bool(),
            crate::PrimitiveWrapperKind::Symbol => value.is_symbol(),
            crate::PrimitiveWrapperKind::BigInt => value.is_bigint(),
        });
        Some(value)
    }

    pub fn ordinary_payload_value(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> Option<Value> {
        let payload = heap.object(id)?.ordinary_payload()?;
        heap.value_cell(payload).map(|cell| cell.stored_value())
    }

    pub fn is_date_object(&self, id: ObjectRef) -> bool {
        matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_date()
        )
    }

    pub fn is_json_raw_object(&self, id: ObjectRef) -> bool {
        matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_json_raw()
        )
    }

    pub fn is_array_buffer_object(&self, id: ObjectRef) -> bool {
        matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_array_buffer()
        ) && self.array_buffer(id).is_some()
    }

    pub fn is_shared_array_buffer_object(&self, id: ObjectRef) -> bool {
        matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_shared_array_buffer()
        ) && self.array_buffer(id).is_some()
    }

    pub fn is_array_buffer_family_object(&self, id: ObjectRef) -> bool {
        matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_array_buffer_family()
        ) && self.array_buffer(id).is_some()
    }

    pub fn is_map_object(&self, id: ObjectRef) -> bool {
        matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_map()
        ) && self.map(id).is_some()
    }

    pub fn install_map_object(&mut self, id: ObjectRef, map: MapObjectData) -> bool {
        if !matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_map()
        ) {
            return false;
        }
        self.store_map_slot(id, map);
        true
    }

    pub fn map(&self, id: ObjectRef) -> Option<&MapObjectData> {
        if !matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_map()
        ) {
            return None;
        }
        self.map_slot(id)
    }

    pub fn with_map_mut<R>(
        &mut self,
        id: ObjectRef,
        f: impl FnOnce(&mut MapObjectData) -> R,
    ) -> Option<R> {
        if !matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_map()
        ) {
            return None;
        }
        Some(f(self.map_slot_mut(id)?))
    }

    pub fn is_set_object(&self, id: ObjectRef) -> bool {
        matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_set()
        ) && self.set_object_data(id).is_some()
    }

    pub fn is_weak_map_object(&self, id: ObjectRef) -> bool {
        matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_weak_map()
        )
    }

    pub fn is_weak_set_object(&self, id: ObjectRef) -> bool {
        matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_weak_set()
        )
    }

    pub fn is_weak_ref_object(&self, id: ObjectRef) -> bool {
        matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_weak_ref()
        )
    }

    pub fn is_finalization_registry_object(&self, id: ObjectRef) -> bool {
        matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_finalization_registry()
        )
    }

    pub fn install_set_object(&mut self, id: ObjectRef, set: SetObjectData) -> bool {
        if !matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_set()
        ) {
            return false;
        }
        self.store_set_slot(id, set);
        true
    }

    pub fn set_object_data(&self, id: ObjectRef) -> Option<&SetObjectData> {
        if !matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_set()
        ) {
            return None;
        }
        self.set_slot(id)
    }

    pub fn with_set_mut<R>(
        &mut self,
        id: ObjectRef,
        f: impl FnOnce(&mut SetObjectData) -> R,
    ) -> Option<R> {
        if !matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_set()
        ) {
            return None;
        }
        Some(f(self.set_slot_mut(id)?))
    }

    pub fn install_array_buffer_object(
        &mut self,
        id: ObjectRef,
        array_buffer: ArrayBufferObjectData,
    ) -> bool {
        if !matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_array_buffer_family()
        ) {
            return false;
        }
        self.store_array_buffer_slot(id, array_buffer);
        true
    }

    pub fn array_buffer(&self, id: ObjectRef) -> Option<ArrayBufferObjectData> {
        if !matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_array_buffer_family()
        ) {
            return None;
        }
        self.array_buffer_slot(id)
    }

    pub fn is_data_view_object(&self, id: ObjectRef) -> bool {
        matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_data_view()
        ) && self.data_view(id).is_some()
    }

    pub fn install_data_view_object(
        &mut self,
        id: ObjectRef,
        data_view: DataViewObjectData,
    ) -> bool {
        if !matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_data_view()
        ) {
            return false;
        }
        self.store_data_view_slot(id, data_view);
        true
    }

    pub fn data_view(&self, id: ObjectRef) -> Option<DataViewObjectData> {
        if !matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_data_view()
        ) {
            return None;
        }
        self.data_view_slot(id)
    }

    pub fn is_typed_array_object(&self, id: ObjectRef) -> bool {
        matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.typed_array_kind().is_some()
        ) && self.typed_array(id).is_some()
    }

    pub fn install_typed_array_object(
        &mut self,
        id: ObjectRef,
        typed_array: TypedArrayObjectData,
    ) -> bool {
        if !matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.typed_array_kind().is_some()
        ) {
            return false;
        }
        self.store_typed_array_slot(id, typed_array);
        true
    }

    pub fn typed_array(&self, id: ObjectRef) -> Option<TypedArrayObjectData> {
        if !matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.typed_array_kind().is_some()
        ) {
            return None;
        }
        self.typed_array_slot(id)
    }

    pub fn typed_array_views_of_buffer(
        &self,
        heap: PrimitiveHeapView<'_>,
        viewed_array_buffer: ObjectRef,
    ) -> Vec<(ObjectRef, TypedArrayObjectData)> {
        self.typed_arrays
            .iter()
            .enumerate()
            .filter_map(|(index, typed_array)| {
                let typed_array = typed_array.as_ref().copied()?;
                if typed_array.viewed_array_buffer() != viewed_array_buffer {
                    return None;
                }
                let raw = u32::try_from(index + 1).ok()?;
                let id = ObjectRef::from_raw(raw)?;
                heap.object(id)?;
                self.is_typed_array_object(id).then_some((id, typed_array))
            })
            .collect()
    }

    pub fn date_value(&self, heap: PrimitiveHeapView<'_>, id: ObjectRef) -> Option<Value> {
        self.is_date_object(id)
            .then(|| self.ordinary_payload_value(heap, id))
            .flatten()
    }

    pub fn set_date_value(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        value: Value,
    ) -> bool {
        if !self.is_date_object(id) {
            return false;
        }
        let payload = match heap
            .view()
            .object(id)
            .and_then(|record| record.ordinary_payload())
        {
            Some(payload) => payload,
            None => return false,
        };
        heap.mut_store_value(lyng_js_gc::ValueStoreTarget::ValueCell(payload), value)
    }

    pub fn is_temporal_object_kind(&self, id: ObjectRef, kind: TemporalObjectKind) -> bool {
        matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.temporal_kind() == Some(kind)
        ) && self.temporal_object(id).is_some()
    }

    pub fn install_temporal_object(&mut self, id: ObjectRef, payload: TemporalObjectData) -> bool {
        if !matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.temporal_kind() == Some(payload.kind())
        ) {
            return false;
        }
        self.store_temporal_object_slot(id, payload);
        true
    }

    pub fn temporal_object(&self, id: ObjectRef) -> Option<&TemporalObjectData> {
        if !matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.temporal_kind().is_some()
        ) {
            return None;
        }
        self.temporal_object_slot(id)
    }

    pub fn is_regexp_object(&self, id: ObjectRef) -> bool {
        matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_regexp()
        ) && self.regexp_payload(id).is_some()
    }

    pub fn is_generator_object(&self, id: ObjectRef) -> bool {
        matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_generator()
        ) && self.generator_state(id).is_some()
    }

    pub fn install_generator_object(
        &mut self,
        id: ObjectRef,
        state: super::GeneratorState,
    ) -> bool {
        if !matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_generator()
        ) {
            return false;
        }
        self.store_generator_state_slot(id, state);
        true
    }

    pub fn generator_state(&self, id: ObjectRef) -> Option<super::GeneratorState> {
        if !matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_generator()
        ) {
            return None;
        }
        self.generator_state_slot(id)
    }

    pub fn set_generator_state(&mut self, id: ObjectRef, state: super::GeneratorState) -> bool {
        if !self.is_generator_object(id) {
            return false;
        }
        self.store_generator_state_slot(id, state);
        true
    }

    pub fn generator_suspended(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> Option<lyng_js_types::SuspendedExecutionRef> {
        if !self.is_generator_object(id) {
            return None;
        }
        self.ordinary_payload_value(heap, id)
            .and_then(lyng_js_types::Value::as_suspended_execution_ref)
    }

    pub fn set_generator_suspended(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        suspended: Option<lyng_js_types::SuspendedExecutionRef>,
    ) -> bool {
        if !self.is_generator_object(id) {
            return false;
        }
        let payload = match heap
            .view()
            .object(id)
            .and_then(|record| record.ordinary_payload())
        {
            Some(payload) => payload,
            None => return false,
        };
        heap.mut_store_value(
            lyng_js_gc::ValueStoreTarget::ValueCell(payload),
            suspended.map_or(Value::undefined(), Value::from_suspended_execution_ref),
        )
    }

    pub fn regexp_payload(&self, id: ObjectRef) -> Option<&RegExpPayload> {
        if !matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_regexp()
        ) {
            return None;
        }
        self.regexp_payload_slot(id)
    }

    pub fn store_regexp_payload(&mut self, id: ObjectRef, payload: RegExpPayload) -> bool {
        if !matches!(
            self.object_metadata(id).map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(data)) if data.is_regexp()
        ) {
            return false;
        }
        self.store_regexp_payload_slot(id, payload);
        true
    }

    pub fn regexp_payload_accounting(
        &self,
        heap: PrimitiveHeapView<'_>,
    ) -> RegExpPayloadAccounting {
        let mut accounting = RegExpPayloadAccounting::default();
        for (index, payload) in self.regexp_payloads.iter().enumerate() {
            let Some(payload) = payload else {
                continue;
            };
            let raw = u32::try_from(index + 1).expect("object side-table index should fit u32");
            let Some(id) = ObjectRef::from_raw(raw) else {
                continue;
            };
            if heap.object(id).is_none() || !self.is_regexp_object(id) {
                continue;
            }
            accounting.records += 1;
            accounting.metadata_bytes += std::mem::size_of::<RegExpPayload>();
            accounting.payload_bytes += payload.payload_bytes();
        }
        accounting.live_bytes = accounting.metadata_bytes + accounting.payload_bytes;
        accounting
    }

    pub fn object_header(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> Option<ObjectHeader> {
        let record = heap.object(id)?;
        let metadata = self.object_metadata(id)?;
        Some(ObjectHeader::new(
            id,
            metadata.kind,
            metadata.flags,
            record.prototype(),
            record.shape()?,
            record.named_slots().map(NamedSlotStorageRef::new),
            record.elements().map(ElementStorageRef::new),
        ))
    }

    pub fn object(&self, heap: PrimitiveHeapView<'_>, id: ObjectRef) -> Option<ObjectRecord> {
        let header = self.object_header(heap, id)?;
        let metadata = self.object_metadata(id)?;
        Some(ObjectRecord::new(header, metadata.cold.clone()))
    }

    pub fn free_object(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
    ) -> Option<ObjectRecord> {
        let header = self.object_header(heap.view(), id)?;
        let metadata = self.take_object_metadata(id)?;
        let _ = self.take_class_record_slot(id);
        let _ = self.take_module_namespace_slot(id);
        let _ = self.take_map_slot(id);
        let _ = self.take_set_slot(id);
        let _ = self.take_data_view_slot(id);
        let _ = self.take_typed_array_slot(id);
        let _ = self.take_temporal_object_slot(id);
        let _ = self.take_regexp_payload_slot(id);
        let _ = self.take_generator_state_slot(id);
        heap.free_object(id)?;
        Some(ObjectRecord::new(header, metadata.cold))
    }

    #[inline]
    pub fn named_slots<'a>(
        &self,
        heap: PrimitiveHeapView<'a>,
        id: ObjectRef,
    ) -> Option<&'a [Value]> {
        let record = heap.object(id)?;
        self.object_metadata(id)?;
        heap.object_slots(record.named_slots()?)
    }

    #[inline]
    pub fn elements<'a>(&self, heap: PrimitiveHeapView<'a>, id: ObjectRef) -> Option<&'a [Value]> {
        let record = heap.object(id)?;
        self.object_metadata(id)?;
        heap.object_slots(record.elements()?)
    }

    #[inline]
    pub const fn current_invalidation_epoch(&self) -> u64 {
        self.next_invalidation_epoch
    }

    #[inline]
    pub fn invalidation_event(&self, id: ObjectRef) -> Option<InvalidationEvent> {
        self.object_metadata(id)?.last_invalidation
    }

    #[inline]
    pub fn named_property_storage_mode(&self, id: ObjectRef) -> Option<NamedPropertyStorageMode> {
        Some(self.object_metadata(id)?.named_properties.mode())
    }

    #[inline]
    pub fn named_property_churn(&self, id: ObjectRef) -> Option<u32> {
        Some(self.object_metadata(id)?.named_property_churn)
    }

    pub fn named_property_dictionary_entry(
        &self,
        id: ObjectRef,
        key: PropertyKey,
    ) -> Option<NamedPropertyDictionaryEntry> {
        let metadata = self.object_metadata(id)?;
        let NamedPropertyStorage::Dictionary(dictionary) = &metadata.named_properties else {
            return None;
        };
        dictionary.entry(key)
    }

    pub fn named_property_dictionary_entries(
        &self,
        id: ObjectRef,
    ) -> Option<Vec<NamedPropertyDictionaryEntry>> {
        let metadata = self.object_metadata(id)?;
        let NamedPropertyStorage::Dictionary(dictionary) = &metadata.named_properties else {
            return None;
        };
        Some(dictionary.ordered_entries())
    }

    #[inline]
    pub fn element_mode(&self, id: ObjectRef) -> Option<ElementMode> {
        Some(self.object_metadata(id)?.element_storage.mode())
    }

    #[inline]
    pub fn element_logical_len(&self, id: ObjectRef) -> Option<u32> {
        Some(self.object_metadata(id)?.element_storage.logical_len())
    }

    pub fn sparse_element(&self, id: ObjectRef, index: u32) -> Option<SparseElementEntry> {
        let metadata = self.object_metadata(id)?;
        let ElementStorageMetadata::Sparse { entries, .. } = &metadata.element_storage else {
            return None;
        };
        entries.get(&index).copied()
    }

    pub fn element(&self, heap: PrimitiveHeapView<'_>, id: ObjectRef, index: u32) -> Option<Value> {
        let metadata = self.object_metadata(id)?;
        match &metadata.element_storage {
            ElementStorageMetadata::Empty => Some(Value::array_hole()),
            ElementStorageMetadata::Dense { logical_len } => {
                if index >= *logical_len {
                    return Some(Value::array_hole());
                }
                let record = heap.object(id)?;
                let slots = heap.object_slots(record.elements()?)?;
                Some(
                    slots
                        .get(index as usize)
                        .copied()
                        .unwrap_or(Value::array_hole()),
                )
            }
            ElementStorageMetadata::Sparse { entries, .. } => Some(
                entries
                    .get(&index)
                    .copied()
                    .and_then(SparseElementEntry::data_value)
                    .unwrap_or(Value::array_hole()),
            ),
        }
    }
}
