# Design: Spec 2 — IC fast-path migration onto watchpoints + JSC-style MetadataTable

**Date:** 2026-05-26
**Status:** Design draft; awaiting user review.
**Spec relationship:** Spec 2 of 2 for the IC→JSC migration. Builds on Spec 1 (`2026-05-25-shape-transitions-and-watchpoints-design.md`), which landed the `WatchpointSet` primitive, made `Object.setPrototypeOf` shape-transitioning, and wired four fire sites (proto mutation, dictionary transition, property redefinition, property deletion, property-addition transition). Spec 2 migrates the IC fast path off the parallel epoch-based invalidation system onto watchpoints, then rebuilds the underlying feedback storage as a JSC-LLInt-style `MetadataTable`.
**Memory pointer:** `~/.claude/projects/-Users-sondre-dev-lyng/memory/project_feedback_jsc_migration.md`.
**JSC references:**
- `bytecode/MetadataTable.h`, `bytecode/UnlinkedMetadataTable.h` (per-CodeBlock variable-width metadata buffer)
- `bytecode/GetByIdMetadata.h`, `bytecode/CallLinkInfo.h`, `bytecode/ValueProfile.h` (per-opcode-kind metadata structs)
- `llint/LowLevelInterpreter.asm:578` (asm metadata-resolve macro, ~5 instructions)
- `bytecode/GetByStatus.h`, `bytecode/CallLinkStatus.h` (on-demand status projections)
- `bytecode/LLIntPrototypeLoadAdaptiveStructureWatchpoint.h/.cpp` (adaptive proto-cache invalidation; Lyng departs from JSC by clearing instead of retargeting)

---

## 1. Goal, scope, exit criteria

### Goal

Spec 2 completes the IC→JSC migration. After Spec 2:

- ICs invalidate via shape-compare + watchpoints. No more `last_invalidation_epoch` field, no more `bump_invalidation` calls.
- Polymorphic chains live in a heap-side stub map keyed by `(code, slot)`.
- Per-code metadata lives in a JSC-style variable-width `MetadataTable` (one buffer per code object, indexed via an offset table by opcode kind).
- The legacy `vm/feedback.rs` IC state machine and `feedback_flat_storage` mirror are deleted; `MetadataTable` is the single source of truth.
- Snapshot APIs (`FeedbackVectorFootprint`, `FeedbackVectorSnapshot`) are deleted; per-kind `Status` types provide on-demand projections (JSC's `GetByStatus` / `CallLinkStatus` analog).

### In scope

- **Phase A — Watchpoint IC adoption + epoch removal + counter lift.** Add `ShapeInvalidationObserver::AdaptiveProtoLoad { code, slot, generation }`. IC slow path registers `AdaptiveProtoLoad` watchpoints on every proto-chain shape at install. IC fast path drops the epoch comparison. `named_epoch` + `named_aux_epoch` removed from `FeedbackEntry`. `bump_invalidation` callsites deleted (proto mutation, redefine, delete, dictionary). `RuntimeObjectRecord::last_invalidation_epoch`, `ObjectRuntime::next_invalidation_epoch`, and the `InvalidationCause` enum deleted. `warmup_counter` lifted from `FeedbackVector` onto `Tiering`. Per-IC `generation: u32` added.
- **Phase B — Polymorphic out-of-line.** Lift `entries[POLY_LIMIT..8]` from `NamedPropertyFeedback` into `Vm::polymorphic_chains: HashMap<(CodeId, FeedbackSlotId), PolymorphicChain>`. Slow-path reads/writes go through the map. The inline `polymorphic_own_data_handlers` sidecar stays (still drives the asm fast path) until Phase C.
- **Phase C — `MetadataTable` per code object.** One contiguous buffer per code with `[LinkingData][Offset16/32 table][per-opcode-kind run]...` layout. Per-kind metadata structs (`PropertyMetadata`, `CallMetadata`, `ArithMetadata`, `ComparisonMetadata`, `KeyedPropertyMetadata`). Asm DSL pin (`x21`) repurposed from the flat-array base to the MetadataTable base; resolve macro grows to ~5 instructions. Dual-write integration with legacy storage during the transition.
- **Phase D — Flip system of record.** Re-home the IC state machine (Uninit→Mono→Poly→Mega transitions) onto per-kind metadata methods. Delete `FeedbackVector`, `NamedPropertyFeedback`, `feedback_flat_storage`, `mirror_flat_slot`, and the dual-write paths.
- **Phase E — Status projections + API drop.** Add `NamedPropertyStatus`, `CallStatus`, `ArithStatus`, `ComparisonStatus`, `KeyedPropertyStatus` + `MetadataTableFootprint` types and `Vm` queries. Update the ~12 test consumers in `crates/vm/src/tests/feedback.rs` + `tests/inline_caches.rs`. Delete `FeedbackVectorSnapshot`, `NamedPropertyFeedbackSnapshot`, `FeedbackSiteSnapshot`, `FeedbackVectorFootprint`.

### Out of scope

- New IC kinds beyond what exists today (load-from-global, super-property access, etc.).
- Watchpoint retargeting (`AdaptiveStructure`-style retarget on fire). Spec 2 stays with clear-on-fire.
- DFG/baseline tiers. Spec 2 is LLInt-tier IC machinery only.
- Cross-function IC sharing (megamorphic stub caches).
- Watchpoint root-pinning for fire targets. `AdaptiveProtoLoad`'s `CodeRef` is held weakly; pinning relies on the code's own GC roots.
- `ObjectRuntime::recording_watchpoint_fires` cleanup (still ungated for test visibility per Spec 1's decision).

### Exit criteria

- All workspace tests pass at every phase boundary.
- `crates/vm/src/tests/inline_caches.rs` (~33 tests) green throughout.
- Microbench: ≤3% wall-clock regression vs. pre-Spec-2 baseline on the property-addition bench from Spec 1 and a new IC-hot-loop bench that mixes property loads, arithmetic, and calls. Measured at each phase boundary.
- `FeedbackEntry`: gone (replaced by per-kind metadata).
- `bump_invalidation` callsites, `last_invalidation_epoch`, `InvalidationCause`, and `next_invalidation_epoch` are absent (grep returns no matches).
- `FeedbackVector`, `feedback_flat_storage`, `mirror_flat_slot`, `FeedbackVectorSnapshot`, `FeedbackVectorFootprint` are absent.
- Cargo clippy + fmt clean.

---

## 2. Architecture overview

### 2.1 Component map

| File / module | Phase | Change |
|---|---|---|
| `crates/objects/src/watchpoint.rs` | A | Add `ShapeInvalidationObserver::AdaptiveProtoLoad { code, slot, generation }` variant. Existing `Watchpoint::ShapeInvalidation` enum stays one-deep; the new observer kind reuses the existing dispatch path. |
| `crates/env/src/agent.rs` | A | Extend `fire_watchpoints_for_shape`'s dispatch loop with the new observer arm: call `Vm::clear_ic_slot_if_generation_matches`. |
| `crates/vm/src/vm.rs` | A | Add `Vm::clear_ic_slot_if_generation_matches`. Move `warmup_counter` from `FeedbackVector` onto `Tiering`. |
| `crates/vm/src/vm/feedback.rs` | A | Slow-path registers `AdaptiveProtoLoad` on each proto-chain shape at install. Per-IC `generation: u32` added. Remove epoch field reads/writes. |
| `crates/vm/src/vm/dispatch/property.rs` | A | Replace `record.last_invalidation_epoch() == cached_epoch` with shape-only compare on both monomorphic and proto fast paths. |
| `crates/vm/src/dsl/feedback_flat.rs` | A | Drop `named_epoch` + `named_aux_epoch`. Add `generation: u32`. Slot stays 64B (asm pin contract unchanged until Phase C). |
| `crates/objects/src/internal_methods.rs` + named_properties.rs | A | Delete `bump_invalidation` calls. Delete `bump_prototype_mutation_epoch` wrapper. |
| `crates/objects/src/shapes.rs` | A | Delete `InvalidationCause` enum. |
| `crates/objects/src/runtime.rs` | A | Delete `next_invalidation_epoch` field. Delete `bump_invalidation` method. |
| `crates/gc/src/arena/records.rs` | A | Delete `last_invalidation_epoch` field from `RuntimeObjectRecord`. |
| `crates/vm/src/vm/feedback/polymorphic.rs` | B (NEW) | `PolymorphicChain` type + install / walk / clear API. |
| `crates/vm/src/vm/feedback.rs` | B | Slow-path migrates entries[POLY_LIMIT..8] to `Vm::polymorphic_chains`. |
| `crates/env/src/agent/weak_finalization.rs` | B | Prune `polymorphic_chains` entries whose `code` is unmarked. |
| `crates/vm/src/vm/metadata_table.rs` | C (NEW) | `MetadataTable` per code object, variable-width buffer + offset table. |
| `crates/vm/src/vm/metadata_table/property.rs` | C (NEW) | `PropertyMetadata` struct + accessors. |
| `crates/vm/src/vm/metadata_table/call.rs` | C (NEW) | `CallMetadata`. |
| `crates/vm/src/vm/metadata_table/arith.rs` | C (NEW) | `ArithMetadata`. |
| `crates/vm/src/vm/metadata_table/comparison.rs` | C (NEW) | `ComparisonMetadata`. |
| `crates/vm/src/vm/metadata_table/keyed_property.rs` | C (NEW) | `KeyedPropertyMetadata`. |
| `crates/vm/src/dsl/backend/aarch64/feedback.rs` | C | New resolve macro: load table base, load offsets[kind], add slot*stride. ~5 instructions vs. today's `add x21, x17, lsl #6`. |
| `crates/vm/src/vm/installation.rs` (or equivalent) | C | Allocate `MetadataTable` at code install; size from per-kind opcode counts stashed on `CompiledScriptUnit`. |
| `crates/vm/src/vm/metadata_table/property.rs` (etc.) | D | Re-home IC state machine: `install_handler`, `transition_to_mega`, `bump_generation`, `clear` move from `NamedPropertyFeedback` to per-kind metadata impls. |
| `crates/vm/src/vm/feedback.rs` | D | Delete `FeedbackVector`, `FeedbackSiteState`, `NamedPropertyFeedback`, `with_feedback_slot_mut`, `with_feedback_slot`. |
| `crates/vm/src/dsl/feedback_flat.rs` | D | Delete entirely. |
| `crates/vm/src/vm/status.rs` | E (NEW) | Per-kind `*Status` types + `MetadataTableFootprint`. Built on demand from `MetadataTable` + `polymorphic_chains`. |
| `crates/vm/src/vm.rs` | E | `named_property_status`, `call_status`, `arith_status`, `comparison_status`, `keyed_property_status`, `metadata_table_footprint`. |
| `crates/vm/src/tests/feedback.rs` + `crates/vm/src/tests/inline_caches.rs` | E | Update ~12 test consumers from `FeedbackVectorSnapshot`/`Footprint` to per-kind status. Delete `FeedbackVectorSnapshot`, `FeedbackVectorFootprint`, related types. |

### 2.2 Backward compatibility

- **Phase A** is the only phase with user-visible behavior change, and it's a no-op: `Object.setPrototypeOf` no longer carries a per-object epoch bump. Spec 1 already made it shape-transitioning, so the epoch was redundant. Tests that asserted on epoch counts (e.g., `redefine_delete_and_prototype_mutation_bump_invalidation_epochs`) are rewritten to assert on shape changes — Spec 1 already augmented that test with shape-change assertions, so removing the epoch parts is mechanical.
- **Phases B–E** are pure refactors. Same JS semantics, same IC mono→poly→mega state machine, same hit rates. The asm dispatch contract changes in Phase C; that's an internal contract, not user-visible.
- **Public API breakage** is concentrated in Phase E's single PR. Test/profiler callers of `FeedbackVectorSnapshot`/`Footprint` update in the same PR that deletes them.

### 2.3 Cross-cutting invariants Spec 2 must preserve

1. **IC state machine externally observable**: Uninit → Mono → Poly → Mega transitions remain detectable via the new per-kind status API at each phase boundary.
2. **POLY_LIMIT_TOTAL = 8**: the 9th distinct receiver shape transitions the slot to Mega.
3. **Tier-up timing**: `warmup_counter ≥ 2` triggers feedback-storage allocation; per-site execution counts increment at the same opcode boundaries.
4. **Shape-compare semantics**: IC fast-path's shape compare is bit-exact `ShapeId == ShapeId`. No relaxation.

---

## 3. Phase A — Watchpoint IC adoption + epoch removal + counter lift

### 3.1 `AdaptiveProtoLoad` observer

```rust
// crates/objects/src/watchpoint.rs (extension)

pub enum Watchpoint {
    ShapeInvalidation { observer: ShapeInvalidationObserver },
    // No new top-level variant.
}

pub enum ShapeInvalidationObserver {
    Recording { token: u64 },
    /// Production consumer. Identifies an IC slot to clear when a depended-on
    /// shape transitions. The `generation` field guards against stale fires
    /// against a slot that has been re-cached since registration.
    AdaptiveProtoLoad {
        code: CodeRef,
        slot: FeedbackSlotId,
        generation: u32,
    },
}
```

Reusing the existing `Watchpoint::ShapeInvalidation` variant keeps Spec 1's dispatch path exhaustive — adding a new observer kind is a single new arm in `Agent::fire_watchpoints_for_shape`'s inner match.

### 3.2 Per-IC `generation` + IC slot identity

Each IC slot gains a `generation: u32`. Every install/re-install bumps the slot's generation. `AdaptiveProtoLoad::fire` reads the slot's current generation, compares to its own, no-ops on mismatch.

```rust
// In Agent::fire_watchpoints_for_shape's dispatch loop:
ShapeInvalidationObserver::AdaptiveProtoLoad { code, slot, generation } => {
    self.vm.clear_ic_slot_if_generation_matches(code, slot, generation);
}
```

```rust
// crates/vm/src/vm.rs (NEW)
pub(crate) fn clear_ic_slot_if_generation_matches(
    &mut self,
    code: CodeRef,
    slot: FeedbackSlotId,
    expected_generation: u32,
) {
    let Some(vector) = self.feedback_vector_mut(code) else { return };
    let Some(site) = vector.site_mut(slot) else { return };
    if site.generation() != expected_generation { return; }
    site.clear();
    site.bump_generation();
    self.mirror_flat_slot(code, slot);  // dual-write until Phase D
}
```

The `feedback_vector_mut` returning `None` for a dead code object is the GC-safety guard (§8.5).

### 3.3 IC slow-path registers `AdaptiveProtoLoad` on chain shapes

On a successful proto-chain lookup that caches into an IC slot (`crates/objects/src/internal_methods/property_cache.rs`'s install path):

```rust
let generation = vm.feedback_vector_mut(code).site_mut(slot).bump_generation_and_get();

for &shape in &chain_shapes {
    // chain_shapes = [proto1_shape, proto2_shape, ..., holder_shape]
    // receiver_shape is excluded (covered by IC fast-path shape compare).
    let result = agent.objects_mut().watchpoint_set_mut(shape).register(
        Watchpoint::ShapeInvalidation {
            observer: ShapeInvalidationObserver::AdaptiveProtoLoad {
                code: code.clone(),
                slot,
                generation,
            },
        },
    );
    if result.is_err() {
        // The shape is already Invalidated. Abandon install — next read
        // retries against the new state.
        vm.feedback_vector_mut(code).site_mut(slot).clear();
        return;
    }
}

// All registrations succeeded; commit the cache entry.
vm.feedback_vector_mut(code).site_mut(slot).install_proto_handler(handler);
```

### 3.4 IC fast-path: drop epoch comparison

Today (`crates/vm/src/vm/dispatch/property.rs:108-114`):
```rust
record.shape() == handler.receiver_shape()
    && record.last_invalidation_epoch().unwrap_or(0) == cached_epoch
```

After Phase A:
```rust
record.shape() == handler.receiver_shape()
```

The proto-cache fast path drops `record.last_invalidation_epoch() == named_aux_epoch` similarly. Proto-chain mutations now invalidate via `AdaptiveProtoLoad` firing and clearing the slot.

### 3.5 `FeedbackEntry` shrinks

```rust
// Phase A FeedbackEntry (64B for stride compat; epoch fields become padding):
struct FeedbackEntry {
    mode: u8,
    _pad: [u8; 3],
    generation: u32,              // NEW
    named_handler_bits: u64,
    named_aux_bits: u64,
    scalar_observed_bits: u32,
    scalar_execution_count: u32,
    _tail_pad: [u8; 32],          // grew 16B (reclaimed from epochs)
}
```

Stride stays 64B until Phase C. The asm pin (`x21 + slot*64`) is unchanged. `FeedbackEntry` is deleted entirely in Phase D; Phase C replaces it with per-kind metadata structs reachable through the offset-table-based resolve.

### 3.6 Delete invalidation infrastructure

Once Phase A's IC fast path no longer reads epochs and slow path no longer writes them:

- Delete `RuntimeObjectRecord::last_invalidation_epoch` field.
- Delete `ObjectRuntime::next_invalidation_epoch` field.
- Delete `ObjectRuntime::bump_invalidation` method.
- Delete `InvalidationCause` enum.
- Delete `bump_prototype_mutation_epoch` wrapper from Spec 1.
- Update the four callsites (`set_prototype_of`, `define_own_property`, `delete`, `ensure_named_property_dictionary`) to remove the `bump_invalidation` calls. The Spec 1 watchpoint fires remain — those are the new (and now sole) invalidation signal.

### 3.7 Tier-up counter lift

`FeedbackVector::warmup_counter: u16` moves onto `Tiering` (the per-VM tiering struct lifted in Spec 1). The single existing reader (`bump_warmup`) and the allocation-threshold check (`warmup_counter >= 2`) update to the new home. Per-site `scalar_execution_count` stays inside the per-IC metadata — the asm DSL increments it on the dispatch hot path, where memory adjacency to the handler bits matters.

### 3.8 Phase A test surface

| # | What it proves |
|---|---|
| A1 | IC fast path: receiver shape mismatch → cache miss → slow path. (Existing test family verifies no epoch read remains.) |
| A2 | Proto-chain holder mutation: `obj.__proto__.x = newValue` after `obj.x` cached → `AdaptiveProtoLoad` fires → IC slot cleared → next read re-caches. |
| A3 | Intermediate proto mutation: two-hop chain, mutate the middle proto → all dependent IC slots cleared. |
| A4 | Generation guard: install IC → fire `AdaptiveProtoLoad` → re-install at new chain (new generation) → orphan watchpoint from original install fires → no-op (slot's current generation doesn't match). |
| A5 | Register-on-invalidated-shape during install: abandon install → next IC read retries against the new state and succeeds. |
| A6 | Tier-up counter moved: `warmup → allocation` at threshold still works; existing tier-up tests pass. |
| A7 | `last_invalidation_epoch` field absence: grep returns no matches; `RuntimeObjectRecord` size shrinks. |
| A8 | Updated `redefine_delete_and_prototype_mutation_bump_invalidation_epochs`: assertions on epoch counts removed; shape-change assertions (Spec 1) retained. |
| A9 | Full `crates/vm/src/tests/inline_caches.rs` suite green. |

Phase A ships in 3 PRs:
- A.1: `AdaptiveProtoLoad` variant + slow-path registration + generation field on `FeedbackEntry`. (Tests A4, A5 land here.)
- A.2: IC fast-path migrates to shape-only compare + epoch fields removed from `FeedbackEntry`. (Tests A1, A2, A3 land here.)
- A.3: Delete `bump_invalidation` / `last_invalidation_epoch` / `InvalidationCause` + tier-up lift. (Tests A6, A7, A8, A9.)

---

## 4. Phase B — Polymorphic chain out-of-line

### 4.1 Current state recap

Per recon (`crates/vm/src/vm/feedback.rs:687-701`):
```rust
struct NamedPropertyFeedback {
    entry_count: u8,
    entries: [Option<NamedPropertyCacheEntry>; 8],
    polymorphic_own_data_handlers: [NamedPropertyHandler; POLY_LIMIT],  // 2
    polymorphic_own_data_epochs: [u64; POLY_LIMIT],  // deleted in Phase A
}
```

- `entries[0..POLY_LIMIT]` (the first 2) live in `NamedPropertyFeedback` AND mirror into `feedback_flat_storage` for asm fast-path walks.
- `entries[POLY_LIMIT..8]` (the remaining 6) live only in `NamedPropertyFeedback`, reachable via a slow-path binary search.

Phase B lifts `entries[POLY_LIMIT..8]` out of `NamedPropertyFeedback` into a separate map. The inline `polymorphic_own_data_handlers` sidecar stays — the asm fast path still uses it; Phase C folds everything together.

### 4.2 New storage

```rust
// crates/vm/src/vm/feedback/polymorphic.rs (NEW)

pub struct PolymorphicChain {
    /// Up to (8 - POLY_LIMIT) entries; the inline sidecar holds the first POLY_LIMIT.
    /// On total overflow (8th distinct receiver shape), the slot transitions to
    /// Megamorphic and this entry is cleared.
    entries: Vec<NamedPropertyCacheEntry>,
}

impl PolymorphicChain {
    pub fn new() -> Self { ... }
    pub fn len(&self) -> usize { ... }
    pub fn push(&mut self, entry: NamedPropertyCacheEntry) -> InstallResult { ... }
    pub fn find_by_shape(&self, shape: ShapeId) -> Option<&NamedPropertyCacheEntry> { ... }
    pub fn clear(&mut self) { ... }
}

// On Vm:
pub struct Vm {
    // ...existing fields...
    polymorphic_chains: HashMap<(CodeId, FeedbackSlotId), PolymorphicChain>,
}
```

Lazy: monomorphic and ≤POLY_LIMIT polymorphic slots have no map entry.

### 4.3 Mid-state during Phase B

For each slot:
- `entries[0..POLY_LIMIT]` live in `NamedPropertyFeedback` (legacy, dual-written to flat storage). The asm fast path still walks these 2 entries.
- `entries[POLY_LIMIT..8]` live in `Vm::polymorphic_chains[(code, slot)]`. The slow path walks both inline and out-of-line.

This is one transitional step; in Phase D the inline copy is also lifted, eliminating the duplication.

### 4.4 Slow-path operations

**Install (transition to 3+ entries):**
```rust
fn install_polymorphic_entry(
    vm: &mut Vm,
    code: CodeId,
    slot: FeedbackSlotId,
    entry: NamedPropertyCacheEntry,
) -> InstallOutcome {
    let feedback = vm.feedback_vector_mut(code).site_mut(slot);
    if feedback.entry_count() < POLY_LIMIT {
        feedback.push_inline(entry);
        return InstallOutcome::Polymorphic;
    }
    let chain = vm.polymorphic_chains
        .entry((code, slot))
        .or_insert_with(PolymorphicChain::new);
    if feedback.entry_count() + chain.len() >= POLY_LIMIT_TOTAL {
        feedback.transition_to_mega();
        vm.polymorphic_chains.remove(&(code, slot));
        return InstallOutcome::Megamorphic;
    }
    chain.push(entry);
    feedback.bump_entry_count();
    InstallOutcome::Polymorphic
}
```

**Walk (slow-path lookup):**
```rust
fn walk_polymorphic(
    vm: &Vm,
    code: CodeId,
    slot: FeedbackSlotId,
    receiver_shape: ShapeId,
) -> Option<&NamedPropertyCacheEntry> {
    let feedback = vm.feedback_vector(code)?.site(slot)?;
    if let Some(entry) = feedback.find_by_shape_inline(receiver_shape) {
        return Some(entry);
    }
    vm.polymorphic_chains.get(&(code, slot))?.find_by_shape(receiver_shape)
}
```

**Clear (on `AdaptiveProtoLoad` fire or Mega transition):**
```rust
fn clear_slot(vm: &mut Vm, code: CodeId, slot: FeedbackSlotId) {
    if let Some(feedback) = vm.feedback_vector_mut(code).site_mut(slot) {
        feedback.clear_inline();
        feedback.bump_generation();
    }
    vm.polymorphic_chains.remove(&(code, slot));
    vm.mirror_flat_slot(code, slot);
}
```

### 4.5 GC

`PolymorphicChain` holds `NamedPropertyCacheEntry`s, which reference `ObjectRef`s (holder shapes, proto identity, etc.). Two GC concerns:

1. **Tracing:** `Vm::polymorphic_chains` must be visited as a root. Hook into the existing `Vm` trace pass.
2. **Pruning on dead code:** when a `CodeRef` is GC'd, drop all `(code, slot)` keys for it. Mirror Spec 1's `prune_dead_prototype_transitions` pattern at `crates/env/src/agent/weak_finalization.rs`.

### 4.6 Phase B test surface

| # | What it proves |
|---|---|
| B1 | Mono→Poly: 2 entries cached, second install → both inline, no map entry. |
| B2 | Poly→Poly: third install → out-of-line map entry created; total entries = 3. |
| B3 | Poly→Mega: ninth install → map entry deleted, slot transitions to Mega. |
| B4 | Walk order: inline-first then out-of-line returns same result as legacy unified walk. |
| B5 | Clear on `AdaptiveProtoLoad` fire removes both inline and out-of-line entries; map cleaned. |
| B6 | GC: code object dies → its `(code, slot)` entries pruned. |
| B7 | GC: code object lives → entries retained. |
| B8 | `crates/vm/src/tests/inline_caches.rs` Poly cases (≥3 distinct receiver shapes) still pass. |

Phase B ships in 2 PRs:
- B.1: `PolymorphicChain` + `Vm::polymorphic_chains` + slow-path migration. (B1–B5 land here.)
- B.2: GC sweep for dead-code polymorphic entries. (B6, B7, B8.)

Asm fast path is unchanged across Phase B (still reads the inline 2-entry sidecar through `feedback_flat_storage`). All Phase B changes are slow-path only.

---

## 5. Phase C — `MetadataTable` per code object

### 5.1 Layout

One `MetadataTable` buffer per code object, allocated at code installation. Variable-width per opcode kind, with an offset table that maps `(opcode_kind, instance_index)` to a byte offset.

```
MetadataTable byte layout:

  +----------------------+  <- table_base (held in x21 during dispatch)
  | LinkingData          |     u32 buffer_size, u32 metadata_count[K]
  +----------------------+
  | Offset table         |     u32[K] (one offset per opcode kind)
  +----------------------+  <- offsets[Property]
  | PropertyMetadata[N0] |
  +----------------------+  <- offsets[Call]
  | CallMetadata[N1]     |
  +----------------------+  <- offsets[Arith]
  | ArithMetadata[N2]    |
  +----------------------+  <- offsets[Comparison]
  | ComparisonMetadata[N3]|
  +----------------------+  <- offsets[KeyedProperty]
  | KeyedPropertyMetadata[N4]|
  +----------------------+
```

Per-kind struct layouts (placeholder sizes; finalized during Phase C implementation):

```rust
struct PropertyMetadata {
    mode: u8,
    _pad: [u8; 3],
    generation: u32,
    handler_bits: u64,        // OwnData / Proto / Polymorphic discriminant + payload
    aux_bits: u64,            // proto holder ref for proto-cache
    execution_count: u32,
    _tail: u32,
}                              // 32B

struct CallMetadata {
    mode: u8,
    _pad: [u8; 3],
    generation: u32,
    callee_bits: u64,
    execution_count: u32,
    _tail: u32,
}                              // 24B

struct ArithMetadata {
    observed_bits: u32,
    execution_count: u32,
}                              // 8B

struct ComparisonMetadata {
    observed_bits: u32,
    execution_count: u32,
}                              // 8B

struct KeyedPropertyMetadata {
    mode: u8,
    _pad: [u8; 3],
    generation: u32,
    handler_bits: u64,
    execution_count: u32,
    _tail: u32,
}                              // 24B
```

The structural commitment is per-kind variable-width and stable sizes per kind; exact byte counts settled by Phase C implementation against actual `NamedPropertyHandler` / `CallHandler` representations.

### 5.2 Asm DSL resolve macro

Today (`load_feedback_site!` in `crates/vm/src/dsl/backend/aarch64/feedback.rs`):
```asm
add x{dst}, x21, x17, lsl #6     // x21 (FV base) + slot * 64
```

After Phase C:
```asm
ldr w{kind_off}, [x21, #OFFSET_TABLE_OFFSET + KIND * 4]  // load offsets[kind]
add x{dst}, x21, x{kind_off}                              // run base
mov x{stride}, #PROPERTY_METADATA_STRIDE                  // const per kind
madd x{dst}, x{slot}, x{stride}, x{dst}                   // dst = stride*slot + run_base
```

~5 instructions per resolve. Bytecode emission pre-computes the opcode kind, so `KIND` and `STRIDE` are constants in the emitted code.

`x21` is repurposed: it now holds the `MetadataTable` base. The save/restore boundaries don't change (still per-call), only what `x21` points at.

### 5.3 Allocation

`MetadataTable` allocates at code installation:

1. Count opcodes by kind (already known at bytecode emit time; stash on `CompiledScriptUnit`).
2. Allocate one contiguous `Box<[u8]>` sized to fit `LinkingData` + offset table + per-kind runs.
3. Initialize `LinkingData` header.
4. Compute offsets[kind] for each kind, write to the offset table.
5. Zero-initialize the per-kind runs.
6. Stash the box pointer on `Vm::metadata_tables: Vec<Option<Box<[u8]>>>` keyed by `CodeId`.

The buffer lifetime is tied to the code object. When the code is GC'd, the table drops.

### 5.4 Migration strategy

Phase C is the highest-risk phase. Staged dual-write:

1. **C.1 — Layout + allocation.** Add `MetadataTable` allocation alongside the legacy `FeedbackVector` allocation. Per-kind metadata structs defined. Don't read from the table yet.
2. **C.2 — Dual-write integration.** Every mutation of `FeedbackVector` also mutates the corresponding `MetadataTable` entry. Both storages track state in parallel.
3. **C.3 — Equivalence assertion.** Add a debug-build assertion that compares `FeedbackVector[slot]`'s projection against `MetadataTable[slot]` on every IC mutation. Run the full test suite with this enabled.
4. **C.4 — Asm pin flip.** Switch the asm DSL pin contract: `x21` now loads from `Vm::metadata_tables[code_id]` instead of `feedback_flat_storage`. IC fast-path reads hit the new buffer. Step 5 of Phase D drops the legacy storage.

Step C.4 is the asm contract flip — a single PR, well-bounded, blast radius = asm dispatch tests. If it destabilizes, revert to the dual-write state in C.3 (still functional, just slower) until the asm path is fixed.

### 5.5 Phase C test surface

| # | What it proves |
|---|---|
| C1 | Table allocation: every installed function has a `MetadataTable` sized per its opcode counts. |
| C2 | Offset table correctness: `offsets[kind] + slot * stride` lands inside the run for `kind`. |
| C3 | Dual-write equivalence (debug-only): every `FeedbackVector` mutation reflects in `MetadataTable`. |
| C4 | Asm fast path reads from `MetadataTable`: existing IC hit/miss tests pass on the new layout. |
| C5 | Per-kind sizes: arithmetic site uses 8B; property site uses ~32B; sizes match the offset-table computed runs. |
| C6 | GC of code object releases the table. |
| C7 | Workspace + IC suite green. |
| C8 | Microbench (Spec 1's property-addition + new IC-hot-loop): ≤3% wall-clock delta vs. pre-Phase-C. |

Phase C ships in 4 PRs (C.1, C.2, C.3, C.4). C.1–C.3 are pure additions with no production-path changes; C.4 flips the asm pin.

---

## 6. Phase D — Flip system of record

### 6.1 State after Phase C

Both storages exist:
- Legacy `FeedbackVector` + `feedback_flat_storage` (mirror) carry the state machine logic.
- New `MetadataTable` carries the IC data the asm pin reads.
- Dual-write keeps them in sync.

Phase D re-homes the state machine onto `MetadataTable` and deletes the legacy paths.

### 6.2 D.1 — Re-home IC state machine

Port `NamedPropertyFeedback::refresh_monomorphic_own_data_handler`, the install/transition logic, and the per-site state methods from `vm/feedback.rs` to per-kind metadata impls:

```rust
// crates/vm/src/vm/metadata_table/property.rs

impl PropertyMetadata {
    pub fn install_monomorphic(&mut self, handler: NamedPropertyHandler) -> InstallOutcome { ... }
    pub fn install_polymorphic(&mut self, handler: NamedPropertyHandler) -> InstallOutcome { ... }
    pub fn transition_to_mega(&mut self) { ... }
    pub fn clear(&mut self) { ... }
    pub fn bump_generation(&mut self) -> u32 { ... }
}
```

Same shape for `CallMetadata`, `KeyedPropertyMetadata`, etc. Each per-kind metadata file owns its state machine. `Vm::polymorphic_chains` is independent of `FeedbackVector`; it stays as Phase B left it.

The IC slow-path call shape changes:

Before (Phase C+B):
```rust
let outcome = vm.with_feedback_slot_mut(code, slot, |site| {
    site.install_named_property_handler(handler)
});
vm.mirror_flat_slot(code, slot);
```

After (Phase D):
```rust
let outcome = vm.metadata_table_mut(code)
    .property_metadata_mut(slot)
    .install_handler(handler);
```

### 6.3 D.2 — Delete legacy storage

- Delete `Vm::feedback_vectors: Vec<FeedbackVector>`.
- Delete `FeedbackVector`, `FeedbackSiteState`, `NamedPropertyFeedback`.
- Delete `with_feedback_slot_mut`, `with_feedback_slot`.
- Delete `mirror_flat_slot` + `feedback_flat_storage` module entirely.
- Delete the debug-only equivalence assertion from C.3.

Stub `FeedbackVectorSnapshot` / `FeedbackVectorFootprint` to keep the workspace compiling — return empty/zero values, `#[ignore]` dependent tests for the brief window until Phase E re-implements them.

### 6.4 Phase D test surface

| # | What it proves |
|---|---|
| D1 | State machine: Uninit→Mono via single install on `PropertyMetadata`. |
| D2 | State machine: Mono→Poly via second install with distinct shape. |
| D3 | State machine: Poly→Mega via 8th install (inline + out-of-line full). |
| D4 | Re-install on Mono after `AdaptiveProtoLoad` clear → returns to Uninit then Mono. |
| D5 | `feedback_flat_storage` deleted: grep returns no matches outside Phase D's deletion commit. |
| D6 | `FeedbackVector` deleted: same grep check. |
| D7 | Full `crates/vm/src/tests/inline_caches.rs` suite green on the new state-machine code path. |
| D8 | Microbench: wall-clock delta vs. pre-Phase-D ≤1% (pure cleanup; regression indicates a bug). |

Phase D ships in 2 PRs (D.1 state-machine re-home, D.2 deletion).

---

## 7. Phase E — Status projections + API drop

### 7.1 Per-kind Status types

```rust
// crates/vm/src/vm/status.rs (NEW)

pub struct NamedPropertyStatus {
    pub state: InlineCacheState,
    pub generation: u32,
    pub execution_count: u32,
    pub entries: Vec<NamedPropertyStatusEntry>,
}

pub struct NamedPropertyStatusEntry {
    pub receiver_shape: ShapeId,
    pub kind: NamedPropertyEntryKind,  // OwnData / Proto / etc.
    pub handler_summary: NamedPropertyHandlerSummary,
}

pub struct CallStatus {
    pub state: InlineCacheState,
    pub generation: u32,
    pub execution_count: u32,
    pub callee: Option<CalleeSummary>,
}

pub struct ArithStatus {
    pub observed: ScalarObserved,
    pub execution_count: u32,
}

pub struct ComparisonStatus { /* analogous */ }
pub struct KeyedPropertyStatus { /* analogous */ }

pub struct MetadataTableFootprint {
    pub allocated_bytes: usize,
    pub live_site_count_by_kind: [usize; OPCODE_KIND_COUNT],
    pub total_execution_count: u64,
}
```

Each `*Status` is `Clone`, has no GC roots, is purely a value type. Callers can keep the value across IC mutations.

### 7.2 Query API

```rust
impl Vm {
    pub fn named_property_status(&self, code: CodeRef, slot: FeedbackSlotId) -> NamedPropertyStatus { ... }
    pub fn call_status(&self, code: CodeRef, slot: FeedbackSlotId) -> CallStatus { ... }
    pub fn arith_status(&self, code: CodeRef, slot: FeedbackSlotId) -> ArithStatus { ... }
    pub fn comparison_status(&self, code: CodeRef, slot: FeedbackSlotId) -> ComparisonStatus { ... }
    pub fn keyed_property_status(&self, code: CodeRef, slot: FeedbackSlotId) -> KeyedPropertyStatus { ... }
    pub fn metadata_table_footprint(&self, code: CodeRef) -> MetadataTableFootprint { ... }
}
```

Each query reads the relevant `MetadataTable` slot, projects into the value type, returns. No caching — every call walks the buffer.

### 7.3 Test consumer updates

Pattern across the ~12 consumers in `crates/vm/src/tests/feedback.rs` + `tests/inline_caches.rs`:

Before (Phase D stubbed):
```rust
let snap = vm.feedback_vector_snapshot(code);
assert_eq!(snap.sites[3].state, InlineCacheState::Polymorphic);
assert_eq!(snap.sites[3].entries.len(), 3);
```

After:
```rust
let status = vm.named_property_status(code, FeedbackSlotId(3));
assert_eq!(status.state, InlineCacheState::Polymorphic);
assert_eq!(status.entries.len(), 3);
```

The per-kind query is more ergonomic — tests already know which kind they're asserting on.

### 7.4 Deletions (same PR as consumer updates)

- Delete `FeedbackVectorSnapshot`, `NamedPropertyFeedbackSnapshot`, `FeedbackSiteSnapshot`.
- Delete `FeedbackVectorFootprint`.
- Delete `Vm::feedback_vector_snapshot`, `Vm::feedback_vector_footprint`.
- Delete the Phase D stubs.

### 7.5 Phase E test surface

| # | What it proves |
|---|---|
| E1 | `named_property_status` returns the same `state`/`entries` shape that the old snapshot exposed. |
| E2 | `call_status` likewise. |
| E3 | `arith_status` returns observed kind + execution count. |
| E4 | `metadata_table_footprint` reports correct allocated bytes (verifiable via known per-kind sizes). |
| E5 | Status types `Clone` + outlive subsequent IC mutations (no internal refs). |
| E6 | All ~12 updated test consumers pass. |
| E7 | Grep: `FeedbackVectorSnapshot` / `FeedbackVectorFootprint` appear only in the Phase E deletion commit's diff. |
| E8 | Workspace + IC suite green. |

Phase E ships in 1 PR (Status API + consumer updates + deletions are tightly coupled).

---

## 8. Error handling and edge cases

### 8.1 `AdaptiveProtoLoad` fire ordering vs. IC re-cache

Spec 1 fires watchpoints *after* the object's shape pointer is updated. This continues. The fire callback (`AdaptiveProtoLoad`) takes `&mut Vm` (via `&mut Agent`) and mutates IC storage. No JS callback runs during fire (clear-on-fire is pure storage mutation), so no reentrancy through JS.

If a shape transitions twice in quick succession (e.g., property add + property delete on the same proto), the second transition's watchpoint fire is a no-op — the IC was already cleared by the first.

### 8.2 Generation counter wraparound

`generation: u32` allows ~4 billion (re-)installs per IC slot before wraparound. Cold paths reach thousands of re-installs at most. Wraparound is a correctness concern only in pathological microbenchmarks; documented as acceptable risk. Widen to `u64` if it becomes an issue.

### 8.3 Polymorphic chain capacity overflow

`POLY_LIMIT_TOTAL = 8`. The 9th distinct shape transitions the slot to Mega and discards all 8 cached entries. `PolymorphicChain::push` returns `InstallOutcome::Megamorphic` when at cap; the caller transitions and removes the map entry. Existing test coverage for the Mega transition catches this.

### 8.4 GC during IC fast-path read

The IC fast path runs in the asm DSL hot loop; GC cannot interrupt it (single-threaded VM, no safepoints in the IC read sequence). Mutations to `MetadataTable` only happen on slow path / fire callback, both with `&mut Vm` access and not under asm dispatch.

### 8.5 Code object GC with watchpoints still Watched

A code object is GC'd; its `(code, slot)` `AdaptiveProtoLoad` watchpoints are still in `Watched` state on various shape sets. When those shapes later transition, the fire callback reads `Vm::feedback_vector_mut(code)` and gets `None` — early return. The Watched watchpoint becomes detritus on the shape set until the shape transitions or is itself GC'd.

This matches Spec 1's tolerance: orphan watchpoints are accepted; they only consume memory until their owning shape goes through GC.

If profiling shows accumulation of dead-code orphan watchpoints, add a sweep in `weak_finalization.rs` that walks `Watched` sets and drops watchpoints whose `code` is unmarked. Out of scope for Spec 2; flagged for future profiling.

### 8.6 Concurrent access

VM stays single-threaded. `MetadataTable`'s `Box<[u8]>` is owned by `Vm`. The asm DSL pin holds a raw `*mut` to the buffer; `Vm` invariants ensure no concurrent mutation while asm dispatches. Same contract as `feedback_flat_storage` today.

### 8.7 Asm pin desynchronization (Phase C.4 risk)

If the asm pin loads `x21` from the wrong code object's `MetadataTable`, dispatch reads bogus IC data. Mitigation: every call into a function reloads `x21` from the callee's metadata-table pointer; every return restores the caller's `x21`. Lifecycle matches today's `x21` (FV base) — only the pointed-at storage changes.

### 8.8 `Vm::polymorphic_chains` keyed by `CodeId`

`CodeId` is currently used as a stable identifier across the VM. When a code object is GC'd, its `CodeId` may or may not be reused. The GC sweep in B.2 drops entries whose code is unmarked; this runs before any new code could be assigned a recycled ID, so no stale-key collision is possible. Verified by Spec 1's GC sweep pattern.

### 8.9 Recording observer overhead

Spec 1 left `recording_watchpoint_fires: Vec<u64>` ungated on `ObjectRuntime` (24B production overhead). Spec 2 doesn't change this. Could be revisited in a cleanup pass post-Spec-2.

---

## 9. Implementation order

```
Phase A (3 PRs)
  A.1: AdaptiveProtoLoad variant + slow-path registration + generation field
  A.2: IC fast-path migrates to shape-only compare + FeedbackEntry epoch removal
  A.3: Delete bump_invalidation / last_invalidation_epoch / InvalidationCause + tier-up lift

Phase B (2 PRs)
  B.1: PolymorphicChain + Vm::polymorphic_chains + slow-path migration
  B.2: GC sweep for dead-code polymorphic entries

Phase C (4 PRs) — highest risk
  C.1: MetadataTable allocation + LinkingData header + offset table
  C.2: Per-kind metadata structs + dual-write integration
  C.3: Debug-build equivalence assertion across the test suite
  C.4: Asm DSL pin flip — production reads from MetadataTable

Phase D (2 PRs)
  D.1: Re-home IC state machine onto per-kind metadata impls
  D.2: Delete legacy FeedbackVector + feedback_flat_storage + mirror_flat_slot

Phase E (1 PR)
  E.1: Status types + Vm queries + 12-consumer test update + delete FeedbackVector*Snapshot/Footprint
```

Total: 12 PRs. Each phase's first PR is reviewable in isolation against its phase boundary. Tests green at every commit.

---

## 10. Testing strategy

Per-phase test surfaces are listed inline (§3.8, §4.6, §5.5, §6.4, §7.5).

Aggregate gates at every phase boundary:

- **Watchpoint adoption regression:** full `crates/vm/src/tests/inline_caches.rs` (~33 tests) green.
- **Storage equivalence (Phase C.2 → C.4 window):** debug-build assertion compares `FeedbackVector[slot]` projection against `MetadataTable[slot]` on every IC mutation. Runs across the entire test suite for the duration of dual-write.
- **Microbench:**
  - Spec 1's `crates/vm/benches/property_addition.rs`.
  - **NEW** `crates/vm/benches/ic_hot_loop.rs` — a JS function that mixes property loads, arithmetic, and calls. Catches per-kind metadata-resolve cost (the ~5 extra asm instructions in Phase C).
  - Ceiling: ≤3% wall-clock regression vs. pre-Spec-2 baseline at each phase boundary.
- **Phase D / E coordination:** after Phase D deletion, `FeedbackVectorSnapshot`/`Footprint` are stubbed (empty) and dependent tests `#[ignore]`'d. Phase E re-enables them by porting to the Status API in the same PR; no test stays `#[ignore]`'d across a release boundary.

---

## 11. Out of scope (future specs)

- Additional IC kinds (load-from-global, super-property access). Existing kinds map to existing per-kind metadata structs.
- Watchpoint retargeting (`AdaptiveStructure`-style retarget on fire). Spec 2 stays with clear-on-fire.
- DFG / baseline tiers. Spec 2 is LLInt-tier IC machinery only.
- Cross-function IC sharing (megamorphic stub caches).
- Watchpoint root-pinning for fire targets. `AdaptiveProtoLoad`'s `CodeRef` is held weakly; pinning relies on the code's own GC roots.
- `recording_watchpoint_fires` ungating cleanup (Spec 1 decision; 24B production overhead acceptable).
- Sweep for dead-code orphan watchpoints on `Watched` sets. Add if profiling justifies it.

---

## 12. References

- Spec 1: `docs/superpowers/specs/2026-05-25-shape-transitions-and-watchpoints-design.md` — established the watchpoint primitive, shape-transition contracts, and the four fire sites Spec 2 consumes.
- Memory pointer: `~/.claude/projects/-Users-sondre-dev-lyng/memory/project_feedback_jsc_migration.md` — migration sequence rationale, JSC reference points.
- JSC analog file pointers: header of this document.
- Lyng files referenced throughout: §2.1 component map.
