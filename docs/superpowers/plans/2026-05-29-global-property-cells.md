# Global Property Cells (Milestone 1) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make global variable/property reads fast and independent of global-scope size by backing every global binding with a heap `ValueCell` and caching the cell in a monomorphic global inline cache, with constant folding for write-once globals.

**Architecture:** Global `var`/`function` bindings become cell-backed entries in the global object's dictionary (a new `NamedPropertyValue::DataCell` payload, gated by a `CELL_BACKED_DICTIONARY` object flag set only on the global object). Global `let`/`const`/`class` bindings become cells in the global environment with a `name→cell` map (retiring the O(n) lexical scan). `LoadGlobal`/`StoreGlobal`/`AssignGlobal` resolve a binding to its cell once on the cold path, then cache the cell ref in a new cell-keyed IC; reads load (or, when the cell is a write-once constant, return) the value without a hashmap probe. A constness lattice plus a cell-keyed watchpoint registry (mirroring the existing shape `watchpoint_sets`) deopts folded sites on reassignment, and structural changes (delete / accessor redefine / non-writable / lexical shadowing) drain the cell's dependent sites.

**Tech Stack:** Rust workspace (`crates/{gc,objects,env,vm}`), the existing heap `PrimitiveValueCell` primitive, the existing `WatchpointSet`/generation IC-invalidation machinery, `lyng-test262` and `lyng-bench` for verification.

**Spec:** `docs/superpowers/specs/2026-05-29-global-property-cells-design.md`

**Key implementation decision (read before starting):** `LoadGlobal` is serviced by a Rust probe (`crates/vm/src/dsl/handlers/cold.rs::op_load_global_rust_probe_rs` → `Vm::try_load_global_rust_probe_for_dsl`), not by a `PropertyMetadata`-mode asm fast path. The global cell IC is therefore implemented in the Rust probe / `*_with_feedback` functions — **no assembly or `PropertyMetadata` mode changes in M1.** Constant folding in M1 saves the cell dereference (one heap load) per read; the larger zero-load asm win is a future optimization on top of this foundation and is explicitly out of scope.

---

## File Structure

**Modified:**
- `crates/objects/src/core.rs` — add `CELL_BACKED_DICTIONARY` flag + accessor.
- `crates/objects/src/shapes.rs` — add `NamedPropertyValue::DataCell` variant + helpers.
- `crates/objects/src/object_metadata.rs` — dictionary entry helpers for cell payloads.
- `crates/objects/src/internal_methods/named_properties.rs` — read/define/delete/enumerate handle `DataCell`.
- `crates/objects/src/runtime.rs` — `cell_watchpoints` registry + cell value read/write helpers + cell-backed define entrypoint.
- `crates/objects/src/watchpoint.rs` — `CellInvalidationObserver` (or reuse observer enum keyed by cell).
- `crates/gc/src/rooting.rs` — trace cell-backed dictionary entry cell refs (and confirm dictionary-value tracing from Task 0.1).
- `crates/env/src/environment_records.rs` — `GlobalLexicalBindingRecord` gains a cell; `EnvironmentMetadata::Global` gains a `name→cell` map.
- `crates/env/src/agent/environments.rs` — lexical cell accessors; O(1) `name→cell` lookup.
- `crates/env/src/agent.rs` — `fire_cell_watchpoints`, cell-watchpoint registration.
- `crates/vm/src/vm/global_script.rs` — create cell-backed var/function bindings; create lexical cells.
- `crates/vm/src/vm/names.rs` — cold-path cell resolution; cell-load fast path in `load/store/assign_global_with_feedback`; retire lexical linear scan.
- `crates/vm/src/vm/feedback.rs` — `GlobalCellIcState` (cell-keyed IC), constness transitions, deopt clearing.
- `crates/vm/src/vm.rs` — wire cell-watchpoint dispatch into the `AdaptiveProtoLoadDispatch` impl if needed.

**New test files:**
- `crates/tests/src/gc_global_cell.rs` — GC stress for cell-backed globals.
- `crates/vm/src/tests/global_cells.rs` — cell IC hit/miss, constant fold + deopt, invalidation, TDZ.

**Verification (no code):** `lyng-test262`, `lyng-bench` (`compare` + `v8suite`), and the global-count microbenchmark from the investigation.

---

## Phase 0 — De-risk and scaffold

### Task 0.1: Characterize how dictionary property *values* are kept alive by GC

A `var x = {…}` on a dictionary-mode global object stores its value in
`ObjectMetadata.named_properties` (agent-side), not in the heap object record's
slots. The cell design depends on knowing whether/how those values are traced.
This task settles it with a test before anything is built on top.

**Files:**
- Test: `crates/tests/src/gc_global_cell.rs` (new; register in `crates/tests/src/lib.rs` if that file lists test modules)

- [ ] **Step 1: Write a GC-survival test for a dictionary-mode global value**

```rust
// crates/tests/src/gc_global_cell.rs
use lyng_vm::Runtime;
// Use the crate's existing test harness imports; mirror crates/tests/src/gc_stress_frame_context.rs
// for Runtime/Agent/heap access and forcing collection.

#[test]
fn dictionary_global_object_value_survives_collection() {
    // 1. Build a realm.
    // 2. Force the global object into dictionary mode (declare >= 64 globals,
    //    or call agent.ensure_named_property_dictionary on the global object).
    // 3. Define a global `var` whose value is a freshly allocated heap object
    //    (e.g. an ordinary object) and keep NO other root to it.
    // 4. Force a full GC (agent.force_collect() / the harness equivalent).
    // 5. Read the global back and assert the object is intact (not collected /
    //    not a use-after-free). Assert a property set on it before GC is still
    //    readable after GC.
}
```

- [ ] **Step 2: Run it**

Run: `cargo test -p lyng-tests dictionary_global_object_value_survives_collection -- --nocapture`
Expected: either PASS (dictionary values ARE traced — find and document the path in a code comment in the test) or FAIL (a latent tracing gap exists).

- [ ] **Step 3: Document the finding inline**

Add a top-of-file comment in `gc_global_cell.rs` stating exactly how dictionary
values are traced today (cite the file:line of the tracing code), or that they
are not. If the test FAILS, STOP and surface this: it is a pre-existing GC bug
that must be fixed (or the tracing hooked) before cell-backing — and Phase 6's
tracing task becomes mandatory rather than confirmatory. If it PASSES, note the
mechanism so Phase 6 extends the same path for cell refs.

- [ ] **Step 4: Commit**

```bash
git add crates/tests/src/gc_global_cell.rs crates/tests/src/lib.rs
git commit -m "test: characterize GC tracing of dictionary-mode global values"
```

### Task 0.2: Add the `CELL_BACKED_DICTIONARY` object flag

**Files:**
- Modify: `crates/objects/src/core.rs:11-110`

- [ ] **Step 1: Write the failing test**

