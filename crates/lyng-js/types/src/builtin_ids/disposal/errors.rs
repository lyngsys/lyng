use super::super::*;

#[inline]
pub fn suppressed_error_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(SUPPRESSED_ERROR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}
