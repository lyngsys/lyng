use super::{
    ArrayFamilyPrototypes, BinaryDataFamilyPrototypes, CollectionFamilyPrototypes,
    DateFamilyPrototypes, ErrorFamilyPrototypes, FamilyInstallContext, FunctionFamilyPrototypes,
    IteratorFamilyPrototypes, JsonFamilyObjects, ModuleFamilyPrototypes,
    ObjectReflectionFamilyObjects, PrimitiveFamilyObjects, PrimitiveFamilyPrototypes,
    PromiseDisposalFamilyPrototypes, PublicRealmPrototypeHandles, RegExpFamilyPrototypes,
    StringFamilyPrototypes,
};
use crate::internal::InternalRealmBuiltins;
use crate::public::{
    allocate_builtin_function_object, allocate_builtin_ordinary_object,
    allocate_builtin_primitive_wrapper_object, public_builtin_metadata, reparent_builtin_object,
};
use lyng_js_env::{Agent, Intrinsics};
use lyng_js_gc::{AllocationLifetime, SymbolFlags};
use lyng_js_objects::{ObjectFlags, PrimitiveWrapperKind};
use lyng_js_types::{
    function_prototype_builtin, internal_throw_type_error_builtin, EnvironmentRef, ObjectRef,
    RealmRef, ShapeId, Value,
};

#[derive(Clone, Copy)]
pub(in crate::public) struct ScaffoldingRequest<'a> {
    pub(in crate::public) realm: RealmRef,
    pub(in crate::public) global_env: EnvironmentRef,
    pub(in crate::public) root_shape: ShapeId,
    pub(in crate::public) internal: InternalRealmBuiltins,
    pub(in crate::public) intrinsics: &'a Intrinsics,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::public) struct PublicRealmScaffolding {
    pub(in crate::public) cx: FamilyInstallContext,
    pub(in crate::public) function: FunctionFamilyPrototypes,
    pub(in crate::public) iterator: IteratorFamilyPrototypes,
    pub(in crate::public) collection: CollectionFamilyPrototypes,
    pub(in crate::public) binary_data: BinaryDataFamilyPrototypes,
    pub(in crate::public) array: ArrayFamilyPrototypes,
    pub(in crate::public) string: StringFamilyPrototypes,
    pub(in crate::public) regexp: RegExpFamilyPrototypes,
    pub(in crate::public) date: DateFamilyPrototypes,
    pub(in crate::public) primitive: PrimitiveFamilyPrototypes,
    pub(in crate::public) primitive_objects: PrimitiveFamilyObjects,
    pub(in crate::public) json: JsonFamilyObjects,
    pub(in crate::public) object_reflection: ObjectReflectionFamilyObjects,
    pub(in crate::public) module: ModuleFamilyPrototypes,
    pub(in crate::public) error: ErrorFamilyPrototypes,
    pub(in crate::public) promise_disposal: PromiseDisposalFamilyPrototypes,
    pub(in crate::public) intrinsics: PublicRealmPrototypeHandles,
}

#[derive(Clone, Copy)]
struct RootObjects {
    object_prototype: ObjectRef,
    function_prototype: ObjectRef,
}

#[derive(Clone, Copy)]
struct IteratorScaffolding {
    iterator_prototype: ObjectRef,
    async_from_sync_iterator_prototype: ObjectRef,
    array_iterator_prototype: ObjectRef,
    map_iterator_prototype: ObjectRef,
    set_iterator_prototype: ObjectRef,
    string_iterator_prototype: ObjectRef,
    family: IteratorFamilyPrototypes,
}

#[derive(Clone, Copy)]
struct NamespaceObjects {
    math: ObjectRef,
    json: ObjectRef,
    reflect: ObjectRef,
    abstract_module_source_prototype: ObjectRef,
}

