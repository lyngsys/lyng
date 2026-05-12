use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use lyng_js_env::{
    Agent, ExecutableId, PromiseReactionHandler, PromiseReactionKind, PromiseReactionRecord,
    RuntimeJobPayload,
};
use lyng_js_host::HostJobKind;
use lyng_js_ops::{errors, read};
use lyng_js_types::{
    abstract_module_source_builtin, EmbeddingFunctionId, ObjectRef, PropertyKey, Value,
};
use lyng_js_vm::{
    EmbeddingFunctionContext, EmbeddingFunctionMetadata, EmbeddingInvocation,
    RealmExtensionInstallation, RealmExtensionProvider, VmError,
};

use crate::print_sink::{Test262PrintSink, Test262StdoutPrintSink};

const TEST262_EVAL_SCRIPT_RAW: u32 = 1;
const TEST262_CREATE_REALM_RAW: u32 = 2;
const TEST262_DETACH_ARRAY_BUFFER_RAW: u32 = 3;
const TEST262_GC_RAW: u32 = 4;
const TEST262_PRINT_RAW: u32 = 5;
const TEST262_SAME_VALUE_RAW: u32 = 6;
const TEST262_AGENT_GET_REPORT_RAW: u32 = 7;
const TEST262_AGENT_SLEEP_RAW: u32 = 8;
const TEST262_AGENT_MONOTONIC_NOW_RAW: u32 = 9;
const TEST262_SET_TIMEOUT_RAW: u32 = 10;
const TEST262_IS_HTMLDDA_RAW: u32 = 11;
const TEST262_BUILD_STRING_RAW: u32 = 12;

/// Realm extension that installs the Test262 host harness surface.
///
/// Installation defines the `$262` host object on the global, plus the
/// `print` and `setTimeout` globals that Test262's harness scripts and async
/// helpers rely on. The `print` destination is supplied by the caller as a
/// [`Test262PrintSink`].
#[derive(Clone)]
pub struct Test262RealmExtension {
    print_sink: Arc<dyn Test262PrintSink>,
}

impl Test262RealmExtension {
    /// Creates a harness extension that forwards `print` to the given sink.
    #[must_use]
    pub fn new(print_sink: Arc<dyn Test262PrintSink>) -> Self {
        Self { print_sink }
    }

    /// Creates a harness extension that writes `print` output to process
    /// stdout. Convenience for embedders that want the canonical Test262
    /// shell-style behaviour.
    #[must_use]
    pub fn to_stdout() -> Self {
        Self::new(Arc::new(Test262StdoutPrintSink))
    }

    fn call_print(
        &self,
        context: &mut EmbeddingFunctionContext<'_>,
        invocation: EmbeddingInvocation<'_>,
    ) -> Result<Value, VmError> {
        let message = context.value_to_string_text(first_invocation_argument(&invocation))?;
        self.print_sink.record(message);
        Ok(Value::undefined())
    }
}

