use crate::public::{dispatch_internal_spec_like_builtin, PublicBuiltinDispatchContext};
use crate::{BuiltinEntryMetadata, BuiltinInvocation};
use lyng_js_common::{AtomId, WellKnownAtom};
use lyng_js_env::Agent;
use lyng_js_gc::{AllocationLifetime, PrimitiveMutator};
use lyng_js_objects::{
    FunctionObjectData, FunctionThisMode, ObjectAllocation, ObjectColdData, ObjectFlags,
    OrdinaryObjectData, PrimitiveWrapperKind,
};
use lyng_js_types::{
    internal_array_index_of_builtin, internal_array_pop_builtin, internal_array_push_builtin,
    internal_bind_function_private_env_builtin, internal_capture_arrow_context_builtin,
    internal_construct_super_builtin, internal_construct_super_spread_builtin,
    internal_define_class_getter_property_builtin, internal_define_class_setter_property_builtin,
    internal_define_getter_property_builtin, internal_define_method_property_builtin,
    internal_define_private_field_builtin, internal_define_setter_property_builtin,
    internal_direct_eval_builtin, internal_dynamic_import_builtin, internal_function_call_builtin,
    internal_get_instance_field_key_builtin, internal_get_template_object_builtin,
    internal_import_meta_builtin, internal_install_instance_field_key_builtin,
    internal_instance_of_builtin, internal_object_has_own_property_builtin,
    internal_object_literal_set_prototype_builtin, internal_object_to_string_builtin,
    internal_private_field_get_builtin, internal_private_field_init_builtin,
    internal_private_field_set_builtin, internal_private_has_builtin,
    internal_set_function_home_object_builtin, internal_string_index_of_builtin,
    internal_string_replace_builtin, internal_super_property_get_builtin,
    internal_super_property_set_builtin, internal_template_to_string_builtin,
    internal_throw_type_error_builtin, BuiltinFunctionId, EnvironmentRef, ObjectRef,
    PropertyDescriptor, PropertyKey, RealmRef, ShapeId, Value,
};
use std::collections::HashMap;

/// Cold compatibility cache for the reserved internal builtin namespace.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InternalBuiltinCache {
    realms: HashMap<RealmRef, InternalRealmBuiltins>,
}

/// Per-realm object set backing the reserved internal builtin namespace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InternalRealmBuiltins {
    object_prototype: ObjectRef,
    function_prototype: ObjectRef,
    array_prototype: ObjectRef,
    string_prototype: ObjectRef,
    number_prototype: ObjectRef,
    bigint_prototype: ObjectRef,
    boolean_prototype: ObjectRef,
    function_call: ObjectRef,
    string_replace: ObjectRef,
    string_index_of: ObjectRef,
    array_index_of: ObjectRef,
    array_push: ObjectRef,
    array_pop: ObjectRef,
    object_to_string: ObjectRef,
    object_has_own_property: ObjectRef,
    template_to_string: ObjectRef,
    get_template_object: ObjectRef,
    instance_of: ObjectRef,
    define_method_property: ObjectRef,
    define_getter_property: ObjectRef,
    define_setter_property: ObjectRef,
    define_class_getter_property: ObjectRef,
    define_class_setter_property: ObjectRef,
    define_private_field: ObjectRef,
    private_field_init: ObjectRef,
    private_field_get: ObjectRef,
    private_field_set: ObjectRef,
    private_has: ObjectRef,
    super_property_get: ObjectRef,
    super_property_set: ObjectRef,
    construct_super: ObjectRef,
    construct_super_spread: ObjectRef,
    set_function_home_object: ObjectRef,
    object_literal_set_prototype: ObjectRef,
    bind_function_private_env: ObjectRef,
    capture_arrow_context: ObjectRef,
    install_instance_field_key: ObjectRef,
    get_instance_field_key: ObjectRef,
    throw_type_error: ObjectRef,
    import_meta: ObjectRef,
    dynamic_import: ObjectRef,
    direct_eval: ObjectRef,
}

