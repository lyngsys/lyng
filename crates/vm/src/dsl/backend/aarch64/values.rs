//! Value-tag check and tag-manipulation asm fragments for `AArch64`.
//!
//! Per [`reports/lyng/llint-dsl-value-layout.md`], `Value` is a
//! NaN-tag-space `u64` with a 16-bit `TagKind` field in bits 32-47 and
//! a 32-bit payload in bits 0-31. The high 13 bits encode the
//! canonical-NaN prefix (`TAG_HEADER = 0x7ff8_...`).
//!
//! Predicate shapes:
//!
//! - **SMI / `ObjectRef` / `StringRef` / `Bool`**: the high 32 bits of any
//!   tagged value are exactly `0x7ff8_000K` where `K` is the
//!   `TagKind`. We `lsr xN, #32` into a scratch (extracting only the
//!   tag-bearing word and zeroing the payload simultaneously), build
//!   `0x7ff8_000K` in a second scratch via `MOVZ`+`MOVK`, then `CMP w`
//!   (32-bit form) and `B.NE`. 5 instructions total
//!   (`LSR`/`MOVZ`/`MOVK`/`CMP`/`B.NE`), down from the previous
//!   `AND`-`MOVZ`-`MOVK`-`CMP` 7-insn shape.
//! - **Undefined / Null**: full 64-bit `CMP` against the canonical
//!   pattern (payload is always 0) — 4 instructions
//!   (MOVZ/MOVK/CMP/B.NE). The 32-bit lsr trick saves nothing here
//!   because the comparand `0x7ff8_0001` (etc.) doesn't fit a 12-bit
//!   `CMP` immediate, so we'd still need 2 insns to materialize it.
//! - **Double**: negation of `is_tagged_bits`. The fast path branches
//!   to slow only when the value *is* tagged — `CMP/B.EQ` against
//!   `0x7ff8` after an LSR by 48 (3 insns total).
//!
//! Note: a single-bit `TBZ`/`TBNZ` predicate is **not** viable for any
//! of these macros — the 16-bit `TagKind` field carries
//! non-bit-disjoint values (Bool=3=0b0011, SMI=4=0b0100, ObjectRef=5
//! =0b0101, etc.), so no single bit distinguishes one kind from
//! another. The 5-insn LSR/MOVZ/MOVK/CMP/B.NE shape is the shortest
//! correct sequence for these checks under the current encoding.
//!
//! All internal scratch use is on `x16` / `x17` (AAPCS64 "IP0/IP1"
//! intra-procedure-call temporaries — caller-saved, no agreed-upon
//! semantic role in the DSL pinned-register convention). Operand
//! slots `t0..t6` map to `x9..x15`. This keeps the live-operand
//! budget at 7 slots without colliding with macro-internal scratch.
//!
//! `AArch64` mov-immediate-with-shift syntax: the canonical 16-bit
//! immediate forms are `movz` (zero rest), `movn` (invert and zero
//! rest), and `movk` (keep rest). `mov xR, #imm, lsl #shift` is *not*
//! a separate form — the assembler accepts it as an alias for `movz`
//! / `movn` in narrow cases, but rejects mid-range shifts on Apple
//! Silicon's `AArch64` assembler (`lsl #32`/`lsl #48` are exactly the
//! cases we hit). We therefore emit `movz` / `movk` explicitly.

// ===========================================================================
// Tag-check macros (branch to label on miss).
// ===========================================================================

/// Check `reg` holds an SMI; branch to `label` on miss.
///
/// 5 instructions: `LSR` the value down by 32 to drop the payload and
/// land the `0x7ff8_<kind>` half-word in `w16`, materialize the
/// expected `0x7ff8_0004` (SMI kind) in `w17` via `MOVZ`+`MOVK`, then
/// 32-bit `CMP` and `B.NE`.
#[macro_export]
macro_rules! check_smi {
    ($reg:tt, $label:tt) => {
        concat!(
            "lsr    x16, x",
            stringify!($reg),
            ", #32\n",
            // w17 := 0x7ff8_0004 == TAG_HEADER>>32 | SMI kind (4)
            "movz   w17, #0x4\n",
            "movk   w17, #0x7ff8, lsl #16\n",
            "cmp    w16, w17\n",
            "b.ne   ",
            stringify!($label),
            "\n",
        )
    };
}

