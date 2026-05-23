# Phase 1.B.2 — Microbench + slow-path-share results

Originally measured 2026-05-19 after `op_load_const8` and `op_load_this`
inline ports landed (HEAD `3a5facc4`). **Snippets for `LoadConst8` and
`LoadThis` were backfilled in DSL-1 Phase 1.B cleanup batch 1 (commit
`922ff5f2`)**; the microbench gate is now verified post-hoc with real
measurements. Section below updated 2026-05-20 to replace the
"deferred" rows.

## Microbench (post-port ns/dispatch)

The `lyng-bench microbench` runner inspects the snippet table at
`tools/lyng-bench/src/microbench/snippets.rs`. The Phase-1.B.0 commit
(`ad240f50` "DSL-1 Phase 1.B.0 Tasks 7+8: microbench snippets for 14
opcodes") added snippets for 7 Phase-1.A opcodes + 7 Phase-1.B anchor
opcodes (LoadLocal0..3, StoreLocal3, LoadEnvSlot, Ldar). **It did NOT
add `LoadConst8` or `LoadThis` snippets**, despite the Phase 1.B.2 spec
§5 claiming both were present. The cleanup-batch-1 backfill closes that
gap.

```text
| Opcode      | Samples | Median ns/dispatch  | Notes                                  |
|-------------|--------:|--------------------:|----------------------------------------|
| LoadConst8  | 7       | 36.34 (±0.09)       | Backfilled snippet — 4× Float64 literals per iter |
| LoadThis    | 7       | 36.52 (±0.14)       | Backfilled snippet — 4× `this` reads per iter (ThisState::Value arm) |
| LoadSmi8    | 7       | 45.07 (±0.09)       | Closest Phase-1.A analog — inline Value materialization |
| LoadZero    | 7       | 36.24 (±0.27)       | Trivial inline path (1 mov + dispatch) |
| LoadLocal0  | 7       | 34.61 (±0.40)       | Inline frame-relative read             |
| Ldar        | 7       | 43.18 (±0.06)       | Accumulator load                       |
```

(Source: `/tmp/cleanup-microbench.md` produced by
`cargo run --release -p lyng-bench -- microbench --samples 7
--output /tmp/cleanup-microbench.md` at HEAD `922ff5f2`.)

**Observed shape vs the analogous-opcode prediction:**

Pre-cleanup predictions (linear extrapolation, since deferred):
- LoadConst8 expected ~40-45 ns; **measured 36.34 ns** — slightly
  better than predicted; the 9-instruction inline body's indexed
  constant-pool load (1 load + 1 indexed load) is faster than
  LoadSmi8's immediate-materialization sequence on Apple Silicon.
- LoadThis expected ~50-55 ns; **measured 36.52 ns** — significantly
  better than predicted. The 4-instruction sentinel-materialization
  was assumed to dominate latency; in practice Apple Silicon's
  wide-issue OOO execution overlaps it with the dispatch tail, so the
  effective added cost is near-zero on the `ThisState::Value(v)` fast
  path the snippet exercises.

LLInt reference for the gate (the 2× ratio):
- LoadConst8 ≤ ~110 ns (2× ~55 ns JSC LLInt `op_mov` constant-pool read).
- LoadThis ≤ ~140 ns (2× ~70 ns JSC LLInt `op_to_this` fast path).

**Verdict (microbench gate):** ✅ both opcodes within 2× LLInt
reference. LoadConst8 at 36.34 ns vs ~110 ns budget (3.0× headroom);
LoadThis at 36.52 ns vs ~140 ns budget (3.8× headroom). The
per-handler asm baselines confirm both inline bodies are within the
≤ 12 instruction body budget, and the V8 v7 A/B gives the dispositive
same-load measurement (see below).

## Slow-path-share on V8 v7

Run with `cargo run --release -p lyng-bench -- v8suite --samples 3
--count-opcodes --count-slow-path-share --json /tmp/phase-1b2-slowshare.json
--counts-json /tmp/phase-1b2-slowshare-counts.json`. The
`--count-slow-path-share` flag joins the per-opcode semantic and
safepoint slow-path entry counts into the same opcode-counters output;
all sites recorded since the Phase 1.B.0 Task 5 wiring landed
(`call_slow!` → `record_semantic_slow_path`).

Aggregate across the full V8 v7 suite (3 samples, 6 workloads):

| Opcode          | Total dispatches | Semantic slow-path entries | Slow-path-share | Within 20% gate? |
|-----------------|-----------------:|---------------------------:|----------------:|:----------------:|
| `op_load_this`  | 239,159,248      | 0                          | 0.000%          | ✅               |
| `op_load_const8`| 102,913,132      | 0                          | 0.000%          | ✅               |

Per-workload breakdown for `op_load_this`:

| Workload     | Dispatches | Semantic SP | Share   |
|--------------|-----------:|------------:|--------:|
| Richards     |  82,426,819 |          0 |  0.00% |
| DeltaBlue    |  55,009,605 |          0 |  0.00% |
| Crypto       |  35,929,858 |          0 |  0.00% |
| RayTrace     |  60,452,179 |          0 |  0.00% |
| NavierStokes |       1,605 |          0 |  0.00% |
| Splay        |   5,339,182 |          0 |  0.00% |

Per-workload breakdown for `op_load_const8`:

| Workload     | Dispatches | Semantic SP | Share   |
|--------------|-----------:|------------:|--------:|
| Richards     |     233,267 |          0 |  0.00% |
| DeltaBlue    |      85,632 |          0 |  0.00% |
| Crypto       |  84,404,055 |          0 |  0.00% |
| RayTrace     |      55,703 |          0 |  0.00% |
| NavierStokes |  10,646,322 |          0 |  0.00% |
| Splay        |   7,488,153 |          0 |  0.00% |

**Both opcodes report 0% slow-path-share across every V8 v7 workload.**

For `op_load_const8` this is the expected outcome — the inline path
handles every `ConstantValue` variant the pre-resolution pipeline
materializes, so no bail condition can fire. The slow-path stub was
deleted in Task 2.

For `op_load_this`, the 0% reflects that V8 v7's workloads exercise
only `ThisState::Value(v)` paths — no derived constructors (TDZ /
Uninitialized) and no arrow functions reading a lexical `this` (the
arrow tests in Phase 1.B.2 Task 3's integration suite explicitly check
this, and `resolve_initial_this_value` resolves lexical `this` to a
concrete `Value` at trampoline entry, not the sentinel). The sentinel
compare on the fast path is therefore "always not equal" for these
workloads. Future workloads (e.g., Test262 with class-heavy fixtures)
may push share above 0%, but the < 20% gate has substantial headroom.

## Asm baselines

The Task 2 + 3 commits captured asm baselines:

- `reports/lyng/dsl-asm-baseline-aarch64/op_load_const8.asm`
- `reports/lyng/dsl-asm-baseline-aarch64/op_load_this.asm`

Both confirm the inline bodies meet the spec gates:

| Opcode         | Body instructions | Total (incl. dispatch tail) | ≤ 12 inline budget? |
|----------------|------------------:|----------------------------:|:-------------------:|
| op_load_const8 | 5                 | 9                           | ✅                  |
| op_load_this   | 8                 | 14 (fast path)              | ✅ (12 body slots)  |

The `asm-diff --check` tool does not yet support the
`dsl::handlers::cold::*` namespace; the asm baselines under
`reports/lyng/dsl-asm-baseline-aarch64/` (captured manually via
`cargo rustc --release -p lyng-vm --lib -- --emit=asm`) are the
authoritative artifact.

## Verdict

| Gate                                  | op_load_const8         | op_load_this            |
|---------------------------------------|------------------------|-------------------------|
| ≤ 12 inline instructions (body)       | ✅ 5 instr             | ✅ 8 instr              |
| Microbench within 2× LLInt reference  | ✅ 36.34 ns (3.0× headroom) | ✅ 36.52 ns (3.8× headroom) |
| Slow-path-share < 20% on V8 v7        | ✅ 0.00%               | ✅ 0.00%                |
| Behavioral parity                     | ✅                     | ✅                      |
| Per-handler ported report present     | ✅                     | ✅                      |
| Asm baseline captured                 | ✅                     | ✅                      |

All three quantitative gates now pass cleanly. The microbench gate
was originally deferred at Phase 1.B.2 closure because the
`LoadConst8` / `LoadThis` snippets were missing from
`tools/lyng-bench/src/microbench/snippets.rs`; the snippets were
backfilled in Phase 1.B cleanup batch 1 (commit `922ff5f2`) and the
gate is verified post-hoc. The V8 v7 A/B (see
[`phase-1b2-ab-comparison.md`](phase-1b2-ab-comparison.md)) is the
dispositive measurement at the suite level: **+4.89% geomean**, well
above the +0.3% expected.
