use crate::public::PublicRealmBuiltins;
use lyng_js_env::{Agent, Intrinsics};
use lyng_js_types::{ObjectRef, RealmRef};

#[derive(Clone, Copy, Debug)]
#[allow(clippy::struct_field_names)]
pub(in crate::public) struct PublicRealmPrototypeHandles {
    pub(in crate::public) array_prototype: ObjectRef,
    pub(in crate::public) map_prototype: ObjectRef,
    pub(in crate::public) map_iterator_prototype: ObjectRef,
    pub(in crate::public) set_prototype: ObjectRef,
    pub(in crate::public) set_iterator_prototype: ObjectRef,
    pub(in crate::public) weak_map_prototype: ObjectRef,
    pub(in crate::public) weak_set_prototype: ObjectRef,
    pub(in crate::public) weak_ref_prototype: ObjectRef,
    pub(in crate::public) finalization_registry_prototype: ObjectRef,
    pub(in crate::public) array_buffer_prototype: ObjectRef,
    pub(in crate::public) shared_array_buffer_prototype: ObjectRef,
    pub(in crate::public) data_view_prototype: ObjectRef,
    pub(in crate::public) typed_array_prototype: ObjectRef,
    pub(in crate::public) int8_array_prototype: ObjectRef,
    pub(in crate::public) int16_array_prototype: ObjectRef,
    pub(in crate::public) int32_array_prototype: ObjectRef,
    pub(in crate::public) float32_array_prototype: ObjectRef,
    pub(in crate::public) float64_array_prototype: ObjectRef,
    pub(in crate::public) big_int64_array_prototype: ObjectRef,
    pub(in crate::public) big_uint64_array_prototype: ObjectRef,
    pub(in crate::public) uint32_array_prototype: ObjectRef,
    pub(in crate::public) uint16_array_prototype: ObjectRef,
    pub(in crate::public) uint8_clamped_array_prototype: ObjectRef,
    pub(in crate::public) uint8_array_prototype: ObjectRef,
    pub(in crate::public) iterator_prototype: ObjectRef,
    pub(in crate::public) async_from_sync_iterator_prototype: ObjectRef,
    pub(in crate::public) array_iterator_prototype: ObjectRef,
    pub(in crate::public) string_iterator_prototype: ObjectRef,
}

pub(in crate::public) fn install_public_realm_intrinsics(
    agent: &mut Agent,
    realm: RealmRef,
    existing: &Intrinsics,
    builtins: &PublicRealmBuiltins,
    prototypes: &PublicRealmPrototypeHandles,
) -> bool {
    let intrinsics = with_core_intrinsics(existing, builtins, prototypes);
    let intrinsics = with_collection_intrinsics(&intrinsics, builtins, prototypes);
    let intrinsics = with_binary_data_intrinsics(&intrinsics, builtins, prototypes);
    let intrinsics = with_namespace_intrinsics(&intrinsics, builtins, prototypes);
    let intrinsics = with_error_and_promise_intrinsics(&intrinsics, builtins);
    agent.set_realm_intrinsics(realm, intrinsics)
}

fn with_core_intrinsics(
    intrinsics: &Intrinsics,
    builtins: &PublicRealmBuiltins,
    prototypes: &PublicRealmPrototypeHandles,
) -> Intrinsics {
    (*intrinsics)
        .with_object(Some(builtins.object))
        .with_object_prototype(Some(builtins.object_prototype))
        .with_function(Some(builtins.function))
        .with_function_prototype(Some(builtins.function_prototype))
        .with_async_function(Some(builtins.async_function))
        .with_async_function_prototype(Some(builtins.async_function_prototype))
        .with_async_generator_function(Some(builtins.async_generator_function))
        .with_async_generator_function_prototype(Some(builtins.async_generator_function_prototype))
        .with_async_generator_prototype(Some(builtins.async_generator_prototype))
        .with_generator_function(Some(builtins.generator_function))
        .with_generator_function_prototype(Some(builtins.generator_function_prototype))
        .with_generator_prototype(Some(builtins.generator_prototype))
        .with_array(Some(builtins.array))
        .with_array_prototype(Some(prototypes.array_prototype))
        .with_iterator_prototype(Some(prototypes.iterator_prototype))
        .with_async_iterator_prototype(Some(builtins.async_iterator_prototype))
        .with_async_from_sync_iterator_prototype(Some(
            prototypes.async_from_sync_iterator_prototype,
        ))
        .with_array_iterator_prototype(Some(prototypes.array_iterator_prototype))
}

fn with_collection_intrinsics(
    intrinsics: &Intrinsics,
    builtins: &PublicRealmBuiltins,
    prototypes: &PublicRealmPrototypeHandles,
) -> Intrinsics {
    (*intrinsics)
        .with_map(Some(builtins.map))
        .with_map_prototype(Some(prototypes.map_prototype))
        .with_map_iterator_prototype(Some(prototypes.map_iterator_prototype))
        .with_set(Some(builtins.set))
        .with_set_prototype(Some(prototypes.set_prototype))
        .with_set_iterator_prototype(Some(prototypes.set_iterator_prototype))
        .with_weak_map(Some(builtins.weak_map))
        .with_weak_map_prototype(Some(prototypes.weak_map_prototype))
        .with_weak_set(Some(builtins.weak_set))
        .with_weak_set_prototype(Some(prototypes.weak_set_prototype))
        .with_weak_ref(Some(builtins.weak_ref))
        .with_weak_ref_prototype(Some(prototypes.weak_ref_prototype))
        .with_finalization_registry(Some(builtins.finalization_registry))
        .with_finalization_registry_prototype(Some(prototypes.finalization_registry_prototype))
}

