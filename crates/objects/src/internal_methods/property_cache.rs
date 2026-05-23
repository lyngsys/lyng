use super::*;

impl ObjectRuntime {
    /// Builds one substrate-owned named-property inline-cache record when the current access path
    /// is compatible with the shape-based cache hit path.
    ///
    /// # Errors
    /// Returns an error when the receiver or a traversed prototype object is missing or when the
    /// runtime detects corrupt state while planning the cache entry.
    pub fn plan_named_property_cache_entry(
        &self,
        heap: PrimitiveHeapView<'_>,
        receiver: ObjectRef,
        key: PropertyKey,
        purpose: NamedPropertyCachePurpose,
    ) -> InternalMethodResult<Option<NamedPropertyCacheEntry>> {
        if key.is_index() {
            return Ok(None);
        }

        let receiver_header = self
            .object_header(heap, receiver)
            .ok_or(InternalMethodError::MissingObject)?;
        if receiver_header.kind() == ObjectKind::Proxy {
            return Ok(None);
        }
        if matches!(purpose, NamedPropertyCachePurpose::Store)
            && receiver_header.flags().is_engine_array()
            && key.as_atom() == Some(WellKnownAtom::length.id())
        {
            return Ok(None);
        }
        let mut dependencies = [None; PROPERTY_CACHE_MAX_DEPENDENCIES];
        let mut dependency_count = 0u8;
        if !Self::push_property_cache_dependency(
            heap,
            &mut dependencies,
            &mut dependency_count,
            receiver,
        )? {
            return Ok(None);
        }

        // Receivers that store their own properties in a dictionary may
        // shadow inherited values with own accessor / writable / configurable
        // entries that the shape table doesn't reflect. Walking the
        // prototype chain past such a receiver would record a
        // `PrototypeData` plan for a property that's actually overridden
        // on the receiver — see classes with `static get name()` on top
        // of `Function.prototype.name`. Bail rather than risk a stale plan.
        if receiver_header.flags().uses_named_property_dictionary() {
            return Ok(None);
        }
        if let Some(property) = self.shape_property(receiver_header.shape(), key) {
            return Ok(Self::build_named_property_cache_entry(
                purpose,
                receiver_header.shape(),
                receiver,
                receiver_header.shape(),
                property,
                NamedPropertyCachePath::OwnData,
                dependency_count,
                dependencies,
            ));
        }

        if matches!(purpose, NamedPropertyCachePurpose::Store) {
            return Ok(None);
        }

        let mut current = receiver_header.prototype();
        while let Some(object) = current {
            let header = self
                .object_header(heap, object)
                .ok_or(InternalMethodError::MissingObject)?;
            if header.kind() == ObjectKind::Proxy {
                return Ok(None);
            }
            if !Self::push_property_cache_dependency(
                heap,
                &mut dependencies,
                &mut dependency_count,
                object,
            )? {
                return Ok(None);
            }
            if header.flags().uses_named_property_dictionary() {
                return Ok(None);
            }
            if let Some(property) = self.shape_property(header.shape(), key) {
                return Ok(Self::build_named_property_cache_entry(
                    purpose,
                    receiver_header.shape(),
                    object,
                    header.shape(),
                    property,
                    NamedPropertyCachePath::PrototypeData,
                    dependency_count,
                    dependencies,
                ));
            }
            current = header.prototype();
        }

        Ok(None)
    }

