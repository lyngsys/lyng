# VM-Internal Time-Attribution Profiler + samply Drill-Down

**Date:** 2026-05-30
**Status:** design — pending implementation plan
**Target:** faster optimize→measure loop for JSC LLInt parity work

## Problem

The active performance target is JSC LLInt parity on the V8 v7 suite (see
`reports/lyng/llint-parity-state-of-engine.md`). The V8 v7 suite is and remains
the agreed first performance gate; modern-JS coverage is a later concern and is
explicitly out of scope here.

The loop today is bottlenecked on **attribution**, in two forms the user named:

1. **Lyng-side time attribution** — *where* does wall-time go in a workload.
2. **JSC delta attribution** — *which mechanism* makes JSC LLInt faster.

This round addresses (1) only. (2) is deferred: the Lyng profiler will reveal
where JSC comparison is actually needed, so building a symmetric cross-engine
capture first risks instrumenting the wrong thing. Until then JSC deltas are
done by hand against existing `capture-llint` / `asm-diff`.

### Why attribution is the bottleneck today

`reports/lyng/v8-raytrace-profile-2026-05-23.md` is the evidence. The flamegraph
pipeline is broken (`cargo-flamegraph` and `flamegraph` both fail collapsing the
macOS trace: `IllFormed(MismatchedEndTag ...)`), so the actual analysis fell
back to running macOS `sample` by hand and **counting recursive stack
appearances** ("`op_assign_named_property_dsl`: 97 appearances"). The engine
already has exact dispatch *counts* (`--count-opcodes`) but no *time* axis, and
the bridge between counts and time is rebuilt by hand every session.

The decisive read from that report — "`AssignNamedProperty` is only 5.57% of
dispatches but fans out into the most expensive machinery" — is exactly the
count-vs-time gap a time axis closes automatically.

## Key Insight: the (opcode × path) dimension already exists as counts

`crates/vm/src/opcode_counts.rs` already maintains, per opcode, three banks in
`DispatchCounters`: `dispatch`, `slow_semantic`, and `slow_safepoint`. The asm
dispatch prologue increments these directly via a fixed `Vm` offset
(`VM_DISPATCH_COUNTERS_PTR_OFFSET = offset_of!(Vm, counters) +
offset_of!(OpcodeCounters, dispatch)`). All of it is gated behind the
`diagnostic-counters` cargo feature (`crates/vm/Cargo.toml:20`, `default = []`).

So this is **not a new instrumentation system**. It is a *time axis* bolted onto
an existing, already-feature-gated counter hook. We add statistical sampling of
the *current opcode*; combined with the existing exact counts it yields both
time share and cost-per-dispatch.

## Hard Constraint: no observation cost when disabled

Observing the engine must not regress the engine. **All** profiler state and
instrumentation rides the existing `diagnostic-counters` feature gate (off by
default). With the feature off:

- the `current_opcode` cell does not exist on `Vm`,
- the asm dispatch prologue emits no current-opcode store,
- the `SamplingProfiler` field, thread, and builder hooks compile out.

The hot path is byte-identical to today's. This is enforced structurally by
`#[cfg(feature = "diagnostic-counters")]`, the same way `counters` is gated —
not by a runtime branch. A determinism test asserts dispatch behavior is
unchanged when the feature is off (build-level; the gated code simply is not
compiled).

## Sampler Mechanism (decided: timer-thread + published cell)

Three options were considered:

- **(A) Timer-thread + published `current_opcode` cell — CHOSEN.** The asm
  prologue (already touching the counter bank under the feature gate)
  additionally stores the current opcode index into a `current_opcode` atomic
  cell at a fixed `Vm` offset — one relaxed store, only in the gated build. A
  background sampler thread wakes every N µs and reads the cell into a
  histogram. The fast/slow axis reuses existing slow-stub entry points: stubs
  OR the index with `SLOW_BIT`, so each sample keys on (opcode, fast/slow).
- **(B) SIGPROF signal-based sampler — rejected.** No per-dispatch store, but
  async-signal-safety constraints, awkward per-thread delivery on macOS, and
  fragile interpreter-state reads from a signal handler. Fights the platform.
- **(C) rdtsc per-dispatch deterministic timing — rejected.** Fully
  deterministic but heavily perturbs the hottest loop and distorts the cache
  behavior central to the slow paths we chase. Measures a perturbed engine.

(A) wins: it slots into the existing gated counter hook, is cross-platform,
reproducible (interval is a knob, not wall-clock-dependent), perturbs uniformly
across opcodes so the *shape* stays faithful, and speaks in opcodes+path — the
units optimization work already reasons in.

## Components

All engine-side components are `#[cfg(feature = "diagnostic-counters")]`.

### 1. `current_opcode` cell + slow-path publish (`crate vm`)

- A cell holding an opcode index plus a `SLOW_BIT` (e.g. `AtomicU16`, or a
  `Cell<u16>` if single-threaded publish + cross-thread read is modeled via a
  dedicated atomic — implementation plan picks the exact type; it must be
  lock-free and readable from the sampler thread). Lives next to / within the
  instrumentation state on `Vm`, following the existing fixed-offset
  asm-binding pattern.
- Dispatch prologue stores the opcode index (one relaxed store) — added beside
  the existing counter increment, under the same `#[cfg]`.
- Slow-stub entry points OR the index with `SLOW_BIT`, reusing the sites that
  already feed `slow_semantic`. No new slow-path plumbing.
