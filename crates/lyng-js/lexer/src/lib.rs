//! Streaming lexer for the ECMA-262 Edition 16 lexical grammar.
//!
//! This crate tokenizes JavaScript source text one token at a time. The
//! parser drives the lexer via `next_token()` and controls lexer mode for
//! disambiguating `/` (division vs regexp) and template continuations.

#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    clippy::match_same_arms,
    clippy::doc_markdown
)]

mod lexer;
mod literals;
mod token;

pub use lexer::{Lexer, LexerMode};
pub use literals::{BigIntLiteral, LiteralTable, RegExpLiteral, StringLiteral, TemplateChunk};
pub use token::{LiteralId, Token, TokenFlags, TokenKind, TokenPayload};

#[cfg(test)]
mod tests;
