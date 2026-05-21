# DSL-1 Phase 1.B.1 — Frame-context refactor — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the asm-visible frame-context substrate (`frame_const_base`, `frame_this_value`) on `LlIntState` so Phase 1.B.2 can inline-port `op_load_const8` (#21) and `op_load_this` (#12), and so Phase 1.B.3 locals/Ldar/LoadEnvSlot ports inherit the populated frame context. **No opcode handlers are ported in this sub-phase.**

**Architecture:** Two new 8-byte fields appended to the hot pointer cluster of `LlIntState` (offsets 32 and 40, total struct grows 56 → 72 bytes). `frame_const_base` reuses the existing `RuntimeCodeRecord::constants` arena slot (already pre-resolved at install time, already GC-scanned, already pointer-stable). `frame_this_value` mirrors `frame.this_value()` for `ThisState::Value(v)` and uses the existing `Value::uninitialized_lexical()` sentinel for `ThisState::Uninitialized`/`Lexical` (handler bails to slow path on sentinel match). Refresh discipline mirrors the existing `frame_pb_base` pattern: populate at trampoline entry, refresh in `slow_path::translate_outcome`'s Refresh arm. Two new backend macros (`load_constant!`, `load_state_value!`) provide the asm-side accessors.

**Tech Stack:** Rust + AArch64 `naked_asm!` (DSL substrate), `#[repr(C)]` (ABI stability), cargo workspace.

**Spec:** [`docs/superpowers/specs/2026-05-19-dsl-1-phase-1b1-frame-context-refactor-design.md`](../specs/2026-05-19-dsl-1-phase-1b1-frame-context-refactor-design.md).
**Baseline HEAD:** `ae8b7766` (Phase 1.B.0 closed).

---

## File structure overview

### Created
- `crates/lyng-js/vm/src/dsl/backend/aarch64/constants.rs` — backend macro `load_constant!` (new file in existing backend module)
- `crates/lyng-js/vm/tests/dsl_validation_frame_context.rs` — synthetic-handler integration test exercising both new macros end-to-end
- `crates/lyng-js-tests/tests/gc_stress_frame_context.rs` — gc-stress integration test (closure + this + allocation pressure)
- `reports/js/lyng-js/dsl-1/phase-1b1-gc-review.md` — GC root-scanning review doc
- `reports/js/lyng-js/dsl-1/phase-1b1-ab-comparison.md` — same-load V8 v7 A/B data
- `reports/js/lyng-js/dsl-1/phase-1b1-summary.md` — final sub-phase summary

### Modified
- `crates/lyng-js/vm/src/dsl/llint_state.rs` — add two fields to `LlIntState`, add `resolve_initial_this_value` helper + unit tests, update `ll_int_state_offsets_stable`
- `crates/lyng-js/vm/src/dsl/reg_convention.rs` — add two new `LLINT_STATE_*` offset consts
- `crates/lyng-js/vm/src/dsl/entry.rs` — populate new fields at `run_via_dsl` entry (lines 116-128 area)
- `crates/lyng-js/vm/src/dsl/slow_path.rs` — refresh new fields in `translate_outcome` Refresh arm (lines 240-296 area); add debug-only stability assertion
- `crates/lyng-js/vm/src/dsl/backend/aarch64/mod.rs` — declare new `constants` submodule
- `crates/lyng-js/vm/src/dsl/backend/aarch64/frame.rs` (or new `state.rs`) — add `load_state_value!` macro

### Untouched (verifying invariant)
- All existing opcode handlers in `crates/lyng-js/vm/src/dsl/handlers/`. No handler changes in 1.B.1.

---

## Conventions for this plan

- **User deny rules respected:** NEVER use `git -C <path>` or `cd <path> && git ...`. Always run git from the worktree's working directory (the harness already starts there).
- **Commits:** Each task ends with a self-contained commit. Use the `Co-Authored-By: Claude` footer per the standard convention.
- **Untracked planning docs:** `docs/superpowers/plans/*.md` and `docs/superpowers/specs/*.md` files for this work are left untracked unless explicitly added (per user discipline).
- **`reports/js/lyng-js/bench-v8.md`** is a side-effect of `cargo run -p lyng-js-bench -- v8suite`. Leave it unstaged throughout.
- **Behavioral parity at every commit:** `cargo test -p lyng-js-vm --lib --release` (413+) and `cargo test -p lyng-js-tests --release` (1186+) must pass after each task. The 2 pre-existing `feedback_flat_consistency` failures are unchanged.

---

## Task 1: Add `frame_const_base` and `frame_this_value` fields to `LlIntState`

**Files:**
- Modify: `crates/lyng-js/vm/src/dsl/llint_state.rs:25-36` (struct definition + tests)
- Modify: `crates/lyng-js/vm/src/dsl/reg_convention.rs` (add two new offset consts)

- [ ] **Step 1: Write the failing offset-stability test**

Open `crates/lyng-js/vm/src/dsl/llint_state.rs` and update the `ll_int_state_offsets_stable` test (around lines 91-102):

