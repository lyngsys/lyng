# Lyng JS Runtime Primitives

This document describes the concrete architecture of the primitive runtime layer: crate
boundaries, `Value`, typed handles, GC ownership, primitive heap records, descriptors,
completions, and primitive abstract-operation APIs.

This note is narrower than the full runtime model. It focuses on the primitive substrate
that later object, environment, compiler, and VM phases build on:

- [Architecture Overview](architecture.md)
- [Runtime Model](runtime-model.md)
- [Engineering Standards](engineering-standards.md)

## Goals

- freeze the primitive-runtime crate DAG before object or VM code exists
- keep `Value` and handle traffic compact in hot interpreter paths
- make rooting and tracing explicit so Rust stack locals never become hidden GC state
- keep primitive abstract operations centralized and spec-traceable
- define primitive heap records now so later phases do not retrofit strings, symbols, or bigints
- freeze atom lifetime classes now so runtime string atomization does not become an
  immortal-memory leak by accident

## Crate Ownership and Dependency DAG

Phase 2 owns three crates:

```text
lyng-js-common
  -> lyng-js-types

lyng-js-gc
  depends on lyng-js-common, lyng-js-types

lyng-js-ops
  depends on lyng-js-common, lyng-js-types, lyng-js-gc
```

Ownership is intentionally strict:

- `lyng-js-types` owns copyable runtime-facing data types only
- `lyng-js-gc` owns allocation, rooting, tracing, and storage-domain management
- `lyng-js-ops` owns ECMA-262 primitive semantics and descriptor/completion helpers

This is the frozen answer to the Phase 2 layering question:

- `lyng-js-types` does not depend on `lyng-js-gc`
- `lyng-js-gc` allocates and traces records addressed by handles defined in `lyng-js-types`
- `lyng-js-ops` depends on storage and type crates, not the other way around
- later `lyng-js-env` and `lyng-js-objects` depend on `lyng-js-types` and `lyng-js-gc`
- later object-aware modules inside `lyng-js-ops` may depend on `lyng-js-env` and
  `lyng-js-objects`, but those crates must not depend back on `lyng-js-ops`

That last rule prevents a future `ops <-> objects` cycle while keeping abstract-operation
ownership centralized.

The planned crate-dependency growth is explicit:

- Phase 2 keeps `lyng-js-ops` primitive-only
- Phase 4 intentionally expands `lyng-js-ops` to depend on `lyng-js-env` and
  `lyng-js-objects` for the first object-aware abstract-operation modules
- that Cargo.toml growth is expected and is part of the phase plan, not a layering accident
- if that growth later turns `lyng-js-ops` into a compile-time or ownership bottleneck, the
  primitive-only subset may be mechanically extracted into `lyng-js-ops_primitive` or an
  equivalent crate without changing semantic ownership

## `lyng-js-types`: Value and Copyable Runtime Types

`lyng-js-types` is a data-only crate. It owns representation, not allocation policy.

Required owned types include:

- `Value`
- typed handles such as `ObjectRef`, `StringRef`, `SymbolRef`, `BigIntRef`, `EnvironmentRef`, `CodeRef`, `RealmRef`
- `ShapeId`
- `PropertyKey`
- `PropertyDescriptor`
- `Completion<T>` and `AbruptCompletion`
- compact feedback-site metadata types that later bytecode/runtime layers reuse

`lyng-js-types` must not own:

- heap allocation APIs
- root registration
- tracing code that reaches into allocator internals
- object-model semantics

Handle dereferencing is therefore not part of `lyng-js-types`.

Rules:

- typed handles are opaque identity tokens plus type information
- equality, copying, hashing, and use as `Value` payloads are owned by `lyng-js-types`
- reading the record behind a handle always goes through `lyng-js-gc`
- later crates that need string contents, symbol descriptions, bigint limbs, or object
  payloads depend on `lyng-js-gc` for that dereference path

### Handle Representation

All typed handles are compact newtypes over `NonZeroU32`.

This is a frozen decision.

Consequences:

- `Option<ObjectRef>` and similar option types stay 32 bits under Rust niche optimization
- handle equality and hashing are cheap integer operations
- handle payloads fit directly in the 32-bit non-double payload space of `Value`
- handle zero is reserved as invalid and never exposed as a live handle

Handle rules:

- each handle family has its own newtype
- a numeric payload is never reinterpreted as another handle type
- handles are stable for the lifetime of the referenced record
- debug builds may carry out-of-band generation or tombstone tracking, but release handle layout stays unchanged

