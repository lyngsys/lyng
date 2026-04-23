//! ECMA-262 abstract operations for the lyng-js staged runtime.
//!
//! Ownership: `lyng_js_ops` owns spec-facing semantic helpers built on top of the
//! representation layer in `lyng_js_types`, the storage layer in `lyng_js_gc`,
//! and, in Phase 4, the public environment and object substrate entrypoints.

#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use
)]

pub mod allocating;
mod context;
mod convert;
pub mod enumeration;
pub mod errors;
pub mod iterator;
mod marker;
pub mod names;
pub mod object;
pub mod promise;
pub mod proxy;
pub mod pure;
pub mod read;
pub mod shared_memory;
pub mod temporal;

pub use context::PrimitiveContext;
pub use convert::number_to_string;
pub use marker::PrimitiveOpsMarker;
