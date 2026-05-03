use super::compile_and_run_string_with_host;

#[test]
fn temporal_plain_date_constructor_getters_and_serialization() {
    let result = compile_and_run_string_with_host(
        r#"
        let date = new Temporal.PlainDate(2024, 2, 29);
        let threw = (() => {
            try {
                date.valueOf();
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        [
            date.year,
            date.month,
            date.monthCode,
            date.day,
            date.calendarId,
            date.toString(),
            date.toJSON(),
            Object.prototype.toString.call(date),
            threw,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "2024|2|M02|29|iso8601|2024-02-29|2024-02-29|[object Temporal.PlainDate]|true"
    );
}

#[test]
fn temporal_plain_date_to_json_ignores_argument_properties() {
    let result = compile_and_run_string_with_host(
        r#"
        let options = new Proxy({}, {
            get() {
                throw new Error("should not get properties off argument");
            }
        });
        [
            new Temporal.PlainDate(1972, 1, 1).toJSON(),
            new Temporal.PlainDate(1972, 1, 1).toJSON(options),
            new Temporal.PlainDate(1972, 12, 31).toJSON(),
            new Temporal.PlainDate(1972, 12, 31).toJSON(options),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "1972-01-01|1972-01-01|1972-12-31|1972-12-31");
}

#[test]
fn temporal_plain_date_to_locale_string_ignores_argument_properties() {
    let result = compile_and_run_string_with_host(
        r#"
        let options = new Proxy({}, {
            get() {
                throw new Error("should not get properties off argument");
            }
        });
        [
            new Temporal.PlainDate(1972, 1, 1).toLocaleString(),
            new Temporal.PlainDate(1972, 1, 1).toLocaleString(options),
            new Temporal.PlainDate(1972, 12, 31).toLocaleString(),
            new Temporal.PlainDate(1972, 12, 31).toLocaleString(options),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "1972-01-01|1972-01-01|1972-12-31|1972-12-31");
}

#[test]
fn temporal_plain_date_to_string_honors_calendar_name_options() {
    let result = compile_and_run_string_with_host(
        r#"
        let date = new Temporal.PlainDate(2000, 5, 2);
        let log = [];
        let options = {
            get calendarName() {
                log.push("get options.calendarName");
                return {
                    get toString() {
                        log.push("get options.calendarName.toString");
                        return function () {
                        log.push("call options.calendarName.toString");
                        return "auto";
                        };
                    },
                };
            }
        };
        let badOptions = [null, true, "some string", Symbol(), 1, 2n];
        let badCount = 0;
        for (let value of badOptions) {
            try {
                date.toString(value);
            } catch (error) {
                badCount += error.constructor === TypeError ? 1 : 0;
            }
        }
        [
            date.toString({ calendarName: "always" }),
            date.toString({ calendarName: "critical" }),
            date.toString({ calendarName: "never" }),
            date.toString(options),
            log.join(","),
            badCount
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "2000-05-02[u-ca=iso8601]|2000-05-02[!u-ca=iso8601]|2000-05-02|2000-05-02|get options.calendarName,get options.calendarName.toString,call options.calendarName.toString|6"
    );
}

#[test]
fn temporal_plain_date_iso_derived_getters() {
    let result = compile_and_run_string_with_host(
        r#"
        let leap = new Temporal.PlainDate(2024, 2, 29);
        let common = new Temporal.PlainDate(2023, 12, 31);
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
fn temporal_plain_date_from_clones_dates_and_normalizes_property_bags() {
    let result = compile_and_run_string_with_host(
        r#"
        let existing = new Temporal.PlainDate(2024, 2, 29);
        let clone = Temporal.PlainDate.from(existing);
        let bag = Temporal.PlainDate.from({ year: 1976, month: 11, day: 18 });
        let string = Temporal.PlainDate.from("2017-01-01");
        let invalidStringThrew = (() => {
            try {
                Temporal.PlainDate.from("2017-02-29");
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        })();
        let emptyBagThrew = (() => {
            try {
                Temporal.PlainDate.from({});
                return false;
            } catch (error) {
                return error.constructor === TypeError;
            }
        })();
        [
            clone !== existing,
            clone.toString(),
            bag.year,
            bag.month,
            bag.day,
            bag.toString(),
            string.toString(),
            invalidStringThrew,
            emptyBagThrew,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "true|2024-02-29|1976|11|18|1976-11-18|2017-01-01|true|true"
    );
}

#[test]
fn temporal_plain_date_from_validates_options_after_invalid_strings() {
    let result = compile_and_run_string_with_host(
        r#"
        let badOptions = [null, true, "options", 1, Symbol()];
        let count = 0;
        for (let options of badOptions) {
            try {
                Temporal.PlainDate.from({ year: 1976, month: 11, day: 18 }, options);
            } catch (error) {
                count += error.constructor === TypeError ? 1 : 0;
            }
            try {
                Temporal.PlainDate.from(new Temporal.PlainDate(1976, 11, 18), options);
            } catch (error) {
                count += error.constructor === TypeError ? 1 : 0;
            }
            try {
                Temporal.PlainDate.from("1976-11-18Z", options);
            } catch (error) {
                count += error.constructor === RangeError ? 1 : 0;
            }
        }
        String(count);
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "15");
}

#[test]
fn temporal_plain_date_from_reads_options_before_invalid_month_code_validation() {
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
                Temporal.PlainDate.from({ year: 2025, monthCode: "M08L", day: 14 }, options);
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        })();
        [threw, log.join(",")].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "true|get options.overflow,get options.overflow.toString,call options.overflow.toString"
    );
}

#[test]
fn temporal_plain_date_from_constrains_property_bags_and_requires_string_month_code() {
    let result = compile_and_run_string_with_host(
        r#"
        let constrained = Temporal.PlainDate.from({ year: 2021, month: 13, day: 500 });
        let monthCodeValues = [5, 5n, false, null, { toString: () => 5 }];
        let typeErrors = 0;
        for (let monthCode of monthCodeValues) {
            try {
                Temporal.PlainDate.from({ year: 2026, monthCode, day: 1 });
            } catch (error) {
                typeErrors += error.constructor === TypeError ? 1 : 0;
            }
        }
        try {
            Temporal.PlainDate.from({ year: 2026, monthCode: Symbol(), day: 1 });
        } catch (error) {
            typeErrors += error.constructor === TypeError ? 1 : 0;
        }
        [
            constrained.toString(),
            constrained.monthCode,
            typeErrors,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "2021-12-31|M12|6");
}

#[test]
fn temporal_plain_date_from_validates_month_code_syntax_before_year_type() {
    let result = compile_and_run_string_with_host(
        r#"
        let badSyntax = (() => {
            try {
                Temporal.PlainDate.from({ day: 1, monthCode: "L99M", year: Symbol() });
                return "no throw";
            } catch (error) {
                return error.name;
            }
        })();
        let badIsoMonthCode = (() => {
            try {
                Temporal.PlainDate.from({ day: 1, monthCode: "M99L", year: Symbol() });
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
fn temporal_plain_date_calendar_strings_distinguish_constructor_and_with_calendar_paths() {
    let result = compile_and_run_string_with_host(
        r#"
        let constructorRangeErrors = 0;
        for (let calendar of ["1997-12-04[u-ca=iso8601]", "11111111", "1111-11-11"]) {
            try {
                new Temporal.PlainDate(2000, 5, 2, calendar);
            } catch (error) {
                constructorRangeErrors += error.constructor === RangeError ? 1 : 0;
            }
        }
        let withCalendarCalendarId = (() => {
            try {
                return new Temporal.PlainDate(2000, 5, 2).withCalendar("15:23:30.123").calendarId;
            } catch (error) {
                return error.name;
            }
        })();
        let invalidAnnotation = (() => {
            try {
                Temporal.PlainDate.from("2020-01-01[u-ca=notexist]");
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        })();
        [constructorRangeErrors, withCalendarCalendarId, invalidAnnotation].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "3|iso8601|true");
}

#[test]
fn temporal_plain_date_with_replaces_iso_fields() {
    let result = compile_and_run_string_with_host(
        r#"
        let date = new Temporal.PlainDate(2024, 2, 29);
        let changed = date.with({ year: 1976, month: 11 });
        let constrained = date.with({ month: 2, day: 30 });
        let rejectThrew = (() => {
            try {
                date.with({ month: 2, day: 30 }, { overflow: "reject" });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        [
            changed instanceof Temporal.PlainDate,
            changed.toString(),
            date.toString(),
            constrained.toString(),
            rejectThrew,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|1976-11-29|2024-02-29|2024-02-29|true");
}

#[test]
fn temporal_plain_date_with_honors_overflow_and_rejects_calendar_like_bags() {
    let result = compile_and_run_string_with_host(
        r#"
        let date = new Temporal.PlainDate(2019, 1, 31);
        let constrained = date.with({ monthCode: "M02" });
        let rejectThrew = (() => {
            try {
                date.with({ monthCode: "M02" }, { overflow: "reject" });
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        })();
        let calendarLikeThrew = (() => {
            try {
                date.with({ year: 2021, calendar: "iso8601" });
                return false;
            } catch (error) {
                return error.constructor === TypeError;
            }
        })();
        let order = [];
        date.with(
            {
                get calendar() {
                    order.push("get fields.calendar");
                    return undefined;
                },
                get timeZone() {
                    order.push("get fields.timeZone");
                    return undefined;
                },
                get day() {
                    order.push("get fields.day");
                    return 31;
                },
                get month() {
                    order.push("get fields.month");
                    return 2;
                },
                get monthCode() {
                    order.push("get fields.monthCode");
                    return "M02";
                },
                get year() {
                    order.push("get fields.year");
                    return 2019;
                }
            },
            {
                get overflow() {
                    order.push("get options.overflow");
                    return "constrain";
                }
            }
        );
        [
            constrained.toString(),
            rejectThrew,
            calendarLikeThrew,
            order.join(",")
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "2019-02-28|true|true|get fields.calendar,get fields.timeZone,get fields.day,get fields.month,get fields.monthCode,get fields.year,get options.overflow"
    );
}

#[test]
fn temporal_plain_date_add_balances_iso_duration_fields() {
    let result = compile_and_run_string_with_host(
        r#"
        let leap = new Temporal.PlainDate(2020, 2, 29).add({ years: 1 });
        let constrained = new Temporal.PlainDate(2021, 1, 31).add({ months: 1 });
        let balanced = new Temporal.PlainDate(1976, 11, 18).add({
            weeks: 2,
            days: 3,
            hours: 48,
            minutes: 1440,
        });
        let original = new Temporal.PlainDate(2020, 2, 29);
        [
            leap instanceof Temporal.PlainDate,
            leap.toString(),
            constrained.toString(),
            balanced.toString(),
            original.toString(),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|2021-02-28|2021-02-28|1976-12-08|2020-02-29");
}

#[test]
fn temporal_plain_date_subtract_balances_iso_duration_fields() {
    let result = compile_and_run_string_with_host(
        r#"
        let leap = new Temporal.PlainDate(2021, 2, 28).subtract({ years: 1 });
        let constrained = new Temporal.PlainDate(2021, 3, 31).subtract({ months: 1 });
        let balanced = new Temporal.PlainDate(1976, 12, 8).subtract({
            weeks: 2,
            days: 3,
            hours: 48,
            minutes: 1440,
        });
        [
            leap instanceof Temporal.PlainDate,
            leap.toString(),
            constrained.toString(),
            balanced.toString(),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|2020-02-28|2021-02-28|1976-11-18");
}

#[test]
fn temporal_plain_date_add_subtract_and_to_plain_date_time_honor_options_and_limits() {
    let result = compile_and_run_string_with_host(
        r#"
        let addConstrained = new Temporal.PlainDate(2019, 1, 31).add({ months: 1 });
        let addRejected = (() => {
            try {
                new Temporal.PlainDate(2019, 1, 31).add({ months: 1 }, { overflow: "reject" });
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        })();
        let subtractConstrained = new Temporal.PlainDate(2019, 3, 31).subtract({ months: 1 });
        let subtractRejected = (() => {
            try {
                new Temporal.PlainDate(2019, 3, 31).subtract({ months: 1 }, { overflow: "reject" });
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        })();
        let min = new Temporal.PlainDate(-271821, 4, 19);
        let midnightThrew = (() => {
            try {
                min.toPlainDateTime(new Temporal.PlainTime(0, 0));
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        })();
        let firstNs = min.toPlainDateTime(new Temporal.PlainTime(0, 0, 0, 0, 0, 1));
        [
            Temporal.PlainDate.prototype.toPlainDateTime.length,
            addConstrained.toString(),
            addRejected,
            subtractConstrained.toString(),
            subtractRejected,
            midnightThrew,
            firstNs.toString()
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "0|2019-02-28|true|2019-02-28|true|true|-271821-04-19T00:00:00.000000001"
    );
}

#[test]
fn temporal_plain_date_since_and_until_return_iso_date_durations() {
    let result = compile_and_run_string_with_host(
        r#"
        let earlier = new Temporal.PlainDate(2020, 1, 15);
        let later = new Temporal.PlainDate(2020, 3, 1);
        let since = later.since(earlier);
        let until = earlier.until("2020-03-01", { largestUnit: "week" });
        [
            Temporal.PlainDate.prototype.since.name,
            Temporal.PlainDate.prototype.since.length,
            Temporal.PlainDate.prototype.until.name,
            Temporal.PlainDate.prototype.until.length,
            since.days,
            until.weeks,
            until.days,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "since|1|until|1|46|6|4");
}

#[test]
fn temporal_plain_date_since_until_balance_calendar_units() {
    let result = compile_and_run_string_with_host(
        r#"
        let sinceMonth = Temporal.PlainDate.from("2021-08-17").since("2021-07-16", {
            largestUnit: "month"
        });
        let untilYear = Temporal.PlainDate.from("2021-07-16").until("2022-09-19", {
            largestUnit: "year"
        });
        let untilWeeks = Temporal.PlainDate.from("2021-07-16").until("2021-08-13", {
            largestUnit: "week"
        });
        let mismatch = (() => {
            try {
                Temporal.PlainDate.from("2021-07-16").until("2021-08-13", {
                    largestUnit: "week",
                    smallestUnit: "month"
                });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        [
            sinceMonth.years,
            sinceMonth.months,
            sinceMonth.weeks,
            sinceMonth.days,
            untilYear.years,
            untilYear.months,
            untilYear.weeks,
            untilYear.days,
            untilWeeks.weeks,
            untilWeeks.days,
            mismatch,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "0|1|0|1|1|2|0|3|4|0|true");
}

#[test]
fn temporal_plain_date_until_counts_days_across_year_boundaries() {
    let result = compile_and_run_string_with_host(
        r#"
        let one = Temporal.PlainDate.from("2019-01-01");
        let two = Temporal.PlainDate.from("2020-01-01");
        let common = one.until(two, { largestUnit: "days" });
        let leap = two.until("2021-01-01", { largestUnit: "days" });
        [
            common.days,
            leap.days,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "365|366");
}

#[test]
fn temporal_plain_date_until_uses_auto_largest_unit_for_rounding() {
    let result = compile_and_run_string_with_host(
        r#"
        let earlier = Temporal.PlainDate.from("2019-01-08");
        let later = Temporal.PlainDate.from("2021-09-07");
        let years = earlier.until(later, {
            smallestUnit: "years",
            roundingIncrement: 4,
            roundingMode: "halfExpand",
        });
        let months = earlier.until(later, {
            smallestUnit: "months",
            roundingIncrement: 10,
            roundingMode: "halfExpand",
        });
        let weeks = earlier.until(later, {
            smallestUnit: "weeks",
            roundingIncrement: 12,
            roundingMode: "halfExpand",
        });
        [
            years.years,
            months.months,
            weeks.weeks,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "4|30|144");
}

#[test]
fn temporal_plain_date_difference_rejects_rounding_outside_iso_limits() {
    let result = compile_and_run_string_with_host(
        r#"
        let from = new Temporal.PlainDate(1970, 1, 1);
        let to = new Temporal.PlainDate(1971, 1, 1);
        let options = { roundingIncrement: 100000000, smallestUnit: "months" };
        let since = (() => {
            try {
                from.since(to, options);
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        })();
        let until = (() => {
            try {
                to.until(from, options);
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        })();
        [since, until].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|true");
}

#[test]
fn temporal_plain_date_since_rounds_months_relative_to_receiver() {
    let result = compile_and_run_string_with_host(
        r#"
        let earlier = Temporal.PlainDate.from("2019-01-01");
        let later = Temporal.PlainDate.from("2019-02-15");
        let positive = later.since(earlier, {
            smallestUnit: "months",
            roundingMode: "halfExpand",
        });
        let negative = earlier.since(later, {
            smallestUnit: "months",
            roundingMode: "halfExpand",
        });
        [
            positive.months,
            positive.days,
            negative.months,
            negative.days,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "1|0|-2|0");
}

#[test]
fn temporal_plain_date_since_balances_months_relative_to_receiver() {
    let result = compile_and_run_string_with_host(
        r#"
        let forward = Temporal.PlainDate.from("2019-03-01").since("2019-01-29", {
            largestUnit: "months",
        });
        let backward = Temporal.PlainDate.from("2019-01-29").since("2019-03-01", {
            largestUnit: "months",
        });
        [
            forward.months,
            forward.days,
            backward.months,
            backward.days,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "1|3|-1|-1");
}

#[test]
fn temporal_plain_date_difference_reads_options_before_validation() {
    let result = compile_and_run_string_with_host(
        r#"
        let log = [];
        let options = {
            get largestUnit() {
                log.push("get options.largestUnit");
                return {
                    toString() {
                        log.push("call options.largestUnit.toString");
                        return "hour";
                    }
                };
            },
            get roundingIncrement() {
                log.push("get options.roundingIncrement");
                return {
                    valueOf() {
                        log.push("call options.roundingIncrement.valueOf");
                        return 1;
                    }
                };
            },
            get roundingMode() {
                log.push("get options.roundingMode");
                return {
                    toString() {
                        log.push("call options.roundingMode.toString");
                        return "halfFloor";
                    }
                };
            },
            get smallestUnit() {
                log.push("get options.smallestUnit");
                return {
                    toString() {
                        log.push("call options.smallestUnit.toString");
                        return "nanosecond";
                    }
                };
            },
        };
        let threw = (() => {
            try {
                new Temporal.PlainDate(2025, 8, 14).since(new Temporal.PlainDate(2025, 3, 14), options);
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        })();
        [threw, log.join(",")].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "true|get options.largestUnit,call options.largestUnit.toString,get options.roundingIncrement,call options.roundingIncrement.valueOf,get options.roundingMode,call options.roundingMode.toString,get options.smallestUnit,call options.smallestUnit.toString"
    );
}

#[test]
fn temporal_plain_date_difference_reads_other_bag_in_spec_order() {
    let result = compile_and_run_string_with_host(
        r#"
        let log = [];
        let other = {
            get calendar() {
                log.push("get other.calendar");
                return "iso8601";
            },
            get day() {
                log.push("get other.day");
                return {
                    valueOf() {
                        log.push("call other.day.valueOf");
                        return 2;
                    }
                };
            },
            get month() {
                log.push("get other.month");
                return {
                    valueOf() {
                        log.push("call other.month.valueOf");
                        return 6;
                    }
                };
            },
            get monthCode() {
                log.push("get other.monthCode");
                return {
                    toString() {
                        log.push("call other.monthCode.toString");
                        return "M06";
                    }
                };
            },
            get year() {
                log.push("get other.year");
                return {
                    valueOf() {
                        log.push("call other.year.valueOf");
                        return 2001;
                    }
                };
            },
        };
        let options = {
            get largestUnit() {
                log.push("get options.largestUnit");
                return {
                    toString() {
                        log.push("call options.largestUnit.toString");
                        return "years";
                    }
                };
            },
            get roundingIncrement() {
                log.push("get options.roundingIncrement");
                return {
                    valueOf() {
                        log.push("call options.roundingIncrement.valueOf");
                        return 1;
                    }
                };
            },
            get roundingMode() {
                log.push("get options.roundingMode");
                return {
                    toString() {
                        log.push("call options.roundingMode.toString");
                        return "halfExpand";
                    }
                };
            },
            get smallestUnit() {
                log.push("get options.smallestUnit");
                return {
                    toString() {
                        log.push("call options.smallestUnit.toString");
                        return "days";
                    }
                };
            },
        };
        new Temporal.PlainDate(2000, 5, 2).since(other, options);
        log.join(",");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "get other.calendar,get other.day,call other.day.valueOf,get other.month,call other.month.valueOf,get other.monthCode,call other.monthCode.toString,get other.year,call other.year.valueOf,get options.largestUnit,call options.largestUnit.toString,get options.roundingIncrement,call options.roundingIncrement.valueOf,get options.roundingMode,call options.roundingMode.toString,get options.smallestUnit,call options.smallestUnit.toString"
    );
}

#[test]
fn temporal_plain_date_converts_to_partial_plain_dates() {
    let result = compile_and_run_string_with_host(
        r#"
        let date = new Temporal.PlainDate(2024, 2, 29);
        let yearMonth = date.toPlainYearMonth();
        let monthDay = date.toPlainMonthDay();
        [
            yearMonth instanceof Temporal.PlainYearMonth,
            yearMonth.toString(),
            monthDay instanceof Temporal.PlainMonthDay,
            monthDay.toString(),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|2024-02|true|02-29");
}

#[test]
fn temporal_plain_date_to_zoned_date_time_resolves_midnight_and_plain_time() {
    let result = compile_and_run_string_with_host(
        r#"
        let date = new Temporal.PlainDate(2020, 1, 1);
        let midnight = date.toZonedDateTime("UTC");
        let noon = date.toZonedDateTime({ timeZone: "UTC", plainTime: "12:00" });
        let partialTime = date.toZonedDateTime({
            timeZone: "UTC",
            plainTime: { minute: 30, microsecond: 555 }
        });
        [
            Temporal.PlainDate.prototype.toZonedDateTime.name,
            Temporal.PlainDate.prototype.toZonedDateTime.length,
            midnight.toString(),
            String(midnight.epochNanoseconds),
            noon.toString(),
            String(noon.epochNanoseconds),
            partialTime.toString(),
            String(partialTime.epochNanoseconds)
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "toZonedDateTime|1|2020-01-01T00:00:00+00:00[UTC]|1577836800000000000|2020-01-01T12:00:00+00:00[UTC]|1577880000000000000|2020-01-01T00:30:00.000555+00:00[UTC]|1577838600000555000"
    );
}

#[test]
fn temporal_plain_date_to_zoned_date_time_normalizes_time_zone_like_strings() {
    let result = compile_and_run_string_with_host(
        r#"
        let date = new Temporal.PlainDate(2020, 1, 1);
        let utc = date.toZonedDateTime("2021-08-19T17:30Z");
        let offset = date.toZonedDateTime("2021-08-19T17:30-07:00");
        let annotation = date.toZonedDateTime("2021-08-19T17:30-07:00[+01:46]");
        let wrongPrimitiveThrew = (() => {
            try {
                date.toZonedDateTime(1);
                return false;
            } catch (error) {
                return error.constructor === TypeError;
            }
        })();
        let wrongPropertyThrew = (() => {
            try {
                date.toZonedDateTime({ timeZone: {}, plainTime: "12:00" });
                return false;
            } catch (error) {
                return error.constructor === TypeError;
            }
        })();
        let subMinuteThrew = (() => {
            try {
                date.toZonedDateTime("-12:12:59.9");
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        })();
        let invalidYearZeroThrew = (() => {
            try {
                date.toZonedDateTime("-000000-10-31T17:45+00:00[UTC]");
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        })();
        [
            utc.timeZoneId,
            offset.timeZoneId,
            annotation.timeZoneId,
            wrongPrimitiveThrew,
            wrongPropertyThrew,
            subMinuteThrew,
            invalidYearZeroThrew
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "UTC|-07:00|+01:46|true|true|true|true");
}

#[test]
fn temporal_zoned_date_time_prototype_getters_are_redefinable_for_slot_fast_path_tests() {
    let result = compile_and_run_string_with_host(
        r#"
        let descriptor = Object.getOwnPropertyDescriptor(
            Temporal.ZonedDateTime.prototype,
            "year"
        );
        let redefined = (() => {
            try {
                Object.defineProperty(Temporal.ZonedDateTime.prototype, "year", {
                    get() { return 1; }
                });
                return true;
            } catch (error) {
                return error instanceof TypeError ? "TypeError" : error.name;
            }
        })();
        [
            descriptor && descriptor.configurable,
            descriptor && descriptor.enumerable,
            typeof descriptor.get,
            redefined,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|false|function|true");
}

#[test]
fn temporal_plain_date_from_zoned_date_time_uses_slots_without_getters() {
    let result = compile_and_run_string_with_host(
        r#"
        const actual = [];
        const prototypeDescrs = Object.getOwnPropertyDescriptors(Temporal.ZonedDateTime.prototype);
        const getters = ["year", "month", "monthCode", "day", "hour", "minute", "second", "millisecond", "microsecond", "nanosecond", "calendar"];

        for (const property of getters) {
          Object.defineProperty(Temporal.ZonedDateTime.prototype, property, {
            get() {
              actual.push(`get ${property}`);
              const value = prototypeDescrs[property].get.call(this);
              return {
                toString() {
                  actual.push(`toString ${property}`);
                  return value.toString();
                },
                valueOf() {
                  actual.push(`valueOf ${property}`);
                  return value;
                },
              };
            },
          });
        }

        const arg = new Temporal.ZonedDateTime(0n, "UTC");
        Temporal.PlainDate.from(arg);
        actual.join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "");
}

#[test]
fn temporal_zoned_date_time_slot_fast_path_probe() {
    let result = compile_and_run_string_with_host(
        r#"
        const prototypeDescrs = Object.getOwnPropertyDescriptors(Temporal.ZonedDateTime.prototype);
        const getters = ["year", "month", "monthCode", "day", "hour", "minute", "second", "millisecond", "microsecond", "nanosecond", "calendar"];
        let outcome = "ok";
        for (const property of getters) {
          const descriptor = prototypeDescrs[property];
          if (!descriptor) {
            outcome = property + "|missing";
            break;
          }
          if (typeof descriptor.get !== "function") {
            outcome = property + "|no-getter|" + typeof descriptor.get;
            break;
          }
          if (descriptor.configurable !== true) {
            outcome = property + "|configurable:" + descriptor.configurable;
            break;
          }
          try {
            Object.defineProperty(Temporal.ZonedDateTime.prototype, property, {
              get() { return descriptor.get.call(this); }
            });
          } catch (error) {
            outcome = property + "|" + error.name;
            break;
          }
        }
        outcome;
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "ok");
}