fn with_binary_data_intrinsics(
    intrinsics: &Intrinsics,
    builtins: &PublicRealmBuiltins,
    prototypes: &PublicRealmPrototypeHandles,
) -> Intrinsics {
    (*intrinsics)
        .with_array_buffer(Some(builtins.array_buffer))
        .with_array_buffer_prototype(Some(prototypes.array_buffer_prototype))
        .with_shared_array_buffer(Some(builtins.shared_array_buffer))
        .with_shared_array_buffer_prototype(Some(prototypes.shared_array_buffer_prototype))
        .with_data_view(Some(builtins.data_view))
        .with_data_view_prototype(Some(prototypes.data_view_prototype))
        .with_atomics(Some(builtins.atomics))
        .with_typed_array(Some(builtins.typed_array))
        .with_typed_array_prototype(Some(prototypes.typed_array_prototype))
        .with_int8_array(Some(builtins.int8_array))
        .with_int8_array_prototype(Some(prototypes.int8_array_prototype))
        .with_int16_array(Some(builtins.int16_array))
        .with_int16_array_prototype(Some(prototypes.int16_array_prototype))
        .with_int32_array(Some(builtins.int32_array))
        .with_int32_array_prototype(Some(prototypes.int32_array_prototype))
        .with_float32_array(Some(builtins.float32_array))
        .with_float32_array_prototype(Some(prototypes.float32_array_prototype))
        .with_float64_array(Some(builtins.float64_array))
        .with_float64_array_prototype(Some(prototypes.float64_array_prototype))
        .with_big_int64_array(Some(builtins.big_int64_array))
        .with_big_int64_array_prototype(Some(prototypes.big_int64_array_prototype))
        .with_big_uint64_array(Some(builtins.big_uint64_array))
        .with_big_uint64_array_prototype(Some(prototypes.big_uint64_array_prototype))
        .with_uint32_array(Some(builtins.uint32_array))
        .with_uint32_array_prototype(Some(prototypes.uint32_array_prototype))
        .with_uint16_array(Some(builtins.uint16_array))
        .with_uint16_array_prototype(Some(prototypes.uint16_array_prototype))
        .with_uint8_clamped_array(Some(builtins.uint8_clamped_array))
        .with_uint8_clamped_array_prototype(Some(prototypes.uint8_clamped_array_prototype))
        .with_uint8_array(Some(builtins.uint8_array))
        .with_uint8_array_prototype(Some(prototypes.uint8_array_prototype))
}

fn with_namespace_intrinsics(
    intrinsics: &Intrinsics,
    builtins: &PublicRealmBuiltins,
    prototypes: &PublicRealmPrototypeHandles,
) -> Intrinsics {
    (*intrinsics)
        .with_string(Some(builtins.string))
        .with_string_prototype(Some(builtins.string_prototype))
        .with_string_iterator_prototype(Some(prototypes.string_iterator_prototype))
        .with_regexp(Some(builtins.regexp))
        .with_regexp_prototype(Some(builtins.regexp_prototype))
        .with_date(Some(builtins.date))
        .with_date_prototype(Some(builtins.date_prototype))
        .with_number(Some(builtins.number))
        .with_number_prototype(Some(builtins.number_prototype))
        .with_math(Some(builtins.math))
        .with_bigint(Some(builtins.bigint))
        .with_bigint_prototype(Some(builtins.bigint_prototype))
        .with_boolean(Some(builtins.boolean))
        .with_boolean_prototype(Some(builtins.boolean_prototype))
        .with_symbol(Some(builtins.symbol))
        .with_symbol_prototype(Some(builtins.symbol_prototype))
        .with_json(Some(builtins.json))
        .with_reflect(Some(builtins.reflect))
        .with_proxy(Some(builtins.proxy))
}

fn with_error_and_promise_intrinsics(
    intrinsics: &Intrinsics,
    builtins: &PublicRealmBuiltins,
) -> Intrinsics {
    (*intrinsics)
        .with_error(Some(builtins.error))
        .with_error_prototype(Some(builtins.error_prototype))
        .with_eval_error(Some(builtins.eval_error))
        .with_eval_error_prototype(Some(builtins.eval_error_prototype))
        .with_range_error(Some(builtins.range_error))
        .with_range_error_prototype(Some(builtins.range_error_prototype))
        .with_reference_error(Some(builtins.reference_error))
        .with_reference_error_prototype(Some(builtins.reference_error_prototype))
        .with_syntax_error(Some(builtins.syntax_error))
        .with_syntax_error_prototype(Some(builtins.syntax_error_prototype))
        .with_type_error(Some(builtins.type_error))
        .with_type_error_prototype(Some(builtins.type_error_prototype))
        .with_uri_error(Some(builtins.uri_error))
        .with_uri_error_prototype(Some(builtins.uri_error_prototype))
        .with_aggregate_error(Some(builtins.aggregate_error))
        .with_aggregate_error_prototype(Some(builtins.aggregate_error_prototype))
        .with_suppressed_error(Some(builtins.suppressed_error))
        .with_suppressed_error_prototype(Some(builtins.suppressed_error_prototype))
        .with_promise(Some(builtins.promise))
        .with_promise_prototype(Some(builtins.promise_prototype))
        .with_disposable_stack(Some(builtins.disposable_stack))
        .with_disposable_stack_prototype(Some(builtins.disposable_stack_prototype))
        .with_async_disposable_stack(Some(builtins.async_disposable_stack))
        .with_async_disposable_stack_prototype(Some(builtins.async_disposable_stack_prototype))
}
