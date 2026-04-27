use lyng_js_common::AtomId;

/// Minimal placeholder proving the Phase 2 data-only crate owns copyable runtime types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeOwnershipMarker {
    property_name: AtomId,
}

impl TypeOwnershipMarker {
    #[inline]
    pub const fn new(property_name: AtomId) -> Self {
        Self { property_name }
    }

    #[inline]
    pub const fn property_name(self) -> AtomId {
        self.property_name
    }
}
