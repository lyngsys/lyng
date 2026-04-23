mod exports;
mod patterns;

use std::collections::{HashMap, HashSet};

use lyng_js_ast::{CatchClause, Decl, ForInOfLeft, ForInit, FunctionKind, Stmt, VariableKind};
use lyng_js_common::AtomId;

use super::{Analyzer, LexicalDeclaredName, LexicalDeclaredNameKind};
use crate::binding::DeclarationKind;
use crate::scope::ScopeKind;

impl<'a> Analyzer<'a> {
    fn hoist_function_binding(
        &mut self,
        name: AtomId,
        scope: crate::ids::ScopeId,
        span: lyng_js_common::Span,
    ) {
        if self.function_binding_is_lexical_in_scope(scope) {
            self.declare_binding(name, DeclarationKind::Function, scope, span);
            return;
        }

        let var_scope = self.find_var_scope();
        if let Some(&existing_bid) = self.scope_binding_names.get(&(var_scope, name)) {
            let existing = self.bindings.get(existing_bid);
            if matches!(
                existing.kind,
                DeclarationKind::Function | DeclarationKind::Var
            ) {
                return;
            }
            if self.binding_is_lexical_in_scope(existing.kind, var_scope) {
                self.diagnostics.error(
                    span,
                    "function declaration conflicts with lexical declaration",
                );
                return;
            }
        }

        self.declare_binding(name, DeclarationKind::Function, var_scope, span);
    }

    pub(super) fn check_statement_list_redeclarations(
        &mut self,
        list: lyng_js_ast::NodeList<lyng_js_ast::StmtId>,
        scope_kind: ScopeKind,
    ) {
        let mut lexical_names = Vec::new();
        let mut var_names = Vec::new();
        self.collect_lexically_declared_names_from_stmt_list(list, scope_kind, &mut lexical_names);
        self.collect_var_declared_names_from_stmt_list(list, scope_kind, &mut var_names);

        self.report_lexical_var_conflicts(&lexical_names, var_names);
        self.report_duplicate_lexical_names(scope_kind, &lexical_names);
    }

    pub(super) fn check_switch_case_redeclarations(
        &mut self,
        cases: lyng_js_ast::NodeList<lyng_js_ast::SwitchCase>,
    ) {
        let mut lexical_names = Vec::new();
        let mut var_names = Vec::new();
        for case in self.ast.get_switch_case_list(cases) {
            self.collect_lexically_declared_names_from_stmt_list(
                case.consequent,
                ScopeKind::Switch,
                &mut lexical_names,
            );
            self.collect_var_declared_names_from_stmt_list(
                case.consequent,
                ScopeKind::Switch,
                &mut var_names,
            );
        }

        self.report_lexical_var_conflicts(&lexical_names, var_names);
        self.report_duplicate_lexical_names(ScopeKind::Switch, &lexical_names);
    }

    pub(super) fn report_lexical_var_conflicts(
        &mut self,
        lexical_names: &[LexicalDeclaredName],
        var_names: Vec<AtomId>,
    ) {
        let var_name_set: HashSet<AtomId> = var_names.into_iter().collect();
        let mut reported = HashSet::new();
        for entry in lexical_names {
            if var_name_set.contains(&entry.name) && reported.insert(entry.name) {
                self.diagnostics.error(
                    entry.span,
                    "lexical declaration conflicts with var-scoped declaration",
                );
            }
        }
    }

    pub(super) fn report_duplicate_lexical_names(
        &mut self,
        scope_kind: ScopeKind,
        lexical_names: &[LexicalDeclaredName],
    ) {
        let mut seen = HashMap::new();
        let mut reported = HashSet::new();
        for entry in lexical_names {
            if let Some(previous_kind) = seen.get(&entry.name).copied() {
                let duplicate_allowed = !self.ctx.strict
                    && matches!(scope_kind, ScopeKind::Block | ScopeKind::Switch)
                    && previous_kind == LexicalDeclaredNameKind::AnnexBFunction
                    && entry.kind == LexicalDeclaredNameKind::AnnexBFunction;

                if !duplicate_allowed && reported.insert(entry.name) {
                    self.diagnostics
                        .error(entry.span, "duplicate lexical declaration");
                }

                let combined_kind = if previous_kind == LexicalDeclaredNameKind::AnnexBFunction
                    && entry.kind == LexicalDeclaredNameKind::AnnexBFunction
                {
                    LexicalDeclaredNameKind::AnnexBFunction
                } else {
                    LexicalDeclaredNameKind::Other
                };
                seen.insert(entry.name, combined_kind);
            } else {
                seen.insert(entry.name, entry.kind);
            }
        }
    }

