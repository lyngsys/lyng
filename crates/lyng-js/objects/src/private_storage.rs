use super::{
    AllocationLifetime, ClassPrivateElementKind, ClassRecord, InstalledPrivateBrand,
    InternalMethodError, ObjectRef, ObjectRuntime, ObjectSlotsHandleStoreTarget, PrimitiveHeapView,
    PrimitiveMutator, PrivateDescriptorSummary, Value, ValueStoreTarget,
};
use lyng_js_common::AtomId;

impl ObjectRuntime {
    pub fn install_instance_public_field_key(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        class_object: ObjectRef,
        field_index: u32,
        key_value: Value,
        lifetime: AllocationLifetime,
    ) -> Result<(), InternalMethodError> {
        self.object_metadata(class_object)
            .ok_or(InternalMethodError::MissingObject)?;

        let mut record = self
            .class_record_slot(class_object)
            .cloned()
            .unwrap_or_else(|| ClassRecord::new(None, None));
        let field_index =
            usize::try_from(field_index).expect("instance public field index should fit usize");
        while record.instance_public_field_key_slots.len() <= field_index {
            let slot_index = record.static_shared_slot_count;
            record.static_shared_slot_count = record.static_shared_slot_count.saturating_add(1);
            record.instance_public_field_key_slots.push(slot_index);
        }
        let slot_index = record.instance_public_field_key_slots[field_index];
        let required_len = record.static_shared_slot_count;
        self.store_class_record_slot(class_object, record);
        self.ensure_private_shared_storage(heap, class_object, required_len, lifetime)?;
        let slots = heap
            .view()
            .object(class_object)
            .and_then(|record| record.private_slots())
            .ok_or(InternalMethodError::CorruptObjectState)?;
        if !heap.mut_store_value(ValueStoreTarget::ObjectSlot(slots, slot_index), key_value) {
            return Err(InternalMethodError::CorruptObjectState);
        }
        Ok(())
    }

    pub fn instance_public_field_key(
        &self,
        heap: PrimitiveHeapView<'_>,
        class_object: ObjectRef,
        field_index: u32,
    ) -> Result<Value, InternalMethodError> {
        let record = self
            .class_record_slot(class_object)
            .ok_or(InternalMethodError::MissingClassRecord)?;
        let slot_index = record
            .instance_public_field_key_slots
            .get(
                usize::try_from(field_index).expect("instance public field index should fit usize"),
            )
            .copied()
            .ok_or(InternalMethodError::MissingClassRecord)?;
        let slots = heap
            .object(class_object)
            .and_then(|record| record.private_slots())
            .ok_or(InternalMethodError::CorruptObjectState)?;
        heap.object_slots(slots)
            .and_then(|slots| slots.get(slot_index as usize).copied())
            .ok_or(InternalMethodError::CorruptObjectState)
    }

    pub fn private_descriptor_is_static(
        &self,
        class_key: ObjectRef,
        descriptor_index: u32,
    ) -> Option<bool> {
        let record = self.class_record_slot(class_key)?;
        let descriptor = record.descriptors.get(descriptor_index as usize)?;
        Some(descriptor.is_static())
    }

    pub fn private_element_kind(
        &self,
        class_key: ObjectRef,
        descriptor_index: u32,
    ) -> Result<ClassPrivateElementKind, InternalMethodError> {
        let descriptor = self.private_element_descriptor(class_key, descriptor_index)?;
        Ok(descriptor.kind())
    }

    pub fn private_descriptor_summaries(
        &self,
        class_key: ObjectRef,
    ) -> Result<Vec<PrivateDescriptorSummary>, InternalMethodError> {
        let record = self
            .class_record_slot(class_key)
            .ok_or(InternalMethodError::MissingClassRecord)?;
        Ok(record
            .descriptors
            .iter()
            .copied()
            .map(|descriptor| {
                PrivateDescriptorSummary::new(
                    descriptor.name(),
                    descriptor.is_static(),
                    descriptor.kind(),
                )
            })
            .collect())
    }

