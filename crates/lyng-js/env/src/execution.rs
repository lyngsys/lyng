use lyng_js_common::AtomId;
use lyng_js_gc::{PrimitiveTracer, TraceHeapEdges};
use lyng_js_types::{CodeRef, EnvironmentRef, ObjectRef, RealmRef, ShapeId, Value};

/// Execution identity categories reserved by the Phase 3 runtime substrate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutableId {
    Script,
    Module,
    Builtin,
    Bytecode(CodeRef),
}

/// Typed placeholder table for realm-owned intrinsics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Intrinsics {
    object: Option<ObjectRef>,
    object_prototype: Option<ObjectRef>,
    function: Option<ObjectRef>,
    function_prototype: Option<ObjectRef>,
    async_function: Option<ObjectRef>,
    async_function_prototype: Option<ObjectRef>,
    async_generator_function: Option<ObjectRef>,
    async_generator_function_prototype: Option<ObjectRef>,
    async_generator_prototype: Option<ObjectRef>,
    generator_function: Option<ObjectRef>,
    generator_function_prototype: Option<ObjectRef>,
    generator_prototype: Option<ObjectRef>,
    array: Option<ObjectRef>,
    array_prototype: Option<ObjectRef>,
    map: Option<ObjectRef>,
    map_prototype: Option<ObjectRef>,
    map_iterator_prototype: Option<ObjectRef>,
    set: Option<ObjectRef>,
    set_prototype: Option<ObjectRef>,
    set_iterator_prototype: Option<ObjectRef>,
    weak_map: Option<ObjectRef>,
    weak_map_prototype: Option<ObjectRef>,
    weak_set: Option<ObjectRef>,
    weak_set_prototype: Option<ObjectRef>,
    weak_ref: Option<ObjectRef>,
    weak_ref_prototype: Option<ObjectRef>,
    finalization_registry: Option<ObjectRef>,
    finalization_registry_prototype: Option<ObjectRef>,
    array_buffer: Option<ObjectRef>,
    array_buffer_prototype: Option<ObjectRef>,
    shared_array_buffer: Option<ObjectRef>,
    shared_array_buffer_prototype: Option<ObjectRef>,
    data_view: Option<ObjectRef>,
    data_view_prototype: Option<ObjectRef>,
    atomics: Option<ObjectRef>,
    typed_array: Option<ObjectRef>,
    typed_array_prototype: Option<ObjectRef>,
    int8_array: Option<ObjectRef>,
    int8_array_prototype: Option<ObjectRef>,
    int16_array: Option<ObjectRef>,
    int16_array_prototype: Option<ObjectRef>,
    int32_array: Option<ObjectRef>,
    int32_array_prototype: Option<ObjectRef>,
    float32_array: Option<ObjectRef>,
    float32_array_prototype: Option<ObjectRef>,
    float64_array: Option<ObjectRef>,
    float64_array_prototype: Option<ObjectRef>,
    big_int64_array: Option<ObjectRef>,
    big_int64_array_prototype: Option<ObjectRef>,
    big_uint64_array: Option<ObjectRef>,
    big_uint64_array_prototype: Option<ObjectRef>,
    uint32_array: Option<ObjectRef>,
    uint32_array_prototype: Option<ObjectRef>,
    uint16_array: Option<ObjectRef>,
    uint16_array_prototype: Option<ObjectRef>,
    uint8_clamped_array: Option<ObjectRef>,
    uint8_clamped_array_prototype: Option<ObjectRef>,
    uint8_array: Option<ObjectRef>,
    uint8_array_prototype: Option<ObjectRef>,
    iterator_prototype: Option<ObjectRef>,
    async_iterator_prototype: Option<ObjectRef>,
    async_from_sync_iterator_prototype: Option<ObjectRef>,
    array_iterator_prototype: Option<ObjectRef>,
    string: Option<ObjectRef>,
    string_prototype: Option<ObjectRef>,
    string_iterator_prototype: Option<ObjectRef>,
    regexp: Option<ObjectRef>,
    regexp_prototype: Option<ObjectRef>,
    date: Option<ObjectRef>,
    date_prototype: Option<ObjectRef>,
    number: Option<ObjectRef>,
    number_prototype: Option<ObjectRef>,
    math: Option<ObjectRef>,
    bigint: Option<ObjectRef>,
    bigint_prototype: Option<ObjectRef>,
    boolean: Option<ObjectRef>,
    boolean_prototype: Option<ObjectRef>,
    symbol: Option<ObjectRef>,
    symbol_prototype: Option<ObjectRef>,
    json: Option<ObjectRef>,
    reflect: Option<ObjectRef>,
    proxy: Option<ObjectRef>,
    error: Option<ObjectRef>,
    error_prototype: Option<ObjectRef>,
    eval_error: Option<ObjectRef>,
    eval_error_prototype: Option<ObjectRef>,
    range_error: Option<ObjectRef>,
    range_error_prototype: Option<ObjectRef>,
    reference_error: Option<ObjectRef>,
    reference_error_prototype: Option<ObjectRef>,
    syntax_error: Option<ObjectRef>,
    syntax_error_prototype: Option<ObjectRef>,
    type_error: Option<ObjectRef>,
    type_error_prototype: Option<ObjectRef>,
    uri_error: Option<ObjectRef>,
    uri_error_prototype: Option<ObjectRef>,
    aggregate_error: Option<ObjectRef>,
    aggregate_error_prototype: Option<ObjectRef>,
    suppressed_error: Option<ObjectRef>,
    suppressed_error_prototype: Option<ObjectRef>,
    promise: Option<ObjectRef>,
    promise_prototype: Option<ObjectRef>,
    disposable_stack: Option<ObjectRef>,
    disposable_stack_prototype: Option<ObjectRef>,
    async_disposable_stack: Option<ObjectRef>,
    async_disposable_stack_prototype: Option<ObjectRef>,
    throw_type_error: Option<ObjectRef>,
}

