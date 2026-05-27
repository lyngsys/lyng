//! `CallStatus` + `ConstructStatus` projections (Spec 2 Phase E).

use lyng_types::{BuiltinFunctionId, ObjectRef, RealmRef, ShapeId};

use crate::vm::feedback::FeedbackInlineCacheState;

/// Compact summary of one cached callee/constructor for the per-kind status API.
///
/// For Call sites, this surfaces the cached `(callee, callee_shape)` plus
/// any decoded builtin id / realm. For Construct sites, it surfaces the
/// cached `(constructor, constructor_shape)` plus an optional cached
/// `created_shape` from the most recent observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CalleeSummary {
    /// Cached function object (callee for Call, constructor for Construct).
    pub function: ObjectRef,
    /// Shape of the cached function object at observation time.
    pub function_shape: ShapeId,
    /// Realm of the cached function, if it carries function object data.
    pub realm: Option<RealmRef>,
    /// Builtin entry id of the cached function, if it is a native builtin.
    pub builtin: Option<BuiltinFunctionId>,
    /// For Construct: the shape of the freshly-created instance the last
    /// time the IC observed this constructor. `None` for Call sites and
    /// for Construct sites that haven't yet seen a created object.
    pub created_shape: Option<ShapeId>,
}

/// Status projection for one `Call` IC slot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallStatus {
    pub state: FeedbackInlineCacheState,
    pub generation: u32,
    pub execution_count: u32,
    /// Monomorphic-only callee summary. `None` when the slot is
    /// Uninitialized, Megamorphic, or Polymorphic (>= 2 callees observed).
    pub callee: Option<CalleeSummary>,
    /// Polymorphic-only entry list (1..=N callees observed). Empty for
    /// non-polymorphic slots; populated alongside `callee` when monomorphic
    /// and equal-length to the active callee count when polymorphic.
    pub entries: Vec<CalleeSummary>,
    /// Compile-time expected arity for this call site, if the bytecode
    /// descriptor recorded one.
    pub expected_arity: Option<u16>,
}

impl CallStatus {
    /// Convenience accessor — the IC state machine variant.
    #[inline]
    #[must_use]
    pub const fn state(&self) -> FeedbackInlineCacheState {
        self.state
    }

    /// Convenience accessor — the compile-time expected arity, if any.
    #[inline]
    #[must_use]
    pub const fn expected_arity(&self) -> Option<u16> {
        self.expected_arity
    }
}

/// Status projection for one `Construct` IC slot. Same shape as
/// `CallStatus`; the kind distinction is implicit in the query method.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructStatus {
    pub state: FeedbackInlineCacheState,
    pub generation: u32,
    pub execution_count: u32,
    pub callee: Option<CalleeSummary>,
    pub entries: Vec<CalleeSummary>,
    pub expected_arity: Option<u16>,
}

impl ConstructStatus {
    /// Convenience accessor — the IC state machine variant.
    #[inline]
    #[must_use]
    pub const fn state(&self) -> FeedbackInlineCacheState {
        self.state
    }

    /// Convenience accessor — the compile-time expected arity, if any.
    #[inline]
    #[must_use]
    pub const fn expected_arity(&self) -> Option<u16> {
        self.expected_arity
    }
}
