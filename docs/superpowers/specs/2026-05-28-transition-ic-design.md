# Transition-Aware Write IC Design

**Spec date:** 2026-05-28
**Branch:** `feedback_refactor` (continuation of the Spec 2 IC→JSC migration)
**Status:** approved by user; awaiting plan-writing

## Goal

Recover RayTrace's V8 benchmark score (currently 232, ~25% below pre-Spec-2's 291) by making the IC layer recognize and cache **shape-transitioning writes** — the pattern produced by Prototype.js-style `Object.extend(obj, props)` initializers. The design generalizes: every benchmark with constructor-style write patterns (RayTrace, DeltaBlue, Richards) should benefit, with no regression on transition-light workloads (Crypto, NavierStokes, Splay).

## Diagnosis (from the diagnostic-counters infrastructure)

The Spec 2 IC→JSC migration correctly moved the engine from epoch-based invalidation to watchpoint-based invalidation. But the IC *dispatch* layer hasn't adapted: the asm fast paths still only recognize **non-transition** writes (`OwnDataInlineLoad`-style entries where the receiver's shape doesn't change). On RayTrace, this is catastrophic:

- 7.4M `AssignNamedProperty` slow entries per benchmark run
- 98.3% of those are classified `shape_mismatch` — the IC is in Monomorphic state, but the receiver's pre-write shape doesn't match the cached source_shape, because each iteration's `Object.extend` allocates a fresh object whose shape transitions on every property assignment.

Two prior fast-path attempts (subagent 1's eager validation, subagent 2/3's lean reorder) confirmed empirically: tuning the asm dispatch chain doesn't help. **The cache miss is structural** — the cached entry caches the *post*-transition shape, but the receiver always arrives with the *pre*-transition shape. The fix is to cache the transition arrow itself: `(source_shape, target_shape, slot_location)`.

## Architecture summary

Introduce a unified IC kind: `OwnDataInlineWrite`. Each entry holds `(source_shape, target_shape, slot_location, writable_flag)`. The asm fast path's hit semantics: compare receiver_shape to source_shape, store the value to the inline slot, write target_shape into the object's shape pointer (a no-op when source_shape == target_shape, i.e. for non-transitioning writes).

The new IC kind plugs into the existing IC state machine (Uninit → Mono → Poly → Mega), the existing watchpoint subsystem (one new `AdaptiveOwnWrite` observer variant), the existing slow-path observation pipeline (consumes the existing `OwnDataTransition` plan path), and the existing polymorphic chain infrastructure (up to 2 entries inline, deeper chains via the Rust probe). No state-machine rebuild, no bytecode-level changes, no compiler changes. **Additive across the board.**

---

## 1. The cache entry abstraction

A single unified entry kind handles both transitioning and non-transitioning inline-slot writes:

