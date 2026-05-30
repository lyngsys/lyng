//! Inline-cache feedback macros.
//!
//! Phase C.4 pin flip: `x21` now holds the `MetadataTable` buffer base
//! (`state.frame_metadata_table_base`). Both `load_feedback_site!` (Property
//! kind) and `record_smi!` / `record_object!` / `record_double!` (Arith kind)
//! resolve through x21 directly.
//!
//! All internal scratch use is on `x16` / `x17` (see
//! `values.rs` for the rationale).
//!
//! Precomputed-offset layout (Phase C optimization):
//! The `MetadataTable` buffer starts with a `slot_to_entry_offset[N]` table:
//! each entry is a `u32` byte offset from the buffer base to the metadata
//! entry for that (1-based) slot. Asm resolves a slot in 3 instructions:
//!   sub  x17, x{slot}, #1          (0-based)
//!   ldr  w16, [x21, x17, lsl #2]   (load precomputed offset)
//!   add  x{dst}, x21, x16          (buffer + offset = entry ptr)
//!
//! Bindings expected from the proc-macro lowerer:
//! - `{state_mt}` — byte offset of `LlIntState::frame_metadata_table_base`.
//! - `{arith_metadata_observed_bits_offset}` — byte offset of `ArithMetadata::observed_bits` = 0.
//! - `{arith_metadata_exec_count_offset}` — byte offset of `ArithMetadata::execution_count` = 4.
//! - `{feedback_mode}` — byte offset of the `LLInt` IC mode byte.
//! - `{feedback_named_handler_bits}` — byte offset of the packed named
//!   property handler word.
//! - `{feedback_named_aux_bits}` — byte offset of the auxiliary
//!   named-property handler word.

/// Resolve a 1-based `FeedbackSlotId` to a `*mut PropertyMetadata` entry and
/// write the pointer into `x{$dst}`. x21 (MT pin) must hold the `MetadataTable`
/// buffer base (Phase C.4 pin flip).
///
/// Uses the precomputed `slot_to_entry_offset` table at buffer[0..N*4].
///
/// Scratch: x16, x17 (AAPCS64 IP0/IP1 — never overlap live operand slots).
///
/// 3-instruction sequence (x21 = `MetadataTable` buffer base):
/// 0. `sub  x17, x{slot}, #1`         — 0-based slot index
/// 1. `ldr  w16, [x21, x17, lsl #2]`  — w16 = `slot_to_entry_offset`[slot-1]
/// 2. `add  x{dst}, x21, x16`         — dst = buffer + `entry_offset`
#[macro_export]
macro_rules! load_feedback_site {
    ($slot:tt => $dst:tt) => {
        concat!(
            // (0) Convert 1-based slot id to 0-based index.
            "sub    x17, x",
            stringify!($slot),
            ", #1\n",
            // (1) Load precomputed entry offset from slot_to_entry_offset table.
            "ldr    w16, [x21, x17, lsl #2]\n",
            // (2) Entry pointer = buffer_base + entry_offset.
            "add    x",
            stringify!($dst),
            ", x21, x16\n",
        )
    };
}

/// Branch to `$label` unless the flat feedback entry is a named
/// monomorphic `OwnData` inline-slot load header.
#[macro_export]
macro_rules! branch_named_own_inline_mode {
    ($entry:tt, $label:tt) => {
        concat!(
            "ldrb   w16, [x",
            stringify!($entry),
            ", {feedback_mode}]\n",
            "cmp    w16, #1\n",
            "b.ne   ",
            stringify!($label),
            "\n",
        )
    };
}

/// Branch to `$label` unless the metadata-table property entry is a
/// `GlobalCellLoad` header (mode == 7 — `LLINT_IC_MODE_GLOBAL_CELL_LOAD`).
/// The packed `handler_bits` carries the 1-based `PrimitiveValueCellRef`
/// raw u32 in its low 32 bits; `generation` carries the global-IC
/// generation captured at install. Used by `op_load_global`'s mode-7
/// asm fast read.
///
/// Scratch: x16 (AAPCS64 IP0).
#[macro_export]
macro_rules! branch_global_cell_mode {
    ($entry:tt, $label:tt) => {
        concat!(
            "ldrb   w16, [x",
            stringify!($entry),
            ", {feedback_mode}]\n",
            "cmp    w16, #7\n",
            "b.ne   ",
            stringify!($label),
            "\n",
        )
    };
}

