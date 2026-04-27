use super::super::*;

#[inline]
pub fn weak_ref_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(WEAK_REF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn weak_ref_deref_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(WEAK_REF_DEREF_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}
