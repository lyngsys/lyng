//! Constants-access backend macro for Phase 1.B.1.
//!
//! [`load_constant!`] loads a [`lyng_types::Value`] from the
//! pre-resolved constants array via
//! [`LlIntState::frame_const_base`](crate::dsl::llint_state::LlIntState::frame_const_base).
//!
//! The `frame_const_base` pointer is populated at trampoline entry
//! (`entry.rs::run_via_dsl`) and refreshed on every slow-path Refresh
//! egress (`slow_path.rs::translate_outcome`). Handler code reads
//! through it within a Refresh-to-Refresh window — between bridges the
//! arena slot is pointer-stable (same precedent as `frame_pb_base`).
//!
//! ## Emitted shape (2 instructions)
//!
//! ```text
//!     ldr  x16, [x24, {vm_const_base}]        ; x16 = frame_const_base (*const Value)
//!     ldr  {dst}, [x16, {idx}, lsl #3]        ; dst = base[idx] (Value is 8B → lsl #3)
//! ```
//!
//! The base pointer lives on `LlIntState` (the STATE pin = x24), not
//! `Vm` (the VM pin = x22) — `LLINT_STATE_FRAME_CONST_BASE` is
//! `offset_of!(LlIntState, frame_const_base)`. Earlier drafts of this
//! macro emitted `[x22, …]` because the binding is named
//! `vm_const_base` for historical reasons (parallel to
//! `vm_counter_base` which does live on `Vm`); the offset itself was
//! always LlIntState-relative. Phase 1.B.2 promotes the macro from
//! "compiles only" (structural validation tests, opcode 210 never
//! dispatches) to "runs in op_load_const8", which exposes the bug.
//! Fixed here.
//!
//! ## Scratch-register convention
//!
//! - **x16** (AAPCS64 IP0) is the macro-internal scratch holding the
//!   loaded base pointer. Per `reg_convention.rs`, IP0/IP1 are reserved
//!   for backend macros and never overlap the lowerer's `t0..t6`
//!   operand pool. Callers must not assume `x16` is preserved across
//!   this macro.
//! - `$idx_reg` is the operand-scratch register number holding the
//!   constant-index value (already in x9..x15 thanks to the lowerer's
//!   ident substitution).
//! - `$dst_reg` is the operand-scratch register number that will hold
//!   the loaded Value.
//!
//! ## Named binding supplied by the lowerer
//!
//! - `{vm_const_base}` resolves to
//!   [`reg_convention::LLINT_STATE_FRAME_CONST_BASE`](crate::dsl::reg_convention::LLINT_STATE_FRAME_CONST_BASE).
//!
//! See spec §3.5 and `crates/vm-dsl/src/lower.rs` for the
//! injection.

/// Load a [`lyng_types::Value`] (8 bytes) from
/// `frame_const_base[$idx_reg]` into `$dst_reg`.
///
/// Two instructions; uses x16 (IP0) as macro-internal scratch.
///
/// Usage from a handler body (operand idents `idx`, `dst` are
/// substituted by the lowerer to register-number literals):
///
/// ```ignore
/// load_constant!(idx => dst);
/// ```
///
/// See module docs for the emitted asm shape and binding details.
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
