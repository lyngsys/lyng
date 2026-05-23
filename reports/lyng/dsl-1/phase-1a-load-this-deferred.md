# `op_load_this` deferred from Phase 1.A

**Status:** Deferred to Phase 1.B alongside `op_load_const8` (combined frame-context refactor).
**Plan section:** [`docs/superpowers/plans/2026-05-18-dsl-1-phase-1a-trivial-loads.md`](../../../docs/superpowers/plans/2026-05-18-dsl-1-phase-1a-trivial-loads.md) Task 9 off-ramp.
**Off-ramp triggered:** 2026-05-18 during Task 9 investigation.
**Companion deferral:** [`phase-1a-load-const8-deferred.md`](phase-1a-load-const8-deferred.md) — same root cause.

## Why deferred

`this` is not stored at a fixed register slot. The semantic body at [`crates/vm/src/vm/semantics/names.rs:600-627`](../../../crates/vm/src/vm/semantics/names.rs) resolves `this` dynamically through `agent.current_execution_context()` with three `ThisState` variants (`Value`, `Uninitialized`, `Lexical`), with a fallback to `frame.this_value()` on the [`FrameRecord`](../../../crates/vm/src/frame.rs).

```
state.rust_context (opaque)
  → LlIntRustContext { agent, frame, .. }
    ├─ agent.current_execution_context() → Option<ExecutionContextEntry>
    │    ├─ Some(ec) → ec.this_state() → ThisState { Value | Uninitialized (throws) | Lexical (lex-env walk) }
    │    └─ None    → frame.state.this_value  (3+ indirections)
```

None of these paths are reachable from asm without either (a) a multi-indirection chase through non-`#[repr(C)]` Rust types, or (b) a precomputed asm-visible Value-typed slot on `LlIntState`.

## Why this is harder than `op_load_const8`

Unlike `op_load_const8`'s `Smi`/`Float64Bits` variants which can be pre-resolved at install time, `op_load_this` faces:

- **`ThisState::Lexical`** requires walking the lexical environment chain at resolution time (see `Vm::resolve_this_binding` in [`vm/src/bytecode_calls.rs`](../../../crates/vm/src/vm/bytecode_calls.rs)), which can change between calls.
- **`ThisState::Uninitialized`** must throw `ReferenceError` on observation; cannot be eagerly resolved without altering throw semantics (loop-invariant throws would surface at the wrong PC).
- **Dominant path is via `agent.current_execution_context()`,** not `frame.this_value()`. The execution-context `this_state` can override the frame slot; inlining only the frame slot diverges from current semantics.

A bailout-on-discriminant approach (read both Option<ExecutionContextEntry> and ThisState discriminants in asm) takes 4-5 loads to decide whether to bail — strictly more work than the current `call_slow!` shim's single branch-and-link.

## Recommendation: co-design with `op_load_const8` refactor in Phase 1.B

Both deferred opcodes want the same kind of asm-visible pre-resolved Value-typed frame-context field on [`LlIntState`](../../../crates/vm/src/dsl/llint_state.rs). A unified refactor:

1. **Add fields to `LlIntState`:**
   - `frame_const_base: *const Value` (for `op_load_const8`, indexed)
   - `frame_this_value: Value` (for `op_load_this`, fixed offset)
2. **Add offsets to [`reg_convention.rs`](../../../crates/vm/src/dsl/reg_convention.rs)** via `offset_of!`.
3. **Pre-resolve at activation entry** in [`entry.rs`](../../../crates/vm/src/dsl/entry.rs):
   - Constants: resolve all `ConstantValue` entries to flat `Value` array.
   - This: resolve the `ThisState::Value` happy path; design a sentinel scheme for `Uninitialized`/`Lexical` that branches to slow path.
4. **Add refresh discipline** in [`slow_path.rs`](../../../crates/vm/src/dsl/slow_path.rs) bridges, mirroring PB/REGS/FV.
5. **Add DSL backend macros:**
   - `load_constant!($idx => $dst)` for indexed loads
   - Fixed-offset Value-load for this (note: Value is 16 bytes, so 2× `ldp` or a sentinel-aware single-load approach)
6. **GC root-scanning design review** — both fields are GC roots requiring scanning alongside register stack roots.

Estimated: 2-3 days of implementation + GC design review.

## Impact on Phase 1.A exit gate

`op_load_this` is top-30 dispatch share #12 (~256M dispatches per V8 v7 run vs Move's 4.6B at #1). Deferring it costs measurable V8 v7 score points but leaves the Phase 1.A ≥ +5% gate achievable via the 7 ports that did land:

| Ported | Top-30 share | Notes |
|--------|-------------:|-------|
| op_load_undefined | (not in top-30) | adjacent-family |
| op_load_null | (not in top-30) | adjacent-family |
| op_load_true | (not in top-30) | adjacent-family |
| op_load_false | (not in top-30) | adjacent-family |
| op_load_zero | #16 | top-30 |
| op_load_one | (not in top-30) | adjacent-family |
| op_load_smi8 | #7 | top-30 — highest in phase |

LoadSmi8 alone (#7, 388M dispatches/run) is the dominant Phase 1.A contributor. The Phase 1.A summary at Task 10 will document `op_load_this` and `op_load_const8` deferrals as known limitations.

Cold-stub at [`crates/vm/src/dsl/handlers/cold.rs`](../../../crates/vm/src/dsl/handlers/cold.rs) for `op_load_this_dsl` remains in place; the opcode continues to dispatch through `op_load_this_slow_rs` during Phase 1.A.