    pub fn private_shared_element_value(
        &self,
        heap: PrimitiveHeapView<'_>,
        class_key: ObjectRef,
        descriptor_index: u32,
    ) -> Result<Value, InternalMethodError> {
        let descriptor = self.private_element_descriptor(class_key, descriptor_index)?;
        if descriptor.kind() == ClassPrivateElementKind::Field {
            return Err(InternalMethodError::InvalidPrivateElement);
        }
        let slots = heap
            .object(class_key)
            .and_then(|record| record.private_slots())
            .ok_or(InternalMethodError::CorruptObjectState)?;
        heap.object_slots(slots)
            .and_then(|slots| slots.get(descriptor.slot_index() as usize).copied())
            .ok_or(InternalMethodError::CorruptObjectState)
    }

    pub fn install_private_element_value(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        class_key: ObjectRef,
        descriptor_index: u32,
        value: Value,
        lifetime: AllocationLifetime,
    ) -> Result<(), InternalMethodError> {
        let descriptor = self.private_element_descriptor(class_key, descriptor_index)?;
        if descriptor.kind() == ClassPrivateElementKind::Field {
            return Err(InternalMethodError::InvalidPrivateElement);
        }
        let required_len = self.private_shared_slot_count(class_key, descriptor.is_static())?;
        self.ensure_private_shared_storage(heap, class_key, required_len, lifetime)?;
        let slots = heap
            .view()
            .object(class_key)
            .and_then(|record| record.private_slots())
            .ok_or(InternalMethodError::CorruptObjectState)?;
        if !heap.mut_store_value(
            ValueStoreTarget::ObjectSlot(slots, descriptor.slot_index()),
            value,
        ) {
            return Err(InternalMethodError::CorruptObjectState);
        }
        Ok(())
    }

    pub fn define_private_field_layout(
        &mut self,
        class_object: ObjectRef,
        prototype: ObjectRef,
        name: AtomId,
        is_static: bool,
    ) -> Option<u32> {
        self.define_private_element_layout(
            class_object,
            prototype,
            name,
            is_static,
            ClassPrivateElementKind::Field,
        )
    }

    pub fn define_private_element_layout(
        &mut self,
        class_object: ObjectRef,
        prototype: ObjectRef,
        name: AtomId,
        is_static: bool,
        kind: ClassPrivateElementKind,
    ) -> Option<u32> {
        self.object_metadata(class_object)?;
        self.object_metadata(prototype)?;

        let mut record = self
            .class_record_slot(class_object)
            .cloned()
            .or_else(|| self.class_record_slot(prototype).cloned())
            .unwrap_or_else(|| ClassRecord::new(None, None));
        let descriptor_index =
            u32::try_from(record.descriptors.len()).expect("private descriptor index must fit u32");
        let slot_index = match kind {
            ClassPrivateElementKind::Field => {
                if is_static {
                    if record.static_brand.is_none() {
                        record.static_brand = Some(self.alloc_private_brand_id());
                    }
                    let slot_index = record.static_slot_count;
                    record.static_slot_count = record.static_slot_count.saturating_add(1);
                    slot_index
                } else {
                    if record.instance_brand.is_none() {
                        record.instance_brand = Some(self.alloc_private_brand_id());
                    }
                    let slot_index = record.instance_slot_count;
                    record.instance_slot_count = record.instance_slot_count.saturating_add(1);
                    slot_index
                }
            }
            ClassPrivateElementKind::Method
            | ClassPrivateElementKind::Getter
            | ClassPrivateElementKind::Setter => {
                if is_static {
                    if record.static_brand.is_none() {
                        record.static_brand = Some(self.alloc_private_brand_id());
                    }
                    let slot_index = record.static_shared_slot_count;
                    record.static_shared_slot_count =
                        record.static_shared_slot_count.saturating_add(1);
                    slot_index
                } else {
                    if record.instance_brand.is_none() {
                        record.instance_brand = Some(self.alloc_private_brand_id());
                    }
                    let slot_index = record.instance_shared_slot_count;
                    record.instance_shared_slot_count =
                        record.instance_shared_slot_count.saturating_add(1);
                    slot_index
                }
            }
        };
        record
            .descriptors
            .push(super::object_metadata::ClassPrivateElementDescriptor::new(
                name, is_static, kind, slot_index,
            ));

        self.store_class_record_slot(class_object, record.clone());
        self.store_class_record_slot(prototype, record);
        Some(descriptor_index)
    }

