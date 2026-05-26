//! Microbenchmark: property-addition hot path regression guard (Spec 1).
//!
//! Spec 1 added a WatchpointSet lookup on every property-addition store in the
//! IC fast path. This bench measures the cost of a tight property-addition loop
//! so that Spec 1's overhead can be quantified relative to the pre-Spec-1
//! baseline.
//!
//! **Baseline comparison deferred**: The baseline (pre-Spec-1 commit `58d8dd57`)
//! has not been captured in this commit. To perform the comparison, check out
//! `58d8dd57`, run `cargo bench -p lyng-vm --bench property_addition`, save the
//! criterion output (or the generated `target/criterion/` HTML), then return to
//! HEAD and re-run. Criterion's `--save-baseline` / `--baseline` flags can
//! automate this comparison.
//!
//! **What the bench covers**: Each iteration allocates a fresh ordinary object
//! (root shape, no properties) and adds 5 named properties through
//! `Agent::define_own_property`. This is the same code path exercised by the
//! IC fast path: shape lookup → transition → watchpoint-set probe →
//! store-to-slot. The 5-property count keeps the object firmly in shape-stable
//! mode (well below the 128-property dictionary limit) so we measure the pure
//! transition + watchpoint-probe overhead without any dictionary-transition
//! branch.

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use lyng_common::AtomId;
use lyng_env::Runtime;
use lyng_gc::AllocationLifetime;
use lyng_host::NoopHostHooks;
use lyng_objects::{NoopAdaptiveProtoLoadDispatch, ObjectAllocation};
use lyng_types::{PropertyDescriptor, PropertyKey, Value};

/// Number of properties to add to each object per iteration.
/// Must be << 128 (the dictionary-transition limit) to stay in shape-stable mode.
const PROPS_PER_OBJ: u32 = 5;

/// Number of objects allocated+populated per benchmark iteration.
const OBJS_PER_ITER: u32 = 10_000;

/// Atom IDs used as property keys.  We use atoms 100..=104 (well clear of
/// atom 1 used by helper fixtures in tests) so the raw IDs are stable across
/// any future atom-table changes in test helpers.
const ATOM_BASE: u32 = 100;

fn bench_property_addition(c: &mut Criterion) {
    let mut group = c.benchmark_group("property_addition");

    group.bench_function(
        BenchmarkId::new(
            "shape_stable",
            format!("{PROPS_PER_OBJ}_props_x{OBJS_PER_ITER}_objs"),
        ),
        |b| {
            b.iter_batched(
                // ── Setup ────────────────────────────────────────────────────
                // Spin up a fresh Runtime for each measurement batch.  A single
                // Runtime can handle many alloc+define cycles before the heap
                // pressure becomes a variable, but criterion's SmallInput
                // batching keeps the setup cost amortised.
                || Runtime::new(NoopHostHooks),
                // ── Measured body ────────────────────────────────────────────
                |mut runtime| {
                    let agent = runtime.root_agent_mut();

                    for _ in 0..OBJS_PER_ITER {
                        // Allocate a fresh ordinary object on the root shape.
                        let obj = agent.with_heap_and_objects(|heap, objects| {
                            let root = objects.root_shape(
                                &mut heap.mutator(),
                                None,
                                AllocationLifetime::LongLived,
                            );
                            objects.alloc_object(
                                &mut heap.mutator(),
                                ObjectAllocation::ordinary(root),
                                AllocationLifetime::LongLived,
                            )
                        });

                        // Add PROPS_PER_OBJ properties sequentially.
                        // Each call exercises: shape probe → watchpoint-set
                        // lookup → shape transition → slot write.
                        for i in 0..PROPS_PER_OBJ {
                            let key = PropertyKey::from_atom(AtomId::from_raw(ATOM_BASE + i));
                            let mut desc = PropertyDescriptor::new();
                            desc.set_value(Value::from_smi(i as i32));
                            desc.set_writable(true);
                            desc.set_enumerable(true);
                            desc.set_configurable(true);
                            agent
                                .define_own_property(
                                    obj,
                                    key,
                                    desc,
                                    AllocationLifetime::LongLived,
                                    &mut NoopAdaptiveProtoLoadDispatch,
                                )
                                .expect("property addition should not fail in bench");
                        }
                    }
                },
                BatchSize::SmallInput,
            );
        },
    );

    group.finish();
}

criterion_group!(benches, bench_property_addition);
criterion_main!(benches);
