# Lyng JS Builtin Bootstrap

`lyng-js-builtins` creates the default realm builtin surface and dispatches native builtin
functions. It owns builtin metadata, descriptor tables, realm builtin caches, bootstrap
entrypoints, and public/internal builtin dispatch.

## Crate Ownership

The builtins crate owns:

- builtin registry entries
- public builtin IDs and metadata
- internal helper builtin IDs and metadata
- descriptor table shapes
- family installers
- realm builtin caches
- bootstrap request/result types
- native builtin call ABI
- dispatch from builtin ID to Rust handler

It does not own VM dispatch, object storage, environment storage, or host hooks.

## Bootstrap Flow

Realm bootstrap creates intrinsic skeletons, installs constructor/prototype relationships,
defines properties from descriptor metadata, installs global bindings, and records builtin
handles in `RealmBuiltins`.

The bootstrap entrypoints are:

- `bootstrap_default_realm`
- `bootstrap_realm`
- `BuiltinBootstrap`
- `BootstrapRequest`
- `BootstrapArtifacts`

`BootstrapMode` selects the installation mode needed by engine entrypoints and tests.

## Family Installers

Builtin families are organized by semantic owner:

- primitives
- globals
- objects and object reflection
- functions
- arrays
- strings
- collections
- iterators
- promises
- errors
- date
- JSON
- RegExp
- modules
- binary data
- Temporal
- prototype links
- intrinsics

Family installers create and wire their own constructor/prototype/global properties while
sharing descriptor helpers and realm caches.

## Descriptor Tables

Descriptor metadata represents:

- target object
- property key
- data or accessor value
- attributes
- builtin function identity
- length/name metadata
- intrinsic references

Descriptor tables are data-oriented so bootstrap code can install properties consistently
without hand-encoding attributes at each call site.

## Native Builtin Call ABI

Builtin dispatch uses:

- `BuiltinCallContext`
- `BuiltinInvocation`
- `BuiltinResult`
- `BuiltinHandler`
- `PublicBuiltinDispatchContext`
- `InternalBuiltinDispatchContext`

Handlers receive explicit context and invocation data. Guest-visible errors return through
the engine completion model.

## Public And Internal IDs

Public builtin IDs represent spec-visible builtin functions. Internal builtin IDs represent
compiler/VM helper entrypoints such as function call helpers, dynamic import helpers,
RegExp literal helpers, and constructor guards.

The two lanes are distinct so public builtin metadata and internal lowering helpers cannot
alias each other.

## Host And Embedding Extensions

Embedding functions use `EmbeddingFunctionId` and realm extension providers in the VM/host
surface. Builtin bootstrap can install extension-provided functions into a realm without
making them part of the core builtin namespace.

## Invariants

- Builtin families own their descriptor metadata and handlers.
- Bootstrap mutates realm/object state through shared helper APIs.
- Builtins call shared abstract operations for coercion, object access, promises,
  descriptors, errors, Temporal helpers, and shared memory.
- Internal helper builtins are not public ECMAScript builtins.
