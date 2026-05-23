# Lyng JS GC

This note documents the collector ordering that matters to guest-visible weak
references, finalization registries, and concurrent major sweep.

## Major Collection Order

Major collection uses the following order:

1. Bounded major-mark slices run at allocation safepoints.
2. The atomic mark finish drains all remaining strong mark work.
3. Weak structures are traced to a fixpoint. Live `WeakMap` entries can keep
   values alive, and live `FinalizationRegistry` holdings are traced before
   death is observed.
4. The collector asserts that no gray mark work remains.
5. Weak state is resolved on the agent thread.
6. Unmarked slot handles are handed to the background sweep boundary.
7. The background worker returns a sweep plan.
8. The agent thread applies the plan and only then publishes reclaimed slots
   back to the allocation free lists.

The background sweep worker does not mutate heap pages, side-allocation tables,
weak state, finalization queues, or free lists. Those structures stay
agent-thread-owned. A pending sweep plan must be synchronized before candidate
slots can be reused.

## WeakRef Ordering

`WeakRef` targets are cleared after the mark finish and weak-structure fixpoint,
but before the background sweep plan is applied. A live `WeakRef` object whose
target is unmarked observes `None` as soon as weak state is resolved. The target
record may still occupy its slot until the sweep plan is synchronized, but it is
already unreachable through the `WeakRef` state.

Dead `WeakRef` owner objects lose their weak state during the same weak-state
resolution pass. The background worker never decides whether a weak target is
dead; it only receives handles that were already classified by the completed
mark set.

## FinalizationRegistry Ordering

`FinalizationRegistry` cells are also processed after the mark finish and before
background sweep. For a live registry, each cell whose target is unmarked is
removed from the live cell list, its holdings are moved to the registry pending
holdings queue, and the registry is added to the pending cleanup list when no
cleanup job is already active.

The held value is traced during the weak-structure fixpoint before target death
is observed. That means the holdings remain alive even though the target is a
sweep candidate. The target slot is reclaimed later, when the agent thread
applies the background sweep plan.

`lyng-js-env` enqueues host-visible finalization cleanup jobs after the agent
collection report returns. Job enqueueing observes the pending registry list
created during weak-state resolution, not the background worker. The ordering
guarantee is therefore:

1. weak target death is classified by the completed mark set;
2. weak refs are cleared and finalizer holdings are queued on the agent thread;
3. cleanup jobs are enqueued from the pending registry list;
4. sweep-plan application reclaims candidate slots before those slots can be
   reused by allocation.

This keeps guest-observable weak-reference and finalizer behavior independent
from the exact timing of the background sweep worker.
