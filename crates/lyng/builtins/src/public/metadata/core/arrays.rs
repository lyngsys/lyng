use super::super::{BuiltinEntryMetadata, PublicBuiltinMetadataRow};
use lyng_js_types::{
    array_at_builtin, array_builtin, array_concat_builtin, array_copy_within_builtin,
    array_entries_builtin, array_every_builtin, array_fill_builtin, array_filter_builtin,
    array_find_builtin, array_find_index_builtin, array_find_last_builtin,
    array_find_last_index_builtin, array_flat_builtin, array_flat_map_builtin,
    array_for_each_builtin, array_from_async_builtin, array_from_builtin, array_includes_builtin,
    array_index_of_builtin, array_is_array_builtin, array_join_builtin, array_keys_builtin,
    array_last_index_of_builtin, array_map_builtin, array_of_builtin, array_pop_builtin,
    array_push_builtin, array_reduce_builtin, array_reduce_right_builtin, array_reverse_builtin,
    array_shift_builtin, array_slice_builtin, array_some_builtin, array_sort_builtin,
    array_splice_builtin, array_to_locale_string_builtin, array_to_reversed_builtin,
    array_to_sorted_builtin, array_to_spliced_builtin, array_to_string_builtin,
    array_unshift_builtin, array_values_builtin, array_with_builtin,
};
pub(in crate::public::metadata) const PUBLIC_ARRAY_BUILTIN_METADATA: &[PublicBuiltinMetadataRow] =
    &[
        PublicBuiltinMetadataRow::new(
            array_builtin(),
            BuiltinEntryMetadata::new("Array", 1, true, true),
        ),
        PublicBuiltinMetadataRow::new(
            array_from_builtin(),
            BuiltinEntryMetadata::new("from", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_from_async_builtin(),
            BuiltinEntryMetadata::new("fromAsync", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_of_builtin(),
            BuiltinEntryMetadata::new("of", 0, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_is_array_builtin(),
            BuiltinEntryMetadata::new("isArray", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_at_builtin(),
            BuiltinEntryMetadata::new("at", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_concat_builtin(),
            BuiltinEntryMetadata::new("concat", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_copy_within_builtin(),
            BuiltinEntryMetadata::new("copyWithin", 2, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_fill_builtin(),
            BuiltinEntryMetadata::new("fill", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_join_builtin(),
            BuiltinEntryMetadata::new("join", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_pop_builtin(),
            BuiltinEntryMetadata::new("pop", 0, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_push_builtin(),
            BuiltinEntryMetadata::new("push", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_shift_builtin(),
            BuiltinEntryMetadata::new("shift", 0, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_unshift_builtin(),
            BuiltinEntryMetadata::new("unshift", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_every_builtin(),
            BuiltinEntryMetadata::new("every", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_filter_builtin(),
            BuiltinEntryMetadata::new("filter", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_flat_builtin(),
            BuiltinEntryMetadata::new("flat", 0, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_flat_map_builtin(),
            BuiltinEntryMetadata::new("flatMap", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_find_builtin(),
            BuiltinEntryMetadata::new("find", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_find_index_builtin(),
            BuiltinEntryMetadata::new("findIndex", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_find_last_builtin(),
            BuiltinEntryMetadata::new("findLast", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_find_last_index_builtin(),
            BuiltinEntryMetadata::new("findLastIndex", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_for_each_builtin(),
            BuiltinEntryMetadata::new("forEach", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_includes_builtin(),
            BuiltinEntryMetadata::new("includes", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_index_of_builtin(),
            BuiltinEntryMetadata::new("indexOf", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_map_builtin(),
            BuiltinEntryMetadata::new("map", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_reduce_builtin(),
            BuiltinEntryMetadata::new("reduce", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_reduce_right_builtin(),
            BuiltinEntryMetadata::new("reduceRight", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_reverse_builtin(),
            BuiltinEntryMetadata::new("reverse", 0, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_slice_builtin(),
            BuiltinEntryMetadata::new("slice", 2, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_some_builtin(),
            BuiltinEntryMetadata::new("some", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_last_index_of_builtin(),
            BuiltinEntryMetadata::new("lastIndexOf", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_sort_builtin(),
            BuiltinEntryMetadata::new("sort", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_splice_builtin(),
            BuiltinEntryMetadata::new("splice", 2, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_to_reversed_builtin(),
            BuiltinEntryMetadata::new("toReversed", 0, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_to_sorted_builtin(),
            BuiltinEntryMetadata::new("toSorted", 1, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_to_spliced_builtin(),
            BuiltinEntryMetadata::new("toSpliced", 2, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_to_string_builtin(),
            BuiltinEntryMetadata::new("toString", 0, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_to_locale_string_builtin(),
            BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_values_builtin(),
            BuiltinEntryMetadata::new("values", 0, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_keys_builtin(),
            BuiltinEntryMetadata::new("keys", 0, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_entries_builtin(),
            BuiltinEntryMetadata::new("entries", 0, false, false),
        ),
        PublicBuiltinMetadataRow::new(
            array_with_builtin(),
            BuiltinEntryMetadata::new("with", 2, false, false),
        ),
    ];
