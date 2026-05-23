//! Cross-crate integration coverage for the runtime primitive layer.

use lyng_js_common::{AtomId, AtomLifetime, AtomTable};
use lyng_js_gc::{
    AllocationLifetime, AtomGcSweep, BigIntSign, PrimitiveAtomMetadata, PrimitiveCollectionTrigger,
    PrimitiveHeap, PrimitiveRoots, PrimitiveStringRecord, PrimitiveSymbolClass, StringEncoding,
    StringHandleStoreTarget, SymbolFlags, TraceAtomEdges, ValueStoreTarget,
    PRIMITIVE_SLOTS_PER_PAGE,
};
use lyng_js_types::Value;

#[test]
fn nested_runtime_atom_edges_keep_live_atoms_across_heap_records() {
    let mut atoms = AtomTable::new();
    let cached_atom = atoms.intern_collectible("cached-edge");
    let metadata_atom = atoms.intern_collectible("metadata-edge");
    let dead_atom = atoms.intern_collectible("dead-edge");
    let permanent_atom = atoms.intern("permanent-edge");

    let string_record =
        PrimitiveStringRecord::with_cached_atom(StringEncoding::Utf16, 12, cached_atom);
    let collectible_metadata = PrimitiveAtomMetadata::new(Some(metadata_atom));
    let metadata = PrimitiveAtomMetadata::new(Some(permanent_atom));

    let mut sweep = AtomGcSweep::new(&mut atoms);
    string_record.trace_atom_edges(&mut sweep);
    collectible_metadata.trace_atom_edges(&mut sweep);
    metadata.trace_atom_edges(&mut sweep);
    let stats = sweep.sweep();

    assert_eq!(stats.reclaimed_collectible, 1);
    assert_eq!(stats.retained_collectible, 2);
    assert_eq!(atoms.resolve(cached_atom), "cached-edge");
    assert_eq!(atoms.resolve(metadata_atom), "metadata-edge");
    assert_eq!(
        atoms.lifetime(permanent_atom),
        Some(AtomLifetime::Permanent)
    );
    assert_eq!(atoms.get(dead_atom), None);
}

#[test]
fn post_sweep_reuse_preserves_live_cross_crate_state() {
    let mut atoms = AtomTable::new();
    let cached_atom = atoms.intern_collectible("cached-edge");
    let metadata_atom = atoms.intern_collectible("metadata-edge");
    let dead_atom = atoms.intern_collectible("dead-edge");

    let string_record =
        PrimitiveStringRecord::with_cached_atom(StringEncoding::Latin1, 10, cached_atom);
    let metadata = PrimitiveAtomMetadata::new(Some(metadata_atom));

    let mut sweep = AtomGcSweep::new(&mut atoms);
    string_record.trace_atom_edges(&mut sweep);
    metadata.trace_atom_edges(&mut sweep);
    let stats = sweep.sweep();

    assert_eq!(stats.reclaimed_collectible, 1);
    assert_eq!(atoms.resolve(cached_atom), "cached-edge");
    assert_eq!(atoms.resolve(metadata_atom), "metadata-edge");

    let replacement = atoms.intern_collectible("replacement-edge");
    assert_eq!(replacement, dead_atom);
    assert_eq!(atoms.resolve(replacement), "replacement-edge");
    assert_eq!(string_record.cached_atom(), Some(cached_atom));
    assert_eq!(metadata.retained_atom(), Some(metadata_atom));
}

#[test]
fn primitive_heap_allocates_domain_records_and_reuses_string_slots() {
    let mut heap = PrimitiveHeap::new();
    let mut mutator = heap.mutator();
    let description_atom = AtomId::from_raw(31);
    let description = mutator.alloc_string(
        StringEncoding::Latin1,
        4,
        b"desc",
        Some(description_atom),
        AllocationLifetime::Default,
    );
    let reusable = mutator.alloc_string(
        StringEncoding::Latin1,
        3,
        b"tmp",
        None,
        AllocationLifetime::Default,
    );
    let symbol = mutator.alloc_symbol(
        Some(description),
        SymbolFlags::well_known(),
        AllocationLifetime::LongLived,
    );
    let bigint = mutator.alloc_bigint(
        BigIntSign::Negative,
        &[7, 5, 0],
        AllocationLifetime::LongLived,
    );
    let view = mutator.view();

    assert_eq!(
        view.string(description).unwrap().cached_atom(),
        Some(description_atom)
    );
    assert_eq!(view.string_payload(description), Some(&b"desc"[..]));
    assert_eq!(
        view.symbol(symbol).unwrap().description(),
        Some(description)
    );
    assert!(view.symbol(symbol).unwrap().flags().is_well_known());
    assert_eq!(view.bigint(bigint).unwrap().sign(), BigIntSign::Negative);
    assert_eq!(view.bigint_limbs(bigint), Some(vec![7, 5]));
    assert_eq!(view.string_stats().default_slots, 2);
    assert_eq!(view.symbol_stats().long_lived_slots, 1);
    assert_eq!(view.bigint_stats().long_lived_slots, 1);

    let reusable_payload = view.string(reusable).unwrap().payload().unwrap();
    let _ = mutator.free_string(reusable).unwrap();
    let replacement = mutator.alloc_string(
        StringEncoding::Latin1,
        3,
        b"new",
        None,
        AllocationLifetime::Default,
    );
    let view = mutator.view();

    assert_eq!(replacement, reusable);
    assert_eq!(view.string_payload(replacement), Some(&b"new"[..]));
    assert_eq!(
        view.string(replacement).unwrap().payload(),
        Some(reusable_payload)
    );
}

