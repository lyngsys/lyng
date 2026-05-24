//! Inline-cache feedback macros.
//!
//! The `FV` pin (`x21`) holds the base of the current
//! `Box<[FeedbackEntry]>` for this function's feedback vector. A slot
//! index shifted by the compact entry stride yields a `*mut FeedbackEntry`
//! the macros can read / write.
//!
//! All internal scratch use is on `x16` / `x17` (see
//! `values.rs` for the rationale).
//!
//! Bindings expected from the proc-macro lowerer:
//!
//! - `{entry_stride_shift}` — `log2(size_of::<FeedbackEntry>())`.
//! - `{feedback_mode}` — byte offset of the `LLInt` IC mode byte.
//! - `{feedback_named_handler_bits}` — byte offset of the packed named
//!   property handler word.
//! - `{feedback_named_epoch}` — byte offset of the named-property
//!   invalidation epoch snapshot.
//! - `{feedback_named_aux_bits}` — byte offset of the auxiliary
//!   named-property handler word.
//! - `{feedback_named_aux_epoch}` — byte offset of the auxiliary
//!   named-property invalidation epoch snapshot.
//! - `{entry_observed}` — byte offset of the "observed types" word
//!   inside `FeedbackEntry`.
//! - `{feedback_scalar_execution_count}` — byte offset of the pending
//!   scalar feedback execution count inside `FeedbackEntry`.

/// Compute a pointer to the `FeedbackEntry` at slot `$slot` and write
/// it into `$dst`. Feedback slot ids are one-based, so the computed
/// flat-array index is `slot - 1`.
#[macro_export]
macro_rules! load_feedback_site {
    ($slot:tt => $dst:tt) => {
        concat!(
            "sub    x17, x",
            stringify!($slot),
            ", #1\n",
            "add    x",
            stringify!($dst),
            ", x21, x17, lsl #{entry_stride_shift}\n",
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

#[macro_export]
macro_rules! load_named_epoch {
    ($entry:tt => $dst:tt) => {
        concat!(
            "ldr    x",
            stringify!($dst),
            ", [x",
            stringify!($entry),
            ", {feedback_named_epoch}]\n",
        )
    };
}

#[macro_export]
macro_rules! load_named_aux_epoch {
    ($entry:tt => $dst:tt) => {
        concat!(
            "ldr    x",
            stringify!($dst),
            ", [x",
            stringify!($entry),
            ", {feedback_named_aux_epoch}]\n",
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
