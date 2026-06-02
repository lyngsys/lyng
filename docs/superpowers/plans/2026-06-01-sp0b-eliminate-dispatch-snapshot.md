# SP-0b Snapshot Elimination (Option A) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eliminate the per-frame dispatch snapshot `DispatchState.frame: FrameRecord` so the interpreter reads/writes the `FrameHeader` overlay + `FrameColdTable` directly — the SP-1-aligned register-file model — removing the per-call/return `reconstruct_frame_from_header` cost that caused the ~15% SP-0b call/return regression.

**Architecture:** Production runs the asm (LLInt) path; the per-opcode hot loop reads `LlIntState` pinned fields (`frame_pc_offset`, `frame_regs_base`, `frame_this_value`, …) and never touches `DispatchState.frame`. The snapshot is used only by (a) slow-path Rust semantic bodies and (b) the frame-switch bridge in `dsl/slow_path.rs`. The `Refresh` arm rebuilds an entire `FrameRecord` via `reconstruct_frame_from_header` on **every** call/return, then derives the `LlIntState` fields from it — that reconstruction is the regression. We replace the snapshot with a thin **active-frame view** on `DispatchState` (`cfr`, `pc`, `code`, `regs_len`) that addresses the overlay directly; semantic bodies read frame fields through new `DispatchState` accessor methods that delegate to `vm.frame_header(cfr)` / `vm.frame_cold`; the bridge maintains the thin view and populates `LlIntState` straight from the overlay with no `FrameRecord` materialized on the hot path.

**Tech Stack:** Rust; `crates/vm` (the bytecode VM). Key modules: `vm/dispatch_state.rs`, `vm/dispatch.rs`, `dsl/slow_path.rs`, `dsl/llint_state.rs`, `vm/semantics/*`, `vm/registers.rs`, `vm/call.rs`, `vm/with_env.rs`, `vm/bytecode_calls.rs`, `frame_header.rs`, `frame_cold.rs`, `frame.rs`, `vm.rs`.

**Invariants (do not break):**
- Whole-corpus Test262 must stay at baseline: **49729 runnable / 49729 passed / 0 failed / 0 panicked / 3324 skips**.
- All heavy runs (cargo test, Test262, bench) execute under `/tmp/memtest.sh '<cmd>'` (16 GB machine, no ulimit). Never write bench/262 reports to the checked-in `reports/lyng/` paths — use `/tmp`.
- Build green at the end of **every task**: `cargo build --workspace --all-targets --all-features` + `cargo clippy --workspace --all-targets --all-features`.
- The asm-ABI offset constants (`FRAME_HEADER_*`, `LLINT_STATE_*`) and the `frame_header_offsets_stable` test must remain unchanged unless a task explicitly says so.

---

## Background facts (verified against HEAD 7d199d16)

**`DispatchState<'vm>`** (`vm/dispatch_state.rs:42`) currently holds: `vm`, `agent`, `host`, `registry`, `installed: Arc<InstalledFunction>`, `frame: FrameRecord`, `frame_depth: usize`, `frame_check_epoch: u32`, `prefix: Option<Opcode>`.

**Field storage tiers** (target of each read after migration):
- **Overlay** (`vm.frame_header(cfr)` / `frame_header_mut(cfr)`, accessors in `frame_header.rs`): `code`, `callee`, `this_value`, `this_state`, `construct_this`, `new_target`, `variable_env`, `lexical_env`, `private_env`, `return_register`, `flags` (`flags_bits`), `kind` (`kind_raw`), `arg_count`, `caller_cfr`, `saved_pc`.
- **Cold** (`vm.frame_cold.get(depth)` / `get_mut(depth)`, `frame_cold.rs`): `handler_cursor`, `tail_caller` (+`tail_caller_strict`), `resume_kind`, `resume_value`, `resume_active`, `parameter_initializer_end_offset`.
- **Thin view** (new fields on `DispatchState`): `pc` (≙ old `instruction_offset`), `registers` window (base = `cfr + HEADER_SLOTS`, len = `regs_len`), `code` (cached `CodeRef`), `cfr`.

**Register access:** semantic bodies call `inner.frame.registers()` → `RegisterWindow{base,len}` and pass it to `vm.read_register_unchecked(window, idx)` / `write_register_unchecked`. The window is metadata only; register **values** live in `vm.arena`. `RegisterWindow`, `CodeRef`, `Value`, `EnvironmentRef`, `ObjectRef`, `ThisState`, `FrameFlags` are all `Copy`.

**Borrow-checker pattern (critical):** many semantic bodies destructure `let DispatchState { vm, agent, frame, .. } = &mut *inner;` to hold `&mut vm` and read `frame` simultaneously. After migration, frame data lives *inside* `vm` (the overlay), so reading it needs `&vm` — conflicting with `&mut vm`. **Resolution:** pull every needed frame field into a `Copy` local *before* the `let DispatchState { vm, .. }` destructure, then use the locals. Example:
```rust
// BEFORE
let DispatchState { vm, agent, frame, .. } = &mut *inner;
let registers = frame.registers();
vm.execute_add_opcode(agent, host, registry, registers, b, c);
// AFTER
let registers = inner.registers();          // Copy out of the thin view
let DispatchState { vm, agent, host, registry, .. } = &mut *inner;
vm.execute_add_opcode(agent, host, registry, registers, b, c);
```

**The bridge functions** (all in scope for Phase 3):
- `DispatchState::sync_active_frame` / `refresh_from_active_frame` (`dispatch_state.rs:129,186`).
- `Vm::sync_dispatch_frame` → `write_snapshot_into_backing` (`vm/dispatch.rs:190`, `vm.rs:1531`).
- `Vm::refresh_dispatch_frame` → `reconstruct_frame_from_header` (`vm/dispatch.rs:206`, `vm.rs:1595`).
- `Vm::handle_dispatch_result` / `finish_abc_value_result` (`vm/dispatch.rs:239,...`) — take `frame: &mut FrameRecord`.
- `advance_dispatch_frame` / `next_dispatch_instruction_offset` (`vm/dispatch.rs`) — take `&mut FrameRecord` / `&FrameRecord`.
- `Vm::finish_frame` / `pop_current_frame` / `release_frame_to_caller` (`vm/registers.rs:106..`, `vm.rs:1791,1807`).
- `LlIntDispatchState::{sync_from_asm, translate_outcome (Continue/Refresh arms), current_instruction_offset, decode_current_*_operands}` (`dsl/slow_path.rs`).
- `resolve_initial_this_value(&FrameRecord)` (`dsl/llint_state.rs:152`).
- `Vm::push_with_environment` / `pop_with_environment` (`vm/with_env.rs`).

**`reconstruct_frame_from_header` / `FrameRecord` retention:** `FrameRecord` stays as the **construction** type used at push (`write_header_from_record`) and by generator snapshot/restore (`vm/generators.rs`). `reconstruct_frame_from_header` is removed from the hot call/return path; it is retained ONLY if a rare path (generator restore) still needs it — Phase 4 deletes it if no caller remains.

---

## Phase 1 — Thin-view scaffolding + register/PC/code reader migration (behavior-identical, green)

Add the thin-view fields, maintain them in lockstep with `frame`, and migrate the highest-volume readers (`registers()`, `instruction_offset()`, `code()`) to thin-view accessors. The thin view mirrors `frame` exactly, so behavior is unchanged.

### Task 1: Add thin-view fields + accessors to `DispatchState`

**Files:**
- Modify: `crates/vm/src/vm/dispatch_state.rs`
- Modify: `crates/vm/src/dsl/entry.rs` (constructor call sites for `new_for_dsl_entry`)
- Modify: `crates/vm/src/dsl/test_helpers.rs` (`new_for_dsl_harness` call sites)

- [ ] **Step 1: Add fields.** In `DispatchState<'vm>` add, after `frame: FrameRecord`:
```rust
    /// Active frame's cfr (arena slot index of its `FrameHeader`). Mirrors
    /// `vm.current_cfr` for the active frame; addresses the overlay + window.
    pub(crate) cfr: u32,
    /// Live program counter (== old `frame.instruction_offset()`). The one
    /// frame datum not parked in the overlay mid-frame; synced to/from the
    /// asm side's `LlIntState.frame_pc_offset` at slow-path boundaries.
    pub(crate) pc: u32,
    /// Cached active `CodeRef` (== `installed.function()` code). Hot in the
    /// decode + jump paths.
    pub(crate) code_ref: CodeRef,
    /// Active register-window length (window base = `cfr + HEADER_SLOTS`).
    pub(crate) regs_len: u16,
```

- [ ] **Step 2: Add accessors.** Add to `impl DispatchState`:
```rust
    #[inline]
    pub(crate) const fn pc(&self) -> u32 { self.pc }
    #[inline]
    pub(crate) fn set_pc(&mut self, pc: u32) { self.pc = pc; }
    #[inline]
    pub(crate) const fn cfr(&self) -> u32 { self.cfr }
    #[inline]
    pub(crate) fn registers(&self) -> crate::frame::RegisterWindow {
        crate::frame::RegisterWindow::new(
            self.cfr + crate::frame_header::HEADER_SLOTS as u32,
            self.regs_len,
        )
    }
```
Change `code()` (currently `self.frame.code()`) to `self.code_ref`.

- [ ] **Step 3: Seed the fields in both constructors.** In `new_for_dsl_entry` and `new_for_dsl_harness`, seed:
```rust
        cfr: crate::vm::Vm::cfr_of(&frame),
        pc: frame.instruction_offset(),
        code_ref: frame.code(),
        regs_len: frame.registers().len(),
```
(Confirm `Vm::cfr_of` is reachable from these modules; if not, derive `cfr` from `frame.registers().base() - HEADER_SLOTS`.)

- [ ] **Step 4: Build + clippy.**
Run: `cargo build --workspace --all-targets --all-features 2>&1 | tail -20`
Expected: compiles. Unused-field warnings on the new fields are acceptable at this step (they get consumers in Task 3).

- [ ] **Step 5: Commit.**
```bash
git add -A && git commit -m "refactor(vm): add DispatchState thin-frame view fields (cfr/pc/code_ref/regs_len)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

### Task 2: Maintain the thin view in lockstep with `frame` at every frame-state transition

Every place that assigns `frame` / advances PC must also update `cfr`/`pc`/`code_ref`/`regs_len`. While `frame` still exists this is redundant mirroring; it lets Task 3 readers switch safely.

**Files:**
- Modify: `crates/vm/src/vm/dispatch_state.rs` (`refresh_from_active_frame`, `sync_active_frame`)
- Modify: `crates/vm/src/dsl/slow_path.rs` (`sync_from_asm`, `translate_outcome` Continue + Refresh arms)

- [ ] **Step 1:** In `DispatchState::refresh_from_active_frame`, after `self.frame = frame;` add:
```rust
        self.cfr = cfr;
        self.pc = frame.instruction_offset();
        self.code_ref = frame.code();
        self.regs_len = frame.registers().len();
```

- [ ] **Step 2:** In `LlIntDispatchState::sync_from_asm` (the `Asm` arm), after setting `frame.set_instruction_offset((**state).frame_pc_offset)` add:
```rust
                rust.dispatch.pc = (**state).frame_pc_offset;
```

- [ ] **Step 3:** In `translate_outcome` **Continue** arm, where `new_offset` is computed, also store it on the thin view:
```rust
                    rust.dispatch.pc = new_offset;
```
(immediately after `let new_offset = ...wrapping_add(pc_advance);`).

- [ ] **Step 4:** In `translate_outcome` **Refresh** arm, after `rust.dispatch.frame = ...reconstruct_frame_from_header(cfr, current_depth - 1);` and the `installed` reassignment, add:
```rust
                    rust.dispatch.cfr = cfr;
                    rust.dispatch.pc = rust.dispatch.frame.instruction_offset();
                    rust.dispatch.code_ref = rust.dispatch.frame.code();
                    rust.dispatch.regs_len = rust.dispatch.frame.registers().len();
```
(use the `cfr` already bound by the `if let Some(cfr)`; hoist it so it's in scope, or recompute via `rust.dispatch.vm.current_cfr_opt()`).

- [ ] **Step 5: Build + test the slow-path/dispatch unit tests.**
Run: `/tmp/memtest.sh 'cargo test -p lyng-vm --all-features 2>&1 | tail -30'`
Expected: all pass; thin view now tracks `frame` everywhere it changes.

- [ ] **Step 6: Commit.**
```bash
git add -A && git commit -m "refactor(vm): maintain DispatchState thin-frame view in lockstep with the snapshot

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

### Task 3: Migrate register / PC / code readers to thin-view accessors

Replace `inner.frame.registers()` → `inner.registers()`, `inner.frame.instruction_offset()` → `inner.pc()`, `inner.frame.code()` → `inner.code()` across semantic bodies and the bridge decode helpers. Apply the borrow-pattern (pull `let registers = inner.registers();` before any `let DispatchState { vm, .. }`).

**Files (migrate `registers()`/PC/`code()` reads in each; full site list per the catalogue):**
- `crates/vm/src/vm/semantics/loads.rs` (`registers()` ×several: lines ~51,75,139,209)
- `crates/vm/src/vm/semantics/control_flow.rs` (PC reads ~104,181; `code()` ~110,187; `registers()` ~164,300)
- `crates/vm/src/vm/semantics/{arithmetic,names,scope,property,calls,exceptions,iterators,prefix,misc,generators}.rs`
- `crates/vm/src/dsl/slow_path.rs` (`decode_current_abx/abc/abc_slot_operands`: `inner.frame.instruction_offset()` → `inner.pc()`; `inner.code()` stays — already a method; `current_instruction_offset()` → `state.pc` via match arms)

- [ ] **Step 1: Migrate one file (`loads.rs`) as the exemplar.** For each `let registers = inner.frame.registers();` change to `let registers = inner.registers();`. Run `cargo build -p lyng-vm --all-features 2>&1 | tail -20` — expect green.

- [ ] **Step 2: Migrate the remaining semantics files** the same way (`registers()`, `instruction_offset()`→`pc()`, `code()`). After each file, rebuild.

- [ ] **Step 3: Migrate `slow_path.rs` decode helpers + `current_instruction_offset`:**
```rust
    pub const fn current_instruction_offset(&self) -> u32 {
        match &self.inner {
            LlIntDispatchInner::Alpha(state) => state.pc,
            LlIntDispatchInner::Asm { rust, .. } => rust.dispatch.pc,
        }
    }
```
and in each `decode_current_*` replace `let pc = inner.frame.instruction_offset();` with `let pc = inner.pc();`.

- [ ] **Step 4: Verify no `frame.registers()`/`frame.instruction_offset()`/`frame.code()` reads remain in semantics:**
Run: `grep -rn "frame\.registers()\|frame\.instruction_offset()\|frame\.code()" crates/vm/src/vm/semantics crates/vm/src/dsl/slow_path.rs`
Expected: no matches (bridge-internal `frame` uses in `translate_outcome`/`finish_abc_value_result` remain until Phase 3).

- [ ] **Step 5: Build + clippy + vm tests.**
Run: `/tmp/memtest.sh 'cargo build --workspace --all-targets --all-features && cargo clippy --workspace --all-targets --all-features 2>&1 | tail -30'`
Run: `/tmp/memtest.sh 'cargo test -p lyng-vm --all-features 2>&1 | tail -20'`
Expected: green; tests pass.

- [ ] **Step 6: Commit.**
```bash
git add -A && git commit -m "refactor(vm): read registers/PC/code via the DispatchState thin view

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

### Task 4: Phase-1 checkpoint — full Test262

- [ ] **Step 1: Run whole-corpus Test262 under the watchdog.**
Run: `/tmp/memtest.sh 'cd /Users/sondre/dev/lyng && cargo run -q -p lyng-test262 --release -- --report /tmp/t262-phase1.md -j 8 2>&1 | tail -20'`
(Adjust the runner invocation to match `tools/lyng-test262`'s actual CLI; do NOT write to `reports/lyng/`.)
Expected: `49729 runnable / 49729 passed / 0 failed / 0 panicked / 3324 skips`.

- [ ] **Step 2:** If any regression, bisect within Phase 1 commits before proceeding. Do not advance to Phase 2 on a red 262.

---

## Phase 2 — Overlay/cold-direct field accessors + remaining reader migration (green)

Add `DispatchState` accessor methods for the overlay/cold fields and migrate the remaining `frame.X()` reads. Before switching a reader to the overlay, confirm that field's overlay/cold copy is authoritative mid-frame (every mutation writes through). The snapshot's copies of these fields become unused.

### Task 5: Audit + close mid-frame write-through gaps

Confirm each overlay/cold field that semantic bodies READ is written to the overlay/cold at the moment it is mutated (not only at boundaries), so an overlay read mid-frame is never stale.

**Files:** `crates/vm/src/vm/with_env.rs`, `crates/vm/src/vm/semantics/generators.rs`, `crates/vm/src/vm/generators.rs`, plus any `frame.set_*` mutation in semantics.

- [ ] **Step 1: Enumerate mid-frame mutations of overlay/cold-tier fields.**
Run: `grep -rn "\.set_lexical_env\|\.clear_resume\|\.set_this\|\.set_construct_this\|with_resume\|set_resume" crates/vm/src/vm/semantics crates/vm/src/vm/with_env.rs crates/vm/src/vm/generators.rs`

- [ ] **Step 2:** For each, verify a corresponding overlay/cold write exists:
  - `lexical_env`: `with_env.rs` already mirrors to `frame_header_mut(cfr).set_lexical_env(...)` on push/pop. ✓
  - `resume_active`/`resume_*`: confirm `clear_resume`/`with_resume` paths write `vm.frame_cold.get_mut(depth)`; if a path only mutates the snapshot, add the cold write.
  - `this_value`/`this_state` (super() initialization): confirm the slow path writes `frame_header_mut(cfr).set_this(...)`. Since super() egresses through `Refresh`, the overlay must already hold it; verify and add a write if missing.
- [ ] **Step 3:** Add any missing overlay/cold write-throughs (small, targeted). Build.
- [ ] **Step 4: Commit** (skip if no changes were needed):
```bash
git add -A && git commit -m "fix(vm): write overlay/cold through on mid-frame this/resume mutations

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

### Task 6: Add overlay/cold field accessors on `DispatchState`

**Files:** `crates/vm/src/vm/dispatch_state.rs`

