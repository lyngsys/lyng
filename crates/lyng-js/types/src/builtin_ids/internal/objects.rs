use super::super::*;

#[inline]
pub fn internal_instance_of_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_INSTANCE_OF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_define_getter_property_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_DEFINE_GETTER_PROPERTY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_define_setter_property_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_DEFINE_SETTER_PROPERTY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_object_has_own_property_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_OBJECT_HAS_OWN_PROPERTY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_throw_type_error_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_THROW_TYPE_ERROR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_define_method_property_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_DEFINE_METHOD_PROPERTY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_object_literal_set_prototype_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_OBJECT_LITERAL_SET_PROTOTYPE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}
