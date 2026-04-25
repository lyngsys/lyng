use super::{
    environment_index, layout_index, realm_index, total_live_bytes, AgentId, AgentJobQueues,
    AgentPhase6Accounting, AllocationLifetime, AsyncDisposalOperationId, AsyncDisposalResumeId,
    AsyncDisposalResumeRecord, AsyncWaiterRecord, AtomTable, BackingStoreRuntime, BootstrapAtoms,
    CodeRef, DeclarativeEnvironmentRecord, DisposableResourceRecord, DisposalCapabilityId,
    DisposalCapabilityKind, DisposalCapabilityRecord, DisposalCapabilityState, EnvironmentLayout,
    EnvironmentLayoutId, EnvironmentLayoutKind, EnvironmentMetadata, EnvironmentRecord,
    EnvironmentRef, EnvironmentSlotsRef, ExecutableId, ExecutionContext, FunctionEnvironmentRecord,
    GlobalEnvironmentRecord, GlobalLexicalBindingRecord, GlobalSymbolRegistry,
    GlobalSymbolRegistryEntry, HostAgentId, HostJobKind, HostThreadId, Intrinsics, JobId,
    JobQueueKind, ModuleBindingAlias, ModuleEnvironmentRecord, ModuleRecord, ModuleResolvedExport,
    ModuleStatus, ObjectEnvironmentRecord, ObjectHandleStoreTarget, ObjectRuntime,
    ParkedAgentRecord, PrimitiveHeap, PrimitiveRoots, PrivateEnvironmentRecord,
    PromiseCapabilityId, PromiseFinallyFunctionRecord, PromiseId, PromiseReactionKind,
    PromiseReactionRecord, PromiseResolvingFunctionRecord, RealmBootstrapState, RealmMetadata,
    RealmRecord, RealmRef, RuntimeEnvironmentRecord, RuntimeJob, RuntimeJobPayload,
    RuntimeRealmRecord, SharedBackingStoreRecord, ThisBindingStatus, ThisState, ValueStoreTarget,
    WaitLocation, WaiterRecord, WaiterToken, WellKnownSymbols,
};
use lyng_js_common::AtomId;
use lyng_js_gc::{
    PrimitiveCollectionReport, PrimitiveTracer, StringEncoding, SymbolFlags, TraceHeapEdges,
    WeakHeapRef,
};
use lyng_js_host::{ImportMetaProperties, ModuleKey};
use lyng_js_objects::ObjectAllocation;
use lyng_js_types::{
    internal_finalization_registry_cleanup_job_builtin, BackingStoreRef, ObjectRef, StringRef,
    SymbolRef, Value, WellKnownSymbolId,
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
    marker::PhantomData,
    rc::Rc,
};

#[derive(Clone)]
pub(crate) struct ClusterBackingStoreHandle(Rc<RefCell<BackingStoreRuntime>>);

impl ClusterBackingStoreHandle {
    #[inline]
    pub(crate) fn new(runtime: Rc<RefCell<BackingStoreRuntime>>) -> Self {
        Self(runtime)
    }

    #[inline]
    fn allocate(&self, byte_length: usize) -> Option<BackingStoreRef> {
        self.0.borrow_mut().allocate(byte_length)
    }

    #[inline]
    fn allocate_shared(&self, byte_length: usize) -> Option<BackingStoreRef> {
        self.0.borrow_mut().allocate_shared(byte_length)
    }

    #[inline]
    fn clone_range(
        &self,
        store: BackingStoreRef,
        start: usize,
        end: usize,
    ) -> Option<BackingStoreRef> {
        self.0.borrow_mut().clone_range(store, start, end)
    }

    #[inline]
    fn byte_length(&self, store: BackingStoreRef) -> Option<usize> {
        self.0.borrow().byte_length(store)
    }

    #[inline]
    fn is_detached(&self, store: BackingStoreRef) -> Option<bool> {
        self.0.borrow().is_detached(store)
    }

    #[inline]
    fn is_shared(&self, store: BackingStoreRef) -> Option<bool> {
        self.0.borrow().is_shared(store)
    }

    #[inline]
    fn get_byte(&self, store: BackingStoreRef, index: usize) -> Option<u8> {
        self.0.borrow().get_byte(store, index)
    }

    #[inline]
    fn set_byte(&self, store: BackingStoreRef, index: usize, value: u8) -> bool {
        self.0.borrow_mut().set_byte(store, index, value)
    }

    #[inline]
    fn atomic_load_bits(
        &self,
        store: BackingStoreRef,
        index: usize,
        byte_width: usize,
    ) -> Option<u64> {
        self.0.borrow().atomic_load_bits(store, index, byte_width)
    }

    #[inline]
    fn atomic_store_bits(
        &self,
        store: BackingStoreRef,
        index: usize,
        byte_width: usize,
        bits: u64,
    ) -> bool {
        self.0
            .borrow_mut()
            .atomic_store_bits(store, index, byte_width, bits)
    }

    #[inline]
    fn atomic_compare_exchange_bits(
        &self,
        store: BackingStoreRef,
        index: usize,
        byte_width: usize,
        expected: u64,
        replacement: u64,
    ) -> Option<u64> {
        self.0.borrow_mut().atomic_compare_exchange_bits(
            store,
            index,
            byte_width,
            expected,
            replacement,
        )
    }

    #[inline]
    fn detach(&self, store: BackingStoreRef) -> bool {
        self.0.borrow_mut().detach(store)
    }
}

#[derive(Clone)]
pub(crate) struct ClusterSharedMemoryHandle(Rc<RefCell<super::SharedMemoryRuntime>>);

impl ClusterSharedMemoryHandle {
    #[inline]
    pub(crate) fn new(runtime: Rc<RefCell<super::SharedMemoryRuntime>>) -> Self {
        Self(runtime)
    }

    #[inline]
    fn register_shared_backing_store(
        &self,
        owner: AgentId,
        backing_store: BackingStoreRef,
        byte_length: usize,
    ) -> bool {
        self.0
            .borrow_mut()
            .register_shared_backing_store(owner, backing_store, byte_length)
    }

    #[inline]
    fn cache_shared_backing_store_handle(
        &self,
        backing_store: BackingStoreRef,
        shared_buffer: lyng_js_host::HostSharedBufferId,
    ) -> bool {
        self.0
            .borrow_mut()
            .cache_shared_backing_store_handle(backing_store, shared_buffer)
    }

    #[inline]
    fn share_shared_backing_store(&self, backing_store: BackingStoreRef, target: AgentId) -> bool {
        self.0
            .borrow_mut()
            .share_shared_backing_store(backing_store, target)
    }

    #[inline]
    fn shared_backing_store(
        &self,
        backing_store: BackingStoreRef,
    ) -> Option<SharedBackingStoreRecord> {
        self.0.borrow().shared_backing_store(backing_store).cloned()
    }

    #[inline]
    fn park_agent(&self, location: WaitLocation, parked: ParkedAgentRecord) -> Option<WaiterToken> {
        self.0.borrow_mut().park_agent(location, parked)
    }

