use crate::rooting::PrimitiveMajorMarkMetrics;
use crate::{
    card_table::CardDomain, nursery::PrimitiveAllocationProfile, PrimitiveBigIntRecord,
    PrimitiveCollectionStats, PrimitiveDomainStats, PrimitiveHeap, PrimitiveRoots,
    PrimitiveStringRecord, PrimitiveSymbolRecord, PrimitiveValueCellRecord, RuntimeCodeRecord,
    RuntimeEnvironmentRecord, RuntimeFunctionRecord, RuntimeObjectRecord, RuntimeRealmRecord,
    RuntimeShapeRecord, RuntimeSuspendedExecutionRecord, TraceHeapEdges, PRIMITIVE_SLOTS_PER_PAGE,
};
use std::mem::size_of;

pub const DEFAULT_COLLECTION_BUDGET_BYTES: usize = 1024;
pub const DEFAULT_MAJOR_MARK_SLICE_BUDGET: usize = 64;
const COLLECTION_GROWTH_FACTOR: usize = 2;

/// Accounting summary for one primitive heap domain.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrimitiveDomainAccounting {
    pub live_bytes: usize,
    pub young_live_bytes: usize,
    pub old_live_bytes: usize,
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
    pub young_live_bytes: usize,
    pub old_live_bytes: usize,
    pub reclaimable_bytes: usize,
    pub reserved_bytes: usize,
    pub allocation_profile: PrimitiveAllocationProfile,
    pub nursery_capacity_bytes: usize,
    pub nursery_used_bytes: usize,
    pub minor_collections: usize,
    pub last_minor_pause_ns: u128,
    pub last_minor_survivors: usize,
    pub last_minor_tenured: usize,
    pub last_minor_reclaimed: usize,
    pub last_minor_cards_dirtied: usize,
    pub last_minor_cards_scanned: usize,
    pub last_major_mark_slices: usize,
    pub last_major_mark_slice_budget: usize,
    pub last_major_mark_work_items: usize,
    pub last_major_max_mark_slice_work_items: usize,
    pub last_major_total_mark_pause_ns: u128,
    pub last_major_max_mark_pause_ns: u128,
    pub last_major_mark_finish_work_items: usize,
    pub last_major_mark_finish_pause_ns: u128,
    pub last_major_gray_work_items_after_finish: usize,
    pub last_major_background_sweep_started: bool,
    pub last_major_background_sweep_completed: bool,
    pub last_major_background_sweep_worker_thread_id: u64,
    pub last_major_background_sweep_candidates: usize,
    pub last_major_background_sweep_reclaimed: usize,
    pub last_major_background_sweep_duration_ns: u128,
    pub last_major_background_sweep_apply_pause_ns: u128,
}

/// Which collector policy produced a collection report.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PrimitiveCollectionKind {
    #[default]
    Major,
    Minor,
}

/// Why a collection cycle ran.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveCollectionTrigger {
    Forced,
    NurseryAllocationLimit,
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

/// Nursery-specific accounting for one minor collection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrimitiveMinorCollectionStats {
    pub survivors: usize,
    pub tenured: usize,
    pub reclaimed: usize,
    pub cards_dirtied: usize,
    pub cards_scanned: usize,
    pub pause_ns: u128,
}

