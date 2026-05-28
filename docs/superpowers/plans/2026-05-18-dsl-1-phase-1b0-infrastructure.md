# DSL-1 Phase 1.B.0 — Infrastructure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire the slow-path-share and per-opcode dispatch counters into the DSL `dispatch!`/`call_slow!`/`poll_safepoint!` macros (Task 10.A), and add microbench snippets for the 14 in-scope opcodes (Task 10.B). These two infra deliverables make per-opcode gates enforceable for the rest of DSL-1.

**Architecture:** Replace the `Option<OpcodeDispatchCounterStore>` field on `Vm` with an asm-stable `Box<DispatchCounters>` containing three flat `[u64; 256]` banks (dispatch, slow_semantic, slow_safepoint). The pointer offset is exposed via `offset_of!` and bound into the per-handler `naked_asm!` block. The proc-macro lowerer emits `inc_counter!` at handler entry; `call_slow!` and `poll_safepoint!` macros gain inline counter increments for the slow-path banks. Microbench snippets follow the existing pattern in [`tools/lyng-bench/src/microbench/snippets.rs`](../../../tools/lyng-bench/src/microbench/snippets.rs).

**Tech Stack:** Rust 2024 stable (≥1.88 for `naked_asm!`), `lyng-vm-dsl` proc-macro (modified), AArch64 backend macros, `cargo-features` for `diagnostic-counters` gate.

**Parent spec:** [`docs/superpowers/specs/2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md`](../specs/2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md) — Phase 1.B.0.

---

## Scope

This plan covers **Phase 1.B.0 only**. Phases 1.B.1 (frame-context refactor), 1.B.2 (op_load_const8/this backfill), 1.B.3 (opcode ports) each get their own plan at sub-phase boundaries.

At plan completion: counter infrastructure produces correct per-opcode dispatch counts and slow-path-share data on V8 v7 workloads; microbench produces ns/dispatch with CI95 for 14 in-scope opcodes; the 1.B.0 gate (counter+microbench sanity + ≤5% per-feature overhead) is met.

