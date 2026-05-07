use super::{
    AllocationLifetime, BigIntRef, CodeRef, CodeSlotsRef, EnvironmentRef, EnvironmentSlotsRef,
    FunctionPayloadRef, ObjectRef, ObjectSlotsRef, PrimitiveDomainStats, PrimitiveValueCellRef,
    RealmRef, ShapeId, SideAllocationClass, SideAllocationRef, SideAllocationStats, StringEncoding,
    StringRef, SuspendedExecutionRef, SuspendedRegistersRef, SymbolRef, Value,
    PRIMITIVE_SLOTS_PER_PAGE,
};
use std::array::from_fn;
use std::collections::BTreeMap;
use std::marker::PhantomData;

pub(super) trait ArenaHandle: Copy {
    fn from_raw(raw: u32) -> Option<Self>;
    fn get(self) -> u32;
}

impl ArenaHandle for StringRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for SymbolRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for BigIntRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for PrimitiveValueCellRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for ObjectRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for EnvironmentRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for CodeRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for RealmRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for SuspendedExecutionRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for ShapeId {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for ObjectSlotsRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for EnvironmentSlotsRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for CodeSlotsRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for FunctionPayloadRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

impl ArenaHandle for SuspendedRegistersRef {
    fn from_raw(raw: u32) -> Option<Self> {
        Self::from_raw(raw)
    }

    fn get(self) -> u32 {
        self.get()
    }
}

pub(super) struct SlotArena<Record, Handle> {
    pages: Vec<SlotPage<Record>>,
    pages_with_available_slots: usize,
    first_page_with_available_slot: Option<usize>,
    marker: PhantomData<Handle>,
}

impl<Record, Handle> Default for SlotArena<Record, Handle> {
    fn default() -> Self {
        Self {
            pages: Vec::new(),
            pages_with_available_slots: 0,
            first_page_with_available_slot: None,
            marker: PhantomData,
        }
    }
}

impl<Record: Copy, Handle: ArenaHandle> SlotArena<Record, Handle> {
    pub(super) const fn allocation_requires_growth(&self) -> bool {
        !self.pages.is_empty() && self.pages_with_available_slots == 0
    }

    pub(super) fn allocate(&mut self, record: Record, lifetime: AllocationLifetime) -> Handle {
        if let Some(page_index) = self.first_page_with_available_slot {
            let slot_index = self.pages[page_index]
                .allocate(record, lifetime)
                .expect("page availability hint must point at a page with a free slot");
            if !self.pages[page_index].has_available_slot() {
                self.pages_with_available_slots -= 1;
                self.first_page_with_available_slot =
                    self.find_available_page_after(page_index + 1);
            }
            return make_handle::<Handle>(page_index, slot_index);
        }

        let mut page = SlotPage::new();
        let slot_index = page
            .allocate(record, lifetime)
            .expect("fresh primitive page must accept its first record");
        let page_has_available_slot = page.has_available_slot();
        self.pages.push(page);
        let page_index = self.pages.len() - 1;
        if page_has_available_slot {
            self.pages_with_available_slots += 1;
            if self.first_page_with_available_slot.is_none() {
                self.first_page_with_available_slot = Some(page_index);
            }
        }
        make_handle::<Handle>(page_index, slot_index)
    }

    pub(super) fn get(&self, handle: Handle) -> Option<Record> {
        let (page_index, slot_index) = locate::<Handle>(handle)?;
        self.pages.get(page_index)?.get(slot_index)
    }

    pub(super) fn free(&mut self, handle: Handle) -> Option<Record> {
        let (page_index, slot_index) = locate::<Handle>(handle)?;
        let (was_available, is_available, record) = {
            let page = self.pages.get_mut(page_index)?;
            let was_available = page.has_available_slot();
            let record = page.free(slot_index)?;
            (was_available, page.has_available_slot(), record)
        };
        self.update_page_availability(page_index, was_available, is_available);
        Some(record)
    }

    pub(super) fn mark(&mut self, handle: Handle) -> bool {
        let Some((page_index, slot_index)) = locate::<Handle>(handle) else {
            return false;
        };
        match self.pages.get_mut(page_index) {
            Some(page) => page.mark(slot_index),
            None => false,
        }
    }

