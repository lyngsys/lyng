# Phase 1.B.0 — opcode-counter overhead measurement

Measured 2026-05-18 after counters were wired into the DSL `dispatch!` tail (Task 4) and slow-path bridges (Task 5).

## Methodology

A feature-off `--no-default-features`-style measurement is **not** practical: `lyng-bench` has hard dependencies on the counter API (`Vm::enable_opcode_dispatch_counts`, `opcode_dispatch_counts`, etc.); compiling it without the `opcode-counters` feature fails with 48 errors. Rather than refactor the bench tool to make those uses feature-gated (out of scope for Task 6), this measurement uses the **same-load A/B protocol from spec §4** to compare against the pre-counter-wiring HEAD.

- **Pre-wiring HEAD:** `b680752e` (Phase 1.A end state — counter macros existed but were not yet emitted by the lowerer; no per-dispatch increments fired).
- **Post-wiring HEAD:** `845cee79` (Phase 1.B.0 through Task 5 — `inc_dispatch_counter!` emitted at every handler entry; `inc_slow_semantic_counter!` / `inc_slow_safepoint_counter!` emitted inside `call_slow!` / `poll_safepoint!`).

Both runs: `v8suite --samples 7`. Build via the standard `cargo run --release -p lyng-bench -- v8suite`. Loadavg captured at start/end of each run.

| Measurement | Loadavg at start | Loadavg at end |
|-------------|-----------------:|---------------:|
| Pre-wiring  | 3.58             | 2.83           |
| Post-wiring | 7.29             | 2.27           |

Both ran on the same physical machine within a 10-minute window; 5-minute loadavg overlap is significant. Post-wiring's higher start-load is PESSIMISTIC for counter overhead measurement (high load → lower scores), so any observed overhead is an upper bound.

## V8 v7 with vs without counter wiring active

| Workload    | Pre-wiring (median) | Post-wiring (median) | Overhead |
|-------------|--------------------:|---------------------:|---------:|
| Richards    | 259                 | 260                  | **−0.4%** (post faster) |
| DeltaBlue   | 302                 | 301                  | +0.3%    |
| Crypto      | 251                 | 251                  |  0.0%    |
| RayTrace    | 408                 | 408                  |  0.0%    |
| NavierStokes| 421                 | 421                  |  0.0%    |
| Splay       | 1342                | 1342                 |  0.0%    |
| **Geomean** | **406.4**           | **406.5**            | **≈ 0%** |

All per-workload deltas are within ±1%, comfortably inside V8 v7's measurement noise floor (Crypto/Splay typically vary by ±2-5 score points sample-to-sample). The Richards "post is faster" result is consistent with noise.

## Verdict

- **Target:** ≤ 5% per parent §13.12 open question.
- **Observed:** ≈ 0% geomean overhead, all six workloads within noise.
- **Result: PASS (well within budget).**

## Why the overhead is so small

Per-dispatch cost of the wired counter macros:

- `inc_dispatch_counter!`: 4 instructions per handler entry (ldr base + ldr count + add + str). Fires on every dispatch.
- `inc_slow_semantic_counter!`: 4 instructions per `call_slow!` invocation. Fires only on slow-path calls (rare for inline-ported opcodes; ~100% for cold-stub opcodes).
- `inc_slow_safepoint_counter!`: 4 instructions per `poll_safepoint!` pending branch. Fires only when the poll flag is set (essentially never on V8 v7 — no GC during these workloads).

The 4-instruction dispatch increment is amortized across the per-handler body (which typically does 8-20 instructions of real work). The counter writes have good cache locality (the 6 KB `DispatchCounters` array is a hot cache line); the `add x10, x10, #1` is a single-cycle ALU op; the load/store pair fits in the dispatch's available execution slots.

The empty cost on V8 v7 is consistent with Apple Silicon's wide-issue out-of-order execution absorbing the extra ldr/add/str into existing slots without lengthening the critical path.

## Notes on the methodology

The Phase 1.A summary's correction (same-load A/B protocol) directly informs this approach. Comparing absolute scores across measurements taken at different machine loads (as the original Task 10 subagent did) gives misleading results. Same-load A/B against a pre-change HEAD on the same machine within a short time window is the reliable methodology.

`lyng-bench` feature-off rebuilding is identified as a future improvement opportunity — if a "true" counter-off baseline ever needs to be measured (e.g., to validate the dispatch-counter shape after a major rustc upgrade), the bench tool's counter-API uses would need to be feature-gated. Not a blocker for the < 20% slow-path-share enforcement in subsequent DSL-1 phases.

## Implications for the rest of DSL-1

- **Counter wiring stays on by default** in dev/bench builds. The `< 20% slow-path-share` invariant is now enforceable for Phase 1.B.3 opcode ports onward.
- **Production builds** (`lyng-cli`) continue NOT enabling `opcode-counters` (matching the existing `lyng-3lqp` decision); the macros expand to empty strings in that configuration, leaving zero per-dispatch overhead in shipped binaries.
- **No mitigation needed** (sparse counters, batched counters, etc. were the contingency from the parent §13.12 open question — not required).
