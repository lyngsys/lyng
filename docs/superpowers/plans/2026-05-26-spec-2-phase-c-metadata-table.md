# Spec 2 Phase C — MetadataTable per code object

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the JSC-style per-code-object `MetadataTable`: one variable-width buffer per `CodeRef`, with `[header][kind_offset_table][slot→in_kind_index table][per-kind runs]` layout. By the end of Phase C, the asm DSL fast path resolves IC slots through this buffer; `feedback_flat_storage` remains alive but unread (deleted in Phase D).

**Architecture:** Triple-write during Phase C. `FeedbackVector` stays the semantic source. `feedback_flat_storage` is the asm fast-path source through C.3. `MetadataTable` is the new parallel storage filled by C.1–C.3, then becomes the asm fast-path source in C.4. Every mutation that today writes to `feedback_flat_storage` via `mirror_flat_slot` also writes to `MetadataTable` via `mirror_metadata_slot`. A debug-only equivalence assertion in C.3 ensures byte-level agreement before the C.4 flip.

**Tech Stack:** Rust, `Box<[u8]>` for the metadata buffer, `offset_of!` for asm-visible field offsets, criterion for regression checks.

**Spec:** `docs/superpowers/specs/2026-05-26-spec-2-ic-jsc-migration-design.md` §5.

**Memory:** `~/.claude/projects/-Users-sondre-dev-lyng/memory/project_feedback_jsc_migration.md`.

---

## Context — post-Phase-B state

Recon from 2026-05-26 (after Phase B landed):

| Concept | Location | Today |
|---|---|---|
| Semantic IC storage | `Vm::feedback_vectors: Vec<FeedbackVector>` in `crates/vm/src/vm.rs:166` | Source of truth. Keyed by `code_index(code) = code.raw().get() - 1`. |
| Asm fast-path storage | `Vm::feedback_flat_storage: Vec<Box<[FeedbackEntry]>>` in `crates/vm/src/vm.rs:180` | 64B stride mirror. Same key. Eagerly allocated at install. |
| IC slot operand | `FeedbackSlotId(NonZeroU32)` at `crates/types/src/ids.rs:98` | 1-based; global sequential across all opcode kinds. |
| Per-slot kind | `BytecodeFunction::feedback_sites() -> &[FeedbackSiteDescriptor]` at `crates/bytecode/src/function/template.rs:369` | Each descriptor carries `slot`, `kind: FeedbackSiteKind`, `metadata`. |
| Kind enum | `FeedbackSiteKind` at `crates/bytecode/src/metadata.rs:475` | 7 variants: `Arithmetic`, `Comparison`, `NamedPropertyLoad`, `NamedPropertyStore`, `KeyedPropertyAccess`, `Call`, `Construct`. |
| Asm pin | `x21 = *mut FeedbackEntry` base, loaded from `LlIntState::frame_fv_base` at `crates/vm/src/dsl/llint_state.rs:36` | Reset on call entry; slow path can rewrite (`crates/vm/src/dsl/slow_path.rs:235`). |
| Resolve macro | `crates/vm/src/dsl/backend/aarch64/feedback.rs:24` | `sub x17, x{slot}, #1; add x{dst}, x21, x17, lsl #6`. |
| Mirror function | `Vm::mirror_flat_slot(code, slot)` at `crates/vm/src/vm/feedback.rs:2141` | Projects `FeedbackVector` site → 64B flat entry. |
| Mirror call sites | ~11 sites in `crates/vm/src/vm/feedback.rs` + 1 in `crates/vm/src/vm.rs` | Lines 1586, 2135, 2310, 2328, 2441, 2465, 2513, 2593, 2912, 3189, 3216. |
| Install hook | `Vm::install_function_for_dsl(installed, code)` at `crates/vm/src/vm/install.rs:830` | Resizes `feedback_vectors` / `feedback_flat_storage` to `code_index + 1` and allocates the flat box. |

### Recon corrections to the design doc

| Design claim (§5) | Reality |
|---|---|
| Five per-kind metadata structs (`Property`, `Call`, `Arith`, `Comparison`, `KeyedProperty`) | Correct, but `FeedbackSiteKind` has **7** variants. Map `NamedPropertyLoad` + `NamedPropertyStore` → `Property` run; `Call` + `Construct` → `Call` run. |
| `x21` resolve uses per-kind `instance_index` | Bytecode operands today carry **global** `FeedbackSlotId`, not per-kind indices. Phase C maps global slot → per-kind in-kind index at **install time**, stored in a `slot_to_in_kind_index: Box<[u32]>` table that lives **inside the MetadataTable buffer** so asm can resolve via two loads (offset table + slot mapping). |
| `Vm::metadata_tables: Vec<Option<Box<[u8]>>>` | Recommend `Vec<Option<MetadataTable>>` instead: the wrapper holds cached `kind_offsets: [u32; METADATA_KIND_COUNT]` and `per_kind_counts` for cheap slow-path access. Asm dereferences only the `buffer` pointer. |
| `feedback_flat_storage` deletion in Phase C | No. C.4 only flips the asm pin to read from `MetadataTable.buffer`. `feedback_flat_storage` allocation + mirroring stays alive (dead code) until Phase D's cleanup. |
| Test inspection of MetadataTable contents | Add `Vm::metadata_table_inspect(code, slot)` (test-only) that returns a typed view of the per-kind run entry for a given global slot. Mirrors the `feedback_vector_*` test accessors. |

---

## File map

| File | Phase | New / existing | Responsibility |
|---|---|---|---|
| `crates/vm/src/vm/metadata_table.rs` | C.1 | NEW | `MetadataTable` struct, layout constants, allocator, kind-offset / slot-index lookup helpers. |
| `crates/vm/src/vm/metadata_table/kind.rs` | C.1 | NEW | `MetadataKind` enum (5 variants) + mapping `FeedbackSiteKind → MetadataKind`. Stride/size constants per kind. |
| `crates/vm/src/vm/metadata_table/property.rs` | C.2 | NEW | `PropertyMetadata` repr-C struct. Mirrors the `Named*` fields from `FeedbackEntry`. |
| `crates/vm/src/vm/metadata_table/call.rs` | C.2 | NEW | `CallMetadata`. Mirrors call IC bits. |
| `crates/vm/src/vm/metadata_table/arith.rs` | C.2 | NEW | `ArithMetadata`. Mirrors `scalar_observed_bits` + `scalar_execution_count`. |
| `crates/vm/src/vm/metadata_table/comparison.rs` | C.2 | NEW | `ComparisonMetadata`. Same shape as `ArithMetadata`. |
| `crates/vm/src/vm/metadata_table/keyed_property.rs` | C.2 | NEW | `KeyedPropertyMetadata`. Mirrors keyed-property IC bits. |
| `crates/vm/src/vm.rs` | C.1+C.2+C.4 | existing | Add `metadata_tables: Vec<Option<MetadataTable>>`. Add `Vm::metadata_table(code)`, `metadata_table_mut(code)`. Add `mirror_metadata_slot(code, slot)` parallel to `mirror_flat_slot`. C.4: swap `LlIntState::frame_fv_base` source. |
| `crates/vm/src/vm/install.rs` | C.1 | existing | At install: count opcodes by kind from `feedback_sites()`, allocate `MetadataTable`, fill `kind_offsets` + `slot_to_in_kind_index`. |
| `crates/vm/src/vm/feedback.rs` | C.2 | existing | Add `mirror_metadata_slot` call after every `mirror_flat_slot` site. Update the `mirror_flat_slot` body's docstring to mention the parallel call. |
| `crates/vm/src/vm/feedback.rs` | C.3 | existing | Add `debug_assert_metadata_matches_flat(code, slot)` invoked inside `mirror_metadata_slot`. |
| `crates/vm/src/dsl/llint_state.rs` | C.4 | existing | Rename `frame_fv_base` field to `frame_metadata_table_base` (or keep name and change source). |
| `crates/vm/src/dsl/entry.rs` | C.4 | existing | Trampoline entry: load `metadata_tables[code_index].buffer` instead of `feedback_flat_storage[code_index]`. |
| `crates/vm/src/dsl/slow_path.rs` | C.4 | existing | Slow-path pin refresh (line ~235): rewrite to load `metadata_tables` buffer. |
| `crates/vm/src/dsl/backend/aarch64/feedback.rs` | C.4 | existing | Replace `load_feedback_site!` with `load_metadata_slot!`: 5-instruction resolve through offset table + slot-index table. |
| `crates/vm/src/dsl/feedback_flat.rs` | C.4 | existing | `FeedbackEntry` stays alive (dual-storage). New constants for MetadataTable header offsets exported here OR moved to `metadata_table.rs`. Recommend: new constants live in `metadata_table.rs`; `feedback_flat.rs` is unchanged. |
| `crates/vm/src/tests/feedback.rs` + `crates/vm/src/tests/inline_caches.rs` | all | existing | Add C1–C8 tests. |
| `crates/vm/benches/property_addition.rs` | C.4 | existing | Baseline already captured pre-Phase-C; verify ≤3% delta. |

