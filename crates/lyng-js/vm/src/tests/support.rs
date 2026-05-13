pub(super) use crate::{
    seed_registers, FeedbackInlineCacheState, FeedbackKeyedPropertyFamily, FeedbackSiteDetail,
    FeedbackVectorSnapshot, FrameFlags, FrameRecord, InstalledCode, RegisterWindow, TierStatus, Vm,
    VmDebugCommand, VmDebugHook, VmDebugPauseContext, VmDebugPauseReason, VmDebugSafepointKind,
    VmDebugStepMode, VmError,
};
pub(super) use lyng_js_bytecode::{
    ArgumentsMode, BytecodeBuilder, BytecodeEnvironmentBinding, BytecodeEnvironmentSlotFlags,
    BytecodeFunction, BytecodeFunctionId, BytecodeFunctionKind, CompiledAtom, CompiledFunctionUnit,
    CompiledScriptUnit, ConstantValue, DeoptFrameValue, DeoptValueSource, ExceptionHandler,
    ExceptionHandlerKind, FeedbackSiteKind, FeedbackSiteMetadata, Instruction, Opcode,
    SafepointKind,
};
pub(super) use lyng_js_common::{AtomId, AtomTable, SourceId, WellKnownAtom};
pub(super) use lyng_js_compiler::{compile_module, compile_script, CompiledModuleUnit};
pub(super) use lyng_js_env::{
    EnvironmentBindingLayout, EnvironmentLayout, EnvironmentLayoutKind, EnvironmentSlotFlags,
    ExecutableId, ExecutionContextKind, JobQueueKind, ModuleStatus, PromiseReactionHandler,
    PromiseReactionKind, PromiseReactionRecord, RealmBootstrapState, Runtime, RuntimeJobPayload,
};
pub(super) use lyng_js_gc::{
    AllocationLifetime, BigIntSign, PrimitiveMutator, PrimitiveRoots, PrimitiveStringView,
    StringEncoding,
};
pub(super) use lyng_js_host::{
    HostCall, HostJobKind, HostJobPhase, ImportMetaProperties, ImportMetaProperty, ImportMetaValue,
    LoadedModuleSource, ModuleKey, ModuleSourceRequest, NoopHostHooks, TestHost,
};
pub(super) use lyng_js_objects::{
    FunctionEntryIdentity, InternalMethodResult, NamedPropertyCachePath, NamedPropertyStorageMode,
    NativeCallRequest, NativeConstructRequest, NativeFunctionRegistry, ObjectAllocation,
    ObjectRuntime,
};
pub(super) use lyng_js_ops::object::{ordinary_create_data_property, ordinary_get};
pub(super) use lyng_js_parser::{parse_module, parse_script};
pub(super) use lyng_js_sema::{analyze_module, analyze_script};
pub(super) use lyng_js_types::{
    function_builtin, internal_function_call_builtin, symbol_builtin, CodeRef, EmbeddingFunctionId,
    EnvironmentRef, FeedbackSlotId, NativeFunctionId, ObjectRef, PropertyKey, RealmRef, Value,
};
pub(super) use std::fmt::Write;
pub(super) use std::mem::size_of;
pub(super) use std::sync::Arc;

#[derive(Default)]
pub(super) struct RejectingRegistry;

impl NativeFunctionRegistry for RejectingRegistry {
    fn call(
        &mut self,
        _runtime: &mut ObjectRuntime,
        _heap: &mut PrimitiveMutator<'_>,
        _request: NativeCallRequest<'_>,
    ) -> InternalMethodResult<Value> {
        panic!("unexpected native call during vm promise test");
    }

    fn construct(
        &mut self,
        _runtime: &mut ObjectRuntime,
        _heap: &mut PrimitiveMutator<'_>,
        _request: NativeConstructRequest<'_>,
    ) -> InternalMethodResult<ObjectRef> {
        panic!("unexpected native construct during vm promise test");
    }
}

const TEST_EMBEDDING_EVAL_SCRIPT_RAW: u32 = 1;

fn test_embedding_eval_script_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(TEST_EMBEDDING_EVAL_SCRIPT_RAW)
        .expect("embedding function ids should stay non-zero")
}

fn test_embedding_property_key(agent: &mut lyng_js_env::Agent, text: &str) -> PropertyKey {
    PropertyKey::from_atom(agent.atoms_mut().intern_collectible(text))
}