- `SLOW_BIT` encoding/decoding has a round-trip unit test.

### 2. `SamplingProfiler` (`crate vm`)

- Owns a background thread, a configurable interval (default ~200 µs), and a
  `[u64; OPCODE_COUNT * 2]` histogram (fast lane + slow lane), plus a total
  sample count.
- Started via a builder hook (`with_sampling_profiler`) parallel to the existing
  `with_opcode_counters`, on `EvaluateScript` / `EvaluateInstalled`. The thread
  starts when `.run()` begins and stops + snapshots on completion.
- Snapshot output: per-opcode sample counts split fast/slow, plus total samples.
- The thread reads the `current_opcode` cell at each tick; no engine-thread
  coordination beyond the single relaxed store/load on the cell.

### 3. samply integration (`tools/lyng-bench`)

- Replaces the broken cargo-flamegraph path. A thin wrapper that runs the target
  script under `samply record` and reports the output profile path plus how to
  open it (Firefox-profiler format).
- Pure tooling; no engine change. This is the function-level microscope for
  splitting a single slow handler's internals once the internal profiler has
  pointed at it.
- If `samply` is not installed, emit a clear actionable message (install hint),
  not a crash.

### 4. `profile` subcommand (`tools/lyng-bench`)

New top-level command alongside `runtime|density|test262|compare|v8suite|...`:

```
lyng-bench profile --filter RayTrace [--samples N] [--interval-us 200] [--samply]
                   [--report <path>] [--json <path>]
```

- Default: builds/uses the `diagnostic-counters`-enabled binary, runs the
  VM-internal sampler over the selected V8 v7 workload(s), emits the report.
- `--samply`: additionally captures a samply profile for drill-down.
- Reuses the existing v8-v7 workload selection / script-generation plumbing
  shared with `v8suite` / `compare`.
- CLI parse tests mirror the existing `cli.rs` test style.

> Implementation note: `profile` requires the `diagnostic-counters` feature in
> the engine build it drives. The plan must specify how the subcommand ensures
> that build (e.g. documented invocation / cargo feature wiring), consistent
> with how `--count-opcodes` runs are produced today.

## Report Format

Markdown + JSON, written under `reports/lyng/`. JSON schema id
`lyng-bench/profile/v1`. One ranked table per workload, sorted by time share:

| Opcode | Time share | Slow share (of its time) | Dispatches | Samples / Mdispatch |
| --- | ---: | ---: | ---: | ---: |

- **Time share** = opcode samples ÷ total samples.
- **Slow share (of its time)** = slow-lane samples ÷ this opcode's samples —
  answers "slow because hot, or slow because its slow path fires."
- **Dispatches** = exact count from existing `DispatchCounters`.
- **Samples / Mdispatch** = samples ÷ (dispatches / 1e6) — cost-per-dispatch
  proxy; this is what surfaces "5.57% of dispatches, dominates time."

Header records: workload, sample interval, total samples, total dispatches,
binary path, feature set. The report is statistical — it documents the interval
and sample count so a reader can judge confidence, and notes that small-share
rows are noise-dominated.

## Testing

- **Unit:** histogram accounting; fast/slow lane keying; `SLOW_BIT` round-trip;
  snapshot + reset semantics.
- **No-regression (build-level):** with `diagnostic-counters` off, the profiler
  code is not compiled (the `#[cfg]` is the guarantee); a test under the feature
  asserts enabling the *counters* without the *sampler* leaves dispatch counts
  unchanged, confirming the sampler adds no counted side effects.
- **Statistical correctness:** a short deterministic script dominated by one
  known hot opcode → assert that opcode ranks top in the histogram, with a
  generous threshold to avoid flakiness.
- **CLI:** `profile` arg parsing tests in the `cli.rs` style; `--samply` toggle;
  default interval.
- **samply wrapper:** graceful "not installed" path test (no panic, actionable
  message).

## Out of Scope (this round)

- **JSC delta attribution / symmetric cross-engine capture** — deferred. Handled
  by hand via existing `capture-llint` / `asm-diff` until the Lyng profiler shows
  where JSC comparison pays off. The `profile/v1` report format is designed so a
  later JSC cost-center column can sit beside it without a reshape.
- **IC hit/miss / megamorphic telemetry** — a separate concern; not selected.
- **Before/after variance harness** — `v8suite` / `compare` already cover
  scoring; not part of this tool.
- **Replacing `--count-opcodes` or `microbench`** — the profiler complements
  them (it consumes the exact counts; microbench stays the isolated per-opcode
  ns source).

## Affected Surfaces

- `crates/vm/Cargo.toml` — no new feature; reuse `diagnostic-counters`.
- `crates/vm/src/opcode_counts.rs` / instrumentation state — `current_opcode`
  cell, `SamplingProfiler`.
- `crates/vm/src/vm.rs` — builder hook `with_sampling_profiler`; gated field.
- asm dispatch prologue / slow-stub entry (vm-dsl-emitted) — gated
  current-opcode store + `SLOW_BIT` publish.
- `tools/lyng-bench/src/cli.rs` — `profile` command + tests.
- `tools/lyng-bench/src/` — new `profile.rs`; samply wrapper.
- `reports/lyng/` — generated profile report (and a refreshed RayTrace profile
  as the first real artifact).
- `tools/lyng-bench/AGENTS.md` / `reports/lyng/llint-parity-state-of-engine.md`
  — document the new command and evidence file.
