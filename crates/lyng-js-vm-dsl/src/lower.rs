//! AST → `naked_asm!` body lowerer.
//!
//! The lowerer is where the design's "DSL surface ≈ asm shape" decision
//! pays out: each body statement is one DSL-op invocation, and the
//! lowerer simply wraps the body in `naked_asm!`. The body tokens are
//! interpolated as-is so that per-arch DSL-op `macro_rules!` macros
//! (under `crates/lyng-js/vm/src/dsl/backend/aarch64/`, added in Batch
//! 4 / tasks B20–B28) expand at the call site into `concat!`-produced
//! string literals consumed by `naked_asm!`.
//!
//! ## Risk acknowledged (load-bearing question of the spike)
//!
//! Whether `naked_asm!` cleanly accepts the macro-string composition
//! described above is the load-bearing question of DSL-0b. Validation
//! case B30 is the first call site that exercises this. If `naked_asm!`
//! rejects the macro-string composition, the lowerer here must instead
//! expand DSL macros into a single string at proc-macro time and emit
//! that as the `naked_asm!` template. That refactor is internal to
//! `lower.rs`; the public DSL surface stays the same.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Result;

use crate::layouts::Layout;
use crate::parse::HandlerAst;
use crate::scratch::ScratchAllocator;

pub(crate) fn lower_handler(ast: HandlerAst) -> Result<TokenStream> {
    let layout = Layout::from_ident(&ast.layout)?;
    let operands: Vec<_> = ast.operand_idents.iter().cloned().collect();
    if operands.len() != layout.operand_arity() {
        return Err(syn::Error::new(
            ast.layout.span(),
            format!(
                "layout {} has arity {}, got {} operand bindings",
                ast.layout,
                layout.operand_arity(),
                operands.len(),
            ),
        ));
    }

    // Pre-assign operand identifiers to scratch registers. The DSL-op
    // backend macros (Batch 4) consult the allocator's mapping via
    // their own lookup tables; for Batch 1 we surface budget overruns
    // at proc-macro expand time even though the body still uses
    // placeholder strings.
    let mut scratch = ScratchAllocator::new();
    for name in &operands {
        scratch.assign(name)?;
    }

    let name = &ast.name;
    let length = &ast.length;
    let prologue = layout.decode_prologue_asm(&operands);
    let body = &ast.body;

    Ok(quote! {
        #[unsafe(naked)]
        pub extern "C" fn #name() -> ! {
            ::core::arch::naked_asm!(
                #prologue,
                // The body is interpolated as a sequence of `concat!`-string
                // expressions. Each backend `macro_rules!` macro expands to
                // a string literal usable inside `naked_asm!`.
                #body
                options(noreturn),
                length = const #length as u32,
            )
        }
    })
}
