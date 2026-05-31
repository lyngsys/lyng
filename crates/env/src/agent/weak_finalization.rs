use super::{Agent, AgentCollectionSnapshot};
use crate::{ExecutableId, RuntimeJobPayload};
use lyng_gc::{PrimitiveCollectionReport, PrimitiveTracer, TraceHeapEdges, WeakHeapRef};
use lyng_host::HostJobKind;
use lyng_types::{internal_finalization_registry_cleanup_job_builtin, ObjectRef, Value};

struct AgentCollectionRoots<'a, T: TraceHeapEdges + ?Sized> {
    snapshot: AgentCollectionSnapshot,
    additional_roots: &'a T,
}

impl<T: TraceHeapEdges + ?Sized> TraceHeapEdges for AgentCollectionRoots<'_, T> {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.snapshot.trace_heap_edges(tracer);
        self.additional_roots.trace_heap_edges(tracer);
    }
}

impl Agent {
    pub fn keep_weak_target_alive(&mut self, target: WeakHeapRef) {
        if !self.kept_objects.contains(&target) {
            self.kept_objects.push(target);
        }
    }

    pub fn clear_kept_objects(&mut self) {
        self.kept_objects.clear();
    }

    pub fn force_collect(&mut self) -> PrimitiveCollectionReport {
        self.force_collect_with_additional_roots(&())
    }

    pub fn force_collect_with_additional_roots<T: TraceHeapEdges + ?Sized>(
        &mut self,
        additional_roots: &T,
    ) -> PrimitiveCollectionReport {
        let roots = AgentCollectionRoots {
            // `from_agent` takes `&self` and produces an owned snapshot, so no borrow
            // of `self` outlives this line — clearing the way for the disjoint
            // shared(`objects`)/mut(`heap`) borrows below.
            snapshot: AgentCollectionSnapshot::from_agent(self),
            additional_roots,
        };
        // Pass `&self.objects` as the per-object metadata tracer so dictionary-mode
        // property values (stored in ObjectRuntime's metadata side-table, outside the
        // GC heap) are marked during the stop-the-world walk. Disjoint field borrows:
        // `heap` mutably, `objects`/`roots` shared.
        let report =
            self.heap
                .force_collect_tracing_with_metadata(&self.roots, &roots, &self.objects);
        // Post-mark sweep: prune dead prototype-transition entries and drop
        // invalidated watchpoint sets. Both operate purely on ObjectRuntime
        // side tables, so they live here at the Agent layer (ObjectRuntime is
        // not accessible from inside PrimitiveHeap::sweep_weak_state).
        //
        // Note: mark bits are cleared by the end of `force_collect_tracing`
        // (inside `complete_background_sweep`). We therefore use
        // `heap.view().object(obj).is_some()` as the liveness test: after the
        // concurrent sweep, dead objects' slots are freed (return `None`) while
        // live objects' slots remain occupied (return `Some`).
        {
            let heap = &self.heap;
            self.objects
                .prune_dead_prototype_transitions(|obj| heap.view().object(obj).is_some());
            // Per-constructor `.prototype` watchpoint sets (construct IC fast
            // path) are keyed by constructor object; drop sets whose
            // constructor has been collected.
            self.objects
                .prune_dead_construct_prototype_watchpoints(|obj| {
                    heap.view().object(obj).is_some()
                });
        }
        self.objects.sweep_invalidated_watchpoint_sets();
        self.enqueue_pending_finalization_cleanup_jobs();
        report
    }

    pub fn finalization_cleanup_callback(&self, registry: ObjectRef) -> Option<ObjectRef> {
        self.objects
            .ordinary_payload_value(self.heap.view(), registry)
            .and_then(Value::as_object_ref)
    }

    pub fn take_finalization_cleanup_holdings(&mut self, registry: ObjectRef) -> Vec<Value> {
        self.heap
            .mutator()
            .take_finalization_cleanup_holdings(registry)
    }

    pub fn set_finalization_cleanup_active(&mut self, registry: ObjectRef, active: bool) -> bool {
        self.heap
            .mutator()
            .set_finalization_cleanup_active(registry, active)
    }

    pub fn finalization_cleanup_pending(&mut self, registry: ObjectRef) -> bool {
        self.heap
            .mutator()
            .finalization_cleanup_pending(registry)
            .unwrap_or(false)
    }

    pub fn weak_ref_target(&mut self, object: ObjectRef) -> Option<Option<WeakHeapRef>> {
        self.heap.view().weak_ref_target(object)
    }

    pub fn enqueue_finalization_cleanup_job(&mut self, registry: ObjectRef) -> bool {
        if !self.set_finalization_cleanup_active(registry, true) {
            return false;
        }
        let realm = self
            .finalization_cleanup_callback(registry)
            .and_then(|callback| self.objects.function_data(callback))
            .and_then(lyng_objects::FunctionObjectData::realm);
        let _ = self.enqueue_job_with_payload(
            HostJobKind::Native(internal_finalization_registry_cleanup_job_builtin()),
            ExecutableId::Builtin,
            RuntimeJobPayload::FinalizationCleanup { registry },
            realm,
            Some("FinalizationRegistryCleanup".into()),
        );
        true
    }

    fn enqueue_pending_finalization_cleanup_jobs(&mut self) {
        let pending = self.heap.mutator().pending_finalization_registries();
        for registry in pending {
            let _ = self.enqueue_finalization_cleanup_job(registry);
        }
    }
}