impl InternalRealmBuiltins {
    #[inline]
    pub const fn object_prototype(self) -> ObjectRef {
        self.object_prototype
    }

    #[inline]
    pub const fn function_prototype(self) -> ObjectRef {
        self.function_prototype
    }

    #[inline]
    pub const fn array_prototype(self) -> ObjectRef {
        self.array_prototype
    }

    #[inline]
    pub const fn string_prototype(self) -> ObjectRef {
        self.string_prototype
    }

    #[inline]
    pub const fn number_prototype(self) -> ObjectRef {
        self.number_prototype
    }

    #[inline]
    pub const fn bigint_prototype(self) -> ObjectRef {
        self.bigint_prototype
    }

    #[inline]
    pub const fn boolean_prototype(self) -> ObjectRef {
        self.boolean_prototype
    }

    #[inline]
    pub fn builtin_object(self, entry: BuiltinFunctionId) -> Option<ObjectRef> {
        if entry == internal_function_call_builtin() {
            return Some(self.function_call);
        }
        if entry == internal_string_replace_builtin() {
            return Some(self.string_replace);
        }
        if entry == internal_string_index_of_builtin() {
            return Some(self.string_index_of);
        }
        if entry == internal_array_index_of_builtin() {
            return Some(self.array_index_of);
        }
        if entry == internal_array_push_builtin() {
            return Some(self.array_push);
        }
        if entry == internal_array_pop_builtin() {
            return Some(self.array_pop);
        }
        if entry == internal_object_to_string_builtin() {
            return Some(self.object_to_string);
        }
        if entry == internal_template_to_string_builtin() {
            return Some(self.template_to_string);
        }
        if entry == internal_get_template_object_builtin() {
            return Some(self.get_template_object);
        }
        if entry == internal_instance_of_builtin() {
            return Some(self.instance_of);
        }
        if entry == internal_define_method_property_builtin() {
            return Some(self.define_method_property);
        }
        if entry == internal_define_getter_property_builtin() {
            return Some(self.define_getter_property);
        }
        if entry == internal_define_setter_property_builtin() {
            return Some(self.define_setter_property);
        }
        if entry == internal_define_class_getter_property_builtin() {
            return Some(self.define_class_getter_property);
        }
        if entry == internal_define_class_setter_property_builtin() {
            return Some(self.define_class_setter_property);
        }
        if entry == internal_define_private_field_builtin() {
            return Some(self.define_private_field);
        }
        if entry == internal_private_field_init_builtin() {
            return Some(self.private_field_init);
        }
        if entry == internal_private_field_get_builtin() {
            return Some(self.private_field_get);
        }
        if entry == internal_private_field_set_builtin() {
            return Some(self.private_field_set);
        }
        if entry == internal_private_has_builtin() {
            return Some(self.private_has);
        }
        if entry == internal_super_property_get_builtin() {
            return Some(self.super_property_get);
        }
        if entry == internal_super_property_set_builtin() {
            return Some(self.super_property_set);
        }
        if entry == internal_construct_super_builtin() {
            return Some(self.construct_super);
        }
        if entry == internal_construct_super_spread_builtin() {
            return Some(self.construct_super_spread);
        }
        if entry == internal_object_has_own_property_builtin() {
            return Some(self.object_has_own_property);
        }
        if entry == internal_set_function_home_object_builtin() {
            return Some(self.set_function_home_object);
        }
        if entry == internal_object_literal_set_prototype_builtin() {
            return Some(self.object_literal_set_prototype);
        }
        if entry == internal_bind_function_private_env_builtin() {
            return Some(self.bind_function_private_env);
        }
        if entry == internal_capture_arrow_context_builtin() {
            return Some(self.capture_arrow_context);
        }
        if entry == internal_install_instance_field_key_builtin() {
            return Some(self.install_instance_field_key);
        }
        if entry == internal_get_instance_field_key_builtin() {
            return Some(self.get_instance_field_key);
        }
        if entry == internal_throw_type_error_builtin() {
            return Some(self.throw_type_error);
        }
        if entry == internal_import_meta_builtin() {
            return Some(self.import_meta);
        }
        if entry == internal_dynamic_import_builtin() {
            return Some(self.dynamic_import);
        }
        if entry == internal_direct_eval_builtin() {
            return Some(self.direct_eval);
        }
        None
    }
}

