use super::{ArrayIterationKind, PublicBuiltinDispatchContext};
use crate::BuiltinInvocation;
use lyng_js_types::{BuiltinFunctionId, Value};

pub(super) fn dispatch_collection_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_collection_constructor_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_map_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if let Some(result) = dispatch_set_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    dispatch_weak_collection_builtin(context, entry, invocation)
}

fn dispatch_collection_constructor_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_map_builtin() {
        return super::map_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_set_builtin() {
        return super::set_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_weak_map_builtin() {
        return super::weak_map_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_weak_set_builtin() {
        return super::weak_set_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_weak_ref_builtin() {
        return super::weak_ref_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_finalization_registry_builtin() {
        return super::finalization_registry_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_map_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_map_get_builtin() {
        return super::map_get_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_map_set_builtin() {
        return super::map_set_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_map_has_builtin() {
        return super::map_has_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_map_delete_builtin() {
        return super::map_delete_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_map_clear_builtin() {
        return super::map_clear_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_map_entries_builtin() {
        return super::map_iterator_factory_builtin(context, invocation, ArrayIterationKind::Entry)
            .map(Some);
    }
    if entry == super::js3_map_values_builtin() {
        return super::map_iterator_factory_builtin(context, invocation, ArrayIterationKind::Value)
            .map(Some);
    }
    if entry == super::js3_map_keys_builtin() {
        return super::map_iterator_factory_builtin(context, invocation, ArrayIterationKind::Key)
            .map(Some);
    }
    if entry == super::js3_map_for_each_builtin() {
        return super::map_for_each_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_map_size_getter_builtin() {
        return super::map_size_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_map_iterator_next_builtin() {
        return super::map_iterator_next_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_set_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_set_add_builtin() {
        return super::set_add_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_set_has_builtin() {
        return super::set_has_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_set_delete_builtin() {
        return super::set_delete_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_set_clear_builtin() {
        return super::set_clear_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_set_entries_builtin() {
        return super::set_iterator_factory_builtin(context, invocation, ArrayIterationKind::Entry)
            .map(Some);
    }
    if entry == super::js3_set_values_builtin() {
        return super::set_iterator_factory_builtin(context, invocation, ArrayIterationKind::Value)
            .map(Some);
    }
    if entry == super::js3_set_keys_builtin() {
        return super::set_iterator_factory_builtin(context, invocation, ArrayIterationKind::Key)
            .map(Some);
    }
    if entry == super::js3_set_for_each_builtin() {
        return super::set_for_each_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_set_size_getter_builtin() {
        return super::set_size_getter_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_set_iterator_next_builtin() {
        return super::set_iterator_next_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

fn dispatch_weak_collection_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::js3_weak_map_get_builtin() {
        return super::weak_map_get_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_weak_map_set_builtin() {
        return super::weak_map_set_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_weak_map_has_builtin() {
        return super::weak_map_has_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_weak_map_delete_builtin() {
        return super::weak_map_delete_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_weak_set_add_builtin() {
        return super::weak_set_add_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_weak_set_has_builtin() {
        return super::weak_set_has_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_weak_set_delete_builtin() {
        return super::weak_set_delete_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_weak_ref_deref_builtin() {
        return super::weak_ref_deref_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_finalization_registry_register_builtin() {
        return super::finalization_registry_register_builtin(context, invocation).map(Some);
    }
    if entry == super::js3_finalization_registry_unregister_builtin() {
        return super::finalization_registry_unregister_builtin(context, invocation).map(Some);
    }
    Ok(None)
}
