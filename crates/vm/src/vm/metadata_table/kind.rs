use lyng_bytecode::FeedbackSiteKind;

/// IC metadata kinds in the table layout. Each kind owns its own per-kind run
/// in the buffer. Two `FeedbackSiteKind`s may map to the same `MetadataKind`
/// (e.g. `NamedPropertyLoad` + `NamedPropertyStore` → `Property`).
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum MetadataKind {
    Property = 0,
    Call = 1,
    Arith = 2,
    Comparison = 3,
    KeyedProperty = 4,
}

#[allow(dead_code)]
pub const METADATA_KIND_COUNT: usize = 5;

#[allow(dead_code)]
impl MetadataKind {
    pub const fn from_site_kind(kind: FeedbackSiteKind) -> Self {
        match kind {
            FeedbackSiteKind::NamedPropertyLoad | FeedbackSiteKind::NamedPropertyStore => {
                Self::Property
            }
            FeedbackSiteKind::Call | FeedbackSiteKind::Construct => Self::Call,
            FeedbackSiteKind::Arithmetic => Self::Arith,
            FeedbackSiteKind::Comparison => Self::Comparison,
            FeedbackSiteKind::KeyedPropertyAccess => Self::KeyedProperty,
        }
    }

    pub const fn index(self) -> usize {
        self as usize
    }

    /// Byte size of one metadata entry for this kind. Placeholder values for
    /// Phase C.1; Phase C.2 makes these match the actual per-kind struct sizes.
    pub const fn stride_bytes(self) -> usize {
        match self {
            Self::Property => 32,
            Self::Call => 24,
            Self::Arith => 8,
            Self::Comparison => 8,
            Self::KeyedProperty => 24,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn site_kind_maps_to_metadata_kind() {
        assert_eq!(
            MetadataKind::from_site_kind(FeedbackSiteKind::NamedPropertyLoad),
            MetadataKind::Property
        );
        assert_eq!(
            MetadataKind::from_site_kind(FeedbackSiteKind::NamedPropertyStore),
            MetadataKind::Property
        );
        assert_eq!(
            MetadataKind::from_site_kind(FeedbackSiteKind::Call),
            MetadataKind::Call
        );
        assert_eq!(
            MetadataKind::from_site_kind(FeedbackSiteKind::Construct),
            MetadataKind::Call
        );
        assert_eq!(
            MetadataKind::from_site_kind(FeedbackSiteKind::Arithmetic),
            MetadataKind::Arith
        );
        assert_eq!(
            MetadataKind::from_site_kind(FeedbackSiteKind::Comparison),
            MetadataKind::Comparison
        );
        assert_eq!(
            MetadataKind::from_site_kind(FeedbackSiteKind::KeyedPropertyAccess),
            MetadataKind::KeyedProperty
        );
    }

    #[test]
    fn metadata_kind_count_matches_variants() {
        assert_eq!(MetadataKind::KeyedProperty.index() + 1, METADATA_KIND_COUNT);
    }
}