    pub(super) fn is_marked(&self, handle: Handle) -> bool {
        let Some((page_index, slot_index)) = locate::<Handle>(handle) else {
            return false;
        };
        self.pages
            .get(page_index)
            .is_some_and(|page| page.is_marked(slot_index))
    }

    pub(super) fn clear_marks(&mut self) {
        for page in &mut self.pages {
            page.clear_marks();
        }
    }

    pub(super) fn stats(&self, side_allocations: SideAllocationStats) -> PrimitiveDomainStats {
        let mut stats = PrimitiveDomainStats {
            pages: self.pages.len(),
            side_allocations,
            ..PrimitiveDomainStats::default()
        };

        for page in &self.pages {
            stats.occupied_slots += page.occupied;
            stats.reusable_slots += page.free_list.len();
            stats.marked_slots += page.marked_slots();
            stats.default_slots += page.default_slots();
            stats.long_lived_slots += page.long_lived_slots();
        }

        stats
    }

    pub(super) fn sweep(&mut self, mut reclaim: impl FnMut(Record)) -> usize {
        let mut reclaimed = 0;

        for page_index in 0..self.pages.len() {
            let (was_available, is_available, page_reclaimed) = {
                let page = &mut self.pages[page_index];
                let was_available = page.has_available_slot();
                let page_reclaimed = page.sweep(&mut reclaim);
                (was_available, page.has_available_slot(), page_reclaimed)
            };
            self.update_page_availability(page_index, was_available, is_available);
            reclaimed += page_reclaimed;
        }

        reclaimed
    }

    pub(super) fn update(&mut self, handle: Handle, mut update: impl FnMut(&mut Record)) -> bool {
        let Some((page_index, slot_index)) = locate::<Handle>(handle) else {
            return false;
        };

        match self.pages.get_mut(page_index) {
            Some(page) => page.update(slot_index, &mut update),
            None => false,
        }
    }

    fn find_available_page_after(&self, start: usize) -> Option<usize> {
        self.pages
            .iter()
            .enumerate()
            .skip(start)
            .find_map(|(page_index, page)| page.has_available_slot().then_some(page_index))
    }

    fn update_page_availability(
        &mut self,
        page_index: usize,
        was_available: bool,
        is_available: bool,
    ) {
        match (was_available, is_available) {
            (false, true) => {
                self.pages_with_available_slots += 1;
                if self
                    .first_page_with_available_slot
                    .is_none_or(|first_page| page_index < first_page)
                {
                    self.first_page_with_available_slot = Some(page_index);
                }
            }
            (true, false) => {
                self.pages_with_available_slots -= 1;
                if self.first_page_with_available_slot == Some(page_index) {
                    self.first_page_with_available_slot =
                        self.find_available_page_after(page_index + 1);
                }
            }
            _ => {}
        }

        debug_assert_eq!(
            self.pages_with_available_slots,
            self.pages
                .iter()
                .filter(|page| page.has_available_slot())
                .count(),
            "slot arena availability metadata must track page capacity exactly"
        );
        debug_assert_eq!(
            self.first_page_with_available_slot,
            self.pages
                .iter()
                .enumerate()
                .find_map(|(index, page)| page.has_available_slot().then_some(index)),
            "slot arena availability hint must target the first page with capacity"
        );
    }
}

struct SlotPage<Record> {
    slots: [Option<Record>; PRIMITIVE_SLOTS_PER_PAGE],
    marks: [bool; PRIMITIVE_SLOTS_PER_PAGE],
    lifetimes: [AllocationLifetime; PRIMITIVE_SLOTS_PER_PAGE],
    occupied: usize,
    next_uninitialized: usize,
    free_list: Vec<u16>,
}

impl<Record: Copy> SlotPage<Record> {
    fn new() -> Self {
        Self {
            slots: from_fn(|_| None),
            marks: [false; PRIMITIVE_SLOTS_PER_PAGE],
            lifetimes: [AllocationLifetime::Default; PRIMITIVE_SLOTS_PER_PAGE],
            occupied: 0,
            next_uninitialized: 0,
            free_list: Vec::new(),
        }
    }

    fn allocate(&mut self, record: Record, lifetime: AllocationLifetime) -> Option<usize> {
        let slot_index = if let Some(slot) = self.free_list.pop() {
            usize::from(slot)
        } else if self.next_uninitialized < PRIMITIVE_SLOTS_PER_PAGE {
            let slot_index = self.next_uninitialized;
            self.next_uninitialized += 1;
            slot_index
        } else {
            return None;
        };

        self.slots[slot_index] = Some(record);
        self.marks[slot_index] = false;
        self.lifetimes[slot_index] = lifetime;
        self.occupied += 1;
        Some(slot_index)
    }