/// Branch to `$label` when the cached `metadata.generation` no longer
/// matches the live `Vm::dsl_global_ic_generation` mirror (x22 = Vm). A
/// global structural mutation (delete/defineProperty/sloppy-create) bumps
/// the agent generation, which `translate_outcome` mirrors into the Vm on
/// every slow egress; a mismatch means the cached cell ref is stale and the
/// hit must bail to the cold path for re-resolution.
///
/// Scratch: x16, x17 (AAPCS64 IP0/IP1).
#[macro_export]
macro_rules! branch_global_cell_generation_mismatch {
    ($entry:tt, $label:tt) => {
        concat!(
            "ldr    w16, [x",
            stringify!($entry),
            ", {feedback_generation}]\n",
            "ldr    w17, [x22, {vm_global_ic_gen}]\n",
            "cmp    w16, w17\n",
            "b.ne   ",
            stringify!($label),
            "\n",
        )
    };
}

/// Load the global cell's stored value into `x{$dst}`. `handler_bits` (low
/// 32 bits of the metadata entry) is the 1-based `PrimitiveValueCellRef`;
/// the value-cell pointer table base lives in `LlIntState` (x24 = STATE).
/// `table[ref]` is a `*const PrimitiveValueCellRecord` (8-byte stride →
/// `lsl #3`); the stored value sits at `{cell_stored_value}` (= 0) within
/// the record. A null table entry (freed/republished cell) branches to
/// `$label` so the cold path re-resolves.
///
/// Scratch: x16, x17 (AAPCS64 IP0/IP1).
#[macro_export]
macro_rules! load_global_cell_value_or_branch {
    ($entry:tt => $dst:tt, $label:tt) => {
        concat!(
            // (0) handler_bits low 32 bits = 1-based cell ref (zero-extended).
            "ldr    w16, [x",
            stringify!($entry),
            ", {feedback_named_handler_bits}]\n",
            // (1) value-cell pointer table base from LlIntState (x24).
            "ldr    x17, [x24, {state_value_cells}]\n",
            // (2) table[ref] = *const PrimitiveValueCellRecord (8-byte stride).
            "ldr    x16, [x17, x16, lsl #3]\n",
            // (3) null entry (freed cell) -> bail to cold path.
            "cbz    x16, ",
            stringify!($label),
            "\n",
            // (4) load the stored value at record + {cell_stored_value}.
            "ldr    x",
            stringify!($dst),
            ", [x16, {cell_stored_value}]\n",
        )
    };
}

/// Branch to `$label` unless the flat feedback entry is a named
/// monomorphic one-hop `PrototypeData` inline-slot load header.
#[macro_export]
macro_rules! branch_named_proto_inline_mode {
    ($entry:tt, $label:tt) => {
        concat!(
            "ldrb   w16, [x",
            stringify!($entry),
            ", {feedback_mode}]\n",
            "cmp    w16, #2\n",
            "b.ne   ",
            stringify!($label),
            "\n",
        )
    };
}

/// Branch to `$label` unless the flat feedback entry is a named
/// monomorphic `OwnData` out-of-line-slot load header.
#[macro_export]
macro_rules! branch_named_own_outline_mode {
    ($entry:tt, $label:tt) => {
        concat!(
            "ldrb   w16, [x",
            stringify!($entry),
            ", {feedback_mode}]\n",
            "cmp    w16, #3\n",
            "b.ne   ",
            stringify!($label),
            "\n",
        )
    };
}

/// Branch to `$label` unless the metadata-table property entry is a
/// polymorphic `OwnData` inline-slot load header. The pair of cached
/// handlers is packed into the existing primary (slot 0) and auxiliary
/// (slot 1) fields by the Rust install path.
#[macro_export]
macro_rules! branch_named_own_polymorphic_mode {
    ($entry:tt, $label:tt) => {
        concat!(
            "ldrb   w16, [x",
            stringify!($entry),
            ", {feedback_mode}]\n",
            "cmp    w16, #4\n",
            "b.ne   ",
            stringify!($label),
            "\n",
        )
    };
}

/// Branch to `$label` unless the metadata-table property entry is a
/// monomorphic `OwnDataInlineWrite` header. `mode == 5` —
/// `LLINT_IC_MODE_NAMED_OWN_INLINE_WRITE`. The packed handler word
/// carries (`source_shape`, slot, `writable_flag`, `inline_flag`) in the
/// same layout as the read-side own-inline mode; `aux_bits` carries
/// the target shape.
#[macro_export]
macro_rules! branch_named_own_inline_write_mode {
    ($entry:tt, $label:tt) => {
        concat!(
            "ldrb   w16, [x",
            stringify!($entry),
            ", {feedback_mode}]\n",
            "cmp    w16, #5\n",
            "b.ne   ",
            stringify!($label),
            "\n",
        )
    };
}

