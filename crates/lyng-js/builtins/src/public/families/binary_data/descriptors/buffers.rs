use crate::public::PublicRealmBuiltins;
use crate::{
    BuiltinAttributes, BuiltinPropertyDescriptor, BuiltinPropertyKeySpec, BuiltinPropertyValueSpec,
};
use lyng_js_common::{AtomId, WellKnownAtom};
use lyng_js_types::{
    array_buffer_byte_length_getter_builtin, array_buffer_detached_getter_builtin,
    array_buffer_is_view_builtin, array_buffer_max_byte_length_getter_builtin,
    array_buffer_resizable_getter_builtin, array_buffer_resize_builtin, array_buffer_slice_builtin,
    array_buffer_transfer_builtin, array_buffer_transfer_to_fixed_length_builtin,
    array_species_getter_builtin, shared_array_buffer_byte_length_getter_builtin,
    shared_array_buffer_grow_builtin, shared_array_buffer_growable_getter_builtin,
    shared_array_buffer_max_byte_length_getter_builtin, shared_array_buffer_slice_builtin, Value,
    WellKnownSymbolId,
};

pub(super) struct BufferDescriptorAtoms {
    pub(super) is_view: AtomId,
    pub(super) byte_length: AtomId,
    pub(super) detached: AtomId,
    pub(super) max_byte_length: AtomId,
    pub(super) resizable: AtomId,
    pub(super) resize: AtomId,
    pub(super) slice: AtomId,
    pub(super) transfer: AtomId,
    pub(super) transfer_to_fixed_length: AtomId,
    pub(super) grow: AtomId,
    pub(super) growable: AtomId,
}

pub(super) struct BufferDescriptorTags {
    pub(super) array_buffer: Value,
    pub(super) shared_array_buffer: Value,
}

pub(super) struct BufferDescriptorSets {
    pub(super) array_buffer: [BuiltinPropertyDescriptor; 2],
    pub(super) array_buffer_prototype: [BuiltinPropertyDescriptor; 10],
    pub(super) shared_array_buffer: [BuiltinPropertyDescriptor; 1],
    pub(super) shared_array_buffer_prototype: [BuiltinPropertyDescriptor; 7],
}

pub(super) const fn descriptor_sets(
    builtins: &PublicRealmBuiltins,
    atoms: BufferDescriptorAtoms,
    tags: BufferDescriptorTags,
) -> BufferDescriptorSets {
    BufferDescriptorSets {
        array_buffer: [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.is_view),
                BuiltinPropertyValueSpec::BuiltinFunction(array_buffer_is_view_builtin()),
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
        array_buffer_prototype: [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(builtins.array_buffer)),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.byte_length),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(array_buffer_byte_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.detached),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(array_buffer_detached_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.max_byte_length),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(array_buffer_max_byte_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.resizable),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(array_buffer_resizable_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.resize),
                BuiltinPropertyValueSpec::BuiltinFunction(array_buffer_resize_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.slice),
                BuiltinPropertyValueSpec::BuiltinFunction(array_buffer_slice_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.transfer),
                BuiltinPropertyValueSpec::BuiltinFunction(array_buffer_transfer_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.transfer_to_fixed_length),
                BuiltinPropertyValueSpec::BuiltinFunction(
                    array_buffer_transfer_to_fixed_length_builtin(),
                ),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(tags.array_buffer),
                BuiltinAttributes::new(false, false, true),
            ),
        ],
        shared_array_buffer: [BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::Species),
            BuiltinPropertyValueSpec::Accessor {
                get: Some(array_species_getter_builtin()),
                set: None,
            },
            BuiltinAttributes::new(false, false, true),
        )],
        shared_array_buffer_prototype: [
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(WellKnownAtom::constructor.id()),
                BuiltinPropertyValueSpec::Data(Value::from_object_ref(
                    builtins.shared_array_buffer,
                )),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.byte_length),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(shared_array_buffer_byte_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.grow),
                BuiltinPropertyValueSpec::BuiltinFunction(shared_array_buffer_grow_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.growable),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(shared_array_buffer_growable_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.max_byte_length),
                BuiltinPropertyValueSpec::Accessor {
                    get: Some(shared_array_buffer_max_byte_length_getter_builtin()),
                    set: None,
                },
                BuiltinAttributes::new(false, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_atom(atoms.slice),
                BuiltinPropertyValueSpec::BuiltinFunction(shared_array_buffer_slice_builtin()),
                BuiltinAttributes::new(true, false, true),
            ),
            BuiltinPropertyDescriptor::new(
                BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
                BuiltinPropertyValueSpec::Data(tags.shared_array_buffer),
                BuiltinAttributes::new(false, false, true),
            ),
        ],
    }
}
