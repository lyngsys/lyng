use super::{
    agent_index, Agent, AgentId, BackingStoreRuntime, ClusterBackingStoreHandle,
    ClusterSharedMemoryHandle, HostAgentId, ParkedAgentRecord, RuntimeDomainAccounting,
    SharedBackingStoreRecord, SharedMemoryRuntime, WaitLocation, WaiterKind, WaiterRecord,
};
use lyng_js_host::HostSharedBufferId;
use lyng_js_types::BackingStoreRef;
use std::mem::size_of;
use std::{cell::RefCell, rc::Rc};

/// Cluster-owned shared coordination plus the agent table.
pub struct AgentCluster {
    root_agent: AgentId,
    agents: Vec<Option<Agent>>,
    next_agent_id: u32,
    backing_stores: Rc<RefCell<BackingStoreRuntime>>,
    shared_memory: Rc<RefCell<SharedMemoryRuntime>>,
}

impl Default for AgentCluster {
    fn default() -> Self {
        let mut cluster = Self {
            root_agent: AgentId::from_raw(1).expect("root agent id must stay non-zero"),
            agents: Vec::new(),
            next_agent_id: 1,
            backing_stores: Rc::new(RefCell::new(BackingStoreRuntime::new())),
            shared_memory: Rc::new(RefCell::new(SharedMemoryRuntime::default())),
        };
        let root = cluster.add_agent(None, Some("root-agent".into()));
        cluster.root_agent = root;
        cluster
    }
}

impl AgentCluster {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds one agent record to the cluster.
    ///
    /// # Panics
    /// Panics if the agent table grows beyond the supported non-zero `u32` id range.
    pub fn add_agent(
        &mut self,
        host_id: Option<HostAgentId>,
        debug_name: Option<String>,
    ) -> AgentId {
        let raw_id = self.next_agent_id.max(1);
        self.next_agent_id = raw_id
            .checked_add(1)
            .expect("agent id overflowed supported u32 range");
        let id = AgentId::from_raw(raw_id).expect("agent id must stay non-zero");
        let index = agent_index(id);
        if self.agents.len() <= index {
            self.agents.resize_with(index + 1, || None);
        }
        let backing_store_handle = self.backing_store_handle();
        let shared_memory_handle = self.shared_memory_handle();
        self.agents[index] = Some(Agent::new(
            id,
            host_id,
            debug_name,
            backing_store_handle,
            shared_memory_handle,
        ));
        id
    }

    #[inline]
    pub const fn root_agent_id(&self) -> AgentId {
        self.root_agent
    }

    /// Returns the root agent for this cluster.
    ///
    /// # Panics
    /// Panics if the cluster has lost its root agent record.
    pub fn root_agent(&self) -> &Agent {
        self.agent(self.root_agent)
            .expect("root agent must exist in the cluster")
    }

    /// Returns the mutable root agent for this cluster.
    ///
    /// # Panics
    /// Panics if the cluster has lost its root agent record.
    pub fn root_agent_mut(&mut self) -> &mut Agent {
        self.agent_mut(self.root_agent)
            .expect("root agent must exist in the cluster")
    }

    #[inline]
    pub fn agent_count(&self) -> usize {
        self.agents.iter().flatten().count()
    }

    pub fn agent_ids(&self) -> Vec<AgentId> {
        self.agents
            .iter()
            .flatten()
            .map(Agent::id)
            .collect::<Vec<_>>()
    }

    pub fn agent(&self, id: AgentId) -> Option<&Agent> {
        self.agents.get(agent_index(id))?.as_ref()
    }

    pub fn agent_mut(&mut self, id: AgentId) -> Option<&mut Agent> {
        self.agents.get_mut(agent_index(id))?.as_mut()
    }

    #[cfg(test)]
    pub(crate) fn set_next_agent_id_for_test(&mut self, next_agent_id: u32) {
        self.next_agent_id = next_agent_id;
    }

