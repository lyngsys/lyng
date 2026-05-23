# Lyng

Lyng is a JavaScript engine in Rust, distributed as a Cargo workspace of small,
single-responsibility crates (frontend, compiler, runtime substrate, builtins,
CLI) plus the verification tooling around it.

## What this is

An interpreter-first ECMA-262 implementation, grounded in spec text and verified
against the public Test262 conformance suite — tens of thousands of cases
covering language and built-in semantics.

Lyng has no JIT, no native code execution, and no browser or Node APIs. The core
engine implements language semantics and the runtime substrate needed to host
them.

## Current state

As of May 2026, Lyng passes 100% of Test262 in every category except `intl402`,
which has not been started. Current work is on runtime performance.

| Category | Selected files | Runnable files | Pass | Fail | Skip | Panic | Rate |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `annexB` | `1086` | `1086` | `1086` | `0` | `0` | `0` | `100.00%` |
| `built-ins` | `23388` | `23388` | `23388` | `0` | `0` | `0` | `100.00%` |
| `harness` | `116` | `116` | `116` | `0` | `0` | `0` | `100.00%` |
| `intl402` | `3323` | `0` | `0` | `0` | `3323` | `0` | `0.00%` |
| `language` | `23632` | `23632` | `23632` | `0` | `0` | `0` | `100.00%` |
| `staging` | `1484` | `1484` | `1484` | `0` | `0` | `0` | `100.00%` |

`intl402` entries report as Skip because ECMA-402 Intl is unimplemented.

## Workspace shape

- `crates/`: engine crates, integration tests, runtime/compiler implementation
- `crates/vm-dsl/`: proc-macro substrate for the asm-DSL interpreter
- `tools/`: Test262 runner, benchmark/runtime-report tooling, DSL codegen

## Read next

- [Engine overview](crates/README.md)
- [Docs index](docs/lyng/README.md)
- [Repo conventions](AGENTS.md)
