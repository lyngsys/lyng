use super::compile_and_run_string_with_host;

#[test]
fn temporal_plain_date_time_constructor_getters_and_serialization() {
    let result = compile_and_run_string_with_host(
        r#"
        let dateTime = new Temporal.PlainDateTime(2024, 2, 29, 23, 59, 58, 123, 456, 789);
        let threw = (() => {
            try {
                dateTime.valueOf();
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        [
            dateTime.year,
            dateTime.month,
            dateTime.monthCode,
            dateTime.day,
            dateTime.hour,
            dateTime.minute,
            dateTime.second,
            dateTime.millisecond,
            dateTime.microsecond,
            dateTime.nanosecond,
            dateTime.calendarId,
            dateTime.toString(),
            dateTime.toJSON(),
            Object.prototype.toString.call(dateTime),
            threw,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "2024|2|M02|29|23|59|58|123|456|789|iso8601|2024-02-29T23:59:58.123456789|2024-02-29T23:59:58.123456789|[object Temporal.PlainDateTime]|true"
    );
}

#[test]
fn temporal_plain_date_time_iso_derived_getters_use_date_part() {
    let result = compile_and_run_string_with_host(
        r#"
        let leap = new Temporal.PlainDateTime(2024, 2, 29, 23, 59);
        let common = new Temporal.PlainDateTime(2023, 12, 31, 0, 1);
        [
            leap.dayOfWeek,
            leap.dayOfYear,
            leap.daysInMonth,
            leap.daysInYear,
            leap.monthsInYear,
            leap.inLeapYear,
            common.dayOfWeek,
            common.dayOfYear,
            common.daysInMonth,
            common.daysInYear,
            common.monthsInYear,
            common.inLeapYear,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "4|60|29|366|12|true|7|365|31|365|12|false");
}

#[test]
fn temporal_plain_date_time_from_clones_date_times_and_normalizes_property_bags() {
    let result = compile_and_run_string_with_host(
        r#"
        let existing = new Temporal.PlainDateTime(2024, 2, 29, 23, 59, 58);
        let clone = Temporal.PlainDateTime.from(existing);
        let bag = Temporal.PlainDateTime.from({
            year: 1976,
            month: 11,
            day: 18,
            hour: 1,
            minute: 2,
            nanosecond: 3,
        });
        [
            clone !== existing,
            clone.toString(),
            bag.year,
            bag.month,
            bag.day,
            bag.hour,
            bag.minute,
            bag.second,
            bag.nanosecond,
            bag.toString(),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "true|2024-02-29T23:59:58|1976|11|18|1|2|0|3|1976-11-18T01:02:00.000000003"
    );
}

#[test]
fn temporal_plain_date_time_from_defaults_to_constrain_and_reads_options_for_fast_paths() {
    let result = compile_and_run_string_with_host(
        r#"
        let log = [];
        let options = {
            get overflow() {
                log.push("get options.overflow");
                return {
                    get toString() {
                        log.push("get options.overflow.toString");
                        return function () {
                            log.push("call options.overflow.toString");
                            return "constrain";
                        };
                    }
                };
            }
        };
        let constrained = Temporal.PlainDateTime.from({ year: 2019, month: 1, day: 32 });
        Temporal.PlainDateTime.from(new Temporal.PlainDateTime(2000, 5, 2), options);
        let cloneLog = log.join(",");
        log = [];
        Temporal.PlainDateTime.from(new Temporal.PlainDate(2000, 5, 2), options);
        let plainDateLog = log.join(",");
        [
            constrained.toString(),
            cloneLog,
            plainDateLog,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "2019-01-31T00:00:00|get options.overflow,get options.overflow.toString,call options.overflow.toString|get options.overflow,get options.overflow.toString,call options.overflow.toString"
    );
}

#[test]
fn temporal_plain_date_time_from_validates_month_code_syntax_before_year_type() {
    let result = compile_and_run_string_with_host(
        r#"
        let badSyntax = (() => {
            try {
                Temporal.PlainDateTime.from({ day: 1, monthCode: "L99M", year: Symbol() });
                return "no throw";
            } catch (error) {
                return error.name;
            }
        })();
        let badIsoMonthCode = (() => {
            try {
                Temporal.PlainDateTime.from({ day: 1, monthCode: "M99L", year: Symbol() });
                return "no throw";
            } catch (error) {
                return error.name;
            }
        })();
        [badSyntax, badIsoMonthCode].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "RangeError|TypeError");
}

#[test]
fn temporal_plain_date_time_calendar_strings_handle_time_strings_and_annotations() {
    let result = compile_and_run_string_with_host(
        r#"
        let withCalendarCalendarId = (() => {
            try {
                return new Temporal.PlainDateTime(2000, 5, 2, 12, 34).withCalendar("T15:23:30").calendarId;
            } catch (error) {
                return error.name;
            }
        })();
        let invalidFirstCalendarAnnotation = (() => {
            try {
                new Temporal.PlainDateTime(2000, 5, 2, 12, 34).equals("1997-12-04[u-ca=11111111]");
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        })();
        let ignoredSecondCalendar = Temporal.PlainDateTime
            .from("1970-01-01T00:00[u-ca=iso8601][u-ca=discord]")
            .calendarId;
        [withCalendarCalendarId, invalidFirstCalendarAnnotation, ignoredSecondCalendar].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "iso8601|true|iso8601");
}

#[test]
fn temporal_plain_date_time_with_replaces_date_and_time_fields() {
    let result = compile_and_run_string_with_host(
        r#"
        let dateTime = new Temporal.PlainDateTime(2024, 2, 29, 23, 59, 58, 123, 456, 789);
        let changed = dateTime.with({ year: 1976, month: 11, hour: 1, nanosecond: 3 });
        let constrained = dateTime.with({ month: 2, day: 30 });
        let rejected = (() => {
            try {
                dateTime.with({ month: 2, day: 30 }, { overflow: "reject" });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let missingFields = (() => {
            try {
                dateTime.with({});
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        let calendarField = (() => {
            try {
                dateTime.with({ calendar: "iso8601" });
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        let timeZoneField = (() => {
            try {
                dateTime.with({ timeZone: "UTC" });
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        [
            changed instanceof Temporal.PlainDateTime,
            changed.toString(),
            constrained.toString(),
            dateTime.toString(),
            rejected,
            missingFields,
            calendarField,
            timeZoneField,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "true|1976-11-29T01:59:58.123456003|2024-02-29T23:59:58.123456789|2024-02-29T23:59:58.123456789|true|true|true|true"
    );
}

#[test]
fn temporal_plain_date_time_with_plain_time_replaces_only_time_fields() {
    let result = compile_and_run_string_with_host(
        r#"
        let dateTime = new Temporal.PlainDateTime(2015, 12, 7, 3, 24, 30, 0, 3, 500);
        let fromTime = dateTime.withPlainTime(new Temporal.PlainTime(11, 22));
        let fromString = dateTime.withPlainTime("T05:06:07.008009010");
        let fromBag = dateTime.withPlainTime({ hour: 9, microsecond: 123 });
        let midnight = dateTime.withPlainTime();
        [
            Temporal.PlainDateTime.prototype.withPlainTime.name,
            Temporal.PlainDateTime.prototype.withPlainTime.length,
            fromTime.toString(),
            fromString.toString(),
            fromBag.toString(),
            midnight.toString(),
            dateTime.toString()
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "withPlainTime|0|2015-12-07T11:22:00|2015-12-07T05:06:07.00800901|2015-12-07T09:00:00.000123|2015-12-07T00:00:00|2015-12-07T03:24:30.0000035"
    );
}

#[test]
fn temporal_plain_date_time_arithmetic_rounding_and_difference_compose_date_and_time() {
    let result = compile_and_run_string_with_host(
        r#"
        let start = new Temporal.PlainDateTime(2020, 1, 31, 23, 30);
        let added = start.add({ months: 1, hours: 2, minutes: 15 });
        let subtracted = added.subtract("PT26H15M");
        let rounded = new Temporal.PlainDateTime(2020, 1, 1, 23, 59, 30).round("minute");
        let earlier = new Temporal.PlainDateTime(2020, 1, 15, 1, 0);
        let later = new Temporal.PlainDateTime(2020, 1, 17, 3, 30);
        let since = later.since(earlier, { largestUnit: "day" });
        let until = earlier.until(later, { largestUnit: "day" });
        [
            Temporal.PlainDateTime.prototype.add.name,
            Temporal.PlainDateTime.prototype.add.length,
            Temporal.PlainDateTime.prototype.round.name,
            Temporal.PlainDateTime.prototype.round.length,
            Temporal.PlainDateTime.prototype.since.name,
            Temporal.PlainDateTime.prototype.since.length,
            Temporal.PlainDateTime.prototype.until.name,
            Temporal.PlainDateTime.prototype.until.length,
            added.toString(),
            subtracted.toString(),
            rounded.toString(),
            since.days,
            since.hours,
            since.minutes,
            until.days,
            until.hours,
            until.minutes,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "add|1|round|1|since|1|until|1|2020-03-01T01:45:00|2020-02-28T23:30:00|2020-01-02T00:00:00|2|2|30|2|2|30"
    );
}

#[test]
fn temporal_plain_date_time_add_subtract_validate_overflow_options() {
    let result = compile_and_run_string_with_host(
        r#"
        let start = new Temporal.PlainDateTime(2024, 1, 31, 23, 30);
        let constrainedAdd = start.add({ months: 1 }).toString();
        let rejectedAdd = (() => {
            try {
                start.add({ months: 1 }, { overflow: "reject" });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let constrainedSubtract = start.subtract({ months: 11 }).toString();
        let rejectedSubtract = (() => {
            try {
                start.subtract({ months: 11 }, { overflow: "reject" });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        [
            constrainedAdd,
            rejectedAdd,
            constrainedSubtract,
            rejectedSubtract,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "2024-02-29T23:30:00|true|2023-02-28T23:30:00|true");
}

#[test]
fn temporal_plain_date_time_converts_to_plain_date_and_time() {
    let result = compile_and_run_string_with_host(
        r#"
        let dateTime = new Temporal.PlainDateTime(1976, 11, 18, 1, 2, 3, 4, 5, 6);
        let date = dateTime.toPlainDate();
        let time = dateTime.toPlainTime();
        [
            date instanceof Temporal.PlainDate,
            date.toString(),
            time instanceof Temporal.PlainTime,
            time.toString(),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|1976-11-18|true|01:02:03.004005006");
}

#[test]
fn temporal_plain_date_time_to_string_honors_precision_and_calendar_options() {
    let result = compile_and_run_string_with_host(
        r#"
        let dateTime = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 400);
        [
            dateTime.toString({ fractionalSecondDigits: 2 }),
            dateTime.toString({ fractionalSecondDigits: 2.5 }),
            dateTime.toString({ smallestUnit: "minutes" }),
            dateTime.toString({ smallestUnit: "second", roundingMode: "halfExpand" }),
            dateTime.toString({ calendarName: "always" }),
            dateTime.toString({ calendarName: "critical" }),
            dateTime.toJSON({ smallestUnit: "minute" }),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "1976-11-18T15:23:30.12|1976-11-18T15:23:30.12|1976-11-18T15:23|1976-11-18T15:23:30|1976-11-18T15:23:30.1234[u-ca=iso8601]|1976-11-18T15:23:30.1234[!u-ca=iso8601]|1976-11-18T15:23:30.1234"
    );
}

#[test]
fn temporal_plain_date_time_to_zoned_date_time_resolves_civil_time() {
    let result = compile_and_run_string_with_host(
        r#"
        let dateTime = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789);
        let zoned = dateTime.toZonedDateTime("UTC");
        let offset = new Temporal.PlainDateTime(2000, 10, 29, 1, 45)
            .toZonedDateTime("+03:30", { disambiguation: "later" });
        let badOptionsThrew = (() => {
            try {
                dateTime.toZonedDateTime("UTC", "bad options");
                return false;
            } catch (error) {
                return error.constructor === TypeError;
            }
        })();
        let badDisambiguationThrew = (() => {
            try {
                dateTime.toZonedDateTime("UTC", { disambiguation: "bad" });
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        })();
        [
            Temporal.PlainDateTime.prototype.toZonedDateTime.name,
            Temporal.PlainDateTime.prototype.toZonedDateTime.length,
            zoned instanceof Temporal.ZonedDateTime,
            zoned.toString(),
            String(zoned.epochNanoseconds),
            offset.timeZoneId,
            String(offset.epochNanoseconds),
            badOptionsThrew,
            badDisambiguationThrew
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "toZonedDateTime|1|true|1976-11-18T15:23:30.123456789+00:00[UTC]|217178610123456789|+03:30|972771300000000000|true|true"
    );
}

#[test]
fn temporal_plain_date_time_to_zoned_date_time_handles_fixed_offset_limits() {
    let result = compile_and_run_string_with_host(
        r#"
        const oneHour = 1n * 60n * 60n * 1000n**3n;

        const minDt = new Temporal.PlainDateTime(-271821, 4, 19, 1, 0, 0, 0, 0, 0);
        const minValidDt = new Temporal.PlainDateTime(-271821, 4, 20, 0, 0, 0, 0, 0, 0);
        const maxDt = new Temporal.PlainDateTime(275760, 9, 13, 0, 0, 0, 0, 0, 0);

        const throwsRange = (callback) => {
            try {
                callback();
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        };

        const minValidZero = minValidDt.toZonedDateTime("+00", { disambiguation: "earlier" });
        const minValidMinus = minValidDt.toZonedDateTime("-01", { disambiguation: "later" });
        const maxValidZero = maxDt.toZonedDateTime("+00");
        const maxValidPlus = maxDt.toZonedDateTime("+01");

        [
            throwsRange(() => minDt.toZonedDateTime("+00")),
            throwsRange(() => minDt.toZonedDateTime("+01")),
            throwsRange(() => minDt.toZonedDateTime("-01")),
            minValidZero.epochNanoseconds === -8640000000000000000000n,
            minValidMinus.epochNanoseconds === -8640000000000000000000n + oneHour,
            throwsRange(() => minValidDt.toZonedDateTime("+01")),
            maxValidZero.epochNanoseconds === 8640000000000000000000n,
            maxValidPlus.epochNanoseconds === 8640000000000000000000n - oneHour,
            throwsRange(() => maxDt.toZonedDateTime("-01")),
            String(oneHour)
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "true|true|true|true|true|true|true|true|true|3600000000000"
    );
}