### Value Encoding Contract

`Value` remains a transparent 64-bit word using the NaN-tag-space family frozen in Phase 0.

Phase 2 freezes the logical encoding contract further:

- finite and non-NaN doubles are stored directly as IEEE-754 `f64`
- quiet-NaN space is partitioned into engine tags for all non-double values
- non-double payloads carry:
  - a primary kind tag
  - a 32-bit payload for `Smi(i32)` or a typed-handle value
  - internal sentinel identities

The exact tag constants still belong to implementation, but the logical classes are fixed:

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

`lyng-js-types` must provide fast helper families:

- constructors such as `Value::undefined()`, `Value::from_bool`, `Value::from_smi`
- predicates such as `is_number`, `is_object`, `is_string`, `is_sentinel`
- accessors such as `as_smi`, `as_f64`, `as_object_ref`
- numeric helpers that avoid unnecessary boxing or heap traffic on arithmetic hot paths

Internal sentinels are not guest-visible and are reserved for runtime structure use only.
Required sentinels include:

- array hole
- uninitialized lexical binding
- empty internal slot

The sentinel set is extensible within the same non-double encoding family:

- all sentinels share the sentinel logical class in `Value`
- distinct sentinel identities are carried in sentinel payload bits
- later runtime phases may add sentinels such as dictionary tombstones or
  uninitialized-`this` markers without changing the overall `Value` layout family

### Feedback Metadata Types

Phase 2 does not build feedback vectors yet, but it does freeze the compact ID types that
later phases use.

`lyng-js-types` owns:

- `FeedbackSlotId` or equivalent compact slot/index type
- `BuiltinFunctionId` or equivalent compact builtin-entry identity type used below the
  `lyng-js-builtins` crate boundary
- small helper ID/newtype wrappers needed to address feedback storage without committing to
  the final bytecode-level inline-cache taxonomy

The actual per-code-template feedback arrays and the final inline-cache kind taxonomy
belong to later bytecode/runtime phases.

`BuiltinFunctionId` lives in `lyng-js-types` because runtime function payloads must be able
to store builtin callable identities without depending on `lyng-js-builtins`. Phase 5 then
adds the registry that maps those IDs to concrete handlers and metadata.

## `lyng-js-gc`: Allocation, Rooting, and Tracing

`lyng-js-gc` owns all heap allocation and collection policy.

Phase 2 freezes the collector architecture further than the high-level runtime model:

- storage is page-based and domain-separated
- handles point to stable slots, not raw pointers
- tracing is explicit and typed
- the engine never uses conservative Rust stack scanning

### Storage Domains and Slot Arenas

Each storage concern has its own slot arena:

- strings
- symbols
- bigints
- objects
- environments
- code templates
- shapes

Phase 2 implements only the primitive-owning domains it needs directly, but the allocator
shape is fixed across all domains now.

Each domain uses fixed-capacity pages of homogeneous record type with:

- occupancy metadata
- mark bits
- free-slot discovery state
- contiguous out-of-line payload storage where the record type needs it

Records do not move. Sweep returns dead slots to the domain-local free structure.

### Fragmentation and Size-Class Policy

Non-moving collectors only stay healthy if fragmentation is managed explicitly.

Phase 2 therefore freezes these mitigation rules:

- fixed-size record pages remain homogeneous by domain and record kind
- variable-size payloads such as string code units, bigint limbs, slot buffers, and similar
  out-of-line storage use size-classed side allocations owned by the same domain rather than
  one mixed page format
- page-local free tracking stays domain-owned so fully free pages can be recycled or released
  without a whole-heap compaction step
- fragmentation accounting is part of runtime reporting; the engine measures both live bytes
  and reclaimable free space so non-moving costs stay visible in reports

### Collection Trigger Policy

The initial collector uses a simple explicit trigger policy.

Rules:

- collection is triggered from allocation slow paths, not from arbitrary read-only operations
- a domain that cannot satisfy an allocation from its current free space or budget may
  trigger a collection before growing storage
- after collection, the next allocation budget is derived from live bytes plus a growth factor
- tests and benchmarks may force a collection through explicit test-only or debug APIs;
  correctness must not depend on guessing when implicit collection fires

Phase 2 scaffolding exposes this explicitly with `PrimitiveHeap::force_collect()`,
`PrimitiveHeapView::accounting()`, and rooted `PrimitiveMutator` slow-path collection via
`PrimitiveHeap::mutator_with_roots()`.

