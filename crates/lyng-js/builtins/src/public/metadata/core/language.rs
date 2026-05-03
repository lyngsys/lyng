use super::super::*;

pub(in crate::public::metadata) const PUBLIC_MODULE_BUILTIN_METADATA:
    &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        abstract_module_source_builtin,
        BuiltinEntryMetadata::new("AbstractModuleSource", 0, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        abstract_module_source_to_string_tag_getter_builtin,
        BuiltinEntryMetadata::new("get [Symbol.toStringTag]", 0, false, false),
    ),
];

pub(in crate::public::metadata) const PUBLIC_LANGUAGE_SUPPORT_BUILTIN_METADATA:
    &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        error_builtin,
        BuiltinEntryMetadata::new("Error", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        error_to_string_builtin,
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        eval_error_builtin,
        BuiltinEntryMetadata::new("EvalError", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        range_error_builtin,
        BuiltinEntryMetadata::new("RangeError", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        reference_error_builtin,
        BuiltinEntryMetadata::new("ReferenceError", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        syntax_error_builtin,
        BuiltinEntryMetadata::new("SyntaxError", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        type_error_builtin,
        BuiltinEntryMetadata::new("TypeError", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        uri_error_builtin,
        BuiltinEntryMetadata::new("URIError", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        aggregate_error_builtin,
        BuiltinEntryMetadata::new("AggregateError", 2, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        suppressed_error_builtin,
        BuiltinEntryMetadata::new("SuppressedError", 3, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        error_is_error_builtin,
        BuiltinEntryMetadata::new("isError", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        eval_builtin,
        BuiltinEntryMetadata::new("eval", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_builtin,
        BuiltinEntryMetadata::new("Promise", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        disposable_stack_builtin,
        BuiltinEntryMetadata::new("DisposableStack", 0, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        disposable_stack_use_builtin,
        BuiltinEntryMetadata::new("use", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        disposable_stack_adopt_builtin,
        BuiltinEntryMetadata::new("adopt", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        disposable_stack_defer_builtin,
        BuiltinEntryMetadata::new("defer", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        disposable_stack_move_builtin,
        BuiltinEntryMetadata::new("move", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        disposable_stack_disposed_getter_builtin,
        BuiltinEntryMetadata::new("get disposed", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        disposable_stack_dispose_builtin,
        BuiltinEntryMetadata::new("dispose", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        async_disposable_stack_builtin,
        BuiltinEntryMetadata::new("AsyncDisposableStack", 0, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        async_disposable_stack_use_builtin,
        BuiltinEntryMetadata::new("use", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        async_disposable_stack_adopt_builtin,
        BuiltinEntryMetadata::new("adopt", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        async_disposable_stack_defer_builtin,
        BuiltinEntryMetadata::new("defer", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        async_disposable_stack_move_builtin,
        BuiltinEntryMetadata::new("move", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        async_disposable_stack_disposed_getter_builtin,
        BuiltinEntryMetadata::new("get disposed", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        async_disposable_stack_dispose_async_builtin,
        BuiltinEntryMetadata::new("disposeAsync", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        async_disposal_resume_builtin,
        BuiltinEntryMetadata::new("", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        create_sync_disposal_scope_builtin,
        BuiltinEntryMetadata::new("", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        create_async_disposal_scope_builtin,
        BuiltinEntryMetadata::new("", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        add_sync_disposable_resource_builtin,
        BuiltinEntryMetadata::new("", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        add_async_disposable_resource_builtin,
        BuiltinEntryMetadata::new("", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        dispose_scope_builtin,
        BuiltinEntryMetadata::new("", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        dispose_scope_async_builtin,
        BuiltinEntryMetadata::new("", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_then_builtin,
        BuiltinEntryMetadata::new("then", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_catch_builtin,
        BuiltinEntryMetadata::new("catch", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_finally_builtin,
        BuiltinEntryMetadata::new("finally", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_finally_function_builtin,
        BuiltinEntryMetadata::new("", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_finally_continuation_builtin,
        BuiltinEntryMetadata::new("", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_resolve_builtin,
        BuiltinEntryMetadata::new("resolve", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_reject_builtin,
        BuiltinEntryMetadata::new("reject", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_all_builtin,
        BuiltinEntryMetadata::new("all", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_all_settled_builtin,
        BuiltinEntryMetadata::new("allSettled", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_race_builtin,
        BuiltinEntryMetadata::new("race", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_any_builtin,
        BuiltinEntryMetadata::new("any", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_try_builtin,
        BuiltinEntryMetadata::new("try", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_with_resolvers_builtin,
        BuiltinEntryMetadata::new("withResolvers", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_species_getter_builtin,
        BuiltinEntryMetadata::new("get [Symbol.species]", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_capability_executor_builtin,
        BuiltinEntryMetadata::new("", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_resolve_function_builtin,
        BuiltinEntryMetadata::new("", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_reject_function_builtin,
        BuiltinEntryMetadata::new("", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_all_resolve_element_builtin,
        BuiltinEntryMetadata::new("", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_all_settled_resolve_element_builtin,
        BuiltinEntryMetadata::new("", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_all_settled_reject_element_builtin,
        BuiltinEntryMetadata::new("", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        promise_any_reject_element_builtin,
        BuiltinEntryMetadata::new("", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        parse_int_builtin,
        BuiltinEntryMetadata::new("parseInt", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        parse_float_builtin,
        BuiltinEntryMetadata::new("parseFloat", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        is_nan_builtin,
        BuiltinEntryMetadata::new("isNaN", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        is_finite_builtin,
        BuiltinEntryMetadata::new("isFinite", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        encode_uri_builtin,
        BuiltinEntryMetadata::new("encodeURI", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        encode_uri_component_builtin,
        BuiltinEntryMetadata::new("encodeURIComponent", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        decode_uri_builtin,
        BuiltinEntryMetadata::new("decodeURI", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        decode_uri_component_builtin,
        BuiltinEntryMetadata::new("decodeURIComponent", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        escape_builtin,
        BuiltinEntryMetadata::new("escape", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        unescape_builtin,
        BuiltinEntryMetadata::new("unescape", 1, false, false),
    ),
];
