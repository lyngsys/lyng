use lyng_js_ast::{Decl, ForInOfLeft, ForInit, Stmt, VariableKind};
use lyng_js_common::WellKnownAtom;

use super::Analyzer;
use crate::binding::DeclarationKind;
use crate::scope::ScopeKind;

impl<'a> Analyzer<'a> {
    pub(super) fn walk_stmt_list(&mut self, list: lyng_js_ast::NodeList<lyng_js_ast::StmtId>) {
        let scope_kind = self.scopes.get(self.ctx.current_scope).kind;
        self.check_statement_list_redeclarations(list, scope_kind);
        let stmts = self.ast.get_stmt_list(list);
        for &stmt_id in stmts {
            self.hoist_declarations(stmt_id);
        }
        let stmts = self.ast.get_stmt_list(list);
        for &stmt_id in stmts {
            self.walk_stmt(stmt_id);
        }
    }

    fn walk_stmt(&mut self, stmt_id: lyng_js_ast::StmtId) {
        let stmt = self.ast.get_stmt(stmt_id);
        match stmt {
            Stmt::Block { body, .. } => {
                self.push_scope(ScopeKind::Block);
                self.walk_stmt_list(*body);
                self.pop_scope();
            }
            Stmt::Empty { .. } | Stmt::Debugger { .. } => {}
            Stmt::Expression { expression, .. } => {
                self.walk_expr(*expression);
            }
            Stmt::If {
                test,
                consequent,
                alternate,
                ..
            } => {
                self.walk_expr(*test);
                self.walk_stmt(*consequent);
                if let Some(alt) = alternate {
                    self.walk_stmt(*alt);
                }
            }
            Stmt::DoWhile { body, test, .. } => {
                let old_in_loop = self.ctx.in_loop;
                self.ctx.in_loop = true;
                self.walk_stmt(*body);
                self.ctx.in_loop = old_in_loop;
                self.walk_expr(*test);
            }
            Stmt::While { test, body, .. } => {
                self.walk_expr(*test);
                let old_in_loop = self.ctx.in_loop;
                self.ctx.in_loop = true;
                self.walk_stmt(*body);
                self.ctx.in_loop = old_in_loop;
            }
            Stmt::For {
                init,
                test,
                update,
                body,
                ..
            } => {
                let has_lexical_init = matches!(init, Some(ForInit::Declaration(decl_id)) if {
                    let decl = self.ast.get_decl(*decl_id);
                    matches!(
                        decl,
                        Decl::Variable {
                            kind:
                                VariableKind::Let
                                | VariableKind::Const
                                | VariableKind::Using
                                | VariableKind::AwaitUsing,
                            ..
                        }
                    )
                });

                if has_lexical_init {
                    self.push_scope(ScopeKind::ForLoop);
                }

                if let Some(fi) = init {
                    match fi {
                        ForInit::Declaration(decl_id) => self.walk_decl(*decl_id),
                        ForInit::Expression(expr_id) => self.walk_expr(*expr_id),
                    }
                }
                if let Some(t) = test {
                    self.walk_expr(*t);
                }
                if let Some(u) = update {
                    self.walk_expr(*u);
                }

                let old_in_loop = self.ctx.in_loop;
                self.ctx.in_loop = true;
                self.walk_stmt(*body);
                self.ctx.in_loop = old_in_loop;

                if has_lexical_init {
                    self.pop_scope();
                }
            }
            Stmt::ForIn {
                left, right, body, ..
            }
            | Stmt::ForOf {
                left, right, body, ..
            } => {
                let has_lexical = self.for_in_of_has_lexical(left);
                if has_lexical {
                    self.push_scope(ScopeKind::ForLoop);
                }
                self.walk_for_in_of_left(left);
                self.walk_expr(*right);
                let old_in_loop = self.ctx.in_loop;
                self.ctx.in_loop = true;
                self.walk_stmt(*body);
                self.ctx.in_loop = old_in_loop;
                if has_lexical {
                    self.pop_scope();
                }
            }
            Stmt::Continue { label, span, .. } => {
                if let Some(lbl) = label {
                    if !self.ctx.loop_labels.contains(lbl) {
                        self.diagnostics
                            .error(*span, "continue with label not targeting a loop");
                    }
                } else if !self.ctx.in_loop {
                    self.diagnostics.error(*span, "'continue' outside of loop");
                }
            }
            Stmt::Break { label, span, .. } => {
                if let Some(lbl) = label {
                    if !self.ctx.labels.contains(lbl) && !self.ctx.loop_labels.contains(lbl) {
                        self.diagnostics.error(*span, "break with undefined label");
                    }
                } else if !self.ctx.in_loop && !self.ctx.in_switch {
                    self.diagnostics
                        .error(*span, "'break' outside of loop or switch");
                }
            }
            Stmt::Return { span, argument, .. } => {
                if !self.ctx.in_function {
                    self.diagnostics
                        .error(*span, "'return' outside of function");
                }
                if let Some(arg) = argument {
                    self.walk_expr(*arg);
                }
            }
            Stmt::With {
                object, body, span, ..
            } => {
                if self.ctx.strict {
                    self.diagnostics
                        .error(*span, "'with' statement not allowed in strict mode");
                }
                self.walk_expr(*object);
                self.scopes.get_mut(self.ctx.current_scope).has_with = true;
                self.push_scope(ScopeKind::With);
                self.walk_stmt(*body);
                self.pop_scope();
            }
            Stmt::Switch {
                discriminant,
                cases,
                ..
            } => {
                self.walk_expr(*discriminant);
                self.check_switch_case_redeclarations(*cases);
                self.push_scope(ScopeKind::Switch);
                self.predeclare_switch_case_bindings(*cases);
                let old_in_switch = self.ctx.in_switch;
                self.ctx.in_switch = true;
                let case_list = self.ast.get_switch_case_list(*cases);
                for case in case_list {
                    if let Some(test) = case.test {
                        self.walk_expr(test);
                    }
                    self.walk_stmt_list(case.consequent);
                }
                self.ctx.in_switch = old_in_switch;
                self.pop_scope();
            }
            Stmt::Labeled {
                label, body, span, ..
            } => {
                if self.ctx.in_static_block
                    && self.ctx.current_function.is_none()
                    && *label == WellKnownAtom::r#await.id()
                {
                    self.diagnostics.error(
                        *span,
                        "'await' cannot be used as a label inside a class static block",
                    );
                }
                if self.ctx.labels.contains(label) {
                    let name = self.atoms.resolve(*label);
                    self.diagnostics
                        .error(*span, format!("duplicate label '{name}'"));
                }
                let is_loop = self.stmt_is_loop(*body);
                self.ctx.labels.push(*label);
                if is_loop {
                    self.ctx.loop_labels.push(*label);
                }
                self.walk_stmt(*body);
                self.ctx.labels.pop();
                if is_loop {
                    self.ctx.loop_labels.pop();
                }
            }
            Stmt::Throw { argument, .. } => {
                self.walk_expr(*argument);
            }
            Stmt::Try {
                block,
                handler,
                finalizer,
                ..
            } => {
                self.walk_stmt(*block);
                if let Some(catch) = handler {
                    self.check_catch_clause_early_errors(catch);
                    self.push_scope(ScopeKind::Catch);
                    if let Some(param) = catch.param {
                        self.declare_pattern_bindings(param, DeclarationKind::CatchParam);
                        self.walk_binding_pattern_expressions(param);
                    }
                    self.walk_stmt(catch.body);
                    self.pop_scope();
                }
                if let Some(fin) = finalizer {
                    self.walk_stmt(*fin);
                }
            }
            Stmt::Declaration { decl, .. } => {
                self.walk_decl(*decl);
            }
            Stmt::InvalidStatement { .. } => {}
        }
    }

