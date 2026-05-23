# LLInt GetNamedProperty Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the `GetNamedProperty` LLInt Rust probe with a true DSL asm hit path for monomorphic own-data inline-slot property loads.

**Architecture:** Add the missing asm-visible object-record pointer table first, then add a compact flat IC header that the `FV` pin can read without touching Rust enums. The initial opcode port handles the common monomorphic OwnData inline-slot case in LLInt and falls back to the counted semantic slow path for out-of-line slots, proto, polymorphic, megamorphic, non-object receivers, and invalidated caches.

**Tech Stack:** Rust, AArch64 `naked_asm!` DSL backend, `lyng_gc` object storage, `lyng_vm` feedback flat storage.

---

## File Structure

- Modify `crates/gc/src/arena/storage.rs`: expose stable `SlotArena` record pointers for occupied slots.
- Modify `crates/gc/src/arena.rs`: maintain an object-record pointer table indexed by raw `ObjectRef`.
- Modify `crates/gc/src/arena/records.rs`: make `RuntimeObjectRecord` offset-safe for generated asm constants and expose object-record field offsets.
- Modify `crates/gc/src/mutator.rs`: expose the object-record pointer table through `PrimitiveHeapView`.
- Modify `crates/vm/src/dsl/llint_state.rs`: add an asm-visible object-record table pointer to `LlIntState`.
- Modify `crates/vm/src/dsl/entry.rs` and `crates/vm/src/dsl/slow_path.rs`: populate and refresh that pointer at trampoline entry and every slow-path egress.
- Modify `crates/vm/src/dsl/reg_convention.rs`: expose the new `LlIntState` offset and imported object-record offsets.
- Modify `crates/vm/src/dsl/feedback_flat.rs`: add a small IC header in front of the legacy `state`.
- Modify `crates/vm/src/vm/feedback.rs`: mirror named-property monomorphic OwnData inline-slot cache data into the flat IC header.
- Modify `crates/objects/src/shapes.rs`: expose raw bits from `NamedPropertyHandler`.
- Modify `crates/vm-dsl/src/lower.rs`: bind the new feedback and object offsets into every `naked_asm!`.
- Modify `crates/vm/src/dsl/backend/aarch64/objects.rs`: replace the placeholder `vm_heap_pool` path with object-record table loads from `LlIntState`.
- Modify `crates/vm/src/dsl/backend/aarch64/feedback.rs`: add named-property IC header load/check macros.
- Modify `crates/vm/src/dsl/handlers/cold.rs`: port `op_get_named_property_dsl` monomorphic inline-slot hit to DSL and remove its Rust probe bridge.
- Modify `crates/vm/src/tests/llint_architecture.rs`: reduce and enumerate the remaining Rust probes.
- Modify `reports/lyng/llint-fast-path-audit-2026-05-23.md`: update the remaining-probe list after `GetNamedProperty` is ported.

## Task 1: Object-Record Pointer Table

**Files:**
- Modify: `crates/gc/src/arena/storage.rs`
- Modify: `crates/gc/src/arena.rs`
- Modify: `crates/gc/src/arena/records.rs`
- Modify: `crates/gc/src/mutator.rs`

- [ ] **Step 1: Add a focused GC test for object pointer table stability**

Add an inline test in `crates/gc/src/arena.rs` that allocates two objects, reads the table base, verifies `base[object.get()]` points at the same record as `object_ref(object)`, then allocates enough objects to grow the table and verifies the original object pointer still dereferences to the original shape.

- [ ] **Step 2: Run the test and verify it fails**

Run:

```sh
cargo test -p lyng-gc object_record_pointer_table_tracks_allocated_objects
```

Expected before implementation: compile failure for the missing pointer-table accessor.

- [ ] **Step 3: Implement pointer access in `SlotArena`**

Add a private helper:

```rust
pub(super) fn get_ptr(&self, handle: Handle) -> Option<*const Record> {
    let (page_index, slot_index) = locate::<Handle>(handle)?;
    self.pages
        .get(page_index)?
        .get_ref(slot_index)
        .map(|record| record as *const Record)
}
```

- [ ] **Step 4: Add the object pointer table to `PrimitiveHeap`**

Add `object_record_ptrs: Vec<*const RuntimeObjectRecord>` to `PrimitiveHeap`, initialize it with `Vec::new()`, and update `alloc_object`:

```rust
let id = self.objects.allocate(record, lifetime, generation);
let index = id.get() as usize;
if self.object_record_ptrs.len() <= index {
    self.object_record_ptrs.resize(index + 1, std::ptr::null());
}
self.object_record_ptrs[index] = self.objects.get_ptr(id).unwrap_or(std::ptr::null());
```

Add:

```rust
pub(crate) fn object_record_ptr_table(&self) -> *const *const RuntimeObjectRecord {
    self.object_record_ptrs.as_ptr()
}
```

The table uses index `0` as a null sentinel because all runtime IDs are non-zero.

- [ ] **Step 5: Expose the table through `PrimitiveHeapView`**

Add:

```rust
pub fn object_record_ptr_table(self) -> *const *const RuntimeObjectRecord {
    self.heap.object_record_ptr_table()
}
```

