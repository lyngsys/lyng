mod collections;
mod functions;
mod objects;

use crate::public::{allocate_builtin_function_object, public_builtin_metadata};
use lyng_js_env::Agent;
use lyng_js_types::{BuiltinFunctionId, EnvironmentRef, ObjectRef, RealmRef, ShapeId};

pub(super) use collections::install_collection_family;
pub(super) use functions::install_function_family;
pub(super) use objects::install_object_family;

#[derive(Clone, Copy, Debug)]
pub(super) struct FamilyInstallContext {
    pub(super) realm: RealmRef,
    pub(super) global_env: EnvironmentRef,
    pub(super) root_shape: ShapeId,
    pub(super) function_prototype: ObjectRef,
    pub(super) object_prototype: ObjectRef,
}

pub(super) fn install_public_builtin_function(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    entry: BuiltinFunctionId,
    prototype_object: Option<ObjectRef>,
) -> ObjectRef {
    let metadata =
        public_builtin_metadata(entry).expect("family installer entry must have public metadata");
    allocate_builtin_function_object(
        agent,
        cx.realm,
        cx.global_env,
        cx.root_shape,
        cx.function_prototype,
        cx.object_prototype,
        entry,
        metadata,
        prototype_object,
    )
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ObjectFamilyBuiltins {
    pub(super) object: ObjectRef,
    pub(super) object_prototype: ObjectRef,
    pub(super) object_create: ObjectRef,
    pub(super) object_get_prototype_of: ObjectRef,
    pub(super) object_set_prototype_of: ObjectRef,
    pub(super) object_get_own_property_descriptor: ObjectRef,
    pub(super) object_get_own_property_descriptors: ObjectRef,
    pub(super) object_get_own_property_names: ObjectRef,
    pub(super) object_get_own_property_symbols: ObjectRef,
    pub(super) object_define_properties: ObjectRef,
    pub(super) object_define_property: ObjectRef,
    pub(super) object_assign: ObjectRef,
    pub(super) object_from_entries: ObjectRef,
    pub(super) object_group_by: ObjectRef,
    pub(super) object_prevent_extensions: ObjectRef,
    pub(super) object_is_extensible: ObjectRef,
    pub(super) object_is: ObjectRef,
    pub(super) object_seal: ObjectRef,
    pub(super) object_freeze: ObjectRef,
    pub(super) object_is_sealed: ObjectRef,
    pub(super) object_is_frozen: ObjectRef,
    pub(super) object_to_locale_string: ObjectRef,
    pub(super) object_to_string: ObjectRef,
    pub(super) object_value_of: ObjectRef,
    pub(super) object_has_own_property: ObjectRef,
    pub(super) object_is_prototype_of: ObjectRef,
    pub(super) object_property_is_enumerable: ObjectRef,
    pub(super) object_define_getter: ObjectRef,
    pub(super) object_define_setter: ObjectRef,
    pub(super) object_lookup_getter: ObjectRef,
    pub(super) object_lookup_setter: ObjectRef,
    pub(super) object_proto_getter: ObjectRef,
    pub(super) object_proto_setter: ObjectRef,
    pub(super) object_keys: ObjectRef,
    pub(super) object_entries: ObjectRef,
    pub(super) object_values: ObjectRef,
    pub(super) object_has_own: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
#[allow(clippy::struct_field_names)]
pub(super) struct FunctionFamilyPrototypes {
    pub(super) async_function_prototype: ObjectRef,
    pub(super) async_generator_function_prototype: ObjectRef,
    pub(super) async_generator_prototype: ObjectRef,
    pub(super) generator_function_prototype: ObjectRef,
    pub(super) generator_prototype: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct FunctionFamilyBuiltins {
    pub(super) function: ObjectRef,
    pub(super) function_prototype: ObjectRef,
    pub(super) function_call: ObjectRef,
    pub(super) function_apply: ObjectRef,
    pub(super) function_bind: ObjectRef,
    pub(super) function_to_string: ObjectRef,
    pub(super) function_symbol_has_instance: ObjectRef,
    pub(super) async_function: ObjectRef,
    pub(super) async_function_prototype: ObjectRef,
    pub(super) async_generator_function: ObjectRef,
    pub(super) async_generator_function_prototype: ObjectRef,
    pub(super) async_generator_prototype: ObjectRef,
    pub(super) async_generator_next: ObjectRef,
    pub(super) async_generator_return: ObjectRef,
    pub(super) async_generator_throw: ObjectRef,
    pub(super) generator_function: ObjectRef,
    pub(super) generator_function_prototype: ObjectRef,
    pub(super) generator_prototype: ObjectRef,
    pub(super) generator_next: ObjectRef,
    pub(super) generator_return: ObjectRef,
    pub(super) generator_throw: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
#[allow(clippy::struct_field_names)]
pub(super) struct CollectionFamilyPrototypes {
    pub(super) map_prototype: ObjectRef,
    pub(super) set_prototype: ObjectRef,
    pub(super) weak_map_prototype: ObjectRef,
    pub(super) weak_set_prototype: ObjectRef,
    pub(super) weak_ref_prototype: ObjectRef,
    pub(super) finalization_registry_prototype: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CollectionFamilyBuiltins {
    pub(super) map: ObjectRef,
    pub(super) set: ObjectRef,
    pub(super) weak_map: ObjectRef,
    pub(super) weak_set: ObjectRef,
    pub(super) weak_ref: ObjectRef,
    pub(super) finalization_registry: ObjectRef,
    pub(super) map_get: ObjectRef,
    pub(super) map_set: ObjectRef,
    pub(super) map_has: ObjectRef,
    pub(super) map_delete: ObjectRef,
    pub(super) map_clear: ObjectRef,
    pub(super) map_entries: ObjectRef,
    pub(super) map_values: ObjectRef,
    pub(super) map_keys: ObjectRef,
    pub(super) map_for_each: ObjectRef,
    pub(super) map_size_getter: ObjectRef,
    pub(super) set_add: ObjectRef,
    pub(super) set_has: ObjectRef,
    pub(super) set_delete: ObjectRef,
    pub(super) set_clear: ObjectRef,
    pub(super) set_entries: ObjectRef,
    pub(super) set_values: ObjectRef,
    pub(super) set_keys: ObjectRef,
    pub(super) set_for_each: ObjectRef,
    pub(super) set_size_getter: ObjectRef,
    pub(super) weak_map_get: ObjectRef,
    pub(super) weak_map_set: ObjectRef,
    pub(super) weak_map_has: ObjectRef,
    pub(super) weak_map_delete: ObjectRef,
    pub(super) weak_set_add: ObjectRef,
    pub(super) weak_set_has: ObjectRef,
    pub(super) weak_set_delete: ObjectRef,
    pub(super) weak_ref_deref: ObjectRef,
    pub(super) finalization_registry_register: ObjectRef,
    pub(super) finalization_registry_unregister: ObjectRef,
    pub(super) map_prototype: ObjectRef,
    pub(super) set_prototype: ObjectRef,
    pub(super) weak_map_prototype: ObjectRef,
    pub(super) weak_set_prototype: ObjectRef,
    pub(super) weak_ref_prototype: ObjectRef,
    pub(super) finalization_registry_prototype: ObjectRef,
}
