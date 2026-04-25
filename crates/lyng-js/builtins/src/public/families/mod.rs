mod functions;
mod objects;

use lyng_js_types::{EnvironmentRef, ObjectRef, RealmRef, ShapeId};

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
