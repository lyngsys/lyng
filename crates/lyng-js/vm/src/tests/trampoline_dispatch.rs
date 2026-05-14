//! Phase 1 trampoline-dispatch parity tests (lyng-5zrf).
//!
//! Compiled and run only when the `trampoline-dispatch` feature is enabled,
//! since that feature routes `Vm::run` through `run_via_trampoline` instead
//! of the legacy `run_dispatch_loop`. Each test compiles a trivial script
//! that uses only opcodes implemented in sub-3 (Move, Lda*, Load*, Star0..7,
//! Jump, Jump8, LoopHeader, Return, ReturnUndefined) and asserts the
//! trampoline produces the expected value — matching what the legacy path
//! would have produced.
//!
//! As subsequent sub-issues port additional opcode families, the script
//! coverage here grows.

use super::support::*;

#[test]
fn trampoline_executes_null_literal_script() {
    let unit = compile_test_unit(0, "null");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm
        .evaluate_script(agent, realm, &unit)
        .expect("trampoline-dispatch should execute `null` cleanly");

    assert_eq!(
        result,
        Value::null(),
        "script-level `null` expression statement should evaluate to null"
    );
}

#[test]
fn trampoline_executes_true_literal_script() {
    let unit = compile_test_unit(1, "true");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm
        .evaluate_script(agent, realm, &unit)
        .expect("trampoline-dispatch should execute `true` cleanly");

    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn trampoline_executes_false_literal_script() {
    let unit = compile_test_unit(2, "false");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm
        .evaluate_script(agent, realm, &unit)
        .expect("trampoline-dispatch should execute `false` cleanly");

    assert_eq!(result, Value::from_bool(false));
}

/// Sequence of two expression statements with the same literal — exercises
/// repeated dispatch + advance through the trampoline rather than just one
/// load + return.
#[test]
fn trampoline_executes_sequence_of_null_literals() {
    let unit = compile_test_unit(3, "null; null");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm
        .evaluate_script(agent, realm, &unit)
        .expect("trampoline-dispatch should execute `null; null` cleanly");

    assert_eq!(result, Value::null());
}

/// `42` exercises LdaSmi8 (8-bit signed-integer load to the accumulator) on
/// top of the existing Lda*-style return path.
#[test]
fn trampoline_executes_smi8_literal() {
    let unit = compile_test_unit(4, "42");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm
        .evaluate_script(agent, realm, &unit)
        .expect("trampoline-dispatch should execute `42` cleanly");

    assert_eq!(result, Value::from_smi(42));
}

/// A SMI outside the i8 range exercises the wider Load/Lda forms that fall
/// back to LoadSmi or LoadConst, depending on what the compiler picks.
#[test]
fn trampoline_executes_smi_larger_than_byte() {
    let unit = compile_test_unit(5, "1000");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm
        .evaluate_script(agent, realm, &unit)
        .expect("trampoline-dispatch should execute `1000` cleanly");

    assert_eq!(result, Value::from_smi(1000));
}

/// Multiple SMI literals in a row exercise repeated dispatch through the
/// trampoline plus the script-completion return semantics (the last
/// expression's value is the script result).
#[test]
fn trampoline_executes_smi_sequence_returns_last_value() {
    let unit = compile_test_unit(6, "1; 2; 3");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm
        .evaluate_script(agent, realm, &unit)
        .expect("trampoline-dispatch should execute `1; 2; 3` cleanly");

    assert_eq!(result, Value::from_smi(3));
}

/// `if (true) 1; else 2;` exercises a JumpIfFalse over the consequent.
#[test]
fn trampoline_executes_if_true_consequent() {
    let unit = compile_test_unit(7, "if (true) { 1 } else { 2 }");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm
        .evaluate_script(agent, realm, &unit)
        .expect("trampoline-dispatch should execute if/else cleanly");

    assert_eq!(result, Value::from_smi(1));
}

/// `if (false) 1; else 2;` takes the else branch via JumpIfFalse.
#[test]
fn trampoline_executes_if_false_alternate() {
    let unit = compile_test_unit(8, "if (false) { 1 } else { 2 }");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm
        .evaluate_script(agent, realm, &unit)
        .expect("trampoline-dispatch should execute if/else cleanly");

    assert_eq!(result, Value::from_smi(2));
}

// =====================================================================
// sub-4 (lyng-54em): arithmetic family parity tests
// =====================================================================

#[test]
fn trampoline_executes_smi_add() {
    let unit = compile_test_unit(9, "5 + 7");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm
        .evaluate_script(agent, realm, &unit)
        .expect("trampoline-dispatch should execute `5 + 7` cleanly");

    assert_eq!(result, Value::from_smi(12));
}

#[test]
fn trampoline_executes_chained_smi_add() {
    let unit = compile_test_unit(10, "1 + 2 + 3 + 4 + 5");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm
        .evaluate_script(agent, realm, &unit)
        .expect("trampoline-dispatch should execute chained add cleanly");

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn trampoline_executes_negate_literal() {
    let unit = compile_test_unit(11, "-7");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm
        .evaluate_script(agent, realm, &unit)
        .expect("trampoline-dispatch should execute `-7` cleanly");

    assert_eq!(result, Value::from_smi(-7));
}

#[test]
fn trampoline_executes_smi_mul_and_sub() {
    let unit = compile_test_unit(12, "10 * 3 - 4");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm
        .evaluate_script(agent, realm, &unit)
        .expect("trampoline-dispatch should execute `10 * 3 - 4` cleanly");

    assert_eq!(result, Value::from_smi(26));
}

#[test]
fn trampoline_executes_strict_equal_true() {
    let unit = compile_test_unit(13, "3 === 3");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn trampoline_executes_strict_equal_false() {
    let unit = compile_test_unit(14, "3 === 4");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    assert_eq!(result, Value::from_bool(false));
}

#[test]
fn trampoline_executes_less_than() {
    let unit = compile_test_unit(15, "1 < 2");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn trampoline_executes_if_with_comparison_chooses_consequent() {
    let unit = compile_test_unit(16, "if (10 > 3) { 1 } else { 2 }");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    assert_eq!(result, Value::from_smi(1));
}

#[test]
fn trampoline_executes_bitwise_ops() {
    let unit = compile_test_unit(17, "(5 & 3) + (5 | 3) + (5 ^ 3)");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    // (5 & 3) = 1; (5 | 3) = 7; (5 ^ 3) = 6 → 14
    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    assert_eq!(result, Value::from_smi(14));
}

#[test]
fn trampoline_executes_shifts() {
    let unit = compile_test_unit(18, "(1 << 4) + (32 >> 2)");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    // (1 << 4) = 16; (32 >> 2) = 8 → 24
    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    assert_eq!(result, Value::from_smi(24));
}

#[test]
fn trampoline_executes_smi_mod() {
    let unit = compile_test_unit(19, "17 % 5");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    assert_eq!(result, Value::from_smi(2));
}

#[test]
fn trampoline_executes_exp_smi() {
    let unit = compile_test_unit(20, "2 ** 10");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    assert_eq!(result, Value::from_smi(1024));
}

// =====================================================================
// sub-5 (lyng-5mqv): property access parity tests
// =====================================================================

#[test]
fn trampoline_executes_object_literal_and_property_load() {
    let unit = compile_test_unit(21, "({ x: 42 }).x");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    assert_eq!(result, Value::from_smi(42));
}

#[test]
fn trampoline_executes_multi_property_load() {
    let unit = compile_test_unit(22, "({ x: 1, y: 2, z: 3 }).x + ({ x: 1, y: 2, z: 3 }).z");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    assert_eq!(result, Value::from_smi(4));
}

#[test]
fn trampoline_executes_array_literal_and_indexed_load() {
    let unit = compile_test_unit(23, "[10, 20, 30][1]");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    assert_eq!(result, Value::from_smi(20));
}

#[test]
fn trampoline_executes_undefined_via_load_global() {
    let unit = compile_test_unit(24, "undefined");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    assert_eq!(result, Value::undefined());
}

#[test]
fn trampoline_executes_nan_via_load_global() {
    let unit = compile_test_unit(25, "NaN");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    assert!(result.is_nan(), "NaN global should resolve to a NaN Value");
}

#[test]
fn trampoline_executes_in_operator() {
    let unit = compile_test_unit(26, "'x' in { x: 1, y: 2 }");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn trampoline_executes_delete_property() {
    let unit = compile_test_unit(27, "delete ({ x: 42 }).x");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm.evaluate_script(agent, realm, &unit).unwrap();
    assert_eq!(result, Value::from_bool(true));
}
