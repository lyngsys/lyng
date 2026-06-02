//! Free-function semantic bodies.
//!
//! Each `op_xxx_semantic` function implements the semantic effect of one
//! bytecode opcode. The handler decodes operands and calls the semantic
//! body; the DSL cold-stub shim in `dsl/slow_path.rs` reaches the same
//! functions from the asm path. `OpXxxArgs` structs live alongside their
//! semantic body.

#![allow(
    clippy::needless_pass_by_value,
    reason = "Semantic bodies share a uniform dispatch surface where small Copy operand structs are passed by value"
)]

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
