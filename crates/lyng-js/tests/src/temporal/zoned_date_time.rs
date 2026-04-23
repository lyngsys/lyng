use super::compile_and_run_string_with_host;
use lyng_js_host::{
    TemporalCivilDateTime, TemporalCivilTime, TemporalCivilToInstantRequest,
    TemporalDisambiguation, TemporalInstantToCivilRequest, TemporalInstantWithOffset, TestHost,
};

#[test]
fn temporal_zoned_date_time_constructor_getters_and_serialization() {
    let result = compile_and_run_string_with_host(
        r#"
        let zoned = new Temporal.ZonedDateTime(217178610123456789n, "UTC");
        let threw = (() => {
            try {
                zoned.valueOf();
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        [
            zoned.year,
            zoned.month,
            zoned.monthCode,
            zoned.day,
            zoned.hour,
            zoned.minute,
            zoned.second,
            zoned.millisecond,
            zoned.microsecond,
            zoned.nanosecond,
            String(zoned.epochNanoseconds),
            zoned.epochMilliseconds,
            zoned.offset,
            zoned.offsetNanoseconds,
            zoned.timeZoneId,
            zoned.calendarId,
            zoned.toString(),
            zoned.toJSON(),
            Object.prototype.toString.call(zoned),
            threw,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "1976|11|M11|18|15|23|30|123|456|789|217178610123456789|217178610123|+00:00|0|UTC|iso8601|1976-11-18T15:23:30.123456789+00:00[UTC]|1976-11-18T15:23:30.123456789+00:00[UTC]|[object Temporal.ZonedDateTime]|true"
    );
}

#[test]
fn temporal_zoned_date_time_iso_derived_getters_use_civil_date() {
    let result = compile_and_run_string_with_host(
        r#"
        let leap = new Temporal.Instant(1709164800000000000n).toZonedDateTimeISO("UTC");
        let common = new Temporal.Instant(1703980800000000000n).toZonedDateTimeISO("UTC");
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
fn temporal_zoned_date_time_week_year_boundary_getters_follow_iso_rules() {
    let result = compile_and_run_string_with_host(
        r#"
        let nsPerDay = 864n * 10n ** 11n;
        [
            new Temporal.ZonedDateTime(0n, "UTC"),
            new Temporal.ZonedDateTime(-3n * nsPerDay, "UTC"),
            new Temporal.ZonedDateTime(-4n * nsPerDay, "UTC"),
            new Temporal.ZonedDateTime(367n * nsPerDay, "UTC"),
            new Temporal.ZonedDateTime(368n * nsPerDay, "UTC"),
        ].map((zdt) => `${zdt.weekOfYear},${zdt.yearOfWeek}`).join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "1,1970|1,1970|52,1969|53,1970|1,1971");
}

#[test]
fn temporal_zoned_date_time_from_clones_existing_instances() {
    let result = compile_and_run_string_with_host(
        r#"
        let existing = new Temporal.ZonedDateTime(217178610123456789n, "UTC");
        let clone = Temporal.ZonedDateTime.from(existing);
        [
            clone !== existing,
            clone.toString(),
            String(clone.epochNanoseconds),
            clone.timeZoneId,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "true|1976-11-18T15:23:30.123456789+00:00[UTC]|217178610123456789|UTC"
    );
}

#[test]
fn temporal_zoned_date_time_property_bags_accept_month_code_and_iso_calendar_strings() {
    let result = compile_and_run_string_with_host(
        r#"
        let from = Temporal.ZonedDateTime.from({
            year: 1970,
            monthCode: "M01",
            day: 1,
            timeZone: "UTC",
            calendar: "2020-01-01[u-ca=iso8601]"
        });
        let equal = new Temporal.ZonedDateTime(0n, "UTC").equals({
            year: 1970,
            monthCode: "M01",
            day: 1,
            timeZone: "UTC",
            calendar: "IsO8601"
        });
        let since = new Temporal.ZonedDateTime(0n, "UTC").since({
            year: 1969,
            monthCode: "M12",
            day: 31,
            hour: 23,
            timeZone: "UTC",
            calendar: "2020-01"
        });
        let invalidCalendarThrew = (() => {
            try {
                Temporal.ZonedDateTime.from({
                    year: 1970,
                    monthCode: "M01",
                    day: 1,
                    timeZone: "UTC",
                    calendar: "1997-12-04[u-ca=notacal]"
                });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        [
            from.toString(),
            equal,
            since.hours,
            invalidCalendarThrew
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "1970-01-01T00:00:00+00:00[UTC]|true|1|true");
}

#[test]
fn temporal_zoned_date_time_compare_and_add_use_epoch_order_and_utc_resolution() {
    let result = compile_and_run_string_with_host(
        r#"
        let start = new Temporal.ZonedDateTime(0n, "UTC");
        let added = start.add({ days: 1, hours: 2 });
        [
            Temporal.ZonedDateTime.compare.name,
            Temporal.ZonedDateTime.compare.length,
            Temporal.ZonedDateTime.compare(start, added),
            Temporal.ZonedDateTime.compare(added, start),
            Temporal.ZonedDateTime.prototype.add.name,
            Temporal.ZonedDateTime.prototype.add.length,
            added.toString(),
            String(added.epochNanoseconds)
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "compare|2|-1|1|add|1|1970-01-02T02:00:00+00:00[UTC]|93600000000000"
    );
}

#[test]
fn temporal_zoned_date_time_subtract_reuses_utc_civil_resolution() {
    let result = compile_and_run_string_with_host(
        r#"
        let start = Temporal.ZonedDateTime.from("1970-01-02T02:30+00:00[UTC]");
        let subtracted = start.subtract({ days: 1, minutes: 45 });
        [
            Temporal.ZonedDateTime.prototype.subtract.name,
            Temporal.ZonedDateTime.prototype.subtract.length,
            subtracted.toString(),
            String(subtracted.epochNanoseconds)
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "subtract|1|1970-01-01T01:45:00+00:00[UTC]|6300000000000"
    );
}

#[test]
fn temporal_zoned_date_time_with_time_zone_and_calendar_allocate_iso_clones() {
    let result = compile_and_run_string_with_host(
        r#"
        let zoned = Temporal.ZonedDateTime.from("1970-01-02T02:30+00:00[UTC]");
        let withZone = zoned.withTimeZone("UTC");
        let withCalendar = zoned.withCalendar("ISO8601");
        [
            Temporal.ZonedDateTime.prototype.withTimeZone.name,
            Temporal.ZonedDateTime.prototype.withTimeZone.length,
            withZone !== zoned,
            String(withZone.epochNanoseconds),
            withZone.toString(),
            Temporal.ZonedDateTime.prototype.withCalendar.name,
            Temporal.ZonedDateTime.prototype.withCalendar.length,
            withCalendar !== zoned,
            withCalendar.calendarId,
            withCalendar.toString()
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "withTimeZone|1|true|95400000000000|1970-01-02T02:30:00+00:00[UTC]|withCalendar|1|true|iso8601|1970-01-02T02:30:00+00:00[UTC]"
    );
}

#[test]
fn temporal_zoned_date_time_with_plain_time_replaces_civil_time_in_zone() {
    let result = compile_and_run_string_with_host(
        r#"
        let zoned = Temporal.ZonedDateTime.from("1970-01-02T02:30+00:00[UTC]");
        let replaced = zoned.withPlainTime("05:06:07.008009010");
        let midnight = zoned.withPlainTime();
        [
            Temporal.ZonedDateTime.prototype.withPlainTime.name,
            Temporal.ZonedDateTime.prototype.withPlainTime.length,
            replaced.toString(),
            String(replaced.epochNanoseconds),
            midnight.toString(),
            String(midnight.epochNanoseconds)
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "withPlainTime|0|1970-01-02T05:06:07.00800901+00:00[UTC]|104767008009010|1970-01-02T00:00:00+00:00[UTC]|86400000000000"
    );
}

#[test]
fn temporal_zoned_date_time_with_plain_time_throws_range_error_for_limit_cases() {
    let result = compile_and_run_string_with_host(
        r#"
        let cases = [
            () => new Temporal.ZonedDateTime(-864n * 10n**19n, "-01").withPlainTime(),
            () => new Temporal.ZonedDateTime(-864n * 10n**19n, "+01").withPlainTime(),
            () => new Temporal.ZonedDateTime(-864n * 10n**19n, "-01").withPlainTime("00:00"),
            () => new Temporal.ZonedDateTime(-864n * 10n**19n, "+01").withPlainTime("00:00"),
            () => new Temporal.ZonedDateTime(864n * 10n**19n, "UTC").withPlainTime("01:00"),
        ];
        cases.map((run) => {
            try {
                run();
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        }).join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|true|true|true|true");
}

#[test]
fn temporal_zoned_date_time_start_of_day_and_hours_in_day_use_zone_resolution() {
    let result = compile_and_run_string_with_host(
        r#"
        let zoned = Temporal.ZonedDateTime.from("1970-01-02T02:30+00:00[UTC]");
        let start = zoned.startOfDay();
        [
            Temporal.ZonedDateTime.prototype.startOfDay.name,
            Temporal.ZonedDateTime.prototype.startOfDay.length,
            start.toString(),
            String(start.epochNanoseconds),
            zoned.hoursInDay
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "startOfDay|0|1970-01-02T00:00:00+00:00[UTC]|86400000000000|24"
    );
}

#[test]
fn temporal_zoned_date_time_to_string_honors_precision_and_annotation_options() {
    let result = compile_and_run_string_with_host(
        r#"
        let zoned = Temporal.ZonedDateTime.from("1970-01-02T03:04:05.678901234+00:00[UTC]");
        [
            zoned.toString({ smallestUnit: "minute" }),
            zoned.toString({ fractionalSecondDigits: 3 }),
            zoned.toString({ smallestUnit: "second", roundingMode: "ceil" }),
            zoned.toString({ offset: "never" }),
            zoned.toString({ timeZoneName: "never" }),
            zoned.toString({ calendarName: "always" }),
            zoned.toString({ timeZoneName: "critical", calendarName: "critical" })
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "1970-01-02T03:04+00:00[UTC]|1970-01-02T03:04:05.678+00:00[UTC]|1970-01-02T03:04:06+00:00[UTC]|1970-01-02T03:04:05.678901234[UTC]|1970-01-02T03:04:05.678901234+00:00|1970-01-02T03:04:05.678901234+00:00[UTC][u-ca=iso8601]|1970-01-02T03:04:05.678901234+00:00[!UTC][!u-ca=iso8601]"
    );
}

#[test]
fn temporal_zoned_date_time_round_uses_exact_time_options() {
    let host = TestHost::new();
    host.define_temporal_instant_to_civil(
        TemporalInstantToCivilRequest {
            time_zone_id: "+01:00".into(),
            epoch_nanoseconds: 217_175_010_123_987_500,
        },
        Ok(TemporalCivilTime {
            date_time: TemporalCivilDateTime::new(1976, 11, 18, 15, 23, 30, 123, 987, 500),
            offset_nanoseconds: 3_600_000_000_000,
        }),
    );
    host.define_temporal_civil_to_instant(
        TemporalCivilToInstantRequest {
            time_zone_id: "+01:00".into(),
            date_time: TemporalCivilDateTime::new(1976, 11, 19, 0, 0, 0, 0, 0, 0),
            disambiguation: TemporalDisambiguation::Compatible,
        },
        Ok(TemporalInstantWithOffset {
            epoch_nanoseconds: 217_206_000_000_000_000,
            offset_nanoseconds: 3_600_000_000_000,
        }),
    );
    host.define_temporal_instant_to_civil(
        TemporalInstantToCivilRequest {
            time_zone_id: "+01:00".into(),
            epoch_nanoseconds: 217_206_000_000_000_000,
        },
        Ok(TemporalCivilTime {
            date_time: TemporalCivilDateTime::new(1976, 11, 19, 0, 0, 0, 0, 0, 0),
            offset_nanoseconds: 3_600_000_000_000,
        }),
    );
    host.define_temporal_instant_to_civil(
        TemporalInstantToCivilRequest {
            time_zone_id: "+01:00".into(),
            epoch_nanoseconds: 217_209_600_000_000_000,
        },
        Ok(TemporalCivilTime {
            date_time: TemporalCivilDateTime::new(1976, 11, 19, 1, 0, 0, 0, 0, 0),
            offset_nanoseconds: 3_600_000_000_000,
        }),
    );

    let result = compile_and_run_string_with_host(
        r#"
        let zoned = new Temporal.ZonedDateTime(1000000000987654321n, "UTC");
        let roundedSecond = zoned.round("second");
        let roundedMinute = zoned.round({ smallestUnit: "minute", roundingMode: "ceil" });
        let roundedDay = zoned.round({ smallestUnit: "day" });
        let offset = new Temporal.ZonedDateTime(217175010123987500n, "+01:00");
        let offsetDay = offset.round({ smallestUnit: "day", roundingMode: "ceil" });
        [
            Temporal.ZonedDateTime.prototype.round.name,
            Temporal.ZonedDateTime.prototype.round.length,
            roundedSecond.toString(),
            String(roundedSecond.epochNanoseconds),
            roundedMinute.toString(),
            String(roundedMinute.epochNanoseconds),
            roundedDay.toString(),
            String(roundedDay.epochNanoseconds),
            offsetDay.toString(),
            String(offsetDay.epochNanoseconds)
        ].join("|");
        "#,
        host,
    );

    assert_eq!(
        result,
        "round|1|2001-09-09T01:46:41+00:00[UTC]|1000000001000000000|2001-09-09T01:47:00+00:00[UTC]|1000000020000000000|2001-09-09T00:00:00+00:00[UTC]|999993600000000000|1976-11-19T00:00:00+01:00[+01:00]|217206000000000000"
    );
}

#[test]
fn temporal_zoned_date_time_with_replaces_civil_fields() {
    let result = compile_and_run_string_with_host(
        r#"
        let zoned = Temporal.ZonedDateTime.from("1976-11-18T15:23:30.123456789+00:00[UTC]");
        let replaced = zoned.with({ year: 2019, monthCode: "M05", day: 6, hour: 7, minute: 8, second: 9, millisecond: 10, microsecond: 11, nanosecond: 12 });
        let partial = zoned.with({ month: 5, second: 15 });
        [
            Temporal.ZonedDateTime.prototype.with.name,
            Temporal.ZonedDateTime.prototype.with.length,
            replaced.toString(),
            String(replaced.epochNanoseconds),
            partial.toString(),
            String(partial.epochNanoseconds)
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "with|1|2019-05-06T07:08:09.010011012+00:00[UTC]|1557126489010011012|1976-05-18T15:23:15.123456789+00:00[UTC]|201280995123456789"
    );
}

#[test]
fn temporal_zoned_date_time_from_compare_and_equals_accept_utc_strings_and_bags() {
    let result = compile_and_run_string_with_host(
        r#"
        let fromString = Temporal.ZonedDateTime.from("1970-01-02T02:00+00:00[UTC]");
        let fromBag = Temporal.ZonedDateTime.from({
            year: 1970,
            month: 1,
            day: 2,
            hour: 2,
            timeZone: "UTC"
        });
        [
            fromString.toString(),
            String(fromString.epochNanoseconds),
            fromBag.toString(),
            String(fromBag.epochNanoseconds),
            Temporal.ZonedDateTime.compare("1970-01-02T01:00+00:00[UTC]", fromBag),
            fromBag.equals("1970-01-02T02:00+00:00[UTC]")
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "1970-01-02T02:00:00+00:00[UTC]|93600000000000|1970-01-02T02:00:00+00:00[UTC]|93600000000000|-1|true"
    );
}

#[test]
fn temporal_zoned_date_time_since_and_until_return_exact_utc_durations() {
    let result = compile_and_run_string_with_host(
        r#"
        let start = Temporal.ZonedDateTime.from("1970-01-01T00:00+00:00[UTC]");
        let end = Temporal.ZonedDateTime.from("1970-01-03T03:04:05.006007008+00:00[UTC]");
        [
            Temporal.ZonedDateTime.prototype.since.name,
            Temporal.ZonedDateTime.prototype.since.length,
            Temporal.ZonedDateTime.prototype.until.name,
            Temporal.ZonedDateTime.prototype.until.length,
            end.since(start).toString(),
            start.until(end).toString(),
            end.since(start, { largestUnit: "auto" }).toString(),
            start.until(end, { largestUnit: "auto" }).toString(),
            end.since(start, { largestUnit: "hour" }).toString(),
            start.until(end, { smallestUnit: "second" }).toString(),
            start.since(end).toString()
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "since|1|until|1|PT51H4M5.006007008S|PT51H4M5.006007008S|PT51H4M5.006007008S|PT51H4M5.006007008S|PT51H4M5.006007008S|PT51H4M5S|-PT51H4M5.006007008S"
    );
}

#[test]
fn temporal_zoned_date_time_converts_to_instant_and_plain_types() {
    let result = compile_and_run_string_with_host(
        r#"
        let zoned = new Temporal.ZonedDateTime(217178610123456789n, "UTC");
        let instant = zoned.toInstant();
        let plainDateTime = zoned.toPlainDateTime();
        let plainDate = zoned.toPlainDate();
        let plainTime = zoned.toPlainTime();
        [
            instant instanceof Temporal.Instant,
            String(instant.epochNanoseconds),
            plainDateTime.toString(),
            plainDate.toString(),
            plainTime.toString(),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "true|217178610123456789|1976-11-18T15:23:30.123456789|1976-11-18|15:23:30.123456789"
    );
}
