use crate::{AllocationLifetime, PrimitiveMinorCollectionStats};

pub const DEFAULT_NURSERY_CAPACITY_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NurseryDomain {
    String,
    StringPayload,
    Symbol,
    BigInt,
    BigIntPayload,
    ValueCell,
    Object,
    FunctionPayload,
    ObjectSlots,
    SuspendedExecution,
    SuspendedRegisters,
    Environment,
    EnvironmentSlots,
    Code,
    CodeSlots,
    Realm,
    Shape,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrimitiveAllocationProfile {
    pub nursery_allocations: usize,
    pub old_allocations: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NurseryStats {
    pub capacity_bytes: usize,
    pub used_bytes: usize,
    pub allocated_bytes: usize,
    pub nursery_allocations: usize,
    pub old_allocations: usize,
    pub minor_collections: usize,
    pub total_minor_pause_ns: u128,
    pub last_minor_pause_ns: u128,
    pub last_survivors: usize,
    pub last_tenured: usize,
    pub last_reclaimed: usize,
    pub last_cards_dirtied: usize,
    pub last_cards_scanned: usize,
}

#[derive(Clone, Debug)]
pub struct Nursery {
    capacity_bytes: usize,
    used_bytes: usize,
    tenuring_threshold: u8,
    stats: NurseryStats,
}

impl Default for Nursery {
    fn default() -> Self {
        let capacity_bytes = DEFAULT_NURSERY_CAPACITY_BYTES;
        Self {
            capacity_bytes,
            used_bytes: 0,
            tenuring_threshold: 1,
            stats: NurseryStats {
                capacity_bytes,
                ..NurseryStats::default()
            },
        }
    }
}

impl PrimitiveAllocationProfile {
    #[inline]
    pub const fn total_allocations(self) -> usize {
        self.nursery_allocations + self.old_allocations
    }

    #[inline]
    pub const fn nursery_allocation_ratio(self) -> usize {
        let total = self.total_allocations();
        if total == 0 {
            0
        } else {
            self.nursery_allocations * 100 / total
        }
    }
}

impl Nursery {
    #[inline]
    pub(crate) const fn tenuring_threshold(&self) -> u8 {
        self.tenuring_threshold
    }

    #[inline]
    pub(crate) fn set_capacity_bytes(&mut self, bytes: usize) {
        self.capacity_bytes = bytes.max(1);
        self.used_bytes = self.used_bytes.min(self.capacity_bytes);
        self.stats.capacity_bytes = self.capacity_bytes;
        self.stats.used_bytes = self.used_bytes;
    }

    #[inline]
    pub(crate) fn set_tenuring_threshold(&mut self, threshold: u8) {
        self.tenuring_threshold = threshold.max(1);
    }

    #[inline]
    pub(crate) const fn stats(&self) -> NurseryStats {
        self.stats
    }

    #[inline]
    pub(crate) const fn profile(&self) -> PrimitiveAllocationProfile {
        PrimitiveAllocationProfile {
            nursery_allocations: self.stats.nursery_allocations,
            old_allocations: self.stats.old_allocations,
        }
    }

    #[inline]
    pub(crate) const fn can_fit(&self, bytes: usize) -> bool {
        bytes <= self.capacity_bytes.saturating_sub(self.used_bytes)
    }

    #[inline]
    pub(crate) const fn is_nursery_eligible(
        domain: NurseryDomain,
        lifetime: AllocationLifetime,
    ) -> bool {
        if matches!(lifetime, AllocationLifetime::LongLived) {
            return false;
        }
        !matches!(
            domain,
            NurseryDomain::Code
                | NurseryDomain::CodeSlots
                | NurseryDomain::Realm
                | NurseryDomain::Shape
        )
    }

    #[inline]
    pub(crate) const fn reserve(&mut self, bytes: usize) -> bool {
        if !self.can_fit(bytes) {
            return false;
        }
        self.used_bytes += bytes;
        self.stats.used_bytes = self.used_bytes;
        self.stats.allocated_bytes += bytes;
        self.stats.nursery_allocations += 1;
        true
    }

    #[inline]
    pub(crate) const fn note_old_allocation(&mut self) {
        self.stats.old_allocations += 1;
    }

    #[inline]
    pub(crate) fn finish_minor_collection(
        &mut self,
        young_live_bytes: usize,
        minor: PrimitiveMinorCollectionStats,
    ) {
        self.used_bytes = young_live_bytes.min(self.capacity_bytes);
        self.stats.used_bytes = self.used_bytes;
        self.stats.minor_collections += 1;
        self.stats.total_minor_pause_ns += minor.pause_ns;
        self.stats.last_minor_pause_ns = minor.pause_ns;
        self.stats.last_survivors = minor.survivors;
        self.stats.last_tenured = minor.tenured;
        self.stats.last_reclaimed = minor.reclaimed;
        self.stats.last_cards_dirtied = minor.cards_dirtied;
        self.stats.last_cards_scanned = minor.cards_scanned;
    }
}
