use super::{
    internal_private_field_get_builtin, internal_private_field_set_builtin, AtomId, Expr, ExprId,
    FunctionCompiler, LoweringError, LoweringResult, ResolutionKind, SemanticBindingId, Span,
    StorageClass,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReferenceUsage {
    WriteOnly,
    ReadWrite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PreparedReferenceTarget {
    Identifier(PreparedIdentifierTarget),
    NamedProperty {
        object: u16,
        property: AtomId,
    },
    KeyedProperty {
        object: u16,
        key: u16,
    },
    SuperProperty {
        receiver: u16,
        key: u16,
        base: u16,
        span: Span,
    },
    Private(PreparedPrivateReferenceTarget),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PreparedIdentifierTarget {
    expr_id: ExprId,
    name: AtomId,
    resolution_kind: ResolutionKind,
    binding: Option<SemanticBindingId>,
    arguments_access: Option<(u8, u32)>,
    reference: Option<u16>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PreparedPrivateReferenceTarget {
    expr_id: ExprId,
    receiver: u16,
    get_descriptor: Option<u16>,
    set_descriptor: u16,
    depth: u16,
    span: Span,
}

impl FunctionCompiler<'_, '_> {
    pub(super) fn prepare_reference_target(
        &mut self,
        expr_id: ExprId,
        usage: ReferenceUsage,
    ) -> LoweringResult<Option<PreparedReferenceTarget>> {
        match self.ast().get_expr(expr_id).clone() {
            Expr::ParenthesizedExpression { expression, .. } => {
                self.prepare_reference_target(expression, usage)
            }
            Expr::Identifier { name, .. } => {
                self.prepare_identifier_reference(expr_id, name).map(Some)
            }
            Expr::StaticMemberExpression {
                object, property, ..
            } => {
                if matches!(self.ast().get_expr(object), Expr::Super { .. }) {
                    self.prepare_super_static_reference(object, property)
                        .map(Some)
                } else {
                    let object = self.lower_expr_to_temp(object)?;
                    Ok(Some(PreparedReferenceTarget::NamedProperty {
                        object,
                        property,
                    }))
                }
            }
            Expr::ComputedMemberExpression {
                object, property, ..
            } => {
                if matches!(self.ast().get_expr(object), Expr::Super { .. }) {
                    self.prepare_super_computed_reference(object, property, usage)
                        .map(Some)
                } else {
                    let object = self.lower_expr_to_temp(object)?;
                    let raw_key = self.lower_expr_to_temp(property)?;
                    let key = if usage == ReferenceUsage::WriteOnly {
                        raw_key
                    } else {
                        self.emit_check_object_coercible(object)?;
                        let key = self.alloc_temp()?;
                        self.emit_to_property_key(key, raw_key)?;
                        key
                    };
                    Ok(Some(PreparedReferenceTarget::KeyedProperty { object, key }))
                }
            }
            Expr::PrivateMemberExpression {
                object, property, ..
            } => self
                .prepare_private_reference(expr_id, object, property, usage)
                .map(Some),
            _ => Ok(None),
        }
    }

    pub(super) const fn reference_target_inferred_name(
        target: PreparedReferenceTarget,
    ) -> Option<AtomId> {
        match target {
            PreparedReferenceTarget::Identifier(target) => Some(target.name),
            PreparedReferenceTarget::NamedProperty { .. }
            | PreparedReferenceTarget::KeyedProperty { .. }
            | PreparedReferenceTarget::SuperProperty { .. }
            | PreparedReferenceTarget::Private(_) => None,
        }
    }

    pub(super) fn load_prepared_reference(
        &mut self,
        target: PreparedReferenceTarget,
        dest: u16,
    ) -> LoweringResult<()> {
        match target {
            PreparedReferenceTarget::Identifier(target) => target.load(self, dest),
            PreparedReferenceTarget::NamedProperty { object, property } => {
                self.emit_get_property_by_atom(dest, object, property)
            }
            PreparedReferenceTarget::KeyedProperty { object, key } => {
                self.emit_get_keyed_property(dest, object, key)
            }
            PreparedReferenceTarget::SuperProperty {
                receiver,
                key,
                base,
                span,
            } => self.emit_super_property_get_from_base(receiver, key, base, span, dest),
            PreparedReferenceTarget::Private(target) => target.load(self, dest),
        }
    }

    pub(super) fn assign_prepared_reference(
        &mut self,
        target: PreparedReferenceTarget,
        value: u16,
    ) -> LoweringResult<()> {
        match target {
            PreparedReferenceTarget::Identifier(target) => target.assign(self, value),
            PreparedReferenceTarget::NamedProperty { object, property } => {
                self.emit_assign_property_by_atom(object, value, property)
            }
            PreparedReferenceTarget::KeyedProperty { object, key } => {
                self.emit_assign_keyed_property(object, value, key)
            }
            PreparedReferenceTarget::SuperProperty {
                receiver,
                key,
                base,
                span,
            } => self.emit_super_property_set_from_base(receiver, key, value, base, span, value),
            PreparedReferenceTarget::Private(target) => target.assign(self, value),
        }
    }

    pub(super) fn store_prepared_reference(
        &mut self,
        target: PreparedReferenceTarget,
        value: u16,
    ) -> LoweringResult<()> {
        match target {
            PreparedReferenceTarget::Identifier(target) => target.store(self, value),
            PreparedReferenceTarget::NamedProperty { .. }
            | PreparedReferenceTarget::KeyedProperty { .. }
            | PreparedReferenceTarget::SuperProperty { .. }
            | PreparedReferenceTarget::Private(_) => self.assign_prepared_reference(target, value),
        }
    }

    fn prepare_identifier_reference(
        &mut self,
        expr_id: ExprId,
        name: AtomId,
    ) -> LoweringResult<PreparedReferenceTarget> {
        let (resolution_kind, binding, arguments_access) = {
            let use_site = self.use_site(expr_id)?;
            (
                use_site.resolution_kind,
                use_site.resolved_binding,
                self.arguments_access_for_use(use_site)?,
            )
        };
        let capture_reference = if arguments_access.is_some() {
            false
        } else {
            match resolution_kind {
                ResolutionKind::Dynamic | ResolutionKind::Global | ResolutionKind::Unresolved => {
                    true
                }
                ResolutionKind::Local | ResolutionKind::Captured => {
                    let binding_id = binding.ok_or(LoweringError::MissingResolvedBinding {
                        expr: expr_id,
                        name,
                    })?;
                    let binding_record = self.binding(binding_id)?;
                    matches!(
                        binding_record.storage_class,
                        StorageClass::DynamicLookup | StorageClass::DynamicVariableLookup
                    )
                }
            }
        };
        let reference = if capture_reference {
            let reference = self.alloc_temp()?;
            self.emit_capture_name(reference, name)?;
            Some(reference)
        } else {
            None
        };

        Ok(PreparedReferenceTarget::Identifier(
            PreparedIdentifierTarget {
                expr_id,
                name,
                resolution_kind,
                binding,
                arguments_access,
                reference,
            },
        ))
    }

    fn prepare_super_static_reference(
        &mut self,
        object: ExprId,
        property: AtomId,
    ) -> LoweringResult<PreparedReferenceTarget> {
        let receiver = self.lower_super_receiver()?;
        let key = self.alloc_temp()?;
        self.emit_load_atom_string(key, property)?;
        let base = self.alloc_temp()?;
        self.emit_super_base(base, self.ast().get_expr(object).span())?;
        Ok(PreparedReferenceTarget::SuperProperty {
            receiver,
            key,
            base,
            span: self.ast().get_expr(object).span(),
        })
    }

    fn prepare_super_computed_reference(
        &mut self,
        object: ExprId,
        property: ExprId,
        usage: ReferenceUsage,
    ) -> LoweringResult<PreparedReferenceTarget> {
        let receiver = self.lower_super_receiver()?;
        let raw_key = self.lower_expr_to_temp(property)?;
        let base = self.alloc_temp()?;
        self.emit_super_base(base, self.ast().get_expr(object).span())?;
        let key = if usage == ReferenceUsage::ReadWrite {
            let key = self.alloc_temp()?;
            self.emit_to_property_key(key, raw_key)?;
            key
        } else {
            raw_key
        };
        Ok(PreparedReferenceTarget::SuperProperty {
            receiver,
            key,
            base,
            span: self.ast().get_expr(object).span(),
        })
    }

    fn prepare_private_reference(
        &mut self,
        expr_id: ExprId,
        object: ExprId,
        property: AtomId,
        usage: ReferenceUsage,
    ) -> LoweringResult<PreparedReferenceTarget> {
        let get_descriptor = if usage == ReferenceUsage::ReadWrite {
            let (descriptor_index, _) =
                self.resolved_private_field_access(expr_id, property, false)?;
            let descriptor = self.alloc_temp()?;
            let descriptor_smi = i16::try_from(descriptor_index)
                .map_err(|_| LoweringError::UnsupportedExpression { expr: expr_id })?;
            self.emit_load_smi(descriptor, descriptor_smi)?;
            Some(descriptor)
        } else {
            None
        };
        let (set_descriptor_index, class_depth) =
            self.resolved_private_field_access(expr_id, property, true)?;
        let receiver = self.lower_expr_to_temp(object)?;
        let set_descriptor = self.alloc_temp()?;
        let descriptor_smi = i16::try_from(set_descriptor_index)
            .map_err(|_| LoweringError::UnsupportedExpression { expr: expr_id })?;
        self.emit_load_smi(set_descriptor, descriptor_smi)?;
        let depth = self.alloc_temp()?;
        let depth_smi = i16::try_from(class_depth)
            .map_err(|_| LoweringError::UnsupportedExpression { expr: expr_id })?;
        self.emit_load_smi(depth, depth_smi)?;
        Ok(PreparedReferenceTarget::Private(
            PreparedPrivateReferenceTarget {
                expr_id,
                receiver,
                get_descriptor,
                set_descriptor,
                depth,
                span: self.ast().get_expr(object).span(),
            },
        ))
    }
}

impl PreparedIdentifierTarget {
    fn load(self, compiler: &mut FunctionCompiler<'_, '_>, dest: u16) -> LoweringResult<()> {
        if let Some((depth, slot)) = self.arguments_access {
            return compiler.emit_load_env_slot(dest, depth, slot);
        }
        if let Some(reference) = self.reference {
            compiler.emit_load_captured_name(dest, reference)
        } else {
            compiler.lower_identifier(self.expr_id, self.name, dest)
        }
    }

    fn assign(self, compiler: &mut FunctionCompiler<'_, '_>, value: u16) -> LoweringResult<()> {
        if let Some((depth, slot)) = self.arguments_access {
            return compiler.emit_assign_env_slot(value, depth, slot);
        }
        if let Some(reference) = self.reference {
            return compiler.emit_assign_captured_name(value, reference);
        }
        match self.resolution_kind {
            ResolutionKind::Local | ResolutionKind::Captured => {
                let binding = self.binding.ok_or(LoweringError::MissingResolvedBinding {
                    expr: self.expr_id,
                    name: self.name,
                })?;
                compiler.assign_binding_value(binding, self.name, value)
            }
            ResolutionKind::Global | ResolutionKind::Unresolved => {
                compiler.emit_assign_global(value, self.name)
            }
            ResolutionKind::Dynamic => compiler.emit_assign_name(value, self.name),
        }
    }

    fn store(self, compiler: &mut FunctionCompiler<'_, '_>, value: u16) -> LoweringResult<()> {
        if let Some((depth, slot)) = self.arguments_access {
            return compiler.emit_store_env_slot(value, depth, slot);
        }
        if let Some(reference) = self.reference {
            return compiler.emit_assign_captured_name(value, reference);
        }
        match self.resolution_kind {
            ResolutionKind::Local | ResolutionKind::Captured => {
                let binding = self.binding.ok_or(LoweringError::MissingResolvedBinding {
                    expr: self.expr_id,
                    name: self.name,
                })?;
                compiler.store_binding_value(binding, self.name, value)
            }
            ResolutionKind::Global | ResolutionKind::Unresolved => {
                compiler.emit_store_global(value, self.name)
            }
            ResolutionKind::Dynamic => compiler.emit_assign_name(value, self.name),
        }
    }
}

impl PreparedPrivateReferenceTarget {
    fn load(self, compiler: &mut FunctionCompiler<'_, '_>, dest: u16) -> LoweringResult<()> {
        let descriptor = self
            .get_descriptor
            .ok_or(LoweringError::UnsupportedExpression { expr: self.expr_id })?;
        compiler.emit_internal_builtin_call_into(
            internal_private_field_get_builtin(),
            &[self.receiver, descriptor, self.depth],
            self.span,
            dest,
        )
    }

    fn assign(self, compiler: &mut FunctionCompiler<'_, '_>, value: u16) -> LoweringResult<()> {
        compiler.emit_internal_builtin_call_into(
            internal_private_field_set_builtin(),
            &[self.receiver, self.set_descriptor, value, self.depth],
            self.span,
            value,
        )
    }
}
