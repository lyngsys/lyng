use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    atomics_load_builtin => super::super::ATOMICS_LOAD_RAW;
    atomics_store_builtin => super::super::ATOMICS_STORE_RAW;
    atomics_add_builtin => super::super::ATOMICS_ADD_RAW;
    atomics_sub_builtin => super::super::ATOMICS_SUB_RAW;
    atomics_and_builtin => super::super::ATOMICS_AND_RAW;
    atomics_or_builtin => super::super::ATOMICS_OR_RAW;
    atomics_xor_builtin => super::super::ATOMICS_XOR_RAW;
    atomics_exchange_builtin => super::super::ATOMICS_EXCHANGE_RAW;
    atomics_compare_exchange_builtin => super::super::ATOMICS_COMPARE_EXCHANGE_RAW;
    atomics_notify_builtin => super::super::ATOMICS_NOTIFY_RAW;
    atomics_wait_builtin => super::super::ATOMICS_WAIT_RAW;
    atomics_wait_async_builtin => super::super::ATOMICS_WAIT_ASYNC_RAW;
    atomics_pause_builtin => super::super::ATOMICS_PAUSE_RAW;
    atomics_is_lock_free_builtin => super::super::ATOMICS_IS_LOCK_FREE_RAW;
}
