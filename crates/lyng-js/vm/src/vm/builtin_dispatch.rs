mod class_helpers;
mod dispatch_context;
mod dynamic_import;
mod function_allocation;
mod function_helpers;
mod object_helpers;
mod regexp_helpers;
mod template_helpers;

use dispatch_context::VmBuiltinDispatch;

use super::property_access::VmProxyBridge;
use super::values::{alloc_code_unit_string, alloc_string, to_f64_number};
use super::{
    Agent, AllocationLifetime, FrameRecord, NativeFunctionRegistry, ObjectRef, TemplateCacheKey,
    ThisState, Value, Vm, VmError, VmResult, WellKnownAtom, WellKnownSymbolId,
};
use crate::extensions::{EmbeddingFunctionContext, EmbeddingInvocation};
use crate::frame::GeneratorResumeKind;
use lyng_js_builtins::{
    builtin_metadata, dispatch_builtin, BuiltinInvocation, DynamicFunctionKind,
    InternalBuiltinDispatchContext, PublicBuiltinDispatchContext,
};
use lyng_js_common::AtomTable;
use lyng_js_env::{
    EnvironmentLayout, EnvironmentLayoutKind, PromiseResolvingFunctionKind, ThisBindingStatus,
};
use lyng_js_host::{
    HostErrorKind, HostHooks, ImportMetaValue, ModuleImportAttribute, ModuleKey,
    ModuleSourceRequest, TemporalCivilTime, TemporalCivilToInstantRequest,
    TemporalCurrentInstantRequest, TemporalDefaultTimeZone, TemporalDefaultTimeZoneRequest,
    TemporalInstant, TemporalInstantToCivilRequest, TemporalInstantWithOffset,
};
use lyng_js_objects::{
    ClassPrivateElementKind, FunctionConstructorFlags, FunctionEntryIdentity, FunctionObjectData,
    FunctionThisMode, ObjectAllocation, ObjectColdData,
};
use lyng_js_ops::object::ToPrimitiveHint;
use lyng_js_ops::{errors, object, proxy, read};
use lyng_js_parser::parse_script;
use lyng_js_types::{
    eval_builtin, internal_dynamic_import_builtin, internal_import_meta_builtin,
    internal_regexp_literal_builtin, object_to_string_builtin, promise_capability_executor_builtin,
    string_from_code_point_builtin, AbruptCompletion, BuiltinFunctionId, EmbeddingFunctionId,
    PropertyDescriptor, PropertyKey, RealmRef,
};

impl Vm {
    pub(super) fn abrupt_intrinsic_error(
        agent: &mut Agent,
        realm: RealmRef,
        kind: errors::ErrorKind,
    ) -> VmError {
        let thrown = errors::create_intrinsic_error_object(agent, realm, kind, None)
            .map(Value::from_object_ref)
            .unwrap_or(Value::undefined());
        VmError::Abrupt(AbruptCompletion::throw(thrown))
    }

    pub(super) fn builtin_entry(
        agent: &Agent,
        callee_object: ObjectRef,
    ) -> Option<BuiltinFunctionId> {
        let data = agent.objects().function_data(callee_object)?;
        let FunctionEntryIdentity::Native(entry) = data.entry()? else {
            return None;
        };
        entry.builtin_entry()
    }

    pub(super) fn embedding_entry(
        agent: &Agent,
        callee_object: ObjectRef,
    ) -> Option<EmbeddingFunctionId> {
        let data = agent.objects().function_data(callee_object)?;
        let FunctionEntryIdentity::Native(entry) = data.entry()? else {
            return None;
        };
        entry.embedding_entry()
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn call_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameRecord,
        callee_object: ObjectRef,
        this_value: Value,
        arguments: &[Value],
        new_target: Option<ObjectRef>,
    ) -> VmResult<Option<Value>> {
        if let Some(entry) = Self::embedding_entry(agent, callee_object) {
            let Some(provider) = self.active_extension_provider.clone() else {
                return Err(VmError::MissingRealmExtensionProvider);
            };
            let mut context = EmbeddingFunctionContext::new(
                self,
                agent,
                host,
                registry,
                &provider,
                &caller_frame,
                callee_object,
            );
            let invocation = EmbeddingInvocation::new(this_value, arguments, new_target);
            let value = match new_target {
                Some(new_target) => Value::from_object_ref(provider.construct_embedding_function(
                    &mut context,
                    entry,
                    invocation,
                    new_target,
                )?),
                None => provider.call_embedding_function(&mut context, entry, invocation)?,
            };
            return Ok(Some(value));
        }
        let Some(entry) = Self::builtin_entry(agent, callee_object) else {
            return Ok(None);
        };
        if entry == internal_import_meta_builtin() {
            return Self::import_meta_builtin(agent, &caller_frame).map(Some);
        }
        if entry == internal_dynamic_import_builtin() {
            return self
                .dynamic_import_builtin(agent, host, registry, &caller_frame, arguments)
                .map(Some);
        }
        if entry == internal_regexp_literal_builtin() {
            return Self::regexp_literal_builtin(agent, caller_frame, arguments).map(Some);
        }
        let mut bridge = VmBuiltinDispatch {
            vm: self,
            agent,
            host,
            registry,
            caller_frame: &caller_frame,
            callee_object,
        };
        dispatch_builtin(
            &mut bridge,
            entry,
            BuiltinInvocation::new(this_value, arguments, new_target),
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn call_frame_safe_builtin(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameRecord,
        callee_object: ObjectRef,
        entry: BuiltinFunctionId,
        this_value: Value,
        arguments: &[Value],
    ) -> VmResult<Option<Value>> {
        debug_assert!(super::feedback::call_feedback_builtin_is_frame_safe(entry));
        let mut bridge = VmBuiltinDispatch {
            vm: self,
            agent,
            host,
            registry,
            caller_frame: &caller_frame,
            callee_object,
        };
        dispatch_builtin(
            &mut bridge,
            entry,
            BuiltinInvocation::new(this_value, arguments, None),
        )
    }

    fn builtin_realm(
        agent: &Agent,
        callee_object: ObjectRef,
        caller_frame: FrameRecord,
    ) -> RealmRef {
        agent
            .objects()
            .function_data(callee_object)
            .and_then(lyng_js_objects::FunctionObjectData::realm)
            .unwrap_or_else(|| caller_frame.realm())
    }
}
