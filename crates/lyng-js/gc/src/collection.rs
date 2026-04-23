use crate::{
    PrimitiveBigIntRecord, PrimitiveCollectionStats, PrimitiveDomainStats, PrimitiveHeap,
    PrimitiveRoots, PrimitiveStringRecord, PrimitiveSymbolRecord, PrimitiveValueCellRecord,
    RuntimeCodeRecord, RuntimeEnvironmentRecord, RuntimeFunctionRecord, RuntimeObjectRecord,
    RuntimeRealmRecord, RuntimeShapeRecord, RuntimeSuspendedExecutionRecord, TraceHeapEdges,
    PRIMITIVE_SLOTS_PER_PAGE,
};
use std::mem::size_of;

pub(crate) const DEFAULT_COLLECTION_BUDGET_BYTES: usize = 1024;
const COLLECTION_GROWTH_FACTOR: usize = 2;

/// Accounting summary for one primitive heap domain.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrimitiveDomainAccounting {
    pub live_bytes: usize,
    pub reclaimable_bytes: usize,
    pub reserved_bytes: usize,
}

/// Cross-domain accounting summary exposed to reports and benches.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrimitiveHeapAccounting {
    pub strings: PrimitiveDomainAccounting,
    pub symbols: PrimitiveDomainAccounting,
    pub bigints: PrimitiveDomainAccounting,
    pub value_cells: PrimitiveDomainAccounting,
    pub objects: PrimitiveDomainAccounting,
    pub environments: PrimitiveDomainAccounting,
    pub codes: PrimitiveDomainAccounting,
    pub realms: PrimitiveDomainAccounting,
    pub shapes: PrimitiveDomainAccounting,
    pub live_bytes: usize,
    pub reclaimable_bytes: usize,
    pub reserved_bytes: usize,
}

/// Why a collection cycle ran.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveCollectionTrigger {
    Forced,
    StringAllocationSlowPath,
    SymbolAllocationSlowPath,
    BigIntAllocationSlowPath,
    ValueCellAllocationSlowPath,
    ObjectAllocationSlowPath,
    EnvironmentAllocationSlowPath,
    CodeAllocationSlowPath,
    RealmAllocationSlowPath,
    ShapeAllocationSlowPath,
}

/// Policy/report wrapper around a full primitive-domain collection cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrimitiveCollectionReport {
    pub trigger: PrimitiveCollectionTrigger,
    pub before: PrimitiveHeapAccounting,
    pub after: PrimitiveHeapAccounting,
    pub stats: PrimitiveCollectionStats,
    pub next_budget_bytes: usize,
}

impl PrimitiveHeap {
    pub fn accounting(&self) -> PrimitiveHeapAccounting {
        let strings = domain_accounting::<PrimitiveStringRecord>(self.string_stats());
        let symbols = domain_accounting::<PrimitiveSymbolRecord>(self.symbol_stats());
        let bigints = domain_accounting::<PrimitiveBigIntRecord>(self.bigint_stats());
        let value_cells = domain_accounting::<PrimitiveValueCellRecord>(self.value_cell_stats());
        let objects = merge_domain_accounting(
            merge_domain_accounting(
                domain_accounting::<RuntimeObjectRecord>(self.object_stats()),
                domain_accounting::<RuntimeFunctionRecord>(self.function_payload_stats()),
            ),
            domain_accounting::<RuntimeSuspendedExecutionRecord>(self.suspended_execution_stats()),
        );
        let environments = domain_accounting::<RuntimeEnvironmentRecord>(self.environment_stats());
        let codes = domain_accounting::<RuntimeCodeRecord>(self.code_stats());
        let realms = domain_accounting::<RuntimeRealmRecord>(self.realm_stats());
        let shapes = domain_accounting::<RuntimeShapeRecord>(self.shape_stats());

        PrimitiveHeapAccounting {
            strings,
            symbols,
            bigints,
            value_cells,
            objects,
            environments,
            codes,
            realms,
            shapes,
            live_bytes: strings.live_bytes
                + symbols.live_bytes
                + bigints.live_bytes
                + value_cells.live_bytes
                + objects.live_bytes
                + environments.live_bytes
                + codes.live_bytes
                + realms.live_bytes
                + shapes.live_bytes,
            reclaimable_bytes: strings.reclaimable_bytes
                + symbols.reclaimable_bytes
                + bigints.reclaimable_bytes
                + value_cells.reclaimable_bytes
                + objects.reclaimable_bytes
                + environments.reclaimable_bytes
                + codes.reclaimable_bytes
                + realms.reclaimable_bytes
                + shapes.reclaimable_bytes,
            reserved_bytes: strings.reserved_bytes
                + symbols.reserved_bytes
                + bigints.reserved_bytes
                + value_cells.reserved_bytes
                + objects.reserved_bytes
                + environments.reserved_bytes
                + codes.reserved_bytes
                + realms.reserved_bytes
                + shapes.reserved_bytes,
        }
    }