- [ ] **Step 1:** Add overlay-reading accessors (each delegates to `self.vm.frame_header(self.cfr)`):
```rust
    #[inline]
    pub(crate) fn this_value(&self) -> Value { self.vm.frame_header(self.cfr).this_value() }
    #[inline]
    pub(crate) fn this_state(&self) -> crate::frame::ThisState { self.vm.frame_header(self.cfr).this_state() }
    #[inline]
    pub(crate) fn lexical_env(&self) -> EnvironmentRef { self.vm.frame_header(self.cfr).lexical_env() }
    #[inline]
    pub(crate) fn variable_env(&self) -> EnvironmentRef { self.vm.frame_header(self.cfr).variable_env() }
    #[inline]
    pub(crate) fn private_env(&self) -> Option<EnvironmentRef> { self.vm.frame_header(self.cfr).private_env() }
    #[inline]
    pub(crate) fn callee(&self) -> Option<ObjectRef> { self.vm.frame_header(self.cfr).callee() }
    #[inline]
    pub(crate) fn new_target(&self) -> Option<ObjectRef> { self.vm.frame_header(self.cfr).new_target() }
    #[inline]
    pub(crate) fn construct_this(&self) -> Option<ObjectRef> { self.vm.frame_header(self.cfr).construct_this() }
    #[inline]
    pub(crate) fn return_register(&self) -> Option<u16> { self.vm.frame_header(self.cfr).return_register() }
    #[inline]
    pub(crate) fn flags(&self) -> crate::frame::FrameFlags {
        crate::frame::FrameFlags::from_raw(self.vm.frame_header(self.cfr).flags_bits())
    }
    #[inline]
    pub(crate) fn frame_kind(&self) -> lyng_env::ExecutionContextKind { self.vm.frame_header(self.cfr).kind() }
    #[inline]
    pub(crate) fn set_lexical_env(&mut self, env: EnvironmentRef) {
        let cfr = self.cfr;
        self.vm.frame_header_mut(cfr).set_lexical_env(env);
    }
```

- [ ] **Step 2:** Add cold accessors (depth = `self.frame_depth - 1`):
```rust
    #[inline]
    fn cold_index(&self) -> usize { self.frame_depth - 1 }
    #[inline]
    pub(crate) fn resume_kind(&self) -> crate::frame::GeneratorResumeKind {
        self.vm.frame_cold.get(self.cold_index()).resume_kind
    }
    #[inline]
    pub(crate) fn resume_value(&self) -> Value {
        self.vm.frame_cold.get(self.cold_index()).resume_value
    }
    #[inline]
    pub(crate) fn resume_active(&self) -> bool {
        self.vm.frame_cold.get(self.cold_index()).resume_active
    }
    #[inline]
    pub(crate) fn clear_resume(&mut self) {
        let i = self.cold_index();
        self.vm.frame_cold.get_mut(i).resume_active = false;
    }
    #[inline]
    pub(crate) fn handler_cursor(&self) -> u16 {
        self.vm.frame_cold.get(self.cold_index()).handler_cursor
    }
```
(Confirm `vm.frame_cold` field visibility from `dispatch_state.rs`; if not `pub(in crate::vm)`, add small `Vm` getter/setter wrappers instead of touching the field directly. Confirm the exact `FrameColdState` field names against `frame_cold.rs`.)

- [ ] **Step 3: Build** (`cargo build -p lyng-vm --all-features`). Expect green (accessors unused yet → allow dead-code until Task 7).

- [ ] **Step 4: Commit.**
```bash
git add -A && git commit -m "refactor(vm): add DispatchState overlay/cold field accessors

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

### Task 7: Migrate remaining `frame.X()` field reads/writes to the accessors

**Files (per catalogue):**
- `this_value`/`this_state`: `vm/semantics/names.rs:606` (`LoadThis`).
- `lexical_env` R: `vm/semantics/scope.rs:87,118,149`; W: `vm/with_env.rs:26,49` (the snapshot `frame.set_lexical_env` write — drop it; overlay mirror already present).
- `variable_env`: `vm/async_functions.rs:38`.
- `callee`: `vm/bytecode_calls.rs:82`.
- `construct_this`: `vm/bytecode_calls.rs:86`.
- `new_target`: `vm/bytecode_calls.rs:575`.
- `private_env`: `vm/bytecode_calls.rs:814`.
- `return_register`/`flags`: `vm/bytecode_calls.rs:103,105`.
- `kind`: `vm/generators.rs:1671`.
- `resume_*`: `vm/semantics/generators.rs:296,317,322`; `vm/async_functions.rs:150,152`; `vm/generators.rs:864`.
- `handler_cursor`: `vm/generators.rs:1669`.

- [ ] **Step 1:** Migrate `LoadThis` in `names.rs`:
```rust
        let this_state = inner.this_state();
        // ... uses inner.this_value() in the Uninitialized/fallback arm
```
Rebuild.

- [ ] **Step 2:** Migrate the rest field-by-field, applying the borrow pattern (pull Copy locals before `let DispatchState { vm, .. }`). For `with_env.rs`, the helpers take `frame: &mut FrameRecord` today — change them to operate via the `vm` overlay only (they already mirror to the overlay), and update callers to stop passing the snapshot (this dovetails with Phase 3; if the signature change is awkward here, leave `with_env` for Task 9 and only migrate the *reads* in Phase 2).

- [ ] **Step 3: Verify no snapshot field reads remain in semantics:**
Run: `grep -rn "frame\.\(this_value\|this_state\|lexical_env\|variable_env\|private_env\|callee\|new_target\|construct_this\|return_register\|flags\|kind\|resume_\|handler_cursor\)" crates/vm/src/vm/semantics crates/vm/src/vm/async_functions.rs crates/vm/src/vm/bytecode_calls.rs crates/vm/src/vm/generators.rs`
Expected: only bridge-internal / construction (`write_header_from_record`, generator snapshot) uses remain.

- [ ] **Step 4: Build + clippy + vm tests.**
Run: `/tmp/memtest.sh 'cargo build --workspace --all-targets --all-features && cargo clippy --workspace --all-targets --all-features 2>&1 | tail -30'`
Run: `/tmp/memtest.sh 'cargo test -p lyng-vm --all-features 2>&1 | tail -20'`
Expected: green.

- [ ] **Step 5: Commit.**
```bash
git add -A && git commit -m "refactor(vm): read frame fields from the overlay/cold via DispatchState accessors

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

### Task 8: Phase-2 checkpoint — full Test262

- [ ] **Step 1:** Run whole-corpus Test262 (as Task 4 Step 1, report `/tmp/t262-phase2.md`).
Expected: baseline (49729/49729/0/0, 3324 skips).
- [ ] **Step 2:** Bisect within Phase 2 on any regression before advancing.

---

## REVISION (2026-06-01, mid-execution): the snapshot is threaded as a parameter — Phases 3–4 redone around `FrameView`

**Discovery during Phase 2:** `DispatchState.frame` is not only read in semantic bodies — it is threaded as a `frame: &FrameRecord` / `&mut FrameRecord` **parameter through ~252 vm-method signatures** (`vm/dispatch/arithmetic.rs`, `vm/dispatch/property.rs`, `vm/values.rs`, `vm/bytecode_calls.rs`, `vm/builtin_dispatch.rs`, `vm/generators.rs`, `vm/with_env.rs`, `vm/ic_slow_counters.rs`, …). Usage is overwhelmingly `frame.registers()` and `frame.code()` (e.g. arithmetic 28× registers-only; property 53× code + 37× registers; values 8× registers + 1× code + 1× pc) — **header fields are read only in a minority** (bytecode_calls `caller_frame.this_value/callee/new_target/construct_this/return_register/flags`, generators frame-encode, with_env lexical_env).

**Realization of option A:** introduce a cheap `Copy` **`FrameView { cfr: u32, pc: u32, regs_len: u16, code: CodeRef }`** that addresses the overlay, and thread *it* in place of `frame: &FrameRecord`. This is option A's end state — no fat `FrameRecord` snapshot, no `reconstruct_frame_from_header` on call/return, overlay = single source of truth, identical to what SP-1's asm keeps in registers (cfr/regs_base/pc/code). `frame.registers()`/`frame.code()`/`frame.instruction_offset()` are O(1) on `FrameView` (no lookup), so method bodies that only use those are unchanged — it is a mechanical parameter-type swap. `FrameRecord` survives ONLY as the push/construction type (`write_header_from_record`) and generator snapshot/restore.

**Revised tasks (supersede the original Phase 3 Task 9–12 and Phase 4 Task 13 below):**

### Task 9 (revised): Define `FrameView` + `DispatchState::frame_view()`
- Add `FrameView` (Copy) in `crates/vm/src/frame.rs` (next to `RegisterWindow`): fields `cfr`, `pc`, `regs_len`, `code`; methods `registers() -> RegisterWindow` (= `RegisterWindow::new(cfr + HEADER_SLOTS, regs_len)`), `code() -> CodeRef`, `instruction_offset() -> u32` (= `pc`), `cfr() -> u32`.
- Add `DispatchState::frame_view(&self) -> FrameView` building it from the thin-view fields.
- Migrate ONE proof file — `vm/dispatch/arithmetic.rs` (registers-only, ~10 methods / 28 `frame.registers()` sites): change each `frame: &FrameRecord` param to `frame: FrameView`; bodies keep `frame.registers()` verbatim; update all callers (compiler-enforced) to pass `state.frame_view()` / the threaded `FrameView`. Build + vm tests (controller). Commit.

### Task 10 (revised): Migrate the register/code/pc-only method families to `FrameView`
- `vm/dispatch/property.rs` (code + registers), `vm/values.rs` (registers/code/pc). Same mechanical swap; bodies unchanged. Per-file build + vm tests. Commit per file.

### Task 11 (revised): Migrate the header-field-reading method families to `FrameView` + overlay reads
- `vm/bytecode_calls.rs`, `vm/generators.rs` (frame-encode at ~1682–1692 reads all fields → read `self.frame_header(view.cfr).*`), `vm/with_env.rs`, `vm/builtin_dispatch.rs`, `vm/ic_slow_counters.rs`, `vm/values.rs::require_object`, `Vm::resolve_this_binding`, plus the semantic-body field reads deferred from Phase 2 (`semantics/scope.rs` lexical_env ×3 → `inner.lexical_env()`; `semantics/names.rs` LoadThis/LoadCallee/LoadNewTarget → `inner.this_state()/callee()/new_target()`). Field reads become `self.frame_header(view.cfr).field()` (methods are `&self`/`&mut self` on `Vm`) or `inner.<field>()` (semantic bodies). Per-file build + vm tests. Commit per file.

### Task 12 (revised): Bridge — drop the reconstruct from the hot call/return path
- `dsl/slow_path.rs` `Refresh` arm: populate `LlIntState` (regs_base/pb_base/mt_base/const_base/this) directly from `frame_header(cfr)` + `installed` + `frame_window_len(cfr)`; update the thin view (cfr/pc=saved_pc/code/regs_len/depth); **no `reconstruct_frame_from_header`**. `Continue` arm: source pc/code/regs from the thin view (drop `let active_frame = rust.dispatch.frame`). `sync_from_asm`: set thin-view `pc` only.
- `vm/dispatch.rs` `handle_dispatch_result`/`finish_abc_value_result`: take the thin view (`&mut DispatchState` or cfr/pc/depth) instead of `frame: &mut FrameRecord`; park PC into the overlay (`frame_header_mut(cfr).set_saved_pc(pc)`) on the throw path; advance via thin-view pc.
- `vm/registers.rs` `finish_frame`: read return_register/window/caller_cfr from the overlay; no reconstruct.
- `vm/with_env.rs` push/pop: drop the `frame: &mut FrameRecord` param (overlay write already present).
- Test262 checkpoint + Richards/RayTrace bench signal.

### Task 13 (revised): Delete `DispatchState.frame` + dead bridge
- Remove the `frame: FrameRecord` field + the kept snapshot resume writes (`inner.frame.clear_resume()` etc.) + `write_snapshot_into_backing` + `refresh_dispatch_frame` + `reconstruct_frame_from_header` (iff no generator-restore caller remains) + `advance_dispatch_frame`/`next_dispatch_instruction_offset` if unused. Fix the two `DispatchState` constructors to build the thin view from `(cfr, pc, code, regs_len, depth)`. Compile-error-driven cleanup. Full workspace tests.

---

## EXECUTION LOG / REMAINING-WORK (live, 2026-06-01)

**Done + green (Test262 = baseline 49729/0/0; vm 609/0):**
- P1 (thin view + reg/pc/code readers): commits f0d471d8, c02a6ffb, e9067953, c0011dff.
- P2 (resume_* → cold authoritative): commit 16e2414f.
- FrameView + `frame_view()` + `from_record` bridge: e0bb1a92.
- Arithmetic/coercion component PARTIAL (require_object, object_register, smi helpers, strict_equal, equal_zero migrated; `from_record` bridges left in dispatch/property.rs + dispatch/arithmetic.rs; the value-coercion chain add_values/to_primitive/etc. NOT migrated): 4edd5d16.
- Bridge: LlIntState-from-overlay + finish_frame overlay-direct caller write + Continue-PC-from-snapshot fix + **EAGER reconstruct** (lazy deferred — it hung async then corrupted superPropChains; throwaway): d72d8d10, be2deb7f, 40bffb0b.

**KEY FACTS for the remaining migration:**
- The −15% reconstruct is STILL on the hot path (eager). It disappears only when the LAST `&FrameRecord` snapshot consumer is migrated and the snapshot is deleted → perf is all-or-nothing at the very end.
- ~207 methods take `frame: &FrameRecord`/`&mut FrameRecord`, threaded transitively. Subagent-hostile (they sprawl/wall).
- The realm wall (`Vm::frame_record_realm(agent, frame)` reads `frame.callee()`) is resolved by routing callers to `Vm::realm_of(agent, cfr)` (already cfr-based, vm.rs:1980).

**Remaining steps (the efficient mechanic — de-field-ify decouples the hard part):**
1. **De-field-ify** (behavior-preserving, per-method-local, ANY order, Test262-gated): in every `frame: &FrameRecord`/`&mut FrameRecord` method, replace header-field reads `frame.callee()/this_value()/this_state()/lexical_env()/variable_env()/private_env()/new_target()/construct_this()/return_register()/flags()/kind()/handler_cursor()` with `self.frame_header(Self::cfr_of(frame)).<field>()` (hoist `let cfr = Self::cfr_of(frame);` once). Leave `frame.registers()/code()/instruction_offset()` (geometry — FrameView provides). EXCLUDE: `frame_record_realm` (replaced by `realm_of(cfr)` at call sites in step 2), `trace_all_frame_edges` (GC synthetic record), generator save/restore that BUILDS a FrameRecord for push. After this, every frame-param method uses `frame` only for geometry.
2. **Signature swap, leaves-up** (with `from_record` at the moving frontier): change `frame: &FrameRecord`/`&mut FrameRecord` → `frame: FrameView`; bodies' `registers()/code()/instruction_offset()` unchanged; `&mut` PC-mutators (`advance_dispatch_frame`/`finish_abc_value_result`) redirect the PC bump to the thin-view `DispatchState.pc`; `frame_record_realm(agent,frame)` call sites → `self.realm_of(agent, frame.cfr())`. Caller of a migrated method either passes its own FrameView or `FrameView::from_record(frame)` (temporary).
3. **Remove snapshot**: delete `DispatchState.frame` + `from_record` + `reconstruct_frame_from_header`/`write_snapshot_into_backing`/`refresh_dispatch_frame`; **switch the Refresh arm + sync_from_asm from eager reconstruct to NOTHING** (no snapshot to reconstruct) — this is where the −15% disappears. Compile-error-driven.
4. **Validate**: full Test262 = baseline + clean v8 A/B (baseline worktree `1b39a0ec` vs branch, 7 samples, under /tmp/memtest.sh, /tmp reports) confirming the call/return regression recovered.

### OBSTACLE + DECISION (2026-06-01): synthetic frames → uniform overlay-backed frame model

De-field-ify (1db32690) was REVERTED: `Self::cfr_of(frame)` underflows on SYNTHETIC frames — internal-call/builtin/job `FrameRecord`s built with `RegisterWindow::new(0, 0)` (12 sites: modules.rs, bytecode_calls.rs ×5, runtime_objects.rs ×4, call.rs, jobs.rs) that are NOT backed by a real arena `[header][window]` slot and have no overlay. The overlay-addressing migration cannot represent them. (These are only ever `caller_frame`-style PARAMS to helper methods; the live dispatch snapshot is always a real frame, so the lazy reconstruct is unaffected by this.)

**USER DECISION: commit to the full drop (mega-refactor).** New prerequisite **Step 0** (before de-field-ify / swap):

0. **Uniform overlay-backed frame model**: every frame that is threaded as a `&FrameRecord` must have a real arena slot + overlay. Convert the ~12 synthetic `RegisterWindow::new(0, 0)` frame constructions to reserve a real (possibly zero-width) arena frame + write its `FrameHeader` (so `cfr_of`/`frame_header(cfr)` are valid), OR establish that the field reads on these frames can be sourced another way. Investigate each site's role first (is it pushed/made-current, or only used for env/realm derivation?). Gate with full Test262 at each step. This is a deep change to the internal-call/builtin/job/module machinery and is the bulk of the new risk.

Then proceed with steps 1–4 above (de-field-ify → signature swap → remove snapshot → validate). Perf (−15% removal) lands only at step 3 (snapshot deletion removes the reconstruct). This is a multi-week, multi-session effort; durable state = commits + this plan + the task tracker.

### REFINED Step 0 (after synthetic-frame investigation, 2026-06-01)

GOOD NEWS — the synthetic frames are FEW, not pervasive. Of 14 zero-width `FrameRecord` constructions: **3 are already arena-backed** (production job root jobs.rs:773 via reserve_frame(0)+push; 2 test roots) — leave them. The other **11 are "param-only"** (never pushed, passed to a helper to read `code`/`lexical_env`/`private_env`/realm) and **~10 are in TEST functions**. Production param-only sites: `initialize_module_hoisted_functions` (modules.rs) + the helpers that receive a `caller_frame` in the internal-call path (`prepare_bytecode_call`, `create_closure`, `get_property_from_value`). (The async-resume→getter underflow proves at least one production internal-call path threads a synthetic `caller_frame` into `prepare_bytecode_call`.)

