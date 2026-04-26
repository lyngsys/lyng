use lyng_js_bytecode::CompiledScriptUnit;
use lyng_js_common::{AtomTable, SourceId};
use lyng_js_compiler::compile_script;
use lyng_js_env::{Agent, RealmRecord, Runtime};
use lyng_js_gc::{AllocationLifetime, PrimitiveStringView};
use lyng_js_host::NoopHostHooks;
use lyng_js_objects::{
    FunctionConstructorFlags, FunctionObjectData, FunctionThisMode, NativeFunctionRegistry,
    ObjectAllocation, ObjectColdData,
};
use lyng_js_ops::object::ordinary_create_data_property;
use lyng_js_parser::parse_script;
use lyng_js_sema::analyze_script;
use lyng_js_types::{BuiltinFunctionId, ObjectRef, PropertyKey, Value};
use lyng_js_vm::Vm;

pub(super) fn compile_unit(source: &str, atoms: &mut AtomTable) -> CompiledScriptUnit {
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

pub(super) fn compile_and_run(source: &str) -> Value {
    let mut atoms = AtomTable::new();
    let unit = compile_unit(source, &mut atoms);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    vm.evaluate_script(agent, realm, &unit)
        .expect("compiled script should execute")
}

pub(super) fn compile_and_run_string(source: &str) -> String {
    let mut atoms = AtomTable::new();
    let unit = compile_unit(source, &mut atoms);

    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let result = vm
        .evaluate_script(agent, realm, &unit)
        .expect("compiled script should execute");
    let string = result
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

pub(super) fn evaluate_with_registry(
    unit: &CompiledScriptUnit,
    install_globals: impl FnOnce(&mut Agent, RealmRecord),
    registry: &mut dyn NativeFunctionRegistry,
) -> Value {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    install_globals(agent, realm);

    let mut vm = Vm::new();
    vm.evaluate_script_with_registry(agent, realm, unit, registry)
        .expect("compiled script should execute")
}

pub(super) fn install_native_global(
    agent: &mut Agent,
    realm: RealmRecord,
    name: &str,
    entry: BuiltinFunctionId,
    constructible: bool,
) -> ObjectRef {
    let root_shape = realm
        .root_shape()
        .expect("default realm should expose a root shape");
    let function_object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        let function_data = FunctionObjectData::native(realm.id(), realm.global_env(), entry)
            .with_this_mode(FunctionThisMode::Global)
            .with_constructor_flags(if constructible {
                FunctionConstructorFlags::constructible()
            } else {
                FunctionConstructorFlags::empty()
            });
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::function(root_shape)
                .with_cold_data(ObjectColdData::Function(function_data)),
            AllocationLifetime::Default,
        )
    });
    let name = agent.atoms_mut().intern_collectible(name);

    assert!(ordinary_create_data_property(
        agent,
        realm.global_object(),
        PropertyKey::from_atom(name),
        Value::from_object_ref(function_object),
        AllocationLifetime::Default,
    )
    .expect("native global should install"));
    function_object
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
