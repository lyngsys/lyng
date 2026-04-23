# Lyng JS Runtime Substrate

This document describes the concrete architecture for the runtime layer above primitive
values and below bytecode execution: runtime and agent ownership, realms, execution
contexts, environment records, object storage, shapes, internal-method dispatch,
foundational function objects, and host integration boundaries.

This note builds on the earlier runtime notes:

- [Architecture Overview](architecture.md)
- [Runtime Model](runtime-model.md)
- [Runtime Primitives](runtime-primitives.md)
- [Shared Memory and Backing Stores](shared-memory-and-backing-stores.md)
- [Engineering Standards](engineering-standards.md)

## Goals

- freeze the crate DAG for `lyng-js-env`, `lyng-js-objects`, and `lyng-js-host`
- define the runtime-owned records that later VM, builtin, and module work will reuse
- keep normal object access on dense slot paths and lexical access on direct register or
  environment-slot fast paths
- centralize internal-method ownership before builtins and compiler code start calling into it
- fix the host boundary now so later async and module features extend it instead of replacing it
- freeze the `Runtime`/`AgentCluster`/`Agent` split before shared-memory work arrives

## Crate Ownership and Dependency DAG

Phase 3 owns three crates:

```text
lyng-js-host
  depends on lyng-js-common, lyng-js-types

lyng-js-objects
  depends on lyng-js-common, lyng-js-types, lyng-js-gc

lyng-js-env
  depends on lyng-js-common, lyng-js-types, lyng-js-gc, lyng-js-objects, lyng-js-host
```

Ownership is intentionally asymmetric:

- `lyng-js-objects` owns object storage, shapes, element storage, and internal methods
- `lyng-js-env` owns `Runtime`, `Agent`, realms, execution contexts, environment records,
  and runtime-owned job queues
- `lyng-js-host` owns host-facing traits and request or response types only

This is the frozen answer to the Phase 3 layering question:

- `lyng-js-objects` does not depend on `lyng-js-env`
- object payloads that need realm or environment identity store `RealmRef` or
  `EnvironmentRef` handles from `lyng-js-types`
- `lyng-js-env` depends on `lyng-js-objects` because `Agent` physically owns the object and
  shape runtime
- `lyng-js-host` is an interface crate and does not depend on `lyng-js-env` or `lyng-js-objects`
- later object-aware abstract operations in `lyng-js-ops` depend on `lyng-js-env` and
  `lyng-js-objects`, not the other way around

That crate DAG keeps the runtime acyclic while still giving one semantic owner to each
layer.

## `lyng-js-env`: Runtime, Agent, Realms, and Environments

`lyng-js-env` is the owning crate for runtime-wide mutable state.

Required owned types include:

- `Runtime`
- `AgentCluster`
- `Agent`
- `Realm`
- `ExecutionContext`
- `EnvironmentLayoutId` and `EnvironmentLayout`
- environment-record families behind `EnvironmentRef`
- runtime-owned job records and job queues

### Runtime and Agent Ownership

The public embedding entrypoint is `Runtime`.

`Runtime` owns:

- the root `AgentCluster`
- the installed `HostHooks`

The host boundary is intentionally cold-path and may use boxed dynamic dispatch:

- `Runtime` owns `Box<dyn HostHooks>` or an equivalent vtable-backed hook object
- host-hook dispatch is not part of interpreter hot loops
- VM and builtin code must not route normal property access, arithmetic, or lexical access
  through host callbacks

`AgentCluster` owns the state that must be shared across one or more ECMAScript agents
once shared memory exists:

- the agent table
- shared backing-store coordination used by `SharedArrayBuffer`
- parked-agent and wakeup bookkeeping used by `Atomics.wait`, `Atomics.waitAsync`, and `Atomics.notify`
- wait queues keyed by shared-memory wait locations

Cluster ownership rules:

- `AgentCluster` owns the shared coordination records; no single `Agent` owns or mutates the
  cross-agent wait-queue tables as private state