#[test]
fn explicit_root_scopes_keep_symbol_value_edges_alive_across_collection() {
    let mut heap = PrimitiveHeap::new();
    let roots = PrimitiveRoots::new();
    let (description, symbol, dead) = {
        let mut mutator = heap.mutator();
        let description = mutator.alloc_string(
            StringEncoding::Latin1,
            4,
            b"desc",
            None,
            AllocationLifetime::Default,
        );
        let symbol = mutator.alloc_symbol(
            Some(description),
            SymbolFlags::ordinary(),
            AllocationLifetime::Default,
        );
        let dead = mutator.alloc_string(
            StringEncoding::Latin1,
            4,
            b"dead",
            None,
            AllocationLifetime::Default,
        );
        (description, symbol, dead)
    };

    {
        let scope = roots.scope();
        let _rooted_value = scope.root_value(Value::from_symbol_ref(symbol));
        let stats = heap.collect(&roots);
        let view = heap.view();

        assert_eq!(scope.active_root_count(), 1);
        assert_eq!(stats.trace.values_traced, 1);
        assert_eq!(stats.trace.symbols_marked, 1);
        assert_eq!(stats.trace.strings_marked, 1);
        assert_eq!(stats.strings_reclaimed, 1);
        assert_eq!(
            view.symbol(symbol).unwrap().description(),
            Some(description)
        );
        assert_eq!(view.string(dead), None);
    }

    let stats = heap.collect(&roots);
    let view = heap.view();
    assert_eq!(stats.symbols_reclaimed, 1);
    assert_eq!(stats.strings_reclaimed, 1);
    assert_eq!(view.symbol(symbol), None);
    assert_eq!(view.string(description), None);
}

#[test]
fn barrier_ready_store_helpers_mutate_only_through_mutator_api() {
    let mut heap = PrimitiveHeap::new();
    let roots = PrimitiveRoots::new();
    let (string, symbol, cell) = {
        let mut mutator = heap.mutator();
        let string = mutator.alloc_string(
            StringEncoding::Latin1,
            4,
            b"edge",
            None,
            AllocationLifetime::Default,
        );
        let symbol =
            mutator.alloc_symbol(None, SymbolFlags::ordinary(), AllocationLifetime::Default);
        let cell = mutator.alloc_value_cell(AllocationLifetime::Default);

        assert!(mutator.init_store_string_handle(
            StringHandleStoreTarget::SymbolDescription(symbol),
            Some(string),
        ));
        assert!(mutator.init_store_value(
            ValueStoreTarget::ValueCell(cell),
            Value::from_symbol_ref(symbol),
        ));
        assert!(mutator.init_store_string_handle(
            StringHandleStoreTarget::ValueCellLinkedString(cell),
            Some(string),
        ));
        (string, symbol, cell)
    };

    let _rooted_cell = roots.root_value_cell(cell);

    let stats = heap.collect(&roots);
    let view = heap.view();

    assert_eq!(stats.trace.value_cells_marked, 1);
    assert_eq!(stats.trace.values_traced, 1);
    assert_eq!(stats.trace.symbols_marked, 1);
    assert_eq!(stats.trace.strings_marked, 1);
    assert_eq!(stats.value_cells_reclaimed, 0);
    assert_eq!(stats.symbols_reclaimed, 0);
    assert_eq!(stats.strings_reclaimed, 0);
    assert_eq!(view.symbol(symbol).unwrap().description(), Some(string));
    assert_eq!(
        view.value_cell(cell).unwrap().stored_value(),
        Value::from_symbol_ref(symbol)
    );
    assert_eq!(view.value_cell(cell).unwrap().linked_string(), Some(string));
}

