use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn weak_ref_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::WEAK_REF_RAW)
}

#[inline]
pub const fn weak_ref_deref_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::WEAK_REF_DEREF_RAW)
}
