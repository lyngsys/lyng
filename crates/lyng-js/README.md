# Lyng JS

Lyng JS is the repository JavaScript engine. It is an interpreter-first ECMA-262
implementation in Rust with explicit crate ownership across the frontend, compiler,
runtime, builtins, host boundary, CLI, and verification tooling.

## Scope

Lyng JS implements core JavaScript language semantics and the engine substrate needed by
the in-repository Test262 and benchmark tools. The core engine does not provide ECMA-402
Intl, browser APIs, Node APIs, or native-code execution.

## Current State

- `common`, `lexer`, `ast`, `parser`, and `sema` own source text, tokens, arena ASTs,
  parse entrypoints, early errors, scope tables, and binding metadata.
- `compiler` lowers frontend artifacts into immutable bytecode templates held by
  `bytecode`.
- `vm` installs and executes bytecode through an interpreter, register windows, call
  frames, feedback vectors, inline-cache state, and module/evaluation entrypoints.
- `types`, `gc`, `objects`, `env`, `ops`, and `host` define the runtime value model,
  typed handles, allocation/rooting substrate, object operations, environments, jobs,
  realms, agents, host hooks, and shared backing-store coordination.
- `builtins` bootstraps default realms, constructors, prototypes, globals, intrinsic
  tables, builtin descriptor metadata, and native builtin dispatch.
- `cli`, `crates/lyng-js/tests`, `tools/lyng-js-test262`, and `tools/lyng-js-bench`
  provide local entrypoints for evaluation, regression tests, conformance runs, runtime
  reports, and bytecode-density reports.

## Architecture At A Glance

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

The crate tree is organized around ownership boundaries:

- Frontend: `common`, `lexer`, `ast`, `parser`, `sema`
- Runtime and execution: `types`, `gc`, `ops`, `host`, `objects`, `env`, `bytecode`,
  `compiler`, `vm`, `builtins`
- Entry points and verification: `cli`, `tests`, `tools/lyng-js-test262`,
  `tools/lyng-js-bench`

The main engineering constraints are stable crate boundaries, spec-traceable behavior,
minimal dependency growth, explicit ownership of abstract operations, and hot-path
discipline in the VM/runtime layers.

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
- [Engineering Standards](../../docs/lyng-js/engineering-standards.md)
- [Runtime Model](../../docs/lyng-js/runtime-model.md)
- [Bytecode And VM](../../docs/lyng-js/bytecode-and-vm.md)
