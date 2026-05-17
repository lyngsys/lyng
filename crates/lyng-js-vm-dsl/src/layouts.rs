//! Operand-layout descriptors.
//!
//! Each variant of `Layout` corresponds to one of the operand decoders in
//! `crates/lyng-js/vm/src/vm/dispatch.rs` (e.g. `decode_abc_operands`,
//! `decode_abx_operands`, `decode_ax_operands`). The proc-macro consumes
//! the layout to (a) validate the operand binding count and (b) emit the
//! per-handler decode prologue.
//!
//! For DSL-0b Batch 1 the decode-prologue strings are placeholders. The
//! real asm fragments are produced by the per-arch DSL-op macros under
//! `crates/lyng-js/vm/src/dsl/backend/aarch64/` (Batch 4, tasks B20–B28),
//! which compose via `concat!` inside `naked_asm!`.

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
            Self::A => 1,
            Self::Ab => 2,
            Self::Abc | Self::Abx | Self::Ax => 3,
            Self::AbcSlot => 4,
        }
    }

    /// Emit the operand-decode asm fragment that runs at handler entry.
    ///
    /// For Batch 1 each variant returns a placeholder comment. The real
    /// asm — which reads operand bytes from the PC pin and materializes
    /// them into scratch registers — is filled in once the backend
    /// `macro_rules!` macros land (Batch 4). The lowerer composes this
    /// string with the user-provided body inside `naked_asm!`, so this
    /// becomes a no-op once the backend takes over the decode work.
    pub(crate) fn decode_prologue_asm(self, _operands: &[Ident]) -> String {
        match self {
            Self::Abc => "// decode_abc prologue placeholder\n".to_string(),
            Self::AbcSlot => "// decode_abc_slot prologue placeholder\n".to_string(),
            Self::Abx => "// decode_abx prologue placeholder\n".to_string(),
            Self::Ax => "// decode_ax prologue placeholder\n".to_string(),
            Self::Ab => "// decode_ab prologue placeholder\n".to_string(),
            Self::A => "// decode_a prologue placeholder\n".to_string(),
            Self::None => "// decode_none prologue placeholder\n".to_string(),
        }
    }
}
