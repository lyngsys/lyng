use super::{
    allocate_builtin_function_object, allocate_builtin_ordinary_object,
    define_builtin_accessor_property, define_builtin_data_property, public_builtin_metadata,
};
use crate::BuiltinEntryMetadata;
use lyng_js_common::WellKnownAtom;
use lyng_js_env::Agent;
use lyng_js_gc::AllocationLifetime;
use lyng_js_types::{
    BuiltinFunctionId, EnvironmentRef, ObjectRef, PropertyKey, RealmRef, ShapeId, Value,
    WellKnownSymbolId,
};

pub(super) fn install_temporal_public_objects(
    agent: &mut Agent,
    realm: RealmRef,
    global_env: EnvironmentRef,
    root_shape: ShapeId,
    function_prototype: ObjectRef,
    object_prototype: ObjectRef,
    global_object: ObjectRef,
) -> Option<()> {
    let instant_prototype_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "Temporal.Instant",
        None,
        AllocationLifetime::Default,
    ));
    let duration_prototype_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "Temporal.Duration",
        None,
        AllocationLifetime::Default,
    ));
    let plain_date_prototype_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "Temporal.PlainDate",
        None,
        AllocationLifetime::Default,
    ));
    let plain_time_prototype_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "Temporal.PlainTime",
        None,
        AllocationLifetime::Default,
    ));
    let plain_date_time_prototype_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "Temporal.PlainDateTime",
        None,
        AllocationLifetime::Default,
    ));
    let plain_year_month_prototype_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "Temporal.PlainYearMonth",
        None,
        AllocationLifetime::Default,
    ));
    let plain_month_day_prototype_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "Temporal.PlainMonthDay",
        None,
        AllocationLifetime::Default,
    ));
    let zoned_date_time_prototype_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "Temporal.ZonedDateTime",
        None,
        AllocationLifetime::Default,
    ));
    let temporal_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "Temporal",
        None,
        AllocationLifetime::Default,
    ));
    let temporal_now_tag = Value::from_string_ref(agent.alloc_runtime_string(
        "Temporal.Now",
        None,
        AllocationLifetime::Default,
    ));
    let to_string_tag_key = PropertyKey::from_symbol(
        agent
            .well_known_symbol(WellKnownSymbolId::ToStringTag)
            .expect("Symbol.toStringTag should exist"),
    );

    let temporal_object =
        allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype));
    let temporal_now_object =
        allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype));
    let instant_prototype =
        allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype));
    let duration_prototype =
        allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype));
    let plain_date_prototype =
        allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype));
    let plain_time_prototype =
        allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype));
    let plain_date_time_prototype =
        allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype));
    let plain_year_month_prototype =
        allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype));
    let plain_month_day_prototype =
        allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype));
    let zoned_date_time_prototype =
        allocate_builtin_ordinary_object(agent, root_shape, Some(object_prototype));
    let instant_constructor = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_instant_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_instant_builtin())?,
        Some(instant_prototype),
    );
    let instant_from = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_instant_from_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_instant_from_builtin())?,
        None,
    );
    let instant_from_epoch_nanoseconds = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_instant_from_epoch_nanoseconds_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_instant_from_epoch_nanoseconds_builtin(),
        )?,
        None,
    );
    let instant_from_epoch_milliseconds = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_instant_from_epoch_milliseconds_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_instant_from_epoch_milliseconds_builtin(),
        )?,
        None,
    );
    let instant_compare = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_instant_compare_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_instant_compare_builtin())?,
        None,
    );
    let instant_epoch_nanoseconds_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_instant_epoch_nanoseconds_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_instant_epoch_nanoseconds_getter_builtin(),
        )?,
        None,
    );
    let instant_epoch_milliseconds_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_instant_epoch_milliseconds_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_instant_epoch_milliseconds_getter_builtin(),
        )?,
        None,
    );
    let instant_epoch_seconds_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_instant_epoch_seconds_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_instant_epoch_seconds_getter_builtin())?,
        None,
    );
    let instant_to_string = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_instant_to_string_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_instant_to_string_builtin())?,
        None,
    );
    let instant_to_json = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_instant_to_json_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_instant_to_json_builtin())?,
        None,
    );
    let instant_to_locale_string = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_instant_to_locale_string_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_instant_to_locale_string_builtin())?,
        None,
    );
    let instant_value_of = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_instant_value_of_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_instant_value_of_builtin())?,
        None,
    );
    let instant_equals = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_instant_equals_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_instant_equals_builtin())?,
        None,
    );
    let instant_add = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_instant_add_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_instant_add_builtin())?,
        None,
    );
    let instant_subtract = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_instant_subtract_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_instant_subtract_builtin())?,
        None,
    );
    let instant_round = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_instant_round_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_instant_round_builtin())?,
        None,
    );
    let instant_since = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_instant_since_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_instant_since_builtin())?,
        None,
    );
    let instant_until = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_instant_until_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_instant_until_builtin())?,
        None,
    );
    let instant_to_zoned_date_time_iso = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_instant_to_zoned_date_time_iso_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_instant_to_zoned_date_time_iso_builtin(),
        )?,
        None,
    );
    let duration_constructor = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_builtin())?,
        Some(duration_prototype),
    );
    let duration_years_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_years_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_years_getter_builtin())?,
        None,
    );
    let duration_months_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_months_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_months_getter_builtin())?,
        None,
    );
    let duration_weeks_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_weeks_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_weeks_getter_builtin())?,
        None,
    );
    let duration_days_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_days_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_days_getter_builtin())?,
        None,
    );
    let duration_hours_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_hours_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_hours_getter_builtin())?,
        None,
    );
    let duration_minutes_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_minutes_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_minutes_getter_builtin())?,
        None,
    );
    let duration_seconds_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_seconds_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_seconds_getter_builtin())?,
        None,
    );
    let duration_milliseconds_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_milliseconds_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_milliseconds_getter_builtin())?,
        None,
    );
    let duration_microseconds_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_microseconds_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_microseconds_getter_builtin())?,
        None,
    );
    let duration_nanoseconds_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_nanoseconds_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_nanoseconds_getter_builtin())?,
        None,
    );
    let duration_sign_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_sign_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_sign_getter_builtin())?,
        None,
    );
    let duration_blank_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_blank_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_blank_getter_builtin())?,
        None,
    );
    let duration_to_string = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_to_string_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_to_string_builtin())?,
        None,
    );
    let duration_to_json = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_to_json_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_to_json_builtin())?,
        None,
    );
    let duration_to_locale_string = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_to_locale_string_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_to_locale_string_builtin())?,
        None,
    );
    let duration_negated = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_negated_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_negated_builtin())?,
        None,
    );
    let duration_abs = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_abs_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_abs_builtin())?,
        None,
    );
    let duration_with = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_with_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_with_builtin())?,
        None,
    );
    let duration_round = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_round_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_round_builtin())?,
        None,
    );
    let duration_total = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_total_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_total_builtin())?,
        None,
    );
    let duration_add = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_add_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_add_builtin())?,
        None,
    );
    let duration_subtract = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_subtract_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_subtract_builtin())?,
        None,
    );
    let duration_value_of = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_value_of_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_value_of_builtin())?,
        None,
    );
    let duration_from = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_from_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_from_builtin())?,
        None,
    );
    let duration_compare = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_duration_compare_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_duration_compare_builtin())?,
        None,
    );
    let plain_date_constructor = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_builtin())?,
        Some(plain_date_prototype),
    );
    let plain_date_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_year_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_year_getter_builtin())?,
        None,
    );
    let plain_date_month_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_month_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_month_getter_builtin())?,
        None,
    );
    let plain_date_month_code_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_month_code_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_month_code_getter_builtin())?,
        None,
    );
    let plain_date_day_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_day_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_day_getter_builtin())?,
        None,
    );
    let plain_date_day_of_week_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_day_of_week_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_day_of_week_getter_builtin(),
        )?,
        None,
    );
    let plain_date_day_of_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_day_of_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_day_of_year_getter_builtin(),
        )?,
        None,
    );
    let plain_date_days_in_month_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_days_in_month_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_days_in_month_getter_builtin(),
        )?,
        None,
    );
    let plain_date_days_in_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_days_in_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_days_in_year_getter_builtin(),
        )?,
        None,
    );
    let plain_date_months_in_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_months_in_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_months_in_year_getter_builtin(),
        )?,
        None,
    );
    let plain_date_in_leap_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_in_leap_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_in_leap_year_getter_builtin(),
        )?,
        None,
    );
    let plain_date_days_in_week_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_days_in_week_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_days_in_week_getter_builtin(),
        )?,
        None,
    );
    let plain_date_week_of_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_week_of_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_week_of_year_getter_builtin(),
        )?,
        None,
    );
    let plain_date_year_of_week_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_year_of_week_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_year_of_week_getter_builtin(),
        )?,
        None,
    );
    let plain_date_era_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_era_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_era_getter_builtin())?,
        None,
    );
    let plain_date_era_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_era_year_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_era_year_getter_builtin())?,
        None,
    );
    let plain_date_calendar_id_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_calendar_id_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_calendar_id_getter_builtin(),
        )?,
        None,
    );
    let plain_date_to_string = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_to_string_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_to_string_builtin())?,
        None,
    );
    let plain_date_to_json = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_to_json_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_to_json_builtin())?,
        None,
    );
    let plain_date_to_locale_string = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_to_locale_string_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_to_locale_string_builtin())?,
        None,
    );
    let plain_date_value_of = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_value_of_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_value_of_builtin())?,
        None,
    );
    let plain_date_equals = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_equals_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_equals_builtin())?,
        None,
    );
    let plain_date_with = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_with_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_with_builtin())?,
        None,
    );
    let plain_date_with_calendar = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_with_calendar_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_with_calendar_builtin())?,
        None,
    );
    let plain_date_add = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_add_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_add_builtin())?,
        None,
    );
    let plain_date_subtract = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_subtract_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_subtract_builtin())?,
        None,
    );
    let plain_date_since = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_since_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_since_builtin())?,
        None,
    );
    let plain_date_until = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_until_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_until_builtin())?,
        None,
    );
    let plain_date_to_plain_date_time = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_to_plain_date_time_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_to_plain_date_time_builtin(),
        )?,
        None,
    );
    let plain_date_to_zoned_date_time = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_to_zoned_date_time_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_to_zoned_date_time_builtin(),
        )?,
        None,
    );
    let plain_date_to_plain_year_month = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_to_plain_year_month_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_to_plain_year_month_builtin(),
        )?,
        None,
    );
    let plain_date_to_plain_month_day = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_to_plain_month_day_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_to_plain_month_day_builtin(),
        )?,
        None,
    );
    let plain_date_from = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_from_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_from_builtin())?,
        None,
    );
    let plain_date_compare = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_compare_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_compare_builtin())?,
        None,
    );
    let plain_time_constructor = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_time_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_time_builtin())?,
        Some(plain_time_prototype),
    );
    let plain_time_hour_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_time_hour_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_time_hour_getter_builtin())?,
        None,
    );
    let plain_time_minute_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_time_minute_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_time_minute_getter_builtin())?,
        None,
    );
    let plain_time_second_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_time_second_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_time_second_getter_builtin())?,
        None,
    );
    let plain_time_millisecond_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_time_millisecond_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_time_millisecond_getter_builtin(),
        )?,
        None,
    );
    let plain_time_microsecond_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_time_microsecond_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_time_microsecond_getter_builtin(),
        )?,
        None,
    );
    let plain_time_nanosecond_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_time_nanosecond_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_time_nanosecond_getter_builtin())?,
        None,
    );
    let plain_time_to_string = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_time_to_string_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_time_to_string_builtin())?,
        None,
    );
    let plain_time_to_json = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_time_to_json_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_time_to_json_builtin())?,
        None,
    );
    let plain_time_to_locale_string = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_time_to_locale_string_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_time_to_locale_string_builtin())?,
        None,
    );
    let plain_time_value_of = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_time_value_of_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_time_value_of_builtin())?,
        None,
    );
    let plain_time_equals = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_time_equals_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_time_equals_builtin())?,
        None,
    );
    let plain_time_with = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_time_with_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_time_with_builtin())?,
        None,
    );
    let plain_time_add = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_time_add_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_time_add_builtin())?,
        None,
    );
    let plain_time_subtract = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_time_subtract_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_time_subtract_builtin())?,
        None,
    );
    let plain_time_round = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_time_round_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_time_round_builtin())?,
        None,
    );
    let plain_time_since = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_time_since_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_time_since_builtin())?,
        None,
    );
    let plain_time_until = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_time_until_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_time_until_builtin())?,
        None,
    );
    let plain_time_to_plain_date_time = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_time_to_plain_date_time_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_time_to_plain_date_time_builtin(),
        )?,
        None,
    );
    let plain_time_from = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_time_from_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_time_from_builtin())?,
        None,
    );
    let plain_time_compare = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_time_compare_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_time_compare_builtin())?,
        None,
    );
    let plain_date_time_constructor = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_time_builtin())?,
        Some(plain_date_time_prototype),
    );
    let plain_date_time_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_year_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_time_year_getter_builtin())?,
        None,
    );
    let plain_date_time_month_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_month_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_time_month_getter_builtin())?,
        None,
    );
    let plain_date_time_month_code_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_month_code_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_month_code_getter_builtin(),
        )?,
        None,
    );
    let plain_date_time_day_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_day_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_time_day_getter_builtin())?,
        None,
    );
    let plain_date_time_day_of_week_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_day_of_week_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_day_of_week_getter_builtin(),
        )?,
        None,
    );
    let plain_date_time_day_of_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_day_of_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_day_of_year_getter_builtin(),
        )?,
        None,
    );
    let plain_date_time_days_in_month_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_days_in_month_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_days_in_month_getter_builtin(),
        )?,
        None,
    );
    let plain_date_time_days_in_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_days_in_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_days_in_year_getter_builtin(),
        )?,
        None,
    );
    let plain_date_time_months_in_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_months_in_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_months_in_year_getter_builtin(),
        )?,
        None,
    );
    let plain_date_time_in_leap_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_in_leap_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_in_leap_year_getter_builtin(),
        )?,
        None,
    );
    let plain_date_time_days_in_week_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_days_in_week_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_days_in_week_getter_builtin(),
        )?,
        None,
    );
    let plain_date_time_week_of_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_week_of_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_week_of_year_getter_builtin(),
        )?,
        None,
    );
    let plain_date_time_year_of_week_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_year_of_week_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_year_of_week_getter_builtin(),
        )?,
        None,
    );
    let plain_date_time_era_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_era_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_time_era_getter_builtin())?,
        None,
    );
    let plain_date_time_era_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_era_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_era_year_getter_builtin(),
        )?,
        None,
    );
    let plain_date_time_hour_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_hour_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_time_hour_getter_builtin())?,
        None,
    );
    let plain_date_time_minute_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_minute_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_minute_getter_builtin(),
        )?,
        None,
    );
    let plain_date_time_second_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_second_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_second_getter_builtin(),
        )?,
        None,
    );
    let plain_date_time_millisecond_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_millisecond_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_millisecond_getter_builtin(),
        )?,
        None,
    );
    let plain_date_time_microsecond_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_microsecond_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_microsecond_getter_builtin(),
        )?,
        None,
    );
    let plain_date_time_nanosecond_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_nanosecond_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_nanosecond_getter_builtin(),
        )?,
        None,
    );
    let plain_date_time_calendar_id_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_calendar_id_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_calendar_id_getter_builtin(),
        )?,
        None,
    );
    let plain_date_time_to_string = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_to_string_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_time_to_string_builtin())?,
        None,
    );
    let plain_date_time_to_json = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_to_json_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_time_to_json_builtin())?,
        None,
    );
    let plain_date_time_to_locale_string = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_to_locale_string_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_to_locale_string_builtin(),
        )?,
        None,
    );
    let plain_date_time_value_of = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_value_of_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_time_value_of_builtin())?,
        None,
    );
    let plain_date_time_equals = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_equals_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_time_equals_builtin())?,
        None,
    );
    let plain_date_time_with = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_with_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_time_with_builtin())?,
        None,
    );
    let plain_date_time_with_plain_time = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_with_plain_time_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_with_plain_time_builtin(),
        )?,
        None,
    );
    let plain_date_time_with_calendar = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_with_calendar_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_with_calendar_builtin(),
        )?,
        None,
    );
    let plain_date_time_add = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_add_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_time_add_builtin())?,
        None,
    );
    let plain_date_time_subtract = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_subtract_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_time_subtract_builtin())?,
        None,
    );
    let plain_date_time_round = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_round_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_time_round_builtin())?,
        None,
    );
    let plain_date_time_since = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_since_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_time_since_builtin())?,
        None,
    );
    let plain_date_time_until = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_until_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_time_until_builtin())?,
        None,
    );
    let plain_date_time_to_plain_date = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_to_plain_date_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_to_plain_date_builtin(),
        )?,
        None,
    );
    let plain_date_time_to_plain_time = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_to_plain_time_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_to_plain_time_builtin(),
        )?,
        None,
    );
    let plain_date_time_to_zoned_date_time = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_to_zoned_date_time_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_date_time_to_zoned_date_time_builtin(),
        )?,
        None,
    );
    let plain_date_time_from = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_from_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_time_from_builtin())?,
        None,
    );
    let plain_date_time_compare = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_date_time_compare_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_date_time_compare_builtin())?,
        None,
    );
    let plain_year_month_constructor = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_year_month_builtin())?,
        Some(plain_year_month_prototype),
    );
    let plain_year_month_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_year_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_year_month_year_getter_builtin())?,
        None,
    );
    let plain_year_month_month_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_month_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_year_month_month_getter_builtin(),
        )?,
        None,
    );
    let plain_year_month_month_code_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_month_code_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_year_month_month_code_getter_builtin(),
        )?,
        None,
    );
    let plain_year_month_days_in_month_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_days_in_month_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_year_month_days_in_month_getter_builtin(),
        )?,
        None,
    );
    let plain_year_month_days_in_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_days_in_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_year_month_days_in_year_getter_builtin(),
        )?,
        None,
    );
    let plain_year_month_months_in_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_months_in_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_year_month_months_in_year_getter_builtin(),
        )?,
        None,
    );
    let plain_year_month_in_leap_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_in_leap_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_year_month_in_leap_year_getter_builtin(),
        )?,
        None,
    );
    let plain_year_month_era_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_era_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_year_month_era_getter_builtin())?,
        None,
    );
    let plain_year_month_era_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_era_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_year_month_era_year_getter_builtin(),
        )?,
        None,
    );
    let plain_year_month_calendar_id_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_calendar_id_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_year_month_calendar_id_getter_builtin(),
        )?,
        None,
    );
    let plain_year_month_to_string = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_to_string_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_year_month_to_string_builtin())?,
        None,
    );
    let plain_year_month_to_json = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_to_json_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_year_month_to_json_builtin())?,
        None,
    );
    let plain_year_month_to_locale_string = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_to_locale_string_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_year_month_to_locale_string_builtin(),
        )?,
        None,
    );
    let plain_year_month_value_of = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_value_of_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_year_month_value_of_builtin())?,
        None,
    );
    let plain_year_month_equals = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_equals_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_year_month_equals_builtin())?,
        None,
    );
    let plain_year_month_with = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_with_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_year_month_with_builtin())?,
        None,
    );
    let plain_year_month_add = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_add_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_year_month_add_builtin())?,
        None,
    );
    let plain_year_month_subtract = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_subtract_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_year_month_subtract_builtin())?,
        None,
    );
    let plain_year_month_since = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_since_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_year_month_since_builtin())?,
        None,
    );
    let plain_year_month_until = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_until_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_year_month_until_builtin())?,
        None,
    );
    let plain_year_month_to_plain_date = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_to_plain_date_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_year_month_to_plain_date_builtin(),
        )?,
        None,
    );
    let plain_year_month_from = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_from_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_year_month_from_builtin())?,
        None,
    );
    let plain_year_month_compare = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_year_month_compare_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_year_month_compare_builtin())?,
        None,
    );
    let plain_month_day_constructor = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_month_day_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_month_day_builtin())?,
        Some(plain_month_day_prototype),
    );
    let plain_month_day_month_code_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_month_day_month_code_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_month_day_month_code_getter_builtin(),
        )?,
        None,
    );
    let plain_month_day_day_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_month_day_day_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_month_day_day_getter_builtin())?,
        None,
    );
    let plain_month_day_calendar_id_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_month_day_calendar_id_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_month_day_calendar_id_getter_builtin(),
        )?,
        None,
    );
    let plain_month_day_to_string = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_month_day_to_string_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_month_day_to_string_builtin())?,
        None,
    );
    let plain_month_day_to_json = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_month_day_to_json_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_month_day_to_json_builtin())?,
        None,
    );
    let plain_month_day_to_locale_string = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_month_day_to_locale_string_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_month_day_to_locale_string_builtin(),
        )?,
        None,
    );
    let plain_month_day_value_of = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_month_day_value_of_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_month_day_value_of_builtin())?,
        None,
    );
    let plain_month_day_equals = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_month_day_equals_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_month_day_equals_builtin())?,
        None,
    );
    let plain_month_day_with = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_month_day_with_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_month_day_with_builtin())?,
        None,
    );
    let plain_month_day_to_plain_date = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_month_day_to_plain_date_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_plain_month_day_to_plain_date_builtin(),
        )?,
        None,
    );
    let plain_month_day_from = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_plain_month_day_from_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_plain_month_day_from_builtin())?,
        None,
    );
    let zoned_date_time_constructor = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_zoned_date_time_builtin())?,
        Some(zoned_date_time_prototype),
    );
    let zoned_date_time_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_year_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_zoned_date_time_year_getter_builtin())?,
        None,
    );
    let zoned_date_time_month_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_month_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_zoned_date_time_month_getter_builtin())?,
        None,
    );
    let zoned_date_time_month_code_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_month_code_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_month_code_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_day_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_day_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_zoned_date_time_day_getter_builtin())?,
        None,
    );
    let zoned_date_time_day_of_week_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_day_of_week_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_day_of_week_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_day_of_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_day_of_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_day_of_year_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_days_in_month_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_days_in_month_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_days_in_month_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_days_in_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_days_in_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_days_in_year_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_months_in_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_months_in_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_months_in_year_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_in_leap_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_in_leap_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_in_leap_year_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_days_in_week_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_days_in_week_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_days_in_week_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_week_of_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_week_of_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_week_of_year_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_year_of_week_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_year_of_week_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_year_of_week_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_era_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_era_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_zoned_date_time_era_getter_builtin())?,
        None,
    );
    let zoned_date_time_era_year_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_era_year_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_era_year_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_hour_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_hour_getter_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_zoned_date_time_hour_getter_builtin())?,
        None,
    );
    let zoned_date_time_minute_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_minute_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_minute_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_second_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_second_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_second_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_millisecond_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_millisecond_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_millisecond_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_microsecond_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_microsecond_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_microsecond_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_nanosecond_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_nanosecond_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_nanosecond_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_epoch_nanoseconds_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_epoch_nanoseconds_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_epoch_nanoseconds_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_epoch_milliseconds_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_epoch_milliseconds_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_epoch_milliseconds_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_time_zone_id_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_time_zone_id_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_time_zone_id_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_calendar_id_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_calendar_id_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_calendar_id_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_offset_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_offset_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_offset_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_offset_nanoseconds_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_offset_nanoseconds_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_offset_nanoseconds_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_to_string = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_to_string_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_zoned_date_time_to_string_builtin())?,
        None,
    );
    let zoned_date_time_to_json = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_to_json_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_zoned_date_time_to_json_builtin())?,
        None,
    );
    let zoned_date_time_to_locale_string = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_to_locale_string_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_to_locale_string_builtin(),
        )?,
        None,
    );
    let zoned_date_time_value_of = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_value_of_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_zoned_date_time_value_of_builtin())?,
        None,
    );
    let zoned_date_time_equals = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_equals_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_zoned_date_time_equals_builtin())?,
        None,
    );
    let zoned_date_time_add = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_add_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_zoned_date_time_add_builtin())?,
        None,
    );
    let zoned_date_time_round = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_round_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_zoned_date_time_round_builtin())?,
        None,
    );
    let zoned_date_time_with = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_with_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_zoned_date_time_with_builtin())?,
        None,
    );
    let zoned_date_time_subtract = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_subtract_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_zoned_date_time_subtract_builtin())?,
        None,
    );
    let zoned_date_time_with_time_zone = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_with_time_zone_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_with_time_zone_builtin(),
        )?,
        None,
    );
    let zoned_date_time_with_calendar = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_with_calendar_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_with_calendar_builtin(),
        )?,
        None,
    );
    let zoned_date_time_with_plain_time = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_with_plain_time_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_with_plain_time_builtin(),
        )?,
        None,
    );
    let zoned_date_time_start_of_day = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_start_of_day_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_zoned_date_time_start_of_day_builtin())?,
        None,
    );
    let zoned_date_time_hours_in_day_getter = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_hours_in_day_getter_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_hours_in_day_getter_builtin(),
        )?,
        None,
    );
    let zoned_date_time_since = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_since_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_zoned_date_time_since_builtin())?,
        None,
    );
    let zoned_date_time_until = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_until_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_zoned_date_time_until_builtin())?,
        None,
    );
    let zoned_date_time_from = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_from_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_zoned_date_time_from_builtin())?,
        None,
    );
    let zoned_date_time_compare = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_compare_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_zoned_date_time_compare_builtin())?,
        None,
    );
    let zoned_date_time_to_instant = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_to_instant_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_zoned_date_time_to_instant_builtin())?,
        None,
    );
    let zoned_date_time_to_plain_date_time = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_to_plain_date_time_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_to_plain_date_time_builtin(),
        )?,
        None,
    );
    let zoned_date_time_to_plain_date = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_to_plain_date_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_to_plain_date_builtin(),
        )?,
        None,
    );
    let zoned_date_time_to_plain_time = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_zoned_date_time_to_plain_time_builtin(),
        public_builtin_metadata(
            lyng_js_types::js3_temporal_zoned_date_time_to_plain_time_builtin(),
        )?,
        None,
    );
    let now_instant = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_now_instant_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_now_instant_builtin())?,
        None,
    );
    let now_time_zone_id = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_now_time_zone_id_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_now_time_zone_id_builtin())?,
        None,
    );
    let now_plain_date_iso = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_now_plain_date_iso_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_now_plain_date_iso_builtin())?,
        None,
    );
    let now_plain_time_iso = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_now_plain_time_iso_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_now_plain_time_iso_builtin())?,
        None,
    );
    let now_plain_date_time_iso = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_now_plain_date_time_iso_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_now_plain_date_time_iso_builtin())?,
        None,
    );
    let now_zoned_date_time_iso = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::js3_temporal_now_zoned_date_time_iso_builtin(),
        public_builtin_metadata(lyng_js_types::js3_temporal_now_zoned_date_time_iso_builtin())?,
        None,
    );

    let (
        temporal_key,
        now_key,
        instant_key,
        duration_key,
        plain_date_key,
        plain_time_key,
        plain_date_time_key,
        plain_year_month_key,
        plain_month_day_key,
        zoned_date_time_key,
        from_key,
        from_epoch_nanoseconds_key,
        from_epoch_milliseconds_key,
        compare_key,
    ) = {
        let atoms = agent.atoms_mut();
        (
            PropertyKey::from_atom(atoms.intern_collectible("Temporal")),
            PropertyKey::from_atom(atoms.intern_collectible("Now")),
            PropertyKey::from_atom(atoms.intern_collectible("Instant")),
            PropertyKey::from_atom(atoms.intern_collectible("Duration")),
            PropertyKey::from_atom(atoms.intern_collectible("PlainDate")),
            PropertyKey::from_atom(atoms.intern_collectible("PlainTime")),
            PropertyKey::from_atom(atoms.intern_collectible("PlainDateTime")),
            PropertyKey::from_atom(atoms.intern_collectible("PlainYearMonth")),
            PropertyKey::from_atom(atoms.intern_collectible("PlainMonthDay")),
            PropertyKey::from_atom(atoms.intern_collectible("ZonedDateTime")),
            PropertyKey::from_atom(atoms.intern_collectible("from")),
            PropertyKey::from_atom(atoms.intern_collectible("fromEpochNanoseconds")),
            PropertyKey::from_atom(atoms.intern_collectible("fromEpochMilliseconds")),
            PropertyKey::from_atom(atoms.intern_collectible("compare")),
        )
    };
    let (
        instant_method_key,
        plain_date_iso_key,
        plain_time_iso_key,
        plain_date_time_iso_key,
        zoned_date_time_iso_key,
        time_zone_id_key,
        epoch_nanoseconds_key,
        epoch_milliseconds_key,
        epoch_seconds_key,
        offset_key,
        offset_nanoseconds_key,
        to_json_key,
        to_locale_string_key,
        equals_key,
    ) = {
        let atoms = agent.atoms_mut();
        (
            PropertyKey::from_atom(atoms.intern_collectible("instant")),
            PropertyKey::from_atom(atoms.intern_collectible("plainDateISO")),
            PropertyKey::from_atom(atoms.intern_collectible("plainTimeISO")),
            PropertyKey::from_atom(atoms.intern_collectible("plainDateTimeISO")),
            PropertyKey::from_atom(atoms.intern_collectible("zonedDateTimeISO")),
            PropertyKey::from_atom(atoms.intern_collectible("timeZoneId")),
            PropertyKey::from_atom(atoms.intern_collectible("epochNanoseconds")),
            PropertyKey::from_atom(atoms.intern_collectible("epochMilliseconds")),
            PropertyKey::from_atom(atoms.intern_collectible("epochSeconds")),
            PropertyKey::from_atom(atoms.intern_collectible("offset")),
            PropertyKey::from_atom(atoms.intern_collectible("offsetNanoseconds")),
            PropertyKey::from_atom(atoms.intern_collectible("toJSON")),
            PropertyKey::from_atom(atoms.intern_collectible("toLocaleString")),
            PropertyKey::from_atom(atoms.intern_collectible("equals")),
        )
    };
    let (
        years_key,
        months_key,
        weeks_key,
        days_key,
        hours_key,
        minutes_key,
        seconds_key,
        milliseconds_key,
        microseconds_key,
        nanoseconds_key,
        sign_key,
        blank_key,
        negated_key,
        abs_key,
    ) = {
        let atoms = agent.atoms_mut();
        (
            PropertyKey::from_atom(atoms.intern_collectible("years")),
            PropertyKey::from_atom(atoms.intern_collectible("months")),
            PropertyKey::from_atom(atoms.intern_collectible("weeks")),
            PropertyKey::from_atom(atoms.intern_collectible("days")),
            PropertyKey::from_atom(atoms.intern_collectible("hours")),
            PropertyKey::from_atom(atoms.intern_collectible("minutes")),
            PropertyKey::from_atom(atoms.intern_collectible("seconds")),
            PropertyKey::from_atom(atoms.intern_collectible("milliseconds")),
            PropertyKey::from_atom(atoms.intern_collectible("microseconds")),
            PropertyKey::from_atom(atoms.intern_collectible("nanoseconds")),
            PropertyKey::from_atom(atoms.intern_collectible("sign")),
            PropertyKey::from_atom(atoms.intern_collectible("blank")),
            PropertyKey::from_atom(atoms.intern_collectible("negated")),
            PropertyKey::from_atom(atoms.intern_collectible("abs")),
        )
    };
    let (
        year_key,
        month_key,
        month_code_key,
        day_key,
        day_of_week_key,
        days_in_week_key,
        week_of_year_key,
        year_of_week_key,
        era_key,
        era_year_key,
        day_of_year_key,
        days_in_month_key,
        days_in_year_key,
        months_in_year_key,
        in_leap_year_key,
        calendar_key,
        calendar_id_key,
    ) = {
        let atoms = agent.atoms_mut();
        (
            PropertyKey::from_atom(atoms.intern_collectible("year")),
            PropertyKey::from_atom(atoms.intern_collectible("month")),
            PropertyKey::from_atom(atoms.intern_collectible("monthCode")),
            PropertyKey::from_atom(atoms.intern_collectible("day")),
            PropertyKey::from_atom(atoms.intern_collectible("dayOfWeek")),
            PropertyKey::from_atom(atoms.intern_collectible("daysInWeek")),
            PropertyKey::from_atom(atoms.intern_collectible("weekOfYear")),
            PropertyKey::from_atom(atoms.intern_collectible("yearOfWeek")),
            PropertyKey::from_atom(atoms.intern_collectible("era")),
            PropertyKey::from_atom(atoms.intern_collectible("eraYear")),
            PropertyKey::from_atom(atoms.intern_collectible("dayOfYear")),
            PropertyKey::from_atom(atoms.intern_collectible("daysInMonth")),
            PropertyKey::from_atom(atoms.intern_collectible("daysInYear")),
            PropertyKey::from_atom(atoms.intern_collectible("monthsInYear")),
            PropertyKey::from_atom(atoms.intern_collectible("inLeapYear")),
            PropertyKey::from_atom(atoms.intern_collectible("calendar")),
            PropertyKey::from_atom(atoms.intern_collectible("calendarId")),
        )
    };
    let (hour_key, minute_key, second_key, millisecond_key, microsecond_key, nanosecond_key) = {
        let atoms = agent.atoms_mut();
        (
            PropertyKey::from_atom(atoms.intern_collectible("hour")),
            PropertyKey::from_atom(atoms.intern_collectible("minute")),
            PropertyKey::from_atom(atoms.intern_collectible("second")),
            PropertyKey::from_atom(atoms.intern_collectible("millisecond")),
            PropertyKey::from_atom(atoms.intern_collectible("microsecond")),
            PropertyKey::from_atom(atoms.intern_collectible("nanosecond")),
        )
    };
    let (
        to_instant_key,
        to_plain_date_time_key,
        to_plain_date_key,
        to_plain_time_key,
        to_plain_year_month_key,
        to_plain_month_day_key,
    ) = {
        let atoms = agent.atoms_mut();
        (
            PropertyKey::from_atom(atoms.intern_collectible("toInstant")),
            PropertyKey::from_atom(atoms.intern_collectible("toPlainDateTime")),
            PropertyKey::from_atom(atoms.intern_collectible("toPlainDate")),
            PropertyKey::from_atom(atoms.intern_collectible("toPlainTime")),
            PropertyKey::from_atom(atoms.intern_collectible("toPlainYearMonth")),
            PropertyKey::from_atom(atoms.intern_collectible("toPlainMonthDay")),
        )
    };
    let to_zoned_date_time_iso_key = {
        let atoms = agent.atoms_mut();
        PropertyKey::from_atom(atoms.intern_collectible("toZonedDateTimeISO"))
    };
    let to_zoned_date_time_key = {
        let atoms = agent.atoms_mut();
        PropertyKey::from_atom(atoms.intern_collectible("toZonedDateTime"))
    };
    let with_key = {
        let atoms = agent.atoms_mut();
        PropertyKey::from_atom(atoms.intern_collectible("with"))
    };
    let with_calendar_key = {
        let atoms = agent.atoms_mut();
        PropertyKey::from_atom(atoms.intern_collectible("withCalendar"))
    };
    let with_time_zone_key = {
        let atoms = agent.atoms_mut();
        PropertyKey::from_atom(atoms.intern_collectible("withTimeZone"))
    };
    let with_plain_time_key = {
        let atoms = agent.atoms_mut();
        PropertyKey::from_atom(atoms.intern_collectible("withPlainTime"))
    };
    let start_of_day_key = {
        let atoms = agent.atoms_mut();
        PropertyKey::from_atom(atoms.intern_collectible("startOfDay"))
    };
    let hours_in_day_key = {
        let atoms = agent.atoms_mut();
        PropertyKey::from_atom(atoms.intern_collectible("hoursInDay"))
    };
    let add_key = {
        let atoms = agent.atoms_mut();
        PropertyKey::from_atom(atoms.intern_collectible("add"))
    };
    let subtract_key = {
        let atoms = agent.atoms_mut();
        PropertyKey::from_atom(atoms.intern_collectible("subtract"))
    };
    let round_key = {
        let atoms = agent.atoms_mut();
        PropertyKey::from_atom(atoms.intern_collectible("round"))
    };
    let since_key = {
        let atoms = agent.atoms_mut();
        PropertyKey::from_atom(atoms.intern_collectible("since"))
    };
    let until_key = {
        let atoms = agent.atoms_mut();
        PropertyKey::from_atom(atoms.intern_collectible("until"))
    };
    let total_key = {
        let atoms = agent.atoms_mut();
        PropertyKey::from_atom(atoms.intern_collectible("total"))
    };

    define_builtin_data_property(
        agent,
        global_object,
        temporal_key,
        Value::from_object_ref(temporal_object),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        temporal_object,
        now_key,
        Value::from_object_ref(temporal_now_object),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        temporal_object,
        instant_key,
        Value::from_object_ref(instant_constructor),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        temporal_object,
        duration_key,
        Value::from_object_ref(duration_constructor),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        temporal_object,
        plain_date_key,
        Value::from_object_ref(plain_date_constructor),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        temporal_object,
        plain_time_key,
        Value::from_object_ref(plain_time_constructor),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        temporal_object,
        plain_date_time_key,
        Value::from_object_ref(plain_date_time_constructor),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        temporal_object,
        plain_year_month_key,
        Value::from_object_ref(plain_year_month_constructor),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        temporal_object,
        plain_month_day_key,
        Value::from_object_ref(plain_month_day_constructor),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        temporal_object,
        zoned_date_time_key,
        Value::from_object_ref(zoned_date_time_constructor),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        temporal_object,
        to_string_tag_key,
        temporal_tag,
        false,
        false,
        true,
    );

    define_builtin_data_property(
        agent,
        temporal_now_object,
        instant_method_key,
        Value::from_object_ref(now_instant),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        temporal_now_object,
        time_zone_id_key,
        Value::from_object_ref(now_time_zone_id),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        temporal_now_object,
        plain_date_iso_key,
        Value::from_object_ref(now_plain_date_iso),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        temporal_now_object,
        plain_time_iso_key,
        Value::from_object_ref(now_plain_time_iso),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        temporal_now_object,
        plain_date_time_iso_key,
        Value::from_object_ref(now_plain_date_time_iso),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        temporal_now_object,
        zoned_date_time_iso_key,
        Value::from_object_ref(now_zoned_date_time_iso),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        temporal_now_object,
        to_string_tag_key,
        temporal_now_tag,
        false,
        false,
        true,
    );

    define_builtin_data_property(
        agent,
        instant_constructor,
        from_key,
        Value::from_object_ref(instant_from),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_constructor,
        from_epoch_nanoseconds_key,
        Value::from_object_ref(instant_from_epoch_nanoseconds),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_constructor,
        from_epoch_milliseconds_key,
        Value::from_object_ref(instant_from_epoch_milliseconds),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_constructor,
        compare_key,
        Value::from_object_ref(instant_compare),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_constructor,
        from_key,
        Value::from_object_ref(duration_from),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_constructor,
        compare_key,
        Value::from_object_ref(duration_compare),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_constructor,
        from_key,
        Value::from_object_ref(plain_date_from),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_constructor,
        compare_key,
        Value::from_object_ref(plain_date_compare),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_constructor,
        from_key,
        Value::from_object_ref(plain_time_from),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_constructor,
        compare_key,
        Value::from_object_ref(plain_time_compare),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_time_constructor,
        from_key,
        Value::from_object_ref(plain_date_time_from),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_time_constructor,
        compare_key,
        Value::from_object_ref(plain_date_time_compare),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_year_month_constructor,
        from_key,
        Value::from_object_ref(plain_year_month_from),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_year_month_constructor,
        compare_key,
        Value::from_object_ref(plain_year_month_compare),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_month_day_constructor,
        from_key,
        Value::from_object_ref(plain_month_day_from),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_constructor,
        from_key,
        Value::from_object_ref(zoned_date_time_from),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_constructor,
        compare_key,
        Value::from_object_ref(zoned_date_time_compare),
        true,
        false,
        true,
    );

    define_builtin_data_property(
        agent,
        instant_prototype,
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        Value::from_object_ref(instant_constructor),
        true,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        instant_prototype,
        epoch_nanoseconds_key,
        Some(instant_epoch_nanoseconds_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        instant_prototype,
        epoch_milliseconds_key,
        Some(instant_epoch_milliseconds_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        instant_prototype,
        epoch_seconds_key,
        Some(instant_epoch_seconds_getter),
        None,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        PropertyKey::from_atom(WellKnownAtom::toString.id()),
        Value::from_object_ref(instant_to_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        to_json_key,
        Value::from_object_ref(instant_to_json),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        to_locale_string_key,
        Value::from_object_ref(instant_to_locale_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        equals_key,
        Value::from_object_ref(instant_equals),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        add_key,
        Value::from_object_ref(instant_add),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        subtract_key,
        Value::from_object_ref(instant_subtract),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        round_key,
        Value::from_object_ref(instant_round),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        since_key,
        Value::from_object_ref(instant_since),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        until_key,
        Value::from_object_ref(instant_until),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        to_zoned_date_time_iso_key,
        Value::from_object_ref(instant_to_zoned_date_time_iso),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        PropertyKey::from_atom(WellKnownAtom::valueOf.id()),
        Value::from_object_ref(instant_value_of),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        instant_prototype,
        to_string_tag_key,
        instant_prototype_tag,
        false,
        false,
        true,
    );

    define_builtin_data_property(
        agent,
        duration_prototype,
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        Value::from_object_ref(duration_constructor),
        true,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        years_key,
        Some(duration_years_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        months_key,
        Some(duration_months_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        weeks_key,
        Some(duration_weeks_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        days_key,
        Some(duration_days_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        hours_key,
        Some(duration_hours_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        minutes_key,
        Some(duration_minutes_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        seconds_key,
        Some(duration_seconds_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        milliseconds_key,
        Some(duration_milliseconds_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        microseconds_key,
        Some(duration_microseconds_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        nanoseconds_key,
        Some(duration_nanoseconds_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        sign_key,
        Some(duration_sign_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        duration_prototype,
        blank_key,
        Some(duration_blank_getter),
        None,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        PropertyKey::from_atom(WellKnownAtom::toString.id()),
        Value::from_object_ref(duration_to_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        to_json_key,
        Value::from_object_ref(duration_to_json),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        to_locale_string_key,
        Value::from_object_ref(duration_to_locale_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        negated_key,
        Value::from_object_ref(duration_negated),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        abs_key,
        Value::from_object_ref(duration_abs),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        with_key,
        Value::from_object_ref(duration_with),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        round_key,
        Value::from_object_ref(duration_round),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        total_key,
        Value::from_object_ref(duration_total),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        add_key,
        Value::from_object_ref(duration_add),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        subtract_key,
        Value::from_object_ref(duration_subtract),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        PropertyKey::from_atom(WellKnownAtom::valueOf.id()),
        Value::from_object_ref(duration_value_of),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        duration_prototype,
        to_string_tag_key,
        duration_prototype_tag,
        false,
        false,
        true,
    );

    define_builtin_data_property(
        agent,
        plain_date_prototype,
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        Value::from_object_ref(plain_date_constructor),
        true,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        year_key,
        Some(plain_date_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        month_key,
        Some(plain_date_month_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        month_code_key,
        Some(plain_date_month_code_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        day_key,
        Some(plain_date_day_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        day_of_week_key,
        Some(plain_date_day_of_week_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        day_of_year_key,
        Some(plain_date_day_of_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        days_in_month_key,
        Some(plain_date_days_in_month_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        days_in_year_key,
        Some(plain_date_days_in_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        months_in_year_key,
        Some(plain_date_months_in_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        in_leap_year_key,
        Some(plain_date_in_leap_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        days_in_week_key,
        Some(plain_date_days_in_week_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        week_of_year_key,
        Some(plain_date_week_of_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        year_of_week_key,
        Some(plain_date_year_of_week_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        era_key,
        Some(plain_date_era_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        era_year_key,
        Some(plain_date_era_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_prototype,
        calendar_id_key,
        Some(plain_date_calendar_id_getter),
        None,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        PropertyKey::from_atom(WellKnownAtom::toString.id()),
        Value::from_object_ref(plain_date_to_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        to_json_key,
        Value::from_object_ref(plain_date_to_json),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        to_locale_string_key,
        Value::from_object_ref(plain_date_to_locale_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        equals_key,
        Value::from_object_ref(plain_date_equals),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        with_key,
        Value::from_object_ref(plain_date_with),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        with_calendar_key,
        Value::from_object_ref(plain_date_with_calendar),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        add_key,
        Value::from_object_ref(plain_date_add),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        subtract_key,
        Value::from_object_ref(plain_date_subtract),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        since_key,
        Value::from_object_ref(plain_date_since),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        until_key,
        Value::from_object_ref(plain_date_until),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        to_plain_date_time_key,
        Value::from_object_ref(plain_date_to_plain_date_time),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        to_zoned_date_time_key,
        Value::from_object_ref(plain_date_to_zoned_date_time),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        to_plain_year_month_key,
        Value::from_object_ref(plain_date_to_plain_year_month),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        to_plain_month_day_key,
        Value::from_object_ref(plain_date_to_plain_month_day),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        PropertyKey::from_atom(WellKnownAtom::valueOf.id()),
        Value::from_object_ref(plain_date_value_of),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_prototype,
        to_string_tag_key,
        plain_date_prototype_tag,
        false,
        false,
        true,
    );

    define_builtin_data_property(
        agent,
        plain_time_prototype,
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        Value::from_object_ref(plain_time_constructor),
        true,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_time_prototype,
        hour_key,
        Some(plain_time_hour_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_time_prototype,
        minute_key,
        Some(plain_time_minute_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_time_prototype,
        second_key,
        Some(plain_time_second_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_time_prototype,
        millisecond_key,
        Some(plain_time_millisecond_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_time_prototype,
        microsecond_key,
        Some(plain_time_microsecond_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_time_prototype,
        nanosecond_key,
        Some(plain_time_nanosecond_getter),
        None,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        PropertyKey::from_atom(WellKnownAtom::toString.id()),
        Value::from_object_ref(plain_time_to_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        to_json_key,
        Value::from_object_ref(plain_time_to_json),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        to_locale_string_key,
        Value::from_object_ref(plain_time_to_locale_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        equals_key,
        Value::from_object_ref(plain_time_equals),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        with_key,
        Value::from_object_ref(plain_time_with),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        add_key,
        Value::from_object_ref(plain_time_add),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        subtract_key,
        Value::from_object_ref(plain_time_subtract),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        round_key,
        Value::from_object_ref(plain_time_round),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        since_key,
        Value::from_object_ref(plain_time_since),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        until_key,
        Value::from_object_ref(plain_time_until),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        to_plain_date_time_key,
        Value::from_object_ref(plain_time_to_plain_date_time),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        PropertyKey::from_atom(WellKnownAtom::valueOf.id()),
        Value::from_object_ref(plain_time_value_of),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_time_prototype,
        to_string_tag_key,
        plain_time_prototype_tag,
        false,
        false,
        true,
    );

    define_builtin_data_property(
        agent,
        plain_date_time_prototype,
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        Value::from_object_ref(plain_date_time_constructor),
        true,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        year_key,
        Some(plain_date_time_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        month_key,
        Some(plain_date_time_month_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        month_code_key,
        Some(plain_date_time_month_code_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        day_key,
        Some(plain_date_time_day_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        day_of_week_key,
        Some(plain_date_time_day_of_week_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        day_of_year_key,
        Some(plain_date_time_day_of_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        days_in_month_key,
        Some(plain_date_time_days_in_month_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        days_in_year_key,
        Some(plain_date_time_days_in_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        months_in_year_key,
        Some(plain_date_time_months_in_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        in_leap_year_key,
        Some(plain_date_time_in_leap_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        days_in_week_key,
        Some(plain_date_time_days_in_week_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        week_of_year_key,
        Some(plain_date_time_week_of_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        year_of_week_key,
        Some(plain_date_time_year_of_week_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        era_key,
        Some(plain_date_time_era_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        era_year_key,
        Some(plain_date_time_era_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        hour_key,
        Some(plain_date_time_hour_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        minute_key,
        Some(plain_date_time_minute_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        second_key,
        Some(plain_date_time_second_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        millisecond_key,
        Some(plain_date_time_millisecond_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        microsecond_key,
        Some(plain_date_time_microsecond_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        nanosecond_key,
        Some(plain_date_time_nanosecond_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_date_time_prototype,
        calendar_id_key,
        Some(plain_date_time_calendar_id_getter),
        None,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_time_prototype,
        PropertyKey::from_atom(WellKnownAtom::toString.id()),
        Value::from_object_ref(plain_date_time_to_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_time_prototype,
        to_json_key,
        Value::from_object_ref(plain_date_time_to_json),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_time_prototype,
        to_locale_string_key,
        Value::from_object_ref(plain_date_time_to_locale_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_time_prototype,
        equals_key,
        Value::from_object_ref(plain_date_time_equals),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_time_prototype,
        with_key,
        Value::from_object_ref(plain_date_time_with),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_time_prototype,
        with_plain_time_key,
        Value::from_object_ref(plain_date_time_with_plain_time),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_time_prototype,
        with_calendar_key,
        Value::from_object_ref(plain_date_time_with_calendar),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_time_prototype,
        add_key,
        Value::from_object_ref(plain_date_time_add),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_time_prototype,
        subtract_key,
        Value::from_object_ref(plain_date_time_subtract),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_time_prototype,
        round_key,
        Value::from_object_ref(plain_date_time_round),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_time_prototype,
        since_key,
        Value::from_object_ref(plain_date_time_since),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_time_prototype,
        until_key,
        Value::from_object_ref(plain_date_time_until),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_time_prototype,
        to_plain_date_key,
        Value::from_object_ref(plain_date_time_to_plain_date),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_time_prototype,
        to_plain_time_key,
        Value::from_object_ref(plain_date_time_to_plain_time),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_time_prototype,
        to_zoned_date_time_key,
        Value::from_object_ref(plain_date_time_to_zoned_date_time),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_time_prototype,
        PropertyKey::from_atom(WellKnownAtom::valueOf.id()),
        Value::from_object_ref(plain_date_time_value_of),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_date_time_prototype,
        to_string_tag_key,
        plain_date_time_prototype_tag,
        false,
        false,
        true,
    );

    define_builtin_data_property(
        agent,
        plain_year_month_prototype,
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        Value::from_object_ref(plain_year_month_constructor),
        true,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_year_month_prototype,
        year_key,
        Some(plain_year_month_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_year_month_prototype,
        month_key,
        Some(plain_year_month_month_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_year_month_prototype,
        month_code_key,
        Some(plain_year_month_month_code_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_year_month_prototype,
        days_in_month_key,
        Some(plain_year_month_days_in_month_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_year_month_prototype,
        days_in_year_key,
        Some(plain_year_month_days_in_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_year_month_prototype,
        months_in_year_key,
        Some(plain_year_month_months_in_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_year_month_prototype,
        in_leap_year_key,
        Some(plain_year_month_in_leap_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_year_month_prototype,
        era_key,
        Some(plain_year_month_era_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_year_month_prototype,
        era_year_key,
        Some(plain_year_month_era_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_year_month_prototype,
        calendar_id_key,
        Some(plain_year_month_calendar_id_getter),
        None,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_year_month_prototype,
        PropertyKey::from_atom(WellKnownAtom::toString.id()),
        Value::from_object_ref(plain_year_month_to_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_year_month_prototype,
        to_json_key,
        Value::from_object_ref(plain_year_month_to_json),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_year_month_prototype,
        to_locale_string_key,
        Value::from_object_ref(plain_year_month_to_locale_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_year_month_prototype,
        equals_key,
        Value::from_object_ref(plain_year_month_equals),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_year_month_prototype,
        with_key,
        Value::from_object_ref(plain_year_month_with),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_year_month_prototype,
        add_key,
        Value::from_object_ref(plain_year_month_add),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_year_month_prototype,
        subtract_key,
        Value::from_object_ref(plain_year_month_subtract),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_year_month_prototype,
        since_key,
        Value::from_object_ref(plain_year_month_since),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_year_month_prototype,
        until_key,
        Value::from_object_ref(plain_year_month_until),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_year_month_prototype,
        to_plain_date_key,
        Value::from_object_ref(plain_year_month_to_plain_date),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_year_month_prototype,
        PropertyKey::from_atom(WellKnownAtom::valueOf.id()),
        Value::from_object_ref(plain_year_month_value_of),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_year_month_prototype,
        to_string_tag_key,
        plain_year_month_prototype_tag,
        false,
        false,
        true,
    );

    define_builtin_data_property(
        agent,
        plain_month_day_prototype,
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        Value::from_object_ref(plain_month_day_constructor),
        true,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_month_day_prototype,
        month_code_key,
        Some(plain_month_day_month_code_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_month_day_prototype,
        day_key,
        Some(plain_month_day_day_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        plain_month_day_prototype,
        calendar_id_key,
        Some(plain_month_day_calendar_id_getter),
        None,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_month_day_prototype,
        PropertyKey::from_atom(WellKnownAtom::toString.id()),
        Value::from_object_ref(plain_month_day_to_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_month_day_prototype,
        to_json_key,
        Value::from_object_ref(plain_month_day_to_json),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_month_day_prototype,
        to_locale_string_key,
        Value::from_object_ref(plain_month_day_to_locale_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_month_day_prototype,
        equals_key,
        Value::from_object_ref(plain_month_day_equals),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_month_day_prototype,
        with_key,
        Value::from_object_ref(plain_month_day_with),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_month_day_prototype,
        to_plain_date_key,
        Value::from_object_ref(plain_month_day_to_plain_date),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_month_day_prototype,
        PropertyKey::from_atom(WellKnownAtom::valueOf.id()),
        Value::from_object_ref(plain_month_day_value_of),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        plain_month_day_prototype,
        to_string_tag_key,
        plain_month_day_prototype_tag,
        false,
        false,
        true,
    );

    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        PropertyKey::from_atom(WellKnownAtom::constructor.id()),
        Value::from_object_ref(zoned_date_time_constructor),
        true,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        year_key,
        Some(zoned_date_time_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        month_key,
        Some(zoned_date_time_month_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        month_code_key,
        Some(zoned_date_time_month_code_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        day_key,
        Some(zoned_date_time_day_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        day_of_week_key,
        Some(zoned_date_time_day_of_week_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        day_of_year_key,
        Some(zoned_date_time_day_of_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        days_in_month_key,
        Some(zoned_date_time_days_in_month_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        days_in_year_key,
        Some(zoned_date_time_days_in_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        months_in_year_key,
        Some(zoned_date_time_months_in_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        in_leap_year_key,
        Some(zoned_date_time_in_leap_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        days_in_week_key,
        Some(zoned_date_time_days_in_week_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        week_of_year_key,
        Some(zoned_date_time_week_of_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        year_of_week_key,
        Some(zoned_date_time_year_of_week_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        era_key,
        Some(zoned_date_time_era_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        era_year_key,
        Some(zoned_date_time_era_year_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        hour_key,
        Some(zoned_date_time_hour_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        minute_key,
        Some(zoned_date_time_minute_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        second_key,
        Some(zoned_date_time_second_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        millisecond_key,
        Some(zoned_date_time_millisecond_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        microsecond_key,
        Some(zoned_date_time_microsecond_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        nanosecond_key,
        Some(zoned_date_time_nanosecond_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        epoch_nanoseconds_key,
        Some(zoned_date_time_epoch_nanoseconds_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        epoch_milliseconds_key,
        Some(zoned_date_time_epoch_milliseconds_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        time_zone_id_key,
        Some(zoned_date_time_time_zone_id_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        calendar_key,
        Some(zoned_date_time_calendar_id_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        calendar_id_key,
        Some(zoned_date_time_calendar_id_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        offset_key,
        Some(zoned_date_time_offset_getter),
        None,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        offset_nanoseconds_key,
        Some(zoned_date_time_offset_nanoseconds_getter),
        None,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        PropertyKey::from_atom(WellKnownAtom::toString.id()),
        Value::from_object_ref(zoned_date_time_to_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        to_json_key,
        Value::from_object_ref(zoned_date_time_to_json),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        to_locale_string_key,
        Value::from_object_ref(zoned_date_time_to_locale_string),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        equals_key,
        Value::from_object_ref(zoned_date_time_equals),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        add_key,
        Value::from_object_ref(zoned_date_time_add),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        round_key,
        Value::from_object_ref(zoned_date_time_round),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        with_key,
        Value::from_object_ref(zoned_date_time_with),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        subtract_key,
        Value::from_object_ref(zoned_date_time_subtract),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        with_time_zone_key,
        Value::from_object_ref(zoned_date_time_with_time_zone),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        with_calendar_key,
        Value::from_object_ref(zoned_date_time_with_calendar),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        with_plain_time_key,
        Value::from_object_ref(zoned_date_time_with_plain_time),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        start_of_day_key,
        Value::from_object_ref(zoned_date_time_start_of_day),
        true,
        false,
        true,
    );
    define_builtin_accessor_property(
        agent,
        zoned_date_time_prototype,
        hours_in_day_key,
        Some(zoned_date_time_hours_in_day_getter),
        None,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        since_key,
        Value::from_object_ref(zoned_date_time_since),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        until_key,
        Value::from_object_ref(zoned_date_time_until),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        to_instant_key,
        Value::from_object_ref(zoned_date_time_to_instant),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        to_plain_date_time_key,
        Value::from_object_ref(zoned_date_time_to_plain_date_time),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        to_plain_date_key,
        Value::from_object_ref(zoned_date_time_to_plain_date),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        to_plain_time_key,
        Value::from_object_ref(zoned_date_time_to_plain_time),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        PropertyKey::from_atom(WellKnownAtom::valueOf.id()),
        Value::from_object_ref(zoned_date_time_value_of),
        true,
        false,
        true,
    );
    define_builtin_data_property(
        agent,
        zoned_date_time_prototype,
        to_string_tag_key,
        zoned_date_time_prototype_tag,
        false,
        false,
        true,
    );

    Some(())
}

pub(super) fn temporal_builtin_metadata(entry: BuiltinFunctionId) -> Option<BuiltinEntryMetadata> {
    if entry == lyng_js_types::js3_temporal_instant_builtin() {
        return Some(BuiltinEntryMetadata::new("Instant", 1, true, true));
    }
    if entry == lyng_js_types::js3_temporal_now_instant_builtin() {
        return Some(BuiltinEntryMetadata::new("instant", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_now_time_zone_id_builtin() {
        return Some(BuiltinEntryMetadata::new("timeZoneId", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_now_plain_date_iso_builtin() {
        return Some(BuiltinEntryMetadata::new("plainDateISO", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_now_plain_time_iso_builtin() {
        return Some(BuiltinEntryMetadata::new("plainTimeISO", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_now_plain_date_time_iso_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "plainDateTimeISO",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_now_zoned_date_time_iso_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "zonedDateTimeISO",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_instant_from_builtin() {
        return Some(BuiltinEntryMetadata::new("from", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_instant_from_epoch_nanoseconds_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "fromEpochNanoseconds",
            1,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_instant_from_epoch_milliseconds_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "fromEpochMilliseconds",
            1,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_instant_compare_builtin() {
        return Some(BuiltinEntryMetadata::new("compare", 2, false, false));
    }
    if entry == lyng_js_types::js3_temporal_instant_epoch_nanoseconds_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get epochNanoseconds",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_instant_epoch_milliseconds_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get epochMilliseconds",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_instant_epoch_seconds_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get epochSeconds",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_instant_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_instant_to_json_builtin() {
        return Some(BuiltinEntryMetadata::new("toJSON", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_instant_to_locale_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toLocaleString", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_instant_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_instant_equals_builtin() {
        return Some(BuiltinEntryMetadata::new("equals", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_instant_add_builtin() {
        return Some(BuiltinEntryMetadata::new("add", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_instant_subtract_builtin() {
        return Some(BuiltinEntryMetadata::new("subtract", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_instant_round_builtin() {
        return Some(BuiltinEntryMetadata::new("round", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_instant_since_builtin() {
        return Some(BuiltinEntryMetadata::new("since", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_instant_until_builtin() {
        return Some(BuiltinEntryMetadata::new("until", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_instant_to_zoned_date_time_iso_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "toZonedDateTimeISO",
            1,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_duration_builtin() {
        return Some(BuiltinEntryMetadata::new("Duration", 0, true, true));
    }
    if entry == lyng_js_types::js3_temporal_duration_years_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get years", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_duration_months_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get months", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_duration_weeks_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get weeks", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_duration_days_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get days", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_duration_hours_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get hours", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_duration_minutes_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get minutes", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_duration_seconds_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get seconds", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_duration_milliseconds_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get milliseconds",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_duration_microseconds_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get microseconds",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_duration_nanoseconds_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get nanoseconds",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_duration_sign_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get sign", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_duration_blank_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get blank", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_duration_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_duration_to_json_builtin() {
        return Some(BuiltinEntryMetadata::new("toJSON", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_duration_to_locale_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toLocaleString", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_duration_negated_builtin() {
        return Some(BuiltinEntryMetadata::new("negated", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_duration_abs_builtin() {
        return Some(BuiltinEntryMetadata::new("abs", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_duration_with_builtin() {
        return Some(BuiltinEntryMetadata::new("with", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_duration_round_builtin() {
        return Some(BuiltinEntryMetadata::new("round", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_duration_total_builtin() {
        return Some(BuiltinEntryMetadata::new("total", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_duration_add_builtin() {
        return Some(BuiltinEntryMetadata::new("add", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_duration_subtract_builtin() {
        return Some(BuiltinEntryMetadata::new("subtract", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_duration_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_duration_from_builtin() {
        return Some(BuiltinEntryMetadata::new("from", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_duration_compare_builtin() {
        return Some(BuiltinEntryMetadata::new("compare", 2, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_builtin() {
        return Some(BuiltinEntryMetadata::new("PlainDate", 3, true, true));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get year", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_month_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get month", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_month_code_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get monthCode", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_day_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get day", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_day_of_week_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get dayOfWeek", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_day_of_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get dayOfYear", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_days_in_month_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get daysInMonth",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_days_in_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get daysInYear", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_months_in_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get monthsInYear",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_in_leap_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get inLeapYear", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_days_in_week_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get daysInWeek", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_week_of_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get weekOfYear", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_year_of_week_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get yearOfWeek", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_era_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get era", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_era_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get eraYear", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_calendar_id_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get calendarId", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_to_json_builtin() {
        return Some(BuiltinEntryMetadata::new("toJSON", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_to_locale_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toLocaleString", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_equals_builtin() {
        return Some(BuiltinEntryMetadata::new("equals", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_with_builtin() {
        return Some(BuiltinEntryMetadata::new("with", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_with_calendar_builtin() {
        return Some(BuiltinEntryMetadata::new("withCalendar", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_add_builtin() {
        return Some(BuiltinEntryMetadata::new("add", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_subtract_builtin() {
        return Some(BuiltinEntryMetadata::new("subtract", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_since_builtin() {
        return Some(BuiltinEntryMetadata::new("since", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_until_builtin() {
        return Some(BuiltinEntryMetadata::new("until", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_to_plain_date_time_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "toPlainDateTime",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_to_zoned_date_time_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "toZonedDateTime",
            1,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_to_plain_year_month_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "toPlainYearMonth",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_to_plain_month_day_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "toPlainMonthDay",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_from_builtin() {
        return Some(BuiltinEntryMetadata::new("from", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_compare_builtin() {
        return Some(BuiltinEntryMetadata::new("compare", 2, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_time_builtin() {
        return Some(BuiltinEntryMetadata::new("PlainTime", 0, true, true));
    }
    if entry == lyng_js_types::js3_temporal_plain_time_hour_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get hour", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_time_minute_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get minute", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_time_second_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get second", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_time_millisecond_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get millisecond",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_plain_time_microsecond_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get microsecond",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_plain_time_nanosecond_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get nanosecond", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_time_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_time_to_json_builtin() {
        return Some(BuiltinEntryMetadata::new("toJSON", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_time_to_locale_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toLocaleString", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_time_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_time_equals_builtin() {
        return Some(BuiltinEntryMetadata::new("equals", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_time_with_builtin() {
        return Some(BuiltinEntryMetadata::new("with", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_time_add_builtin() {
        return Some(BuiltinEntryMetadata::new("add", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_time_subtract_builtin() {
        return Some(BuiltinEntryMetadata::new("subtract", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_time_round_builtin() {
        return Some(BuiltinEntryMetadata::new("round", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_time_since_builtin() {
        return Some(BuiltinEntryMetadata::new("since", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_time_until_builtin() {
        return Some(BuiltinEntryMetadata::new("until", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_time_to_plain_date_time_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "toPlainDateTime",
            1,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_plain_time_from_builtin() {
        return Some(BuiltinEntryMetadata::new("from", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_time_compare_builtin() {
        return Some(BuiltinEntryMetadata::new("compare", 2, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_builtin() {
        return Some(BuiltinEntryMetadata::new("PlainDateTime", 3, true, true));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get year", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_month_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get month", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_month_code_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get monthCode", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_day_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get day", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_day_of_week_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get dayOfWeek", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_day_of_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get dayOfYear", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_days_in_month_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get daysInMonth",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_days_in_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get daysInYear", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_months_in_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get monthsInYear",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_in_leap_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get inLeapYear", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_days_in_week_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get daysInWeek", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_week_of_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get weekOfYear", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_year_of_week_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get yearOfWeek", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_era_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get era", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_era_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get eraYear", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_hour_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get hour", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_minute_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get minute", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_second_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get second", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_millisecond_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get millisecond",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_microsecond_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get microsecond",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_nanosecond_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get nanosecond", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_calendar_id_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get calendarId", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_to_json_builtin() {
        return Some(BuiltinEntryMetadata::new("toJSON", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_to_locale_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toLocaleString", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_equals_builtin() {
        return Some(BuiltinEntryMetadata::new("equals", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_with_builtin() {
        return Some(BuiltinEntryMetadata::new("with", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_with_plain_time_builtin() {
        return Some(BuiltinEntryMetadata::new("withPlainTime", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_with_calendar_builtin() {
        return Some(BuiltinEntryMetadata::new("withCalendar", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_add_builtin() {
        return Some(BuiltinEntryMetadata::new("add", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_subtract_builtin() {
        return Some(BuiltinEntryMetadata::new("subtract", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_round_builtin() {
        return Some(BuiltinEntryMetadata::new("round", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_since_builtin() {
        return Some(BuiltinEntryMetadata::new("since", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_until_builtin() {
        return Some(BuiltinEntryMetadata::new("until", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_to_plain_date_builtin() {
        return Some(BuiltinEntryMetadata::new("toPlainDate", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_to_plain_time_builtin() {
        return Some(BuiltinEntryMetadata::new("toPlainTime", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_to_zoned_date_time_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "toZonedDateTime",
            1,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_from_builtin() {
        return Some(BuiltinEntryMetadata::new("from", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_date_time_compare_builtin() {
        return Some(BuiltinEntryMetadata::new("compare", 2, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_builtin() {
        return Some(BuiltinEntryMetadata::new("PlainYearMonth", 2, true, true));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get year", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_month_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get month", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_month_code_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get monthCode", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_days_in_month_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get daysInMonth",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_days_in_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get daysInYear", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_months_in_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get monthsInYear",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_in_leap_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get inLeapYear", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_era_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get era", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_era_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get eraYear", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_calendar_id_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get calendarId", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_to_json_builtin() {
        return Some(BuiltinEntryMetadata::new("toJSON", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_to_locale_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toLocaleString", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_equals_builtin() {
        return Some(BuiltinEntryMetadata::new("equals", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_with_builtin() {
        return Some(BuiltinEntryMetadata::new("with", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_add_builtin() {
        return Some(BuiltinEntryMetadata::new("add", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_subtract_builtin() {
        return Some(BuiltinEntryMetadata::new("subtract", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_since_builtin() {
        return Some(BuiltinEntryMetadata::new("since", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_until_builtin() {
        return Some(BuiltinEntryMetadata::new("until", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_to_plain_date_builtin() {
        return Some(BuiltinEntryMetadata::new("toPlainDate", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_from_builtin() {
        return Some(BuiltinEntryMetadata::new("from", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_year_month_compare_builtin() {
        return Some(BuiltinEntryMetadata::new("compare", 2, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_builtin() {
        return Some(BuiltinEntryMetadata::new("PlainMonthDay", 2, true, true));
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_month_code_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get monthCode", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_day_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get day", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_calendar_id_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get calendarId", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_to_json_builtin() {
        return Some(BuiltinEntryMetadata::new("toJSON", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_to_locale_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toLocaleString", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_equals_builtin() {
        return Some(BuiltinEntryMetadata::new("equals", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_with_builtin() {
        return Some(BuiltinEntryMetadata::new("with", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_to_plain_date_builtin() {
        return Some(BuiltinEntryMetadata::new("toPlainDate", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_plain_month_day_from_builtin() {
        return Some(BuiltinEntryMetadata::new("from", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_builtin() {
        return Some(BuiltinEntryMetadata::new("ZonedDateTime", 2, true, true));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get year", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_month_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get month", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_month_code_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get monthCode", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_day_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get day", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_day_of_week_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get dayOfWeek", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_day_of_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get dayOfYear", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_days_in_month_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get daysInMonth",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_days_in_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get daysInYear", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_months_in_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get monthsInYear",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_in_leap_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get inLeapYear", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_days_in_week_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get daysInWeek", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_week_of_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get weekOfYear", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_year_of_week_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get yearOfWeek", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_era_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get era", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_era_year_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get eraYear", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_hour_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get hour", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_minute_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get minute", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_second_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get second", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_millisecond_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get millisecond",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_microsecond_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get microsecond",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_nanosecond_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get nanosecond", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_epoch_nanoseconds_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get epochNanoseconds",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_epoch_milliseconds_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get epochMilliseconds",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_time_zone_id_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get timeZoneId", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_calendar_id_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get calendarId", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_offset_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get offset", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_offset_nanoseconds_getter_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "get offsetNanoseconds",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_to_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toString", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_to_json_builtin() {
        return Some(BuiltinEntryMetadata::new("toJSON", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_to_locale_string_builtin() {
        return Some(BuiltinEntryMetadata::new("toLocaleString", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_value_of_builtin() {
        return Some(BuiltinEntryMetadata::new("valueOf", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_equals_builtin() {
        return Some(BuiltinEntryMetadata::new("equals", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_add_builtin() {
        return Some(BuiltinEntryMetadata::new("add", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_round_builtin() {
        return Some(BuiltinEntryMetadata::new("round", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_with_builtin() {
        return Some(BuiltinEntryMetadata::new("with", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_subtract_builtin() {
        return Some(BuiltinEntryMetadata::new("subtract", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_with_time_zone_builtin() {
        return Some(BuiltinEntryMetadata::new("withTimeZone", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_with_calendar_builtin() {
        return Some(BuiltinEntryMetadata::new("withCalendar", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_with_plain_time_builtin() {
        return Some(BuiltinEntryMetadata::new("withPlainTime", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_start_of_day_builtin() {
        return Some(BuiltinEntryMetadata::new("startOfDay", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_hours_in_day_getter_builtin() {
        return Some(BuiltinEntryMetadata::new("get hoursInDay", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_since_builtin() {
        return Some(BuiltinEntryMetadata::new("since", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_until_builtin() {
        return Some(BuiltinEntryMetadata::new("until", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_from_builtin() {
        return Some(BuiltinEntryMetadata::new("from", 1, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_compare_builtin() {
        return Some(BuiltinEntryMetadata::new("compare", 2, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_to_instant_builtin() {
        return Some(BuiltinEntryMetadata::new("toInstant", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_to_plain_date_time_builtin() {
        return Some(BuiltinEntryMetadata::new(
            "toPlainDateTime",
            0,
            false,
            false,
        ));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_to_plain_date_builtin() {
        return Some(BuiltinEntryMetadata::new("toPlainDate", 0, false, false));
    }
    if entry == lyng_js_types::js3_temporal_zoned_date_time_to_plain_time_builtin() {
        return Some(BuiltinEntryMetadata::new("toPlainTime", 0, false, false));
    }
    None
}