- shared backing-store allocations use atomic lifetime management and may outlive any single
  agent-local wrapper object
- the wait-queue data structure is engine-owned even when the host provides the blocking or
  wakeup primitive used to park threads

`Agent` owns the runtime state that must remain agent-local:

- GC heaps from `lyng-js-gc`
- atom table and related atomized runtime state
- object and shape runtime from `lyng-js-objects`
- realm table
- execution-context stack
- runtime-owned job queues
- code-template and well-known-symbol state as later phases require

The engine is initially one single-threaded `Agent` inside one `AgentCluster`. Phase 3 does
not introduce cross-thread synchronization into the hot path, but it does freeze the
cluster-owned shared-memory coordination boundary now so later phases do not retrofit it.

Thread-affinity rules:

- an `Agent` runs on one host thread at a time
- the runtime should treat `Agent` as thread-affine and non-shareable by default
- moving or sharing agent-local heaps, realms, execution contexts, or job queues across threads
  is outside the initial design

### Realms and Intrinsics

Each realm is a first-class runtime record.

Every realm stores:

- `global_object: ObjectRef`
- `global_env: EnvironmentRef`
- `intrinsics: Intrinsics`
- realm-level flags and host data hooks needed by later phases

`Intrinsics` is an explicit struct of typed fields. It is not a string-keyed registry.

Rules:

- Phase 3 freezes the typed-table approach, not the final field list
- Phase 5 extends the intrinsic field set in place for concrete builtins
- missing later intrinsic entries are represented as explicit optional fields or placeholder
  slots, not by falling back to string lookups
- realms may later be multiplied, but the initial engine boots one default realm first

### Execution Context Stack

Execution contexts are runtime records owned by `lyng-js-env`, not incidental VM locals.

Each `ExecutionContext` stores:

- `realm: RealmRef`
- `executable: ExecutableId`
- `lexical_env: EnvironmentRef`
- `variable_env: EnvironmentRef`
- `private_env: Option<EnvironmentRef>`
- current `this` or `new.target` state needed by later VM and builtin code
- context kind flags such as script, module, function, eval, or job

`ExecutableId` is an extensible runtime enum, not an untyped payload.

Phase 3 freezes the category split, not every later variant payload:

- `Script`
- `Module`
- `Builtin`
- `Bytecode(CodeRef)`
- later execution kinds may extend the enum in place

Rules:

- the execution-context stack lives in `Agent`
- pushing and popping contexts is owned by `lyng-js-env`, even when later triggered by VM calls
- the VM may cache top-of-stack context fields for speed, but `lyng-js-env` remains the
  semantic owner of the context model
- the context stack is cold control state, not a substitute for bytecode call frames

### Environment Layout Contract

Environment layouts are immutable metadata records that connect frontend scope analysis to
runtime slot storage.

`EnvironmentLayout` stores at minimum:

- layout kind
- slot count
- stable binding order as produced by Phase 1 sema
- per-slot flags such as mutable, lexical, or needs-TDZ
- optional binding atoms for diagnostics, debugging, and slow-path lookups

Rules:

- environment layouts are created from compiler output later, but the record shape is frozen now
- runtime environment instances point to a layout rather than storing binding metadata inline
- layout slot order must exactly match the deterministic sema order defined in Phase 1
- declarative runtime lookup uses slot indices, not name maps, on normal paths

### Environment Record Families

Phase 3 freezes the core environment-record families and their runtime payloads.

#### Declarative Environments

Declarative environments store:

- `outer: Option<EnvironmentRef>`
- `layout: EnvironmentLayoutId`
- dense slot storage

Rules:

- slots use the Phase 2 internal uninitialized sentinel for TDZ
- binding access by name is a slow-path helper for debugging or dynamic constructs only
- normal compiled lexical access uses the slot index from the layout
- writes to environment slot storage route through the Phase 2 `mut_store` helper family
- environment construction and bootstrap initialization may use the Phase 2 `init_store`
  helper only before the new environment record is published as reachable heap state

#### Function Environments

