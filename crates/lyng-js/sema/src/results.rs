use lyng_js_ast::{ParsedModule, ParsedScript, PatternId};
use lyng_js_common::{AtomId, AtomTable, DiagnosticList};

use crate::{
    analyzer, BindingTable, ClassPrivateElementRecord, ClassPrivateLayoutTable, FunctionSemaTable,
    PrivateNameTable, PrivateUseTable, ScopeTable, SemanticBindingId, UseSiteTable,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DirectEvalScriptAnalysisOptions {
    ambient_private_layouts: Vec<Vec<ClassPrivateElementRecord>>,
    forbid_arguments_in_class_initializer: bool,
    annex_b_blocked_var_names: Vec<AtomId>,
}

impl DirectEvalScriptAnalysisOptions {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn with_ambient_private_layouts(
        mut self,
        ambient_private_layouts: Vec<Vec<ClassPrivateElementRecord>>,
    ) -> Self {
        self.ambient_private_layouts = ambient_private_layouts;
        self
    }

    #[inline]
    pub const fn with_forbid_arguments_in_class_initializer(mut self, enabled: bool) -> Self {
        self.forbid_arguments_in_class_initializer = enabled;
        self
    }

    #[inline]
    pub fn with_annex_b_blocked_var_names(mut self, names: Vec<AtomId>) -> Self {
        self.annex_b_blocked_var_names = names;
        self
    }

    #[inline]
    pub fn ambient_private_layouts(&self) -> &[Vec<ClassPrivateElementRecord>] {
        &self.ambient_private_layouts
    }

    #[inline]
    pub const fn forbid_arguments_in_class_initializer(&self) -> bool {
        self.forbid_arguments_in_class_initializer
    }

    #[inline]
    pub fn annex_b_blocked_var_names(&self) -> &[AtomId] {
        &self.annex_b_blocked_var_names
    }
}

/// Borrowed compiler-facing view over shared sema tables.
#[derive(Clone, Copy)]
pub struct ProgramSemaView<'a> {
    pub scope_table: &'a ScopeTable,
    pub binding_table: &'a BindingTable,
    pub function_table: &'a FunctionSemaTable,
    pattern_bindings: &'a [Option<SemanticBindingId>],
    pub use_sites: &'a UseSiteTable,
    pub private_names: &'a PrivateNameTable,
    pub private_uses: &'a PrivateUseTable,
    pub class_private_layouts: &'a ClassPrivateLayoutTable,
}

impl<'a> ProgramSemaView<'a> {
    #[inline]
    pub fn pattern_binding(&self, pattern: PatternId) -> Option<SemanticBindingId> {
        self.pattern_bindings
            .get(pattern.raw() as usize)
            .copied()
            .flatten()
    }
}

/// The result of semantic analysis on a script.
pub struct ScriptSema {
    pub scope_table: ScopeTable,
    pub binding_table: BindingTable,
    pub function_table: FunctionSemaTable,
    pattern_bindings: Vec<Option<SemanticBindingId>>,
    pub use_sites: UseSiteTable,
    pub private_names: PrivateNameTable,
    pub private_uses: PrivateUseTable,
    pub class_private_layouts: ClassPrivateLayoutTable,
    pub diagnostics: DiagnosticList,
}

/// The result of semantic analysis on a module.
pub struct ModuleSema {
    pub scope_table: ScopeTable,
    pub binding_table: BindingTable,
    pub function_table: FunctionSemaTable,
    pattern_bindings: Vec<Option<SemanticBindingId>>,
    pub use_sites: UseSiteTable,
    pub private_names: PrivateNameTable,
    pub private_uses: PrivateUseTable,
    pub class_private_layouts: ClassPrivateLayoutTable,
    pub diagnostics: DiagnosticList,
}

impl ScriptSema {
    #[inline]
    pub fn pattern_binding(&self, pattern: PatternId) -> Option<SemanticBindingId> {
        self.pattern_bindings
            .get(pattern.raw() as usize)
            .copied()
            .flatten()
    }

    #[inline]
    pub fn view(&self) -> ProgramSemaView<'_> {
        ProgramSemaView {
            scope_table: &self.scope_table,
            binding_table: &self.binding_table,
            function_table: &self.function_table,
            pattern_bindings: &self.pattern_bindings,
            use_sites: &self.use_sites,
            private_names: &self.private_names,
            private_uses: &self.private_uses,
            class_private_layouts: &self.class_private_layouts,
        }
    }
}

impl ModuleSema {
    #[inline]
    pub fn pattern_binding(&self, pattern: PatternId) -> Option<SemanticBindingId> {
        self.pattern_bindings
            .get(pattern.raw() as usize)
            .copied()
            .flatten()
    }

    #[inline]
    pub fn view(&self) -> ProgramSemaView<'_> {
        ProgramSemaView {
            scope_table: &self.scope_table,
            binding_table: &self.binding_table,
            function_table: &self.function_table,
            pattern_bindings: &self.pattern_bindings,
            use_sites: &self.use_sites,
            private_names: &self.private_names,
            private_uses: &self.private_uses,
            class_private_layouts: &self.class_private_layouts,
        }
    }
}

/// Analyzes a parsed script and produces semantic metadata.
pub fn analyze_script(parsed: &ParsedScript, atoms: &AtomTable) -> ScriptSema {
    analyze_direct_eval_script(parsed, atoms, DirectEvalScriptAnalysisOptions::default())
}

/// Analyzes a parsed direct-eval script with caller-sensitive analysis options.
pub fn analyze_direct_eval_script(
    parsed: &ParsedScript,
    atoms: &AtomTable,
    options: DirectEvalScriptAnalysisOptions,
) -> ScriptSema {
    let analyzer = analyzer::Analyzer::analyze_direct_eval_script(
        &parsed.ast,
        atoms,
        parsed.root,
        parsed.strict,
        options,
    );
    ScriptSema {
        scope_table: analyzer.scopes,
        binding_table: analyzer.bindings,
        function_table: analyzer.functions,
        pattern_bindings: analyzer.pattern_bindings,
        use_sites: analyzer.use_sites,
        private_names: analyzer.private_names,
        private_uses: analyzer.private_uses,
        class_private_layouts: analyzer.class_private_layouts,
        diagnostics: analyzer.diagnostics,
    }
}

/// Analyzes a parsed module and produces semantic metadata.
pub fn analyze_module(parsed: &ParsedModule, atoms: &AtomTable) -> ModuleSema {
    let analyzer = analyzer::Analyzer::analyze_module(&parsed.ast, atoms, parsed.root);
    ModuleSema {
        scope_table: analyzer.scopes,
        binding_table: analyzer.bindings,
        function_table: analyzer.functions,
        pattern_bindings: analyzer.pattern_bindings,
        use_sites: analyzer.use_sites,
        private_names: analyzer.private_names,
        private_uses: analyzer.private_uses,
        class_private_layouts: analyzer.class_private_layouts,
        diagnostics: analyzer.diagnostics,
    }
}
