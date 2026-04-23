use lyng_js_ast::Pattern;
use lyng_js_common::Span;

use super::Analyzer;
use crate::binding::DeclarationKind;
use crate::ids::SemanticBindingId;

impl<'a> Analyzer<'a> {
    pub(crate) fn hoist_var_pattern(&mut self, pat_id: lyng_js_ast::PatternId, span: Span) {
        let pat = self.ast.get_pattern(pat_id);
        match pat {
            Pattern::Identifier { name, span, .. } => {
                let binding = self.declare_var_binding(*name, *span);
                self.record_pattern_binding(pat_id, binding);
            }
            Pattern::Object {
                properties, rest, ..
            } => {
                let props = self.ast.get_obj_pattern_prop_list(*properties);
                for prop in props {
                    self.hoist_var_pattern(prop.value, span);
                }
                if let Some(rest_id) = rest {
                    self.hoist_var_pattern(*rest_id, span);
                }
            }
            Pattern::Array { elements, rest, .. } => {
                let elems = self.ast.get_opt_pattern_elem_list(*elements);
                for elem in elems.iter().flatten() {
                    self.hoist_var_pattern(elem.pattern, span);
                }
                if let Some(rest_id) = rest {
                    self.hoist_var_pattern(*rest_id, span);
                }
            }
            Pattern::Assignment { left, .. } => {
                self.hoist_var_pattern(*left, span);
            }
            Pattern::InvalidPattern { .. } => {}
        }
    }

    pub(crate) fn declare_pattern_bindings(
        &mut self,
        pat_id: lyng_js_ast::PatternId,
        kind: DeclarationKind,
    ) {
        let pat = self.ast.get_pattern(pat_id);
        match pat {
            Pattern::Identifier { name, span, .. } => {
                if self
                    .pattern_bindings
                    .get(pat_id.raw() as usize)
                    .copied()
                    .flatten()
                    .is_some()
                {
                    return;
                }
                let binding = self.declare_binding(*name, kind, self.ctx.current_scope, *span);
                self.record_pattern_binding(pat_id, binding);
            }
            Pattern::Object {
                properties, rest, ..
            } => {
                let props = self.ast.get_obj_pattern_prop_list(*properties);
                for prop in props {
                    self.declare_pattern_bindings(prop.value, kind);
                }
                if let Some(rest_id) = rest {
                    self.declare_pattern_bindings(*rest_id, kind);
                }
            }
            Pattern::Array { elements, rest, .. } => {
                let elems = self.ast.get_opt_pattern_elem_list(*elements);
                for elem in elems.iter().flatten() {
                    self.declare_pattern_bindings(elem.pattern, kind);
                }
                if let Some(rest_id) = rest {
                    self.declare_pattern_bindings(*rest_id, kind);
                }
            }
            Pattern::Assignment { left, right: _, .. } => {
                self.declare_pattern_bindings(*left, kind);
            }
            Pattern::InvalidPattern { .. } => {}
        }
    }

    pub(crate) fn walk_binding_pattern_expressions(&mut self, pat_id: lyng_js_ast::PatternId) {
        let pat = self.ast.get_pattern(pat_id);
        match pat {
            Pattern::Identifier { .. } | Pattern::InvalidPattern { .. } => {}
            Pattern::Object {
                properties, rest, ..
            } => {
                let props = self.ast.get_obj_pattern_prop_list(*properties);
                for prop in props {
                    if prop.computed {
                        self.walk_expr(prop.key);
                    }
                    self.walk_binding_pattern_expressions(prop.value);
                }
                if let Some(rest_id) = rest {
                    self.walk_binding_pattern_expressions(*rest_id);
                }
            }
            Pattern::Array { elements, rest, .. } => {
                let elems = self.ast.get_opt_pattern_elem_list(*elements);
                for elem in elems.iter().flatten() {
                    self.walk_binding_pattern_expressions(elem.pattern);
                }
                if let Some(rest_id) = rest {
                    self.walk_binding_pattern_expressions(*rest_id);
                }
            }
            Pattern::Assignment { left, right, .. } => {
                self.walk_binding_pattern_expressions(*left);
                self.walk_expr(*right);
            }
        }
    }

    fn record_pattern_binding(
        &mut self,
        pattern: lyng_js_ast::PatternId,
        binding: SemanticBindingId,
    ) {
        let index = pattern.raw() as usize;
        if self.pattern_bindings.len() <= index {
            self.pattern_bindings.resize(index + 1, None);
        }
        self.pattern_bindings[index] = Some(binding);
    }

    pub(crate) fn walk_pattern(&mut self, pat_id: lyng_js_ast::PatternId) {
        let pat = self.ast.get_pattern(pat_id);
        match pat {
            Pattern::Identifier { name, span, .. } => {
                self.record_use(None, *name, *span);
            }
            Pattern::Object {
                properties, rest, ..
            } => {
                let props = self.ast.get_obj_pattern_prop_list(*properties);
                for prop in props {
                    if !prop.shorthand {
                        self.walk_expr(prop.key);
                    }
                    self.walk_pattern(prop.value);
                }
                if let Some(rest_id) = rest {
                    self.walk_pattern(*rest_id);
                }
            }
            Pattern::Array { elements, rest, .. } => {
                let elems = self.ast.get_opt_pattern_elem_list(*elements);
                for elem in elems.iter().flatten() {
                    self.walk_pattern(elem.pattern);
                }
                if let Some(rest_id) = rest {
                    self.walk_pattern(*rest_id);
                }
            }
            Pattern::Assignment { left, right, .. } => {
                self.walk_pattern(*left);
                self.walk_expr(*right);
            }
            Pattern::InvalidPattern { .. } => {}
        }
    }
}