---

## Layout

```
MetadataTable.buffer byte layout (target of x21 in Phase C.4):

  +------------------------------------+ <- x21 = buffer_ptr
  | LinkingDataHeader                  |     (16B: buffer_size u32, slot_count u32, kind_offsets_ptr_offset u32, slot_index_ptr_offset u32)
  +------------------------------------+
  | kind_offsets: [u32; KIND_COUNT]    |     (5 * 4 = 20B; one offset per MetadataKind)
  +------------------------------------+
  | slot_to_in_kind_index: [u32; N]    |     (N = feedback_slot_count; sized at install)
  +------------------------------------+ <- kind_offsets[Property]
  | PropertyMetadata[count_Property]   |
  +------------------------------------+ <- kind_offsets[Call]
  | CallMetadata[count_Call]           |
  +------------------------------------+ <- kind_offsets[Arith]
  | ArithMetadata[count_Arith]         |
  +------------------------------------+ <- kind_offsets[Comparison]
  | ComparisonMetadata[count_Comp]     |
  +------------------------------------+ <- kind_offsets[KeyedProperty]
  | KeyedPropertyMetadata[count_KP]    |
  +------------------------------------+

All runs aligned to 8 bytes. Buffer is 8-aligned overall (Box<[u8]> via aligned alloc helper).
```

Asm resolve (per IC opcode, KIND known at emit time):

```asm
;; inputs: x{slot} = 1-based FeedbackSlotId (from bytecode operand)
;; constants per opcode: KIND_OFFSET_OFFSET = offset of kind_offsets[KIND] in buffer
;;                       SLOT_INDEX_TABLE_OFFSET = offset of slot_to_in_kind_index table
;;                       KIND_STRIDE = sizeof(KindMetadata)
sub  x17, x{slot}, #1                                          ;; x17 = slot - 1 (0-based)
ldr  w{idx}, [x21, x17, lsl #2, +SLOT_INDEX_TABLE_OFFSET]      ;; idx = slot_to_in_kind_index[slot-1]
ldr  w{koff}, [x21, #KIND_OFFSET_OFFSET]                        ;; koff = kind_offsets[KIND]
add  x{base}, x21, x{koff}                                      ;; base = buffer + koff
add  x{dst},  x{base}, x{idx}, lsl #KIND_STRIDE_SHIFT          ;; dst = base + idx * stride (power-of-2 kinds)
;;                                                                OR madd for non-power-of-2 strides
```

5 instructions for power-of-2 stride kinds (Arith/Comparison: 8B); 5 instructions for power-of-2 32B Property; MADD variant adds one for Call/KeyedProperty (24B).

---

## PR C.1 — `MetadataTable` allocation + LinkingData + offset table

### Task 1.1: `MetadataKind` enum + `FeedbackSiteKind → MetadataKind` mapping

**Files:**
- Create: `crates/vm/src/vm/metadata_table/kind.rs`
- Create: `crates/vm/src/vm/metadata_table.rs` (module declaration + re-exports only at this stage)
- Modify: `crates/vm/src/vm.rs` (`pub(crate) mod metadata_table;` near other submodule declarations)

- [ ] **Step 1: Module wiring**

In `crates/vm/src/vm.rs`, near other `mod` declarations for `vm/*` submodules (search for `mod feedback;`), add:
```rust
pub(crate) mod metadata_table;
```

Create `crates/vm/src/vm/metadata_table.rs` with:
```rust
//! JSC-style per-code-object `MetadataTable`. Spec 2 Phase C.
//!
//! Layout: header + per-kind offset table + slot→in-kind-index table + per-kind runs.
//! Phase C.1 lands the type + allocator; reads and writes wire up in C.2/C.4.

pub mod kind;

pub use kind::{MetadataKind, METADATA_KIND_COUNT};
```

- [ ] **Step 2: Define `MetadataKind` + mapping in `metadata_table/kind.rs`**

```rust
use lyng_bytecode::metadata::FeedbackSiteKind;

/// IC metadata kinds in the table layout. Each kind owns its own per-kind run
/// in the buffer. Two `FeedbackSiteKind`s may map to the same `MetadataKind`
/// (e.g. `NamedPropertyLoad` + `NamedPropertyStore` → `Property`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum MetadataKind {
    Property = 0,
    Call = 1,
    Arith = 2,
    Comparison = 3,
    KeyedProperty = 4,
}

pub const METADATA_KIND_COUNT: usize = 5;

impl MetadataKind {
    pub const fn from_site_kind(kind: FeedbackSiteKind) -> Self {
        match kind {
            FeedbackSiteKind::NamedPropertyLoad
            | FeedbackSiteKind::NamedPropertyStore => Self::Property,
            FeedbackSiteKind::Call | FeedbackSiteKind::Construct => Self::Call,
            FeedbackSiteKind::Arithmetic => Self::Arith,
            FeedbackSiteKind::Comparison => Self::Comparison,
            FeedbackSiteKind::KeyedPropertyAccess => Self::KeyedProperty,
        }
    }

    pub const fn index(self) -> usize {
        self as usize
    }

    /// Byte size of one metadata entry for this kind. Set as a placeholder for
    /// Phase C.1; Phase C.2 makes these match the actual per-kind struct sizes.
    pub const fn stride_bytes(self) -> usize {
        match self {
            Self::Property => 32,
            Self::Call => 24,
            Self::Arith => 8,
            Self::Comparison => 8,
            Self::KeyedProperty => 24,
        }
    }
}
```

- [ ] **Step 3: Add unit tests in `kind.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn site_kind_maps_to_metadata_kind() {
        assert_eq!(MetadataKind::from_site_kind(FeedbackSiteKind::NamedPropertyLoad), MetadataKind::Property);
        assert_eq!(MetadataKind::from_site_kind(FeedbackSiteKind::NamedPropertyStore), MetadataKind::Property);
        assert_eq!(MetadataKind::from_site_kind(FeedbackSiteKind::Call), MetadataKind::Call);
        assert_eq!(MetadataKind::from_site_kind(FeedbackSiteKind::Construct), MetadataKind::Call);
        assert_eq!(MetadataKind::from_site_kind(FeedbackSiteKind::Arithmetic), MetadataKind::Arith);
        assert_eq!(MetadataKind::from_site_kind(FeedbackSiteKind::Comparison), MetadataKind::Comparison);
        assert_eq!(MetadataKind::from_site_kind(FeedbackSiteKind::KeyedPropertyAccess), MetadataKind::KeyedProperty);
    }

    #[test]
    fn metadata_kind_count_matches_variants() {
        // Sanity: METADATA_KIND_COUNT must equal the maximum index + 1.
        assert_eq!(MetadataKind::KeyedProperty.index() + 1, METADATA_KIND_COUNT);
    }
}
```

