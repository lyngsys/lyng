//! Verify the SMI-elision writeback invariant for `op_increment` and
//! `op_decrement` against the non-SMI slow path.
//!
//! The inline SMI hit path for `op_increment`/`op_decrement`:
//!
//! ```text
//!     load_reg!(b => t0);
//!     check_smi!(t0, .slow);
//!     untag_smi!(t0);
//!     inc_smi_overflow!(t0 => t1, .slow);   ; dec_smi_overflow! for op_decrement
//!     tag_smi!(t1);
//!     store_reg!(a, t1);
//!     call_slow!(op_*_record_smi_rs, args = [slot]);
//!     dispatch_after_slow!();
//!     .slow:
//!     call_slow!(op_*_slow_rs, args = [a, b, c, slot]);
//!     dispatch_after_slow!();
//! ```
//!
//! The semantic body writes `numeric = ToNumeric(src)` back to `args.src`
//! BEFORE storing the updated `value` to `args.dst`. The inline SMI hit
//! path elides that writeback because for SMI src `ToNumeric(SMI) == SMI`.
//! Non-SMI src (string, `BigInt`, Object with valueOf) takes `.slow`, which
//! DOES perform the writeback.
//!
//! For postfix `s++` with `s = "1"`:
//!   * `current` holds `"1"` when `op_increment` enters
//!   * Slow path writes `ToNumeric("1") = 1` back to `current` and `2` to `result`
//!   * `s` is stored from `result` (= 2); `r` (postfix value) comes from `current` (= Smi 1)
//!
//! Each test returns a Smi sentinel (1 = pass, 0 = fail) asserting the four
//! invariants: `s` is a number, `s` equals the post-update integer, `r` is a
//! number (writeback ran), `r` equals the pre-update integer.

use lyng_common::{AtomTable, SourceId};
use lyng_compiler::compile_script;
use lyng_env::Runtime;
use lyng_host::NoopHostHooks;
use lyng_parser::parse_script;
use lyng_sema::analyze_script;
use lyng_types::Value;
use lyng_vm::Vm;

/// Compile + execute `src` in a fresh realm, returning the script's
/// completion value.
fn run_script(src: &str) -> Value {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, SourceId::new(0), src);
    assert!(
        !parsed.diagnostics.has_errors(),
        "script should parse cleanly: {:?}",
        parsed.diagnostics.as_slice()
    );
    let sema = analyze_script(&parsed, &atoms);
    assert!(
        !sema.diagnostics.has_errors(),
        "script should pass sema: {:?}",
        sema.diagnostics.as_slice()
    );
    let unit = compile_script(&parsed, &sema, &mut atoms).expect("script should compile");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut vm = Vm::new();
    vm.evaluate_script(agent, realm, &unit)
        .run()
        .expect("script should execute without VM error")
}

#[test]
fn increment_string_src_writes_coerced_numeric_back_to_src() {
    // Postfix `s++` with `s = "1"`: oldValue = ToNumeric("1") = 1, newValue = 2.
    // s := 2, r := Smi(1). The writeback is observable as the type+value of `r`.
    let value = run_script(
        r#"
        (function() {
            let s = "1";
            let r = s++;
            return (typeof s === "number")
                && (s === 2)
                && (typeof r === "number")
                && (r === 1)
                ? 1 : 0;
        })();
        "#,
    );
    assert_eq!(
        value,
        Value::from_smi(1),
        "op_increment slow path must write ToNumeric(src) back to src \
         when src is non-SMI; got value {value:?}"
    );
}

#[test]
fn decrement_string_src_writes_coerced_numeric_back_to_src() {
    // Postfix `s--` with `s = "2"`: oldValue = ToNumeric("2") = 2, newValue = 1.
    // s := 1, r := Smi(2). Without the writeback `r` stays the string "2".
    let value = run_script(
        r#"
        (function() {
            let s = "2";
            let r = s--;
            return (typeof s === "number")
                && (s === 1)
                && (typeof r === "number")
                && (r === 2)
                ? 1 : 0;
        })();
        "#,
    );
    assert_eq!(
        value,
        Value::from_smi(1),
        "op_decrement slow path must write ToNumeric(src) back to src \
         when src is non-SMI; got value {value:?}"
    );
}

#[test]
fn increment_prefix_string_src_coerces_via_slow_path() {
    // Prefix `++s` with `s = "5"`: oldValue = ToNumeric("5") = 5, newValue = 6.
    // r := 6 (prefix returns newValue). Confirms slow path coercion.
    let value = run_script(
        r#"
        (function() {
            let s = "5";
            let r = ++s;
            return (typeof s === "number")
                && (s === 6)
                && (typeof r === "number")
                && (r === 6)
                ? 1 : 0;
        })();
        "#,
    );
    assert_eq!(
        value,
        Value::from_smi(1),
        "op_increment slow path must compute newValue = ToNumeric(src) + 1 \
         for non-SMI src; got value {value:?}"
    );
}

#[test]
fn decrement_prefix_string_src_coerces_via_slow_path() {
    // Prefix `--s` with `s = "10"`:
    //   1. oldValue = ToNumeric(GetValue(lhs))      -> Number(10)
    //   2. newValue = oldValue - 1                  -> Number(9)
    //   3. PutValue(lhs, newValue)                  -> s := 9
    //   4. Return newValue                          -> r := 9
    let value = run_script(
        r#"
        (function() {
            let s = "10";
            let r = --s;
            return (typeof s === "number")
                && (s === 9)
                && (typeof r === "number")
                && (r === 9)
                ? 1 : 0;
        })();
        "#,
    );
    assert_eq!(
        value,
        Value::from_smi(1),
        "op_decrement slow path must compute newValue = ToNumeric(src) - 1 \
         for non-SMI src; got value {value:?}"
    );
}
