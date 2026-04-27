use super::super::*;

#[inline]
pub fn aggregate_error_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(AGGREGATE_ERROR_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}
