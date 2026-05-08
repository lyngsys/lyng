use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    internal_import_meta_builtin => super::super::INTERNAL_IMPORT_META_RAW;
    internal_dynamic_import_builtin => super::super::INTERNAL_DYNAMIC_IMPORT_RAW;
    internal_finalization_registry_cleanup_job_builtin => super::super::INTERNAL_FINALIZATION_REGISTRY_CLEANUP_JOB_RAW;
    internal_direct_eval_builtin => super::super::INTERNAL_DIRECT_EVAL_RAW;
}
