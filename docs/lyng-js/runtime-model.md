# Lyng JS Runtime Model

This document describes the current JS3 runtime architecture. It defines the data
structures and ownership boundaries that are too expensive to rediscover later.

The concrete Phase 2 crate-level primitive architecture is specified in
[Runtime Primitives](runtime-primitives.md).
The concrete Phase 3 crate-level runtime substrate is specified in
[Runtime Substrate](runtime-substrate.md).

## Goals

- maximize interpreter throughput without sacrificing readability
- keep runtime objects compact and cache-friendly
- ensure guest-visible semantics are spec-traceable
- avoid foundational rewrites in `Value`, heap handles, object layout, or environments

## Runtime Topology

The top-level runtime is split into three layers:

- `Runtime`
  - owns the host hooks and the root `AgentCluster`
  - exposes the public embedding API
- `AgentCluster`
  - owns one or more `Agent` records
  - owns shared backing-store coordination and parked-agent state needed by later shared-memory features
- `Agent`
  - owns all engine-global mutable state for one JS execution agent
  - contains heaps, atoms, shapes, realms, code templates, and job queues

The initial implementation boots one `Runtime` with one `AgentCluster` containing one
single-threaded `Agent`. Future shared-memory and multi-agent work must extend the
cluster layer rather than replacing the runtime topology.

Thread-affinity rules:

- an `Agent` is thread-affine and executes on one host thread at a time
- the initial engine treats `Agent` as non-shareable across threads; any future Rust trait
  implementation details such as `!Send` or `!Sync` are expected to preserve that model
- cross-thread state is confined to `AgentCluster` coordination records and shared backing stores

## Core Handles

All heap-owned runtime entities are referenced by typed 32-bit handles.

Required handle families:

- `ObjectRef`
- `StringRef`
- `SymbolRef`
- `BigIntRef`
- `EnvironmentRef`
- `CodeRef`
- `ShapeId`
- `RealmRef`
- `AtomId`

Rules:

- handles are copyable and cheap to pass by value
- handles are never reused as another handle type
- hot paths use typed handles, not trait objects or string keys
- release builds use plain 32-bit handles; debug-only generation tracking may be added out
  of band if needed for invariant checking

## Value Representation

`Value` is an 8-byte tagged representation optimized for 64-bit little-endian hosts.

The representation must support these classes:

- `Undefined`
- `Null`
- `Boolean`
- `Smi(i32)`
- `Double(f64)`
- `ObjectRef`
- `StringRef`
- `SymbolRef`
- `BigIntRef`
- internal sentinels

Internal sentinel values are required for:

- array holes
- uninitialized lexical bindings
- empty internal slots where the spec requires a non-guest sentinel

The exact bit patterns are still a Phase 2 implementation detail, but the encoding family
is now frozen: take 3 uses a NaN-tag-space strategy. In other words:

- `Double(f64)` values are stored directly
- non-double values use NaN tag space for:
  - a small type tag
  - a 32-bit payload for `Smi(i32)` or typed handles
  - internal sentinel values

Phase 2 may still choose the final tag constants and helper APIs, but it does not revisit
the architectural choice between NaN-tag-space encoding, a top-byte tagging scheme, or a
boxed enum-like representation.

## Atoms, Strings, and Symbols

Take 3 uses two distinct string-like concepts:

- atoms
  - owned by an atom table
  - used for identifiers, property names, well-known strings, and bytecode metadata
  - represented by `AtomId`
  - not guest-visible JS values
- runtime strings
  - guest-visible string values
  - represented by `StringRef`
  - heap-owned and immutable

Property keys are represented as:

```rust
enum PropertyKey {
    Index(u32),
    Atom(AtomId),
    Symbol(SymbolRef),
}
```

Implications:

- normal property lookup never uses heap string comparison in the fast path
- identifier resolution and global name access compile against atom IDs
- `ToPropertyKey` on a string value atomizes once and then uses the atom thereafter
- `PropertyKey::Index(u32)` is reserved for canonical array-index keys only (`0` through
  `2^32 - 2`)