**Step 0 = Option 2 (NOT a deep arena rework):** refactor the few production helpers that take `caller_frame: &FrameRecord` purely for field/realm derivation to instead take the EXPLICIT fields they use:
- `prepare_bytecode_call`: reads `caller_frame.lexical_env()` (MissingEnvironment fallback) + `frame_record_realm(caller_frame)` (realm fallback, "unreachable in practice"). → take `caller_lexical_env: EnvironmentRef` (+ derive realm from the callee's `[[Realm]]`, dropping the caller-frame realm fallback or passing realm explicitly). Update ~all callers (real callers pass the active frame's lexical_env; the internal-call path passes its context env).
- `create_closure` (runtime_objects): reads `code`/`lexical_env`/`private_env` → take those explicitly.
- `get_property_from_value` (call.rs): reads frame only for realm → derive realm differently or pass it.
- `initialize_module_hoisted_functions` (modules.rs): same — pass the env/code explicitly to `create_closure`.
Update the ~10 TEST sites to match the new signatures (drop the synthetic `FrameRecord`). After Step 0, NO production path threads a synthetic frame into a method that will become overlay-addressing, so de-field-ify (step 1) + the FrameView swap (step 2) are unblocked. Gate each with full Test262.

### Step 0 PROGRESS (2026-06-01)
- **Task 1 DONE (61589187, Test262 baseline):** `prepare_bytecode_call` + `lexical_call_state` take explicit fields (`caller_lexical_env`/`caller_this_value`/`caller_new_target`) instead of `caller_frame`; realm fallback → `current_realm_of(agent)`. Callers (incl. internal_calls, the underflow path) pass the fields off their local frame.
- **Task 2 DONE (96805b40, Test262 pending):** `create_closure` takes explicit `enclosing_code/lexical_env/private_env/realm`; `initialize_module_hoisted_functions` (modules.rs) synthetic `FrameRecord` DELETED. Test callers updated.
- **`get_property_from_value` ESCALATED (deferred to step 2):** its frame is NOT realm-only — it threads through the whole `VmProxyBridge` getter/setter dispatch chain (15+ methods: `get_property_from_object`, `call_property_getter`, `get_own_property_from_object`, `set_property_on_object`, …). In PRODUCTION this frame is REAL (only a *test*, call.rs:1262, uses a synthetic frame there), so it is NOT a Step-0 production blocker. It is a big property-access threading chain handled in step 2 (FrameView swap), where the synthetic case must be watched.
- **REMAINING Step 0:** essentially complete for PRODUCTION synthetic frames (the two production helpers fixed). Leftover `RegisterWindow::new(0,0)` are test-only (runtime_objects.rs:1320, call.rs:1262) — they can stay until their callee migrates.

### CAUTION for de-field-ify (step 1) and the swap (step 2)
- The `VmProxyBridge` chain (rooted at `get_property_from_value`/`get_property_from_object` in call.rs/property_access.rs) threads a frame that CAN be synthetic in the async-resume→getter path. Do NOT de-field-ify (overlay-address `cfr_of(frame)`) methods in that chain until it is converted to explicit fields / FrameView with the synthetic case handled. De-field-ify must skip the VmProxyBridge frame-threading methods.

### Step 2 (swap) PROGRESS + ESTABLISHED PATTERNS (2026-06-01)

**DONE + green (Test262 baseline at each):**
- Env/scope/frame-state methods → FrameView (7f29a33f): direct_eval_env, loop_iteration, + names leaves (enter/leave_env_scope, delete_global, frame_is_strict, load_captured_name_this). 61 temporary `FrameView::from_record` bridge sites at the frontier (shrink as callers swap up).
- **Property/proxy hub CONVERTED (120b54d9, Test262 baseline) — the hardest piece.** `VmProxyBridge` holds `caller_realm`/`caller_lexical_env`/`caller_code`/`caller_pc` (not a frame); the whole property/coercion chain (`get_property_from_value`, `set_property_on_value`, `property_key_from_value`, `to_primitive`, `copy_data_properties`, getter/setter invocation, ~22 fns) takes those explicitly; dispatch/property.rs entry points extract them once from the live frame (`frame_record_realm(agent,frame)`+`frame.lexical_env()`+`frame.code()`). 2 test synthetic frames deleted.

**ESTABLISHED PATTERNS for the remaining ~201 frame-param signatures (by file: arithmetic 36, names 28, call 15, values 14, generators 13, super_ops 10, runtime_objects 9, private_fields 8, bytecode_calls 7, vm.rs 6, dispatch.rs 5, internal_calls 4, builtin_dispatch* ~20, …):**
- **Geometry-on-a-REAL-frame methods** (arithmetic/values/dispatch register reads, generators geometry): `frame: &FrameRecord` → `frame: FrameView`; `.registers()/.code()/.instruction_offset()` unchanged. Where they call the now-explicit property/coercion chain (e.g. `to_primitive`), derive realm/lexical_env on demand from the REAL frame's cfr: `self.realm_of(agent, frame.cfr())` / `self.frame_header(frame.cfr()).lexical_env()` (valid because these frames are never synthetic).
- **Realm/context consumers that CAN see a synthetic frame** (the call_to_completion/construct/builtin-dispatch chain — call.rs/builtin_dispatch*/internal_calls): use the property-hub pattern — take explicit `caller_realm`(+lexical_env); extract at the entry/caller off the FrameRecord directly. (call_to_completion/construct_to_completion still take `&FrameRecord` for the dispatch-call path; assess whether they can take FrameView since their production frame is real, or explicit realm.)
- **class-helpers super_ops/private_fields**: read/write `this`/`construct_this` — already overlay-direct via `frame_header_mut(cfr)`; swap their frame param to FrameView and read/write the overlay via `view.cfr()`.
- **Leaves-up + `from_record` at the frontier**; semantic-body callers pass `inner.frame_view()`. Gate each component with full Test262.
- **FINAL (step 3)**: when NO method takes `&FrameRecord`, delete `DispatchState.frame` + `reconstruct_frame_from_header` + the eager-reconstruct in the Refresh arm/sync_from_asm/finish_frame + `from_record` → the −15% reconstruct vanishes. Then validate (Test262 + v8 A/B).

### THE HARD CORE (remaining ~124 sigs; 2026-06-01) — PC-CARRIER FLIP

**Done + green (Test262 baseline at each): arithmetic (45a5f40f), names (edb85055)** — these are "pure" methods: they return `VmResult` and the SEMANTIC BODY (e.g. `route_binary_result`) handles PC-advance + exception via `finish_abc_value_result(&mut snapshot)`/`handle_dispatch_result(&mut snapshot)`. The snapshot is the PC carrier the **Continue arm reads** (`rust.dispatch.frame.instruction_offset()`, set in be2deb7f). Pure methods swap to FrameView trivially because they DON'T touch the PC carrier.

**LESSON (two hangs cost):** the snapshot is the PC carrier. `finish_abc_value_result`/`advance_dispatch_frame`/`handle_dispatch_result`/`finish_frame` advance/read it; the Continue arm reads it. A `FrameView` is `Copy` — advancing a FrameView's PC writes a discarded local → the advance is lost → async-generator resume loops forever. So these PC/exception helpers (and the dispatch/property `execute_*` ops that call them INTERNALLY) CANNOT take a FrameView. (5b057225 tried `*_view` helpers → reverted at 16ce3e9a-style revert.)

**REQUIRED before the remaining swaps + deletion — the PC-CARRIER FLIP (do this as ONE careful, consistent change; gate hard with the async-generator suite as the canary):**
1. Make `DispatchState.pc` (thin view) the sole PC carrier. `sync_from_asm` already sets `rust.dispatch.pc = frame_pc_offset`.
2. `finish_abc_value_result` / `advance_dispatch_frame` / `next_dispatch_instruction_offset` / `handle_dispatch_result` / `finish_frame`: advance/read `DispatchState.pc` (pass `&mut pc` + cfr/regs_len/code, or `&mut DispatchState`) instead of `&mut snapshot`. Park PC into the overlay for exception transfer via `frame_header_mut(cfr).set_saved_pc(pc)`.
3. **Continue arm** (`dsl/slow_path.rs`): read `rust.dispatch.pc` (NOT `rust.dispatch.frame.instruction_offset()`).
4. Restructure the **dispatch/property `execute_*` ops**: they currently call finish/handle_dispatch_result INTERNALLY (as Vm methods, no DispatchState). Move PC/exception handling OUT to the semantic bodies (`semantics/property.rs`) — the body calls `handle_dispatch_result`/finish on the DispatchState (like `route_binary_result`); the vm method returns `VmResult<Value>`. THEN the vm method swaps to FrameView (geometry-only).
5. Then swap the remaining pure-ish components: generators (13 — careful: resume/suspend frame-switch + cold handler_cursor via `frame_cold.get(depth_of(cfr))`), class-helpers super_ops/private_fields (this/construct_this overlay-direct via cfr), runtime_objects, call/builtin-dispatch chain (call.rs/builtin_dispatch* — frame-switch; assess synthetic frames; likely explicit-realm like the property hub), misc (vm.rs/exceptions/async/dispatch.rs).
6. Delete the snapshot (step 3 above). The eager-reconstruct + reconstruct_frame_from_header go away → −15% recovered.

Gate EVERY step with the lyng-vm async_and_generators suite (the hang canary) + full Test262. The hard core is hang-prone; do it carefully and sequentially, not as wide parallel batches.

### SESSION CHECKPOINT (2026-06-01): 201 → 101 sigs done; the coupled core remains

**DONE + green (Test262 baseline at each commit):** property/VmProxyBridge hub (120b54d9); arithmetic/coercion (45a5f40f); names/global-access (edb85055); env/scope (7f29a33f); class-helpers super_ops/private_fields/class_helpers + runtime-objects iterators (310bbd11); Step 0 (61589187, 96805b40) + de-field-ify (5a2bdcbb) + bridge (d72d8d10/be2deb7f/40bffb0b). HEAD builds green; `cargo test -p lyng-vm` 609/0; whole-corpus Test262 49729/0/0.

**REMAINING ~101 sigs = ONE coupled hang-prone core** (do NOT attempt as wide parallel subagent batches — it cost two hangs):
- `dispatch/property.rs` (~15), `call.rs` (~15, 9 internal finish/handle calls), `generators.rs` (~13, frame-switch + cold handler_cursor), `builtin_dispatch*` async-iterator-state + call_builtin chain (~25), `dispatch.rs` (finish_abc_value_result/handle_dispatch_result/advance_dispatch_frame/next_dispatch_instruction_offset — the PC-carrier helpers), `with_env.rs` (push/pop_with — frame_record_realm + lexical_env mutate), `vm.rs`/`internal_calls`/`async_functions`/`exceptions` misc.
- These all call the PC/exception helpers INTERNALLY (as Vm methods, no DispatchState.pc), so they cannot take a `FrameView` (Copy → lost PC). 

**EXACT NEXT MOVE (the PC-carrier flip + defer-to-body restructure — see "THE HARD CORE" section above for the 6-step design):** restructure these vm methods to RETURN VmResult and let their SEMANTIC BODIES handle PC-advance + exception (exactly like arithmetic's `route_binary_result` does with `finish_abc_value_result`); flip `finish_abc_value_result`/`advance_dispatch_frame`/`handle_dispatch_result` + the slow_path Continue arm from the snapshot to `DispatchState.pc`; then swap the now-pure vm methods to `FrameView`; then delete `DispatchState.frame` + `reconstruct_frame_from_header` + the eager reconstruct → the −15% recovers. Gate each sub-step with the async_and_generators canary + full Test262. Do this with FRESH context, sequentially — it is the delicate, hang-prone crux.

**Then validate (step 4): full Test262 = baseline + clean v8 A/B (baseline worktree 1b39a0ec vs branch, 7 samples, /tmp/memtest.sh, /tmp reports) confirming the call/return regression recovered.**

### SESSION CHECKPOINT (2026-06-01 #2): defer-to-body restructure STARTED on dispatch/property.rs (6 methods green)

**Strategy decision (locked by user this session): "Plan's defer-to-body"** (THE HARD CORE step 4) — make each `execute_*` helper pure and move handle/advance into the semantic body. Crucial realization that de-risked it: the defer-to-body restructure is **behavior-preserving** because the snapshot stays the PC carrier — moving *where* `handle_dispatch_result`/advance are called (internal → body) changes nothing observable. The actual snapshot→`DispatchState.pc` flip is the FINAL localized step, only after NO `execute_*` calls handle/advance internally. So per-method hang risk is ~zero; the hang risk is concentrated in the final flip. Gate per-method with the vm suite (fast, ~12s) + full Test262 at cluster boundaries.

**Two established body-tail archetypes (mirror `op_negate_semantic`, NOT `finish_abc_value_result`):**
- **Read-shape** (method → `VmResult<Value>`): body does `let handled = inner.handle_dispatch_result(result); match { Ok(Some(v)) => v, Ok(None) => return Continue{0}, Err(e) => return ExitError{e} }; let registers = inner.registers(); inner.vm.write_register(registers, target, value); Continue{instruction_len}`.
- **Write-shape** (method → `VmResult<()>`, no register write): body does `match inner.handle_dispatch_result(result) { Ok(Some(())) => Continue{instruction_len}, Ok(None) => Continue{0}, Err(e) => ExitError{e} }`.
- The pure method: replace each `let Some(x) = self.handle_dispatch_result(agent, frame_depth, frame, R)? else { return Ok(()) }` with `let x = R?;`, each IC-fast-path `write+advance+return Ok(())` with `return Ok(value)`, drop `frame_depth`/`instruction_len`/`target` params, keep `frame: &mut FrameRecord` (FrameView swap is a SEPARATE later step). Equivalence proof: only the first abrupt completion short-circuits, so per-step vs end-of-method catch are observationally identical; store side-effects persist across caught throws in both forms.

**DONE + green (each: build + vm 609/0; Test262 baseline 49729/0/0 where noted):**
- `execute_in_opcode` (124fc8b2) — read-shape. **Test262 baseline ✓** (pattern proof).
- `execute_get_named_property_opcode` (e816ae33) — read-shape.
- `execute_get_keyed_property_opcode` (1d62050f) — read-shape. **read cluster Test262 baseline ✓** (t262-reads.md).
- `execute_set_named_property_opcode` (2722db3c) — write-shape.
- `execute_to_property_key_opcode` + `execute_delete_property_opcode` (0db79e22) — read-shape. **writes-batch Test262 baseline ✓** (t262-writes.md: set_named + to_property_key + delete = 49729/0/0).
- `execute_copy_data_properties_opcode` + `execute_store_dense_element_opcode` (write-shape) + `execute_load_dense_element_opcode` (read-shape) (9687592b) — vm 609/0 + clippy clean; **dense-batch Test262 baseline ✓** (t262-dense.md: 49729/0/0). NOTE: this batch (and get_keyed earlier) routes the previously-`?`-bypassed `mapped_arguments`/`try_direct_own_index`/`try_direct_typed_array` abrupt paths through the body's `handle_dispatch_result` — i.e. now CAUGHT (consistent with the method's other paths), where before they fell through to `route_execute_result`→`ExitError` (uncaught). Behaviorally these paths don't produce catchable throws in the tested corpus (Test262 baseline both times); the caught behavior is the spec-correct one. Also fixed clippy this batch: dropped `too_many_lines` on get_named's expectation (shrank <100 lines), simplified two get_keyed let-and-return arms, kept `too_many_arguments` on the 8-param copy_data/store_dense (clippy counts `self`, so ≤7 real args = no attribute, 8+ params = keep it).
- `execute_define_named_property_opcode` + `execute_define_keyed_property_opcode` + the nested `fn define_data_property` made pure (3788ccd9) — write-shape; **define-batch Test262 baseline ✓** (t262-def.md: 49729/0/0). Resolved the extensions.rs question: `extensions.rs::define_data_property` is a DISTINCT 6-arg embedding API on another type, NOT the private `Vm::define_data_property` — the private one's only callers are the two define opcodes, so it was free to make pure (drop frame_depth; the define + created==false TypeError propagate via `?`). The previously-internal define catch now happens at the body (consistent); fixes a latent caught-then-advance double-advance that was unexercised.
- `execute_set_keyed_property_opcode` (18027379, subagent-authored) — write-shape, the LAST property method. Whole-method replacement: all handle points → `?`, internal advances removed; body `op_set_keyed_property_shared` → write-shape route. Side-effect of completing it: `route_execute_result` is now DEAD repo-wide (set_keyed was its last caller) → marked `#[expect(dead_code, reason=…)]` (kept for forthcoming non-property opcode families). vm 609/0 + clippy clean; **set_keyed Test262 baseline ✓** (t262-setk.md: 49729/0/0).

**`dispatch/property.rs` defer-to-body is COMPLETE — all 12 execute_* methods converted.** The create/misc opcodes (CreateObject/CreateArray/SetFunctionName/CheckObjectCoercible/ThrowIfUninitialized) were ALREADY body-driven (write reg + Continue / route via handle_dispatch_result directly) — no execute_* change needed.