    #[inline]
    pub fn collection_budget_bytes(&self) -> usize {
        self.collection_budget_bytes
    }

    #[inline]
    pub fn set_collection_budget_bytes(&mut self, bytes: usize) {
        self.collection_budget_bytes = bytes;
    }

    pub fn force_collect(&mut self, roots: &PrimitiveRoots) -> PrimitiveCollectionReport {
        self.collect_with_trigger(roots, PrimitiveCollectionTrigger::Forced)
    }

    pub fn force_collect_tracing<T: TraceHeapEdges + ?Sized>(
        &mut self,
        roots: &PrimitiveRoots,
        additional_roots: &T,
    ) -> PrimitiveCollectionReport {
        let before = self.accounting();
        let stats = self.collect_tracing(roots, additional_roots);
        let after = self.accounting();
        let next_budget_bytes = next_collection_budget(after.live_bytes);
        self.collection_budget_bytes = next_budget_bytes;

        PrimitiveCollectionReport {
            trigger: PrimitiveCollectionTrigger::Forced,
            before,
            after,
            stats,
            next_budget_bytes,
        }
    }

    pub(crate) fn maybe_collect_before_growth(
        &mut self,
        roots: &PrimitiveRoots,
        trigger: PrimitiveCollectionTrigger,
    ) -> Option<PrimitiveCollectionReport> {
        let before = self.accounting();
        if before.reserved_bytes == 0 {
            return None;
        }

        if before.live_bytes < self.collection_budget_bytes && before.reclaimable_bytes == 0 {
            return None;
        }

        Some(self.finish_collection_report(roots, trigger, before))
    }

    pub(crate) fn collect_with_trigger(
        &mut self,
        roots: &PrimitiveRoots,
        trigger: PrimitiveCollectionTrigger,
    ) -> PrimitiveCollectionReport {
        let before = self.accounting();
        self.finish_collection_report(roots, trigger, before)
    }

    fn finish_collection_report(
        &mut self,
        roots: &PrimitiveRoots,
        trigger: PrimitiveCollectionTrigger,
        before: PrimitiveHeapAccounting,
    ) -> PrimitiveCollectionReport {
        let stats = self.collect(roots);
        let after = self.accounting();
        let next_budget_bytes = next_collection_budget(after.live_bytes);
        self.collection_budget_bytes = next_budget_bytes;

        PrimitiveCollectionReport {
            trigger,
            before,
            after,
            stats,
            next_budget_bytes,
        }
    }
}

fn domain_accounting<Record>(stats: PrimitiveDomainStats) -> PrimitiveDomainAccounting {
    let slot_bytes = size_of::<Record>();

    PrimitiveDomainAccounting {
        live_bytes: stats.occupied_slots * slot_bytes + stats.side_allocations.live_payload_bytes,
        reclaimable_bytes: stats.reusable_slots * slot_bytes
            + stats.side_allocations.reusable_reserved_bytes,
        reserved_bytes: stats.pages * PRIMITIVE_SLOTS_PER_PAGE * slot_bytes
            + stats.side_allocations.reserved_bytes,
    }
}

fn merge_domain_accounting(
    left: PrimitiveDomainAccounting,
    right: PrimitiveDomainAccounting,
) -> PrimitiveDomainAccounting {
    PrimitiveDomainAccounting {
        live_bytes: left.live_bytes + right.live_bytes,
        reclaimable_bytes: left.reclaimable_bytes + right.reclaimable_bytes,
        reserved_bytes: left.reserved_bytes + right.reserved_bytes,
    }
}

