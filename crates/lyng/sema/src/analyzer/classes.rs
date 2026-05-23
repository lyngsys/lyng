use std::collections::HashMap;

use lyng_js_ast::{ClassElement, Expr, FunctionKind, MethodKind};
use lyng_js_common::{AtomId, Span, WellKnownAtom};

use super::{Analyzer, ContainmentQuery, PrivateNameUsage};
use crate::class_private_layout::{
    ClassPrivateElementKind, ClassPrivateElementRecord, ClassPrivateLayoutRecord,
};
use crate::ids::ScopeId;
use crate::private_use::PrivateUseRecord;
use crate::scope::ScopeKind;

impl Analyzer<'_> {
    fn class_element_name_is(&self, key: lyng_js_ast::ExprId, atom: WellKnownAtom) -> bool {
        match self.ast.get_expr(key) {
            Expr::Identifier { name: actual, .. } => *actual == atom.id(),
            Expr::StringLiteral { value, .. } => {
                self.ast.literals().get_string(*value) == self.atoms.resolve(atom.id())
            }
            _ => false,
        }
    }

    pub(super) fn expr_is_private_member_reference(&self, expr_id: lyng_js_ast::ExprId) -> bool {
        match self.ast.get_expr(expr_id) {
            Expr::PrivateMemberExpression { .. } => true,
            Expr::ParenthesizedExpression { expression, .. } => {
                self.expr_is_private_member_reference(*expression)
            }
            _ => false,
        }
    }

    pub(super) fn walk_class_body(
        &mut self,
        body: lyng_js_ast::NodeList<lyng_js_ast::ClassElementId>,
        span: Span,
        class_name: Option<AtomId>,
        super_class: Option<lyng_js_ast::ExprId>,
        has_heritage: bool,
    ) {
        let scope_id = self.push_scope(ScopeKind::ClassBody);
        let old_strict = self.ctx.strict;
        self.ctx.strict = true;
        self.scopes.get_mut(scope_id).strict = true;
        if let Some(class_name) = class_name {
            self.declare_binding(
                class_name,
                crate::DeclarationKind::ClassName,
                scope_id,
                span,
            );
        }
        if let Some(super_class) = super_class {
            self.walk_expr(super_class);
        }
        self.ctx.class_scopes.push(scope_id);
        let mut private_name_usages = HashMap::<AtomId, PrivateNameUsage>::new();
        let mut private_layout_entries = Vec::new();
        let mut constructor_seen = false;

        let elements = self.ast.get_class_element_list(body);
        for &elem_id in elements {
            let elem = self.ast.get_class_element(elem_id);
            match elem {
                ClassElement::Method {
                    key,
                    value,
                    computed,
                    private,
                    span,
                    kind,
                    r#static,
                    ..
                } => {
                    if !computed && *private {
                        if let Expr::Identifier { name, .. } = self.ast.get_expr(*key) {
                            private_layout_entries.push(ClassPrivateElementRecord::new(
                                *name,
                                *r#static,
                                match kind {
                                    MethodKind::Get => ClassPrivateElementKind::Getter,
                                    MethodKind::Set => ClassPrivateElementKind::Setter,
                                    MethodKind::Method | MethodKind::Constructor => {
                                        ClassPrivateElementKind::Method
                                    }
                                },
                                *span,
                            ));
                            if *name == WellKnownAtom::constructor.id() {
                                self.diagnostics.error(
                                    *span,
                                    "private class elements cannot be named '#constructor'",
                                );
                            }
                            if !private_name_usages.contains_key(name) {
                                self.private_names
                                    .alloc(crate::private_name::PrivateNameRecord {
                                        name: *name,
                                        scope: scope_id,
                                        span: *span,
                                    });
                            }
                            let usage = private_name_usages.entry(*name).or_default();
                            let duplicate = match kind {
                                MethodKind::Get => {
                                    let duplicate = usage.getter_static.is_some()
                                        || usage.other
                                        || usage.setter_static.is_some_and(|s| s != *r#static);
                                    usage.getter_static = Some(*r#static);
                                    duplicate
                                }
                                MethodKind::Set => {
                                    let duplicate = usage.setter_static.is_some()
                                        || usage.other
                                        || usage.getter_static.is_some_and(|s| s != *r#static);
                                    usage.setter_static = Some(*r#static);
                                    duplicate
                                }
                                MethodKind::Method | MethodKind::Constructor => {
                                    let duplicate = usage.getter_static.is_some()
                                        || usage.setter_static.is_some()
                                        || usage.other;
                                    usage.other = true;
                                    duplicate
                                }
                            };
                            if duplicate {
                                self.diagnostics.error(*span, "duplicate private name");
                            }
                        }
                    } else if !computed {
                        let is_constructor =
                            self.class_element_name_is(*key, WellKnownAtom::constructor);
                        let is_prototype =
                            self.class_element_name_is(*key, WellKnownAtom::prototype);
                        if !r#static && is_constructor {
                            let func = self.ast.get_function(*value);
                            if *kind == MethodKind::Constructor
                                || (*kind == MethodKind::Method
                                    && func.kind == FunctionKind::Normal)
                            {
                                if constructor_seen {
                                    self.diagnostics.error(*span, "duplicate constructor");
                                }
                                constructor_seen = true;
                            } else {
                                self.diagnostics
                                    .error(*span, "special methods cannot be named 'constructor'");
                            }
                        }
                        if *r#static && is_prototype {
                            self.diagnostics
                                .error(*span, "static class elements cannot be named 'prototype'");
                        }
                    }
                }
                ClassElement::Property {
                    key,
                    computed,
                    private,
                    span,
                    r#static,
                    auto_accessor_private_name,
                    ..
                } => {
                    if let Some(backing_name) = auto_accessor_private_name {
                        private_layout_entries.push(ClassPrivateElementRecord::new(
                            *backing_name,
                            *r#static,
                            ClassPrivateElementKind::Field,
                            *span,
                        ));
                    }
                    if !computed && *private {
                        if let Expr::Identifier { name, .. } = self.ast.get_expr(*key) {
                            private_layout_entries.push(ClassPrivateElementRecord::new(
                                *name,
                                *r#static,
                                ClassPrivateElementKind::Field,
                                *span,
                            ));
                            if *name == WellKnownAtom::constructor.id() {
                                self.diagnostics.error(
                                    *span,
                                    "private class elements cannot be named '#constructor'",
                                );
                            }
                            if !private_name_usages.contains_key(name) {
                                self.private_names
                                    .alloc(crate::private_name::PrivateNameRecord {
                                        name: *name,
                                        scope: scope_id,
                                        span: *span,
                                    });
                            }
                            let usage = private_name_usages.entry(*name).or_default();
                            let duplicate = usage.getter_static.is_some()
                                || usage.setter_static.is_some()
                                || usage.other;
                            usage.other = true;
                            if duplicate {
                                self.diagnostics.error(*span, "duplicate private name");
                            }
                        }
                    } else if !computed {
                        if self.class_element_name_is(*key, WellKnownAtom::constructor) {
                            self.diagnostics
                                .error(*span, "class fields cannot be named 'constructor'");
                        }
                        if *r#static && self.class_element_name_is(*key, WellKnownAtom::prototype) {
                            self.diagnostics
                                .error(*span, "static class elements cannot be named 'prototype'");
                        }
                    }
                }
                _ => {}
            }
        }

        self.class_private_layouts
            .alloc(ClassPrivateLayoutRecord::new(
                body,
                span,
                scope_id,
                private_layout_entries,
            ));

        for &elem_id in elements {
            let elem = self.ast.get_class_element(elem_id);
            match elem {
                ClassElement::Method {
                    key,
                    value,
                    computed,
                    kind,
                    private,
                    r#static,
                    span,
                    ..
                } => {
                    if *computed {
                        self.walk_expr(*key);
                    }
                    let func = self.ast.get_function(*value);
                    let param_len = self.ast.get_pattern_list(func.params.params).len();
                    if *kind == MethodKind::Get && (param_len != 0 || func.params.rest.is_some()) {
                        self.diagnostics
                            .error(*span, "getter methods must not declare parameters");
                    }
                    if *kind == MethodKind::Set && (param_len != 1 || func.params.rest.is_some()) {
                        self.diagnostics.error(
                            *span,
                            "setter methods must declare exactly one non-rest parameter",
                        );
                    }
                    let is_constructor = !*private
                        && !*computed
                        && !*r#static
                        && self.class_element_name_is(*key, WellKnownAtom::constructor)
                        && func.kind == FunctionKind::Normal;
                    if self.function_body_contains_query(*value, ContainmentQuery::DirectSuperCall)
                        && !(is_constructor && has_heritage)
                    {
                        self.diagnostics.error(
                            *span,
                            "direct super() is only valid in derived constructors",
                        );
                    }
                    self.walk_function(*value);
                }
                ClassElement::Property {
                    key,
                    value,
                    computed,
                    span,
                    ..
                } => {
                    if *computed {
                        self.walk_expr(*key);
                    }
                    if let Some(v) = value {
                        if self.expr_contains_query(*v, ContainmentQuery::ArgumentsIdentifier) {
                            self.diagnostics
                                .error(*span, "class field initializer cannot contain 'arguments'");
                        }
                        if self.expr_contains_query(*v, ContainmentQuery::DirectSuperCall) {
                            self.diagnostics.error(
                                *span,
                                "class field initializer cannot contain direct super()",
                            );
                        }
                        self.walk_class_field_initializer(*v);
                    }
                }
                ClassElement::StaticBlock { body, .. } => {
                    let old_ctx = (
                        self.ctx.current_function,
                        self.ctx.in_function,
                        self.ctx.in_loop,
                        self.ctx.in_switch,
                        self.ctx.in_static_block,
                    );
                    let old_labels = std::mem::take(&mut self.ctx.labels);
                    let old_loop_labels = std::mem::take(&mut self.ctx.loop_labels);
                    self.push_scope(ScopeKind::Function);
                    self.ctx.current_function = None;
                    self.ctx.in_function = false;
                    self.ctx.in_loop = false;
                    self.ctx.in_switch = false;
                    self.ctx.in_static_block = true;
                    self.walk_stmt_list(*body);
                    self.ctx.current_function = old_ctx.0;
                    self.ctx.in_function = old_ctx.1;
                    self.ctx.in_loop = old_ctx.2;
                    self.ctx.in_switch = old_ctx.3;
                    self.ctx.in_static_block = old_ctx.4;
                    self.ctx.labels = old_labels;
                    self.ctx.loop_labels = old_loop_labels;
                    self.pop_scope();
                }
                ClassElement::InvalidElement { .. } => {}
            }
        }

        self.ctx.class_scopes.pop();
        self.ctx.strict = old_strict;
        self.pop_scope();
    }

    pub(super) fn is_private_name_defined(&self, name: AtomId) -> bool {
        self.resolve_private_name(name).is_some()
    }

    pub(super) fn resolve_private_name(&self, name: AtomId) -> Option<(ScopeId, u16)> {
        for (depth, &class_scope_id) in self.ctx.class_scopes.iter().rev().enumerate() {
            if self
                .private_names
                .as_slice()
                .iter()
                .any(|pn| pn.name == name && pn.scope == class_scope_id)
            {
                return Some((class_scope_id, u16::try_from(depth).unwrap_or(u16::MAX)));
            }
        }
        None
    }

    pub(super) fn record_private_use(&mut self, expr: lyng_js_ast::ExprId, name: AtomId) {
        if let Some((defining_scope, class_depth)) = self.resolve_private_name(name) {
            self.private_uses.alloc(PrivateUseRecord::new(
                expr,
                name,
                defining_scope,
                class_depth,
            ));
        }
    }

    fn walk_class_field_initializer(&mut self, expr_id: lyng_js_ast::ExprId) {
        let old_ctx = (
            self.ctx.current_function,
            self.ctx.in_function,
            self.ctx.in_loop,
            self.ctx.in_switch,
            self.ctx.in_static_block,
        );
        let old_labels = std::mem::take(&mut self.ctx.labels);
        let old_loop_labels = std::mem::take(&mut self.ctx.loop_labels);
        self.push_scope(ScopeKind::Function);
        self.ctx.current_function = None;
        self.ctx.in_function = false;
        self.ctx.in_loop = false;
        self.ctx.in_switch = false;
        self.ctx.in_static_block = false;
        self.walk_expr(expr_id);
        self.ctx.current_function = old_ctx.0;
        self.ctx.in_function = old_ctx.1;
        self.ctx.in_loop = old_ctx.2;
        self.ctx.in_switch = old_ctx.3;
        self.ctx.in_static_block = old_ctx.4;
        self.ctx.labels = old_labels;
        self.ctx.loop_labels = old_loop_labels;
        self.pop_scope();
    }
}
