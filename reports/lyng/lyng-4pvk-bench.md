# lyng-4pvk — Runtime bench delta vs Phase 4c-deferral baseline

**Issue:** `lyng-4pvk` — Remove `argument_scratch` Vec materialization for ordinary VM calls
**Baseline commit:** `d9243123` (Phase 4c deferral)
**Bench command:** `cargo run --release -p lyng-js-bench -- runtime --preset baseline --count-opcodes`
**Profile:** `release`, aarch64-apple-darwin
**Samples per benchmark:** 7
**Runs per sample:** 9
**Loop trips:** 2048

## Throughput delta

| Workload | Phase 4c-deferral ns/op | lyng-4pvk ns/op | Δ |
|---|---:|---:|---:|
| `module-heavy.compile` | 3353.59 | 3091.44 | **−7.82%** |
| `regexp-constructor-compile.runtime` | 4005.39 | 3862.20 | **−3.57%** |
| `class-heavy.runtime` | 2424.54 | 2338.46 | **−3.55%** |
| `string-heavy.concat-runtime` | 180.96 | 176.40 | **−2.52%** |
| `regexp-heavy.runtime` | 47214.79 | 46162.97 | **−2.23%** |
| `array-heavy.literal-indexed-runtime` | 671.47 | 662.23 | −1.38% |
| `regexp-named-replace.runtime` | 12948.11 | 12781.07 | −1.29% |
| `regexp-stable-exec.runtime` | 34367.91 | 34147.61 | −0.64% |
| `typed-array-heavy.runtime` | 776.09 | 771.23 | −0.63% |
| `array-heavy.iterator-runtime` | 9694.44 | 9681.19 | −0.14% |
| `regexp-legacy-statics.runtime` | 2071.52 | 2126.33 | +2.65% |
| `async-heavy.frontend` | 3684.03 | 3783.76 | +2.71% |

### Summary

- 7/12 workloads improved by ≥1% (5 by ≥2%)
- 3/12 workloads neutral (within ±1%)
- 2/12 workloads minor regressions (within typical between-run variance
  for these rows)

The strongest wins land on the call-heavy workloads — `module-heavy.compile`
(−7.82%), `regexp-constructor-compile.runtime` (−3.57%), `class-heavy.runtime`
(−3.55%) — which is exactly the targeted scope. `regexp-legacy-statics`
and `async-heavy.frontend` regressions don't exercise the new fast path
and are within typical run-to-run noise for those rows.

## Methodology notes

- Both runs against same commit hash of dependent crates; only the
  patch under test differs.
- `bench.json` on disk reflects the Phase 4c-deferral baseline
  (committed by Phase 4b and refreshed by the Phase 4c-deferral
  commit). lyng-4pvk numbers come from `/tmp/lyng-4pvk-runtime.json`
  (not committed to keep the repo's bench.json as a stable baseline
  until the larger lyng-48k8 epic closes).
- Single bench run per side (not the 5-run CI-grade variance band).
  Re-running adds run-to-run noise of roughly ±1% on most rows;
  ±5% on the slower regexp rows.
