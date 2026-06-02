//! Fixed-immediate-index register-window load/store macros.
//!
//! `op_load_local_N` / `op_store_local_N` handlers hardcode the slot
//! index. Using `load_reg!`/`store_reg!` with a runtime index costs 2
//! instructions (movz + ldr/str); the fixed-offset form costs 1.
//!
//! Both macros emit a single instruction:
//!
//! ```text
//!     ldr  x{dst}, [x20, #{N*8}]      ; load_local_fixed!
//!     str  x{src}, [x20, #{N*8}]      ; store_local_fixed!
//! ```
//!
//! `x20` is the REGS pin (register-window base). Each slot is an 8-byte
//! [`lyng_types::Value`]; N in 0..=3 fits the `AArch64` immediate range trivially.
//!
//! ## Lowerer interaction
//!
//! The index argument is a literal, not a scratch ident. The lowerer's
//! `substitute_idents` pass replaces scratch idents but passes numeric
//! literals through unchanged.

/// Load a [`lyng_types::Value`] from register-window slot `$n` into
/// `$dst_reg`. One instruction: `ldr x{dst}, [x20, #(n * 8)]`.
///
/// ```ignore
/// load_local_fixed!(1 => dst);
/// ```
#[macro_export]
macro_rules! load_local_fixed {
    ($n:literal => $dst_reg:tt) => {
        concat!(
            "ldr    x",
            stringify!($dst_reg),
            ", [x20, #",
            stringify!($n),
            " * 8]\n",
        )
    };
}

/// Store the [`lyng_types::Value`] in `$src_reg` into the
/// register-window slot at fixed index `$n`. Mirror of
/// [`load_local_fixed!`].
///
/// One instruction: `str x{src}, [x20, #(n * 8)]`.
#[macro_export]
macro_rules! store_local_fixed {
    ($src_reg:tt, $n:literal) => {
        concat!(
            "str    x",
            stringify!($src_reg),
            ", [x20, #",
            stringify!($n),
            " * 8]\n",
        )
    };
}
