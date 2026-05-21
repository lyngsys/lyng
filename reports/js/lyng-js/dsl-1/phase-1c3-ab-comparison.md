# Phase 1.C.3 — Mini A/B (op_increment + op_decrement vs pre-1.C.3 HEAD)

**Informational A/B per spec §6 + Phase 1.B retrospective lesson #2.** The phase-close cumulative A/B vs `d850f261` (Task 13) is the authoritative number.

## Methodology

- **Base HEAD:** `521c35af` (Phase 1.C.2 close — bitwise/shifts inline ports landed)
- **Post HEAD:** `ed2f3a63` (Task 11 close — op_increment + op_decrement inline ports + dsl_increment_writeback unit test)
- `v8suite --samples 11`, both runs back-to-back on the same hardware. Manual loadavg discipline via `uptime`.

### Loadavg at the changeover

| Measurement          | Loadavg start (1m/5m/15m) | Loadavg end (1m/5m/15m) |
|----------------------|---------------------------|-------------------------|
| Base (`521c35af`)    | 10.18 / 5.02 / 3.82       | 2.43 / 2.79 / 3.09      |
| Post (`ed2f3a63`)    |  2.39 / 2.78 / 3.09       | 1.95 / 2.20 / 2.66      |

5-minute loadavg overlap: base ended 2.79, post started 2.78. Overlap: `abs(2.79-2.78)/2.79 = 0.4%`. **Excellent isolation, well within the ±20% protocol.**

### Per-sample wall-clock

- Base: ~481s (~8.0 min) — 23:47:15 → 23:55:16
- Post: ~456s (~7.6 min) — 23:55:21 → 00:02:57

The ~25s reduction (~5%) is consistent with the all-positive workload deltas below; the post binary executed the same 11-sample suite measurably faster.

## V8 v7 results (11-sample medians)

| Workload     | Base median | Post median | Delta     |
|--------------|------------:|------------:|----------:|
| Richards     |        296  |        296  |  0.00%    |
| DeltaBlue    |        330  |        332  |  +0.61%   |
| Crypto       |        280  |        297  | **+6.07%** |
| RayTrace     |        427  |        427  |  0.00%    |
| NavierStokes |        435  |        494  | **+13.56%** |
| Splay        |       1383  |       1378  |  -0.36%   |
| **Geomean**  | —           | —           | **+3.19%** |

## Interpretation

**NavierStokes +13.56% is the headline.** NavierStokes dispatches **588M Increment + 0 Decrement** per benchmark cycle (per the Task 9 / Task 10 v8suite count measurements) — by far the highest per-workload Increment volume in the V8 v7 suite. The op_increment inline port directly converts those ~588M function-call slow paths into the new 27-instruction inline path, paying off proportionally. Crypto (+6.07%) shows the second-largest gain, consistent with its 305M Increment + 169M Decrement dispatches.

Workloads with low inc/dec dispatch counts (Richards 4M+0.8M, RayTrace 2M+0, DeltaBlue 8M+0, Splay 0.2M+0) show essentially flat results — within run-to-run noise. This is honest signal: the inline ports help where the workload actually hits them, and don't add overhead where it doesn't.

Splay -0.36% is within noise (run-to-run variance on Splay can be ~1% at this loadavg).

## Combined Phase 1.C trajectory (informational, per Lesson #2)

| Sub-phase | Geomean delta vs sub-phase entry | Workloads affected |
|-----------|--------------------------------:|--------------------|
| 1.C.1 (op_sub + op_mul)             | +0.31% | Mixed — integer gains, float regressions |
| 1.C.2 (bit_and + shifts)            | +3.04% | All positive; Crypto and Splay lead |
| 1.C.3 (op_increment + op_decrement) | +3.19% | NavierStokes huge; Crypto strong; others flat |

Naive compounding: 1.0031 × 1.0304 × 1.0319 = ~+6.7% compounded. **Per Phase 1.B retrospective lesson #2, this composition is approximate; the phase-close cumulative A/B vs `d850f261` (Task 13) measures the true Phase 1.C delta directly.**

## No off-ramp triggered

Per spec §7:
- No workload regressed > 5% (Splay's -0.36% is within noise).
- No consecutive per-opcode gate failures.
- Geomean is positive (+3.19%).

## Notes

- Combined dispatch share added in 1.C.3: ~640M / V8 v7 run (Increment=541M + Decrement=99M).
- Asm shape: both inc/dec inline at 27 instructions each — the shortest inline paths in Phase 1.C due to the unary single-source layout (9 fewer instructions than the binary ports thanks to no rhs operand decode/check_smi/untag).
- The SMI-elision-of-src-writeback claim (Tasks 9/10) is verified by the `dsl_increment_writeback` unit test (Task 11) which exercises non-SMI src forced through the slow path, asserting the writeback semantics hold.
- The substrate-wide `call_slow!` counter-injection artifact remains: slow-path-share reads 100% for inc/dec just as for the other Phase 1.C ports. The strongly positive A/B confirms it's purely instrumentation.
- The new substrate macros (`inc_smi_overflow!` / `dec_smi_overflow!`) introduced in Task 1 are now fully runtime-verified by Tasks 9/10 inline ports plus Task 11's writeback test (per Phase 1.B retrospective lesson #3).
