use crate::public::PublicRealmBuiltins;
use crate::{
    BuiltinAttributes, BuiltinPropertyDescriptor, BuiltinPropertyKeySpec, BuiltinPropertyValueSpec,
};
use lyng_js_common::{AtomId, WellKnownAtom};
use lyng_js_types::{
    data_view_buffer_getter_builtin, data_view_byte_length_getter_builtin,
    data_view_byte_offset_getter_builtin, data_view_get_big_int64_builtin,
    data_view_get_big_uint64_builtin, data_view_get_float16_builtin, data_view_get_float32_builtin,
    data_view_get_float64_builtin, data_view_get_int16_builtin, data_view_get_int32_builtin,
    data_view_get_int8_builtin, data_view_get_uint16_builtin, data_view_get_uint32_builtin,
    data_view_get_uint8_builtin, data_view_set_big_int64_builtin, data_view_set_big_uint64_builtin,
    data_view_set_float16_builtin, data_view_set_float32_builtin, data_view_set_float64_builtin,
    data_view_set_int16_builtin, data_view_set_int32_builtin, data_view_set_int8_builtin,
    data_view_set_uint16_builtin, data_view_set_uint32_builtin, data_view_set_uint8_builtin, Value,
    WellKnownSymbolId,
};

pub(super) struct DataViewDescriptorAtoms {
    pub(super) buffer: AtomId,
    pub(super) byte_length: AtomId,
    pub(super) byte_offset: AtomId,
    pub(super) get_big_int64: AtomId,
    pub(super) get_big_uint64: AtomId,
    pub(super) get_float16: AtomId,
    pub(super) get_float32: AtomId,
    pub(super) get_float64: AtomId,
    pub(super) get_int16: AtomId,
    pub(super) get_int32: AtomId,
    pub(super) get_int8: AtomId,
    pub(super) get_uint16: AtomId,
    pub(super) get_uint32: AtomId,
    pub(super) get_uint8: AtomId,
    pub(super) set_big_int64: AtomId,
    pub(super) set_big_uint64: AtomId,
    pub(super) set_float16: AtomId,
    pub(super) set_float32: AtomId,
    pub(super) set_float64: AtomId,
    pub(super) set_int16: AtomId,
    pub(super) set_int32: AtomId,
    pub(super) set_int8: AtomId,
    pub(super) set_uint16: AtomId,
    pub(super) set_uint32: AtomId,
    pub(super) set_uint8: AtomId,
}

pub(super) struct DataViewDescriptorSets {
    pub(super) data_view: [BuiltinPropertyDescriptor; 0],
    pub(super) data_view_prototype: [BuiltinPropertyDescriptor; 27],
}

pub(super) fn descriptor_sets(
    builtins: &PublicRealmBuiltins,
    atoms: DataViewDescriptorAtoms,
    data_view_tag: Value,
) -> DataViewDescriptorSets {
    DataViewDescriptorSets {
        data_view: [],
        data_view_prototype: [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.data_view)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.buffer),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(data_view_buffer_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.byte_length),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(data_view_byte_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.byte_offset),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(data_view_byte_offset_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.get_float32),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_get_float32_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.get_float64),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_get_float64_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.get_int16),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_get_int16_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.get_int32),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_get_int32_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.get_int8),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_get_int8_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.get_uint16),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_get_uint16_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.get_uint32),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_get_uint32_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.get_uint8),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_get_uint8_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.set_float32),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_set_float32_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.set_float64),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_set_float64_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.set_int16),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_set_int16_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.set_int32),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_set_int32_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.set_int8),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_set_int8_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.set_uint16),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_set_uint16_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.set_uint32),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_set_uint32_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.set_uint8),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_set_uint8_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.get_big_int64),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_get_big_int64_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.get_big_uint64),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_get_big_uint64_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.set_big_int64),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_set_big_int64_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.set_big_uint64),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_set_big_uint64_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.get_float16),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_get_float16_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.set_float16),
                BuiltinPropertyValueSpec::BuiltinFunction(data_view_set_float16_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(data_view_tag),
                BuiltinAttributes::new(false, false, true),
            ),
        ],
    }
}