    const fn has_available_slot(&self) -> bool {
        !self.free_list.is_empty() || self.next_uninitialized < PRIMITIVE_SLOTS_PER_PAGE
    }

    fn get(&self, slot_index: usize) -> Option<Record> {
        self.slots.get(slot_index).copied().flatten()
    }

    fn update(&mut self, slot_index: usize, update: &mut impl FnMut(&mut Record)) -> bool {
        match self.slots.get_mut(slot_index) {
            Some(Some(record)) => {
                update(record);
                true
            }
            _ => false,
        }
    }

    fn free(&mut self, slot_index: usize) -> Option<Record> {
        let record = self.slots.get_mut(slot_index)?.take()?;
        self.marks[slot_index] = false;
        self.lifetimes[slot_index] = AllocationLifetime::Default;
        self.occupied -= 1;
        self.free_list
            .push(u16::try_from(slot_index).expect("primitive page slot index must fit into u16"));
        Some(record)
    }

    fn mark(&mut self, slot_index: usize) -> bool {
        match self.slots.get(slot_index) {
            Some(Some(_)) => {
                let was_marked = self.marks[slot_index];
                self.marks[slot_index] = true;
                !was_marked
            }
            _ => false,
        }
    }

    #[inline]
    fn is_marked(&self, slot_index: usize) -> bool {
        self.slots.get(slot_index).is_some_and(Option::is_some) && self.marks[slot_index]
    }

    fn clear_marks(&mut self) {
        for slot_index in 0..self.next_uninitialized {
            if self.slots[slot_index].is_some() {
                self.marks[slot_index] = false;
            }
        }
    }

    fn sweep(&mut self, reclaim: &mut impl FnMut(Record)) -> usize {
        let mut reclaimed = 0;

        for slot_index in 0..self.next_uninitialized {
            match self.slots[slot_index] {
                Some(record) if self.marks[slot_index] => {
                    self.marks[slot_index] = false;
                }
                Some(record) => {
                    self.slots[slot_index] = None;
                    self.marks[slot_index] = false;
                    self.lifetimes[slot_index] = AllocationLifetime::Default;
                    self.occupied -= 1;
                    self.free_list.push(
                        u16::try_from(slot_index)
                            .expect("primitive page slot index must fit into u16"),
                    );
                    reclaim(record);
                    reclaimed += 1;
                }
                None => {}
            }
        }

        reclaimed
    }

    fn marked_slots(&self) -> usize {
        (0..self.next_uninitialized)
            .filter(|&slot_index| self.slots[slot_index].is_some() && self.marks[slot_index])
            .count()
    }

    fn default_slots(&self) -> usize {
        self.count_slots_with_lifetime(AllocationLifetime::Default)
    }

    fn long_lived_slots(&self) -> usize {
        self.count_slots_with_lifetime(AllocationLifetime::LongLived)
    }

    fn count_slots_with_lifetime(&self, lifetime: AllocationLifetime) -> usize {
        (0..self.next_uninitialized)
            .filter(|&slot_index| {
                self.slots[slot_index].is_some() && self.lifetimes[slot_index] == lifetime
            })
            .count()
    }
}

#[derive(Default)]
pub(super) struct SideAllocator {
    slots: Vec<SideAllocationSlot>,
    free_by_class: BTreeMap<SideAllocationClass, Vec<u32>>,
}

struct SideAllocationSlot {
    class: SideAllocationClass,
    lifetime: AllocationLifetime,
    payload_len: usize,
    occupied: bool,
    bytes: Box<[u8]>,
}

impl SideAllocator {
    pub(super) fn allocation_requires_growth(&self, class: SideAllocationClass) -> bool {
        !self.slots.is_empty()
            && self
                .free_by_class
                .get(&class)
                .is_none_or(std::vec::Vec::is_empty)
    }

    pub(super) fn allocate(
        &mut self,
        payload: &[u8],
        lifetime: AllocationLifetime,
    ) -> SideAllocationRef {
        let class = SideAllocationClass::for_payload_len(payload.len());
        let (slot_index, id) = self.reserve_slot(class, payload.len(), lifetime);
        self.slots[slot_index].bytes[..payload.len()].copy_from_slice(payload);
        id
    }

