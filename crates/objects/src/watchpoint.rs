//! Per-shape watchpoint primitive. Spec 1 of the IC→JSC migration.
//!
//! The VM is single-threaded; nothing here is `Sync`. The drain-then-dispatch
//! ordering in `WatchpointSet::fire_all` is load-bearing for reentrancy — see
//! the design doc at docs/superpowers/specs/2026-05-25-shape-transitions-and-watchpoints-design.md §3.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatchpointState {
    Cleared,
    Watched,
    Invalidated,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Invalidated;

#[derive(Debug, PartialEq, Eq)]
pub enum ShapeInvalidationObserver {
    /// Test-only: records the fire event into a `Vec<u64>` so unit tests can
    /// assert "this transition fired exactly the watchpoints I registered."
    /// Carries no heap roots; production builds skip this branch via `cfg(test)`.
    Recording { token: u64 },
}

impl ShapeInvalidationObserver {
    /// Spec 1 dispatch surface for `Recording`. Spec 2's `AdaptiveProtoLoad`
    /// will add a different dispatch (taking `&mut Agent`); the enum split
    /// makes that addition exhaustive-match enforced.
    #[cfg(test)]
    pub(crate) fn fire_into(&self, sink: &mut Vec<u64>) {
        match self {
            Self::Recording { token } => sink.push(*token),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Watchpoint {
    ShapeInvalidation {
        observer: ShapeInvalidationObserver,
    },
}

pub struct WatchpointSet {
    state: WatchpointState,
    watchpoints: Vec<Watchpoint>,
}

impl Default for WatchpointSet {
    fn default() -> Self {
        Self::new()
    }
}

impl WatchpointSet {
    pub fn new() -> Self {
        Self {
            state: WatchpointState::Cleared,
            watchpoints: Vec::new(),
        }
    }

    pub fn state(&self) -> WatchpointState {
        self.state
    }

    pub fn is_invalidated(&self) -> bool {
        matches!(self.state, WatchpointState::Invalidated)
    }

    pub fn register(&mut self, wp: Watchpoint) -> Result<(), Invalidated> {
        if matches!(self.state, WatchpointState::Invalidated) {
            return Err(Invalidated);
        }
        self.state = WatchpointState::Watched;
        self.watchpoints.push(wp);
        Ok(())
    }

    /// Drain-then-dispatch. The fired set is `Invalidated` *before* any callback
    /// runs, so a callback that reregisters on this same set will get
    /// `Err(Invalidated)`. Callbacks should register on the post-transition
    /// shape's set (a different `ShapeId`, a different `WatchpointSet`).
    #[cfg(test)]
    pub(crate) fn fire_all_into(&mut self, sink: &mut Vec<u64>) {
        if matches!(self.state, WatchpointState::Invalidated) {
            return;
        }
        self.state = WatchpointState::Invalidated;
        let fired = std::mem::take(&mut self.watchpoints);
        for wp in fired {
            match wp {
                Watchpoint::ShapeInvalidation { observer } => observer.fire_into(sink),
            }
        }
    }

    /// Used by `Agent::fire_watchpoints_for_shape` to extract the fired list
    /// while it holds `&mut self.objects.watchpoint_sets`, so the dispatch
    /// loop can run after that borrow is dropped.
    pub fn drain_for_fire(&mut self) -> Option<Vec<Watchpoint>> {
        if matches!(self.state, WatchpointState::Invalidated) {
            return None;
        }
        self.state = WatchpointState::Invalidated;
        Some(std::mem::take(&mut self.watchpoints))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T6 — Recording.fire pushes token (will compile once Recording + a sink exist).
    #[test]
    fn recording_fire_pushes_token() {
        let mut sink: Vec<u64> = Vec::new();
        let observer = ShapeInvalidationObserver::Recording { token: 42 };
        observer.fire_into(&mut sink);
        assert_eq!(sink, vec![42]);
    }

    // T1 — Cleared → Watched → Invalidated
    #[test]
    fn state_machine_transitions() {
        let mut set = WatchpointSet::new();
        assert_eq!(set.state(), WatchpointState::Cleared);

        set.register(Watchpoint::ShapeInvalidation {
            observer: ShapeInvalidationObserver::Recording { token: 1 },
        }).unwrap();
        assert_eq!(set.state(), WatchpointState::Watched);

        let mut sink = Vec::new();
        set.fire_all_into(&mut sink);
        assert_eq!(set.state(), WatchpointState::Invalidated);
        assert_eq!(sink, vec![1]);
    }

    // T2 — register on Invalidated is rejected
    #[test]
    fn register_on_invalidated_returns_err() {
        let mut set = WatchpointSet::new();
        set.register(Watchpoint::ShapeInvalidation {
            observer: ShapeInvalidationObserver::Recording { token: 1 },
        }).unwrap();
        let mut sink = Vec::new();
        set.fire_all_into(&mut sink);

        let err = set.register(Watchpoint::ShapeInvalidation {
            observer: ShapeInvalidationObserver::Recording { token: 2 },
        }).unwrap_err();
        assert_eq!(err, Invalidated);
    }

    // T3 — fire_all on Cleared is a no-op (but state moves to Invalidated; terminal)
    #[test]
    fn fire_on_cleared_is_noop() {
        let mut set = WatchpointSet::new();
        let mut sink = Vec::new();
        set.fire_all_into(&mut sink);
        assert_eq!(set.state(), WatchpointState::Invalidated); // terminal even from Cleared
        assert!(sink.is_empty());
    }

    // T5 — dispatch order matches registration order
    #[test]
    fn fire_dispatches_in_registration_order() {
        let mut set = WatchpointSet::new();
        for token in [10, 20, 30] {
            set.register(Watchpoint::ShapeInvalidation {
                observer: ShapeInvalidationObserver::Recording { token },
            }).unwrap();
        }
        let mut sink = Vec::new();
        set.fire_all_into(&mut sink);
        assert_eq!(sink, vec![10, 20, 30]);
    }
}
