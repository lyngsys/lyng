use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn internal_regexp_literal_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_REGEXP_LITERAL_RAW)
}
