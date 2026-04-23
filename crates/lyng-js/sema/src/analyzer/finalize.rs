use super::Analyzer;
use crate::binding::StorageClass;
use crate::ids::{FunctionSemaId, ScopeId, SemanticBindingId};
use crate::scope::ScopeKind;
use crate::use_site::ResolutionKind;

impl<'a> Analyzer<'a> {
    pub(super) fn finalize(&mut self) {
        self.propagate_eval_with();

        for record in self.use_sites.as_mut_slice() {
            let scope = self.scopes.get(record.scope);
            if (scope.has_eval || scope.has_with)
                && matches!(
                    record.resolution_kind,
                    ResolutionKind::Captured | ResolutionKind::Global | ResolutionKind::Unresolved
                )
            {
                record.resolution_kind = ResolutionKind::Dynamic;
            }
        }

        self.promote_dynamic_outer_bindings();

        let binding_count = self.bindings.len();
        for i in 0..binding_count {
            let bid = SemanticBindingId::new(i as u32);
            let binding = self.bindings.get(bid);
            let scope = binding.scope;
            let binding_kind = binding.kind;
            let scope_rec = self.scopes.get(scope);

            let parameter_scope = scope_rec.kind == ScopeKind::Parameter;
            let new_storage = if parameter_scope {
                StorageClass::EnvironmentSlot
            } else if scope_rec.has_eval || scope_rec.has_with {
                if scope_rec.kind == ScopeKind::Global && binding_kind.is_hoisted() {
                    StorageClass::GlobalName
                } else {
                    StorageClass::EnvironmentSlot
                }
            } else if matches!(
                binding_kind,
                crate::binding::DeclarationKind::Using
                    | crate::binding::DeclarationKind::AwaitUsing
            ) {
                StorageClass::EnvironmentSlot
            } else if scope_rec.kind == ScopeKind::Module {
                StorageClass::EnvironmentSlot
            } else if scope_rec.kind == ScopeKind::ClassBody
                && binding.kind == crate::binding::DeclarationKind::ClassName
            {
                StorageClass::EnvironmentSlot
            } else if scope_rec.kind == ScopeKind::Global && binding.kind.is_lexical() {
                StorageClass::EnvironmentSlot
            } else if scope_rec.kind == ScopeKind::ForLoop && binding.kind.is_lexical() {
                StorageClass::EnvironmentSlot
            } else if binding.is_captured {
                StorageClass::EnvironmentSlot
            } else if scope_rec.kind == ScopeKind::Global && binding.kind.is_hoisted() {
                StorageClass::GlobalName
            } else {
                StorageClass::FrameLocal
            };

            let needs_env = new_storage == StorageClass::EnvironmentSlot
                || new_storage == StorageClass::DynamicLookup;

            self.bindings.get_mut(bid).storage_class = new_storage;
            self.bindings.get_mut(bid).needs_environment = needs_env;
            if parameter_scope && binding_kind == crate::binding::DeclarationKind::Parameter {
                self.bindings.get_mut(bid).has_tdz = true;
            }
        }

        let scope_count = self.scopes.len();
        for i in 0..scope_count {
            let sid = ScopeId::new(i as u32);
            let scope = self.scopes.get(sid);
            let needs_env = scope.bindings.iter().any(|&bid| {
                let b = self.bindings.get(bid);
                b.storage_class == StorageClass::EnvironmentSlot
                    || b.storage_class == StorageClass::DynamicLookup
            });
            self.scopes.get_mut(sid).needs_environment = needs_env;
        }

        for i in 0..scope_count {
            let sid = ScopeId::new(i as u32);
            let scope = self.scopes.get(sid);
            if !scope.needs_environment {
                continue;
            }
            let bindings = scope.bindings.clone();
            let mut slot = 0u32;
            for &bid in &bindings {
                let b = self.bindings.get(bid);
                if b.storage_class == StorageClass::EnvironmentSlot
                    || b.storage_class == StorageClass::DynamicLookup
                {
                    self.bindings.get_mut(bid).slot_index = Some(slot);
                    slot += 1;
                }
            }
        }

        let mut scope_to_func: Vec<Option<FunctionSemaId>> = vec![None; scope_count];
        let func_count = self.functions.len();
        for i in 0..func_count {
            let fid = FunctionSemaId::new(i as u32);
            let func = self.functions.get(fid);
            self.mark_scope_owner(func.scope_root, fid, &mut scope_to_func);
            if let Some(param_scope) = func.param_scope {
                self.mark_scope_owner(param_scope, fid, &mut scope_to_func);
            }
        }

        let mut func_captures: Vec<Vec<SemanticBindingId>> = vec![Vec::new(); func_count];
        for record in self.use_sites.as_slice() {
            if record.resolution_kind == ResolutionKind::Captured {
                if let Some(bid) = record.resolved_binding {
                    if let Some(owner) = scope_to_func[record.scope.raw() as usize] {
                        func_captures[owner.raw() as usize].push(bid);
                    }
                }
            }
        }

        for i in 0..func_count {
            let fid = FunctionSemaId::new(i as u32);
            let mut captures = std::mem::take(&mut func_captures[i]);
            captures.sort_unstable_by_key(|b| b.raw());
            captures.dedup();

            let func_scope = self.functions.get(fid).scope_root;
            let needs_env = !captures.is_empty() || self.scope_tree_needs_env(func_scope);

            self.functions.get_mut(fid).captures = captures;
            self.functions.get_mut(fid).needs_environment = needs_env;
        }
    }

