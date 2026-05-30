# Asm-Inline Global Cell Load (mode 7) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Serve the `LoadGlobal` cell-IC *hit* inline in the asm handler (validity-check + one cell load) instead of via a per-dispatch Rust probe, dropping `LoadGlobal` from ~121 samples/Mdispatch toward `GetNamedProperty`'s ~33 on Richards.

**Architecture:** Project the already-cached global cell resolution into the asm-readable `PropertyMetadata` as a new `mode 7 = GlobalCellLoad` (`handler_bits` = cell ref, `generation` = captured global-IC generation). Add the two heap reads the hit needs — a stable value-cell pointer table and a Vm-field mirror of the live global generation — then add the asm hit path. Staged: Rust foundation + a thin-probe win (skips name-read/env-walk/hashmap) land first; the fully-inline asm path is the capstone. The unchanged Rust cold path remains the fallback for every case the asm doesn't serve.

**Tech Stack:** Rust; the aarch64 asm-DSL substrate (`crates/vm/src/dsl`); GC slot arenas (`crates/gc`); the `lyng-bench profile` tool for before/after measurement.

---

## Background the implementer must know

- **This is the asm slice of an approved design.** Spec: `docs/superpowers/specs/2026-05-30-global-cell-asm-load-design.md`. Parent: `docs/superpowers/specs/2026-05-29-global-property-cells-design.md`. The global cell IC (`GlobalCellIcState` / `GlobalCellTarget::{Cell,EnvSlot}` in `crates/vm/src/vm/ic_state/global_cell.rs`) already caches *where* a `LoadGlobal` resolves; this plan moves the *hit* into asm. Constant folding (mode 6), `EnvSlot` inline, and `StoreGlobal` are explicitly OUT of scope.
- **PropertyMetadata** (`crates/vm/src/vm/metadata_table/property.rs`) is `#[repr(C)]`: `mode: u8` @0, `generation: u32` @4, `handler_bits: u64` @8, `aux_bits: u64` @16, `execution_count: u32` @24; stride 32 (`PROPERTY_METADATA_STRIDE_SHIFT = 5`). Offset consts are re-exported to `crates/vm/src/dsl/reg_convention.rs` as `PROPERTY_METADATA_MODE_OFFSET` (0), `_HANDLER_BITS_OFFSET` (8), `_AUX_BITS_OFFSET` (16). Modes 0–5 are taken; **6 and 7 are free**.
- **The asm reads the same metadata buffer** the Rust side writes: `x21` = `frame_metadata_table_base`; `load_feedback_site!(slot => c)` resolves a slot id to the entry pointer; the mode-1 `OwnInline` path in `op_get_named_property_dsl` (`crates/vm/src/dsl/handlers/cold.rs:2622`) is the handler to mirror.
- **The current LoadGlobal handler** `op_load_global_dsl` (`crates/vm/src/dsl/handlers/cold.rs:476`) is `call_rust_probe!(op_load_global_rust_probe_rs, ...)` + `branch_nonzero!(0,.slow)` + `dispatch_probe_hit_no_refresh!()` / `.slow: call_slow!(...)`. The probe is `try_load_global_rust_probe_for_dsl` (`crates/vm/src/vm/names.rs:663`).
- **Cells:** `value_cells: SlotArena<PrimitiveValueCellRecord, PrimitiveValueCellRef>`. `PrimitiveValueCellRef` is `#[repr(transparent)] NonZeroU32`, a 1-based flat index (`crates/gc/src/arena/records.rs:241`). `PrimitiveValueCellRecord` (`records.rs:315`) holds `stored_value: Value` (a `#[repr(transparent)] u64`) — currently at byte offset 0 *by luck* (NOT `#[repr(C)]`). The arena does NOT move records on GC sweep (slots freed in place), so a live cell's address is stable until freed — the generation guard covers the freed case. There is **no** value-cell pointer table yet (objects have one: `object_record_ptrs`, `crates/gc/src/arena.rs:40`).
- **Generation:** `global_structure_generation` (a `u32` in `EnvironmentMetadata::Global`, `crates/env/src/agent/environments.rs:562`), bumped at `:577` by `bump_global_structure_generation`. Four call sites: `crates/vm/src/vm.rs:1784`, `crates/vm/src/vm/names.rs:1554` (both Vm-side, during dispatch), and `crates/env/src/agent.rs:401,451` (agent-side, declaration-time). The asm cannot call the agent, so we mirror the live generation into a Vm field read by `x22`.
- **Build/measure:** `cargo test -p lyng-vm` (and `-p lyng-gc`, `-p lyng-bench`); `lyng-bench` always has the engine; measure with `cargo run --release -p lyng-bench -- profile --filter Richards --samples 5 --report /tmp/g.md --json /tmp/g.json`. The repo keeps pedantic/nursery clippy clean and `cargo fmt` clean.
- **One intentional refinement vs the spec:** the spec sketched `aux_bits = cached generation`. This plan instead stores the captured generation in `PropertyMetadata`'s **dedicated `generation` field** (offset 4, already present and used by the named-property IC) and leaves `aux_bits = 0`. Cleaner and consistent with the existing IC; the asm reads the generation from offset 4.
- **CRITICAL correctness invariant (carried through the whole plan):** a `mode 7` asm hit dereferences a cached cell ref guarded only by `metadata.generation == live_generation_mirror`. Therefore **every** `bump_global_structure_generation` must be paired with a refresh of the Vm mirror *before the next mode-7 hit*. Task 4 establishes the mirror + coherent refresh; Task 8's staleness test is the backstop.

---

## File Structure

