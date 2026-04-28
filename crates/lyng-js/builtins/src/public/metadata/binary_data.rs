use super::*;

pub(super) const PUBLIC_BINARY_DATA_BUILTIN_METADATA: &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        array_buffer_builtin,
        BuiltinEntryMetadata::new("ArrayBuffer", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        array_buffer_is_view_builtin,
        BuiltinEntryMetadata::new("isView", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        shared_array_buffer_builtin,
        BuiltinEntryMetadata::new("SharedArrayBuffer", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_builtin,
        BuiltinEntryMetadata::new("DataView", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_builtin,
        BuiltinEntryMetadata::new("TypedArray", 0, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_from_builtin,
        BuiltinEntryMetadata::new("from", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_of_builtin,
        BuiltinEntryMetadata::new("of", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_every_builtin,
        BuiltinEntryMetadata::new("every", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_some_builtin,
        BuiltinEntryMetadata::new("some", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_find_builtin,
        BuiltinEntryMetadata::new("find", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_find_index_builtin,
        BuiltinEntryMetadata::new("findIndex", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_find_last_builtin,
        BuiltinEntryMetadata::new("findLast", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_find_last_index_builtin,
        BuiltinEntryMetadata::new("findLastIndex", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_fill_builtin,
        BuiltinEntryMetadata::new("fill", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_copy_within_builtin,
        BuiltinEntryMetadata::new("copyWithin", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_filter_builtin,
        BuiltinEntryMetadata::new("filter", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_for_each_builtin,
        BuiltinEntryMetadata::new("forEach", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_includes_builtin,
        BuiltinEntryMetadata::new("includes", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_index_of_builtin,
        BuiltinEntryMetadata::new("indexOf", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_join_builtin,
        BuiltinEntryMetadata::new("join", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_last_index_of_builtin,
        BuiltinEntryMetadata::new("lastIndexOf", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_map_builtin,
        BuiltinEntryMetadata::new("map", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_reduce_builtin,
        BuiltinEntryMetadata::new("reduce", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_reduce_right_builtin,
        BuiltinEntryMetadata::new("reduceRight", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_reverse_builtin,
        BuiltinEntryMetadata::new("reverse", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_sort_builtin,
        BuiltinEntryMetadata::new("sort", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_to_locale_string_builtin,
        BuiltinEntryMetadata::new("toLocaleString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_to_string_builtin,
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_to_reversed_builtin,
        BuiltinEntryMetadata::new("toReversed", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_to_sorted_builtin,
        BuiltinEntryMetadata::new("toSorted", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_with_builtin,
        BuiltinEntryMetadata::new("with", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        int8_array_builtin,
        BuiltinEntryMetadata::new("Int8Array", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        int16_array_builtin,
        BuiltinEntryMetadata::new("Int16Array", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        int32_array_builtin,
        BuiltinEntryMetadata::new("Int32Array", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        float32_array_builtin,
        BuiltinEntryMetadata::new("Float32Array", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        float64_array_builtin,
        BuiltinEntryMetadata::new("Float64Array", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        big_int64_array_builtin,
        BuiltinEntryMetadata::new("BigInt64Array", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        big_uint64_array_builtin,
        BuiltinEntryMetadata::new("BigUint64Array", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        uint32_array_builtin,
        BuiltinEntryMetadata::new("Uint32Array", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        uint16_array_builtin,
        BuiltinEntryMetadata::new("Uint16Array", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        uint8_clamped_array_builtin,
        BuiltinEntryMetadata::new("Uint8ClampedArray", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        uint8_array_builtin,
        BuiltinEntryMetadata::new("Uint8Array", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        array_buffer_byte_length_getter_builtin,
        BuiltinEntryMetadata::new("get byteLength", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        array_buffer_slice_builtin,
        BuiltinEntryMetadata::new("slice", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        shared_array_buffer_byte_length_getter_builtin,
        BuiltinEntryMetadata::new("get byteLength", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        shared_array_buffer_slice_builtin,
        BuiltinEntryMetadata::new("slice", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        atomics_load_builtin,
        BuiltinEntryMetadata::new("load", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        atomics_store_builtin,
        BuiltinEntryMetadata::new("store", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        atomics_add_builtin,
        BuiltinEntryMetadata::new("add", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        atomics_sub_builtin,
        BuiltinEntryMetadata::new("sub", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        atomics_and_builtin,
        BuiltinEntryMetadata::new("and", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        atomics_or_builtin,
        BuiltinEntryMetadata::new("or", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        atomics_xor_builtin,
        BuiltinEntryMetadata::new("xor", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        atomics_exchange_builtin,
        BuiltinEntryMetadata::new("exchange", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        atomics_compare_exchange_builtin,
        BuiltinEntryMetadata::new("compareExchange", 4, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        atomics_notify_builtin,
        BuiltinEntryMetadata::new("notify", 3, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        atomics_wait_builtin,
        BuiltinEntryMetadata::new("wait", 4, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        atomics_wait_async_builtin,
        BuiltinEntryMetadata::new("waitAsync", 4, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        atomics_pause_builtin,
        BuiltinEntryMetadata::new("pause", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        atomics_is_lock_free_builtin,
        BuiltinEntryMetadata::new("isLockFree", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_buffer_getter_builtin,
        BuiltinEntryMetadata::new("get buffer", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_byte_length_getter_builtin,
        BuiltinEntryMetadata::new("get byteLength", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_byte_offset_getter_builtin,
        BuiltinEntryMetadata::new("get byteOffset", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_get_float32_builtin,
        BuiltinEntryMetadata::new("getFloat32", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_get_float64_builtin,
        BuiltinEntryMetadata::new("getFloat64", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_get_int16_builtin,
        BuiltinEntryMetadata::new("getInt16", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_get_int32_builtin,
        BuiltinEntryMetadata::new("getInt32", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_get_int8_builtin,
        BuiltinEntryMetadata::new("getInt8", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_get_uint16_builtin,
        BuiltinEntryMetadata::new("getUint16", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_get_uint32_builtin,
        BuiltinEntryMetadata::new("getUint32", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_get_uint8_builtin,
        BuiltinEntryMetadata::new("getUint8", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_set_float32_builtin,
        BuiltinEntryMetadata::new("setFloat32", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_set_float64_builtin,
        BuiltinEntryMetadata::new("setFloat64", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_set_int16_builtin,
        BuiltinEntryMetadata::new("setInt16", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_set_int32_builtin,
        BuiltinEntryMetadata::new("setInt32", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_set_int8_builtin,
        BuiltinEntryMetadata::new("setInt8", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_set_uint16_builtin,
        BuiltinEntryMetadata::new("setUint16", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_set_uint32_builtin,
        BuiltinEntryMetadata::new("setUint32", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_set_uint8_builtin,
        BuiltinEntryMetadata::new("setUint8", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_get_big_int64_builtin,
        BuiltinEntryMetadata::new("getBigInt64", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_get_big_uint64_builtin,
        BuiltinEntryMetadata::new("getBigUint64", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_set_big_int64_builtin,
        BuiltinEntryMetadata::new("setBigInt64", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_set_big_uint64_builtin,
        BuiltinEntryMetadata::new("setBigUint64", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_get_float16_builtin,
        BuiltinEntryMetadata::new("getFloat16", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        data_view_set_float16_builtin,
        BuiltinEntryMetadata::new("setFloat16", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        uint8_array_buffer_getter_builtin,
        BuiltinEntryMetadata::new("get buffer", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        uint8_array_byte_length_getter_builtin,
        BuiltinEntryMetadata::new("get byteLength", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        uint8_array_byte_offset_getter_builtin,
        BuiltinEntryMetadata::new("get byteOffset", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        uint8_array_length_getter_builtin,
        BuiltinEntryMetadata::new("get length", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        uint8_array_values_builtin,
        BuiltinEntryMetadata::new("values", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        uint8_array_keys_builtin,
        BuiltinEntryMetadata::new("keys", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        uint8_array_entries_builtin,
        BuiltinEntryMetadata::new("entries", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        uint8_array_set_builtin,
        BuiltinEntryMetadata::new("set", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        uint8_array_slice_builtin,
        BuiltinEntryMetadata::new("slice", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        uint8_array_subarray_builtin,
        BuiltinEntryMetadata::new("subarray", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        uint8_array_from_base64_builtin,
        BuiltinEntryMetadata::new("fromBase64", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        uint8_array_from_hex_builtin,
        BuiltinEntryMetadata::new("fromHex", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        uint8_array_set_from_base64_builtin,
        BuiltinEntryMetadata::new("setFromBase64", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        uint8_array_set_from_hex_builtin,
        BuiltinEntryMetadata::new("setFromHex", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        uint8_array_to_base64_builtin,
        BuiltinEntryMetadata::new("toBase64", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        uint8_array_to_hex_builtin,
        BuiltinEntryMetadata::new("toHex", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_at_builtin,
        BuiltinEntryMetadata::new("at", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        typed_array_to_string_tag_getter_builtin,
        BuiltinEntryMetadata::new("get [Symbol.toStringTag]", 0, false, false),
    ),
];
