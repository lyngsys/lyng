use lyng_js_common::AtomId;
use lyng_js_gc::{PrimitiveTracer, TraceHeapEdges};
use lyng_js_host::{ImportMetaProperties, ModuleImportAttribute, ModuleKey};
use lyng_js_types::{CodeRef, EnvironmentRef, ObjectRef, Value};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleRequestRecord {
    specifier: Box<str>,
    attributes: Vec<ModuleImportAttribute>,
    resolved_key: Option<ModuleKey>,
}

impl ModuleRequestRecord {
    #[inline]
    pub fn new(specifier: impl Into<Box<str>>, attributes: Vec<ModuleImportAttribute>) -> Self {
        Self {
            specifier: specifier.into(),
            attributes,
            resolved_key: None,
        }
    }

    #[inline]
    pub fn specifier(&self) -> &str {
        &self.specifier
    }

    #[inline]
    pub fn attributes(&self) -> &[ModuleImportAttribute] {
        &self.attributes
    }

    #[inline]
    pub fn resolved_key(&self) -> Option<&ModuleKey> {
        self.resolved_key.as_ref()
    }

    #[inline]
    pub fn set_resolved_key(&mut self, resolved_key: Option<ModuleKey>) {
        self.resolved_key = resolved_key;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleImportKind {
    Named(AtomId),
    NamespaceObject,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleImportEntry {
    request_index: u32,
    local_name: AtomId,
    local_slot: u32,
    import_kind: ModuleImportKind,
}

impl ModuleImportEntry {
    #[inline]
    pub const fn new(
        request_index: u32,
        local_name: AtomId,
        local_slot: u32,
        import_kind: ModuleImportKind,
    ) -> Self {
        Self {
            request_index,
            local_name,
            local_slot,
            import_kind,
        }
    }

    #[inline]
    pub const fn request_index(self) -> u32 {
        self.request_index
    }

    #[inline]
    pub const fn local_name(self) -> AtomId {
        self.local_name
    }

    #[inline]
    pub const fn local_slot(self) -> u32 {
        self.local_slot
    }

    #[inline]
    pub const fn import_kind(self) -> ModuleImportKind {
        self.import_kind
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleLocalExportEntry {
    export_name: AtomId,
    local_name: Option<AtomId>,
    local_slot: u32,
}

impl ModuleLocalExportEntry {
    #[inline]
    pub const fn new(export_name: AtomId, local_name: Option<AtomId>, local_slot: u32) -> Self {
        Self {
            export_name,
            local_name,
            local_slot,
        }
    }

    #[inline]
    pub const fn export_name(self) -> AtomId {
        self.export_name
    }

    #[inline]
    pub const fn local_name(self) -> Option<AtomId> {
        self.local_name
    }

    #[inline]
    pub const fn local_slot(self) -> u32 {
        self.local_slot
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleIndirectExportEntry {
    export_name: AtomId,
    request_index: u32,
    import_kind: ModuleImportKind,
}

impl ModuleIndirectExportEntry {
    #[inline]
    pub const fn new(
        export_name: AtomId,
        request_index: u32,
        import_kind: ModuleImportKind,
    ) -> Self {
        Self {
            export_name,
            request_index,
            import_kind,
        }
    }

    #[inline]
    pub const fn export_name(self) -> AtomId {
        self.export_name
    }

    #[inline]
    pub const fn request_index(self) -> u32 {
        self.request_index
    }

    #[inline]
    pub const fn import_kind(self) -> ModuleImportKind {
        self.import_kind
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleStarExportEntry {
    request_index: u32,
}

impl ModuleStarExportEntry {
    #[inline]
    pub const fn new(request_index: u32) -> Self {
        Self { request_index }
    }

    #[inline]
    pub const fn request_index(self) -> u32 {
        self.request_index
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleResolvedExportTarget {
    Binding {
        environment: EnvironmentRef,
        slot: u32,
    },
    Value(Value),
}

impl ModuleResolvedExportTarget {
    #[inline]
    pub const fn environment(self) -> Option<EnvironmentRef> {
        match self {
            Self::Binding { environment, .. } => Some(environment),
            Self::Value(_) => None,
        }
    }

    #[inline]
    pub const fn slot(self) -> Option<u32> {
        match self {
            Self::Binding { slot, .. } => Some(slot),
            Self::Value(_) => None,
        }
    }

    #[inline]
    pub const fn value(self) -> Option<Value> {
        match self {
            Self::Binding { .. } => None,
            Self::Value(value) => Some(value),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleResolvedExport {
    export_name: AtomId,
    target: ModuleResolvedExportTarget,
}

impl ModuleResolvedExport {
    #[inline]
    pub const fn new(export_name: AtomId, target: ModuleResolvedExportTarget) -> Self {
        Self {
            export_name,
            target,
        }
    }

    #[inline]
    pub const fn export_name(self) -> AtomId {
        self.export_name
    }

    #[inline]
    pub const fn target(self) -> ModuleResolvedExportTarget {
        self.target
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleStatus {
    New,
    Unlinked,
    Linking,
    Linked,
    Evaluating,
    Evaluated,
    Errored,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleRecord {
    key: ModuleKey,
    display_name: Box<str>,
    requested_modules: Vec<ModuleRequestRecord>,
    import_entries: Vec<ModuleImportEntry>,
    local_exports: Vec<ModuleLocalExportEntry>,
    indirect_exports: Vec<ModuleIndirectExportEntry>,
    star_exports: Vec<ModuleStarExportEntry>,
    code: Option<CodeRef>,
    environment: Option<EnvironmentRef>,
    namespace: Option<ObjectRef>,
    import_meta_object: Option<ObjectRef>,
    import_meta_properties: Option<ImportMetaProperties>,
    resolved_exports: Vec<ModuleResolvedExport>,
    status: ModuleStatus,
    evaluation_error: Option<Value>,
    dfs_index: Option<u32>,
    dfs_ancestor_index: Option<u32>,
}

impl ModuleRecord {
    #[inline]
    pub fn new(
        key: ModuleKey,
        display_name: impl Into<Box<str>>,
        requested_modules: Vec<ModuleRequestRecord>,
        import_entries: Vec<ModuleImportEntry>,
        local_exports: Vec<ModuleLocalExportEntry>,
        indirect_exports: Vec<ModuleIndirectExportEntry>,
        star_exports: Vec<ModuleStarExportEntry>,
    ) -> Self {
        Self {
            key,
            display_name: display_name.into(),
            requested_modules,
            import_entries,
            local_exports,
            indirect_exports,
            star_exports,
            code: None,
            environment: None,
            namespace: None,
            import_meta_object: None,
            import_meta_properties: None,
            resolved_exports: Vec::new(),
            status: ModuleStatus::New,
            evaluation_error: None,
            dfs_index: None,
            dfs_ancestor_index: None,
        }
    }

    #[inline]
    pub fn key(&self) -> &ModuleKey {
        &self.key
    }

    #[inline]
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    #[inline]
    pub fn requested_modules(&self) -> &[ModuleRequestRecord] {
        &self.requested_modules
    }

    #[inline]
    pub fn import_entries(&self) -> &[ModuleImportEntry] {
        &self.import_entries
    }

    #[inline]
    pub fn local_exports(&self) -> &[ModuleLocalExportEntry] {
        &self.local_exports
    }

    #[inline]
    pub fn indirect_exports(&self) -> &[ModuleIndirectExportEntry] {
        &self.indirect_exports
    }

    #[inline]
    pub fn star_exports(&self) -> &[ModuleStarExportEntry] {
        &self.star_exports
    }

    #[inline]
    pub const fn code(&self) -> Option<CodeRef> {
        self.code
    }

    #[inline]
    pub const fn environment(&self) -> Option<EnvironmentRef> {
        self.environment
    }

    #[inline]
    pub const fn namespace(&self) -> Option<ObjectRef> {
        self.namespace
    }

    #[inline]
    pub const fn import_meta_object(&self) -> Option<ObjectRef> {
        self.import_meta_object
    }

    #[inline]
    pub fn import_meta_properties(&self) -> Option<&ImportMetaProperties> {
        self.import_meta_properties.as_ref()
    }

    #[inline]
    pub fn resolved_exports(&self) -> &[ModuleResolvedExport] {
        &self.resolved_exports
    }

    #[inline]
    pub fn resolved_export(&self, export_name: AtomId) -> Option<ModuleResolvedExport> {
        self.resolved_exports
            .iter()
            .copied()
            .find(|entry| entry.export_name() == export_name)
    }

    #[inline]
    pub const fn status(&self) -> ModuleStatus {
        self.status
    }

    #[inline]
    pub const fn evaluation_error(&self) -> Option<Value> {
        self.evaluation_error
    }

    #[inline]
    pub const fn dfs_index(&self) -> Option<u32> {
        self.dfs_index
    }

    #[inline]
    pub const fn dfs_ancestor_index(&self) -> Option<u32> {
        self.dfs_ancestor_index
    }

    #[inline]
    pub fn set_requested_module_resolved_key(
        &mut self,
        request_index: u32,
        resolved_key: Option<ModuleKey>,
    ) -> bool {
        let Some(request) = self.requested_modules.get_mut(request_index as usize) else {
            return false;
        };
        request.set_resolved_key(resolved_key);
        true
    }

    #[inline]
    pub fn set_code(&mut self, code: Option<CodeRef>) {
        self.code = code;
    }

    #[inline]
    pub fn set_environment(&mut self, environment: Option<EnvironmentRef>) {
        self.environment = environment;
    }

    #[inline]
    pub fn set_namespace(&mut self, namespace: Option<ObjectRef>) {
        self.namespace = namespace;
    }

    #[inline]
    pub fn set_import_meta_object(&mut self, import_meta_object: Option<ObjectRef>) {
        self.import_meta_object = import_meta_object;
    }

    #[inline]
    pub fn set_import_meta_properties(&mut self, import_meta_properties: ImportMetaProperties) {
        self.import_meta_properties = Some(import_meta_properties);
    }

    #[inline]
    pub fn set_resolved_exports(&mut self, resolved_exports: Vec<ModuleResolvedExport>) {
        self.resolved_exports = resolved_exports;
    }

    #[inline]
    pub fn set_status(&mut self, status: ModuleStatus) {
        self.status = status;
    }

    #[inline]
    pub fn set_evaluation_error(&mut self, evaluation_error: Option<Value>) {
        self.evaluation_error = evaluation_error;
    }

    #[inline]
    pub fn set_dfs_state(&mut self, dfs_index: Option<u32>, dfs_ancestor_index: Option<u32>) {
        self.dfs_index = dfs_index;
        self.dfs_ancestor_index = dfs_ancestor_index;
    }
}

impl TraceHeapEdges for ModuleResolvedExportTarget {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        match self {
            Self::Binding { environment, .. } => environment.trace_heap_edges(tracer),
            Self::Value(value) => value.trace_heap_edges(tracer),
        }
    }
}

impl TraceHeapEdges for ModuleResolvedExport {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.target.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for ModuleRecord {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.code.trace_heap_edges(tracer);
        self.environment.trace_heap_edges(tracer);
        self.namespace.trace_heap_edges(tracer);
        self.import_meta_object.trace_heap_edges(tracer);
        self.evaluation_error.trace_heap_edges(tracer);
        for export in &self.resolved_exports {
            export.trace_heap_edges(tracer);
        }
    }
}
