use crate::public::PublicRealmBuiltins;
use crate::{
    BuiltinAttributes, BuiltinPropertyDescriptor, BuiltinPropertyKeySpec, BuiltinPropertyValueSpec,
};
use lyng_js_common::{AtomId, WellKnownAtom};
use lyng_js_types::{
    array_species_getter_builtin, array_to_string_builtin, typed_array_at_builtin,
    typed_array_copy_within_builtin, typed_array_every_builtin, typed_array_fill_builtin,
    typed_array_filter_builtin, typed_array_find_builtin, typed_array_find_index_builtin,
    typed_array_find_last_builtin, typed_array_find_last_index_builtin,
    typed_array_for_each_builtin, typed_array_from_builtin, typed_array_includes_builtin,
    typed_array_index_of_builtin, typed_array_join_builtin, typed_array_last_index_of_builtin,
    typed_array_map_builtin, typed_array_of_builtin, typed_array_reduce_builtin,
    typed_array_reduce_right_builtin, typed_array_reverse_builtin, typed_array_some_builtin,
    typed_array_sort_builtin, typed_array_to_locale_string_builtin,
    typed_array_to_reversed_builtin, typed_array_to_sorted_builtin,
    typed_array_to_string_tag_getter_builtin, typed_array_with_builtin,
    uint8_array_buffer_getter_builtin, uint8_array_byte_length_getter_builtin,
    uint8_array_byte_offset_getter_builtin, uint8_array_entries_builtin,
    uint8_array_from_base64_builtin, uint8_array_from_hex_builtin, uint8_array_keys_builtin,
    uint8_array_length_getter_builtin, uint8_array_set_builtin,
    uint8_array_set_from_base64_builtin, uint8_array_set_from_hex_builtin,
    uint8_array_slice_builtin, uint8_array_subarray_builtin, uint8_array_to_base64_builtin,
    uint8_array_to_hex_builtin, uint8_array_values_builtin, ObjectRef, Value, WellKnownSymbolId,
};

pub(super) struct TypedArrayDescriptorAtoms {
    pub(super) last_index_of: AtomId,
    pub(super) copy_within: AtomId,
    pub(super) entries: AtomId,
    pub(super) every: AtomId,
    pub(super) fill: AtomId,
    pub(super) filter: AtomId,
    pub(super) find: AtomId,
    pub(super) find_index: AtomId,
    pub(super) find_last: AtomId,
    pub(super) find_last_index: AtomId,
    pub(super) from: AtomId,
    pub(super) for_each: AtomId,
    pub(super) includes: AtomId,
    pub(super) index_of: AtomId,
    pub(super) join: AtomId,
    pub(super) keys: AtomId,
    pub(super) map: AtomId,
    pub(super) of: AtomId,
    pub(super) reduce: AtomId,
    pub(super) reduce_right: AtomId,
    pub(super) reverse: AtomId,
    pub(super) some: AtomId,
    pub(super) at: AtomId,
    pub(super) slice: AtomId,
    pub(super) buffer: AtomId,
    pub(super) byte_length: AtomId,
    pub(super) byte_offset: AtomId,
    pub(super) bytes_per_element: AtomId,
    pub(super) sort: AtomId,
    pub(super) to_locale_string: AtomId,
    pub(super) to_reversed: AtomId,
    pub(super) to_sorted: AtomId,
    pub(super) values: AtomId,
    pub(super) with: AtomId,
    pub(super) set: AtomId,
    pub(super) subarray: AtomId,
    pub(super) from_base64: AtomId,
    pub(super) from_hex: AtomId,
    pub(super) set_from_base64: AtomId,
    pub(super) set_from_hex: AtomId,
    pub(super) to_base64: AtomId,
    pub(super) to_hex: AtomId,
}

