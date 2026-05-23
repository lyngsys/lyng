# Phase 3d — Keyed property opcode inline IC fast path: status report

**Issue:** `lyng-guem` — Phase 3d: Inline IC fast path for keyed property opcodes
**Parent:** `lyng-2pgt` — Phase 3: Inline IC fast path (epic)
**Date:** 2026-05-15
**Toolchain:** rustc 1.93 (2025-12-15), aarch64-apple-darwin, `--release` profile
**Parent commit:** `e3d40157` (Phase 3c close)
**Commits in this sub:**
- `<3d-A>` Phase 3d (1/2) — packed handler + KeyedPropertyFeedback sidecar infra
- `<3d-B>` Phase 3d (2/2) — inline both family fast paths in dispatch

## What landed

The keyed property IC dispatch chain on the monomorphic hit path is inlined
for all four keyed opcodes (`GetKeyedProperty`, `SetKeyedProperty`,
`AssignKeyedProperty`, `StrictAssignKeyedProperty`), covering **both**
keyed families:

- **Dense-index family** (numeric SMI keys against array receivers): packed
  `KeyedDenseIndexHandler` encoding `(receiver_shape, receiver_flags)` →
  shape + flags compare → barrier-aware element write/read.
- **Named-atom family** (string-atom keys): reuses Phase 3a/3b's
  `NamedPropertyHandler` (shape + slot_offset + writable + inline-tag) plus
  a parallel `monomorphic_named_fast_atom: u32` sidecar so the fast path
  also checks runtime atom equality.

No `bl` to `try_keyed_dense_index_load_inline_cache_hit`,
`try_keyed_dense_index_store_inline_cache_hit`,
`try_keyed_property_load_inline_cache`,
`try_keyed_property_store_inline_cache`, `try_dense_index_load`, or
`try_dense_index_store` on the hit path. The four new `#[inline(always)]`
fast-path helpers (`try_keyed_dense_fast_{load,store}` and
`try_keyed_named_fast_{load,store}`) fully inline into both
`execute_get_keyed_property_opcode` and `execute_set_keyed_property_opcode`.

Polymorphic / Generic / megamorphic / out-of-bounds / non-dense fall
through to the existing keyed slow chain unchanged.

### Per-family details

