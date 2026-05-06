use super::support::compile_and_run_string;

#[test]
fn json_parse_and_stringify_use_proxy_aware_is_array() {
    let result = compile_and_run_string(
        r#"
        let arrayProxy = new Proxy([], {
            get: function(_target, key) {
                if (key === "length") return 2;
                return Number(key);
            }
        });
        let arrayProxyProxy = new Proxy(arrayProxy, {});
        let replacer = new Proxy(["b"], {});
        let visitedOther = [];

        JSON.parse("[null,null]", function(name, value) {
            if (name === "other") {
                visitedOther.push("ordinary");
            }
            this[1] = new Proxy({ length: 0, other: 0 }, {});
            return value;
        });

        JSON.parse("[null,null]", function(name, value) {
            if (name === "other") {
                visitedOther.push("array");
            }
            this[1] = new Proxy([], {});
            return value;
        });

        [
            JSON.stringify(arrayProxy),
            JSON.stringify([[arrayProxyProxy]]),
            JSON.stringify({ a: 1, b: 2 }, replacer),
            visitedOther.join(",")
        ].join("|");
        "#,
    );

    assert_eq!(result, r#"[0,1]|[[[0,1]]]|{"b":2}|ordinary"#);
}

#[test]
fn json_stringify_bigint_calls_to_json_before_replacer() {
    let result = compile_and_run_string(
        r#"
        let step = 0;
        BigInt.prototype.toJSON = function(key) {
            if (key !== "") return "bad-key";
            if (step !== 0) return "bad-order";
            step += 1;
            return 1n;
        };

        try {
            JSON.stringify(0n, function(key, value) {
                if (step !== 1) return "bad-replacer-order";
                step += 1;
                return value;
            });
            "missing";
        } catch (error) {
            (error instanceof TypeError) + ":" + step;
        }
        "#,
    );

    assert_eq!(result, "true:2");
}

#[test]
fn json_parse_reviver_source_survives_same_value_replacement() {
    let result = compile_and_run_string(
        r#"
        let seen = "";
        JSON.parse('{ "a": "b", "c": "ABCDEABCDE" }', function(key, value, context) {
            if (key === "a") {
                this.c = "ABCDEABCDE";
            }
            if (key === "c") {
                seen = String("source" in context) + ":" + context.source;
            }
            return value;
        });
        seen;
        "#,
    );

    assert_eq!(result, r#"true:"ABCDEABCDE""#);
}

#[test]
fn json_stringify_boxed_primitives_use_ordinary_to_primitive() {
    let result = compile_and_run_string(
        r#"
        function redefine(obj, prop, fun) {
            Object.defineProperty(obj, prop, {
                value: fun,
                writable: true,
                configurable: true
            });
        }

        let numToString = Number.prototype.toString;
        let numValueOf = Number.prototype.valueOf;
        let objToString = Object.prototype.toString;
        let objValueOf = Object.prototype.valueOf;
        let strToString = String.prototype.toString;
        let strValueOf = String.prototype.valueOf;

        redefine(Number.prototype, "valueOf", function() { return 17; });
        let numberValueOf = JSON.stringify(new Number(5));
        delete Number.prototype.toString;
        let numberNoToString = JSON.stringify(new Number(5));
        delete Number.prototype.valueOf;
        let numberNoValueOf = JSON.stringify(new Number(5));
        delete Object.prototype.toString;
        let numberThrowsWithoutObjectToString;
        try {
            JSON.stringify(new Number(5));
            numberThrowsWithoutObjectToString = false;
        } catch (error) {
            numberThrowsWithoutObjectToString = error instanceof TypeError;
        }

        redefine(Number.prototype, "toString", numToString);
        redefine(Number.prototype, "valueOf", numValueOf);
        redefine(Object.prototype, "toString", objToString);
        redefine(Object.prototype, "valueOf", objValueOf);

        redefine(String.prototype, "valueOf", function() { return 17; });
        redefine(String.prototype, "toString", function() { return 42; });
        let stringToString = JSON.stringify(new String(5));
        delete String.prototype.toString;
        let stringNoToString = JSON.stringify(new String(5));
        delete Object.prototype.toString;
        let stringObjectToStringDeleted = JSON.stringify(new String(5));
        delete String.prototype.valueOf;
        let stringThrowsWithoutStringValueOf;
        try {
            JSON.stringify(new String(5));
            stringThrowsWithoutStringValueOf = false;
        } catch (error) {
            stringThrowsWithoutStringValueOf = error instanceof TypeError;
        }

        redefine(String.prototype, "toString", strToString);
        redefine(String.prototype, "valueOf", strValueOf);
        redefine(Object.prototype, "toString", objToString);
        redefine(Object.prototype, "valueOf", objValueOf);

        [
            numberValueOf,
            numberNoToString,
            numberNoValueOf,
            String(numberThrowsWithoutObjectToString),
            stringToString,
            stringNoToString,
            stringObjectToStringDeleted,
            String(stringThrowsWithoutStringValueOf)
        ].join("|");
        "#,
    );

    assert_eq!(
        result,
        r#"17|17|null|true|"42"|"[object String]"|"17"|true"#
    );
}
