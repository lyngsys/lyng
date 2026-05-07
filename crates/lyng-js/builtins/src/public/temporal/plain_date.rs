use super::{
    allocate_builtin_function_object, define_builtin_accessor_property,
    define_builtin_data_property, public_builtin_metadata,
};
use lyng_js_common::WellKnownAtom;
use lyng_js_env::Agent;
use lyng_js_types::{EnvironmentRef, ObjectRef, PropertyKey, RealmRef, ShapeId, Value};

#[derive(Clone, Copy)]
pub(super) struct PlainDateBootstrapContext {
    realm: RealmRef,
    global_env: EnvironmentRef,
    root_shape: ShapeId,
    function_prototype: ObjectRef,
    object_prototype: ObjectRef,
}

impl PlainDateBootstrapContext {
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
pub(super) struct PlainDateFunctions {
    constructor: ObjectRef,
    year_getter: ObjectRef,
    month_getter: ObjectRef,
    month_code_getter: ObjectRef,
    day_getter: ObjectRef,
    day_of_week_getter: ObjectRef,
    day_of_year_getter: ObjectRef,
    days_in_month_getter: ObjectRef,
    days_in_year_getter: ObjectRef,
    months_in_year_getter: ObjectRef,
    in_leap_year_getter: ObjectRef,
    days_in_week_getter: ObjectRef,
    week_of_year_getter: ObjectRef,
    year_of_week_getter: ObjectRef,
    era_getter: ObjectRef,
    era_year_getter: ObjectRef,
    calendar_id_getter: ObjectRef,
    to_string: ObjectRef,
    to_json: ObjectRef,
    to_locale_string: ObjectRef,
    value_of: ObjectRef,
    equals: ObjectRef,
    with: ObjectRef,
    with_calendar: ObjectRef,
    add: ObjectRef,
    subtract: ObjectRef,
    since: ObjectRef,
    until: ObjectRef,
    to_plain_date_time: ObjectRef,
    to_zoned_date_time: ObjectRef,
    to_plain_year_month: ObjectRef,
    to_plain_month_day: ObjectRef,
    from: ObjectRef,
    compare: ObjectRef,
}

#[derive(Clone, Copy)]
pub(super) struct PlainDatePrototypeProperties {
    pub(super) year_key: PropertyKey,
    pub(super) month_key: PropertyKey,
    pub(super) month_code_key: PropertyKey,
    pub(super) day_key: PropertyKey,
    pub(super) day_of_week_key: PropertyKey,
    pub(super) day_of_year_key: PropertyKey,
    pub(super) days_in_month_key: PropertyKey,
    pub(super) days_in_year_key: PropertyKey,
    pub(super) months_in_year_key: PropertyKey,
    pub(super) in_leap_year_key: PropertyKey,
    pub(super) days_in_week_key: PropertyKey,
    pub(super) week_of_year_key: PropertyKey,
    pub(super) year_of_week_key: PropertyKey,
    pub(super) era_key: PropertyKey,
    pub(super) era_year_key: PropertyKey,
    pub(super) calendar_id_key: PropertyKey,
    pub(super) to_json_key: PropertyKey,
    pub(super) to_locale_string_key: PropertyKey,
    pub(super) equals_key: PropertyKey,
    pub(super) with_key: PropertyKey,
    pub(super) with_calendar_key: PropertyKey,
    pub(super) add_key: PropertyKey,
    pub(super) subtract_key: PropertyKey,
    pub(super) since_key: PropertyKey,
    pub(super) until_key: PropertyKey,
    pub(super) to_plain_date_time_key: PropertyKey,
    pub(super) to_zoned_date_time_key: PropertyKey,
    pub(super) to_plain_year_month_key: PropertyKey,
    pub(super) to_plain_month_day_key: PropertyKey,
    pub(super) to_string_tag_key: PropertyKey,
    pub(super) prototype_tag: Value,
}

#[allow(
    clippy::too_many_lines,
    reason = "PlainDate builtin allocation follows the ordered Temporal function table"
)]
pub(super) fn allocate_functions(
    agent: &mut Agent,
    context: PlainDateBootstrapContext,
    plain_date_prototype: ObjectRef,
) -> Option<PlainDateFunctions> {
    let constructor = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_builtin())?,
        Some(plain_date_prototype),
    );
    let year_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_year_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_year_getter_builtin())?,
        None,
    );
    let month_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_month_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_month_getter_builtin())?,
        None,
    );
    let month_code_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_month_code_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_month_code_getter_builtin())?,
        None,
    );
    let day_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_day_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_day_getter_builtin())?,
        None,
    );
    let day_of_week_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_day_of_week_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_day_of_week_getter_builtin())?,
        None,
    );
    let day_of_year_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_day_of_year_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_day_of_year_getter_builtin())?,
        None,
    );
    let days_in_month_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_days_in_month_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_days_in_month_getter_builtin())?,
        None,
    );
    let days_in_year_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_days_in_year_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_days_in_year_getter_builtin())?,
        None,
    );
    let months_in_year_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_months_in_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_plain_date_months_in_year_getter_builtin(),
        )?,
        None,
    );
    let in_leap_year_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_in_leap_year_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_in_leap_year_getter_builtin())?,
        None,
    );
    let days_in_week_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_days_in_week_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_days_in_week_getter_builtin())?,
        None,
    );
    let week_of_year_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_week_of_year_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_week_of_year_getter_builtin())?,
        None,
    );
    let year_of_week_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_year_of_week_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_year_of_week_getter_builtin())?,
        None,
    );
    let era_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_era_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_era_getter_builtin())?,
        None,
    );
    let era_year_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_era_year_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_era_year_getter_builtin())?,
        None,
    );
    let calendar_id_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_calendar_id_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_calendar_id_getter_builtin())?,
        None,
    );
    let to_string = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_to_string_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_to_string_builtin())?,
        None,
    );
    let to_json = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_to_json_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_to_json_builtin())?,
        None,
    );
    let to_locale_string = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_to_locale_string_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_to_locale_string_builtin())?,
        None,
    );
    let value_of = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_value_of_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_value_of_builtin())?,
        None,
    );
    let equals = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_equals_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_equals_builtin())?,
        None,
    );
    let with = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_with_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_with_builtin())?,
        None,
    );
    let with_calendar = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_with_calendar_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_with_calendar_builtin())?,
        None,
    );
    let add = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_add_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_add_builtin())?,
        None,
    );
    let subtract = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_subtract_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_subtract_builtin())?,
        None,
    );
    let since = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_since_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_since_builtin())?,
        None,
    );
    let until = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_until_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_until_builtin())?,
        None,
    );
    let to_plain_date_time = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_to_plain_date_time_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_to_plain_date_time_builtin())?,
        None,
    );
    let to_zoned_date_time = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_to_zoned_date_time_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_to_zoned_date_time_builtin())?,
        None,
    );
    let to_plain_year_month = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_to_plain_year_month_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_to_plain_year_month_builtin())?,
        None,
    );
    let to_plain_month_day = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_to_plain_month_day_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_to_plain_month_day_builtin())?,
        None,
    );
    let from = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_from_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_from_builtin())?,
        None,
    );
    let compare = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_plain_date_compare_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_plain_date_compare_builtin())?,
        None,
    );

    Some(PlainDateFunctions {
        constructor,
        year_getter,
        month_getter,
        month_code_getter,
        day_getter,
        day_of_week_getter,
        day_of_year_getter,
        days_in_month_getter,
        days_in_year_getter,
        months_in_year_getter,
        in_leap_year_getter,
        days_in_week_getter,
        week_of_year_getter,
        year_of_week_getter,
        era_getter,
        era_year_getter,
        calendar_id_getter,
        to_string,
        to_json,
        to_locale_string,
        value_of,
        equals,
        with,
        with_calendar,
        add,
        subtract,
        since,
        until,
        to_plain_date_time,
        to_zoned_date_time,
        to_plain_year_month,
        to_plain_month_day,
        from,
        compare,
    })
}

