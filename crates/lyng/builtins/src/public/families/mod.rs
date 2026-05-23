mod arrays;
mod binary_data;
mod collections;
mod date;
mod descriptors;
mod errors;
mod functions;
mod globals;
mod installed;
mod intrinsics;
mod iterators;
mod json;
mod modules;
mod object_reflection;
mod objects;
mod primitives;
mod promises;
mod prototype_links;
mod regexp;
mod scaffolding;
mod strings;

use crate::bootstrap::BuiltinBootstrapError;
use crate::public::{
    allocate_builtin_function_object, public_builtin_metadata, BuiltinCache, PublicRealmBuiltins,
};
use lyng_js_env::Agent;
use lyng_js_types::{BuiltinFunctionId, EnvironmentRef, ObjectRef, RealmRef, ShapeId};

pub(super) use arrays::array_builtin_object;
pub(super) use binary_data::binary_data_builtin_object;
pub(super) use collections::collection_builtin_object;
pub(super) use date::date_builtin_object;
pub(super) use errors::error_builtin_object;
pub(super) use functions::function_builtin_object;
pub(super) use globals::global_function_builtin_object;
pub(super) use installed::InstalledBuiltinFamilies;
pub(super) use intrinsics::{install_public_realm_intrinsics, PublicRealmPrototypeHandles};
pub(super) use iterators::iterator_builtin_object;
pub(super) use json::json_builtin_object;
pub(super) use modules::module_builtin_object;
pub(super) use object_reflection::object_reflection_builtin_object;
pub(super) use objects::object_builtin_object;
pub(super) use primitives::primitive_builtin_object;
pub(super) use promises::promise_disposal_builtin_object;
pub(super) use prototype_links::link_installed_family_prototypes;
pub(super) use regexp::regexp_builtin_object;
pub(super) use scaffolding::{
    allocate_public_realm_scaffolding, PublicRealmScaffolding, ScaffoldingRequest,
};
pub(super) use strings::string_builtin_object;

pub(super) fn install_public_builtin_families(
    agent: &mut Agent,
    scaffolding: &PublicRealmScaffolding,
) -> PublicRealmBuiltins {
    let family_context = scaffolding.cx;
    let object_family = objects::install_object_family(agent, family_context);
    let function_family =
        functions::install_function_family(agent, family_context, scaffolding.function);
    let iterator_family =
        iterators::install_iterator_family(agent, family_context, scaffolding.iterator);
    let collection_family =
        collections::install_collection_family(agent, family_context, scaffolding.collection);
    let binary_data_family =
        binary_data::install_binary_data_family(agent, family_context, scaffolding.binary_data);
    let array_family = arrays::install_array_family(agent, family_context, scaffolding.array);
    let string_family = strings::install_string_family(agent, family_context, scaffolding.string);
    let regexp_family = regexp::install_regexp_family(agent, family_context, scaffolding.regexp);
    let date_family = date::install_date_family(agent, family_context, scaffolding.date);
    let primitive_family = primitives::install_primitive_family(
        agent,
        family_context,
        scaffolding.primitive,
        scaffolding.primitive_objects,
    );
    let json_family = json::install_json_family(agent, family_context, scaffolding.json);
    let object_reflection_family = object_reflection::install_object_reflection_family(
        agent,
        family_context,
        scaffolding.object_reflection,
    );
    let module_family = modules::install_module_family(agent, family_context, scaffolding.module);
    let error_family = errors::install_error_family(agent, family_context, scaffolding.error);
    let promise_disposal_family = promises::install_promise_disposal_family(
        agent,
        family_context,
        scaffolding.promise_disposal,
    );
    let global_function_family = globals::install_global_function_family(agent, family_context);

    InstalledBuiltinFamilies {
        object: object_family,
        function: function_family,
        iterator: iterator_family,
        collection: collection_family,
        binary_data: binary_data_family,
        array: array_family,
        string: string_family,
        regexp: regexp_family,
        date: date_family,
        primitive: primitive_family,
        json: json_family,
        object_reflection: object_reflection_family,
        module: module_family,
        error: error_family,
        promise_disposal: promise_disposal_family,
        global_function: global_function_family,
    }
    .public_realm_builtins()
}

pub(super) fn install_public_family_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
) -> Result<(), BuiltinBootstrapError> {
    modules::install_module_family_descriptors(agent, builtins);
    objects::install_object_family_descriptors(agent, cache, realm, builtins)?;
    functions::install_function_family_descriptors(agent, cache, realm, builtins)?;
    arrays::install_array_family_descriptors(agent, cache, realm, builtins)?;
    collections::install_collection_family_descriptors(agent, cache, realm, builtins)?;
    iterators::install_iterator_family_descriptors(agent, cache, realm, builtins)?;
    object_reflection::install_object_reflection_family_descriptors(agent, cache, realm)?;
    json::install_json_family_descriptors(agent, cache, realm)?;
    errors::install_error_family_descriptors(agent, cache, realm, builtins)?;
    strings::install_string_family_descriptors(agent, cache, realm, builtins)?;
    regexp::install_regexp_family_descriptors(agent, cache, realm, builtins)?;
    date::install_date_family_descriptors(agent, cache, realm, builtins)?;
    primitives::install_primitive_family_descriptors(agent, cache, realm, builtins)?;
    promises::install_promise_disposal_family_descriptors(agent, cache, realm, builtins)?;
    binary_data::install_binary_data_family_descriptors(agent, cache, realm, builtins)
}

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
    install_public_builtin_function_with_function_prototype(
        agent,
        cx,
        cx.function_prototype,
        entry,
        prototype_object,
    )
}

