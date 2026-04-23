use lyng_js_builtins::BootstrapMode;
use lyng_js_env::Runtime;
use lyng_js_host::NoopHostHooks;
use lyng_js_vm::Vm;

const PHASE5_DEFAULT_REALM_BUDGET_BYTES: usize = 1024 * 1024;

#[test]
fn phase5_default_realm_spec_bootstrap_live_heap_stays_within_budget() {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let before = agent.heap().view().accounting();

    let mut vm = Vm::new();
    let _ = vm
        .bootstrap_realm(agent, realm.id(), BootstrapMode::SpecOnly)
        .expect("spec bootstrap should succeed");

    let after = agent.heap().view().accounting();
    eprintln!(
        "phase5 default realm heap: before_live={} after_live={} after_reserved={}",
        before.live_bytes, after.live_bytes, after.reserved_bytes
    );

    assert!(after.live_bytes >= before.live_bytes);
    assert!(
        after.live_bytes <= PHASE5_DEFAULT_REALM_BUDGET_BYTES,
        "default realm spec bootstrap exceeded {} bytes: {}",
        PHASE5_DEFAULT_REALM_BUDGET_BYTES,
        after.live_bytes
    );
}
