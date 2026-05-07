use super::ids::{
    AsyncDisposalOperationId, AsyncDisposalResumeId, DisposalCapabilityId, PromiseCapabilityId,
};
use lyng_js_gc::{PrimitiveTracer, TraceHeapEdges};
use lyng_js_types::{ObjectRef, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisposalCapabilityKind {
    Sync,
    Async,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisposalCapabilityState {
    Active,
    Disposed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisposalMethodKind {
    Sync,
    Async,
    AsyncFromSync,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisposableResourceKind {
    NoMethod,
    UseMethod,
    CallbackWithValue,
    CallbackWithoutValue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisposableResourceRecord {
    kind: DisposableResourceKind,
    method_kind: DisposalMethodKind,
    callable: Option<ObjectRef>,
    value: Value,
}

impl DisposableResourceRecord {
    #[inline]
    pub const fn no_method(method_kind: DisposalMethodKind) -> Self {
        Self {
            kind: DisposableResourceKind::NoMethod,
            method_kind,
            callable: None,
            value: Value::undefined(),
        }
    }

    #[inline]
    pub const fn use_method(
        value: Value,
        callable: ObjectRef,
        method_kind: DisposalMethodKind,
    ) -> Self {
        Self {
            kind: DisposableResourceKind::UseMethod,
            method_kind,
            callable: Some(callable),
            value,
        }
    }

    #[inline]
    pub const fn callback_with_value(
        value: Value,
        callable: ObjectRef,
        method_kind: DisposalMethodKind,
    ) -> Self {
        Self {
            kind: DisposableResourceKind::CallbackWithValue,
            method_kind,
            callable: Some(callable),
            value,
        }
    }

    #[inline]
    pub const fn callback_without_value(
        callable: ObjectRef,
        method_kind: DisposalMethodKind,
    ) -> Self {
        Self {
            kind: DisposableResourceKind::CallbackWithoutValue,
            method_kind,
            callable: Some(callable),
            value: Value::undefined(),
        }
    }

    #[inline]
    pub const fn kind(self) -> DisposableResourceKind {
        self.kind
    }

    #[inline]
    pub const fn method_kind(self) -> DisposalMethodKind {
        self.method_kind
    }

    #[inline]
    pub const fn callable(self) -> Option<ObjectRef> {
        self.callable
    }

    #[inline]
    pub const fn value(self) -> Value {
        self.value
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisposalCapabilityRecord {
    kind: DisposalCapabilityKind,
    state: DisposalCapabilityState,
    resources: Vec<DisposableResourceRecord>,
}

impl DisposalCapabilityRecord {
    #[inline]
    pub const fn new(kind: DisposalCapabilityKind) -> Self {
        Self {
            kind,
            state: DisposalCapabilityState::Active,
            resources: Vec::new(),
        }
    }

    #[inline]
    pub const fn kind(&self) -> DisposalCapabilityKind {
        self.kind
    }

    #[inline]
    pub const fn state(&self) -> DisposalCapabilityState {
        self.state
    }

    #[inline]
    pub const fn is_disposed(&self) -> bool {
        matches!(self.state, DisposalCapabilityState::Disposed)
    }

    #[inline]
    pub fn resources(&self) -> &[DisposableResourceRecord] {
        &self.resources
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AsyncDisposalOperationRecord {
    capability: DisposalCapabilityId,
    promise_capability: PromiseCapabilityId,
    pending_error: Option<Value>,
    has_disposal_error: bool,
    waiting: bool,
    completed: bool,
}

impl AsyncDisposalOperationRecord {
    #[inline]
    pub const fn new(
        capability: DisposalCapabilityId,
        promise_capability: PromiseCapabilityId,
    ) -> Self {
        Self {
            capability,
            promise_capability,
            pending_error: None,
            has_disposal_error: false,
            waiting: false,
            completed: false,
        }
    }

    #[inline]
    pub const fn capability(self) -> DisposalCapabilityId {
        self.capability
    }

    #[inline]
    pub const fn promise_capability(self) -> PromiseCapabilityId {
        self.promise_capability
    }

    #[inline]
    pub const fn pending_error(self) -> Option<Value> {
        self.pending_error
    }

    #[inline]
    pub const fn has_disposal_error(self) -> bool {
        self.has_disposal_error
    }

    #[inline]
    pub const fn waiting(self) -> bool {
        self.waiting
    }

    #[inline]
    pub const fn completed(self) -> bool {
        self.completed
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AsyncDisposalResumeRecord {
    operation: AsyncDisposalOperationId,
    reject: bool,
}

impl AsyncDisposalResumeRecord {
    #[inline]
    pub const fn new(operation: AsyncDisposalOperationId, reject: bool) -> Self {
        Self { operation, reject }
    }

    #[inline]
    pub const fn operation(self) -> AsyncDisposalOperationId {
        self.operation
    }

    #[inline]
    pub const fn reject(self) -> bool {
        self.reject
    }
}

#[derive(Clone, Default)]
pub struct AgentDisposalTables {
    capabilities: Vec<Option<DisposalCapabilityRecord>>,
    capability_by_object: Vec<Option<DisposalCapabilityId>>,
    async_operations: Vec<Option<AsyncDisposalOperationRecord>>,
    async_resumes: Vec<Option<AsyncDisposalResumeRecord>>,
    async_resume_by_object: Vec<Option<AsyncDisposalResumeId>>,
}

impl TraceHeapEdges for DisposableResourceRecord {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.callable.trace_heap_edges(tracer);
        self.value.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for DisposalCapabilityRecord {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        for resource in &self.resources {
            resource.trace_heap_edges(tracer);
        }
    }
}

impl TraceHeapEdges for AsyncDisposalOperationRecord {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.pending_error.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for AsyncDisposalResumeRecord {
    fn trace_heap_edges(&self, _tracer: &mut PrimitiveTracer<'_>) {}
}

impl TraceHeapEdges for AgentDisposalTables {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        for capability in self.capabilities.iter().flatten() {
            capability.trace_heap_edges(tracer);
        }
        trace_object_keys(&self.capability_by_object, tracer);
        for operation in self.async_operations.iter().flatten() {
            operation.trace_heap_edges(tracer);
        }
        for resume in self.async_resumes.iter().flatten() {
            resume.trace_heap_edges(tracer);
        }
        trace_object_keys(&self.async_resume_by_object, tracer);
    }
}

impl AgentDisposalTables {
    pub(crate) fn alloc_capability(
        &mut self,
        kind: DisposalCapabilityKind,
    ) -> DisposalCapabilityId {
        let raw_id = u32::try_from(self.capabilities.len() + 1)
            .expect("disposal capability id should fit into u32");
        let id = DisposalCapabilityId::from_raw(raw_id)
            .expect("disposal capability id must stay non-zero");
        self.capabilities
            .push(Some(DisposalCapabilityRecord::new(kind)));
        id
    }

    pub(crate) fn capability(&self, id: DisposalCapabilityId) -> Option<&DisposalCapabilityRecord> {
        self.capabilities.get(id_index(id))?.as_ref()
    }

    pub(crate) fn capability_id_for_object(
        &self,
        object: ObjectRef,
    ) -> Option<DisposalCapabilityId> {
        self.capability_by_object
            .get(object_index(object))
            .copied()
            .flatten()
    }

    pub(crate) fn bind_capability_object(
        &mut self,
        object: ObjectRef,
        capability: DisposalCapabilityId,
    ) -> bool {
        let index = object_index(object);
        if self.capability_by_object.len() <= index {
            self.capability_by_object.resize(index + 1, None);
        }
        self.capability_by_object[index] = Some(capability);
        true
    }

    pub(crate) fn capability_mut(
        &mut self,
        id: DisposalCapabilityId,
    ) -> Option<&mut DisposalCapabilityRecord> {
        self.capabilities.get_mut(id_index(id))?.as_mut()
    }

    pub(crate) fn push_resource(
        &mut self,
        id: DisposalCapabilityId,
        resource: DisposableResourceRecord,
    ) -> bool {
        let Some(record) = self.capability_mut(id) else {
            return false;
        };
        record.resources.push(resource);
        true
    }

    pub(crate) fn pop_resource(
        &mut self,
        id: DisposalCapabilityId,
    ) -> Option<DisposableResourceRecord> {
        self.capability_mut(id)?.resources.pop()
    }

    pub(crate) fn take_resources(
        &mut self,
        id: DisposalCapabilityId,
    ) -> Option<Vec<DisposableResourceRecord>> {
        Some(std::mem::take(&mut self.capability_mut(id)?.resources))
    }

    pub(crate) fn replace_resources(
        &mut self,
        id: DisposalCapabilityId,
        resources: Vec<DisposableResourceRecord>,
    ) -> bool {
        let Some(record) = self.capability_mut(id) else {
            return false;
        };
        record.resources = resources;
        true
    }

    pub(crate) fn set_capability_state(
        &mut self,
        id: DisposalCapabilityId,
        state: DisposalCapabilityState,
    ) -> bool {
        let Some(record) = self.capability_mut(id) else {
            return false;
        };
        record.state = state;
        true
    }

    pub(crate) fn alloc_async_operation(
        &mut self,
        capability: DisposalCapabilityId,
        promise_capability: PromiseCapabilityId,
    ) -> AsyncDisposalOperationId {
        let raw_id = u32::try_from(self.async_operations.len() + 1)
            .expect("async disposal operation id should fit into u32");
        let id = AsyncDisposalOperationId::from_raw(raw_id)
            .expect("async disposal operation id must stay non-zero");
        self.async_operations
            .push(Some(AsyncDisposalOperationRecord::new(
                capability,
                promise_capability,
            )));
        id
    }

    pub(crate) fn async_operation(
        &self,
        id: AsyncDisposalOperationId,
    ) -> Option<AsyncDisposalOperationRecord> {
        self.async_operations.get(id_index(id))?.as_ref().copied()
    }

    pub(crate) fn set_async_operation_pending_error(
        &mut self,
        id: AsyncDisposalOperationId,
        pending_error: Option<Value>,
    ) -> bool {
        let Some(record) = self
            .async_operations
            .get_mut(id_index(id))
            .and_then(Option::as_mut)
        else {
            return false;
        };
        record.pending_error = pending_error;
        true
    }

    pub(crate) fn set_async_operation_has_disposal_error(
        &mut self,
        id: AsyncDisposalOperationId,
        has_disposal_error: bool,
    ) -> bool {
        let Some(record) = self
            .async_operations
            .get_mut(id_index(id))
            .and_then(Option::as_mut)
        else {
            return false;
        };
        record.has_disposal_error = has_disposal_error;
        true
    }

    pub(crate) fn set_async_operation_waiting(
        &mut self,
        id: AsyncDisposalOperationId,
        waiting: bool,
    ) -> bool {
        let Some(record) = self
            .async_operations
            .get_mut(id_index(id))
            .and_then(Option::as_mut)
        else {
            return false;
        };
        record.waiting = waiting;
        true
    }

    pub(crate) fn set_async_operation_completed(
        &mut self,
        id: AsyncDisposalOperationId,
        completed: bool,
    ) -> bool {
        let Some(record) = self
            .async_operations
            .get_mut(id_index(id))
            .and_then(Option::as_mut)
        else {
            return false;
        };
        record.completed = completed;
        true
    }

    pub(crate) fn alloc_async_resume(
        &mut self,
        object: ObjectRef,
        record: AsyncDisposalResumeRecord,
    ) -> AsyncDisposalResumeId {
        if let Some(existing) = self.async_resume_id_for_object(object) {
            return existing;
        }

        let raw_id = u32::try_from(self.async_resumes.len() + 1)
            .expect("async disposal resume id should fit into u32");
        let id = AsyncDisposalResumeId::from_raw(raw_id)
            .expect("async disposal resume id must stay non-zero");
        self.async_resumes.push(Some(record));
        let index = object_index(object);
        if self.async_resume_by_object.len() <= index {
            self.async_resume_by_object.resize(index + 1, None);
        }
        self.async_resume_by_object[index] = Some(id);
        id
    }

    pub(crate) fn async_resume_id_for_object(
        &self,
        object: ObjectRef,
    ) -> Option<AsyncDisposalResumeId> {
        self.async_resume_by_object
            .get(object_index(object))
            .copied()
            .flatten()
    }

    pub(crate) fn async_resume_for_object(
        &self,
        object: ObjectRef,
    ) -> Option<AsyncDisposalResumeRecord> {
        let id = self.async_resume_id_for_object(object)?;
        self.async_resumes.get(id_index(id))?.as_ref().copied()
    }
}

#[inline]
fn id_index<T: DisposalTableId>(id: T) -> usize {
    usize::try_from(id.raw_value() - 1).expect("disposal table index should fit into usize")
}

#[inline]
fn object_index(object: ObjectRef) -> usize {
    usize::try_from(object.get() - 1).expect("object index should fit into usize")
}

fn trace_object_keys<T>(table: &[Option<T>], tracer: &mut PrimitiveTracer<'_>) {
    for (index, entry) in table.iter().enumerate() {
        if entry.is_none() {
            continue;
        }
        let raw = u32::try_from(index + 1).expect("object table index should fit into u32");
        let object = ObjectRef::from_raw(raw).expect("object side-table key must stay non-zero");
        object.trace_heap_edges(tracer);
    }
}

trait DisposalTableId {
    fn raw_value(self) -> u32;
}

impl DisposalTableId for DisposalCapabilityId {
    #[inline]
    fn raw_value(self) -> u32 {
        self.get()
    }
}

impl DisposalTableId for AsyncDisposalOperationId {
    #[inline]
    fn raw_value(self) -> u32 {
        self.get()
    }
}

impl DisposalTableId for AsyncDisposalResumeId {
    #[inline]
    fn raw_value(self) -> u32 {
        self.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PromiseCapabilityId;

    #[test]
    fn disposal_tables_allocate_and_move_resources() {
        let mut tables = AgentDisposalTables::default();
        let first = tables.alloc_capability(DisposalCapabilityKind::Sync);
        let second = tables.alloc_capability(DisposalCapabilityKind::Sync);
        let callable = ObjectRef::from_raw(9).unwrap();
        let first_object = ObjectRef::from_raw(11).unwrap();

        assert!(tables.bind_capability_object(first_object, first));

        assert!(tables.push_resource(
            first,
            DisposableResourceRecord::callback_with_value(
                Value::from_smi(3),
                callable,
                DisposalMethodKind::Sync,
            ),
        ));

        let resources = tables
            .take_resources(first)
            .expect("resources should be movable");
        assert_eq!(resources.len(), 1);
        assert!(tables.replace_resources(second, resources));
        assert_eq!(tables.capability_id_for_object(first_object), Some(first));
        assert!(tables.capability(first).unwrap().resources().is_empty());
        assert_eq!(tables.capability(second).unwrap().resources().len(), 1);
    }

    #[test]
    fn disposal_tables_track_async_resumes_by_object() {
        let mut tables = AgentDisposalTables::default();
        let capability = tables.alloc_capability(DisposalCapabilityKind::Async);
        let promise = PromiseCapabilityId::from_raw(5).unwrap();
        let operation = tables.alloc_async_operation(capability, promise);
        let resume = ObjectRef::from_raw(17).unwrap();

        let id = tables.alloc_async_resume(resume, AsyncDisposalResumeRecord::new(operation, true));
        let record = tables
            .async_resume_for_object(resume)
            .expect("resume record should round-trip");

        assert_eq!(tables.async_resume_id_for_object(resume), Some(id));
        assert_eq!(record.operation(), operation);
        assert!(record.reject());
    }
}
