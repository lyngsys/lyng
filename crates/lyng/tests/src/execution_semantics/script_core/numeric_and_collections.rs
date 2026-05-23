use super::super::support::*;

#[test]
fn script_core_supports_phase6_number_math_and_bigint_basics() {
    let result = compile_and_run(
        r#"
        let number = Number("41");
        let wrapped = new Number(2);
        let bigint = BigInt("9");
        let total = 0;
        total += (number === 41 ? 1 : 0);
        total += (wrapped.valueOf() === 2 ? 2 : 0);
        total += (wrapped.toString() === "2" ? 4 : 0);
        total += (Math.max(-4, 8, 3) === 8 ? 8 : 0);
        total += (Math.round(-0.25) === 0 ? 16 : 0);
        total += (BigInt.prototype.valueOf.call(bigint) === bigint ? 32 : 0);
        total += (BigInt.prototype.toString.call(bigint) === "9" ? 64 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn script_core_specialized_smi_arithmetic_preserves_negative_zero() {
    let result = compile_and_run(
        r"
        let negative = -1;
        let zero = 0;
        let negativeFour = -4;
        let total = 0;
        total += (Object.is(negative * 0, -0) ? 1 : 0);
        total += (Object.is(zero * negative, -0) ? 2 : 0);
        total += (Object.is(negativeFour % 2, -0) ? 4 : 0);
        total += (Object.is(negativeFour % -2, -0) ? 8 : 0);
        total += (Object.is(0 * 1, +0) ? 16 : 0);
        total += (Object.is(4 % 2, +0) ? 32 : 0);
        total;
        ",
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn script_core_number_min_value_is_minimum_subnormal() {
    let result = compile_and_run(
        r"
        let total = 0;
        total += (Number.MIN_VALUE / 2 === 0 ? 1 : 0);
        total += (1 / (Number.MIN_VALUE / -2) === -Infinity ? 2 : 0);
        total += (Number.MIN_VALUE / 1.9 === Number.MIN_VALUE ? 4 : 0);
        total += (Number.MIN_VALUE * 0.5 === 0 ? 8 : 0);
        total += (1 / (-0.5 * Number.MIN_VALUE) === -Infinity ? 16 : 0);
        total += (Number.MIN_VALUE * 0.51 === Number.MIN_VALUE ? 32 : 0);
        total;
        ",
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn script_core_subnormal_numbers_do_not_collide_with_value_tags() {
    let result = compile_and_run(
        r#"
        let value = 1;
        for (let power = 0; power < 1039; power += 1) {
            value = value * 0.5;
        }

        let total = 0;
        total += (typeof value === "number" ? 1 : 0);
        total += (+value === value ? 2 : 0);

        let next = value * 0.5;
        total += (next * 2 === value ? 4 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn script_core_unary_plus_uses_to_number_instead_of_addition() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += (+"12345" === 12345 ? 1 : 0);
        total += (Number(".12345e-3") === 0.00012345 ? 2 : 0);
        total += (+"-1234567890" === -1234567890 ? 4 : 0);
        try {
            +1n;
        } catch (error) {
            total += (error instanceof TypeError ? 8 : 0);
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_unary_minus_uses_to_primitive_number_hint() {
    let result = compile_and_run(
        r"
        let total = 0;
        total += (-{ valueOf: function() { return -1; } } === 1 ? 1 : 0);
        total += (-new Boolean(true) === -1 ? 2 : 0);
        total += (-1n === -1n ? 4 : 0);
        try {
            -{ valueOf: function() { return {}; }, toString: function() { return {}; } };
        } catch (error) {
            total += (error.constructor === TypeError ? 8 : 0);
        }
        try {
            -Symbol();
        } catch (error) {
            total += (error.constructor === TypeError ? 16 : 0);
        }
        total;
        ",
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_postfix_update_returns_old_numeric_value() {
    let result = compile_and_run(
        r#"
        let total = 0;

        let booleanValue = false;
        total += (booleanValue++ === 0 && booleanValue === 1 ? 1 : 0);

        let objectValue = { valueOf: function() { return 1; } };
        total += (objectValue++ === 1 && objectValue === 2 ? 2 : 0);

        let stringValue = "x";
        let oldStringValue = stringValue++;
        total += (isNaN(oldStringValue) && isNaN(stringValue) ? 4 : 0);

        let bigintValue = 1n;
        total += (bigintValue++ === 1n && bigintValue === 2n ? 8 : 0);

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_supports_number_formatting_builtins() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += ((1).toFixed(1) === "1.0" ? 1 : 0);
        total += (Number.NaN.toFixed(2) === "NaN" ? 2 : 0);
        total += ((3).toFixed(4) === "3.0000" ? 4 : 0);
        total += ((1000).toPrecision(3) === "1.00e+3" ? 8 : 0);
        total += ((7).toPrecision(3) === "7.00" ? 16 : 0);
        total += ((42).toPrecision() === "42" ? 32 : 0);
        total += ((Infinity).toPrecision(1000) === "Infinity" ? 64 : 0);
        total += ((7).toLocaleString() === "7" ? 128 : 0);
        total += ((-0).toExponential(2) === "0.00e+0" ? 256 : 0);
        total += (Number.parseInt === parseInt && Number.parseFloat === parseFloat ? 512 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(1023));
}

#[test]
fn script_core_number_rejects_symbol_arguments() {
    let result = compile_and_run(
        r#"
        let symbol = Symbol("66");
        let total = 0;
        try {
            Number(symbol);
        } catch (error) {
            total += (error instanceof TypeError ? 1 : 0);
        }
        try {
            new Number(symbol);
        } catch (error) {
            total += (error instanceof TypeError ? 2 : 0);
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_supports_extended_math_builtins() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += (Math.acos.length === 1 && Math.acos(1) === 0 ? 1 : 0);
        total += (Math.acosh(1) === 0 ? 2 : 0);
        total += (Math.asin(0) === 0 ? 4 : 0);
        total += (Math.asinh(0) === 0 ? 8 : 0);
        total += (Math.atan(0) === 0 ? 16 : 0);
        total += (Math.atan2.length === 2 && Math.atan2(0, -1) === Math.PI ? 32 : 0);
        total += (Math.atanh(0) === 0 ? 64 : 0);
        total += (Math.cbrt(8) === 2 ? 128 : 0);
        total += (Math.ceil(-0.25) === 0 && 1 / Math.ceil(-0.25) === -Infinity ? 256 : 0);
        total += (Math.clz32(1) === 31 && Math.clz32(-1) === 0 ? 512 : 0);
        total += (Math.cos(0) === 1 && Math.cosh(0) === 1 ? 1024 : 0);
        total += (Math.exp(0) === 1 && Math.expm1(0) === 0 ? 2048 : 0);
        total += (Math.f16round(0.1) === 0.0999755859375 && Math.f16round(65520) === Infinity ? 4096 : 0);
        total += (Math.fround(0.1) === 0.10000000149011612 ? 8192 : 0);
        total += (Math.hypot(3, 4) === 5 && Math.hypot(Infinity, NaN) === Infinity ? 16384 : 0);
        total += (Math.imul(0xffffffff, 5) === -5 ? 32768 : 0);
        total += (Math.log(1) === 0 && Math.log10(100) === 2 ? 65536 : 0);
        total += (Math.log1p(0) === 0 && Math.log2(8) === 3 ? 131072 : 0);
        let random = Math.random();
        total += (typeof random === "number" && random >= 0 && random < 1 ? 262144 : 0);
        total += (Math.sin(0) === 0 && Math.sinh(0) === 0 ? 524288 : 0);
        total += (Math.tan(0) === 0 && Math.tanh(0) === 0 ? 1048576 : 0);
        let calls = 0;
        let coercible = { valueOf: function() { calls += 1; } };
        Math.max(NaN, coercible);
        Math.min(NaN, coercible);
        total += (calls === 2 ? 2097152 : 0);
        total += (
            Math.pow(1, NaN) !== Math.pow(1, NaN) &&
            Math.pow(-1, Infinity) !== Math.pow(-1, Infinity)
                ? 4194304
                : 0
        );
        total += (Math.sumPrecise([1e30, 0.1, -1e30]) === 0.1 ? 8388608 : 0);
        total += (
            Math.sumPrecise([1e308, 1e308, 0.1, 0.1, 1e30, 0.1, -1e30, -1e308, -1e308]) === 0.30000000000000004
                ? 16777216
                : 0
        );
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(33_554_431));
}

#[test]
fn script_core_math_approximation_builtins_match_staging_tolerances() {
    let result = compile_and_run(
        r"
        const f = new Float64Array([0, 0]);
        const u = new Uint32Array(f.buffer);
        let endian = 0;

        function diff(a, b) {
            f[0] = a;
            f[1] = b;
            return Math.abs(
                (u[3 - endian] - u[1 - endian]) * 0x100000000 +
                u[2 + endian] - u[0 + endian]
            );
        }

        if (diff(2, 4) === 0x100000) endian = 1;

        function near(a, b, tolerance) {
            let target = b === 0 ? a * 0 : b;
            return diff(a, target) <= tolerance;
        }

        let score = 0;
        score += near(Math.acosh(1.000007152557373), 0.003782208044661295, 9) ? 1 : 0;
        score += near(Math.acosh(1.0000000001), 0.000014142136208675862, 9) ? 2 : 0;
        score += near(Math.acosh(1e300), 691.4686750787737, 9) ? 4 : 0;
        score += near(Math.atanh(-0.999992847442627), -6.2705920974657525, 2) ? 8 : 0;
        score += near(Math.atanh(-0.9999828338623047), -5.832855225378502, 2) ? 16 : 0;
        score += near(Math.atanh(0.3), 0.3095196042031117, 2) ? 32 : 0;
        score;
        ",
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn script_core_number_prototype_carries_primitive_data_slot_but_bigint_does_not() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += (Number.prototype.toString() === "0" ? 1 : 0);
        total += (Number.prototype.valueOf() === 0 ? 2 : 0);
        try {
            BigInt.prototype.toString();
        } catch (error) {
            total += (error instanceof TypeError ? 4 : 0);
        }
        try {
            BigInt.prototype.valueOf();
        } catch (error) {
            total += (error instanceof TypeError ? 8 : 0);
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_number_constructor_distinguishes_absent_and_undefined_arguments() {
    let result = compile_and_run(
        r"
        let functionDefault = Number();
        let constructorDefault = new Number().valueOf();
        let functionUndefined = Number(undefined);
        let constructorUndefined = new Number(undefined).valueOf();
        let total = 0;
        total += (functionDefault === 0 ? 1 : 0);
        total += (constructorDefault === 0 ? 2 : 0);
        total += (functionUndefined !== functionUndefined ? 4 : 0);
        total += (constructorUndefined !== constructorUndefined ? 8 : 0);
        total += (Number.prototype.toString.length === 1 ? 16 : 0);
        total;
        ",
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_bigint_boxing_exposes_wrapper_identity_and_value_of() {
    let result = compile_and_run(
        r#"
        let boxed = Object(1n);
        let total = 0;
        total += (typeof boxed === "object" ? 1 : 0);
        total += (boxed instanceof BigInt ? 2 : 0);
        total += (BigInt.prototype.valueOf.call(boxed) === 1n ? 4 : 0);
        total += (Object.prototype.toString.call(boxed) === "[object BigInt]" ? 8 : 0);
        total += (Object.getPrototypeOf(boxed) === BigInt.prototype ? 16 : 0);
        total += (boxed.valueOf() === 1n ? 32 : 0);
        total += (!(boxed instanceof Boolean) ? 64 : 0);
        total += (BigInt.prototype !== Boolean.prototype ? 128 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(255));
}

#[test]
fn script_core_bigint_width_builtins_wrap_with_toindex_and_tobigint() {
    let result = compile_and_run(
        r"
        let total = 0;
        total += (BigInt.asIntN(2, 3n) === -1n ? 1 : 0);
        total += (BigInt.asIntN(3, 10n) === 2n ? 2 : 0);
        total += (BigInt.asUintN(2, -1n) === 3n ? 4 : 0);
        total += (BigInt.asUintN(8, 0x123n) === 0x23n ? 8 : 0);
        total += (BigInt.asIntN.length === 2 ? 16 : 0);
        total += (BigInt.asUintN.length === 2 ? 32 : 0);
        try {
            BigInt.asIntN(0n, 0n);
        } catch (error) {
            total += (error instanceof TypeError ? 64 : 0);
        }
        try {
            BigInt.asUintN(9007199254740992, 0n);
        } catch (error) {
            total += (error instanceof RangeError ? 128 : 0);
        }
        total;
        ",
    );

    assert_eq!(result, Value::from_smi(255));
}

#[test]
fn script_core_supports_bigint_literals_and_number_radix_poisoning_cases() {
    let result = compile_and_run(
        r#"
        function Test262Error() {}
        let total = 0;
        let poisoned = {
            valueOf() {
                throw new Test262Error();
            }
        };
        try {
            0..toString(poisoned);
        } catch (error) {
            total += (error.constructor === Test262Error ? 1 : 0);
        }
        total += ((15n).toString(16) === "f" ? 2 : 0);
        try {
            (0n).toString(Symbol());
        } catch (error) {
            total += (error.constructor === TypeError ? 4 : 0);
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn script_core_bigint_to_string_uses_lowercase_digits_through_radix_36() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += ((10n).toString(11) === "a" ? 1 : 0);
        total += ((35n).toString(36) === "z" ? 2 : 0);
        total += (Number(97n) === 97 ? 4 : 0);
        total += (String.fromCharCode(Number(97n)) === "a" ? 8 : 0);
        let loopText = "";
        for (let i = 10n; i < 13; i++) {
            loopText += i.toString(36);
        }
        total += (loopText === "abc" ? 16 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}

#[test]
fn script_core_bigint_wrapper_ordinary_to_primitive_observes_overridden_methods() {
    let result = compile_and_run_string(
        r#"
        const BigIntToString = BigInt.prototype.toString;
        const BigIntValueOf = BigInt.prototype.valueOf;
        let toStringGets = 0;
        let toStringCalls = 0;
        let valueOfGets = 0;
        let valueOfCalls = 0;
        let toStringFunction = function() {
            ++toStringCalls;
            return `${BigIntToString.call(this)}foo`;
        };
        let valueOfFunction = function() {
            ++valueOfCalls;
            return BigIntValueOf.call(this) * 2n;
        };
        function record(thunk) {
            try {
                return String(thunk());
            } catch (error) {
                return error instanceof TypeError ? "TypeError" : "throw";
            }
        }
        Object.defineProperty(BigInt.prototype, "toString", {
            get: function() {
                ++toStringGets;
                return toStringFunction;
            },
        });

        let output = "";
        output += record(function() { return "" + Object(1n); }) + "|";
        output += record(function() { return +Object(1n); }) + "|";
        output += record(function() { return `${Object(1n)}`; }) + "|";
        output += toStringGets + "," + toStringCalls + "|";

        Object.defineProperty(BigInt.prototype, "valueOf", {
            get: function() {
                ++valueOfGets;
                return valueOfFunction;
            },
        });

        output += record(function() { return Object(1n) == 2n; }) + "|";
        output += record(function() { return Object(1n) + 1n; }) + "|";
        output += record(function() { return ({ "1foo": 1, "2": 2 })[Object(1n)]; }) + "|";
        output += toStringGets + "," + toStringCalls + "," + valueOfGets + "," + valueOfCalls + "|";

        toStringFunction = undefined;
        output += record(function() { return 1 + Object(1n); }) + "|";
        output += record(function() { return Object(1n) * 1n; }) + "|";
        output += record(function() { return "".concat(Object(1n)); }) + "|";
        output += toStringGets + "," + toStringCalls + "," + valueOfGets + "," + valueOfCalls;
        output;
        "#,
    );

    assert_eq!(
        result,
        "1|TypeError|1foo|1,1|true|3|1|2,2,2,2|TypeError|2|2|3,2,5,5"
    );
}

#[test]
fn script_core_bigint_bitwise_operators_use_to_numeric() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += ((0b101n & 0b011n) === 0b001n ? 1 : 0);
        total += ((-2n & -3n) === -4n ? 2 : 0);

        let calls = "";
        let left = {
            [Symbol.toPrimitive]: function(hint) {
                calls += "L" + hint;
                return 5n;
            }
        };
        let right = {
            [Symbol.toPrimitive]: function(hint) {
                calls += "R" + hint;
                return 3n;
            }
        };
        total += ((left & right) === 1n ? 4 : 0);
        total += (calls === "LnumberRnumber" ? 8 : 0);
        total += (({
            valueOf: 1,
            toString: function() {
                return 6n;
            }
        } & 7n) === 6n ? 16 : 0);

        try {
            1n & 1;
        } catch (error) {
            total += (error.constructor === TypeError ? 32 : 0);
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn script_core_bitwise_not_supports_number_and_bigint() {
    let result = compile_and_run(
        r"
        let total = 0;
        total += (~0 === -1 ? 1 : 0);
        total += (~1n === -2n ? 2 : 0);
        total += (~Object(1n) === -2n ? 4 : 0);
        total += (~{ valueOf: function() { return 1n; } } === -2n ? 8 : 0);
        total;
        ",
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_addition_uses_default_to_primitive_hint() {
    let result = compile_and_run(
        r#"
        let total = 0;
        let calls = "";
        let left = {
            [Symbol.toPrimitive]: function(hint) {
                calls += "L" + hint;
                return 2;
            }
        };
        let right = {
            [Symbol.toPrimitive]: function(hint) {
                calls += "R" + hint;
                return 3;
            }
        };
        total += (left + right === 5 ? 1 : 0);
        total += (calls === "LdefaultRdefault" ? 2 : 0);
        total += ({ [Symbol.toPrimitive]: function(hint) { return hint; } } + "" === "default" ? 4 : 0);
        try {
            0 + { [Symbol.toPrimitive]: function() { return Symbol.toPrimitive; } };
        } catch (error) {
            total += (error.constructor === TypeError ? 8 : 0);
        }
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_bigint_shift_operators_use_to_numeric() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += ((8n << 2n) === 32n ? 1 : 0);
        total += ((32n >> 2n) === 8n ? 2 : 0);
        total += ((-5n >> 1n) === -3n ? 4 : 0);
        total += ((8n << -1n) === 4n ? 8 : 0);
        total += ((8n >> -1n) === 16n ? 16 : 0);

        let calls = "";
        let left = {
            [Symbol.toPrimitive]: function(hint) {
                calls += "L" + hint;
                return 4n;
            }
        };
        let right = {
            [Symbol.toPrimitive]: function(hint) {
                calls += "R" + hint;
                return 1n;
            }
        };
        total += ((left << right) === 8n ? 32 : 0);
        total += (calls === "LnumberRnumber" ? 64 : 0);

        try {
            1n << 1;
        } catch (error) {
            total += (error.constructor === TypeError ? 128 : 0);
        }
        try {
            1n >>> 0n;
        } catch (error) {
            total += (error.constructor === TypeError ? 256 : 0);
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(511));
}

#[test]
fn script_core_exponentiation_uses_to_numeric_order_and_infinity_edges() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += (isNaN(1 ** Infinity) ? 1 : 0);
        total += (isNaN((-1) ** -Infinity) ? 2 : 0);

        let trace = "";
        try {
            ({
                valueOf: function() {
                    trace += "L";
                    return Symbol("x");
                }
            }) ** ({
                valueOf: function() {
                    trace += "R";
                    return 1;
                }
            });
        } catch (error) {
            total += (error.constructor === TypeError ? 4 : 0);
        }
        total += (trace === "L" ? 8 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_bigint_relational_comparison_uses_to_numeric_ordering() {
    let result = compile_and_run(
        r#"
        let total = 0;
        total += (0n < 1 ? 1 : 0);
        total += (0.000000000001 < 1n ? 2 : 0);
        total += ((1n < 1) === false ? 4 : 0);
        total += ((Number.MIN_VALUE < -10n) === false ? 8 : 0);
        total += (1n < "2" ? 16 : 0);
        total += ("2" < 3n ? 32 : 0);
        total += ((1n < "not numeric") === false ? 64 : 0);
        total += (("0e0" < 1n) === false ? 128 : 0);
        total += ((0n < "1e0") === false ? 256 : 0);
        total += (("" < 1n) ? 512 : 0);

        try {
            1n < Symbol();
        } catch (error) {
            total += (error.constructor === TypeError ? 1024 : 0);
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(2047));
}

#[test]
fn script_core_sparse_array_literals_preserve_length_and_holes() {
    let result = compile_and_run(
        r"
        let oneHole = [,];
        let twoHoles = [,,];
        let total = 0;
        total += (oneHole.length === 1 ? 1 : 0);
        total += (!(0 in oneHole) ? 2 : 0);
        total += (oneHole[0] === undefined ? 4 : 0);
        total += (twoHoles.length === 2 ? 8 : 0);
        total += (!(1 in twoHoles) ? 16 : 0);
        total += ([].length === 0 ? 32 : 0);
        total;
        ",
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn script_core_map_and_set_iterables_preserve_size_and_insertion_order() {
    let result = compile_and_run(
        r#"
        let set = new Set([1, "a", true]);
        let map = new Map([[1, "a"], ["b", true]]);
        let setOrder = "";
        for (const value of set) {
            setOrder += value + "|";
        }
        let mapOrder = "";
        for (const entry of map) {
            mapOrder += entry[0] + ":" + entry[1] + "|";
        }
        let total = 0;
        total += (set.size === 3 ? 1 : 0);
        total += (setOrder === "1|a|true|" ? 2 : 0);
        total += (map.size === 2 ? 4 : 0);
        total += (mapOrder === "1:a|b:true|" ? 8 : 0);
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn script_core_collection_constructors_follow_iterator_protocol_edges() {
    let result = compile_and_run(
        r#"
        let total = 0;

        let nextArgCount = -1;
        let iterable = {
            [Symbol.iterator]() {
                return this;
            },
            next() {
                nextArgCount = arguments.length;
                return { done: true };
            }
        };
        new Map(iterable);
        total += nextArgCount === 0 ? 1 : 0;

        let typeofThis = "unset";
        Number.prototype[Symbol.iterator] = function() {
            "use strict";
            typeofThis = typeof this;
            return {
                next() {
                    return { done: true };
                }
            };
        };
        new Map(0);
        delete Number.prototype[Symbol.iterator];
        total += typeofThis === "number" ? 2 : 0;

        function doesNotCloseWhenIteratorValueThrows(Ctor) {
            let closed = false;
            let caught = false;
            let source = {
                [Symbol.iterator]() {
                    return {
                        next() {
                            return {
                                get value() {
                                    throw "value throws";
                                },
                                done: false
                            };
                        },
                        return() {
                            closed = true;
                            return {};
                        }
                    };
                }
            };

            try {
                new Ctor(source);
            } catch (error) {
                caught = error === "value throws";
            }
            return caught && closed === false;
        }

        function closesWhenMapEntryKeyThrows(Ctor) {
            let closed = false;
            let caught = false;
            let source = {
                [Symbol.iterator]() {
                    return {
                        next() {
                            return {
                                value: {
                                    get 0() {
                                        throw "key throws";
                                    }
                                },
                                done: false
                            };
                        },
                        return() {
                            closed = true;
                            return {};
                        }
                    };
                }
            };

            try {
                new Ctor(source);
            } catch (error) {
                caught = error === "key throws";
            }
            return caught && closed === true;
        }

        total += doesNotCloseWhenIteratorValueThrows(Map) ? 4 : 0;
        total += doesNotCloseWhenIteratorValueThrows(WeakMap) ? 8 : 0;
        total += doesNotCloseWhenIteratorValueThrows(Set) ? 16 : 0;
        total += doesNotCloseWhenIteratorValueThrows(WeakSet) ? 32 : 0;
        total += closesWhenMapEntryKeyThrows(Map) ? 64 : 0;
        total += closesWhenMapEntryKeyThrows(WeakMap) ? 128 : 0;
        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(255));
}

#[test]
fn script_core_iterator_from_wrappers_forward_without_helper_state() {
    let result = compile_and_run(
        r#"
        let total = 0;

        try {
            let primitiveNext = Iterator.from({
                next() {
                    return 42;
                }
            }).next();
            total += primitiveNext === 42 ? 1 : 0;
        } catch (error) {
        }

        let iter = {
            next() {
                return { done: false, value: 0 };
            },
            return(value = "old return") {
                return { done: true, value };
            }
        };
        let wrap = Iterator.from(iter);
        let firstReturn = wrap.return("ignored");
        total += firstReturn.done === true && firstReturn.value === "old return" ? 2 : 0;

        iter.return = function() {
            throw "new return";
        };
        try {
            wrap.return();
        } catch (error) {
            total += error === "new return" ? 4 : 0;
        }

        iter.return = null;
        let nullReturn = wrap.return("ignored");
        total += nullReturn.done === true && nullReturn.value === undefined ? 8 : 0;

        let log = [];
        let proxyHandler = {
            get(target, key, receiver) {
                log.push("get:" + String(key));
                let item = Reflect.get(target, key, receiver);
                if (typeof item === "function") {
                    return item.bind(receiver);
                }
                return item;
            },
            getPrototypeOf(target) {
                log.push("proto");
                return Reflect.getPrototypeOf(target);
            }
        };
        let proxiedIterator = new Proxy({
            next() {
                return { done: false, value: 1 };
            }
        }, proxyHandler);
        let proxiedWrap = Iterator.from(proxiedIterator);
        proxiedWrap.next();
        proxiedWrap.next();
        total += log.join("|") === "get:Symbol(Symbol.iterator)|get:next|proto" ? 16 : 0;

        let prototypeHits = 0;
        class Iter extends Iterator {
            [Symbol.iterator]() {
                return this;
            }
            next() {
                return { done: false, value: 2 };
            }
        }
        let iteratorProxy = new Proxy(new Iter(), {
            get(target, key, receiver) {
                return Reflect.get(target, key, receiver);
            },
            getPrototypeOf(target) {
                prototypeHits += 1;
                return Reflect.getPrototypeOf(target);
            }
        });
        total += Iterator.from(iteratorProxy) === iteratorProxy && prototypeHits > 0 ? 32 : 0;

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "single conformance fixture exercises lazy iterator zip validation order"
)]
fn script_core_iterator_zip_defers_direct_next_validation() {
    let result = compile_and_run(
        r#"
        let total = 0;

        let throwingIterator = {
            next() {
                throw new Error("next should not be called during construction");
            },
            return() {
                throw new Error("return should not be called during construction");
            }
        };

        let iterableReturningThrowingIterator = {
            [Symbol.iterator]() {
                return throwingIterator;
            }
        };

        function makeGetProxy(name, object, log) {
            return new Proxy(object, {
                get(target, key, receiver) {
                    log.push(name + "::" + String(key));
                    return Reflect.get(target, key, receiver);
                }
            });
        }

        function zipConstructionLog() {
            let log = [];
            let elements = [
                makeGetProxy("first", throwingIterator, log),
                makeGetProxy("second", iterableReturningThrowingIterator, log),
                makeGetProxy("third", Object.create(null), log),
            ];
            let elementsIter = elements.values();
            let iterables = makeGetProxy("iterables", {
                [Symbol.iterator]() {
                    return this;
                },
                next() {
                    log.push("call next");
                    return elementsIter.next();
                },
                return() {
                    throw new Error("input should not close during construction");
                }
            }, log);
            Iterator.zip(iterables);
            return log.join("|");
        }

        function makeAllProxy(name, object, log) {
            return new Proxy(object, {
                ownKeys(target) {
                    log.push(name + "::keys");
                    return Reflect.ownKeys(target);
                },
                getOwnPropertyDescriptor(target, key) {
                    log.push(name + "::desc:" + String(key));
                    return Reflect.getOwnPropertyDescriptor(target, key);
                },
                get(target, key, receiver) {
                    log.push(name + "::get:" + String(key));
                    return Reflect.get(target, key, receiver);
                },
            });
        }

        function zipKeyedConstructionLog() {
            let log = [];
            let iterables = makeAllProxy("iterables", {
                a: makeAllProxy("first", throwingIterator, log),
                b: makeAllProxy("second", iterableReturningThrowingIterator, log),
                c: makeAllProxy("third", {}, log),
            }, log);
            Iterator.zipKeyed(iterables);
            return log.join("|");
        }

        try {
            total += zipConstructionLog() === [
                "iterables::Symbol(Symbol.iterator)",
                "iterables::next",
                "call next",
                "first::Symbol(Symbol.iterator)",
                "first::next",
                "call next",
                "second::Symbol(Symbol.iterator)",
                "call next",
                "third::Symbol(Symbol.iterator)",
                "third::next",
                "call next"
            ].join("|") ? 1 : 0;
        } catch (error) {
        }

        try {
            total += zipKeyedConstructionLog() === [
                "iterables::keys",
                "iterables::desc:a",
                "iterables::get:a",
                "first::get:Symbol(Symbol.iterator)",
                "first::get:next",
                "iterables::desc:b",
                "iterables::get:b",
                "second::get:Symbol(Symbol.iterator)",
                "iterables::desc:c",
                "iterables::get:c",
                "third::get:Symbol(Symbol.iterator)",
                "third::get:next"
            ].join("|") ? 2 : 0;
        } catch (error) {
        }

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(3));
}

#[test]
fn script_core_iterator_helpers_do_not_close_source_on_source_abrupts() {
    let result = compile_and_run(
        r#"
        let total = 0;

        let methods = [
            iter => iter.map(value => value),
            iter => iter.filter(value => true),
            iter => iter.take(1),
            iter => iter.drop(0),
            iter => iter.flatMap(value => [value]),
        ];

        function makeIterator(mode, close) {
            let iter = {
                next() {
                    if (mode === "next") {
                        throw "next";
                    }
                    if (mode === "done") {
                        return {
                            get done() {
                                throw "done";
                            },
                            value: 1
                        };
                    }
                    if (mode === "value") {
                        return {
                            done: false,
                            get value() {
                                throw "value";
                            }
                        };
                    }
                    if (mode === "primitive") {
                        return 1;
                    }
                    return { done: false, value: 1 };
                },
                return() {
                    close();
                    return { done: true };
                }
            };
            Object.setPrototypeOf(iter, Iterator.prototype);
            return iter;
        }

        function sourceAbruptDoesNotClose(method, mode, matches) {
            let closed = false;
            let iter = makeIterator(mode, () => { closed = true; });
            try {
                method(iter).next();
            } catch (error) {
                return matches(error) && closed === false;
            }
            return false;
        }

        for (let method of methods) {
            total += sourceAbruptDoesNotClose(method, "next", error => error === "next") ? 1 : 0;
            total += sourceAbruptDoesNotClose(method, "done", error => error === "done") ? 1 : 0;
            total += sourceAbruptDoesNotClose(method, "value", error => error === "value") ? 1 : 0;
            total += sourceAbruptDoesNotClose(method, "primitive", error => error instanceof TypeError) ? 1 : 0;
        }

        function callbackThrowCloses(method) {
            let closed = false;
            let iter = makeIterator("ok", () => { closed = true; });
            try {
                method(iter).next();
            } catch (error) {
                return error === "callback" && closed === true;
            }
            return false;
        }

        total += callbackThrowCloses(iter => iter.map(() => { throw "callback"; })) ? 1 : 0;
        total += callbackThrowCloses(iter => iter.filter(() => { throw "callback"; })) ? 1 : 0;
        total += callbackThrowCloses(iter => iter.flatMap(() => { throw "callback"; })) ? 1 : 0;

        function flatMapInnerFailureCloses(inner) {
            let closed = false;
            let iter = makeIterator("ok", () => { closed = true; });
            try {
                iter.flatMap(() => inner).next();
            } catch (error) {
                return error instanceof TypeError && closed === true;
            }
            return false;
        }

        total += flatMapInnerFailureCloses({}) ? 1 : 0;
        total += flatMapInnerFailureCloses({
            [Symbol.iterator]() {
                return {};
            }
        }) ? 1 : 0;

        total;
        "#,
    );

    assert_eq!(result, Value::from_smi(25));
}

#[test]
fn script_core_set_methods_observe_receiver_mutation_order() {
    let result = compile_and_run(
        r#"
        function setItems(set) {
            let items = [];
            for (let value of set) {
                items.push(value);
            }
            return items.join(",");
        }

        function mutateOnNextLookup(set) {
            return {
                size: 0,
                has() {
                    throw new Error("has should not be called");
                },
                keys() {
                    return {
                        get next() {
                            set.clear();
                            set.add(4);
                            return function() {
                                return { done: true };
                            };
                        }
                    };
                }
            };
        }

        let unionSet = new Set([1, 2, 3]);
        let unionResult = unionSet.union(mutateOnNextLookup(unionSet));

        let symmetricSet = new Set([1, 2, 3]);
        let symmetricResult = symmetricSet.symmetricDifference(mutateOnNextLookup(symmetricSet));

        let seen = [];
        let intersectionSet = new Set([1, 2, 3]);
        let intersectionResult = intersectionSet.intersection({
            size: 100,
            has(value) {
                if (value === 2 && seen.indexOf(value) === -1) {
                    intersectionSet.delete(value);
                    intersectionSet.add(value);
                }
                seen.push(value);
                return true;
            },
            keys() {
                throw new Error("keys should not be called");
            }
        });

        let score = 0;
        score += setItems(unionResult) === "4" ? 1 : 0;
        score += setItems(symmetricResult) === "4" ? 2 : 0;
        score += setItems(intersectionResult) === "1,2,3" ? 4 : 0;
        score += seen.join(",") === "1,2,3,2" ? 8 : 0;
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(15));
}