pub(super) struct TypedArrayDescriptorSets {
    pub(super) typed_array_descriptors: [BuiltinPropertyDescriptor; 3],
    pub(super) typed_array_prototype_descriptors: [BuiltinPropertyDescriptor; 38],
    pub(super) int8_array_descriptors: [BuiltinPropertyDescriptor; 1],
    pub(super) int8_array_prototype_descriptors: [BuiltinPropertyDescriptor; 2],
    pub(super) int16_array_descriptors: [BuiltinPropertyDescriptor; 1],
    pub(super) int16_array_prototype_descriptors: [BuiltinPropertyDescriptor; 2],
    pub(super) int32_array_descriptors: [BuiltinPropertyDescriptor; 1],
    pub(super) int32_array_prototype_descriptors: [BuiltinPropertyDescriptor; 2],
    pub(super) float32_array_descriptors: [BuiltinPropertyDescriptor; 1],
    pub(super) float32_array_prototype_descriptors: [BuiltinPropertyDescriptor; 2],
    pub(super) float64_array_descriptors: [BuiltinPropertyDescriptor; 1],
    pub(super) float64_array_prototype_descriptors: [BuiltinPropertyDescriptor; 2],
    pub(super) big_int64_array_descriptors: [BuiltinPropertyDescriptor; 1],
    pub(super) big_int64_array_prototype_descriptors: [BuiltinPropertyDescriptor; 2],
    pub(super) big_uint64_array_descriptors: [BuiltinPropertyDescriptor; 1],
    pub(super) big_uint64_array_prototype_descriptors: [BuiltinPropertyDescriptor; 2],
    pub(super) uint32_array_descriptors: [BuiltinPropertyDescriptor; 1],
    pub(super) uint32_array_prototype_descriptors: [BuiltinPropertyDescriptor; 2],
    pub(super) uint16_array_descriptors: [BuiltinPropertyDescriptor; 1],
    pub(super) uint16_array_prototype_descriptors: [BuiltinPropertyDescriptor; 2],
    pub(super) uint8_clamped_array_descriptors: [BuiltinPropertyDescriptor; 1],
    pub(super) uint8_clamped_array_prototype_descriptors: [BuiltinPropertyDescriptor; 2],
    pub(super) uint8_array_descriptors: [BuiltinPropertyDescriptor; 3],
    pub(super) uint8_array_prototype_descriptors: [BuiltinPropertyDescriptor; 6],
}

fn concrete_typed_array_prototype_descriptors(
    constructor: ObjectRef,
    bytes_per_element_atom: AtomId,
    bytes_per_element: i32,
) -> [BuiltinPropertyDescriptor; 2] {
    [
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
            BuiltinPropertyValueSpec::Data(Value::from_object_ref(constructor)),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(bytes_per_element)),
            BuiltinAttributes::new(false, false, false),
        ),
    ]
}

