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

    pub(super) fn predeclare_switch_case_bindings(
        &mut self,
        cases: lyng_js_ast::NodeList<lyng_js_ast::SwitchCase>,
    ) {
        for case in self.ast.get_switch_case_list(cases) {
            for &stmt_id in self.ast.get_stmt_list(case.consequent) {
                self.predeclare_stmt_lexical_bindings(stmt_id);
            }
        }
    }

    pub(super) fn hoist_switch_case_declarations(
        &mut self,
        cases: lyng_js_ast::NodeList<lyng_js_ast::SwitchCase>,
    ) {
        let cases = self.ast.get_switch_case_list(cases).to_vec();
        for case in &cases {
            self.hoist_annex_b_block_function_var_bindings(case.consequent);
        }
        for case in &cases {
            for &stmt_id in self.ast.get_stmt_list(case.consequent) {
                self.hoist_declarations(stmt_id);
            }
        }
    }

    pub(super) fn predeclare_stmt_lexical_bindings(&mut self, stmt_id: lyng_js_ast::StmtId) {
        let Stmt::Declaration { decl, .. } = self.ast.get_stmt(stmt_id) else {
            return;
        };
        self.predeclare_decl_lexical_bindings(*decl);
    }

    fn predeclare_decl_lexical_bindings(&mut self, decl_id: lyng_js_ast::DeclId) {
        let decl = self.ast.get_decl(decl_id);
        match decl {
            Decl::Variable {
                kind, declarators, ..
            } => {
                let declaration_kind = match kind {
                    VariableKind::Var => return,
                    VariableKind::Let => DeclarationKind::Let,
                    VariableKind::Const => DeclarationKind::Const,
                    VariableKind::Using => DeclarationKind::Using,
                    VariableKind::AwaitUsing => DeclarationKind::AwaitUsing,
                };
                for declarator in self.ast.get_var_declarator_list(*declarators) {
                    self.declare_pattern_bindings(declarator.id, declaration_kind);
                }
            }
            Decl::Class { name, span, .. } => {
                if let Some(name) = name {
                    if !self
                        .scope_binding_names
                        .contains_key(&(self.ctx.current_scope, *name))
                    {
                        self.declare_binding(
                            *name,
                            DeclarationKind::Class,
                            self.ctx.current_scope,
                            *span,
                        );
                    }
                }
            }
            Decl::Export { kind, .. } => match kind {
                lyng_js_ast::ExportKind::Declaration { decl } => {
                    self.predeclare_decl_lexical_bindings(*decl);
                }
                lyng_js_ast::ExportKind::Default { declaration } => match declaration {
                    lyng_js_ast::ExportDefaultDecl::Class(class) => {
                        if let Decl::Class {
                            name: Some(name),
                            span,
                            ..
                        } = self.ast.get_decl(*class)
                        {
                            if !self
                                .scope_binding_names
                                .contains_key(&(self.ctx.current_scope, *name))
                            {
                                self.declare_binding(
                                    *name,
                                    DeclarationKind::Class,
                                    self.ctx.current_scope,
                                    *span,
                                );
                            }
                        }
                    }
                    lyng_js_ast::ExportDefaultDecl::Function(_) => {}
                    lyng_js_ast::ExportDefaultDecl::Expression(_) => {}
                },
                lyng_js_ast::ExportKind::Named { .. } | lyng_js_ast::ExportKind::All { .. } => {}
            },
            Decl::Function { .. } | Decl::Import { .. } | Decl::InvalidDeclaration { .. } => {}
        }
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
                        | lyng_js_ast::ImportSpecifier::Namespace { local, span, .. }
                        | lyng_js_ast::ImportSpecifier::Source { local, span, .. } => {
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

    pub(super) fn check_for_head_body_var_conflicts(
        &mut self,
        decl_id: lyng_js_ast::DeclId,
        body: lyng_js_ast::StmtId,
    ) {
        let mut head_names = Vec::new();
        self.collect_for_head_lexical_bound_names(decl_id, &mut head_names);
        if head_names.is_empty() {
            return;
        }

        let scope_kind = self.scopes.get(self.ctx.current_scope).kind;
        let mut body_var_names = Vec::new();
        self.collect_var_declared_names_from_stmt(body, scope_kind, &mut body_var_names);
        let body_var_name_set: HashSet<AtomId> = body_var_names.into_iter().collect();

        let mut reported = HashSet::new();
        for (name, span) in head_names {
            if body_var_name_set.contains(&name) && reported.insert(name) {
                self.diagnostics.error(
                    span,
                    "for loop head declaration conflicts with body var declaration",
                );
            }
        }
    }

    fn collect_for_head_lexical_bound_names(
        &self,
        decl_id: lyng_js_ast::DeclId,
        out: &mut Vec<(AtomId, lyng_js_common::Span)>,
    ) {
        let Decl::Variable {
            kind:
                VariableKind::Let | VariableKind::Const | VariableKind::Using | VariableKind::AwaitUsing,
            declarators,
            ..
        } = self.ast.get_decl(decl_id)
        else {
            return;
        };

        for declarator in self.ast.get_var_declarator_list(*declarators) {
            self.collect_pattern_names(declarator.id, out);
        }
    }

    pub(super) fn hoist_annex_b_block_function_var_bindings(
        &mut self,
        list: lyng_js_ast::NodeList<lyng_js_ast::StmtId>,
    ) {
        let scope_kind = self.scopes.get(self.ctx.current_scope).kind;
        if self.ctx.strict || !matches!(scope_kind, ScopeKind::Block | ScopeKind::Switch) {
            return;
        }

        for &stmt_id in self.ast.get_stmt_list(list) {
            if let Some((name, span)) = self.annex_b_block_level_function_declaration(stmt_id) {
                self.hoist_annex_b_block_function_var_binding(name, span);
            }
        }
    }

    pub(super) fn hoist_annex_b_contained_block_function_var_bindings(
        &mut self,
        list: lyng_js_ast::NodeList<lyng_js_ast::StmtId>,
    ) {
        let scope_kind = self.scopes.get(self.ctx.current_scope).kind;
        if self.ctx.strict || !matches!(scope_kind, ScopeKind::Global | ScopeKind::Function) {
            return;
        }

        let blocked = self.annex_b_non_function_lexical_names_in_list(list);
        self.hoist_annex_b_contained_functions_from_stmt_list(list, false, &blocked);
    }

    fn hoist_annex_b_contained_functions_from_stmt_list(
        &mut self,
        list: lyng_js_ast::NodeList<lyng_js_ast::StmtId>,
        direct_candidate_context: bool,
        inherited_blocked_names: &HashSet<AtomId>,
    ) {
        let mut candidate_blocked_names = inherited_blocked_names.clone();
        candidate_blocked_names.extend(self.annex_b_non_function_lexical_names_in_list(list));

        if direct_candidate_context {
            for &stmt_id in self.ast.get_stmt_list(list) {
                if let Some((name, span)) = self.annex_b_block_level_function_declaration(stmt_id) {
                    if !candidate_blocked_names.contains(&name) {
                        self.hoist_annex_b_block_function_var_binding(name, span);
                    }
                }
            }
        }

        let mut child_blocked_names = candidate_blocked_names;
        if direct_candidate_context {
            child_blocked_names.extend(self.annex_b_direct_function_names_in_list(list));
        }

        for &stmt_id in self.ast.get_stmt_list(list) {
            self.hoist_annex_b_contained_functions_from_stmt(stmt_id, &child_blocked_names);
        }
    }

    fn hoist_annex_b_contained_functions_from_stmt(
        &mut self,
        stmt_id: lyng_js_ast::StmtId,
        inherited_blocked_names: &HashSet<AtomId>,
    ) {
        match self.ast.get_stmt(stmt_id) {
            Stmt::Block { body, .. } => {
                self.hoist_annex_b_contained_functions_from_stmt_list(
                    *body,
                    true,
                    inherited_blocked_names,
                );
            }
            Stmt::If {
                consequent,
                alternate,
                ..
            } => {
                self.hoist_annex_b_if_clause_function(*consequent, inherited_blocked_names);
                if let Some(alternate) = alternate {
                    self.hoist_annex_b_if_clause_function(*alternate, inherited_blocked_names);
                }
                self.hoist_annex_b_contained_functions_from_stmt(
                    *consequent,
                    inherited_blocked_names,
                );
                if let Some(alternate) = alternate {
                    self.hoist_annex_b_contained_functions_from_stmt(
                        *alternate,
                        inherited_blocked_names,
                    );
                }
            }
            Stmt::DoWhile { body, .. }
            | Stmt::While { body, .. }
            | Stmt::With { body, .. }
            | Stmt::Labeled { body, .. } => {
                self.hoist_annex_b_contained_functions_from_stmt(*body, inherited_blocked_names);
            }
            Stmt::For { init, body, .. } => {
                let mut blocked = inherited_blocked_names.clone();
                if let Some(ForInit::Declaration(decl)) = init {
                    self.annex_b_collect_for_head_lexical_names(*decl, &mut blocked);
                }
                self.hoist_annex_b_contained_functions_from_stmt(*body, &blocked);
            }
            Stmt::ForIn { left, body, .. } | Stmt::ForOf { left, body, .. } => {
                let mut blocked = inherited_blocked_names.clone();
                if let ForInOfLeft::Declaration(decl) = left {
                    self.annex_b_collect_for_head_lexical_names(*decl, &mut blocked);
                }
                self.hoist_annex_b_contained_functions_from_stmt(*body, &blocked);
            }
            Stmt::Switch { cases, .. } => {
                let mut switch_blocked_names = inherited_blocked_names.clone();
                for case in self.ast.get_switch_case_list(*cases) {
                    switch_blocked_names
                        .extend(self.annex_b_non_function_lexical_names_in_list(case.consequent));
                }
                for case in self.ast.get_switch_case_list(*cases) {
                    self.hoist_annex_b_contained_functions_from_stmt_list(
                        case.consequent,
                        true,
                        &switch_blocked_names,
                    );
                }
            }
            Stmt::Try {
                block,
                handler,
                finalizer,
                ..
            } => {
                self.hoist_annex_b_contained_functions_from_stmt(*block, inherited_blocked_names);
                if let Some(catch) = handler {
                    let mut catch_blocked_names = inherited_blocked_names.clone();
                    self.annex_b_collect_catch_blocking_names(catch, &mut catch_blocked_names);
                    self.hoist_annex_b_contained_functions_from_stmt(
                        catch.body,
                        &catch_blocked_names,
                    );
                }
                if let Some(finalizer) = finalizer {
                    self.hoist_annex_b_contained_functions_from_stmt(
                        *finalizer,
                        inherited_blocked_names,
                    );
                }
            }
            Stmt::Declaration { .. }
            | Stmt::Expression { .. }
            | Stmt::Return { .. }
            | Stmt::Throw { .. }
            | Stmt::Break { .. }
            | Stmt::Continue { .. }
            | Stmt::Empty { .. }
            | Stmt::Debugger { .. }
            | Stmt::InvalidStatement { .. } => {}
        }
    }

    fn hoist_annex_b_if_clause_function(
        &mut self,
        stmt_id: lyng_js_ast::StmtId,
        inherited_blocked_names: &HashSet<AtomId>,
    ) {
        if let Some((name, span)) = self.annex_b_direct_function_declaration(stmt_id) {
            if !inherited_blocked_names.contains(&name) {
                self.hoist_annex_b_block_function_var_binding(name, span);
            }
        }
    }

    fn annex_b_direct_function_declaration(
        &self,
        stmt_id: lyng_js_ast::StmtId,
    ) -> Option<(AtomId, lyng_js_common::Span)> {
        let Stmt::Declaration { decl, .. } = self.ast.get_stmt(stmt_id) else {
            return None;
        };
        let Decl::Function { function, span, .. } = self.ast.get_decl(*decl) else {
            return None;
        };
        let func = self.ast.get_function(*function);
        (func.kind == FunctionKind::Normal).then_some((func.name?, *span))
    }

    fn annex_b_block_level_function_declaration(
        &self,
        stmt_id: lyng_js_ast::StmtId,
    ) -> Option<(AtomId, lyng_js_common::Span)> {
        match self.ast.get_stmt(stmt_id) {
            Stmt::Labeled { body, .. } => self.annex_b_block_level_function_declaration(*body),
            _ => self.annex_b_direct_function_declaration(stmt_id),
        }
    }

    fn annex_b_direct_function_names_in_list(
        &self,
        list: lyng_js_ast::NodeList<lyng_js_ast::StmtId>,
    ) -> HashSet<AtomId> {
        self.ast
            .get_stmt_list(list)
            .iter()
            .filter_map(|&stmt| self.annex_b_block_level_function_declaration(stmt))
            .map(|(name, _)| name)
            .collect()
    }

    fn annex_b_non_function_lexical_names_in_list(
        &self,
        list: lyng_js_ast::NodeList<lyng_js_ast::StmtId>,
    ) -> HashSet<AtomId> {
        let mut names = HashSet::new();
        for &stmt_id in self.ast.get_stmt_list(list) {
            let Stmt::Declaration { decl, .. } = self.ast.get_stmt(stmt_id) else {
                continue;
            };
            self.annex_b_collect_non_function_lexical_names(*decl, &mut names);
        }
        names
    }

    fn annex_b_collect_non_function_lexical_names(
        &self,
        decl_id: lyng_js_ast::DeclId,
        names: &mut HashSet<AtomId>,
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
                for declarator in self.ast.get_var_declarator_list(*declarators) {
                    let mut bound = Vec::new();
                    self.collect_pattern_names(declarator.id, &mut bound);
                    names.extend(bound.into_iter().map(|(name, _)| name));
                }
            }
            Decl::Class {
                name: Some(name), ..
            } => {
                names.insert(*name);
            }
            Decl::Function { function, .. } => {
                let func = self.ast.get_function(*function);
                if func.kind != FunctionKind::Normal {
                    if let Some(name) = func.name {
                        names.insert(name);
                    }
                }
            }
            Decl::Export { kind, .. } => {
                if let lyng_js_ast::ExportKind::Declaration { decl } = kind {
                    self.annex_b_collect_non_function_lexical_names(*decl, names);
                }
            }
            Decl::Import { specifiers, .. } => {
                for spec in self.ast.get_import_spec_list(*specifiers) {
                    match spec {
                        lyng_js_ast::ImportSpecifier::Default { local, .. }
                        | lyng_js_ast::ImportSpecifier::Namespace { local, .. }
                        | lyng_js_ast::ImportSpecifier::Source { local, .. }
                        | lyng_js_ast::ImportSpecifier::Named { local, .. } => {
                            names.insert(*local);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn annex_b_collect_for_head_lexical_names(
        &self,
        decl_id: lyng_js_ast::DeclId,
        names: &mut HashSet<AtomId>,
    ) {
        let Decl::Variable {
            kind:
                VariableKind::Let | VariableKind::Const | VariableKind::Using | VariableKind::AwaitUsing,
            declarators,
            ..
        } = self.ast.get_decl(decl_id)
        else {
            return;
        };
        for declarator in self.ast.get_var_declarator_list(*declarators) {
            let mut bound = Vec::new();
            self.collect_pattern_names(declarator.id, &mut bound);
            names.extend(bound.into_iter().map(|(name, _)| name));
        }
    }

    fn annex_b_collect_catch_blocking_names(
        &self,
        catch: &CatchClause,
        names: &mut HashSet<AtomId>,
    ) {
        let Some(param) = catch.param else {
            return;
        };
        if matches!(
            self.ast.get_pattern(param),
            lyng_js_ast::Pattern::Identifier { .. }
        ) {
            return;
        }
        let mut bound = Vec::new();
        self.collect_pattern_names(param, &mut bound);
        names.extend(bound.into_iter().map(|(name, _)| name));
    }

    fn hoist_annex_b_block_function_var_binding(
        &mut self,
        name: AtomId,
        span: lyng_js_common::Span,
    ) {
        if name == lyng_js_common::WellKnownAtom::arguments.id()
            || self.annex_b_parameter_names_contain(name)
        {
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
            return;
        }

        if self.annex_b_var_replacement_would_conflict(name, var_scope) {
            return;
        }

        self.declare_binding(name, DeclarationKind::Var, var_scope, span);
    }

    fn annex_b_parameter_names_contain(&self, name: AtomId) -> bool {
        let Some(function) = self.ctx.current_function else {
            return false;
        };
        let record = self.functions.get(function);
        [Some(record.scope_root), record.param_scope]
            .into_iter()
            .flatten()
            .any(|scope| {
                self.scopes.get(scope).bindings.iter().any(|&binding| {
                    let binding = self.bindings.get(binding);
                    binding.kind == DeclarationKind::Parameter && binding.name == name
                })
            })
    }

    fn annex_b_var_replacement_would_conflict(
        &self,
        name: AtomId,
        var_scope: crate::ids::ScopeId,
    ) -> bool {
        if self
            .ctx
            .annex_b_blocked_catch_names
            .iter()
            .any(|names| names.contains(&name))
            || self.ctx.annex_b_blocked_var_names.contains(&name)
        {
            return true;
        }

        let mut scope_id = self.ctx.current_scope;
        loop {
            if let Some(&binding) = self.scope_binding_names.get(&(scope_id, name)) {
                let binding = self.bindings.get(binding);
                if self.binding_is_lexical_in_scope(binding.kind, scope_id) {
                    return true;
                }
            }
            if scope_id == var_scope {
                return false;
            }
            let Some(parent) = self.scopes.get(scope_id).parent else {
                return false;
            };
            scope_id = parent;
        }
    }

    pub(super) fn hoist_declarations(&mut self, stmt_id: lyng_js_ast::StmtId) {
        let stmt = self.ast.get_stmt(stmt_id);
        match stmt {
            Stmt::Declaration { decl, .. } => {
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
                            if let lyng_js_ast::ExportDefaultDecl::Function(function) = declaration
                            {
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
                                | lyng_js_ast::ImportSpecifier::Source { local, span, .. }
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
                    _ => {}
                }
            }
            Stmt::Labeled { body, .. } => {
                self.hoist_declarations(*body);
            }
            _ => {}
        }
        self.hoist_var_declarations_from_stmt(stmt_id);
    }

    fn hoist_var_declarations_from_stmt_list(
        &mut self,
        list: lyng_js_ast::NodeList<lyng_js_ast::StmtId>,
    ) {
        let stmts = self.ast.get_stmt_list(list).to_vec();
        for stmt_id in stmts {
            self.hoist_var_declarations_from_stmt(stmt_id);
        }
    }

    fn hoist_var_declarations_from_stmt(&mut self, stmt_id: lyng_js_ast::StmtId) {
        match self.ast.get_stmt(stmt_id).clone() {
            Stmt::Block { body, .. } => {
                self.hoist_var_declarations_from_stmt_list(body);
            }
            Stmt::If {
                consequent,
                alternate,
                ..
            } => {
                self.hoist_var_declarations_from_stmt(consequent);
                if let Some(alternate) = alternate {
                    self.hoist_var_declarations_from_stmt(alternate);
                }
            }
            Stmt::DoWhile { body, .. }
            | Stmt::While { body, .. }
            | Stmt::With { body, .. }
            | Stmt::Labeled { body, .. } => {
                self.hoist_var_declarations_from_stmt(body);
            }
            Stmt::For { init, body, .. } => {
                if let Some(ForInit::Declaration(decl)) = init {
                    self.hoist_var_declarations_from_decl(decl);
                }
                self.hoist_var_declarations_from_stmt(body);
            }
            Stmt::ForIn { left, body, .. } | Stmt::ForOf { left, body, .. } => {
                if let ForInOfLeft::Declaration(decl) = left {
                    self.hoist_var_declarations_from_decl(decl);
                }
                self.hoist_var_declarations_from_stmt(body);
            }
            Stmt::Switch { cases, .. } => {
                for case in self.ast.get_switch_case_list(cases).to_vec() {
                    self.hoist_var_declarations_from_stmt_list(case.consequent);
                }
            }
            Stmt::Try {
                block,
                handler,
                finalizer,
                ..
            } => {
                self.hoist_var_declarations_from_stmt(block);
                if let Some(catch) = handler {
                    self.hoist_var_declarations_from_stmt(catch.body);
                }
                if let Some(finalizer) = finalizer {
                    self.hoist_var_declarations_from_stmt(finalizer);
                }
            }
            Stmt::Declaration { decl, .. } => {
                self.hoist_var_declarations_from_decl(decl);
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

    fn hoist_var_declarations_from_decl(&mut self, decl_id: lyng_js_ast::DeclId) {
        match self.ast.get_decl(decl_id).clone() {
            Decl::Variable {
                kind: VariableKind::Var,
                declarators,
                ..
            } => {
                let decls = self.ast.get_var_declarator_list(declarators).to_vec();
                for decl in decls {
                    self.hoist_var_pattern(decl.id, decl.span);
                }
            }
            Decl::Export {
                kind: lyng_js_ast::ExportKind::Declaration { decl },
                ..
            } => {
                self.hoist_var_declarations_from_decl(decl);
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
                    if !self
                        .scope_binding_names
                        .contains_key(&(self.ctx.current_scope, *n))
                    {
                        self.declare_binding(
                            *n,
                            DeclarationKind::Class,
                            self.ctx.current_scope,
                            *span,
                        );
                    }
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