### Rooting Model

Take 3 uses explicit rooting from the start.

This is a frozen decision.

Rules:

- GC never scans Rust stack memory conservatively
- any heap handle that must survive a collection across an allocating operation must be
  reachable from:
  - an existing heap object or runtime structure already known to the collector, or
  - an explicit root registration owned by `lyng-js-gc`
- `lyng-js-gc` provides small typed root guards for temporary roots and grouped root scopes
- later VM frames, realm tables, builtin tables, and job queues become collector root sources through the same interface

This keeps lifetime rules explicit and avoids a future rewrite from "raw handles
everywhere" to "hidden root wrappers everywhere."

### Tracing Contract

Tracing is type-directed and centralized.

`lyng-js-gc` owns the tracing interfaces for:

- `Value`
- typed handles
- heap records that contain `Value`
- heap records that contain typed handles directly

Rules:

- tracing a `Value` decodes its logical class and marks the owning domain only when the
  value is a heap reference
- tracing a typed handle marks exactly one storage domain with no dynamic type lookup
- records never trace by string-keyed indirection or trait-object dispatch in hot paths
- cross-domain references are always explicit in record layouts
- all heap mutation paths that may later need a write barrier go through centralized helper
  APIs even while the initial collector treats those helpers as barrier-free fast paths

Weak references, ephemerons, and finalization queues now extend the GC layer without
changing the strong-reference tracing path described here.

### Write Barrier Contract

Take 3 freezes the mutation boundary now so later GC evolution does not require an
engine-wide retrofit of raw store sites.

`lyng-js-gc` owns the barrier-ready helper family for traced heap storage:

- `init_store` or an equivalent helper for writes into freshly allocated records that have
  not yet been published as collector-visible heap state
- `mut_store` or an equivalent helper for overwriting fields in existing traced records
- typed variants of the same helpers for `Value` fields and directly stored typed handles

Rules:

- all stores of `Value` or typed-handle fields into traced heap-owned records must route
  through `lyng-js-gc`-owned store helpers
- helper calls carry the owning traced-record identity plus the written payload so later
  collectors can attach remembered-set or incremental-barrier policy at one boundary
- `init_store` is reserved for freshly allocated records that have not yet become visible to
  the collector or guest execution through any published heap edge
- `mut_store` is required for overwriting fields in existing traced records, including
  object slots, element storage, environment slots, and traced metadata edges
- the initial collector implements both paths as direct writes; the distinction exists so a
  later generational or incremental collector can insert barriers without redesigning APIs
- the barrier-free implementation must compile down to zero-extra-policy overhead in optimized
  builds:
  - no trait-object dispatch
  - no hidden allocation
  - no residual non-inlined helper call on hot store sites
- helper entrypoints should therefore be force-inlined or otherwise demonstrated by benchmark
  or generated-code inspection to reduce to direct stores in the barrier-free collector
- non-heap root storage such as VM registers or explicit root stacks is outside this helper
  family, but any frame-, job-, or context-like structure that becomes a traced heap record
  falls back under the same contract
- no crate other than `lyng-js-gc` defines bypass helpers for traced-edge stores

This contract does not commit take 3 to a nursery, incremental marking, or concurrent
collection. It only freezes the mutation boundary those later collectors would need.

### Mutator Access Policy

Heap access APIs must preserve rooting clarity.

Rules:

- borrowed views into heap storage come from non-allocating APIs that take `&Heap` or an
  equivalent shared borrow of the primitive heap context
- the Phase 2 scaffolding names this split explicitly with a read-only `PrimitiveHeapView`
  and an allocation-capable `PrimitiveMutator`
- any API that takes `&mut PrimitiveContext` or `&mut Heap` is allocation-capable unless
  it is explicitly documented as non-allocating
- borrowed views into page storage must not be held across calls that take
  `&mut PrimitiveContext` or `&mut Heap`
- short-lived field accessors and mutators are allowed, but allocation-capable code paths
  must not hide unrooted heap references
- collector-aware mutation APIs are preferred over exposing raw storage internals
- mutation APIs that write `Value` or typed-handle fields in heap-owned records must route
  through the `init_store` or `mut_store` helper family rather than exposing raw store sites
- metadata fields that carry traced heap edges follow the same rule; the barrier contract is
  not limited to guest-visible property values

This is the intended Rust-level safety shape:

- read-only inspection APIs return views tied to a shared heap borrow
- allocation-capable APIs require a mutable primitive context
- Rust borrow checking then prevents accidental retention of heap views across potential
  collection points

