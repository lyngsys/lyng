//! syn-based AST for `llint_handler! { ... }` invocations.
//!
//! Input shape (see design §4):
//! ```ignore
//! llint_handler! {
//!     name, layout = LayoutIdent, length = N, |a, b, c, slot| { <body> }
//! }
//! ```
//!
//! `<body>` is captured as a raw `TokenStream` and forwarded to the
//! lowerer. The proc-macro deliberately does NOT pre-expand body macros;
//! the per-arch DSL-op `macro_rules!` macros (defined under
//! `crates/lyng-js/vm/src/dsl/backend/aarch64/` in later tasks) emit asm
//! string fragments that get concatenated by `naked_asm!` at the call
//! site.

use proc_macro2::TokenStream;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, LitInt, Result, Token,
};

pub(crate) struct HandlerAst {
    /// Function name (e.g. `op_add`).
    pub(crate) name: Ident,
    /// Layout identifier (e.g. `AbcSlot`, `None`).
    pub(crate) layout: Ident,
    /// Encoded instruction length, used as the `length = const N as u32`
    /// option of `naked_asm!` for tooling/debug purposes.
    pub(crate) length: LitInt,
    /// Named operand bindings (e.g. `a, b, c, slot`). May be empty for
    /// `layout = None` handlers (the input then looks like `|| { ... }`).
    pub(crate) operand_idents: Punctuated<Ident, Token![,]>,
    /// Raw handler body — passed through to `naked_asm!` so backend
    /// `macro_rules!` macros expand at the call site.
    pub(crate) body: TokenStream,
}

impl Parse for HandlerAst {
    fn parse(input: ParseStream) -> Result<Self> {
        // `<name>,`
        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        // `layout = <ident>,`
        let layout_ident: Ident = input.parse()?;
        if layout_ident != "layout" {
            return Err(syn::Error::new(
                layout_ident.span(),
                "expected `layout = ...`",
            ));
        }
        input.parse::<Token![=]>()?;
        let layout: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        // `length = <lit>,`
        let length_ident: Ident = input.parse()?;
        if length_ident != "length" {
            return Err(syn::Error::new(
                length_ident.span(),
                "expected `length = ...`",
            ));
        }
        input.parse::<Token![=]>()?;
        let length: LitInt = input.parse()?;
        input.parse::<Token![,]>()?;

        // `|<idents>?| { <body> }`. The operand list may be empty for
        // `layout = None` handlers, so `parse_separated_nonempty` is
        // unsuitable — parse manually instead.
        input.parse::<Token![|]>()?;
        let mut operand_idents: Punctuated<Ident, Token![,]> = Punctuated::new();
        while !input.peek(Token![|]) {
            let ident: Ident = input.parse()?;
            operand_idents.push_value(ident);
            if input.peek(Token![|]) {
                break;
            }
            let comma: Token![,] = input.parse()?;
            operand_idents.push_punct(comma);
        }
        input.parse::<Token![|]>()?;

        let body_content;
        braced!(body_content in input);
        let body: TokenStream = body_content.parse()?;

        Ok(HandlerAst {
            name,
            layout,
            length,
            operand_idents,
            body,
        })
    }
}

pub(crate) fn parse_handler(input: TokenStream) -> Result<HandlerAst> {
    syn::parse2(input)
}
