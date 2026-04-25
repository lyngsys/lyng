use super::descriptors::{
    builtin_function_atom_property, data_symbol_property, descriptor_tag_with_atom,
    readonly_builtin_attributes,
};
use super::{
    install_public_builtin_function, FamilyInstallContext, JsonFamilyBuiltins, JsonFamilyObjects,
};
use crate::bootstrap::{install_descriptor_tables, BuiltinBootstrapError};
use crate::public::{BuiltinCache, PublicRealmBuiltins};
use crate::{BuiltinDescriptorTable, BuiltinInstallTarget, BuiltinIntrinsic};
use lyng_js_common::AtomId;
use lyng_js_env::Agent;
use lyng_js_types::{
    json_is_raw_json_builtin, json_parse_builtin, json_raw_json_builtin, json_stringify_builtin,
    BuiltinFunctionId, ObjectRef, RealmRef, WellKnownSymbolId,
};

pub(in crate::public) fn install_json_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    objects: JsonFamilyObjects,
) -> JsonFamilyBuiltins {
    JsonFamilyBuiltins {
        json: objects.json,
        json_parse: install_public_builtin_function(agent, cx, json_parse_builtin(), None),
        json_stringify: install_public_builtin_function(agent, cx, json_stringify_builtin(), None),
        json_raw_json: install_public_builtin_function(agent, cx, json_raw_json_builtin(), None),
        json_is_raw_json: install_public_builtin_function(
            agent,
            cx,
            json_is_raw_json_builtin(),
            None,
        ),
    }
}

pub(in crate::public) fn json_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (json_parse_builtin(), builtins.json_parse),
        (json_stringify_builtin(), builtins.json_stringify),
        (json_raw_json_builtin(), builtins.json_raw_json),
        (json_is_raw_json_builtin(), builtins.json_is_raw_json),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

pub(in crate::public) fn install_json_family_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
) -> Result<(), BuiltinBootstrapError> {
    let atoms = JsonDescriptorAtoms::new(agent);
    let json_atom = agent.bootstrap_atoms().json();
    let json_tag = descriptor_tag_with_atom(agent, "JSON", json_atom);
    let descriptors = [
        builtin_function_atom_property(atoms.parse, json_parse_builtin()),
        builtin_function_atom_property(atoms.stringify, json_stringify_builtin()),
        builtin_function_atom_property(atoms.raw_json, json_raw_json_builtin()),
        builtin_function_atom_property(atoms.is_raw_json, json_is_raw_json_builtin()),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            json_tag,
            readonly_builtin_attributes(),
        ),
    ];
    let tables = [BuiltinDescriptorTable::new(
        BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Json),
        &descriptors,
    )];
    install_descriptor_tables(agent, cache, realm, &tables)
}

#[derive(Clone, Copy)]
struct JsonDescriptorAtoms {
    parse: AtomId,
    stringify: AtomId,
    raw_json: AtomId,
    is_raw_json: AtomId,
}

impl JsonDescriptorAtoms {
    fn new(agent: &mut Agent) -> Self {
        Self {
            parse: agent.atoms_mut().intern_collectible("parse"),
            stringify: agent.atoms_mut().intern_collectible("stringify"),
            raw_json: agent.atoms_mut().intern_collectible("rawJSON"),
            is_raw_json: agent.atoms_mut().intern_collectible("isRawJSON"),
        }
    }
}
