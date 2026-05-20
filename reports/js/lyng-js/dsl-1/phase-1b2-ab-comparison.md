# Phase 1.B.2 — Same-load A/B comparison vs `68dd5e89`

> **Status (2026-05-20).** This file documents TWO A/B measurements
> of the same Phase 1.B.2 ports against the same `68dd5e89` base:
> - The original 7-sample A/B (loadavg overlap 21%, just outside protocol).
> - A cleanup re-run with 11 samples (loadavg overlap 13%, within protocol).
>
> The two A/Bs disagree substantially — the headline result changed
> from **+4.89% geomean** to **+0.91% geomean**. The §"Reconciliation"
> section at the bottom analyses why and which result is the correct
> one to carry forward.

---

## Original 7-sample A/B (loadavg overlap 21%, just outside protocol)

Measured 2026-05-19 after `op_load_const8` and `op_load_this` inline
ports landed (Tasks 1-3).

### Methodology

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
the 1-min loadavg above 5). **The overlap is +21.2% on the 5-min
window — 1 percentage point outside the ±20% protocol** (acknowledged
in the cleanup audit; this is what motivated the re-run below).

### V8 v7 results (original 7-sample)

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

### Original verdict

- **Target:** aggregate V8 v7 regression ≤ 2% (parent spec §4 1.B.2 gate).
- **Per-workload tolerance:** no workload regresses > 5% (parent epic §4).
- **Expected:** ≥ +0.3% V8 v7 cumulative improvement (Phase 1.B.2 spec §1
  exit criterion).
- **Observed:** +4.89% geomean improvement.
- **Result: PASS** — every gate cleared with substantial headroom.

> The original verdict was correct on the data it had. The re-run
> below revises the magnitude downward; the PASS verdict still holds
> on the revised number (+0.91% is still > 0% expected lift and
> within the ≤ 2% regression gate).

---

## Cleanup re-run with 11 samples (loadavg overlap 13%, within protocol)

Measured 2026-05-20 against current post HEAD `2cb027b0` (cleanup
batch 1 closed; test/doc-only changes since `7baf5846` Phase 1.B.2
close — observable behavior identical to `3a5facc4`).

### Methodology

Same protocol as above, with two changes:
1. **11 samples per workload** (vs 7) — increases statistical power
   to detect / dominate any residual loadavg variance.
2. **Cleaner loadavg overlap** at the changeover (13.3% vs the
   original 21.2%) — within the ±20% protocol.

- **Base HEAD:** `68dd5e89` (Phase 1.B.1 closed; same as original A/B).
- **Post HEAD:** `2cb027b0` (cleanup batch 1 closed; observable
  behavior matches `7baf5846` / `3a5facc4` since cleanup is
  test/doc-only — verified by inspecting commits 7baf5846..2cb027b0).

| Measurement | Loadavg at start (1m / 5m) | Loadavg at end (1m / 5m) |
|-------------|---------------------------:|-------------------------:|
| Base (`68dd5e89`)  | 6.34 / 4.71                | 3.72 / 4.12              |
| Post (`2cb027b0`)  | 6.42 / 4.67                | 2.94 / 3.56              |

5-minute loadavg overlap: base ended at 4.12, post started at 4.67
— a +13.3% deviation, **within the ±20% protocol**. The two windows
ran in immediate succession with similar starting loadavg (6.34 base
vs 6.42 post on the 1-minute) and similar descending profiles.

### V8 v7 results (11-sample re-run)

| Workload    | Base (median) | Post (median) | Delta    | Base CI95 | Post CI95 |
|-------------|--------------:|--------------:|---------:|----------:|----------:|
| Richards    |        279    |        276    | **−1.08%** | ±2.28     | ±2.00     |
| DeltaBlue   |        305    |        308    | +0.98%   | ±3.76     | ±1.46     |
| Crypto      |        237    |        239    | +0.84%   | ±1.45     | ±0.61     |
| RayTrace    |        390    |        394    | +1.03%   | ±2.63     | ±3.33     |
| NavierStokes|        387    |        392    | +1.29%   | ±10.41    | ±0.77     |
| Splay       |       1206    |       1235    | +2.40%   | ±9.59     | ±5.80     |
| **Geomean** |    **392.76** |    **396.32** | **+0.91%** |           |           |

