use std::mem::offset_of;

/// Per-slot metadata for `Comparison` IC sites. 8-byte stride.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ComparisonMetadata {
    pub observed_bits: u32,
    pub execution_count: u32,
}

pub const COMPARISON_METADATA_STRIDE: usize = std::mem::size_of::<ComparisonMetadata>();
#[allow(dead_code)]
pub const COMPARISON_METADATA_OBSERVED_BITS_OFFSET: usize =
    offset_of!(ComparisonMetadata, observed_bits);
#[allow(dead_code)]
pub const COMPARISON_METADATA_EXEC_COUNT_OFFSET: usize =
    offset_of!(ComparisonMetadata, execution_count);

const _: () = assert!(COMPARISON_METADATA_STRIDE == 8);