Function environments extend declarative environments with function-specific runtime state.

Required payload:

- declarative base fields
- `function_object: ObjectRef`
- `this_binding_status: ThisBindingStatus`
- `this_value: Value`
- `new_target: Option<ObjectRef>`
- `home_object: Option<ObjectRef>`

`ThisBindingStatus` is a small explicit enum:

- `Lexical`
- `Uninitialized`
- `Initialized`

Rules:

- arrow functions use `Lexical`
- derived-constructor `this` starts `Uninitialized`
- the function environment is the semantic owner of `this` and `new.target`
- writes to `this_value`, `new_target`, `home_object`, and any other traced heap-edge fields
  in function environments follow the same Phase 2 `init_store` / `mut_store` contract as
  other traced heap storage
- execution contexts may cache function-environment state but do not replace it

#### Global Environments

Global environments are specialized records, not a generic declarative environment.

Required payload:

- `global_object: ObjectRef`
- declarative binding storage for global lexical declarations
- `var_names` membership structure keyed by atoms for global var and function declaration bookkeeping

Rules:

- lexical globals live in dense slots, not on the global object
- var and function globals route through the global object path
- the global environment is the only place where these two binding domains are composed
- `var_names` is a set-like lookup structure, not an ordered list

#### Object Environments

Object environments are the slow-path binding form used for constructs like `with`.

Required payload:

- `binding_object: ObjectRef`
- `with_environment: bool`
- `outer: Option<EnvironmentRef>`

Rules:

- object environments do not own declarative slot arrays
- name resolution delegates to the bound object through object operations
- `with_environment` is retained so later `@@unscopables` semantics can extend the same record shape

#### Module and Private Environments

The runtime carries distinct private-environment and module-environment record families:

- private environments reuse dense slot storage but stay typed separately from ordinary
  declarative environments
- bytecode execution contexts and function payloads may carry `private_env` independently of
  the lexical-environment chain
- module environments reuse dense slots plus import-binding metadata

### Jobs

The runtime owns job queues for promises, modules, async resumption, and related engine work.

Phase 3 freezes these rules:

- job queues are stored in `Agent`
- queued jobs are runtime records, not host-owned closures
- host callbacks may request or observe work, but the queue data structure and drain order
  remain engine-owned

This prevents later promise and module work from inventing a second scheduling model.

### Shared-Memory Coordination

Phase 3 freezes the ownership boundary for later shared-memory work even though the feature
family does not become executable until Phase 6.

Rules:

- wait queues for `Atomics.wait`, `Atomics.waitAsync`, and `Atomics.notify` are owned by
  `AgentCluster`, not by the host and not by any one `Agent`
- wait locations are keyed by a cluster-owned backing-store identity plus byte offset or an
  equivalent typed wait-location key
- host code provides the OS-thread parking or wakeup primitive, but the engine owns queue
  membership, wakeup selection, and observable ordering policy
- shared backing stores are exposed across agents through cluster-owned backing-store handles,
  not through raw pointers leaked out of one agent's local heap

## `lyng-js-objects`: Objects, Shapes, Elements, and Internal Methods

`lyng-js-objects` owns the full object substrate and all internal-method dispatch.

Required owned types include:

- object records behind `ObjectRef`
- shape records behind `ShapeId`
- named-slot and element storage records
- ordinary internal-method implementations
- foundational function-object payloads

### Object Record Shape

Every object record has a compact hot header with:

- `kind: ObjectKind`
- `flags: ObjectFlags`
- `prototype: Option<ObjectRef>`
- `shape: ShapeId`
- named-slot storage reference
- indexed-element storage reference
- optional private-slot storage reference used by class brands and private fields

Kind-specific cold metadata lives out of line.

Rules:

- the hot header stays small and fixed-layout
- function or other exotic payload data is not stuffed into the hot header
- ordinary objects with no exotic payload pay only the header plus slot and element storage references
- private-field state uses the same traced slot-buffer substrate as named slots and elements rather
  than a parallel symbol-emulation path