impl Intrinsics {
    #[inline]
    pub const fn new() -> Self {
        Self {
            object: None,
            object_prototype: None,
            function: None,
            function_prototype: None,
            async_function: None,
            async_function_prototype: None,
            async_generator_function: None,
            async_generator_function_prototype: None,
            async_generator_prototype: None,
            generator_function: None,
            generator_function_prototype: None,
            generator_prototype: None,
            array: None,
            array_prototype: None,
            map: None,
            map_prototype: None,
            map_iterator_prototype: None,
            set: None,
            set_prototype: None,
            set_iterator_prototype: None,
            weak_map: None,
            weak_map_prototype: None,
            weak_set: None,
            weak_set_prototype: None,
            weak_ref: None,
            weak_ref_prototype: None,
            finalization_registry: None,
            finalization_registry_prototype: None,
            array_buffer: None,
            array_buffer_prototype: None,
            shared_array_buffer: None,
            shared_array_buffer_prototype: None,
            data_view: None,
            data_view_prototype: None,
            atomics: None,
            typed_array: None,
            typed_array_prototype: None,
            int8_array: None,
            int8_array_prototype: None,
            int16_array: None,
            int16_array_prototype: None,
            int32_array: None,
            int32_array_prototype: None,
            float32_array: None,
            float32_array_prototype: None,
            float64_array: None,
            float64_array_prototype: None,
            big_int64_array: None,
            big_int64_array_prototype: None,
            big_uint64_array: None,
            big_uint64_array_prototype: None,
            uint32_array: None,
            uint32_array_prototype: None,
            uint16_array: None,
            uint16_array_prototype: None,
            uint8_clamped_array: None,
            uint8_clamped_array_prototype: None,
            uint8_array: None,
            uint8_array_prototype: None,
            iterator_prototype: None,
            async_iterator_prototype: None,
            async_from_sync_iterator_prototype: None,
            array_iterator_prototype: None,
            string: None,
            string_prototype: None,
            string_iterator_prototype: None,
            regexp: None,
            regexp_prototype: None,
            date: None,
            date_prototype: None,
            number: None,
            number_prototype: None,
            math: None,
            bigint: None,
            bigint_prototype: None,
            boolean: None,
            boolean_prototype: None,
            symbol: None,
            symbol_prototype: None,
            json: None,
            reflect: None,
            proxy: None,
            error: None,
            error_prototype: None,
            eval_error: None,
            eval_error_prototype: None,
            range_error: None,
            range_error_prototype: None,
            reference_error: None,
            reference_error_prototype: None,
            syntax_error: None,
            syntax_error_prototype: None,
            type_error: None,
            type_error_prototype: None,
            uri_error: None,
            uri_error_prototype: None,
            aggregate_error: None,
            aggregate_error_prototype: None,
            suppressed_error: None,
            suppressed_error_prototype: None,
            promise: None,
            promise_prototype: None,
            disposable_stack: None,
            disposable_stack_prototype: None,
            async_disposable_stack: None,
            async_disposable_stack_prototype: None,
            throw_type_error: None,
        }
    }

    #[inline]
    pub const fn object(self) -> Option<ObjectRef> {
        self.object
    }

    #[inline]
    pub const fn object_prototype(self) -> Option<ObjectRef> {
        self.object_prototype
    }

    #[inline]
    pub const fn function(self) -> Option<ObjectRef> {
        self.function
    }

    #[inline]
    pub const fn function_prototype(self) -> Option<ObjectRef> {
        self.function_prototype
    }

    #[inline]
    pub const fn async_function(self) -> Option<ObjectRef> {
        self.async_function
    }

    #[inline]
    pub const fn async_function_prototype(self) -> Option<ObjectRef> {
        self.async_function_prototype
    }

    #[inline]
    pub const fn async_generator_function(self) -> Option<ObjectRef> {
        self.async_generator_function
    }

    #[inline]
    pub const fn async_generator_function_prototype(self) -> Option<ObjectRef> {
        self.async_generator_function_prototype
    }

    #[inline]
    pub const fn async_generator_prototype(self) -> Option<ObjectRef> {
        self.async_generator_prototype
    }

    #[inline]
    pub const fn generator_function(self) -> Option<ObjectRef> {
        self.generator_function
    }

    #[inline]
    pub const fn generator_function_prototype(self) -> Option<ObjectRef> {
        self.generator_function_prototype
    }

    #[inline]
    pub const fn generator_prototype(self) -> Option<ObjectRef> {
        self.generator_prototype
    }

    #[inline]
    pub const fn array(self) -> Option<ObjectRef> {
        self.array
    }

    #[inline]
    pub const fn array_prototype(self) -> Option<ObjectRef> {
        self.array_prototype
    }

    #[inline]
    pub const fn map(self) -> Option<ObjectRef> {
        self.map
    }

