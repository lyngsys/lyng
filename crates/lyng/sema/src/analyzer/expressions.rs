use lyng_js_ast::{AssignOp, Expr, NumericLiteralSyntax, PropertyKind, UnaryOp};
use lyng_js_common::WellKnownAtom;

use super::{Analyzer, ContainmentQuery};

impl Analyzer<'_> {
    pub(super) fn walk_expr(&mut self, expr_id: lyng_js_ast::ExprId) {
        let expr = self.ast.get_expr(expr_id);
        match expr {
            Expr::Identifier { name, span, .. } => {
                if self.ctx.in_static_block
                    && self.ctx.current_function.is_none()
                    && (*name == WellKnownAtom::arguments.id()
                        || *name == WellKnownAtom::r#await.id())
                {
                    self.diagnostics.error(
                        *span,
                        if *name == WellKnownAtom::arguments.id() {
                            "'arguments' cannot be used inside a class static block"
                        } else {
                            "'await' cannot be used inside a class static block"
                        },
                    );
                }
                self.record_use(Some(expr_id), *name, *span);
            }

            Expr::This { .. } => {
                if let Some(func_id) = self.ctx.current_function {
                    self.functions.get_mut(func_id).references_this = true;
                }
            }

            Expr::Super { span, .. } => {
                if let Some(func_id) = self.ctx.current_function {
                    self.functions.get_mut(func_id).references_super = true;
                } else if self.ctx.in_static_block {
                } else {
                    self.diagnostics
                        .error(*span, "'super' keyword outside of a method");
                }
            }

            Expr::FunctionExpression { function, .. } => {
                if self.function_body_contains_query(*function, ContainmentQuery::SuperKeyword) {
                    self.diagnostics
                        .error(expr.span(), "'super' keyword outside of a method");
                }
                self.walk_function(*function);
            }

            Expr::ArrowFunctionExpression { function, .. } => {
                self.walk_function(*function);
            }

            Expr::ClassExpression {
                name,
                super_class,
                body,
                span,
                ..
            } => {
                let old_strict = self.ctx.strict;
                self.ctx.strict = true;
                self.walk_class_body(*body, *span, *name, *super_class, super_class.is_some());
                self.ctx.strict = old_strict;
            }

            Expr::ArrayExpression { elements, .. } => {
                let elems = self.ast.get_opt_expr_list(*elements);
                for elem in elems.iter().flatten() {
                    self.walk_expr(*elem);
                }
            }

            Expr::ObjectExpression { properties, .. } => {
                let props = self.ast.get_property_list(*properties);
                self.validate_object_literal_properties(props);
                for prop in props {
                    if prop.computed {
                        self.walk_expr(prop.key);
                    } else if prop.shorthand {
                        self.walk_expr(prop.value);
                        continue;
                    }
                    if prop.method || prop.kind != PropertyKind::Init {
                        if let Expr::FunctionExpression { function, .. } =
                            self.ast.get_expr(prop.value)
                        {
                            let func = self.ast.get_function(*function);
                            let param_len = self.ast.get_pattern_list(func.params.params).len();
                            if prop.kind == PropertyKind::Get
                                && (param_len != 0 || func.params.rest.is_some())
                            {
                                self.diagnostics
                                    .error(prop.span, "getter methods must not declare parameters");
                            }
                            if prop.kind == PropertyKind::Set
                                && (param_len != 1 || func.params.rest.is_some())
                            {
                                self.diagnostics.error(
                                    prop.span,
                                    "setter methods must declare exactly one non-rest parameter",
                                );
                            }
                            if self.function_body_contains_query(
                                *function,
                                ContainmentQuery::DirectSuperCall,
                            ) {
                                self.diagnostics.error(
                                    prop.span,
                                    "direct super() is not allowed in object methods",
                                );
                            }
                            self.check_duplicate_params(&func.params);
                            self.walk_function(*function);
                            continue;
                        }
                    }
                    self.walk_expr(prop.value);
                }
            }

            Expr::UnaryExpression {
                operator,
                argument,
                span,
            } => {
                if *operator == UnaryOp::Delete
                    && self.ctx.strict
                    && self.expr_is_private_member_reference(*argument)
                {
                    self.diagnostics.error(
                        *span,
                        "cannot delete a private class element in strict mode",
                    );
                }
                self.walk_expr(*argument);
            }

            Expr::UpdateExpression { argument, .. }
            | Expr::AwaitExpression { argument, .. }
            | Expr::SpreadElement { argument, .. } => {
                if let Expr::AwaitExpression { .. } = expr {
                    if self.ctx.in_static_block && self.ctx.current_function.is_none() {
                        self.diagnostics.error(
                            expr.span(),
                            "'await' is not allowed in a class static block",
                        );
                    } else if let Some(func_id) = self.ctx.current_function {
                        self.functions.get_mut(func_id).has_await = true;
                    }
                }
                self.walk_expr(*argument);
            }

            Expr::YieldExpression { argument, .. } => {
                if self.ctx.in_static_block && self.ctx.current_function.is_none() {
                    self.diagnostics.error(
                        expr.span(),
                        "'yield' is not allowed in a class static block",
                    );
                } else if let Some(func_id) = self.ctx.current_function {
                    self.functions.get_mut(func_id).has_yield = true;
                }
                if let Some(arg) = argument {
                    self.walk_expr(*arg);
                }
            }

            Expr::BinaryExpression { left, right, .. }
            | Expr::LogicalExpression { left, right, .. } => {
                self.walk_expr(*left);
                self.walk_expr(*right);
            }

            Expr::AssignmentExpression {
                left,
                right,
                span,
                operator,
                ..
            } => {
                if self.ctx.strict {
                    if let Expr::Identifier { name, .. } = self.ast.get_expr(*left) {
                        if *name == WellKnownAtom::eval.id() {
                            self.diagnostics
                                .error(*span, "assignment to 'eval' in strict mode");
                        } else if *name == WellKnownAtom::arguments.id() {
                            self.diagnostics
                                .error(*span, "assignment to 'arguments' in strict mode");
                        }
                    }
                }
                if *operator == AssignOp::Assign {
                    self.walk_assignment_target_expr(*left);
                } else {
                    self.walk_expr(*left);
                }
                self.walk_expr(*right);
            }

            Expr::ConditionalExpression {
                test,
                consequent,
                alternate,
                ..
            } => {
                self.walk_expr(*test);
                self.walk_expr(*consequent);
                self.walk_expr(*alternate);
            }

            Expr::CallExpression {
                callee, arguments, ..
            } => {
                let mut direct_eval_callee = *callee;
                while let Expr::ParenthesizedExpression { expression, .. } =
                    self.ast.get_expr(direct_eval_callee)
                {
                    direct_eval_callee = *expression;
                }
                if let Expr::Identifier { name, .. } = self.ast.get_expr(direct_eval_callee) {
                    if *name == WellKnownAtom::eval.id() {
                        self.scopes.get_mut(self.ctx.current_scope).has_eval = true;
                        if let Some(func_id) = self.ctx.current_function {
                            self.functions.get_mut(func_id).has_eval = true;
                        }
                    }
                }
                if self.ctx.in_static_block
                    && self.ctx.current_function.is_none()
                    && matches!(self.ast.get_expr(*callee), Expr::Super { .. })
                {
                    self.diagnostics.error(
                        expr.span(),
                        "direct super() is not allowed in a class static block",
                    );
                }
                self.walk_expr(*callee);
                let args = self.ast.get_expr_list(*arguments);
                for &arg in args {
                    self.walk_expr(arg);
                }
            }

            Expr::NewExpression {
                callee, arguments, ..
            } => {
                self.walk_expr(*callee);
                let args = self.ast.get_expr_list(*arguments);
                for &arg in args {
                    self.walk_expr(arg);
                }
            }

            Expr::StaticMemberExpression {
                object, property, ..
            } => {
                if *property == WellKnownAtom::arguments.id() {
                    if let Some(func_id) = self.ctx.current_function {
                        self.functions.get_mut(func_id).needs_arguments = true;
                    }
                }
                self.walk_expr(*object);
            }

            Expr::ComputedMemberExpression {
                object, property, ..
            } => {
                self.walk_expr(*object);
                self.walk_expr(*property);
            }

            Expr::PrivateMemberExpression {
                object,
                property,
                span,
                ..
            } => {
                if matches!(self.ast.get_expr(*object), Expr::Super { .. }) {
                    self.diagnostics
                        .error(*span, "private names cannot be accessed through super");
                }
                self.walk_expr(*object);
                if self.is_private_name_defined(*property) {
                    self.record_private_use(expr_id, *property);
                } else {
                    self.diagnostics
                        .error(*span, "private name used outside of class body");
                }
            }

            Expr::PrivateInExpression {
                object,
                property,
                span,
                ..
            } => {
                self.walk_expr(*object);
                if self.is_private_name_defined(*property) {
                    self.record_private_use(expr_id, *property);
                } else {
                    self.diagnostics
                        .error(*span, "private name used outside of class body");
                }
            }

            Expr::OptionalChainExpression { base, .. } => {
                self.walk_expr(*base);
            }

            Expr::SequenceExpression { expressions, .. } => {
                let exprs = self.ast.get_expr_list(*expressions);
                for &e in exprs {
                    self.walk_expr(e);
                }
            }

            Expr::TemplateLiteral { template, .. } => {
                let exprs = self.ast.templates().get_expressions(*template);
                for &e in exprs {
                    self.walk_expr(e);
                }
            }

            Expr::TaggedTemplateExpression { tag, template, .. } => {
                self.walk_expr(*tag);
                let exprs = self.ast.templates().get_expressions(*template);
                for &e in exprs {
                    self.walk_expr(e);
                }
            }

            Expr::MetaProperty {
                meta,
                property,
                span,
                ..
            } => {
                if *meta == WellKnownAtom::new.id() && *property == WellKnownAtom::target.id() {
                    if let Some(func_id) = self.ctx.current_function {
                        self.functions.get_mut(func_id).references_new_target = true;
                    } else if self.ctx.in_static_block {
                    } else {
                        self.diagnostics
                            .error(*span, "'new.target' outside of a function");
                    }
                }
            }

            Expr::ImportExpression {
                source, options, ..
            } => {
                self.walk_expr(*source);
                if let Some(options) = options {
                    self.walk_expr(*options);
                }
            }

            Expr::ParenthesizedExpression { expression, .. } => {
                self.walk_expr(*expression);
            }

            Expr::NumericLiteral { span, syntax, .. } => {
                if self.ctx.strict && *syntax == NumericLiteralSyntax::LegacyOctalLikeDecimal {
                    self.diagnostics.error(
                        *span,
                        "legacy octal-like decimal literal not allowed in strict mode",
                    );
                }
            }

            Expr::StringLiteral { span, syntax, .. } => {
                if self.ctx.strict && syntax.has_strict_mode_escape() {
                    self.diagnostics
                        .error(*span, "legacy string escape not allowed in strict mode");
                }
            }

            Expr::NullLiteral { .. }
            | Expr::BooleanLiteral { .. }
            | Expr::BigIntLiteral { .. }
            | Expr::RegExpLiteral { .. }
            | Expr::InvalidExpression { .. } => {}
        }
    }

    pub(super) fn walk_assignment_target_expr(&mut self, expr_id: lyng_js_ast::ExprId) {
        match self.ast.get_expr(expr_id).clone() {
            Expr::ParenthesizedExpression { span, expression } => {
                if matches!(
                    self.ast.get_expr(expression),
                    Expr::ObjectExpression { .. } | Expr::ArrayExpression { .. }
                ) {
                    self.diagnostics.error(span, "invalid assignment target");
                }
                self.walk_assignment_target_expr(expression);
            }
            Expr::ArrayExpression { elements, .. } => {
                let elems = self.ast.get_opt_expr_list(elements).to_vec();
                for elem in elems.into_iter().flatten() {
                    if let Expr::SpreadElement { argument, .. } = self.ast.get_expr(elem).clone() {
                        self.walk_assignment_target_expr(argument);
                    } else {
                        self.walk_assignment_target_expr(elem);
                    }
                }
            }
            Expr::ObjectExpression { properties, .. } => {
                let props = self.ast.get_property_list(properties).to_vec();
                for prop in props {
                    if prop.computed {
                        self.walk_expr(prop.key);
                    }

                    if prop.method || prop.kind != PropertyKind::Init {
                        self.walk_expr(prop.value);
                        continue;
                    }

                    if let Expr::SpreadElement { argument, .. } =
                        self.ast.get_expr(prop.value).clone()
                    {
                        if prop.key == prop.value {
                            self.walk_assignment_target_expr(argument);
                            continue;
                        }
                    }

                    if prop.shorthand {
                        self.walk_assignment_target_expr(prop.key);
                        if prop.value != prop.key {
                            self.walk_expr(prop.value);
                        }
                    } else {
                        self.walk_assignment_target_expr(prop.value);
                    }
                }
            }
            Expr::AssignmentExpression {
                operator: AssignOp::Assign,
                left,
                right,
                ..
            } => {
                self.walk_assignment_target_expr(left);
                self.walk_expr(right);
            }
            _ => self.walk_expr(expr_id),
        }
    }

    pub(super) fn expr_is_destructuring_pattern(&self, expr_id: lyng_js_ast::ExprId) -> bool {
        match self.ast.get_expr(expr_id) {
            Expr::ArrayExpression { .. } | Expr::ObjectExpression { .. } => true,
            Expr::ParenthesizedExpression { expression, .. } => {
                self.expr_is_destructuring_pattern(*expression)
            }
            _ => false,
        }
    }

    fn validate_object_literal_properties(&mut self, properties: &[lyng_js_ast::Property]) {
        let mut proto_seen = false;

        for prop in properties {
            if prop.shorthand && prop.value != prop.key {
                self.diagnostics.error(
                    prop.span,
                    "cover initialized names are only allowed in destructuring patterns",
                );
            }

            if self.is_proto_data_property(prop) {
                if proto_seen {
                    self.diagnostics
                        .error(prop.span, "duplicate __proto__ property in object literal");
                } else {
                    proto_seen = true;
                }
            }
        }
    }

    fn is_proto_data_property(&self, property: &lyng_js_ast::Property) -> bool {
        if property.kind != PropertyKind::Init
            || property.computed
            || property.method
            || property.shorthand
        {
            return false;
        }

        match self.ast.get_expr(property.key) {
            Expr::Identifier { name, .. } => *name == WellKnownAtom::__proto__.id(),
            Expr::StringLiteral { value, .. } => {
                self.ast.literals().get_string(*value) == "__proto__"
            }
            _ => false,
        }
    }
}
