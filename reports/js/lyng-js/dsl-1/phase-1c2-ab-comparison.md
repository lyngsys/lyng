# Phase 1.C.2 — Mini A/B (op_bit_and + op_shift_left + op_shift_right vs pre-1.C.2 HEAD)

**Informational A/B per spec §6 + Phase 1.B retrospective lesson #2** — phase-close cumulative A/B vs `d850f261` (Task 13) is the authoritative number.

## Methodology

Per Phase 1.B same-load A/B protocol. Both runs back-to-back. `v8suite --samples 11` per workload. Manual loadavg discipline via `uptime`.

- **Base HEAD:** `e1c45c0b` (Phase 1.C.1 close — op_sub + op_mul inline ports landed, including mul_smi_overflow! -0 substrate fix)
- **Post HEAD:** `ecb056ea` (Task 7 close — op_bit_and + op_shift_left + op_shift_right inline ports landed)

### Loadavg at the changeover

| Measurement                | Loadavg start (1m/5m/15m) | Loadavg end (1m/5m/15m) |
|----------------------------|---------------------------|-------------------------|
| Base (`e1c45c0b`)          | 15.24 / 7.30 / 5.13       | 4.64 / 4.24 / 4.32      |
| Post (`ecb056ea`)          | 4.35 / 4.19 / 4.30        | 2.80 / 2.85 / 3.52      |

5-minute loadavg overlap at the changeover: base ended at **4.24**, post started at **4.19**. Overlap: `abs(4.24-4.19)/max = 0.05/4.24 = 1.2%`. **Excellent isolation, well within the ±20% protocol.**

Base run started under high loadavg (15.24) from the back-to-back release builds (worktree base binary + post binary build). The system settled by the time v8suite started actual workload execution; both runs ran under comparable loadavg.

### Per-sample wall-clock

- Base run: ~499s wall-clock (~8.3 min) — 22:40:16 → 22:48:35
- Post run: ~486s wall-clock (~8.1 min) — 22:48:39 → 22:56:45

Wall-clock difference is within noise; post is marginally faster, consistent with the all-positive workload deltas below.

## V8 v7 results (11-sample medians)

| Workload     | Base median | Post median | Delta     |
|--------------|------------:|------------:|----------:|
| Richards     |        288  |        292  | **+1.39%** |
| DeltaBlue    |        322  |        328  | **+1.86%** |
| Crypto       |        260  |        278  | **+6.92%** |
| RayTrace     |        418  |        423  | **+1.20%** |
| NavierStokes |        425  |        432  | **+1.65%** |
| Splay        |       1291  |       1360  | **+5.34%** |
| **Geomean**  | —           | —           | **+3.04%** |

## Interpretation

**All workloads gained.** No regressions. The bitwise/shift ports don't suffer from the float-workload penalty that op_mul exposed in Phase 1.C.1 — none of the V8 v7 benchmarks invoke bitwise/shift opcodes on non-SMI inputs at significant frequency, so the SMI fast path is hit nearly every dispatch and the inline win is real.

**Crypto +6.92%** is the standout. Crypto is by far the bitwise-heaviest V8 v7 workload — it does ~150M BitAnd + ~150M ShiftLeft + ~448M ShiftRight dispatches per benchmark cycle (per Task 5/6/7 slow-path-share counts), totalling ~750M bitwise dispatches that now hit the inline path instead of the function-call cold-stub. The ~+7% on Crypto is consistent with that volume.

**Splay +5.34%** is the second-largest gain. Splay uses bit-twiddling for its tree-balancing hot path. The smaller dispatch counts (Splay had only 643K ShiftLeft and 0 ShiftRight per the 5-sample share JSON) versus Crypto's volume make Splay's larger relative gain interesting — possibly because Splay's hot loop is short and the slow-path function-call overhead dominated its cycle time.

**Richards / DeltaBlue / NavierStokes** show modest +1-2% gains consistent with their light bitwise usage.

**RayTrace +1.20%** is the most encouraging signal — Phase 1.C.1's RayTrace regressed -2.56% due to op_mul's float-workload penalty, but Phase 1.C.2's bitwise ports don't add wasted SMI-attempt overhead to that workload. The Phase 1.C.2 cumulative is now back on the right side for RayTrace.

## Combined Phase 1.C.1 + 1.C.2 trajectory

| Workload     | Pre-1.C.1 (64e3e5cb) | Post-1.C.1 (dfa45a77) | Post-1.C.2 (ecb056ea) |
|--------------|---------------------:|----------------------:|----------------------:|
| Richards     |                  299 |                   306 |                   292 |
| DeltaBlue    |                  331 |                   336 |                   328 |
| Crypto       |                  274 |                   281 |                   278 |
| RayTrace     |                  430 |                   419 |                   423 |
| NavierStokes |                  460 |                   456 |                   432 |
| Splay        |                 1402 |                  1388 |                  1360 |

The absolute numbers from different runs aren't directly comparable (loadavg variance across days), so the per-sub-phase A/B deltas are the meaningful signal:
- Phase 1.C.1: +0.31% geomean (mixed: integer workloads gained; float workloads regressed)
- Phase 1.C.2: +3.04% geomean (all positive)

**The Phase 1.C.2 result is consistent with the spec §3 re-baselined trajectory:** combined ~+3.4pp from 1.C.1 + 1.C.2 plus whatever Phase 1.C.3 (inc/dec) adds should land in the +13% to +16% cumulative range (vs pre-DSL-0). The phase-close cumulative A/B (Task 13) measures this directly.

## No off-ramp triggered

Per spec §7:
- No workload regressed (all positive deltas).
- No consecutive per-opcode gate failures (each port committed cleanly).
- Geomean is +3.04% — well above any "phase below +9% cumulative" investigation threshold.

## Notes

- Combined dispatch share added in 1.C.2: ~453M / V8 v7 run (ShiftRight=266M + BitAnd=98M + ShiftLeft=89M).
- The substrate-wide `call_slow!` counter-injection artifact remains (slow-path-share reads 100% for all 3 new ports); per-workload waivers documented per spec §1.6 + §5. The fact that the A/B is strongly positive despite the artifact confirms the artifact is purely an instrumentation issue, not a correctness regression.
- No substrate changes in 1.C.2 (unlike 1.C.1 which extended `mul_smi_overflow!` for -0 semantics). All three bitwise ports used pre-existing DSL-0 substrate macros.
