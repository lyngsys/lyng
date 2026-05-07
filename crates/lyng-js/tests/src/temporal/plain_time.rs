use super::compile_and_run_string_with_host;

#[test]
fn temporal_plain_time_constructor_getters_and_serialization() {
    let result = compile_and_run_string_with_host(
        r#"
        let time = new Temporal.PlainTime(23, 59, 58, 123, 456, 789);
        let truncated = new Temporal.PlainTime(11.9, "12.8", 13.7, 14.6, 15.5, 1.999999);
        let threw = (() => {
            try {
                time.valueOf();
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        [
            time.hour,
            time.minute,
            time.second,
            time.millisecond,
            time.microsecond,
            time.nanosecond,
            time.toString(),
            time.toJSON(),
            truncated.toString(),
            Object.prototype.toString.call(time),
            threw,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "23|59|58|123|456|789|23:59:58.123456789|23:59:58.123456789|11:12:13.014015001|[object Temporal.PlainTime]|true"
    );
}

#[test]
fn temporal_plain_time_to_locale_string_matches_non_intl_to_string_shape() {
    let result = compile_and_run_string_with_host(
        r#"
        let time = new Temporal.PlainTime(12, 34, 56, 987, 654, 321);
        let descriptor = Object.getOwnPropertyDescriptor(Temporal.PlainTime.prototype, "toLocaleString");
        let brandThrew = (() => {
            try {
                Temporal.PlainTime.prototype.toLocaleString.call({});
                return false;
            } catch (error) {
                return error.constructor === TypeError;
            }
        })();
        [
            typeof Temporal.PlainTime.prototype.toLocaleString,
            Temporal.PlainTime.prototype.toLocaleString.length,
            time.toLocaleString(),
            descriptor.writable,
            descriptor.enumerable,
            descriptor.configurable,
            brandThrew
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "function|0|12:34:56.987654321|true|false|true|true");
}

#[test]
fn temporal_plain_time_from_clones_times_and_normalizes_property_bags() {
    let result = compile_and_run_string_with_host(
        r#"
        let existing = new Temporal.PlainTime(12, 34, 56);
        let dateTime = new Temporal.PlainDateTime(2024, 5, 6, 7, 8, 9, 10, 11, 12);
        let getterCalls = [];
        ["hour", "minute", "second", "millisecond", "microsecond", "nanosecond"].forEach((property) => {
            Object.defineProperty(dateTime, property, {
                get() {
                    getterCalls.push(property);
                    return 99;
                }
            });
        });
        let clone = Temporal.PlainTime.from(existing);
        let fromDateTime = Temporal.PlainTime.from(dateTime);
        let bag = Temporal.PlainTime.from({
            hour: 1,
            minute: 2,
            nanosecond: 3,
        });
        let leap = Temporal.PlainTime.from({
            hour: 23,
            minute: 59,
            second: 60,
        });
        let constrained = Temporal.PlainTime.from({ hour: 26, minute: -1 });
        let rejectThrew = (() => {
            try {
                Temporal.PlainTime.from({ second: 60 }, { overflow: "reject" });
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        })();
        [
            clone !== existing,
            clone.toString(),
            fromDateTime.toString(),
            getterCalls.join(","),
            bag.hour,
            bag.minute,
            bag.second,
            bag.nanosecond,
            bag.toString(),
            leap.second,
            leap.toString(),
            constrained.toString(),
            rejectThrew,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "true|12:34:56|07:08:09.010011012||1|2|0|3|01:02:00.000000003|59|23:59:59|23:00:00|true"
    );
}

#[test]
fn temporal_plain_time_from_reads_fields_before_options_and_clones_zoned_values() {
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

        let actual = [];
        let fields = {
            get hour() {
                actual.push("get fields.hour");
                return observedInteger(actual, "fields.hour", 1.7);
            },
            get microsecond() {
                actual.push("get fields.microsecond");
                return observedInteger(actual, "fields.microsecond", 1.7);
            },
            get millisecond() {
                actual.push("get fields.millisecond");
                return observedInteger(actual, "fields.millisecond", 1.7);
            },
            get minute() {
                actual.push("get fields.minute");
                return observedInteger(actual, "fields.minute", 1.7);
            },
            get nanosecond() {
                actual.push("get fields.nanosecond");
                return observedInteger(actual, "fields.nanosecond", 1.7);
            },
            get second() {
                actual.push("get fields.second");
                return observedInteger(actual, "fields.second", 1.7);
            },
            calendar: "iso8601",
        };
        let options = {
            get overflow() {
                actual.push("get options.overflow");
                return observedOverflow(actual);
            }
        };
        Temporal.PlainTime.from(fields, options);
        let fieldsOrder = actual.join(",");

        actual = [];
        Temporal.PlainTime.from(new Temporal.PlainTime(12, 34), options);
        let plainTimeOrder = actual.join(",");

        actual = [];
        Temporal.PlainTime.from(new Temporal.PlainDateTime(2000, 5, 2, 12, 34), options);
        let plainDateTimeOrder = actual.join(",");

        actual = [];
        let zoned = new Temporal.ZonedDateTime(0n, "UTC");
        let fromZoned = Temporal.PlainTime.from(zoned, options);
        let zonedOrder = actual.join(",");

        actual = [];
        Temporal.PlainTime.from("12:34", options);
        let stringOrder = actual.join(",");

        [
            fieldsOrder,
            plainTimeOrder,
            plainDateTimeOrder,
            fromZoned.toString(),
            zonedOrder,
            stringOrder,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "get fields.hour,get fields.hour.valueOf,call fields.hour.valueOf,get fields.microsecond,get fields.microsecond.valueOf,call fields.microsecond.valueOf,get fields.millisecond,get fields.millisecond.valueOf,call fields.millisecond.valueOf,get fields.minute,get fields.minute.valueOf,call fields.minute.valueOf,get fields.nanosecond,get fields.nanosecond.valueOf,call fields.nanosecond.valueOf,get fields.second,get fields.second.valueOf,call fields.second.valueOf,get options.overflow,get options.overflow.toString,call options.overflow.toString|get options.overflow,get options.overflow.toString,call options.overflow.toString|get options.overflow,get options.overflow.toString,call options.overflow.toString|00:00:00|get options.overflow,get options.overflow.toString,call options.overflow.toString|get options.overflow,get options.overflow.toString,call options.overflow.toString"
    );
}

#[test]
fn temporal_plain_time_with_replaces_time_fields() {
    let result = compile_and_run_string_with_host(
        r#"
        let time = new Temporal.PlainTime(23, 59, 58, 123, 456, 789);
        let changed = time.with({ hour: 1, minute: 2, nanosecond: 3 });
        let constrained = time.with({ minute: 67 });
        let rejectThrew = (() => {
            try {
                time.with({ minute: 67 }, { overflow: "reject" });
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        })();
        let invalidThrew = (() => {
            try {
                time.with({ hour: Infinity });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        [
            changed instanceof Temporal.PlainTime,
            changed.toString(),
            constrained.toString(),
            time.toString(),
            rejectThrew,
            invalidThrew,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "true|01:02:58.123456003|23:59:58.123456789|23:59:58.123456789|true|true"
    );
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "single Temporal fixture preserves field and option access ordering"
)]
fn temporal_plain_time_with_rejects_invalid_bags_and_reads_fields_before_options() {
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

        let time = new Temporal.PlainTime(12, 34, 56, 987, 654, 321);
        let invalidChecks = [
            (() => {
                try {
                    time.with({});
                    return false;
                } catch (error) {
                    return error.constructor === TypeError;
                }
            })(),
            (() => {
                try {
                    time.with({ hours: 14 });
                    return false;
                } catch (error) {
                    return error.constructor === TypeError;
                }
            })(),
            (() => {
                try {
                    time.with({ hour: 14, calendar: "iso8601" });
                    return false;
                } catch (error) {
                    return error.constructor === TypeError;
                }
            })(),
            (() => {
                try {
                    time.with({ hour: 14, timeZone: "UTC" });
                    return false;
                } catch (error) {
                    return error.constructor === TypeError;
                }
            })(),
            (() => {
                try {
                    time.with(new Temporal.PlainTime(1, 2, 3));
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
            get hour() {
                actual.push("get fields.hour");
                return observedInteger(actual, "fields.hour", 1.7);
            },
            get microsecond() {
                actual.push("get fields.microsecond");
                return observedInteger(actual, "fields.microsecond", 1.7);
            },
            get millisecond() {
                actual.push("get fields.millisecond");
                return observedInteger(actual, "fields.millisecond", 1.7);
            },
            get minute() {
                actual.push("get fields.minute");
                return observedInteger(actual, "fields.minute", 1.7);
            },
            get nanosecond() {
                actual.push("get fields.nanosecond");
                return observedInteger(actual, "fields.nanosecond", 1.7);
            },
            get second() {
                actual.push("get fields.second");
                return observedInteger(actual, "fields.second", 1.7);
            },
        };
        let options = {
            get overflow() {
                actual.push("get options.overflow");
                return observedOverflow(actual);
            }
        };
        time.with(fields, options);
        invalidChecks + "|" + actual.join(",");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "true,true,true,true,true|get fields.calendar,get fields.timeZone,get fields.hour,get fields.hour.valueOf,call fields.hour.valueOf,get fields.microsecond,get fields.microsecond.valueOf,call fields.microsecond.valueOf,get fields.millisecond,get fields.millisecond.valueOf,call fields.millisecond.valueOf,get fields.minute,get fields.minute.valueOf,call fields.minute.valueOf,get fields.nanosecond,get fields.nanosecond.valueOf,call fields.nanosecond.valueOf,get fields.second,get fields.second.valueOf,call fields.second.valueOf,get options.overflow,get options.overflow.toString,call options.overflow.toString"
    );
}

#[test]
fn temporal_plain_time_add_balances_sub_day_units() {
    let result = compile_and_run_string_with_host(
        r#"
        let time = new Temporal.PlainTime(15, 23, 30, 123, 456, 789);
        let wrapped = time.add({ hours: 16 });
        let borrowed = new Temporal.PlainTime(1, 1, 1, 1, 1, 1).add({ nanoseconds: -2 });
        let ignored = time.add({ years: 1, months: 1, weeks: 1, days: 1 });
        [
            wrapped instanceof Temporal.PlainTime,
            wrapped.toString(),
            borrowed.toString(),
            ignored.toString(),
            time.toString(),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "true|07:23:30.123456789|01:01:01.001000999|15:23:30.123456789|15:23:30.123456789"
    );
}

#[test]
fn temporal_plain_time_string_conversion_rounding_and_difference_are_exact_time_operations() {
    let result = compile_and_run_string_with_host(
        r#"
        let time = Temporal.PlainTime.from("15:23:30.123456789");
        let other = Temporal.PlainTime.from("10:20:30.223456000");
        let subtracted = time.subtract("PT5H3M0.1S");
        let rounded = time.round({ smallestUnit: "millisecond", roundingMode: "halfExpand" });
        let roundedFromString = time.round("second");
        let since = time.since(other);
        let until = other.until(time, { smallestUnit: "millisecond", roundingMode: "halfExpand" });
        [
            Temporal.PlainTime.prototype.subtract.name,
            Temporal.PlainTime.prototype.subtract.length,
            Temporal.PlainTime.prototype.round.name,
            Temporal.PlainTime.prototype.round.length,
            Temporal.PlainTime.prototype.since.name,
            Temporal.PlainTime.prototype.since.length,
            Temporal.PlainTime.prototype.until.name,
            Temporal.PlainTime.prototype.until.length,
            time.toString(),
            Temporal.PlainTime.compare("15:23:30.123456789", time),
            time.equals("15:23:30.123456789"),
            subtracted.toString(),
            rounded.toString(),
            roundedFromString.toString(),
            since.hours,
            since.minutes,
            since.seconds,
            since.milliseconds,
            since.microseconds,
            since.nanoseconds,
            until.hours,
            until.minutes,
            until.seconds,
            until.milliseconds,
            until.microseconds,
            until.nanoseconds,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "subtract|1|round|1|since|1|until|1|15:23:30.123456789|0|true|10:20:30.023456789|15:23:30.123|15:23:30|5|2|59|900|0|789|5|2|59|900|0|0"
    );
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "single Temporal fixture preserves other-value and options access ordering"
)]
fn temporal_plain_time_difference_reads_other_then_options_and_delays_validation() {
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

        function observedString(log, label, value) {
            return Object.defineProperty({}, "toString", {
                get() {
                    log.push(`get ${label}.toString`);
                    return function () {
                        log.push(`call ${label}.toString`);
                        return value;
                    };
                }
            });
        }

        let actual = [];
        let instance = new Temporal.PlainTime(12, 34, 56, 987, 654, 321);
        let other = {
            get hour() {
                actual.push("get other.hour");
                return observedInteger(actual, "other.hour", 1.7);
            },
            get microsecond() {
                actual.push("get other.microsecond");
                return observedInteger(actual, "other.microsecond", 1.7);
            },
            get millisecond() {
                actual.push("get other.millisecond");
                return observedInteger(actual, "other.millisecond", 1.7);
            },
            get minute() {
                actual.push("get other.minute");
                return observedInteger(actual, "other.minute", 1.7);
            },
            get nanosecond() {
                actual.push("get other.nanosecond");
                return observedInteger(actual, "other.nanosecond", 1.7);
            },
            get second() {
                actual.push("get other.second");
                return observedInteger(actual, "other.second", 1.7);
            },
            calendar: "iso8601",
        };
        let options = {
            get largestUnit() {
                actual.push("get options.largestUnit");
                return observedString(actual, "options.largestUnit", "hours");
            },
            get roundingIncrement() {
                actual.push("get options.roundingIncrement");
                return observedInteger(actual, "options.roundingIncrement", 1);
            },
            get roundingMode() {
                actual.push("get options.roundingMode");
                return observedString(actual, "options.roundingMode", "trunc");
            },
            get smallestUnit() {
                actual.push("get options.smallestUnit");
                return observedString(actual, "options.smallestUnit", "nanoseconds");
            },
            additional: true,
        };
        instance.since(other, options);
        let sinceOrder = actual.join(",");

        actual = [];
        instance.until(other, options);
        let untilOrder = actual.join(",");

        actual = [];
        let invalidOptions = {
            get largestUnit() {
                actual.push("get options.largestUnit");
                return observedString(actual, "options.largestUnit", "week");
            },
            get roundingIncrement() {
                actual.push("get options.roundingIncrement");
                return observedInteger(actual, "options.roundingIncrement", 1);
            },
            get roundingMode() {
                actual.push("get options.roundingMode");
                return observedString(actual, "options.roundingMode", "halfFloor");
            },
            get smallestUnit() {
                actual.push("get options.smallestUnit");
                return observedString(actual, "options.smallestUnit", "hour");
            },
        };
        let sinceInvalid = (() => {
            try {
                new Temporal.PlainTime(14).since(new Temporal.PlainTime(16), invalidOptions);
                return "no-throw";
            } catch (error) {
                return `${error.constructor === RangeError}|${actual.join(",")}`;
            }
        })();

        let untilInvalid = (() => {
            actual = [];
            try {
                new Temporal.PlainTime(14).until(new Temporal.PlainTime(16), invalidOptions);
                return "no-throw";
            } catch (error) {
                return `${error.constructor === RangeError}|${actual.join(",")}`;
            }
        })();

        let mismatchSince = (() => {
            try {
                new Temporal.PlainTime(13, 35, 57, 987, 654, 321).since(
                    new Temporal.PlainTime(12, 34, 56),
                    { largestUnit: "minutes", smallestUnit: "hours" }
                );
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        })();
        [
            sinceOrder,
            untilOrder,
            sinceInvalid,
            untilInvalid,
            mismatchSince,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "get other.hour,get other.hour.valueOf,call other.hour.valueOf,get other.microsecond,get other.microsecond.valueOf,call other.microsecond.valueOf,get other.millisecond,get other.millisecond.valueOf,call other.millisecond.valueOf,get other.minute,get other.minute.valueOf,call other.minute.valueOf,get other.nanosecond,get other.nanosecond.valueOf,call other.nanosecond.valueOf,get other.second,get other.second.valueOf,call other.second.valueOf,get options.largestUnit,get options.largestUnit.toString,call options.largestUnit.toString,get options.roundingIncrement,get options.roundingIncrement.valueOf,call options.roundingIncrement.valueOf,get options.roundingMode,get options.roundingMode.toString,call options.roundingMode.toString,get options.smallestUnit,get options.smallestUnit.toString,call options.smallestUnit.toString|get other.hour,get other.hour.valueOf,call other.hour.valueOf,get other.microsecond,get other.microsecond.valueOf,call other.microsecond.valueOf,get other.millisecond,get other.millisecond.valueOf,call other.millisecond.valueOf,get other.minute,get other.minute.valueOf,call other.minute.valueOf,get other.nanosecond,get other.nanosecond.valueOf,call other.nanosecond.valueOf,get other.second,get other.second.valueOf,call other.second.valueOf,get options.largestUnit,get options.largestUnit.toString,call options.largestUnit.toString,get options.roundingIncrement,get options.roundingIncrement.valueOf,call options.roundingIncrement.valueOf,get options.roundingMode,get options.roundingMode.toString,call options.roundingMode.toString,get options.smallestUnit,get options.smallestUnit.toString,call options.smallestUnit.toString|true|get options.largestUnit,get options.largestUnit.toString,call options.largestUnit.toString,get options.roundingIncrement,get options.roundingIncrement.valueOf,call options.roundingIncrement.valueOf,get options.roundingMode,get options.roundingMode.toString,call options.roundingMode.toString,get options.smallestUnit,get options.smallestUnit.toString,call options.smallestUnit.toString|true|get options.largestUnit,get options.largestUnit.toString,call options.largestUnit.toString,get options.roundingIncrement,get options.roundingIncrement.valueOf,call options.roundingIncrement.valueOf,get options.roundingMode,get options.roundingMode.toString,call options.roundingMode.toString,get options.smallestUnit,get options.smallestUnit.toString,call options.smallestUnit.toString|true"
    );
}

#[test]
fn temporal_plain_time_since_until_reject_smaller_largest_unit_pairs() {
    let result = compile_and_run_string_with_host(
        r#"
        let earlier = new Temporal.PlainTime(12, 34, 56, 0, 0, 0);
        let later = new Temporal.PlainTime(13, 35, 57, 987, 654, 321);
        let units = ["hours", "minutes", "seconds", "milliseconds", "microseconds", "nanoseconds"];
        let failures = [];
        for (let largestIdx = 1; largestIdx < units.length; largestIdx++) {
            for (let smallestIdx = 0; smallestIdx < largestIdx; smallestIdx++) {
                let largestUnit = units[largestIdx];
                let smallestUnit = units[smallestIdx];
                for (let method of ["since", "until"]) {
                    try {
                        later[method](earlier, { largestUnit, smallestUnit });
                        failures.push(`${method}:${largestUnit}/${smallestUnit}`);
                    } catch (error) {
                        if (error.constructor !== RangeError) {
                            failures.push(`${method}:${largestUnit}/${smallestUnit}:${error.constructor.name}`);
                        }
                    }
                }
            }
        }
        failures.join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "");
}

#[test]
fn temporal_plain_time_to_string_honors_precision_and_rounding_options() {
    let result = compile_and_run_string_with_host(
        r#"
        let time = new Temporal.PlainTime(23, 59, 59, 999, 500, 0);
        [
            time.toString({ smallestUnit: "minute", roundingMode: "trunc" }),
            time.toString({ smallestUnit: "minutes", roundingMode: "trunc" }),
            time.toString({ smallestUnit: "second", roundingMode: "halfExpand" }),
            time.toString({ fractionalSecondDigits: 3, roundingMode: "trunc" }),
            time.toString({ fractionalSecondDigits: 2.5, roundingMode: "trunc" }),
            time.toString({ smallestUnit: "microsecond", roundingMode: "halfExpand" }),
            time.toJSON({ smallestUnit: "minute" }),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "23:59|23:59|00:00:00|23:59:59.999|23:59:59.99|23:59:59.999500|23:59:59.9995"
    );
}

#[test]
fn temporal_plain_date_converts_to_plain_date_time() {
    let result = compile_and_run_string_with_host(
        r#"
        let date = new Temporal.PlainDate(2024, 2, 29);
        let time = new Temporal.PlainTime(1, 2, 3, 4, 5, 6);
        let defaultDateTime = date.toPlainDateTime();
        let dateTimeWithTime = date.toPlainDateTime(time);
        [
            defaultDateTime instanceof Temporal.PlainDateTime,
            defaultDateTime.toString(),
            dateTimeWithTime.toString(),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "true|2024-02-29T00:00:00|2024-02-29T01:02:03.004005006"
    );
}
