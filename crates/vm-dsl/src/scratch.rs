//! Compile-time scratch-register allocator.
//!
//! Maps named operand identifiers and DSL-internal scratch variables
//! (e.g. `t0..t6`) to AArch64 caller-saved register numbers x9..x15.
//! Budget: 7 scratch regs total. Exceeding it produces a `syn::Error`
//! pointing at the offending identifier — the handler must be rewritten
//! to spill or share registers.
//!
//! See design §5 of
//! `docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md` for the
//! pinned-register split (PC/CFR/JSST/FB/etc. occupy x16+ on AArch64,
//! leaving x9..x15 for scratch).

use std::collections::HashMap;
use syn::{Error, Ident, Result};

pub(crate) struct ScratchAllocator {
    /// Maps user-visible names (operand idents like `a`, `slot`, plus
    /// DSL-internal scratch like `t0`) to AArch64 register numbers.
    map: HashMap<String, u8>,
    /// Index into the budget — next free slot.
    next: u8,
}

impl ScratchAllocator {
    /// Maximum number of distinct named scratch slots a single handler
    /// may allocate. Mirrors the design's per-arch scratch budget.
    pub(crate) const BUDGET: u8 = 7;
    /// First AArch64 caller-saved register usable for scratch (x9).
    pub(crate) const FIRST: u8 = 9;

    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
            next: 0,
        }
    }

    /// Assign `name` a scratch register, or return the previously-assigned
    /// register if `name` was already seen.
    ///
    /// Returns the AArch64 register number (e.g. `9` for `x9`).
    pub(crate) fn assign(&mut self, name: &Ident) -> Result<u8> {
        if let Some(&reg) = self.map.get(&name.to_string()) {
            return Ok(reg);
        }
        if self.next >= Self::BUDGET {
            return Err(Error::new(
                name.span(),
                format!(
                    "DSL handler exceeded scratch-register budget of {} (would assign x{} to `{}`)",
                    Self::BUDGET,
                    Self::FIRST + self.next,
                    name,
                ),
            ));
        }
        let reg = Self::FIRST + self.next;
        self.next += 1;
        self.map.insert(name.to_string(), reg);
        Ok(reg)
    }

    /// Look up a previously-assigned scratch register without allocating.
    pub(crate) fn lookup(&self, name: &str) -> Option<u8> {
        self.map.get(name).copied()
    }

    /// Assign or look up an identifier *only if* it's a known scratch
    /// name shape — operand bindings (already pre-assigned) or one of the
    /// reserved `t0..t6` internal scratch names. Returns `Some(reg)` if a
    /// substitution should happen, `None` if the identifier should pass
    /// through unchanged (e.g. a label name, a macro name, or an unrelated
    /// Rust ident inside a macro body).
    ///
    /// This is the substitution-eligibility test the lowerer uses when
    /// walking the body TokenStream. Operand idents (`a`, `b`, `c`,
    /// `slot`, `src`, `dst`, etc.) are recognized via the pre-assignment
    /// path (`lookup` hits); `t0..t6` are recognized via the reserved
    /// prefix and allocated on first use.
    pub(crate) fn substitute(&mut self, name: &Ident) -> Result<Option<u8>> {
        let s = name.to_string();
        if let Some(reg) = self.lookup(&s) {
            return Ok(Some(reg));
        }
        // Reserved internal-scratch slots t0..t6.
        if let Some(rest) = s.strip_prefix('t') {
            if let Ok(idx) = rest.parse::<u8>() {
                if idx < Self::BUDGET {
                    let reg = self.assign(name)?;
                    return Ok(Some(reg));
                }
            }
        }
        Ok(None)
    }
}
