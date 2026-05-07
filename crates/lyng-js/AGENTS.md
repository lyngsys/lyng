# AGENTS

This file is the Lyng JS operating guide for coding agents. Read it before touching
`crates/lyng-js/*`, `docs/lyng-js/*`, `tools/lyng-js-test262`,
`tools/lyng-js-bench`, `reports/js/lyng-js/*`, or Test262-related fixtures.

Lyng JS is the repository JavaScript engine. The goal is a gold-standard ECMA-262
implementation in Rust: spec-compliant, fast, readable, maintainable, and disciplined
about memory behavior.

## The Quality Bar

Passing the targeted Test262 tests is necessary, but it is not sufficient. Test262 is
evidence, not the definition of done.

Every Lyng JS change must preserve or improve all of these:

- Spec compliance: behavior should be traceable to ECMA-262 concepts and observable
  semantics, not just to a fixture outcome.
- Performance: hot paths should stay compact, allocation-aware, and easy to profile.
- Readability: code should be explicit, algorithm-shaped, and reviewable by someone
  comparing it to the spec or architecture docs.
- Module focus: code should live in focused modules with clear responsibilities. Files
  ballooning in size are a design warning, not a badge of progress.
- Memory behavior: rooting, tracing, ownership, handle representation, and allocation
  patterns are first-order design constraints.
- Lint quality: code should pass pedantic Clippy and the experimental nursery lint group.

If making a test pass requires a shortcut that weakens any of these, stop and fix the
underlying design instead.

## What Good Work Looks Like

- Implement the semantic operation, state transition, or representation the engine is
  actually missing.
- Put behavior in the crate that owns it instead of adding convenience paths in callers.
- Keep files focused. Split large implementation files by semantic responsibility before
  adding another unrelated block to the bottom.
- Keep slow paths explicit and hot paths free of hidden allocation, string lookup, or
  broad dynamic dispatch.
- Prefer small, spec-shaped changes over broad rewrites.
- Add focused tests for the behavior being changed, then use Test262 as compatibility
  evidence.
- Update docs or reports when the architecture, verification story, or current behavior
  changes.

## Read Before Editing

Start with:

- `crates/lyng-js/README.md`
- `docs/lyng-js/README.md`
- `docs/lyng-js/architecture.md`
- `docs/lyng-js/engineering-standards.md`

Then read the subsystem note and nearby tests for the area being changed:

- Frontend: `docs/lyng-js/frontend-architecture.md`
- Runtime model: `docs/lyng-js/runtime-model.md`
- Runtime primitives: `docs/lyng-js/runtime-primitives.md`
- Runtime substrate, realms, agents, jobs, modules: `docs/lyng-js/runtime-substrate.md`
- Shared memory and backing stores: `docs/lyng-js/shared-memory-and-backing-stores.md`
- Bytecode, VM, and feedback: `docs/lyng-js/bytecode-and-vm.md`
- Builtin bootstrap: `docs/lyng-js/builtin-bootstrap.md`
- Dynamic scope and eval: `docs/lyng-js/dynamic-scope-and-eval.md`

## Crate Ownership

Respect these boundaries:

- `common`: shared source, span, string, value-adjacent primitives
- `lexer`, `parser`, `ast`, `sema`: source frontend, syntax, early errors, scope and
  binding metadata
- `types`: representation-only runtime types and IDs
- `gc`: allocation, rooting, tracing, and storage substrate mechanics
- `objects`: object storage and ordinary internal methods
- `env`: agents, realms, execution contexts, environments, jobs, modules, and backing
  store coordination
- `ops`: reusable ECMA-262 abstract operations
- `bytecode`: immutable bytecode templates, opcodes, metadata containers
- `compiler`: AST plus semantic metadata to bytecode lowering
- `vm`: bytecode installation and interpretation
- `builtins`: realm bootstrap, intrinsic objects, builtin descriptors, native dispatch
- `host`: embedding hooks and host-defined behavior
- `cli`, `tests`, `tools/lyng-js-test262`, `tools/lyng-js-bench`: entrypoints and
  verification tooling

Do not move semantics sideways to make a local patch easier. If a VM change needs object
or environment semantics, route it through the owning API.

## Module Shape

Lyng JS code should be split into focused modules that map to semantic domains,
representation ownership, or execution phases. A module should have a clear answer to:
what it owns, who calls it, and which invariants it protects.

Large files are a warning sign. Before adding more code to an already large file, decide
whether the new behavior belongs in a focused child module, an existing domain module, or
an owning crate API. Splitting is especially important for VM, builtin, object,
environment, and operation code, where mixed responsibilities quickly hide performance
costs and spec-observable behavior.

