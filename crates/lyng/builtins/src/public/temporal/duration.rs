use super::{
    allocate_builtin_function_object, define_builtin_accessor_property,
    define_builtin_data_property, public_builtin_metadata,
};
use lyng_js_common::WellKnownAtom;
use lyng_js_env::Agent;
use lyng_js_types::{EnvironmentRef, ObjectRef, PropertyKey, RealmRef, ShapeId, Value};

#[derive(Clone, Copy)]
pub(super) struct DurationBootstrapContext {
    realm: RealmRef,
    global_env: EnvironmentRef,
    root_shape: ShapeId,
    function_prototype: ObjectRef,
    object_prototype: ObjectRef,
}

impl DurationBootstrapContext {
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
pub(super) struct DurationFunctions {
    constructor: ObjectRef,
    years_getter: ObjectRef,
    months_getter: ObjectRef,
    weeks_getter: ObjectRef,
    days_getter: ObjectRef,
    hours_getter: ObjectRef,
    minutes_getter: ObjectRef,
    seconds_getter: ObjectRef,
    milliseconds_getter: ObjectRef,
    microseconds_getter: ObjectRef,
    nanoseconds_getter: ObjectRef,
    sign_getter: ObjectRef,
    blank_getter: ObjectRef,
    to_string: ObjectRef,
    to_json: ObjectRef,
    to_locale_string: ObjectRef,
    negated: ObjectRef,
    abs: ObjectRef,
    with: ObjectRef,
    round: ObjectRef,
    total: ObjectRef,
    add: ObjectRef,
    subtract: ObjectRef,
    value_of: ObjectRef,
    from: ObjectRef,
    compare: ObjectRef,
}

#[derive(Clone, Copy)]
pub(super) struct DurationPrototypeProperties {
    pub(super) years_key: PropertyKey,
    pub(super) months_key: PropertyKey,
    pub(super) weeks_key: PropertyKey,
    pub(super) days_key: PropertyKey,
    pub(super) hours_key: PropertyKey,
    pub(super) minutes_key: PropertyKey,
    pub(super) seconds_key: PropertyKey,
    pub(super) milliseconds_key: PropertyKey,
    pub(super) microseconds_key: PropertyKey,
    pub(super) nanoseconds_key: PropertyKey,
    pub(super) sign_key: PropertyKey,
    pub(super) blank_key: PropertyKey,
    pub(super) to_json_key: PropertyKey,
    pub(super) to_locale_string_key: PropertyKey,
    pub(super) negated_key: PropertyKey,
    pub(super) abs_key: PropertyKey,
    pub(super) with_key: PropertyKey,
    pub(super) round_key: PropertyKey,
    pub(super) total_key: PropertyKey,
    pub(super) add_key: PropertyKey,
    pub(super) subtract_key: PropertyKey,
    pub(super) to_string_tag_key: PropertyKey,
    pub(super) prototype_tag: Value,
}

#[allow(
    clippy::too_many_lines,
    reason = "Duration builtin allocation follows the ordered Temporal function table"
)]
pub(super) fn allocate_functions(
    agent: &mut Agent,
    context: DurationBootstrapContext,
    duration_prototype: ObjectRef,
) -> Option<DurationFunctions> {
    let constructor = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_builtin())?,
        Some(duration_prototype),
    );
    let years_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_years_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_years_getter_builtin())?,
        None,
    );
    let months_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_months_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_months_getter_builtin())?,
        None,
    );
    let weeks_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_weeks_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_weeks_getter_builtin())?,
        None,
    );
    let days_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_days_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_days_getter_builtin())?,
        None,
    );
    let hours_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_hours_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_hours_getter_builtin())?,
        None,
    );
    let minutes_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_minutes_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_minutes_getter_builtin())?,
        None,
    );
    let seconds_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_seconds_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_seconds_getter_builtin())?,
        None,
    );
    let milliseconds_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_milliseconds_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_milliseconds_getter_builtin())?,
        None,
    );
    let microseconds_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_microseconds_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_microseconds_getter_builtin())?,
        None,
    );
    let nanoseconds_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_nanoseconds_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_nanoseconds_getter_builtin())?,
        None,
    );
    let sign_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_sign_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_sign_getter_builtin())?,
        None,
    );
    let blank_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_blank_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_blank_getter_builtin())?,
        None,
    );
    let to_string = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_to_string_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_to_string_builtin())?,
        None,
    );
    let to_json = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_to_json_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_to_json_builtin())?,
        None,
    );
    let to_locale_string = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_to_locale_string_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_to_locale_string_builtin())?,
        None,
    );
    let negated = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_negated_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_negated_builtin())?,
        None,
    );
    let abs = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_abs_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_abs_builtin())?,
        None,
    );
    let with = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_with_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_with_builtin())?,
        None,
    );
    let round = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_round_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_round_builtin())?,
        None,
    );
    let total = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_total_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_total_builtin())?,
        None,
    );
    let add = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_add_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_add_builtin())?,
        None,
    );
    let subtract = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_subtract_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_subtract_builtin())?,
        None,
    );
    let value_of = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_value_of_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_value_of_builtin())?,
        None,
    );
    let from = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_from_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_from_builtin())?,
        None,
    );
    let compare = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_duration_compare_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_duration_compare_builtin())?,
        None,
    );

    Some(DurationFunctions {
        constructor,
        years_getter,
        months_getter,
        weeks_getter,
        days_getter,
        hours_getter,
        minutes_getter,
        seconds_getter,
        milliseconds_getter,
        microseconds_getter,
        nanoseconds_getter,
        sign_getter,
        blank_getter,
        to_string,
        to_json,
        to_locale_string,
        negated,
        abs,
        with,
        round,
        total,
        add,
        subtract,
        value_of,
        from,
        compare,
    })
}