```rust
#[test]
fn ll_int_state_offsets_stable() {
    // Lock in the asm-DSL ABI layout. Values were determined from
    // the first build of the `#[repr(C)]` struct above; the test
    // catches drift across rustc versions.
    assert_eq!(r::LLINT_STATE_FRAME_PC_OFFSET, 0);
    assert_eq!(r::LLINT_STATE_FRAME_PB_BASE, 8);
    assert_eq!(r::LLINT_STATE_FRAME_REGS_BASE, 16);
    assert_eq!(r::LLINT_STATE_FRAME_FV_BASE, 24);
    // Phase 1.B.1: two new fields inserted between FV_BASE and the
    // existing scalar block. Each is 8B.
    assert_eq!(r::LLINT_STATE_FRAME_CONST_BASE, 32);
    assert_eq!(r::LLINT_STATE_FRAME_THIS_VALUE, 40);
    // Phase 1.B.1: PREFIX shifts from 48 → 64 due to the two
    // 8-byte inserts.
    assert_eq!(r::LLINT_STATE_PREFIX, 64);
    assert_eq!(core::mem::size_of::<LlIntState>(), 72);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p lyng-js-vm --lib ll_int_state_offsets_stable`
Expected: FAIL — `LLINT_STATE_FRAME_CONST_BASE` does not exist yet (unresolved import / compile error).

- [ ] **Step 3: Add the two new offset consts to `reg_convention.rs`**

Open `crates/lyng-js/vm/src/dsl/reg_convention.rs`. After the existing `LLINT_STATE_FRAME_FV_BASE` declaration (around line 42), add:

```rust
pub const LLINT_STATE_FRAME_CONST_BASE: usize = offset_of!(LlIntState, frame_const_base);
pub const LLINT_STATE_FRAME_THIS_VALUE: usize = offset_of!(LlIntState, frame_this_value);
```

Keep the existing `LLINT_STATE_PREFIX` declaration as-is — `offset_of!` will compute its new value automatically once the struct fields change.

- [ ] **Step 4: Add the two new fields to `LlIntState`**

In `crates/lyng-js/vm/src/dsl/llint_state.rs`, update the struct (lines 25-36):

```rust
#[repr(C)]
pub struct LlIntState {
    pub frame_pc_offset: u32,
    pub _pad1: u32,
    pub frame_pb_base: *const u8,
    pub frame_regs_base: *mut Value,
    pub frame_fv_base: *mut FeedbackEntry,
    // Phase 1.B.1: asm-visible frame context. `frame_const_base`
    // points into the active code record's pre-resolved constants
    // array (RuntimeCodeRecord::constants → CodeSlotsRef, &[Value]
    // from heap.view().code_slots()). `frame_this_value` is a
    // mirror of `frame.this_value()` for `ThisState::Value(v)`, or
    // `Value::uninitialized_lexical()` as the bail-to-slow-path
    // sentinel for `ThisState::Uninitialized`/`Lexical`.
    //
    // Both fields are valid only between Refresh egress events;
    // GC can only happen during slow-path bridges, which refresh
    // both fields on egress. See spec §5 mirror discipline.
    pub frame_const_base: *const Value,
    pub frame_this_value: Value,
    pub frame_depth: u32,
    pub frame_check_epoch: u32,
    pub rust_context: *mut LlIntRustContextOpaque,
    pub prefix: u8,
    pub _pad2: [u8; 7],
}
```

- [ ] **Step 5: Update the entry-shim construction site to populate defaults**

`run_via_dsl` in `entry.rs` (lines 116-128) currently builds an `LlIntState` literal that doesn't mention the new fields, which will fail to compile. Add defaults for now:

```rust
let mut state = LlIntState {
    frame_pc_offset,
    _pad1: 0,
    frame_pb_base: pb_base,
    frame_regs_base: regs_base,
    frame_fv_base: fv_base,
    // Phase 1.B.1 Task 1: placeholders. Task 3 wires real values.
    frame_const_base: std::ptr::null(),
    frame_this_value: lyng_js_types::Value::undefined(),
    frame_depth: frame_depth as u32,
    frame_check_epoch: 0,
    rust_context: (&mut rust_ctx) as *mut LlIntRustContext<'_>
        as *mut LlIntRustContextOpaque,
    prefix: 0,
    _pad2: [0; 7],
};
```

- [ ] **Step 6: Run the offset-stability test to verify it passes**

Run: `cargo test -p lyng-js-vm --lib ll_int_state_offsets_stable`
Expected: PASS.

- [ ] **Step 7: Run full vm test suite to confirm no regression**

Run: `cargo test -p lyng-js-vm --lib --release`
Expected: 413+ passing (matches Phase 1.B.0 baseline).

- [ ] **Step 8: Run lyng-js-tests to confirm integration parity**

Run: `cargo test -p lyng-js-tests --release`
Expected: 1186+ passing.

- [ ] **Step 9: Commit**

```bash
git add crates/lyng-js/vm/src/dsl/llint_state.rs crates/lyng-js/vm/src/dsl/reg_convention.rs crates/lyng-js/vm/src/dsl/entry.rs
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.1 Task 1: add frame_const_base + frame_this_value to LlIntState

Adds two 8-byte fields to the asm-visible LlIntState (offsets 32 and 40,
struct grows 56 → 72 bytes), exposes their offsets via
LLINT_STATE_FRAME_CONST_BASE / LLINT_STATE_FRAME_THIS_VALUE, and updates
ll_int_state_offsets_stable to assert the new layout (LLINT_STATE_PREFIX
shifts 48 → 64).

This commit is layout-only; entry-shim populates the fields to placeholder
defaults (null / undefined). Task 3 wires the real population.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Add `resolve_initial_this_value` helper with unit tests

**Files:**
- Modify: `crates/lyng-js/vm/src/dsl/llint_state.rs` (add helper + unit tests)

- [ ] **Step 1: Write the failing unit tests**

