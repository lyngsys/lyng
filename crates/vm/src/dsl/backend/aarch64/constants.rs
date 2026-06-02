//! Constants-access backend macro.
//!
//! [`load_constant!`] loads a [`lyng_types::Value`] from the
//! pre-resolved constants array via
//! [`LlIntState::frame_const_base`](crate::dsl::llint_state::LlIntState::frame_const_base).
//!
//! `frame_const_base` is populated at trampoline entry and refreshed on
//! every Refresh egress; the pointer is stable within a Refresh window.
//!
//! ## Emitted shape (2 instructions)
//!
//! ```text
//!     ldr  x16, [x24, {vm_const_base}]        ; x16 = frame_const_base (*const Value)
//!     ldr  {dst}, [x16, {idx}, lsl #3]        ; dst = base[idx] (Value is 8B → lsl #3)
//! ```
//!
//! The base pointer lives on `LlIntState` (STATE pin = x24); the binding
//! `{vm_const_base}` resolves to `LLINT_STATE_FRAME_CONST_BASE` despite
//! its name (parallel to `vm_counter_base` which does live on `Vm`).
//!
//! ## Scratch-register convention
//!
//! - **x16** (AAPCS64 IP0): macro-internal scratch for the loaded base pointer.
//!   Not preserved across the macro.
//! - `$idx_reg`: operand-scratch holding the constant index (x9..x15).
//! - `$dst_reg`: operand-scratch that receives the loaded Value.

/// Load a [`lyng_types::Value`] from `frame_const_base[$idx_reg]` into
/// `$dst_reg`. Two instructions; uses x16 (IP0) as scratch.
///
/// ```ignore
/// load_constant!(idx => dst);
/// ```
#[macro_export]
macro_rules! load_constant {
    ($idx_reg:tt => $dst_reg:tt) => {
        concat!(
            "ldr    x16, [x24, {vm_const_base}]\n",
            "ldr    x",
            stringify!($dst_reg),
            ", [x16, x",
            stringify!($idx_reg),
            ", lsl #3]\n",
        )
    };
}