#[macro_export]
macro_rules! load_named_handler_bits {
    ($entry:tt => $dst:tt) => {
        concat!(
            "ldr    x",
            stringify!($dst),
            ", [x",
            stringify!($entry),
            ", {feedback_named_handler_bits}]\n",
        )
    };
}

#[macro_export]
macro_rules! load_named_aux_bits {
    ($entry:tt => $dst:tt) => {
        concat!(
            "ldr    x",
            stringify!($dst),
            ", [x",
            stringify!($entry),
            ", {feedback_named_aux_bits}]\n",
        )
    };
}

/// Validate a packed named-property handler as an inline-slot handler
/// and extract its low 30-bit slot index.
#[macro_export]
macro_rules! load_named_inline_slot_index_or_branch {
    ($handler:tt => $slot_index:tt, $label:tt) => {
        concat!(
            "cbz    x",
            stringify!($handler),
            ", ",
            stringify!($label),
            "\n",
            "tbz    x",
            stringify!($handler),
            ", #31, ",
            stringify!($label),
            "\n",
            "ubfx   x",
            stringify!($slot_index),
            ", x",
            stringify!($handler),
            ", #0, #30\n",
        )
    };
}

/// Validate a packed named-property handler as a *writable* inline-slot
/// handler and extract its low 30-bit slot index. Branches to `$label`
/// when the handler is invalid (`bits == 0`), an out-of-line slot
/// (bit 31 = 0), or read-only (bit 30 = 0). Store-side counterpart of
/// [`load_named_inline_slot_index_or_branch!`]: the writable-bit check
/// (bit 30 of the packed handler — see
/// `NamedPropertyHandler::HANDLER_WRITABLE_FLAG`) distinguishes a
/// store-eligible monomorphic `OwnData` entry from a read-only one.
/// Read-only hits must miss to the Rust probe / slow path so the
/// strict-mode `TypeError` contract is preserved.
#[macro_export]
macro_rules! load_named_inline_writable_slot_index_or_branch {
    ($handler:tt => $slot_index:tt, $label:tt) => {
        concat!(
            "cbz    x",
            stringify!($handler),
            ", ",
            stringify!($label),
            "\n",
            "tbz    x",
            stringify!($handler),
            ", #31, ",
            stringify!($label),
            "\n",
            "tbz    x",
            stringify!($handler),
            ", #30, ",
            stringify!($label),
            "\n",
            "ubfx   x",
            stringify!($slot_index),
            ", x",
            stringify!($handler),
            ", #0, #30\n",
        )
    };
}

/// Validate a packed named-property handler as an out-of-line-slot
/// handler and extract its low 30-bit slot index.
#[macro_export]
macro_rules! load_named_outline_slot_index_or_branch {
    ($handler:tt => $slot_index:tt, $label:tt) => {
        concat!(
            "cbz    x",
            stringify!($handler),
            ", ",
            stringify!($label),
            "\n",
            "tbnz   x",
            stringify!($handler),
            ", #31, ",
            stringify!($label),
            "\n",
            "ubfx   x",
            stringify!($slot_index),
            ", x",
            stringify!($handler),
            ", #0, #30\n",
        )
    };
}

#[macro_export]
macro_rules! load_named_handler_shape {
    ($handler:tt => $dst:tt) => {
        concat!(
            "lsr    x",
            stringify!($dst),
            ", x",
            stringify!($handler),
            ", #32\n",
        )
    };
}

/// Load the target shape from the feedback entry's `aux_bits` field
/// into register `$dst`. The low 32 bits of `aux_bits` carry the target
/// `ShapeId` raw u32 (high 32 bits reserved/zero). Used by the
/// `OwnDataInlineWrite` asm fast path to update the object's shape
/// pointer after the inline-slot store.
#[macro_export]
macro_rules! load_named_target_shape {
    ($entry:tt => $dst:tt) => {
        concat!(
            "ldr    w",
            stringify!($dst),
            ", [x",
            stringify!($entry),
            ", {feedback_named_aux_bits}]\n",
        )
    };
}

