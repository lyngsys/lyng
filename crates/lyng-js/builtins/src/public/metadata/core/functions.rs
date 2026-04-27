use super::super::*;

pub(in crate::public::metadata) const PUBLIC_FUNCTION_BUILTIN_METADATA:
    &[PublicBuiltinMetadataRow] = &[
    PublicBuiltinMetadataRow::new(
        function_builtin,
        BuiltinEntryMetadata::new("Function", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        function_prototype_builtin,
        BuiltinEntryMetadata::new("", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        function_call_builtin,
        BuiltinEntryMetadata::new("call", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        function_apply_builtin,
        BuiltinEntryMetadata::new("apply", 2, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        function_bind_builtin,
        BuiltinEntryMetadata::new("bind", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        function_to_string_builtin,
        BuiltinEntryMetadata::new("toString", 0, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        function_symbol_has_instance_builtin,
        BuiltinEntryMetadata::new("[Symbol.hasInstance]", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        async_function_builtin,
        BuiltinEntryMetadata::new("AsyncFunction", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        async_generator_function_builtin,
        BuiltinEntryMetadata::new("AsyncGeneratorFunction", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        async_generator_next_builtin,
        BuiltinEntryMetadata::new("next", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        async_generator_return_builtin,
        BuiltinEntryMetadata::new("return", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        async_generator_throw_builtin,
        BuiltinEntryMetadata::new("throw", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        generator_function_builtin,
        BuiltinEntryMetadata::new("GeneratorFunction", 1, true, true),
    ),
    PublicBuiltinMetadataRow::new(
        generator_next_builtin,
        BuiltinEntryMetadata::new("next", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        generator_return_builtin,
        BuiltinEntryMetadata::new("return", 1, false, false),
    ),
    PublicBuiltinMetadataRow::new(
        generator_throw_builtin,
        BuiltinEntryMetadata::new("throw", 1, false, false),
    ),
];
