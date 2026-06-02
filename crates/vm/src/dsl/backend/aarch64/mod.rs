//! `AArch64` backend for the DSL substrate.
//!
//! Module layout mirrors the operation taxonomy in `dsl/ops.md`:
//!
//! - [`operands`] — operand-byte decode + register-file access.
//! - [`values`]   — NaN-tag checks and tag manipulation.
//! - [`objects`]  — `ObjectRecord` access via `ObjectRef` handles.
//! - [`arithmetic`] — SMI arithmetic + bitwise.
//! - [`constants`] — indexed load from `frame_const_base`.
//! - [`control`]  — dispatch, branches, slow-path bridge, prefix.
//! - [`feedback`] — IC-site lookups + observed-type recording.
//! - [`frame`]    — fixed-offset `LlIntState` Value loads.
//! - [`locals`]   — fixed-immediate-index register-window load/store.
//! - [`safepoint`] — interrupt-poll macro.
//! - [`memory`]   — raw load/store fragments referenced by other macros.
//! - [`counters`] — feature-gated opcode counters.
//! - [`prelude`]  — NaN-tag mask constants + layout helpers.

pub mod arithmetic;
pub mod constants;
pub mod control;
pub mod counters;
pub mod feedback;
pub mod frame;
pub mod locals;
pub mod memory;
pub mod objects;
pub mod operands;
pub mod prelude;
pub mod safepoint;
pub mod values;

/// Top-level body builder invoked by the proc-macro. Concatenates the
/// operand-decode prologue, body fragments, and dispatch trailer into a
/// single `core::arch::naked_asm!` block. The proc-macro lowerer
/// (`lyng-vm-dsl::lower`) supplies the named bindings (`{length}`,
/// `{shim}`, `{state_pc}`, etc.) that backend macros reference by name.
#[macro_export]
macro_rules! __llint_handler_body {
    (
        layout = $layout:ident,
        operands = [$($op:ident),*],
        length = $length:literal,
        body = { $($body:tt)* }
    ) => {
        // Single naked_asm! block containing:
        //   1. Operand-decode prologue (per layout).
        //   2. Body fragments.
        //   3. Dispatch trailer (auto-appended if not present in body).
        ::core::arch::naked_asm!(
            // Prologue placeholder; replaced per layout by the lowerer.
            "// prologue: layout = ", stringify!($layout),
            // Body fragments (each DSL op produces a string literal):
            $($body)*
            options(noreturn),
        )
    };
}
pub use __llint_handler_body;
