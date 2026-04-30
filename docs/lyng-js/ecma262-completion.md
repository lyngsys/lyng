# Lyng JS ECMA-262 Completion

This is the live Phase 6 status and conformance reference for Lyng JS. It tracks which
sub-milestones are already closed, which shared contracts the remaining work still relies on,
and what remains in the active `6H` tail.

Related notes:

- [Architecture Overview](architecture.md)
- [Runtime Model](runtime-model.md)
- [Runtime Substrate](runtime-substrate.md)
- [Bytecode and VM](bytecode-and-vm.md)
- [Builtin Bootstrap](builtin-bootstrap.md)
- [Dynamic Scope and Eval](dynamic-scope-and-eval.md)
- [Engineering Standards](engineering-standards.md)

## Scope

- In scope:
  - ECMA-262 core semantics, including Annex B
  - the non-Intl `Temporal` surface used by the core Test262 suite
  - embeddable host hooks, module hooks, and conformance-tooling entrypoints required by Lyng JS
- Out of scope:
  - ECMA-402 / Intl
  - web or Node host APIs
  - JIT code generation

## Milestone Status

- Closed:
  - `6A1`: numeric, text, regexp, date, parsing globals, URI globals
  - `6A2`: scoped non-Intl `Temporal`
  - `6B`: arrays and iteration
  - `6C`: classes and instance initialization
  - `6D`: modules
  - `6E1`: generators and suspended execution
  - `6E2`: promises, jobs, and dynamic `import()`
  - `6E3`: async surface closure
  - `6F`: collections and binary data
  - `6G1`: reflection and proxy closure
  - `6G2a`: weak references and finalization
  - `6G2b`: shared memory and atomics
  - `6G2c`: explicit resource management
- Active:
  - `6H`: dynamic scope, proper tail calls, Annex B, and final conformance burn-down

## Active 6H Workstreams

The active `6H` closure work currently centers on these buckets:

- dynamic compilation and `eval`
  - shared dynamic-compilation service and cache contract
  - indirect `eval`
  - direct `eval` environment integration
  - class-element interactions involving `eval`, `new.target`, and `super`
- dynamic scope and control flow
  - `with` and `@@unscopables`
  - proper tail calls
  - script strictness and lexical cleanup
  - optional-chaining execution closure
- compatibility and remaining semantic tails
  - Annex B is green in the filtered report, with only manifest `IsHTMLDDA` host-only exclusions
  - remaining RegExp-literal lowering gaps
  - remaining Temporal conformance tail
  - class/private/computed-name conformance tail
  - arguments-object exactness
  - destructuring/completion-value exactness
  - reference-valued operator and assignment exactness
- final stabilization
  - whole-corpus panic elimination
  - externalized Test262 harness cleanup
  - whole-suite zero-unexplained-failures burn-down

Current owner-cluster triage also centers on:

- dynamic-scope and lexical spillover
- class/object/super and computed-name tail
- shared reference/operator exactness
- shared destructuring/completion tail
- arguments-object exactness
- optional-chaining closure

## Operating Rules

Phase 6 remains broad, but it does not reopen foundational ownership.

- no new semantic owner crate is introduced for Phase 6 closure work
- no second call model, iterator model, module cache, or job queue is introduced
- dynamic-scope, proxy, weak-reference, and shared-memory paths extend the existing runtime
  and GC layers instead of bypassing them
- remaining failures are grouped by owner and root cause, not hidden in a generic skip bucket

## Shared Contracts Still In Force

Several cross-cutting contracts are already shared across the closed milestones and remain
the basis for `6H`:

- one dynamic-compilation service
  - `Function` constructor, indirect `eval`, and direct `eval` reuse the same parse -> sema
    -> compile -> install pipeline
  - caller-sensitive cache behavior belongs in that service, not in ad hoc VM paths
- one iterator model
  - `lyng-js-ops` owns the iterator abstract operations used by `for-of`, spread, array
    helpers, generators, async iteration, collections, and typed-array traversal
