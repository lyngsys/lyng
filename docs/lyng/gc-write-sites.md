# Lyng JS GC Allocation And Write-Site Audit

This audit began as the baseline for `lyng-16x3` under the JIT-ready GC plumbing epic.
`lyng-3gn5` updated the strong heap-record write sites to route through `HeapWriter`.
It documents heap allocation entrypoints and writes that store `Value` or typed heap handles
(`ObjectRef`, `StringRef`, `SymbolRef`, `BigIntRef`, `EnvironmentRef`, `CodeRef`,
`RealmRef`, `ShapeId`, and slot-buffer handles) into records that are heap-owned or traced
as runtime roots.

No engine behavior is changed by this document. The guard script at
`tools/check-lyng-js-gc-write-sites.sh` reads the allowlist at the end of this file and
fails when a new direct write matching the checked patterns appears without updating this
audit.

## Barrier Tags

- `Barrier: yes` means the site overwrites or installs a heap edge that a future
  generational or incremental collector must route through a barrier-capable API when the
  owning record can already be reachable.
- `Barrier: init-only` means the site initializes a freshly allocated record or buffer
  before publication. The allocation path must stay auditable, but an old-to-young write
  barrier is not expected for the initial store itself.
- `Barrier: no-root` means the site is an agent, VM, or host-side root table rather than a
  heap-resident record. It must stay in root tracing, but it is not an object remembered-set
  write.
- `Barrier: weak-special` means the edge belongs to weak or ephemeron state and will need
  weak-table-specific treatment instead of the ordinary strong-reference write barrier.

## Allocation Entry Points

### `crates/lyng-js/gc`

- `PrimitiveHeap::{alloc_string, alloc_latin1_concat_string, alloc_utf16_concat_string}` in
  `crates/lyng-js/gc/src/arena.rs`: low-level string record allocation, including flat
  side payloads and cons-string child refs. Barrier: init-only.
- `PrimitiveHeap::{alloc_symbol, alloc_bigint, alloc_value_cell}` in
  `crates/lyng-js/gc/src/arena.rs`: low-level symbol, bigint, and mutable value-cell
  allocation. Barrier: init-only.
- `PrimitiveHeap::{alloc_object, alloc_function_payload, alloc_object_slots}` in
  `crates/lyng-js/gc/src/arena.rs`: low-level object record, function payload, and
  object slot-buffer allocation. Barrier: init-only.
- `PrimitiveHeap::{alloc_suspended_execution, alloc_suspended_registers}` in
  `crates/lyng-js/gc/src/arena.rs`: generator/async suspension records and copied register
  buffers. Barrier: init-only.
- `PrimitiveHeap::{alloc_environment, alloc_environment_slots}` in
  `crates/lyng-js/gc/src/arena.rs`: low-level environment records and dense binding slots.
  Barrier: init-only.
- `PrimitiveHeap::{alloc_code, alloc_code_slots}` in `crates/lyng-js/gc/src/arena.rs`:
  installed code records and constant slots. Barrier: init-only.
- `PrimitiveHeap::{alloc_realm, alloc_shape}` in `crates/lyng-js/gc/src/arena.rs`:
  low-level realm and shape records. Barrier: init-only.
- `PrimitiveMutator::alloc_*` in `crates/lyng-js/gc/src/mutator.rs`: public allocation
  chokepoints that run collection-before-growth checks and delegate to `PrimitiveHeap`.
  Future nursery allocation policy belongs here first. Barrier: init-only.
- `ValueSlotAllocator::allocate` in `crates/lyng-js/gc/src/arena/storage.rs`: allocates
  object, environment, code, and suspended-register value buffers filled with one `Value`.
  Barrier: init-only.

### `crates/lyng-js/objects`

- `ObjectRuntime::{alloc_shape, root_shape, transition_shape}` in
  `crates/lyng-js/objects/src/runtime.rs`: shape allocation and side metadata creation.
  Shape parent and prototype guard are initialized in the `RuntimeShapeRecord`; transition
  metadata is side-table state. Barrier: init-only for heap records, no-root for transition
  maps.
