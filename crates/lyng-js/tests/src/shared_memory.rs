use lyng_js_builtins::BootstrapMode;
use lyng_js_common::{AtomTable, SourceId};
use lyng_js_compiler::compile_script;
use lyng_js_env::Runtime;
use lyng_js_gc::PrimitiveMutator;
use lyng_js_host::NoopHostHooks;
use lyng_js_objects::{
    InternalMethodResult, NativeCallRequest, NativeConstructRequest, NativeFunctionRegistry,
    ObjectRuntime,
};
use lyng_js_parser::parse_script;
use lyng_js_sema::analyze_script;
use lyng_js_types::{ObjectRef, Value};
use lyng_js_vm::Vm;

#[derive(Default)]
struct RejectingRegistry;

impl NativeFunctionRegistry for RejectingRegistry {
    fn call(
        &mut self,
        _runtime: &mut ObjectRuntime,
        _heap: &mut PrimitiveMutator<'_>,
        _request: NativeCallRequest<'_>,
    ) -> InternalMethodResult<Value> {
        panic!("unexpected native call during shared-memory test");
    }

    fn construct(
        &mut self,
        _runtime: &mut ObjectRuntime,
        _heap: &mut PrimitiveMutator<'_>,
        _request: NativeConstructRequest<'_>,
    ) -> InternalMethodResult<ObjectRef> {
        panic!("unexpected native construct during shared-memory test");
    }
}

fn compile_unit(source: &str, atoms: &mut AtomTable) -> lyng_js_bytecode::CompiledScriptUnit {
    let parsed = parse_script(atoms, SourceId::new(0), source);
    assert!(
        !parsed.diagnostics.has_errors(),
        "unexpected parse errors: {:?}",
        parsed.diagnostics.as_slice()
    );
    let sema = analyze_script(&parsed, atoms);
    assert!(
        !sema.diagnostics.has_errors(),
        "unexpected sema errors: {:?}",
        sema.diagnostics.as_slice()
    );
    compile_script(&parsed, &sema, atoms).expect("script should lower")
}

fn run_spec_script(source: &str) -> Value {
    let mut atoms = AtomTable::new();
    let unit = compile_unit(source, &mut atoms);
    let mut runtime = Runtime::new(NoopHostHooks);
    let host = NoopHostHooks;
    let mut registry = RejectingRegistry;
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let _ = vm
        .bootstrap_realm(agent, realm.id(), BootstrapMode::SpecOnly)
        .expect("spec bootstrap should succeed");
    let value = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .expect("script should execute");
    vm.checkpoint_promise_jobs(agent, &host, &mut registry)
        .expect("promise jobs should drain after shared-memory script");
    value
}

fn run_spec_script_with_readback(source: &str, readback: &str) -> Value {
    let mut atoms = AtomTable::new();
    let unit = compile_unit(source, &mut atoms);
    let readback = compile_unit(readback, &mut atoms);
    let mut runtime = Runtime::new(NoopHostHooks);
    let host = NoopHostHooks;
    let mut registry = RejectingRegistry;
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let _ = vm
        .bootstrap_realm(agent, realm.id(), BootstrapMode::SpecOnly)
        .expect("spec bootstrap should succeed");
    let _ = vm
        .evaluate_script_with_registry_and_host(agent, realm, &unit, &host, &mut registry)
        .expect("script should execute");
    vm.checkpoint_promise_jobs(agent, &host, &mut registry)
        .expect("promise jobs should drain after shared-memory script");
    let value = vm
        .evaluate_script_with_registry_and_host(agent, realm, &readback, &host, &mut registry)
        .expect("readback script should execute");
    vm.checkpoint_promise_jobs(agent, &host, &mut registry)
        .expect("promise jobs should drain after shared-memory readback");
    value
}