- [ ] **Step 4: Run** `cargo test -p lyng-vm metadata_table::kind` — expect 2 tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm.rs crates/vm/src/vm/metadata_table.rs crates/vm/src/vm/metadata_table/kind.rs
git commit -m "vm/metadata_table: introduce MetadataKind enum + FeedbackSiteKind mapping"
```

---

### Task 1.2: `MetadataTable` struct + allocator

**Files:**
- Modify: `crates/vm/src/vm/metadata_table.rs`

- [ ] **Step 1: Write the failing tests first**

Append to `metadata_table.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use lyng_bytecode::metadata::FeedbackSiteKind;

    fn site(slot_one_based: u32, kind: FeedbackSiteKind) -> SiteDescriptor {
        SiteDescriptor { slot: slot_one_based, kind }
    }

    #[test]
    fn empty_table_has_only_header_and_offset_block() {
        let table = MetadataTable::allocate(&[]);
        // Header + kind_offsets[5] = 16 + 20 = 36, rounded up to 40 for 8-alignment.
        assert!(table.buffer().len() >= 36);
        // No slots: slot_to_in_kind_index table is zero-length.
        assert_eq!(table.slot_count(), 0);
        // Every kind run is empty.
        for kind_idx in 0..METADATA_KIND_COUNT {
            assert_eq!(table.run_len_for_kind_index(kind_idx), 0);
        }
    }

    #[test]
    fn table_assigns_per_kind_indices_in_slot_order() {
        // 3 property loads, 1 call, 2 arith ops, in declared order.
        let sites = vec![
            site(1, FeedbackSiteKind::NamedPropertyLoad),
            site(2, FeedbackSiteKind::Arithmetic),
            site(3, FeedbackSiteKind::NamedPropertyLoad),
            site(4, FeedbackSiteKind::Call),
            site(5, FeedbackSiteKind::Arithmetic),
            site(6, FeedbackSiteKind::NamedPropertyLoad),
        ];
        let table = MetadataTable::allocate(&sites);
        // Property kind: slots 1,3,6 → in-kind indices 0,1,2
        assert_eq!(table.in_kind_index_for_slot(1), 0);
        assert_eq!(table.in_kind_index_for_slot(3), 1);
        assert_eq!(table.in_kind_index_for_slot(6), 2);
        // Arith kind: slots 2,5 → 0,1
        assert_eq!(table.in_kind_index_for_slot(2), 0);
        assert_eq!(table.in_kind_index_for_slot(5), 1);
        // Call kind: slot 4 → 0
        assert_eq!(table.in_kind_index_for_slot(4), 0);
    }

    #[test]
    fn table_run_lengths_match_per_kind_counts() {
        let sites = vec![
            site(1, FeedbackSiteKind::NamedPropertyLoad),
            site(2, FeedbackSiteKind::NamedPropertyStore),
            site(3, FeedbackSiteKind::Call),
        ];
        let table = MetadataTable::allocate(&sites);
        assert_eq!(table.run_len_for_kind(MetadataKind::Property), 2);
        assert_eq!(table.run_len_for_kind(MetadataKind::Call), 1);
        assert_eq!(table.run_len_for_kind(MetadataKind::Arith), 0);
    }

    #[test]
    fn table_kind_offsets_land_inside_buffer() {
        let sites = vec![site(1, FeedbackSiteKind::NamedPropertyLoad)];
        let table = MetadataTable::allocate(&sites);
        for kind_idx in 0..METADATA_KIND_COUNT {
            let off = table.kind_offset_by_index(kind_idx);
            assert!(off as usize <= table.buffer().len());
        }
    }
}
```

- [ ] **Step 2: Run, expect FAIL** (`MetadataTable`, `SiteDescriptor` etc. not defined).

- [ ] **Step 3: Implement**

Append to `metadata_table.rs` (above the `#[cfg(test)] mod tests`):
```rust
use std::alloc::{alloc_zeroed, Layout};
use std::ptr::NonNull;

/// Compact descriptor used by the allocator. Mirrors the shape of
/// `lyng_bytecode::metadata::FeedbackSiteDescriptor` but elides the metadata
/// payload (the allocator only needs the slot and kind). Tests construct this
/// inline; production calls `MetadataTable::from_bytecode_function`.
#[derive(Clone, Copy, Debug)]
pub struct SiteDescriptor {
    /// 1-based `FeedbackSlotId`.
    pub slot: u32,
    pub kind: lyng_bytecode::metadata::FeedbackSiteKind,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct LinkingDataHeader {
    buffer_size: u32,
    slot_count: u32,
    slot_index_table_offset: u32,  // byte offset to slot_to_in_kind_index[]
    _reserved: u32,
}

const HEADER_SIZE: usize = std::mem::size_of::<LinkingDataHeader>();
const KIND_OFFSETS_OFFSET: usize = HEADER_SIZE;
const KIND_OFFSETS_SIZE: usize = METADATA_KIND_COUNT * 4;

/// Per-code-object IC metadata buffer.
///
/// The buffer is owned by the table; consumers hold a `*mut u8` for asm
/// dispatch but must never escape the lifetime of the owning `MetadataTable`.
pub struct MetadataTable {
    buffer: Box<[u8]>,
    /// Cached per-kind base offsets (also stored inside `buffer`). Lets
    /// slow-path queries avoid a buffer read.
    kind_offsets: [u32; METADATA_KIND_COUNT],
    /// Per-kind run counts (for `run_len_for_kind`).
    per_kind_counts: [u32; METADATA_KIND_COUNT],
    /// Total slots covered.
    slot_count: u32,
}

impl MetadataTable {
    /// Allocate a fresh table sized to hold the per-kind metadata for the
    /// given feedback sites. Sites need not be sorted; `slot_to_in_kind_index`
    /// is keyed by `slot - 1`.
    pub fn allocate(sites: &[SiteDescriptor]) -> Self {
        let slot_count = sites.len() as u32;

        // 1. Tally per-kind counts.
        let mut per_kind_counts = [0u32; METADATA_KIND_COUNT];
        for site in sites {
            let mk = MetadataKind::from_site_kind(site.kind);
            per_kind_counts[mk.index()] += 1;
        }

        // 2. Compute buffer layout.
        let slot_index_table_offset = KIND_OFFSETS_OFFSET + KIND_OFFSETS_SIZE;
        let slot_index_table_size = (slot_count as usize) * 4;
        let runs_start = align_up(slot_index_table_offset + slot_index_table_size, 8);

        let mut kind_offsets = [0u32; METADATA_KIND_COUNT];
        let mut cursor = runs_start;
        for kind_idx in 0..METADATA_KIND_COUNT {
            kind_offsets[kind_idx] = cursor as u32;
            let stride = stride_for_kind_index(kind_idx);
            cursor += stride * (per_kind_counts[kind_idx] as usize);
            cursor = align_up(cursor, 8);
        }
        let buffer_size = cursor;

        // 3. Allocate zeroed buffer.
        let mut buffer = vec![0u8; buffer_size].into_boxed_slice();

        // 4. Write header.
        let header = LinkingDataHeader {
            buffer_size: buffer_size as u32,
            slot_count,
            slot_index_table_offset: slot_index_table_offset as u32,
            _reserved: 0,
        };
        // SAFETY: buffer has at least HEADER_SIZE bytes (≥ 16).
        unsafe {
            std::ptr::write(buffer.as_mut_ptr() as *mut LinkingDataHeader, header);
        }

        // 5. Write kind offsets.
        for kind_idx in 0..METADATA_KIND_COUNT {
            let off = KIND_OFFSETS_OFFSET + kind_idx * 4;
            buffer[off..off + 4].copy_from_slice(&kind_offsets[kind_idx].to_ne_bytes());
        }

        // 6. Assign in-kind indices in slot-ascending order. Sort sites by
        //    slot first so the per-kind index is stable & deterministic.
        let mut sorted: Vec<SiteDescriptor> = sites.to_vec();
        sorted.sort_by_key(|s| s.slot);
        let mut next_in_kind = [0u32; METADATA_KIND_COUNT];
        for site in &sorted {
            let mk = MetadataKind::from_site_kind(site.kind);
            let in_kind_idx = next_in_kind[mk.index()];
            next_in_kind[mk.index()] += 1;
            let slot_zero_based = (site.slot - 1) as usize;
            let off = slot_index_table_offset + slot_zero_based * 4;
            buffer[off..off + 4].copy_from_slice(&in_kind_idx.to_ne_bytes());
        }

        Self { buffer, kind_offsets, per_kind_counts, slot_count }
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    pub fn buffer_ptr(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    pub fn buffer_ptr_mut(&mut self) -> *mut u8 {
        self.buffer.as_mut_ptr()
    }

    pub fn slot_count(&self) -> u32 {
        self.slot_count
    }

    pub fn kind_offset(&self, kind: MetadataKind) -> u32 {
        self.kind_offsets[kind.index()]
    }

    pub fn kind_offset_by_index(&self, kind_index: usize) -> u32 {
        self.kind_offsets[kind_index]
    }

    pub fn run_len_for_kind(&self, kind: MetadataKind) -> u32 {
        self.per_kind_counts[kind.index()]
    }

    pub fn run_len_for_kind_index(&self, kind_index: usize) -> u32 {
        self.per_kind_counts[kind_index]
    }

    /// Returns the per-kind in-kind index for `slot_one_based`. Panics if
    /// the slot is out of range; callers (asm/slow-path) must validate first.
    pub fn in_kind_index_for_slot(&self, slot_one_based: u32) -> u32 {
        debug_assert!(slot_one_based >= 1 && slot_one_based <= self.slot_count);
        let zero_based = (slot_one_based - 1) as usize;
        let table_offset = KIND_OFFSETS_OFFSET + KIND_OFFSETS_SIZE;
        let byte_off = table_offset + zero_based * 4;
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.buffer[byte_off..byte_off + 4]);
        u32::from_ne_bytes(bytes)
    }
}

fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

fn stride_for_kind_index(kind_index: usize) -> usize {
    match kind_index {
        0 => MetadataKind::Property.stride_bytes(),
        1 => MetadataKind::Call.stride_bytes(),
        2 => MetadataKind::Arith.stride_bytes(),
        3 => MetadataKind::Comparison.stride_bytes(),
        4 => MetadataKind::KeyedProperty.stride_bytes(),
        _ => unreachable!("kind index out of range"),
    }
}
```

