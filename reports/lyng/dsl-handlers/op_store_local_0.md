# `op_store_local_0` DSL port (Phase 1.B.3, Task 3)

Inline write of source-register value into slot 0 (accumulator).
Macro-shared symmetric pair of `op_store_local_3`; qualifies under
the umbrella's 15-min rule.

## Finding: opcode is functionally unreachable

**V8 v7 aggregate dispatches (3 samples × 6 workloads): 0.** This is
not a bug — it is the expected behavior of the bytecode-builder
peephole.

The peephole at
`crates/lyng/bytecode/src/builder.rs:150-166`
(`compact_move_instruction`) evaluates the conditions in this order:

```rust
// Line 154-158: Move(dst=N, src=0) → StarN (read from accumulator).
if operands.b() == 0 && let Some(opcode) = accumulator_store_opcode(...) { ... }

// Line 159-161: Move(dst=0, src=B) → Ldar B (load accumulator from reg B).
if operands.a() == 0 && u8::try_from(operands.b()).is_ok() { ... }

// Line 162-164: Otherwise, if dst in [0..3], emit StoreLocalN.
if let Some(opcode) = store_local_opcode(operands.a()) { ... }
```

Slot 0 dst always lands on the `Ldar` branch (line 159) before
reaching the `store_local_opcode` branch (line 162). The handler
remains live in the dispatch table for completeness (a hand-crafted
bytecode stream could trigger it), but the emit pipeline never
produces it.

## DSL source

`crates/lyng/vm/src/dsl/handlers/cold.rs`:

```rust
llint_handler! {
    op_store_local_0_dsl, opcode_byte = 148, layout = A, length = 2, |a| {
        load_reg!(a => 10);
        store_local_fixed!(10, 0);
        dispatch!();
    }
}
```

- `a` (byte 1): source register id.
- Destination slot = literal `0`.
- No slow path.

## Current asm (AArch64)

See `reports/lyng/dsl-asm-baseline-aarch64/op_store_local_0.asm`.

```asm
op_store_local_0_dsl:
    ldrb    w9, [x19, #1]              ; decode_a!(a) — byte at PC+1 → w9 (a)
    ldr     x10, [x20, x9, lsl #3]     ; load_reg!(a => 10) — x10 = REGS[a]
    str     x10, [x20]                 ; store_local_fixed!(10, 0) — REGS[0] := x10 (#0 * 8 = 0)
    add     x19, x19, #2               ; dispatch!() — advance PC by length=2
    ldrb    w8, [x19]                  ; dispatch!() — next opcode byte
    ldr     x16, [x23, x8, lsl #3]     ; dispatch!() — look up next handler
    br      x16                        ; dispatch!() — tail-jump
```

**7 instructions** total. Same shape as StoreLocal3 with slot offset
= 0; `store_local_fixed!(10, 0)` collapses to `str x10, [x20]`
(LLVM/the assembler accepts the `#0 * 8` form and emits the
zero-offset form). Within the ≤12 budget.

## Slow path

**Deleted.** `op_store_local_0_slow_rs` had no callers after this
port landed (no callers existed even before, given the dispatch-zero
finding above).

## Microbench

**No measurement** — the opcode is unreachable through the standard
emit pipeline, so no representative snippet can be constructed. The
`StoreLocal0` snippet was attempted (writing `p0 = v` in a loop) but
the peephole rewrote every store to `Ldar`, yielding zero StoreLocal0
dispatches in the test. See `tools/lyng-bench/src/microbench/
snippets.rs` documentation and the `verify_opcodes_per_iter` test for
the explicit omission rationale.

The inline body is the same shape as StoreLocal3 / StoreLocal1 /
StoreLocal2; their measurements (46.0 ns ± 0.1) apply by analogy.

## V8 v7 slow-path-share

**0.000%** (trivially — 0 dispatches over 3 samples × 6 workloads).
Slow-path-share gate (< 20%) is satisfied with maximum headroom.

## Behavioral tests

vm 418 / lyng-tests 1209. The `op_locals_inline.rs::
store_local_0_1_2_via_assignments` test writes parameters via `a = a
* 2;` style assignments but — due to the peephole behavior described
above — the compiled bytecode dispatches `Ldar` rather than
`StoreLocal0` for the first parameter. The TEST PASSES (the assigned
value is preserved through Ldar's `registers[0] := registers[src]`
semantics), confirming the end-to-end semantic is correct regardless
of which opcode the compiler chose.

## Notes

- **Dead-code conservation rationale.** The handler is retained
  (not deleted as truly-dead code) because:
  1. The opcode constant `Opcode::StoreLocal0 = 148` is registered
     in the dispatch table; removing the handler would require a
     coordinated removal of the opcode constant, which is out of
     scope for Phase 1.B.3.
  2. A future peephole change or hand-crafted bytecode could
     legitimately dispatch this opcode; the inline body is correct
     and cheap.
  3. The macro-shared `store_local_fixed!(10, N)` form is symmetric
     across N in 0..3; including StoreLocal0 keeps the source
     tree consistent.
- **A potential follow-up** (out of scope for 1.B.3): consider
  deleting the `Opcode::StoreLocal0` entirely if a future audit
  confirms it's unreachable across all emit paths (not just the
  peephole).
