use super::AgentId;
use lyng_js_host::{HostSharedBufferId, HostThreadId, WaitLocation};
use lyng_js_types::{BackingStoreRef, ObjectRef};
use std::collections::{HashMap, VecDeque};
use std::num::NonZeroU64;

/// One agent parked on a shared-memory wait location.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParkedAgentRecord {
    agent: AgentId,
    thread_id: Option<HostThreadId>,
    allow_async: bool,
}

impl ParkedAgentRecord {
    #[inline]
    pub const fn new(agent: AgentId, thread_id: Option<HostThreadId>, allow_async: bool) -> Self {
        Self {
            agent,
            thread_id,
            allow_async,
        }
    }

    #[inline]
    pub const fn agent(self) -> AgentId {
        self.agent
    }

    #[inline]
    pub const fn thread_id(self) -> Option<HostThreadId> {
        self.thread_id
    }

    #[inline]
    pub const fn allow_async(self) -> bool {
        self.allow_async
    }
}

/// One async waiter parked on a shared-memory wait location.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AsyncWaiterRecord {
    agent: AgentId,
    promise: ObjectRef,
}

impl AsyncWaiterRecord {
    #[inline]
    pub const fn new(agent: AgentId, promise: ObjectRef) -> Self {
        Self { agent, promise }
    }

    #[inline]
    pub const fn agent(self) -> AgentId {
        self.agent
    }