pub(in crate::public) fn allocate_public_realm_scaffolding(
    agent: &mut Agent,
    request: ScaffoldingRequest<'_>,
) -> PublicRealmScaffolding {
    let root = allocate_root_objects(agent, request);
    let collection = allocate_collection_prototypes(agent, request, root.object_prototype);
    let binary_data = allocate_binary_data_prototypes(agent, request, root.object_prototype);
    let iterators = allocate_iterator_prototypes(agent, request, root.object_prototype);
    let function = allocate_function_prototypes(agent, request, root, &iterators);
    let primitive = allocate_primitive_prototypes(agent, request, root.object_prototype);
    let array = allocate_array_prototypes(agent, request, root.object_prototype);
    let text = allocate_text_prototypes(agent, request, root.object_prototype);
    let namespaces = allocate_namespace_objects(agent, request, root.object_prototype);
    let error = allocate_error_prototypes(agent, request, root.object_prototype);
    let promise_disposal =
        allocate_promise_disposal_prototypes(agent, request, root.object_prototype);
    let cx = FamilyInstallContext {
        realm: request.realm,
        global_env: request.global_env,
        root_shape: request.root_shape,
        function_prototype: root.function_prototype,
        object_prototype: root.object_prototype,
    };
    PublicRealmScaffolding {
        cx,
        function,
        iterator: iterators.family,
        collection,
        binary_data,
        array,
        string: StringFamilyPrototypes {
            string_prototype: text.string,
        },
        regexp: RegExpFamilyPrototypes {
            regexp_prototype: text.regexp,
        },
        date: DateFamilyPrototypes {
            date_prototype: text.date,
        },
        primitive,
        primitive_objects: PrimitiveFamilyObjects {
            math: namespaces.math,
        },
        json: JsonFamilyObjects {
            json: namespaces.json,
        },
        object_reflection: ObjectReflectionFamilyObjects {
            reflect: namespaces.reflect,
        },
        module: ModuleFamilyPrototypes {
            abstract_module_source_prototype: namespaces.abstract_module_source_prototype,
        },
        error,
        promise_disposal,
        intrinsics: public_intrinsic_handles(
            collection,
            binary_data,
            iterators,
            array.array_prototype,
        ),
    }
}

fn allocate_root_objects(agent: &mut Agent, request: ScaffoldingRequest<'_>) -> RootObjects {
    let object_prototype = allocate_builtin_ordinary_object(agent, request.root_shape, None);
    let _ = agent
        .objects_mut()
        .insert_flags(object_prototype, ObjectFlags::IMMUTABLE_PROTOTYPE);
    let function_prototype = allocate_builtin_function_object(
        agent,
        request.realm,
        request.global_env,
        request.root_shape,
        object_prototype,
        object_prototype,
        function_prototype_builtin(),
        public_builtin_metadata(function_prototype_builtin()).unwrap(),
        None,
    );
    if let Some(throw_type_error) = request
        .internal
        .builtin_object(internal_throw_type_error_builtin())
    {
        reparent_builtin_object(agent, throw_type_error, Some(function_prototype));
    }
    RootObjects {
        object_prototype,
        function_prototype,
    }
}

fn allocate_function_prototypes(
    agent: &mut Agent,
    request: ScaffoldingRequest<'_>,
    root: RootObjects,
    iterators: &IteratorScaffolding,
) -> FunctionFamilyPrototypes {
    let async_function_prototype = ordinary_intrinsic(
        agent,
        request,
        request.intrinsics.async_function_prototype(),
        root.function_prototype,
    );
    let async_generator_function_prototype = ordinary_intrinsic(
        agent,
        request,
        request.intrinsics.async_generator_function_prototype(),
        root.function_prototype,
    );
    let generator_function_prototype = ordinary_intrinsic(
        agent,
        request,
        request.intrinsics.generator_function_prototype(),
        root.function_prototype,
    );
    let generator_prototype = ordinary_intrinsic(
        agent,
        request,
        request.intrinsics.generator_prototype(),
        iterators.iterator_prototype,
    );
    let async_generator_prototype = ordinary_intrinsic(
        agent,
        request,
        request.intrinsics.async_generator_prototype(),
        iterators.family.async_iterator_prototype,
    );
    FunctionFamilyPrototypes {
        async_function_prototype,
        async_generator_function_prototype,
        async_generator_prototype,
        generator_function_prototype,
        generator_prototype,
    }
}

