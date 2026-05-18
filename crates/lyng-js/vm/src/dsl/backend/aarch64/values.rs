//! Value-tag check and tag-manipulation asm fragments for AArch64.
//!
//! Per [`reports/js/lyng-js/llint-dsl-value-layout.md`], `Value` is a
//! NaN-tag-space `u64` with a 16-bit `TagKind` field in bits 32-47 and
//! a 32-bit payload in bits 0-31. The high 13 bits encode the
//! canonical-NaN prefix (`TAG_HEADER = 0x7ff8_...`).
//!
//! Predicate shapes:
//!
//! - **SMI / ObjectRef / StringRef / etc.**: `(bits & MASK_KIND_HDR) == PATTERN_KIND`
//!   where `MASK_KIND_HDR = 0x7fff_ffff_0000_0000` and
//!   `PATTERN_KIND = 0x7ff8_<kind>_0000_0000` — 4 instructions
//!   (MOVZ/MOVK/AND/MOVZ/MOVK/CMP/B.NE).
//! - **Undefined / Null**: full 64-bit `CMP` against the canonical
//!   pattern (payload is always 0) — 2 instructions (CMP/B.NE).
//! - **Bool**: same shape as SMI; payload is 0/1.
//! - **Double**: negation of `is_tagged_bits`. The fast path branches
//!   to slow only when the value *is* tagged — `CMP/B.EQ` against
//!   `0x7ff8_0000_0000_0000` masked.
//!
//! All internal scratch use is on `x16` / `x17` (AAPCS64 "IP0/IP1"
//! intra-procedure-call temporaries — caller-saved, no agreed-upon
//! semantic role in the DSL pinned-register convention). Operand
//! slots `t0..t6` map to `x9..x15`. This keeps the live-operand
//! budget at 7 slots without colliding with macro-internal scratch.
//!
//! AArch64 mov-immediate-with-shift syntax: the canonical 16-bit
//! immediate forms are `movz` (zero rest), `movn` (invert and zero
//! rest), and `movk` (keep rest). `mov xR, #imm, lsl #shift` is *not*
//! a separate form — the assembler accepts it as an alias for `movz`
//! / `movn` in narrow cases, but rejects mid-range shifts on Apple
//! Silicon's AArch64 assembler (`lsl #32`/`lsl #48` are exactly the
//! cases we hit). We therefore emit `movz` / `movk` explicitly.

// ===========================================================================
// Tag-check macros (branch to label on miss).
// ===========================================================================

/// Check `reg` holds an SMI; branch to `label` on miss.
#[macro_export]
macro_rules! check_smi {
    ($reg:tt, $label:tt) => {
        concat!(
            // x16 := TAG_HEADER | TAG_KIND_MASK == 0x7fff_ffff_0000_0000
            "movz   x16, #0xffff, lsl #32\n",
            "movk   x16, #0x7ff8, lsl #48\n",
            "and    x16, x", stringify!($reg), ", x16\n",
            // x17 := TAG_HEADER | (4 << 32) == 0x7ff8_0004_0000_0000
            "movz   x17, #0x4, lsl #32\n",
            "movk   x17, #0x7ff8, lsl #48\n",
            "cmp    x16, x17\n",
            "b.ne   ", stringify!($label), "\n",
        )
    };
}

/// Check `reg` holds an ObjectRef; branch to `label` on miss.
#[macro_export]
macro_rules! check_object_ref {
    ($reg:tt, $label:tt) => {
        concat!(
            "movz   x16, #0xffff, lsl #32\n",
            "movk   x16, #0x7ff8, lsl #48\n",
            "and    x16, x", stringify!($reg), ", x16\n",
            "movz   x17, #0x5, lsl #32\n",
            "movk   x17, #0x7ff8, lsl #48\n",
            "cmp    x16, x17\n",
            "b.ne   ", stringify!($label), "\n",
        )
    };
}

/// Check `reg` holds a StringRef; branch to `label` on miss.
#[macro_export]
macro_rules! check_string_ref {
    ($reg:tt, $label:tt) => {
        concat!(
            "movz   x16, #0xffff, lsl #32\n",
            "movk   x16, #0x7ff8, lsl #48\n",
            "and    x16, x", stringify!($reg), ", x16\n",
            "movz   x17, #0x6, lsl #32\n",
            "movk   x17, #0x7ff8, lsl #48\n",
            "cmp    x16, x17\n",
            "b.ne   ", stringify!($label), "\n",
        )
    };
}

/// Check `reg` is `undefined`; branch to `label` on miss.
#[macro_export]
macro_rules! check_undefined {
    ($reg:tt, $label:tt) => {
        concat!(
            "movz   x16, #0x1, lsl #32\n",
            "movk   x16, #0x7ff8, lsl #48\n",
            "cmp    x", stringify!($reg), ", x16\n",
            "b.ne   ", stringify!($label), "\n",
        )
    };
}

