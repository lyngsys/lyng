use std::mem::offset_of;

/// Per-slot metadata for `Call` / `Construct` IC sites. 24-byte stride.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CallMetadata {
    pub mode: u8,
    pub _pad: [u8; 3],
    pub generation: u32,
    pub callee_bits: u64,
    pub execution_count: u32,
    pub _tail: u32,
}

pub const CALL_METADATA_STRIDE: usize = std::mem::size_of::<CallMetadata>();
#[allow(dead_code)]
pub const CALL_METADATA_MODE_OFFSET: usize = offset_of!(CallMetadata, mode);
#[allow(dead_code)]
pub const CALL_METADATA_GENERATION_OFFSET: usize = offset_of!(CallMetadata, generation);
#[allow(dead_code)]
pub const CALL_METADATA_CALLEE_BITS_OFFSET: usize = offset_of!(CallMetadata, callee_bits);
#[allow(dead_code)]
pub const CALL_METADATA_EXEC_COUNT_OFFSET: usize = offset_of!(CallMetadata, execution_count);

const _: () = assert!(CALL_METADATA_STRIDE == 24);
