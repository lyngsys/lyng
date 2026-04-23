use lyng_js_ast::{Decl, Stmt};
use lyng_js_common::{AtomId, Span, WellKnownAtom};

use super::Analyzer;
use crate::binding::{BindingRecord, DeclarationKind, StorageClass};
use crate::ids::{ScopeId, SemanticBindingId};
use crate::scope::ScopeKind;
use crate::use_site::{ResolutionKind, UseSiteRecord};

impl<'a> Analyzer<'a> {
    pub(super) fn declare_binding(
        &mut self,
        name: AtomId,
        kind: DeclarationKind,
        scope: ScopeId,
        span: Span,
    ) -> SemanticBindingId {
        let key = (scope, name);
        if let Some(&existing_bid) = self.scope_binding_names.get(&key) {
            let existing = self.bindings.get(existing_bid);
            let new_is_lexical = self.binding_is_lexical_in_scope(kind, scope);
            let existing_is_lexical = self.binding_is_lexical_in_scope(existing.kind, scope);
            if new_is_lexical
                && self.annex_b_allows_duplicate_block_function(scope, kind, existing.kind)
            {
            } else if new_is_lexical {
                self.diagnostics
                    .error(span, "duplicate lexical declaration");
            } else if kind == DeclarationKind::Var && existing_is_lexical {
                self.diagnostics
                    .error(span, "var declaration conflicts with lexical declaration");
            }
        }

        if self.ctx.strict {
            self.check_strict_binding_name(name, span);
        }

        if self.ctx.in_static_block
            && self.ctx.current_function.is_none()
            && name == WellKnownAtom::r#await.id()
        {
            self.diagnostics.error(
                span,
                "'await' cannot be used as a binding name inside a class static block",
            );
        }

        let id = self.bindings.alloc(BindingRecord {
            name,
            kind,
            scope,
            is_captured: false,
            needs_environment: false,
            storage_class: StorageClass::FrameLocal,
            has_tdz: kind.has_tdz(),
            slot_index: None,
        });

        self.scopes.get_mut(scope).bindings.push(id);
        self.scope_binding_names.insert(key, id);
        id
    }

    pub(super) fn declare_var_binding(&mut self, name: AtomId, span: Span) -> SemanticBindingId {
        let var_scope = self.find_var_scope();
        if let Some(&existing_bid) = self.scope_binding_names.get(&(var_scope, name)) {
            let existing = self.bindings.get(existing_bid);
            if matches!(
                existing.kind,
                DeclarationKind::Var | DeclarationKind::Function
            ) {
                return existing_bid;
            }
            if self.binding_is_lexical_in_scope(existing.kind, var_scope) {
                self.diagnostics
                    .error(span, "var declaration conflicts with lexical declaration");
                return existing_bid;
            }
        }

        self.check_var_conflicts(name, span, var_scope);
        self.declare_binding(name, DeclarationKind::Var, var_scope, span)
    }

    fn check_var_conflicts(&mut self, name: AtomId, span: Span, var_scope: ScopeId) {
        let mut scope_id = self.ctx.current_scope;
        loop {
            if let Some(&bid) = self.scope_binding_names.get(&(scope_id, name)) {
                let existing = self.bindings.get(bid);
                if self.binding_is_lexical_in_scope(existing.kind, scope_id) {
                    self.diagnostics
                        .error(span, "var declaration conflicts with lexical declaration");
                    return;
                }
            }
            if scope_id == var_scope {
                break;
            }
            match self.scopes.get(scope_id).parent {
                Some(parent) => scope_id = parent,
                None => break,
            }
        }
    }

    pub(super) fn binding_is_lexical_in_scope(
        &self,
        kind: DeclarationKind,
        scope: ScopeId,
    ) -> bool {
        kind.is_lexical()
            || (kind == DeclarationKind::Function
                && self.function_binding_is_lexical_in_scope(scope))
    }

    pub(super) fn function_binding_is_lexical_in_scope(&self, scope: ScopeId) -> bool {
        self.function_binding_is_lexical_in_scope_kind(self.scopes.get(scope).kind)
    }

    pub(super) fn function_binding_is_lexical_in_scope_kind(&self, kind: ScopeKind) -> bool {
        !matches!(kind, ScopeKind::Global | ScopeKind::Function)
    }