/// Check two registers both hold SMIs; branch to `label` on either miss.
///
/// Paired counterpart to `check_smi!`. Binary-op handlers (`op_add`,
/// `op_sub`, ...) emit two consecutive SMI guards that share a single
/// slow label; the comparand `0x7ff8_0004` is identical between them
/// and so can be hoisted out of the per-operand body.
///
/// ## Emitted shape (8 instructions total)
///
/// ```text
///     ; hoisted comparand — w17 := 0x7ff8_0004 (high 32 bits of an SMI Value)
///     movz  w17, #0x4
///     movk  w17, #0x7ff8, lsl #16
///     ; per-operand SMI check (reg_a)
///     lsr   x16, x{reg_a}, #32
///     cmp   w16, w17
///     b.ne  {label}
///     ; per-operand SMI check (reg_b)
///     lsr   x16, x{reg_b}, #32
///     cmp   w16, w17
///     b.ne  {label}
/// ```
///
/// Two separate `check_smi!` invocations emit 5 instructions each
/// (`LSR`/`MOVZ`/`MOVK`/`CMP`/`B.NE`) — 10 instructions for two
/// operands. The paired form is **8**, saving 2 instructions per
/// binary-op fast path by hoisting the duplicated `MOVZ`+`MOVK`
/// comparand setup.
///
/// ## Why pairing works
///
/// The comparand `0x7ff8_0004` is a per-kind constant: it depends
/// only on `TagKind::Smi` (= 4) and the canonical-NaN header — never
/// on the operand value. Building it once in `w17` and reusing it
/// across both compares is a textbook scalar-CSE move; the
/// proc-macro lowerer cannot perform this CSE itself because each
/// `check_smi!` is an opaque `concat!` blob to the assembler.
///
/// ## Scratch register usage
///
/// `x16` and `x17` are AAPCS64 IP0/IP1 — caller-saved intra-procedure
/// scratch with no semantic role in the DSL pinned-register
/// convention. `x17` carries the hoisted comparand across both per-
/// operand bodies; `x16` is rewritten by each `lsr`. Neither `reg_a`
/// nor `reg_b` is mutated, so they remain available for subsequent
/// `untag_smi!` / arithmetic.
///
/// ## When to use
///
/// Use whenever a handler emits two adjacent `check_smi!` calls that
/// share a slow label. Out-of-scope for non-paired sites (single SMI
/// guards keep `check_smi!`), and for paired guards with diverging
/// labels (rare; pair the labels first or keep the singles).
#[macro_export]
macro_rules! check_smi_pair {
    ($reg_a:tt, $reg_b:tt, $label:tt) => {
        concat!(
            // w17 := 0x7ff8_0004 — high 32 bits of an SMI Value, built
            // once and reused across both per-operand compares.
            "movz   w17, #0x4\n",
            "movk   w17, #0x7ff8, lsl #16\n",
            // per-operand: shift high half of the Value into w16, then
            // 32-bit compare against the SMI tag pattern.
            "lsr    x16, x",
            stringify!($reg_a),
            ", #32\n",
            "cmp    w16, w17\n",
            "b.ne   ",
            stringify!($label),
            "\n",
            "lsr    x16, x",
            stringify!($reg_b),
            ", #32\n",
            "cmp    w16, w17\n",
            "b.ne   ",
            stringify!($label),
            "\n",
        )
    };
}

/// Check `reg` holds an `ObjectRef`; branch to `label` on miss.
///
/// 5 instructions; see `check_smi!` for the `LSR`/`MOVZ`/`MOVK`/`CMP`/`B.NE`
/// shape rationale. Comparand is `0x7ff8_0005` (`ObjectRef` kind = 5).
#[macro_export]
macro_rules! check_object_ref {
    ($reg:tt, $label:tt) => {
        concat!(
            "lsr    x16, x",
            stringify!($reg),
            ", #32\n",
            "movz   w17, #0x5\n",
            "movk   w17, #0x7ff8, lsl #16\n",
            "cmp    w16, w17\n",
            "b.ne   ",
            stringify!($label),
            "\n",
        )
    };
}

/// Check `reg` holds a `StringRef`; branch to `label` on miss.
///
/// 5 instructions; see `check_smi!` for the `LSR`/`MOVZ`/`MOVK`/`CMP`/`B.NE`
/// shape rationale. Comparand is `0x7ff8_0006` (`StringRef` kind = 6).
#[macro_export]
macro_rules! check_string_ref {
    ($reg:tt, $label:tt) => {
        concat!(
            "lsr    x16, x",
            stringify!($reg),
            ", #32\n",
            "movz   w17, #0x6\n",
            "movk   w17, #0x7ff8, lsl #16\n",
            "cmp    w16, w17\n",
            "b.ne   ",
            stringify!($label),
            "\n",
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
            "cmp    x",
            stringify!($reg),
            ", x16\n",
            "b.ne   ",
            stringify!($label),
            "\n",
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
            "cmp    x",
            stringify!($reg),
            ", x16\n",
            "b.ne   ",
            stringify!($label),
            "\n",
        )
    };
}

