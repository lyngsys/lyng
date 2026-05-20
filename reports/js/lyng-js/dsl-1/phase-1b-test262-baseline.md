# Test262 baseline at Phase 1.B mid-phase

**Date:** 2026-05-20
**HEAD:** `78e25a6b` (cleanup batch 2 D1 commit — A/B re-run report only;
observable runtime behavior identical to `7baf5846` since all
cleanup-batch commits between `7baf5846` and `78e25a6b` are
test/doc-only).
**Run command:** `cargo run --release -p lyng-js-test262 -- --report
/tmp/phase-1b-test262-baseline.md`
**Duration:** 69.7s wall-clock (12 threads — runner auto-detects
hardware concurrency; the historical pre-DSL-0 baseline used 4 jobs
and clocked 133s on the same hardware).

## Results

| Category | Count |
|----------|------:|
| Candidate files | 53194 |
| Selection-excluded files | 141 |
| Selected files | 53053 |
| Runnable files | 49729 |
| **Passing files** | **49729** |
| **Failing files** | **0** |
| Panicked files | 0 |
| Skipped files (manifest, out-of-scope) | 3324 |
| Selected variant executions | 101853 |
| Runnable variant executions | 95205 |
| **Passing variants** | **95205** |
| **Failing variants** | **0** |
| Skipped variants (intl402 out-of-scope) | 6648 |

Pass rate over runnable files: **100.00%**. Pass rate over all
selected files (which counts skipped intl402 files against the
denominator): **93.73%** (same as the historical baseline; the
denominator includes 3324 intl402 files skipped by manifest).

## Comparison to pre-DSL-0 baseline

| Metric | Pre-DSL-0 (`d850f261`) | Phase 1.B mid (`78e25a6b`) | Delta |
|--------|-----------------------:|---------------------------:|-------|
| Passing files | 49728 | 49729 | **+1** |
| Failing files | 1 | 0 | **−1** |
| Skipped files (intl402) | 3324 | 3324 | 0 |
| Runnable file pass rate | 99.998% | 100.000% | +0.002 pp |
| Variant pass rate | 99.998% (2 fail) | 100.000% (0 fail) | +0.002 pp |

The single pre-DSL-0 failure was in `language/identifiers/` (a
unicode-identifier timeout cluster, per the pre-DSL-0 baseline
report's "Failure Clusters" section: 2 variants of
`language/identifiers/start-unicode-9.0.0.js` timing out at 1.0s).

The current baseline shows **no failures** — the unicode-identifier
timeout is no longer reproducing under the current code path. This
is unrelated to the DSL-1 substrate / opcode-port work (no
identifier-recognition code is on the DSL fast path); the
timeout cluster is sensitive to background load + per-test wall
clock. Reading the runner output, this test still ran but the
1.0s budget was met under current conditions.

## Per-category breakdown (passing files / runnable files)

| Category | Selected | Runnable | Passing | Rate |
|----------|---------:|---------:|--------:|-----:|
| annexB | 1086 | 1086 | 1086 | 100.00% |
| built-ins | 23402 | 23402 | 23402 | 100.00% |
| harness | 116 | 116 | 116 | 100.00% |
| intl402 | 3324 | 0 | 0 | (skipped — out of scope) |
| language | 23640 | 23640 | 23640 | 100.00% |
| staging | 1485 | 1485 | 1485 | 100.00% |

## Purpose

This is the **umbrella-level Test262 baseline for Phase 1.B**.
Phase 1.B.3 (locals + Ldar + LoadEnvSlot inline ports) will land
real opcodes that touch the runtime; the umbrella §4 gate is
"Test262 ≥ baseline". This file is that baseline.

Sub-phases 1.B.0, 1.B.1, 1.B.2 deferred Test262 measurement on the
"no semantic surface touched" argument. That deferral was reasonable
for substrate-only work but obscured the cumulative state. Capturing
now closes the loophole identified in the post-1.B.2 audit.

## Known skip pattern

3324 files in `testdata/test262/test/intl402/` are skipped via the
checked-in manifest rule at
[`reports/js/lyng-js/test262-exclusions.txt`](../test262-exclusions.txt).
Reason: ECMA-402 Intl is out of scope for the active ECMA-262
conformance sweep. This is the same skip pattern present in the
pre-DSL-0 baseline; not Phase-1.B-introduced.

Within the manifest, three Stage-3-below proposal suites are also
selection-excluded (141 files total): ShadowRealm (64),
immutable-arraybuffer (40), await-dictionary (37). These are
selection exclusions at proposal-stage policy, not pass/fail
denominators.

## Next measurement

Phase 1.B.3 closure must:

1. Re-run Test262 at the post-1.B.3 HEAD.
2. Assert passing-file count ≥ **49729** (this baseline's value).
3. Assert variant pass rate ≥ **100.00%** (this baseline's value).
4. Document any new failure in the Phase 1.B.3 summary.

Per the umbrella §4 gate, a regression below this baseline blocks
Phase 1.B.3 closure. The same-load A/B for V8 v7 (the other gate)
is a separate measurement.

## Notes on the measurement

- 12 jobs vs the pre-DSL-0 baseline's 4 jobs: the runner now
  auto-detects hardware concurrency on Apple Silicon (M-series, 12
  performance + efficiency cores). 12 jobs reduces wall time from
  133s to 69.7s without changing the per-test outcome (Test262
  files are independent; parallelism is purely scheduling).
- Per-test timeout is 1.0s, same as the pre-DSL-0 baseline.
- The runner's exclusion manifest is unchanged since pre-DSL-0
  capture; comparison is on equal terms.
- Background loadavg during the run: started at 7.27, peaked
  during the run at ~25 (Test262 is highly parallel; each worker
  spawns a process and consumes wall time). The wall clock is not
  directly comparable to the pre-DSL-0 133s if any background
  scheduling changed; the **pass/fail counts are deterministic per
  the runner output and are the load-bearing comparison**.
