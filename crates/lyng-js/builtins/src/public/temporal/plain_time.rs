use super::{
    allocate_builtin_function_object, define_builtin_accessor_property,
    define_builtin_data_property, public_builtin_metadata,
};
use lyng_js_common::WellKnownAtom;
use lyng_js_env::Agent;
use lyng_js_types::{EnvironmentRef, ObjectRef, PropertyKey, RealmRef, ShapeId, Value};

#[derive(Clone, Copy)]
pub(super) struct PlainTimeBootstrapContext {
    realm: RealmRef,
    global_env: EnvironmentRef,
    root_shape: ShapeId,
    function_prototype: ObjectRef,
    object_prototype: ObjectRef,
}

impl PlainTimeBootstrapContext {
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
pub(super) struct PlainTimeFunctions {
    constructor: ObjectRef,
    hour_getter: ObjectRef,
    minute_getter: ObjectRef,
    second_getter: ObjectRef,
    millisecond_getter: ObjectRef,
    microsecond_getter: ObjectRef,
    nanosecond_getter: ObjectRef,
    to_string: ObjectRef,
    to_json: ObjectRef,
    to_locale_string: ObjectRef,
    value_of: ObjectRef,
    equals: ObjectRef,
    with: ObjectRef,
    add: ObjectRef,
    subtract: ObjectRef,
    round: ObjectRef,
    since: ObjectRef,
    until: ObjectRef,
    from: ObjectRef,
    compare: ObjectRef,
}

#[derive(Clone, Copy)]
pub(super) struct PlainTimePrototypeProperties {
    pub(super) hour_key: PropertyKey,
    pub(super) minute_key: PropertyKey,
    pub(super) second_key: PropertyKey,
    pub(super) millisecond_key: PropertyKey,
    pub(super) microsecond_key: PropertyKey,
    pub(super) nanosecond_key: PropertyKey,
    pub(super) to_json_key: PropertyKey,
    pub(super) to_locale_string_key: PropertyKey,
    pub(super) equals_key: PropertyKey,
    pub(super) with_key: PropertyKey,
    pub(super) add_key: PropertyKey,
    pub(super) subtract_key: PropertyKey,
    pub(super) round_key: PropertyKey,
    pub(super) since_key: PropertyKey,
    pub(super) until_key: PropertyKey,
    pub(super) to_string_tag_key: PropertyKey,
    pub(super) prototype_tag: Value,
}

pub(super) fn allocate_functions(
    agent: &mut Agent,
    context: PlainTimeBootstrapContext,
    plain_time_prototype: ObjectRef,
) -> Option<PlainTimeFunctions> {
    let constructor = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_time_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_time_builtin())?,
        Some(plain_time_prototype),
    );
    let hour_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_time_hour_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_time_hour_getter_builtin())?,
        None,
    );
    let minute_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_time_minute_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_time_minute_getter_builtin())?,
        None,
    );
    let second_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_time_second_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_time_second_getter_builtin())?,
        None,
    );
    let millisecond_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_time_millisecond_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_time_millisecond_getter_builtin())?,
        None,
    );
    let microsecond_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_time_microsecond_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_time_microsecond_getter_builtin())?,
        None,
    );
    let nanosecond_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_time_nanosecond_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_time_nanosecond_getter_builtin())?,
        None,
    );
    let to_string = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_time_to_string_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_time_to_string_builtin())?,
        None,
    );
    let to_json = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_time_to_json_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_time_to_json_builtin())?,
        None,
    );
    let to_locale_string = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_time_to_locale_string_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_time_to_locale_string_builtin())?,
        None,
    );
    let value_of = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_time_value_of_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_time_value_of_builtin())?,
        None,
    );
    let equals = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_time_equals_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_time_equals_builtin())?,
        None,
    );
    let with = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_time_with_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_time_with_builtin())?,
        None,
    );
    let add = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_time_add_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_time_add_builtin())?,
        None,
    );
    let subtract = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_time_subtract_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_time_subtract_builtin())?,
        None,
    );
    let round = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_time_round_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_time_round_builtin())?,
        None,
    );
    let since = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_time_since_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_time_since_builtin())?,
        None,
    );
    let until = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_time_until_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_time_until_builtin())?,
        None,
    );
    let from = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_time_from_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_time_from_builtin())?,
        None,
    );
    let compare = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_time_compare_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_time_compare_builtin())?,
        None,
    );

    Some(PlainTimeFunctions {
        constructor,
        hour_getter,
        minute_getter,
        second_getter,
        millisecond_getter,
        microsecond_getter,
        nanosecond_getter,
        to_string,
        to_json,
        to_locale_string,
        value_of,
        equals,
        with,
        add,
        subtract,
        round,
        since,
        until,
        from,
        compare,
    })
}

pub(super) fn install_temporal_object_property(
    agent: &mut Agent,
    temporal_object: ObjectRef,
    plain_time_key: PropertyKey,
    functions: PlainTimeFunctions,
) {
    define_builtin_data_property(
        agent,
        temporal_object,
        plain_time_key,
        Value::from_object_ref(functions.constructor),
        true,
        false,
        true,
    );
}

pub(super) fn install_constructor_properties(
    agent: &mut Agent,
    functions: PlainTimeFunctions,
    from_key: PropertyKey,
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
        compare_key,
        Value::from_object_ref(functions.compare),
        true,
        false,
        true,
    );
}

pub(super) fn install_prototype_properties(
    agent: &mut Agent,
    plain_time_prototype: ObjectRef,
    functions: PlainTimeFunctions,
    properties: PlainTimePrototypeProperties,
) {
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        Value::from_object_ref(functions.constructor),
        true,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_time_prototype,
        properties.hour_key,
        Some(functions.hour_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_time_prototype,
        properties.minute_key,
        Some(functions.minute_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_time_prototype,
        properties.second_key,
        Some(functions.second_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_time_prototype,
        properties.millisecond_key,
        Some(functions.millisecond_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_time_prototype,
        properties.microsecond_key,
        Some(functions.microsecond_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_time_prototype,
        properties.nanosecond_key,
        Some(functions.nanosecond_getter),
        None,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        PropertyKey::from_atom(WellKnownAtom::toString.id()),
        Value::from_object_ref(functions.to_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        properties.to_json_key,
        Value::from_object_ref(functions.to_json),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        properties.to_locale_string_key,
        Value::from_object_ref(functions.to_locale_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        properties.equals_key,
        Value::from_object_ref(functions.equals),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        properties.with_key,
        Value::from_object_ref(functions.with),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        properties.add_key,
        Value::from_object_ref(functions.add),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        properties.subtract_key,
        Value::from_object_ref(functions.subtract),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        properties.round_key,
        Value::from_object_ref(functions.round),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        properties.since_key,
        Value::from_object_ref(functions.since),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        properties.until_key,
        Value::from_object_ref(functions.until),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        PropertyKey::from_atom(WellKnownAtom::valueOf.id()),
        Value::from_object_ref(functions.value_of),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        properties.to_string_tag_key,
        properties.prototype_tag,
        false,
        false,
        true,
    );
}
