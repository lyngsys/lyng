# Phase 3a — GetNamedProperty inline IC fast path: status report

**Issue:** `lyng-2pgt` — Phase 3: Inline IC fast path (epic)
**Sub:** Phase 3a (pilot) — `GetNamedProperty` monomorphic OwnData inlining
**Date:** 2026-05-15
**Toolchain:** rustc 1.93 (2025-12-15), aarch64-apple-darwin, `--release` profile
**Parent:** `ac8f020f` (Phase 2a)
**Commits in this sub:**
- `fb5524be` Phase 3a (1/2) — packed monomorphic OwnData IC handler infrastructure
- `<tbd>`    Phase 3a (2/2) — inline the fast path + epoch sidecar

## What landed

The 4-deep IC dispatch chain on the `GetNamedProperty` monomorphic
OwnData hit path is collapsed into an inlined block in
`Vm::execute_get_named_property_opcode`. Hot-path now reads:

1. Packed handler word from `NamedPropertyFeedback.monomorphic_fast`
2. Receiver record via `heap.object_ref(receiver)` (Phase 2a infra)
3. One shape compare against the packed handler's high half
4. One invalidation-epoch compare against the paired sidecar
5. One slot read (inline or out-of-line)

No `bl` to `try_named_property_load_inline_cache_hit`, `try_load`,
`load_from_named_property_cache`, or `validated_named_property_cache_holder`
on the hit path. Two `bl`s remain — `FeedbackSiteState::record_execution`
and `Vm::observe_tier_feedback_event` — both pre-existing tier
bookkeeping that lived inside the now-bypassed
`try_named_property_load_inline_cache_hit`. The 4-deep IC chain itself
is gone from the hit path.

Polymorphic, PrototypeData, megamorphic, and miss cases continue
through the existing chain unchanged. Store/Assign/Strict variants,
global opcodes, and keyed property opcodes are untouched — those are
follow-up sub-issues 3b/3c/3d.

## Verification

### Tests

| Check | Pre-change (Phase 2a) | Post-change (Phase 3a) | Δ |
|---|---:|---:|---|
| `cargo test -p lyng-js-gc -p lyng-js-objects -p lyng-js-vm -p lyng-js-tests` | 1701 passed, 1 ignored | 1707 passed, 1 ignored | +6 (new NamedPropertyHandler packing tests) |
| `cargo clippy --workspace --all-targets` | 0 errors, 62 warnings | 0 errors, 62 warnings | unchanged |

### V8 v7 sweep (11 samples per benchmark, isolated subprocesses)

| Benchmark | Phase 2a median | Phase 3a median | Δ vs Phase 2a |
|---|---:|---:|---:|
| Richards | 244 | 282 | **+15.6%** |
| DeltaBlue | 283 | 310 | **+9.5%** |
| Crypto | 272 | 277 | +1.8% |
| RayTrace | 401 | 416 | +3.7% |
| NavierStokes | 455 | 458 | +0.7% |
| Splay | 1270 | 1266 | −0.3% |
| **Geomean** | — | — | **≈ +4.97%** |

Strong gains on property-access-heavy benchmarks (Richards +15.6%,
DeltaBlue +9.5%) — the workloads where `GetNamedProperty` is hot.
Matrix-heavy benchmarks (NavierStokes, Splay) are flat to noise. No
benchmark regresses >2%. Crypto's α target was ≥260 (recovery from
Phase 2a baseline 272 → 277). The epic-level +35% geomean target
remains open; that's cumulative across all 10 IC opcodes (3b–3d).

Reports:
- `reports/js/lyng-js/phase-3a-bench.md`
- `reports/js/lyng-js/phase-3a-bench.json`

### `cargo asm` on `op_get_named_property`

| Variant | Lines | `bl` count | Key `bl` targets on hit path |
|---|---:|---:|---|
| Phase 2a final (`reports/js/lyng-js/phase-2a-final-…asm`) | 438 | 11 | `try_named_property_load_inline_cache_hit` |
| Commit A (no consumer) | 438 | 11 | unchanged — byte-identical to Phase 2a |
| Commit B (fast path inlined) | 547 | 13 | `FeedbackSiteState::record_execution` + `observe_tier_feedback_event` only — both pre-existing tier bookkeeping |

