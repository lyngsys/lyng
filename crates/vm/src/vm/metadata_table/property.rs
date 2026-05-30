use std::mem::offset_of;

/// Per-slot metadata for `NamedPropertyLoad` / `NamedPropertyStore` IC sites.
/// 32-byte stride. Phase C.2 dual-write only; Phase D becomes system of record.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
// `_pad`/`_tail_pad` are explicit layout padding kept `pub` so the asm offset
// asserts and `repr(C)` layout stay verifiable from outside the module.
#[allow(clippy::pub_underscore_fields)]
pub struct PropertyMetadata {
    pub mode: u8,
    pub _pad: [u8; 3],
    pub generation: u32,
    pub handler_bits: u64,
    pub aux_bits: u64,
    pub execution_count: u32,
    pub _tail_pad: u32,
}

/// Mode byte written by `project_property_write_into_meta` for a monomorphic
/// own-data inline write (including transitioning writes). Consumed by the asm
/// `op_assign_named_property` fast path.
pub const LLINT_IC_MODE_NAMED_OWN_INLINE_WRITE: u8 = 5;

/// Asm IC mode: global cell load. `handler_bits` = the `PrimitiveValueCellRef`
/// raw u32; `generation` = the global-IC generation captured at install. The asm
/// hit loads the cell value when `generation` matches the live Vm mirror. Mode 6
/// is reserved for a future GlobalCellConstant (constant-fold) mode.
pub const LLINT_IC_MODE_GLOBAL_CELL_LOAD: u8 = 7;

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
