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
