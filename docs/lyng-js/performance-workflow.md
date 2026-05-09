# Lyng JS Performance Workflow

This note defines the first Lyng JS performance loop. The initial corpus is Test262
because it is already integrated, semantically meaningful, and visible in checked-in
reports. V8, Octane, or other external benchmark suites are second-phase work after this
loop is stable.

Run commands from the workspace root. Use release builds for measurements. Write
exploratory reports under `/tmp`; refresh checked-in reports only when intentionally
updating verification evidence.

## Loop Shape

Use the narrowest loop that answers the current question:

1. Read the checked-in whole-suite timing table.
2. Run a fast targeted Test262 performance smoke.
3. Resample the target with stable JSON output.
4. If the result is actionable, write or update a focused dcat issue.
5. If the target is too broad for diagnosis, extract a microbenchmark or profiler target.
6. After a fix, rerun the same target and record the delta before widening.

Do not start by adding a new benchmark corpus. Test262 should remain the first source of
truth for runtime bottleneck discovery until the workflow itself stops answering the
question.

## Finding Targets

Start with the slowest checked-in Test262 timings:

```sh
sed -n '/^## Slowest Test Timings/,$p' reports/js/lyng-js/test262.md
```

The current top groups include:

- `staging/sm/Array/toSpliced-dense.js`
- `built-ins/decodeURI/S15.1.3.1_A2.5_T1.js`
- `built-ins/decodeURIComponent/S15.1.3.2_A2.5_T1.js`
- `staging/sm/TypedArray/sort_large_countingsort.js`
- `staging/sm/TypedArray/element-setting-converts-using-ToNumber.js`
- `staging/sm/String/string-upper-lower-mapping.js`
- `built-ins/RegExp/property-escapes/generated/*`
- `staging/sm/Date/dst-offset-caching-*-of-8.js`

Prefer runtime-dominated groups before frontend or harness dominated groups when the goal
is interpreter/runtime performance.

## Fast Smoke

Use a tiny targeted run to check whether a suspected target is still slow and whether the
diagnostic path works:

```sh
cargo run --release -p lyng-js-bench -- test262 \
  --filter staging/sm/Date/dst-offset-caching-2-of-8 \
  --report /tmp/lyng-js-test262-perf-smoke.md \
  --json /tmp/lyng-js-test262-perf-smoke.json \
  --mode hybrid \
  --samples 1 \
  --warmup-samples 0 \
  --sample-files 2 \
  --timeout-ms 3000 \
  -j 4
```

Use this loop for triage only. One sample is not regression evidence.

## Targeted Resampling

Once the target is plausible, run a stable command with several samples and a persistent
JSON path. The `test262` benchmark reads the previous JSON at the same path before
writing the new report, so repeated runs produce report-only deltas.

```sh
cargo run --release -p lyng-js-bench -- test262 \
  --filter staging/sm/Date/dst-offset-caching-2-of-8 \
  --report /tmp/lyng-js-test262-perf-date-dst.md \
  --json /tmp/lyng-js-test262-perf-date-dst.json \
  --mode hybrid \
  --samples 5 \
  --warmup-samples 1 \
  --sample-files 2 \
  --timeout-ms 3000 \
  -j 4
```

Use one JSON path per target group. Keep the path stable across before and after runs.
If timing variance is high, increase samples before drawing conclusions.

## Wider Sweep

Use a filtered sweep when one family has many candidate files:

```sh
cargo run --release -p lyng-js-bench -- test262 \
  --filter built-ins/RegExp/property-escapes/generated \
  --report /tmp/lyng-js-test262-perf-regexp-properties.md \
  --json /tmp/lyng-js-test262-perf-regexp-properties.json \
  --mode hybrid \
  --samples 3 \
  --warmup-samples 1 \
  --sample-files 25 \
  --timeout-ms 3000 \
  -j 8
```

Use the whole-corpus performance diagnostic mode only when a broad ranking is needed:

```sh
cargo run --release -p lyng-js-bench -- test262 \
  --report /tmp/lyng-js-test262-perf.md \
  --json /tmp/lyng-js-test262-perf.json \
  --mode hybrid \
  --samples 3 \
  --warmup-samples 1 \
  --sample-files 25 \
  --timeout-ms 1000 \
  -j 12
```

This scans the selected Test262 variants before resampling the slowest candidates. It is
not an inner loop.

## Full Report Usage

Use the main Test262 runner to verify broad conformance and refresh the slowest timing
table after material performance work:

```sh
cargo run --release -p lyng-js-test262 -- \
  --report reports/js/lyng-js/test262.md \
  -j 12
```

Do not use the whole-suite report as the first measurement for a small change. It is a
guardrail and ranking input, not a tight benchmark loop.

## Reading Reports

The Markdown report is for quick triage. Start with:

- `Sampled Bottlenecks`
- `Median total`
- `Median eval`
- `Dominant phase`
- `Cause hints`

The JSON report is the handoff and regression source. Useful fields are:

- `aggregates[].identity`
- `aggregates[].median_total_ms`
- `aggregates[].delta.median_total_ms`
- `aggregates[].samples[].timings`
- `aggregates[].samples[].diagnostics.function_count`
- `aggregates[].samples[].diagnostics.instruction_words`
- `aggregates[].samples[].diagnostics.feedback_slots`
- `aggregates[].samples[].diagnostics.live_feedback_sites`
- `aggregates[].samples[].diagnostics.megamorphic_sites`
- `aggregates[].samples[].diagnostics.runtime_live_bytes_delta`

Current limitation: Test262 diagnostics expose broad timing buckets. In particular,
`install_or_load` and `evaluation` are not yet separated into install, bootstrap, global
instantiation, bytecode execution, and job checkpoint phases. Treat the dominant phase as
the first subsystem pointer, then use issue `lyng-24ry` to improve timing precision.

## Dcat Handoff Evidence

Before implementing a performance fix, the owning dcat issue should contain enough data
for a fresh agent to reproduce the bottleneck:

- Issue or parent epic id.
- Exact command.
- Date, platform, and whether the binary was a release build.
- Filter, mode, samples, warmup samples, sample files, timeout, and jobs.
- Markdown and JSON report paths.
- Top affected variants with median, min, max, and dominant phase.
- Relevant diagnostics: function count, instruction words, feedback slots, live sites,
  megamorphic sites, and runtime live-byte delta.
- Previous baseline or JSON delta when available.
- Suspected owning subsystem and the next verification command.

Use a focused implementation issue for each bottleneck family. Do not batch unrelated
Date, RegExp, string, typed-array, and URI work into one fix.

## When Test262 Is Not Enough

Stop relying on a Test262 case alone and create a dedicated microbenchmark or profiler
target when:

- The test mixes too many subsystems to isolate a hot path.
- The loop cannot run quickly enough for repeated local iteration.
- Timing variance hides the effect size.
- A profiler needs a longer-running stable workload.
- The bottleneck depends on one builtin, opcode family, allocation path, or inline-cache
  state that can be represented directly.
- The needed measurement is not visible in Test262 diagnostics.

Microbenchmarks should live in `tools/lyng-js-bench` unless there is a stronger ownership
reason. Profiler targets should use the same source as the Test262 finding when possible,
so the link between conformance evidence and performance evidence stays clear.

## Phase Two

V8, Octane, and other external benchmark suites belong after the Test262 loop can already
produce reproducible targets, JSON deltas, and issue-ready evidence. Track that work under
`lyng-2o6e`; do not vendor an external corpus or add benchmark dependencies as part of the
first Test262 performance iteration.
