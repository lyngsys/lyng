# Lyng JS Builtin Bootstrap

This document describes the concrete bootstrap, builtin-installation, and native-call
architecture for the default JS3 runtime environment. Later builtin work extends the same
bootstrap path rather than inventing parallel installation machinery.

This note is intentionally bootstrap-scoped. Runtime storage, object layout, and bytecode
execution are specified elsewhere:

- [Architecture Overview](architecture.md)
- [Runtime Model](runtime-model.md)
- [Runtime Substrate](runtime-substrate.md)
- [Bytecode and VM](bytecode-and-vm.md)
- [Engineering Standards](engineering-standards.md)

## Goals

- freeze one bootstrap path for the default realm, global object, and foundational intrinsics
- handle constructor or prototype cycles without ad hoc post-init patches scattered through the codebase
- make builtin call dispatch explicit and fast without forcing crate cycles between `lyng-js-vm`
  and `lyng-js-builtins`
- ensure builtin installation is table-driven and descriptor-exact
- keep spec builtins, host additions, and harness additions clearly separated

## Crate Ownership and Dependency Shape

Phase 5 owns the builtin-facing layer above the runtime substrate.

```text
lyng-js-builtins
  depends on lyng-js-common, lyng-js-types, lyng-js-gc, lyng-js-ops,
             lyng-js-objects, lyng-js-env, lyng-js-host

lyng-js-vm
  may depend on lyng-js-builtins for builtin-call dispatch

lyng-js-builtins
  must not depend back on lyng-js-vm
```

Rules:

- `lyng-js-builtins` owns bootstrap tables, builtin descriptors, and builtin implementations
- `lyng-js-vm` owns the act of dispatching a builtin call from an executing frame
- builtin implementations operate through a narrow builtin-call context and do not read or
  mutate VM frame internals directly
- `BuiltinFunctionId` is owned by `lyng-js-types`; `lyng-js-builtins` owns only the registry that
  maps those IDs to handlers and metadata
- any service that a builtin needs from execution machinery, such as dynamic function-source
  compilation, is exposed through that narrow context rather than by making `lyng-js-builtins`
  depend on `lyng-js-vm`

## Bootstrap Model

Realm bootstrap is a cold path, but it must still be deterministic and typed.

Phase 5 freezes a multi-pass bootstrap:

1. create the realm-owned global object and global environment shell
2. allocate intrinsic skeleton objects for constructors and prototypes that participate in cycles
3. wire the core prototype graph and constructor links
4. populate the realm intrinsic table with the allocated handles
5. install builtin methods, accessors, and data properties from typed descriptor tables
6. install global object properties and global bindings
7. run host or harness extension hooks after spec bootstrap is complete

The bootstrap code may be cold and table-driven, but the final object graph must not depend
on deferred string-keyed lookups or "fix it later" mutation spread across unrelated modules.

## Post-Core Snapshot Compatibility

Phase 5 still assumes live table-driven bootstrap on realm creation. Startup snapshotting is
not part of the core gate.

The bootstrap architecture must nevertheless remain snapshot-compatible:

- a later serialized default-realm image or lazy intrinsic-materialization scheme must produce
  the same intrinsic graph, descriptor attributes, and builtin entry identities as live bootstrap
- typed descriptor tables and intrinsic records remain the source of truth; a snapshot is a
  cache of bootstrap results, not a second semantic definition
- snapshot load paths must not bypass the typed bootstrap ownership boundaries in a way that
  forces builtin installers, object creation helpers, or realm records to diverge

This keeps post-core startup work open without turning Phase 5 into a snapshot project.

## Intrinsic Skeleton Pass

Some builtin families are cyclic and cannot be constructed in one linear pass.

Phase 5 requires a skeleton-allocation pass for at least:

- `Object.prototype`
- `Function.prototype`
- `Object`
- `Function`
- `Error.prototype`
- native error prototypes and constructors owned by Phase 5

Rules:

- skeleton allocation creates objects with the correct coarse kind and placeholder internal links
- `Function.prototype` is itself allocated as a callable builtin function object, not as an
  ordinary prototype shell
