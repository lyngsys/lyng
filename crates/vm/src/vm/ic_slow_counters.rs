//! Per-cause IC slow-path entry counters for the Property IC families
//! (`GetNamedProperty`, `SetNamedProperty`, `AssignNamedProperty`,
//! `StrictAssignNamedProperty`).
//!
//! These counters are bumped exactly once per slow-path entry from the
//! DSL/asm cold handler bridges in `crate::dsl::handlers::cold`. They are
//! single-threaded (plain `u64`) because the `Vm` is single-owner. The
//! counters are inert until something reads them — only the bump itself
//! costs anything on the slow path.
//!
//! The "cause" classification reads the live `FeedbackSiteState` (the
//! semantic source of truth) plus the receiver shape/epoch to decide
//! which bucket a slow-path entry falls into. Hot paths never touch
//! this — the bump only fires after the asm cache hit path has already
//! decided to bail to Rust.

use crate::vm::Vm;
use crate::FrameRecord;
use lyng_env::Agent;
use lyng_types::{CodeRef, FeedbackSlotId};
use std::fmt::Write as _;

/// Reason a slow-path entry happened. Mirrors the structural decision
/// tree of the aarch64 `branch_named_*_mode!` chain in
/// `crate::dsl::backend::aarch64::feedback`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IcSlowPathCause {
    /// `feedback_slot` was `None` at dispatch time. Caller asked for a
    /// slow probe explicitly.
    NoSlot,
    /// Receiver was not an object value (asm `check_object_ref` failed).
    NonObject,
    /// Feedback site exists but is in the `Uninitialized` cache state —
    /// the asm read mode byte = 0 and fell through every cache hit branch.
    Uninitialized,
    /// Site is `Monomorphic` but the asm fell through because the mode
    /// byte didn't match (e.g. handler is `OwnInline` but the receiver
    /// actually has an out-of-line slot, or the site is monomorphic in
    /// a non-load cache that has no asm fast path on the assign side).
    ModeMismatch,
    /// Site is `Monomorphic` and the asm matched the mode byte, but the
    /// receiver shape didn't match the cached shape — the shape guard
    /// fired.
    ShapeMismatch,
    /// Site is `Monomorphic`, mode and shape matched, but the cached
    /// epoch didn't match the receiver's `last_invalidation_epoch` —
    /// a watchpoint fired (prototype mutation, shape teardown, etc.).
    EpochMismatch,
    /// Site is `Polymorphic` — the monomorphic asm fast path doesn't
    /// cover polymorphic shapes.
    Polymorphic,
    /// Site is `Megamorphic` — no entry-cache fast path can apply.
    Megamorphic,
    /// Catch-all: site exists but doesn't fit any of the above buckets
    /// (e.g. monomorphic but the handler is `NamedPropertyHandler::NONE`,
    /// or a non-`NamedProperty` site state somehow reached this opcode).
    Other,
}

impl IcSlowPathCause {
    const COUNT: usize = 10;
    const fn index(self) -> usize {
        match self {
            Self::NoSlot => 0,
            Self::NonObject => 1,
            Self::Uninitialized => 2,
            Self::ModeMismatch => 3,
            Self::ShapeMismatch => 4,
            Self::EpochMismatch => 5,
            Self::Polymorphic => 6,
            Self::Megamorphic => 7,
            Self::Other => 8,
            // last index reserved for total
        }
    }
    const TOTAL_INDEX: usize = 9;

