//! Flat-array feedback storage placeholder for the DSL `FV` pin.
//!
//! This file is a **placeholder** for DSL-0b Batch 2. The real
//! `FeedbackEntry` layout (mirroring today's per-site IC state) lands
//! in Batch 3 (Task B14). The forward declaration here keeps
//! `LlIntState`'s `*mut FeedbackEntry` field-type compile cleanly so
//! the ABI layout is stable across the two batches.
//!
//! The placeholder is a zero-sized opaque struct so the pointer width
//! matches whatever the eventual flat-array entry type ends up being —
//! `*mut FeedbackEntryStub` and `*mut FeedbackEntry` are interchangeable
//! at the ABI level.

/// Forward-declared placeholder for `FeedbackEntry`. The real layout
/// lands in Batch 3 (Task B14); see plan for the migration.
#[doc(hidden)]
#[repr(C)]
pub struct FeedbackEntryStub {
    _private: [u8; 0],
}

/// Alias used by `LlIntState` and any other Batch 2 code. Batch 3
/// replaces the placeholder with the real `FeedbackEntry` struct.
pub type FeedbackEntry = FeedbackEntryStub;