pub(super) fn install_public_builtin_function_with_metadata(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    entry: BuiltinFunctionId,
    metadata: crate::BuiltinEntryMetadata,
    prototype_object: Option<ObjectRef>,
) -> ObjectRef {
    allocate_public_builtin_function(
        agent,
        cx,
        cx.function_prototype,
        entry,
        metadata,
        prototype_object,
    )
}

pub(super) fn install_public_builtin_function_with_function_prototype(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    function_prototype: ObjectRef,
    entry: BuiltinFunctionId,
    prototype_object: Option<ObjectRef>,
) -> ObjectRef {
    let metadata =
        public_builtin_metadata(entry).expect("family installer entry must have public metadata");
    allocate_public_builtin_function(
        agent,
        cx,
        function_prototype,
        entry,
        metadata,
        prototype_object,
    )
}

fn allocate_public_builtin_function(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    function_prototype: ObjectRef,
    entry: BuiltinFunctionId,
    metadata: crate::BuiltinEntryMetadata,
    prototype_object: Option<ObjectRef>,
) -> ObjectRef {
    allocate_builtin_function_object(
        agent,
        cx.realm,
        cx.global_env,
        cx.root_shape,
        function_prototype,
        cx.object_prototype,
        entry,
        metadata,
        prototype_object,
    )
}