    #[inline]
    pub const fn promise(self) -> ObjectRef {
        self.promise
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WaiterToken(NonZeroU64);

impl WaiterToken {
    #[inline]
    pub const fn new(raw: NonZeroU64) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaiterKind {
    Blocking(ParkedAgentRecord),
    Async(AsyncWaiterRecord),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaiterRecord {
    token: WaiterToken,
    kind: WaiterKind,
}

impl WaiterRecord {
    #[inline]
    pub const fn new(token: WaiterToken, kind: WaiterKind) -> Self {
        Self { token, kind }
    }

    #[inline]
    pub const fn token(self) -> WaiterToken {
        self.token
    }

    #[inline]
    pub const fn kind(self) -> WaiterKind {
        self.kind
    }
}

/// Cluster-owned shared backing-store visibility record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedBackingStoreRecord {
    pub(crate) host_shared_buffer: Option<HostSharedBufferId>,
    pub(crate) backing_store: BackingStoreRef,
    pub(crate) byte_length: usize,
    pub(crate) visible_to: Vec<AgentId>,
}

impl SharedBackingStoreRecord {
    #[inline]
    pub const fn host_shared_buffer(&self) -> Option<HostSharedBufferId> {
        self.host_shared_buffer
    }

    #[inline]
    pub const fn backing_store(&self) -> BackingStoreRef {
        self.backing_store
    }

    #[inline]
    pub const fn byte_length(&self) -> usize {
        self.byte_length
    }

    #[inline]
    pub fn visible_to(&self) -> &[AgentId] {
        &self.visible_to
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WaitQueueKey {
    pub(crate) backing_store: BackingStoreRef,
    pub(crate) byte_offset: u64,
}

impl From<WaitLocation> for WaitQueueKey {
    fn from(value: WaitLocation) -> Self {
        Self {
            backing_store: value.backing_store,
            byte_offset: value.byte_offset,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SharedMemoryRuntime {
    pub(crate) shared_backing_stores: HashMap<BackingStoreRef, SharedBackingStoreRecord>,
    pub(crate) shared_backing_store_handles: HashMap<HostSharedBufferId, BackingStoreRef>,
    pub(crate) wait_queues: HashMap<WaitQueueKey, VecDeque<WaiterRecord>>,
    next_waiter_token: u64,
}

impl SharedMemoryRuntime {
    pub(crate) fn register_shared_backing_store(
        &mut self,
        owner: AgentId,
        backing_store: BackingStoreRef,
        byte_length: usize,
    ) -> bool {
        self.shared_backing_stores
            .insert(
                backing_store,
                SharedBackingStoreRecord {
                    host_shared_buffer: None,
                    backing_store,
                    byte_length,
                    visible_to: vec![owner],
                },
            )
            .is_none()
    }

    pub(crate) fn cache_shared_backing_store_handle(
        &mut self,
        backing_store: BackingStoreRef,
        shared_buffer: HostSharedBufferId,
    ) -> bool {
        let Some(record) = self.shared_backing_stores.get_mut(&backing_store) else {
            return false;
        };
        if let Some(cached) = record.host_shared_buffer {
            return cached == shared_buffer;
        }
        if let Some(existing) = self.shared_backing_store_handles.get(&shared_buffer) {
            return *existing == backing_store;
        }
        record.host_shared_buffer = Some(shared_buffer);
        self.shared_backing_store_handles
            .insert(shared_buffer, backing_store);
        true
    }

    pub(crate) fn share_shared_backing_store(
        &mut self,
        backing_store: BackingStoreRef,
        target: AgentId,
    ) -> bool {
        let Some(record) = self.shared_backing_stores.get_mut(&backing_store) else {
            return false;
        };
        if !record.visible_to.contains(&target) {
            record.visible_to.push(target);
        }
        true
    }

    pub(crate) fn update_shared_backing_store_byte_length(
        &mut self,
        backing_store: BackingStoreRef,
        byte_length: usize,
    ) -> bool {
        let Some(record) = self.shared_backing_stores.get_mut(&backing_store) else {
            return false;
        };
        record.byte_length = byte_length;
        true
    }

    pub(crate) fn shared_backing_store(
        &self,
        backing_store: BackingStoreRef,
    ) -> Option<&SharedBackingStoreRecord> {
        self.shared_backing_stores.get(&backing_store)
    }

    pub(crate) fn shared_backing_store_by_host_id(
        &self,
        shared_buffer: HostSharedBufferId,
    ) -> Option<&SharedBackingStoreRecord> {
        let backing_store = *self.shared_backing_store_handles.get(&shared_buffer)?;
        self.shared_backing_store(backing_store)
    }

    pub(crate) fn park_agent(
        &mut self,
        location: WaitLocation,
        parked: ParkedAgentRecord,
    ) -> WaiterToken {
        self.push_waiter(location, WaiterKind::Blocking(parked))
    }

    pub(crate) fn park_async_waiter(
        &mut self,
        location: WaitLocation,
        parked: AsyncWaiterRecord,
    ) -> WaiterToken {
        self.push_waiter(location, WaiterKind::Async(parked))
    }

    pub(crate) fn remove_waiter(&mut self, location: WaitLocation, token: WaiterToken) -> bool {
        let key = WaitQueueKey::from(location);
        let Some(queue) = self.wait_queues.get_mut(&key) else {
            return false;
        };
        let original_len = queue.len();
        queue.retain(|record| record.token() != token);
        let removed = queue.len() != original_len;
        if queue.is_empty() {
            self.wait_queues.remove(&key);
        }
        removed
    }

    pub(crate) fn waiter_count(&self, location: WaitLocation) -> usize {
        self.wait_queues
            .get(&location.into())
            .map_or(0, VecDeque::len)
    }

    pub(crate) fn parked_agents(&self, location: WaitLocation) -> Vec<ParkedAgentRecord> {
        self.wait_queues
            .get(&location.into())
            .map(|queue| {
                queue
                    .iter()
                    .filter_map(|record| match record.kind() {
                        WaiterKind::Blocking(parked) => Some(parked),
                        WaiterKind::Async(_) => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub(crate) fn wake_waiters(
        &mut self,
        location: WaitLocation,
        max_count: u32,
    ) -> Vec<WaiterRecord> {
        let key = WaitQueueKey::from(location);
        let Some(queue) = self.wait_queues.get_mut(&key) else {
            return Vec::new();
        };
        let mut woken = Vec::new();
        for _ in 0..max_count {
            let Some(waiter) = queue.pop_front() else {
                break;
            };
            woken.push(waiter);
        }
        if queue.is_empty() {
            self.wait_queues.remove(&key);
        }
        woken
    }

    fn push_waiter(&mut self, location: WaitLocation, kind: WaiterKind) -> WaiterToken {
        let next_raw = self.next_waiter_token.saturating_add(1).max(1);
        self.next_waiter_token = next_raw;
        let token =
            WaiterToken::new(NonZeroU64::new(next_raw).expect("waiter token must stay non-zero"));
        self.wait_queues
            .entry(location.into())
            .or_default()
            .push_back(WaiterRecord::new(token, kind));
        token
    }
}
