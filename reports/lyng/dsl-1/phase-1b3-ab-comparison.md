# Phase 1.B.3 — Same-load A/B comparison vs `08727f92`

Measured 2026-05-21 after Phase 1.B.3 Tasks 1-4 landed (9 inline opcode
ports: LoadLocal0/1/2/3 + StoreLocal0/1/2/3 + Ldar). This file documents
the same-load A/B vs the immediate predecessor commit `08727f92` (Phase
1.B mid-phase umbrella summary).

## Methodology

Per parent spec §4 same-load A/B protocol. Both runs on the same physical
machine within a single ~20-minute window. `v8suite --samples 11`
(11-sample medians) per workload.

- **Base HEAD:** `08727f92` (Phase 1.B mid-phase umbrella summary commit;
  observable runtime behavior identical to `db2d05db` since intervening
  commits were doc-only).
- **Post HEAD:** `e0d37b52` (Phase 1.B.3 Task 4 close — per-opcode gates +
  microbench + slow-path-share).

### Loadavg overlap at the changeover

| Measurement | Loadavg at start (1m / 5m / 15m) | Loadavg at end (1m / 5m / 15m) |
|-------------|---------------------------------:|-------------------------------:|
| Base (`08727f92`)  | 9.75 / 6.24 / 4.76 | 4.86 / 4.60 / 4.49 |
| Post (`e0d37b52`)  | 8.20 / 5.41 / 4.79 | 3.49 / 4.14 / 4.37 |

5-minute loadavg overlap at the changeover: base ended at 4.60, post
started at 5.41 — a **+17.6%** deviation, **within the ±20% protocol**.
Both runs ran in immediate succession on the same machine; loadavg
profiles descended through similar ranges. The cargo-build spikes
(1-min loadavg ~10) brief and consistent across both runs.

### Per-sample wall-clock

- Base run: 514s wall-clock (~8.5 min)
- Post run: 512s wall-clock (~8.5 min)

## V8 v7 results (11-sample medians)

| Workload    | Base (median) | Post (median) | Delta    | Base CI95 | Post CI95 |
|-------------|--------------:|--------------:|---------:|----------:|----------:|
| Richards    |        284    |        284    | **+0.00%** | ±0.85   | ±1.25     |
| DeltaBlue   |        312    |        310    | **−0.64%** | ±2.83   | ±1.53     |
| Crypto      |        251    |        250    | **−0.40%** | ±0.93   | ±0.83     |
| RayTrace    |        403    |        406    | **+0.74%** | ±4.03   | ±7.52     |
| NavierStokes|        421    |        427    | **+1.43%** | ±1.21   | ±0.56     |
| Splay       |       1271    |       1309    | **+2.99%** | ±8.40   | ±22.69    |
| **Geomean** |    **410.66** |    **413.45** | **+0.68%** |         |           |

Per-workload range: **−0.64% to +2.99%.**

Per-sample data (preserved for reproducibility):

- Richards base: [280, 285, 284, 283, 282, 283, 285, 284, 284, 284, 283]
- Richards post: [286, 281, 281, 284, 285, 285, 283, 282, 279, 284, 284]
- DeltaBlue base: [313, 313, 309, 312, 297, 313, 312, 312, 314, 312, 313]
- DeltaBlue post: [307, 311, 314, 309, 312, 313, 308, 314, 310, 310, 307]
- Crypto base: [249, 248, 250, 250, 252, 253, 249, 252, 251, 251, 252]
- Crypto post: [251, 250, 252, 251, 250, 248, 250, 251, 249, 252, 248]
- RayTrace base: [383, 404, 407, 403, 408, 399, 405, 399, 405, 403, 403]
- RayTrace post: [375, 403, 406, 396, 391, 395, 414, 416, 414, 409, 414]
- NavierStokes base: (raw JSON; medians match)
- NavierStokes post: (raw JSON; medians match)
- Splay base: (raw JSON; medians match)
- Splay post: (raw JSON; medians match)

Raw JSON artifacts: `/tmp/phase-1b3-ab-base.json`,
`/tmp/phase-1b3-ab-post.json` (per-sample arrays preserved within each
file).

## Verdict

