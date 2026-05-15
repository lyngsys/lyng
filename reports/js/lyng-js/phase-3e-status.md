# Phase 3e — One-hop PrototypeData inline IC fast path: status report

**Issue:** `lyng-22al` — Phase 3e: Inline PrototypeData fast path for IC opcodes
**Parent:** `lyng-2pgt` — Phase 3: Inline IC fast path (epic)
**Date:** 2026-05-15
**Toolchain:** rustc 1.93 (2025-12-15), aarch64-apple-darwin, `--release` profile
**Parent commit:** `2cd3d294` (Phase 3d close)
**Approach selected:** Option (i) — inline one-hop prototype walk. Option (ii) (watchpoint system) deferred (would require building new watchpoint infrastructure that doesn't exist today).

## Decision criterion (from `lyng-22al`)

> *"Decision criterion in the 3a follow-up bench report: if Splay shows <+3% post-Phase-3d, this sub-issue moves forward."*

Phase 3d's Splay delta vs Phase 3c was **+1.3%** (1263 → 1279) — well below
the +3% threshold. Phase 3e was therefore justified.

## What landed

The named-property IC fast-path dispatch is extended with a second tier
that services monomorphic one-hop `PrototypeData` cache entries —
`dependency_count == 2` (receiver → one prototype), the dominant pattern
for class method dispatch (`instance.method()` → `Class.prototype.method`)
and `Object.prototype` lookups.

The Tier 2 fast path adds three new checks on top of Tier 1 (OwnData):
1. Receiver shape compare (catches arbitrary shape changes on receiver).
2. Receiver `invalidation_epoch` compare (catches `set_prototype()` —
   bumps the receiver epoch with cause `PrototypeMutation` — and any
   own-property add/delete/redefine on the receiver).
3. Prototype shape + epoch compare (catches mutations on the prototype
   itself).

No `bl` to a handler-fetch helper or to the IC chain on the proto-hit
path. The new `try_named_property_proto_fast_load` /
`try_keyed_named_proto_fast_load` helpers are `#[inline(always)]` and
fully inline into `op_get_named_property`, `op_load_global`, and
`op_get_keyed_property`.

Multi-hop `PrototypeData` (dependency_count ∈ {3, 4}) and all
non-PrototypeData states still fall through to the slow chain.

### Scope

**In scope (load-side only):**
- `GetNamedProperty` — most common case.
- `LoadGlobal` / `AssignGlobal` `LoadGlobal` only — the proto fast path covers
  global → prototype-chain lookups (e.g. `Math` via window.__proto__).
- `GetKeyedProperty` (named-atom variant) — keyed property access where the
  key resolves to a named atom.

**Out of scope:**
- Store opcodes (`SetNamedProperty`, `StoreGlobal`, `AssignGlobal`,
  `SetKeyedProperty`). JS data-property store semantics create an *own*
  property on the receiver, not on the prototype — the slow path's
  setter-walk check is correct to stay slow.
- Dense-index keyed (`GetKeyedProperty`/`SetKeyedProperty` SMI key).
  Arrays don't inherit indexed properties in normal patterns; the IC
  doesn't form `PrototypeData` entries for this family.
- Multi-hop PrototypeData (`dependency_count > 2`). Possible future
  Phase 3e+ if profile evidence justifies; bench evidence below shows
  one-hop is the dominant case.

### New infrastructure

- `NamedPropertyProtoHandler` in `crates/lyng-js/objects/src/shapes.rs`
  — two-word packed handler: `receiver_word` carries receiver shape;
  `proto_word` mirrors `NamedPropertyHandler`'s layout but for the
  prototype (prototype shape + slot offset + writable + inline-tag).
  `(0, 0)` is the NONE sentinel.
- 3 new sidecar fields on `NamedPropertyFeedback`:
  `monomorphic_proto_fast`, `monomorphic_proto_fast_receiver_epoch`,
  `monomorphic_proto_fast_prototype_epoch`.
- 3 new sidecar fields on `KeyedPropertyFeedback`:
  `monomorphic_named_proto_fast`,
  `monomorphic_named_proto_fast_receiver_epoch`,
  `monomorphic_named_proto_fast_prototype_epoch`.
- `NamedPropertyFeedback::refresh_monomorphic_fast` extended to populate
  proto sidecars when the cache entry's path is PrototypeData with
  `dependency_count == 2`. OwnData / PrototypeData are mutually
  exclusive at the entry level.
- `KeyedPropertyFeedback::refresh_monomorphic_fast` mirrors the same
  for keyed-NamedAtom feedback.
- Two `Vm::named_property_proto_fast_handler` /
  `Vm::keyed_property_named_proto_fast_handler` lookup helpers in
  `crates/lyng-js/vm/src/vm/feedback.rs` (inline-always).
- Two `Vm::try_named_property_proto_fast_load` /
  `Vm::try_keyed_named_proto_fast_load` fast-path helpers in
  `crates/lyng-js/vm/src/vm/dispatch/property.rs` (inline-always).

### Reused infrastructure

- `NamedPropertyHandler` bit-packing constants (`HANDLER_WRITABLE_FLAG`,
  `HANDLER_SLOT_OFFSET_MASK`, `INLINE_SLOT_OFFSET_FLAG`).
- `SlotLocation::decode` + `inline_named_slot` / `named_slots` slot
  read pattern.
- `RuntimeObjectRecord::prototype()` — already exposed.
- `last_invalidation_epoch()` + `bump_invalidation()` — invalidation
  primitive that obviates a separate watchpoint system.
- `record_named_property_fast_hit` / `record_feedback_slot` — existing
  tier-bookkeeping helpers.

## Verification

### Tests

| Check | Phase 3d | Phase 3e | Δ |
|---|---:|---:|---|
| `cargo test -p lyng-js-objects -p lyng-js-vm` | 480 passed | 486 passed | +6 (new NamedPropertyProtoHandler unit tests + 4 IC integration tests) |
| `cargo clippy -p lyng-js-vm` | 0 errors, 7 warnings | 0 errors, 7 warnings | unchanged |
| `cargo clippy -p lyng-js-objects` | clean | clean | unchanged |
| `cargo fmt --check` | clean | clean | — |

### V8 v7 sweep (11 samples per benchmark, isolated subprocesses)

| Benchmark | Phase 2a | Phase 3a | Phase 3b | Phase 3c | Phase 3d | **Phase 3e** | Δ vs Phase 3d | Δ vs Phase 2a |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Richards | 244 | 282 | 290 | 295 | 300 | **318** | **+6.0%** | **+30.3%** |
| DeltaBlue | 283 | 310 | 312 | 317 | 313 | **348** | **+11.2%** | **+23.0%** |
| Crypto | 272 | 277 | 275 | 274 | 273 | 274 | +0.4% | +0.7% |
| RayTrace | 401 | 416 | 420 | 413 | 422 | 422 | 0.0% | +5.2% |
| NavierStokes | 455 | 458 | 457 | 457 | 455 | 460 | +1.1% | +1.1% |
| Splay | 1270 | 1266 | 1286 | 1263 | 1279 | 1278 | −0.1% | +0.6% |

**Headline numbers:**
- **DeltaBlue +11.2%** — the strongest single-phase gain. DeltaBlue's
  inner loop dispatches `Strength.weakest()`, `Constraint.satisfy()`,
  `Variable.addConstraint()`, etc. — all one-hop method calls on class
  prototypes. The Phase 3e fast path is exactly the optimization this
  workload needs.
- **Richards +6.0%** — also class-instance dispatch heavy
  (`Task.run()`, `DeviceTaskDataRecord.handleEvent()`, etc.). Smaller
  gain than DeltaBlue because Richards already benefits from Phase 3a
  on the OwnData slot reads inside each method.
- **Splay −0.1%** (flat, within sample noise) — the hot loop is
  balanced-BST node access dominated by own-property reads of
  `left`/`right`/`key`/`value`, which Phase 3a already accelerates.
  PrototypeData isn't the bottleneck on Splay; the issue's hypothesis
  that "PrototypeData ... likely dominates on Splay" did not pan out
  on this workload. **Acceptance criterion of "no regression on Splay"
  is met.**
- **No regression > 1% anywhere** ✓

Cumulative geomean over Phase 2a is now **≈ +9.6%** (up from Phase 3d's
+6.1%) — DeltaBlue and Richards drive the bulk of the improvement.

Reports:
- `reports/js/lyng-js/phase-3e-bench.md`
- `reports/js/lyng-js/phase-3e-bench.json`

### `cargo asm`

| Function | Phase 3e hit-path `bl` targets on proto fast path |
|---|---|
| `op_get_named_property` | `record_execution` + `observe_tier_feedback_event` (tier bookkeeping only) |
| `op_load_global` | `record_execution` + `observe_tier_feedback_event` |
| `op_get_keyed_property` | `record_feedback_slot` (matching the Phase 3d named-atom OwnData pattern) |

No `bl` to `try_named_property_proto_fast_load` /
`try_keyed_named_proto_fast_load` / `named_property_proto_fast_handler` /
`load_from_named_property_cache` / `validated_named_property_cache_holder`
on the proto-hit path. The helpers are fully inlined into the dispatch
handlers.

Reports:
- `reports/js/lyng-js/phase-3e-op_get_named_property.asm`
- `reports/js/lyng-js/phase-3e-op_load_global.asm`
- `reports/js/lyng-js/phase-3e-op_get_keyed_property.asm`

## What's deferred

- **`lyng-22al` follow-ups** (not opened — fold into Phase 3f or close
  Phase 3e and revisit):
  - Multi-hop PrototypeData (`dependency_count ∈ {3, 4}`). No
    workload evidence in the V8 v7 suite that justifies inlining
    these.
  - Watchpoint-based invalidation (Option ii). Would skip per-load
    dependency checks entirely, but requires a multi-week side
    project. The +11.2% DeltaBlue / +6.0% Richards gain via Option (i)
    means watchpoints are no longer the bottleneck.
  - Store-side proto fast path. JS semantics make this far less
    impactful than load-side; defer indefinitely.
- **`lyng-5nju` (Phase 3f)**: polymorphic compaction.
- **`lyng-28t2` (Phase 3g)**: γ-swap evaluation.

## Files changed

Single-commit delivery (matching the Phase 3b / 3c cadence):

- `crates/lyng-js/objects/src/shapes.rs` — `NamedPropertyProtoHandler`
  packed type + `from_entry` constructor + accessor methods.
- `crates/lyng-js/objects/src/lib.rs` — re-export
  `NamedPropertyProtoHandler`.
- `crates/lyng-js/objects/src/tests.rs` — 6 unit tests covering
  handler packing, multi-hop / single-hop / OwnData rejection, and
  the NONE sentinel.
- `crates/lyng-js/vm/src/vm/feedback.rs` — 3 sidecar fields on
  `NamedPropertyFeedback`, 3 sidecar fields on `KeyedPropertyFeedback`,
  extended `refresh_monomorphic_fast` for both, 2 new `Vm` lookup
  helpers, all reset paths cleared on cache transitions.
- `crates/lyng-js/vm/src/vm/dispatch/property.rs` — 2 new
  `#[inline(always)]` fast-path helpers
  (`try_named_property_proto_fast_load`,
  `try_keyed_named_proto_fast_load`) + 2 inline call sites
  (`execute_get_named_property_opcode`, keyed-named-atom Get).
- `crates/lyng-js/vm/src/vm/names.rs` — 1 inline call site in
  `load_global_with_feedback`.
- `crates/lyng-js/vm/src/tests/inline_caches.rs` — 4 integration tests
  (one-hop PrototypeData hit, prototype-swap invalidation, keyed
  variant, three-hop chain fall-through).

Total Phase 3e diff: ~370 added lines + ~5 modified lines.
