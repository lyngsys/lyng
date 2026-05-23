//! Object-record access via `ObjectRef` handles.
//!
//! Per the value-layout report §"Irreducible deltas vs LLInt", every
//! object load is a **two-load indirection**: the `ObjectRef` (u32) is
//! a side-table index into the VM's `heap_pool_base`, which yields a
//! `*const ObjectRecord`; the record then carries the shape pointer
//! and inline / outline slots. When the pointer-identity-cell refactor
//! lands (design §9 DSL-3), these macros are renamed `load_cell_*!`
//! and emit one fewer instruction each.
//!
//! Bindings expected from the proc-macro lowerer:
//!
//! - `{vm_heap_pool}` — offset of `Vm::heap_pool_base` inside `Vm`
//!   (provided by `reg_convention::VM_HEAP_POOL_OFFSET`, currently a
//!   placeholder until Task B41 lands the real field).
//! - `{record_shape}` — offset of the shape pointer inside
//!   `ObjectRecord`. Concrete value lands when the record-shape
//!   loader is first invoked (Batch 6).
//! - `{record_inline_slots}` — offset of the first inline slot.
//! - `{record_outline_slots}` — offset of the outline-slots pointer.
//!
//! Scratch convention: `x9` is owned by the macro for arithmetic.

/// Resolve an `ObjectRef` handle (id in `$ref`) to a `*const
/// ObjectRecord` pointer in `$dst`. Two-load indirection — see
/// module doc.
#[macro_export]
macro_rules! load_object_record {
    ($ref:ident => $dst:ident) => {
        concat!(
            // x9 := VM->heap_pool_base
            "ldr    x9, [x22, {vm_heap_pool}]\n",
            // dst := heap_pool_base[ref] (each entry is 8 bytes)
            "ldr    x", stringify!($dst), ", [x9, x", stringify!($ref), ", lsl #3]\n",
        )
    };
}

/// Load the shape pointer from an ObjectRecord at `$rec` into `$dst`.
///
/// The shape word is a 32-bit Shape ID; we load it as `w` and let the
/// caller widen if needed.
#[macro_export]
macro_rules! load_record_shape {
    ($rec:ident => $dst:ident) => {
        concat!(
            "ldr    w", stringify!($dst), ", [x", stringify!($rec), ", {record_shape}]\n",
        )
    };
}

/// Load inline slot `$idx` from an ObjectRecord at `$rec` into `$dst`
/// as a `Value` (full 64 bits).
#[macro_export]
macro_rules! load_record_inline_slot {
    ($rec:ident, $idx:ident => $dst:ident) => {
        concat!(
            // x9 := record_base + inline_slots_offset
            "add    x9, x", stringify!($rec), ", {record_inline_slots}\n",
            // dst := *(x9 + idx * 8)
            "ldr    x", stringify!($dst), ", [x9, x", stringify!($idx), ", lsl #3]\n",
        )
    };
}

/// Store `$src` into inline slot `$idx` of an ObjectRecord at `$rec`.
#[macro_export]
macro_rules! store_record_inline_slot {
    ($rec:ident, $idx:ident, $src:ident) => {
        concat!(
            "add    x9, x", stringify!($rec), ", {record_inline_slots}\n",
            "str    x", stringify!($src), ", [x9, x", stringify!($idx), ", lsl #3]\n",
        )
    };
}

/// Load the outline-slots pointer from an ObjectRecord at `$rec` into
/// `$dst`. Outline slots live in a separately-allocated `Vec<Value>`
/// whose base pointer hangs off the record.
#[macro_export]
macro_rules! load_record_outline_slots {
    ($rec:ident => $dst:ident) => {
        concat!(
            "ldr    x", stringify!($dst), ", [x", stringify!($rec), ", {record_outline_slots}]\n",
        )
    };
}

/// Load outline slot `$idx` given an outline-slots base pointer in
/// `$base` (typically the result of `load_record_outline_slots!`).
#[macro_export]
macro_rules! load_outline_slot {
    ($base:ident, $idx:ident => $dst:ident) => {
        concat!(
            "ldr    x", stringify!($dst), ", [x", stringify!($base), ", x", stringify!($idx), ", lsl #3]\n",
        )
    };
}