- class-owned private metadata lives in object-runtime side tables keyed by the class constructor
  and prototype, while per-object installed brands remain explicit runtime metadata records

### Object Flags

`ObjectFlags` must at minimum include:

- `EXTENSIBLE`
- integrity-level summary bits needed to avoid rediscovering sealed or frozen state
- object-specific slow-path flags such as dictionary-mode named properties

Flags summarize runtime state. They do not replace descriptor checks where the spec
requires them.

### Named Properties and Shapes

Named properties remain shape-based from day one.

Every shape stores:

- parent shape
- transition key
- property metadata entry
- slot count
- property count
- stable enumeration-order metadata
- small-shape inline descriptor data plus a flattened property table once the checked-in
  property-count threshold is crossed

Every shape property entry stores:

- `PropertyKey`
- slot offset
- normalized descriptor attributes
- property kind: data or accessor
- enumeration index

Rules:

- adding a property follows deterministic transitions
- accessor properties consume two contiguous value slots for getter and setter
- very small shapes may use inline descriptor scan, but larger stable shapes consult the
  flattened property table rather than walking transition history linearly on every lookup
- shape lookup is allocation-free on steady-state shapes
- redefining a data property as an accessor property, or the reverse, is a slow-path event
  that may force dictionary mode instead of rewriting the slot layout of existing shape families
- deletion and highly dynamic redefinition may force dictionary mode instead of complex
  reverse transitions
- shapes are shared metadata; objects do not own copies of property descriptors

### Shape and Prototype Invalidation

Shape-based execution support requires an explicit invalidation contract before inline caches
arrive in Phase 4.

Rules:

- prototype mutations, property deletion or redefinition on cached prototype paths,
  dictionary-mode transitions, and other shape-family-invalidating events route through
  centralized `lyng-js-objects` mutation paths
- those mutation paths update explicit invalidation state owned by the object substrate, such
  as epochs, dependency records, or watchpoint-style tokens
- Phase 4 inline caches consume that invalidation state rather than inventing VM-local
  prototype-mutation heuristics
- the initial implementation may invalidate conservatively, including flushing affected cache
  families, but the invalidation owner remains `lyng-js-objects`

### Named Slot Storage

Named-slot storage is dense `Value` storage addressed by the shape slot offset.

Rules:

- ordinary reads are `shape -> slot offset -> slot load`
- ordinary writes are `shape -> slot offset -> slot store`
- slot buffers may grow or be replaced during shape transitions, but stable-property
  reads do not perform name lookup
- slot buffers never expose internal sentinels to guest-visible semantics
- slot stores route through the Phase 2 `mut_store` helper family so future write barriers
  can be inserted without auditing every property write site again
- object construction, builtin bootstrap, and shape-transition installation paths may use
  the Phase 2 `init_store` helper only before the new slot buffer becomes published heap
  state

### Indexed Elements

Indexed elements are a separate storage concern from named properties.

Phase 3 freezes three element modes:

- `Empty`
- `Dense`
- `Sparse`

Dense elements store:

- contiguous `Value` storage
- initialized length or logical span metadata
- capacity metadata

Rules:

- dense elements support the array-hole sentinel
- sparse elements are dictionary-style slow-path storage
- sparse element entries carry enough normalized attribute metadata to support indexed
  property semantics without pretending they are named-property shapes
- named-property and indexed-element operations stay separate even when they are surfaced
  through the same object internal method
- element stores on existing arrays route through the Phase 2 `mut_store` helper family
- array construction and other fill-before-publication paths may use the Phase 2
  `init_store` helper for freshly allocated element storage

### Dictionary Fallback

Dictionary mode is the explicit slow path for named properties.

Phase 3 freezes these rules:

- repeated add-delete churn may force dictionary mode
- deleting shaped properties may prefer dictionary fallback over heroic shape surgery
- dictionary entries store normalized attributes plus payload data directly
- dictionary mode must not contaminate the fast path for shape-stable objects
- the initial implementation uses one explicit checked-in fallback threshold for named-property
  churn rather than ad hoc heuristics spread across call sites
