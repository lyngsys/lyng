# V8 And Octane Benchmark Plan

This note records the `lyng-2o6e` decision for second-phase V8/Octane-style
benchmarking. Test262 remains the first performance corpus for conformance-shaped
bottlenecks. V8/Octane-style workloads are comparison targets for standalone runtime
throughput after a suspected bottleneck can run outside the Test262 harness.

## Recommendation

Use the local V8 v7 corpus as the first and only approved Octane-style corpus:

- `Richards`
- `DeltaBlue`
- `Crypto`
- `RayTrace`
- `EarleyBoyer`
- `RegExp`
- `Splay`
- `NavierStokes`

These workloads are pure JavaScript, run without browser or Node host APIs, and already
fit the external-engine comparison path in `tools/lyng-js-bench compare`. Keep filtered
per-benchmark runs as the default inner loop. Use the full V8 v7 suite only for wider
checkpoints because full-suite scoring is too coarse and slow for day-to-day diagnosis.

Do not add another external benchmark corpus directly to the default workflow. Any
additional Octane, JetStream, Kraken, SunSpider, browser, Node, or library benchmark
subset needs a focused dcat issue that states why the current Test262, runtime, density,
synthetic compare, and V8 v7 coverage is insufficient.

## Corpus Classification

Accepted strict ECMA-262 workloads:

- Pure JavaScript algorithm/data-structure workloads that need only the default language
  runtime plus the benchmark harness callback.
- Standard builtin stress workloads such as `RegExp`, strings, arrays, objects, numbers,
  functions, closures, and ordinary property access.
- Deterministic standalone scripts that can run under Lyng JS, QuickJS, and Boa with no
  per-engine source changes.

Deferred standalone-but-large workloads:

- Generated-code or application-sized Octane workloads such as compilers, emulators,
  compression libraries, physics libraries, or PDF processing.
- Workloads that are ECMA-262-only after bundling but are large enough to hide the
  owning bottleneck without a smaller extracted target.
- Workloads with significant parse/compile time where runtime parity is not the question
  being asked.

Rejected unless a separate issue expands Lyng's benchmark scope:

- Browser workloads requiring DOM, canvas, fetch, timers, Web APIs, or event-loop
  semantics outside the current Lyng JS host contract.
- Node workloads requiring CommonJS, `process`, filesystem, buffers, streams, or package
  resolution.
- ECMA-402 `Intl` workloads. Lyng JS explicitly keeps Intl outside the current ECMA-262
  implementation scope.
- Benchmarks that require native extensions, shell commands, network access, or
  benchmark-specific host globals beyond the controlled `--shell` print path.

## Local Fixture And License Policy

The approved V8 v7 fixtures live under `testdata/js-benchmarks/v8-v7/`. They are checked
in so Lyng benchmark tooling does not depend on `../js-engine-benchmark`, a network
checkout, or any mutable external project. The copied files retain the V8 project's
BSD-style copyright and redistribution header, and `NOTICE.md` explains the local copy.

Future fixture additions must follow the same rule:

- Keep source fixtures in `testdata/js-benchmarks/<corpus>/`.
- Preserve upstream copyright and license text in source files or an adjacent notice.
- Avoid new Rust or JavaScript package dependencies unless the issue explicitly approves
  the dependency and explains why vendored source is not enough.
- Never make a new external corpus the default comparison corpus without a separate
  reviewable change.

## Harness Requirements

`tools/lyng-js-bench compare` owns external-engine benchmark wiring. It should keep doing
all of the following:

- Generate standalone scripts under the configured scripts directory.
- Support per-benchmark filtering for inner-loop runs.
- Keep optional full-suite mode available but non-default.
- Run Lyng JS as `target/release/lyng-js --shell` for V8 v7 so `print(...)` is available.
- Run QuickJS with its explicit `--script` argument.
- Run Boa with its normal command form.
- Record the exact command vector in Markdown and JSON so results are reproducible.

Benchmark sources should not encode engine-specific branches beyond portable output
fallbacks such as `print` followed by `console.log`.

## Report Contract

External comparison reports should stay aligned with
`reports/js/lyng-js/external-engine-compare.*`:

- Markdown is the human triage artifact.
- JSON is the machine-readable baseline and handoff artifact.
- Settings include corpus, filter, full-suite mode, samples, warmup samples, scripts
  directory, engine executable paths, and comparison policy.
- Results include workload name, category, script path, engine, command, metric kind,
  wall-clock samples, score samples when present, medians, min/max timing, and QuickJS
  ratio.
- QuickJS remains the primary interpreter baseline. Boa remains a Rust-engine reference
  point.
- Wall-time workloads use `engine median ms / quickjs median ms`.
- V8 v7 score workloads use `quickjs score / engine score`; QuickJS is `1.00x`, and lower
  ratios are better for other engines.

## Follow-Up Criteria

Create a new child issue before expanding beyond V8 v7 when at least one of these is
true:

- A Test262 bottleneck is optimized enough that V8 v7 no longer exercises the relevant
  runtime path.
- The team needs application-sized parse/compile/load measurements instead of isolated
  runtime throughput.
- A candidate corpus requires new host APIs, module loading, Intl, Node compatibility, or
  browser behavior.
- A benchmark cannot be represented as a deterministic standalone script under Lyng JS,
  QuickJS, and Boa.

That child issue should include the candidate suite, license and dependency assessment,
required harness work, expected report fields, and a smoke command that a fresh agent can
run before implementation starts.
