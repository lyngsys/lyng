use crate::script::{CompilationState, ProgramRootKind, ProgramSource};
use crate::LoweringResult;
use lyng_js_ast::{
    Decl, ExportDefaultDecl, ExportKind, ImportAttribute, ImportSpecifier, ParsedModule, Pattern,
    Stmt,
};
use lyng_js_bytecode::{BytecodeFunction, BytecodeFunctionId, CompiledAtom};
use lyng_js_common::{AtomId, AtomTable, SourceId, Span, WellKnownAtom};
use lyng_js_host::ModuleImportAttribute;
use lyng_js_sema::{ModuleSema, ScopeId, SemanticBindingId};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleImportKind {
    Named(AtomId),
    NamespaceObject,
    Source,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleRequestPhase {
    Evaluation,
    Source,
    Defer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestedModule {
    specifier: Box<str>,
    attributes: Vec<ModuleImportAttribute>,
    phase: ModuleRequestPhase,
}

impl RequestedModule {
    #[inline]
    pub fn new(
        specifier: impl Into<Box<str>>,
        attributes: Vec<ModuleImportAttribute>,
        phase: ModuleRequestPhase,
    ) -> Self {
        Self {
            specifier: specifier.into(),
            attributes,
            phase,
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
    pub const fn phase(&self) -> ModuleRequestPhase {
        self.phase
    }
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
pub struct LocalExportEntry {
    export_name: AtomId,
    local_name: Option<AtomId>,
    local_slot: u32,
}

impl LocalExportEntry {
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
pub struct IndirectExportEntry {
    export_name: AtomId,
    request_index: u32,
    import_kind: ModuleImportKind,
}

impl IndirectExportEntry {
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
pub struct StarExportEntry {
    request_index: u32,
}

impl StarExportEntry {
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
pub struct DynamicImportSite {
    span: Span,
}

impl DynamicImportSite {
    #[inline]
    pub const fn new(span: Span) -> Self {
        Self { span }
    }

    #[inline]
    pub const fn span(self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledModuleUnit {
    source: SourceId,
    entry: BytecodeFunctionId,
    functions: Vec<BytecodeFunction>,
    atoms: Vec<(AtomId, CompiledAtom)>,
    source_text: Option<Box<str>>,
    requested_modules: Vec<RequestedModule>,
    import_entries: Vec<ModuleImportEntry>,
    local_exports: Vec<LocalExportEntry>,
    indirect_exports: Vec<IndirectExportEntry>,
    star_exports: Vec<StarExportEntry>,
    has_import_meta: bool,
    dynamic_import_sites: Vec<DynamicImportSite>,
}

impl CompiledModuleUnit {
    #[inline]
    pub const fn new(
        source: SourceId,
        entry: BytecodeFunctionId,
        functions: Vec<BytecodeFunction>,
        requested_modules: Vec<RequestedModule>,
        import_entries: Vec<ModuleImportEntry>,
        local_exports: Vec<LocalExportEntry>,
        indirect_exports: Vec<IndirectExportEntry>,
        star_exports: Vec<StarExportEntry>,
    ) -> Self {
        Self {
            source,
            entry,
            functions,
            atoms: Vec::new(),
            source_text: None,
            requested_modules,
            import_entries,
            local_exports,
            indirect_exports,
            star_exports,
            has_import_meta: false,
            dynamic_import_sites: Vec::new(),
        }
    }

    #[inline]
    pub const fn source(&self) -> SourceId {
        self.source
    }

    #[inline]
    pub const fn entry(&self) -> BytecodeFunctionId {
        self.entry
    }

    #[inline]
    pub fn functions(&self) -> &[BytecodeFunction] {
        &self.functions
    }

    #[inline]
    pub fn function(&self, id: BytecodeFunctionId) -> Option<&BytecodeFunction> {
        self.functions.iter().find(|function| function.id() == id)
    }

    #[inline]
    pub fn atoms(&self) -> &[(AtomId, CompiledAtom)] {
        &self.atoms
    }

    #[inline]
    pub fn source_text(&self) -> Option<&str> {
        self.source_text.as_deref()
    }

    #[inline]
    pub fn atom_text(&self, atom: AtomId) -> Option<&str> {
        self.atoms
            .iter()
            .find_map(|(candidate, text)| (*candidate == atom).then_some(text.as_str()).flatten())
    }

    #[inline]
    pub fn atom_utf16(&self, atom: AtomId) -> Option<&[u16]> {
        self.atoms
            .iter()
            .find_map(|(candidate, text)| (*candidate == atom).then_some(text.as_utf16()).flatten())
    }

    #[inline]
    pub fn requested_modules(&self) -> &[RequestedModule] {
        &self.requested_modules
    }

    #[inline]
    pub fn import_entries(&self) -> &[ModuleImportEntry] {
        &self.import_entries
    }

    #[inline]
    pub fn local_exports(&self) -> &[LocalExportEntry] {
        &self.local_exports
    }

    #[inline]
    pub fn indirect_exports(&self) -> &[IndirectExportEntry] {
        &self.indirect_exports
    }

    #[inline]
    pub fn star_exports(&self) -> &[StarExportEntry] {
        &self.star_exports
    }

    #[inline]
    pub const fn has_import_meta(&self) -> bool {
        self.has_import_meta
    }

    #[inline]
    pub fn dynamic_import_sites(&self) -> &[DynamicImportSite] {
        &self.dynamic_import_sites
    }

    #[inline]
    pub fn with_atoms(mut self, atoms: Vec<(AtomId, CompiledAtom)>) -> Self {
        self.atoms = atoms;
        self
    }

    #[inline]
    pub fn with_source_text(mut self, source_text: impl Into<Box<str>>) -> Self {
        self.source_text = Some(source_text.into());
        self
    }

    #[inline]
    pub const fn with_import_meta(mut self, has_import_meta: bool) -> Self {
        self.has_import_meta = has_import_meta;
        self
    }

    #[inline]
    pub fn with_dynamic_import_sites(
        mut self,
        dynamic_import_sites: Vec<DynamicImportSite>,
    ) -> Self {
        self.dynamic_import_sites = dynamic_import_sites;
        self
    }
}

pub fn compile_module(
    parsed: &ParsedModule,
    sema: &ModuleSema,
    atoms: &mut AtomTable,
) -> LoweringResult<CompiledModuleUnit> {
    let module = parsed.ast.get_module(parsed.root);
    let source = module.span.source;
    let mut compilation = CompilationState::new(
        ProgramSource {
            ast: &parsed.ast,
            body: module.body,
            span: module.span,
            strict: true,
            kind: ProgramRootKind::Module,
        },
        sema.view(),
        atoms,
    )?;
    let entry = compilation.compile_root_entry()?;
    let metadata = derive_module_metadata(parsed, &compilation)?;
    let (functions, mut unit_atoms) = compilation.into_parts();
    append_module_metadata_atoms(&mut unit_atoms, atoms, &metadata);
    Ok(CompiledModuleUnit::new(
        source,
        entry,
        functions,
        metadata.requested_modules,
        metadata.import_entries,
        metadata.local_exports,
        metadata.indirect_exports,
        metadata.star_exports,
    )
    .with_atoms(unit_atoms)
    .with_source_text(parsed.source_text.clone())
    .with_import_meta(metadata.has_import_meta)
    .with_dynamic_import_sites(metadata.dynamic_import_sites))
}

fn append_module_metadata_atoms(
    unit_atoms: &mut Vec<(AtomId, CompiledAtom)>,
    atoms: &AtomTable,
    metadata: &ModuleMetadata,
) {
    let mut seen = unit_atoms
        .iter()
        .map(|(atom, _)| *atom)
        .collect::<HashSet<_>>();
    let mut push_atom = |atom: AtomId| {
        if !seen.insert(atom) {
            return;
        }
        let text = if let Some(text) = atoms.get(atom) {
            CompiledAtom::from(text)
        } else {
            let units = atoms
                .get_utf16(atom)
                .expect("module metadata atom should resolve to UTF-8 or UTF-16 storage");
            CompiledAtom::from(units.to_vec())
        };
        unit_atoms.push((atom, text));
    };

    for entry in &metadata.import_entries {
        push_atom(entry.local_name());
        if let ModuleImportKind::Named(name) = entry.import_kind() {
            push_atom(name);
        }
    }
    for entry in &metadata.local_exports {
        push_atom(entry.export_name());
        if let Some(local_name) = entry.local_name() {
            push_atom(local_name);
        }
    }
    for entry in &metadata.indirect_exports {
        push_atom(entry.export_name());
        if let ModuleImportKind::Named(name) = entry.import_kind() {
            push_atom(name);
        }
    }
}

#[derive(Default)]
struct ModuleMetadata {
    requested_modules: Vec<RequestedModule>,
    import_entries: Vec<ModuleImportEntry>,
    local_exports: Vec<LocalExportEntry>,
    indirect_exports: Vec<IndirectExportEntry>,
    star_exports: Vec<StarExportEntry>,
    has_import_meta: bool,
    dynamic_import_sites: Vec<DynamicImportSite>,
}

fn derive_module_metadata(
    parsed: &ParsedModule,
    compilation: &CompilationState<'_>,
) -> LoweringResult<ModuleMetadata> {
    let module_scope = ScopeId::new(0);
    let mut bindings_by_name = HashMap::new();
    let sema = compilation.sema();
    for (index, binding) in sema.binding_table.as_slice().iter().enumerate() {
        if binding.scope != module_scope {
            continue;
        }
        bindings_by_name.insert(binding.name, SemanticBindingId::new(index as u32));
    }

    let mut metadata = ModuleMetadata::default();
    for &stmt in parsed
        .ast
        .get_stmt_list(parsed.ast.get_module(parsed.root).body)
    {
        if let Stmt::Declaration { decl, .. } = parsed.ast.get_stmt(stmt) {
            derive_decl_metadata(
                &parsed.ast,
                *decl,
                compilation,
                &bindings_by_name,
                &mut metadata,
            )?;
        }
    }
    collect_module_expression_sites(
        &parsed.ast,
        parsed.ast.get_module(parsed.root).body,
        &mut metadata,
    );
    Ok(metadata)
}

fn derive_decl_metadata(
    ast: &lyng_js_ast::Ast,
    decl_id: lyng_js_ast::DeclId,
    compilation: &CompilationState<'_>,
    bindings_by_name: &HashMap<AtomId, SemanticBindingId>,
    metadata: &mut ModuleMetadata,
) -> LoweringResult<()> {
    match ast.get_decl(decl_id) {
        Decl::Import {
            source,
            specifiers,
            attributes,
            ..
        } => {
            let specifiers = ast.get_import_spec_list(*specifiers);
            let request_phase = import_request_phase(specifiers);
            let request_index = push_requested_module(
                ast,
                *source,
                *attributes,
                request_phase,
                compilation,
                metadata,
            );
            for specifier in specifiers {
                match specifier {
                    ImportSpecifier::Default { local, .. } => {
                        metadata.import_entries.push(ModuleImportEntry::new(
                            request_index,
                            *local,
                            local_binding_slot(*local, compilation, bindings_by_name)?,
                            ModuleImportKind::Named(WellKnownAtom::default.id()),
                        ));
                    }
                    ImportSpecifier::Namespace { local, .. } => {
                        metadata.import_entries.push(ModuleImportEntry::new(
                            request_index,
                            *local,
                            local_binding_slot(*local, compilation, bindings_by_name)?,
                            ModuleImportKind::NamespaceObject,
                        ));
                    }
                    ImportSpecifier::Source { local, .. } => {
                        metadata.import_entries.push(ModuleImportEntry::new(
                            request_index,
                            *local,
                            local_binding_slot(*local, compilation, bindings_by_name)?,
                            ModuleImportKind::Source,
                        ));
                    }
                    ImportSpecifier::Named {
                        imported, local, ..
                    } => metadata.import_entries.push(ModuleImportEntry::new(
                        request_index,
                        *local,
                        local_binding_slot(*local, compilation, bindings_by_name)?,
                        ModuleImportKind::Named(*imported),
                    )),
                }
            }
        }
        Decl::Export { kind, .. } => {
            derive_export_metadata(ast, kind, compilation, bindings_by_name, metadata)?;
        }
        _ => {}
    }
    Ok(())
}

fn derive_export_metadata(
    ast: &lyng_js_ast::Ast,
    kind: &ExportKind,
    compilation: &CompilationState<'_>,
    bindings_by_name: &HashMap<AtomId, SemanticBindingId>,
    metadata: &mut ModuleMetadata,
) -> LoweringResult<()> {
    match kind {
        ExportKind::Named {
            specifiers,
            source,
            attributes,
        } => {
            if let Some(source) = source {
                let request_index = push_requested_module(
                    ast,
                    *source,
                    *attributes,
                    ModuleRequestPhase::Evaluation,
                    compilation,
                    metadata,
                );
                for specifier in ast.get_export_spec_list(*specifiers) {
                    metadata.indirect_exports.push(IndirectExportEntry::new(
                        specifier.exported,
                        request_index,
                        ModuleImportKind::Named(specifier.local),
                    ));
                }
            } else {
                for specifier in ast.get_export_spec_list(*specifiers) {
                    metadata.local_exports.push(LocalExportEntry::new(
                        specifier.exported,
                        Some(specifier.local),
                        local_binding_slot(specifier.local, compilation, bindings_by_name)?,
                    ));
                }
            }
        }
        ExportKind::Default { declaration } => {
            let named_binding = match declaration {
                ExportDefaultDecl::Function(function) => ast.get_function(*function).name,
                ExportDefaultDecl::Class(decl) => match ast.get_decl(*decl) {
                    Decl::Class { name, .. } => *name,
                    _ => None,
                },
                ExportDefaultDecl::Expression(_) => None,
            };
            let (local_name, slot) = if let Some(name) = named_binding {
                (
                    Some(name),
                    local_binding_slot(name, compilation, bindings_by_name)?,
                )
            } else {
                (
                    None,
                    compilation.module_default_export_slot().expect(
                        "module default export slot should exist when lowering one default export",
                    ),
                )
            };
            metadata.local_exports.push(LocalExportEntry::new(
                WellKnownAtom::default.id(),
                local_name,
                slot,
            ));
        }
        ExportKind::All {
            source,
            exported,
            attributes,
        } => {
            let request_index = push_requested_module(
                ast,
                *source,
                *attributes,
                ModuleRequestPhase::Evaluation,
                compilation,
                metadata,
            );
            if let Some(exported) = exported {
                metadata.indirect_exports.push(IndirectExportEntry::new(
                    *exported,
                    request_index,
                    ModuleImportKind::NamespaceObject,
                ));
            } else {
                metadata
                    .star_exports
                    .push(StarExportEntry::new(request_index));
            }
        }
        ExportKind::Declaration { decl } => match ast.get_decl(*decl) {
            Decl::Variable { declarators, .. } => {
                for declarator in ast.get_var_declarator_list(*declarators) {
                    collect_pattern_exports(
                        ast,
                        declarator.id,
                        compilation,
                        bindings_by_name,
                        metadata,
                    )?;
                }
            }
            Decl::Function { function, .. } => {
                if let Some(name) = ast.get_function(*function).name {
                    metadata.local_exports.push(LocalExportEntry::new(
                        name,
                        Some(name),
                        local_binding_slot(name, compilation, bindings_by_name)?,
                    ));
                }
            }
            Decl::Class {
                name: Some(name), ..
            } => {
                metadata.local_exports.push(LocalExportEntry::new(
                    *name,
                    Some(*name),
                    local_binding_slot(*name, compilation, bindings_by_name)?,
                ));
            }
            _ => {}
        },
    }
    Ok(())
}

fn collect_pattern_exports(
    ast: &lyng_js_ast::Ast,
    pattern: lyng_js_ast::PatternId,
    compilation: &CompilationState<'_>,
    bindings_by_name: &HashMap<AtomId, SemanticBindingId>,
    metadata: &mut ModuleMetadata,
) -> LoweringResult<()> {
    match ast.get_pattern(pattern) {
        Pattern::Identifier { name, .. } => {
            metadata.local_exports.push(LocalExportEntry::new(
                *name,
                Some(*name),
                local_binding_slot(*name, compilation, bindings_by_name)?,
            ));
        }
        Pattern::Object {
            properties, rest, ..
        } => {
            for property in ast.get_obj_pattern_prop_list(*properties) {
                collect_pattern_exports(
                    ast,
                    property.value,
                    compilation,
                    bindings_by_name,
                    metadata,
                )?;
            }
            if let Some(rest) = rest {
                collect_pattern_exports(ast, *rest, compilation, bindings_by_name, metadata)?;
            }
        }
        Pattern::Array { elements, rest, .. } => {
            for element in ast.get_opt_pattern_elem_list(*elements).iter().flatten() {
                collect_pattern_exports(
                    ast,
                    element.pattern,
                    compilation,
                    bindings_by_name,
                    metadata,
                )?;
            }
            if let Some(rest) = rest {
                collect_pattern_exports(ast, *rest, compilation, bindings_by_name, metadata)?;
            }
        }
        Pattern::Assignment { left, .. } => {
            collect_pattern_exports(ast, *left, compilation, bindings_by_name, metadata)?;
        }
        Pattern::InvalidPattern { .. } => {}
    }
    Ok(())
}

fn local_binding_slot(
    name: AtomId,
    compilation: &CompilationState<'_>,
    bindings_by_name: &HashMap<AtomId, SemanticBindingId>,
) -> LoweringResult<u32> {
    let binding = bindings_by_name
        .get(&name)
        .copied()
        .expect("module export/import names should resolve after sema");
    compilation.runtime_slot_for_binding(binding)
}

fn push_requested_module(
    ast: &lyng_js_ast::Ast,
    source: lyng_js_ast::StringLiteralId,
    attributes: lyng_js_ast::NodeList<ImportAttribute>,
    phase: ModuleRequestPhase,
    compilation: &CompilationState<'_>,
    metadata: &mut ModuleMetadata,
) -> u32 {
    let specifier = ast.literals().get_string(source).to_owned();
    let attributes = ast
        .get_import_attr_list(attributes)
        .iter()
        .map(|attribute| ModuleImportAttribute {
            key: compilation.resolve_atom(attribute.key).to_owned(),
            value: ast.literals().get_string(attribute.value).to_owned(),
        })
        .collect::<Vec<_>>();
    let index = u32::try_from(metadata.requested_modules.len()).unwrap_or(u32::MAX);
    metadata
        .requested_modules
        .push(RequestedModule::new(specifier, attributes, phase));
    index
}

fn import_request_phase(specifiers: &[ImportSpecifier]) -> ModuleRequestPhase {
    if specifiers
        .iter()
        .any(|specifier| matches!(specifier, ImportSpecifier::Source { .. }))
    {
        return ModuleRequestPhase::Source;
    }
    if specifiers
        .iter()
        .any(|specifier| matches!(specifier, ImportSpecifier::Namespace { deferred: true, .. }))
    {
        return ModuleRequestPhase::Defer;
    }
    ModuleRequestPhase::Evaluation
}

fn collect_module_expression_sites(
    ast: &lyng_js_ast::Ast,
    body: lyng_js_ast::NodeList<lyng_js_ast::StmtId>,
    metadata: &mut ModuleMetadata,
) {
    for &stmt in ast.get_stmt_list(body) {
        collect_expression_sites_from_stmt(ast, stmt, metadata);
    }
}

fn collect_expression_sites_from_stmt(
    ast: &lyng_js_ast::Ast,
    stmt_id: lyng_js_ast::StmtId,
    metadata: &mut ModuleMetadata,
) {
    match ast.get_stmt(stmt_id) {
        Stmt::Block { body, .. } => collect_module_expression_sites(ast, *body, metadata),
        Stmt::Expression { expression, .. } => {
            collect_expression_sites_from_expr(ast, *expression, metadata);
        }
        Stmt::If {
            test,
            consequent,
            alternate,
            ..
        } => {
            collect_expression_sites_from_expr(ast, *test, metadata);
            collect_expression_sites_from_stmt(ast, *consequent, metadata);
            if let Some(alternate) = alternate {
                collect_expression_sites_from_stmt(ast, *alternate, metadata);
            }
        }
        Stmt::DoWhile { body, test, .. } | Stmt::While { test, body, .. } => {
            collect_expression_sites_from_stmt(ast, *body, metadata);
            collect_expression_sites_from_expr(ast, *test, metadata);
        }
        Stmt::For {
            init,
            test,
            update,
            body,
            ..
        } => {
            if let Some(init) = init {
                match init {
                    lyng_js_ast::ForInit::Declaration(decl) => {
                        if let Decl::Export { kind, .. } = ast.get_decl(*decl) {
                            let _ = kind;
                        }
                    }
                    lyng_js_ast::ForInit::Expression(expr) => {
                        collect_expression_sites_from_expr(ast, *expr, metadata);
                    }
                }
            }
            if let Some(test) = test {
                collect_expression_sites_from_expr(ast, *test, metadata);
            }
            if let Some(update) = update {
                collect_expression_sites_from_expr(ast, *update, metadata);
            }
            collect_expression_sites_from_stmt(ast, *body, metadata);
        }
        Stmt::ForIn { right, body, .. } | Stmt::ForOf { right, body, .. } => {
            collect_expression_sites_from_expr(ast, *right, metadata);
            collect_expression_sites_from_stmt(ast, *body, metadata);
        }
        Stmt::Return { argument, .. } => {
            if let Some(argument) = argument {
                collect_expression_sites_from_expr(ast, *argument, metadata);
            }
        }
        Stmt::With { object, body, .. } => {
            collect_expression_sites_from_expr(ast, *object, metadata);
            collect_expression_sites_from_stmt(ast, *body, metadata);
        }
        Stmt::Switch {
            discriminant,
            cases,
            ..
        } => {
            collect_expression_sites_from_expr(ast, *discriminant, metadata);
            for case in ast.get_switch_case_list(*cases) {
                if let Some(test) = case.test {
                    collect_expression_sites_from_expr(ast, test, metadata);
                }
                collect_module_expression_sites(ast, case.consequent, metadata);
            }
        }
        Stmt::Labeled { body, .. } => collect_expression_sites_from_stmt(ast, *body, metadata),
        Stmt::Throw { argument, .. } => {
            collect_expression_sites_from_expr(ast, *argument, metadata);
        }
        Stmt::Try {
            block,
            handler,
            finalizer,
            ..
        } => {
            collect_expression_sites_from_stmt(ast, *block, metadata);
            if let Some(handler) = handler {
                collect_expression_sites_from_stmt(ast, handler.body, metadata);
            }
            if let Some(finalizer) = finalizer {
                collect_expression_sites_from_stmt(ast, *finalizer, metadata);
            }
        }
        Stmt::Declaration { decl, .. } => collect_expression_sites_from_decl(ast, *decl, metadata),
        Stmt::Empty { .. }
        | Stmt::Continue { .. }
        | Stmt::Break { .. }
        | Stmt::Debugger { .. }
        | Stmt::InvalidStatement { .. } => {}
    }
}

fn collect_expression_sites_from_decl(
    ast: &lyng_js_ast::Ast,
    decl_id: lyng_js_ast::DeclId,
    metadata: &mut ModuleMetadata,
) {
    match ast.get_decl(decl_id) {
        Decl::Variable { declarators, .. } => {
            for declarator in ast.get_var_declarator_list(*declarators) {
                if let Some(init) = declarator.init {
                    collect_expression_sites_from_expr(ast, init, metadata);
                }
            }
        }
        Decl::Function { function, .. } => {
            collect_expression_sites_from_function(ast, *function, metadata);
        }
        Decl::Class {
            super_class, body, ..
        } => {
            if let Some(super_class) = super_class {
                collect_expression_sites_from_expr(ast, *super_class, metadata);
            }
            collect_expression_sites_from_class_body(ast, *body, metadata);
        }
        Decl::Export { kind, .. } => match kind {
            ExportKind::Default { declaration } => match declaration {
                ExportDefaultDecl::Function(function) => {
                    collect_expression_sites_from_function(ast, *function, metadata);
                }
                ExportDefaultDecl::Class(decl) => {
                    collect_expression_sites_from_decl(ast, *decl, metadata);
                }
                ExportDefaultDecl::Expression(expr) => {
                    collect_expression_sites_from_expr(ast, *expr, metadata);
                }
            },
            ExportKind::Declaration { decl } => {
                collect_expression_sites_from_decl(ast, *decl, metadata);
            }
            ExportKind::Named { .. } | ExportKind::All { .. } => {}
        },
        Decl::Import { .. } | Decl::InvalidDeclaration { .. } => {}
    }
}

fn collect_expression_sites_from_function(
    ast: &lyng_js_ast::Ast,
    function: lyng_js_ast::FunctionId,
    metadata: &mut ModuleMetadata,
) {
    let function = ast.get_function(function);
    collect_module_expression_sites(ast, function.body, metadata);
    if let Some(expression) = function.expression_body {
        collect_expression_sites_from_expr(ast, expression, metadata);
    }
}

fn collect_expression_sites_from_class_body(
    ast: &lyng_js_ast::Ast,
    body: lyng_js_ast::NodeList<lyng_js_ast::ClassElementId>,
    metadata: &mut ModuleMetadata,
) {
    for &element in ast.get_class_element_list(body) {
        match ast.get_class_element(element) {
            lyng_js_ast::ClassElement::Method {
                key,
                value,
                computed,
                ..
            } => {
                if *computed {
                    collect_expression_sites_from_expr(ast, *key, metadata);
                }
                collect_expression_sites_from_function(ast, *value, metadata);
            }
            lyng_js_ast::ClassElement::Property {
                key,
                value,
                computed,
                ..
            } => {
                if *computed {
                    collect_expression_sites_from_expr(ast, *key, metadata);
                }
                if let Some(value) = value {
                    collect_expression_sites_from_expr(ast, *value, metadata);
                }
            }
            lyng_js_ast::ClassElement::StaticBlock { body, .. } => {
                collect_module_expression_sites(ast, *body, metadata);
            }
            lyng_js_ast::ClassElement::InvalidElement { .. } => {}
        }
    }
}

fn collect_expression_sites_from_expr(
    ast: &lyng_js_ast::Ast,
    expr_id: lyng_js_ast::ExprId,
    metadata: &mut ModuleMetadata,
) {
    match ast.get_expr(expr_id) {
        lyng_js_ast::Expr::MetaProperty { meta, property, .. } => {
            if *meta == WellKnownAtom::import.id() && *property == WellKnownAtom::meta.id() {
                metadata.has_import_meta = true;
            }
        }
        lyng_js_ast::Expr::ImportExpression {
            span,
            source,
            options,
            ..
        } => {
            metadata
                .dynamic_import_sites
                .push(DynamicImportSite::new(*span));
            collect_expression_sites_from_expr(ast, *source, metadata);
            if let Some(options) = options {
                collect_expression_sites_from_expr(ast, *options, metadata);
            }
        }
        lyng_js_ast::Expr::ArrayExpression { elements, .. } => {
            for element in ast.get_opt_expr_list(*elements).iter().flatten() {
                collect_expression_sites_from_expr(ast, *element, metadata);
            }
        }
        lyng_js_ast::Expr::ObjectExpression { properties, .. } => {
            for property in ast.get_property_list(*properties) {
                if property.computed {
                    collect_expression_sites_from_expr(ast, property.key, metadata);
                }
                collect_expression_sites_from_expr(ast, property.value, metadata);
            }
        }
        lyng_js_ast::Expr::FunctionExpression { function, .. }
        | lyng_js_ast::Expr::ArrowFunctionExpression { function, .. } => {
            collect_expression_sites_from_function(ast, *function, metadata);
        }
        lyng_js_ast::Expr::ClassExpression {
            super_class, body, ..
        } => {
            if let Some(super_class) = super_class {
                collect_expression_sites_from_expr(ast, *super_class, metadata);
            }
            collect_expression_sites_from_class_body(ast, *body, metadata);
        }
        lyng_js_ast::Expr::TemplateLiteral { template, .. } => {
            for &expression in ast.templates().get_expressions(*template) {
                collect_expression_sites_from_expr(ast, expression, metadata);
            }
        }
        lyng_js_ast::Expr::TaggedTemplateExpression { tag, template, .. } => {
            collect_expression_sites_from_expr(ast, *tag, metadata);
            for &expression in ast.templates().get_expressions(*template) {
                collect_expression_sites_from_expr(ast, expression, metadata);
            }
        }
        lyng_js_ast::Expr::UnaryExpression { argument, .. }
        | lyng_js_ast::Expr::UpdateExpression { argument, .. }
        | lyng_js_ast::Expr::AwaitExpression { argument, .. }
        | lyng_js_ast::Expr::SpreadElement { argument, .. }
        | lyng_js_ast::Expr::OptionalChainExpression { base: argument, .. }
        | lyng_js_ast::Expr::ParenthesizedExpression {
            expression: argument,
            ..
        } => collect_expression_sites_from_expr(ast, *argument, metadata),
        lyng_js_ast::Expr::BinaryExpression { left, right, .. }
        | lyng_js_ast::Expr::LogicalExpression { left, right, .. }
        | lyng_js_ast::Expr::AssignmentExpression { left, right, .. } => {
            collect_expression_sites_from_expr(ast, *left, metadata);
            collect_expression_sites_from_expr(ast, *right, metadata);
        }
        lyng_js_ast::Expr::ConditionalExpression {
            test,
            consequent,
            alternate,
            ..
        } => {
            collect_expression_sites_from_expr(ast, *test, metadata);
            collect_expression_sites_from_expr(ast, *consequent, metadata);
            collect_expression_sites_from_expr(ast, *alternate, metadata);
        }
        lyng_js_ast::Expr::SequenceExpression { expressions, .. } => {
            for &expression in ast.get_expr_list(*expressions) {
                collect_expression_sites_from_expr(ast, expression, metadata);
            }
        }
        lyng_js_ast::Expr::CallExpression {
            callee, arguments, ..
        }
        | lyng_js_ast::Expr::NewExpression {
            callee, arguments, ..
        } => {
            collect_expression_sites_from_expr(ast, *callee, metadata);
            for &argument in ast.get_expr_list(*arguments) {
                collect_expression_sites_from_expr(ast, argument, metadata);
            }
        }
        lyng_js_ast::Expr::StaticMemberExpression { object, .. }
        | lyng_js_ast::Expr::PrivateMemberExpression { object, .. } => {
            collect_expression_sites_from_expr(ast, *object, metadata);
        }
        lyng_js_ast::Expr::ComputedMemberExpression {
            object, property, ..
        } => {
            collect_expression_sites_from_expr(ast, *object, metadata);
            collect_expression_sites_from_expr(ast, *property, metadata);
        }
        lyng_js_ast::Expr::PrivateInExpression { object, .. } => {
            collect_expression_sites_from_expr(ast, *object, metadata);
        }
        lyng_js_ast::Expr::YieldExpression { argument, .. } => {
            if let Some(argument) = argument {
                collect_expression_sites_from_expr(ast, *argument, metadata);
            }
        }
        lyng_js_ast::Expr::This { .. }
        | lyng_js_ast::Expr::Super { .. }
        | lyng_js_ast::Expr::Identifier { .. }
        | lyng_js_ast::Expr::NullLiteral { .. }
        | lyng_js_ast::Expr::BooleanLiteral { .. }
        | lyng_js_ast::Expr::NumericLiteral { .. }
        | lyng_js_ast::Expr::StringLiteral { .. }
        | lyng_js_ast::Expr::BigIntLiteral { .. }
        | lyng_js_ast::Expr::RegExpLiteral { .. }
        | lyng_js_ast::Expr::InvalidExpression { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_parser::parse_module;
    use lyng_js_sema::analyze_module;

    #[test]
    fn compile_module_emits_real_module_artifact() {
        let mut atoms = AtomTable::new();
        let parsed = parse_module(
            &mut atoms,
            SourceId::new(3),
            "import value from './dep.js'; export const local = value; export { local as alias };",
        );
        assert!(
            !parsed.diagnostics.has_errors(),
            "unexpected parse errors: {:?}",
            parsed.diagnostics.as_slice()
        );
        let sema = analyze_module(&parsed, &atoms);
        assert!(
            !sema.diagnostics.has_errors(),
            "unexpected sema errors: {:?}",
            sema.diagnostics.as_slice()
        );

        let unit = compile_module(&parsed, &sema, &mut atoms).unwrap();

        assert_eq!(unit.source(), SourceId::new(3));
        assert_eq!(unit.requested_modules().len(), 1);
        assert_eq!(unit.requested_modules()[0].specifier(), "./dep.js");
        assert_eq!(unit.import_entries().len(), 1);
        assert_eq!(unit.local_exports().len(), 2);
        assert!(unit.function(unit.entry()).is_some());
        assert!(unit.source_text().is_some());
    }

    #[test]
    fn compile_module_preserves_dynamic_import_sites_without_rejecting_lowering() {
        let mut atoms = AtomTable::new();
        let parsed = parse_module(
            &mut atoms,
            SourceId::new(4),
            "export default import('./dep.js');",
        );
        assert!(
            !parsed.diagnostics.has_errors(),
            "unexpected parse errors: {:?}",
            parsed.diagnostics.as_slice()
        );
        let sema = analyze_module(&parsed, &atoms);
        assert!(
            !sema.diagnostics.has_errors(),
            "unexpected sema errors: {:?}",
            sema.diagnostics.as_slice()
        );

        let unit = compile_module(&parsed, &sema, &mut atoms).unwrap();

        assert_eq!(unit.dynamic_import_sites().len(), 1);
        assert!(unit.function(unit.entry()).is_some());
    }

    #[test]
    fn compile_module_lowers_import_meta_and_preserves_metadata_atoms() {
        let mut atoms = AtomTable::new();
        let parsed = parse_module(
            &mut atoms,
            SourceId::new(5),
            "export const same = import.meta === import.meta; export default import.meta.url;",
        );
        assert!(
            !parsed.diagnostics.has_errors(),
            "unexpected parse errors: {:?}",
            parsed.diagnostics.as_slice()
        );
        let sema = analyze_module(&parsed, &atoms);
        assert!(
            !sema.diagnostics.has_errors(),
            "unexpected sema errors: {:?}",
            sema.diagnostics.as_slice()
        );

        let unit = compile_module(&parsed, &sema, &mut atoms).unwrap();

        assert!(unit.has_import_meta());
        assert!(unit
            .local_exports()
            .iter()
            .any(|entry| unit.atom_text(entry.export_name()) == Some("same")));
        assert!(unit.function(unit.entry()).is_some());
    }

    #[test]
    fn compile_module_lowers_exported_function_declaration_forms() {
        let mut atoms = AtomTable::new();
        let parsed = parse_module(
            &mut atoms,
            SourceId::new(6),
            "export function f() { return 23; } export default function g() { return f(); }",
        );
        assert!(
            !parsed.diagnostics.has_errors(),
            "unexpected parse errors: {:?}",
            parsed.diagnostics.as_slice()
        );
        let sema = analyze_module(&parsed, &atoms);
        assert!(
            !sema.diagnostics.has_errors(),
            "unexpected sema errors: {:?}",
            sema.diagnostics.as_slice()
        );

        let unit = compile_module(&parsed, &sema, &mut atoms).unwrap();

        assert!(unit
            .local_exports()
            .iter()
            .any(|entry| unit.atom_text(entry.export_name()) == Some("f")));
        assert!(unit
            .local_exports()
            .iter()
            .any(|entry| unit.atom_text(entry.export_name()) == Some("default")));
    }

    #[test]
    fn compile_module_lowers_named_default_function_with_local_binding() {
        let mut atoms = AtomTable::new();
        let parsed = parse_module(
            &mut atoms,
            SourceId::new(7),
            "export default function named() { return named; }",
        );
        assert!(
            !parsed.diagnostics.has_errors(),
            "unexpected parse errors: {:?}",
            parsed.diagnostics.as_slice()
        );
        let sema = analyze_module(&parsed, &atoms);
        assert!(
            !sema.diagnostics.has_errors(),
            "unexpected sema errors: {:?}",
            sema.diagnostics.as_slice()
        );

        let unit = compile_module(&parsed, &sema, &mut atoms).unwrap();

        assert_eq!(unit.local_exports().len(), 1);
        assert_eq!(
            unit.atom_text(unit.local_exports()[0].export_name()),
            Some("default")
        );
        assert_eq!(
            unit.local_exports()[0]
                .local_name()
                .and_then(|atom| unit.atom_text(atom)),
            Some("named")
        );
    }
}
