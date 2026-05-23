# Pre-Phase-1.A baseline

Captured 2026-05-18 on Kaylee. 1-min loadavg at v8suite start = 2.21
(microbench / second v8suite run completed with 1-min loadavg between
2.4 and 2.7 — see Notes / Data quality below). Lyng-js commit
`d850f261` (claude/epic-saha-8f0b96 tip).

Captures:

- V8 v7 scores: `/tmp/pre-phase-1a-v8.json`
  (schema `lyng-bench/v8suite/v1`, 7 samples / workload).
- Microbench (markdown — `microbench` has no JSON writer):
  `/tmp/pre-phase-1a-microbench.md` (7 samples / opcode).
- Slow-path-share (V8 v7 in-process opcode counts):
  `/tmp/pre-phase-1a-slowshare.json` — see "Slow-path-share" below
  for the known DSL-0c gap that leaves this file all-zero.

## V8 v7 (median score, 7 samples per workload)

V8 standard score = `100 * reference_µs / mean_µs`. CI95 half-width is
`1.96 * stddev / sqrt(N)` over the per-sample score array (N=7).

| Workload     | Score   | CI95   |
|--------------|---------|--------|
| Richards     |   247.0 | ±1.20  |
| DeltaBlue    |   299.0 | ±2.83  |
| Crypto       |   235.0 | ±4.21  |
| RayTrace     |   392.0 | ±4.93  |
| NavierStokes |   407.0 | ±1.41  |
| Splay        |  1215.0 | ±8.89  |
| **Geomean**  | **387.09** |     |

Per-sample arrays (preserved from `pre-phase-1a-v8.json`):

- Richards: [247, 245, 249, 244, 247, 246, 247]
- DeltaBlue: [306, 305, 305, 298, 298, 298, 299]
- Crypto: [237, 235, 237, 236, 235, 235, 221]
- RayTrace: [375, 388, 392, 394, 391, 393, 393]
- NavierStokes: [410, 407, 406, 404, 407, 405, 407]
- Splay: [1190, 1223, 1213, 1213, 1217, 1228, 1215]

## Microbench (ns/dispatch, 7-sample median)

The Phase 1.A opcode list is LoadUndefined, LoadNull, LoadTrue,
LoadFalse, LoadZero, LoadOne, LoadSmi8, LoadConst8, LoadThis. **None
of the nine opcodes currently have a microbench snippet**, so the
table below shows `—` for them. This means the per-opcode microbench
geomean for the Phase 1.A nine cannot be measured today; the
phase-exit gate must compare against the post-port-with-snippets
captures Task 10 will produce (or use slow-path-share / V8 v7
score-delta as the primary signal, with microbench only on the
opcodes that gain snippets during the phase).

The four hot opcodes that DO have snippets are captured below as
indirect baseline data — they are NOT in the Phase 1.A nine but
share the same dispatch path so a regression in them would also
flag a Phase 1.A problem.

| Opcode               | Samples | Median ns/dispatch | Min    | Max    | CI95   | Snippet ratio |
|----------------------|---------|--------------------|--------|--------|--------|---------------|
| `LoadUndefined`      | —       | no snippet         | —      | —      | —      | —             |
| `LoadNull`           | —       | not in config      | —      | —      | —      | —             |
| `LoadTrue`           | —       | not in config      | —      | —      | —      | —             |
| `LoadFalse`          | —       | not in config      | —      | —      | —      | —             |
| `LoadZero`           | —       | no snippet         | —      | —      | —      | —             |
| `LoadOne`            | —       | not in config      | —      | —      | —      | —             |
| `LoadSmi8`           | —       | no snippet         | —      | —      | —      | —             |
| `LoadConst8`         | —       | no snippet         | —      | —      | —      | —             |
| `LoadThis`           | —       | no snippet         | —      | —      | —      | —             |
| `Move` (reference)   | 7       | 42.71              | 42.47  | 43.61  | ±0.05  | 4 ops/iter    |
| `Add` (reference)    | 7       | 138.53             | 137.97 | 139.37 | ±0.42  | 1 op/iter     |
| `GetNamedProperty` (reference) | 7 | 64.95         | 64.77  | 65.32  | ±0.16  | 3 ops/iter    |
| `Jump` (reference)   | 7       | 113.76             | 112.65 | 114.15 | ±0.39  | 1 op/iter     |

