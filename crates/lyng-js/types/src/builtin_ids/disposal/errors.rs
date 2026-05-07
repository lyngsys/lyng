use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn suppressed_error_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::SUPPRESSED_ERROR_RAW)
}
