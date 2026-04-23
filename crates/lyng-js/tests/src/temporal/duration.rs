use super::compile_and_run_string_with_host;

#[test]
fn temporal_duration_constructor_getters_and_stringification() {
    let result = compile_and_run_string_with_host(
        r#"
        let duration = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
        [
            duration.years,
            duration.months,
            duration.weeks,
            duration.days,
            duration.hours,
            duration.minutes,
            duration.seconds,
            duration.milliseconds,
            duration.microseconds,
            duration.nanoseconds,
            duration.sign,
            duration.blank,
            duration.toString(),
            Object.prototype.toString.call(duration),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "1|2|3|4|5|6|7|8|9|10|1|false|P1Y2M3W4DT5H6M7.00800901S|[object Temporal.Duration]"
    );
}

#[test]
fn temporal_duration_zero_and_value_of_behavior() {
    let result = compile_and_run_string_with_host(
        r#"
        let duration = new Temporal.Duration();
        let threw = (() => {
            try {
                duration.valueOf();
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        [duration.sign, duration.blank, duration.toString(), threw].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "0|true|PT0S|true");
}

#[test]
fn temporal_duration_relational_comparison_invokes_throwing_value_of() {
    let result = compile_and_run_string_with_host(
        r#"
        function throwsTypeError(callback) {
            try {
                callback();
                return false;
            } catch (error) {
                return error.constructor === TypeError;
            }
        }
        let duration = Temporal.Duration.from("P3DT1H");
        [
            throwsTypeError(() => duration.valueOf()),
            throwsTypeError(() => duration < duration),
            throwsTypeError(() => duration <= duration),
            throwsTypeError(() => duration > duration),
            throwsTypeError(() => duration >= duration),
            duration === duration,
            duration !== duration
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|true|true|true|true|true|false");
}

#[test]
fn temporal_duration_to_json_matches_to_string() {
    let result = compile_and_run_string_with_host(
        r#"
        let duration = new Temporal.Duration(0, 0, 0, 1, 2, 3, 4, 5, 6, 7);
        [duration.toString(), duration.toJSON()].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "P1DT2H3M4.005006007S|P1DT2H3M4.005006007S");
}

#[test]
fn temporal_duration_to_locale_string_matches_non_intl_to_string_shape() {
    let result = compile_and_run_string_with_host(
        r#"
        let duration = new Temporal.Duration(0, 0, 0, 1, 2, 3, 4, 5, 6, 7);
        let descriptor = Object.getOwnPropertyDescriptor(Temporal.Duration.prototype, "toLocaleString");
        let brandThrew = (() => {
            try {
                Temporal.Duration.prototype.toLocaleString.call({});
                return false;
            } catch (error) {
                return error.constructor === TypeError;
            }
        })();
        [
            typeof Temporal.Duration.prototype.toLocaleString,
            Temporal.Duration.prototype.toLocaleString.length,
            duration.toLocaleString(),
            descriptor.writable,
            descriptor.enumerable,
            descriptor.configurable,
            brandThrew
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "function|0|P1DT2H3M4.005006007S|true|false|true|true"
    );
}

#[test]
fn temporal_duration_to_string_honors_seconds_precision_options() {
    let result = compile_and_run_string_with_host(
        r#"
        let duration = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 987, 650, 0);
        let rounded = new Temporal.Duration(0, 0, 0, 0, 1, 59, 59, 900);
        let roundedSeconds = new Temporal.Duration(0, 0, 0, 0, 0, 0, 59, 900);
        let blank = new Temporal.Duration();
        let badDigitsThrew = (() => {
            try {
                duration.toString({ fractionalSecondDigits: 10 });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let badUnitThrew = (() => {
            try {
                duration.toString({ smallestUnit: "minute" });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let badRoundingThrew = (() => {
            try {
                duration.toString({ smallestUnit: "microsecond", roundingMode: "halfexpand" });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        [
            duration.toString({ fractionalSecondDigits: 2 }),
            duration.toString({ smallestUnit: "microsecond" }),
            duration.toString({ fractionalSecondDigits: 0 }),
            blank.toString({ smallestUnit: "milliseconds" }),
            rounded.toString({ fractionalSecondDigits: 0, roundingMode: "expand" }),
            roundedSeconds.toString({ fractionalSecondDigits: 0, roundingMode: "expand" }),
            badDigitsThrew,
            badUnitThrew,
            badRoundingThrew
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "P1Y2M3W4DT5H6M7.98S|P1Y2M3W4DT5H6M7.987650S|P1Y2M3W4DT5H6M7S|PT0.000S|PT2H0S|PT60S|true|true|true"
    );
}

#[test]
fn temporal_duration_to_string_reads_and_coerces_options_in_spec_order() {
    let result = compile_and_run_string_with_host(
        r#"
        let actual = [];
        let options = {};
        function observed(name, value) {
            return {
                toString: function() {
                    actual.push("call " + name + ".toString");
                    return value;
                }
            };
        }
        Object.defineProperty(options, "fractionalSecondDigits", {
            get: function() {
                actual.push("get fractionalSecondDigits");
                return observed("fractionalSecondDigits", "auto");
            }
        });
        Object.defineProperty(options, "roundingMode", {
            get: function() {
                actual.push("get roundingMode");
                return observed("roundingMode", "halfExpand");
            }
        });
        Object.defineProperty(options, "smallestUnit", {
            get: function() {
                actual.push("get smallestUnit");
                return observed("smallestUnit", "millisecond");
            }
        });
        new Temporal.Duration(0, 0, 0, 0, 0, 0, 1, 234).toString(options);
        actual.join(",");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "get fractionalSecondDigits,call fractionalSecondDigits.toString,get roundingMode,call roundingMode.toString,get smallestUnit,call smallestUnit.toString"
    );
}

#[test]
fn temporal_duration_negated_and_abs_return_duration_instances() {
    let result = compile_and_run_string_with_host(
        r#"
        let negative = new Temporal.Duration(-1, -2, -3, -4, -5, -6, -7, -8, -9, -10);
        let positive = new Temporal.Duration(0, 0, 0, 1, 2, 3, 4, 5, 6, 7);
        let negated = negative.negated();
        let absolute = negative.abs();
        let positiveAbs = positive.abs();
        let zeroNegated = new Temporal.Duration().negated();
        [
            negated instanceof Temporal.Duration,
            negated.toString(),
            absolute.toString(),
            positiveAbs.toString(),
            zeroNegated.toString(),
            negative.toString(),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "true|P1Y2M3W4DT5H6M7.00800901S|P1Y2M3W4DT5H6M7.00800901S|P1DT2H3M4.005006007S|PT0S|-P1Y2M3W4DT5H6M7.00800901S"
    );
}

#[test]
fn temporal_duration_from_clones_duration_and_normalizes_property_bags() {
    let result = compile_and_run_string_with_host(
        r#"
        let existing = new Temporal.Duration(1, 2, 0, 4);
        let clone = Temporal.Duration.from(existing);
        let bag = Temporal.Duration.from({
            hours: 2,
            minutes: 30,
            nanoseconds: 5,
        });
        let unbalanced = Temporal.Duration.from({
            milliseconds: 1000,
            month: 1
        });
        [
            clone !== existing,
            clone.toString(),
            bag.years,
            bag.hours,
            bag.minutes,
            bag.nanoseconds,
            bag.toString(),
            unbalanced.seconds,
            unbalanced.milliseconds,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|P1Y2M4D|0|2|30|5|PT2H30M0.000000005S|0|1000");
}

#[test]
fn temporal_duration_from_reads_and_converts_property_bags_in_spec_order() {
    let result = compile_and_run_string_with_host(
        r#"
        let actual = [];
        let fields = {};
        function observed(name) {
            return {
                valueOf: function() {
                    actual.push("call " + name + ".valueOf");
                    return 1;
                }
            };
        }
        function install(name) {
            Object.defineProperty(fields, name, {
                get: function() {
                    actual.push("get " + name);
                    return observed(name);
                }
            });
        }
        [
            "days",
            "hours",
            "microseconds",
            "milliseconds",
            "minutes",
            "months",
            "nanoseconds",
            "seconds",
            "weeks",
            "years"
        ].forEach(install);
        let duration = Temporal.Duration.from(fields);
        duration.toString() + "|" + actual.join(",");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "P1Y1M1W1DT1H1M1.001001001S|get days,call days.valueOf,get hours,call hours.valueOf,get microseconds,call microseconds.valueOf,get milliseconds,call milliseconds.valueOf,get minutes,call minutes.valueOf,get months,call months.valueOf,get nanoseconds,call nanoseconds.valueOf,get seconds,call seconds.valueOf,get weeks,call weeks.valueOf,get years,call years.valueOf"
    );
}

#[test]
fn temporal_duration_from_balances_large_exact_subsecond_property_bag_values() {
    let result = compile_and_run_string_with_host(
        r#"
        let positive = Temporal.Duration.from({
            milliseconds: 4503599627370497_000,
            microseconds: 4503599627370495_000000
        });
        let negative = Temporal.Duration.from({
            milliseconds: -4503599627370497_000,
            microseconds: -4503599627370495_000000
        });
        positive.toString() + "|" + negative.toString();
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "PT9007199254740991.975424S|-PT9007199254740991.975424S"
    );
}

#[test]
fn temporal_duration_from_parses_iso_duration_strings() {
    let result = compile_and_run_string_with_host(
        r#"
        let mixed = Temporal.Duration.from("P1Y2M3W4DT5H6M7.00800901S");
        let negative = Temporal.Duration.from("-PT2H20M30S");
        let zero = Temporal.Duration.from("PT0S");
        [
            mixed instanceof Temporal.Duration,
            mixed.years,
            mixed.months,
            mixed.weeks,
            mixed.days,
            mixed.hours,
            mixed.minutes,
            mixed.seconds,
            mixed.milliseconds,
            mixed.microseconds,
            mixed.nanoseconds,
            mixed.toString(),
            negative.toString(),
            negative.blank,
            zero.toString(),
            zero.blank,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "true|1|2|3|4|5|6|7|8|9|10|P1Y2M3W4DT5H6M7.00800901S|-PT2H20M30S|false|PT0S|true"
    );
}

#[test]
fn temporal_duration_from_rejects_invalid_strings_and_empty_property_bags() {
    let result = compile_and_run_string_with_host(
        r#"
        let invalidStringThrew = (() => {
            try {
                Temporal.Duration.from("P");
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let fractionalNonSecondsThrew = (() => {
            try {
                Temporal.Duration.from("PT0.5H5S");
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let emptyBagThrew = (() => {
            try {
                Temporal.Duration.from({});
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        [invalidStringThrew, fractionalNonSecondsThrew, emptyBagThrew].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|true|true");
}

#[test]
fn temporal_duration_from_non_string_inputs_throw_type_error_for_all_options() {
    let result = compile_and_run_string_with_host(
        r#"
        function throwsTypeError(callback) {
            try {
                callback();
                return "none";
            } catch (error) {
                return String(error.constructor === TypeError);
            }
        }

        function check(value) {
            return [
                throwsTypeError(() => Temporal.Duration.from(value)),
                throwsTypeError(() => Temporal.Duration.from(value, undefined)),
                throwsTypeError(() => Temporal.Duration.from(value, { overflow: "constrain" })),
                throwsTypeError(() => Temporal.Duration.from(value, { overflow: "reject" }))
            ].join(",");
        }

        [
            check(undefined),
            check(null),
            check(true),
            check(1),
            check(19761118),
            check(1n),
            check(Symbol()),
            check({}),
            check(Temporal.Duration),
            check(Temporal.Duration.prototype)
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "true,true,true,true|true,true,true,true|true,true,true,true|true,true,true,true|true,true,true,true|true,true,true,true|true,true,true,true|true,true,true,true|true,true,true,true|true,true,true,true"
    );
}

#[test]
fn temporal_duration_compare_orders_time_only_durations_and_coerces_arguments() {
    let result = compile_and_run_string_with_host(
        r#"
        [
            typeof Temporal.Duration.compare,
            Temporal.Duration.compare(new Temporal.Duration(0, 0, 0, 0, 5, 5, 5, 5, 5, 5), new Temporal.Duration(0, 0, 0, 0, 5, 4, 5, 5, 5, 5)),
            Temporal.Duration.compare(new Temporal.Duration(0, 0, 0, 0, -5, -4, -5, -5, -5, -5), new Temporal.Duration(0, 0, 0, 0, -5, -5, -5, -5, -5, -5)),
            Temporal.Duration.compare("PT12H", new Temporal.Duration()),
            Temporal.Duration.compare(new Temporal.Duration(), { minutes: 60 }),
            Temporal.Duration.compare({ hours: 1 }, { minutes: 60 }, {}),
            Temporal.Duration.compare(new Temporal.Duration(0, 0, 0, 1), new Temporal.Duration(0, 0, 0, 2)),
            Temporal.Duration.compare(new Temporal.Duration(0, 0, 0, 200), new Temporal.Duration(0, 0, 0, 200, 0, 0, 0, 0, 0, 1)),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "function|1|1|1|-1|0|-1|-1");
}

#[test]
fn temporal_duration_compare_validates_options_before_equal_early_return() {
    let result = compile_and_run_string_with_host(
        r#"
        let duration = Temporal.Duration.from({ hours: 1 });
        let values = [null, true, "options", 1, Symbol()];
        let count = 0;
        for (let value of values) {
            try {
                Temporal.Duration.compare(duration, duration, value);
            } catch (error) {
                count += error.constructor === TypeError ? 1 : 0;
            }
        }
        String(count);
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "5");
}

#[test]
fn temporal_duration_compare_uses_relative_to_for_matching_calendar_units() {
    let result = compile_and_run_string_with_host(
        r#"
        let relativeTo = new Temporal.PlainDate(2017, 1, 1);
        let larger = new Temporal.Duration(5, 5, 5, 5, 5, 5, 5, 5, 5, 5);
        let smaller = new Temporal.Duration(5, 5, 5, 5, 5, 4, 5, 5, 5, 5);
        [
            Temporal.Duration.compare(larger, larger, { relativeTo }),
            Temporal.Duration.compare(smaller, larger, { relativeTo }),
            Temporal.Duration.compare(larger, smaller, { relativeTo })
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "0|-1|1");
}

#[test]
fn temporal_duration_compare_round_and_total_support_zoned_relative_to() {
    let result = compile_and_run_string_with_host(
        r#"
        let relativeTo = Temporal.ZonedDateTime.from("2020-02-01T00:00[UTC]");
        let month = Temporal.Duration.from("P1M");
        let rounded = month.round({
            relativeTo,
            largestUnit: "day",
            smallestUnit: "day",
        });
        [
            Temporal.Duration.compare(month, Temporal.Duration.from("P29D"), { relativeTo }),
            month.total({ relativeTo, unit: "day" }),
            month.total({
                relativeTo: "2020-02-01T00:00[UTC]",
                unit: "day",
            }),
            month.total({
                relativeTo: {
                    year: 2020,
                    month: 2,
                    day: 1,
                    timeZone: "UTC",
                },
                unit: "day",
            }),
            rounded.toString(),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "0|29|29|29|P29D");
}

#[test]
fn temporal_duration_compare_rejects_calendar_units_without_relative_to() {
    let result = compile_and_run_string_with_host(
        r#"
        let identical = Temporal.Duration.compare(
            new Temporal.Duration(5, 5, 5, 5, 5, 5, 5, 5, 5, 5),
            new Temporal.Duration(5, 5, 5, 5, 5, 5, 5, 5, 5, 5)
        );
        let nonIdenticalThrew = (() => {
            try {
                Temporal.Duration.compare(new Temporal.Duration(1), new Temporal.Duration(0, 12));
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        [identical, nonIdenticalThrew].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "0|true");
}

#[test]
fn temporal_duration_add_and_subtract_balance_duration_parts() {
    let result = compile_and_run_string_with_host(
        r#"
        let base = Temporal.Duration.from({ days: 1, minutes: 5 });
        let balanced = Temporal.Duration.from("P50DT50H50M50.500500500S");
        let subtractBase = Temporal.Duration.from({ days: 3, hours: 1, minutes: 10 });
        [
            typeof Temporal.Duration.prototype.add,
            typeof Temporal.Duration.prototype.subtract,
            base.add({ days: 2, minutes: 5 }).toString(),
            base.add({ hours: 12, seconds: 30 }).toString(),
            balanced.add(balanced).toString(),
            Temporal.Duration.from({ hours: -1, seconds: -60 }).add({ minutes: 122 }).toString(),
            subtractBase.subtract({ minutes: 15 }).toString(),
            subtractBase.subtract(subtractBase).toString(),
            Temporal.Duration.from({ hours: 1, seconds: 3721 }).subtract({ minutes: 61, nanoseconds: 3722000000001 }).toString(),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "function|function|P3DT10M|P1DT12H5M30S|P104DT5H41M41.001001S|PT1H1M|P3DT55M|PT0S|-PT1M1.000000001S"
    );
}

#[test]
fn temporal_duration_add_and_subtract_validate_argument_signs() {
    let result = compile_and_run_string_with_host(
        r#"
        let addMixedThrew = (() => {
            try {
                new Temporal.Duration(0, 0, 0, 1).add({ hours: 1, minutes: -30 });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let subtractMixedThrew = (() => {
            try {
                new Temporal.Duration(0, 0, 0, 1).subtract({ hours: 1, minutes: -30 });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let addNumberThrew = (() => {
            try {
                new Temporal.Duration().add(7);
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        let subtractEmptyStringThrew = (() => {
            try {
                new Temporal.Duration().subtract("");
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        [addMixedThrew, subtractMixedThrew, addNumberThrew, subtractEmptyStringThrew].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|true|true|true");
}

#[test]
fn temporal_duration_with_merges_partial_duration_fields() {
    let result = compile_and_run_string_with_host(
        r#"
        let duration = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
        let merged = duration.with({ years: 9, minutes: 4, nanoseconds: 123 });
        let replacedSign = Temporal.Duration.from({ years: 5, days: 1 })
            .with({ years: -1, days: 0, minutes: -1 });
        let primitiveThrew = (() => {
            try {
                duration.with("P1D");
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        let emptyThrew = (() => {
            try {
                duration.with({});
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        let mixedThrew = (() => {
            try {
                duration.with({ hours: 1, minutes: -30 });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        [
            typeof Temporal.Duration.prototype.with,
            Temporal.Duration.prototype.with.length,
            merged instanceof Temporal.Duration,
            merged !== duration,
            merged.toString(),
            replacedSign.toString(),
            primitiveThrew,
            emptyThrew,
            mixedThrew
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "function|1|true|true|P9Y2M3W4DT5H4M7.008009123S|-P1YT1M|true|true|true"
    );
}

#[test]
fn temporal_duration_round_balances_exact_time_units() {
    let result = compile_and_run_string_with_host(
        r#"
        let subsecond = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 999, 999999, 999999999);
        let negative = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, -999, -999999, -999999999);
        let roundedUp = new Temporal.Duration(0, 0, 0, 0, 1, 59, 59, 900);
        let blank = new Temporal.Duration();
        [
            typeof Temporal.Duration.prototype.round,
            Temporal.Duration.prototype.round.length,
            subsecond.round({ largestUnit: "seconds" }).toString(),
            negative.round({ largestUnit: "seconds" }).toString(),
            roundedUp.round({ smallestUnit: "minute", roundingMode: "ceil" }).toString(),
            blank.round("seconds").toString(),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "function|1|PT2.998998999S|-PT2.998998999S|PT2H|PT0S"
    );
}

#[test]
fn temporal_duration_total_supports_exact_time_units() {
    let result = compile_and_run_string_with_host(
        r#"
        let duration = new Temporal.Duration(0, 0, 0, 5, 5, 5, 5, 5, 5, 5);
        let subsecond = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 999, 999999, 999999999);
        let negative = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, -999, -999999, -999999999);
        let blank = new Temporal.Duration();
        [
            typeof Temporal.Duration.prototype.total,
            Temporal.Duration.prototype.total.length,
            duration.total("seconds"),
            duration.total({ unit: "milliseconds" }),
            subsecond.total("seconds"),
            negative.total("seconds"),
            blank.total("hours"),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "function|1|450305.005005005|450305005.005005|2.998998999|-2.998998999|0"
    );
}

#[test]
fn temporal_duration_round_and_total_use_iso_relative_to_for_calendar_units() {
    let result = compile_and_run_string_with_host(
        r#"
        let february = Temporal.Duration.from("P1M");
        let roundedDate = february.round({
            relativeTo: new Temporal.PlainDate(2020, 2, 1),
            largestUnit: "day",
            smallestUnit: "day"
        });
        let roundedDateTime = Temporal.Duration.from("P1DT12H").round({
            relativeTo: new Temporal.PlainDateTime(2020, 1, 1, 12),
            largestUnit: "hour",
            smallestUnit: "hour"
        });
        [
            february.total({ relativeTo: "2020-02-01", unit: "day" }),
            february.total({ relativeTo: new Temporal.PlainDate(2021, 2, 1), unit: "day" }),
            Temporal.Duration.from("P1MT12H").total({
                relativeTo: new Temporal.PlainDateTime(2020, 2, 1, 12),
                unit: "hour"
            }),
            roundedDate.toString(),
            roundedDateTime.toString()
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "29|28|708|P29D|PT36H");
}

#[test]
fn temporal_duration_rejects_out_of_range_components_after_balancing() {
    let result = compile_and_run_string_with_host(
        r#"
        let yearsThrew = (() => {
            try {
                new Temporal.Duration(4294967296);
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let maxYears = new Temporal.Duration(4294967295).years;
        let daysThrew = (() => {
            try {
                new Temporal.Duration(0, 0, 0, 104249991375);
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let balancedSecondsThrew = (() => {
            try {
                Temporal.Duration.from({ seconds: 9007199254740991, milliseconds: 1000 });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let maxTime = Temporal.Duration.from({
            days: 104249991374,
            nanoseconds: 27391999999999
        }).sign;
        [yearsThrew, maxYears, daysThrew, balancedSecondsThrew, maxTime].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|4294967295|true|true|1");
}
