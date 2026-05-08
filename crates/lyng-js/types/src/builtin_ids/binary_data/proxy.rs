use super::super::{builtin_id, BuiltinFunctionId};

builtin_id_accessors! {
    proxy_builtin => super::super::PROXY_RAW;
    proxy_revocable_builtin => super::super::PROXY_REVOCABLE_RAW;
    proxy_revoke_builtin => super::super::PROXY_REVOKE_RAW;
}
