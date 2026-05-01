use super::state::ClassInstanceElementPlan;
use super::*;
use lyng_js_types::{
    internal_construct_super_spread_builtin, internal_require_constructor_builtin,
};

#[derive(Clone, Copy)]
enum StaticPublicFieldKey {
    Atom(AtomId),
    Register(u16),
}

#[derive(Clone, Copy)]
enum PendingStaticClassElement {
    PublicField {
        key: StaticPublicFieldKey,
        value: Option<ExprId>,
    },
    PrivateField {
        name: AtomId,
        value: Option<ExprId>,
        span: Span,
    },
    Block {
        body: NodeList<StmtId>,
        span: Span,
    },
}

struct PrivateElementDescriptorLookup {
    by_name_and_kind: HashMap<(AtomId, lyng_js_sema::ClassPrivateElementKind), (u32, Span)>,
}

#[derive(Clone, Copy)]
struct PrivateElementDefinitionScratch {
    arguments: u16,
}

impl PrivateElementDefinitionScratch {
    const ARGUMENT_COUNT: u16 = 6;

    fn allocate(compiler: &mut FunctionCompiler<'_, '_>) -> LoweringResult<Self> {
        let arguments = compiler
            .builder
            .try_alloc_registers(Self::ARGUMENT_COUNT)
            .ok_or(LoweringError::RegisterOverflow { register: u16::MAX })?;
        let last_argument = arguments + Self::ARGUMENT_COUNT - 1;
        let _ = compiler.encode_register(last_argument)?;
        Ok(Self { arguments })
    }
}

#[derive(Clone, Copy)]
struct PrivateElementInitializerScratch {
    arguments: u16,
}

impl PrivateElementInitializerScratch {
    const ARGUMENT_COUNT: u16 = 3;

    fn allocate(compiler: &mut FunctionCompiler<'_, '_>) -> LoweringResult<Self> {
        let arguments = compiler
            .builder
            .try_alloc_registers(Self::ARGUMENT_COUNT)
            .ok_or(LoweringError::RegisterOverflow { register: u16::MAX })?;
        let last_argument = arguments + Self::ARGUMENT_COUNT - 1;
        let _ = compiler.encode_register(last_argument)?;
        Ok(Self { arguments })
    }
}

impl PrivateElementDescriptorLookup {
    fn from_layout(layout: &lyng_js_sema::ClassPrivateLayoutRecord) -> Self {
        let mut by_name_and_kind = HashMap::with_capacity(layout.entries().len());
        for (index, entry) in layout.entries().iter().copied().enumerate() {
            by_name_and_kind
                .entry((entry.name(), entry.kind()))
                .or_insert_with(|| {
                    (
                        u32::try_from(index).expect("descriptor index should fit u32"),
                        entry.span(),
                    )
                });
        }
        Self { by_name_and_kind }
    }

    fn get(
        &self,
        name: AtomId,
        kind: lyng_js_sema::ClassPrivateElementKind,
    ) -> Option<(u32, Span)> {
        self.by_name_and_kind.get(&(name, kind)).copied()
    }
}

impl<'a, 'b> FunctionCompiler<'a, 'b> {
    pub(super) fn lower_class_expression(
        &mut self,
        expr_id: ExprId,
        name: Option<AtomId>,
        super_class: Option<ExprId>,
        body: lyng_js_ast::NodeList<lyng_js_ast::ClassElementId>,
        dest: u16,
    ) -> LoweringResult<()> {
        let class_span = self.ast().get_expr(expr_id).span();
        self.lower_class_definition(name, super_class, body, class_span, dest)
            .map_err(|error| match error {
                LoweringError::UnsupportedDeclaration { .. } => {
                    LoweringError::UnsupportedExpression { expr: expr_id }
                }
                other => other,
            })
    }

    pub(super) fn lower_class_declaration(
        &mut self,
        decl_id: DeclId,
        name: Option<AtomId>,
        super_class: Option<ExprId>,
        body: lyng_js_ast::NodeList<lyng_js_ast::ClassElementId>,
    ) -> LoweringResult<()> {
        let name = name.ok_or(LoweringError::UnsupportedDeclaration { decl: decl_id })?;
        let binding_id = self.find_named_binding(name, DeclarationKind::Class)?;
        let value_register = self.alloc_temp()?;
        let class_span = self.ast().get_decl(decl_id).span();
        self.lower_class_definition(Some(name), super_class, body, class_span, value_register)?;
        self.store_binding_value(binding_id, name, value_register)
    }

