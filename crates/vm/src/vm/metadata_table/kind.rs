use lyng_bytecode::FeedbackSiteKind;

use super::arith::ARITH_METADATA_STRIDE;
use super::call::CALL_METADATA_STRIDE;
use super::comparison::COMPARISON_METADATA_STRIDE;
use super::keyed_property::KEYED_PROPERTY_METADATA_STRIDE;
use super::property::PROPERTY_METADATA_STRIDE;

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

    /// Byte size of one metadata entry for this kind. Sourced from the per-kind
    /// STRIDE constants — single source of truth for the buffer layout math.
    pub const fn stride_bytes(self) -> usize {
        match self {
            Self::Property => PROPERTY_METADATA_STRIDE,
            Self::Call => CALL_METADATA_STRIDE,
            Self::Arith => ARITH_METADATA_STRIDE,
            Self::Comparison => COMPARISON_METADATA_STRIDE,
            Self::KeyedProperty => KEYED_PROPERTY_METADATA_STRIDE,
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
