use lyng_js_gc::{PrimitiveAllocationProfile, PrimitiveDomainAccounting, PrimitiveHeapAccounting};

/// Runtime-owned memory summary for one Phase 6 domain outside the primitive heap.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RuntimeDomainAccounting {
    pub records: usize,
    pub metadata_bytes: usize,
    pub payload_bytes: usize,
    pub live_bytes: usize,
}

impl RuntimeDomainAccounting {
    #[inline]
    pub const fn empty() -> Self {
        Self {
            records: 0,
            metadata_bytes: 0,
            payload_bytes: 0,
            live_bytes: 0,
        }
    }

    #[inline]
    pub const fn merge(self, other: Self) -> Self {
        Self {
            records: self.records + other.records,
            metadata_bytes: self.metadata_bytes + other.metadata_bytes,
            payload_bytes: self.payload_bytes + other.payload_bytes,
            live_bytes: self.live_bytes + other.live_bytes,
        }
    }
}

/// Agent-local Phase 6 accounting snapshot layered on top of primitive-heap accounting.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AgentPhase6Accounting {
    pub heap: PrimitiveHeapAccounting,
    pub iterator_records: RuntimeDomainAccounting,
    pub regexp_payloads: RuntimeDomainAccounting,
    pub regexp_literal_cache: RuntimeDomainAccounting,
    pub module_caches: RuntimeDomainAccounting,
    pub promise_jobs: RuntimeDomainAccounting,
    pub live_bytes: usize,
}

/// Runtime-wide Phase 6 accounting snapshot exposed to reports and benchmarks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RuntimePhase6Accounting {
    pub heap: PrimitiveHeapAccounting,
    pub iterator_records: RuntimeDomainAccounting,
    pub regexp_payloads: RuntimeDomainAccounting,
    pub regexp_literal_cache: RuntimeDomainAccounting,
    pub module_caches: RuntimeDomainAccounting,
    pub promise_jobs: RuntimeDomainAccounting,
    pub backing_stores: RuntimeDomainAccounting,
    pub live_bytes: usize,
}

#[inline]
pub const fn total_live_bytes(
    heap: &PrimitiveHeapAccounting,
    iterator_records: RuntimeDomainAccounting,
    regexp_payloads: RuntimeDomainAccounting,
    regexp_literal_cache: RuntimeDomainAccounting,
    module_caches: RuntimeDomainAccounting,
    promise_jobs: RuntimeDomainAccounting,
    backing_stores: RuntimeDomainAccounting,
) -> usize {
    heap.live_bytes
        + iterator_records.live_bytes
        + regexp_payloads.live_bytes
        + regexp_literal_cache.live_bytes
        + module_caches.live_bytes
        + promise_jobs.live_bytes
        + backing_stores.live_bytes
}

