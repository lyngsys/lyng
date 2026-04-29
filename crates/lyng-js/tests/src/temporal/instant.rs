use super::{compile_and_run_string_with_host, compile_and_run_with_host};
use lyng_js_host::{
    HostHooks, HostResult, TemporalCurrentInstantRequest, TemporalInstant, TestHost,
};
use lyng_js_types::Value;
use std::time::{SystemTime, UNIX_EPOCH};

struct LiveClockHost;

impl HostHooks for LiveClockHost {
    fn temporal_current_instant(
        &self,
        _request: &TemporalCurrentInstantRequest,
    ) -> HostResult<TemporalInstant> {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should stay after UNIX_EPOCH");
        let nanoseconds = i128::from(elapsed.as_secs())
            .checked_mul(1_000_000_000)
            .and_then(|seconds| seconds.checked_add(i128::from(elapsed.subsec_nanos())))
            .expect("current time should fit inside Temporal range");
        Ok(TemporalInstant::new(nanoseconds))
    }
}

#[test]
fn temporal_bootstrap_exposes_namespace_now_and_instant() {
    let result = compile_and_run_with_host(
        r#"
        (typeof Temporal === "object" ? 1 : 0)
            + (typeof Temporal.Now === "object" ? 2 : 0)
            + (typeof Temporal.Now.instant === "function" ? 4 : 0)
            + (typeof Temporal.Now.timeZoneId === "function" ? 8 : 0)
            + (typeof Temporal.Instant === "function" ? 16 : 0)
            + (Object.prototype.toString.call(Temporal.Instant.prototype) === "[object Temporal.Instant]" ? 32 : 0)
            + (typeof Temporal.Duration === "function" ? 64 : 0)
            + (Object.prototype.toString.call(Temporal.Duration.prototype) === "[object Temporal.Duration]" ? 128 : 0)
            + (typeof Temporal.PlainTime === "function" ? 256 : 0)
            + (Object.prototype.toString.call(Temporal.PlainTime.prototype) === "[object Temporal.PlainTime]" ? 512 : 0)
            + (typeof Temporal.PlainDateTime === "function" ? 1024 : 0)
            + (Object.prototype.toString.call(Temporal.PlainDateTime.prototype) === "[object Temporal.PlainDateTime]" ? 2048 : 0)
            + (typeof Temporal.PlainYearMonth === "function" ? 4096 : 0)
            + (Object.prototype.toString.call(Temporal.PlainYearMonth.prototype) === "[object Temporal.PlainYearMonth]" ? 8192 : 0)
            + (typeof Temporal.PlainMonthDay === "function" ? 16384 : 0)
            + (Object.prototype.toString.call(Temporal.PlainMonthDay.prototype) === "[object Temporal.PlainMonthDay]" ? 32768 : 0)
            + (typeof Temporal.ZonedDateTime === "function" ? 65536 : 0)
            + (Object.prototype.toString.call(Temporal.ZonedDateTime.prototype) === "[object Temporal.ZonedDateTime]" ? 131072 : 0)
            + (typeof Temporal.Now.plainDateISO === "function" ? 262144 : 0)
            + (typeof Temporal.Now.plainTimeISO === "function" ? 524288 : 0)
            + (typeof Temporal.Now.plainDateTimeISO === "function" ? 1048576 : 0)
            + (typeof Temporal.Now.zonedDateTimeISO === "function" ? 2097152 : 0);
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, Value::from_smi(4_194_303));
}

