use super::super::*;

#[inline]
pub fn internal_import_meta_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_IMPORT_META_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_dynamic_import_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_DYNAMIC_IMPORT_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_finalization_registry_cleanup_job_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_FINALIZATION_REGISTRY_CLEANUP_JOB_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}

#[inline]
pub fn internal_direct_eval_builtin() -> BuiltinFunctionId {
    match BuiltinFunctionId::from_raw(INTERNAL_DIRECT_EVAL_RAW) {
        Some(id) => id,
        None => unreachable!("builtin id should stay non-zero"),
    }
}
