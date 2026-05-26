# Spec 2 Phase B — Polymorphic Chain Out-of-Line Storage

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Lift `NamedPropertyFeedback`'s out-of-line polymorphic entries (indices `POLY_LIMIT..POLYMORPHIC_PROPERTY_CACHE_LIMIT`, i.e. entries 2-7) into a `Vm`-side map keyed by `(CodeRef, FeedbackSlotId)`. The inline `entries[0..POLY_LIMIT]` and the `polymorphic_own_data_handlers` sidecar stay where they are — the asm fast path is unchanged in Phase B.

**Architecture:** Additive at the slow-path layer. `Vm` gains a `polymorphic_chains: HashMap<(CodeRef, FeedbackSlotId), PolymorphicChain>` field. `PolymorphicChain` carries `Vec<NamedPropertyCacheEntry>` (up to `POLYMORPHIC_PROPERTY_CACHE_LIMIT - POLY_LIMIT = 6` entries). Lazy: monomorphic and ≤POLY_LIMIT polymorphic slots have no map entry. Slow-path operations (install/walk/clear) route through both the inline portion and the map. On code GC, the map's entries are pruned.

**Tech Stack:** Rust, `std::collections::HashMap`, the existing post-mark sweep hook in `crates/env/src/agent/weak_finalization.rs`.

**Spec:** `docs/superpowers/specs/2026-05-26-spec-2-ic-jsc-migration-design.md` (§4 — Phase B).

---

## Context

Phase A landed (commits `04b920b5` → `e0510725`). Post-Phase-A state:

- `NamedPropertyFeedback` ([crates/vm/src/vm/feedback.rs:649-682](crates/vm/src/vm/feedback.rs#L649-L682)) holds `entries: [Option<NamedPropertyCacheEntry>; 8]` as the system of record for all polymorphic entries. `POLY_LIMIT = 2` is the inline-sidecar capacity; `POLYMORPHIC_PROPERTY_CACHE_LIMIT = 8` is the total chain cap.
- `polymorphic_own_data_handlers: [NamedPropertyHandler; 2]` is the asm-visible sidecar (Phase 3f). Drives the inline polymorphic OwnData fast path. Phase B does NOT touch this.
- `entries[POLY_LIMIT..entry_count]` (indices 2..N where N≤8) are reachable only via the slow path's `find_entry_index` (binary search by receiver shape on the full `entries[0..entry_count]`).
- Phase A added `generation: u32` to `NamedPropertyFeedback`; the slow-path install bumps generation and registers `AdaptiveProtoLoad` watchpoints on proto-chain shapes (`register_proto_chain_watchpoints` at `feedback.rs:2966`).

Phase B's job:
1. Define a `PolymorphicChain` type that owns `Vec<NamedPropertyCacheEntry>`.
2. Add `Vm::polymorphic_chains: HashMap<(CodeRef, FeedbackSlotId), PolymorphicChain>`.
3. Migrate the slow-path install path: when `entry_count >= POLY_LIMIT`, push to the map's chain instead of `entries[POLY_LIMIT..]`. Shrink `entries` to `[Option<NamedPropertyCacheEntry>; POLY_LIMIT]`.
4. Migrate the slow-path walk: search inline, then map.
5. Migrate the slow-path clear: clear inline + remove map entry.
6. Add GC pruning for entries whose `CodeRef` is dead (mirrors Spec 1's `prune_dead_prototype_transitions`).
7. Tests B1-B8.

After Phase B:
- `NamedPropertyFeedback.entries` shrinks from 8 slots to 2 (POLY_LIMIT).
- Walks/installs cost one extra `HashMap` lookup when chain is in poly state but inline is full.
- `Vm::polymorphic_chains` is `None` for the >99% of code/slot pairs that stay monomorphic or fit in inline poly.

---

## File map

| File | New / existing | Responsibility |
|---|---|---|
| `crates/vm/src/vm/feedback/polymorphic.rs` | NEW | `PolymorphicChain` type + install / walk / clear API. |
| `crates/vm/src/vm/feedback.rs` | existing | Shrink `entries` from 8 to `POLY_LIMIT`. Update install/walk/clear logic to consult `Vm::polymorphic_chains` for entries beyond inline. Update `record_named_property_cache_entry` and `observe_slow_path`. |
| `crates/vm/src/vm.rs` | existing | Add `pub(crate) polymorphic_chains: HashMap<(CodeRef, FeedbackSlotId), PolymorphicChain>` field. Add `pub(crate) fn prune_dead_code_polymorphic_chains(is_live: impl Fn(CodeRef) -> bool)` method. Add `Vm` constructor initialization. |
| `crates/env/src/agent/weak_finalization.rs` | existing | Wire `prune_dead_code_polymorphic_chains` into the post-mark sweep at line 56-ish (alongside `prune_dead_prototype_transitions` and `sweep_invalidated_watchpoint_sets`). |
| `crates/vm/src/tests/inline_caches.rs` | existing | Tests B1-B5 (install/walk/clear scenarios) + B8 (existing poly cases stay passing). |
| `crates/vm/src/tests/feedback.rs` | existing | Tests B6, B7 (GC sweep for dead-code entries). |

---

# PR B.1 — `PolymorphicChain` + `Vm::polymorphic_chains` + slow-path migration

## Task B.1.1: Define `PolymorphicChain`

**Files:**
- Create: `crates/vm/src/vm/feedback/polymorphic.rs`
- Modify: `crates/vm/src/vm/feedback.rs` (add `mod polymorphic;` and a `pub(super) use` for the type)

- [ ] **Step 1: Create the new module**

Create `crates/vm/src/vm/feedback/polymorphic.rs`:

```rust
//! Out-of-line storage for polymorphic IC chain entries beyond `POLY_LIMIT`.
//! Spec 2 Phase B.
//!
//! Each `(CodeRef, FeedbackSlotId)` that grows into a 3+ entry polymorphic
//! state gets a `PolymorphicChain` entry in `Vm::polymorphic_chains`.
//! The chain holds entries [POLY_LIMIT..POLYMORPHIC_PROPERTY_CACHE_LIMIT].
//! Entries [0..POLY_LIMIT] stay inline in `NamedPropertyFeedback.entries`
//! to keep the asm fast path's sidecar (`polymorphic_own_data_handlers`)
//! addressable in the existing layout.
//!
//! On 9th distinct shape the IC transitions to Megamorphic and the chain
//! entry is dropped (caller's responsibility).

use lyng_objects::NamedPropertyCacheEntry;
use lyng_types::ShapeId;

use super::POLYMORPHIC_PROPERTY_CACHE_LIMIT;

/// Maximum number of out-of-line entries per chain.
/// `POLYMORPHIC_PROPERTY_CACHE_LIMIT - POLY_LIMIT` once flattened.
pub(crate) const POLYMORPHIC_CHAIN_CAP: usize = POLYMORPHIC_PROPERTY_CACHE_LIMIT - 2; // = 6

pub(crate) struct PolymorphicChain {
    entries: Vec<NamedPropertyCacheEntry>,
}

impl PolymorphicChain {
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::with_capacity(POLYMORPHIC_CHAIN_CAP),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn is_full(&self) -> bool {
        self.entries.len() >= POLYMORPHIC_CHAIN_CAP
    }

    /// Linear search by receiver shape. Chain is small (≤6) so linear beats
    /// binary search hash overhead. Returns `None` if no entry matches.
    pub(crate) fn find_by_shape(&self, receiver_shape: ShapeId) -> Option<&NamedPropertyCacheEntry> {
        self.entries
            .iter()
            .find(|entry| entry.receiver_shape() == receiver_shape)
    }

    /// Pushes a new entry. Caller must verify `!is_full()` before calling.
    pub(crate) fn push(&mut self, entry: NamedPropertyCacheEntry) {
        debug_assert!(self.entries.len() < POLYMORPHIC_CHAIN_CAP);
        self.entries.push(entry);
    }

    /// Iterator over entries — used by the slow path for fallback walks
    /// and by GC tracing to visit holder references.
    pub(crate) fn entries(&self) -> impl Iterator<Item = &NamedPropertyCacheEntry> {
        self.entries.iter()
    }
}
```

- [ ] **Step 2: Wire the module into `feedback.rs`**

In `crates/vm/src/vm/feedback.rs`, near the top of the file (alongside other `mod`/`use` declarations):

```rust
mod polymorphic;
pub(crate) use polymorphic::{PolymorphicChain, POLYMORPHIC_CHAIN_CAP};
```

- [ ] **Step 3: Build**

```bash
cargo check --workspace --all-targets
```

Expected: clean. No callers yet.

- [ ] **Step 4: Commit**

```bash
git add crates/vm/src/vm/feedback/polymorphic.rs crates/vm/src/vm/feedback.rs
git commit -m "$(cat <<'EOF'
vm/feedback: add PolymorphicChain type (no callers yet)

Spec 2 Phase B: out-of-line storage for IC polymorphic entries beyond
POLY_LIMIT. PolymorphicChain holds a Vec<NamedPropertyCacheEntry> with
capacity POLYMORPHIC_PROPERTY_CACHE_LIMIT - POLY_LIMIT (= 6). Linear
search by receiver shape (chain is small enough that linear beats hash
overhead). No production callers yet; B.1.2 onwards wires it in.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task B.1.2: Add `Vm::polymorphic_chains` field + accessors

**Files:**
- Modify: `crates/vm/src/vm.rs` (struct field + initializer + accessor methods)

- [ ] **Step 1: Add the field**

In `crates/vm/src/vm.rs`, in the `Vm` struct (around line 148-222 alongside `feedback_vectors`), add:

```rust
/// Spec 2 Phase B: out-of-line polymorphic IC entries (indices POLY_LIMIT..8).
/// Keyed by (CodeRef, FeedbackSlotId). Lazy: monomorphic and ≤POLY_LIMIT
/// polymorphic slots have no entry. Cleared on AdaptiveProtoLoad fire
/// (via clear path) and on code GC (via prune_dead_code_polymorphic_chains).
polymorphic_chains: HashMap<(CodeRef, FeedbackSlotId), PolymorphicChain>,
```

Add the import at the top of vm.rs:

```rust
use crate::vm::feedback::PolymorphicChain;
```

Adjust import grouping to match the surrounding style.

- [ ] **Step 2: Initialize the field**

In `Vm`'s `Default` derive (line 147), the new HashMap field will pick up `HashMap::default()` automatically — verify by reading the derive. If `Vm` has an explicit `Default` impl or `new()` constructor, add `polymorphic_chains: HashMap::new(),` to the initializer.

- [ ] **Step 3: Add accessor methods**

Below the existing `impl Vm` block (or in a new impl section), add:

```rust
impl Vm {
    /// Returns the polymorphic chain for `(code, slot)` if any.
    pub(crate) fn polymorphic_chain(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<&PolymorphicChain> {
        self.polymorphic_chains.get(&(code, slot))
    }

    /// Returns a mutable reference to the polymorphic chain for `(code, slot)`,
    /// lazily creating an empty chain on first access.
    pub(crate) fn polymorphic_chain_mut(
        &mut self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> &mut PolymorphicChain {
        self.polymorphic_chains
            .entry((code, slot))
            .or_insert_with(PolymorphicChain::new)
    }

    /// Removes the polymorphic chain for `(code, slot)`. Called when the IC
    /// transitions to Megamorphic or is cleared by an AdaptiveProtoLoad fire.
    pub(crate) fn drop_polymorphic_chain(
        &mut self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) {
        self.polymorphic_chains.remove(&(code, slot));
    }
}
```

- [ ] **Step 4: Build + test**

```bash
cargo check --workspace --all-targets
cargo test --workspace --all-targets
```

Expected: clean, 2648 tests still pass (no new callers yet).

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm.rs
git commit -m "vm: add polymorphic_chains map + accessors (Spec 2 Phase B)"
```

---

## Task B.1.3: Migrate slow-path install (push to map after POLY_LIMIT)

**Files:**
- Modify: `crates/vm/src/vm/feedback.rs` — the IC slow-path install at `record_named_property_cache_entry` (~line 2950) and the lower-level `insert_entry_at` / `promote_to_megamorphic` methods.

This is the hottest part of Phase B. Read the existing install logic carefully first:

```bash
grep -n "insert_entry_at\|promote_to_megamorphic\|install_first_entry\|find_entry_index\|search_entry_index" /Users/sondre/dev/lyng/crates/vm/src/vm/feedback.rs | head -20
```

The current state machine (per recon):
- `install_first_entry` → places at `entries[0]`, sets state Monomorphic.
- `insert_entry_at(index, entry)` shifts existing entries from `index` onwards, places new entry at `entries[index]`, increments `entry_count`.
- `promote_to_megamorphic` zeros `entries`, sets state Mega.
- Transition Mono→Poly: when `entry_count == 1` and new shape inserts at position 1.
- Transition Poly→Mega: when `entry_count >= POLYMORPHIC_PROPERTY_CACHE_LIMIT` at `insert_entry_at`.

After Phase B, the inline `entries` array shrinks to `[Option<NamedPropertyCacheEntry>; POLY_LIMIT]`. When `entry_count` reaches POLY_LIMIT, additional installs route to `Vm::polymorphic_chains[(code, slot)]`.

- [ ] **Step 1: Shrink `NamedPropertyFeedback.entries` to POLY_LIMIT**

In `crates/vm/src/vm/feedback.rs`, find the struct definition (around line 649). Change:

```rust
entries: [Option<NamedPropertyCacheEntry>; POLYMORPHIC_PROPERTY_CACHE_LIMIT],
```

to:

```rust
/// Inline polymorphic entries (mirrored into polymorphic_own_data_handlers).
/// Out-of-line entries [POLY_LIMIT..POLYMORPHIC_PROPERTY_CACHE_LIMIT] live in
/// Vm::polymorphic_chains keyed by (code, slot). Spec 2 Phase B §4.
entries: [Option<NamedPropertyCacheEntry>; POLY_LIMIT],
```

Update `NamedPropertyFeedback::new()` if it initialized `entries: [None; POLYMORPHIC_PROPERTY_CACHE_LIMIT]`:

```rust
entries: [None; POLY_LIMIT],
```

- [ ] **Step 2: Update `find_entry_index` / `search_entry_index` to only search inline**

The binary search on `entries[0..entry_count]` now searches at most `POLY_LIMIT` (=2) entries. Linear search is fine. Change `search_entry_index` to a linear walk that returns `Result<usize, usize>`:

```rust
fn search_entry_index(&self, receiver_shape: ShapeId) -> Result<usize, usize> {
    let inline_count = usize::from(self.entry_count).min(POLY_LIMIT);
    for (i, slot) in self.entries[..inline_count].iter().enumerate() {
        let Some(entry) = slot else { continue; };
        match entry.receiver_shape().cmp(&receiver_shape) {
            Ordering::Equal => return Ok(i),
            Ordering::Greater => return Err(i),
            Ordering::Less => {}
        }
    }
    Err(inline_count)
}
```

(The `Less/Greater/Equal` ordering keeps the "insertion point" semantics of the previous `binary_search_by`.)

- [ ] **Step 3: Update `find_entry_index` to also consult the map**

Add a new helper `find_entry_full(vm, code, slot, receiver_shape) -> Option<NamedPropertyCacheEntry>` that searches inline first then falls through to `Vm::polymorphic_chain(code, slot).and_then(|c| c.find_by_shape(receiver_shape)).copied()`.

Update callers of `find_entry_index` in the slow-path read path to use the new helper.

The existing helper that walks inline-only stays for hot paths that don't need the full search.

- [ ] **Step 4: Update install (insert_entry_at) to push to map when inline is full**

Current `insert_entry_at` shifts entries within the 8-slot array. New behavior:

```rust
fn insert_entry_at(
    &mut self,
    vm_chains: &mut HashMap<(CodeRef, FeedbackSlotId), PolymorphicChain>,
    code: CodeRef,
    slot: FeedbackSlotId,
    index: usize,
    entry: NamedPropertyCacheEntry,
) {
    let count = usize::from(self.entry_count);

    // Mega transition: total (inline + chain) at cap.
    let chain_len = vm_chains
        .get(&(code, slot))
        .map_or(0, |c| c.len());
    if count + chain_len >= POLYMORPHIC_PROPERTY_CACHE_LIMIT {
        self.promote_to_megamorphic();
        vm_chains.remove(&(code, slot));
        return;
    }

    if index < POLY_LIMIT {
        // Insert into inline array; if it's full, the displaced entry spills to the chain.
        if count >= POLY_LIMIT {
            // Displaced entry: take the current entries[POLY_LIMIT - 1] and push it to the chain.
            let displaced = self.entries[POLY_LIMIT - 1]
                .take()
                .expect("inline slot must be present when count >= POLY_LIMIT");
            let chain = vm_chains
                .entry((code, slot))
                .or_insert_with(PolymorphicChain::new);
            chain.push(displaced);
        }
        // Shift inline entries [index..POLY_LIMIT-1] right by one to make room.
        self.entries.copy_within(index..POLY_LIMIT - 1, index + 1);
        self.entries[index] = Some(entry);
        self.entry_count = self.entry_count.saturating_add(1);
    } else {
        // index >= POLY_LIMIT means the new entry goes directly to the chain.
        let chain = vm_chains
            .entry((code, slot))
            .or_insert_with(PolymorphicChain::new);
        chain.push(entry);
        self.entry_count = self.entry_count.saturating_add(1);
    }

    self.cache_state = if self.entry_count == 1 {
        InlineCacheState::Monomorphic
    } else {
        InlineCacheState::Polymorphic
    };
}
```

This signature now takes `&mut HashMap<...>` and the `(code, slot)` key — meaning every caller of `insert_entry_at` must thread these through. The callers are in `record_named_property_cache_entry` and similar. Update them by accessing `vm.polymorphic_chains` directly (since `vm` is `&mut self` in that context).

Note: there's a borrow-checker subtlety. `record_named_property_cache_entry` likely calls `self.feedback_vectors[index].site_mut(slot)` to get the `&mut NamedPropertyFeedback`, then needs to also access `self.polymorphic_chains`. Use split-borrow:

```rust
let Self {
    feedback_vectors,
    polymorphic_chains,
    ..
} = self;
let Some(vector) = feedback_vectors.get_mut(code_index(code)) else { return };
let Some(site) = vector.site_mut(slot) else { return };
let FeedbackSiteState::NamedProperty(named) = site else { return };
named.insert_entry_at(polymorphic_chains, code, slot, ...);
```

Or refactor `record_named_property_cache_entry` to be a method on `Vm` that handles the split internally.

- [ ] **Step 5: Update `promote_to_megamorphic` to drop the map entry**

Add a parameter or have the caller drop the map entry:

```rust
fn promote_to_megamorphic(&mut self) {
    // Inline only - caller drops Vm::polymorphic_chains entry.
    self.cache_state = InlineCacheState::Megamorphic;
    self.entry_count = 0;
    for slot in &mut self.entries {
        *slot = None;
    }
    // ...zero sidecars as before...
}
```

The caller in `insert_entry_at` (after the Mega check) already does `vm_chains.remove(&(code, slot))` per the new code.

- [ ] **Step 6: Build + iterate**

```bash
cargo check --workspace --all-targets
```

Iterate on compile errors. The borrow-checker will force you to thread `(code, slot)` and `&mut polymorphic_chains` through several functions. This is expected.

- [ ] **Step 7: Run existing IC tests**

```bash
cargo test -p lyng-vm --test inline_caches 2>&1 | tail -10
cargo test --workspace --all-targets 2>&1 | tail -10
```

All 2648 tests should still pass — Phase B is behavior-preserving for the polymorphic semantics, only the storage location changes.

If `named_property_load_ic_keeps_six_shape_polymorphic_cache` (line 648) or `named_property_load_ic_promotes_to_megamorphic_beyond_polymorphic_capacity` (line 827) fails, the transition logic above has a bug. Investigate.

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "$(cat <<'EOF'
vm/feedback: route polymorphic entries beyond POLY_LIMIT into Vm::polymorphic_chains

NamedPropertyFeedback.entries shrinks from [Option<...>; 8] to
[Option<...>; POLY_LIMIT]. Out-of-line entries (indices 2..8) now live in
Vm::polymorphic_chains keyed by (CodeRef, FeedbackSlotId), populated by
the slow-path insert when inline is full and dropped on Mega transition.

The asm fast path is unchanged (still walks polymorphic_own_data_handlers).
Walk semantics preserved: inline first, then map fallback.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task B.1.4: Migrate slow-path clear (AdaptiveProtoLoad fire)

**Files:**
- Modify: `crates/vm/src/vm.rs` — `clear_ic_slot_if_generation_matches` (around line 1490)

The current implementation (Phase A) calls `vector.clear_site(slot)` which sets the `Option<FeedbackSiteState>` to `None`. The map entry for `(code, slot)` survives. Phase B must also drop the map entry.

- [ ] **Step 1: Update `clear_ic_slot_if_generation_matches`**

```rust
pub(crate) fn clear_ic_slot_if_generation_matches(
    &mut self,
    code: CodeRef,
    slot: FeedbackSlotId,
    expected_generation: u32,
) {
    let Some(vector) = self.feedback_vectors.get_mut(code_index(code)) else {
        return;
    };
    if vector.generation(slot) != expected_generation {
        return;
    }
    vector.clear_site(slot);
    self.polymorphic_chains.remove(&(code, slot));  // NEW
    self.mirror_flat_slot(code, slot);
}
```

(Verify the actual current method body and patch the new line in the right place.)

- [ ] **Step 2: Update `reinit_named_property_site_if_cleared` similarly if it exists**

If Task 5+6's `reinit_named_property_site_if_cleared` (Phase A) interacts with the chain, ensure it doesn't leak stale entries. Likely safe: re-init creates a fresh `NamedPropertyFeedback` with `entry_count: 0`, so an existing chain entry would be unreachable. The next install starts from scratch. But to be tidy, the chain entry should be dropped on reinit too.

If `reinit_named_property_site_if_cleared` is the entry point for re-installs, ensure it doesn't observe a stale chain. Either:
- Drop the chain entry inside `clear_ic_slot_if_generation_matches` (done in Step 1), OR
- Drop it inside `reinit_named_property_site_if_cleared` as a defensive step.

If both paths are reachable, do both.

- [ ] **Step 3: Test**

```bash
cargo test --workspace --all-targets
```

The Phase A `adaptive_proto_load_*` tests in inline_caches.rs should still pass. They cover monomorphic clearing; polymorphic clearing is covered by Test B5 in Task B.1.6.

- [ ] **Step 4: Commit**

```bash
git add crates/vm/src/vm.rs
git commit -m "vm: drop polymorphic_chains entry on AdaptiveProtoLoad fire (Spec 2 Phase B)"
```

---

## Task B.1.5: GC tracing for `polymorphic_chains`

**Files:**
- Modify: wherever `Vm` is traced (look for `trace_heap_edges` calls that visit `feedback_vectors`).

`NamedPropertyCacheEntry` holds `holder: ObjectRef` and `dependencies: [Option<PropertyCacheDependency>; PROPERTY_CACHE_MAX_DEPENDENCIES]` (each `PropertyCacheDependency` holds an `ObjectRef`). These must be traced as GC roots — otherwise the GC could collect a holder that the IC still depends on, leading to a dangling cache entry.

The existing `entries` array on `NamedPropertyFeedback` already has its `holder` traced (since `feedback_vectors` is traced). Find the existing trace machinery and add `polymorphic_chains` to it.

- [ ] **Step 1: Locate the existing tracing**

```bash
grep -rn "trace_heap_edges\|TraceHeapEdges" /Users/sondre/dev/lyng/crates/vm/src/vm.rs /Users/sondre/dev/lyng/crates/env/src/agent.rs 2>&1 | head -20
```

Find where `feedback_vectors` is traced (or, if `FeedbackVector` doesn't impl `TraceHeapEdges`, trace whatever wraps it). The pattern Spec 1 used for `watchpoint_sets` is the closest reference.

If no explicit tracing exists for `feedback_vectors` (because `NamedPropertyCacheEntry.holder` is "weak" or the IC tolerates a dangling holder), then `polymorphic_chains` also doesn't need tracing — the prune-on-code-death is enough.

**Verification:** check whether `NamedPropertyCacheEntry.holder` survives GC pressure. Spec 2 Phase B should match whatever guarantee exists today.

- [ ] **Step 2: Add tracing for the map (if needed)**

If the existing `FeedbackVector` tracing visits `holder` and each `dependencies[i].object()`, add a `polymorphic_chains.values().flat_map(|c| c.entries()).for_each(|e| tracer.visit(e.holder()))` pattern in the same `trace_heap_edges` impl.

If `PolymorphicChain` needs its own `TraceHeapEdges` impl, add one to `crates/vm/src/vm/feedback/polymorphic.rs`.

If existing tracing is implicit (e.g., the heap walker discovers reachable objects without an explicit visit-from-Vm step), Phase B doesn't need new tracing — just pruning on code GC (Task B.2.1).

- [ ] **Step 3: Build + test**

```bash
cargo check --workspace --all-targets
cargo test --workspace --all-targets
```

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "vm/feedback: trace PolymorphicChain holders + dependencies (Spec 2 Phase B)"
```

Skip if no tracing changes were needed.

---

## Task B.1.6: Tests B1–B5

**Files:**
- Modify: `crates/vm/src/tests/inline_caches.rs`

- [ ] **Step 1: Test B1 — Mono → 2-entry Poly stays inline**

```rust
#[test]
fn b1_polymorphic_two_entries_stay_inline_no_map() {
    // Two distinct receiver shapes → state Polymorphic, entry_count=2, no map entry.
    let mut agent = /* setup */;
    let code = /* compile a function that loads obj.x */;
    let slot = FeedbackSlotId::from_raw(1).unwrap();

    // Trigger two distinct shapes.
    run_with_shape_a(&mut agent, code);
    run_with_shape_b(&mut agent, code);

    let snapshot = agent.vm().named_property_cache_snapshot(code, slot).unwrap();
    assert_eq!(snapshot.state(), FeedbackInlineCacheState::Polymorphic);
    assert_eq!(snapshot.entries().len(), 2);
    assert!(agent.vm().polymorphic_chain(code, slot).is_none());
}
```

(Use existing test helpers in `inline_caches.rs` for setup/run. Replace `/* setup */` and helper function names with actual ones from the file.)

- [ ] **Step 2: Test B2 — 3rd entry creates out-of-line chain**

```rust
#[test]
fn b2_polymorphic_third_entry_creates_chain_entry() {
    let mut agent = /* setup */;
    let code = /* compile */;
    let slot = /* slot ID */;

    run_with_shape_a(&mut agent, code);
    run_with_shape_b(&mut agent, code);
    run_with_shape_c(&mut agent, code);

    let chain = agent.vm().polymorphic_chain(code, slot).expect("chain must exist");
    assert_eq!(chain.len(), 1);

    let snapshot = agent.vm().named_property_cache_snapshot(code, slot).unwrap();
    assert_eq!(snapshot.entries().len(), 3); // Total = 2 inline + 1 chain
}
```

- [ ] **Step 3: Test B3 — 9th entry transitions to Mega + drops chain**

```rust
#[test]
fn b3_polymorphic_ninth_entry_transitions_to_mega_and_drops_chain() {
    // Drive 9 distinct receiver shapes through the same IC slot.
    let mut agent = /* setup */;
    let code = /* compile */;
    let slot = /* slot ID */;

    for shape_seed in 0..9 {
        run_with_distinct_shape(&mut agent, code, shape_seed);
    }

    let snapshot = agent.vm().named_property_cache_snapshot(code, slot).unwrap();
    assert_eq!(snapshot.state(), FeedbackInlineCacheState::Megamorphic);
    assert!(agent.vm().polymorphic_chain(code, slot).is_none());
}
```

- [ ] **Step 4: Test B4 — Walk order matches legacy unified walk**

Pick a known polymorphic configuration (e.g., 4 entries with specific shapes returning known values) and verify each shape's lookup returns the correct value. This is a regression check that the inline + map walk preserves the lookup contract.

- [ ] **Step 5: Test B5 — AdaptiveProtoLoad fire clears both inline + map**

```rust
#[test]
fn b5_adaptive_proto_load_fire_clears_inline_and_chain() {
    let mut agent = /* setup */;
    let code = /* compile */;
    let slot = /* slot ID */;

    // Build a polymorphic IC with 3+ entries (at least one in the chain).
    run_with_shape_a(&mut agent, code);  // proto-cache entry
    run_with_shape_b(&mut agent, code);
    run_with_shape_c(&mut agent, code);
    assert!(agent.vm().polymorphic_chain(code, slot).is_some());

    // Mutate the shared prototype (fires AdaptiveProtoLoad on holder shape).
    mutate_holder_proto(&mut agent);

    // IC slot cleared: no chain entry, no inline entries.
    assert!(agent.vm().polymorphic_chain(code, slot).is_none());
    assert!(agent.vm().named_property_cache_snapshot(code, slot).is_none()
            || agent.vm().named_property_cache_snapshot(code, slot).unwrap().entries().is_empty());
}
```

- [ ] **Step 6: Build + run new tests**

```bash
cargo test -p lyng-vm --test inline_caches b1_polymorphic b2_polymorphic b3_polymorphic b4_polymorphic b5_polymorphic 2>&1 | tail -20
```

All 5 should pass.

- [ ] **Step 7: Run full IC suite**

```bash
cargo test -p lyng-vm --test inline_caches 2>&1 | tail -10
```

All ~35 tests still green.

- [ ] **Step 8: Commit**

```bash
git add crates/vm/src/tests/inline_caches.rs
git commit -m "vm/tests/inline_caches: B1-B5 polymorphic out-of-line chain tests"
```

---

# PR B.2 — GC sweep for dead-code polymorphic entries + Tests B6–B8

## Task B.2.1: `prune_dead_code_polymorphic_chains`

**Files:**
- Modify: `crates/vm/src/vm.rs`

- [ ] **Step 1: Add the method**

```rust
impl Vm {
    /// Spec 2 Phase B: post-mark GC sweep. Drops polymorphic chain entries
    /// for code that is no longer live. Mirrors
    /// `ObjectRuntime::prune_dead_prototype_transitions` from Spec 1.
    pub(crate) fn prune_dead_code_polymorphic_chains(
        &mut self,
        is_live: impl Fn(CodeRef) -> bool,
    ) {
        self.polymorphic_chains.retain(|(code, _slot), _chain| is_live(*code));
    }
}
```

The closure-based liveness check matches Spec 1's pattern (see `objects::runtime::prune_dead_prototype_transitions`).

- [ ] **Step 2: Build**

```bash
cargo check --workspace --all-targets
```

No callers yet — Task B.2.2 wires it in.

- [ ] **Step 3: Commit**

```bash
git add crates/vm/src/vm.rs
git commit -m "vm: add prune_dead_code_polymorphic_chains (Spec 2 Phase B)"
```

---

## Task B.2.2: Wire into post-mark sweep

**Files:**
- Modify: `crates/env/src/agent/weak_finalization.rs` (around line 43-61)

- [ ] **Step 1: Locate the sweep hook**

Read the existing `force_collect_with_additional_roots`:

```bash
sed -n '40,70p' /Users/sondre/dev/lyng/crates/env/src/agent/weak_finalization.rs
```

Find the lines where `prune_dead_prototype_transitions` and `sweep_invalidated_watchpoint_sets` run (Spec 1 wired these). The new call goes alongside.

- [ ] **Step 2: Determine the liveness predicate**

Look at how Spec 1's `prune_dead_prototype_transitions` is called: likely `|obj| heap.view().object(obj).is_some()` for `ObjectRef`. For `CodeRef`, the equivalent is "is the code installed?" — i.e., `vm.installed.get(code_index(code)).is_some_and(|s| s.is_some())`, OR if codes are heap-allocated, `|code| heap.view().code(code).is_some()` (look for whatever the GC-aware code liveness check is).

If unsure, search how Spec 1 checks code-liveness elsewhere or how `Vm::installed` is mutated on GC.

If `installed` entries are kept alive forever (not GC'd individually), then the prune is essentially a no-op for code that's still installed. But the map should still be cleaned when code is uninstalled (e.g., dynamic_function_cache evicts). The exact policy depends on Vm's code lifecycle — read it and match.

- [ ] **Step 3: Wire the call**

Add (after `sweep_invalidated_watchpoint_sets`):

```rust
self.vm.prune_dead_code_polymorphic_chains(|code| {
    // Liveness predicate — mirror whatever pattern Spec 1's
    // prune_dead_prototype_transitions uses for code/object liveness.
    self.vm.installed.get(code_index(code)).is_some_and(|slot| slot.is_some())
});
```

The borrow may be tricky — `prune_dead_code_polymorphic_chains` takes `&mut self.vm` and the closure captures `&self.vm`. Use split-borrow if needed:

```rust
let installed = &self.vm.installed;
self.vm.polymorphic_chains.retain(|(code, _), _| {
    installed.get(code_index(*code)).is_some_and(|s| s.is_some())
});
```

(Or inline the retain logic directly here.)

- [ ] **Step 4: Build + test**

```bash
cargo check --workspace --all-targets
cargo test --workspace --all-targets
```

- [ ] **Step 5: Commit**

```bash
git add crates/env/src/agent/weak_finalization.rs
git commit -m "env/agent: wire prune_dead_code_polymorphic_chains into post-mark sweep"
```

---

## Task B.2.3: Tests B6–B8

**Files:**
- Modify: `crates/vm/src/tests/feedback.rs` (or `crates/env/src/tests.rs` if GC sweep tests live there)

- [ ] **Step 1: Test B6 — Dead code → entry pruned**

```rust
#[test]
fn b6_polymorphic_chain_pruned_when_code_dies() {
    let mut agent = /* setup */;
    let code = /* compile a dynamic function that goes through the dynamic_function_cache */;
    let slot = /* slot ID */;

    // Drive 3+ shapes to populate a chain entry.
    populate_polymorphic_chain(&mut agent, code, 3);
    assert!(agent.vm().polymorphic_chain(code, slot).is_some());

    // Force code uninstallation (drop the dynamic_function_cache entry).
    evict_dynamic_function(&mut agent, code);

    // Run GC.
    agent.force_collect();

    // Chain entry pruned.
    assert!(agent.vm().polymorphic_chain(code, slot).is_none());
}
```

The mechanism to "evict" code depends on Vm internals. If there's no straightforward way, the test may need to be a unit test on `Vm::prune_dead_code_polymorphic_chains` directly (passing a closure that returns false for the test code).

- [ ] **Step 2: Test B7 — Live code → entry retained**

```rust
#[test]
fn b7_polymorphic_chain_retained_when_code_lives() {
    let mut agent = /* setup */;
    let code = /* compile, KEEP code rooted */;
    let slot = /* slot ID */;

    populate_polymorphic_chain(&mut agent, code, 3);
    assert!(agent.vm().polymorphic_chain(code, slot).is_some());

    agent.force_collect();

    assert!(agent.vm().polymorphic_chain(code, slot).is_some());
}
```

- [ ] **Step 3: Test B8 — Existing IC poly cases still pass**

This is the existing test suite at `crates/vm/src/tests/inline_caches.rs` Poly-related cases:

```bash
cargo test -p lyng-vm --test inline_caches polymorphic 2>&1 | tail -10
```

Expected: all green. Mention this verification in the PR description; no new test needed.

- [ ] **Step 4: Run + commit**

```bash
cargo test --workspace --all-targets
git add crates/vm/src/tests/feedback.rs crates/env/src/tests.rs
git commit -m "vm/tests: B6 + B7 GC sweep tests for polymorphic_chains"
```

---

## Task B.2.4: Final Phase B sweep

- [ ] **Step 1: Run all gates**

```bash
echo "=== Build ===" && cargo check --workspace --all-targets 2>&1 | tail -3
echo "=== Tests ===" && cargo test --workspace --all-targets 2>&1 | tail -3
echo "=== Clippy ===" && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -5
echo "=== Fmt ===" && cargo fmt --check 2>&1 | tail -5
```

Expected: all clean.

- [ ] **Step 2: Verify the architectural invariants**

```bash
# 1. NamedPropertyFeedback.entries is now [_; POLY_LIMIT] not [_; 8].
grep -n "entries: \[Option<NamedPropertyCacheEntry>" /Users/sondre/dev/lyng/crates/vm/src/vm/feedback.rs

# 2. Vm::polymorphic_chains exists.
grep -n "polymorphic_chains:" /Users/sondre/dev/lyng/crates/vm/src/vm.rs

# 3. polymorphic.rs module exists.
ls /Users/sondre/dev/lyng/crates/vm/src/vm/feedback/polymorphic.rs
```

- [ ] **Step 3: Commit any cleanup**

```bash
git status
# If anything outstanding:
git add -A
git commit -m "phase-b: final cleanup (fmt, clippy, unused imports)"
```

Skip if clean.

---

## Verification (end-to-end Phase B)

After PR B.2 lands:

1. `cargo test --workspace --all-targets` green (≥ 2648 + 7 new tests = 2655).
2. `cargo test -p lyng-vm --test inline_caches` green (~37 tests).
3. Clippy clean, fmt clean.
4. `NamedPropertyFeedback.entries` has length POLY_LIMIT (2), not 8.
5. `Vm::polymorphic_chains` exists and is exercised by tests B1-B5.
6. GC sweep prunes dead-code entries (B6) and retains live-code entries (B7).
7. Microbench (`crates/vm/benches/property_addition.rs`): ≤3% wall-clock regression vs. pre-Phase-B baseline. Run before and after if perf is in question.

---

## Out of scope (Phase C onwards)

- MetadataTable layout (Phase C).
- Flipping the system of record so `entries[0..POLY_LIMIT]` also moves to MetadataTable (Phase D).
- Per-kind Status projections (Phase E).
