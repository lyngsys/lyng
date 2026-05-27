//! Per-kind IC `Status` projections (Spec 2 Phase E).
//!
//! Each `*Status` type is a plain value (`Clone`, no GC roots) projecting
//! a single IC slot's observable state from the Vec-indexed side-tables
//! plus the slot's [`MetadataTable`](crate::vm::metadata_table::MetadataTable)
//! entry. Callers can keep the value across subsequent IC mutations.
//!
//! The per-kind API is more ergonomic than a walk-the-vector snapshot:
//! each test or consumer already knows which kind it is asserting on, and
//! the projection reads exactly one slot's worth of state.
//!
//! Construction lives on [`Vm`](crate::Vm) (see
//! [`Vm::named_property_status`](crate::Vm::named_property_status) and
//! friends). This module owns the value types and the entry-summary helpers.

mod arith;
mod call;
mod comparison;
mod footprint;
mod keyed_property;
mod named_property;

pub use arith::{ArithStatus, ScalarObserved};
pub use call::{CallStatus, CalleeSummary, ConstructStatus};
pub use comparison::ComparisonStatus;
pub use footprint::MetadataTableFootprint;
pub use keyed_property::{
    KeyedPropertyDenseStatusEntry, KeyedPropertyNamedStatusEntry, KeyedPropertyStatus,
};
pub use named_property::{
    NamedPropertyEntryKind, NamedPropertyHandlerSummary, NamedPropertyStatus,
    NamedPropertyStatusEntry,
};