impl InternalBuiltinCache {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ensure_realm_builtins(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
    ) -> Option<InternalRealmBuiltins> {
        if let Some(builtins) = self.realms.get(&realm).copied() {
            return Some(builtins);
        }

        let realm_record = agent.realm(realm)?;
        let root_shape = realm_record.root_shape()?;
        let global_env = realm_record.global_env();
        let existing_intrinsics = realm_record.intrinsics();
        let empty_string = agent.alloc_runtime_string(
            "",
            Some(WellKnownAtom::Empty.id()),
            AllocationLifetime::Default,
        );

        let builtins = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let object_prototype = existing_intrinsics.object_prototype().unwrap_or_else(|| {
                objects.alloc_object(
                    &mut mutator,
                    ObjectAllocation::ordinary(root_shape),
                    AllocationLifetime::Default,
                )
            });
            let function_prototype =
                existing_intrinsics.function_prototype().unwrap_or_else(|| {
                    objects.alloc_object(
                        &mut mutator,
                        ObjectAllocation::ordinary(root_shape)
                            .with_prototype(Some(object_prototype)),
                        AllocationLifetime::Default,
                    )
                });
            let array_prototype = existing_intrinsics.array_prototype().unwrap_or_else(|| {
                objects.alloc_object(
                    &mut mutator,
                    ObjectAllocation::ordinary(root_shape)
                        .with_flags(ObjectFlags::extensible().union(ObjectFlags::ENGINE_ARRAY))
                        .with_prototype(Some(object_prototype)),
                    AllocationLifetime::Default,
                )
            });
            let string_prototype = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape)
                    .with_prototype(Some(object_prototype))
                    .with_cold_data(ObjectColdData::Ordinary(
                        OrdinaryObjectData::PrimitiveWrapper(PrimitiveWrapperKind::String),
                    ))
                    .with_primitive_wrapper_value(Value::from_string_ref(empty_string)),
                AllocationLifetime::Default,
            );
            let number_prototype = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape)
                    .with_prototype(Some(object_prototype))
                    .with_cold_data(ObjectColdData::Ordinary(
                        OrdinaryObjectData::PrimitiveWrapper(PrimitiveWrapperKind::Number),
                    ))
                    .with_primitive_wrapper_value(Value::from_smi(0)),
                AllocationLifetime::Default,
            );
            let bigint_prototype = existing_intrinsics.bigint_prototype().unwrap_or_else(|| {
                objects.alloc_object(
                    &mut mutator,
                    ObjectAllocation::ordinary(root_shape).with_prototype(Some(object_prototype)),
                    AllocationLifetime::Default,
                )
            });
            let boolean_prototype = existing_intrinsics.boolean_prototype().unwrap_or_else(|| {
                objects.alloc_object(
                    &mut mutator,
                    ObjectAllocation::ordinary(root_shape).with_prototype(Some(object_prototype)),
                    AllocationLifetime::Default,
                )
            });
            let mut alloc_builtin = |entry, mutator: &mut PrimitiveMutator<'_>| {
                alloc_internal_builtin_function(
                    objects,
                    mutator,
                    realm,
                    global_env,
                    root_shape,
                    function_prototype,
                    entry,
                )
            };

            InternalRealmBuiltins {
                object_prototype,
                function_prototype,
                array_prototype,
                string_prototype,
                number_prototype,
                bigint_prototype,
                boolean_prototype,
                function_call: alloc_builtin(internal_function_call_builtin(), &mut mutator),
                string_replace: alloc_builtin(internal_string_replace_builtin(), &mut mutator),
                string_index_of: alloc_builtin(internal_string_index_of_builtin(), &mut mutator),
                array_index_of: alloc_builtin(internal_array_index_of_builtin(), &mut mutator),
                array_push: alloc_builtin(internal_array_push_builtin(), &mut mutator),
                array_pop: alloc_builtin(internal_array_pop_builtin(), &mut mutator),
                object_to_string: alloc_builtin(internal_object_to_string_builtin(), &mut mutator),
                object_has_own_property: alloc_builtin(
                    internal_object_has_own_property_builtin(),
                    &mut mutator,
                ),
                template_to_string: alloc_builtin(
                    internal_template_to_string_builtin(),
                    &mut mutator,
                ),
                get_template_object: alloc_builtin(
                    internal_get_template_object_builtin(),
                    &mut mutator,
                ),
                instance_of: alloc_builtin(internal_instance_of_builtin(), &mut mutator),
                define_method_property: alloc_builtin(
                    internal_define_method_property_builtin(),
                    &mut mutator,
                ),
                define_getter_property: alloc_builtin(
                    internal_define_getter_property_builtin(),
                    &mut mutator,
                ),
                define_setter_property: alloc_builtin(
                    internal_define_setter_property_builtin(),
                    &mut mutator,
                ),
                define_class_getter_property: alloc_builtin(
                    internal_define_class_getter_property_builtin(),
                    &mut mutator,
                ),
                define_class_setter_property: alloc_builtin(
                    internal_define_class_setter_property_builtin(),
                    &mut mutator,
                ),
                define_private_field: alloc_builtin(
                    internal_define_private_field_builtin(),
                    &mut mutator,
                ),
                private_field_init: alloc_builtin(
                    internal_private_field_init_builtin(),
                    &mut mutator,
                ),
                private_field_get: alloc_builtin(
                    internal_private_field_get_builtin(),
                    &mut mutator,
                ),
                private_field_set: alloc_builtin(
                    internal_private_field_set_builtin(),
                    &mut mutator,
                ),
                private_has: alloc_builtin(internal_private_has_builtin(), &mut mutator),
                super_property_get: alloc_builtin(
                    internal_super_property_get_builtin(),
                    &mut mutator,
                ),
                super_property_set: alloc_builtin(
                    internal_super_property_set_builtin(),
                    &mut mutator,
                ),
                construct_super: alloc_builtin(internal_construct_super_builtin(), &mut mutator),
                construct_super_spread: alloc_builtin(
                    internal_construct_super_spread_builtin(),
                    &mut mutator,
                ),
                set_function_home_object: alloc_builtin(
                    internal_set_function_home_object_builtin(),
                    &mut mutator,
                ),
                object_literal_set_prototype: alloc_builtin(
                    internal_object_literal_set_prototype_builtin(),
                    &mut mutator,
                ),
                bind_function_private_env: alloc_builtin(
                    internal_bind_function_private_env_builtin(),
                    &mut mutator,
                ),
                capture_arrow_context: alloc_builtin(
                    internal_capture_arrow_context_builtin(),
                    &mut mutator,
                ),
                install_instance_field_key: alloc_builtin(
                    internal_install_instance_field_key_builtin(),
                    &mut mutator,
                ),
                get_instance_field_key: alloc_builtin(
                    internal_get_instance_field_key_builtin(),
                    &mut mutator,
                ),
                throw_type_error: alloc_builtin(internal_throw_type_error_builtin(), &mut mutator),
                import_meta: alloc_builtin(internal_import_meta_builtin(), &mut mutator),
                dynamic_import: alloc_builtin(internal_dynamic_import_builtin(), &mut mutator),
                direct_eval: alloc_builtin(internal_direct_eval_builtin(), &mut mutator),
            }
        });

        let updated_intrinsics = existing_intrinsics
            .with_object_prototype(Some(builtins.object_prototype))
            .with_function_prototype(Some(builtins.function_prototype))
            .with_array_prototype(Some(builtins.array_prototype))
            .with_string_prototype(Some(builtins.string_prototype))
            .with_number_prototype(Some(builtins.number_prototype))
            .with_bigint_prototype(Some(builtins.bigint_prototype))
            .with_boolean_prototype(Some(builtins.boolean_prototype));
        if !agent.set_realm_intrinsics(realm, updated_intrinsics) {
            return None;
        }

        let index_of = agent.atoms_mut().intern_collectible("indexOf");
        let pop = agent.atoms_mut().intern_collectible("pop");
        let push = agent.atoms_mut().intern_collectible("push");
        let replace = agent.atoms_mut().intern_collectible("replace");
        let has_own_property = agent.atoms_mut().intern_collectible("hasOwnProperty");

        define_builtin_method(
            agent,
            builtins.string_prototype,
            index_of,
            builtins.string_index_of,
        );
        define_builtin_method(
            agent,
            builtins.string_prototype,
            replace,
            builtins.string_replace,
        );
        define_builtin_method(
            agent,
            builtins.array_prototype,
            index_of,
            builtins.array_index_of,
        );
        define_builtin_method(agent, builtins.array_prototype, pop, builtins.array_pop);
        define_builtin_method(agent, builtins.array_prototype, push, builtins.array_push);
        define_builtin_method(
            agent,
            builtins.object_prototype,
            WellKnownAtom::toString.id(),
            builtins.object_to_string,
        );
        define_builtin_method(
            agent,
            builtins.object_prototype,
            has_own_property,
            builtins.object_has_own_property,
        );
        define_data_property_with_attrs(
            agent,
            builtins.throw_type_error,
            PropertyKey::from_atom(WellKnownAtom::length.id()),
            Value::from_smi(0),
            false,
            false,
            false,
        );
        define_data_property_with_attrs(
            agent,
            builtins.throw_type_error,
            PropertyKey::from_atom(WellKnownAtom::name.id()),
            Value::from_string_ref(empty_string),
            false,
            false,
            false,
        );
        prevent_extensions(agent, builtins.throw_type_error);

        self.realms.insert(realm, builtins);
        Some(builtins)
    }

    #[inline]
    pub fn builtin_constant(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        entry: BuiltinFunctionId,
    ) -> Option<Value> {
        self.ensure_realm_builtins(agent, realm)?
            .builtin_object(entry)
            .map(Value::from_object_ref)
    }
}

