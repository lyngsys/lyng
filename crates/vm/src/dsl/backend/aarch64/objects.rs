//! Object-record access via `ObjectRef` handles.
//!
//! Per the value-layout report §"Irreducible deltas vs `LLInt`", every
//! object load is a **two-load indirection**: the `ObjectRef` (u32) is
//! a side-table index into `LlIntState.object_records_base`, which
//! yields a `*const RuntimeObjectRecord`; the record then carries the
//! shape id and inline / outline slots. When the pointer-identity-cell
//! refactor lands (design §9 DSL-3), these macros are renamed
//! `load_cell_*!` and emit one fewer instruction each.
//!
//! Bindings expected from the proc-macro lowerer:
//!
//! - `{state_object_records}` — offset of
//!   `LlIntState::object_records_base`.
//! - `{state_object_slots}` — offset of
//!   `LlIntState::object_slots_base`.
//! - `{object_shape}` — offset of the raw `Option<ShapeId>` word
//!   inside `RuntimeObjectRecord`.
//! - `{object_prototype}` — offset of the raw `Option<ObjectRef>` word
//!   inside `RuntimeObjectRecord`.
//! - `{object_inline_slots}` — offset of the first inline named slot.
//!
//! Scratch convention: macros use `x16`/`x17` only.

/// Resolve an `ObjectRef` handle (id in `$ref`) to a `*const
/// RuntimeObjectRecord` pointer in `$dst`. Branches to `$label` if
/// the table base or table entry is null.
#[macro_export]
macro_rules! load_object_record_from_state_or_branch {
    ($ref:tt => $dst:tt, $label:tt) => {
        concat!(
            "ldr    x16, [x24, {state_object_records}]\n",
            "cbz    x16, ",
            stringify!($label),
            "\n",
            "ldr    x",
            stringify!($dst),
            ", [x16, x",
            stringify!($ref),
            ", lsl #3]\n",
            "cbz    x",
            stringify!($dst),
            ", ",
            stringify!($label),
            "\n",
        )
    };
}

/// Load the shape pointer from an `ObjectRecord` at `$rec` into `$dst`.
///
/// The shape word is a 32-bit Shape ID; we load it as `w` and let the
/// caller widen if needed.
#[macro_export]
macro_rules! load_record_shape {
    ($rec:tt => $dst:tt) => {
        concat!(
            "ldr    w",
            stringify!($dst),
            ", [x",
            stringify!($rec),
            ", {object_shape}]\n",
        )
    };
}

/// Load the prototype `ObjectRef` handle from an `ObjectRecord` at `$rec`
/// into `$dst`, branching to `$label` when the receiver has no
/// prototype. The raw `Option<ObjectRef>` representation is a u32
/// non-zero handle with zero as `None`.
#[macro_export]
macro_rules! load_record_prototype_or_branch {
    ($rec:tt => $dst:tt, $label:tt) => {
        concat!(
            "ldr    w",
            stringify!($dst),
            ", [x",
            stringify!($rec),
            ", {object_prototype}]\n",
            "cbz    x",
            stringify!($dst),
            ", ",
            stringify!($label),
            "\n",
        )
    };
}

/// Load inline slot `$idx` from an `ObjectRecord` at `$rec` into `$dst`
/// as a `Value` (full 64 bits).
#[macro_export]
macro_rules! load_record_inline_slot {
    ($rec:tt, $idx:tt => $dst:tt) => {
        concat!(
            "add    x16, x",
            stringify!($rec),
            ", {object_inline_slots}\n",
            "ldr    x",
            stringify!($dst),
            ", [x16, x",
            stringify!($idx),
            ", lsl #3]\n",
        )
    };
}

/// Store `$src` into inline slot `$idx` of an `ObjectRecord` at `$rec`.
#[macro_export]
macro_rules! store_record_inline_slot {
    ($rec:tt, $idx:tt, $src:tt) => {
        concat!(
            "add    x16, x",
            stringify!($rec),
            ", {object_inline_slots}\n",
            "str    x",
            stringify!($src),
            ", [x16, x",
            stringify!($idx),
            ", lsl #3]\n",
        )
    };
}

/// Resolve an `ObjectRecord`'s named-slot handle to an outline-slots
/// base pointer in `$dst`. Branches to `$label` when the object has no
/// named-slot storage or the pointer table has no live entry.
#[macro_export]
macro_rules! load_record_outline_slots_from_state_or_branch {
    ($rec:tt => $dst:tt, $label:tt) => {
        concat!(
            "ldr    w",
            stringify!($dst),
            ", [x",
            stringify!($rec),
            ", {object_named_slots}]\n",
            "cbz    x",
            stringify!($dst),
            ", ",
            stringify!($label),
            "\n",
            "ldr    x16, [x24, {state_object_slots}]\n",
            "cbz    x16, ",
            stringify!($label),
            "\n",
            "ldr    x",
            stringify!($dst),
            ", [x16, x",
            stringify!($dst),
            ", lsl #3]\n",
            "cbz    x",
            stringify!($dst),
            ", ",
            stringify!($label),
            "\n",
        )
    };
}

/// Load outline slot `$idx` given an outline-slots base pointer in
/// `$base` (typically the result of
/// `load_record_outline_slots_from_state_or_branch!`).
#[macro_export]
macro_rules! load_outline_slot {
    ($base:tt, $idx:tt => $dst:tt) => {
        concat!(
            "ldr    x",
            stringify!($dst),
            ", [x",
            stringify!($base),
            ", x",
            stringify!($idx),
            ", lsl #3]\n",
        )
    };
}
