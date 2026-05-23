use super::descriptors::{
    builtin_function_atom_property, data_atom_property, descriptor_tag_with_atom,
    writable_builtin_attributes,
};
use super::{
    install_public_builtin_function, install_public_builtin_function_with_function_prototype,
    ErrorFamilyBuiltins, ErrorFamilyPrototypes, FamilyInstallContext,
};
use crate::bootstrap::{install_descriptor_tables, BuiltinBootstrapError};
use crate::public::{BuiltinCache, PublicRealmBuiltins};
use crate::{BuiltinDescriptorTable, BuiltinInstallTarget, BuiltinIntrinsic};
use lyng_js_common::{AtomId, WellKnownAtom};
use lyng_js_env::Agent;
use lyng_js_types::{
    aggregate_error_builtin, error_builtin, error_is_error_builtin, error_to_string_builtin,
    eval_error_builtin, range_error_builtin, reference_error_builtin, suppressed_error_builtin,
    syntax_error_builtin, type_error_builtin, uri_error_builtin, BuiltinFunctionId, ObjectRef,
    RealmRef, Value,
};

pub(in crate::public) fn install_error_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototypes: ErrorFamilyPrototypes,
) -> ErrorFamilyBuiltins {
    let error = install_public_builtin_function(
        agent,
        cx,
        error_builtin(),
        Some(prototypes.error_prototype),
    );

    ErrorFamilyBuiltins {
        error,
        error_prototype: prototypes.error_prototype,
        error_to_string: install_public_builtin_function(
            agent,
            cx,
            error_to_string_builtin(),
            None,
        ),
        eval_error: install_error_constructor(
            agent,
            cx,
            error,
            eval_error_builtin(),
            prototypes.eval_error_prototype,
        ),
        eval_error_prototype: prototypes.eval_error_prototype,
        range_error: install_error_constructor(
            agent,
            cx,
            error,
            range_error_builtin(),
            prototypes.range_error_prototype,
        ),
        range_error_prototype: prototypes.range_error_prototype,
        reference_error: install_error_constructor(
            agent,
            cx,
            error,
            reference_error_builtin(),
            prototypes.reference_error_prototype,
        ),
        reference_error_prototype: prototypes.reference_error_prototype,
        syntax_error: install_error_constructor(
            agent,
            cx,
            error,
            syntax_error_builtin(),
            prototypes.syntax_error_prototype,
        ),
        syntax_error_prototype: prototypes.syntax_error_prototype,
        type_error: install_error_constructor(
            agent,
            cx,
            error,
            type_error_builtin(),
            prototypes.type_error_prototype,
        ),
        type_error_prototype: prototypes.type_error_prototype,
        uri_error: install_error_constructor(
            agent,
            cx,
            error,
            uri_error_builtin(),
            prototypes.uri_error_prototype,
        ),
        uri_error_prototype: prototypes.uri_error_prototype,
        aggregate_error: install_error_constructor(
            agent,
            cx,
            error,
            aggregate_error_builtin(),
            prototypes.aggregate_error_prototype,
        ),
        aggregate_error_prototype: prototypes.aggregate_error_prototype,
        suppressed_error: install_error_constructor(
            agent,
            cx,
            error,
            suppressed_error_builtin(),
            prototypes.suppressed_error_prototype,
        ),
        suppressed_error_prototype: prototypes.suppressed_error_prototype,
        error_is_error: install_public_builtin_function(agent, cx, error_is_error_builtin(), None),
    }
}

pub(in crate::public) fn error_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (error_builtin(), builtins.error),
        (error_to_string_builtin(), builtins.error_to_string),
        (eval_error_builtin(), builtins.eval_error),
        (range_error_builtin(), builtins.range_error),
        (reference_error_builtin(), builtins.reference_error),
        (syntax_error_builtin(), builtins.syntax_error),
        (type_error_builtin(), builtins.type_error),
        (uri_error_builtin(), builtins.uri_error),
        (aggregate_error_builtin(), builtins.aggregate_error),
        (suppressed_error_builtin(), builtins.suppressed_error),
        (error_is_error_builtin(), builtins.error_is_error),
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

pub(in crate::public) fn install_error_family_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
) -> Result<(), BuiltinBootstrapError> {
    let values = ErrorDescriptorValues::new(agent);
    install_error_constructor_descriptors(agent, cache, realm, values)?;
    install_error_prototype_descriptors(agent, cache, realm, builtins.error, values)?;

    for spec in native_error_prototype_specs(builtins, values) {
        install_native_error_prototype_descriptors(agent, cache, realm, spec, values)?;
    }

    Ok(())
}

fn install_error_constructor_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    values: ErrorDescriptorValues,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [builtin_function_atom_property(
        values.is_error,
        error_is_error_builtin(),
    )];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Error),
            &descriptors,
        )],
    )
}

fn install_error_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    error: ObjectRef,
    values: ErrorDescriptorValues,
) -> Result<(), BuiltinBootstrapError> {
    let error_prototype_descriptors = [
        data_atom_property(
            WellKnownAtom::constructor.id(),
            Value::from_object_ref(error),
            writable_builtin_attributes(),
        ),
        builtin_function_atom_property(WellKnownAtom::toString.id(), error_to_string_builtin()),
        data_atom_property(
            WellKnownAtom::name.id(),
            values.error_name,
            writable_builtin_attributes(),
        ),
        data_atom_property(
            values.message,
            values.empty_string,
            writable_builtin_attributes(),
        ),
    ];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::ErrorPrototype),
            &error_prototype_descriptors,
        )],
    )
}

