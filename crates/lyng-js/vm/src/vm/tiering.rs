use super::*;
use std::num::NonZeroU32;

const TIER_READY_HOTNESS_THRESHOLD: u32 = 8;
const FEEDBACK_EVENT_WEIGHT: u32 = 1;
const BACKEDGE_EVENT_WEIGHT: u32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TierStatus {
    InterpreterOnly,
    Collecting,
    ReadyForNative,
    NativeAttached,
    Invalidated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TieringSnapshot {
    eligible: bool,
    status: TierStatus,
    hotness: u32,
    feedback_events: u32,
    backedge_events: u32,
    invalidation_epoch: u32,
    native_generation: Option<NonZeroU32>,
}

impl TieringSnapshot {
    #[inline]
    pub const fn is_eligible(self) -> bool {
        self.eligible
    }

    #[inline]
    pub const fn status(self) -> TierStatus {
        self.status
    }

    #[inline]
    pub const fn hotness(self) -> u32 {
        self.hotness
    }

    #[inline]
    pub const fn feedback_events(self) -> u32 {
        self.feedback_events
    }

    #[inline]
    pub const fn backedge_events(self) -> u32 {
        self.backedge_events
    }

    #[inline]
    pub const fn invalidation_epoch(self) -> u32 {
        self.invalidation_epoch
    }

    #[inline]
    pub const fn native_generation(self) -> Option<NonZeroU32> {
        self.native_generation
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TieringState {
    eligible: bool,
    status: TierStatus,
    hotness: u32,
    feedback_events: u32,
    backedge_events: u32,
    invalidation_epoch: u32,
    native_generation: Option<NonZeroU32>,
}

impl Default for TieringState {
    #[inline]
    fn default() -> Self {
        Self {
            eligible: false,
            status: TierStatus::InterpreterOnly,
            hotness: 0,
            feedback_events: 0,
            backedge_events: 0,
            invalidation_epoch: 0,
            native_generation: None,
        }
    }
}

impl TieringState {
    #[inline]
    fn snapshot(self) -> TieringSnapshot {
        TieringSnapshot {
            eligible: self.eligible,
            status: self.status,
            hotness: self.hotness,
            feedback_events: self.feedback_events,
            backedge_events: self.backedge_events,
            invalidation_epoch: self.invalidation_epoch,
            native_generation: self.native_generation,
        }
    }

    #[inline]
    fn set_eligible(&mut self, eligible: bool) {
        self.eligible = eligible;
        if eligible {
            if self.status == TierStatus::InterpreterOnly {
                self.status = TierStatus::Collecting;
            }
        } else {
            self.status = TierStatus::InterpreterOnly;
            self.hotness = 0;
            self.feedback_events = 0;
            self.backedge_events = 0;
            self.native_generation = None;
        }
    }

    #[inline]
    fn invalidate(&mut self) {
        self.status = TierStatus::Invalidated;
        self.hotness = 0;
        self.feedback_events = 0;
        self.backedge_events = 0;
        self.invalidation_epoch = self.invalidation_epoch.saturating_add(1);
        self.native_generation = None;
    }

    #[inline]
    fn observe_feedback_event(&mut self) {
        if !self.eligible {
            return;
        }
        self.feedback_events = self.feedback_events.saturating_add(1);
        self.observe_hotness(FEEDBACK_EVENT_WEIGHT);
    }

    #[inline]
    fn observe_backedge_event(&mut self) {
        if !self.eligible {
            return;
        }
        self.backedge_events = self.backedge_events.saturating_add(1);
        self.observe_hotness(BACKEDGE_EVENT_WEIGHT);
    }

    #[inline]
    fn observe_hotness(&mut self, weight: u32) {
        if self.status == TierStatus::Invalidated {
            self.status = TierStatus::Collecting;
        }
        self.hotness = self.hotness.saturating_add(weight);
        if matches!(self.status, TierStatus::Collecting)
            && self.hotness >= TIER_READY_HOTNESS_THRESHOLD
        {
            self.status = TierStatus::ReadyForNative;
        }
    }
}

impl Vm {
    #[inline]
    pub(super) fn ensure_tiering_capacity(&mut self, code: CodeRef) {
        let index = code_index(code);
        if self.tiering.len() <= index {
            self.tiering.resize_with(index + 1, || None);
        }
    }

    #[inline]
    pub fn tiering_snapshot(&self, code: CodeRef) -> Option<TieringSnapshot> {
        self.tiering
            .get(code_index(code))
            .and_then(|state| state.map(TieringState::snapshot))
    }

    #[inline]
    pub fn set_tier_eligible(&mut self, code: CodeRef, eligible: bool) -> bool {
        let index = code_index(code);
        let Some(state) = self.tiering.get_mut(index).and_then(Option::as_mut) else {
            return false;
        };
        state.set_eligible(eligible);
        true
    }

    #[inline]
    pub fn invalidate_tier_state(&mut self, code: CodeRef) -> bool {
        let index = code_index(code);
        let Some(state) = self.tiering.get_mut(index).and_then(Option::as_mut) else {
            return false;
        };
        state.invalidate();
        true
    }

    #[inline]
    pub(super) fn observe_tier_feedback_event(&mut self, code: CodeRef) {
        if let Some(state) = self
            .tiering
            .get_mut(code_index(code))
            .and_then(Option::as_mut)
        {
            state.observe_feedback_event();
        }
    }

    #[inline]
    pub(super) fn observe_tier_backedge_event(&mut self, code: CodeRef) {
        if let Some(state) = self
            .tiering
            .get_mut(code_index(code))
            .and_then(Option::as_mut)
        {
            state.observe_backedge_event();
        }
    }
}
