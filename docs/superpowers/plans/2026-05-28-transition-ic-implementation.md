# Transition-Aware Write IC Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an asm-recognized monomorphic write IC kind (`OwnDataInlineWrite`, mode = 5) that caches the transition arrow `(source_shape, target_shape, slot_location)` per `(code, slot)`, recovering RayTrace's V8 score from 232 → 290+ without regressing other benchmarks.

**Architecture:** New cache entry kind plugs into the existing IC state machine (Uninit → Mono → Mega — polymorphic-write asm deferred), the existing watchpoint subsystem (new `AdaptiveOwnWrite` observer variant), and the existing slow-path observation pipeline (consumes existing `OwnDataTransition` plan path). No `PropertyMetadata` layout changes — `handler_bits` + `aux_bits` hold one entry's data. Polymorphic-write asm fast path is a clean follow-up if data shows it matters.

**Tech Stack:** Rust 2021, aarch64 inline asm via the Lyng VM DSL macros, criterion-style V8 benchmarks via `lyng-bench`.

**Spec:** `docs/superpowers/specs/2026-05-28-transition-ic-design.md`

---

## Context

The Spec 2 IC→JSC migration moved the engine from epoch-based to watchpoint-based invalidation but didn't extend the asm dispatch layer to recognize transitioning writes. Diagnostic counters (committed in `b8b3c83f`) show RayTrace pays 7.4M `AssignNamedProperty` slow-path entries per run, 98.3% classified `shape_mismatch` — the IC is in Monomorphic state but the receiver's pre-write shape never matches the cached post-write shape because every iteration's `Object.extend` allocates a fresh object whose shape transitions on each property assignment.

This plan adds a new IC kind that caches the transition arrow itself: `(source_shape → target_shape, slot_location)`. The asm fast path matches against `source_shape` (the pre-write shape), writes the value, then atomically updates the object's shape pointer to `target_shape`. For non-transitioning writes (`o.x = v` where `o` already has `x`), `source_shape == target_shape` and the shape-pointer write is a no-op.

**MVP scope (this plan):** Monomorphic-only. One source_shape per `(code, slot)`. Mode = 5 = `OwnDataInlineWrite`. Polymorphic-write asm path (mode = 6) is deferred — polymorphic IC sites stay on the existing Rust probe walk.

## File map

| File | Role |
|---|---|
| `crates/objects/src/shapes.rs` | Add new `NamedPropertyInlineWriteHandler` packed-handler struct (mirrors `NamedPropertyHandler` layout). |
| `crates/objects/src/watchpoint.rs` | Add `AdaptiveOwnWrite` variant to `ShapeInvalidationObserver` enum. |
| `crates/env/src/agent.rs` | Dispatch `AdaptiveOwnWrite` fire callback (mirrors `AdaptiveProtoLoad` handling). |
| `crates/vm/src/vm/ic_state/property.rs` | Add `monomorphic_own_inline_write_handler` sidecar field to `PropertyIcState`. |
| `crates/vm/src/vm/feedback.rs` | Slow-path routing for write opcodes; projection of write entries into PropertyMetadata mode 5; watchpoint registration. |
| `crates/vm/src/dsl/backend/aarch64/feedback.rs` | New asm macros: `branch_named_own_inline_write_mode!`, `load_named_target_shape!`. |
| `crates/vm/src/dsl/backend/aarch64/operands.rs` | New asm macro: `store_record_shape!`. |
| `crates/vm/src/dsl/handlers/cold.rs` | Add monomorphic-write hit path to `op_assign_named_property_dsl` body. |
| `crates/vm/src/tests/inline_caches.rs` | New asm correctness tests. |
| `reports/lyng/bench-v8.md` | Refresh post-implementation. |

## Bench baseline (committed, thermally stable, 3-run median)

| Benchmark | Score |
|---|---:|
| Richards | 470 |
| DeltaBlue | 389 |
| Crypto | 441 |
| RayTrace | 232 |
| NavierStokes | 602 |
| Splay | 1465 |

## Acceptance bar (after Task 8)

| Benchmark | Target | Acceptance bar |
|---|---:|---|
| RayTrace | 290+ | Must recover pre-Spec-2 (291) ±2% |
| DeltaBlue | 420+ | Material gain (≥8%) |
| Richards | 480+ | Modest gain (≥2%) |
| Crypto | 441 | No regression (within ±1.5% noise) |
| NavierStokes | 602 | No regression (within ±1.5% noise) |
| Splay | 1465 | No regression (within ±1.5% noise) |

---

## Task 1: New types — packed handler + watchpoint variant

**Goal:** Add the foundation types so subsequent tasks have something to import. Pure additions; no existing behavior changes.

**Files:**
- Modify: `crates/objects/src/shapes.rs:165-266` (add `NamedPropertyInlineWriteHandler` after `NamedPropertyHandler`)
- Modify: `crates/objects/src/watchpoint.rs` (add `AdaptiveOwnWrite` variant)

### Step 1.1: Write failing tests for `NamedPropertyInlineWriteHandler`

Add to the bottom of `crates/objects/src/shapes.rs` (or to the existing tests module if one exists):

```rust
#[cfg(test)]
mod inline_write_handler_tests {
    use super::*;

    #[test]
    fn from_transition_entry_packs_source_shape_target_shape_and_inline_slot() {
        let source = ShapeId::from_raw(7).expect("non-zero");
        let target = ShapeId::from_raw(11).expect("non-zero");
        let entry = NamedPropertyCacheEntry::new(
            /* receiver_shape */ source,
            /* holder */ ObjectRef::from_raw(1).expect("non-zero"),
            /* holder_shape */ target,
            /* slot_offset */ INLINE_SLOT_OFFSET_FLAG | 3, // inline slot 3
            /* attrs */ DescriptorAttributes::writable_data(),
            NamedPropertyCachePath::OwnDataTransition,
            /* dependency_count */ 1,
            /* dependencies */ [None; PROPERTY_CACHE_MAX_DEPENDENCIES],
        );
        let handler = NamedPropertyInlineWriteHandler::from_entry(entry);
        assert!(handler.is_valid());
        assert_eq!(handler.source_shape(), Some(source));
        assert_eq!(handler.target_shape(), Some(target));
        assert_eq!(handler.slot_location(), SlotLocation::Inline(3));
        assert!(handler.writable());
    }

    #[test]
    fn from_own_data_entry_uses_same_source_and_target_shape() {
        // Non-transition write: receiver_shape == holder_shape → target == source.
        let shape = ShapeId::from_raw(42).expect("non-zero");
        let entry = NamedPropertyCacheEntry::new(
            shape,
            ObjectRef::from_raw(1).expect("non-zero"),
            shape, // holder == receiver — no transition
            INLINE_SLOT_OFFSET_FLAG | 0,
            DescriptorAttributes::writable_data(),
            NamedPropertyCachePath::OwnData,
            1,
            [None; PROPERTY_CACHE_MAX_DEPENDENCIES],
        );
        let handler = NamedPropertyInlineWriteHandler::from_entry(entry);
        assert!(handler.is_valid());
        assert_eq!(handler.source_shape(), Some(shape));
        assert_eq!(handler.target_shape(), Some(shape));
    }

    #[test]
    fn from_outline_entry_is_none() {
        // MVP: outline-slot writes stay on the Rust probe.
        let shape = ShapeId::from_raw(5).expect("non-zero");
        let entry = NamedPropertyCacheEntry::new(
            shape,
            ObjectRef::from_raw(1).expect("non-zero"),
            shape,
            7, // INLINE_SLOT_OFFSET_FLAG NOT set → outline slot 7
            DescriptorAttributes::writable_data(),
            NamedPropertyCachePath::OwnData,
            1,
            [None; PROPERTY_CACHE_MAX_DEPENDENCIES],
        );
        let handler = NamedPropertyInlineWriteHandler::from_entry(entry);
        assert!(!handler.is_valid());
    }

    #[test]
    fn from_prototype_data_entry_is_none() {
        let receiver = ShapeId::from_raw(1).expect("non-zero");
        let holder = ShapeId::from_raw(2).expect("non-zero");
        let entry = NamedPropertyCacheEntry::new(
            receiver,
            ObjectRef::from_raw(1).expect("non-zero"),
            holder,
            INLINE_SLOT_OFFSET_FLAG | 0,
            DescriptorAttributes::writable_data(),
            NamedPropertyCachePath::PrototypeData,
            2,
            [None; PROPERTY_CACHE_MAX_DEPENDENCIES],
        );
        let handler = NamedPropertyInlineWriteHandler::from_entry(entry);
        assert!(!handler.is_valid());
    }

    #[test]
    fn none_sentinel_is_invalid() {
        assert!(!NamedPropertyInlineWriteHandler::NONE.is_valid());
    }
}
```

