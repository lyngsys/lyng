//! Operand-layout descriptors.
//!
//! Each variant of `Layout` corresponds to one of the operand decoders in
//! `crates/lyng-js/vm/src/vm/dispatch.rs` (e.g. `decode_abc_operands`,
//! `decode_abx_operands`, `decode_ax_operands`). The proc-macro consumes
//! the layout to (a) validate the operand binding count and (b) emit the
//! per-handler decode prologue.
//!
//! The decode prologue is emitted as a TokenStream that the consumer
//! crate's backend macros (`decode_abc!`, `decode_ab!`, `decode_ax!`,
//! etc., under `crates/lyng-js/vm/src/dsl/backend/aarch64/operands.rs`)
//! expand into a `concat!(...)`-produced asm fragment. The operand
//! identifiers passed here are first substituted to their scratch-register
//! numbers by `lower::substitute_idents`, so the asm comes out with real
//! `w9`, `w10`, ... register names.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Ident, Result};

#[derive(Clone, Copy)]
pub(crate) enum Layout {
    /// Single register operand (e.g. `op_ldar a`).
    A,
    /// Two register operands.
    Ab,
    /// Three register operands (`a, b, c`).
    Abc,
    /// `Abc` plus a feedback-vector slot operand. The hot arithmetic
    /// handlers (`op_add`, etc.) use this.
    AbcSlot,
    /// `a` register + extended `bx` operand.
    Abx,
    /// Extended `ax` operand (used by jump targets).
    Ax,
    /// No operands (only the opcode byte and feedback-slot trailer).
    None,
}

impl Layout {
    pub(crate) fn from_ident(ident: &Ident) -> Result<Self> {
        match ident.to_string().as_str() {
            "Abc" => Ok(Self::Abc),
            "AbcSlot" => Ok(Self::AbcSlot),
            "Abx" => Ok(Self::Abx),
            "Ax" => Ok(Self::Ax),
            "Ab" => Ok(Self::Ab),
            "A" => Ok(Self::A),
            "None" => Ok(Self::None),
            other => Err(Error::new(
                ident.span(),
                format!("unknown layout `{other}`"),
            )),
        }
    }

    pub(crate) fn operand_arity(self) -> usize {
        match self {
            Self::None => 0,
            // `Ax` is the extended (u32-immediate) jump-target form —
            // a single operand binding mapped to a 4-byte read at PC+1.
            Self::A | Self::Ax => 1,
            Self::Ab | Self::Abx => 2,
            Self::Abc => 3,
            Self::AbcSlot => 4,
        }
    }

    /// Emit the operand-decode prologue as a backend-macro invocation
    /// token stream. The lowerer splices this into `naked_asm!`'s
    /// template list. Operand identifiers are passed through the
    /// scratch-substitution pass before reaching the asm; the macros
    /// themselves stringify their arguments to build `wN`/`xN` operands.
    pub(crate) fn decode_prologue_tokens(self, operands: &[Ident]) -> TokenStream {
        match (self, operands) {
            (Self::None, _) => quote! { "" },
            (Self::A, [a]) => quote! { decode_a!(#a) },
            (Self::Ab, [a, b]) => quote! { decode_ab!(#a, #b) },
            (Self::Abc, [a, b, c]) => quote! { decode_abc!(#a, #b, #c) },
            (Self::AbcSlot, [a, b, c, slot]) => {
                quote! { decode_abc_slot!(#a, #b, #c, #slot) }
            }
            (Self::Abx, [a, bx]) => quote! { decode_abx!(#a, #bx) },
            (Self::Ax, [ax]) => quote! { decode_ax!(#ax) },
            _ => {
                // Arity is enforced upstream by `lower_handler`; this
                // arm is unreachable in practice.
                quote! { "" }
            }
        }
    }
}
