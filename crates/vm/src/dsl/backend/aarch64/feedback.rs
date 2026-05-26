//! Inline-cache feedback macros.
//!
//! Phase C Task 4.3: `load_feedback_site!` is rewritten to resolve through
//! the `MetadataTable` buffer. `x21` (FV pin) continues to hold the flat
//! `FeedbackEntry` base so that `record_smi!` / `record_object!` /
//! `record_double!` (parallel writes live through Phase C) are undisturbed.
//! `load_feedback_site!` loads `state.frame_metadata_table_base` itself via
//! `x24` (STATE pin) and `{state_mt}`, so no pinned-register change is needed.
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
//! - `{entry_stride_shift}` — `log2(size_of::<FeedbackEntry>())` = 6 (legacy; used
//!   by `record_smi!` / `record_object!` / `record_double!`).
//! - `{feedback_mode}` — byte offset of the `LLInt` IC mode byte.
//! - `{feedback_named_handler_bits}` — byte offset of the packed named
//!   property handler word.
//! - `{feedback_named_aux_bits}` — byte offset of the auxiliary
//!   named-property handler word.
//! - `{entry_observed}` — byte offset of the "observed types" word
//!   inside `FeedbackEntry`.
//! - `{feedback_scalar_execution_count}` — byte offset of the pending
//!   scalar feedback execution count inside `FeedbackEntry`.

/// Resolve a 1-based `FeedbackSlotId` to a `*mut PropertyMetadata` entry and
/// write the pointer into `x{$dst}`. x24 (STATE pin) must be live; the macro
/// loads `state.frame_metadata_table_base` itself so x21 (FV pin) remains
/// undisturbed for `record_smi!` / `record_object!` / `record_double!` writes
/// to the legacy flat FeedbackEntry storage.
///
/// Hardcoded for `Property` kind (index 0). The kind-offsets entry for
/// Property is the first `u32` in the kind-offsets table, so
/// `kind_offsets[Property] = *(buffer + {mt_kind_offsets_offset})` with no
/// index multiplication.
///
/// Scratch: x16, x17 (AAPCS64 IP0/IP1 — never overlap live operand slots).
///
/// 7-instruction sequence:
/// 0. `ldr  x{dst}, [x24, #{state_mt}]`                    — MT = state.frame_metadata_table_base
/// 1. `sub  x17, x{slot}, #1`                              — 0-based slot index
/// 2. `add  x16, x{dst}, #{mt_slot_index_table_offset}`    — base of slot→idx table
/// 3. `ldr  w16, [x16, x17, lsl #2]`                       — idx = table[slot-1]
/// 4. `ldr  w17, [x{dst}, #{mt_kind_offsets_offset}]`      — koff = kind_offsets[Property]
/// 5. `add  x{dst}, x{dst}, x17`                           — Property run base
/// 6. `add  x{dst}, x{dst}, x16, lsl #{property_metadata_stride_shift}` — + idx*32
#[macro_export]
macro_rules! load_feedback_site {
    ($slot:tt => $dst:tt) => {
        concat!(
            // (0) Load MetadataTable buffer pointer from state. Using x{dst} as the
            // MT base register avoids clobbering x21 (FV pin used by record_* macros).
            "ldr    x",
            stringify!($dst),
            ", [x24, #{state_mt}]\n",
            // (1) Convert 1-based slot id to 0-based index.
            "sub    x17, x",
            stringify!($slot),
            ", #1\n",
            // (2) Point x16 at the slot→in-kind-index table.
            "add    x16, x",
            stringify!($dst),
            ", #{mt_slot_index_table_offset}\n",
            // (3) Load the in-kind index for this slot (u32).
            "ldr    w16, [x16, x17, lsl #2]\n",
            // (4) Load kind_offsets[Property] = first u32 in the kind-offsets table.
            "ldr    w17, [x",
            stringify!($dst),
            ", #{mt_kind_offsets_offset}]\n",
            // (5) Property run base = buffer + kind_offset.
            "add    x",
            stringify!($dst),
            ", x",
            stringify!($dst),
            ", x17\n",
            // (6) Entry pointer = run_base + in_kind_index * stride.
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
/// the pending scalar execution count. Used by inline scalar `LLInt` hits.
#[macro_export]
macro_rules! record_smi {
    ($slot:tt) => {
        concat!(
            "sub    x17, x",
            stringify!($slot),
            ", #1\n",
            "add    x16, x21, x17, lsl #{entry_stride_shift}\n",
            "ldr    w17, [x16, {entry_observed}]\n",
            "orr    w17, w17, #0x1\n",
            "str    w17, [x16, {entry_observed}]\n",
            "ldr    w17, [x16, {feedback_scalar_execution_count}]\n",
            "adds   w17, w17, #1\n",
            "csinv  w17, w17, wzr, cc\n",
            "str    w17, [x16, {feedback_scalar_execution_count}]\n",
        )
    };
}

/// Record an Object observation. SMI bit 0; Object bit 1.
#[macro_export]
macro_rules! record_object {
    ($slot:tt) => {
        concat!(
            "sub    x17, x",
            stringify!($slot),
            ", #1\n",
            "add    x16, x21, x17, lsl #{entry_stride_shift}\n",
            "ldr    w17, [x16, {entry_observed}]\n",
            "orr    w17, w17, #0x2\n",
            "str    w17, [x16, {entry_observed}]\n",
            "ldr    w17, [x16, {feedback_scalar_execution_count}]\n",
            "adds   w17, w17, #1\n",
            "csinv  w17, w17, wzr, cc\n",
            "str    w17, [x16, {feedback_scalar_execution_count}]\n",
        )
    };
}

/// Record a Double observation (bit 2 of observed types).
#[macro_export]
macro_rules! record_double {
    ($slot:tt) => {
        concat!(
            "sub    x17, x",
            stringify!($slot),
            ", #1\n",
            "add    x16, x21, x17, lsl #{entry_stride_shift}\n",
            "ldr    w17, [x16, {entry_observed}]\n",
            "orr    w17, w17, #0x4\n",
            "str    w17, [x16, {entry_observed}]\n",
            "ldr    w17, [x16, {feedback_scalar_execution_count}]\n",
            "adds   w17, w17, #1\n",
            "csinv  w17, w17, wzr, cc\n",
            "str    w17, [x16, {feedback_scalar_execution_count}]\n",
        )
    };
}
