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
