use super::{
    ids::{
        PromiseCapabilityId, PromiseCombinatorElementId, PromiseCombinatorId,
        PromiseFinallyFunctionId, PromiseId, PromiseReactionId, PromiseResolvingFunctionId,
    },
    RealmRef,
};
use lyng_js_gc::{PrimitiveTracer, TraceHeapEdges};
use lyng_js_types::{ObjectRef, SuspendedExecutionRef, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromiseState {
    Pending,
    Fulfilled,
    Rejected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromiseReactionKind {
    Fulfill,
    Reject,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromiseReactionHandler {
    Identity,
    Thrower,
    Callable(ObjectRef),
    PassThrough(Value),
    ThrowWith(Value),
    Finally {
        on_finally: ObjectRef,
        constructor: ObjectRef,
        reject: bool,
    },
    AsyncResume {
        suspended: SuspendedExecutionRef,
        reject: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PromiseReactionRecord {
    kind: PromiseReactionKind,
    handler: PromiseReactionHandler,
    capability: Option<PromiseCapabilityId>,
}

impl PromiseReactionRecord {
    #[inline]
    pub const fn new(
        kind: PromiseReactionKind,
        handler: PromiseReactionHandler,
        capability: Option<PromiseCapabilityId>,
    ) -> Self {
        Self {
            kind,
            handler,
            capability,
        }
    }

    #[inline]
    pub const fn kind(self) -> PromiseReactionKind {
        self.kind
    }

    #[inline]
    pub const fn handler(self) -> PromiseReactionHandler {
        self.handler
    }

    #[inline]
    pub const fn capability(self) -> Option<PromiseCapabilityId> {
        self.capability
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PromiseCapabilityRecord {
    promise: Option<ObjectRef>,
    resolve: Option<Value>,
    reject: Option<Value>,
    already_resolved: bool,
}

impl PromiseCapabilityRecord {
    #[inline]
    pub const fn new() -> Self {
        Self {
            promise: None,
            resolve: None,
            reject: None,
            already_resolved: false,
        }
    }

    #[inline]
    pub const fn promise(self) -> Option<ObjectRef> {
        self.promise
    }

    #[inline]
    pub const fn resolve_value(self) -> Option<Value> {
        self.resolve
    }

    #[inline]
    pub const fn resolve(self) -> Option<ObjectRef> {
        match self.resolve {
            Some(value) => value.as_object_ref(),
            None => None,
        }
    }

    #[inline]
    pub const fn reject_value(self) -> Option<Value> {
        self.reject
    }

    #[inline]
    pub const fn reject(self) -> Option<ObjectRef> {
        match self.reject {
            Some(value) => value.as_object_ref(),
            None => None,
        }
    }

    #[inline]
    pub const fn already_resolved(self) -> bool {
        self.already_resolved
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromiseResolvingFunctionKind {
    Resolve,
    Reject,
    CapabilityExecutor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PromiseResolvingFunctionRecord {
    kind: PromiseResolvingFunctionKind,
    capability: PromiseCapabilityId,
}

impl PromiseResolvingFunctionRecord {
    #[inline]
    pub const fn new(kind: PromiseResolvingFunctionKind, capability: PromiseCapabilityId) -> Self {
        Self { kind, capability }
    }

    #[inline]
    pub const fn kind(self) -> PromiseResolvingFunctionKind {
        self.kind
    }

    #[inline]
    pub const fn capability(self) -> PromiseCapabilityId {
        self.capability
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromiseFinallyFunctionKind {
    Then,
    Catch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PromiseFinallyFunctionRecord {
    kind: PromiseFinallyFunctionKind,
    on_finally: ObjectRef,
    constructor: ObjectRef,
}

impl PromiseFinallyFunctionRecord {
    #[inline]
    pub const fn new(
        kind: PromiseFinallyFunctionKind,
        on_finally: ObjectRef,
        constructor: ObjectRef,
    ) -> Self {
        Self {
            kind,
            on_finally,
            constructor,
        }
    }

    #[inline]
    pub const fn kind(self) -> PromiseFinallyFunctionKind {
        self.kind
    }

    #[inline]
    pub const fn on_finally(self) -> ObjectRef {
        self.on_finally
    }

    #[inline]
    pub const fn constructor(self) -> ObjectRef {
        self.constructor
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromiseCombinatorKind {
    All,
    AllSettled,
    Any,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromiseCombinatorElementKind {
    AllResolve,
    AllSettledResolve,
    AllSettledReject,
    AnyReject,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromiseCombinatorRecord {
    kind: PromiseCombinatorKind,
    capability: PromiseCapabilityId,
    values: Vec<Value>,
    already_called: Vec<bool>,
    remaining_elements: usize,
}

impl PromiseCombinatorRecord {
    #[inline]
    pub fn new(kind: PromiseCombinatorKind, capability: PromiseCapabilityId) -> Self {
        Self {
            kind,
            capability,
            values: Vec::new(),
            already_called: Vec::new(),
            remaining_elements: 1,
        }
    }

    #[inline]
    pub const fn kind(&self) -> PromiseCombinatorKind {
        self.kind
    }

    #[inline]
    pub const fn capability(&self) -> PromiseCapabilityId {
        self.capability
    }

    #[inline]
    pub fn values(&self) -> &[Value] {
        &self.values
    }

    #[inline]
    pub fn already_called(&self) -> &[bool] {
        &self.already_called
    }

    #[inline]
    pub const fn remaining_elements(&self) -> usize {
        self.remaining_elements
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PromiseCombinatorElementRecord {
    kind: PromiseCombinatorElementKind,
    combinator: PromiseCombinatorId,
    index: usize,
    already_called: bool,
}

impl PromiseCombinatorElementRecord {
    #[inline]
    pub const fn new(
        kind: PromiseCombinatorElementKind,
        combinator: PromiseCombinatorId,
        index: usize,
    ) -> Self {
        Self {
            kind,
            combinator,
            index,
            already_called: false,
        }
    }

    #[inline]
    pub const fn kind(self) -> PromiseCombinatorElementKind {
        self.kind
    }

    #[inline]
    pub const fn combinator(self) -> PromiseCombinatorId {
        self.combinator
    }

    #[inline]
    pub const fn index(self) -> usize {
        self.index
    }

    #[inline]
    pub const fn already_called(self) -> bool {
        self.already_called
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromiseRecord {
    object: ObjectRef,
    realm: RealmRef,
    state: PromiseState,
    result: Value,
    fulfill_reactions: Vec<PromiseReactionId>,
    reject_reactions: Vec<PromiseReactionId>,
    handled: bool,
}

impl PromiseRecord {
    #[inline]
    pub fn new(object: ObjectRef, realm: RealmRef) -> Self {
        Self {
            object,
            realm,
            state: PromiseState::Pending,
            result: Value::undefined(),
            fulfill_reactions: Vec::new(),
            reject_reactions: Vec::new(),
            handled: false,
        }
    }

    #[inline]
    pub const fn object(&self) -> ObjectRef {
        self.object
    }

    #[inline]
    pub const fn realm(&self) -> RealmRef {
        self.realm
    }

    #[inline]
    pub const fn state(&self) -> PromiseState {
        self.state
    }

    #[inline]
    pub const fn result(&self) -> Value {
        self.result
    }

    #[inline]
    pub fn fulfill_reactions(&self) -> &[PromiseReactionId] {
        &self.fulfill_reactions
    }

    #[inline]
    pub fn reject_reactions(&self) -> &[PromiseReactionId] {
        &self.reject_reactions
    }

    #[inline]
    pub const fn handled(&self) -> bool {
        self.handled
    }
}

#[derive(Clone, Default)]
pub(crate) struct AgentPromiseTables {
    promises: Vec<Option<PromiseRecord>>,
    promise_by_object: Vec<Option<PromiseId>>,
    reactions: Vec<Option<PromiseReactionRecord>>,
    capabilities: Vec<Option<PromiseCapabilityRecord>>,
    resolving_functions: Vec<Option<PromiseResolvingFunctionRecord>>,
    resolving_function_by_object: Vec<Option<PromiseResolvingFunctionId>>,
    finally_functions: Vec<Option<PromiseFinallyFunctionRecord>>,
    finally_function_by_object: Vec<Option<PromiseFinallyFunctionId>>,
    combinators: Vec<Option<PromiseCombinatorRecord>>,
    combinator_elements: Vec<Option<PromiseCombinatorElementRecord>>,
    combinator_element_by_object: Vec<Option<PromiseCombinatorElementId>>,
}

impl TraceHeapEdges for PromiseReactionHandler {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        match self {
            Self::Identity | Self::Thrower => {}
            Self::Callable(object) => object.trace_heap_edges(tracer),
            Self::PassThrough(value) | Self::ThrowWith(value) => value.trace_heap_edges(tracer),
            Self::Finally {
                on_finally,
                constructor,
                ..
            } => {
                on_finally.trace_heap_edges(tracer);
                constructor.trace_heap_edges(tracer);
            }
            Self::AsyncResume { suspended, .. } => suspended.trace_heap_edges(tracer),
        }
    }
}

impl TraceHeapEdges for PromiseReactionRecord {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.handler.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for PromiseCapabilityRecord {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.promise.trace_heap_edges(tracer);
        self.resolve.trace_heap_edges(tracer);
        self.reject.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for PromiseResolvingFunctionRecord {
    fn trace_heap_edges(&self, _tracer: &mut PrimitiveTracer<'_>) {}
}

impl TraceHeapEdges for PromiseFinallyFunctionRecord {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.on_finally.trace_heap_edges(tracer);
        self.constructor.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for PromiseCombinatorRecord {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        for value in &self.values {
            value.trace_heap_edges(tracer);
        }
    }
}

impl TraceHeapEdges for PromiseCombinatorElementRecord {
    fn trace_heap_edges(&self, _tracer: &mut PrimitiveTracer<'_>) {}
}

impl TraceHeapEdges for PromiseRecord {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.object.trace_heap_edges(tracer);
        self.realm.trace_heap_edges(tracer);
        self.result.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for AgentPromiseTables {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        for promise in self.promises.iter().flatten() {
            promise.trace_heap_edges(tracer);
        }
        for reaction in self.reactions.iter().flatten() {
            reaction.trace_heap_edges(tracer);
        }
        for capability in self.capabilities.iter().flatten() {
            capability.trace_heap_edges(tracer);
        }
        for record in self.resolving_functions.iter().flatten() {
            record.trace_heap_edges(tracer);
        }
        for record in self.finally_functions.iter().flatten() {
            record.trace_heap_edges(tracer);
        }
        for combinator in self.combinators.iter().flatten() {
            combinator.trace_heap_edges(tracer);
        }
        for element in self.combinator_elements.iter().flatten() {
            element.trace_heap_edges(tracer);
        }
        trace_object_keys(&self.promise_by_object, tracer);
        trace_object_keys(&self.resolving_function_by_object, tracer);
        trace_object_keys(&self.finally_function_by_object, tracer);
        trace_object_keys(&self.combinator_element_by_object, tracer);
    }
}

impl AgentPromiseTables {
    pub(crate) fn promise_id_for_object(&self, object: ObjectRef) -> Option<PromiseId> {
        self.promise_by_object
            .get(object_index(object))
            .copied()
            .flatten()
    }

    pub(crate) fn promise(&self, id: PromiseId) -> Option<&PromiseRecord> {
        self.promises.get(id_index(id))?.as_ref()
    }

    pub(crate) fn promise_mut(&mut self, id: PromiseId) -> Option<&mut PromiseRecord> {
        self.promises.get_mut(id_index(id))?.as_mut()
    }

    pub(crate) fn promise_for_object(&self, object: ObjectRef) -> Option<&PromiseRecord> {
        self.promise_id_for_object(object)
            .and_then(|id| self.promise(id))
    }

    pub(crate) fn promise_for_object_mut(
        &mut self,
        object: ObjectRef,
    ) -> Option<&mut PromiseRecord> {
        let id = self.promise_id_for_object(object)?;
        self.promise_mut(id)
    }

    pub(crate) fn alloc_promise(&mut self, object: ObjectRef, realm: RealmRef) -> PromiseId {
        if let Some(existing) = self.promise_id_for_object(object) {
            return existing;
        }

        let raw_id =
            u32::try_from(self.promises.len() + 1).expect("promise id should fit into u32");
        let id = PromiseId::from_raw(raw_id).expect("promise id must stay non-zero");
        self.promises.push(Some(PromiseRecord::new(object, realm)));

        let index = object_index(object);
        if self.promise_by_object.len() <= index {
            self.promise_by_object.resize(index + 1, None);
        }
        self.promise_by_object[index] = Some(id);
        id
    }

    pub(crate) fn set_promise_fulfilled(&mut self, object: ObjectRef, value: Value) -> bool {
        let Some(record) = self.promise_for_object_mut(object) else {
            return false;
        };
        if record.state != PromiseState::Pending {
            return false;
        }
        record.state = PromiseState::Fulfilled;
        record.result = value;
        true
    }

    pub(crate) fn set_promise_rejected(&mut self, object: ObjectRef, reason: Value) -> bool {
        let Some(record) = self.promise_for_object_mut(object) else {
            return false;
        };
        if record.state != PromiseState::Pending {
            return false;
        }
        record.state = PromiseState::Rejected;
        record.result = reason;
        true
    }

    pub(crate) fn set_promise_handled(&mut self, object: ObjectRef, handled: bool) -> bool {
        let Some(record) = self.promise_for_object_mut(object) else {
            return false;
        };
        record.handled = handled;
        true
    }

    pub(crate) fn push_promise_reaction(
        &mut self,
        object: ObjectRef,
        kind: PromiseReactionKind,
        reaction: PromiseReactionId,
    ) -> bool {
        let Some(record) = self.promise_for_object_mut(object) else {
            return false;
        };
        match kind {
            PromiseReactionKind::Fulfill => record.fulfill_reactions.push(reaction),
            PromiseReactionKind::Reject => record.reject_reactions.push(reaction),
        }
        true
    }

    pub(crate) fn take_promise_reactions(
        &mut self,
        object: ObjectRef,
        kind: PromiseReactionKind,
    ) -> Option<Vec<PromiseReactionId>> {
        let record = self.promise_for_object_mut(object)?;
        Some(match kind {
            PromiseReactionKind::Fulfill => std::mem::take(&mut record.fulfill_reactions),
            PromiseReactionKind::Reject => std::mem::take(&mut record.reject_reactions),
        })
    }

    pub(crate) fn alloc_reaction(&mut self, reaction: PromiseReactionRecord) -> PromiseReactionId {
        let raw_id =
            u32::try_from(self.reactions.len() + 1).expect("reaction id should fit into u32");
        let id = PromiseReactionId::from_raw(raw_id).expect("reaction id must stay non-zero");
        self.reactions.push(Some(reaction));
        id
    }

    pub(crate) fn reaction(&self, id: PromiseReactionId) -> Option<PromiseReactionRecord> {
        self.reactions.get(id_index(id))?.as_ref().copied()
    }

    pub(crate) fn alloc_capability(&mut self) -> PromiseCapabilityId {
        let raw_id =
            u32::try_from(self.capabilities.len() + 1).expect("capability id should fit into u32");
        let id = PromiseCapabilityId::from_raw(raw_id).expect("capability id must stay non-zero");
        self.capabilities.push(Some(PromiseCapabilityRecord::new()));
        id
    }

    pub(crate) fn capability(&self, id: PromiseCapabilityId) -> Option<PromiseCapabilityRecord> {
        self.capabilities.get(id_index(id))?.as_ref().copied()
    }

    pub(crate) fn set_capability_promise(
        &mut self,
        id: PromiseCapabilityId,
        promise: ObjectRef,
    ) -> bool {
        let Some(record) = self
            .capabilities
            .get_mut(id_index(id))
            .and_then(Option::as_mut)
        else {
            return false;
        };
        record.promise = Some(promise);
        true
    }

    pub(crate) fn set_capability_resolve(
        &mut self,
        id: PromiseCapabilityId,
        resolve: ObjectRef,
    ) -> bool {
        self.set_capability_resolve_value(id, Value::from_object_ref(resolve))
    }

    pub(crate) fn set_capability_resolve_value(
        &mut self,
        id: PromiseCapabilityId,
        resolve: Value,
    ) -> bool {
        let Some(record) = self
            .capabilities
            .get_mut(id_index(id))
            .and_then(Option::as_mut)
        else {
            return false;
        };
        record.resolve = Some(resolve);
        true
    }

    pub(crate) fn set_capability_reject(
        &mut self,
        id: PromiseCapabilityId,
        reject: ObjectRef,
    ) -> bool {
        self.set_capability_reject_value(id, Value::from_object_ref(reject))
    }

    pub(crate) fn set_capability_reject_value(
        &mut self,
        id: PromiseCapabilityId,
        reject: Value,
    ) -> bool {
        let Some(record) = self
            .capabilities
            .get_mut(id_index(id))
            .and_then(Option::as_mut)
        else {
            return false;
        };
        record.reject = Some(reject);
        true
    }

    pub(crate) fn set_capability_already_resolved(
        &mut self,
        id: PromiseCapabilityId,
        already_resolved: bool,
    ) -> bool {
        let Some(record) = self
            .capabilities
            .get_mut(id_index(id))
            .and_then(Option::as_mut)
        else {
            return false;
        };
        record.already_resolved = already_resolved;
        true
    }

    pub(crate) fn alloc_resolving_function(
        &mut self,
        object: ObjectRef,
        record: PromiseResolvingFunctionRecord,
    ) -> PromiseResolvingFunctionId {
        if let Some(existing) = self.resolving_function_id_for_object(object) {
            return existing;
        }

        let raw_id = u32::try_from(self.resolving_functions.len() + 1)
            .expect("resolving function id should fit into u32");
        let id = PromiseResolvingFunctionId::from_raw(raw_id)
            .expect("resolving function id must stay non-zero");
        self.resolving_functions.push(Some(record));

        let index = object_index(object);
        if self.resolving_function_by_object.len() <= index {
            self.resolving_function_by_object.resize(index + 1, None);
        }
        self.resolving_function_by_object[index] = Some(id);
        id
    }

    pub(crate) fn resolving_function_id_for_object(
        &self,
        object: ObjectRef,
    ) -> Option<PromiseResolvingFunctionId> {
        self.resolving_function_by_object
            .get(object_index(object))
            .copied()
            .flatten()
    }

    pub(crate) fn resolving_function_for_object(
        &self,
        object: ObjectRef,
    ) -> Option<PromiseResolvingFunctionRecord> {
        let id = self.resolving_function_id_for_object(object)?;
        self.resolving_functions
            .get(id_index(id))?
            .as_ref()
            .copied()
    }

    pub(crate) fn alloc_finally_function(
        &mut self,
        object: ObjectRef,
        record: PromiseFinallyFunctionRecord,
    ) -> PromiseFinallyFunctionId {
        if let Some(existing) = self.finally_function_id_for_object(object) {
            return existing;
        }

        let raw_id = u32::try_from(self.finally_functions.len() + 1)
            .expect("finally function id should fit into u32");
        let id = PromiseFinallyFunctionId::from_raw(raw_id)
            .expect("finally function id must stay non-zero");
        self.finally_functions.push(Some(record));

        let index = object_index(object);
        if self.finally_function_by_object.len() <= index {
            self.finally_function_by_object.resize(index + 1, None);
        }
        self.finally_function_by_object[index] = Some(id);
        id
    }

    pub(crate) fn finally_function_id_for_object(
        &self,
        object: ObjectRef,
    ) -> Option<PromiseFinallyFunctionId> {
        self.finally_function_by_object
            .get(object_index(object))
            .copied()
            .flatten()
    }

    pub(crate) fn finally_function_for_object(
        &self,
        object: ObjectRef,
    ) -> Option<PromiseFinallyFunctionRecord> {
        let id = self.finally_function_id_for_object(object)?;
        self.finally_functions.get(id_index(id))?.as_ref().copied()
    }

    pub(crate) fn alloc_combinator(
        &mut self,
        kind: PromiseCombinatorKind,
        capability: PromiseCapabilityId,
    ) -> PromiseCombinatorId {
        let raw_id =
            u32::try_from(self.combinators.len() + 1).expect("combinator id should fit into u32");
        let id = PromiseCombinatorId::from_raw(raw_id).expect("combinator id must stay non-zero");
        self.combinators
            .push(Some(PromiseCombinatorRecord::new(kind, capability)));
        id
    }

    pub(crate) fn combinator(&self, id: PromiseCombinatorId) -> Option<&PromiseCombinatorRecord> {
        self.combinators.get(id_index(id))?.as_ref()
    }

    pub(crate) fn combinator_mut(
        &mut self,
        id: PromiseCombinatorId,
    ) -> Option<&mut PromiseCombinatorRecord> {
        self.combinators.get_mut(id_index(id))?.as_mut()
    }

    pub(crate) fn combinator_push_placeholder(&mut self, id: PromiseCombinatorId) -> Option<usize> {
        let record = self.combinator_mut(id)?;
        let index = record.values.len();
        record.values.push(Value::undefined());
        record.already_called.push(false);
        record.remaining_elements = record
            .remaining_elements
            .checked_add(1)
            .expect("promise combinator remaining elements should not overflow");
        Some(index)
    }

    pub(crate) fn combinator_set_value(
        &mut self,
        id: PromiseCombinatorId,
        index: usize,
        value: Value,
    ) -> bool {
        let Some(record) = self.combinator_mut(id) else {
            return false;
        };
        let Some(slot) = record.values.get_mut(index) else {
            return false;
        };
        *slot = value;
        true
    }

    pub(crate) fn combinator_already_called(
        &self,
        id: PromiseCombinatorId,
        index: usize,
    ) -> Option<bool> {
        self.combinator(id)?.already_called.get(index).copied()
    }

    pub(crate) fn combinator_set_already_called(
        &mut self,
        id: PromiseCombinatorId,
        index: usize,
        already_called: bool,
    ) -> bool {
        let Some(record) = self.combinator_mut(id) else {
            return false;
        };
        let Some(slot) = record.already_called.get_mut(index) else {
            return false;
        };
        *slot = already_called;
        true
    }

    pub(crate) fn combinator_decrement_remaining(
        &mut self,
        id: PromiseCombinatorId,
    ) -> Option<usize> {
        let record = self.combinator_mut(id)?;
        record.remaining_elements = record.remaining_elements.checked_sub(1)?;
        Some(record.remaining_elements)
    }

    pub(crate) fn alloc_combinator_element(
        &mut self,
        object: ObjectRef,
        record: PromiseCombinatorElementRecord,
    ) -> PromiseCombinatorElementId {
        if let Some(existing) = self.combinator_element_id_for_object(object) {
            return existing;
        }

        let raw_id = u32::try_from(self.combinator_elements.len() + 1)
            .expect("combinator element id should fit into u32");
        let id = PromiseCombinatorElementId::from_raw(raw_id)
            .expect("combinator element id must stay non-zero");
        self.combinator_elements.push(Some(record));

        let index = object_index(object);
        if self.combinator_element_by_object.len() <= index {
            self.combinator_element_by_object.resize(index + 1, None);
        }
        self.combinator_element_by_object[index] = Some(id);
        id
    }

    pub(crate) fn combinator_element_id_for_object(
        &self,
        object: ObjectRef,
    ) -> Option<PromiseCombinatorElementId> {
        self.combinator_element_by_object
            .get(object_index(object))
            .copied()
            .flatten()
    }

    pub(crate) fn combinator_element_for_object(
        &self,
        object: ObjectRef,
    ) -> Option<PromiseCombinatorElementRecord> {
        let id = self.combinator_element_id_for_object(object)?;
        self.combinator_elements
            .get(id_index(id))?
            .as_ref()
            .copied()
    }

    pub(crate) fn set_combinator_element_already_called(
        &mut self,
        object: ObjectRef,
        already_called: bool,
    ) -> bool {
        let Some(id) = self.combinator_element_id_for_object(object) else {
            return false;
        };
        let Some(record) = self
            .combinator_elements
            .get_mut(id_index(id))
            .and_then(Option::as_mut)
        else {
            return false;
        };
        record.already_called = already_called;
        true
    }
}

#[inline]
fn id_index<T: PromiseTableId>(id: T) -> usize {
    usize::try_from(id.raw_value() - 1).expect("promise table index should fit into usize")
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
        let object = ObjectRef::from_raw(raw).expect("promise side-table key must stay non-zero");
        object.trace_heap_edges(tracer);
    }
}

trait PromiseTableId {
    fn raw_value(self) -> u32;
}

impl PromiseTableId for PromiseId {
    #[inline]
    fn raw_value(self) -> u32 {
        self.get()
    }
}

impl PromiseTableId for PromiseReactionId {
    #[inline]
    fn raw_value(self) -> u32 {
        self.get()
    }
}

impl PromiseTableId for PromiseCapabilityId {
    #[inline]
    fn raw_value(self) -> u32 {
        self.get()
    }
}

impl PromiseTableId for PromiseResolvingFunctionId {
    #[inline]
    fn raw_value(self) -> u32 {
        self.get()
    }
}

impl PromiseTableId for PromiseFinallyFunctionId {
    #[inline]
    fn raw_value(self) -> u32 {
        self.get()
    }
}

impl PromiseTableId for PromiseCombinatorId {
    #[inline]
    fn raw_value(self) -> u32 {
        self.get()
    }
}

impl PromiseTableId for PromiseCombinatorElementId {
    #[inline]
    fn raw_value(self) -> u32 {
        self.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_types::RealmRef;

    #[test]
    fn promise_tables_allocate_records_and_round_trip_by_object() {
        let mut tables = AgentPromiseTables::default();
        let object = ObjectRef::from_raw(7).unwrap();
        let realm = RealmRef::from_raw(3).unwrap();

        let id = tables.alloc_promise(object, realm);
        let promise = tables.promise(id).expect("promise should be allocated");

        assert_eq!(tables.promise_id_for_object(object), Some(id));
        assert_eq!(promise.object(), object);
        assert_eq!(promise.realm(), realm);
        assert_eq!(promise.state(), PromiseState::Pending);
        assert_eq!(promise.result(), Value::undefined());
        assert!(promise.fulfill_reactions().is_empty());
        assert!(promise.reject_reactions().is_empty());
        assert!(!promise.handled());
    }

    #[test]
    fn promise_tables_transition_state_once_and_drain_reactions() {
        let mut tables = AgentPromiseTables::default();
        let object = ObjectRef::from_raw(11).unwrap();
        let realm = RealmRef::from_raw(2).unwrap();
        let _ = tables.alloc_promise(object, realm);
        let fulfill = tables.alloc_reaction(PromiseReactionRecord::new(
            PromiseReactionKind::Fulfill,
            PromiseReactionHandler::Identity,
            None,
        ));
        let reject = tables.alloc_reaction(PromiseReactionRecord::new(
            PromiseReactionKind::Reject,
            PromiseReactionHandler::Thrower,
            None,
        ));

        assert!(tables.push_promise_reaction(object, PromiseReactionKind::Fulfill, fulfill));
        assert!(tables.push_promise_reaction(object, PromiseReactionKind::Reject, reject));
        assert!(tables.set_promise_fulfilled(object, Value::from_smi(23)));
        assert!(!tables.set_promise_rejected(object, Value::from_smi(99)));

        let promise = tables
            .promise_for_object(object)
            .expect("promise should still exist");
        assert_eq!(promise.state(), PromiseState::Fulfilled);
        assert_eq!(promise.result(), Value::from_smi(23));
        assert_eq!(
            tables.take_promise_reactions(object, PromiseReactionKind::Fulfill),
            Some(vec![fulfill])
        );
        assert_eq!(
            tables.take_promise_reactions(object, PromiseReactionKind::Reject),
            Some(vec![reject])
        );
        assert_eq!(
            tables.take_promise_reactions(object, PromiseReactionKind::Fulfill),
            Some(Vec::new())
        );
    }

    #[test]
    fn promise_capabilities_and_resolving_functions_share_object_side_tables() {
        let mut tables = AgentPromiseTables::default();
        let capability = tables.alloc_capability();
        let promise = ObjectRef::from_raw(21).unwrap();
        let resolve = ObjectRef::from_raw(22).unwrap();
        let reject = ObjectRef::from_raw(23).unwrap();

        assert!(tables.set_capability_promise(capability, promise));
        assert!(tables.set_capability_resolve(capability, resolve));
        assert!(tables.set_capability_reject(capability, reject));
        assert!(tables.set_capability_already_resolved(capability, true));
        tables.alloc_resolving_function(
            resolve,
            PromiseResolvingFunctionRecord::new(PromiseResolvingFunctionKind::Resolve, capability),
        );
        tables.alloc_resolving_function(
            reject,
            PromiseResolvingFunctionRecord::new(PromiseResolvingFunctionKind::Reject, capability),
        );

        let capability_record = tables
            .capability(capability)
            .expect("capability should exist");
        assert_eq!(capability_record.promise(), Some(promise));
        assert_eq!(
            capability_record.resolve_value(),
            Some(Value::from_object_ref(resolve))
        );
        assert_eq!(capability_record.resolve(), Some(resolve));
        assert_eq!(
            capability_record.reject_value(),
            Some(Value::from_object_ref(reject))
        );
        assert_eq!(capability_record.reject(), Some(reject));
        assert!(capability_record.already_resolved());
        assert_eq!(
            tables.resolving_function_for_object(resolve),
            Some(PromiseResolvingFunctionRecord::new(
                PromiseResolvingFunctionKind::Resolve,
                capability,
            ))
        );
        assert_eq!(
            tables.resolving_function_for_object(reject),
            Some(PromiseResolvingFunctionRecord::new(
                PromiseResolvingFunctionKind::Reject,
                capability,
            ))
        );
    }

    #[test]
    fn promise_finally_functions_share_object_side_tables() {
        let mut tables = AgentPromiseTables::default();
        let function = ObjectRef::from_raw(31).unwrap();
        let on_finally = ObjectRef::from_raw(32).unwrap();
        let constructor = ObjectRef::from_raw(33).unwrap();

        let id = tables.alloc_finally_function(
            function,
            PromiseFinallyFunctionRecord::new(
                PromiseFinallyFunctionKind::Then,
                on_finally,
                constructor,
            ),
        );
        let record = tables
            .finally_function_for_object(function)
            .expect("finally function should round-trip");

        assert_eq!(id.get(), 1);
        assert_eq!(record.kind(), PromiseFinallyFunctionKind::Then);
        assert_eq!(record.on_finally(), on_finally);
        assert_eq!(record.constructor(), constructor);
    }

    #[test]
    fn promise_combinator_shared_state_tracks_slots_and_already_called_elements() {
        let mut tables = AgentPromiseTables::default();
        let capability = tables.alloc_capability();
        let combinator = tables.alloc_combinator(PromiseCombinatorKind::AllSettled, capability);
        let first = ObjectRef::from_raw(31).unwrap();
        let second = ObjectRef::from_raw(32).unwrap();

        let first_index = tables
            .combinator_push_placeholder(combinator)
            .expect("combinator should exist");
        let second_index = tables
            .combinator_push_placeholder(combinator)
            .expect("combinator should exist");
        tables.alloc_combinator_element(
            first,
            PromiseCombinatorElementRecord::new(
                PromiseCombinatorElementKind::AllSettledResolve,
                combinator,
                first_index,
            ),
        );
        tables.alloc_combinator_element(
            second,
            PromiseCombinatorElementRecord::new(
                PromiseCombinatorElementKind::AllSettledReject,
                combinator,
                second_index,
            ),
        );

        assert_eq!(
            tables
                .combinator(combinator)
                .map(PromiseCombinatorRecord::remaining_elements),
            Some(3)
        );
        assert!(tables.set_combinator_element_already_called(first, true));
        assert_eq!(
            tables
                .combinator_element_for_object(first)
                .map(PromiseCombinatorElementRecord::already_called),
            Some(true)
        );
        assert_eq!(
            tables.combinator_already_called(combinator, first_index),
            Some(false)
        );
        assert!(tables.combinator_set_already_called(combinator, first_index, true));
        assert_eq!(
            tables.combinator_already_called(combinator, first_index),
            Some(true)
        );
        assert!(tables.combinator_set_value(combinator, first_index, Value::from_smi(1)));
        assert!(tables.combinator_set_value(combinator, second_index, Value::from_smi(2)));
        assert_eq!(tables.combinator_decrement_remaining(combinator), Some(2));
        assert_eq!(tables.combinator_decrement_remaining(combinator), Some(1));
        assert_eq!(tables.combinator_decrement_remaining(combinator), Some(0));
        assert_eq!(
            tables
                .combinator(combinator)
                .map(PromiseCombinatorRecord::values)
                .map(<[Value]>::to_vec),
            Some(vec![Value::from_smi(1), Value::from_smi(2)])
        );
    }
}