- `crates/gc/src/arena/records.rs` — `#[repr(C)]` on `PrimitiveValueCellRecord` + a `stored_value` offset const (Task 1).
- `crates/gc/src/arena.rs`, `crates/gc/src/mutator.rs` — `value_cell_ptrs` table + `value_cell_ptr_table()` accessor, maintained in alloc/free (Task 2).
- `crates/vm/src/vm/metadata_table/property.rs` — `LLINT_IC_MODE_GLOBAL_CELL_LOAD = 7` + `PROPERTY_METADATA_GENERATION_OFFSET` (Task 3).
- `crates/vm/src/vm/feedback.rs` — `project_global_cell_load_into_meta` + the install helper (Tasks 3, 5).
- `crates/vm/src/vm.rs` — `dsl_global_ic_generation` Vm field + refresh helper (Task 4).
- `crates/vm/src/vm/names.rs` — coherent refresh at Vm-side bumps; cold-path mode-7 install; the thin-probe fast read (Tasks 4, 5, 6).
- `crates/env/src/agent.rs` — confirm/handle the two agent-side bump sites (Task 4).
- `crates/vm/src/dsl/llint_state.rs`, `reg_convention.rs`, `entry.rs` — value-cell table base binding + Vm-generation offset binding (Task 7).
- `crates/vm/src/dsl/backend/aarch64/feedback.rs` (+ `objects.rs`) — mode-7 + cell-deref DSL macros (Task 8).
- `crates/vm/src/dsl/handlers/cold.rs` — the rewritten `op_load_global_dsl` (Task 8).

---

## Task 1: `#[repr(C)]` `PrimitiveValueCellRecord` + stored-value offset

**Files:**
- Modify: `crates/gc/src/arena/records.rs:315`
- Test: a new `#[cfg(test)]` block in the same file (or `crates/gc/tests/` mirroring existing layout tests).

- [ ] **Step 1: Write the failing layout test**

Add to `crates/gc/src/arena/records.rs` (bottom, in a `#[cfg(test)] mod value_cell_layout_tests`):

```rust
#[cfg(test)]
mod value_cell_layout_tests {
    use super::{PrimitiveValueCellRecord, PRIMITIVE_VALUE_CELL_RECORD_STORED_VALUE_OFFSET};
    use std::mem::offset_of;

    #[test]
    fn stored_value_is_at_pinned_offset() {
        // The asm mode-7 hit loads the cell value from
        // [record_ptr, #PRIMITIVE_VALUE_CELL_RECORD_STORED_VALUE_OFFSET].
        assert_eq!(
            offset_of!(PrimitiveValueCellRecord, stored_value),
            PRIMITIVE_VALUE_CELL_RECORD_STORED_VALUE_OFFSET
        );
    }
}
```

- [ ] **Step 2: Run it — expect compile failure**

Run: `cargo test -p lyng-gc value_cell_layout`
Expected: FAIL — `cannot find value PRIMITIVE_VALUE_CELL_RECORD_STORED_VALUE_OFFSET`.

- [ ] **Step 3: Add `#[repr(C)]` + the offset const**

In `crates/gc/src/arena/records.rs`, add `#[repr(C)]` to the struct and define the const next to it:

```rust
/// Byte offset of `stored_value` within `PrimitiveValueCellRecord`. The asm
/// mode-7 GlobalCellLoad hit loads the cell's `Value` from this offset. `Value`
/// is `#[repr(transparent)] u64`; `#[repr(C)]` keeps `stored_value` first so
/// this stays 0. Pinned by `value_cell_layout_tests`.
pub const PRIMITIVE_VALUE_CELL_RECORD_STORED_VALUE_OFFSET: usize = 0;

#[repr(C)]
pub struct PrimitiveValueCellRecord {
    pub(super) stored_value: Value,
    pub(super) linked_string: Option<StringRef>,
}
```

(Keep the existing field order — `stored_value` must remain first so the offset is 0.)

- [ ] **Step 4: Run the test — expect PASS**

Run: `cargo test -p lyng-gc value_cell_layout`
Expected: PASS.

- [ ] **Step 5: Confirm no wider breakage + the offset is exported**

Ensure the const is reachable from the vm crate later: it must be `pub` and re-exported if `records` is private. Check `crates/gc/src/lib.rs` exports `PrimitiveValueCellRecord`; add `PRIMITIVE_VALUE_CELL_RECORD_STORED_VALUE_OFFSET` to the same `pub use`.

Run: `cargo build -p lyng-gc && cargo test -p lyng-gc`
Expected: success.

- [ ] **Step 6: Commit**

```bash
git add crates/gc/src/arena/records.rs crates/gc/src/lib.rs
git commit -m "feat(gc): repr(C) PrimitiveValueCellRecord + pinned stored_value offset

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: Value-cell pointer table

Mirror the existing `object_record_ptrs` pattern so a `PrimitiveValueCellRef` can be resolved to a stable `*const PrimitiveValueCellRecord` the asm can dereference.

**Files:**
- Modify: `crates/gc/src/arena.rs` (the `PrimitiveHeap` struct + `alloc_value_cell` / free path)
- Modify: `crates/gc/src/mutator.rs` (expose `value_cell_ptr_table()` on the view)
- Test: `crates/gc/tests/` or an in-file test.

- [ ] **Step 1: Write the failing test**

Add to `crates/gc/src/mutator.rs` (in its `#[cfg(test)] mod tests`, mirroring how object-table tests are written there if present; otherwise a new module):

```rust
#[cfg(test)]
mod value_cell_ptr_table_tests {
    use crate::PrimitiveHeap;
    use lyng_common::Value;

    #[test]
    fn ptr_table_resolves_live_cell_to_its_record() {
        let mut heap = PrimitiveHeap::new();
        let cell = heap.alloc_value_cell(Value::from_i32(42), None);
        let table = heap.view().value_cell_ptr_table();
        // Table is indexed by the 1-based ref (entry 0 unused), like object_record_ptrs.
        let ptr = table[cell.get() as usize];
        assert!(!ptr.is_null());
        // SAFETY: cell is live; ptr was just published by alloc.
        let record = unsafe { &*ptr };
        assert_eq!(record.stored_value(), Value::from_i32(42));
    }
}
```

