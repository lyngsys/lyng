#![allow(clippy::wildcard_imports)]

use super::*;
use lyng_js_common::WellKnownAtom;

impl ObjectRuntime {
    /// Returns the prototype of one object.
    ///
    /// # Errors
    /// Returns an error when the object record is missing or invalid.
    pub fn get_prototype_of(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> InternalMethodResult<Option<ObjectRef>> {
        match self.require_object_kind(id)? {
            ObjectKind::Proxy => {
                let data = self
                    .proxy_data(id)
                    .ok_or(InternalMethodError::MissingObject)?;
                if data.revoked() {
                    Err(InternalMethodError::RevokedProxy)
                } else {
                    self.get_prototype_of(heap, data.target())
                }
            }
            ObjectKind::Ordinary | ObjectKind::Function => {
                Self::ordinary_get_prototype_of(heap, id)
            }
        }
    }

    /// Attempts to replace the prototype of one object.
    ///
    /// # Errors
    /// Returns an error when the object record is missing or invalid.
    pub fn set_prototype_of(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        prototype: Option<ObjectRef>,
    ) -> InternalMethodResult<bool> {
        match self.require_object_kind(id)? {
            ObjectKind::Proxy => {
                let data = self
                    .proxy_data(id)
                    .ok_or(InternalMethodError::MissingObject)?;
                if data.revoked() {
                    Err(InternalMethodError::RevokedProxy)
                } else {
                    self.set_prototype_of(heap, data.target(), prototype)
                }
            }
            ObjectKind::Ordinary | ObjectKind::Function => {
                if self.is_module_namespace_object(id) {
                    Ok(Self::ordinary_get_prototype_of(heap.view(), id)? == prototype)
                } else {
                    self.ordinary_set_prototype_of(heap, id, prototype)
                }
            }
        }
    }

    /// Reports whether one object is still extensible.
    ///
    /// # Errors
    /// Returns an error when the object record is missing or invalid.
    pub fn is_extensible(&self, id: ObjectRef) -> InternalMethodResult<bool> {
        match self.require_object_kind(id)? {
            ObjectKind::Proxy => {
                let data = self
                    .proxy_data(id)
                    .ok_or(InternalMethodError::MissingObject)?;
                if data.revoked() {
                    Err(InternalMethodError::RevokedProxy)
                } else {
                    self.is_extensible(data.target())
                }
            }
            ObjectKind::Ordinary | ObjectKind::Function => self.ordinary_is_extensible(id),
        }
    }

    /// Applies `[[PreventExtensions]]` to one object.
    ///
    /// # Errors
    /// Returns an error when the object record is missing or its runtime state is corrupt.
    pub fn prevent_extensions(
        &mut self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> InternalMethodResult<bool> {
        match self.require_object_kind(id)? {
            ObjectKind::Proxy => {
                let data = self
                    .proxy_data(id)
                    .ok_or(InternalMethodError::MissingObject)?;
                if data.revoked() {
                    Err(InternalMethodError::RevokedProxy)
                } else {
                    self.prevent_extensions(heap, data.target())
                }
            }
            ObjectKind::Ordinary | ObjectKind::Function => {
                if self.is_module_namespace_object(id) {
                    return Ok(true);
                }
                let changed = self.ordinary_prevent_extensions(id)?;
                if changed {
                    return if self.refresh_integrity_level_flags(heap, id) {
                        Ok(true)
                    } else {
                        Err(InternalMethodError::CorruptObjectState)
                    };
                }
                Ok(false)
            }
        }
    }

    /// Returns one own property descriptor from the target object.
    ///
    /// # Errors
    /// Returns an error when the object record is missing or invalid.
    pub fn get_own_property(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
        key: PropertyKey,
    ) -> InternalMethodResult<Option<PropertyDescriptor>> {
        match self.require_object_kind(id)? {
            ObjectKind::Proxy => {
                let data = self
                    .proxy_data(id)
                    .ok_or(InternalMethodError::MissingObject)?;
                if data.revoked() {
                    Err(InternalMethodError::RevokedProxy)
                } else {
                    self.get_own_property(heap, data.target(), key)
                }
            }
            ObjectKind::Ordinary | ObjectKind::Function => {
                if self.is_module_namespace_object(id) {
                    self.module_namespace_get_own_property(heap, id, key)
                } else if self.is_string_exotic_object(id) {
                    self.string_exotic_get_own_property(heap, id, key)
                } else {
                    self.ordinary_get_own_property(heap, id, key)
                }
            }
        }
    }

    /// Defines or updates one own property on the target object.
    ///
    /// # Errors
    /// Returns an error when the object record is missing, the descriptor is invalid, or the
    /// runtime detects corrupt state while applying the update.
    pub fn define_own_property(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        key: PropertyKey,
        descriptor: PropertyDescriptor,
        lifetime: AllocationLifetime,
    ) -> InternalMethodResult<bool> {
        let updated = match self.require_object_kind(id)? {
            ObjectKind::Proxy => {
                let data = self
                    .proxy_data(id)
                    .ok_or(InternalMethodError::MissingObject)?;
                if data.revoked() {
                    Err(InternalMethodError::RevokedProxy)
                } else {
                    self.define_own_property(heap, data.target(), key, descriptor, lifetime)
                }
            }
            ObjectKind::Ordinary | ObjectKind::Function => {
                if self.is_module_namespace_object(id) {
                    self.module_namespace_define_own_property(heap, id, key, descriptor, lifetime)
                } else if self.is_string_exotic_object(id) {
                    self.string_exotic_define_own_property(heap, id, key, descriptor, lifetime)
                } else if self.is_typed_array_object(id) && key.as_index().is_some() {
                    self.typed_array_define_own_property(heap, id, key, descriptor, lifetime)
                } else if self
                    .object_header(heap.view(), id)
                    .is_some_and(|header| header.flags().is_engine_array())
                {
                    self.engine_array_define_own_property(heap, id, key, descriptor, lifetime)
                } else {
                    self.ordinary_define_own_property(heap, id, key, descriptor, lifetime)
                }
            }
        }?;
        if updated && !self.refresh_integrity_level_flags(heap.view(), id) {
            return Err(InternalMethodError::CorruptObjectState);
        }
        Ok(updated)
    }

    /// Reports whether the target object has the requested property on itself or its prototype
    /// chain.
    ///
    /// # Errors
    /// Returns an error when the object record is missing or invalid.
    pub fn has_property(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
        key: PropertyKey,
    ) -> InternalMethodResult<bool> {
        match self.require_object_kind(id)? {
            ObjectKind::Proxy => {
                let data = self
                    .proxy_data(id)
                    .ok_or(InternalMethodError::MissingObject)?;
                if data.revoked() {
                    Err(InternalMethodError::RevokedProxy)
                } else {
                    self.has_property(heap, data.target(), key)
                }
            }
            ObjectKind::Ordinary | ObjectKind::Function => {
                self.ordinary_has_property(heap, id, key)
            }
        }
    }

    /// Reads one property value from the target object.
    ///
    /// # Errors
    /// Returns an error when the object record is missing or invalid.
    pub fn get(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
        key: PropertyKey,
        receiver: Value,
    ) -> InternalMethodResult<Value> {
        match self.require_object_kind(id)? {
            ObjectKind::Proxy => {
                let data = self
                    .proxy_data(id)
                    .ok_or(InternalMethodError::MissingObject)?;
                if data.revoked() {
                    Err(InternalMethodError::RevokedProxy)
                } else {
                    self.get(heap, data.target(), key, receiver)
                }
            }
            ObjectKind::Ordinary | ObjectKind::Function => {
                self.ordinary_get(heap, id, key, receiver)
            }
        }
    }

    /// Writes one property value to the target object.
    ///
    /// # Errors
    /// Returns an error when the object record is missing, the write cannot be represented, or the
    /// runtime detects corrupt state while applying the update.
    pub fn set(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        key: PropertyKey,
        value: Value,
        receiver: Value,
        lifetime: AllocationLifetime,
    ) -> InternalMethodResult<bool> {
        match self.require_object_kind(id)? {
            ObjectKind::Proxy => {
                let data = self
                    .proxy_data(id)
                    .ok_or(InternalMethodError::MissingObject)?;
                if data.revoked() {
                    Err(InternalMethodError::RevokedProxy)
                } else {
                    self.set(heap, data.target(), key, value, receiver, lifetime)
                }
            }
            ObjectKind::Ordinary | ObjectKind::Function => {
                if self
                    .module_namespace_slot(id)
                    .and_then(|namespace| key.as_atom().and_then(|atom| namespace.export(atom)))
                    .is_some()
                {
                    return Ok(false);
                }
                self.ordinary_set(heap, id, key, value, receiver, lifetime)
            }
        }
    }

    /// Deletes one property from the target object.
    ///
    /// # Errors
    /// Returns an error when the object record is missing or invalid.
    pub fn delete(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        key: PropertyKey,
    ) -> InternalMethodResult<bool> {
        match self.require_object_kind(id)? {
            ObjectKind::Proxy => {
                let data = self
                    .proxy_data(id)
                    .ok_or(InternalMethodError::MissingObject)?;
                if data.revoked() {
                    Err(InternalMethodError::RevokedProxy)
                } else {
                    self.delete(heap, data.target(), key)
                }
            }
            ObjectKind::Ordinary | ObjectKind::Function => {
                if self.is_module_namespace_object(id) {
                    self.module_namespace_delete(heap, id, key)
                } else if self.is_string_exotic_object(id) {
                    self.string_exotic_delete(heap, id, key)
                } else if self.is_typed_array_object(id) && key.as_index().is_some() {
                    self.typed_array_delete(id, key)
                } else {
                    self.ordinary_delete(heap, id, key)
                }
            }
        }
    }

    /// Returns the target object's own property keys in ECMAScript enumeration order.
    ///
    /// # Errors
    /// Returns an error when the object record is missing or invalid.
    pub fn own_property_keys(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> InternalMethodResult<Vec<PropertyKey>> {
        match self.require_object_kind(id)? {
            ObjectKind::Proxy => {
                let data = self
                    .proxy_data(id)
                    .ok_or(InternalMethodError::MissingObject)?;
                if data.revoked() {
                    Err(InternalMethodError::RevokedProxy)
                } else {
                    self.own_property_keys(heap, data.target())
                }
            }
            ObjectKind::Ordinary | ObjectKind::Function => {
                if self.is_module_namespace_object(id) {
                    self.module_namespace_own_property_keys(heap, id)
                } else if self.is_string_exotic_object(id) {
                    self.string_exotic_own_property_keys(heap, id)
                } else if self.is_typed_array_object(id) {
                    self.typed_array_own_property_keys(heap, id)
                } else {
                    self.ordinary_own_property_keys(heap, id)
                }
            }
        }
    }

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

        if !receiver_header.flags().uses_named_property_dictionary() {
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

    pub fn set_flags(&mut self, id: ObjectRef, flags: ObjectFlags) -> bool {
        let Some(metadata) = self.object_metadata_mut(id) else {
            return false;
        };
        metadata.flags = if metadata.named_properties.is_dictionary() {
            flags.union(ObjectFlags::NAMED_PROPERTIES_DICTIONARY)
        } else {
            flags.without(ObjectFlags::NAMED_PROPERTIES_DICTIONARY)
        };
        true
    }

    pub fn insert_flags(&mut self, id: ObjectRef, flags: ObjectFlags) -> bool {
        let Some(metadata) = self.object_metadata_mut(id) else {
            return false;
        };
        metadata.flags = if metadata.named_properties.is_dictionary() {
            metadata
                .flags
                .union(flags)
                .union(ObjectFlags::NAMED_PROPERTIES_DICTIONARY)
        } else {
            metadata
                .flags
                .union(flags.without(ObjectFlags::NAMED_PROPERTIES_DICTIONARY))
        };
        true
    }

    pub fn set_prototype(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        prototype: Option<ObjectRef>,
    ) -> bool {
        let Some(record) = heap.view().object(id) else {
            return false;
        };
        if self.object_metadata(id).is_none() {
            return false;
        }
        if record.prototype() == prototype {
            return true;
        }
        if !heap.mut_store_object_handle(ObjectHandleStoreTarget::ObjectPrototype(id), prototype) {
            return false;
        }
        self.bump_invalidation(id, InvalidationCause::PrototypeMutation)
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
            metadata.named_property_churn >= NAMED_PROPERTY_CHURN_DICTIONARY_THRESHOLD
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

        let dictionary = self.snapshot_named_property_dictionary(heap.view(), record);
        let Some(metadata) = self.object_metadata_mut(id) else {
            return false;
        };
        metadata.named_properties = NamedPropertyStorage::Dictionary(dictionary);
        metadata.flags = metadata
            .flags
            .union(ObjectFlags::NAMED_PROPERTIES_DICTIONARY);
        if !heap
            .mut_store_object_slots_handle(ObjectSlotsHandleStoreTarget::ObjectNamedSlots(id), None)
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

    pub(crate) fn refresh_integrity_level_flags(
        &mut self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> bool {
        if self.is_module_namespace_object(id) {
            let Some(metadata) = self.object_metadata_mut(id) else {
                return false;
            };
            metadata.flags = metadata
                .flags
                .without(ObjectFlags::EXTENSIBLE)
                .union(ObjectFlags::SEALED)
                .without(ObjectFlags::FROZEN)
                .without(ObjectFlags::NAMED_PROPERTIES_DICTIONARY);
            return true;
        }
        let computed = {
            let Some(record) = heap.object(id) else {
                return false;
            };
            let Some(metadata) = self.object_metadata(id) else {
                return false;
            };
            let mut flags = metadata
                .flags
                .without(ObjectFlags::SEALED)
                .without(ObjectFlags::FROZEN);
            if flags.is_extensible() {
                Some(flags)
            } else {
                let (sealed, frozen) = self.compute_integrity_summaries(record, metadata);
                if sealed {
                    flags = flags.union(ObjectFlags::SEALED);
                }
                if frozen {
                    flags = flags.union(ObjectFlags::FROZEN);
                }
                Some(flags)
            }
        };
        let Some(computed) = computed else {
            return false;
        };
        let Some(metadata) = self.object_metadata_mut(id) else {
            return false;
        };
        metadata.flags = if metadata.named_properties.is_dictionary() {
            computed.union(ObjectFlags::NAMED_PROPERTIES_DICTIONARY)
        } else {
            computed.without(ObjectFlags::NAMED_PROPERTIES_DICTIONARY)
        };
        true
    }

    fn compute_integrity_summaries(
        &self,
        record: RuntimeObjectRecord,
        metadata: &ObjectMetadata,
    ) -> (bool, bool) {
        let mut sealed = true;
        let mut frozen = true;

        match &metadata.named_properties {
            NamedPropertyStorage::ShapeStable => {
                let Some(shape) = record.shape() else {
                    return (false, false);
                };
                let Some(properties) = self.shape_properties(shape) else {
                    return (false, false);
                };
                for property in properties {
                    update_integrity_flags(
                        property.kind(),
                        property.attrs(),
                        &mut sealed,
                        &mut frozen,
                    );
                    if !sealed && !frozen {
                        return (sealed, frozen);
                    }
                }
            }
            NamedPropertyStorage::Dictionary(dictionary) => {
                for entry in dictionary.entries.values() {
                    update_integrity_flags(
                        entry.payload().kind(),
                        entry.attrs(),
                        &mut sealed,
                        &mut frozen,
                    );
                    if !sealed && !frozen {
                        return (sealed, frozen);
                    }
                }
            }
        }

        match &metadata.element_storage {
            ElementStorageMetadata::Empty => {}
            ElementStorageMetadata::Dense { logical_len } => {
                if *logical_len > 0 {
                    return (false, false);
                }
            }
            ElementStorageMetadata::Sparse { entries, .. } => {
                for entry in entries.values() {
                    update_integrity_flags(
                        entry.payload().kind(),
                        entry.attrs(),
                        &mut sealed,
                        &mut frozen,
                    );
                    if !sealed && !frozen {
                        return (sealed, frozen);
                    }
                }
            }
        }

        (sealed, frozen)
    }

    fn ordinary_get_prototype_of(
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> InternalMethodResult<Option<ObjectRef>> {
        heap.object(id)
            .map(RuntimeObjectRecord::prototype)
            .ok_or(InternalMethodError::MissingObject)
    }

    fn ordinary_set_prototype_of(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        prototype: Option<ObjectRef>,
    ) -> InternalMethodResult<bool> {
        let current = Self::ordinary_get_prototype_of(heap.view(), id)?;
        if current == prototype {
            return Ok(true);
        }
        if !self.ordinary_is_extensible(id)? {
            return Ok(false);
        }
        if let Some(prototype) = prototype {
            if self.prototype_chain_contains(heap.view(), prototype, id)? {
                return Ok(false);
            }
        }
        if self.set_prototype(heap, id, prototype) {
            Ok(true)
        } else {
            Err(InternalMethodError::CorruptObjectState)
        }
    }

    fn ordinary_is_extensible(&self, id: ObjectRef) -> InternalMethodResult<bool> {
        self.object_metadata(id)
            .map(|metadata| metadata.flags.is_extensible())
            .ok_or(InternalMethodError::MissingObject)
    }

    fn ordinary_prevent_extensions(&mut self, id: ObjectRef) -> InternalMethodResult<bool> {
        let Some(metadata) = self.object_metadata_mut(id) else {
            return Err(InternalMethodError::MissingObject);
        };
        metadata.flags = metadata.flags.without(ObjectFlags::EXTENSIBLE);
        Ok(true)
    }

    fn ordinary_get_own_property(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
        key: PropertyKey,
    ) -> InternalMethodResult<Option<PropertyDescriptor>> {
        let _ = self.require_object_kind(id)?;
        if let Some(index) = key.as_index() {
            return self.ordinary_own_index_property(heap, id, index);
        }
        self.ordinary_own_named_property(heap, id, key)
    }

    #[inline]
    fn is_string_exotic_object(&self, id: ObjectRef) -> bool {
        self.primitive_wrapper_kind(id) == Some(crate::PrimitiveWrapperKind::String)
    }

    fn module_namespace_get_own_property(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
        key: PropertyKey,
    ) -> InternalMethodResult<Option<PropertyDescriptor>> {
        let Some(atom) = key.as_atom() else {
            return self.ordinary_get_own_property(heap, id, key);
        };
        let Some(export) = self
            .module_namespace_slot(id)
            .and_then(|namespace| namespace.export(atom))
        else {
            return self.ordinary_get_own_property(heap, id, key);
        };
        let mut descriptor = PropertyDescriptor::new();
        descriptor.set_value(self.module_namespace_export_value(heap, export)?);
        descriptor.set_writable(true);
        descriptor.set_enumerable(true);
        descriptor.set_configurable(false);
        Ok(Some(descriptor))
    }

    fn module_namespace_define_own_property(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        key: PropertyKey,
        descriptor: PropertyDescriptor,
        lifetime: AllocationLifetime,
    ) -> InternalMethodResult<bool> {
        let Some(atom) = key.as_atom() else {
            return self.ordinary_define_own_property(heap, id, key, descriptor, lifetime);
        };
        let Some(export) = self
            .module_namespace_slot(id)
            .and_then(|namespace| namespace.export(atom))
        else {
            return self.ordinary_define_own_property(heap, id, key, descriptor, lifetime);
        };
        if descriptor.getter().is_some() || descriptor.setter().is_some() {
            return Ok(false);
        }
        if descriptor.configurable() == Some(true) {
            return Ok(false);
        }
        if descriptor.enumerable() == Some(false) {
            return Ok(false);
        }
        if descriptor.writable() == Some(false) {
            return Ok(false);
        }
        if let Some(value) = descriptor.value() {
            if value != self.module_namespace_export_value(heap.view(), export)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn string_exotic_get_own_property(
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

    fn string_exotic_define_own_property(
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

    fn typed_array_define_own_property(
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

    fn engine_array_define_own_property(
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
            if !validate_descriptor_change(current, descriptor)? {
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

    fn string_exotic_delete(
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

    fn module_namespace_delete(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        key: PropertyKey,
    ) -> InternalMethodResult<bool> {
        if self
            .module_namespace_slot(id)
            .and_then(|namespace| key.as_atom().and_then(|atom| namespace.export(atom)))
            .is_some()
        {
            return Ok(false);
        }
        self.ordinary_delete(heap, id, key)
    }

    fn module_namespace_own_property_keys(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> InternalMethodResult<Vec<PropertyKey>> {
        let Some(namespace) = self.module_namespace_slot(id) else {
            return Err(InternalMethodError::MissingObject);
        };
        let mut keys = namespace
            .exports()
            .iter()
            .map(|entry| PropertyKey::from_atom(entry.export_name()))
            .collect::<Vec<_>>();
        let (_, mut symbols) = self.collect_own_named_keys(heap, id)?;
        keys.append(&mut symbols);
        Ok(keys)
    }

    fn string_exotic_own_property_keys(
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

    fn typed_array_delete(
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

    fn typed_array_own_property_keys(
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

    fn module_namespace_export_value(
        &self,
        heap: PrimitiveHeapView<'_>,
        export: crate::ModuleNamespaceExport,
    ) -> InternalMethodResult<Value> {
        match export.target() {
            crate::ModuleNamespaceExportTarget::Binding { environment, slot } => {
                let slots = heap
                    .environment(environment)
                    .and_then(|record| record.slots())
                    .and_then(|slots| heap.environment_slots(slots))
                    .ok_or(InternalMethodError::CorruptObjectState)?;
                let value = slots
                    .get(slot as usize)
                    .copied()
                    .ok_or(InternalMethodError::CorruptObjectState)?;
                if value == Value::uninitialized_lexical() {
                    return Err(InternalMethodError::ReferenceError);
                }
                Ok(value)
            }
            crate::ModuleNamespaceExportTarget::Value(value) => Ok(value),
        }
    }

    fn ordinary_define_own_property(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        key: PropertyKey,
        descriptor: PropertyDescriptor,
        lifetime: AllocationLifetime,
    ) -> InternalMethodResult<bool> {
        let _ = self.require_object_kind(id)?;
        let current = self.get_own_property(heap.view(), id, key)?;
        if let Some(current) = current {
            if !validate_descriptor_change(current, descriptor)? {
                return Ok(false);
            }
        } else if !self.ordinary_is_extensible(id)? {
            return Ok(false);
        }

        if let Some(index) = key.as_index() {
            return self.ordinary_define_own_index_property(
                heap, id, index, current, descriptor, lifetime,
            );
        }

        self.ordinary_define_own_named_property(heap, id, key, current, descriptor, lifetime)
    }

    fn ordinary_has_property(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
        key: PropertyKey,
    ) -> InternalMethodResult<bool> {
        if self.get_own_property(heap, id, key)?.is_some() {
            return Ok(true);
        }
        let Some(prototype) = self.get_prototype_of(heap, id)? else {
            return Ok(false);
        };
        self.has_property(heap, prototype, key)
    }

    fn ordinary_get(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
        key: PropertyKey,
        receiver: Value,
    ) -> InternalMethodResult<Value> {
        if let Some(descriptor) = self.get_own_property(heap, id, key)? {
            return resolve_get_from_descriptor(descriptor, receiver);
        }
        let Some(prototype) = self.get_prototype_of(heap, id)? else {
            return Ok(Value::undefined());
        };
        self.get(heap, prototype, key, receiver)
    }

    fn ordinary_set(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        key: PropertyKey,
        value: Value,
        receiver: Value,
        lifetime: AllocationLifetime,
    ) -> InternalMethodResult<bool> {
        if let Some(descriptor) = self.get_own_property(heap.view(), id, key)? {
            return self.ordinary_set_from_descriptor(
                heap, id, key, descriptor, value, receiver, lifetime,
            );
        }

        if let Some(prototype) = self.get_prototype_of(heap.view(), id)? {
            return self.set(heap, prototype, key, value, receiver, lifetime);
        }

        let Some(receiver) = receiver.as_object_ref() else {
            return Ok(false);
        };
        let mut descriptor = PropertyDescriptor::new();
        descriptor.set_value(value);
        descriptor.set_writable(true);
        descriptor.set_enumerable(true);
        descriptor.set_configurable(true);
        self.define_own_property(heap, receiver, key, descriptor, lifetime)
    }

    fn ordinary_delete(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        key: PropertyKey,
    ) -> InternalMethodResult<bool> {
        let Some(descriptor) = self.get_own_property(heap.view(), id, key)? else {
            return Ok(true);
        };
        if !descriptor.configurable().unwrap_or(false) {
            return Ok(false);
        }

        if let Some(index) = key.as_index() {
            return if self.delete_element(heap, id, index) {
                Ok(true)
            } else {
                Err(InternalMethodError::CorruptObjectState)
            };
        }

        if self.delete_named_property(heap, id, key) {
            Ok(true)
        } else {
            Err(InternalMethodError::CorruptObjectState)
        }
    }

    fn ordinary_own_property_keys(
        &self,
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> InternalMethodResult<Vec<PropertyKey>> {
        let _ = self.require_object_kind(id)?;
        let mut keys = self.collect_own_element_keys(heap, id)?;
        let (mut strings, mut symbols) = self.collect_own_named_keys(heap, id)?;
        keys.append(&mut strings);
        keys.append(&mut symbols);
        Ok(keys)
    }

    fn ordinary_own_named_property(
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

    fn ordinary_own_index_property(
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

    fn ordinary_define_own_named_property(
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

    fn ordinary_define_own_index_property(
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

    #[allow(clippy::too_many_arguments)]
    fn ordinary_set_from_descriptor(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        _id: ObjectRef,
        key: PropertyKey,
        descriptor: PropertyDescriptor,
        value: Value,
        receiver: Value,
        lifetime: AllocationLifetime,
    ) -> InternalMethodResult<bool> {
        match descriptor_kind(descriptor)? {
            DescriptorKind::Data => {
                if !descriptor.writable().unwrap_or(false) {
                    return Ok(false);
                }
                let Some(receiver) = receiver.as_object_ref() else {
                    return Ok(false);
                };
                if let Some(receiver_desc) = self.get_own_property(heap.view(), receiver, key)? {
                    match descriptor_kind(receiver_desc)? {
                        DescriptorKind::Accessor => {
                            let setter = receiver_desc.setter().unwrap_or(Value::undefined());
                            if setter == Value::undefined() {
                                Ok(false)
                            } else {
                                Err(InternalMethodError::AccessorCallPending)
                            }
                        }
                        DescriptorKind::Data => {
                            if !receiver_desc.writable().unwrap_or(false) {
                                return Ok(false);
                            }
                            let mut update = PropertyDescriptor::new();
                            update.set_value(value);
                            self.define_own_property(heap, receiver, key, update, lifetime)
                        }
                        DescriptorKind::Generic => unreachable!(),
                    }
                } else {
                    let mut create = PropertyDescriptor::new();
                    create.set_value(value);
                    create.set_writable(true);
                    create.set_enumerable(true);
                    create.set_configurable(true);
                    self.define_own_property(heap, receiver, key, create, lifetime)
                }
            }
            DescriptorKind::Accessor => {
                let setter = descriptor.setter().unwrap_or(Value::undefined());
                if setter == Value::undefined() {
                    Ok(false)
                } else {
                    Err(InternalMethodError::AccessorCallPending)
                }
            }
            DescriptorKind::Generic => unreachable!(),
        }
    }

    fn ordinary_define_absent_shaped_named_property(
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
        let current_shape = record
            .shape()
            .ok_or(InternalMethodError::CorruptObjectState)?;
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
    fn ordinary_update_shaped_named_property(
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

    fn ordinary_store_sparse_index_property(
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

    fn collect_own_element_keys(
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

    fn collect_own_element_indices_descending_from(
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

    fn collect_own_named_keys(
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
            NamedPropertyCachePath::OwnData => {}
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

    fn prototype_chain_contains(
        &self,
        heap: PrimitiveHeapView<'_>,
        start: ObjectRef,
        target: ObjectRef,
    ) -> InternalMethodResult<bool> {
        let mut current = Some(start);
        while let Some(object) = current {
            if object == target {
                return Ok(true);
            }
            current = self.get_prototype_of(heap, object)?;
        }
        Ok(false)
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