- [ ] **Step 1.2: Run tests, verify FAIL**

```bash
cargo test -p lyng-objects inline_write_handler_tests 2>&1 | tail -10
```

Expected: compile error — `NamedPropertyInlineWriteHandler` doesn't exist yet.

- [ ] **Step 1.3: Implement `NamedPropertyInlineWriteHandler`**

Add to `crates/objects/src/shapes.rs` after the existing `NamedPropertyHandler` impl block (around line 266):

```rust
/// Bit-packed monomorphic `OwnDataInlineWrite` cache handler.
///
/// Layout — two 64-bit words:
///   `handler_bits` (same layout as [`NamedPropertyHandler`]):
///     bits  0..30  inline slot index
///     bit   30     writable flag (`HANDLER_WRITABLE_FLAG`)
///     bit   31     inline-slot flag (`INLINE_SLOT_OFFSET_FLAG`) — must be set
///     bits 32..64  source `ShapeId` raw `u32` (pre-write shape, `NonZero`)
///   `target_bits`:
///     bits  0..32  target `ShapeId` raw `u32` (post-write shape; equal to
///                  source for non-transitioning writes)
///     bits 32..64  reserved (currently always zero)
///
/// `is_valid()` is `false` when `handler_bits == 0` (the NONE sentinel) OR
/// when the inline-slot flag is unset. Both target and source must be
/// non-zero `ShapeId`s.
///
/// **ShapeId stability assumption:** the handler stores raw shape ids that
/// rely on the existing slab persistence in `ObjectRuntime::shape_metadata`.
/// If shape collection is ever introduced, this struct's consumers (the
/// asm fast path + the IC cache hit-path verifier) would need to participate
/// in pinning or sweep — see the design doc §6.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NamedPropertyInlineWriteHandler {
    handler_bits: u64,
    target_bits: u64,
}

impl NamedPropertyInlineWriteHandler {
    /// Sentinel value indicating "no cache handler available".
    pub const NONE: Self = Self {
        handler_bits: 0,
        target_bits: 0,
    };

    #[inline]
    #[must_use]
    pub const fn handler_bits(self) -> u64 {
        self.handler_bits
    }

    #[inline]
    #[must_use]
    pub const fn target_bits(self) -> u64 {
        self.target_bits
    }

    /// Build a write handler from a cache entry. Returns [`Self::NONE`] for
    /// entries the asm write fast path cannot service:
    /// - `PrototypeData` paths (no own-data write semantics)
    /// - Multi-dependency entries (more than one shape guard required)
    /// - Out-of-line slot entries (MVP scope; deferred)
    /// - Slot offsets exceeding 30 bits (defensive)
    #[inline]
    #[must_use]
    pub const fn from_entry(entry: NamedPropertyCacheEntry) -> Self {
        match entry.path() {
            NamedPropertyCachePath::OwnData | NamedPropertyCachePath::OwnDataTransition => {}
            NamedPropertyCachePath::PrototypeData => return Self::NONE,
        }
        if entry.dependency_count() != 1 {
            return Self::NONE;
        }
        let encoded_offset = entry.slot_offset();
        if encoded_offset & INLINE_SLOT_OFFSET_FLAG == 0 {
            // MVP: outline-slot writes are not asm-cacheable.
            return Self::NONE;
        }
        let offset_bits = encoded_offset & INLINE_SLOT_OFFSET_MASK;
        if offset_bits > HANDLER_SLOT_OFFSET_MASK {
            return Self::NONE;
        }
        let source_shape = entry.receiver_shape();
        let target_shape = entry.holder_shape();
        let writable_bit = if entry.attrs().writable() {
            HANDLER_WRITABLE_FLAG
        } else {
            0
        };
        let low = INLINE_SLOT_OFFSET_FLAG | writable_bit | offset_bits;
        let handler_bits = ((source_shape.get() as u64) << 32) | (low as u64);
        let target_bits = target_shape.get() as u64;
        Self {
            handler_bits,
            target_bits,
        }
    }

    /// Returns the cached source (pre-write) `ShapeId`, or `None` when this
    /// is the [`Self::NONE`] sentinel.
    #[inline]
    #[must_use]
    pub const fn source_shape(self) -> Option<ShapeId> {
        ShapeId::from_raw((self.handler_bits >> 32) as u32)
    }

    /// Returns the cached target (post-write) `ShapeId`, or `None` when this
    /// is the [`Self::NONE`] sentinel.
    #[inline]
    #[must_use]
    pub const fn target_shape(self) -> Option<ShapeId> {
        ShapeId::from_raw(self.target_bits as u32)
    }

    /// Decoded slot location. Only meaningful when [`Self::is_valid`].
    /// MVP: always returns `SlotLocation::Inline` (outline writes are
    /// filtered to [`Self::NONE`] by [`Self::from_entry`]).
    #[inline]
    #[must_use]
    pub const fn slot_location(self) -> SlotLocation {
        let low = self.handler_bits as u32;
        let offset = low & HANDLER_SLOT_OFFSET_MASK;
        SlotLocation::Inline(offset)
    }

    /// `true` when the cached property is writable. Asm stores must check
    /// this and bail to the Rust probe on a read-only hit so the
    /// strict-mode TypeError contract stays authoritative.
    #[inline]
    #[must_use]
    pub const fn writable(self) -> bool {
        (self.handler_bits as u32) & HANDLER_WRITABLE_FLAG != 0
    }

    /// `true` when this handler carries a valid `OwnDataInlineWrite` cache
    /// path. `false` for [`Self::NONE`].
    #[inline]
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.handler_bits != 0
    }
}
```

- [ ] **Step 1.4: Run handler tests, verify PASS**

```bash
cargo test -p lyng-objects inline_write_handler_tests 2>&1 | tail -5
```

Expected: all 5 tests pass.

### Step 1.5: Write failing test for `AdaptiveOwnWrite` watchpoint variant

In `crates/objects/src/watchpoint.rs`, locate the existing test module (`mod tests`). Add this test:

```rust
#[test]
fn adaptive_own_write_observer_carries_code_slot_generation() {
    let observer = ShapeInvalidationObserver::AdaptiveOwnWrite {
        code: CodeRef::from_raw(7).expect("non-zero"),
        slot: FeedbackSlotId::from_raw(3).expect("non-zero"),
        generation: 5,
    };
    match observer {
        ShapeInvalidationObserver::AdaptiveOwnWrite { code, slot, generation } => {
            assert_eq!(code.get(), 7);
            assert_eq!(slot.get(), 3);
            assert_eq!(generation, 5);
        }
        _ => panic!("expected AdaptiveOwnWrite variant"),
    }
}
```

