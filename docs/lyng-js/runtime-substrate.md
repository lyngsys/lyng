# Lyng JS Runtime Substrate

The runtime substrate is the set of crates that make bytecode execution possible:
environment records, objects, shapes, backing stores, realms, agents, jobs, host hooks,
and operation contexts.

## Crate Ownership

- `lyng-js-env` owns runtime, cluster, agent, realm, execution context, environment, module,
  symbol, backing-store, and job structures.
- `lyng-js-objects` owns object records, shapes, property storage, indexed elements,
  receiver payloads, primitive wrappers, private elements, and ordinary internal methods.
- `lyng-js-host` owns embedding hooks and host-facing traits.
- `lyng-js-ops` owns semantic operation APIs that coordinate values, objects,
  environments, descriptors, proxies, promises, shared memory, and Temporal helpers.

## Runtime, Cluster, And Agent

`Runtime` is the embedding root. It owns host hooks and an `AgentCluster`.

`AgentCluster` owns agent records and shared backing-store coordination. Shared memory is
cluster-owned because the backing allocation can be referenced by more than one agent.

`Agent` owns per-agent mutable JavaScript state:

- atom table and well-known names
- heaps and typed storage domains
- shape table
- realms
- code templates
- execution contexts
- job queues
- VM-owned execution state

Agents are thread-affine in the current embedding model.

## Realms

Realms hold global object state, intrinsic objects, builtin caches, and host extension
installations. Realm bootstrap is performed by `lyng-js-builtins`; the environment crate
owns the storage and references that make the realm reachable from the runtime.

## Execution Contexts

Execution contexts carry realm, function/script/module state, lexical environment,
variable environment, private environment, and `this` binding information. The VM pushes
and pops contexts as scripts, modules, functions, eval code, and builtin calls execute.

## Environment Records

The substrate represents ECMA-262 environment record families explicitly:

- declarative environments
- function environments
- global environments
- object environments
- module environments
- private environments

Sema and compiler metadata decide where bindings live. Runtime environment records store
captured bindings, globals, object-backed dynamic scope, module bindings, and private-name
state.

## Objects And Shapes

Objects combine a compact header, object kind, prototype reference, shape ID, named slots,
indexed elements, and optional object-specific payload. Shapes describe named property
layout and support shape-guarded access.

Named property storage uses slots when the shape is compact. Dictionary storage handles
objects that need less regular property behavior. Indexed elements are separate from named
properties and support dense and sparse representations.

Internal methods are implemented by object kind and routed through operation contexts when
guest code can observe traps, receivers, prototype traversal, or abrupt completion.

## Function Objects

Function objects connect object records to code references, realm information, environment
captures, builtin IDs, embedding IDs, constructor behavior, and call/construct metadata.
The VM owns call-frame execution. Builtins own native builtin dispatch. Object records own
the function object payload shape.

## Host Boundary

The host crate supplies traits and records for:

- job scheduling
- module loading and resolution
- dynamic import
- embedding native functions
- realm extension installation

Host hooks are explicit inputs to runtime operations. The parser, compiler, and object
storage crates do not call host APIs directly.

## Cross-Crate Contracts

- The compiler lowers lexical facts into registers, environment slots, global plans, and
  dynamic lookup sites.
- The VM calls environment and object operation APIs instead of duplicating their semantics.
- Builtins install into realm-owned storage and call shared operations for coercion,
  descriptors, object access, promises, Temporal helpers, and errors.
- Host extensions are installed through realm extension providers and embedding function
  metadata.

## Invariants

- Runtime substrate records are typed and domain-owned.
- Cross-agent bytes live behind cluster-owned backing stores.
- Proxy-observable behavior goes through operation contexts.
- Environment records reflect ECMA-262 record families rather than one string map.