    fn annex_b_allows_duplicate_block_function(
        &self,
        scope: ScopeId,
        new_kind: DeclarationKind,
        existing_kind: DeclarationKind,
    ) -> bool {
        !self.ctx.strict
            && new_kind == DeclarationKind::Function
            && existing_kind == DeclarationKind::Function
            && matches!(
                self.scopes.get(scope).kind,
                ScopeKind::Block | ScopeKind::Switch
            )
    }

    pub(super) fn annex_b_if_clause_function_skips_var_name(
        &self,
        stmt_id: lyng_js_ast::StmtId,
        scope_kind: ScopeKind,
    ) -> bool {
        if self.ctx.strict || !matches!(scope_kind, ScopeKind::Global | ScopeKind::Function) {
            return false;
        }

        let Stmt::Declaration { decl, .. } = self.ast.get_stmt(stmt_id) else {
            return false;
        };

        matches!(self.ast.get_decl(*decl), Decl::Function { .. })
    }

    pub(super) fn find_var_scope(&self) -> ScopeId {
        let mut scope_id = self.ctx.current_scope;
        loop {
            let kind = self.scopes.get(scope_id).kind;
            match kind {
                ScopeKind::Global | ScopeKind::Module | ScopeKind::Function => return scope_id,
                _ => match self.scopes.get(scope_id).parent {
                    Some(parent) => scope_id = parent,
                    None => return scope_id,
                },
            }
        }
    }

    pub(super) fn resolve_name(&self, name: AtomId) -> (Option<SemanticBindingId>, ResolutionKind) {
        let mut scope_id = self.ctx.current_scope;
        let mut crossed_function = false;

        loop {
            let scope = self.scopes.get(scope_id);

            if scope.has_eval || scope.has_with {
                return (None, ResolutionKind::Dynamic);
            }

            for &bid in &scope.bindings {
                if self.bindings.get(bid).name == name {
                    let kind = if crossed_function {
                        ResolutionKind::Captured
                    } else {
                        ResolutionKind::Local
                    };
                    return (Some(bid), kind);
                }
            }

            if scope.kind == ScopeKind::Function || scope.kind == ScopeKind::Parameter {
                if let Some(parent) = scope.parent {
                    let parent_kind = self.scopes.get(parent).kind;
                    if parent_kind != ScopeKind::Parameter {
                        crossed_function = true;
                    }
                }
            }

            match scope.parent {
                Some(parent) => scope_id = parent,
                None => break,
            }
        }

        (None, ResolutionKind::Global)
    }

    pub(super) fn check_strict_binding_name(&mut self, name: AtomId, span: Span) {
        if name == WellKnownAtom::eval.id() {
            self.diagnostics.error(
                span,
                "'eval' cannot be used as a binding name in strict mode",
            );
        } else if name == WellKnownAtom::arguments.id() {
            self.diagnostics.error(
                span,
                "'arguments' cannot be used as a binding name in strict mode",
            );
        } else if self.is_strict_reserved_word(name) {
            let name_str = self.atoms.resolve(name);
            self.diagnostics.error(
                span,
                format!("'{name_str}' is a reserved word in strict mode"),
            );
        }
    }

    fn is_strict_reserved_word(&self, name: AtomId) -> bool {
        name == WellKnownAtom::implements.id()
            || name == WellKnownAtom::interface.id()
            || name == WellKnownAtom::let_.id()
            || name == WellKnownAtom::package.id()
            || name == WellKnownAtom::private.id()
            || name == WellKnownAtom::protected.id()
            || name == WellKnownAtom::public.id()
            || name == WellKnownAtom::r#static.id()
            || name == WellKnownAtom::yield_.id()
    }

    pub(super) fn record_use(
        &mut self,
        expr: Option<lyng_js_ast::ExprId>,
        name: AtomId,
        _span: Span,
    ) {
        let (binding, kind) = self.resolve_name(name);
        let scope = self.ctx.current_scope;
        self.use_sites.alloc(UseSiteRecord {
            expr,
            name,
            scope,
            resolved_binding: binding,
            resolution_kind: kind,
        });

        if kind == ResolutionKind::Captured {
            if let Some(bid) = binding {
                self.bindings.get_mut(bid).is_captured = true;
            }
        }
    }
}
