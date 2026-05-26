use super::{
    AgentId, AgentJobQueues, AllocationLifetime, AtomTable, BootstrapAtoms, EnvironmentLayout,
    EnvironmentMetadata, ExecutionContext, GlobalSymbolRegistry, HostAgentId, HostThreadId,
    Intrinsics, ModuleRecord, ObjectRuntime, PrimitiveHeap, PrimitiveRoots, RealmMetadata,
    RealmRef, RegExpLegacyStaticState, WellKnownSymbols,
};
use lyng_gc::{ObjectHandleStoreTarget, PrimitiveTracer, TraceHeapEdges, WeakHeapRef};
use lyng_host::ModuleKey;
use lyng_objects::{
    AdaptiveProtoLoadDispatch, InternalMethodError, InternalMethodResult, PrototypeKey,
    RegExpPayload, ShapeInvalidationObserver, ShapeTransitionKey, Watchpoint,
};
use lyng_types::{
    CodeRef, FeedbackSlotId, ObjectRef, PropertyDescriptor, PropertyKey, ShapeId, StringRef,
};
use std::{
    collections::{BTreeMap, HashMap},
    marker::PhantomData,
    rc::Rc,
};

mod accounting;
mod cluster_handles;
mod disposal;
mod environments;
mod execution_contexts;
mod jobs;
mod modules;
mod promises;
mod realms;
mod regexp_literals;
mod symbols;
mod weak_finalization;

pub use self::cluster_handles::{ClusterBackingStoreHandle, ClusterSharedMemoryHandle};
#[derive(Clone)]
struct AgentCollectionSnapshot {
    well_known_symbols: WellKnownSymbols,
    global_symbol_registry: GlobalSymbolRegistry,
    realms: Vec<RealmRef>,
    intrinsics: Vec<Intrinsics>,
    execution_contexts: Vec<ExecutionContext>,
    modules: Vec<ModuleRecord>,
    regexp_legacy_static_states: Vec<RegExpLegacyStaticState>,
    promise_tables: super::AgentPromiseTables,
    disposal_tables: super::AgentDisposalTables,
    job_queues: AgentJobQueues,
    kept_objects: Vec<WeakHeapRef>,
    latin1_single_code_unit_strings: [Option<StringRef>; 256],
    recent_short_latin1_strings: [Option<RecentShortLatin1String>; 256],
    recent_two_code_unit_string: Option<RecentTwoCodeUnitString>,
}

impl AgentCollectionSnapshot {
    fn from_agent(agent: &Agent) -> Self {
        Self {
            well_known_symbols: agent.well_known_symbols,
            global_symbol_registry: agent.global_symbol_registry.clone(),
            realms: agent.realms.clone(),
            intrinsics: agent
                .realm_metadata
                .iter()
                .filter_map(|metadata| metadata.as_ref().map(|metadata| metadata.intrinsics))
                .collect(),
            execution_contexts: agent.execution_contexts.clone(),
            modules: agent.modules.values().cloned().collect(),
            regexp_legacy_static_states: agent
                .realm_metadata
                .iter()
                .filter_map(|metadata| {
                    metadata
                        .as_ref()
                        .map(|metadata| metadata.regexp_legacy_static_state.clone())
                })
                .collect(),
            promise_tables: agent.promise_tables.clone(),
            disposal_tables: agent.disposal_tables.clone(),
            job_queues: agent.job_queues.clone(),
            kept_objects: agent.kept_objects.clone(),
            latin1_single_code_unit_strings: agent.latin1_single_code_unit_strings,
            recent_short_latin1_strings: agent.recent_short_latin1_strings,
            recent_two_code_unit_string: agent.recent_two_code_unit_string,
        }
    }
}