- [ ] **Step 4: Run tests, expect PASS.**

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm/metadata_table.rs
git commit -m "vm/metadata_table: add MetadataTable allocator (header + offset + slot index tables)"
```

---

### Task 1.3: Hook allocation into `install_function_for_dsl`

**Files:**
- Modify: `crates/vm/src/vm.rs` (add `metadata_tables` field + accessors)
- Modify: `crates/vm/src/vm/install.rs` (allocate at install)

- [ ] **Step 1: Write failing test in `tests/feedback.rs`**

```rust
#[test]
fn metadata_table_allocated_at_install_with_correct_per_kind_counts() {
    // A small function with 2 property loads, 1 call, 1 arith op.
    let src = r"
        function f(o) {
            return o.x + o.y + f();
        }
    ";
    let unit = compile_test_unit(101, src);
    let mut agent = build_agent();
    let installed = vm_install_script(&mut agent, &unit);
    let code = installed.code();

    let table = agent.vm().metadata_table(code).expect("table allocated");
    // Two named loads (o.x, o.y), one call (f()), one add — counts:
    assert_eq!(table.run_len_for_kind(MetadataKind::Property), 2);
    assert_eq!(table.run_len_for_kind(MetadataKind::Call), 1);
    assert_eq!(table.run_len_for_kind(MetadataKind::Arith), 1);
    assert_eq!(table.run_len_for_kind(MetadataKind::Comparison), 0);
    assert_eq!(table.run_len_for_kind(MetadataKind::KeyedProperty), 0);
}
```

(The test helpers `compile_test_unit`, `build_agent`, `vm_install_script` already exist in `crates/vm/src/tests/feedback.rs` and `tests/common.rs` — reuse the canonical pattern at lines 3–52 of `feedback.rs`.)

- [ ] **Step 2: Run, expect FAIL** (`Vm::metadata_table` doesn't exist).

- [ ] **Step 3: Add field + accessors to `Vm`**

In `crates/vm/src/vm.rs`, alongside `feedback_flat_storage: Vec<Box<[FeedbackEntry]>>` (line 180), add:
```rust
pub(crate) metadata_tables: Vec<Option<MetadataTable>>,
```

Update the `Vm::new` constructor to initialize `metadata_tables: Vec::new()`. Find the existing constructor by searching for `feedback_flat_storage: Vec::new()` and add the new field alongside.

Add accessors near `feedback_vector`/`feedback_vector_mut` (search for `pub fn feedback_vector`):
```rust
pub fn metadata_table(&self, code: CodeRef) -> Option<&MetadataTable> {
    let idx = code_index(code);
    self.metadata_tables.get(idx).and_then(|t| t.as_ref())
}

pub(crate) fn metadata_table_mut(&mut self, code: CodeRef) -> Option<&mut MetadataTable> {
    let idx = code_index(code);
    self.metadata_tables.get_mut(idx).and_then(|t| t.as_mut())
}
```

Add `use crate::vm::metadata_table::MetadataTable;` at the top of `vm.rs`.

- [ ] **Step 4: Hook allocation into `install_function_for_dsl`**

In `crates/vm/src/vm/install.rs` around line 843 (where `feedback_flat_storage` is resized), add:

```rust
// Phase C: allocate parallel MetadataTable buffer keyed by code_index.
if self.metadata_tables.len() <= index {
    self.metadata_tables.resize_with(index + 1, || None);
}
let descriptors: Vec<SiteDescriptor> = installed
    .function
    .feedback_sites()
    .iter()
    .map(|d| SiteDescriptor { slot: d.slot().get(), kind: d.kind() })
    .collect();
self.metadata_tables[index] = Some(MetadataTable::allocate(&descriptors));
```

Add `use crate::vm::metadata_table::{MetadataTable, SiteDescriptor};` at the top of `install.rs`.

- [ ] **Step 5: Run the new test, expect PASS.** Run `cargo test -p lyng-vm` — expect green.

- [ ] **Step 6: Commit**

```bash
git add crates/vm/src/vm.rs crates/vm/src/vm/install.rs crates/vm/src/tests/feedback.rs
git commit -m "vm: allocate MetadataTable at install + add per-code accessors"
```

---

### Task 1.4: Test C1 + C2 — table allocation invariants

**Files:**
- Modify: `crates/vm/src/tests/feedback.rs`

- [ ] **Step 1: Add C1 test (offset-table correctness)**

```rust
#[test]
fn metadata_table_kind_offsets_partition_buffer() {
    let src = r"
        function f(o) {
            return o.a + o.b + o.c;  // 3 property loads, 2 arith
        }
    ";
    let unit = compile_test_unit(102, src);
    let mut agent = build_agent();
    let installed = vm_install_script(&mut agent, &unit);
    let table = agent.vm().metadata_table(installed.code()).expect("table");

    // kind_offsets[Property] should be inside the buffer and aligned.
    let property_off = table.kind_offset(MetadataKind::Property) as usize;
    assert!(property_off % 8 == 0);
    assert!(property_off < table.buffer().len());

    // Each kind's run end ≤ next kind's offset (or buffer end for the last).
    let mut prev_end = property_off;
    for kind in [MetadataKind::Property, MetadataKind::Call, MetadataKind::Arith,
                 MetadataKind::Comparison, MetadataKind::KeyedProperty] {
        let off = table.kind_offset(kind) as usize;
        assert!(off >= prev_end, "{:?} offset overlaps previous kind", kind);
        prev_end = off + (table.run_len_for_kind(kind) as usize) * kind.stride_bytes();
    }
    assert!(prev_end <= table.buffer().len());
}
```

- [ ] **Step 2: Add C2 test (in-kind-index lookup)**

```rust
#[test]
fn metadata_table_in_kind_indices_are_monotone_per_kind() {
    let src = r"
        function f(o) {
            return o.x + o.y;  // slots: load x, load y, add
        }
    ";
    let unit = compile_test_unit(103, src);
    let mut agent = build_agent();
    let installed = vm_install_script(&mut agent, &unit);
    let table = agent.vm().metadata_table(installed.code()).expect("table");

    // Walk all slots in order, group by kind, assert in-kind indices are 0,1,2...
    let entry_fn = unit.function(unit.entry()).unwrap();
    let mut seen_per_kind: std::collections::HashMap<MetadataKind, u32> =
        std::collections::HashMap::new();
    for descriptor in entry_fn.feedback_sites() {
        let mk = MetadataKind::from_site_kind(descriptor.kind());
        let expected = *seen_per_kind.entry(mk).or_insert(0);
        assert_eq!(table.in_kind_index_for_slot(descriptor.slot().get()), expected,
                   "slot {:?} kind {:?}", descriptor.slot(), descriptor.kind());
        *seen_per_kind.get_mut(&mk).unwrap() += 1;
    }
}
```

- [ ] **Step 3: Run, expect PASS.**

- [ ] **Step 4: Commit**

```bash
git add crates/vm/src/tests/feedback.rs
git commit -m "vm/tests/feedback: C1+C2 MetadataTable layout invariants"
```

---

### Task 1.5: PR C.1 boundary check

- [ ] **Step 1: Run** `cargo test --workspace` — expect green.
- [ ] **Step 2: Run** `cargo clippy --workspace --all-targets -- -D warnings` — fix any.
- [ ] **Step 3: Run** `cargo fmt --check` — fix any.
- [ ] **Step 4: Commit** any cleanup. Skip if clean.

PR C.1 boundary: `MetadataTable` exists, is allocated per code object, but nothing reads or writes per-kind data. Pure addition; no production behavior change.

---

## PR C.2 — Per-kind metadata structs + dual-write integration

### Task 2.1: Define the 5 per-kind metadata structs

**Files:**
- Create: `crates/vm/src/vm/metadata_table/property.rs`
- Create: `crates/vm/src/vm/metadata_table/call.rs`
- Create: `crates/vm/src/vm/metadata_table/arith.rs`
- Create: `crates/vm/src/vm/metadata_table/comparison.rs`
- Create: `crates/vm/src/vm/metadata_table/keyed_property.rs`
- Modify: `crates/vm/src/vm/metadata_table.rs` (add `pub mod property;` etc.)

- [ ] **Step 1: Write `property.rs` — mirrors `FeedbackEntry`'s named fields**

```rust
//! `PropertyMetadata` mirrors the per-slot named-property IC state from
//! `FeedbackEntry`. 32-byte stride. Phase C.2 dual-write only — Phase D
//! makes this the system of record.

