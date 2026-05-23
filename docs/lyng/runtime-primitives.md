# Lyng JS Runtime Primitives

Runtime primitives are the copyable types, heap records, descriptors, completions, and
primitive abstract operations used by the rest of the engine.

## Crate Ownership

- `lyng-js-types` owns representation-only runtime data: `Value`, typed handles, builtin
  IDs, property keys, descriptors, and completion types.
- `lyng-js-gc` owns allocation domains, mutator access helpers, tracing, rooting, weak
  state, and collection bookkeeping.
- `lyng-js-ops` owns ECMA-262 abstract operations and semantic helpers over runtime values,
  objects, property keys, descriptors, promises, Temporal records, shared memory, and
  conversions.

`types` does not allocate. `gc` does not own JavaScript semantics. `ops` coordinates
semantic behavior through runtime contexts and shared helper APIs.

## Values And Handles

`Value` is compact and copyable. Heap-backed values carry typed handles, while immediate
values cover undefined, null, booleans, small integers, doubles, and internal sentinels.

Typed handles are non-zero 32-bit IDs with separate Rust types for each storage family.
The type system prevents accidental mixing of object, string, symbol, bigint, environment,
code, backing-store, realm, shape, and feedback IDs.

## Property Keys And Descriptors

`PropertyKey` separates canonical array indices, atomized names, and symbols. This keeps
ordinary property lookup away from string comparison in normal paths.

`PropertyDescriptor` and descriptor attribute masks represent data/accessor descriptors,
presence bits, configurability, enumerability, writability, getters, setters, and values.
Descriptor helpers live in the operation layer so builtins and object internals share the
same normalization and validation logic.

## Completion Model

Guest-visible abrupt control uses:

- `Completion<T>` for normal or abrupt operation results
- `AbruptCompletion` for throw, return, break, continue, and engine error payloads

Rust panics are not guest-visible JavaScript failures. Semantic helpers return completion
types when guest code can observe failure.

## Heap Domains

The heap is separated into typed storage domains. Important record families include:

- objects
- strings
- symbols
- bigints
- environments
- code templates
- shapes
- backing-store references and metadata

The GC API exposes typed allocation and tracing entrypoints. Records that store handles or
values participate in explicit tracing.

## Primitive Records

Strings are immutable runtime records with text storage and optional cached metadata.
Symbols are identity records with optional descriptions and well-known-symbol support.
BigInts store signed arbitrary-precision integer data behind `BigIntRef`.

Atoms are not guest values. They are shared engine names used by identifiers, property
keys, builtin metadata, bytecode constants, and shape records.

## Operation Families

`lyng-js-ops` is organized by semantic area:

- pure primitive conversion and comparison
- object operations and internal-method contexts
- property reads and writes
- enumeration
- promise helpers
- proxy-sensitive behavior
- private elements
- typed-array and shared-memory helpers
- Temporal parsing and operations
- error construction and propagation helpers

Object-aware operations receive a runtime context so proxy traps, receiver handling,
ordinary/exotic dispatch, and abrupt completions remain centralized.

## Invariants

- `lyng-js-types` remains representation-only.
- Heap access and mutation stay behind typed storage and mutator helper APIs.
- Builtins, VM handlers, and object internals reuse shared operation helpers for
  guest-visible semantics.
- Descriptor and completion data remain explicit instead of encoded in ad hoc sentinel
  values.
