use super::{
    allocate_builtin_function_object, define_builtin_accessor_property,
    define_builtin_data_property, public_builtin_metadata,
};
use lyng_js_common::WellKnownAtom;
use lyng_js_env::Agent;
use lyng_js_types::{EnvironmentRef, ObjectRef, PropertyKey, RealmRef, ShapeId, Value};

#[derive(Clone, Copy)]
pub(super) struct InstantBootstrapContext {
    realm: RealmRef,
    global_env: EnvironmentRef,
    root_shape: ShapeId,
    function_prototype: ObjectRef,
    object_prototype: ObjectRef,
}

impl InstantBootstrapContext {
    pub(super) const fn new(
        realm: RealmRef,
        global_env: EnvironmentRef,
        root_shape: ShapeId,
        function_prototype: ObjectRef,
        object_prototype: ObjectRef,
    ) -> Self {
        Self {
            realm,
            global_env,
            root_shape,
            function_prototype,
            object_prototype,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct InstantFunctions {
    constructor: ObjectRef,
    from: ObjectRef,
    from_epoch_nanoseconds: ObjectRef,
    from_epoch_milliseconds: ObjectRef,
    compare: ObjectRef,
    epoch_nanoseconds_getter: ObjectRef,
    epoch_milliseconds_getter: ObjectRef,
    epoch_seconds_getter: ObjectRef,
    to_string: ObjectRef,
    to_json: ObjectRef,
    to_locale_string: ObjectRef,
    value_of: ObjectRef,
    equals: ObjectRef,
    add: ObjectRef,
    subtract: ObjectRef,
    round: ObjectRef,
    since: ObjectRef,
    until: ObjectRef,
    to_zoned_date_time_iso: ObjectRef,
}

#[derive(Clone, Copy)]
pub(super) struct InstantPrototypeProperties {
    pub(super) epoch_nanoseconds_key: PropertyKey,
    pub(super) epoch_milliseconds_key: PropertyKey,
    pub(super) epoch_seconds_key: PropertyKey,
    pub(super) to_json_key: PropertyKey,
    pub(super) to_locale_string_key: PropertyKey,
    pub(super) equals_key: PropertyKey,
    pub(super) add_key: PropertyKey,
    pub(super) subtract_key: PropertyKey,
    pub(super) round_key: PropertyKey,
    pub(super) since_key: PropertyKey,
    pub(super) until_key: PropertyKey,
    pub(super) to_zoned_date_time_iso_key: PropertyKey,
    pub(super) to_string_tag_key: PropertyKey,
    pub(super) prototype_tag: Value,
}

pub(super) fn allocate_functions(
    agent: &mut Agent,
    context: InstantBootstrapContext,
    instant_prototype: ObjectRef,
) -> Option<InstantFunctions> {
    let constructor = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_instant_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_instant_builtin())?,
        Some(instant_prototype),
    );
    let from = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_instant_from_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_instant_from_builtin())?,
        None,
    );
    let from_epoch_nanoseconds = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_instant_from_epoch_nanoseconds_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_instant_from_epoch_nanoseconds_builtin())?,
        None,
    );
    let from_epoch_milliseconds = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_instant_from_epoch_milliseconds_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_instant_from_epoch_milliseconds_builtin())?,
        None,
    );
    let compare = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_instant_compare_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_instant_compare_builtin())?,
        None,
    );
    let epoch_nanoseconds_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_instant_epoch_nanoseconds_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_instant_epoch_nanoseconds_getter_builtin(),
        )?,
        None,
    );
    let epoch_milliseconds_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_instant_epoch_milliseconds_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_instant_epoch_milliseconds_getter_builtin(),
        )?,
        None,
    );
    let epoch_seconds_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_instant_epoch_seconds_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_instant_epoch_seconds_getter_builtin())?,
        None,
    );
    let to_string = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_instant_to_string_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_instant_to_string_builtin())?,
        None,
    );
    let to_json = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_instant_to_json_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_instant_to_json_builtin())?,
        None,
    );
    let to_locale_string = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_instant_to_locale_string_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_instant_to_locale_string_builtin())?,
        None,
    );
    let value_of = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_instant_value_of_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_instant_value_of_builtin())?,
        None,
    );
    let equals = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_instant_equals_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_instant_equals_builtin())?,
        None,
    );
    let add = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_instant_add_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_instant_add_builtin())?,
        None,
    );
    let subtract = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_instant_subtract_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_instant_subtract_builtin())?,
        None,
    );
    let round = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_instant_round_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_instant_round_builtin())?,
        None,
    );
    let since = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_instant_since_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_instant_since_builtin())?,
        None,
    );
    let until = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_instant_until_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_instant_until_builtin())?,
        None,
    );
    let to_zoned_date_time_iso = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_instant_to_zoned_date_time_iso_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_instant_to_zoned_date_time_iso_builtin())?,
        None,
    );

    Some(InstantFunctions {
        constructor,
        from,
        from_epoch_nanoseconds,
        from_epoch_milliseconds,
        compare,
        epoch_nanoseconds_getter,
        epoch_milliseconds_getter,
        epoch_seconds_getter,
        to_string,
        to_json,
        to_locale_string,
        value_of,
        equals,
        add,
        subtract,
        round,
        since,
        until,
        to_zoned_date_time_iso,
    })
}

