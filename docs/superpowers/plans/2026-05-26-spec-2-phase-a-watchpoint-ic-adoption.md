# Spec 2 Phase A — Watchpoint IC Adoption + Epoch Removal + Tier-Up Counter Lift

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate Lyng's IC fast path off the parallel epoch-based invalidation system onto Spec 1's watchpoint primitive. End state: ICs invalidate via shape-compare + `AdaptiveProtoLoad` watchpoint fires only; `last_invalidation_epoch`, `bump_invalidation`, `InvalidationCause`, and `next_invalidation_epoch` are deleted; `warmup_counter` lifts from `FeedbackVector` to `TieringState`.

**Architecture:** Additive at the watchpoint layer — `AdaptiveProtoLoad` observer kind extends `ShapeInvalidationObserver`. IC slow path registers `AdaptiveProtoLoad` on each shape in the proto chain (excluding the receiver) at install time. IC fast path drops the epoch comparison; receiver shape transitions are caught by Spec 1's shape-transitioning machinery, proto-chain mutations are caught by `AdaptiveProtoLoad` clearing the IC slot. Per-IC `generation: u32` guards against stale watchpoints firing on a re-cached slot.

**Tech Stack:** Rust, watchpoint primitive from Spec 1, existing `FeedbackVector`/`FeedbackEntry` storage, `Tiering`/`TieringState` from Spec 1's tiering lift.

**Spec:** `docs/superpowers/specs/2026-05-26-spec-2-ic-jsc-migration-design.md` (§3 — Phase A).

---

## Context

Spec 1 made `Object.setPrototypeOf` shape-transitioning and added per-shape `WatchpointSet`s, but kept the parallel epoch-based IC invalidation alive. Phase A retires the epoch system. The migration is internally safe because:

1. **Receiver-shape transitions** were already redirected by Spec 1: `setPrototypeOf`, `defineProperty` to non-configurable, `delete`, and dictionary transitions all change the object's `ShapeId`. The IC fast path's shape compare catches these.
2. **Proto-chain mutations** are not caught by the IC's receiver-shape compare (the receiver shape is unchanged). Phase A wires `AdaptiveProtoLoad` watchpoints registered at IC install time on every chain shape; when a chain shape transitions (e.g., property add to a proto), the watchpoint fires, the IC slot clears, the next read re-caches.
3. **Generation counter** on each IC slot guards against orphan watchpoints from prior installs firing on a re-cached slot. Each install/clear bumps the slot's generation; `AdaptiveProtoLoad::fire` no-ops on mismatch.

