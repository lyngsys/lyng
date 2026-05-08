use super::{
    iterator, property_key_from_text, type_error, BuiltinInvocation, PropertyKey,
    PublicBuiltinDispatchContext, Value, WellKnownAtom,
};

pub(super) fn iterator_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    // Step 1: Iterator() called as a function: TypeError.
    let new_target = invocation.new_target().ok_or_else(|| type_error(cx))?;
    // Step 2: new Iterator() with NewTarget == Iterator (no subclass): TypeError.
    if new_target == cx.callee_object() {
        return Err(type_error(cx));
    }
    // Step 3: Subclass — allocate ordinary object with the subclass's prototype
    // chained through %Iterator.prototype%.
    let realm = cx.builtin_realm();
    let default_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().iterator_prototype())
        .ok_or_else(|| type_error(cx))?;
    let prototype =
        cx.ordinary_constructor_prototype(realm, Some(new_target), default_prototype)?;
    let object = cx.allocate_ordinary_object_with_prototype(realm, Some(prototype))?;
    Ok(Value::from_object_ref(object))
}

pub(super) fn iterator_to_string_tag_value<Cx: PublicBuiltinDispatchContext>(cx: &mut Cx) -> Value {
    // The default getter returns the literal string "Iterator". Per the
    // spec, custom subclass setters can override this on a per-instance
    // basis via the brand-checked accessor pair below; the getter only
    // observes the brand-installed override.
    let realm = cx.builtin_realm();
    let agent = cx.agent();
    let intrinsics = agent
        .realm(realm)
        .map(|realm| realm.intrinsics())
        .unwrap_or_default();
    let _ = intrinsics; // suppress unused warning; reserved for future custom-tag logic
    super::string_value(cx, "Iterator")
}

pub(super) fn iterator_to_string_tag_setter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    // SetterThatIgnoresPrototypeProperties: if `this` is the Iterator.prototype
    // itself, throw TypeError. Otherwise, define the property on `this` as a
    // plain data property (or update an existing data property).
    let this_value = invocation.this_value();
    let this_object = this_value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let realm = cx.builtin_realm();
    let iterator_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().iterator_prototype())
        .ok_or_else(|| type_error(cx))?;
    if this_object == iterator_prototype {
        return Err(type_error(cx));
    }
    let new_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let symbol_ref = cx
        .agent()
        .well_known_symbol(lyng_js_types::WellKnownSymbolId::ToStringTag)
        .ok_or_else(|| type_error(cx))?;
    let symbol_key = PropertyKey::from_symbol(symbol_ref);
    super::define_data_property_with_attrs(
        cx,
        this_object,
        symbol_key,
        new_value,
        true,
        true,
        true,
    )?;
    Ok(Value::undefined())
}

pub(super) fn iterator_constructor_getter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    _invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    // The default getter returns %Iterator% (the constructor itself).
    let realm = cx.builtin_realm();
    let iterator = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().iterator())
        .ok_or_else(|| type_error(cx))?;
    Ok(Value::from_object_ref(iterator))
}

pub(super) fn iterator_constructor_setter_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    // Mirror image of iterator_to_string_tag_setter_builtin: refuse to set
    // on Iterator.prototype itself, otherwise install a data property on the
    // receiver.
    let this_value = invocation.this_value();
    let this_object = this_value.as_object_ref().ok_or_else(|| type_error(cx))?;
    let realm = cx.builtin_realm();
    let iterator_prototype = cx
        .agent()
        .realm(realm)
        .and_then(|record| record.intrinsics().iterator_prototype())
        .ok_or_else(|| type_error(cx))?;
    if this_object == iterator_prototype {
        return Err(type_error(cx));
    }
    let new_value = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let key = PropertyKey::from_atom(WellKnownAtom::constructor.id());
    super::define_data_property_with_attrs(cx, this_object, key, new_value, true, true, true)?;
    Ok(Value::undefined())
}

// Helper: build an IteratorRecord from an arbitrary `O` whose `next` is the
// only access we need (GetIteratorDirect from the spec).
pub(super) fn iterator_direct_record<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
) -> Result<iterator::IteratorRecord, Cx::Error> {
    let next_key = property_key_from_text(cx, "next");
    let next_value = cx.get_property_value(Value::from_object_ref(object_ref), next_key)?;
    let next_method = cx.require_callable_object(next_value)?;
    Ok(iterator::IteratorRecord::new(object_ref, next_method))
}

// Helper: call O.return() if it exists, ignoring any errors. Used for the
// argument-validation-failure branch of the eager helpers, where the spec
// asks IteratorClose to run on a record whose [[NextMethod]] hasn't been
// populated yet (so we can't use the regular IteratorRecord-based close).
pub(super) fn iterator_close_for_validation_failure<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    object_ref: lyng_js_types::ObjectRef,
) {
    let return_key = property_key_from_text(cx, "return");
    let Ok(return_value) = cx.get_property_value(Value::from_object_ref(object_ref), return_key)
    else {
        return;
    };
    if return_value.is_undefined() || return_value.is_null() {
        return;
    }
    if let Ok(return_method) = cx.require_callable_object(return_value) {
        // Per spec, the original ThrowCompletion is preserved over any
        // completion produced by return(); ignore both Ok and Err here.
        let _ = cx.call_to_completion(return_method, Value::from_object_ref(object_ref), &[]);
    }
}

pub(super) fn iterator_this_object<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<lyng_js_types::ObjectRef, Cx::Error> {
    value.as_object_ref().ok_or_else(|| type_error(cx))
}