    #[inline]
    pub const fn map_prototype(self) -> Option<ObjectRef> {
        self.map_prototype
    }

    #[inline]
    pub const fn map_iterator_prototype(self) -> Option<ObjectRef> {
        self.map_iterator_prototype
    }

    #[inline]
    pub const fn set(self) -> Option<ObjectRef> {
        self.set
    }

    #[inline]
    pub const fn set_prototype(self) -> Option<ObjectRef> {
        self.set_prototype
    }

    #[inline]
    pub const fn set_iterator_prototype(self) -> Option<ObjectRef> {
        self.set_iterator_prototype
    }

    #[inline]
    pub const fn weak_map(self) -> Option<ObjectRef> {
        self.weak_map
    }

    #[inline]
    pub const fn weak_map_prototype(self) -> Option<ObjectRef> {
        self.weak_map_prototype
    }

    #[inline]
    pub const fn weak_set(self) -> Option<ObjectRef> {
        self.weak_set
    }

    #[inline]
    pub const fn weak_set_prototype(self) -> Option<ObjectRef> {
        self.weak_set_prototype
    }

    #[inline]
    pub const fn weak_ref(self) -> Option<ObjectRef> {
        self.weak_ref
    }

    #[inline]
    pub const fn weak_ref_prototype(self) -> Option<ObjectRef> {
        self.weak_ref_prototype
    }

    #[inline]
    pub const fn finalization_registry(self) -> Option<ObjectRef> {
        self.finalization_registry
    }

    #[inline]
    pub const fn finalization_registry_prototype(self) -> Option<ObjectRef> {
        self.finalization_registry_prototype
    }

    #[inline]
    pub const fn array_buffer(self) -> Option<ObjectRef> {
        self.array_buffer
    }

    #[inline]
    pub const fn array_buffer_prototype(self) -> Option<ObjectRef> {
        self.array_buffer_prototype
    }

    #[inline]
    pub const fn shared_array_buffer(self) -> Option<ObjectRef> {
        self.shared_array_buffer
    }

    #[inline]
    pub const fn shared_array_buffer_prototype(self) -> Option<ObjectRef> {
        self.shared_array_buffer_prototype
    }

    #[inline]
    pub const fn data_view(self) -> Option<ObjectRef> {
        self.data_view
    }

    #[inline]
    pub const fn data_view_prototype(self) -> Option<ObjectRef> {
        self.data_view_prototype
    }

    #[inline]
    pub const fn atomics(self) -> Option<ObjectRef> {
        self.atomics
    }

    #[inline]
    pub const fn typed_array(self) -> Option<ObjectRef> {
        self.typed_array
    }

    #[inline]
    pub const fn typed_array_prototype(self) -> Option<ObjectRef> {
        self.typed_array_prototype
    }

    #[inline]
    pub const fn int8_array(self) -> Option<ObjectRef> {
        self.int8_array
    }

    #[inline]
    pub const fn int8_array_prototype(self) -> Option<ObjectRef> {
        self.int8_array_prototype
    }

    #[inline]
    pub const fn int16_array(self) -> Option<ObjectRef> {
        self.int16_array
    }

    #[inline]
    pub const fn int16_array_prototype(self) -> Option<ObjectRef> {
        self.int16_array_prototype
    }

    #[inline]
    pub const fn int32_array(self) -> Option<ObjectRef> {
        self.int32_array
    }

    #[inline]
    pub const fn int32_array_prototype(self) -> Option<ObjectRef> {
        self.int32_array_prototype
    }

    #[inline]
    pub const fn float32_array(self) -> Option<ObjectRef> {
        self.float32_array
    }

    #[inline]
    pub const fn float32_array_prototype(self) -> Option<ObjectRef> {
        self.float32_array_prototype
    }

    #[inline]
    pub const fn float64_array(self) -> Option<ObjectRef> {
        self.float64_array
    }

    #[inline]
    pub const fn float64_array_prototype(self) -> Option<ObjectRef> {
        self.float64_array_prototype
    }

    #[inline]
    pub const fn big_int64_array(self) -> Option<ObjectRef> {
        self.big_int64_array
    }

    #[inline]
    pub const fn big_int64_array_prototype(self) -> Option<ObjectRef> {
        self.big_int64_array_prototype
    }

    #[inline]
    pub const fn big_uint64_array(self) -> Option<ObjectRef> {
        self.big_uint64_array
    }

    #[inline]
    pub const fn big_uint64_array_prototype(self) -> Option<ObjectRef> {
        self.big_uint64_array_prototype
    }

    #[inline]
    pub const fn uint32_array(self) -> Option<ObjectRef> {
        self.uint32_array
    }

    #[inline]
    pub const fn uint32_array_prototype(self) -> Option<ObjectRef> {
        self.uint32_array_prototype
    }

    #[inline]
    pub const fn uint16_array(self) -> Option<ObjectRef> {
        self.uint16_array
    }

    #[inline]
    pub const fn uint16_array_prototype(self) -> Option<ObjectRef> {
        self.uint16_array_prototype
    }

    #[inline]
    pub const fn uint8_clamped_array(self) -> Option<ObjectRef> {
        self.uint8_clamped_array
    }

    #[inline]
    pub const fn uint8_clamped_array_prototype(self) -> Option<ObjectRef> {
        self.uint8_clamped_array_prototype
    }

