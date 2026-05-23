//! DSL-0b (B17/B18) dual-write invariant test for the flat
//! `FeedbackEntry` array vs the legacy
//! `Vec<Option<FeedbackSiteState>>`.
//!
//! After the α path runs a hot loop, the legacy `FeedbackVector`
//! slot and the new flat `FeedbackEntry` slot must observe the same
//! IC state — including all Phase 3f packed sidecars. This test
//! exercises:
//!
//! - **SMI-add hot loop** (B17): drives `record_feedback_slot`
//!   through the arithmetic feedback path and confirms flat ==
//!   legacy after warmup + steady-state.
//!
//! - **Polymorphic property-access hot loop** (B18): drives the
//!   named-property IC through enough receiver shapes to enter the
//!   polymorphic state, exercising `observe_named_property_slow_path`
//!   and the packed `polymorphic_fast` sidecar. The invariant check
//!   is identical (legacy bits == flat bits) because both storages
//!   hold the same `FeedbackSiteState` per slot — the wrap into
//!   `FeedbackEntry { state }` carries every sidecar automatically
//!   per design §9.

use lyng_common::{AtomTable, SourceId};
use lyng_compiler::compile_script;
use lyng_env::Runtime;
use lyng_host::NoopHostHooks;
use lyng_parser::parse_script;
use lyng_sema::analyze_script;
use lyng_vm::Vm;

fn run_script_n_times(source: &str, iterations: usize) -> (Vm, lyng_types::CodeRef) {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, SourceId::new(1), source);
    assert!(!parsed.diagnostics.has_errors(), "parse error");
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors(), "sema error");
    let unit = compile_script(&parsed, &sema, &mut atoms).expect("compile");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let installed = vm
        .install_script(agent, realm.id(), &unit)
        .expect("install");

    // Run enough iterations to cross the feedback warmup threshold
    // and exercise the IC fast / slow paths.
    for _ in 0..iterations {
        let _ = vm
            .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
            .expect("evaluate");
    }
    (vm, installed.code())
}

#[test]
fn dual_write_keeps_smi_add_legacy_and_flat_in_sync() {
    // SMI-add hot loop. The inner `+` exercises arithmetic feedback
    // via `record_feedback_slot`; iteration count must clear the
    // FEEDBACK_ALLOCATION_THRESHOLD so the legacy vector allocates
    // and the dual-write mirror kicks in.
    let (vm, code) = run_script_n_times(
        r"
            (function add(a, b) {
                let sum = 0;
                for (let i = 0; i < 100; i = i + 1) {
                    sum = sum + a + b;
                }
                return sum;
            })(1, 2);
        ",
        4,
    );

    match vm.feedback_flat_matches_legacy(code) {
        Ok(()) => {}
        Err((slot, diff)) => panic!(
            "dual-write mismatch after SMI-add hot loop: slot {} -> {}",
            slot, diff
        ),
    }
}

#[test]
fn dual_write_keeps_polymorphic_property_access_legacy_and_flat_in_sync() {
    // Polymorphic property-access hot loop. The body's `obj.x`
    // walks three distinct receiver shapes (`{x}`, `{x, y}`,
    // `{x, y, z}`), driving the named-property IC into the
    // polymorphic state and exercising the Phase 3f packed
    // `polymorphic_fast` sidecar. Both legacy and flat storages
    // hold the same `FeedbackSiteState`, so the wrap into
    // `FeedbackEntry { state }` carries every sidecar automatically
    // per design §9.
    let (vm, code) = run_script_n_times(
        r"
            (function poly() {
                let total = 0;
                const a = { x: 1 };
                const b = { x: 2, y: 0 };
                const c = { x: 3, y: 0, z: 0 };
                for (let i = 0; i < 50; i = i + 1) {
                    total = total + a.x + b.x + c.x;
                }
                return total;
            })();
        ",
        4,
    );

    match vm.feedback_flat_matches_legacy(code) {
        Ok(()) => {}
        Err((slot, diff)) => panic!(
            "dual-write mismatch after polymorphic property-access loop: slot {} -> {}",
            slot, diff
        ),
    }
}

#[test]
fn dual_write_holds_on_cold_install_with_unallocated_legacy_vector() {
    // Compile + install but execute zero times: the legacy vector
    // stays in its unallocated sentinel (sites.len() == 0) while
    // the flat array carries `function.feedback_slot_count()`
    // default `None`-state entries. The invariant matcher accepts
    // this asymmetry and confirms every flat slot is `None`.
    let mut atoms = AtomTable::new();
    let parsed = parse_script(
        &mut atoms,
        SourceId::new(2),
        "(function id(x) { return x; })(7);",
    );
    let sema = analyze_script(&parsed, &atoms);
    let unit = compile_script(&parsed, &sema, &mut atoms).expect("compile");
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm");
    let mut vm = Vm::new();
    let installed = vm
        .install_script(agent, realm.id(), &unit)
        .expect("install");

    match vm.feedback_flat_matches_legacy(installed.code()) {
        Ok(()) => {}
        Err((slot, diff)) => panic!(
            "cold install should leave flat fully-default: slot {} -> {}",
            slot, diff
        ),
    }
}
