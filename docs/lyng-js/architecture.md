# Lyng JS Architecture

Source of truth: ECMA-262 Edition 16 (`docs/ECMA-262_16th_edition_june_2025.pdf`)

This is the top-level architecture reference for JS3. It captures the runtime and compiler
decisions that are expensive to revisit and points to the deeper subsystem notes that
extend them. For contributor onboarding, start with [README.md](README.md). For current
Phase 6 status and the active conformance tail, read [ecma262-completion.md](ecma262-completion.md).

Companion architecture notes:

- [Frontend Architecture](frontend-architecture.md)
- [Runtime Model](runtime-model.md)
- [Runtime Primitives](runtime-primitives.md)
- [Runtime Substrate](runtime-substrate.md)
- [Shared Memory and Backing Stores](shared-memory-and-backing-stores.md)
- [Bytecode and VM](bytecode-and-vm.md)
- [Dynamic Scope and Eval](dynamic-scope-and-eval.md)
- [Builtin Bootstrap](builtin-bootstrap.md)
- [ECMA-262 Completion](ecma262-completion.md)
- [Engineering Standards](engineering-standards.md)

## Scope

This architecture set freezes the decisions that are expensive to revisit later:

- frontend/shared atom boundary
- runtime ownership and process model
- primitive value, handle, GC, and rooting contracts
- runtime, realm, environment, object, and host layering
- `Value`, handle, atom, string, object, and environment layout
- bytecode encoding and VM execution contract
- benchmarking, safety, and code review standards

It does not restate every algorithm or every conformance bug. Subsystem-specific behavior
and the live completion state belong in the companion notes, especially
[ecma262-completion.md](ecma262-completion.md).

## Current Status

- JS3 is the repository JavaScript engine.
- Phase 6 is active.
- Milestones `6A1` through `6G2c` are closed.
- `6H` is the active remaining tail and is closing on top of the architecture described here.

## System Shape

Take 3 is a layered engine with explicit crate ownership:

```text
lyng-js-common
  -> lyng-js-lexer
  -> lyng-js-ast

lyng-js-parser
  depends on lyng-js-common, lyng-js-lexer, lyng-js-ast

lyng-js-sema
  depends on lyng-js-common, lyng-js-ast

source text
  -> lyng-js-lexer
  -> lyng-js-parser
  -> lyng-js-sema
  -> lyng-js-compiler
  -> lyng-js-bytecode
  -> lyng-js-vm
  -> lyng-js-builtins / lyng-js-host / lyng-js-cli
```

The runtime-facing layer below compilation is:

```text
lyng-js-common
  -> lyng-js-types

lyng-js-gc
  depends on lyng-js-common, lyng-js-types

lyng-js-ops
  depends on lyng-js-common, lyng-js-types, lyng-js-gc

lyng-js-host
  depends on lyng-js-common, lyng-js-types

lyng-js-objects
  depends on lyng-js-common, lyng-js-types, lyng-js-gc

lyng-js-env
  depends on lyng-js-common, lyng-js-types, lyng-js-gc, lyng-js-objects, lyng-js-host

lyng-js-compiler / lyng-js-vm / lyng-js-builtins
  consume lyng-js-common, lyng-js-types, lyng-js-gc, and later higher runtime layers as needed
```

## Non-Negotiable Architecture Constraints

- The engine is interpreter first, but not interpreter only.
- No foundational rewrite is acceptable in:
  - `Value`
  - typed handles
  - string and atom ownership
  - object header and property storage
  - environment layout
  - bytecode encoding
  - call-frame layout
- The fast path must be data-oriented:
  - compact copyable handles
  - contiguous slot storage
  - no string maps in normal property or lexical access
  - no heap allocation in normal arithmetic, property lookup, or local variable access
- The engine must remain spec-traceable:
  - abstract operations have a single owner
  - builtin code calls shared abstract operations rather than reimplementing them
  - internal methods are centralized instead of scattered across match arms in unrelated modules
  - the centralized internal-method path is designed to admit later Proxy interception without rewrite

## Runtime Pipeline

At a high level:

1. `lyng-js-lexer` tokenizes source into compact token records with spans.
2. `lyng-js-parser` builds a typed-ID AST in arena storage.
3. `lyng-js-sema` performs early errors and computes scope and binding layout metadata.
4. `lyng-js-compiler` lowers AST plus semantic metadata into register bytecode.
5. `lyng-js-vm` executes bytecode against the runtime model defined in `runtime-model.md`.
6. `lyng-js-builtins` boots a realm and installs constructors, prototypes, and globals.
7. `lyng-js-host` supplies the environment-specific hooks for jobs, modules, and generic embedding integration.

Conformance tooling such as `tools/lyng-js-test262` is an external embedding of the engine.
It may install embedding extensions like `$262`, but those are not part of the spec-only
engine bootstrap surface.

The key performance consequence is that the compiler must use the semantic metadata to
resolve as much as possible ahead of time:

- local bindings become fixed frame-local registers
- captured bindings become environment slots
- property names become `AtomId`s
- feedback sites are assigned at compile time

## Core Runtime Types

These types are treated as architecture, not implementation detail:

- `Value`
- `AtomId`
- `ObjectRef`
- `StringRef`
- `SymbolRef`
- `BigIntRef`
- `EnvironmentRef`
- `ShapeId`
- `CodeRef`
- `RealmRef`

The runtime model note defines their semantics in detail.

## Fast Path and Slow Path Policy

Take 3 explicitly separates fast-path and slow-path behavior.

- Fast path:
  - frame-register access for uncaptured locals
  - environment-slot access for captured bindings
  - shape-guarded named-property access
  - dense indexed-element access
  - Smi and double arithmetic
  - monomorphic and polymorphic call/property feedback
- Slow path:
  - dictionary objects
  - sparse elements
  - direct `eval` and `with`
  - megamorphic property access
  - uncommon descriptor transitions
  - host callbacks and late-bound module work

Slow paths are acceptable. Hidden fast-path penalties are not.

## Lifetime and Ownership Model

- Each `Agent` owns heaps, atoms, code templates, shapes, and realms; `Runtime` and
  `AgentCluster` own host and shared-coordination state above that layer.
- Guest objects, strings, bigints, and environments live behind stable typed handles.
- The parser and compiler do not own runtime values.
- The VM does not own semantics that belong in `lyng-js-ops`, `lyng-js-env`, or `lyng-js-objects`.
- Builtins do not own duplicated abstract operations.

## Documentation Precedence

Read the live JS3 docs in this order:

1. [README.md](README.md)
2. this file
3. [ecma262-completion.md](ecma262-completion.md)
4. [engineering-standards.md](engineering-standards.md)
5. the relevant subsystem note for the code you are touching

If a design change touches a frozen structure, all affected documents must move together.