/// Compatibility metadata for the reserved internal builtin namespace.
#[inline]
pub fn internal_builtin_metadata(entry: BuiltinFunctionId) -> Option<BuiltinEntryMetadata> {
    if entry == internal_function_call_builtin() {
        return Some(BuiltinEntryMetadata::new("call", 1, false, false));
    }
    if entry == internal_string_replace_builtin() {
        return Some(BuiltinEntryMetadata::new("replace", 2, false, false));
    }
    if entry == internal_string_index_of_builtin() {
        return Some(BuiltinEntryMetadata::new("indexOf", 1, false, false));
    }
    if entry == internal_array_index_of_builtin() {
        return Some(BuiltinEntryMetadata::new("indexOf", 1, false, false));
    }
    if entry == internal_array_push_builtin() {
        return Some(BuiltinEntryMetadata::new("push", 1, false, false));
    }
    if entry == internal_array_pop_builtin() {
        return Some(BuiltinEntryMetadata::new("pop", 0, false, false));
    }
    if entry == internal_object_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == internal_template_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_templateToString",
            1,
            false,
            false,
        ));
    }
    if entry == internal_get_template_object_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_getTemplateObject",
            0,
            false,
            false,
        ));
    }
    if entry == internal_instance_of_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_instanceOf",
            2,
            false,
            false,
        ));
    }
    if entry == internal_define_method_property_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_defineMethodProperty",
            3,
            false,
            false,
        ));
    }
    if entry == internal_define_getter_property_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_defineGetterProperty",
            3,
            false,
            false,
        ));
    }
    if entry == internal_define_setter_property_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_defineSetterProperty",
            3,
            false,
            false,
        ));
    }
    if entry == internal_define_class_getter_property_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_defineClassGetterProperty",
            3,
            false,
            false,
        ));
    }
    if entry == internal_define_class_setter_property_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_defineClassSetterProperty",
            3,
            false,
            false,
        ));
    }
    if entry == internal_define_private_field_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_definePrivateField",
            6,
            false,
            false,
        ));
    }
    if entry == internal_private_field_init_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_privateFieldInit",
            3,
            false,
            false,
        ));
    }
    if entry == internal_private_field_get_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_privateFieldGet",
            3,
            false,
            false,
        ));
    }
    if entry == internal_private_field_set_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_privateFieldSet",
            4,
            false,
            false,
        ));
    }
    if entry == internal_private_has_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_privateHas",
            3,
            false,
            false,
        ));
    }
    if entry == internal_super_property_get_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_superPropertyGet",
            2,
            false,
            false,
        ));
    }
    if entry == internal_super_property_set_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_superPropertySet",
            3,
            false,
            false,
        ));
    }
    if entry == internal_construct_super_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_constructSuper",
            0,
            false,
            false,
        ));
    }
    if entry == internal_construct_super_spread_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_constructSuperSpread",
            1,
            false,
            false,
        ));
    }
    if entry == internal_object_has_own_property_builtin() {
        return Some(BuiltinEntryMetadata::new("hasOwnProperty", 1, false, false));
    }
    if entry == internal_set_function_home_object_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_setFunctionHomeObject",
            2,
            false,
            false,
        ));
    }
    if entry == internal_object_literal_set_prototype_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_objectLiteralSetPrototype",
            2,
            false,
            false,
        ));
    }
    if entry == internal_bind_function_private_env_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_bindFunctionPrivateEnv",
            3,
            false,
            false,
        ));
    }
    if entry == internal_capture_arrow_context_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_captureArrowContext",
            3,
            false,
            false,
        ));
    }
    if entry == internal_install_instance_field_key_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_installInstanceFieldKey",
            3,
            false,
            false,
        ));
    }
    if entry == internal_get_instance_field_key_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_getInstanceFieldKey",
            2,
            false,
            false,
        ));
    }
    if entry == internal_throw_type_error_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_throwTypeError",
            0,
            false,
            false,
        ));
    }
    if entry == internal_import_meta_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_importMeta",
            0,
            false,
            false,
        ));
    }
    if entry == internal_dynamic_import_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_dynamicImport",
            2,
            false,
            false,
        ));
    }
    if entry == internal_direct_eval_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "internal_directEval",
            1,
            false,
            false,
        ));
    }
    None
}

