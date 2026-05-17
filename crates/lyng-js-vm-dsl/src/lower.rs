//! AST → `naked_asm!` body lowerer.
//!
//! The lowerer is where the design's "DSL surface ≈ asm shape" decision
//! pays out: each body statement is one DSL-op invocation, and the
//! lowerer composes them all into a single `naked_asm!` template. Each
//! per-arch DSL-op `macro_rules!` macro (under
//! `crates/lyng-js/vm/src/dsl/backend/aarch64/`, added in Batch 4 /
//! tasks B20–B28) expands at the *consumer crate's* call site into a
//! `concat!`-produced `&'static str` fragment.
//!
//! ## Emission shape
//!
//! For a handler `op_xxx, layout = L, length = N, |args| { m1!(...); m2!(...); }`:
//!
//! ```ignore
//! #[unsafe(naked)]
//! pub extern "C" fn op_xxx() -> ! {
//!     ::core::arch::naked_asm!(
//!         "/* len={length} */\n",    // unconditional reference to {length}
//!         "<L decode prologue>\n",   // string literal from `Layout::decode_prologue_asm`
//!         m1!(...),                  // macro call returning &'static str (via concat!)
//!         m2!(...),                  // ditto
//!         length = const N as u32,
//!     )
//! }
//! ```
//!
//! `naked_asm!` accepts a comma-separated list of string-expression
//! fragments for its template (per the inline-assembly reference). Each
//! macro call in the list expands to a `concat!(...)`-produced literal,
//! which is exactly what the template syntax wants — no outer `concat!`
//! wrapper is needed.
//!
//! ## `noreturn` and `length`
//!
//! `naked_asm!` *implicitly* requires `noreturn` (it's the only valid
//! mode for a naked-function body); passing `options(noreturn)`
//! explicitly is a hard rustc error. So the emission carries no
//! `options(...)` clause.
//!
//! The `length = const N as u32` binding is referenced by the bare
//! `dispatch!()` macro (which auto-advances PC by `{length}`). For
//! handlers that only use `dispatch!(advance = N)` (literal advance)
//! the named arg would be unused — rustc rejects that. We sidestep the
//! issue by emitting a fixed `"/* len={length} */\n"` comment line
//! that always references `{length}`; asm comments are stripped by
//! the assembler so this costs nothing at runtime.
//!
//! ## Why the body is parsed (not raw-spliced)
//!
//! The earlier Batch-1 lowerer interpolated the body as a raw
//! `TokenStream`. That carries the trailing `;` from each statement
//! straight into `naked_asm!`, which rejects it (`expected token: ,`).
//! B30 forced us to parse the body into `BodyStmt::MacroCall` entries
//! and splice them comma-separated — see [`crate::parse::parse_body`].

use proc_macro2::TokenStream;
use quote::quote;
use syn::Result;

use crate::layouts::Layout;
use crate::parse::{BodyStmt, HandlerAst};
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
    let body_calls = ast.body.iter().map(|stmt| match stmt {
        BodyStmt::MacroCall(tokens) => tokens.clone(),
    });

    // Compose the template list: prologue first, then one entry per
    // body macro call. Emit each followed by `,` so an empty body
    // produces no stray comma.
    let template_entries = std::iter::once(quote! { #prologue })
        .chain(body_calls)
        .map(|tokens| quote! { #tokens, });

    Ok(quote! {
        #[unsafe(naked)]
        pub extern "C" fn #name() -> ! {
            // `naked_asm!` implies `noreturn` (it's the only valid mode
            // for a naked function body) — explicitly passing
            // `options(noreturn)` is a hard error. The body is a
            // comma-separated list of string-typed template expressions
            // followed by named-binding entries.
            //
            // The leading `"/* len={length} */"` comment fragment
            // unconditionally references the `{length}` named binding
            // so rustc never complains about an unused named arg, even
            // for handlers that only use `dispatch!(advance = N)` and
            // never the bare `dispatch!()` that consumes `{length}`.
            // Asm comments are stripped by the assembler, so this is
            // free at runtime.
            ::core::arch::naked_asm!(
                "/* len={length} */\n",
                // Each body fragment is a backend `macro_rules!` call
                // (e.g. `dispatch!(advance = 0)`) that expands to a
                // `concat!(...)`-produced `&'static str`. `naked_asm!`
                // accepts a comma-separated list of such string-typed
                // expressions as its template — no outer `concat!`
                // wrapper needed.
                #(#template_entries)*
                length = const #length as u32,
            )
        }
    })
}