    pub(super) fn allocate_concat(
        &mut self,
        left: SideAllocationRef,
        left_len: usize,
        right: SideAllocationRef,
        right_len: usize,
        lifetime: AllocationLifetime,
    ) -> Option<SideAllocationRef> {
        let left_index = side_allocation_index(left)?;
        let right_index = side_allocation_index(right)?;
        if self.source_payload_len(left_index)? != left_len
            || self.source_payload_len(right_index)? != right_len
        {
            return None;
        }

        let payload_len = left_len.checked_add(right_len)?;
        let class = SideAllocationClass::for_payload_len(payload_len);
        let (slot_index, id) = self.reserve_slot(class, payload_len, lifetime);
        copy_between_side_slots(&mut self.slots, left_index, left_len, slot_index, 0);
        copy_between_side_slots(
            &mut self.slots,
            right_index,
            right_len,
            slot_index,
            left_len,
        );
        Some(id)
    }

    pub(super) fn allocate_utf16_concat(
        &mut self,
        left: SideAllocationRef,
        left_encoding: StringEncoding,
        left_code_unit_len: u32,
        right: SideAllocationRef,
        right_encoding: StringEncoding,
        right_code_unit_len: u32,
        lifetime: AllocationLifetime,
    ) -> Option<SideAllocationRef> {
        let left_index = side_allocation_index(left)?;
        let right_index = side_allocation_index(right)?;
        if self.source_payload_len(left_index)?
            != string_payload_len_for_encoding(left_encoding, left_code_unit_len)?
            || self.source_payload_len(right_index)?
                != string_payload_len_for_encoding(right_encoding, right_code_unit_len)?
        {
            return None;
        }

        let payload_len =
            utf16_payload_len_for_code_units(left_code_unit_len.checked_add(right_code_unit_len)?)?;
        let class = SideAllocationClass::for_payload_len(payload_len);
        let (slot_index, id) = self.reserve_slot(class, payload_len, lifetime);
        copy_string_payload_to_utf16_slot(
            &mut self.slots,
            left_index,
            left_encoding,
            left_code_unit_len,
            slot_index,
            0,
        );
        copy_string_payload_to_utf16_slot(
            &mut self.slots,
            right_index,
            right_encoding,
            right_code_unit_len,
            slot_index,
            usize::try_from(left_code_unit_len).ok()?.checked_mul(2)?,
        );
        Some(id)
    }

    pub(super) fn get(&self, id: SideAllocationRef) -> Option<&[u8]> {
        let slot = self.slots.get((id.get() - 1) as usize)?;
        if slot.occupied {
            Some(&slot.bytes[..slot.payload_len])
        } else {
            None
        }
    }

    pub(super) fn free(&mut self, id: SideAllocationRef) -> bool {
        let Some(slot) = self.slots.get_mut((id.get() - 1) as usize) else {
            return false;
        };
        if !slot.occupied {
            return false;
        }

        slot.occupied = false;
        slot.payload_len = 0;
        self.free_by_class
            .entry(slot.class)
            .or_default()
            .push(id.get());
        true
    }

    pub(super) fn stats(&self) -> SideAllocationStats {
        let mut stats = SideAllocationStats::default();

        for slot in &self.slots {
            stats.reserved_bytes += slot.class.slot_bytes();
            if slot.occupied {
                stats.live_allocations += 1;
                stats.live_payload_bytes += slot.payload_len;
                match slot.lifetime {
                    AllocationLifetime::Default => stats.default_allocations += 1,
                    AllocationLifetime::LongLived => stats.long_lived_allocations += 1,
                }
            } else {
                stats.reusable_allocations += 1;
                stats.reusable_reserved_bytes += slot.class.slot_bytes();
            }
        }

        stats
    }