#[test]
fn phase6_shared_array_buffer_bootstraps_slices_and_builds_shared_views() {
    let result = run_spec_script(
        r#"
        let score = 0;
        let sab = new SharedArrayBuffer(8);
        let view = new Uint8Array(sab);
        view[0] = 7;
        view[1] = 9;
        let slice = sab.slice(0, 2);
        let sliceView = new Uint8Array(slice);
        class CustomSAB extends SharedArrayBuffer {}
        let custom = new CustomSAB(4);

        if (typeof SharedArrayBuffer === "function") score += 1;
        if (sab.byteLength === 8) score += 2;
        if (Object.prototype.toString.call(sab) === "[object SharedArrayBuffer]") score += 4;
        if (custom instanceof SharedArrayBuffer && custom instanceof CustomSAB && custom.byteLength === 4) score += 8;
        if (slice !== sab && slice.byteLength === 2 && sliceView[0] === 7 && sliceView[1] === 9) score += 16;
        if (new DataView(sab).byteLength === 8) score += 32;

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(63));
}

#[test]
fn phase6_atomics_integer_surface_loads_stores_and_updates_shared_typed_arrays() {
    let result = run_spec_script(
        r#"
        let sab = new SharedArrayBuffer(16);
        let i32 = new Int32Array(sab);
        let u32 = new Uint32Array(sab);
        let score = 0;

        if (Atomics.store(i32, 0, 7) === 7) score += 1;
        if (Atomics.load(i32, 0) === 7) score += 2;
        if (Atomics.add(i32, 0, 5) === 7) score += 4;
        if (Atomics.sub(i32, 0, 2) === 12) score += 8;
        if (Atomics.and(i32, 0, 6) === 10) score += 16;
        if (Atomics.or(i32, 0, 8) === 2) score += 32;
        if (Atomics.xor(i32, 0, 3) === 10) score += 64;
        if (Atomics.compareExchange(i32, 0, 9, 21) === 9) score += 128;
        if (Atomics.load(i32, 0) === 21) score += 256;
        if (Atomics.exchange(u32, 1, 11) === 0) score += 512;
        if (Atomics.load(u32, 1) === 11) score += 1024;
        if (Atomics.isLockFree(4) && !Atomics.isLockFree(3)) score += 2048;

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(4095));
}

#[test]
fn phase6_atomics_bigint_wait_and_timeout_paths_work_on_bigint64_arrays() {
    let result = run_spec_script(
        r#"
        let sab = new SharedArrayBuffer(16);
        let big = new BigInt64Array(sab);
        let score = 0;

        if (Atomics.store(big, 0, 1n) === 1n) score += 1;
        if (Atomics.add(big, 0, 2n) === 1n) score += 2;
        if (Atomics.sub(big, 0, 1n) === 3n) score += 4;
        if (Atomics.compareExchange(big, 0, 2n, 9n) === 2n) score += 8;
        if (Atomics.load(big, 0) === 9n) score += 16;
        if (Atomics.wait(big, 0, 1n, 0) === "not-equal") score += 32;
        if (Atomics.wait(big, 0, 9n, 0) === "timed-out") score += 64;

        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(127));
}

#[test]
fn phase6_wait_async_notifies_pending_waiters_and_reports_immediate_states() {
    let result = run_spec_script_with_readback(
        r#"
        let sab = new SharedArrayBuffer(8);
        let i32 = new Int32Array(sab);
        let immediate = Atomics.waitAsync(i32, 0, 1, 0);
        Atomics.store(i32, 0, 1);
        let timeout = Atomics.waitAsync(i32, 0, 1, 0);
        let pending = Atomics.waitAsync(i32, 0, 1);
        globalThis.phase6WaitAsyncScore = 0;
        globalThis.phase6WaitAsyncResolved = "";
        pending.value.then(v => { globalThis.phase6WaitAsyncResolved = v; });
        if (immediate.async === false && immediate.value === "not-equal") phase6WaitAsyncScore += 1;
        if (timeout.async === false && timeout.value === "timed-out") phase6WaitAsyncScore += 2;
        if (pending.async === true) phase6WaitAsyncScore += 4;
        if (Atomics.notify(i32, 0, 1) === 1) phase6WaitAsyncScore += 8;
        "#,
        r#"
        let score = phase6WaitAsyncScore;
        if (phase6WaitAsyncResolved === "ok") score += 16;
        score;
        "#,
    );

    assert_eq!(result, Value::from_smi(31));
}