    pub(super) fn check_catch_clause_early_errors(&mut self, catch: &CatchClause) {
        let Some(param) = catch.param else {
            return;
        };

        let mut bound_names = Vec::new();
        self.collect_pattern_names(param, &mut bound_names);

        let mut seen = HashSet::new();
        for (name, span) in &bound_names {
            if !seen.insert(*name) {
                self.diagnostics
                    .error(*span, "duplicate binding in catch parameter");
            }
        }

        let Stmt::Block { body, .. } = self.ast.get_stmt(catch.body) else {
            return;
        };

        let mut lexical_names = Vec::new();
        self.collect_lexically_declared_names_from_stmt_list(
            *body,
            ScopeKind::Block,
            &mut lexical_names,
        );

        let bound_name_set: HashSet<AtomId> =
            bound_names.into_iter().map(|(name, _)| name).collect();
        let mut reported = HashSet::new();
        for entry in lexical_names {
            if bound_name_set.contains(&entry.name) && reported.insert(entry.name) {
                self.diagnostics.error(
                    entry.span,
                    "catch parameter conflicts with lexical declaration",
                );
            }
        }
    }

    pub(super) fn collect_lexically_declared_names_from_stmt_list(
        &self,
        list: lyng_js_ast::NodeList<lyng_js_ast::StmtId>,
        scope_kind: ScopeKind,
        out: &mut Vec<LexicalDeclaredName>,
    ) {
        for &stmt_id in self.ast.get_stmt_list(list) {
            self.collect_lexically_declared_names_from_stmt_list_item(stmt_id, scope_kind, out);
        }
    }

    pub(super) fn collect_lexically_declared_names_from_stmt_list_item(
        &self,
        stmt_id: lyng_js_ast::StmtId,
        scope_kind: ScopeKind,
        out: &mut Vec<LexicalDeclaredName>,
    ) {
        let stmt = self.ast.get_stmt(stmt_id);
        if let Stmt::Declaration { decl, .. } = stmt {
            self.collect_lexically_declared_names_from_decl(*decl, scope_kind, out);
        }
    }

    pub(super) fn collect_lexically_declared_names_from_decl(
        &self,
        decl_id: lyng_js_ast::DeclId,
        scope_kind: ScopeKind,
        out: &mut Vec<LexicalDeclaredName>,
    ) {
        match self.ast.get_decl(decl_id) {
            Decl::Variable {
                kind:
                    VariableKind::Let
                    | VariableKind::Const
                    | VariableKind::Using
                    | VariableKind::AwaitUsing,
                declarators,
                ..
            } => {
                for decl in self.ast.get_var_declarator_list(*declarators) {
                    let mut names = Vec::new();
                    self.collect_pattern_names(decl.id, &mut names);
                    out.extend(names.into_iter().map(|(name, span)| LexicalDeclaredName {
                        name,
                        span,
                        kind: LexicalDeclaredNameKind::Other,
                    }));
                }
            }
            Decl::Function { function, .. }
                if self.function_binding_is_lexical_in_scope_kind(scope_kind) =>
            {
                let func = self.ast.get_function(*function);
                if let Some(name) = func.name {
                    let kind = if !self.ctx.strict
                        && matches!(scope_kind, ScopeKind::Block | ScopeKind::Switch)
                        && func.kind == FunctionKind::Normal
                    {
                        LexicalDeclaredNameKind::AnnexBFunction
                    } else {
                        LexicalDeclaredNameKind::Other
                    };
                    out.push(LexicalDeclaredName {
                        name,
                        span: func.span,
                        kind,
                    });
                }
            }
            Decl::Class {
                name: Some(name),
                span,
                ..
            } => {
                out.push(LexicalDeclaredName {
                    name: *name,
                    span: *span,
                    kind: LexicalDeclaredNameKind::Other,
                });
            }
            Decl::Import { specifiers, .. } => {
                for spec in self.ast.get_import_spec_list(*specifiers) {
                    match spec {
                        lyng_js_ast::ImportSpecifier::Default { local, span, .. }
                        | lyng_js_ast::ImportSpecifier::Namespace { local, span, .. } => {
                            out.push(LexicalDeclaredName {
                                name: *local,
                                span: *span,
                                kind: LexicalDeclaredNameKind::Other,
                            });
                        }
                        lyng_js_ast::ImportSpecifier::Named { local, span, .. } => {
                            out.push(LexicalDeclaredName {
                                name: *local,
                                span: *span,
                                kind: LexicalDeclaredNameKind::Other,
                            });
                        }
                    }
                }
            }
            Decl::Export { kind, .. } => {
                if let lyng_js_ast::ExportKind::Declaration { decl } = kind {
                    self.collect_lexically_declared_names_from_decl(*decl, scope_kind, out);
                }
            }
            _ => {}
        }
    }