    fn reserve_slot(
        &mut self,
        class: SideAllocationClass,
        payload_len: usize,
        lifetime: AllocationLifetime,
    ) -> (usize, SideAllocationRef) {
        if let Some(id) = self
            .free_by_class
            .get_mut(&class)
            .and_then(std::vec::Vec::pop)
        {
            let slot_index = (id - 1) as usize;
            let slot = &mut self.slots[slot_index];
            slot.lifetime = lifetime;
            slot.payload_len = payload_len;
            slot.occupied = true;
            return (slot_index, SideAllocationRef::from_raw(id).unwrap());
        }

        let bytes = vec![0_u8; class.slot_bytes()].into_boxed_slice();
        self.slots.push(SideAllocationSlot {
            class,
            lifetime,
            payload_len,
            occupied: true,
            bytes,
        });
        let id = SideAllocationRef::from_raw(
            u32::try_from(self.slots.len())
                .expect("side allocation handle count must fit into u32"),
        )
        .unwrap();
        (self.slots.len() - 1, id)
    }

    fn source_payload_len(&self, slot_index: usize) -> Option<usize> {
        let slot = self.slots.get(slot_index)?;
        slot.occupied.then_some(slot.payload_len)
    }
}

fn string_payload_len_for_encoding(encoding: StringEncoding, code_unit_len: u32) -> Option<usize> {
    match encoding {
        StringEncoding::Latin1 => usize::try_from(code_unit_len).ok(),
        StringEncoding::Utf16 => utf16_payload_len_for_code_units(code_unit_len),
    }
}

fn utf16_payload_len_for_code_units(code_unit_len: u32) -> Option<usize> {
    usize::try_from(code_unit_len).ok()?.checked_mul(2)
}

fn copy_between_side_slots(
    slots: &mut [SideAllocationSlot],
    source_index: usize,
    source_len: usize,
    destination_index: usize,
    destination_offset: usize,
) {
    if source_len == 0 {
        return;
    }
    debug_assert_ne!(
        source_index, destination_index,
        "side-allocation concat destination must not alias an occupied source"
    );

    if source_index < destination_index {
        let (before_destination, destination_and_after) = slots.split_at_mut(destination_index);
        let source = &before_destination[source_index];
        let destination = &mut destination_and_after[0];
        destination.bytes[destination_offset..destination_offset + source_len]
            .copy_from_slice(&source.bytes[..source_len]);
    } else {
        let (before_source, source_and_after) = slots.split_at_mut(source_index);
        let destination = &mut before_source[destination_index];
        let source = &source_and_after[0];
        destination.bytes[destination_offset..destination_offset + source_len]
            .copy_from_slice(&source.bytes[..source_len]);
    }
}

fn copy_string_payload_to_utf16_slot(
    slots: &mut [SideAllocationSlot],
    source_index: usize,
    source_encoding: StringEncoding,
    source_code_unit_len: u32,
    destination_index: usize,
    destination_offset: usize,
) {
    if source_code_unit_len == 0 {
        return;
    }
    debug_assert_ne!(
        source_index, destination_index,
        "side-allocation concat destination must not alias an occupied source"
    );

    if source_index < destination_index {
        let (before_destination, destination_and_after) = slots.split_at_mut(destination_index);
        let source = &before_destination[source_index];
        let destination = &mut destination_and_after[0];
        copy_string_payload_to_utf16_bytes(
            source,
            source_encoding,
            source_code_unit_len,
            destination,
            destination_offset,
        );
    } else {
        let (before_source, source_and_after) = slots.split_at_mut(source_index);
        let destination = &mut before_source[destination_index];
        let source = &source_and_after[0];
        copy_string_payload_to_utf16_bytes(
            source,
            source_encoding,
            source_code_unit_len,
            destination,
            destination_offset,
        );
    }
}

fn copy_string_payload_to_utf16_bytes(
    source: &SideAllocationSlot,
    source_encoding: StringEncoding,
    source_code_unit_len: u32,
    destination: &mut SideAllocationSlot,
    destination_offset: usize,
) {
    let source_len = usize::try_from(source_code_unit_len)
        .expect("string code unit length must fit addressable storage");
    match source_encoding {
        StringEncoding::Latin1 => {
            for (index, byte) in source.bytes[..source_len].iter().copied().enumerate() {
                let offset = destination_offset + index * 2;
                destination.bytes[offset] = byte;
                destination.bytes[offset + 1] = 0;
            }
        }
        StringEncoding::Utf16 => {
            let byte_len = source_len
                .checked_mul(2)
                .expect("UTF-16 payload length must fit addressable storage");
            destination.bytes[destination_offset..destination_offset + byte_len]
                .copy_from_slice(&source.bytes[..byte_len]);
        }
    }
}

fn side_allocation_index(id: SideAllocationRef) -> Option<usize> {
    usize::try_from(id.get()).ok()?.checked_sub(1)
}

pub(super) struct ValueSlotAllocator<Handle> {
    slots: Vec<ValueSlotBufferSlot>,
    free_by_len: BTreeMap<usize, Vec<u32>>,
    marker: PhantomData<Handle>,
}

impl<Handle> Default for ValueSlotAllocator<Handle> {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            free_by_len: BTreeMap::new(),
            marker: PhantomData,
        }
    }
}

