use lyng_js_builtins::BootstrapMode;
use lyng_js_common::{AtomTable, SourceId};
use lyng_js_compiler::compile_script;
use lyng_js_env::Runtime;
use lyng_js_gc::PrimitiveStringView;
use lyng_js_host::HostHooks;
use lyng_js_parser::parse_script;
use lyng_js_sema::analyze_script;
use lyng_js_types::Value;
use lyng_js_vm::Vm;

mod duration;
mod instant;
mod plain_date;
mod plain_date_time;
mod plain_month_day;
mod plain_shared;
mod plain_time;
mod plain_year_month;
mod surface;
mod zoned_date_time;

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

#[allow(
    clippy::needless_pass_by_value,
    reason = "Temporal tests pass zero-sized host hooks directly for concise fixtures"
)]
fn compile_and_run_with_host(source: &str, host: impl HostHooks + 'static) -> Value {
    let mut atoms = AtomTable::new();
    let unit = compile_unit(source, &mut atoms);
    let mut runtime = Runtime::new(lyng_js_host::NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let _ = vm
        .bootstrap_realm(agent, realm.id(), BootstrapMode::SpecOnly)
        .expect("requested bootstrap should succeed");
    vm.evaluate_script_with_host(agent, realm, &unit, &host)
        .expect("script should execute")
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "Temporal tests pass zero-sized host hooks directly for concise fixtures"
)]
fn compile_and_run_string_with_host(source: &str, host: impl HostHooks + 'static) -> String {
    let mut atoms = AtomTable::new();
    let unit = compile_unit(source, &mut atoms);
    let mut runtime = Runtime::new(lyng_js_host::NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let _ = vm
        .bootstrap_realm(agent, realm.id(), BootstrapMode::SpecOnly)
        .expect("requested bootstrap should succeed");
    let value = vm
        .evaluate_script_with_host(agent, realm, &unit, &host)
        .expect("script should execute");
    let string = value
        .as_string_ref()
        .expect("script should return a string value");
    decode_string(
        agent
            .heap()
            .view()
            .string_view(string)
            .expect("string should exist in the heap"),
    )
}

fn decode_string(view: PrimitiveStringView<'_>) -> String {
    if let Some(bytes) = view.latin1_bytes() {
        return bytes.iter().map(|byte| char::from(*byte)).collect();
    }
    let bytes = view
        .utf16_bytes()
        .expect("string view must be Latin1 or UTF-16");
    let mut units = Vec::with_capacity(view.code_unit_len() as usize);
    for chunk in bytes.chunks_exact(2) {
        units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    String::from_utf16_lossy(&units)
}
