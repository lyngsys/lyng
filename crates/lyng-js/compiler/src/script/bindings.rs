use super::*;

impl<'a, 'b> FunctionCompiler<'a, 'b> {
    fn declaration_kind_matches(
        actual_kind: DeclarationKind,
        expected_kind: DeclarationKind,
    ) -> bool {
        actual_kind == expected_kind
            || (expected_kind == DeclarationKind::Var && actual_kind == DeclarationKind::Function)
    }

    pub(super) fn use_site(&self, expr: ExprId) -> LoweringResult<&UseSiteRecord> {
        self.state
            .sema
            .use_sites
            .for_expr(expr)
            .ok_or(LoweringError::MissingUseSite { expr })
    }

    pub(super) fn private_use(
        &self,
        expr: ExprId,
    ) -> LoweringResult<&lyng_js_sema::PrivateUseRecord> {
        self.state
            .sema
            .private_uses
            .for_expr(expr)
            .ok_or(LoweringError::MissingPrivateUse { expr })
    }

    pub(super) fn binding(
        &self,
        binding: SemanticBindingId,
    ) -> LoweringResult<&lyng_js_sema::BindingRecord> {
        self.state
            .sema
            .binding_table
            .as_slice()
            .get(binding.raw() as usize)
            .ok_or(LoweringError::MissingBinding { binding })
    }

    pub(super) fn find_named_binding(
        &self,
        name: AtomId,
        expected_kind: DeclarationKind,
    ) -> LoweringResult<SemanticBindingId> {
        let mut matches = self
            .state
            .sema
            .binding_table
            .as_slice()
            .iter()
            .enumerate()
            .filter_map(|(index, binding)| {
                (binding.name == name
                    && binding.kind == expected_kind
                    && self.binding_belongs_to_current_function(binding.scope))
                .then_some(SemanticBindingId::new(index as u32))
            });

        let Some(first) = matches.next() else {
            return Err(LoweringError::MissingDeclarationBinding { name });
        };
        if matches.next().is_some() {
            return Err(LoweringError::AmbiguousDeclarationBinding { name });
        }
        Ok(first)
    }

    pub(super) fn declared_binding_for_pattern(
        &self,
        pattern: lyng_js_ast::PatternId,
        expected_kind: DeclarationKind,
    ) -> LoweringResult<SemanticBindingId> {
        let Pattern::Identifier { name, .. } = self.ast().get_pattern(pattern).clone() else {
            return Err(LoweringError::UnsupportedPattern { pattern });
        };
        if let Some(binding) = self.state.sema.pattern_binding(pattern) {
            if Self::declaration_kind_matches(self.binding(binding)?.kind, expected_kind) {
                return Ok(binding);
            }
        }
        self.find_innermost_named_binding(name, expected_kind)
            .ok_or(LoweringError::MissingDeclarationBinding { name })
    }

    fn find_innermost_named_binding(
        &self,
        name: AtomId,
        expected_kind: DeclarationKind,
    ) -> Option<SemanticBindingId> {
        self.state
            .sema
            .binding_table
            .as_slice()
            .iter()
            .enumerate()
            .filter_map(|(index, binding)| {
                (Self::declaration_kind_matches(binding.kind, expected_kind)
                    && binding.name == name
                    && self.binding_belongs_to_owner(binding.scope, self.current_function))
                .then_some((SemanticBindingId::new(index as u32), binding.scope.raw()))
            })
            .max_by_key(|(_, scope)| *scope)
            .map(|(binding, _)| binding)
    }

    pub(super) fn binding_belongs_to_current_function(&self, scope: ScopeId) -> bool {
        self.binding_belongs_to_owner(scope, self.current_function)
    }

    pub(super) fn binding_belongs_to_owner(
        &self,
        scope: ScopeId,
        owner: Option<FunctionSemaId>,
    ) -> bool {
        self.scope_owner(scope) == owner
    }

    pub(super) fn scope_owner(&self, scope: ScopeId) -> Option<FunctionSemaId> {
        self.state.sema.scope_table.get(scope).owning_function
    }

    pub(super) fn current_activation(&self) -> LoweringResult<&FunctionActivationPlan> {
        self.current_function
            .map(|function| self.state.activation(function))
            .ok_or(LoweringError::UnsupportedFunction {
                function: self
                    .current_function_ast
                    .unwrap_or_else(|| FunctionId::new(0)),
            })
    }