/// Narrow VM-side bridge for executing the reserved internal builtin namespace.
///
/// Methods that return `Result` use the implementing context's error channel.
/// That keeps internal helper dispatch independent from VM error storage while
/// still allowing abrupt completions and VM failures to propagate uniformly.
#[allow(clippy::missing_errors_doc)]
pub trait InternalBuiltinDispatchContext {
    type Error;

    fn function_call_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn template_to_string_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn get_template_object_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn instance_of_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn define_method_property_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn define_getter_property_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn define_setter_property_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn define_class_getter_property_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn define_class_setter_property_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn define_private_field_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn private_field_init_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn private_field_get_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn private_field_set_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn private_has_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn super_property_get_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn super_property_set_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn construct_super_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn construct_super_spread_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn set_function_home_object_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn object_literal_set_prototype_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn bind_function_private_env_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn capture_arrow_context_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn install_instance_field_key_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn get_instance_field_key_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn throw_type_error_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;

    fn direct_eval_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error>;
}

/// Dispatches one reserved internal builtin entry through the builtins-owned bridge.
///
/// Returns `Ok(None)` when `entry` is not part of the reserved internal builtin
/// namespace.
///
/// # Errors
///
/// Propagates errors from the selected [`InternalBuiltinDispatchContext`] hook.
pub fn dispatch_internal_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if let Some(result) = dispatch_internal_spec_like_builtin(context, entry, invocation)? {
        return Ok(Some(result));
    }
    if entry == internal_function_call_builtin() {
        return context.function_call_builtin(invocation).map(Some);
    }
    if entry == internal_template_to_string_builtin() {
        return context.template_to_string_builtin(invocation).map(Some);
    }
    if entry == internal_get_template_object_builtin() {
        return context.get_template_object_builtin(invocation).map(Some);
    }
    if entry == internal_instance_of_builtin() {
        return context.instance_of_builtin(invocation).map(Some);
    }
    if entry == internal_define_method_property_builtin() {
        return context.define_method_property_builtin(invocation).map(Some);
    }
    if entry == internal_define_getter_property_builtin() {
        return context.define_getter_property_builtin(invocation).map(Some);
    }
    if entry == internal_define_setter_property_builtin() {
        return context.define_setter_property_builtin(invocation).map(Some);
    }
    if entry == internal_define_class_getter_property_builtin() {
        return context
            .define_class_getter_property_builtin(invocation)
            .map(Some);
    }
    if entry == internal_define_class_setter_property_builtin() {
        return context
            .define_class_setter_property_builtin(invocation)
            .map(Some);
    }
    if entry == internal_define_private_field_builtin() {
        return context.define_private_field_builtin(invocation).map(Some);
    }
    if entry == internal_private_field_init_builtin() {
        return context.private_field_init_builtin(invocation).map(Some);
    }
    if entry == internal_private_field_get_builtin() {
        return context.private_field_get_builtin(invocation).map(Some);
    }
    if entry == internal_private_field_set_builtin() {
        return context.private_field_set_builtin(invocation).map(Some);
    }
    if entry == internal_private_has_builtin() {
        return context.private_has_builtin(invocation).map(Some);
    }
    if entry == internal_super_property_get_builtin() {
        return context.super_property_get_builtin(invocation).map(Some);
    }
    if entry == internal_super_property_set_builtin() {
        return context.super_property_set_builtin(invocation).map(Some);
    }
    if entry == internal_construct_super_builtin() {
        return context.construct_super_builtin(invocation).map(Some);
    }
    if entry == internal_construct_super_spread_builtin() {
        return context.construct_super_spread_builtin(invocation).map(Some);
    }
    if entry == internal_set_function_home_object_builtin() {
        return context
            .set_function_home_object_builtin(invocation)
            .map(Some);
    }
    if entry == internal_object_literal_set_prototype_builtin() {
        return context
            .object_literal_set_prototype_builtin(invocation)
            .map(Some);
    }
    if entry == internal_bind_function_private_env_builtin() {
        return context
            .bind_function_private_env_builtin(invocation)
            .map(Some);
    }
    if entry == internal_capture_arrow_context_builtin() {
        return context.capture_arrow_context_builtin(invocation).map(Some);
    }
    if entry == internal_install_instance_field_key_builtin() {
        return context
            .install_instance_field_key_builtin(invocation)
            .map(Some);
    }
    if entry == internal_get_instance_field_key_builtin() {
        return context.get_instance_field_key_builtin(invocation).map(Some);
    }
    if entry == internal_throw_type_error_builtin() {
        return context.throw_type_error_builtin(invocation).map(Some);
    }
    if entry == internal_direct_eval_builtin() {
        return context.direct_eval_builtin(invocation).map(Some);
    }
    Ok(None)
}