#[inline]
pub const fn merge_primitive_domain_accounting(
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

#[allow(clippy::too_many_lines)]
pub const fn merge_primitive_heap_accounting(
    left: &PrimitiveHeapAccounting,
    right: &PrimitiveHeapAccounting,
) -> PrimitiveHeapAccounting {
    PrimitiveHeapAccounting {
        strings: merge_primitive_domain_accounting(left.strings, right.strings),
        symbols: merge_primitive_domain_accounting(left.symbols, right.symbols),
        bigints: merge_primitive_domain_accounting(left.bigints, right.bigints),
        value_cells: merge_primitive_domain_accounting(left.value_cells, right.value_cells),
        objects: merge_primitive_domain_accounting(left.objects, right.objects),
        environments: merge_primitive_domain_accounting(left.environments, right.environments),
        codes: merge_primitive_domain_accounting(left.codes, right.codes),
        realms: merge_primitive_domain_accounting(left.realms, right.realms),
        shapes: merge_primitive_domain_accounting(left.shapes, right.shapes),
        live_bytes: left.live_bytes + right.live_bytes,
        young_live_bytes: left.young_live_bytes + right.young_live_bytes,
        old_live_bytes: left.old_live_bytes + right.old_live_bytes,
        reclaimable_bytes: left.reclaimable_bytes + right.reclaimable_bytes,
        reserved_bytes: left.reserved_bytes + right.reserved_bytes,
        allocation_profile: PrimitiveAllocationProfile {
            nursery_allocations: left.allocation_profile.nursery_allocations
                + right.allocation_profile.nursery_allocations,
            old_allocations: left.allocation_profile.old_allocations
                + right.allocation_profile.old_allocations,
        },
        nursery_capacity_bytes: left.nursery_capacity_bytes + right.nursery_capacity_bytes,
        nursery_used_bytes: left.nursery_used_bytes + right.nursery_used_bytes,
        minor_collections: left.minor_collections + right.minor_collections,
        last_minor_pause_ns: left.last_minor_pause_ns + right.last_minor_pause_ns,
        last_minor_survivors: left.last_minor_survivors + right.last_minor_survivors,
        last_minor_tenured: left.last_minor_tenured + right.last_minor_tenured,
        last_minor_reclaimed: left.last_minor_reclaimed + right.last_minor_reclaimed,
        last_minor_cards_dirtied: left.last_minor_cards_dirtied + right.last_minor_cards_dirtied,
        last_minor_cards_scanned: left.last_minor_cards_scanned + right.last_minor_cards_scanned,
        last_major_mark_slices: left.last_major_mark_slices + right.last_major_mark_slices,
        last_major_mark_slice_budget: left.last_major_mark_slice_budget
            + right.last_major_mark_slice_budget,
        last_major_mark_work_items: left.last_major_mark_work_items
            + right.last_major_mark_work_items,
        last_major_max_mark_slice_work_items: if left.last_major_max_mark_slice_work_items
            > right.last_major_max_mark_slice_work_items
        {
            left.last_major_max_mark_slice_work_items
        } else {
            right.last_major_max_mark_slice_work_items
        },
        last_major_total_mark_pause_ns: left.last_major_total_mark_pause_ns
            + right.last_major_total_mark_pause_ns,
        last_major_max_mark_pause_ns: if left.last_major_max_mark_pause_ns
            > right.last_major_max_mark_pause_ns
        {
            left.last_major_max_mark_pause_ns
        } else {
            right.last_major_max_mark_pause_ns
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ExecutableId, Runtime};
    use lyng_js_gc::AllocationLifetime;
    use lyng_js_host::{HostJobKind, HostSharedBufferId, NoopHostHooks};
    use lyng_js_objects::{ObjectAllocation, ObjectColdData, OrdinaryObjectData, RegExpPayload};
    use lyng_js_types::CodeRef;

    #[test]
    fn empty_runtime_phase6_accounting_starts_with_zero_future_domains() {
        let runtime = Runtime::new(NoopHostHooks);
        let accounting = runtime.phase6_accounting();

        assert_eq!(
            accounting.iterator_records,
            RuntimeDomainAccounting::empty()
        );
        assert_eq!(accounting.regexp_payloads, RuntimeDomainAccounting::empty());
        assert_eq!(
            accounting.regexp_literal_cache,
            RuntimeDomainAccounting::empty()
        );
        assert_eq!(accounting.module_caches, RuntimeDomainAccounting::empty());
        assert_eq!(accounting.promise_jobs, RuntimeDomainAccounting::empty());
        assert_eq!(accounting.backing_stores, RuntimeDomainAccounting::empty());
        assert_eq!(accounting.live_bytes, accounting.heap.live_bytes);
    }

    #[test]
    fn runtime_phase6_accounting_reports_promise_jobs_and_backing_stores() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let root = runtime.root_agent_id();
        let worker = runtime
            .root_cluster_mut()
            .add_agent(None, Some("bench-worker".into()));
        let local_store = runtime
            .root_agent_mut()
            .allocate_backing_store(128)
            .expect("local backing store should allocate");
        let shared_store = runtime
            .root_cluster_mut()
            .register_shared_backing_store(root, 4096)
            .expect("shared backing store should allocate");
        assert!(runtime
            .root_cluster_mut()
            .cache_shared_backing_store_handle(
                shared_store,
                HostSharedBufferId::from_raw(7).unwrap()
            ));
        assert!(runtime
            .root_cluster_mut()
            .share_shared_backing_store(shared_store, worker));
        runtime
            .enqueue_job(
                root,
                HostJobKind::Promise,
                ExecutableId::Builtin,
                None,
                Some("phase6-promise".into()),
            )
            .unwrap();

        let accounting = runtime.phase6_accounting();

        assert_eq!(
            runtime.root_agent().backing_store_byte_length(local_store),
            Some(128)
        );
        assert_eq!(accounting.promise_jobs.records, 1);
        assert!(accounting.promise_jobs.metadata_bytes >= std::mem::size_of::<crate::RuntimeJob>());
        assert!(accounting.promise_jobs.payload_bytes >= "phase6-promise".len());
        assert_eq!(accounting.backing_stores.records, 2);
        assert_eq!(accounting.backing_stores.payload_bytes, 4096 + 128);
        assert!(accounting.backing_stores.metadata_bytes > 0);
        assert_eq!(
            accounting.iterator_records,
            RuntimeDomainAccounting::empty()
        );
        assert_eq!(accounting.regexp_payloads, RuntimeDomainAccounting::empty());
        assert_eq!(
            accounting.regexp_literal_cache,
            RuntimeDomainAccounting::empty()
        );
        assert_eq!(accounting.module_caches, RuntimeDomainAccounting::empty());
        assert_eq!(
            accounting.live_bytes,
            accounting.heap.live_bytes
                + accounting.regexp_payloads.live_bytes
                + accounting.regexp_literal_cache.live_bytes
                + accounting.promise_jobs.live_bytes
                + accounting.backing_stores.live_bytes
        );
    }

    #[test]
    fn runtime_phase6_accounting_reports_regexp_payloads() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let root = objects.root_shape(&mut mutator, None, AllocationLifetime::Default);
            let regexp = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root)
                    .with_cold_data(ObjectColdData::Ordinary(OrdinaryObjectData::RegExp)),
                AllocationLifetime::Default,
            );
            assert!(
                objects.store_regexp_payload(regexp, RegExpPayload::compile("a", "dg").unwrap())
            );
        });

        let accounting = runtime.phase6_accounting();

        assert_eq!(accounting.regexp_payloads.records, 1);
        assert!(accounting.regexp_payloads.metadata_bytes > 0);
        assert!(accounting.regexp_payloads.payload_bytes > 0);
        assert!(
            accounting.live_bytes
                >= accounting.heap.live_bytes + accounting.regexp_payloads.live_bytes
        );
    }

    #[test]
    fn runtime_phase6_accounting_reports_regexp_literal_cache() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .expect("default realm should exist")
            .id();
        let code = CodeRef::from_raw(17).expect("test code ref should be non-zero");

        assert!(agent.cache_regexp_literal_payload(
            realm,
            code,
            3,
            RegExpPayload::compile("cache", "g").unwrap()
        ));
        assert!(!agent.cache_regexp_literal_payload(
            realm,
            code,
            3,
            RegExpPayload::compile("other", "").unwrap()
        ));

        let cached = agent
            .regexp_literal_cached_payload(realm, code, 3)
            .expect("literal payload should be cached");
        assert_eq!(cached.source(), "cache");
        assert_eq!(cached.flag_text(), "g");

        let accounting = runtime.phase6_accounting();

        assert_eq!(accounting.regexp_payloads.records, 0);
        assert_eq!(accounting.regexp_literal_cache.records, 1);
        assert!(accounting.regexp_literal_cache.metadata_bytes > 0);
        assert!(accounting.regexp_literal_cache.payload_bytes > 0);
        assert!(
            accounting.live_bytes
                >= accounting.heap.live_bytes + accounting.regexp_literal_cache.live_bytes
        );
    }
}
