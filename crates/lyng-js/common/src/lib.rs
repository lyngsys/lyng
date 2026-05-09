//! Shared infrastructure for the lyng-js JavaScript engine.
//!
//! This crate owns the types shared across the full engine that are not
//! runtime values: source locations, atoms, and diagnostics.

#![allow(
    clippy::cast_possible_truncation,
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::redundant_closure_for_method_calls,
    clippy::return_self_not_must_use,
    reason = "shared atom and source primitives use compact u32 IDs, ECMA-262 wording, and cheap public accessors"
)]

mod atom;
mod diagnostic;
mod source;

pub use atom::{
    AtomCollection, AtomId, AtomLifetime, AtomSweepStats, AtomTable, WellKnownAtom,
    WELL_KNOWN_ATOMS,
};
pub use diagnostic::{Diagnostic, DiagnosticList, Severity};
pub use source::{SourceId, Span, TextOffset, TextRange};
