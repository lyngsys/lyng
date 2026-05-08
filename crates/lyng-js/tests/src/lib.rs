//! Integration tests for lyng-js cross-crate contracts and conformance surfaces.
//!
//! Validates frontend coverage, runtime primitive and substrate integration,
//! execution semantics, embedding extension behavior, and large-input smoke cases.

#[cfg(test)]
mod atom_lifetime;

#[cfg(test)]
mod end_to_end;

#[cfg(test)]
mod diagnostics;

#[cfg(test)]
mod parser_coverage;

#[cfg(test)]
mod runtime_primitives_integration;

#[cfg(test)]
mod runtime_primitives;

#[cfg(test)]
mod runtime_substrate_surface;

#[cfg(test)]
mod runtime_substrate_integration;

#[cfg(test)]
mod api_surface;

#[cfg(test)]
mod execution_semantics;

#[cfg(test)]
mod embedding_extensions;

#[cfg(test)]
mod bootstrap_memory;

#[cfg(test)]
mod shared_memory;

#[cfg(test)]
mod temporal;

#[cfg(test)]
mod sema_integration;

#[cfg(test)]
mod smoke;
