use crate::{internal::internal_builtin_metadata, BuiltinEntryMetadata};
use lyng_js_types::BuiltinFunctionId;

mod binary_data;
mod core;
mod temporal;

use self::binary_data::PUBLIC_BINARY_DATA_BUILTIN_METADATA;
use self::core::{
    PUBLIC_ARRAY_BUILTIN_METADATA, PUBLIC_DATE_BUILTIN_METADATA, PUBLIC_FUNCTION_BUILTIN_METADATA,
    PUBLIC_KEYED_COLLECTION_BUILTIN_METADATA, PUBLIC_LANGUAGE_SUPPORT_BUILTIN_METADATA,
    PUBLIC_MODULE_BUILTIN_METADATA, PUBLIC_OBJECT_BUILTIN_METADATA,
    PUBLIC_OBJECT_REFLECTION_BUILTIN_METADATA, PUBLIC_PRIMITIVE_BUILTIN_METADATA,
    PUBLIC_REGEXP_BUILTIN_METADATA, PUBLIC_TEXT_BUILTIN_METADATA, PUBLIC_WEAK_REF_BUILTIN_METADATA,
};

/// Compatibility metadata for one public builtin entry.
#[derive(Clone, Copy, Debug)]
struct PublicBuiltinMetadataRow {
    entry: BuiltinFunctionId,
    metadata: BuiltinEntryMetadata,
}

impl PublicBuiltinMetadataRow {
    #[inline]
    const fn new(entry: BuiltinFunctionId, metadata: BuiltinEntryMetadata) -> Self {
        Self { entry, metadata }
    }

    #[cfg(test)]
    #[inline]
    const fn entry(self) -> BuiltinFunctionId {
        self.entry
    }

    #[cfg(test)]
    #[inline]
    const fn metadata(self) -> BuiltinEntryMetadata {
        self.metadata
    }

    #[inline]
    fn metadata_for(self, entry: BuiltinFunctionId) -> Option<BuiltinEntryMetadata> {
        if self.entry == entry {
            Some(self.metadata)
        } else {
            None
        }
    }
}

/// Named metadata table for one public builtin family or subfamily.
#[derive(Clone, Copy, Debug)]
struct PublicBuiltinMetadataGroup {
    #[cfg(test)]
    name: &'static str,
    rows: &'static [PublicBuiltinMetadataRow],
}

impl PublicBuiltinMetadataGroup {
    #[inline]
    const fn new(name: &'static str, rows: &'static [PublicBuiltinMetadataRow]) -> Self {
        let _ = name;
        Self {
            #[cfg(test)]
            name,
            rows,
        }
    }

    #[cfg(test)]
    #[inline]
    const fn name(self) -> &'static str {
        self.name
    }

    #[cfg(test)]
    #[inline]
    const fn rows(self) -> &'static [PublicBuiltinMetadataRow] {
        self.rows
    }

    #[inline]
    fn metadata_for(self, entry: BuiltinFunctionId) -> Option<BuiltinEntryMetadata> {
        self.rows.iter().find_map(|row| row.metadata_for(entry))
    }
}

