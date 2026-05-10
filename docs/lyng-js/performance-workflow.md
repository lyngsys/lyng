# Lyng JS Performance Workflow

This note defines the Lyng JS performance loop. Test262 remains the first corpus because
it is semantically meaningful and visible in checked-in reports. The external-engine
comparison loop also has a local V8 v7 benchmark corpus for QuickJS/Boa parity work after
a bottleneck needs cross-engine measurement.

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

## Presets

Use named presets for repeatable local loops:

- `smoke`: fastest end-to-end check that the target and report path work.
- `inner-loop`: short local iteration while changing code.
- `baseline`: checked-in default measurement shape.
- `ci-regression`: wider agent or CI regression sweep.
- `profile-target`: one long-running target intended for profiler attachment.

Explicit flags after `--preset` override the preset values. Keep `--json` stable across
before and after runs so the report can render deltas from the previous JSON baseline.

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
  --preset smoke \
  --filter staging/sm/Date/dst-offset-caching-2-of-8 \
  --report /tmp/lyng-js-test262-perf-smoke.md \
  --json /tmp/lyng-js-test262-perf-smoke.json \
  -j 4
```

Use this loop for triage only. One sample is not regression evidence.

## Targeted Resampling

Once the target is plausible, run a stable command with several samples and a persistent
JSON path. The `test262` benchmark reads the previous JSON at the same path before
writing the new report, so repeated runs produce report-only deltas.

```sh
cargo run --release -p lyng-js-bench -- test262 \
  --preset baseline \
  --filter staging/sm/Date/dst-offset-caching-2-of-8 \
  --report /tmp/lyng-js-test262-perf-date-dst.md \
  --json /tmp/lyng-js-test262-perf-date-dst.json \
  --sample-files 2 \
  --timeout-ms 3000 \
  -j 4
```

Use one JSON path per target group. Keep the path stable across before and after runs.
If timing variance is high, increase samples before drawing conclusions.

## Profiler Targets

Use `profile-target` when a target needs stable wall-clock time for profiler attachment.
The preset keeps the sampled target count to one and then runs the selected slowest
variant in a separate profile loop. That loop is intentionally not folded back into the
Markdown or JSON medians.

```sh
cargo build --release -p lyng-js-bench

target/release/lyng-js-bench test262 \
  --preset profile-target \
  --filter staging/sm/Date/dst-offset-caching-2-of-8 \
  --report /tmp/lyng-js-test262-profile.md \
  --json /tmp/lyng-js-test262-profile.json \
  --profile-loop-ms 30000 \
  --print-counters
```

For macOS `sample`, launch the workload directly from the release binary, then attach to
the profile loop:

```sh
target/release/lyng-js-bench test262 \
  --preset profile-target \
  --filter staging/sm/Date/dst-offset-caching-2-of-8 \
  --report /tmp/lyng-js-test262-profile.md \
  --json /tmp/lyng-js-test262-profile.json \
  --profile-loop-ms 30000 \
  --print-counters &
bench_pid=$!
sample "$bench_pid" 10 -file /tmp/lyng-js-test262-profile.sample.txt
wait "$bench_pid"
```

For macOS `xctrace`, record the same release binary command under the Time Profiler
template:

```sh
xcrun xctrace record \
  --template 'Time Profiler' \
  --output /tmp/lyng-js-test262-profile.trace \
  --launch -- \
  target/release/lyng-js-bench test262 \
    --preset profile-target \
    --filter staging/sm/Date/dst-offset-caching-2-of-8 \
    --report /tmp/lyng-js-test262-profile.md \
    --json /tmp/lyng-js-test262-profile.json \
    --profile-loop-ms 30000 \
    --print-counters
```

Use the runtime and density profile presets when Test262 is too broad or the suspected
path is already represented by an in-repo workload:

```sh
target/release/lyng-js-bench runtime \
  --preset profile-target \
  --report /tmp/lyng-js-runtime-profile.md \
  --json /tmp/lyng-js-runtime-profile.json

target/release/lyng-js-bench density \
  --preset profile-target \
  --report /tmp/lyng-js-density-profile.md \
  --json /tmp/lyng-js-density-profile.json
```

When profiler tooling is unavailable, use a plain release timing loop and compare the
stable JSON report path before and after the change:

```sh
for run in 1 2 3 4 5; do
  time target/release/lyng-js-bench test262 \
    --preset inner-loop \
    --filter staging/sm/Date/dst-offset-caching-2-of-8 \
    --report /tmp/lyng-js-test262-profile.md \
    --json /tmp/lyng-js-test262-profile.json \
    --sample-files 1 \
    -j 1
done
```

## External Engine Comparison

Use the external comparison suite when a bottleneck has been reduced to a standalone
script that can run without Test262 harness globals. This is the QuickJS/Boa comparison
loop; it is not a replacement for Test262 diagnostics.

Build the Lyng JS CLI and benchmark runner first:

```sh
cargo build --release -p lyng-js-cli -p lyng-js-bench
```

Run the smoke comparison to check local wiring:

```sh
target/release/lyng-js-bench compare \
  --preset smoke \
  --report /tmp/lyng-js-external-compare-smoke.md \
  --json /tmp/lyng-js-external-compare-smoke.json