    pub fn private_field_init(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        receiver: ObjectRef,
        class_key: ObjectRef,
        descriptor_index: u32,
        value: Value,
        lifetime: AllocationLifetime,
    ) -> Result<(), InternalMethodError> {
        let (descriptor, brand, field_slot_count, shared_slot_count) =
            self.private_element_brand_descriptor_layout(class_key, descriptor_index)?;
        let slot_base = self.ensure_private_brand_storage(
            heap,
            receiver,
            brand,
            field_slot_count.saturating_add(shared_slot_count),
            lifetime,
        )?;
        let record = heap
            .view()
            .object(receiver)
            .ok_or(InternalMethodError::MissingObject)?;
        let slots = record
            .private_slots()
            .ok_or(InternalMethodError::CorruptObjectState)?;
        let slot_index =
            slot_base.saturating_add(self.private_brand_storage_slot(descriptor, field_slot_count));
        let existing = heap
            .view()
            .object_slots(slots)
            .and_then(|slots| slots.get(slot_index as usize).copied())
            .ok_or(InternalMethodError::CorruptObjectState)?;
        if existing != Value::empty_internal_slot() {
            return Err(InternalMethodError::DuplicatePrivateElement);
        }
        let stored = if descriptor.kind() == ClassPrivateElementKind::Field {
            value
        } else {
            Value::undefined()
        };
        if !heap.mut_store_value(ValueStoreTarget::ObjectSlot(slots, slot_index), stored) {
            return Err(InternalMethodError::CorruptObjectState);
        }
        Ok(())
    }

    pub fn private_field_get(
        &self,
        heap: PrimitiveHeapView<'_>,
        receiver: ObjectRef,
        class_key: ObjectRef,
        descriptor_index: u32,
    ) -> Result<Value, InternalMethodError> {
        let (descriptor, brand, _slot_count) =
            self.private_element_brand_descriptor(class_key, descriptor_index)?;
        if descriptor.kind() != ClassPrivateElementKind::Field {
            return Err(InternalMethodError::InvalidPrivateElement);
        }
        let brand = self
            .installed_private_brand(receiver, brand)
            .ok_or(InternalMethodError::InvalidPrivateBrand)?;
        let record = heap
            .object(receiver)
            .ok_or(InternalMethodError::MissingObject)?;
        let slots = record
            .private_slots()
            .ok_or(InternalMethodError::CorruptObjectState)?;
        heap.object_slots(slots)
            .and_then(|slots| {
                slots
                    .get(brand.slot_base().saturating_add(descriptor.slot_index()) as usize)
                    .copied()
            })
            .ok_or(InternalMethodError::CorruptObjectState)
    }

    pub fn private_field_set(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        receiver: ObjectRef,
        class_key: ObjectRef,
        descriptor_index: u32,
        value: Value,
    ) -> Result<(), InternalMethodError> {
        let (descriptor, brand, _slot_count) =
            self.private_element_brand_descriptor(class_key, descriptor_index)?;
        if descriptor.kind() != ClassPrivateElementKind::Field {
            return Err(InternalMethodError::InvalidPrivateElement);
        }
        let brand = self
            .installed_private_brand(receiver, brand)
            .ok_or(InternalMethodError::InvalidPrivateBrand)?;
        let record = heap
            .view()
            .object(receiver)
            .ok_or(InternalMethodError::MissingObject)?;
        let slots = record
            .private_slots()
            .ok_or(InternalMethodError::CorruptObjectState)?;
        if !heap.mut_store_value(
            ValueStoreTarget::ObjectSlot(
                slots,
                brand.slot_base().saturating_add(descriptor.slot_index()),
            ),
            value,
        ) {
            return Err(InternalMethodError::CorruptObjectState);
        }
        Ok(())
    }