#[test]
fn temporal_instant_constructor_and_epoch_getters_round_trip_epoch_nanoseconds() {
    let result = compile_and_run_with_host(
        r#"
        let instant = new Temporal.Instant(1234567890123456789n);
        (instant.epochNanoseconds === 1234567890123456789n ? 1 : 0)
            + (instant.epochMilliseconds === 1234567890123 ? 2 : 0)
            + (instant.epochSeconds === 1234567890 ? 4 : 0);
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn temporal_instant_static_from_and_compare_accept_instant_instances() {
    let result = compile_and_run_with_host(
        r#"
        let instant = new Temporal.Instant(1234567890123456789n);
        let later = Temporal.Instant.fromEpochNanoseconds(1234567890123456790n);
        (Temporal.Instant.compare(instant, later) === -1 ? 1 : 0)
            + (Temporal.Instant.from(instant).epochNanoseconds === instant.epochNanoseconds ? 2 : 0);
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn temporal_instant_static_factories_ignore_subclassing() {
    let result = compile_and_run_string_with_host(
        r#"
        class InstantSubclass extends Temporal.Instant {}
        let from = InstantSubclass.from("1976-11-18T14:23:30.123456789Z");
        let fromEpoch = InstantSubclass.fromEpochNanoseconds(10n);
        [
            from instanceof Temporal.Instant,
            from instanceof InstantSubclass,
            Object.getPrototypeOf(from) === Temporal.Instant.prototype,
            from.epochNanoseconds === 217175010123456789n,
            fromEpoch instanceof InstantSubclass,
            Object.getPrototypeOf(fromEpoch) === Temporal.Instant.prototype,
            fromEpoch.epochNanoseconds === 10n
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|false|true|true|false|true|true");
}

#[test]
fn temporal_instant_from_compare_and_equals_accept_iso_strings() {
    let result = compile_and_run_string_with_host(
        r#"
        let epoch = Temporal.Instant.from("1970-01-01T00:00:00Z");
        let offset = Temporal.Instant.from("1970-01-01T01:00:00+01:00");
        let fractional = Temporal.Instant.from("1970-01-01T00:00:00.123456789Z");
        let basic = Temporal.Instant.from("1976-11-18T15:23Z");
        let compact = Temporal.Instant.from("19761118T152330.1+0000");
        let invalid = (() => {
            try {
                Temporal.Instant.from("1970-01-01T00:00:00");
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        [
            epoch.epochNanoseconds === 0n,
            offset.epochNanoseconds === 0n,
            fractional.epochNanoseconds === 123456789n,
            basic.epochNanoseconds === 217178580000000000n,
            compact.epochNanoseconds === 217178610100000000n,
            Temporal.Instant.compare("1970-01-01T00:00:00Z", "1970-01-01T00:00:01Z") === -1,
            epoch.equals("1970-01-01T01:00:00+01:00"),
            invalid
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|true|true|true|true|true|true|true");
}

#[test]
fn temporal_instant_from_accepts_test262_iso_string_forms() {
    let result = compile_and_run_string_with_host(
        r#"
        [
            Temporal.Instant.from("1976-11-18T15:23z").epochNanoseconds === 217178580000000000n,
            Temporal.Instant.from("1976-11-18T15:23:30.123456789-00:00:00.000000001").epochNanoseconds === 217178610123456790n,
            Temporal.Instant.from("19761118T15:23:30.1+00:00").epochNanoseconds === 217178610100000000n,
            Temporal.Instant.from("1976-11-18T152330.1+0000").epochNanoseconds === 217178610100000000n,
            Temporal.Instant.from("+0019761118T152330.1+0000").epochNanoseconds === 217178610100000000n,
            Temporal.Instant.from("1976-11-18T15:23:30+00").epochNanoseconds === 217178610000000000n,
            Temporal.Instant.from("1976-11-18T15Z").epochNanoseconds === 217177200000000000n,
            Temporal.Instant.from("1976-11-18T15:23:30.123456789Z[u-ca=discord]").epochNanoseconds === 217178610123456789n,
            Temporal.Instant.from("1976-11-18T15:23:30.123456789Z[+12]").epochNanoseconds === 217178610123456789n,
            Temporal.Instant.from("1976-11-18T15:23:30.123456789Z[NotATimeZone]").epochNanoseconds === 217178610123456789n,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|true|true|true|true|true|true|true|true|true");
}

#[test]
fn temporal_instant_string_conversion_rejects_bigint_primitives() {
    let result = compile_and_run_string_with_host(
        r#"
        let instant = new Temporal.Instant(0n);
        let fromBigInt = (() => {
            try {
                Temporal.Instant.from(1n);
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        let compareBigInt = (() => {
            try {
                Temporal.Instant.compare(1n, instant);
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        let equalsBigInt = (() => {
            try {
                instant.equals(1n);
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        [
            new Temporal.Instant(1n).epochNanoseconds === 1n,
            fromBigInt,
            compareBigInt,
            equalsBigInt
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|true|true|true");
}

#[test]
fn temporal_instant_from_rejects_wrong_primitive_types() {
    let result = compile_and_run_string_with_host(
        r#"
        function throwsTypeError(callback) {
            try {
                callback();
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        }
        [
            throwsTypeError(() => Temporal.Instant.from()),
            throwsTypeError(() => Temporal.Instant.from(undefined)),
            throwsTypeError(() => Temporal.Instant.from(undefined, { overflow: "reject" })),
            throwsTypeError(() => Temporal.Instant.from(null)),
            throwsTypeError(() => Temporal.Instant.from(null, { overflow: "constrain" })),
            throwsTypeError(() => Temporal.Instant.from(true)),
            throwsTypeError(() => Temporal.Instant.from(1)),
            throwsTypeError(() => Temporal.Instant.from(19761118)),
            throwsTypeError(() => Temporal.Instant.from(1n)),
            throwsTypeError(() => Temporal.Instant.from(Symbol())),
            throwsTypeError(() => Temporal.Instant.from(Symbol(), { overflow: "reject" })),
            throwsTypeError(() => Temporal.Instant.from(Temporal.Instant.prototype)),
            throwsTypeError(() => Temporal.Instant.from(Temporal.Instant.prototype, { overflow: "reject" }))
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "true|true|true|true|true|true|true|true|true|true|true|true|true"
    );
}

#[test]
fn temporal_instant_constructor_and_from_epoch_nanoseconds_reject_wrong_primitive_types() {
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
        [
            throwsTypeError(() => new Temporal.Instant()),
            throwsTypeError(() => new Temporal.Instant(undefined)),
            throwsTypeError(() => new Temporal.Instant(null)),
            throwsTypeError(() => new Temporal.Instant(42)),
            throwsTypeError(() => new Temporal.Instant(Symbol())),
            throwsTypeError(() => Temporal.Instant.fromEpochNanoseconds()),
            throwsTypeError(() => Temporal.Instant.fromEpochNanoseconds(undefined)),
            throwsTypeError(() => Temporal.Instant.fromEpochNanoseconds(null)),
            throwsTypeError(() => Temporal.Instant.fromEpochNanoseconds(42)),
            throwsTypeError(() => Temporal.Instant.fromEpochNanoseconds(Symbol()))
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|true|true|true|true|true|true|true|true|true");
}

#[test]
fn temporal_instant_from_epoch_milliseconds_converts_and_validates_number_input() {
    let result = compile_and_run_string_with_host(
        r#"
        let after = Temporal.Instant.fromEpochMilliseconds(217175010123);
        let before = Temporal.Instant.fromEpochMilliseconds(-217175010876);
        let limit = 8640000000000000;
        let rangeThrew = (() => {
            try {
                Temporal.Instant.fromEpochMilliseconds(limit + 1);
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let nonIntegerThrew = (() => {
            try {
                Temporal.Instant.fromEpochMilliseconds(1.5);
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let bigintThrew = (() => {
            try {
                Temporal.Instant.fromEpochMilliseconds(42n);
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        [
            typeof Temporal.Instant.fromEpochMilliseconds,
            after.epochNanoseconds === 217175010123000000n,
            before.epochMilliseconds === -217175010876,
            Temporal.Instant.fromEpochMilliseconds(limit).epochMilliseconds === limit,
            rangeThrew,
            nonIntegerThrew,
            bigintThrew
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "function|true|true|true|true|true|true");
}

#[test]
fn temporal_now_uses_host_backed_clock_and_default_zone() {
    let host = TestHost::new();
    host.set_temporal_current_instant(TemporalInstant::new(1_234_567_890_123_456_789));
    host.set_temporal_default_time_zone("UTC");

    let result = compile_and_run_with_host(
        r#"
        let instant = Temporal.Now.instant();
        (instant.epochNanoseconds === 1234567890123456789n ? 1 : 0)
            + (Temporal.Now.timeZoneId() === "UTC" ? 2 : 0)
            + (instant.toString() === "2009-02-13T23:31:30.123456789Z" ? 4 : 0);
        "#,
        host,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn temporal_now_iso_helpers_use_host_clock_and_default_zone() {
    let host = TestHost::new();
    host.set_temporal_current_instant(TemporalInstant::new(217_178_610_123_456_789));
    host.set_temporal_default_time_zone("UTC");

    let result = compile_and_run_string_with_host(
        r#"
        [
            Temporal.Now.plainDateISO().toString(),
            Temporal.Now.plainTimeISO().toString(),
            Temporal.Now.plainDateTimeISO().toString(),
            Temporal.Now.zonedDateTimeISO().toString(),
            Temporal.Now.plainDateISO("UTC").toString(),
            Temporal.Now.zonedDateTimeISO("UTC").timeZoneId,
            Temporal.Now.plainDateISO.length,
            Temporal.Now.plainTimeISO.length,
            Temporal.Now.plainDateTimeISO.length,
            Temporal.Now.zonedDateTimeISO.length,
        ].join("|");
        "#,
        host,
    );

    assert_eq!(
        result,
        "1976-11-18|15:23:30.123456789|1976-11-18T15:23:30.123456789|1976-11-18T15:23:30.123456789+00:00[UTC]|1976-11-18|UTC|0|0|0|0"
    );
}

#[test]
fn temporal_now_instant_epoch_nanoseconds_supports_bigint_division() {
    let host = TestHost::new();
    host.set_temporal_current_instant(TemporalInstant::new(1_234_567_890_123_456_789));

    let result = compile_and_run_string_with_host(
        r#"
        let instant = Temporal.Now.instant();
        [
            instant.epochNanoseconds === 1234567890123456789n,
            Number(instant.epochNanoseconds / 1000000n) === 1234567890123,
        ].join("|");
        "#,
        host,
    );

    assert_eq!(result, "true|true");
}

#[test]
fn temporal_now_instant_matches_date_now_with_live_host_clock() {
    let result = compile_and_run_string_with_host(
        r#"
        let step = "start";
        try {
            step = "date-before";
            let nowBefore = Date.now();
            step = "instant";
            let instant = Temporal.Now.instant();
            step = "epoch-nanoseconds";
            let nanos = instant.epochNanoseconds;
            step = "divide";
            let divided = nanos / 1000000n;
            step = "number";
            let millis = Number(divided);
            step = "date-after";
            let nowAfter = Date.now();
            step = "compare-before";
            let afterBefore = millis >= nowBefore;
            step = "compare-after";
            let beforeAfter = millis <= nowAfter;
            [
                instant instanceof Temporal.Instant,
                typeof nanos,
                typeof divided,
                afterBefore,
                beforeAfter,
            ].join("|");
        } catch (error) {
            "error:" + step + ":" + error.constructor.name + ":" + error.message;
        }
        "#,
        LiveClockHost,
    );

    assert_eq!(result, "true|bigint|bigint|true|true");
}

#[test]
fn temporal_instant_to_json_matches_to_string_and_value_of_throws() {
    let result = compile_and_run_string_with_host(
        r#"
        let instant = new Temporal.Instant(0n);
        let threw = (() => {
            try {
                instant.valueOf();
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        instant.toJSON() + "|" + threw;
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "1970-01-01T00:00:00Z|true");
}

#[test]
fn temporal_instant_to_locale_string_matches_non_intl_to_string_shape() {
    let result = compile_and_run_string_with_host(
        r#"
        let instant = new Temporal.Instant(123456789n);
        let descriptor = Object.getOwnPropertyDescriptor(Temporal.Instant.prototype, "toLocaleString");
        let brandThrew = (() => {
            try {
                Temporal.Instant.prototype.toLocaleString.call({});
                return false;
            } catch (error) {
                return error.constructor === TypeError;
            }
        })();
        [
            typeof Temporal.Instant.prototype.toLocaleString,
            Temporal.Instant.prototype.toLocaleString.length,
            instant.toLocaleString(),
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
        "function|0|1970-01-01T00:00:00.123456789Z|true|false|true|true"
    );
}

#[test]
fn temporal_instant_to_string_honors_precision_and_rounding_options() {
    let result = compile_and_run_string_with_host(
        r#"
        let instant = new Temporal.Instant(987654321n);
        let invalidDigits = (() => {
            try {
                instant.toString({ fractionalSecondDigits: "AUTO" });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let invalidUnit = (() => {
            try {
                instant.toString({ smallestUnit: "hour" });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let invalidRoundingMode = (() => {
            try {
                instant.toString({ roundingMode: "sideways" });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let flooredNonIntegerDigits = (() => {
            try {
                return instant.toString({ fractionalSecondDigits: 1.5 });
            } catch (error) {
                return "threw";
            }
        })();
        [
            instant.toString({ smallestUnit: "microsecond" }),
            instant.toString({ fractionalSecondDigits: 3 }),
            instant.toString({ fractionalSecondDigits: 1 }),
            instant.toString({ fractionalSecondDigits: 0 }),
            instant.toString({ smallestUnit: "minute", roundingMode: "ceil" }),
            instant.toString({ smallestUnit: "second", roundingMode: "ceil" }),
            invalidDigits,
            invalidUnit,
            invalidRoundingMode,
            flooredNonIntegerDigits
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "1970-01-01T00:00:00.987654Z|1970-01-01T00:00:00.987Z|1970-01-01T00:00:00.9Z|1970-01-01T00:00:00Z|1970-01-01T00:01Z|1970-01-01T00:00:01Z|true|true|true|1970-01-01T00:00:00.9Z"
    );
}

#[test]
fn temporal_instant_to_string_supports_time_zone_and_extended_year_options() {
    let result = compile_and_run_string_with_host(
        r#"
        let zero = new Temporal.Instant(0n);
        let extendedNegative = Temporal.Instant.from("-100000-07-01T21:30:36Z");
        let extendedPositive = Temporal.Instant.from("+010000-07-01T21:30:36Z");
        let roundedNegative = Temporal.Instant.from("-000099-12-15T12:00:00.5Z");
        let invalidTimeZoneThrew = (() => {
            try {
                zero.toString({ timeZone: "2021-08-19T17:30" });
                return false;
            } catch (error) {
                return error.constructor === RangeError;
            }
        })();
        let orderLog = [];
        zero.toString({
            get fractionalSecondDigits() {
                orderLog.push("get options.fractionalSecondDigits");
                return {
                    get toString() {
                        orderLog.push("get options.fractionalSecondDigits.toString");
                        return function () {
                            orderLog.push("call options.fractionalSecondDigits.toString");
                            return "auto";
                        };
                    }
                };
            },
            get roundingMode() {
                orderLog.push("get options.roundingMode");
                return {
                    get toString() {
                        orderLog.push("get options.roundingMode.toString");
                        return function () {
                            orderLog.push("call options.roundingMode.toString");
                            return "halfExpand";
                        };
                    }
                };
            },
            get smallestUnit() {
                orderLog.push("get options.smallestUnit");
                return {
                    get toString() {
                        orderLog.push("get options.smallestUnit.toString");
                        return function () {
                            orderLog.push("call options.smallestUnit.toString");
                            return "millisecond";
                        };
                    }
                };
            },
            get timeZone() {
                orderLog.push("get options.timeZone");
                return "UTC";
            }
        });
        let invalidOrder = [];
        let invalidSmallestUnitReadTimeZone = (() => {
            try {
                zero.toString({
                    get fractionalSecondDigits() {
                        invalidOrder.push("get options.fractionalSecondDigits");
                        return "auto";
                    },
                    get roundingMode() {
                        invalidOrder.push("get options.roundingMode");
                        return "expand";
                    },
                    get smallestUnit() {
                        invalidOrder.push("get options.smallestUnit");
                        return {
                            get toString() {
                                invalidOrder.push("get options.smallestUnit.toString");
                                return function () {
                                    invalidOrder.push("call options.smallestUnit.toString");
                                    return "month";
                                };
                            }
                        };
                    },
                    get timeZone() {
                        invalidOrder.push("get options.timeZone");
                        return undefined;
                    }
                });
                return "no-throw";
            } catch (error) {
                return `${error.constructor === RangeError}|${invalidOrder.join(",")}`;
            }
        })();
        [
            zero.toString({ timeZone: "UTC" }),
            zero.toString({ timeZone: "+01:00" }),
            zero.toString({ timeZone: "2021-08-19T17:30-07:00" }),
            extendedNegative.toString(),
            extendedPositive.toJSON(),
            roundedNegative.toString({ smallestUnit: "second", roundingMode: "ceil" }),
            invalidTimeZoneThrew,
            orderLog.join(","),
            invalidSmallestUnitReadTimeZone
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "1970-01-01T00:00:00+00:00|1970-01-01T01:00:00+01:00|1969-12-31T17:00:00-07:00|-100000-07-01T21:30:36Z|+010000-07-01T21:30:36Z|-000099-12-15T12:00:01Z|true|get options.fractionalSecondDigits,get options.fractionalSecondDigits.toString,call options.fractionalSecondDigits.toString,get options.roundingMode,get options.roundingMode.toString,call options.roundingMode.toString,get options.smallestUnit,get options.smallestUnit.toString,call options.smallestUnit.toString,get options.timeZone|true|get options.fractionalSecondDigits,get options.roundingMode,get options.smallestUnit,get options.smallestUnit.toString,call options.smallestUnit.toString,get options.timeZone"
    );
}

#[test]
fn temporal_instant_add_and_subtract_time_durations() {
    let result = compile_and_run_string_with_host(
        r#"
        let instant = new Temporal.Instant(50000n);
        let addResult = instant.add(new Temporal.Duration(0, 0, 0, 0, 6, 5, 4, 3, 2, 1));
        let subtractResult = instant.subtract({ minutes: 5, seconds: 4, milliseconds: 3, microseconds: 2, nanoseconds: 1 });
        let subtractRoundTrip = subtractResult.add({ minutes: 5, seconds: 4, milliseconds: 3, microseconds: 2, nanoseconds: 1 });
        let stringResult = instant.add("PT1H");
        let dateUnitThrew = (() => {
            try {
                instant.add({ days: 1 });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        [
            typeof Temporal.Instant.prototype.add,
            typeof Temporal.Instant.prototype.subtract,
            addResult.epochNanoseconds === 21904003052001n,
            Temporal.Instant.compare(subtractResult, instant) === -1,
            subtractRoundTrip.epochNanoseconds === instant.epochNanoseconds,
            stringResult.epochNanoseconds === 3600000050000n,
            dateUnitThrew
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "function|function|true|true|true|true|true");
}

#[test]
fn temporal_instant_additive_results_reject_out_of_range_epoch_nanoseconds() {
    let result = compile_and_run_string_with_host(
        r#"
        let limit = 8640000000000000;
        let max = Temporal.Instant.fromEpochMilliseconds(limit);
        let min = Temporal.Instant.fromEpochMilliseconds(-limit);
        let maxOverflow = (() => {
            try {
                max.add({ nanoseconds: 1 });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        let minOverflow = (() => {
            try {
                min.subtract({ nanoseconds: 1 });
                return false;
            } catch (error) {
                return error instanceof RangeError;
            }
        })();
        [
            max.add({ nanoseconds: 0 }).epochMilliseconds === limit,
            min.subtract({ nanoseconds: 0 }).epochMilliseconds === -limit,
            maxOverflow,
            minOverflow
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|true|true|true");
}

#[test]
fn temporal_instant_round_since_and_until_are_exact_time_operations() {
    let result = compile_and_run_string_with_host(
        r#"
        let earlier = new Temporal.Instant(1_500_000_000n);
        let later = new Temporal.Instant(9_876_543_210n);
        let since = later.since(earlier);
        let until = earlier.until(later, { smallestUnit: "millisecond", roundingMode: "halfExpand" });
        let rounded = later.round({ smallestUnit: "microsecond", roundingMode: "trunc" });
        let roundedFromString = later.round("second");
        let missingUnitThrew = (() => {
            try {
                later.round();
                return false;
            } catch (error) {
                return error instanceof TypeError;
            }
        })();
        [
            typeof Temporal.Instant.prototype.round,
            Temporal.Instant.prototype.round.name,
            Temporal.Instant.prototype.round.length,
            typeof Temporal.Instant.prototype.since,
            Temporal.Instant.prototype.since.name,
            Temporal.Instant.prototype.since.length,
            typeof Temporal.Instant.prototype.until,
            Temporal.Instant.prototype.until.name,
            Temporal.Instant.prototype.until.length,
            since.seconds,
            since.milliseconds,
            since.microseconds,
            since.nanoseconds,
            until.seconds,
            until.milliseconds,
            until.microseconds,
            until.nanoseconds,
            rounded.epochNanoseconds === 9876543000n,
            roundedFromString.epochNanoseconds === 10000000000n,
            missingUnitThrew
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "function|round|1|function|since|1|function|until|1|8|376|543|210|8|377|0|0|true|true|true"
    );
}

#[test]
fn temporal_instant_difference_uses_number_precision_duration_fields() {
    let result = compile_and_run_string_with_host(
        r#"
        let i1 = new Temporal.Instant(0n);
        let i2 = new Temporal.Instant(18446744073_709_551_616n);
        let since = i1.since(i2, { largestUnit: "microseconds" });
        let until = i1.until(i2, { largestUnit: "microseconds" });
        [
            since.microseconds,
            since.toString(),
            Temporal.Duration.compare(since.add({ microseconds: 1 }), since),
            until.microseconds,
            until.toString(),
            Temporal.Duration.compare(until.add({ microseconds: 1 }), until),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "-18446744073709550|-PT18446744073.709552616S|0|18446744073709550|PT18446744073.709552616S|0"
    );
}

#[test]
fn temporal_instant_round_accepts_day_dividing_increments() {
    let result = compile_and_run_string_with_host(
        r#"
        let instant = Temporal.Instant.from("1976-11-18T14:23:30.123456789Z");
        let expected = Temporal.Instant.from("1976-11-19T00:00:00Z");
        [
            instant.round({ smallestUnit: "hour", roundingIncrement: 24 }).epochNanoseconds === expected.epochNanoseconds,
            instant.round({ smallestUnit: "minute", roundingIncrement: 1440 }).epochNanoseconds === expected.epochNanoseconds,
            instant.round({ smallestUnit: "second", roundingIncrement: 86400 }).epochNanoseconds === expected.epochNanoseconds,
            instant.round({ smallestUnit: "millisecond", roundingIncrement: 86400000 }).epochNanoseconds === expected.epochNanoseconds
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|true|true|true");
}

#[test]
fn temporal_instant_round_accepts_test262_rounding_increments() {
    let result = compile_and_run_string_with_host(
        r#"
        const inst = Temporal.Instant.from("1976-11-18T14:23:30.123456789Z");
        const unitsAndIncrements = {
           hour: [1, 2, 4, 6, 8, 12, 24],
           minute: [1, 5, 10, 20, 30, 40, 80, 120, 720, 1440],
           second: [1, 5, 10, 20, 25, 30, 50, 100, 400, 86400],
           millisecond: [1, 5, 10, 20, 25, 30, 50, 100, 86400000],
           microsecond: [1, 5, 10, 20, 25, 30, 50, 100],
           nanosecond: [1, 5, 10, 20, 25, 30, 50, 100],
        };
        const failures = [];
        Object.entries(unitsAndIncrements).forEach(([unit, increments]) => {
            increments.forEach((increment) => {
                try {
                    const result = inst.round({
                        smallestUnit: unit,
                        roundingMode: "ceil",
                        roundingIncrement: increment,
                    });
                    if (!(result instanceof Temporal.Instant)) {
                        failures.push(`${unit}:${increment}:not-instant`);
                    }
                } catch (error) {
                    failures.push(`${unit}:${increment}:${error.constructor.name}`);
                }
            });
        });
        failures.join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "");
}

#[test]
fn temporal_instant_stringification_formats_boundary_years() {
    let result = compile_and_run_string_with_host(
        r#"
        function epochNsInYear(year) {
          let avgNsPerYear = 31_556_952_000_000_000n;
          return (year - 1970n) * avgNsPerYear + (avgNsPerYear / 2n);
        }
        [
            new Temporal.Instant(epochNsInYear(-10000n)).toString(),
            new Temporal.Instant(epochNsInYear(-1n)).toString(),
            new Temporal.Instant(epochNsInYear(0n)).toString(),
            new Temporal.Instant(epochNsInYear(9999n)).toJSON(),
            new Temporal.Instant(epochNsInYear(10000n)).toJSON()
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "-010000-07-01T21:30:36Z|-000001-07-02T15:41:24Z|0000-07-01T21:30:36Z|9999-07-02T15:41:24Z|+010000-07-01T21:30:36Z"
    );
}

#[test]
fn temporal_instant_difference_reads_options_in_spec_order_before_validation() {
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
        let instance = new Temporal.Instant(1_000_000_000_000_000_000n);
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
                return observedString(actual, "options.roundingMode", "halfExpand");
            },
            get smallestUnit() {
                actual.push("get options.smallestUnit");
                return observedString(actual, "options.smallestUnit", "minutes");
            },
            additional: true,
        };
        let other = Object.defineProperty({}, "toString", {
            get() {
                actual.push("get other.toString");
                return function () {
                    actual.push("call other.toString");
                    return "1970-01-01T00:00Z";
                };
            }
        });
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
                return observedString(actual, "options.smallestUnit", "nanosecond");
            },
        };
        let invalidSince = (() => {
            try {
                new Temporal.Instant(0n).since(new Temporal.Instant(1000n), invalidOptions);
                return "no-throw";
            } catch (error) {
                return `${error.constructor === RangeError}|${actual.join(",")}`;
            }
        })();

        let invalidUntil = (() => {
            actual = [];
            try {
                new Temporal.Instant(0n).until(new Temporal.Instant(1000n), invalidOptions);
                return "no-throw";
            } catch (error) {
                return `${error.constructor === RangeError}|${actual.join(",")}`;
            }
        })();
        [
            sinceOrder,
            untilOrder,
            invalidSince,
            invalidUntil,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "get other.toString,call other.toString,get options.largestUnit,get options.largestUnit.toString,call options.largestUnit.toString,get options.roundingIncrement,get options.roundingIncrement.valueOf,call options.roundingIncrement.valueOf,get options.roundingMode,get options.roundingMode.toString,call options.roundingMode.toString,get options.smallestUnit,get options.smallestUnit.toString,call options.smallestUnit.toString|get other.toString,call other.toString,get options.largestUnit,get options.largestUnit.toString,call options.largestUnit.toString,get options.roundingIncrement,get options.roundingIncrement.valueOf,call options.roundingIncrement.valueOf,get options.roundingMode,get options.roundingMode.toString,call options.roundingMode.toString,get options.smallestUnit,get options.smallestUnit.toString,call options.smallestUnit.toString|true|get options.largestUnit,get options.largestUnit.toString,call options.largestUnit.toString,get options.roundingIncrement,get options.roundingIncrement.valueOf,call options.roundingIncrement.valueOf,get options.roundingMode,get options.roundingMode.toString,call options.roundingMode.toString,get options.smallestUnit,get options.smallestUnit.toString,call options.smallestUnit.toString|true|get options.largestUnit,get options.largestUnit.toString,call options.largestUnit.toString,get options.roundingIncrement,get options.roundingIncrement.valueOf,call options.roundingIncrement.valueOf,get options.roundingMode,get options.roundingMode.toString,call options.roundingMode.toString,get options.smallestUnit,get options.smallestUnit.toString,call options.smallestUnit.toString"
    );
}

#[test]
fn temporal_instant_since_until_reject_smaller_largest_unit_pairs() {
    let result = compile_and_run_string_with_host(
        r#"
        let earlier = new Temporal.Instant(0n);
        let later = new Temporal.Instant(10_000_000_000n);
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
fn temporal_instant_to_zoned_date_time_iso_uses_named_zone() {
    let result = compile_and_run_string_with_host(
        r#"
        let instant = new Temporal.Instant(217178610123456789n);
        let zoned = instant.toZonedDateTimeISO("UTC");
        [
            zoned instanceof Temporal.ZonedDateTime,
            zoned.toString(),
            zoned.timeZoneId,
            zoned.calendarId,
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(
        result,
        "true|1976-11-18T15:23:30.123456789+00:00[UTC]|UTC|iso8601"
    );
}

#[test]
fn temporal_instant_and_zoned_date_time_equals_compare_payloads() {
    let result = compile_and_run_string_with_host(
        r#"
        let instant = new Temporal.Instant(42n);
        let zoned = new Temporal.ZonedDateTime(42n, "UTC");
        [
            instant.equals(new Temporal.Instant(42n)),
            instant.equals(new Temporal.Instant(43n)),
            instant.equals({ epochNanoseconds: 42n }),
            zoned.equals(Temporal.ZonedDateTime.from(zoned)),
            zoned.equals(new Temporal.ZonedDateTime(43n, "UTC")),
        ].join("|");
        "#,
        lyng_js_host::NoopHostHooks,
    );

    assert_eq!(result, "true|false|true|true|false");
}