**`dispatch/property.rs` FrameView swap DONE (f0643b58, subagent)** — **Test262 baseline ✓** (t262-fv.md: 49729/0/0). 14 of 15 frame-param functions (the 12 execute_* + `define_data_property` + `check_property_assignment_result`) now take `frame: FrameView` instead of `&FrameRecord`; header reads route via the overlay (`realm_of(frame.cfr())`, `frame_header(frame.cfr()).lexical_env()`), geometry via FrameView (`registers()`/`code()`/`instruction_offset()`); semantic bodies pass `inner.frame_view()`. Behavior-preserving (property opcodes always run on the active REAL frame, so overlay-by-cfr == snapshot). KEY EXCEPTION: `try_assign_named_property_rust_probe_for_dsl` (property.rs:~237) stays `&mut FrameRecord` — it WRITES the PC via `advance_dispatch_frame` (it's the asm assign-named rust-probe; caller `dsl/handlers/cold.rs:~2855` passes `&mut dispatch.frame` and reads back the advanced `instruction_offset()`). FrameView is Copy (no PC setter), so this PC-writer can't swap until the PC-carrier flip — it's part of the flip's surface. So `FrameRecord` is still imported/used in property.rs (the one PC-writer).

**GOTCHAS hit (each cost a cycle):**
- The `llint_architecture::rust_vm_hot_paths_do_not_use_llint_fast_path_terminology` meta-test BANS the substrings "fast path"/"fast-path"/"fast_"/"_fast" in `dispatch/property.rs` (+ names/call/bytecode_calls/builtin_dispatch). Do NOT write "fast path" in comments there — use "cache-hit path".
- Write-shape methods' `if assignment { let assignment_result = ...; let Some(()) = handle(...)? else {...} }` snippet is TEXTUALLY IDENTICAL across set_named / set_keyed / define_*. NEVER use file-wide `replace_all` — it silently mis-transforms not-yet-migrated methods (whose bodies still use `route_execute_result`, which does NOT catch → a propagated throw becomes ExitError, a correctness bug). Use whole-method replacement or method-unique-context targeted edits.
- HARNESS LSP E0061/E0308 diagnostics after a sig change are STALE (show the pre-edit body). Trust `cargo build -p lyng-vm --all-features` (exit 0 / 0 `^error`).

**REMAINING (next session) — dispatch/property.rs is DONE; the frame-param families left:**
- `call.rs` (~15), `generators.rs` (~13), `builtin_dispatch*` chains, misc (vm.rs/internal_calls/async_functions/exceptions). MORE complex than property.rs: frame-switch, possibly-synthetic frames (use the explicit-realm/explicit-fields pattern the property hub used — do NOT overlay-address a synthetic frame's cfr), and generators' cold `handler_cursor` + resume/suspend. Same defer-to-body principle, but assess each method's frame role first. Apply the same per-batch gate (build + clippy + vm 609/0, Test262 at batch boundaries) and the same read/write body-tail archetypes. A single scoped subagent per file/cluster works well for the mechanical ones (set_keyed proved it) — but keep them SEQUENTIAL, never wide-parallel on this coupled core.

**THEN the PC-CARRIER FLIP** (steps 1–3 of THE HARD CORE): once NO `execute_*` calls handle/advance internally, flip `finish_abc_value_result`/`advance_dispatch_frame`/`handle_dispatch_result` + the slow_path Continue arm from the snapshot to `DispatchState.pc` (park to overlay via `set_saved_pc` for exception transfer; handle's caught path sets `*pc = frame_header(cfr).saved_pc()`). Gate HARD with the async/generator suite + Test262 — this is the hang-prone step. **THEN** swap pure methods to FrameView + delete `DispatchState.frame`/`reconstruct_frame_from_header`/eager reconstruct → −15% recovers. **THEN** v8 A/B validation.

### call.rs MAP + DESIGN FINDING (2026-06-01 #3) — call.rs is NOT a mechanical property.rs repeat

Mapped `crates/vm/src/vm/call.rs` (read-only). **15 frame-param functions. Crucial: call.rs calls `handle_dispatch_result` ZERO times and `advance_dispatch_frame` 9 times.** So *exception*-deferral is ALREADY done — call errors propagate via `?` to the semantic bodies (`semantics/calls.rs`: op_call_semantic→`call_value`:344, op_call0/small→`call_value_small`:419, op_tail_call→`tail_call_value`:739, op_construct→`construct_value`:800), which route through `handle_dispatch_result`. The ONLY internal PC concern is `advance_dispatch_frame`.

**THE DESIGN CRUX (why call.rs ≠ property.rs):** call.rs's `advance_dispatch_frame(frame, len)` advances the **CALLER** frame's PC *at the moment just before a frame switch* (entering the callee / after a synchronous builtin). It CANNOT be deferred to the semantic body the way property.rs's was — by the time the body regains control, the active frame is the CALLEE (or the call already Refreshed), so the body's `DispatchState.pc` is no longer the caller's. The caller-PC advance is entangled with the frame switch. **So the call/construct PC-advance must be handled AS PART OF the PC-carrier flip itself**, not pre-converted: at the flip, these advances must target the caller's `DispatchState.pc` directly at the call site (pre-switch). This is precisely why call/return is the hang-prone cluster. NEXT SESSION: design the frame-switch PC-advance↔flip interaction FIRST (how does the caller's pc get advanced+parked before Refresh, and where does the callee's pc come from on entry — `sync_from_asm`/Refresh already source it from the overlay) before touching call.rs.

call.rs categorization from the map (line #s):
- **(A) geometry-only / mechanical** (frame used only for `registers()`/`code()` + IC observe, advance is the caller-pre-switch case): `try_invoke_cached_builtin_call`:169, `invoke_bytecode_call_from_caller_arg_window`:595, `invoke_bytecode_construct_from_caller_arg_window`:647, `collect_arguments_into`:1096, `append_spread_argument`:1123, `finalize_frame_result`:1136 (flags only). These can swap frame→FrameView for their geometry use.
- **(B) realm/synthetic-frame-sensitive** (`Self::frame_record_realm`, lexical_env, the call_to_completion/construct/builtin/tail chain — can see synthetic frames): `invoke_call_target`:22, `invoke_collected_call_value`:123, `invoke_function_call_builtin_target`:204, `invoke_tail_call_target`:272, `call_value`:344, `call_value_small`:419, `tail_call_value`:739, `construct_value`:800 (MOST delicate: 3 advances, multiple realm derivations, proxy/bytecode/builtin paths). Use the property-hub explicit-realm/fields pattern; never overlay-address a synthetic cfr.

---

### THE PC-CARRIER FLIP — VERIFIED MODEL + DECOMPOSITION (2026-06-01 #4, fresh-context audit)

Full read-only audit of `vm/dispatch.rs`, `dsl/slow_path.rs`, `vm/registers.rs`, `vm/dispatch_state.rs`, `vm/call.rs` (advance sites), and the `vm.rs` push/pop/park/reconstruct machinery. The model below is the authoritative one for the flip.

**FOUR PC representations + how they relate:**
1. **asm** — `LlIntState.frame_pc_offset`: what asm dispatch reads/advances on the pure-asm hot path.
2. **snapshot** — `DispatchState.frame.instruction_offset()`: TODAY the same-frame PC carrier across a slow-path body.
3. **thin view** — `DispatchState.pc`: a *mirror* of the snapshot PC, synced at boundaries; already read by `decode_current_*` + `current_instruction_offset` (Phase 1). **Flip target = make THIS the sole same-frame carrier.**
4. **overlay** — `FrameHeader.saved_pc` (per cfr): the PARKED PC for non-active frames + exception transfer. `set_saved_pc`/`saved_pc` at `frame_header.rs:98,103`. Read back only at frame switches (`reconstruct_frame_from_header`, the `Refresh` arm). Seeded on push by `write_header_from_record` (vm.rs:1691, `= record.instruction_offset()`).

**The carrier loop today (verified):**
- **Entry** (`sync_from_asm`, slow_path.rs:123): lazily reconstruct snapshot if `frame_dirty`; then `snapshot.set_instruction_offset(frame_pc_offset)` + `dispatch.pc = frame_pc_offset`. ⇒ snapshot.pc == dispatch.pc == asm.pc on entry.
- **Same-frame opcode** — TWO production patterns:
  - *finish_abc style* (arithmetic only; `finish_abc_value_result`, 2 callers = `route_binary_result`/`route_binary_smi_result`): write reg → `advance_dispatch_frame(snapshot, len)` bumps **snapshot.pc only** → body returns `Continue{pc_advance:0}`.
  - *explicit-advance style* (property defer-to-body, names, control_flow): body touches NO PC → returns `Continue{pc_advance:len|jumpΔ}`.
- **Continue arm** (slow_path.rs:251): `new_offset = snapshot.pc + pc_advance; dispatch.pc = new_offset; asm.frame_pc_offset = new_offset`. Reconciles both (finish_abc: snapshot advanced, Δ=0; explicit: snapshot=entry, Δ=len).
- **Call** (call.rs, 9 sites — each is `advance_dispatch_frame(caller_snapshot, len); self.sync_dispatch_frame(depth, *caller_snapshot)`): advance the CALLER PC, then **park caller snapshot → caller overlay** (`write_snapshot_into_backing` writes this/lex/construct + **saved_pc = entry+len** + cold). Push callee (seeds callee overlay saved_pc). Body returns `Refresh`. Refresh arm reads NEW current cfr (callee) saved_pc.
- **Return** (`finish_frame`, registers.rs:96): pop callee, write retval to caller window (overlay-direct, NO snapshot), restore current_cfr. Op returns `Refresh`; Refresh arm reads caller overlay saved_pc (= the parked entry+len) → caller resumes right after the Call. **finish_frame carries NO PC.**
- **Caught throw** (`handle_dispatch_result`, dispatch.rs:239): `sync_dispatch_frame` parks current PC; `transfer_to_exception_handler` rewrites the handler frame's overlay saved_pc to the catch PC (+ unwinds frames if cross-frame); then **same-frame**: `refresh_dispatch_frame` reconstructs snapshot from current cfr ⇒ snapshot.pc = catch PC; **cross-frame**: `refresh_dispatch_frame`'s `index < frame_depth()` guard is FALSE ⇒ snapshot left STALE, and the Continue arm's epoch+depth check (slow_path.rs:212) promotes `Continue`→`Refresh` which reloads from the handler frame's saved_pc.

**KEY INSIGHT — the ONLY same-frame divergence between snapshot.pc and dispatch.pc is (1) `finish_abc`'s `advance_dispatch_frame` and (2) the same-frame caught-throw `refresh_dispatch_frame` (which sets snapshot.pc but NOT dispatch.pc).** The call/gen/async advance+park sites are FRAME-SWITCH ops (end in `Refresh`, park to overlay) — they never feed the same-frame Continue arm, so they are NOT part of the same-frame carrier flip; they convert later, at their FrameView swap (park via overlay `set_saved_pc` keyed on the caller cfr). So the same-frame flip is SMALL and isolatable.

**DECOMPOSITION (each behavior-preserving + async-canary + Test262 gated):**
- **F1 (lockstep, no behavior change):** `finish_abc_value_result` also advances `dispatch.pc` (thread `pc: &mut u32`; both route helpers destructure `pc`). Nothing reads dispatch.pc on this path before the Continue arm, so observationally inert.
- **F3 (caught-throw thin-view refresh, no behavior change yet):** in the `DispatchState::handle_dispatch_result` WRAPPER (dispatch_state.rs:259), on the `Ok(None)` (caught) result, if **same-frame** (`self.vm.frame_depth() == self.frame_depth`) set `self.pc = self.vm.frame_header(self.cfr).saved_pc()` (the handler PC `transfer` parked). Leave cross-frame STALE (the Continue→Refresh promotion needs the stale `frame_depth` to fire, and Refresh reloads the thin view). After F1+F3, dispatch.pc == snapshot.pc on every Continue-returning path.
- **F2 (THE FLIP — behavior-activating; gate HARD, async canary):** Continue arm reads `rust.dispatch.pc` (not `rust.dispatch.frame.instruction_offset()`) as the base. No-op iff F1+F3 hold the equality — but this is where the two prior hangs lived, so gate with async_and_generators + full Test262.
- After F1–F3+F2, `dispatch.pc` is the authoritative same-frame carrier; the snapshot stays maintained only for the not-yet-FrameView-swapped readers (call/gen/async/builtin/misc) and the frame-switch park/reconstruct.
- **Then (separate, larger):** FrameView-swap call.rs/generators/async/builtin/misc; at each advance+park site replace `advance_dispatch_frame(frame,len)+sync_dispatch_frame(...)` with overlay park `frame_header_mut(view.cfr()).set_saved_pc(view.instruction_offset().wrapping_add(len))` (caller PC base = view.instruction_offset() == dispatch.pc; fields already write-through so the field-park is redundant — VERIFY via gate). Flip generators' `next_dispatch_instruction_offset(&frame,len)` → `inner.pc()+len`. Flip the property rust-probe + its cold.rs reader to dispatch.pc.
- **Then (deletion = where −15% lands):** delete `DispatchState.frame` + `frame_dirty`; `sync_from_asm` → just `dispatch.pc = frame_pc_offset`; `Refresh` arm → drop the eager reconstruct; `finish_frame`/`pop_current_frame` → return-register+window from overlay (no reconstruct); remove `reconstruct_frame_from_header` (iff no generator-restore caller) / `write_snapshot_into_backing` / `refresh_dispatch_frame` / `sync_dispatch_frame` / `advance_dispatch_frame` / `next_dispatch_instruction_offset` / `FrameView::from_record`.
- **Then validate:** full Test262 = baseline + clean v8 A/B (baseline worktree `1b39a0ec`, 7 samples, /tmp/memtest.sh) showing the call/return cluster recovered.

### SESSION #4 EXECUTION — same-frame carrier flip done (F1 + unified-F3 + op_await + F2)

**F1 (committed):** `finish_abc_value_result` gains `pc: &mut u32` and writes `*pc = frame.instruction_offset()` after the advance/caught-refresh (both arithmetic route helpers thread `pc`). Covers the `finish_abc` (arithmetic route_binary*) bodies — these call the vm `handle_dispatch_result` directly, not the wrapper.

**F2 FIRST ATTEMPT HUNG (async/generator, memory climbing — killed at 7GB).** Root cause: F1 only covered `finish_abc`. The complete criterion is: **a body returns `SemanticOutcome::Continue { pc_advance: 0 }` exactly when a helper has pre-advanced (success) or refreshed-to-handler-PC (same-frame caught) the snapshot; for every such path `dispatch.pc` must equal `snapshot.pc` at the Continue arm, else F2 re-executes the same instruction forever.** Missed sites: `op_await` (direct `await_value`), `op_delegate_yield`'s Complete-success (via `finish_delegate_yield_outcome`), and EVERY `Ok(Some)`-success wrapper body (the original caught-only F3 synced only `Ok(None)`).

**The complete fix (all green: canary 35/35, vm 609/0):**
- **Unified F3** (`DispatchState::handle_dispatch_result` wrapper): sync `self.pc = self.frame.instruction_offset()` on EVERY same-frame return (drop the `is_none()` guard). This makes `dispatch.pc == snapshot.pc` at the Continue arm for **all wrapper-routed bodies**, so F2's `dispatch.pc + pc_advance` is *provably identical* to the legacy `snapshot.pc + pc_advance` (the body does only a register-write between the wrapper call and its `Continue` return — neither PC changes). Cross-frame results (caught-throw unwind, or a call that pushed a callee) change `frame_depth` → guard skips → the egress's `Continue`→`Refresh` promotion reloads the thin view. Covers property/names/scope/iterators/control_flow/calls/arith-unary(negate/bit_not/update)/delegate_yield.
- **op_await body sync** (`semantics/generators.rs`): `await_value` is NOT wrapper-routed, so mirror its finalized snapshot PC (`inner.pc = inner.frame.instruction_offset()`) before its `Continue{0}` (await's caught resume-throw is always same-frame and doesn't bump the epoch).
- **F2** (`dsl/slow_path.rs` Continue arm): `new_offset = rust.dispatch.pc.wrapping_add(pc_advance)`.

**Not needing a sync (verified):** yield/suspend `Ok(())=>Continue{0}` arms are UNREACHABLE (helpers always `Err(GeneratorYield/Start)`); `route_execute_result` (property.rs:134) is dead (`#[expect(dead_code)]`); `prefix` bodies don't advance the snapshot and have no production path; the **assign-named rust-probe** (`op_assign_named_property_rust_probe_rs`, cold.rs:2835) reads the snapshot directly and writes `frame_pc_offset` manually — it BYPASSES `translate_outcome`'s Continue arm, so F2 doesn't touch it (the subagent map was wrong here). The call.rs `advance_dispatch_frame`+`sync_dispatch_frame` sites are FRAME-SWITCH (park caller PC → overlay, return `Refresh`) — they never feed the same-frame Continue arm, so they convert later at their FrameView swap.

**LESSON:** the same-frame Continue carrier is fed by MANY bodies, not just `finish_abc`. Enumerate by the precise criterion (`Continue{pc_advance:0}` ⟺ snapshot pre-positioned) and prefer a single wrapper-level choke point (unified F3) whose equivalence to the legacy arm is provable, over hunting individual sites.

**STATUS: same-frame carrier flip COMMITTED + GREEN** — F1 `c733e740`, F2+unified-F3+op_await `61141516`, doc `a7a877f9`. Gates: async_and_generators canary 35/35 (no hang), vm suite 609/0, clippy clean (edited files), whole-corpus Test262 **49729/0/0/3324 baseline** (variants 95205/0/0). `dispatch.pc` is now the sole SAME-frame PC carrier; the snapshot is still maintained (entry sync + boundaries + the call/gen/async frame-switch park) for the not-yet-FrameView-swapped readers and the frame-switch reconstruct.

**EXACT NEXT MOVE (next session, fresh context):** the remaining work is the larger mechanical chunk — NOT a same-frame concern anymore.
1. Convert the FRAME-SWITCH park sites (call.rs ×9 `advance_dispatch_frame(frame,len)+sync_dispatch_frame`, generators.rs:1186/1174/1305/1357, async_functions.rs:161/169/177, runtime_objects.rs) to park via the overlay keyed on the caller cfr: `frame_header_mut(cfr).set_saved_pc(pc_base.wrapping_add(len))` where `pc_base` = the caller's `dispatch.pc` (== `frame.instruction_offset()` while the snapshot lives, == `view.instruction_offset()` after the FrameView swap). The field-park (`this`/`lexical_env`/`construct_this`) is redundant once write-through is confirmed — VERIFY via gate, don't assume. This conversion only matters as it enables the FrameView swap (FrameView is Copy → can't `*frame` for `sync_dispatch_frame`).
2. FrameView-swap the ~124 remaining `frame: &FrameRecord` methods (call/generators/async/builtin_dispatch/misc) per the established patterns (geometry→FrameView; realm/synthetic-frame-sensitive→explicit-realm/fields hub pattern; class-helpers overlay-direct via cfr).
3. Flip the property rust-probe (`op_assign_named_property_rust_probe_rs`, cold.rs:2854/2866) + `next_dispatch_instruction_offset` (generators.rs:144/176) to `dispatch.pc`.
4. DELETE the snapshot (`DispatchState.frame` + `frame_dirty`): `sync_from_asm` → just `dispatch.pc = frame_pc_offset` (no reconstruct); `Refresh` arm → drop the eager reconstruct; `finish_frame`/`pop_current_frame` → return-register+window from overlay; remove `reconstruct_frame_from_header`/`write_snapshot_into_backing`/`refresh_dispatch_frame`/`sync_dispatch_frame`/`advance_dispatch_frame`/`next_dispatch_instruction_offset`/`FrameView::from_record`/`set_pc`/`cfr()` (the `#[expect(dead_code)]` ones get users or go). **−15% lands HERE.**
5. Validate: full Test262 = baseline + clean v8 A/B (baseline worktree `1b39a0ec`, 7 samples, /tmp/memtest.sh, /tmp reports) showing the call/return cluster recovered. Gate every sub-step with the async canary + Test262 at cluster boundaries.

### REMAINING-WORK HAZARD MAP (verified by code-read, session #4) — read before delegating any swap

An inventory subagent was run to classify the ~78 remaining `frame: &FrameRecord` params (G geometry / H header / R realm / P park). **Its P and synthetic classifications had errors — DO NOT trust a classification subagent for these safety-critical calls; verify by code-read.** Verified facts:

- **The realm-consumer chain is SYNTHETIC-EXPOSED and needs the explicit-realm hub pattern, NOT a cfr-swap.** `Self::frame_record_realm(agent, frame)` reads `frame.callee()` *from the FrameRecord struct* (works on synthetic `RegisterWindow::new(0,0)` frames). `Self::realm_of(agent, cfr)` reads the OVERLAY at `cfr` — for a synthetic frame `Self::cfr_of(frame)` UNDERFLOWS (base 0 − HEADER_SLOTS). Verified synthetic-exposed: `call_to_completion` (internal_calls.rs:55) passes its possibly-synthetic `caller_frame` straight into `call_builtin` (builtin_dispatch.rs:86) AND the VmProxyBridge AND `frame_record_realm`; `construct_to_completion` likewise; `resume_generator`/`resume_async_generator`/`resume_async_generator_from_value` (generators.rs:80/175/208) use `caller_frame` ONLY for `frame_record_realm` and a `.next()`/`.throw()` caller CAN be inside a getter (synthetic). ⇒ the entire `call_to_completion`/`construct_to_completion`/`call_builtin`/builtin_dispatch/generator-resume/`await_value` realm chain must take an explicit `caller_realm` (extracted once where the frame is real, computed at the synthetic-frame construction sites — internal_calls.rs:424 `synthetic_caller_frame`, jobs root — which already KNOW their realm) — the same hub refactor that `120b54d9` did for the property/proxy chain. This is the BULK of the remaining R work and is careful DRIVER work.
- **Inventory errors caught:** `finish_delegate_yield_outcome` (generators.rs:1152) is **P** not G (contains `advance_dispatch_frame` at 1186, Complete branch); `close_iterator_state` (runtime_objects.rs:610) contains a `sync_dispatch_frame` (≈642) so it is **P** not G; `select_exception_handler`/`suspended_call_instruction_offset` (exceptions.rs) are fed a *transient reconstruct* (`reconstruct_frame_from_header` at exceptions.rs:20) and read `saved_pc`-as-PC — swapping them to take `cfr` would also delete that per-exception reconstruct, but it's in the delicate transfer path. `trace_all_frame_edges` (state.rs) is the GC synthetic-frame tracer — EXCLUDE (stays `&FrameRecord`).
- **Genuinely mechanical (always-real, no-park, no-synthetic, geometry/registers-only)** surface is SMALL — e.g. `collect_arguments_into`/`append_spread_argument` (call.rs, real active frame, `.registers()` only), `suspend_generator_start`/`snapshot_suspended_execution` (generators, `.registers()` on a real generator frame), the iterator-state readers in runtime_objects that DON'T contain a sync site. Each must be code-verified (not inventory-trusted) for park-freedom + real-frame before a FrameView swap.

**Sequencing recommendation:** (a) explicit-realm hub for the call/construct/builtin/generator-resume/await realm chain (driver; subagent for mechanical caller updates under a precise spec); (b) park-site conversion + FrameView swap of the call.rs/generators/async P methods (driver, flip surface); (c) the small mechanical geometry leaves; (d) snapshot deletion (−15% lands); (e) validate. Never fire a blind cfr-swap subagent at a realm-consumer — it underflows on synthetic frames.

### EXPLICIT-REALM HUB — DESIGN (session #4)

Target: a `Copy` `CallerContext { realm: RealmRef, lexical_env: EnvironmentRef, code: CodeRef, pc: u32 }` threaded instead of `caller_frame: &FrameRecord` (mirrors `VmProxyBridge`'s existing 4 fields). Realm helpers (vm.rs): `frame_record_realm(agent, &FrameRecord)` = callee `[[Realm]]` else `running_context.realm()` (synthetic-SAFE, reads `frame.callee()`); `realm_of(agent, cfr)` = callee `[[Realm]]` else establishment-scope (needs a real cfr; UNDERFLOWS on synthetic). For a real active frame the two agree.

**Strategy = leaves-up with `frame_record_realm` as the frontier bridge.** Convert each realm-consumer method `caller_frame: &FrameRecord` → `caller_realm: RealmRef`; at the (still-frame-holding) caller pass `Self::frame_record_realm(agent, frame)`. This is synthetic-safe (no `cfr_of`), green per commit, and the bridge concentrates upward until the true entries replace it (real dispatch frame → `realm_of(cfr)`/`current_realm_of`; the ~3 synthetic-construction sites — `internal_calls.rs:424 synthetic_caller_frame`, jobs root — pass their KNOWN realm). At deletion `frame_record_realm` is removed.

**HARD CORE = `VmBuiltinDispatch` (builtin_dispatch/dispatch_context.rs) dual role.** It holds `caller_frame: &FrameRecord` and (i) for PUBLIC builtins + realm fallback reads `frame_record_realm`/`lexical_env`/`code`/`pc` (→ `CallerContext`, synthetic-reachable via `call_to_completion`→`call_builtin`), AND (ii) for INTERNAL class-helper builtins (super_base/private-field/capture-arrow, internal.rs ~20 `FrameView::from_record(self.caller_frame)` sites) needs a REAL cfr-based FrameView. The class-helper dispatch is REAL-frame-only (compiler-emitted, never via `call_to_completion`; today's synthetic path would already underflow `from_record` and Test262 is green, proving it's not hit). Resolution: give `VmBuiltinDispatch` a `CallerContext` for (i); for (ii) the class-helpers can read this/construct_this/private_env from `self.vm.current_cfr` (the active frame == the caller during builtin dispatch) instead of the passed FrameView — a separate follow-on. So convert the realm/proxy/public side first (CallerContext), keep the internal class-helper FrameView path on the real frame until its current_cfr conversion. `EmbeddingFunctionContext` (extensions) also holds `caller_frame` → same CallerContext treatment.

Order within the hub: realm-only LEAVES first (generators resume/delegate, async await/promise_resolve, `builtin_realm`, dynamic_import/object_helpers builtins) → then `call_to_completion`/`construct_to_completion` + `VmIteratorBridge` (CallerContext) → then `call_builtin`/`VmBuiltinDispatch` (CallerContext + class-helper current_cfr) → then the entries (`realm_of(cfr)` for real, known realm for synthetic). Gate each with the async canary + Test262.

**REFINEMENTS (session #4 verification):**
- **Most "leaves" are NOT realm-only — they need the FULL CallerContext** (realm+lexical_env+code+pc) to build a `VmProxyBridge`. Verified: `object_helpers::set_integrity_level`/`test_integrity_level` (object_helpers.rs:119-122/168-171) extract all 4 from `frame`. So introduce `CallerContext` early and thread it; don't chase realm-only leaves piecemeal (the only truly realm-only leaves were the 3 generator-resume methods — DONE, hub leaf 1, commit `<this>`).
- **`VmBuiltinDispatch` class-helpers need the live caller PC, not just cfr.** Verified: `super_ops.rs:113` reads `caller.instruction_offset()` (for feedback at the super-op PC). The overlay `saved_pc` is the PARKED pc (stale mid-op), so a `current_cfr`-only FrameView is WRONG. Resolution: `VmBuiltinDispatch` holds `caller: CallerContext` (incl. `pc` = the live caller pc, == `caller_frame.instruction_offset()` == `dispatch.pc` at build time); class-helper sites (internal.rs ~20 `FrameView::from_record(self.caller_frame)`) build their FrameView as `FrameView::new(self.vm.current_cfr, self.caller.pc, frame_window_len(current_cfr), self.caller.code)` — valid because class-helper dispatch is REAL-frame-only (current_cfr == caller cfr) and never synthetic. Realm/proxy/public sites use `self.caller.realm`/`.lexical_env`/`.code`/`.pc`. A SYNTHETIC `caller_frame` (call_to_completion path) yields a valid CallerContext too — its `lexical_env`/`code`/`pc`/`callee` are real struct fields set at construction (only its window is `(0,0)`); `frame_record_realm` reads `callee`. So `CallerContext` is built synthetic-safe at every construction site.
- **`call_builtin` is the natural CallerContext extraction point** (build once from `caller_frame`, pass to `VmBuiltinDispatch`/`builtin_realm`/import/dynamic/regexp). It keeps `caller_frame` transitionally for the not-yet-converted callees. This is the meaty next chunk — large + coordinated; do with focused context.

### SESSION #5 EXECUTION — THE EXPLICIT-REALM HUB IS LANDED (CallerContext)

The hub is DONE and whole-corpus Test262-validated (49729/0/0/3324, exact baseline). `CallerContext { realm, lexical_env, code, pc }` (Copy, `crate::frame::CallerContext`) + the synthetic-safe frontier bridge `Vm::caller_context_from_record(agent, &FrameRecord)` now replace `caller_frame: &FrameRecord` across the entire builtin-dispatch / call / iterator realm chain.

**Commits (each gated: build + clippy[7 pre-existing only] + vm 609/0 + async canary 35/35):**
1. `172ae7b0` — `CallerContext` + helper + the cycle-FREE realm/proxy leaves (set/test_integrity_level, create_bound_function, collect_array_like_arguments/array_like_arguments_length, create_construct_this). Frame-holding callers bridge via `caller_context_from_record`.
2. `799838ed` — `caller_is_strict(code: CodeRef)` (was a whole FrameRecord, only read `.code()`); `builtin_realm(caller_realm: RealmRef)` (was a frame, only needs the fallback realm). Drops a `frame_record_for_view` at the super-op set site.
3. `a7651d98` — **the cyclic core**, one coordinated commit (the cycle is `call_to_completion → call_builtin → VmBuiltinDispatch → call_to_completion`): `call_to_completion`/`construct_to_completion`/`call_optional_callback`/`call_if_callable_object`/getters+setters (internal_calls), `call_builtin`/`call_frame_safe_builtin` (builtin_dispatch), `VmBuiltinDispatch` + public/internal/support, `instance_of_builtin`/`ordinary_has_instance_with_context` (object_helpers), `template_to_string`/`get_template_object` (template_helpers), import_meta/dynamic_import chain, regexp_literal, `EmbeddingFunctionContext` (extensions). ~25 frontier sites (call/generators/jobs/runtime_objects/property_access/super_ops/private_fields) bridge via `caller_context_from_record` (extract BEFORE the `&mut agent` arg). Class-helper/super-op internal builtins build a live-frame FrameView via `self.caller_frame_view()` = `FrameView::new(current_cfr, caller.pc, frame_window_len(current_cfr), caller.code)`. Getters/setters build CallerContext straight from their 4 fields (synthetic_caller_frame + self.frame() reconstruction GONE).
4. `<pending VmIteratorBridge>` — `VmIteratorBridge` holds `caller: CallerContext` (17 construction sites across runtime_objects/generators/jobs bridge via `caller_context_from_record`). Gated green (build/clippy/vm 609/0/canary 35/35); Test262 re-validating. Unblocks the FrameView swap of the arg/iterator/spread holder methods (`append_iterator_values`/`collect_arguments_into`/`append_spread_argument` etc. — their `frame: &FrameRecord` is now used ONLY to build the bridge's CallerContext, so they can flip to `frame: FrameView` once their callers thread a view).

**KEY LESSON (cost a 153-file Test262 regression, caught + fixed before commit): the caller-frame substitution must preserve COLD fields where a consumer reads them.** `direct_eval_builtin` first used `frame_record_for_view(caller_frame_view())` — but eval parameter-scope analysis (`dynamic_compilation.rs:498`) reads the COLD `parameter_initializer_end_offset`, which `frame_record_for_view` zeroes (hot-only). That regressed 136 `language/eval-code/direct` + 17 `eval-var-scope-syntax-err` files (SyntaxError→ReferenceError). Fix: `direct_eval_builtin` rebuilds the FULL live frame via `self.vm.frame()` (= `reconstruct_frame_from_header`, hot+cold) = the old `*self.caller_frame`. `EmbeddingFunctionContext::force_collect` similarly needs a frame for GC roots — it builds a synthetic frame from CallerContext (`Vm::synthetic_caller_frame(CallerContext)`, repurposed), heap-edge-equivalent (live caller → arena cfr-walk covers; synthetic caller → reproduces lexical_env/code edges; realm not traced off the record). When converting ANY remaining `&FrameRecord` consumer: code-read what it reads — a cold field (`parameter_initializer_end_offset`/`handler_cursor`/`tail_caller`/`resume_*`) means it needs the full `frame()`/snapshot, not a view-derived record.

**Arg-collection trio FrameView-swapped (commit `0c79e054`):** `collect_arguments_into`/`append_spread_argument`/`append_iterator_values` now take `frame: FrameView` (the hub removed their only non-geometry frame use; the iterator realm flows through a `CallerContext` built from the view via `realm_of(cfr)` — real-frame-only, all callers verified non-synthetic). Dropped a `frame_record_for_view` in `construct_super_spread_builtin`. Delegated to a sequential subagent under the safety-stop pattern, driver-verified + Test262-gated. **FINDING after this:** there is NO further large pile of clean blind-delegatable FrameView swaps — the remaining `~60 &/&mut FrameRecord` params are the COUPLED deletion core: the `&mut FrameRecord` dispatch ops in call.rs (`invoke_call_target`/`monomorphic_construct_prototype`/`invoke_tail_call_target`/`try_invoke_cached_builtin_call`/`call_value_small_bytecode_direct`) with their `advance_dispatch_frame`+`sync_dispatch_frame` park/PC sites, the generator suspend/resume cold-field machinery (`resume_kind`/`resume_value`/`handler_cursor`), the async `await_value` park, the runtime_objects iterator-state park methods (`close_iterator_state`/`advance_async_iterator_state`/`close_async_iterator_state`), and the finish_frame/exceptions geometry leaves that are only reached via a reconstructed frame. These are DRIVER work (PC/park correctness — where the prior hangs/OOMs lived), not subagent fan-out. The Refresh arm + thin view ALREADY populate `LlIntState` from the overlay; the snapshot persists ONLY because these dispatch ops take `&mut frame` and the slow-path bodies (e.g. `dispatch_state.rs` `read_constant` `self.frame.code()`, `handle_dispatch_result` PC mirror) still read `self.frame`. Deletion = convert those readers off the snapshot, then drop the field + the eager reconstruct (slow_path.rs:357-358 BISECT) + sync_from_asm reconstruct + finish_frame reconstruct.

**EXACT NEXT MOVE (next session, fresh context) — unchanged from the prior plan, now UNBLOCKED:** the hub removed the realm blocker, so the remaining work is the larger FrameView-swap + frame-switch-park + snapshot-deletion sweep where the −15% lands. Per "EXACT NEXT MOVE" in SESSION #4 EXECUTION above: (1) frame-switch park conversion (call.rs ×9 + generators/async/runtime_objects park sites → overlay `set_saved_pc`); (2) FrameView-swap the now-unblocked ~remaining `frame: &FrameRecord` geometry/realm methods (arg/iterator/spread trio first — VmIteratorBridge no longer threads a frame); (3) flip the property rust-probe + `next_dispatch_instruction_offset` to `dispatch.pc`; (4) DELETE the snapshot (`DispatchState.frame`+`frame_dirty`; `sync_from_asm`→`dispatch.pc = frame_pc_offset`; Refresh arm drop reconstruct; finish_frame overlay-direct; remove reconstruct_frame_from_header/write_snapshot_into_backing/refresh_dispatch_frame/sync_dispatch_frame/advance_dispatch_frame/next_dispatch_instruction_offset/FrameView::from_record) — **−15% lands HERE**; (5) validate Test262 baseline + clean v8 A/B (baseline worktree `1b39a0ec`). NOTE: `frame_record_for_view` and `Vm::frame()` (reconstruct) are STILL needed until snapshot deletion (direct_eval/force_collect/class-helpers use them); they go at step 4.

**DECISIVE FINDING (session #4, proven by a scoped subagent's safety-stop): there is NO clean mechanical FrameView chunk left — the realm chain is the central blocker.** An attempt to swap the "registers-only" arg-collection trio `collect_arguments_into`→`append_spread_argument`→`append_iterator_values` was correctly aborted: `append_iterator_values` (runtime_objects.rs:531) is NOT registers-only — it moves `frame` into a `VmIteratorBridge { frame: &'a FrameRecord, .. }` (runtime_objects.rs:30/545) whose `IteratorOpsContext` impl reads `frame_record_realm`/`lexical_env`/`code`/`instruction_offset` and forwards `frame` to `call_to_completion` (the synthetic-exposed realm chain). So the spread/iterator path threads the FULL frame for realm, exactly like the property path did before the hub. **Conclusion: the explicit-realm hub conversion of `call_to_completion`/`construct_to_completion`/`call_builtin`/`call_frame_safe_builtin`/`builtin_realm`/`VmIteratorBridge`/dynamic-import+object builtins/generator-resume/`await_value` (take explicit `caller_realm`(+lexical_env/code/pc), extract at real-frame entries / known at synthetic-construction sites) is THE foundational next step — it unblocks the arg/iterator/spread FrameView swaps and most of the rest.** It mirrors the property/proxy hub (120b54d9): `VmProxyBridge` already carries `caller_realm/caller_lexical_env/caller_code/caller_pc`; `VmIteratorBridge` must do the same, and `call_to_completion` (internal_calls.rs:55, already extracts those for the proxy path at lines 99–102) drops `caller_frame` for explicit fields. **The scoped-subagent-with-safety-stop loop is VALIDATED** (it prevented a bad edit) — once the hub lands, the mechanical leaf swaps become safe to delegate. Do the hub with fresh, focused context; it is a multi-method careful refactor, not a sweep.

---

### SESSION #6 EXECUTION — field/cold-park redundancy ELIMINATED + CORPUS-PROVEN; deferred Phase-2 reader migration finished

The central correctness hypothesis of the whole snapshot deletion — that the dispatch-frame **field-park and cold-park are redundant** (the overlay/cold tiers are written through at mutation time, so the snapshot's copies always equal the backing at a park) — is now CODE-VERIFIED *and* CORPUS-PROVEN by a live debug tripwire.

**Verified facts (code-read this session — trust these, NOT a classifier):**
- **All call ops return `Refresh` on success** (`semantics/calls.rs`: op_call/op_call_small/op_construct/op_tail_call → `SemanticOutcome::Refresh`; both `Ok(Some)` and `Ok(None)` → Refresh). So EVERY `call.rs` park's PC is read back via the slow_path **Refresh arm** from the overlay `saved_pc`, NEVER the same-frame Continue arm — even the synchronous-builtin path (`invoke_call_target` advances+parks the caller, then the op returns Refresh which re-reads the caller's `saved_pc`). This is why the park content reduces to `set_saved_pc` uniformly across all sites, frame-switch or "same-frame" builtin alike.
- **lexical_env** is overlay write-through: `with_env` push/pop writes BOTH the snapshot and `frame_header_mut(cfr).set_lexical_env` (with_env.rs:34/53).
- **this/this_state/construct_this** are overlay-authoritative: the ONLY mid-frame mutator is `super_ops` (super_ops.rs:254/259/264/296/297) writing `frame_header_mut(cfr).set_this/set_construct_this` DIRECTLY, and it egresses via `Refresh` (reconstructs the snapshot from the overlay). No same-frame snapshot-only `this` write exists, so snapshot==overlay at every park.
- **cold (resume_*/handler_cursor/tail_caller)** is cold-table-authoritative: every mutation writes `frame_cold.get_mut(..)` directly (exceptions.rs:41 handler_cursor; bytecode_calls.rs:114-115 tail_caller; resume clears at dispatch_state.rs:183 / runtime_objects.rs:776/951 / async_functions.rs:156 / generators.rs:1396). The snapshot copy is kept defensively in sync only so the (now-deleted) `write_cold` could not resurrect it; exceptions.rs parks BEFORE setting cold, so no clobber.

**Step A — park primitive simplified (commit `1f03a054`, Test262 baseline 49729/0/0/3324):** `sync_dispatch_frame` now writes ONLY `frame_header_mut(cfr).set_saved_pc(frame.instruction_offset())`. `Vm::write_snapshot_into_backing` and `FrameRecord::write_cold` (the snapshot→backing/cold write halves) are DELETED. A `#[cfg(debug_assertions)]` tripwire in `sync_dispatch_frame` asserts snapshot==overlay (this/this_state/lexical_env/construct_this) and snapshot.cold==cold-table at every park — **fired 0× across the full vm suite (609/0) AND whole-corpus Test262** (debug builds run it on every park), corpus-proving the redundancy. The snapshot is otherwise FULLY intact (still advanced/threaded/reconstructed) — this commit only drops the proven-redundant writes, decoupling the park from the snapshot's mutable fields.

**Step B — deferred Phase-2 semantic-body reader migration (commit `bf440ea7`, Test262 baseline 49729/0/0/3324):** `read_constant` reads `self.code_ref` (thin view) not `self.frame.code()`; the three `op_*_env_slot` bodies (scope.rs:87/118/149) read `inner.vm.frame_header(inner.cfr).lexical_env()` (overlay) not `inner.frame.lexical_env()`. Behavior-identical (thin view/overlay maintained in lockstep). vm 609/0, clippy 7 pre-existing.

**TRIPWIRE ↔ now-dead defensive clears (IMPORTANT for the next session):** the Step-A tripwire asserts snapshot.cold == cold-table, which the defensive snapshot writes — `inner.frame.clear_resume()` (semantics/generators.rs:331), the paired clears at async_functions.rs:159 / runtime_objects.rs:779/954 / generators.rs:1399, and `*frame = (*frame).with_resume(..)` at generators.rs:1496 — KEEP TRUE. After `write_cold`'s deletion those defensive snapshot writes are DEAD (nothing reads the snapshot's cold: resume reads go through `inner.resume_*()` = the cold table; the snapshot is rebuilt from cold at every Refresh). They were LEFT IN PLACE this session because removing them would make the tripwire's cold half fire. **At snapshot deletion (or whenever the tripwire is dropped), remove the defensive clears + the cold half of the tripwire together.**

**EXACT NEXT MOVE (next session, FRESH context — the hang-prone coupled core; −15% lands at its END):** the park-content redundancy is now PROVEN, so the FrameView swaps need only handle GEOMETRY + the PC park + realm-via-CallerContext (hub already landed). The park at every site is now equivalent to `frame_header_mut(view.cfr()).set_saved_pc(view.instruction_offset().wrapping_add(len))` (or no `+len` for the standalone pre-builtin park, which parks the entry PC). Sequence (gate EVERY sub-step: build + clippy[7 pre-existing] + vm 609/0 + `async_and_generators` canary 35/35 [hang detector] + Test262 at cluster boundaries):
  1. **FrameView-swap the remaining ~55 `frame: &FrameRecord`/`&mut FrameRecord` methods** (call.rs 13, generators 10, runtime_objects 6, bytecode_calls 6, async_functions 3, with_env 2, exceptions 2, ic_slow_counters 1, names 1, dispatch/property rust-probe 1). Per method: inline the park as `set_saved_pc` keyed on `view.cfr()`; replace `frame.registers()/code()/instruction_offset()` with the FrameView methods; build CallerContext from the view (`realm_of(view.cfr())` + overlay) for REAL frames, or thread explicit CallerContext for the synthetic-exposed call/construct/builtin chain (hub pattern). `close_iterator_state` / general-builtin `invoke_call_target` keep their `sync`→work→`refresh` bracket: the `refresh_dispatch_frame` (snapshot rebuild after a re-entrant builtin) stays until step 3 (the method's caller still uses the snapshot). EXCLUDE (these go at deletion, step 3): `cfr_of`, `frame_record_realm`, `caller_context_from_record`, `FrameView::from_record`, `trace_all_frame_edges` (GC synthetic), `advance_dispatch_frame`, `next_dispatch_instruction_offset`, `handle_dispatch_result`, `finish_abc_value_result`, `debugger.rs`. **COLD-FIELD GOTCHA** still applies (direct_eval reads cold `parameter_initializer_end_offset` → needs full `frame()`, not a view-derived record).
  2. **Residual snapshot readers:** `handle_dispatch_result` wrapper F3 (`self.pc = self.frame.instruction_offset()`, dispatch_state.rs:289) → on the same-frame caught path set `self.pc = self.vm.frame_header(self.cfr).saved_pc()` (the handler PC `transfer` parked); op_await PC sync (semantics/generators.rs:284); the assign-named rust-probe (cold.rs:2854/2866) → `dispatch.pc`.
  3. **DELETE the snapshot:** `DispatchState.frame` + `frame_dirty`; `sync_from_asm` → just `dispatch.pc = frame_pc_offset` (drop the lazy reconstruct + `frame.set_instruction_offset`); Refresh arm (slow_path.rs:357, `BISECT(superprop)`) → drop the eager reconstruct; `finish_frame` (registers.rs:106) → drop `pop_current_frame`'s reconstruct, read return_register/window from the overlay (it ALREADY reads this/lex/construct from the overlay — just stop reconstructing); `refresh_from_active_frame` → thin-view-from-overlay; remove `reconstruct_frame_from_header` (iff no generator-restore caller remains — CHECK exceptions.rs unwind + generators save/restore), `refresh_dispatch_frame`, `sync_dispatch_frame`, `advance_dispatch_frame`, `next_dispatch_instruction_offset`, `FrameView::from_record`, the defensive snapshot clears (tripwire note above), and the Step-A tripwire. **−15% lands HERE** (the eager reconstruct is gone).
  4. **Validate:** whole-corpus Test262 = baseline + clean v8 A/B (baseline worktree `1b39a0ec`, 7 samples, /tmp/memtest.sh, /tmp reports) showing the call/return cluster (RayTrace, DeltaBlue) recovered to ≥ baseline.
The hang risk concentrates at step 3 (snapshot deletion / Refresh-arm reconstruct removal). Do steps 1–2 as gated per-file clusters; do step 3 in one focused pass with the async canary after each removal.

---

### SESSION #7 EXECUTION — the ENTIRE call/construct/tail dispatch core is FrameView-swapped (57→38 params)

The hardest, most-interconnected, hang-prone surface — `call.rs` (the "NOT a mechanical property.rs repeat" core) + the `bytecode_calls.rs` call leaves — is now fully `FrameView`. `call.rs` no longer references `FrameRecord` at all. Each step gated: build + clippy (7 pre-existing) + vm 609/0 (Step-A tripwire silent) + async canary + whole-corpus Test262 49729/0/0/3324 (baseline).

**Two reusable primitives introduced this session:**
- `Vm::park_caller_pc(&mut self, cfr: u32, pc: u32)` (vm/dispatch.rs) — the FrameView-era park: `frame_header_mut(cfr).set_saved_pc(pc)`. Replaces `advance_dispatch_frame(frame,len)+sync_dispatch_frame(depth,*frame)` (call `park_caller_pc(view.cfr(), view.instruction_offset().wrapping_add(len))`) and the standalone pre-builtin `sync_dispatch_frame` (`park_caller_pc(view.cfr(), view.instruction_offset())`, entry PC). It does NOT run the Step-A tripwire (that lives in `sync_dispatch_frame`); fine, the redundancy is corpus-proven.
- `Vm::caller_context_from_view(&self, agent, view) -> CallerContext` (vm.rs) — the REAL-frame analog of `caller_context_from_record`: realm via `realm_of(view.cfr())`, lexical_env via `frame_header(view.cfr())`. MUST NOT be used on a synthetic frame; all call.rs frames are real (semantic-body-reached only — verified `internal_calls`/jobs never thread these methods, they use `prepare_bytecode_call` with explicit lexical_env).

**KEY VALIDATED PATTERN — dropping `refresh_dispatch_frame` on re-entrant builtin paths.** `invoke_call_target` (general-builtin) and `construct_value` (proxy + builtin) used to `sync`(park)→builtin-re-enters-VM→`refresh_dispatch_frame`(rebuild snapshot)→write-reg→advance→`sync`. After the FrameView swap the mid-method `refresh_dispatch_frame` is DROPPED: the op returns `Refresh` (all call/construct ops do), which reconstructs the snapshot from the restored active caller frame; and the post-re-entry code uses only `view` geometry (the caller cfr is unchanged across the synchronous re-entry, so `view.registers()`/`view.code()` stay valid). construct_value alone was Test262-gated FIRST to validate this before the call core repeated it.

**Commits (8 this session):** `1f03a054` (Step A: park-content redundancy), `bf440ea7` (Step B: Phase-2 readers), `d107dd9a` (bytecode-call register-window fast path: enter_bytecode_call/enter_bytecode_call_from_caller_registers/invoke_bytecode_call+construct_from_caller_arg_window/call_value_small_bytecode_direct), `4a1eb811` (construct_value), `3d168702` (call-dispatch core: call_value/call_value_small/invoke_collected_call_value/invoke_call_target↔invoke_function_call_builtin_target/try_invoke_cached_builtin_call), `ad818ea5` (tail-call cluster: tail_call_value/invoke_tail_call_target/recycle_tail_bytecode_call/teardown_tail_frame + `release_frame_to_caller(popped_cfr: u32)` — all 9 release callers pass `cfr_of(real popped)`), `8ecb6f76` (finalize_frame_result→`flags: FrameFlags`, record_ic_slow_entry→FrameView). `frame_depth` dropped from every swapped call/construct/tail method.

**REMAINING ~38 `&FrameRecord`/`&mut FrameRecord` params (next session, fresh context):**
- **EXCLUDE / deletion-endgame (≈12, do NOT swap — they go at deletion):** `cfr_of`, `frame_record_realm`, `caller_context_from_record` (vm.rs bridge); `FrameView::from_record` (frame.rs bridge); `trace_all_frame_edges` (state.rs GC synthetic); `advance_dispatch_frame`/`next_dispatch_instruction_offset`/`handle_dispatch_result`/`finish_abc_value_result` (dispatch.rs PC/exception helpers — the PC-carrier surface); the assign-named rust-probe (dispatch/property.rs + cold.rs); `debugger.rs`; `names::load_name` (#[cfg(test)] helper, builds its OWN record, not the snapshot — harmless).
- **SWAPPABLE clusters left (~20):**
  - **runtime_objects iterators (6):** `advance_iterator_state`/`close_iterator_state`/`advance_async_iterator_state`/`close_async_iterator_state`/`async_from_sync_iterator_continuation`/`+1`. Geometry + `caller_context_from_view` + the sync→`iterator_close`(re-entry)→refresh bracket (drop refresh like the call core — VERIFY the op returns Refresh or the caller doesn't read the snapshot after). `close_async`/`advance_async` contain the defensive `frame.clear_resume()` (see entanglement below).
  - **async_functions (3):** `promise_resolve_in_realm` (check frame use), `suspend_for_await_promise`+`await_value` (suspend machinery → `snapshot_suspended_execution`; await_value has the defensive clear).
  - **generators (10):** the trickiest — resume/suspend frame-switch, cold `handler_cursor`/`resume_*` via `frame_cold.get(depth)`, the frame-encode (`with_this_value`/`with_construct_this`/`with_resume` BUILD a FrameRecord for push/restore — those STAY FrameRecord). Do with fresh context + async canary after each.
  - **exceptions (2):** `select_exception_handler`/`suspended_call_instruction_offset` — fed a transient `reconstruct_frame_from_header` and read `saved_pc`-as-PC; swapping them to take `cfr` would also delete that per-exception reconstruct (delicate transfer path).
  - **with_env (2):** `push_with_environment`/`pop_with_environment` — WRITE-SIDE entangled: they write BOTH the snapshot `frame.set_lexical_env` AND the overlay. Dropping the snapshot write (forced by FrameView) leaves snapshot.lexical_env stale → fires the Step-A tripwire AND breaks any remaining `frame.lexical_env()` snapshot reader. Swap ONLY after all snapshot-`lexical_env` readers are gone (then drop the snapshot write + the param entirely — operate on the active overlay via `current_cfr`).
  - **bytecode_calls static (2):** `resolve_this_binding`/`resolve_super_home_object` — `static` fns (no `&self`) reading `caller_frame.this_value()`; take an explicit `this_value: Value` (the caller reads `frame_header(cfr).this_value()`), or convert to `&self` + cfr.

**TRIPWIRE ↔ resume-clear ENTANGLEMENT (carry forward from SESSION #6, now the gating issue for the resume methods):** the Step-A tripwire (`sync_dispatch_frame`) asserts snapshot.cold == cold-table. The defensive snapshot `frame.clear_resume()` / `*frame = (*frame).with_resume(..)` (async_functions await_value, runtime_objects close/advance_async, generators resume) KEEP that true. FrameView-swapping those methods FORCES dropping the defensive snapshot writes (FrameView is Copy, no `clear_resume`) → snapshot.cold diverges → the cold half of the tripwire fires. So WHEN swapping the first resume-clearing method: **remove the COLD half of the Step-A tripwire** (the `frame.resume_*()/handler_cursor()/tail_caller*()` block in `sync_dispatch_frame`), keeping the overlay half (this/this_state/lexical_env/construct_this). The cold half asserts an invariant maintained only by the now-removable defensive clears; the overlay half is the real write-through-gap detector and stays until `with_env` is swapped. (Resume reads already go through `inner.resume_*()` = the cold table, so the snapshot's stale cold is functionally dead.)

**EXACT NEXT MOVE:** continue the FrameView swaps on the remaining ~20 (runtime_objects iterators → async → exceptions → generators[careful] → with_env[last, write-side] → the 2 static bytecode_calls), per the established primitives (`park_caller_pc`, `caller_context_from_view`, dropped-refresh-when-op-Refreshes, cold-field-gotcha for any cold reader). Then the **deletion endgame** (the SESSION #6 step 2–3): residual readers (F3 `self.frame.instruction_offset()`→overlay `saved_pc` on same-frame caught; op_await PC sync; rust-probe→dispatch.pc) → DELETE `DispatchState.frame`+`frame_dirty`, drop the eager reconstruct (slow_path Refresh arm `BISECT(superprop)` + sync_from_asm + finish_frame), remove `reconstruct_frame_from_header`/`refresh_dispatch_frame`/`sync_dispatch_frame`/`advance_dispatch_frame`/`next_dispatch_instruction_offset`/`FrameView::from_record`/the defensive clears/the Step-A tripwire — **−15% lands HERE** — then v8 A/B (baseline worktree `1b39a0ec`).

---

### SESSION #8 EXECUTION — the runtime_objects iterator + async clusters off the snapshot, tripwire cold-half removed, op_await reader converted (37 `&FrameRecord` refs, was 50)

Seven commits, each gated (build + clippy 7-pre-existing + vm 609/0 [Step-A overlay tripwire silent] + whole-corpus Test262 49729/0/0/3324, variants 95205/0/0 = baseline). `runtime_objects.rs` and `async_functions.rs` are both `FrameRecord`-free (outside test): all 6 iterator methods + the 2 async leaves + the suspend primitive + `await_value` are migrated; the Step-A tripwire's cold half is gone (the "first resume-clearer" milestone); `op_await`'s snapshot-PC reader is converted to the overlay (a deletion-endgame step landed early); and the 2 generator suspend wrappers are migrated. What remains is the `yield*` delegate generator core (7 interconnected methods), exceptions, with_env, the 2 statics — then the deletion endgame.

**Commits (7 this session):**
- `cbc8ccb2` — `promise_resolve_in_realm` → `CallerContext` (NOT FrameView). KEY FINDING: it has TWO synthetic-frame callers (`generators.rs` async-generator return-completion jobs pass `synthetic_job_caller_frame(&realm)`), so `realm_of(cfr)`/`frame_header(cfr)` would underflow. Its entire frame-derived surface is exactly `realm`/`lexical_env`/`code`/`pc` (the four `get_property_from_value` args) = a `CallerContext`. All 10 call sites build it via `caller_context_from_record` (synthetic-safe); the 8 real ones agree field-for-field, the 2 synthetic keep their running-context fallback realm.
- `a4fad358` — `async_from_sync_iterator_continuation` + `close_iterator_state_preserving_completion` → `CallerContext`. Both pure-overlay (no geometry/cold/park/suspend); `frame_realm` = `caller.realm`. Real-frame-only callers (advance_async/close_async reuse their existing `caller`; close_iterator_state builds it on the preserve-completion branch).
- `a545453e` — the SUSPEND PRIMITIVE: `snapshot_suspended_execution` + `suspend_for_await_promise` → `FrameView`. KEY: `snapshot_suspended_execution` now reads the cold `handler_cursor` from the depth-keyed cold table (`frame_cold.get(frame_depth - 1).handler_cursor`) instead of the snapshot copy — every one of its 3 callers asserts `current_cfr == frame.cfr()`, so the suspending frame's cold slot is `frame_depth - 1`, and the tripwire already proves that slot == the snapshot copy. Realm via `realm_of(frame.cfr())` (suspending frame is always a real callee-bearing async/generator frame). The 3 snapshot callers + 7 suspend callers (still `&FrameRecord`) bridge via `FrameView::from_record`; those bridges drop out as their methods migrate.
- `f4e1994b` — the RESUME-CLEARER milestone: `advance_async_iterator_state` + `close_async_iterator_state` → `FrameView`, and the Step-A tripwire's COLD half REMOVED (`vm/dispatch.rs`). Dropped each `frame.clear_resume()` (cold table is authoritative + cleared in place); `sync_dispatch_frame(_, *frame)`→`park_caller_pc(frame.cfr(), frame.instruction_offset())`; `caller_context_from_record`→`caller_context_from_view`; `frame_record_realm`→`realm_of(cfr)`; close_async's `get_property_from_value` reads off a `CallerContext`; suspend `from_record` bridges dropped. The overlay half of the tripwire stays (this/this_state/lexical_env/construct_this) until `with_env` migrates. Verified safe: ops egress `Continue` (not Refresh), no `refresh_dispatch_frame`, no op-level snapshot-PC read. advance_async + tripwire-cold-half was Test262-baseline-validated *in isolation* before close_async was added.
- `b0c48a26` — the sync entries `advance_iterator_state` + `close_iterator_state` → `FrameView`, COMPLETING the runtime_objects iterator cluster. The op bodies use `let view = inner.frame_view();` + drop `frame` from the destructure. `close_iterator_state`'s post-`iterator_close` `refresh_dispatch_frame` DROPPED (balanced nested call never mutates this frame's overlay/cold + op egresses Continue ⇒ reconstruct would be a no-op). Test module gains `use crate::frame::FrameRecord` (`cargo build` masks test-only usages — the LSP caught it; `cargo test` is the real gate).
- `ad699f6b` — `await_value` → `FrameView` + `op_await`'s PC reader routed through the overlay (the deletion-endgame "op_await PC sync", done early because coupled). await_value's 3 snapshot-PC ops became overlay parks: resume-throw parks the await PC then `transfer_to_exception_handler` overwrites `saved_pc` with the handler PC and the in-method `refresh_dispatch_frame` is DROPPED (caught resume-throw is always same-frame); resume-next parks `await_pc + len`; suspend parks the await PC + drops the suspend bridge. `op_await` reads `inner.vm.frame_header(inner.cfr).saved_pc()` on Ok (the F3 pattern). async_functions.rs is now FrameRecord-free; dropped its `FrameRecord` + `advance_dispatch_frame` imports.
- `d4215b60` — the generator suspend wrappers `suspend_current_generator_frame` / `suspend_generator_start` → `FrameView` (structurally identical to `suspend_for_await_promise`): snapshot bridge dropped, `cfr_of`→`frame.cfr()`. The op bodies (op_yield / op_suspend_generator_start) use the `frame_view()` pattern (resume-offset still via the maintained snapshot's `next_dispatch_instruction_offset` — unchanged); `finish_delegate_yield_outcome`'s internal suspend site bridges via `FrameView::from_record`.

**REMAINING SWAPPABLE — precise analysis for next session (do with FRESH context per the hazard rules):**
- **generators `yield*` delegate core (7, THE NEXT MOVE — the trickiest; DRIVE with fresh context + async/generator canary after EACH):** `delegate_yield`, `finish_delegate_return_resume`, `finish_delegate_yield_outcome` (its internal suspend site already bridges via `FrameView::from_record`), `start_async_delegate_next`, `start_async_delegate_iterator_result_await`, `start_async_delegate_value_await` (these last two call `promise_resolve_in_realm`[CallerContext, done] + sync + `suspend_for_await_promise`[FrameView, done]), `resume_async_delegate_yield`. Interconnected resume/suspend frame-switch + the delegate-await state machine; code-read EACH for cold reads (`frame_cold.get(depth)` resume_*/handler_cursor — need the cold table, NOT a view-derived record) and for the frame-ENCODE (`with_this_value`/`with_construct_this`/`with_resume` BUILD a FrameRecord for push/restore — those STAY `FrameRecord`). [Already done this session: `await_value`→FrameView + `op_await` overlay-PC reader; the 2 suspend wrappers→FrameView.]
- **exceptions (2):** `select_exception_handler`/`suspended_call_instruction_offset` — fed a transient `reconstruct_frame_from_header`, read `saved_pc`-as-PC; delicate transfer path (deletion-endgame-adjacent).
- **with_env (2):** `push_with_environment`/`pop_with_environment` — WRITE-SIDE; swap LAST (they keep the overlay-half tripwire honest). Drop the snapshot `set_lexical_env` write + the param only after all snapshot-`lexical_env` readers are gone.
- **bytecode_calls static (2):** `resolve_this_binding` (reads `caller_frame.this_value()` → take `caller_this_value: Value`) / `resolve_super_home_object` (reads `caller_frame.callee()` → take `caller_callee: Option<ObjectRef>`). NOTE: callers live in `builtin_dispatch/class_helpers` (super_ops.rs:69/99/137, class_helpers.rs:148) + semantics/names.rs:609 and use their own frame abstractions (`self.frame_record(...)`, `caller_record`, the F-handler `frame`) — plumb the explicit value from each caller's available frame surface.

**EXACT NEXT MOVE:** the `yield*` delegate generator core — the 7 interconnected methods (`delegate_yield` / `finish_delegate_return_resume` / `finish_delegate_yield_outcome` / `start_async_delegate_next` / `start_async_delegate_iterator_result_await` / `start_async_delegate_value_await` / `resume_async_delegate_yield`), careful + canary after each, the `with_*`/`with_resume` ENCODE stays `FrameRecord`. Then exceptions (2), with_env (2, LAST — write-side, then drop the overlay-half tripwire too), the 2 static bytecode_calls. Then the **deletion endgame** (residual readers F3 `self.frame.instruction_offset()`→overlay `saved_pc` + the assign-named rust-probe→dispatch.pc; DELETE `DispatchState.frame`+`frame_dirty`; drop the eager reconstruct in slow_path Refresh / sync_from_asm / finish_frame; remove `reconstruct_frame_from_header`/`refresh_dispatch_frame`/`sync_dispatch_frame`/`advance_dispatch_frame`/`next_dispatch_instruction_offset`/`FrameView::from_record`/the remaining defensive clears/the Step-A tripwire) — **−15% lands HERE** — then v8 A/B (baseline worktree `1b39a0ec`).

---

### SESSION #9 EXECUTION — the `yield*` delegate core is FrameView-swapped (generators.rs FrameRecord-free) + the 2 static resolvers done (37→28 refs)

The trickiest surface — the 7-method `yield*` delegate generator core — is OFF the snapshot. `generators.rs` no longer references `FrameRecord` in any of the 7 methods (FrameRecord-free outside its `#[cfg(test)]` module and the generator suspend/restore ENCODE `FrameRecord::new(...).with_*()`/`with_resume`, which stays). Then the 2 static super/this resolvers landed as a clean bonus. Each commit gated (build + clippy 7-pre-existing + vm 609/0 [Step-A overlay tripwire silent] + whole-corpus Test262 49729/0/0/3324, variants 95205/0/0/6648 = baseline).

**Commits (4 this session):**
- `e70170e0` — the 3 suspend-only async starts `start_async_delegate_next` / `start_async_delegate_iterator_result_await` / `start_async_delegate_value_await` → `FrameView` (drop `frame_depth`). Each egresses via `suspend_for_await_promise` (`Err(AsyncSuspend)`) so — like the runtime_objects iterator cluster — none touches the snapshot-advance/F3 PC mirror: mechanical swap (`frame_record_realm`→`realm_of(cfr)`, `caller_context_from_record`→`caller_context_from_view`, `sync_dispatch_frame`→`park_caller_pc`, `suspend_for_await_promise(_, FrameView::from_record(frame), _)`→`(_, frame, _)`). Real-frame-safe (the frame is the active callee — `suspend_for_await_promise` asserts `current_cfr == frame.cfr()`).
- `35e2142a` — `finish_delegate_yield_outcome` → `FrameView` (drop `frame_depth`) + `op_delegate_yield`'s PC reader routed through the overlay. THE COUPLED UNIT (mirrors the await_value/op_await pair, a deletion-endgame F3 step that lands here because it can't be separated): the Complete branch's `advance_dispatch_frame(frame, len)` (snapshot bump) → `park_caller_pc(cfr, pc + len)` (overlay park, FrameView is Copy → no snapshot to advance); `op_delegate_yield_semantic` now mirrors `frame_header(cfr).saved_pc()` into `inner.pc` on `Ok` (op_await pattern). Verified: Complete parks `delegate-PC + len`; a caught throw routes through `handle_dispatch_result` whose `transfer_to_exception_handler` overwrites `saved_pc` with the handler PC + `refresh_dispatch_frame` rebuilds the snapshot from it, so the overlay holds the right PC for both `Ok` arms; `yield*`'s caught throw is ALWAYS same-frame (inner iterator's nested calls are balanced → the generator frame is active when delegate_yield returns), so `cfr` is valid/unchanged and `translate_outcome`'s Continue arm sources PC from the thin view. Also dropped the now-unused `advance_dispatch_frame` import + the now-dead `frame_depth` param on `finish_delegate_return_resume`.
- `8659b6cb` — the 3 `&mut FrameRecord` methods `delegate_yield` / `finish_delegate_return_resume` / `resume_async_delegate_yield` → `FrameView`, COMPLETING the cluster. `delegate_yield`/`resume_async_delegate_yield` KEEP `frame_depth` (the cold table is depth-keyed: `resume_kind`/`resume_value`/`resume_active` reads stay `frame_cold.get(frame_depth - 1)`; FrameView carries cfr/pc/code/regs_len, not depth). `caller_context_from_record`→`caller_context_from_view` (and in-bridge `Vm::…(bridge.agent, frame)`→`bridge.vm.caller_context_from_view(bridge.agent, frame)`); the prior commits' `FrameView::from_record` bridges collapse to passing `frame`. Dropped the two defensive snapshot writes the resume path no longer needs (FrameView Copy): `frame.clear_resume()` (cold cleared in place one line up) + `*frame = (*frame).with_resume(Return, v)` (the cold write beside it is the live store `LoadResume*` observes; the Step-A cold-half tripwire that policed snapshot↔cold parity was removed SESSION #8). `op_delegate_yield` grabs `let view = inner.frame_view();` before the destructure and passes the view. The synthetic-frame `caller_context_from_record(agent, &caller_frame)` calls (async-generator return-completion jobs) STAY `FrameRecord`.
- `66f725c6` — the 2 static `bytecode_calls` resolvers `resolve_this_binding` (→ `caller_this_value: Value`) / `resolve_super_home_object` (→ `caller_callee: Option<ObjectRef>`). Each read a single caller-frame field; take it explicitly. Callers plumb it: `op_load_this` passes `frame.this_value()`; `super_property_get/set_builtin`/`super_base_builtin` (super_ops.rs) + `capture_arrow_context_builtin` (class_helpers.rs) read `frame_header(caller.cfr()).callee()` off the overlay — DROPPING the 4 throwaway `frame_record_for_view(caller)` materializations these super paths built solely to feed the old `&FrameRecord` param.

**REMAINING (28 refs): 4 swappable + ~24 EXCLUDE/deletion-endgame.**
- **exceptions (2, ENDGAME-COUPLED — do WITH the reconstruct elimination):** `select_exception_handler` / `suspended_call_instruction_offset` (exceptions.rs:59/86) read only `frame.code()`+`frame.instruction_offset()`. Their SOLE caller `transfer_to_exception_handler` (exceptions.rs:20) builds a transient `reconstruct_frame_from_header(cfr, depth-1)` PURELY to feed them — the `instruction_offset` must be the parked `saved_pc` (the handler-search PC). Swap to `FrameView` AND build `FrameView::new(cfr, frame_header(cfr).saved_pc(), <window-len>, frame_header(cfr).code())` directly, deleting that transient reconstruct (a deletion-surface item — `reconstruct_frame_from_header` is on the EXCLUDE list). regs_len: these two never read `registers()`, but pass the real window len (a `window_len_for(code)`/`frame_window_len(cfr)` helper) not 0, to stay a footgun-free real view. Best done in the deletion-endgame session, not piecemeal.
- **with_env (2, LAST — write-side):** `push_with_environment` / `pop_with_environment` (with_env.rs:7/39) write BOTH the snapshot `frame.set_lexical_env` AND the overlay. Swap to operate on the active overlay only (`current_cfr`) and drop the snapshot write + param ONLY after all snapshot-`lexical_env` readers are gone — THEN ALSO drop the overlay-half Step-A tripwire (this/this_state/lexical_env/construct_this in `sync_dispatch_frame`), which depends on `with_env` keeping `snapshot.lexical_env` in sync. NOTE: `op_load_this` (semantics/names.rs) still destructures the snapshot `frame` for `this_state()`/`this_value()`/`lexical_env()` — those are snapshot READERS that the deletion endgame (or a Phase-2 reader migration to `inner.this_state()`/`inner.lexical_env()` overlay accessors) must clear before `with_env` is safe.
- **EXCLUDE / deletion-endgame surface (≈24, go at deletion or stay):** `cfr_of` / `frame_record_realm` / `caller_context_from_record` / `refresh_running_context_to_caller(popped)` / `write_header_from_record`(vm.rs:1654) (vm.rs bridges); `FrameView::from_record` (frame.rs); `trace_all_frame_edges` (state.rs GC synthetic); `advance_dispatch_frame` / `next_dispatch_instruction_offset` / `refresh_dispatch_frame` / `handle_dispatch_result` / `finish_abc_value_result` / `reconstruct_frame_from_header` (dispatch.rs PC/exception helpers — the deletion surface); the rust-probe (dispatch/property.rs:237); `debugger.rs:59`; `names::load_name` (#[cfg(test)] helper, builds its own record via `FrameView::from_record`).

**EXACT NEXT MOVE (fresh context):** the **deletion endgame** — the −15% lands HERE. Do the exceptions (2) swap AS PART OF it (it deletes the transient reconstruct). Residual readers: F3 `dispatch_state.rs:292 self.pc = self.frame.instruction_offset()` (the `handle_dispatch_result` wrapper) → overlay `frame_header(cfr).saved_pc()` on same-frame; the assign-named rust-probe (dispatch/property.rs + dsl/handlers/cold.rs) → `dispatch.pc`. Then swap `with_env` (write-side) + drop the overlay-half Step-A tripwire. Then DELETE `DispatchState.frame` + `frame_dirty`; rewrite `sync_from_asm` (just `dispatch.pc = frame_pc_offset`), the slow_path `Refresh` arm (populate LlIntState from the overlay, no reconstruct — Task 10), and `finish_frame` (overlay-direct return-reg/window, no reconstruct — Task 11); remove `reconstruct_frame_from_header` / `refresh_dispatch_frame` / `sync_dispatch_frame` / `advance_dispatch_frame` / `next_dispatch_instruction_offset` / `FrameView::from_record` / the remaining defensive `clear_resume()`s / the Step-A tripwire. Compile-error-driven. Then full Test262 = baseline + clean v8 A/B (baseline worktree `1b39a0ec`, 7 samples each, confirm RayTrace/DeltaBlue recovered ≥ baseline).

---

### SESSION #10 EXECUTION — endgame prerequisites (rust-probe + Refresh-defer + handler reads) + the KEY A/B finding: perf is all-or-nothing at field deletion (28→still-28 swappable refs, but the snapshot reader surface is now nearly closed)

Did the three SAFE, gated endgame prerequisites and — critically — **MEASURED a 7-sample v8 A/B that overturns a working assumption**: deferring the Refresh-arm reconstruct does NOT recover the −15% on its own. Each commit gated (build + clippy 7-pre-existing + vm 609/0 + whole-corpus Test262 49729/0/0/3324, variants 95205/0/0).

**Commits (3 this session):**
- `07e52cfb` — the assign-named rust-probe (`try_assign_named_property_rust_probe_for_dsl` + `op_assign_named_property_rust_probe_rs`) → reads the **thin view**, not the snapshot. This was the SOLE slow-path entry bypassing `sync_from_asm` (it `call_rust_probe!`s + manually synced only the PC via `dispatch.frame.set_instruction_offset`), which is exactly why the Refresh arm had to keep the eager reconstruct (`BISECT(superprop)` = a `SetNamedProperty` after a `super` access read the stale snapshot). Now: `dispatch.pc = entry_pc`, pass `dispatch.frame_view()`, on hit `next_pc = entry_pc + 6`. Dropped property.rs's `advance_dispatch_frame`/`FrameRecord` imports.
- `cfd2705b` — the `translate_outcome` **Refresh arm** flips the eager `reconstruct_frame_from_header` to `frame_dirty = true` (lazy rebuild via `sync_from_asm`). Behavior-preserving; superprop suite passes (proving E1 was the missing prerequisite the `BISECT` marker needed).
- `1982e5ad` — the remaining `dsl/handlers/` direct snapshot reads (InvalidJumpTarget/DoublePrefix guards + `feedback_slot_from_pc`/`call_range_from_pc`/wide-prefix decoders) → thin view (`inner.code()`/`inner.pc()`). `dsl/handlers/` now holds ZERO direct `DispatchState.frame` reads.

**🔑 THE A/B FINDING (measured, 7 samples, baseline `1b39a0ec` vs HEAD-after-Refresh-defer) — READ BEFORE THE DELETION:**
| Bench | base | branch | Δ |  | Bench | base | branch | Δ |
|---|---|---|---|---|---|---|---|---|
| Richards | 508 | 439 | −13.6% |  | RayTrace | 459 | 379 | −17.4% |
| DeltaBlue | 386 | 329 | −14.8% |  | NavierStokes | 545 | 460 | −15.6% |
| Crypto | 432 | 387 | −10.4% |  | Splay | 1479 | 1324 | −10.5% |

The branch STILL carries the full ~15% SP-0b regression. **Deferring the Refresh reconstruct moved the cost, it did not remove it**: for these call-heavy benchmarks the call/return ops route through slow stubs that call `sync_from_asm`, which now does the (deferred) reconstruct instead. The −15% recovers ONLY when `DispatchState.frame` is DELETED so `sync_from_asm` has NOTHING to rebuild (`sync_from_asm` becomes just `dispatch.pc = frame_pc_offset`) AND `finish_frame` stops reconstructing the popped frame (`pop_current_frame`). This is the handoff's "all-or-nothing at deletion", now empirically confirmed — DO NOT expect partial perf wins from reader migrations; the entire deletion must land in one validated push.

**De-risking facts established this session:**
- The **Alpha dispatch variant is TEST-ONLY** (`LlIntDispatchState::from_alpha` is called only in `dsl/test_helpers.rs:345`). Production runs exclusively the Asm path. So the deletion's α impact is bounded to test/reference code — the `dsl/handlers/` reads are the *shared* handler fns reached post-`sync_from_asm` on the Asm path.
- `finish_frame` (registers.rs:106) reconstructs the POPPED frame every return via `pop_current_frame` — a SECOND per-return reconstruct independent of the Refresh arm. Both must go (Task 11: read `return_register`/caller window off `frame_header(current_cfr)`/the overlay before `release_frame_to_caller`).

**REMAINING SNAPSHOT READERS (the deletion surface — nearly closed):** the direct `DispatchState.frame` reads left are all bridge/endgame-coupled: `dispatch_state.rs:227` (`sync_active_frame` reads `self.frame` → `sync_dispatch_frame`), `:292` (F3 PC-mirror — convert to `if same-frame && handled.is_none() { self.pc = frame_header(cfr).saved_pc() }`, equivalent because on success `self.pc` already == entry PC), `:310-311` (`refresh_from_active_frame`, α-path reconstruct+write); `semantics/scope.rs:315` (`pop_with_environment(&mut inner.frame)`); `semantics/generators.rs:144/175` (`next_dispatch_instruction_offset(&inner.frame, len)` → `inner.pc().wrapping_add(len)`) + `:340` (`inner.frame.clear_resume()` — DEAD defensive clear, just delete: cold table is authoritative + `inner.clear_resume()` runs one line above) + the `op_yield`/`op_suspend_generator_start` `sync_active_frame()` calls. Param-takers still on the snapshot: exceptions (`select_exception_handler`/`suspended_call_instruction_offset`, fed the transient `reconstruct_frame_from_header` in `transfer_to_exception_handler`:20 — swap to `FrameView::new(cfr, frame_header(cfr).saved_pc(), window_len, frame_header(cfr).code())` and delete the transient), with_env (`push`/`pop_with_environment`, write-side — drop snapshot `set_lexical_env` + param + the overlay-half Step-A tripwire), and the bridge helpers themselves (`reconstruct_frame_from_header`/`sync_dispatch_frame`/`refresh_dispatch_frame`/`advance_dispatch_frame`/`next_dispatch_instruction_offset`/`handle_dispatch_result`/`finish_abc_value_result`/`cfr_of`/`frame_record_realm`/`caller_context_from_record`/`refresh_running_context_to_caller`/`FrameView::from_record`/`write_header_from_record`/`trace_all_frame_edges`-GC/`debugger.rs`/`names::load_name`-test).

**EXACT NEXT MOVE (fresh, focused context — this is the −15% and it is ATOMIC):** the full deletion in ONE validated push: (1) migrate the last direct readers (`generators.rs` next_dispatch+clear_resume; F3 PC-mirror conditional; `sync_active_frame`→`park_caller_pc`); (2) swap exceptions (delete the transient reconstruct) + with_env (drop the snapshot write + overlay-half tripwire); (3) rewrite `finish_frame` overlay-direct (Task 11) + `sync_from_asm` (`dispatch.pc = frame_pc_offset`, no reconstruct) + the Refresh arm (no field); (4) DELETE `DispatchState.frame` + `frame_dirty`, compile-error-drive the removal of `reconstruct_frame_from_header`/`refresh_dispatch_frame`/`sync_dispatch_frame`/`advance_dispatch_frame`/`next_dispatch_instruction_offset`/`FrameView::from_record`/`refresh_from_active_frame`'s reconstruct/the Step-A tripwire. Validate: full Test262 = baseline + re-run the SAME 7-sample A/B (binaries: `/tmp/lyng-base/target/release/lyng` already built at `1b39a0ec`; `target/release/lyng-bench` already built) and confirm Richards/RayTrace/DeltaBlue recover to ≥ baseline (508/459/386).

---

## Phase 3 (ORIGINAL — superseded by the revision above) — Bridge rewrite: drop `reconstruct_frame_from_header` from the hot path (green, PERF)

Make the frame-switch bridge maintain the thin view + populate `LlIntState` directly from the overlay, with no `FrameRecord` materialized on call/return.

### Task 9: Drop the `&mut FrameRecord` params from dispatch/with_env helpers

**Files:** `crates/vm/src/vm/dispatch.rs`, `crates/vm/src/vm/with_env.rs`, callers.

- [ ] **Step 1:** Change `advance_dispatch_frame` / `next_dispatch_instruction_offset` to operate on the thin-view PC. Simplest: replace their use in `finish_abc_value_result` with `dispatch.pc = dispatch.pc + instruction_len` semantics. Convert `handle_dispatch_result` and `finish_abc_value_result` to take `&mut DispatchState` (or the explicit `cfr`/`pc`/`depth`) instead of `frame: &mut FrameRecord`:
  - In `handle_dispatch_result`'s throw arm: replace `self.sync_dispatch_frame(frame_depth, *frame)` with `self.park_pc_into_overlay(cfr, pc)` (new tiny helper: `self.frame_header_mut(cfr).set_saved_pc(pc)`), then after `transfer_to_exception_handler` replace `self.refresh_dispatch_frame(...)` with refreshing the thin view from the overlay (`pc = frame_header(current_cfr).saved_pc()`, `cfr/code_ref/regs_len/depth` from vm).
  - `finish_abc_value_result`: `record_feedback_slot(self.frame_header(cfr).code(), ...)`; `let target = absolute_register(window_from(cfr, regs_len), target_register)`; write result; `pc += instruction_len`.
- [ ] **Step 2:** Change `push_with_environment`/`pop_with_environment` to drop the `frame: &mut FrameRecord` param (they already write the overlay); update the two callers in `vm/semantics/scope.rs` / wherever invoked.
- [ ] **Step 3:** Update `DispatchState::handle_dispatch_result` wrapper (`dispatch_state.rs:171`) to pass the thin view rather than `frame`.
- [ ] **Step 4: Build + vm tests.** `/tmp/memtest.sh 'cargo build --workspace --all-targets --all-features && cargo test -p lyng-vm --all-features 2>&1 | tail -20'`. Green.
- [ ] **Step 5: Commit.**
```bash
git add -A && git commit -m "refactor(vm): thread the thin frame view through dispatch/with_env helpers

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

### Task 10: Rewrite the slow-path `Refresh` arm to populate `LlIntState` from the overlay (no reconstruct)

**Files:** `crates/vm/src/dsl/slow_path.rs`, `crates/vm/src/dsl/llint_state.rs`

- [ ] **Step 1:** Add an overlay-based `this` resolver in `llint_state.rs`:
```rust
pub(crate) fn resolve_initial_this_value_from_header(h: &crate::frame_header::FrameHeader) -> Value {
    resolve_this_state_to_mirror(Some(h.this_state()), h.this_value())
}
```
- [ ] **Step 2:** In `translate_outcome`'s **Refresh** arm, replace the `reconstruct_frame_from_header` block with direct overlay reads:
```rust
                    let current_depth = rust.dispatch.vm.frame_depth();
                    let Some(cfr) = rust.dispatch.vm.current_cfr_opt() else { /* program exit: leave mirrors */ };
                    let code = rust.dispatch.vm.frame_header(cfr).code();
                    let installed = rust.dispatch.vm
                        .installed_for_dsl_runtime(code)
                        .unwrap_or_else(|| rust.dispatch.installed.clone());
                    let regs_len = rust.dispatch.vm.frame_window_len(cfr);
                    let saved_pc = rust.dispatch.vm.frame_header(cfr).saved_pc();
                    // update thin view
                    rust.dispatch.cfr = cfr;
                    rust.dispatch.pc = saved_pc;
                    rust.dispatch.code_ref = code;
                    rust.dispatch.regs_len = regs_len;
                    rust.dispatch.frame_depth = current_depth;
                    rust.dispatch.installed = installed;
                    rust.dispatch.frame_check_epoch = rust.dispatch.vm.dispatch_frame_check_epoch_for_dsl();
                    // populate LlIntState mirrors (regs_base, pb_base, mt_base, const_base, this) from overlay+installed
                    let regs_base_ptr = unsafe {
                        rust.dispatch.vm.register_stack_storage_mut_ptr()
                            .add((cfr + crate::frame_header::HEADER_SLOTS as u32) as usize)
                    };
                    let this_value = crate::dsl::llint_state::resolve_initial_this_value_from_header(
                        rust.dispatch.vm.frame_header(cfr),
                    );
                    // ...pb_base/mt_base/object tables/const_base exactly as today, using `code`...
                    unsafe {
                        (**state).frame_pc_offset = saved_pc;
                        (**state).frame_pb_base = pb_base;
                        (**state).frame_regs_base = regs_base_ptr;
                        (**state).frame_metadata_table_base = mt_base;
                        (**state).object_records_base = object_records_base;
                        (**state).object_slots_base = object_slots_base;
                        (**state).value_cells_base = value_cells_base;
                        (**state).frame_const_base = const_base;
                        (**state).frame_this_value = this_value;
                    }
```
Keep the `#[cfg(debug_assertions)]` const-base stability assertion (recompute via `code`). No `FrameRecord` is built.
- [ ] **Step 2b:** In the **Continue** arm, replace `active_frame.registers().base()` / `active_frame.code()` / `frame.instruction_offset()` reads with thin-view equivalents (`cfr + HEADER_SLOTS`, `rust.dispatch.code_ref`, `rust.dispatch.pc`). Remove the `let active_frame = rust.dispatch.frame;` binding.
- [ ] **Step 3:** In `sync_from_asm`, remove the `frame.set_instruction_offset(...)` line (keep `rust.dispatch.pc = (**state).frame_pc_offset;`).
- [ ] **Step 4: Build + vm tests.** `/tmp/memtest.sh 'cargo build --workspace --all-targets --all-features && cargo clippy --workspace --all-targets --all-features 2>&1 | tail -30'` then `/tmp/memtest.sh 'cargo test -p lyng-vm --all-features 2>&1 | tail -20'`. Green.
- [ ] **Step 5: Commit.**
```bash
git add -A && git commit -m "perf(vm): populate LlIntState from the overlay on Refresh; drop per-call/return FrameRecord reconstruct

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

### Task 11: Convert `finish_frame` / pop path to overlay-direct (no reconstruct)

**Files:** `crates/vm/src/vm/registers.rs`, `crates/vm/src/vm.rs`

- [ ] **Step 1:** In `finish_frame` (`registers.rs:106..`), replace `let frame = self.pop_current_frame();` + the post-release `reconstruct_frame_from_header(caller_cfr, ...)` with direct overlay reads:
  - returning frame's `return_register` ← `self.frame_header(self.current_cfr).return_register()` (read BEFORE release).
  - returning frame's `caller_cfr` ← header.
  - after `release_frame_to_caller`, write the return value via the caller's window: `caller_window = RegisterWindow::new(caller_cfr + HEADER_SLOTS, self.window_len_for(self.frame_header(caller_cfr).code()))`.
  - `refresh_running_context_to_caller` currently takes `&FrameRecord`; add/keep a cfr-based variant `refresh_running_context_to_caller_cfr(agent, popped_cfr)` that reads `caller_cfr` from the overlay.
- [ ] **Step 2:** Keep `pop_current_frame`/`reconstruct_frame_from_header` only if still used by generator snapshot/restore; otherwise mark for deletion in Task 13.
- [ ] **Step 3: Build + vm tests.** Green.
- [ ] **Step 4: Commit.**
```bash
git add -A && git commit -m "perf(vm): finish_frame reads return-register/window from the overlay; no reconstruct

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

### Task 12: Phase-3 checkpoint — full Test262 + first A/B bench signal

- [ ] **Step 1:** Whole-corpus Test262 (report `/tmp/t262-phase3.md`). Expected baseline.
- [ ] **Step 2:** Quick bench sanity (Richards + RayTrace only, few samples) to confirm the direction is positive before the full A/B in Phase 5:
Run: `/tmp/memtest.sh 'cargo build --release -p lyng-cli 2>&1 | tail -3 && tools/lyng-bench v8suite --samples 5 --filter "Richards|RayTrace" --lyng-bin target/release/lyng --report /tmp/bench-phase3.md 2>&1 | tail -20'`
(Match the actual `tools/lyng-bench` CLI.)

---

## Phase 4 — Delete the snapshot + dead bridge (green)

### Task 13: Remove `DispatchState.frame` and now-dead helpers

**Files:** `crates/vm/src/vm/dispatch_state.rs`, `crates/vm/src/vm/dispatch.rs`, `crates/vm/src/vm.rs`, `crates/vm/src/dsl/slow_path.rs`, `crates/vm/src/dsl/entry.rs`, `crates/vm/src/dsl/test_helpers.rs`, `crates/vm/src/frame.rs`

- [ ] **Step 1:** Delete the `frame: FrameRecord` field from `DispatchState`. Fix the two constructors to build the thin view from `(cfr, depth, code, pc)` args instead of a `FrameRecord` (entry shim already has these; pass them through). Delete `sync_active_frame`/`refresh_from_active_frame` if unused, or rewrite to thin-view form.
- [ ] **Step 2:** Delete now-dead helpers: `sync_dispatch_frame` + `write_snapshot_into_backing`, `refresh_dispatch_frame`, and `reconstruct_frame_from_header` **iff** no caller remains (check generator restore). Delete `advance_dispatch_frame`/`next_dispatch_instruction_offset` if no longer used.
- [ ] **Step 3:** Update `current_instruction_offset` (already thin-view from Task 3); delete `resolve_initial_this_value(&FrameRecord)` if `resolve_initial_this_value_from_header` fully replaced it (keep whichever generator/entry still needs).
- [ ] **Step 4:** Compile-error-drive the cleanup: `cargo build --workspace --all-targets --all-features` and fix every reference until green. Run clippy.
- [ ] **Step 5: Full vm + workspace tests.** `/tmp/memtest.sh 'cargo test --workspace --all-features 2>&1 | tail -30'`. Green.
- [ ] **Step 6: Commit.**
```bash
git add -A && git commit -m "refactor(vm): delete the DispatchState frame snapshot and the snapshot↔backing bridge

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

### Task 14: Update SP-0b invariant debug-asserts + offset docs

**Files:** `crates/vm/src/vm.rs` (debug-assert helpers from T17), `crates/vm/src/dsl/reg_convention.rs`

- [ ] **Step 1:** Audit the T17 debug-assertions — any that reference the snapshot must move to the thin view / overlay. Ensure `debug_assert_cfr_chain_invariant` etc. still hold.
- [ ] **Step 2:** Confirm `frame_header_offsets_stable` and `LLINT_STATE_*` offset tests still pass unchanged. `cargo test -p lyng-vm --all-features frame_header_offsets_stable`.
- [ ] **Step 3: Commit** (if changes).

---

## Phase 5 — Validation: Test262 baseline + clean A/B perf (the T18 gate)

### Task 15: Whole-corpus Test262 final

- [ ] **Step 1:** `/tmp/memtest.sh '... lyng-test262 --report /tmp/t262-final.md ...'`. MUST equal baseline: 49729/49729/0/0, 3324 skips. Any delta blocks merge.

### Task 16: Clean same-machine v8 A/B bench

- [ ] **Step 1: Build the baseline binary** from the SP-0b merge-base in a worktree:
```bash
git worktree add /tmp/lyng-base 1b39a0ec
/tmp/memtest.sh 'cd /tmp/lyng-base && cargo build --release -p lyng-cli 2>&1 | tail -3'
```
- [ ] **Step 2: Build the branch binary:** `/tmp/memtest.sh 'cargo build --release -p lyng-cli 2>&1 | tail -3'`.
- [ ] **Step 3: Bench both** (subprocess-per-sample, 7 samples) under the watchdog, full v8 suite, writing to `/tmp`:
```bash
/tmp/memtest.sh 'tools/lyng-bench v8suite --samples 7 --lyng-bin /tmp/lyng-base/target/release/lyng --report /tmp/bench-base.md 2>&1 | tail -20'
/tmp/memtest.sh 'tools/lyng-bench v8suite --samples 7 --lyng-bin target/release/lyng --report /tmp/bench-sp0b.md 2>&1 | tail -20'
```
- [ ] **Step 4: Compare.** Acceptance: each score within noise of baseline (target ≈ Richards 516 / RayTrace 456 / etc. from the pre-regression baseline), with the call/return cluster (RayTrace, DeltaBlue) recovered to ≥ baseline. Record the table in the task report.
- [ ] **Step 5: Clean up the worktree:** `git worktree remove /tmp/lyng-base`.

### Task 17: Branch-level review + finish

- [ ] **Step 1:** Whole-branch diff review (spec-compliance + code-quality reviewer subagents) against the merge-base.
- [ ] **Step 2:** Invoke `superpowers:finishing-a-development-branch` for the merge/PR decision.

---

## Self-review notes

- **Spec coverage:** snapshot field reads (Phase 1–2), bridge reconstruct removal (Phase 3 = the perf fix), snapshot deletion (Phase 4), Test262 + A/B validation (Phase 5). The original SP-0b plan's T7/T8 "read the overlay directly" intent is completed by Phases 1–2; T18's perf gate is Phase 5.
- **Type consistency:** thin-view field names (`cfr`, `pc`, `code_ref`, `regs_len`) and accessor names (`pc()`, `cfr()`, `registers()`, `this_value()`, …) are used consistently across tasks. `resolve_initial_this_value_from_header` is the overlay variant introduced in Task 10 and consumed by the Refresh arm.
- **Risk:** Phase 2 borrow-checker churn (overlay reads need `&vm`); mitigation = pull Copy locals before `&mut vm` destructure (documented in Background). Each phase ends on a green full Test262, so a regression is bisectable to one phase.
- **Open items to confirm during execution (not placeholders — explicit verification steps):** `vm.frame_cold` field visibility from `dispatch_state.rs` (Task 6 Step 2); exact `FrameColdState` field names (`frame_cold.rs`); whether `reconstruct_frame_from_header` retains a generator-restore caller (Task 11 Step 2 / Task 13 Step 2); `Vm::cfr_of` reachability (Task 1 Step 3).