/// Check `reg` is a Boolean (true or false); branch to `label` on miss.
///
/// 5 instructions; see `check_smi!` for the LSR/MOVZ/MOVK/CMP/B.NE
/// shape rationale. Comparand is `0x7ff8_0003` (Boolean kind = 3).
#[macro_export]
macro_rules! check_bool {
    ($reg:tt, $label:tt) => {
        concat!(
            "lsr    x16, x",
            stringify!($reg),
            ", #32\n",
            "movz   w17, #0x3\n",
            "movk   w17, #0x7ff8, lsl #16\n",
            "cmp    w16, w17\n",
            "b.ne   ",
            stringify!($label),
            "\n",
        )
    };
}

/// Check `reg` holds a double; branch to `label` on miss.
#[macro_export]
macro_rules! check_double {
    ($reg:tt, $label:tt) => {
        concat!(
            "lsr    x16, x",
            stringify!($reg),
            ", #48\n",
            "movz   x17, #0x7ff8\n",
            "cmp    x16, x17\n",
            "b.eq   ",
            stringify!($label),
            "\n",
        )
    };
}

/// After a raw `Value` equality match, branch to `$true_label` when the
/// value is known to be strictly equal to itself, or `$false_label` for
/// the canonical NaN value.
///
/// Raw-equal non-NaN doubles are true. Tagged values are true because
/// the raw identity already matched. The only raw-equal value that is
/// not strictly equal to itself is the canonical NaN encoding, whose
/// high word is the NaN tag header and whose kind field is zero.
#[macro_export]
macro_rules! branch_raw_equal_strict_result {
    ($reg:tt, true = $true_label:tt, false = $false_label:tt) => {
        concat!(
            "lsr    x16, x",
            stringify!($reg),
            ", #48\n",
            "movz   x17, #0x7ff8\n",
            "cmp    x16, x17\n",
            "b.ne   ",
            stringify!($true_label),
            "\n",
            "ubfx   x16, x",
            stringify!($reg),
            ", #32, #16\n",
            "cbz    x16, ",
            stringify!($false_label),
            "\n",
            "b      ",
            stringify!($true_label),
            "\n",
        )
    };
}

/// Extract a non-double tag kind into `$kind` or branch to `$label`.
///
/// Tagged values use the `0x7ff8` high-word header and a non-zero
/// 16-bit kind field. Doubles either use another high word or, for the
/// canonical NaN encoding, use the `0x7ff8` header with kind zero.
#[macro_export]
macro_rules! tagged_kind_or_branch {
    ($reg:tt => $kind:tt, $label:tt) => {
        concat!(
            "lsr    x16, x",
            stringify!($reg),
            ", #48\n",
            "movz   x17, #0x7ff8\n",
            "cmp    x16, x17\n",
            "b.ne   ",
            stringify!($label),
            "\n",
            "ubfx   x",
            stringify!($kind),
            ", x",
            stringify!($reg),
            ", #32, #16\n",
            "cbz    x",
            stringify!($kind),
            ", ",
            stringify!($label),
            "\n",
        )
    };
}

/// Branch when a tag kind may require heap content comparison for
/// strict equality even though the raw handles differ.
#[macro_export]
macro_rules! branch_if_string_or_bigint_kind {
    ($kind:tt, $label:tt) => {
        concat!(
            "cmp    x",
            stringify!($kind),
            ", #6\n",
            "b.eq   ",
            stringify!($label),
            "\n",
            "cmp    x",
            stringify!($kind),
            ", #8\n",
            "b.eq   ",
            stringify!($label),
            "\n",
        )
    };
}

/// Branch when a tag kind is `undefined` or `null`.
#[macro_export]
macro_rules! branch_if_nullish_kind {
    ($kind:tt, $label:tt) => {
        concat!(
            "cmp    x",
            stringify!($kind),
            ", #1\n",
            "b.eq   ",
            stringify!($label),
            "\n",
            "cmp    x",
            stringify!($kind),
            ", #2\n",
            "b.eq   ",
            stringify!($label),
            "\n",
        )
    };
}