#[derive(Clone, Default)]
pub(super) struct TestEmbeddingProvider;

impl crate::RealmExtensionProvider for TestEmbeddingProvider {
    fn embedding_function_metadata(
        &self,
        entry: EmbeddingFunctionId,
    ) -> Option<crate::EmbeddingFunctionMetadata> {
        (entry == test_embedding_eval_script_entry()).then_some(
            crate::EmbeddingFunctionMetadata::new("evalScript", 1, false, false),
        )
    }

    fn install_realm_extensions(
        &self,
        installation: &mut crate::RealmExtensionInstallation<'_>,
    ) -> Result<(), VmError> {
        let key = test_embedding_property_key(installation.agent(), "embeddingEvalScript");
        let _ = installation.define_function_property(
            installation.global_object(),
            key,
            test_embedding_eval_script_entry(),
            true,
            false,
            true,
        )?;
        Ok(())
    }

    fn call_embedding_function(
        &self,
        context: &mut crate::EmbeddingFunctionContext<'_>,
        entry: EmbeddingFunctionId,
        invocation: crate::EmbeddingInvocation<'_>,
    ) -> Result<Value, VmError> {
        if entry != test_embedding_eval_script_entry() {
            return Err(VmError::MissingEmbeddingFunction(entry));
        }
        let source = invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined());
        let source_text = context.value_to_string_text(source)?;
        context.evaluate_script_in_realm(context.function_realm(), &source_text)
    }
}

pub(super) fn compile_test_unit(source_id: u32, source: &str) -> CompiledScriptUnit {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, SourceId::new(source_id), source);
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    compile_script(&parsed, &sema, &mut atoms).unwrap()
}

pub(super) fn compile_test_module(source_id: u32, source: &str) -> CompiledModuleUnit {
    let mut atoms = AtomTable::new();
    let parsed = parse_module(&mut atoms, SourceId::new(source_id), source);
    assert!(!parsed.diagnostics.has_errors());
    let sema = analyze_module(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());
    compile_module(&parsed, &sema, &mut atoms).unwrap()
}

pub(super) fn unit_atom(unit: &CompiledScriptUnit, text: &str) -> AtomId {
    unit.atoms()
        .iter()
        .find_map(|(atom, candidate)| (candidate.as_str() == Some(text)).then_some(*atom))
        .unwrap_or_else(|| panic!("compiled unit should intern atom {text:?}"))
}

pub(super) fn unit_runtime_atom(
    agent: &mut lyng_js_env::Agent,
    unit: &CompiledScriptUnit,
    atom: AtomId,
) -> AtomId {
    if let Some(text) = unit.atom_text(atom) {
        return agent.atoms_mut().intern_collectible(text);
    }
    let units = unit
        .atom_utf16(atom)
        .expect("compiled unit atom should resolve to UTF-8 or UTF-16 data");
    agent.atoms_mut().intern_collectible_utf16(units)
}

pub(super) fn install_global_value(
    agent: &mut lyng_js_env::Agent,
    realm: &lyng_js_env::RealmRecord,
    name: AtomId,
    value: Value,
) {
    assert!(ordinary_create_data_property(
        agent,
        realm.global_object(),
        PropertyKey::from_atom(name),
        value,
        AllocationLifetime::Default,
    )
    .unwrap());
}

pub(super) fn decode_string(view: &PrimitiveStringView<'_>) -> String {
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

pub(super) fn global_value(
    agent: &mut lyng_js_env::Agent,
    realm: &lyng_js_env::RealmRecord,
    name: &str,
) -> Value {
    let atom = agent.atoms_mut().intern_collectible(name);
    ordinary_get(agent, realm.global_object(), PropertyKey::from_atom(atom)).unwrap()
}

pub(super) fn iterator_result_fields(
    agent: &mut lyng_js_env::Agent,
    result: Value,
) -> (Value, Value) {
    let object = result
        .as_object_ref()
        .expect("iterator result should be an object");
    let value = ordinary_get(
        agent,
        object,
        PropertyKey::from_atom(lyng_js_common::WellKnownAtom::value.id()),
    )
    .unwrap();
    let done_atom = agent.atoms_mut().intern_collectible("done");
    let done = ordinary_get(agent, object, PropertyKey::from_atom(done_atom)).unwrap();
    (value, done)
}
