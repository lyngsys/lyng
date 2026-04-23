use lyng_js_ast::{Decl, Pattern};
use lyng_js_common::{AtomId, Span, WellKnownAtom};

use super::Analyzer;

impl<'a> Analyzer<'a> {
    fn record_export_name(&mut self, name: AtomId, span: Span) {
        if !self.ctx.exported_names.insert(name) {
            let name_str = self.atoms.resolve(name);
            self.diagnostics
                .error(span, format!("duplicate export name '{name_str}'"));
        }
    }

    pub(super) fn walk_export_kind(&mut self, kind: &lyng_js_ast::ExportKind, span: Span) {
        match kind {
            lyng_js_ast::ExportKind::Named {
                specifiers, source, ..
            } => {
                let specs = self.ast.get_export_spec_list(*specifiers);
                for spec in specs {
                    self.record_export_name(spec.exported, spec.span);
                    if source.is_none() {
                        let (binding, _) = self.resolve_name(spec.local);
                        if binding.is_none() {
                            let name = self.atoms.resolve(spec.local);
                            self.diagnostics
                                .error(spec.span, format!("export '{name}' is not defined"));
                        }
                    }
                }
            }
            lyng_js_ast::ExportKind::Default { declaration } => {
                self.record_export_name(WellKnownAtom::default.id(), span);
                match declaration {
                    lyng_js_ast::ExportDefaultDecl::Function(func_id) => {
                        self.walk_function(*func_id);
                    }
                    lyng_js_ast::ExportDefaultDecl::Class(decl_id) => self.walk_decl(*decl_id),
                    lyng_js_ast::ExportDefaultDecl::Expression(expr_id) => self.walk_expr(*expr_id),
                }
            }
            lyng_js_ast::ExportKind::All { exported, .. } => {
                if let Some(name) = exported {
                    self.record_export_name(*name, span);
                }
            }
            lyng_js_ast::ExportKind::Declaration { decl } => {
                let d = self.ast.get_decl(*decl);
                match d {
                    Decl::Variable { declarators, .. } => {
                        let decls = self.ast.get_var_declarator_list(*declarators);
                        for vd in decls {
                            self.collect_export_names_from_pattern(vd.id);
                        }
                    }
                    Decl::Function { function, span, .. } => {
                        let func = self.ast.get_function(*function);
                        if let Some(name) = func.name {
                            self.record_export_name(name, *span);
                        }
                    }
                    Decl::Class { name, span, .. } => {
                        if let Some(n) = name {
                            self.record_export_name(*n, *span);
                        }
                    }
                    _ => {}
                }
                self.walk_decl(*decl);
            }
        }
    }

    fn collect_export_names_from_pattern(&mut self, pat_id: lyng_js_ast::PatternId) {
        let pat = self.ast.get_pattern(pat_id);
        if let Pattern::Identifier { name, span, .. } = pat {
            self.record_export_name(*name, *span);
        }
    }
}