    pub(super) fn lower_class_definition(
        &mut self,
        name: Option<AtomId>,
        super_class: Option<ExprId>,
        body: lyng_js_ast::NodeList<lyng_js_ast::ClassElementId>,
        class_span: Span,
        dest: u16,
    ) -> LoweringResult<()> {
        let elements = self.ast().get_class_element_list(body).to_vec();
        let constructor = self.class_constructor_method(body)?;
        let has_private_entries = self
            .class_layout_for_span(body, class_span)
            .is_some_and(|layout| !layout.entries().is_empty());
        let evaluated_super = super_class
            .map(|expr| self.lower_expr_to_temp(expr))
            .transpose()?;
        let super_is_literal_null = super_class
            .map(|expr| matches!(self.ast().get_expr(expr), Expr::NullLiteral { .. }))
            .unwrap_or(false);
        let super_span = super_class.map(|expr| self.ast().get_expr(expr).span());
        let has_derived_heritage = super_class.is_some();
        let mut next_computed_instance_field_key = 0u32;
        let instance_elements = elements
            .iter()
            .filter_map(
                |element_id| match self.ast().get_class_element(*element_id) {
                    lyng_js_ast::ClassElement::Method {
                        key,
                        kind,
                        private: true,
                        r#static: false,
                        ..
                    } => match self.ast().get_expr(*key) {
                        Expr::Identifier { name, .. } => {
                            Some(ClassInstanceElementPlan::PrivateElement {
                                name: *name,
                                kind: match kind {
                                    lyng_js_ast::MethodKind::Method
                                    | lyng_js_ast::MethodKind::Constructor => {
                                        lyng_js_sema::ClassPrivateElementKind::Method
                                    }
                                    lyng_js_ast::MethodKind::Get => {
                                        lyng_js_sema::ClassPrivateElementKind::Getter
                                    }
                                    lyng_js_ast::MethodKind::Set => {
                                        lyng_js_sema::ClassPrivateElementKind::Setter
                                    }
                                },
                                value: None,
                            })
                        }
                        _ => None,
                    },
                    lyng_js_ast::ClassElement::Property {
                        key,
                        value,
                        computed,
                        r#static: false,
                        private: false,
                        ..
                    } => Some(ClassInstanceElementPlan::PublicField {
                        key: *key,
                        value: *value,
                        computed: *computed,
                        computed_key_index: if *computed {
                            let index = next_computed_instance_field_key;
                            next_computed_instance_field_key =
                                next_computed_instance_field_key.saturating_add(1);
                            Some(index)
                        } else {
                            None
                        },
                    }),
                    lyng_js_ast::ClassElement::Property {
                        key,
                        value,
                        private: true,
                        r#static: false,
                        ..
                    } => match self.ast().get_expr(*key) {
                        Expr::Identifier { name, .. } => {
                            Some(ClassInstanceElementPlan::PrivateElement {
                                name: *name,
                                kind: lyng_js_sema::ClassPrivateElementKind::Field,
                                value: *value,
                            })
                        }
                        _ => None,
                    },
                    _ => None,
                },
            )
            .collect::<Vec<_>>();
        if let Some(constructor) = constructor {
            let child_index = self.ensure_child_index(constructor)?;
            self.emit_create_closure(dest, child_index, self.ast().get_function(constructor).span)?;
        } else {
            self.emit_synthetic_default_class_constructor(
                dest,
                name,
                has_derived_heritage,
                &instance_elements,
                body,
                class_span,
            )?;
        }

        let prototype = self.alloc_temp()?;
        self.emit_get_property_by_atom(prototype, dest, WellKnownAtom::prototype.id())?;
        self.set_class_prototype_chain(
            dest,
            prototype,
            evaluated_super,
            super_is_literal_null,
            super_span,
        )?;

        let class_name = name.unwrap_or_else(|| self.state.atoms.intern_collectible(""));
        let name_value = self.alloc_temp()?;
        self.emit_load_atom_string(name_value, class_name)?;
        self.emit_set_function_name(dest, name_value)?;

        if let Some(name) = name {
            if let Some(binding_id) = self.class_self_binding(body, class_span, name)? {
                self.store_binding_value(binding_id, name, dest)?;
            }
        }

        let previous_class_contexts = self.active_class_contexts.len();
        self.active_class_contexts.push(ActiveClassContext {
            class_object: dest,
            prototype,
            has_private_entries,
        });
        self.bind_function_home_object(dest, prototype, class_span)?;
        self.bind_function_private_env(dest, class_span)?;

        let previous_class_body = self.active_class_body.replace(body);
        let previous_class_span = self.active_class_span.replace(class_span);
        let private_define_scratch = if has_private_entries {
            Some(PrivateElementDefinitionScratch::allocate(self)?)
        } else {
            None
        };
        let private_initializer_scratch = if has_private_entries {
            Some(PrivateElementInitializerScratch::allocate(self)?)
        } else {
            None
        };
        let mut next_computed_instance_field_key = 0u32;
        let mut pending_static_elements = Vec::new();
        for element_id in elements {
            match self.ast().get_class_element(element_id).clone() {
                lyng_js_ast::ClassElement::Method {
                    kind,
                    key,
                    value,
                    private: true,
                    r#static,
                    span,
                    ..
                } => {
                    if kind == lyng_js_ast::MethodKind::Constructor {
                        continue;
                    }
                    let private_name = match self.ast().get_expr(key) {
                        Expr::Identifier { name, .. } => *name,
                        _ => {
                            return Err(LoweringError::UnsupportedDeclaration {
                                decl: DeclId::new(0),
                            })
                        }
                    };
                    let method = self.alloc_temp()?;
                    let child_index = self.ensure_child_index(value)?;
                    self.emit_create_closure(method, child_index, span)?;
                    let name_register = self.alloc_temp()?;
                    self.emit_load_private_function_name(name_register, private_name, kind)?;
                    self.emit_set_function_name(method, name_register)?;
                    let home_object = if r#static { dest } else { prototype };
                    self.bind_function_home_object(method, home_object, span)?;
                    self.bind_function_private_env(method, span)?;
                    let private_kind = match kind {
                        lyng_js_ast::MethodKind::Method | lyng_js_ast::MethodKind::Constructor => {
                            lyng_js_sema::ClassPrivateElementKind::Method
                        }
                        lyng_js_ast::MethodKind::Get => {
                            lyng_js_sema::ClassPrivateElementKind::Getter
                        }
                        lyng_js_ast::MethodKind::Set => {
                            lyng_js_sema::ClassPrivateElementKind::Setter
                        }
                    };
                    self.emit_define_private_element_with_scratch(
                        dest,
                        prototype,
                        private_name,
                        r#static,
                        private_kind,
                        Some(method),
                        span,
                        private_define_scratch
                            .expect("private definition scratch should exist for private methods"),
                    )?;
                    if r#static {
                        let descriptor_index = self.private_element_descriptor_index(
                            body,
                            private_name,
                            private_kind,
                        )?;
                        self.lower_private_element_initializer_with_scratch(
                            dest,
                            descriptor_index,
                            None,
                            span,
                            None,
                            None,
                            None,
                            private_initializer_scratch.expect(
                                "private initializer scratch should exist for private methods",
                            ),
                        )?;
                    }
                }
                lyng_js_ast::ClassElement::Method {
                    kind,
                    key,
                    value,
                    computed,
                    private: false,
                    r#static,
                    span,
                } => {
                    if kind == lyng_js_ast::MethodKind::Constructor {
                        continue;
                    }
                    let target = if r#static { dest } else { prototype };
                    let (key_register, name_register) =
                        self.lower_class_element_key(key, computed)?;
                    let method = self.alloc_temp()?;
                    let child_index = self.ensure_child_index(value)?;
                    self.emit_create_closure(method, child_index, span)?;
                    if kind == lyng_js_ast::MethodKind::Method {
                        if let Some(name_register) = name_register {
                            self.emit_set_function_name(method, name_register)?;
                        }
                    }
                    let home_object = if r#static { dest } else { prototype };
                    self.bind_function_home_object(method, home_object, span)?;
                    self.bind_function_private_env(method, span)?;
                    match kind {
                        lyng_js_ast::MethodKind::Method => {
                            self.emit_internal_builtin_call(
                                internal_define_method_property_builtin(),
                                &[target, key_register, method],
                                span,
                            )?;
                        }
                        lyng_js_ast::MethodKind::Get => {
                            self.emit_internal_builtin_call(
                                internal_define_class_getter_property_builtin(),
                                &[target, key_register, method],
                                span,
                            )?;
                        }
                        lyng_js_ast::MethodKind::Set => {
                            self.emit_internal_builtin_call(
                                internal_define_class_setter_property_builtin(),
                                &[target, key_register, method],
                                span,
                            )?;
                        }
                        lyng_js_ast::MethodKind::Constructor => unreachable!(),
                    }
                }
                lyng_js_ast::ClassElement::Property {
                    key,
                    value,
                    computed,
                    private: false,
                    r#static: false,
                    span,
                    ..
                } if computed => {
                    let key_register = self.lower_expr_to_temp(key)?;
                    self.emit_install_instance_field_key(
                        dest,
                        next_computed_instance_field_key,
                        key_register,
                        span,
                    )?;
                    next_computed_instance_field_key =
                        next_computed_instance_field_key.saturating_add(1);
                }
                lyng_js_ast::ClassElement::Property {
                    key,
                    value,
                    computed,
                    private,
                    r#static: true,
                    span,
                    ..
                } => {
                    if private {
                        let private_name = match self.ast().get_expr(key) {
                            Expr::Identifier { name, .. } => *name,
                            _ => {
                                return Err(LoweringError::UnsupportedDeclaration {
                                    decl: DeclId::new(0),
                                })
                            }
                        };
                        self.emit_define_private_element_with_scratch(
                            dest,
                            prototype,
                            private_name,
                            true,
                            lyng_js_sema::ClassPrivateElementKind::Field,
                            None,
                            span,
                            private_define_scratch.expect(
                                "private definition scratch should exist for private fields",
                            ),
                        )?;
                        pending_static_elements.push(PendingStaticClassElement::PrivateField {
                            name: private_name,
                            value,
                            span,
                        });
                    } else {
                        let key = self.lower_static_public_field_key(key, computed)?;
                        pending_static_elements
                            .push(PendingStaticClassElement::PublicField { key, value });
                    }
                }
                lyng_js_ast::ClassElement::Property {
                    key,
                    private: true,
                    r#static: false,
                    span,
                    ..
                } => {
                    let private_name = match self.ast().get_expr(key) {
                        Expr::Identifier { name, .. } => *name,
                        _ => {
                            return Err(LoweringError::UnsupportedDeclaration {
                                decl: DeclId::new(0),
                            })
                        }
                    };
                    self.emit_define_private_element_with_scratch(
                        dest,
                        prototype,
                        private_name,
                        false,
                        lyng_js_sema::ClassPrivateElementKind::Field,
                        None,
                        span,
                        private_define_scratch
                            .expect("private definition scratch should exist for private fields"),
                    )?;
                }
                lyng_js_ast::ClassElement::StaticBlock { body, span } => {
                    pending_static_elements.push(PendingStaticClassElement::Block { body, span });
                }
                lyng_js_ast::ClassElement::Property { .. }
                | lyng_js_ast::ClassElement::InvalidElement { .. } => {}
            }
        }
        for element in pending_static_elements {
            self.lower_pending_static_class_element(
                dest,
                body,
                element,
                private_initializer_scratch,
            )?;
        }
        self.active_class_body = previous_class_body;
        self.active_class_span = previous_class_span;
        self.active_class_contexts.truncate(previous_class_contexts);

