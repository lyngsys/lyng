#![allow(
    clippy::wildcard_imports,
    reason = "internal method dispatch keeps object-operation imports local to this narrow module"
)]

use super::*;
use lyng_js_common::WellKnownAtom;

mod elements;
mod engine_arrays;
mod integrity;
mod module_namespace;
mod named_properties;
mod ordinary;
mod property_cache;
mod string_exotics;
mod typed_arrays;

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
                if self.typed_array_prevent_extensions_rejected(id)? {
                    return Ok(false);
                }
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
                if self.is_module_namespace_object(id) {
                    self.module_namespace_has_property(heap, id, key)
                } else {
                    self.ordinary_has_property(heap, id, key)
                }
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
                if self.is_module_namespace_object(id) {
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
        self.bump_invalidation(heap, id, InvalidationCause::PrototypeMutation)
    }
}