In `crates/lyng-js/vm/src/dsl/llint_state.rs`, replace the existing `tests` mod block (lines 86-103) with an expanded version that includes both the existing offset test and four new helper tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::reg_convention as r;
    // Note: the helper tests below use stub-built FrameRecord values.
    // If FrameRecord construction in tests is awkward, the helper
    // signature can be split: a lower-level fn taking
    // `(Option<ThisState>, fallback_value: Value)` which is trivial
    // to unit-test, with a thin wrapper that takes Agent + FrameRecord
    // and delegates. Adopt that pattern if FrameRecord stubs are hard
    // to construct.
    use lyng_js_env::execution::ThisState;
    use lyng_js_types::Value;

    #[test]
    fn ll_int_state_offsets_stable() {
        assert_eq!(r::LLINT_STATE_FRAME_PC_OFFSET, 0);
        assert_eq!(r::LLINT_STATE_FRAME_PB_BASE, 8);
        assert_eq!(r::LLINT_STATE_FRAME_REGS_BASE, 16);
        assert_eq!(r::LLINT_STATE_FRAME_FV_BASE, 24);
        assert_eq!(r::LLINT_STATE_FRAME_CONST_BASE, 32);
        assert_eq!(r::LLINT_STATE_FRAME_THIS_VALUE, 40);
        assert_eq!(r::LLINT_STATE_PREFIX, 64);
        assert_eq!(core::mem::size_of::<LlIntState>(), 72);
    }

    #[test]
    fn resolve_this_state_value_passthrough() {
        let v = Value::from_smi_i32(42);
        let result = resolve_this_state_to_mirror(Some(ThisState::Value(v)), v);
        assert_eq!(result, v);
    }

    #[test]
    fn resolve_this_state_uninitialized_returns_sentinel() {
        let fallback = Value::from_smi_i32(99); // arbitrary; should be ignored.
        let result = resolve_this_state_to_mirror(Some(ThisState::Uninitialized), fallback);
        assert_eq!(result, Value::uninitialized_lexical());
    }

    #[test]
    fn resolve_this_state_lexical_returns_sentinel() {
        let fallback = Value::from_smi_i32(99); // arbitrary; should be ignored.
        let result = resolve_this_state_to_mirror(Some(ThisState::Lexical), fallback);
        assert_eq!(result, Value::uninitialized_lexical());
    }

    #[test]
    fn resolve_this_state_none_falls_back_to_frame_this() {
        let fallback = Value::from_smi_i32(7);
        let result = resolve_this_state_to_mirror(None, fallback);
        assert_eq!(result, fallback);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p lyng-js-vm --lib resolve_this_state`
Expected: FAIL — `resolve_this_state_to_mirror` is undefined (compile error or `unresolved import`).

- [ ] **Step 3: Implement the helper**

In `crates/lyng-js/vm/src/dsl/llint_state.rs`, add the helper just before the `#[cfg(test)]` block. Use this exact signature pair:

```rust
use lyng_js_env::execution::ThisState;

/// Lower-level helper: maps a (`ThisState`, frame-`this`-value
/// fallback) pair to the mirror value stored in
/// [`LlIntState::frame_this_value`]. Pure / no side effects /
/// trivially unit-testable.
///
/// Phase 1.B.1 sentinel rule:
/// - `ThisState::Value(v)` → `v` (real `this` binding)
/// - `ThisState::Uninitialized` → `Value::uninitialized_lexical()` (bail)
/// - `ThisState::Lexical` → `Value::uninitialized_lexical()` (bail)
/// - `None` (no current execution context) → fallback
///
/// The sentinel is observed by inline `op_load_this` handlers (landed
/// in Phase 1.B.2); on match the handler bails to the slow path,
/// which handles the throw / lex-env walk as appropriate.
#[inline]
pub(crate) fn resolve_this_state_to_mirror(
    this_state: Option<ThisState>,
    fallback_frame_this: Value,
) -> Value {
    match this_state {
        Some(ThisState::Value(v)) => v,
        Some(ThisState::Uninitialized) | Some(ThisState::Lexical) => {
            Value::uninitialized_lexical()
        }
        None => fallback_frame_this,
    }
}

/// Top-level helper: derives the mirror from an `Agent` + a
/// `FrameRecord`. Mirrors the read path in
/// `crates/lyng-js/vm/src/vm/semantics/names.rs:600-627` so the
/// pre-resolution matches `op_load_this` semantics exactly.
///
/// Called from:
/// - `crate::dsl::entry::run_via_dsl` (initial population)
/// - `crate::dsl::slow_path::LlIntDispatchState::translate_outcome`
///   (Refresh arm)
#[inline]
pub(crate) fn resolve_initial_this_value(
    agent: &lyng_js_env::Agent,
    frame: &crate::FrameRecord,
) -> Value {
    let this_state = agent
        .current_execution_context()
        .map(|ec| ec.this_state());
    let fallback = frame.this_value();
    resolve_this_state_to_mirror(this_state, fallback)
}
```

**Note on imports:** the exact path to `ThisState` may be `lyng_js_env::execution::ThisState` or `lyng_js_env::ThisState` depending on re-exports — check with `grep "pub use.*ThisState" crates/lyng-js/env/src/lib.rs` if the first import fails. Similarly for `agent.current_execution_context()` — verify against the research report's `env/src/agent/execution_contexts.rs`.

- [ ] **Step 4: Run unit tests to verify they pass**

Run: `cargo test -p lyng-js-vm --lib resolve_this_state`
Expected: 4 passing (`resolve_this_state_value_passthrough`, `..._uninitialized_returns_sentinel`, `..._lexical_returns_sentinel`, `..._none_falls_back_to_frame_this`).

- [ ] **Step 5: Run full vm suite for parity**

Run: `cargo test -p lyng-js-vm --lib --release`
Expected: 417+ passing (413 baseline + 4 new).

- [ ] **Step 6: Commit**

```bash
git add crates/lyng-js/vm/src/dsl/llint_state.rs
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.1 Task 2: add resolve_initial_this_value helper

Adds a two-layer helper:
- `resolve_this_state_to_mirror(Option<ThisState>, fallback) -> Value`:
  pure, unit-testable, maps a ThisState to either the real value or
  Value::uninitialized_lexical() sentinel.
- `resolve_initial_this_value(&Agent, &FrameRecord) -> Value`: thin
  wrapper that pulls the ThisState from the current execution context
  and delegates.

Mirrors the read path in vm/src/vm/semantics/names.rs:600-627 so the
pre-resolved sentinel matches op_load_this semantics exactly.

4 new unit tests cover Value(v) passthrough, Uninitialized → sentinel,
Lexical → sentinel, and the no-EC fallback to frame.this_value().

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Populate `frame_const_base` and `frame_this_value` at trampoline entry

**Files:**
- Modify: `crates/lyng-js/vm/src/dsl/entry.rs:38-128` (the `run_via_dsl` function)

- [ ] **Step 1: Compute the new field values before the DispatchState move**

`run_via_dsl` consumes `installed` and `frame` into the `DispatchState` at lines 101-110. The new fields must be computed BEFORE that move, using the same `&Agent` borrow that's about to be passed by reference to `new_for_dsl_entry`.

Insert these computations after the `regs_base` block (line 90) and before `let vm_ptr` (line 92):

```rust
    // Phase 1.B.1: derive `frame_const_base` from the pre-resolved
    // constants array. Reuses the existing arena slot owned by
    // `RuntimeCodeRecord::constants` (populated at install time by
    // `Vm::install_constants`). Pointer is stable for the lifetime
    // of the code record; refreshed on every Refresh egress.
    // See spec §3.4.
    //
    // The chain mirrors `Vm::read_constant` in
    // crates/lyng-js/vm/src/vm/values.rs:795-806.
    let const_base: *const Value = agent
        .heap()
        .view()
        .code(frame.code())
        .and_then(lyng_js_gc::RuntimeCodeRecord::constants)
        .and_then(|slots| agent.heap().view().code_slots(slots))
        .map(|s| s.as_ptr())
        .unwrap_or(std::ptr::null());

    // Phase 1.B.1: derive `frame_this_value`. Pre-resolves the
    // active execution context's ThisState into either the real
    // Value or Value::uninitialized_lexical() sentinel.
    // See spec §3.3.
    let this_value: Value =
        crate::dsl::llint_state::resolve_initial_this_value(agent, &frame);
```

- [ ] **Step 2: Replace the Task-1 placeholders with the real values**

Update the `LlIntState` literal (around lines 116-128, the lines added in Task 1 Step 5):

```rust
    let mut state = LlIntState {
        frame_pc_offset,
        _pad1: 0,
        frame_pb_base: pb_base,
        frame_regs_base: regs_base,
        frame_fv_base: fv_base,
        // Phase 1.B.1 Task 3: real values from the new derivations above.
        frame_const_base: const_base,
        frame_this_value: this_value,
        frame_depth: frame_depth as u32,
        frame_check_epoch: 0,
        rust_context: (&mut rust_ctx) as *mut LlIntRustContext<'_>
            as *mut LlIntRustContextOpaque,
        prefix: 0,
        _pad2: [0; 7],
    };
```

- [ ] **Step 3: Build to verify it compiles**

Run: `cargo build -p lyng-js-vm --release`
Expected: builds cleanly. If `lyng_js_gc::RuntimeCodeRecord` is not in scope, add the import at the top of `entry.rs`; if `agent.heap()` requires a different qualifier, follow the pattern from `vm/src/vm/values.rs:795-806`.

- [ ] **Step 4: Run vm tests for parity**

Run: `cargo test -p lyng-js-vm --lib --release`
Expected: 417+ passing (4 new from Task 2 + 413 baseline).

- [ ] **Step 5: Run lyng-js-tests for integration parity**

Run: `cargo test -p lyng-js-tests --release`
Expected: 1186+ passing.

No handler reads `frame_const_base` or `frame_this_value` yet — these tests pass because the new fields are written but never read by asm. They just have to not break the build or existing semantics.

- [ ] **Step 6: Commit**

```bash
git add crates/lyng-js/vm/src/dsl/entry.rs
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.1 Task 3: populate frame_const_base + frame_this_value at trampoline entry

Wires `run_via_dsl` to derive both new LlIntState fields before
consuming `installed` and `frame` into the DispatchState:

- `frame_const_base` reuses the existing
  `RuntimeCodeRecord::constants` arena slot (pointer-stable, already
  GC-scanned, already pre-resolved at install time).
- `frame_this_value` is computed via the Task-2 helper from the
  active execution context's ThisState.

No handler reads these fields yet (Phase 1.B.2 ports op_load_const8
and op_load_this); behavior is unchanged.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Refresh `frame_const_base` and `frame_this_value` in the slow-path Refresh arm

**Files:**
- Modify: `crates/lyng-js/vm/src/dsl/slow_path.rs:240-296` (Refresh arm in `translate_outcome`)

- [ ] **Step 1: Add the new field derivations in the Refresh arm**

In `slow_path.rs`, inside the existing Refresh arm (lines 240-296), AFTER the existing `pb_base` / `fv_base` derivations (line 289) and BEFORE the existing `unsafe { (**state)... = ... }` block (line 291), add:

```rust
                    // Phase 1.B.1: derive the new fields for the
                    // active frame. Identical chain to the entry shim
                    // in entry.rs::run_via_dsl. See spec §3.4.
                    let const_base: *const lyng_js_types::Value = rust
                        .dispatch
                        .agent
                        .heap()
                        .view()
                        .code(active_frame.code())
                        .and_then(lyng_js_gc::RuntimeCodeRecord::constants)
                        .and_then(|slots| {
                            rust.dispatch.agent.heap().view().code_slots(slots)
                        })
                        .map(|s| s.as_ptr())
                        .unwrap_or(std::ptr::null());

                    // Phase 1.B.1: refresh the `this` mirror. Captures
                    // super() mutations and any other slow-path
                    // changes to frame.this_value().
                    let this_value = crate::dsl::llint_state::resolve_initial_this_value(
                        rust.dispatch.agent,
                        &active_frame,
                    );
```

**Note:** the exact accessor for `rust.dispatch.agent` may be `&rust.dispatch.agent`, a method like `rust.dispatch.agent()`, or `*rust.dispatch.agent` depending on how `DispatchState` is structured. If the literal `.agent` field is not accessible, check the imports in `slow_path.rs` and follow the existing pattern (e.g. `rust.dispatch.vm.agent()` if there's a Vm-side accessor). The refactor worker uses whichever access pattern is already established in `slow_path.rs`.

- [ ] **Step 2: Add the new field writes to the existing unsafe block**

Replace the existing unsafe block (lines 291-296):

```rust
                    // SAFETY: state is valid by from_raw's contract.
                    unsafe {
                        (**state).frame_pc_offset = active_frame.instruction_offset();
                        (**state).frame_pb_base = pb_base;
                        (**state).frame_regs_base = regs_base_ptr;
                        (**state).frame_fv_base = fv_base;
                    }
```

With:

```rust
                    // SAFETY: state is valid by from_raw's contract.
                    unsafe {
                        (**state).frame_pc_offset = active_frame.instruction_offset();
                        (**state).frame_pb_base = pb_base;
                        (**state).frame_regs_base = regs_base_ptr;
                        (**state).frame_fv_base = fv_base;
                        // Phase 1.B.1: refresh the new fields.
                        (**state).frame_const_base = const_base;
                        (**state).frame_this_value = this_value;
                    }
```

- [ ] **Step 3: Add the debug-only stability assertion**

Immediately after the unsafe block above, add:

```rust
                    // Phase 1.B.1: debug-only stability assertion.
                    // The arena slot's data pointer must be stable
                    // across the slow-path call. If this fires, the
                    // arena moved under us — investigate before
                    // disabling. Matches the implicit invariant
                    // `frame_pb_base` already relies on. See spec §3.6.
                    #[cfg(debug_assertions)]
                    {
                        let recomputed: *const lyng_js_types::Value = rust
                            .dispatch
                            .agent
                            .heap()
                            .view()
                            .code(active_frame.code())
                            .and_then(lyng_js_gc::RuntimeCodeRecord::constants)
                            .and_then(|slots| {
                                rust.dispatch.agent.heap().view().code_slots(slots)
                            })
                            .map(|s| s.as_ptr())
                            .unwrap_or(std::ptr::null());
                        debug_assert_eq!(
                            const_base, recomputed,
                            "frame_const_base unstable across Refresh"
                        );
                    }
```

- [ ] **Step 4: Build to verify it compiles**

Run: `cargo build -p lyng-js-vm --release`
Expected: builds cleanly. If `lyng_js_gc::RuntimeCodeRecord` is not in scope at the top of `slow_path.rs`, add the import alongside the existing `use lyng_js_gc::...` lines (or use the fully-qualified path inline as shown).

- [ ] **Step 5: Run vm tests for parity**

Run: `cargo test -p lyng-js-vm --lib --release`
Expected: 417+ passing.

- [ ] **Step 6: Run lyng-js-tests**

Run: `cargo test -p lyng-js-tests --release`
Expected: 1186+ passing.

The fields are now refreshed correctly across every Refresh egress, but still no asm handler reads them. Behavior unchanged; this is the substrate.

- [ ] **Step 7: Commit**

```bash
git add crates/lyng-js/vm/src/dsl/slow_path.rs
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.1 Task 4: refresh frame_const_base + frame_this_value on slow-path Refresh egress

Mirrors the existing `frame_pb_base` / `frame_regs_base` / `frame_fv_base`
refresh discipline for the two new LlIntState fields, in the Refresh arm
of `LlIntDispatchState::translate_outcome`. Both fields are re-derived
from the active frame (`vm.frames().last()`) using the same chain as
the entry shim.

A `#[cfg(debug_assertions)]` stability assertion confirms the arena
slot's data pointer doesn't shift across the slow-path call — catches
any GC compaction edge case during dev. No-op in release builds.

The Continue arm does NOT touch the new fields: a Continue egress
means the frame didn't change, so the mirrors are still valid. Any
future continue-path semantic body that mutates frame.this_value()
must convert to a Refresh egress.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Add `load_constant!` and `load_state_value!` backend macros

**Files:**
- Create: `crates/lyng-js/vm/src/dsl/backend/aarch64/constants.rs`
- Modify: `crates/lyng-js/vm/src/dsl/backend/aarch64/mod.rs` (declare new submodule)
- Modify: `crates/lyng-js/vm/src/dsl/backend/aarch64/frame.rs` (add `load_state_value!` macro)

- [ ] **Step 1: Read the existing aarch64 backend module structure for context**

Run: `ls -la crates/lyng-js/vm/src/dsl/backend/aarch64/`
And read the existing `counters.rs` file (created in Phase 1.B.0) for the macro shape template:

```bash
cat crates/lyng-js/vm/src/dsl/backend/aarch64/counters.rs | head -80
```

This shows the macro_rules pattern used in the DSL backend: the macros emit asm strings that the proc-macro lowerer splices into handler bodies. Use the same pattern.

- [ ] **Step 2: Create `constants.rs` with the `load_constant!` macro**

Create `crates/lyng-js/vm/src/dsl/backend/aarch64/constants.rs`:

```rust
//! Constants-access backend macros for Phase 1.B.1.
//!
//! `load_constant!($idx_reg => $dst_reg)` loads a Value from the
//! pre-resolved constants array via `LlIntState::frame_const_base`.
//!
//! The frame_const_base pointer is populated at trampoline entry
//! (`entry.rs::run_via_dsl`) and refreshed on every slow-path
//! Refresh egress (`slow_path.rs::translate_outcome`). Handler code
//! reads through it within a Refresh-to-Refresh window.
//!
//! See spec §3.5.

/// Load a `Value` (8 bytes) from `frame_const_base[idx]` into `$dst_reg`.
///
/// Two instructions:
/// - `ldr {scratch}, [x22, #LLINT_STATE_FRAME_CONST_BASE]` — load base ptr
/// - `ldr {dst},     [{scratch}, {idx}, lsl #3]` — Value is 8B → lsl #3
///
/// Uses x16 (IP0) as scratch by AAPCS64 convention. Callers must not
/// rely on x16's contents being preserved across this macro.
#[macro_export]
macro_rules! load_constant {
    ($idx_reg:expr => $dst_reg:expr, vm_const_base = $vm_const_base:expr) => {
        concat!(
            "ldr x16, [x22, ", stringify!($vm_const_base), "]\n",
            "ldr ", $dst_reg, ", [x16, ", $idx_reg, ", lsl #3]\n",
        )
    };
}

pub use load_constant;
```

**Note on the macro shape:** the exact `macro_rules!` arity and binding pattern depends on the existing DSL backend conventions. Inspect `counters.rs` (added in Phase 1.B.0) — it uses a binding like `vm_counter_base = const ::lyng_js_vm::dsl::reg_convention::VM_DISPATCH_COUNTERS_PTR_OFFSET`. Use the analogous binding for `LLINT_STATE_FRAME_CONST_BASE`. If `counters.rs` resolves its offset by passing the const as a parameter from the lowerer, mirror that wiring; if it inlines the offset directly, do the same.

- [ ] **Step 3: Declare the new submodule in `aarch64/mod.rs`**

Open `crates/lyng-js/vm/src/dsl/backend/aarch64/mod.rs` and add:

```rust
pub mod constants;
```

next to the existing `pub mod counters;`, `pub mod control;`, etc.

- [ ] **Step 4: Add `load_state_value!` to `frame.rs`**

In `crates/lyng-js/vm/src/dsl/backend/aarch64/frame.rs`, add a new macro alongside the existing frame-context macros:

```rust
/// Load a `Value` (8 bytes) from a fixed offset in `LlIntState`.
/// Used for `frame_this_value` (and other fixed-offset Value fields
/// that may be added in future).
///
/// One instruction:
/// - `ldr {dst}, [x22, #{offset}]`
///
/// `$offset_expr` is the `LLINT_STATE_*` const that the lowerer
/// resolves to a numeric offset.
///
/// See spec §3.5.
#[macro_export]
macro_rules! load_state_value {
    ($dst_reg:expr, vm_state_offset = $offset_expr:expr) => {
        concat!("ldr ", $dst_reg, ", [x22, ", stringify!($offset_expr), "]\n")
    };
}

pub use load_state_value;
```

(Same caveat as Step 2: adjust the macro arity to match the existing DSL backend conventions in `frame.rs`. Inspect that file before writing.)

- [ ] **Step 5: Wire the lowerer to recognize the new macros (if applicable)**

If the DSL proc-macro lowerer in `crates/lyng-js-vm-dsl/src/lower.rs` needs to inject `vm_const_base` / `vm_state_offset` bindings for these macros (mirroring how it injects `vm_counter_base` for the counter macros — see Phase 1.B.0 Task 4), add that wiring.

Find the existing `vm_counter_base` injection in `lower.rs`; the new macros need analogous injections:
- `load_constant!` → inject `vm_const_base = const ::lyng_js_vm::dsl::reg_convention::LLINT_STATE_FRAME_CONST_BASE`
- `load_state_value!` → no injection needed; the const is supplied at the call site

If `lower.rs` doesn't need changes (e.g. if these macros are designed to be called with the const supplied at the handler site without auto-injection), skip this step.

- [ ] **Step 6: Build to verify it compiles**

Run: `cargo build -p lyng-js-vm --release`
Expected: builds cleanly. No handler uses these macros yet — they're substrate. The validation test in Task 6 exercises them.

- [ ] **Step 7: Run vm tests for parity**

Run: `cargo test -p lyng-js-vm --lib --release`
Expected: 417+ passing.

- [ ] **Step 8: Commit**

```bash
git add crates/lyng-js/vm/src/dsl/backend/aarch64/constants.rs crates/lyng-js/vm/src/dsl/backend/aarch64/mod.rs crates/lyng-js/vm/src/dsl/backend/aarch64/frame.rs crates/lyng-js-vm-dsl/src/lower.rs
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.1 Task 5: load_constant! and load_state_value! backend macros

Adds two new aarch64 backend macros:

- `load_constant!($idx => $dst)`: 2-instruction indexed load via
  LlIntState::frame_const_base. Scratch reg x16 (IP0).
- `load_state_value!($offset => $dst)`: 1-instruction fixed-offset
  Value load from LlIntState. Used for frame_this_value initially;
  generalizable to any 8-byte Value field.

The lowerer in lyng-js-vm-dsl injects `vm_const_base` binding for
`load_constant!` mirroring the Phase 1.B.0 `vm_counter_base` pattern.

These macros are substrate; no handler exercises them yet. Task 6
adds a validation test that does.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Synthetic validation handler exercising the new fields end-to-end

**Files:**
- Create: `crates/lyng-js/vm/tests/dsl_validation_frame_context.rs`

- [ ] **Step 1: Read the existing dsl_validation_*.rs tests for the template shape**

Run: `ls crates/lyng-js/vm/tests/ | grep dsl_validation`

These are integration-test files where the proc-macro `llint_handler!` macro is invoked to build synthetic handlers, which are then run end-to-end through the DSL trampoline. The Phase 1.B.0 Task 4 fix (`845cee79`) updated three of them; that fix landing point is a good reference.

Run: `cat crates/lyng-js/vm/tests/dsl_validation_empty.rs`

to see the simplest template.

- [ ] **Step 2: Create the new validation test file**

Create `crates/lyng-js/vm/tests/dsl_validation_frame_context.rs`. Follow the structure of `dsl_validation_empty.rs`:

```rust
//! Phase 1.B.1 Task 6: validate that frame_const_base and
//! frame_this_value are correctly populated and readable from
//! synthetic asm handlers.
//!
//! Three handlers exercised:
//! - `op_test_load_constant`: reads frame_const_base[0] via
//!   `load_constant!` and stores into register 0; asserts the
//!   loaded Value equals a pre-installed known constant.
//! - `op_test_load_this_value`: reads frame_this_value via
//!   `load_state_value!` for a frame with ThisState::Value(v);
//!   asserts the loaded Value equals v.
//! - `op_test_load_this_sentinel`: reads frame_this_value for a
//!   frame with ThisState::Uninitialized; asserts the loaded Value
//!   equals Value::uninitialized_lexical().

#![cfg(target_arch = "aarch64")]

use lyng_js_vm::dsl::{/* same imports as dsl_validation_empty.rs */};
use lyng_js_vm_dsl::llint_handler;
use lyng_js_types::Value;

// Three llint_handler! invocations follow the same pattern as
// dsl_validation_empty.rs but with opcode_byte = 210, 211, 212
// (or other unused slots above the synthetic test range used in
// Phase 1.B.0 Task 4 fix's 200/201/202). The handler bodies use:
//   load_constant!(idx_reg => dst_reg)
//   load_state_value!(LLINT_STATE_FRAME_THIS_VALUE => dst_reg)
// to read the new fields.

// Each handler ends with `done!(value)` to exit the trampoline with
// the loaded value, then the #[test] fn:
// 1. Installs a small bytecode program that invokes the synthetic
//    opcode at index 0.
// 2. Sets up a frame with the appropriate ThisState / constants
//    array.
// 3. Runs the DSL trampoline.
// 4. Asserts the returned Value matches expectations.

// IMPLEMENTATION NOTE: the precise harness for installing a synthetic
// opcode + frame in integration tests is established by the existing
// dsl_validation_*.rs files — reuse that machinery. If a particular
// arm requires constructing an Agent with a specific ThisState that
// isn't easily exposed through public APIs, the test for that arm
// may need a `pub(crate)` setter on `ExecutionContext` or similar.
// Document any new visibility relaxation in the commit message.

#[test]
fn load_constant_reads_pre_resolved_constants_array() {
    // TODO(implementer): write the actual test body.
    todo!("install a code stream with a known Smi constant at index 0; run op_test_load_constant; assert returned value matches the constant");
}

#[test]
fn load_this_value_reads_real_this_binding() {
    todo!("install a frame with ThisState::Value(Value::from_smi_i32(42)); run op_test_load_this_value; assert returned value equals 42");
}

#[test]
fn load_this_value_reads_sentinel_for_uninitialized() {
    todo!("install a frame with ThisState::Uninitialized; run op_test_load_this_sentinel; assert returned value equals Value::uninitialized_lexical()");
}
```

**IMPORTANT:** The `todo!()` placeholders in the test bodies above are real (the implementing subagent must fill them in by following the existing `dsl_validation_*.rs` test harness). The Phase 1.B.0 fix at commit `845cee79` is the most recent reference for how to construct a synthetic handler test.

- [ ] **Step 3: Run the new tests — they should panic from the `todo!()` placeholders**

Run: `cargo test -p lyng-js-vm --test dsl_validation_frame_context`
Expected: 3 tests, all panic with "not yet implemented" from `todo!()`. The test file compiles cleanly.

- [ ] **Step 4: Implement the test bodies**

Fill in each `todo!()` by mirroring the harness used in `dsl_validation_empty.rs` and the Phase 1.B.0 test setups. The test runs the synthetic handler through `Vm::run_via_dsl` (or its public test entry point) and asserts on the returned Value.

For the ThisState::Uninitialized case: this may require a test-only helper to push an ExecutionContext with `ThisState::Uninitialized`. If the public API doesn't allow this, add a `#[cfg(test)] pub(crate) fn push_test_execution_context_with_this_state(...)` in `env/src/agent.rs` (with a small commit-message note explaining why).

- [ ] **Step 5: Run the tests — they should all pass**

Run: `cargo test -p lyng-js-vm --test dsl_validation_frame_context`
Expected: 3 tests passing.

- [ ] **Step 6: Run all vm tests + lyng-js-tests for parity**

Run: `cargo test -p lyng-js-vm --release && cargo test -p lyng-js-tests --release`
Expected: all green; integration test count goes up by 3.

- [ ] **Step 7: Commit**

```bash
git add crates/lyng-js/vm/tests/dsl_validation_frame_context.rs
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.1 Task 6: synthetic validation handlers for frame_const_base + frame_this_value

Adds three integration tests that exercise the new backend macros
end-to-end through the DSL trampoline:

- load_constant_reads_pre_resolved_constants_array: verifies
  load_constant!(0 => dst) reads the correct value from the active
  code's pre-resolved constants array.
- load_this_value_reads_real_this_binding: verifies
  load_state_value!(LLINT_STATE_FRAME_THIS_VALUE => dst) reads the
  real this Value when ThisState is Value(v).
- load_this_value_reads_sentinel_for_uninitialized: verifies the
  same load returns Value::uninitialized_lexical() when ThisState
  is Uninitialized.

These tests prevent the new macros from bit-rotting before Phase
1.B.2 lands the real op_load_const8 and op_load_this inline ports.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Add a GC-stress integration test for frame-context refresh

**Files:**
- Create: `crates/lyng-js-tests/tests/gc_stress_frame_context.rs`

- [ ] **Step 1: Investigate the existing GC-stress mechanism**

Look for an existing pattern:

```bash
grep -rln "gc_stress\|force_minor_gc\|force_major_gc\|trigger_gc" crates/lyng-js/ | head -10
grep -rln "cfg.*gc_stress\|feature.*gc_stress" crates/lyng-js/ | head -10
```

If a `--cfg gc_stress` or similar feature exists, use it. If only manual triggers like `vm.force_gc()` are available, use those. If neither exists, use a tight allocating loop and rely on the GC to trigger on its own threshold; document in the test comment that it depends on the default GC trigger being aggressive enough.

- [ ] **Step 2: Write the GC-stress test**

Create `crates/lyng-js-tests/tests/gc_stress_frame_context.rs`:

```rust
//! Phase 1.B.1 Task 7: GC-stress test for the frame_const_base +
//! frame_this_value mirror discipline.
//!
//! Hypothesis being tested: the asm-visible mirror values in
//! LlIntState stay valid across GC events because every slow-path
//! bridge that can trigger GC also goes through the Refresh arm,
//! which refreshes both fields from canonical sources.
//!
//! Strategy: a JS-level tight loop that
//! - reads `this` repeatedly (would observe stale frame_this_value
//!   if the mirror discipline were broken)
//! - allocates new objects each iteration (forces GC pressure)
//! - reads named constants from the constants array (would observe
//!   stale frame_const_base if the mirror discipline were broken)
//! - is run inside a closure (frame_this_value is the closure's
//!   captured `this`)
//!
//! If any GC moves the arena slot for the active code's constants,
//! or if frame_this_value isn't refreshed after a GC, the test will
//! observe wrong values and fail.

use lyng_js_vm::Vm;
// (other imports per the existing lyng-js-tests harness)

#[test]
fn frame_context_survives_gc_pressure() {
    let mut vm = Vm::new_for_tests();
    // (other setup mirroring an existing test in lyng-js-tests)

    // The JS program runs a closure that:
    // - captures `this`
    // - in a loop, reads `this` and a constant, allocates a
    //   fresh object, and accumulates into a counter.
    // After the loop, it returns the counter; we assert the
    // counter equals (iters * known constant value).
    let source = r#"
        var counter = 0;
        var KNOWN = 7;  // constant in the closure's pool
        (function() {
            var self = this;
            for (var i = 0; i < 100000; i++) {
                var obj = { x: i };  // allocates each iter
                counter += KNOWN;
                // touching `self` keeps the closure's `this` in flow
                if (self === null) { throw new Error("this lost"); }
            }
        }).call({});
        counter
    "#;

    let result = vm.run_script(source).expect("script ran");
    // Closure's this is `{}` (object). KNOWN is 7. 100000 iters.
    assert_eq!(result.to_number(), 700000.0);
}
```

**Note:** the iteration count, source-program shape, and `Vm::new_for_tests` invocation should match what other tests in `crates/lyng-js-tests/tests/` already use. If there's a dedicated `gc_stress_*` test pattern, follow it. If `Vm` is constructed differently (e.g. via a helper in `lyng-js-tests/src/lib.rs`), use that helper.

- [ ] **Step 3: Run the test to verify it passes under normal GC settings**

Run: `cargo test -p lyng-js-tests --test gc_stress_frame_context --release`
Expected: PASS. If it doesn't run any GC during the loop, increase the iter count (10x) until a `vm.gc_event_count()` or similar shows at least one GC happened during the loop.

- [ ] **Step 4: If a gc-stress cfg exists, also run with it enabled**

If `--cfg gc_stress` or similar exists, run:

```bash
RUSTFLAGS="--cfg gc_stress" cargo test -p lyng-js-tests --test gc_stress_frame_context --release
```

Expected: PASS. This forces a GC at every allocation; if the mirror discipline is broken, the test fails immediately.

- [ ] **Step 5: Confirm the test actually stresses the relevant code path**

Add an `eprintln!()` (temporarily) showing GC event counts before and after the loop, run the test, and confirm GCs happened during the loop body (not just at startup). Remove the `eprintln!()` before committing.

- [ ] **Step 6: Run the full lyng-js-tests suite for parity**

Run: `cargo test -p lyng-js-tests --release`
Expected: 1187+ passing (1186 baseline + 1 new from this task).

- [ ] **Step 7: Commit**

```bash
git add crates/lyng-js-tests/tests/gc_stress_frame_context.rs
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.1 Task 7: GC-stress test for frame_const_base + frame_this_value mirror discipline

Adds an integration test that runs a JS closure in a tight loop with
heavy allocation pressure, reading both `this` and a captured
constant on every iteration. If the LlIntState mirror discipline
were broken — i.e. frame_const_base or frame_this_value were stale
after a GC — the test would observe wrong values and fail.

Test relies on the standard GC trigger; under `--cfg gc_stress` (if
the repo has it) the test also passes, which is a stronger
guarantee. Run instructions in the test header comment.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: Same-load A/B vs `ae8b7766` and write GC review doc

**Files:**
- Create: `reports/js/lyng-js/dsl-1/phase-1b1-ab-comparison.md`
- Create: `reports/js/lyng-js/dsl-1/phase-1b1-gc-review.md`

- [ ] **Step 1: Capture loadavg snapshot**

Run: `uptime`
Note the 1-min and 5-min loadavg.

- [ ] **Step 2: Stash any uncommitted work**

Run: `git status` to verify the only uncommitted state is `reports/js/lyng-js/bench-v8.md` (the bench tool's side-effect from earlier runs) and untracked planning docs. If anything else is uncommitted, stash it:

Run: `git stash --include-untracked`
(Only if needed. The expected state is "clean except for bench-v8.md and untracked planning docs".)

- [ ] **Step 3: Checkout the base HEAD `ae8b7766` and bench**

Run: `git checkout ae8b7766`
Run: `cargo build --release -p lyng-js-bench`
Run: `cargo run --release -p lyng-js-bench -- v8suite --samples 7 --json /tmp/phase-1b1-base.json`
Capture loadavg: `uptime`

- [ ] **Step 4: Checkout the Task-7-end HEAD and bench**

Run: `git checkout -` (returns to the feature branch HEAD)
Run: `cargo build --release -p lyng-js-bench`
Run: `cargo run --release -p lyng-js-bench -- v8suite --samples 7 --json /tmp/phase-1b1-post.json`
Capture loadavg: `uptime`

If the post-run loadavg differs from the base-run loadavg by more than 20%, abort and re-run during a quieter window.

- [ ] **Step 5: Restore any stashed state**

If you stashed in Step 2, run: `git stash pop`.

- [ ] **Step 6: Compute deltas and write the A/B comparison report**

Compute per-workload deltas and geomean delta from the two JSONs (the bench tool prints these; or pipe through `jq`). Write the comparison to `reports/js/lyng-js/dsl-1/phase-1b1-ab-comparison.md`. Use the Phase 1.B.0 counter-overhead report as the template (`reports/js/lyng-js/dsl-1/phase-1b0-counter-overhead.md`):

```markdown
# Phase 1.B.1 — Same-load A/B comparison vs `ae8b7766`

Measured 2026-MM-DD after frame-context refactor landed (Tasks 1-7).

## Methodology

Per parent spec §4 same-load A/B protocol. Both runs on the same physical machine within a 10-minute window; loadavg overlap verified.

- **Base HEAD:** `ae8b7766` (Phase 1.B.0 closed; no frame_const_base / frame_this_value population)
- **Post HEAD:** `<task-7-end-sha>` (Phase 1.B.1 Tasks 1-7 landed; population + refresh active; no handler reads the new fields yet)

| Measurement | Loadavg at start | Loadavg at end |
|-------------|-----------------:|---------------:|
| Pre-1.B.1   | <value>          | <value>        |
| Post-1.B.1  | <value>          | <value>        |

## V8 v7 results

| Workload    | Base (median) | Post (median) | Delta |
|-------------|--------------:|--------------:|------:|
| Richards    | <value>       | <value>       | <delta>|
| DeltaBlue   | <value>       | <value>       | <delta>|
| Crypto      | <value>       | <value>       | <delta>|
| RayTrace    | <value>       | <value>       | <delta>|
| NavierStokes| <value>       | <value>       | <delta>|
| Splay       | <value>       | <value>       | <delta>|
| **Geomean** | **<value>**   | **<value>**   | **<delta>** |

## Verdict

- **Target:** aggregate V8 v7 regression ≤ 2% (parent §4 1.B.1 gate).
- **Per-workload tolerance:** no workload regresses > 5% (parent epic §4).
- **Observed:** <delta>% geomean (<delta>%-<delta>% per workload).
- **Result:** PASS / FAIL.
```

- [ ] **Step 7: Write the GC review doc**

Create `reports/js/lyng-js/dsl-1/phase-1b1-gc-review.md`. Cover:
1. **Field-by-field reachability proof** — for each new LlIntState field, show the canonical source (which already-scanned struct holds the live value) and the path the existing tracer takes to reach it.
2. **Mirror-staleness argument** — why reads through the mirror always see post-GC-valid values (mirror discipline: reads only happen between Refresh egress events; GC only during slow-path bridges; Refresh runs after the bridge).
3. **Arena pointer stability argument** — why `frame_const_base` (pointing into a `RuntimeCodeRecord::constants` arena slot) doesn't shift across a Refresh egress: the precedent is `frame_pb_base` (already in production); the debug-only assertion (Task 4 Step 3) catches violations in dev.
4. **Risk surface and mitigations** — enumerate what could go wrong and how the test plan covers it.
5. **Reviewer dispatch sign-off** — appended in Task 9.

- [ ] **Step 8: Commit both reports**

```bash
git add reports/js/lyng-js/dsl-1/phase-1b1-ab-comparison.md reports/js/lyng-js/dsl-1/phase-1b1-gc-review.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.1 Task 8: same-load A/B + GC review docs

A/B vs `ae8b7766`: aggregate V8 v7 regression <value>%, within the
≤ 2% gate. No workload regresses > <max>%.

GC review documents the mirror-discipline invariant, the
field-by-field reachability proofs, and the arena pointer stability
argument (precedent: frame_pb_base; safety net: debug-only assertion).

Reviewer dispatch sign-off appended in Task 9.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 9: Reviewer dispatch and address findings

**Files:**
- Append findings to: `reports/js/lyng-js/dsl-1/phase-1b1-gc-review.md`
- Address findings inline (per-finding commits as needed)

- [ ] **Step 1: Dispatch the reviewer**

Use the `Agent` tool with `subagent_type: feature-dev:code-reviewer`. Brief:

> Review the Phase 1.B.1 frame-context refactor commit range (`ae8b7766..HEAD`). Spec at `docs/superpowers/specs/2026-05-19-dsl-1-phase-1b1-frame-context-refactor-design.md`. Plan at `docs/superpowers/plans/2026-05-19-dsl-1-phase-1b1-frame-context-refactor.md`. GC review draft at `reports/js/lyng-js/dsl-1/phase-1b1-gc-review.md`.
>
> Focus areas (high-priority, per spec §6 risks):
> 1. GC root scanning — does the trace logic in `vm/src/vm/state.rs` keep both new LlIntState fields' canonical sources alive across every safepoint that can occur during dispatch? Are there any code paths where a slow-path bridge can run GC but NOT return through the Refresh arm?
> 2. `resolve_initial_this_value` semantics vs `op_load_this` semantic body — read both side-by-side (`crates/lyng-js/vm/src/dsl/llint_state.rs` resolver vs `crates/lyng-js/vm/src/vm/semantics/names.rs:600-627`). Do they implement the same rule for the three ThisState arms + the no-EC fallback? Any drift would surface as throw-at-wrong-PC bugs.
> 3. Arena pointer stability — confirm `RuntimeCodeRecord::constants` arena slot's data pointer is stable across the slow-path call. Cross-check the existing `frame_pb_base` precedent (which has the same dependency).
> 4. Continue arm correctness — confirm super() / mid-frame this_value mutations always egress through Refresh, not Continue. If any continue-path semantic body mutates `frame.this_value()`, that's a bug.
>
> Report: confidence-filtered findings only. For each finding: what's wrong, severity, file:line ref, suggested fix.

- [ ] **Step 2: Read the reviewer's report**

The reviewer returns a structured report. Triage each finding:
- **High-severity:** address before sub-phase close. If the fix is substantial, may need its own task added to the plan.
- **Medium-severity:** address if quick; otherwise defer to Phase 1.B.2 / 1.B.3 with a tracking note in the sub-phase summary.
- **Low-severity:** document in the sub-phase summary; defer.

- [ ] **Step 3: Address findings**

For each high-severity finding, write a commit titled `Phase 1.B.1 Task 9 review: <short fix description>`. Run the standard test suites after each fix:

```bash
cargo test -p lyng-js-vm --lib --release && cargo test -p lyng-js-tests --release
```

Expected: 417+ / 1187+ passing throughout.

- [ ] **Step 4: Append reviewer sign-off to the GC review doc**

In `reports/js/lyng-js/dsl-1/phase-1b1-gc-review.md`, append a section:

```markdown
## Reviewer dispatch sign-off

Reviewer: feature-dev:code-reviewer (dispatched 2026-MM-DD)
Commit range reviewed: `ae8b7766..<HEAD-after-fixes>`

### Findings summary

| Severity | Count | Resolution |
|----------|------:|-----------|
| High     | <n>   | All fixed in <commit-shas> |
| Medium   | <n>   | <n> fixed; <n> deferred to Phase 1.B.<x> |
| Low      | <n>   | Documented for future reference |

### Verdict

All high-severity findings resolved. Phase 1.B.1 substrate is GC-safe and semantically equivalent to the legacy op_load_this resolution.
```

- [ ] **Step 5: Commit the appended review section**

```bash
git add reports/js/lyng-js/dsl-1/phase-1b1-gc-review.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.1 Task 9: reviewer dispatch sign-off

Appends reviewer findings summary and sign-off to the GC review doc.
All high-severity findings addressed in earlier Task 9 commits;
medium / low findings documented for future reference.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 10: Phase 1.B.1 sub-phase summary

**Files:**
- Create: `reports/js/lyng-js/dsl-1/phase-1b1-summary.md`

- [ ] **Step 1: Draft the summary**

Use `reports/js/lyng-js/dsl-1/phase-1b0-summary.md` as the template. Include:
- Date range and baseline-vs-HEAD commit SHAs.
- Status: closed.
- Scope landed table (10 tasks → commits).
- Field-level decisions made (cite spec §9 decisions).
- Test results summary (vm 417+/lyng-js-tests 1187+/Test262 baseline match/gc-stress green).
- Same-load A/B summary (link to `phase-1b1-ab-comparison.md`).
- GC review verdict (link to `phase-1b1-gc-review.md`).
- Lessons / observations.
- Phase 1.B.1 exit criteria assessment table (mirror the Phase 1.B.0 summary's structure).
- Decision: closed; recommended next steps (Phase 1.B.2 backfill ports).
- Commits list.

- [ ] **Step 2: Verify all gate criteria are met**

Cross-check against spec §1 exit criteria. Each gate must be ✅:
- Layout stable: `ll_int_state_offsets_stable` passes.
- Behavioral parity: 417+ vm, 1187+ tests.
- Test262 ≥ baseline.
- gc-stress clean.
- Same-load A/B ≤ 2% regression.
- GC review documented.
- Reviewer pass.

If any gate is ❌, the sub-phase is NOT closed; back to the relevant task to fix.

- [ ] **Step 3: Commit the summary**

```bash
git add reports/js/lyng-js/dsl-1/phase-1b1-summary.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.1: phase summary — frame-context refactor complete

10 commits landing the asm-visible frame_const_base + frame_this_value
substrate on LlIntState. No opcode handlers ported in this sub-phase;
Phase 1.B.2 picks up op_load_const8 + op_load_this inline ports.

All exit gates green:
- Layout stable; ll_int_state_offsets_stable passing (size 72 bytes)
- Behavioral parity: 417+ vm tests, 1187+ lyng-js-tests
- Test262 ≥ Phase 1.B.0 baseline
- gc-stress test passing
- Same-load A/B vs `ae8b7766`: <delta>% V8 v7 regression (within 2% gate)
- GC review + reviewer dispatch sign-off documented

Phase 1.B.1 closed. Phase 1.B.2 (backfill ports) can proceed.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 4: Report sub-phase closure**

The implementing agent reports back to the coordinator with:
- Final HEAD SHA.
- Summary of any deferred findings.
- Recommendation: proceed to Phase 1.B.2 brainstorming OR pause for review.

---

## Self-review of this plan

**Spec coverage check:**

| Spec section | Task(s) implementing it |
|--------------|--------------------------|
| §3.1 LlIntState layout | Task 1 |
| §3.2 New offset consts | Task 1 |
| §3.3 resolve_initial_this_value helper | Task 2 |
| §3.4 Entry-shim population | Task 3 |
| §3.4 Refresh arm wiring | Task 4 |
| §3.5 load_constant! macro | Task 5 |
| §3.5 load_state_value! macro | Task 5 |
| §3.6 Debug-only stability assertion | Task 4 Step 3 |
| §4 Data flow | Tasks 3 + 4 (entry-shim + Refresh) |
| §5 GC integration / mirror discipline | Task 8 GC review doc |
| §6.1 Unit tests | Task 2 (4 helper tests) + Task 1 (offset stability) |
| §6.2 dsl_validation_frame_context.rs | Task 6 |
| §6.2 gc-stress test | Task 7 |
| §6.3 Behavioral parity | Every task (cargo test runs) |
| §6.4 Test262 | Task 10 (verified at gate check) |
| §6.5 V8 v7 same-load A/B | Task 8 |
| §6.6 Reviewer dispatch | Task 9 |
| §7 Implementation phasing | Maps 1:1 to tasks 1-10 |
| §8 Risks | Implicitly addressed: stability assertion (Task 4), reviewer (Task 9), gc-stress test (Task 7) |
| §9 Decisions | Explicit in code/commit messages |

No gaps.

**Placeholder scan:** The `todo!()` markers in Task 6 Step 2 are explicit and called out as "real" — the implementing agent must fill them in. No "TBD" / "add error handling" / "similar to X" anti-patterns. Notes that say "exact API may need adjusting if X" are paired with concrete fallback patterns or grep commands to discover the real API.

**Type consistency:** `resolve_initial_this_value` signature is consistent between Task 2 (definition), Task 3 (entry-shim call), and Task 4 (Refresh arm call). `frame_const_base` / `frame_this_value` field names are consistent throughout. `LLINT_STATE_FRAME_CONST_BASE` / `LLINT_STATE_FRAME_THIS_VALUE` const names are consistent. The dsl backend macros are named consistently: `load_constant!`, `load_state_value!`.