    pub(super) fn collect_var_declared_names_from_stmt_list(
        &self,
        list: lyng_js_ast::NodeList<lyng_js_ast::StmtId>,
        scope_kind: ScopeKind,
        out: &mut Vec<AtomId>,
    ) {
        for &stmt_id in self.ast.get_stmt_list(list) {
            self.collect_var_declared_names_from_stmt(stmt_id, scope_kind, out);
        }
    }

    pub(super) fn collect_var_declared_names_from_stmt(
        &self,
        stmt_id: lyng_js_ast::StmtId,
        scope_kind: ScopeKind,
        out: &mut Vec<AtomId>,
    ) {
        match self.ast.get_stmt(stmt_id) {
            Stmt::Block { body, .. } => {
                self.collect_var_declared_names_from_stmt_list(*body, ScopeKind::Block, out);
            }
            Stmt::If {
                consequent,
                alternate,
                ..
            } => {
                if !self.annex_b_if_clause_function_skips_var_name(*consequent, scope_kind) {
                    self.collect_var_declared_names_from_stmt(*consequent, scope_kind, out);
                }
                if let Some(alt) = alternate {
                    if !self.annex_b_if_clause_function_skips_var_name(*alt, scope_kind) {
                        self.collect_var_declared_names_from_stmt(*alt, scope_kind, out);
                    }
                }
            }
            Stmt::DoWhile { body, .. }
            | Stmt::While { body, .. }
            | Stmt::With { body, .. }
            | Stmt::Labeled { body, .. } => {
                self.collect_var_declared_names_from_stmt(*body, scope_kind, out);
            }
            Stmt::For { init, body, .. } => {
                if let Some(ForInit::Declaration(decl)) = init {
                    self.collect_var_declared_names_from_decl(*decl, scope_kind, out);
                }
                self.collect_var_declared_names_from_stmt(*body, scope_kind, out);
            }
            Stmt::ForIn { left, body, .. } | Stmt::ForOf { left, body, .. } => {
                if let ForInOfLeft::Declaration(decl) = left {
                    self.collect_var_declared_names_from_decl(*decl, scope_kind, out);
                }
                self.collect_var_declared_names_from_stmt(*body, scope_kind, out);
            }
            Stmt::Switch { cases, .. } => {
                for case in self.ast.get_switch_case_list(*cases) {
                    self.collect_var_declared_names_from_stmt_list(
                        case.consequent,
                        ScopeKind::Switch,
                        out,
                    );
                }
            }
            Stmt::Try {
                block,
                handler,
                finalizer,
                ..
            } => {
                self.collect_var_declared_names_from_stmt(*block, scope_kind, out);
                if let Some(catch) = handler {
                    self.collect_var_declared_names_from_stmt(catch.body, ScopeKind::Catch, out);
                }
                if let Some(finalizer) = finalizer {
                    self.collect_var_declared_names_from_stmt(*finalizer, scope_kind, out);
                }
            }
            Stmt::Declaration { decl, .. } => {
                self.collect_var_declared_names_from_decl(*decl, scope_kind, out);
            }
            Stmt::Expression { .. }
            | Stmt::Return { .. }
            | Stmt::Throw { .. }
            | Stmt::Break { .. }
            | Stmt::Continue { .. }
            | Stmt::Empty { .. }
            | Stmt::Debugger { .. }
            | Stmt::InvalidStatement { .. } => {}
        }
    }