```

The default `synthetic` corpus writes three standalone scripts under
`/tmp/lyng-js-bench-compare-scripts`:

- `arithmetic-loop.js`: arithmetic, branches, and loop backedges.
- `array-object-loop.js`: array growth, dense indexed reads, object literals, and named
  property reads.
- `builtin-string-regexp-loop.js`: string case mapping, RegExp replacement, URI decoding,
  and character access.

Defaults use `target/release/lyng-js`, `/opt/homebrew/bin/qjs` when present, and
`/opt/homebrew/bin/boa` when present. Override paths explicitly when needed:

```sh
target/release/lyng-js-bench compare \
  --preset baseline \
  --lyng-js target/release/lyng-js \
  --qjs /opt/homebrew/bin/qjs \
  --boa /opt/homebrew/bin/boa \
  --report reports/js/lyng-js/external-engine-compare.md \
  --json reports/js/lyng-js/external-engine-compare.json
```

Each engine/workload attempt has a timeout. The default is 30000ms, `profile-target`
raises it to 120000ms, and `--timeout-ms 0` disables it for manual profiler sessions.
When an engine fails or times out, the compare run records that status in Markdown and
JSON, keeps any successful samples already collected, and continues with the remaining
engines and workloads.

The report records each engine command, status, wall-clock samples, median/min/max
timings, and a ratio against QuickJS for the same workload when both sides completed.
Treat QuickJS as the primary interpreter baseline and Boa as a Rust-engine reference
point. Evaluate parity by workload family and measured gap; do not expect exact equality
across every script before moving on to JIT work.

The local V8 v7 corpus is vendored under `testdata/js-benchmarks/v8-v7/` with the
upstream notice preserved. Use it when comparing old Octane-style workloads such as
Richards, DeltaBlue, Crypto, RayTrace, EarleyBoyer, RegExp, Splay, and NavierStokes:

```sh
target/release/lyng-js-bench compare \
  --corpus v8-v7 \
  --filter Richards \
  --preset smoke \
  --timeout-ms 30000 \
  --report /tmp/lyng-js-v8-v7-richards-smoke.md \
  --json /tmp/lyng-js-v8-v7-richards-smoke.json
```

Filtered V8 v7 runs generate one standalone script per selected benchmark. Lyng JS uses
`target/release/lyng-js --shell` for this corpus so the benchmark harness can call
`print`; QuickJS keeps `--script`, and Boa runs with its normal command form. V8 v7
reports use benchmark scores as the primary metric: QuickJS score ratio is
`quickjs score / engine score`, so QuickJS is `1.00x` and lower is better for other
engines. Wall-clock columns remain in the report for diagnosing process overhead and
profiling setup.

Use the full V8 v7 suite only for wider checkpoints, not the inner loop:

```sh
target/release/lyng-js-bench compare \
  --corpus v8-v7 \
  --full-suite \
  --preset baseline \
  --report /tmp/lyng-js-v8-v7-full.md \
  --json /tmp/lyng-js-v8-v7-full.json
```

For profiling, regenerate scripts with a longer loop and use the report's generated
`sample` or `xctrace` commands:

```sh
target/release/lyng-js-bench compare \
  --preset profile-target \
  --report /tmp/lyng-js-external-compare-profile.md \
  --json /tmp/lyng-js-external-compare-profile.json
```

When a Test262 bottleneck is too harness-specific for external engines, extract only the
observable hot path into a standalone script:

- Keep the same builtin, opcode family, allocation path, or inline-cache shape that made
  the Test262 case slow.
- Remove `$262`, harness assertions, module-loader assumptions, and non-portable host
  hooks.
- Keep the extracted script small enough to understand and long-running enough for
  profiler attachment.
- Link the extracted workload back to the Test262 command and JSON report in the dcat
  issue so conformance evidence and performance evidence stay connected.

## Wider Sweep

Use a filtered sweep when one family has many candidate files:

```sh
cargo run --release -p lyng-js-bench -- test262 \
  --preset ci-regression \
  --filter built-ins/RegExp/property-escapes/generated \
  --report /tmp/lyng-js-test262-perf-regexp-properties.md \
  --json /tmp/lyng-js-test262-perf-regexp-properties.json \
  --timeout-ms 3000 \
  -j 8
```

Use the whole-corpus performance diagnostic mode only when a broad ranking is needed:

```sh
cargo run --release -p lyng-js-bench -- test262 \
  --preset ci-regression \
  --report /tmp/lyng-js-test262-perf.md \
  --json /tmp/lyng-js-test262-perf.json \
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
- `aggregates[].samples[].timings.script_install_ms`
- `aggregates[].samples[].timings.realm_bootstrap_ms`
- `aggregates[].samples[].timings.extension_install_ms`
- `aggregates[].samples[].timings.global_instantiation_ms`
- `aggregates[].samples[].timings.bytecode_execution_ms`
- `aggregates[].samples[].timings.job_checkpoint_ms`
- `aggregates[].samples[].diagnostics.function_count`
- `aggregates[].samples[].diagnostics.instruction_words`
- `aggregates[].samples[].diagnostics.feedback_slots`
- `aggregates[].samples[].diagnostics.live_feedback_sites`
- `aggregates[].samples[].diagnostics.megamorphic_sites`
- `aggregates[].samples[].diagnostics.runtime_live_bytes_delta`
- `settings.profile_loop_ms`
- `settings.print_counters`

The broad `install_or_load_ms` and `evaluation_ms` fields remain for compatibility with
existing report consumers. For runtime bottleneck selection, prefer the refined setup and
execution fields above plus the rendered dominant phase.

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

## External Corpus Scope

The V8 v7 corpus is intentionally local and dependency-free. Do not rely on an external
checkout such as `../js-engine-benchmark` when adding or running Lyng benchmark
comparisons. See [V8 And Octane Benchmark Plan](v8-octane-benchmark-plan.md) for the
accepted corpus, deferred corpus categories, harness policy, and report contract. Track
future corpus expansion in focused child issues, and keep Test262 as the default
diagnostic loop for conformance-shaped performance work.