    #[inline]
    pub const fn uint8_array(self) -> Option<ObjectRef> {
        self.uint8_array
    }

    #[inline]
    pub const fn uint8_array_prototype(self) -> Option<ObjectRef> {
        self.uint8_array_prototype
    }

    #[inline]
    pub const fn iterator_prototype(self) -> Option<ObjectRef> {
        self.iterator_prototype
    }

    #[inline]
    pub const fn async_iterator_prototype(self) -> Option<ObjectRef> {
        self.async_iterator_prototype
    }

    #[inline]
    pub const fn async_from_sync_iterator_prototype(self) -> Option<ObjectRef> {
        self.async_from_sync_iterator_prototype
    }

    #[inline]
    pub const fn array_iterator_prototype(self) -> Option<ObjectRef> {
        self.array_iterator_prototype
    }

    #[inline]
    pub const fn string(self) -> Option<ObjectRef> {
        self.string
    }

    #[inline]
    pub const fn string_prototype(self) -> Option<ObjectRef> {
        self.string_prototype
    }

    #[inline]
    pub const fn string_iterator_prototype(self) -> Option<ObjectRef> {
        self.string_iterator_prototype
    }

    #[inline]
    pub const fn regexp(self) -> Option<ObjectRef> {
        self.regexp
    }

    #[inline]
    pub const fn regexp_prototype(self) -> Option<ObjectRef> {
        self.regexp_prototype
    }

    #[inline]
    pub const fn date(self) -> Option<ObjectRef> {
        self.date
    }

    #[inline]
    pub const fn date_prototype(self) -> Option<ObjectRef> {
        self.date_prototype
    }

    #[inline]
    pub const fn number(self) -> Option<ObjectRef> {
        self.number
    }

    #[inline]
    pub const fn number_prototype(self) -> Option<ObjectRef> {
        self.number_prototype
    }

    #[inline]
    pub const fn math(self) -> Option<ObjectRef> {
        self.math
    }

    #[inline]
    pub const fn bigint(self) -> Option<ObjectRef> {
        self.bigint
    }

    #[inline]
    pub const fn bigint_prototype(self) -> Option<ObjectRef> {
        self.bigint_prototype
    }

    #[inline]
    pub const fn boolean(self) -> Option<ObjectRef> {
        self.boolean
    }

    #[inline]
    pub const fn boolean_prototype(self) -> Option<ObjectRef> {
        self.boolean_prototype
    }

    #[inline]
    pub const fn symbol(self) -> Option<ObjectRef> {
        self.symbol
    }

    #[inline]
    pub const fn symbol_prototype(self) -> Option<ObjectRef> {
        self.symbol_prototype
    }

    #[inline]
    pub const fn json(self) -> Option<ObjectRef> {
        self.json
    }

    #[inline]
    pub const fn reflect(self) -> Option<ObjectRef> {
        self.reflect
    }

    #[inline]
    pub const fn proxy(self) -> Option<ObjectRef> {
        self.proxy
    }

    #[inline]
    pub const fn error(self) -> Option<ObjectRef> {
        self.error
    }

    #[inline]
    pub const fn error_prototype(self) -> Option<ObjectRef> {
        self.error_prototype
    }

    #[inline]
    pub const fn eval_error(self) -> Option<ObjectRef> {
        self.eval_error
    }

    #[inline]
    pub const fn eval_error_prototype(self) -> Option<ObjectRef> {
        self.eval_error_prototype
    }

    #[inline]
    pub const fn range_error(self) -> Option<ObjectRef> {
        self.range_error
    }

    #[inline]
    pub const fn range_error_prototype(self) -> Option<ObjectRef> {
        self.range_error_prototype
    }

    #[inline]
    pub const fn reference_error(self) -> Option<ObjectRef> {
        self.reference_error
    }

    #[inline]
    pub const fn reference_error_prototype(self) -> Option<ObjectRef> {
        self.reference_error_prototype
    }

    #[inline]
    pub const fn syntax_error(self) -> Option<ObjectRef> {
        self.syntax_error
    }

    #[inline]
    pub const fn syntax_error_prototype(self) -> Option<ObjectRef> {
        self.syntax_error_prototype
    }

    #[inline]
    pub const fn type_error(self) -> Option<ObjectRef> {
        self.type_error
    }

    #[inline]
    pub const fn type_error_prototype(self) -> Option<ObjectRef> {
        self.type_error_prototype
    }

    #[inline]
    pub const fn uri_error(self) -> Option<ObjectRef> {
        self.uri_error
    }

    #[inline]
    pub const fn uri_error_prototype(self) -> Option<ObjectRef> {
        self.uri_error_prototype
    }

    #[inline]
    pub const fn aggregate_error(self) -> Option<ObjectRef> {
        self.aggregate_error
    }

    #[inline]
    pub const fn aggregate_error_prototype(self) -> Option<ObjectRef> {
        self.aggregate_error_prototype
    }

    #[inline]
    pub const fn suppressed_error(self) -> Option<ObjectRef> {
        self.suppressed_error
    }

    #[inline]
    pub const fn suppressed_error_prototype(self) -> Option<ObjectRef> {
        self.suppressed_error_prototype
    }

    #[inline]
    pub const fn promise(self) -> Option<ObjectRef> {
        self.promise
    }

    #[inline]
    pub const fn promise_prototype(self) -> Option<ObjectRef> {
        self.promise_prototype
    }

