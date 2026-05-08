use super::{
    AgentId, AgentJobQueues, AllocationLifetime, AtomTable, BootstrapAtoms, EnvironmentLayout,
    EnvironmentMetadata, ExecutionContext, GlobalSymbolRegistry, HostAgentId, HostThreadId,
    Intrinsics, ModuleRecord, ObjectRuntime, PrimitiveHeap, PrimitiveRoots, RealmMetadata,
    RealmRef, RegExpLegacyStaticState, WellKnownSymbols,
};
use lyng_js_gc::{PrimitiveTracer, TraceHeapEdges, WeakHeapRef};
use lyng_js_host::ModuleKey;
use lyng_js_objects::RegExpPayload;
use lyng_js_types::{CodeRef, StringRef};
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
}