    pub const ALL: [Self; 9] = [
        Self::NoSlot,
        Self::NonObject,
        Self::Uninitialized,
        Self::ModeMismatch,
        Self::ShapeMismatch,
        Self::EpochMismatch,
        Self::Polymorphic,
        Self::Megamorphic,
        Self::Other,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::NoSlot => "no_slot",
            Self::NonObject => "non_object",
            Self::Uninitialized => "uninitialized",
            Self::ModeMismatch => "mode_mismatch",
            Self::ShapeMismatch => "shape_mismatch",
            Self::EpochMismatch => "epoch_mismatch",
            Self::Polymorphic => "polymorphic",
            Self::Megamorphic => "megamorphic",
            Self::Other => "other",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
// Variants intentionally mirror the IC opcode-family names; the shared
// `NamedProperty` suffix is meaningful, not accidental noise.
#[allow(clippy::enum_variant_names)]
pub enum IcSlowPathKind {
    #[default]
    GetNamedProperty,
    SetNamedProperty,
    AssignNamedProperty,
    StrictAssignNamedProperty,
}

impl IcSlowPathKind {
    const COUNT: usize = 4;
    const fn index(self) -> usize {
        match self {
            Self::GetNamedProperty => 0,
            Self::SetNamedProperty => 1,
            Self::AssignNamedProperty => 2,
            Self::StrictAssignNamedProperty => 3,
        }
    }

    pub const ALL: [Self; Self::COUNT] = [
        Self::GetNamedProperty,
        Self::SetNamedProperty,
        Self::AssignNamedProperty,
        Self::StrictAssignNamedProperty,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::GetNamedProperty => "GetNamedProperty",
            Self::SetNamedProperty => "SetNamedProperty",
            Self::AssignNamedProperty => "AssignNamedProperty",
            Self::StrictAssignNamedProperty => "StrictAssignNamedProperty",
        }
    }
}

/// Per-kind, per-cause IC slow-path entry counters. Stored as a flat
/// 2-D table; the final column is the per-kind total (the sum of
/// every cause column) so the dispatch-count comparison is a single
/// load instead of a fold across the row.
#[derive(Clone, Debug, Default)]
pub struct IcSlowPathCounters {
    /// `[kind_index][cause_index_or_total]`.
    cells: [[u64; IcSlowPathCause::COUNT]; IcSlowPathKind::COUNT],
    /// Probe dispatches for `AssignNamedProperty` (one bump per opcode
    /// dispatch, including the ones that hit the rust probe and never
    /// fell to slow). Lets us read the fraction of assigns that fall
    /// off the fast path.
    pub(crate) assign_probe_dispatches: u64,
    /// Probe hits for `AssignNamedProperty` (rust probe returned
    /// `true`, no slow path). `assign_probe_dispatches - assign_probe_hits`
    /// is exactly the number of `AssignNamedProperty` slow-path entries
    /// (cross-check against the per-kind total below).
    pub(crate) assign_probe_hits: u64,
}

impl IcSlowPathCounters {
    #[inline]
    pub const fn new() -> Self {
        Self {
            cells: [[0; IcSlowPathCause::COUNT]; IcSlowPathKind::COUNT],
            assign_probe_dispatches: 0,
            assign_probe_hits: 0,
        }
    }

    #[inline]
    pub(crate) const fn bump(&mut self, kind: IcSlowPathKind, cause: IcSlowPathCause) {
        let row = &mut self.cells[kind.index()];
        row[cause.index()] = row[cause.index()].saturating_add(1);
        row[IcSlowPathCause::TOTAL_INDEX] = row[IcSlowPathCause::TOTAL_INDEX].saturating_add(1);
    }

    #[inline]
    pub(crate) const fn bump_assign_probe_dispatch(&mut self) {
        self.assign_probe_dispatches = self.assign_probe_dispatches.saturating_add(1);
    }

    #[inline]
    pub(crate) const fn bump_assign_probe_hit(&mut self) {
        self.assign_probe_hits = self.assign_probe_hits.saturating_add(1);
    }

    #[inline]
    #[must_use]
    pub const fn count(&self, kind: IcSlowPathKind, cause: IcSlowPathCause) -> u64 {
        self.cells[kind.index()][cause.index()]
    }

    #[inline]
    #[must_use]
    pub const fn total(&self, kind: IcSlowPathKind) -> u64 {
        self.cells[kind.index()][IcSlowPathCause::TOTAL_INDEX]
    }

    #[inline]
    #[must_use]
    pub const fn assign_probe_dispatches(&self) -> u64 {
        self.assign_probe_dispatches
    }

    #[inline]
    #[must_use]
    pub const fn assign_probe_hits(&self) -> u64 {
        self.assign_probe_hits
    }

