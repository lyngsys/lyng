use super::Agent;
use crate::{CodeRef, EnvironmentRef, ModuleRecord, ModuleResolvedExport, ModuleStatus};
use lyng_js_host::{ImportMetaProperties, ModuleKey};
use lyng_js_types::{ObjectRef, Value};

impl Agent {
    #[inline]
    pub fn module_record(&self, key: &ModuleKey) -> Option<&ModuleRecord> {
        self.modules.get(key)
    }

    #[inline]
    pub fn install_module_record(&mut self, record: ModuleRecord) -> Option<ModuleRecord> {
        self.modules.insert(record.key().clone(), record)
    }

    pub fn set_module_record_environment(
        &mut self,
        key: &ModuleKey,
        environment: Option<EnvironmentRef>,
    ) -> bool {
        let Some(record) = self.modules.get_mut(key) else {
            return false;
        };
        record.set_environment(environment);
        true
    }

    pub fn set_module_record_code(&mut self, key: &ModuleKey, code: Option<CodeRef>) -> bool {
        let Some(record) = self.modules.get_mut(key) else {
            return false;
        };
        record.set_code(code);
        true
    }

    pub fn set_module_record_namespace(
        &mut self,
        key: &ModuleKey,
        namespace: Option<ObjectRef>,
    ) -> bool {
        let Some(record) = self.modules.get_mut(key) else {
            return false;
        };
        record.set_namespace(namespace);
        true
    }

    pub fn set_module_record_import_meta_object(
        &mut self,
        key: &ModuleKey,
        import_meta_object: Option<ObjectRef>,
    ) -> bool {
        let Some(record) = self.modules.get_mut(key) else {
            return false;
        };
        record.set_import_meta_object(import_meta_object);
        true
    }

    pub fn set_module_record_import_meta_properties(
        &mut self,
        key: &ModuleKey,
        import_meta_properties: ImportMetaProperties,
    ) -> bool {
        let Some(record) = self.modules.get_mut(key) else {
            return false;
        };
        record.set_import_meta_properties(import_meta_properties);
        true
    }

    pub fn set_module_record_resolved_exports(
        &mut self,
        key: &ModuleKey,
        resolved_exports: Vec<ModuleResolvedExport>,
    ) -> bool {
        let Some(record) = self.modules.get_mut(key) else {
            return false;
        };
        record.set_resolved_exports(resolved_exports);
        true
    }

    pub fn set_module_record_status(&mut self, key: &ModuleKey, status: ModuleStatus) -> bool {
        let Some(record) = self.modules.get_mut(key) else {
            return false;
        };
        record.set_status(status);
        true
    }

    pub fn set_module_record_evaluation_error(
        &mut self,
        key: &ModuleKey,
        evaluation_error: Option<Value>,
    ) -> bool {
        let Some(record) = self.modules.get_mut(key) else {
            return false;
        };
        record.set_evaluation_error(evaluation_error);
        true
    }

    pub fn set_module_record_dfs_state(
        &mut self,
        key: &ModuleKey,
        dfs_index: Option<u32>,
        dfs_ancestor_index: Option<u32>,
    ) -> bool {
        let Some(record) = self.modules.get_mut(key) else {
            return false;
        };
        record.set_dfs_state(dfs_index, dfs_ancestor_index);
        true
    }

    pub fn set_module_requested_key(
        &mut self,
        key: &ModuleKey,
        request_index: u32,
        resolved_key: Option<ModuleKey>,
    ) -> bool {
        let Some(record) = self.modules.get_mut(key) else {
            return false;
        };
        record.set_requested_module_resolved_key(request_index, resolved_key)
    }

    pub fn module_key_for_environment(&self, environment: EnvironmentRef) -> Option<ModuleKey> {
        self.modules.iter().find_map(|(key, record)| {
            (record.environment() == Some(environment)).then(|| key.clone())
        })
    }
}
