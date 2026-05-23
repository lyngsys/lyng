# `op_store_local_1` DSL port (Phase 1.B.3, Task 3)

Inline write of source-register value into slot 1. Macro-shared
symmetric pair of `op_store_local_3`. V8 v7 aggregate dispatches:
**3,187,008** (3 samples × 6 workloads).

## DSL source

`crates/vm/src/dsl/handlers/cold.rs`:

```rust
llint_handler! {
    op_store_local_1_dsl, opcode_byte = 149, layout = A, length = 2, |a| {
        load_reg!(a => 10);
        store_local_fixed!(10, 1);
        dispatch!();
    }
}
```

- `a` (byte 1): source register id.
- Destination slot = literal `1`.
- No slow path.

## Current asm (AArch64)

See `reports/lyng/dsl-asm-baseline-aarch64/op_store_local_1.asm`.

```asm
op_store_local_1_dsl:
    ldrb    w9, [x19, #1]              ; decode_a!(a) — byte at PC+1 → w9 (a)
    ldr     x10, [x20, x9, lsl #3]     ; load_reg!(a => 10) — x10 = REGS[a]
    str     x10, [x20, #8]             ; store_local_fixed!(10, 1) — REGS[1] := x10 (#1 * 8 = 8)
    add     x19, x19, #2               ; dispatch!() — advance PC by length=2
    ldrb    w8, [x19]                  ; dispatch!() — next opcode byte
    ldr     x16, [x23, x8, lsl #3]     ; dispatch!() — look up next handler
    br      x16                        ; dispatch!() — tail-jump
```

**7 instructions** total. Within the ≤12 budget.

## Slow path

**Deleted.** `op_store_local_1_slow_rs` had no callers after this
port landed.

## Microbench

7-sample median: **46.01 ns/dispatch ± 0.08** (4 ops/iter).

The StoreLocalN snippet shape (`pN = i` in a tight loop) drives the
peephole through `compact_move_instruction:162-164`'s
`store_local_opcode` branch for N in 1..=3. Gate verdict: **✅**
within 2× LLInt reference.

## V8 v7 slow-path-share

**0.000%** (3-sample run, 6 workloads). Aggregate **3,187,008
dispatches** with **0** slow-path entries.

| Workload     | Dispatches  | Slow-path Share |
|--------------|------------:|----------------:|
| Richards     |           0 |          0.000% |
| DeltaBlue    |           0 |          0.000% |
| Crypto       |      32,256 |          0.000% |
| RayTrace     |   3,154,536 |          0.000% |
| NavierStokes |         216 |          0.000% |
| Splay        |           0 |          0.000% |

RayTrace is the dominant dispatcher (99% of aggregate StoreLocal1
share). Gate (< 20%) satisfied.

## Behavioral tests

vm 418 / lyng-tests 1209. Integration test
`store_local_0_1_2_via_assignments` in
`crates/tests/src/op_locals_inline.rs` exercises StoreLocal1
via `b = b * 3` parameter mutation.

## Notes

Same `store_local_fixed!(10, N)` macro pattern as StoreLocal2/3 —
identical 7-instruction shape with only the literal slot offset
differing across the three handlers.