const PUBLIC_BUILTIN_METADATA_GROUPS: &[PublicBuiltinMetadataGroup] = &[
    PublicBuiltinMetadataGroup::new("object", PUBLIC_OBJECT_BUILTIN_METADATA),
    PublicBuiltinMetadataGroup::new("function", PUBLIC_FUNCTION_BUILTIN_METADATA),
    PublicBuiltinMetadataGroup::new("array", PUBLIC_ARRAY_BUILTIN_METADATA),
    PublicBuiltinMetadataGroup::new("keyed_collection", PUBLIC_KEYED_COLLECTION_BUILTIN_METADATA),
    PublicBuiltinMetadataGroup::new("weak_ref", PUBLIC_WEAK_REF_BUILTIN_METADATA),
    PublicBuiltinMetadataGroup::new("module", PUBLIC_MODULE_BUILTIN_METADATA),
    PublicBuiltinMetadataGroup::new("binary_data", PUBLIC_BINARY_DATA_BUILTIN_METADATA),
    PublicBuiltinMetadataGroup::new(
        "object_reflection",
        PUBLIC_OBJECT_REFLECTION_BUILTIN_METADATA,
    ),
    PublicBuiltinMetadataGroup::new("text", PUBLIC_TEXT_BUILTIN_METADATA),
    PublicBuiltinMetadataGroup::new("regexp", PUBLIC_REGEXP_BUILTIN_METADATA),
    PublicBuiltinMetadataGroup::new("date", PUBLIC_DATE_BUILTIN_METADATA),
    PublicBuiltinMetadataGroup::new(
        "temporal_instant",
        temporal::PUBLIC_TEMPORAL_INSTANT_BUILTIN_METADATA,
    ),
    PublicBuiltinMetadataGroup::new(
        "temporal_now",
        temporal::PUBLIC_TEMPORAL_NOW_BUILTIN_METADATA,
    ),
    PublicBuiltinMetadataGroup::new(
        "temporal_duration",
        temporal::PUBLIC_TEMPORAL_DURATION_BUILTIN_METADATA,
    ),
    PublicBuiltinMetadataGroup::new(
        "temporal_plain_date",
        temporal::PUBLIC_TEMPORAL_PLAIN_DATE_BUILTIN_METADATA,
    ),
    PublicBuiltinMetadataGroup::new(
        "temporal_plain_time",
        temporal::PUBLIC_TEMPORAL_PLAIN_TIME_BUILTIN_METADATA,
    ),
    PublicBuiltinMetadataGroup::new(
        "temporal_plain_date_time",
        temporal::PUBLIC_TEMPORAL_PLAIN_DATE_TIME_BUILTIN_METADATA,
    ),
    PublicBuiltinMetadataGroup::new(
        "temporal_plain_year_month",
        temporal::PUBLIC_TEMPORAL_PLAIN_YEAR_MONTH_BUILTIN_METADATA,
    ),
    PublicBuiltinMetadataGroup::new(
        "temporal_plain_month_day",
        temporal::PUBLIC_TEMPORAL_PLAIN_MONTH_DAY_BUILTIN_METADATA,
    ),
    PublicBuiltinMetadataGroup::new(
        "temporal_zoned_date_time",
        temporal::PUBLIC_TEMPORAL_ZONED_DATE_TIME_BUILTIN_METADATA,
    ),
    PublicBuiltinMetadataGroup::new("primitive", PUBLIC_PRIMITIVE_BUILTIN_METADATA),
    PublicBuiltinMetadataGroup::new("language_support", PUBLIC_LANGUAGE_SUPPORT_BUILTIN_METADATA),
];

/// Compatibility metadata for the public builtin namespace.
#[inline]
pub fn public_builtin_metadata(entry: BuiltinFunctionId) -> Option<BuiltinEntryMetadata> {
    PUBLIC_BUILTIN_METADATA_GROUPS
        .iter()
        .find_map(|group| group.metadata_for(entry))
}

#[inline]
pub fn builtin_metadata(entry: BuiltinFunctionId) -> Option<BuiltinEntryMetadata> {
    public_builtin_metadata(entry).or_else(|| internal_builtin_metadata(entry))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_types::BuiltinIdNamespace;
    use std::collections::HashSet;

    #[test]
    fn public_metadata_registry_rows_are_unique_registered_public_ids() {
        let mut seen = HashSet::new();

        for group in PUBLIC_BUILTIN_METADATA_GROUPS {
            assert!(!group.name().is_empty());
            assert!(
                !group.rows().is_empty(),
                "metadata group {} must contain rows",
                group.name()
            );

            for row in group.rows() {
                let entry = row.entry();
                assert!(
                    seen.insert(entry),
                    "duplicate public metadata entry {:?} in {}",
                    entry,
                    group.name()
                );
                let Some(id_entry) = lyng_js_types::builtin_id_registry_entry(entry) else {
                    panic!("metadata row {entry:?} missing from builtin id registry");
                };
                assert_ne!(id_entry.namespace(), BuiltinIdNamespace::Internal);
                assert_eq!(public_builtin_metadata(entry), Some(row.metadata()));
                assert_eq!(builtin_metadata(entry), Some(row.metadata()));
            }
        }
    }

    #[test]
    fn public_metadata_registry_keeps_representative_entries_stable() {
        assert_eq!(
            public_builtin_metadata(lyng_js_types::object_builtin()),
            Some(BuiltinEntryMetadata::new("Object", 1, true, true))
        );
        assert_eq!(
            public_builtin_metadata(lyng_js_types::array_from_builtin()),
            Some(BuiltinEntryMetadata::new("from", 1, false, false))
        );
        assert_eq!(
            public_builtin_metadata(lyng_js_types::promise_then_builtin()),
            Some(BuiltinEntryMetadata::new("then", 2, false, false))
        );
        assert_eq!(
            public_builtin_metadata(lyng_js_types::temporal_instant_builtin()),
            Some(BuiltinEntryMetadata::new("Instant", 1, true, true))
        );
        assert_eq!(
            public_builtin_metadata(lyng_js_types::uint8_array_to_base64_builtin()),
            Some(BuiltinEntryMetadata::new("toBase64", 0, false, false))
        );
    }
}
