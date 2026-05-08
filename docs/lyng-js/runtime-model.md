# Lyng JS Runtime Model

The runtime model defines the engine data structures that guest execution observes
indirectly: values, handles, atoms, strings, symbols, bigints, objects, environments,
realms, agents, jobs, code records, and backing stores.

## Runtime Topology

The runtime is split into three layers:

- `Runtime` owns embedding-facing host hooks and the root `AgentCluster`.
- `AgentCluster` owns agent records, shared backing-store coordination, and cross-agent
  synchronization records.
- `Agent` owns the mutable state for one JavaScript execution agent: heaps, atoms, shapes,
  realms, code templates, environments, jobs, and VM-owned execution state.

The current embedding path boots one runtime with one cluster and one thread-affine agent.
Cross-agent data is kept behind the cluster boundary.

## Typed Handles

Heap-owned runtime entities are referenced by compact typed handles:

- `ObjectRef`
- `StringRef`
- `SymbolRef`
- `BigIntRef`
- `EnvironmentRef`
- `CodeRef`
- `SuspendedExecutionRef`
- `BackingStoreRef`
- `RealmRef`
- `ShapeId`
- `FeedbackSlotId`

Handles are copyable, non-zero 32-bit IDs. `Option<Handle>` stays the same size as the
handle. Handle families do not alias each other.

## Values

`Value` is the guest value carrier. It represents:

- `undefined`
- `null`
- booleans
- small integers
- doubles
- objects
- strings
- symbols
- bigints
- internal sentinels

Internal sentinels cover array holes, uninitialized lexical bindings, and empty internal
slots that cannot be exposed as guest values.

## Atoms, Strings, Symbols, And BigInts

Atoms are engine-owned names used for identifiers, property names, well-known strings,
bytecode metadata, and builtin metadata. They are represented by `AtomId` and are not guest
string values.

Runtime strings are guest-visible immutable strings represented by `StringRef`. String
records may cache hash and atomization data.

Symbols are heap records represented by `SymbolRef`. Well-known symbols have stable IDs and
are available through realm/agent state.

BigInts are heap records represented by `BigIntRef` with arithmetic implemented through the
runtime operation layer.

Property keys use:

```rust
enum PropertyKey {
    Index(u32),
    Atom(AtomId),
    Symbol(SymbolRef),
}
```

## Heap And Rooting

`lyng-js-gc` owns typed allocation domains, explicit rooting, tracing support, and weak
state management. Runtime storage is separated by record family rather than placed in a
single untyped arena.

Rooting is explicit. The collector does not rely on conservative Rust stack scanning.
Runtime records that store `Value` or typed handles trace through their owning domains.
Heap mutation routes through centralized helpers so store policy remains visible.

## Objects

`lyng-js-objects` owns object records, shapes, named slots, indexed elements, dictionary
fallback, receiver payloads, primitive wrappers, private elements, and ordinary internal
method implementations.

Object layout is organized around:

- object header and kind
- prototype reference
- shape ID
- named-property slot storage
- indexed-element storage
- flags for extensibility and object-specific behavior
- internal payloads for exotic and builtin-backed objects

Normal named-property access is shape/slot based. Dictionary mode is the fallback for
objects whose property shape no longer fits compact slot storage.

## Environments, Realms, And Jobs

`lyng-js-env` owns runtime, cluster, agent, realm, execution context, environment, module
record, backing-store, symbol, and job-queue substrate.

Environment records cover declarative, function, global, object, module, and private
environment behavior. Compiler-assigned binding layout maps lexical operations to frame
registers, environment slots, global records, or dynamic lookup.

Realms hold intrinsic objects, builtin caches, global object state, and host integration
points. Job queues and host hooks sit at the runtime/agent boundary.

## Host Boundary

`lyng-js-host` defines embedding hooks for jobs, module loading, dynamic import, and realm
extensions. Host behavior is explicit and does not leak into parser, compiler, object, or
builtin ownership.

## Invariants

- Runtime values and handles remain compact and copyable.
- Guest-visible failures use the engine `Completion` and `AbruptCompletion` model.
- Object semantics route through the operation layer when traps or observable internal
  methods can be involved.
- The VM owns execution mechanics, not object or environment semantics.
