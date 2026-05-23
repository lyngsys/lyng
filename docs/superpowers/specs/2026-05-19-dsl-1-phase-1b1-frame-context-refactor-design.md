# Design: DSL-1 Phase 1.B.1 — Frame-context refactor

**Date:** 2026-05-19
**Status:** Design draft; awaiting user review.
**Parent spec:** [`2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md`](2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md) — Phase 1.B umbrella.
**Sibling specs:** [`2026-05-18-dsl-1-hot-opcode-rollout-design.md`](2026-05-18-dsl-1-hot-opcode-rollout-design.md) (epic), [`reports/lyng/dsl-1/phase-1b0-summary.md`](../../../reports/lyng/dsl-1/phase-1b0-summary.md) (predecessor sub-phase).
**Baseline HEAD:** `ae8b7766` (Phase 1.B.0 closed).
**Deferral inputs:**
- [`reports/lyng/dsl-1/phase-1a-load-const8-deferred.md`](../../../reports/lyng/dsl-1/phase-1a-load-const8-deferred.md)
- [`reports/lyng/dsl-1/phase-1a-load-this-deferred.md`](../../../reports/lyng/dsl-1/phase-1a-load-this-deferred.md)

---

## 1. Goal, scope, exit criteria

### Goal

Land the asm-visible frame-context substrate (`frame_const_base`, `frame_this_value`) on `LlIntState` so that **Phase 1.B.2** can inline-port `op_load_const8` (#21) and `op_load_this` (#12), and so **Phase 1.B.3**'s locals/Ldar/LoadEnvSlot ports inherit the populated frame context.

This sub-phase is substrate-only. **No opcode handlers are ported in 1.B.1.**

### In scope

- Two new fields on the `#[repr(C)]` `LlIntState`: `frame_const_base: *const Value` and `frame_this_value: Value`.
- Two new `LLINT_STATE_*` offset consts in `reg_convention.rs`.
- A `resolve_initial_this_value` helper that maps a frame + execution context's `ThisState` to either the real `Value` or the bail sentinel `Value::uninitialized_lexical()`.
- Population at trampoline entry (`entry.rs::run_via_dsl`).
- Refresh in `slow_path.rs::translate_outcome`'s Refresh arm.
- Two new backend macros: `load_constant!` (indexed load from `frame_const_base`) and `load_state_value!` (fixed-offset Value load from `LlIntState`).
- Debug-only stability assertion in the Refresh arm.
- Updated `ll_int_state_offsets_stable` test.
- A `dsl_validation_frame_context.rs` integration test that exercises the new fields through a synthetic handler.
- A `gc-stress` integration test that catches stale-mirror bugs.
- A GC review document.

### Out of scope

- Inline ports of `op_load_const8` and `op_load_this` — Phase 1.B.2.
- Phase 1.B.3 opcode ports (locals, Ldar, LoadEnvSlot).
- Pre-resolution of `Atom`/`Builtin` constants beyond what's already done at install time (the existing `Vm::install_constants` pipeline already populates `RuntimeCodeRecord::constants` — we reuse it).
- Any new `Vm` field. All state goes on existing structures (`LlIntState`, `LlIntRustContext` helper, no new top-level Vm storage).
- Counter wiring (already done in 1.B.0).

### Exit criteria

1. **Layout stable.** `ll_int_state_offsets_stable` asserts new offsets and updated total size.
2. **Behavioral parity.** `cargo test -p lyng-vm --lib --release` (≥413 passing), `cargo test -p lyng-tests --release` (≥1186 passing). 2 pre-existing `feedback_flat_consistency` failures stay unrelated.
3. **Test262 ≥ baseline.** Pass count ≥ what's recorded at Phase 1.B.0 closure (captured at sub-phase kickoff).
4. **GC stress clean.** New `gc-stress` test with `this`-binding + closure tight loop passes. No use-after-free if Miri can run the slow-path bridge.
5. **Same-load A/B vs `ae8b7766`.** Aggregate V8 v7 regression ≤ 2% (per parent §4 protocol).
6. **GC review documented.** `reports/lyng/dsl-1/phase-1b1-gc-review.md` covers both new fields and is signed off by the reviewer dispatch.
7. **Reviewer pass.** `feature-dev:code-reviewer` against the full sub-phase commit range; major findings addressed before sub-phase close.

---

## 2. Background: what we found

The deferral notes for `op_load_const8` and `op_load_this` define the requirements. Pre-1.B.1 investigation surfaced several useful facts:

### 2.1 Constants are already pre-resolved at install time

`Vm::install_constants` (in `crates/lyng/vm/src/vm/install.rs:715-754`) walks each `BytecodeFunction::constants: Vec<ConstantValue>` and produces a flat `Box<[Value]>`-equivalent stored in `RuntimeCodeRecord::constants: Option<CodeSlotsRef>` (an arena-allocated slot in the GC heap). The `Atom`/`Builtin` resolution and any required allocations happen there; the resulting `&[Value]` is reachable via `heap.view().code_slots(id)`.

**Implication:** we don't need new install-time work. The pre-resolved Values already exist; we only need to expose a pointer to them on `LlIntState`.

### 2.2 The existing `LlIntState` already mirrors arena pointers

`frame_pb_base: *const u8` points into the active code record's instruction bytes — also an arena pointer. The Refresh arm of `slow_path.rs::translate_outcome` (around lines 239-297) re-derives it from `installed.function().instruction_bytes()` after every frame transition. The same pattern applies to `frame_regs_base` (heap-allocated register stack) and `frame_fv_base` (per-code feedback slab).

**Implication:** `frame_const_base` reuses the exact same mirror discipline. The Refresh arm refreshes pointers; handlers only read between Refresh egress points; GC can only happen during slow-path bridges → mirror always sees post-GC values when handlers run.

### 2.3 `Value::uninitialized_lexical()` is an existing const sentinel

`Value` is 8 bytes (NaN-tagged `u64`). `Value::uninitialized_lexical()` is already defined in `crates/lyng/types/src/value.rs` (line 186-188) as a const sentinel value — it's used for TDZ checks in the lexical environment system, so it can never appear as a legitimate `this` binding.

**Implication:** the sentinel for "bail to slow path on `this` load" is already designed, named, and asserted unique. We reuse it.

### 2.4 `frame.this_value` mutation happens through slow paths

`frame.set_this_value` (frame.rs:474) is callable mid-activation (super() in derived constructors). Super() goes through a semantic body in `vm/src/vm/semantics/`, which returns through `translate_outcome`'s Refresh arm.

**Implication:** as long as the Refresh arm calls `resolve_initial_this_value` for the active frame, super() mutations propagate to `frame_this_value` automatically. No special wiring.

### 2.5 GC scanning is already complete

- `RuntimeCodeRecord::trace_heap_edges` (gc/src/rooting.rs:1637-1643) calls `self.constants().trace_heap_edges(tracer)`. The code record is reached via every frame in `vm.frames`, traced by `trace_frame_record` in `vm/src/vm/state.rs:293+`. The pre-resolved `Value`s in the arena slot are already scanned.
- `frame.this_value()` is scanned at `state.rs:298: frame.this_value().trace_heap_edges(tracer)`. The canonical source stays scanned.

**Implication:** no new GC scan code needed. Both new `LlIntState` fields are derived from already-scanned storage.

---

## 3. Architecture

### 3.1 `LlIntState` layout

```rust
#[repr(C)]
pub struct LlIntState {
    pub frame_pc_offset: u32,            // existing, offset 0
    pub _pad1: u32,                      // existing, offset 4
    pub frame_pb_base: *const u8,        // existing, offset 8
    pub frame_regs_base: *mut Value,     // existing, offset 16
    pub frame_fv_base: *mut FeedbackEntry, // existing, offset 24
    pub frame_const_base: *const Value,  // NEW, offset 32
    pub frame_this_value: Value,         // NEW, offset 40 (8 bytes)
    pub frame_depth: u32,                // existing, shifted to offset 48
    pub frame_check_epoch: u32,          // existing, shifted to offset 52
    pub rust_context: *mut LlIntRustContextOpaque, // existing, shifted to offset 56
    pub prefix: u8,                      // existing, shifted to offset 64
    pub _pad2: [u8; 7],                  // existing, shifted to offset 65
}
// Total size: 72 bytes (was 56). Math: existing 56 + 8 (frame_const_base) + 8 (frame_this_value) = 72.
// Both new fields are 8-aligned and slot in at offsets 32 and 40, which are natural 8-byte boundaries —
// no alignment padding is introduced. The trailing `prefix`/`_pad2` block shifts by 16 bytes wholesale.
```

**Field-ordering rationale:** new fields go between the existing base-pointers (`frame_fv_base`) and the existing scalar block (`frame_depth`/`frame_check_epoch`). This keeps the "hot pointer cluster" contiguous (PC, PB, REGS, FV, CONST, THIS_VALUE), maximizing the chance that an aarch64 dispatch reads them with adjacent ldr's. The trailing scalar/pointer block stays at the end.

### 3.2 New offset consts (`reg_convention.rs`)

```rust
pub const LLINT_STATE_FRAME_CONST_BASE: usize = offset_of!(LlIntState, frame_const_base);
pub const LLINT_STATE_FRAME_THIS_VALUE: usize = offset_of!(LlIntState, frame_this_value);
```

The existing `LLINT_STATE_PREFIX` const shifts; the test `ll_int_state_offsets_stable` is updated to assert the new values.

### 3.3 The `resolve_initial_this_value` helper

```rust
// In crates/lyng/vm/src/dsl/llint_state.rs or a new module.
#[inline]
pub(crate) fn resolve_initial_this_value(agent: &Agent, frame: &FrameRecord) -> Value {
    let this_state = agent.current_execution_context()
        .map(|ec| ec.this_state())
        .unwrap_or(ThisState::Value(frame.this_value()));
    match this_state {
        ThisState::Value(v) => v,
        ThisState::Uninitialized | ThisState::Lexical => Value::uninitialized_lexical(),
    }
}
```

**Key invariant:** this helper is the *single source of truth* for the sentinel rule. Any handler that wants to read `frame_this_value` MUST compare against `Value::uninitialized_lexical()` and bail to slow path on match. The slow path (`op_load_this_slow_rs`) handles all three `ThisState` arms and throws / walks the lex-env as appropriate.

### 3.4 Population sites

#### Trampoline entry (`entry.rs::run_via_dsl`)

Around lines 116-128 where `LlIntState` is constructed, add:

```rust
// Reaches the pre-resolved Value array via the existing heap-view API.
// `code_slots` returns &[Value] from gc/src/mutator.rs:242.
// The exact accessor for the CodeSlotsRef on InstalledFunction is determined
// by the refactor worker (existing precedent: see vm/src/vm/values.rs::read_constant
// for an example of code-slots-from-CodeRef resolution).
let const_base = vm.heap_view()
    .code_slots(installed.function_record_code_slots())
    .map(|s| s.as_ptr())
    .unwrap_or(std::ptr::null());  // null is safe — only opcodes with constants emit reads
let this_value = resolve_initial_this_value(&agent, &frame);
```

Then in the `LlIntState { ... }` literal, add:
```rust
frame_const_base: const_base,
frame_this_value: this_value,
```

The `null` fallback handles the edge case where a code record has no constants (empty function). Inline handlers that read `frame_const_base` are only emitted for opcodes that actually use constants (`op_load_const8`); those opcodes will never appear in a code stream with no constants array, so the null pointer is never dereferenced. The fallback exists for safety / clarity, not correctness.

#### Refresh arm (`slow_path.rs::translate_outcome`)

After the existing block that refreshes `frame_pc_offset`/`frame_pb_base`/`frame_regs_base`/`frame_fv_base`, add:

```rust
let const_base = vm.heap_view()
    .code_slots(installed.function_record_code_slots())
    .map(|s| s.as_ptr())
    .unwrap_or(std::ptr::null());
let this_value = resolve_initial_this_value(vm.agent(), &active_frame);

unsafe {
    (**state).frame_const_base = const_base;
    (**state).frame_this_value = this_value;
}
```

#### Continue arm (`slow_path.rs::translate_outcome`)

The Continue arm runs when a slow-path bridge returned to the SAME frame (e.g. semantic body ran but no call/return happened). In this case, `installed` and `frame.this_value()` are unchanged, so `frame_const_base` and `frame_this_value` are valid as-is.

**Decision: Continue arm does NOT write the new fields.** This matches what the existing Continue arm does for `frame_pb_base` (it doesn't refresh it). If the assumption ever breaks (e.g., a continue-path semantic body that mutates `this_value`), the debug-only stability assertion (§3.6) catches it in dev builds.

### 3.5 Backend macros

Both macros live under `crates/lyng/vm/src/dsl/backend/aarch64/`:

#### `load_constant!($idx_reg:expr => $dst_reg:expr)`

New file: `crates/lyng/vm/src/dsl/backend/aarch64/constants.rs`. Or extended into `frame.rs` if more frame-context macros land later — refactor worker's judgement.

Body emits:
```asm
ldr  {scratch}, [x22, #LLINT_STATE_FRAME_CONST_BASE]
ldr  {dst},     [{scratch}, {idx}, lsl #3]
```

The scratch register is one of the standard scratch register pool (the DSL macro infrastructure already reserves `x16`/`x17` IP0/IP1 for this purpose — same as `inc_dispatch_counter!` uses).

#### `load_state_value!($offset:expr => $dst_reg:expr)`

Extension to `crates/lyng/vm/src/dsl/backend/aarch64/frame.rs` (or a new `state.rs` if `frame.rs` is already crowded — refactor worker's judgement).

Body emits a single instruction:
```asm
ldr  {dst}, [x22, #{offset}]
```

Used for any fixed-offset 8-byte load from `LlIntState`. Phase 1.B.1 only uses it for `frame_this_value`, but `frame_pb_base`/`frame_regs_base`/`frame_fv_base` reads could migrate to it in a future refactor.

**Both macros are "skeleton" in 1.B.1.** The `dsl_validation_frame_context.rs` test exercises them via a synthetic handler so they don't bit-rot; Phase 1.B.2 will exercise them through the real `op_load_const8` and `op_load_this` ports.

### 3.6 Debug-only stability assertion

Added to `slow_path::translate_outcome`'s Refresh arm, gated on `debug_assertions`:

```rust
#[cfg(debug_assertions)]
{
    // Paranoia: confirm the const-base derivation is stable across the slow-path call.
    // If this ever fires, the arena moved under us — investigate before disabling.
    let recomputed = vm.heap_view()
        .code_slots(installed.function_record_code_slots())
        .map(|s| s.as_ptr())
        .unwrap_or(std::ptr::null());
    debug_assert_eq!(const_base, recomputed, "frame_const_base unstable across Refresh");
}
```

Release builds skip the assertion entirely. This catches misconfiguration during 1.B.2/1.B.3 development without paying production cost.

---

## 4. Data flow

```
At trampoline entry (entry.rs::run_via_dsl):
  vm.installed[code_idx] (Arc<InstalledFunction>)
    → installed.function_record_code_slots() (CodeSlotsRef)
    → vm.heap_view().code_slots(slot) -> &[Value]
    → as_ptr() → frame_const_base

  agent.current_execution_context()
    → ec.this_state() (ThisState)
    → resolve_initial_this_value() → Value (real value OR Value::uninitialized_lexical())
    → frame_this_value

At slow-path Refresh egress (slow_path.rs::translate_outcome refresh arm):
  vm.frames().last() → active_frame
  installed = vm.installed_for_dsl_runtime(active_frame.code())
  ...same two derivations as above, written back through *mut LlIntState

In asm handlers (Phase 1.B.2+ — shown here for context, NOT in 1.B.1 scope):
  op_load_const8:
    decode idx (u8 operand)
    load_constant!(idx => result)            ; 2 instructions
    dispatch!()

  op_load_this:
    load_state_value!(LLINT_STATE_FRAME_THIS_VALUE => result)   ; 1 instruction (ldr)
    ; Sentinel compare inline in the handler body (NOT a 1.B.1 macro).
    ; ~3 instructions to materialize the 64-bit sentinel + cmp + branch:
    ;   ldr {scratch}, =Value::uninitialized_lexical().bits()   ; literal pool load
    ;   cmp result, {scratch}
    ;   b.eq slow_op_load_this
    ; (scratch register from the IP0/IP1 scratch pool.)
    move_to_dest!(result, dst_reg)
    dispatch!()
```

---

## 5. GC integration

### 5.1 Why no new scan code is needed

| Field | Canonical source | Already scanned by | Reachability |
|-------|-------------------|---------------------|--------------|
| `frame_const_base` | `RuntimeCodeRecord::constants` (CodeSlotsRef arena slot) | `RuntimeCodeRecord::trace_heap_edges` (gc/src/rooting.rs:1637-1643) | Every frame in `vm.frames` carries a `CodeRef`; `trace_frame_record` walks it → reaches code record → reaches constants slot |
| `frame_this_value` | `FrameRecord::this_value` | `trace_frame_record` (state.rs:298: `frame.this_value().trace_heap_edges(tracer)`) | Every frame in `vm.frames` |

The new fields are **mirrors of already-scanned state**. They are valid only between Refresh egress points; GC can only happen during slow-path bridges; Refresh refreshes both fields after the bridge returns. So a handler always reads a post-GC-valid value/pointer.

### 5.2 Mirror discipline (documented and tested)

> **INVARIANT:** Reads of `frame_const_base`/`frame_this_value` from asm handlers are valid only between Refresh egress events. Any code path that can trigger GC MUST egress to `translate_outcome`'s Refresh arm before re-entering the handler dispatch.

This invariant is identical to what already holds for `frame_pb_base`, `frame_regs_base`, `frame_fv_base`. The mirror lifetime is bounded by Refresh-to-Refresh windows.

### 5.3 GC review deliverable

A standalone document at `reports/lyng/dsl-1/phase-1b1-gc-review.md` will cover:
- Per-field reachability proof (canonical source → trace path).
- Mirror-staleness argument (Refresh egress = safe point).
- Stability of arena slot data pointer (matches `frame_pb_base` precedent).
- Risk surface and mitigations.
- Reviewer dispatch sign-off (recorded as a comment block at the end).

---

## 6. Test plan

### 6.1 Unit tests (in `crates/lyng/vm/src/dsl/llint_state.rs`)

| Test | Asserts |
|------|---------|
| `ll_int_state_offsets_stable` (updated) | All offsets including the two new ones; new total size |
| `resolve_initial_this_value_value_state` | `ThisState::Value(v)` returns `v` |
| `resolve_initial_this_value_uninitialized` | `ThisState::Uninitialized` returns `Value::uninitialized_lexical()` |
| `resolve_initial_this_value_lexical` | `ThisState::Lexical` returns `Value::uninitialized_lexical()` |
| `resolve_initial_this_value_no_ec_fallback` | No `current_execution_context()` falls back to `frame.this_value()` |

### 6.2 Integration tests

| File | Tests |
|------|-------|
| `crates/lyng/vm/tests/dsl_validation_frame_context.rs` (new) | Three synthetic handlers, modeled on existing `dsl_validation_*.rs` test handlers: (1) reads `frame_const_base[0]` via `load_constant!` and asserts the value matches a known-pre-resolved constant; (2) reads `frame_this_value` via `load_state_value!` in a frame with `ThisState::Value(v)` and asserts the value equals `v`; (3) reads `frame_this_value` in a frame with `ThisState::Uninitialized` and asserts the value equals `Value::uninitialized_lexical()`. |
| `crates/lyng-tests/` (`lyng-tests` crate) | GC-stress test: tight loop with a closure that reads `this` and allocates new objects. Uses whatever GC-stress mechanism the repo currently supports (refactor worker investigates: e.g., `--cfg gc_stress_force_collect`, or explicit `force_minor_gc()` calls between iterations). Asserts the loop completes correctly and `this` is observed correctly each iteration. The reviewer dispatch confirms the test actually exercises a frame-context refresh across a GC event. |

### 6.3 Behavioral parity

- `cargo test -p lyng-vm --lib --release` — ≥413 passing.
- `cargo test -p lyng-tests --release` — ≥1186 passing.
- 2 pre-existing `feedback_flat_consistency` failures are unchanged (stay unrelated).

### 6.4 Test262

Run at sub-phase kickoff to capture baseline. Re-run at sub-phase close. Pass count must be ≥ baseline.

### 6.5 V8 v7 same-load A/B

Per parent spec §4 protocol:
- Comparison base: `ae8b7766` (Phase 1.B.0 closed).
- Comparison post: Phase 1.B.1 final commit.
- 7-sample medians, `uptime` within ±20%.
- Aggregate V8 v7 regression must be ≤ 2% (parent §4 gate for 1.B.1).
- Per-workload tolerance: no workload regresses > 5% (per §4 of parent epic spec).
- Output committed to `reports/lyng/dsl-1/phase-1b1-ab-comparison.md`.

### 6.6 Reviewer dispatch

`feature-dev:code-reviewer` invoked against the full sub-phase commit range. Brief includes:
- Pointer to this design doc.
- Explicit list of GC invariants to verify.
- Pointer to `phase-1b1-gc-review.md`.
- Asks for an independent read on the resolve_initial_this_value semantics vs the existing `op_load_this` semantic body in `vm/src/vm/semantics/names.rs:600-627`.

---

## 7. Implementation phasing within 1.B.1

Single refactor worker subagent, ~3-4 days wall-clock. Suggested commit sequence (refactor worker's own TodoWrite/TaskCreate breakdown):

1. **Task 1: layout + offsets** — extend `LlIntState`, add offset consts, update `ll_int_state_offsets_stable`. Tiny commit, no behavior change yet (fields populated to defaults).
2. **Task 2: resolve helper** — add `resolve_initial_this_value` with full unit test coverage.
3. **Task 3: entry-shim population** — wire population at trampoline entry. Behavior unchanged because no handler reads the new fields yet. Behavioral parity check.
4. **Task 4: Refresh arm wiring** — wire refresh in `slow_path.rs`. Include debug-only stability assertion. Behavioral parity check.
5. **Task 5: backend macros** — add `load_constant!` and `load_state_value!`. Skeleton; no opcode uses them.
6. **Task 6: synthetic validation handler** — add `dsl_validation_frame_context.rs`. Exercises both macros end-to-end through a test-only handler. Catches macro-emit bugs without depending on 1.B.2 opcodes.
7. **Task 7: gc-stress test** — add the closure-this allocation-pressure test in `lyng-tests`. Run with whatever stress mechanism the repo currently supports.
8. **Task 8: GC review doc + V8 v7 A/B** — write `phase-1b1-gc-review.md`, run same-load A/B, commit comparison.
9. **Task 9: reviewer dispatch** — dispatch `feature-dev:code-reviewer`. Address major findings.
10. **Task 10: sub-phase summary** — write `reports/lyng/dsl-1/phase-1b1-summary.md` mirroring the 1.B.0 format. Close sub-phase.

Each task is a single commit. Behavioral parity (and the existing 413 + 1186 test suites) must pass at every commit boundary.

---

## 8. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------:|-------:|-----------|
| Arena pointer for `constants` moves across a Refresh boundary | low | high | Debug-only stability assertion catches it in dev; matches `frame_pb_base` precedent which has the same dependency |
| `resolve_initial_this_value` diverges from `op_load_this` semantic body's resolution rule | medium | high | Side-by-side reviewer pass; unit tests for all three ThisState arms plus the no-EC fallback |
| Super() mutation not picked up because Continue arm doesn't refresh | low | medium | Super() returns through Refresh, not Continue (verified at impl time); test added with derived constructor + this-read |
| New `LlIntState` size breaks an unexpected assumption elsewhere | low | medium | `ll_int_state_offsets_stable` covers it; grep for `size_of::<LlIntState>` confirms no other call sites |
| GC stress test doesn't actually stress the relevant path | medium | low | Reviewer dispatch checks the test exercises closure-this + cross-frame allocation; if the stress mechanism is too weak, refactor worker adds explicit `force_minor_gc()` calls in the test |
| Backend macros land but bit-rot before 1.B.2 | low | low | `dsl_validation_frame_context.rs` synthetic handler exercises them in 1.B.1 itself |
| Refactor worker hits a snag and the sub-phase blows past 4 days | medium | medium | Off-ramp: if refactor regresses Test262 unfixably or GC review finds a fundamental issue, abort sub-phase, reset to `ae8b7766`. 1.B.3 ports proceed independently (don't depend on this refactor). |
| V8 v7 same-load A/B shows > 2% regression | low | medium | Per-handler microbench (1.B.0 infra) can localize the cause. If regression is from added Refresh-arm work, investigate inlining or fast-paths in the Refresh helper. |

---

## 9. Decisions made (for the record)

1. **Constants source: reuse `RuntimeCodeRecord::constants` arena slot.** Not a fresh `Box<[Value]>` on `InstalledFunction`, not a per-call cache. Smallest diff, no new GC work, identical mirror invariant to existing `frame_pb_base`.

2. **This sentinel: `Value::uninitialized_lexical()`.** Existing const, already asserted unique (used for TDZ). Reused as the "bail to slow path" marker for `ThisState::Uninitialized` and `ThisState::Lexical`.

3. **Field placement in `LlIntState`:** between `frame_fv_base` and `frame_depth`. Keeps the hot pointer cluster contiguous; doesn't break the existing offset stability test (only updates it).

4. **Refresh discipline:** Refresh arm refreshes both new fields; Continue arm does not. Debug-only stability assertion in Refresh arm catches violations during 1.B.2/1.B.3 development.

5. **Sentinel comparison stays in handler bodies**, not in the macros. Keeps macros minimal and reusable; handler bodies stay clear about the bail behavior.

6. **No new top-level `Vm` field, no new install-time work.** Reuses existing `RuntimeCodeRecord::constants` and existing `agent.current_execution_context()` / `frame.this_value()` APIs.

7. **`dsl_validation_frame_context.rs` synthetic handler is mandatory.** Without it, the backend macros could regress between 1.B.1 close and 1.B.2 start; the synthetic handler keeps them exercised.

---

## 10. References

- **Parent design:** [`docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md`](../../lyng/2026-05-16-asm-dsl-llint-interpreter-design.md) §10 DSL-1.
- **Phase 1.B umbrella spec:** [`2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md`](2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md).
- **Epic spec:** [`2026-05-18-dsl-1-hot-opcode-rollout-design.md`](2026-05-18-dsl-1-hot-opcode-rollout-design.md).
- **Predecessor sub-phase:** [`reports/lyng/dsl-1/phase-1b0-summary.md`](../../../reports/lyng/dsl-1/phase-1b0-summary.md).
- **Deferral notes:**
  - [`reports/lyng/dsl-1/phase-1a-load-const8-deferred.md`](../../../reports/lyng/dsl-1/phase-1a-load-const8-deferred.md)
  - [`reports/lyng/dsl-1/phase-1a-load-this-deferred.md`](../../../reports/lyng/dsl-1/phase-1a-load-this-deferred.md)
- **Key source files:**
  - `crates/lyng/vm/src/dsl/llint_state.rs` — struct + tests
  - `crates/lyng/vm/src/dsl/reg_convention.rs` — offset consts
  - `crates/lyng/vm/src/dsl/entry.rs` — trampoline entry
  - `crates/lyng/vm/src/dsl/slow_path.rs` — Refresh/Continue arms
  - `crates/lyng/vm/src/dsl/backend/aarch64/` — new macros land here
  - `crates/lyng/vm/src/vm/install.rs` — existing constants pipeline
  - `crates/lyng/vm/src/vm/semantics/names.rs` — `op_load_this` semantic body (reference for resolve helper)
  - `crates/lyng/types/src/value.rs` — `Value::uninitialized_lexical()` sentinel
  - `crates/lyng/env/src/execution.rs` — `ThisState` enum
- **GC scanning:**
  - `crates/lyng/gc/src/rooting.rs:1637-1643` — `RuntimeCodeRecord::trace_heap_edges`
  - `crates/lyng/vm/src/vm/state.rs:204-291` — `ActiveVmRoots`, `trace_frame_record`
