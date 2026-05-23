//! AArch64-specific constants referenced by DSL operation macros:
//! NaN-tag masks, kind discriminator values, layout-decode helpers.
//!
//! Authoritative source: [`reports/js/lyng-js/llint-dsl-value-layout.md`]
//! and [`reports/js/lyng-js/llint-dsl-abi.md`].
//!
//! Encoding recap (NaN-tag-space, kind in bits 32-47):
//!
//! ```text
//!  63                51                  47                  31                                0
//!   |                 |                   |                   |                                 |
//!   0111 1111 1111 1000   kkkk kkkk kkkk kkkk   pppp pppp pppp pppp pppp pppp pppp pppp pppp pppp
//!   \_________ ____________/  \_________ _________/  \_______________ ______________/
//!             |                         |                            |
//!        TAG_HEADER                TAG_KIND_MASK                  PAYLOAD_MASK
//!    0x7ff8_0000_0000_0000     0x0000_ffff_0000_0000          0x0000_0000_ffff_ffff
//! ```
//!
//! Every `check_*!` macro that targets a tag-kind K compares the
//! `TAG_HEADER | (K << 32)` composite against the high 32 bits of the
//! Value (the low 32 are payload). For singletons (`undefined`, `null`)
//! the full 64-bit value can be compared because payload is zero.

use lyng_js_types::Value;

// Core mask family (matches `value.rs` private constants).
pub const VALUE_TAG_HEADER: u64 = 0x7ff8_0000_0000_0000;
pub const VALUE_TAG_KIND_MASK: u64 = 0x0000_ffff_0000_0000;
pub const VALUE_PAYLOAD_MASK: u64 = 0x0000_0000_ffff_ffff;
pub const VALUE_CANONICAL_NAN_BITS: u64 = 0x7ff8_0000_0000_0000;

// Bit shift to place the 16-bit kind into bits 32-47.
pub const VALUE_TAG_KIND_SHIFT: u32 = 32;

// SMI payload width (32 bits, signed).
pub const VALUE_SMI_PAYLOAD_BITS: u32 = 32;

// TagKind discriminator values (mirror `value.rs` `TagKind` enum).
pub const TAG_KIND_UNDEFINED: u64 = 1;
pub const TAG_KIND_NULL: u64 = 2;
pub const TAG_KIND_BOOLEAN: u64 = 3;
pub const TAG_KIND_SMI: u64 = 4;
pub const TAG_KIND_OBJECT_REF: u64 = 5;
pub const TAG_KIND_STRING_REF: u64 = 6;
pub const TAG_KIND_SYMBOL_REF: u64 = 7;
pub const TAG_KIND_BIGINT_REF: u64 = 8;
pub const TAG_KIND_SENTINEL: u64 = 9;
pub const TAG_KIND_SUSPENDED_EXEC_REF: u64 = 10;

// Composite "header | kind << 32" patterns that backend macros use as
// immediate-compare targets. These are the high half of the Value bits
// for each kind; the low 32 bits are payload.
pub const VALUE_TAG_SMI_PATTERN: u64 = VALUE_TAG_HEADER | (TAG_KIND_SMI << VALUE_TAG_KIND_SHIFT);
pub const VALUE_TAG_OBJECT_REF_PATTERN: u64 =
    VALUE_TAG_HEADER | (TAG_KIND_OBJECT_REF << VALUE_TAG_KIND_SHIFT);
pub const VALUE_TAG_STRING_REF_PATTERN: u64 =
    VALUE_TAG_HEADER | (TAG_KIND_STRING_REF << VALUE_TAG_KIND_SHIFT);
pub const VALUE_TAG_SYMBOL_REF_PATTERN: u64 =
    VALUE_TAG_HEADER | (TAG_KIND_SYMBOL_REF << VALUE_TAG_KIND_SHIFT);
pub const VALUE_TAG_BIGINT_REF_PATTERN: u64 =
    VALUE_TAG_HEADER | (TAG_KIND_BIGINT_REF << VALUE_TAG_KIND_SHIFT);
pub const VALUE_TAG_BOOL_PATTERN: u64 =
    VALUE_TAG_HEADER | (TAG_KIND_BOOLEAN << VALUE_TAG_KIND_SHIFT);
pub const VALUE_TAG_SENTINEL_PATTERN: u64 =
    VALUE_TAG_HEADER | (TAG_KIND_SENTINEL << VALUE_TAG_KIND_SHIFT);

// Full 64-bit canonical bit patterns for singletons (payload = 0).
pub const VALUE_UNDEFINED_BITS: u64 =
    VALUE_TAG_HEADER | (TAG_KIND_UNDEFINED << VALUE_TAG_KIND_SHIFT);
pub const VALUE_NULL_BITS: u64 = VALUE_TAG_HEADER | (TAG_KIND_NULL << VALUE_TAG_KIND_SHIFT);
pub const VALUE_TRUE_BITS: u64 = VALUE_TAG_BOOL_PATTERN | 1;
pub const VALUE_FALSE_BITS: u64 = VALUE_TAG_BOOL_PATTERN;

/// 64-bit bit pattern of `Value::uninitialized_lexical()`. Used by the
/// `load_uninit_lex_sentinel!` backend macro to materialize the sentinel
/// for sentinel-bail comparisons in `op_load_this` and any future opcode
/// that needs to compare against this sentinel.
///
/// `Value::uninitialized_lexical()` is a `const fn` returning
/// `Self::tagged(TagKind::Sentinel, InternalSentinel::UninitializedLexical.raw())`,
/// so this const folds at compile time. The compile-time assertion below
/// pins the relationship.
pub const VALUE_UNINIT_LEX_BITS: u64 = Value::uninitialized_lexical().bits();

// Mask combining TAG_HEADER + TAG_KIND_MASK — the bits a `check_kind!`
// macro AND's into a scratch reg before comparing against the pattern.
pub const VALUE_TAG_KIND_AND_HEADER_MASK: u64 = VALUE_TAG_HEADER | VALUE_TAG_KIND_MASK;

// Compile-time sanity: verify our hand-computed patterns match what the
// Value constructors actually emit. The TagKind enum is private so we
// pin via the public `Value::from_smi` / `Value::undefined` /
// `Value::null` accessors. Any mismatch (e.g. someone reorders the
// TagKind enum in `value.rs`) is caught at build time.
const _: () = {
    assert!(Value::from_smi(0).bits() == VALUE_TAG_SMI_PATTERN);
    assert!(Value::undefined().bits() == VALUE_UNDEFINED_BITS);
    assert!(Value::null().bits() == VALUE_NULL_BITS);
    assert!(Value::from_bool(true).bits() == VALUE_TRUE_BITS);
    assert!(Value::from_bool(false).bits() == VALUE_FALSE_BITS);
    // Phase 1.B.2: pin sentinel bits to the runtime constructor.
    assert!(Value::uninitialized_lexical().bits() == VALUE_UNINIT_LEX_BITS);
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_uninit_lex_bits_matches_runtime() {
        assert_eq!(VALUE_UNINIT_LEX_BITS, Value::uninitialized_lexical().bits());
        // Sanity: the sentinel must be distinguishable from common Values.
        assert_ne!(VALUE_UNINIT_LEX_BITS, Value::undefined().bits());
        assert_ne!(VALUE_UNINIT_LEX_BITS, Value::null().bits());
        assert_ne!(VALUE_UNINIT_LEX_BITS, Value::from_smi(0).bits());
    }
}
