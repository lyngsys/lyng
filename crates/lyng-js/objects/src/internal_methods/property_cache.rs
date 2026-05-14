use super::*;

impl ObjectRuntime {
    /// Builds one substrate-owned named-property inline-cache record when the current access path
    /// is compatible with the shape-based fast path.
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

        if !receiver_header.flags().uses_named_property_dictionary()
            && let Some(property) = self.shape_property(receiver_header.shape(), key)
        {
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
    /// V8's in-object property fast path; out-of-line slots read from the heap-allocated
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
        entry: NamedPropertyCacheEntry,
        value: Value,
    ) -> InternalMethodResult<Option<bool>> {
        if entry.path() != NamedPropertyCachePath::OwnData {
            return Ok(None);
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
