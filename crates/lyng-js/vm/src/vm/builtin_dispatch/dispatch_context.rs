use super::*;

mod internal;
mod public;
mod support;

pub(super) struct VmBuiltinDispatch<'a, 'agent, 'registry> {
    pub(super) vm: &'a mut Vm,
    pub(super) agent: &'agent mut Agent,
    pub(super) host: &'a dyn HostHooks,
    pub(super) registry: &'registry mut dyn NativeFunctionRegistry,
    pub(super) caller_frame: FrameRecord,
    pub(super) callee_object: ObjectRef,
}
