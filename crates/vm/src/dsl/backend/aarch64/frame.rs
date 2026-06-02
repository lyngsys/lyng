//! Fixed-offset [`lyng_types::Value`] load from
//! [`LlIntState`](crate::dsl::llint_state::LlIntState) through the
//! STATE pin (x24).
//!
//! ## Emitted shape (1 instruction)
//!
//! ```text
//!     ldr  {dst}, [x24, {state_offset}]
//! ```
//!
//! ## Argument conventions
//!
//! - `$dst_reg`: operand-scratch register number (lowerer `t0..t6` → `9..15`).
//! - `vm_state_offset = $binding`: a `naked_asm!`-supplied named binding
//!   resolving to an `LLINT_STATE_*` byte offset (e.g. `state_this_value`).
//!   Add new bindings to the lowerer's universal binding set to expose
//!   additional `LlIntState` Value-typed fields.

/// Load a [`lyng_types::Value`] from a fixed offset in `LlIntState` into
/// `$dst_reg`. One instruction: `ldr x{dst}, [x24, {<binding>}]`.
///
/// ```ignore
/// load_state_value!(dst, vm_state_offset = state_this_value);
/// ```
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
