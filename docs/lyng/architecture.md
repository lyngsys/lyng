# Lyng JS Architecture

Source of truth: ECMA-262 Edition 16 (`docs/ECMA-262_16th_edition_june_2025.pdf`).

This is the top-level architecture reference for Lyng JS. It records ownership
boundaries, runtime shape, and execution contracts that define the current engine.

Companion notes:

- [Frontend Architecture](frontend-architecture.md)
- [Runtime Model](runtime-model.md)
- [Runtime Primitives](runtime-primitives.md)
- [Runtime Substrate](runtime-substrate.md)
- [Shared Memory and Backing Stores](shared-memory-and-backing-stores.md)
- [Bytecode and VM](bytecode-and-vm.md)
- [Dynamic Scope and Eval](dynamic-scope-and-eval.md)
- [Builtin Bootstrap](builtin-bootstrap.md)
- [Engineering Standards](engineering-standards.md)
- [asm-DSL LLInt-style Interpreter Design](2026-05-16-asm-dsl-llint-interpreter-design.md)

## Current Engine Shape

Lyng JS is the only JavaScript implementation in this repository. It is a layered
interpreter built from small Rust crates with explicit ownership boundaries.

```text
source text
  -> lyng-lexer
  -> lyng-parser
  -> lyng-sema
  -> lyng-compiler
  -> lyng-bytecode
  -> lyng-vm
  -> lyng-builtins / lyng-host / lyng-cli
```

Runtime-facing crates sit below and beside that pipeline:

```text
lyng-common
  -> lyng-types
  -> lyng-gc
  -> lyng-objects
  -> lyng-env
  -> lyng-ops
  -> lyng-host
```

The compiler consumes frontend metadata and emits bytecode. The VM installs bytecode and
dispatches it through the asm-DSL substrate described in the Dispatch Substrate section.
Runtime semantics that belong to objects, environments, abstract operations, or host
hooks live outside the VM.

## Architecture Constraints

- The engine dispatches through the asm-DSL LLInt-style interpreter described in the
  Dispatch Substrate section. A Sparkplug-style Baseline JIT remains a future phase —
  see [`reports/lyng/jsc-aligned-engine-roadmap.md`](../../reports/lyng/jsc-aligned-engine-roadmap.md).
  New code should not assume native-code execution exists yet; the data model
  (FeedbackVector, Structures, NaN-boxed Value, 32-bit ShapeId) is being maintained as
  forward-compatible with that work.
- `Value`, typed handles, atoms, object storage, environment storage, bytecode templates,
  and call frames are architecture-level contracts.
- Normal local access uses frame registers or environment slots, not name lookup.
- Normal property access uses atoms, shapes, slots, elements, and feedback data, not string
  maps in the hot path.
- Guest-visible object operations route through the object-operation context APIs in
  `lyng-ops` unless a path is proven ordinary-only bootstrap code.
- Builtin implementations call shared abstract operations instead of duplicating spec logic.
- Test262 harness extensions such as `$262` are embedding behavior, not default realm
  bootstrap behavior.

## Dispatch Substrate

The VM dispatches bytecode through an asm-DSL LLInt-style substrate inspired by JSC's
`LowLevelInterpreter*.asm`. Handlers are tail-jump-dispatched through a single dispatch
table; engine state lives in pinned callee-saved registers across the entire handler
chain. Hot opcodes carry inline `naked_asm!` bodies; cold opcodes are short stubs that
bridge into shared Rust semantic bodies through `call_slow!`. AArch64 is the only
supported target today.

See [Bytecode and VM](bytecode-and-vm.md) for handler categories, register conventions,
`LlIntState` layout, and the slow-path bridge protocol. The parent design is in
[2026-05-16-asm-dsl-llint-interpreter-design.md](2026-05-16-asm-dsl-llint-interpreter-design.md);
the freshest engine snapshot is
[`reports/lyng/asm-dsl-engine-state-2026-05-22.md`](../../reports/lyng/asm-dsl-engine-state-2026-05-22.md).

## Runtime Pipeline

1. `lyng-lexer` tokenizes source text into compact tokens with spans and contextual flags.
2. `lyng-parser` builds typed-ID ASTs in arena storage.
3. `lyng-sema` performs early errors and computes scope, binding, capture, and layout data.
4. `lyng-compiler` lowers AST plus semantic metadata into bytecode templates.
5. `lyng-vm` installs templates as `CodeRef` records and executes them with register
   windows and call frames.
6. `lyng-builtins` creates the default realm shape by installing intrinsics,
   constructors, prototypes, global bindings, and native builtin dispatch tables.
7. `lyng-host` supplies embedding hooks for jobs, modules, dynamic import, and realm
   extensions.

## Core Runtime Types

The following runtime-facing types are treated as stable architecture concepts:

- `Value`
- `AtomId`
- `ObjectRef`
- `StringRef`
- `SymbolRef`
- `BigIntRef`
- `EnvironmentRef`
- `BackingStoreRef`
- `CodeRef`
- `RealmRef`
- `ShapeId`
- `FeedbackSlotId`
- `BuiltinFunctionId`
- `EmbeddingFunctionId`

## Fast And Slow Paths

Fast paths use:

- frame registers for uncaptured local bindings
- environment slots for captured bindings
- atomized property names
- shape-guarded named-property access
- dense indexed elements
- direct Smi/double arithmetic paths
- feedback vectors and inline-cache state keyed by bytecode feedback sites

Slow paths handle:

- dictionary properties
- sparse indexed elements
- direct `eval`
- `with` environments and unscopables
- proxy-observable object operations
- host callbacks
- module and dynamic import hooks

Slow paths are acceptable when explicit. Hidden hot-path penalties are not.

## Documentation Precedence

Read the live Lyng JS docs in this order:

1. [README.md](README.md)
2. this file
3. [engineering-standards.md](engineering-standards.md)
4. [2026-05-16-asm-dsl-llint-interpreter-design.md](2026-05-16-asm-dsl-llint-interpreter-design.md) when touching the dispatch substrate
5. the relevant subsystem note for the code being changed

Design changes that touch architecture-level structures must update every affected note in
the same change.
