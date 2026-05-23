//! Frame-context backend macro for Phase 1.B.1.
//!
//! [`load_state_value!`] is a 1-instruction fixed-offset
//! [`lyng_types::Value`] load from
//! [`LlIntState`](crate::dsl::llint_state::LlIntState) through the
//! STATE pin (x24).
//!
//! Phase 1.B.1 uses this for
//! [`LlIntState::frame_this_value`](crate::dsl::llint_state::LlIntState::frame_this_value)
//! via the universally-bound `state_this_value` named arg. Other
//! 8-byte `Value`-typed fields on `LlIntState` could migrate to this
//! macro in a future refactor.
//!
//! ## Emitted shape (1 instruction)
//!
//! ```text
//!     ldr  {dst}, [x24, {state_offset}]
//! ```
//!
//! ## Argument conventions
//!
//! - `$dst_reg` is the operand-scratch register number (the lowerer's
//!   `t0..t6` slots have already been substituted to literals like
//!   `9..15` by the time the macro expands).
//! - `vm_state_offset = $binding` names a `naked_asm!`-supplied named
//!   binding that resolves to an `LLINT_STATE_*` byte offset. The
//!   lowerer already supplies `state_pb`, `state_fv`, `state_regs`,
//!   `state_prefix`; Phase 1.B.1 adds `state_this_value`. Future
//!   `LlIntState` Value-typed fields can be reached by adding their
//!   bindings to the lowerer's universal binding set.
//!
//! No per-handler injection is needed — the macro simply refers to a
//! universally-bound name supplied at the call site.
//!
//! See spec §3.5.

/// Load a [`lyng_types::Value`] (8 bytes) from a fixed offset in
/// [`LlIntState`](crate::dsl::llint_state::LlIntState) into `$dst_reg`.
///
/// The offset is named via the `vm_state_offset = <binding>` keyword
/// argument; `<binding>` is the name of a `naked_asm!`-supplied named
/// constant (e.g. `state_this_value`).
///
/// One instruction:
///
/// ```text
///     ldr  x{dst}, [x24, {<binding>}]
/// ```
///
/// Usage from a handler body (the lowerer substitutes operand idents
/// like `dst` to register-number literals before macro expansion):
///
/// ```ignore
/// load_state_value!(dst, vm_state_offset = state_this_value);
/// ```
///
/// See module docs for the binding set and Phase 1.B.1 spec §3.5.
#[macro_export]
macro_rules! load_state_value {
    ($dst_reg:tt, vm_state_offset = $binding:ident) => {
        concat!(
            "ldr    x",
            stringify!($dst_reg),
            ", [x24, {",
            stringify!($binding),
            "}]\n",
        )
    };
}
