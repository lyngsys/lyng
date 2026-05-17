//! syn-based AST for `llint_handler! { ... }` invocations.
//!
//! Input shape (see design §4):
//! ```ignore
//! llint_handler! {
//!     name, layout = LayoutIdent, length = N, |a, b, c, slot| { <body> }
//! }
//! ```
//!
//! `<body>` is a sequence of DSL-op macro invocations, each terminated by
//! `;`. The parser splits the body into one [`BodyStmt`] per macro call;
//! the lowerer emits each as a comma-separated argument to the enclosing
//! `naked_asm!`. Backend macros expand to `concat!(...)`-produced
//! `&'static str` fragments, which `naked_asm!` consumes as its template.

use proc_macro2::TokenStream;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, LitInt, Result, Token,
};

/// A single statement of a handler body — either a macro call (`name!(args)`)
/// or a bare label (`.label:`). The lowerer renders each as a string
/// fragment passed positionally to the enclosing `naked_asm!`.
pub(crate) enum BodyStmt {
    /// `dispatch!(...)`, `check_smi!(...)`, etc. The tokens are
    /// re-emitted verbatim; the backend `macro_rules!` macro expands at
    /// the `naked_asm!` call site.
    MacroCall(TokenStream),
}

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
    /// Parsed body statements — one entry per `<macro_call>;` line.
    pub(crate) body: Vec<BodyStmt>,
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
        let body = parse_body(&body_content)?;

        Ok(HandlerAst {
            name,
            layout,
            length,
            operand_idents,
            body,
        })
    }
}

/// Parse a handler body as a sequence of `<macro_call>;` statements.
///
/// The body is intentionally a restricted DSL: each line is a single
/// `name!(...)` invocation followed by `;`. The parser does **not**
/// pre-expand the macros — it preserves the raw tokens of each call so
/// the backend `macro_rules!` macros expand at the consumer crate's
/// call site (where they're in scope).
fn parse_body(input: ParseStream) -> Result<Vec<BodyStmt>> {
    let mut stmts = Vec::new();
    while !input.is_empty() {
        // Each statement is `<ident>!(<args>)` followed by `;`. Use
        // `syn::Macro` to parse the call shape; that gives us the path
        // (just an ident here) and the delimited args, which we re-
        // serialize as a `TokenStream` for the lowerer to splice.
        let mac: syn::Macro = input.parse()?;
        input.parse::<Token![;]>()?;
        // Re-serialize as `<path>!(<tokens>)`. We use `quote!` to
        // preserve hygiene of the macro path so backend macros resolve
        // at the call site.
        let path = &mac.path;
        let tokens = &mac.tokens;
        let call = quote::quote! { #path!(#tokens) };
        stmts.push(BodyStmt::MacroCall(call));
    }
    Ok(stmts)
}

pub(crate) fn parse_handler(input: TokenStream) -> Result<HandlerAst> {
    syn::parse2(input)
}