    /// Attempts to load one value through a previously planned named-property cache entry.
    ///
    /// The cached `slot_offset` carries an inline/out-of-line bit
    /// ([`super::SlotLocation::decode`]). Inline slots read directly from the holder's
    /// `RuntimeObjectRecord.inline_named_slots` (one indexed load past the object header), matching
    /// V8's in-object property cache hit path; out-of-line slots read from the heap-allocated
    /// `named_slots` array as before.
    ///
    /// # Errors
    /// Returns an error when the cached holder object or its slot storage is missing or corrupt.
    #[inline]
    pub fn load_from_named_property_cache(
        &self,
        heap: PrimitiveHeapView<'_>,
        receiver: ObjectRef,
        entry: NamedPropertyCacheEntry,
    ) -> InternalMethodResult<Option<Value>> {
        let Some(holder) = Self::validated_named_property_cache_holder(heap, receiver, entry)?
        else {
            return Ok(None);
        };
        match SlotLocation::decode(entry.slot_offset()) {
            SlotLocation::Inline(index) => Ok(holder.inline_named_slot(index as usize)),
            SlotLocation::OutOfLine(offset) => {
                let slots = holder
                    .named_slots()
                    .and_then(|slots| heap.object_slots(slots))
                    .ok_or(InternalMethodError::CorruptObjectState)?;
                Ok(slots.get(offset as usize).copied())
            }
        }
    }

    /// Directly probes ordinary shape-stable named data properties without materializing a
    /// `PropertyDescriptor`.
    ///
    /// Returns `None` when the path is not safely serviceable by the direct probe: proxies,
    /// dictionary-backed/exotic objects, or accessor properties. Returns
    /// [`NamedPropertyDirectGet::Absent`] only after proving the full ordinary prototype chain has
    /// no matching property.
    #[inline]
    pub fn try_direct_get_named_data_property(
        &self,
        heap: PrimitiveHeapView<'_>,
        receiver: ObjectRef,
        key: PropertyKey,
    ) -> Option<NamedPropertyDirectGet> {
        if key.is_index() {
            return None;
        }
        let mut current = Some(receiver);
        while let Some(object) = current {
            let header = self.object_header(heap, object)?;
            if !self.named_data_get_object_is_cacheable(header) {
                return None;
            }
            if let Some(property) = self.shape_property(header.shape(), key) {
                if property.kind() != ShapePropertyKind::Data {
                    return None;
                }
                let value = self.read_named_property_slot(heap, object, property.slot_offset())?;
                return Some(NamedPropertyDirectGet::Data(value));
            }
            current = header.prototype();
        }
        Some(NamedPropertyDirectGet::Absent)
    }

    /// Attempts to store one value through a previously planned named-property cache entry.
    ///
    /// Same inline/out-of-line dispatch as the load path. Inline writes are followed by an
    /// explicit incremental-marking value barrier on the holder so any heap reference newly
    /// embedded in the inline slot is shaded gray when an incremental mark is in flight
    /// (inline storage lives outside the GC heap arena, so the arena's automatic barrier
    /// doesn't fire on those writes).
    ///
    /// # Errors
    /// Returns an error when the cached holder object or its slot storage is missing or corrupt.
    pub fn store_to_named_property_cache(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        receiver: ObjectRef,
        key: PropertyKey,
        entry: NamedPropertyCacheEntry,
        value: Value,
    ) -> InternalMethodResult<Option<bool>> {
        match entry.path() {
            NamedPropertyCachePath::OwnData => {}
            NamedPropertyCachePath::OwnDataTransition => {
                return self
                    .store_to_named_property_transition_cache(heap, receiver, key, entry, value);
            }
            NamedPropertyCachePath::PrototypeData => return Ok(None),
        }
        let Some(holder) =
            Self::validated_named_property_cache_holder(heap.view(), receiver, entry)?
        else {
            return Ok(None);
        };
        if !entry.attrs().writable() {
            return Ok(Some(false));
        }

        match SlotLocation::decode(entry.slot_offset()) {
            SlotLocation::Inline(index) => {
                if !heap.mut_store_value(ValueStoreTarget::InlineNamedSlot(receiver, index), value)
                {
                    return Err(InternalMethodError::CorruptObjectState);
                }
                Ok(Some(true))
            }
            SlotLocation::OutOfLine(offset) => {
                let slots = holder
                    .named_slots()
                    .ok_or(InternalMethodError::CorruptObjectState)?;
                if !heap.mut_store_value(ValueStoreTarget::ObjectSlot(slots, offset), value) {
                    return Err(InternalMethodError::CorruptObjectState);
                }
                Ok(Some(true))
            }
        }
    }