**Dense fast path** (called for both SMI key and post-coercion `key.as_index()`):
- Look up packed `KeyedDenseIndexHandler` via `keyed_property_dense_fast_handler`.
- Fetch `ObjectHeader` (one `bl` to `object_header` — same cost as the
  existing slow-chain's first call).
- Compare cached shape + flags against header.
- Load `elements()`, look up slot at the runtime index.
- Bail if the slot is `array_hole` (existing slow chain handles
  prototype walk for holes).
- On store: barrier-aware `mut_store_value(ObjectSlot(elements, index), value)`.
- Bookkeeping: `record_execution + observe_tier_feedback_event` (same as
  the existing `try_keyed_dense_index_*_inline_cache_hit` helpers do
  internally).

**Named-atom fast path** (called for post-coercion `key.as_atom()`):
- Look up packed `NamedPropertyHandler` + epoch via
  `keyed_property_named_fast_handler` (with atom-equality check baked in).
- Read receiver record via `view.object_ref(receiver)` (no `bl`).
- Shape + epoch compare.
- Load slot value (inline / out-of-line).
- On store: writable bit check (non-writable ⇒ `stored = false` matching
  slow-chain semantics, becomes TypeError under strict assignment),
  barrier-aware `mut_store_value`.
- Bookkeeping: `record_feedback_slot` (matching the slow chain's existing
  atom-hit bookkeeping in dispatch).

### Reused infrastructure

- `NamedPropertyHandler` (Phase 3a/3b) — packed shape + offset + writable + inline-tag.
- `record_named_property_fast_hit` — used by the dense fast path for tier bookkeeping.
- `record_feedback_slot` — used by the named-atom fast path (same as the existing dispatch-layer call).
- `agent.objects().object_header(view, receiver)` — already in the slow chain.
- `ValueStoreTarget::ObjectSlot` and `mut_store_value` — already used by Phase 3b/3c.

### New types and infrastructure

- `KeyedDenseIndexHandler` in `crates/lyng/objects/src/shapes.rs` —
  packs `(receiver_shape: NonZeroU32, receiver_flags: ObjectFlags u16)`
  into a single u64. Low half == 0 ⇒ NONE sentinel.
- `ObjectFlags::bits` / `from_bits` in `crates/lyng/objects/src/core.rs`
  — public raw 16-bit accessors needed by the packed handler.
- 4 new sidecar fields on `KeyedPropertyFeedback`:
  `monomorphic_named_fast`, `monomorphic_named_fast_atom`,
  `monomorphic_named_fast_dependency_epoch`, `monomorphic_dense_fast`.
- `KeyedPropertyFeedback::refresh_monomorphic_fast()` — recomputes all 4
  fields from `named_entries[0] / dense_entries[0]` based on
  `cache_state + family`. Wired into 3 entry points:
  `observe_named_atom_slow_path`, `observe_dense_index`, and
  `promote_to_megamorphic`.
- Two `Vm::keyed_property_{named,dense}_fast_handler` helpers in
  `crates/lyng/vm/src/vm/feedback.rs` (inline-always).
- Four `Vm::try_keyed_{dense,named}_fast_{load,store}` helpers in
  `crates/lyng/vm/src/vm/dispatch/property.rs` (inline-always).

## Verification

### Tests

| Check | Phase 3c | Phase 3d | Δ |
|---|---:|---:|---|
| `cargo test -p lyng-gc -p lyng-objects -p lyng-vm -p lyng-tests` | 1709 passed | 1712 passed | +3 (new KeyedDenseIndexHandler unit tests) |
| `cargo clippy -p lyng-vm` | 0 errors, 7 warnings | 0 errors, 7 warnings | unchanged |

### V8 v7 sweep (11 samples per benchmark, isolated subprocesses)

| Benchmark | Phase 2a | Phase 3a | Phase 3b | Phase 3c | Phase 3d | Δ vs Phase 3c | Δ vs Phase 2a |
|---|---:|---:|---:|---:|---:|---:|---:|
| Richards | 244 | 282 | 290 | 295 | **300** | +1.7% | **+23.0%** |
| DeltaBlue | 283 | 310 | 312 | 317 | 313 | −1.3% | **+10.6%** |
| Crypto | 272 | 277 | 275 | 274 | 273 | −0.4% | +0.4% |
| RayTrace | 401 | 416 | 420 | 413 | **422** | +2.2% | **+5.2%** |
| NavierStokes | 455 | 458 | 457 | 457 | 455 | −0.4% | 0.0% |
| Splay | 1270 | 1266 | 1286 | 1263 | **1279** | +1.3% | +0.7% |
| **Geomean** | — | — | — | — | — | **≈ +0.3%** | **≈ +6.1%** |

Splay recovered from Phase 3c's −0.6% to +0.7% — keyed dense-index
access is meaningful on tree-node traversal. RayTrace got the biggest
single-phase gain (+2.2% vs 3c) — its inner loops use `vec[0]`,
`vec[1]`, `vec[2]` patterns. Richards continues to creep up.

Cumulative geomean over Phase 2a is now **≈ +6.1%** — the highest yet
across the four sub-issues 3a-3d.

Reports:
- `reports/lyng/phase-3d-bench.md`
- `reports/lyng/phase-3d-bench.json`

### `cargo asm`

| Function | Phase 3d hit-path `bl` targets |
|---|---|
| `op_get_keyed_property` | `object_header` (dense flags), `record_execution + observe_tier_feedback_event` (dense bookkeeping) — atom load uses `view.object_ref` (no bl) + `record_feedback_slot` |
| `op_set_keyed_property_common` | `object_header` (dense flags), `mut_store_value` (barrier-aware write), `record_execution + observe_tier_feedback_event` or `record_feedback_slot` |

IC chain helpers (`try_keyed_*_inline_cache*` and `try_dense_index_*`)
are gone from the hit path — they sit on the slow-fall-through branches
for polymorphic / Generic / megamorphic / miss only.

Reports:
- `reports/lyng/phase-3d-commit-{a,b}-op_get_keyed_property.asm`
- `reports/lyng/phase-3d-commit-{a,b}-op_set_keyed_property_common.asm`

### Test262

| | Files runnable | Files passed | Pass rate | Δ vs Phase 2a |
|---|---:|---:|---:|---:|
| Phase 2a baseline | 49729 | 49722 | 99.99% | — |
| Phase 3a | 49729 | 49720 | 99.98% | −2 |
| Phase 3b | 49729 | 49721 | 99.98% | −1 |
| Phase 3c | 49729 | 49720 | 99.98% | −2 |
| **Phase 3d** | **49729** | **49720** | **99.98%** | **−2** |

Same 9 file failures as Phase 3a/3c. Variant count is 8 staging
failures (vs Phase 3c's 7) — the `unicode-class-braced` timeout now
hits both strict and non-strict variants instead of just non-strict.
File-level pass count is unchanged. No new deterministic failures
from Phase 3d.

Report:
- `reports/lyng/phase-3d-test262.md`

## What's deferred

- **`lyng-2tr1`**: the carry-over `object-literal-__proto__` Test262 regression — not addressed by 3d.
- **`lyng-22al` (Phase 3e)**: PrototypeData inline path.
- **`lyng-5nju` (Phase 3f)**: polymorphic compaction (extends the packed-handler scheme to N≥2 entries inline).
- **`lyng-28t2` (Phase 3g)**: γ-swap evaluation.

## Files changed

**Phase 3d (1/2)** — infrastructure:
- `crates/lyng/objects/src/core.rs` — `ObjectFlags::bits / from_bits` raw accessors.
- `crates/lyng/objects/src/shapes.rs` — `KeyedDenseIndexHandler` packed type.
- `crates/lyng/objects/src/lib.rs` — re-export `KeyedDenseIndexHandler`.
- `crates/lyng/objects/src/tests.rs` — 3 unit tests for dense handler.
- `crates/lyng/vm/src/vm/feedback.rs` — 4 sidecar fields + `refresh_monomorphic_fast` + 2 `Vm` lookup helpers.

**Phase 3d (2/2)** — consumer:
- `crates/lyng/vm/src/vm/dispatch/property.rs` — 4 `#[inline(always)]` fast-path helpers + 6 inline call sites (2 dense get, 1 atom get, 2 dense set, 1 atom set).

Total Phase 3d diff: ~310 added lines + ~30 modified across 6 files.
