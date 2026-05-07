# AGENTS

This file is the repo-level operating guide for coding agents working in `lyng`.

## What This Repo Is

`lyng` is a Rust workspace with two active implementation tracks:

- A WHATWG-style HTML parser in `crates/html_parser`, backed by an arena DOM in `crates/dom`
- The `lyng-js` JavaScript engine in `crates/lyng-js/*`, which is the active JavaScript implementation

The root workspace members are defined in `Cargo.toml`. Several other directories exist as placeholders (`crates/css`, `crates/gfx`, `crates/layout`, `crates/net`, `crates/platform`, `components/webview`) but are not active workspace crates today.

## First Files To Read

Start here before making non-trivial changes:

- `Cargo.toml`
- `crates/html_parser/README.md`
- `crates/html_parser/docs/architecture.md`
- `crates/html_parser/docs/implementation-notes.md`
- `docs/lyng-js/README.md`
- `docs/lyng-js/architecture.md`
- `docs/lyng-js/engineering-standards.md`

If you are changing a specific subsystem, read the crate-local sources and tests for that subsystem before editing.

For any Lyng JS work, also read `crates/lyng-js/AGENTS.md`. This applies even when the
files being edited live outside `crates/lyng-js`, such as `docs/lyng-js`,
`tools/lyng-js-test262`, `tools/lyng-js-bench`, `reports/js/lyng-js`, or
`testdata/test262`.

## Workspace Map

### HTML stack

- `crates/dom`: arena-backed DOM types and serialization helpers
- `crates/html_parser`: tokenizer, tree builder, input stream, parse errors, public parse APIs
- `tools/html5lib_runner`: runs html5lib suites and writes reports under `reports/html/`
- `testdata/html5lib`: upstream-style fixture corpus for tokenizer, tree construction, serializer, and encoding

### Lyng JS stack

- `crates/lyng-js/common`: shared Lyng JS value/string/source-location types and interning
- `crates/lyng-js/lexer`: hand-written lexer
- `crates/lyng-js/parser`: parser and parse errors
- `crates/lyng-js/ast`: arena-backed AST nodes
- `crates/lyng-js/sema`: semantic analysis tables and resolution metadata
- `crates/lyng-js/bytecode`: bytecode IR, opcodes, disassembler
- `crates/lyng-js/compiler`: AST/sema -> bytecode lowering
- `crates/lyng-js/gc`: GC-adjacent runtime storage primitives
- `crates/lyng-js/types`: shared runtime and builtin ids/types
- `crates/lyng-js/host`: host hooks and embedding interfaces
- `crates/lyng-js/objects`: object model/runtime objects
- `crates/lyng-js/env`: environments and execution-context substrate
- `crates/lyng-js/ops`: runtime semantic operations
- `crates/lyng-js/vm`: bytecode interpreter
- `crates/lyng-js/builtins`: builtin bootstrap, constructors, prototypes, and globals
- `crates/lyng-js/cli`: CLI entrypoint for parse/compile/evaluate flows
- `crates/lyng-js/tests`: Lyng JS integration, conformance, and regression coverage
- `tools/lyng-js-bench`: unified Lyng JS benchmark, memory-report, and bytecode-density runner
- `tools/lyng-js-test262`: external whole-corpus Test262 embedding and report entrypoint with path-based filtering
- `testdata/test262`: Test262 checkout used by the Lyng JS harnesses

## Repo Priorities

Follow these project-specific constraints when making changes:

- Spec fidelity beats clever abstraction. This repo prefers explicit state machines and algorithm-shaped code.
- This project has an unusually high quality bar. Code quality is paramount: correctness, readability, maintainability, performance, and memory discipline are all first-order requirements.
- Keep dependencies minimal and well-justified. The JS PRD explicitly treats dependency growth as a design decision, not a convenience.
- Prefer targeted fixes over broad rewrites. Large refactors are risky in the parser and VM codepaths.
- Preserve crate boundaries. Shared types belong in `common` or `dom`; avoid creating sideways dependencies between higher-level crates.
- Do not treat placeholder directories as implemented subsystems unless the user explicitly asks to expand them.

## Lyng JS Priorities

See `crates/lyng-js/AGENTS.md` for the detailed Lyng JS operating guide.

