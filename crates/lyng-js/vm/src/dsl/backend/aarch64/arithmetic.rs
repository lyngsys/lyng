//! SMI fast-path arithmetic asm fragments for AArch64.
//!
//! Each macro operates on **already-untagged** SMI payloads (32-bit
//! signed integers in the low half of an X-register, sign-extended).
//! Pair with [`crate::untag_smi!`] on inputs and [`crate::tag_smi!`]
//! on the destination — typically:
//!
//! ```text
//!     check_smi!(lhs, slow);
//!     check_smi!(rhs, slow);
//!     untag_smi!(lhs);
//!     untag_smi!(rhs);
//!     add_smi_overflow!(lhs, rhs => dst, slow);
//!     tag_smi!(dst);
//!     store_reg!(out, dst);
//! ```
//!
//! Overflow detection uses the V flag set by the W-form `adds` / `subs`
//! / `smull+cmp` sequences. On overflow we branch to `$label` (the
//! slow-path target the caller supplies).

/// 32-bit signed add with overflow detection. On overflow, branch to
/// `$label` (slow path). The result is written into `$dst` as a
/// sign-extended i64.
#[macro_export]
macro_rules! add_smi_overflow {
    ($lhs:tt, $rhs:tt => $dst:tt, $label:tt) => {
        concat!(
            "adds   w", stringify!($dst), ", w", stringify!($lhs), ", w", stringify!($rhs), "\n",
            "b.vs   ", stringify!($label), "\n",
            // Sign-extend so downstream tagging picks up the correct low 32.
            "sxtw   x", stringify!($dst), ", w", stringify!($dst), "\n",
        )
    };
}

/// 32-bit signed subtract with overflow detection.
#[macro_export]
macro_rules! sub_smi_overflow {
    ($lhs:tt, $rhs:tt => $dst:tt, $label:tt) => {
        concat!(
            "subs   w", stringify!($dst), ", w", stringify!($lhs), ", w", stringify!($rhs), "\n",
            "b.vs   ", stringify!($label), "\n",
            "sxtw   x", stringify!($dst), ", w", stringify!($dst), "\n",
        )
    };
}

/// 32-bit signed multiply with overflow detection.
///
/// AArch64 has no W-form `muls`; we compute the full 64-bit product
/// via `smull` (sign-extend lhs/rhs to 64 bits, multiply), then check
/// that the 64-bit result equals its sign-extended 32-bit form.
#[macro_export]
macro_rules! mul_smi_overflow {
    ($lhs:tt, $rhs:tt => $dst:tt, $label:tt) => {
        concat!(
            // x_dst = (i64) lhs * (i64) rhs  (signed widening multiply)
            "smull  x", stringify!($dst), ", w", stringify!($lhs), ", w", stringify!($rhs), "\n",
            // x16 = sxtw(x_dst[31:0])
            "sxtw   x16, w", stringify!($dst), "\n",
            // Overflow if sign-extended low 32 bits != full 64-bit product.
            "cmp    x", stringify!($dst), ", x16\n",
            "b.ne   ", stringify!($label), "\n",
        )
    };
}

/// 32-bit bitwise AND on SMI payloads (no overflow possible).
#[macro_export]
macro_rules! bit_and_smi {
    ($lhs:tt, $rhs:tt => $dst:tt) => {
        concat!(
            "and    w", stringify!($dst), ", w", stringify!($lhs), ", w", stringify!($rhs), "\n",
            "sxtw   x", stringify!($dst), ", w", stringify!($dst), "\n",
        )
    };
}

/// 32-bit bitwise OR on SMI payloads.
#[macro_export]
macro_rules! bit_or_smi {
    ($lhs:tt, $rhs:tt => $dst:tt) => {
        concat!(
            "orr    w", stringify!($dst), ", w", stringify!($lhs), ", w", stringify!($rhs), "\n",
            "sxtw   x", stringify!($dst), ", w", stringify!($dst), "\n",
        )
    };
}

/// 32-bit bitwise XOR on SMI payloads.
#[macro_export]
macro_rules! bit_xor_smi {
    ($lhs:tt, $rhs:tt => $dst:tt) => {
        concat!(
            "eor    w", stringify!($dst), ", w", stringify!($lhs), ", w", stringify!($rhs), "\n",
            "sxtw   x", stringify!($dst), ", w", stringify!($dst), "\n",
        )
    };
}

/// Left shift `$lhs` by `$rhs` bits. Only the low 5 bits of `$rhs`
/// matter per ECMAScript `<<` semantics. Always succeeds (no overflow
/// branch).
#[macro_export]
macro_rules! shift_left_smi {
    ($lhs:tt, $rhs:tt => $dst:tt) => {
        concat!(
            // Mask shift count to 5 bits (ECMAScript ToUint32 + & 31).
            "and    w16, w", stringify!($rhs), ", #0x1f\n",
            "lsl    w", stringify!($dst), ", w", stringify!($lhs), ", w16\n",
            "sxtw   x", stringify!($dst), ", w", stringify!($dst), "\n",
        )
    };
}

/// Arithmetic right shift (sign-preserving) by low 5 bits of `$rhs`.
#[macro_export]
macro_rules! shift_right_smi {
    ($lhs:tt, $rhs:tt => $dst:tt) => {
        concat!(
            "and    w16, w", stringify!($rhs), ", #0x1f\n",
            "asr    w", stringify!($dst), ", w", stringify!($lhs), ", w16\n",
            "sxtw   x", stringify!($dst), ", w", stringify!($dst), "\n",
        )
    };
}

/// Logical right shift (zero-fill) by low 5 bits of `$rhs`.
///
/// Note: ECMAScript `>>>` returns a Uint32; if the high bit is set the
/// result *cannot* be represented as a signed SMI. Callers branch to
/// the slow path when the result is negative-when-interpreted-signed.
/// This macro itself only performs the shift; the overflow check is
/// caller-emitted (typically `tbnz wDst, #31, slow`).
#[macro_export]
macro_rules! ushift_right_smi {
    ($lhs:tt, $rhs:tt => $dst:tt) => {
        concat!(
            "and    w16, w", stringify!($rhs), ", #0x1f\n",
            "lsr    w", stringify!($dst), ", w", stringify!($lhs), ", w16\n",
            // Zero-extend (don't sign-extend) — high bit stays the LSR result.
            "uxtw   x", stringify!($dst), ", w", stringify!($dst), "\n",
        )
    };
}

/// Negate an SMI in-place; branch to `$label` on overflow (only
/// `i32::MIN` overflows — its negation isn't representable as i32).
#[macro_export]
macro_rules! neg_smi_overflow {
    ($reg:tt, $label:tt) => {
        concat!(
            "negs   w", stringify!($reg), ", w", stringify!($reg), "\n",
            "b.vs   ", stringify!($label), "\n",
            "sxtw   x", stringify!($reg), ", w", stringify!($reg), "\n",
        )
    };
}

/// Bitwise NOT on an SMI (no overflow possible).
#[macro_export]
macro_rules! bit_not_smi {
    ($reg:tt) => {
        concat!(
            "mvn    w", stringify!($reg), ", w", stringify!($reg), "\n",
            "sxtw   x", stringify!($reg), ", w", stringify!($reg), "\n",
        )
    };
}
