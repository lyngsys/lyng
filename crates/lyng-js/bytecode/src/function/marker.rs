use crate::ids::BytecodeFunctionId;
use lyng_js_common::SourceId;
use lyng_js_types::FeedbackSlotId;

/// Minimal marker proving bytecode template identity is compiler-owned before runtime installation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BytecodeMarker {
    source: SourceId,
    entry: BytecodeFunctionId,
    feedback_slot: FeedbackSlotId,
}

impl BytecodeMarker {
    #[inline]
    pub const fn new(
        source: SourceId,
        entry: BytecodeFunctionId,
        feedback_slot: FeedbackSlotId,
    ) -> Self {
        Self {
            source,
            entry,
            feedback_slot,
        }
    }

    #[inline]
    pub const fn source(self) -> SourceId {
        self.source
    }

    #[inline]
    pub const fn entry(self) -> BytecodeFunctionId {
        self.entry
    }

    #[inline]
    pub const fn feedback_slot(self) -> FeedbackSlotId {
        self.feedback_slot
    }
}