```rust
// In crates/objects/src/core.rs (add a #[cfg(test)] mod tests block, or extend the existing one)
#[test]
fn cell_backed_dictionary_flag_roundtrips() {
    let flags = ObjectFlags::empty().union(ObjectFlags::CELL_BACKED_DICTIONARY);
    assert!(flags.uses_cell_backed_dictionary());
    assert!(!ObjectFlags::empty().uses_cell_backed_dictionary());
    // Independent from the dictionary flag:
    assert!(!ObjectFlags::CELL_BACKED_DICTIONARY.contains(ObjectFlags::NAMED_PROPERTIES_DICTIONARY));
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p lyng-objects cell_backed_dictionary_flag_roundtrips`
Expected: FAIL — `CELL_BACKED_DICTIONARY` and `uses_cell_backed_dictionary` not defined.

- [ ] **Step 3: Add the flag and accessor**

In `crates/objects/src/core.rs`, add the constant alongside the others (next
free bit is `1 << 9`; `IS_HTMLDDA` is `1 << 8`):

```rust
    pub const CELL_BACKED_DICTIONARY: Self = Self(1 << 9);
```

And the accessor alongside `uses_named_property_dictionary`:

```rust
    #[inline]
    pub const fn uses_cell_backed_dictionary(self) -> bool {
        self.contains(Self::CELL_BACKED_DICTIONARY)
    }
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p lyng-objects cell_backed_dictionary_flag_roundtrips`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/objects/src/core.rs
git commit -m "feat(objects): add CELL_BACKED_DICTIONARY object flag"
```

### Task 0.3: Trace dictionary entry values via a per-object metadata mark hook

**Why (from Task 0.1):** dictionary-mode property values live in agent-side
`ObjectRuntime::object_metadata`, which the GC mark walk never visits — a heap
object reachable only through a dictionary entry is collected. This task fixes
the latent bug generally and creates the hook that will later mark `DataCell`
cell refs. Approach (validated against the GC architecture): a callback trait the
marker invokes per marked object; `ObjectRuntime` implements it to mark its
dictionary entry values.

**Files:**
- Modify: `crates/gc/src/rooting.rs` (the `MarkWorkItem::Object(id)` arm ~701; `PrimitiveTracer` ~93; tracer construction sites)
- Modify: `crates/gc/src/collection.rs` (`force_collect_tracing` ~246) and any other tracer entry points (incremental `mark_step`, minor GC) so the metadata tracer is threaded through ALL of them
- Modify: `crates/env/src/agent/weak_finalization.rs` (`force_collect_with_additional_roots` ~34) and any incremental-collection driver in `crates/env` to pass `&self.objects`
- Create: `crates/objects/src/gc_integration.rs` (impl the trait for `ObjectRuntime`)
- Test: extend `crates/tests/src/gc_global_cell.rs`

- [ ] **Step 1: Flip the Task 0.1 test to assert survival**

Rename `dictionary_global_object_value_survives_collection` so the name matches
intent, and change the final assertion from "collected" to "survives": after
`force_collect()`, the inner object's heap record must still exist AND its
sentinel property must read back as `0xDEAD`. Update the module doc comment to
describe the (new) tracing mechanism.

```rust
#[test]
fn dictionary_global_object_value_survives_collection() {
    // ... same setup ...
    let _ = agent.force_collect();
    let heap_record = agent.heap().view().object(recovered_inner);
    assert!(heap_record.is_some(), "dictionary entry value must survive GC");
    // assert sentinel property still reads 0xDEAD via get_own_property on recovered_inner
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p lyng-tests dictionary_global_object_value_survives_collection`
Expected: FAIL — value still collected (no tracing yet).

- [ ] **Step 3: Add the metadata mark hook**

In `crates/gc/src/rooting.rs` define a callback trait and a no-op impl for `()`:

```rust
pub trait TraceObjectMetadataEdges {
    fn trace_object_metadata_edges(&self, object: ObjectRef, tracer: &mut PrimitiveTracer<'_>);
}
impl TraceObjectMetadataEdges for () {
    fn trace_object_metadata_edges(&self, _: ObjectRef, _: &mut PrimitiveTracer<'_>) {}
}
```

Add `metadata_tracer: &'a dyn TraceObjectMetadataEdges` to `PrimitiveTracer`, and
in the `MarkWorkItem::Object(id)` arm, after `record.trace_heap_edges(self)`,
call `self.metadata_tracer.trace_object_metadata_edges(id, self)`. Thread the
`&dyn TraceObjectMetadataEdges` through EVERY tracer construction site (force
collect AND incremental/minor mark slices) — grep for `PrimitiveTracer {` and
every `force_collect_tracing`/`mark_step` caller. Default existing callers that
have no objects layer to `&()`.

In `crates/objects/src/gc_integration.rs`:

```rust
use crate::{NamedPropertyStorage, NamedPropertyValue, ObjectRuntime};
use lyng_gc::{ObjectRef, PrimitiveTracer, TraceObjectMetadataEdges};

impl TraceObjectMetadataEdges for ObjectRuntime {
    fn trace_object_metadata_edges(&self, object: ObjectRef, tracer: &mut PrimitiveTracer<'_>) {
        let Some(metadata) = self.object_metadata(object) else { return };
        let NamedPropertyStorage::Dictionary(dict) = &metadata.named_properties else { return };
        for entry in dict.entries.values() {
            match entry.payload {
                NamedPropertyValue::Data(value) => tracer.mark_value(value),
                NamedPropertyValue::Accessor { get, set } => {
                    tracer.mark_value(get);
                    tracer.mark_value(set);
                }
                // DataCell arm added in Task 1.1's follow-up (mark the cell ref).
            }
        }
    }
}
```

Wire `&self.objects` as the metadata tracer in the agent collection entry
point(s). Mind the borrow split: build `AgentCollectionSnapshot::from_agent(self)`
first, then split-borrow `heap` (mut) and `objects` (shared) as separate fields.

- [ ] **Step 4: Run to verify it passes + full test262**

Run: `cargo test -p lyng-tests dictionary_global_object_value_survives_collection`
Expected: PASS.
Run: `cargo run -p lyng-test262 -- 2>&1 | tail -20`
Expected: no new failures. **Record the baseline pass count on `main` now** (check out main in a throwaway worktree or note a prior run) so later phases can compare.

- [ ] **Step 5: Commit**

```bash
git add crates/gc/src/rooting.rs crates/gc/src/collection.rs crates/env/src/agent/weak_finalization.rs crates/objects/src/gc_integration.rs crates/objects/src/lib.rs crates/tests/src/gc_global_cell.rs
git commit -m "fix(gc): trace dictionary-mode object property values via metadata mark hook"
```

### Task 0.4: Incremental-marking write barrier for dictionary edge writes

**Why:** dictionary entries are written agent-side (`NamedPropertyDictionary::upsert`),
bypassing the heap write barrier. If a value (or, later, a cell ref) is inserted
into a dictionary entry of an already-marked (black) object *during* incremental
marking, the mark hook from Task 0.3 already ran for that object and won't run
again — the new edge is missed and the value can be collected mid-cycle. This
task adds a barrier so such writes shade the new value. It covers both `Data`
values and (later) `DataCell` cell refs, since both flow through `upsert`.

**Scope note (decided after Task 0.3):** the *generational* facet — a YOUNG value
reachable only through an OLD object's dictionary entry being lost in a MINOR GC
(dictionary writes don't dirty cards; `PrimitiveMinorTracer` doesn't visit
metadata) — is an accepted pre-existing limitation and is NOT addressed here. The
global-property-cells feature sidesteps it by allocating global cells **tenured**
(`AllocationLifetime::Default`, already specified in Task 1.3): a tenured cell ref
is an old→old edge handled by major collection (Task 0.3), and the cell's *value*
gets full generational + incremental barriers for free via the existing
`ValueCell` machinery (`set_value_cell_value` dirties cards + shades). This task is
only the incremental-marking barrier, which tenure does NOT solve (a black object
gaining a new edge mid-major-cycle must shade regardless of generation).

**Files:**
- Modify: `crates/objects/src/internal_methods/named_properties.rs` (`redefine_named_property` ~190 / wherever `dictionary.upsert` is called)
- Modify: `crates/gc/src` (expose a "shade value if incremental mark in progress and owner is black" entry the objects layer can call; mirror `incremental_value_barrier` in `crates/gc/src/writer.rs`)
- Test: `crates/tests/src/gc_global_cell.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn dictionary_value_written_during_incremental_mark_survives() {
    // 1. Force the global object to dictionary mode and mark it (start an
    //    incremental mark and step until the global object is black — use the
    //    incremental mark API the gc crate exposes; mirror any existing
    //    incremental-marking test under crates/tests or crates/gc).
    // 2. While the mark is in progress, define a NEW global var holding a fresh
    //    heap object (sole ref) -> goes through the dictionary upsert path.
    // 3. Finish the mark + sweep.
    // 4. Assert the new object survived.
}
```

If no incremental-marking test harness exists to drive steps, report
NEEDS_CONTEXT before guessing — the controller will supply the incremental API.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p lyng-tests dictionary_value_written_during_incremental_mark_survives`
Expected: FAIL — new value collected mid-cycle.

- [ ] **Step 3: Add the barrier on dictionary writes**

Expose a barrier helper from the gc layer (e.g. `PrimitiveMutator::dictionary_edge_write_barrier(owner: ObjectRef, value: Value)`) that shades `value` when `incremental_mark_in_progress()` and `owner` is marked — reusing `incremental_value_barrier` internals. Call it from the dictionary `upsert` path (in `redefine_named_property`, before/after the upsert) for the owning object and the written value (and, once cells exist, when inserting a `DataCell` ref, shade the cell). The objects layer has `&mut PrimitiveMutator` in these paths.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p lyng-tests dictionary_value_written_during_incremental_mark_survives`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "fix(gc): write barrier for dictionary edge writes during incremental marking"
```

---

## Phase 1 — Cell-backed dictionary storage (global object only)

### Task 1.1: Add the `DataCell` payload variant

**Files:**
- Modify: `crates/objects/src/shapes.rs:704-745`

- [ ] **Step 1: Write the failing test**

```rust
// crates/objects/src/shapes.rs tests module
#[test]
fn named_property_value_datacell_reports_data_kind() {
    let cell = lyng_gc::PrimitiveValueCellRef::from_raw(1).unwrap();
    let v = NamedPropertyValue::DataCell(cell);
    assert_eq!(v.kind(), ShapePropertyKind::Data);
    assert_eq!(v.cell(), Some(cell));
    assert_eq!(v.data_value(), None); // value lives in the cell, not inline
    assert_eq!(NamedPropertyValue::data(Value::from_smi(1)).cell(), None);
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p lyng-objects named_property_value_datacell_reports_data_kind`
Expected: FAIL — `DataCell` variant and `cell()` not defined.

- [ ] **Step 3: Add the variant and helpers**

In `crates/objects/src/shapes.rs`, extend the enum and impl:

```rust
pub enum NamedPropertyValue {
    Data(Value),
    DataCell(lyng_gc::PrimitiveValueCellRef),
    Accessor { get: Value, set: Value },
}

impl NamedPropertyValue {
    // ... existing data()/accessor() ...

    #[inline]
    pub const fn kind(self) -> ShapePropertyKind {
        match self {
            Self::Data(_) | Self::DataCell(_) => ShapePropertyKind::Data,
            Self::Accessor { .. } => ShapePropertyKind::Accessor,
        }
    }

    #[inline]
    pub const fn cell(self) -> Option<lyng_gc::PrimitiveValueCellRef> {
        match self {
            Self::DataCell(cell) => Some(cell),
            _ => None,
        }
    }

    // data_value() stays as-is: DataCell returns None (value is indirected).
}
```

Then run `cargo build -p lyng-objects` and add the `DataCell` arm to every
remaining `match` on `NamedPropertyValue` the compiler flags. Known sites to
expect: `data_value`, `accessor_values`, and `descriptor_from_payload` in
`crates/objects/src/internal_methods/named_properties.rs` (handled in Task 1.2 —
for now add a `Self::DataCell(_) => unreachable!("cell-backed read goes through descriptor_from_cell_payload")` placeholder ONLY in non-read helpers, and leave `descriptor_from_payload` for Task 1.2). Do not leave `unreachable!` in any path a cell-backed entry can reach; Task 1.2 replaces them.

**Also extend the GC metadata mark hook** (`crates/objects/src/gc_integration.rs`
from Task 0.3): add a `NamedPropertyValue::DataCell(cell) => tracer.mark_value_cell(cell)`
arm so cell refs in dictionary entries are traced (the cell's `stored_value` is
then traced by the existing `ValueCell` machinery). Without this, cell-backed
global values would be collected.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p lyng-objects named_property_value_datacell_reports_data_kind && cargo build -p lyng-objects`
Expected: PASS + clean build.

- [ ] **Step 5: Commit**

```bash
git add crates/objects/src/shapes.rs
git commit -m "feat(objects): add NamedPropertyValue::DataCell payload variant"
```

### Task 1.2: Read cell-backed entries in dictionary internal methods

**Files:**
- Modify: `crates/objects/src/internal_methods/named_properties.rs` (`ordinary_own_named_property` ~251-276, `collect_own_named_keys` ~452-492, and `descriptor_from_payload`)
- Modify: `crates/objects/src/runtime.rs` (add `cell_value`/`set_cell_value` helpers near the existing `ordinary_payload_value` ~737)

- [ ] **Step 1: Write the failing test**

```rust
// crates/objects/src/internal_methods/named_properties.rs tests, or a runtime-level test.
// Build an object, force dictionary mode, insert a DataCell entry whose cell
// holds Value::from_smi(42), and assert get_own_property returns a data
// descriptor with value 42.
#[test]
fn dictionary_datacell_entry_reads_through_cell() {
    // setup via the crate's object-runtime test helpers (mirror existing tests
    // in this file). Use heap.alloc_value_cell + init_store_value to make the cell.
    // Assert ordinary_own_named_property -> Some(descriptor) with value 42.
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p lyng-objects dictionary_datacell_entry_reads_through_cell`
Expected: FAIL (panics on `unreachable!` or returns wrong value).

- [ ] **Step 3: Implement cell-aware reads**

Add a helper that resolves a payload to a descriptor, dereferencing cells:

```rust
// crates/objects/src/internal_methods/named_properties.rs
fn descriptor_from_cell_payload(
    &self,
    heap: PrimitiveHeapView<'_>,
    payload: NamedPropertyValue,
    attrs: DescriptorAttributes,
) -> InternalMethodResult<PropertyDescriptor> {
    match payload {
        NamedPropertyValue::DataCell(cell) => {
            let value = heap
                .value_cell(cell)
                .ok_or(InternalMethodError::CorruptObjectState)?
                .stored_value();
            Ok(descriptor_from_payload(NamedPropertyValue::Data(value), attrs))
        }
        other => Ok(descriptor_from_payload(other, attrs)),
    }
}
```

Update `ordinary_own_named_property`'s dictionary arm to call
`self.descriptor_from_cell_payload(heap, entry.payload(), entry.attrs())?`
instead of `descriptor_from_payload(...)`. `collect_own_named_keys` is unchanged
(keys/enumeration don't read values). Remove any `unreachable!` arms added in
Task 1.1 that are now reachable.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p lyng-objects dictionary_datacell_entry_reads_through_cell`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/objects/src/internal_methods/named_properties.rs crates/objects/src/runtime.rs
git commit -m "feat(objects): read cell-backed dictionary entries through the cell"
```

### Task 1.3: Make `redefine_named_property` cell-aware with identity preservation

This is the central mechanism: on a `CELL_BACKED_DICTIONARY` object, **every**
data-property define/overwrite must route through a cell, and overwriting an
existing cell-backed entry must **reuse the same cell** (write through it) rather
than allocate a new one — otherwise the IC's cached cell ref would dangle. This
single change covers declared globals, sloppy implicit globals (`x = 1`),
`globalThis.foo = 1`, and `Object.defineProperty(globalThis, …)`, since they all
funnel through `redefine_named_property` for dictionary objects.

**Files:**
- Modify: `crates/objects/src/internal_methods/named_properties.rs` (`redefine_named_property` ~190, `delete_named_property` ~212)
- Modify: `crates/objects/src/runtime.rs` (add `cell_backed_entry` accessor)

- [ ] **Step 1: Write the failing test**

```rust
// crates/objects/src/internal_methods/named_properties.rs tests
#[test]
fn cell_backed_redefine_preserves_cell_identity_on_overwrite() {
    // 1. force dictionary + set CELL_BACKED_DICTIONARY on the object.
    // 2. redefine_named_property(key, Data(undefined), attrs) -> entry is DataCell(c0).
    let c0 = /* cell_backed_entry(id, key) */;
    // 3. redefine_named_property(key, Data(7), attrs) -> SAME cell c0, value now 7.
    assert_eq!(cell_backed_entry(id, key), Some(c0));
    // get_own_property(key) -> 7.
    // 4. A NON-cell-backed object's redefine still produces a plain Data entry.
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p lyng-objects cell_backed_redefine_preserves_cell_identity_on_overwrite`
Expected: FAIL — redefine always stores plain `Data`.

- [ ] **Step 3: Route data payloads through cells on cell-backed objects**

Rewrite `redefine_named_property` so that, when the object's flags contain
`CELL_BACKED_DICTIONARY` and `payload` is `Data(v)` (or `DataCell`):

```rust
// inside redefine_named_property, after ensure_named_property_dictionary:
let cell_backed = self
    .object_metadata(id)
    .is_some_and(|m| m.flags.uses_cell_backed_dictionary());

if cell_backed {
    if let NamedPropertyValue::Data(value) | NamedPropertyValue::DataCell(_) = payload {
        let value = match payload {
            NamedPropertyValue::Data(v) => v,
            NamedPropertyValue::DataCell(c) => heap.view().value_cell(c)
                .map(|r| r.stored_value()).unwrap_or(Value::undefined()),
            _ => unreachable!(),
        };
        // Reuse the existing cell if present (identity preservation); else alloc.
        let existing = self.cell_backed_entry(id, key);
        let cell = match existing {
            Some(c) => { heap.mut_store_value(ValueStoreTarget::ValueCell(c), value); c }
            None => {
                let c = heap.alloc_value_cell(AllocationLifetime::Default);
                heap.init_store_value(ValueStoreTarget::ValueCell(c), value);
                c
            }
        };
        let metadata = self.object_metadata_mut(id)?; // adjust to fn's error style
        let NamedPropertyStorage::Dictionary(dict) = &mut metadata.named_properties else { return false; };
        dict.upsert(key, NamedPropertyValue::DataCell(cell), attrs);
        return self.refresh_integrity_level_flags(heap.view(), id);
    }
    // Accessor payloads on a cell-backed object: store as plain Accessor
    // (drop any prior cell — Phase 5 drains its dependents first).
}
// ...existing plain path unchanged for non-cell-backed objects/accessors.
dictionary.upsert(key, payload, attrs);
self.refresh_integrity_level_flags(heap.view(), id)
```

Add the `cell_backed_entry` accessor to the object runtime:

```rust
pub fn cell_backed_entry(&self, id: ObjectRef, key: PropertyKey)
    -> Option<lyng_gc::PrimitiveValueCellRef>
{
    let metadata = self.object_metadata(id)?;
    let NamedPropertyStorage::Dictionary(dictionary) = &metadata.named_properties else { return None; };
    dictionary.entry(key)?.payload().cell()
}
```

Extend `delete_named_property` to return the removed entry
(`Option<NamedPropertyDictionaryEntry>`) instead of `bool` so the VM layer can
drain + free the cell in Phase 5; update its callers. (If the signature change
is too invasive here, add a sibling `delete_named_property_entry(...) ->
Option<NamedPropertyDictionaryEntry>` and route the global object through it.)

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p lyng-objects cell_backed_redefine_preserves_cell_identity_on_overwrite`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/objects/src/internal_methods/named_properties.rs crates/objects/src/runtime.rs
git commit -m "feat(objects): cell-backed redefine routes data through cells, preserving identity"
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p lyng-objects cell_backed_binding_define_write_read_delete`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/objects/src/runtime.rs crates/objects/src/internal_methods/named_properties.rs
git commit -m "feat(objects): cell-backed define/read/delete entrypoints"
```

### Task 1.4: Create global var/function bindings as cell-backed entries

**Files:**
- Modify: `crates/vm/src/vm/global_script.rs` (`define_global_binding_property` ~177, `ensure_global_object_dictionary` ~91)

- [ ] **Step 1: Write the failing test**

```rust
// crates/vm/src/tests/global_cells.rs (new; register the module in the tests mod)
#[test]
fn global_var_binding_is_cell_backed() {
    // Compile + run "var x = 5; x" through a realm whose global object has been
    // forced cell-backed (>=64 globals OR a test hook). Assert the result is 5
    // AND that agent.cell_backed_entry(global_object, key("x")).is_some().
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p lyng-vm global_var_binding_is_cell_backed`
Expected: FAIL — bindings are plain `Data`, not `DataCell`.

- [ ] **Step 3: Make the global object cell-backed and define cell entries**

In `ensure_global_object_dictionary`, after forcing the dictionary, set the
`CELL_BACKED_DICTIONARY` flag on the global object (add an agent method
`set_cell_backed_dictionary(global_object)` in `crates/objects/src/runtime.rs`
that unions the flag onto the metadata). **Gate:** only the global object is
made cell-backed; ordinary dictionaries are untouched.

`define_global_binding_property` keeps calling the generic
`ordinary_define_property` — no change needed there, because Task 1.3 made
`redefine_named_property` (which the dictionary define path funnels through)
produce `DataCell` entries automatically once the global object carries
`CELL_BACKED_DICTIONARY`. So the only change in this task is ensuring the flag is
set on the global object (`set_cell_backed_dictionary`). Task 1.5 sets it at
realm creation so it does not depend on the 64-global threshold.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p lyng-vm global_var_binding_is_cell_backed`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm/global_script.rs crates/objects/src/runtime.rs crates/vm/src/tests/global_cells.rs
git commit -m "feat(vm): create global var/function bindings as cell-backed entries"
```

### Task 1.5: Force the global object cell-backed at realm creation; full test262

**Files:**
- Modify: realm bootstrap (search `alloc_global_environment` callers / realm setup in `crates/vm` or `crates/env`) to make the global object a cell-backed dictionary from the start, so cell-backing does not depend on the 64-global threshold.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn small_script_global_is_cell_backed_from_start() {
    // Run "var a = 1; a" in a fresh realm (NO bulk globals). Assert result 1
    // and cell_backed_entry(global_object, key("a")).is_some().
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p lyng-vm small_script_global_is_cell_backed_from_start`
Expected: FAIL — small-script global object is shape-stable, not cell-backed.

- [ ] **Step 3: Make the global object cell-backed at creation**

At realm/global-object construction, force the global object into a cell-backed
dictionary (call `ensure_named_property_dictionary` + `set_cell_backed_dictionary`).
Pre-existing builtin globals defined during bootstrap will be plain dictionary
entries; that is fine — `cell_backed_entry` returns `None` for them and the IC
(Phase 3) falls back. Newly declared script globals use the cell path.

- [ ] **Step 4: Run the test + full suite**

Run: `cargo test -p lyng-vm small_script_global_is_cell_backed_from_start`
Expected: PASS.
Run: `cargo run -p lyng-test262 -- 2>&1 | tail -20` (full suite)
Expected: **no new failures vs. the pre-change baseline.** Record the baseline pass count BEFORE Phase 1 (run test262 on `main` once and note the number); compare here. Investigate any regression before proceeding.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(vm): global object is cell-backed from realm creation"
```

---

## Phase 2 — Global lexical bindings as cells

### Task 2.1: `GlobalLexicalBindingRecord` carries a cell; add a `name→cell` map

**Files:**
- Modify: `crates/env/src/environment_records.rs:182-213` (record) and `:371-376` (`EnvironmentMetadata::Global`)
- Modify: `crates/env/src/agent/environments.rs` (`alloc_global_environment` ~131, `global_set_lexical_binding` ~506, `global_lexical_binding` ~481, add a map insert/lookup)

- [ ] **Step 1: Write the failing test**

```rust
// crates/vm/src/tests/global_cells.rs
#[test]
fn global_lexical_binding_resolves_through_cell_map() {
    // Run "let y = 9; y" in a fresh realm. Assert result 9 and that the global
    // env reports a cell for "y" via the new agent.global_lexical_cell(env, key("y")).
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p lyng-vm global_lexical_binding_resolves_through_cell_map`
Expected: FAIL — no cell map / accessor.

- [ ] **Step 3: Add the cell map and accessors**

Add `cell: Option<PrimitiveValueCellRef>` to `GlobalLexicalBindingRecord` (and a
`with_cell`/`cell()` accessor). Add a `lexical_cells: HashMap<AtomId,
PrimitiveValueCellRef>` to `EnvironmentMetadata::Global` (init empty in
`alloc_global_environment`). Add:

```rust
// crates/env/src/agent/environments.rs
pub fn global_lexical_cell(&self, id: EnvironmentRef, name: AtomId)
    -> Option<PrimitiveValueCellRef>
{
    match self.environment_metadata(id) {
        Some(EnvironmentMetadata::Global { lexical_cells, .. }) =>
            lexical_cells.get(&name).copied(),
        _ => None,
    }
}

pub fn global_set_lexical_cell(&mut self, id: EnvironmentRef, name: AtomId,
    cell: PrimitiveValueCellRef) -> bool
{
    let Some(EnvironmentMetadata::Global { lexical_cells, .. }) =
        self.environment_metadata_mut(id) else { return false; };
    lexical_cells.insert(name, cell);
    true
}
```

- [ ] **Step 4: Run to verify it fails differently (binding creation not yet wired)**

Run: `cargo test -p lyng-vm global_lexical_binding_resolves_through_cell_map`
Expected: still FAIL — the cell is never created yet (Task 2.2).

- [ ] **Step 5: Commit**

```bash
git add crates/env/src/environment_records.rs crates/env/src/agent/environments.rs
git commit -m "feat(env): global lexical bindings carry a cell + name->cell map"
```

### Task 2.2: Create lexical cells at instantiation; initialize on first write; TDZ sentinel

**Files:**
- Modify: `crates/vm/src/vm/global_script.rs` (the `lexical_names` loop ~66) and wherever global lexical bindings are *initialized* (search for `global_set_lexical_binding` callers and lexical init in `crates/vm/src/vm/names.rs`)

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn global_lexical_binding_resolves_through_cell_map() { /* from Task 2.1, now passes */ }

#[test]
fn global_const_tdz_throws_before_initialization() {
    // Run "x; let x = 1;" (use of x before its let) -> ReferenceError (TDZ).
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p lyng-vm global_lexical_binding_resolves_through_cell_map global_const_tdz_throws_before_initialization`
Expected: FAIL.

- [ ] **Step 3: Allocate a TDZ cell per lexical name; init on declaration**

In the `lexical_names` loop, allocate a cell initialized to the TDZ sentinel
(`Value::empty_internal_slot()` is used elsewhere as an uninitialized marker —
confirm the project's TDZ sentinel; search `tdz`/`uninitialized` in
`crates/vm`), store it via `global_set_lexical_cell`, and record it on the
`GlobalLexicalBindingRecord`. On the binding's *initialization* opcode, write
the real value into the cell. `LoadGlobal` resolving a cell whose value is the
TDZ sentinel throws ReferenceError (wired in Phase 3; for now the slow path must
also honor it).

- [ ] **Step 4: Run to verify they pass**

Run: `cargo test -p lyng-vm global_lexical_binding_resolves_through_cell_map global_const_tdz_throws_before_initialization`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm/global_script.rs crates/vm/src/vm/names.rs
git commit -m "feat(vm): global lexical bindings backed by cells with TDZ sentinel"
```

### Task 2.3: Retire the lexical linear scan

**Files:**
- Modify: `crates/vm/src/vm/names.rs` (`lookup_global_lexical_binding_ref` ~1791, `lookup_global_lexical_binding` ~1500)

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn many_lexical_globals_resolve_in_constant_time_smoke() {
    // Declare 2000 `let` globals then read one in a tight loop. This is a smoke
    // test for behavior, not timing: assert correct value. (Timing is validated
    // in Phase 7.) Primarily guards that the map path returns correct results.
}
```

- [ ] **Step 2: Run to verify it passes already (behavioral) but scan still present**

Run: `cargo test -p lyng-vm many_lexical_globals_resolve_in_constant_time_smoke`
Expected: PASS behaviorally even before the change.

- [ ] **Step 3: Replace the scan with the map lookup**

Change `lookup_global_lexical_binding_ref` / `lookup_global_lexical_binding` to
consult `global_lexical_cell` (O(1) map) first and construct the binding record
from the map, falling back to `lookup_global_layout_binding_ref` only for layout
bindings not represented in the map. Ensure the cell ref is carried on the
returned record.

- [ ] **Step 4: Run the test + full test262**

Run: `cargo test -p lyng-vm many_lexical_globals_resolve_in_constant_time_smoke`
Expected: PASS.
Run: `cargo run -p lyng-test262 -- 2>&1 | tail -20`
Expected: no new failures.

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm/names.rs
git commit -m "perf(vm): resolve global lexical bindings via O(1) cell map"
```

---

## Phase 3 — The global cell inline cache (cell-load fast path)

### Task 3.1: Add `GlobalCellIcState` keyed per (code, slot)

**Files:**
- Modify: `crates/vm/src/vm/feedback.rs` (add a parallel per-slot store; mirror `property_ic_state`)

- [ ] **Step 1: Write the failing test**

```rust
// crates/vm/src/tests/global_cells.rs
#[test]
fn global_cell_ic_state_default_is_empty() {
    let vm = Vm::new();
    // global_cell_ic_state(code, slot) returns None until installed.
    // (Use a compiled unit + its LoadGlobal feedback slot, as in inline_caches.rs.)
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p lyng-vm global_cell_ic_state_default_is_empty`
Expected: FAIL — `global_cell_ic_state` not defined.

- [ ] **Step 3: Add the IC state type and accessors**

```rust
// crates/vm/src/vm/feedback.rs (or a new ic_state/global_cell.rs)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GlobalCellIcState {
    pub cell: Option<lyng_gc::PrimitiveValueCellRef>,
    pub constant: Option<Value>, // Some => folded constant; None => load from cell
    pub generation: u32,
}
```

Store it in a per-code slab keyed by slot, mirroring how `property_ic_states`
are stored. Add `global_cell_ic_state(code, slot) -> Option<GlobalCellIcState>`,
`install_global_cell_ic(code, slot, cell, constant, generation)`, and
`clear_global_cell_ic(code, slot)`. Hook `clear_global_cell_ic` into
`clear_ic_slot_if_generation_matches` (feedback.rs:2730) so existing invalidation
also clears global-cell entries.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p lyng-vm global_cell_ic_state_default_is_empty`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm/feedback.rs
git commit -m "feat(vm): add GlobalCellIcState keyed per (code, slot)"
```

### Task 3.2: Cold-path cell resolution + install in `load_global_with_feedback`

**Files:**
- Modify: `crates/vm/src/vm/names.rs` (`load_global_with_feedback` ~523, `try_load_global_rust_probe_for_dsl` ~616)

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn load_global_installs_cell_ic_and_hits() {
    // Run "var x = 5; x; x" — second read should hit the cell IC.
    // Assert result 5 and global_cell_ic_state(code, slot).cell.is_some().
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p lyng-vm load_global_installs_cell_ic_and_hits`
Expected: FAIL — no cell IC install.

- [ ] **Step 3: Resolve to a cell and install; add the hit path**

At the TOP of `load_global_with_feedback` (and in
`try_load_global_rust_probe_for_dsl`), add a cell-IC hit check before the
existing lexical scan / shape-IC checks:

```rust
// Hit path: cached global cell.
if let Some(slot) = feedback_slot
    && let Some(state) = self.global_cell_ic_state(code, slot)
{
    if let Some(constant) = state.constant {
        return Ok(constant); // folded — Phase 4 sets this
    }
    if let Some(cell) = state.cell {
        let value = agent.heap().view().value_cell(cell)
            .ok_or_else(|| /* corrupt */ VmError::Abrupt(errors::throw_type_error(agent)))?
            .stored_value();
        // TDZ check: empty/uninitialized sentinel -> ReferenceError.
        if value == Value::empty_internal_slot() {
            return Err(VmError::Abrupt(errors::throw_reference_error(agent)));
        }
        return Ok(value);
    }
}
```

On the cold path (miss), resolve the binding to its cell — lexical
`global_lexical_cell(global, name)` first, else
`agent.cell_backed_entry(global_object, key)` — and if a cell is found, install
it via `install_global_cell_ic(code, slot, cell, /*constant*/ None, gen)`. If no
cell (e.g. a bootstrap builtin still plain-backed, or a prototype-chain global),
fall through to today's existing path unchanged.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p lyng-vm load_global_installs_cell_ic_and_hits`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm/names.rs
git commit -m "feat(vm): cell-load fast path + cold-path install for LoadGlobal"
```

### Task 3.3: Cell store/assign fast path

**Files:**
- Modify: `crates/vm/src/vm/names.rs` (`store_global_with_feedback` ~701, `assign_global_with_feedback` ~810)

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn store_global_writes_through_cached_cell() {
    // Run "var x = 1; x = 2; x" -> 2. After the store, the cell IC must remain
    // valid and the value read back through the cell must be 2.
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p lyng-vm store_global_writes_through_cached_cell`
Expected: FAIL — store path doesn't use the cell IC.

- [ ] **Step 3: Add cell store/assign hit + cold install**

Mirror Task 3.2 in `store_global_with_feedback` / `assign_global_with_feedback`:
if a cell IC is installed, write through the cell
(`mut_store_value(ValueStoreTarget::ValueCell(cell), value)`) and run the
constness transition (Phase 4 — for now, just write). On cold-path miss, resolve
the cell and install. Assign-side strict-mode semantics (throw on non-writable)
must read the entry's `attrs` (non-writable cells are not cell-IC'd for
stores — bail to the slow path so existing semantics hold; covered in Phase 5).

- [ ] **Step 4: Run to verify it passes + full test262**

Run: `cargo test -p lyng-vm store_global_writes_through_cached_cell`
Expected: PASS.
Run: `cargo run -p lyng-test262 -- 2>&1 | tail -20`
Expected: no new failures.

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm/names.rs
git commit -m "feat(vm): cell store/assign fast path for global writes"
```

---

## Phase 4 — Constness lattice, constant folding, and deopt

### Task 4.1: Cell-keyed watchpoint registry + constness state

**Files:**
- Modify: `crates/objects/src/runtime.rs` (add `cell_watchpoints: HashMap<PrimitiveValueCellRef, CellWatchpointSet>` next to `watchpoint_sets` ~60)
- Modify: `crates/objects/src/watchpoint.rs` (add `Constness` + `CellWatchpointSet`)

- [ ] **Step 1: Write the failing test**

```rust
// crates/objects/src/watchpoint.rs tests
#[test]
fn constness_lattice_transitions() {
    let mut s = CellWatchpointSet::default(); // state = Uninitialized
    assert_eq!(s.observe_store(Value::from_smi(5)), ConstnessOutcome::BecameConstant);
    assert_eq!(s.observe_store(Value::from_smi(5)), ConstnessOutcome::Unchanged);
    assert_eq!(s.observe_store(Value::from_smi(6)), ConstnessOutcome::Degraded);
    assert_eq!(s.observe_store(Value::from_smi(7)), ConstnessOutcome::Unchanged); // already Mutable
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p lyng-objects constness_lattice_transitions`
Expected: FAIL.

- [ ] **Step 3: Implement the lattice + set**

```rust
// crates/objects/src/watchpoint.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Constness { Uninitialized, Constant(Value), Mutable }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstnessOutcome { BecameConstant, Unchanged, Degraded }

#[derive(Debug, Default)]
pub struct CellWatchpointSet {
    state: Constness,            // Default = Uninitialized
    dependents: Vec<(CodeRef, FeedbackSlotId, u32)>,
}

impl CellWatchpointSet {
    pub fn observe_store(&mut self, v: Value) -> ConstnessOutcome {
        match self.state {
            Constness::Uninitialized => { self.state = Constness::Constant(v); ConstnessOutcome::BecameConstant }
            Constness::Constant(prev) if prev == v => ConstnessOutcome::Unchanged,
            Constness::Constant(_) => { self.state = Constness::Mutable; ConstnessOutcome::Degraded }
            Constness::Mutable => ConstnessOutcome::Unchanged,
        }
    }
    pub fn add_dependent(&mut self, code: CodeRef, slot: FeedbackSlotId, generation: u32) { /* push */ }
    pub fn drain_dependents(&mut self) -> Vec<(CodeRef, FeedbackSlotId, u32)> { std::mem::take(&mut self.dependents) }
    pub const fn state(&self) -> Constness { self.state }
}
```

Add the registry field + `cell_watchpoint_set_mut(cell)` and
`drain_cell_dependents(cell)` accessors on the object runtime (mirror
`watchpoint_set_mut`/`drain_watchpoints_for_shape`).

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p lyng-objects constness_lattice_transitions`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/objects/src/watchpoint.rs crates/objects/src/runtime.rs
git commit -m "feat(objects): cell-keyed constness lattice + watchpoint set"
```

### Task 4.2: `fire_cell_watchpoints` + run the lattice on every cell store

**Files:**
- Modify: `crates/env/src/agent.rs` (`fire_cell_watchpoints` mirroring `fire_watchpoints_for_shape` ~512)
- Modify: `crates/vm/src/vm/names.rs` (cell store paths from Task 3.3 call `observe_store`)

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn reassigning_global_degrades_and_clears_folded_sites() {
    // Run a function that reads global G many times (folds as constant in Phase 4.3),
    // then reassign G to a different value, then read again -> sees the new value.
    // Assert the cell IC for the read site was cleared (re-planned to load).
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p lyng-vm reassigning_global_degrades_and_clears_folded_sites`
Expected: FAIL.

- [ ] **Step 3: Wire store → lattice → deopt**

In the cell store/assign paths, after writing the cell, call
`agent.observe_cell_store(cell, value)` which runs `observe_store` and, on
`Degraded`, calls `fire_cell_watchpoints(cell, self)`:

```rust
// crates/env/src/agent.rs
pub fn fire_cell_watchpoints(&mut self, cell: PrimitiveValueCellRef,
    vm_dispatch: &mut dyn AdaptiveProtoLoadDispatch)
{
    for (code, slot, generation) in self.objects.drain_cell_dependents(cell) {
        vm_dispatch.clear_ic_slot_if_generation_matches(code, slot, generation);
    }
}
```

`clear_ic_slot_if_generation_matches` already clears the (now also) global-cell
IC (wired in Task 3.1). Sites re-plan on next dispatch as cell-load.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p lyng-vm reassigning_global_degrades_and_clears_folded_sites`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/env/src/agent.rs crates/vm/src/vm/names.rs
git commit -m "feat: fire cell watchpoints + run constness lattice on global stores"
```

### Task 4.3: Fold constant cells into the load IC

**Files:**
- Modify: `crates/vm/src/vm/names.rs` (cold-path install in `load_global_with_feedback`)

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn write_once_global_folds_as_constant() {
    // Run "const C = 42;" then a function reading C in a loop.
    // Assert global_cell_ic_state(code, slot).constant == Some(42).
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p lyng-vm write_once_global_folds_as_constant`
Expected: FAIL — install always stores `constant = None`.

- [ ] **Step 3: Install constant when the cell is `Constant`**

On the cold-path install, query the cell's constness
(`agent.cell_constness(cell)`); if `Constant(v)` and not in TDZ, install with
`constant = Some(v)` AND register the site as a dependent via
`cell_watchpoint_set_mut(cell).add_dependent(code, slot, generation)` (bump the
generation first, mirroring `register_adaptive_proto_load_for_chain`). Otherwise
install `constant = None` (cell-load). The hit path from Task 3.2 already returns
`state.constant` when present.

- [ ] **Step 4: Run the test + full test262**

Run: `cargo test -p lyng-vm write_once_global_folds_as_constant`
Expected: PASS.
Run: `cargo run -p lyng-test262 -- 2>&1 | tail -20`
Expected: no new failures.

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm/names.rs
git commit -m "feat(vm): fold write-once globals as constants in the cell IC"
```

---

## Phase 5 — Structural invalidation completeness

### Task 5.1: Delete drains dependents and frees the cell

**Files:**
- Modify: `crates/vm/src/vm/names.rs` (or wherever global `delete` is handled) + `crates/objects/src/internal_methods/named_properties.rs` (`delete_named_property` from Task 1.3)

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn deleting_global_clears_cached_sites() {
    // sloppy: "x = 1; function read(){ return typeof x; } read(); delete x; read();"
    // After delete, the read site must not read a freed cell; result "undefined".
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p lyng-vm deleting_global_clears_cached_sites`
Expected: FAIL or UB (use-after-free / stale value).

- [ ] **Step 3: Drain before free**

When deleting a cell-backed global, before freeing the cell: call
`fire_cell_watchpoints(cell, self)` to clear all dependent sites, then
`heap.free_value_cell(cell)` and remove the dictionary entry. Order matters —
drain first so no site holds a dangling ref.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p lyng-vm deleting_global_clears_cached_sites`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm/names.rs crates/objects/src/internal_methods/named_properties.rs
git commit -m "feat: drain cell dependents and free cell on global delete"
```

### Task 5.2: Accessor redefine, non-writable reconfigure, lexical shadowing

**Files:**
- Modify: `crates/objects/src/internal_methods/named_properties.rs` / `crates/objects/src/runtime.rs` (`[[DefineOwnProperty]]` path for cell-backed entries)
- Modify: `crates/vm/src/vm/global_script.rs` (lexical-shadowing-a-var at instantiation)

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn redefining_global_as_accessor_drops_cell_backing() {
    // "var x = 1; function r(){return x;} r();
    //  Object.defineProperty(globalThis,'x',{get(){return 9;},configurable:true}); r();" -> 9
}
#[test]
fn lexical_shadowing_var_invalidates_var_sites() {
    // In one script that declares `var x` then (eval) introduces `let x`, a read
    // site that bound to the var cell must re-resolve to the lexical cell.
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p lyng-vm redefining_global_as_accessor_drops_cell_backing lexical_shadowing_var_invalidates_var_sites`
Expected: FAIL.

- [ ] **Step 3: Drain on each structural change**

In `[[DefineOwnProperty]]` for a cell-backed entry: when converting data→accessor
or clearing writable, drain the cell's dependents (`fire_cell_watchpoints`) and,
for accessor conversion, replace the `DataCell` entry with an `Accessor` payload
(free the cell after draining). For lexical-shadowing-a-var at instantiation,
drain the existing var cell's dependents for that name when adding the lexical
binding.

- [ ] **Step 4: Run the tests + full test262**

Run: `cargo test -p lyng-vm redefining_global_as_accessor_drops_cell_backing lexical_shadowing_var_invalidates_var_sites`
Expected: PASS.
Run: `cargo run -p lyng-test262 -- 2>&1 | tail -20`
Expected: no new failures.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: invalidate cell sites on accessor redefine / non-writable / shadowing"
```

---

## Phase 6 — GC tracing & cell lifecycle

### Task 6.1: Trace cell refs held by cell-backed dictionary entries and lexical maps

**Files:**
- Modify: `crates/gc/src/rooting.rs` (object edge tracing ~1068) and/or the objects/env crates that feed roots — guided by the Task 0.1 finding.

- [ ] **Step 1: Write the failing test**

```rust
// crates/tests/src/gc_global_cell.rs
#[test]
fn cell_backed_global_object_value_survives_collection() {
    // Like Task 0.1 but with cell-backing on: define a global var holding a heap
    // object, drop other roots, force GC, assert it survives and is readable.
}
#[test]
fn unreferenced_cell_is_freed_and_registry_entry_dropped() {
    // After a cell-backed global is deleted (Phase 5), force GC and a watchpoint
    // sweep; assert cell_watchpoints no longer contains the freed cell.
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p lyng-tests cell_backed_global_object_value_survives_collection unreferenced_cell_is_freed_and_registry_entry_dropped`
Expected: FAIL if dictionary cell refs aren't traced (per Task 0.1), or registry entries leak.

- [ ] **Step 3: Trace cell refs + sweep registry**

Ensure the GC marks: (a) each `DataCell` cell ref in a cell-backed dictionary
entry (extend the path identified in Task 0.1 so the cell — and transitively its
`stored_value`, already handled by `mark_value_cell` — is marked), and (b) each
cell in the global env `lexical_cells` map (the global env is rooted; mark its
cells). Add a `sweep_invalidated_cell_watchpoints()` (mirror
`sweep_invalidated_watchpoint_sets`) that drops registry entries whose cell has
been freed, and call it from the same place the shape sweep runs.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p lyng-tests cell_backed_global_object_value_survives_collection unreferenced_cell_is_freed_and_registry_entry_dropped`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/gc/src/rooting.rs crates/objects/src/runtime.rs
git commit -m "feat(gc): trace cell-backed global cells; sweep cell watchpoint registry"
```

---

## Phase 7 — Verification & performance

### Task 7.1: Full correctness + performance sweep

**Files:** none (verification only).

- [ ] **Step 1: Full test262, compared to baseline**

Run: `cargo run -p lyng-test262 -- 2>&1 | tail -30`
Expected: pass count ≥ the pre-Phase-0 baseline; **zero new failures.** If any
regression, fix before claiming done.

- [ ] **Step 2: External bundled suite (the regression we set out to fix)**

Run: `cargo build --release -p lyng-cli && (cd ../js-engine-benchmark && bash scripts/run-lyng.sh)`
Expected: **RayTrace recovers from ~55 toward ~260**; overall Score materially up.

- [ ] **Step 3: Global-count sweep flattens**

Re-run the investigation microbenchmark (inject N empty globals before a clean
RayTrace, N ∈ {0, 512, 2000, 8000}). Expected: RayTrace score stays ~flat across
N instead of the prior 261→72 decline.

- [ ] **Step 4: No regression on the isolated internal suite**

Run: `cargo run --release -p lyng-bench -- v8suite`
Expected: scores within noise of the pre-change report
(`reports/lyng/bench-v8.md`); no benchmark regresses materially.

- [ ] **Step 5: Refresh the external comparison report + commit**

Run: `cargo run --release -p lyng-bench -- compare --report reports/lyng/external-engine-compare.md`

```bash
git add reports/lyng/
git commit -m "chore: refresh bench reports after global property cells (M1)"
```

---

## Self-Review notes (for the executor)

- **Constant folding payoff in M1 is modest** (saves one cell deref per read) because `LoadGlobal` runs through the Rust probe, not a `PropertyMetadata`-mode asm fast path. The constness/watchpoint machinery is built primarily as the foundation for M2 and a future dedicated asm fast path. If time-boxed, Phases 1–3 alone fix the regression; Phases 4–5 complete the spec.
- **Two dictionary payload representations** (`Data` vs `DataCell`) coexist; only the global object carries `CELL_BACKED_DICTIONARY` in M1. Never produce `DataCell` for non-cell-backed objects.
- **Order invariant for invalidation:** always drain a cell's dependents *before* freeing the cell.
- **TDZ sentinel:** confirm the exact uninitialized marker the codebase uses (`Value::empty_internal_slot()` is referenced by `alloc_value_cell`); reuse it consistently for lexical TDZ and the IC hit-path TDZ check.
- **Baseline first:** record the test262 pass count on `main` before Phase 0 so every "no new failures" check is meaningful.
