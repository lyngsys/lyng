use lyng_types::Value;

/// Total never-realloc value/frame backing capacity, in 64-bit slots (4 MiB).
pub const ARENA_CAPACITY_SLOTS: usize = 512 * 1024;
/// Headroom reserved above the soft limit so the `RangeError` throw path — which
/// itself needs a frame + window — runs inside the reservation.
pub const ARENA_SLACK_SLOTS: usize = 4096;
/// Frame pushes are rejected at or above this; the slack remains for the throw.
pub const ARENA_SOFT_LIMIT_SLOTS: usize = ARENA_CAPACITY_SLOTS - ARENA_SLACK_SLOTS;

/// The single pre-reserved, never-reallocated value/frame stack.
///
/// Frames bump-allocate `[header][window]` runs from the base; the backing
/// `Box<[Value]>` is allocated once and never moves, so a pointer into it stays
/// valid across every push.
pub struct FrameArena {
    slots: Box<[Value]>,
    top: usize,
}

impl FrameArena {
    #[inline]
    pub fn new() -> Self {
        Self {
            slots: vec![Value::undefined(); ARENA_CAPACITY_SLOTS].into_boxed_slice(),
            top: 0,
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn base_ptr(&self) -> *const Value {
        self.slots.as_ptr()
    }

    #[inline]
    pub fn base_mut_ptr(&mut self) -> *mut Value {
        self.slots.as_mut_ptr()
    }

    #[inline]
    pub const fn top(&self) -> usize {
        self.top
    }

    #[inline]
    pub fn slots(&self) -> &[Value] {
        &self.slots
    }

    #[inline]
    pub fn slots_mut(&mut self) -> &mut [Value] {
        &mut self.slots
    }

    /// Reserve `slot_count` contiguous slots at the current top. Slots are NOT
    /// cleared on reuse — only the initial allocation is zeroed; callers must
    /// initialize every slot they read. Returns the base slot offset (the new
    /// frame's `cfr`), or `None` if it would cross the soft limit (caller throws
    /// `RangeError`).
    #[inline]
    pub fn bump(&mut self, slot_count: usize) -> Option<u32> {
        let base = self.top;
        let end = base.checked_add(slot_count)?;
        if end >= ARENA_SOFT_LIMIT_SLOTS {
            return None;
        }
        self.top = end;
        // base < ARENA_SOFT_LIMIT_SLOTS (520192) << u32::MAX, so this never truncates;
        // a debug assert documents the invariant instead of conflating overflow with a soft-limit hit.
        debug_assert!(u32::try_from(base).is_ok());
        Some(base as u32)
    }

    /// Release every slot at or above `slot_offset`. Only the cursor moves.
    #[inline]
    pub fn release_to(&mut self, slot_offset: u32) {
        debug_assert!(
            (slot_offset as usize) <= self.top,
            "release_to must not advance the cursor"
        );
        self.top = slot_offset as usize;
    }

    /// Set the cursor directly (used by the window-reservation bridge). Asserts the
    /// new top stays within capacity.
    #[inline]
    pub fn set_top(&mut self, top: usize) {
        debug_assert!(top <= ARENA_CAPACITY_SLOTS);
        self.top = top;
    }

    /// Eager backing cannot grow; a future lazy-commit backing would commit pages instead.
    #[allow(dead_code)]
    #[inline]
    pub const fn try_grow(&mut self, _needed_top: usize) -> bool {
        false
    }
}

impl Default for FrameArena {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bump_and_release_move_the_cursor_without_realloc() {
        let mut arena = FrameArena::new();
        let base_ptr = arena.base_ptr();
        assert_eq!(arena.top(), 0);

        let cfr = arena.bump(7 + 3).expect("space for one frame");
        assert_eq!(cfr, 0);
        assert_eq!(arena.top(), 10);

        arena.slots_mut()[7] = Value::from_smi(99);
        assert_eq!(arena.slots()[7], Value::from_smi(99));

        arena.release_to(cfr);
        assert_eq!(arena.top(), 0);
        assert_eq!(arena.base_ptr(), base_ptr);
    }

    #[test]
    fn bump_past_soft_limit_returns_none() {
        let mut arena = FrameArena::new();
        assert!(arena.bump(ARENA_SOFT_LIMIT_SLOTS - 1).is_some());
        assert!(arena.bump(1).is_none()); // end == SOFT_LIMIT is rejected (>=)
        assert!(arena.bump(2).is_none());
    }
}
