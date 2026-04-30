use super::super::*;

#[inline]
pub fn internal_regexp_literal_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_REGEXP_LITERAL_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}