pub(super) fn install_temporal_object_property(
    agent: &mut Agent,
    temporal_object: ObjectRef,
    plain_date_key: PropertyKey,
    functions: PlainDateFunctions,
) {
    define_builtin_data_property(
        agent,
        temporal_object,
        plain_date_key,
        Value::from_object_ref(functions.constructor),
        true,
        false,
        true,
    );
}

pub(super) fn install_constructor_properties(
    agent: &mut Agent,
    functions: PlainDateFunctions,
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
    reason = "PlainDate prototype descriptors are kept inline as an ordered Temporal table"
)]
pub(super) fn install_prototype_properties(
    agent: &mut Agent,
    plain_date_prototype: ObjectRef,
    functions: PlainDateFunctions,
    properties: PlainDatePrototypeProperties,
) {
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        Value::from_object_ref(functions.constructor),
        true,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        properties.year_key,
        Some(functions.year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        properties.month_key,
        Some(functions.month_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        properties.month_code_key,
        Some(functions.month_code_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        properties.day_key,
        Some(functions.day_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        properties.day_of_week_key,
        Some(functions.day_of_week_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        properties.day_of_year_key,
        Some(functions.day_of_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        properties.days_in_month_key,
        Some(functions.days_in_month_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        properties.days_in_year_key,
        Some(functions.days_in_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        properties.months_in_year_key,
        Some(functions.months_in_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        properties.in_leap_year_key,
        Some(functions.in_leap_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        properties.days_in_week_key,
        Some(functions.days_in_week_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        properties.week_of_year_key,
        Some(functions.week_of_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        properties.year_of_week_key,
        Some(functions.year_of_week_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        properties.era_key,
        Some(functions.era_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        properties.era_year_key,
        Some(functions.era_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        properties.calendar_id_key,
        Some(functions.calendar_id_getter),
        None,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        PropertyKey::from_atom(WellKnownAtom::toString.id()),
        Value::from_object_ref(functions.to_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        properties.to_json_key,
        Value::from_object_ref(functions.to_json),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        properties.to_locale_string_key,
        Value::from_object_ref(functions.to_locale_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        properties.equals_key,
        Value::from_object_ref(functions.equals),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        properties.with_key,
        Value::from_object_ref(functions.with),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        properties.with_calendar_key,
        Value::from_object_ref(functions.with_calendar),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        properties.add_key,
        Value::from_object_ref(functions.add),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        properties.subtract_key,
        Value::from_object_ref(functions.subtract),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        properties.since_key,
        Value::from_object_ref(functions.since),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        properties.until_key,
        Value::from_object_ref(functions.until),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        properties.to_plain_date_time_key,
        Value::from_object_ref(functions.to_plain_date_time),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        properties.to_zoned_date_time_key,
        Value::from_object_ref(functions.to_zoned_date_time),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        properties.to_plain_year_month_key,
        Value::from_object_ref(functions.to_plain_year_month),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        properties.to_plain_month_day_key,
        Value::from_object_ref(functions.to_plain_month_day),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        PropertyKey::from_atom(WellKnownAtom::valueOf.id()),
        Value::from_object_ref(functions.value_of),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        properties.to_string_tag_key,
        properties.prototype_tag,
        false,
        false,
        true,
    );
}