- `ObjectRuntime::{alloc_object, alloc_date_object}` in
  `crates/lyng-js/objects/src/runtime.rs`: object allocation owner that allocates named
  slots, indexed elements, optional value-cell payloads, and optional function payloads
  before publishing `ObjectRef`. Barrier: init-only.
- `ObjectAllocation` and `FunctionObjectData` builder methods in
  `crates/lyng-js/objects/src/object_records.rs` and
  `crates/lyng-js/objects/src/functions.rs`: pre-allocation request data. Barrier: init-only.
- Private/shared class storage allocation in `crates/lyng-js/objects/src/private_storage.rs`:
  grows object private slot buffers and writes method/accessor/field values through
  `PrimitiveMutator` store APIs. Barrier: yes for overwrites, init-only for copied slots.

### `crates/lyng-js/env`

- `Agent::alloc_runtime_string`, `alloc_string_for_atom`, Latin-1 short-string caches, and
  two-code-unit string cache in `crates/lyng-js/env/src/agent/symbols.rs`: agent-level
  string allocation wrappers. Barrier: init-only; cache table updates are no-root.
- `Agent::{alloc_declarative_environment, alloc_private_environment,
  alloc_module_environment, alloc_function_environment, alloc_global_environment,
  alloc_object_environment}` in `crates/lyng-js/env/src/agent/environments.rs`: environment
  record allocation wrappers over `PrimitiveMutator`. Barrier: init-only.
- `Agent::alloc_environment_layout` in `crates/lyng-js/env/src/agent/environments.rs`:
  layout metadata allocation. It stores atoms and flags, not heap object edges. Barrier:
  no-root.
- Promise side-table allocation in `crates/lyng-js/env/src/promise.rs`: promise,
  reaction, capability, resolving-function, finally-function, combinator, and combinator
  element records. Barrier: no-root today because `AgentPromiseTables` is traced from the
  agent snapshot, but these writes must stay centralized if promise records move into heap
  storage.
- Disposal side-table allocation in `crates/lyng-js/env/src/disposal.rs`: disposal
  capabilities, async operations, and async resumes. Barrier: no-root today for the same
  agent-side-table reason.

### `crates/lyng-js/vm`

- `Vm::install_functions` and `Vm::install_constants` in
  `crates/lyng-js/vm/src/vm/install.rs`: installed code allocation, code constants, and
  parent-code backpatching. Barrier: yes for parent-code mutation; init-only for code
  constants allocated before publication.
- String and bigint helpers in `crates/lyng-js/vm/src/vm/values.rs` and
  `crates/lyng-js/vm/src/extensions.rs`: VM-facing wrappers over string and bigint
  allocation. Barrier: init-only.
- `Vm::snapshot_suspended_execution` in `crates/lyng-js/vm/src/vm/generators.rs`: copies
  register values into a freshly allocated suspended-register buffer, then allocates the
  suspended execution record. Barrier: init-only for the snapshot writes.
- Feedback vector allocation in `crates/lyng-js/vm/src/vm/feedback.rs`: VM side table keyed
  by installed `CodeRef`. Barrier: no-root today; entries contain object/shape cache facts
  and must remain traced or invalidated when they become heap-owned.

## Strong Heap Record Write Sites

The strong heap-record writes below are routed through `HeapWriter::write_ref` or
`HeapWriter::write_value` from the `PrimitiveMutator` store path. Mutating writes also run
the active incremental Dijkstra barrier from the writer boundary: when the owning heap
record has already been marked in the current major mark phase, the newly written strong
referent is shaded through the active marker before the mark is finalized. Generational
old-to-young card marking remains in the owning `PrimitiveHeap` store helper so the common
non-incremental path can return before touching marker state.

### Value Slot Buffers

