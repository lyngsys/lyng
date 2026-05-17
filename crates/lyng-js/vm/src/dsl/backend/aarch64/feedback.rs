//! Inline-cache feedback macros.
//!
//! The `FV` pin (`x21`) holds the base of the current
//! `Box<[FeedbackEntry]>` for this function's feedback vector. A slot
//! index multiplied by the entry stride yields a `*mut FeedbackEntry`
//! the macros can read / write.
//!
//! Bindings expected from the proc-macro lowerer:
//!
//! - `{entry_stride_shift}` — log2 of `size_of::<FeedbackEntry>()`.
//!   `FeedbackEntry = Option<FeedbackSiteState>`; the size is
//!   determined at proc-macro lower time and emitted as a literal
//!   shift count. For DSL-0b we use a placeholder `6` (= 64 bytes)
//!   pending exact size measurement when the first IC handler lands.
//! - `{entry_observed}` — byte offset of the "observed types" word
//!   inside `FeedbackEntry`. Resolved when the recording handlers
//!   land in Batch 6.

/// Compute a pointer to the `FeedbackEntry` at slot `$slot` and write
/// it into `$dst`. Compiles to 2 instructions.
#[macro_export]
macro_rules! load_feedback_site {
    ($slot:ident => $dst:ident) => {
        concat!(
            "lsl    x9, x", stringify!($slot), ", {entry_stride_shift}\n",
            "add    x", stringify!($dst), ", x21, x9\n",
        )
    };
}

/// Record that an SMI was observed at slot `$slot` (OR-in the SMI bit
/// of the observed-types word). Used by warmup recording handlers.
#[macro_export]
macro_rules! record_smi {
    ($slot:ident) => {
        concat!(
            "lsl    x9, x", stringify!($slot), ", {entry_stride_shift}\n",
            "add    x9, x21, x9\n",
            "ldr    w10, [x9, {entry_observed}]\n",
            "orr    w10, w10, #0x1\n",
            "str    w10, [x9, {entry_observed}]\n",
        )
    };
}

/// Record an Object observation. SMI bit 0; Object bit 1.
#[macro_export]
macro_rules! record_object {
    ($slot:ident) => {
        concat!(
            "lsl    x9, x", stringify!($slot), ", {entry_stride_shift}\n",
            "add    x9, x21, x9\n",
            "ldr    w10, [x9, {entry_observed}]\n",
            "orr    w10, w10, #0x2\n",
            "str    w10, [x9, {entry_observed}]\n",
        )
    };
}

/// Record a Double observation (bit 2 of observed types).
#[macro_export]
macro_rules! record_double {
    ($slot:ident) => {
        concat!(
            "lsl    x9, x", stringify!($slot), ", {entry_stride_shift}\n",
            "add    x9, x21, x9\n",
            "ldr    w10, [x9, {entry_observed}]\n",
            "orr    w10, w10, #0x4\n",
            "str    w10, [x9, {entry_observed}]\n",
        )
    };
}