fn install_native_error_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    spec: NativeErrorPrototypeSpec,
    values: ErrorDescriptorValues,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = native_error_prototype_descriptors(
        spec.constructor,
        values.message,
        values.empty_string,
        spec.name,
    );
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(spec.target),
            &descriptors,
        )],
    )
}

#[derive(Clone, Copy)]
struct ErrorDescriptorValues {
    message: AtomId,
    empty_string: Value,
    error_name: Value,
    eval_error_name: Value,
    range_error_name: Value,
    reference_error_name: Value,
    syntax_error_name: Value,
    type_error_name: Value,
    uri_error_name: Value,
    aggregate_error_name: Value,
    suppressed_error_name: Value,
    is_error: AtomId,
}

impl ErrorDescriptorValues {
    fn new(agent: &mut Agent) -> Self {
        let bootstrap_atoms = agent.bootstrap_atoms();
        let suppressed_error_name_atom = agent.atoms_mut().intern("SuppressedError");
        let is_error_atom = agent.atoms_mut().intern("isError");

        Self {
            message: bootstrap_atoms.message(),
            empty_string: descriptor_tag_with_atom(agent, "", WellKnownAtom::Empty.id()),
            error_name: descriptor_tag_with_atom(agent, "Error", bootstrap_atoms.error()),
            eval_error_name: descriptor_tag_with_atom(
                agent,
                "EvalError",
                bootstrap_atoms.eval_error(),
            ),
            range_error_name: descriptor_tag_with_atom(
                agent,
                "RangeError",
                bootstrap_atoms.range_error(),
            ),
            reference_error_name: descriptor_tag_with_atom(
                agent,
                "ReferenceError",
                bootstrap_atoms.reference_error(),
            ),
            syntax_error_name: descriptor_tag_with_atom(
                agent,
                "SyntaxError",
                bootstrap_atoms.syntax_error(),
            ),
            type_error_name: descriptor_tag_with_atom(
                agent,
                "TypeError",
                bootstrap_atoms.type_error(),
            ),
            uri_error_name: descriptor_tag_with_atom(
                agent,
                "URIError",
                bootstrap_atoms.uri_error(),
            ),
            aggregate_error_name: descriptor_tag_with_atom(
                agent,
                "AggregateError",
                bootstrap_atoms.aggregate_error(),
            ),
            suppressed_error_name: descriptor_tag_with_atom(
                agent,
                "SuppressedError",
                suppressed_error_name_atom,
            ),
            is_error: is_error_atom,
        }
    }
}

#[derive(Clone, Copy)]
struct NativeErrorPrototypeSpec {
    target: BuiltinIntrinsic,
    constructor: ObjectRef,
    name: Value,
}

const fn native_error_prototype_specs(
    builtins: &PublicRealmBuiltins,
    values: ErrorDescriptorValues,
) -> [NativeErrorPrototypeSpec; 8] {
    [
        NativeErrorPrototypeSpec {
            target: BuiltinIntrinsic::EvalErrorPrototype,
            constructor: builtins.eval_error,
            name: values.eval_error_name,
        },
        NativeErrorPrototypeSpec {
            target: BuiltinIntrinsic::RangeErrorPrototype,
            constructor: builtins.range_error,
            name: values.range_error_name,
        },
        NativeErrorPrototypeSpec {
            target: BuiltinIntrinsic::ReferenceErrorPrototype,
            constructor: builtins.reference_error,
            name: values.reference_error_name,
        },
        NativeErrorPrototypeSpec {
            target: BuiltinIntrinsic::SyntaxErrorPrototype,
            constructor: builtins.syntax_error,
            name: values.syntax_error_name,
        },
        NativeErrorPrototypeSpec {
            target: BuiltinIntrinsic::TypeErrorPrototype,
            constructor: builtins.type_error,
            name: values.type_error_name,
        },
        NativeErrorPrototypeSpec {
            target: BuiltinIntrinsic::UriErrorPrototype,
            constructor: builtins.uri_error,
            name: values.uri_error_name,
        },
        NativeErrorPrototypeSpec {
            target: BuiltinIntrinsic::AggregateErrorPrototype,
            constructor: builtins.aggregate_error,
            name: values.aggregate_error_name,
        },
        NativeErrorPrototypeSpec {
            target: BuiltinIntrinsic::SuppressedErrorPrototype,
            constructor: builtins.suppressed_error,
            name: values.suppressed_error_name,
        },
    ]
}

const fn native_error_prototype_descriptors(
    constructor: ObjectRef,
    message: AtomId,
    empty_string: Value,
    name: Value,
) -> [crate::BuiltinPropertyDescriptor; 3] {
    [
        data_atom_property(
            WellKnownAtom::constructor.id(),
            Value::from_object_ref(constructor),
            writable_builtin_attributes(),
        ),
        data_atom_property(
            WellKnownAtom::name.id(),
            name,
            writable_builtin_attributes(),
        ),
        data_atom_property(message, empty_string, writable_builtin_attributes()),
    ]
}
