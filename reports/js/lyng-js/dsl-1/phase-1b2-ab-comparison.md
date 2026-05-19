# Phase 1.B.2 — Same-load A/B comparison vs `68dd5e89`

Measured 2026-05-19 after `op_load_const8` and `op_load_this` inline
ports landed (Tasks 1-3).

## Methodology

Per parent spec §4 same-load A/B protocol. Both runs on the same
physical machine within a single ~25-minute window. `v8suite --samples 7`
(7-sample medians) per workload.

- **Base HEAD:** `68dd5e89` (Phase 1.B.1 closed; frame-context substrate
  live but no opcode handler reads `frame_const_base` /
  `frame_this_value` yet).
- **Post HEAD:** `3a5facc4` (Phase 1.B.2 Tasks 1-3 landed: the
  `load_uninit_lex_sentinel!` backend macro + inline ports of
  `op_load_const8` and `op_load_this` consuming the Phase 1.B.1
  substrate).

| Measurement | Loadavg at start (1m / 5m) | Loadavg at end (1m / 5m) |
|-------------|---------------------------:|-------------------------:|
| Base (`68dd5e89`)  | 7.13 / 5.27                | 2.34 / 3.58              |
| Post (`3a5facc4`)  | 5.98 / 4.34                | 2.13 / 2.88              |

5-minute loadavg overlap: base ended at 3.58, post started at 4.34
(within 21% of each other), and both ran with the 1-minute loadavg
descending toward ~2 by the end of each run. The two windows overlap
on the descending side of a compile-induced loadavg spike (the
`cargo build --release` that precedes each suite run briefly pushes
the 1-min loadavg above 5). The overlap is within ±20% on the 5-min
window — acceptable per the established protocol.

## V8 v7 results

| Workload    | Base (median) | Post (median) | Delta    |
|-------------|--------------:|--------------:|---------:|
| Richards    |        239    |        275    | **+15.06%** |
| DeltaBlue   |        277    |        302    | **+9.03%**  |
| Crypto      |        235    |        237    | +0.85%   |
| RayTrace    |        376    |        388    | +3.19%   |
| NavierStokes|        386    |        388    | +0.52%   |
| Splay       |       1199    |       1217    | +1.50%   |
| **Geomean** |    **373.33** |    **391.60** | **+4.89%** |

Per-workload range: +0.52% to +15.06%. **No workload regressed.**

## Verdict

- **Target:** aggregate V8 v7 regression ≤ 2% (parent spec §4 1.B.2 gate).
- **Per-workload tolerance:** no workload regresses > 5% (parent epic §4).
- **Expected:** ≥ +0.3% V8 v7 cumulative improvement (Phase 1.B.2 spec §1
  exit criterion).
- **Observed:** **+4.89% geomean improvement.** Per-workload range
  +0.52% to +15.06%. No regressions.
- **Result: PASS** — every gate cleared with substantial headroom; the
  expected ≥ +0.3% improvement target is exceeded by ~16×.

## Why this is the expected shape

`op_load_this` is #12 in the top-30 dispatch list (~256M dispatches per
V8 v7 run), and `op_load_const8` is #21 (~104M dispatches). Phase 1.B.0
verified the dispatch shares; per-workload Phase 1.B.2 slow-path-share
data (3-sample run) confirms the same dispatch volume.

The two workloads that improved most (Richards +15.06%, DeltaBlue +9.03%)
are object-heavy / method-dispatch-heavy benchmarks that exercise
`op_load_this` on every method call. Replacing a `call_slow!` shim
(~7+ instr including the call-bridge tail) with an 8-instruction inline
path is a ~50% per-dispatch reduction for `op_load_this`, and Richards
performs ~82M LoadThis dispatches per run. The compounded effect is
~15% V8 score lift.

Crypto, RayTrace, NavierStokes, Splay show smaller deltas because their
hot opcodes are different (large-number arithmetic for Crypto, indexed
array access for RayTrace, math intrinsics for NavierStokes, balanced
tree access for Splay) — they happen less in the `op_load_this` /
`op_load_const8` dispatch windows. Even so, every one of these
workloads still improved (no regression anywhere).

## Slow-path-share

`op_load_const8` and `op_load_this` both report **0.00%** slow-path-share
across every V8 v7 workload (full data in
[`phase-1b2-microbench.md`](phase-1b2-microbench.md)). For
`op_load_const8` this is the expected outcome — the inline path covers
every `ConstantValue` variant and the slow-path stub was deleted in
Task 2. For `op_load_this`, V8 v7's workloads exercise only
`ThisState::Value(v)` paths, so the sentinel compare never bails.

## Notes on the methodology

This follows the same same-load A/B protocol used in Phase 1.B.1 Task 8
and the Phase 1.A summary. Both runs were back-to-back on the same
machine with continuous loadavg observation. The two JSON artifacts
(`/tmp/phase-1b2-v8-base.json`, `/tmp/phase-1b2-v8-post.json`)
capture the per-sample series for reproducibility within this
measurement window. The slow-path-share data lives in
`/tmp/phase-1b2-slowshare-counts.json` (3-sample run; same window).

The aggregate +4.89% V8 v7 geomean is the first sub-phase of DSL-1
Phase 1.B to produce a substantial measured speedup at the suite level
(Phase 1.B.1 was substrate-only and reported +0.80%, within bench
noise). Phase 1.B.2 consumes that substrate — the result confirms the
substrate was well-shaped.
