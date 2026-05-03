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