impl RealmExtensionProvider for Test262RealmExtension {
    fn embedding_function_metadata(
        &self,
        entry: EmbeddingFunctionId,
    ) -> Option<EmbeddingFunctionMetadata> {
        if entry == test262_eval_script_entry() {
            return Some(EmbeddingFunctionMetadata::new(
                "evalScript",
                1,
                false,
                false,
            ));
        }
        if entry == test262_create_realm_entry() {
            return Some(EmbeddingFunctionMetadata::new(
                "createRealm",
                0,
                false,
                false,
            ));
        }
        if entry == test262_detach_array_buffer_entry() {
            return Some(EmbeddingFunctionMetadata::new(
                "detachArrayBuffer",
                1,
                false,
                false,
            ));
        }
        if entry == test262_gc_entry() {
            return Some(EmbeddingFunctionMetadata::new("gc", 0, false, false));
        }
        if entry == test262_print_entry() {
            return Some(EmbeddingFunctionMetadata::new("print", 1, false, false));
        }
        if entry == test262_same_value_entry() {
            return Some(EmbeddingFunctionMetadata::new("sameValue", 2, false, false));
        }
        if entry == test262_agent_get_report_entry() {
            return Some(EmbeddingFunctionMetadata::new("getReport", 0, false, false));
        }
        if entry == test262_agent_sleep_entry() {
            return Some(EmbeddingFunctionMetadata::new("sleep", 1, false, false));
        }
        if entry == test262_agent_monotonic_now_entry() {
            return Some(EmbeddingFunctionMetadata::new(
                "monotonicNow",
                0,
                false,
                false,
            ));
        }
        if entry == test262_set_timeout_entry() {
            return Some(EmbeddingFunctionMetadata::new(
                "setTimeout",
                2,
                false,
                false,
            ));
        }
        if entry == test262_is_html_dda_entry() {
            return Some(EmbeddingFunctionMetadata::new("IsHTMLDDA", 0, false, false));
        }
        if entry == test262_build_string_entry() {
            return Some(EmbeddingFunctionMetadata::new(
                "buildString",
                1,
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
        let global = installation.global_object();
        let object_prototype = installation
            .agent()
            .realm(realm)
            .and_then(|realm| realm.intrinsics().object_prototype())
            .ok_or(VmError::MissingRootShape(realm))?;
        let harness = installation.allocate_ordinary_object(Some(object_prototype))?;

        define_test262_data_property(
            installation,
            global,
            "$262",
            Value::from_object_ref(harness),
        )?;
        let agent = installation.allocate_ordinary_object(Some(object_prototype))?;
        define_test262_data_property(
            installation,
            harness,
            "agent",
            Value::from_object_ref(agent),
        )?;
        define_test262_data_property(
            installation,
            harness,
            "global",
            Value::from_object_ref(global),
        )?;
        let _ = define_test262_function_property(
            installation,
            harness,
            "evalScript",
            test262_eval_script_entry(),
        )?;
        let _ = define_test262_function_property(
            installation,
            harness,
            "createRealm",
            test262_create_realm_entry(),
        )?;
        let _ = define_test262_function_property(
            installation,
            harness,
            "detachArrayBuffer",
            test262_detach_array_buffer_entry(),
        )?;
        let _ = define_test262_function_property(installation, harness, "gc", test262_gc_entry())?;
        let _ =
            define_test262_function_property(installation, global, "print", test262_print_entry())?;
        let _ = define_test262_function_property(
            installation,
            global,
            "setTimeout",
            test262_set_timeout_entry(),
        )?;
        let _ = define_test262_function_property(
            installation,
            harness,
            "sameValue",
            test262_same_value_entry(),
        )?;
        let is_html_dda = define_test262_function_property(
            installation,
            harness,
            "IsHTMLDDA",
            test262_is_html_dda_entry(),
        )?;
        installation.mark_is_html_dda_object(is_html_dda)?;
        let _ = define_test262_function_property(
            installation,
            harness,
            "buildString",
            test262_build_string_entry(),
        )?;
        let _ = define_test262_function_property(
            installation,
            agent,
            "getReport",
            test262_agent_get_report_entry(),
        )?;
        let _ = define_test262_function_property(
            installation,
            agent,
            "sleep",
            test262_agent_sleep_entry(),
        )?;
        let _ = define_test262_function_property(
            installation,
            agent,
            "monotonicNow",
            test262_agent_monotonic_now_entry(),
        )?;
        let abstract_module_source =
            installation.builtin_constant(abstract_module_source_builtin())?;
        define_test262_data_property(
            installation,
            harness,
            "AbstractModuleSource",
            abstract_module_source,
        )?;
        Ok(())
    }

    fn call_embedding_function(
        &self,
        context: &mut EmbeddingFunctionContext<'_>,
        entry: EmbeddingFunctionId,
        invocation: EmbeddingInvocation<'_>,
    ) -> Result<Value, VmError> {
        if entry == test262_eval_script_entry() {
            return call_eval_script(context, invocation);
        }
        if entry == test262_create_realm_entry() {
            return call_create_realm(context);
        }
        if entry == test262_detach_array_buffer_entry() {
            return call_detach_array_buffer(context, invocation);
        }
        if entry == test262_gc_entry() {
            // The primitive collector is not yet safe to run from arbitrary
            // VM frames. Keep the Test262 host hook observationally inert
            // until allocation slow paths can trace active VM state.
            return Ok(Value::undefined());
        }
        if entry == test262_print_entry() {
            return self.call_print(context, invocation);
        }
        if entry == test262_same_value_entry() {
            return call_same_value(context, invocation);
        }
        if entry == test262_agent_get_report_entry() {
            return Ok(Value::null());
        }
        if entry == test262_agent_sleep_entry() {
            return Ok(Value::undefined());
        }
        if entry == test262_agent_monotonic_now_entry() {
            return Ok(Value::from_f64(monotonic_now_milliseconds()));
        }
        if entry == test262_set_timeout_entry() {
            return call_set_timeout(context, invocation);
        }
        if entry == test262_is_html_dda_entry() {
            return Ok(Value::null());
        }
        if entry == test262_build_string_entry() {
            return test262_build_string(context, invocation);
        }
        Err(VmError::MissingEmbeddingFunction(entry))
    }
}

const fn test262_eval_script_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(TEST262_EVAL_SCRIPT_RAW)
        .expect("test262 embedding function ids should stay non-zero")
}

const fn test262_create_realm_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(TEST262_CREATE_REALM_RAW)
        .expect("test262 embedding function ids should stay non-zero")
}

const fn test262_detach_array_buffer_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(TEST262_DETACH_ARRAY_BUFFER_RAW)
        .expect("test262 embedding function ids should stay non-zero")
}