    /// Format the counter table as a multi-line string suitable for
    /// dumping at the end of a benchmark run. One block per kind: a
    /// header line with the total, followed by one indented line per
    /// non-zero cause.
    #[must_use]
    // Counts cast to `f64` only to compute a display percentage; precision loss
    // would require > 2^52 slow-path entries, which never happens.
    #[allow(clippy::cast_precision_loss)]
    pub fn dump(&self) -> String {
        let mut out = String::new();
        out.push_str("# IC slow-path entry counters\n");
        let probe_dispatches = self.assign_probe_dispatches;
        let probe_hits = self.assign_probe_hits;
        writeln!(
            out,
            "AssignNamedProperty probe dispatches: {probe_dispatches}"
        )
        .unwrap();
        writeln!(out, "AssignNamedProperty probe hits:       {probe_hits}").unwrap();
        let assign_total = self.total(IcSlowPathKind::AssignNamedProperty);
        let probe_miss = self
            .assign_probe_dispatches
            .saturating_sub(self.assign_probe_hits);
        writeln!(
            out,
            "AssignNamedProperty probe misses (=> slow entries): {probe_miss}  (cross-check: kind total = {assign_total})"
        )
        .unwrap();
        out.push('\n');
        for kind in IcSlowPathKind::ALL {
            let total = self.total(kind);
            writeln!(out, "{:30} total slow entries: {total}", kind.name()).unwrap();
            for cause in IcSlowPathCause::ALL {
                let n = self.count(kind, cause);
                if n == 0 {
                    continue;
                }
                let pct = if total > 0 {
                    (n as f64 * 100.0) / (total as f64)
                } else {
                    0.0
                };
                writeln!(out, "    {:18} {n:>10}  ({pct:5.1}%)", cause.name()).unwrap();
            }
        }
        out
    }
}

impl Vm {
    /// Read-only accessor for the IC slow-path counters. The default
    /// `Vm::new()` constructs a zeroed table; callers can read it at
    /// any time to dump the per-kind / per-cause breakdown.
    #[inline]
    pub const fn ic_slow_path_counters(&self) -> &IcSlowPathCounters {
        &self.ic_slow_path_counters
    }

    /// Format the IC slow-path counter table as a multi-line string,
    /// suitable for dumping at the end of a benchmark run. See
    /// [`IcSlowPathCounters::dump`] for the format.
    #[inline]
    #[must_use]
    pub fn dump_ic_slow_path_counters(&self) -> String {
        self.ic_slow_path_counters.dump()
    }

    /// Classify and record a property-IC slow-path entry. Called from
    /// the cold handler bridges in `crate::dsl::handlers::cold` exactly
    /// once per slow entry, after `sync_from_asm` has aligned the Rust
    /// frame snapshot with the asm PC. `receiver_register` is the `b`
    /// operand of the opcode — the register that holds the receiver
    /// value the asm just tried (and failed) to load from.
    pub(crate) fn record_ic_slow_entry(
        &mut self,
        agent: &Agent,
        frame: &FrameRecord,
        slot: u32,
        receiver_register: u16,
        kind: IcSlowPathKind,
    ) {
        let slot = FeedbackSlotId::from_raw(slot);
        let receiver = self.read_register(frame.registers(), receiver_register);
        let cause = self.classify_named_slow_cause(agent, frame.code(), slot, receiver);
        self.ic_slow_path_counters.bump(kind, cause);
    }

    /// Record that the `AssignNamedProperty` rust probe was invoked
    /// (every dispatch of the opcode hits this — there's no asm fast
    /// path) and whether it succeeded. Lets the dump report
    /// "probe miss rate" alongside the per-kind slow-path total.
    pub(crate) const fn record_assign_named_property_probe(&mut self, hit: bool) {
        self.ic_slow_path_counters.bump_assign_probe_dispatch();
        if hit {
            self.ic_slow_path_counters.bump_assign_probe_hit();
        }
    }

    fn classify_named_slow_cause(
        &self,
        agent: &Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        receiver: lyng_types::Value,
    ) -> IcSlowPathCause {
        let Some(slot) = slot else {
            return IcSlowPathCause::NoSlot;
        };
        let Some(object) = receiver.as_object_ref() else {
            return IcSlowPathCause::NonObject;
        };
        self.classify_named_slow_cause_for_object(agent, code, slot, object)
    }
}