    pub fn register_shared_backing_store(
        &mut self,
        owner: AgentId,
        byte_length: usize,
    ) -> Option<BackingStoreRef> {
        if self.agent(owner).is_none() {
            return None;
        }
        let Some(backing_store) = self
            .backing_stores
            .borrow_mut()
            .allocate_shared(byte_length)
        else {
            return None;
        };
        if self
            .shared_memory
            .borrow_mut()
            .register_shared_backing_store(owner, backing_store, byte_length)
        {
            Some(backing_store)
        } else {
            None
        }
    }

    pub fn cache_shared_backing_store_handle(
        &mut self,
        backing_store: BackingStoreRef,
        shared_buffer: HostSharedBufferId,
    ) -> bool {
        self.shared_memory
            .borrow_mut()
            .cache_shared_backing_store_handle(backing_store, shared_buffer)
    }

    pub fn share_shared_backing_store(
        &mut self,
        backing_store: BackingStoreRef,
        target: AgentId,
    ) -> bool {
        if self.agent(target).is_none() {
            return false;
        }
        self.shared_memory
            .borrow_mut()
            .share_shared_backing_store(backing_store, target)
    }

    pub fn shared_backing_store(
        &self,
        backing_store: BackingStoreRef,
    ) -> Option<SharedBackingStoreRecord> {
        self.shared_memory
            .borrow()
            .shared_backing_store(backing_store)
            .cloned()
    }

    pub fn shared_backing_store_by_host_id(
        &self,
        shared_buffer: HostSharedBufferId,
    ) -> Option<SharedBackingStoreRecord> {
        self.shared_memory
            .borrow()
            .shared_backing_store_by_host_id(shared_buffer)
            .cloned()
    }

    pub fn backing_store_accounting(&self) -> RuntimeDomainAccounting {
        let shared_metadata_bytes = self
            .shared_memory
            .borrow()
            .shared_backing_stores
            .values()
            .map(|record| {
                size_of::<SharedBackingStoreRecord>()
                    + size_of::<AgentId>() * record.visible_to.len()
            })
            .sum::<usize>();
        let mut accounting = self.backing_stores.borrow().accounting();
        accounting.metadata_bytes += shared_metadata_bytes;
        accounting.live_bytes += shared_metadata_bytes;
        accounting
    }

    pub fn park_agent(&mut self, location: WaitLocation, parked: ParkedAgentRecord) -> bool {
        if self.agent(parked.agent()).is_none() {
            return false;
        }
        self.shared_memory
            .borrow_mut()
            .park_agent(location, parked)
            .is_some()
    }

    pub fn waiter_count(&self, location: WaitLocation) -> usize {
        self.shared_memory.borrow().waiter_count(location)
    }

    pub fn parked_agents(&self, location: WaitLocation) -> Vec<ParkedAgentRecord> {
        self.shared_memory.borrow().parked_agents(location)
    }

    pub fn wake_waiters(&mut self, location: WaitLocation, max_count: u32) -> Vec<WaiterRecord> {
        self.shared_memory
            .borrow_mut()
            .wake_waiters(location, max_count)
    }

    pub fn unpark_agents(
        &mut self,
        location: WaitLocation,
        max_count: u32,
    ) -> Vec<ParkedAgentRecord> {
        self.wake_waiters(location, max_count)
            .into_iter()
            .filter_map(|waiter| match waiter.kind() {
                WaiterKind::Blocking(parked) => Some(parked),
                WaiterKind::Async(_) => None,
            })
            .collect()
    }

    fn backing_store_handle(&mut self) -> ClusterBackingStoreHandle {
        ClusterBackingStoreHandle::new(Rc::clone(&self.backing_stores))
    }

    fn shared_memory_handle(&mut self) -> ClusterSharedMemoryHandle {
        ClusterSharedMemoryHandle::new(Rc::clone(&self.shared_memory))
    }
}