    pub fn private_has(
        &self,
        receiver: ObjectRef,
        class_key: ObjectRef,
        descriptor_index: u32,
    ) -> Result<bool, InternalMethodError> {
        let (_descriptor, brand, _slot_count) =
            self.private_element_brand_descriptor(class_key, descriptor_index)?;
        Ok(self.installed_private_brand(receiver, brand).is_some())
    }

    fn private_element_descriptor(
        &self,
        class_key: ObjectRef,
        descriptor_index: u32,
    ) -> Result<super::object_metadata::ClassPrivateElementDescriptor, InternalMethodError> {
        let record = self
            .class_record_slot(class_key)
            .ok_or(InternalMethodError::MissingClassRecord)?;
        record
            .descriptors
            .get(descriptor_index as usize)
            .copied()
            .ok_or(InternalMethodError::InvalidPrivateElement)
    }

    fn private_element_brand_descriptor(
        &self,
        class_key: ObjectRef,
        descriptor_index: u32,
    ) -> Result<
        (
            super::object_metadata::ClassPrivateElementDescriptor,
            super::object_metadata::PrivateBrandId,
            u32,
        ),
        InternalMethodError,
    > {
        let (descriptor, brand, field_slot_count, _shared_slot_count) =
            self.private_element_brand_descriptor_layout(class_key, descriptor_index)?;
        Ok((descriptor, brand, field_slot_count))
    }

    fn private_element_brand_descriptor_layout(
        &self,
        class_key: ObjectRef,
        descriptor_index: u32,
    ) -> Result<
        (
            super::object_metadata::ClassPrivateElementDescriptor,
            super::object_metadata::PrivateBrandId,
            u32,
            u32,
        ),
        InternalMethodError,
    > {
        let record = self
            .class_record_slot(class_key)
            .ok_or(InternalMethodError::MissingClassRecord)?;
        let descriptor = record
            .descriptors
            .get(descriptor_index as usize)
            .copied()
            .ok_or(InternalMethodError::InvalidPrivateElement)?;
        let (brand, field_slot_count, shared_slot_count) = if descriptor.is_static() {
            (
                record
                    .static_brand
                    .ok_or(InternalMethodError::InvalidPrivateElement)?,
                record.static_slot_count,
                record.static_shared_slot_count,
            )
        } else {
            (
                record
                    .instance_brand
                    .ok_or(InternalMethodError::InvalidPrivateElement)?,
                record.instance_slot_count,
                record.instance_shared_slot_count,
            )
        };
        Ok((descriptor, brand, field_slot_count, shared_slot_count))
    }

    fn private_shared_slot_count(
        &self,
        class_key: ObjectRef,
        is_static: bool,
    ) -> Result<u32, InternalMethodError> {
        let record = self
            .class_record_slot(class_key)
            .ok_or(InternalMethodError::MissingClassRecord)?;
        Ok(if is_static {
            record.static_shared_slot_count
        } else {
            record.instance_shared_slot_count
        })
    }

    fn private_brand_storage_slot(
        &self,
        descriptor: super::object_metadata::ClassPrivateElementDescriptor,
        field_slot_count: u32,
    ) -> u32 {
        match descriptor.kind() {
            ClassPrivateElementKind::Field => descriptor.slot_index(),
            ClassPrivateElementKind::Method
            | ClassPrivateElementKind::Getter
            | ClassPrivateElementKind::Setter => {
                field_slot_count.saturating_add(descriptor.slot_index())
            }
        }
    }