#[test]
fn rooted_mutator_slow_path_collects_before_string_page_growth() {
    let mut heap = PrimitiveHeap::new();
    heap.set_collection_budget_bytes(1);
    let roots = PrimitiveRoots::new();
    let mut mutator = heap.mutator_with_roots(&roots);
    let live = mutator.alloc_string(
        StringEncoding::Latin1,
        0,
        b"",
        None,
        AllocationLifetime::Default,
    );
    let _rooted = roots.root_string(live);

    let mut reusable = None;
    for _ in 1..PRIMITIVE_SLOTS_PER_PAGE {
        reusable = Some(mutator.alloc_string(
            StringEncoding::Latin1,
            0,
            b"",
            None,
            AllocationLifetime::Default,
        ));
    }

    let replacement = mutator.alloc_string(
        StringEncoding::Latin1,
        0,
        b"",
        None,
        AllocationLifetime::Default,
    );
    let report = mutator.last_collection_report().unwrap();
    let view = mutator.view();

    assert_eq!(
        report.trigger,
        PrimitiveCollectionTrigger::StringAllocationSlowPath
    );
    assert_eq!(report.stats.trace.strings_marked, 1);
    assert_eq!(report.stats.strings_reclaimed, PRIMITIVE_SLOTS_PER_PAGE - 1);
    assert_eq!(view.string_stats().pages, 1);
    assert_eq!(replacement, reusable.unwrap());
    assert_eq!(view.string_payload(replacement), None);
    assert_eq!(view.string(replacement).unwrap().code_unit_len(), 0);
    assert!(report.after.reclaimable_bytes > 0);
    assert_eq!(
        view.accounting().reserved_bytes,
        report.after.reserved_bytes
    );
    assert!(view.accounting().live_bytes > report.after.live_bytes);
    assert!(view.accounting().reclaimable_bytes < report.after.reclaimable_bytes);
    assert_eq!(view.collection_budget_bytes(), report.next_budget_bytes);
}

#[test]
fn force_collect_public_api_exposes_accounting_for_reports() {
    let mut heap = PrimitiveHeap::new();
    let roots = PrimitiveRoots::new();
    let live = {
        let mut mutator = heap.mutator();
        let live = mutator.alloc_string(
            StringEncoding::Latin1,
            4,
            b"live",
            None,
            AllocationLifetime::Default,
        );
        let _dead = mutator.alloc_string(
            StringEncoding::Latin1,
            4,
            b"dead",
            None,
            AllocationLifetime::Default,
        );
        live
    };

    let _rooted = roots.root_string(live);
    let report = heap.force_collect(&roots);
    let view = heap.view();

    assert_eq!(report.trigger, PrimitiveCollectionTrigger::Forced);
    assert_eq!(report.stats.trace.strings_marked, 1);
    assert_eq!(report.stats.strings_reclaimed, 1);
    assert!(report.before.live_bytes > report.after.live_bytes);
    assert!(report.after.reclaimable_bytes > 0);
    assert_eq!(view.accounting(), report.after);
    assert_eq!(view.collection_budget_bytes(), report.next_budget_bytes);
}

#[test]
fn flat_runtime_string_views_support_hash_caching_and_mixed_encoding_equality() {
    let mut heap = PrimitiveHeap::new();
    let mut mutator = heap.mutator();
    let latin1 = mutator.alloc_string(
        StringEncoding::Latin1,
        3,
        b"abc",
        Some(AtomId::from_raw(91)),
        AllocationLifetime::Default,
    );
    let utf16 = mutator.alloc_string(
        StringEncoding::Utf16,
        3,
        &[0x61, 0x00, 0x62, 0x00, 0x63, 0x00],
        None,
        AllocationLifetime::Default,
    );
    let wide = mutator.alloc_string(
        StringEncoding::Utf16,
        2,
        &[0x41, 0x00, 0xA9, 0x03],
        None,
        AllocationLifetime::Default,
    );

    let wide_view = {
        let view = mutator.view();
        view.string_view(wide).unwrap()
    };

    assert_eq!(wide_view.encoding(), StringEncoding::Utf16);
    assert_eq!(wide_view.code_unit_len(), 2);
    assert_eq!(wide_view.code_unit_at(0), Some(0x0041));
    assert_eq!(wide_view.code_unit_at(1), Some(0x03A9));
    assert_eq!(wide_view.utf16_bytes(), Some(&[0x41, 0x00, 0xA9, 0x03][..]));

    let latin1_hash = mutator.cache_string_hash(latin1).unwrap();
    let utf16_hash = mutator.cache_string_hash(utf16).unwrap();

    assert_eq!(latin1_hash, utf16_hash);
    assert!(mutator.memoize_string_atom(utf16, AtomId::from_raw(91)));

    let view = mutator.view();
    assert_eq!(
        view.string(latin1).unwrap().cached_hash(),
        Some(latin1_hash)
    );
    assert_eq!(view.string(utf16).unwrap().cached_hash(), Some(utf16_hash));
    assert_eq!(
        view.string(utf16).unwrap().cached_atom(),
        Some(AtomId::from_raw(91))
    );
    assert_eq!(view.strings_equal(latin1, utf16), Some(true));
}

