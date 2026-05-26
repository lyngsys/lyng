//! JSC-style per-code-object `MetadataTable`. Spec 2 Phase C.
//!
//! Layout: header + per-kind offset table + slot→in-kind-index table + per-kind runs.
//! Phase C.1 lands the type + allocator; reads and writes wire up in C.2/C.4.

pub mod arith;
pub mod call;
pub mod comparison;
pub mod keyed_property;
pub mod kind;
pub mod property;

#[allow(unused_imports)]
pub use arith::{
    ArithMetadata, ARITH_METADATA_EXEC_COUNT_OFFSET, ARITH_METADATA_OBSERVED_BITS_OFFSET,
    ARITH_METADATA_STRIDE, ARITH_METADATA_STRIDE_SHIFT,
};
#[allow(unused_imports)]
pub use call::{CallMetadata, CALL_METADATA_STRIDE};
#[allow(unused_imports)]
pub use comparison::{ComparisonMetadata, COMPARISON_METADATA_STRIDE};
#[allow(unused_imports)]
pub use keyed_property::{KeyedPropertyMetadata, KEYED_PROPERTY_METADATA_STRIDE};
#[allow(unused_imports)]
pub use kind::{MetadataKind, METADATA_KIND_COUNT};
#[allow(unused_imports)]
pub use property::{PropertyMetadata, PROPERTY_METADATA_STRIDE};

/// Compact descriptor used by the allocator. Mirrors the shape of
/// `lyng_bytecode::FeedbackSiteDescriptor` but elides the metadata payload
/// (the allocator only needs the slot and kind).
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct SiteDescriptor {
    /// 1-based `FeedbackSlotId` value.
    pub slot: u32,
    pub kind: lyng_bytecode::FeedbackSiteKind,
}

#[repr(C)]
#[allow(dead_code)]
#[derive(Clone, Copy)]
struct LinkingDataHeader {
    buffer_size: u32,
    slot_count: u32,
    slot_index_table_offset: u32,
    _reserved: u32,
}

/// Size of the [`LinkingDataHeader`] prefix at the start of a MetadataTable buffer.
/// Used by asm to locate the kind-offsets table and the slot→in-kind-index table.
pub const METADATA_TABLE_HEADER_SIZE: usize = std::mem::size_of::<LinkingDataHeader>();
/// Byte offset of the per-kind run-offset table within a MetadataTable buffer.
pub const METADATA_TABLE_KIND_OFFSETS_OFFSET: usize = METADATA_TABLE_HEADER_SIZE;
/// Byte size of the per-kind run-offset table (`METADATA_KIND_COUNT × 4`).
pub const METADATA_TABLE_KIND_OFFSETS_SIZE: usize = METADATA_KIND_COUNT * 4;
/// Byte offset of the slot→in-kind-index table within a MetadataTable buffer.
/// Each entry is a `u32`; entry at index `slot - 1` gives the in-kind index for that slot.
pub const METADATA_TABLE_SLOT_INDEX_TABLE_OFFSET: usize =
    METADATA_TABLE_KIND_OFFSETS_OFFSET + METADATA_TABLE_KIND_OFFSETS_SIZE;

/// Byte offset within the MetadataTable buffer of the `kind_offsets[Arith]` entry.
/// `MetadataKind::Arith = 2`, so this is `METADATA_TABLE_KIND_OFFSETS_OFFSET + 2 * 4`.
pub const METADATA_TABLE_ARITH_KIND_OFFSET: usize = METADATA_TABLE_KIND_OFFSETS_OFFSET + 8;

// Sanity-assert: if LinkingDataHeader size or METADATA_KIND_COUNT changes, this
// will fail loudly rather than silently producing wrong asm offsets.
const _: () = assert!(
    METADATA_TABLE_SLOT_INDEX_TABLE_OFFSET == 36,
    "MetadataTable slot-index table must start at offset 36; \
     update asm bindings if LinkingDataHeader or METADATA_KIND_COUNT changed"
);
const _: () = assert!(
    METADATA_TABLE_ARITH_KIND_OFFSET == 24,
    "METADATA_TABLE_ARITH_KIND_OFFSET must be 24 (header=16, Arith index=2, 4 bytes/entry)"
);

// Private aliases kept for internal allocator use.
#[allow(dead_code)]
const HEADER_SIZE: usize = METADATA_TABLE_HEADER_SIZE;
#[allow(dead_code)]
const KIND_OFFSETS_OFFSET: usize = METADATA_TABLE_KIND_OFFSETS_OFFSET;
#[allow(dead_code)]
const KIND_OFFSETS_SIZE: usize = METADATA_TABLE_KIND_OFFSETS_SIZE;

