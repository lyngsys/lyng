use super::{
    AbruptCompletion, Agent, AllocationLifetime, BuiltinFunctionId, BuiltinInvocation,
    CallerContext, DynamicFunctionKind, GeneratorResumeKind, HostErrorKind, HostHooks,
    InternalBuiltinDispatchContext, NativeFunctionRegistry, ObjectRef, PropertyDescriptor,
    PropertyKey, PublicBuiltinDispatchContext, RealmRef, TemporalCivilTime,
    TemporalCivilToInstantRequest, TemporalCurrentInstantRequest, TemporalDefaultTimeZone,
    TemporalDefaultTimeZoneRequest, TemporalInstant, TemporalInstantToCivilRequest,
    TemporalInstantWithOffset, Value, Vm, VmError, VmProxyBridge, VmResult, WellKnownAtom,
    alloc_code_unit_string, errors, eval_builtin, object, object_to_string_builtin, read,
    to_f64_number,
};
use crate::frame::FrameView;

mod internal;
mod public;
mod support;

/// Builtin-dispatch bridge. Holds the caller's [`CallerContext`] (realm/
/// `lexical_env/code/pc`) by value rather than a `&FrameRecord`, so it is valid
/// on the synthetic `call_to_completion` path (no live frame) as well as the
/// real-frame dispatch path. The class-helper / super-op builtins additionally
/// need a live-frame `FrameView`; they build it via [`Self::caller_frame_view`].
pub(super) struct VmBuiltinDispatch<'a, 'agent, 'registry> {
    pub(super) vm: &'a mut Vm,
    pub(super) agent: &'agent mut Agent,
    pub(super) host: &'a dyn HostHooks,
    pub(super) registry: &'registry mut dyn NativeFunctionRegistry,
    pub(super) caller: CallerContext,
    pub(super) callee_object: ObjectRef,
}

impl VmBuiltinDispatch<'_, '_, '_> {
    /// `FrameView` for the class-helper / super-op internal builtins, which are
    /// compiler-emitted and run with the caller as the live current frame
    /// (REAL-frame-only — never reached via the synthetic `call_to_completion`
    /// path). `cfr`/`regs_len` come from the live current frame; `pc`/`code` are
    /// the caller's (equal to the live frame's), with `pc` the LIVE caller pc the
    /// super-op feedback sites record at (the overlay `saved_pc` would be stale).
    fn caller_frame_view(&self) -> FrameView {
        let cfr = self
            .vm
            .current_cfr_opt()
            .expect("class-helper builtin dispatch runs with the caller as the live frame");
        FrameView::new(
            cfr,
            self.caller.pc,
            self.vm.frame_window_len(cfr),
            self.caller.code,
        )
    }
}