- [ ] **Step 1.6: Run, verify FAIL** (`AdaptiveOwnWrite` doesn't exist)

```bash
cargo test -p lyng-objects watchpoint::tests::adaptive_own_write 2>&1 | tail -5
```

- [ ] **Step 1.7: Add `AdaptiveOwnWrite` variant**

In `crates/objects/src/watchpoint.rs`, find the `ShapeInvalidationObserver` enum (search for `pub enum ShapeInvalidationObserver`) and add a new variant. The existing `AdaptiveProtoLoad` variant shows the exact pattern:

```rust
pub enum ShapeInvalidationObserver {
    /// Test-only: records the fire event into a `Vec<u64>` so unit tests
    /// can assert "this transition fired exactly the watchpoints I
    /// registered."
    #[cfg(test)]
    Recording { token: u64 },

    /// Spec 2 Phase A: AdaptiveProtoLoad — clears the IC slot at
    /// `(code, slot)` if its generation still matches `generation`.
    AdaptiveProtoLoad {
        code: CodeRef,
        slot: FeedbackSlotId,
        generation: u32,
    },

    /// Spec 2 transition-IC: AdaptiveOwnWrite — clears the
    /// `OwnDataInlineWrite` IC slot at `(code, slot)` if its generation
    /// still matches `generation`. Registered by the slow path on the
    /// receiver's source shape when a monomorphic write entry is
    /// installed. Fires when the source shape goes dictionary, has a
    /// new property added, or receives a prototype mutation — the slot
    /// is cleared and re-installs on the next slow-path miss.
    AdaptiveOwnWrite {
        code: CodeRef,
        slot: FeedbackSlotId,
        generation: u32,
    },
}
```

- [ ] **Step 1.8: Run, verify PASS**

```bash
cargo test -p lyng-objects watchpoint::tests::adaptive_own_write 2>&1 | tail -5
```

- [ ] **Step 1.9: Commit**

```bash
git add crates/objects/src/shapes.rs crates/objects/src/watchpoint.rs
git commit -m "objects: add NamedPropertyInlineWriteHandler + AdaptiveOwnWrite

Foundation for the transition-aware write IC. The handler packs
(source_shape, target_shape, inline_slot, writable_flag) into 16 bytes
mirroring the existing NamedPropertyHandler layout. The watchpoint
variant carries the (code, slot, generation) tuple needed by
clear_ic_slot_if_generation_matches.

Both are passive additions — no callers yet."
```

---

## Task 2: Wire `AdaptiveOwnWrite` fire dispatch in `Agent`

**Goal:** Make `agent.fire_watchpoints_for_shape(...)` clear the matching IC slot when an `AdaptiveOwnWrite` observer fires. Mirrors the existing `AdaptiveProtoLoad` dispatch.

**Files:**
- Modify: `crates/env/src/agent.rs` (the `fire_watchpoints_for_shape` match-on-`Watchpoint` block — search for `AdaptiveProtoLoad` to find it)

### Step 2.1: Write failing test

In `crates/objects/src/watchpoint.rs` tests module (or `crates/env/src/` if there's an agent test module — check both):

```rust
#[test]
fn fire_watchpoints_for_shape_dispatches_adaptive_own_write() {
    // Setup: an Agent + Vm where (code, slot) has an installed
    // PropertyIcState with generation 3. Register an AdaptiveOwnWrite
    // observer on shape S. Fire watchpoints for shape S. Expected:
    // the IC state at (code, slot) is cleared (PropertyIcState::default()).
    //
    // Use the same test scaffold as existing AdaptiveProtoLoad fire tests
    // (grep `fire_watchpoints_for_shape` in this file). The new test
    // mirrors that pattern with the new observer variant.
    todo!("write the test by mirroring existing AdaptiveProtoLoad fire test");
}
```

Actually, locate the existing test for `AdaptiveProtoLoad` fire dispatch by grepping `rg "AdaptiveProtoLoad" crates/env/ crates/objects/`. Mirror it:

```rust
// In whatever module the AdaptiveProtoLoad fire test lives:
#[test]
fn fire_watchpoints_for_shape_clears_ic_via_adaptive_own_write() {
    use lyng_objects::watchpoint::{ShapeInvalidationObserver, Watchpoint};
    let (mut agent, mut vm) = make_test_agent_and_vm(); // existing helper
    let code = install_dummy_code(&mut agent, &mut vm); // existing helper
    let slot = FeedbackSlotId::from_raw(1).expect("non-zero");
    let shape = ShapeId::from_raw(42).expect("non-zero");

    // Pre-populate the IC slot (Phase D.4 lazy slab — install the slab + entry).
    install_dummy_property_ic_state(&mut vm, code, slot, /* generation */ 7);

    // Register AdaptiveOwnWrite on shape — payload matches the installed
    // generation.
    agent
        .objects_mut()
        .watchpoint_set_mut(shape)
        .register(Watchpoint::ShapeInvalidation {
            observer: ShapeInvalidationObserver::AdaptiveOwnWrite {
                code,
                slot,
                generation: 7,
            },
        })
        .expect("register on Cleared/Watched");

    agent.fire_watchpoints_for_shape(shape);

    // IC slot must have been cleared.
    assert!(vm.property_ic_state(code, slot).is_none());
}
```

(If the helpers `make_test_agent_and_vm`, `install_dummy_code`, `install_dummy_property_ic_state` don't yet exist with those exact names, grep for the AdaptiveProtoLoad fire test's setup and reuse its helpers verbatim.)

- [ ] **Step 2.2: Run, verify FAIL** (`AdaptiveOwnWrite` not yet dispatched)

```bash
cargo test -p lyng-env fire_watchpoints_for_shape_clears_ic_via_adaptive_own_write 2>&1 | tail -10
```

Expected: test panics or asserts because IC slot was NOT cleared.

- [ ] **Step 2.3: Add `AdaptiveOwnWrite` arm to the dispatch match**

Open `crates/env/src/agent.rs` and find the function `fire_watchpoints_for_shape`. Inside its `for wp in fired { match wp { ... } }` loop, find the `AdaptiveProtoLoad` arm. Add the `AdaptiveOwnWrite` arm right after it (same body — both call `clear_ic_slot_if_generation_matches`):

```rust
// Inside the inner observer match:
ShapeInvalidationObserver::AdaptiveProtoLoad {
    code,
    slot,
    generation,
} => {
    self.with_heap_and_objects(|_, _objects| {
        // (Existing AdaptiveProtoLoad body — find it via rg.)
    });
    self.vm_mut()
        .clear_ic_slot_if_generation_matches(code, slot, generation);
}
ShapeInvalidationObserver::AdaptiveOwnWrite {
    code,
    slot,
    generation,
} => {
    self.vm_mut()
        .clear_ic_slot_if_generation_matches(code, slot, generation);
}
```

The exact surrounding code may differ — use the AdaptiveProtoLoad arm as the structural template and replicate it for AdaptiveOwnWrite. The semantic action is identical: call `clear_ic_slot_if_generation_matches` with the payload triple.

- [ ] **Step 2.4: Run, verify PASS**

```bash
cargo test -p lyng-env fire_watchpoints_for_shape_clears_ic_via_adaptive_own_write 2>&1 | tail -5
```

- [ ] **Step 2.5: Run full workspace tests, verify no regressions**

```bash
cargo test --workspace --no-fail-fast 2>&1 | tail -3
```

Expected: 2663 passed (2662 existing + 1 new), 19 ignored.

- [ ] **Step 2.6: Commit**

```bash
git add crates/env/src/agent.rs crates/objects/src/watchpoint.rs
git commit -m "env/agent: dispatch AdaptiveOwnWrite watchpoint to clear IC slot

Mirrors AdaptiveProtoLoad dispatch — when the observer fires for a
shape transitioning to dictionary mode or receiving an unrelated
property addition, the matching (code, slot) IC entry is cleared via
clear_ic_slot_if_generation_matches. The asm fast path for the new
transition IC kind will rely on this for invalidation safety."
```

---

## Task 3: Add `monomorphic_own_inline_write_handler` sidecar to `PropertyIcState`

**Goal:** Extend the per-slot IC state struct with storage for one `OwnDataInlineWrite` entry. Keep symmetric with the existing read-side `monomorphic_own_data_handler` field.

**Files:**
- Modify: `crates/vm/src/vm/ic_state/property.rs` (the `PropertyIcState` struct + its `refresh_sidecars` method)

### Step 3.1: Write failing test

Add to `crates/vm/src/vm/ic_state/property.rs` test module (or in `crates/vm/src/tests/inline_caches.rs` if there's no inline test module):

```rust
#[test]
fn refresh_sidecars_populates_monomorphic_own_inline_write_handler() {
    use lyng_objects::{
        DescriptorAttributes, INLINE_SLOT_OFFSET_FLAG, NamedPropertyCacheEntry,
        NamedPropertyCachePath, NamedPropertyInlineWriteHandler, ObjectRef, ShapeId,
    };
    let source = ShapeId::from_raw(7).expect("non-zero");
    let target = ShapeId::from_raw(11).expect("non-zero");
    let entry = NamedPropertyCacheEntry::new(
        source,
        ObjectRef::from_raw(1).expect("non-zero"),
        target,
        INLINE_SLOT_OFFSET_FLAG | 2,
        DescriptorAttributes::writable_data(),
        NamedPropertyCachePath::OwnDataTransition,
        1,
        [None; lyng_objects::PROPERTY_CACHE_MAX_DEPENDENCIES],
    );

    let mut state = PropertyIcState::new();
    state.cache_state = InlineCacheState::Monomorphic;
    state.entry_count = 1;
    state.entries[0] = Some(entry);
    state.refresh_sidecars();

    let expected = NamedPropertyInlineWriteHandler::from_entry(entry);
    assert_eq!(state.monomorphic_own_inline_write_handler, expected);
    assert!(state.monomorphic_own_inline_write_handler.is_valid());
}
```

- [ ] **Step 3.2: Run, verify FAIL** (field doesn't exist)

```bash
cargo test -p lyng-vm refresh_sidecars_populates_monomorphic_own_inline_write_handler 2>&1 | tail -10
```

- [ ] **Step 3.3: Add the sidecar field + initialize it**

Open `crates/vm/src/vm/ic_state/property.rs`. Locate the `PropertyIcState` struct (around line 26). Add a new field after `monomorphic_proto_data_handler`:

```rust
pub struct PropertyIcState {
    pub cache_state: InlineCacheState,
    pub entry_count: u8,
    pub entries: [Option<NamedPropertyCacheEntry>; POLY_LIMIT],
    pub monomorphic_own_data_handler: NamedPropertyHandler,
    pub monomorphic_proto_data_handler: NamedPropertyProtoHandler,
    /// Monomorphic OwnDataInlineWrite sidecar. `NamedPropertyInlineWriteHandler::NONE`
    /// when not applicable (non-OwnData/non-OwnDataTransition, Poly, Mega, Uninit,
    /// or out-of-line slot — see [`NamedPropertyInlineWriteHandler::from_entry`]).
    /// Phase D successor of the failed eager/lean asm attempts: this is the
    /// asm-readable cache used by the `op_assign_named_property` write fast path.
    pub monomorphic_own_inline_write_handler: NamedPropertyInlineWriteHandler,
    pub polymorphic_own_data_handlers: [NamedPropertyHandler; POLY_LIMIT],
    pub generation: u32,
    pub execution_count: u32,
    pub is_cleared: bool,
}
```

Update the `import` at the top of the file (add `NamedPropertyInlineWriteHandler` to the `use lyng_objects::{...}` line).

Update `PropertyIcState::new()` (around line 56) to initialize the new field:

```rust
impl PropertyIcState {
    pub const fn new() -> Self {
        Self {
            cache_state: InlineCacheState::Uninitialized,
            entry_count: 0,
            entries: [None; POLY_LIMIT],
            monomorphic_own_data_handler: NamedPropertyHandler::NONE,
            monomorphic_proto_data_handler: NamedPropertyProtoHandler::NONE,
            monomorphic_own_inline_write_handler: NamedPropertyInlineWriteHandler::NONE,
            polymorphic_own_data_handlers: [NamedPropertyHandler::NONE; POLY_LIMIT],
            generation: 0,
            execution_count: 0,
            is_cleared: false,
        }
    }
```

Update `refresh_sidecars` (around line 80). Find the existing logic that populates `monomorphic_own_data_handler`. Add a parallel block:

```rust
pub const fn refresh_sidecars(&mut self) {
    self.monomorphic_own_data_handler = NamedPropertyHandler::NONE;
    self.monomorphic_proto_data_handler = NamedPropertyProtoHandler::NONE;
    self.monomorphic_own_inline_write_handler = NamedPropertyInlineWriteHandler::NONE;
    let mut i = 0;
    while i < POLY_LIMIT {
        self.polymorphic_own_data_handlers[i] = NamedPropertyHandler::NONE;
        i += 1;
    }
    match self.cache_state {
        InlineCacheState::Monomorphic => {
            // Existing read-side refresh — find this block and add the
            // mirroring write-side projection right after it.
            if let Some(entry) = self.entries[0] {
                // (existing) self.monomorphic_own_data_handler = NamedPropertyHandler::from_entry(entry);
                // (existing) self.monomorphic_proto_data_handler = NamedPropertyProtoHandler::from_entry(entry);

                // NEW (after the existing two lines):
                self.monomorphic_own_inline_write_handler =
                    NamedPropertyInlineWriteHandler::from_entry(entry);
            }
        }
        InlineCacheState::Polymorphic => {
            // (existing block — leave alone; polymorphic-write asm not in MVP)
        }
        InlineCacheState::Uninitialized | InlineCacheState::Megamorphic => {}
    }
}
```

(The exact existing structure of `refresh_sidecars` will determine the precise edits — preserve every existing line, only ADD the write-side projection.)

- [ ] **Step 3.4: Run, verify PASS**

```bash
cargo test -p lyng-vm refresh_sidecars_populates_monomorphic_own_inline_write_handler 2>&1 | tail -5
```

- [ ] **Step 3.5: Verify all VM tests still pass**

```bash
cargo test -p lyng-vm --no-fail-fast 2>&1 | tail -3
```

- [ ] **Step 3.6: Commit**

```bash
git add crates/vm/src/vm/ic_state/property.rs
git commit -m "vm/ic_state: add OwnDataInlineWrite sidecar to PropertyIcState

Mirror of monomorphic_own_data_handler for the write fast path.
Populated by refresh_sidecars() on every cache state transition,
zeroed on Uninit/Mega/Poly. No callers consume it yet — that's
Task 4 (projection) and Task 6 (asm reads the projected bits).

The handler stays NONE for outline-slot writes and PrototypeData
plans, keeping the asm fast path's responsibility narrow."
```

---

## Task 4: Extend `project_property_into_meta` to write mode 5

**Goal:** Project the new sidecar into asm-readable PropertyMetadata bits. When `monomorphic_own_inline_write_handler.is_valid()`, write mode = 5, handler_bits = source+slot+flags, aux_bits = target_shape.

**Files:**
- Modify: `crates/vm/src/vm/feedback.rs` — `project_property_into_meta` function (around line 1829) AND `named_llint_load_header_from_state` (around line 869 — the projection input)

### Step 4.1: Understand the existing projection flow

Read `crates/vm/src/vm/feedback.rs:1829-1900` (the `project_property_into_meta` function). It takes an `llint_header` (whatever the existing helper produces) and writes mode + handler_bits + aux_bits into the `PropertyMetadata`. Also read `named_llint_load_header_from_state` which produces the header.

We need a new mode byte constant. Find where existing mode constants live (search for `mode: u8` + `= 1` in the file, or `LLINT_IC_MODE_NAMED_OWN_INLINE_LOAD` if defined). Add a parallel constant:

```rust
// In whatever module defines the mode-byte constants (usually near
// project_property_into_meta or in metadata_table/property.rs):
pub const LLINT_IC_MODE_NAMED_OWN_INLINE_WRITE: u8 = 5;
```

### Step 4.2: Write failing test for the projection

In `crates/vm/src/vm/feedback.rs` tests module:

```rust
#[test]
fn project_property_into_meta_writes_mode_5_for_inline_write() {
    use crate::vm::metadata_table::PropertyMetadata;
    use lyng_objects::{
        DescriptorAttributes, INLINE_SLOT_OFFSET_FLAG, NamedPropertyCacheEntry,
        NamedPropertyCachePath, NamedPropertyInlineWriteHandler, ObjectRef, ShapeId,
    };

    let source = ShapeId::from_raw(7).expect("non-zero");
    let target = ShapeId::from_raw(11).expect("non-zero");
    let entry = NamedPropertyCacheEntry::new(
        source,
        ObjectRef::from_raw(1).expect("non-zero"),
        target,
        INLINE_SLOT_OFFSET_FLAG | 3,
        DescriptorAttributes::writable_data(),
        NamedPropertyCachePath::OwnDataTransition,
        1,
        [None; lyng_objects::PROPERTY_CACHE_MAX_DEPENDENCIES],
    );

    let mut state = PropertyIcState::new();
    state.cache_state = InlineCacheState::Monomorphic;
    state.entry_count = 1;
    state.entries[0] = Some(entry);
    state.refresh_sidecars();

    let mut meta = PropertyMetadata::default();
    Vm::project_property_into_meta(
        Vm::named_llint_load_header_from_state(&state),
        state.generation,
        state.execution_count,
        &mut meta,
    );

    // For now, the read-side projection takes precedence (mode 1 with
    // NamedPropertyHandler::NONE for transitioning entries). After this
    // task: write-side overrides when the write handler is valid AND
    // the slot is part of a write-style feedback site.
    //
    // NOTE: this test expects "always project write mode when valid" —
    // verify the actual call-site distinguishes read vs write slots.
    // See Task 5 for the routing concern.
    assert_eq!(meta.mode, LLINT_IC_MODE_NAMED_OWN_INLINE_WRITE);
    let expected_handler = NamedPropertyInlineWriteHandler::from_entry(entry);
    assert_eq!(meta.handler_bits, expected_handler.handler_bits());
    assert_eq!(meta.aux_bits, expected_handler.target_bits());
}
```

- [ ] **Step 4.3: Run, verify FAIL**

```bash
cargo test -p lyng-vm project_property_into_meta_writes_mode_5_for_inline_write 2>&1 | tail -10
```

- [ ] **Step 4.4: Add the new mode constant + projection branch**

The projection currently uses `named_llint_load_header_from_state` to build a `header` whose mode byte determines the projection. The header type may be a tuple/struct. We need to extend it to carry write-mode information.

Option (a): Add a new field to the header indicating "this slot wants mode 5 with these handler+target bits."

Option (b): Make `project_property_into_meta` look at `state.monomorphic_own_inline_write_handler` directly (bypass header for write entries).

Read the existing code in `crates/vm/src/vm/feedback.rs:869-880` to determine which approach fits. The simpler choice is (b) — `project_property_into_meta` can take both the read-side header AND a write-side handler, and write the mode based on which is valid (write takes precedence when both are non-NONE, but for a given feedback-slot-kind only one will ever be valid).

Implement:

```rust
// In Vm impl (where project_property_into_meta lives):
pub(super) fn project_property_into_meta(
    llint_header: ReadHeader, // existing type — rename to ReadHeader if helps clarity
    generation: u32,
    execution_count: u32,
    meta: &mut PropertyMetadata,
) {
    // Existing body writes mode 1/2/3/4 based on the read-side header.
    // Add nothing here yet — the write-side projection lives in a NEW
    // function called by the WRITE-SITE install path (see Task 5).
    // (Existing body preserved verbatim.)
}

/// Project the write-side cache state into `PropertyMetadata`. Called from
/// the assign opcode's slow-path install instead of (not alongside)
/// `project_property_into_meta` for read-side slots.
pub(super) fn project_property_write_into_meta(
    write_handler: NamedPropertyInlineWriteHandler,
    generation: u32,
    execution_count: u32,
    meta: &mut PropertyMetadata,
) {
    if write_handler.is_valid() {
        meta.mode = LLINT_IC_MODE_NAMED_OWN_INLINE_WRITE;
        meta.handler_bits = write_handler.handler_bits();
        meta.aux_bits = write_handler.target_bits();
    } else {
        // No asm-cacheable entry — zero the metadata so the asm
        // mode-byte check bails to the Rust probe.
        meta.mode = 0;
        meta.handler_bits = 0;
        meta.aux_bits = 0;
    }
    meta.generation = generation;
    meta.execution_count = execution_count;
}
```

Update the test (it called `project_property_into_meta` but the new function is `project_property_write_into_meta`):

```rust
Vm::project_property_write_into_meta(
    state.monomorphic_own_inline_write_handler,
    state.generation,
    state.execution_count,
    &mut meta,
);
```

- [ ] **Step 4.5: Run, verify PASS**

```bash
cargo test -p lyng-vm project_property_into_meta_writes_mode_5_for_inline_write 2>&1 | tail -5
```

- [ ] **Step 4.6: Commit**

```bash
git add crates/vm/src/vm/feedback.rs crates/vm/src/vm/metadata_table/property.rs
git commit -m "vm/feedback: project OwnDataInlineWrite into PropertyMetadata mode 5

Introduces project_property_write_into_meta as the write-side
counterpart to project_property_into_meta. Writes mode = 5,
handler_bits = source+slot+writable_flag, aux_bits = target_shape.

The function is called by the assign opcode's install path (Task 5).
Read-side slots continue to use the existing projection unchanged."
```

---

## Task 5: Slow-path routing — install write entries via the new IC kind

**Goal:** When `op_assign_named_property_slow_rs` runs, plan the cache entry, install it into `PropertyIcState`, refresh sidecars, register the `AdaptiveOwnWrite` watchpoint, and project mode 5 into `PropertyMetadata`.

**Files:**
- Modify: `crates/vm/src/vm/feedback.rs` — extend `named_property_install_slow_path` (around line 843) OR add a new `named_property_write_install_slow_path`. AND ensure `record_named_property_cache_entry` routes correctly.
- Modify: `crates/vm/src/vm/semantics/property.rs` OR `crates/vm/src/vm/names.rs` — the actual write opcode slow path that calls into the IC machinery.

### Step 5.1: Trace the existing assign slow-path observation flow

Run:

```bash
rg "purpose: NamedPropertyCachePurpose::Store" crates/vm/src/vm/
```

Each result is a site that observes an assign for cache install. List them in your scratch notes — these are the call sites you'll route into the new IC kind. From the earlier recon, expected sites: `crates/vm/src/vm/dispatch/property.rs:530`, `crates/vm/src/vm/dispatch/property.rs:1314`, `crates/vm/src/vm/names.rs:800`, `crates/vm/src/vm/names.rs:933`.

For each, the flow is:

```rust
self.observe_named_property_slow_path(
    agent,
    code,
    Some(slot),
    receiver,
    atom,
    NamedPropertyCachePurpose::Store,
);
```

This routes into `observe_named_property_slow_path` → `record_named_property_cache_entry` → `named_property_install_slow_path` → `named_property_observe_slow_path_on_state` + `project_property_into_meta`.

### Step 5.2: Write failing test (end-to-end install)

In `crates/vm/src/tests/inline_caches.rs`, add:

```rust
#[test]
fn assign_named_property_slow_path_installs_own_inline_write_entry() {
    use crate::vm::{LLINT_IC_MODE_NAMED_OWN_INLINE_WRITE, Vm};

    let mut agent = make_test_agent(); // existing helper
    let mut vm = Vm::new();
    // Compile and install a tiny script:
    //   function f(o) { o.x = 1; }
    //   const a = {};
    //   f(a);
    //   f({});  // second call with a fresh object — same source shape, hot
    let installed = compile_and_install(&mut agent, &mut vm, r#"
        function f(o) { o.x = 1; }
        f({});
        f({});
    "#);

    // After two calls with the Object.extend transition pattern, the
    // IC state at f's slot should be Monomorphic with a valid write
    // handler. The PropertyMetadata mode byte should be 5.
    let (code, slot) = installed.named_property_assign_slot(); // existing helper
    let state = vm.property_ic_state(code, slot).expect("installed");
    assert_eq!(state.cache_state, InlineCacheState::Monomorphic);
    assert!(state.monomorphic_own_inline_write_handler.is_valid());

    let meta = vm.metadata_table_view(code).property(slot.get());
    assert_eq!(meta.mode, LLINT_IC_MODE_NAMED_OWN_INLINE_WRITE);
}
```

If `installed.named_property_assign_slot()` or `compile_and_install` don't exist exactly, grep `crates/vm/src/tests/inline_caches.rs` for the closest existing IC-test scaffold and adapt.

- [ ] **Step 5.3: Run, verify FAIL** (currently no mode 5 written)

```bash
cargo test -p lyng-vm assign_named_property_slow_path_installs_own_inline_write_entry 2>&1 | tail -10
```

- [ ] **Step 5.4: Detect write-purpose slots in the install path**

Modify `named_property_install_slow_path` (in `crates/vm/src/vm/feedback.rs:843`) to BOTH:
1. Continue calling the existing `project_property_into_meta` (for backward compat — read sites stay unchanged)
2. Additionally call `project_property_write_into_meta` (Task 4) when the IcState's `monomorphic_own_inline_write_handler` is valid

But there's a routing concern: read sites and write sites have different slot IDs (different opcodes allocate different slots), so a given slot is exclusively one or the other. The PropertyMetadata for a write slot should have mode = 5 (not mode = 1 from the read-side projection).

The simplest scheme: **derive write-vs-read from the cache plan's purpose**, which is already passed through `observe_named_property_slow_path`. Route the install accordingly.

Concrete change to `record_named_property_cache_entry` (around line 820):

```rust
fn record_named_property_cache_entry(
    &mut self,
    agent: &mut Agent,
    code: CodeRef,
    slot: FeedbackSlotId,
    plan: Option<NamedPropertyCacheEntry>,
    purpose: NamedPropertyCachePurpose,  // NEW PARAMETER
) {
    // Existing watchpoint registration for PrototypeData stays.
    if let Some(plan_entry) = plan
        && plan_entry.path() == NamedPropertyCachePath::PrototypeData
        && !Self::register_proto_chain_watchpoints(self, agent, code, slot, plan_entry)
    {
        return;
    }
    self.named_property_install_slow_path(code, slot, plan, purpose);
}
```

And `named_property_install_slow_path`:

```rust
fn named_property_install_slow_path(
    &mut self,
    code: CodeRef,
    slot: FeedbackSlotId,
    plan: Option<NamedPropertyCacheEntry>,
    purpose: NamedPropertyCachePurpose,
) {
    let Self {
        property_ic_states,
        polymorphic_chains,
        ..
    } = self;
    let index = code_index(code);
    let slot_zero = (slot.get() - 1) as usize;
    let slab = property_ic_states[index]
        .as_deref_mut()
        .expect("property_ic_states slab must be allocated at install");
    let state = slab[slot_zero].get_or_insert_with(PropertyIcState::default);
    let chain_slot = &mut polymorphic_chains[index]
        .as_deref_mut()
        .expect("polymorphic_chains slab must be allocated at install")[slot_zero];
    Self::named_property_observe_slow_path_on_state(state, chain_slot, plan);

    // Write asm-readable bits to PropertyMetadata. Branch on purpose so
    // read slots and write slots get the right mode byte.
    let generation = state.generation;
    let execution_count = state.execution_count;
    let write_handler = state.monomorphic_own_inline_write_handler;
    if let Some(table) = self.metadata_table_mut(code) {
        let meta = table.property_mut(slot.get());
        match purpose {
            NamedPropertyCachePurpose::Store => {
                Self::project_property_write_into_meta(
                    write_handler,
                    generation,
                    execution_count,
                    meta,
                );
            }
            NamedPropertyCachePurpose::Load => {
                let llint_header = Self::named_llint_load_header_from_state(state);
                Self::project_property_into_meta(
                    llint_header,
                    generation,
                    execution_count,
                    meta,
                );
            }
        }
    }
}
```

Then thread `purpose` through `observe_named_property_slow_path` (line 773) and `observe_named_property_cache_entry` (line 803). Update the 4 callers from Step 5.1 — they all already pass `NamedPropertyCachePurpose::Store` or `Load`.

- [ ] **Step 5.5: Run, verify PASS**

```bash
cargo test -p lyng-vm assign_named_property_slow_path_installs_own_inline_write_entry 2>&1 | tail -5
```

- [ ] **Step 5.6: Run full workspace tests**

```bash
cargo test --workspace --no-fail-fast 2>&1 | tail -3
```

Expected: 2664+ passed (test count grows with each task).

- [ ] **Step 5.7: Commit**

```bash
git add crates/vm/src/vm/feedback.rs crates/vm/src/tests/inline_caches.rs
git commit -m "vm/feedback: route Store-purpose IC installs through write projection

Thread NamedPropertyCachePurpose through the install path. When the
slow path observes a Store, project the write-side handler (mode = 5)
into PropertyMetadata; Load slots continue to use the existing
project_property_into_meta. End-to-end test covers Object.extend
hot-loop install pattern."
```

---

## Task 6: Register `AdaptiveOwnWrite` watchpoint on install

**Goal:** When the slow path installs an `OwnDataInlineWrite` entry, register a watchpoint on the source shape so the entry clears on shape invalidation.

**Files:**
- Modify: `crates/vm/src/vm/feedback.rs` — `record_named_property_cache_entry` (around line 820)

### Step 6.1: Write failing test

In `crates/vm/src/tests/inline_caches.rs`:

```rust
#[test]
fn dictionary_transition_clears_own_inline_write_ic_via_adaptive_own_write() {
    use crate::vm::{LLINT_IC_MODE_NAMED_OWN_INLINE_WRITE, Vm};

    let mut agent = make_test_agent();
    let mut vm = Vm::new();
    let installed = compile_and_install(&mut agent, &mut vm, r#"
        function f(o) { o.x = 1; }
        const sample = {};
        f(sample);
        f(sample);
    "#);
    let (code, slot) = installed.named_property_assign_slot();

    // Pre-condition: cache is hot (mode = 5).
    assert_eq!(
        vm.metadata_table_view(code).property(slot.get()).mode,
        LLINT_IC_MODE_NAMED_OWN_INLINE_WRITE,
    );

    // Force the source shape into dictionary mode. The specific API may be
    // `agent.objects_mut().ensure_named_property_dictionary(receiver)` —
    // grep the existing T10/T11 dictionary-fire tests for the canonical
    // helper.
    let receiver = installed.last_sample_receiver(); // existing helper
    agent.ensure_named_property_dictionary(receiver);

    // The watchpoint should have fired and cleared the IC slot.
    assert_eq!(
        vm.metadata_table_view(code).property(slot.get()).mode,
        0,
        "AdaptiveOwnWrite watchpoint should have cleared the IC after dictionary transition"
    );
}
```

- [ ] **Step 6.2: Run, verify FAIL** (watchpoint not registered yet)

```bash
cargo test -p lyng-vm dictionary_transition_clears_own_inline_write_ic_via_adaptive_own_write 2>&1 | tail -10
```

- [ ] **Step 6.3: Register the watchpoint on install**

In `crates/vm/src/vm/feedback.rs`, modify `record_named_property_cache_entry` to register the watchpoint when installing an `OwnData`/`OwnDataTransition` plan with `purpose = Store`:

```rust
fn record_named_property_cache_entry(
    &mut self,
    agent: &mut Agent,
    code: CodeRef,
    slot: FeedbackSlotId,
    plan: Option<NamedPropertyCacheEntry>,
    purpose: NamedPropertyCachePurpose,
) {
    // Existing PrototypeData watchpoint registration.
    if let Some(plan_entry) = plan
        && plan_entry.path() == NamedPropertyCachePath::PrototypeData
        && !Self::register_proto_chain_watchpoints(self, agent, code, slot, plan_entry)
    {
        return;
    }

    // NEW: register AdaptiveOwnWrite on the source shape for write
    // installs of OwnData / OwnDataTransition entries.
    if purpose == NamedPropertyCachePurpose::Store
        && let Some(plan_entry) = plan
        && matches!(
            plan_entry.path(),
            NamedPropertyCachePath::OwnData | NamedPropertyCachePath::OwnDataTransition
        )
    {
        // The handler's source_shape IS the receiver's pre-write shape
        // (plan.receiver_shape()). Register on it; bumping generation
        // for the slot happens implicitly because install advances
        // the IcState's generation field.
        let source_shape = plan_entry.receiver_shape();
        let generation = self
            .property_ic_state(code, slot)
            .map(|s| s.generation)
            .unwrap_or(0);
        // Bumping happens AFTER install — get the NEW generation:
        // capture it after we install, register watchpoint with that.
        // Defer the registration until after install.
        let observer = lyng_objects::watchpoint::ShapeInvalidationObserver::AdaptiveOwnWrite {
            code,
            slot,
            generation: generation.wrapping_add(1),
        };
        let _ = agent
            .objects_mut()
            .watchpoint_set_mut(source_shape)
            .register(lyng_objects::watchpoint::Watchpoint::ShapeInvalidation { observer });
        // (Registration on Invalidated returns Err — that's fine; the
        // shape is already invalidated, the next slow-path miss will
        // re-attempt registration on the post-invalidation source shape.)
    }

    self.named_property_install_slow_path(code, slot, plan, purpose);
}
```

Note: this exact code may need adjustment because the `generation` capture happens BEFORE the install advances it. The cleanest fix is to either:
(a) Read the generation AFTER install (move the registration to after `named_property_install_slow_path`), OR
(b) Pre-compute the next generation by inspecting state's current generation + 1.

Use (a) — register after install — so the captured generation matches what `clear_ic_slot_if_generation_matches` will see:

```rust
self.named_property_install_slow_path(code, slot, plan, purpose);

// Register watchpoint AFTER install so generation matches.
if purpose == NamedPropertyCachePurpose::Store
    && let Some(plan_entry) = plan
    && matches!(
        plan_entry.path(),
        NamedPropertyCachePath::OwnData | NamedPropertyCachePath::OwnDataTransition
    )
{
    let source_shape = plan_entry.receiver_shape();
    let generation = self
        .property_ic_state(code, slot)
        .expect("install just populated this slot")
        .generation;
    let observer = lyng_objects::watchpoint::ShapeInvalidationObserver::AdaptiveOwnWrite {
        code,
        slot,
        generation,
    };
    let _ = agent
        .objects_mut()
        .watchpoint_set_mut(source_shape)
        .register(lyng_objects::watchpoint::Watchpoint::ShapeInvalidation { observer });
}
```

- [ ] **Step 6.4: Run, verify PASS**

```bash
cargo test -p lyng-vm dictionary_transition_clears_own_inline_write_ic_via_adaptive_own_write 2>&1 | tail -5
```

- [ ] **Step 6.5: Run full workspace**

```bash
cargo test --workspace --no-fail-fast 2>&1 | tail -3
```

- [ ] **Step 6.6: Commit**

```bash
git add crates/vm/src/vm/feedback.rs crates/vm/src/tests/inline_caches.rs
git commit -m "vm/feedback: register AdaptiveOwnWrite watchpoint on Store install

Mirrors the existing AdaptiveProtoLoad registration pattern. The
watchpoint fires on dictionary transitions, prototype mutations, and
unrelated property additions to the source shape — all of which
invalidate the cached transition arrow. clear_ic_slot_if_generation_matches
zeros the PropertyMetadata so the asm path bails to the Rust probe."
```

---

## Task 7: New asm macros (mode check, target shape load, shape store)

**Goal:** Add the asm DSL macros that the new fast path will use. Each macro emits a small fragment of aarch64 inline asm.

**Files:**
- Modify: `crates/vm/src/dsl/backend/aarch64/feedback.rs` (add `branch_named_own_inline_write_mode!` and `load_named_target_shape!`)
- Modify: `crates/vm/src/dsl/backend/aarch64/operands.rs` OR a new `crates/vm/src/dsl/backend/aarch64/records.rs` (add `store_record_shape!`)

### Step 7.1: Add `branch_named_own_inline_write_mode!`

In `crates/vm/src/dsl/backend/aarch64/feedback.rs`, after the existing `branch_named_own_polymorphic_mode!` (or the closest existing `branch_named_*_mode!` macro — find it via `rg`):

```rust
/// Branch to `$label` unless the metadata-table property entry is a
/// monomorphic `OwnDataInlineWrite` header. `mode == 5` —
/// `LLINT_IC_MODE_NAMED_OWN_INLINE_WRITE`. The packed handler word
/// carries (source_shape, slot, writable_flag, inline_flag) in the
/// same layout as the read-side own-inline mode; `aux_bits` carries
/// the target shape.
#[macro_export]
macro_rules! branch_named_own_inline_write_mode {
    ($entry:tt, $label:tt) => {
        concat!(
            "ldrb   w16, [x",
            stringify!($entry),
            ", {feedback_mode}]\n",
            "cmp    w16, #5\n",
            "b.ne   ",
            stringify!($label),
            "\n",
        )
    };
}
```

### Step 7.2: Add `load_named_target_shape!`

Same file, after the new branch macro:

```rust
/// Load the target shape from the feedback entry's `aux_bits` field into
/// register `$dst`. The low 32 bits of `aux_bits` carry the target
/// `ShapeId` raw u32 (high 32 bits reserved/zero). Used by the
/// `OwnDataInlineWrite` asm fast path to update the object's shape
/// pointer after the inline-slot store.
#[macro_export]
macro_rules! load_named_target_shape {
    ($entry:tt => $dst:tt) => {
        concat!(
            "ldr    w",
            stringify!($dst),
            ", [x",
            stringify!($entry),
            ", {feedback_named_aux_bits}]\n",
        )
    };
}
```

### Step 7.3: Add `store_record_shape!`

In `crates/vm/src/dsl/backend/aarch64/operands.rs` (or wherever the `load_record_shape!` macro lives — find with `rg`):

```rust
/// Store the 32-bit target shape value in `$src` into the object
/// record's shape field. Used by the `OwnDataInlineWrite` asm fast
/// path to update the object's shape pointer after the inline-slot
/// store, in lockstep with the slow path's `transition_shape` followed
/// by retarget_shape sequence (for transitioning writes) or a no-op
/// store-of-same-value (for non-transitioning writes).
///
/// The offset to the shape field is supplied via the
/// `{record_shape_offset}` bound interpolation — same binding as
/// `load_record_shape!` uses.
#[macro_export]
macro_rules! store_record_shape {
    ($record:tt, $src:tt) => {
        concat!(
            "str    w",
            stringify!($src),
            ", [x",
            stringify!($record),
            ", {record_shape_offset}]\n",
        )
    };
}
```

(Confirm the binding name by reading `load_record_shape!`'s definition. If it uses a different name like `{object_record_shape_offset}`, mirror it exactly.)

### Step 7.4: Snapshot test the macros emit correct asm strings

Add to the existing macro tests (likely in `crates/vm/src/dsl/backend/aarch64/feedback.rs` or `operands.rs`):

```rust
#[cfg(test)]
mod inline_write_macro_tests {
    #[test]
    fn branch_named_own_inline_write_mode_emits_mode_5_check() {
        let asm = branch_named_own_inline_write_mode!(9, .miss);
        assert!(asm.contains("ldrb   w16, [x9, {feedback_mode}]"));
        assert!(asm.contains("cmp    w16, #5"));
        assert!(asm.contains("b.ne   .miss"));
    }

    #[test]
    fn load_named_target_shape_emits_aux_bits_load() {
        let asm = load_named_target_shape!(9 => 10);
        assert!(asm.contains("ldr    w10, [x9, {feedback_named_aux_bits}]"));
    }

    #[test]
    fn store_record_shape_emits_str_at_shape_offset() {
        let asm = store_record_shape!(11, 12);
        assert!(asm.contains("str    w12, [x11, {record_shape_offset}]"));
    }
}
```

- [ ] **Step 7.5: Run, verify PASS** (these are unit tests on macro string output)

```bash
cargo test -p lyng-vm inline_write_macro_tests 2>&1 | tail -10
```

- [ ] **Step 7.6: Verify the workspace still builds**

```bash
cargo check --workspace 2>&1 | tail -5
```

- [ ] **Step 7.7: Commit**

```bash
git add crates/vm/src/dsl/backend/aarch64/feedback.rs crates/vm/src/dsl/backend/aarch64/operands.rs
git commit -m "vm/dsl: add asm macros for OwnDataInlineWrite fast path

- branch_named_own_inline_write_mode!: 3-insn mode-byte check (mode=5)
- load_named_target_shape!: load post-write shape from aux_bits low 32
- store_record_shape!: write target shape into object record's shape field

Macros are passive — consumed by op_assign_named_property_dsl in Task 8.
Snapshot tests pin the emitted asm strings."
```

---

## Task 8: Asm monomorphic-write hit path in `op_assign_named_property_dsl`

**Goal:** Wire the asm hit path. When mode = 5, the dispatch checks source shape, value primitiveness, writable bit, then does inline-slot store + shape-pointer update + dispatch. Cold sites pay 3 instructions before bailing to the existing Rust probe.

**Files:**
- Modify: `crates/vm/src/dsl/handlers/cold.rs` — replace `op_assign_named_property_dsl` body (around line 2784)

### Step 8.1: Write failing test for end-to-end asm hit behavior

In `crates/vm/src/tests/inline_caches.rs`, add a counter-based assertion (uses the `diagnostic-counters` feature):

```rust
#[cfg(feature = "diagnostic-counters")]
#[test]
fn assign_named_property_asm_fast_path_hits_after_warmup() {
    let mut agent = make_test_agent();
    let mut vm = Vm::new();
    let installed = compile_and_install(&mut agent, &mut vm, r#"
        function f(o) { o.x = 1; }
        for (let i = 0; i < 100; i++) f({});
    "#);
    let _ = installed.run(&mut agent, &mut vm).expect("script runs");
    let counters = vm.ic_slow_path_counters();
    let total = counters.total(IcSlowPathKind::AssignNamedProperty);
    let probe_dispatches = counters.assign_probe_dispatches();
    // After warmup, the asm fast path should intercept the bulk of writes.
    // Probe dispatches should be MUCH smaller than the loop iteration count.
    assert!(
        probe_dispatches < 20,
        "asm should intercept hot loop writes; probe_dispatches = {}",
        probe_dispatches,
    );
}
```

- [ ] **Step 8.2: Run, verify FAIL** (asm path not added yet; probe_dispatches ≈ 100)

```bash
cargo test -p lyng-vm --features diagnostic-counters \
    assign_named_property_asm_fast_path_hits_after_warmup 2>&1 | tail -10
```

- [ ] **Step 8.3: Replace `op_assign_named_property_dsl` body**

In `crates/vm/src/dsl/handlers/cold.rs:2784`, replace the existing `llint_handler! { op_assign_named_property_dsl, ... }` block with:

```rust
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_assign_named_property_dsl, opcode_byte = 79, layout = AbcSlot, length = 6, |a, b, c, slot| {
        // Monomorphic `OwnDataInlineWrite` fast path. Cheap mode-byte
        // check bails 3 instructions in; the hit path performs the
        // primitive-value inline-slot store + atomic shape pointer
        // update, then dispatches without touching the Rust probe.
        //
        // Bail conditions (all go to `.probe`, which re-decodes operands):
        // - mode != 5 (uncached / read-side cached / polymorphic /
        //   megamorphic / outline write)
        // - receiver is not an object
        // - cached entry is read-only (writable bit unset)
        // - receiver shape doesn't match cached source shape
        // - value is a heap reference (GC barrier required)
        //
        // Operand binding (layout = AbcSlot):
        //   a    = receiver register
        //   b    = value register
        //   c    = atom-constant index (unused on the inline path)
        //   slot = feedback slot
        //
        // Cheap mode-byte check first — operand registers untouched
        // so `.probe` reuses them without re-decode.
        load_feedback_site!(slot => t0);
        branch_named_own_inline_write_mode!(t0, .probe);

        // Hit-path validation. Commits to the chain — subsequent bails
        // land at `.probe_dirty` which re-decodes before calling the
        // Rust probe.
        load_reg!(a => a);
        check_object_ref!(a, .probe_dirty);
        untag_object_ref!(a);
        load_named_handler_bits!(t0 => slot);
        load_named_target_shape!(t0 => t1);

        // Validate inline + writable bits, extract 30-bit slot index
        // into `c`. Read-only entries miss to the Rust probe so the
        // strict-mode TypeError contract stays in the slow path.
        load_named_inline_writable_slot_index_or_branch!(slot => c, .probe_dirty);

        // Shape guard.
        load_object_record_from_state_or_branch!(a => a, .probe_dirty);
        load_record_shape!(a => t2);
        load_named_handler_shape!(slot => t3);
        cmp_branch_ne!(t2, t3, .probe_dirty);

        // GC barrier safety: only primitive writes can skip the card
        // mark + incremental value barrier (`write_object_inline_named_slot`'s
        // early returns make the barrier helpers no-ops for primitives).
        // Heap-referencing kinds bail to the Rust probe which routes
        // through `mut_store_value`.
        load_reg!(b => b);
        branch_value_references_heap!(b, .probe_dirty);

        // Commit: inline-slot store, then update the object's shape
        // pointer to target_shape (no-op for non-transitioning writes
        // since source_shape == target_shape in that case).
        store_record_inline_slot!(a, c, b);
        store_record_shape!(a, t1);
        dispatch!();

        .probe_dirty:
        // Hit-path validation clobbered operand registers — re-decode
        // before calling the probe.
        decode_abc_slot!(a, b, c, slot);
        .probe:
        call_rust_probe!(op_assign_named_property_rust_probe_rs, args = [a, b, c, slot]);
        branch_nonzero!(0, .slow);
        dispatch_probe_hit_no_refresh!();
        .slow:
        // Probe can clobber caller-saved registers.
        decode_abc_slot!(a, b, c, slot);
        call_slow!(op_assign_named_property_slow_rs, args = [a, b, c, slot]);
        dispatch_after_slow!();
    }
}
```

Apply the same body to `op_strict_assign_named_property_dsl` (the strict-mode opcode at byte 80 — find via rg).

Update the imports at the top of `cold.rs` to add the new macros: `branch_named_own_inline_write_mode`, `load_named_target_shape`, `store_record_shape`. Also `branch_value_references_heap`, `load_named_inline_writable_slot_index_or_branch`, `store_record_inline_slot` if not already imported (the eager/lean attempts had these; the current cold.rs may have lost them in cleanup).

- [ ] **Step 8.4: Run, verify PASS**

```bash
cargo test -p lyng-vm --features diagnostic-counters \
    assign_named_property_asm_fast_path_hits_after_warmup 2>&1 | tail -5
```

- [ ] **Step 8.5: Run full workspace tests (default features)**

```bash
cargo test --workspace --no-fail-fast 2>&1 | tail -3
```

Expected: all existing tests still pass.

- [ ] **Step 8.6: Run V8 benchmark suite (will validate the perf claim in Task 9)**

```bash
cargo run --release -p lyng-bench -- v8suite 2>&1 | tail -10
```

Record the scores. Expect RayTrace to be in the 270-310 range (up from 232). Crypto/NavierStokes/Splay should be within ±2% of baseline.

- [ ] **Step 8.7: Commit**

```bash
git add crates/vm/src/dsl/handlers/cold.rs crates/vm/src/tests/inline_caches.rs
git commit -m "vm/dsl: add asm monomorphic OwnDataInlineWrite hit path

op_assign_named_property and op_strict_assign_named_property both get
the new fast path: cheap 3-insn mode-byte check, then on mode=5 do
shape guard + value-kind check + inline-slot store + shape pointer
update + dispatch. Cold sites pay 3 insns before bailing to the
existing Rust probe — no regression on non-write-heavy benchmarks.

Diagnostic-counter test pins the hit rate after warmup."
```

---

## Task 9: V8 benchmark validation + report refresh

**Goal:** Confirm the perf claims with a 3-run thermally-stable bench. Refresh `reports/lyng/bench-v8.md`.

**Files:**
- Modify: `reports/lyng/bench-v8.md` (refresh medians, add a "Transition-IC migration" section)
- Modify: `reports/lyng/bench-v8.json` (auto-generated by the bench tool)

### Step 9.1: Run 3-sample bench on stable hardware

```bash
# Cooldown gap between runs is important for thermal stability.
cargo run --release -p lyng-bench -- v8suite 2>&1 | tail -10
sleep 60
cargo run --release -p lyng-bench -- v8suite 2>&1 | tail -10
sleep 60
cargo run --release -p lyng-bench -- v8suite 2>&1 | tail -10
```

Record all three runs. Take the per-benchmark median.

### Step 9.2: Verify against the acceptance bar

| Benchmark | Baseline | Target | Achieved (fill in) | Δ |
|---|---:|---:|---:|---:|
| RayTrace | 232 | 290+ | ? | ? |
| DeltaBlue | 389 | 420+ | ? | ? |
| Richards | 470 | 480+ | ? | ? |
| Crypto | 441 | 441 (no regression) | ? | ? |
| NavierStokes | 602 | 602 (no regression) | ? | ? |
| Splay | 1465 | 1465 (no regression) | ? | ? |

If any acceptance bar fails:
- **RayTrace below 290:** check the diagnostic counters. If `AssignNamedProperty` slow entries are still in the millions with `shape_mismatch` cause, the asm path isn't firing — diagnose with `cargo run --release --features diagnostic-counters -p lyng-cli -- --shell --dump-ic-counters /tmp/raytrace-harness.js` (build the harness as in the previous session: `cat testdata/js-benchmarks/v8-v7/base.js testdata/js-benchmarks/v8-v7/raytrace.js > /tmp/raytrace-harness.js && echo 'BenchmarkSuite.RunSuites({NotifyScore: function(score) { print("SCORE\t" + score); },});' >> /tmp/raytrace-harness.js`).
- **Crypto/NavierStokes/Splay regression >2%:** the 3-insn cheap bail is somehow not as cheap as expected. Check if the asm grew unexpectedly (`objdump` the binary or look at the captured asm in `reports/lyng/dsl-asm-baseline-aarch64/AssignNamedProperty.asm`).

### Step 9.3: Refresh `reports/lyng/bench-v8.md`

Update the score table with the new medians. Add a new section:

```markdown
## Transition IC (Spec 3) bench journey

Post-Phase-D RayTrace sat at 232 (-25% vs pre-Spec-2's 291) because the
asm dispatch layer didn't recognize transitioning writes — RayTrace's
Object.extend pattern shape-thrashed the cache with a 98.3% slow-path
miss rate. Spec 3 added the `OwnDataInlineWrite` IC kind: mode = 5,
asm-recognized monomorphic transition cache, watchpoint-invalidated.

| Stage | Richards | DeltaBlue | Crypto | RayTrace | NavierStokes | Splay |
|---|---:|---:|---:|---:|---:|---:|
| Pre-Spec-2 (`857d2528`) | 484 | 421 | 393 | 291 | 541 | 1440 |
| Phase D end (`b8b3c83f`) | 470 | 389 | 441 | 232 | 602 | 1465 |
| Spec 3 (this commit) | <X> | <Y> | <Z> | <RT> | <NS> | <SP> |
```

Fill in `<X>`, `<Y>`, `<Z>`, `<RT>`, `<NS>`, `<SP>` with the 3-run medians from Step 9.1.

- [ ] **Step 9.4: Verify against the acceptance bar (final pass)**

Re-read the table. If everything passes, proceed to commit. If anything regresses, file a follow-up issue in `dcat` (or back up and diagnose).

- [ ] **Step 9.5: Commit**

```bash
git add reports/lyng/bench-v8.md reports/lyng/bench-v8.json
git commit -m "reports/bench-v8: refresh post-transition-IC scores

3-run median on a thermally-stabilized system. RayTrace recovers
to <RT> (was 232, target 290+, pre-Spec-2 baseline 291). DeltaBlue
+<X%>, Richards +<Y%>. Crypto / NavierStokes / Splay within
±1.5% noise floor.

Transition IC bench journey table added documenting the recovery."
```

---

## Verification (end-to-end)

After Task 9:

1. **All tests:** `cargo test --workspace --no-fail-fast` — must show ≥2670 passed (existing 2662 + ~8 new from tasks 1-9), 19 ignored.
2. **Test262 conformance:** `cargo run --release -p lyng-test262` — count of passing tests must not decrease vs the pre-implementation baseline.
3. **V8 bench:** RayTrace ≥ 290, DeltaBlue ≥ 420, Richards ≥ 480, Crypto/NavierStokes/Splay within ±1.5% of baseline (per Task 9 acceptance bar).
4. **Diagnostic counters:** with `--features diagnostic-counters`, RayTrace's `AssignNamedProperty` slow-entry total drops from 7.4M to <500k. Remaining slow entries should be classified as `polymorphic` or `megamorphic`, NOT `shape_mismatch`.
5. **No new clippy warnings:** `cargo clippy --workspace --all-targets -- -D warnings`.
6. **cargo fmt clean:** `cargo fmt --check`.

## Out of scope (deferred follow-ups, separate plans)

- **Polymorphic-write asm fast path** (mode = 6). Once profile data shows polymorphic-transition workloads matter, add a 2-entry inline asm walk with `PropertyMetadata` layout widening (`aux_bits_2`, `aux_bits_3`).
- **Outline-slot writes** (mode = 7). For objects exceeding the inline-slot budget. Needs a separate handler structure carrying an out-of-line slot index instead of an inline index.
- **Per-property-atom watchpoint granularity.** Currently `AdaptiveOwnWrite` clears the cache on any shape invalidation event for source_shape — including unrelated property additions. A more granular fire payload (carrying the transitioning atom) would let `AdaptiveOwnWrite` selectively clear only matching entries.
- **Bytecode-level specialization** (`DefineOwnProperty` opcode for object-literal initializers). The current path goes through `op_assign_named_property` for everything, including known-new properties. A specialized opcode could skip the cache state-machine overhead entirely on cold first-touch.
