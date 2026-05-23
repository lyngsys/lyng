# `op_load_local_3` DSL port (Phase 1.B.3, Task 2)

Inline read of slot 3 into the destination register identified by
operand byte `a`. Top-30 dispatch share for the LoadLocal3 anchor;
V8 v7 aggregate (3 samples × 6 workloads) = **273,185,846 dispatches**.

## DSL source

`crates/lyng/vm/src/dsl/handlers/cold.rs`:

```rust
llint_handler! {
    op_load_local_3_dsl, opcode_byte = 147, layout = A, length = 2, |a| {
        load_local_fixed!(3 => 10);
        store_reg!(a, 10);
        dispatch!();
    }
}
```

- `a` (byte 1): destination register id.
- Source slot = literal `3`.
- No slow path.

## Current asm (AArch64)

See `reports/lyng/dsl-asm-baseline-aarch64/op_load_local_3.asm`.

```asm
op_load_local_3_dsl:
    ldrb    w9, [x19, #1]              ; decode_a!(a) — byte at PC+1 → w9 (a)
    ldr     x10, [x20, #24]            ; load_local_fixed!(3 => 10) — x10 = REGS[3] (#3 * 8 = 24)
    str     x10, [x20, x9, lsl #3]     ; store_reg!(a, 10) — REGS[a] := x10
    add     x19, x19, #2               ; dispatch!() — advance PC by length=2
    ldrb    w8, [x19]                  ; dispatch!() — next opcode byte
    ldr     x16, [x23, x8, lsl #3]     ; dispatch!() — look up next handler
    br      x16                        ; dispatch!() — tail-jump
```

**7 instructions** total. Within the ≤12 budget.

## Slow path

**Deleted.** `op_load_local_3_slow_rs` had no callers after this
port landed.

## Microbench

7-sample median: **54.10 ns/dispatch ± 0.04** (4 ops/iter).

Gate verdict: **✅** within 2× LLInt reference.

## V8 v7 slow-path-share

**0.000%** (3-sample run, 6 workloads). Aggregate **273,185,846
dispatches** with **0** slow-path entries.

| Workload     | Dispatches  | Slow-path Share |
|--------------|------------:|----------------:|
| Richards     |   4,472,934 |          0.000% |
| DeltaBlue    |  11,498,406 |          0.000% |
| Crypto       | 172,478,826 |          0.000% |
| RayTrace     |   2,696,658 |          0.000% |
| NavierStokes |  81,400,545 |          0.000% |
| Splay        |     638,477 |          0.000% |

## Behavioral tests

vm 418 / lyng-tests 1209. Integration test
`load_local_3_returns_fourth_parameter` in
`crates/lyng/tests/src/op_locals_inline.rs` covers direct
fourth-parameter access.

## Notes

The largest LoadLocalN anchor by Crypto-workload share (172M
dispatches in Crypto alone — about 27% of Crypto's total
dispatches). The inline port eliminates the slow-path round-trip
across all of them.