use std::mem::offset_of;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PropertyMetadata {
    pub mode: u8,
    pub _pad: [u8; 3],
    pub generation: u32,
    pub handler_bits: u64,
    pub aux_bits: u64,
    pub execution_count: u32,
    pub _tail_pad: u32,
}

pub const PROPERTY_METADATA_STRIDE: usize = std::mem::size_of::<PropertyMetadata>();

pub const PROPERTY_METADATA_MODE_OFFSET: usize = offset_of!(PropertyMetadata, mode);
pub const PROPERTY_METADATA_GENERATION_OFFSET: usize = offset_of!(PropertyMetadata, generation);
pub const PROPERTY_METADATA_HANDLER_BITS_OFFSET: usize = offset_of!(PropertyMetadata, handler_bits);
pub const PROPERTY_METADATA_AUX_BITS_OFFSET: usize = offset_of!(PropertyMetadata, aux_bits);
pub const PROPERTY_METADATA_EXEC_COUNT_OFFSET: usize = offset_of!(PropertyMetadata, execution_count);

const _: () = assert!(PROPERTY_METADATA_STRIDE == 32);
```

- [ ] **Step 2: Write `call.rs`**

```rust
use std::mem::offset_of;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CallMetadata {
    pub mode: u8,
    pub _pad: [u8; 3],
    pub generation: u32,
    pub callee_bits: u64,
    pub execution_count: u32,
    pub _tail: u32,
}

pub const CALL_METADATA_STRIDE: usize = std::mem::size_of::<CallMetadata>();
pub const CALL_METADATA_MODE_OFFSET: usize = offset_of!(CallMetadata, mode);
pub const CALL_METADATA_GENERATION_OFFSET: usize = offset_of!(CallMetadata, generation);
pub const CALL_METADATA_CALLEE_BITS_OFFSET: usize = offset_of!(CallMetadata, callee_bits);
pub const CALL_METADATA_EXEC_COUNT_OFFSET: usize = offset_of!(CallMetadata, execution_count);

const _: () = assert!(CALL_METADATA_STRIDE == 24);
```

- [ ] **Step 3: Write `arith.rs`**

```rust
use std::mem::offset_of;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ArithMetadata {
    pub observed_bits: u32,
    pub execution_count: u32,
}

pub const ARITH_METADATA_STRIDE: usize = std::mem::size_of::<ArithMetadata>();
pub const ARITH_METADATA_OBSERVED_BITS_OFFSET: usize = offset_of!(ArithMetadata, observed_bits);
pub const ARITH_METADATA_EXEC_COUNT_OFFSET: usize = offset_of!(ArithMetadata, execution_count);

const _: () = assert!(ARITH_METADATA_STRIDE == 8);
```

- [ ] **Step 4: Write `comparison.rs`** — identical shape to `arith.rs` with `ComparisonMetadata` name.

- [ ] **Step 5: Write `keyed_property.rs`** — same layout as `CallMetadata` (24B) but named `KeyedPropertyMetadata` with field `handler_bits` instead of `callee_bits`.

- [ ] **Step 6: Update `metadata_table.rs` module declarations**

```rust
pub mod kind;
pub mod property;
pub mod call;
pub mod arith;
pub mod comparison;
pub mod keyed_property;

pub use kind::{MetadataKind, METADATA_KIND_COUNT};
pub use property::{PropertyMetadata, PROPERTY_METADATA_STRIDE};
pub use call::{CallMetadata, CALL_METADATA_STRIDE};
pub use arith::{ArithMetadata, ARITH_METADATA_STRIDE};
pub use comparison::{ComparisonMetadata, COMPARISON_METADATA_STRIDE};
pub use keyed_property::{KeyedPropertyMetadata, KEYED_PROPERTY_METADATA_STRIDE};
```

Update `stride_for_kind_index` in `metadata_table.rs` to use the `*_STRIDE` constants instead of the placeholder values in `MetadataKind::stride_bytes`. Also update `MetadataKind::stride_bytes` to import & reference the strides — single source of truth.

- [ ] **Step 7: Run** `cargo test -p lyng-vm` — expect green (no behavior change, just new types).

- [ ] **Step 8: Commit**

```bash
git add crates/vm/src/vm/metadata_table*
git commit -m "vm/metadata_table: add per-kind metadata structs (Property/Call/Arith/Comparison/KeyedProperty)"
```

---

### Task 2.2: Slot-typed metadata accessors on `MetadataTable`

**Files:**
- Modify: `crates/vm/src/vm/metadata_table.rs`

- [ ] **Step 1: Write failing tests for typed accessors**

```rust
#[cfg(test)]
mod typed_access_tests {
    use super::*;
    use crate::vm::metadata_table::property::PropertyMetadata;
    use lyng_bytecode::metadata::FeedbackSiteKind;

    #[test]
    fn write_then_read_property_metadata_roundtrips() {
        let sites = vec![
            SiteDescriptor { slot: 1, kind: FeedbackSiteKind::NamedPropertyLoad },
            SiteDescriptor { slot: 2, kind: FeedbackSiteKind::NamedPropertyLoad },
        ];
        let mut table = MetadataTable::allocate(&sites);
        *table.property_mut(1) = PropertyMetadata {
            mode: 3,
            generation: 7,
            handler_bits: 0xdeadbeef,
            aux_bits: 0xcafe,
            execution_count: 42,
            ..Default::default()
        };
        let got = *table.property(1);
        assert_eq!(got.mode, 3);
        assert_eq!(got.generation, 7);
        assert_eq!(got.handler_bits, 0xdeadbeef);
        assert_eq!(got.aux_bits, 0xcafe);
        assert_eq!(got.execution_count, 42);
        // Slot 2 untouched.
        assert_eq!(*table.property(2), PropertyMetadata::default());
    }
}
```

- [ ] **Step 2: Run, expect FAIL** (`MetadataTable::property` doesn't exist).

- [ ] **Step 3: Implement typed accessors**

Add to `metadata_table.rs`:
```rust
use property::PropertyMetadata;
use call::CallMetadata;
use arith::ArithMetadata;
use comparison::ComparisonMetadata;
use keyed_property::KeyedPropertyMetadata;

