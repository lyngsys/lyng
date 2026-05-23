# Phase 3f Status — Polymorphic compaction for IC fast path (lyng-5nju)

Phase 3a–3e inlined the IC fast path for the monomorphic case: a single
packed `monomorphic_fast: NamedPropertyHandler` word on
`NamedPropertyFeedback` lets the 10 IC-shaped opcodes do a one-cmp /
one-eq / one-load fast path that bypasses the 4-deep slow chain. Phase
3e extended that to a one-hop `PrototypeData` companion word.

Phase 3f layers polymorphic OwnData on top: a fixed-size
`[NamedPropertyHandler; POLY_LIMIT]` sidecar on the feedback site lets
2..POLY_LIMIT cached shapes also resolve inline, without entering the
binary-search slow chain. Modeled after JSC's `PolymorphicAccess` —
monomorphic remains bit-packed inline; polymorphic becomes a small
fixed-size walk inline up to a cap; mega-poly transitions zero the
sidecar and fall to the existing slow path unchanged.

## POLY_LIMIT decision

V8 v7 suite, 11 samples per benchmark, isolated lyng subprocess per
sample. Scored against the Phase 3e baseline (Richards 318, DeltaBlue
348, Crypto 274, RayTrace 422, NavierStokes 460, Splay 1278).

| POLY_LIMIT | Richards | DeltaBlue | Crypto | RayTrace | NavierStokes | Splay | Memory per site |
|---|---:|---:|---:|---:|---:|---:|---:|
| **2** (chosen) | **318** (=) | **362** (+4.0%) | **274** (=) | **428** (+1.4%) | **460** (=) | **1374** (+7.5%) | +32 bytes |
| 4 | 315 (−0.9%) | 358 (+2.9%) | 272 (−0.7%) | 419 (−0.7%) | 463 (+0.7%) | 1376 (+7.7%) | +64 bytes |
| 8 | 316 (−0.6%) | 359 (+3.2%) | 274 (=) | 429 (+1.7%) | 463 (+0.7%) | 1387 (+8.5%) | +128 bytes |

(Wider POLY_LIMIT results from the same hardware at 5 samples — the
4-/8-row deltas above are 5-sample medians while POLY_LIMIT=2 is the
canonical 11-sample run committed at `phase-3f-bench.{md,json}`.)

**POLY_LIMIT = 2** is the final value. It gives the strongest
DeltaBlue gain (the most-poly workload in the suite), a real Splay
gain, and zero regressions on Richards / Crypto / NavierStokes /
RayTrace, at the smallest sidecar footprint. 8-entry capacity adds
+96 bytes per feedback site (×8 sites per typical hot loop is +768
bytes) for ≤0.5% extra geomean over 2-entry — not defensible.

Cumulative geomean over Phase 0 sits at **≈ +18%** by Phase 3f.
DeltaBlue is now **+30.7%** over its Phase 1 baseline; Richards **+35.9%**.

Reports:
- `reports/lyng/phase-3f-bench.md`
- `reports/lyng/phase-3f-bench.json`

## Inline-asm verification

`cargo asm` snapshots of the 5 representative IC handlers committed at:

- `reports/lyng/phase-3f-op_get_named_property.asm` (788 lines)
- `reports/lyng/phase-3f-op_load_global.asm` (737 lines)
- `reports/lyng/phase-3f-op_get_keyed_property.asm` (1422 lines)
- `reports/lyng/phase-3f-op_set_named_property_common.asm` (3281 lines)
- `reports/lyng/phase-3f-op_store_or_assign_global.asm` (1272 lines)
- `reports/lyng/phase-3f-op_set_keyed_property_common.asm` (2266 lines)

| Handler | `bl` to polymorphic helpers on fast path | `bl` to slow chain on fast path |
|---|---|---|
| `op_get_named_property` | none | none — slow chain only on miss |
| `op_load_global` | none | none — slow chain only on miss |
| `op_get_keyed_property` | none | none — slow chain only on miss |
| `op_set_named_property_common` | none | none — slow chain only on miss |
| `op_store_or_assign_global` | none | none — slow chain only on miss |
| `op_set_keyed_property_common` | none | none — slow chain only on miss |

