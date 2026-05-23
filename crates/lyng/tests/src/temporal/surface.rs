use super::compile_and_run_with_host;
use lyng_js_types::Value;

#[test]
fn temporal_june_2024_removed_names_are_absent_from_public_surface() {
    let result = compile_and_run_with_host(
        r#"
        const checks = [
            !("Calendar" in Temporal),
            !("TimeZone" in Temporal),
            !("fromEpochMicroseconds" in Temporal.Instant),
            !("fromEpochSeconds" in Temporal.Instant),
            !("epochMicroseconds" in Temporal.Instant.prototype),
            !("epochSeconds" in Temporal.Instant.prototype),
            !("toZonedDateTime" in Temporal.Instant.prototype),
            !("plainDate" in Temporal.Now),
            !("plainDateTime" in Temporal.Now),
            !("zonedDateTime" in Temporal.Now),
            !("getCalendar" in Temporal.PlainDate.prototype),
            !("getISOFields" in Temporal.PlainDate.prototype),
            !("getCalendar" in Temporal.PlainDateTime.prototype),
            !("getISOFields" in Temporal.PlainDateTime.prototype),
            !("toPlainMonthDay" in Temporal.PlainDateTime.prototype),
            !("toPlainYearMonth" in Temporal.PlainDateTime.prototype),
            !("withPlainDate" in Temporal.PlainDateTime.prototype),
            !("getCalendar" in Temporal.PlainMonthDay.prototype),
            !("getISOFields" in Temporal.PlainMonthDay.prototype),
            !("getISOFields" in Temporal.PlainTime.prototype),
            !("toPlainDateTime" in Temporal.PlainTime.prototype),
            !("toZonedDateTime" in Temporal.PlainTime.prototype),
            !("epochMicroseconds" in Temporal.ZonedDateTime.prototype),
            !("epochSeconds" in Temporal.ZonedDateTime.prototype),
            !("getCalendar" in Temporal.ZonedDateTime.prototype),
            !("getISOFields" in Temporal.ZonedDateTime.prototype),
            !("getTimeZone" in Temporal.ZonedDateTime.prototype),
            !("toPlainMonthDay" in Temporal.ZonedDateTime.prototype),
            !("toPlainYearMonth" in Temporal.ZonedDateTime.prototype),
            !("withPlainDate" in Temporal.ZonedDateTime.prototype),
            typeof Temporal.Now.plainDateISO === "function",
            typeof Temporal.Now.plainDateTimeISO === "function",
            typeof Temporal.Now.zonedDateTimeISO === "function",
            typeof Temporal.Instant.fromEpochNanoseconds === "function",
            typeof Temporal.Instant.fromEpochMilliseconds === "function",
            "epochNanoseconds" in Temporal.Instant.prototype,
            "epochMilliseconds" in Temporal.Instant.prototype,
            "calendarId" in Temporal.PlainDate.prototype,
            "calendarId" in Temporal.PlainDateTime.prototype,
            "calendarId" in Temporal.PlainMonthDay.prototype,
            "timeZoneId" in Temporal.ZonedDateTime.prototype,
            "calendarId" in Temporal.ZonedDateTime.prototype,
        ];
        let allPresentOrAbsent = true;
        for (let i = 0; i < checks.length; i += 1) {
            allPresentOrAbsent = allPresentOrAbsent && checks[i];
        }
        allPresentOrAbsent;
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, Value::from_bool(true));
}