impl MetadataTable {
    fn entry_byte_offset(&self, kind: MetadataKind, slot_one_based: u32) -> usize {
        let in_kind = self.in_kind_index_for_slot(slot_one_based) as usize;
        (self.kind_offset(kind) as usize) + in_kind * kind.stride_bytes()
    }

    pub fn property(&self, slot: u32) -> &PropertyMetadata {
        let off = self.entry_byte_offset(MetadataKind::Property, slot);
        // SAFETY: allocator reserves stride_bytes(Property)=32 at this offset;
        // PropertyMetadata is repr(C), 8-byte-aligned, allocator aligns runs to 8.
        unsafe { &*(self.buffer.as_ptr().add(off) as *const PropertyMetadata) }
    }

    pub fn property_mut(&mut self, slot: u32) -> &mut PropertyMetadata {
        let off = self.entry_byte_offset(MetadataKind::Property, slot);
        unsafe { &mut *(self.buffer.as_mut_ptr().add(off) as *mut PropertyMetadata) }
    }

    // …repeat for call/call_mut, arith/arith_mut, comparison/comparison_mut,
    // keyed_property/keyed_property_mut.
}
```

- [ ] **Step 4: Run, expect PASS.** Add roundtrip tests for the other four kinds (Call, Arith, Comparison, KeyedProperty) — same shape as the Property test.

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm/metadata_table.rs
git commit -m "vm/metadata_table: add typed slot accessors per kind"
```

---

### Task 2.3: `Vm::mirror_metadata_slot` — the dual-write hook

**Files:**
- Modify: `crates/vm/src/vm/feedback.rs`

- [ ] **Step 1: Add the helper alongside `mirror_flat_slot`**

In `crates/vm/src/vm/feedback.rs` near `mirror_flat_slot` (line 2141), add:
```rust
/// Phase C dual-write: project the semantic `FeedbackVector` slot state
/// into the parallel `MetadataTable` per-kind entry. Mirrors what
/// `mirror_flat_slot` does for `FeedbackEntry`, but routes per-kind.
pub(super) fn mirror_metadata_slot(&mut self, code: CodeRef, slot: FeedbackSlotId) {
    let Some(vector) = self.feedback_vector(code) else { return };
    let Some(site) = vector.site(slot) else { return };

    // Resolve kind from descriptor (FeedbackSlotId is global; per-kind index
    // lives in the MetadataTable).
    let Some(installed) = self.installed_function_for_code(code) else { return };
    let Some(descriptor) = installed.function.feedback_sites().iter().find(|d| d.slot() == slot)
        else { return };
    let kind = MetadataKind::from_site_kind(descriptor.kind());

    let Some(table) = self.metadata_table_mut(code) else { return };

    match kind {
        MetadataKind::Property => {
            *table.property_mut(slot.get()) = project_property(site);
        }
        MetadataKind::Call => {
            *table.call_mut(slot.get()) = project_call(site);
        }
        MetadataKind::Arith => {
            *table.arith_mut(slot.get()) = project_arith(site);
        }
        MetadataKind::Comparison => {
            *table.comparison_mut(slot.get()) = project_comparison(site);
        }
        MetadataKind::KeyedProperty => {
            *table.keyed_property_mut(slot.get()) = project_keyed_property(site);
        }
    }
}
```

Add the `project_*` free helpers below — each takes a `&FeedbackSiteState` and returns the matching metadata struct populated from the same fields the existing `mirror_flat_slot` reads. Mirror logic from `mirror_flat_slot` lines 2141-2183:

```rust
fn project_property(site: &FeedbackSiteState) -> PropertyMetadata {
    let header = site.as_named_property_header();  // existing helper
    PropertyMetadata {
        mode: header.mode,
        _pad: [0; 3],
        generation: header.generation,
        handler_bits: header.handler_bits,
        aux_bits: header.aux_bits,
        execution_count: header.execution_count,
        _tail_pad: 0,
    }
}

fn project_call(site: &FeedbackSiteState) -> CallMetadata {
    // Mirror the existing call IC projection. Refer to mirror_flat_slot for
    // the field mapping today.
    todo!("port from mirror_flat_slot's call projection")
}

// …similar for project_arith, project_comparison, project_keyed_property.
```

**Implementer note:** `mirror_flat_slot` today projects ONLY named-property IC fields (the `named_handler_bits` + `named_aux_bits` parts of `FeedbackEntry`). Per-kind metadata structs in Phase C cover the full per-site state, so each `project_*` extracts the relevant subset from `FeedbackSiteState`. Use the existing IC-state accessors (`as_named_property_handler`, `as_call_callee`, `as_scalar_observed`, etc.). When a site is in a state that doesn't apply to its declared kind (e.g. Uninit), return the zero-default.

- [ ] **Step 2: Add imports** at top of `feedback.rs`:

```rust
use crate::vm::metadata_table::{
    MetadataKind, PropertyMetadata, CallMetadata, ArithMetadata, ComparisonMetadata,
    KeyedPropertyMetadata,
};
```

- [ ] **Step 3: Run** `cargo build -p lyng-vm` — iterate until it compiles. Tests will follow.

- [ ] **Step 4: Commit**

```bash
git add crates/vm/src/vm/feedback.rs
git commit -m "vm/feedback: add mirror_metadata_slot helper (Phase C dual-write skeleton)"
```

---

### Task 2.4: Hook `mirror_metadata_slot` into every `mirror_flat_slot` call site

**Files:**
- Modify: `crates/vm/src/vm/feedback.rs` (lines 1586, 2135, 2310, 2328, 2441, 2465, 2513, 2593, 2912, 3189, 3216 per recon)
- Modify: `crates/vm/src/vm.rs` (line 1586 per recon — confirm via grep)

- [ ] **Step 1: Locate every call** (sanity check the recon):

```bash
rg -n 'mirror_flat_slot' crates/vm/src/
```

- [ ] **Step 2: For each `self.mirror_flat_slot(code, slot)`, add a parallel `self.mirror_metadata_slot(code, slot)` immediately after.**

Pattern:
```rust
self.mirror_flat_slot(code, slot);
self.mirror_metadata_slot(code, slot);  // Phase C dual-write
```

Do this in one commit per file (or one commit total if changes are small). Keep ordering consistent: `mirror_flat_slot` first, then `mirror_metadata_slot`.

- [ ] **Step 3: Run** `cargo test -p lyng-vm` — expect green (production behavior unchanged; reads still hit flat storage).

- [ ] **Step 4: Commit**

```bash
git add crates/vm/src/vm.rs crates/vm/src/vm/feedback.rs
git commit -m "vm: wire mirror_metadata_slot into every mirror_flat_slot site"
```

---

### Task 2.5: PR C.2 boundary check

- [ ] **Step 1: Run** `cargo test --workspace` — expect green.
- [ ] **Step 2: Run** `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check` — fix any.
- [ ] **Step 3: Commit** any cleanup. Skip if clean.

PR C.2 boundary: every IC mutation now writes BOTH `feedback_flat_storage` and `MetadataTable`. The asm fast path still reads from `feedback_flat_storage`. MetadataTable contents are populated but unread.

---

## PR C.3 — Debug-build equivalence assertion

### Task 3.1: Project flat entry → per-kind metadata for comparison

**Files:**
- Modify: `crates/vm/src/vm/feedback.rs`

- [ ] **Step 1: Add `debug_assert_metadata_matches_flat` helper**

