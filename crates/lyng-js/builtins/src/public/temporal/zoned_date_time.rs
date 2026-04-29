use super::{
    allocate_builtin_function_object, define_builtin_accessor_property,
    define_builtin_data_property, public_builtin_metadata,
};
use lyng_js_common::WellKnownAtom;
use lyng_js_env::Agent;
use lyng_js_types::{EnvironmentRef, ObjectRef, PropertyKey, RealmRef, ShapeId, Value};

#[derive(Clone, Copy)]
pub(super) struct ZonedDateTimeBootstrapContext {
    realm: RealmRef,
    global_env: EnvironmentRef,
    root_shape: ShapeId,
    function_prototype: ObjectRef,
    object_prototype: ObjectRef,
}

impl ZonedDateTimeBootstrapContext {
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
pub(super) struct ZonedDateTimeFunctions {
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
    hour_getter: ObjectRef,
    minute_getter: ObjectRef,
    second_getter: ObjectRef,
    millisecond_getter: ObjectRef,
    microsecond_getter: ObjectRef,
    nanosecond_getter: ObjectRef,
    epoch_nanoseconds_getter: ObjectRef,
    epoch_milliseconds_getter: ObjectRef,
    time_zone_id_getter: ObjectRef,
    calendar_id_getter: ObjectRef,
    offset_getter: ObjectRef,
    offset_nanoseconds_getter: ObjectRef,
    to_string: ObjectRef,
    to_json: ObjectRef,
    to_locale_string: ObjectRef,
    value_of: ObjectRef,
    equals: ObjectRef,
    add: ObjectRef,
    round: ObjectRef,
    with: ObjectRef,
    subtract: ObjectRef,
    with_time_zone: ObjectRef,
    with_calendar: ObjectRef,
    with_plain_time: ObjectRef,
    start_of_day: ObjectRef,
    get_time_zone_transition: ObjectRef,
    hours_in_day_getter: ObjectRef,
    since: ObjectRef,
    until: ObjectRef,
    from: ObjectRef,
    compare: ObjectRef,
    to_instant: ObjectRef,
    to_plain_date_time: ObjectRef,
    to_plain_date: ObjectRef,
    to_plain_time: ObjectRef,
}

#[derive(Clone, Copy)]
pub(super) struct ZonedDateTimePrototypeProperties {
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
    pub(super) hour_key: PropertyKey,
    pub(super) minute_key: PropertyKey,
    pub(super) second_key: PropertyKey,
    pub(super) millisecond_key: PropertyKey,
    pub(super) microsecond_key: PropertyKey,
    pub(super) nanosecond_key: PropertyKey,
    pub(super) epoch_nanoseconds_key: PropertyKey,
    pub(super) epoch_milliseconds_key: PropertyKey,
    pub(super) time_zone_id_key: PropertyKey,
    pub(super) calendar_key: PropertyKey,
    pub(super) calendar_id_key: PropertyKey,
    pub(super) offset_key: PropertyKey,
    pub(super) offset_nanoseconds_key: PropertyKey,
    pub(super) to_json_key: PropertyKey,
    pub(super) to_locale_string_key: PropertyKey,
    pub(super) equals_key: PropertyKey,
    pub(super) add_key: PropertyKey,
    pub(super) round_key: PropertyKey,
    pub(super) with_key: PropertyKey,
    pub(super) subtract_key: PropertyKey,
    pub(super) with_time_zone_key: PropertyKey,
    pub(super) with_calendar_key: PropertyKey,
    pub(super) with_plain_time_key: PropertyKey,
    pub(super) start_of_day_key: PropertyKey,
    pub(super) get_time_zone_transition_key: PropertyKey,
    pub(super) hours_in_day_key: PropertyKey,
    pub(super) since_key: PropertyKey,
    pub(super) until_key: PropertyKey,
    pub(super) to_instant_key: PropertyKey,
    pub(super) to_plain_date_time_key: PropertyKey,
    pub(super) to_plain_date_key: PropertyKey,
    pub(super) to_plain_time_key: PropertyKey,
    pub(super) to_string_tag_key: PropertyKey,
    pub(super) prototype_tag: Value,
}

pub(super) fn allocate_functions(
    agent: &mut Agent,
    context: ZonedDateTimeBootstrapContext,
    zoned_date_time_prototype: ObjectRef,
) -> Option<ZonedDateTimeFunctions> {
    let constructor = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_builtin())?,
        Some(zoned_date_time_prototype),
    );
    let year_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_year_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_year_getter_builtin())?,
        None,
    );
    let month_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_month_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_month_getter_builtin())?,
        None,
    );
    let month_code_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_month_code_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_month_code_getter_builtin(),
        )?,
        None,
    );
    let day_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_day_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_day_getter_builtin())?,
        None,
    );
    let day_of_week_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_day_of_week_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_day_of_week_getter_builtin(),
        )?,
        None,
    );
    let day_of_year_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_day_of_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_day_of_year_getter_builtin(),
        )?,
        None,
    );
    let days_in_month_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_days_in_month_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_days_in_month_getter_builtin(),
        )?,
        None,
    );
    let days_in_year_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_days_in_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_days_in_year_getter_builtin(),
        )?,
        None,
    );
    let months_in_year_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_months_in_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_months_in_year_getter_builtin(),
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
        lyng_js_types::temporal_zoned_date_time_in_leap_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_in_leap_year_getter_builtin(),
        )?,
        None,
    );
    let days_in_week_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_days_in_week_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_days_in_week_getter_builtin(),
        )?,
        None,
    );
    let week_of_year_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_week_of_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_week_of_year_getter_builtin(),
        )?,
        None,
    );
    let year_of_week_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_year_of_week_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_year_of_week_getter_builtin(),
        )?,
        None,
    );
    let era_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_era_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_era_getter_builtin())?,
        None,
    );
    let era_year_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_era_year_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_era_year_getter_builtin())?,
        None,
    );
    let hour_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_hour_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_hour_getter_builtin())?,
        None,
    );
    let minute_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_minute_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_minute_getter_builtin())?,
        None,
    );
    let second_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_second_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_second_getter_builtin())?,
        None,
    );
    let millisecond_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_millisecond_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_millisecond_getter_builtin(),
        )?,
        None,
    );
    let microsecond_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_microsecond_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_microsecond_getter_builtin(),
        )?,
        None,
    );
    let nanosecond_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_nanosecond_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_nanosecond_getter_builtin(),
        )?,
        None,
    );
    let epoch_nanoseconds_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_epoch_nanoseconds_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_epoch_nanoseconds_getter_builtin(),
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
        lyng_js_types::temporal_zoned_date_time_epoch_milliseconds_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_epoch_milliseconds_getter_builtin(),
        )?,
        None,
    );
    let time_zone_id_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_time_zone_id_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_time_zone_id_getter_builtin(),
        )?,
        None,
    );
    let calendar_id_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_calendar_id_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_calendar_id_getter_builtin(),
        )?,
        None,
    );
    let offset_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_offset_getter_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_offset_getter_builtin())?,
        None,
    );
    let offset_nanoseconds_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_offset_nanoseconds_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_offset_nanoseconds_getter_builtin(),
        )?,
        None,
    );
    let to_string = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_to_string_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_to_string_builtin())?,
        None,
    );
    let to_json = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_to_json_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_to_json_builtin())?,
        None,
    );
    let to_locale_string = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_to_locale_string_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_to_locale_string_builtin(),
        )?,
        None,
    );
    let value_of = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_value_of_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_value_of_builtin())?,
        None,
    );
    let equals = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_equals_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_equals_builtin())?,
        None,
    );
    let add = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_add_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_add_builtin())?,
        None,
    );
    let round = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_round_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_round_builtin())?,
        None,
    );
    let with = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_with_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_with_builtin())?,
        None,
    );
    let subtract = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_subtract_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_subtract_builtin())?,
        None,
    );
    let with_time_zone = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_with_time_zone_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_with_time_zone_builtin())?,
        None,
    );
    let with_calendar = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_with_calendar_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_with_calendar_builtin())?,
        None,
    );
    let with_plain_time = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_with_plain_time_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_with_plain_time_builtin())?,
        None,
    );
    let start_of_day = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_start_of_day_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_start_of_day_builtin())?,
        None,
    );
    let get_time_zone_transition = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_get_time_zone_transition_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_get_time_zone_transition_builtin(),
        )?,
        None,
    );
    let hours_in_day_getter = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_hours_in_day_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_hours_in_day_getter_builtin(),
        )?,
        None,
    );
    let since = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_since_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_since_builtin())?,
        None,
    );
    let until = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_until_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_until_builtin())?,
        None,
    );
    let from = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_from_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_from_builtin())?,
        None,
    );
    let compare = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_compare_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_compare_builtin())?,
        None,
    );
    let to_instant = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_to_instant_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_to_instant_builtin())?,
        None,
    );
    let to_plain_date_time = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_to_plain_date_time_builtin(),
        public_builtin_metadata(
            lyng_js_types::temporal_zoned_date_time_to_plain_date_time_builtin(),
        )?,
        None,
    );
    let to_plain_date = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_to_plain_date_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_to_plain_date_builtin())?,
        None,
    );
    let to_plain_time = allocate_builtin_function_object(
        agent,
        context.realm,
        context.global_env,
        context.root_shape,
        context.function_prototype,
        context.object_prototype,
        lyng_js_types::temporal_zoned_date_time_to_plain_time_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_zoned_date_time_to_plain_time_builtin())?,
        None,
    );

    Some(ZonedDateTimeFunctions {
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
        hour_getter,
        minute_getter,
        second_getter,
        millisecond_getter,
        microsecond_getter,
        nanosecond_getter,
        epoch_nanoseconds_getter,
        epoch_milliseconds_getter,
        time_zone_id_getter,
        calendar_id_getter,
        offset_getter,
        offset_nanoseconds_getter,
        to_string,
        to_json,
        to_locale_string,
        value_of,
        equals,
        add,
        round,
        with,
        subtract,
        with_time_zone,
        with_calendar,
        with_plain_time,
        start_of_day,
        get_time_zone_transition,
        hours_in_day_getter,
        since,
        until,
        from,
        compare,
        to_instant,
        to_plain_date_time,
        to_plain_date,
        to_plain_time,
    })
}

