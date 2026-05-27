//! `ArithStatus` projection (Spec 2 Phase E).

/// Observed-operand classification for one `Arithmetic` IC slot, projected
/// from `ArithMetadata.observed_bits`.
///
/// The asm IC writes a small bitfield indicating which operand kinds have
/// been observed. We surface the unpacked flags so tests and tier
/// heuristics don't have to know the raw layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ScalarObserved {
    /// Raw observed-bits word (kept for callers that want low-level access).
    pub raw_bits: u32,
    /// Smi (small integer) operand observed at least once.
    pub smi: bool,
    /// Object operand observed at least once.
    pub object: bool,
    /// Double operand observed at least once.
    pub double: bool,
}

// Bit definitions are taken from the asm IC writer (see
// `drain_llint_scalar_feedback`). Only the SMI bit is currently written by
// asm (`LLINT_FEEDBACK_OBSERVED_SMI = 0x1`), but we decode the higher bits
// defensively so future asm/IC writes are surfaced without changes here.
const ARITH_OBSERVED_SMI: u32 = 0x1;
const ARITH_OBSERVED_OBJECT: u32 = 0x2;
const ARITH_OBSERVED_DOUBLE: u32 = 0x4;

impl ScalarObserved {
    /// Decode the raw observed-bits word into typed flags.
    #[must_use]
    pub const fn from_bits(raw_bits: u32) -> Self {
        Self {
            raw_bits,
            smi: raw_bits & ARITH_OBSERVED_SMI != 0,
            object: raw_bits & ARITH_OBSERVED_OBJECT != 0,
            double: raw_bits & ARITH_OBSERVED_DOUBLE != 0,
        }
    }
}

/// Status projection for one `Arithmetic` IC slot.
///
/// Arithmetic feedback is MetadataTable-owned (no Rust-side IC state):
/// asm writes `observed_bits` and `execution_count` directly into the slot's
/// `ArithMetadata`. This struct surfaces both as a single value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ArithStatus {
    pub observed: ScalarObserved,
    pub execution_count: u32,
}

impl ArithStatus {
    /// Build from the raw `ArithMetadata` fields.
    #[must_use]
    pub const fn from_metadata(observed_bits: u32, execution_count: u32) -> Self {
        Self {
            observed: ScalarObserved::from_bits(observed_bits),
            execution_count,
        }
    }
}