#[test]
fn symbol_identity_stays_handle_based_even_with_matching_descriptions() {
    let mut heap = PrimitiveHeap::new();
    let roots = PrimitiveRoots::new();
    let (symbol_a, symbol_b, well_known) = {
        let mut mutator = heap.mutator();
        let description_a = mutator.alloc_string(
            StringEncoding::Latin1,
            4,
            b"same",
            None,
            AllocationLifetime::Default,
        );
        let description_b = mutator.alloc_string(
            StringEncoding::Utf16,
            4,
            &[0x73, 0x00, 0x61, 0x00, 0x6D, 0x00, 0x65, 0x00],
            None,
            AllocationLifetime::Default,
        );
        let symbol_a = mutator.alloc_symbol(
            Some(description_a),
            SymbolFlags::for_class(PrimitiveSymbolClass::Ordinary),
            AllocationLifetime::Default,
        );
        let symbol_b = mutator.alloc_symbol(
            Some(description_b),
            SymbolFlags::for_class(PrimitiveSymbolClass::Ordinary),
            AllocationLifetime::Default,
        );
        let well_known = mutator.alloc_symbol(
            Some(description_a),
            SymbolFlags::for_class(PrimitiveSymbolClass::WellKnown),
            AllocationLifetime::LongLived,
        );
        (symbol_a, symbol_b, well_known)
    };

    {
        let scope = roots.scope();
        let _a = scope.root_symbol(symbol_a);
        let _b = scope.root_symbol(symbol_b);
        let _wk = scope.root_symbol(well_known);
        let stats = heap.collect(&roots);
        let view = heap.view();
        let left_symbol_view = view.symbol_view(symbol_a).unwrap();
        let right_symbol_view = view.symbol_view(symbol_b).unwrap();
        let well_known_view = view.symbol_view(well_known).unwrap();

        assert_eq!(stats.trace.symbols_marked, 3);
        assert_eq!(stats.trace.strings_marked, 2);
        assert_ne!(left_symbol_view.identity(), right_symbol_view.identity());
        assert!(left_symbol_view
            .description_view()
            .unwrap()
            .equals(right_symbol_view.description_view().unwrap()));
        assert!(left_symbol_view.is_ordinary());
        assert!(well_known_view.is_well_known());
    }
}

#[test]
fn bigint_views_expose_normalized_little_endian_magnitude_without_heap_mutation() {
    let mut heap = PrimitiveHeap::new();
    let (zero, value) = {
        let mut mutator = heap.mutator();
        let zero = mutator.alloc_bigint(
            BigIntSign::Negative,
            &[0, 0, 0],
            AllocationLifetime::Default,
        );
        let value = mutator.alloc_bigint(
            BigIntSign::Negative,
            &[0x0102_0304_0506_0708, 0x8877_6655_4433_2211, 0, 0],
            AllocationLifetime::Default,
        );
        (zero, value)
    };

    let view = heap.view();
    let zero_view = view.bigint_view(zero).unwrap();
    let value_view = view.bigint_view(value).unwrap();

    assert_eq!(zero_view.sign(), BigIntSign::NonNegative);
    assert!(zero_view.is_zero());
    assert_eq!(zero_view.limb_bytes_le(), &[]);

    assert_eq!(value_view.sign(), BigIntSign::Negative);
    assert_eq!(value_view.limb_count(), 2);
    assert_eq!(value_view.limb_at(0), Some(0x0102_0304_0506_0708));
    assert_eq!(value_view.limb_at(1), Some(0x8877_6655_4433_2211));
    assert_eq!(
        value_view.limb_bytes_le(),
        &[
            0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x77, 0x88,
        ][..]
    );
}