    #[inline]
    pub const fn disposable_stack(self) -> Option<ObjectRef> {
        self.disposable_stack
    }

    #[inline]
    pub const fn disposable_stack_prototype(self) -> Option<ObjectRef> {
        self.disposable_stack_prototype
    }

    #[inline]
    pub const fn async_disposable_stack(self) -> Option<ObjectRef> {
        self.async_disposable_stack
    }

    #[inline]
    pub const fn async_disposable_stack_prototype(self) -> Option<ObjectRef> {
        self.async_disposable_stack_prototype
    }

    #[inline]
    pub const fn throw_type_error(self) -> Option<ObjectRef> {
        self.throw_type_error
    }

    #[inline]
    pub const fn with_object(mut self, value: Option<ObjectRef>) -> Self {
        self.object = value;
        self
    }

    #[inline]
    pub const fn with_object_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.object_prototype = value;
        self
    }

    #[inline]
    pub const fn with_function(mut self, value: Option<ObjectRef>) -> Self {
        self.function = value;
        self
    }

    #[inline]
    pub const fn with_function_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.function_prototype = value;
        self
    }

    #[inline]
    pub const fn with_async_function(mut self, value: Option<ObjectRef>) -> Self {
        self.async_function = value;
        self
    }

    #[inline]
    pub const fn with_async_function_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.async_function_prototype = value;
        self
    }

    #[inline]
    pub const fn with_async_generator_function(mut self, value: Option<ObjectRef>) -> Self {
        self.async_generator_function = value;
        self
    }

    #[inline]
    pub const fn with_async_generator_function_prototype(
        mut self,
        value: Option<ObjectRef>,
    ) -> Self {
        self.async_generator_function_prototype = value;
        self
    }

    #[inline]
    pub const fn with_async_generator_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.async_generator_prototype = value;
        self
    }

    #[inline]
    pub const fn with_generator_function(mut self, value: Option<ObjectRef>) -> Self {
        self.generator_function = value;
        self
    }

    #[inline]
    pub const fn with_generator_function_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.generator_function_prototype = value;
        self
    }

    #[inline]
    pub const fn with_generator_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.generator_prototype = value;
        self
    }

    #[inline]
    pub const fn with_array(mut self, value: Option<ObjectRef>) -> Self {
        self.array = value;
        self
    }

    #[inline]
    pub const fn with_array_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.array_prototype = value;
        self
    }

    #[inline]
    pub const fn with_map(mut self, value: Option<ObjectRef>) -> Self {
        self.map = value;
        self
    }

    #[inline]
    pub const fn with_map_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.map_prototype = value;
        self
    }

    #[inline]
    pub const fn with_map_iterator_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.map_iterator_prototype = value;
        self
    }

    #[inline]
    pub const fn with_set(mut self, value: Option<ObjectRef>) -> Self {
        self.set = value;
        self
    }

    #[inline]
    pub const fn with_set_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.set_prototype = value;
        self
    }

    #[inline]
    pub const fn with_set_iterator_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.set_iterator_prototype = value;
        self
    }

    #[inline]
    pub const fn with_weak_map(mut self, value: Option<ObjectRef>) -> Self {
        self.weak_map = value;
        self
    }

    #[inline]
    pub const fn with_weak_map_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.weak_map_prototype = value;
        self
    }

    #[inline]
    pub const fn with_weak_set(mut self, value: Option<ObjectRef>) -> Self {
        self.weak_set = value;
        self
    }

    #[inline]
    pub const fn with_weak_set_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.weak_set_prototype = value;
        self
    }

    #[inline]
    pub const fn with_weak_ref(mut self, value: Option<ObjectRef>) -> Self {
        self.weak_ref = value;
        self
    }

    #[inline]
    pub const fn with_weak_ref_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.weak_ref_prototype = value;
        self
    }

    #[inline]
    pub const fn with_finalization_registry(mut self, value: Option<ObjectRef>) -> Self {
        self.finalization_registry = value;
        self
    }

    #[inline]
    pub const fn with_finalization_registry_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.finalization_registry_prototype = value;
        self
    }

    #[inline]
    pub const fn with_array_buffer(mut self, value: Option<ObjectRef>) -> Self {
        self.array_buffer = value;
        self
    }

    #[inline]
    pub const fn with_array_buffer_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.array_buffer_prototype = value;
        self
    }

    #[inline]
    pub const fn with_shared_array_buffer(mut self, value: Option<ObjectRef>) -> Self {
        self.shared_array_buffer = value;
        self
    }

    #[inline]
    pub const fn with_shared_array_buffer_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.shared_array_buffer_prototype = value;
        self
    }

    #[inline]
    pub const fn with_data_view(mut self, value: Option<ObjectRef>) -> Self {
        self.data_view = value;
        self
    }

    #[inline]
    pub const fn with_data_view_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.data_view_prototype = value;
        self
    }

    #[inline]
    pub const fn with_atomics(mut self, value: Option<ObjectRef>) -> Self {
        self.atomics = value;
        self
    }

    #[inline]
    pub const fn with_typed_array(mut self, value: Option<ObjectRef>) -> Self {
        self.typed_array = value;
        self
    }

    #[inline]
    pub const fn with_typed_array_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.typed_array_prototype = value;
        self
    }

    #[inline]
    pub const fn with_int8_array(mut self, value: Option<ObjectRef>) -> Self {
        self.int8_array = value;
        self
    }

    #[inline]
    pub const fn with_int8_array_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.int8_array_prototype = value;
        self
    }

    #[inline]
    pub const fn with_int16_array(mut self, value: Option<ObjectRef>) -> Self {
        self.int16_array = value;
        self
    }

    #[inline]
    pub const fn with_int16_array_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.int16_array_prototype = value;
        self
    }

    #[inline]
    pub const fn with_int32_array(mut self, value: Option<ObjectRef>) -> Self {
        self.int32_array = value;
        self
    }

    #[inline]
    pub const fn with_int32_array_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.int32_array_prototype = value;
        self
    }

    #[inline]
    pub const fn with_float32_array(mut self, value: Option<ObjectRef>) -> Self {
        self.float32_array = value;
        self
    }

    #[inline]
    pub const fn with_float32_array_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.float32_array_prototype = value;
        self
    }

    #[inline]
    pub const fn with_float64_array(mut self, value: Option<ObjectRef>) -> Self {
        self.float64_array = value;
        self
    }

    #[inline]
    pub const fn with_float64_array_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.float64_array_prototype = value;
        self
    }

    #[inline]
    pub const fn with_big_int64_array(mut self, value: Option<ObjectRef>) -> Self {
        self.big_int64_array = value;
        self
    }

    #[inline]
    pub const fn with_big_int64_array_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.big_int64_array_prototype = value;
        self
    }

    #[inline]
    pub const fn with_big_uint64_array(mut self, value: Option<ObjectRef>) -> Self {
        self.big_uint64_array = value;
        self
    }

    #[inline]
    pub const fn with_big_uint64_array_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.big_uint64_array_prototype = value;
        self
    }

    #[inline]
    pub const fn with_uint32_array(mut self, value: Option<ObjectRef>) -> Self {
        self.uint32_array = value;
        self
    }

    #[inline]
    pub const fn with_uint32_array_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.uint32_array_prototype = value;
        self
    }

    #[inline]
    pub const fn with_uint16_array(mut self, value: Option<ObjectRef>) -> Self {
        self.uint16_array = value;
        self
    }

    #[inline]
    pub const fn with_uint16_array_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.uint16_array_prototype = value;
        self
    }

    #[inline]
    pub const fn with_uint8_clamped_array(mut self, value: Option<ObjectRef>) -> Self {
        self.uint8_clamped_array = value;
        self
    }

    #[inline]
    pub const fn with_uint8_clamped_array_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.uint8_clamped_array_prototype = value;
        self
    }

    #[inline]
    pub const fn with_uint8_array(mut self, value: Option<ObjectRef>) -> Self {
        self.uint8_array = value;
        self
    }

    #[inline]
    pub const fn with_uint8_array_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.uint8_array_prototype = value;
        self
    }

    #[inline]
    pub const fn with_iterator_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.iterator_prototype = value;
        self
    }

    #[inline]
    pub const fn with_async_iterator_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.async_iterator_prototype = value;
        self
    }

    #[inline]
    pub const fn with_async_from_sync_iterator_prototype(
        mut self,
        value: Option<ObjectRef>,
    ) -> Self {
        self.async_from_sync_iterator_prototype = value;
        self
    }

    #[inline]
    pub const fn with_array_iterator_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.array_iterator_prototype = value;
        self
    }

    #[inline]
    pub const fn with_string(mut self, value: Option<ObjectRef>) -> Self {
        self.string = value;
        self
    }

    #[inline]
    pub const fn with_string_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.string_prototype = value;
        self
    }

    #[inline]
    pub const fn with_string_iterator_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.string_iterator_prototype = value;
        self
    }

    #[inline]
    pub const fn with_regexp(mut self, value: Option<ObjectRef>) -> Self {
        self.regexp = value;
        self
    }

    #[inline]
    pub const fn with_regexp_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.regexp_prototype = value;
        self
    }

    #[inline]
    pub const fn with_date(mut self, value: Option<ObjectRef>) -> Self {
        self.date = value;
        self
    }

    #[inline]
    pub const fn with_date_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.date_prototype = value;
        self
    }

    #[inline]
    pub const fn with_number(mut self, value: Option<ObjectRef>) -> Self {
        self.number = value;
        self
    }

    #[inline]
    pub const fn with_number_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.number_prototype = value;
        self
    }

    #[inline]
    pub const fn with_math(mut self, value: Option<ObjectRef>) -> Self {
        self.math = value;
        self
    }

    #[inline]
    pub const fn with_bigint(mut self, value: Option<ObjectRef>) -> Self {
        self.bigint = value;
        self
    }

    #[inline]
    pub const fn with_bigint_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.bigint_prototype = value;
        self
    }

    #[inline]
    pub const fn with_boolean(mut self, value: Option<ObjectRef>) -> Self {
        self.boolean = value;
        self
    }

    #[inline]
    pub const fn with_boolean_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.boolean_prototype = value;
        self
    }

    #[inline]
    pub const fn with_symbol(mut self, value: Option<ObjectRef>) -> Self {
        self.symbol = value;
        self
    }

    #[inline]
    pub const fn with_symbol_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.symbol_prototype = value;
        self
    }

    #[inline]
    pub const fn with_json(mut self, value: Option<ObjectRef>) -> Self {
        self.json = value;
        self
    }

    #[inline]
    pub const fn with_reflect(mut self, value: Option<ObjectRef>) -> Self {
        self.reflect = value;
        self
    }

    #[inline]
    pub const fn with_proxy(mut self, value: Option<ObjectRef>) -> Self {
        self.proxy = value;
        self
    }

    #[inline]
    pub const fn with_error(mut self, value: Option<ObjectRef>) -> Self {
        self.error = value;
        self
    }

    #[inline]
    pub const fn with_error_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.error_prototype = value;
        self
    }

    #[inline]
    pub const fn with_eval_error(mut self, value: Option<ObjectRef>) -> Self {
        self.eval_error = value;
        self
    }

    #[inline]
    pub const fn with_eval_error_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.eval_error_prototype = value;
        self
    }

    #[inline]
    pub const fn with_range_error(mut self, value: Option<ObjectRef>) -> Self {
        self.range_error = value;
        self
    }

    #[inline]
    pub const fn with_range_error_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.range_error_prototype = value;
        self
    }

    #[inline]
    pub const fn with_reference_error(mut self, value: Option<ObjectRef>) -> Self {
        self.reference_error = value;
        self
    }

    #[inline]
    pub const fn with_reference_error_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.reference_error_prototype = value;
        self
    }

    #[inline]
    pub const fn with_syntax_error(mut self, value: Option<ObjectRef>) -> Self {
        self.syntax_error = value;
        self
    }

    #[inline]
    pub const fn with_syntax_error_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.syntax_error_prototype = value;
        self
    }

    #[inline]
    pub const fn with_type_error(mut self, value: Option<ObjectRef>) -> Self {
        self.type_error = value;
        self
    }

    #[inline]
    pub const fn with_type_error_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.type_error_prototype = value;
        self
    }

    #[inline]
    pub const fn with_uri_error(mut self, value: Option<ObjectRef>) -> Self {
        self.uri_error = value;
        self
    }

    #[inline]
    pub const fn with_uri_error_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.uri_error_prototype = value;
        self
    }

    #[inline]
    pub const fn with_aggregate_error(mut self, value: Option<ObjectRef>) -> Self {
        self.aggregate_error = value;
        self
    }

    #[inline]
    pub const fn with_aggregate_error_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.aggregate_error_prototype = value;
        self
    }

    #[inline]
    pub const fn with_suppressed_error(mut self, value: Option<ObjectRef>) -> Self {
        self.suppressed_error = value;
        self
    }

    #[inline]
    pub const fn with_suppressed_error_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.suppressed_error_prototype = value;
        self
    }

    #[inline]
    pub const fn with_promise(mut self, value: Option<ObjectRef>) -> Self {
        self.promise = value;
        self
    }

    #[inline]
    pub const fn with_promise_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.promise_prototype = value;
        self
    }

    #[inline]
    pub const fn with_disposable_stack(mut self, value: Option<ObjectRef>) -> Self {
        self.disposable_stack = value;
        self
    }

    #[inline]
    pub const fn with_disposable_stack_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.disposable_stack_prototype = value;
        self
    }

    #[inline]
    pub const fn with_async_disposable_stack(mut self, value: Option<ObjectRef>) -> Self {
        self.async_disposable_stack = value;
        self
    }

    #[inline]
    pub const fn with_async_disposable_stack_prototype(mut self, value: Option<ObjectRef>) -> Self {
        self.async_disposable_stack_prototype = value;
        self
    }

    #[inline]
    pub const fn with_throw_type_error(mut self, value: Option<ObjectRef>) -> Self {
        self.throw_type_error = value;
        self
    }
}

