//! The main AST walker that builds scope tables, binding tables, resolves
//! references, computes captures, detects early errors, and assigns storage.
//!
//! The analysis is a single depth-first walk over the AST. Scopes are pushed
//! and popped as the walker enters and exits scope-introducing constructs.

mod bindings;
mod classes;
mod containment;
mod declarations;
mod directives;
mod expressions;
mod finalize;
mod functions;
mod statements;

use lyng_js_ast::{Ast, FunctionId};
use std::collections::{HashMap, HashSet};

use lyng_js_common::{AtomId, AtomTable, DiagnosticList, SourceId, Span};

use crate::binding::BindingTable;
use crate::class_private_layout::ClassPrivateLayoutTable;
use crate::function_sema::FunctionSemaTable;
use crate::ids::{FunctionSemaId, ScopeId, SemanticBindingId};
use crate::private_name::{PrivateNameRecord, PrivateNameTable};
use crate::private_use::PrivateUseTable;
use crate::results::DirectEvalScriptAnalysisOptions;
use crate::scope::{ScopeKind, ScopeRecord, ScopeTable};
use crate::use_site::UseSiteTable;

/// Context flags carried during the walk.
struct WalkContext {
    /// The current scope.
    current_scope: ScopeId,
    /// The current owning function sema id, if inside a function.
    current_function: Option<FunctionSemaId>,
    /// Whether we are currently in strict mode.
    strict: bool,
    /// Whether we are inside a loop (for break/continue validation).
    in_loop: bool,
    /// Whether we are inside a switch (for break validation).
    in_switch: bool,
    /// Whether we are inside a function (for return validation).
    in_function: bool,
    /// Whether we are inside a module.
    #[allow(dead_code)]
    in_module: bool,
    /// Active labels for labeled statement validation.
    labels: Vec<AtomId>,
    /// Active labels that are loop labels (for `continue` validation).
    loop_labels: Vec<AtomId>,
    /// Stack of class-body scopes for private name resolution.
    class_scopes: Vec<ScopeId>,
    /// Whether we are currently inside a class static block.
    in_static_block: bool,
    /// Exported names for duplicate-export detection in modules.
    exported_names: HashSet<AtomId>,
    /// Destructured catch parameters whose names block Annex B var replacement.
    annex_b_blocked_catch_names: Vec<HashSet<AtomId>>,
    /// Caller-sensitive names that block Annex B var replacement in direct eval.
    annex_b_blocked_var_names: HashSet<AtomId>,
}

/// The analyzer state accumulates all side tables during the walk.
pub(crate) struct Analyzer<'a> {
    ast: &'a Ast,
    atoms: &'a AtomTable,
    pub(crate) scopes: ScopeTable,
    pub(crate) bindings: BindingTable,
    pub(crate) functions: FunctionSemaTable,
    pub(crate) pattern_bindings: Vec<Option<SemanticBindingId>>,
    pub(crate) use_sites: UseSiteTable,
    pub(crate) private_names: PrivateNameTable,
    pub(crate) private_uses: PrivateUseTable,
    pub(crate) class_private_layouts: ClassPrivateLayoutTable,
    pub(crate) diagnostics: DiagnosticList,
    ctx: WalkContext,
    suppressed_function_name_bindings: HashSet<FunctionId>,
    /// Fast name-to-binding lookup per scope, avoiding O(n) linear scans
    /// in `declare_binding` and `declare_var_binding`.
    scope_binding_names: HashMap<(ScopeId, AtomId), SemanticBindingId>,
}

#[derive(Clone, Copy)]
enum ContainmentQuery {
    ArgumentsIdentifier,
    DirectSuperCall,
    NewTarget,
    SuperKeyword,
    YieldExpression,
}

#[derive(Clone, Copy, Default)]
struct PrivateNameUsage {
    getter_static: Option<bool>,
    setter_static: Option<bool>,
    other: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LexicalDeclaredNameKind {
    Other,
    AnnexBFunction,
}

#[derive(Clone, Copy)]
struct LexicalDeclaredName {
    name: AtomId,
    span: Span,
    kind: LexicalDeclaredNameKind,
}

impl<'a> Analyzer<'a> {
    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    fn new_for_script(ast: &'a Ast, atoms: &'a AtomTable, strict: bool) -> Self {
        let mut scopes = ScopeTable::new();
        let global_scope = scopes.alloc(ScopeRecord {
            parent: None,
            kind: ScopeKind::Global,
            owning_function: None,
            strict,
            has_eval: false,
            has_with: false,
            needs_environment: false,
            bindings: Vec::new(),
            children: Vec::new(),
        });

        Self {
            ast,
            atoms,
            scopes,
            bindings: BindingTable::new(),
            functions: FunctionSemaTable::new(),
            pattern_bindings: Vec::new(),
            use_sites: UseSiteTable::new(),
            private_names: PrivateNameTable::new(),
            private_uses: PrivateUseTable::new(),
            class_private_layouts: ClassPrivateLayoutTable::new(),
            diagnostics: DiagnosticList::new(),
            suppressed_function_name_bindings: HashSet::new(),
            scope_binding_names: HashMap::new(),
            ctx: WalkContext {
                current_scope: global_scope,
                current_function: None,
                strict,
                in_loop: false,
                in_switch: false,
                in_function: false,
                in_module: false,
                labels: Vec::new(),
                loop_labels: Vec::new(),
                class_scopes: Vec::new(),
                in_static_block: false,
                exported_names: HashSet::new(),
                annex_b_blocked_catch_names: Vec::new(),
                annex_b_blocked_var_names: HashSet::new(),
            },
        }
    }