struct ValueSlotBufferSlot {
    lifetime: AllocationLifetime,
    occupied: bool,
    values: Box<[Value]>,
}

impl<Handle: ArenaHandle> ValueSlotAllocator<Handle> {
    pub(super) fn allocation_requires_growth(&self, slot_count: usize) -> bool {
        !self.slots.is_empty()
            && self
                .free_by_len
                .get(&slot_count)
                .is_none_or(std::vec::Vec::is_empty)
    }

    pub(super) fn allocate(
        &mut self,
        slot_count: usize,
        fill: Value,
        lifetime: AllocationLifetime,
    ) -> Handle {
        if let Some(id) = self
            .free_by_len
            .get_mut(&slot_count)
            .and_then(std::vec::Vec::pop)
        {
            let slot = &mut self.slots[(id - 1) as usize];
            slot.lifetime = lifetime;
            slot.occupied = true;
            for value in &mut slot.values {
                *value = fill;
            }
            return Handle::from_raw(id).unwrap();
        }

        self.slots.push(ValueSlotBufferSlot {
            lifetime,
            occupied: true,
            values: vec![fill; slot_count].into_boxed_slice(),
        });
        Handle::from_raw(
            u32::try_from(self.slots.len()).expect("value slot buffer count must fit into u32"),
        )
        .unwrap()
    }

    pub(super) fn get(&self, id: Handle) -> Option<&[Value]> {
        let slot = self.slots.get((id.get() - 1) as usize)?;
        if slot.occupied {
            Some(&slot.values)
        } else {
            None
        }
    }

    pub(super) fn write(&mut self, id: Handle, index: u32, value: Value) -> bool {
        let Some(slot) = self.slots.get_mut((id.get() - 1) as usize) else {
            return false;
        };
        if !slot.occupied {
            return false;
        }
        let Some(target) = slot.values.get_mut(index as usize) else {
            return false;
        };
        *target = value;
        true
    }

    pub(super) fn free(&mut self, id: Handle) -> bool {
        let Some(slot) = self.slots.get_mut((id.get() - 1) as usize) else {
            return false;
        };
        if !slot.occupied {
            return false;
        }

        slot.occupied = false;
        self.free_by_len
            .entry(slot.values.len())
            .or_default()
            .push(id.get());
        true
    }

    pub(super) fn stats(&self) -> SideAllocationStats {
        let mut stats = SideAllocationStats::default();
        let value_bytes = std::mem::size_of::<Value>();

        for slot in &self.slots {
            let reserved_bytes = slot.values.len() * value_bytes;
            stats.reserved_bytes += reserved_bytes;
            if slot.occupied {
                stats.live_allocations += 1;
                stats.live_payload_bytes += reserved_bytes;
                match slot.lifetime {
                    AllocationLifetime::Default => stats.default_allocations += 1,
                    AllocationLifetime::LongLived => stats.long_lived_allocations += 1,
                }
            } else {
                stats.reusable_allocations += 1;
                stats.reusable_reserved_bytes += reserved_bytes;
            }
        }

        stats
    }
}

fn make_handle<Handle: ArenaHandle>(page_index: usize, slot_index: usize) -> Handle {
    let raw = u32::try_from(page_index * PRIMITIVE_SLOTS_PER_PAGE + slot_index + 1)
        .expect("primitive arena handle IDs must fit into u32");
    Handle::from_raw(raw).expect("primitive arena handle IDs must stay non-zero")
}

fn locate<Handle: ArenaHandle>(handle: Handle) -> Option<(usize, usize)> {
    let raw = handle.get().checked_sub(1)? as usize;
    Some((
        raw / PRIMITIVE_SLOTS_PER_PAGE,
        raw % PRIMITIVE_SLOTS_PER_PAGE,
    ))
}