fn allocate_collection_prototypes(
    agent: &mut Agent,
    request: ScaffoldingRequest<'_>,
    object_prototype: ObjectRef,
) -> CollectionFamilyPrototypes {
    CollectionFamilyPrototypes {
        map_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.map_prototype(),
            object_prototype,
        ),
        set_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.set_prototype(),
            object_prototype,
        ),
        weak_map_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.weak_map_prototype(),
            object_prototype,
        ),
        weak_set_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.weak_set_prototype(),
            object_prototype,
        ),
        weak_ref_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.weak_ref_prototype(),
            object_prototype,
        ),
        finalization_registry_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.finalization_registry_prototype(),
            object_prototype,
        ),
    }
}

fn allocate_binary_data_prototypes(
    agent: &mut Agent,
    request: ScaffoldingRequest<'_>,
    object_prototype: ObjectRef,
) -> BinaryDataFamilyPrototypes {
    let typed_array_prototype = ordinary_intrinsic(
        agent,
        request,
        request.intrinsics.typed_array_prototype(),
        object_prototype,
    );
    BinaryDataFamilyPrototypes {
        array_buffer_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.array_buffer_prototype(),
            object_prototype,
        ),
        shared_array_buffer_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.shared_array_buffer_prototype(),
            object_prototype,
        ),
        data_view_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.data_view_prototype(),
            object_prototype,
        ),
        typed_array_prototype,
        int8_array_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.int8_array_prototype(),
            typed_array_prototype,
        ),
        int16_array_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.int16_array_prototype(),
            typed_array_prototype,
        ),
        int32_array_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.int32_array_prototype(),
            typed_array_prototype,
        ),
        float32_array_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.float32_array_prototype(),
            typed_array_prototype,
        ),
        float64_array_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.float64_array_prototype(),
            typed_array_prototype,
        ),
        big_int64_array_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.big_int64_array_prototype(),
            typed_array_prototype,
        ),
        big_uint64_array_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.big_uint64_array_prototype(),
            typed_array_prototype,
        ),
        uint32_array_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.uint32_array_prototype(),
            typed_array_prototype,
        ),
        uint16_array_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.uint16_array_prototype(),
            typed_array_prototype,
        ),
        uint8_clamped_array_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.uint8_clamped_array_prototype(),
            typed_array_prototype,
        ),
        uint8_array_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.uint8_array_prototype(),
            typed_array_prototype,
        ),
    }
}

fn allocate_iterator_prototypes(
    agent: &mut Agent,
    request: ScaffoldingRequest<'_>,
    object_prototype: ObjectRef,
) -> IteratorScaffolding {
    let iterator_prototype = ordinary_intrinsic(
        agent,
        request,
        request.intrinsics.iterator_prototype(),
        object_prototype,
    );
    let async_iterator_prototype = ordinary_intrinsic(
        agent,
        request,
        request.intrinsics.async_iterator_prototype(),
        object_prototype,
    );
    IteratorScaffolding {
        iterator_prototype,
        async_from_sync_iterator_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.async_from_sync_iterator_prototype(),
            async_iterator_prototype,
        ),
        array_iterator_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.array_iterator_prototype(),
            iterator_prototype,
        ),
        map_iterator_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.map_iterator_prototype(),
            iterator_prototype,
        ),
        set_iterator_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.set_iterator_prototype(),
            iterator_prototype,
        ),
        string_iterator_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.string_iterator_prototype(),
            iterator_prototype,
        ),
        family: IteratorFamilyPrototypes {
            async_iterator_prototype,
            iterator_prototype,
        },
    }
}