"not in config" means the opcode does not appear in
`tools/lyng-bench/hot-opcodes.toml` (LoadNull / LoadTrue /
LoadFalse / LoadOne fall outside the V8 v7 top-30 dispatch list).
"no snippet" means the opcode is in the config but
`tools/lyng-bench/src/microbench/snippets.rs` does not yet
provide a generator for it.

## Slow-path-share (V8 v7)

**Captured as all-zero** — this is a known DSL-0c gap, not a Phase
1.A blocker.

`crates/vm/src/vm.rs:335-353` documents the cause: when the
α trampoline was removed in DSL-0c C2, the
`maybe_record_opcode_dispatch` call site went with it. The
`opcode-counters` feature still compiles, but `vm.enable_opcode_dispatch_counts()`
+ `run` records nothing because the DSL `dispatch!` tail does not
re-emit the counter hook. The `--count-opcodes` and
`--count-slow-path-share` flags therefore both emit zeroed JSON
today (verified — every per-opcode count and every slow-path-share
field in `/tmp/pre-phase-1a-slowshare.json` is `0`).

| Opcode         | Slow-path-share |
|----------------|-----------------|
| LoadUndefined  | unmeasurable (counter unwired) |
| LoadNull       | unmeasurable (counter unwired) |
| LoadTrue       | unmeasurable (counter unwired) |
| LoadFalse      | unmeasurable (counter unwired) |
| LoadZero       | unmeasurable (counter unwired) |
| LoadOne        | unmeasurable (counter unwired) |
| LoadSmi8       | unmeasurable (counter unwired) |
| LoadConst8     | unmeasurable (counter unwired) |
| LoadThis       | unmeasurable (counter unwired) |

For Phase 1.A's < 20% slow-path-share invariant to be measurable at
the exit gate, the counter hook must be re-wired into the DSL
`dispatch!` tail first. That fix is out of scope for Phase 1.A Task 0
(it should land before Task 10 or the gate's slow-path-share check
needs a substitute signal — recommend raising as a Phase-1.A
prerequisite ticket).

## Notes / Data quality

- **Load conditions.** The machine was running with 1-min loadavg
  oscillating between 2.2 and 3.7 throughout capture (3 active SSH
  sessions; this is a long-uptime workstation, not a quiesced bench
  box). The v8suite subprocess-per-sample harness is somewhat
  resilient to background noise because each sample is isolated,
  but the wider CI95 on Crypto (±4.21, one outlier sample at 221
  vs the 235-237 cluster) and Splay (±8.89) reflect the load. The
  bench tool's `--require-isolation` flag exists on `microbench`
  only — `v8suite` has no isolation gate; comparison against this
  baseline at the Phase 1.A exit gate should re-measure on the same
  machine under similar conditions for a fair delta.
- **Flag adaptation.** The plan-prescribed
  `--require-isolation` flag does not exist on `v8suite`; the
  plan's `--output PATH` flag for `v8suite` is named `--json`; the
  plan's `--opcodes <list>` filter does not exist on `microbench`
  (it runs every opcode in the loaded config). The `--features
  lyng-vm/opcode-counters` cargo flag is also a no-op — the
  feature is baked into `tools/lyng-bench/Cargo.toml`
  unconditionally. All four `cargo run -p lyng-bench --` invocations were
  adapted accordingly; the captured JSON / MD files are still
  valid baseline reference data.
- **Microbench snippet gap.** Adding snippets for the nine Phase
  1.A opcodes to
  `tools/lyng-bench/src/microbench/snippets.rs` is the cleanest
  path to a microbench-driven Phase-1.A exit gate. Without that,
  the Task 10 comparison has to rely on the V8 v7 score-delta
  (across-the-suite signal, less targeted) and on the slow-path-
  share once the counter is re-wired.