    pub(super) fn collect_var_declared_names_from_decl(
        &self,
        decl_id: lyng_js_ast::DeclId,
        scope_kind: ScopeKind,
        out: &mut Vec<AtomId>,
    ) {
        match self.ast.get_decl(decl_id) {
            Decl::Variable {
                kind: VariableKind::Var,
                declarators,
                ..
            } => {
                for decl in self.ast.get_var_declarator_list(*declarators) {
                    let mut names = Vec::new();
                    self.collect_pattern_names(decl.id, &mut names);
                    out.extend(names.into_iter().map(|(name, _)| name));
                }
            }
            Decl::Function { function, .. }
                if !self.function_binding_is_lexical_in_scope_kind(scope_kind) =>
            {
                if let Some(name) = self.ast.get_function(*function).name {
                    out.push(name);
                }
            }
            Decl::Export { kind, .. } => match kind {
                lyng_js_ast::ExportKind::Declaration { decl } => {
                    self.collect_var_declared_names_from_decl(*decl, scope_kind, out);
                }
                lyng_js_ast::ExportKind::Default { declaration } => {
                    if let lyng_js_ast::ExportDefaultDecl::Function(function_id) = declaration {
                        if !self.function_binding_is_lexical_in_scope_kind(scope_kind) {
                            if let Some(name) = self.ast.get_function(*function_id).name {
                                out.push(name);
                            }
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub(super) fn hoist_declarations(&mut self, stmt_id: lyng_js_ast::StmtId) {
        let stmt = self.ast.get_stmt(stmt_id);
        match stmt {
            Stmt::Declaration { decl, span, .. } => {
                let decl_node = self.ast.get_decl(*decl);
                match decl_node {
                    Decl::Function { function, span, .. } => {
                        let func = self.ast.get_function(*function);
                        if let Some(name) = func.name {
                            self.hoist_function_binding(name, self.ctx.current_scope, *span);
                        }
                    }
                    Decl::Export { kind, .. } => match kind {
                        lyng_js_ast::ExportKind::Declaration { decl } => {
                            if let Decl::Function { function, span, .. } = self.ast.get_decl(*decl)
                            {
                                let func = self.ast.get_function(*function);
                                if let Some(name) = func.name {
                                    self.hoist_function_binding(
                                        name,
                                        self.ctx.current_scope,
                                        *span,
                                    );
                                }
                            }
                        }
                        lyng_js_ast::ExportKind::Default { declaration } => {
                            if let lyng_js_ast::ExportDefaultDecl::Function(function) = declaration {
                                let func = self.ast.get_function(*function);
                                if let Some(name) = func.name {
                                    self.hoist_function_binding(
                                        name,
                                        self.ctx.current_scope,
                                        func.span,
                                    );
                                }
                            }
                        }
                        lyng_js_ast::ExportKind::Named { .. }
                        | lyng_js_ast::ExportKind::All { .. } => {}
                    },
                    Decl::Import { specifiers, .. } => {
                        for spec in self.ast.get_import_spec_list(*specifiers) {
                            match spec {
                                lyng_js_ast::ImportSpecifier::Default { local, span, .. }
                                | lyng_js_ast::ImportSpecifier::Namespace { local, span, .. }
                                | lyng_js_ast::ImportSpecifier::Named { local, span, .. } => {
                                    self.declare_binding(
                                        *local,
                                        DeclarationKind::Import,
                                        self.ctx.current_scope,
                                        *span,
                                    );
                                }
                            }
                        }
                    }
                    Decl::Variable {
                        kind: VariableKind::Var,
                        declarators,
                        ..
                    } => {
                        let decls = self.ast.get_var_declarator_list(*declarators);
                        for decl in decls {
                            self.hoist_var_pattern(decl.id, *span);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub(super) fn walk_decl(&mut self, decl_id: lyng_js_ast::DeclId) {
        let decl = self.ast.get_decl(decl_id);
        match decl {
            Decl::Variable {
                kind, declarators, ..
            } => {
                let kind = *kind;
                let declarators = *declarators;
                let decl_kind = match kind {
                    VariableKind::Var => DeclarationKind::Var,
                    VariableKind::Let => DeclarationKind::Let,
                    VariableKind::Const => DeclarationKind::Const,
                    VariableKind::Using => DeclarationKind::Using,
                    VariableKind::AwaitUsing => DeclarationKind::AwaitUsing,
                };
                let decl_list = self.ast.get_var_declarator_list(declarators);
                for d in decl_list {
                    if kind == VariableKind::Var {
                        self.hoist_var_pattern(d.id, d.span);
                    } else {
                        self.declare_pattern_bindings(d.id, decl_kind);
                    }
                    self.walk_binding_pattern_expressions(d.id);
                    if let Some(init) = d.init {
                        self.walk_expr(init);
                    }
                }
            }
            Decl::Function { function, .. } => {
                let func = self.ast.get_function(*function);
                let function_strict = self.ctx.strict
                    || (func.expression_body.is_none()
                        && self.strict_directive_span(func.body).is_some());
                if self
                    .function_body_contains_query(*function, super::ContainmentQuery::SuperKeyword)
                {
                    self.diagnostics
                        .error(func.span, "'super' keyword outside of a method");
                }
                if function_strict {
                    if let Some(name) = func.name {
                        self.check_strict_binding_name(name, func.span);
                    }
                }
                self.walk_function(*function);
            }
            Decl::Class {
                name,
                super_class,
                body,
                span,
                ..
            } => {
                let old_strict = self.ctx.strict;
                self.ctx.strict = true;
                if let Some(n) = name {
                    self.declare_binding(*n, DeclarationKind::Class, self.ctx.current_scope, *span);
                }
                self.walk_class_body(*body, *span, *name, *super_class, super_class.is_some());
                self.ctx.strict = old_strict;
            }
            Decl::Import { .. } => {}
            Decl::Export { kind, span, .. } => {
                self.walk_export_kind(kind, *span);
            }
            Decl::InvalidDeclaration { .. } => {}
        }
    }
}
