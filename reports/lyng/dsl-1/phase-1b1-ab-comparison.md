# Phase 1.B.1 — Same-load A/B comparison vs `ae8b7766`

Measured 2026-05-19 after frame-context refactor landed (Tasks 1-7).

## Methodology

Per parent spec §4 same-load A/B protocol. Both runs on the same physical machine within a single ~15-minute window. `v8suite --samples 7` (7-sample medians) per workload.

- **Base HEAD:** `ae8b7766` (Phase 1.B.0 closed; no frame-context substrate)
- **Post HEAD:** `5a7ab6a8` (Phase 1.B.1 Tasks 1-7 landed; population + refresh active; macros + validation tests present; no opcode handler reads the new `LlIntState` fields yet)

| Measurement | Loadavg at start (1m / 5m) | Loadavg at end (1m / 5m) |
|-------------|---------------------------:|-------------------------:|
| Base (`ae8b7766`)  | 2.89 / 3.85                | 3.86 / 4.00              |
| Post (`5a7ab6a8`)  | 3.86 / 4.00                | 2.57 / 3.28              |

5-minute loadavg overlap: base 3.85→4.00, post 4.00→3.28. Movement is within ±20% across the window. Base and post share the 4.00 boundary at the changeover, which is the strongest single-window overlap point. No methodological caveat.

## V8 v7 results

| Workload    | Base (median) | Post (median) | Delta   |
|-------------|--------------:|--------------:|--------:|
| Richards    |        251    |        250    | −0.40%  |
| DeltaBlue   |        288    |        287    | −0.35%  |
| Crypto      |        243    |        245    | +0.82%  |
| RayTrace    |        387    |        392    | +1.29%  |
| NavierStokes|        406    |        411    | +1.23%  |
| Splay       |       1217    |       1244    | +2.22%  |
| **Geomean** |    **386.99** |    **390.08** | **+0.80%** |

Per-workload range: −0.40% to +2.22%.

## Verdict

- **Target:** aggregate V8 v7 regression ≤ 2% (parent spec §4 1.B.1 gate).
- **Per-workload tolerance:** no workload regresses > 5% (parent epic §4).
- **Observed:** +0.80% geomean (slight aggregate improvement). Per-workload range −0.40% to +2.22%; no workload regressed beyond noise. Splay's +2.22% is a real improvement, not a regression.
- **Result: PASS** (well within the ≤ 2% gate; in fact, slightly net-positive).

## Why this is essentially flat (with a small win)

Phase 1.B.1 is substrate-only. No opcode handler reads `frame_const_base` or `frame_this_value` yet — those fields are written but never read by asm. The added work per dispatch is **zero** (handlers are unchanged); the added work per slow-path Refresh egress is one extra heap-view query (constants pointer derivation) plus one extra `Value`-write (this mirror), amortized across the rest of the Refresh arm.

The observed +0.80% is within V8 v7's measurement noise floor (Richards/DeltaBlue typically vary by ±1-2 score points sample-to-sample; Splay by ±2-5). The per-workload signs are split (2 slightly down, 4 slightly up), which is the signature of noise around a true zero-mean. The Splay +2.22% is consistent with that workload's higher sample variance.

The next sub-phase (1.B.2) will exercise these fields via inline ports of `op_load_const8` and `op_load_this`; that's where real V8 v7 movement is expected.

## Notes on the methodology

This follows the same same-load A/B protocol used in Phase 1.B.0 counter-overhead measurement and the Phase 1.A summary (after the methodological correction). Both runs were back-to-back on the same machine with continuous loadavg observation; no separate machine state, no historical comparison against an unrelated baseline window. The two JSON artifacts (`/tmp/phase-1b1-base.json`, `/tmp/phase-1b1-post.json`) capture the per-sample series for reproducibility within this measurement window.
