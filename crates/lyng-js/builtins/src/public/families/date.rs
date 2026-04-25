use super::{
    install_public_builtin_function, DateFamilyBuiltins, DateFamilyPrototypes, FamilyInstallContext,
};
use lyng_js_env::Agent;
use lyng_js_types::{
    js3_date_builtin, js3_date_get_date_builtin, js3_date_get_day_builtin,
    js3_date_get_full_year_builtin, js3_date_get_hours_builtin, js3_date_get_milliseconds_builtin,
    js3_date_get_minutes_builtin, js3_date_get_month_builtin, js3_date_get_seconds_builtin,
    js3_date_get_time_builtin, js3_date_get_timezone_offset_builtin, js3_date_get_utc_date_builtin,
    js3_date_get_utc_day_builtin, js3_date_get_utc_full_year_builtin,
    js3_date_get_utc_hours_builtin, js3_date_get_utc_milliseconds_builtin,
    js3_date_get_utc_minutes_builtin, js3_date_get_utc_month_builtin,
    js3_date_get_utc_seconds_builtin, js3_date_now_builtin, js3_date_parse_builtin,
    js3_date_set_date_builtin, js3_date_set_full_year_builtin, js3_date_set_hours_builtin,
    js3_date_set_milliseconds_builtin, js3_date_set_minutes_builtin, js3_date_set_month_builtin,
    js3_date_set_seconds_builtin, js3_date_set_time_builtin, js3_date_set_utc_date_builtin,
    js3_date_set_utc_full_year_builtin, js3_date_set_utc_hours_builtin,
    js3_date_set_utc_milliseconds_builtin, js3_date_set_utc_minutes_builtin,
    js3_date_set_utc_month_builtin, js3_date_set_utc_seconds_builtin,
    js3_date_to_date_string_builtin, js3_date_to_iso_string_builtin, js3_date_to_json_builtin,
    js3_date_to_locale_date_string_builtin, js3_date_to_locale_string_builtin,
    js3_date_to_locale_time_string_builtin, js3_date_to_primitive_builtin,
    js3_date_to_string_builtin, js3_date_to_temporal_instant_builtin,
    js3_date_to_time_string_builtin, js3_date_to_utc_string_builtin, js3_date_utc_builtin,
    js3_date_value_of_builtin, ObjectRef,
};

pub(in crate::public) fn install_date_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototypes: DateFamilyPrototypes,
) -> DateFamilyBuiltins {
    let statics = install_date_static_methods(agent, cx);
    let formatters = install_date_format_methods(agent, cx);
    let getters = install_date_getter_methods(agent, cx);
    let setters = install_date_setter_methods(agent, cx);
    let conversions = install_date_conversion_methods(agent, cx);

    DateFamilyBuiltins {
        date: install_public_builtin_function(
            agent,
            cx,
            js3_date_builtin(),
            Some(prototypes.date_prototype),
        ),
        date_prototype: prototypes.date_prototype,
        date_now: statics.now,
        date_parse: statics.parse,
        date_utc: statics.utc,
        date_to_string: formatters.string,
        date_to_date_string: formatters.date_string,
        date_to_time_string: formatters.time_string,
        date_to_locale_string: formatters.locale_string,
        date_to_locale_date_string: formatters.locale_date_string,
        date_to_locale_time_string: formatters.locale_time_string,
        date_value_of: conversions.value_of,
        date_get_time: getters.time,
        date_get_full_year: getters.full_year,
        date_get_utc_full_year: getters.utc_full_year,
        date_get_month: getters.month,
        date_get_utc_month: getters.utc_month,
        date_get_date: getters.date,
        date_get_utc_date: getters.utc_date,
        date_get_day: getters.day,
        date_get_utc_day: getters.utc_day,
        date_get_hours: getters.hours,
        date_get_utc_hours: getters.utc_hours,
        date_get_minutes: getters.minutes,
        date_get_utc_minutes: getters.utc_minutes,
        date_get_seconds: getters.seconds,
        date_get_utc_seconds: getters.utc_seconds,
        date_get_milliseconds: getters.milliseconds,
        date_get_utc_milliseconds: getters.utc_milliseconds,
        date_get_timezone_offset: getters.timezone_offset,
        date_set_time: setters.time,
        date_set_milliseconds: setters.milliseconds,
        date_set_utc_milliseconds: setters.utc_milliseconds,
        date_set_seconds: setters.seconds,
        date_set_utc_seconds: setters.utc_seconds,
        date_set_minutes: setters.minutes,
        date_set_utc_minutes: setters.utc_minutes,
        date_set_hours: setters.hours,
        date_set_utc_hours: setters.utc_hours,
        date_set_date: setters.date,
        date_set_utc_date: setters.utc_date,
        date_set_month: setters.month,
        date_set_utc_month: setters.utc_month,
        date_set_full_year: setters.full_year,
        date_set_utc_full_year: setters.utc_full_year,
        date_to_utc_string: conversions.to_utc_string,
        date_to_iso_string: conversions.to_iso_string,
        date_to_json: conversions.to_json,
        date_to_primitive: conversions.to_primitive,
        date_to_temporal_instant: conversions.to_temporal_instant,
    }
}

