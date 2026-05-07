use super::super::{builtin_id, BuiltinFunctionId};

#[inline]
pub const fn internal_import_meta_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_IMPORT_META_RAW)
}

#[inline]
pub const fn internal_dynamic_import_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_DYNAMIC_IMPORT_RAW)
}

#[inline]
pub const fn internal_finalization_registry_cleanup_job_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_FINALIZATION_REGISTRY_CLEANUP_JOB_RAW)
}

#[inline]
pub const fn internal_direct_eval_builtin() -> BuiltinFunctionId {
    builtin_id(super::super::INTERNAL_DIRECT_EVAL_RAW)
}