Existing state (verified pre-plan):
- `Watchpoint::ShapeInvalidation { observer: ShapeInvalidationObserver::Recording { token } }` is the only variant ([crates/objects/src/watchpoint.rs:18-42](crates/objects/src/watchpoint.rs#L18-L42)).
- `ObjectRuntime::fire_watchpoints_for_shape` drains and dispatches `Recording` ([crates/objects/src/runtime.rs:1237-1254](crates/objects/src/runtime.rs#L1237-L1254)).
- `Agent::fire_watchpoints_for_shape` delegates to objects layer ([crates/env/src/agent.rs:453-455](crates/env/src/agent.rs#L453-L455)).
- IC fast path checks epoch at three sites: monomorphic OwnData ([property.rs:108-132](crates/vm/src/vm/dispatch/property.rs#L108-L132)), polymorphic OwnData (via helper), and PrototypeData (via helper, [property.rs:156-162](crates/vm/src/vm/dispatch/property.rs#L156-L162)).
- `FeedbackEntry` is 64B with two epoch fields at offsets 16 and 32 ([crates/vm/src/dsl/feedback_flat.rs:70-86](crates/vm/src/dsl/feedback_flat.rs#L70-L86)).
- `bump_invalidation` is called at 5 sites: 2 in `internal_methods.rs` (lines 425, 460), 3 in `named_properties.rs` (lines 173, 195, 235).
- `warmup_counter: u16` lives on `FeedbackVector` ([crates/vm/src/vm/feedback.rs:2043-2046](crates/vm/src/vm/feedback.rs#L2043-L2046)).
- `TieringState` is `pub(super)` with fields like `eligible`, `hotness`, `feedback_events` ([crates/vm/src/vm/tiering.rs:66-74](crates/vm/src/vm/tiering.rs#L66-L74)).

---

## File map

| File | New/existing | Responsibility |
|---|---|---|
| `crates/objects/src/watchpoint.rs` | existing | Add `ShapeInvalidationObserver::AdaptiveProtoLoad { code, slot, generation }`. Add `pub struct FeedbackSlotId(pub u32)` for typed slot index (use `lyng_types::FeedbackSlotId` if already there). Refactor `ObjectRuntime::fire_watchpoints_for_shape` into a drain-only `drain_watchpoints_for_shape`. |
| `crates/objects/src/runtime.rs` | existing | Rename `fire_watchpoints_for_shape` → `drain_watchpoints_for_shape(shape) -> Option<Vec<Watchpoint>>` (drain-only, no dispatch). |
| `crates/env/src/agent.rs` | existing | `Agent::fire_watchpoints_for_shape` does all dispatch: routes `Recording` to `objects.recording_watchpoint_fires`, routes `AdaptiveProtoLoad` to `Vm::clear_ic_slot_if_generation_matches`. |
| `crates/vm/src/vm.rs` | existing | Add `pub(crate) fn clear_ic_slot_if_generation_matches(&mut self, code, slot, generation)`. |
| `crates/vm/src/vm/feedback.rs` | existing | Add per-slot `generation: u32` access. Slow-path install bumps generation + registers `AdaptiveProtoLoad`. PR A.2: drop epoch reads from header projection. PR A.3: lift `warmup_counter` to `TieringState`; delete `FeedbackVector::warmup_counter`. |
| `crates/vm/src/dsl/feedback_flat.rs` | existing | PR A.1: add `generation: u32` to `FeedbackEntry`. PR A.2: drop `named_epoch` + `named_aux_epoch`; update `set_named_*` methods. |
| `crates/vm/src/vm/dispatch/property.rs` | existing | PR A.2: drop epoch comparisons. |
| `crates/objects/src/internal_methods/property_cache.rs` | existing | Slow path: register `AdaptiveProtoLoad` on each proto-chain shape; remove `record.last_invalidation_epoch()` capture into `PropertyCacheDependency::new(...)`; remove epoch field from `PropertyCacheDependency`. |
| `crates/objects/src/internal_methods.rs` | existing | PR A.3: delete `bump_invalidation` calls (lines 425, 460); delete `bump_prototype_mutation_epoch` wrapper. |
| `crates/objects/src/internal_methods/named_properties.rs` | existing | PR A.3: delete `bump_invalidation` calls (lines 173, 195, 235). |
| `crates/objects/src/runtime_storage.rs` | existing | PR A.3: delete `ObjectRuntime::bump_invalidation` method. |
| `crates/objects/src/runtime.rs` | existing | PR A.3: delete `next_invalidation_epoch` field. |
| `crates/objects/src/shapes.rs` | existing | PR A.3: delete `InvalidationCause` enum. |
| `crates/gc/src/arena/records.rs` | existing | PR A.3: delete `last_invalidation_epoch` field + `last_invalidation_epoch()` getter + `mut_store_object_invalidation_epoch` mutator. |
| `crates/vm/src/vm/tiering.rs` | existing | PR A.3: add `warmup_counter: u16` to `TieringState`. |
| `crates/vm/src/tests/inline_caches.rs` | existing | Add A2, A3, A4, A5 tests across PRs. Update existing tests for shape-only fast path. |
| `crates/objects/src/tests.rs` | existing | Update `redefine_delete_and_prototype_mutation_bump_invalidation_epochs` to drop epoch assertions; retain shape-change assertions from Spec 1. |

---

# PR A.1 — `AdaptiveProtoLoad` observer + slow-path registration + generation field

**Goal:** Land the `AdaptiveProtoLoad` watchpoint observer kind with both halves of the contract wired (registration in slow path, dispatch in `Agent::fire_watchpoints_for_shape`). Epochs remain live; this PR is purely additive at the IC layer.

---

### Task A.1.1: Refactor `ObjectRuntime::fire_watchpoints_for_shape` → drain-only

**Why:** Spec 2's `AdaptiveProtoLoad` dispatch needs `&mut Vm` access (`Vm::clear_ic_slot_if_generation_matches`). The current `ObjectRuntime::fire_watchpoints_for_shape` dispatches `Recording` directly without giving the caller a chance to handle other observer kinds. Lift the dispatch one layer up — `ObjectRuntime` drains only, `Agent` dispatches all kinds.

**Files:**
- Modify: `crates/objects/src/runtime.rs:1230-1254`
- Modify: `crates/env/src/agent.rs:453-455`

- [ ] **Step 1: Replace `ObjectRuntime::fire_watchpoints_for_shape` body with a drain-only implementation**

In `crates/objects/src/runtime.rs`, replace lines 1230-1254 with:

```rust
/// Drains the watchpoint list for `shape` and marks its set `Invalidated`.
/// Returns the drained watchpoints for the caller to dispatch.
///
/// Splits dispatch responsibility: `ObjectRuntime` owns the side-table
/// (`watchpoint_sets`) and the test sink (`recording_watchpoint_fires`),
/// but Spec 2 observer kinds (`AdaptiveProtoLoad`) need `&mut Agent`/`&mut Vm`
/// for their fire effects, which only the `Agent` wrapper can provide.
/// `Agent::fire_watchpoints_for_shape` is the dispatch site for all kinds.
pub fn drain_watchpoints_for_shape(&mut self, shape: ShapeId) -> Option<Vec<Watchpoint>> {
    self.watchpoint_sets
        .get_mut(&shape)
        .and_then(|s| s.drain_for_fire())
}
```

- [ ] **Step 2: Add `pub fn push_recording_fire(&mut self, token: u64)` on `ObjectRuntime`**

`recording_watchpoint_fires` is `pub(crate)` on `ObjectRuntime` ([crates/objects/src/runtime.rs:66](crates/objects/src/runtime.rs#L66)), so the Agent (different crate) can't write to it directly. Add a public push method alongside the existing `take_recording_fires` / `watchpoint_sets_inspect` test-helper methods on `ObjectRuntime`:

```rust
/// Dispatch target for the `Recording` observer kind from the Agent layer.
/// In production this is unreachable (no caller constructs `Recording`); in
/// tests it accumulates tokens that `take_recording_fires` later drains.
pub fn push_recording_fire(&mut self, token: u64) {
    self.recording_watchpoint_fires.push(token);
}
```

- [ ] **Step 3: Update `Agent::fire_watchpoints_for_shape` to call the renamed method and dispatch all kinds**

In `crates/env/src/agent.rs:453-455`, replace with:

```rust
pub fn fire_watchpoints_for_shape(&mut self, shape: ShapeId) {
    let Some(fired) = self.objects.drain_watchpoints_for_shape(shape) else {
        return;
    };
    for wp in fired {
        match wp {
            Watchpoint::ShapeInvalidation { observer } => match observer {
                ShapeInvalidationObserver::Recording { token } => {
                    self.objects.push_recording_fire(token);
                }
            },
        }
    }
}
```

Confirm the necessary imports are present at the top of `agent.rs`:
```rust
use lyng_objects::{ShapeInvalidationObserver, Watchpoint};
```

- [ ] **Step 4: Run** `cargo check -p lyng-env -p lyng-objects` **and resolve any compile errors.**

Expected: clean build. The four existing `self.fire_watchpoints_for_shape(...)` callers in `agent.rs` (lines 302, 329, 349, 438, 482) continue to work — same method name and signature.

- [ ] **Step 5: Run** `cargo test --workspace` **and verify all Spec 1 tests still pass.**

Expected: all green. The behavioral contract is unchanged — Spec 1's `Recording` dispatch still works; only the layering moved.

- [ ] **Step 6: Commit**

```bash
git add crates/objects/src/runtime.rs crates/env/src/agent.rs
git commit -m "$(cat <<'EOF'
objects/agent: lift watchpoint dispatch into Agent layer

Renames ObjectRuntime::fire_watchpoints_for_shape to
drain_watchpoints_for_shape (drain-only). Agent::fire_watchpoints_for_shape
becomes the single dispatch site, so Spec 2's AdaptiveProtoLoad observer
can call into &mut Vm without violating Rust borrow rules.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

### Task A.1.2: Add `AdaptiveProtoLoad` variant to `ShapeInvalidationObserver`

**Files:**
- Modify: `crates/objects/src/watchpoint.rs:18-25` (the enum)
- Modify: `crates/objects/src/watchpoint.rs:31-36` (the `fire_into` dispatch)

- [ ] **Step 1: Add the variant**

In `crates/objects/src/watchpoint.rs`, replace lines 18-25 with:

```rust
pub enum ShapeInvalidationObserver {
    /// Records the fire event into a `Vec<u64>` so unit tests can
    /// assert "this transition fired exactly the watchpoints I registered."
    /// Carries no heap roots. Always present in production builds; production code
    /// never constructs this variant, so `ObjectRuntime::recording_watchpoint_fires`
    /// stays empty at runtime (24-byte overhead).
    Recording { token: u64 },
    /// Production consumer (Spec 2). Identifies an IC slot to clear when a
    /// depended-on shape transitions. `generation` guards against stale
    /// watchpoints firing on a slot that has been re-cached since registration.
    AdaptiveProtoLoad {
        code: CodeRef,
        slot: FeedbackSlotId,
        generation: u32,
    },
}
```

Add the necessary imports at the top of `watchpoint.rs`:
```rust
use lyng_types::{CodeRef, FeedbackSlotId};
```

- [ ] **Step 2: Extend the test-only `fire_into` dispatch**

In `crates/objects/src/watchpoint.rs:31-36`, replace with:

```rust
#[cfg(test)]
pub(crate) fn fire_into(&self, sink: &mut Vec<u64>) {
    match self {
        Self::Recording { token } => sink.push(*token),
        // AdaptiveProtoLoad is a production observer; in tests it should not
        // be dispatched through this sink. The Agent-layer dispatcher routes
        // it to Vm::clear_ic_slot_if_generation_matches.
        Self::AdaptiveProtoLoad { .. } => {}
    }
}
```

- [ ] **Step 3: Update `Agent::fire_watchpoints_for_shape` dispatch loop to handle the new variant (skeleton — the Vm-side method lands in Task A.1.4)**

In `crates/env/src/agent.rs:453-466` (the method just rewritten in Task A.1.1), replace the inner match arm with:

```rust
pub fn fire_watchpoints_for_shape(&mut self, shape: ShapeId) {
    let Some(fired) = self.objects.drain_watchpoints_for_shape(shape) else {
        return;
    };
    for wp in fired {
        match wp {
            Watchpoint::ShapeInvalidation { observer } => match observer {
                ShapeInvalidationObserver::Recording { token } => {
                    self.objects.recording_watchpoint_fires.push(token);
                }
                ShapeInvalidationObserver::AdaptiveProtoLoad {
                    code,
                    slot,
                    generation,
                } => {
                    self.vm.clear_ic_slot_if_generation_matches(code, slot, generation);
                }
            },
        }
    }
}
```

(The `self.vm.clear_ic_slot_if_generation_matches(...)` call won't compile yet — Task A.1.4 adds it. This is intentional sequencing.)

- [ ] **Step 4: Run** `cargo check -p lyng-objects` **— expect green (objects-level only).**

Run: `cargo check -p lyng-env` — expect FAIL with `no method named clear_ic_slot_if_generation_matches found for type Vm`. This is expected; Task A.1.4 lands the method.

- [ ] **Step 5: Commit (red — partial state, will be fixed next task)**

Don't commit yet — wait until Task A.1.4 makes the workspace build again. The plan executor should keep these two tasks together as one logical unit.

---

### Task A.1.3: Inspect `CodeRef` and `FeedbackSlotId` types

**Why:** Need to confirm `CodeRef` and `FeedbackSlotId` exist in `lyng_types` and have the shape `AdaptiveProtoLoad` needs (`Clone + Eq + Debug`).

**Files:**
- Read-only: `crates/lyng-types/src/lib.rs` or wherever these types are defined.

- [ ] **Step 1: Locate the types**

Run:
```bash
grep -rn "pub struct CodeRef\|pub struct FeedbackSlotId\|pub type FeedbackSlotId\|pub type CodeRef" /Users/sondre/dev/lyng/crates/lyng-types/ /Users/sondre/dev/lyng/crates/objects/ 2>/dev/null
```

- [ ] **Step 2: Verify `CodeRef` is `Clone + Debug + PartialEq + Eq`**

If not, derive them. `AdaptiveProtoLoad` lives inside `Watchpoint` which derives `Debug, PartialEq, Eq`, so its fields must support those.

- [ ] **Step 3: Verify `FeedbackSlotId` is `Copy + Debug + PartialEq + Eq + Hash`**

If `FeedbackSlotId` is not yet a typed `pub struct FeedbackSlotId(pub u32)`, look for the existing type used in `crates/vm/src/vm/feedback.rs` (likely `pub(crate) struct FeedbackSlotId(pub(crate) u32)` or similar). If only an internal type exists, promote it to `pub` in a `lyng-types`-accessible location, or relocate the watchpoint observer's slot field to use whatever public alias exists.

This is a discovery step. If discrepancies surface, fix them by:
- Promoting `FeedbackSlotId` to `pub` in its current home.
- Re-exporting from `lyng-types` if needed.
- Confirming `CodeRef` derives.

No code changes yet — this task surfaces the dependency.

- [ ] **Step 4: Commit any small visibility/derive fixes if needed**

```bash
git add crates/lyng-types/ crates/vm/src/vm/feedback.rs
git commit -m "types: expose CodeRef/FeedbackSlotId for cross-crate use in watchpoint observer"
```

Skip if no changes were needed.

---

### Task A.1.4: Add `Vm::clear_ic_slot_if_generation_matches`

**Files:**
- Modify: `crates/vm/src/vm.rs` (add the method to the `impl Vm` block)
- Modify: `crates/vm/src/vm/feedback.rs` (add a generation accessor on the per-site state, returning a placeholder `0` for now — the actual generation field lands in Task A.1.5)

- [ ] **Step 1: Add `generation()` and `bump_generation()` placeholder accessors to `FeedbackSiteState`**

In `crates/vm/src/vm/feedback.rs`, find the `FeedbackSiteState` enum (near line 894 per recon). Add methods to its impl block:

```rust
impl FeedbackSiteState {
    // ...existing methods...

    /// Returns the per-slot install generation. Bumped on every install /
    /// re-install / clear; `AdaptiveProtoLoad` watchpoints carry the
    /// generation they were registered at and no-op on mismatch.
    /// Placeholder returns 0 until Task A.1.5 lands the field.
    pub(super) const fn generation(&self) -> u32 {
        0
    }

    /// Increments and returns the new generation. Called from the slow path
    /// before installing a new cache entry. Placeholder no-op until A.1.5.
    pub(super) fn bump_generation(&mut self) -> u32 {
        0
    }

    /// Clears the cache state (transitions to Uninitialized).
    /// `AdaptiveProtoLoad` fire dispatches through this after a generation match.
    pub(super) fn clear(&mut self) {
        *self = FeedbackSiteState::Uninitialized;
    }
}
```

(Adjust based on the existing `FeedbackSiteState` definition; the recon indicated the variants are `Uninitialized`, `Arithmetic`, `Comparison`, `NamedProperty(...)`, `KeyedProperty`, `Call`, `Construct`. Wire `clear` to set to `Uninitialized`.)

- [ ] **Step 2: Add `Vm::clear_ic_slot_if_generation_matches`**

In `crates/vm/src/vm.rs`, add to the `impl Vm` block:

```rust
/// Spec 2 Phase A: dispatched from `Agent::fire_watchpoints_for_shape` when
/// an `AdaptiveProtoLoad` observer fires. Clears the IC slot if its current
/// generation matches the watchpoint's. Stale watchpoints from prior installs
/// no-op; the slot stays cached against whatever it currently holds.
pub(crate) fn clear_ic_slot_if_generation_matches(
    &mut self,
    code: lyng_types::CodeRef,
    slot: lyng_types::FeedbackSlotId,
    expected_generation: u32,
) {
    let Some(vector) = self.feedback_vectors.get_mut(code_index(code)) else {
        return;
    };
    let Some(site) = vector.site_mut(slot) else {
        return;
    };
    if site.generation() != expected_generation {
        return;
    }
    site.clear();
    site.bump_generation();
    self.mirror_flat_slot(code, slot);
}
```

(`code_index` is the existing helper at `crates/vm/src/vm.rs`; `feedback_vectors` is the `Vec<FeedbackVector>` field.)

- [ ] **Step 3: Run** `cargo build --workspace` **— expect green.** This is the first compile-clean point after Task A.1.2.

- [ ] **Step 4: Run** `cargo test --workspace` **— all Spec 1 tests still pass.**

- [ ] **Step 5: Commit the four-task unit (A.1.1 already committed, A.1.2 + A.1.3 + A.1.4 land together)**

```bash
git add crates/objects/src/watchpoint.rs crates/env/src/agent.rs \
        crates/vm/src/vm.rs crates/vm/src/vm/feedback.rs
git commit -m "$(cat <<'EOF'
watchpoint: add AdaptiveProtoLoad observer + Vm dispatch shell

Adds the ShapeInvalidationObserver::AdaptiveProtoLoad variant carrying
(code, slot, generation). Agent::fire_watchpoints_for_shape dispatches it
through Vm::clear_ic_slot_if_generation_matches. Generation accessors are
placeholders returning 0; the actual u32 field lands in the next commit.

No production caller registers AdaptiveProtoLoad yet, so the dispatch path
is exercised by tests only in this commit. The slow-path registration in
property_cache.rs lands in Task A.1.6.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

### Task A.1.5: Add `generation: u32` field to `FeedbackEntry` and `NamedPropertyFeedback`

**Files:**
- Modify: `crates/vm/src/dsl/feedback_flat.rs:70-86` (the `FeedbackEntry` struct)
- Modify: `crates/vm/src/vm/feedback.rs` (add `generation: u32` to `NamedPropertyFeedback` and wire `generation()` / `bump_generation()` to read/write it)

- [ ] **Step 1: Update `FeedbackEntry` to include `generation`**

In `crates/vm/src/dsl/feedback_flat.rs`, replace lines 70-86 with:

```rust
/// Single feedback entry. Pointer-stable for the lifetime of the
/// owning `InstalledFunction`.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeedbackEntry {
    pub(crate) mode: u8,
    pub(crate) _pad_a: [u8; 3],
    /// Spec 2 Phase A: per-IC install generation. Bumped on every install /
    /// re-install / clear. `AdaptiveProtoLoad` watchpoints carry the generation
    /// they were registered at and no-op on mismatch.
    pub(crate) generation: u32,
    /// `OwnData` mode: `NamedPropertyHandler::bits()`.
    /// `PrototypeData` mode: `NamedPropertyProtoHandler::proto_word()`.
    pub(crate) named_handler_bits: u64,
    /// `OwnData` mode: receiver invalidation epoch.
    /// `PrototypeData` mode: receiver invalidation epoch.
    /// (Removed in PR A.2; Phase A retains the field during transitional dual-check.)
    pub(crate) named_epoch: u64,
    /// `PrototypeData` mode: `NamedPropertyProtoHandler::receiver_word()`.
    pub(crate) named_aux_bits: u64,
    /// `PrototypeData` mode: prototype invalidation epoch.
    /// (Removed in PR A.2.)
    pub(crate) named_aux_epoch: u64,
    pub(crate) scalar_observed_bits: u32,
    pub(crate) scalar_execution_count: u32,
    pub(crate) _tail_pad: [u8; 16],
}
```

Total size: `1 + 3 + 4 + 8 + 8 + 8 + 8 + 4 + 4 + 16 = 64` bytes. Stride unchanged. The 4 bytes for `generation` come from the existing 7-byte `_pad` (now `_pad_a: [u8; 3]`).

- [ ] **Step 2: Add `generation` to `NamedPropertyFeedback`**

In `crates/vm/src/vm/feedback.rs`, locate `pub(crate) struct NamedPropertyFeedback` (around line 687). Add the field:

```rust
pub(crate) struct NamedPropertyFeedback {
    pub(crate) entry_count: u8,
    pub(crate) entries: [Option<NamedPropertyCacheEntry>; 8],
    pub(crate) polymorphic_own_data_handlers: [NamedPropertyHandler; POLY_LIMIT],
    pub(crate) polymorphic_own_data_epochs: [u64; POLY_LIMIT],
    /// Spec 2 Phase A: per-site install generation.
    pub(crate) generation: u32,
}
```

Initialize `generation: 0` in any `NamedPropertyFeedback::new()` / `Default::default()` calls. Search for `NamedPropertyFeedback {` and fix every constructor site:

```bash
rg "NamedPropertyFeedback \{" /Users/sondre/dev/lyng/crates/vm/src/vm/feedback.rs
```

- [ ] **Step 3: Replace placeholder `generation()` / `bump_generation()` on `FeedbackSiteState`**

In `crates/vm/src/vm/feedback.rs` (the methods added in Task A.1.4 step 1), replace:

```rust
pub(super) const fn generation(&self) -> u32 {
    match self {
        FeedbackSiteState::NamedProperty(named) => named.generation,
        _ => 0, // Other kinds don't carry generations (Spec 2 extends per-kind in Phase C).
    }
}

pub(super) fn bump_generation(&mut self) -> u32 {
    match self {
        FeedbackSiteState::NamedProperty(named) => {
            named.generation = named.generation.wrapping_add(1);
            named.generation
        }
        _ => 0,
    }
}
```

`wrapping_add` is intentional — generation is u32 and wraparound is acceptable per spec §8.2.

- [ ] **Step 4: Update `mirror_flat_slot` to write `generation` into `FeedbackEntry`**

In `crates/vm/src/vm/feedback.rs:2152-2204`, modify the function to read `feedback.generation` (where `feedback` is the `NamedPropertyFeedback` source) and write it onto the flat entry. Pattern:

```rust
#[inline]
fn mirror_flat_slot(&mut self, code: CodeRef, slot: FeedbackSlotId) {
    let index = code_index(code);
    let (header, generation) = self
        .feedback_vectors
        .get(index)
        .and_then(|vector| vector.site(slot))
        .map(|site| {
            let header = Self::named_llint_load_header(site);
            let gen = site.generation();
            (header, gen)
        })
        .unwrap_or((None, 0));
    let Some(slot_index) = Self::flat_feedback_slot_index(slot) else {
        return;
    };
    let Some(entry) = self
        .feedback_flat_storage
        .get_mut(index)
        .and_then(|entries| entries.get_mut(slot_index))
    else {
        return;
    };
    entry.clear_ic_header();
    entry.generation = generation;  // NEW

    match header {
        // ...existing match arms unchanged...
    }
}
```

Confirm `entry.clear_ic_header()` doesn't zero the new `generation` field (check the `clear_ic_header` impl in `feedback_flat.rs`; if it does, exclude `generation` from the zeroing).

- [ ] **Step 5: Run** `cargo build --workspace` **and** `cargo test --workspace` **— expect green.**

- [ ] **Step 6: Commit**

```bash
git add crates/vm/src/dsl/feedback_flat.rs crates/vm/src/vm/feedback.rs
git commit -m "$(cat <<'EOF'
feedback: add per-IC generation field (u32) to FeedbackEntry + NamedPropertyFeedback

Generation bumps on every install/re-install/clear. AdaptiveProtoLoad
watchpoints carry the generation at registration time and no-op when the
slot has been re-cached since (Spec 2 §3.2, §8.2). 4 bytes reclaimed from
the existing 7-byte _pad; total FeedbackEntry size unchanged at 64B.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

### Task A.1.6: Slow-path registers `AdaptiveProtoLoad` on chain shapes

**Why:** With the observer variant, dispatch path, and generation field in place, the slow path can now register watchpoints when it installs a proto-chain cache entry. Spec §3.3: register on every shape in the chain *except* the receiver (the receiver shape is caught by the fast-path shape compare).

**Files:**
- Modify: `crates/objects/src/internal_methods/property_cache.rs` (slow-path install — needs the chain-shape list + access to `Agent` to register)

The slow path currently lives at `property_cache.rs` and is called via `push_property_cache_dependency` (lines 570-593). The dependency walk happens at lines 35, 79, 283, 300 — these are the four install sites for different lookup kinds (own, proto-chain, etc.).

This task requires more context-finding work than the previous ones. The implementer must:

1. Identify which of the four `push_property_cache_dependency` callsites correspond to proto-chain installs (vs own-data installs which don't need `AdaptiveProtoLoad`).
2. Find a way to register watchpoints from the slow path — `Agent::register_adaptive_proto_load` wrapper.
3. Decide where to bump generation in the install flow.

- [ ] **Step 1: Locate the proto-chain install site**

Run:
```bash
sed -n '20,100p' /Users/sondre/dev/lyng/crates/objects/src/internal_methods/property_cache.rs
sed -n '270,320p' /Users/sondre/dev/lyng/crates/objects/src/internal_methods/property_cache.rs
```

Identify the function(s) responsible for installing the proto-data IC handler. Look for names like `try_install_proto_handler`, `install_named_property_proto_data`, or similar. The recon indicated lines 283 and 300 are proto-related (the own/proto fork happens earlier in the call stack).

- [ ] **Step 2a: Add `Vm::feedback_vector_mut` helper if absent**

Confirm whether `Vm::feedback_vector_mut(&mut self, code: CodeRef) -> Option<&mut FeedbackVector>` exists:

```bash
grep -n "feedback_vector_mut\|fn feedback_vector" /Users/sondre/dev/lyng/crates/vm/src/vm.rs /Users/sondre/dev/lyng/crates/vm/src/vm/feedback.rs
```

If absent, add it alongside the existing `feedback_vectors` field access pattern. The implementation matches what `Vm::clear_ic_slot_if_generation_matches` (Task A.1.4) already does inline:

```rust
// In impl Vm (crates/vm/src/vm.rs):
pub(crate) fn feedback_vector_mut(&mut self, code: lyng_types::CodeRef) -> Option<&mut FeedbackVector> {
    self.feedback_vectors.get_mut(code_index(code))
}
```

(`FeedbackVector` is `pub(super)` — if the visibility doesn't reach into the `env` crate, either keep the body inlined at the Agent layer using `self.vm.feedback_vectors.get_mut(code_index(code))`, or promote `FeedbackVector` to `pub(crate)`. Choose the inlining if the visibility promotion has wider implications; this is a one-call-site helper.)

- [ ] **Step 2b: Add `Agent::register_adaptive_proto_load_for_chain`**

In `crates/env/src/agent.rs`, add (alongside `fire_watchpoints_for_shape`):

```rust
/// Spec 2 Phase A: registers `AdaptiveProtoLoad` watchpoints on each shape
/// in a proto-cache's dependency chain (excluding the receiver, which is
/// covered by the IC fast-path shape compare). Returns `Err(())` if any
/// shape is already `Invalidated`, signaling the slow path to abandon the
/// install. The slot's generation is bumped exactly once on success.
pub fn register_adaptive_proto_load_for_chain(
    &mut self,
    code: lyng_types::CodeRef,
    slot: lyng_types::FeedbackSlotId,
    chain_shapes: &[lyng_types::ShapeId],
) -> Result<u32, ()> {
    let generation = match self.vm.feedback_vector_mut(code.clone()) {
        Some(vector) => match vector.site_mut(slot) {
            Some(site) => site.bump_generation(),
            None => return Err(()),
        },
        None => return Err(()),
    };

    for &shape in chain_shapes {
        let result = self.objects.watchpoint_set_mut(shape).register(
            lyng_objects::Watchpoint::ShapeInvalidation {
                observer: lyng_objects::ShapeInvalidationObserver::AdaptiveProtoLoad {
                    code: code.clone(),
                    slot,
                    generation,
                },
            },
        );
        if result.is_err() {
            // Abandon: clear the slot we just bumped so a future install proceeds clean.
            if let Some(vector) = self.vm.feedback_vector_mut(code.clone()) {
                if let Some(site) = vector.site_mut(slot) {
                    site.clear();
                    site.bump_generation();
                }
            }
            return Err(());
        }
    }

    Ok(generation)
}
```

(Method `site_mut(slot)` already exists on `FeedbackVector` per recon; verify with `grep -n "fn site_mut" crates/vm/src/vm/feedback.rs`.)

- [ ] **Step 3: Wire the slow-path call site**

In `crates/objects/src/internal_methods/property_cache.rs`, in the proto-chain install function:

After `push_property_cache_dependency` has populated the dependency list (which carries the chain shapes), call `agent.register_adaptive_proto_load_for_chain(code, slot, &chain_shapes_excluding_receiver)`. On `Err(())`, abandon the install — do not write the IC handler.

The exact shape of "exclude the receiver" depends on where the receiver appears in the dependency list:
- If dependencies[0] is always the receiver, slice as `&dependencies[1..count]`.
- If receiver and holder are tagged differently, filter accordingly.

Reading the install path's existing logic (Step 1's reads) will tell you. The plan's commitment is: **chain shapes excluding receiver are passed to `register_adaptive_proto_load_for_chain`**.

- [ ] **Step 4: Run** `cargo build --workspace` **— expect green.**

- [ ] **Step 5: Add Task A.1.7's failing tests first; then run, verify the test fails the way the new wiring should fix.**

Skip to Task A.1.7 for the tests, then return here to commit both this task and the tests as one logical unit.

---

### Task A.1.7: Tests A.4 + A.5 — generation guard + abandon-on-Invalidated

**Files:**
- Modify: `crates/vm/src/tests/inline_caches.rs`

- [ ] **Step 1: Write test A.4 — generation guard rejects stale watchpoint fire**

Append to `crates/vm/src/tests/inline_caches.rs`:

```rust
#[test]
fn adaptive_proto_load_generation_guard_rejects_stale_fire() {
    // Setup: install a proto-cache IC entry; capture the chain shape S₁
    // that AdaptiveProtoLoad was registered on. Then clear + re-install
    // against a different chain (bumps generation). Finally, transition S₁
    // and verify the stale AdaptiveProtoLoad fires but the slot stays
    // cached against the new chain (no clear).
    let (mut agent, code, slot) = build_proto_cache_test_fixture();
    let original_chain_shape = chain_shape_for_slot(&agent, code, slot);

    // First-stage install: implicit during build_proto_cache_test_fixture.
    let original_generation = ic_generation(&agent, code, slot);
    assert!(ic_is_cached(&agent, code, slot));

    // Force a clear (e.g., by mutating the chain) then re-install against a
    // *different* chain. Generation bumps; original watchpoint goes orphan.
    rebuild_with_different_chain(&mut agent, code, slot);
    let new_generation = ic_generation(&agent, code, slot);
    assert_ne!(original_generation, new_generation);
    assert!(ic_is_cached(&agent, code, slot));

    // Transition the ORIGINAL chain shape; the orphan AdaptiveProtoLoad fires.
    transition_shape(&mut agent, original_chain_shape);

    // Assertion: slot still cached (generation mismatch caused fire to no-op).
    assert!(ic_is_cached(&agent, code, slot));
    assert_eq!(ic_generation(&agent, code, slot), new_generation);
}
```

(The helpers `build_proto_cache_test_fixture`, `chain_shape_for_slot`, `ic_generation`, `ic_is_cached`, `rebuild_with_different_chain`, `transition_shape` need to be implemented or located in the existing test infrastructure. If they don't exist, prefix-search `crates/vm/src/tests/inline_caches.rs` for similar helpers and adapt.)

- [ ] **Step 2: Write test A.5 — register-on-invalidated abandons install**

```rust
#[test]
fn proto_cache_install_abandons_on_invalidated_chain_shape() {
    // Setup: build a proto-chain scenario. Pre-invalidate one shape in the
    // chain BEFORE the IC install attempt. The slow-path install should
    // see Err(()) from register_adaptive_proto_load_for_chain and leave
    // the slot uncached.
    let (mut agent, code, slot, mid_chain_shape) = build_unprimed_proto_cache_fixture();

    // Pre-invalidate the middle proto shape.
    agent.objects_mut().watchpoint_set_mut(mid_chain_shape).register(
        lyng_objects::Watchpoint::ShapeInvalidation {
            observer: lyng_objects::ShapeInvalidationObserver::Recording { token: 0 },
        },
    ).unwrap();
    agent.fire_watchpoints_for_shape(mid_chain_shape);
    assert_eq!(
        agent.objects().watchpoint_sets_inspect(mid_chain_shape).unwrap().state(),
        lyng_objects::WatchpointState::Invalidated,
    );

    // Now do the IC read that would normally install a proto-cache.
    execute_proto_load(&mut agent, code, slot);

    // Slot stays uncached because register_adaptive_proto_load_for_chain returned Err.
    assert!(!ic_is_cached(&agent, code, slot));
}
```

- [ ] **Step 3: Run** `cargo test -p lyng-vm adaptive_proto_load_generation_guard_rejects_stale_fire proto_cache_install_abandons_on_invalidated_chain_shape` **— expect PASS.**

If they FAIL, the slow-path wiring from Task A.1.6 needs revisiting. Iterate.

- [ ] **Step 4: Run** `cargo test --workspace` **— full suite green.**

- [ ] **Step 5: Commit (Task A.1.6 + A.1.7 together)**

```bash
git add crates/objects/src/internal_methods/property_cache.rs \
        crates/env/src/agent.rs \
        crates/vm/src/tests/inline_caches.rs
git commit -m "$(cat <<'EOF'
property_cache: slow path registers AdaptiveProtoLoad on proto-chain shapes

When the slow path installs a prototype-cache IC entry, register
AdaptiveProtoLoad watchpoints on each chain shape (excluding the receiver,
which the IC fast path already guards via shape compare). Abandon the install
if any shape is already Invalidated.

Adds tests A.4 (generation-guard rejects stale fires after re-install) and
A.5 (install abandons when a chain shape is pre-invalidated). Epochs still
live in this PR — Phase A.2 retires the IC fast-path epoch comparisons.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

PR A.1 done. Ready for review. Phase A.1 is purely additive: epoch comparisons still happen on the fast path; `AdaptiveProtoLoad` is wired but invalidation is effectively dual-signal until PR A.2 retires the epoch check.

---

# PR A.2 — IC fast-path drops epoch comparison + epoch fields removed

**Goal:** With `AdaptiveProtoLoad` registration in place, the IC fast path can shed its epoch comparison. Receiver-shape transitions are caught by Spec 1's shape changes; proto-chain mutations are caught by `AdaptiveProtoLoad` clearing the slot before the next fast-path read. Then drop `named_epoch` + `named_aux_epoch` from `FeedbackEntry`.

---

### Task A.2.1: Drop epoch comparison from monomorphic OwnData fast path

**Files:**
- Modify: `crates/vm/src/vm/dispatch/property.rs:108-132`

- [ ] **Step 1: Replace the epoch check with shape-only compare**

In `crates/vm/src/vm/dispatch/property.rs`, replace lines 108-132 with:

```rust
if let Some((handler, _cached_epoch)) =
    self.named_property_own_data_handler(frame.code(), feedback_slot)
{
    let heap_view = agent.heap().view();
    if let Some(record) = heap_view.object_ref(object)
        && record.shape() == handler.receiver_shape()
    {
        let cached_value = match handler.slot_location() {
            SlotLocation::Inline(index) => record.inline_named_slot(index as usize),
            SlotLocation::OutOfLine(offset) => record
                .named_slots()
                .and_then(|slots| heap_view.object_slots(slots))
                .and_then(|slots| slots.get(offset as usize).copied()),
        };
        if let Some(value) = cached_value {
            if let Some(slot) = feedback_slot {
                self.record_named_property_cache_hit(frame.code(), slot);
            }
            self.register_stack[target_index] = value;
            advance_dispatch_frame(frame, instruction_len);
            return Ok(());
        }
    }
}
```

The `_cached_epoch` is still destructured (silenced with `_`) because the helper's return signature hasn't changed yet. Task A.2.3 simplifies the helper.

- [ ] **Step 2: Run** `cargo test -p lyng-vm --test inline_caches` **— expect green.** The shape compare alone is sufficient because:
  - Receiver-shape mutations transition the shape (Spec 1).
  - Proto-chain mutations clear the slot via `AdaptiveProtoLoad` before this path reads it (PR A.1).

- [ ] **Step 3: Run** `cargo test --workspace` **— full suite green.**

- [ ] **Step 4: Commit**

```bash
git add crates/vm/src/vm/dispatch/property.rs
git commit -m "vm/dispatch/property: drop epoch compare from monomorphic OwnData IC fast path"
```

---

### Task A.2.2: Drop epoch comparison from PrototypeData + polymorphic OwnData paths

**Files:**
- Modify: `crates/vm/src/vm/dispatch/property.rs` (the `try_named_property_proto_data_load` and `try_named_property_polymorphic_own_data_load` helpers)

These helpers live near the same dispatch site. Find them:
```bash
grep -n "try_named_property_proto_data_load\|try_named_property_polymorphic_own_data_load" /Users/sondre/dev/lyng/crates/vm/src/vm/dispatch/property.rs /Users/sondre/dev/lyng/crates/vm/src/vm/feedback.rs
```

- [ ] **Step 1: Locate both helpers** and read their bodies. They check both receiver shape and epochs.

- [ ] **Step 2: Drop the epoch comparisons from both helpers**

For each helper, find the line that reads `record.last_invalidation_epoch()` and compares to a cached epoch (e.g., `cached_epoch == receiver_epoch`). Delete the AND-comparison, leaving the shape-equality check standing alone.

Example pattern (the exact code varies per helper):
```rust
// Before:
if record.shape() == cached_shape
    && record.last_invalidation_epoch().unwrap_or(0) == cached_receiver_epoch
{
    // ...
}

// After:
if record.shape() == cached_shape {
    // ...
}
```

Repeat for the prototype-side check in `try_named_property_proto_data_load` (the holder shape compare also drops its epoch counterpart).

- [ ] **Step 3: Run** `cargo test -p lyng-vm --test inline_caches` **— expect green.**

- [ ] **Step 4: Run** `cargo test --workspace` **— expect green.**

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm/dispatch/property.rs crates/vm/src/vm/feedback.rs
git commit -m "vm/dispatch/property: drop epoch compare from proto + polymorphic IC fast paths"
```

---

### Task A.2.3: Drop `named_epoch` + `named_aux_epoch` fields from `FeedbackEntry`

**Files:**
- Modify: `crates/vm/src/dsl/feedback_flat.rs:70-86`
- Modify: `crates/vm/src/dsl/feedback_flat.rs` — `set_named_*` methods (drop epoch params)
- Modify: `crates/vm/src/vm/feedback.rs` — `mirror_flat_slot` (drop epoch reads)

- [ ] **Step 1: Update `FeedbackEntry`**

In `crates/vm/src/dsl/feedback_flat.rs`, replace the struct definition with:

```rust
/// Single feedback entry. Pointer-stable for the lifetime of the
/// owning `InstalledFunction`.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeedbackEntry {
    pub(crate) mode: u8,
    pub(crate) _pad_a: [u8; 3],
    /// Spec 2 Phase A: per-IC install generation.
    pub(crate) generation: u32,
    /// `OwnData` mode: `NamedPropertyHandler::bits()`.
    /// `PrototypeData` mode: `NamedPropertyProtoHandler::proto_word()`.
    pub(crate) named_handler_bits: u64,
    /// `PrototypeData` mode: `NamedPropertyProtoHandler::receiver_word()`.
    pub(crate) named_aux_bits: u64,
    pub(crate) scalar_observed_bits: u32,
    pub(crate) scalar_execution_count: u32,
    pub(crate) _tail_pad: [u8; 32],
}
```

Size: `1 + 3 + 4 + 8 + 8 + 4 + 4 + 32 = 64` bytes. Stride preserved. Tail padding grew from 16B to 32B (reclaimed from the two 8B epoch fields).

- [ ] **Step 2: Update the `set_named_*` methods**

Find each `set_named_*_load` method on `FeedbackEntry`:
```bash
grep -n "pub fn set_named_" /Users/sondre/dev/lyng/crates/vm/src/dsl/feedback_flat.rs
```

For each one, delete the epoch parameters and assignments. Example:

Before:
```rust
pub fn set_named_own_inline_load(&mut self, handler_bits: u64, epoch: u64) {
    self.mode = NAMED_OWN_INLINE_MODE;
    self.named_handler_bits = handler_bits;
    self.named_epoch = epoch;
}
```

After:
```rust
pub fn set_named_own_inline_load(&mut self, handler_bits: u64) {
    self.mode = NAMED_OWN_INLINE_MODE;
    self.named_handler_bits = handler_bits;
}
```

Repeat for `set_named_own_outline_load`, `set_named_proto_inline_load`, `set_named_own_polymorphic`. Drop `epoch` / `receiver_epoch` / `prototype_epoch` / `slot0_epoch` / `slot1_epoch` parameters.

- [ ] **Step 3: Update `mirror_flat_slot`**

In `crates/vm/src/vm/feedback.rs:2152-2204`, drop the epoch fields from the `LlIntNamedPropertyHeader` destructures. The header type itself also drops its epoch fields (see Step 4).

```rust
match header {
    Some(LlIntNamedPropertyHeader::OwnInline { handler_bits }) =>
        entry.set_named_own_inline_load(handler_bits),
    Some(LlIntNamedPropertyHeader::OwnOutline { handler_bits }) =>
        entry.set_named_own_outline_load(handler_bits),
    Some(LlIntNamedPropertyHeader::ProtoInline { receiver_word, proto_word }) =>
        entry.set_named_proto_inline_load(receiver_word, proto_word),
    Some(LlIntNamedPropertyHeader::OwnPolymorphic {
        slot0_handler_bits,
        slot1_handler_bits,
    }) => entry.set_named_own_polymorphic(slot0_handler_bits, slot1_handler_bits),
    None => {}
}
```

- [ ] **Step 4: Update `LlIntNamedPropertyHeader`**

Find its definition (likely in `crates/vm/src/vm/feedback.rs` near `mirror_flat_slot`):
```bash
grep -n "enum LlIntNamedPropertyHeader" /Users/sondre/dev/lyng/crates/vm/src/vm/feedback.rs
```

Drop the `epoch` / `receiver_epoch` / `prototype_epoch` / `slot0_epoch` / `slot1_epoch` fields from every variant. Update `named_llint_load_header` (the projection function) to stop reading the epoch fields from `NamedPropertyFeedback` / `NamedPropertyCacheEntry`.

- [ ] **Step 5: Drop the `polymorphic_own_data_epochs` field from `NamedPropertyFeedback`**

In `crates/vm/src/vm/feedback.rs`, find `pub(crate) struct NamedPropertyFeedback` and delete:
```rust
polymorphic_own_data_epochs: [u64; POLY_LIMIT],
```

Fix all `NamedPropertyFeedback {` constructors. Fix any reads of `polymorphic_own_data_epochs`.

- [ ] **Step 6: Drop the helper return-tuple epoch from `named_property_own_data_handler`**

Find the helper:
```bash
grep -n "named_property_own_data_handler" /Users/sondre/dev/lyng/crates/vm/src/vm/feedback.rs
```

Change the return type from `Option<(Handler, Epoch)>` to `Option<Handler>`. Update the call site in `property.rs:108` to drop the `_cached_epoch` destructure (now `if let Some(handler) = ...`).

- [ ] **Step 7: Drop epoch field from `PropertyCacheDependency`**

In `crates/objects/src/internal_methods/property_cache.rs`, find `PropertyCacheDependency::new` (line 586) and `pub struct PropertyCacheDependency`. Drop the `invalidation_epoch: Option<u64>` field and the corresponding `invalidation_epoch()` accessor. Update `record_matches_cache_dependency` (lines 484-490) to drop the epoch comparison — shape-only.

```rust
#[inline]
fn record_matches_cache_dependency(
    record: RuntimeObjectRecord,
    dependency: PropertyCacheDependency,
) -> bool {
    record.shape() == Some(dependency.shape())
}
```

Update `push_property_cache_dependency` (lines 570-593) to stop reading `record.last_invalidation_epoch()`.

Update test sites in `crates/objects/src/tests.rs` (lines 283, 286, 400, 401, 434, 437, 440) that call `PropertyCacheDependency::new(holder, shape, None)` — drop the trailing `None` parameter.

- [ ] **Step 8: Run** `cargo build --workspace` **— iterate on compile errors until clean.**

- [ ] **Step 9: Add test A.2 — proto-chain holder mutation clears IC**

In `crates/vm/src/tests/inline_caches.rs`:

```rust
#[test]
fn proto_chain_holder_mutation_clears_ic_slot() {
    // obj inherits x from proto. Cache obj.x reading x at proto's offset.
    // Then write proto.x = newValue (a property *redefinition* on proto,
    // which transitions proto's shape). AdaptiveProtoLoad fires, IC clears,
    // next read re-caches against the new proto shape.
    let (mut agent, code, slot, obj, proto) = build_two_level_proto_cache();
    execute_named_load(&mut agent, code, slot, obj);  // primes IC
    assert!(ic_is_cached(&agent, code, slot));

    // Mutate proto.x — a property write triggers redefinition path.
    redefine_named_property(&mut agent, proto, "x", value_two());

    // The redefinition transitioned proto's shape; AdaptiveProtoLoad fired.
    assert!(!ic_is_cached(&agent, code, slot));

    // Next read re-caches.
    execute_named_load(&mut agent, code, slot, obj);
    assert!(ic_is_cached(&agent, code, slot));
}
```

- [ ] **Step 10: Add test A.3 — two-hop chain, middle mutation**

```rust
#[test]
fn two_hop_chain_middle_proto_mutation_clears_ic() {
    // obj → mid → root. Property x lives on root.
    // Mutate mid (property add) → mid's shape transitions → AdaptiveProtoLoad fires.
    let (mut agent, code, slot, obj, mid, root) = build_three_level_proto_cache();
    execute_named_load(&mut agent, code, slot, obj);
    assert!(ic_is_cached(&agent, code, slot));

    add_named_property(&mut agent, mid, "irrelevant", value_one());

    assert!(!ic_is_cached(&agent, code, slot));
}
```

- [ ] **Step 11: Run** `cargo test -p lyng-vm --test inline_caches proto_chain_holder_mutation_clears_ic_slot two_hop_chain_middle_proto_mutation_clears_ic` **— expect green.**

- [ ] **Step 12: Run** `cargo test --workspace` **— full suite green.**

- [ ] **Step 13: Commit**

```bash
git add crates/vm/src/dsl/feedback_flat.rs \
        crates/vm/src/vm/feedback.rs \
        crates/vm/src/vm/dispatch/property.rs \
        crates/objects/src/internal_methods/property_cache.rs \
        crates/objects/src/tests.rs \
        crates/vm/src/tests/inline_caches.rs
git commit -m "$(cat <<'EOF'
ic: drop named_epoch + named_aux_epoch from FeedbackEntry + dependency record

With AdaptiveProtoLoad watchpoints clearing slots on proto-chain transitions
(PR A.1), the IC fast path no longer needs epoch comparisons. Removes
named_epoch + named_aux_epoch from FeedbackEntry (16B reclaimed into tail
padding), polymorphic_own_data_epochs from NamedPropertyFeedback,
LlIntNamedPropertyHeader epoch fields, and PropertyCacheDependency's
invalidation_epoch.

Adds tests A.2 (single-hop proto holder mutation clears IC) and A.3 (two-hop
middle-proto mutation clears IC).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

PR A.2 done. IC fast path is now epoch-free. Per-object `last_invalidation_epoch` is still bumped at the 5 callsites but no longer read by ICs — PR A.3 retires the bumps.

---

# PR A.3 — Delete `bump_invalidation` infrastructure + `warmup_counter` lift

**Goal:** Now that no IC reads the per-object epoch, retire the entire bump infrastructure. Then lift `warmup_counter` from `FeedbackVector` to `TieringState`.

---

### Task A.3.1: Lift `warmup_counter` from `FeedbackVector` to `TieringState`

**Files:**
- Modify: `crates/vm/src/vm/tiering.rs:66-89` (`TieringState` struct + Default impl + snapshot)
- Modify: `crates/vm/src/vm/feedback.rs` (delete `warmup_counter` field + accessor + bump method on `FeedbackVector`)
- Modify: callers of `bump_warmup` / `warmup_counter` (find via grep)

- [ ] **Step 1: Add `warmup_counter` to `TieringState`**

In `crates/vm/src/vm/tiering.rs`, replace the struct (lines 66-74) with:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TieringState {
    eligible: bool,
    status: TierStatus,
    hotness: u32,
    feedback_events: u32,
    backedge_events: u32,
    invalidation_epoch: u32,
    native_generation: Option<NonZeroU32>,
    /// Spec 2 Phase A: pre-allocation execution counter. Bumped on each
    /// invocation before the feedback vector is allocated. When this hits
    /// `FEEDBACK_ALLOCATION_THRESHOLD` (=2), the feedback vector allocates.
    /// Lifted from `FeedbackVector::warmup_counter`.
    warmup_counter: u16,
}
```

Update `Default` impl (lines 76-89):
```rust
impl Default for TieringState {
    #[inline]
    fn default() -> Self {
        Self {
            eligible: false,
            status: TierStatus::InterpreterOnly,
            hotness: 0,
            feedback_events: 0,
            backedge_events: 0,
            invalidation_epoch: 0,
            native_generation: None,
            warmup_counter: 0,
        }
    }
}
```

Add methods to `TieringState`:
```rust
impl TieringState {
    // ...existing methods...

    #[inline]
    pub(super) const fn warmup_counter(&self) -> u16 {
        self.warmup_counter
    }

    #[inline]
    pub(super) fn bump_warmup(&mut self) -> u16 {
        self.warmup_counter = self.warmup_counter.saturating_add(1);
        self.warmup_counter
    }
}
```

- [ ] **Step 2: Expose getter/bumper on `Tiering`**

In the same file, add to `impl Tiering`:
```rust
pub(super) fn warmup_counter(&self, code: CodeRef) -> u16 {
    self.states
        .get(code_index(code))
        .and_then(Option::as_ref)
        .map_or(0, TieringState::warmup_counter)
}

pub(super) fn bump_warmup(&mut self, code: CodeRef) -> u16 {
    self.state_mut(code).bump_warmup()
}
```

(Use whatever existing pattern `Tiering` already follows for `state_mut(code)`-style entry helpers. If none, lazily insert a `Default` `TieringState` at `code_index(code)`.)

- [ ] **Step 3: Move call sites**

Find all callers of `FeedbackVector::bump_warmup()` / `warmup_counter()`:
```bash
grep -rn "bump_warmup\|warmup_counter" /Users/sondre/dev/lyng/crates/
```

Replace each `vm.feedback_vector(code).warmup_counter()` with `vm.tiering.warmup_counter(code)`, and `vm.feedback_vector_mut(code).bump_warmup()` with `vm.tiering.bump_warmup(code)`. The `FEEDBACK_ALLOCATION_THRESHOLD` constant (`crates/vm/src/vm/feedback.rs:19`) stays where it is — the threshold check now reads `tiering.warmup_counter(code)`.

- [ ] **Step 4: Delete `warmup_counter` from `FeedbackVector`**

In `crates/vm/src/vm/feedback.rs:2043-2046`:
```rust
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct FeedbackVector {
    sites: Vec<Option<FeedbackSiteState>>,
}
```

Delete the `bump_warmup` and `warmup_counter` methods on `FeedbackVector` (lines 2073-2083). Update any `FeedbackVector { ... }` constructors that initialized `warmup_counter: 0`.

- [ ] **Step 5: Update `FeedbackVectorSnapshot::warmup_counter`**

`FeedbackVectorSnapshot` (in `crates/vm/src/vm/feedback.rs` around line 553-603) has a `warmup_counter: u16` field. Two options:

(a) Keep the snapshot field, populate it from `Tiering` at snapshot time.
(b) Move the field to a new `TieringSnapshot` (already exists at `tiering.rs:17-26`).

Choose (a) for Phase A — minimal churn. The snapshot is going away in Phase E anyway. Update `feedback_vector_snapshot(code)` to read warmup from `Tiering`:

```rust
// inside feedback_vector_snapshot or wherever the snapshot is built:
let warmup_counter = self.tiering.warmup_counter(code);
// ... populate snapshot ...
```

- [ ] **Step 6: Run** `cargo build --workspace` **— iterate.**

- [ ] **Step 7: Add test A.6 — tier-up still works**

In `crates/vm/src/tests/feedback.rs` (or wherever tier-up tests live), confirm the allocation-threshold test:

```rust
#[test]
fn warmup_counter_after_lift_still_triggers_allocation_at_threshold() {
    // Setup: install a function. First two calls don't allocate the feedback
    // vector. Third call triggers allocation.
    let (mut agent, code) = build_unallocated_function();
    assert!(!agent.vm().feedback_vector_allocated(code));

    // First two calls bump warmup.
    invoke_function(&mut agent, code);
    assert_eq!(agent.vm().tiering.warmup_counter(code), 1);
    assert!(!agent.vm().feedback_vector_allocated(code));

    invoke_function(&mut agent, code);
    assert_eq!(agent.vm().tiering.warmup_counter(code), 2);

    // Third call — threshold met → vector allocated.
    invoke_function(&mut agent, code);
    assert!(agent.vm().feedback_vector_allocated(code));
}
```

Adapt the helper names to match what exists in `crates/vm/src/tests/feedback.rs`. The intent: the existing tier-up tests should already cover this via existing assertions; this test exists as a sanity guard around the lift specifically.

- [ ] **Step 8: Run** `cargo test --workspace` **— green.**

- [ ] **Step 9: Commit**

```bash
git add crates/vm/src/vm/tiering.rs crates/vm/src/vm/feedback.rs crates/vm/src/tests/feedback.rs
git commit -m "$(cat <<'EOF'
vm/tiering: lift warmup_counter from FeedbackVector to TieringState

warmup_counter is per-code allocation hysteresis (threshold = 2). Moving it
onto TieringState consolidates per-code state on Tiering and removes a
field FeedbackVector no longer needs. Snapshot.warmup_counter is populated
from Tiering at snapshot time (no public-API break).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

### Task A.3.2: Delete `bump_invalidation` callsites

**Files:**
- Modify: `crates/objects/src/internal_methods.rs` (lines 425, 460)
- Modify: `crates/objects/src/internal_methods/named_properties.rs` (lines 173, 195, 235)

- [ ] **Step 1: Read each callsite to understand context**

Run:
```bash
sed -n '420,430p' /Users/sondre/dev/lyng/crates/objects/src/internal_methods.rs
sed -n '455,465p' /Users/sondre/dev/lyng/crates/objects/src/internal_methods.rs
sed -n '168,178p' /Users/sondre/dev/lyng/crates/objects/src/internal_methods/named_properties.rs
sed -n '190,200p' /Users/sondre/dev/lyng/crates/objects/src/internal_methods/named_properties.rs
sed -n '230,240p' /Users/sondre/dev/lyng/crates/objects/src/internal_methods/named_properties.rs
```

Confirm each line is a single `self.bump_invalidation(heap, id, InvalidationCause::*)` call that can be safely deleted (no surrounding control flow depends on its return value).

- [ ] **Step 2: Delete the calls**

Use `Edit` to remove each `self.bump_invalidation(...)` line and its trailing semicolon. If the call has a return-value check (`if !self.bump_invalidation(...) { ... }`), remove the whole if-block — note no Spec 1 caller treats bump failure as fatal except in test wrappers.

In `internal_methods.rs` line 425, this is inside `bump_prototype_mutation_epoch` — entire wrapper is deleted in Task A.3.3, so leave the body alone for now (Task A.3.3 deletes the whole method).

In `internal_methods.rs` line 460, this is inside the legacy `set_prototype` (kept for bootstrap paths per Spec 1). Delete the line. The line above it (epoch bump) is the only line being removed; the prototype write itself stays.

In `named_properties.rs`:
- Line 173: inside `ensure_named_property_dictionary` after the dictionary flag union — delete.
- Line 195: inside `redefine_named_property` — delete.
- Line 235: inside `delete_named_property` — delete.

Spec 1's watchpoint fires (`agent.fire_watchpoints_for_shape(old)` calls in `agent.rs` at lines 302, 329, 349, 438, 482) are the new and now sole invalidation signal.

- [ ] **Step 3: Run** `cargo build --workspace` **— may have warnings about unused imports of `InvalidationCause`. Fix imports as needed.**

- [ ] **Step 4: Run** `cargo test --workspace` **— full suite green.**

If any IC tests fail, the failure mode is likely a stale IC entry surviving an invalidation. Investigate at the actual test site — the fix is usually that the test was implicitly relying on the epoch bump where it should now rely on the watchpoint fire (Spec 1 already wired the fire; the test should still be valid).

- [ ] **Step 5: Commit**

```bash
git add crates/objects/src/internal_methods.rs crates/objects/src/internal_methods/named_properties.rs
git commit -m "objects/internal_methods: delete bump_invalidation calls at 5 sites"
```

---

### Task A.3.3: Delete `bump_prototype_mutation_epoch` + `ObjectRuntime::bump_invalidation`

**Files:**
- Modify: `crates/objects/src/internal_methods.rs:420-426` (delete `bump_prototype_mutation_epoch`)
- Modify: `crates/objects/src/runtime_storage.rs:231-250` (delete `ObjectRuntime::bump_invalidation`)
- Modify: callers of `bump_prototype_mutation_epoch` (grep)

- [ ] **Step 1: Find callers of `bump_prototype_mutation_epoch`**

```bash
grep -rn "bump_prototype_mutation_epoch" /Users/sondre/dev/lyng/crates/
```

Likely callers: `Agent::set_prototype_of` in `crates/env/src/agent.rs`. Remove each call.

- [ ] **Step 2: Delete `bump_prototype_mutation_epoch`**

In `crates/objects/src/internal_methods.rs:420-426`, delete the whole method.

- [ ] **Step 3: Delete `ObjectRuntime::bump_invalidation`**

In `crates/objects/src/runtime_storage.rs:231-250`, delete the method.

- [ ] **Step 4: Run** `cargo build --workspace` **— iterate on unused-import warnings.**

- [ ] **Step 5: Commit**

```bash
git add crates/objects/src/internal_methods.rs crates/objects/src/runtime_storage.rs crates/env/src/agent.rs
git commit -m "objects/runtime: delete bump_prototype_mutation_epoch + bump_invalidation"
```

---

### Task A.3.4: Delete `next_invalidation_epoch` field + `last_invalidation` metadata

**Files:**
- Modify: `crates/objects/src/runtime.rs` (delete `next_invalidation_epoch` field around line 60)
- Modify: `crates/objects/src/runtime.rs` (delete `InvalidationEvent` if unused; check imports)
- Modify: `crates/objects/src/object_metadata.rs` (delete `last_invalidation: Option<InvalidationEvent>` if present on `ObjectMetadata`)

- [ ] **Step 1: Delete the field**

In `crates/objects/src/runtime.rs`, find and delete:
```rust
pub(crate) next_invalidation_epoch: u64,
```

Update `Default` / `new` constructors that initialize it.

- [ ] **Step 2: Check `ObjectMetadata.last_invalidation` field**

The `bump_invalidation` method (now deleted) wrote to `metadata.last_invalidation = Some(InvalidationEvent::new(epoch, cause))`. Grep for the field:
```bash
grep -rn "last_invalidation" /Users/sondre/dev/lyng/crates/objects/
```

If `ObjectMetadata.last_invalidation` is unused (only written by the now-deleted bump), delete the field. If it's read by other code (e.g., debugger introspection), keep the field and the `InvalidationEvent` type but mark them dead-code-allowed or actually exercise them via a test-only path.

Likely outcome: delete the field and `InvalidationEvent` type entirely.

- [ ] **Step 3: Run** `cargo build --workspace` **— iterate.**

- [ ] **Step 4: Run** `cargo test --workspace` **— green.**

- [ ] **Step 5: Commit**

```bash
git add crates/objects/src/runtime.rs crates/objects/src/object_metadata.rs
git commit -m "objects/runtime: delete next_invalidation_epoch + last_invalidation metadata"
```

---

### Task A.3.5: Delete `RuntimeObjectRecord::last_invalidation_epoch` field + getter + mutator

**Files:**
- Modify: `crates/gc/src/arena/records.rs:350-366` (delete the field)
- Modify: `crates/gc/src/arena/records.rs:450-455` (delete the getter)
- Modify: storage-target enum (for `mut_store_object_invalidation_epoch`)

- [ ] **Step 1: Delete the field on `RuntimeObjectRecord`**

In `crates/gc/src/arena/records.rs:360`, delete:
```rust
pub(super) last_invalidation_epoch: u64,
```

Adjust struct constructors that initialized this field (likely with `last_invalidation_epoch: 0`).

- [ ] **Step 2: Delete the getter (lines 450-455)**

```rust
#[inline]
pub const fn last_invalidation_epoch(self) -> Option<u64> { ... }
```

- [ ] **Step 3: Delete the mutator**

Find `mut_store_object_invalidation_epoch`:
```bash
grep -rn "mut_store_object_invalidation_epoch\|ObjectInvalidationEpoch" /Users/sondre/dev/lyng/crates/
```

Delete the method and any `ObjectInvalidationEpoch` enum variant in the storage-target enum (e.g., `ObjectHandleStoreTarget` or similar).

- [ ] **Step 4: Run** `cargo build --workspace` **— iterate on remaining callers.**

The recon flagged `record.last_invalidation_epoch()` reads at:
- `property_cache.rs:589` (already deleted in Task A.2.3 Step 7).

Verify no remaining grep hits:
```bash
grep -rn "last_invalidation_epoch" /Users/sondre/dev/lyng/crates/
```

Expected: zero matches.

- [ ] **Step 5: Run** `cargo test --workspace` **— green.**

- [ ] **Step 6: Commit**

```bash
git add crates/gc/src/arena/records.rs
git commit -m "gc/arena: delete RuntimeObjectRecord::last_invalidation_epoch field + accessors"
```

---

### Task A.3.6: Delete `InvalidationCause` enum

**Files:**
- Modify: `crates/objects/src/shapes.rs:22-27`
- Modify: any callers that imported `InvalidationCause`

- [ ] **Step 1: Delete the enum**

In `crates/objects/src/shapes.rs`, delete lines 22-27:
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InvalidationCause {
    PrototypeMutation,
    PropertyRedefinition,
    PropertyDeletion,
    DictionaryTransition,
}
```

- [ ] **Step 2: Remove `pub use InvalidationCause` from `crates/objects/src/lib.rs`**

Grep:
```bash
grep -n "InvalidationCause" /Users/sondre/dev/lyng/crates/objects/src/lib.rs
```

Delete the re-export.

- [ ] **Step 3: Update any remaining import sites**

```bash
grep -rn "InvalidationCause" /Users/sondre/dev/lyng/crates/
```

Expected: zero matches (all callsites were removed in earlier tasks).

- [ ] **Step 4: Run** `cargo build --workspace` **— green.**

- [ ] **Step 5: Run** `cargo test --workspace` **— green.**

- [ ] **Step 6: Commit**

```bash
git add crates/objects/src/shapes.rs crates/objects/src/lib.rs
git commit -m "objects/shapes: delete InvalidationCause enum (epoch system fully retired)"
```

---

### Task A.3.7: Update `redefine_delete_and_prototype_mutation_bump_invalidation_epochs` test

**Files:**
- Modify: `crates/objects/src/tests.rs:2133-2204` (or wherever the test now lives after Spec 1's augmentation)

- [ ] **Step 1: Locate the test**

```bash
grep -n "redefine_delete_and_prototype_mutation_bump_invalidation_epochs" /Users/sondre/dev/lyng/crates/objects/src/tests.rs /Users/sondre/dev/lyng/crates/env/src/tests.rs
```

The test was originally at `crates/objects/src/tests.rs:2133-2204` (Spec 1 augmented it with shape-change assertions). It may have moved to `crates/env/src/tests.rs` during Spec 1's `Agent::set_prototype_of` routing.

- [ ] **Step 2: Strip epoch assertions; rename test**

Find every assertion involving `last_invalidation_epoch()` or `epoch == N`. Delete them. Retain only the shape-change assertions added in Spec 1.

Rename the test to reflect the new contract:
```rust
#[test]
fn redefine_delete_and_prototype_mutation_transition_shapes() {
    // ... existing body with epoch checks removed, shape-change checks retained ...
}
```

- [ ] **Step 3: Run** `cargo test --workspace -- redefine_delete_and_prototype_mutation_transition_shapes` **— green.**

- [ ] **Step 4: Run** `cargo test --workspace` **— green.**

- [ ] **Step 5: Commit**

```bash
git add crates/objects/src/tests.rs crates/env/src/tests.rs
git commit -m "objects/tests: replace epoch assertions with shape-change assertions in the bump test family"
```

---

### Task A.3.8: Final Phase A sweep — A.7 grep check + A.9 full IC suite

**Files:**
- Read-only checks + final commit if cleanup needed.

- [ ] **Step 1: Grep check (test A.7) — confirm all infrastructure is gone**

```bash
grep -rn "last_invalidation_epoch\|bump_invalidation\|next_invalidation_epoch\|InvalidationCause\|bump_prototype_mutation_epoch\|mut_store_object_invalidation_epoch\|named_epoch\|named_aux_epoch\|polymorphic_own_data_epochs\|invalidation_epoch.*PropertyCacheDependency" /Users/sondre/dev/lyng/crates/
```

Expected: zero matches. If any survive, audit them — they may be in test code that should also be updated, or in comments that should be cleaned.

(The `invalidation_epoch` on `TieringState` / `TieringSnapshot` is unrelated — it tracks tier-up invalidations, not IC invalidations. Leave it. Grep with care.)

- [ ] **Step 2: Run** `cargo test -p lyng-vm --test inline_caches` **— full IC suite green (~33 tests, test A.9).**

- [ ] **Step 3: Run** `cargo test --workspace` **— full workspace green.**

- [ ] **Step 4: Run** `cargo clippy --workspace --all-targets -- -D warnings` **— fix any.**

- [ ] **Step 5: Run** `cargo fmt --check` **— fix any.**

- [ ] **Step 6: Commit any cleanup (skip if clean).**

```bash
git status
# If anything outstanding:
git add -A
git commit -m "phase-a: final cleanup (fmt, clippy, unused imports)"
```

---

## Verification (end-to-end Phase A)

After PR A.3 lands:

1. **All tests:** `cargo test --workspace` green.
2. **IC regression:** `cargo test -p lyng-vm --test inline_caches` green (~33 tests, test A.9).
3. **Test A.1 (no epoch read):** `grep -n "last_invalidation_epoch" crates/vm/src/vm/dispatch/property.rs` returns 0 matches.
4. **Test A.2 + A.3 + A.4 + A.5:** new tests added across PRs A.1 and A.2 all pass.
5. **Test A.6 (tier-up):** `cargo test --workspace warmup_counter_after_lift_still_triggers_allocation_at_threshold` green; plus the existing tier-up tests still pass.
6. **Test A.7 (epoch grep absent):** Task A.3.8 step 1 returns 0 matches.
7. **Test A.8 (renamed bump test):** `cargo test --workspace redefine_delete_and_prototype_mutation_transition_shapes` green.
8. **No new clippy warnings; cargo fmt clean.**
9. **Phase B readiness:** the `Vm::polymorphic_chains` map (Phase B) consumes nothing new from Phase A; the boundary holds.

---

## Out of scope (Phase B onwards)

- Polymorphic chain out-of-line storage (Phase B).
- `MetadataTable` per code object (Phase C).
- Flipping the system of record (Phase D).
- `Status` API projections (Phase E).
- `recording_watchpoint_fires` ungating cleanup.
