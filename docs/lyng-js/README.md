# Lyng JS Documentation

This directory is the current-state documentation set for Lyng JS, the only JavaScript
engine implementation in this repository.

## Scope

- Core ECMA-262 semantics, including Annex B
- The non-Intl `Temporal` surface exercised by the core Test262 suite
- Embeddable parse, compile, evaluate, host-hook, and conformance-tooling APIs

Out of scope for core completion:

- ECMA-402 / Intl
- Web, Node, or product-specific host APIs
- JIT code generation

## Current Status

- Phase 6 is the active Lyng JS stage.
- Milestones `6A1` through `6G2c` are closed.
- `6H` is the active conformance tail:
  - direct and indirect `eval`
  - `with` and `@@unscopables`
  - proper tail calls
  - Annex B closure
  - remaining Temporal conformance tail
  - optional chaining, RegExp-literal, and final whole-suite burn-down work

## Read This First

1. [architecture.md](architecture.md)
2. [ecma262-completion.md](ecma262-completion.md)
3. [engineering-standards.md](engineering-standards.md)
4. Relevant subsystem notes:
   - [frontend-architecture.md](frontend-architecture.md)
   - [runtime-model.md](runtime-model.md)
   - [runtime-primitives.md](runtime-primitives.md)
   - [runtime-substrate.md](runtime-substrate.md)
   - [shared-memory-and-backing-stores.md](shared-memory-and-backing-stores.md)
   - [bytecode-and-vm.md](bytecode-and-vm.md)
   - [builtin-bootstrap.md](builtin-bootstrap.md)
   - [dynamic-scope-and-eval.md](dynamic-scope-and-eval.md)
   - [temporal-support-matrix.md](temporal-support-matrix.md)

## Current Verification Commands

Focused crate tests:

```sh
cargo test -p lyng-js-parser
cargo test -p lyng-js-compiler
cargo test -p lyng-js-vm
cargo test -p lyng-js-tests
```

Targeted and whole-corpus Test262 runs:

```sh
cargo run --release -p lyng-js-test262 -- --filter built-ins/Temporal/Instant --report /tmp/lyng-js-test262-temporal.md -j 4
cargo run --release -p lyng-js-test262 -- --report /tmp/lyng-js-test262-report.md -j 12
```

Runtime and density reports:

```sh
cargo run --release -p lyng-js-bench -- runtime --report /tmp/lyng-js-bench.md
cargo run --release -p lyng-js-bench -- density --report /tmp/lyng-js-bytecode-density.md
```

The checked-in live report/manifests live under `reports/js/lyng-js/`, including:

- `reports/js/lyng-js/test262.md`
- `reports/js/lyng-js/test262-exclusions.txt`
- `reports/js/lyng-js/bench.md`
- `reports/js/lyng-js/bytecode-density-<arch>.md`

## Historical Context

The old phased planning set no longer lives under `docs/lyng-js/`. Historical phase-by-phase
planning and closeout detail still exists in:

- git history
- checked-in reports under `reports/js/lyng-js/`
- any retained planning or issue history kept outside this directory
