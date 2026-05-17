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
//!   `PATTERN_KIND = 0x7ff8_<kind>_0000_0000` — 3 instructions
//!   (AND/CMP/B.NE).
//! - **Undefined / Null**: full 64-bit `CMP` against the canonical
//!   pattern (payload is always 0) — 2 instructions (CMP/B.NE).
//! - **Bool**: same shape as SMI; payload is 0/1.
//! - **Double**: negation of `is_tagged_bits`. The fast path branches
//!   to slow only when the value *is* tagged — `CMP/B.EQ` against
//!   `0x7ff8_0000_0000_0000` masked.
//!
//! All scratch use of `x9` is owned by the macro itself (caller-saved,
//! no live data across the macro boundary). The proc-macro lowerer
//! never assigns `t0..t6` to `x9` for live operands.

// ===========================================================================
// Tag-check macros (branch to label on miss).
// ===========================================================================

/// Check `reg` holds an SMI; branch to `label` on miss.
///
/// Fast path: 3 instructions after constants are hoisted by the
/// register allocator (AND / CMP / B.NE). Matches the asm shape in
/// `reports/js/lyng-js/dsl-asm-baseline-aarch64/Add.asm:99-113`.
#[macro_export]
macro_rules! check_smi {
    ($reg:ident, $label:tt) => {
        concat!(
            // x9 := value & (TAG_HEADER | TAG_KIND_MASK)
            "mov    x9, #0xffff, lsl #32\n",
            "movk   x9, #0x7ff8, lsl #48\n",
            "and    x9, x", stringify!($reg), ", x9\n",
            // cmp against TAG_HEADER | (4 << 32) == 0x7ff8_0004_0000_0000
            "mov    x10, #0x4, lsl #32\n",
            "movk   x10, #0x7ff8, lsl #48\n",
            "cmp    x9, x10\n",
            "b.ne   ", stringify!($label), "\n",
        )
    };
}

/// Check `reg` holds an ObjectRef; branch to `label` on miss.
#[macro_export]
macro_rules! check_object_ref {
    ($reg:ident, $label:tt) => {
        concat!(
            "mov    x9, #0xffff, lsl #32\n",
            "movk   x9, #0x7ff8, lsl #48\n",
            "and    x9, x", stringify!($reg), ", x9\n",
            // kind = 5
            "mov    x10, #0x5, lsl #32\n",
            "movk   x10, #0x7ff8, lsl #48\n",
            "cmp    x9, x10\n",
            "b.ne   ", stringify!($label), "\n",
        )
    };
}

/// Check `reg` holds a StringRef; branch to `label` on miss.
#[macro_export]
macro_rules! check_string_ref {
    ($reg:ident, $label:tt) => {
        concat!(
            "mov    x9, #0xffff, lsl #32\n",
            "movk   x9, #0x7ff8, lsl #48\n",
            "and    x9, x", stringify!($reg), ", x9\n",
            // kind = 6
            "mov    x10, #0x6, lsl #32\n",
            "movk   x10, #0x7ff8, lsl #48\n",
            "cmp    x9, x10\n",
            "b.ne   ", stringify!($label), "\n",
        )
    };
}

/// Check `reg` is `undefined`; branch to `label` on miss.
///
/// Fast path: 2 instructions — undefined is a single canonical bit
/// pattern (`0x7ff8_0001_0000_0000`), so a full 64-bit compare works.
#[macro_export]
macro_rules! check_undefined {
    ($reg:ident, $label:tt) => {
        concat!(
            // x9 := 0x7ff8_0001_0000_0000
            "mov    x9, #0x1, lsl #32\n",
            "movk   x9, #0x7ff8, lsl #48\n",
            "cmp    x", stringify!($reg), ", x9\n",
            "b.ne   ", stringify!($label), "\n",
        )
    };
}

/// Check `reg` is `null`; branch to `label` on miss.
#[macro_export]
macro_rules! check_null {
    ($reg:ident, $label:tt) => {
        concat!(
            // x9 := 0x7ff8_0002_0000_0000
            "mov    x9, #0x2, lsl #32\n",
            "movk   x9, #0x7ff8, lsl #48\n",
            "cmp    x", stringify!($reg), ", x9\n",
            "b.ne   ", stringify!($label), "\n",
        )
    };
}

/// Check `reg` is a Boolean (true or false); branch to `label` on miss.
///
/// Mask off the payload (bit 0) before comparing the tag header+kind.
#[macro_export]
macro_rules! check_bool {
    ($reg:ident, $label:tt) => {
        concat!(
            "mov    x9, #0xffff, lsl #32\n",
            "movk   x9, #0x7ff8, lsl #48\n",
            "and    x9, x", stringify!($reg), ", x9\n",
            // kind = 3
            "mov    x10, #0x3, lsl #32\n",
            "movk   x10, #0x7ff8, lsl #48\n",
            "cmp    x9, x10\n",
            "b.ne   ", stringify!($label), "\n",
        )
    };
}

