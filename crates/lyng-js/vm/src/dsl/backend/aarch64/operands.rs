//! Operand-decoding asm fragments for AArch64.
//!
//! These macros produce asm string fragments interpolated into the
//! per-handler `naked_asm!` block by the proc-macro lowerer. The
//! lowerer maps each named operand (e.g. `b`, `c`) to a scratch
//! W/X register slot (`t0..t6` → `w9..w15` / `x9..x15`); the macro
//! arguments here are the bare scratch indices.
//!
//! Per the pinned-register convention (`reg_convention.rs`):
//!
//! - `x19` = PC      (bytecode-stream pointer)
//! - `x20` = REGS    (register-file base, `*mut Value`)
//! - `x21` = FV      (feedback-vector base, `*mut FeedbackEntry`)
//!
//! Narrow-form decoders only (Wide / ExtraWide land in Batch 7
//! alongside `op_wide` / `op_extra_wide`).

/// Decode three byte operands (narrow). Reads `[PC + 1]`, `[PC + 2]`,
/// `[PC + 3]` as `u8` into the named scratch w-registers.
#[macro_export]
macro_rules! decode_abc {
    ($a:ident, $b:ident, $c:ident) => {
        concat!(
            "ldrb   w", stringify!($a), ", [x19, #1]\n",
            "ldrb   w", stringify!($b), ", [x19, #2]\n",
            "ldrb   w", stringify!($c), ", [x19, #3]\n",
        )
    };
}

/// Decode three byte operands + a 16-bit feedback-slot index at
/// offsets 1, 2, 3, 4-5.
#[macro_export]
macro_rules! decode_abc_slot {
    ($a:ident, $b:ident, $c:ident, $slot:ident) => {
        concat!(
            "ldrb   w", stringify!($a), ", [x19, #1]\n",
            "ldrb   w", stringify!($b), ", [x19, #2]\n",
            "ldrb   w", stringify!($c), ", [x19, #3]\n",
            "ldrh   w", stringify!($slot), ", [x19, #4]\n",
        )
    };
}

/// Decode `[byte, u16]` — a byte operand followed by a 16-bit operand.
#[macro_export]
macro_rules! decode_abx {
    ($a:ident, $bx:ident) => {
        concat!(
            "ldrb   w", stringify!($a), ", [x19, #1]\n",
            "ldrh   w", stringify!($bx), ", [x19, #2]\n",
        )
    };
}

/// Decode a single u32 operand at `[PC + 1]`.
#[macro_export]
macro_rules! decode_ax {
    ($ax:ident) => {
        concat!(
            "ldr    w", stringify!($ax), ", [x19, #1]\n",
        )
    };
}

/// Load a Value from the register file at index in `$idx` into `$dst`.
/// Compiles to `ldr xDst, [x20, xIdx, lsl #3]` — single instruction.
#[macro_export]
macro_rules! load_reg {
    ($idx:ident => $dst:ident) => {
        concat!(
            "ldr    x", stringify!($dst), ", [x20, x", stringify!($idx), ", lsl #3]\n",
        )
    };
}

/// Store a Value `$src` into the register file at index `$idx`.
#[macro_export]
macro_rules! store_reg {
    ($idx:ident, $src:ident) => {
        concat!(
            "str    x", stringify!($src), ", [x20, x", stringify!($idx), ", lsl #3]\n",
        )
    };
}

/// Read the accumulator (register 0) into `$dst`.
#[macro_export]
macro_rules! load_acc {
    ($dst:ident) => {
        concat!(
            "ldr    x", stringify!($dst), ", [x20]\n",
        )
    };
}

/// Write the accumulator (register 0) from `$src`.
#[macro_export]
macro_rules! store_acc {
    ($src:ident) => {
        concat!(
            "str    x", stringify!($src), ", [x20]\n",
        )
    };
}
