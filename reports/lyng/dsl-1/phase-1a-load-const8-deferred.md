# `op_load_const8` deferred from Phase 1.A

**Status:** Deferred to Phase 1.B (or earlier if a focused refactor is scheduled).
**Plan section:** [`docs/superpowers/plans/2026-05-18-dsl-1-phase-1a-trivial-loads.md`](../../../docs/superpowers/plans/2026-05-18-dsl-1-phase-1a-trivial-loads.md) Task 8 off-ramp.
**Off-ramp triggered:** 2026-05-18 during Task 8 investigation.

## Why deferred

The asm-visible [`LlIntState`](../../../crates/vm/src/dsl/llint_state.rs) has no `frame_const_base` field. The constants array is not a flat `*const Value` — it lives behind several pointer hops as a `Vec<ConstantValue>` enum, where the `ConstantValue::Atom` and `ConstantValue::Builtin` variants require runtime heap-allocation or builtin lookup to materialize a `Value`. Neither is feasible inline in asm.

Path from the asm-visible state to a constant today:

```
state.rust_context (opaque)
  → LlIntRustContext
    → DispatchState
      → frame.code()
        → vm.installed[code_index(code)]
          → installed.function.constants()    // &[ConstantValue]
            → [index]
              → match ConstantValue { Smi(i32) | Float64Bits(u64) | Atom(AtomId) | Builtin(BuiltinFunctionId) }
```

`ConstantValue` enum definition: [crates/bytecode/src/metadata.rs:152-159](../../../crates/bytecode/src/metadata.rs).

Even a minimal inline path that handles only the `Smi`/`Float64Bits` cases and bails to a slow path for `Atom`/`Builtin` would need 6+ loads to reach the discriminant — strictly worse than the current full slow-path call.

## What the refactor would require

To enable inlining, lyng needs:

1. **Asm-visible `frame_const_base: *const Value` on [`LlIntState`](../../../crates/vm/src/dsl/llint_state.rs)** — pointer to a precomputed flat `Value` array of resolved constants.
2. **`LLINT_STATE_FRAME_CONST_BASE_OFFSET` const in [`reg_convention.rs`](../../../crates/vm/src/dsl/reg_convention.rs)** — exposed via `offset_of!` for the DSL backend.
3. **Eagerly resolve `Atom`/`Builtin` constants at function install time** into a flat `Box<[Value]>` per `InstalledFunction`. This implies:
   - Resolving atoms (heap-allocated strings) at install time, or lazily with a stable base pointer.
   - GC root scanning must include this flat array (atoms and builtins are GC roots).
   - Design review for the GC implications.
4. **Update the entry shim** ([`vm/src/dsl/entry.rs`](../../../crates/vm/src/dsl/entry.rs)) to populate the new field on `Vm::run` entry.
5. **Update the call/return slow paths** ([`vm/src/dsl/slow_path.rs`](../../../crates/vm/src/dsl/slow_path.rs)) to refresh the field on frame transitions, mirroring the existing PB/REGS/FV refresh discipline.
6. **New `load_constant!` DSL macro** in the backend.

Estimated: 1-2 days of implementation plus a GC root-scanning design review.

## Recommendation

Defer to **Phase 1.B (Local register access)**, where the parent spec at [`docs/superpowers/specs/2026-05-18-dsl-1-hot-opcode-rollout-design.md`](../../../docs/superpowers/specs/2026-05-18-dsl-1-hot-opcode-rollout-design.md) already scopes calling-convention and frame-layout work. The flat-constants design can be co-designed with the calling-convention work to avoid two separate `LlIntState` layout changes.

Cold-stub at [`crates/vm/src/dsl/handlers/cold.rs`](../../../crates/vm/src/dsl/handlers/cold.rs) remains in place; `op_load_const8` continues to dispatch through the slow path during Phase 1.A.

## Impact on Phase 1.A exit gate

Phase 1.A V8 v7 cumulative target is ≥ +5%. `op_load_const8` is top-30 dispatch share #21 (104M dispatches per V8 v7 run vs Move's 4.6B at #1). Its slow-path-share contribution is small but non-zero. The Phase 1.A summary at Task 10 will document this deferral as a known limitation; the +5% target is achievable without it via the other 4 top-30 ports in Phase 1.A (LoadSmi8 #7, LoadThis #12, LoadZero #16) plus the 5 adjacent-family ports (LoadUndefined, LoadNull, LoadTrue, LoadFalse, LoadOne).
