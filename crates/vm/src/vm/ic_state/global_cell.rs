//! Per-site global cell inline cache state.
//!
//! Caches, per `(code, feedback_slot)`, WHERE a `LoadGlobal` resolves so
//! subsequent reads skip name resolution entirely. The cached target is either
//! the backing primitive-value cell of a cell-backed global-object entry, or a
//! global lexical environment slot.
//!
//! Correctness comes from the coarse `global_structure_generation` carried on
//! the global environment, not from immutability: every structural change to
//! the globals (binding deleted, data <-> accessor redefine, a new lexical
//! shadowing a var) bumps the generation, so a cached site whose recorded
//! generation no longer matches re-resolves rather than dereferencing a stale
//! (possibly freed) cell. Plain value writes do NOT bump the generation — the
//! cell/slot is read live on every hit.

use lyng_gc::PrimitiveValueCellRef;
use lyng_types::EnvironmentRef;

/// Where a cached global resolves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlobalCellTarget {
    /// A cell-backed global-object data entry. Read live via the cell's
    /// `stored_value()`.
    Cell(PrimitiveValueCellRef),
    /// A global lexical binding slot `(environment, slot)`. Read live via the
    /// environment slot (with the usual TDZ check).
    EnvSlot(EnvironmentRef, u32),
}

/// Cached resolution for one `LoadGlobal` site.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlobalCellIcState {
    /// The cached resolution target.
    pub target: GlobalCellTarget,
    /// The `global_structure_generation` captured at install time. A hit is
    /// only valid while this equals the live generation; on mismatch the site
    /// re-resolves.
    pub structure_gen: u32,
}

impl GlobalCellIcState {
    #[inline]
    pub const fn new(target: GlobalCellTarget, structure_gen: u32) -> Self {
        Self {
            target,
            structure_gen,
        }
    }
}
