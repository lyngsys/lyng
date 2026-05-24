# State Of The Engine: JSC LLInt Parity

**Date:** 2026-05-24
**Status:** live performance target

The active interpreter performance target is **JSC LLInt parity**. Older
staged score gates and local baseline/target tables are not acceptance
criteria. Use this document, `external-engine-compare.*`, and fresh targeted
`lyng-bench` runs as the reference set for optimization work.

## Target

Parity means Lyng's interpreter should be in the same workload family as JSC
with JIT disabled (`jsc --useJIT=false`) on the V8 v7 workloads. Exact equality
on every benchmark is not required, but persistent multi-x gaps point to engine
architecture work, not benchmark tuning.

## Current Cross-Engine Evidence

The latest checked-in cross-engine report is
[`external-engine-compare.md`](external-engine-compare.md). It measures Lyng,
QuickJS, and JSC LLInt on the same generated V8 v7 scripts.

| Workload | Lyng score | JSC LLInt score | JSC/Lyng gap |
| --- | ---: | ---: | ---: |
| `Richards` | 318 | 1871 | 5.9x |
| `DeltaBlue` | 360 | 1684 | 4.7x |
| `Crypto` | 269 | 2119 | 7.9x |
| `RayTrace` | 427 | 4547 | 10.6x |
| `EarleyBoyer` | 519 | 6084 | 11.7x |
| `RegExp` | 101 | 1071 | 10.6x |
| `Splay` | 1372 | 9893 | 7.2x |
| `NavierStokes` | 448 | 2931 | 6.5x |

These numbers are a direction-setting baseline, not a substitute for fresh
same-machine before/after runs on the workload being optimized.

## Live Benchmark Contract

`lyng-bench v8suite` now reports only:

- benchmark name and source file
- V8 reference time
- raw score samples
- median score
- median microseconds per iteration
- execution errors, if any

The JSON schema is `lyng-bench/v8suite/v2`. It intentionally does not expose
local baseline, target, or gate fields.

## Optimization Direction

Optimize general engine mechanisms that move Lyng toward LLInt behavior:

- Keep hot hit paths inside asm-DSL handlers. A Rust call from a hit path is a
  probe or bridge, not LLInt-equivalent execution.
- Prefer asm-visible feedback and IC metadata over Rust-side enum/object probes
  on hot property, equality, global, call, and arithmetic paths.
- Reduce call/return and completion overhead where profiling shows dispatch and
  frame churn dominating.
- Improve object and value layouts when they remove structural extra loads from
  ordinary LLInt hit paths.
- Preserve ECMA-262 correctness and Test262 evidence while optimizing.

## Current Evidence Files

- [`external-engine-compare.md`](external-engine-compare.md): cross-engine
  Lyng/QuickJS/JSC LLInt comparison.
- [`bench-v8.md`](bench-v8.md): current Lyng-only V8 v7 suite scores.
- [`llint-fast-path-audit-2026-05-23.md`](llint-fast-path-audit-2026-05-23.md):
  terminology and remaining Rust-probe audit.
- [`v8-raytrace-profile-2026-05-23.md`](v8-raytrace-profile-2026-05-23.md):
  property-heavy workload profile.