/// Record that an SMI was observed at slot `$slot` and saturating-increment
/// the pending scalar execution count. Uses x21 as `MetadataTable` base (Phase C.4).
///
/// Resolves `ArithMetadata` pointer via the precomputed `slot_to_entry_offset` table.
/// Writes to `ArithMetadata.{observed_bits, execution_count}` (offsets 0 and 4).
///
/// 10-instruction sequence (x16/x17 scratch):
/// - `0.` sub  x17, x{slot}, #1
/// - `1.` ldr  w16, [x21, x17, lsl #2]
/// - `2.` add  x16, x21, x16
/// - `3.` ldr  w17, [x16, #{`arith_metadata_observed_bits_offset`}]
/// - `4.` orr  w17, w17, #0x1
/// - `5.` str  w17, [x16, #{`arith_metadata_observed_bits_offset`}]
/// - `6.` ldr  w17, [x16, #{`arith_metadata_exec_count_offset`}]
/// - `7.` adds w17, w17, #1
/// - `8.` csinv w17, w17, wzr, cc
/// - `9.` str  w17, [x16, #{`arith_metadata_exec_count_offset`}]
#[macro_export]
macro_rules! record_smi {
    ($slot:tt) => {
        concat!(
            // (0) 0-based slot index.
            "sub    x17, x",
            stringify!($slot),
            ", #1\n",
            // (1) Load precomputed entry offset.
            "ldr    w16, [x21, x17, lsl #2]\n",
            // (2) Entry pointer = buffer_base + entry_offset.
            "add    x16, x21, x16\n",
            // (3-5) Update observed_bits |= SMI bit (0x1).
            "ldr    w17, [x16, #{arith_metadata_observed_bits_offset}]\n",
            "orr    w17, w17, #0x1\n",
            "str    w17, [x16, #{arith_metadata_observed_bits_offset}]\n",
            // (6-9) Saturating-increment execution_count.
            "ldr    w17, [x16, #{arith_metadata_exec_count_offset}]\n",
            "adds   w17, w17, #1\n",
            "csinv  w17, w17, wzr, cc\n",
            "str    w17, [x16, #{arith_metadata_exec_count_offset}]\n",
        )
    };
}

/// Record an Object observation (bit 1). SMI bit 0; Object bit 1.
/// Uses x21 as `MetadataTable` base (Phase C.4). See `record_smi!` for layout.
#[macro_export]
macro_rules! record_object {
    ($slot:tt) => {
        concat!(
            // (0) 0-based slot index.
            "sub    x17, x",
            stringify!($slot),
            ", #1\n",
            // (1) Load precomputed entry offset.
            "ldr    w16, [x21, x17, lsl #2]\n",
            // (2) Entry pointer = buffer_base + entry_offset.
            "add    x16, x21, x16\n",
            // (3-5) Update observed_bits |= Object bit (0x2).
            "ldr    w17, [x16, #{arith_metadata_observed_bits_offset}]\n",
            "orr    w17, w17, #0x2\n",
            "str    w17, [x16, #{arith_metadata_observed_bits_offset}]\n",
            // (6-9) Saturating-increment execution_count.
            "ldr    w17, [x16, #{arith_metadata_exec_count_offset}]\n",
            "adds   w17, w17, #1\n",
            "csinv  w17, w17, wzr, cc\n",
            "str    w17, [x16, #{arith_metadata_exec_count_offset}]\n",
        )
    };
}

/// Record a Double observation (bit 2 of observed types).
/// Uses x21 as `MetadataTable` base (Phase C.4). See `record_smi!` for layout.
#[macro_export]
macro_rules! record_double {
    ($slot:tt) => {
        concat!(
            // (0) 0-based slot index.
            "sub    x17, x",
            stringify!($slot),
            ", #1\n",
            // (1) Load precomputed entry offset.
            "ldr    w16, [x21, x17, lsl #2]\n",
            // (2) Entry pointer = buffer_base + entry_offset.
            "add    x16, x21, x16\n",
            // (3-5) Update observed_bits |= Double bit (0x4).
            "ldr    w17, [x16, #{arith_metadata_observed_bits_offset}]\n",
            "orr    w17, w17, #0x4\n",
            "str    w17, [x16, #{arith_metadata_observed_bits_offset}]\n",
            // (6-9) Saturating-increment execution_count.
            "ldr    w17, [x16, #{arith_metadata_exec_count_offset}]\n",
            "adds   w17, w17, #1\n",
            "csinv  w17, w17, wzr, cc\n",
            "str    w17, [x16, #{arith_metadata_exec_count_offset}]\n",
        )
    };
}

#[cfg(test)]
mod inline_write_macro_tests {
    #[test]
    fn branch_named_own_inline_write_mode_emits_mode_5_check() {
        let asm = branch_named_own_inline_write_mode!(9, miss);
        assert!(asm.contains("ldrb   w16, [x9, {feedback_mode}]"));
        assert!(asm.contains("cmp    w16, #5"));
        assert!(asm.contains("b.ne"));
    }

    #[test]
    fn load_named_target_shape_emits_aux_bits_load() {
        let asm = load_named_target_shape!(9 => 10);
        assert!(asm.contains("ldr    w10, [x9, {feedback_named_aux_bits}]"));
    }
}