Keep `lib.rs` thin: module declarations, intentional re-exports, and top-level wiring.
Avoid utility catch-all modules. Prefer names that describe the domain or algorithm they
own.

## Spec Compliance

- Anchor behavior to ECMA-262 Edition 16 concepts when practical.
- Preserve guest-visible completion behavior, abrupt completions, coercion order,
  property access observability, realm boundaries, job ordering, and host hooks.
- Builtins should share abstract operations instead of copying subtly different logic.
- Test262 harness behavior such as `$262` belongs to embedding and test tooling, not
  default realm bootstrap.
- Do not add ECMA-402 Intl, browser APIs, Node APIs, or native-code execution unless the
  task explicitly asks for that scope.

When a failing fixture points at a broader semantic gap, implement the broader semantic
rule. Avoid fixture-shaped special cases.

## Performance Discipline

Before editing VM, object, environment, operation, bytecode, compiler, or builtin hot
paths, identify whether the path is hot or cold.

Hot-path expectations:

- No heap allocation in normal local binding access.
- No string-map lookup in normal lexical or named-property access.
- Prefer atoms, typed IDs, shapes, slots, registers, handles, and metadata tables.
- Keep VM dispatch direct and easy to profile.
- Avoid trait-object dispatch in interpreter loops unless measurement justifies it.
- Keep runtime feedback separate from immutable bytecode templates.

Run `tools/lyng-js-bench` when a change plausibly affects runtime throughput, allocation
behavior, memory reporting, or bytecode density.

## Memory And Safety

- Rooting and tracing must be explicit around allocation paths.
- `unsafe` must have a local invariant comment and a narrow scope.
- Preserve compact handle and value representations unless the task explicitly requires a
  layout change.
- Represent sentinels explicitly and prevent them from escaping as guest values.
- Host callbacks and embedding functions need clear ownership and error propagation.
- Shared-memory behavior must stay behind backing-store and shared-memory operation APIs.

## Verification

Use the narrowest verification that proves the change, then widen when the change affects
shared semantics or hot paths.

Focused crate tests:

```sh
cargo test -p lyng-js-parser
cargo test -p lyng-js-compiler
cargo test -p lyng-js-vm
cargo test -p lyng-js-tests
```

Linting:

```sh
cargo clippy --all-targets --all-features -- -W clippy::pedantic -W clippy::nursery
```

Treat Clippy warnings as code-quality findings. Prefer improving names, structure,
control flow, ownership, or APIs over suppressing lints. Use local `allow` attributes only
when the lint conflicts with a deliberate spec-shaped implementation or measured hot-path
choice, and document the reason.

Targeted Test262:

```sh
cargo run --release -p lyng-js-test262 -- --filter built-ins/Temporal/Instant --report /tmp/lyng-js-test262-temporal.md -j 4
```

Whole-corpus Test262, when broad semantic changes justify it:

```sh
cargo run --release -p lyng-js-test262 -- --report /tmp/lyng-js-test262-report.md -j 12
```

Benchmarks and density reports:

```sh
cargo run --release -p lyng-js-bench -- runtime --report /tmp/lyng-js-bench.md
cargo run --release -p lyng-js-bench -- density --report /tmp/lyng-js-bytecode-density.md
```

Do not claim a change is complete just because a targeted Test262 slice passes. Report
what was tested, what was not tested, and any remaining risk.

## Review Checklist

Before handing off Lyng JS work, ask:

- Is the behavior spec-traceable?
- Did the change preserve crate ownership?
- Did it avoid fixture-shaped shortcuts?
- Are hot paths free of accidental allocation and string lookup?
- Are proxy, environment, host, and realm-observable operations routed through owning APIs?
- Are rooting, tracing, handles, and storage lifetimes explicit?
- Is the code easier to read and maintain than the path it replaced?
- Are modules focused, or is a file growing because it has taken on multiple
  responsibilities?
- Does it pass pedantic and nursery Clippy without broad suppressions?
- Is verification scoped to the actual risk, including performance or density where needed?

## Red Flags

Stop and reconsider if you are about to:

- Mark work done solely because targeted Test262 passes.
- Add a special case named after a test file or fixture shape.
- Duplicate abstract-operation logic in a builtin, VM helper, or object path.
- Put runtime semantics in `vm` when `ops`, `objects`, `env`, or `builtins` owns them.
- Introduce heap allocation, string lookup, or trait-object dispatch into a hot path
  without measurement.
- Add a dependency as a convenience.
- Hand-edit generated reports as if they were source.
- Reintroduce legacy JavaScript-engine assumptions.
