//! Handler-body parser stub. Implemented in Task B2.

use proc_macro2::TokenStream;
use syn::Result;

pub(crate) struct HandlerAst {
    // Populated by Task B2.
    pub(crate) _placeholder: (),
}

pub(crate) fn parse_handler(_input: TokenStream) -> Result<HandlerAst> {
    Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        "llint_handler! parser stub — Task B2",
    ))
}
