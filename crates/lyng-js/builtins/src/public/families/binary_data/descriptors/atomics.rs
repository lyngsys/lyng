use crate::{
    BuiltinAttributes, BuiltinPropertyDescriptor, BuiltinPropertyKeySpec, BuiltinPropertyValueSpec,
};
use lyng_js_common::AtomId;
use lyng_js_types::{
    atomics_add_builtin, atomics_and_builtin, atomics_compare_exchange_builtin,
    atomics_exchange_builtin, atomics_is_lock_free_builtin, atomics_load_builtin,
    atomics_notify_builtin, atomics_or_builtin, atomics_pause_builtin, atomics_store_builtin,
    atomics_sub_builtin, atomics_wait_async_builtin, atomics_wait_builtin, atomics_xor_builtin,
    Value, WellKnownSymbolId,
};

pub(super) struct AtomicsDescriptorAtoms {
    pub(super) add: AtomId,
    pub(super) and: AtomId,
    pub(super) compare_exchange: AtomId,
    pub(super) exchange: AtomId,
    pub(super) is_lock_free: AtomId,
    pub(super) load: AtomId,
    pub(super) notify: AtomId,
    pub(super) or: AtomId,
    pub(super) pause: AtomId,
    pub(super) store: AtomId,
    pub(super) sub: AtomId,
    pub(super) wait: AtomId,
    pub(super) wait_async: AtomId,
    pub(super) xor: AtomId,
}

pub(super) const fn descriptors(
    atoms: AtomicsDescriptorAtoms,
    atomics_tag: Value,
) -> [BuiltinPropertyDescriptor; 15] {
    [
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.add),
            BuiltinPropertyValueSpec::BuiltinFunction(atomics_add_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.and),
            BuiltinPropertyValueSpec::BuiltinFunction(atomics_and_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.compare_exchange),
            BuiltinPropertyValueSpec::BuiltinFunction(atomics_compare_exchange_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.exchange),
            BuiltinPropertyValueSpec::BuiltinFunction(atomics_exchange_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.is_lock_free),
            BuiltinPropertyValueSpec::BuiltinFunction(atomics_is_lock_free_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.load),
            BuiltinPropertyValueSpec::BuiltinFunction(atomics_load_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.notify),
            BuiltinPropertyValueSpec::BuiltinFunction(atomics_notify_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.or),
            BuiltinPropertyValueSpec::BuiltinFunction(atomics_or_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.pause),
            BuiltinPropertyValueSpec::BuiltinFunction(atomics_pause_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.store),
            BuiltinPropertyValueSpec::BuiltinFunction(atomics_store_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.sub),
            BuiltinPropertyValueSpec::BuiltinFunction(atomics_sub_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.wait),
            BuiltinPropertyValueSpec::BuiltinFunction(atomics_wait_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.wait_async),
            BuiltinPropertyValueSpec::BuiltinFunction(atomics_wait_async_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_atom(atoms.xor),
            BuiltinPropertyValueSpec::BuiltinFunction(atomics_xor_builtin()),
            BuiltinAttributes::new(true, false, true),
        ),
        BuiltinPropertyDescriptor::new(
            BuiltinPropertyKeySpec::from_well_known_symbol(WellKnownSymbolId::ToStringTag),
            BuiltinPropertyValueSpec::Data(atomics_tag),
            BuiltinAttributes::new(false, false, true),
        ),
    ]
}