| Pattern | source_shape | target_shape | Asm action |
|---|---|---|---|
| Non-transition write (`o.x = v` where `o` already has `x`) | shape S | shape S (same) | inline store; shape pointer write is a no-op |
| Transition write (`o.x = v` where `o` doesn't have `x` yet) | shape S_in | shape S_out | inline store; shape pointer update to S_out |

The asm fast path doesn't branch on the transition vs non-transition case. It always:
1. Compares `receiver_shape == source_shape`, bails on mismatch
2. Stores the value at `slot_location`
3. Stores `target_shape` into the object's shape header (one extra MOV when source == target, ~1 cycle on OoO ARM64)

The atom name is implicit per IC site (the `c` operand of `op_assign_named_property` is a constant atom-index from the bytecode constant pool). Each IC site only ever sees one atom — no need to cache it.

**Out of scope for MVP:** out-of-line slot writes. The structural recovery comes from inline-slot transitions (the first ~7 properties on a fresh object). Outline-slot transitioning writes stay on the Rust probe in this design; an outline-slot fast path (mode = 7 / 8) is a clean follow-up if profile data shows it matters.

## 2. IC state machine integration

The new write IC kind lives inside the existing `PropertyIcState` struct (in `crates/vm/src/vm/ic_state/property.rs`) as a new sidecar field: `monomorphic_own_inline_write_handler: NamedPropertyInlineWriteHandler`. Mirrors how `monomorphic_own_data_handler` works for reads.

Per-slot state progression mirrors the read side (Uninit → Mono → Poly → Mega):

| State | Behavior on slow-path entry |
|---|---|
| Uninitialized | Observe + install first `OwnDataInlineWrite` entry → transition to Monomorphic |
| Monomorphic | Compare incoming source_shape to cached entry. Match → execution count bump, handler unchanged. Mismatch → upgrade to Polymorphic |
| Polymorphic | Append entry (up to POLY_LIMIT). Already-cached source → exec count bump. Overflow → upgrade to Megamorphic |
| Megamorphic | Bail to Rust probe permanently. PropertyMetadata mode byte = 0 |

Polymorphic chains reuse the existing `polymorphic_own_data_handlers: [_; POLY_LIMIT]` shape, specialized for the new handler type.

**Three additive surface changes:**

1. `PropertyIcState` gets a new sidecar field `monomorphic_own_inline_write_handler` plus the polymorphic-chain analog
2. `PropertyMetadata` projection writes a new mode byte: 5 (mono inline write) or 6 (poly inline write)
3. Slow-path observation routes `OwnData` / `OwnDataTransition` plans into the new IC kind

**MVP scope:** inline-slot writes only (mode 5 and 6). Out-of-line writes stay on the Rust probe.

## 3. Asm fast path

The new asm path lives in `op_assign_named_property_dsl` (and `op_strict_assign_named_property_dsl`) in `crates/vm/src/dsl/handlers/cold.rs`. Dispatch chain:

```
load_feedback_site → entry                  (3 insn)
branch_named_own_inline_write_mode          (3 insn; bail to .try_poly_write on mode != 5)
  [monomorphic-write hit path]
.try_poly_write:
branch_named_own_inline_write_poly_mode     (3 insn; bail to .probe on mode != 6)
  [polymorphic-write hit path]
.probe:
call_rust_probe ...                         (existing fallback, unchanged)
```

**Per-miss budget:** 6 instructions (3 + 3) before bailing to the Rust probe for cold IC sites — feedback slot stays Uninitialized → mode = 0 → bail immediately. Crypto / NavierStokes / Splay pay this and nothing else.

**Monomorphic-write hit path** (~25-28 instructions, paid only on a true match):
1. Load handler_bits → extract source_shape, slot, writable_flag
2. Load aux_bits → extract target_shape
3. Load receiver from register, check object_ref, untag
4. Load object record + load record_shape
5. Compare record_shape == source_shape → bail on mismatch
6. Load value from register
7. `branch_value_references_heap` → bail on heap-ref values (GC barrier safety)
8. Inline-slot store via `store_record_inline_slot!`
9. New macro `store_record_shape!` writes target_shape into object's shape field
10. Dispatch

**Polymorphic-write hit path** (~30-35 instructions per entry, up to 2 inline entries). Mirrors `branch_named_own_polymorphic_mode`. Three-entry+ chains stay on the Rust probe.

**Critical correctness invariants:**
- Heap-referencing values bail to Rust probe — no inline GC barriers in asm
- Shape pointer write happens AFTER slot store (asm sequence enforces this ordering — important for watchpoint / GC consistency)
- The cached arrow `source → target` was created by an earlier slow-path call to `transition_shape` (which fired watchpoints at that time); asm reuses it without firing again

**New asm primitives needed:**
- `branch_named_own_inline_write_mode!` (mode = 5 check)
- `branch_named_own_inline_write_poly_mode!` (mode = 6 check)
- `store_record_shape!` (writes target_shape to object's shape field)
- `load_named_target_shape!` (extracts target_shape from aux_bits)
- Reuse: `branch_value_references_heap!`, `load_named_inline_writable_slot_index_or_branch!`, `store_record_inline_slot!`, `load_named_handler_bits!`, `load_named_handler_shape!`

## 4. Slow-path observation and install

The plan layer already produces both `OwnData` (non-transition) and `OwnDataTransition` (transitioning) cache plans via `plan_named_property_cache_entry`. The new design consumes these existing plans — no plan-layer rewrite needed.

**State-machine install flow** (in `crates/vm/src/vm/feedback.rs`):

1. `record_named_property_cache_entry(plan)` — existing entry point; gates on watchpoint registration; calls `named_property_install_slow_path(plan)`
2. New branch in `named_property_observe_slow_path_on_state`: when the slot is a write slot (detected via opcode kind) AND the plan is `OwnData` or `OwnDataTransition`, install the entry into the new `monomorphic_own_inline_write_handler` sidecar (or polymorphic chain)
3. State transition follows existing Uninit → Mono → Poly → Mega rules

**Projection** (after each install — extends `project_property_into_meta`):

The function reads the current IcState and writes asm-readable bits to `PropertyMetadata`:
- Monomorphic write: mode = 5, handler_bits packed (source_shape + slot + flags), aux_bits = target_shape
- Polymorphic write: mode = 6, handler_bits = entry 0, aux_bits = entry 1

**`PropertyMetadata` layout widening:** the existing 64-bit handler_bits + aux_bits pair holds source + slot for one entry, but polymorphic-write needs TWO target_shapes packed alongside. The struct currently has unused bytes (Property metadata is 32 bytes; less than half is consumed). Add two more u32 fields: `aux_bits_2` (entry 0's target_shape) and `aux_bits_3` (entry 1's target_shape). This keeps polymorphic-write symmetrical with polymorphic-read.

**Plan layer responsibility:** `plan_named_property_cache_entry` for `purpose = Store` may need extension to ALWAYS produce `OwnDataTransition` when the write would transition shape. If a transition-producing write currently returns `None` (uncacheable), the slow path can't install — needs verification during implementation.

**Generation / clearance semantics:** unchanged. Watchpoint fires call `clear_ic_slot_if_generation_matches` which clears the IcState entry and zeros `PropertyMetadata`. The new IC kind inherits this for free.

## 5. Watchpoint coupling

The new IC kind plugs into the existing `WatchpointSet` infrastructure (Spec 1) and reuses `clear_ic_slot_if_generation_matches` (Spec 2 Phase A).

**On install:** when the slow path installs an `OwnDataInlineWrite` entry with `source_shape = S`, it registers a new `AdaptiveOwnWrite` watchpoint on `S` (a new `ShapeInvalidationObserver` variant in `crates/objects/src/watchpoint.rs`) with payload `(code, slot, generation)`. The fire callback is identical to `AdaptiveProtoLoad`: `clear_ic_slot_if_generation_matches(code, slot, generation)`.

**On asm hit:** the fast path does NOT fire watchpoints. It uses a pre-cached transition arrow that's intrinsic to the shape system and stable for the lifetime of source_shape. No new shape transition is created — no consumer needs notification.

**Invalidation events that fire the watchpoint on S** (per Spec 1's existing design):

| Event | Fires watchpoints on | Effect on our cache |
|---|---|---|
| Dictionary transition (`ensure_named_property_dictionary(o)` where `o` has shape S) | S | Clears cache (correct — dictionary mode needs different store path) |
| Property addition transition (`transition_shape(S, atom_Y)` for any atom Y) | S | Clears cache (overly conservative; entry for atom X is still valid) |
| Prototype mutation (`setPrototypeOf` on object with shape S) | S | Clears cache (overly conservative; own-data transitions don't depend on proto) |

**MVP accepts the conservative invalidation.** Property additions and proto mutations are rare relative to property writes. Cache re-installs within a few iterations after each unrelated invalidation. Per-event granularity (extending watchpoint fire payload to carry the transitioning atom, with selective clearance) is a deferrable follow-up.

**Asm safety claim:** the asm fast path writes `target_shape` to the object's shape header without firing watchpoints. This is safe because the arrow `source → target` was already created by an earlier slow-path call to `transition_shape` (which DID fire watchpoints at that time). No new transition is being introduced.

## 6. GC and shape liveness

The new IC kind references two shape IDs per entry. Lifetime story leans entirely on existing Lyng invariants — no new pinning, no new sweep paths.

**Shape liveness today:** `ObjectRuntime::shape_metadata: Vec<Option<ShapeMetadata>>` is a slab that grows but doesn't shrink. Shapes are pinned for the VM's lifetime. The existing read-side IC stores raw `ShapeId`s without explicit pinning.

**Cache entry lifetime anchors:**

| Anchor | Existing sweep path | Effect on new write IC |
|---|---|---|
| Code object reachability | `prune_dead_code_property_ic_states` (Spec 2 Phase D.4) | Whole IC slab for dead code goes away |
| Watchpoint generation | `clear_ic_slot_if_generation_matches` (Spec 2 Phase A.2) | Already used by `AdaptiveProtoLoad`; new `AdaptiveOwnWrite` reuses it |

**Target shape reachability is transitive:** a cache entry's source_shape (S_in) has a transitions table containing `atom → S_out`. As long as S_in is alive (it is — shapes don't collect), S_out is alive via that table.

**Memory bound:** one cache entry per `(code, slot)` pair, plus up to POLY_LIMIT polymorphic entries. Each entry ~32 bytes. Bounded by program code size; identical footprint to the existing read-side IC.

**Documented assumption:** if shape collection is ever introduced, the IC layer would need to participate (pin source/target shapes or sweep entries pointing to collected shapes). Add a comment in the `NamedPropertyInlineWriteHandler` struct flagging this.

## 7. Polymorphic fallback and megamorphic behavior

The new IC kind reuses the existing polymorphic chain (`polymorphic_chains` slab on `Vm`). The asm-cacheable subset mirrors the read-side: 2 entries inline; deeper chains via Rust probe.

| Cache state | # source shapes seen | Asm path | Storage |
|---|---|---|---|
| Monomorphic | 1 | mode = 5, full asm hit | Inline in `PropertyMetadata.handler_bits` + `aux_bits` |
| Polymorphic (asm-cacheable) | 2 | mode = 6, walks 2 inline entries | Both inline in handler_bits + aux_bits + aux_bits_2 + aux_bits_3 |
| Polymorphic (chain-extended) | 3 – POLY_LIMIT (4) | mode = 6, asm tries 2 inline then bails to Rust-walked chain | Inline 2 + chain in `polymorphic_chains` slab |
| Megamorphic | > POLY_LIMIT (8+ typical threshold) | mode = 0, asm bypasses cache | Cache state permanent megamorphic; entry data dropped |

**Megamorphic behavior:** permanent uncached state with 6-instruction cheap bail to Rust probe. Same dispatch cost as today's uncached state — no regression for megamorphic sites.

**Where megamorphic shows up in practice:** generic property accessors or polyglot reflection across many unrelated shape lineages. Constructor patterns (RayTrace's Vector, DeltaBlue's Constraint) stay monomorphic per call site. Hot-loop transition writes essentially never hit megamorphic.

**Trade-off:** going above 2 inline asm entries inflates the dispatcher with shape-compare + slot-extract + write per entry — ~25-30 instructions per entry. This starts hitting branch-predictor and icache costs without proportional benefit. The read-side made the same call. Revisiting POLY_LIMIT is an orthogonal axis.

## 8. Testing strategy

Three layers — state-machine correctness, asm correctness, end-to-end perf — each pinning a different invariant.

### Layer 1: IC state machine unit tests

In `crates/vm/src/vm/feedback.rs` test module:
- Install `OwnDataInlineWrite` entry from a planned transition, verify state moves Uninit → Monomorphic, sidecar populated, mode = 5 in `PropertyMetadata`
- Second entry with different source_shape, verify Mono → Polymorphic, mode = 6, both shapes packed correctly
- POLY_LIMIT+1 entries, verify Poly → Megamorphic, mode = 0
- Register `AdaptiveOwnWrite` watchpoint, fire shape invalidation, verify `clear_ic_slot_if_generation_matches` clears the entry and zeros `PropertyMetadata`

### Layer 2: Asm correctness tests

In `crates/vm/src/tests/inline_caches.rs`:
- Hot-loop transition write (`for (i...) { o = {}; o.x = i; }`) — verify after N iterations `o.x === N-1`, IC reached Monomorphic-write
- Polymorphic write (alternating constructor functions producing different starting shapes) — verify IC reaches Polymorphic-write with 2 source shapes, asm hits both
- Heap-ref value bail (`o.x = otherObject`) — verify Rust probe handles it, IC state unchanged, value correct
- Shape mismatch bail — verify Rust probe handles it, IC stays Monomorphic
- Read-only property assign — verify Rust probe handles it (TypeError or silent ignore per strict mode), IC unchanged
- All existing IC tests continue passing

### Layer 3: V8 bench validation with diagnostic counters

3-run medians on a thermally stable system:

| Benchmark | Baseline | Target | Acceptance bar |
|---|---:|---:|---|
| RayTrace | 232 | 290+ | Must recover pre-Spec-2 (291) ±2% |
| DeltaBlue | 389 | 420+ | Material gain (≥8%) |
| Richards | 470 | 480+ | Modest gain (≥2%) |
| Crypto | 441 | 441 | No regression (within ±1.5% noise) |
| NavierStokes | 602 | 602 | No regression (within ±1.5% noise) |
| Splay | 1465 | 1465 | No regression (within ±1.5% noise) |

**Counter-based correctness validation** (using the `diagnostic-counters` cargo feature):
- RayTrace `AssignNamedProperty` slow-entry total should drop from 7.4M to <500k; remaining slow entries should be `polymorphic` / `megamorphic`, not `shape_mismatch`
- DeltaBlue: similar shape_mismatch reduction
- Crypto / NavierStokes / Splay: slow-entry counts unchanged

If counter data shows the asm path firing as expected AND the bench scores hit the targets, the design works. If asm fires but scores don't move, there's a hidden cost (icache, branch predictor) to investigate. If asm doesn't fire, the install/projection is buggy.

**Workspace regression bar:** `cargo test --workspace` keeps passing (currently 2662 tests, 19 ignored). Test262 conformance must not regress.

---

## Scope summary

**In MVP:**
- `OwnDataInlineWrite` IC kind (mode = 5 / 6) for inline-slot writes
- Asm fast paths for monomorphic-write and 2-entry polymorphic-write
- `AdaptiveOwnWrite` watchpoint variant
- Slow-path observation routing for both `OwnData` and `OwnDataTransition` plans
- `PropertyMetadata` layout widening to fit polymorphic-write target_shapes
- New asm macros: `branch_named_own_inline_write_mode!`, `branch_named_own_inline_write_poly_mode!`, `store_record_shape!`, `load_named_target_shape!`

**Deferred / out of scope:**
- Out-of-line slot writes (mode = 7 / 8)
- Per-property-atom granularity on watchpoint fire (currently fires on any S event)
- Polymorphic asm walking beyond 2 entries (3+ stays on Rust probe)
- Bytecode-level specialization (separate `DefineOwnProperty` opcode for object literals)
- Compiler-side transition prediction

## Files touched (anticipated)

| File | Purpose |
|---|---|
| `crates/vm/src/vm/ic_state/property.rs` | Add `monomorphic_own_inline_write_handler` + polymorphic chain sidecar |
| `crates/vm/src/vm/feedback.rs` | Install path, state machine extension, projection function, slow-path routing |
| `crates/vm/src/vm/metadata_table/mod.rs` | `PropertyMetadata` layout — add `aux_bits_2`, `aux_bits_3` |
| `crates/vm/src/dsl/handlers/cold.rs` | `op_assign_named_property_dsl` body with mode 5 / 6 hit paths |
| `crates/vm/src/dsl/backend/aarch64/feedback.rs` | New asm macros: `branch_named_own_inline_write_mode!`, `branch_named_own_inline_write_poly_mode!`, `load_named_target_shape!` |
| `crates/vm/src/dsl/backend/aarch64/operands.rs` | New `store_record_shape!` macro (or co-located) |
| `crates/objects/src/watchpoint.rs` | Add `AdaptiveOwnWrite` variant to `ShapeInvalidationObserver` |
| `crates/env/src/agent.rs` | Dispatch `AdaptiveOwnWrite` fire callback (mirroring `AdaptiveProtoLoad` handling) |
| `crates/objects/src/` (new handler type) | `NamedPropertyInlineWriteHandler` packed-handler struct (mirrors existing `NamedPropertyHandler`) |
| `crates/vm/src/tests/inline_caches.rs` | New asm correctness tests |
| `reports/lyng/bench-v8.md` | Refresh post-implementation |

## Predecessor work this design builds on

- **Spec 1 (2026-05-25)** — shape transitions for `setPrototypeOf`, `WatchpointSet` primitive: provides the watchpoint infrastructure this design plugs into
- **Spec 2 Phase A** — `AdaptiveProtoLoad` observer, watchpoint-based IC invalidation: provides the `clear_ic_slot_if_generation_matches` callback pattern this design reuses
- **Spec 2 Phases B-E** — MetadataTable layout, per-kind IC state machines, Vec-indexed side-tables: provides the asm-readable metadata projection and per-slot state infrastructure
- **Diagnostic counters (commit `b8b3c83f`)** — provides empirical validation of hit/miss rates per IC kind, enabling Layer 3 correctness validation
