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
//! Bindings expected from the proc-macro lowerer:
//!
//! - `{state_mt}` — byte offset of `LlIntState::frame_metadata_table_base`.
//! - `{mt_kind_offsets_offset}` — byte offset of the kind-offsets table in the
//!   MetadataTable buffer (= 16 after the 16-byte `LinkingDataHeader`).
//! - `{mt_slot_index_table_offset}` — byte offset of the slot→in-kind-index
//!   table in the MetadataTable buffer (= 36).
//! - `{property_metadata_stride_shift}` — `log2(size_of::<PropertyMetadata>())` = 5.
//! - `{mt_arith_kind_offset}` — byte offset of `kind_offsets[Arith]` in the
//!   MetadataTable buffer (= 24; header=16, Arith index=2, 4 bytes/entry).
//! - `{arith_metadata_observed_bits_offset}` — byte offset of `ArithMetadata::observed_bits` = 0.
//! - `{arith_metadata_exec_count_offset}` — byte offset of `ArithMetadata::execution_count` = 4.
//! - `{arith_metadata_stride_shift}` — `log2(size_of::<ArithMetadata>())` = 3.
//! - `{feedback_mode}` — byte offset of the `LLInt` IC mode byte.
//! - `{feedback_named_handler_bits}` — byte offset of the packed named
//!   property handler word.
//! - `{feedback_named_aux_bits}` — byte offset of the auxiliary
//!   named-property handler word.