const fn test262_gc_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(TEST262_GC_RAW)
        .expect("test262 embedding function ids should stay non-zero")
}

const fn test262_print_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(TEST262_PRINT_RAW)
        .expect("test262 embedding function ids should stay non-zero")
}

const fn test262_same_value_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(TEST262_SAME_VALUE_RAW)
        .expect("test262 embedding function ids should stay non-zero")
}

const fn test262_agent_get_report_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(TEST262_AGENT_GET_REPORT_RAW)
        .expect("test262 embedding function ids should stay non-zero")
}

const fn test262_agent_sleep_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(TEST262_AGENT_SLEEP_RAW)
        .expect("test262 embedding function ids should stay non-zero")
}

const fn test262_agent_monotonic_now_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(TEST262_AGENT_MONOTONIC_NOW_RAW)
        .expect("test262 embedding function ids should stay non-zero")
}

const fn test262_set_timeout_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(TEST262_SET_TIMEOUT_RAW)
        .expect("test262 embedding function ids should stay non-zero")
}

const fn test262_is_html_dda_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(TEST262_IS_HTMLDDA_RAW)
        .expect("test262 embedding function ids should stay non-zero")
}

const fn test262_build_string_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(TEST262_BUILD_STRING_RAW)
        .expect("test262 embedding function ids should stay non-zero")
}

fn test262_property_key(agent: &mut Agent, text: &str) -> PropertyKey {
    PropertyKey::from_atom(agent.atoms_mut().intern_collectible(text))
}

fn read_test262_object(agent: &mut Agent, global_object: ObjectRef) -> Result<ObjectRef, VmError> {
    let key = test262_property_key(agent, "$262");
    agent
        .objects()
        .get_own_property(agent.heap().view(), global_object, key)
        .map_err(|_| VmError::Abrupt(errors::throw_type_error(agent)))?
        .and_then(lyng_js_types::PropertyDescriptor::value)
        .and_then(Value::as_object_ref)
        .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))
}

fn define_test262_data_property(
    installation: &mut RealmExtensionInstallation<'_>,
    object: ObjectRef,
    name: &str,
    value: Value,
) -> Result<(), VmError> {
    let key = test262_property_key(installation.agent(), name);
    installation.define_data_property(object, key, value, true, false, true)
}

fn define_test262_function_property(
    installation: &mut RealmExtensionInstallation<'_>,
    object: ObjectRef,
    name: &str,
    entry: EmbeddingFunctionId,
) -> Result<ObjectRef, VmError> {
    let key = test262_property_key(installation.agent(), name);
    installation.define_function_property(object, key, entry, true, false, true)
}

fn first_invocation_argument(invocation: &EmbeddingInvocation<'_>) -> Value {
    invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined())
}

fn call_eval_script(
    context: &mut EmbeddingFunctionContext<'_>,
    invocation: EmbeddingInvocation<'_>,
) -> Result<Value, VmError> {
    let source_text = context.value_to_string_text(first_invocation_argument(&invocation))?;
    context.evaluate_script_in_realm(context.function_realm(), &source_text)
}

fn call_create_realm(context: &mut EmbeddingFunctionContext<'_>) -> Result<Value, VmError> {
    let artifacts = context.create_embedding_realm()?;
    read_test262_object(context.agent(), artifacts.global_object()).map(Value::from_object_ref)
}

fn call_detach_array_buffer(
    context: &mut EmbeddingFunctionContext<'_>,
    invocation: EmbeddingInvocation<'_>,
) -> Result<Value, VmError> {
    let Some(object) = first_invocation_argument(&invocation).as_object_ref() else {
        return Err(VmError::Abrupt(errors::throw_type_error(context.agent())));
    };
    let Some(array_buffer) = context.agent().objects().array_buffer(object) else {
        return Err(VmError::Abrupt(errors::throw_type_error(context.agent())));
    };
    let _ = context
        .agent()
        .detach_backing_store(array_buffer.backing_store());
    Ok(Value::undefined())
}

fn call_same_value(
    context: &mut EmbeddingFunctionContext<'_>,
    invocation: EmbeddingInvocation<'_>,
) -> Result<Value, VmError> {
    let actual = first_invocation_argument(&invocation);
    let expected = invocation
        .arguments()
        .get(1)
        .copied()
        .unwrap_or(Value::undefined());
    let same = {
        let agent = context.agent();
        read::same_value(agent.heap().view(), actual, expected).map_err(VmError::Abrupt)?
    };
    if same {
        Ok(Value::undefined())
    } else {
        Err(VmError::Abrupt(errors::throw_type_error(context.agent())))
    }
}

