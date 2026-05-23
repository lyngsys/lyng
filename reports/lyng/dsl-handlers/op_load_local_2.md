# `op_load_local_2` DSL port (Phase 1.B.3, Task 2)

Inline read of slot 2 into the destination register identified by
operand byte `a`. Top-30 dispatch share for the LoadLocal2 anchor;
V8 v7 aggregate (3 samples × 6 workloads) = **144,349,854 dispatches**.

## DSL source

`crates/vm/src/dsl/handlers/cold.rs`:

```rust
llint_handler! {
    op_load_local_2_dsl, opcode_byte = 146, layout = A, length = 2, |a| {
        load_local_fixed!(2 => 10);
        store_reg!(a, 10);
        dispatch!();
    }
}
```

- `a` (byte 1): destination register id.
- Source slot = literal `2`.
- No slow path.

## Current asm (AArch64)

See `reports/lyng/dsl-asm-baseline-aarch64/op_load_local_2.asm`.

```asm
op_load_local_2_dsl:
    ldrb    w9, [x19, #1]              ; decode_a!(a) — byte at PC+1 → w9 (a)
    ldr     x10, [x20, #16]            ; load_local_fixed!(2 => 10) — x10 = REGS[2] (#2 * 8 = 16)
    str     x10, [x20, x9, lsl #3]     ; store_reg!(a, 10) — REGS[a] := x10
    add     x19, x19, #2               ; dispatch!() — advance PC by length=2
    ldrb    w8, [x19]                  ; dispatch!() — next opcode byte
    ldr     x16, [x23, x8, lsl #3]     ; dispatch!() — look up next handler
    br      x16                        ; dispatch!() — tail-jump
```

**7 instructions** total. Within the ≤12 budget. Same shape as
LoadLocal1 with the immediate offset adjusted to `#16` (= 2 * 8).

## Slow path

**Deleted.** `op_load_local_2_slow_rs` had no callers after this
port landed.

## Microbench

7-sample median: **53.86 ns/dispatch ± 0.04** (4 ops/iter snippet ratio).

Gate verdict: **✅** within 2× LLInt reference. Same methodology
caveat as LoadLocal1 (the median measures inline-body cost under
a mixed-op loop).

## V8 v7 slow-path-share

**0.000%** (3-sample run, 6 workloads). Aggregate **144,349,854
dispatches** with **0** slow-path entries.

| Workload     | Dispatches  | Slow-path Share |
|--------------|------------:|----------------:|
| Richards     |      14,535 |          0.000% |
| DeltaBlue    |     120,006 |          0.000% |
| Crypto       |   7,513,677 |          0.000% |
| RayTrace     |  16,146,906 |          0.000% |
| NavierStokes | 120,554,721 |          0.000% |
| Splay        |           9 |          0.000% |

## Behavioral tests

vm 418 / lyng-tests 1209. Integration test
`load_local_2_returns_third_parameter` in
`crates/tests/src/op_locals_inline.rs` covers direct
third-parameter access.

## Notes

Same `load_local_fixed!`-based macro shape as LoadLocal1/3 — the
literal slot offset is the only differentiator.
