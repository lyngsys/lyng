//! Free-function semantic bodies per design §10 DSL-0a.
//!
//! Each `op_xxx_semantic` function implements the semantic effect of
//! one bytecode opcode. The α handler in `dispatch_handlers/` decodes
//! operands and calls into one of these; in DSL-0b the same function
//! is also reachable from the DSL cold-stub shim in
//! `crates/vm/src/dsl/slow_path.rs`.
//!
//! Per-family submodules are added by family-extraction tasks A8–A18.
//! `OpXxxArgs` structs live alongside their semantic body.

#![allow(
    clippy::needless_pass_by_value,
    reason = "Semantic bodies share a uniform legacy-dispatch and LLInt-shim surface where small Copy operand structs are passed by value"
)]

// Family submodules are added by tasks A8–A18.
pub mod arithmetic;
pub mod calls;
pub mod control_flow;
pub mod exceptions;
pub mod generators;
pub mod iterators;
pub mod loads;
pub mod misc;
pub mod names;
pub mod prefix;
pub mod property;
pub mod scope;
