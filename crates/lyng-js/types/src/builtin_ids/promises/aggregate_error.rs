use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn aggregate_error_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::AGGREGATE_ERROR_RAW)
}
