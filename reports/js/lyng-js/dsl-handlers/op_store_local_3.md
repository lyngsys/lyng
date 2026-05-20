# `op_store_local_3` DSL port (Phase 1.B.3, Task 3)

Inline write of source-register value into slot 3. **Top-30
anchor** — V8 v7 aggregate dispatches: **101,644,452** (3 samples ×
6 workloads).

## DSL source

`crates/lyng-js/vm/src/dsl/handlers/cold.rs`:

```rust
llint_handler! {
    op_store_local_3_dsl, opcode_byte = 151, layout = A, length = 2, |a| {
        load_reg!(a => 10);
        store_local_fixed!(10, 3);
        dispatch!();
    }
}
```

- `a` (byte 1): source register id.
- Destination slot = literal `3`.
- No slow path.

## Current asm (AArch64)

See `reports/js/lyng-js/dsl-asm-baseline-aarch64/op_store_local_3.asm`.

```asm
op_store_local_3_dsl:
    ldrb    w9, [x19, #1]              ; decode_a!(a) — byte at PC+1 → w9 (a)
    ldr     x10, [x20, x9, lsl #3]     ; load_reg!(a => 10) — x10 = REGS[a]
    str     x10, [x20, #24]            ; store_local_fixed!(10, 3) — REGS[3] := x10 (#3 * 8 = 24)
    add     x19, x19, #2               ; dispatch!() — advance PC by length=2
    ldrb    w8, [x19]                  ; dispatch!() — next opcode byte
    ldr     x16, [x23, x8, lsl #3]     ; dispatch!() — look up next handler
    br      x16                        ; dispatch!() — tail-jump
```

**7 instructions** total. Within the ≤12 budget.

## Slow path

**Deleted.** `op_store_local_3_slow_rs` had no callers after this
port landed.

## Microbench

7-sample median: **45.96 ns/dispatch ± 0.02** (4 ops/iter). Gate
verdict: **✅** within 2× LLInt reference.

## V8 v7 slow-path-share

**0.000%** (3-sample run, 6 workloads). Aggregate **101,644,452
dispatches** with **0** slow-path entries.

| Workload     | Dispatches  | Slow-path Share |
|--------------|------------:|----------------:|
| Richards     |   6,630,369 |          0.000% |
| DeltaBlue    |   7,147,668 |          0.000% |
| Crypto       |  85,956,261 |          0.000% |
| RayTrace     |   1,615,035 |          0.000% |
| NavierStokes |         306 |          0.000% |
| Splay        |     294,813 |          0.000% |

Crypto dominates (85% of aggregate share). Gate satisfied with
maximum headroom.

## Behavioral tests

vm 418 / lyng-js-tests 1209. Integration test
`store_local_3_updates_param_via_assignment` in
`crates/lyng-js/tests/src/op_locals_inline.rs` covers the
`d = a + b + c + d` reassignment pattern that lands at slot 3.

## Notes

- **The umbrella's top-30 StoreLocalN anchor.** All four StoreLocal
  handlers share the same `store_local_fixed!(10, N)` shape; this is
  the only one in the V8 v7 top-30 by dispatch share.
- The inline port eliminates ~101M slow-path round-trips per V8 v7
  run (3 samples). Combined with the LoadLocal0-3 inline ports
  (1.06B+ dispatches), Phase 1.B.3 removes ~1.16B slow-path entries
  from the V8 v7 hot path.