fn allocate_primitive_prototypes(
    agent: &mut Agent,
    request: ScaffoldingRequest<'_>,
    object_prototype: ObjectRef,
) -> PrimitiveFamilyPrototypes {
    let boolean_prototype = request
        .intrinsics
        .boolean_prototype()
        .filter(|object| {
            agent.objects().primitive_wrapper_kind(*object) == Some(PrimitiveWrapperKind::Boolean)
        })
        .unwrap_or_else(|| {
            allocate_builtin_primitive_wrapper_object(
                agent,
                request.root_shape,
                Some(object_prototype),
                PrimitiveWrapperKind::Boolean,
                Value::from_bool(false),
            )
        });
    let symbol_prototype = request
        .intrinsics
        .symbol_prototype()
        .filter(|object| {
            agent.objects().primitive_wrapper_kind(*object) == Some(PrimitiveWrapperKind::Symbol)
        })
        .unwrap_or_else(|| {
            let symbol = agent.heap_mut().mutator().alloc_symbol(
                None,
                SymbolFlags::ordinary(),
                AllocationLifetime::Default,
            );
            allocate_builtin_primitive_wrapper_object(
                agent,
                request.root_shape,
                Some(object_prototype),
                PrimitiveWrapperKind::Symbol,
                Value::from_symbol_ref(symbol),
            )
        });
    reparent_builtin_object(agent, boolean_prototype, Some(object_prototype));
    reparent_builtin_object(agent, symbol_prototype, Some(object_prototype));
    let number_prototype = request.internal.number_prototype();
    let bigint_prototype = request.internal.bigint_prototype();
    reparent_builtin_object(agent, number_prototype, Some(object_prototype));
    reparent_builtin_object(agent, bigint_prototype, Some(object_prototype));
    PrimitiveFamilyPrototypes {
        number_prototype,
        bigint_prototype,
        boolean_prototype,
        symbol_prototype,
    }
}

fn allocate_array_prototypes(
    agent: &mut Agent,
    request: ScaffoldingRequest<'_>,
    object_prototype: ObjectRef,
) -> ArrayFamilyPrototypes {
    let array_prototype = request.internal.array_prototype();
    reparent_builtin_object(agent, array_prototype, Some(object_prototype));
    ArrayFamilyPrototypes {
        array_prototype,
        array_unscopables: allocate_builtin_ordinary_object(agent, request.root_shape, None),
    }
}

fn allocate_text_prototypes(
    agent: &mut Agent,
    request: ScaffoldingRequest<'_>,
    object_prototype: ObjectRef,
) -> TextPrototypes {
    let string_prototype = request.internal.string_prototype();
    reparent_builtin_object(agent, string_prototype, Some(object_prototype));
    TextPrototypes {
        string: string_prototype,
        regexp: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.regexp_prototype(),
            object_prototype,
        ),
        date: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.date_prototype(),
            object_prototype,
        ),
    }
}

#[derive(Clone, Copy)]
struct TextPrototypes {
    string: ObjectRef,
    regexp: ObjectRef,
    date: ObjectRef,
}

fn allocate_namespace_objects(
    agent: &mut Agent,
    request: ScaffoldingRequest<'_>,
    object_prototype: ObjectRef,
) -> NamespaceObjects {
    NamespaceObjects {
        math: allocate_builtin_ordinary_object(agent, request.root_shape, Some(object_prototype)),
        json: allocate_builtin_ordinary_object(agent, request.root_shape, Some(object_prototype)),
        reflect: allocate_builtin_ordinary_object(
            agent,
            request.root_shape,
            Some(object_prototype),
        ),
        abstract_module_source_prototype: allocate_builtin_ordinary_object(
            agent,
            request.root_shape,
            Some(object_prototype),
        ),
    }
}

