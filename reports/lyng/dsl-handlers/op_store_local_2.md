# `op_store_local_2` DSL port (Phase 1.B.3, Task 3)

Inline write of source-register value into slot 2. Macro-shared
symmetric pair of `op_store_local_3`. V8 v7 aggregate dispatches:
**3,154,626** (3 samples × 6 workloads).

## DSL source

`crates/lyng/vm/src/dsl/handlers/cold.rs`:

```rust
llint_handler! {
    op_store_local_2_dsl, opcode_byte = 150, layout = A, length = 2, |a| {
        load_reg!(a => 10);
        store_local_fixed!(10, 2);
        dispatch!();
    }
}
```

- `a` (byte 1): source register id.
- Destination slot = literal `2`.
- No slow path.

## Current asm (AArch64)

See `reports/lyng/dsl-asm-baseline-aarch64/op_store_local_2.asm`.

```asm
op_store_local_2_dsl:
    ldrb    w9, [x19, #1]              ; decode_a!(a) — byte at PC+1 → w9 (a)
    ldr     x10, [x20, x9, lsl #3]     ; load_reg!(a => 10) — x10 = REGS[a]
    str     x10, [x20, #16]            ; store_local_fixed!(10, 2) — REGS[2] := x10 (#2 * 8 = 16)
    add     x19, x19, #2               ; dispatch!() — advance PC by length=2
    ldrb    w8, [x19]                  ; dispatch!() — next opcode byte
    ldr     x16, [x23, x8, lsl #3]     ; dispatch!() — look up next handler
    br      x16                        ; dispatch!() — tail-jump
```

**7 instructions** total. Within the ≤12 budget.

## Slow path

**Deleted.** `op_store_local_2_slow_rs` had no callers after this
port landed.

## Microbench

7-sample median: **45.95 ns/dispatch ± 0.07** (4 ops/iter). Gate
verdict: **✅** within 2× LLInt reference.

## V8 v7 slow-path-share

**0.000%** (3-sample run, 6 workloads). Aggregate **3,154,626
dispatches** with **0** slow-path entries.

| Workload     | Dispatches  | Slow-path Share |
|--------------|------------:|----------------:|
| Richards     |           0 |          0.000% |
| DeltaBlue    |           0 |          0.000% |
| Crypto       |           0 |          0.000% |
| RayTrace     |   3,154,410 |          0.000% |
| NavierStokes |         216 |          0.000% |
| Splay        |           0 |          0.000% |

RayTrace dominates (99.99% of aggregate share). Gate satisfied.

## Behavioral tests

vm 418 / lyng-tests 1209. Integration test
`store_local_0_1_2_via_assignments` covers StoreLocal2 via `c = c * 4`
parameter mutation.

## Notes

Same shape as StoreLocal1/3. The only V8 v7 workload with non-trivial
StoreLocal2 share is RayTrace (vector-math style code with mid-
function reassignments to parameter slots).