/// Branch when a tag kind is an `ObjectRef`.
#[macro_export]
macro_rules! branch_if_object_kind {
    ($kind:tt, $label:tt) => {
        concat!(
            "cmp    x",
            stringify!($kind),
            ", #5\n",
            "b.eq   ",
            stringify!($label),
            "\n",
        )
    };
}

/// Branch when a tag kind belongs to the VM-internal range.
#[macro_export]
macro_rules! branch_if_internal_kind {
    ($kind:tt, $label:tt) => {
        concat!(
            "cmp    x",
            stringify!($kind),
            ", #9\n",
            "b.hs   ",
            stringify!($label),
            "\n",
        )
    };
}

// ===========================================================================
// Untag macros — extract payload from the tagged Value.
// ===========================================================================

#[macro_export]
macro_rules! untag_smi {
    ($reg:tt) => {
        concat!("sxtw   x", stringify!($reg), ", w", stringify!($reg), "\n",)
    };
}

#[macro_export]
macro_rules! untag_object_ref {
    ($reg:tt) => {
        concat!("uxtw   x", stringify!($reg), ", w", stringify!($reg), "\n",)
    };
}

#[macro_export]
macro_rules! untag_bool {
    ($reg:tt) => {
        concat!(
            "and    x",
            stringify!($reg),
            ", x",
            stringify!($reg),
            ", #0x1\n",
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
            "uxtw   x",
            stringify!($reg),
            ", w",
            stringify!($reg),
            "\n",
            "orr    x",
            stringify!($reg),
            ", x16, x",
            stringify!($reg),
            "\n",
        )
    };
}

#[macro_export]
macro_rules! tag_object_ref {
    ($reg:tt) => {
        concat!(
            "movz   x16, #0x5, lsl #32\n",
            "movk   x16, #0x7ff8, lsl #48\n",
            "uxtw   x",
            stringify!($reg),
            ", w",
            stringify!($reg),
            "\n",
            "orr    x",
            stringify!($reg),
            ", x16, x",
            stringify!($reg),
            "\n",
        )
    };
}

#[macro_export]
macro_rules! tag_undefined {
    ($reg:tt) => {
        concat!(
            "movz   x",
            stringify!($reg),
            ", #0x1, lsl #32\n",
            "movk   x",
            stringify!($reg),
            ", #0x7ff8, lsl #48\n",
        )
    };
}

#[macro_export]
macro_rules! tag_null {
    ($reg:tt) => {
        concat!(
            "movz   x",
            stringify!($reg),
            ", #0x2, lsl #32\n",
            "movk   x",
            stringify!($reg),
            ", #0x7ff8, lsl #48\n",
        )
    };
}

#[macro_export]
macro_rules! tag_bool_const {
    ($reg:tt, $payload:literal) => {
        concat!(
            "movz   x",
            stringify!($reg),
            ", #",
            stringify!($payload),
            "\n",
            "movk   x",
            stringify!($reg),
            ", #0x3, lsl #32\n",
            "movk   x",
            stringify!($reg),
            ", #0x7ff8, lsl #48\n",
        )
    };
}

/// Tag a Boolean payload already materialized as 0/1 in `$reg`.
///
/// `AArch64` W-register writes zero-extend into the paired X-register, so
/// callers can feed this directly from `cset w{reg}, cond`. The macro
/// masks the payload to keep the value representation tight, then ORs
/// in the Boolean tag/header bits.
#[macro_export]
macro_rules! tag_bool_payload {
    ($reg:tt) => {
        concat!(
            "and    x",
            stringify!($reg),
            ", x",
            stringify!($reg),
            ", #0x1\n",
            "movz   x16, #0x3, lsl #32\n",
            "movk   x16, #0x7ff8, lsl #48\n",
            "orr    x",
            stringify!($reg),
            ", x16, x",
            stringify!($reg),
            "\n",
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
            "movz   x",
            stringify!($reg),
            ", #",
            stringify!($payload),
            "\n",
            "movk   x",
            stringify!($reg),
            ", #0x4, lsl #32\n",
            "movk   x",
            stringify!($reg),
            ", #0x7ff8, lsl #48\n",
        )
    };
}