- `ValueSlotAllocator::write` in `crates/lyng-js/gc/src/arena/storage.rs` writes one
  `Value` into object slot buffers, environment slot buffers, code slots, or suspended
  register buffers through `HeapWriter::write_value`. Barrier: yes for `mut_store_value`; init-only for current
  `init_store_value` and suspended snapshot writes.

### Function Payload Records

- `set_function_payload_home_object`, `set_function_payload_environment`, and
  `set_function_payload_private_env` in `crates/lyng-js/gc/src/arena/store_helpers.rs`
  route typed handle writes through `HeapWriter::write_ref`. Barrier: yes.

### Symbol And Value-Cell Records

- `set_symbol_description` in `crates/lyng-js/gc/src/arena/store_helpers.rs` routes the
  description handle through `HeapWriter::write_ref`. Barrier: yes.
- `set_value_cell_value` and `set_value_cell_linked_string` in
  `crates/lyng-js/gc/src/arena/store_helpers.rs` route value and linked-string writes
  through `HeapWriter`. Barrier: yes.

### Object Records

- `set_object_prototype`, `set_object_shape`, `set_object_named_slots`,
  `set_object_elements`, and `set_object_private_slots` in
  `crates/lyng-js/gc/src/arena/store_helpers.rs` route typed handle writes through
  `HeapWriter::write_ref`. Barrier: yes.

### Environment Records

- `set_environment_outer`, `set_environment_function_object`, `set_environment_this_value`,
  `set_environment_new_target`, and `set_environment_home_object` in
  `crates/lyng-js/gc/src/arena/store_helpers.rs` route typed handle and `Value` writes
  through `HeapWriter`. Barrier: yes.
- `Agent::{init_environment_slot, set_environment_slot, set_function_this_binding,
  set_function_new_target, set_function_home_object}` in
  `crates/lyng-js/env/src/agent/environments.rs` route environment slot and function-env
  writes through `PrimitiveMutator`. Barrier: yes for mutation, init-only for initialization.

### Code, Realm, And Shape Records

- `set_code_parent` and `set_code_realm` in
  `crates/lyng-js/gc/src/arena/store_helpers.rs` route typed handle writes through
  `HeapWriter::write_ref`. Barrier: yes.
- `set_realm_global_object`, `set_realm_global_env`, `set_realm_bootstrap_code`, and
  `set_realm_root_shape` in `crates/lyng-js/gc/src/arena/store_helpers.rs` route typed
  handle writes through `HeapWriter::write_ref`. Barrier: yes.
- `set_shape_parent` and `set_shape_prototype_guard` in
  `crates/lyng-js/gc/src/arena/store_helpers.rs` route typed handle writes through
  `HeapWriter::write_ref`. Barrier: yes.

## Weak Or Ephemeron Write Sites

- `WeakMapState::set` in `crates/lyng-js/gc/src/weak.rs` writes `WeakHeapRef -> Value`
  entries. Barrier: weak-special.
- `WeakSetState::insert` in `crates/lyng-js/gc/src/weak.rs` writes weak heap refs.
  Barrier: weak-special.
- `WeakRefState` and finalization-registry state are initialized and mutated through
  `PrimitiveMutator` methods in `crates/lyng-js/gc/src/mutator.rs` and
  `crates/lyng-js/gc/src/arena/weak_state.rs`. Barrier: weak-special.

## Object And Shape Side-Metadata Write Sites

- `NamedPropertyDictionary::upsert` in `crates/lyng-js/objects/src/object_metadata.rs`
  writes data/accessor `Value` payloads and symbol-bearing property keys into object
  dictionary metadata. Barrier: yes, because this metadata is logically object payload.
- `ObjectRuntime::{transition_elements_to_sparse_payload, store_sparse_element_payload}` in
  `crates/lyng-js/objects/src/runtime_storage.rs` writes sparse indexed element payloads.
  Barrier: yes, because sparse metadata is logically object payload.
