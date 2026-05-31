# Construct Fast Path — Verification & Outcome

**Date:** 2026-05-31
**Branch:** `feat/construct-fast-path` (HEAD `7bc924ec`)
**Spec/plan:** `docs/superpowers/specs/2026-05-30-construct-fast-path-design.md`,
`docs/superpowers/plans/2026-05-30-construct-fast-path.md`

## TL;DR

The Construct fast path was implemented, is **correct and well-tested**, and is
**throughput-neutral (no regression)** on the V8 v7 gate. But it delivers a
**~0% win on RayTrace** — the workload it was designed to attack — because
**every RayTrace constructor is ineligible for the fast path**. The plan's
premise does not hold for *this* RayTrace.

## Correctness

- Full unit suites green: `lyng-vm`, `lyng-tests`, `lyng-objects`, `lyng-env`
  (0 failures). Behavior tests cover fast-path instances, single & double
  `.prototype` reassignment (re-arm), excluded callees (derived/bound/proxy/
  spread), and a constructor returning an object.
- Two-stage review (spec + code-quality) on every task; the zero-read soundness
  invariant (arm-at-cache-time + eager-clear on `.prototype` write, across all
  Rust write paths incl. the store-IC bypass) was adversarially verified.
- Full Test262 conformance (whole corpus, `-j 12`, 59.9s): **100% parity** —
  `annexB` 1086/1086, `built-ins` 23402/23402, `harness` 116/116, `language`
  23640/23640 all 100%; `staging` 1484/1485 with the single miss being
  `staging/sm/TypedArray/set-same-buffer-different-source-target-types.js`
  **timing out at the 1.0s limit under load** (passes cleanly when run isolated
  — a TypedArray test unrelated to construct/`.prototype`, the documented
  `staging/sm` timing-flake class, not a semantics regression). `intl402`
  skipped (ECMA-402 unimplemented, by design). No semantic regression: the
  branch is at 100% in every started category, matching the documented
  baseline.

## Performance — the key result

### In-process time-attribution profile (3 samples, 200µs sampler)

RayTrace `Construct`, before → after:

| Metric | Before | After |
| --- | ---: | ---: |
| Time share | 41.70% | 41.63% |
| Slow share | 99.97% | 99.97% |
| Samples / Mdispatch | 4172.83 | 4224.27 |

Unchanged. (The 99.97% "slow share" is uninformative here: `Construct` is a cold
shim that always exits asm to a Rust handler, so it reads as "slow" whether or
not the Rust fast branch is taken — the profiler cannot distinguish Rust-fast
from Rust-slow. The flat **time share** is the real signal.)

### Controlled v8suite throughput (same machine, back-to-back, 7 samples)

| Workload | main `d97e10d2` | branch `7bc924ec` | Δ |
| --- | ---: | ---: | ---: |
| RayTrace | 463 | 471 | +1.7% (within noise) |
| Richards | 513 | 523 | +1.9% (within noise) |

Flat / no regression. (The committed `bench-v8.json` baseline of 587/458 is from
non-comparable conditions and was disregarded; Richards moving *up* vs it while
RayTrace moved *down* proved it non-comparable.)

## Root cause: every RayTrace constructor is ineligible

RayTrace builds all constructors through Prototype.js's `Class.create()`
(`testdata/js-benchmarks/v8-v7/raytrace.js:33`):

```js
var Class = {
  create: function() {
    return function() {
      this.initialize.apply(this, arguments);   // <-- uses `arguments`
    }
  }
};
```

All 15 constructors (`Vector`, `Color`, `Ray`, `Sphere`, `Camera`, …) share this
one body. It references `arguments`, so `arguments_mode() != None`, and
`ordinary_bytecode_construct_eligibility` returns `None` for **every** RayTrace
construction. The fast branch never fires; the unchanged 41.7% is fully
explained.

The original analysis (`v8-raytrace-profile-2026-05-30.md`) correctly measured
`Construct` at 41.7% of RayTrace, but the design assumed those were "ordinary,
frame-shaped, no-argument-adaptation" constructors. They are the opposite — the
`arguments`/`.apply` adaptation case the fast path explicitly excludes.

Richards is unaffected by design: its `Construct` is 0.15%; its cost is `Call0`
(21%) + `TailCall` (17%) + `Return` (7%).

## What the data now says about the next lever

The broadly-applicable hot cluster on *both* gate workloads is **call/return**,
not construct:

- Richards: `Call0` 21.0% + `TailCall` 16.8% + `Return` 7.1% ≈ **45%**
- RayTrace: `Call2` 13.5% + `ReturnUndefined` 12.7% + `Call1`/`Call0`/`Return` ≈ **30%**

These are the user's originally-planned "Stage 2: inline-asm call path for
environment-free callees," and the data indicates it is the higher-leverage,
workload-general move for the V8 v7 gate — whereas a construct fast path only
helps eligible-constructor workloads the gate does not exercise.

## Status of the implemented work

Correct, reviewed, tested, throughput-neutral. It wires up the previously-dormant
construct IC and *will* benefit eligible-constructor code (simple/modern-class
constructors with no `arguments`/rest, non-derived). It is safe to merge as
groundwork; it simply does not move the V8 v7 gate. Direction decision pending.