    fn propagate_eval_with(&mut self) {
        let scope_count = self.scopes.len();
        for i in (0..scope_count).rev() {
            let sid = ScopeId::new(i as u32);
            let scope = self.scopes.get(sid);
            if scope.has_eval || scope.has_with {
                if let Some(parent) = scope.parent {
                    if self.scopes.get(parent).owning_function != scope.owning_function {
                        continue;
                    }
                    let has_eval = scope.has_eval;
                    let has_with = scope.has_with;
                    if has_eval {
                        self.scopes.get_mut(parent).has_eval = true;
                    }
                    if has_with {
                        self.scopes.get_mut(parent).has_with = true;
                    }
                }
            }
        }
    }

    fn promote_dynamic_outer_bindings(&mut self) {
        let dynamic_uses = self
            .use_sites
            .as_slice()
            .iter()
            .filter_map(|record| {
                (record.resolution_kind == ResolutionKind::Dynamic)
                    .then_some((record.scope, record.name))
            })
            .collect::<Vec<_>>();

        for (scope, name) in dynamic_uses {
            self.mark_dynamic_outer_binding_captured(scope, name);
        }
    }

    fn mark_dynamic_outer_binding_captured(
        &mut self,
        scope_id: ScopeId,
        name: lyng_js_common::AtomId,
    ) {
        let mut current = Some(scope_id);
        let mut crossed_function = false;

        while let Some(scope_id) = current {
            let scope = self.scopes.get(scope_id);
            for &bid in &scope.bindings {
                if self.bindings.get(bid).name != name {
                    continue;
                }
                if crossed_function {
                    self.bindings.get_mut(bid).is_captured = true;
                }
                return;
            }

            let parent = scope.parent;
            if matches!(scope.kind, ScopeKind::Function | ScopeKind::Parameter) {
                if let Some(parent) = parent {
                    if self.scopes.get(parent).kind != ScopeKind::Parameter {
                        crossed_function = true;
                    }
                }
            }

            current = parent;
        }
    }

    fn mark_scope_owner(
        &self,
        scope_id: ScopeId,
        func_id: FunctionSemaId,
        map: &mut [Option<FunctionSemaId>],
    ) {
        map[scope_id.raw() as usize] = Some(func_id);
        let child_count = self.scopes.get(scope_id).children.len();
        for i in 0..child_count {
            let child = self.scopes.get(scope_id).children[i];
            if self.scopes.get(child).owning_function == Some(func_id)
                || self.scopes.get(child).owning_function.is_none()
            {
                self.mark_scope_owner(child, func_id, map);
            }
        }
    }

    fn scope_tree_needs_env(&self, scope_id: ScopeId) -> bool {
        let scope = self.scopes.get(scope_id);
        if scope.needs_environment {
            return true;
        }
        for &child in &scope.children {
            if self.scope_tree_needs_env(child) {
                return true;
            }
        }
        false
    }
}