/// Policy/report wrapper around a full primitive-domain collection cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrimitiveCollectionReport {
    pub kind: PrimitiveCollectionKind,
    pub trigger: PrimitiveCollectionTrigger,
    pub before: PrimitiveHeapAccounting,
    pub after: PrimitiveHeapAccounting,
    pub stats: PrimitiveCollectionStats,
    pub minor: PrimitiveMinorCollectionStats,
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

        let nursery_stats = self.nursery_stats();
        let background_sweep = self.last_major_background_sweep;

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
            young_live_bytes: strings.young_live_bytes
                + symbols.young_live_bytes
                + bigints.young_live_bytes
                + value_cells.young_live_bytes
                + objects.young_live_bytes
                + environments.young_live_bytes
                + codes.young_live_bytes
                + realms.young_live_bytes
                + shapes.young_live_bytes,
            old_live_bytes: strings.old_live_bytes
                + symbols.old_live_bytes
                + bigints.old_live_bytes
                + value_cells.old_live_bytes
                + objects.old_live_bytes
                + environments.old_live_bytes
                + codes.old_live_bytes
                + realms.old_live_bytes
                + shapes.old_live_bytes,
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
            allocation_profile: self.allocation_profile(),
            nursery_capacity_bytes: nursery_stats.capacity_bytes,
            nursery_used_bytes: nursery_stats.used_bytes,
            minor_collections: nursery_stats.minor_collections,
            last_minor_pause_ns: nursery_stats.last_minor_pause_ns,
            last_minor_survivors: nursery_stats.last_survivors,
            last_minor_tenured: nursery_stats.last_tenured,
            last_minor_reclaimed: nursery_stats.last_reclaimed,
            last_minor_cards_dirtied: nursery_stats.last_cards_dirtied,
            last_minor_cards_scanned: nursery_stats.last_cards_scanned,
            last_major_mark_slices: self.last_major_mark_slices,
            last_major_mark_slice_budget: self.major_mark_slice_budget,
            last_major_mark_work_items: self.last_major_mark_work_items,
            last_major_max_mark_slice_work_items: self.last_major_max_mark_slice_work_items,
            last_major_total_mark_pause_ns: self.last_major_total_mark_pause_ns,
            last_major_max_mark_pause_ns: self.last_major_max_mark_pause_ns,
            last_major_mark_finish_work_items: self.last_major_mark_finish_work_items,
            last_major_mark_finish_pause_ns: self.last_major_mark_finish_pause_ns,
            last_major_gray_work_items_after_finish: self.last_major_gray_work_items_after_finish,
            last_major_background_sweep_started: background_sweep.started,
            last_major_background_sweep_completed: background_sweep.completed,
            last_major_background_sweep_worker_thread_id: background_sweep.worker_thread_id,
            last_major_background_sweep_candidates: background_sweep.candidates,
            last_major_background_sweep_reclaimed: background_sweep.reclaimed,
            last_major_background_sweep_duration_ns: background_sweep.duration_ns,
            last_major_background_sweep_apply_pause_ns: background_sweep.apply_pause_ns,
        }
    }

    #[inline]
    pub const fn collection_budget_bytes(&self) -> usize {
        self.collection_budget_bytes
    }

    #[inline]
    pub const fn set_collection_budget_bytes(&mut self, bytes: usize) {
        self.collection_budget_bytes = bytes;
    }

    #[inline]
    pub const fn major_mark_slice_budget(&self) -> usize {
        self.major_mark_slice_budget
    }

    #[inline]
    pub const fn set_major_mark_slice_budget(&mut self, budget: usize) {
        self.major_mark_slice_budget = budget;
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
        let mut metrics = PrimitiveMajorMarkMetrics::new(self.major_mark_slice_budget().max(1));
        let stats = self.collect_tracing_with_mark_metrics(roots, additional_roots, &mut metrics);
        self.record_last_major_mark_stats(&stats);
        let after = self.accounting();
        let next_budget_bytes = next_collection_budget(after.live_bytes);
        self.collection_budget_bytes = next_budget_bytes;

        PrimitiveCollectionReport {
            kind: PrimitiveCollectionKind::Major,
            trigger: PrimitiveCollectionTrigger::Forced,
            before,
            after,
            stats,
            minor: PrimitiveMinorCollectionStats::default(),
            next_budget_bytes,
        }
    }

    pub fn force_minor_collect(&mut self, roots: &PrimitiveRoots) -> PrimitiveCollectionReport {
        self.minor_collect_with_trigger(roots, PrimitiveCollectionTrigger::Forced)
    }

    pub(crate) fn minor_collect_with_trigger(
        &mut self,
        roots: &PrimitiveRoots,
        trigger: PrimitiveCollectionTrigger,
    ) -> PrimitiveCollectionReport {
        let before = self.accounting();
        let start = std::time::Instant::now();
        self.clear_all_marks();

        let (cards_dirtied, dirty_cards) = {
            let cards_dirtied = self.card_table_dirtied_since_minor();
            (cards_dirtied, self.take_dirty_cards())
        };

        {
            let mut tracer = crate::rooting::PrimitiveMinorTracer::new(self);
            roots.trace_minor_roots(&mut tracer);
            for card in &dirty_cards {
                match card.domain() {
                    CardDomain::String => tracer.trace_string_card(card.index()),
                    CardDomain::Symbol => tracer.trace_symbol_card(card.index()),
                    CardDomain::ObjectSlots => tracer.trace_object_slots_card(card.index()),
                    CardDomain::EnvironmentSlots => {
                        tracer.trace_environment_slots_card(card.index());
                    }
                    CardDomain::CodeSlots => tracer.trace_code_slots_card(card.index()),
                    CardDomain::Object => tracer.trace_object_card(card.index()),
                    CardDomain::Environment => tracer.trace_environment_card(card.index()),
                    CardDomain::FunctionPayload => tracer.trace_function_payload_card(card.index()),
                    CardDomain::ValueCell => tracer.trace_value_cell_card(card.index()),
                    CardDomain::SuspendedExecution => {
                        tracer.trace_suspended_execution_card(card.index());
                    }
                    CardDomain::SuspendedRegisters => {
                        tracer.trace_suspended_registers_card(card.index());
                    }
                    CardDomain::Realm => tracer.trace_realm_card(card.index()),
                    CardDomain::Shape => tracer.trace_shape_card(card.index()),
                }
            }
        }

        let minor = self.sweep_young_generation(cards_dirtied, dirty_cards.len(), start.elapsed());
        let after = self.accounting();
        PrimitiveCollectionReport {
            kind: PrimitiveCollectionKind::Minor,
            trigger,
            before,
            after,
            stats: PrimitiveCollectionStats::default(),
            minor,
            next_budget_bytes: self.collection_budget_bytes,
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

        Some(self.finish_collection_report(roots, trigger, &before))
    }

    pub(crate) fn collect_with_trigger(
        &mut self,
        roots: &PrimitiveRoots,
        trigger: PrimitiveCollectionTrigger,
    ) -> PrimitiveCollectionReport {
        let before = self.accounting();
        self.finish_collection_report(roots, trigger, &before)
    }

    fn finish_collection_report(
        &mut self,
        roots: &PrimitiveRoots,
        trigger: PrimitiveCollectionTrigger,
        before: &PrimitiveHeapAccounting,
    ) -> PrimitiveCollectionReport {
        let mut metrics = PrimitiveMajorMarkMetrics::new(self.major_mark_slice_budget().max(1));
        let stats = self.collect_tracing_with_mark_metrics(roots, &(), &mut metrics);
        self.record_last_major_mark_stats(&stats);
        let after = self.accounting();
        let next_budget_bytes = next_collection_budget(after.live_bytes);
        self.collection_budget_bytes = next_budget_bytes;

        PrimitiveCollectionReport {
            kind: PrimitiveCollectionKind::Major,
            trigger,
            before: *before,
            after,
            stats,
            minor: PrimitiveMinorCollectionStats::default(),
            next_budget_bytes,
        }
    }

    const fn record_last_major_mark_stats(&mut self, stats: &PrimitiveCollectionStats) {
        self.last_major_mark_slices = stats.major_mark_slices;
        self.last_major_mark_work_items = stats.major_mark_work_items;
        self.last_major_max_mark_slice_work_items = stats.max_major_mark_slice_work_items;
        self.last_major_total_mark_pause_ns = stats.total_major_mark_pause_ns;
        self.last_major_max_mark_pause_ns = stats.max_major_mark_pause_ns;
        self.last_major_mark_finish_work_items = stats.major_mark_finish_work_items;
        self.last_major_mark_finish_pause_ns = stats.major_mark_finish_pause_ns;
        self.last_major_gray_work_items_after_finish =
            stats.major_mark_gray_work_items_after_finish;
        self.last_major_background_sweep.started = stats.background_sweep_started;
        self.last_major_background_sweep.completed = stats.background_sweep_completed;
        self.last_major_background_sweep.worker_thread_id = stats.background_sweep_worker_thread_id;
        self.last_major_background_sweep.candidates = stats.background_sweep_candidates;
        self.last_major_background_sweep.reclaimed = stats.background_sweep_reclaimed;
        self.last_major_background_sweep.duration_ns = stats.background_sweep_duration_ns;
        self.last_major_background_sweep.apply_pause_ns = stats.background_sweep_apply_pause_ns;
    }
}

