#![allow(
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::cast_possible_truncation,
    clippy::module_name_repetitions
)]

//! Compiler-oriented, arena-backed AST for ECMA-262 Edition 16.
//!
//! This crate defines the AST node types, typed node IDs, arena storage,
//! and literal tables for the lyng-js JavaScript engine. The parser produces
//! a `ParsedScript` or `ParsedModule` that owns an `Ast` container.
//!
//! # Design
//!
//! - **Arena storage**: Each node family lives in its own `NodeArena`.
//!   Child lists are stored in `ListArena` side storage and referenced by
//!   `NodeList<T>` (a `(start, len)` pair).
//! - **Typed IDs**: Every node reference is a 32-bit, `Copy` ID type.
//!   No per-node heap allocation for child lists.
//! - **Patterns separate from expressions**: Patterns are their own node
//!   family and do not reuse expression variants.

pub mod arena;
pub mod common;
pub mod decl;
pub mod expr;
pub mod function;
pub mod ids;
pub mod literal;
pub mod module;
pub mod pattern;
pub mod roots;
pub mod script;
pub mod stmt;
pub mod template;

// Re-export node types at crate root for convenience.
pub use common::*;
pub use decl::{
    Decl, ExportDefaultDecl, ExportKind, ExportSpecifier, ImportAttribute, ImportSpecifier,
    VariableDeclarator,
};
pub use expr::{Expr, ImportExpressionPhase, Property};
pub use function::{ClassElement, FormalParameters, Function};
pub use ids::*;
pub use literal::{
    LiteralTable, NumericLiteral, NumericLiteralSyntax, RegExpValue, StringLiteralSyntax,
    StringLiteralValue,
};
pub use module::Module;
pub use pattern::{ArrayPatternElement, ObjectPatternProperty, Pattern};
pub use roots::{ParsedModule, ParsedScript};
pub use script::Script;
pub use stmt::{CatchClause, ForInOfLeft, ForInit, Stmt, SwitchCase};
pub use template::{TemplateArena, TemplateLiteralData, TemplateQuasi};

use arena::{ListArena, NodeArena};

// ---------------------------------------------------------------------------
// Ast container
// ---------------------------------------------------------------------------

/// The central AST container that owns all node arenas, list arenas,
/// literal tables, and template storage.
///
/// The parser builds an `Ast` incrementally via `alloc_*` methods, then
/// wraps it in a `ParsedScript` or `ParsedModule`.
pub struct Ast {
    // Node arenas
    scripts: NodeArena<ScriptId, Script>,
    modules: NodeArena<ModuleId, Module>,
    stmts: NodeArena<StmtId, Stmt>,
    exprs: NodeArena<ExprId, Expr>,
    decls: NodeArena<DeclId, Decl>,
    patterns: NodeArena<PatternId, Pattern>,
    functions: NodeArena<FunctionId, Function>,
    class_elements: NodeArena<ClassElementId, ClassElement>,

    // List arenas
    stmt_lists: ListArena<StmtId>,
    expr_lists: ListArena<ExprId>,
    opt_expr_lists: ListArena<Option<ExprId>>,
    decl_lists: ListArena<DeclId>,
    pattern_lists: ListArena<PatternId>,
    opt_pattern_elem_lists: ListArena<Option<ArrayPatternElement>>,
    class_element_lists: ListArena<ClassElementId>,
    import_spec_lists: ListArena<ImportSpecifier>,
    import_attr_lists: ListArena<ImportAttribute>,
    export_spec_lists: ListArena<ExportSpecifier>,
    var_declarator_lists: ListArena<VariableDeclarator>,
    switch_case_lists: ListArena<SwitchCase>,
    property_lists: ListArena<Property>,
    obj_pattern_prop_lists: ListArena<ObjectPatternProperty>,

    // Literal tables
    literals: LiteralTable,

    // Template arena
    templates: TemplateArena,
}

impl Default for Ast {
    fn default() -> Self {
        Self::new()
    }
}