    /// Builds a transition-store cache entry for `Set` on an absent own named data property.
    ///
    /// The entry records the receiver's pre-store shape, the post-store transition shape, and
    /// dependencies for the receiver and every prototype object observed to be property-free.
    /// Applying the entry later is valid for same-shaped fresh receivers with the same unchanged
    /// prototype chain.
    ///
    /// # Errors
    /// Returns an error when the receiver, a traversed prototype, or shape/slot metadata is
    /// missing or internally inconsistent while building the transition entry.
    pub fn plan_named_property_transition_store_entry(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        receiver: ObjectRef,
        key: PropertyKey,
        lifetime: AllocationLifetime,
    ) -> InternalMethodResult<Option<NamedPropertyCacheEntry>> {
        if key.is_index() {
            return Ok(None);
        }
        let receiver_header = self
            .object_header(heap.view(), receiver)
            .ok_or(InternalMethodError::MissingObject)?;
        if !self.transition_store_object_is_cacheable(receiver_header) {
            return Ok(None);
        }
        if self.shape_property(receiver_header.shape(), key).is_some() {
            return Ok(None);
        }
        let receiver_record = heap
            .view()
            .object(receiver)
            .ok_or(InternalMethodError::MissingObject)?;
        if self.has_reserved_named_slots(heap.view(), receiver_record) {
            return Ok(None);
        }
        if self
            .shape(heap.view(), receiver_header.shape())
            .is_some_and(|shape| {
                shape.property_count() >= NAMED_PROPERTY_ADDITION_CHAIN_DICTIONARY_LIMIT
            })
        {
            return Ok(None);
        }
        if self.object_metadata(receiver).is_some_and(|metadata| {
            metadata.named_property_additions >= NAMED_PROPERTY_ADDITION_CHAIN_DICTIONARY_LIMIT
        }) {
            return Ok(None);
        }

        let mut dependencies = [None; PROPERTY_CACHE_MAX_DEPENDENCIES];
        let mut dependency_count = 0u8;
        if !Self::push_property_cache_dependency(
            heap.view(),
            &mut dependencies,
            &mut dependency_count,
            receiver,
        )? {
            return Ok(None);
        }

        let mut current = receiver_header.prototype();
        while let Some(object) = current {
            let header = self
                .object_header(heap.view(), object)
                .ok_or(InternalMethodError::MissingObject)?;
            if !self.transition_store_object_is_cacheable(header) {
                return Ok(None);
            }
            if !Self::push_property_cache_dependency(
                heap.view(),
                &mut dependencies,
                &mut dependency_count,
                object,
            )? {
                return Ok(None);
            }
            if self.shape_property(header.shape(), key).is_some() {
                return Ok(None);
            }
            current = header.prototype();
        }

        let attrs = ordinary_property_attrs();
        let transition = ShapeTransitionKey::new(key, ShapePropertyKind::Data, attrs);
        let Some(next_shape) =
            self.transition_shape(heap, receiver_header.shape(), transition, lifetime)
        else {
            return Ok(None);
        };
        let property = self
            .shape_property(next_shape, key)
            .ok_or(InternalMethodError::CorruptObjectState)?;
        Ok(Some(NamedPropertyCacheEntry::new(
            receiver_header.shape(),
            receiver,
            next_shape,
            property.slot_offset(),
            attrs,
            NamedPropertyCachePath::OwnDataTransition,
            dependency_count,
            dependencies,
        )))
    }