This is primarily a correctness rule, but it also protects code readability by making GC
safety a visible part of API design instead of an ambient assumption.

### Allocation Lifetime Hints

Phase 2 also freezes one small extension point for future generational allocation policy.

Allocation-capable APIs may accept an `AllocationLifetime` hint or an equivalent small enum.

Required semantic classes:

- `Default` for ordinary runtime allocations with no long-lived promise
- `LongLived` for records that are known at creation time to be runtime metadata or
  bootstrap state likely to outlive typical guest objects

Rules:

- the initial allocator may treat both classes identically
- the hint exists so later collectors can route long-lived allocations away from a nursery
  or other short-lived allocation domain without redesigning call sites
- likely `LongLived` examples include realm intrinsics, shapes, code templates, environment
  layouts, and bootstrap-installed runtime structures
- ordinary guest-created objects, strings, arrays, iterator-result objects, and similar
  transient allocations default to `Default` unless a later benchmarked policy proves
  otherwise

### Future Generational Extension Path

Phase 2 does not freeze a nursery design, but it does freeze the only acceptable insertion path.

Rules:

- any future young-generation or nursery policy layers on top of the existing stable-handle
  model rather than replacing it with moving raw-pointer identity
- `Default` allocations are the expected young-generation candidates; `LongLived` exists so
  metadata-heavy or bootstrap allocations can bypass that path
- remembered sets and cross-generation barriers attach to the existing `mut_store` helper family
- explicit roots remain the liveness source for temporary values; no later design may rely on
  conservative Rust stack scanning to make a generational collector workable
- nursery sizing, promotion thresholds, and tenuring heuristics remain benchmark-driven
  implementation policy rather than Phase 2 architecture

## Primitive Heap Records

Phase 2 must fully specify the heap-owned primitive records that later phases depend on.

### String Storage

Runtime strings are immutable flat strings with dual-width storage:

- `Latin1` for strings whose code units fit in `u8`
- `Utf16` for general JS string storage

This is a frozen decision for the initial implementation.

Rationale:

- JS semantic indexing is defined over UTF-16 code units
- one-byte strings avoid wasting space on common ASCII and Latin-1-heavy workloads
- flat storage keeps the first implementation simple and predictable

Each string record stores:

- encoding kind
- code-unit length
- cached hash, computed lazily
- optional cached `AtomId` for strings that have already been atomized
- inline metadata plus out-of-line contiguous code-unit storage

Rules:

- strings are immutable after allocation
- string equality first checks handle identity, then length and cached hash, then code units
- `ToPropertyKey` and other atomizing paths may memoize `AtomId` in the string record
- a cached `AtomId` is a real liveness edge for collectible runtime atoms while the owning
  string record remains live
- primitive-record tracing in `lyng-js-gc` must therefore visit cached string atoms during the
  ordinary mark walk, not through a separate atom-specific retention pass
- the cached hash is deterministic and non-cryptographic; it is for internal hash-table use,
  not adversarial collision resistance
- mixed-encoding operations widen to `Utf16`; `Latin1` is a storage optimization, not a
  semantic distinction visible to higher layers
- ropes, substring views, and external strings are deferred

Phase 2 scaffolding exposes read-only flat-string inspection through `PrimitiveHeapView::string_view()`
and keeps lazy metadata updates explicit through `PrimitiveMutator::cache_string_hash()` and
`PrimitiveMutator::memoize_string_atom()`.

### Atom Table and Atom Lifetime

`AtomId` lives in `lyng-js-common`, but atom lifetime policy is frozen as part of the
primitive runtime architecture.

Atom classes:

- permanent atoms
  - source identifiers and literal-name metadata retained by parsed or compiled code
  - builtin names and well-known strings
  - any atom explicitly retained by long-lived runtime metadata
- collectible runtime atoms
  - atoms created by guest-triggered string atomization such as `ToPropertyKey`
  - other host or runtime-created atoms that are not promoted into permanent metadata

Rules:

- both classes share one `AtomId` namespace and one `AtomTable` API surface
- `AtomTable::intern` remains the permanent/default path used by frontend code, while
  runtime atomization uses an explicit collectible entrypoint
- runtime atomization must preserve full ECMAScript UTF-16 string semantics, including lone
  surrogates, rather than routing through lossy scalar-only text conversion
- if permanent metadata later interns a string that already exists as a collectible atom,
  the existing `AtomId` is promoted rather than duplicated