/// Stable execution-context classification frozen by Phase 3.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionContextKind {
    Script,
    Module,
    Builtin,
    Function,
    Eval,
    Job,
}

/// Current `this` state tracked by one execution context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThisState {
    Lexical,
    Uninitialized,
    Value(Value),
}

/// Cold execution-context record owned by `lyng_js_env`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExecutionContext {
    realm: RealmRef,
    executable: ExecutableId,
    lexical_env: EnvironmentRef,
    variable_env: EnvironmentRef,
    private_env: Option<EnvironmentRef>,
    script_or_module_referrer: Option<AtomId>,
    this_state: ThisState,
    new_target: Option<ObjectRef>,
    kind: ExecutionContextKind,
}

impl ExecutionContext {
    #[inline]
    pub const fn new(
        kind: ExecutionContextKind,
        realm: RealmRef,
        executable: ExecutableId,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) -> Self {
        Self {
            realm,
            executable,
            lexical_env,
            variable_env,
            private_env: None,
            script_or_module_referrer: None,
            this_state: ThisState::Uninitialized,
            new_target: None,
            kind,
        }
    }

    #[inline]
    pub const fn script(
        realm: RealmRef,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) -> Self {
        Self::new(
            ExecutionContextKind::Script,
            realm,
            ExecutableId::Script,
            lexical_env,
            variable_env,
        )
    }

