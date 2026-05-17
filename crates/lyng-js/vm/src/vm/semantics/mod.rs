//! Free-function semantic bodies per design §10 DSL-0a.
//!
//! Each `op_xxx_semantic` function implements the semantic effect of
//! one bytecode opcode. The α handler in `dispatch_handlers/` decodes
//! operands and calls into one of these; in DSL-0b the same function
//! is also reachable from the DSL cold-stub shim in
//! `crates/lyng-js/vm/src/dsl/slow_path.rs`.
//!
//! Per-family submodules are added by family-extraction tasks A8–A18.
//! `OpXxxArgs` structs live alongside their semantic body.

// Family submodules are added by tasks A8–A18.
pub(crate) mod arithmetic;
pub(crate) mod calls;
pub(crate) mod control_flow;
pub(crate) mod generators;
pub(crate) mod iterators;
pub(crate) mod loads;
pub(crate) mod names;
pub(crate) mod property;
pub(crate) mod scope;
