# Lyng

Lyng is a Rust JavaScript engine, distributed as a Cargo workspace of small,
single-responsibility crates (frontend, compiler, runtime substrate, builtins,
CLI) plus the verification tooling around it.

## What this is

An experiment in agentic coding.

JavaScript is a uniquely good target for an LLM-driven implementation: ECMA-262 is a
precise, exhaustive spec, and Test262 is a large, public conformance suite with tens of
thousands of cases. That combination lets an agent ground every behavioral decision in
spec text and verify itself against a corpus it cannot bullshit its way through.

The code is slop. The goal is **solid gold slop** — spec-faithful, well-tested, cleanly
organized, and held to a real quality bar despite being agent-written. The `AGENTS.md` at
the repo root and the one inside `crates/lyng-js/` encode the standards the agents are
held to.

## Current state

As of May 2026, Lyng passes 100% of Test262 in every category except `intl402`, which
has not been started. Current work is on runtime performance.

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

- `crates/lyng-js/`: engine crates, integration tests, runtime/compiler implementation
- `crates/lyng-js-vm-dsl/`: proc-macro substrate for the asm-DSL interpreter
- `tools/`: Test262 runner, benchmark/runtime-report tooling, DSL codegen

## Read next

- [Engine overview](crates/lyng-js/README.md)
- [Docs index](docs/lyng-js/README.md)
- [Repo-level agent guide](AGENTS.md)