    #[inline]
    pub const fn module(
        realm: RealmRef,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) -> Self {
        Self::new(
            ExecutionContextKind::Module,
            realm,
            ExecutableId::Module,
            lexical_env,
            variable_env,
        )
    }

    #[inline]
    pub const fn builtin(
        realm: RealmRef,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) -> Self {
        Self::new(
            ExecutionContextKind::Builtin,
            realm,
            ExecutableId::Builtin,
            lexical_env,
            variable_env,
        )
    }

    #[inline]
    pub const fn eval(
        realm: RealmRef,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) -> Self {
        Self::new(
            ExecutionContextKind::Eval,
            realm,
            ExecutableId::Script,
            lexical_env,
            variable_env,
        )
    }

    #[inline]
    pub const fn job(
        realm: RealmRef,
        executable: ExecutableId,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) -> Self {
        Self::new(
            ExecutionContextKind::Job,
            realm,
            executable,
            lexical_env,
            variable_env,
        )
    }

    #[inline]
    pub const fn bytecode(
        realm: RealmRef,
        code: CodeRef,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) -> Self {
        Self::new(
            ExecutionContextKind::Function,
            realm,
            ExecutableId::Bytecode(code),
            lexical_env,
            variable_env,
        )
    }

    #[inline]
    pub const fn with_private_env(mut self, private_env: Option<EnvironmentRef>) -> Self {
        self.private_env = private_env;
        self
    }