/// Per-code-object IC metadata buffer. Phase C.1 ships the type and allocator;
/// per-kind reads/writes wire up in C.2, the asm fast path consumes it in C.4.
#[allow(dead_code)]
pub struct MetadataTable {
    buffer: Box<[u8]>,
    kind_offsets: [u32; METADATA_KIND_COUNT],
    per_kind_counts: [u32; METADATA_KIND_COUNT],
    slot_count: u32,
}

#[allow(dead_code)]
impl MetadataTable {
    /// Allocate a fresh table sized to hold per-kind metadata for `sites`.
    /// Sites need not be sorted; in-kind indices are assigned in slot order.
    pub fn allocate(sites: &[SiteDescriptor]) -> Self {
        let slot_count = sites.len() as u32;

        // 1. Tally per-kind counts.
        let mut per_kind_counts = [0u32; METADATA_KIND_COUNT];
        for site in sites {
            let mk = MetadataKind::from_site_kind(site.kind);
            per_kind_counts[mk.index()] += 1;
        }

        // 2. Compute buffer layout.
        let slot_index_table_offset = KIND_OFFSETS_OFFSET + KIND_OFFSETS_SIZE;
        let slot_index_table_size = (slot_count as usize) * 4;
        let runs_start = align_up(slot_index_table_offset + slot_index_table_size, 8);

        let mut kind_offsets = [0u32; METADATA_KIND_COUNT];
        let mut cursor = runs_start;
        for kind_idx in 0..METADATA_KIND_COUNT {
            kind_offsets[kind_idx] = cursor as u32;
            let stride = stride_for_kind_index(kind_idx);
            cursor += stride * (per_kind_counts[kind_idx] as usize);
            cursor = align_up(cursor, 8);
        }
        let buffer_size = cursor;

        // 3. Allocate zeroed buffer.
        let mut buffer = vec![0u8; buffer_size].into_boxed_slice();

        // 4. Write header (4 × u32 fields at offsets 0/4/8/12).
        buffer[0..4].copy_from_slice(&(buffer_size as u32).to_ne_bytes());
        buffer[4..8].copy_from_slice(&slot_count.to_ne_bytes());
        buffer[8..12].copy_from_slice(&(slot_index_table_offset as u32).to_ne_bytes());
        buffer[12..16].copy_from_slice(&0u32.to_ne_bytes()); // _reserved

        // 5. Write kind offsets immediately after the header.
        for (kind_idx, &ko) in kind_offsets.iter().enumerate() {
            let off = KIND_OFFSETS_OFFSET + kind_idx * 4;
            buffer[off..off + 4].copy_from_slice(&ko.to_ne_bytes());
        }

        // 6. Assign in-kind indices in slot-ascending order.
        let mut sorted: Vec<SiteDescriptor> = sites.to_vec();
        sorted.sort_by_key(|s| s.slot);
        let mut next_in_kind = [0u32; METADATA_KIND_COUNT];
        for site in &sorted {
            let mk = MetadataKind::from_site_kind(site.kind);
            let in_kind_idx = next_in_kind[mk.index()];
            next_in_kind[mk.index()] += 1;
            let slot_zero_based = (site.slot - 1) as usize;
            let off = slot_index_table_offset + slot_zero_based * 4;
            buffer[off..off + 4].copy_from_slice(&in_kind_idx.to_ne_bytes());
        }

        Self {
            buffer,
            kind_offsets,
            per_kind_counts,
            slot_count,
        }
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    pub fn buffer_ptr(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    pub fn buffer_ptr_mut(&mut self) -> *mut u8 {
        self.buffer.as_mut_ptr()
    }

    pub fn slot_count(&self) -> u32 {
        self.slot_count
    }

    pub fn kind_offset(&self, kind: MetadataKind) -> u32 {
        self.kind_offsets[kind.index()]
    }

    pub fn kind_offset_by_index(&self, kind_index: usize) -> u32 {
        self.kind_offsets[kind_index]
    }

    pub fn run_len_for_kind(&self, kind: MetadataKind) -> u32 {
        self.per_kind_counts[kind.index()]
    }

    pub fn run_len_for_kind_index(&self, kind_index: usize) -> u32 {
        self.per_kind_counts[kind_index]
    }

    /// Returns the per-kind in-kind index for `slot_one_based`. Reads from the
    /// in-buffer table (single source of truth, what asm will also read).
    pub fn in_kind_index_for_slot(&self, slot_one_based: u32) -> u32 {
        debug_assert!(slot_one_based >= 1 && slot_one_based <= self.slot_count);
        let zero_based = (slot_one_based - 1) as usize;
        let table_offset = KIND_OFFSETS_OFFSET + KIND_OFFSETS_SIZE;
        let byte_off = table_offset + zero_based * 4;
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.buffer[byte_off..byte_off + 4]);
        u32::from_ne_bytes(bytes)
    }

    fn entry_byte_offset(&self, kind: MetadataKind, slot_one_based: u32) -> usize {
        let in_kind = self.in_kind_index_for_slot(slot_one_based) as usize;
        (self.kind_offset(kind) as usize) + in_kind * kind.stride_bytes()
    }

    pub fn property(&self, slot: u32) -> &property::PropertyMetadata {
        let off = self.entry_byte_offset(MetadataKind::Property, slot);
        // SAFETY: allocator reserves `stride_bytes(Property) = 32` bytes at this offset
        // inside the Property run; the run starts at an 8-byte-aligned offset (allocator
        // aligns runs to 8). PropertyMetadata is repr(C) with max field alignment 8, so
        // this raw cast lands on a properly aligned pointer.
        unsafe { &*(self.buffer.as_ptr().add(off) as *const property::PropertyMetadata) }
    }

    pub fn property_mut(&mut self, slot: u32) -> &mut property::PropertyMetadata {
        let off = self.entry_byte_offset(MetadataKind::Property, slot);
        // SAFETY: same invariants as `property` above; exclusive &mut self gives
        // exclusive access to the underlying byte range.
        unsafe { &mut *(self.buffer.as_mut_ptr().add(off) as *mut property::PropertyMetadata) }
    }

    pub fn call(&self, slot: u32) -> &call::CallMetadata {
        let off = self.entry_byte_offset(MetadataKind::Call, slot);
        // SAFETY: allocator reserves `stride_bytes(Call) = 24` bytes at this offset;
        // run is 8-aligned; CallMetadata is repr(C) with max field alignment 8.
        unsafe { &*(self.buffer.as_ptr().add(off) as *const call::CallMetadata) }
    }

    pub fn call_mut(&mut self, slot: u32) -> &mut call::CallMetadata {
        let off = self.entry_byte_offset(MetadataKind::Call, slot);
        // SAFETY: same invariants as `call` above; exclusive &mut self.
        unsafe { &mut *(self.buffer.as_mut_ptr().add(off) as *mut call::CallMetadata) }
    }

    pub fn arith(&self, slot: u32) -> &arith::ArithMetadata {
        let off = self.entry_byte_offset(MetadataKind::Arith, slot);
        // SAFETY: allocator reserves `stride_bytes(Arith) = 8` bytes at this offset;
        // run is 8-aligned; ArithMetadata is repr(C) with max field alignment 4.
        unsafe { &*(self.buffer.as_ptr().add(off) as *const arith::ArithMetadata) }
    }

    pub fn arith_mut(&mut self, slot: u32) -> &mut arith::ArithMetadata {
        let off = self.entry_byte_offset(MetadataKind::Arith, slot);
        // SAFETY: same invariants as `arith` above; exclusive &mut self.
        unsafe { &mut *(self.buffer.as_mut_ptr().add(off) as *mut arith::ArithMetadata) }
    }

    pub fn comparison(&self, slot: u32) -> &comparison::ComparisonMetadata {
        let off = self.entry_byte_offset(MetadataKind::Comparison, slot);
        // SAFETY: allocator reserves `stride_bytes(Comparison) = 8` bytes at this offset;
        // run is 8-aligned; ComparisonMetadata is repr(C) with max field alignment 4.
        unsafe { &*(self.buffer.as_ptr().add(off) as *const comparison::ComparisonMetadata) }
    }

    pub fn comparison_mut(&mut self, slot: u32) -> &mut comparison::ComparisonMetadata {
        let off = self.entry_byte_offset(MetadataKind::Comparison, slot);
        // SAFETY: same invariants as `comparison` above; exclusive &mut self.
        unsafe { &mut *(self.buffer.as_mut_ptr().add(off) as *mut comparison::ComparisonMetadata) }
    }

    pub fn keyed_property(&self, slot: u32) -> &keyed_property::KeyedPropertyMetadata {
        let off = self.entry_byte_offset(MetadataKind::KeyedProperty, slot);
        // SAFETY: allocator reserves `stride_bytes(KeyedProperty) = 24` bytes at this offset;
        // run is 8-aligned; KeyedPropertyMetadata is repr(C) with max field alignment 8.
        unsafe { &*(self.buffer.as_ptr().add(off) as *const keyed_property::KeyedPropertyMetadata) }
    }

    pub fn keyed_property_mut(&mut self, slot: u32) -> &mut keyed_property::KeyedPropertyMetadata {
        let off = self.entry_byte_offset(MetadataKind::KeyedProperty, slot);
        // SAFETY: same invariants as `keyed_property` above; exclusive &mut self.
        unsafe {
            &mut *(self.buffer.as_mut_ptr().add(off) as *mut keyed_property::KeyedPropertyMetadata)
        }
    }
}

#[allow(dead_code)]
const fn align_up(value: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (value + align - 1) & !(align - 1)
}

const _: () = assert!(
    METADATA_KIND_COUNT == 5,
    "stride_for_kind_index must be updated when METADATA_KIND_COUNT changes"
);

#[allow(dead_code)]
const fn stride_for_kind_index(kind_index: usize) -> usize {
    match kind_index {
        0 => MetadataKind::Property.stride_bytes(),
        1 => MetadataKind::Call.stride_bytes(),
        2 => MetadataKind::Arith.stride_bytes(),
        3 => MetadataKind::Comparison.stride_bytes(),
        4 => MetadataKind::KeyedProperty.stride_bytes(),
        _ => panic!("kind index out of range"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_bytecode::FeedbackSiteKind;

    fn site(slot_one_based: u32, kind: FeedbackSiteKind) -> SiteDescriptor {
        SiteDescriptor {
            slot: slot_one_based,
            kind,
        }
    }

    #[test]
    fn empty_table_has_only_header_and_offset_block() {
        let table = MetadataTable::allocate(&[]);
        // Header (16) + kind_offsets (20) = 36 bytes; aligned up to 40.
        assert!(table.buffer().len() >= 36);
        assert_eq!(table.slot_count(), 0);
        for kind_idx in 0..METADATA_KIND_COUNT {
            assert_eq!(table.run_len_for_kind_index(kind_idx), 0);
        }
    }

    #[test]
    fn table_assigns_per_kind_indices_in_slot_order() {
        let sites = vec![
            site(1, FeedbackSiteKind::NamedPropertyLoad),
            site(2, FeedbackSiteKind::Arithmetic),
            site(3, FeedbackSiteKind::NamedPropertyLoad),
            site(4, FeedbackSiteKind::Call),
            site(5, FeedbackSiteKind::Arithmetic),
            site(6, FeedbackSiteKind::NamedPropertyLoad),
        ];
        let table = MetadataTable::allocate(&sites);
        assert_eq!(table.in_kind_index_for_slot(1), 0);
        assert_eq!(table.in_kind_index_for_slot(3), 1);
        assert_eq!(table.in_kind_index_for_slot(6), 2);
        assert_eq!(table.in_kind_index_for_slot(2), 0);
        assert_eq!(table.in_kind_index_for_slot(5), 1);
        assert_eq!(table.in_kind_index_for_slot(4), 0);
    }

    #[test]
    fn table_run_lengths_match_per_kind_counts() {
        let sites = vec![
            site(1, FeedbackSiteKind::NamedPropertyLoad),
            site(2, FeedbackSiteKind::NamedPropertyStore),
            site(3, FeedbackSiteKind::Call),
        ];
        let table = MetadataTable::allocate(&sites);
        assert_eq!(table.run_len_for_kind(MetadataKind::Property), 2);
        assert_eq!(table.run_len_for_kind(MetadataKind::Call), 1);
        assert_eq!(table.run_len_for_kind(MetadataKind::Arith), 0);
    }

    #[test]
    fn table_kind_offsets_land_inside_buffer() {
        let sites = vec![site(1, FeedbackSiteKind::NamedPropertyLoad)];
        let table = MetadataTable::allocate(&sites);
        for kind_idx in 0..METADATA_KIND_COUNT {
            let off = table.kind_offset_by_index(kind_idx);
            assert!(off as usize <= table.buffer().len());
        }
    }

    #[test]
    fn write_then_read_property_metadata_roundtrips() {
        use crate::vm::metadata_table::property::PropertyMetadata;
        let sites = vec![
            SiteDescriptor {
                slot: 1,
                kind: FeedbackSiteKind::NamedPropertyLoad,
            },
            SiteDescriptor {
                slot: 2,
                kind: FeedbackSiteKind::NamedPropertyLoad,
            },
        ];
        let mut table = MetadataTable::allocate(&sites);
        *table.property_mut(1) = PropertyMetadata {
            mode: 3,
            generation: 7,
            handler_bits: 0xdeadbeef,
            aux_bits: 0xcafe,
            execution_count: 42,
            ..Default::default()
        };
        let got = *table.property(1);
        assert_eq!(got.mode, 3);
        assert_eq!(got.generation, 7);
        assert_eq!(got.handler_bits, 0xdeadbeef);
        assert_eq!(got.aux_bits, 0xcafe);
        assert_eq!(got.execution_count, 42);
        assert_eq!(*table.property(2), PropertyMetadata::default());
    }

    #[test]
    fn write_then_read_call_metadata_roundtrips() {
        use crate::vm::metadata_table::call::CallMetadata;
        let sites = vec![
            SiteDescriptor {
                slot: 1,
                kind: FeedbackSiteKind::Call,
            },
            SiteDescriptor {
                slot: 2,
                kind: FeedbackSiteKind::Call,
            },
        ];
        let mut table = MetadataTable::allocate(&sites);
        *table.call_mut(1) = CallMetadata {
            mode: 5,
            generation: 11,
            callee_bits: 0xabcd1234_5678ef90,
            execution_count: 99,
            ..Default::default()
        };
        let got = *table.call(1);
        assert_eq!(got.mode, 5);
        assert_eq!(got.generation, 11);
        assert_eq!(got.callee_bits, 0xabcd1234_5678ef90);
        assert_eq!(got.execution_count, 99);
        assert_eq!(*table.call(2), CallMetadata::default());
    }

    #[test]
    fn write_then_read_arith_metadata_roundtrips() {
        use crate::vm::metadata_table::arith::ArithMetadata;
        let sites = vec![
            SiteDescriptor {
                slot: 1,
                kind: FeedbackSiteKind::Arithmetic,
            },
            SiteDescriptor {
                slot: 2,
                kind: FeedbackSiteKind::Arithmetic,
            },
        ];
        let mut table = MetadataTable::allocate(&sites);
        *table.arith_mut(1) = ArithMetadata {
            observed_bits: 0xf00d,
            execution_count: 77,
        };
        let got = *table.arith(1);
        assert_eq!(got.observed_bits, 0xf00d);
        assert_eq!(got.execution_count, 77);
        assert_eq!(*table.arith(2), ArithMetadata::default());
    }

    #[test]
    fn write_then_read_comparison_metadata_roundtrips() {
        use crate::vm::metadata_table::comparison::ComparisonMetadata;
        let sites = vec![
            SiteDescriptor {
                slot: 1,
                kind: FeedbackSiteKind::Comparison,
            },
            SiteDescriptor {
                slot: 2,
                kind: FeedbackSiteKind::Comparison,
            },
        ];
        let mut table = MetadataTable::allocate(&sites);
        *table.comparison_mut(1) = ComparisonMetadata {
            observed_bits: 0x1234,
            execution_count: 55,
        };
        let got = *table.comparison(1);
        assert_eq!(got.observed_bits, 0x1234);
        assert_eq!(got.execution_count, 55);
        assert_eq!(*table.comparison(2), ComparisonMetadata::default());
    }

    #[test]
    fn write_then_read_keyed_property_metadata_roundtrips() {
        use crate::vm::metadata_table::keyed_property::KeyedPropertyMetadata;
        let sites = vec![
            SiteDescriptor {
                slot: 1,
                kind: FeedbackSiteKind::KeyedPropertyAccess,
            },
            SiteDescriptor {
                slot: 2,
                kind: FeedbackSiteKind::KeyedPropertyAccess,
            },
        ];
        let mut table = MetadataTable::allocate(&sites);
        *table.keyed_property_mut(1) = KeyedPropertyMetadata {
            mode: 2,
            generation: 9,
            handler_bits: 0x0102030405060708,
            execution_count: 33,
            ..Default::default()
        };
        let got = *table.keyed_property(1);
        assert_eq!(got.mode, 2);
        assert_eq!(got.generation, 9);
        assert_eq!(got.handler_bits, 0x0102030405060708);
        assert_eq!(got.execution_count, 33);
        assert_eq!(*table.keyed_property(2), KeyedPropertyMetadata::default());
    }
}
