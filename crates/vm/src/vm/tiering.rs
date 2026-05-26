use super::{code_index, CodeRef, Vm};
use std::num::NonZeroU32;

const TIER_READY_HOTNESS_THRESHOLD: u32 = 8;
const FEEDBACK_EVENT_WEIGHT: u32 = 1;
// DSL-0c C6: BACKEDGE_EVENT_WEIGHT deleted with α path's backedge accounting.

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
    warmup_counter: u16,
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

    #[inline]
    pub const fn warmup_counter(self) -> u16 {
        self.warmup_counter
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
    warmup_counter: u16,
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
            warmup_counter: 0,
        }
    }
}

impl TieringState {
    #[inline]
    const fn snapshot(self) -> TieringSnapshot {
        TieringSnapshot {
            eligible: self.eligible,
            status: self.status,
            hotness: self.hotness,
            feedback_events: self.feedback_events,
            backedge_events: self.backedge_events,
            invalidation_epoch: self.invalidation_epoch,
            native_generation: self.native_generation,
            warmup_counter: self.warmup_counter,
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
    const fn invalidate(&mut self) {
        self.status = TierStatus::Invalidated;
        self.hotness = 0;
        self.feedback_events = 0;
        self.backedge_events = 0;
        self.invalidation_epoch = self.invalidation_epoch.saturating_add(1);
        self.native_generation = None;
    }

    #[inline]
    fn observe_feedback_events(&mut self, count: u32) {
        if !self.eligible {
            return;
        }
        self.feedback_events = self.feedback_events.saturating_add(count);
        self.observe_hotness(FEEDBACK_EVENT_WEIGHT.saturating_mul(count));
    }

    // DSL-0c C6: observe_backedge_event deleted with α path's
    // backedge accounting.

    #[inline]
    pub(super) const fn warmup_counter(&self) -> u16 {
        self.warmup_counter
    }

    #[inline]
    pub(super) fn bump_warmup(&mut self) -> u16 {
        self.warmup_counter = self.warmup_counter.saturating_add(1);
        self.warmup_counter
    }

    /// Saturating increment by `n`; returns the new value.
    #[inline]
    pub(super) fn bump_warmup_by(&mut self, n: u16) -> u16 {
        self.warmup_counter = self.warmup_counter.saturating_add(n);
        self.warmup_counter
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

/// Caller-ownable container for per-`CodeRef` tier-up state.
///
/// The default `Vm::new()` holds a *disabled* `Tiering` (`Tiering::disabled()`)
/// — `ensure_slot` is a no-op and `observe_feedback_events` short-circuits
/// before any `Vec` work. Callers that want to collect tier state construct
/// an active `Tiering::new()`, swap it in via
/// `EvaluateScript::with_tiering` / `EvaluateInstalled::with_tiering`, then
/// read snapshots off the caller-owned struct afterwards.
///
/// The current shape is scaffolding for an eventual JSC-style tier ladder
/// (LLInt → Baseline → DFG → FTL). DSL-0c (§2, §6, §10) deliberately defers
/// the JIT and tier accounting; this struct exists so the bookkeeping can
/// be opt-in at near-zero cost until that work lands.
pub struct Tiering {
    states: Vec<Option<TieringState>>,
    enabled: bool,
}

impl Default for Tiering {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Tiering {
    /// Construct an *active* tiering store. Slots are allocated on
    /// `ensure_slot`, and IC feedback observations accumulate hotness.
    #[inline]
    pub fn new() -> Self {
        Self {
            states: Vec::new(),
            enabled: true,
        }
    }

    /// Construct a *disabled* tiering store. `ensure_slot` is a no-op,
    /// `observe_feedback_event(s)` short-circuit immediately, and
    /// `snapshot` always returns `None`. Used as the always-present
    /// internal store on `Vm::new()` so the default VM pays nothing for
    /// the tiering scaffold until a caller opts in via `with_tiering`.
    #[inline]
    pub(super) fn disabled() -> Self {
        Self {
            states: Vec::new(),
            enabled: false,
        }
    }

    /// Register `code` for tier tracking, creating a default-state slot if
    /// one isn't present yet. Idempotent: existing slot state is preserved.
    /// No-op on a disabled tiering store.
    ///
    /// Call this on a caller-owned `Tiering` for every `CodeRef` you want
    /// to track *before* swapping it into the VM via `with_tiering`. The
    /// VM's internal install path also calls `ensure_slot` on whatever
    /// `Tiering` is currently swapped in, so codes installed during an
    /// `evaluate_script` run are auto-registered on the caller-owned
    /// `Tiering` for that run.
    #[inline]
    pub fn ensure_slot(&mut self, code: CodeRef) {
        if !self.enabled {
            return;
        }
        let index = code_index(code);
        if self.states.len() <= index {
            self.states.resize_with(index + 1, || None);
        }
        if self.states[index].is_none() {
            self.states[index] = Some(TieringState::default());
        }
    }

    #[inline]
    pub fn snapshot(&self, code: CodeRef) -> Option<TieringSnapshot> {
        self.states
            .get(code_index(code))
            .and_then(|state| state.map(TieringState::snapshot))
    }

    #[inline]
    pub fn set_eligible(&mut self, code: CodeRef, eligible: bool) -> bool {
        let index = code_index(code);
        let Some(state) = self.states.get_mut(index).and_then(Option::as_mut) else {
            return false;
        };
        state.set_eligible(eligible);
        true
    }

    #[inline]
    pub fn invalidate(&mut self, code: CodeRef) -> bool {
        let index = code_index(code);
        let Some(state) = self.states.get_mut(index).and_then(Option::as_mut) else {
            return false;
        };
        state.invalidate();
        true
    }

    #[inline]
    pub(super) fn observe_feedback_event(&mut self, code: CodeRef) {
        self.observe_feedback_events(code, 1);
    }

    #[inline]
    pub(super) fn observe_feedback_events(&mut self, code: CodeRef, count: u32) {
        if !self.enabled || self.states.is_empty() {
            return;
        }
        if let Some(state) = self
            .states
            .get_mut(code_index(code))
            .and_then(Option::as_mut)
        {
            state.observe_feedback_events(count);
        }
    }

    // DSL-0c C6: observe_backedge_event deleted with α path's
    // backedge accounting. The interpreter has no tier-up accounting
    // post-DSL-0c (design §6 + §10).

    /// Return the warmup counter for `code`, or 0 if no slot exists yet.
    /// Works on both enabled and disabled `Tiering` stores.
    #[inline]
    pub(super) fn warmup_counter(&self, code: CodeRef) -> u16 {
        self.states
            .get(code_index(code))
            .and_then(Option::as_ref)
            .map_or(0, TieringState::warmup_counter)
    }

    /// Saturating bump of the warmup counter for `code`; returns the new
    /// value. Lazily inserts a default slot if none exists. Works on both
    /// enabled and disabled `Tiering` stores so that allocation-threshold
    /// logic always has a counter to track.
    #[inline]
    pub(super) fn bump_warmup(&mut self, code: CodeRef) -> u16 {
        let index = code_index(code);
        if self.states.len() <= index {
            self.states.resize_with(index + 1, || None);
        }
        let state = self.states[index].get_or_insert_with(TieringState::default);
        state.bump_warmup()
    }

    /// Saturating increment of the warmup counter by `n` for `code`; returns
    /// the new value. Lazily inserts a default slot if none exists.
    #[inline]
    pub(super) fn bump_warmup_by(&mut self, code: CodeRef, n: u16) -> u16 {
        let index = code_index(code);
        if self.states.len() <= index {
            self.states.resize_with(index + 1, || None);
        }
        let state = self.states[index].get_or_insert_with(TieringState::default);
        state.bump_warmup_by(n)
    }
}

impl Vm {
    /// Read a tiering snapshot from the VM's *internal* `Tiering`.
    ///
    /// In the default configuration this is the always-empty `Tiering` that
    /// `Vm::new()` constructs, so the snapshot is `None`. To collect tier
    /// state, swap in a caller-owned `Tiering` via
    /// `EvaluateScript::with_tiering` / `EvaluateInstalled::with_tiering`
    /// and read the snapshot off the caller-owned struct after `.run()`.
    #[inline]
    pub fn tiering_snapshot(&self, code: CodeRef) -> Option<TieringSnapshot> {
        self.tiering.snapshot(code)
    }
}
