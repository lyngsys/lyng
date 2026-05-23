# Lyng JS Shared Memory And Backing Stores

Binary data is stored outside ordinary object slots. Wrapper objects stay agent-local, while
the byte storage for array buffers and shared array buffers is owned by the runtime
substrate.

## Core Model

The engine uses `BackingStoreRef` for byte storage referenced by:

- `ArrayBuffer`
- `SharedArrayBuffer`
- typed-array views
- `DataView`

Backing stores carry byte length, sharedness, detachment state for non-shared buffers, and
the raw byte allocation. Typed-array and `DataView` objects hold view metadata over a
backing store: byte offset, byte length, element kind, and viewed buffer reference.

## Ownership

Ordinary wrapper objects are agent-local GC objects. The byte allocation is managed by the
runtime substrate so it can outlive an individual wrapper object and, for shared buffers,
be referenced through cluster-owned coordination.

`ArrayBuffer` detachment changes buffer metadata. `SharedArrayBuffer` is not detachable.
Sharedness is a backing-store property, not a wrapper-object property.

## Agent Boundary

The current agent execution model is thread-affine. Shared backing stores are the explicit
cross-agent data structure. Agent-local heaps do not own shared byte allocations.

## Atomic Access

Atomic operations validate:

- the buffer is shared where required
- the view is attached
- the element type supports the requested atomic operation
- the index is in range
- the operation uses the correct byte width and alignment semantics

Atomic reads, writes, and read-modify-write behavior live in shared-memory operation
helpers rather than in individual builtin dispatch functions.

## GC Interaction

Wrapper objects trace their references to viewed buffers and view metadata. Backing-store
byte allocations use explicit lifetime management outside a single agent heap's mark-sweep
walk. The cluster owns shared backing-store coordination.

## Current State

The runtime has typed backing-store IDs, backing-store records, ArrayBuffer and
SharedArrayBuffer object families, typed-array and DataView dispatch, detachment checks,
sharedness checks, and Atomics dispatch metadata.

## Invariants

- Backing-store bytes are not ordinary object slots.
- Shared byte storage is cluster-owned.
- Detachment is represented on non-shared buffers.
- Atomic behavior is centralized in shared-memory operations.