- [ ] **Step 6: Pin object-record field offsets**

Add `#[repr(C)]` to `RuntimeObjectRecord` and public offset constants in `crates/gc/src/arena/records.rs`:

```rust
pub const RUNTIME_OBJECT_SHAPE_OFFSET: usize = core::mem::offset_of!(RuntimeObjectRecord, shape);
pub const RUNTIME_OBJECT_NAMED_SLOTS_OFFSET: usize = core::mem::offset_of!(RuntimeObjectRecord, named_slots);
pub const RUNTIME_OBJECT_LAST_INVALIDATION_EPOCH_OFFSET: usize =
    core::mem::offset_of!(RuntimeObjectRecord, last_invalidation_epoch);
pub const RUNTIME_OBJECT_INLINE_NAMED_SLOTS_OFFSET: usize =
    core::mem::offset_of!(RuntimeObjectRecord, inline_named_slots);
```

- [ ] **Step 7: Run GC tests**

Run:

```sh
cargo test -p lyng-gc object_record_pointer_table_tracks_allocated_objects
```

Expected: pass.

## Task 2: Add Object Table To The LLInt ABI

**Files:**
- Modify: `crates/vm/src/dsl/llint_state.rs`
- Modify: `crates/vm/src/dsl/entry.rs`
- Modify: `crates/vm/src/dsl/slow_path.rs`
- Modify: `crates/vm/src/dsl/reg_convention.rs`
- Modify: `crates/vm-dsl/src/lower.rs`

- [ ] **Step 1: Extend the ABI test**

Update `ll_int_state_offsets_stable` to include the new object table pointer and the new total size.

- [ ] **Step 2: Add the field**

Insert this field in `LlIntState` after `frame_fv_base`:

```rust
pub object_records_base: *const *const lyng_gc::RuntimeObjectRecord,
```

- [ ] **Step 3: Populate at entry**

In `run_via_dsl`, derive:

```rust
let object_records_base = agent.heap().view().object_record_ptr_table();
```

Store it into `LlIntState`.

- [ ] **Step 4: Refresh at every slow-path egress**

In both `SemanticOutcome::Continue` and `SemanticOutcome::Refresh`, write:

```rust
(**state).object_records_base = rust.dispatch.agent.heap().view().object_record_ptr_table();
```

This is required because object allocation can grow the pointer table even when the slow path returns `Continue`.

- [ ] **Step 5: Bind offsets into the lowerer**

Add named bindings in `crates/vm-dsl/src/lower.rs` for:

- `state_object_records`
- `object_shape`
- `object_last_epoch`
- `object_inline_slots`
- `feedback_entry_stride`
- `feedback_mode`
- `feedback_named_handler_bits`
- `feedback_named_epoch`

Reference them in the leading comment string so unused bindings do not warn.

- [ ] **Step 6: Run VM ABI tests**

Run:

```sh
cargo test -p lyng-vm llint_state
```

Expected: pass after updating expected offsets.

## Task 3: Flat IC Header For Named OwnData Inline Loads

**Files:**
- Modify: `crates/vm/src/dsl/feedback_flat.rs`
- Modify: `crates/vm/src/vm/feedback.rs`
- Modify: `crates/objects/src/shapes.rs`

- [ ] **Step 1: Add a failing mirror test**

Add a VM unit test that warms `source.value`, fetches the flat feedback entry for its slot through a `#[cfg(test)]` accessor, and asserts:

```rust
assert_eq!(entry.mode(), LlIntIcMode::NamedOwnInlineLoad);
assert_ne!(entry.named_handler_bits(), 0);
assert_eq!(entry.named_epoch(), 0);
```

- [ ] **Step 2: Add the IC header**

Change `FeedbackEntry` to:

```rust
#[repr(u8)]
pub(crate) enum LlIntIcMode {
    Empty = 0,
    NamedOwnInlineLoad = 1,
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeedbackEntry {
    pub(crate) mode: u8,
    pub(crate) _pad: [u8; 7],
    pub(crate) named_handler_bits: u64,
    pub(crate) named_epoch: u64,
    pub(crate) state: Option<FeedbackSiteState>,
}
```

Keep `Default` as all-zero mode/header plus `state: None`.

Also add public-in-crate offset constants for the header fields and bind
`size_of::<FeedbackEntry>()` into the asm lowerer. Do not use the old
`entry_stride_shift = 6` placeholder for IC reads; `FeedbackEntry` is not
guaranteed to be 64 bytes and the LLInt must compute `slot * actual_stride`.

- [ ] **Step 3: Expose handler bits**

Add to `NamedPropertyHandler`:

```rust
#[inline]
#[must_use]
pub const fn bits(self) -> u64 {
    self.0
}
```

- [ ] **Step 4: Mirror only the small header**

Change `mirror_flat_slot` so it does not clone the legacy enum. It should clear the header by default, then set `mode`, `named_handler_bits`, and `named_epoch` only when the legacy slot is `NamedProperty` with a valid monomorphic OwnData handler whose slot offset is inline.

- [ ] **Step 5: Run the mirror test**

Run:

