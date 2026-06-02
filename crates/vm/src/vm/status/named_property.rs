//! `NamedPropertyStatus` projection (Spec 2 Phase E).

use lyng_objects::{
    NamedPropertyCacheEntry, NamedPropertyCachePath, PROPERTY_CACHE_MAX_DEPENDENCIES,
    PropertyCacheDependency,
};
use lyng_types::{ObjectRef, ShapeId};

use crate::vm::feedback::FeedbackInlineCacheState;

/// Which slow-path cache kind this entry represents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NamedPropertyEntryKind {
    /// Direct own-data property access (no proto walk).
    OwnData,
    /// Own-data property added by a shape transition (store).
    OwnDataTransition,
    /// One-hop prototype data access.
    PrototypeData,
}

impl From<NamedPropertyCachePath> for NamedPropertyEntryKind {
    #[inline]
    fn from(path: NamedPropertyCachePath) -> Self {
        match path {
            NamedPropertyCachePath::OwnData => Self::OwnData,
            NamedPropertyCachePath::OwnDataTransition => Self::OwnDataTransition,
            NamedPropertyCachePath::PrototypeData => Self::PrototypeData,
        }
    }
}

/// Plain-value summary of the cache-entry payload the slow path installed.
///
/// Pulled directly off the `NamedPropertyCacheEntry`; everything tests care
/// about (path, holder, slot offset, dependencies) is surfaced as a value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedPropertyHandlerSummary {
    pub holder: ObjectRef,
    pub holder_shape: ShapeId,
    pub slot_offset: u32,
    pub path: NamedPropertyCachePath,
    pub dependencies: Vec<PropertyCacheDependency>,
}

impl NamedPropertyHandlerSummary {
    pub(crate) fn from_entry(entry: NamedPropertyCacheEntry) -> Self {
        let mut dependencies = Vec::with_capacity(PROPERTY_CACHE_MAX_DEPENDENCIES);
        for i in 0..usize::from(entry.dependency_count()) {
            if let Some(dep) = entry.dependency(i) {
                dependencies.push(dep);
            }
        }
        Self {
            holder: entry.holder(),
            holder_shape: entry.holder_shape(),
            slot_offset: entry.slot_offset(),
            path: entry.path(),
            dependencies,
        }
    }
}

/// One entry in a `NamedPropertyStatus`: a `(receiver_shape, kind, handler)`
/// triple, with the underlying handler summary exposed for tests that need
/// holder/slot-offset/dependency assertions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedPropertyStatusEntry {
    pub receiver_shape: ShapeId,
    pub kind: NamedPropertyEntryKind,
    pub handler_summary: NamedPropertyHandlerSummary,
}

impl NamedPropertyStatusEntry {
    pub(crate) fn from_entry(entry: NamedPropertyCacheEntry) -> Self {
        Self {
            receiver_shape: entry.receiver_shape(),
            kind: entry.path().into(),
            handler_summary: NamedPropertyHandlerSummary::from_entry(entry),
        }
    }

    /// Convenience accessor — the receiver shape this entry was cached for.
    #[inline]
    #[must_use]
    pub const fn receiver_shape(&self) -> ShapeId {
        self.receiver_shape
    }

    /// Convenience accessor — the underlying cache-entry path.
    #[inline]
    #[must_use]
    pub const fn path(&self) -> NamedPropertyCachePath {
        self.handler_summary.path
    }

    /// Convenience accessor — holder reference for the cached property.
    #[inline]
    #[must_use]
    pub const fn holder(&self) -> ObjectRef {
        self.handler_summary.holder
    }

    /// Convenience accessor — dependency list (`PropertyCacheDependency`).
    #[inline]
    #[must_use]
    pub fn dependencies(&self) -> &[PropertyCacheDependency] {
        &self.handler_summary.dependencies
    }
}

/// Status projection for one `NamedProperty` (Load or Store) IC slot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedPropertyStatus {
    pub state: FeedbackInlineCacheState,
    pub generation: u32,
    pub execution_count: u32,
    /// Active inline + chain entries, in ascending receiver-shape order.
    /// Inline shapes are strictly less than chain shapes by construction.
    pub entries: Vec<NamedPropertyStatusEntry>,
}

impl NamedPropertyStatus {
    /// Convenience accessor — the IC state machine variant.
    #[inline]
    #[must_use]
    pub const fn state(&self) -> FeedbackInlineCacheState {
        self.state
    }

    /// Convenience accessor — entries list (inline + chain, shape-ascending).
    #[inline]
    #[must_use]
    pub fn entries(&self) -> &[NamedPropertyStatusEntry] {
        &self.entries
    }
}
