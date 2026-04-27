use crate::proxy;
use lyng_js_gc::AllocationLifetime;
use lyng_js_types::{ObjectRef, PropertyDescriptor, PropertyKey, Value};

mod bigint;
mod conversions;
mod ordinary;
mod primitive_wrappers;
mod private_elements;
mod receiver_payloads;
mod typed_array_indices;

pub use self::bigint::{
    bigint_to_string, integral_number_to_radix_string, primitive_to_bigint, string_to_bigint_value,
};
pub use self::conversions::{
    ordinary_to_primitive, to_number, to_numeric, to_primitive, ToPrimitiveContext, ToPrimitiveHint,
};
pub use self::ordinary::{
    call, construct, ordinary_create_data_property, ordinary_define_property,
    ordinary_delete_property, ordinary_get, ordinary_get_own_property, ordinary_get_prototype_of,
    ordinary_get_with_receiver, ordinary_has_property, ordinary_is_extensible,
    ordinary_own_property_keys, ordinary_prevent_extensions, ordinary_set,
    ordinary_set_prototype_of, ordinary_set_with_receiver, super_base, super_get, super_set,
};
pub use self::primitive_wrappers::{
    allocate_primitive_wrapper_object, require_primitive_wrapper_value, to_object,
    wrap_primitive_value,
};
pub use self::private_elements::{
    define_private_element_layout, define_private_field_layout, install_instance_public_field_key,
    install_private_element_value, instance_public_field_key, private_element_kind,
    private_field_get, private_field_init, private_field_set, private_has,
    private_shared_element_value,
};
pub use self::receiver_payloads::{require_date_value, require_temporal_object};

pub trait ObjectOpsContext: proxy::ProxyTrapContext {}

impl<Cx: proxy::ProxyTrapContext> ObjectOpsContext for Cx {}

/// ECMAScript `HasProperty` over a proxy-aware object operations context.
///
/// # Errors
/// Returns an abrupt completion if ordinary object internal methods fail or a
/// proxy trap fails.
pub fn has_property_in_context<Cx: ObjectOpsContext>(
    context: &mut Cx,
    object: ObjectRef,
    key: PropertyKey,
) -> Result<bool, Cx::Error> {
    proxy::has_property(context, object, key)
}

/// ECMAScript `Get` over a proxy-aware object operations context.
///
/// # Errors
/// Returns an abrupt completion if ordinary object internal methods fail or a
/// proxy trap fails.
pub fn get_in_context<Cx: ObjectOpsContext>(
    context: &mut Cx,
    object: ObjectRef,
    key: PropertyKey,
) -> Result<Value, Cx::Error> {
    get_with_receiver_in_context(context, object, key, Value::from_object_ref(object))
}

/// ECMAScript `Get` over a proxy-aware object operations context with an explicit receiver.
///
/// # Errors
/// Returns an abrupt completion if ordinary object internal methods fail or a
/// proxy trap fails.
pub fn get_with_receiver_in_context<Cx: ObjectOpsContext>(
    context: &mut Cx,
    object: ObjectRef,
    key: PropertyKey,
    receiver: Value,
) -> Result<Value, Cx::Error> {
    proxy::get(context, object, key, receiver)
}

/// ECMAScript `Set` over a proxy-aware object operations context.
///
/// # Errors
/// Returns an abrupt completion if ordinary object internal methods fail or a
/// proxy trap fails.
pub fn set_in_context<Cx: ObjectOpsContext>(
    context: &mut Cx,
    object: ObjectRef,
    key: PropertyKey,
    value: Value,
    lifetime: AllocationLifetime,
) -> Result<bool, Cx::Error> {
    set_with_receiver_in_context(
        context,
        object,
        key,
        value,
        Value::from_object_ref(object),
        lifetime,
    )
}

/// ECMAScript `Set` over a proxy-aware object operations context with an explicit receiver.
///
/// # Errors
/// Returns an abrupt completion if ordinary object internal methods fail or a
/// proxy trap fails.
pub fn set_with_receiver_in_context<Cx: ObjectOpsContext>(
    context: &mut Cx,
    object: ObjectRef,
    key: PropertyKey,
    value: Value,
    receiver: Value,
    lifetime: AllocationLifetime,
) -> Result<bool, Cx::Error> {
    proxy::set(context, object, key, value, receiver, lifetime)
}

/// ECMAScript `GetOwnProperty` over a proxy-aware object operations context.
///
/// # Errors
/// Returns an abrupt completion if ordinary object internal methods fail or a
/// proxy trap fails.
pub fn get_own_property_in_context<Cx: ObjectOpsContext>(
    context: &mut Cx,
    object: ObjectRef,
    key: PropertyKey,
) -> Result<Option<PropertyDescriptor>, Cx::Error> {
    proxy::get_own_property(context, object, key)
}

/// ECMAScript `DefineOwnProperty` over a proxy-aware object operations context.
///
/// # Errors
/// Returns an abrupt completion if ordinary object internal methods fail or a
/// proxy trap fails.
pub fn define_property_in_context<Cx: ObjectOpsContext>(
    context: &mut Cx,
    object: ObjectRef,
    key: PropertyKey,
    descriptor: PropertyDescriptor,
    lifetime: AllocationLifetime,
) -> Result<bool, Cx::Error> {
    proxy::define_property(context, object, key, descriptor, lifetime)
}

/// ECMAScript `DeletePropertyOrThrow`-style primitive over a proxy-aware object operations
/// context.
///
/// # Errors
/// Returns an abrupt completion if ordinary object internal methods fail or a
/// proxy trap fails.
pub fn delete_property_in_context<Cx: ObjectOpsContext>(
    context: &mut Cx,
    object: ObjectRef,
    key: PropertyKey,
) -> Result<bool, Cx::Error> {
    proxy::delete_property(context, object, key)
}

/// ECMAScript `OwnPropertyKeys` over a proxy-aware object operations context.
///
/// # Errors
/// Returns an abrupt completion if ordinary object internal methods fail or a
/// proxy trap fails.
pub fn own_property_keys_in_context<Cx: ObjectOpsContext>(
    context: &mut Cx,
    object: ObjectRef,
) -> Result<Vec<PropertyKey>, Cx::Error> {
    proxy::own_property_keys(context, object)
}

/// ECMAScript `GetPrototypeOf` over a proxy-aware object operations context.
///
/// # Errors
/// Returns an abrupt completion if ordinary object internal methods fail or a
/// proxy trap fails.
pub fn get_prototype_of_in_context<Cx: ObjectOpsContext>(
    context: &mut Cx,
    object: ObjectRef,
) -> Result<Option<ObjectRef>, Cx::Error> {
    proxy::get_prototype_of(context, object)
}

/// ECMAScript `SetPrototypeOf` over a proxy-aware object operations context.
///
/// # Errors
/// Returns an abrupt completion if ordinary object internal methods fail or a
/// proxy trap fails.
pub fn set_prototype_of_in_context<Cx: ObjectOpsContext>(
    context: &mut Cx,
    object: ObjectRef,
    prototype: Option<ObjectRef>,
) -> Result<bool, Cx::Error> {
    proxy::set_prototype_of(context, object, prototype)
}

#[cfg(test)]
mod tests;
