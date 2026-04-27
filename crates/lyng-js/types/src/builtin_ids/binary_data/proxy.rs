use super::super::*;

#[inline]
pub fn proxy_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROXY_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn proxy_revocable_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROXY_REVOCABLE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn proxy_revoke_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(PROXY_REVOKE_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}
