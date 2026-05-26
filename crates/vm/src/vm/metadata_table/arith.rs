use std::mem::offset_of;

/// Per-slot metadata for `Arithmetic` IC sites. 8-byte stride.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ArithMetadata {
    pub observed_bits: u32,
    pub execution_count: u32,
}

pub const ARITH_METADATA_STRIDE: usize = std::mem::size_of::<ArithMetadata>();
#[allow(dead_code)]
pub const ARITH_METADATA_OBSERVED_BITS_OFFSET: usize = offset_of!(ArithMetadata, observed_bits);
#[allow(dead_code)]
pub const ARITH_METADATA_EXEC_COUNT_OFFSET: usize = offset_of!(ArithMetadata, execution_count);

const _: () = assert!(ARITH_METADATA_STRIDE == 8);
