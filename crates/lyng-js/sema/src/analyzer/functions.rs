use lyng_js_ast::{FunctionKind, Pattern};
use lyng_js_common::{AtomId, Span};

use super::{Analyzer, ContainmentQuery};
use crate::binding::DeclarationKind;
use crate::function_sema::FunctionSemaRecord;
use crate::scope::ScopeKind;

impl Analyzer<'_> {
    pub(super) fn walk_function(&mut self, func_id: lyng_js_ast::FunctionId) {
        let func = self.ast.get_function(func_id);
        let func_span = func.span;
        let func_name = func.name;
        let func_kind = func.kind;
        let params = func.params.clone();
        let body = func.body;
        let expression_body = func.expression_body;

        let has_non_simple_params = self.has_non_simple_params(&params);
        let strict_directive_span = if expression_body.is_none() {
            self.strict_directive_span(body)
        } else {
            None
        };
        let function_strict = self.ctx.strict || strict_directive_span.is_some();

        if let Some(directive_span) = strict_directive_span {
            if has_non_simple_params {
                self.diagnostics.error(
                    directive_span,
                    "'use strict' directive not allowed in function with non-simple parameter list",
                );
            }
        }

        if function_strict || has_non_simple_params {
            self.check_duplicate_params(&params);
        }
        if has_non_simple_params && expression_body.is_none() {
            self.check_body_lexical_redeclarations_of_parameters(&params, body);
        }

        if matches!(
            func_kind,
            FunctionKind::Generator | FunctionKind::AsyncGenerator
        ) && (self
            .ast
            .get_pattern_list(params.params)
            .iter()
            .any(|&param| self.pattern_contains_query(param, ContainmentQuery::YieldExpression))
            || params.rest.map_or(false, |rest| {
                self.pattern_contains_query(rest, ContainmentQuery::YieldExpression)
            }))
        {
            self.diagnostics.error(
                func_span,
                "yield expressions are not allowed in generator parameters",
            );
        }

        let old_ctx = (
            self.ctx.current_function,
            self.ctx.in_function,
            self.ctx.in_loop,
            self.ctx.in_switch,
            self.ctx.strict,
            self.ctx.in_static_block,
        );
        self.ctx.strict = function_strict;

        let func_scope = self.push_scope(ScopeKind::Function);
        let func_sema_id = self.functions.alloc(FunctionSemaRecord {
            function_id: func_id,
            strict: function_strict,
            scope_root: func_scope,
            param_scope: None,
            needs_environment: false,
            has_eval: false,
            has_with: false,
            needs_arguments: false,
            references_super: false,
            references_new_target: false,
            references_this: false,
            has_await: false,
            has_yield: false,
            captures: Vec::new(),
        });

        self.scopes.get_mut(func_scope).owning_function = Some(func_sema_id);
        self.ctx.current_function = Some(func_sema_id);
        self.ctx.in_function = true;
        self.ctx.in_loop = false;
        self.ctx.in_switch = false;
        self.ctx.in_static_block = false;

        let old_labels = std::mem::take(&mut self.ctx.labels);
        let old_loop_labels = std::mem::take(&mut self.ctx.loop_labels);

        let mut has_named_function_expression_scope = false;
        if let Some(name) = func_name {
            let parent_scope = self.scopes.get(func_scope).parent;
            let is_declaration = parent_scope.map_or(false, |ps| {
                self.scopes.get(ps).bindings.iter().any(|&bid| {
                    let b = self.bindings.get(bid);
                    b.name == name && b.kind == DeclarationKind::Function
                })
            });

            if !is_declaration && !self.suppressed_function_name_bindings.contains(&func_id) {
                self.declare_binding(name, DeclarationKind::FunctionName, func_scope, func_span);
                has_named_function_expression_scope = true;
            }
        }

        if has_non_simple_params {
            let param_scope = self.push_scope(ScopeKind::Parameter);
            self.functions.get_mut(func_sema_id).param_scope = Some(param_scope);
            self.scopes.get_mut(param_scope).owning_function = Some(func_sema_id);

            self.declare_params(&params);

            let body_scope = self.push_scope(ScopeKind::Function);
            self.scopes.get_mut(body_scope).owning_function = Some(func_sema_id);
            self.functions.get_mut(func_sema_id).scope_root = body_scope;

            if let Some(expr_body) = expression_body {
                self.walk_expr(expr_body);
            } else {
                self.walk_stmt_list(body);
            }

            self.pop_scope();
            self.pop_scope();
        } else if has_named_function_expression_scope {
            let body_scope = self.push_scope(ScopeKind::Function);
            self.scopes.get_mut(body_scope).owning_function = Some(func_sema_id);
            self.functions.get_mut(func_sema_id).scope_root = body_scope;

            self.declare_params(&params);

            if let Some(expr_body) = expression_body {
                self.walk_expr(expr_body);
            } else {
                self.walk_stmt_list(body);
            }

            self.pop_scope();
        } else {
            self.declare_params(&params);

            if let Some(expr_body) = expression_body {
                self.walk_expr(expr_body);
            } else {
                self.walk_stmt_list(body);
            }
        }

        self.ctx.labels = old_labels;
        self.ctx.loop_labels = old_loop_labels;
        self.ctx.current_function = old_ctx.0;
        self.ctx.in_function = old_ctx.1;
        self.ctx.in_loop = old_ctx.2;
        self.ctx.in_switch = old_ctx.3;
        self.ctx.strict = old_ctx.4;
        self.ctx.in_static_block = old_ctx.5;

        self.pop_scope();
    }

    fn declare_params(&mut self, params: &lyng_js_ast::FormalParameters) {
        let param_ids = self.ast.get_pattern_list(params.params);
        for &pid in param_ids {
            self.declare_pattern_bindings(pid, DeclarationKind::Parameter);
            self.walk_binding_pattern_expressions(pid);
        }
        if let Some(rest) = params.rest {
            self.declare_pattern_bindings(rest, DeclarationKind::Parameter);
            self.walk_binding_pattern_expressions(rest);
        }
    }

    fn has_non_simple_params(&self, params: &lyng_js_ast::FormalParameters) -> bool {
        if params.rest.is_some() {
            return true;
        }
        let param_ids = self.ast.get_pattern_list(params.params);
        for &pid in param_ids {
            let pat = self.ast.get_pattern(pid);
            if !matches!(pat, Pattern::Identifier { .. }) {
                return true;
            }
        }
        false
    }

    pub(super) fn check_duplicate_params(&mut self, params: &lyng_js_ast::FormalParameters) {
        let mut seen = Vec::new();
        let param_ids = self.ast.get_pattern_list(params.params);
        for &pid in param_ids {
            self.collect_pattern_names(pid, &mut seen);
        }
        if let Some(rest) = params.rest {
            self.collect_pattern_names(rest, &mut seen);
        }
        let mut checked = Vec::new();
        for &(name, span) in &seen {
            if checked.contains(&name) {
                self.diagnostics.error(
                    span,
                    "duplicate parameter name in strict mode or with non-simple parameters",
                );
            } else {
                checked.push(name);
            }
        }
    }

    fn check_body_lexical_redeclarations_of_parameters(
        &mut self,
        params: &lyng_js_ast::FormalParameters,
        body: lyng_js_ast::NodeList<lyng_js_ast::StmtId>,
    ) {
        let mut parameter_names = Vec::new();
        let param_ids = self.ast.get_pattern_list(params.params);
        for &pid in param_ids {
            self.collect_pattern_names(pid, &mut parameter_names);
        }
        if let Some(rest) = params.rest {
            self.collect_pattern_names(rest, &mut parameter_names);
        }

        let mut lexical_names = Vec::new();
        self.collect_lexically_declared_names_from_stmt_list(
            body,
            ScopeKind::Function,
            &mut lexical_names,
        );

        let mut reported = Vec::new();
        for entry in lexical_names {
            if parameter_names.iter().any(|&(name, _)| name == entry.name)
                && !reported.contains(&entry.name)
            {
                reported.push(entry.name);
                self.diagnostics.error(
                    entry.span,
                    "function body lexical declaration conflicts with parameter",
                );
            }
        }
    }

    pub(super) fn collect_pattern_names(
        &self,
        pat_id: lyng_js_ast::PatternId,
        out: &mut Vec<(AtomId, Span)>,
    ) {
        let pat = self.ast.get_pattern(pat_id);
        match pat {
            Pattern::Identifier { name, span, .. } => {
                out.push((*name, *span));
            }
            Pattern::Object {
                properties, rest, ..
            } => {
                let props = self.ast.get_obj_pattern_prop_list(*properties);
                for prop in props {
                    self.collect_pattern_names(prop.value, out);
                }
                if let Some(rest_id) = rest {
                    self.collect_pattern_names(*rest_id, out);
                }
            }
            Pattern::Array { elements, rest, .. } => {
                let elems = self.ast.get_opt_pattern_elem_list(*elements);
                for elem in elems.iter().flatten() {
                    self.collect_pattern_names(elem.pattern, out);
                }
                if let Some(rest_id) = rest {
                    self.collect_pattern_names(*rest_id, out);
                }
            }
            Pattern::Assignment { left, .. } => {
                self.collect_pattern_names(*left, out);
            }
            Pattern::InvalidPattern { .. } => {}
        }
    }
}