#[derive(Clone, Copy, Debug)]
struct DateStaticMethods {
    now: ObjectRef,
    parse: ObjectRef,
    utc: ObjectRef,
}

fn install_date_static_methods(agent: &mut Agent, cx: FamilyInstallContext) -> DateStaticMethods {
    DateStaticMethods {
        now: install_public_builtin_function(agent, cx, js3_date_now_builtin(), None),
        parse: install_public_builtin_function(agent, cx, js3_date_parse_builtin(), None),
        utc: install_public_builtin_function(agent, cx, js3_date_utc_builtin(), None),
    }
}

#[derive(Clone, Copy, Debug)]
struct DateFormatMethods {
    string: ObjectRef,
    date_string: ObjectRef,
    time_string: ObjectRef,
    locale_string: ObjectRef,
    locale_date_string: ObjectRef,
    locale_time_string: ObjectRef,
}

fn install_date_format_methods(agent: &mut Agent, cx: FamilyInstallContext) -> DateFormatMethods {
    DateFormatMethods {
        string: install_public_builtin_function(agent, cx, js3_date_to_string_builtin(), None),
        date_string: install_public_builtin_function(
            agent,
            cx,
            js3_date_to_date_string_builtin(),
            None,
        ),
        time_string: install_public_builtin_function(
            agent,
            cx,
            js3_date_to_time_string_builtin(),
            None,
        ),
        locale_string: install_public_builtin_function(
            agent,
            cx,
            js3_date_to_locale_string_builtin(),
            None,
        ),
        locale_date_string: install_public_builtin_function(
            agent,
            cx,
            js3_date_to_locale_date_string_builtin(),
            None,
        ),
        locale_time_string: install_public_builtin_function(
            agent,
            cx,
            js3_date_to_locale_time_string_builtin(),
            None,
        ),
    }
}

#[derive(Clone, Copy, Debug)]
struct DateGetterMethods {
    time: ObjectRef,
    full_year: ObjectRef,
    utc_full_year: ObjectRef,
    month: ObjectRef,
    utc_month: ObjectRef,
    date: ObjectRef,
    utc_date: ObjectRef,
    day: ObjectRef,
    utc_day: ObjectRef,
    hours: ObjectRef,
    utc_hours: ObjectRef,
    minutes: ObjectRef,
    utc_minutes: ObjectRef,
    seconds: ObjectRef,
    utc_seconds: ObjectRef,
    milliseconds: ObjectRef,
    utc_milliseconds: ObjectRef,
    timezone_offset: ObjectRef,
}

fn install_date_getter_methods(agent: &mut Agent, cx: FamilyInstallContext) -> DateGetterMethods {
    DateGetterMethods {
        time: install_public_builtin_function(agent, cx, js3_date_get_time_builtin(), None),
        full_year: install_public_builtin_function(
            agent,
            cx,
            js3_date_get_full_year_builtin(),
            None,
        ),
        utc_full_year: install_public_builtin_function(
            agent,
            cx,
            js3_date_get_utc_full_year_builtin(),
            None,
        ),
        month: install_public_builtin_function(agent, cx, js3_date_get_month_builtin(), None),
        utc_month: install_public_builtin_function(
            agent,
            cx,
            js3_date_get_utc_month_builtin(),
            None,
        ),
        date: install_public_builtin_function(agent, cx, js3_date_get_date_builtin(), None),
        utc_date: install_public_builtin_function(agent, cx, js3_date_get_utc_date_builtin(), None),
        day: install_public_builtin_function(agent, cx, js3_date_get_day_builtin(), None),
        utc_day: install_public_builtin_function(agent, cx, js3_date_get_utc_day_builtin(), None),
        hours: install_public_builtin_function(agent, cx, js3_date_get_hours_builtin(), None),
        utc_hours: install_public_builtin_function(
            agent,
            cx,
            js3_date_get_utc_hours_builtin(),
            None,
        ),
        minutes: install_public_builtin_function(agent, cx, js3_date_get_minutes_builtin(), None),
        utc_minutes: install_public_builtin_function(
            agent,
            cx,
            js3_date_get_utc_minutes_builtin(),
            None,
        ),
        seconds: install_public_builtin_function(agent, cx, js3_date_get_seconds_builtin(), None),
        utc_seconds: install_public_builtin_function(
            agent,
            cx,
            js3_date_get_utc_seconds_builtin(),
            None,
        ),
        milliseconds: install_public_builtin_function(
            agent,
            cx,
            js3_date_get_milliseconds_builtin(),
            None,
        ),
        utc_milliseconds: install_public_builtin_function(
            agent,
            cx,
            js3_date_get_utc_milliseconds_builtin(),
            None,
        ),
        timezone_offset: install_public_builtin_function(
            agent,
            cx,
            js3_date_get_timezone_offset_builtin(),
            None,
        ),
    }
}

