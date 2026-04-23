use super::compile_and_run_string_with_host;

#[test]
fn temporal_plain_month_day_constructor_getters_and_serialization() {
    let result = compile_and_run_string_with_host(
        r#"
        let monthDay = new Temporal.PlainMonthDay(2, 29);
        let threw = (() => {
            try {
                monthDay.valueOf();
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        [
            monthDay.monthCode,
            monthDay.day,
            monthDay.calendarId,
            monthDay.toString(),
            monthDay.toJSON(),
            Object.prototype.toString.call(monthDay),
            threw,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "M02|29|iso8601|02-29|02-29|[object Temporal.PlainMonthDay]|true"
    );
}

#[test]
fn temporal_plain_month_day_from_clones_and_normalizes_property_bags() {
    let result = compile_and_run_string_with_host(
        r#"
        let existing = new Temporal.PlainMonthDay(2, 29);
        let clone = Temporal.PlainMonthDay.from(existing);
        let bag = Temporal.PlainMonthDay.from({ month: 11, day: 18 });
        let monthCodeBag = Temporal.PlainMonthDay.from({ monthCode: "M12", day: 31 });
        [
            clone !== existing,
            clone.toString(),
            bag.monthCode,
            bag.day,
            bag.toString(),
            monthCodeBag.monthCode,
            monthCodeBag.day,
            monthCodeBag.toString(),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|02-29|M11|18|11-18|M12|31|12-31");
}

#[test]
fn temporal_plain_month_day_with_replaces_iso_fields() {
    let result = compile_and_run_string_with_host(
        r#"
        let monthDay = new Temporal.PlainMonthDay(2, 29);
        let changed = monthDay.with({ month: 11, day: 18 });
        let constrained = monthDay.with({ month: 2, day: 30 });
        [
            changed instanceof Temporal.PlainMonthDay,
            changed.toString(),
            monthDay.toString(),
            constrained.toString(),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|11-18|02-29|02-29");
}

#[test]
fn temporal_partial_plain_dates_convert_to_plain_date() {
    let result = compile_and_run_string_with_host(
        r#"
        let yearMonthDate = new Temporal.PlainYearMonth(2024, 2).toPlainDate({ day: 29 });
        let monthDayDate = new Temporal.PlainMonthDay(2, 29).toPlainDate({ year: 2024 });
        [
            yearMonthDate instanceof Temporal.PlainDate,
            yearMonthDate.toString(),
            monthDayDate instanceof Temporal.PlainDate,
            monthDayDate.toString(),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|2024-02-29|true|2024-02-29");
}

#[test]
fn temporal_plain_month_day_stringification_uses_reference_year_when_requested() {
    let result = compile_and_run_string_with_host(
        r#"
        let implicit = new Temporal.PlainMonthDay(5, 2, "iso8601");
        let explicitUndefined = new Temporal.PlainMonthDay(5, 2, "iso8601", undefined);
        let explicitYear = new Temporal.PlainMonthDay(10, 31, "iso8601", 2019);
        [
            implicit.toString({ calendarName: "always" }),
            explicitUndefined.toString({ calendarName: "always" }),
            explicitYear.toString({ calendarName: "always" }),
            implicit.toString({ calendarName: "critical" }),
            implicit.toString({ calendarName: "never" }),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "1972-05-02[u-ca=iso8601]|1972-05-02[u-ca=iso8601]|2019-10-31[u-ca=iso8601]|1972-05-02[!u-ca=iso8601]|05-02"
    );
}

#[test]
fn temporal_plain_month_day_to_string_reads_calendar_name_option_in_order() {
    let result = compile_and_run_string_with_host(
        r#"
        let events = [];
        let calendarName = {};
        Object.defineProperty(calendarName, "toString", {
            get() {
                events.push("get options.calendarName.toString");
                return function() {
                    events.push("call options.calendarName.toString");
                    return "auto";
                };
            }
        });
        let options = {
            get calendarName() {
                events.push("get options.calendarName");
                return calendarName;
            }
        };
        new Temporal.PlainMonthDay(5, 2).toString(options);
        events.join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "get options.calendarName|get options.calendarName.toString|call options.calendarName.toString"
    );
}

#[test]
fn temporal_plain_month_day_from_uses_overflow_without_storing_bag_year() {
    let result = compile_and_run_string_with_host(
        r#"
        let constrained = Temporal.PlainMonthDay.from({ year: 2001, month: 2, day: 29 });
        let rejected = (() => {
            try {
                Temporal.PlainMonthDay.from(
                    { year: 2001, monthCode: "M02", day: 29 },
                    { overflow: "reject" }
                );
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let cloned = Temporal.PlainMonthDay.from(
            new Temporal.PlainMonthDay(11, 16, undefined, 1960),
            { overflow: "reject" }
        );
        [
            constrained.toString({ calendarName: "always" }),
            constrained.day,
            rejected,
            cloned.toString({ calendarName: "always" }),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "1972-02-28[u-ca=iso8601]|28|true|1960-11-16[u-ca=iso8601]"
    );
}

#[test]
fn temporal_plain_month_day_with_uses_overflow_year_without_replacing_reference_year() {
    let result = compile_and_run_string_with_host(
        r#"
        let leap = new Temporal.PlainMonthDay(2, 29, "iso8601", 1972);
        let constrained = leap.with({ year: 2001 });
        let rejected = (() => {
            try {
                leap.with({ year: 2001 }, { overflow: "reject" });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let invalidCalendar = (() => {
            try {
                leap.with({ day: 1, calendar: "iso8601" });
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        let empty = (() => {
            try {
                leap.with({});
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        [
            constrained.toString({ calendarName: "always" }),
            rejected,
            invalidCalendar,
            empty,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "1972-02-28[u-ca=iso8601]|true|true|true");
}

#[test]
fn temporal_plain_month_day_to_plain_date_constrains_and_to_json_ignores_arguments() {
    let result = compile_and_run_string_with_host(
        r#"
        let observed = [];
        let options = new Proxy({}, {
            get(_target, property) {
                observed.push(String(property));
                throw new Error("should not read options");
            }
        });
        let leapDay = new Temporal.PlainMonthDay(2, 29);
        let constrained = leapDay.toPlainDate({ year: 2023 }, options);
        let json = new Temporal.PlainMonthDay(12, 31).toJSON(options);
        [
            constrained.toString(),
            json,
            observed.length,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "2023-02-28|12-31|0");
}

#[test]
fn temporal_plain_month_day_parses_basic_and_time_offset_string_forms() {
    let result = compile_and_run_string_with_host(
        r#"
        let basic = Temporal.PlainMonthDay.from("1001");
        let prefixed = Temporal.PlainMonthDay.from("--1001");
        let timed = Temporal.PlainMonthDay.from("2000-05-02T00+00:00[UTC]");
        let offsetWithoutTime = (() => {
            try {
                Temporal.PlainMonthDay.from("09-15+01:00");
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let equalsBasic = new Temporal.PlainMonthDay(10, 1).equals("19761001T152330.1+0000");
        [
            basic.toString(),
            prefixed.toString(),
            timed.toString(),
            offsetWithoutTime,
            equalsBasic,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "10-01|10-01|05-02|true|true");
}

#[test]
fn temporal_plain_month_day_month_code_syntax_is_checked_before_year_type() {
    let result = compile_and_run_string_with_host(
        r#"
        let syntaxBeforeYear = (() => {
            try {
                Temporal.PlainMonthDay.from({ day: 1, monthCode: "L99M", year: Symbol() });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let suitabilityAfterYear = (() => {
            try {
                Temporal.PlainMonthDay.from({ day: 1, monthCode: "M99L", year: Symbol() });
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        [syntaxBeforeYear, suitabilityAfterYear].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|true");
}

#[test]
fn temporal_plain_month_day_constructor_checks_reference_year_limits() {
    let result = compile_and_run_string_with_host(
        r#"
        let upper = (() => {
            try {
                new Temporal.PlainMonthDay(9, 14, "iso8601", 275760);
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let lower = (() => {
            try {
                new Temporal.PlainMonthDay(4, 18, "iso8601", -271821);
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        [upper, lower].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|true");
}
