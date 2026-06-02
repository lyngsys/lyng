# SP-0b — Unified Register-File Frame Arena Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Collapse the two growable Vecs (`frames: Vec<FrameRecord>` and `register_stack: Vec<Value>`) into one pre-reserved, never-realloc bump arena where each frame is a `repr(C)` `offset_of!`-pinned header immediately followed by its value window; derive realm fully; add a soft-limit prologue check. No asm executes in this phase — behavior-preserving, Test262 100% per commit.

**Architecture:** A `FrameArena` wraps a fixed `Box<[Value]>` (never reallocated) plus a bump cursor. Each frame occupies `[FrameHeader: HEADER_SLOTS × Value-sized slots][value window: M slots]`; `cfr` is the slot offset of the header, `regs_base = cfr + HEADER_SLOTS`. The header is a `#[repr(C)]` POD overlaid on the arena slots via pointer cast — typed accessors convert the raw `u32`/`u16`/`u8` fields back to `CodeRef`/`ObjectRef`/`EnvironmentRef`/`ThisState`. The asm-hot cluster (slots 0–3) holds what a call prologue touches; the interpreter-warm cluster (slots 4–6) is read only by the interpreter; realm/referrer/executable/geometry are derived; the rare per-activation state (generator resume, tail-caller, handler cursor, parameter-initializer end) lives in a depth-indexed cold side-table. GC walks the `caller_cfr` chain, tracing each header's typed refs plus that frame's window as Values. Migration is bridge-first: the arena + header are stood up alongside the existing Vecs, readers migrate cluster by cluster, and the old fields are deleted last — every task is a commit that compiles and keeps Test262 at 100%.

**Tech Stack:** Rust (workspace crates `lyng_env`, `lyng_vm`, `lyng_types`, `lyng_gc`); test runner `cargo test` + the Test262 harness (`crates/test262-harness`); benches via the V8 suite (Richards/RayTrace).

**Spec:** `docs/superpowers/specs/2026-05-31-sp0b-unified-register-file-frame-arena-design.md`. Read it first.

**Conventions for every commit message in this plan:** end with
```
Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
```
Branch is `feat/sp0b-unified-register-file-frame-arena` (already created; the design spec is committed there). Do not merge to `main` until the whole plan is done and the perf gate (Task 20) passes.

---

## Representation decisions (locked — every task assumes these)

