use super::compile_and_run_string_with_host;

#[test]
fn temporal_plain_year_month_constructor_getters_and_serialization() {
    let result = compile_and_run_string_with_host(
        r#"
        let yearMonth = new Temporal.PlainYearMonth(2024, 2);
        let threw = (() => {
            try {
                yearMonth.valueOf();
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        [
            yearMonth.year,
            yearMonth.month,
            yearMonth.monthCode,
            yearMonth.calendarId,
            yearMonth.toString(),
            yearMonth.toJSON(),
            Object.prototype.toString.call(yearMonth),
            threw,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "2024|2|M02|iso8601|2024-02|2024-02|[object Temporal.PlainYearMonth]|true"
    );
}

#[test]
fn temporal_plain_year_month_to_string_honors_calendar_name_and_reference_day() {
    let result = compile_and_run_string_with_host(
        r#"
        let yearMonth = new Temporal.PlainYearMonth(2000, 5, undefined, 7);
        [
            yearMonth.toString({ calendarName: "auto" }),
            yearMonth.toString({ calendarName: "never" }),
            yearMonth.toString({ calendarName: "always" }),
            yearMonth.toString({ calendarName: "critical" }),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "2000-05|2000-05|2000-05-07[u-ca=iso8601]|2000-05-07[!u-ca=iso8601]"
    );
}

#[test]
fn temporal_plain_year_month_calendar_always_matches_test262_examples() {
    let result = compile_and_run_string_with_host(
        r#"
        let constructed = new Temporal.PlainYearMonth(2019, 10, "iso8601", 31);
        let parsed = Temporal.PlainYearMonth.from("2019-10-31");
        [
            constructed.toString({ calendarName: "always" }),
            parsed.toString({ calendarName: "always" }),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "2019-10-31[u-ca=iso8601]|2019-10-01[u-ca=iso8601]");
}

#[test]
fn temporal_plain_year_month_to_string_uses_reference_day_from_string_input() {
    let result = compile_and_run_string_with_host(
        r#"
        let yearMonth = Temporal.PlainYearMonth.from("2019-10-31");
        yearMonth.toString({ calendarName: "always" });
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "2019-10-01[u-ca=iso8601]");
}

#[test]
fn temporal_plain_year_month_to_json_ignores_argument_properties() {
    let result = compile_and_run_string_with_host(
        r#"
        let options = new Proxy({}, {
            get() {
                throw new Error("should not get properties off argument");
            }
        });
        [
            new Temporal.PlainYearMonth(1972, 1).toJSON(),
            new Temporal.PlainYearMonth(1972, 1).toJSON(options),
            new Temporal.PlainYearMonth(1972, 12).toJSON(),
            new Temporal.PlainYearMonth(1972, 12).toJSON(options),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "1972-01|1972-01|1972-12|1972-12");
}

#[test]
fn temporal_plain_year_month_iso_derived_getters() {
    let result = compile_and_run_string_with_host(
        r#"
        let leapFebruary = new Temporal.PlainYearMonth(2024, 2);
        let commonDecember = new Temporal.PlainYearMonth(2023, 12);
        [
            leapFebruary.daysInMonth,
            leapFebruary.daysInYear,
            leapFebruary.monthsInYear,
            leapFebruary.inLeapYear,
            commonDecember.daysInMonth,
            commonDecember.daysInYear,
            commonDecember.monthsInYear,
            commonDecember.inLeapYear,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "29|366|12|true|31|365|12|false");
}

#[test]
fn temporal_plain_year_month_from_clones_and_normalizes_property_bags() {
    let result = compile_and_run_string_with_host(
        r#"
        let existing = new Temporal.PlainYearMonth(2024, 2);
        let clone = Temporal.PlainYearMonth.from(existing);
        let bag = Temporal.PlainYearMonth.from({ year: 1976, month: 11 });
        [
            clone !== existing,
            clone.toString(),
            bag.year,
            bag.month,
            bag.monthCode,
            bag.toString(),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|2024-02|1976|11|M11|1976-11");
}

#[test]
fn temporal_plain_year_month_from_preserves_reference_day_for_stringification() {
    let result = compile_and_run_string_with_host(
        r#"
        let fromDate = Temporal.PlainYearMonth.from(new Temporal.PlainDate(1976, 11, 18));
        let original = new Temporal.PlainYearMonth(2000, 5, undefined, 7);
        let clone = Temporal.PlainYearMonth.from(original);
        [
            fromDate.toString({ calendarName: "always" }),
            clone !== original,
            clone.toString({ calendarName: "always" }),
            Temporal.PlainYearMonth.compare(clone, new Temporal.PlainYearMonth(2000, 5, undefined, 7)),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "1976-11-01[u-ca=iso8601]|true|2000-05-07[u-ca=iso8601]|0"
    );
}

#[test]
fn temporal_plain_year_month_from_reads_options_before_validation_and_constrains_months() {
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
        let threw = (() => {
            try {
                Temporal.PlainYearMonth.from({ year: 2025, monthCode: "M08L" }, options);
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        })();
        let constrained = Temporal.PlainYearMonth.from({ year: 2021, month: 13 }, { overflow: "constrain" });
        [
            threw,
            log.join(","),
            constrained.toString(),
            constrained.monthCode,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "true|get options.overflow,get options.overflow.toString,call options.overflow.toString|2021-12|M12"
    );
}

#[test]
fn temporal_plain_year_month_from_requires_string_month_code() {
    let result = compile_and_run_string_with_host(
        r#"
        let monthCodeValues = [5, 5n, false, null, { toString: () => 5 }];
        let typeErrors = 0;
        for (let monthCode of monthCodeValues) {
            try {
                Temporal.PlainYearMonth.from({ year: 2026, monthCode });
            } catch (error) {
                typeErrors += error.constructor === TypeError ? 1 : 0;
            }
        }
        try {
            Temporal.PlainYearMonth.from({ year: 2026, monthCode: Symbol() });
        } catch (error) {
            typeErrors += error.constructor === TypeError ? 1 : 0;
        }
        String(typeErrors);
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "6");
}

#[test]
fn temporal_plain_year_month_from_validates_month_code_syntax_before_year_type() {
    let result = compile_and_run_string_with_host(
        r#"
        let badSyntax = (() => {
            try {
                Temporal.PlainYearMonth.from({ monthCode: "L99M", year: Symbol() });
                return "no throw";
            } catch (error) {
                return error.name;
            }
        })();
        let badIsoMonthCode = (() => {
            try {
                Temporal.PlainYearMonth.from({ monthCode: "M99L", year: Symbol() });
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
fn temporal_plain_year_month_calendar_strings_reject_constructor_date_strings() {
    let result = compile_and_run_string_with_host(
        r#"
        let constructorRangeErrors = 0;
        for (let calendar of ["1997-12-04[u-ca=iso8601]", "11111111", "1111-11-11"]) {
            try {
                new Temporal.PlainYearMonth(2000, 5, calendar, 1);
            } catch (error) {
                constructorRangeErrors += error.constructor === RangeError ? 1 : 0;
            }
        }
        let propertyBagCalendar = Temporal.PlainYearMonth.from({
            year: 2019,
            monthCode: "M06",
            calendar: "2020-01-01T00:00:00.000000000[u-ca=iso8601]",
        }).calendarId;
        let invalidAnnotation = (() => {
            try {
                Temporal.PlainYearMonth.from("1997-12-04[u-ca=notacal]");
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        })();
        [constructorRangeErrors, propertyBagCalendar, invalidAnnotation].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "3|iso8601|true");
}

#[test]
fn temporal_plain_year_month_string_parsing_handles_offsets_and_limits() {
    let result = compile_and_run_string_with_host(
        r#"
        let valid = [
            "1976-11",
            "1976-11-18T15:23",
            "1976-11-01T00:00:00+05:00",
            "2019-12[Africa/Abidjan]",
            "2019-12-15T00+00:00[UTC]",
            "+00197611",
            "19761118T15:23:30.1+00:00",
            "19761118T152330.1+0000",
            "+0019761118T15:23:30.1+00:00",
            "+0019761118T152330.1+0000",
            "-009999-11",
        ];
        let validResults = [];
        for (let input of valid) {
            try {
                validResults.push(`${input}=>${Temporal.PlainYearMonth.from(input).toString({ calendarName: "always" })}`);
            } catch (error) {
                validResults.push(`${input}=>!${error.name}`);
            }
        }
        let invalid = [
            "2022-09+01:00",
            "2022-09-15+00:00",
            "2022-09-15Z",
            "+999999-01",
            "1976-11[U-CA=iso8601]",
        ];
        let invalidResults = [];
        for (let input of invalid) {
            try {
                Temporal.PlainYearMonth.from(input);
                invalidResults.push(`${input}=>false`);
            } catch (error) {
                invalidResults.push(`${input}=>${error.constructor === RangeError}`);
            }
        }
        validResults.concat(invalidResults).join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "1976-11=>1976-11-01[u-ca=iso8601]|1976-11-18T15:23=>1976-11-01[u-ca=iso8601]|1976-11-01T00:00:00+05:00=>1976-11-01[u-ca=iso8601]|2019-12[Africa/Abidjan]=>2019-12-01[u-ca=iso8601]|2019-12-15T00+00:00[UTC]=>2019-12-01[u-ca=iso8601]|+00197611=>1976-11-01[u-ca=iso8601]|19761118T15:23:30.1+00:00=>1976-11-01[u-ca=iso8601]|19761118T152330.1+0000=>1976-11-01[u-ca=iso8601]|+0019761118T15:23:30.1+00:00=>1976-11-01[u-ca=iso8601]|+0019761118T152330.1+0000=>1976-11-01[u-ca=iso8601]|-009999-11=>-009999-11-01[u-ca=iso8601]|2022-09+01:00=>true|2022-09-15+00:00=>true|2022-09-15Z=>true|+999999-01=>true|1976-11[U-CA=iso8601]=>true"
    );
}

#[test]
fn temporal_plain_year_month_with_replaces_iso_fields() {
    let result = compile_and_run_string_with_host(
        r#"
        let yearMonth = new Temporal.PlainYearMonth(2024, 2);
        let changed = yearMonth.with({ year: 1976, month: 11 });
        let constrained = yearMonth.with({ month: 13 });
        let rejected = (() => {
            try {
                yearMonth.with({ month: 13 }, { overflow: "reject" });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let missingFields = (() => {
            try {
                yearMonth.with({});
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        let calendarField = (() => {
            try {
                yearMonth.with({ calendar: "iso8601" });
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        let timeZoneField = (() => {
            try {
                yearMonth.with({ timeZone: "UTC" });
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        [
            changed instanceof Temporal.PlainYearMonth,
            changed.toString(),
            constrained.toString(),
            yearMonth.toString(),
            rejected,
            missingFields,
            calendarField,
            timeZoneField,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|1976-11|2024-12|2024-02|true|true|true|true");
}

#[test]
fn temporal_plain_year_month_with_rejects_temporal_objects_and_reads_fields_before_options() {
    let result = compile_and_run_string_with_host(
        r#"
        function observedInteger(log, label, value) {
            return Object.defineProperty({}, "valueOf", {
                get() {
                    log.push(`get ${label}.valueOf`);
                    return function () {
                        log.push(`call ${label}.valueOf`);
                        return value;
                    };
                }
            });
        }

        function observedOverflow(log) {
            return Object.defineProperty({}, "toString", {
                get() {
                    log.push("get options.overflow.toString");
                    return function () {
                        log.push("call options.overflow.toString");
                        return "constrain";
                    };
                }
            });
        }

        let yearMonth = new Temporal.PlainYearMonth(2024, 2);
        let invalidChecks = [
            (() => {
                try {
                    yearMonth.with(new Temporal.PlainYearMonth(2020, 11));
                    return false;
                } catch (error) {
                    return error.constructor === TypeError;
                }
            })(),
            (() => {
                try {
                    yearMonth.with("2020-11");
                    return false;
                } catch (error) {
                    return error.constructor === TypeError;
                }
            })(),
            (() => {
                try {
                    yearMonth.with({ month: 11 }, 1);
                    return false;
                } catch (error) {
                    return error.constructor === TypeError;
                }
            })(),
        ].join(",");

        let actual = [];
        let fields = {
            get calendar() {
                actual.push("get fields.calendar");
                return undefined;
            },
            get timeZone() {
                actual.push("get fields.timeZone");
                return undefined;
            },
            get month() {
                actual.push("get fields.month");
                return observedInteger(actual, "fields.month", 11.9);
            },
            get monthCode() {
                actual.push("get fields.monthCode");
                return undefined;
            },
            get year() {
                actual.push("get fields.year");
                return observedInteger(actual, "fields.year", 1976.4);
            },
        };
        let options = {
            get overflow() {
                actual.push("get options.overflow");
                return observedOverflow(actual);
            }
        };
        yearMonth.with(fields, options);
        let overflowIndex = actual.indexOf("get options.overflow");
        [
            invalidChecks,
            overflowIndex > actual.indexOf("get fields.year"),
            overflowIndex > actual.indexOf("get fields.month"),
            overflowIndex > actual.indexOf("get fields.monthCode"),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true,true,true|true|true|true");
}

#[test]
fn temporal_plain_year_month_add_accepts_years_months_only() {
    let result = compile_and_run_string_with_host(
        r#"
        let yearMonth = new Temporal.PlainYearMonth(2019, 11);
        let months = yearMonth.add({ months: 2 });
        let years = yearMonth.add({ years: 1 });
        let lowerUnitThrew = (() => {
            try {
                yearMonth.add({ days: 1 });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        [
            months instanceof Temporal.PlainYearMonth,
            months.toString(),
            years.toString(),
            yearMonth.toString(),
            lowerUnitThrew,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|2020-01|2020-11|2019-11|true");
}

#[test]
fn temporal_plain_year_month_subtract_accepts_years_months_only() {
    let result = compile_and_run_string_with_host(
        r#"
        let yearMonth = new Temporal.PlainYearMonth(2019, 11);
        let months = yearMonth.subtract({ months: 2 });
        let years = yearMonth.subtract({ years: 1 });
        let lowerUnitThrew = (() => {
            try {
                yearMonth.subtract({ days: 1 });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        [
            months instanceof Temporal.PlainYearMonth,
            months.toString(),
            years.toString(),
            yearMonth.toString(),
            lowerUnitThrew,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|2019-09|2018-11|2019-11|true");
}

#[test]
fn temporal_plain_year_month_add_subtract_validate_overflow_options() {
    let result = compile_and_run_string_with_host(
        r#"
        let yearMonth = new Temporal.PlainYearMonth(2024, 1, undefined, 31);
        let constrainedAdd = (() => {
            try {
                return yearMonth.add({ months: 1 }).toString({ calendarName: "always" });
            } catch (error) {
                return error.name;
            }
        })();
        let rejectedAdd = (() => {
            try {
                yearMonth.add({ months: 1 }, { overflow: "reject" });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let constrainedSubtract = (() => {
            try {
                return yearMonth.subtract({ months: 1 }).toString({ calendarName: "always" });
            } catch (error) {
                return error.name;
            }
        })();
        let rejectedSubtract = (() => {
            try {
                yearMonth.subtract({ months: 11 }, { overflow: "reject" });
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

    assert_eq!(
        result,
        "2024-02-29[u-ca=iso8601]|true|2023-12-31[u-ca=iso8601]|true"
    );
}

#[test]
fn temporal_plain_year_month_since_and_until_return_iso_month_durations() {
    let result = compile_and_run_string_with_host(
        r#"
        let earlier = new Temporal.PlainYearMonth(2020, 1);
        let later = new Temporal.PlainYearMonth(2021, 3);
        let since = later.since(earlier, { largestUnit: "year" });
        let until = earlier.until("2021-03", { largestUnit: "month" });
        [
            Temporal.PlainYearMonth.prototype.since.name,
            Temporal.PlainYearMonth.prototype.since.length,
            Temporal.PlainYearMonth.prototype.until.name,
            Temporal.PlainYearMonth.prototype.until.length,
            since.years,
            since.months,
            until.years,
            until.months,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "since|1|until|1|1|2|0|14");
}

#[test]
fn temporal_plain_year_month_since_until_default_to_balanced_years_and_months() {
    let result = compile_and_run_string_with_host(
        r#"
        let earlier = new Temporal.PlainYearMonth(2020, 1);
        let later = new Temporal.PlainYearMonth(2021, 3);
        let sinceDefault = later.since(earlier);
        let sinceAuto = later.since(earlier, { largestUnit: "auto" });
        let sinceUndefined = later.since(earlier, { largestUnit: undefined });
        let untilObject = earlier.until("2021-03", {});
        [
            sinceDefault.years,
            sinceDefault.months,
            sinceAuto.years,
            sinceAuto.months,
            sinceUndefined.years,
            sinceUndefined.months,
            untilObject.years,
            untilObject.months,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "1|2|1|2|1|2|1|2");
}

#[test]
fn temporal_plain_year_month_difference_rounds_month_remainder_when_balanced() {
    let result = compile_and_run_string_with_host(
        r#"
        let earlier = new Temporal.PlainYearMonth(2019, 1);
        let later = new Temporal.PlainYearMonth(2021, 9);
        let sinceYears = later.since(earlier, {
            smallestUnit: "years",
            roundingIncrement: 4,
            roundingMode: "halfExpand",
        });
        let sinceMixed = later.since(earlier, {
            smallestUnit: "months",
            roundingIncrement: 5,
        });
        let sinceMonths = later.since(earlier, {
            largestUnit: "months",
            smallestUnit: "months",
            roundingIncrement: 10,
        });
        let untilMixed = earlier.until(later, {
            smallestUnit: "months",
            roundingIncrement: 5,
        });
        [
            sinceYears.years,
            sinceYears.months,
            sinceMixed.years,
            sinceMixed.months,
            sinceMonths.years,
            sinceMonths.months,
            untilMixed.years,
            untilMixed.months,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "4|0|2|5|0|30|2|5");
}

#[test]
fn temporal_plain_year_month_difference_rejects_rounded_month_boundary_outside_range() {
    let result = compile_and_run_string_with_host(
        r#"
        let from = new Temporal.PlainYearMonth(1970, 1);
        let to = new Temporal.PlainYearMonth(1971, 1);
        let options = { roundingIncrement: 100_000_000 };
        let sinceError = "none";
        let untilError = "none";
        try {
            from.since(to, options);
        } catch (error) {
            sinceError = error.name;
        }
        try {
            from.until(to, options);
        } catch (error) {
            untilError = error.name;
        }
        `${sinceError}|${untilError}`;
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "RangeError|RangeError");
}

#[test]
fn temporal_plain_year_month_to_plain_date_defaults_to_constrain_and_checks_limits() {
    let result = compile_and_run_string_with_host(
        r#"
        let basic = Temporal.PlainYearMonth.from("2002-01").toPlainDate({ day: 22 }).toString();
        let basicTypeError = (() => {
            try {
                Temporal.PlainYearMonth.from("2002-01").toPlainDate({ something: "nothing" });
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        let constrained = new Temporal.PlainYearMonth(2023, 2).toPlainDate({ day: 29 }).toString();
        let constrainedThirtyFirst = new Temporal.PlainYearMonth(1998, 6).toPlainDate({ day: 31 }).toString();
        let min = Temporal.PlainYearMonth.from("-271821-04");
        let minRangeError = (() => {
            try {
                min.toPlainDate({ day: 18 });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let max = Temporal.PlainYearMonth.from("+275760-09");
        let maxRangeError = (() => {
            try {
                max.toPlainDate({ day: 14 });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        [
            basic,
            basicTypeError,
            constrained,
            constrainedThirtyFirst,
            minRangeError,
            min.toPlainDate({ day: 19 }).toString(),
            maxRangeError,
            max.toPlainDate({ day: 13 }).toString(),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "2002-01-22|true|2023-02-28|1998-06-30|true|-271821-04-19|true|+275760-09-13"
    );
}

#[test]
fn temporal_plain_year_month_from_accepts_string_limits() {
    let result = compile_and_run_string_with_host(
        r#"
        let min = Temporal.PlainYearMonth.from("-271821-04");
        let max = Temporal.PlainYearMonth.from("+275760-09");
        [
            min.toString(),
            min.toString({ calendarName: "always" }),
            max.toString(),
            max.toString({ calendarName: "always" }),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "-271821-04|-271821-04-01[u-ca=iso8601]|+275760-09|+275760-09-01[u-ca=iso8601]"
    );
}

#[test]
fn temporal_plain_year_month_from_accepts_object_limits_without_plain_date_range() {
    let result = compile_and_run_string_with_host(
        r#"
        let min = Temporal.PlainYearMonth.from({ year: -271821, month: 4 });
        let max = Temporal.PlainYearMonth.from({ year: 275760, month: 9 });
        [min.toString(), max.toString()].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "-271821-04|+275760-09");
}

#[test]
fn temporal_plain_year_month_with_validates_bad_fields_before_bad_options() {
    let result = compile_and_run_string_with_host(
        r#"
        let instance = new Temporal.PlainYearMonth(2019, 10);
        let validFields = (() => {
            try {
                instance.with({ year: 2020 }, "bad options");
                return "no throw";
            } catch (error) {
                return error.name;
            }
        })();
        let invalidFields = (() => {
            try {
                instance.with({ month: -1 }, "bad options");
                return "no throw";
            } catch (error) {
                return error.name;
            }
        })();
        [validFields, invalidFields].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "TypeError|RangeError");
}

#[test]
fn temporal_plain_year_month_add_validates_reference_date_after_options() {
    let result = compile_and_run_string_with_host(
        r#"
        let log = [];
        let options = {
            get overflow() {
                log.push("get options.overflow");
                return "constrain";
            }
        };
        let threw = (() => {
            try {
                new Temporal.PlainYearMonth(-271821, 4).add({ months: 1 }, options);
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        })();
        [threw, log.join(",")].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|get options.overflow");
}
