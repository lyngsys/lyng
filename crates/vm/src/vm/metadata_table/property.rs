use std::mem::offset_of;

/// Per-slot metadata for `NamedPropertyLoad` / `NamedPropertyStore` IC sites.
/// 32-byte stride. Phase C.2 dual-write only; Phase D becomes system of record.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PropertyMetadata {
    pub mode: u8,
    pub _pad: [u8; 3],
    pub generation: u32,
    pub handler_bits: u64,
    pub aux_bits: u64,
    pub execution_count: u32,
    pub _tail_pad: u32,
}

pub const PROPERTY_METADATA_STRIDE: usize = std::mem::size_of::<PropertyMetadata>();
/// `log2(PROPERTY_METADATA_STRIDE)` — used by asm to scale an in-kind index
/// to a byte offset within the Property run.
pub const PROPERTY_METADATA_STRIDE_SHIFT: u32 = 5; // log2(32)

#[allow(dead_code)]
pub const PROPERTY_METADATA_MODE_OFFSET: usize = offset_of!(PropertyMetadata, mode);
#[allow(dead_code)]
pub const PROPERTY_METADATA_GENERATION_OFFSET: usize = offset_of!(PropertyMetadata, generation);
#[allow(dead_code)]
pub const PROPERTY_METADATA_HANDLER_BITS_OFFSET: usize = offset_of!(PropertyMetadata, handler_bits);
#[allow(dead_code)]
pub const PROPERTY_METADATA_AUX_BITS_OFFSET: usize = offset_of!(PropertyMetadata, aux_bits);
#[allow(dead_code)]
pub const PROPERTY_METADATA_EXEC_COUNT_OFFSET: usize =
    offset_of!(PropertyMetadata, execution_count);

const _: () = assert!(PROPERTY_METADATA_STRIDE == 32);
const _: () = assert!(
    1 << PROPERTY_METADATA_STRIDE_SHIFT == PROPERTY_METADATA_STRIDE,
    "PROPERTY_METADATA_STRIDE_SHIFT must equal log2(PROPERTY_METADATA_STRIDE)"
);