pub(super) fn install_public_ordinary_object(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototype: Option<ObjectRef>,
) -> ObjectRef {
    crate::public::allocate_builtin_ordinary_object(agent, cx.root_shape, prototype)
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
pub(super) struct IteratorFamilyPrototypes {
    pub(super) async_iterator_prototype: ObjectRef,
    pub(super) iterator_prototype: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct IteratorFamilyBuiltins {
    pub(super) async_iterator_prototype: ObjectRef,
    pub(super) iterator_prototype_iterator: ObjectRef,
    pub(super) async_iterator_method: ObjectRef,
    pub(super) async_iterator_dispose: ObjectRef,
    pub(super) map_iterator_next: ObjectRef,
    pub(super) set_iterator_next: ObjectRef,
    pub(super) iterator: ObjectRef,
    pub(super) iterator_from: ObjectRef,
    pub(super) iterator_concat: ObjectRef,
    pub(super) iterator_zip: ObjectRef,
    pub(super) iterator_zip_keyed: ObjectRef,
    pub(super) iterator_reduce: ObjectRef,
    pub(super) iterator_for_each: ObjectRef,
    pub(super) iterator_some: ObjectRef,
    pub(super) iterator_every: ObjectRef,
    pub(super) iterator_find: ObjectRef,
    pub(super) iterator_to_array: ObjectRef,
    pub(super) iterator_map: ObjectRef,
    pub(super) iterator_filter: ObjectRef,
    pub(super) iterator_take: ObjectRef,
    pub(super) iterator_drop: ObjectRef,
    pub(super) iterator_dispose: ObjectRef,
    pub(super) iterator_flat_map: ObjectRef,
    pub(super) iterator_helper_next: ObjectRef,
    pub(super) iterator_helper_return: ObjectRef,
    pub(super) iterator_to_string_tag_getter: ObjectRef,
    pub(super) iterator_to_string_tag_setter: ObjectRef,
    pub(super) iterator_constructor_getter: ObjectRef,
    pub(super) iterator_constructor_setter: ObjectRef,
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
    pub(super) map_group_by: ObjectRef,
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
    pub(super) map_get_or_insert: ObjectRef,
    pub(super) map_get_or_insert_computed: ObjectRef,
    pub(super) set_add: ObjectRef,
    pub(super) set_has: ObjectRef,
    pub(super) set_delete: ObjectRef,
    pub(super) set_clear: ObjectRef,
    pub(super) set_entries: ObjectRef,
    pub(super) set_values: ObjectRef,
    pub(super) set_keys: ObjectRef,
    pub(super) set_for_each: ObjectRef,
    pub(super) set_size_getter: ObjectRef,
    pub(super) set_union: ObjectRef,
    pub(super) set_intersection: ObjectRef,
    pub(super) set_difference: ObjectRef,
    pub(super) set_symmetric_difference: ObjectRef,
    pub(super) set_is_subset_of: ObjectRef,
    pub(super) set_is_superset_of: ObjectRef,
    pub(super) set_is_disjoint_from: ObjectRef,
    pub(super) weak_map_get: ObjectRef,
    pub(super) weak_map_set: ObjectRef,
    pub(super) weak_map_has: ObjectRef,
    pub(super) weak_map_delete: ObjectRef,
    pub(super) weak_map_get_or_insert: ObjectRef,
    pub(super) weak_map_get_or_insert_computed: ObjectRef,
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

#[derive(Clone, Copy, Debug)]
#[allow(clippy::struct_field_names)]
pub(super) struct BinaryDataFamilyPrototypes {
    pub(super) array_buffer_prototype: ObjectRef,
    pub(super) shared_array_buffer_prototype: ObjectRef,
    pub(super) data_view_prototype: ObjectRef,
    pub(super) typed_array_prototype: ObjectRef,
    pub(super) int8_array_prototype: ObjectRef,
    pub(super) int16_array_prototype: ObjectRef,
    pub(super) int32_array_prototype: ObjectRef,
    pub(super) float16_array_prototype: ObjectRef,
    pub(super) float32_array_prototype: ObjectRef,
    pub(super) float64_array_prototype: ObjectRef,
    pub(super) big_int64_array_prototype: ObjectRef,
    pub(super) big_uint64_array_prototype: ObjectRef,
    pub(super) uint32_array_prototype: ObjectRef,
    pub(super) uint16_array_prototype: ObjectRef,
    pub(super) uint8_clamped_array_prototype: ObjectRef,
    pub(super) uint8_array_prototype: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct BinaryDataFamilyBuiltins {
    pub(super) array_buffer: ObjectRef,
    pub(super) shared_array_buffer: ObjectRef,
    pub(super) atomics: ObjectRef,
    pub(super) array_buffer_is_view: ObjectRef,
    pub(super) data_view: ObjectRef,
    pub(super) typed_array: ObjectRef,
    pub(super) typed_array_from: ObjectRef,
    pub(super) typed_array_of: ObjectRef,
    pub(super) int8_array: ObjectRef,
    pub(super) int16_array: ObjectRef,
    pub(super) int32_array: ObjectRef,
    pub(super) float16_array: ObjectRef,
    pub(super) float32_array: ObjectRef,
    pub(super) float64_array: ObjectRef,
    pub(super) big_int64_array: ObjectRef,
    pub(super) big_uint64_array: ObjectRef,
    pub(super) uint32_array: ObjectRef,
    pub(super) uint16_array: ObjectRef,
    pub(super) uint8_clamped_array: ObjectRef,
    pub(super) uint8_array: ObjectRef,
    pub(super) array_buffer_prototype: ObjectRef,
    pub(super) shared_array_buffer_prototype: ObjectRef,
    pub(super) array_buffer_byte_length_getter: ObjectRef,
    pub(super) array_buffer_detached_getter: ObjectRef,
    pub(super) array_buffer_max_byte_length_getter: ObjectRef,
    pub(super) array_buffer_resizable_getter: ObjectRef,
    pub(super) array_buffer_resize: ObjectRef,
    pub(super) array_buffer_slice: ObjectRef,
    pub(super) array_buffer_transfer: ObjectRef,
    pub(super) array_buffer_transfer_to_fixed_length: ObjectRef,
    pub(super) shared_array_buffer_byte_length_getter: ObjectRef,
    pub(super) shared_array_buffer_grow: ObjectRef,
    pub(super) shared_array_buffer_growable_getter: ObjectRef,
    pub(super) shared_array_buffer_max_byte_length_getter: ObjectRef,
    pub(super) shared_array_buffer_slice: ObjectRef,
    pub(super) atomics_load: ObjectRef,
    pub(super) atomics_store: ObjectRef,
    pub(super) atomics_add: ObjectRef,
    pub(super) atomics_sub: ObjectRef,
    pub(super) atomics_and: ObjectRef,
    pub(super) atomics_or: ObjectRef,
    pub(super) atomics_xor: ObjectRef,
    pub(super) atomics_exchange: ObjectRef,
    pub(super) atomics_compare_exchange: ObjectRef,
    pub(super) atomics_notify: ObjectRef,
    pub(super) atomics_wait: ObjectRef,
    pub(super) atomics_wait_async: ObjectRef,
    pub(super) atomics_pause: ObjectRef,
    pub(super) atomics_is_lock_free: ObjectRef,
    pub(super) data_view_prototype: ObjectRef,
    pub(super) data_view_buffer_getter: ObjectRef,
    pub(super) data_view_byte_length_getter: ObjectRef,
    pub(super) data_view_byte_offset_getter: ObjectRef,
    pub(super) data_view_get_float32: ObjectRef,
    pub(super) data_view_get_float64: ObjectRef,
    pub(super) data_view_get_int16: ObjectRef,
    pub(super) data_view_get_int32: ObjectRef,
    pub(super) data_view_get_int8: ObjectRef,
    pub(super) data_view_get_uint16: ObjectRef,
    pub(super) data_view_get_uint32: ObjectRef,
    pub(super) data_view_get_uint8: ObjectRef,
    pub(super) data_view_set_float32: ObjectRef,
    pub(super) data_view_set_float64: ObjectRef,
    pub(super) data_view_set_int16: ObjectRef,
    pub(super) data_view_set_int32: ObjectRef,
    pub(super) data_view_set_int8: ObjectRef,
    pub(super) data_view_set_uint16: ObjectRef,
    pub(super) data_view_set_uint32: ObjectRef,
    pub(super) data_view_set_uint8: ObjectRef,
    pub(super) data_view_get_big_int64: ObjectRef,
    pub(super) data_view_get_big_uint64: ObjectRef,
    pub(super) data_view_set_big_int64: ObjectRef,
    pub(super) data_view_set_big_uint64: ObjectRef,
    pub(super) data_view_get_float16: ObjectRef,
    pub(super) data_view_set_float16: ObjectRef,
    pub(super) typed_array_prototype: ObjectRef,
    pub(super) int8_array_prototype: ObjectRef,
    pub(super) int16_array_prototype: ObjectRef,
    pub(super) int32_array_prototype: ObjectRef,
    pub(super) float16_array_prototype: ObjectRef,
    pub(super) float32_array_prototype: ObjectRef,
    pub(super) float64_array_prototype: ObjectRef,
    pub(super) big_int64_array_prototype: ObjectRef,
    pub(super) big_uint64_array_prototype: ObjectRef,
    pub(super) uint32_array_prototype: ObjectRef,
    pub(super) uint16_array_prototype: ObjectRef,
    pub(super) uint8_clamped_array_prototype: ObjectRef,
    pub(super) uint8_array_prototype: ObjectRef,
    pub(super) uint8_array_buffer_getter: ObjectRef,
    pub(super) uint8_array_byte_length_getter: ObjectRef,
    pub(super) uint8_array_byte_offset_getter: ObjectRef,
    pub(super) uint8_array_length_getter: ObjectRef,
    pub(super) uint8_array_values: ObjectRef,
    pub(super) uint8_array_keys: ObjectRef,
    pub(super) uint8_array_entries: ObjectRef,
    pub(super) uint8_array_set: ObjectRef,
    pub(super) uint8_array_slice: ObjectRef,
    pub(super) uint8_array_subarray: ObjectRef,
    pub(super) uint8_array_from_base64: ObjectRef,
    pub(super) uint8_array_from_hex: ObjectRef,
    pub(super) uint8_array_set_from_base64: ObjectRef,
    pub(super) uint8_array_set_from_hex: ObjectRef,
    pub(super) uint8_array_to_base64: ObjectRef,
    pub(super) uint8_array_to_hex: ObjectRef,
    pub(super) typed_array_every: ObjectRef,
    pub(super) typed_array_some: ObjectRef,
    pub(super) typed_array_find: ObjectRef,
    pub(super) typed_array_find_index: ObjectRef,
    pub(super) typed_array_find_last: ObjectRef,
    pub(super) typed_array_find_last_index: ObjectRef,
    pub(super) typed_array_fill: ObjectRef,
    pub(super) typed_array_copy_within: ObjectRef,
    pub(super) typed_array_filter: ObjectRef,
    pub(super) typed_array_for_each: ObjectRef,
    pub(super) typed_array_includes: ObjectRef,
    pub(super) typed_array_index_of: ObjectRef,
    pub(super) typed_array_join: ObjectRef,
    pub(super) typed_array_last_index_of: ObjectRef,
    pub(super) typed_array_map: ObjectRef,
    pub(super) typed_array_reduce: ObjectRef,
    pub(super) typed_array_reduce_right: ObjectRef,
    pub(super) typed_array_reverse: ObjectRef,
    pub(super) typed_array_sort: ObjectRef,
    pub(super) typed_array_to_locale_string: ObjectRef,
    pub(super) typed_array_to_string: ObjectRef,
    pub(super) typed_array_to_reversed: ObjectRef,
    pub(super) typed_array_to_sorted: ObjectRef,
    pub(super) typed_array_with: ObjectRef,
    pub(super) typed_array_at: ObjectRef,
    pub(super) typed_array_to_string_tag_getter: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
#[allow(clippy::struct_field_names)]
pub(super) struct ArrayFamilyPrototypes {
    pub(super) array_prototype: ObjectRef,
    pub(super) array_unscopables: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ArrayFamilyBuiltins {
    pub(super) array: ObjectRef,
    pub(super) array_from: ObjectRef,
    pub(super) array_from_async: ObjectRef,
    pub(super) array_of: ObjectRef,
    pub(super) array_unscopables: ObjectRef,
    pub(super) array_is_array: ObjectRef,
    pub(super) array_at: ObjectRef,
    pub(super) array_concat: ObjectRef,
    pub(super) array_copy_within: ObjectRef,
    pub(super) array_fill: ObjectRef,
    pub(super) array_flat: ObjectRef,
    pub(super) array_flat_map: ObjectRef,
    pub(super) array_join: ObjectRef,
    pub(super) array_pop: ObjectRef,
    pub(super) array_push: ObjectRef,
    pub(super) array_shift: ObjectRef,
    pub(super) array_unshift: ObjectRef,
    pub(super) array_every: ObjectRef,
    pub(super) array_filter: ObjectRef,
    pub(super) array_find: ObjectRef,
    pub(super) array_find_index: ObjectRef,
    pub(super) array_find_last: ObjectRef,
    pub(super) array_find_last_index: ObjectRef,
    pub(super) array_for_each: ObjectRef,
    pub(super) array_includes: ObjectRef,
    pub(super) array_index_of: ObjectRef,
    pub(super) array_map: ObjectRef,
    pub(super) array_reduce: ObjectRef,
    pub(super) array_reduce_right: ObjectRef,
    pub(super) array_reverse: ObjectRef,
    pub(super) array_slice: ObjectRef,
    pub(super) array_some: ObjectRef,
    pub(super) array_last_index_of: ObjectRef,
    pub(super) array_sort: ObjectRef,
    pub(super) array_splice: ObjectRef,
    pub(super) array_to_reversed: ObjectRef,
    pub(super) array_to_sorted: ObjectRef,
    pub(super) array_to_spliced: ObjectRef,
    pub(super) array_to_string: ObjectRef,
    pub(super) array_to_locale_string: ObjectRef,
    pub(super) array_values: ObjectRef,
    pub(super) array_keys: ObjectRef,
    pub(super) array_entries: ObjectRef,
    pub(super) array_with: ObjectRef,
    pub(super) array_iterator_next: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
#[allow(clippy::struct_field_names)]
pub(super) struct StringFamilyPrototypes {
    pub(super) string_prototype: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct StringFamilyBuiltins {
    pub(super) string: ObjectRef,
    pub(super) string_prototype: ObjectRef,
    pub(super) string_iterator: ObjectRef,
    pub(super) string_iterator_next: ObjectRef,
    pub(super) string_to_string: ObjectRef,
    pub(super) string_value_of: ObjectRef,
    pub(super) string_concat: ObjectRef,
    pub(super) string_char_at: ObjectRef,
    pub(super) string_char_code_at: ObjectRef,
    pub(super) string_from_char_code: ObjectRef,
    pub(super) string_from_code_point: ObjectRef,
    pub(super) string_raw: ObjectRef,
    pub(super) string_at: ObjectRef,
    pub(super) string_code_point_at: ObjectRef,
    pub(super) string_ends_with: ObjectRef,
    pub(super) string_includes: ObjectRef,
    pub(super) string_index_of: ObjectRef,
    pub(super) string_is_well_formed: ObjectRef,
    pub(super) string_locale_compare: ObjectRef,
    pub(super) string_match: ObjectRef,
    pub(super) string_match_all: ObjectRef,
    pub(super) string_normalize: ObjectRef,
    pub(super) string_last_index_of: ObjectRef,
    pub(super) string_pad_end: ObjectRef,
    pub(super) string_pad_start: ObjectRef,
    pub(super) string_repeat: ObjectRef,
    pub(super) string_replace: ObjectRef,
    pub(super) string_replace_all: ObjectRef,
    pub(super) string_search: ObjectRef,
    pub(super) string_split: ObjectRef,
    pub(super) string_slice: ObjectRef,
    pub(super) string_substr: ObjectRef,
    pub(super) string_substring: ObjectRef,
    pub(super) string_starts_with: ObjectRef,
    pub(super) string_to_locale_lower_case: ObjectRef,
    pub(super) string_to_locale_upper_case: ObjectRef,
    pub(super) string_to_lower_case: ObjectRef,
    pub(super) string_to_upper_case: ObjectRef,
    pub(super) string_to_well_formed: ObjectRef,
    pub(super) string_trim: ObjectRef,
    pub(super) string_trim_end: ObjectRef,
    pub(super) string_trim_start: ObjectRef,
    pub(super) string_anchor: ObjectRef,
    pub(super) string_big: ObjectRef,
    pub(super) string_blink: ObjectRef,
    pub(super) string_bold: ObjectRef,
    pub(super) string_fixed: ObjectRef,
    pub(super) string_fontcolor: ObjectRef,
    pub(super) string_fontsize: ObjectRef,
    pub(super) string_italics: ObjectRef,
    pub(super) string_link: ObjectRef,
    pub(super) string_small: ObjectRef,
    pub(super) string_strike: ObjectRef,
    pub(super) string_sub: ObjectRef,
    pub(super) string_sup: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
#[allow(clippy::struct_field_names)]
pub(super) struct RegExpFamilyPrototypes {
    pub(super) regexp_prototype: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RegExpFamilyBuiltins {
    pub(super) regexp: ObjectRef,
    pub(super) regexp_escape: ObjectRef,
    pub(super) regexp_prototype: ObjectRef,
    pub(super) regexp_compile: ObjectRef,
    pub(super) regexp_legacy_input_getter: ObjectRef,
    pub(super) regexp_legacy_input_setter: ObjectRef,
    pub(super) regexp_legacy_last_match_getter: ObjectRef,
    pub(super) regexp_legacy_last_paren_getter: ObjectRef,
    pub(super) regexp_legacy_left_context_getter: ObjectRef,
    pub(super) regexp_legacy_right_context_getter: ObjectRef,
    pub(super) regexp_legacy_paren1_getter: ObjectRef,
    pub(super) regexp_legacy_paren2_getter: ObjectRef,
    pub(super) regexp_legacy_paren3_getter: ObjectRef,
    pub(super) regexp_legacy_paren4_getter: ObjectRef,
    pub(super) regexp_legacy_paren5_getter: ObjectRef,
    pub(super) regexp_legacy_paren6_getter: ObjectRef,
    pub(super) regexp_legacy_paren7_getter: ObjectRef,
    pub(super) regexp_legacy_paren8_getter: ObjectRef,
    pub(super) regexp_legacy_paren9_getter: ObjectRef,
    pub(super) regexp_to_string: ObjectRef,
    pub(super) regexp_exec: ObjectRef,
    pub(super) regexp_test: ObjectRef,
    pub(super) regexp_global_getter: ObjectRef,
    pub(super) regexp_ignore_case_getter: ObjectRef,
    pub(super) regexp_multiline_getter: ObjectRef,
    pub(super) regexp_dot_all_getter: ObjectRef,
    pub(super) regexp_unicode_getter: ObjectRef,
    pub(super) regexp_unicode_sets_getter: ObjectRef,
    pub(super) regexp_sticky_getter: ObjectRef,
    pub(super) regexp_source_getter: ObjectRef,
    pub(super) regexp_flags_getter: ObjectRef,
    pub(super) regexp_has_indices_getter: ObjectRef,
    pub(super) regexp_species_getter: ObjectRef,
    pub(super) regexp_symbol_match: ObjectRef,
    pub(super) regexp_symbol_replace: ObjectRef,
    pub(super) regexp_symbol_search: ObjectRef,
    pub(super) regexp_symbol_split: ObjectRef,
    pub(super) regexp_symbol_match_all: ObjectRef,
    pub(super) regexp_string_iterator_next: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
#[allow(clippy::struct_field_names)]
pub(super) struct DateFamilyPrototypes {
    pub(super) date_prototype: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct DateFamilyBuiltins {
    pub(super) date: ObjectRef,
    pub(super) date_prototype: ObjectRef,
    pub(super) date_now: ObjectRef,
    pub(super) date_parse: ObjectRef,
    pub(super) date_utc: ObjectRef,
    pub(super) date_to_string: ObjectRef,
    pub(super) date_to_date_string: ObjectRef,
    pub(super) date_to_time_string: ObjectRef,
    pub(super) date_to_locale_string: ObjectRef,
    pub(super) date_to_locale_date_string: ObjectRef,
    pub(super) date_to_locale_time_string: ObjectRef,
    pub(super) date_value_of: ObjectRef,
    pub(super) date_get_time: ObjectRef,
    pub(super) date_get_full_year: ObjectRef,
    pub(super) date_get_year: ObjectRef,
    pub(super) date_get_utc_full_year: ObjectRef,
    pub(super) date_get_month: ObjectRef,
    pub(super) date_get_utc_month: ObjectRef,
    pub(super) date_get_date: ObjectRef,
    pub(super) date_get_utc_date: ObjectRef,
    pub(super) date_get_day: ObjectRef,
    pub(super) date_get_utc_day: ObjectRef,
    pub(super) date_get_hours: ObjectRef,
    pub(super) date_get_utc_hours: ObjectRef,
    pub(super) date_get_minutes: ObjectRef,
    pub(super) date_get_utc_minutes: ObjectRef,
    pub(super) date_get_seconds: ObjectRef,
    pub(super) date_get_utc_seconds: ObjectRef,
    pub(super) date_get_milliseconds: ObjectRef,
    pub(super) date_get_utc_milliseconds: ObjectRef,
    pub(super) date_get_timezone_offset: ObjectRef,
    pub(super) date_set_time: ObjectRef,
    pub(super) date_set_milliseconds: ObjectRef,
    pub(super) date_set_utc_milliseconds: ObjectRef,
    pub(super) date_set_seconds: ObjectRef,
    pub(super) date_set_utc_seconds: ObjectRef,
    pub(super) date_set_minutes: ObjectRef,
    pub(super) date_set_utc_minutes: ObjectRef,
    pub(super) date_set_hours: ObjectRef,
    pub(super) date_set_utc_hours: ObjectRef,
    pub(super) date_set_date: ObjectRef,
    pub(super) date_set_utc_date: ObjectRef,
    pub(super) date_set_month: ObjectRef,
    pub(super) date_set_utc_month: ObjectRef,
    pub(super) date_set_full_year: ObjectRef,
    pub(super) date_set_year: ObjectRef,
    pub(super) date_set_utc_full_year: ObjectRef,
    pub(super) date_to_utc_string: ObjectRef,
    pub(super) date_to_iso_string: ObjectRef,
    pub(super) date_to_json: ObjectRef,
    pub(super) date_to_primitive: ObjectRef,
    pub(super) date_to_temporal_instant: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
#[allow(clippy::struct_field_names)]
pub(super) struct PrimitiveFamilyPrototypes {
    pub(super) number_prototype: ObjectRef,
    pub(super) bigint_prototype: ObjectRef,
    pub(super) boolean_prototype: ObjectRef,
    pub(super) symbol_prototype: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PrimitiveFamilyObjects {
    pub(super) math: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PrimitiveFamilyBuiltins {
    pub(super) number: ObjectRef,
    pub(super) number_prototype: ObjectRef,
    pub(super) number_is_finite: ObjectRef,
    pub(super) number_is_integer: ObjectRef,
    pub(super) number_is_nan: ObjectRef,
    pub(super) number_is_safe_integer: ObjectRef,
    pub(super) number_to_exponential: ObjectRef,
    pub(super) number_to_fixed: ObjectRef,
    pub(super) number_to_locale_string: ObjectRef,
    pub(super) number_to_precision: ObjectRef,
    pub(super) number_to_string: ObjectRef,
    pub(super) number_value_of: ObjectRef,
    pub(super) math: ObjectRef,
    pub(super) math_abs: ObjectRef,
    pub(super) math_acos: ObjectRef,
    pub(super) math_acosh: ObjectRef,
    pub(super) math_asin: ObjectRef,
    pub(super) math_asinh: ObjectRef,
    pub(super) math_atan: ObjectRef,
    pub(super) math_atan2: ObjectRef,
    pub(super) math_atanh: ObjectRef,
    pub(super) math_cbrt: ObjectRef,
    pub(super) math_ceil: ObjectRef,
    pub(super) math_clz32: ObjectRef,
    pub(super) math_cos: ObjectRef,
    pub(super) math_cosh: ObjectRef,
    pub(super) math_exp: ObjectRef,
    pub(super) math_expm1: ObjectRef,
    pub(super) math_f16round: ObjectRef,
    pub(super) math_floor: ObjectRef,
    pub(super) math_fround: ObjectRef,
    pub(super) math_hypot: ObjectRef,
    pub(super) math_imul: ObjectRef,
    pub(super) math_log: ObjectRef,
    pub(super) math_log10: ObjectRef,
    pub(super) math_log1p: ObjectRef,
    pub(super) math_log2: ObjectRef,
    pub(super) math_max: ObjectRef,
    pub(super) math_min: ObjectRef,
    pub(super) math_pow: ObjectRef,
    pub(super) math_random: ObjectRef,
    pub(super) math_round: ObjectRef,
    pub(super) math_sign: ObjectRef,
    pub(super) math_sin: ObjectRef,
    pub(super) math_sinh: ObjectRef,
    pub(super) math_sqrt: ObjectRef,
    pub(super) math_sum_precise: ObjectRef,
    pub(super) math_tan: ObjectRef,
    pub(super) math_tanh: ObjectRef,
    pub(super) math_trunc: ObjectRef,
    pub(super) bigint: ObjectRef,
    pub(super) bigint_as_int_n: ObjectRef,
    pub(super) bigint_as_uint_n: ObjectRef,
    pub(super) bigint_prototype: ObjectRef,
    pub(super) bigint_to_string: ObjectRef,
    pub(super) bigint_value_of: ObjectRef,
    pub(super) boolean: ObjectRef,
    pub(super) boolean_prototype: ObjectRef,
    pub(super) boolean_to_string: ObjectRef,
    pub(super) boolean_value_of: ObjectRef,
    pub(super) symbol: ObjectRef,
    pub(super) symbol_prototype: ObjectRef,
    pub(super) symbol_for: ObjectRef,
    pub(super) symbol_key_for: ObjectRef,
    pub(super) symbol_to_string: ObjectRef,
    pub(super) symbol_value_of: ObjectRef,
    pub(super) symbol_to_primitive: ObjectRef,
    pub(super) array_species_getter: ObjectRef,
    pub(super) symbol_description_getter: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct JsonFamilyObjects {
    pub(super) json: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct JsonFamilyBuiltins {
    pub(super) json: ObjectRef,
    pub(super) json_parse: ObjectRef,
    pub(super) json_stringify: ObjectRef,
    pub(super) json_raw_json: ObjectRef,
    pub(super) json_is_raw_json: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ObjectReflectionFamilyObjects {
    pub(super) reflect: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ObjectReflectionFamilyBuiltins {
    pub(super) reflect: ObjectRef,
    pub(super) reflect_apply: ObjectRef,
    pub(super) reflect_construct: ObjectRef,
    pub(super) reflect_define_property: ObjectRef,
    pub(super) reflect_delete_property: ObjectRef,
    pub(super) reflect_get: ObjectRef,
    pub(super) reflect_get_own_property_descriptor: ObjectRef,
    pub(super) reflect_get_prototype_of: ObjectRef,
    pub(super) reflect_has: ObjectRef,
    pub(super) reflect_is_extensible: ObjectRef,
    pub(super) reflect_own_keys: ObjectRef,
    pub(super) reflect_prevent_extensions: ObjectRef,
    pub(super) reflect_set: ObjectRef,
    pub(super) reflect_set_prototype_of: ObjectRef,
    pub(super) proxy: ObjectRef,
    pub(super) proxy_revocable: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ModuleFamilyPrototypes {
    pub(super) abstract_module_source_prototype: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
#[allow(clippy::struct_field_names)]
pub(super) struct ModuleFamilyBuiltins {
    pub(super) abstract_module_source: ObjectRef,
    pub(super) abstract_module_source_prototype: ObjectRef,
    pub(super) abstract_module_source_to_string_tag_getter: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
#[allow(clippy::struct_field_names)]
pub(super) struct ErrorFamilyPrototypes {
    pub(super) error_prototype: ObjectRef,
    pub(super) eval_error_prototype: ObjectRef,
    pub(super) range_error_prototype: ObjectRef,
    pub(super) reference_error_prototype: ObjectRef,
    pub(super) syntax_error_prototype: ObjectRef,
    pub(super) type_error_prototype: ObjectRef,
    pub(super) uri_error_prototype: ObjectRef,
    pub(super) aggregate_error_prototype: ObjectRef,
    pub(super) suppressed_error_prototype: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ErrorFamilyBuiltins {
    pub(super) error: ObjectRef,
    pub(super) error_prototype: ObjectRef,
    pub(super) error_to_string: ObjectRef,
    pub(super) eval_error: ObjectRef,
    pub(super) eval_error_prototype: ObjectRef,
    pub(super) range_error: ObjectRef,
    pub(super) range_error_prototype: ObjectRef,
    pub(super) reference_error: ObjectRef,
    pub(super) reference_error_prototype: ObjectRef,
    pub(super) syntax_error: ObjectRef,
    pub(super) syntax_error_prototype: ObjectRef,
    pub(super) type_error: ObjectRef,
    pub(super) type_error_prototype: ObjectRef,
    pub(super) uri_error: ObjectRef,
    pub(super) uri_error_prototype: ObjectRef,
    pub(super) aggregate_error: ObjectRef,
    pub(super) aggregate_error_prototype: ObjectRef,
    pub(super) suppressed_error: ObjectRef,
    pub(super) suppressed_error_prototype: ObjectRef,
    pub(super) error_is_error: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
#[allow(clippy::struct_field_names)]
pub(super) struct PromiseDisposalFamilyPrototypes {
    pub(super) promise_prototype: ObjectRef,
    pub(super) disposable_stack_prototype: ObjectRef,
    pub(super) async_disposable_stack_prototype: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PromiseDisposalFamilyBuiltins {
    pub(super) promise: ObjectRef,
    pub(super) promise_prototype: ObjectRef,
    pub(super) disposable_stack: ObjectRef,
    pub(super) disposable_stack_prototype: ObjectRef,
    pub(super) async_disposable_stack: ObjectRef,
    pub(super) async_disposable_stack_prototype: ObjectRef,
    pub(super) disposable_stack_use: ObjectRef,
    pub(super) disposable_stack_adopt: ObjectRef,
    pub(super) disposable_stack_defer: ObjectRef,
    pub(super) disposable_stack_move: ObjectRef,
    pub(super) disposable_stack_disposed_getter: ObjectRef,
    pub(super) disposable_stack_dispose: ObjectRef,
    pub(super) async_disposable_stack_use: ObjectRef,
    pub(super) async_disposable_stack_adopt: ObjectRef,
    pub(super) async_disposable_stack_defer: ObjectRef,
    pub(super) async_disposable_stack_move: ObjectRef,
    pub(super) async_disposable_stack_disposed_getter: ObjectRef,
    pub(super) async_disposable_stack_dispose_async: ObjectRef,
    pub(super) create_sync_disposal_scope: ObjectRef,
    pub(super) create_async_disposal_scope: ObjectRef,
    pub(super) add_sync_disposable_resource: ObjectRef,
    pub(super) add_async_disposable_resource: ObjectRef,
    pub(super) dispose_scope: ObjectRef,
    pub(super) dispose_scope_async: ObjectRef,
    pub(super) promise_then: ObjectRef,
    pub(super) promise_catch: ObjectRef,
    pub(super) promise_finally: ObjectRef,
    pub(super) promise_resolve: ObjectRef,
    pub(super) promise_reject: ObjectRef,
    pub(super) promise_all: ObjectRef,
    pub(super) promise_all_settled: ObjectRef,
    pub(super) promise_race: ObjectRef,
    pub(super) promise_any: ObjectRef,
    pub(super) promise_try: ObjectRef,
    pub(super) promise_with_resolvers: ObjectRef,
    pub(super) promise_species_getter: ObjectRef,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct GlobalFunctionFamilyBuiltins {
    pub(super) eval: ObjectRef,
    pub(super) parse_int: ObjectRef,
    pub(super) parse_float: ObjectRef,
    pub(super) is_nan: ObjectRef,
    pub(super) is_finite: ObjectRef,
    pub(super) encode_uri: ObjectRef,
    pub(super) encode_uri_component: ObjectRef,
    pub(super) decode_uri: ObjectRef,
    pub(super) decode_uri_component: ObjectRef,
    pub(super) escape: ObjectRef,
    pub(super) unescape: ObjectRef,
}
