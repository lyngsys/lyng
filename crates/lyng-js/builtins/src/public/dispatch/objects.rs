use super::PublicBuiltinDispatchContext;
use crate::BuiltinInvocation;
use lyng_js_types::{BuiltinFunctionId, Value};

pub(super) fn dispatch_object_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_object_constructor_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_object_prototype_builtin(context, entry, invocation)
}

fn dispatch_object_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_object_builtin() {
        return super::object_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_create_builtin() {
        return super::object_create_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_get_prototype_of_builtin() {
        return super::object_get_prototype_of_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_set_prototype_of_builtin() {
        return super::object_set_prototype_of_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_get_own_property_descriptor_builtin() {
        return super::object_get_own_property_descriptor_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_get_own_property_descriptors_builtin() {
        return super::object_get_own_property_descriptors_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_get_own_property_names_builtin() {
        return super::object_get_own_property_names_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_get_own_property_symbols_builtin() {
        return super::object_get_own_property_symbols_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_define_properties_builtin() {
        return super::object_define_properties_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_define_property_builtin() {
        return super::object_define_property_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_assign_builtin() {
        return super::object_assign_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_from_entries_builtin() {
        return super::object_from_entries_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_group_by_builtin() {
        return super::object_group_by_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_prevent_extensions_builtin() {
        return super::object_prevent_extensions_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_is_extensible_builtin() {
        return super::object_is_extensible_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_is_builtin() {
        return super::object_is_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_seal_builtin() {
        return super::object_seal_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_freeze_builtin() {
        return super::object_freeze_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_is_sealed_builtin() {
        return super::object_is_sealed_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_is_frozen_builtin() {
        return super::object_is_frozen_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_keys_builtin() {
        return super::object_keys_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_entries_builtin() {
        return super::object_entries_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_values_builtin() {
        return super::object_values_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_has_own_builtin() {
        return super::object_has_own_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_object_prototype_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_object_to_locale_string_builtin() {
        return super::object_to_locale_string_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_to_string_builtin() {
        return super::object_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_value_of_builtin() {
        return super::object_value_of_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_has_own_property_builtin() {
        return super::object_has_own_property_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_is_prototype_of_builtin() {
        return super::object_is_prototype_of_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_property_is_enumerable_builtin() {
        return super::object_property_is_enumerable_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_define_getter_builtin() {
        return super::object_define_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_define_setter_builtin() {
        return super::object_define_setter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_lookup_getter_builtin() {
        return super::object_lookup_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_lookup_setter_builtin() {
        return super::object_lookup_setter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_proto_getter_builtin() {
        return super::object_proto_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_object_proto_setter_builtin() {
        return super::object_proto_setter_builtin(context, invocation).map(Some);
    }
    Ok(None)
}
