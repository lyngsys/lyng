use super::*;

impl ObjectRuntime {
    pub(super) fn ordinary_get_prototype_of(
        heap: PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> InternalMethodResult<Option<ObjectRef>> {
        heap.object(id)
            .map(RuntimeObjectRecord::prototype)
            .ok_or(InternalMethodError::MissingObject)
    }

    pub(super) fn ordinary_set_prototype_of(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        prototype: Option<ObjectRef>,
    ) -> InternalMethodResult<bool> {
        let current = Self::ordinary_get_prototype_of(heap.view(), id)?;
        if current == prototype {
            return Ok(true);
        }
        if self
            .object_metadata(id)
            .is_some_and(|metadata| metadata.flags.has_immutable_prototype())
        {
            return Ok(false);
        }
        if !self.ordinary_is_extensible(id)? {
            return Ok(false);
        }
        if let Some(prototype) = prototype
            && self.prototype_chain_contains(heap.view(), prototype, id)?
        {
            return Ok(false);
        }
        if self.set_prototype(heap, id, prototype) {
            Ok(true)
        } else {
            Err(InternalMethodError::CorruptObjectState)
        }
    }

    pub(super) fn ordinary_is_extensible(&self, id: ObjectRef) -> InternalMethodResult<bool> {
        self.object_metadata(id)
            .map(|metadata| metadata.flags.is_extensible())
            .ok_or(InternalMethodError::MissingObject)
    }

    pub(super) fn ordinary_prevent_extensions(
        &mut self,
        id: ObjectRef,
    ) -> InternalMethodResult<bool> {
        let Some(metadata) = self.object_metadata_mut(id) else {
            return Err(InternalMethodError::MissingObject);
        };
        metadata.flags = metadata.flags.without(ObjectFlags::EXTENSIBLE);
        Ok(true)
    }

    pub(super) fn ordinary_get_own_property(
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

    pub(super) fn ordinary_define_own_property(
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
            if !validate_descriptor_change(heap.view(), current, descriptor)? {
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

    pub(super) fn ordinary_has_property(
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

    pub(super) fn ordinary_get(
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

    pub(super) fn ordinary_set(
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

    pub(super) fn ordinary_delete(
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

    pub(super) fn ordinary_own_property_keys(
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
            if self.require_object_kind(object)? == ObjectKind::Proxy {
                return Ok(false);
            }
            current = Self::ordinary_get_prototype_of(heap, object)?;
        }
        Ok(false)
    }
}