**Off-ramp:** if the proc-macro lowerer change to emit `inc_counter!` at handler entry proves unworkable (e.g., the opcode-byte→handler-symbol mapping isn't available at lower time), pause and report. The coordinator decides whether to refactor the lowerer separately or to fall back to a per-handler hand-coded counter increment.

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| [`crates/vm/src/opcode_counts.rs`](../../../crates/vm/src/opcode_counts.rs) | Modify | Add `DispatchCounters` `#[repr(C)]` struct with 3 flat counter banks. Update `OpcodeDispatchCounterStore` API to read from flat arrays. |
| [`crates/vm/src/vm.rs`](../../../crates/vm/src/vm.rs) | Modify | Replace `Option<OpcodeDispatchCounterStore>` field with `Box<DispatchCounters>` (always allocated when feature is on). |
| [`crates/vm/src/dsl/reg_convention.rs`](../../../crates/vm/src/dsl/reg_convention.rs) | Modify | Add `VM_DISPATCH_COUNTERS_PTR_OFFSET`, `DISPATCH_COUNTER_BANK_DISPATCH`, `DISPATCH_COUNTER_BANK_SLOW_SEMANTIC`, `DISPATCH_COUNTER_BANK_SLOW_SAFEPOINT` consts via `offset_of!`. |
| [`crates/vm/src/dsl/backend/aarch64/counters.rs`](../../../crates/vm/src/dsl/backend/aarch64/counters.rs) | Modify | Replace existing `inc_counter!` macro with three new variants: `inc_dispatch_counter!`, `inc_slow_semantic_counter!`, `inc_slow_safepoint_counter!`, each taking `$opcode_byte:literal`. |
| [`crates/vm/src/dsl/backend/aarch64/control.rs`](../../../crates/vm/src/dsl/backend/aarch64/control.rs) | Modify | Wire `inc_slow_semantic_counter!` into `call_slow!` macros (needs opcode-byte parameter). Wire `inc_slow_safepoint_counter!` into the slow path of `poll_safepoint!`. |
| [`crates/vm-dsl/src/lower.rs`](../../../crates/vm-dsl/src/lower.rs) | Modify | Emit `inc_dispatch_counter!(OPCODE_BYTE)` at the start of every generated handler body. Map handler name → Opcode discriminant. Also pass `opcode_byte` as a binding so the call_slow!/poll_safepoint! macros can use it. |
| [`tools/lyng-bench/src/microbench/snippets.rs`](../../../tools/lyng-bench/src/microbench/snippets.rs) | Modify | Add 14 new `Snippet` entries: 7 Phase-1.A opcodes + 7 Phase-1.B anchors. |
| [`crates/vm/tests/`](../../../crates/vm/tests/) | Modify or Create | Update `ll_int_state_offsets_stable` to also check `Vm` size + `dispatch_counters` offset. Add counter-correctness test (Move dispatch count on Richards ≈ 4.66B ± 5%). |
| [`reports/lyng/dsl-1/phase-1b0-summary.md`](../../../reports/lyng/dsl-1/) | Create | Sub-phase gate summary with measured counter overhead + CI95 confirmation per opcode. |

---

## Task 0: Kickoff — verify starting state

**Files:**
- Read: [`reports/lyng/dsl-1/phase-1a-summary.md`](../../../reports/lyng/dsl-1/phase-1a-summary.md)
- Read: [`crates/vm/src/dsl/backend/aarch64/counters.rs`](../../../crates/vm/src/dsl/backend/aarch64/counters.rs)
- Read: [`crates/vm-dsl/src/lower.rs`](../../../crates/vm-dsl/src/lower.rs)

- [ ] **Step 1: Verify all tests pass at starting HEAD**

```bash
cargo test -p lyng-vm --lib --release 2>&1 | tail -3
cargo test -p lyng-tests --release 2>&1 | tail -3
```

Expected: 413 + 1186 passing (matching Phase 1.A's end state).

- [ ] **Step 2: Verify counter feature compiles cleanly**

```bash
cargo build --release -p lyng-bench 2>&1 | tail -5
```

`lyng-bench` already enables `lyng-vm/diagnostic-counters` in its Cargo.toml. Verify clean build.

- [ ] **Step 3: Capture baseline V8 v7 (for later same-load A/B)**

```bash
git rev-parse HEAD > /tmp/phase-1b0-base-sha
cargo run --release -p lyng-bench -- v8suite --samples 7 --json /tmp/phase-1b0-base-v8.json 2>&1 | tail -10
uptime > /tmp/phase-1b0-base-uptime
```

Records the starting HEAD SHA, V8 v7 numbers, and machine load for the eventual same-load A/B at the phase gate.

- [ ] **Step 4: Locate `Opcode` enum + OPCODE_COUNT**

```bash
grep -n "pub enum Opcode\|OPCODE_COUNT" crates/bytecode/src/lib.rs crates/bytecode/src/*.rs 2>&1 | head -10
```

Confirm: `OPCODE_COUNT` is a `u8` const for the number of opcode variants (likely 152). Verify the `Opcode` enum's discriminant assignment is `#[repr(u8)]` so `opcode as u8` gives the dispatch byte.

This is needed for the lowerer task — the lowerer must resolve a handler symbol like `op_load_undefined_dsl` to its `Opcode` byte.

- [ ] **Step 5: Inspect the proc-macro lowerer**

Read [`crates/vm-dsl/src/lower.rs`](../../../crates/vm-dsl/src/lower.rs) (276 lines). Note:
- How the lowerer builds the `naked_asm!` template (the bindings section).
- Where in the emission order the operand-decode prologue is emitted.
- What named-arg bindings are already in place (`length`, `state_pc`, `state_pb`, `state_regs`, `state_fv`, `state_prefix`, `vm_poll`, `entry_stride_shift`, `entry_observed`, `exit`).

`inc_dispatch_counter!` must be emitted BEFORE the decode prologue (so the counter increments on entry, regardless of whether the handler body branches early).

- [ ] **Step 6: Commit a no-op marker for the plan kickoff**

```bash
cat > /tmp/kickoff-msg.txt <<'EOF'
DSL-1 Phase 1.B.0 kickoff: pre-infra baselines captured

V8 v7 baseline: /tmp/phase-1b0-base-v8.json (loadavg: see /tmp/phase-1b0-base-uptime).
Starting HEAD: see /tmp/phase-1b0-base-sha.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
```

No git commit needed — the baseline files are in /tmp. This step exists to record the start point in the conversation transcript. Mark the task complete and proceed.

---

## Task 1: Add `DispatchCounters` struct + replace `Option<...>` Vm field

**Files:**
- Modify: `crates/vm/src/opcode_counts.rs:1-130` (add `DispatchCounters` struct; rewrite `OpcodeDispatchCounterStore` to wrap it)
- Modify: `crates/vm/src/vm.rs:166` (replace field) + `vm.rs:253` (replace initializer) + `vm.rs:309-368` (rewrite the enable/disable/reset/snapshot/maybe_record API)
- Test: `crates/vm/tests/dispatch_counters_layout.rs` (new — verify struct size + offset stability)

- [ ] **Step 1: Add `DispatchCounters` struct to `opcode_counts.rs`**

After the existing `OpcodeDispatchCounts` impl block, add:

```rust
/// Flat counter banks for the asm-driven counter increments.
///
/// Layout is `#[repr(C)]` with three sequential `[u64; 256]` banks:
/// - `dispatch[op]`        — bumped at handler entry by `inc_dispatch_counter!`
/// - `slow_semantic[op]`   — bumped at `call_slow!` invocation site
/// - `slow_safepoint[op]`  — bumped at `poll_safepoint!` pending branch
///
/// Indexed by raw opcode byte (`opcode as u8`). 256 entries reserves
/// space for the full byte range even though Lyng uses ~152 opcodes,
/// to keep offset math fast (compile-time bank offsets are 0, 2048,
/// 4096 — all encodable as AArch64 LDR/STR immediates).
///
/// Box-allocated so the Vm pointer stays stable across struct moves
/// (Vm itself isn't pinned; the asm-side `[VM, #offset]` access reads
/// the pointer first, then indexes into the heap-allocated array).
#[repr(C)]
pub struct DispatchCounters {
    pub dispatch: [u64; 256],
    pub slow_semantic: [u64; 256],
    pub slow_safepoint: [u64; 256],
}

impl DispatchCounters {
    pub fn new() -> Self {
        // Stack-allocate zeros, then move into Box to avoid a 6 KB stack frame.
        Self {
            dispatch: [0; 256],
            slow_semantic: [0; 256],
            slow_safepoint: [0; 256],
        }
    }

    pub fn reset(&mut self) {
        self.dispatch.fill(0);
        self.slow_semantic.fill(0);
        self.slow_safepoint.fill(0);
    }

    /// Snapshot the dispatch bank into an `OpcodeDispatchCounts`. The
    /// `slow_*` banks are exposed separately via `slow_semantic_count`/
    /// `slow_safepoint_count` accessors.
    pub fn snapshot_dispatch(&self) -> OpcodeDispatchCounts {
        let mut snapshot = OpcodeDispatchCounts::zeroed();
        snapshot.counts = self.dispatch.to_vec();
        snapshot
    }

    pub fn slow_semantic_count(&self, opcode: lyng_bytecode::Opcode) -> u64 {
        self.slow_semantic[opcode as u8 as usize]
    }

    pub fn slow_safepoint_count(&self, opcode: lyng_bytecode::Opcode) -> u64 {
        self.slow_safepoint[opcode as u8 as usize]
    }
}

impl Default for DispatchCounters {
    fn default() -> Self {
        Self::new()
    }
}
```

Note: this requires `OpcodeDispatchCounts::counts` (currently private `Vec<u64>`) to be accessible from `snapshot_dispatch`. Either make `counts` `pub(crate)` or add a `pub(crate) fn from_dispatch_array(arr: &[u64; 256]) -> Self` constructor. Pick the constructor approach — cleaner encapsulation:

```rust
impl OpcodeDispatchCounts {
    pub(crate) fn from_dispatch_array(arr: &[u64; 256]) -> Self {
        Self { counts: arr.to_vec() }
    }
}
```

Update `DispatchCounters::snapshot_dispatch` to use this.

- [ ] **Step 2: Update `OpcodeDispatchCounterStore` to wrap `DispatchCounters`**

Replace the entire `OpcodeDispatchCounterStore` struct in `opcode_counts.rs` with a thin wrapper that owns the `Box<DispatchCounters>`:

```rust
pub struct OpcodeDispatchCounterStore {
    counters: Box<DispatchCounters>,
}

impl OpcodeDispatchCounterStore {
    pub fn new() -> Self {
        Self {
            counters: Box::new(DispatchCounters::new()),
        }
    }

    pub fn counters(&self) -> &DispatchCounters {
        &self.counters
    }

    pub fn counters_mut(&mut self) -> &mut DispatchCounters {
        &mut self.counters
    }

    pub fn counters_ptr(&self) -> *const DispatchCounters {
        &*self.counters as *const _
    }

    pub fn counters_mut_ptr(&mut self) -> *mut DispatchCounters {
        &mut *self.counters as *mut _
    }

    #[inline]
    pub fn increment(&self, opcode: lyng_bytecode::Opcode) {
        // SAFETY: single-threaded VM; counter writes are tear-free on
        // aligned u64. The asm path uses non-atomic `add x10, x10, #1`
        // which has the same single-threaded guarantee.
        unsafe {
            let counters = &mut *(self.counters_ptr() as *mut DispatchCounters);
            counters.dispatch[opcode as u8 as usize] =
                counters.dispatch[opcode as u8 as usize].saturating_add(1);
        }
    }

    pub fn reset(&self) {
        // SAFETY: same as `increment`.
        unsafe {
            let counters = &mut *(self.counters_ptr() as *mut DispatchCounters);
            counters.reset();
        }
    }

    pub fn snapshot(&self) -> OpcodeDispatchCounts {
        self.counters.snapshot_dispatch()
    }
}
```

The `unsafe` blocks are safe because the VM is single-threaded; the counter writes don't need atomicity. The asm path will use the same access pattern (non-atomic `add x10, x10, #1`).

Note: this changes the previous interior-mutability via `Cell<u64>` to direct `u64` access (since the asm path can't use `Cell`). The Rust API preserves the same `&self` increment semantics via the `unsafe` raw-pointer indirection — single-threaded, so this is sound.

- [ ] **Step 3: Update `Vm` struct field**

In `crates/vm/src/vm.rs` around line 166, replace:

```rust
opcode_dispatch_counts: Option<OpcodeDispatchCounterStore>,
```

with:

```rust
#[cfg(feature = "diagnostic-counters")]
dispatch_counters: OpcodeDispatchCounterStore,
```

In the `Vm::new()` initializer around line 253, replace:

```rust
opcode_dispatch_counts: None,
```

with (conditionally):

```rust
#[cfg(feature = "diagnostic-counters")]
dispatch_counters: OpcodeDispatchCounterStore::new(),
```

For the feature-gated path, the field is ALWAYS present when the feature is on; activation just resets the counters.

- [ ] **Step 4: Rewrite the enable/disable/reset/snapshot API in vm.rs**

Replace the `enable_opcode_dispatch_counts`, `disable_opcode_dispatch_counts`, `reset_opcode_dispatch_counts`, `opcode_dispatch_counts`, `maybe_record_opcode_dispatch` functions (vm.rs:309-368) with:

```rust
#[cfg(feature = "diagnostic-counters")]
pub fn enable_opcode_dispatch_counts(&mut self) {
    // No-op when counters are always allocated. Kept for backward
    // compatibility with tests that called this method.
}

#[cfg(feature = "diagnostic-counters")]
pub fn disable_opcode_dispatch_counts(&mut self) {
    // Reset to zero so subsequent runs start fresh. No actual
    // deallocation — the counter array stays for the asm path.
    self.dispatch_counters.reset();
}

#[cfg(feature = "diagnostic-counters")]
pub fn reset_opcode_dispatch_counts(&mut self) {
    self.dispatch_counters.reset();
}

#[cfg(feature = "diagnostic-counters")]
#[inline]
pub fn opcode_dispatch_counts(&self) -> Option<OpcodeDispatchCounts> {
    Some(self.dispatch_counters.snapshot())
}

#[cfg(feature = "diagnostic-counters")]
pub fn dispatch_counters(&self) -> &OpcodeDispatchCounterStore {
    &self.dispatch_counters
}

// `maybe_record_opcode_dispatch` is DELETED — the asm path now writes
// directly to dispatch_counters.dispatch[op]. No Rust hook needed.
```

Remove the `maybe_record_opcode_dispatch` function entirely. Any callers (search with `grep -rn "maybe_record_opcode_dispatch" crates/`) get removed too — verify there are no remaining callers before deleting.

- [ ] **Step 5: Add the layout-stability test**

Create `crates/vm/tests/dispatch_counters_layout.rs`:

```rust
//! Verify `DispatchCounters` layout is stable for asm access.
//!
//! The asm path reads `[VM, #VM_DISPATCH_COUNTERS_PTR_OFFSET]` to get
//! the counter base pointer, then indexes into it with compile-time
//! bank offsets (0, 2048, 4096). These tests pin the layout invariants
//! so a future rustc upgrade or struct re-ordering can't silently
//! break the asm-side reads.

#![cfg(feature = "diagnostic-counters")]

use std::mem::{offset_of, size_of};

use lyng_vm::DispatchCounters;

#[test]
fn dispatch_counters_size_is_expected() {
    // 3 banks × 256 entries × 8 bytes = 6144 bytes.
    assert_eq!(size_of::<DispatchCounters>(), 3 * 256 * 8);
}

#[test]
fn dispatch_counters_field_offsets_are_stable() {
    assert_eq!(offset_of!(DispatchCounters, dispatch), 0);
    assert_eq!(offset_of!(DispatchCounters, slow_semantic), 256 * 8);
    assert_eq!(offset_of!(DispatchCounters, slow_safepoint), 512 * 8);
}
```

You'll need to add `pub use opcode_counts::DispatchCounters;` to `crates/vm/src/lib.rs` for the test to find it.

- [ ] **Step 6: Build + run tests**

```bash
cargo build --release -p lyng-vm
cargo test -p lyng-vm --lib --release 2>&1 | tail -3
cargo test --release --test dispatch_counters_layout 2>&1 | tail -5
```

Expected: 413 (or more — the new test adds 2) passing in the lib suite; 2 passing in the dispatch_counters_layout test.

If the build fails on a removed `maybe_record_opcode_dispatch` caller, fix it (likely an existing test or unused code).

- [ ] **Step 7: Commit**

```bash
git add \
  crates/vm/src/opcode_counts.rs \
  crates/vm/src/vm.rs \
  crates/vm/src/lib.rs \
  crates/vm/tests/dispatch_counters_layout.rs
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.0 Task 1: add DispatchCounters with asm-stable layout

Replace Option<OpcodeDispatchCounterStore> with a Box<DispatchCounters>
holding 3 flat [u64; 256] banks (dispatch, slow_semantic, slow_safepoint).
#[repr(C)] guarantees offset stability for asm access.

OpcodeDispatchCounterStore is now a thin wrapper exposing the same
public API for backward compat with existing tests. The asm path
will write directly to the flat arrays via the new VM offset
(added in Task 3).

maybe_record_opcode_dispatch deleted — the asm path doesn't need a
Rust hook anymore.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Add asm-stable offset consts in reg_convention.rs

**Files:**
- Modify: [`crates/vm/src/dsl/reg_convention.rs`](../../../crates/vm/src/dsl/reg_convention.rs)

- [ ] **Step 1: Read the existing offset-const layout**

```bash
cat crates/vm/src/dsl/reg_convention.rs
```

Identify the existing offset consts (`LLINT_STATE_FRAME_PC_OFFSET`, etc.) and the pattern they use.

- [ ] **Step 2: Add the new VM-relative consts**

In `crates/vm/src/dsl/reg_convention.rs`, after the existing offset consts (likely near the bottom, before the closing `}` if there's a module), add:

```rust
// =============================================================================
// VM-relative offsets (read from pinned register x22 = VM).
// =============================================================================

/// Byte offset of `Vm::dispatch_counters` (the `OpcodeDispatchCounterStore`).
///
/// The asm-side `inc_dispatch_counter!` macro reads `[x22, #VM_DISPATCH_COUNTERS_PTR_OFFSET]`
/// to get a `*mut OpcodeDispatchCounterStore`, then dereferences it to
/// reach the inner `Box<DispatchCounters>` pointer. Note: this is a
/// two-step indirection (Vm field → inner Box) because the Store wraps
/// the Box.
///
/// Only valid when the `diagnostic-counters` feature is on; otherwise the
/// field doesn't exist.
#[cfg(feature = "diagnostic-counters")]
pub const VM_DISPATCH_COUNTERS_PTR_OFFSET: usize =
    ::core::mem::offset_of!(crate::vm::Vm, dispatch_counters);

/// Byte offset of the `dispatch` bank within `DispatchCounters`. 0
/// because it's the first field.
#[cfg(feature = "diagnostic-counters")]
pub const DISPATCH_COUNTER_BANK_DISPATCH: usize = 0;

/// Byte offset of the `slow_semantic` bank within `DispatchCounters`.
/// 256 × 8 = 2048.
#[cfg(feature = "diagnostic-counters")]
pub const DISPATCH_COUNTER_BANK_SLOW_SEMANTIC: usize = 256 * 8;

/// Byte offset of the `slow_safepoint` bank within `DispatchCounters`.
/// 512 × 8 = 4096.
#[cfg(feature = "diagnostic-counters")]
pub const DISPATCH_COUNTER_BANK_SLOW_SAFEPOINT: usize = 512 * 8;
```

CRITICAL: Verify the `Vm::dispatch_counters` field path is correct (might need `pub` visibility for `offset_of!` to reach it — adjust visibility in Task 1's commit if needed).

Also note: `OpcodeDispatchCounterStore` is a wrapper around `Box<DispatchCounters>`. The asm-side code needs to:
1. Read the `OpcodeDispatchCounterStore` field (at `VM_DISPATCH_COUNTERS_PTR_OFFSET`).
2. Dereference into the inner `Box` field — which is the first field of the Store, so offset 0 relative to the Store.
3. The Box is a raw pointer; one more deref reaches the actual `DispatchCounters`.

This is TWO loads to reach the counter array — one for the Box pointer, then indexed loads into the array. Document this in the macro comment.

- [ ] **Step 3: Build + verify**

```bash
cargo build --release -p lyng-vm 2>&1 | tail -5
```

Expected: clean build. If `offset_of!` complains about field access, make `dispatch_counters` `pub(crate)` in vm.rs.

- [ ] **Step 4: Commit**

```bash
git add crates/vm/src/dsl/reg_convention.rs crates/vm/src/vm.rs
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.0 Task 2: add VM-relative counter offset consts

VM_DISPATCH_COUNTERS_PTR_OFFSET (resolved via offset_of!) and the
three bank offsets (0, 2048, 4096) for use by the asm-side counter
increment macros.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Replace `inc_counter!` macro with three bank variants

**Files:**
- Modify: [`crates/vm/src/dsl/backend/aarch64/counters.rs`](../../../crates/vm/src/dsl/backend/aarch64/counters.rs)

- [ ] **Step 1: Read the existing macro**

The current macro at lines 24-41 takes `$opcode_byte:literal` and emits 4 instructions. It references `{vm_counter_base}` — a binding that doesn't yet exist. We're rewriting this to use the new offset consts.

- [ ] **Step 2: Replace with three bank-specific macros**

Rewrite `crates/vm/src/dsl/backend/aarch64/counters.rs` to:

```rust
//! Opcode-counter increments, gated by `--features diagnostic-counters`.
//!
//! When the feature is off, the macros expand to empty strings — zero
//! per-dispatch cost. When on, each emits 5 instructions to bump the
//! per-opcode counter slot in the relevant DispatchCounters bank:
//!
//! ```text
//!     ldr  x9, [x22, {vm_counter_base}]   ; ptr to OpcodeDispatchCounterStore
//!     ldr  x9, [x9]                        ; deref Box<DispatchCounters>
//!     ldr  x10, [x9, #<bank_offset + op*8>]
//!     add  x10, x10, #1
//!     str  x10, [x9, #<bank_offset + op*8>]
//! ```
//!
//! Bindings expected (only when feature is on):
//! - `{vm_counter_base}` — `const VM_DISPATCH_COUNTERS_PTR_OFFSET`.
//!
//! `$opcode_byte` is the opcode discriminator (`u8`) baked at lower
//! time. `bank_offset` is one of: 0 (dispatch), 2048 (slow_semantic),
//! 4096 (slow_safepoint). The compiler synthesizes the addressing
//! mode for `(bank_offset + op * 8)` — for op < 64 the immediate
//! encoding works for all banks; for larger op the assembler uses a
//! shifted-add form.

// =============================================================================
// Opcode-counters feature ON: emit real counter increments.
// =============================================================================

#[cfg(feature = "diagnostic-counters")]
#[macro_export]
macro_rules! inc_dispatch_counter {
    ($opcode_byte:literal) => {
        concat!(
            "ldr    x9, [x22, {vm_counter_base}]\n",
            "ldr    x9, [x9]\n",
            "ldr    x10, [x9, #", stringify!($opcode_byte), " * 8]\n",
            "add    x10, x10, #1\n",
            "str    x10, [x9, #", stringify!($opcode_byte), " * 8]\n",
        )
    };
}

#[cfg(feature = "diagnostic-counters")]
#[macro_export]
macro_rules! inc_slow_semantic_counter {
    ($opcode_byte:literal) => {
        concat!(
            "ldr    x9, [x22, {vm_counter_base}]\n",
            "ldr    x9, [x9]\n",
            "ldr    x10, [x9, #", stringify!($opcode_byte), " * 8 + 2048]\n",
            "add    x10, x10, #1\n",
            "str    x10, [x9, #", stringify!($opcode_byte), " * 8 + 2048]\n",
        )
    };
}

#[cfg(feature = "diagnostic-counters")]
#[macro_export]
macro_rules! inc_slow_safepoint_counter {
    ($opcode_byte:literal) => {
        concat!(
            "ldr    x9, [x22, {vm_counter_base}]\n",
            "ldr    x9, [x9]\n",
            "ldr    x10, [x9, #", stringify!($opcode_byte), " * 8 + 4096]\n",
            "add    x10, x10, #1\n",
            "str    x10, [x9, #", stringify!($opcode_byte), " * 8 + 4096]\n",
        )
    };
}

// =============================================================================
// Opcode-counters feature OFF: empty strings (zero per-dispatch cost).
// =============================================================================

#[cfg(not(feature = "diagnostic-counters"))]
#[macro_export]
macro_rules! inc_dispatch_counter {
    ($opcode_byte:literal) => { "" };
}

#[cfg(not(feature = "diagnostic-counters"))]
#[macro_export]
macro_rules! inc_slow_semantic_counter {
    ($opcode_byte:literal) => { "" };
}

#[cfg(not(feature = "diagnostic-counters"))]
#[macro_export]
macro_rules! inc_slow_safepoint_counter {
    ($opcode_byte:literal) => { "" };
}
```

CRITICAL: Verify the AArch64 immediate encoding works for the bank+opcode offsets. For `slow_safepoint`, the immediate `op*8 + 4096` may exceed the LDR/STR unsigned-immediate range (it goes up to op=255 → 4096 + 2040 = 6136). LDR's unscaled immediate is -256..+255; the scaled (`#imm * scale`) form for u64 supports `#0..#32760` (4095 × 8). So we're fine up to 4096 + 2040 = 6136, which is within the scaled range. Good.

If the assembler complains about the computed-immediate form (`#X * 8 + 2048`), the macro will need to materialize the offset in a scratch register first. Test this before committing.

- [ ] **Step 3: Delete the old `inc_counter!` macro**

Search for any remaining uses:

```bash
grep -rn "inc_counter!" crates/ 2>&1 | head -10
```

If no uses (expected — it wasn't wired in), the old macro is gone in the rewrite above. If uses found, replace them with `inc_dispatch_counter!`.

- [ ] **Step 4: Build**

```bash
cargo build --release -p lyng-vm 2>&1 | tail -5
cargo build --release -p lyng-vm --no-default-features 2>&1 | tail -5
```

Expected: clean build in both configurations. The macros must be no-op when the feature is off.

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/dsl/backend/aarch64/counters.rs
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.0 Task 3: three bank-specific counter macros

Replace single inc_counter! with inc_dispatch_counter!,
inc_slow_semantic_counter!, inc_slow_safepoint_counter! — each takes
an opcode-byte literal and emits 5-instruction increment against the
appropriate DispatchCounters bank.

Feature-off path: empty strings (zero cost). Feature-on path:
double-indirection (Vm.dispatch_counters → Box<DispatchCounters> → bank[op])
then the standard ldr/add/str increment.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Wire `inc_dispatch_counter!` into the proc-macro lowerer

**Files:**
- Modify: [`crates/vm-dsl/src/lower.rs`](../../../crates/vm-dsl/src/lower.rs)
- Modify: [`crates/vm-dsl/src/parse.rs`](../../../crates/vm-dsl/src/parse.rs) (possibly — if the parsed handler signature needs to expose the opcode byte)

This is the most involved change in Phase 1.B.0. The lowerer must map each handler symbol (e.g., `op_load_undefined_dsl`) to its `Opcode` discriminant byte (e.g., the LoadUndefined variant) and emit `inc_dispatch_counter!(BYTE)` at the start of every handler.

- [ ] **Step 1: Read the lowerer's symbol-handling code**

```bash
cat crates/vm-dsl/src/lower.rs | head -150
```

Identify: where the handler's symbol name (the `op_xxx_dsl` ident) is consumed. Look for `quote::format_ident!` or similar that builds the generated `fn op_xxx_dsl()` definition.

- [ ] **Step 2: Decide where the opcode-byte mapping lives**

The lowerer needs to compute the Opcode discriminant byte for a given handler name. Options:

**(a)** Hard-code the mapping in the lowerer (`match handler_name { "op_load_undefined_dsl" => 5_u8, ... }`) — brittle, requires updating the lowerer for every new opcode.

**(b)** Use the `crate::DSL_DISPATCH_TABLE` manifest from `crates/vm/src/dsl/opcode_manifest.rs` to map handler-symbol → Opcode at COMPILE TIME via a build script.

**(c)** Make the user supply the opcode byte in the `llint_handler!` invocation: `llint_handler! { op_load_undefined_dsl, opcode = 5, layout = Abx, ... }`. Lowerer reads the opcode arg.

Option (c) is cleanest and least invasive. Each `llint_handler!` callsite already knows which opcode it implements. The Opcode enum variant is in scope at the callsite, so we can pass it as `opcode = Opcode::LoadUndefined as u8`.

But wait — proc-macros don't evaluate Rust expressions; they only see token trees. `Opcode::LoadUndefined as u8` is a token tree the lowerer can't reduce to a byte literal. We need either:
- A const-eval-friendly form, OR
- The user passes a literal: `opcode = 5_u8`, OR
- The lowerer emits a `const` expression that the generated code resolves.

Actually the cleanest: the lowerer emits `inc_dispatch_counter!(OPCODE_BYTE)` where `OPCODE_BYTE` is a `const u8` resolved at compile time:

```rust
#[unsafe(naked)]
pub extern "C" fn op_load_undefined_dsl() -> ! {
    const OPCODE_BYTE: u8 = lyng_bytecode::Opcode::LoadUndefined as u8;
    ::core::arch::naked_asm!(
        inc_dispatch_counter!(OPCODE_BYTE),  // <-- BUT this is a $literal! macro
        // ... rest of body ...
    )
}
```

Problem: `inc_dispatch_counter!($opcode_byte:literal)` requires a literal. `OPCODE_BYTE` is a const, not a literal — `macro_rules!` doesn't see through const.

Workarounds:
- Change `inc_dispatch_counter!` to take an `$opcode_byte:expr` and use `naked_asm!`'s `const N` binding form: `inc_dispatch_counter!(BYTE) => "ldr x10, [x9, {opcode_byte_offset}]"` with binding `opcode_byte_offset = const (OPCODE_BYTE * 8)`. But this still requires the const to be resolvable at proc-macro expand time — which it ISN'T.
- Have the LOWERER compute the byte and substitute it as a literal. The lowerer has access to the handler-name token; it can map `"op_load_undefined_dsl"` → string → discriminant via a hard-coded match.
- Use a build script to generate a `const` lookup table the lowerer can consult.

Decision: go with option (a) — hard-coded match in the lowerer. The mapping is small (152 entries), update-on-add isn't a frequent operation, and it gives the cleanest emission. Document that the lowerer must be updated when a new Opcode variant is added.

Alternative if (a) is too brittle: introduce a new lowerer arg `opcode_byte = N_LITERAL` that the user supplies explicitly in `llint_handler!`:

```rust
llint_handler! {
    op_load_undefined_dsl, opcode_byte = 5, layout = Abx, length = 4, |a, _bx| { ... }
}
```

The user passes the literal byte; lowerer trusts it; a compile-time assert in the generated code verifies it matches `Opcode::LoadUndefined as u8`:

```rust
const _: () = assert!(5_u8 == lyng_bytecode::Opcode::LoadUndefined as u8);
```

This is option (c-revised) with safety check. Most flexible; lowerer doesn't need a hard-coded mapping.

**Recommend option (c-revised):** add `opcode_byte = N` to `llint_handler!` signature; lowerer uses the literal directly. The compile-time assert in the generated code keeps the user honest (renaming an Opcode variant would break the assert).

- [ ] **Step 3: Update the parser to accept `opcode_byte = N`**

In `crates/vm-dsl/src/parse.rs`, find the handler-signature parsing (likely a struct like `ParsedHandler` or similar). Add an `opcode_byte: u8` field. Parse it from `opcode_byte = <LITERAL>` in the signature.

```rust
// Existing field set probably has: name, layout, length, args, body.
// Add: opcode_byte (u8 literal).
```

If the signature parsing uses `syn::parse_macro_input!`, extend the parsing logic to recognize the new keyword.

- [ ] **Step 4: Update the lowerer to emit `inc_dispatch_counter!` at handler entry**

In `lower.rs`, find where the body's token stream is built. BEFORE the operand-decode prologue, prepend the counter increment.

Roughly:

```rust
// In the asm template construction:
let counter_increment = quote! {
    inc_dispatch_counter!(#opcode_byte_literal),
};

// Insert at the start of the body tokens, before the decode prologue.
```

Also add a binding for `vm_counter_base`:

```rust
// In the named-arg bindings:
let vm_counter_base_binding = quote! {
    vm_counter_base = const crate::dsl::reg_convention::VM_DISPATCH_COUNTERS_PTR_OFFSET,
};
```

Add to the standard bindings list (alongside `length`, `state_pc`, etc.).

- [ ] **Step 5: Emit the safety assert**

In the generated handler body (before `naked_asm!`), emit:

```rust
const _: () = {
    use lyng_bytecode::Opcode;
    // The handler name without `_dsl` suffix should match a Pascal-case
    // Opcode variant. The user-supplied opcode_byte must match.
    assert!(<opcode_byte_literal>_u8 == /* expected */);
};
```

Actually the assert needs the user's expected opcode variant for comparison. Simplest:

```rust
// User provides Opcode variant explicitly too:
llint_handler! {
    op_load_undefined_dsl,
    opcode = Opcode::LoadUndefined,
    layout = Abx,
    length = 4,
    |a, _bx| { ... }
}
```

The lowerer reads `opcode = <Path>`, emits `Opcode::LoadUndefined as u8` as the byte literal via:

```rust
const OPCODE_BYTE: u8 = #opcode_path as u8;
```

Then `OPCODE_BYTE` is a const that the proc-macro... still can't substitute as a literal. We're back to the same problem.

**Hmm. Resolution:** keep the lowerer simple and require BOTH `opcode = Opcode::X` (for the const) AND `opcode_byte = N` (for the macro substitution):

```rust
llint_handler! {
    op_load_undefined_dsl,
    opcode = Opcode::LoadUndefined,
    opcode_byte = 5,                    // must match Opcode::LoadUndefined as u8
    layout = Abx,
    length = 4,
    |a, _bx| { ... }
}
```

The lowerer:
- Uses `opcode_byte` as a literal in `inc_dispatch_counter!(5)`.
- Emits `const _: () = assert!(5_u8 == Opcode::LoadUndefined as u8);` for safety.

Verbose at the call site but correct. Worth it.

Actually this is getting too prescriptive. **The plan should let the subagent decide between options (a), (b), and (c-revised) based on what they discover in the lowerer's current architecture.** I'll soften this step to "subagent investigates and picks the lowest-friction approach; if option (c) requires verbose-callsite changes, document the choice in the commit message".

Reframing this step:

**Step 4 (revised):** Investigate the lowerer's symbol-handling. Decide on the opcode-byte resolution strategy based on what's available:

- If `crate::DSL_DISPATCH_TABLE` is available at proc-macro time (via a build script or similar), use option (b): look up the byte from the manifest.
- If the lowerer can call `lyng_bytecode` at proc-macro time (rare — proc-macros can't depend on the same crate they're emitting for, due to dependency cycles), use direct lookup.
- Otherwise use option (c-revised): require the user to pass `opcode_byte = N` as a literal in `llint_handler!`. Add a compile-time assert in the generated code that verifies the byte matches the expected `Opcode` variant.

Whatever approach the subagent picks, the emission goal is the same: every generated handler begins with `inc_dispatch_counter!(OPCODE_BYTE_LITERAL)` before the decode prologue, gated by `#[cfg(feature = "diagnostic-counters")]` at the macro expansion level.

- [ ] **Step 5 (continued): Update all 152 `llint_handler!` callsites if needed**

If the subagent chose option (c-revised), every existing `llint_handler!` callsite in [`crates/vm/src/dsl/handlers/`](../../../crates/vm/src/dsl/handlers/) (hot.rs, warm.rs, cold.rs) needs the new `opcode_byte = N` parameter added.

Use `grep -n "llint_handler!" crates/vm/src/dsl/handlers/*.rs` to find them all.

For each, look up the corresponding `Opcode::X as u8` value (from `bytecode/src/lib.rs`'s `Opcode` enum, or by inspecting the dispatch table) and add the literal.

This is ~152 mechanical edits. The subagent can scripts this with sed if the pattern is consistent.

- [ ] **Step 6: Build**

```bash
cargo build --release -p lyng-vm 2>&1 | tail -20
```

Expected: clean build. If the const-assert fires for any handler, the user-supplied `opcode_byte` doesn't match the `Opcode` variant — fix the callsite.

- [ ] **Step 7: Run behavioral tests**

```bash
cargo test -p lyng-vm --lib --release 2>&1 | tail -3
cargo test -p lyng-tests --release 2>&1 | tail -3
```

Expected: 413 + 1186 passing (or more if the dispatch_counters_layout test runs in the lib suite). Failing tests would indicate the counter wiring corrupts asm state — investigate immediately.

- [ ] **Step 8: Run a Richards bench to verify counter correctness**

```bash
cargo run --release -p lyng-bench -- v8suite --samples 1 --count-opcodes \
  --counts-json /tmp/post-task4-counter.json 2>&1 | tail -10
```

Inspect `/tmp/post-task4-counter.json`. The Move opcode count should be ~4.66B (matches the pre-DSL-0c counter behavior). If counts are all-zero, the counter wiring isn't firing — investigate (likely the `inc_dispatch_counter!` macro isn't being expanded in the generated handler).

- [ ] **Step 9: Commit**

```bash
git add \
  crates/vm-dsl/src/lower.rs \
  crates/vm-dsl/src/parse.rs \
  crates/vm/src/dsl/handlers/hot.rs \
  crates/vm/src/dsl/handlers/warm.rs \
  crates/vm/src/dsl/handlers/cold.rs
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.0 Task 4: wire inc_dispatch_counter! into lowerer

Every llint_handler! generates inc_dispatch_counter!(OPCODE_BYTE) at
the start of its naked_asm! body when --features diagnostic-counters is on.
Opcode byte is supplied per-callsite via opcode_byte = N parameter
(or whichever resolution strategy the lowerer adopted); a compile-time
assert keeps the literal honest against the Opcode enum variant.

Per-handler counters now produce non-zero output on a Richards run
(Move ~4.66B). Slow-path counters (semantic + safepoint) wire in
Task 5.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Wire slow-path counters into call_slow! and poll_safepoint!

**Files:**
- Modify: [`crates/vm/src/dsl/backend/aarch64/control.rs`](../../../crates/vm/src/dsl/backend/aarch64/control.rs) (`call_slow!` macros)
- Modify: [`crates/vm/src/dsl/backend/aarch64/safepoint.rs`](../../../crates/vm/src/dsl/backend/aarch64/safepoint.rs) (`poll_safepoint!` macro)
- Modify: [`crates/vm-dsl/src/lower.rs`](../../../crates/vm-dsl/src/lower.rs) (pass `opcode_byte` as a binding so the macros can reference it)

- [ ] **Step 1: Pass `opcode_byte` as a named binding**

In `lower.rs`, the existing bindings list includes `length`, `state_pc`, etc. Add:

```rust
opcode_byte = const #opcode_byte_literal,
```

So the per-handler `naked_asm!` has access to `{opcode_byte}` as a literal that the slow-path macros can use.

- [ ] **Step 2: Update `call_slow!` macros to emit slow-semantic counter increment**

In `crates/vm/src/dsl/backend/aarch64/control.rs`, modify the `call_slow!` macros (each arity variant). Before the `bl {<shim>}` instruction, insert `inc_slow_semantic_counter!({opcode_byte})`.

Concretely, for the 0-arg variant (lines 97-106):

```rust
macro_rules! call_slow {
    ($shim:ident, args = []) => {
        concat!(
            // Bump slow-path-semantic counter for THIS opcode.
            $crate::inc_slow_semantic_counter!({opcode_byte}),
            "ldr    x16, [x24, {state_pb}]\n",
            "sub    x17, x19, x16\n",
            "str    w17, [x24, {state_pc}]\n",
            "mov    x0, x24\n",
            "bl     {", stringify!($shim), "}\n",
        )
    };
    // ... repeat for args = [a], [a, b], ..., [a, b, c, d, e]
}
```

Hmm wait — `inc_slow_semantic_counter!` takes an `$opcode_byte:literal`, not `{opcode_byte}`. The macro produces an asm-string fragment that uses `{opcode_byte}` as a `naked_asm!` binding. So the wiring is:

```rust
// inc_slow_semantic_counter!({opcode_byte}) produces a string like:
// "ldr    x9, [x22, {vm_counter_base}]\nldr x9, [x9]\nldr x10, [x9, #{opcode_byte} * 8 + 2048]\n..."
```

But that's using `{opcode_byte}` as a placeholder that `naked_asm!` will resolve via the `opcode_byte = const ...` binding from Step 1.

So the rewritten macro signature should be:

```rust
#[cfg(feature = "diagnostic-counters")]
#[macro_export]
macro_rules! inc_slow_semantic_counter {
    ({$binding:ident}) => {  // accepts a binding-name token
        concat!(
            "ldr    x9, [x22, {vm_counter_base}]\n",
            "ldr    x9, [x9]\n",
            "ldr    x10, [x9, {", stringify!($binding), "}, lsl #3]\n",
            // ... etc
        )
    };
}
```

This is getting complex. The cleanest path is to let the LOWERER substitute the byte directly into the call_slow! / poll_safepoint! invocation:

In the lowerer, when it sees `call_slow!(shim, args = [...])` in the DSL body, it knows the current handler's opcode byte. It can rewrite to `call_slow!(shim, args = [...], opcode = <BYTE_LITERAL>)`, and the macro signature becomes:

```rust
macro_rules! call_slow {
    ($shim:ident, args = [], opcode = $byte:literal) => {
        concat!(
            $crate::inc_slow_semantic_counter!($byte),
            // ... rest of original body
        )
    };
    // ... repeat for arities
}
```

But this requires the lowerer to transform `call_slow!` invocations in the DSL body. That's another substitution pass.

**Recommend:** simpler approach — the lowerer adds the opcode byte as the FIRST parameter to call_slow! / poll_safepoint! / inc_*_counter! calls in the DSL body, via a token-stream walk pass. Look at the lowerer's existing `substitute_idents` function for the pattern.

If this is too involved, the alternative is to drop slow-path-share-counter wiring from this task and ship dispatch-counter only. Slow-path counters become a follow-up.

- [ ] **Step 3: If slow-path counter wiring proves too complex, document the deferral**

If the subagent finds the lowerer transformation is non-trivial (>1 day work) for the slow-path counters, write `reports/lyng/dsl-1/phase-1b0-slow-counter-deferred.md` documenting the issue and recommending it be addressed in a focused refactor before Phase 1.C.

The dispatch counter alone (Task 4) is sufficient to enable per-opcode dispatch-share measurement, which is the most-needed gate. Slow-path-share enforcement can wait if needed.

- [ ] **Step 4: If wiring works, run a Richards bench with slow-path-share counting**

```bash
cargo run --release -p lyng-bench --features lyng-vm/diagnostic-counters -- v8suite \
  --samples 1 --count-opcodes --count-slow-path-share \
  --counts-json /tmp/post-task5-slow-counter.json 2>&1 | tail -10
```

Inspect the output. For each ported opcode (op_move, op_add, op_load_undefined, etc. — all 12 from DSL-0 + 7 from Phase 1.A), the slow-path-semantic and slow-path-safepoint counts should be 0 (those opcodes have no slow-path bridge in their inline path). For cold-stub opcodes, slow-semantic should be roughly equal to dispatch count (every dispatch goes through call_slow).

If counts look sensible, commit.

- [ ] **Step 5: Commit**

```bash
git add \
  crates/vm/src/dsl/backend/aarch64/control.rs \
  crates/vm/src/dsl/backend/aarch64/safepoint.rs \
  crates/vm-dsl/src/lower.rs
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.0 Task 5: wire slow-path counters into call_slow! / poll_safepoint!

Each call_slow! invocation increments the slow_semantic bank for the
current opcode; each poll_safepoint! pending branch increments the
slow_safepoint bank. Lowerer substitutes the opcode byte into the
macro calls at lower time.

slow-path-share data is now available on V8 v7 runs. The < 20%
slow-path-share invariant in DSL-1 is now enforceable.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Measure counter overhead + commit findings

**Files:**
- Create: `reports/lyng/dsl-1/phase-1b0-counter-overhead.md`

- [ ] **Step 1: Run Richards with and without `--features diagnostic-counters`**

```bash
# Without counters (default; lyng-bench currently enables them by default — need to override)
cargo run --release -p lyng-bench --no-default-features -- v8suite --samples 7 --json /tmp/no-counters.json 2>&1 | tail -5
# With counters
cargo run --release -p lyng-bench -- v8suite --samples 7 --json /tmp/with-counters.json 2>&1 | tail -5
```

Note: `lyng-bench`'s Cargo.toml enables `lyng-vm/diagnostic-counters` by default. To get the no-counter measurement, may need to add a `--no-default-features` flag OR build a separate binary with counters disabled. If the bench tool doesn't easily support this, instead use:

```bash
# Build lyng-vm without counters
cargo build --release -p lyng-vm --no-default-features

# Then bench (the lyng-bench Cargo features may need adjusting)
```

This is a measurement detail the subagent will work through.

- [ ] **Step 2: Compute overhead percentage**

For each workload, compare median scores between with-counters and without-counters. The overhead = (no-counters - with-counters) / no-counters × 100%.

Target: ≤5% (per parent §13.12 open question).

- [ ] **Step 3: Write the overhead report**

Create `reports/lyng/dsl-1/phase-1b0-counter-overhead.md`:

```markdown
# Phase 1.B.0 — opcode-counter overhead measurement

Measured 2026-MM-DD with counters wired into the DSL `dispatch!` tail
(Task 4) and slow-path bridges (Task 5).

## V8 v7 with vs without `--features diagnostic-counters`

| Workload    | Without (median) | With (median) | Overhead |
|-------------|-----------------:|--------------:|---------:|
| Richards    | <num>            | <num>         | <pct>%   |
| DeltaBlue   | <num>            | <num>         | <pct>%   |
| ...         | ...              | ...           | ...      |

(Fill from /tmp/{no,with}-counters.json — real numbers.)

## Verdict

- Target: ≤5% per parent §13.12.
- Observed: <pct>%.
- **Result: <within / exceeds> budget.**

If overhead exceeds 5%, consider:
- Sparse counter strategy (increment on every Nth dispatch).
- Per-thread counter caching with batched commit.
- Counter-disable-at-runtime via a feature flag check.
```

- [ ] **Step 4: Commit**

```bash
git add reports/lyng/dsl-1/phase-1b0-counter-overhead.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.0 Task 6: measure counter overhead

V8 v7 with vs without --features diagnostic-counters; per-workload
overhead computed. Target ≤ 5% per parent §13.12.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Add microbench snippets for 7 Phase-1.A opcodes

**Files:**
- Modify: [`tools/lyng-bench/src/microbench/snippets.rs`](../../../tools/lyng-bench/src/microbench/snippets.rs)

- [ ] **Step 1: Read the existing snippet pattern**

Inspect `tools/lyng-bench/src/microbench/snippets.rs`. Note:
- `Snippet` struct shape (`opcode`, `source`, `opcodes_per_iter`).
- The `all_snippets()` function builds a HashMap of opcode-name → Snippet.
- Existing snippets for Move (4 ops/iter), Add (1 op/iter), GetNamedProperty (3 ops/iter), Jump (probably 1 op/iter).

- [ ] **Step 2: Add snippets for the 7 Phase-1.A opcodes**

Append after the existing snippets in `all_snippets()`:

```rust
// LoadUndefined: assign `undefined` to a local in a tight loop.
map.insert("LoadUndefined", Snippet {
    opcode: "LoadUndefined",
    source: r"
        function bench(iters) {
            let x;
            for (let i = 0; i < iters; i++) {
                let a = undefined;
                let b = undefined;
                let c = undefined;
                let d = undefined;
                x = d;
            }
            return x;
        }
    ",
    opcodes_per_iter: 4,
});

// LoadNull: same pattern with null.
map.insert("LoadNull", Snippet {
    opcode: "LoadNull",
    source: r"
        function bench(iters) {
            let x;
            for (let i = 0; i < iters; i++) {
                let a = null;
                let b = null;
                let c = null;
                let d = null;
                x = d;
            }
            return x;
        }
    ",
    opcodes_per_iter: 4,
});

// LoadTrue
map.insert("LoadTrue", Snippet {
    opcode: "LoadTrue",
    source: r"
        function bench(iters) {
            let x;
            for (let i = 0; i < iters; i++) {
                let a = true;
                let b = true;
                let c = true;
                let d = true;
                x = d;
            }
            return x;
        }
    ",
    opcodes_per_iter: 4,
});

// LoadFalse
map.insert("LoadFalse", Snippet {
    opcode: "LoadFalse",
    source: r"
        function bench(iters) {
            let x;
            for (let i = 0; i < iters; i++) {
                let a = false;
                let b = false;
                let c = false;
                let d = false;
                x = d;
            }
            return x;
        }
    ",
    opcodes_per_iter: 4,
});

// LoadZero: assign 0 (note: compiler may emit LoadZero specifically for 0).
map.insert("LoadZero", Snippet {
    opcode: "LoadZero",
    source: r"
        function bench(iters) {
            let x;
            for (let i = 0; i < iters; i++) {
                let a = 0;
                let b = 0;
                let c = 0;
                let d = 0;
                x = d;
            }
            return x;
        }
    ",
    opcodes_per_iter: 4,
});

// LoadOne: same with 1.
map.insert("LoadOne", Snippet {
    opcode: "LoadOne",
    source: r"
        function bench(iters) {
            let x;
            for (let i = 0; i < iters; i++) {
                let a = 1;
                let b = 1;
                let c = 1;
                let d = 1;
                x = d;
            }
            return x;
        }
    ",
    opcodes_per_iter: 4,
});

// LoadSmi8: signed 8-bit int. Use a range that fits in i8 (-128..127).
map.insert("LoadSmi8", Snippet {
    opcode: "LoadSmi8",
    source: r"
        function bench(iters) {
            let x;
            for (let i = 0; i < iters; i++) {
                let a = 42;
                let b = -7;
                let c = 100;
                let d = -42;
                x = d;
            }
            return x;
        }
    ",
    opcodes_per_iter: 4,
});
```

**CRITICAL:** the `opcodes_per_iter` count must be verified by running the snippet through the dispatch counter (use `cargo run -p lyng-bench -- runtime --count-opcodes` per the file's docstring) and confirming the target opcode is dispatched the expected number of times per loop iteration. The compiler may emit different opcodes than expected (e.g., LoadZero for `0`, or constant-pool LoadConst for some literals). Adjust the source to actually exercise the target opcode if needed.

- [ ] **Step 3: Build + verify snippets**

```bash
cargo build --release -p lyng-bench 2>&1 | tail -3
```

Expected: clean build.

- [ ] **Step 4: Run microbench against the new snippets**

```bash
cargo run --release -p lyng-bench -- microbench \
  --opcodes-config tools/lyng-bench/hot-opcodes.toml \
  --samples 7 --output /tmp/phase-1a-microbench.md 2>&1 | tail -20
```

Inspect output. Each of the 7 new snippets should produce a ns/dispatch with CI95. No "no snippet" entries for these 7.

If `opcodes_per_iter` is wrong (e.g., the compiler emitted only 3 LoadUndefined dispatches per iter instead of 4), the ns/dispatch will be wrong. Fix the count and re-run.

- [ ] **Step 5: Commit**

```bash
git add tools/lyng-bench/src/microbench/snippets.rs
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.0 Task 7: microbench snippets for 7 Phase-1.A opcodes

Adds Snippet entries for LoadUndefined, LoadNull, LoadTrue, LoadFalse,
LoadZero, LoadOne, LoadSmi8. Each is a tight-loop JS program with 4
opcodes per iteration. opcodes_per_iter verified via dispatch counter
(now wired in Task 4).

Backfills the microbench gap noted in Phase 1.A's summary. The 7
Phase-1.A inline ports can now be microbenched.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: Add microbench snippets for 7 Phase-1.B anchor opcodes

**Files:**
- Modify: [`tools/lyng-bench/src/microbench/snippets.rs`](../../../tools/lyng-bench/src/microbench/snippets.rs)

- [ ] **Step 1: Append the Phase-1.B anchor snippets**

After the Phase-1.A snippets added in Task 7, add:

```rust
// LoadLocal0..3: load from a specific local slot. The compiler emits
// LoadLocalN for slot N when N ≤ 3.
map.insert("LoadLocal0", Snippet {
    opcode: "LoadLocal0",
    source: r"
        function bench(iters) {
            let s = 0;
            for (let i = 0; i < iters; i++) {
                s = s + s + s + s;  // 4 LoadLocal0 reads
            }
            return s;
        }
    ",
    opcodes_per_iter: 4,
});

map.insert("LoadLocal1", Snippet {
    opcode: "LoadLocal1",
    source: r"
        function bench(iters) {
            let pad = 0;  // pad slot 0
            let s = 0;
            for (let i = 0; i < iters; i++) {
                s = s + s + s + s;  // 4 LoadLocal1 reads (s is now slot 1)
            }
            return s;
        }
    ",
    opcodes_per_iter: 4,
});

map.insert("LoadLocal2", Snippet {
    opcode: "LoadLocal2",
    source: r"
        function bench(iters) {
            let pad0 = 0;
            let pad1 = 0;
            let s = 0;
            for (let i = 0; i < iters; i++) {
                s = s + s + s + s;  // 4 LoadLocal2 reads
            }
            return s;
        }
    ",
    opcodes_per_iter: 4,
});

map.insert("LoadLocal3", Snippet {
    opcode: "LoadLocal3",
    source: r"
        function bench(iters) {
            let pad0 = 0;
            let pad1 = 0;
            let pad2 = 0;
            let s = 0;
            for (let i = 0; i < iters; i++) {
                s = s + s + s + s;  // 4 LoadLocal3 reads
            }
            return s;
        }
    ",
    opcodes_per_iter: 4,
});

// StoreLocal3: assign to slot 3.
map.insert("StoreLocal3", Snippet {
    opcode: "StoreLocal3",
    source: r"
        function bench(iters) {
            let pad0 = 0;
            let pad1 = 0;
            let pad2 = 0;
            let s = 0;
            for (let i = 0; i < iters; i++) {
                s = i;
                s = i;
                s = i;
                s = i;  // 4 StoreLocal3 writes
            }
            return s;
        }
    ",
    opcodes_per_iter: 4,
});

// LoadEnvSlot: read from an environment slot (closure variable).
map.insert("LoadEnvSlot", Snippet {
    opcode: "LoadEnvSlot",
    source: r"
        function bench(iters) {
            let captured = 1;
            function inner() {
                let s = 0;
                for (let i = 0; i < iters; i++) {
                    s = captured + captured + captured + captured;
                }
                return s;
            }
            return inner();
        }
    ",
    opcodes_per_iter: 4,
});

// Ldar: load accumulator. Likely emitted by simple expression statements.
// May need workload-specific tuning to actually emit Ldar (vs Move).
map.insert("Ldar", Snippet {
    opcode: "Ldar",
    source: r"
        function bench(iters) {
            let s = 0;
            for (let i = 0; i < iters; i++) {
                s;
                s;
                s;
                s;
            }
            return s;
        }
    ",
    opcodes_per_iter: 4,
});
```

CRITICAL: As with Task 7, verify each `opcodes_per_iter` count via dispatch counter. The lyng compiler may emit different opcodes than expected — adjust source patterns until the target opcode is actually dispatched the expected number of times.

For `LoadLocal0/1/2/3`: the slot assignment depends on the compiler's local-slot allocation. Verify with the dispatch counter that the intended slot is being read.

For `Ldar`: the `s;` statement-expression pattern may compile to Move or just be elided. If Ldar isn't dispatched at all, the snippet is wrong — revise.

- [ ] **Step 2: Build + verify**

```bash
cargo build --release -p lyng-bench 2>&1 | tail -3
cargo run --release -p lyng-bench -- microbench --samples 7 --output /tmp/phase-1b-microbench.md 2>&1 | tail -20
```

Expected: all 14 new snippets (7 Phase-1.A + 7 Phase-1.B) produce ns/dispatch with CI95.

- [ ] **Step 3: Commit**

```bash
git add tools/lyng-bench/src/microbench/snippets.rs
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.0 Task 8: microbench snippets for 7 Phase-1.B anchors

Adds Snippet entries for LoadLocal0/1/2/3 (#11/8/18/9 in top-30),
StoreLocal3 (#22), LoadEnvSlot (#19), Ldar (#26). Each snippet is a
tight-loop JS program with verified opcodes_per_iter (via dispatch
counter from Task 4).

Phase 1.B.3 opcode ports will now have microbench measurements as
part of the per-opcode gate.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 9: Sub-phase 1.B.0 gate verification + summary

**Files:**
- Create: `reports/lyng/dsl-1/phase-1b0-summary.md`

- [ ] **Step 1: Run full counter sanity check**

```bash
cargo run --release -p lyng-bench -- v8suite \
  --samples 1 --count-opcodes --count-slow-path-share \
  --counts-json /tmp/phase-1b0-end-counter.json 2>&1 | tail -10
```

Inspect `/tmp/phase-1b0-end-counter.json`:
- `Move` dispatch count should be ~4.66B (matches top-30.tsv data).
- `Add` ~879M, `GetKeyedProperty` ~657M, etc.
- Counts within 5% of expected.
- For ported opcodes (Move, Add, op_load_undefined, etc.), slow_semantic count should be 0 (no slow-path bridge in inline path).
- For cold-stub opcodes, slow_semantic ≈ dispatch count.

- [ ] **Step 2: Run full microbench**

```bash
cargo run --release -p lyng-bench -- microbench --samples 7 --output /tmp/phase-1b0-end-microbench.md 2>&1 | tail -30
```

Verify: all 14 in-scope opcodes show ns/dispatch with CI95. No "no snippet" entries.

- [ ] **Step 3: Run full behavioral test suite**

```bash
cargo test -p lyng-vm --lib --release 2>&1 | tail -3
cargo test -p lyng-tests --release 2>&1 | tail -3
```

Expected: 413 + 1186 + new dispatch_counters_layout tests passing.

- [ ] **Step 4: Same-load A/B against pre-1.B.0 HEAD**

Per spec §4 protocol:

```bash
git stash --include-untracked
git checkout $(cat /tmp/phase-1b0-base-sha)
cargo build --release -p lyng-bench
cargo run --release -p lyng-bench -- v8suite --samples 7 --json /tmp/phase-1b0-ab-base.json
uptime > /tmp/phase-1b0-ab-base-uptime

git checkout claude/epic-saha-8f0b96
cargo build --release -p lyng-bench
cargo run --release -p lyng-bench -- v8suite --samples 7 --json /tmp/phase-1b0-ab-post.json
uptime > /tmp/phase-1b0-ab-post-uptime

# Verify uptime within ±20%; if not, re-run.
git stash pop
```

Compute per-workload deltas; the counter overhead (≤5%) should be the dominant signal. If a workload regresses >5%, the counter wiring has a performance bug — investigate.

- [ ] **Step 5: Write the phase summary**

Create `reports/lyng/dsl-1/phase-1b0-summary.md`:

```markdown
# DSL-1 Phase 1.B.0 — Infrastructure (summary)

**Duration:** 2026-MM-DD to 2026-MM-DD (~2-3 days single-dev).
**Range:** baseline commit (see /tmp/phase-1b0-base-sha) → HEAD <SHA>.
**Status:** Phase 1.B.0 closed; counter + microbench infra live.

## Scope landed

| Task | Deliverable | Commit |
|-----:|-------------|--------|
|  1   | `DispatchCounters` repr(C) struct + Vm field | <sha> |
|  2   | VM_DISPATCH_COUNTERS_PTR_OFFSET + bank consts | <sha> |
|  3   | Three bank-specific counter macros | <sha> |
|  4   | Wire `inc_dispatch_counter!` into lowerer | <sha> |
|  5   | Slow-path counters in call_slow! / poll_safepoint! | <sha> |
|  6   | Counter overhead measurement (target ≤ 5%) | <sha> |
|  7   | 7 Phase-1.A microbench snippets | <sha> |
|  8   | 7 Phase-1.B anchor microbench snippets | <sha> |

## Counter correctness

| Opcode             | Counter says | Expected (top-30) | Delta |
|--------------------|-------------:|------------------:|------:|
| Move               | <num>        | 4,665,497,587     | <pct>%|
| Add                | <num>        | 879,112,898       | <pct>%|
| GetKeyedProperty   | <num>        | 656,825,602       | <pct>%|
| ... (rest of top-30) ...

(Fill from /tmp/phase-1b0-end-counter.json. Deltas within ±5% are acceptable.)

## Per-feature overhead

See [`phase-1b0-counter-overhead.md`](phase-1b0-counter-overhead.md).
Result: <pct>% (target ≤ 5%).

## Microbench coverage

14 in-scope opcodes (7 Phase-1.A + 7 Phase-1.B anchors) all produce
ns/dispatch with CI95. No "no snippet" entries for in-scope opcodes.

## Same-load A/B vs pre-1.B.0

| Workload    | Pre-1.B.0 | Post-1.B.0 | Delta |
|-------------|----------:|-----------:|------:|
| Richards    | <num>     | <num>      | <pct>%|
| ...         | ...       | ...        | ...   |
| **Geomean** | <num>     | <num>      | **<pct>%** |

Per-workload tolerance: no regression > 5% (overhead budget). Result: <pass/fail>.

## Decision

✅ Counter infra: <pass/fail>
✅ Microbench infra: <pass/fail>
✅ Overhead within budget: <pass/fail>
✅ Behavioral parity: pass (413 + 1186)
✅ Same-load A/B regression ≤ 5%: <pass/fail>

**Phase 1.B.0 exit criteria met.** Phase 1.B.1 (frame-context refactor)
can proceed; per-opcode gates are now enforceable for the rest of DSL-1.
```

Fill placeholders with real data.

- [ ] **Step 6: Commit**

```bash
git add reports/lyng/dsl-1/phase-1b0-summary.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.0: phase summary

Counter wiring (10.A) and microbench snippets (10.B) infrastructure
landed. Per-opcode dispatch and slow-path-share gates are now
enforceable for the rest of DSL-1. Counter overhead <pct>% (target ≤ 5%).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Self-Review

After writing this plan I reviewed each spec section against the tasks:

- **Spec §1 scope** (10.A + 10.B in 1.B.0): covered by Tasks 1-6 (10.A) and Tasks 7-8 (10.B).
- **Spec §2 1.B.0 exit criterion** (counter records Move ≈ 4.66B; microbench produces CI95 for all 14 opcodes): covered by Task 9 Steps 1-2.
- **Spec §4 per-sub-phase gate** (5% overhead, no regression): covered by Tasks 6 and 9.
- **Spec §6 deliverables**: all listed as task outputs.

**Placeholder scan:** Found `2026-MM-DD` in two summary template placeholders (Task 6 step 3, Task 9 step 5) — these are template instructions for the worker to fill with the actual date; left as-is per the "real data must replace placeholders before commit" pattern.

**Type consistency:** `DispatchCounters` (Task 1), `OpcodeDispatchCounterStore` (existing), `dispatch_counters` (Vm field, Task 1+3), `VM_DISPATCH_COUNTERS_PTR_OFFSET` (Task 2) — names consistent across tasks.

**Architecture risk:** Task 4 (proc-macro lowerer change) is the highest-risk single task — the opcode-byte resolution strategy is non-trivial and may surface design questions the lowerer can't easily answer. The plan documents this with explicit "if too complex, defer" off-ramp in Task 5 Step 3.

No issues to fix inline.
