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

/// Declaring many global `let` bindings and reading specific ones back must
/// resolve to the correct values. This is the behavioral guard for the O(1)
/// `name -> binding` index that replaces the previous linear scan.
#[test]
fn many_global_lexical_bindings_resolve_correctly() {
    let mut src = String::new();
    for i in 0..50 {
        writeln!(src, "let l{i} = {i};").unwrap();
    }
    // Read a spread of specific bindings (first, middle, last) and combine them
    // into a single distinct number: l0 + l25 * 100 + l49 * 10000 = 492500.
    src.push_str("l0 + l25 * 100 + l49 * 10000;");
    let unit = compile_test_unit(7110, &src);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm
        .evaluate_script(agent, realm, &unit)
        .run()
        .expect("many global lexicals should resolve");

    assert_eq!(result, Value::from_smi(492_500));
}

/// `let` and `const` global declarations resolve and combine correctly.
#[test]
fn global_let_const_still_work() {
    let unit = compile_test_unit(7111, "let a = 1; const b = 2; a + b");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm
        .evaluate_script(agent, realm, &unit)
        .run()
        .expect("global let/const should resolve");

    assert_eq!(result, Value::from_smi(3));
}

/// The temporal dead zone is unaffected by the resolution index: reading a
/// `let` binding before its declaration must throw a `ReferenceError`.
#[test]
fn global_lexical_tdz_preserved() {
    let unit = compile_test_unit(
        7112,
        "var ok = 0; try { a; } catch (e) { ok = e instanceof ReferenceError ? 1 : 2; } let a = 1; ok",
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();

    let result = vm
        .evaluate_script(agent, realm, &unit)
        .run()
        .expect("script with TDZ catch should complete");

    assert_eq!(result, Value::from_smi(1));
}

use crate::vm::ic_state::GlobalCellTarget;

/// Helper: the feedback slot of the last (trailing) `NamedPropertyLoad` site in
/// the script entry function — the `LoadGlobal` site for the trailing global
/// read (the `.rev().find(...)` walk returns the last site).
fn entry_named_load_slot(unit: &CompiledScriptUnit) -> FeedbackSlotId {
    let entry = unit.function(unit.entry()).unwrap();
    entry
        .feedback_sites()
        .iter()
        .rev()
        .find(|descriptor| descriptor.kind() == FeedbackSiteKind::NamedPropertyLoad)
        .map(|descriptor| descriptor.slot())
        .expect("entry script should contain a named-load site for the global access")
}

/// `var x = 5; x; x` — the second `LoadGlobal` should resolve through the cell
/// IC, leaving a `Cell` target cached for the load site.
#[test]
fn load_global_var_hits_cell_ic() {
    let unit = compile_test_unit(7200, "var x = 5; x; x");
    let slot = entry_named_load_slot(&unit);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    assert_eq!(result, Value::from_smi(5));
    let ic = vm
        .global_cell_ic_state(installed.code(), slot)
        .expect("global cell IC should be installed for the var load site");
    assert!(
        matches!(ic.target, GlobalCellTarget::Cell(_)),
        "global `var` should cache a Cell target, got {:?}",
        ic.target
    );
}

/// Task 5: when the cold path resolves a `LoadGlobal` site to a `Cell` target it
/// must project mode-7 metadata into the site's `PropertyMetadata` so a future
/// asm hit can serve the load inline. `var g = 7; g; g` resolves the trailing
/// load to a Cell; the load site's metadata must carry mode 7, a non-zero cell
/// ref in `handler_bits`, and the live (captured) generation.
#[test]
fn cold_path_cell_resolution_projects_mode_7_metadata() {
    use crate::vm::metadata_table::LLINT_IC_MODE_GLOBAL_CELL_LOAD;

    let unit = compile_test_unit(7205, "var g = 7; g; g");
    let slot = entry_named_load_slot(&unit);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    assert_eq!(result, Value::from_smi(7));

    // The cold-path resolution must have cached a Cell target...
    let ic = vm
        .global_cell_ic_state(installed.code(), slot)
        .expect("global cell IC should be installed for the var load site");
    assert!(
        matches!(ic.target, GlobalCellTarget::Cell(_)),
        "global `var` should cache a Cell target, got {:?}",
        ic.target
    );

    // ...and projected mode-7 metadata into the load site.
    let meta = *vm
        .metadata_table(installed.code())
        .expect("metadata table should be installed for the script code")
        .property(slot.get());
    assert_eq!(
        meta.mode, LLINT_IC_MODE_GLOBAL_CELL_LOAD,
        "Cell resolution must project mode 7 into the load site metadata"
    );
    assert_ne!(meta.handler_bits, 0, "cell ref must be projected");
    assert_eq!(
        meta.generation,
        vm.dsl_global_ic_generation(),
        "captured generation must equal the live mirror (no structural change)"
    );
}

/// A global lexical binding read from a *different* compilation unit lowers to
/// `LoadGlobal` (cross-unit references resolve as `ResolutionKind::Global`), so
/// the cell IC should cache an `EnvSlot` target. (Same-unit lexical reads lower
/// to a direct env-slot load and never reach `LoadGlobal`.)
#[test]
fn load_global_lexical_hits_env_slot_ic() {
    let decl = compile_test_unit(7201, "let y = 9;");
    let read = compile_test_unit(7211, "y; y");
    let slot = entry_named_load_slot(&read);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut vm = Vm::new();
    // First unit declares the global lexical binding `y`. Use `evaluate_script`
    // so the global lexical instantiation plan runs and `y` is bound + set.
    vm.evaluate_script(agent, realm, &decl).run().unwrap();

    // Second unit reads it via `LoadGlobal`.
    let read_installed = vm.install_script(agent, realm.id(), &read).unwrap();
    let result = vm
        .evaluate_installed(
            agent,
            read_installed,
            realm.global_env(),
            realm.global_env(),
        )
        .run()
        .unwrap();

    assert_eq!(result, Value::from_smi(9));
    let ic = vm
        .global_cell_ic_state(read_installed.code(), slot)
        .expect("global cell IC should be installed for the lexical load site");
    assert!(
        matches!(ic.target, GlobalCellTarget::EnvSlot(_, _)),
        "global `let` read cross-unit should cache an EnvSlot target, got {:?}",
        ic.target
    );
}

/// A configurable global created via assignment (`globalThis.foo = 7`) is read
/// back twice and the load site caches an IC — proving configurable globals ARE
/// cached under the generation scheme (no per-binding configurability gating).
#[test]
fn load_global_configurable_builtin_is_cached() {
    let unit = compile_test_unit(7202, "globalThis.foo = 7; foo; foo");
    let slot = entry_named_load_slot(&unit);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    assert_eq!(result, Value::from_smi(7));
    let ic = vm
        .global_cell_ic_state(installed.code(), slot)
        .expect("configurable global load site should install a cell IC");
    assert!(
        matches!(ic.target, GlobalCellTarget::Cell(_)),
        "configurable global should cache a Cell target, got {:?}",
        ic.target
    );
}

/// Deleting a sloppy global between cached reads must NOT use the stale IC: the
/// generation bump on delete forces re-resolution, so the final read sees the
/// binding gone (`typeof x === "undefined"`) instead of a stale value / crash.
#[test]
fn deleting_global_does_not_use_stale_ic() {
    let unit = compile_test_unit(
        7203,
        "x = 1; function r(){ return typeof x; } r(); r(); delete x; r();",
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut vm = Vm::new();
    let result = vm
        .evaluate_script(agent, realm, &unit)
        .run()
        .expect("delete-global script should complete");

    let string = result
        .as_string_ref()
        .expect("typeof should return a string");
    let view = agent
        .heap()
        .view()
        .string_view(string)
        .expect("string should exist in the heap");
    assert_eq!(
        decode_string(&view),
        "undefined",
        "after delete, the cached site must re-resolve and see the binding gone"
    );
}

/// A global value reassignment between cached reads must be observed live
/// through the IC (the cell is read on every hit; value changes do not bump the
/// generation).
#[test]
fn global_value_reassignment_seen_through_ic() {
    let unit = compile_test_unit(
        7204,
        "var v = 1; function r(){ return v; } var a = r(); v = 2; var b = r(); a * 10 + b",
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut vm = Vm::new();
    let result = vm
        .evaluate_script(agent, realm, &unit)
        .run()
        .expect("value-reassignment script should complete");

    // a == 1 (first read), b == 2 (after reassignment) => 1*10 + 2 == 12.
    assert_eq!(result, Value::from_smi(12));
}

/// A `[[Delete]]` of a cell-backed global property (the path reached by
/// `delete globalThis.x`, `Reflect.deleteProperty`, and qualified
/// `delete this.x`, all of which funnel through `Agent::delete` rather than the
/// sloppy unqualified `delete x` statement) frees the entry's backing cell and
/// MUST bump the global structure generation, so any per-site `Cell` IC
/// re-resolves instead of dereferencing the freed cell. This asserts the
/// invalidation contract directly and deterministically at the `Agent::delete`
/// chokepoint.
#[test]
fn delete_of_cell_backed_global_bumps_structure_generation() {
    let unit = compile_test_unit(7299, "var x = 1; x");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let global_env = realm.global_env();
    let global_object = realm.global_object();
    let x_name = unit_runtime_atom(agent, &unit, unit_atom(&unit, "x"));

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    // Precondition: `x` is a cell-backed entry on the global object.
    assert!(
        agent
            .cell_backed_entry(global_object, PropertyKey::from_atom(x_name))
            .is_some(),
        "global `x` should be cell-backed before the delete"
    );

    let gen_before = agent.global_structure_generation(global_env);
    let deleted = agent
        .delete(
            global_object,
            PropertyKey::from_atom(x_name),
            &mut NoopAdaptiveProtoLoadDispatch,
        )
        .expect("delete should succeed");
    assert!(deleted, "configurable global `x` should be deletable");

    assert!(
        agent.global_structure_generation(global_env) > gen_before,
        "deleting a cell-backed global must bump the structure generation \
         (else a cached Cell IC reads the freed cell)"
    );
    assert!(
        agent
            .cell_backed_entry(global_object, PropertyKey::from_atom(x_name))
            .is_none(),
        "the cell-backed entry should be gone after delete"
    );
}

/// Task 4: the Vm-side global-IC generation mirror must equal the live agent
/// generation after a script that triggers a structural global bump. Here a
/// sloppy global creation + `delete x` both bump the generation through the
/// slow path; after the run the mirror (refreshed at the slow-path choke point
/// and re-primed at every run entry) must match.
#[test]
fn vm_global_ic_generation_mirror_tracks_structural_bumps() {
    let unit = compile_test_unit(7300, "var g = 1; delete globalThis.g; g = 2; g");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let global_env = realm.global_env();

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    assert_eq!(
        vm.dsl_global_ic_generation(),
        agent.global_structure_generation(global_env),
        "after a structural-bumping script the mirror must equal the live generation"
    );
    // And the bump actually happened (the mirror is not coherent by being a
    // trivial 0 == 0 — the generation moved off its initial value).
    assert!(
        agent.global_structure_generation(global_env) > 0,
        "the delete should have bumped the live generation above the baseline"
    );
}

/// Task 4 BACKSTOP: runtime `Object.defineProperty(globalThis, ...)` AND
/// `delete globalThis.x` performed mid-dispatch (inside the executed script,
/// from a function body that runs after earlier reads) must keep the mirror
/// coherent with the live generation through the slow-path choke point, and
/// reads must return the correct post-mutation values. This is the whole point:
/// a stale-low mirror at an asm mode-7 hit would dereference a freed/reused
/// value cell.
#[test]
fn mirror_stays_coherent_across_runtime_define_and_delete() {
    let unit = compile_test_unit(
        7301,
        "
        var g = 5;
        var r0 = g;                         // read 5
        Object.defineProperty(globalThis, 'g', { value: 6, writable: true, configurable: true });
        var r1 = g;                         // must read 6
        delete globalThis.g;
        g = 7;                              // recreate sloppy global
        var r2 = g;                         // must read 7
        r0 * 100 + r1 * 10 + r2             // 5*100 + 6*10 + 7 = 567
        ",
    );

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let global_env = realm.global_env();

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .expect("runtime define/delete script should complete");

    // Reads observed the correct values across the define + delete + recreate.
    assert_eq!(
        result,
        Value::from_smi(567),
        "reads must observe 5 -> 6 (defineProperty) -> 7 (delete + recreate)"
    );
    // The mirror is coherent with the live generation after the run — the
    // slow-path choke point re-synced it after each structural mutation.
    assert_eq!(
        vm.dsl_global_ic_generation(),
        agent.global_structure_generation(global_env),
        "mirror must equal the live generation after runtime define + delete"
    );
}

/// Task 4: the mirror is primed at run entry, so after a run that pre-declares
/// several globals (whose instantiation bumps the structure generation) the
/// mirror equals the live generation — and crucially is NOT left at 0.
#[test]
fn baseline_global_ic_generation_primed_at_entry() {
    let mut src = String::new();
    for i in 0..8 {
        writeln!(src, "var v{i} = {i};").unwrap();
    }
    src.push_str("v0 + v7");
    let unit = compile_test_unit(7302, &src);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let global_env = realm.global_env();

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    vm.evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    let live = agent.global_structure_generation(global_env);
    assert_eq!(
        vm.dsl_global_ic_generation(),
        live,
        "the mirror must be primed at run entry to the live generation"
    );
    assert!(
        live > 0,
        "pre-declared globals should have bumped the generation during instantiation, \
         so the primed baseline is non-zero (proving priming actually ran)"
    );
}

/// Task 6 GUARD: serving a mode-7 (cell-backed) global read via the probe's thin
/// fast path must remain correct across:
///  - a plain warmed read (fast read returns the value),
///  - a reassignment with no structural change (generation unchanged → the live
///    cell read reflects the new value),
///  - a structural change (delete + re-`var`) that bumps the generation, so the
///    fast read MUST bail and re-resolve rather than serve a stale cell.
///
/// Encodes each read into the script result: `a` (=5) read after warming, `b`
/// (=6) read after a plain reassignment, `c` (=7) read after delete+recreate.
/// `a*100 + b*10 + c == 567` proves all three reads observed the correct live
/// value. Passes on the pre-refactor code too (it is a correctness guard).
#[test]
fn mode_7_fast_read_returns_correct_value_through_mutation_and_invalidation() {
    let src = "var g = 5; g; var a = g;\n\
               g = 6; var b = g;\n\
               delete globalThis.g; var g = 7; var c = g;\n\
               a * 100 + b * 10 + c";
    let unit = compile_test_unit(7303, src);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    assert_eq!(
        result,
        Value::from_smi(567),
        "mode-7 reads must be live across reassignment (5->6, no gen bump) and \
         re-resolve after a structural delete+recreate (gen bump → 7), not serve a stale cell"
    );
}

/// Task 8 ASM STALENESS (a, delete+recreate): warm a mode-7 site through a
/// function called twice, then `delete globalThis.g` + recreate the global
/// with a NEW value. The asm hit's generation guard must bail on the bump and
/// re-resolve, so the post-recreate read observes the new value — never the
/// stale (freed) cell. Distinct from the trailing-load variant above by
/// driving the warm reads through a called function (the canonical mode-7
/// warming shape).
#[test]
fn asm_mode_7_bails_on_delete_recreate() {
    let src = "var g = 11; function r(){ return g; } var w0 = r(); r();\n\
               delete globalThis.g; var g = 22; var w1 = r();\n\
               w0 * 1000 + w1";
    let unit = compile_test_unit(7310, src);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    // w0 == 11 (warmed read), w1 == 22 (after delete+recreate, re-resolved).
    assert_eq!(
        result,
        Value::from_smi(11_022),
        "after delete+recreate the warmed asm mode-7 site must re-resolve to the new value, not serve a stale cell"
    );
}

/// Task 8 ASM STALENESS (b, data→accessor): warm a mode-7 data-cell site, then
/// `Object.defineProperty(globalThis, 'g', { get() {...} })` to convert the
/// data global into an accessor. This bumps the structure generation, so the
/// next read MUST bail out of the cached cell and invoke the getter — returning
/// the getter's value, not the stale data cell's value.
#[test]
fn asm_mode_7_bails_on_data_to_accessor_redefine() {
    let src = "var g = 5; function r(){ return g; } var w0 = r(); r();\n\
               Object.defineProperty(globalThis, 'g', { get() { return 99; }, configurable: true });\n\
               var w1 = r();\n\
               w0 * 1000 + w1";
    let unit = compile_test_unit(7311, src);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut vm = Vm::new();
    let installed = vm.install_script(agent, realm.id(), &unit).unwrap();
    let result = vm
        .evaluate_installed(agent, installed, realm.global_env(), realm.global_env())
        .run()
        .unwrap();

    // w0 == 5 (warmed data read), w1 == 99 (getter result after data→accessor).
    assert_eq!(
        result,
        Value::from_smi(5_099),
        "data→accessor redefine must invalidate the cached cell so the read invokes the getter (99), not the stale data value"
    );
}

/// Task 8 ASM STALENESS (c, let-shadow): warm a mode-7 site reading a `var`
/// global from one unit, then evaluate a second unit that declares a global
/// `let` of the same name (shadowing the object property). Installing a global
/// lexical binding that shadows the cell-backed property bumps the structure
/// generation, so a subsequent read from a third unit must re-resolve to the
/// lexical binding's value rather than serve the stale data cell.
#[test]
fn asm_mode_7_bails_on_global_let_shadow() {
    let warm = compile_test_unit(7312, "var s = 1; s; s");
    let shadow = compile_test_unit(7313, "let s = 2;");
    let read = compile_test_unit(7314, "s");

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");

    let mut vm = Vm::new();

    // Warm: `var s = 1` then two reads install + warm the mode-7 site.
    let warm_installed = vm.install_script(agent, realm.id(), &warm).unwrap();
    let warm_result = vm
        .evaluate_installed(
            agent,
            warm_installed,
            realm.global_env(),
            realm.global_env(),
        )
        .run()
        .unwrap();
    assert_eq!(
        warm_result,
        Value::from_smi(1),
        "warmed var read should see 1"
    );

    // Shadow: a global `let s = 2` shadows the var/property binding (gen bump).
    vm.evaluate_script(agent, realm, &shadow).run().unwrap();

    // Read from a fresh unit: must resolve to the lexical binding (2), not the
    // stale data cell (1).
    let read_installed = vm.install_script(agent, realm.id(), &read).unwrap();
    let read_result = vm
        .evaluate_installed(
            agent,
            read_installed,
            realm.global_env(),
            realm.global_env(),
        )
        .run()
        .unwrap();

    assert_eq!(
        read_result,
        Value::from_smi(2),
        "after a global `let` shadows the var binding, the read must re-resolve to the lexical value (2), not the stale cell (1)"
    );
}