const fn domain_accounting<Record>(stats: PrimitiveDomainStats) -> PrimitiveDomainAccounting {
    let slot_bytes = size_of::<Record>();

    PrimitiveDomainAccounting {
        live_bytes: stats.occupied_slots * slot_bytes + stats.side_allocations.live_payload_bytes,
        young_live_bytes: stats.young_slots * slot_bytes
            + stats.side_allocations.young_live_payload_bytes,
        old_live_bytes: stats.old_slots * slot_bytes
            + stats.side_allocations.old_live_payload_bytes,
        reclaimable_bytes: stats.reusable_slots * slot_bytes
            + stats.side_allocations.reusable_reserved_bytes,
        reserved_bytes: stats.pages * PRIMITIVE_SLOTS_PER_PAGE * slot_bytes
            + stats.side_allocations.reserved_bytes,
    }
}

const fn merge_domain_accounting(
    left: PrimitiveDomainAccounting,
    right: PrimitiveDomainAccounting,
) -> PrimitiveDomainAccounting {
    PrimitiveDomainAccounting {
        live_bytes: left.live_bytes + right.live_bytes,
        young_live_bytes: left.young_live_bytes + right.young_live_bytes,
        old_live_bytes: left.old_live_bytes + right.old_live_bytes,
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
    use crate::{
        AllocationLifetime, RuntimeEnvironmentRecord, RuntimeObjectRecord,
        RuntimeSuspendedExecutionRecord, StringEncoding, SymbolFlags, ValueStoreTarget,
        WeakHeapRef,
    };
    use lyng_js_types::{CodeRef, EnvironmentRef, RealmRef, Value};

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
    fn force_collect_reports_background_sweep_completion() {
        let mut heap = PrimitiveHeap::new();
        let roots = PrimitiveRoots::new();
        let live = heap.alloc_object(
            RuntimeObjectRecord::new(None, None, None, None, None),
            AllocationLifetime::Default,
        );
        let dead = heap.alloc_object(
            RuntimeObjectRecord::new(None, None, None, None, None),
            AllocationLifetime::Default,
        );
        let _rooted = roots.root_object(live);

        let report = heap.force_collect(&roots);

        assert!(report.stats.background_sweep_started);
        assert!(report.stats.background_sweep_completed);
        assert_ne!(report.stats.background_sweep_worker_thread_id, 0);
        assert!(report.stats.background_sweep_candidates >= 1);
        assert_eq!(report.stats.background_sweep_reclaimed, 1);
        assert!(report.stats.background_sweep_duration_ns > 0);
        assert!(report.stats.background_sweep_apply_pause_ns > 0);
        assert_eq!(
            heap.accounting().last_major_background_sweep_reclaimed,
            report.stats.background_sweep_reclaimed
        );
        assert!(heap.view().object(live).is_some());
        assert!(heap.view().object(dead).is_none());
    }

    #[test]
    fn pending_background_sweep_defers_reuse_until_synchronized() {
        let mut heap = PrimitiveHeap::new();
        let dead = heap.alloc_object(
            RuntimeObjectRecord::new(None, None, None, None, None),
            AllocationLifetime::Default,
        );

        let candidates = heap.collect_major_sweep_candidates();
        heap.start_background_sweep(candidates);
        assert!(
            heap.view().object(dead).is_some(),
            "candidate storage must stay occupied until the sweep plan is applied"
        );

        let allocated_while_pending = heap.alloc_object(
            RuntimeObjectRecord::new(None, None, None, None, None),
            AllocationLifetime::Default,
        );
        assert_ne!(
            dead, allocated_while_pending,
            "allocation must not reuse pending sweep candidates before synchronization"
        );

        let (reclaimed, stats) = heap
            .complete_background_sweep()
            .expect("background sweep should be pending");

        assert_eq!(reclaimed.objects, 1);
        assert!(stats.completed);
        assert_eq!(stats.reclaimed, 1);
        assert!(heap.view().object(dead).is_none());
        assert!(heap.view().object(allocated_while_pending).is_some());
    }

    #[test]
    fn accounting_reports_default_allocations_as_young_and_long_lived_allocations_as_old() {
        let mut heap = PrimitiveHeap::new();
        let _string = heap.alloc_string(
            StringEncoding::Latin1,
            4,
            b"live",
            None,
            AllocationLifetime::Default,
        );
        let _object = heap.alloc_object(
            RuntimeObjectRecord::new(None, None, None, None, None),
            AllocationLifetime::Default,
        );
        let _shape = heap.alloc_shape(
            RuntimeShapeRecord::new(None, None, 0),
            AllocationLifetime::Default,
        );

        let string_stats = heap.string_stats();
        assert_eq!(string_stats.young_slots, 1);
        assert_eq!(string_stats.old_slots, 0);
        assert_eq!(string_stats.side_allocations.young_allocations, 1);
        assert_eq!(string_stats.side_allocations.old_allocations, 0);

        let accounting = heap.accounting();
        assert!(accounting.young_live_bytes > 0);
        assert!(accounting.old_live_bytes > 0);
        assert_eq!(
            accounting.strings.young_live_bytes,
            accounting.strings.live_bytes
        );
        assert_eq!(
            accounting.objects.young_live_bytes,
            accounting.objects.live_bytes
        );
        assert_eq!(
            accounting.shapes.old_live_bytes,
            accounting.shapes.live_bytes
        );
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

    #[test]
    fn weak_refs_and_finalizers_settle_before_background_sweep_reclaims_targets() {
        let mut heap = PrimitiveHeap::new();
        let weak_ref = heap.alloc_object(
            RuntimeObjectRecord::new(None, None, None, None, None),
            AllocationLifetime::Default,
        );
        let weak_target = heap.alloc_object(
            RuntimeObjectRecord::new(None, None, None, None, None),
            AllocationLifetime::Default,
        );
        let registry = heap.alloc_object(
            RuntimeObjectRecord::new(None, None, None, None, None),
            AllocationLifetime::Default,
        );
        let finalization_target = heap.alloc_object(
            RuntimeObjectRecord::new(None, None, None, None, None),
            AllocationLifetime::Default,
        );
        let holdings = heap.alloc_string(
            StringEncoding::Latin1,
            4,
            b"held",
            None,
            AllocationLifetime::Default,
        );

        {
            let mut mutator = heap.mutator();
            assert!(mutator.init_weak_ref(weak_ref, WeakHeapRef::Object(weak_target)));
            assert!(mutator.init_finalization_registry(registry));
            assert!(mutator.finalization_registry_register(
                registry,
                WeakHeapRef::Object(finalization_target),
                Value::from_string_ref(holdings),
                None,
            ));
        }

        assert!(heap.mark_object(weak_ref));
        assert!(heap.mark_object(registry));
        assert!(heap.mark_string(holdings));

        let (weak_refs_cleared, finalization_cells_queued, pending_registries) =
            heap.sweep_weak_state();

        assert_eq!(weak_refs_cleared, 1);
        assert_eq!(finalization_cells_queued, 1);
        assert_eq!(pending_registries, 1);
        assert_eq!(heap.view().weak_ref_target(weak_ref), Some(None));
        assert_eq!(heap.pending_finalization_registries(), &[registry]);
        assert!(heap.view().object(weak_target).is_some());
        assert!(heap.view().object(finalization_target).is_some());

        let candidates = heap.collect_major_sweep_candidates();
        heap.start_background_sweep(candidates);
        assert!(heap.view().object(weak_target).is_some());
        assert!(heap.view().object(finalization_target).is_some());

        let (reclaimed, stats) = heap
            .complete_background_sweep()
            .expect("background sweep should be pending");

        assert_eq!(reclaimed.objects, 2);
        assert_eq!(stats.reclaimed, 2);
        assert!(heap.view().object(weak_ref).is_some());
        assert!(heap.view().object(registry).is_some());
        assert!(heap.view().object(weak_target).is_none());
        assert!(heap.view().object(finalization_target).is_none());
        assert!(heap.view().string(holdings).is_some());
        assert_eq!(
            heap.take_finalization_cleanup_holdings(registry),
            vec![Value::from_string_ref(holdings)]
        );
    }

    #[test]
    fn nursery_minor_collection_promotes_rooted_survivors_and_reclaims_dead_young_objects() {
        let mut heap = PrimitiveHeap::new();
        heap.set_nursery_capacity_bytes(256);
        let roots = PrimitiveRoots::new();

        let (live, dead) = {
            let mut mutator = heap.mutator_with_roots(&roots);
            let live = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::Default,
            );
            let dead = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::Default,
            );
            (live, dead)
        };
        let _rooted_live = roots.root_object(live);

        let report = heap.force_minor_collect(&roots);

        assert_eq!(report.kind, PrimitiveCollectionKind::Minor);
        assert_eq!(report.minor.survivors, 1);
        assert_eq!(report.minor.tenured, 1);
        assert_eq!(report.minor.reclaimed, 1);
        assert!(heap.view().object(live).is_some());
        assert!(heap.view().object(dead).is_none());
        assert_eq!(heap.accounting().young_live_bytes, 0);
        assert_eq!(heap.nursery_stats().minor_collections, 1);
        assert!(heap.nursery_stats().last_minor_pause_ns > 0);
    }

    #[test]
    fn rooted_mutator_runs_minor_collection_when_nursery_capacity_is_exhausted() {
        let mut heap = PrimitiveHeap::new();
        heap.set_nursery_capacity_bytes(std::mem::size_of::<RuntimeObjectRecord>());
        let roots = PrimitiveRoots::new();

        let mut mutator = heap.mutator_with_roots(&roots);
        let _first = mutator.alloc_object(
            RuntimeObjectRecord::new(None, None, None, None, None),
            AllocationLifetime::Default,
        );
        let _second = mutator.alloc_object(
            RuntimeObjectRecord::new(None, None, None, None, None),
            AllocationLifetime::Default,
        );

        let report = mutator
            .last_collection_report()
            .expect("nursery exhaustion should trigger a minor collection");
        assert_eq!(report.kind, PrimitiveCollectionKind::Minor);
        assert_eq!(
            report.trigger,
            PrimitiveCollectionTrigger::NurseryAllocationLimit
        );
        assert_eq!(report.minor.reclaimed, 1);
        assert_eq!(
            mutator.accounting().young_live_bytes,
            std::mem::size_of::<RuntimeObjectRecord>()
        );
    }

    #[test]
    fn tenuring_threshold_keeps_survivors_young_until_configured_age() {
        let mut heap = PrimitiveHeap::new();
        heap.set_nursery_tenuring_threshold(2);
        let roots = PrimitiveRoots::new();

        let live = heap.mutator().alloc_object(
            RuntimeObjectRecord::new(None, None, None, None, None),
            AllocationLifetime::Default,
        );
        let _rooted_live = roots.root_object(live);

        let first = heap.force_minor_collect(&roots);
        assert_eq!(first.minor.survivors, 1);
        assert_eq!(first.minor.tenured, 0);
        assert!(heap.accounting().young_live_bytes > 0);
        assert_eq!(heap.nursery_age(live), Some(1));

        let second = heap.force_minor_collect(&roots);
        assert_eq!(second.minor.survivors, 1);
        assert_eq!(second.minor.tenured, 1);
        assert_eq!(heap.accounting().young_live_bytes, 0);
        assert_eq!(heap.nursery_age(live), None);
    }

    #[test]
    fn old_to_young_slot_write_marks_card_and_preserves_referent_during_minor_collection() {
        let mut heap = PrimitiveHeap::new();
        let roots = PrimitiveRoots::new();

        let (old, slots, young) = {
            let mut mutator = heap.mutator();
            let old = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::LongLived,
            );
            let slots = mutator.alloc_object_slots(
                1,
                Value::empty_internal_slot(),
                AllocationLifetime::LongLived,
            );
            assert!(mutator.init_store_object_slots_handle(
                crate::ObjectSlotsHandleStoreTarget::ObjectNamedSlots(old),
                Some(slots),
            ));
            let young = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::Default,
            );
            assert!(mutator.mut_store_value(
                ValueStoreTarget::ObjectSlot(slots, 0),
                Value::from_object_ref(young),
            ));
            (old, slots, young)
        };
        let _rooted_old = roots.root_object(old);

        let report = heap.force_minor_collect(&roots);

        assert_eq!(report.kind, PrimitiveCollectionKind::Minor);
        assert_eq!(report.minor.cards_scanned, 1);
        assert_eq!(report.minor.cards_dirtied, 1);
        assert_eq!(report.minor.survivors, 1);
        assert!(heap.view().object(young).is_some());
        assert_eq!(
            heap.view()
                .object_slots(slots)
                .and_then(|values| values[0].as_object_ref()),
            Some(young)
        );
    }

    #[test]
    fn incremental_major_mark_step_respects_budget_and_preserves_live_chain() {
        let mut heap = PrimitiveHeap::new();
        let roots = PrimitiveRoots::new();
        let (root, child, grandchild, dead) = {
            let mut mutator = heap.mutator();
            let grandchild = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::Default,
            );
            let child_slots = mutator.alloc_object_slots(
                1,
                Value::from_object_ref(grandchild),
                AllocationLifetime::Default,
            );
            let child = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, Some(child_slots), None, None),
                AllocationLifetime::Default,
            );
            let root_slots = mutator.alloc_object_slots(
                1,
                Value::from_object_ref(child),
                AllocationLifetime::Default,
            );
            let root = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, Some(root_slots), None, None),
                AllocationLifetime::Default,
            );
            let dead = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::Default,
            );
            (root, child, grandchild, dead)
        };
        let _rooted = roots.root_object(root);

        let mut marker = heap.start_incremental_mark(&roots);
        let first = heap.mark_step(&mut marker, 1);

        assert_eq!(first.work_items_processed, 1);
        assert_eq!(first.max_work_items_per_slice, 1);
        assert!(first.has_more_work());
        while heap.mark_step(&mut marker, 1).has_more_work() {}

        let stats = heap.finish_incremental_mark(marker);
        let view = heap.view();

        assert_eq!(stats.trace.objects_marked, 3);
        assert_eq!(stats.objects_reclaimed, 1);
        assert!(view.object(root).is_some());
        assert!(view.object(child).is_some());
        assert!(view.object(grandchild).is_some());
        assert_eq!(view.object(dead), None);
    }

    #[test]
    fn incremental_major_barrier_preserves_slot_referent_written_after_owner_is_marked() {
        let mut heap = PrimitiveHeap::new();
        let roots = PrimitiveRoots::new();
        let (root, slots, late_child) = {
            let mut mutator = heap.mutator();
            let slots = mutator.alloc_object_slots(
                1,
                Value::empty_internal_slot(),
                AllocationLifetime::Default,
            );
            let root = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, Some(slots), None, None),
                AllocationLifetime::Default,
            );
            let late_child = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::Default,
            );
            (root, slots, late_child)
        };
        let _rooted = roots.root_object(root);

        assert!(heap.begin_incremental_mark(&roots));
        while heap
            .poll_incremental_mark_step()
            .is_some_and(crate::PrimitiveMarkStep::has_more_work)
        {}

        assert!(heap.mutator().mut_store_value(
            ValueStoreTarget::ObjectSlot(slots, 0),
            Value::from_object_ref(late_child),
        ));

        let stats = heap.finish_active_incremental_mark().unwrap();

        assert_eq!(stats.trace.objects_marked, 2);
        assert!(heap.view().object(late_child).is_some());
        assert_eq!(
            heap.view()
                .object_slots(slots)
                .and_then(|values| values[0].as_object_ref()),
            Some(late_child)
        );
    }

    #[test]
    fn incremental_major_barrier_preserves_handle_referent_written_after_owner_is_marked() {
        let mut heap = PrimitiveHeap::new();
        let roots = PrimitiveRoots::new();
        let (root, late_prototype) = {
            let mut mutator = heap.mutator();
            let root = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::Default,
            );
            let late_prototype = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::Default,
            );
            (root, late_prototype)
        };
        let _rooted = roots.root_object(root);

        assert!(heap.begin_incremental_mark(&roots));
        while heap
            .poll_incremental_mark_step()
            .is_some_and(crate::PrimitiveMarkStep::has_more_work)
        {}

        assert!(heap.mutator().mut_store_object_handle(
            crate::ObjectHandleStoreTarget::ObjectPrototype(root),
            Some(late_prototype),
        ));

        let stats = heap.finish_active_incremental_mark().unwrap();

        assert_eq!(stats.trace.objects_marked, 2);
        assert!(heap.view().object(late_prototype).is_some());
        assert_eq!(
            heap.view()
                .object(root)
                .and_then(RuntimeObjectRecord::prototype),
            Some(late_prototype)
        );
    }

    #[test]
    fn force_collect_reports_major_mark_slice_distribution() {
        let mut heap = PrimitiveHeap::new();
        heap.set_major_mark_slice_budget(1);
        let roots = PrimitiveRoots::new();
        let (root, child, extra_root_a, extra_root_b) = {
            let mut mutator = heap.mutator();
            let child = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::Default,
            );
            let root_slots = mutator.alloc_object_slots(
                1,
                Value::from_object_ref(child),
                AllocationLifetime::Default,
            );
            let root = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, Some(root_slots), None, None),
                AllocationLifetime::Default,
            );
            let extra_root_a = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::Default,
            );
            let extra_root_b = mutator.alloc_object(
                RuntimeObjectRecord::new(None, None, None, None, None),
                AllocationLifetime::Default,
            );
            (root, child, extra_root_a, extra_root_b)
        };
        let _rooted = roots.root_object(root);
        let _extra_root_a = roots.root_object(extra_root_a);
        let _extra_root_b = roots.root_object(extra_root_b);

        let report = heap.force_collect(&roots);

        assert_eq!(report.stats.trace.objects_marked, 4);
        assert!(report.stats.major_mark_slices >= 2);
        assert_eq!(report.stats.major_mark_slice_budget, 1);
        assert_eq!(report.stats.max_major_mark_slice_work_items, 1);
        assert!(report.stats.total_major_mark_pause_ns > 0);
        assert!(report.stats.max_major_mark_pause_ns > 0);
        assert!(report.stats.major_mark_finish_work_items > 0);
        assert!(report.stats.major_mark_finish_pause_ns > 0);
        assert_eq!(report.stats.major_mark_gray_work_items_after_finish, 0);
        assert_eq!(
            heap.accounting().last_major_mark_slices,
            report.stats.major_mark_slices
        );
        assert_eq!(
            heap.accounting().last_major_max_mark_slice_work_items,
            report.stats.max_major_mark_slice_work_items
        );
        assert_eq!(
            heap.accounting().last_major_mark_finish_work_items,
            report.stats.major_mark_finish_work_items
        );
        assert_eq!(
            heap.accounting().last_major_mark_finish_pause_ns,
            report.stats.major_mark_finish_pause_ns
        );
        assert_eq!(heap.accounting().last_major_gray_work_items_after_finish, 0);
        assert!(heap.view().object(child).is_some());
    }

    #[test]
    fn allocation_profile_counts_common_and_long_tail_nursery_domains() {
        let mut heap = PrimitiveHeap::new();
        let mut mutator = heap.mutator();

        let _string = mutator.alloc_string(
            StringEncoding::Latin1,
            4,
            b"test",
            None,
            AllocationLifetime::Default,
        );
        let _symbol =
            mutator.alloc_symbol(None, SymbolFlags::ordinary(), AllocationLifetime::Default);
        let _bigint = mutator.alloc_bigint(
            crate::BigIntSign::NonNegative,
            &[1],
            AllocationLifetime::Default,
        );
        let _value_cell = mutator.alloc_value_cell(AllocationLifetime::Default);
        let env_slots =
            mutator.alloc_environment_slots(1, Value::undefined(), AllocationLifetime::Default);
        let env = mutator.alloc_environment(
            RuntimeEnvironmentRecord::new(
                None,
                Some(env_slots),
                None,
                Value::undefined(),
                None,
                None,
            ),
            AllocationLifetime::Default,
        );
        let registers =
            mutator.alloc_suspended_registers(1, Value::undefined(), AllocationLifetime::Default);
        let _suspended = mutator.alloc_suspended_execution(
            RuntimeSuspendedExecutionRecord::new(
                RealmRef::from_raw(1).unwrap(),
                CodeRef::from_raw(1).unwrap(),
                0,
                env,
                EnvironmentRef::from_raw(env.get()).unwrap(),
                None,
                Value::undefined(),
                0,
                None,
                None,
                None,
                0,
                0,
                0,
                Some(registers),
            ),
            AllocationLifetime::Default,
        );
        let _shape = mutator.alloc_shape(
            crate::RuntimeShapeRecord::new(None, None, 0),
            AllocationLifetime::Default,
        );

        let profile = mutator.accounting().allocation_profile;
        assert!(profile.nursery_allocations >= 8);
        assert_eq!(profile.old_allocations, 1);
        assert!(profile.nursery_allocation_ratio() >= 80);
    }
}