        Ok(())
    }

    fn class_layout_for_span(
        &self,
        body: lyng_js_ast::NodeList<lyng_js_ast::ClassElementId>,
        class_span: Span,
    ) -> Option<&lyng_js_sema::ClassPrivateLayoutRecord> {
        self.state
            .sema
            .class_private_layouts
            .get_with_span(body, class_span)
            .or_else(|| self.state.sema.class_private_layouts.get(body))
    }

    fn active_class_layout(
        &self,
        body: lyng_js_ast::NodeList<lyng_js_ast::ClassElementId>,
    ) -> Option<&lyng_js_sema::ClassPrivateLayoutRecord> {
        self.active_class_span
            .and_then(|class_span| self.class_layout_for_span(body, class_span))
            .or_else(|| self.state.sema.class_private_layouts.get(body))
    }

    fn class_constructor_method(
        &self,
        body: lyng_js_ast::NodeList<lyng_js_ast::ClassElementId>,
    ) -> LoweringResult<Option<FunctionId>> {
        for &element in self.ast().get_class_element_list(body) {
            if let lyng_js_ast::ClassElement::Method {
                kind: lyng_js_ast::MethodKind::Constructor,
                value,
                ..
            } = self.ast().get_class_element(element)
            {
                return Ok(Some(*value));
            }
        }
        Ok(None)
    }

    fn emit_load_private_function_name(
        &mut self,
        dest: u16,
        private_name: AtomId,
        kind: lyng_js_ast::MethodKind,
    ) -> LoweringResult<()> {
        let raw_name = self.state.atoms.resolve(private_name).to_owned();
        let text = match kind {
            lyng_js_ast::MethodKind::Get => format!("get #{raw_name}"),
            lyng_js_ast::MethodKind::Set => format!("set #{raw_name}"),
            lyng_js_ast::MethodKind::Method | lyng_js_ast::MethodKind::Constructor => {
                format!("#{raw_name}")
            }
        };
        let name = self.state.atoms.intern(&text);
        self.emit_load_atom_string(dest, name)
    }

    fn private_name_with_hash_atom(&mut self, private_name: AtomId) -> AtomId {
        let raw_name = self.state.atoms.resolve(private_name).to_owned();
        self.state.atoms.intern(&format!("#{raw_name}"))
    }

    fn class_self_binding(
        &self,
        body: lyng_js_ast::NodeList<lyng_js_ast::ClassElementId>,
        class_span: Span,
        name: AtomId,
    ) -> LoweringResult<Option<SemanticBindingId>> {
        let Some(scope) = self
            .class_layout_for_span(body, class_span)
            .map(lyng_js_sema::ClassPrivateLayoutRecord::scope)
        else {
            return Ok(None);
        };
        Ok(self
            .state
            .sema
            .binding_table
            .as_slice()
            .iter()
            .enumerate()
            .find_map(|(index, binding)| {
                (binding.kind == DeclarationKind::ClassName
                    && binding.name == name
                    && binding.scope == scope)
                    .then_some(SemanticBindingId::new(index as u32))
            }))
    }

    fn lower_class_element_key(
        &mut self,
        key: ExprId,
        computed: bool,
    ) -> LoweringResult<(u16, Option<u16>)> {
        if !computed {
            if let Some(atom) = self.named_property_atom(key)? {
                let key_value = self.alloc_temp()?;
                self.emit_load_atom_string(key_value, atom)?;
                return Ok((key_value, Some(key_value)));
            }
        }
        let key_value = self.lower_expr_to_temp(key)?;
        Ok((key_value, None))
    }

    fn lower_static_public_field_key(
        &mut self,
        key: ExprId,
        computed: bool,
    ) -> LoweringResult<StaticPublicFieldKey> {
        if !computed {
            if let Some(atom) = self.named_property_atom(key)? {
                return Ok(StaticPublicFieldKey::Atom(atom));
            }
        }
        let raw_key = self.lower_expr_to_temp(key)?;
        let property_key = self.alloc_temp()?;
        self.emit_to_property_key(property_key, raw_key)?;
        Ok(StaticPublicFieldKey::Register(property_key))
    }

    fn lower_pending_static_class_element(
        &mut self,
        class_object: u16,
        class_body: NodeList<lyng_js_ast::ClassElementId>,
        element: PendingStaticClassElement,
        private_initializer_scratch: Option<PrivateElementInitializerScratch>,
    ) -> LoweringResult<()> {
        match element {
            PendingStaticClassElement::PublicField { key, value } => {
                let inferred_name = match key {
                    StaticPublicFieldKey::Atom(atom) => Some(atom),
                    StaticPublicFieldKey::Register(_) => None,
                };
                let value_register = self.lower_class_field_value(
                    value,
                    inferred_name,
                    Some(class_object),
                    Some(class_object),
                )?;
                match key {
                    StaticPublicFieldKey::Atom(atom) => {
                        self.emit_define_property_by_atom(class_object, value_register, atom)
                    }
                    StaticPublicFieldKey::Register(key_register) => {
                        self.emit_define_keyed_property(class_object, value_register, key_register)
                    }
                }
            }
            PendingStaticClassElement::PrivateField { name, value, span } => {
                let inferred_name = Some(self.private_name_with_hash_atom(name));
                let descriptor_index = self.private_element_descriptor_index(
                    class_body,
                    name,
                    lyng_js_sema::ClassPrivateElementKind::Field,
                )?;
                self.lower_private_element_initializer_with_scratch(
                    class_object,
                    descriptor_index,
                    value,
                    span,
                    inferred_name,
                    Some(class_object),
                    Some(class_object),
                    private_initializer_scratch.ok_or(LoweringError::UnsupportedDeclaration {
                        decl: DeclId::new(0),
                    })?,
                )
            }
            PendingStaticClassElement::Block { body, span } => {
                let previous_override = self.this_override_register.replace(class_object);
                let previous_home_object = self.super_home_object_override.replace(class_object);
                let result = self.lower_statement_list_with_disposal(body, span);
                self.this_override_register = previous_override;
                self.super_home_object_override = previous_home_object;
                result
            }
        }
    }

    fn lower_class_field_value(
        &mut self,
        value: Option<ExprId>,
        inferred_name: Option<AtomId>,
        this_override: Option<u16>,
        home_object_override: Option<u16>,
    ) -> LoweringResult<u16> {
        let previous_override = this_override
            .and_then(|this_override| self.this_override_register.replace(this_override));
        let previous_home_object = home_object_override
            .and_then(|home_object| self.super_home_object_override.replace(home_object));
        let previous_in_class_field_initializer =
            std::mem::replace(&mut self.in_class_field_initializer, true);
        let value_register: LoweringResult<u16> = if let Some(value) = value {
            let value_register = self.alloc_temp()?;
            self.lower_initializer_with_inferred_name(value, inferred_name, value_register)?;
            Ok(value_register)
        } else {
            let undefined = self.alloc_temp()?;
            self.emit_load_undefined(undefined)?;
            Ok(undefined)
        };
        self.in_class_field_initializer = previous_in_class_field_initializer;
        if this_override.is_some() {
            self.this_override_register = previous_override;
        }
        if home_object_override.is_some() {
            self.super_home_object_override = previous_home_object;
        }
        value_register
    }

    fn lower_class_field_initializer(
        &mut self,
        target: u16,
        key: ExprId,
        value: Option<ExprId>,
        computed: bool,
        this_override: Option<u16>,
        home_object_override: Option<u16>,
    ) -> LoweringResult<()> {
        let named_atom = if computed {
            None
        } else {
            self.named_property_atom(key)?
        };
        let value_register =
            self.lower_class_field_value(value, named_atom, this_override, home_object_override)?;

        if let Some(atom) = named_atom {
            return self.emit_define_property_by_atom(target, value_register, atom);
        }

        let key_register = self.lower_expr_to_temp(key)?;
        self.emit_define_keyed_property(target, value_register, key_register)
    }

    fn emit_define_private_element_with_scratch(
        &mut self,
        class_object: u16,
        prototype: u16,
        name: AtomId,
        is_static: bool,
        kind: lyng_js_sema::ClassPrivateElementKind,
        value: Option<u16>,
        span: Span,
        scratch: PrivateElementDefinitionScratch,
    ) -> LoweringResult<()> {
        let arguments = scratch.arguments;
        self.emit_move(arguments, class_object)?;
        self.emit_move(arguments + 1, prototype)?;
        self.emit_load_atom_string(arguments + 2, name)?;
        self.emit_load_bool(arguments + 3, is_static)?;
        self.emit_load_smi(arguments + 4, Self::private_element_kind_tag(kind))?;
        match value {
            Some(value) => {
                self.emit_move(arguments + 5, value)?;
                self.emit_internal_builtin_call_from_argument_range(
                    internal_define_private_field_builtin(),
                    CallRange::new(arguments, 6),
                    span,
                )
            }
            None => self.emit_internal_builtin_call_from_argument_range(
                internal_define_private_field_builtin(),
                CallRange::new(arguments, 5),
                span,
            ),
        }
    }

    fn lower_private_element_initializer_with_scratch(
        &mut self,
        target: u16,
        descriptor_index: u32,
        value: Option<ExprId>,
        span: Span,
        inferred_name: Option<AtomId>,
        this_override: Option<u16>,
        home_object_override: Option<u16>,
        scratch: PrivateElementInitializerScratch,
    ) -> LoweringResult<()> {
        let arguments = scratch.arguments;
        self.emit_move(arguments, target)?;
        let descriptor_smi =
            i16::try_from(descriptor_index).map_err(|_| LoweringError::UnsupportedDeclaration {
                decl: DeclId::new(0),
            })?;
        self.emit_load_smi(arguments + 1, descriptor_smi)?;
        let previous_override = this_override
            .and_then(|this_override| self.this_override_register.replace(this_override));
        let previous_home_object = home_object_override
            .and_then(|home_object| self.super_home_object_override.replace(home_object));
        let previous_in_class_field_initializer =
            std::mem::replace(&mut self.in_class_field_initializer, true);
        let value_register: LoweringResult<u16> = if let Some(value) = value {
            let value_register = self.alloc_temp()?;
            self.lower_initializer_with_inferred_name(value, inferred_name, value_register)?;
            Ok(value_register)
        } else {
            self.emit_load_undefined(arguments + 2)?;
            Ok(arguments + 2)
        };
        self.in_class_field_initializer = previous_in_class_field_initializer;
        let value_register = value_register?;
        if this_override.is_some() || home_object_override.is_some() {
            self.this_override_register = previous_override;
            self.super_home_object_override = previous_home_object;
        }
        if value_register != arguments + 2 {
            self.emit_move(arguments + 2, value_register)?;
        }
        self.emit_internal_builtin_call_from_argument_range(
            internal_private_field_init_builtin(),
            CallRange::new(arguments, PrivateElementInitializerScratch::ARGUMENT_COUNT),
            span,
        )
    }

    pub(super) fn emit_class_constructor_field_prologue(&mut self) -> LoweringResult<()> {
        let Some(function) = self.current_function_ast else {
            return Ok(());
        };
        let Some(plan) = self.state.class_constructor_plan(function).cloned() else {
            return Ok(());
        };
        if plan.derived {
            return Ok(());
        }
        if plan.instance_elements.is_empty() {
            return Ok(());
        }

        let this_register = self.alloc_temp()?;
        self.emit_load_this(this_register)?;
        self.emit_instance_element_initializers(
            this_register,
            &plan.instance_elements,
            plan.class_body,
        )?;
        Ok(())
    }

    pub(super) fn emit_derived_class_super_call_epilogue(
        &mut self,
        this_register: u16,
    ) -> LoweringResult<()> {
        let Some(function) = self.current_function_ast else {
            return Ok(());
        };
        let Some(plan) = self.state.class_constructor_plan(function).cloned() else {
            return Ok(());
        };
        if !plan.derived || plan.instance_elements.is_empty() {
            return Ok(());
        }
        self.emit_instance_element_initializers(
            this_register,
            &plan.instance_elements,
            plan.class_body,
        )
    }

    pub(super) fn emit_create_closure(
        &mut self,
        dest: u16,
        child_index: u16,
        span: Span,
    ) -> LoweringResult<()> {
        let instruction_offset = self.builder.emit_abx(
            Opcode::CreateClosure,
            self.encode_register(dest)?,
            u32::from(child_index),
        )?;
        self.attach_safepoint(instruction_offset, span, SafepointKind::Allocation)?;
        Ok(())
    }

    fn emit_synthetic_default_class_constructor(
        &mut self,
        dest: u16,
        name: Option<AtomId>,
        derived: bool,
        instance_elements: &[ClassInstanceElementPlan],
        class_body: lyng_js_ast::NodeList<lyng_js_ast::ClassElementId>,
        class_span: Span,
    ) -> LoweringResult<()> {
        let (id, function) = self.build_synthetic_default_class_constructor(
            name,
            derived,
            instance_elements,
            class_body,
            class_span,
        )?;
        self.state.functions.push(function);

        let child_index = self.builder.add_child_function(id)?;
        self.emit_create_closure(dest, child_index, self.root_span())
    }

    fn build_synthetic_default_class_constructor(
        &mut self,
        name: Option<AtomId>,
        derived: bool,
        instance_elements: &[ClassInstanceElementPlan],
        class_body: lyng_js_ast::NodeList<lyng_js_ast::ClassElementId>,
        class_span: Span,
    ) -> LoweringResult<(BytecodeFunctionId, BytecodeFunction)> {
        let id = self.state.alloc_function_id();
        let mut builder = BytecodeBuilder::new(id, BytecodeFunctionKind::Function);
        builder.set_name(name);
        builder.set_flags(
            BytecodeFunctionFlags::new(true, false)
                .with_class_constructor(true)
                .with_derived_class_constructor(derived),
        );
        builder.set_this_mode(ThisMode::Strict);
        builder.set_parameter_counts(0, 0);
        builder.set_needs_environment(
            derived || self.instance_elements_need_environment(instance_elements),
        );
        if derived {
            builder.set_has_rest_parameter(true);
            builder.set_environment_bindings(vec![BytecodeEnvironmentBinding::new(
                None,
                BytecodeEnvironmentSlotFlags::var_like(),
            )]);
        }
        builder.set_source_span(Some(class_span));

        if instance_elements.is_empty() && !derived {
            builder.emit_ax(Opcode::ReturnUndefined, 0)?;
            return Ok((id, builder.finish()?));
        }

        let binding_count = self.state.sema.binding_table.len();
        let current_function = self.current_function;
        let function = {
            let state = &mut *self.state;
            let scope_count = state.sema.scope_table.len();
            let mut synthetic = FunctionCompiler {
                state,
                builder,
                current_function,
                current_function_ast: None,
                current_scope: self.current_scope,
                body_scope: self.body_scope,
                scope_child_cursors: vec![0; scope_count],
                local_registers: vec![None; binding_count],
                atom_constants: HashMap::new(),
                float_constants: HashMap::new(),
                builtin_constants: HashMap::new(),
                child_indices: HashMap::new(),
                hoisted_function_decls: HashSet::new(),
                block_instantiated_function_decls: HashSet::new(),
                hoisted_default_export_functions: HashSet::new(),
                parameter_sources: Vec::new(),
                result_register: None,
                call_bridge_registers: None,
                generator_resume_registers: None,
                completion_registers: None,
                control_targets: Vec::new(),
                next_control_target_id: 1,
                finally_stack: Vec::new(),
                this_override_register: None,
                super_home_object_override: None,
                active_class_body: Some(class_body),
                active_class_span: Some(class_span),
                active_class_contexts: Vec::new(),
                active_direct_eval_scopes: self.active_direct_eval_scopes.clone(),
                in_class_field_initializer: false,
                active_disposal_scopes: Vec::new(),
            };
            synthetic.reserve_call_bridge_registers()?;
            let this_register = synthetic.alloc_temp()?;
            if derived {
                let rest_arguments = synthetic.alloc_temp()?;
                synthetic.emit_load_env_slot(rest_arguments, 0, 0)?;
                synthetic.emit_internal_builtin_call_into(
                    internal_construct_super_spread_builtin(),
                    &[rest_arguments],
                    class_span,
                    this_register,
                )?;
            } else {
                synthetic.emit_load_this(this_register)?;
            }
            if derived {
                synthetic.emit_instance_element_initializers(
                    this_register,
                    instance_elements,
                    class_body,
                )?;
                synthetic
                    .builder
                    .emit_ax(Opcode::Return, i32::from(this_register))?;
                synthetic.builder.finish()
            } else {
                synthetic.emit_instance_element_initializers(
                    this_register,
                    instance_elements,
                    class_body,
                )?;
                synthetic.builder.emit_ax(Opcode::ReturnUndefined, 0)?;
                synthetic.builder.finish()
            }
        };

        Ok((id, function?))
    }

    fn instance_elements_need_environment(
        &self,
        instance_elements: &[ClassInstanceElementPlan],
    ) -> bool {
        instance_elements.iter().any(|element| match element {
            ClassInstanceElementPlan::PublicField { value, .. }
            | ClassInstanceElementPlan::PrivateElement { value, .. } => {
                value.is_some_and(|value| {
                    matches!(
                        self.ast().get_expr(value),
                        Expr::ArrowFunctionExpression { .. }
                    )
                })
            }
        })
    }

    fn emit_instance_element_initializers(
        &mut self,
        this_register: u16,
        instance_elements: &[ClassInstanceElementPlan],
        class_body: lyng_js_ast::NodeList<lyng_js_ast::ClassElementId>,
    ) -> LoweringResult<()> {
        let private_descriptors = self
            .active_class_layout(class_body)
            .map(PrivateElementDescriptorLookup::from_layout);
        let private_initializer_scratch = if instance_elements
            .iter()
            .any(|element| matches!(element, ClassInstanceElementPlan::PrivateElement { .. }))
        {
            Some(PrivateElementInitializerScratch::allocate(self)?)
        } else {
            None
        };

        for element in instance_elements.iter().copied() {
            let ClassInstanceElementPlan::PrivateElement { name, kind, .. } = element else {
                continue;
            };
            if kind == lyng_js_sema::ClassPrivateElementKind::Field {
                continue;
            }
            let (descriptor_index, span) =
                Self::private_element_descriptor_from_lookup(&private_descriptors, name, kind)?;
            self.lower_private_element_initializer_with_scratch(
                this_register,
                descriptor_index,
                None,
                span,
                None,
                None,
                None,
                private_initializer_scratch
                    .expect("private initializer scratch should exist for private elements"),
            )?;
        }

        for element in instance_elements.iter().copied() {
            match element {
                ClassInstanceElementPlan::PublicField {
                    key,
                    value,
                    computed,
                    computed_key_index,
                } => {
                    if let Some(computed_key_index) = computed_key_index {
                        let value_register = if let Some(value) = value {
                            let previous_override =
                                self.this_override_register.replace(this_register);
                            let value_register = self.lower_expr_to_temp(value)?;
                            self.this_override_register = previous_override;
                            value_register
                        } else {
                            let undefined = self.alloc_temp()?;
                            self.emit_load_undefined(undefined)?;
                            undefined
                        };
                        let callee = self.alloc_temp()?;
                        self.emit_load_callee(callee)?;
                        let field_index = self.alloc_temp()?;
                        let field_index_smi = i16::try_from(computed_key_index).map_err(|_| {
                            LoweringError::UnsupportedDeclaration {
                                decl: DeclId::new(0),
                            }
                        })?;
                        self.emit_load_smi(field_index, field_index_smi)?;
                        let key_register = self.alloc_temp()?;
                        self.emit_internal_builtin_call_into(
                            internal_get_instance_field_key_builtin(),
                            &[callee, field_index],
                            self.ast().get_expr(key).span(),
                            key_register,
                        )?;
                        self.emit_define_keyed_property(
                            this_register,
                            value_register,
                            key_register,
                        )?;
                    } else {
                        self.lower_class_field_initializer(
                            this_register,
                            key,
                            value,
                            computed,
                            Some(this_register),
                            None,
                        )?;
                    }
                }
                ClassInstanceElementPlan::PrivateElement { name, kind, value } => {
                    if kind != lyng_js_sema::ClassPrivateElementKind::Field {
                        continue;
                    }
                    let inferred_name = Some(self.private_name_with_hash_atom(name));
                    let (descriptor_index, span) = Self::private_element_descriptor_from_lookup(
                        &private_descriptors,
                        name,
                        kind,
                    )?;
                    self.lower_private_element_initializer_with_scratch(
                        this_register,
                        descriptor_index,
                        value,
                        span,
                        inferred_name,
                        Some(this_register),
                        None,
                        private_initializer_scratch.expect(
                            "private initializer scratch should exist for private elements",
                        ),
                    )?;
                }
            }
        }
        Ok(())
    }

    fn private_element_descriptor_from_lookup(
        private_descriptors: &Option<PrivateElementDescriptorLookup>,
        name: AtomId,
        kind: lyng_js_sema::ClassPrivateElementKind,
    ) -> LoweringResult<(u32, Span)> {
        private_descriptors
            .as_ref()
            .and_then(|descriptors| descriptors.get(name, kind))
            .ok_or(LoweringError::UnsupportedDeclaration {
                decl: DeclId::new(0),
            })
    }

    fn private_element_descriptor_index(
        &self,
        body: lyng_js_ast::NodeList<lyng_js_ast::ClassElementId>,
        name: AtomId,
        kind: lyng_js_sema::ClassPrivateElementKind,
    ) -> LoweringResult<u32> {
        let layout =
            self.active_class_layout(body)
                .ok_or(LoweringError::UnsupportedDeclaration {
                    decl: DeclId::new(0),
                })?;
        layout
            .entries()
            .iter()
            .enumerate()
            .find_map(|(index, entry)| {
                (entry.name() == name && entry.kind() == kind)
                    .then_some(u32::try_from(index).expect("descriptor index should fit u32"))
            })
            .ok_or(LoweringError::UnsupportedDeclaration {
                decl: DeclId::new(0),
            })
    }

    pub(super) fn private_access_descriptor_index_for_layout(
        &self,
        layout: &lyng_js_sema::ClassPrivateLayoutRecord,
        name: AtomId,
        set_context: bool,
    ) -> LoweringResult<u32> {
        let preferred = if set_context {
            [
                lyng_js_sema::ClassPrivateElementKind::Field,
                lyng_js_sema::ClassPrivateElementKind::Setter,
                lyng_js_sema::ClassPrivateElementKind::Getter,
                lyng_js_sema::ClassPrivateElementKind::Method,
            ]
        } else {
            [
                lyng_js_sema::ClassPrivateElementKind::Field,
                lyng_js_sema::ClassPrivateElementKind::Method,
                lyng_js_sema::ClassPrivateElementKind::Getter,
                lyng_js_sema::ClassPrivateElementKind::Setter,
            ]
        };
        for kind in preferred {
            let Some(index) = layout
                .entries()
                .iter()
                .enumerate()
                .find_map(|(index, entry)| {
                    (entry.name() == name && entry.kind() == kind)
                        .then_some(u32::try_from(index).expect("descriptor index should fit u32"))
                })
            else {
                continue;
            };
            return Ok(index);
        }
        Err(LoweringError::UnsupportedDeclaration {
            decl: DeclId::new(0),
        })
    }

    pub(super) fn resolved_private_field_access(
        &self,
        expr_id: ExprId,
        property: AtomId,
        set_context: bool,
    ) -> LoweringResult<(u32, u16)> {
        let private_use = self.private_use(expr_id)?;
        let layout = self
            .state
            .sema
            .class_private_layouts
            .get_by_scope(private_use.defining_scope())
            .ok_or(LoweringError::UnsupportedExpression { expr: expr_id })?;
        let descriptor_index =
            self.private_access_descriptor_index_for_layout(layout, property, set_context)?;
        Ok((descriptor_index, private_use.class_depth()))
    }

    pub(super) fn lower_private_field_get(
        &mut self,
        expr_id: ExprId,
        object: ExprId,
        property: AtomId,
        dest: u16,
    ) -> LoweringResult<()> {
        if self.expr_continues_optional_chain(object) {
            return self
                .lower_optional_chain_private_member_continuation(expr_id, object, property, dest);
        }
        let receiver = self.lower_expr_to_temp(object)?;
        let span = self.ast().get_expr(object).span();
        self.emit_private_field_get_from_receiver(expr_id, receiver, property, span, dest)
    }

    pub(super) fn emit_private_field_get_from_receiver(
        &mut self,
        expr_id: ExprId,
        receiver: u16,
        property: AtomId,
        span: Span,
        dest: u16,
    ) -> LoweringResult<()> {
        let (descriptor_index, class_depth) =
            self.resolved_private_field_access(expr_id, property, false)?;
        let descriptor = self.alloc_temp()?;
        let descriptor_smi = i16::try_from(descriptor_index)
            .map_err(|_| LoweringError::UnsupportedExpression { expr: expr_id })?;
        self.emit_load_smi(descriptor, descriptor_smi)?;
        let depth = self.alloc_temp()?;
        let depth_smi = i16::try_from(class_depth)
            .map_err(|_| LoweringError::UnsupportedExpression { expr: expr_id })?;
        self.emit_load_smi(depth, depth_smi)?;
        self.emit_internal_builtin_call_into(
            internal_private_field_get_builtin(),
            &[receiver, descriptor, depth],
            span,
            dest,
        )
    }

    pub(super) fn lower_private_has_expression(
        &mut self,
        expr_id: ExprId,
        object: ExprId,
        property: AtomId,
        dest: u16,
    ) -> LoweringResult<()> {
        let (descriptor_index, class_depth) =
            self.resolved_private_field_access(expr_id, property, false)?;
        let receiver = self.lower_expr_to_temp(object)?;
        let descriptor = self.alloc_temp()?;
        let descriptor_smi = i16::try_from(descriptor_index)
            .map_err(|_| LoweringError::UnsupportedExpression { expr: expr_id })?;
        self.emit_load_smi(descriptor, descriptor_smi)?;
        let depth = self.alloc_temp()?;
        let depth_smi = i16::try_from(class_depth)
            .map_err(|_| LoweringError::UnsupportedExpression { expr: expr_id })?;
        self.emit_load_smi(depth, depth_smi)?;
        let span = self.ast().get_expr(object).span();
        self.emit_internal_builtin_call_into(
            internal_private_has_builtin(),
            &[receiver, descriptor, depth],
            span,
            dest,
        )
    }

    fn private_element_kind_tag(kind: lyng_js_sema::ClassPrivateElementKind) -> i16 {
        match kind {
            lyng_js_sema::ClassPrivateElementKind::Field => 0,
            lyng_js_sema::ClassPrivateElementKind::Method => 1,
            lyng_js_sema::ClassPrivateElementKind::Getter => 2,
            lyng_js_sema::ClassPrivateElementKind::Setter => 3,
        }
    }

    fn emit_install_instance_field_key(
        &mut self,
        class_object: u16,
        field_index: u32,
        key_value: u16,
        span: Span,
    ) -> LoweringResult<()> {
        let field_index_register = self.alloc_temp()?;
        let field_index_smi =
            i16::try_from(field_index).map_err(|_| LoweringError::UnsupportedDeclaration {
                decl: DeclId::new(0),
            })?;
        self.emit_load_smi(field_index_register, field_index_smi)?;
        self.emit_internal_builtin_call(
            internal_install_instance_field_key_builtin(),
            &[class_object, field_index_register, key_value],
            span,
        )
    }

    pub(super) fn bind_function_home_object(
        &mut self,
        function: u16,
        home_object: u16,
        span: Span,
    ) -> LoweringResult<()> {
        self.emit_internal_builtin_call(
            internal_set_function_home_object_builtin(),
            &[function, home_object],
            span,
        )
    }

    pub(super) fn bind_function_private_env(
        &mut self,
        function: u16,
        span: Span,
    ) -> LoweringResult<()> {
        let contexts = self.active_class_contexts.clone();
        for context in contexts {
            let needs_private_env = self.alloc_temp()?;
            self.emit_load_bool(needs_private_env, context.has_private_entries)?;
            self.emit_internal_builtin_call(
                internal_bind_function_private_env_builtin(),
                &[
                    function,
                    context.class_object,
                    context.prototype,
                    needs_private_env,
                ],
                span,
            )?;
        }
        Ok(())
    }

    fn set_class_prototype_chain(
        &mut self,
        class_object: u16,
        prototype: u16,
        super_value: Option<u16>,
        super_is_literal_null: bool,
        super_span: Option<Span>,
    ) -> LoweringResult<()> {
        let Some(super_value) = super_value else {
            return Ok(());
        };
        let super_span = super_span.unwrap_or(self.root_span());

        if !super_is_literal_null {
            self.emit_internal_builtin_call(
                internal_require_constructor_builtin(),
                &[super_value],
                super_span,
            )?;
            self.emit_internal_builtin_call(
                object_set_prototype_of_builtin(),
                &[class_object, super_value],
                super_span,
            )?;
            let super_prototype = self.alloc_temp()?;
            self.emit_get_property_by_atom(
                super_prototype,
                super_value,
                WellKnownAtom::prototype.id(),
            )?;
            self.emit_internal_builtin_call(
                object_set_prototype_of_builtin(),
                &[prototype, super_prototype],
                super_span,
            )?;
            return Ok(());
        }

        let null_value = self.alloc_temp()?;
        self.emit_load_null(null_value)?;
        self.emit_internal_builtin_call(
            object_set_prototype_of_builtin(),
            &[prototype, null_value],
            super_span,
        )
    }

    pub(super) fn emit_internal_builtin_call(
        &mut self,
        builtin: BuiltinFunctionId,
        arguments: &[u16],
        span: Span,
    ) -> LoweringResult<()> {
        let result = self.alloc_temp()?;
        self.emit_internal_builtin_call_into(builtin, arguments, span, result)
    }

    pub(super) fn emit_internal_builtin_call_into(
        &mut self,
        builtin: BuiltinFunctionId,
        arguments: &[u16],
        span: Span,
        dest: u16,
    ) -> LoweringResult<()> {
        let _ = self.emit_internal_builtin_call_into_with_offset(builtin, arguments, span, dest)?;
        Ok(())
    }

    pub(super) fn emit_internal_builtin_call_into_with_offset(
        &mut self,
        builtin: BuiltinFunctionId,
        arguments: &[u16],
        span: Span,
        dest: u16,
    ) -> LoweringResult<u32> {
        self.emit_internal_builtin_call_into_with_offset_and_this(
            builtin, arguments, span, dest, None,
        )
    }

    pub(super) fn emit_internal_builtin_call_into_with_offset_and_this(
        &mut self,
        builtin: BuiltinFunctionId,
        arguments: &[u16],
        span: Span,
        dest: u16,
        this_override: Option<u16>,
    ) -> LoweringResult<u32> {
        let callee = self.alloc_temp()?;
        self.emit_load_builtin(callee, builtin)?;
        let this_value = if let Some(this_override) = this_override {
            this_override
        } else {
            let this_value = self.alloc_temp()?;
            self.emit_load_undefined(this_value)?;
            this_value
        };
        let argument_range = self.materialize_argument_block(arguments)?;
        let (call_result, call_callee, call_this, move_back) =
            self.bridge_call_registers(dest, callee, this_value)?;
        let instruction_offset = self.builder.emit_call(
            self.encode_register(call_result)?,
            self.encode_register(call_callee)?,
            self.encode_register(call_this)?,
            argument_range,
        )?;
        self.attach_safepoint(instruction_offset, span, SafepointKind::Allocation)?;
        if let Some(dest) = move_back {
            self.emit_move(dest, call_result)?;
        }
        Ok(instruction_offset)
    }

    fn emit_internal_builtin_call_from_argument_range(
        &mut self,
        builtin: BuiltinFunctionId,
        arguments: CallRange,
        span: Span,
    ) -> LoweringResult<()> {
        let bridges = self
            .call_bridge_registers
            .expect("call bridge registers should be reserved before lowering");
        self.emit_load_builtin(bridges.callee, builtin)?;
        self.emit_load_undefined(bridges.this_value)?;
        let instruction_offset = self.builder.emit_call(
            self.encode_register(bridges.result)?,
            self.encode_register(bridges.callee)?,
            self.encode_register(bridges.this_value)?,
            arguments,
        )?;
        self.attach_safepoint(instruction_offset, span, SafepointKind::Allocation)?;
        Ok(())
    }
}