- `2^32 - 1` and any larger numeric-looking keys lower to `PropertyKey::Atom`, not
  `PropertyKey::Index`
- numeric-looking property accesses outside the array-index range lower to `PropertyKey::Atom`
- integer-indexed exotica such as TypedArrays impose additional checks above the `PropertyKey` layer

Atom lifetime policy:

- frontend, builtin, and code-metadata atoms live in the permanent portion of the shared
  atom namespace
- atoms created by runtime string atomization are collectible unless an owning runtime or
  code structure promotes them into permanent metadata
- string-record `AtomId` caches, shapes, code templates, environment/global binding tables,
  and other `AtomId`-bearing runtime records participate in explicit atom liveness
- dynamic property-name workloads must not imply immortal atom-table growth
- atom-sweep work is measured separately from the main mark walk once runtime atomization is
  enabled so pause-time regressions are visible in reports

Initial string policy:

- runtime strings are flat immutable strings with `Latin1` or `Utf16` storage
- string concatenation may allocate a new flat string
- string records may lazily cache their hash and atomized `AtomId`
- ropes, substring views, and external strings are deferred until benchmarks justify them
- concat-heavy workloads are an explicit benchmark watch item; if flat-string allocation
  pressure becomes material, rope- or builder-style concat records are the first sanctioned
  refinement rather than ad hoc builtin-specific caches

Initial symbol policy:

- symbols are heap-owned records with optional description strings
- well-known symbols have stable predefined handles reachable from the agent or realm

## Backing Stores

Binary-data storage is separate from ordinary object or element storage.

Take 3 uses cluster-owned backing stores for:

- `ArrayBuffer`
- `SharedArrayBuffer`
- typed-array and `DataView` views layered over those buffers

Rules:

- backing stores are not ordinary object payloads and are not represented as per-agent GC
  arenas owned by one `Agent`
- non-shared and shared backing stores use explicit lifetime management outside any single
  agent's tracing walk
- shared backing stores use atomic cross-agent lifetime management in an `Arc`-like or custom
  atomic-refcounted record owned by the cluster layer
- wrapper JS objects such as `ArrayBuffer`, `SharedArrayBuffer`, and typed-array views remain
  agent-local GC objects; the underlying backing-store allocation may outlive any one wrapper
  until the last cross-agent reference is dropped
- `ArrayBuffer` detachment semantics are buffer-local metadata transitions
- `SharedArrayBuffer` instances are never detached; sharedness is tracked on the backing-store
  record instead
- later growable or resizable buffer features extend the same backing-store owner rather than
  inventing a second raw-byte storage abstraction

## Heap and GC

The initial collector is non-moving mark-sweep with stable handles.

The heap is split by storage concern rather than by one giant tagged arena:

- object storage
- string storage
- bigint storage
- environment storage
- symbol storage
- code template storage
- shape storage

Fragmentation mitigation is part of the initial non-moving design:

- fixed-size record pages stay homogeneous per storage domain
- variable-size payloads such as string buffers, bigint limbs, slot buffers, and similar
  out-of-line storage use size-classed side allocations within the owning domain rather than
  one mixed page format
- fully free pages may be reclaimed or recycled at page granularity
- `AllocationLifetime::LongLived` exists so later implementations can segregate long-lived
  metadata from ordinary guest churn without redesigning allocation APIs

Requirements:

- each storage domain exposes typed allocation APIs
- mark bits and free lists are managed per domain
- root walking is explicit and centralized
- rooting is explicit; the collector never relies on conservative Rust stack scanning
- the VM, realms, job queue, and host strong references are all root sources
- runtime atom entries participate in the same explicit liveness walk discipline as other
  runtime-owned structures; they are not an immortal side map
- cross-domain tracing is explicit:
  - object slots, environment slots, and runtime records that store `Value` instances trace
    by decoding the value tag and marking the owning storage domain
  - runtime records that store typed handles directly trace through the owning domain's marker
- heap-owned `Value` stores and typed-handle stores route through centralized mutation helpers
  even while the initial collector does not yet need write barriers
