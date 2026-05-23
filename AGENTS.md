# AGENTS

This file is the repo-level operating guide for coding agents working in `lyng`.

## What This Repo Is

`lyng` is a Rust workspace containing a single active implementation track: the Lyng JavaScript engine in `crates/lyng/*`, the proc-macro asm-DSL substrate in `crates/lyng/vm-dsl/`, and the tooling around them under `tools/`.

The root workspace members are defined in `Cargo.toml`.

## First Files To Read

Start here before making non-trivial changes:

- `Cargo.toml`
- `docs/lyng/README.md`
- `docs/lyng/architecture.md`
- `docs/lyng/engineering-standards.md`

If you are changing a specific subsystem, read the crate-local sources and tests for that subsystem before editing.

For any Lyng JS work, also read `crates/lyng/AGENTS.md`. This applies even when the
files being edited live outside `crates/lyng`, such as `docs/lyng`,
`tools/lyng-test262`, `tools/lyng-bench`, `reports/lyng`, or
`testdata/test262`.

## Workspace Map

- `crates/lyng/common`: shared Lyng JS value/string/source-location types and interning
- `crates/lyng/lexer`: hand-written lexer
- `crates/lyng/parser`: parser and parse errors
- `crates/lyng/ast`: arena-backed AST nodes
- `crates/lyng/sema`: semantic analysis tables and resolution metadata
- `crates/lyng/bytecode`: bytecode IR, opcodes, disassembler
- `crates/lyng/compiler`: AST/sema -> bytecode lowering
- `crates/lyng/gc`: GC-adjacent runtime storage primitives
- `crates/lyng/types`: shared runtime and builtin ids/types
- `crates/lyng/host`: host hooks and embedding interfaces
- `crates/lyng/objects`: object model/runtime objects
- `crates/lyng/env`: environments and execution-context substrate
- `crates/lyng/ops`: runtime semantic operations
- `crates/lyng/vm`: bytecode interpreter
- `crates/lyng/builtins`: builtin bootstrap, constructors, prototypes, and globals
- `crates/lyng/cli`: CLI entrypoint for parse/compile/evaluate flows
- `crates/lyng/tests`: Lyng JS integration, conformance, and regression coverage
- `crates/lyng/vm-dsl`: proc-macro crate for the asm-DSL interpreter substrate
- `tools/lyng-bench`: unified Lyng JS benchmark, memory-report, and bytecode-density runner
- `tools/lyng-dsl-codegen`: codegen for VM DSL cold-handler stubs
- `tools/lyng-test262`: external whole-corpus Test262 embedding and report entrypoint with path-based filtering
- `testdata/test262`: Test262 checkout used by the Lyng JS harnesses

## Repo Priorities

Follow these project-specific constraints when making changes:

- Spec fidelity beats clever abstraction. This repo prefers explicit state machines and algorithm-shaped code.
- This project has an unusually high quality bar. Code quality is paramount: correctness, readability, maintainability, performance, and memory discipline are all first-order requirements.
- Keep dependencies minimal and well-justified. The JS PRD explicitly treats dependency growth as a design decision, not a convenience.
- Prefer targeted fixes over broad rewrites. Large refactors are risky in the parser and VM codepaths.
- Preserve crate boundaries. Shared types belong in `common`; avoid creating sideways dependencies between higher-level crates.

## Lyng JS Priorities

See `crates/lyng/AGENTS.md` for the detailed Lyng JS operating guide.

- Lyng JS remains focused on ECMA-262 semantics and conformance.
- Aim for a gold-standard implementation bar. Do not treat code quality or readability as secondary to feature completion.
- Prioritize code quality, readability, performance, memory behavior, cleanup, auditability, and verification clarity.
- Performance optimizations must be defensible as general semantic engine improvements, not benchmark-score hacks. Benchmarks are evidence for bottlenecks and regressions; do not special-case a benchmark source shape, input string, or harness behavior just to improve a score. Gold-standard parity comes from improving the engine, not cheating the benchmarks.
- Keep docs, tooling, reports, and issue tracking aligned with the live Lyng JS docs and the checked-in report/report-manifest flow under `reports/lyng/`.
- Do not blur core ECMA-262 completion work with ECMA-402 Intl or other extension work unless the user explicitly asks for that scope.

## Editing Expectations

- Match the surrounding style. The codebase favors straightforward control flow over macro-heavy or framework-heavy patterns.
- Keep public APIs and data layouts stable unless the task requires a breaking change.
- When changing JS semantics, anchor behavior to ECMA-262 sections when practical. The design docs explicitly expect spec-traceable behavior.
- Add comments sparingly and only where the algorithm or ownership model is genuinely non-obvious.
- Leave it cleaner than you found it

## Rust Module Guidelines

- One major type per file when it has significant `impl` blocks.
- Split code into focused modules with clear ownership. If a source file keeps growing
  because it is collecting multiple responsibilities, treat that as a design problem and
  split it by domain before it becomes hard to review.
