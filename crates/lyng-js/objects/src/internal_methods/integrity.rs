use super::*;

impl ObjectRuntime {
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
        let Some(record) = heap.object(id) else {
            return false;
        };
        let Some(metadata) = self.object_metadata(id) else {
            return false;
        };
        let mut computed = metadata
            .flags
            .without(ObjectFlags::SEALED)
            .without(ObjectFlags::FROZEN);
        if !computed.is_extensible() {
            let (sealed, frozen) = self.compute_integrity_summaries(record, metadata);
            if sealed {
                computed = computed.union(ObjectFlags::SEALED);
            }
            if frozen {
                computed = computed.union(ObjectFlags::FROZEN);
            }
        }
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
}