    fn store_to_named_property_transition_cache(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        receiver: ObjectRef,
        key: PropertyKey,
        entry: NamedPropertyCacheEntry,
        value: Value,
    ) -> InternalMethodResult<Option<bool>> {
        let Some(record) =
            Self::validated_named_property_transition_receiver(heap.view(), receiver, entry)?
        else {
            return Ok(None);
        };
        let header = self
            .object_header(heap.view(), receiver)
            .ok_or(InternalMethodError::MissingObject)?;
        if !self.transition_store_object_is_cacheable(header) {
            return Ok(None);
        }
        if !header.flags().is_extensible() {
            return Ok(Some(false));
        }
        if self.has_reserved_named_slots(heap.view(), record) {
            return Ok(None);
        }
        if self.object_metadata(receiver).is_some_and(|metadata| {
            metadata.named_property_additions >= NAMED_PROPERTY_ADDITION_CHAIN_DICTIONARY_LIMIT
        }) {
            return Ok(None);
        }
        let updated = self.ordinary_define_absent_shaped_named_property(
            heap,
            receiver,
            key,
            NamedPropertyValue::data(value),
            entry.attrs(),
            AllocationLifetime::Default,
        )?;
        if !updated {
            return Ok(Some(false));
        }
        let new_shape = heap
            .view()
            .object(receiver)
            .and_then(RuntimeObjectRecord::shape)
            .ok_or(InternalMethodError::CorruptObjectState)?;
        if new_shape != entry.holder_shape() {
            return Ok(Some(true));
        }
        Ok(Some(true))
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "cache entries are assembled from the exact guard tuple stored in the entry"
    )]
    fn build_named_property_cache_entry(
        purpose: NamedPropertyCachePurpose,
        receiver_shape: ShapeId,
        holder: ObjectRef,
        holder_shape: ShapeId,
        property: ShapeProperty,
        path: NamedPropertyCachePath,
        dependency_count: u8,
        dependencies: [Option<PropertyCacheDependency>; PROPERTY_CACHE_MAX_DEPENDENCIES],
    ) -> Option<NamedPropertyCacheEntry> {
        if property.kind() != ShapePropertyKind::Data {
            return None;
        }
        if matches!(purpose, NamedPropertyCachePurpose::Store)
            && path != NamedPropertyCachePath::OwnData
        {
            return None;
        }
        Some(NamedPropertyCacheEntry::new(
            receiver_shape,
            holder,
            holder_shape,
            property.slot_offset(),
            property.attrs(),
            path,
            dependency_count,
            dependencies,
        ))
    }

    #[inline]
    fn validated_named_property_cache_holder(
        heap: PrimitiveHeapView<'_>,
        receiver: ObjectRef,
        entry: NamedPropertyCacheEntry,
    ) -> InternalMethodResult<Option<RuntimeObjectRecord>> {
        let Some(receiver_dependency) = entry.dependency(0) else {
            return Ok(None);
        };
        let receiver_record = heap
            .object(receiver)
            .ok_or(InternalMethodError::MissingObject)?;
        if !Self::record_matches_cache_dependency(receiver_record, receiver_dependency)
            || receiver_record.shape() != Some(entry.receiver_shape())
        {
            return Ok(None);
        }

        match entry.path() {
            NamedPropertyCachePath::OwnData => {
                if entry.dependency_count() != 1 || entry.holder_shape() != entry.receiver_shape() {
                    return Ok(None);
                }
                Ok(Some(receiver_record))
            }
            NamedPropertyCachePath::OwnDataTransition => Ok(None),
            NamedPropertyCachePath::PrototypeData => {
                let mut current = receiver_record.prototype();
                let mut holder = None;
                for index in 1..usize::from(entry.dependency_count()) {
                    let Some(dependency) = entry.dependency(index) else {
                        return Ok(None);
                    };
                    let Some(object) = current else {
                        return Ok(None);
                    };
                    if object != dependency.object() {
                        return Ok(None);
                    }
                    let record = heap
                        .object(object)
                        .ok_or(InternalMethodError::MissingObject)?;
                    if !Self::record_matches_cache_dependency(record, dependency) {
                        return Ok(None);
                    }
                    current = record.prototype();
                    holder = Some((object, record));
                }
                let Some((holder_id, holder_record)) = holder else {
                    return Ok(None);
                };
                if holder_id != entry.holder()
                    || holder_record.shape() != Some(entry.holder_shape())
                {
                    return Ok(None);
                }
                Ok(Some(holder_record))
            }
        }
    }

    #[inline]
    fn record_matches_cache_dependency(
        record: RuntimeObjectRecord,
        dependency: PropertyCacheDependency,
    ) -> bool {
        record.shape() == Some(dependency.shape())
            && record.last_invalidation_epoch() == dependency.invalidation_epoch()
    }

    #[inline]
    fn validated_named_property_transition_receiver(
        heap: PrimitiveHeapView<'_>,
        receiver: ObjectRef,
        entry: NamedPropertyCacheEntry,
    ) -> InternalMethodResult<Option<RuntimeObjectRecord>> {
        if entry.path() != NamedPropertyCachePath::OwnDataTransition {
            return Ok(None);
        }
        let Some(receiver_dependency) = entry.dependency(0) else {
            return Ok(None);
        };
        let receiver_record = heap
            .object(receiver)
            .ok_or(InternalMethodError::MissingObject)?;
        if !Self::record_matches_cache_dependency(receiver_record, receiver_dependency)
            || receiver_record.shape() != Some(entry.receiver_shape())
        {
            return Ok(None);
        }

        let mut current = receiver_record.prototype();
        for index in 1..usize::from(entry.dependency_count()) {
            let Some(dependency) = entry.dependency(index) else {
                return Ok(None);
            };
            let Some(object) = current else {
                return Ok(None);
            };
            if object != dependency.object() {
                return Ok(None);
            }
            let record = heap
                .object(object)
                .ok_or(InternalMethodError::MissingObject)?;
            if !Self::record_matches_cache_dependency(record, dependency) {
                return Ok(None);
            }
            current = record.prototype();
        }
        if current.is_some() {
            return Ok(None);
        }
        Ok(Some(receiver_record))
    }

    #[inline]
    fn transition_store_object_is_cacheable(&self, header: ObjectHeader) -> bool {
        self.has_shape_stable_named_properties(header)
            && header.flags().is_extensible()
            && !header.flags().uses_named_property_dictionary()
            && !header.flags().is_engine_array()
            && !header.flags().is_arguments_object()
            && !self.is_module_namespace_object(header.id())
            && !self.is_typed_array_object(header.id())
            && !self.is_string_exotic_object(header.id())
    }

    #[inline]
    fn named_data_get_object_is_cacheable(&self, header: ObjectHeader) -> bool {
        self.has_shape_stable_named_properties(header)
            && !header.flags().uses_named_property_dictionary()
            && !header.flags().is_engine_array()
            && !header.flags().is_arguments_object()
            && !self.is_module_namespace_object(header.id())
            && !self.is_typed_array_object(header.id())
            && !self.is_string_exotic_object(header.id())
    }

    #[inline]
    fn has_shape_stable_named_properties(&self, header: ObjectHeader) -> bool {
        matches!(
            self.object_metadata(header.id())
                .map(|metadata| &metadata.cold),
            Some(ObjectColdData::Ordinary(OrdinaryObjectData::Plain) | ObjectColdData::Function(_))
        )
    }

    fn push_property_cache_dependency(
        heap: PrimitiveHeapView<'_>,
        dependencies: &mut [Option<PropertyCacheDependency>; PROPERTY_CACHE_MAX_DEPENDENCIES],
        dependency_count: &mut u8,
        object: ObjectRef,
    ) -> InternalMethodResult<bool> {
        let record = heap
            .object(object)
            .ok_or(InternalMethodError::MissingObject)?;
        let Some(shape) = record.shape() else {
            return Err(InternalMethodError::CorruptObjectState);
        };
        let index = usize::from(*dependency_count);
        if index >= PROPERTY_CACHE_MAX_DEPENDENCIES {
            return Ok(false);
        }
        dependencies[index] = Some(PropertyCacheDependency::new(
            object,
            shape,
            record.last_invalidation_epoch(),
        ));
        *dependency_count = dependency_count.saturating_add(1);
        Ok(true)
    }
}
