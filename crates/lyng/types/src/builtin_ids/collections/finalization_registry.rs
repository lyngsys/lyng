use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    finalization_registry_builtin => super::super::FINALIZATION_REGISTRY_RAW;
    finalization_registry_register_builtin => super::super::FINALIZATION_REGISTRY_REGISTER_RAW;
    finalization_registry_unregister_builtin => super::super::FINALIZATION_REGISTRY_UNREGISTER_RAW;
}