- prototype and constructor references are patched during the link pass, not by ad hoc late mutation
- the realm intrinsic table is only considered ready after the link pass completes

This avoids scattered special cases for `Object`/`Function` cycles and the error-family
inheritance chain.

## Intrinsics Table Contract

The realm `Intrinsics` record remains an explicit typed struct.

Phase 5 requires concrete fields for:

- `object`
- `object_prototype`
- `function`
- `function_prototype`
- `boolean`
- `boolean_prototype`
- `symbol`
- `symbol_prototype`
- `error`
- `error_prototype`
- native error constructors and prototypes:
  - `EvalError`
  - `RangeError`
  - `ReferenceError`
  - `SyntaxError`
  - `TypeError`
  - `URIError`
- well-known symbols reachable from the agent or realm as required by installed builtins

Rules:

- later intrinsic families extend this struct in place
- missing later entries remain explicit optional fields or placeholders
- builtin code does not rediscover constructors or prototypes by looking up names on the global object

## Descriptor Table Model

Builtin installation is table-driven.

Each builtin family owns typed static tables describing:

- target object or intrinsic field
- property key as `AtomId`, `PropertyKey`, or well-known symbol identifier
- property kind:
  - data value
  - accessor pair
  - builtin function
- attributes:
  - writable
  - enumerable
  - configurable
- optional function metadata such as name and length

Rules:

- builtin modules describe descriptors declaratively and share one installer path
- installation uses exact spec attributes; builtin code does not hand-roll `define_property`
  calls for every member
- keys are atom-backed or typed well-known symbols, not string literals looked up at runtime
- descriptor tables may be split per family for readability, but they all feed the same
  installation machinery

## Native Builtin Call ABI

Builtin functions use a uniform native call ABI.

Conceptually:

```rust
fn builtin(
    cx: &mut dyn BuiltinCallContext,
    this_value: Value,
    args: &[Value],
    new_target: Option<ObjectRef>,
) -> Completion<Value>
```

The exact Rust signature may use generics or concrete context types, but the semantic split
is frozen:

- builtin code receives explicit `this`
- arguments are presented as a slice-like view, not by poking VM registers
- constructor-sensitive builtins receive explicit `new_target`
- abrupt completion still uses the engine-wide `Completion<Value>` contract

The `dyn` trait form above is conceptual. The shipping implementation may use a concrete or
generic context type on hot builtin paths if profiling shows trait-object dispatch is too
expensive.

`BuiltinCallContext` must provide only the services builtin code actually needs, such as:

- access to the current realm and agent
- allocation helpers for ordinary objects, strings, symbols, and errors
- access to shared abstract operations in `lyng-js-ops`
- an object-operation context for proxy-observable operations rather than ad hoc
  `lyng-js-objects` internal-method access
- descriptor-install helpers used by bootstrap paths
- a narrow dynamic-function compilation hook needed by the `Function` constructor

Builtin code must not become a back door into VM internals.

## Builtin Entry Identity

Builtin callable identities must be storable in runtime function payloads without making the
runtime substrate depend on `lyng-js-builtins`.

Phase 5 freezes this pattern:

- builtin entry identities are compact typed IDs defined in `lyng-js-types`
- function payloads store those IDs as part of their callable entry identity
- `lyng-js-builtins` owns the registry mapping entry ID to:
  - handler function
  - name and length metadata
  - constructor capability
  - bootstrap descriptor information when relevant

This lets `lyng-js-objects` and `lyng-js-env` store callable identities without depending on
the crate that implements the builtin bodies.

## Builtin ID Namespaces

Phase 5 keeps builtin entry identities in one `BuiltinFunctionId` space, but the space is
partitioned by contract.

- reserved JS3-internal helper namespace
  - owned in `lyng-js-types`
  - currently `1_001..=1_012`
  - used only for lowering helpers and runtime template support still emitted by the compiler or VM
  - not a spec-visible builtin surface and not a compatibility promise
- public Phase 5 builtin namespace
  - owned by `lyng-js-builtins`
  - currently `2_001..=2_040`
  - used for spec-visible constructors, prototype methods, and global-install tables