    #[inline]
    pub const fn with_script_or_module_referrer(mut self, referrer: Option<AtomId>) -> Self {
        self.script_or_module_referrer = referrer;
        self
    }

    #[inline]
    pub const fn with_this_state(mut self, this_state: ThisState) -> Self {
        self.this_state = this_state;
        self
    }

    #[inline]
    pub const fn with_new_target(mut self, new_target: Option<ObjectRef>) -> Self {
        self.new_target = new_target;
        self
    }

    #[inline]
    pub const fn realm(self) -> RealmRef {
        self.realm
    }

    #[inline]
    pub const fn executable(self) -> ExecutableId {
        self.executable
    }

    #[inline]
    pub const fn lexical_env(self) -> EnvironmentRef {
        self.lexical_env
    }

    #[inline]
    pub const fn variable_env(self) -> EnvironmentRef {
        self.variable_env
    }

    #[inline]
    pub const fn private_env(self) -> Option<EnvironmentRef> {
        self.private_env
    }

    #[inline]
    pub const fn script_or_module_referrer(self) -> Option<AtomId> {
        self.script_or_module_referrer
    }

    #[inline]
    pub const fn this_state(self) -> ThisState {
        self.this_state
    }

    #[inline]
    pub const fn new_target(self) -> Option<ObjectRef> {
        self.new_target
    }

    #[inline]
    pub const fn kind(self) -> ExecutionContextKind {
        self.kind
    }
}

/// Read-only view over one realm record and its typed intrinsic table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RealmRecord {
    pub(crate) id: RealmRef,
    pub(crate) global_object: ObjectRef,
    pub(crate) global_env: EnvironmentRef,
    pub(crate) bootstrap_code: Option<CodeRef>,
    pub(crate) root_shape: Option<ShapeId>,
    pub(crate) intrinsics: Intrinsics,
    pub(crate) bootstrap_state: RealmBootstrapState,
    pub(crate) is_default: bool,
}

impl RealmRecord {
    #[inline]
    pub const fn id(self) -> RealmRef {
        self.id
    }

    #[inline]
    pub const fn global_object(self) -> ObjectRef {
        self.global_object
    }

    #[inline]
    pub const fn global_env(self) -> EnvironmentRef {
        self.global_env
    }

    #[inline]
    pub const fn bootstrap_code(self) -> Option<CodeRef> {
        self.bootstrap_code
    }

    #[inline]
    pub const fn root_shape(self) -> Option<ShapeId> {
        self.root_shape
    }

    #[inline]
    pub const fn intrinsics(self) -> Intrinsics {
        self.intrinsics
    }

    #[inline]
    pub const fn bootstrap_state(self) -> RealmBootstrapState {
        self.bootstrap_state
    }

    #[inline]
    pub const fn is_default(self) -> bool {
        self.is_default
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RealmBootstrapState {
    spec_ready: bool,
    embedding_ready: bool,
}

impl RealmBootstrapState {
    #[inline]
    pub const fn new() -> Self {
        Self {
            spec_ready: false,
            embedding_ready: false,
        }
    }

    #[inline]
    pub const fn spec_ready(self) -> bool {
        self.spec_ready
    }

    #[inline]
    pub const fn embedding_ready(self) -> bool {
        self.embedding_ready
    }

    #[inline]
    pub const fn with_spec_ready(mut self, spec_ready: bool) -> Self {
        self.spec_ready = spec_ready;
        self
    }

    #[inline]
    pub const fn with_embedding_ready(mut self, embedding_ready: bool) -> Self {
        self.embedding_ready = embedding_ready;
        self
    }
}

impl TraceHeapEdges for ExecutableId {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        if let Self::Bytecode(code) = self {
            code.trace_heap_edges(tracer);
        }
    }
}

impl TraceHeapEdges for ThisState {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        if let Self::Value(value) = self {
            value.trace_heap_edges(tracer);
        }
    }
}

impl TraceHeapEdges for ExecutionContext {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.realm.trace_heap_edges(tracer);
        self.executable.trace_heap_edges(tracer);
        self.lexical_env.trace_heap_edges(tracer);
        self.variable_env.trace_heap_edges(tracer);
        self.private_env.trace_heap_edges(tracer);
        self.this_state.trace_heap_edges(tracer);
        self.new_target.trace_heap_edges(tracer);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RealmMetadata {
    pub(crate) intrinsics: Intrinsics,
    pub(crate) bootstrap_state: RealmBootstrapState,
    pub(crate) is_default: bool,
}
