# Phase 1.B.1 — GC root-scanning review

**Date:** 2026-05-19
**Sub-phase:** DSL-1 Phase 1.B.1 (frame-context refactor)
**HEAD reviewed:** `5a7ab6a8`
**Spec:** [`docs/superpowers/specs/2026-05-19-dsl-1-phase-1b1-frame-context-refactor-design.md`](../../../docs/superpowers/specs/2026-05-19-dsl-1-phase-1b1-frame-context-refactor-design.md)

## Summary

Phase 1.B.1 adds two new mirror fields to the asm-visible `LlIntState`:
- `frame_const_base: *const Value` — pointer into the active code record's pre-resolved constants array (`RuntimeCodeRecord::constants`).
- `frame_this_value: Value` — mirror of the active frame's `this` value, or `Value::uninitialized_lexical()` sentinel for `ThisState::Uninitialized`/`Lexical`.

Both fields are **mirrors of state that is already GC-scanned**. No new tracer code is required.

## Field-by-field reachability proof

### `frame_const_base`

The canonical source is `RuntimeCodeRecord::constants: Option<CodeSlotsRef>` (the arena-allocated slot in the GC heap). Reachability path:

1. Each frame in `vm.frames` carries a `CodeRef`.
2. `trace_frame_record` (`vm/src/vm/state.rs` frame-tracing path) walks each frame's `code()`.
3. `RuntimeCodeRecord::trace_heap_edges` (in `gc/src/rooting.rs`) walks the code record's `constants()`.
4. The `CodeSlotsRef`'s arena slot is then marked + its `Value`s scanned.

`frame_const_base` is a stack-local pointer into that arena slot. It is valid only between Refresh egress events; GC can only happen during slow-path bridges; the Refresh arm refreshes the pointer after the bridge returns; therefore handlers always read a post-GC-valid pointer.

### `frame_this_value`

The canonical source is `FrameRecord::this_value()` (set at frame push, mutable via `frame.set_this_value` during `super()` in derived constructors). Reachability:

1. `trace_frame_record` (in `vm/src/vm/state.rs`) explicitly traces `frame.this_value()` via `trace_heap_edges(tracer)`.

`frame_this_value` in `LlIntState` is a stack-local mirror. Same mirror-discipline invariant as `frame_const_base`: refreshed on every Refresh egress.

## Mirror discipline invariant

> **INVARIANT:** Reads of `frame_const_base`/`frame_this_value` from asm handlers are valid only between Refresh egress events. Any code path that can trigger GC MUST egress to `translate_outcome`'s Refresh arm before re-entering the handler dispatch.

This invariant mirrors the existing invariant for `frame_pb_base`, `frame_regs_base`, `frame_fv_base`. Phase 1.B.0 was at HEAD `ae8b7766` before this refactor; the existing fields have held under it across the full test suite and V8 v7 runs.

## Arena pointer stability

`RuntimeCodeRecord::constants` is an arena slot. The data pointer's stability across slow-path calls is supported by three arguments:

1. **Precedent:** `frame_pb_base` is a pointer into the code record's instruction bytes — the same arena. It has shipped in production since DSL-0c and has held under all V8 v7 and Test262 runs.
2. **Active-frame retention:** GC can compact unused records but cannot move an arena slot that's referenced by an active frame on `vm.frames`. The same retention preserves both the instruction bytes (used by `frame_pb_base`) and the constants slot (used by `frame_const_base`). The constants array and instruction bytes are reached through the same `CodeRef → RuntimeCodeRecord` indirection.
3. **Safety net:** Task 4 added a `#[cfg(debug_assertions)]` stability assertion in the Refresh arm. It re-derives `const_base` from the canonical chain and asserts equality with the value about to be written. **It did NOT fire across 417 vm-lib tests, 1187 lyng-js-tests, the gc-stress test (Task 7), or any V8 v7 sample.** Strong empirical evidence the assumption holds.

## Risk surface and mitigations

| Risk | Mitigation | Status |
|------|------------|--------|
| Arena pointer moves across a Refresh boundary | Debug-only stability assertion (Task 4) + `frame_pb_base` precedent | Not observed across test suite |
| `resolve_initial_this_value` diverges from `op_load_this` semantic body | Side-by-side reading of helper (`llint_state.rs`) and semantic body (`vm/src/vm/semantics/names.rs:600-627`) | Unit tests cover all 4 cases (Value passthrough, Uninitialized → sentinel, Lexical → sentinel, no-EC fallback) |
| `super()`-mutation of `frame.this_value()` not picked up | `super()` returns via Refresh, not Continue. Refresh arm refreshes `frame_this_value` | Wired in Task 4 |
| New fields scanned redundantly or missed by GC | Mirrors of already-scanned canonical state; no new tracer code | No-op for GC (verified: no GC code touched in this sub-phase) |
| Mid-dispatch GC misses a Refresh | Interpreter doesn't attach `ActiveVmRoots` during JS execution (Task 7 finding) → GC cannot fire during dispatch, only at safepoint-equivalents (which all egress via Refresh) | Structural safety |

## Task 7 GC-stress test caveat

The gc-stress test at `crates/lyng-js/tests/src/gc_stress_frame_context.rs` does not auto-trigger minor GC during the loop — the interpreter's mutator path doesn't attach `ActiveVmRoots` at allocation safepoints (documented finding from Task 7).

What the test **does** exercise:
- ~15K object allocations (nursery fills to ~1 MB)
- Every iteration routes through Rust slow paths that egress via `translate_outcome`'s Refresh arm → mirror-refresh discipline runs on every iteration
- A `force_collect()` at the end exercises full cross-frame trace validation
- Counter sum mismatches would result from broken mirror refresh

What the test does **not** exercise:
- Mid-dispatch GC (because the interpreter doesn't trigger it during dispatch)

This is a known limitation. Future work — when the parent design's safepoint substrate lands and `ActiveVmRoots` is attached during dispatch — should add a more aggressive gc-stress test that forces collections inside the dispatch loop.

## Verdict

**GC integration safe.** Both new `LlIntState` fields are mirrors of already-scanned canonical state. The mirror-discipline invariant matches the existing `frame_pb_base` precedent. The debug-only stability assertion provides a dev-build safety net. The Phase 1.B.0+1 test suite (417 vm-lib + 1187 lyng-js-tests) plus the new gc-stress test exercise the substrate without triggering issues. Same-load A/B against the baseline shows +0.80% geomean (slightly net-positive), within the ≤ 2% gate.

## Reviewer dispatch sign-off

(To be appended in Task 9.)
