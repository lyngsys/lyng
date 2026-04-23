# Lyng JS

Lyng JS is the repository JavaScript engine. It is an interpreter-first ECMA-262 implementation in
Rust with explicit crate ownership across the frontend, compiler, runtime, and conformance tooling.

## Scope

In scope:

- core ECMA-262 semantics, including Annex B
- the non-Intl `Temporal` surface exercised by the core Test262 suite
- embeddable parse, compile, evaluate, host-hook, and conformance-tooling APIs

Out of scope for core completion:

- ECMA-402 / Intl
- web, Node, or product-specific host APIs
- JIT code generation

## Current Status

- JS3 is the only in-repo JavaScript engine implementation
- Phase 6 is the active stage
- milestones `6A1` through `6G2c` are closed
- `6H` is the active remaining tail:
  - direct and indirect `eval`
  - `with` and `@@unscopables`
  - proper tail calls
  - Annex B closure
  - remaining Temporal conformance work
  - optional-chaining, RegExp-literal, and final whole-suite burn-down work

## Architecture At A Glance

Execution flows through a layered pipeline:

```text
source text
  -> lexer
  -> parser
  -> sema
  -> compiler
  -> bytecode
  -> vm
  -> builtins / host / cli / external tooling
```

The crate tree is organized around ownership boundaries rather than technical convenience:

- Frontend:
  - `common`, `lexer`, `ast`, `parser`, `sema`
- Runtime and execution:
  - `types`, `gc`, `ops`, `host`, `objects`, `env`, `bytecode`, `compiler`, `vm`, `builtins`
- Entry points and verification:
  - `cli`, `tests`
  - `../../tools/lyng-js-test262`
  - `../../tools/lyng-js-bench`

The main engineering constraints are stable crate boundaries, spec-traceable behavior, minimal
dependency growth, and hot-path discipline in the VM/runtime layers.

## Verification

Focused crate tests:

```sh
cargo test -p lyng-js-parser
cargo test -p lyng-js-compiler
cargo test -p lyng-js-vm
cargo test -p lyng-js-tests
```

Targeted and whole-corpus Test262:

```sh
cargo run --release -p lyng-js-test262 -- --filter built-ins/Temporal/Instant --report /tmp/lyng-js-test262-temporal.md -j 4
cargo run --release -p lyng-js-test262 -- --report /tmp/lyng-js-test262-report.md -j 12
```

Runtime and bytecode reporting:

```sh
cargo run --release -p lyng-js-bench -- runtime --report /tmp/lyng-js-bench.md
cargo run --release -p lyng-js-bench -- density --report /tmp/lyng-js-bytecode-density.md
```

Checked-in reports live under [`../../reports/js/lyng-js/`](../../reports/js/lyng-js/).

## Read Next

- [Docs Index](../../docs/lyng-js/README.md)
- [Architecture](../../docs/lyng-js/architecture.md)
- [ECMA-262 Completion](../../docs/lyng-js/ecma262-completion.md)
- [Engineering Standards](../../docs/lyng-js/engineering-standards.md)
- [Runtime Model](../../docs/lyng-js/runtime-model.md)
- [Bytecode And VM](../../docs/lyng-js/bytecode-and-vm.md)