- harness extension namespace
  - owned by the explicit harness extension pass on top of `lyng-js-builtins`
  - currently `3_001..=3_002`
  - used for `$262.evalScript` and `$262.createRealm`
  - not part of spec bootstrap and not part of the default CLI realm

Rules:

- dispatch may route all three namespaces through the same builtins-owned bridge
- namespace membership, not call site, determines whether an entry is internal helper,
  spec-visible builtin, or harness extension
- new spec builtins extend the public Phase 5 namespace; they do not reuse or rename
  `js3_internal_*` entries
- harness helpers remain a post-bootstrap extension layer even when they share descriptor
  installation machinery with spec builtins

## Foundational Family Boundaries

This document focuses on the bootstrap pattern that first installed the foundational
families and still underpins the broader builtin set that JS3 now ships.

### Global Object

The default global object must install at minimum:

- `globalThis`
- `Infinity`
- `NaN`
- `undefined`
- constructor properties for the builtin families installed in the realm

Rules:

- `globalThis` is the actual realm global object, not a wrapper object
- default realms do not install non-standard globals
- host or harness additions are a later extension pass, not part of spec bootstrap
- global property attributes are installed from descriptor tables, not inferred from object kind

The default global object now carries the foundational bootstrap surface plus later global
functions such as `eval`, numeric parsing helpers, and URI helpers through the same
descriptor-table installation path.

### `Object`

Phase 5 `Object` coverage includes the constructor and the prototype operations needed for
property descriptors, extensibility, and prototype manipulation on top of the Phase 3
substrate.

Required scope includes:

- constructor semantics
- `Object.create`, including `Object.create(null)` and prototype-less objects
- prototype access and mutation
- descriptor queries and definition
- integrity-level operations
- own-key and own-property helper surface needed by early conformance slices
- `Object.prototype.toString`
- `Object.prototype.valueOf`
- `Object.prototype.hasOwnProperty`
- `Object.prototype.isPrototypeOf`
- `Object.prototype.propertyIsEnumerable`
- `Object.prototype.constructor`

Iterator-heavy helpers and later convenience APIs can wait for later builtin phases.

### `Function`

Phase 5 installs the `Function` constructor, `Function.prototype`, and the callable
utilities required for builtin and source-defined functions to feel like ECMAScript
functions.

Required scope includes:

- `Function.prototype`
- `call`
- `apply`
- `bind`
- `toString` with engine-defined but stable source formatting rules
- `Function` constructor semantics

`bind` requires BoundFunction support as a function-payload variant:

- bound functions store the bound target, bound `this`, and bound argument prefix
- `[[Call]]` and `[[Construct]]` dispatch forward through the bound target with prepended arguments
- this is a Phase 5 extension of the existing function-payload branch, not a redesign of object kind dispatch

The `Function` constructor compiles source strings through the existing frontend and
compiler pipeline, but:

- it closes over the target realm's global environment, not the caller's lexical scope
- it uses a narrow dynamic-compilation service exposed through the builtin call context
- it does not imply that direct or indirect `eval` is implemented

`Function.prototype.toString` uses the Phase 4 source-retention contract:

- source-defined functions read their original source slice through retained `SourceId` and range metadata
- builtin-native functions format as stable native-function strings
- the engine does not synthesize pretend source text for source-defined functions when the
  original source slice is available

### `Boolean`

Phase 5 installs `Boolean` constructor and prototype behavior for primitive boxing and
wrapper objects.

This family is intentionally narrow and should reuse shared coercion operations rather than
re-encoding boolean semantics in builtin methods.

### `Symbol`

Phase 5 installs:

- `Symbol`
- `Symbol.prototype`
- well-known symbol values required by installed builtins
- the global symbol registry for `Symbol.for` and `Symbol.keyFor`

The Phase 5 well-known symbol set is explicitly:

- `Symbol.hasInstance`
- `Symbol.toPrimitive`
- `Symbol.toStringTag`

Deferred to later phases:

- `Symbol.iterator`
- `Symbol.asyncIterator`
- `Symbol.species`
- collection, regexp, and resource-management related well-known symbols not used by Phase 5 families

The global symbol registry is an agent-owned strong root source. Symbols reachable from that
registry are not collectible while the registry entry exists.

