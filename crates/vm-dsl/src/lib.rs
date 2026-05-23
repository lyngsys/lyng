//! Proc-macro crate emitting `#[unsafe(naked)] extern "C" fn` DSL handlers.
//!
//! `llint_handler!` parses an offlineasm-flavored handler body (see
//! `docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md` §4) and
//! lowers it to a single `core::arch::naked_asm!` block.
//!
//! Submodules:
//! - `parse`: syn-based AST for handler signatures + bodies.
//! - `layouts`: operand-layout descriptors (Abc, AbcSlot, Abx, Ax, ...).
//! - `scratch`: compile-time scratch-register allocator.
//! - `lower`: AST → `naked_asm!` string assembly.

use proc_macro::TokenStream;

mod layouts;
mod lower;
mod parse;
mod scratch;

/// Define a DSL handler.
///
/// Syntax (see `docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md` §4):
///
/// ```ignore
/// llint_handler! {
///     op_add, layout = AbcSlot, length = 6, |a, b, c, slot| {
///         load_reg!(b => t0);
///         check_smi!(t0, .slow);
///         load_reg!(c => t1);
///         check_smi!(t1, .slow);
///         add_smi_overflow!(t0, t1 => t2, .slow);
///         store_reg!(a, t2);
///         record_smi!(slot);
///         dispatch!();
///
///       .slow:
///         call_slow!(op_add_slow_rs, args = [a, b, c, slot]);
///         dispatch_after_slow!();
///     }
/// }
/// ```
#[proc_macro]
pub fn llint_handler(input: TokenStream) -> TokenStream {
    match parse::parse_handler(input.into()).and_then(lower::lower_handler) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
