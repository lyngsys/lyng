//! Tests that the realm's global object is a cell-backed dictionary from realm
//! creation, so global `var`/`function` bindings (and writes through them) are
//! stored as [`lyng_objects::NamedPropertyValue::DataCell`] entries.

use super::support::*;

/// A global `var` declaration in a freshly bootstrapped realm should be stored
/// as a cell-backed dictionary entry on the global object.
#[test]
fn small_script_global_is_cell_backed_from_start() {
    let unit = compile_test_unit(7100, "var a = 1; a");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let a_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "a"));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    assert_eq!(result, Value::from_smi(1));
    assert!(
        agent
            .cell_backed_entry(realm.global_object(), PropertyKey::from_atom(a_name))
            .is_some(),
        "global `a` should be backed by a primitive-value cell"
    );
}

/// Writing to a global `var` should keep the entry cell-backed and observe the
/// new value.
#[test]
fn global_var_write_through_cell() {
    let unit = compile_test_unit(7101, "var x = 1; x = 2; x");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let x_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "x"));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    assert_eq!(result, Value::from_smi(2));
    assert!(
        agent
            .cell_backed_entry(realm.global_object(), PropertyKey::from_atom(x_name))
            .is_some(),
        "global `x` should remain backed by a primitive-value cell after a write"
    );
}
