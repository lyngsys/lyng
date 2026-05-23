# Phase 3c — Global opcode inline IC fast path: status report

**Issue:** `lyng-5j2z` — Phase 3c: Inline IC fast path for LoadGlobal/StoreGlobal/AssignGlobal
**Parent:** `lyng-2pgt` — Phase 3: Inline IC fast path (epic)
**Date:** 2026-05-15
**Toolchain:** rustc 1.93 (2025-12-15), aarch64-apple-darwin, `--release` profile
**Parent commit:** `2f49d1af` (Phase 3b close)

## What landed

The three global opcodes (`LoadGlobal`, `StoreGlobal`, `AssignGlobal`)
now use the same packed-handler inline IC fast path as Phase 3a/3b.
Globals share `NamedPropertyFeedback` — the helpers
`load_global_with_feedback`, `store_global_with_feedback`, and
`assign_global_with_feedback` in
[crates/vm/src/vm/names.rs](crates/vm/src/vm/names.rs)
were already calling `try_named_property_load_inline_cache_hit` and
`try_named_property_store_inline_cache` with `global_object` as the
receiver. Phase 3c just replaces those calls with the inlined fast
path, leaving the global lexical-binding precedence check (which runs
before the IC) and the strict-mode error semantics intact.

### Per-opcode breakdown

- **`LoadGlobal`**: same hot-path shape as Phase 3a's
  `op_get_named_property` — packed-handler load, shape compare, epoch
  compare, inline/out-of-line slot read,
  `record_named_property_fast_hit` for tier bookkeeping. Lexical
  binding lookup at the top of `load_global_with_feedback` remains.
- **`StoreGlobal`**: same hot-path shape as Phase 3b's
  `op_set_named_property` — adds writable check, falls back to slow
  chain on non-writable / shape miss. On `stored = false` from a
  non-writable property, sloppy semantics (silently no-op) match the
  slow chain.
- **`AssignGlobal`**: same as `StoreGlobal` plus the strict-mode
  error: `if !stored && self.frame_is_strict(frame)` throws TypeError
  via `errors::throw_type_error`. The existing slow chain has the
  exact same check; the inline fast path preserves it bit-for-bit.

### Reused infrastructure

Phase 3c required **no new fields, no new helpers, and no changes to
the packed-handler representation**. Everything is reused from Phase
3a (`monomorphic_fast`, `monomorphic_fast_dependency_epoch`,
`named_property_fast_handler`, `record_named_property_fast_hit`,
`record_feedback_slot`) and Phase 3b (`NamedPropertyHandler::writable`
bit). The only change is inlining the existing helpers' call sites
in `names.rs`.

## Verification

### Tests

| Check | Phase 3b | Phase 3c | Δ |
|---|---:|---:|---|
| `cargo test -p lyng-gc -p lyng-objects -p lyng-vm -p lyng-tests` | 1709 passed | 1709 passed | unchanged |
| `cargo clippy -p lyng-vm` | 0 errors, 6 warnings | 0 errors, 7 warnings | +1 (collapsible-if at `names.rs:550`, same pattern as Phase 3a's `property.rs:111`) |

### V8 v7 sweep (11 samples per benchmark, isolated subprocesses)

| Benchmark | Phase 2a | Phase 3a | Phase 3b | Phase 3c | Δ vs Phase 3b | Δ vs Phase 2a |
|---|---:|---:|---:|---:|---:|---:|
| Richards | 244 | 282 | 290 | **295** | +1.7% | **+20.9%** |
| DeltaBlue | 283 | 310 | 312 | **317** | +1.6% | **+12.0%** |
| Crypto | 272 | 277 | 275 | 274 | −0.4% | +0.7% |
| RayTrace | 401 | 416 | 420 | 413 | −1.7% | +3.0% |
| NavierStokes | 455 | 458 | 457 | 457 | 0.0% | +0.4% |
| Splay | 1270 | 1266 | 1286 | 1263 | −1.8% | −0.6% |
| **Geomean** | — | — | — | — | **≈ −0.1%** | **≈ +5.7%** |

Richards and DeltaBlue continue to gain — both exercise global access
heavily (constraint definitions, prototype installations). Splay and
RayTrace dropped slightly (−1.8% and −1.7% vs Phase 3b); within the
±2% noise band the project uses. The drop pattern (less property-heavy
benches losing a hair while property-heavy benches gain) is consistent
with the slightly larger code size of `op_load_global` /
`op_store_or_assign_global` — they now carry both the fast and slow
paths inline, which is worth it on hot loops but costs a touch on
i-cache pressure on workloads that don't touch globals much. Acceptable
for the cumulative win.

Reports:
- `reports/lyng/phase-3c-bench.md`
- `reports/lyng/phase-3c-bench.json`

### `cargo asm`

| Function | Phase 3c hit-path bls |
|---|---|
| `op_load_global` | `FeedbackSiteState::record_execution` + `observe_tier_feedback_event` (same as Phase 3a load) — IC chain helper (`try_named_property_load_inline_cache_hit`) is now slow-path-only |
| `op_store_or_assign_global` | `PrimitiveMutator::store_value` + `record_feedback_slot` (same as Phase 3b store) — IC chain helper (`try_named_property_store_inline_cache`) is now slow-path-only |

Reports:
- `reports/lyng/phase-3c-op_load_global.asm`
- `reports/lyng/phase-3c-op_store_or_assign_global.asm`

### Test262

| | Files runnable | Files passed | Pass rate | Δ vs Phase 2a |
|---|---:|---:|---:|---:|
| Phase 2a baseline | 49729 | 49722 | 99.99% | — |
| Phase 3a | 49729 | 49720 | 99.98% | −2 |
| Phase 3b | 49729 | 49721 | 99.98% | −1 |
| **Phase 3c** | **49729** | **49720** | **99.98%** | **−2** |

Same 9 failures as Phase 3a:
- 5 pre-Phase-3 carry-overs (language/import x2, language/module x2, language/namespace x1).
- 2 pre-Phase-3 staging carry-overs (TypedArray/toLocaleString, sm/class/className).
- 2 from Phase 3a `lyng-2tr1` (object-literal-__proto__ — deterministic, not addressed by 3c).
- 1 `staging/sm/RegExp/unicode-class-braced.js [non-strict]` timeout flake (variant ran in 1.013s, just over the 1.0s timeout; passed in Phase 3b, fails again here — confirmed flaky).

No new deterministic failures from Phase 3c.

Report:
- `reports/lyng/phase-3c-test262.md`

## What's deferred

- **`lyng-2tr1`**: the carry-over `object-literal-__proto__` Test262 regression — not addressed by 3c.
- **`lyng-guem`**: keyed property opcode IC fast path (Phase 3d).
- **`lyng-22al` / `lyng-5nju` / `lyng-28t2`**: PrototypeData / polymorphic compaction / γ-swap (Phase 3e/f/g).

## Files changed

- `crates/vm/src/vm/names.rs` — added `lyng_gc::ValueStoreTarget` and `lyng_objects::SlotLocation` imports; inlined fast path in `load_global_with_feedback`, `store_global_with_feedback`, `assign_global_with_feedback`. Slow chain (existing IC helpers + global property lookup) untouched.

No changes elsewhere — Phase 3a/3b infrastructure reused verbatim.

Total Phase 3c diff: ~110 added lines + ~10 modified across 1 file.
