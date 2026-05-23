use super::Agent;
use crate::{
    PromiseCapabilityId, PromiseCapabilityRecord, PromiseCombinatorElementId,
    PromiseCombinatorElementRecord, PromiseCombinatorId, PromiseCombinatorKind,
    PromiseCombinatorRecord, PromiseFinallyFunctionId, PromiseFinallyFunctionRecord, PromiseId,
    PromiseReactionId, PromiseReactionKind, PromiseReactionRecord, PromiseRecord,
    PromiseResolvingFunctionId, PromiseResolvingFunctionRecord,
};
use lyng_js_types::{ObjectRef, RealmRef, Value};

impl Agent {
    pub fn alloc_promise(&mut self, object: ObjectRef, realm: RealmRef) -> PromiseId {
        self.promise_tables.alloc_promise(object, realm)
    }

    pub fn promise_id_for_object(&self, object: ObjectRef) -> Option<PromiseId> {
        self.promise_tables.promise_id_for_object(object)
    }

    pub fn promise_record(&self, object: ObjectRef) -> Option<&PromiseRecord> {
        self.promise_tables.promise_for_object(object)
    }

    pub fn set_promise_fulfilled(&mut self, object: ObjectRef, value: Value) -> bool {
        self.promise_tables.set_promise_fulfilled(object, value)
    }

    pub fn set_promise_rejected(&mut self, object: ObjectRef, reason: Value) -> bool {
        self.promise_tables.set_promise_rejected(object, reason)
    }

    pub fn set_promise_handled(&mut self, object: ObjectRef, handled: bool) -> bool {
        self.promise_tables.set_promise_handled(object, handled)
    }

    pub fn push_promise_reaction(
        &mut self,
        object: ObjectRef,
        kind: PromiseReactionKind,
        reaction: PromiseReactionId,
    ) -> bool {
        self.promise_tables
            .push_promise_reaction(object, kind, reaction)
    }

    pub fn take_promise_reactions(
        &mut self,
        object: ObjectRef,
        kind: PromiseReactionKind,
    ) -> Option<Vec<PromiseReactionId>> {
        self.promise_tables.take_promise_reactions(object, kind)
    }

    pub fn alloc_promise_reaction(&mut self, reaction: PromiseReactionRecord) -> PromiseReactionId {
        self.promise_tables.alloc_reaction(reaction)
    }

    pub fn promise_reaction(&self, id: PromiseReactionId) -> Option<PromiseReactionRecord> {
        self.promise_tables.reaction(id)
    }

    pub fn alloc_promise_capability(&mut self) -> PromiseCapabilityId {
        self.promise_tables.alloc_capability()
    }

    pub fn promise_capability(&self, id: PromiseCapabilityId) -> Option<PromiseCapabilityRecord> {
        self.promise_tables.capability(id)
    }

    pub fn set_promise_capability_promise(
        &mut self,
        id: PromiseCapabilityId,
        promise: ObjectRef,
    ) -> bool {
        self.promise_tables.set_capability_promise(id, promise)
    }

    pub fn set_promise_capability_resolve(
        &mut self,
        id: PromiseCapabilityId,
        resolve: ObjectRef,
    ) -> bool {
        self.promise_tables.set_capability_resolve(id, resolve)
    }

    pub fn set_promise_capability_resolve_value(
        &mut self,
        id: PromiseCapabilityId,
        resolve: Value,
    ) -> bool {
        self.promise_tables
            .set_capability_resolve_value(id, resolve)
    }

    pub fn set_promise_capability_reject(
        &mut self,
        id: PromiseCapabilityId,
        reject: ObjectRef,
    ) -> bool {
        self.promise_tables.set_capability_reject(id, reject)
    }

    pub fn set_promise_capability_reject_value(
        &mut self,
        id: PromiseCapabilityId,
        reject: Value,
    ) -> bool {
        self.promise_tables.set_capability_reject_value(id, reject)
    }

    pub fn set_promise_capability_already_resolved(
        &mut self,
        id: PromiseCapabilityId,
        already_resolved: bool,
    ) -> bool {
        self.promise_tables
            .set_capability_already_resolved(id, already_resolved)
    }

    pub fn alloc_promise_resolving_function(
        &mut self,
        object: ObjectRef,
        record: PromiseResolvingFunctionRecord,
    ) -> PromiseResolvingFunctionId {
        self.promise_tables.alloc_resolving_function(object, record)
    }

    pub fn promise_resolving_function(
        &self,
        object: ObjectRef,
    ) -> Option<PromiseResolvingFunctionRecord> {
        self.promise_tables.resolving_function_for_object(object)
    }

    pub fn alloc_promise_finally_function(
        &mut self,
        object: ObjectRef,
        record: PromiseFinallyFunctionRecord,
    ) -> PromiseFinallyFunctionId {
        self.promise_tables.alloc_finally_function(object, record)
    }

    pub fn promise_finally_function(
        &self,
        object: ObjectRef,
    ) -> Option<PromiseFinallyFunctionRecord> {
        self.promise_tables.finally_function_for_object(object)
    }

    pub fn alloc_promise_combinator(
        &mut self,
        kind: PromiseCombinatorKind,
        capability: PromiseCapabilityId,
    ) -> PromiseCombinatorId {
        self.promise_tables.alloc_combinator(kind, capability)
    }

    pub fn promise_combinator(&self, id: PromiseCombinatorId) -> Option<&PromiseCombinatorRecord> {
        self.promise_tables.combinator(id)
    }

    pub fn push_promise_combinator_placeholder(
        &mut self,
        id: PromiseCombinatorId,
    ) -> Option<usize> {
        self.promise_tables.combinator_push_placeholder(id)
    }

    pub fn set_promise_combinator_value(
        &mut self,
        id: PromiseCombinatorId,
        index: usize,
        value: Value,
    ) -> bool {
        self.promise_tables.combinator_set_value(id, index, value)
    }

    pub fn promise_combinator_already_called(
        &self,
        id: PromiseCombinatorId,
        index: usize,
    ) -> Option<bool> {
        self.promise_tables.combinator_already_called(id, index)
    }

    pub fn set_promise_combinator_already_called(
        &mut self,
        id: PromiseCombinatorId,
        index: usize,
        already_called: bool,
    ) -> bool {
        self.promise_tables
            .combinator_set_already_called(id, index, already_called)
    }

    pub fn decrement_promise_combinator_remaining(
        &mut self,
        id: PromiseCombinatorId,
    ) -> Option<usize> {
        self.promise_tables.combinator_decrement_remaining(id)
    }

    pub fn alloc_promise_combinator_element(
        &mut self,
        object: ObjectRef,
        record: PromiseCombinatorElementRecord,
    ) -> PromiseCombinatorElementId {
        self.promise_tables.alloc_combinator_element(object, record)
    }

    pub fn promise_combinator_element(
        &self,
        object: ObjectRef,
    ) -> Option<PromiseCombinatorElementRecord> {
        self.promise_tables.combinator_element_for_object(object)
    }

    pub fn set_promise_combinator_element_already_called(
        &mut self,
        object: ObjectRef,
        already_called: bool,
    ) -> bool {
        self.promise_tables
            .set_combinator_element_already_called(object, already_called)
    }
}