fn alloc_internal_builtin_function(
    objects: &mut lyng_js_objects::ObjectRuntime,
    mutator: &mut PrimitiveMutator<'_>,
    realm: RealmRef,
    global_env: EnvironmentRef,
    root_shape: ShapeId,
    function_prototype: ObjectRef,
    entry: BuiltinFunctionId,
) -> ObjectRef {
    debug_assert!(internal_builtin_metadata(entry).is_some());
    let function_data = FunctionObjectData::native(realm, global_env, entry)
        .with_this_mode(FunctionThisMode::Strict);
    objects.alloc_object(
        mutator,
        ObjectAllocation::function(root_shape)
            .with_prototype(Some(function_prototype))
            .with_cold_data(ObjectColdData::Function(function_data)),
        AllocationLifetime::Default,
    )
}

fn define_builtin_method(agent: &mut Agent, object: ObjectRef, name: AtomId, function: ObjectRef) {
    define_data_property_with_attrs(
        agent,
        object,
        PropertyKey::from_atom(name),
        Value::from_object_ref(function),
        true,
        false,
        true,
    );
}

fn define_data_property_with_attrs(
    agent: &mut Agent,
    object: ObjectRef,
    key: PropertyKey,
    value: Value,
    writable: bool,
    enumerable: bool,
    configurable: bool,
) {
    let mut descriptor = PropertyDescriptor::new();
    descriptor.set_value(value);
    descriptor.set_writable(writable);
    descriptor.set_enumerable(enumerable);
    descriptor.set_configurable(configurable);
    let defined = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.define_own_property(
            &mut mutator,
            object,
            key,
            descriptor,
            AllocationLifetime::Default,
        )
    });
    assert!(
        matches!(defined, Ok(true)),
        "reserved internal builtin property installation should succeed"
    );
}