pub(super) fn install_temporal_object_property(
    agent: &mut Agent,
    temporal_object: ObjectRef,
    duration_key: PropertyKey,
    functions: DurationFunctions,
) {
    define_builtin_data_property(
        agent,
        temporal_object,
        duration_key,
        Value::from_object_ref(functions.constructor),
        true,
        false,
        true,
    );
}

pub(super) fn install_constructor_properties(
    agent: &mut Agent,
    functions: DurationFunctions,
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

#[allow(
    clippy::too_many_lines,
    reason = "Duration prototype descriptors are kept inline as an ordered Temporal table"
)]
pub(super) fn install_prototype_properties(
    agent: &mut Agent,
    duration_prototype: ObjectRef,
    functions: DurationFunctions,
    properties: DurationPrototypeProperties,
) {
    define_builtin_data_property(
        agent,
        duration_prototype,
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        Value::from_object_ref(functions.constructor),
        true,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        properties.years_key,
        Some(functions.years_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        properties.months_key,
        Some(functions.months_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        properties.weeks_key,
        Some(functions.weeks_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        properties.days_key,
        Some(functions.days_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        properties.hours_key,
        Some(functions.hours_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        properties.minutes_key,
        Some(functions.minutes_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        properties.seconds_key,
        Some(functions.seconds_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        properties.milliseconds_key,
        Some(functions.milliseconds_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        properties.microseconds_key,
        Some(functions.microseconds_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        properties.nanoseconds_key,
        Some(functions.nanoseconds_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        properties.sign_key,
        Some(functions.sign_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        properties.blank_key,
        Some(functions.blank_getter),
        None,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        PropertyKey::from_atom(WellKnownAtom::toString.id()),
        Value::from_object_ref(functions.to_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        properties.to_json_key,
        Value::from_object_ref(functions.to_json),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        properties.to_locale_string_key,
        Value::from_object_ref(functions.to_locale_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        properties.negated_key,
        Value::from_object_ref(functions.negated),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        properties.abs_key,
        Value::from_object_ref(functions.abs),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        properties.with_key,
        Value::from_object_ref(functions.with),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        properties.round_key,
        Value::from_object_ref(functions.round),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        properties.total_key,
        Value::from_object_ref(functions.total),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        properties.add_key,
        Value::from_object_ref(functions.add),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        properties.subtract_key,
        Value::from_object_ref(functions.subtract),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        PropertyKey::from_atom(WellKnownAtom::valueOf.id()),
        Value::from_object_ref(functions.value_of),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        properties.to_string_tag_key,
        properties.prototype_tag,
        false,
        false,
        true,
    );
}