- threshold changes are benchmarked and recorded because they affect cache invalidation,
  object memory behavior, and prototype-watchpoint pressure

### Internal-Method Dispatch

`lyng-js-objects` owns internal-method dispatch and ordinary object semantics.

Required internal-method families include:

- `[[GetPrototypeOf]]`
- `[[SetPrototypeOf]]`
- `[[IsExtensible]]`
- `[[PreventExtensions]]`
- `[[GetOwnProperty]]`
- `[[DefineOwnProperty]]`
- `[[HasProperty]]`
- `[[Get]]`
- `[[Set]]`
- `[[Delete]]`
- `[[OwnPropertyKeys]]`
- `[[Call]]`
- `[[Construct]]`

Rules:

- dispatch is centralized by `ObjectKind`
- per-object trait objects or Rust trait impl dispatch are not used on the hot object path
- a `match` on `ObjectKind` or an equivalent static kind table is acceptable
- ordinary-object fast paths are explicit and readable
- later Proxy support extends this dispatch layer instead of replacing it

`lyng-js-objects` owns internal methods. Later `lyng-js-ops` object-aware abstract
operations call into this layer rather than duplicating ordinary semantics.

### Internal Methods and Abstract-Operation Fragments

The crate DAG intentionally prevents `lyng-js-objects` from depending on `lyng-js-ops`.

That means some internal methods must inline narrow abstract-operation fragments rather
than calling back up through public `lyng-js-ops` entrypoints.

Allowed inlined fragments include:

- callability checks and direct `[[Call]]` dispatch needed by ordinary getter and setter invocation
- direct `[[Get]]` or related internal-method dispatch needed by ordinary construction paths
- other internal-method-local helper steps where routing through `lyng-js-ops` would create a crate cycle

Rules:

- these helpers stay private to `lyng-js-objects`
- later public object-aware `lyng-js-ops` wrappers must delegate into the same
  `lyng-js-objects` semantics rather than reimplementing ordinary behavior differently
- if a spec change or bug fix affects one of these inlined fragments, the matching
  `lyng-js-ops` wrapper behavior must stay equivalent

This is not license for broad semantic duplication. It is a narrow DAG-preserving escape
hatch for internal-method-local logic only.

### Function-Object Foundations

Function objects are a coarse object kind with an out-of-line function payload.

Required function payload fields include:

- `realm: RealmRef`
- `environment: EnvironmentRef`
- `private_env: Option<EnvironmentRef>`
- `this_mode`
- `home_object: Option<ObjectRef>`
- constructor capability flags
- function-kind flags such as arrow, class-constructor, generator, async, and async-generator
- callable entry identity for:
  - builtin native entrypoints keyed by `BuiltinFunctionId` from `lyng-js-types`
  - later bytecode-backed code templates
  - bound-function forwarding payloads added in Phase 5

Rules:

- builtins and bytecode functions share the same coarse object kind
- the function payload is the bridge point between Phase 3 objects and later Phase 4 or Phase 5 execution work
- call and construct dispatch first branch on `ObjectKind::Function`, then on function-payload data
- bound functions land in Phase 5 as a function-payload variant required by `Function.prototype.bind`
- other later callable exotics still extend the same base function-object shape rather than redesigning it
- Phase 3 tests exercise `[[Call]]` and `[[Construct]]` through builtin-style native entrypoints
  and harness callables owned by the substrate
- bytecode-backed callable payload variants may already exist as reserved entry identities, but
  become executable only once Phase 4 wires them to compiled code and VM dispatch

## `lyng-js-host`: Host Boundary

`lyng-js-host` is the interface crate for host integration.

It owns:

- `HostHooks`
- typed request or response structs for host interaction
- host error types

Phase 3 freezes the boundary categories, not every later method:

- source and module loading requests
- uncaught-exception and diagnostic reporting
- host-observable job and promise integration hooks needed by later phases
- harness-facing injection points for test262 and CLI integration
- agent creation and thread-start requests used by later multi-agent hosts
- structured-clone or equivalent host boundaries that can transfer detachable `ArrayBuffer`
  ownership without making detachment host-owned semantics
- structured-clone or equivalent host boundaries that can share `SharedArrayBuffer` backing-store
  handles across agents
- host parking and wakeup primitives used by `Atomics.wait`, `Atomics.waitAsync`, and `Atomics.notify`

Rules:

- host interactions are cold path and may use trait-object dispatch
- host hooks do not own queue order, realm state, or object semantics
- host code never becomes the semantic owner of ECMAScript algorithms

## Cross-Phase Contracts

### Compiler to Environment Contract

Phase 4 compiler work must emit environment layouts that match the Phase 1 sema slot order.

Consequences:

- `lyng-js-env` owns runtime environment layout records
- compiler code creates or references those records instead of inventing a new layout format
- runtime slot access assumes the compiler respected Phase 1 sema ordering

### Objects to Ops Contract

The public object-aware `lyng-js-ops` modules build on the `lyng-js-objects` substrate and
provide semantic entrypoints such as:

- `Get`
- `Set`
- `Call`
- `Construct`
- `HasProperty`
- `CreateDataProperty`
- `ToObject`

Rules:

- `lyng-js-objects` provides the substrate those operations wrap
- object-aware `lyng-js-ops` modules depend on `lyng-js-env` and `lyng-js-objects`
- those wrappers call into shared internal-method and runtime primitives rather than inventing
  a second object semantic layer
- bootstrap-dependent operations such as primitive-wrapper `ToObject` continue to extend the
  same object-aware `lyng-js-ops` surface

### Error-Value Contract

`AbruptCompletion::Throw(Value)` is the stable throw shape even before builtin error objects
are fully bootstrapped.

Phase rules:

- Phase 3 internal methods may throw values through the standard completion channel, but do not
  require the full intrinsic error-constructor graph yet
- Phase 4 adds minimal realm-aware helpers for common spec-thrown errors such as `TypeError`
  and `RangeError`
- Phase 5 connects those helpers to fully bootstrapped intrinsic error objects and constructors
  without changing the throw propagation type or control flow

This avoids a pre-Phase-5 placeholder throw model that would later require rewriting every
error-producing path.

### Builtins to Realm Contract

Phase 5 builtin bootstrap fills in realm intrinsics and global-object structure, but it
does not invent a new realm representation. The concrete bootstrap mechanics are specified
in [Builtin Bootstrap](builtin-bootstrap.md).

Consequences:

- the intrinsic table remains a typed struct
- global bootstrap works by populating realm-owned fields and global bindings
- builtin function creation uses the function-object foundation frozen here

### VM to Context Contract

Phase 4 VM work executes against:

- the `ExecutionContext` model from `lyng-js-env`
- environment-record families from `lyng-js-env`
- internal-method dispatch from `lyng-js-objects`

The VM may cache hot fields, but it must not become the semantic owner of those structures.

## Performance and Memory Invariants

Phase 3 non-negotiables:

- normal lexical access uses dense slot arrays, not name maps
- normal named-property access uses `PropertyKey`, shape lookup, and slot offsets
- internal-method dispatch avoids per-object trait-object indirection
- object headers stay compact and keep cold payloads out of line
- host callbacks stay off the interpreter hot path
- dictionary objects, sparse elements, object environments, `with`, and direct `eval`
  remain explicit slow paths
- dictionary fallback thresholds and invalidation ownership stay centralized in
  `lyng-js-objects`; the VM does not grow its own object-shape heuristics

## Deferred Work

This document intentionally defers:

- Proxy trap semantics beyond the dispatch hook point already frozen
- full array exotic length behavior
- typed-array, ArrayBuffer, DataView, and module-namespace object internals
- module-environment import binding semantics
- private-environment storage details
- promise reaction jobs and async scheduling details

Those later phases must extend the substrate defined here rather than replacing it.