- Lyng JS remains focused on ECMA-262 semantics and conformance.
- Aim for a gold-standard implementation bar. Do not treat code quality or readability as secondary to feature completion.
- Prioritize code quality, readability, performance, memory behavior, cleanup, auditability, and verification clarity.
- Keep docs, tooling, reports, and issue tracking aligned with the live Lyng JS docs and the checked-in report/report-manifest flow under `reports/js/lyng-js/`.
- Do not blur core ECMA-262 completion work with ECMA-402 Intl or other extension work unless the user explicitly asks for that scope.
- The old JavaScript engine has been removed. Treat Lyng JS as the only in-repo JavaScript engine and keep docs, tooling, and verification aligned with that cutover.

## Editing Expectations

- Match the surrounding style. The codebase favors straightforward control flow over macro-heavy or framework-heavy patterns.
- Keep public APIs and data layouts stable unless the task requires a breaking change.
- When changing JS semantics, anchor behavior to ECMA-262 sections when practical. The design docs explicitly expect spec-traceable behavior.
- When changing HTML parsing behavior, align naming and control flow with the WHATWG parsing algorithm rather than inventing alternate terminology.
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

- `reports/html/` and `reports/js/` contain generated reports. Do not hand-edit them unless the task is explicitly about report output.
- `testdata/html5lib/` and `testdata/test262/` are fixture corpora. Treat them as test inputs, not normal implementation files.
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

### HTML parser

- `cargo test -p lyng-html-parser --all-features`
- `cargo test -p lyng-html-parser --test tokenizer_tests`
- `cargo test -p lyng-html-parser --test tree_tests`
- `cargo test -p lyng-html-parser --test serializer_tests`
- `cargo test -p lyng-html-parser --test encoding_tests`
- `cargo run -p html5lib_runner -- --suite tokenizer`
- `cargo run -p html5lib_runner -- --suite tree --filter adoption`

Notes:

- The html5lib runner writes a Markdown report under `reports/html/` by default.
- `encoding_rs` support in `crates/html_parser` is behind the `encoding` feature; `--all-features` is the safest full validation path.

### Lyng JS engine

- `cargo test -p lyng-js-parser`
- `cargo test -p lyng-js-compiler`
- `cargo test -p lyng-js-vm`
- `cargo test -p lyng-js-tests`
- `cargo run --release -p lyng-js-test262 -- --filter built-ins/Temporal/Instant --report /tmp/lyng-js-test262-temporal.md -j 4`
- `cargo run --release -p lyng-js-test262 -- --report /tmp/lyng-js-test262-report.md -j 12`
- `cargo run --release -p lyng-js-bench -- runtime --report /tmp/lyng-js-bench.md`
- `cargo run --release -p lyng-js-bench -- density --report /tmp/lyng-js-bytecode-density.md`

Notes:

- Lyng JS is the only JavaScript implementation track in this repo.
- Prefer targeted `lyng-js-*` crate tests first, then the relevant `lyng-js-test262 --filter ...` slice or whole-corpus report flow when semantics or performance-sensitive VM/compiler behavior changes.
- Use `lyng-js-bench density` for bytecode-density/encoding validation.

## Change-Specific Verification

Pick the narrowest useful verification for the area you touch:

- DOM or tree-construction change: run at least the relevant `tree_tests` or html5lib tree filter
- Tokenizer change: run `tokenizer_tests` and any affected serializer/tree coverage
- HTML encoding change: run `encoding_tests` with `--all-features`
- Lyng JS parser/compiler/vm change: run the nearest `lyng-js-*` crate tests plus the relevant `lyng-js-test262 --filter ...` slice or whole-corpus report flow if behavior changes; add `lyng-js-bench runtime` for hot-path or memory-sensitive work and `lyng-js-bench density` when bytecode density/encoding changes
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

- `crates/html_parser/src/tokenizer/mod.rs`
- `crates/html_parser/src/tree/builder.rs`
- `crates/dom/src/node.rs`
- `crates/lyng-js/lexer/src/lexer.rs`
- `crates/lyng-js/parser/src/lib.rs`
- `crates/lyng-js/compiler/src/lib.rs`
- `crates/lyng-js/env/src/lib.rs`
- `crates/lyng-js/objects/src/lib.rs`
- `crates/lyng-js/vm/src/lib.rs`
- `crates/lyng-js/tests/src/lib.rs`
- `tools/html5lib_runner/src/main.rs`

## Avoid These Mistakes

- Do not assume placeholder crates are wired into the build.
- Do not add broad new dependencies without explicit justification.
- Do not hand-edit generated reports as if they were source.
- Do not run the full compatibility suites by default when a targeted slice is enough.
- Do not change parser or VM control flow without reading the corresponding tests and docs first.
- Do not reintroduce legacy-JS assumptions into docs, tooling, or verification; Lyng JS is the repository JavaScript engine.

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