(Adjust `alloc_value_cell` / `PrimitiveHeap::new` / `Value::from_i32` to the actual signatures — read `crates/gc/src/mutator.rs:189` and the existing `alloc_value_cell` to match. The assertion that matters: a freshly allocated cell's table entry is non-null and points to a record with the right `stored_value`.)

- [ ] **Step 2: Run it — expect failure**

Run: `cargo test -p lyng-gc value_cell_ptr_table`
Expected: FAIL — no method `value_cell_ptr_table`.

- [ ] **Step 3: Add the table, maintain it, expose it**

In `crates/gc/src/arena.rs`, next to `object_record_ptrs: Vec<*const RuntimeObjectRecord>` (~line 40), add:

```rust
/// Stable `*const PrimitiveValueCellRecord` per `PrimitiveValueCellRef` (1-based;
/// entry 0 unused), mirroring `object_record_ptrs`. The asm mode-7 GlobalCellLoad
/// hit indexes this with a cached cell ref to reach the record without calling
/// the mutator. Populated on alloc, nulled on free.
value_cell_ptrs: Vec<*const PrimitiveValueCellRecord>,
```

Initialize it (mirror `object_record_ptrs` init) with a single null sentinel at index 0. In `alloc_value_cell` (where the cell record is created), after the slot is materialized, publish the record pointer at `value_cell_ptrs[ref.get()]` (growing the Vec with nulls as needed — copy the exact grow/insert idiom from `alloc_object` at `arena.rs:592`). In the value-cell free/sweep path (mirror where `object_record_ptrs[ref] = null` happens on object free), null the entry.

> **Implementer note:** read `alloc_object` (`arena.rs:592-598`) and the object free/sweep path verbatim and mirror them for value cells. The pointer must be the address of the record *inside its `SlotPage`* — the same address `value_cell(ref)`/`get_ptr` would compute. If a `get_ptr`-style accessor exists for the slot arena (the explorer noted `SlotArena::get_ptr` is `pub(super)`), use it; otherwise publish the pointer at the point of record construction.

In `crates/gc/src/mutator.rs`, expose on the heap view (mirror `object_record_ptr_table()` / `object_slots_ptr_table()`):

```rust
#[must_use]
pub fn value_cell_ptr_table(&self) -> &[*const PrimitiveValueCellRecord] {
    &self.heap.value_cell_ptrs
}
```

(Match the exact view-accessor pattern used for `object_record_ptr_table`.)

- [ ] **Step 4: Run the test — expect PASS**

Run: `cargo test -p lyng-gc value_cell_ptr_table`
Expected: PASS.

- [ ] **Step 5: GC-coherence test — freed cell nulls its entry**

Add a second test: allocate a cell, free it through the normal free/collect path, assert `value_cell_ptr_table()[ref]` is null (or that the slot was reclaimed). This guards against a dangling pointer surviving in the table. If value cells are only reclaimed by full GC, drive a collection; match how the object-table free test (if any) does it. If there is no public free path in a unit test, assert at minimum that re-allocation reuses the slot and republishes a fresh pointer.

Run: `cargo test -p lyng-gc value_cell_ptr_table`
Expected: PASS (both tests).

- [ ] **Step 6: Full gc suite + commit**

Run: `cargo test -p lyng-gc`
Expected: PASS.

```bash
git add crates/gc/src/arena.rs crates/gc/src/mutator.rs
git commit -m "feat(gc): value-cell pointer table mirroring object_record_ptrs

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Mode-7 constant + metadata projection (Rust)

**Files:**
- Modify: `crates/vm/src/vm/metadata_table/property.rs`
- Modify: `crates/vm/src/vm/feedback.rs`

- [ ] **Step 1: Write the failing projection unit test**

In `crates/vm/src/vm/feedback.rs` (in its `#[cfg(test)] mod tests`, or a new one), add:

```rust
#[cfg(test)]
mod global_cell_projection_tests {
    use super::*;
    use crate::vm::metadata_table::{PropertyMetadata, LLINT_IC_MODE_GLOBAL_CELL_LOAD};

    #[test]
    fn project_global_cell_load_writes_mode_7_handler_and_generation() {
        let mut meta = PropertyMetadata::default();
        // cell ref 9, generation 3, execution_count carried as-is.
        Vm::project_global_cell_load_into_meta(9, 3, 11, &mut meta);
        assert_eq!(meta.mode, LLINT_IC_MODE_GLOBAL_CELL_LOAD);
        assert_eq!(meta.handler_bits, 9);
        assert_eq!(meta.generation, 3);
        assert_eq!(meta.execution_count, 11);
        assert_eq!(meta.aux_bits, 0);
    }
}
```

(If `PropertyMetadata` has no `Default`, construct a zeroed one the way the existing tests do.)

- [ ] **Step 2: Run it — expect failure**

Run: `cargo test -p lyng-vm global_cell_projection`
Expected: FAIL — unresolved `LLINT_IC_MODE_GLOBAL_CELL_LOAD` / `project_global_cell_load_into_meta`.

- [ ] **Step 3: Add the mode constant + generation offset**

In `crates/vm/src/vm/metadata_table/property.rs`, next to `LLINT_IC_MODE_NAMED_OWN_INLINE_WRITE` (= 5):

```rust
/// Asm IC mode: global cell load. `handler_bits` = the `PrimitiveValueCellRef`
/// raw u32; `generation` = the global-IC generation captured at install. The asm
/// hit loads the cell value when `generation` matches the live Vm mirror.
pub const LLINT_IC_MODE_GLOBAL_CELL_LOAD: u8 = 7;
```

Ensure a `PROPERTY_METADATA_GENERATION_OFFSET` (= 4) const exists and is exported alongside the other `PROPERTY_METADATA_*_OFFSET` consts (add it if absent; the asm will read it in Task 8).

- [ ] **Step 4: Add the projection function**

In `crates/vm/src/vm/feedback.rs`, next to `project_property_into_meta`:

```rust
/// Project a resolved global cell load into asm-readable `PropertyMetadata`
/// (`mode 7`). `cell_ref_raw` is `PrimitiveValueCellRef::get()`; `generation` is
/// the global-IC generation captured at resolution (compared against the live
/// Vm mirror on the asm hit). `aux_bits` is unused (0) for mode 7.
const fn project_global_cell_load_into_meta(
    cell_ref_raw: u32,
    generation: u32,
    execution_count: u32,
    meta: &mut PropertyMetadata,
) {
    meta.mode = crate::vm::metadata_table::LLINT_IC_MODE_GLOBAL_CELL_LOAD;
    meta.generation = generation;
    meta.handler_bits = cell_ref_raw as u64;
    meta.aux_bits = 0;
    meta.execution_count = execution_count;
}
```

(If the test calls it as `Vm::project_global_cell_load_into_meta`, make it an associated fn on the same impl block as `project_property_into_meta`; match that fn's visibility/receiver convention — it is a free/assoc const fn, no `self`.)

- [ ] **Step 5: Run the test — expect PASS**

Run: `cargo test -p lyng-vm global_cell_projection`
Expected: PASS.

- [ ] **Step 6: Build + clippy + commit**

Run: `cargo build -p lyng-vm && cargo clippy -p lyng-vm --all-targets`
Expected: clean.

```bash
git add crates/vm/src/vm/metadata_table/property.rs crates/vm/src/vm/feedback.rs
git commit -m "feat(vm): add GlobalCellLoad mode 7 + metadata projection

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: Vm-field live-generation mirror + coherent refresh

The asm hit compares `metadata.generation` against a live generation it can read via `x22`. Establish that mirror on `Vm` and keep it coherent with all four bump sites. This task is correctness-critical and lands before any asm.

**Files:**
- Modify: `crates/vm/src/vm.rs` (field + refresh helper + run-entry baseline)
- Modify: `crates/vm/src/vm/names.rs:1554`, `crates/vm/src/vm.rs:1784` (Vm-side bumps refresh inline)
- Inspect: `crates/env/src/agent.rs:401,451` (agent-side bumps — confirm declaration-time)

- [ ] **Step 1: Write the failing coherence test**

In `crates/vm/src/tests/` (a global-cells test file, e.g. extend `crates/vm/src/tests/global_cells.rs`):

```rust
#[test]
fn vm_global_ic_generation_mirror_tracks_structural_bumps() {
    // Build a VM with a global, read the mirror, perform a structural change
    // that bumps the env generation (delete a configurable global), and assert
    // the Vm mirror advanced to match the live env generation.
    // <build runtime + vm + global env per the existing harness in this file>
    let before = vm.dsl_global_ic_generation();
    // run `var g = 1; delete globalThis.g;` (or the existing helper for a
    // structural change) so a bump fires on a Vm-side path:
    // <execute a script that deletes a configurable global property>
    let live = agent.global_structure_generation(global_env);
    assert_eq!(vm.dsl_global_ic_generation(), live, "mirror must equal live gen after a structural bump");
    assert!(vm.dsl_global_ic_generation() > before);
}
```

(Use the exact VM/agent/global-env construction already in `global_cells.rs`. The assertion that matters: after any structural change, `vm.dsl_global_ic_generation() == agent.global_structure_generation(global_env)`.)

- [ ] **Step 2: Run it — expect failure**

Run: `cargo test -p lyng-vm --features diagnostic-counters vm_global_ic_generation_mirror`
(or without the feature if the test doesn't need it)
Expected: FAIL — no method `dsl_global_ic_generation`.

- [ ] **Step 3: Add the Vm field + accessor + refresh helper**

In `crates/vm/src/vm.rs`, add a field to `Vm` (near `dsl_poll_pending` / the other DSL-read fields, so it shares the asm-reachable region):

```rust
/// Mirror of the executing realm's `global_structure_generation`, read by the
/// asm `LoadGlobal` mode-7 hit via `[x22, #VM_GLOBAL_IC_GENERATION_OFFSET]`.
/// MUST equal the live env generation at every mode-7 hit: refreshed at run
/// entry and immediately after every `bump_global_structure_generation`.
pub(crate) dsl_global_ic_generation: u32,
```

Initialize to 0 in the `Vm` constructor(s). Add:

```rust
#[inline]
pub(crate) fn dsl_global_ic_generation(&self) -> u32 {
    self.dsl_global_ic_generation
}

/// Refresh the mirror from the agent's live generation for `global_env`.
/// Call immediately after any structural change that may bump it, and at run
/// entry once the executing realm's global env is known.
#[inline]
pub(crate) fn refresh_global_ic_generation(&mut self, agent: &Agent, global_env: EnvironmentRef) {
    self.dsl_global_ic_generation = agent.global_structure_generation(global_env);
}
```

- [ ] **Step 4: Refresh at the two Vm-side bump sites**

At `crates/vm/src/vm.rs:1784` and `crates/vm/src/vm/names.rs:1554`, immediately after the `agent.bump_global_structure_generation(global_env)` call, add `self.refresh_global_ic_generation(agent, global_env);` (use the same `global_env`/`global.id()` value passed to the bump). These run during dispatch, so the mirror is coherent before the next opcode.

- [ ] **Step 5: Establish the run-entry baseline + handle agent-side bumps**

Inspect `crates/env/src/agent.rs:401,451`. These fire during global declaration instantiation (`var`/`function` global setup) which happens *before* the dispatch loop (during `instantiate_global_script`). Refresh the mirror once at the start of execution, after the realm's global env is resolved and after instantiation — locate the `evaluate_installed` / run entry in `vm.rs` and add a `self.refresh_global_ic_generation(agent, global_env)` there (derive `global_env` the same way the probe does via the realm's global environment). Document in a comment that declaration-time bumps are covered by this baseline and dispatch-time bumps by Step 4, so no mode-7 hit can observe a stale-low mirror.

> **Implementer note (correctness):** if inspection shows either agent-side site can fire *during* the dispatch loop (not just setup), that path must also refresh the mirror — escalate as a finding rather than assuming setup-only. The whole feature's safety rests on "mirror == live gen at every mode-7 hit."

- [ ] **Step 6: Run the test — expect PASS**

Run: `cargo test -p lyng-vm vm_global_ic_generation_mirror`
Expected: PASS.

- [ ] **Step 7: Full vm tests + commit**

Run: `cargo test -p lyng-vm`
Expected: PASS (no regressions — the mirror is currently read by nothing).

```bash
git add crates/vm/src/vm.rs crates/vm/src/vm/names.rs crates/vm/src/tests/global_cells.rs
git commit -m "feat(vm): mirror live global-IC generation on Vm, coherent with bumps

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: Cold path installs mode-7 metadata for Cell targets

**Files:**
- Modify: `crates/vm/src/vm/names.rs` (where `install_global_cell_ic` runs for a `Cell` target)
- Inspect: the feedback-slot kind for `LoadGlobal` (must allocate a `PropertyMetadata` entry)

- [ ] **Step 1: Verify LoadGlobal sites have a PropertyMetadata entry**

Read `MetadataKind::from_site_kind` / `MetadataTable::allocate` (grep `from_site_kind` and `MetadataKind` under `crates/vm/src/vm/metadata_table/`). Confirm the `FeedbackSiteKind` used by `LoadGlobal` maps to a `Property` metadata entry (so `property_mut(slot)` is valid for a LoadGlobal slot). If it does NOT, add that mapping (mirror how `NamedPropertyLoad` maps) — without an allocated entry, the projection in Step 3 would write into the wrong/no slot. Capture the finding in the commit message.

- [ ] **Step 2: Write the failing install test**

In `crates/vm/src/tests/global_cells.rs`:

```rust
#[test]
fn cold_path_cell_resolution_projects_mode_7_metadata() {
    use crate::vm::metadata_table::LLINT_IC_MODE_GLOBAL_CELL_LOAD;
    // Run a script that reads a cell-backed global twice so the cold path
    // resolves to a Cell target and installs the IC + metadata.
    // <build vm; execute `var g = 7; g; g;` (the 2nd read installs)>
    // Then read the site's PropertyMetadata for the LoadGlobal slot:
    let meta = vm.metadata_table(code).property(load_global_slot.get());
    assert_eq!(meta.mode, LLINT_IC_MODE_GLOBAL_CELL_LOAD);
    assert_ne!(meta.handler_bits, 0, "cell ref must be projected");
    assert_eq!(meta.generation, vm.dsl_global_ic_generation());
}
```

(Use the existing `global_cells.rs` harness to get `code` + the LoadGlobal `FeedbackSlotId`. If retrieving the exact slot is awkward, assert via a read accessor that the site reached mode 7 — match how existing tests in this file inspect IC state.)

- [ ] **Step 3: Run it — expect failure**

Run: `cargo test -p lyng-vm cold_path_cell_resolution_projects_mode_7`
Expected: FAIL — metadata mode is not 7 (still 0/empty or the named-property mode).

- [ ] **Step 4: Project mode 7 on Cell resolution**

In `crates/vm/src/vm/names.rs`, do this in **`load_global_with_feedback`** (`names.rs:523`) — the cold-path resolution+install that `op_load_global_slow_rs` calls. This placement is REQUIRED: Task 8 makes the asm `.slow` path route to `op_load_global_slow_rs` (not the probe), so the mode-7 install must be reachable from there or the asm would never see mode 7 and would bail forever. Find where it resolves a site to `GlobalCellTarget::Cell(cell)` and calls `install_global_cell_ic(code, slot, GlobalCellTarget::Cell(cell), structure_gen)`. Immediately after, write the site metadata:

```rust
if let Some(slot) = feedback_slot {
    let exec_count = /* current execution count for the slot, as the named-property
                        install reads it — match observe_named_property_slow_path */;
    if let Some(table) = self.metadata_table_mut(code) {
        let meta = table.property_mut(slot.get());
        Self::project_global_cell_load_into_meta(cell.get(), structure_gen, exec_count, meta);
    }
}
```

Only do this for the `Cell` target — `EnvSlot` resolutions must NOT set mode 7 (leave their metadata untouched so the asm bails to the cold path). Match the exact `metadata_table_mut` / `property_mut` access pattern used by `observe_named_property_slow_path` (`feedback.rs:999`).

- [ ] **Step 5: Run the test — expect PASS**

Run: `cargo test -p lyng-vm cold_path_cell_resolution_projects_mode_7`
Expected: PASS.

- [ ] **Step 6: Regression + commit**

Run: `cargo test -p lyng-vm`
Expected: PASS (nothing reads mode 7 yet, so behavior is unchanged).

```bash
git add crates/vm/src/vm/names.rs crates/vm/src/tests/global_cells.rs crates/vm/src/vm/metadata_table/*.rs
git commit -m "feat(vm): cold path projects mode-7 metadata on global Cell resolution

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 6: Rust thin-probe fast read (measurable win #1)

Before touching asm, make the Rust probe serve mode-7 sites by reading the projected metadata directly — skipping `read_atom_constant`, `find_global_environment_ref`, and the `HashMap` lookup. This is the spec's documented fallback, a real win, and validates the projection end-to-end. The hit still goes through the probe call; Task 8 removes that.

> **Stepping-stone note:** this task's fast read lives in the probe (`try_load_global_rust_probe_for_dsl`). Task 8 stops calling the probe (asm serves the hit) and deletes it, so this code is intentionally superseded. Its value is (a) an early, validated, measurable win and (b) the shipped fallback if Task 8's asm proves infeasible. If you ship Task 6 but not Task 8, this is the end state.

**Files:**
- Modify: `crates/vm/src/vm/names.rs:663` (`try_load_global_rust_probe_for_dsl`)

- [ ] **Step 1: Write the failing equivalence test**

In `crates/vm/src/tests/global_cells.rs`:

```rust
#[test]
fn mode_7_fast_read_returns_same_value_and_tracks_mutation() {
    // Cell-backed global, warmed so the site is mode 7.
    // <build vm; execute `var g = 5; g; g;` to install mode 7>
    // A subsequent read returns 5 via the fast read:
    assert_eq!(/* eval `g` */, Value::from_i32(5));
    // Mutate (no structural change → generation unchanged, cell read live):
    // <execute `g = 6;`>
    assert_eq!(/* eval `g` */, Value::from_i32(6));
    // Structural change bumps generation → fast read must NOT serve a stale cell:
    // <execute `delete globalThis.g; var g = 7;` or redefine>
    assert_eq!(/* eval `g` */, Value::from_i32(7));
}
```

- [ ] **Step 2: Run it — expect it to pass via the OLD path (then make the fast path the one exercised)**

This test passes even on the current probe (the old path is correct), so it is a *guard*, not a red test. To prove the fast path is taken, also add a focused assertion using whatever hit/miss counter the IC exposes (grep `record_named_property_cache_hit` / any global-cell hit counter); if none exists, rely on the profile measurement in Step 5. Run:

Run: `cargo test -p lyng-vm mode_7_fast_read`
Expected: PASS (guards correctness while you refactor the probe).

- [ ] **Step 3: Add the mode-7 fast read at the top of the probe**

In `try_load_global_rust_probe_for_dsl` (`names.rs:663`), BEFORE the current `read_atom_constant` / `find_global_environment_ref` work, add:

```rust
// Mode-7 fast read: the site already resolved to a cell-backed global. Serve
// the hit from the projected metadata, skipping name canonicalization, the
// global-env chain walk, and the IC-state HashMap lookup. Validity is the
// generation mirror (kept coherent with every structural bump, Task 4).
if let Some(slot) = feedback_slot
    && let Some(table) = self.metadata_table(frame.code())
{
    let meta = table.property(slot.get());
    if meta.mode == LLINT_IC_MODE_GLOBAL_CELL_LOAD
        && meta.generation == self.dsl_global_ic_generation()
    {
        let cell = PrimitiveValueCellRef::new(meta.handler_bits as u32);
        if let Some(cell) = cell
            && let Some(record) = agent.heap().view().value_cell(cell)
        {
            let value = record.stored_value();
            self.write_register(frame.registers(), target, value);
            advance_dispatch_frame(frame, instruction_len);
            return true;
        }
    }
}
```

(Use the real `metadata_table` read accessor + `PrimitiveValueCellRef::new`/`from_raw` constructor — read their exact signatures. Keep the rest of the function unchanged as the miss/cold path; on any mismatch fall through to it.) The existing `GlobalCellTarget::Cell` block lower in the function (`names.rs:682`) becomes redundant for warmed sites but stays as a correct fallback; you may leave it.

- [ ] **Step 4: Run the equivalence test + full vm suite**

Run: `cargo test -p lyng-vm`
Expected: PASS. Especially: the mutation and structural-change cases in Step 1's test (live cell read; generation bump forces the slow re-resolve).

- [ ] **Step 5: Measure (win #1)**

Run: `cargo build --release -p lyng-cli` then
`cargo run --release -p lyng-bench -- profile --filter Richards --samples 5 --report /tmp/g6.md --json /tmp/g6.json`
Expected: `LoadGlobal` samples/Mdispatch drops materially below ~121 (the name-read/env-walk/hashmap are gone; the probe call remains). Record the number in the commit message.

- [ ] **Step 6: Commit**

```bash
git add crates/vm/src/vm/names.rs crates/vm/src/tests/global_cells.rs
git commit -m "perf(vm): serve mode-7 global reads via thin fast path (skip name/env/hashmap)

LoadGlobal samples/Mdispatch on Richards: 121 -> <measured>.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 7: Asm reachability — value-cell table base + generation offset bindings

Wire the two reads the asm needs into `LLINT_STATE` / `reg_convention`, following the `object_records`/`object_slots` precedent.

**Files:**
- Modify: `crates/vm/src/dsl/llint_state.rs` (add `value_cells_base` field)
- Modify: `crates/vm/src/dsl/reg_convention.rs` (offset consts: `LLINT_STATE_VALUE_CELLS_BASE`, `VM_GLOBAL_IC_GENERATION_OFFSET`)
- Modify: `crates/vm/src/dsl/entry.rs` (populate `value_cells_base` at trampoline entry)

- [ ] **Step 1: Write the failing layout test**

Extend the `LlIntState` layout test (the explorer noted offsets are pinned in a test near `llint_state.rs`; find it — likely `crates/vm/src/dsl/llint_state.rs` `#[cfg(test)]` or a `tests/` file):

```rust
#[test]
fn value_cells_base_offset_is_pinned() {
    assert_eq!(
        std::mem::offset_of!(LlIntState, value_cells_base),
        LLINT_STATE_VALUE_CELLS_BASE
    );
}

#[test]
fn vm_global_ic_generation_offset_is_pinned() {
    assert_eq!(
        std::mem::offset_of!(Vm, dsl_global_ic_generation),
        VM_GLOBAL_IC_GENERATION_OFFSET
    );
}
```

- [ ] **Step 2: Run — expect failure**

Run: `cargo test -p lyng-vm value_cells_base_offset OR vm_global_ic_generation_offset`
Expected: FAIL (unresolved consts / fields).

- [ ] **Step 3: Add the `LlIntState` field + offset consts**

In `crates/vm/src/dsl/llint_state.rs`, add a field to `LlIntState` mirroring `object_slots_base` (a `*const *const T` table base):

```rust
/// Base of the value-cell pointer table (`PrimitiveValueCellRef` → record ptr),
/// for the asm mode-7 GlobalCellLoad hit. Mirrors `object_slots_base`.
pub value_cells_base: *const *const PrimitiveValueCellRecord,
```

Add the offset consts in `crates/vm/src/dsl/reg_convention.rs` next to `LLINT_STATE_OBJECT_SLOTS_BASE`:

```rust
pub const LLINT_STATE_VALUE_CELLS_BASE: usize = core::mem::offset_of!(crate::dsl::llint_state::LlIntState, value_cells_base);
pub const VM_GLOBAL_IC_GENERATION_OFFSET: usize = core::mem::offset_of!(crate::vm::Vm, dsl_global_ic_generation);
```

(Match the exact form the existing `LLINT_STATE_*` / `VM_*` offset consts use, including the `#[cfg(not(...))]` sentinel-0 fallback pattern if `reg_convention.rs` uses one for non-aarch64 builds.)

- [ ] **Step 4: Populate `value_cells_base` at entry**

In `crates/vm/src/dsl/entry.rs`, where `object_records_base` / `object_slots_base` are set (~line 103), add:

```rust
let value_cells_base = agent.heap().view().value_cell_ptr_table().as_ptr();
```

and store it into the `LlIntState` initializer next to the object bases.

- [ ] **Step 5: Run the layout tests — expect PASS**

Run: `cargo test -p lyng-vm value_cells_base_offset OR vm_global_ic_generation_offset`
Expected: PASS.

- [ ] **Step 6: Build (incl. release) + commit**

Run: `cargo build -p lyng-vm && cargo build --release -p lyng-vm`
Expected: success. (The new field/consts are not read by asm yet.)

```bash
git add crates/vm/src/dsl/llint_state.rs crates/vm/src/dsl/reg_convention.rs crates/vm/src/dsl/entry.rs
git commit -m "feat(vm/dsl): pin value-cell table base + global-IC generation for asm

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 8: Asm mode-7 hit handler (measurable win #2)

Rewrite `op_load_global_dsl` so a mode-7 site is served inline; everything else bails to the unchanged Rust probe/cold path.

**Files:**
- Modify: `crates/vm/src/dsl/backend/aarch64/feedback.rs` (+ `objects.rs` if the cell-deref macro lives there)
- Modify: `crates/vm/src/dsl/handlers/cold.rs:476` (`op_load_global_dsl`)

- [ ] **Step 1: Add the asm DSL macros**

In `crates/vm/src/dsl/backend/aarch64/feedback.rs`, mirror `branch_named_own_inline_mode!` to add a mode-7 check, and add a value-cell load. Bindings available (from Task 7 + existing): `{feedback_mode}` (0), `{feedback_named_handler_bits}` (8), `PROPERTY_METADATA_GENERATION_OFFSET` (4 — bind as `{feedback_generation}`), `{value_cells_base}` (LLINT_STATE_VALUE_CELLS_BASE), `{vm_global_ic_gen}` (VM_GLOBAL_IC_GENERATION_OFFSET), `{cell_stored_value}` (PRIMITIVE_VALUE_CELL_RECORD_STORED_VALUE_OFFSET = 0). The asm holds `x22` = Vm, `x24` = LLINT_STATE.

```rust
// branch if the site is NOT mode 7 (GlobalCellLoad)
macro_rules! branch_global_cell_mode {
    ($entry:tt, $label:tt) => {
        concat!(
            "ldrb   w16, [x", stringify!($entry), ", {feedback_mode}]\n",
            "cmp    w16, #7\n",
            "b.ne   ", stringify!($label), "\n",
        )
    };
}

// generation check: bail if metadata.generation != vm.dsl_global_ic_generation
macro_rules! branch_global_cell_generation_mismatch {
    ($entry:tt, $label:tt) => {
        concat!(
            "ldr    w16, [x", stringify!($entry), ", {feedback_generation}]\n",
            "ldr    w17, [x22, {vm_global_ic_gen}]\n",
            "cmp    w16, w17\n",
            "b.ne   ", stringify!($label), "\n",
        )
    };
}

// load cell value: handler_bits = 1-based cell ref; table[ref] = *const record;
// value at offset {cell_stored_value} (0). Bail if table entry is null.
macro_rules! load_global_cell_value_or_branch {
    ($entry:tt => $dst:tt, $label:tt) => {
        concat!(
            "ldr    w16, [x", stringify!($entry), ", {feedback_named_handler_bits}]\n", // w16 = ref (1-based)
            "ldr    x17, [x24, {value_cells_base}]\n",     // x17 = table base
            "ldr    x16, [x17, x16, lsl #3]\n",            // x16 = table[ref] (*const record)
            "cbz    x16, ", stringify!($label), "\n",      // null → bail
            "ldr    x", stringify!($dst), ", [x16, {cell_stored_value}]\n", // value
        )
    };
}
```

(Match the project's macro module conventions — how `branch_named_own_inline_mode!` is defined/exported and how bindings are surfaced as `naked_asm!` named operands via `reg_convention`. Add `{feedback_generation}`, `{value_cells_base}`, `{vm_global_ic_gen}`, `{cell_stored_value}` to the lowerer's binding list in `crates/vm-dsl/src/lower.rs` next to the existing `feedback_*` / `state_object_*` bindings.)

> Note: `value_cells_base` is a `*const *const Record`; the table is indexed by the **1-based** ref, with entry 0 unused (Task 2) — so `table[ref]` is correct without a `-1` adjustment (matches how `object_record_ptrs` is indexed by `ObjectRef.get()`).

- [ ] **Step 2: Rewrite `op_load_global_dsl`**

In `crates/vm/src/dsl/handlers/cold.rs:476`, replace the body. New shape (mirror the structure of `op_get_named_property_dsl`'s mode dispatch; reuse the existing `load_feedback_site!` and dispatch macros):

```rust
op_load_global_dsl, opcode_byte = 14, layout = Abx, length = 6, |a, bx| {
    load_feedback_site!(bx => c);                       // c = *PropertyMetadata (slot = bx)
    branch_global_cell_mode!(c, .slow);                 // not mode 7 → cold path
    branch_global_cell_generation_mismatch!(c, .slow);  // stale → cold path
    load_global_cell_value_or_branch!(c => t0, .slow);  // t0 = cell.stored_value
    store_reg!(a, t0);
    dispatch!();                                        // pure value load, no Refresh
.slow:
    decode_abx!(a, bx);
    call_slow!(op_load_global_slow_rs, args = [a, bx]);
    dispatch_after_slow!();
}
```

Confirm: (1) the feedback slot operand for `LoadGlobal` is `bx` (read the current handler — the probe is called with `[a, bx]`); `load_feedback_site!` expects the slot id, so pass whatever the current handler passes to the probe as the slot. (2) `store_reg!`/`dispatch!`/`decode_abx!`/`call_slow!` are the same macros the current handler and `op_get_named_property_dsl` use. (3) The `.slow` path uses the existing, unchanged `op_load_global_slow_rs` (full semantic), which calls `load_global_with_feedback` — the path that installs mode-7 metadata (Task 5), so a cold/first-execution site is served by the semantics and installs mode 7 for the next hit.

This handler no longer references `op_load_global_rust_probe_rs` / `call_rust_probe!`. After the rewrite, `op_load_global_rust_probe_rs` and `try_load_global_rust_probe_for_dsl` (the Task-6 fast read) become unused — **remove them** (their fast-read logic now lives in the asm hit; the cold path is `op_load_global_slow_rs` → `load_global_with_feedback`). Verify nothing else references them (`grep`); if something does, leave it and note why. Confirm no dead-code/clippy warning remains.

> **Register usage:** `t0` and the scratch (`x16`/`x17`) must follow the handler's register convention — read how `op_get_named_property_dsl` names its temporaries and mirror it. The macros above use `x16`/`x17` (AAPCS scratch) for intermediates, consistent with the counter macros; verify no live operand (`a`, `bx`/`c`) sits in `x16`/`x17` at these points.

- [ ] **Step 3: Build under the engine — expect it to assemble**

Run: `cargo build --release -p lyng-vm`
Expected: success. If the assembler rejects a macro, fix the instruction syntax (compare against the working `op_get_named_property_dsl` emission). If a binding is unresolved, add it to the lowerer (Step 1 note).

- [ ] **Step 4: Correctness tests (reuse Task 5/6 tests through the asm path)**

The Task-6 tests (`mode_7_fast_read_...`, `cold_path_cell_resolution_projects_mode_7`) and the existing global test262 now exercise the asm hit. Add one asm-specific staleness test (the backstop for the Task-4 invariant):

```rust
#[test]
fn asm_mode_7_hit_does_not_serve_a_freed_or_redefined_global() {
    // Warm a site to mode 7, then perform each structural change and assert the
    // next read is correct (re-resolved), never stale:
    //  (a) delete a configurable global then redeclare with a new value
    //  (b) Object.defineProperty(globalThis,'x',{get(){return 99}}) → accessor
    //  (c) `let x` shadowing an existing `var x`
    // For each: the post-change read returns the NEW semantics, proving the
    // generation bump forced an asm bail + cold re-resolve.
    // <use the global_cells.rs harness; one sub-case per structural change>
}
```

Run: `cargo test -p lyng-vm`
Expected: PASS (full suite, incl. global test262 and the new staleness test).

- [ ] **Step 5: test262 conformance (no regression)**

Run the global-focused test262 the way the repo does (see `tools/lyng-test262` / `crates/tests`): at minimum the global `var`/`let`/`const`/`class`, TDZ, `globalThis` reflection, `delete`, and `Object.defineProperty` suites. 
Run: `cargo test -p lyng-tests` (or the project's test262 entrypoint per `crates/AGENTS.md`)
Expected: no new failures vs the pre-branch baseline.

- [ ] **Step 6: Measure (win #2) + verify slow share**

Run: `cargo run --release -p lyng-bench -- profile --filter Richards --samples 5 --report /tmp/g8.md --json /tmp/g8.json`
Expected: `LoadGlobal` samples/Mdispatch drops toward `GetNamedProperty`'s ~33 or below (now no probe call at all on the hit); slow share stays ~0; LoadGlobal time-share falls from ~15.78%. Also `--filter RayTrace` to confirm no regression there. Record numbers in the commit.

- [ ] **Step 7: Commit**

```bash
git add crates/vm/src/dsl/backend/aarch64/feedback.rs crates/vm-dsl/src/lower.rs crates/vm/src/dsl/handlers/cold.rs crates/vm/src/vm/names.rs crates/vm/src/tests/global_cells.rs
git commit -m "perf(vm/dsl): inline asm LoadGlobal mode-7 cell hit

LoadGlobal samples/Mdispatch on Richards: <task6> -> <measured>; slow share ~0.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Final Verification

- [ ] **Step 1: Full test suites**

Run: `cargo test -p lyng-gc && cargo test -p lyng-vm && cargo test -p lyng-vm --features diagnostic-counters && cargo test -p lyng-bench`
Expected: all PASS.

- [ ] **Step 2: test262 baseline**

Run the project's full/global test262 entrypoint; confirm zero new failures vs `main`.

- [ ] **Step 3: Clippy + fmt**

Run: `cargo clippy --release -p lyng-gc --all-targets && cargo clippy --release -p lyng-vm --all-targets && cargo clippy --release -p lyng-vm --features diagnostic-counters --all-targets && cargo fmt --all && git diff --exit-code`
Expected: no warnings, no fmt diff.

- [ ] **Step 4: Performance acceptance**

Run: `cargo run --release -p lyng-bench -- profile --samples 5 --report /tmp/gfull.md --json /tmp/gfull.json` (whole suite) and the isolated `v8suite`:
`cargo run --release -p lyng-bench -- v8suite --samples 5`
Expected: Richards `LoadGlobal` samples/Mdispatch ≪ 121 (toward ≤33); no `v8suite` score regression on any benchmark; RayTrace unaffected. Capture the before/after in a short note (optionally refresh `reports/lyng/`).