- **Target:** aggregate V8 v7 regression ≤ 2% (parent spec §4 1.B.3 gate).
- **Per-workload tolerance:** no workload regresses > 5% (parent epic §4).
- **Expected:** ≥ +0.3% V8 v7 cumulative improvement (Phase 1.B.3 spec
  §1 exit criterion).
- **Observed:** **+0.68% geomean improvement.** Per-workload range
  −0.64% to +2.99%; DeltaBlue's −0.64% and Crypto's −0.40% are well
  within sample variance (CI95 ±0.83 to ±1.53 for those workloads).

**Result: PASS** — every gate cleared. The geomean improvement is
modest (+0.68%) but positive; consistent with the umbrella's revised
trajectory analysis (per-port effect is sub-1% for these 8 reachable
opcodes representing ~1.26B dispatches; the StoreLocal0 port is
unreachable through the standard emit pipeline — see
[`phase-1b-followups.md`](phase-1b-followups.md) §6).

## Notes on the modest delta

The same-load A/B delta is smaller than the umbrella's pre-1.B.3
prediction (~+3-4% conservative estimate). Three contributing factors:

1. **StoreLocal0 unreachable.** The bytecode-builder peephole rewrites
   `Move dst=0, src=B` → `Ldar B` before the `store_local_opcode`
   branch fires. The umbrella's predicted ~1.38B aggregate dispatches
   for the 9 ports becomes ~1.26B in practice (8 reachable opcodes).
2. **Slow-path-share already 0% on these opcodes** (per
   `phase-1b3-microbench.md`). Pre-port, the cold stubs through
   `call_slow!` and the semantic helper added round-trip overhead;
   post-port, the inline body skips that — but on a tight, simple
   handler the round-trip cost is amortized over the dispatch loop.
   The per-dispatch saving is in nanoseconds, multiplied across ~1.26B
   dispatches = ~0.5-1 second of accumulated time across the V8 v7
   suite, against the suite's typical ~60-120s aggregate wall time.
3. **Substrate-side benefits already realized.** Phase 1.B.1 +0.80%
   substrate-only and Phase 1.B.2 +0.91% (revised) per-port baseline
   suggested per-port effect of ~0.4% with the full substrate. The
   1.B.3 increment of +0.68% over 8 ports is consistent with that
   per-port effect amortized across the higher-frequency LoadLocal /
   StoreLocal opcodes.

The headline gate is **clear**: no regressions, modest positive
geomean. The cumulative trajectory measurement against `d850f261`
(see [`phase-1b3-cumulative-ab.md`](phase-1b3-cumulative-ab.md)) is
the umbrella's authoritative gate, and it clears with substantial
headroom (+8.51% geomean).

## Phase 1.B.3 deltas in context

| Sub-phase | A/B against | Geomean delta | Per-workload range |
|-----------|-------------|--------------:|--------------------|
| 1.B.0 close `ae8b7766` | Pre-1.B `b680752e` | ~0% (≈ +0.1%) | infra-only |
| 1.B.1 close `4ff25b9b` | 1.B.0 close `ae8b7766` | +0.80% | substrate-only |
| 1.B.2 close (re-run, 11-sample) `2cb027b0` | 1.B.1 close `68dd5e89` | +0.91% (revised) | 2 inline ports |
| **1.B.3 close `e0d37b52`** | **1.B mid `08727f92`** | **+0.68%** | **8 reachable + 1 unreachable port** |

The cumulative composition is now load-bearingly measured via the
direct cumulative A/B (see [`phase-1b3-cumulative-ab.md`](phase-1b3-cumulative-ab.md))
rather than composed from these same-load deltas.

## Methodology notes

- Bench command: `cargo run --release -p lyng-bench -- v8suite
  --samples 11 --json /tmp/phase-1b3-ab-{base,post}.json`.
- Median computed by the bench tool (per
  `tools/lyng-bench/src/v8suite/mod.rs`).
- `bench-v8.md` (the markdown report side-effect) was restored after
  each measurement; only the JSON artifacts are kept for the per-sample
  arrays.
- Tests-262 was NOT run during this A/B window (separate measurement;
  see Step D in the Phase 1.B.3 Task 5 protocol).