    fn ensure_private_brand_storage(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        receiver: ObjectRef,
        brand: super::object_metadata::PrivateBrandId,
        slot_count: u32,
        lifetime: AllocationLifetime,
    ) -> Result<u32, InternalMethodError> {
        if let Some(installed) = self.installed_private_brand(receiver, brand) {
            if installed.slot_count() >= slot_count {
                return Ok(installed.slot_base());
            }

            let insertion_at =
                installed.slot_base().saturating_add(installed.slot_count()) as usize;
            let inserted = usize::try_from(slot_count.saturating_sub(installed.slot_count()))
                .expect("expanded brand slot span should fit usize");
            let record = heap
                .view()
                .object(receiver)
                .ok_or(InternalMethodError::MissingObject)?;
            let existing_slots = record.private_slots();
            let existing_len = existing_slots
                .and_then(|slots| heap.view().object_slots(slots))
                .map_or(0usize, <[Value]>::len);
            let new_len = existing_len.saturating_add(inserted);
            let new_slots =
                heap.alloc_object_slots(new_len, Value::empty_internal_slot(), lifetime);
            if let Some(existing) = existing_slots.and_then(|slots| heap.view().object_slots(slots))
            {
                let copied = existing.to_vec();
                for (index, value) in copied.into_iter().enumerate() {
                    let destination = if index < insertion_at {
                        index
                    } else {
                        index.saturating_add(inserted)
                    };
                    if !heap.mut_store_value(
                        ValueStoreTarget::ObjectSlot(
                            new_slots,
                            u32::try_from(destination)
                                .expect("shifted brand slot index should fit u32"),
                        ),
                        value,
                    ) {
                        return Err(InternalMethodError::CorruptObjectState);
                    }
                }
            }
            if !heap.mut_store_object_slots_handle(
                ObjectSlotsHandleStoreTarget::ObjectPrivateSlots(receiver),
                Some(new_slots),
            ) {
                return Err(InternalMethodError::CorruptObjectState);
            }

            let metadata = self
                .object_metadata_mut(receiver)
                .ok_or(InternalMethodError::MissingObject)?;
            for installed_brand in &mut metadata.private_brands {
                if installed_brand.brand() == brand {
                    *installed_brand =
                        InstalledPrivateBrand::new(brand, installed.slot_base(), slot_count);
                } else if installed_brand.slot_base() as usize >= insertion_at {
                    *installed_brand = InstalledPrivateBrand::new(
                        installed_brand.brand(),
                        installed_brand.slot_base().saturating_add(
                            u32::try_from(inserted)
                                .expect("inserted brand slot span should fit u32"),
                        ),
                        installed_brand.slot_count(),
                    );
                }
            }
            return Ok(installed.slot_base());
        }

        let existing_len = self.private_slot_len(heap.view(), receiver)?;
        let slot_base = u32::try_from(existing_len).expect("private slot base should fit into u32");
        let requested_len = existing_len.saturating_add(slot_count as usize);
        self.ensure_private_slot_capacity(heap, receiver, requested_len, lifetime)?;
        let metadata = self
            .object_metadata_mut(receiver)
            .ok_or(InternalMethodError::MissingObject)?;
        metadata
            .private_brands
            .push(InstalledPrivateBrand::new(brand, slot_base, slot_count));
        Ok(slot_base)
    }

