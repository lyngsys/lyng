use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn proxy_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROXY_RAW)
}

#[inline]
pub const fn proxy_revocable_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROXY_REVOCABLE_RAW)
}

#[inline]
pub const fn proxy_revoke_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::PROXY_REVOKE_RAW)
}