- runtime-created collectible atoms must be sweepable when no live string cache, shape,
  code template, environment/global binding table, or other runtime structure still carries
  the `AtomId`
- `lyng-js-common` exposes the atom-table visitation and reclamation surface, while `lyng-js-gc`
  drives marking and sweep through that surface during collection
- the collection surface is explicit: start a collection session, visit `AtomId` edges during
  tracing, then sweep unvisited collectible atoms after the mark walk completes
- atom marking is part of the ordinary mark walk:
  traced strings, shapes, code templates, environment metadata, and other `AtomId`-bearing
  records mark their atoms as they are visited rather than through a second semantic pass
- atom sweep runs after the main mark walk has discovered all strong `AtomId` edges
- frontend code only creates permanent atoms
- dynamic property-key workloads must not force immortal atom-table growth
- reclaimed collectible slots may be reused for future runtime atomization once GC has proven
  the old `AtomId` dead

### Symbol Storage

Symbols are immutable identity records.

Each symbol record stores:

- optional description `StringRef`
- symbol class flags such as ordinary versus well-known
- stable identity payload owned by the handle itself rather than by description text

The description is metadata only. Symbol identity never depends on string comparison.
Phase 5 adds the global symbol registry as an explicit GC root source above this storage
layer. That registry does not change the symbol record shape defined here.

Phase 2 scaffolding exposes read-only symbol inspection through `PrimitiveHeapView::symbol_view()`
with explicit class flags and optional borrowed description views.

### BigInt Storage

BigInts are immutable heap records with sign plus magnitude limbs.

Phase 2 freezes the initial magnitude representation to:

- little-endian `u64` limbs on the 64-bit-only take 3 target set
- normalized magnitude with no redundant high zero limbs

Rules:

- zero is represented in one canonical form
- arithmetic algorithms may start with schoolbook implementations
- more advanced multiplication and division algorithms are future optimization work, not a reason to redesign the record shape
- any later small-bigint optimization must happen within heap-record strategy or allocation
  policy, not by introducing a new `Value` logical class

Phase 2 scaffolding exposes read-only bigint inspection through `PrimitiveHeapView::bigint_view()`
with explicit sign, normalized limb count, and borrowed little-endian limb bytes.

## Property Keys and Descriptors

### Property Keys

`PropertyKey` is frozen as:

```rust
enum PropertyKey {
    Index(u32),
    Atom(AtomId),
    Symbol(SymbolRef),
}
```

Rules:

- runtime strings are not a direct `PropertyKey` variant
- converting a string value to a property key atomizes once and then uses `AtomId`
- `Index(u32)` is reserved for canonical array-index keys only
- numeric-looking names outside the array-index range lower to `Atom(AtomId)`
- private names are not `PropertyKey`; they are a later-phase internal-name mechanism

Phase 2 scaffolding exposes:

- read-only string and bigint inspection helpers through `lyng-js-ops::read`
- `to_property_key` through the explicit `PrimitiveContext<'_>` allocating surface
- cached string `AtomId` reuse on repeated property-key conversion rather than repeated
  atom-table interning or heap string comparison

### Property Descriptors

`PropertyDescriptor` is a spec-facing, slow-path structure. It is not the storage format
for ordinary object properties.

The representation must separate:

- field presence bits
- attribute bits
- payload fields

Conceptually:

```rust
struct PropertyDescriptor {
    present: DescriptorPresent,
    attrs: DescriptorAttributes,
    value: Value,
    get: Value,
    set: Value,
}
```

Rules:

- presence bits distinguish "absent" from "present with false/undefined"
- getter and setter fields are stored as `Value` so validation against callable or
  `undefined` remains explicit
- descriptor normalization helpers live in `lyng-js-ops`
- descriptor predicate and completion helpers are pure operations over `PropertyDescriptor`
- descriptor completion returns a normalized copy; callers that still need the original
  field-presence shape keep the raw descriptor alongside the completed one
- object shapes later store compact normalized attribute metadata rather than full descriptor structs

## Completion Model

`Completion<T>` is an explicit runtime concept, but the Rust surface is frozen to a
`Result`-shaped API for ergonomics.

Conceptually:

```rust
type Completion<T> = Result<T, AbruptCompletion>;

enum AbruptCompletion {
    Throw(Value),
    Return(Value),
    Break(Option<AtomId>),
    Continue(Option<AtomId>),
}
```

Rules:

- `Ok(T)` is the Rust representation of a normal completion
- `Err(AbruptCompletion)` is the Rust representation of an abrupt completion
- labels use `AtomId`, never heap strings
- `Throw` carries the thrown value directly
- `Completion<T>` is for spec-facing helpers and slow paths, not for the interpreter hot loop
- the Rust API deliberately uses `Result` shape so abstract-operation code can use `?`
  without custom control-flow machinery
- later VM execution uses bytecode control flow and exception state directly, but public
  semantic helpers still use the same completion types

## `lyng-js-ops`: Primitive Abstract Operations

`lyng-js-ops` is the single owner of primitive ECMA-262 abstract operations.

Phase 2 freezes two design rules:

- primitive semantics are not reimplemented ad hoc in builtins, VM dispatch, or object code
- `lyng-js-ops` itself is layered by operation kind so future object-aware operations can
  be added without changing primitive ownership

### Operation Families

Required primitive families include:

- type predicates and classification helpers
- primitive conversions
- primitive comparisons and equality semantics
- property-key conversion helpers
- descriptor predicates and normalization helpers

### API Shape Categories

Primitive operations fall into three API categories:

1. Pure operations
   - no heap access
   - examples: `to_boolean`, `same_value`, `same_value_zero`,
     `is_strictly_equal`, and numeric classification helpers over the tagged
     `Value` surface
2. Read-only primitive operations
   - need heap reads but no allocation
   - examples: string equality, bigint sign and magnitude inspection
3. Allocating primitive operations
   - may allocate strings or bigints, or may atomize
   - examples: `to_string`, `to_property_key`, number-to-string formatting

Heap-aware primitive operations must take explicit heap or primitive-context parameters
rather than reaching into global state.

The split is intentional:

- pure helpers operate only on the 8-byte `Value` representation plus typed handles
- heap-backed string-content and bigint-magnitude inspection stays in the read-only family
- higher-level comparison or conversion work composes those layers instead of smuggling heap
  reads into the pure helper surface
- Phase 2 conversion helpers cover primitive inputs only; object-aware coercion wrappers such as
  `ToPrimitive`-driven `ToNumber` or `ToString` growth land later without changing these owners

The context shape is frozen to a concrete borrowed facade, not a trait object:

```rust
struct PrimitiveContext<'a> {
    heap: &'a mut Heap,
    atoms: &'a mut AtomTable,
}
```

Rules:

- allocating operations take `&mut PrimitiveContext`
- read-only heap-aware operations take `&Heap` or an equivalent shared-borrow view
- `PrimitiveContext` is defined in the primitive runtime layer and later constructed from
  the runtime/agent state
- callers are responsible for projecting the `Agent`-owned heap and atom table into a
  `PrimitiveContext` when crossing from runtime code into `lyng-js-ops`
- tests may build the same concrete context around a small standalone heap harness
- Phase 2 does not use dynamic-dispatch allocation traits on hot semantic paths

Phase 2 scaffolding organizes `lyng-js-ops` by family with `pure`, `read`, and `allocating`
modules so later object-aware helpers can grow without collapsing back into one monolithic crate
surface.

### Future Layering Rule

When later phases add object-aware abstract operations:

- new `lyng-js-ops` modules may depend on `lyng-js-env` and `lyng-js-objects`
- `lyng-js-objects` exposes low-level internal methods and descriptor primitives
- `lyng-js-objects` must not depend back on `lyng-js-ops`

This keeps the crate DAG acyclic while preserving one semantic owner for ECMA-262 operations.

## Performance and Memory Invariants

Phase 2 non-negotiables:

- `Value` stays 8 bytes
- typed handles stay 32-bit copyable values
- `Option<Handle>` stays niche-optimized and compact
- primitive allocation and tracing paths avoid string-keyed maps and trait-object dispatch
- runtime strings are immutable and contiguous
- descriptor and completion structs are treated as cold semantic structures, not hot object-layout payloads
- string atomization can memoize back to the string record to avoid repeated hashing on hot property-key paths
- runtime atomization must not create unbounded immortal memory growth

## Deferred Work

This document intentionally does not specify:

- ordinary object instance layout beyond the handle and key contracts already frozen
- environment-record storage beyond the handle and GC contracts already frozen
- object-aware abstract operations such as `Get`, `Set`, or `Call`
- weak-reference and finalization machinery
- ropes, substring views, external strings, or small-bigint specialization

Those belong to later phases and must build on the primitives defined here rather than
replacing them.