fn uint8_array_prototype_descriptors(
    constructor: ObjectRef,
    bytes_per_element_atom: AtomId,
    set_from_base64_atom: AtomId,
    set_from_hex_atom: AtomId,
    to_base64_atom: AtomId,
    to_hex_atom: AtomId,
) -> [BuiltinPropertyDescriptor; 6] {
    [
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
            BuiltinPropertyValueSpec::Data(Value::from_object_ref(constructor)),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(1)),
            BuiltinAttributes::new(false, false, false),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(set_from_base64_atom),
            BuiltinPropertyValueSpec::BuiltinFunction(uint8_array_set_from_base64_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(set_from_hex_atom),
            BuiltinPropertyValueSpec::BuiltinFunction(uint8_array_set_from_hex_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(to_base64_atom),
            BuiltinPropertyValueSpec::BuiltinFunction(uint8_array_to_base64_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(to_hex_atom),
            BuiltinPropertyValueSpec::BuiltinFunction(uint8_array_to_hex_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
    ]
}

#[allow(clippy::too_many_lines)]
pub(super) fn descriptor_sets(
    builtins: &PublicRealmBuiltins,
    atoms: TypedArrayDescriptorAtoms,
) -> TypedArrayDescriptorSets {
    let TypedArrayDescriptorAtoms {
        last_index_of: last_index_of_atom,
        copy_within: copy_within_atom,
        entries: entries_atom,
        every: every_atom,
        fill: fill_atom,
        filter: filter_atom,
        find: find_atom,
        find_index: find_index_atom,
        find_last: find_last_atom,
        find_last_index: find_last_index_atom,
        from: from_atom,
        for_each: for_each_atom,
        includes: includes_atom,
        index_of: index_of_atom,
        join: join_atom,
        keys: keys_atom,
        map: map_atom,
        of: of_atom,
        reduce: reduce_atom,
        reduce_right: reduce_right_atom,
        reverse: reverse_atom,
        some: some_atom,
        at: at_atom,
        slice: slice_atom,
        buffer: buffer_atom,
        byte_length: byte_length_atom,
        byte_offset: byte_offset_atom,
        bytes_per_element: bytes_per_element_atom,
        sort: sort_atom,
        to_locale_string: to_locale_string_atom,
        to_reversed: to_reversed_atom,
        to_sorted: to_sorted_atom,
        values: values_atom,
        with: with_atom,
        set: set_atom,
        subarray: subarray_atom,
        from_base64: from_base64_atom,
        from_hex: from_hex_atom,
        set_from_base64: set_from_base64_atom,
        set_from_hex: set_from_hex_atom,
        to_base64: to_base64_atom,
        to_hex: to_hex_atom,
    } = atoms;
    TypedArrayDescriptorSets {
        typed_array_descriptors: [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(from_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_from_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(of_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_of_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Species),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(array_species_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
        ],
        typed_array_prototype_descriptors: [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.typed_array)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(buffer_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(uint8_array_buffer_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_length_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(uint8_array_byte_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(byte_offset_atom),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(uint8_array_byte_offset_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::length.id()),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(uint8_array_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(copy_within_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_copy_within_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(every_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_every_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(fill_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_fill_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(filter_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_filter_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(includes_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_includes_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(index_of_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_index_of_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(for_each_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_for_each_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(join_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_join_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(map_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_map_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(some_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_some_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(find_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_find_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(find_index_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_find_index_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(find_last_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_find_last_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(find_last_index_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_find_last_index_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(values_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(keys_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(uint8_array_keys_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(entries_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(uint8_array_entries_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(set_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(uint8_array_set_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(slice_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(uint8_array_slice_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(last_index_of_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_last_index_of_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(reduce_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_reduce_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(reduce_right_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_reduce_right_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(subarray_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(uint8_array_subarray_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(reverse_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_reverse_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(sort_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_sort_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_reversed_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_to_reversed_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_sorted_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_to_sorted_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(with_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_with_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(at_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_at_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::toString.id()),
                BuiltinPropertyValueSpec::BuiltinFunction(array_to_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(to_locale_string_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(typed_array_to_locale_string_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Iterator),
                BuiltinPropertyValueSpec::BuiltinFunction(uint8_array_values_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(typed_array_to_string_tag_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
        ],
        int8_array_descriptors: [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(1)),
            BuiltinAttributes::new(false, false, false),
        )],
        int8_array_prototype_descriptors: concrete_typed_array_prototype_descriptors(
            builtins.int8_array,
            bytes_per_element_atom,
            1,
        ),
        int16_array_descriptors: [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(2)),
            BuiltinAttributes::new(false, false, false),
        )],
        int16_array_prototype_descriptors: concrete_typed_array_prototype_descriptors(
            builtins.int16_array,
            bytes_per_element_atom,
            2,
        ),
        int32_array_descriptors: [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(4)),
            BuiltinAttributes::new(false, false, false),
        )],
        int32_array_prototype_descriptors: concrete_typed_array_prototype_descriptors(
            builtins.int32_array,
            bytes_per_element_atom,
            4,
        ),
        float32_array_descriptors: [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(4)),
            BuiltinAttributes::new(false, false, false),
        )],
        float32_array_prototype_descriptors: concrete_typed_array_prototype_descriptors(
            builtins.float32_array,
            bytes_per_element_atom,
            4,
        ),
        float64_array_descriptors: [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(8)),
            BuiltinAttributes::new(false, false, false),
        )],
        float64_array_prototype_descriptors: concrete_typed_array_prototype_descriptors(
            builtins.float64_array,
            bytes_per_element_atom,
            8,
        ),
        big_int64_array_descriptors: [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(8)),
            BuiltinAttributes::new(false, false, false),
        )],
        big_int64_array_prototype_descriptors: concrete_typed_array_prototype_descriptors(
            builtins.big_int64_array,
            bytes_per_element_atom,
            8,
        ),
        big_uint64_array_descriptors: [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(8)),
            BuiltinAttributes::new(false, false, false),
        )],
        big_uint64_array_prototype_descriptors: concrete_typed_array_prototype_descriptors(
            builtins.big_uint64_array,
            bytes_per_element_atom,
            8,
        ),
        uint32_array_descriptors: [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(4)),
            BuiltinAttributes::new(false, false, false),
        )],
        uint32_array_prototype_descriptors: concrete_typed_array_prototype_descriptors(
            builtins.uint32_array,
            bytes_per_element_atom,
            4,
        ),
        uint16_array_descriptors: [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(2)),
            BuiltinAttributes::new(false, false, false),
        )],
        uint16_array_prototype_descriptors: concrete_typed_array_prototype_descriptors(
            builtins.uint16_array,
            bytes_per_element_atom,
            2,
        ),
        uint8_clamped_array_descriptors: [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
            BuiltinPropertyValueSpec::Data(Value::from_smi(1)),
            BuiltinAttributes::new(false, false, false),
        )],
        uint8_clamped_array_prototype_descriptors: concrete_typed_array_prototype_descriptors(
            builtins.uint8_clamped_array,
            bytes_per_element_atom,
            1,
        ),
        uint8_array_descriptors: [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(bytes_per_element_atom),
                BuiltinPropertyValueSpec::Data(Value::from_smi(1)),
                BuiltinAttributes::new(false, false, false),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(from_base64_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(uint8_array_from_base64_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(from_hex_atom),
                BuiltinPropertyValueSpec::BuiltinFunction(uint8_array_from_hex_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
        ],
        uint8_array_prototype_descriptors: uint8_array_prototype_descriptors(
            builtins.uint8_array,
            bytes_per_element_atom,
            set_from_base64_atom,
            set_from_hex_atom,
            to_base64_atom,
            to_hex_atom,
        ),
    }
}
