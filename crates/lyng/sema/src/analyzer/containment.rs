use lyng_js_ast::{Decl, Expr, ForInOfLeft, ForInit, Pattern, Stmt};
use lyng_js_common::WellKnownAtom;

use super::{Analyzer, ContainmentQuery};

impl Analyzer<'_> {
    pub(super) fn pattern_contains_query(
        &self,
        pat_id: lyng_js_ast::PatternId,
        query: ContainmentQuery,
    ) -> bool {
        match self.ast.get_pattern(pat_id) {
            Pattern::Identifier { .. } | Pattern::InvalidPattern { .. } => false,
            Pattern::Object {
                properties, rest, ..
            } => {
                self.ast
                    .get_obj_pattern_prop_list(*properties)
                    .iter()
                    .any(|prop| self.pattern_contains_query(prop.value, query))
                    || rest.is_some_and(|rest| self.pattern_contains_query(rest, query))
            }
            Pattern::Array { elements, rest, .. } => {
                self.ast
                    .get_opt_pattern_elem_list(*elements)
                    .iter()
                    .flatten()
                    .any(|elem| self.pattern_contains_query(elem.pattern, query))
                    || rest.is_some_and(|rest| self.pattern_contains_query(rest, query))
            }
            Pattern::Assignment { left, right, .. } => {
                self.pattern_contains_query(*left, query) || self.expr_contains_query(*right, query)
            }
        }
    }

    pub(super) fn function_body_contains_query(
        &self,
        func_id: lyng_js_ast::FunctionId,
        query: ContainmentQuery,
    ) -> bool {
        let func = self.ast.get_function(func_id);
        self.ast
            .get_pattern_list(func.params.params)
            .iter()
            .any(|&param| self.pattern_contains_query(param, query))
            || func
                .params
                .rest
                .is_some_and(|rest| self.pattern_contains_query(rest, query))
            || func
                .expression_body
                .is_some_and(|expr| self.expr_contains_query(expr, query))
            || self.stmt_list_contains_query(func.body, query)
    }

    pub(super) fn stmt_list_contains_query(
        &self,
        body: lyng_js_ast::NodeList<lyng_js_ast::StmtId>,
        query: ContainmentQuery,
    ) -> bool {
        self.ast
            .get_stmt_list(body)
            .iter()
            .any(|&stmt| self.stmt_contains_query(stmt, query))
    }

    pub(super) fn stmt_contains_query(
        &self,
        stmt_id: lyng_js_ast::StmtId,
        query: ContainmentQuery,
    ) -> bool {
        match self.ast.get_stmt(stmt_id) {
            Stmt::Block { body, .. } => self.stmt_list_contains_query(*body, query),
            Stmt::Expression { expression, .. } => self.expr_contains_query(*expression, query),
            Stmt::If {
                test,
                consequent,
                alternate,
                ..
            } => {
                self.expr_contains_query(*test, query)
                    || self.stmt_contains_query(*consequent, query)
                    || alternate.is_some_and(|alt| self.stmt_contains_query(alt, query))
            }
            Stmt::DoWhile { body, test, .. } => {
                self.stmt_contains_query(*body, query) || self.expr_contains_query(*test, query)
            }
            Stmt::While { test, body, .. } => {
                self.expr_contains_query(*test, query) || self.stmt_contains_query(*body, query)
            }
            Stmt::For {
                init,
                test,
                update,
                body,
                ..
            } => {
                init.is_some_and(|init| match init {
                    ForInit::Declaration(decl) => self.decl_contains_query(decl, query),
                    ForInit::Expression(expr) => self.expr_contains_query(expr, query),
                }) || test.is_some_and(|expr| self.expr_contains_query(expr, query))
                    || update.is_some_and(|expr| self.expr_contains_query(expr, query))
                    || self.stmt_contains_query(*body, query)
            }
            Stmt::ForIn {
                left, right, body, ..
            }
            | Stmt::ForOf {
                left, right, body, ..
            } => {
                (match left {
                    ForInOfLeft::Declaration(decl) => self.decl_contains_query(*decl, query),
                    ForInOfLeft::Pattern(pat) => self.pattern_contains_query(*pat, query),
                    ForInOfLeft::Expression(expr) => self.expr_contains_query(*expr, query),
                }) || self.expr_contains_query(*right, query)
                    || self.stmt_contains_query(*body, query)
            }
            Stmt::Return { argument, .. } => {
                argument.is_some_and(|expr| self.expr_contains_query(expr, query))
            }
            Stmt::With { object, body, .. } => {
                self.expr_contains_query(*object, query) || self.stmt_contains_query(*body, query)
            }
            Stmt::Switch {
                discriminant,
                cases,
                ..
            } => {
                self.expr_contains_query(*discriminant, query)
                    || self.ast.get_switch_case_list(*cases).iter().any(|case| {
                        case.test
                            .is_some_and(|test| self.expr_contains_query(test, query))
                            || self.stmt_list_contains_query(case.consequent, query)
                    })
            }
            Stmt::Labeled { body, .. } => self.stmt_contains_query(*body, query),
            Stmt::Throw { argument, .. } => self.expr_contains_query(*argument, query),
            Stmt::Try {
                block,
                handler,
                finalizer,
                ..
            } => {
                self.stmt_contains_query(*block, query)
                    || handler.is_some_and(|catch| {
                        catch
                            .param
                            .is_some_and(|param| self.pattern_contains_query(param, query))
                            || self.stmt_contains_query(catch.body, query)
                    })
                    || finalizer.is_some_and(|finalizer| self.stmt_contains_query(finalizer, query))
            }
            Stmt::Declaration { decl, .. } => self.decl_contains_query(*decl, query),
            Stmt::Empty { .. }
            | Stmt::Continue { .. }
            | Stmt::Break { .. }
            | Stmt::Debugger { .. }
            | Stmt::InvalidStatement { .. } => false,
        }
    }

    pub(super) fn decl_contains_query(
        &self,
        decl_id: lyng_js_ast::DeclId,
        query: ContainmentQuery,
    ) -> bool {
        match self.ast.get_decl(decl_id) {
            Decl::Variable { declarators, .. } => self
                .ast
                .get_var_declarator_list(*declarators)
                .iter()
                .any(|decl| {
                    self.pattern_contains_query(decl.id, query)
                        || decl
                            .init
                            .is_some_and(|init| self.expr_contains_query(init, query))
                }),
            Decl::Function { .. } | Decl::Class { .. } => false,
            Decl::Export { kind, .. } => match kind {
                lyng_js_ast::ExportKind::Default { declaration } => match declaration {
                    lyng_js_ast::ExportDefaultDecl::Expression(expr) => {
                        self.expr_contains_query(*expr, query)
                    }
                    _ => false,
                },
                lyng_js_ast::ExportKind::Declaration { decl } => {
                    self.decl_contains_query(*decl, query)
                }
                _ => false,
            },
            Decl::Import { .. } | Decl::InvalidDeclaration { .. } => false,
        }
    }

    pub(super) fn expr_contains_query(
        &self,
        expr_id: lyng_js_ast::ExprId,
        query: ContainmentQuery,
    ) -> bool {
        let expr = self.ast.get_expr(expr_id);
        match query {
            ContainmentQuery::ArgumentsIdentifier => {
                if let Expr::Identifier { name, .. } = expr {
                    if *name == WellKnownAtom::arguments.id() {
                        return true;
                    }
                }
            }
            ContainmentQuery::DirectSuperCall => {
                if let Expr::CallExpression { callee, .. } = expr {
                    if matches!(self.ast.get_expr(*callee), Expr::Super { .. }) {
                        return true;
                    }
                }
            }
            ContainmentQuery::NewTarget => {
                if let Expr::MetaProperty { meta, property, .. } = expr {
                    if *meta == WellKnownAtom::new.id() && *property == WellKnownAtom::target.id() {
                        return true;
                    }
                }
            }
            ContainmentQuery::SuperKeyword => {
                if matches!(expr, Expr::Super { .. }) {
                    return true;
                }
            }
            ContainmentQuery::YieldExpression => {
                if matches!(expr, Expr::YieldExpression { .. }) {
                    return true;
                }
            }
        }

        match expr {
            Expr::ArrayExpression { elements, .. } => self
                .ast
                .get_opt_expr_list(*elements)
                .iter()
                .flatten()
                .any(|&elem| self.expr_contains_query(elem, query)),
            Expr::ObjectExpression { properties, .. } => {
                self.ast.get_property_list(*properties).iter().any(|prop| {
                    self.expr_contains_query(prop.key, query)
                        || self.expr_contains_query(prop.value, query)
                })
            }
            Expr::FunctionExpression { .. } => false,
            Expr::ArrowFunctionExpression { function, .. } => {
                self.function_body_contains_query(*function, query)
            }
            Expr::ClassExpression { .. } => false,
            Expr::TaggedTemplateExpression { tag, template, .. } => {
                self.expr_contains_query(*tag, query)
                    || self
                        .ast
                        .templates()
                        .get_expressions(*template)
                        .iter()
                        .any(|&expr| self.expr_contains_query(expr, query))
            }
            Expr::UnaryExpression { argument, .. }
            | Expr::UpdateExpression { argument, .. }
            | Expr::AwaitExpression { argument, .. }
            | Expr::SpreadElement { argument, .. }
            | Expr::ParenthesizedExpression {
                expression: argument,
                ..
            } => self.expr_contains_query(*argument, query),
            Expr::BinaryExpression { left, right, .. }
            | Expr::LogicalExpression { left, right, .. }
            | Expr::AssignmentExpression { left, right, .. } => {
                self.expr_contains_query(*left, query) || self.expr_contains_query(*right, query)
            }
            Expr::ConditionalExpression {
                test,
                consequent,
                alternate,
                ..
            } => {
                self.expr_contains_query(*test, query)
                    || self.expr_contains_query(*consequent, query)
                    || self.expr_contains_query(*alternate, query)
            }
            Expr::SequenceExpression { expressions, .. } => self
                .ast
                .get_expr_list(*expressions)
                .iter()
                .any(|&expr| self.expr_contains_query(expr, query)),
            Expr::CallExpression {
                callee, arguments, ..
            }
            | Expr::NewExpression {
                callee, arguments, ..
            } => {
                self.expr_contains_query(*callee, query)
                    || self
                        .ast
                        .get_expr_list(*arguments)
                        .iter()
                        .any(|&arg| self.expr_contains_query(arg, query))
            }
            Expr::StaticMemberExpression { object, .. } => self.expr_contains_query(*object, query),
            Expr::ComputedMemberExpression {
                object, property, ..
            } => {
                self.expr_contains_query(*object, query)
                    || self.expr_contains_query(*property, query)
            }
            Expr::PrivateMemberExpression { object, .. } => {
                self.expr_contains_query(*object, query)
            }
            Expr::PrivateInExpression { object, .. } => self.expr_contains_query(*object, query),
            Expr::OptionalChainExpression { base, .. } => self.expr_contains_query(*base, query),
            Expr::YieldExpression { argument, .. } => {
                argument.is_some_and(|expr| self.expr_contains_query(expr, query))
            }
            Expr::TemplateLiteral { template, .. } => self
                .ast
                .templates()
                .get_expressions(*template)
                .iter()
                .any(|&expr| self.expr_contains_query(expr, query)),
            Expr::ImportExpression {
                source, options, ..
            } => {
                self.expr_contains_query(*source, query)
                    || options.is_some_and(|expr| self.expr_contains_query(expr, query))
            }
            Expr::This { .. }
            | Expr::Super { .. }
            | Expr::Identifier { .. }
            | Expr::NullLiteral { .. }
            | Expr::BooleanLiteral { .. }
            | Expr::NumericLiteral { .. }
            | Expr::StringLiteral { .. }
            | Expr::BigIntLiteral { .. }
            | Expr::RegExpLiteral { .. }
            | Expr::MetaProperty { .. }
            | Expr::InvalidExpression { .. } => false,
        }
    }
}
