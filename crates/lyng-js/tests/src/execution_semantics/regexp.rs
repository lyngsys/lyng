use super::support::{compile_and_run_string, compile_unit};
use lyng_js_common::AtomTable;
use lyng_js_env::{RegExpLegacyStaticText, Runtime};
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
fn regexp_execution_reusable_units_preserve_utf16_and_last_index_edges() {
    let result = compile_and_run_string(
        r#"
        let latin = /a/g;
        latin.exec("ba");
        let latinLast = latin.lastIndex;

        let sticky = /\u0100/y;
        sticky.lastIndex = 1;
        let stickyMatch = sticky.exec("x\u0100");

        let astralMatch = /\u{1F600}/gu.exec("x\u{1F600}");
        let lone = String.fromCharCode(0xD800);
        let loneMatch = /\uD800/g.exec(lone);
        let emptyUnicodeMatches = /(?:)/gu[Symbol.match]("\u{1F600}");

        [
            latinLast,
            stickyMatch.index,
            sticky.lastIndex,
            stickyMatch[0].charCodeAt(0),
            astralMatch.index,
            astralMatch[0].length,
            loneMatch[0].charCodeAt(0),
            emptyUnicodeMatches.length
        ].join("|");
        "#,
    );

    assert_eq!(result, "2|1|2|256|1|2|55296|2");
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

#[test]
fn regexp_legacy_static_state_stays_lazy_when_global_loop_never_reads_accessors() {
    let mut atoms = AtomTable::new();
    let unit = compile_unit(
        r#"
        (function () {
            let source = "xxxxxxxxxxxxxxxx";
            let rx = /x/g;
            while (rx.exec(source) !== null) {}
        })();
        "#,
        &mut atoms,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    vm.evaluate_script(agent, realm, &unit)
        .expect("compiled script should execute");

    let state = agent
        .regexp_legacy_static_state(realm.id())
        .expect("default realm should have RegExp legacy static state");
    assert!(matches!(
        state.input(),
        RegExpLegacyStaticText::SourceSlice { range, .. } if range == &(0..16)
    ));
    assert!(matches!(
        state.last_match(),
        RegExpLegacyStaticText::SourceSlice { range, .. } if range == &(15..16)
    ));
    assert!(matches!(
        state.right_context(),
        RegExpLegacyStaticText::Empty
    ));
}

#[test]
fn regexp_legacy_static_accessors_survive_collection_after_multiple_matches() {
    let mut atoms = AtomTable::new();
    let record_unit = compile_unit(
        r#"
        (function () {
            let source = "zzababyy";
            let rx = /(a)(b)?/g;
            rx.exec(source);
            rx.exec(source);
        })();
        "#,
        &mut atoms,
    );
    let access_unit = compile_unit(
        r#"
        let total = 0;
        total += (RegExp.input === "zzababyy" ? 1 : 0);
        total += (RegExp.lastMatch === "ab" && RegExp["$&"] === "ab" ? 2 : 0);
        total += (RegExp.leftContext === "zzab" && RegExp["$`"] === "zzab" ? 4 : 0);
        total += (RegExp.rightContext === "yy" && RegExp["$'"] === "yy" ? 8 : 0);
        total += (RegExp.lastParen === "b" && RegExp["$+"] === "b" ? 16 : 0);
        total += (RegExp.$1 === "a" && RegExp.$2 === "b" && RegExp.$9 === "" ? 32 : 0);
        total;
        "#,
        &mut atoms,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    vm.evaluate_script(agent, realm, &record_unit)
        .expect("recording script should execute");
    agent.force_collect();
    let result = vm
        .evaluate_script(agent, realm, &access_unit)
        .expect("accessor script should execute after collection");

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn regexp_symbol_match_default_global_exec_does_not_materialize_per_match_arrays() {
    let mut atoms = AtomTable::new();
    let unit = compile_unit(
        r#"
        let source = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
        let matches = /x/g[Symbol.match](source);
        matches.length;
        "#,
        &mut atoms,
    );
    let bootstrap_unit = compile_unit("0;", &mut atoms);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    vm.evaluate_script(agent, realm, &bootstrap_unit)
        .expect("bootstrap script should execute");
    let before_objects = runtime.phase6_accounting().heap.objects.live_bytes;
    let agent = runtime.root_agent_mut();
    let result = vm
        .evaluate_script(agent, realm, &unit)
        .expect("compiled script should execute");
    let after_objects = runtime.phase6_accounting().heap.objects.live_bytes;

    assert_eq!(result, Value::from_smi(64));
    let object_delta = after_objects - before_objects;
    assert!(
        object_delta < 8_000,
        "default global @@match should not retain one materialized match array per match: {object_delta}"
    );
}
