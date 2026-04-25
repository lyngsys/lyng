use super::{
    install_public_builtin_function, install_public_builtin_function_with_function_prototype,
    ErrorFamilyBuiltins, ErrorFamilyPrototypes, FamilyInstallContext,
};
use crate::public::PublicRealmBuiltins;
use lyng_js_env::Agent;
use lyng_js_types::{
    js3_aggregate_error_builtin, js3_error_builtin, js3_error_to_string_builtin,
    js3_eval_error_builtin, js3_range_error_builtin, js3_reference_error_builtin,
    js3_suppressed_error_builtin, js3_syntax_error_builtin, js3_type_error_builtin,
    js3_uri_error_builtin, BuiltinFunctionId, ObjectRef,
};

pub(in crate::public) fn install_error_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototypes: ErrorFamilyPrototypes,
) -> ErrorFamilyBuiltins {
    let error = install_public_builtin_function(
        agent,
        cx,
        js3_error_builtin(),
        Some(prototypes.error_prototype),
    );

    ErrorFamilyBuiltins {
        error,
        error_prototype: prototypes.error_prototype,
        error_to_string: install_public_builtin_function(
            agent,
            cx,
            js3_error_to_string_builtin(),
            None,
        ),
        eval_error: install_error_constructor(
            agent,
            cx,
            error,
            js3_eval_error_builtin(),
            prototypes.eval_error_prototype,
        ),
        eval_error_prototype: prototypes.eval_error_prototype,
        range_error: install_error_constructor(
            agent,
            cx,
            error,
            js3_range_error_builtin(),
            prototypes.range_error_prototype,
        ),
        range_error_prototype: prototypes.range_error_prototype,
        reference_error: install_error_constructor(
            agent,
            cx,
            error,
            js3_reference_error_builtin(),
            prototypes.reference_error_prototype,
        ),
        reference_error_prototype: prototypes.reference_error_prototype,
        syntax_error: install_error_constructor(
            agent,
            cx,
            error,
            js3_syntax_error_builtin(),
            prototypes.syntax_error_prototype,
        ),
        syntax_error_prototype: prototypes.syntax_error_prototype,
        type_error: install_error_constructor(
            agent,
            cx,
            error,
            js3_type_error_builtin(),
            prototypes.type_error_prototype,
        ),
        type_error_prototype: prototypes.type_error_prototype,
        uri_error: install_error_constructor(
            agent,
            cx,
            error,
            js3_uri_error_builtin(),
            prototypes.uri_error_prototype,
        ),
        uri_error_prototype: prototypes.uri_error_prototype,
        aggregate_error: install_error_constructor(
            agent,
            cx,
            error,
            js3_aggregate_error_builtin(),
            prototypes.aggregate_error_prototype,
        ),
        aggregate_error_prototype: prototypes.aggregate_error_prototype,
        suppressed_error: install_error_constructor(
            agent,
            cx,
            error,
            js3_suppressed_error_builtin(),
            prototypes.suppressed_error_prototype,
        ),
        suppressed_error_prototype: prototypes.suppressed_error_prototype,
    }
}

pub(in crate::public) fn error_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (js3_error_builtin(), builtins.error),
        (js3_error_to_string_builtin(), builtins.error_to_string),
        (js3_eval_error_builtin(), builtins.eval_error),
        (js3_range_error_builtin(), builtins.range_error),
        (js3_reference_error_builtin(), builtins.reference_error),
        (js3_syntax_error_builtin(), builtins.syntax_error),
        (js3_type_error_builtin(), builtins.type_error),
        (js3_uri_error_builtin(), builtins.uri_error),
        (js3_aggregate_error_builtin(), builtins.aggregate_error),
        (js3_suppressed_error_builtin(), builtins.suppressed_error),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}

fn install_error_constructor(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    error: lyng_js_types::ObjectRef,
    entry: lyng_js_types::BuiltinFunctionId,
    prototype: lyng_js_types::ObjectRef,
) -> lyng_js_types::ObjectRef {
    install_public_builtin_function_with_function_prototype(
        agent,
        cx,
        error,
        entry,
        Some(prototype),
    )
}