- `ObjectRuntime::{root_shape, transition_shape}` in
  `crates/lyng-js/objects/src/runtime.rs` writes shape-cache and transition metadata.
  Barrier: no-root for current side tables; if shape transitions move into heap records,
  transition writes become `Barrier: yes`.
- `FunctionObjectData::with_environment` in `crates/lyng-js/objects/src/functions.rs`
  updates pre-allocation function metadata. Barrier: init-only.

## Environment, Module, Promise, And Disposal Side Tables

- `ModuleRecord` setters in `crates/lyng-js/env/src/module_records.rs` write code,
  environment, namespace, import-meta, resolved-export, and evaluation-error edges.
  Barrier: no-root today because module records are agent-rooted and traced from
  `AgentCollectionSnapshot`.
- `Agent::set_module_binding_alias` and `Agent::global_set_lexical_binding` in
  `crates/lyng-js/env/src/agent/environments.rs` write environment refs into environment
  side metadata. Barrier: no-root today; if metadata becomes heap-owned, these become
  `Barrier: yes`.
- Promise mutation methods in `crates/lyng-js/env/src/promise.rs` write promise results,
  capabilities, reactions, resolving/finally function records, and object-to-record indexes.
  Barrier: no-root today through `AgentPromiseTables` tracing.
- Disposal mutation methods in `crates/lyng-js/env/src/disposal.rs` write disposable
  resource records, pending errors, and object-to-record indexes. Barrier: no-root today
  through `AgentDisposalTables` tracing.
- `Agent::keep_weak_target_alive` in `crates/lyng-js/env/src/agent/weak_finalization.rs`
  writes weak keep-alive roots. Barrier: no-root.

## VM Side Tables And Feedback

- `NamedPropertyFeedback` and `KeyedPropertyFeedback` mutations in
  `crates/lyng-js/vm/src/vm/feedback.rs` write object/shape cache entries and dense-index
  cache facts into VM side tables. Barrier: no-root today; future heap-owned feedback
  vectors need a barrier or invalidation policy.
- `Vm::ensure_feedback_site_execution` in `crates/lyng-js/vm/src/vm/feedback.rs` allocates
  the feedback vector side table for one installed code record. Barrier: no-root.

## Guard Allowlist

The following entries are consumed by `tools/check-lyng-js-gc-write-sites.sh`. Each line is
`path<TAB>regex`. Keep the prose above and this list in sync.