- Keep `lib.rs` and `main.rs` thin: use them for `mod` declarations, re-exports, and top-level wiring.
- If a package has both a binary and a library, put the logic in `lib.rs`; keep `main.rs` as a thin wrapper.
- For new module trees, use the Rust 2018 style with a named parent file plus directory children instead of `mod.rs`.
- Default to private visibility. Use `pub(crate)` for crate-internal sharing and `pub(super)` for parent-only access.
- Only use `pub` for intentional public API.
- Flatten public APIs with `pub use` from `lib.rs` so callers do not need to know the internal directory structure.
- Organize by domain, not by technical kind.
- If a crate grows large enough to justify it, use a private `src/prelude.rs` with `pub(crate) use` for frequently shared imports.
- Prefer inline unit tests with `#[cfg(test)] mod tests`; if a test module grows large, extract it to `src/<module>/tests.rs`.
- Put integration tests in `tests/`.

## Generated And Fixture Content

- `reports/lyng/` contains generated reports. Do not hand-edit them unless the task is explicitly about report output.
- `testdata/test262/` is a fixture corpus. Treat it as test input, not normal implementation files.
- The harness tools may generate new report files during verification. Avoid deleting unrelated generated reports unless the user asks.

## Build And Test Commands

Run focused commands first, then widen scope only if needed.

### General workspace

- `cargo test`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -W clippy::pedantic -W clippy::nursery`

All code should pass pedantic Clippy and the experimental nursery lint group. Treat
Clippy findings as design feedback, not cosmetic noise; fix the code unless there is a
clear, documented reason to allow a specific lint locally.

### Lyng JS engine

- `cargo test -p lyng-parser`
- `cargo test -p lyng-compiler`
- `cargo test -p lyng-vm`
- `cargo test -p lyng-tests`
- `cargo run --release -p lyng-test262 -- --filter built-ins/Temporal/Instant --report /tmp/lyng-test262-temporal.md -j 4`
- `cargo run --release -p lyng-test262 -- --report /tmp/lyng-test262-report.md -j 12`
- `cargo run --release -p lyng-bench -- runtime --report /tmp/lyng-bench.md`
- `cargo run --release -p lyng-bench -- density --report /tmp/lyng-bytecode-density.md`

Notes:

- Lyng JS is the only JavaScript implementation track in this repo.
- Prefer targeted `lyng-*` crate tests first, then the relevant `lyng-test262 --filter ...` slice or whole-corpus report flow when semantics or performance-sensitive VM/compiler behavior changes.
- Use `lyng-bench density` for bytecode-density/encoding validation.

## Change-Specific Verification

Pick the narrowest useful verification for the area you touch:

- Lyng JS parser/compiler/vm change: run the nearest `lyng-*` crate tests plus the relevant `lyng-test262 --filter ...` slice or whole-corpus report flow if behavior changes; add `lyng-bench runtime` for hot-path or memory-sensitive work and `lyng-bench density` when bytecode density/encoding changes
- CLI-only change: run the binary directly with a representative script

If you do not run verification, say so clearly in your handoff.

## Practical Workflow

1. Read the relevant docs and the owning crate.
2. Inspect nearby tests before changing behavior.
3. Make the smallest coherent change.
4. Run targeted verification.
5. Summarize behavior changes and any unverified risk.

## Known Good Entry Points

Useful files when tracing behavior:

- `crates/lyng/lexer/src/lexer.rs`
- `crates/lyng/parser/src/lib.rs`
- `crates/lyng/compiler/src/lib.rs`
- `crates/lyng/env/src/lib.rs`
- `crates/lyng/objects/src/lib.rs`
- `crates/lyng/vm/src/lib.rs`
- `crates/lyng/tests/src/lib.rs`

## Avoid These Mistakes

- Do not add broad new dependencies without explicit justification.
- Do not hand-edit generated reports as if they were source.
- Do not run the full Test262 corpus by default when a targeted slice is enough.
- Do not change parser or VM control flow without reading the corresponding tests and docs first.

# Agent Instructions

## Issue tracking

This project uses **dcat** for issue tracking. You MUST run `dcat prime --opinionated` for instructions.
Then run `dcat list --agent-only` to see the list of issues. Generally we work on bugs first, and always on high priority issues first.

When running multiple `dcat` commands, make separate parallel Bash tool calls instead of chaining them with `&&` and `echo` separators.

Mark each issue `in_progress` right when you start working on it — not before. Set `in_review` when work on that issue is done before moving on. The status should reflect what you are *actually* working on right now.

It is okay to work on multiple related issues at the same time, but do NOT batch-mark an entire backlog as `in_progress` upfront. If there is a priority conflict, ask the user which to focus on first.

When research or discussion produces findings relevant to an existing issue, ask these as **separate questions in order**:

1. First ask: "Should I update issue [id] with these findings?"
2. Only after that, separately ask: "Should I start working on the implementation?"
Do NOT combine these into one question. The user may want to update the issue without starting work.

### Closing Issues - IMPORTANT

NEVER close issues without explicit user approval. When work is complete:

1. Set status to `in_review`: `dcat update --status in_review $issueId`
2. Ask the user to test
3. Ask if we can close it: "Can I close issue [id] '[title]'?"
4. Only run `dcat close` after user confirms