    pub(super) fn binding_env_access(
        &self,
        binding: SemanticBindingId,
    ) -> LoweringResult<Option<(u8, u32)>> {
        let binding_record = self.binding(binding)?;
        let owner = self.scope_owner(binding_record.scope);
        let (synthetic_parameter, synthetic_rest) = owner
            .map(|owner| {
                let activation = self.state.activation(owner);
                (
                    activation.arguments_mode == ArgumentsMode::Mapped
                        && binding_record.kind == DeclarationKind::Parameter,
                    activation.rest_binding == Some(binding),
                )
            })
            .unwrap_or((false, false));
        if !matches!(
            binding_record.storage_class,
            StorageClass::EnvironmentSlot | StorageClass::DynamicLookup
        ) && !synthetic_parameter
            && !synthetic_rest
        {
            return Ok(None);
        }
        if binding_record.storage_class == StorageClass::DynamicLookup
            && binding_record.slot_index.is_none()
            && !synthetic_parameter
            && !synthetic_rest
        {
            return Ok(None);
        }
        let depth = self.binding_environment_depth(binding)?;
        let slot = self.state.runtime_slot_for_binding(binding)?;
        Ok(Some((depth, slot)))
    }

    pub(super) fn arguments_access(&self, name: AtomId) -> LoweringResult<Option<(u8, u32)>> {
        if name != WellKnownAtom::arguments.id() {
            return Ok(None);
        }
        let Some(current) = self.current_function else {
            return Ok(None);
        };
        let Some(owner) = self.state.arguments_owner_for_current(Some(current)) else {
            return Ok(None);
        };
        let activation = self.state.activation(owner);
        let Some(slot) = activation.arguments_slot() else {
            return Ok(None);
        };
        let depth = self.state.environment_depth_to_function(current, owner)?;
        Ok(Some((depth, u32::from(slot))))
    }

    pub(super) fn ensure_local_register(
        &mut self,
        binding: SemanticBindingId,
    ) -> LoweringResult<u16> {
        if let Some(register) = self.local_registers[binding.raw() as usize] {
            return Ok(register);
        }

        let register = self.alloc_temp()?;
        self.local_registers[binding.raw() as usize] = Some(register);
        Ok(register)
    }

    pub(super) fn ensure_child_index(&mut self, function: FunctionId) -> LoweringResult<u16> {
        if let Some(index) = self.child_indices.get(&function) {
            return Ok(*index);
        }
        let child_id = self.state.ensure_child_compiled(function)?;
        let child_index = self.builder.add_child_function(child_id)?;
        self.child_indices.insert(function, child_index);
        Ok(child_index)
    }

    pub(super) fn binding_environment_depth(
        &self,
        binding: SemanticBindingId,
    ) -> LoweringResult<u8> {
        let binding_scope = self.binding(binding)?.scope;
        let binding_owner = self.scope_owner(binding_scope);
        match (self.current_function, binding_owner) {
            (None, None) => Ok(0),
            (Some(current), Some(owner)) if current == owner => Ok(0),
            (Some(current), Some(owner)) => self
                .state
                .environment_depth_to_function(current, owner)
                .map_err(|_| LoweringError::InvalidCapturedBindingDepth {
                    binding,
                    function: self.current_function,
                }),
            (Some(current), None) => self.state.environment_depth_to_root(current).map_err(|_| {
                LoweringError::InvalidCapturedBindingDepth {
                    binding,
                    function: self.current_function,
                }
            }),
            _ => Err(LoweringError::InvalidCapturedBindingDepth {
                binding,
                function: self.current_function,
            }),
        }
    }

    pub(super) fn capture_source_depth(&self, binding_scope: ScopeId) -> LoweringResult<u8> {
        let Some(current) = self.current_function else {
            return Ok(0);
        };
        let Some(owner) = self.scope_owner(binding_scope) else {
            return self.state.environment_depth_to_root(current).map_err(|_| {
                LoweringError::UnsupportedFunction {
                    function: self
                        .current_function_ast
                        .expect("capture depth only queried for functions"),
                }
            });
        };
        self.state
            .environment_depth_to_function(current, owner)
            .map_err(|_| LoweringError::UnsupportedFunction {
                function: self
                    .current_function_ast
                    .expect("capture depth only queried for functions"),
            })
    }
}
