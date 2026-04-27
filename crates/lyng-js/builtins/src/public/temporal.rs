use super::{
    allocate_builtin_function_object, allocate_builtin_ordinary_object,
    define_builtin_accessor_property, define_builtin_data_property, public_builtin_metadata,
};
use lyng_js_env::Agent;
use lyng_js_gc::AllocationLifetime;
use lyng_js_types::{
    EnvironmentRef, ObjectRef, PropertyKey, RealmRef, ShapeId, Value, WellKnownSymbolId,
};
mod duration;
mod instant;
mod plain_date;
mod plain_date_time;
mod plain_month_day;
mod plain_time;
mod plain_year_month;
mod zoned_date_time;

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
    let instant_functions = instant::allocate_functions(
        agent,
        instant::InstantBootstrapContext::new(
            realm,
            global_env,
            root_shape,
            function_prototype,
            object_prototype,
        ),
        instant_prototype,
    )?;
    let duration_functions = duration::allocate_functions(
        agent,
        duration::DurationBootstrapContext::new(
            realm,
            global_env,
            root_shape,
            function_prototype,
            object_prototype,
        ),
        duration_prototype,
    )?;
    let plain_date_functions = plain_date::allocate_functions(
        agent,
        plain_date::PlainDateBootstrapContext::new(
            realm,
            global_env,
            root_shape,
            function_prototype,
            object_prototype,
        ),
        plain_date_prototype,
    )?;
    let plain_time_functions = plain_time::allocate_functions(
        agent,
        plain_time::PlainTimeBootstrapContext::new(
            realm,
            global_env,
            root_shape,
            function_prototype,
            object_prototype,
        ),
        plain_time_prototype,
    )?;
    let plain_date_time_functions = plain_date_time::allocate_functions(
        agent,
        plain_date_time::PlainDateTimeBootstrapContext::new(
            realm,
            global_env,
            root_shape,
            function_prototype,
            object_prototype,
        ),
        plain_date_time_prototype,
    )?;
    let plain_year_month_functions = plain_year_month::allocate_functions(
        agent,
        plain_year_month::PlainYearMonthBootstrapContext::new(
            realm,
            global_env,
            root_shape,
            function_prototype,
            object_prototype,
        ),
        plain_year_month_prototype,
    )?;
    let plain_month_day_functions = plain_month_day::allocate_functions(
        agent,
        plain_month_day::PlainMonthDayBootstrapContext::new(
            realm,
            global_env,
            root_shape,
            function_prototype,
            object_prototype,
        ),
        plain_month_day_prototype,
    )?;
    let zoned_date_time_functions = zoned_date_time::allocate_functions(
        agent,
        zoned_date_time::ZonedDateTimeBootstrapContext::new(
            realm,
            global_env,
            root_shape,
            function_prototype,
            object_prototype,
        ),
        zoned_date_time_prototype,
    )?;
    let now_instant = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::temporal_now_instant_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_now_instant_builtin())?,
        None,
    );
    let now_time_zone_id = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::temporal_now_time_zone_id_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_now_time_zone_id_builtin())?,
        None,
    );
    let now_plain_date_iso = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::temporal_now_plain_date_iso_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_now_plain_date_iso_builtin())?,
        None,
    );
    let now_plain_time_iso = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::temporal_now_plain_time_iso_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_now_plain_time_iso_builtin())?,
        None,
    );
    let now_plain_date_time_iso = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::temporal_now_plain_date_time_iso_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_now_plain_date_time_iso_builtin())?,
        None,
    );
    let now_zoned_date_time_iso = allocate_builtin_function_object(
        agent,
        realm,
        global_env,
        root_shape,
        function_prototype,
        object_prototype,
        lyng_js_types::temporal_now_zoned_date_time_iso_builtin(),
        public_builtin_metadata(lyng_js_types::temporal_now_zoned_date_time_iso_builtin())?,
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
    instant::install_temporal_object_property(
        agent,
        temporal_object,
        instant_key,
        instant_functions,
    );
    duration::install_temporal_object_property(
        agent,
        temporal_object,
        duration_key,
        duration_functions,
    );
    plain_date::install_temporal_object_property(
        agent,
        temporal_object,
        plain_date_key,
        plain_date_functions,
    );
    plain_time::install_temporal_object_property(
        agent,
        temporal_object,
        plain_time_key,
        plain_time_functions,
    );
    plain_date_time::install_temporal_object_property(
        agent,
        temporal_object,
        plain_date_time_key,
        plain_date_time_functions,
    );
    plain_year_month::install_temporal_object_property(
        agent,
        temporal_object,
        plain_year_month_key,
        plain_year_month_functions,
    );
    plain_month_day::install_temporal_object_property(
        agent,
        temporal_object,
        plain_month_day_key,
        plain_month_day_functions,
    );
    zoned_date_time::install_temporal_object_property(
        agent,
        temporal_object,
        zoned_date_time_key,
        zoned_date_time_functions,
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

    instant::install_constructor_properties(
        agent,
        instant_functions,
        from_key,
        from_epoch_nanoseconds_key,
        from_epoch_milliseconds_key,
        compare_key,
    );
    duration::install_constructor_properties(agent, duration_functions, from_key, compare_key);
    plain_date::install_constructor_properties(agent, plain_date_functions, from_key, compare_key);
    plain_time::install_constructor_properties(agent, plain_time_functions, from_key, compare_key);
    plain_date_time::install_constructor_properties(
        agent,
        plain_date_time_functions,
        from_key,
        compare_key,
    );
    plain_year_month::install_constructor_properties(
        agent,
        plain_year_month_functions,
        from_key,
        compare_key,
    );
    plain_month_day::install_constructor_properties(agent, plain_month_day_functions, from_key);
    zoned_date_time::install_constructor_properties(
        agent,
        zoned_date_time_functions,
        from_key,
        compare_key,
    );
    instant::install_prototype_properties(
        agent,
        instant_prototype,
        instant_functions,
        instant::InstantPrototypeProperties {
            epoch_nanoseconds_key,
            epoch_milliseconds_key,
            epoch_seconds_key,
            to_json_key,
            to_locale_string_key,
            equals_key,
            add_key,
            subtract_key,
            round_key,
            since_key,
            until_key,
            to_zoned_date_time_iso_key,
            to_string_tag_key,
            prototype_tag: instant_prototype_tag,
        },
    );
    duration::install_prototype_properties(
        agent,
        duration_prototype,
        duration_functions,
        duration::DurationPrototypeProperties {
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
            to_json_key,
            to_locale_string_key,
            negated_key,
            abs_key,
            with_key,
            round_key,
            total_key,
            add_key,
            subtract_key,
            to_string_tag_key,
            prototype_tag: duration_prototype_tag,
        },
    );
    plain_date::install_prototype_properties(
        agent,
        plain_date_prototype,
        plain_date_functions,
        plain_date::PlainDatePrototypeProperties {
            year_key,
            month_key,
            month_code_key,
            day_key,
            day_of_week_key,
            day_of_year_key,
            days_in_month_key,
            days_in_year_key,
            months_in_year_key,
            in_leap_year_key,
            days_in_week_key,
            week_of_year_key,
            year_of_week_key,
            era_key,
            era_year_key,
            calendar_id_key,
            to_json_key,
            to_locale_string_key,
            equals_key,
            with_key,
            with_calendar_key,
            add_key,
            subtract_key,
            since_key,
            until_key,
            to_plain_date_time_key,
            to_zoned_date_time_key,
            to_plain_year_month_key,
            to_plain_month_day_key,
            to_string_tag_key,
            prototype_tag: plain_date_prototype_tag,
        },
    );

    plain_time::install_prototype_properties(
        agent,
        plain_time_prototype,
        plain_time_functions,
        plain_time::PlainTimePrototypeProperties {
            hour_key,
            minute_key,
            second_key,
            millisecond_key,
            microsecond_key,
            nanosecond_key,
            to_json_key,
            to_locale_string_key,
            equals_key,
            with_key,
            add_key,
            subtract_key,
            round_key,
            since_key,
            until_key,
            to_plain_date_time_key,
            to_string_tag_key,
            prototype_tag: plain_time_prototype_tag,
        },
    );
    plain_date_time::install_prototype_properties(
        agent,
        plain_date_time_prototype,
        plain_date_time_functions,
        plain_date_time::PlainDateTimePrototypeProperties {
            year_key,
            month_key,
            month_code_key,
            day_key,
            day_of_week_key,
            day_of_year_key,
            days_in_month_key,
            days_in_year_key,
            months_in_year_key,
            in_leap_year_key,
            days_in_week_key,
            week_of_year_key,
            year_of_week_key,
            era_key,
            era_year_key,
            hour_key,
            minute_key,
            second_key,
            millisecond_key,
            microsecond_key,
            nanosecond_key,
            calendar_id_key,
            to_json_key,
            to_locale_string_key,
            equals_key,
            with_key,
            with_plain_time_key,
            with_calendar_key,
            add_key,
            subtract_key,
            round_key,
            since_key,
            until_key,
            to_plain_date_key,
            to_plain_time_key,
            to_zoned_date_time_key,
            to_string_tag_key,
            prototype_tag: plain_date_time_prototype_tag,
        },
    );

    plain_year_month::install_prototype_properties(
        agent,
        plain_year_month_prototype,
        plain_year_month_functions,
        plain_year_month::PlainYearMonthPrototypeProperties {
            year_key,
            month_key,
            month_code_key,
            days_in_month_key,
            days_in_year_key,
            months_in_year_key,
            in_leap_year_key,
            era_key,
            era_year_key,
            calendar_id_key,
            to_json_key,
            to_locale_string_key,
            equals_key,
            with_key,
            add_key,
            subtract_key,
            since_key,
            until_key,
            to_plain_date_key,
            to_string_tag_key,
            prototype_tag: plain_year_month_prototype_tag,
        },
    );

    plain_month_day::install_prototype_properties(
        agent,
        plain_month_day_prototype,
        plain_month_day_functions,
        plain_month_day::PlainMonthDayPrototypeProperties {
            month_code_key,
            day_key,
            calendar_id_key,
            to_json_key,
            to_locale_string_key,
            equals_key,
            with_key,
            to_plain_date_key,
            to_string_tag_key,
            prototype_tag: plain_month_day_prototype_tag,
        },
    );
    zoned_date_time::install_prototype_properties(
        agent,
        zoned_date_time_prototype,
        zoned_date_time_functions,
        zoned_date_time::ZonedDateTimePrototypeProperties {
            year_key,
            month_key,
            month_code_key,
            day_key,
            day_of_week_key,
            day_of_year_key,
            days_in_month_key,
            days_in_year_key,
            months_in_year_key,
            in_leap_year_key,
            days_in_week_key,
            week_of_year_key,
            year_of_week_key,
            era_key,
            era_year_key,
            hour_key,
            minute_key,
            second_key,
            millisecond_key,
            microsecond_key,
            nanosecond_key,
            epoch_nanoseconds_key,
            epoch_milliseconds_key,
            time_zone_id_key,
            calendar_key,
            calendar_id_key,
            offset_key,
            offset_nanoseconds_key,
            to_json_key,
            to_locale_string_key,
            equals_key,
            add_key,
            round_key,
            with_key,
            subtract_key,
            with_time_zone_key,
            with_calendar_key,
            with_plain_time_key,
            start_of_day_key,
            hours_in_day_key,
            since_key,
            until_key,
            to_instant_key,
            to_plain_date_time_key,
            to_plain_date_key,
            to_plain_time_key,
            to_string_tag_key,
            prototype_tag: zoned_date_time_prototype_tag,
        },
    );

    Some(())
}