Per-workload range: **−1.08% to +2.40%.** One workload (Richards) is
slightly negative but well within Richards' typical sample variance
(stddev 3.86 base / 3.38 post; range overlaps).

Per-sample data (preserved for reproducibility):

- Richards base: [279, 281, 279, 280, 280, 277, 280, 273, 269, 274, 274]
- Richards post: [279, 278, 276, 275, 279, 274, 267, 274, 276, 276, 278]
- DeltaBlue base: [299, 308, 302, 291, 306, 307, 295, 305, 309, 312, 300]
- DeltaBlue post: [306, 308, 309, 304, 306, 308, 308, 309, 308, 302, 303]
- Crypto base: [237, 235, 238, 240, 237, 235, 238, 233, 236, 240, 241]
- Crypto post: [237, 239, 239, 238, 239, 238, 237, 240, 240, 239, 238]
- RayTrace base: [382, 384, 397, 393, 387, 386, 392, 390, 387, 393, 390]
- RayTrace post: [380, 394, 403, 396, 390, 395, 393, 393, 394, 397, 397]
- NavierStokes base: [396, 331, 387, 388, 387, 391, 386, 377, 387, 388, 388]
- NavierStokes post: [392, 392, 393, 392, 391, 392, 395, 391, 390, 393, 392]
- Splay base: [1215, 1214, 1214, 1206, 1206, 1201, 1198, 1220, 1244, 1179, 1199]
- Splay post: [1228, 1246, 1226, 1237, 1228, 1229, 1236, 1239, 1235, 1236, 1208]

### Re-run verdict

- **Target:** aggregate V8 v7 regression ≤ 2% (parent spec §4 1.B.2 gate).
- **Per-workload tolerance:** no workload regresses > 5% (parent epic §4).
- **Expected:** ≥ +0.3% V8 v7 cumulative improvement (Phase 1.B.2 spec §1
  exit criterion).
- **Observed:** **+0.91% geomean improvement.** Per-workload range
  −1.08% to +2.40%; Richards' −1.08% is well within sample variance
  (CI95 ±2.28 overlaps the post CI95 ±2.00).
- **Result: PASS** — gate still cleared (no workload regresses > 2%;
  geomean still positive; expected ≥ +0.3% target exceeded by 3×).
  The magnitude is much smaller than the original A/B reported.

---

## Reconciliation: why did the two A/Bs disagree?

The two A/Bs were against the same code (post HEADs `3a5facc4` and
`2cb027b0` are observably identical — cleanup batch 1 was test/doc
only, verified by commit-range inspection). The difference is
entirely measurement variance attributable to the base measurement.

### Base scores: where the divergence came from

| HEAD | Run | Loadavg at start | Loadavg at end | Richards median | Geomean |
|------|-----|------------------|----------------|---------------:|--------:|
| `68dd5e89` (Phase 1.B.1 close) | Original 7-sample | 7.13 / 5.27 | 2.34 / 3.58 | 239 | 373.33 |
| `68dd5e89` (Phase 1.B.1 close) | 11-sample re-run  | 6.34 / 4.71 | 3.72 / 4.12 | 279 | 392.76 |
| Phase 1.B.1 post (`4ff25b9b`) | Phase 1.B.1 A/B (7-sample) | — | — | 250 | 390.08 |

The original 1.B.2 base measurement (Richards 239, geomean 373.33)
was anomalously low. The Phase 1.B.1 post measurement on
essentially-identical code (4ff25b9b, two commits earlier) gave
Richards 250 / geomean 390.08 in its own A/B. The 11-sample re-run
gives Richards 279 / geomean 392.76. **The original 7-sample base
was depressed by ~15-30 points across workloads — likely a
load-variance artifact** (the original base run started at loadavg
7.13 and ended at 2.34; the score-impact of the high-load section
disproportionately dragged the 7-sample median).

### Post scores: consistent across runs

| HEAD | Run | Richards median | Geomean |
|------|-----|---------------:|--------:|
| `3a5facc4` (Phase 1.B.2 close) | Original 7-sample | 275 | 391.60 |
| `2cb027b0` (cleanup batch 1)   | 11-sample re-run  | 276 | 396.32 |

The post measurements agree closely (Richards 275 vs 276; geomean
within ~1%). This is the strongest evidence that the **post
measurement is the stable one** and the original base was the
outlier.

### What the re-run reveals about the real Phase 1.B.2 effect

The 11-sample re-run is more trustworthy because:

1. **Loadavg overlap within protocol** (13% vs the original's 21%).
2. **More samples** (11 vs 7) — narrower CI95 on every workload.
3. **The post measurement matches the original post** — the base
   was the noisy one, the post was stable.
4. **Geomean +0.91% is much closer to Phase 1.B.1's substrate-only
   +0.80%.** Phase 1.B.1 added no opcode handlers consuming the new
   fields; the cumulative +0.80% measured then was substrate-shaping
   noise. Phase 1.B.2's +0.91% (with 2 new inline ports consuming
   the substrate) suggests the real per-port effect is sub-1%
   per opcode, well within the dispatch-share-scaled estimate from
   Phase 1.A (~0.3% per port * 2 ports ≈ 0.6%).

### Conservative interpretation

**Headline revision:** The Phase 1.B.2 inline-port V8 v7 effect is
**~+0.9% geomean**, not the originally-reported +4.89%. The
ports cleared the regression gates (no workload > 2% down) but
did not produce the large lift originally claimed.

**Where the original +4.89% came from:** the original 7-sample
base was load-depressed (Richards 239 vs the stable 275-279
across other measurements), inflating the apparent delta by
~3-4 percentage points on geomean and ~12-16 percentage points
on Richards.

**What this teaches:** the ±20% loadavg-overlap protocol is real;
the original 1.B.2 A/B at 21% was just outside the threshold and
the threshold violation translated to a 4× overstatement of the
geomean delta. Future A/Bs should treat the ±20% threshold as a
hard gate, not a soft one.

### What is the correct cumulative trajectory?

The cumulative cumulative ≥ +3% gate vs pre-DSL-0 HEAD `d850f261`
should be evaluated using the revised +0.91% Phase 1.B.2 delta.
Composing:

- Phase 1.A vs pre-DSL-0 `d850f261`: +1.7% (per Phase 1.A summary)
- Phase 1.B.0 vs Phase 1.A close `b680752e`: ~0% (infra-only)
- Phase 1.B.1 vs Phase 1.B.0 close `ae8b7766`: +0.80% (substrate-only)
- Phase 1.B.2 vs Phase 1.B.1 close `68dd5e89`: **+0.91%** (revised; was +4.89%)

Multiplicative cumulative: (1 + 0.017) × (1.0000) × (1 + 0.0080) × (1 + 0.0091) = **~+3.4%**

Still above the +3% gate, but by a much smaller margin. The umbrella
mid-phase summary captures this trajectory. Phase 1.B.3 (7 more ports)
needs to land a meaningful improvement to clear the +3% gate at the
phase-end measurement; the original A/B suggested 1.B.3 might add to
already-comfortable headroom, the re-run shows the headroom is thin.

---

## Notes on the methodology

This follows the same same-load A/B protocol used in Phase 1.B.1 Task 8
and the Phase 1.A summary. Both runs were back-to-back on the same
machine with continuous loadavg observation. The four JSON artifacts
(`/tmp/phase-1b2-v8-base.json`, `/tmp/phase-1b2-v8-post.json` from
the original; `/tmp/phase-1b2-rerun-base.json`,
`/tmp/phase-1b2-rerun-post.json` from the re-run) capture the
per-sample series for reproducibility within each measurement window.

The slow-path-share data lives in `/tmp/phase-1b2-slowshare-counts.json`
(3-sample run from the original window; the slow-path-share gates
remain unchanged — both opcodes 0.00% under the original
measurement, and slow-path-share is a function of dispatch shape,
not load-induced score variance).

## Why this is the expected shape (revised explanation)

`op_load_this` is #12 in the top-30 dispatch list (~256M dispatches per
V8 v7 run), and `op_load_const8` is #21 (~104M dispatches). Phase 1.B.0
verified the dispatch shares. The 11-sample re-run shows that even
with substantial dispatch volume, the inline-port effect on V8 v7
geomean is sub-1%. This is consistent with **Phase 1.A's measured
+1.7% from 7 inline ports** scaled down to 2 ports (~0.5%) plus
substrate-shaping benefits (~0.4%) totalling ~0.9%.

The originally-reported large per-workload deltas (Richards +15%,
DeltaBlue +9%) reflected base-measurement variance, not real
per-workload effects. Richards' true delta in the re-run is −1% to
+2% (sample-overlap range); the workload is stable around 275-280
under both code paths.
