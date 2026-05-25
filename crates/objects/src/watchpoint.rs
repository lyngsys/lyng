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
}
