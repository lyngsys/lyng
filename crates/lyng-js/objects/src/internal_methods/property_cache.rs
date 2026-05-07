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
        let mut dependencies = [None; PROPERTY_CACHE_MAX_DEPENDENCIES];
        let mut dependency_count = 0u8;
        if !Self::push_property_cache_dependency(
            self,
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
                self,
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
    /// # Errors
    /// Returns an error when the cached holder object or its slot storage is missing or corrupt.
    pub fn load_from_named_property_cache(
        &self,
        heap: PrimitiveHeapView<'_>,
        receiver: ObjectRef,
        entry: NamedPropertyCacheEntry,
    ) -> InternalMethodResult<Option<Value>> {
        if !self.named_property_cache_entry_valid(heap, receiver, entry)? {
            return Ok(None);
        }

        let holder_id = match entry.path() {
            NamedPropertyCachePath::OwnData => receiver,
            NamedPropertyCachePath::PrototypeData => entry.holder(),
        };
        let holder = heap
            .object(holder_id)
            .ok_or(InternalMethodError::MissingObject)?;
        let slots = holder
            .named_slots()
            .and_then(|slots| heap.object_slots(slots))
            .ok_or(InternalMethodError::CorruptObjectState)?;
        let value = slots
            .get(entry.slot_offset() as usize)
            .copied()
            .ok_or(InternalMethodError::CorruptObjectState)?;
        Ok(Some(value))
    }

    /// Attempts to store one value through a previously planned named-property cache entry.
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
        if !self.named_property_cache_entry_valid(heap.view(), receiver, entry)? {
            return Ok(None);
        }
        if !entry.attrs().writable() {
            return Ok(Some(false));
        }

        let holder = heap
            .view()
            .object(receiver)
            .ok_or(InternalMethodError::MissingObject)?;
        let slots = holder
            .named_slots()
            .ok_or(InternalMethodError::CorruptObjectState)?;
        if !heap.mut_store_value(
            ValueStoreTarget::ObjectSlot(slots, entry.slot_offset()),
            value,
        ) {
            return Err(InternalMethodError::CorruptObjectState);
        }
        Ok(Some(true))
    }

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

    fn named_property_cache_entry_valid(
        &self,
        heap: PrimitiveHeapView<'_>,
        receiver: ObjectRef,
        entry: NamedPropertyCacheEntry,
    ) -> InternalMethodResult<bool> {
        let receiver_header = self
            .object_header(heap, receiver)
            .ok_or(InternalMethodError::MissingObject)?;
        if receiver_header.shape() != entry.receiver_shape()
            || receiver_header.flags().uses_named_property_dictionary()
        {
            return Ok(false);
        }

        match entry.path() {
            NamedPropertyCachePath::OwnData => return Ok(true),
            NamedPropertyCachePath::PrototypeData => {
                let mut current = receiver_header.prototype();
                for index in 1..usize::from(entry.dependency_count()) {
                    let Some(dependency) = entry.dependency(index) else {
                        return Ok(false);
                    };
                    let Some(object) = current else {
                        return Ok(false);
                    };
                    if object != dependency.object() {
                        return Ok(false);
                    }
                    let header = self
                        .object_header(heap, object)
                        .ok_or(InternalMethodError::MissingObject)?;
                    if header.shape() != dependency.shape()
                        || header.flags().uses_named_property_dictionary()
                    {
                        return Ok(false);
                    }
                    let current_epoch = self
                        .invalidation_event(object)
                        .map(InvalidationEvent::epoch);
                    if current_epoch != dependency.invalidation_epoch() {
                        return Ok(false);
                    }
                    current = header.prototype();
                }
            }
        }

        let holder_id = match entry.path() {
            NamedPropertyCachePath::OwnData => receiver,
            NamedPropertyCachePath::PrototypeData => entry.holder(),
        };
        let holder_header = self
            .object_header(heap, holder_id)
            .ok_or(InternalMethodError::MissingObject)?;
        if holder_header.shape() != entry.holder_shape()
            || holder_header.flags().uses_named_property_dictionary()
        {
            return Ok(false);
        }
        Ok(true)
    }

    fn push_property_cache_dependency(
        &self,
        heap: PrimitiveHeapView<'_>,
        dependencies: &mut [Option<PropertyCacheDependency>; PROPERTY_CACHE_MAX_DEPENDENCIES],
        dependency_count: &mut u8,
        object: ObjectRef,
    ) -> InternalMethodResult<bool> {
        let header = self
            .object_header(heap, object)
            .ok_or(InternalMethodError::MissingObject)?;
        let index = usize::from(*dependency_count);
        if index >= PROPERTY_CACHE_MAX_DEPENDENCIES {
            return Ok(false);
        }
        dependencies[index] = Some(PropertyCacheDependency::new(
            object,
            header.shape(),
            self.invalidation_event(object)
                .map(InvalidationEvent::epoch),
        ));
        *dependency_count = dependency_count.saturating_add(1);
        Ok(true)
    }
}
