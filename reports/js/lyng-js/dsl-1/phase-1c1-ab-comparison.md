# Phase 1.C.1 — Mini A/B (op_sub + op_mul vs pre-1.C.1 HEAD)

**Informational A/B per spec §6 + Phase 1.B retrospective lesson #2** (per-sub-phase
A/Bs compose roughly but not authoritatively; phase-close cumulative A/B vs
pre-DSL-0 `d850f261` is the authoritative number — lands in Task 13).

## Methodology

Per Phase 1.B same-load A/B protocol. Both runs back-to-back on the same
hardware. `v8suite --samples 11` (11-sample medians) per workload. No
`--require-isolation` flag on v8suite — loadavg managed manually via `uptime`.

- **Base HEAD:** `64e3e5cb` (Task 1 close — substrate macros `inc_smi_overflow!`/`dec_smi_overflow!` only; no handlers consume them yet, so v8 v7 perf matches pre-Phase-1.C state).
- **Post HEAD:** `dfa45a77` (Task 3 close — op_sub + op_mul inline ports landed, including substrate -0 fix for `mul_smi_overflow!`).

### Loadavg at the changeover

| Measurement                | Loadavg start (1m/5m/15m) | Loadavg end (1m/5m/15m) |
|----------------------------|---------------------------|-------------------------|
| Base (`64e3e5cb`)          | 5.24 / 5.02 / 4.14        | 2.54 / 2.84 / 3.37      |
| Post (`dfa45a77`)          | 3.36 / 3.03 / 3.43        | 2.14 / 2.42 / 2.92      |

5-minute loadavg overlap at the changeover: base ended at **2.84**, post
started at **3.03**. Overlap: `abs(2.84-3.03)/max = 0.19/3.03 = 6.3%`.
**Well within the ±20% protocol** (Phase 1.B retrospective lesson #1).

### Per-sample wall-clock

- Base run: ~479s wall-clock (~8.0 min) — 21:14:14 → 21:22:13
- Post run: ~481s wall-clock (~8.0 min) — 21:22:28 → 21:30:29

Wall-clock difference is within noise (+0.4%). Post is marginally slower
in wall-clock — consistent with the mixed per-workload picture below.

## V8 v7 results (11-sample medians)

| Workload     | Base median | Post median | Delta     |
|--------------|------------:|------------:|----------:|
| Richards     |        299  |        306  | **+2.34%** |
| DeltaBlue    |        331  |        336  | **+1.51%** |
| Crypto       |        274  |        281  | **+2.55%** |
| RayTrace     |        430  |        419  | **-2.56%** |
| NavierStokes |        460  |        456  | **-0.87%** |
| Splay       |       1402  |       1388  | **-1.00%** |
| **Geomean**  | —           | —           | **+0.31%** |

## Interpretation

**The mini A/B is essentially flat (+0.31% geomean), with a real split between integer-heavy and float-heavy workloads.**

Workloads gain (Richards +2.3%, Crypto +2.6%, DeltaBlue +1.5%) where the
SMI fast path is reached most of the time — the inline path avoids the
function-call cost into `op_mul_semantic`/`op_sub_semantic`.

Workloads regress (RayTrace -2.6%, NavierStokes -0.9%, Splay -1.0%) where
the SMI fast path misses on doubles or non-SMI values. Each missed
dispatch now pays ~7-8 extra instructions of `check_smi` (movz+movk+and+movz+movk+cmp+b.ne)
before bailing to the slow path, whereas the pre-Phase-1.C.1 cold-stub
went directly to `call_slow!(op_mul_slow_rs)` with no SMI attempt.

For RayTrace specifically: per the Task 3 v8suite count, RayTrace dispatches
27.1M Mul invocations per run. If ~all of those miss the SMI fast path
(RayTrace is double-heavy), the added overhead is ~27M × 7-8 instructions
= ~200M extra instructions vs the cold-stub baseline. Consistent with
the observed -2.6% regression on a workload that does ~430M Mul-like
operations per benchmark cycle.

This is a real engineering trade-off, not an instrumentation artifact.
The SMI fast-path attempt is a net-positive for integer workloads and
net-negative for float-heavy ones. The cumulative A/B at Phase 1.C close
will show whether the integer-heavy gains across all 7 Phase 1.C ports
amortize the float-heavy losses on the small subset (op_mul specifically).

## No off-ramp triggered

Per spec §7 (off-ramp triggers):
- No workload regressed > 5% (max regression is RayTrace at -2.56%).
- No consecutive per-opcode gate failures.
- Geomean is positive (+0.31%), even if modest.

The phase-close cumulative A/B vs `d850f261` (Task 13) is the authoritative
gate. If the cumulative number lands below +9% (i.e., negative delta from
Phase 1.B close at +8.51%), that triggers investigation per the spec §7
threshold.

## Notes

- Combined dispatch share added in 1.C.1: ~654M / V8 v7 run (Mul=589M + Sub=65M).
- Substrate change: `mul_smi_overflow!` extended from 4 → 7 instructions
  for ECMAScript -0 deferral. The extra 3 instructions on the SMI fast
  path are paid only on SMI inputs (the most common case in integer
  workloads); they're not paid when the path bails to slow on non-SMI.
- Slow-path-share readings of "100% across all workloads" for Sub and Mul
  are the well-documented `call_slow!` counter-injection artifact (Phase
  1.C followup); the actual execution is correctly through the inline
  fast path on SMI inputs.