/// Tag a signed-byte payload (already in `$reg` as the low byte of a
/// w-register, zero-extended by the decode prologue's `ldrb`) into a
/// tagged SMI Value in `$reg`. Sign-extends w-byte → w-word with `sxtb`,
/// zero-extends w-word → x-word with `uxtw` (clears bits 32-63 so the
/// subsequent OR composes cleanly), materializes the SMI tag pattern
/// (kind=0x4, header=0x7ff8) in scratch `x16`, then OR-s the tag into
/// `$reg`.
///
/// 5 instructions: sxtb + uxtw + movz + movk + orr. Used by
/// `op_load_smi8` (i8 payload) and similar narrow-SMI loaders that
/// need sign-extension before tagging.
///
/// Distinct from `tag_smi!` (assumes payload is already an i32 in
/// `$reg`'s low word — no sign-extension) and `tag_smi_const!` (folds
/// a compile-time literal payload into the materialized constant).
#[macro_export]
macro_rules! tag_smi_from_signed_byte {
    ($reg:tt) => {
        concat!(
            "sxtb   w",
            stringify!($reg),
            ", w",
            stringify!($reg),
            "\n",
            "uxtw   x",
            stringify!($reg),
            ", w",
            stringify!($reg),
            "\n",
            "movz   x16, #0x4, lsl #32\n",
            "movk   x16, #0x7ff8, lsl #48\n",
            "orr    x",
            stringify!($reg),
            ", x16, x",
            stringify!($reg),
            "\n",
        )
    };
}

// ===========================================================================
// Sentinel materialization (Phase 1.B.2).
// ===========================================================================

/// Materialize the `Value::uninitialized_lexical()` 64-bit sentinel
/// into the destination register. Used by `op_load_this` to compare
/// against the pre-resolved `frame_this_value` mirror; on match,
/// the handler bails to the slow path which resolves the actual
/// `ThisState` (Uninitialized → throw `ReferenceError`; Lexical → walk
/// lex-env).
///
/// The Apple Silicon `AArch64` assembler (clang's integrated assembler
/// driven by rustc's `naked_asm!`) rejects the `ldr {dst}, =literal`
/// literal-pool form inside a `naked_asm!` block — there's no enclosing
/// function for the assembler to attach the literal pool to, and the
/// inline-asm parser doesn't synthesize one. The existing tag macros
/// (`tag_smi_const!`, `tag_undefined!`, etc.) all use `movz` + `movk`
/// for the same reason; this macro mirrors that pattern.
///
/// The sentinel is `Value::uninitialized_lexical()`, which is
/// `tagged(TagKind::Sentinel = 9, InternalSentinel::UninitializedLexical.raw() = 2)`
/// = `0x7ff8_0009_0000_0002`. We materialize it in 4 instructions:
/// movz the low quarter (payload bits 0-15), then movk the three
/// higher quarters at lsl #16, #32, #48. The named binding
/// `value_uninit_lex_bits` carries the full 64-bit pattern, and the
/// `>> N & 0xffff` arithmetic in the immediate slot is evaluated by
/// the assembler at template-substitution time.
///
/// ## Emitted shape (4 instructions)
///
/// ```text
///     movz  x{dst}, #{value_uninit_lex_bits} & 0xffff
///     movk  x{dst}, #({value_uninit_lex_bits} >> 16) & 0xffff, lsl #16
///     movk  x{dst}, #({value_uninit_lex_bits} >> 32) & 0xffff, lsl #32
///     movk  x{dst}, #({value_uninit_lex_bits} >> 48) & 0xffff, lsl #48
/// ```
///
/// ## Argument conventions
///
/// - `$dst_reg` is the scratch register number (the lowerer's
///   `t0..t6` slots have already been substituted by macro-expansion
///   time).
/// - `value_uninit_lex_bits` is a `naked_asm!`-supplied named binding
///   added to the lowerer's universal binding set in Phase 1.B.2 Task 1.
///
/// See spec §3.2.
#[macro_export]
macro_rules! load_uninit_lex_sentinel {
    ($dst_reg:tt) => {
        concat!(
            "movz   x",
            stringify!($dst_reg),
            ", #({value_uninit_lex_bits} & 0xffff)\n",
            "movk   x",
            stringify!($dst_reg),
            ", #(({value_uninit_lex_bits} >> 16) & 0xffff), lsl #16\n",
            "movk   x",
            stringify!($dst_reg),
            ", #(({value_uninit_lex_bits} >> 32) & 0xffff), lsl #32\n",
            "movk   x",
            stringify!($dst_reg),
            ", #(({value_uninit_lex_bits} >> 48) & 0xffff), lsl #48\n",
        )
    };
}
