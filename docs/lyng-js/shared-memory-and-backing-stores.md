# Lyng JS Shared Memory and Backing Stores

This document describes the ownership and lifetime model for backing stores and shared-memory
coordination. The same runtime shape now underpins the landed binary-data, `SharedArrayBuffer`,
and `Atomics` work without needing a second storage or coordination system.

Related notes:

- [Architecture Overview](architecture.md)
- [Runtime Model](runtime-model.md)
- [Runtime Substrate](runtime-substrate.md)
- [ECMA-262 Completion](ecma262-completion.md)
- [Engineering Standards](engineering-standards.md)

## Goals

- freeze one ownership model for `ArrayBuffer`, `SharedArrayBuffer`, typed-array, and
  `DataView` backing stores
- define which runtime layer owns cross-agent coordination
- define the host boundary for agent creation, shared-buffer sharing, and parking
- make the GC interaction explicit before binary-data and shared-memory phases widen

## Core Model

Take 3 uses two different storage categories:

- agent-local GC objects
  - `ArrayBuffer` wrapper objects
  - `SharedArrayBuffer` wrapper objects
  - typed-array and `DataView` view objects
- cluster-owned backing stores
  - the raw byte allocations that buffers and views point at

This split is frozen.

Rules:

- wrapper objects are ordinary agent-local heap records traced by that agent's collector
- backing stores are not reclaimed by any one agent heap's mark-sweep pass
- shared backing stores use atomic lifetime management outside single-agent tracing
- the same backing-store abstraction is used by both non-shared and shared buffer families,
  with metadata describing sharedness, detachment, and size behavior

## Backing-Store Lifetime

Backing-store lifetime is not owned by any one `Agent`.

Rules:

- backing stores use explicit lifetime management outside any single agent GC walk
- shared backing stores use atomic reference counting or an equivalent `Arc`-like lifetime
  model
- non-shared backing stores may use the same owner type or a cheaper specialization, but they
  still remain outside any one agent-local GC arena
- the backing-store owner lives in `AgentCluster` or an equivalent cluster-owned shared
  coordination record, not in an agent-local GC arena
- local JS wrapper objects increment or hold backing-store references through typed backing-store
  handles owned by the runtime
- the backing store is freed only when the last cross-agent or local runtime reference drops

This does not freeze the exact Rust container type. A custom backing-store allocation is
acceptable if it preserves the same ownership and atomic-lifetime model.

## Detachment and Sharedness

Detachment and sharedness are not interchangeable.

Rules:

- detachable `ArrayBuffer` instances use buffer-local detachment state
- `SharedArrayBuffer` instances are never detached
- sharedness is metadata on the backing-store record, not an ad hoc property on wrapper objects
- typed-array and `DataView` semantics read detachment or sharedness through the backing-store
  and view metadata, not through duplicated object-local flags

## Agent Thread Affinity

Each `Agent` remains single-threaded.

Rules:

- an `Agent` runs on one host thread at a time
- the runtime should treat `Agent` as thread-affine and non-shareable by default
- agent-local heaps, realms, execution contexts, and job queues do not cross thread boundaries
- only cluster-owned coordination state and backing stores are designed to be shared across threads

This means take 3 is not building a shared-heap concurrent GC. It is building single-threaded
agents plus shared backing stores and cluster-owned coordination.

## Agent Creation and Host Boundary

The engine does not own thread creation policy.

Rules:

- the host creates or schedules OS threads or equivalent execution resources
- the host asks the engine to create new agents within an existing or new `AgentCluster`
- the engine owns agent and cluster records; the host owns the policy for when they are started
- host boundaries exist for:
  - creating agents
  - transferring detachable `ArrayBuffer` ownership through structured clone or equivalent host
    messaging without making detachment a host-owned semantic decision
  - sharing `SharedArrayBuffer` backing-store handles across agents through structured clone or
    equivalent host messaging
  - parking and unparking threads for `Atomics.wait`, `Atomics.waitAsync`, and `Atomics.notify`

The host does not own queue ordering, wait-queue contents, or ECMAScript memory semantics.

## Wait-Queue Ownership

`Atomics.wait` and related operations need futex-like coordination.

Rules:

- the wait-queue data structure is engine-owned at the cluster layer
- wait locations are keyed by a cluster-owned backing-store identity plus byte offset or an
  equivalent typed wait-location key
- queue membership, wakeup selection, and observable ordering are engine-owned semantics
- the host provides the blocking or wakeup primitive used to actually park a thread, but it does
  not own the queue data structure itself
- `Atomics.waitAsync` extends the same wait-location ownership model and later integrates with
  the waiting agent's job queue

## Atomic Access Contract

Shared-memory atomics require an explicit low-level access contract.

Rules:

- typed atomic operations are performed against backing-store bytes through a narrow,
  documented `unsafe` boundary
- alignment, bounds, typed-width, and sharedness checks are validated before crossing that
  boundary
- the `unsafe` layer exists to perform correctly aligned typed atomic loads, stores,
  read-modify-write operations, and compare-exchange over backing-store bytes
- the high-level engine APIs above that boundary remain typed and spec-traceable

Take 3 does not need to freeze the exact helper names now, but it does freeze the existence
of one narrow ownership point for unsafe atomic byte access.

## GC Interaction

GC remains per-agent.

Rules:

- one agent's GC pause does not stop other agents by design
- agent-local wrapper objects are traced and collected by their owning agent
- backing stores are not traced by single-agent mark-sweep as ordinary heap records
- weak references and finalization later extend the GC layer, but they do not change the
  cluster-owned backing-store lifetime model
- cluster-owned shared state must remain accessible without depending on a single agent's GC lock

## Current Status

- the `Runtime` -> `AgentCluster` -> `Agent` topology is the live ownership model
- `ArrayBuffer`, `SharedArrayBuffer`, typed arrays, and `DataView` all use the same
  backing-store ownership described here
- `SharedArrayBuffer`, `Atomics`, parking, and multi-agent validation close on top of the
  same cluster-owned wait-queue and backing-store model

## Invariants

- `Agent` is single-threaded and thread-affine
- backing stores are cluster-owned, not agent-GC-owned
- shared backing stores use atomic lifetime management
- `SharedArrayBuffer` is never detached
- wait queues are engine-owned at the cluster layer
- host code provides parking and thread policy, not shared-memory semantics