```rust
#[cfg(debug_assertions)]
fn debug_assert_metadata_matches_flat(&self, code: CodeRef, slot: FeedbackSlotId) {
    let Some(installed) = self.installed_function_for_code(code) else { return };
    let Some(descriptor) = installed.function.feedback_sites().iter().find(|d| d.slot() == slot)
        else { return };
    let kind = MetadataKind::from_site_kind(descriptor.kind());

    let Some(table) = self.metadata_table(code) else { return };
    let idx = code_index(code);
    let Some(flat_box) = self.feedback_flat_storage.get(idx) else { return };
    let zero_based = (slot.get() - 1) as usize;
    let Some(flat_entry) = flat_box.get(zero_based) else { return };

    // Only Property kind has a meaningful flat-entry projection today
    // (FeedbackEntry mirrors named-property IC state). Other kinds are
    // checked by full-state round-trip via the FeedbackVector, not the
    // 64B flat entry.
    if kind == MetadataKind::Property {
        let table_entry = table.property(slot.get());
        debug_assert_eq!(
            table_entry.mode, flat_entry.mode,
            "MetadataTable.property.mode != FeedbackEntry.mode (code={code:?}, slot={slot:?})"
        );
        debug_assert_eq!(
            table_entry.generation, flat_entry.generation,
            "MetadataTable.property.generation != FeedbackEntry.generation"
        );
        debug_assert_eq!(
            table_entry.handler_bits, flat_entry.named_handler_bits,
            "MetadataTable.property.handler_bits != FeedbackEntry.named_handler_bits"
        );
        debug_assert_eq!(
            table_entry.aux_bits, flat_entry.named_aux_bits,
            "MetadataTable.property.aux_bits != FeedbackEntry.named_aux_bits"
        );
    }
}

#[cfg(not(debug_assertions))]
#[inline(always)]
fn debug_assert_metadata_matches_flat(&self, _code: CodeRef, _slot: FeedbackSlotId) {}
```

- [ ] **Step 2: Invoke the assertion from `mirror_metadata_slot`**

In `mirror_metadata_slot`, at the end of the function body (after the per-kind write), add:
```rust
#[cfg(debug_assertions)]
self.debug_assert_metadata_matches_flat(code, slot);
```

- [ ] **Step 3: Run** `cargo test --workspace` (debug mode by default) — expect green. If any assertion fires, the implementer must investigate the divergence — that's the whole point of this PR.

- [ ] **Step 4: Commit**

```bash
git add crates/vm/src/vm/feedback.rs
git commit -m "vm/feedback: add debug-only MetadataTable ↔ FeedbackEntry equivalence assertion"
```

---

### Task 3.2: Run full test suite under assertion

- [ ] **Step 1: Run** `cargo test --workspace` — expect every test pass. Any assertion fire indicates a missed mirror site or a projection bug; fix before proceeding.

- [ ] **Step 2: Run** `cargo test -p lyng-vm --test inline_caches` — explicit IC suite check.

- [ ] **Step 3: Run** `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`.

PR C.3 boundary: the dual-write is verified byte-correct for Property metadata across the full test suite. C.4 can safely flip the asm pin.

---

## PR C.4 — Asm DSL pin flip

### Task 4.1: Add metadata-table buffer pointer to `LlIntState`

**Files:**
- Modify: `crates/vm/src/dsl/llint_state.rs`

- [ ] **Step 1: Add a parallel field**

Don't rename `frame_fv_base` yet — that's used by the resolve macro until the macro is rewritten. Add:
```rust
pub(crate) frame_metadata_table_base: *mut u8,
```

Initialize to `std::ptr::null_mut()` in the constructor / default impl.

- [ ] **Step 2: Add offset constant for asm**

Match the pattern of `state_fv_base` constant (search the file):
```rust
pub(crate) const STATE_METADATA_TABLE_BASE_OFFSET: usize =
    std::mem::offset_of!(LlIntState, frame_metadata_table_base);
```

- [ ] **Step 3: Run** `cargo build -p lyng-vm` — expect green.

- [ ] **Step 4: Commit**

```bash
git add crates/vm/src/dsl/llint_state.rs
git commit -m "vm/dsl/llint_state: add frame_metadata_table_base alongside frame_fv_base"
```

---

### Task 4.2: Populate `frame_metadata_table_base` at trampoline entry

**Files:**
- Modify: `crates/vm/src/dsl/entry.rs` (line ~150 per recon)
- Modify: `crates/vm/src/dsl/slow_path.rs` (line ~235 per recon)

- [ ] **Step 1: At trampoline entry — set both pointers**

In `crates/vm/src/dsl/entry.rs` near where `frame_fv_base` is set from `feedback_flat_storage` (line ~150), add a parallel set:
```rust
// Phase C: also populate the metadata table base for the new resolve macro.
let mt_ptr = vm
    .metadata_tables
    .get(code_index)
    .and_then(|t| t.as_ref())
    .map(|t| t.buffer_ptr() as *mut u8)
    .unwrap_or(std::ptr::null_mut());
state.frame_metadata_table_base = mt_ptr;
```

Both pointers are now alive in the frame. The resolve macro (next task) decides which one to use.

- [ ] **Step 2: Same in `slow_path.rs` refresh** (line ~235)

When the slow-path rewrites `frame_fv_base` for a new code identity, also rewrite `frame_metadata_table_base`. Same pattern.

- [ ] **Step 3: Run** `cargo test --workspace` — expect green (no behavior change yet; asm still uses `frame_fv_base`).

- [ ] **Step 4: Commit**

```bash
git add crates/vm/src/dsl/entry.rs crates/vm/src/dsl/slow_path.rs
git commit -m "vm/dsl: populate frame_metadata_table_base in trampoline + slow path"
```

---

### Task 4.3: Rewrite the asm resolve macro

**Files:**
- Modify: `crates/vm/src/dsl/backend/aarch64/feedback.rs`
- Modify: `crates/vm/src/vm/metadata_table.rs` (export header layout constants)

- [ ] **Step 1: Export layout constants for asm use**

In `crates/vm/src/vm/metadata_table.rs`, expose:
```rust
pub const METADATA_TABLE_KIND_OFFSETS_OFFSET: usize = KIND_OFFSETS_OFFSET;
pub const METADATA_TABLE_SLOT_INDEX_TABLE_OFFSET: usize =
    KIND_OFFSETS_OFFSET + KIND_OFFSETS_SIZE;
```

- [ ] **Step 2: Replace `load_feedback_site!` with `load_metadata_slot!`**

In `crates/vm/src/dsl/backend/aarch64/feedback.rs`, the existing macro reads x21 as flat base. Replace its body with the 5-instruction resolve from the spec, parameterized on KIND and STRIDE_SHIFT (or use MADD for non-power-of-2 strides). The kind is known at emit time per opcode.

The macro signature changes from `load_feedback_site!($slot => $dst)` to `load_metadata_slot!($kind => $slot => $dst)` where `$kind` is a compile-time `MetadataKind` constant.

Pseudo-Rust (asm DSL syntax follows project conventions — refer to neighboring macros for actual syntax):
```rust
macro_rules! load_metadata_slot {
    (Property => $slot:tt => $dst:tt) => {
        // x21 = metadata table buffer base
        // x{slot} = 1-based FeedbackSlotId
        sub  x17, x{slot}, #1
        // idx = slot_to_in_kind_index[slot - 1]  (load u32, zero-extend)
        ldr  w{idx_tmp}, [x21, x17, lsl #2, +SLOT_INDEX_TABLE_OFFSET]
        // koff = kind_offsets[Property]
        ldr  w{koff_tmp}, [x21, +KIND_OFFSETS_OFFSET]
        // base = x21 + koff
        add  x{base_tmp}, x21, x{koff_tmp}
        // dst = base + idx * PROPERTY_METADATA_STRIDE (32 = lsl #5)
        add  x{dst}, x{base_tmp}, x{idx_tmp}, lsl #5
    };
    (Call => $slot:tt => $dst:tt) => {
        // Same as Property but with kind_offsets[Call] (offset 4) and MADD for 24B stride.
        sub  x17, x{slot}, #1
        ldr  w{idx_tmp}, [x21, x17, lsl #2, +SLOT_INDEX_TABLE_OFFSET]
        ldr  w{koff_tmp}, [x21, +KIND_OFFSETS_OFFSET + 4]
        add  x{base_tmp}, x21, x{koff_tmp}
        mov  x{stride_tmp}, #24
        madd x{dst}, x{idx_tmp}, x{stride_tmp}, x{base_tmp}
    };
    // …Arith (stride 8, lsl #3), Comparison (stride 8, lsl #3),
    // KeyedProperty (stride 24, MADD).
}
```