impl TraceHeapEdges for AgentCollectionSnapshot {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.well_known_symbols.trace_heap_edges(tracer);
        self.global_symbol_registry.trace_heap_edges(tracer);
        for realm in &self.realms {
            realm.trace_heap_edges(tracer);
        }
        for intrinsics in &self.intrinsics {
            intrinsics.trace_heap_edges(tracer);
        }
        for context in &self.execution_contexts {
            context.trace_heap_edges(tracer);
        }
        self.promise_tables.trace_heap_edges(tracer);
        self.disposal_tables.trace_heap_edges(tracer);
        self.job_queues.trace_heap_edges(tracer);
        for target in &self.kept_objects {
            target.trace_heap_edges(tracer);
        }
        for string in self.latin1_single_code_unit_strings {
            string.trace_heap_edges(tracer);
        }
        for cached in self.recent_short_latin1_strings.into_iter().flatten() {
            cached.string.trace_heap_edges(tracer);
        }
        if let Some(cached) = self.recent_two_code_unit_string {
            cached.string.trace_heap_edges(tracer);
        }
        for record in &self.modules {
            record.trace_heap_edges(tracer);
        }
        for state in &self.regexp_legacy_static_states {
            state.trace_heap_edges(tracer);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct RegExpLiteralCacheKey {
    realm: RealmRef,
    code: CodeRef,
    site: u32,
}

impl RegExpLiteralCacheKey {
    #[inline]
    const fn new(realm: RealmRef, code: CodeRef, site: u32) -> Self {
        Self { realm, code, site }
    }
}

#[derive(Clone, Copy)]
struct RecentShortLatin1String {
    bytes: [u8; 3],
    len: u8,
    string: StringRef,
}

#[derive(Clone, Copy)]
struct RecentTwoCodeUnitString {
    units: [u16; 2],
    string: StringRef,
}

/// Agent-local runtime state. The `Rc` marker keeps the agent thread-affine.
pub struct Agent {
    id: AgentId,
    host_id: Option<HostAgentId>,
    debug_name: Option<String>,
    bound_thread: Option<HostThreadId>,
    heap: PrimitiveHeap,
    roots: PrimitiveRoots,
    atoms: AtomTable,
    bootstrap_atoms: BootstrapAtoms,
    well_known_symbols: WellKnownSymbols,
    global_symbol_registry: GlobalSymbolRegistry,
    objects: ObjectRuntime,
    backing_stores: ClusterBackingStoreHandle,
    shared_memory: ClusterSharedMemoryHandle,
    environment_layouts: Vec<Option<EnvironmentLayout>>,
    environment_metadata: Vec<Option<EnvironmentMetadata>>,
    realms: Vec<RealmRef>,
    realm_metadata: Vec<Option<RealmMetadata>>,
    default_realm: Option<RealmRef>,
    execution_contexts: Vec<ExecutionContext>,
    modules: BTreeMap<ModuleKey, ModuleRecord>,
    promise_tables: super::AgentPromiseTables,
    disposal_tables: super::AgentDisposalTables,
    job_queues: AgentJobQueues,
    regexp_literal_cache: HashMap<RegExpLiteralCacheKey, RegExpPayload>,
    kept_objects: Vec<WeakHeapRef>,
    latin1_single_code_unit_strings: [Option<StringRef>; 256],
    recent_short_latin1_strings: [Option<RecentShortLatin1String>; 256],
    recent_two_code_unit_string: Option<RecentTwoCodeUnitString>,
    next_job_id: u32,
    thread_affinity: PhantomData<Rc<()>>,
}

impl Agent {
    pub(crate) fn new(
        id: AgentId,
        host_id: Option<HostAgentId>,
        debug_name: Option<String>,
        backing_stores: ClusterBackingStoreHandle,
        shared_memory: ClusterSharedMemoryHandle,
    ) -> Self {
        let mut atoms = AtomTable::new();
        let bootstrap_atoms = BootstrapAtoms::new(&mut atoms);
        let mut agent = Self {
            id,
            host_id,
            debug_name,
            bound_thread: None,
            heap: PrimitiveHeap::new(),
            roots: PrimitiveRoots::new(),
            atoms,
            bootstrap_atoms,
            well_known_symbols: WellKnownSymbols::new(),
            global_symbol_registry: GlobalSymbolRegistry::new(),
            objects: ObjectRuntime::new(),
            backing_stores,
            shared_memory,
            environment_layouts: Vec::new(),
            environment_metadata: Vec::new(),
            realms: Vec::new(),
            realm_metadata: Vec::new(),
            default_realm: None,
            execution_contexts: Vec::new(),
            modules: BTreeMap::new(),
            promise_tables: super::AgentPromiseTables::default(),
            disposal_tables: super::AgentDisposalTables::default(),
            job_queues: AgentJobQueues::default(),
            regexp_literal_cache: HashMap::new(),
            kept_objects: Vec::new(),
            latin1_single_code_unit_strings: [None; 256],
            recent_short_latin1_strings: [None; 256],
            recent_two_code_unit_string: None,
            next_job_id: 1,
            thread_affinity: PhantomData,
        };
        agent.seed_builtin_symbol_state(AllocationLifetime::LongLived);
        let default_realm = agent.create_default_realm_shell(AllocationLifetime::LongLived);
        agent.default_realm = Some(default_realm);
        agent
    }

    #[inline]
    pub const fn id(&self) -> AgentId {
        self.id
    }

    #[inline]
    pub const fn host_id(&self) -> Option<HostAgentId> {
        self.host_id
    }

    #[inline]
    pub fn debug_name(&self) -> Option<&str> {
        self.debug_name.as_deref()
    }

    #[inline]
    pub const fn bound_thread(&self) -> Option<HostThreadId> {
        self.bound_thread
    }

    #[inline]
    pub(crate) const fn bind_thread(&mut self, thread_id: HostThreadId) {
        self.bound_thread = Some(thread_id);
    }

    #[inline]
    pub const fn heap(&self) -> &PrimitiveHeap {
        &self.heap
    }

    #[inline]
    pub const fn heap_mut(&mut self) -> &mut PrimitiveHeap {
        &mut self.heap
    }

    #[inline]
    pub const fn roots(&self) -> &PrimitiveRoots {
        &self.roots
    }

    #[inline]
    pub const fn objects(&self) -> &ObjectRuntime {
        &self.objects
    }

    #[inline]
    pub const fn objects_mut(&mut self) -> &mut ObjectRuntime {
        &mut self.objects
    }

    pub fn with_heap_and_objects<R>(
        &mut self,
        f: impl FnOnce(&mut PrimitiveHeap, &mut ObjectRuntime) -> R,
    ) -> R {
        f(&mut self.heap, &mut self.objects)
    }

    /// Ensures `id` is in dictionary mode, always firing watchpoints on the
    /// pre-call shape. (May fire spuriously even when no shape change occurred —
    /// Spec 1's `Recording` observers tolerate this.)
    ///
    /// This is the production entry point for dictionary transitions.
    /// Callers must use this rather than `objects.ensure_named_property_dictionary`
    /// directly so that shape-invalidation watchpoints are fired correctly.
    ///
    /// `vm_dispatch` routes any `AdaptiveProtoLoad` watchpoints fired by this
    /// transition to IC slot clearing. Callers without a `Vm` in scope pass
    /// `&mut NoopAdaptiveProtoLoadDispatch`.
    pub fn ensure_named_property_dictionary(
        &mut self,
        id: ObjectRef,
        vm_dispatch: &mut dyn AdaptiveProtoLoadDispatch,
    ) -> bool {
        let old_shape = self.heap.view().object(id).and_then(|r| r.shape());

        let ok = self.with_heap_and_objects(|heap, objects| {
            objects.ensure_named_property_dictionary(&mut heap.mutator(), id)
        });

        if let Some(old) = old_shape {
            self.fire_watchpoints_for_shape(old, vm_dispatch);
        }
        ok
    }

    /// Defines or updates one own property on `id`, always firing watchpoints on the
    /// pre-call shape. (May fire spuriously even when no shape change occurred —
    /// Spec 1's `Recording` observers tolerate this.)
    ///
    /// This is the production entry point for property definition.
    /// Callers must use this rather than calling `objects.define_own_property`
    /// through `with_heap_and_objects` directly so that shape-invalidation
    /// watchpoints are fired correctly.
    ///
    /// `vm_dispatch` routes any `AdaptiveProtoLoad` watchpoints fired by this
    /// transition to IC slot clearing. Callers without a `Vm` in scope pass
    /// `&mut NoopAdaptiveProtoLoadDispatch`.
    pub fn define_own_property(
        &mut self,
        id: ObjectRef,
        key: PropertyKey,
        descriptor: PropertyDescriptor,
        lifetime: AllocationLifetime,
        vm_dispatch: &mut dyn AdaptiveProtoLoadDispatch,
    ) -> InternalMethodResult<bool> {
        let old_shape = self.heap.view().object(id).and_then(|r| r.shape());

        let result = self.with_heap_and_objects(|heap, objects| {
            objects.define_own_property(&mut heap.mutator(), id, key, descriptor, lifetime)
        });

        if let Some(old) = old_shape {
            self.fire_watchpoints_for_shape(old, vm_dispatch);
        }
        result
    }

    /// Deletes one property from `id`, always firing watchpoints on the
    /// pre-call shape. (May fire spuriously even when no shape change occurred —
    /// Spec 1's `Recording` observers tolerate this.)
    ///
    /// This is the production entry point for property deletion.
    /// Callers must use this rather than calling `objects.delete` through
    /// `with_heap_and_objects` directly so that shape-invalidation watchpoints
    /// are fired correctly.
    ///
    /// `vm_dispatch` routes any `AdaptiveProtoLoad` watchpoints fired by this
    /// transition to IC slot clearing. Callers without a `Vm` in scope pass
    /// `&mut NoopAdaptiveProtoLoadDispatch`.
    pub fn delete(
        &mut self,
        id: ObjectRef,
        key: PropertyKey,
        vm_dispatch: &mut dyn AdaptiveProtoLoadDispatch,
    ) -> InternalMethodResult<bool> {
        let old_shape = self.heap.view().object(id).and_then(|r| r.shape());

        let result = self
            .with_heap_and_objects(|heap, objects| objects.delete(&mut heap.mutator(), id, key));

        if let Some(old) = old_shape {
            self.fire_watchpoints_for_shape(old, vm_dispatch);
        }
        result
    }

    /// Changes the `[[Prototype]]` of `id` to `prototype`, performing a shape
    /// transition and firing watchpoints on the pre-mutation shape.
    ///
    /// This is the production entry point for `Object.setPrototypeOf` and
    /// `Reflect.setPrototypeOf`. It validates the operation (immutable-prototype
    /// check, extensibility check, cycle check), then atomically transitions the
    /// object to a new shape whose prototype guard reflects the new prototype,
    /// fires watchpoints on the old shape, and bumps the invalidation epoch.
    ///
    /// Bootstrap/initialization code that sets prototypes before IC shapes are
    /// live should use `objects.set_prototype(...)` directly.
    ///
    /// `vm_dispatch` routes any `AdaptiveProtoLoad` watchpoints fired by this
    /// transition to IC slot clearing. Callers without a `Vm` in scope pass
    /// `&mut NoopAdaptiveProtoLoadDispatch`.
    pub fn set_prototype_of(
        &mut self,
        id: ObjectRef,
        prototype: Option<ObjectRef>,
        vm_dispatch: &mut dyn AdaptiveProtoLoadDispatch,
    ) -> InternalMethodResult<bool> {
        // 1. Read current prototype; short-circuit if unchanged.
        let (current, old_shape) = {
            let view = self.heap.view();
            let current = self.objects.get_prototype_of(view, id)?;
            let old_shape = view
                .object(id)
                .and_then(|r| r.shape())
                .ok_or(InternalMethodError::MissingObject)?;
            (current, old_shape)
        };
        if current == prototype {
            return Ok(true);
        }

        // 2. Reject if the object has an immutable prototype slot.
        if self.objects.has_immutable_prototype(id) {
            return Ok(false);
        }

        // 3. Reject if the object is non-extensible.
        if !self.objects.is_extensible(id)? {
            return Ok(false);
        }

        // 4. Reject if setting `prototype` would create a cycle.
        if let Some(p) = prototype {
            let view = self.heap.view();
            if self.objects.check_prototype_chain_contains(view, p, id)? {
                return Ok(false);
            }
        }

        // 5. Resolve (or allocate) the post-mutation shape.
        let key = PrototypeKey::from_optional(prototype);
        let new_shape = self.with_heap_and_objects(|heap, objects| {
            objects.resolve_prototype_transition(
                &mut heap.mutator(),
                old_shape,
                key,
                AllocationLifetime::Default,
            )
        });

        // 6. Commit: update the object's shape pointer and prototype handle.
        let shape_ok = self.with_heap_and_objects(|heap, objects| {
            objects.retarget_shape(&mut heap.mutator(), id, new_shape)
        });
        if !shape_ok {
            return Err(InternalMethodError::CorruptObjectState);
        }
        // Both stores act on the same live object — `retarget_shape` succeeding
        // guarantees `id` exists in the heap, so the prototype store cannot fail
        // here.  The error branch is kept for completeness but documented as
        // unreachable in practice.
        let proto_ok = self
            .heap
            .mutator()
            .mut_store_object_handle(ObjectHandleStoreTarget::ObjectPrototype(id), prototype);
        debug_assert!(
            proto_ok,
            "prototype store after successful retarget_shape must succeed: id={id:?}"
        );
        if !proto_ok {
            return Err(InternalMethodError::CorruptObjectState);
        }

        // 7. Fire watchpoints on the OLD shape — after the object's shape pointer
        //    has been updated. Matches JSC's setPrototypeDirect ordering.
        self.fire_watchpoints_for_shape(old_shape, vm_dispatch);

        // 8. Bump the invalidation epoch.
        self.with_heap_and_objects(|heap, objects| {
            objects.bump_prototype_mutation_epoch(&mut heap.mutator(), id);
        });

        Ok(true)
    }

    /// Drains watchpoints registered against `shape` and dispatches each one.
    /// `ObjectRuntime::drain_watchpoints_for_shape` handles the side-table
    /// drain and invalidation; dispatch of all observer kinds happens here so
    /// that Spec 2's `AdaptiveProtoLoad` observer can call into `&mut Vm`
    /// without violating Rust borrow rules.
    ///
    /// `vm_dispatch` is the routing target for `AdaptiveProtoLoad` fires.
    /// VM-side callers pass `self` (Vm implements the trait); bootstrap and
    /// internal callers without a `Vm` in scope pass
    /// `&mut NoopAdaptiveProtoLoadDispatch` (correct only when no
    /// `AdaptiveProtoLoad` watchpoint can be registered against `shape`,
    /// i.e., during runtime setup before any JS executes).
    pub fn fire_watchpoints_for_shape(
        &mut self,
        shape: ShapeId,
        vm_dispatch: &mut dyn AdaptiveProtoLoadDispatch,
    ) {
        let Some(fired) = self.objects.drain_watchpoints_for_shape(shape) else {
            return;
        };
        for wp in fired {
            match wp {
                Watchpoint::ShapeInvalidation { observer } => match observer {
                    ShapeInvalidationObserver::Recording { token } => {
                        self.objects.push_recording_fire(token);
                    }
                    ShapeInvalidationObserver::AdaptiveProtoLoad {
                        code,
                        slot,
                        generation,
                    } => {
                        vm_dispatch.clear_ic_slot_if_generation_matches(code, slot, generation);
                    }
                },
            }
        }
    }

    /// Spec 2 Phase A: registers `AdaptiveProtoLoad` watchpoints on each shape
    /// in a proto-cache's dependency chain (excluding the receiver, which is
    /// covered by the IC cache-hit shape compare) and returns the post-bump
    /// generation on success.
    ///
    /// `vm_dispatch` is used to mint the new generation and to roll the slot
    /// back to a fresh state if any shape in `chain_shapes` is already
    /// `Invalidated`; in that case the caller observes `Err(())` and must
    /// abandon the install (i.e., not write the IC handler). After an abandon
    /// the slot's generation has been bumped twice — first by the install
    /// attempt, then again by the rollback `clear_ic_slot_if_generation_matches`
    /// path — so any watchpoints registered earlier in the same call will
    /// carry a stale generation and no-op when they fire.
    ///
    /// `chain_shapes` must contain exactly the shapes that the IC cache-hit
    /// path does *not* validate via direct shape compare. For a
    /// `PrototypeData` entry that is the prototype objects' shapes
    /// (entry dependencies `[1..dependency_count]`).
    pub fn register_adaptive_proto_load_for_chain(
        &mut self,
        code: CodeRef,
        slot: FeedbackSlotId,
        chain_shapes: &[ShapeId],
        vm_dispatch: &mut dyn AdaptiveProtoLoadDispatch,
    ) -> Result<u32, ()> {
        // Bump generation BEFORE registration so the new watchpoints carry
        // the post-install generation.
        let generation = vm_dispatch.bump_generation_for_install(code, slot);

        for &shape in chain_shapes {
            let result =
                self.objects
                    .watchpoint_set_mut(shape)
                    .register(Watchpoint::ShapeInvalidation {
                        observer: ShapeInvalidationObserver::AdaptiveProtoLoad {
                            code,
                            slot,
                            generation,
                        },
                    });
            if result.is_err() {
                // Some shape is already Invalidated — abandon install.
                // Clear+bump again so any orphan watchpoints registered
                // earlier in this same call (on the shapes we already walked)
                // carry a stale generation and no-op when they fire.
                vm_dispatch.clear_ic_slot_if_generation_matches(code, slot, generation);
                return Err(());
            }
        }

        Ok(generation)
    }

    /// Records a property-addition transition on `obj`'s current shape (which
    /// becomes the parent of the new child shape), always firing watchpoints
    /// registered on the parent shape after the transition is recorded.
    ///
    /// May fire spuriously when the child already existed in the parent's
    /// transition table (Spec 1's `Recording` observers tolerate this).
    ///
    /// This is the production entry point for property-addition shape
    /// transitions. Callers must use this rather than calling
    /// `objects.transition_shape` through `with_heap_and_objects` directly so
    /// that shape-invalidation watchpoints are fired correctly.
    ///
    /// `vm_dispatch` routes any `AdaptiveProtoLoad` watchpoints fired by this
    /// transition to IC slot clearing. Callers without a `Vm` in scope pass
    /// `&mut NoopAdaptiveProtoLoadDispatch`.
    pub fn transition_shape(
        &mut self,
        obj: ObjectRef,
        transition: ShapeTransitionKey,
        lifetime: AllocationLifetime,
        vm_dispatch: &mut dyn AdaptiveProtoLoadDispatch,
    ) -> Option<ShapeId> {
        let parent_shape = self.heap.view().object(obj).and_then(|r| r.shape());

        let result = self.with_heap_and_objects(|heap, objects| {
            let parent = parent_shape?;
            objects.transition_shape(&mut heap.mutator(), parent, transition, lifetime)
        });

        if let Some(parent) = parent_shape {
            self.fire_watchpoints_for_shape(parent, vm_dispatch);
        }
        result
    }
}