/// Resolve a 1-based `FeedbackSlotId` to a `*mut PropertyMetadata` entry and
/// write the pointer into `x{$dst}`. x21 (MT pin) must hold the MetadataTable
/// buffer base (Phase C.4 pin flip).
///
/// Hardcoded for `Property` kind (index 0). The kind-offsets entry for
/// Property is the first `u32` in the kind-offsets table, so
/// `kind_offsets[Property] = *(buffer + {mt_kind_offsets_offset})` with no
/// index multiplication.
///
/// Scratch: x16, x17 (AAPCS64 IP0/IP1 — never overlap live operand slots).
///
/// 5-instruction sequence (x21 = MetadataTable buffer base):
/// 0. `sub  x17, x{slot}, #1`                              — 0-based slot index
/// 1. `add  x16, x21, #{mt_slot_index_table_offset}`       — base of slot→idx table
/// 2. `ldr  w16, [x16, x17, lsl #2]`                       — idx = table[slot-1]
/// 3. `ldr  w17, [x21, #{mt_kind_offsets_offset}]`         — koff = kind_offsets[Property]
/// 4. `add  x{dst}, x21, x17`                              — Property run base
/// 5. `add  x{dst}, x{dst}, x16, lsl #{property_metadata_stride_shift}` — + idx*32
#[macro_export]
macro_rules! load_feedback_site {
    ($slot:tt => $dst:tt) => {
        concat!(
            // (0) Convert 1-based slot id to 0-based index.
            "sub    x17, x",
            stringify!($slot),
            ", #1\n",
            // (1) Point x16 at the slot→in-kind-index table (x21 = MT buffer).
            "add    x16, x21, #{mt_slot_index_table_offset}\n",
            // (2) Load the in-kind index for this slot (u32).
            "ldr    w16, [x16, x17, lsl #2]\n",
            // (3) Load kind_offsets[Property] = first u32 in the kind-offsets table.
            "ldr    w17, [x21, #{mt_kind_offsets_offset}]\n",
            // (4) Property run base = mt_buffer + kind_offset.
            "add    x",
            stringify!($dst),
            ", x21, x17\n",
            // (5) Entry pointer = run_base + in_kind_index * stride.
            "add    x",
            stringify!($dst),
            ", x",
            stringify!($dst),
            ", x16, lsl #{property_metadata_stride_shift}\n",
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

/// Branch to `$label` unless the flat feedback entry is a polymorphic
/// `OwnData` inline-slot load header. The pair of cached handlers / epochs
/// is packed into the existing primary (slot 0) and auxiliary (slot 1)
/// fields by [`FeedbackEntry::set_named_own_polymorphic`].
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

/// Record that an SMI was observed at slot `$slot` and saturating-increment
/// the pending scalar execution count. Uses x21 as MetadataTable base (Phase C.4).
///
/// Resolves ArithMetadata pointer via: slot_to_in_kind table → kind_offsets[Arith] → entry.
/// Writes to `ArithMetadata.{observed_bits, execution_count}` (offsets 0 and 4).
///
/// 13-instruction sequence (x16/x17 scratch):
/// - `0.` sub  x17, x{slot}, #1
/// - `1.` add  x16, x21, #{mt_slot_index_table_offset}
/// - `2.` ldr  w16, [x16, x17, lsl #2]
/// - `3.` ldr  w17, [x21, #{mt_arith_kind_offset}]
/// - `4.` add  x17, x21, x17
/// - `5.` add  x16, x17, x16, lsl #{arith_metadata_stride_shift}
/// - `6.` ldr  w17, [x16, #{arith_metadata_observed_bits_offset}]
/// - `7.` orr  w17, w17, #0x1
/// - `8.` str  w17, [x16, #{arith_metadata_observed_bits_offset}]
/// - `9.` ldr  w17, [x16, #{arith_metadata_exec_count_offset}]
/// - `10.` adds w17, w17, #1
/// - `11.` csinv w17, w17, wzr, cc
/// - `12.` str  w17, [x16, #{arith_metadata_exec_count_offset}]
#[macro_export]
macro_rules! record_smi {
    ($slot:tt) => {
        concat!(
            // (0) 0-based slot index.
            "sub    x17, x",
            stringify!($slot),
            ", #1\n",
            // (1) Slot→in-kind-index table base (x21 = MetadataTable buffer).
            "add    x16, x21, #{mt_slot_index_table_offset}\n",
            // (2) in-kind index = slot_to_in_kind[slot-1].
            "ldr    w16, [x16, x17, lsl #2]\n",
            // (3) kind_offsets[Arith] = *(mt + mt_arith_kind_offset).
            "ldr    w17, [x21, #{mt_arith_kind_offset}]\n",
            // (4) Arith run base = mt + kind_offsets[Arith].
            "add    x17, x21, x17\n",
            // (5) Entry pointer = run_base + in_kind_index * stride.
            "add    x16, x17, x16, lsl #{arith_metadata_stride_shift}\n",
            // (6-8) Update observed_bits |= SMI bit (0x1).
            "ldr    w17, [x16, #{arith_metadata_observed_bits_offset}]\n",
            "orr    w17, w17, #0x1\n",
            "str    w17, [x16, #{arith_metadata_observed_bits_offset}]\n",
            // (9-C) Saturating-increment execution_count.
            "ldr    w17, [x16, #{arith_metadata_exec_count_offset}]\n",
            "adds   w17, w17, #1\n",
            "csinv  w17, w17, wzr, cc\n",
            "str    w17, [x16, #{arith_metadata_exec_count_offset}]\n",
        )
    };
}

/// Record an Object observation (bit 1). SMI bit 0; Object bit 1.
/// Uses x21 as MetadataTable base (Phase C.4). See `record_smi!` for layout.
#[macro_export]
macro_rules! record_object {
    ($slot:tt) => {
        concat!(
            // (0) 0-based slot index.
            "sub    x17, x",
            stringify!($slot),
            ", #1\n",
            // (1) Slot→in-kind-index table base.
            "add    x16, x21, #{mt_slot_index_table_offset}\n",
            // (2) in-kind index.
            "ldr    w16, [x16, x17, lsl #2]\n",
            // (3) kind_offsets[Arith].
            "ldr    w17, [x21, #{mt_arith_kind_offset}]\n",
            // (4) Arith run base.
            "add    x17, x21, x17\n",
            // (5) Entry pointer.
            "add    x16, x17, x16, lsl #{arith_metadata_stride_shift}\n",
            // (6-8) Update observed_bits |= Object bit (0x2).
            "ldr    w17, [x16, #{arith_metadata_observed_bits_offset}]\n",
            "orr    w17, w17, #0x2\n",
            "str    w17, [x16, #{arith_metadata_observed_bits_offset}]\n",
            // (9-C) Saturating-increment execution_count.
            "ldr    w17, [x16, #{arith_metadata_exec_count_offset}]\n",
            "adds   w17, w17, #1\n",
            "csinv  w17, w17, wzr, cc\n",
            "str    w17, [x16, #{arith_metadata_exec_count_offset}]\n",
        )
    };
}

/// Record a Double observation (bit 2 of observed types).
/// Uses x21 as MetadataTable base (Phase C.4). See `record_smi!` for layout.
#[macro_export]
macro_rules! record_double {
    ($slot:tt) => {
        concat!(
            // (0) 0-based slot index.
            "sub    x17, x",
            stringify!($slot),
            ", #1\n",
            // (1) Slot→in-kind-index table base.
            "add    x16, x21, #{mt_slot_index_table_offset}\n",
            // (2) in-kind index.
            "ldr    w16, [x16, x17, lsl #2]\n",
            // (3) kind_offsets[Arith].
            "ldr    w17, [x21, #{mt_arith_kind_offset}]\n",
            // (4) Arith run base.
            "add    x17, x21, x17\n",
            // (5) Entry pointer.
            "add    x16, x17, x16, lsl #{arith_metadata_stride_shift}\n",
            // (6-8) Update observed_bits |= Double bit (0x4).
            "ldr    w17, [x16, #{arith_metadata_observed_bits_offset}]\n",
            "orr    w17, w17, #0x4\n",
            "str    w17, [x16, #{arith_metadata_observed_bits_offset}]\n",
            // (9-C) Saturating-increment execution_count.
            "ldr    w17, [x16, #{arith_metadata_exec_count_offset}]\n",
            "adds   w17, w17, #1\n",
            "csinv  w17, w17, wzr, cc\n",
            "str    w17, [x16, #{arith_metadata_exec_count_offset}]\n",
        )
    };
}
