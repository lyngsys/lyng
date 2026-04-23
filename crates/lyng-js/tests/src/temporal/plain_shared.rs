use super::compile_and_run_string_with_host;

#[test]
fn temporal_plain_static_compare_orders_iso_fields() {
    let result = compile_and_run_string_with_host(
        r#"
        [
            Temporal.PlainDate.compare(new Temporal.PlainDate(2024, 2, 29), new Temporal.PlainDate(2024, 3, 1)),
            Temporal.PlainDate.compare({ year: 2024, month: 3, day: 1 }, { year: 2024, month: 2, day: 29 }),
            Temporal.PlainTime.compare(new Temporal.PlainTime(1, 2, 3), new Temporal.PlainTime(1, 2, 3)),
            Temporal.PlainTime.compare({ hour: 1, minute: 2, second: 4 }, { hour: 1, minute: 2, second: 3 }),
            Temporal.PlainDateTime.compare(
                new Temporal.PlainDateTime(2024, 2, 29, 23, 59),
                new Temporal.PlainDateTime(2024, 3, 1, 0, 0)
            ),
            Temporal.PlainDateTime.compare(
                { year: 2024, month: 3, day: 1, hour: 0, minute: 0, second: 0, millisecond: 0, microsecond: 0, nanosecond: 1 },
                { year: 2024, month: 3, day: 1, hour: 0, minute: 0, second: 0, millisecond: 0, microsecond: 0, nanosecond: 0 }
            ),
            Temporal.PlainYearMonth.compare(new Temporal.PlainYearMonth(2024, 2), new Temporal.PlainYearMonth(2024, 3)),
            Temporal.PlainYearMonth.compare({ year: 2024, month: 3 }, { year: 2024, month: 2 }),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "-1|1|0|1|-1|1|-1|1");
}

#[test]
fn temporal_plain_prototype_equals_accepts_matching_property_bags() {
    let result = compile_and_run_string_with_host(
        r#"
        [
            new Temporal.PlainDate(2024, 2, 29).equals({ year: 2024, month: 2, day: 29 }),
            new Temporal.PlainDate(2024, 2, 29).equals(new Temporal.PlainDate(2024, 3, 1)),
            new Temporal.PlainTime(1, 2, 3, 4, 5, 6).equals({ hour: 1, minute: 2, second: 3, millisecond: 4, microsecond: 5, nanosecond: 6 }),
            new Temporal.PlainTime(1, 2, 3, 4, 5, 6).equals({ hour: 1, minute: 2, second: 3, millisecond: 4, microsecond: 5, nanosecond: 7 }),
            new Temporal.PlainDateTime(1976, 11, 18, 1, 2, 3, 4, 5, 6).equals({ year: 1976, month: 11, day: 18, hour: 1, minute: 2, second: 3, millisecond: 4, microsecond: 5, nanosecond: 6 }),
            new Temporal.PlainDateTime(1976, 11, 18, 1, 2, 3, 4, 5, 6).equals({ year: 1976, month: 11, day: 19, hour: 1, minute: 2, second: 3, millisecond: 4, microsecond: 5, nanosecond: 6 }),
            new Temporal.PlainYearMonth(2024, 2).equals({ year: 2024, month: 2 }),
            new Temporal.PlainYearMonth(2024, 2).equals({ year: 2025, month: 2 }),
            new Temporal.PlainMonthDay(2, 29).equals({ month: 2, day: 29 }),
            new Temporal.PlainMonthDay(2, 29).equals({ monthCode: "M03", day: 1 }),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "true|false|true|false|true|false|true|false|true|false"
    );
}

#[test]
fn temporal_plain_and_zoned_to_locale_string_match_non_intl_to_string_shape() {
    let result = compile_and_run_string_with_host(
        r#"
        let rows = [
            [new Temporal.PlainDate(1976, 11, 18), Temporal.PlainDate, "1976-11-18"],
            [new Temporal.PlainDateTime(1976, 11, 18, 1, 2, 3, 4, 5, 6), Temporal.PlainDateTime, "1976-11-18T01:02:03.004005006"],
            [new Temporal.PlainYearMonth(1976, 11), Temporal.PlainYearMonth, "1976-11"],
            [new Temporal.PlainMonthDay(11, 18), Temporal.PlainMonthDay, "11-18"],
            [new Temporal.ZonedDateTime(0n, "UTC"), Temporal.ZonedDateTime, "1970-01-01T00:00:00+00:00[UTC]"],
        ];
        rows.map(([value, constructor, expected]) => {
            let descriptor = Object.getOwnPropertyDescriptor(constructor.prototype, "toLocaleString");
            let brandThrew = (() => {
                try {
                    constructor.prototype.toLocaleString.call({});
                    return false;
                } catch (error) {
                    return error.constructor === TypeError;
                }
            })();
            return [
                typeof constructor.prototype.toLocaleString,
                constructor.prototype.toLocaleString.length,
                value.toLocaleString(),
                value.toLocaleString() === expected,
                descriptor.writable,
                descriptor.enumerable,
                descriptor.configurable,
                brandThrew
            ].join(",");
        }).join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "function,0,1976-11-18,true,true,false,true,true|function,0,1976-11-18T01:02:03.004005006,true,true,false,true,true|function,0,1976-11,true,true,false,true,true|function,0,11-18,true,true,false,true,true|function,0,1970-01-01T00:00:00+00:00[UTC],true,true,false,true,true"
    );
}

#[test]
fn temporal_plain_string_conversion_accepts_shared_iso_date_time_forms() {
    let result = compile_and_run_string_with_host(
        r#"
        let date = Temporal.PlainDate.from("1976-11-18T01:02:03.004005006[u-ca=iso8601]");
        let dateTime = Temporal.PlainDateTime.from("1976-11-18T01:02:03.004005006[u-ca=iso8601]");
        let yearMonth = Temporal.PlainYearMonth.from("1976-11-18T01:02:03.004005006[u-ca=iso8601]");
        let monthDay = Temporal.PlainMonthDay.from("1976-11-18T01:02:03.004005006[u-ca=iso8601]");
        [
            date.toString(),
            Temporal.PlainDate.compare("1976-11-18T01:02:03.004005006", date),
            date.equals("1976-11-18T01:02:03.004005006"),
            dateTime.toString(),
            Temporal.PlainDateTime.compare("1976-11-18T01:02:03.004005006", dateTime),
            dateTime.equals("1976-11-18T01:02:03.004005006"),
            yearMonth.toString(),
            Temporal.PlainYearMonth.compare("1976-11-18T01:02:03.004005006", yearMonth),
            yearMonth.equals("1976-11-18T01:02:03.004005006"),
            monthDay.toString(),
            monthDay.equals("1976-11-18T01:02:03.004005006"),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "1976-11-18|0|true|1976-11-18T01:02:03.004005006|0|true|1976-11|0|true|11-18|true"
    );
}

#[test]
fn temporal_plain_property_bags_normalize_month_code_calendar_and_time_defaults() {
    let result = compile_and_run_string_with_host(
        r#"
        let dateTime = Temporal.PlainDateTime.from({
            year: 1976,
            monthCode: "M11",
            day: 18,
            calendar: "2020-01-01[u-ca=iso8601]"
        });
        let dateTimeEqual = new Temporal.PlainDateTime(1976, 11, 18).equals({
            year: 1976,
            monthCode: "M11",
            day: 18,
            calendar: "IsO8601"
        });
        let yearMonth = Temporal.PlainYearMonth.from({
            year: 2019,
            monthCode: "M06",
            calendar: "2016-12-31T23:59:60+00:00[UTC]"
        });
        let monthDay = Temporal.PlainMonthDay.from({
            monthCode: "M02",
            day: 29,
            calendar: "IsO8601"
        });
        let time = Temporal.PlainTime.from({ minute: 30, microsecond: 555 });
        let constructorCalendars = [
            new Temporal.PlainDate(1976, 11, 18, "IsO8601").calendarId,
            new Temporal.PlainDateTime(1976, 11, 18, 1, 2, 3, 4, 5, 6, "IsO8601").calendarId,
            new Temporal.PlainYearMonth(1976, 11, "IsO8601", 18).calendarId,
            new Temporal.PlainMonthDay(11, 18, "IsO8601", 1976).calendarId
        ].join(",");
        let emptyTimeBagThrew = (() => {
            try {
                Temporal.PlainTime.from({});
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        let invalidCalendarThrew = (() => {
            try {
                Temporal.PlainYearMonth.from({
                    year: 2019,
                    monthCode: "M06",
                    calendar: "1997-12-04[u-ca=notacal]"
                });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let invalidConstructorCalendarThrew = (() => {
            try {
                new Temporal.PlainDate(1976, 11, 18, "\u0130SO8601");
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        [
            dateTime.toString(),
            dateTimeEqual,
            yearMonth.toString(),
            monthDay.toString(),
            time.toString(),
            constructorCalendars,
            emptyTimeBagThrew,
            invalidCalendarThrew,
            invalidConstructorCalendarThrew
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "1976-11-18T00:00:00|true|2019-06|02-29|00:30:00.000555|iso8601,iso8601,iso8601,iso8601|true|true|true"
    );
}

#[test]
fn temporal_plain_property_bags_accept_temporal_object_calendars_by_slot() {
    let result = compile_and_run_string_with_host(
        r#"
        function poisonCalendarAccess(value) {
            Object.defineProperty(value, "calendar", {
                get() {
                    throw new Error("calendar getter should not be called");
                }
            });
            Object.defineProperty(value, "calendarId", {
                get() {
                    throw new Error("calendarId getter should not be called");
                }
            });
            return value;
        }

        let calendars = [
            poisonCalendarAccess(new Temporal.PlainDate(2000, 5, 2)),
            poisonCalendarAccess(new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321)),
            poisonCalendarAccess(new Temporal.PlainMonthDay(5, 2)),
            poisonCalendarAccess(new Temporal.PlainYearMonth(2000, 5)),
            poisonCalendarAccess(new Temporal.ZonedDateTime(1000000000000000000n, "UTC"))
        ];

        calendars.map((calendar) => [
            Temporal.PlainDate.from({ year: 2000, month: 5, day: 2, calendar }).calendarId,
            Temporal.PlainDateTime.from({ year: 2000, month: 5, day: 2, calendar }).calendarId,
            Temporal.PlainMonthDay.from({ month: 5, day: 2, calendar }).calendarId,
            Temporal.PlainYearMonth.from({ year: 2000, month: 5, calendar }).calendarId
        ].join(",")).join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "iso8601,iso8601,iso8601,iso8601|iso8601,iso8601,iso8601,iso8601|iso8601,iso8601,iso8601,iso8601|iso8601,iso8601,iso8601,iso8601|iso8601,iso8601,iso8601,iso8601"
    );
}

#[test]
fn temporal_plain_with_calendar_allocates_iso_clones_and_validates_argument_types() {
    let result = compile_and_run_string_with_host(
        r#"
        let date = new Temporal.PlainDate(1976, 11, 18);
        let dateTime = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789);
        let temporalCalendar = new Temporal.PlainYearMonth(2000, 5);
        let dateClone = date.withCalendar("ISO8601");
        let dateTimeClone = dateTime.withCalendar(temporalCalendar);
        let missingThrew = (() => {
            try {
                date.withCalendar();
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        let wrongTypeThrew = (() => {
            try {
                dateTime.withCalendar(1);
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        [
            Temporal.PlainDate.prototype.withCalendar.name,
            Temporal.PlainDate.prototype.withCalendar.length,
            dateClone !== date,
            dateClone.toString(),
            dateClone.calendarId,
            Temporal.PlainDateTime.prototype.withCalendar.name,
            Temporal.PlainDateTime.prototype.withCalendar.length,
            dateTimeClone !== dateTime,
            dateTimeClone.toString(),
            dateTimeClone.calendarId,
            missingThrew,
            wrongTypeThrew
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "withCalendar|1|true|1976-11-18|iso8601|withCalendar|1|true|1976-11-18T15:23:30.123456789|iso8601|true|true"
    );
}

#[test]
fn temporal_plain_and_zoned_iso_week_and_era_getters() {
    let result = compile_and_run_string_with_host(
        r#"
        function show(value) {
            return [
                value.daysInWeek,
                value.weekOfYear,
                value.yearOfWeek,
                String(value.era),
                String(value.eraYear)
            ].join(",");
        }
        let date = new Temporal.PlainDate(2021, 1, 1);
        let dateTime = new Temporal.PlainDateTime(2021, 1, 4, 12);
        let yearMonth = new Temporal.PlainYearMonth(2021, 1);
        let zoned = new Temporal.ZonedDateTime(0n, "UTC");
        [
            show(date),
            show(dateTime),
            [String(yearMonth.era), String(yearMonth.eraYear)].join(","),
            show(zoned),
            typeof Object.getOwnPropertyDescriptor(Temporal.PlainDate.prototype, "weekOfYear").get,
            typeof Object.getOwnPropertyDescriptor(Temporal.ZonedDateTime.prototype, "era").get
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "7,53,2020,undefined,undefined|7,1,2021,undefined,undefined|undefined,undefined|7,1,1970,undefined,undefined|function|function"
    );
}
