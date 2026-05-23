//! Fixed-immediate-index register-window load/store macros for DSL-1
//! Phase 1.B.3.
//!
//! `op_load_local_N` and `op_store_local_N` (N in 0..3) hardcode the
//! source/destination local slot index. Materializing N into a scratch
//! register and using `load_reg!`/`store_reg!` would cost 2 instructions
//! (movz + ldr/str); the fixed-offset form below costs 1 (ldr/str with
//! immediate offset).
//!
//! Both macros emit a single instruction:
//!
//! ```text
//!     ldr  x{dst}, [x20, #{N*8}]      ; load_local_fixed!
//!     str  x{src}, [x20, #{N*8}]      ; store_local_fixed!
//! ```
//!
//! `x20` is the REGS pin (register-window base per
//! [`crate::dsl::reg_convention`]). `N * 8` is the byte offset because
//! each register-window slot is a 64-bit [`lyng_js_types::Value`]. The
//! AArch64 `ldr/str (immediate)` post-indexed form accepts a
//! `#imm12 * 8` byte offset directly when the destination is an x-reg;
//! N in 0..=3 fits trivially.
//!
//! ## Why a dedicated macro instead of `load_reg!`/`store_reg!`?
//!
//! `load_reg!(idx => dst)` requires `idx` to be a scratch-register
//! identifier holding the index value at runtime. For a compile-time
//! constant slot, we'd have to materialize the literal into a scratch
//! first (`movz xN, #idx`) and then issue the indexed load — 2
//! instructions vs the 1-instruction fixed-offset form below. Across
//! the 7 LoadLocalN / StoreLocalN handlers that use this shape (1.B.3
//! Tasks 2 + 3), the saving is 7 fewer instructions per dispatch on the
//! hot path.
//!
//! ## Lowerer interaction
//!
//! Unlike `load_reg!` / `store_reg!`, the index argument here is a
//! literal (e.g. `1`), NOT a scratch ident. The lowerer's
//! `substitute_idents` pass walks the token stream and replaces matched
//! scratch idents with their assigned register numbers; numeric literals
//! pass through unchanged. So `load_local_fixed!(1 => 10)` expands to
//! `ldr x10, [x20, #1 * 8]` after `stringify!`.
//!
//! Spec §2 (Phase 1.B.3 design).

/// Load a [`lyng_js_types::Value`] from the register-window slot at
/// fixed index `$n` into `$dst_reg`.
///
/// `$n` is a numeric literal (typically 0..=3 from Phase 1.B.3 ports;
/// the AArch64 immediate-offset range for an `ldr (unsigned offset)`
/// with an `xN` destination is wider — up to `#32760` — but bytecode
/// slot indices won't approach that). `$dst_reg` is a scratch register
/// number produced by the lowerer's ident substitution.
///
/// One instruction: `ldr x{dst}, [x20, #(n * 8)]`.
///
/// Usage from a handler body (the lowerer substitutes `dst` to a
/// register number literal before macro expansion):
///
/// ```ignore
/// load_local_fixed!(1 => dst);
/// ```
#[macro_export]
macro_rules! load_local_fixed {
    ($n:literal => $dst_reg:tt) => {
        concat!(
            "ldr    x", stringify!($dst_reg), ", [x20, #", stringify!($n), " * 8]\n",
        )
    };
}

/// Store the [`lyng_js_types::Value`] in `$src_reg` into the
/// register-window slot at fixed index `$n`. Mirror of
/// [`load_local_fixed!`].
///
/// One instruction: `str x{src}, [x20, #(n * 8)]`.
#[macro_export]
macro_rules! store_local_fixed {
    ($src_reg:tt, $n:literal) => {
        concat!(
            "str    x", stringify!($src_reg), ", [x20, #", stringify!($n), " * 8]\n",
        )
    };
}