pub(super) fn install_temporal_object_property(
    agent: &mut Agent,
    temporal_object: ObjectRef,
    zoned_date_time_key: PropertyKey,
    functions: ZonedDateTimeFunctions,
) {
    define_builtin_data_property(
        agent,
        temporal_object,
        zoned_date_time_key,
        Value::from_object_ref(functions.constructor),
        true,
        false,
        true,
    );
}

pub(super) fn install_constructor_properties(
    agent: &mut Agent,
    functions: ZonedDateTimeFunctions,
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
    zoned_date_time_prototype: ObjectRef,
    functions: ZonedDateTimeFunctions,
    properties: ZonedDateTimePrototypeProperties,
) {
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        Value::from_object_ref(functions.constructor),
        true,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.year_key,
        Some(functions.year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.month_key,
        Some(functions.month_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.month_code_key,
        Some(functions.month_code_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.day_key,
        Some(functions.day_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.day_of_week_key,
        Some(functions.day_of_week_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.day_of_year_key,
        Some(functions.day_of_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.days_in_month_key,
        Some(functions.days_in_month_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.days_in_year_key,
        Some(functions.days_in_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.months_in_year_key,
        Some(functions.months_in_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.in_leap_year_key,
        Some(functions.in_leap_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.days_in_week_key,
        Some(functions.days_in_week_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.week_of_year_key,
        Some(functions.week_of_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.year_of_week_key,
        Some(functions.year_of_week_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.era_key,
        Some(functions.era_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.era_year_key,
        Some(functions.era_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.hour_key,
        Some(functions.hour_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.minute_key,
        Some(functions.minute_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.second_key,
        Some(functions.second_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.millisecond_key,
        Some(functions.millisecond_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.microsecond_key,
        Some(functions.microsecond_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.nanosecond_key,
        Some(functions.nanosecond_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.epoch_nanoseconds_key,
        Some(functions.epoch_nanoseconds_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.epoch_milliseconds_key,
        Some(functions.epoch_milliseconds_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.time_zone_id_key,
        Some(functions.time_zone_id_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.calendar_key,
        Some(functions.calendar_id_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.calendar_id_key,
        Some(functions.calendar_id_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.offset_key,
        Some(functions.offset_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.offset_nanoseconds_key,
        Some(functions.offset_nanoseconds_getter),
        None,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        PropertyKey::from_atom(WellKnownAtom::toString.id()),
        Value::from_object_ref(functions.to_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        properties.to_json_key,
        Value::from_object_ref(functions.to_json),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        properties.to_locale_string_key,
        Value::from_object_ref(functions.to_locale_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        properties.equals_key,
        Value::from_object_ref(functions.equals),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        properties.add_key,
        Value::from_object_ref(functions.add),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        properties.round_key,
        Value::from_object_ref(functions.round),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        properties.with_key,
        Value::from_object_ref(functions.with),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        properties.subtract_key,
        Value::from_object_ref(functions.subtract),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        properties.with_time_zone_key,
        Value::from_object_ref(functions.with_time_zone),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        properties.with_calendar_key,
        Value::from_object_ref(functions.with_calendar),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        properties.with_plain_time_key,
        Value::from_object_ref(functions.with_plain_time),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        properties.start_of_day_key,
        Value::from_object_ref(functions.start_of_day),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        properties.get_time_zone_transition_key,
        Value::from_object_ref(functions.get_time_zone_transition),
        true,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        properties.hours_in_day_key,
        Some(functions.hours_in_day_getter),
        None,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        properties.since_key,
        Value::from_object_ref(functions.since),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        properties.until_key,
        Value::from_object_ref(functions.until),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        properties.to_instant_key,
        Value::from_object_ref(functions.to_instant),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        properties.to_plain_date_time_key,
        Value::from_object_ref(functions.to_plain_date_time),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        properties.to_plain_date_key,
        Value::from_object_ref(functions.to_plain_date),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        properties.to_plain_time_key,
        Value::from_object_ref(functions.to_plain_time),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        PropertyKey::from_atom(WellKnownAtom::valueOf.id()),
        Value::from_object_ref(functions.value_of),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        properties.to_string_tag_key,
        properties.prototype_tag,
        false,
        false,
        true,
    );
}
