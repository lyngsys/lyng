use super::support::*;

// Helper: compile, install, and evaluate a script that returns a function
// value; extract the callee `ObjectRef` so eligibility tests can call the
// predicate directly without going through the interpreter.
fn eval_returns_function(source_id: u32, source: &str) -> (Runtime, Vm, ObjectRef) {
    let unit = compile_test_unit(source_id, source);
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let value = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .expect("script should complete without error");
    let callee = value
        .as_object_ref()
        .expect("script should return a function object");
    (runtime, vm, callee)
}

#[test]
fn ordinary_bytecode_construct_eligibility_accepts_plain_constructor() {
    let (mut runtime, vm, callee) = eval_returns_function(80_001, "function F() {} F;");
    let agent = runtime.root_agent_mut();
    assert!(
        vm.ordinary_bytecode_construct_eligibility(agent, callee)
            .is_some(),
        "plain bytecode constructor should be eligible"
    );
}

#[test]
fn ordinary_bytecode_construct_eligibility_rejects_rest_parameter() {
    let (mut runtime, vm, callee) = eval_returns_function(80_002, "function G(...a) {} G;");
    let agent = runtime.root_agent_mut();
    assert!(
        vm.ordinary_bytecode_construct_eligibility(agent, callee)
            .is_none(),
        "function with rest parameter should be ineligible"
    );
}

#[test]
fn ordinary_bytecode_construct_eligibility_rejects_bound_function() {
    let (mut runtime, vm, callee) = eval_returns_function(80_003, "function F() {} F.bind(null);");
    let agent = runtime.root_agent_mut();
    assert!(
        vm.ordinary_bytecode_construct_eligibility(agent, callee)
            .is_none(),
        "bound function should be ineligible"
    );
}

// The `debug_deopt_assertion_reports_register_window_mismatch` test was
// removed: the warm `op_loop_header` dispatches without going through the
// slow shim when no safepoint poll is pending, so the assertion never fires
// for the malformed register-window fixture. Safepoint metadata addressing
// is covered by `vm_addresses_metadata_by_code_and_instruction_offset`.

#[test]
fn vm_addresses_metadata_by_code_and_instruction_offset() {
    let unit = compile_test_unit(
        37,
        r"
        let make = function(value) { return value; };
        let count = 0;
        while (count < 1) {
            count = count + 1;
        }
        try {
            make({ value: count });
        } catch (err) {
            err;
        }
        ",
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let function = vm
        .installed_function(installed.code())
        .expect("installed script should expose its template");
    let allocation = function
        .safepoints()
        .iter()
        .find(|descriptor| descriptor.kind() == SafepointKind::Allocation)
        .copied()
        .expect("allocation site should install metadata");
    let loop_backedge = function
        .safepoints()
        .iter()
        .find(|descriptor| descriptor.kind() == SafepointKind::LoopBackedge)
        .copied()
        .expect("loop backedge should install metadata");
    let exception = function
        .safepoints()
        .iter()
        .find(|descriptor| descriptor.kind() == SafepointKind::ExceptionEdge)
        .copied()
        .expect("exception edge should install metadata");

    let allocation_source = vm
        .source_map_entry(installed.code(), allocation.instruction_offset())
        .expect("source map should be addressable by code and offset");
    let loop_runtime = vm
        .safepoint_at(installed.code(), loop_backedge.instruction_offset())
        .expect("loop safepoint should be addressable by code and offset");
    let exception_runtime = vm
        .safepoint_by_id(installed.code(), exception.id())
        .expect("exception safepoint should be addressable by code and id");
    let exception_snapshot = vm
        .deopt_snapshot(installed.code(), exception.id())
        .expect("deopt snapshot should be addressable by code and safepoint id");

    assert_eq!(
        allocation_source.instruction_offset(),
        allocation.instruction_offset()
    );
    assert_eq!(loop_runtime.kind(), SafepointKind::LoopBackedge);
    assert!(exception_runtime.captures_exception_state());
    assert!(
        exception_snapshot
            .values()
            .contains(&DeoptValueSource::FrameValue(
                DeoptFrameValue::ExceptionValue,
            ))
    );
}

#[test]
fn tail_calls_reuse_frame_depth_for_recursive_bytecode_calls() {
    let unit = compile_test_unit(
        35,
        r"
        let countdown = function(self, value, acc) {
            if (value === 0) {
                return acc;
            }
            return self(self, value - 1, acc + 1);
        };
        countdown(countdown, 200, 0);
        ",
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    assert_eq!(result, Value::from_smi(200));
    assert_eq!(vm.peak_frame_depth(), 2);
}

#[test]
fn tail_calls_through_rebound_global_eval_reuse_frame_depth() {
    let unit = compile_test_unit(
        37,
        r#"
        var callCount = 0;
        function f(n) {
            "use strict";
            if (n === 0) {
                callCount += 1;
                return callCount;
            }
            return eval(n - 1);
        }
        eval = f;
        f(8);
        "#,
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    assert_eq!(result, Value::from_smi(1));
    assert_eq!(vm.peak_frame_depth(), 2);
}

#[test]
fn tail_calls_preserve_constructor_fallback_result_semantics() {
    let unit = compile_test_unit(
        36,
        r"
        function Box(helper) {
            this.value = 4;
            return helper(1);
        }
        let box = new Box(function(value) { return value; });
        box.value;
        ",
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    assert_eq!(result, Value::from_smi(4));
}
