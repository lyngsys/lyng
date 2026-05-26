//! JSC-style per-code-object `MetadataTable`. Spec 2 Phase C.
//!
//! Layout: header + per-kind offset table + slot→in-kind-index table + per-kind runs.
//! Phase C.1 lands the type + allocator; reads and writes wire up in C.2/C.4.

pub mod kind;

#[allow(unused_imports)]
pub use kind::{MetadataKind, METADATA_KIND_COUNT};

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

#[allow(dead_code)]
const HEADER_SIZE: usize = std::mem::size_of::<LinkingDataHeader>();
#[allow(dead_code)]
const KIND_OFFSETS_OFFSET: usize = HEADER_SIZE;
#[allow(dead_code)]
const KIND_OFFSETS_SIZE: usize = METADATA_KIND_COUNT * 4;

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

        // 4. Write header.
        let header = LinkingDataHeader {
            buffer_size: buffer_size as u32,
            slot_count,
            slot_index_table_offset: slot_index_table_offset as u32,
            _reserved: 0,
        };
        // SAFETY: buffer has at least HEADER_SIZE bytes (16); LinkingDataHeader
        // is repr(C) with 4-byte fields and the buffer is allocated 8-aligned
        // (via Vec → Box).
        unsafe {
            std::ptr::write(buffer.as_mut_ptr() as *mut LinkingDataHeader, header);
        }

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
}

#[allow(dead_code)]
const fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

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
}