impl Ast {
    /// Creates a new, empty AST container.
    pub fn new() -> Self {
        Self {
            scripts: NodeArena::new(),
            modules: NodeArena::new(),
            stmts: NodeArena::new(),
            exprs: NodeArena::new(),
            decls: NodeArena::new(),
            patterns: NodeArena::new(),
            functions: NodeArena::new(),
            class_elements: NodeArena::new(),

            stmt_lists: ListArena::new(),
            expr_lists: ListArena::new(),
            opt_expr_lists: ListArena::new(),
            decl_lists: ListArena::new(),
            pattern_lists: ListArena::new(),
            opt_pattern_elem_lists: ListArena::new(),
            class_element_lists: ListArena::new(),
            import_spec_lists: ListArena::new(),
            import_attr_lists: ListArena::new(),
            export_spec_lists: ListArena::new(),
            var_declarator_lists: ListArena::new(),
            switch_case_lists: ListArena::new(),
            property_lists: ListArena::new(),
            obj_pattern_prop_lists: ListArena::new(),

            literals: LiteralTable::new(),
            templates: TemplateArena::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Node allocation
    // -----------------------------------------------------------------------

    /// Allocates a `Script` node and returns its ID.
    #[inline]
    pub fn alloc_script(&mut self, node: Script) -> ScriptId {
        self.scripts.alloc(node)
    }

    /// Allocates a `Module` node and returns its ID.
    #[inline]
    pub fn alloc_module(&mut self, node: Module) -> ModuleId {
        self.modules.alloc(node)
    }

    /// Allocates a `Stmt` node and returns its ID.
    #[inline]
    pub fn alloc_stmt(&mut self, node: Stmt) -> StmtId {
        self.stmts.alloc(node)
    }

    /// Allocates an `Expr` node and returns its ID.
    #[inline]
    pub fn alloc_expr(&mut self, node: Expr) -> ExprId {
        self.exprs.alloc(node)
    }

    /// Allocates a `Decl` node and returns its ID.
    #[inline]
    pub fn alloc_decl(&mut self, node: Decl) -> DeclId {
        self.decls.alloc(node)
    }

    /// Allocates a `Pattern` node and returns its ID.
    #[inline]
    pub fn alloc_pattern(&mut self, node: Pattern) -> PatternId {
        self.patterns.alloc(node)
    }

    /// Allocates a `Function` node and returns its ID.
    #[inline]
    pub fn alloc_function(&mut self, node: Function) -> FunctionId {
        self.functions.alloc(node)
    }

    /// Allocates a `ClassElement` node and returns its ID.
    #[inline]
    pub fn alloc_class_element(&mut self, node: ClassElement) -> ClassElementId {
        self.class_elements.alloc(node)
    }

    // -----------------------------------------------------------------------
    // Node access
    // -----------------------------------------------------------------------

    /// Returns a reference to the `Script` at the given ID.
    #[inline]
    pub fn get_script(&self, id: ScriptId) -> &Script {
        self.scripts.get(id)
    }

    /// Returns a reference to the `Module` at the given ID.
    #[inline]
    pub fn get_module(&self, id: ModuleId) -> &Module {
        self.modules.get(id)
    }

    /// Returns a reference to the `Stmt` at the given ID.
    #[inline]
    pub fn get_stmt(&self, id: StmtId) -> &Stmt {
        self.stmts.get(id)
    }

    /// Returns a reference to the `Expr` at the given ID.
    #[inline]
    pub fn get_expr(&self, id: ExprId) -> &Expr {
        self.exprs.get(id)
    }

    /// Returns a reference to the `Decl` at the given ID.
    #[inline]
    pub fn get_decl(&self, id: DeclId) -> &Decl {
        self.decls.get(id)
    }

    /// Returns a reference to the `Pattern` at the given ID.
    #[inline]
    pub fn get_pattern(&self, id: PatternId) -> &Pattern {
        self.patterns.get(id)
    }

    /// Returns a reference to the `Function` at the given ID.
    #[inline]
    pub fn get_function(&self, id: FunctionId) -> &Function {
        self.functions.get(id)
    }

    /// Returns a reference to the `ClassElement` at the given ID.
    #[inline]
    pub fn get_class_element(&self, id: ClassElementId) -> &ClassElement {
        self.class_elements.get(id)
    }

    // -----------------------------------------------------------------------
    // List allocation
    // -----------------------------------------------------------------------

    /// Allocates a list of statement IDs and returns a `NodeList<StmtId>`.
    #[inline]
    pub fn alloc_stmt_list(&mut self, items: &[StmtId]) -> NodeList<StmtId> {
        self.stmt_lists.alloc(items)
    }

    /// Allocates a list of expression IDs.
    #[inline]
    pub fn alloc_expr_list(&mut self, items: &[ExprId]) -> NodeList<ExprId> {
        self.expr_lists.alloc(items)
    }

    /// Allocates a list of optional expression IDs (for array literals with elisions).
    #[inline]
    pub fn alloc_opt_expr_list(&mut self, items: &[Option<ExprId>]) -> NodeList<Option<ExprId>> {
        self.opt_expr_lists.alloc(items)
    }

    /// Allocates a list of declaration IDs.
    #[inline]
    pub fn alloc_decl_list(&mut self, items: &[DeclId]) -> NodeList<DeclId> {
        self.decl_lists.alloc(items)
    }

    /// Allocates a list of pattern IDs.
    #[inline]
    pub fn alloc_pattern_list(&mut self, items: &[PatternId]) -> NodeList<PatternId> {
        self.pattern_lists.alloc(items)
    }

    /// Allocates a list of optional array pattern elements.
    #[inline]
    pub fn alloc_opt_pattern_elem_list(
        &mut self,
        items: &[Option<ArrayPatternElement>],
    ) -> NodeList<Option<ArrayPatternElement>> {
        self.opt_pattern_elem_lists.alloc(items)
    }

    /// Allocates a list of class element IDs.
    #[inline]
    pub fn alloc_class_element_list(
        &mut self,
        items: &[ClassElementId],
    ) -> NodeList<ClassElementId> {
        self.class_element_lists.alloc(items)
    }

    /// Allocates a list of import specifiers.
    #[inline]
    pub fn alloc_import_spec_list(
        &mut self,
        items: &[ImportSpecifier],
    ) -> NodeList<ImportSpecifier> {
        self.import_spec_lists.alloc(items)
    }

    /// Allocates a list of retained import attributes.
    #[inline]
    pub fn alloc_import_attr_list(
        &mut self,
        items: &[ImportAttribute],
    ) -> NodeList<ImportAttribute> {
        self.import_attr_lists.alloc(items)
    }

    /// Allocates a list of export specifiers.
    #[inline]
    pub fn alloc_export_spec_list(
        &mut self,
        items: &[ExportSpecifier],
    ) -> NodeList<ExportSpecifier> {
        self.export_spec_lists.alloc(items)
    }

    /// Allocates a list of variable declarators.
    #[inline]
    pub fn alloc_var_declarator_list(
        &mut self,
        items: &[VariableDeclarator],
    ) -> NodeList<VariableDeclarator> {
        self.var_declarator_lists.alloc(items)
    }

    /// Allocates a list of switch cases.
    #[inline]
    pub fn alloc_switch_case_list(&mut self, items: &[SwitchCase]) -> NodeList<SwitchCase> {
        self.switch_case_lists.alloc(items)
    }

    /// Allocates a list of object properties.
    #[inline]
    pub fn alloc_property_list(&mut self, items: &[Property]) -> NodeList<Property> {
        self.property_lists.alloc(items)
    }

    /// Allocates a list of object pattern properties.
    #[inline]
    pub fn alloc_obj_pattern_prop_list(
        &mut self,
        items: &[ObjectPatternProperty],
    ) -> NodeList<ObjectPatternProperty> {
        self.obj_pattern_prop_lists.alloc(items)
    }

    // -----------------------------------------------------------------------
    // List access
    // -----------------------------------------------------------------------

    /// Returns the statement IDs for a list.
    #[inline]
    pub fn get_stmt_list(&self, list: NodeList<StmtId>) -> &[StmtId] {
        self.stmt_lists.get(list)
    }

    /// Returns the expression IDs for a list.
    #[inline]
    pub fn get_expr_list(&self, list: NodeList<ExprId>) -> &[ExprId] {
        self.expr_lists.get(list)
    }

    /// Returns the optional expression IDs for a list.
    #[inline]
    pub fn get_opt_expr_list(&self, list: NodeList<Option<ExprId>>) -> &[Option<ExprId>] {
        self.opt_expr_lists.get(list)
    }

    /// Returns the declaration IDs for a list.
    #[inline]
    pub fn get_decl_list(&self, list: NodeList<DeclId>) -> &[DeclId] {
        self.decl_lists.get(list)
    }

    /// Returns the pattern IDs for a list.
    #[inline]
    pub fn get_pattern_list(&self, list: NodeList<PatternId>) -> &[PatternId] {
        self.pattern_lists.get(list)
    }

    /// Returns the optional array pattern elements for a list.
    #[inline]
    pub fn get_opt_pattern_elem_list(
        &self,
        list: NodeList<Option<ArrayPatternElement>>,
    ) -> &[Option<ArrayPatternElement>] {
        self.opt_pattern_elem_lists.get(list)
    }

    /// Returns the class element IDs for a list.
    #[inline]
    pub fn get_class_element_list(&self, list: NodeList<ClassElementId>) -> &[ClassElementId] {
        self.class_element_lists.get(list)
    }

    /// Returns the import specifiers for a list.
    #[inline]
    pub fn get_import_spec_list(&self, list: NodeList<ImportSpecifier>) -> &[ImportSpecifier] {
        self.import_spec_lists.get(list)
    }

    /// Returns the import attributes for a list.
    #[inline]
    pub fn get_import_attr_list(&self, list: NodeList<ImportAttribute>) -> &[ImportAttribute] {
        self.import_attr_lists.get(list)
    }

    /// Returns the export specifiers for a list.
    #[inline]
    pub fn get_export_spec_list(&self, list: NodeList<ExportSpecifier>) -> &[ExportSpecifier] {
        self.export_spec_lists.get(list)
    }

    /// Returns the variable declarators for a list.
    #[inline]
    pub fn get_var_declarator_list(
        &self,
        list: NodeList<VariableDeclarator>,
    ) -> &[VariableDeclarator] {
        self.var_declarator_lists.get(list)
    }

    /// Returns the switch cases for a list.
    #[inline]
    pub fn get_switch_case_list(&self, list: NodeList<SwitchCase>) -> &[SwitchCase] {
        self.switch_case_lists.get(list)
    }

    /// Returns the object properties for a list.
    #[inline]
    pub fn get_property_list(&self, list: NodeList<Property>) -> &[Property] {
        self.property_lists.get(list)
    }

    /// Returns the object pattern properties for a list.
    #[inline]
    pub fn get_obj_pattern_prop_list(
        &self,
        list: NodeList<ObjectPatternProperty>,
    ) -> &[ObjectPatternProperty] {
        self.obj_pattern_prop_lists.get(list)
    }

    // -----------------------------------------------------------------------
    // Literal table access
    // -----------------------------------------------------------------------

    /// Returns a mutable reference to the literal table.
    #[inline]
    pub const fn literals_mut(&mut self) -> &mut LiteralTable {
        &mut self.literals
    }

    /// Returns a reference to the literal table.
    #[inline]
    pub const fn literals(&self) -> &LiteralTable {
        &self.literals
    }

    // -----------------------------------------------------------------------
    // Template arena access
    // -----------------------------------------------------------------------

    /// Returns a mutable reference to the template arena.
    #[inline]
    pub const fn templates_mut(&mut self) -> &mut TemplateArena {
        &mut self.templates
    }

    /// Returns a reference to the template arena.
    #[inline]
    pub const fn templates(&self) -> &TemplateArena {
        &self.templates
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_common::{AtomId, SourceId, Span};

    fn test_span() -> Span {
        Span::from_offsets(SourceId::new(0), 0, 10)
    }

    // -- Node allocation and access ----------------------------------------

    #[test]
    fn alloc_and_get_stmt() {
        let mut ast = Ast::new();
        let id = ast.alloc_stmt(Stmt::Empty { span: test_span() });
        let stmt = ast.get_stmt(id);
        assert!(matches!(stmt, Stmt::Empty { .. }));
    }

    #[test]
    fn alloc_and_get_expr() {
        let mut ast = Ast::new();
        let id = ast.alloc_expr(Expr::This { span: test_span() });
        let expr = ast.get_expr(id);
        assert!(matches!(expr, Expr::This { .. }));
    }

    #[test]
    fn alloc_and_get_decl() {
        let mut ast = Ast::new();
        let id = ast.alloc_decl(Decl::InvalidDeclaration { span: test_span() });
        let decl = ast.get_decl(id);
        assert!(matches!(decl, Decl::InvalidDeclaration { .. }));
    }

    #[test]
    fn alloc_and_get_pattern() {
        let mut ast = Ast::new();
        let id = ast.alloc_pattern(Pattern::Identifier {
            span: test_span(),
            name: AtomId::from_raw(1),
        });
        let pat = ast.get_pattern(id);
        assert!(matches!(pat, Pattern::Identifier { .. }));
    }

    #[test]
    fn alloc_and_get_function() {
        let mut ast = Ast::new();
        let params_list = ast.alloc_pattern_list(&[]);
        let body_list = ast.alloc_stmt_list(&[]);
        let id = ast.alloc_function(Function {
            span: test_span(),
            name: None,
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span: test_span(),
                params: params_list,
                rest: None,
            },
            body: body_list,
            expression_body: None,
        });
        let func = ast.get_function(id);
        assert_eq!(func.kind, FunctionKind::Normal);
        assert!(func.name.is_none());
    }

    #[test]
    fn alloc_and_get_class_element() {
        let mut ast = Ast::new();
        let id = ast.alloc_class_element(ClassElement::InvalidElement { span: test_span() });
        let elem = ast.get_class_element(id);
        assert!(matches!(elem, ClassElement::InvalidElement { .. }));
    }

    // -- List allocation and access ----------------------------------------

    #[test]
    fn stmt_list_roundtrip() {
        let mut ast = Ast::new();
        let s0 = ast.alloc_stmt(Stmt::Empty { span: test_span() });
        let s1 = ast.alloc_stmt(Stmt::Debugger { span: test_span() });
        let list = ast.alloc_stmt_list(&[s0, s1]);
        let items = ast.get_stmt_list(list);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], s0);
        assert_eq!(items[1], s1);
    }