fn allocate_error_prototypes(
    agent: &mut Agent,
    request: ScaffoldingRequest<'_>,
    object_prototype: ObjectRef,
) -> ErrorFamilyPrototypes {
    let error_prototype =
        allocate_builtin_ordinary_object(agent, request.root_shape, Some(object_prototype));
    ErrorFamilyPrototypes {
        error_prototype,
        eval_error_prototype: allocate_builtin_ordinary_object(
            agent,
            request.root_shape,
            Some(error_prototype),
        ),
        range_error_prototype: allocate_builtin_ordinary_object(
            agent,
            request.root_shape,
            Some(error_prototype),
        ),
        reference_error_prototype: allocate_builtin_ordinary_object(
            agent,
            request.root_shape,
            Some(error_prototype),
        ),
        syntax_error_prototype: allocate_builtin_ordinary_object(
            agent,
            request.root_shape,
            Some(error_prototype),
        ),
        type_error_prototype: allocate_builtin_ordinary_object(
            agent,
            request.root_shape,
            Some(error_prototype),
        ),
        uri_error_prototype: allocate_builtin_ordinary_object(
            agent,
            request.root_shape,
            Some(error_prototype),
        ),
        aggregate_error_prototype: allocate_builtin_ordinary_object(
            agent,
            request.root_shape,
            Some(error_prototype),
        ),
        suppressed_error_prototype: allocate_builtin_ordinary_object(
            agent,
            request.root_shape,
            Some(error_prototype),
        ),
    }
}

fn allocate_promise_disposal_prototypes(
    agent: &mut Agent,
    request: ScaffoldingRequest<'_>,
    object_prototype: ObjectRef,
) -> PromiseDisposalFamilyPrototypes {
    PromiseDisposalFamilyPrototypes {
        promise_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.promise_prototype(),
            object_prototype,
        ),
        disposable_stack_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.disposable_stack_prototype(),
            object_prototype,
        ),
        async_disposable_stack_prototype: ordinary_intrinsic(
            agent,
            request,
            request.intrinsics.async_disposable_stack_prototype(),
            object_prototype,
        ),
    }
}

fn public_intrinsic_handles(
    collection: CollectionFamilyPrototypes,
    binary_data: BinaryDataFamilyPrototypes,
    iterators: IteratorScaffolding,
    array_prototype: ObjectRef,
) -> PublicRealmPrototypeHandles {
    PublicRealmPrototypeHandles {
        array_prototype,
        map_prototype: collection.map_prototype,
        map_iterator_prototype: iterators.map_iterator_prototype,
        set_prototype: collection.set_prototype,
        set_iterator_prototype: iterators.set_iterator_prototype,
        weak_map_prototype: collection.weak_map_prototype,
        weak_set_prototype: collection.weak_set_prototype,
        weak_ref_prototype: collection.weak_ref_prototype,
        finalization_registry_prototype: collection.finalization_registry_prototype,
        array_buffer_prototype: binary_data.array_buffer_prototype,
        shared_array_buffer_prototype: binary_data.shared_array_buffer_prototype,
        data_view_prototype: binary_data.data_view_prototype,
        typed_array_prototype: binary_data.typed_array_prototype,
        int8_array_prototype: binary_data.int8_array_prototype,
        int16_array_prototype: binary_data.int16_array_prototype,
        int32_array_prototype: binary_data.int32_array_prototype,
        float32_array_prototype: binary_data.float32_array_prototype,
        float64_array_prototype: binary_data.float64_array_prototype,
        big_int64_array_prototype: binary_data.big_int64_array_prototype,
        big_uint64_array_prototype: binary_data.big_uint64_array_prototype,
        uint32_array_prototype: binary_data.uint32_array_prototype,
        uint16_array_prototype: binary_data.uint16_array_prototype,
        uint8_clamped_array_prototype: binary_data.uint8_clamped_array_prototype,
        uint8_array_prototype: binary_data.uint8_array_prototype,
        iterator_prototype: iterators.iterator_prototype,
        async_from_sync_iterator_prototype: iterators.async_from_sync_iterator_prototype,
        array_iterator_prototype: iterators.array_iterator_prototype,
        string_iterator_prototype: iterators.string_iterator_prototype,
    }
}

fn ordinary_intrinsic(
    agent: &mut Agent,
    request: ScaffoldingRequest<'_>,
    existing: Option<ObjectRef>,
    prototype: ObjectRef,
) -> ObjectRef {
    let object = existing.unwrap_or_else(|| {
        allocate_builtin_ordinary_object(agent, request.root_shape, Some(prototype))
    });
    reparent_builtin_object(agent, object, Some(prototype));
    object
}