The exact asm DSL macro syntax follows the project's existing convention — check `load_feedback_site!`'s current source for the operator/punctuation grammar.

- [ ] **Step 3: Source x21 from `frame_metadata_table_base`**

Find where x21 is loaded at trampoline entry (`ldr x21, [x24, {state_fv}]` per recon at `entry.rs:255`). Add a parallel asm DSL macro that loads from `frame_metadata_table_base`. The hot loop entry uses this for x21 instead.

Strategy: keep both loads as choices, switch the trampoline-entry instruction to load from `state_metadata_table_base` in the same commit that flips the resolve macro to use `load_metadata_slot!`.

- [ ] **Step 4: Update every opcode that reads feedback** to use the new macro. Search for `load_feedback_site!` to find all sites; replace with `load_metadata_slot!($kind => ...)` where `$kind` is determined by the opcode (e.g. `OpGetById` → `Property`, `OpCall` → `Call`).

- [ ] **Step 5: Update `FeedbackEntry` field-offset asm references**

Replace asm reads of `FEEDBACK_ENTRY_NAMED_HANDLER_BITS_OFFSET = 8` etc. with `PROPERTY_METADATA_HANDLER_BITS_OFFSET = 8` (the offsets happen to match for Property since both have `mode` at 0, generation at 4, handler_bits at 8, aux_bits at 16). Same offsets — only the symbol names change.

For other kinds (Call, Arith, etc.), use the kind-specific offset constants from `metadata_table/*.rs`.

- [ ] **Step 6: Run** `cargo test --workspace` — this is the high-risk moment. Expect green; if not, the resolve macro has a layout bug. Use `crates/vm/src/tests/inline_caches.rs` failures to debug.

- [ ] **Step 7: Run** `cargo test -p lyng-vm --test inline_caches` explicitly — IC suite must be green.

- [ ] **Step 8: Commit**

```bash
git add crates/vm/src/dsl/ crates/vm/src/vm/metadata_table.rs
git commit -m "vm/dsl: flip asm IC resolve to MetadataTable (load_metadata_slot! macro)"
```

---

### Task 4.4: Mark `feedback_flat_storage` as legacy (no deletion yet)

**Files:**
- Modify: `crates/vm/src/vm.rs`
- Modify: `crates/vm/src/dsl/feedback_flat.rs`

- [ ] **Step 1: Update docstrings**

Mark `feedback_flat_storage` as "legacy, awaiting Phase D deletion. The asm fast path no longer reads this; mirror_flat_slot updates it solely for the debug-equivalence assertion."

Mark `mirror_flat_slot` as "legacy, kept for debug equivalence; deleted in Phase D."

- [ ] **Step 2: Verify the asm path no longer touches `feedback_flat_storage`**

Run:
```bash
rg -n "feedback_flat_storage" crates/vm/src/dsl/
```
Expected: zero matches (the asm DSL no longer references it). If matches remain, investigate.

- [ ] **Step 3: No production code change here, just verification. Commit only if docstrings changed.**

```bash
git add crates/vm/src/vm.rs crates/vm/src/dsl/feedback_flat.rs
git commit -m "vm: mark feedback_flat_storage as legacy (read by asm: no; written: yes for C.3)"
```

---

### Task 4.5: Test C4 + C6 — asm reads MetadataTable; GC releases table

**Files:**
- Modify: `crates/vm/src/tests/inline_caches.rs`

- [ ] **Step 1: Add C4 — asm fast path reads MetadataTable**

Write a test that:
1. Installs a function that loads `o.x` in a hot loop.
2. Runs until the IC caches monomorphic.
3. Mutates `MetadataTable.property_mut(slot).handler_bits = 0` directly.
4. Reads `o.x` again — verify the asm fast path now misses (returns to slow path).
5. Verify the slow path re-caches.

This proves the asm fast path is sourcing from MetadataTable, not the old flat storage. (Mutating only the table — not the flat storage or FeedbackVector — and observing the behavior change is the litmus.)

- [ ] **Step 2: Add C6 — GC releases the table**

In `crates/vm/src/tests/feedback.rs`:
```rust
#[test]
fn metadata_table_released_when_code_is_gcd() {
    let mut agent = build_agent();
    let unit = compile_test_unit(200, r"function f() { return 42; }");
    let code = {
        let installed = vm_install_script(&mut agent, &unit);
        let c = installed.code();
        // …drop installed reference path …
        c
    };
    let code_idx = code.raw().get() as usize - 1;
    // Force GC. (Use the existing test helper for full collection.)
    agent.heap_mut().run_full_collection();
    // After GC, the metadata table slot for this code should be cleared.
    assert!(agent.vm().metadata_tables.get(code_idx).map_or(true, |t| t.is_none()));
}
```

Verify the GC sweep for dead code drops the metadata table entry. If the existing GC sweep doesn't handle `metadata_tables`, add a `prune_dead_code_metadata_tables` helper following the Phase B pattern (`prune_dead_code_polymorphic_chains` at `vm.rs`).

- [ ] **Step 3: Run** new tests, expect PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/vm/src/tests/inline_caches.rs crates/vm/src/tests/feedback.rs crates/vm/src/vm.rs
git commit -m "vm/tests: C4 asm reads MetadataTable + C6 GC releases table"
```

---

### Task 4.6: Microbench + final verification

- [ ] **Step 1: Run microbench**

```bash
cargo bench -p lyng-vm --bench property_addition -- --baseline pre-spec2
```

Compare wall-clock delta vs the pre-Spec-2 baseline. **Ceiling: ≤3%** per spec §1 exit criteria.

- [ ] **Step 2: Run V8 suite** (if available — check `crates/vm/v8-bench/` or `bench/v8/` for the canonical command):

```bash
# Run V8 bench; compare against Phase B baseline.
```

- [ ] **Step 3: Full workspace verification**

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```

All green required.

- [ ] **Step 4: Commit** any cleanup. Skip if clean.

PR C.4 boundary: Phase C is complete. `MetadataTable` is the system of record for asm dispatch. `feedback_flat_storage` and `FeedbackVector` remain alive for the debug-equivalence assertion and as the semantic state-machine source (deleted in Phase D).

---

## Verification (Phase C end-to-end)

After PR C.4 lands:

1. **All tests:** `cargo test --workspace` → green.
2. **IC regression:** `cargo test -p lyng-vm --test inline_caches` → green; existing ~33 IC tests pass on the new asm path.
3. **MetadataTable inspection tests:** C1–C7 all pass.
4. **Microbench:** wall-clock delta within ≤3% of `pre-spec2` baseline.
5. **V8 bench:** within tolerance vs. Phase B baseline.
6. **No new clippy warnings; cargo fmt clean.**
7. **Asm contract:** `rg "feedback_flat_storage" crates/vm/src/dsl/` returns zero matches — the asm path is fully off legacy storage.
8. **Phase D readiness:** `feedback_flat_storage` + `mirror_flat_slot` + `FeedbackEntry` + the debug-equivalence assertion all exist but are unused by production code; Phase D's deletion is mechanical.

## Out of scope (deferred)

- **Phase D:** delete `feedback_flat_storage`, `mirror_flat_slot`, `FeedbackEntry`, the debug equivalence assertion, and re-home the IC state machine onto per-kind metadata impls.
- **Phase E:** add `*Status` projections; delete `FeedbackVectorSnapshot`/`Footprint`; update test consumers.
- **Asm DSL non-aarch64 backends:** if any exist (check `crates/vm/src/dsl/backend/`), the analogous resolve macro is added there in the same PR as C.4.
- **`PropertyMetadata` aux fields beyond `handler_bits`/`aux_bits`:** Phase C carries forward only what `FeedbackEntry` carries (the IC fast-path projection). Full IC state (chain entries, polymorphic sidecar, etc.) stays in `NamedPropertyFeedback`; Phase D ports it.