    #[test]
    fn expr_list_roundtrip() {
        let mut ast = Ast::new();
        let e0 = ast.alloc_expr(Expr::This { span: test_span() });
        let e1 = ast.alloc_expr(Expr::NullLiteral { span: test_span() });
        let list = ast.alloc_expr_list(&[e0, e1]);
        let items = ast.get_expr_list(list);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], e0);
        assert_eq!(items[1], e1);
    }

    #[test]
    fn opt_expr_list_with_elision() {
        let mut ast = Ast::new();
        let e = ast.alloc_expr(Expr::NullLiteral { span: test_span() });
        let list = ast.alloc_opt_expr_list(&[Some(e), None, Some(e)]);
        let items = ast.get_opt_expr_list(list);
        assert_eq!(items.len(), 3);
        assert!(items[0].is_some());
        assert!(items[1].is_none());
        assert!(items[2].is_some());
    }

    #[test]
    fn empty_list() {
        let mut ast = Ast::new();
        let list = ast.alloc_stmt_list(&[]);
        assert!(list.is_empty());
        assert_eq!(ast.get_stmt_list(list).len(), 0);
    }

    // -- Script / Module root allocation -----------------------------------

    #[test]
    fn alloc_script() {
        let mut ast = Ast::new();
        let body = ast.alloc_stmt_list(&[]);
        let id = ast.alloc_script(Script {
            span: test_span(),
            body,
        });
        let script = ast.get_script(id);
        assert!(script.body.is_empty());
    }

    #[test]
    fn alloc_module() {
        let mut ast = Ast::new();
        let body = ast.alloc_stmt_list(&[]);
        let id = ast.alloc_module(Module {
            span: test_span(),
            body,
        });
        let module = ast.get_module(id);
        assert!(module.body.is_empty());
    }

    // -- Literal table integration -----------------------------------------

    #[test]
    fn literal_table_through_ast() {
        let mut ast = Ast::new();
        let sid = ast.literals_mut().alloc_string("hello");
        assert_eq!(ast.literals().get_string(sid), "hello");
    }

    // -- Template arena integration ----------------------------------------

    #[test]
    fn template_through_ast() {
        let mut ast = Ast::new();
        let raw = ast.literals_mut().alloc_string("raw text");
        let cooked = ast.literals_mut().alloc_string("cooked text");
        let quasi = TemplateQuasi {
            cooked: Some(cooked),
            raw,
        };
        let tid = ast.templates_mut().alloc(&[quasi], &[]);
        assert_eq!(ast.templates().get_quasis(tid).len(), 1);
    }

    // -- Span access on nodes ----------------------------------------------

    #[test]
    fn expr_span() {
        let span = test_span();
        let expr = Expr::This { span };
        assert_eq!(expr.span(), span);
    }

    #[test]
    fn stmt_span() {
        let span = test_span();
        let stmt = Stmt::Debugger { span };
        assert_eq!(stmt.span(), span);
    }

    #[test]
    fn decl_span() {
        let span = test_span();
        let decl = Decl::InvalidDeclaration { span };
        assert_eq!(decl.span(), span);
    }

    #[test]
    fn pattern_span() {
        let span = test_span();
        let pat = Pattern::InvalidPattern { span };
        assert_eq!(pat.span(), span);
    }

    #[test]
    fn class_element_span() {
        let span = test_span();
        let elem = ClassElement::InvalidElement { span };
        assert_eq!(elem.span(), span);
    }

    // -- Compound AST construction (simulating parser output) ---------------

    #[test]
    fn build_if_statement() {
        let mut ast = Ast::new();
        let span = test_span();

        // if (true) { return 1; }
        let test_expr = ast.alloc_expr(Expr::BooleanLiteral { span, value: true });
        let ret_val = ast.alloc_expr(Expr::NumericLiteral {
            span,
            value: NumericLiteral::Int32(1),
            syntax: NumericLiteralSyntax::Normal,
        });
        let ret_stmt = ast.alloc_stmt(Stmt::Return {
            span,
            argument: Some(ret_val),
        });
        let body_list = ast.alloc_stmt_list(&[ret_stmt]);
        let block = ast.alloc_stmt(Stmt::Block {
            span,
            body: body_list,
        });
        let if_stmt = ast.alloc_stmt(Stmt::If {
            span,
            test: test_expr,
            consequent: block,
            alternate: None,
        });

        let stmt = ast.get_stmt(if_stmt);
        assert!(matches!(
            stmt,
            Stmt::If {
                alternate: None,
                ..
            }
        ));

        // Navigate: get the block body
        if let Stmt::If { consequent, .. } = stmt {
            if let Stmt::Block { body, .. } = ast.get_stmt(*consequent) {
                let stmts = ast.get_stmt_list(*body);
                assert_eq!(stmts.len(), 1);
                assert!(matches!(ast.get_stmt(stmts[0]), Stmt::Return { .. }));
            } else {
                panic!("expected block");
            }
        }
    }

    #[test]
    fn build_function_declaration() {
        let mut ast = Ast::new();
        let span = test_span();
        let name = AtomId::from_raw(100);

        // function foo(x) { return x; }
        let param = ast.alloc_pattern(Pattern::Identifier {
            span,
            name: AtomId::from_raw(101), // "x"
        });
        let params_list = ast.alloc_pattern_list(&[param]);
        let x_ref = ast.alloc_expr(Expr::Identifier {
            span,
            name: AtomId::from_raw(101),
        });
        let ret = ast.alloc_stmt(Stmt::Return {
            span,
            argument: Some(x_ref),
        });
        let body = ast.alloc_stmt_list(&[ret]);
        let func_id = ast.alloc_function(Function {
            span,
            name: Some(name),
            kind: FunctionKind::Normal,
            params: FormalParameters {
                span,
                params: params_list,
                rest: None,
            },
            body,
            expression_body: None,
        });
        let decl_id = ast.alloc_decl(Decl::Function {
            span,
            function: func_id,
        });

        let decl = ast.get_decl(decl_id);
        if let Decl::Function { function, .. } = decl {
            let func = ast.get_function(*function);
            assert_eq!(func.name, Some(name));
            assert_eq!(func.kind, FunctionKind::Normal);
            let params = ast.get_pattern_list(func.params.params);
            assert_eq!(params.len(), 1);
        } else {
            panic!("expected function declaration");
        }
    }

    #[test]
    fn build_variable_declaration() {
        let mut ast = Ast::new();
        let span = test_span();

        // const x = 42;
        let pat = ast.alloc_pattern(Pattern::Identifier {
            span,
            name: AtomId::from_raw(10),
        });
        let init = ast.alloc_expr(Expr::NumericLiteral {
            span,
            value: NumericLiteral::Int32(42),
            syntax: NumericLiteralSyntax::Normal,
        });
        let declarator = VariableDeclarator {
            span,
            id: pat,
            init: Some(init),
        };
        let declarators = ast.alloc_var_declarator_list(&[declarator]);
        let decl_id = ast.alloc_decl(Decl::Variable {
            span,
            kind: VariableKind::Const,
            declarators,
        });

        if let Decl::Variable {
            kind, declarators, ..
        } = ast.get_decl(decl_id)
        {
            assert_eq!(*kind, VariableKind::Const);
            let decls = ast.get_var_declarator_list(*declarators);
            assert_eq!(decls.len(), 1);
            assert!(decls[0].init.is_some());
        } else {
            panic!("expected variable declaration");
        }
    }

    #[test]
    fn build_object_destructuring() {
        let mut ast = Ast::new();
        let span = test_span();

        // const { a, b: c } = obj;
        let key_a = ast.alloc_expr(Expr::Identifier {
            span,
            name: AtomId::from_raw(20),
        });
        let val_a = ast.alloc_pattern(Pattern::Identifier {
            span,
            name: AtomId::from_raw(20),
        });
        let key_b = ast.alloc_expr(Expr::Identifier {
            span,
            name: AtomId::from_raw(21),
        });
        let val_c = ast.alloc_pattern(Pattern::Identifier {
            span,
            name: AtomId::from_raw(22),
        });

        let props = ast.alloc_obj_pattern_prop_list(&[
            ObjectPatternProperty {
                span,
                key: key_a,
                value: val_a,
                computed: false,
                shorthand: true,
            },
            ObjectPatternProperty {
                span,
                key: key_b,
                value: val_c,
                computed: false,
                shorthand: false,
            },
        ]);

        let pat_id = ast.alloc_pattern(Pattern::Object {
            span,
            properties: props,
            rest: None,
        });

        if let Pattern::Object { properties, .. } = ast.get_pattern(pat_id) {
            let ps = ast.get_obj_pattern_prop_list(*properties);
            assert_eq!(ps.len(), 2);
            assert!(ps[0].shorthand);
            assert!(!ps[1].shorthand);
        } else {
            panic!("expected object pattern");
        }
    }

    #[test]
    fn build_parsed_script() {
        let mut ast = Ast::new();
        let span = test_span();
        let body = ast.alloc_stmt_list(&[]);
        let root = ast.alloc_script(Script { span, body });
        let parsed = ParsedScript {
            ast,
            root,
            source_text: "".into(),
            diagnostics: lyng_js_common::DiagnosticList::new(),
            strict: false,
        };
        assert!(!parsed.strict);
        assert!(parsed.diagnostics.is_empty());
        assert!(parsed.ast.get_script(parsed.root).body.is_empty());
    }

    #[test]
    fn build_parsed_module() {
        let mut ast = Ast::new();
        let span = test_span();
        let body = ast.alloc_stmt_list(&[]);
        let root = ast.alloc_module(Module { span, body });
        let parsed = ParsedModule {
            ast,
            root,
            source_text: "".into(),
            diagnostics: lyng_js_common::DiagnosticList::new(),
        };
        assert!(parsed.diagnostics.is_empty());
        assert!(parsed.ast.get_module(parsed.root).body.is_empty());
    }

    // -- Error recovery nodes ----------------------------------------------

    #[test]
    fn invalid_expression() {
        let mut ast = Ast::new();
        let id = ast.alloc_expr(Expr::InvalidExpression { span: test_span() });
        assert!(matches!(ast.get_expr(id), Expr::InvalidExpression { .. }));
    }

    #[test]
    fn invalid_statement() {
        let mut ast = Ast::new();
        let id = ast.alloc_stmt(Stmt::InvalidStatement { span: test_span() });
        assert!(matches!(ast.get_stmt(id), Stmt::InvalidStatement { .. }));
    }

    #[test]
    fn invalid_pattern() {
        let mut ast = Ast::new();
        let id = ast.alloc_pattern(Pattern::InvalidPattern { span: test_span() });
        assert!(matches!(
            ast.get_pattern(id),
            Pattern::InvalidPattern { .. }
        ));
    }
}