fn prevent_extensions(agent: &mut Agent, object: ObjectRef) {
    let prevented = agent
        .with_heap_and_objects(|heap, objects| objects.prevent_extensions(heap.view(), object));
    assert!(
        matches!(prevented, Ok(true)),
        "builtin object should accept PreventExtensions"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_env::Runtime;
    use lyng_js_host::NoopHostHooks;

    #[test]
    fn internal_builtin_cache_bootstraps_compatibility_objects_and_constants() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent.default_realm().expect("default realm should exist");
        let mut cache = InternalBuiltinCache::new();

        let builtins = cache
            .ensure_realm_builtins(agent, realm.id())
            .expect("default realm should bootstrap reserved internal builtins");
        let second = cache
            .ensure_realm_builtins(agent, realm.id())
            .expect("bootstrap should be cached per realm");
        assert_eq!(builtins, second);
        assert_eq!(
            agent
                .realm(realm.id())
                .unwrap()
                .intrinsics()
                .object_prototype(),
            Some(builtins.object_prototype())
        );
        assert_eq!(
            agent
                .realm(realm.id())
                .unwrap()
                .intrinsics()
                .function_prototype(),
            Some(builtins.function_prototype())
        );
        assert_eq!(
            agent
                .realm(realm.id())
                .unwrap()
                .intrinsics()
                .string_prototype(),
            Some(builtins.string_prototype())
        );
        assert_eq!(
            agent
                .objects()
                .primitive_wrapper_kind(builtins.string_prototype()),
            Some(PrimitiveWrapperKind::String)
        );
        assert_eq!(
            agent
                .realm(realm.id())
                .unwrap()
                .intrinsics()
                .number_prototype(),
            Some(builtins.number_prototype())
        );
        assert_eq!(
            agent
                .objects()
                .primitive_wrapper_kind(builtins.number_prototype()),
            Some(PrimitiveWrapperKind::Number)
        );
        assert_eq!(
            agent
                .objects()
                .primitive_wrapper_value(agent.heap().view(), builtins.number_prototype()),
            Some(Value::from_smi(0))
        );
        assert_eq!(
            agent
                .realm(realm.id())
                .unwrap()
                .intrinsics()
                .bigint_prototype(),
            Some(builtins.bigint_prototype())
        );
        assert_eq!(
            agent
                .objects()
                .primitive_wrapper_kind(builtins.bigint_prototype()),
            None
        );
        assert_eq!(
            agent
                .realm(realm.id())
                .unwrap()
                .intrinsics()
                .boolean_prototype(),
            Some(builtins.boolean_prototype())
        );
    }
}
