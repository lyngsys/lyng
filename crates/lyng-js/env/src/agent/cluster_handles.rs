use super::Agent;
use crate::{
    AgentId, AsyncWaiterRecord, BackingStoreRuntime, ParkedAgentRecord, SharedBackingStoreRecord,
    SharedMemoryRuntime, WaiterRecord, WaiterToken,
};
use lyng_js_host::{HostSharedBufferId, WaitLocation};
use lyng_js_types::BackingStoreRef;
use std::{cell::RefCell, rc::Rc};

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
    fn resize(&self, store: BackingStoreRef, byte_length: usize) -> bool {
        self.0.borrow_mut().resize(store, byte_length)
    }

    #[inline]
    fn grow_shared(&self, store: BackingStoreRef, byte_length: usize) -> bool {
        self.0.borrow_mut().grow_shared(store, byte_length)
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
pub(crate) struct ClusterSharedMemoryHandle(Rc<RefCell<SharedMemoryRuntime>>);

impl ClusterSharedMemoryHandle {
    #[inline]
    pub(crate) fn new(runtime: Rc<RefCell<SharedMemoryRuntime>>) -> Self {
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
        shared_buffer: HostSharedBufferId,
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
    fn update_shared_backing_store_byte_length(
        &self,
        backing_store: BackingStoreRef,
        byte_length: usize,
    ) -> bool {
        self.0
            .borrow_mut()
            .update_shared_backing_store_byte_length(backing_store, byte_length)
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

impl Agent {
    #[inline]
    pub fn backing_store_allocation_limit(&self) -> usize {
        crate::backing_store::MAX_BACKING_STORE_BYTE_LENGTH
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
    pub fn backing_store_resize(&mut self, store: BackingStoreRef, byte_length: usize) -> bool {
        self.backing_stores.resize(store, byte_length)
    }

    #[inline]
    pub fn grow_shared_backing_store(
        &mut self,
        store: BackingStoreRef,
        byte_length: usize,
    ) -> bool {
        if !self.backing_stores.grow_shared(store, byte_length) {
            return false;
        }
        self.shared_memory
            .update_shared_backing_store_byte_length(store, byte_length)
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
}