No `bl` to `try_named_property_polymorphic_fast_load`,
`try_named_property_polymorphic_fast_store`,
`try_keyed_named_polymorphic_fast_load`,
`try_keyed_named_polymorphic_fast_store`,
`try_keyed_dense_polymorphic_fast_load`,
`try_keyed_dense_polymorphic_fast_store`,
`named_property_polymorphic_fast_handler`,
`keyed_property_named_polymorphic_fast_handler`, or
`keyed_property_dense_polymorphic_fast_handler` on any hit-path linear
region. The helpers are fully inlined into each dispatch handler.

`bl record_named_property_fast_hit`, `bl record_feedback_slot`, and
`bl observe_tier_feedback_event` remain on the hit path (tier
bookkeeping, identical to Phase 3a–3e). Slow-chain `bl` to
`try_named_property_load_inline_cache_hit` /
`try_named_property_store_inline_cache` etc. appears only on the
fall-through path after both monomorphic and polymorphic inline checks
miss.

## test262

| Run | Passed files | Failed files |
|---|---:|---:|
| Pristine HEAD (today's corpus, this hardware) | `49710` | `19` |
| Phase 3f | `49711` | `18` |

Phase 3f matches the pristine HEAD pass count to within +1 file (+1
variant); no new failures. The pre-existing failure clusters
(`built-ins/Promise/*`, `harness/deepEqual-*`, `staging/sm/TypedArray/*`,
etc.) reproduce identically on pristine HEAD and are unrelated to the
polymorphic fast path. The historical phase-4b-test262.md baseline
(49721 passed / 8 failed) reflects an older test262 corpus state on
this checkout; Phase 3f does not regress against the *current* corpus.

Report: `reports/lyng/phase-3f-test262.md`.

## What landed

### Data layout (`crates/lyng/vm/src/vm/feedback.rs`)

`NamedPropertyFeedback` adds:
- `polymorphic_fast: [NamedPropertyHandler; POLY_LIMIT]`
- `polymorphic_fast_dependency_epochs: [u64; POLY_LIMIT]`

`KeyedPropertyFeedback` adds:
- `polymorphic_named_fast: [NamedPropertyHandler; POLY_LIMIT]`
- `polymorphic_named_fast_atoms: [u32; POLY_LIMIT]`
- `polymorphic_named_fast_dependency_epochs: [u64; POLY_LIMIT]`
- `polymorphic_dense_fast: [KeyedDenseIndexHandler; POLY_LIMIT]`

All initialized to NONE / 0 in `new()` and reset on every relevant
cache transition.

### Transitions

- `NamedPropertyFeedback::refresh_monomorphic_fast` — extended to also
  populate `polymorphic_fast` on `Polymorphic` state. Clears the whole
  sidecar each call before repopulating.
- `NamedPropertyFeedback::insert_entry_at` — now calls
  `refresh_monomorphic_fast` after the Monomorphic→Polymorphic
  transition (the function already cleared mono/proto words explicitly;
  the refresh consolidates that with the new poly population).
- `NamedPropertyFeedback::observe_slow_path` — refreshes on every
  update to `entries[0..POLY_LIMIT]` (was `index == 0`), since any
  inline-sidecar slot may now need repacking.
- `NamedPropertyFeedback::promote_to_megamorphic` — delegates the
  sidecar clear to `refresh_monomorphic_fast`.
- `KeyedPropertyFeedback::refresh_monomorphic_fast` — restructured to
  match both `cache_state` and `family`, populating the appropriate
  monomorphic words *and* the matching polymorphic sidecar.
- `KeyedPropertyFeedback::promote_to_megamorphic` — clears the new
  polymorphic arrays alongside the existing monomorphic words.

### Fast-path entry points

New helpers in `crates/lyng/vm/src/vm/feedback.rs`:
- `Vm::named_property_polymorphic_fast_handler(code, slot, shape)`
- `Vm::keyed_property_named_polymorphic_fast_handler(code, slot, atom, shape)`
- `Vm::keyed_property_dense_polymorphic_fast_handler(code, slot)`

All `#[inline(always)]`, walk `0..POLY_LIMIT` against the runtime
shape (+ atom for keyed-named, + flags for dense), return the matching
handler on hit or `None` on miss.

New inline helpers in `crates/lyng/vm/src/vm/dispatch/property.rs`:
- `Vm::try_named_property_polymorphic_fast_load`
- `Vm::try_named_property_polymorphic_fast_store`
- `Vm::try_keyed_named_polymorphic_fast_load`
- `Vm::try_keyed_named_polymorphic_fast_store`
- `Vm::try_keyed_dense_polymorphic_fast_load`
- `Vm::try_keyed_dense_polymorphic_fast_store`

Each wraps the corresponding `*_polymorphic_fast_handler` lookup with
the receiver-side epoch check and slot read/write, mirroring the
Phase 3e proto-fast helper shape.

### Call-site additions (10 IC opcodes)

Inserted between the existing monomorphic-OwnData block (Phase 3a)
and the one-hop proto block (Phase 3e):

- `op_get_named_property` (Get/StrictGet) — load side
- `op_set_named_property_common` (Set/Assign/StrictAssign) — store side
- `op_load_global` (LoadGlobal) — load side
- `op_store_or_assign_global` (StoreGlobal/AssignGlobal) — store side
- `op_get_keyed_property` — dense-index + named-atom load sides
- `op_set_keyed_property_common` — dense-index + named-atom store sides

### Tests (`crates/lyng/vm/src/tests/inline_caches.rs`)

5 new tests covering Phase 3f-specific behavior:
- `named_property_load_ic_polymorphic_fast_load_returns_value_for_two_shapes`
- `named_property_load_ic_polymorphic_fast_load_falls_through_beyond_poly_limit`
- `named_property_store_ic_polymorphic_fast_store_writes_correct_slot`
- `named_property_load_ic_polymorphic_fast_load_invalidates_on_prototype_swap`
- `keyed_named_property_load_ic_polymorphic_fast_load_returns_value_for_two_shapes`

411 vm tests pass (up from 406 at Phase 3e).

## Memory cost

| `NamedPropertyFeedback` field | Size delta |
|---|---:|
| `polymorphic_fast` | 16 bytes (2 × 8) |
| `polymorphic_fast_dependency_epochs` | 16 bytes (2 × 8) |
| **Total per named-prop site** | **+32 bytes** |

| `KeyedPropertyFeedback` field | Size delta |
|---|---:|
| `polymorphic_named_fast` | 16 bytes |
| `polymorphic_named_fast_atoms` | 8 bytes |
| `polymorphic_named_fast_dependency_epochs` | 16 bytes |
| `polymorphic_dense_fast` | 16 bytes |
| **Total per keyed-prop site** | **+56 bytes** |

(Per-site struct growth; actual per-script feedback-vector cost
scales with the live-site count per script and is summarized in the
existing `FeedbackVectorFootprint` accounting.)

## What's deferred

- **Polymorphic prototype-data fast path** — Phase 3f covers OwnData
  only. PrototypeData polymorphism (e.g., class hierarchies where the
  same method is reached through different receiver shapes that all
  delegate to the same prototype) still walks the slow chain. The V8
  v7 evidence (DeltaBlue and Splay are the bigger wins) suggests
  OwnData polymorphism is the dominant case; if class-method
  dispatch shows up as a bottleneck post-Phase-3f, open a follow-up.
- **Phase 3g γ-swap evaluation** (`lyng-28t2`) — separate sub-issue.
  After Phase 3f, the central trampoline indirect call is the
  remaining dispatch overhead; the gated re-profile that decides
  whether to swap `dispatch_next!` for an inline-asm tail call
  belongs in 3g.

## Files changed

- `crates/lyng/vm/src/vm/feedback.rs` — `POLY_LIMIT` constant,
  4 new sidecar fields on `NamedPropertyFeedback`, 4 new fields on
  `KeyedPropertyFeedback`, extended `refresh_monomorphic_fast` for
  both, 3 new lookup helpers, all reset paths cleared on cache
  transitions.
- `crates/lyng/vm/src/vm/dispatch/property.rs` — 6 new inline
  helpers, 6 new call sites (Get/Set named, Get/Set keyed-dense,
  Get/Set keyed-named).
- `crates/lyng/vm/src/vm/names.rs` — 3 new call sites
  (LoadGlobal, StoreGlobal, AssignGlobal).
- `crates/lyng/vm/src/tests/inline_caches.rs` — 5 new tests +
  one shared `make_object_with_value` helper.
- `reports/lyng/phase-3f-*` — bench, test262, asm snapshots,
  and this status report.