- backing stores used by `ArrayBuffer` and `SharedArrayBuffer` are not reclaimed by any one
  agent heap's mark-sweep pass; local wrapper objects are traced, while backing-store lifetime
  is coordinated separately by the cluster-owned backing-store owner

Out of scope for the initial collector:

- compaction
- generations
- concurrent marking
- moving objects
- weak references, ephemerons, and finalization semantics needed by `WeakRef` and
  `FinalizationRegistry`

These may be added later only if the stable-handle contract is preserved or an explicit
redesign is approved in the plan.

Future generational, incremental, or concurrent collection work is expected to require write
barriers on heap-to-heap stores. Take 3 therefore treats centralized mutation hooks as part
of the frozen runtime contract even before any non-no-op barrier implementation exists.

The intended later generational path is additive, not an architectural reset:

- a nursery or other young-allocation policy may be layered onto ordinary `Default`
  allocations while `LongLived` allocations bypass it
- remembered sets and cross-generation barriers hang off the existing `mut_store` boundary
- explicit roots and stable typed handles remain the rooting and identity model
- nursery sizing, tenuring heuristics, and incremental policy remain intentionally unfrozen,
  but no later collector design should require conservative stack scanning or pointer-identity rewrites

## Object Layout

The object model is shape-based for named properties and separate for indexed elements.

Every object has:

- object flags
- object kind
- prototype reference
- shape ID for named-property layout
- named-slot storage
- element storage

The core object header must stay small and hot. Cold metadata belongs elsewhere.
Class-private data follows the same rule: objects may carry one traced private-slot buffer in the
hot record, while class records, private layouts, and installed-brand metadata stay out of line.

Internal-method dispatch is centralized and object-kind driven from the start. That
dispatch layer must already be Proxy-aware even before `Proxy` lands as a later feature,
so future trap interception extends the existing dispatch path rather than replacing it.

## Object Kind Taxonomy

The object header stores a coarse object kind, and some kinds carry additional subtype or
flag metadata in their out-of-line payloads.

Required coarse kinds:

- `Ordinary`
- `Function`
- `Array`
- `Proxy`
- `ModuleNamespace`
- `TypedArray`
- `ArrayBuffer`
- `DataView`
- later exotics as additional phases require

The coarse `ObjectKind` enum is forward-compatible and carries these major variants from
the start. Phase 3 only requires ordinary and function dispatch to be executable; the
other coarse kinds are reserved enum variants whose concrete semantics arrive in their
owning later phases.

Function objects are a coarse kind with a richer payload. The function payload carries the
internal slots needed for dispatch:

- `[[Code]]` or builtin entrypoint identity such as `BuiltinFunctionId`
- `[[Environment]]`
- `[[PrivateEnvironment]]` when the function was created during class evaluation
- `[[Realm]]`
- `[[ThisMode]]`
- constructor capability and constructor kind
- function flags for arrow, class constructor, generator, async, and async generator

Class-private dispatch relies on the existing function payload and object model rather than on a
parallel class runtime:

- methods use `[[HomeObject]]` to recover the owning class record
- functions created during class evaluation may additionally carry `[[PrivateEnvironment]]`
- private-field storage itself is object-local slot data guarded by explicit installed brands

The call hot path dispatches first on coarse object kind and then on the function payload
kind and flags. This keeps `[[Call]]` and `[[Construct]]` dispatch explicit without
exploding the number of object-header-level kinds.

### Named Properties

Named properties are governed by shapes.

Shape metadata stores:

- `PropertyKey`
- slot offset
- descriptor attributes
- property kind
- enumeration order metadata
- transition edges to child shapes

Named slot storage is dense and addressed by slot offset.

Descriptor kinds:

- data property
  - consumes one slot
- accessor property
  - consumes two contiguous slots for getter and setter

Consequences:

- ordinary objects with stable shapes are fast to read and write
- accessors do not automatically force dictionary mode
- attributes are not stored redundantly on every object instance

### Indexed Elements

Indexed elements are separate from named properties from day one.

Element storage starts in one of these modes:

- `Empty`
- `Dense`
- `Sparse`

Dense mode:

