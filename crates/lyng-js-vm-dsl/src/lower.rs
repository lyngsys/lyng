//! AST → `naked_asm!` lowerer stub. Implemented in Task B5.

use proc_macro2::TokenStream;
use syn::Result;

use crate::parse::HandlerAst;

pub(crate) fn lower_handler(_ast: HandlerAst) -> Result<TokenStream> {
    Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        "llint_handler! lowerer stub — Task B5",
    ))
}
