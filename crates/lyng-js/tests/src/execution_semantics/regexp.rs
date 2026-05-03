use super::support::{compile_and_run_string, compile_unit};
use lyng_js_common::AtomTable;
use lyng_js_env::Runtime;
use lyng_js_host::NoopHostHooks;
use lyng_js_types::Value;
use lyng_js_vm::Vm;

#[test]
fn regexp_scoped_ignore_case_applies_to_backrefs_boundaries_and_properties() {
    let result = compile_and_run_string(
        r#"
        let longS = String.fromCharCode(0x017f);
        [
            /(a)(?i:\1)/.test("aA"),
            /(a)(?-i:\1)/i.test("aA"),
            /(a)(?-i:\1)/i.test("AA"),
            /(?i:\b)/u.test(longS),
            /(?-i:\b)/ui.test(longS),
            /(?i:Z\B)/u.test("Z" + longS),
            /(?-i:Z\B)/ui.test("Z" + longS),
            /(?i:\p{Lu})/u.test("a"),
            /(?i:\P{Lu})/u.test("A")
        ].join("|");
        "#,
    );

    assert_eq!(result, "true|false|true|true|false|true|false|true|true");
}

#[test]
fn regexp_duplicate_named_backrefs_and_escaped_astral_names_match_spec() {
    let result = compile_and_run_string(
        r#"
        function captureEscapedAstralName() {
            try {
                let match = "quick brown fox".match(/(?<\ud835\udcd1\ud835\udcfb\ud835\udcf8\ud835\udd00\ud835\udcf7>brown)/);
                return match.groups["\u{1d4d1}\u{1d4fb}\u{1d4f8}\u{1d500}\u{1d4f7}"];
            } catch (error) {
                return Object.prototype.toString.call(error);
            }
        }

        let simple = /(?:(?<x>a)|(?<x>b))\k<x>/;
        let repeated = /(?:(?:(?<x>a)|(?<x>b))\k<x>){2}/;
        let repeatedMatch = repeated.exec("aabb");

        [
            simple.exec("bb")[0],
            simple.exec("abab") === null,
            repeatedMatch[0],
            repeatedMatch.groups.x,
            repeated.exec("abab") === null,
            captureEscapedAstralName()
        ].join("|");
        "#,
    );

    assert_eq!(result, "bb|true|aabb|b|true|brown");
}

#[test]
fn regexp_group_names_are_readable_through_escaped_identifier_properties() {
    let result = compile_and_run_string(
        r#"
        try {
            let match = "quick brown fox".match(/(?<\u{1d4d1}\u{1d4fb}\u{1d4f8}\u{1d500}\u{1d4f7}>brown)/);
            match === null
                ? "null"
                : match.groups.\u{1d4d1}\u{1d4fb}\u{1d4f8}\u{1d500}\u{1d4f7};
        } catch (error) {
            error.name + ":" + Object.prototype.toString.call(error);
        }
        "#,
    );

    assert_eq!(result, "brown");
}

#[test]
fn regexp_literal_cache_keeps_objects_fresh_and_last_index_independent() {
    let mut atoms = AtomTable::new();
    let unit = compile_unit(
        r#"
        function make() {
            return /cache/g;
        }
        let first = make();
        first.lastIndex = 4;
        let second = make();
        second.lastIndex = 9;
        let third = make();
        let total = 0;
        total += (first !== second ? 1 : 0);
        total += (second !== third ? 2 : 0);
        total += (first.lastIndex === 4 ? 4 : 0);
        total += (second.lastIndex === 9 ? 8 : 0);
        total += (third.lastIndex === 0 ? 16 : 0);
        total += (third.test("cache") && third.lastIndex === 5 ? 32 : 0);
        total;
        "#,
        &mut atoms,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    {
        let agent = runtime.root_agent_mut();
        let realm = agent.default_realm().expect("default realm should exist");
        let mut vm = Vm::new();
        let result = vm
            .evaluate_script(agent, realm, &unit)
            .expect("compiled script should execute");

        assert_eq!(result, Value::from_smi(63));
    }

    let accounting = runtime.phase6_accounting();
    assert_eq!(accounting.regexp_literal_cache.records, 1);
    assert!(accounting.regexp_payloads.records >= 3);
}