/// Check `reg` holds a double; branch to `label` on miss (i.e. when the
/// value *is* tag-encoded).
///
/// Inverse of `check_smi!` / `check_object_ref!`: if `(value & MASK) ==
/// TAG_HEADER` then the value has the NaN-prefix; we then must also
/// verify the kind is a known TagKind. The fast path here approximates
/// "high 16 bits == 0x7ff8" → tagged → slow. False positives (a finite
/// double whose high 16 bits coincide with TAG_HEADER) are caught by
/// the slow path's full validation. See value-layout report §3.
#[macro_export]
macro_rules! check_double {
    ($reg:ident, $label:tt) => {
        concat!(
            "lsr    x9, x", stringify!($reg), ", #48\n",
            "cmp    x9, #0x7ff8\n",
            "b.eq   ", stringify!($label), "\n",
        )
    };
}

// ===========================================================================
// Untag macros — extract payload from the tagged Value.
// ===========================================================================

/// Untag an SMI into `$reg` (in place); leaves the sign-extended i64
/// in the X-register, ready for arithmetic.
///
/// SMI payload is the low 32 bits, sign-extended. On AArch64 a `sxtw`
/// from the same X-register's W-half does it in one instruction.
#[macro_export]
macro_rules! untag_smi {
    ($reg:ident) => {
        concat!(
            "sxtw   x", stringify!($reg), ", w", stringify!($reg), "\n",
        )
    };
}

/// Untag an ObjectRef (`u32` handle id) into `$reg` in place.
///
/// Payload is the low 32 bits, zero-extended (the handle is a
/// `NonZeroU32`). A W-register read of `$reg`'s low half does it; we
/// emit an explicit `uxtw` so the macro is composable with downstream
/// X-register-indexed loads (e.g. `ldr xR, [base, xIdx, lsl #3]`).
#[macro_export]
macro_rules! untag_object_ref {
    ($reg:ident) => {
        concat!(
            "uxtw   x", stringify!($reg), ", w", stringify!($reg), "\n",
        )
    };
}

/// Untag a bool into `$reg` (low bit of the payload).
#[macro_export]
macro_rules! untag_bool {
    ($reg:ident) => {
        concat!(
            "and    x", stringify!($reg), ", x", stringify!($reg), ", #0x1\n",
        )
    };
}

// ===========================================================================
// Tag macros — bake an untagged payload into a tagged Value.
// ===========================================================================

/// Tag an i32 in `$reg` as an SMI Value (in-place).
///
/// Compiles to `mov xHdrKind, #...; orr xReg, xHdrKind, xReg, uxtw`.
/// The constant materialization is one MOV+MOVK; the proc-macro hoists
/// it across handler basic blocks when SMI tagging recurs.
#[macro_export]
macro_rules! tag_smi {
    ($reg:ident) => {
        concat!(
            // x9 := 0x7ff8_0004_0000_0000
            "mov    x9, #0x4, lsl #32\n",
            "movk   x9, #0x7ff8, lsl #48\n",
            // result = header | (kind << 32) | (payload as u32)
            "orr    x", stringify!($reg), ", x9, x", stringify!($reg), ", uxtw\n",
        )
    };
}

/// Tag an ObjectRef id in `$reg` as an `ObjectRef` Value (in-place).
#[macro_export]
macro_rules! tag_object_ref {
    ($reg:ident) => {
        concat!(
            // x9 := 0x7ff8_0005_0000_0000
            "mov    x9, #0x5, lsl #32\n",
            "movk   x9, #0x7ff8, lsl #48\n",
            "orr    x", stringify!($reg), ", x9, x", stringify!($reg), ", uxtw\n",
        )
    };
}

/// Materialize a `Value::undefined()` into `$reg`.
#[macro_export]
macro_rules! tag_undefined {
    ($reg:ident) => {
        concat!(
            "mov    x", stringify!($reg), ", #0x1, lsl #32\n",
            "movk   x", stringify!($reg), ", #0x7ff8, lsl #48\n",
        )
    };
}

/// Materialize a `Value::null()` into `$reg`.
#[macro_export]
macro_rules! tag_null {
    ($reg:ident) => {
        concat!(
            "mov    x", stringify!($reg), ", #0x2, lsl #32\n",
            "movk   x", stringify!($reg), ", #0x7ff8, lsl #48\n",
        )
    };
}

/// Materialize a constant boolean (`true` if `$payload == 1`,
/// `false` if `0`) into `$reg`.
#[macro_export]
macro_rules! tag_bool_const {
    ($reg:ident, $payload:literal) => {
        concat!(
            "mov    x", stringify!($reg), ", #", stringify!($payload), "\n",
            "movk   x", stringify!($reg), ", #0x3, lsl #32\n",
            "movk   x", stringify!($reg), ", #0x7ff8, lsl #48\n",
        )
    };
}
