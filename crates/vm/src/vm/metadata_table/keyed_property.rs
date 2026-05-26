use std::mem::offset_of;

/// Per-slot metadata for `KeyedPropertyAccess` IC sites. 24-byte stride.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KeyedPropertyMetadata {
    pub mode: u8,
    pub _pad: [u8; 3],
    pub generation: u32,
    pub handler_bits: u64,
    pub execution_count: u32,
    pub _tail: u32,
}

pub const KEYED_PROPERTY_METADATA_STRIDE: usize = std::mem::size_of::<KeyedPropertyMetadata>();
#[allow(dead_code)]
pub const KEYED_PROPERTY_METADATA_MODE_OFFSET: usize = offset_of!(KeyedPropertyMetadata, mode);
#[allow(dead_code)]
pub const KEYED_PROPERTY_METADATA_GENERATION_OFFSET: usize =
    offset_of!(KeyedPropertyMetadata, generation);
#[allow(dead_code)]
pub const KEYED_PROPERTY_METADATA_HANDLER_BITS_OFFSET: usize =
    offset_of!(KeyedPropertyMetadata, handler_bits);
#[allow(dead_code)]
pub const KEYED_PROPERTY_METADATA_EXEC_COUNT_OFFSET: usize =
    offset_of!(KeyedPropertyMetadata, execution_count);

const _: () = assert!(KEYED_PROPERTY_METADATA_STRIDE == 24);