    fn new_for_module(ast: &'a Ast, atoms: &'a AtomTable) -> Self {
        let mut scopes = ScopeTable::new();
        let module_scope = scopes.alloc(ScopeRecord {
            parent: None,
            kind: ScopeKind::Module,
            owning_function: None,
            strict: true, // modules are always strict
            has_eval: false,
            has_with: false,
            needs_environment: false,
            bindings: Vec::new(),
            children: Vec::new(),
        });

        Self {
            ast,
            atoms,
            scopes,
            bindings: BindingTable::new(),
            functions: FunctionSemaTable::new(),
            pattern_bindings: Vec::new(),
            use_sites: UseSiteTable::new(),
            private_names: PrivateNameTable::new(),
            private_uses: PrivateUseTable::new(),
            class_private_layouts: ClassPrivateLayoutTable::new(),
            diagnostics: DiagnosticList::new(),
            suppressed_function_name_bindings: HashSet::new(),
            scope_binding_names: HashMap::new(),
            ctx: WalkContext {
                current_scope: module_scope,
                current_function: None,
                strict: true,
                in_loop: false,
                in_switch: false,
                in_function: false,
                in_module: true,
                labels: Vec::new(),
                loop_labels: Vec::new(),
                class_scopes: Vec::new(),
                in_static_block: false,
                exported_names: HashSet::new(),
                annex_b_blocked_catch_names: Vec::new(),
                annex_b_blocked_var_names: HashSet::new(),
            },
        }
    }

    // -----------------------------------------------------------------------
    // Public entry points
    // -----------------------------------------------------------------------

    pub(crate) fn analyze_direct_eval_script(
        ast: &'a Ast,
        atoms: &'a AtomTable,
        root: lyng_js_ast::ScriptId,
        strict: bool,
        options: DirectEvalScriptAnalysisOptions,
    ) -> Self {
        let mut this = Self::new_for_script(ast, atoms, strict);
        let script = ast.get_script(root);
        this.suppressed_function_name_bindings = options
            .suppressed_function_name_bindings()
            .iter()
            .copied()
            .collect();
        this.seed_direct_eval_private_layouts(script.span.source, &options);
        this.ctx.annex_b_blocked_var_names = options
            .annex_b_blocked_var_names()
            .iter()
            .copied()
            .collect();
        if options.forbid_arguments_in_class_initializer()
            && this.stmt_list_contains_query(script.body, ContainmentQuery::ArgumentsIdentifier)
        {
            this.diagnostics.error(
                script.span,
                "class field initializer cannot contain 'arguments'",
            );
        }
        if options.forbid_direct_super_call()
            && this.stmt_list_contains_query(script.body, ContainmentQuery::DirectSuperCall)
        {
            this.diagnostics
                .error(script.span, "direct eval cannot contain super() here");
        }
        if options.forbid_super_call_in_class_initializer()
            && this.stmt_list_contains_query(script.body, ContainmentQuery::DirectSuperCall)
        {
            this.diagnostics.error(
                script.span,
                "class field initializer cannot contain direct super()",
            );
        }
        this.apply_directive_prologue(script.body);
        this.walk_stmt_list(script.body);
        this.check_global_code_contains(script.body);
        this.finalize();
        this
    }

    pub(crate) fn analyze_module(
        ast: &'a Ast,
        atoms: &'a AtomTable,
        root: lyng_js_ast::ModuleId,
    ) -> Self {
        let mut this = Self::new_for_module(ast, atoms);
        let module = ast.get_module(root);
        this.walk_stmt_list(module.body);
        this.check_global_code_contains(module.body);
        this.finalize();
        this
    }

    // -----------------------------------------------------------------------
    // Scope management
    // -----------------------------------------------------------------------

    fn push_scope(&mut self, kind: ScopeKind) -> ScopeId {
        let parent = self.ctx.current_scope;
        let id = self.scopes.alloc(ScopeRecord {
            parent: Some(parent),
            kind,
            owning_function: self.ctx.current_function,
            strict: self.ctx.strict,
            has_eval: false,
            has_with: false,
            needs_environment: false,
            bindings: Vec::new(),
            children: Vec::new(),
        });
        self.scopes.get_mut(parent).children.push(id);
        self.ctx.current_scope = id;
        id
    }

    fn pop_scope(&mut self) {
        let current = self.ctx.current_scope;
        if let Some(parent) = self.scopes.get(current).parent {
            self.ctx.current_scope = parent;
        }
    }

    fn seed_direct_eval_private_layouts(
        &mut self,
        source: SourceId,
        options: &DirectEvalScriptAnalysisOptions,
    ) {
        for entries in options.ambient_private_layouts() {
            let scope = self.scopes.alloc(ScopeRecord {
                parent: None,
                kind: ScopeKind::ClassBody,
                owning_function: None,
                strict: true,
                has_eval: false,
                has_with: false,
                needs_environment: false,
                bindings: Vec::new(),
                children: Vec::new(),
            });
            self.ctx.class_scopes.push(scope);
            let span = entries
                .first()
                .map_or(Span::from_offsets(source, 0, 0), |entry| entry.span());
            let mut seen_names = HashSet::new();
            for entry in entries {
                if seen_names.insert(entry.name()) {
                    self.private_names.alloc(PrivateNameRecord {
                        name: entry.name(),
                        scope,
                        span: entry.span(),
                    });
                }
            }
            self.class_private_layouts
                .alloc_imported(scope, span, entries.clone());
        }
    }
}