- one suspended-execution model
  - generators, async functions, async generators, and promise-job resumption all build on
    the same heap-owned suspended-execution records
- one backing-store model
  - `ArrayBuffer`, `SharedArrayBuffer`, typed arrays, and `DataView` share the same
    cluster-owned backing-store ownership and lifetime rules
- one job-queue model
  - promise jobs, module jobs, async resumption, explicit resource-management cleanup, and
    finalization cleanup ride the same engine-owned queueing layer
- one host boundary
  - conformance tooling, CLI embedding, module loading, time-zone hooks, and shared-memory
    coordination all extend `lyng-js-host` rather than bypassing it

## Crate Ownership

- `lyng-js-builtins`
  - builtin constructors, namespace objects, and prototype methods
  - remaining Annex B and conformance-tail builtin closure
- `lyng-js-compiler`
  - `eval`/`with` lowering, tail-position analysis, optional chaining, and remaining exactness tails
- `lyng-js-vm`
  - execution semantics for dynamic scope, tail calls, and remaining runtime conformance cleanup
- `lyng-js-env`
  - environments, module records, job queues, and shared dynamic-compilation integration
- `lyng-js-objects`
  - exotic objects, proxy-sensitive behavior, private-state storage, and remaining object exactness tails
- `lyng-js-ops`
  - iterator, promise, module, `eval`, `with`, wrapper, and other shared abstract operations
- `lyng-js-gc`
  - weak references, finalization, and shared-backing-store support
- `lyng-js-host`
  - module loading hooks, Temporal clock/time-zone hooks, and shared-memory coordination

## Reporting and Verification

Current conformance and performance tracking uses the generic Lyng JS tools, not the retired
phase-specific binaries:

```sh
cargo run --release -p lyng-js-test262 -- --report reports/js/lyng-js/test262.md -j 12
cargo run --release -p lyng-js-test262 -- --filter annexB --report reports/js/lyng-js/test262-annexb.md -j 4
cargo run --release -p lyng-js-test262 -- --filter built-ins/Temporal --report /tmp/lyng-js-temporal.md -j 4
cargo run --release -p lyng-js-test262 -- --filter language --report /tmp/lyng-js-language.md -j 4
cargo run --release -p lyng-js-bench -- runtime --report reports/js/lyng-js/bench.md
cargo run --release -p lyng-js-bench -- density --report /tmp/lyng-js-bytecode-density.md
```

- Checked-in live reports and manifests live under `reports/js/lyng-js/`.
- Historical phase closeout notes under `reports/js/lyng-js/` remain useful evidence, but they are
  not the live source of truth for current status.
- The exclusion manifest remains reserved for `intl402/*` and explicit host-only cases.
- Remaining skips or failures in core scope stay visible in the reports and current closure
  tracking.

Current Annex B closure is tracked by `reports/js/lyng-js/test262-annexb.md`.
The filtered Annex B run currently has `1307` passed, `0` failed, `0` panicked,
and `70` skipped. The remaining skipped tests are all `IsHTMLDDA` cases, which
require a browser `document.all` compatibility object supplied by the host. The
standalone Lyng JS Test262 harness does not provide that host object, so these
are checked-in manifest path exclusions rather than hidden Annex B language or
builtin implementation gaps. Default-suite skips must come from the manifest,
which remains limited to out-of-scope suites or explicit host-only cases.

## Completion Bar

Phase 6 is only complete when all of the following are true:

- `6H` is closed
- there are no unexplained whole-suite Test262 failures outside the checked-in exclusion manifest
- remaining Temporal misses are either fixed or made explicit as out-of-scope exclusions
- no `6H` closure item has introduced a second runtime owner for dynamic scope, iterator,
  job, backing-store, or host behavior
- the live docs, current closure tracking, and checked-in reports agree on what is complete and
  what still remains