    #[inline]
    fn park_async_waiter(&self, location: WaitLocation, parked: AsyncWaiterRecord) -> WaiterToken {
        self.0.borrow_mut().park_async_waiter(location, parked)
    }

    #[inline]
    fn remove_waiter(&self, location: WaitLocation, token: WaiterToken) -> bool {
        self.0.borrow_mut().remove_waiter(location, token)
    }

    #[inline]
    fn waiter_count(&self, location: WaitLocation) -> usize {
        self.0.borrow().waiter_count(location)
    }

    #[inline]
    fn wake_waiters(&self, location: WaitLocation, max_count: u32) -> Vec<WaiterRecord> {
        self.0.borrow_mut().wake_waiters(location, max_count)
    }
}

#[derive(Clone)]
struct AgentCollectionSnapshot {
    well_known_symbols: WellKnownSymbols,
    global_symbol_registry: GlobalSymbolRegistry,
    realms: Vec<RealmRef>,
    execution_contexts: Vec<ExecutionContext>,
    modules: Vec<ModuleRecord>,
    promise_tables: super::AgentPromiseTables,
    disposal_tables: super::AgentDisposalTables,
    job_queues: AgentJobQueues,
    kept_objects: Vec<WeakHeapRef>,
}

impl AgentCollectionSnapshot {
    fn from_agent(agent: &Agent) -> Self {
        Self {
            well_known_symbols: agent.well_known_symbols,
            global_symbol_registry: agent.global_symbol_registry.clone(),
            realms: agent.realms.clone(),
            execution_contexts: agent.execution_contexts.clone(),
            modules: agent.modules.values().cloned().collect(),
            promise_tables: agent.promise_tables.clone(),
            disposal_tables: agent.disposal_tables.clone(),
            job_queues: agent.job_queues.clone(),
            kept_objects: agent.kept_objects.clone(),
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
        for context in &self.execution_contexts {
            context.trace_heap_edges(tracer);
        }
        self.promise_tables.trace_heap_edges(tracer);
        self.disposal_tables.trace_heap_edges(tracer);
        self.job_queues.trace_heap_edges(tracer);
        for target in &self.kept_objects {
            target.trace_heap_edges(tracer);
        }
        for record in &self.modules {
            record.trace_heap_edges(tracer);
        }
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
    kept_objects: Vec<WeakHeapRef>,
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
            kept_objects: Vec::new(),
            next_job_id: 1,
            thread_affinity: PhantomData,
        };
        agent.seed_phase5_symbol_state(AllocationLifetime::LongLived);
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
    pub(crate) fn bind_thread(&mut self, thread_id: HostThreadId) {
        self.bound_thread = Some(thread_id);
    }

    #[cfg(test)]
    pub(crate) fn set_next_job_id_for_test(&mut self, next_job_id: u32) {
        self.next_job_id = next_job_id;
    }

    #[inline]
    pub fn heap(&self) -> &PrimitiveHeap {
        &self.heap
    }

    #[inline]
    pub fn heap_mut(&mut self) -> &mut PrimitiveHeap {
        &mut self.heap
    }

    #[inline]
    pub fn roots(&self) -> &PrimitiveRoots {
        &self.roots
    }

    #[inline]
    pub fn atoms(&self) -> &AtomTable {
        &self.atoms
    }

    #[inline]
    pub const fn bootstrap_atoms(&self) -> BootstrapAtoms {
        self.bootstrap_atoms
    }

    #[inline]
    pub const fn well_known_symbols(&self) -> WellKnownSymbols {
        self.well_known_symbols
    }

    #[inline]
    pub const fn well_known_symbol(&self, id: WellKnownSymbolId) -> Option<SymbolRef> {
        self.well_known_symbols.get(id)
    }

    #[inline]
    pub fn global_symbol_registry(&self) -> &[GlobalSymbolRegistryEntry] {
        self.global_symbol_registry.entries()
    }

    #[inline]
    pub fn global_symbol(&self, key: AtomId) -> Option<SymbolRef> {
        self.global_symbol_registry.symbol_for(key)
    }

    #[inline]
    pub fn global_symbol_key_for(&self, symbol: SymbolRef) -> Option<AtomId> {
        self.global_symbol_registry.key_for(symbol)
    }

    #[inline]
    pub fn atoms_mut(&mut self) -> &mut AtomTable {
        &mut self.atoms
    }

    #[inline]
    pub fn objects(&self) -> &ObjectRuntime {
        &self.objects
    }

    #[inline]
    pub fn objects_mut(&mut self) -> &mut ObjectRuntime {
        &mut self.objects
    }

    pub fn with_heap_and_objects<R>(
        &mut self,
        f: impl FnOnce(&mut PrimitiveHeap, &mut ObjectRuntime) -> R,
    ) -> R {
        f(&mut self.heap, &mut self.objects)
    }

    #[inline]
    pub fn allocate_backing_store(&mut self, byte_length: usize) -> Option<BackingStoreRef> {
        self.backing_stores.allocate(byte_length)
    }

    #[inline]
    pub fn allocate_shared_backing_store(&mut self, byte_length: usize) -> Option<BackingStoreRef> {
        let backing_store = self.backing_stores.allocate_shared(byte_length)?;
        let registered =
            self.shared_memory
                .register_shared_backing_store(self.id, backing_store, byte_length);
        if registered {
            Some(backing_store)
        } else {
            None
        }
    }

    #[inline]
    pub fn clone_backing_store_range(
        &mut self,
        store: BackingStoreRef,
        start: usize,
        end: usize,
    ) -> Option<BackingStoreRef> {
        self.backing_stores.clone_range(store, start, end)
    }

    #[inline]
    pub fn backing_store_byte_length(&self, store: BackingStoreRef) -> Option<usize> {
        self.backing_stores.byte_length(store)
    }

    #[inline]
    pub fn backing_store_is_detached(&self, store: BackingStoreRef) -> Option<bool> {
        self.backing_stores.is_detached(store)
    }

    #[inline]
    pub fn backing_store_is_shared(&self, store: BackingStoreRef) -> Option<bool> {
        self.backing_stores.is_shared(store)
    }

    #[inline]
    pub fn backing_store_get_byte(&self, store: BackingStoreRef, index: usize) -> Option<u8> {
        self.backing_stores.get_byte(store, index)
    }

    #[inline]
    pub fn backing_store_set_byte(
        &mut self,
        store: BackingStoreRef,
        index: usize,
        value: u8,
    ) -> bool {
        self.backing_stores.set_byte(store, index, value)
    }

    #[inline]
    pub fn backing_store_atomic_load_bits(
        &self,
        store: BackingStoreRef,
        index: usize,
        byte_width: usize,
    ) -> Option<u64> {
        self.backing_stores
            .atomic_load_bits(store, index, byte_width)
    }

    #[inline]
    pub fn backing_store_atomic_store_bits(
        &mut self,
        store: BackingStoreRef,
        index: usize,
        byte_width: usize,
        bits: u64,
    ) -> bool {
        self.backing_stores
            .atomic_store_bits(store, index, byte_width, bits)
    }

    #[inline]
    pub fn backing_store_atomic_compare_exchange_bits(
        &mut self,
        store: BackingStoreRef,
        index: usize,
        byte_width: usize,
        expected: u64,
        replacement: u64,
    ) -> Option<u64> {
        self.backing_stores.atomic_compare_exchange_bits(
            store,
            index,
            byte_width,
            expected,
            replacement,
        )
    }

    #[inline]
    pub fn detach_backing_store(&mut self, store: BackingStoreRef) -> bool {
        self.backing_stores.detach(store)
    }

    #[inline]
    pub fn cache_shared_backing_store_handle(
        &mut self,
        backing_store: BackingStoreRef,
        shared_buffer: lyng_js_host::HostSharedBufferId,
    ) -> bool {
        self.shared_memory
            .cache_shared_backing_store_handle(backing_store, shared_buffer)
    }

    #[inline]
    pub fn share_shared_backing_store(
        &mut self,
        backing_store: BackingStoreRef,
        target: AgentId,
    ) -> bool {
        self.shared_memory
            .share_shared_backing_store(backing_store, target)
    }

    #[inline]
    pub fn shared_backing_store(
        &self,
        backing_store: BackingStoreRef,
    ) -> Option<SharedBackingStoreRecord> {
        self.shared_memory.shared_backing_store(backing_store)
    }

    #[inline]
    pub fn park_shared_memory_waiter(
        &mut self,
        location: WaitLocation,
        parked: ParkedAgentRecord,
    ) -> Option<WaiterToken> {
        self.shared_memory.park_agent(location, parked)
    }

    #[inline]
    pub fn park_async_shared_memory_waiter(
        &mut self,
        location: WaitLocation,
        parked: AsyncWaiterRecord,
    ) -> WaiterToken {
        self.shared_memory.park_async_waiter(location, parked)
    }

    #[inline]
    pub fn remove_shared_memory_waiter(
        &mut self,
        location: WaitLocation,
        token: WaiterToken,
    ) -> bool {
        self.shared_memory.remove_waiter(location, token)
    }

    #[inline]
    pub fn shared_memory_waiter_count(&self, location: WaitLocation) -> usize {
        self.shared_memory.waiter_count(location)
    }

    #[inline]
    pub fn wake_shared_memory_waiters(
        &mut self,
        location: WaitLocation,
        max_count: u32,
    ) -> Vec<WaiterRecord> {
        self.shared_memory.wake_waiters(location, max_count)
    }

    pub fn ensure_well_known_symbol(
        &mut self,
        id: WellKnownSymbolId,
        lifetime: AllocationLifetime,
    ) -> SymbolRef {
        if let Some(symbol) = self.well_known_symbol(id) {
            return symbol;
        }

        let description_atom = self.bootstrap_atoms.well_known_symbol_description(id);
        let description = self.alloc_string_for_atom(description_atom, lifetime);
        let symbol = self.heap.mutator().alloc_symbol(
            Some(description),
            SymbolFlags::well_known(),
            lifetime,
        );
        self.well_known_symbols.set(id, Some(symbol));
        symbol
    }

    pub fn global_symbol_for(&mut self, key: AtomId, lifetime: AllocationLifetime) -> SymbolRef {
        if let Some(symbol) = self.global_symbol(key) {
            return symbol;
        }

        let description = self.alloc_string_for_atom(key, lifetime);
        let symbol =
            self.heap
                .mutator()
                .alloc_symbol(Some(description), SymbolFlags::ordinary(), lifetime);
        self.global_symbol_registry.insert(key, symbol)
    }

    pub fn alloc_runtime_string(
        &mut self,
        text: &str,
        cached_atom: Option<AtomId>,
        lifetime: AllocationLifetime,
    ) -> StringRef {
        if let Some(atom) = cached_atom {
            return self.alloc_string_for_atom(atom, lifetime);
        }
        if text.chars().all(|ch| u32::from(ch) <= u32::from(u8::MAX)) {
            let bytes: Vec<u8> = text
                .chars()
                .map(|ch| u8::try_from(u32::from(ch)).expect("Latin-1 code point must fit into u8"))
                .collect();
            return self.heap.mutator().alloc_string(
                StringEncoding::Latin1,
                u32::try_from(bytes.len()).expect("Latin-1 string length must fit into u32"),
                &bytes,
                None,
                lifetime,
            );
        }

        let code_units: Vec<u16> = text.encode_utf16().collect();
        let mut bytes = Vec::with_capacity(code_units.len() * 2);
        for unit in &code_units {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        self.heap.mutator().alloc_string(
            StringEncoding::Utf16,
            u32::try_from(code_units.len()).expect("UTF-16 string length must fit into u32"),
            &bytes,
            None,
            lifetime,
        )
    }

    /// Allocates one immutable environment layout record.
    ///
    /// # Panics
    /// Panics if the layout table grows beyond the `u32` id range.
    pub fn alloc_environment_layout(&mut self, layout: EnvironmentLayout) -> EnvironmentLayoutId {
        let raw_id = u32::try_from(self.environment_layouts.len() + 1)
            .expect("environment layout id must fit into u32");
        let id = EnvironmentLayoutId::from_raw(raw_id)
            .expect("environment layout id must stay non-zero");
        self.environment_layouts.push(Some(layout));
        id
    }

    pub fn environment_layout(&self, id: EnvironmentLayoutId) -> Option<&EnvironmentLayout> {
        self.environment_layouts.get(layout_index(id))?.as_ref()
    }

    pub fn alloc_declarative_environment(
        &mut self,
        outer: Option<EnvironmentRef>,
        layout: EnvironmentLayoutId,
        lifetime: AllocationLifetime,
    ) -> Option<EnvironmentRef> {
        if self.environment_layout(layout)?.kind() != EnvironmentLayoutKind::Declarative {
            return None;
        }
        Some(self.alloc_environment_record(
            outer,
            Some(layout),
            None,
            Value::undefined(),
            None,
            None,
            EnvironmentMetadata::Declarative { layout },
            lifetime,
        ))
    }

    pub fn alloc_private_environment(
        &mut self,
        outer: Option<EnvironmentRef>,
        layout: EnvironmentLayoutId,
        lifetime: AllocationLifetime,
    ) -> Option<EnvironmentRef> {
        if self.environment_layout(layout)?.kind() != EnvironmentLayoutKind::Private {
            return None;
        }
        Some(self.alloc_environment_record(
            outer,
            Some(layout),
            None,
            Value::undefined(),
            None,
            None,
            EnvironmentMetadata::Private { layout },
            lifetime,
        ))
    }

    pub fn alloc_module_environment(
        &mut self,
        outer: Option<EnvironmentRef>,
        layout: EnvironmentLayoutId,
        lifetime: AllocationLifetime,
    ) -> Option<EnvironmentRef> {
        if self.environment_layout(layout)?.kind() != EnvironmentLayoutKind::Module {
            return None;
        }
        Some(self.alloc_environment_record(
            outer,
            Some(layout),
            None,
            Value::undefined(),
            None,
            None,
            EnvironmentMetadata::Module {
                layout,
                import_aliases: vec![
                    None;
                    usize::try_from(
                        self.environment_layout(layout)
                            .expect("checked module layout should exist")
                            .slot_count()
                    )
                    .expect("module layout slot count must fit into usize")
                ],
            },
            lifetime,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn alloc_function_environment(
        &mut self,
        outer: Option<EnvironmentRef>,
        layout: EnvironmentLayoutId,
        function_object: ObjectRef,
        this_binding_status: ThisBindingStatus,
        this_value: Value,
        new_target: Option<ObjectRef>,
        home_object: Option<ObjectRef>,
        lifetime: AllocationLifetime,
    ) -> Option<EnvironmentRef> {
        if self.environment_layout(layout)?.kind() != EnvironmentLayoutKind::Function {
            return None;
        }
        Some(self.alloc_environment_record(
            outer,
            Some(layout),
            Some(function_object),
            this_value,
            new_target,
            home_object,
            EnvironmentMetadata::Function {
                layout,
                this_binding_status,
            },
            lifetime,
        ))
    }

    pub fn alloc_global_environment(
        &mut self,
        outer: Option<EnvironmentRef>,
        layout: EnvironmentLayoutId,
        global_object: ObjectRef,
        lifetime: AllocationLifetime,
    ) -> Option<EnvironmentRef> {
        if self.environment_layout(layout)?.kind() != EnvironmentLayoutKind::Global {
            return None;
        }
        Some(self.alloc_environment_record(
            outer,
            Some(layout),
            Some(global_object),
            Value::undefined(),
            None,
            None,
            EnvironmentMetadata::Global {
                layout,
                lexical_names: HashSet::new(),
                lexical_bindings: Vec::new(),
                var_names: HashSet::new(),
            },
            lifetime,
        ))
    }

    /// Allocates one object environment record.
    pub fn alloc_object_environment(
        &mut self,
        outer: Option<EnvironmentRef>,
        binding_object: ObjectRef,
        with_environment: bool,
        lifetime: AllocationLifetime,
    ) -> EnvironmentRef {
        self.alloc_environment_record(
            outer,
            None,
            Some(binding_object),
            Value::undefined(),
            None,
            None,
            EnvironmentMetadata::Object { with_environment },
            lifetime,
        )
    }

    pub fn environment(&self, id: EnvironmentRef) -> Option<EnvironmentRecord> {
        let record = self.heap.view().environment(id)?;
        let metadata = self.environment_metadata(id)?;
        match metadata {
            EnvironmentMetadata::Declarative { layout } => Some(EnvironmentRecord::Declarative(
                DeclarativeEnvironmentRecord {
                    id,
                    outer: record.outer(),
                    layout: *layout,
                    slots: record.slots(),
                },
            )),
            EnvironmentMetadata::Private { layout } => {
                Some(EnvironmentRecord::Private(PrivateEnvironmentRecord {
                    id,
                    outer: record.outer(),
                    layout: *layout,
                    slots: record.slots(),
                }))
            }
            EnvironmentMetadata::Function {
                layout,
                this_binding_status,
            } => Some(EnvironmentRecord::Function(FunctionEnvironmentRecord {
                declarative: DeclarativeEnvironmentRecord {
                    id,
                    outer: record.outer(),
                    layout: *layout,
                    slots: record.slots(),
                },
                function_object: record.function_object()?,
                this_binding_status: *this_binding_status,
                this_value: record.this_value(),
                new_target: record.new_target(),
                home_object: record.home_object(),
            })),
            EnvironmentMetadata::Module { layout, .. } => {
                Some(EnvironmentRecord::Module(ModuleEnvironmentRecord {
                    declarative: DeclarativeEnvironmentRecord {
                        id,
                        outer: record.outer(),
                        layout: *layout,
                        slots: record.slots(),
                    },
                }))
            }
            EnvironmentMetadata::Global {
                layout,
                lexical_names,
                lexical_bindings,
                var_names,
            } => Some(EnvironmentRecord::Global(GlobalEnvironmentRecord {
                id,
                outer: record.outer(),
                layout: *layout,
                lexical_slots: record.slots(),
                global_object: record.function_object()?,
                lexical_names: lexical_names.clone(),
                lexical_bindings: lexical_bindings.clone(),
                var_names: var_names.clone(),
            })),
            EnvironmentMetadata::Object { with_environment } => {
                Some(EnvironmentRecord::Object(ObjectEnvironmentRecord {
                    id,
                    outer: record.outer(),
                    binding_object: record.function_object()?,
                    with_environment: *with_environment,
                }))
            }
        }
    }

    pub fn private_environment(&self, id: EnvironmentRef) -> Option<PrivateEnvironmentRecord> {
        match self.environment(id)? {
            EnvironmentRecord::Private(record) => Some(record),
            _ => None,
        }
    }

    pub fn declarative_environment(
        &self,
        id: EnvironmentRef,
    ) -> Option<DeclarativeEnvironmentRecord> {
        match self.environment(id)? {
            EnvironmentRecord::Declarative(record) => Some(record),
            _ => None,
        }
    }

    pub fn function_environment(&self, id: EnvironmentRef) -> Option<FunctionEnvironmentRecord> {
        match self.environment(id)? {
            EnvironmentRecord::Function(record) => Some(record),
            _ => None,
        }
    }

    pub fn module_environment(&self, id: EnvironmentRef) -> Option<ModuleEnvironmentRecord> {
        match self.environment(id)? {
            EnvironmentRecord::Module(record) => Some(record),
            _ => None,
        }
    }

    pub fn global_environment(&self, id: EnvironmentRef) -> Option<GlobalEnvironmentRecord> {
        match self.environment(id)? {
            EnvironmentRecord::Global(record) => Some(record),
            _ => None,
        }
    }

    pub fn object_environment(&self, id: EnvironmentRef) -> Option<ObjectEnvironmentRecord> {
        match self.environment(id)? {
            EnvironmentRecord::Object(record) => Some(record),
            _ => None,
        }
    }

    pub fn environment_slots(&self, id: EnvironmentRef) -> Option<&[Value]> {
        let slots = self.heap.view().environment(id)?.slots()?;
        self.heap.view().environment_slots(slots)
    }

    pub fn module_binding_alias(
        &self,
        id: EnvironmentRef,
        slot: u32,
    ) -> Option<ModuleBindingAlias> {
        let EnvironmentMetadata::Module { import_aliases, .. } = self.environment_metadata(id)?
        else {
            return None;
        };
        import_aliases.get(slot as usize).copied().flatten()
    }

    pub fn set_module_binding_alias(
        &mut self,
        id: EnvironmentRef,
        slot: u32,
        alias: Option<ModuleBindingAlias>,
    ) -> bool {
        let Some(EnvironmentMetadata::Module { import_aliases, .. }) =
            self.environment_metadata_mut(id)
        else {
            return false;
        };
        let Some(target) = import_aliases.get_mut(slot as usize) else {
            return false;
        };
        *target = alias;
        true
    }

    fn resolved_environment_slot_target(
        &self,
        id: EnvironmentRef,
        index: u32,
    ) -> Option<(EnvironmentRef, u32)> {
        let mut environment = id;
        let mut slot = index;
        let mut traversed = 0usize;
        loop {
            if traversed >= self.environment_metadata.len().max(1) {
                return None;
            }
            let Some(alias) = self.module_binding_alias(environment, slot) else {
                return Some((environment, slot));
            };
            environment = alias.environment();
            slot = alias.slot();
            traversed = traversed.saturating_add(1);
        }
    }

    pub fn environment_slot(&self, id: EnvironmentRef, index: u32) -> Option<Value> {
        let (id, index) = self.resolved_environment_slot_target(id, index)?;
        self.environment_slots(id)?.get(index as usize).copied()
    }

    pub fn init_environment_slot(&mut self, id: EnvironmentRef, index: u32, value: Value) -> bool {
        let Some((id, index)) = self.resolved_environment_slot_target(id, index) else {
            return false;
        };
        let Some(slots) = self
            .heap
            .view()
            .environment(id)
            .and_then(RuntimeEnvironmentRecord::slots)
        else {
            return false;
        };
        self.heap
            .mutator()
            .init_store_value(ValueStoreTarget::EnvironmentSlot(slots, index), value)
    }

    pub fn set_environment_slot(&mut self, id: EnvironmentRef, index: u32, value: Value) -> bool {
        let Some((id, index)) = self.resolved_environment_slot_target(id, index) else {
            return false;
        };
        let Some(slots) = self
            .heap
            .view()
            .environment(id)
            .and_then(RuntimeEnvironmentRecord::slots)
        else {
            return false;
        };
        self.heap
            .mutator()
            .mut_store_value(ValueStoreTarget::EnvironmentSlot(slots, index), value)
    }

    pub fn set_function_this_binding(
        &mut self,
        id: EnvironmentRef,
        status: ThisBindingStatus,
        value: Value,
    ) -> bool {
        let Some(EnvironmentMetadata::Function {
            this_binding_status,
            ..
        }) = self.environment_metadata_mut(id)
        else {
            return false;
        };
        *this_binding_status = status;
        let stored_value = match status {
            ThisBindingStatus::Initialized => value,
            ThisBindingStatus::Lexical | ThisBindingStatus::Uninitialized => Value::undefined(),
        };
        self.heap
            .mutator()
            .mut_store_value(ValueStoreTarget::EnvironmentThisValue(id), stored_value)
    }

    pub fn set_function_new_target(
        &mut self,
        id: EnvironmentRef,
        new_target: Option<ObjectRef>,
    ) -> bool {
        matches!(
            self.environment_metadata(id),
            Some(EnvironmentMetadata::Function { .. })
        ) && self.heap.mutator().mut_store_object_handle(
            ObjectHandleStoreTarget::EnvironmentNewTarget(id),
            new_target,
        )
    }

    pub fn set_function_home_object(
        &mut self,
        id: EnvironmentRef,
        home_object: Option<ObjectRef>,
    ) -> bool {
        matches!(
            self.environment_metadata(id),
            Some(EnvironmentMetadata::Function { .. })
        ) && self.heap.mutator().mut_store_object_handle(
            ObjectHandleStoreTarget::EnvironmentHomeObject(id),
            home_object,
        )
    }

    pub fn global_has_lexical_name(&self, id: EnvironmentRef, name: AtomId) -> bool {
        match self.environment_metadata(id) {
            Some(EnvironmentMetadata::Global { lexical_names, .. }) => {
                lexical_names.contains(&name)
            }
            _ => false,
        }
    }

    pub fn global_lexical_binding(
        &self,
        id: EnvironmentRef,
        name: AtomId,
    ) -> Option<GlobalLexicalBindingRecord> {
        match self.environment_metadata(id) {
            Some(EnvironmentMetadata::Global {
                lexical_bindings, ..
            }) => lexical_bindings
                .iter()
                .copied()
                .find(|binding| binding.name() == name),
            _ => None,
        }
    }

    pub fn global_add_lexical_name(&mut self, id: EnvironmentRef, name: AtomId) -> bool {
        let Some(EnvironmentMetadata::Global { lexical_names, .. }) =
            self.environment_metadata_mut(id)
        else {
            return false;
        };
        lexical_names.insert(name)
    }

    pub fn global_set_lexical_binding(
        &mut self,
        id: EnvironmentRef,
        name: AtomId,
        environment: EnvironmentRef,
        slot: u32,
    ) -> bool {
        let Some(EnvironmentMetadata::Global {
            lexical_bindings, ..
        }) = self.environment_metadata_mut(id)
        else {
            return false;
        };

        let binding = GlobalLexicalBindingRecord::new(name, environment, slot);
        if let Some(existing) = lexical_bindings
            .iter_mut()
            .find(|existing| existing.name() == name)
        {
            *existing = binding;
        } else {
            lexical_bindings.push(binding);
        }
        true
    }

    pub fn global_has_var_name(&self, id: EnvironmentRef, name: AtomId) -> bool {
        match self.environment_metadata(id) {
            Some(EnvironmentMetadata::Global { var_names, .. }) => var_names.contains(&name),
            _ => false,
        }
    }

    pub fn global_add_var_name(&mut self, id: EnvironmentRef, name: AtomId) -> bool {
        let Some(EnvironmentMetadata::Global { var_names, .. }) = self.environment_metadata_mut(id)
        else {
            return false;
        };
        var_names.insert(name)
    }

    #[inline]
    pub fn module_record(&self, key: &ModuleKey) -> Option<&ModuleRecord> {
        self.modules.get(key)
    }

    #[inline]
    pub fn install_module_record(&mut self, record: ModuleRecord) -> Option<ModuleRecord> {
        self.modules.insert(record.key().clone(), record)
    }

    pub fn set_module_record_environment(
        &mut self,
        key: &ModuleKey,
        environment: Option<EnvironmentRef>,
    ) -> bool {
        let Some(record) = self.modules.get_mut(key) else {
            return false;
        };
        record.set_environment(environment);
        true
    }

    pub fn set_module_record_code(&mut self, key: &ModuleKey, code: Option<CodeRef>) -> bool {
        let Some(record) = self.modules.get_mut(key) else {
            return false;
        };
        record.set_code(code);
        true
    }

    pub fn set_module_record_namespace(
        &mut self,
        key: &ModuleKey,
        namespace: Option<ObjectRef>,
    ) -> bool {
        let Some(record) = self.modules.get_mut(key) else {
            return false;
        };
        record.set_namespace(namespace);
        true
    }

    pub fn set_module_record_import_meta_object(
        &mut self,
        key: &ModuleKey,
        import_meta_object: Option<ObjectRef>,
    ) -> bool {
        let Some(record) = self.modules.get_mut(key) else {
            return false;
        };
        record.set_import_meta_object(import_meta_object);
        true
    }

    pub fn set_module_record_import_meta_properties(
        &mut self,
        key: &ModuleKey,
        import_meta_properties: ImportMetaProperties,
    ) -> bool {
        let Some(record) = self.modules.get_mut(key) else {
            return false;
        };
        record.set_import_meta_properties(import_meta_properties);
        true
    }

    pub fn set_module_record_resolved_exports(
        &mut self,
        key: &ModuleKey,
        resolved_exports: Vec<ModuleResolvedExport>,
    ) -> bool {
        let Some(record) = self.modules.get_mut(key) else {
            return false;
        };
        record.set_resolved_exports(resolved_exports);
        true
    }

    pub fn set_module_record_status(&mut self, key: &ModuleKey, status: ModuleStatus) -> bool {
        let Some(record) = self.modules.get_mut(key) else {
            return false;
        };
        record.set_status(status);
        true
    }

    pub fn set_module_record_evaluation_error(
        &mut self,
        key: &ModuleKey,
        evaluation_error: Option<Value>,
    ) -> bool {
        let Some(record) = self.modules.get_mut(key) else {
            return false;
        };
        record.set_evaluation_error(evaluation_error);
        true
    }

    pub fn set_module_record_dfs_state(
        &mut self,
        key: &ModuleKey,
        dfs_index: Option<u32>,
        dfs_ancestor_index: Option<u32>,
    ) -> bool {
        let Some(record) = self.modules.get_mut(key) else {
            return false;
        };
        record.set_dfs_state(dfs_index, dfs_ancestor_index);
        true
    }

    pub fn set_module_requested_key(
        &mut self,
        key: &ModuleKey,
        request_index: u32,
        resolved_key: Option<ModuleKey>,
    ) -> bool {
        let Some(record) = self.modules.get_mut(key) else {
            return false;
        };
        record.set_requested_module_resolved_key(request_index, resolved_key)
    }

    pub fn module_key_for_environment(&self, environment: EnvironmentRef) -> Option<ModuleKey> {
        self.modules.iter().find_map(|(key, record)| {
            (record.environment() == Some(environment)).then(|| key.clone())
        })
    }

    #[inline]
    pub fn realm_refs(&self) -> &[RealmRef] {
        &self.realms
    }

    #[inline]
    pub const fn default_realm_id(&self) -> Option<RealmRef> {
        self.default_realm
    }

    pub fn default_realm(&self) -> Option<RealmRecord> {
        self.default_realm.and_then(|realm| self.realm(realm))
    }

    #[inline]
    pub fn execution_contexts(&self) -> &[ExecutionContext] {
        &self.execution_contexts
    }

    #[inline]
    pub fn current_execution_context(&self) -> Option<ExecutionContext> {
        self.execution_contexts.last().copied()
    }

    pub fn push_execution_context(&mut self, context: ExecutionContext) {
        self.execution_contexts.push(context);
    }

    pub fn keep_weak_target_alive(&mut self, target: WeakHeapRef) {
        if !self.kept_objects.contains(&target) {
            self.kept_objects.push(target);
        }
    }

    pub fn clear_kept_objects(&mut self) {
        self.kept_objects.clear();
    }

    pub fn set_current_execution_context_this_state(&mut self, this_state: ThisState) -> bool {
        let Some(context) = self.execution_contexts.last_mut() else {
            return false;
        };
        *context = context.with_this_state(this_state);
        true
    }

    pub fn set_execution_context_this_state_for_lexical_env(
        &mut self,
        lexical_env: EnvironmentRef,
        this_state: ThisState,
    ) -> bool {
        let Some(context) = self
            .execution_contexts
            .iter_mut()
            .rev()
            .find(|context| context.lexical_env() == lexical_env)
        else {
            return false;
        };
        *context = context.with_this_state(this_state);
        true
    }

    pub fn push_script_context(
        &mut self,
        realm: RealmRef,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) {
        self.push_execution_context(ExecutionContext::script(realm, lexical_env, variable_env));
    }

    pub fn push_module_context(
        &mut self,
        realm: RealmRef,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) {
        self.push_execution_context(ExecutionContext::module(realm, lexical_env, variable_env));
    }

    pub fn push_builtin_context(
        &mut self,
        realm: RealmRef,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) {
        self.push_execution_context(ExecutionContext::builtin(realm, lexical_env, variable_env));
    }

    pub fn push_eval_context(
        &mut self,
        realm: RealmRef,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) {
        self.push_execution_context(ExecutionContext::eval(realm, lexical_env, variable_env));
    }

    pub fn push_job_context(
        &mut self,
        realm: RealmRef,
        executable: ExecutableId,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) {
        self.push_execution_context(ExecutionContext::job(
            realm,
            executable,
            lexical_env,
            variable_env,
        ));
    }

    pub fn push_bytecode_context(
        &mut self,
        realm: RealmRef,
        code: CodeRef,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) {
        self.push_execution_context(ExecutionContext::bytecode(
            realm,
            code,
            lexical_env,
            variable_env,
        ));
    }

    pub fn pop_execution_context(&mut self) -> Option<ExecutionContext> {
        self.execution_contexts.pop()
    }

    pub fn force_collect(&mut self) -> PrimitiveCollectionReport {
        let snapshot = AgentCollectionSnapshot::from_agent(self);
        let report = self.heap.force_collect_tracing(&self.roots, &snapshot);
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
            .and_then(|data| data.realm());
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

    /// Allocates the default realm shell used to bootstrap the runtime.
    ///
    /// # Panics
    /// Panics if the bootstrap global environment allocation fails unexpectedly.
    pub fn create_default_realm_shell(&mut self, lifetime: AllocationLifetime) -> RealmRef {
        let global_layout = self.alloc_environment_layout(EnvironmentLayout::empty(
            EnvironmentLayoutKind::Global,
            true,
        ));
        let (global_object, root_shape) = self.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let root_shape = objects.root_shape(&mut mutator, None, lifetime);
            let global_object = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape),
                lifetime,
            );
            (global_object, root_shape)
        });
        let global_env = self
            .alloc_global_environment(None, global_layout, global_object, lifetime)
            .expect("default realm global environment should allocate");
        let realm = self.heap.mutator().alloc_realm(
            RuntimeRealmRecord::new(
                Some(global_object),
                Some(global_env),
                None,
                Some(root_shape),
            ),
            lifetime,
        );

        self.store_realm_metadata(
            realm,
            RealmMetadata {
                intrinsics: Intrinsics::new(),
                bootstrap_state: RealmBootstrapState::new(),
                is_default: self.default_realm.is_none(),
            },
        );
        if !self.realms.contains(&realm) {
            self.realms.push(realm);
        }
        if self.default_realm.is_none() {
            self.default_realm = Some(realm);
        }
        debug_assert_eq!(
            self.realm(realm),
            Some(RealmRecord {
                id: realm,
                global_object,
                global_env,
                bootstrap_code: None,
                root_shape: Some(root_shape),
                intrinsics: Intrinsics::new(),
                bootstrap_state: RealmBootstrapState::new(),
                is_default: self.default_realm == Some(realm),
            })
        );
        realm
    }

    pub fn realm(&self, realm: RealmRef) -> Option<RealmRecord> {
        let record = self.heap.view().realm(realm)?;
        let metadata = self.realm_metadata(realm)?;
        Some(RealmRecord {
            id: realm,
            global_object: record.global_object()?,
            global_env: record.global_env()?,
            bootstrap_code: record.bootstrap_code(),
            root_shape: record.root_shape(),
            intrinsics: metadata.intrinsics,
            bootstrap_state: metadata.bootstrap_state,
            is_default: metadata.is_default,
        })
    }

    pub fn set_realm_intrinsics(&mut self, realm: RealmRef, intrinsics: Intrinsics) -> bool {
        let Some(metadata) = self.realm_metadata_mut(realm) else {
            return false;
        };
        metadata.intrinsics = intrinsics;
        true
    }

    pub fn realm_bootstrap_state(&self, realm: RealmRef) -> Option<RealmBootstrapState> {
        Some(self.realm_metadata(realm)?.bootstrap_state)
    }

    pub fn set_realm_bootstrap_state(
        &mut self,
        realm: RealmRef,
        bootstrap_state: RealmBootstrapState,
    ) -> bool {
        let Some(metadata) = self.realm_metadata_mut(realm) else {
            return false;
        };
        metadata.bootstrap_state = bootstrap_state;
        true
    }

    pub fn mark_realm_spec_bootstrapped(&mut self, realm: RealmRef) -> bool {
        let Some(metadata) = self.realm_metadata_mut(realm) else {
            return false;
        };
        metadata.bootstrap_state = metadata.bootstrap_state.with_spec_ready(true);
        true
    }

    pub fn mark_realm_embedding_bootstrapped(&mut self, realm: RealmRef) -> bool {
        let Some(metadata) = self.realm_metadata_mut(realm) else {
            return false;
        };
        metadata.bootstrap_state = metadata
            .bootstrap_state
            .with_spec_ready(true)
            .with_embedding_ready(true);
        true
    }

    #[allow(clippy::too_many_arguments)]
    fn alloc_environment_record(
        &mut self,
        outer: Option<EnvironmentRef>,
        layout: Option<EnvironmentLayoutId>,
        function_object: Option<ObjectRef>,
        this_value: Value,
        new_target: Option<ObjectRef>,
        home_object: Option<ObjectRef>,
        metadata: EnvironmentMetadata,
        lifetime: AllocationLifetime,
    ) -> EnvironmentRef {
        let layout = layout.and_then(|id| self.environment_layout(id).cloned());
        let env = {
            let mut mutator = self.heap.mutator();
            let slots = layout.as_ref().and_then(|layout| {
                Self::alloc_environment_slots_for_layout(&mut mutator, layout, lifetime)
            });
            mutator.alloc_environment(
                RuntimeEnvironmentRecord::new(
                    outer,
                    slots,
                    function_object,
                    this_value,
                    new_target,
                    home_object,
                ),
                lifetime,
            )
        };
        self.store_environment_metadata(env, metadata);
        env
    }

    fn alloc_environment_slots_for_layout(
        mutator: &mut lyng_js_gc::PrimitiveMutator<'_>,
        layout: &EnvironmentLayout,
        lifetime: AllocationLifetime,
    ) -> Option<EnvironmentSlotsRef> {
        let slot_count = usize::try_from(layout.slot_count())
            .expect("environment layout slot count must fit into usize");
        if slot_count == 0 {
            return None;
        }

        let slots = mutator.alloc_environment_slots(slot_count, Value::undefined(), lifetime);
        for (index, binding) in layout.bindings().iter().enumerate() {
            if binding.flags().needs_tdz() || !binding.flags().is_mutable() {
                let index = u32::try_from(index).expect("environment slot index must fit into u32");
                assert!(mutator.init_store_value(
                    ValueStoreTarget::EnvironmentSlot(slots, index),
                    Value::uninitialized_lexical(),
                ));
            }
        }
        Some(slots)
    }

    fn store_environment_metadata(&mut self, env: EnvironmentRef, metadata: EnvironmentMetadata) {
        let index = environment_index(env);
        if self.environment_metadata.len() <= index {
            self.environment_metadata.resize_with(index + 1, || None);
        }
        self.environment_metadata[index] = Some(metadata);
    }

    fn seed_phase5_symbol_state(&mut self, lifetime: AllocationLifetime) {
        for id in WellKnownSymbolId::ALL {
            let _ = self.ensure_well_known_symbol(id, lifetime);
        }
    }

    fn alloc_string_for_atom(&mut self, atom: AtomId, lifetime: AllocationLifetime) -> StringRef {
        if let Some(text) = self.atoms.get(atom) {
            let text = text.to_owned();
            if text.chars().all(|ch| u32::from(ch) <= u32::from(u8::MAX)) {
                let bytes: Vec<u8> = text
                    .chars()
                    .map(|ch| {
                        u8::try_from(u32::from(ch)).expect("Latin-1 code point must fit into u8")
                    })
                    .collect();
                return self.heap.mutator().alloc_string(
                    StringEncoding::Latin1,
                    u32::try_from(bytes.len()).expect("Latin-1 string length must fit into u32"),
                    &bytes,
                    Some(atom),
                    lifetime,
                );
            }

            let code_units: Vec<u16> = text.encode_utf16().collect();
            let mut bytes = Vec::with_capacity(code_units.len() * 2);
            for unit in &code_units {
                bytes.extend_from_slice(&unit.to_le_bytes());
            }
            return self.heap.mutator().alloc_string(
                StringEncoding::Utf16,
                u32::try_from(code_units.len()).expect("UTF-16 string length must fit into u32"),
                &bytes,
                Some(atom),
                lifetime,
            );
        }

        let code_units = self
            .atoms
            .get_utf16(atom)
            .expect("atom should resolve to UTF-8 or UTF-16 storage")
            .to_vec();
        let mut bytes = Vec::with_capacity(code_units.len() * 2);
        for unit in &code_units {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        self.heap.mutator().alloc_string(
            StringEncoding::Utf16,
            u32::try_from(code_units.len()).expect("UTF-16 string length must fit into u32"),
            &bytes,
            Some(atom),
            lifetime,
        )
    }

    fn environment_metadata(&self, env: EnvironmentRef) -> Option<&EnvironmentMetadata> {
        self.environment_metadata
            .get(environment_index(env))?
            .as_ref()
    }

    fn environment_metadata_mut(
        &mut self,
        env: EnvironmentRef,
    ) -> Option<&mut EnvironmentMetadata> {
        self.environment_metadata
            .get_mut(environment_index(env))?
            .as_mut()
    }

    fn store_realm_metadata(&mut self, realm: RealmRef, metadata: RealmMetadata) {
        let index = realm_index(realm);
        if self.realm_metadata.len() <= index {
            self.realm_metadata.resize_with(index + 1, || None);
        }
        self.realm_metadata[index] = Some(metadata);
    }

    fn realm_metadata(&self, realm: RealmRef) -> Option<&RealmMetadata> {
        self.realm_metadata.get(realm_index(realm))?.as_ref()
    }

    fn realm_metadata_mut(&mut self, realm: RealmRef) -> Option<&mut RealmMetadata> {
        self.realm_metadata.get_mut(realm_index(realm))?.as_mut()
    }

    /// Enqueues one runtime job on this agent.
    ///
    /// # Panics
    /// Panics if the monotonic job id overflows the supported non-zero `u32` range.
    pub fn enqueue_job(
        &mut self,
        kind: HostJobKind,
        executable: ExecutableId,
        realm: Option<RealmRef>,
        debug_name: Option<String>,
    ) -> RuntimeJob {
        self.enqueue_job_with_payload(
            kind,
            executable,
            RuntimeJobPayload::Executable,
            realm,
            debug_name,
        )
    }

    pub fn enqueue_job_with_payload(
        &mut self,
        kind: HostJobKind,
        executable: ExecutableId,
        payload: RuntimeJobPayload,
        realm: Option<RealmRef>,
        debug_name: Option<String>,
    ) -> RuntimeJob {
        let raw_id = self.next_job_id.max(1);
        self.next_job_id = raw_id
            .checked_add(1)
            .expect("runtime job id overflowed supported u32 range");
        let id = JobId::from_raw(raw_id).expect("runtime job id must stay non-zero");
        let job = RuntimeJob {
            id,
            kind,
            executable,
            payload,
            realm,
            debug_name,
        };
        self.job_queues.enqueue(job.clone());
        job
    }

    pub fn dequeue_job(&mut self, kind: JobQueueKind) -> Option<RuntimeJob> {
        self.job_queues.dequeue(kind)
    }

    pub fn queued_jobs(&self, kind: JobQueueKind) -> Vec<RuntimeJob> {
        self.job_queues.snapshot(kind)
    }

    #[inline]
    pub fn queued_job_count(&self, kind: JobQueueKind) -> usize {
        self.job_queues.len(kind)
    }

    #[inline]
    pub fn total_queued_jobs(&self) -> usize {
        self.job_queues.total_len()
    }

    pub fn alloc_promise(&mut self, object: ObjectRef, realm: RealmRef) -> PromiseId {
        self.promise_tables.alloc_promise(object, realm)
    }

    pub fn promise_id_for_object(&self, object: ObjectRef) -> Option<PromiseId> {
        self.promise_tables.promise_id_for_object(object)
    }

    pub fn promise_record(&self, object: ObjectRef) -> Option<&super::PromiseRecord> {
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
        reaction: super::PromiseReactionId,
    ) -> bool {
        self.promise_tables
            .push_promise_reaction(object, kind, reaction)
    }

    pub fn take_promise_reactions(
        &mut self,
        object: ObjectRef,
        kind: PromiseReactionKind,
    ) -> Option<Vec<super::PromiseReactionId>> {
        self.promise_tables.take_promise_reactions(object, kind)
    }

    pub fn alloc_promise_reaction(
        &mut self,
        reaction: PromiseReactionRecord,
    ) -> super::PromiseReactionId {
        self.promise_tables.alloc_reaction(reaction)
    }

    pub fn promise_reaction(&self, id: super::PromiseReactionId) -> Option<PromiseReactionRecord> {
        self.promise_tables.reaction(id)
    }

    pub fn alloc_promise_capability(&mut self) -> PromiseCapabilityId {
        self.promise_tables.alloc_capability()
    }

    pub fn promise_capability(
        &self,
        id: PromiseCapabilityId,
    ) -> Option<super::PromiseCapabilityRecord> {
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
    ) -> super::PromiseResolvingFunctionId {
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
    ) -> super::PromiseFinallyFunctionId {
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
        kind: super::PromiseCombinatorKind,
        capability: PromiseCapabilityId,
    ) -> super::PromiseCombinatorId {
        self.promise_tables.alloc_combinator(kind, capability)
    }

    pub fn promise_combinator(
        &self,
        id: super::PromiseCombinatorId,
    ) -> Option<&super::PromiseCombinatorRecord> {
        self.promise_tables.combinator(id)
    }

    pub fn push_promise_combinator_placeholder(
        &mut self,
        id: super::PromiseCombinatorId,
    ) -> Option<usize> {
        self.promise_tables.combinator_push_placeholder(id)
    }

    pub fn set_promise_combinator_value(
        &mut self,
        id: super::PromiseCombinatorId,
        index: usize,
        value: Value,
    ) -> bool {
        self.promise_tables.combinator_set_value(id, index, value)
    }

    pub fn promise_combinator_already_called(
        &self,
        id: super::PromiseCombinatorId,
        index: usize,
    ) -> Option<bool> {
        self.promise_tables.combinator_already_called(id, index)
    }

    pub fn set_promise_combinator_already_called(
        &mut self,
        id: super::PromiseCombinatorId,
        index: usize,
        already_called: bool,
    ) -> bool {
        self.promise_tables
            .combinator_set_already_called(id, index, already_called)
    }

    pub fn decrement_promise_combinator_remaining(
        &mut self,
        id: super::PromiseCombinatorId,
    ) -> Option<usize> {
        self.promise_tables.combinator_decrement_remaining(id)
    }

    pub fn alloc_promise_combinator_element(
        &mut self,
        object: ObjectRef,
        record: super::PromiseCombinatorElementRecord,
    ) -> super::PromiseCombinatorElementId {
        self.promise_tables.alloc_combinator_element(object, record)
    }

    pub fn promise_combinator_element(
        &self,
        object: ObjectRef,
    ) -> Option<super::PromiseCombinatorElementRecord> {
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
    ) -> Option<super::AsyncDisposalOperationRecord> {
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

    pub fn phase6_accounting(&self) -> AgentPhase6Accounting {
        let heap = self.heap.accounting();
        let iterator_records = Default::default();
        let regexp_payloads = {
            let accounting = self.objects.regexp_payload_accounting(self.heap.view());
            crate::RuntimeDomainAccounting {
                records: accounting.records,
                metadata_bytes: accounting.metadata_bytes,
                payload_bytes: accounting.payload_bytes,
                live_bytes: accounting.live_bytes,
            }
        };
        let module_caches = Default::default();
        let promise_jobs = self.job_queues.promise_job_accounting();
        AgentPhase6Accounting {
            heap,
            iterator_records,
            regexp_payloads,
            module_caches,
            promise_jobs,
            live_bytes: total_live_bytes(
                heap,
                iterator_records,
                regexp_payloads,
                module_caches,
                promise_jobs,
                Default::default(),
            ),
        }
    }
}