fn next_collection_budget(live_bytes: usize) -> usize {
    live_bytes
        .saturating_mul(COLLECTION_GROWTH_FACTOR)
        .max(DEFAULT_COLLECTION_BUDGET_BYTES)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AllocationLifetime, RuntimeObjectRecord, StringEncoding, WeakHeapRef};
    use lyng_js_types::Value;

    #[test]
    fn force_collect_reports_live_and_reclaimable_bytes() {
        let mut heap = PrimitiveHeap::new();
        let roots = PrimitiveRoots::new();
        let live = heap.alloc_string(
            StringEncoding::Latin1,
            4,
            b"live",
            None,
            AllocationLifetime::Default,
        );
        let _rooted = roots.root_string(live);
        let _dead = heap.alloc_string(
            StringEncoding::Latin1,
            4,
            b"dead",
            None,
            AllocationLifetime::Default,
        );

        let report = heap.force_collect(&roots);

        assert_eq!(report.trigger, PrimitiveCollectionTrigger::Forced);
        assert_eq!(report.stats.trace.strings_marked, 1);
        assert_eq!(report.stats.strings_reclaimed, 1);
        assert!(report.before.live_bytes > report.after.live_bytes);
        assert!(report.after.reclaimable_bytes > 0);
        assert!(report.next_budget_bytes >= report.after.live_bytes);
        assert_eq!(heap.accounting(), report.after);
        assert_eq!(heap.collection_budget_bytes(), report.next_budget_bytes);
    }

    #[test]
    fn weak_refs_clear_dead_targets_when_wrapper_stays_live() {
        let mut heap = PrimitiveHeap::new();
        let roots = PrimitiveRoots::new();
        let (wrapper, target) = {
            let mut mutator = heap.mutator();
            let wrapper = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::Default,
            );
            let target = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::Default,
            );

            assert!(mutator.init_weak_ref(wrapper, WeakHeapRef::Object(target)));
            (wrapper, target)
        };

        let _rooted_wrapper = roots.root_object(wrapper);
        let report = heap.force_collect(&roots);
        let view = heap.view();

        assert_eq!(report.stats.weak_refs_cleared, 1);
        assert_eq!(view.weak_ref_target(wrapper), Some(None));
        assert_eq!(view.object(target), None);
    }

    #[test]
    fn ephemeron_fixpoint_marks_values_reached_through_weak_map_chains() {
        let mut heap = PrimitiveHeap::new();
        let roots = PrimitiveRoots::new();
        let (map1, map2, key, intermediate, terminal) = {
            let mut mutator = heap.mutator();
            let map1 = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::Default,
            );
            let map2 = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::Default,
            );
            let key = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::Default,
            );
            let intermediate = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::Default,
            );
            let terminal = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::Default,
            );

            assert!(mutator.init_weak_map(map1));
            assert!(mutator.init_weak_map(map2));
            assert!(mutator.weak_map_set(
                map1,
                WeakHeapRef::Object(key),
                Value::from_object_ref(intermediate),
            ));
            assert!(mutator.weak_map_set(
                map2,
                WeakHeapRef::Object(intermediate),
                Value::from_object_ref(terminal),
            ));

            (map1, map2, key, intermediate, terminal)
        };

        let _rooted_map1 = roots.root_object(map1);
        let _rooted_map2 = roots.root_object(map2);
        let _rooted_key = roots.root_object(key);
        let report = heap.force_collect(&roots);
        let view = heap.view();

        assert_eq!(report.stats.ephemeron_fixes, 2);
        assert!(view.object(intermediate).is_some());
        assert!(view.object(terminal).is_some());
        assert_eq!(
            view.weak_map_get(map2, WeakHeapRef::Object(intermediate)),
            Some(Some(Value::from_object_ref(terminal)))
        );
    }

    #[test]
    fn finalization_registries_queue_dead_targets_and_preserve_holdings_until_drained() {
        let mut heap = PrimitiveHeap::new();
        let roots = PrimitiveRoots::new();
        let (registry, target, holdings, token) = {
            let mut mutator = heap.mutator();
            let registry = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::Default,
            );
            let target = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::Default,
            );
            let token = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::Default,
            );
            let holdings = mutator.alloc_string(
                StringEncoding::Latin1,
                4,
                b"held",
                None,
                AllocationLifetime::Default,
            );

            assert!(mutator.init_finalization_registry(registry));
            assert!(mutator.finalization_registry_register(
                registry,
                WeakHeapRef::Object(target),
                Value::from_string_ref(holdings),
                Some(WeakHeapRef::Object(token)),
            ));

            (registry, target, holdings, token)
        };

        let _rooted_registry = roots.root_object(registry);
        let report = heap.force_collect(&roots);
        let view = heap.view();

        assert_eq!(report.stats.finalization_cells_queued, 1);
        assert_eq!(view.object(target), None);
        assert!(view.string(holdings).is_some());

        let mut mutator = heap.mutator();
        assert_eq!(mutator.pending_finalization_registries(), vec![registry]);
        assert_eq!(
            mutator.take_finalization_cleanup_holdings(registry),
            vec![Value::from_string_ref(holdings)]
        );
        assert!(mutator.pending_finalization_registries().is_empty());

        let _ = token;
    }
}
