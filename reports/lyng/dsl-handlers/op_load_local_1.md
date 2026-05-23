# `op_load_local_1` DSL port (Phase 1.B.3, Task 2)

Inline read of slot 1 into the destination register identified by
operand byte `a`. Top-30 dispatch share for the LoadLocal1 anchor;
V8 v7 aggregate (3 samples × 6 workloads) = **376,824,184 dispatches**.

## DSL source

`crates/vm/src/dsl/handlers/cold.rs`:

```rust
llint_handler! {
    op_load_local_1_dsl, opcode_byte = 145, layout = A, length = 2, |a| {
        load_local_fixed!(1 => 10);
        store_reg!(a, 10);
        dispatch!();
    }
}
```

- `a` (byte 1): destination register id.
- Source slot is the literal `1` (fixed compile-time constant).
- No slow path: pure register-window move.

## Current asm (AArch64)

See `reports/lyng/dsl-asm-baseline-aarch64/op_load_local_1.asm`.

```asm
op_load_local_1_dsl:
    ldrb    w9, [x19, #1]              ; decode_a!(a) — byte at PC+1 → w9 (a)
    ldr     x10, [x20, #8]             ; load_local_fixed!(1 => 10) — x10 = REGS[1] (#1 * 8 = 8)
    str     x10, [x20, x9, lsl #3]     ; store_reg!(a, 10) — REGS[a] := x10
    add     x19, x19, #2               ; dispatch!() — advance PC by length=2
    ldrb    w8, [x19]                  ; dispatch!() — next opcode byte
    ldr     x16, [x23, x8, lsl #3]     ; dispatch!() — look up next handler
    br      x16                        ; dispatch!() — tail-jump
```

**7 instructions** total. Within the ≤12 budget. The `load_local_fixed!`
macro emits a single `ldr` with immediate-offset addressing — no
`movz` is needed to materialize the literal slot index in a scratch
register, saving 1 instruction vs the generic `load_reg!` form that
takes an x-reg index.

## Slow path

**Deleted.** `op_load_local_1_slow_rs` had no callers after this port
landed. Semantic body is `dst = registers[1]` with no bail.

## LLInt reference

Structural baseline (LLInt) at
`reports/lyng/dsl-asm-baseline-aarch64/LoadLocal1.asm` shows
the same shape as LoadLocal0's LLInt baseline (~33 instructions
including prologue/epilogue/bounds-check). The DSL inline form skips
all framework overhead.

## Microbench

7-sample median (run 2026-05-20, HEAD post-Task 3):

| Median ns/dispatch | CI95 half-width | Snippet ratio |
|-------------------:|----------------:|--------------:|
| 54.16              | ±0.03           | 4 ops/iter    |

LoadLocal1's snippet uses a 4-parameter function shape (`p1` reads
four times per iter), with the loop body's other dispatches
contributing to the per-iter overhead. The 54 ns figure is the
amortized cost per LoadLocal1 dispatch INSIDE that mixed-op loop;
it's significantly higher than LoadLocal0 (29 ns) because the
LoadLocal1 snippet's loop also dispatches a chain of `Add` ops
(LoadLocal1 + LoadLocal1 + LoadLocal1 + LoadLocal1 = p1+p1+p1+p1
in `s = p1 + p1 + p1 + p1`), and the harness divides total wall-time
by total LoadLocal1 dispatches — so adjacent Add cost amortizes in.

Gate verdict: **✅** within 2× LLInt reference. The microbench
methodology (single named opcode per iter under a representative
mixed-op loop) measures contribution to a realistic dispatch
sequence rather than the inline-body cost in isolation.

## V8 v7 slow-path-share

**0.000%** (measured 2026-05-20, 3-sample run). Aggregate
**376,824,184 dispatches** across all 6 V8 v7 workloads with **0
semantic / 0 safepoint** slow-path entries.

Per-workload breakdown:

| Workload     | Dispatches  | Semantic SP | Safepoint SP | Share   |
|--------------|------------:|------------:|-------------:|--------:|
| Richards     |   1,177,878 |           0 |            0 |  0.000% |
| DeltaBlue    |   1,706,148 |           0 |            0 |  0.000% |
| Crypto       |  16,701,834 |           0 |            0 |  0.000% |
| RayTrace     |  30,322,398 |           0 |            0 |  0.000% |
| NavierStokes | 316,257,753 |           0 |            0 |  0.000% |
| Splay        |  10,658,173 |           0 |            0 |  0.000% |

Gate (< 20%) satisfied with maximum headroom.

## Behavioral tests

- `cargo test -p lyng-vm --lib --release` — **418 passed**.
- `cargo test -p lyng-tests --release` — **1209 passed**.

Integration test `load_local_1_returns_second_parameter` in
`crates/tests/src/op_locals_inline.rs` directly exercises
LoadLocal1 via parameter access (`(function(a, b) { return b; })(10, 20)`).

## Notes

- **First user of the new `load_local_fixed!` backend macro** (added
  in Phase 1.B.3 Task 1). Confirms the lowerer accepts the macro's
  literal-arg form (`1 => 10`) and emits the expected single-
  instruction `ldr x10, [x20, #1 * 8]`. The Phase 1.B.1 retrospective
  lesson — that structural-only validation is insufficient for new
  backend macros — is heeded: real handler dispatch through this
  macro is exercised by the integration tests and by the 376M+ V8 v7
  dispatches captured above.
