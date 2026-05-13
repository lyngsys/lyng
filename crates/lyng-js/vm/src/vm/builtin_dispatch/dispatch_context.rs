use super::{
    alloc_code_unit_string, errors, eval_builtin, object, object_to_string_builtin, read,
    to_f64_number, AbruptCompletion, Agent, AllocationLifetime, BuiltinFunctionId,
    BuiltinInvocation, DynamicFunctionKind, FrameRecord, GeneratorResumeKind, HostErrorKind,
    HostHooks, InternalBuiltinDispatchContext, NativeFunctionRegistry, ObjectRef,
    PropertyDescriptor, PropertyKey, PublicBuiltinDispatchContext, RealmRef, TemporalCivilTime,
    TemporalCivilToInstantRequest, TemporalCurrentInstantRequest, TemporalDefaultTimeZone,
    TemporalDefaultTimeZoneRequest, TemporalInstant, TemporalInstantToCivilRequest,
    TemporalInstantWithOffset, Value, Vm, VmError, VmProxyBridge, VmResult, WellKnownAtom,
};

mod internal;
mod public;
mod support;

pub(super) struct VmBuiltinDispatch<'a, 'agent, 'registry> {
    pub(super) vm: &'a mut Vm,
    pub(super) agent: &'agent mut Agent,
    pub(super) host: &'a dyn HostHooks,
    pub(super) registry: &'registry mut dyn NativeFunctionRegistry,
    pub(super) caller_frame: &'a FrameRecord,
    pub(super) callee_object: ObjectRef,
}