- **Arena slot type = `Value`.** `register_stack: Vec<Value>` becomes `FrameArena` backed by `Box<[Value]>`. Both header words and window slots are 64-bit; the window is `&[Value]` directly (hot register accessors unchanged), and the header is a `repr(C)` POD overlaid on `Value`-sized slots via pointer cast. `Value` is 8-byte aligned, so a header sized to a multiple of 8 bytes and aligned to 8 fits cleanly over `HEADER_SLOTS` slots.
- **`HEADER_SLOTS = 7`** (56-byte header). Layout in Task 2.
- **Capacity / limits:** `ARENA_CAPACITY_SLOTS = 512 * 1024` (4 MiB), `ARENA_SLACK_SLOTS = 4096` (32 KiB headroom so the `RangeError` throw can run), `ARENA_SOFT_LIMIT_SLOTS = ARENA_CAPACITY_SLOTS - ARENA_SLACK_SLOTS`. The existing `MAX_BYTECODE_CALL_DEPTH = 8_192` frame-count cap stays as a secondary guard.
- **`caller_cfr`** is a **slot offset** into the arena; sentinel `u32::MAX` = root/no caller.
- **Handle sentinels in the header:** `code` is a raw `u32` (always non-zero). `callee`/`new_target`/`private_env`/`construct_this` are raw `u32` with `0 = None` (matches `Option<NonZeroU32>`'s niche). `variable_env`/`lexical_env` are raw `u32` (always non-zero). `return_register: Option<u16>` is stored as a `u16` plus a `HAS_RETURN_REGISTER` flag bit. `this_state` is a `u8` tag (`0=Uninitialized, 1=Lexical, 2=Value`); when the tag is `Value`, the binding is the header's `this_value` slot.
- **Cold side-table** is keyed by `frame_depth` (already tracked in `LlIntState.frame_depth` and via `self.frames.len()` today), reset to default on push.

---

## File Structure

**New — `lyng_vm`:**
- `crates/vm/src/frame_arena.rs` — the `FrameArena` newtype (`Box<[Value]>` + cursor + soft-limit + `base()`/bump/release/`try_grow`). One responsibility: the never-realloc value/frame backing store.
- `crates/vm/src/frame_header.rs` — the `#[repr(C)]` `FrameHeader` POD + raw fields + typed accessors + the overlay read/write helpers + `frame_header_offsets_stable()` lock-in test.
- `crates/vm/src/frame_cold.rs` — the depth-indexed `FrameColdState` side-table (`handler_cursor`, `tail_caller`(+strict), generator resume, `parameter_initializer_end_offset`) + its GC trace.

**Modified — `lyng_vm`:**
- `crates/vm/src/frame.rs` — `FrameRecord` keeps its accessor surface during migration (the bridge); the realm field and the cold/derived fields are removed in the final tasks.
- `crates/vm/src/vm.rs` — replace `register_stack`/`register_stack_top` with `arena: FrameArena`; replace `frames: Vec<FrameRecord>` with `current_cfr: u32` + `frame_depth: u32`; the establishment side-stack gains a `realm`; frame push/unwind/`frame()`/`frame_mut()`; `refresh_running_context`; `realm_of`.
- `crates/vm/src/vm/registers.rs` — repoint `read_register`/`write_register`(`_unchecked`)/`finish_frame`/`reserve_register_window`/`release_register_window` onto the arena + header overlay.
- `crates/vm/src/vm/bytecode_calls.rs`, `vm/generators.rs`, `vm/jobs.rs`, `vm/internal_calls.rs` — frame construction writes the header overlay + cold slot; unwind walks cfr/depth; realm derivation.
- `crates/vm/src/vm/state.rs` — GC switches from the wholesale register trace + `frames` iteration to the per-frame arena walk.
- `crates/vm/src/dsl/entry.rs`, `dsl/llint_state.rs`, `dsl/reg_convention.rs` — `frame_regs_base` derives from `arena.base().add(regs_base)`; add header offset constants if asm pins any header field (it does not in SP-0b, but the constants are defined for SP-1).

**Modified — `lyng_env`:**
- `crates/env/src/execution.rs` — no change to `RunningContext`; realm continues to flow through it.

---

## Group 1 — Stand up the arena, header, and cold table (behind bridges)

## Task 1: Add the `FrameArena` newtype

**Files:**
- Create: `crates/vm/src/frame_arena.rs`
- Modify: `crates/vm/src/lib.rs` (add `mod frame_arena;` + re-export `FrameArena` if other modules need it — match how `frame` is declared)
- Test: `crates/vm/src/frame_arena.rs` (`#[cfg(test)]` module)

- [ ] **Step 1: Write the failing test**

Create `crates/vm/src/frame_arena.rs` with the test first:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use lyng_types::Value;

    #[test]
    fn bump_and_release_move_the_cursor_without_realloc() {
        let mut arena = FrameArena::new();
        let base_ptr = arena.base_ptr();
        assert_eq!(arena.top(), 0);

        let cfr = arena.bump(7 + 3).expect("space for one frame");
        assert_eq!(cfr, 0);
        assert_eq!(arena.top(), 10);

        // Writing through the window slice round-trips.
        arena.slots_mut()[7] = Value::from_smi(99);
        assert_eq!(arena.slots()[7], Value::from_smi(99));

        arena.release_to(cfr);
        assert_eq!(arena.top(), 0);
        // Never reallocated: the backing pointer is stable across bump/release.
        assert_eq!(arena.base_ptr(), base_ptr);
    }

    #[test]
    fn bump_past_soft_limit_returns_none() {
        let mut arena = FrameArena::new();
        // One frame just under the soft limit succeeds…
        assert!(arena.bump(ARENA_SOFT_LIMIT_SLOTS - 1).is_some());
        // …and the next slot crosses it.
        assert!(arena.bump(2).is_none());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p lyng_vm --lib frame_arena::tests`
Expected: FAIL — `cannot find type FrameArena`.

- [ ] **Step 3: Implement `FrameArena`**

Above the test module in `crates/vm/src/frame_arena.rs`:
```rust
use lyng_types::Value;

/// Total never-realloc value/frame backing capacity, in 64-bit slots (4 MiB).
pub const ARENA_CAPACITY_SLOTS: usize = 512 * 1024;
/// Headroom reserved above the soft limit so the RangeError throw path — which
/// itself needs a frame + window — runs inside the reservation.
pub const ARENA_SLACK_SLOTS: usize = 4096;
/// Frame pushes are rejected at or above this; the slack remains for the throw.
pub const ARENA_SOFT_LIMIT_SLOTS: usize = ARENA_CAPACITY_SLOTS - ARENA_SLACK_SLOTS;

/// The single pre-reserved, never-reallocated value/frame stack. Frames
/// bump-allocate `[header][window]` runs from the base; the backing `Box<[Value]>`
/// is allocated once and never moves, so a pointer into it stays valid across
/// every push — the safety property the future asm call path relies on.
///
/// This is the storage half of the interface the deferred lazy-commit backing will
/// implement: `base_ptr`, `bump`/`release_to`, `top`, and the soft-limit check.
/// `try_grow` is a no-op here (the eager `Box` cannot grow); a lazy-commit backing
/// would commit more pages instead.
pub struct FrameArena {
    slots: Box<[Value]>,
    top: usize,
}

impl FrameArena {
    #[inline]
    pub fn new() -> Self {
        Self {
            slots: vec![Value::undefined(); ARENA_CAPACITY_SLOTS].into_boxed_slice(),
            top: 0,
        }
    }

    /// Stable base pointer of the backing store (never changes for the arena's life).
    #[inline]
    pub fn base_ptr(&self) -> *const Value {
        self.slots.as_ptr()
    }

    #[inline]
    pub fn base_mut_ptr(&mut self) -> *mut Value {
        self.slots.as_mut_ptr()
    }

    #[inline]
    pub fn top(&self) -> usize {
        self.top
    }

    #[inline]
    pub fn slots(&self) -> &[Value] {
        &self.slots
    }

    #[inline]
    pub fn slots_mut(&mut self) -> &mut [Value] {
        &mut self.slots
    }

    /// Reserve `slot_count` contiguous slots at the current top, zero-filled by
    /// construction. Returns the base slot offset (the new frame's `cfr`), or
    /// `None` if it would cross the soft limit (caller throws `RangeError`).
    #[inline]
    pub fn bump(&mut self, slot_count: usize) -> Option<u32> {
        let base = self.top;
        let end = base.checked_add(slot_count)?;
        if end >= ARENA_SOFT_LIMIT_SLOTS {
            return None;
        }
        self.top = end;
        u32::try_from(base).ok()
    }

    /// Release every slot at or above `slot_offset`. Slots are left as-is (stale
    /// values are overwritten on the next bump's caller, exactly like the old
    /// register-stack cursor); only the cursor moves.
    #[inline]
    pub fn release_to(&mut self, slot_offset: u32) {
        self.top = slot_offset as usize;
    }

    /// Eager backing cannot grow; the lazy-commit backing will override this.
    #[inline]
    pub fn try_grow(&mut self, _needed_top: usize) -> bool {
        false
    }
}

impl Default for FrameArena {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
```
Add `mod frame_arena;` to `crates/vm/src/lib.rs` next to the other `mod` lines (e.g. `mod frame;`), and re-export the type if `vm.rs` needs it: `pub use frame_arena::{FrameArena, ARENA_CAPACITY_SLOTS, ARENA_SOFT_LIMIT_SLOTS};` (match the existing re-export style).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p lyng_vm --lib frame_arena::tests`
Expected: PASS (both).

- [ ] **Step 5: Build the workspace**

Run: `cargo build -p lyng_vm`
Expected: success (the type is unused so far; allow `dead_code` if the linter complains, or leave — it is used in Task 4).

- [ ] **Step 6: Commit**

```bash
git add crates/vm/src/frame_arena.rs crates/vm/src/lib.rs
git commit -m "feat(vm): add FrameArena (pre-reserved never-realloc value/frame stack)

Box<[Value]> + bump cursor + soft-limit check behind a swap-ready interface
(base_ptr/bump/release_to/top/try_grow). Not yet wired in.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: Add the `repr(C)` `FrameHeader` + offset lock-in test

**Files:**
- Create: `crates/vm/src/frame_header.rs`
- Modify: `crates/vm/src/lib.rs` (`mod frame_header;`)
- Test: `crates/vm/src/frame_header.rs` (`#[cfg(test)]`)

- [ ] **Step 1: Write the failing tests**

Create `crates/vm/src/frame_header.rs`; put the tests at the bottom:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::{align_of, offset_of, size_of};
    use lyng_env::{ExecutionContextKind, ThisState};
    use lyng_types::{CodeRef, EnvironmentRef, Value};
    use std::num::NonZeroU32;

    const fn id(raw: u32) -> NonZeroU32 {
        match NonZeroU32::new(raw) {
            Some(v) => v,
            None => panic!("non-zero"),
        }
    }

    #[test]
    fn frame_header_offsets_stable() {
        // Locks the asm-addressable header ABI; SP-1's prologue depends on it.
        assert_eq!(offset_of!(FrameHeader, caller_cfr), 0);
        assert_eq!(offset_of!(FrameHeader, saved_pc), 4);
        assert_eq!(offset_of!(FrameHeader, code), 8);
        assert_eq!(offset_of!(FrameHeader, callee), 12);
        assert_eq!(offset_of!(FrameHeader, this_value), 16);
        assert_eq!(offset_of!(FrameHeader, arg_count), 24);
        assert_eq!(offset_of!(FrameHeader, return_register), 26);
        assert_eq!(offset_of!(FrameHeader, flags), 28);
        assert_eq!(offset_of!(FrameHeader, this_state_tag), 29);
        assert_eq!(offset_of!(FrameHeader, kind), 30);
        assert_eq!(offset_of!(FrameHeader, variable_env), 32);
        assert_eq!(offset_of!(FrameHeader, lexical_env), 36);
        assert_eq!(offset_of!(FrameHeader, private_env), 40);
        assert_eq!(offset_of!(FrameHeader, new_target), 44);
        assert_eq!(offset_of!(FrameHeader, construct_this), 48);
        assert_eq!(size_of::<FrameHeader>(), HEADER_SLOTS * size_of::<Value>());
        assert_eq!(size_of::<FrameHeader>(), 56);
        assert_eq!(align_of::<FrameHeader>(), align_of::<Value>());
    }

    #[test]
    fn typed_accessors_round_trip() {
        let mut h = FrameHeader::zeroed();
        h.set_code(CodeRef::new(id(7)));
        h.set_callee(Some(lyng_types::ObjectRef::new(id(3))));
        h.set_callee(None); // 0 sentinel
        h.set_variable_env(EnvironmentRef::new(id(5)));
        h.set_this(ThisState::Value(Value::from_smi(11)), Value::from_smi(11));
        h.set_return_register(Some(4));
        assert_eq!(h.code(), CodeRef::new(id(7)));
        assert_eq!(h.callee(), None);
        assert_eq!(h.variable_env(), EnvironmentRef::new(id(5)));
        assert_eq!(h.this_state(), ThisState::Value(Value::from_smi(11)));
        assert_eq!(h.this_value(), Value::from_smi(11));
        assert_eq!(h.return_register(), Some(4));
        assert_eq!(FrameHeader::zeroed().return_register(), None);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p lyng_vm --lib frame_header::tests`
Expected: FAIL — `cannot find type FrameHeader`.

- [ ] **Step 3: Implement `FrameHeader`**

At the top of `crates/vm/src/frame_header.rs`:
```rust
use lyng_env::{ExecutionContextKind, ThisState};
use lyng_types::{CodeRef, EnvironmentRef, ObjectRef, Value};

/// Number of 64-bit slots the header occupies at the front of every frame.
pub const HEADER_SLOTS: usize = 7;

const ROOT_CFR: u32 = u32::MAX;

mod flag {
    pub const HAS_RETURN_REGISTER: u8 = 1 << 4; // bits 0..3 mirror FrameFlags
}

mod this_tag {
    pub const UNINITIALIZED: u8 = 0;
    pub const LEXICAL: u8 = 1;
    pub const VALUE: u8 = 2;
}

/// asm-addressable, GC-traced per-frame header. Overlaid as POD on the first
/// `HEADER_SLOTS` `Value`-sized slots of a frame in the [`crate::FrameArena`].
/// Field order is the ABI (locked by `frame_header_offsets_stable`); slots 0–3 are
/// the asm-hot cluster, 4–6 the interpreter-warm cluster. Raw integer fields keep
/// the struct POD; typed accessors convert. realm/referrer/executable/geometry are
/// NOT stored (derived elsewhere).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FrameHeader {
    // --- slot 0 ---
    caller_cfr: u32,
    saved_pc: u32,
    // --- slot 1 ---
    code: u32,
    callee: u32, // 0 = None
    // --- slot 2 ---
    this_value: Value,
    // --- slot 3 ---
    arg_count: u16,
    return_register: u16, // valid iff flags & HAS_RETURN_REGISTER
    flags: u8,
    this_state_tag: u8,
    kind: u8,
    _pad0: u8,
    // --- slot 4 ---
    variable_env: u32,
    lexical_env: u32,
    // --- slot 5 ---
    private_env: u32,    // 0 = None
    new_target: u32,     // 0 = None
    // --- slot 6 ---
    construct_this: u32, // 0 = None
    _pad1: u32,
}

impl FrameHeader {
    #[inline]
    pub const fn zeroed() -> Self {
        Self {
            caller_cfr: ROOT_CFR,
            saved_pc: 0,
            code: 0,
            callee: 0,
            this_value: Value::undefined(),
            arg_count: 0,
            return_register: 0,
            flags: 0,
            this_state_tag: this_tag::UNINITIALIZED,
            kind: 0,
            _pad0: 0,
            variable_env: 0,
            lexical_env: 0,
            private_env: 0,
            new_target: 0,
            construct_this: 0,
            _pad1: 0,
        }
    }

    // --- caller link / return state ---
    #[inline]
    pub fn caller_cfr(&self) -> Option<u32> {
        (self.caller_cfr != ROOT_CFR).then_some(self.caller_cfr)
    }
    #[inline]
    pub fn set_caller_cfr(&mut self, cfr: Option<u32>) {
        self.caller_cfr = cfr.unwrap_or(ROOT_CFR);
    }
    #[inline]
    pub fn saved_pc(&self) -> u32 {
        self.saved_pc
    }
    #[inline]
    pub fn set_saved_pc(&mut self, pc: u32) {
        self.saved_pc = pc;
    }
    #[inline]
    pub fn return_register(&self) -> Option<u16> {
        (self.flags & flag::HAS_RETURN_REGISTER != 0).then_some(self.return_register)
    }
    #[inline]
    pub fn set_return_register(&mut self, reg: Option<u16>) {
        match reg {
            Some(r) => {
                self.return_register = r;
                self.flags |= flag::HAS_RETURN_REGISTER;
            }
            None => {
                self.return_register = 0;
                self.flags &= !flag::HAS_RETURN_REGISTER;
            }
        }
    }

    // --- code / callee ---
    #[inline]
    pub fn code(&self) -> CodeRef {
        CodeRef::from_raw(self.code).expect("frame header code is always non-zero")
    }
    #[inline]
    pub fn set_code(&mut self, code: CodeRef) {
        self.code = code.raw();
    }
    #[inline]
    pub fn callee(&self) -> Option<ObjectRef> {
        ObjectRef::from_raw(self.callee)
    }
    #[inline]
    pub fn set_callee(&mut self, callee: Option<ObjectRef>) {
        self.callee = callee.map_or(0, ObjectRef::raw);
    }

    // --- this ---
    #[inline]
    pub fn this_value(&self) -> Value {
        self.this_value
    }
    #[inline]
    pub fn this_state(&self) -> ThisState {
        match self.this_state_tag {
            this_tag::LEXICAL => ThisState::Lexical,
            this_tag::VALUE => ThisState::Value(self.this_value),
            _ => ThisState::Uninitialized,
        }
    }
    #[inline]
    pub fn set_this(&mut self, this_state: ThisState, this_value: Value) {
        self.this_value = this_value;
        self.this_state_tag = match this_state {
            ThisState::Uninitialized => this_tag::UNINITIALIZED,
            ThisState::Lexical => this_tag::LEXICAL,
            ThisState::Value(_) => this_tag::VALUE,
        };
    }
    #[inline]
    pub fn set_this_value(&mut self, this_value: Value) {
        self.this_value = this_value;
        if self.this_state_tag == this_tag::VALUE {
            // keep the Value(v) binding consistent
            self.this_value = this_value;
        }
    }

    // --- envs / new_target / construct_this ---
    #[inline]
    pub fn variable_env(&self) -> EnvironmentRef {
        EnvironmentRef::from_raw(self.variable_env).expect("variable_env non-zero")
    }
    #[inline]
    pub fn set_variable_env(&mut self, env: EnvironmentRef) {
        self.variable_env = env.raw();
    }
    #[inline]
    pub fn lexical_env(&self) -> EnvironmentRef {
        EnvironmentRef::from_raw(self.lexical_env).expect("lexical_env non-zero")
    }
    #[inline]
    pub fn set_lexical_env(&mut self, env: EnvironmentRef) {
        self.lexical_env = env.raw();
    }
    #[inline]
    pub fn private_env(&self) -> Option<EnvironmentRef> {
        EnvironmentRef::from_raw(self.private_env)
    }
    #[inline]
    pub fn set_private_env(&mut self, env: Option<EnvironmentRef>) {
        self.private_env = env.map_or(0, EnvironmentRef::raw);
    }
    #[inline]
    pub fn new_target(&self) -> Option<ObjectRef> {
        ObjectRef::from_raw(self.new_target)
    }
    #[inline]
    pub fn set_new_target(&mut self, nt: Option<ObjectRef>) {
        self.new_target = nt.map_or(0, ObjectRef::raw);
    }
    #[inline]
    pub fn construct_this(&self) -> Option<ObjectRef> {
        ObjectRef::from_raw(self.construct_this)
    }
    #[inline]
    pub fn set_construct_this(&mut self, ct: Option<ObjectRef>) {
        self.construct_this = ct.map_or(0, ObjectRef::raw);
    }

    // --- small scalars ---
    #[inline]
    pub fn arg_count(&self) -> u16 {
        self.arg_count
    }
    #[inline]
    pub fn set_arg_count(&mut self, n: u16) {
        self.arg_count = n;
    }
    #[inline]
    pub fn flags_bits(&self) -> u8 {
        self.flags & 0x0F
    }
    #[inline]
    pub fn set_flags_bits(&mut self, bits: u8) {
        self.flags = (self.flags & !0x0F) | (bits & 0x0F);
    }
    #[inline]
    pub fn kind_raw(&self) -> u8 {
        self.kind
    }
    #[inline]
    pub fn set_kind_raw(&mut self, kind: u8) {
        self.kind = kind;
    }
}
```

> **Note on the handle APIs:** this code calls `CodeRef::from_raw(u32) -> Option<CodeRef>` / `CodeRef::raw(self) -> u32` and the same for `ObjectRef`/`EnvironmentRef`. The frame tests in `frame.rs` use `CodeRef::new(NonZeroU32)` / `EnvironmentRef::new(NonZeroU32)`. Confirm the exact raw constructor/getter names in `lyng_types` (grep `impl CodeRef`); if they are named differently (e.g. `try_from_raw`/`get`/`as_u32`), use those — the field encoding (raw `u32`, `0 = None`) is unchanged. `ExecutionContextKind` raw conversion uses whatever `as u8` / `from_u8` exists (it is a `#[repr(u8)]` enum — see `frame.rs` use of `ExecutionContextKind`).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p lyng_vm --lib frame_header::tests`
Expected: PASS. If `frame_header_offsets_stable` fails, the compiler reordered nothing (`repr(C)` is fixed) — a failure means the literal offsets above are wrong for this field order; read the assertion output and correct the expected constants (do not reorder fields).

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/frame_header.rs crates/vm/src/lib.rs
git commit -m "feat(vm): add repr(C) FrameHeader + offset lock-in test

Typed accessors over a POD overlay; asm-hot slots 0-3, warm slots 4-6;
realm/referrer/geometry not stored. Not yet overlaid on the arena.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Add the depth-indexed cold side-table

**Files:**
- Create: `crates/vm/src/frame_cold.rs`
- Modify: `crates/vm/src/lib.rs` (`mod frame_cold;`)
- Test: `crates/vm/src/frame_cold.rs` (`#[cfg(test)]`)

- [ ] **Step 1: Write the failing test**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::GeneratorResumeKind;
    use lyng_types::Value;

    #[test]
    fn cold_state_defaults_and_round_trips_by_depth() {
        let mut cold = FrameColdTable::new();
        cold.reset_at(0);
        assert_eq!(cold.get(0).handler_cursor, 0);
        assert!(!cold.get(0).resume_active);

        let slot = cold.get_mut(0);
        slot.handler_cursor = 5;
        slot.resume_active = true;
        slot.resume_value = Value::from_smi(8);
        assert_eq!(cold.get(0).handler_cursor, 5);
        assert!(cold.get(0).resume_active);

        // Reusing depth 0 for a new frame clears the stale state.
        cold.reset_at(0);
        assert_eq!(cold.get(0).handler_cursor, 0);
        assert!(!cold.get(0).resume_active);
        let _ = GeneratorResumeKind::Next;
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p lyng_vm --lib frame_cold::tests`
Expected: FAIL — `cannot find type FrameColdTable`.

- [ ] **Step 3: Implement the cold table**
```rust
use crate::frame::GeneratorResumeKind;
use lyng_types::{ObjectRef, Value};

/// Rare per-activation state that the asm fast path never touches and that does not
/// belong in the asm-addressable header: exception-handler cursor, tail-call linkage,
/// generator resume state, and the parameter-initializer end offset. Reset to default
/// on every frame push; keyed by frame depth.
#[derive(Clone, Copy, Debug)]
pub struct FrameColdState {
    pub handler_cursor: u16,
    pub tail_caller: Option<ObjectRef>,
    pub tail_caller_strict: bool,
    pub resume_kind: GeneratorResumeKind,
    pub resume_value: Value,
    pub resume_active: bool,
    pub parameter_initializer_end_offset: u32,
}

impl Default for FrameColdState {
    #[inline]
    fn default() -> Self {
        Self {
            handler_cursor: 0,
            tail_caller: None,
            tail_caller_strict: false,
            resume_kind: GeneratorResumeKind::Next,
            resume_value: Value::undefined(),
            resume_active: false,
            parameter_initializer_end_offset: 0,
        }
    }
}

/// Depth-indexed dense store of [`FrameColdState`]. Grows lazily to the deepest
/// frame seen; `reset_at` is called on every push to clear stale state from a prior
/// frame that occupied the same depth.
pub struct FrameColdTable {
    slots: Vec<FrameColdState>,
}

impl FrameColdTable {
    #[inline]
    pub fn new() -> Self {
        Self { slots: Vec::new() }
    }

    #[inline]
    fn ensure(&mut self, depth: usize) {
        if self.slots.len() <= depth {
            self.slots.resize(depth + 1, FrameColdState::default());
        }
    }

    #[inline]
    pub fn reset_at(&mut self, depth: usize) {
        self.ensure(depth);
        self.slots[depth] = FrameColdState::default();
    }

    #[inline]
    pub fn get(&self, depth: usize) -> &FrameColdState {
        &self.slots[depth]
    }

    #[inline]
    pub fn get_mut(&mut self, depth: usize) -> &mut FrameColdState {
        &mut self.slots[depth]
    }
}

impl Default for FrameColdTable {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
```
Make `GeneratorResumeKind` visible to this module: in `crates/vm/src/frame.rs` it is already `pub`; ensure `crate::frame::GeneratorResumeKind` resolves (it does — `frame` is a crate module).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p lyng_vm --lib frame_cold::tests`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/frame_cold.rs crates/vm/src/lib.rs
git commit -m "feat(vm): add depth-indexed FrameColdTable for rare frame state

handler_cursor/tail_caller/generator-resume/param-init-end, off the asm header.
Reset on push. Not yet wired in.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Group 2 — Move the value stack into the arena (interleaved with headers)

## Task 4: Replace `register_stack` with `FrameArena` (value windows only, no header offset yet)

This is the first behavior-touching task: the value backing becomes the never-realloc arena. **Headers are NOT yet in the arena** — `frames: Vec<FrameRecord>` stays authoritative; window bases keep their current values (no `+HEADER_SLOTS` offset yet). This isolates the never-realloc + soft-limit change for the value stack.

**Files:**
- Modify: `crates/vm/src/vm.rs` (fields `register_stack`/`register_stack_top` ~134-135; `register_stack_top()`; `reserve_register_window` ~1254-1278; `release_register_window`/`release_register_stack_to`; `register_stack_storage_mut_ptr`; any `register_stack()` iterator accessor used by GC)
- Modify: `crates/vm/src/vm/registers.rs` (`read_register`/`write_register`/`read_register_unchecked`/`write_register_unchecked`)
- Modify: `crates/vm/src/dsl/entry.rs` (regs base derivation ~78)
- Test: `crates/vm/src/vm.rs` test module (deep-recursion RangeError) + the existing register round-trip path (covered by the suite)

- [ ] **Step 1: Write the failing test**

In the `vm.rs` test module (search for `mod tests` / where `Vm::new()` unit tests live; if none, add a `#[cfg(test)] mod arena_tests` at the bottom of `vm.rs`):
```rust
    #[test]
    fn deep_recursion_throws_range_error_not_panic() {
        // Recurse past the arena soft limit; expect a clean RangeError, not a panic
        // or OOM. Reuse the script-eval harness other vm tests use to run source and
        // capture the thrown value's constructor name.
        let outcome = run_script("function f(n){ return f(n+1) } f(0)");
        assert!(
            outcome.threw_range_error(),
            "stack overflow must surface as RangeError"
        );
    }
```
Adapt `run_script`/`threw_range_error` to the existing harness (grep for how other vm tests run source and inspect a thrown error). If the harness is awkward, assert the lower-level invariant instead: a loop calling `vm.reserve_register_window(base, len)` past `ARENA_SOFT_LIMIT_SLOTS` returns the `RangeError` path rather than growing.

- [ ] **Step 2: Run test to verify it fails (or document today's behavior)**

Run: `cargo test -p lyng_vm --lib deep_recursion_throws_range_error_not_panic`
Expected: today this likely already throws via `MAX_BYTECODE_CALL_DEPTH` before the value stack matters — if it already passes, keep it as a guard and proceed (the arena must preserve it). If it fails or OOMs, that is the behavior this task fixes.

- [ ] **Step 3: Swap the fields**

In `struct Vm` (`vm.rs` ~134-135) replace:
```rust
    register_stack: Vec<Value>,
    register_stack_top: usize,
```
with:
```rust
    arena: crate::frame_arena::FrameArena,
```
In `Vm::new()` replace the `register_stack`/`register_stack_top` initializers with:
```rust
            arena: crate::frame_arena::FrameArena::new(),
```

- [ ] **Step 4: Repoint the arena helpers**

Replace `register_stack_top(&self) -> usize` with `self.arena.top()`. Rewrite `reserve_register_window` to bump the arena and reject on soft limit (was `resize`-based):
```rust
    fn reserve_register_window(&mut self, register_base: u32, register_len: u16) -> VmResult<()> {
        // register_base is the caller-chosen window base; in this task it still equals
        // the old register_stack index (no header offset yet). Ensure the arena cursor
        // covers the window end, rejecting past the soft limit.
        let end = usize::from(register_len)
            .checked_add(register_base as usize)
            .filter(|&end| end < crate::frame_arena::ARENA_SOFT_LIMIT_SLOTS)
            .ok_or_else(|| VmError::Abrupt(errors::throw_range_error(/* agent */)))?;
        if self.arena.top() < end {
            self.arena.set_top(end); // add a pub(crate) set_top that asserts end <= capacity
        }
        Ok(())
    }
```
(Keep the existing signature/return type — if `reserve_register_window` currently returns `()` and signals overflow elsewhere, preserve that contract; the key change is the backing + the soft-limit rejection. Add `FrameArena::set_top(&mut self, top: usize)` with `debug_assert!(top <= ARENA_CAPACITY_SLOTS)`. Thread `agent` to `throw_range_error` the same way the existing `MAX_BYTECODE_CALL_DEPTH` check does in `bytecode_calls.rs`.)

Replace `release_register_stack_to`/`release_register_window` bodies to call `self.arena.release_to(top)`. Replace `register_stack_storage_mut_ptr()` with `self.arena.base_mut_ptr()`. Replace the GC iterator accessor `register_stack()` with one returning `self.arena.slots()` (the wholesale trace stays for now — Task 12 changes it).

- [ ] **Step 5: Repoint the register accessors (`registers.rs`)**

In all four accessors replace `self.register_stack[absolute]` / `get_unchecked` with the arena slice:
```rust
        self.arena.slots()[absolute]                       // read
        self.arena.slots_mut()[absolute] = value;          // write
        unsafe { *self.arena.slots().as_ptr().add(absolute) }            // read_unchecked
        unsafe { *self.arena.slots_mut().as_mut_ptr().add(absolute) = value } // write_unchecked
```
and replace the `debug_assert!(absolute < self.register_stack_top())` lines with `absolute < self.arena.top()`.

- [ ] **Step 6: Repoint `dsl/entry.rs` regs base (~78)**

Was `unsafe { vm.register_stack_storage_mut_ptr().add(base) }`; becomes:
```rust
        let regs_base = unsafe { vm.arena.base_mut_ptr().add(base) };
```
(Match the exact surrounding expression; only the storage pointer source changes.)

- [ ] **Step 7: Build + run the vm suite**

Run: `cargo build -p lyng_vm`
Expected: success (fix any stragglers still naming `register_stack`).
Run: `cargo test -p lyng_vm`
Expected: PASS (including the deep-recursion guard).

- [ ] **Step 8: Commit**

```bash
git add crates/vm/src/vm.rs crates/vm/src/vm/registers.rs crates/vm/src/dsl/entry.rs
git commit -m "refactor(vm): back the value stack with the never-realloc FrameArena

register_stack -> FrameArena (fixed Box<[Value]>, bump cursor, soft limit).
Headers still in frames Vec; window bases unchanged. Stack overflow now a clean
RangeError at the soft limit.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: Interleave the header — reserve `HEADER_SLOTS` before each window and mirror the `FrameHeader`

Now make every frame occupy `[header][window]` in the arena: bump `HEADER_SLOTS + window_len`, set `RegisterWindow.base = cfr + HEADER_SLOTS`, and write the `FrameHeader` overlay at `cfr` mirroring the `FrameRecord` that is still authoritative. Readers still use `frames`/`FrameRecord`; this task only adds the mirror + the cfr/depth bookkeeping.

**Files:**
- Modify: `crates/vm/src/vm.rs` (`Vm` fields: add `current_cfr: u32`, `frame_cold: FrameColdTable`; `reserve_register_window` to account for the header; add `frame_header(cfr)`/`frame_header_mut(cfr)` overlay helpers; add `write_header_from_record`)
- Modify: `crates/vm/src/vm/bytecode_calls.rs` (both push sites), `crates/vm/src/vm.rs` (entry push), `crates/vm/src/vm/generators.rs` (restore), `crates/vm/src/vm/jobs.rs` (job frame)
- Test: `crates/vm/src/vm.rs` test module (header overlay mirrors the record)

- [ ] **Step 1: Write the failing test**
```rust
    #[test]
    fn arena_header_overlay_mirrors_the_record_at_entry() {
        // Drive a call to a live frame (reuse the Task-from-SP-0a harness that exposes
        // vm.frame()). Assert the overlaid header at the current cfr matches the record:
        let (vm, _agent) = run_to_first_call("function f(){ return 1 } f()");
        let rec = vm.frame().expect("a live frame");
        let hdr = vm.current_frame_header().expect("a live header");
        assert_eq!(hdr.code(), rec.code());
        assert_eq!(hdr.callee(), rec.callee());
        assert_eq!(hdr.variable_env(), rec.variable_env());
        assert_eq!(hdr.lexical_env(), rec.lexical_env());
        assert_eq!(hdr.this_value(), rec.this_value());
        assert_eq!(hdr.this_state(), rec.this_state());
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p lyng_vm --lib arena_header_overlay_mirrors_the_record_at_entry`
Expected: FAIL — `no method named current_frame_header`.

- [ ] **Step 3: Add the overlay helpers + fields (`vm.rs`)**

Add fields to `struct Vm` next to `arena`:
```rust
    arena: crate::frame_arena::FrameArena,
    current_cfr: u32,                       // u32::MAX when no frame is active
    frame_cold: crate::frame_cold::FrameColdTable,
```
Init in `new()`: `current_cfr: u32::MAX,` and `frame_cold: crate::frame_cold::FrameColdTable::new(),`.

Add overlay accessors (POD pointer cast onto arena slots):
```rust
    /// Borrow the `FrameHeader` overlaid at `cfr` (slot offset into the arena).
    #[inline]
    pub(crate) fn frame_header(&self, cfr: u32) -> &crate::frame_header::FrameHeader {
        let ptr = self.arena.slots().as_ptr();
        // SAFETY: cfr is a valid frame base reserved with HEADER_SLOTS+window slots;
        // FrameHeader is repr(C) POD sized to HEADER_SLOTS * size_of::<Value>().
        unsafe { &*(ptr.add(cfr as usize) as *const crate::frame_header::FrameHeader) }
    }

    #[inline]
    pub(crate) fn frame_header_mut(&mut self, cfr: u32) -> &mut crate::frame_header::FrameHeader {
        let ptr = self.arena.slots_mut().as_mut_ptr();
        // SAFETY: as above; &mut access is unique while held.
        unsafe { &mut *(ptr.add(cfr as usize) as *mut crate::frame_header::FrameHeader) }
    }

    #[inline]
    pub(crate) fn current_frame_header(&self) -> Option<&crate::frame_header::FrameHeader> {
        (self.current_cfr != u32::MAX).then(|| self.frame_header(self.current_cfr))
    }
```
Add a helper that fills a header overlay from a `FrameRecord` (the mirror) and seeds the cold slot:
```rust
    fn write_header_from_record(&mut self, cfr: u32, caller_cfr: Option<u32>, record: &FrameRecord) {
        let depth = self.frames.len(); // depth of the frame being pushed (pre-push len)
        self.frame_cold.reset_at(depth);
        {
            let cold = self.frame_cold.get_mut(depth);
            cold.handler_cursor = record.handler_cursor();
            cold.tail_caller = record.tail_caller();
            cold.tail_caller_strict = record.tail_caller_strict();
            cold.resume_kind = record.resume_kind();
            cold.resume_value = record.resume_value();
            cold.resume_active = record.resume_active();
            cold.parameter_initializer_end_offset = record.parameter_initializer_end_offset();
        }
        let h = self.frame_header_mut(cfr);
        *h = crate::frame_header::FrameHeader::zeroed();
        h.set_caller_cfr(caller_cfr);
        h.set_saved_pc(record.instruction_offset());
        h.set_code(record.code());
        h.set_callee(record.callee());
        h.set_this(record.this_state(), record.this_value());
        h.set_return_register(record.return_register());
        h.set_variable_env(record.variable_env());
        h.set_lexical_env(record.lexical_env());
        h.set_private_env(record.private_env());
        h.set_new_target(record.new_target());
        h.set_construct_this(record.construct_this());
        h.set_arg_count(u16::try_from(record.registers().len()).unwrap_or(u16::MAX));
        h.set_flags_bits(record.flags().raw());
        h.set_kind_raw(record.kind() as u8);
    }
```
(`record.kind() as u8` requires `ExecutionContextKind: Copy + #[repr(u8)]`; if it lacks an `as u8` cast, add/locate a `kind.raw()`-style method. Confirm in `lyng_env`.)

- [ ] **Step 4: Reserve the header in `reserve_register_window`**

Change the window allocation so each frame bumps the header + the window in one contiguous run, and the window base is returned as `cfr + HEADER_SLOTS`. The simplest bridge: add a new helper used at push sites and keep `reserve_register_window` for compatibility:
```rust
    /// Reserve `[header][window]` for a new frame at the current arena top.
    /// Returns `(cfr, window_base)` where `window_base = cfr + HEADER_SLOTS`.
    fn reserve_frame(&mut self, register_len: u16) -> VmResult<(u32, u32)> {
        let slots = crate::frame_header::HEADER_SLOTS + usize::from(register_len);
        let cfr = self
            .arena
            .bump(slots)
            .ok_or_else(|| VmError::Abrupt(errors::throw_range_error(/* agent */)))?;
        let window_base = cfr + crate::frame_header::HEADER_SLOTS as u32;
        Ok((cfr, window_base))
    }
```
At each push site (entry `vm.rs`, `bytecode_calls.rs` ×2, `generators.rs` restore, `jobs.rs`), replace the current `register_base` computation + `reserve_register_window(base, len)` with `let (cfr, register_base) = self.reserve_frame(register_len)?;`, build the `FrameRecord` exactly as today using `register_base` for the `RegisterWindow`, then **after** `self.frames.push(frame)` add:
```rust
        let caller_cfr = (self.current_cfr != u32::MAX).then_some(self.current_cfr);
        self.write_header_from_record(cfr, caller_cfr, &frame);
        self.current_cfr = cfr;
```
(Place `self.current_cfr = cfr;` so the new frame is current; the unwind in Task 6 restores it.)

- [ ] **Step 5: Restore `current_cfr` on unwind/return (bridge)**

In `finish_frame` (`registers.rs`) and every frame-unwind loop (`vm.rs` entry unwind, `internal_calls.rs`), after popping a frame set `current_cfr` to the caller and release the arena to the popped frame's `cfr`:
```rust
        // after self.frames.pop():
        self.current_cfr = self.frame_header(popped_cfr).caller_cfr().unwrap_or(u32::MAX);
        self.arena.release_to(popped_cfr);
```
To know `popped_cfr`, read it from the frame being popped before releasing — since `RegisterWindow.base == cfr + HEADER_SLOTS`, `popped_cfr = frame.registers().base() - HEADER_SLOTS as u32`. Add a tiny helper `fn cfr_of(frame: &FrameRecord) -> u32 { frame.registers().base() - HEADER_SLOTS as u32 }`. (This is the bridge linking the record to its overlay; Task 7+ removes the record.)

- [ ] **Step 6: Build + run the vm suite**

Run: `cargo build -p lyng_vm`
Run: `cargo test -p lyng_vm`
Expected: PASS (including `arena_header_overlay_mirrors_the_record_at_entry`). The overlay now exists for every frame; nothing reads it yet except the test.

- [ ] **Step 7: Commit**

```bash
git add crates/vm/src/vm.rs crates/vm/src/vm/registers.rs crates/vm/src/vm/bytecode_calls.rs crates/vm/src/vm/generators.rs crates/vm/src/vm/jobs.rs crates/vm/src/vm/internal_calls.rs
git commit -m "feat(vm): interleave FrameHeader before each window in the arena

Every frame now occupies [header][window]; current_cfr + caller_cfr chain track
the activation stack; the cold table is seeded on push. Header mirrors the still-
authoritative FrameRecord; no readers migrated yet.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Group 3 — Migrate readers onto the overlay + cfr/depth

## Task 6: Add overlay-backed frame accessors and depth/walk helpers

Provide the API the readers will switch to: a current-frame view over the overlay + cold table, a `caller_cfr` walk, and a `frame_depth`. This lets later tasks replace `self.frames.last()`/`self.frames.len()`/`frame.X()` without each one re-deriving the overlay.

**Files:**
- Modify: `crates/vm/src/vm.rs` (accessor cluster near `frame()`)
- Test: `crates/vm/src/vm.rs` test module

- [ ] **Step 1: Write the failing test**
```rust
    #[test]
    fn frame_depth_and_caller_walk_track_nested_calls() {
        // Drive two nested calls; at the innermost live frame assert frame_depth()==N
        // and walking caller_cfr reaches the root (None). Reuse the call harness.
        let (vm, _agent) = run_to_nested_call("function g(){ return 1 } function f(){ return g() } f()");
        assert!(vm.frame_depth() >= 2);
        // Walk to the root:
        let mut cfr = vm.current_cfr_opt().expect("live");
        let mut steps = 0;
        while let Some(caller) = vm.frame_header(cfr).caller_cfr() {
            cfr = caller;
            steps += 1;
        }
        assert!(steps >= 1, "caller chain bottoms out at the root");
    }
```

- [ ] **Step 2: Run it (fails — missing helpers)**

Run: `cargo test -p lyng_vm --lib frame_depth_and_caller_walk_track_nested_calls`
Expected: FAIL — `no method named frame_depth`/`current_cfr_opt`.

- [ ] **Step 3: Add the helpers**
```rust
    #[inline]
    pub(crate) fn current_cfr_opt(&self) -> Option<u32> {
        (self.current_cfr != u32::MAX).then_some(self.current_cfr)
    }

    /// Depth of the active frame stack (0 == empty). Bridge: equals `self.frames.len()`
    /// until the Vec is deleted; afterwards it is maintained directly.
    #[inline]
    pub(crate) fn frame_depth(&self) -> usize {
        self.frames.len()
    }

    /// Cold state of the current frame.
    #[inline]
    pub(crate) fn current_cold(&self) -> Option<&crate::frame_cold::FrameColdState> {
        (self.current_cfr != u32::MAX).then(|| self.frame_cold.get(self.frame_depth() - 1))
    }
```

- [ ] **Step 4: Run it**

Run: `cargo test -p lyng_vm --lib frame_depth_and_caller_walk_track_nested_calls`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm.rs
git commit -m "feat(vm): overlay-backed current-frame + caller-walk + depth helpers

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 7: Migrate the hot per-opcode readers onto the overlay

Switch the dispatch-path readers of frame fields from the `FrameRecord` to the overlay. **Field → overlay accessor mapping (identical names):** `frame.code()`→`hdr.code()`, `.callee()`→`hdr.callee()`, `.this_value()`→`hdr.this_value()`, `.this_state()`→`hdr.this_state()`, `.lexical_env()`→`hdr.lexical_env()`, `.variable_env()`→`hdr.variable_env()`, `.private_env()`→`hdr.private_env()`, `.new_target()`→`hdr.new_target()`, `.construct_this()`→`hdr.construct_this()`, `.kind()`→`ExecutionContextKind::from_u8(hdr.kind_raw())`, `.return_register()`→`hdr.return_register()`, `.registers()`→derive `RegisterWindow::new(cfr + HEADER_SLOTS, window_len_from_code)`; `.handler_cursor()/.tail_caller()/.resume_*()/.parameter_initializer_end_offset()`→`self.current_cold()`.

> This is a sweep. Use the SP-0a Task-10 method: keep `self.frame()` returning a `FrameRecord` (the bridge) so non-hot readers are untouched this task; migrate ONLY the hot dispatch readers that show up in the V8 profile. Migrate the rest in Task 8.

**Files & representative sites** (grep `self.frame()` / `frame.` in these hot paths):
- `crates/vm/src/vm/semantics/names.rs` (this/scope reads)
- `crates/vm/src/vm/property_access.rs`
- `crates/vm/src/dsl/entry.rs` (initial this/const base population — already reads `frame.this_state()`/`this_value()` via `resolve_initial_this_value`)
- `crates/vm/src/vm/registers.rs` (`finish_frame` return-register read → `hdr.return_register()` of the popped frame)

- [ ] **Step 1: Write/confirm the guard test**

Reuse the suite — these are behavior-preserving. Add one explicit guard if missing:
```rust
    #[test]
    fn this_read_matches_after_overlay_migration() {
        // `function f(){ return this } f.call(42)` returns 42 (boxed) — exercises the
        // overlay this_value/this_state read on the hot path.
        assert_eq!(run_script_to_number("(function(){ return this }).call(42)"), Some(42.0));
    }
```

- [ ] **Step 2: Migrate each hot reader**

Representative diff — `dsl/entry.rs` initial-this population currently does `resolve_initial_this_value(&frame)` from a `FrameRecord`. Change the source to the overlay (the function takes the same fields):
```rust
        // was: let this_mirror = resolve_initial_this_value(&frame);
        let hdr = vm.current_frame_header().expect("entry has a live frame");
        let this_mirror = crate::dsl::llint_state::resolve_this_state_to_mirror(
            Some(hdr.this_state()),
            hdr.this_value(),
        );
```
For `finish_frame` (`registers.rs`), the return-register write to the caller becomes (read the caller overlay, not `caller.registers()`):
```rust
        if let Some(caller_cfr) = self.current_cfr_opt() {
            if let Some(return_register) = popped_return_register {
                let caller_window = RegisterWindow::new(
                    caller_cfr + crate::frame_header::HEADER_SLOTS as u32,
                    self.window_len_for(self.frame_header(caller_cfr).code()),
                );
                self.write_register(caller_window, return_register, result);
            }
            return Ok(None);
        }
```
where `popped_return_register = hdr_of_popped.return_register()` and `window_len_for(code)` reads the code's register count (the value `RegisterWindow.len` is set from today; add `fn window_len_for(&self, code: CodeRef) -> u16` that reads it from the code record the same way `reserve_register_window` callers compute `register_len`).

- [ ] **Step 3: Run the suite**

Run: `cargo test -p lyng_vm`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "refactor(vm): hot dispatch readers use the FrameHeader overlay

this/scope/return-register hot paths read the arena overlay; FrameRecord bridge
retained for the cold readers (next task).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 8: Migrate the remaining frame readers and the whole-stack `this_state` scan

Replace every remaining `self.frames.last()`/`.last_mut()`/`self.frames[i]`/`frame.X()` reader with the overlay (`frame_header(cfr)`/`frame_header_mut(cfr)`), the cold table, or the caller walk. The mutators (`set_this_value`/`set_construct_this`/`set_this_state`/`set_lexical_env`/`set_instruction_offset`/`set_handler_cursor`/`set_tail_caller`/`clear_resume`) move to `frame_header_mut(self.current_cfr)` / `current_cold mut`.

**Files & exact sites** (from the census — read each for context):
- `crates/vm/src/vm/registers.rs:88-94` (`clear_active_resume` → cold table)
- `crates/vm/src/vm/bytecode_calls.rs:76-109` (tail-call recycle: `frames.last_mut().set_tail_caller` → cold table; `frame_is_strict`)
- `crates/vm/src/vm/builtin_dispatch/class_helpers/super_ops.rs` (the `super()` writes `set_this_value`/`set_construct_this`/`set_this_state`; the whole-stack scan `for frame in self.frames.iter_mut().filter(|f| f.lexical_env() == function_env)`)
- `crates/vm/src/vm/generators.rs` (snapshot reads `frame.X()`; restore writes — see Task 9)
- `crates/vm/src/vm/exceptions.rs` (handler cursor reads/writes → cold table)
- `crates/vm/src/vm/semantics/control_flow.rs`, `vm/call.rs`, `vm/dispatch_state.rs`, `vm/async_functions.rs`, `vm/dynamic_compilation.rs` (residual `frame.X()` reads)

**The whole-stack `this_state` scan** (`super_ops.rs`) must now iterate the arena frames by depth rather than `self.frames`:
```rust
    fn set_this_state_for_lexical_env(&mut self, function_env: EnvironmentRef, this_value: Value) {
        let mut updated = false;
        for cfr in self.frame_cfrs() { // new helper: walk all live cfrs by depth
            if self.frame_header(cfr).lexical_env() == function_env {
                self.frame_header_mut(cfr).set_this(ThisState::Value(this_value), this_value);
                updated = true;
            }
        }
        if !updated {
            if let Some(cfr) = self.current_cfr_opt() {
                self.frame_header_mut(cfr).set_this(ThisState::Value(this_value), this_value);
            }
        }
    }
```
Add `fn frame_cfrs(&self) -> impl Iterator<Item = u32>`: walk from `current_cfr` via `caller_cfr` collecting offsets (or iterate `self.frames` mapping to `cfr_of` during the bridge — either works; prefer the caller walk so it survives Task 11).

- [ ] **Step 1: Write/confirm the guard test**

Reuse `super_initializes_this_for_arrow_closures` (carried from SP-0a) — it must keep passing through the scan migration. If absent, add it (see SP-0a plan Task 10).

- [ ] **Step 2: Migrate each site** per the mapping. Representative — `super_ops.rs` constructed-frame write:
```rust
        // was: frame.set_this_value(this_value); frame.set_construct_this(Some(obj)); frame.set_this_state(...)
        let cfr = self.current_cfr_opt().expect("super() runs in a live constructor frame");
        let h = self.frame_header_mut(cfr);
        h.set_this(ThisState::Value(this_value), this_value);
        h.set_construct_this(Some(this_object));
```

- [ ] **Step 3: Run the suite**

Run: `cargo test -p lyng_vm`
Expected: PASS (including the super-arrow guard).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "refactor(vm): migrate all frame readers/mutators onto the overlay + cold table

Including the super() this_state writes and the lexical-env whole-stack scan
(now an arena cfr walk). FrameRecord no longer read except by the bridge accessor.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 9: Generator snapshot/restore via the overlay + cold table

Snapshot must copy the window slice + the header/cold fields into `SuspendedExecutionRef`; restore reconstructs the frame in the arena and re-seeds the overlay + cold slot.

**Files:**
- Modify: `crates/vm/src/vm/generators.rs` (`snapshot_suspended_execution` ~1616+, `restore_suspended_execution` ~1551-1583)
- Test: `crates/vm/src/vm/generators.rs` test module (round-trip)

- [ ] **Step 1: Write the failing test**
```rust
    #[test]
    fn generator_snapshot_restore_round_trips_window_and_this() {
        // function* g(){ const a = 1; yield; return a + 1 }
        // Drive to the yield (snapshot), resume (restore), assert the final value is 2
        // and `this`/resume_value survived. Reuse the generator test harness.
        assert_eq!(run_generator_to_completion("function* g(){ const a=1; yield; return a+1 } g()"), Some(2.0));
    }
```

- [ ] **Step 2: Run it**

Run: `cargo test -p lyng_vm --lib generator_snapshot_restore_round_trips_window_and_this`
Expected: PASS today (still on the record path) — keep as a guard, then migrate.

- [ ] **Step 3: Snapshot from the overlay/window**

`snapshot_suspended_execution` already copies `self.register_stack[register_base..].to_vec()` — change to `self.arena.slots()[register_base..register_base + window_len].to_vec()` (bounded by the window, not to-end). Read the header/cold fields from `self.frame_header(cfr)` + `self.frame_cold.get(depth)` instead of the `&FrameRecord` argument (or keep the `&FrameRecord` argument during the bridge; it still mirrors). The stored fields are unchanged (`code/offset/realm/envs/this_state/this_value/kind/callee/new_target/construct_this/handler_cursor/flags/registers`).

- [ ] **Step 4: Restore by reserving a frame + seeding the overlay**

In `restore_suspended_execution`, replace the `reserve_register_window` + `frames.push` with `reserve_frame(window_len)` + write the window slice back + `write_header_from_record(cfr, caller_cfr, &frame)` (the record is rebuilt from the snapshot exactly as today), then `self.current_cfr = cfr;` and seed `resume_*` into the cold slot. Restore-time `push_referrer_scope`/`refresh_running_context` stay.

- [ ] **Step 5: Run the generator/async suite**

Run: `cargo test -p lyng_vm generator; cargo test -p lyng_vm async`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/vm/src/vm/generators.rs
git commit -m "refactor(vm): generator snapshot/restore over the arena window + overlay

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Group 4 — GC over the arena

## Task 10: Switch GC to the per-frame arena walk

Replace the wholesale `register_stack` trace + `frames` iteration with a frame walk: per frame trace the typed header refs + that frame's window slice as Values, plus the cold table refs and the establishment side-stack (realm added in Task 11).

**Files:**
- Modify: `crates/vm/src/vm/state.rs` (`ActiveVmRoots::trace_heap_edges` ~204-213; `trace_frame_record` ~293-305 → `trace_frame_header`)
- Test: `crates/vm/src/vm/state.rs` test module (sole-ref-in-window survives; header ref survives; no header slot mistraced)

- [ ] **Step 1: Write the failing tests**
```rust
    #[test]
    fn minor_gc_object_only_in_register_window_survives() {
        // Allocate an object, store it ONLY in a live frame's register window, run a
        // minor GC, assert it still resolves. Model on the existing
        // minor_gc_cell_backed_global_value_survives test.
    }

    #[test]
    fn minor_gc_header_callee_survives() {
        // A callee object reachable only via a live frame header survives a minor GC.
    }
```

- [ ] **Step 2: Run them**

Run: `cargo test -p lyng_vm --lib state::tests::minor_gc_object_only_in_register_window_survives state::tests::minor_gc_header_callee_survives`
Expected: today they likely PASS (wholesale trace covers windows; record trace covers callee) — keep as guards; they must stay green after the walk replaces the wholesale trace.

- [ ] **Step 3: Replace the trace**

In `ActiveVmRoots::trace_heap_edges`, delete the `for value in self.vm.register_stack() { value.trace_heap_edges(tracer); }` wholesale loop and the `for frame in &self.vm.frames { trace_frame_record(...) }` loop; replace with a cfr walk:
```rust
        // Trace each live frame: typed header refs + that frame's window as Values.
        for cfr in self.vm.frame_cfrs() {
            trace_frame_header(self.vm, cfr, tracer);
        }
        // Cold-table refs (tail_caller, resume_value) for live frames.
        self.vm.trace_cold_table(tracer);
```
Add `trace_frame_header`:
```rust
fn trace_frame_header(vm: &Vm, cfr: u32, tracer: &mut PrimitiveTracer<'_>) {
    let h = vm.frame_header(cfr);
    h.code().trace_heap_edges(tracer);
    h.variable_env().trace_heap_edges(tracer);
    h.lexical_env().trace_heap_edges(tracer);
    if let Some(e) = h.private_env() { e.trace_heap_edges(tracer); }
    h.this_value().trace_heap_edges(tracer);
    if let Some(o) = h.construct_this() { o.trace_heap_edges(tracer); }
    if let Some(o) = h.new_target() { o.trace_heap_edges(tracer); }
    if let Some(o) = h.callee() { o.trace_heap_edges(tracer); }
    // realm is NOT traced here in this task; Task 11 traces it via the side-stack.
    // Window: this + args + locals + temps live in [regs_base .. regs_base + len].
    let regs_base = cfr as usize + crate::frame_header::HEADER_SLOTS;
    let len = usize::from(vm.window_len_for(h.code()));
    for value in &vm.arena.slots()[regs_base..regs_base + len] {
        value.trace_heap_edges(tracer);
    }
}
```
Add `Vm::trace_cold_table` iterating live depths tracing `tail_caller` + `resume_value`. Keep `frame.realm()` traced via the record for THIS task (the record is still authoritative for realm until Task 11) — trace it inside `trace_frame_header` from the record bridge: `vm.frames[depth_of(cfr)].realm().trace_heap_edges(tracer);` — or simply leave the `frames` realm trace loop in place until Task 11 and remove it there. Pick one; the cleaner path is to leave a minimal `for frame in &self.vm.frames { frame.realm().trace_heap_edges(tracer); }` realm-only loop here and delete it in Task 11.

- [ ] **Step 4: Run the GC suite**

Run: `cargo test -p lyng_vm gc; cargo test -p lyng_vm --lib state::tests`
Expected: PASS. **This is the heaviest-risk task** — also run the full suite: `cargo test -p lyng_vm`.

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm/state.rs crates/vm/src/vm.rs
git commit -m "refactor(vm): GC walks the arena frame chain (per-window trace)

Replaces the wholesale register-stack trace (unsafe under interleaved headers)
with per-frame header-refs + window-Values tracing + cold-table refs. Header slots
are never traced as Values.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Group 5 — Realm + referrer derivation

## Task 11: Derive realm; merge realm into the establishment side-stack; delete the stored realm

**Files:**
- Modify: `crates/vm/src/vm.rs` (the `ReferrerScope`/establishment side-stack from SP-0a → add `realm`; `push_referrer_scope` → `push_establishment_scope(base_depth, realm, referrer)`; add `realm_of`; `refresh_running_context` computes realm via `realm_of`; GC traces the side-stack realms)
- Modify: `crates/vm/src/vm/bytecode_calls.rs` (drop the stored-realm derivation `function_data.realm().unwrap_or_else(caller_frame.realm())` — `realm_of` subsumes it), the establishment pushes at the four roots
- Modify: `crates/vm/src/frame_header.rs` — no realm field exists (good); `crates/vm/src/frame.rs` — remove the `realm` field + `with`/accessor in this task or Task 13
- Modify: `crates/vm/src/vm/state.rs` — delete the realm-only `frames` trace loop left in Task 10
- Modify: the ~10 arbitrary-frame `.realm()` readers → `self.realm_of(cfr)`; the ~70 current-frame readers → `agent.running_context()...realm()` (most already do post-SP-0a)
- Test: multi-realm + cross-realm + root realm

- [ ] **Step 1: Write the failing tests**
```rust
    #[test]
    fn cross_realm_call_frame_uses_callee_realm() {
        // A function defined in realm B, called from realm A: inside B's frame,
        // realm_of(current) == B. Build via the multi-realm test helper.
    }

    #[test]
    fn throw_in_secondary_realm_uses_that_realms_error_prototype() {
        // (carried from SP-0a Task 8 as error_uses_running_context_realm) — confirm.
    }
```

- [ ] **Step 2: Run them**

Run: `cargo test -p lyng_vm --lib cross_realm_call_frame_uses_callee_realm`
Expected: FAIL — `no method named realm_of`.

- [ ] **Step 3: Add `realm_of` + the establishment realm**

Extend the side-stack entry (SP-0a's `ReferrerScope`) with `realm: RealmRef`, rename to `EstablishmentScope`, and update `push_*`/`unwind_*`/`current_referrer` accordingly. Add:
```rust
    /// Realm of the frame at `cfr`: function frames derive from the callee's [[Realm]];
    /// root frames read the covering establishment scope.
    pub(crate) fn realm_of(&self, agent: &Agent, cfr: u32) -> RealmRef {
        let h = self.frame_header(cfr);
        if let Some(callee) = h.callee() {
            if let Some(realm) = agent.objects().function_data(callee).and_then(|d| d.realm()) {
                return realm;
            }
        }
        // root / callee-less: covering establishment scope by this frame's depth
        self.establishment_realm_covering(self.depth_of(cfr))
    }
```
(`function_data(callee).realm()` is the pure accessor at `objects/functions.rs:967`; confirm `agent.objects().function_data(obj)` returns `Option<&FunctionObjectData>`.) `establishment_realm_covering(depth)` mirrors `current_referrer`'s walk but returns the scope's `realm`. Update `refresh_running_context` to compute `realm` via `realm_of(agent, self.current_cfr)` instead of `frame.realm()`. Add the side-stack realm trace in `state.rs`.

- [ ] **Step 4: Push the realm at the four establishment points**

Wherever SP-0a pushed a referrer scope (script/module/job/generator-restore), pass the realm in hand (`realm.id()` from the `RealmRecord`/the entry param). Delete the per-call realm derivation in `bytecode_calls.rs` (function frames now derive on read).

- [ ] **Step 5: Migrate the `.realm()` readers + delete the stored realm**

Arbitrary-frame readers → `self.realm_of(agent, cfr)`. Current-frame readers that still call `frame.realm()` → `agent.running_context().map(RunningContext::realm)` (or `realm_of(current_cfr)`). Then remove `realm` from `FrameRecord::new`/`FrameMetadata`/accessor and the `RealmRef` import if now unused in `frame.rs`. Remove the realm-only trace loop in `state.rs`.

- [ ] **Step 6: Build + run the suite**

Run: `cargo build -p lyng_vm`
Run: `cargo test -p lyng_vm; cargo test -p lyng_ops`
Expected: PASS (incl. multi-realm + cross-realm tests).

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "refactor(vm): derive realm (callee/establishment); drop the stored realm

realm_of derives function-frame realm from the callee and root realm from the
establishment side-stack (now carrying realm + referrer). running_context realm
computed via realm_of; FrameRecord.realm removed; GC traces side-stack realms.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Group 6 — Soft limit reconciliation

## Task 12: Reconcile the soft-limit prologue with the depth cap

The arena soft-limit (Task 4/5) is the byte-budget guard. Confirm it co-exists with `MAX_BYTECODE_CALL_DEPTH` and that both throw `RangeError` cleanly, and add a debug assert that the slack is sufficient for the throw path.

**Files:**
- Modify: `crates/vm/src/vm/bytecode_calls.rs` (the `MAX_BYTECODE_CALL_DEPTH` check ~259/423 — keep as the secondary cap), `crates/vm/src/vm.rs` (`reserve_frame` is the primary soft-limit gate)
- Test: `crates/vm/src/vm.rs` (deep recursion throws once, no double-throw / no panic)

- [ ] **Step 1: Write the failing/guard test**
```rust
    #[test]
    fn soft_limit_throw_runs_inside_the_reservation() {
        // Recurse to overflow; assert exactly one RangeError surfaces and the VM is
        // reusable afterward (run a trivial script post-overflow and get a value).
        let mut h = harness();
        assert!(h.run("function f(){ return f() } f()").threw_range_error());
        assert_eq!(h.run("1 + 1").as_number(), Some(2.0));
    }
```

- [ ] **Step 2: Run it**

Run: `cargo test -p lyng_vm --lib soft_limit_throw_runs_inside_the_reservation`
Expected: PASS (Tasks 4/5 already throw). If it panics/OOMs, raise `ARENA_SLACK_SLOTS` until the throw path fits.

- [ ] **Step 3: Add the slack debug assert** in `reserve_frame`:
```rust
        debug_assert!(
            crate::frame_arena::ARENA_SLACK_SLOTS
                >= crate::frame_header::HEADER_SLOTS + 256,
            "slack must cover the throw path's own frame + window"
        );
```

- [ ] **Step 4: Run the suite**

Run: `cargo test -p lyng_vm`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(vm): soft-limit prologue is the primary stack-overflow gate

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Group 7 — Delete the old path

## Task 13: Delete `frames: Vec<FrameRecord>` and the bridge

Now that all readers use the overlay/cold/derivation, remove the Vec and any `FrameRecord`-bridge accessor. `frame_depth()` is maintained directly; `frame()` returning a `FrameRecord` is deleted (callers already moved).

**Files:**
- Modify: `crates/vm/src/vm.rs` (delete `frames` field; `frame()`/`frame_mut()` bridge; maintain `frame_depth` as a `u32` field bumped/decremented on push/unwind; `cfr_of`/`depth_of` helpers now derive from the maintained depth + caller walk)
- Modify: every site that still references `self.frames` (push/unwind loops set `current_cfr`/`frame_depth` directly; `frame_cfrs` walks the caller chain)
- Modify: `crates/vm/src/frame.rs` — `FrameRecord` may still be used as a transient builder at push sites (build → `write_header_from_record` → discard). If so, keep it as a pure builder; otherwise delete it. Decide per remaining usage.
- Test: full suite + Test262 checkpoint (Task 14)

- [ ] **Step 1: Add a maintained depth + walk-based `frame_cfrs`**

Replace `frame_depth()`'s `self.frames.len()` body with a `frame_depth: u32` field incremented on push, decremented on unwind. Replace `frame_cfrs` and `depth_of` with caller-chain walks. Replace `cfr_of(frame)` (record-based) usages with the maintained `current_cfr`.

- [ ] **Step 2: Delete `self.frames`**

Remove the field + init. Fix each compile error: push sites no longer `self.frames.push(frame)` (they `write_header_from_record` + bump `frame_depth` + set `current_cfr`); unwind loops `while self.frame_depth() > target { pop_one_frame(); }` where `pop_one_frame` reads `current_cfr`, releases the arena to it, sets `current_cfr = caller`, decrements depth, and runs the existing `close_*_frames`/`clear_window` cleanups keyed by depth.

- [ ] **Step 3: Build iteratively**

Run: `cargo build -p lyng_vm`
Fix each `self.frames` straggler. Expected end: clean build.

- [ ] **Step 4: Run the suite**

Run: `cargo test -p lyng_vm; cargo test -p lyng_ops; cargo test --workspace`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "refactor(vm): delete frames Vec; the arena is the sole frame backing

current_cfr + a maintained frame_depth + the caller_cfr chain replace the Vec.
FrameHeader overlay + cold table + derivation are the single source of truth.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 14: Test262 full-suite checkpoint (no code change)

- [ ] **Step 1: Run the entire workspace**

Run: `cargo test --workspace`
Expected: PASS.

- [ ] **Step 2: Run Test262**

Run the harness the repo uses (check `crates/test262-harness`; the SP-0a plan used e.g. `cargo run -p lyng-test262-harness --release -- --threads 8`).
Expected: **100% pass rate, identical to the pre-SP-0b baseline (49728 passed, 0 panics).** If anything regresses, STOP and bisect against the branch point — do not proceed.

- [ ] **Step 3: No commit** (verification only). Record pass counts in the task notes.

---

## Task 15: Publish the arena base to `LlIntState`; define header offset constants for SP-1

SP-0b has no asm, but pin the contract SP-1 will consume: `frame_regs_base` derives from the arena, and the `FrameHeader` field offsets get named constants beside `LlIntState`'s, locked by the Task-2 test.

**Files:**
- Modify: `crates/vm/src/dsl/reg_convention.rs` (add `FRAME_HEADER_*` offset constants via `offset_of!(FrameHeader, …)`)
- Modify: `crates/vm/src/dsl/entry.rs` (already repointed in Task 4 — confirm `frame_regs_base` = `arena.base_mut_ptr().add(window_base)` where `window_base = cfr + HEADER_SLOTS`)
- Test: an offset-equality test mirroring `value_cells_base_offset_is_pinned`

- [ ] **Step 1: Add the constants + test**
```rust
// in reg_convention.rs
pub const FRAME_HEADER_CALLER_CFR: usize = core::mem::offset_of!(crate::frame_header::FrameHeader, caller_cfr);
pub const FRAME_HEADER_CODE: usize = core::mem::offset_of!(crate::frame_header::FrameHeader, code);
pub const FRAME_HEADER_CALLEE: usize = core::mem::offset_of!(crate::frame_header::FrameHeader, callee);
pub const FRAME_HEADER_THIS_VALUE: usize = core::mem::offset_of!(crate::frame_header::FrameHeader, this_value);
// (add the rest as SP-1 needs them)
```
Add a test in `frame_header.rs` asserting these equal the literal offsets locked in `frame_header_offsets_stable`.

- [ ] **Step 2: Run + commit**

Run: `cargo test -p lyng_vm --lib frame_header`
Expected: PASS.
```bash
git add crates/vm/src/dsl/reg_convention.rs crates/vm/src/frame_header.rs crates/vm/src/dsl/entry.rs
git commit -m "feat(vm): pin FrameHeader offset constants + arena-based regs base for SP-1

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 16: Targeted regression tests for the risky spots

Guard the four hard cases independently of Test262.

**Files:** test modules in `crates/vm/src/vm/state.rs`, `vm.rs`, `crates/ops/src/errors.rs`.

- [ ] **Step 1: Confirm/add the four tests** (some land earlier — keep one canonical copy each):
  1. **GC sole-ref in window survives** — `minor_gc_object_only_in_register_window_survives` (Task 10).
  2. **GC header callee survives** — `minor_gc_header_callee_survives` (Task 10).
  3. **No header slot mistraced as Value** — add: allocate a frame whose header's `code`/`callee` raw `u32` bit patterns, if misread as a `Value`, would point at a bogus object; run GC; assert no spurious mark/crash (run under the GC's debug verifier if one exists).
  4. **Cross-realm + multi-realm realm derivation** — `cross_realm_call_frame_uses_callee_realm` + `throw_in_secondary_realm_uses_that_realms_error_prototype` (Task 11).
  5. **Generator window round-trip** — `generator_snapshot_restore_round_trips_window_and_this` (Task 9).

- [ ] **Step 2: Run**

Run: `cargo test -p lyng_vm; cargo test -p lyng_ops`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "test: guard SP-0b hard cases (GC window/header, realm derivation, generator)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 17: Debug-assert the invariants

**Files:** `crates/vm/src/vm.rs` (push/unwind paths), `crates/vm/src/vm/registers.rs`.

- [ ] **Step 1: Add the asserts** (compiled out of release):
```rust
        debug_assert_eq!(
            self.current_cfr_opt().is_none(),
            self.frame_depth() == 0,
            "current_cfr empty iff depth 0"
        );
        debug_assert!(
            self.establishment_scopes_empty() || self.frame_depth() > 0,
            "establishment side-stack empty when no frames"
        );
        // after refresh_running_context:
        debug_assert_eq!(
            agent.running_context().map(|rc| rc.realm()),
            self.current_cfr_opt().map(|cfr| self.realm_of(agent, cfr)),
            "running_context realm tracks derive(current frame)"
        );
        // caller_cfr chain is strictly decreasing and bottoms at root:
        debug_assert!(self.caller_chain_well_formed(), "caller_cfr chain well-formed");
```
Add the small `establishment_scopes_empty`/`caller_chain_well_formed` helpers.

- [ ] **Step 2: Run the suite in debug**

Run: `cargo test -p lyng_vm`
Expected: PASS (no assert fires).

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "test(vm): debug-assert arena/cfr/running-context invariants

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 18: Perf gate (no regression on the V8 suite)

**Files:** none (measurement).

- [ ] **Step 1: Baseline** — check out the branch point (`main` at branch creation), run the V8 bench harness (Richards + RayTrace; see how prior commits report scores, e.g. the RayTrace A/B in project memory). Record scores.

- [ ] **Step 2: Branch** — `git checkout feat/sp0b-unified-register-file-frame-arena`, run the same bench.

- [ ] **Step 3: Compare** — register access moved from `Vec<Value>` indexing to `Box<[Value]>` indexing (equivalent); the overlay adds a pointer cast per header read; window len is now derived from code per trace/finish. Expected: **flat** (within noise). If Richards/RayTrace regress beyond noise:
  - Confirm `read_register_unchecked`/`write_register_unchecked` still elide bounds checks on the arena slice (they should — same `get_unchecked`).
  - Confirm `window_len_for(code)` is not doing a heap lookup on the hot return path; cache the window len in the header (`arg_count` slot already exists — repurpose or add a `window_len` field within slot 3's padding) if it shows up.
  - Re-measure.

- [ ] **Step 4: Record** results in the task notes. Commit only if a perf fix was needed (`perf(vm):` message).

---

## Self-Review (completed by plan author)

- **Spec coverage:** §1 arena absorption (Tasks 1,4,5,13); §2 tiered repr(C) header + offset test (Tasks 2,15) + cold table (Task 3); §3 GC per-window walk (Task 10); §4 realm/referrer derivation via merged establishment side-stack (Task 11); §5 soft-limit prologue + reservation (Tasks 1,4,5,12); §6 bridge-first migration (Tasks 4–9,11,13 each green); §7 testing/perf/asserts (Tasks 14,16,17,18); commit sequence steps 1→7 map to Groups 1→7. SP-1 offset pinning prepped (Task 15). Out-of-scope items (asm, arg-VR layout, lazy-commit, referrer-correctness fix) are not tasks, per the spec.
- **Placeholders:** test-harness specifics ("reuse the call harness", "the multi-realm test helper", `run_script`/`harness()`) and a few "confirm the exact `lyng_types` raw constructor name" notes are deliberate and match the SP-0a plan's accepted style — the existing mid-execution inspection + multi-realm harnesses must be read from the current test modules. Every non-test source change shows full code or an exact representative diff + the precise site list.
- **Type consistency:** `FrameArena::{new,base_ptr,base_mut_ptr,top,slots,slots_mut,bump,release_to,set_top,try_grow}`; `FrameHeader::{zeroed,caller_cfr,set_caller_cfr,saved_pc,set_saved_pc,code,set_code,callee,set_callee,this_value,this_state,set_this,set_this_value,variable_env,set_variable_env,lexical_env,set_lexical_env,private_env,set_private_env,new_target,set_new_target,construct_this,set_construct_this,arg_count,set_arg_count,flags_bits,set_flags_bits,kind_raw,set_kind_raw,return_register,set_return_register}`; `HEADER_SLOTS`; `FrameColdTable::{new,reset_at,get,get_mut}` + `FrameColdState`; `Vm::{frame_header,frame_header_mut,current_frame_header,current_cfr_opt,frame_depth,current_cold,reserve_frame,write_header_from_record,realm_of,frame_cfrs,window_len_for}` used consistently across tasks.
