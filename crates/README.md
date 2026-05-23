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
- `cli`, `crates/tests`, `tools/lyng-test262`, and `tools/lyng-bench`
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
- Entry points and verification: `cli`, `tests`, `tools/lyng-test262`,
  `tools/lyng-bench`

The main engineering constraints are stable crate boundaries, spec-traceable behavior,
minimal dependency growth, explicit ownership of abstract operations, and hot-path
discipline in the VM/runtime layers.

## Verification

Focused crate tests:

```sh
cargo test -p lyng-parser
cargo test -p lyng-compiler
cargo test -p lyng-vm
cargo test -p lyng-tests
```

Targeted and whole-corpus Test262:

```sh
cargo run --release -p lyng-test262 -- --filter built-ins/Temporal/Instant --report /tmp/lyng-test262-temporal.md -j 4
cargo run --release -p lyng-test262 -- --report /tmp/lyng-test262-report.md -j 12
```

Runtime and bytecode reporting:

```sh
cargo run --release -p lyng-bench -- runtime --report /tmp/lyng-bench.md
cargo run --release -p lyng-bench -- density --report /tmp/lyng-bytecode-density.md
```

Checked-in reports live under [`../../reports/lyng/`](../../reports/lyng/).

## Read Next

- [Docs Index](../../docs/lyng/README.md)
- [Architecture](../../docs/lyng/architecture.md)
- [Engineering Standards](../../docs/lyng/engineering-standards.md)
- [Runtime Model](../../docs/lyng/runtime-model.md)
- [Bytecode And VM](../../docs/lyng/bytecode-and-vm.md)