    fn ensure_private_shared_storage(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        object: ObjectRef,
        slot_count: u32,
        lifetime: AllocationLifetime,
    ) -> Result<(), InternalMethodError> {
        self.object_metadata(object)
            .ok_or(InternalMethodError::MissingObject)?;
        let record = heap
            .view()
            .object(object)
            .ok_or(InternalMethodError::MissingObject)?;
        let existing_slots = record.private_slots();
        let existing_len = existing_slots
            .and_then(|slots| heap.view().object_slots(slots))
            .map_or(0usize, <[Value]>::len);
        let requested_len = slot_count as usize;
        let brand_start = self
            .object_metadata(object)
            .ok_or(InternalMethodError::MissingObject)?
            .private_brands
            .iter()
            .map(|brand| brand.slot_base() as usize)
            .min()
            .unwrap_or(existing_len);
        if existing_len >= requested_len && brand_start >= requested_len {
            return Ok(());
        }
        if brand_start >= requested_len {
            return self.ensure_private_slot_capacity(heap, object, requested_len, lifetime);
        }

        let inserted = requested_len.saturating_sub(brand_start);
        let new_len = existing_len.saturating_add(inserted);
        let new_slots = heap.alloc_object_slots(new_len, Value::empty_internal_slot(), lifetime);
        if let Some(existing) = existing_slots.and_then(|slots| heap.view().object_slots(slots)) {
            let copied = existing.to_vec();
            for (index, value) in copied.into_iter().enumerate() {
                let destination = if index < brand_start {
                    index
                } else {
                    index.saturating_add(inserted)
                };
                if !heap.mut_store_value(
                    ValueStoreTarget::ObjectSlot(
                        new_slots,
                        u32::try_from(destination)
                            .expect("shifted private slot index should fit u32"),
                    ),
                    value,
                ) {
                    return Err(InternalMethodError::CorruptObjectState);
                }
            }
        }

        if !heap.mut_store_object_slots_handle(
            ObjectSlotsHandleStoreTarget::ObjectPrivateSlots(object),
            Some(new_slots),
        ) {
            return Err(InternalMethodError::CorruptObjectState);
        }

        let metadata = self
            .object_metadata_mut(object)
            .ok_or(InternalMethodError::MissingObject)?;
        for brand in &mut metadata.private_brands {
            if brand.slot_base() as usize >= brand_start {
                *brand = InstalledPrivateBrand::new(
                    brand.brand(),
                    brand.slot_base().saturating_add(
                        u32::try_from(inserted).expect("inserted private slot span should fit u32"),
                    ),
                    brand.slot_count(),
                );
            }
        }
        Ok(())
    }

    fn ensure_private_slot_capacity(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        object: ObjectRef,
        requested_len: usize,
        lifetime: AllocationLifetime,
    ) -> Result<(), InternalMethodError> {
        self.object_metadata(object)
            .ok_or(InternalMethodError::MissingObject)?;
        let record = heap
            .view()
            .object(object)
            .ok_or(InternalMethodError::MissingObject)?;
        let existing_slots = record.private_slots();
        let existing_len = existing_slots
            .and_then(|slots| heap.view().object_slots(slots))
            .map_or(0usize, <[Value]>::len);
        if existing_len >= requested_len {
            return Ok(());
        }
        let new_slots = if requested_len == 0 {
            None
        } else {
            let new_slots =
                heap.alloc_object_slots(requested_len, Value::empty_internal_slot(), lifetime);
            if let Some(existing) = existing_slots.and_then(|slots| heap.view().object_slots(slots))
            {
                let copied = existing.to_vec();
                for (index, value) in copied.into_iter().enumerate() {
                    if !heap.mut_store_value(
                        ValueStoreTarget::ObjectSlot(
                            new_slots,
                            u32::try_from(index).expect("private slot copy index should fit u32"),
                        ),
                        value,
                    ) {
                        return Err(InternalMethodError::CorruptObjectState);
                    }
                }
            }
            Some(new_slots)
        };

        if !heap.mut_store_object_slots_handle(
            ObjectSlotsHandleStoreTarget::ObjectPrivateSlots(object),
            new_slots,
        ) {
            return Err(InternalMethodError::CorruptObjectState);
        }
        Ok(())
    }

    fn private_slot_len(
        &self,
        heap: PrimitiveHeapView<'_>,
        object: ObjectRef,
    ) -> Result<usize, InternalMethodError> {
        self.object_metadata(object)
            .ok_or(InternalMethodError::MissingObject)?;
        Ok(heap
            .object(object)
            .and_then(|record| record.private_slots())
            .and_then(|slots| heap.object_slots(slots))
            .map_or(0usize, <[Value]>::len))
    }

    fn installed_private_brand(
        &self,
        receiver: ObjectRef,
        brand: super::object_metadata::PrivateBrandId,
    ) -> Option<InstalledPrivateBrand> {
        self.object_metadata(receiver)?
            .private_brands
            .iter()
            .copied()
            .find(|installed| installed.brand() == brand)
    }

    fn alloc_private_brand_id(&mut self) -> super::object_metadata::PrivateBrandId {
        self.next_private_brand_raw = self.next_private_brand_raw.saturating_add(1).max(1);
        super::object_metadata::PrivateBrandId::new(self.next_private_brand_raw)
    }
}