<!-- gc-write-site-allowlist:start -->
crates/lyng-js/gc/src/weak.rs	self[.]entries[.]insert[(]key, value[)];
crates/lyng-js/gc/src/weak.rs	self[.]entries[.]insert[(]value[)];
crates/lyng-js/objects/src/object_metadata.rs	self[.]entries[.]insert[(]key, entry[)];
crates/lyng-js/objects/src/runtime.rs	self[.]root_shapes[.]insert[(]key, id[)];
crates/lyng-js/objects/src/runtime.rs	properties[.]push[(]property[)];
crates/lyng-js/objects/src/runtime_storage.rs	entries[.]insert[(]index, SparseElementEntry::new[(]payload, attrs[)][)];
crates/lyng-js/objects/src/functions.rs	self[.]environment = environment;
crates/lyng-js/env/src/module_records.rs	self[.]code = code;
crates/lyng-js/env/src/module_records.rs	self[.]environment = environment;
crates/lyng-js/env/src/module_records.rs	self[.]namespace = namespace;
crates/lyng-js/env/src/module_records.rs	self[.]deferred_namespace = namespace;
crates/lyng-js/env/src/module_records.rs	self[.]import_meta_object = import_meta_object;
crates/lyng-js/env/src/module_records.rs	self[.]resolved_exports = resolved_exports;
crates/lyng-js/env/src/module_records.rs	self[.]evaluation_error = evaluation_error;
crates/lyng-js/env/src/promise.rs	self[.]promises[.]push[(]Some[(]PromiseRecord::new[(]object, realm[)][)][)];
crates/lyng-js/env/src/promise.rs	self[.]promise_by_object\[index\] = Some[(]id[)];
crates/lyng-js/env/src/promise.rs	record[.]result = value;
crates/lyng-js/env/src/promise.rs	record[.]result = reason;
crates/lyng-js/env/src/promise.rs	self[.]reactions[.]push[(]Some[(]reaction[)][)];
crates/lyng-js/env/src/promise.rs	self[.]capabilities[.]push[(]Some[(]PromiseCapabilityRecord::new[(][)][)][)];
crates/lyng-js/env/src/promise.rs	record[.]promise = Some[(]promise[)];
crates/lyng-js/env/src/promise.rs	record[.]resolve = Some[(]resolve[)];
crates/lyng-js/env/src/promise.rs	record[.]reject = Some[(]reject[)];
crates/lyng-js/env/src/promise.rs	self[.]resolving_functions[.]push[(]Some[(]record[)][)];
crates/lyng-js/env/src/promise.rs	self[.]resolving_function_by_object\[index\] = Some[(]id[)];
crates/lyng-js/env/src/promise.rs	self[.]finally_functions[.]push[(]Some[(]record[)][)];
crates/lyng-js/env/src/promise.rs	self[.]finally_function_by_object\[index\] = Some[(]id[)];
crates/lyng-js/env/src/promise.rs	self[.]combinator_elements[.]push[(]Some[(]record[)][)];
crates/lyng-js/env/src/promise.rs	self[.]combinator_element_by_object\[index\] = Some[(]id[)];
crates/lyng-js/env/src/disposal.rs	self[.]capability_by_object\[index\] = Some[(]capability[)];
crates/lyng-js/env/src/disposal.rs	record[.]resources[.]push[(]resource[)];
crates/lyng-js/env/src/disposal.rs	record[.]pending_error = pending_error;
crates/lyng-js/env/src/disposal.rs	self[.]async_resumes[.]push[(]Some[(]record[)][)];
crates/lyng-js/env/src/disposal.rs	self[.]async_resume_by_object\[index\] = Some[(]id[)];
crates/lyng-js/env/src/agent/environments.rs	self[.]environment_layouts[.]push[(]Some[(]layout[)][)];
crates/lyng-js/env/src/agent/environments.rs	[*]target = alias;
crates/lyng-js/env/src/agent/environments.rs	lexical_bindings[.]push[(]binding[)];
crates/lyng-js/env/src/agent/weak_finalization.rs	self[.]kept_objects[.]push[(]target[)];
crates/lyng-js/vm/src/vm/feedback.rs	self[.]entries\[index\] = Some[(]plan[)];
crates/lyng-js/vm/src/vm/feedback.rs	self[.]entries\[usize::from[(]self[.]entry_count[)]\] = Some[(]plan[)];
crates/lyng-js/vm/src/vm/feedback.rs	self[.]entries\[0\] = Some[(]entry[)];
crates/lyng-js/vm/src/vm/feedback.rs	self[.]named_entries\[0\] = Some[(]KeyedNamedPropertyCacheEntry
crates/lyng-js/vm/src/vm/feedback.rs	self[.]named_entries\[0\] =
crates/lyng-js/vm/src/vm/feedback.rs	self[.]named_entries\[index\] =
crates/lyng-js/vm/src/vm/feedback.rs	self[.]named_entries\[usize::from[(]self[.]named_entry_count[)]\] =
crates/lyng-js/vm/src/vm/feedback.rs	self[.]dense_entries\[index\] = Some[(]plan[)];
crates/lyng-js/vm/src/vm/feedback.rs	self[.]dense_entries\[usize::from[(]self[.]dense_entry_count[)]\] = Some[(]plan[)];
crates/lyng-js/vm/src/vm/feedback.rs	self[.]dense_entries\[0\] = Some[(]entry[)];
crates/lyng-js/vm/src/vm/feedback.rs	self[.]feedback_vectors\[index\] =
<!-- gc-write-site-allowlist:end -->