- contiguous vector-like storage
- supports hole sentinel values
- optimized for arrays and array-like objects

Sparse mode:

- dictionary fallback for pathological or very sparse indices
- slower, but semantically complete

Rules:

- named properties and indexed elements are never conflated
- array length semantics are implemented through element storage plus object flags or kind
- array exotics build on this split rather than replacing it

### Dictionary Fallback

Dictionary mode exists, but it is the slow path.

Objects transition to dictionary mode only for cases like:

- repeated add/delete churn
- shape-incompatible mutation patterns
- very dynamic descriptor changes that defeat shape sharing
- the initial implementation uses one explicit checked-in churn threshold rather than
  ad hoc call-site heuristics, and threshold changes are benchmarked like other object-layout
  decisions

Normal code should stay on shapes.

## Shape System

Shapes are canonicalized and shared across compatible objects.

Each shape records:

- parent shape
- transition key
- property metadata entry
- slot count
- property count
- small-shape inline descriptor data plus a flattened property table once the checked-in
  property-count threshold is crossed

Shape design requirements:

- adding a property follows deterministic transitions
- shape checks are cheap integer comparisons
- stable shapes with many properties consult the flattened property table rather than walking
  transition history linearly on every lookup
- property lookup on stable shapes does not allocate
- shape metadata is shared, not copied per object

Deletion policy:

- deletion may force dictionary mode rather than requiring complex back-transition logic
- readability and correctness win over heroic shape surgery for uncommon patterns

## Environments

Environments are split into compile-time layouts and runtime instances.

Compile-time data:

- `EnvironmentLayout`
  - binding names as atoms
  - binding kinds and flags
  - slot indices
  - whether the scope must be environment-backed

Runtime data:

- `Environment`
  - outer environment reference
  - layout reference
  - dense slot storage
  - optional object binding for object environments

### Binding Access Strategy

Normal lexical access should not use name lookup at runtime.

Compiler lowering rules:

- uncaptured local bindings use fixed frame-local registers
- captured bindings use environment slots
- global name access uses atom-based global lookup
- direct `eval` and `with` force slower dynamic lookup paths

This is one of the most important rewrite-avoidance decisions in the engine.

### TDZ and Uninitialized State

Lexical bindings in TDZ use an internal sentinel in the slot array rather than an external
map or side table. The slot itself carries the uninitialized state until initialization.

## Realms and Intrinsics

Each realm owns:

- the global object
- the global environment
- the intrinsic table

The intrinsic table uses explicit typed fields for well-known constructors, prototypes, and
singleton functions. It must not devolve into a string-keyed global map.

The runtime may support multiple realms, but the initial engine will bootstrap one default
realm first.

## Execution Contexts

Execution contexts are runtime records, not incidental VM locals.

Each context must at minimum track:

- current realm
- current executable identity
- lexical environment
- variable environment
- private environment when relevant
- `this` binding state
- `new.target` when relevant

The execution context stack lives in `lyng-js-env`, not in random VM helper state.

## Jobs and Host Boundary

The core runtime owns job queue data structures, but host-triggered behavior is routed
through `lyng-js-host`.

This split means:

- the engine controls ordering and lifecycle of queued jobs
- the host controls how external work is requested or surfaced
- test262 support can inject host behaviors without polluting core runtime ownership

Module loading, job integration points, and diagnostic output all use the host boundary
rather than ad hoc callbacks scattered around the VM.

## Slow Path Policy

The runtime explicitly allows slower fallback modes for difficult semantics:

- dictionary objects
- sparse elements
- direct `eval`
- `with`
- megamorphic property access
- uncommon descriptor transformations

These paths must be correct and readable, but they should not contaminate the normal fast path.

## Invariants

Non-negotiable invariants:

- `PropertyKey::Atom` is the default named-key path
- `2^32 - 1` is not treated as `PropertyKey::Index`
- ordinary object named-slot access is shape-plus-offset based
- environments use dense slots for non-dynamic bindings
- handles remain stable for the lifetime of a live object
- guest execution must not observe internal sentinels
- root walking must cover all live runtime-owned state