pub(super) fn install_temporal_object_property(
    agent: &mut Agent,
    temporal_object: ObjectRef,
    instant_key: PropertyKey,
    functions: InstantFunctions,
) {
    define_builtin_data_property(
        agent,
        temporal_object,
        instant_key,
        Value::from_object_ref(functions.constructor),
        true,
        false,
        true,
    );
}

pub(super) fn install_constructor_properties(
    agent: &mut Agent,
    functions: InstantFunctions,
    from_key: PropertyKey,
    from_epoch_nanoseconds_key: PropertyKey,
    from_epoch_milliseconds_key: PropertyKey,
    compare_key: PropertyKey,
) {
    define_builtin_data_property(
        agent,
        functions.constructor,
        from_key,
        Value::from_object_ref(functions.from),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        functions.constructor,
        from_epoch_nanoseconds_key,
        Value::from_object_ref(functions.from_epoch_nanoseconds),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        functions.constructor,
        from_epoch_milliseconds_key,
        Value::from_object_ref(functions.from_epoch_milliseconds),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        functions.constructor,
        compare_key,
        Value::from_object_ref(functions.compare),
        true,
        false,
        true,
    );
}

pub(super) fn install_prototype_properties(
    agent: &mut Agent,
    instant_prototype: ObjectRef,
    functions: InstantFunctions,
    properties: InstantPrototypeProperties,
) {
    define_builtin_data_property(
        agent,
        instant_prototype,
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        Value::from_object_ref(functions.constructor),
        true,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        instant_prototype,
        properties.epoch_nanoseconds_key,
        Some(functions.epoch_nanoseconds_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        instant_prototype,
        properties.epoch_milliseconds_key,
        Some(functions.epoch_milliseconds_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        instant_prototype,
        properties.epoch_seconds_key,
        Some(functions.epoch_seconds_getter),
        None,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        PropertyKey::from_atom(WellKnownAtom::toString.id()),
        Value::from_object_ref(functions.to_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        properties.to_json_key,
        Value::from_object_ref(functions.to_json),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        properties.to_locale_string_key,
        Value::from_object_ref(functions.to_locale_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        properties.equals_key,
        Value::from_object_ref(functions.equals),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        properties.add_key,
        Value::from_object_ref(functions.add),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        properties.subtract_key,
        Value::from_object_ref(functions.subtract),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        properties.round_key,
        Value::from_object_ref(functions.round),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        properties.since_key,
        Value::from_object_ref(functions.since),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        properties.until_key,
        Value::from_object_ref(functions.until),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        properties.to_zoned_date_time_iso_key,
        Value::from_object_ref(functions.to_zoned_date_time_iso),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        PropertyKey::from_atom(WellKnownAtom::valueOf.id()),
        Value::from_object_ref(functions.value_of),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        properties.to_string_tag_key,
        properties.prototype_tag,
        false,
        false,
        true,
    );
}