    fn stmt_is_loop(&self, stmt_id: lyng_js_ast::StmtId) -> bool {
        let stmt = self.ast.get_stmt(stmt_id);
        matches!(
            stmt,
            Stmt::While { .. }
                | Stmt::DoWhile { .. }
                | Stmt::For { .. }
                | Stmt::ForIn { .. }
                | Stmt::ForOf { .. }
        )
    }

    fn for_in_of_has_lexical(&self, left: &ForInOfLeft) -> bool {
        match left {
            ForInOfLeft::Declaration(decl_id) => {
                let d = self.ast.get_decl(*decl_id);
                matches!(
                    d,
                    Decl::Variable {
                        kind: VariableKind::Let
                            | VariableKind::Const
                            | VariableKind::Using
                            | VariableKind::AwaitUsing,
                        ..
                    }
                )
            }
            _ => false,
        }
    }

    fn walk_for_in_of_left(&mut self, left: &ForInOfLeft) {
        match left {
            ForInOfLeft::Declaration(decl_id) => self.walk_decl(*decl_id),
            ForInOfLeft::Pattern(pat_id) => self.walk_pattern(*pat_id),
            ForInOfLeft::Expression(expr_id) => {
                if self.expr_is_destructuring_pattern(*expr_id) {
                    self.walk_assignment_target_expr(*expr_id);
                } else {
                    self.walk_expr(*expr_id);
                }
            }
        }
    }
}