#[derive(Clone, Copy, Debug)]
struct DateSetterMethods {
    time: ObjectRef,
    milliseconds: ObjectRef,
    utc_milliseconds: ObjectRef,
    seconds: ObjectRef,
    utc_seconds: ObjectRef,
    minutes: ObjectRef,
    utc_minutes: ObjectRef,
    hours: ObjectRef,
    utc_hours: ObjectRef,
    date: ObjectRef,
    utc_date: ObjectRef,
    month: ObjectRef,
    utc_month: ObjectRef,
    full_year: ObjectRef,
    utc_full_year: ObjectRef,
}

fn install_date_setter_methods(agent: &mut Agent, cx: FamilyInstallContext) -> DateSetterMethods {
    DateSetterMethods {
        time: install_public_builtin_function(agent, cx, js3_date_set_time_builtin(), None),
        milliseconds: install_public_builtin_function(
            agent,
            cx,
            js3_date_set_milliseconds_builtin(),
            None,
        ),
        utc_milliseconds: install_public_builtin_function(
            agent,
            cx,
            js3_date_set_utc_milliseconds_builtin(),
            None,
        ),
        seconds: install_public_builtin_function(agent, cx, js3_date_set_seconds_builtin(), None),
        utc_seconds: install_public_builtin_function(
            agent,
            cx,
            js3_date_set_utc_seconds_builtin(),
            None,
        ),
        minutes: install_public_builtin_function(agent, cx, js3_date_set_minutes_builtin(), None),
        utc_minutes: install_public_builtin_function(
            agent,
            cx,
            js3_date_set_utc_minutes_builtin(),
            None,
        ),
        hours: install_public_builtin_function(agent, cx, js3_date_set_hours_builtin(), None),
        utc_hours: install_public_builtin_function(
            agent,
            cx,
            js3_date_set_utc_hours_builtin(),
            None,
        ),
        date: install_public_builtin_function(agent, cx, js3_date_set_date_builtin(), None),
        utc_date: install_public_builtin_function(agent, cx, js3_date_set_utc_date_builtin(), None),
        month: install_public_builtin_function(agent, cx, js3_date_set_month_builtin(), None),
        utc_month: install_public_builtin_function(
            agent,
            cx,
            js3_date_set_utc_month_builtin(),
            None,
        ),
        full_year: install_public_builtin_function(
            agent,
            cx,
            js3_date_set_full_year_builtin(),
            None,
        ),
        utc_full_year: install_public_builtin_function(
            agent,
            cx,
            js3_date_set_utc_full_year_builtin(),
            None,
        ),
    }
}

#[derive(Clone, Copy, Debug)]
struct DateConversionMethods {
    value_of: ObjectRef,
    to_utc_string: ObjectRef,
    to_iso_string: ObjectRef,
    to_json: ObjectRef,
    to_primitive: ObjectRef,
    to_temporal_instant: ObjectRef,
}

fn install_date_conversion_methods(
    agent: &mut Agent,
    cx: FamilyInstallContext,
) -> DateConversionMethods {
    DateConversionMethods {
        value_of: install_public_builtin_function(agent, cx, js3_date_value_of_builtin(), None),
        to_utc_string: install_public_builtin_function(
            agent,
            cx,
            js3_date_to_utc_string_builtin(),
            None,
        ),
        to_iso_string: install_public_builtin_function(
            agent,
            cx,
            js3_date_to_iso_string_builtin(),
            None,
        ),
        to_json: install_public_builtin_function(agent, cx, js3_date_to_json_builtin(), None),
        to_primitive: install_public_builtin_function(
            agent,
            cx,
            js3_date_to_primitive_builtin(),
            None,
        ),
        to_temporal_instant: install_public_builtin_function(
            agent,
            cx,
            js3_date_to_temporal_instant_builtin(),
            None,
        ),
    }
}