```sh
cargo test -p lyng-vm flat_named_property_header_tracks_monomorphic_inline_load
```

Expected: pass.

## Task 4: AArch64 Macros For Named OwnData Inline Loads

**Files:**
- Modify: `crates/vm/src/dsl/backend/aarch64/objects.rs`
- Modify: `crates/vm/src/dsl/backend/aarch64/feedback.rs`

- [ ] **Step 1: Add object-record table macros**

Replace the placeholder VM heap-pool assumption with:

```rust
load_object_record_from_state!(object_ref_reg => record_reg)
```

The macro loads `LlIntState.object_records_base`, then loads `base[object_ref]` as a pointer and branches to slow if it is null.

- [ ] **Step 2: Add IC-header macros**

Add macros for:

- `load_feedback_entry!(slot => entry)`
- `branch_named_own_inline_mode!(entry, slow)`
- `load_named_handler_bits!(entry => handler)`
- `load_named_epoch!(entry => epoch)`

- [ ] **Step 3: Decode handler bits in asm**

The handler layout is:

- high 32 bits: receiver shape raw
- bit 31 in low half: inline-slot flag
- bit 30 in low half: writable flag, ignored for load
- low 30 bits: slot offset

The load path must reject `handler == 0`, reject missing inline bit, compare receiver shape, compare invalidation epoch, then load `inline_named_slots[offset]`.

`load_feedback_entry!` must materialize `{feedback_entry_stride}` and use
`madd`/`mul+add`, not `lsl {entry_stride_shift}`. Existing observation
recording macros may continue to share the same helper, but no new IC reader
may depend on a hard-coded stride shift.

- [ ] **Step 4: Compile the macro users**

Run:

```sh
cargo test -p lyng-vm-dsl
```

Expected: pass.

## Task 5: Port `op_get_named_property_dsl`

**Files:**
- Modify: `crates/vm/src/dsl/handlers/cold.rs`
- Modify: `crates/vm/src/tests/llint_architecture.rs`
- Modify: `reports/lyng/llint-fast-path-audit-2026-05-23.md`

- [ ] **Step 1: Update the architecture test first**

Change the known `call_rust_probe!` count from `3` to `2`, and make the failure message enumerate the remaining allowed probes: `LoadGlobal` and `AssignNamedProperty`.

- [ ] **Step 2: Run the test and verify it fails**

Run:

```sh
cargo test -p lyng-vm llint_rust_probes_are_explicitly_enumerated
```

Expected before port: fail because `GetNamedProperty` still uses a Rust probe.

- [ ] **Step 3: Replace the handler body**

`op_get_named_property_dsl` should:

1. Load receiver from register `b`.
2. Check `ObjectRef`, else `.slow`.
3. Load flat feedback entry from `slot`.
4. Check mode `NamedOwnInlineLoad`, else `.slow`.
5. Load handler bits and reject zero / out-of-line.
6. Load object record from `LlIntState.object_records_base`.
7. Compare receiver shape against handler high half.
8. Compare `last_invalidation_epoch` against flat `named_epoch`.
9. Load `inline_named_slots[offset]` into register `a`.
10. `dispatch!()`.
11. `.slow:` call `op_get_named_property_slow_rs`.

Remove `op_get_named_property_rust_probe_rs` and `Vm::try_get_named_property_rust_probe_for_dsl`.

- [ ] **Step 4: Run the targeted opcode-counter test**

Run:

```sh
cargo test -p lyng-vm --features opcode-counters named_property_load_ic_hit_avoids_semantic_slow_path
```

Expected: pass with `GetNamedProperty` dispatch count `1` and semantic slow-path count `0`.

- [ ] **Step 5: Update the audit report**

Remove `GetNamedProperty` from the remaining Rust-probe table. Add a note that the current LLInt path covers monomorphic OwnData inline slots only and intentionally falls back to counted semantic slow path for other IC modes.

## Task 6: Verification And Commit

**Files:**
- All modified files from Tasks 1-5.

- [ ] **Step 1: Format touched crates**

Run:

```sh
cargo fmt -p lyng-gc -p lyng-objects -p lyng-vm -p lyng-vm-dsl
```

- [ ] **Step 2: Run focused tests**

Run:

```sh
cargo test -p lyng-gc object_record_pointer_table_tracks_allocated_objects
cargo test -p lyng-vm-dsl
cargo test -p lyng-vm llint_
cargo test -p lyng-vm --features opcode-counters named_property_load_ic_hit_avoids_semantic_slow_path
```

- [ ] **Step 3: Run broader VM library tests**

Run:

```sh
cargo test -p lyng-vm --lib
cargo test -p lyng-vm --features opcode-counters --lib
```

- [ ] **Step 4: Run release Richards**

Run:

```sh
cargo run --release -p lyng-bench -- v8suite --filter Richards --timeout-secs 120
cargo run --release -p lyng-bench -- v8suite --filter Richards --timeout-secs 120 --count-opcodes --count-slow-path-share --counts-json /tmp/lyng-richards-getnamed-counts.json
```

- [ ] **Step 5: Commit**

Commit message:

```sh
git commit -m "Port GetNamedProperty IC hit to LLInt"
```
