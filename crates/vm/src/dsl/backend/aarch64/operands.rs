//! Operand-decoding asm fragments for AArch64.
//!
//! These macros produce asm string fragments interpolated into the
//! per-handler `naked_asm!` block by the proc-macro lowerer. The
//! lowerer maps each named operand (e.g. `b`, `c`) to a scratch
//! W/X register slot (`t0..t6` → `w9..w15` / `x9..x15`); the macro
//! arguments here are the bare scratch indices (passed as literals
//! after the lowerer's ident-substitution pass — see
//! `lyng_vm_dsl::lower::substitute_idents`).
//!
//! Per the pinned-register convention (`reg_convention.rs`):
//!
//! - `x19` = PC      (bytecode-stream pointer)
//! - `x20` = REGS    (register-file base, `*mut Value`)
//! - `x21` = FV      (feedback-vector base, `*mut FeedbackEntry`)
//!
//! Narrow-form decoders only (Wide / ExtraWide land in Batch 7
//! alongside `op_wide` / `op_extra_wide`).

/// No-operand prologue (used by `op_wide` / `op_extra_wide` etc.).
/// Emits an empty fragment so the lowerer can splice it uniformly with
/// the other decode prologues.
#[macro_export]
macro_rules! decode_none {
    () => {
        ""
    };
}

/// Decode a single byte operand (narrow). Reads `[PC + 1]` as `u8` into
/// the named scratch w-register. Used by `op_return` (layout = A,
/// length = 2): the single byte at PC+1 is the register-id source.
#[macro_export]
macro_rules! decode_a {
    ($a:tt) => {
        concat!("ldrb   w", stringify!($a), ", [x19, #1]\n",)
    };
}

/// Decode two byte operands (narrow). Reads `[PC + 1]` and `[PC + 2]`
/// as `u8` into the named scratch w-registers. Used by `op_move`
/// (layout = Ab, length = 3).
#[macro_export]
macro_rules! decode_ab {
    ($a:tt, $b:tt) => {
        concat!(
            "ldrb   w",
            stringify!($a),
            ", [x19, #1]\n",
            "ldrb   w",
            stringify!($b),
            ", [x19, #2]\n",
        )
    };
}

/// Decode three byte operands (narrow). Reads `[PC + 1]`, `[PC + 2]`,
/// `[PC + 3]` as `u8` into the named scratch w-registers.
#[macro_export]
macro_rules! decode_abc {
    ($a:tt, $b:tt, $c:tt) => {
        concat!(
            "ldrb   w",
            stringify!($a),
            ", [x19, #1]\n",
            "ldrb   w",
            stringify!($b),
            ", [x19, #2]\n",
            "ldrb   w",
            stringify!($c),
            ", [x19, #3]\n",
        )
    };
}

/// Decode three byte operands + a 16-bit feedback-slot index at
/// offsets 1, 2, 3, 4-5.
#[macro_export]
macro_rules! decode_abc_slot {
    ($a:tt, $b:tt, $c:tt, $slot:tt) => {
        concat!(
            "ldrb   w",
            stringify!($a),
            ", [x19, #1]\n",
            "ldrb   w",
            stringify!($b),
            ", [x19, #2]\n",
            "ldrb   w",
            stringify!($c),
            ", [x19, #3]\n",
            "ldrh   w",
            stringify!($slot),
            ", [x19, #4]\n",
        )
    };
}

/// Decode `[byte, u16]` — a byte operand followed by a 16-bit operand.
#[macro_export]
macro_rules! decode_abx {
    ($a:tt, $bx:tt) => {
        concat!(
            "ldrb   w",
            stringify!($a),
            ", [x19, #1]\n",
            "ldrh   w",
            stringify!($bx),
            ", [x19, #2]\n",
        )
    };
}

/// Decode a single u32 operand at `[PC + 1]`.
#[macro_export]
macro_rules! decode_ax {
    ($ax:tt) => {
        concat!("ldr    w", stringify!($ax), ", [x19, #1]\n",)
    };
}

/// Load a Value from the register file at index in `$idx` into `$dst`.
/// Compiles to `ldr xDst, [x20, xIdx, lsl #3]` — single instruction.
#[macro_export]
macro_rules! load_reg {
    ($idx:tt => $dst:tt) => {
        concat!(
            "ldr    x",
            stringify!($dst),
            ", [x20, x",
            stringify!($idx),
            ", lsl #3]\n",
        )
    };
}

/// Store a Value `$src` into the register file at index `$idx`.
#[macro_export]
macro_rules! store_reg {
    ($idx:tt, $src:tt) => {
        concat!(
            "str    x",
            stringify!($src),
            ", [x20, x",
            stringify!($idx),
            ", lsl #3]\n",
        )
    };
}

/// Read the accumulator (register 0) into `$dst`.
#[macro_export]
macro_rules! load_acc {
    ($dst:tt) => {
        concat!("ldr    x", stringify!($dst), ", [x20]\n",)
    };
}

/// Write the accumulator (register 0) from `$src`.
#[macro_export]
macro_rules! store_acc {
    ($src:tt) => {
        concat!("str    x", stringify!($src), ", [x20]\n",)
    };
}
