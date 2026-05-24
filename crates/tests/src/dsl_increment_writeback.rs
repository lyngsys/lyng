//! Phase 1.C.3 Task 11: verify the SMI-elision writeback claim for
//! `op_increment` and `op_decrement` against the non-SMI slow path.
//!
//! Tasks 9 / 10 (`crates/vm/src/dsl/handlers/cold.rs:1892-1909`
//! and 1968-1985) ported the inline SMI hit path for `op_increment` /
//! `op_decrement` to:
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
//! The semantic body
//! (`crates/vm/src/vm/semantics/arithmetic.rs:796-833`) writes
//! `numeric = ToNumeric(src)` back to `args.src` BEFORE storing the
//! updated `value` to `args.dst`. The inline SMI hit path ELIDES that
//! writeback because for SMI src, `ToNumeric(SMI) == SMI` — the
//! writeback would be a no-op. Non-SMI src (string, `BigInt`, Object
//! with valueOf) takes the `.slow` branch which calls
//! `op_*_slow_rs → op_*_semantic`, which DOES perform the writeback.
//!
//! This test locks down the slow-path writeback by forcing a non-SMI
//! src via a string source. The compiler's
//! `lower_update_expression` (crates/compiler/src/script/
//! property_exprs.rs:12-33) emits a `Move`/load into a `current` temp
//! followed by `Increment result, current` / `Decrement result, current`
//! followed by a store back to the variable AND an
//! `emit_move(dest, if prefix { result } else { current })`. So:
//!
//!   - For postfix `s++` with `s = "1"`:
//!     * `current` holds `"1"` (string) when `op_increment` enters
//!     * Slow path: writes `ToNumeric("1") = 1` (Smi) back to `current`
//!       and writes `1 + 1 = 2` to `result`
//!     * The variable `s` is then stored from `result` (= 2)
//!     * The postfix expression value (== `r` in `let r = s++`) is
//!       `current`, which now holds the coerced Smi `1` — NOT the
//!       string `"1"`. This is the writeback proof.
//!
//!   - For postfix `s--` with `s = "2"`:
//!     * `current` holds `"2"`
//!     * Slow path: writes `ToNumeric("2") = 2` (Smi) back to `current`
//!       and writes `2 - 1 = 1` to `result`
//!     * `s` becomes `1`; `r` (= postfix value) becomes `2`
//!
//! Each test returns a SMI sentinel (1 = pass, 0 = fail) so we can
//! assert with the existing `Value::from_smi` helper used elsewhere
//! in this crate. Inline JS assertions exercise the four observable
//! invariants for each case:
//!   1. `s` is a number (writeback + update succeeded)
//!   2. `s` equals the expected post-update integer
//!   3. `r` is a number (the postfix returns the coerced numeric — proves
//!      the writeback to `current` ran)
//!   4. `r` equals the expected pre-update integer (1 / 2)
//!
//! If the inline SMI hit path had been mis-implemented to elide the
//! writeback unconditionally (i.e. for non-SMI src as well), `r`
//! would hold the original string and `typeof r === "number"` would
//! be `false`, returning 0 and failing the assertion.
//!
//! See `reports/lyng/dsl-handlers/op_increment.md` and
//! `op_decrement.md` § SMI-elision-of-src-writeback for the structural
//! claim this test backstops.

use lyng_common::{AtomTable, SourceId};
use lyng_compiler::compile_script;
use lyng_env::Runtime;
use lyng_host::NoopHostHooks;
use lyng_parser::parse_script;
use lyng_sema::analyze_script;
use lyng_types::Value;
use lyng_vm::Vm;

/// Compile + execute `src` in a fresh realm, returning the script's
/// completion value. Mirrors the helper used by `op_ldar_inline.rs`,
/// `op_load_const8_inline.rs`, `op_locals_inline.rs`, etc.
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
        .expect("script should execute without VM error")
}

#[test]
fn increment_string_src_writes_coerced_numeric_back_to_src() {
    // Postfix `s++` with `s = "1"`:
    //   ECMAScript abstract steps (12.4.4.1):
    //     1. oldValue = ToNumeric(GetValue(lhs))      -> Number(1)
    //     2. newValue = oldValue + 1                  -> Number(2)
    //     3. PutValue(lhs, newValue)                  -> s := 2
    //     4. Return oldValue                          -> r := 1
    //
    // The Vm body computes the (numeric, value) pair in
    // `update_register_value` and writes BOTH `numeric -> args.src`
    // (the temp holding the post-coercion oldValue) and `value ->
    // args.dst` (the temp holding the post-update newValue). The
    // postfix expression's emitted Move reads from the SRC temp, so
    // the writeback is observable as the type+value of `r`.
    //
    // Assertion encoded as a SMI sentinel returned from the script:
    //   1 means all four invariants hold; 0 means at least one failed.
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
    // Postfix `s--` with `s = "2"`:
    //   1. oldValue = ToNumeric(GetValue(lhs))      -> Number(2)
    //   2. newValue = oldValue - 1                  -> Number(1)
    //   3. PutValue(lhs, newValue)                  -> s := 1
    //   4. Return oldValue                          -> r := 2
    //
    // Same writeback mechanism as op_increment — the slow path's
    // `op_decrement_semantic` writes `numeric -> src`. Without the
    // writeback, `r` would still hold the string "2" and the
    // `typeof r === "number"` check would fail.
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
    // Prefix `++s` with `s = "5"`:
    //   1. oldValue = ToNumeric(GetValue(lhs))      -> Number(5)
    //   2. newValue = oldValue + 1                  -> Number(6)
    //   3. PutValue(lhs, newValue)                  -> s := 6
    //   4. Return newValue                          -> r := 6
    //
    // Prefix's emitted Move reads from the RESULT temp (the dst of
    // op_increment), not the SRC temp, so this test does NOT exercise
    // the writeback path the same way postfix does. It still confirms
    // the slow path runs (string -> number coercion) and produces the
    // correct `value` half of the pair.
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
