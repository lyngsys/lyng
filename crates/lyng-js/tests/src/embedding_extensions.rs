use std::sync::Arc;

use lyng_js_common::{AtomTable, SourceId};
use lyng_js_compiler::compile_script;
use lyng_js_env::{Agent, Runtime};
use lyng_js_gc::AllocationLifetime;
use lyng_js_gc::PrimitiveStringView;
use lyng_js_host::NoopHostHooks;
use lyng_js_ops::errors;
use lyng_js_parser::parse_script;
use lyng_js_sema::analyze_script;
use lyng_js_types::{EmbeddingFunctionId, PropertyKey, Value};
use lyng_js_vm::{
    EmbeddingFunctionContext, EmbeddingFunctionMetadata, EmbeddingInvocation,
    RealmExtensionInstallation, RealmExtensionProvider, SharedRealmExtensionProvider, Vm, VmError,
};

const EMBEDDING_EVAL_SCRIPT_RAW: u32 = 1;
const EMBEDDING_CREATE_REALM_RAW: u32 = 2;

fn embedding_eval_script_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(EMBEDDING_EVAL_SCRIPT_RAW)
        .expect("embedding function ids should stay non-zero")
}

fn embedding_create_realm_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(EMBEDDING_CREATE_REALM_RAW)
        .expect("embedding function ids should stay non-zero")
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

fn embedding_property_key(agent: &mut Agent, text: &str) -> PropertyKey {
    PropertyKey::from_atom(agent.atoms_mut().intern_collectible(text))
}

#[derive(Clone, Default)]
struct DemoExtensionProvider;

impl RealmExtensionProvider for DemoExtensionProvider {
    fn embedding_function_metadata(
        &self,
        entry: EmbeddingFunctionId,
    ) -> Option<EmbeddingFunctionMetadata> {
        if entry == embedding_eval_script_entry() {
            return Some(EmbeddingFunctionMetadata::new(
                "evalScript",
                1,
                false,
                false,
            ));
        }
        if entry == embedding_create_realm_entry() {
            return Some(EmbeddingFunctionMetadata::new(
                "createRealm",
                0,
                false,
                false,
            ));
        }
        None
    }

    fn install_realm_extensions(
        &self,
        installation: &mut RealmExtensionInstallation<'_>,
    ) -> Result<(), VmError> {
        let realm = installation.realm();
        let object_prototype = installation
            .agent()
            .realm(realm)
            .and_then(|realm| realm.intrinsics().object_prototype())
            .ok_or(VmError::MissingRootShape(realm))?;
        let embedding = installation.allocate_ordinary_object(Some(object_prototype))?;

        let embedding_key = embedding_property_key(installation.agent(), "embedding");
        let marker_key = embedding_property_key(installation.agent(), "embeddingMarker");
        let eval_script_key = embedding_property_key(installation.agent(), "evalScript");
        let create_realm_key = embedding_property_key(installation.agent(), "createRealm");
        let marker_value = Value::from_string_ref(installation.agent().alloc_runtime_string(
            "installed",
            None,
            AllocationLifetime::Default,
        ));

        installation.define_data_property(
            installation.global_object(),
            embedding_key,
            Value::from_object_ref(embedding),
            true,
            false,
            true,
        )?;
        installation.define_data_property(
            installation.global_object(),
            marker_key,
            marker_value,
            true,
            false,
            true,
        )?;
        let _ = installation.define_function_property(
            embedding,
            eval_script_key,
            embedding_eval_script_entry(),
            true,
            false,
            true,
        )?;
        let _ = installation.define_function_property(
            embedding,
            create_realm_key,
            embedding_create_realm_entry(),
            true,
            false,
            true,
        )?;
        Ok(())
    }

    fn call_embedding_function(
        &self,
        context: &mut EmbeddingFunctionContext<'_>,
        entry: EmbeddingFunctionId,
        invocation: EmbeddingInvocation<'_>,
    ) -> Result<Value, VmError> {
        if entry == embedding_eval_script_entry() {
            let source = invocation
                .arguments()
                .first()
                .copied()
                .unwrap_or(Value::undefined());
            let source_text = context.value_to_string_text(source)?;
            return context.evaluate_script_in_realm(context.function_realm(), &source_text);
        }
        if entry == embedding_create_realm_entry() {
            return Ok(Value::from_object_ref(
                context.create_embedding_realm()?.global_object(),
            ));
        }
        Err(VmError::Abrupt(errors::throw_type_error(context.agent())))
    }
}

fn compile_and_run(source: &str, provider: Option<&SharedRealmExtensionProvider>) -> Value {
    let mut atoms = AtomTable::new();
    let unit = compile_unit(source, &mut atoms);
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    vm.evaluate_script_with_host_referrer_and_extensions(
        agent,
        realm,
        &unit,
        None,
        &NoopHostHooks,
        provider,
    )
    .expect("script should execute")
}

fn compile_and_run_string(source: &str, provider: Option<&SharedRealmExtensionProvider>) -> String {
    let mut atoms = AtomTable::new();
    let unit = compile_unit(source, &mut atoms);
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent.default_realm().expect("default realm should exist");
    let mut vm = Vm::new();
    let value = vm
        .evaluate_script_with_host_referrer_and_extensions(
            agent,
            realm,
            &unit,
            None,
            &NoopHostHooks,
            provider,
        )
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

#[test]
fn spec_only_realms_do_not_install_embedding_extensions() {
    let result = compile_and_run_string("typeof embedding;", None);

    assert_eq!(result, "undefined");
}

#[test]
fn embedding_extensions_install_globals_and_dispatch_native_functions() {
    let provider: SharedRealmExtensionProvider = Arc::new(DemoExtensionProvider);
    let result = compile_and_run(
        r#"
        let value = embedding.evalScript("globalThis.embeddingValue = 41; embeddingValue;");
        (typeof embedding === "object" ? 1 : 0)
            + (embeddingMarker === "installed" ? 2 : 0)
            + (value === 41 ? 4 : 0)
            + (embeddingValue === 41 ? 8 : 0);
        "#,
        Some(&provider),
    );

    assert_eq!(result, Value::from_smi(15));
}

#[test]
fn embedding_extensions_propagate_to_child_realms() {
    let provider: SharedRealmExtensionProvider = Arc::new(DemoExtensionProvider);
    let result = compile_and_run(
        r#"
        let childGlobal = embedding.createRealm();
        (childGlobal !== globalThis ? 1 : 0)
            + (typeof childGlobal.embedding === "object" ? 2 : 0)
            + (childGlobal.embeddingMarker === "installed" ? 4 : 0);
        "#,
        Some(&provider),
    );

    assert_eq!(result, Value::from_smi(7));
}

#[test]
fn indirect_eval_uses_eval_functions_realm() {
    let provider: SharedRealmExtensionProvider = Arc::new(DemoExtensionProvider);
    let result = compile_and_run_string(
        r#"
        let childGlobal = embedding.createRealm();
        let otherEval = childGlobal.eval;
        otherEval("var x = 23;");
        typeof x + ":" + String(childGlobal.x);
        "#,
        Some(&provider),
    );

    assert_eq!(result, "undefined:23");
}