/// Check `reg` is `null`; branch to `label` on miss.
#[macro_export]
macro_rules! check_null {
    ($reg:tt, $label:tt) => {
        concat!(
            "movz   x16, #0x2, lsl #32\n",
            "movk   x16, #0x7ff8, lsl #48\n",
            "cmp    x", stringify!($reg), ", x16\n",
            "b.ne   ", stringify!($label), "\n",
        )
    };
}

/// Check `reg` is a Boolean (true or false); branch to `label` on miss.
#[macro_export]
macro_rules! check_bool {
    ($reg:tt, $label:tt) => {
        concat!(
            "movz   x16, #0xffff, lsl #32\n",
            "movk   x16, #0x7ff8, lsl #48\n",
            "and    x16, x", stringify!($reg), ", x16\n",
            "movz   x17, #0x3, lsl #32\n",
            "movk   x17, #0x7ff8, lsl #48\n",
            "cmp    x16, x17\n",
            "b.ne   ", stringify!($label), "\n",
        )
    };
}

/// Check `reg` holds a double; branch to `label` on miss.
#[macro_export]
macro_rules! check_double {
    ($reg:tt, $label:tt) => {
        concat!(
            "lsr    x16, x", stringify!($reg), ", #48\n",
            "cmp    x16, #0x7ff8\n",
            "b.eq   ", stringify!($label), "\n",
        )
    };
}

// ===========================================================================
// Untag macros — extract payload from the tagged Value.
// ===========================================================================

#[macro_export]
macro_rules! untag_smi {
    ($reg:tt) => {
        concat!(
            "sxtw   x", stringify!($reg), ", w", stringify!($reg), "\n",
        )
    };
}

#[macro_export]
macro_rules! untag_object_ref {
    ($reg:tt) => {
        concat!(
            "uxtw   x", stringify!($reg), ", w", stringify!($reg), "\n",
        )
    };
}

#[macro_export]
macro_rules! untag_bool {
    ($reg:tt) => {
        concat!(
            "and    x", stringify!($reg), ", x", stringify!($reg), ", #0x1\n",
        )
    };
}

// ===========================================================================
// Tag macros — bake an untagged payload into a tagged Value.
// ===========================================================================

#[macro_export]
macro_rules! tag_smi {
    ($reg:tt) => {
        concat!(
            "movz   x16, #0x4, lsl #32\n",
            "movk   x16, #0x7ff8, lsl #48\n",
            "uxtw   x", stringify!($reg), ", w", stringify!($reg), "\n",
            "orr    x", stringify!($reg), ", x16, x", stringify!($reg), "\n",
        )
    };
}

#[macro_export]
macro_rules! tag_object_ref {
    ($reg:tt) => {
        concat!(
            "movz   x16, #0x5, lsl #32\n",
            "movk   x16, #0x7ff8, lsl #48\n",
            "uxtw   x", stringify!($reg), ", w", stringify!($reg), "\n",
            "orr    x", stringify!($reg), ", x16, x", stringify!($reg), "\n",
        )
    };
}

#[macro_export]
macro_rules! tag_undefined {
    ($reg:tt) => {
        concat!(
            "movz   x", stringify!($reg), ", #0x1, lsl #32\n",
            "movk   x", stringify!($reg), ", #0x7ff8, lsl #48\n",
        )
    };
}

#[macro_export]
macro_rules! tag_null {
    ($reg:tt) => {
        concat!(
            "movz   x", stringify!($reg), ", #0x2, lsl #32\n",
            "movk   x", stringify!($reg), ", #0x7ff8, lsl #48\n",
        )
    };
}

#[macro_export]
macro_rules! tag_bool_const {
    ($reg:tt, $payload:literal) => {
        concat!(
            "movz   x", stringify!($reg), ", #", stringify!($payload), "\n",
            "movk   x", stringify!($reg), ", #0x3, lsl #32\n",
            "movk   x", stringify!($reg), ", #0x7ff8, lsl #48\n",
        )
    };
}

/// Tag a compile-time SMI literal payload into `$reg`. Produces a fully
/// tagged `Value` carrying the SMI variant + the literal payload. The
/// SMI tag kind is `0x4`; the payload occupies the low 32 bits, the kind
/// 16 bits, and the NaN-tag header bits 48-63. 3 instructions: movz
/// payload, movk kind, movk header.
///
/// Used by `op_load_zero` (payload = 0), `op_load_one` (payload = 1),
/// and similar SMI constant-loader opcodes. Distinct from `tag_smi!`,
/// which assumes the payload is already in the register's low word.
#[macro_export]
macro_rules! tag_smi_const {
    ($reg:tt, $payload:literal) => {
        concat!(
            "movz   x", stringify!($reg), ", #", stringify!($payload), "\n",
            "movk   x", stringify!($reg), ", #0x4, lsl #32\n",
            "movk   x", stringify!($reg), ", #0x7ff8, lsl #48\n",
        )
    };
}
