use super::Agent;
use crate::{
    AsyncDisposalOperationId, AsyncDisposalOperationRecord, AsyncDisposalResumeId,
    AsyncDisposalResumeRecord, DisposableResourceRecord, DisposalCapabilityId,
    DisposalCapabilityKind, DisposalCapabilityRecord, DisposalCapabilityState, PromiseCapabilityId,
};
use lyng_js_types::{ObjectRef, Value};

impl Agent {
    pub fn alloc_disposal_capability(
        &mut self,
        kind: DisposalCapabilityKind,
    ) -> DisposalCapabilityId {
        self.disposal_tables.alloc_capability(kind)
    }

    pub fn disposal_capability(
        &self,
        id: DisposalCapabilityId,
    ) -> Option<&DisposalCapabilityRecord> {
        self.disposal_tables.capability(id)
    }

    pub fn disposal_capability_id_for_object(
        &self,
        object: ObjectRef,
    ) -> Option<DisposalCapabilityId> {
        self.disposal_tables.capability_id_for_object(object)
    }

    pub fn bind_disposal_capability_object(
        &mut self,
        object: ObjectRef,
        capability: DisposalCapabilityId,
    ) -> bool {
        self.disposal_tables
            .bind_capability_object(object, capability)
    }

    pub fn set_disposal_capability_state(
        &mut self,
        id: DisposalCapabilityId,
        state: DisposalCapabilityState,
    ) -> bool {
        self.disposal_tables.set_capability_state(id, state)
    }

    pub fn push_disposal_resource(
        &mut self,
        id: DisposalCapabilityId,
        resource: DisposableResourceRecord,
    ) -> bool {
        self.disposal_tables.push_resource(id, resource)
    }

    pub fn pop_disposal_resource(
        &mut self,
        id: DisposalCapabilityId,
    ) -> Option<DisposableResourceRecord> {
        self.disposal_tables.pop_resource(id)
    }

    pub fn take_disposal_resources(
        &mut self,
        id: DisposalCapabilityId,
    ) -> Option<Vec<DisposableResourceRecord>> {
        self.disposal_tables.take_resources(id)
    }

    pub fn replace_disposal_resources(
        &mut self,
        id: DisposalCapabilityId,
        resources: Vec<DisposableResourceRecord>,
    ) -> bool {
        self.disposal_tables.replace_resources(id, resources)
    }

    pub fn alloc_async_disposal_operation(
        &mut self,
        capability: DisposalCapabilityId,
        promise_capability: PromiseCapabilityId,
    ) -> AsyncDisposalOperationId {
        self.disposal_tables
            .alloc_async_operation(capability, promise_capability)
    }

    pub fn async_disposal_operation(
        &self,
        id: AsyncDisposalOperationId,
    ) -> Option<AsyncDisposalOperationRecord> {
        self.disposal_tables.async_operation(id)
    }

    pub fn set_async_disposal_operation_pending_error(
        &mut self,
        id: AsyncDisposalOperationId,
        pending_error: Option<Value>,
    ) -> bool {
        self.disposal_tables
            .set_async_operation_pending_error(id, pending_error)
    }

    pub fn set_async_disposal_operation_has_disposal_error(
        &mut self,
        id: AsyncDisposalOperationId,
        has_disposal_error: bool,
    ) -> bool {
        self.disposal_tables
            .set_async_operation_has_disposal_error(id, has_disposal_error)
    }

    pub fn set_async_disposal_operation_waiting(
        &mut self,
        id: AsyncDisposalOperationId,
        waiting: bool,
    ) -> bool {
        self.disposal_tables
            .set_async_operation_waiting(id, waiting)
    }

    pub fn set_async_disposal_operation_completed(
        &mut self,
        id: AsyncDisposalOperationId,
        completed: bool,
    ) -> bool {
        self.disposal_tables
            .set_async_operation_completed(id, completed)
    }

    pub fn alloc_async_disposal_resume(
        &mut self,
        object: ObjectRef,
        record: AsyncDisposalResumeRecord,
    ) -> AsyncDisposalResumeId {
        self.disposal_tables.alloc_async_resume(object, record)
    }

    pub fn async_disposal_resume(&self, object: ObjectRef) -> Option<AsyncDisposalResumeRecord> {
        self.disposal_tables.async_resume_for_object(object)
    }
}
