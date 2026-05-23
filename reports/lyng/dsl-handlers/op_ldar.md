# `op_ldar` DSL port (Phase 1.B.3, Task 3)

"Load Accumulator from Register" — copies `registers[a]` into the
accumulator (`registers[0]`). Emitted by the bytecode-builder
peephole when a `Move dst=0, src=B` is produced (see
`crates/bytecode/src/builder.rs:159-161`). V8 v7 aggregate
dispatches: **89,313,894** (3 samples × 6 workloads).

## DSL source

`crates/vm/src/dsl/handlers/cold.rs`:

```rust
llint_handler! {
    op_ldar_dsl, opcode_byte = 130, layout = A, length = 2, |a| {
        load_reg!(a => 10);
        store_acc!(10);
        dispatch!();
    }
}
```

- `a` (byte 1): source register id.
- Destination is slot 0 (accumulator) — written via `store_acc!`.
- No slow path: pure register-to-register move.

## Current asm (AArch64)

See `reports/lyng/dsl-asm-baseline-aarch64/op_ldar.asm`.

```asm
op_ldar_dsl:
    ldrb    w9, [x19, #1]              ; decode_a!(a) — byte at PC+1 → w9 (a)
    ldr     x10, [x20, x9, lsl #3]     ; load_reg!(a => 10) — x10 = REGS[a]
    str     x10, [x20]                 ; store_acc!(10) — REGS[0] := x10 (slot 0 = acc)
    add     x19, x19, #2               ; dispatch!() — advance PC by length=2
    ldrb    w8, [x19]                  ; dispatch!() — next opcode byte
    ldr     x16, [x23, x8, lsl #3]     ; dispatch!() — look up next handler
    br      x16                        ; dispatch!() — tail-jump
```

**7 instructions** total. Within the ≤12 budget. Uses existing Phase
1.A `load_reg!` and `store_acc!` macros — no new substrate.

## Slow path

**Deleted.** `op_ldar_slow_rs` had no callers after this port landed.
Semantic body at `crates/vm/src/vm/semantics/loads.rs:322-333`
is `registers[0] = registers[args.a]` with no bail conditions.

## LLInt reference

Structural baseline at
`reports/lyng/dsl-asm-baseline-aarch64/Ldar.asm` shows the
LLInt path: function call into `op_ldar` with stack-frame setup +
bounds check + indexed load + store + Star-fusion peephole check
(JSC LLInt fuses Ldar+Star into a single Move at runtime; lyng's
peephole runs at compile time so this isn't relevant here). ~50
instructions including the Star-fusion fast path. The DSL inline form
skips all framework overhead — 7 instructions inline.

## Microbench

7-sample median (run 2026-05-20, HEAD post-Task 3):

| Median ns/dispatch | CI95 half-width | Snippet ratio |
|-------------------:|----------------:|--------------:|
| 37.56              | ±0.04           | 4 ops/iter    |

LdarN snippet uses 4 successive `p0 = v` writes per iter (which the
peephole rewrites to Ldar via the `Move dst=0` branch). Ldar's
ns/dispatch (~37.6 ns) sits comfortably between LoadLocal0 (~29 ns)
and the StoreLocalN family (~46 ns), reflecting the symmetric
1-decode + 2-body + 4-dispatch shape.

Gate verdict: **✅** within 2× LLInt reference. >2× headroom.

## V8 v7 slow-path-share

**0.000%** (measured 2026-05-20, 3-sample run). Aggregate
**89,313,894 dispatches** across all 6 V8 v7 workloads with **0
semantic / 0 safepoint** slow-path entries.

| Workload     | Dispatches  | Semantic SP | Safepoint SP | Share   |
|--------------|------------:|------------:|-------------:|--------:|
| Richards     |           0 |           0 |            0 |  0.000% |
| DeltaBlue    |           0 |           0 |            0 |  0.000% |
| Crypto       |  85,956,660 |           0 |            0 |  0.000% |
| RayTrace     |   3,357,018 |           0 |            0 |  0.000% |
| NavierStokes |         216 |           0 |            0 |  0.000% |
| Splay        |           0 |           0 |            0 |  0.000% |

Crypto dominates (96% of aggregate Ldar share). Gate (< 20%) satisfied.

## Behavioral tests

- `cargo test -p lyng-vm --lib --release` — **418 passed**.
- `cargo test -p lyng-tests --release` — **1209 passed**.

Integration tests in `crates/tests/src/op_ldar_inline.rs`
cover:
1. `ldar_via_intermediate_temporary` — `(a + b) * 2`: the temp is
   Ldar'd into the accumulator before the multiply.
2. `ldar_in_chained_arithmetic` — multi-step `x + c; y * 10` chain.
3. `ldar_with_function_call_result` — function-call result Ldar'd
   into accumulator before subsequent op.

The same tests passed pre-port (with the cold stub) and post-port
(with the inline body), demonstrating semantic parity.

## Notes

- **No new substrate.** Re-uses Phase 1.A's `load_reg!` and
  `store_acc!` macros. The new Phase 1.B.3 `load_local_fixed!` /
  `store_local_fixed!` macros are NOT used here — Ldar's destination
  is the accumulator (slot 0), which has its own dedicated `store_acc!`
  emitter.
- **High-share opcode.** ~89M dispatches per V8 v7 run (aggregate
  across 3 samples). The inline port saves a slow-path round-trip on
  every dispatch — multiplied across all V8 v7 workloads, the
  same-load A/B should show measurable improvement on Crypto.