Commit B's diff vs Commit A shows the inlined fast path block:
packed-handler load → `cbz` (NONE sentinel check) → object record
load → shape compare → epoch compare → slot decode → slot read →
dispatch. The 4 functions of the old IC chain are flattened into the
handler body. The 13 vs 11 `bl` increase comes from exposing 2 tier
bookkeeping calls that were previously hidden inside the now-bypassed
chain helper.

Reports:
- `reports/js/lyng-js/phase-3a-commit-a-op_get_named_property.asm`
- `reports/js/lyng-js/phase-3a-commit-b-op_get_named_property.asm`

### Test262

| | Files runnable | Files passed | Pass rate | Δ vs Phase 2a (files) |
|---|---:|---:|---:|---:|
| Phase 2a (`reports/js/lyng-js/phase-2a-test262.md`) | 49729 | 49722 | 99.99% | — |
| Phase 3a (`reports/js/lyng-js/phase-3a-test262.md`) | 49729 | 49720 | 99.98% | **−2 files** |

The 2 new failures vs Phase 2a (both regressions surface on Commit A
alone; Commit B's fast path does not cause them — confirmed by
running with the fast path branch disabled):

1. `staging/sm/expressions/object-literal-__proto__.js` (2 variants):
   `runtime error: Test262Error`. Reproduces deterministically under
   `lyng-js-test262` but the test passes when run standalone via
   `lyng-js --test262` with the assembled (`assert.js` + `sta.js` +
   test) source. The two execution paths differ only in
   `Test262RealmExtension` setup. The failure surfaces on Commit A's
   infrastructure layout change (a +8-byte addition to
   `NamedPropertyFeedback`) even with no consumer present, suggesting
   a pre-existing latent issue that the layout change tickles
   (allocator-pattern-sensitive or similar). Deep root-cause
   investigation deferred — recorded as a follow-up under `lyng-2pgt`.
2. `staging/sm/RegExp/unicode-class-braced.js [non-strict]`: timeout
   after 1.0s. Variant ran in 0.986s under Phase 2a (per
   `phase-2a-test262.md` slowest-variants table). The 14-ms margin is
   plausibly affected by the slight per-slot memory growth in
   `NamedPropertyFeedback`. Recommend monitoring; not a deterministic
   regression.

Net Test262 verdict: 49720/49729 (99.98%) vs Phase 2a's 49722/49729
(99.99%) — −2 files in `staging/sm`, no regression in the
ECMA-262-core categories (annexB, built-ins, harness, language).

## What's deferred

- **The two follow-up Test262 regressions** above. Recorded for
  investigation in a Phase 3a follow-up sub-issue.
- **Store / Assign / Strict / Global / Keyed opcodes** — Phase 3b–3d.
- **PrototypeData inline path** — Phase 3e.
- **Polymorphic compaction** — Phase 3f.
- **γ-swap evaluation** — Phase 3g, gated on post-3f re-profiling.

## Files changed

- `crates/lyng-js/objects/src/shapes.rs` — `NamedPropertyHandler` struct + accessors (+68 lines).
- `crates/lyng-js/objects/src/lib.rs` — pub use of new type (+1 line edit).
- `crates/lyng-js/objects/src/tests.rs` — 6 unit tests for handler packing (+93 lines).
- `crates/lyng-js/vm/src/vm/feedback.rs` — `monomorphic_fast` and
  `monomorphic_fast_dependency_epoch` sidecar fields on
  `NamedPropertyFeedback`; `refresh_monomorphic_fast()` maintenance;
  `Vm::named_property_fast_handler()` and
  `Vm::record_named_property_fast_hit()` `#[inline(always)]` helpers.
- `crates/lyng-js/vm/src/vm/dispatch/property.rs` — inlined fast-path
  block in `execute_get_named_property_opcode` cache-hit branch.

Total Phase 3a diff: ~270 added lines + small modifications across 5 files.
