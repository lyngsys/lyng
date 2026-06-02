use crate::frame::GeneratorResumeKind;
use lyng_types::{ObjectRef, Value};

/// Rare per-activation state that the asm path never touches and that does not
/// belong in the asm-addressable header.
///
/// Holds the exception-handler cursor, tail-call linkage, generator resume state,
/// and the parameter-initializer end offset. Reset to default on every frame push;
/// keyed by frame depth.
#[derive(Clone, Copy, Debug)]
pub struct FrameColdState {
    pub handler_cursor: u16,
    pub tail_caller: Option<ObjectRef>,
    pub tail_caller_strict: bool,
    pub resume_kind: GeneratorResumeKind,
    pub resume_value: Value,
    pub resume_active: bool,
    pub parameter_initializer_end_offset: u32,
}

impl Default for FrameColdState {
    #[inline]
    fn default() -> Self {
        Self {
            handler_cursor: 0,
            tail_caller: None,
            tail_caller_strict: false,
            resume_kind: GeneratorResumeKind::Next,
            resume_value: Value::undefined(),
            resume_active: false,
            parameter_initializer_end_offset: 0,
        }
    }
}

/// Depth-indexed dense store of [`FrameColdState`]. Grows lazily to the deepest
/// frame seen; `reset_at` is called on every push to clear stale state from a prior
/// frame that occupied the same depth.
pub struct FrameColdTable {
    slots: Vec<FrameColdState>,
}

impl FrameColdTable {
    #[inline]
    pub const fn new() -> Self {
        Self { slots: Vec::new() }
    }

    #[inline]
    fn ensure(&mut self, depth: usize) {
        if self.slots.len() <= depth {
            self.slots.resize(depth + 1, FrameColdState::default());
        }
    }

    #[inline]
    pub fn reset_at(&mut self, depth: usize) {
        self.ensure(depth);
        self.slots[depth] = FrameColdState::default();
    }

    /// # Panics
    /// Panics if `reset_at(depth)` has not been called for this depth.
    #[allow(
        dead_code,
        reason = "seeded via reset_at/get_mut; direct read used only in reconstruction"
    )]
    #[inline]
    pub fn get(&self, depth: usize) -> &FrameColdState {
        &self.slots[depth]
    }

    /// # Panics
    /// Panics if `reset_at(depth)` has not been called for this depth.
    #[inline]
    pub fn get_mut(&mut self, depth: usize) -> &mut FrameColdState {
        &mut self.slots[depth]
    }

    /// The live cold slots for the `depth` currently-active frames (`0..depth`).
    /// Returns the seeded prefix; never the lazily-grown tail beyond `depth`.
    #[inline]
    pub fn live_slots(&self, depth: usize) -> &[FrameColdState] {
        let end = depth.min(self.slots.len());
        &self.slots[..end]
    }
}

impl Default for FrameColdTable {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_types::Value;

    #[test]
    fn cold_state_defaults_and_round_trips_by_depth() {
        let mut cold = FrameColdTable::new();
        cold.reset_at(0);
        assert_eq!(cold.get(0).handler_cursor, 0);
        assert!(!cold.get(0).resume_active);

        let slot = cold.get_mut(0);
        slot.handler_cursor = 5;
        slot.resume_active = true;
        slot.resume_value = Value::from_smi(8);
        assert_eq!(cold.get(0).handler_cursor, 5);
        assert!(cold.get(0).resume_active);

        // Reusing depth 0 for a new frame clears the stale state.
        cold.reset_at(0);
        assert_eq!(cold.get(0).handler_cursor, 0);
        assert!(!cold.get(0).resume_active);
    }
}
