# Design: Shape transitions on prototype mutation + per-shape WatchpointSet primitive

**Date:** 2026-05-25
**Status:** Design draft; awaiting user review.
**Spec relationship:** Foundational. Spec 1 of 2 for the IC migration to a JSC LLInt-style architecture. Spec 2 (the IC migration proper — MetadataTable layout, polymorphic out-of-line, epoch removal, watchpoint adoption in the IC slow path) will consume the primitives landed here.
**Memory pointer:** `~/.claude/projects/-Users-sondre-dev-lyng/memory/project_feedback_jsc_migration.md`.
**JSC references:**
- `bytecode/MetadataTable.h`, `bytecode/UnlinkedMetadataTable.h` (target IC storage shape for Spec 2)
- `bytecode/LLIntPrototypeLoadAdaptiveStructureWatchpoint.h/.cpp` (analog for Spec 2's adaptive variant)
- `llint/LLIntSlowPaths.cpp:845-900` (`setupGetByIdPrototypeCache` — analog install path)
- `runtime/JSObject.cpp:2104-2118` (`JSObject::setPrototypeDirect` — fire-after-transition ordering, see §5.3)
- `bytecode/Watchpoint.cpp:124-183` (`WatchpointSet::fireAll` — reentrancy semantics, see §8)

---

## 1. Goal, scope, exit criteria

### Goal

Establish two new engine-wide invariants and one new primitive that Spec 2 will consume. After Spec 1, `Object.setPrototypeOf` is a shape-changing operation, every shape transition fires watchpoints registered on the source shape, and a `WatchpointSet` primitive exists with one validated production consumer (the dictionary-transition path).

### New invariants

1. **Proto mutation transitions the shape.** `Object.setPrototypeOf(obj, newProto)` always transitions `obj` to a fresh `ShapeId` derived from `(oldShapeId, identityOf(newProto))`. The pre-mutation shape stays valid for other objects that have not been mutated.
2. **Every shape transition fires watchpoints.** Proto mutation, property addition transitions (existing `ShapeTransitionStorage` path), and dictionary transition all call `fire_watchpoints_for_shape(old_shape, agent)` after the object's shape pointer has been updated to the new shape.

### New primitive

`WatchpointSet` stored in a side-table keyed by `ShapeId` on `ObjectRuntime`. Lazy: shapes that never have a registration consume zero memory in the side-table. Registered by slow paths that want notification when a specific shape transitions; consumed (in Spec 1) only by test-only `Recording` watchpoints.

### In scope

- `WatchpointSet` and `Watchpoint` types in a new `crates/objects/src/watchpoint.rs`.
- `ObjectRuntime::watchpoint_sets: HashMap<ShapeId, WatchpointSet>` + accessors.
- `PrototypeTransitionStorage` on `ShapeMetadata`, `Option<Box<HashMap<PrototypeKey, ShapeId>>>`.
- `shapes_with_proto_transitions: HashSet<ShapeId>` on `ObjectRuntime` for GC weak-sweep registration.
- `ordinary_set_prototype_of` rewritten to allocate-or-lookup the post-mutation shape and fire watchpoints.
- `ensure_named_property_dictionary` augmented to fire watchpoints on the source shape.
- Property-addition transition path augmented to fire watchpoints on the source shape.
- Post-mark sweep integration: prune dead-prototype entries from `PrototypeTransitionStorage`; drop `Invalidated` `WatchpointSet` entries.
- Test suite per §10.
- Microbenchmark for the new property-addition fire-call cost; regression ceiling 3%.

### Not in scope (Spec 2 boundary)

- IC-level adoption of watchpoints. ICs continue to use the existing epoch-based invalidation (`PropertyCacheDependency::invalidation_epoch` checks at `crates/objects/src/internal_methods/property_cache.rs:485-490`).
- The per-object epoch bump in `ordinary_set_prototype_of`, `redefine_named_property`, `delete_named_property`, and `ensure_named_property_dictionary` stays in place. Removed in Spec 2.
- `AdaptiveProtoLoad` watchpoint variant carrying a `CodeRef` and `FeedbackSlotId`. Spec 2.
- MetadataTable layout, polymorphic out-of-line, `vm/feedback.rs` rewrite. All Spec 2.
- Property-addition-transition fast-path optimization (`shape_has_any_watchpoint_ever` skip flag). Promoted to Spec 1 only if the §10 perf regression guard exceeds the 3% ceiling.

### Exit criteria

- All existing tests pass, including `crates/vm/src/tests/inline_caches.rs` (the IC suite) and `crates/objects/src/tests.rs:2133-2204` (existing invalidation test family).
- New tests in §10 pass.
- The property-addition microbench shows ≤3% regression vs the pre-Spec-1 baseline. If above 3%, the optimization in §11.1 is in scope and lands in the same PR.

---

## 2. Architecture overview

### Component map

| File | New / existing | Change |
|---|---|---|
| `crates/objects/src/watchpoint.rs` | NEW | `WatchpointSet`, `Watchpoint` enum, `ShapeInvalidationObserver`, state machine, registration & fire API. |
| `crates/objects/src/runtime.rs` | existing | Add `watchpoint_sets: HashMap<ShapeId, WatchpointSet>` and `shapes_with_proto_transitions: HashSet<ShapeId>`. Add `watchpoint_set_mut`, `fire_watchpoints_for_shape`, `sweep_invalidated_watchpoint_sets`, `prune_dead_prototype_transitions`. |
| `crates/objects/src/object_metadata.rs` | existing | Add `prototype_transitions: Option<Box<HashMap<PrototypeKey, ShapeId>>>` to `ShapeMetadata`. Add `PrototypeKey` and `PrototypeTransitionStorage` helpers. |
| `crates/objects/src/shapes.rs` | existing | New `allocate_proto_transitioned_shape(from: ShapeId, key: PrototypeKey) -> ShapeId`. |
| `crates/objects/src/internal_methods.rs` | existing | Rewrite `ordinary_set_prototype_of` per §5.3. |
| `crates/objects/src/internal_methods/named_properties.rs` | existing | Insert `fire_watchpoints_for_shape` call into `ensure_named_property_dictionary` per §6. |
| Property-addition transition path | existing | Same insertion at the existing transition site in `object_metadata.rs` — the function that records a new child shape under `ShapeTransitionStorage` after a property addition (recon located this around lines 338-454). The exact function name is identified during PR 4 implementation; the design commitment is "wherever the property-add transition is finalized on an existing parent shape, fire watchpoints on that parent shape after the child is recorded." |
| `crates/gc/src/arena/weak_state.rs` (or equivalent) | existing | Add `prune_dead_prototype_transitions` and `sweep_invalidated_watchpoint_sets` to the existing post-mark sweep pass. |
| `crates/objects/src/tests.rs` | existing | Extend test family per §10. |
| `crates/vm/src/tests/inline_caches.rs` | existing | Update `named_property_load_ic_invalidates_proto_cache_on_prototype_swap` assertion to be mechanism-agnostic. |
| `crates/gc/src/tests/` (path TBC) | existing | New sweep tests per §10. |

### Backward-compatibility stance

Spec 1 is **purely additive at the IC layer**. ICs continue to use epoch-based invalidation; the per-object epoch is still bumped at the same callsites. Behavioral changes are confined to:

1. `ShapeId` values are no longer stable across `Object.setPrototypeOf`. Tooling that caches shape IDs externally (debugger, profilers, snapshot tools) must accept that the shape may differ after a proto swap.
2. Watchpoints can fire on transitions; in Spec 1 only the test-only `Recording` consumer registers, so production code observes no fires.

The new invariant ("every shape transition fires watchpoints") imposes one HashMap lookup per shape transition site. The dominant cost site is property addition; the §10 perf regression guard verifies this stays under 3%.

---

## 3. `WatchpointSet` primitive

### Type

```rust
// crates/objects/src/watchpoint.rs

pub struct WatchpointSet {
    state: WatchpointState,
    watchpoints: Vec<Watchpoint>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatchpointState {
    Cleared,
    Watched,
    Invalidated,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Invalidated;
```

### State machine

```
Cleared ──register──> Watched ──fire_all──> Invalidated (terminal)
   │                                              ▲
   │                                              │
   └──── fire_all is a no-op when Cleared ────────┘
```

`Invalidated` is terminal per-`WatchpointSet`-instance. Subsequent `register` calls on an `Invalidated` set return `Err(Invalidated)`. Rationale: a fired set means the assumption its watchpoints guarded is no longer true; silently re-arming would let stale assumptions sneak back. Callers needing to keep watching must register on the post-transition shape's set (a different `ShapeId`, a different `WatchpointSet`).

### API

```rust
impl WatchpointSet {
    pub fn new() -> Self;                          // → Cleared, empty
    pub fn state(&self) -> WatchpointState;
    pub fn is_invalidated(&self) -> bool;
    pub fn register(&mut self, wp: Watchpoint) -> Result<(), Invalidated>;
    pub fn fire_all(&mut self, agent: &mut Agent);  // Cleared/Watched → Invalidated
    pub fn visit_roots(&self, visitor: &mut Visitor);
}
```

### Fire semantics: drain-then-dispatch

`fire_all` does the following in order:

1. If `state == Invalidated`, return immediately.
2. Set `state = Invalidated`.
3. `let fired = std::mem::take(&mut self.watchpoints)`.
4. For each `wp in fired`: `wp.fire(agent)`.

Drain-then-dispatch matters because a fire callback may register a *new* watchpoint on a *different* shape. That new set must not see the about-to-fire watchpoints from the old one. The fire callback also cannot accidentally observe the set in an inconsistent half-empty state.

Reentrancy on the *same* shape's set during fire: returns `Err(Invalidated)` because the state has already flipped. This is the correct behavior — see §8.1.

### Side-table on `ObjectRuntime`

```rust
// crates/objects/src/runtime.rs (additive)
pub struct ObjectRuntime {
    // ...existing fields...
    watchpoint_sets: HashMap<ShapeId, WatchpointSet>,
    shapes_with_proto_transitions: HashSet<ShapeId>,
}

impl ObjectRuntime {
    pub fn watchpoint_set_mut(&mut self, shape: ShapeId) -> &mut WatchpointSet {
        self.watchpoint_sets.entry(shape).or_insert_with(WatchpointSet::new)
    }

    pub fn fire_watchpoints_for_shape(&mut self, shape: ShapeId, agent: &mut Agent) {
        if let Some(set) = self.watchpoint_sets.get_mut(&shape) {
            set.fire_all(agent);
        }
    }

    pub fn sweep_invalidated_watchpoint_sets(&mut self) {
        self.watchpoint_sets.retain(|_, set| !set.is_invalidated());
    }
}
```

`fire_watchpoints_for_shape` leaves the entry in `Invalidated` state in the HashMap; `sweep_invalidated_watchpoint_sets` runs in the GC post-mark sweep pass and removes them. Subsequent `register` attempts between the fire and the sweep correctly return `Err(Invalidated)` from the still-present `Invalidated` entry.

### Common-case cost

Most shape transitions land on the no-watchpoint path: a single `HashMap::get_mut(&shape_id)` returning `None`. One hash + one slot probe. Allocations and callback dispatch occur only when a watchpoint has actually been registered for that specific shape.

If §10's microbench shows this cost exceeds the 3% ceiling on the property-addition path, §11.1 promotes a quick-skip flag into Spec 1.

---

## 4. `Watchpoint` enum

### Type

```rust
// crates/objects/src/watchpoint.rs

pub enum Watchpoint {
    /// Notifies an observer that a specific shape has transitioned.
    /// Spec 1's only variant. Spec 2 will add `AdaptiveProtoLoad`.
    ShapeInvalidation {
        observer: ShapeInvalidationObserver,
    },
}

pub enum ShapeInvalidationObserver {
    /// Test-only consumer. Records the fire event into a side buffer so
    /// unit tests can assert "this transition fired exactly the watchpoints
    /// I registered." Carries no heap roots.
    Recording { token: u64 },
}
```

### Dispatch

```rust
impl Watchpoint {
    fn fire(self, agent: &mut Agent) {
        match self {
            Watchpoint::ShapeInvalidation { observer } => observer.fire(agent),
        }
    }

    fn visit_roots(&self, visitor: &mut Visitor) {
        match self {
            Watchpoint::ShapeInvalidation { observer } => observer.visit_roots(visitor),
        }
    }

    fn is_alive(&self, heap: &HeapView) -> bool {
        match self {
            Watchpoint::ShapeInvalidation { observer } => observer.is_alive(heap),
        }
    }
}
```

`Recording`'s `fire` pushes `token` to `ObjectRuntime::recording_watchpoint_fires` (a `#[cfg(test)] Vec<u64>`). `Recording`'s `visit_roots` is a no-op; `is_alive` returns `true` unconditionally.

### Design rationale: enum over trait object

- Closed set, known at compile time, one crate. Pattern-match dispatch is one indirection; trait-object vtable dispatch is one indirection. No performance difference.
- `Vec<Watchpoint>` is contiguous storage; `Vec<Box<dyn Watchpoint>>` would heap-allocate per registration.
- Adding a new variant in Spec 2 forces exhaustive `match` compilation errors at every dispatch site — useful for ensuring the new variant is wired through `fire` / `visit_roots` / `is_alive`.

### Recording buffer location

```rust
// In ObjectRuntime, alongside watchpoint_sets:
#[cfg(test)]
recording_watchpoint_fires: Vec<u64>,
```

`cfg(test)` so production builds carry zero overhead. Test helpers expose `take_recording_fires(&mut self) -> Vec<u64>`.

---

## 5. Shape transition for `Object.setPrototypeOf`

### 5.1 `PrototypeKey` and the transition table

```rust
// crates/objects/src/object_metadata.rs (additive)

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrototypeKey {
    Object(ObjectRef),
    Null,
}

impl PrototypeKey {
    pub fn from_optional(proto: Option<ObjectRef>) -> Self {
        match proto {
            Some(obj) => Self::Object(obj),
            None => Self::Null,
        }
    }
}

pub struct ShapeMetadata {
    // ...existing fields...
    prototype_transitions: Option<Box<HashMap<PrototypeKey, ShapeId>>>,
}
```

**Storage rationale.** `Option<Box<HashMap<...>>>` costs 8 bytes per shape in the no-transition case (the >99% case for typical JS, where most classes never get their prototype mutated). One heap allocation is paid on the first proto transition for a given source shape. Inline-1 (or larger) alternatives were considered; `inline-1` + `Option<HashMap>` costs ~32B per shape, which is dead weight for the vast majority of shapes. See the brainstorming transcript for the full memory comparison.

### 5.2 `resolve_prototype_transition`

```rust
impl ObjectRuntimeMut {
    pub fn resolve_prototype_transition(
        &mut self,
        from_shape: ShapeId,
        key: PrototypeKey,
    ) -> ShapeId {
        // Fast path: existing transition.
        if let Some(target) = self.shape_metadata(from_shape)
            .prototype_transitions
            .as_deref()
            .and_then(|table| table.get(&key).copied())
        {
            return target;
        }

        // Slow path: allocate a fresh shape derived from `from_shape` with
        // `key` as prototype. Reuses the existing shape-allocation machinery
        // (`allocate_proto_transitioned_shape`) — the new shape inherits
        // property layout (slots, property table) from `from_shape`; only
        // the prototype guard changes.
        let new_shape = self.allocate_proto_transitioned_shape(from_shape, key);
        self.shape_metadata_mut(from_shape)
            .prototype_transitions
            .get_or_insert_with(|| Box::new(HashMap::new()))
            .insert(key, new_shape);
        self.shapes_with_proto_transitions.insert(from_shape);
        new_shape
    }
}
```

### 5.3 `ordinary_set_prototype_of` rewrite

```rust
// crates/objects/src/internal_methods.rs (rewrite)
pub fn ordinary_set_prototype_of(
    agent: &mut Agent,
    object: ObjectRef,
    new_proto: Option<ObjectRef>,
) -> Result<bool, /* existing error type */> {
    let old_shape = agent.objects().object_header(...).shape();
    let key = PrototypeKey::from_optional(new_proto);

    // 1. Compute new shape (table lookup → fresh allocate on miss).
    let new_shape = agent.objects_mut().resolve_prototype_transition(old_shape, key);

    // 2. Commit the transition on the object.
    agent.objects_mut().retarget_shape(object, new_shape);
    agent.objects_mut().write_prototype_slot(object, new_proto);

    // 3. Fire watchpoints on the OLD shape, AFTER the object's shape pointer
    //    has been updated. Callbacks see the object already in its new shape.
    //    Matches JSC's `JSObject::setPrototypeDirect` ordering
    //    (`runtime/JSObject.cpp:2104-2118`).
    agent.objects_mut().fire_watchpoints_for_shape(old_shape, agent);

    // 4. Spec 1 retains the existing epoch bump. Spec 2 removes this when
    //    ICs adopt watchpoints.
    bump_invalidation(heap, object, InvalidationCause::PrototypeMutation);

    Ok(true)
}
```

### 5.4 What the new shape inherits from the old

- Same `slot_count`, `property_count`, `property` chain, `parent`.
- Same `uses_flat_lookup` setting.
- **Different `prototype_guard`:** points at `new_proto` (or `None`).
- **Different `id`:** fresh `ShapeId` from the existing allocator.

The existing property-addition transition tree on the old shape is unchanged. A property added later on the new shape produces a different `ShapeId` than one added on the old shape — correct, because the resulting shapes have different prototypes.

### 5.5 Shape blowup analysis

In typical JS, `setPrototypeOf` is one-off (class-wiring time). One entry per (oldShape, proto) pair. Pathological case: a loop calling `setPrototypeOf` with thousands of different protos on instances sharing a source shape would cause linear shape growth — same blowup profile as `Object.assign` with many distinct keys on instances sharing a base shape. Acceptable.

---

## 6. Dictionary-transition consumer (validating use case)

The existing dictionary-transition path at `crates/objects/src/internal_methods/named_properties.rs:158-174` (`ensure_named_property_dictionary`) already clears the shape — it is already a shape-changing event. Spec 1 adds one line: `fire_watchpoints_for_shape(old_shape, agent)` after the shape clearing, before the existing `bump_invalidation`.

```rust
// crates/objects/src/internal_methods/named_properties.rs (modified)
pub(crate) fn ensure_named_property_dictionary(
    agent: &mut Agent,
    object: ObjectRef,
) -> ... {
    // ...existing path that detects "already dictionary" and early-returns...

    let old_shape = agent.objects().object_header(...).shape();

    // 1. Existing logic: clear shape, transition to dictionary storage mode.
    agent.objects_mut().transition_to_dictionary_mode(object);

    // 2. NEW: fire watchpoints registered on the old shape.
    agent.objects_mut().fire_watchpoints_for_shape(old_shape, agent);

    // 3. Existing: bump epoch (Spec 1 retains).
    bump_invalidation(heap, object, InvalidationCause::DictionaryTransition);
}
```

Callers (property redefinition, property deletion, property-addition-past-128) route through this entry point, so this single insertion covers all three triggers.

The property-addition transition path (existing `ShapeTransitionStorage` path) takes the same `fire_watchpoints_for_shape(old_shape, agent)` insertion at its transition site in `object_metadata.rs` (the function that records the new child shape under `ShapeTransitionStorage`, around lines 338-454 per the recon). It is part of the §1 in-scope work and lands in PR 4 of §9.

### Why this is sufficient to validate the primitive

The dictionary path exercises every production code path of `WatchpointSet`:

- **Registration:** `Recording` watchpoints register before the transition.
- **Fire dispatch:** the transition runs `fire_watchpoints_for_shape`, lookup hits, `fire_all` runs, drain-then-dispatch executes each watchpoint's callback.
- **State machine:** `Watched → Invalidated`. Second registration attempt returns `Err(Invalidated)`.
- **Side-table sweep:** entry is left in `Invalidated`; `sweep_invalidated_watchpoint_sets` drops it next GC.
- **GC visitation:** `Recording` holds no roots, so `visit_roots` walks an empty set. This intentionally keeps Spec 1's GC surface clean — root-handling stress is a Spec 2 concern when `AdaptiveProtoLoad` arrives with a `CodeRef`.

### Acknowledged Spec 2 gaps

- The "callback re-registers on the post-transition shape" reentrancy pattern. Spec 2's `AdaptiveProtoLoad` does this; covered there.
- GC pinning of fire targets via watchpoint roots. Same reason.

These gaps are bounded to the new watchpoint *kind* added in Spec 2, not to the primitive itself.

---

## 7. Data flow

### 7.1 Three transition sites that fire watchpoints

| Site | File | Hotness |
|---|---|---|
| `ordinary_set_prototype_of` | `internal_methods.rs` | Cold |
| `ensure_named_property_dictionary` | `internal_methods/named_properties.rs` | Cold |
| Property-addition shape transition | `object_metadata.rs` | **Hot** — per property addition on a non-dictionary object |

The third site is the only one with hot-path cost. The §10 perf guard targets it specifically.

### 7.2 End-to-end: dictionary-transition fire

```
test setup:
  1. Allocate object O₁ with shape S.
  2. agent.objects_mut().watchpoint_set_mut(S).register(
       Watchpoint::ShapeInvalidation { observer: Recording { token: 42 } }
     )?;                                           // S's set: Cleared → Watched
  3. Assert state(S) == Watched.

trigger:
  4. Object.defineProperty(O₁, "x", { configurable: false, value: 1 });
     // Path: redefine_named_property → ensure_named_property_dictionary
       a. old_shape = S (read from O₁'s header)
       b. transition_to_dictionary_mode(O₁)        // O₁'s shape pointer flips
       c. fire_watchpoints_for_shape(S, agent)
          ├─ HashMap::get_mut(&S)                  // hit
          ├─ WatchpointSet::fire_all(agent)
          │   ├─ state: Watched → Invalidated
          │   ├─ drain watchpoints into local Vec
          │   └─ for each: observer.fire(agent)
          │       └─ Recording { token: 42 } pushes 42 to recording_watchpoint_fires
          └─ entry left in HashMap, state Invalidated
       d. bump_invalidation(InvalidationCause::DictionaryTransition)

assertion:
  5. recording_watchpoint_fires.take() == [42]
  6. state(S) == Invalidated
  7. Re-register on S: Err(Invalidated).

later (next GC):
  8. sweep_invalidated_watchpoint_sets() removes the entry for S.
```

### 7.3 End-to-end: proto-mutation fire

```
test setup:
  1. Allocate O₁ with shape S₁, prototype P₁.
  2. Register Recording { token: 7 } on S₁.

trigger:
  3. Object.setPrototypeOf(O₁, P₂):
     a. old_shape = S₁
     b. key = PrototypeKey::Object(P₂)
     c. S₂ = resolve_prototype_transition(S₁, key)  // table miss → allocate
     d. retarget_shape(O₁, S₂); write_prototype_slot(O₁, P₂)
     e. fire_watchpoints_for_shape(S₁, agent)
     f. bump_invalidation(PrototypeMutation)

assertion:
  4. recording_watchpoint_fires == [7]
  5. object_header(O₁).shape() == S₂  (≠ S₁)
  6. state(S₁) == Invalidated; state(S₂) == Cleared
```

### 7.4 Common-case miss

A property addition on a shape with no registered watchpoints: `HashMap::get_mut(&S)` returns `None`, no allocation, no dispatch. This is the dominant case and the regression-guarded hot path.

---

## 8. Error handling and edge cases

### 8.1 Reentrancy: callback registers during fire

Not exercised in Spec 1 (`Recording` does not re-register). Primitive must support it for Spec 2's `AdaptiveProtoLoad`. The drain-then-dispatch ordering in §3 handles it:

- By callback time, the fired set's `watchpoints: Vec` is empty and `state == Invalidated`.
- If the callback calls `watchpoint_set_mut(other_shape).register(new_wp)`, it lands on a *different* `WatchpointSet` entry. No aliasing.
- If the callback tries to register back on the *same* shape, `Err(Invalidated)`. The JSC adaptive pattern always registers on the *new* shape (the one the object transitioned to), which is a different `ShapeId`.

### 8.2 GC during fire

Spec 1's `Recording` variant does not allocate. Fire callbacks cannot trigger GC. Spec 2's `AdaptiveProtoLoad` will need a `DeferGC` analog around `fire_all`; the safepoint poll machinery (`dsl_poll_pending` at `crates/vm/src/vm.rs:182`) is the existing hook.

### 8.3 Shape transition with no registration

`fire_watchpoints_for_shape(S, agent)` → `HashMap::get_mut(&S)` → `None` → return. Single hash lookup, no allocation. The dominant case.

### 8.4 Double registration after invalidation

`register` on an `Invalidated` set returns `Err(Invalidated)`. No panic, no log. Spec 2 callers must handle this by retargeting to the new shape's set.

### 8.5 Slot recycling for GC'd prototype objects

Handled by the existing weak-state sweep pattern at `crates/gc/src/arena/weak_state.rs:202`. The new `shapes_with_proto_transitions: HashSet<ShapeId>` registers Spec 1's table with the same pass:

1. Mark phase completes.
2. Existing weak-state sweep runs (`weak_maps.retain(...)`, etc.).
3. **NEW:** `prune_dead_prototype_transitions(objects: &ObjectMarker)`:
   - For each `ShapeId` in `shapes_with_proto_transitions`:
     - Walk that shape's `PrototypeTransitionStorage`.
     - Drop entries where `PrototypeKey::Object(ref)` is unmarked.
     - If the table is now empty, set `prototype_transitions = None`.
   - Remove `ShapeId`s from `shapes_with_proto_transitions` whose table is now empty.
4. **NEW:** `sweep_invalidated_watchpoint_sets()` runs alongside.
5. Sweep phase reclaims slots.

The set is bounded by the number of shapes that have *ever* received a proto transition (typically a few per class hierarchy). Sweep cost is negligible.

### 8.6 Watchpoint leak (un-fired Watched set with dead owner)

Spec 2 concern, flagged here. Spec 1's `Recording` is test-only and torn down at test end. Spec 2's `AdaptiveProtoLoad` will hold a weak `CodeRef`; needs a cleanup path that drops watchpoints whose owner is dead even on un-fired sets. Hook: extend the post-mark sweep to walk all `Watched` sets and drop dead-owner watchpoints; if the set becomes empty, drop the entry.

### 8.7 Concurrent access

The VM is single-threaded. `Vm`, `Agent`, `ObjectRuntime` are not `Sync`. No locks. Documented in the `watchpoint.rs` module-level comment.

### 8.8 Recursive shape transition during fire

A callback that triggers another shape transition (e.g., property assignment that adds a property elsewhere): the recursive `fire_watchpoints_for_shape` lands on a *different* shape's set, different HashMap entry. Single-threaded, no aliasing. Correct.

---

## 9. Implementation order

Approach 1 from the brainstorm: bottom-up, validate the primitive before depending on it.

1. **Primitive only.** Land `WatchpointSet`, `Watchpoint`, `ShapeInvalidationObserver::Recording`, the side-table on `ObjectRuntime`, and unit tests (T1–T9 in §10). No callers in production code. PR 1.
2. **Dictionary consumer.** Wire `fire_watchpoints_for_shape` into `ensure_named_property_dictionary`. Land T10–T12 + T20–T21 (IC regression). Epochs still bump in parallel. PR 2.
3. **Proto transition.** Land `PrototypeKey`, `PrototypeTransitionStorage`, `resolve_prototype_transition`, `allocate_proto_transitioned_shape`. Rewrite `ordinary_set_prototype_of`. Update affected tests (T13–T16, T22). Add `shapes_with_proto_transitions` and the GC sweep extension (T18–T19). PR 3.
4. **Property-addition fire site.** Insert the third `fire_watchpoints_for_shape` callsite. Land T17 and the §10 perf benchmark. If the benchmark exceeds the 3% ceiling, also land §11.1's optimization. PR 4.

Each PR is independently testable. The `cargo test` suite stays green at every step, including the full IC regression suite (T20–T21) at every PR boundary — not just after PR 2. Spec 2 begins after PR 4 merges.

---

## 10. Testing strategy

### 10.1 Test matrix

| # | Layer | What it proves |
|---|---|---|
| T1 | `WatchpointSet` unit | `new()` → `Cleared`; `register` → `Watched`; `fire_all` → `Invalidated`. |
| T2 | `WatchpointSet` unit | `register` on `Invalidated` returns `Err(Invalidated)`. |
| T3 | `WatchpointSet` unit | `fire_all` on `Cleared` is a no-op (no panic, no state change). |
| T4 | `WatchpointSet` unit | Reentrancy: a `Recording` callback that calls `register` on a different set succeeds; the originating set stays empty post-fire. |
| T5 | `WatchpointSet` unit | `fire_all` dispatches in registration order. |
| T6 | `Watchpoint` unit | `Recording { token }.fire(agent)` pushes `token` to `recording_watchpoint_fires`. |
| T7 | `ObjectRuntime` unit | `watchpoint_set_mut(S)` is lazy: no entry for unregistered S. |
| T8 | `ObjectRuntime` unit | `sweep_invalidated_watchpoint_sets` drops `Invalidated`; retains `Watched` and `Cleared`. |
| T9 | `ObjectRuntime` unit | `fire_watchpoints_for_shape` on a shape with no entry is a no-op. |
| T10 | Dictionary consumer | Register `Recording {7}` on S → `Object.defineProperty(obj, "x", { configurable: false, ... })` → fire observed, `obj.shape() != S`, `state(S) == Invalidated`. |
| T11 | Dictionary consumer | Same as T10 via `delete obj.x`. |
| T12 | Dictionary consumer | Same as T10 via 128 property additions forcing dictionary mode. |
| T13 | Proto transition | `setPrototypeOf(obj, P)` allocates a fresh `ShapeId`; `obj.shape() != original`. |
| T14 | Proto transition | Two `setPrototypeOf` calls with the same `P` on two objects sharing the source shape produce the same destination `ShapeId` (transition table dedup). |
| T15 | Proto transition | `setPrototypeOf(obj, null)` produces a distinct shape from `setPrototypeOf(obj, someP)`. |
| T16 | Proto transition | Fire-after-transition ordering: register `Recording {9}` on S₁ → `setPrototypeOf` an S₁ object to P → callback runs while `obj.shape() == S₂`. |
| T17 | Property addition | Register `Recording` on S → add a property to an S object → fire observed (validates the property-add callsite). |
| T18 | GC sweep | Allocate prototype P, register `(S, P) → S'`, drop P's other roots, GC → `(S, P)` entry pruned. |
| T19 | GC sweep | Same setup but P remains rooted → entry retained. |
| T20 | IC regression | `named_property_load_ic_invalidates_proto_cache_on_prototype_swap` still passes (now via shape change *and* epoch bump). |
| T21 | IC regression | Full `crates/vm/src/tests/inline_caches.rs` suite passes. |
| T22 | Existing test update | `redefine_delete_and_prototype_mutation_bump_invalidation_epochs` augmented to also assert post-proto-mutation `shape != original`. |

### 10.2 Coverage targets

- `WatchpointSet`: 100% line coverage.
- `ordinary_set_prototype_of`: all three sub-paths (existing prototype unchanged, new prototype is a value, new prototype is null).
- GC sweep integration: both presence and absence cases.

### 10.3 Performance regression guard

A microbenchmark in `crates/vm/benches/` (exact path to be confirmed during implementation; if no equivalent exists, add one) exercises:

```js
function tight_property_addition() {
    for (let i = 0; i < 100000; i++) {
        const o = {};
        o.a = 1; o.b = 2; o.c = 3; o.d = 4; o.e = 5;
    }
}
```

Run before and after PR 4. Ceiling: 3% wall-clock regression. If exceeded, §11.1's optimization promotes into PR 4 scope.

### 10.4 What's deliberately NOT tested in Spec 1

- "Watchpoint callback re-registers on the post-transition shape" (Spec 2's `AdaptiveProtoLoad`).
- Watchpoint callbacks that allocate / trigger GC (Spec 2's `AdaptiveProtoLoad`).
- IC fast-path consumption of watchpoints (Spec 2).
- Concurrent register/fire (VM is single-threaded).

---

## 11. Conditional in-scope: hot-path optimization

### 11.1 `shape_has_any_watchpoint_ever` flag

If §10.3's microbench exceeds the 3% ceiling, this optimization moves into Spec 1 (PR 4):

Add `has_any_watchpoint_ever: bool` to `ShapeMetadata`, set to `true` the first time anything calls `watchpoint_set_mut(self_id)`. Never reset (even after `Invalidated`).

`fire_watchpoints_for_shape` checks this flag *before* the HashMap lookup:

```rust
pub fn fire_watchpoints_for_shape(&mut self, shape: ShapeId, agent: &mut Agent) {
    if !self.shape_metadata(shape).has_any_watchpoint_ever {
        return;  // O(1) bool check, no hash, no probe
    }
    if let Some(set) = self.watchpoint_sets.get_mut(&shape) {
        set.fire_all(agent);
    }
}
```

For the >99% of shapes that never see a registration, the cost drops to a single boolean check.

If the microbench stays under 3% without this, the optimization defers to Spec 2.

---

## 12. References

- Brainstorm transcript / decision log: prior assistant conversation, stored in conversation history.
- Spec 1 / Spec 2 split rationale: §1 of the in-memory project plan.
- JSC analog file pointers: header of this document.
- Lyng files referenced throughout: see component table in §2.
