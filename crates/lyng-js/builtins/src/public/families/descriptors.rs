use crate::{
    BuiltinAttributes, BuiltinPropertyDescriptor, BuiltinPropertyKeySpec, BuiltinPropertyValueSpec,
};
use lyng_js_common::AtomId;
use lyng_js_env::Agent;
use lyng_js_gc::AllocationLifetime;
use lyng_js_types::{BuiltinFunctionId, Value, WellKnownSymbolId};

pub(super) fn builtin_function_atom_property(
    atom: AtomId,
    entry: BuiltinFunctionId,
) -> BuiltinPropertyDescriptor {
    BuiltinPropertyDescriptor::new(
        BuiltinPropertyKeySpec::from_atom(atom),
        BuiltinPropertyValueSpec::BuiltinFunction(entry),
        writable_builtin_attributes(),
    )
}

pub(super) fn builtin_function_symbol_property(
    symbol: WellKnownSymbolId,
    entry: BuiltinFunctionId,
    attributes: BuiltinAttributes,
) -> BuiltinPropertyDescriptor {
    BuiltinPropertyDescriptor::new(
        BuiltinPropertyKeySpec::from_well_known_symbol(symbol),
        BuiltinPropertyValueSpec::BuiltinFunction(entry),
        attributes,
    )
}

pub(super) fn data_atom_property(
    atom: AtomId,
    value: Value,
    attributes: BuiltinAttributes,
) -> BuiltinPropertyDescriptor {
    BuiltinPropertyDescriptor::new(
        BuiltinPropertyKeySpec::from_atom(atom),
        BuiltinPropertyValueSpec::Data(value),
        attributes,
    )
}

pub(super) fn data_symbol_property(
    symbol: WellKnownSymbolId,
    value: Value,
    attributes: BuiltinAttributes,
) -> BuiltinPropertyDescriptor {
    BuiltinPropertyDescriptor::new(
        BuiltinPropertyKeySpec::from_well_known_symbol(symbol),
        BuiltinPropertyValueSpec::Data(value),
        attributes,
    )
}

pub(super) fn accessor_atom_property(
    atom: AtomId,
    get: Option<BuiltinFunctionId>,
    set: Option<BuiltinFunctionId>,
    attributes: BuiltinAttributes,
) -> BuiltinPropertyDescriptor {
    BuiltinPropertyDescriptor::new(
        BuiltinPropertyKeySpec::from_atom(atom),
        BuiltinPropertyValueSpec::Accessor { get, set },
        attributes,
    )
}

pub(super) fn accessor_symbol_property(
    symbol: WellKnownSymbolId,
    get: Option<BuiltinFunctionId>,
    set: Option<BuiltinFunctionId>,
    attributes: BuiltinAttributes,
) -> BuiltinPropertyDescriptor {
    BuiltinPropertyDescriptor::new(
        BuiltinPropertyKeySpec::from_well_known_symbol(symbol),
        BuiltinPropertyValueSpec::Accessor { get, set },
        attributes,
    )
}

pub(super) fn descriptor_tag(agent: &mut Agent, text: &str) -> Value {
    Value::from_string_ref(agent.alloc_runtime_string(text, None, AllocationLifetime::Default))
}

pub(super) const fn writable_builtin_attributes() -> BuiltinAttributes {
    BuiltinAttributes::new(true, false, true)
}

pub(super) const fn readonly_builtin_attributes() -> BuiltinAttributes {
    BuiltinAttributes::new(false, false, true)
}

pub(super) const fn hidden_builtin_attributes() -> BuiltinAttributes {
    BuiltinAttributes::new(false, false, false)
}
