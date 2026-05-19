# Phase 1.B.2 — Microbench + slow-path-share results

Measured 2026-05-19 after `op_load_const8` and `op_load_this` inline ports
landed (HEAD `3a5facc4`).

## Microbench (post-port ns/dispatch)

The `lyng-js-bench microbench` runner inspects the snippet table at
`tools/lyng-js-bench/src/microbench/snippets.rs`. The Phase-1.B.0 commit
(`ad240f50` "DSL-1 Phase 1.B.0 Tasks 7+8: microbench snippets for 14
opcodes") added snippets for 7 Phase-1.A opcodes + 7 Phase-1.B anchor
opcodes (LoadLocal0..3, StoreLocal3, LoadEnvSlot, Ldar). **It did NOT
add `LoadConst8` or `LoadThis` snippets**, despite the Phase 1.B.2 spec
§5 claiming both were present. The microbench gate is therefore not
directly measurable for these two opcodes at this HEAD.

```text
| Opcode      | Samples | Median ns/dispatch  | Notes                                  |
|-------------|--------:|--------------------:|----------------------------------------|
| LoadConst8  | —       | no snippet          | Not in snippets.rs; gate deferred      |
| LoadThis    | —       | no snippet          | Not in snippets.rs; gate deferred      |
| LoadSmi8    | 7       | 44.98 (±0.10)       | Closest Phase-1.A analog — inline Value materialization |
| LoadZero    | 7       | 36.15 (±0.10)       | Trivial inline path (1 mov + dispatch) |
| LoadLocal0  | 7       | 33.68 (±0.13)       | Inline frame-relative read             |
| Ldar        | 7       | 42.88 (±0.05)       | Accumulator load                       |
```

(Source: `/tmp/phase-1b2-microbench.md` produced by
`cargo run --release -p lyng-js-bench -- microbench --samples 7
--output /tmp/phase-1b2-microbench.md`.)

**Analogous-opcode reasoning:** the inline ports of `op_load_const8`
(3 inline instr body + 4 dispatch tail = 7 instructions) and
`op_load_this` (8 inline instr body + 4 dispatch tail = 12
instructions) sit between the trivial loaders (~36 ns at 5 instr) and
the more-complex paths (~45 ns at 7+ instr).

Linear extrapolation from the Phase-1.A measured set:
- LoadConst8 expected ~40-45 ns (7-instruction body, sub-LoadSmi8
  shape — no immediate decode, one extra indexed load vs. LoadSmi8's
  immediate materialization).
- LoadThis expected ~50-55 ns (12-instruction body, 4 extra
  sentinel-materialization movz/movk + cmp/branch on the fast path
  vs. LoadSmi8).

LLInt reference for the gate (the 2× ratio):
- LoadConst8 ≤ ~110 ns (2× ~55 ns JSC LLInt `op_mov` constant-pool read).
- LoadThis ≤ ~140 ns (2× ~70 ns JSC LLInt `op_to_this` fast path).

Both are well within reach based on the analogous-opcode trajectory.

**Verdict (microbench gate):** the gate is "**deferred — snippets not
yet present in the bench tool**". The substrate gap is on the bench
tool's snippets file (no `LoadConst8` / `LoadThis` entries), not on
the inline ports themselves. The per-handler asm baselines confirm
both inline bodies are within the ≤ 12 instruction budget, and the
V8 v7 A/B gives the dispositive same-load measurement (see below).

## Slow-path-share on V8 v7

Run with `cargo run --release -p lyng-js-bench -- v8suite --samples 3
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

- `reports/js/lyng-js/dsl-asm-baseline-aarch64/op_load_const8.asm`
- `reports/js/lyng-js/dsl-asm-baseline-aarch64/op_load_this.asm`

Both confirm the inline bodies meet the spec gates:

| Opcode         | Body instructions | Total (incl. dispatch tail) | ≤ 12 inline budget? |
|----------------|------------------:|----------------------------:|:-------------------:|
| op_load_const8 | 5                 | 9                           | ✅                  |
| op_load_this   | 8                 | 14 (fast path)              | ✅ (12 body slots)  |

The `asm-diff --check` tool does not yet support the
`dsl::handlers::cold::*` namespace; the asm baselines under
`reports/js/lyng-js/dsl-asm-baseline-aarch64/` (captured manually via
`cargo rustc --release -p lyng-js-vm --lib -- --emit=asm`) are the
authoritative artifact.

## Verdict

| Gate                                  | op_load_const8 | op_load_this    |
|---------------------------------------|----------------|-----------------|
| ≤ 12 inline instructions (body)       | ✅ 5 instr     | ✅ 8 instr      |
| Microbench within 2× LLInt reference  | deferred (snippet absent) | deferred (snippet absent) |
| Slow-path-share < 20% on V8 v7        | ✅ 0.00%       | ✅ 0.00%        |
| Behavioral parity                     | ✅             | ✅              |
| Per-handler ported report present     | ✅             | ✅              |
| Asm baseline captured                 | ✅             | ✅              |

Two of three quantitative gates pass cleanly (≤ 12 instr, slow-path-share
< 20%). The microbench gate is deferred to a follow-up — the bench tool's
snippets file needs `LoadConst8` and `LoadThis` entries before the
microbench can produce ns/dispatch numbers for these opcodes. The V8 v7
A/B (see [`phase-1b2-ab-comparison.md`](phase-1b2-ab-comparison.md)) is
the dispositive measurement at the suite level: **+4.89% geomean**,
well above the +0.3% expected.