## Progressive `ToObject` Completion

`ToObject` is introduced as a public abstract operation before the full builtin universe exists.

Phase rules:

- Phase 4 `ToObject` must already handle object identity and null or undefined errors
- Phase 5 completes wrapper-object creation for the primitive families it installs directly,
  notably `Boolean` and `Symbol`
- wrapper closure for `Number`, `String`, and `BigInt` primitives completes with their own
  builtin families in later phases

This keeps the abstract-operation entrypoint stable while making its primitive-wrapper surface
grow in the same order as the installed intrinsic prototypes.

### `Error` and Native Errors

Phase 5 completes the error-family bootstrap promised by earlier phases.

Required scope includes:

- `Error`
- `Error.prototype`
- native error constructors and prototypes in the Phase 5 family set
- message and name properties with spec-appropriate defaults

Rules:

- Phase 4 realm-aware error helpers now allocate actual error objects through the Phase 5
  intrinsic table
- stack traces remain host or implementation details unless standardized behavior is required
- builtin code and VM handlers continue to throw through `Completion<Value>`; only the backing
  object creation becomes fully bootstrapped here
- message formatting is centralized in cold-path helpers that build heap-owned runtime strings
  from static templates plus runtime interpolation where needed, although later
  implementations may delay actual `.message` string materialization until observation

## Function and Error Creation Helpers

Phase 5 centralizes two cold but high-value helpers:

- builtin function-object creation
- error-object creation

The creation path must:

- allocate the correct object kind and payload
- assign the right realm, prototype, callable entry identity, and constructor flags
- install standard non-enumerable metadata such as `name`, `length`, and `.prototype` when required
- route all common error creation through one realm-aware helper family

Later builtin families must reuse these helpers rather than open-coding constructor or error
allocation.

## Host and Embedding Extension Pass

Host additions happen after spec bootstrap through embedding-owned extension providers.

Phase 5 freezes these rules:

- the default realm bootstrap produces a spec-only global surface
- the engine exposes generic embedding hooks for follow-up realm extension installation
- external conformance embeddings such as `tools/lyng-js-test262` may install a `$262` surface on top of those hooks
- `$262` is not part of the intrinsic table and not part of the default CLI realm
- `$262.createRealm()` uses the same default bootstrap pipeline to create a fresh realm rather
  than a special embedding-only realm shape
- host helpers such as cross-realm evaluation, realm creation, GC exposure, or job draining live
  behind explicit generic embedding points
- other `$262` helpers continue to ride the same embedding-extension hooks as features such
  as agents, binary-data helpers, or async job coordination evolve

This keeps conformance behavior separate from the engine bootstrap contract.

## CLI Contract

`lyng-js-cli` is a thin embedding of the engine, not a second semantic owner.

Rules:

- the CLI creates a default realm through the same bootstrap entrypoint used elsewhere
- the default CLI does not inject Node-style or browser-style globals
- uncaught exceptions are reported through the same error objects produced by the engine
- the CLI drains jobs only through the host/runtime job API, not by touching internal queues directly

The CLI is a validation surface, not a privileged runtime mode.

## Performance and Memory Invariants

Phase 5 non-negotiables:

- bootstrap is cold and may use table-driven descriptors freely
- builtin call dispatch in hot code paths uses compact builtin entry IDs, not string-keyed lookups
- builtin implementations call shared abstract operations rather than duplicating coercion or object semantics
- builtin implementations route `Get`, `Set`, `HasProperty`, `GetOwnProperty`,
  `DefineOwnProperty`, prototype operations, and own-key collection through
  `lyng-js-ops::object` when proxy traps are observable
- global and intrinsic lookup during bootstrap are typed and direct, not stringly
- later builtin families extend the same registration and creation helpers instead of inventing parallel installers

## Deferred Work

This document intentionally does not require:

- `eval`
- numeric parsing globals and URI globals
- `Array` constructor and `Array.prototype`
- iterator helpers and iterator protocol builtins
- modules, promises, generators, async functions, or Intl

Those belong to later phases and must build on the bootstrap and builtin-call contracts fixed
here.
