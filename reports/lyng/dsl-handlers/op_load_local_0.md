# `op_load_local_0` DSL port (Phase 1.B.3, Task 2)

Inline read of slot 0 (accumulator) into the destination register
identified by operand byte `a`. Top-30 dispatch share for the
LoadLocal0 anchor; V8 v7 aggregate (3 samples × 6 workloads) =
**268,151,144 dispatches**.

## DSL source

`crates/lyng/vm/src/dsl/handlers/cold.rs`:

```rust
llint_handler! {
    op_load_local_0_dsl, opcode_byte = 144, layout = A, length = 2, |a| {
        load_acc!(10);
        store_reg!(a, 10);
        dispatch!();
    }
}
```

- `a` (byte 1): destination register id.
- No slow path: pure register-window move. Inline path handles 100%
  of dispatches.

## Current asm (AArch64)

See `reports/lyng/dsl-asm-baseline-aarch64/op_load_local_0.asm`.

```asm
op_load_local_0_dsl:
    ldrb    w9, [x19, #1]              ; decode_a!(a) — byte at PC+1 → w9 (a)
    ldr     x10, [x20]                 ; load_acc!(10) — x10 = REGS[0]
    str     x10, [x20, x9, lsl #3]     ; store_reg!(a, 10) — REGS[a] := x10
    add     x19, x19, #2               ; dispatch!() — advance PC by length=2
    ldrb    w8, [x19]                  ; dispatch!() — next opcode byte
    ldr     x16, [x23, x8, lsl #3]     ; dispatch!() — look up next handler
    br      x16                        ; dispatch!() — tail-jump
```

**7 instructions** total. Well within the ≤12 inline budget. The
`load_acc!` macro re-uses the existing slot-0 reader added in Phase
1.A; no new substrate.

## Slow path

**Deleted.** `op_load_local_0_slow_rs` had no callers after this port
landed (verified via grep across `crates/lyng/`). The semantic body
in `crates/lyng/vm/src/vm/semantics/loads.rs:436-448` is `dst =
registers[0]` with no bail conditions. No runtime failure mode exists.

## LLInt reference

The structural baseline at
`reports/lyng/dsl-asm-baseline-aarch64/LoadLocal0.asm`
shows the LLInt path: a function call into `op_load_local_0` with
stack-frame setup + bounds check + indexed load + store, then a
dispatch jump. ~33 instructions including the function prologue / 
epilogue and the slow-path bail target. The DSL inline form skips
all of this — single `ldr` from slot 0 plus `store_reg!` and dispatch.

## Microbench

7-sample median (run 2026-05-20, HEAD post-Task 3):

| Median ns/dispatch | CI95 half-width | Snippet ratio |
|-------------------:|----------------:|--------------:|
| 28.94              | ±0.02           | 5 ops/iter    |

Compared against LLInt-style hot-path constant loaders (LoadZero =
35.25 ns, LoadNull = 35.25 ns), LoadLocal0 is actually **faster than
the constant-loaders** at ~29 ns. The reason: `load_acc!(10)` is a
single `ldr` with no immediate-offset arithmetic (offset = 0); the
`movz` + `or` instruction overhead of the LoadZero / LoadNull
constant-materialization isn't present. The 5-ops-per-iter ratio
includes the loop's `i < iters` test (which is also LoadLocal0
because `iters` sits at slot 0).

Gate verdict: **✅** comfortably within 2× LLInt reference (29 ns vs
~60-80 ns budget; >2× headroom).

## V8 v7 slow-path-share

**0.000%** (measured 2026-05-20, 3-sample run via
`v8suite --count-opcodes --count-slow-path-share`). Across all 6
V8 v7 workloads, op_load_local_0 dispatched **268,151,144 times** and
recorded **0 semantic / 0 safepoint** slow-path entries.

Per-workload breakdown:

| Workload     | Dispatches  | Semantic SP | Safepoint SP | Share   |
|--------------|------------:|------------:|-------------:|--------:|
| Richards     |  10,882,758 |           0 |            0 |  0.000% |
| DeltaBlue    |   2,092,272 |           0 |            0 |  0.000% |
| Crypto       | 185,313,540 |           0 |            0 |  0.000% |
| RayTrace     |  35,682,054 |           0 |            0 |  0.000% |
| NavierStokes |  19,713,534 |           0 |            0 |  0.000% |
| Splay        |  14,466,986 |           0 |            0 |  0.000% |

Gate (< 20%) satisfied with maximum headroom.

## Behavioral tests

- `cargo test -p lyng-vm --lib --release` — **418 passed**.
- `cargo test -p lyng-tests --release` — **1209 passed** (1198
  baseline + 8 new `op_locals_inline.rs` + 3 new `op_ldar_inline.rs`
  integration tests).

Integration tests in `crates/lyng/tests/src/op_locals_inline.rs`
cover:
1. `load_local_0_returns_first_parameter` — direct first-param read.
5. `load_locals_aggregate` — `a + b + c + d` exercises 0..3 together.
8. `locals_in_tight_loop_sum` — 100-iter loop with parameter reads.

The same tests passed pre-port (with the cold stub) and post-port
(with the inline body), demonstrating semantic parity.

## Notes

- **Slow-path shim deleted.** `op_load_local_0_slow_rs` had no
  callers after this port landed and was removed alongside the
  handler-body change. Grep across `crates/lyng/` confirms no
  remaining references.
- **No new substrate.** Re-uses Phase 1.A's `load_acc!` macro from
  `crates/lyng/vm/src/dsl/backend/aarch64/operands.rs:126`. The
  new Phase 1.B.3 `load_local_fixed!` macro is NOT used here (slot 0
  has its own dedicated `load_acc!` macro).