fn monotonic_now_milliseconds() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64() * 1000.0)
        .unwrap_or(0.0)
}

fn call_set_timeout(
    context: &mut EmbeddingFunctionContext<'_>,
    invocation: EmbeddingInvocation<'_>,
) -> Result<Value, VmError> {
    let callback = first_invocation_argument(&invocation)
        .as_object_ref()
        .filter(|object| context.agent().objects().is_callable(*object))
        .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(context.agent())))?;
    let realm = context.function_realm();
    let reaction = context.agent().alloc_promise_reaction(
        PromiseReactionRecord::new(
            PromiseReactionKind::Fulfill,
            PromiseReactionHandler::Callable(callback),
            None,
        )
        .with_script_or_module_referrer(None),
    );
    let _ = context.agent().enqueue_job_with_payload(
        HostJobKind::Harness,
        ExecutableId::Builtin,
        RuntimeJobPayload::PromiseReaction {
            reaction,
            argument: Value::undefined(),
        },
        Some(realm),
        Some("Test262SetTimeout".into()),
    );
    Ok(Value::undefined())
}

fn test262_build_string(
    context: &mut EmbeddingFunctionContext<'_>,
    invocation: EmbeddingInvocation<'_>,
) -> Result<Value, VmError> {
    let args = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let Some(args) = args.as_object_ref() else {
        return Ok(Value::null());
    };

    let lone_code_points = get_test262_property(context.agent(), args, "loneCodePoints")?;
    let ranges = get_test262_property(context.agent(), args, "ranges")?;
    let mut units = Vec::new();
    if !append_code_point_array(context.agent(), lone_code_points, &mut units) {
        return Ok(Value::null());
    }
    if !append_code_point_ranges(context.agent(), ranges, &mut units) {
        return Ok(Value::null());
    }

    Ok(context.alloc_code_unit_string(&units))
}

fn get_test262_property(
    agent: &mut Agent,
    object: ObjectRef,
    name: &str,
) -> Result<Value, VmError> {
    let key = PropertyKey::from_atom(agent.atoms_mut().intern_collectible(name));
    lyng_js_ops::object::ordinary_get(agent, object, key).map_err(VmError::Abrupt)
}

fn append_code_point_array(agent: &Agent, value: Value, units: &mut Vec<u16>) -> bool {
    let Some(array) = value.as_object_ref() else {
        return false;
    };
    let Some(length) = agent.objects().element_logical_len(array) else {
        return false;
    };
    let Some(elements) = agent.objects().elements(agent.heap().view(), array) else {
        return false;
    };

    for index in 0..length {
        let Some(value) = elements.get(index as usize).copied() else {
            return false;
        };
        let Some(code_point) = code_point_from_value(value) else {
            return false;
        };
        append_code_point_units(units, code_point);
    }
    true
}

fn append_code_point_ranges(agent: &Agent, value: Value, units: &mut Vec<u16>) -> bool {
    let Some(ranges) = value.as_object_ref() else {
        return false;
    };
    let Some(length) = agent.objects().element_logical_len(ranges) else {
        return false;
    };
    let Some(elements) = agent.objects().elements(agent.heap().view(), ranges) else {
        return false;
    };

    for index in 0..length {
        let Some(range) = elements.get(index as usize).copied() else {
            return false;
        };
        let Some((start, end)) = code_point_range_from_value(agent, range) else {
            return false;
        };
        for code_point in start..=end {
            append_code_point_units(units, code_point);
        }
    }
    true
}

fn code_point_range_from_value(agent: &Agent, value: Value) -> Option<(u32, u32)> {
    let range = value.as_object_ref()?;
    let elements = agent.objects().elements(agent.heap().view(), range)?;
    let start = code_point_from_value(*elements.first()?)?;
    let end = code_point_from_value(*elements.get(1)?)?;
    (start <= end).then_some((start, end))
}

fn code_point_from_value(value: Value) -> Option<u32> {
    let code_point = value.as_smi().and_then(|value| u32::try_from(value).ok())?;
    (code_point <= 0x0010_FFFF).then_some(code_point)
}

fn append_code_point_units(units: &mut Vec<u16>, code_point: u32) {
    if code_point <= 0xFFFF {
        units.push(u16::try_from(code_point).expect("BMP code point should fit u16"));
        return;
    }

    let adjusted = code_point - 0x1_0000;
    let high = u16::try_from(adjusted >> 10).expect("surrogate high bits should fit u16");
    let low = u16::try_from(adjusted & 0x03FF).expect("surrogate low bits should fit u16");
    units.push(0xD800 | high);
    units.push(0xDC00 | low);
}
