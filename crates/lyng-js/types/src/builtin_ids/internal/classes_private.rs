use super::super::*;

#[inline]
pub fn internal_define_class_getter_property_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_DEFINE_CLASS_GETTER_PROPERTY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_define_class_setter_property_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_DEFINE_CLASS_SETTER_PROPERTY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_define_private_field_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_DEFINE_PRIVATE_FIELD_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_private_field_init_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_PRIVATE_FIELD_INIT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_private_field_get_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_PRIVATE_FIELD_GET_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_private_field_set_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_PRIVATE_FIELD_SET_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_private_has_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_PRIVATE_HAS_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_bind_function_private_env_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_BIND_FUNCTION_PRIVATE_ENV_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_install_instance_field_key_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_INSTALL_INSTANCE_FIELD_KEY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_get_instance_field_key_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_GET_INSTANCE_FIELD_KEY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}
