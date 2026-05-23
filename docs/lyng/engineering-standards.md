# Lyng JS Engineering Standards

Lyng JS code is held to a high bar for correctness, readability, maintainability,
performance, memory behavior, and verification clarity.

## Principles

- Spec fidelity beats clever abstraction.
- Crate ownership is explicit.
- Public APIs are intentional and narrow.
- Hot paths stay compact and allocation-aware.
- Guest-visible behavior is traceable to ECMA-262 concepts.
- Dependencies are added only when their ownership and maintenance cost is justified.

## Layering Rules

- Frontend crates do not depend on runtime or VM crates.
- `lyng-types` remains representation-only.
- `lyng-gc` owns allocation/rooting/tracing mechanics, not JavaScript semantics.
- `lyng-objects` owns object storage and ordinary internal methods.
- `lyng-env` owns agents, realms, execution contexts, environments, jobs, modules, and
  backing-store coordination.
- `lyng-ops` owns reusable abstract operations.
- `lyng-compiler` owns lowering.
- `lyng-bytecode` owns bytecode templates and metadata containers.
- `lyng-vm` owns installation and interpretation.
- `lyng-builtins` owns realm bootstrap and builtin dispatch.
- `lyng-host` owns embedding hooks.

## API Ownership

- Prefer typed IDs over `usize` or strings across crate boundaries.
- Keep visibility private by default.
- Use `pub(crate)` for crate-internal sharing and `pub` only for intentional public API.
- Put shared semantic helpers in the owning crate instead of copying logic across call sites.
- Keep `lib.rs` thin: module declarations, re-exports, and top-level wiring.

## Hot Path Rules

- No heap allocation in normal local access.
- Avoid string maps in normal lexical or named-property access.
- Use atoms, shapes, slots, registers, and typed handles in hot paths.
- Keep VM dispatch direct and easy to profile.
- Avoid trait-object dispatch in core interpreter loops unless measurement justifies it.
- Route proxy-observable operations through shared operation contexts.

## Data Structure Rules

- Preserve compact handle and value representations.
- Keep object, environment, code, and backing-store records domain-owned.
- Store metadata in tables when it is cold or sparse.
- Keep runtime feedback separate from immutable bytecode templates.
- Represent sentinels explicitly and prevent them from escaping as guest values.

## Documentation Rules

- Architecture docs describe the current engine shape and invariants.
- Reports under `reports/lyng/` record verification output.
- Source comments explain non-obvious algorithms, ownership constraints, or spec mapping.
- Avoid comments that restate obvious code.

## Safety Rules

- Guest-visible failure uses `Completion` and `AbruptCompletion`.
- Unsafe Rust requires a local invariant comment and a narrow scope.
- Rooting and tracing must be explicit around allocation paths.
- Host callbacks and embedding functions must have clear ownership and error propagation.
- Shared-memory behavior must remain behind backing-store and shared-memory operation APIs.
- DSL boundary: the asm-DSL substrate (`crates/lyng/vm-dsl/` and
  `crates/lyng/vm/src/dsl/`) is the audited home for inline assembly,
  `#[unsafe(naked)]` functions, and the slow-path bridge. Changes to those
  modules require: a `// SAFETY:` invariant comment per unsafe block, an asm
  snapshot diff via `lyng-bench asm-diff` (when the DSL handler set is
  populated), and Miri coverage for the shim layer.

## Testing Rules

Use focused tests first:

```sh
cargo test -p lyng-parser
cargo test -p lyng-compiler
cargo test -p lyng-vm
cargo test -p lyng-tests
```

Use targeted Test262 filters for semantic changes and whole-corpus reports for broad
conformance changes:

```sh
cargo run --release -p lyng-test262 -- --filter built-ins/Temporal/Instant --report /tmp/lyng-test262-temporal.md -j 4
cargo run --release -p lyng-test262 -- --report /tmp/lyng-test262-report.md -j 12
```

Use benchmark tooling for hot-path, memory, or bytecode-density changes:

```sh
cargo run --release -p lyng-bench -- runtime --preset inner-loop --report /tmp/lyng-bench.md --json /tmp/lyng-bench.json
cargo run --release -p lyng-bench -- density --preset inner-loop --report /tmp/lyng-bytecode-density.md --json /tmp/lyng-bytecode-density.json
```

## Review Checklist

- Is the behavior spec-traceable?
- Does the change preserve crate ownership?
- Are guest-visible failures represented through completion paths?
- Are hot paths free of accidental allocation or string lookup?
- Are typed handles and metadata tables used consistently?
- Are proxy, environment, and host observability routed through the owning APIs?
- Is verification scoped to the changed behavior?

## Done Criteria

- Relevant docs and source agree on ownership and current behavior.
- Focused tests or reports have been run for the changed area.
- Unrun verification is called out explicitly.
- Generated reports are not hand-edited.
