use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn atomics_load_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ATOMICS_LOAD_RAW)
}

#[inline]
pub const fn atomics_store_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ATOMICS_STORE_RAW)
}

#[inline]
pub const fn atomics_add_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ATOMICS_ADD_RAW)
}

#[inline]
pub const fn atomics_sub_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ATOMICS_SUB_RAW)
}

#[inline]
pub const fn atomics_and_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ATOMICS_AND_RAW)
}

#[inline]
pub const fn atomics_or_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ATOMICS_OR_RAW)
}

#[inline]
pub const fn atomics_xor_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ATOMICS_XOR_RAW)
}

#[inline]
pub const fn atomics_exchange_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ATOMICS_EXCHANGE_RAW)
}

#[inline]
pub const fn atomics_compare_exchange_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ATOMICS_COMPARE_EXCHANGE_RAW)
}

#[inline]
pub const fn atomics_notify_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ATOMICS_NOTIFY_RAW)
}

#[inline]
pub const fn atomics_wait_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ATOMICS_WAIT_RAW)
}

#[inline]
pub const fn atomics_wait_async_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ATOMICS_WAIT_ASYNC_RAW)
}

#[inline]
pub const fn atomics_pause_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ATOMICS_PAUSE_RAW)
}

#[inline]
pub const fn atomics_is_lock_free_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::ATOMICS_IS_LOCK_FREE_RAW)
}
