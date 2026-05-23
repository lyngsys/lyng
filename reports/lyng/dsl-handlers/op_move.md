# `op_move` DSL port (B39)

First real DSL handler — validates the proc-macro lowerer's operand-decode
prologue + scratch-register substitution + named-binding emission against
a minimal opcode.

## DSL source

`crates/lyng/vm/src/dsl/handlers/hot.rs`:

```rust
llint_handler! {
    op_move, layout = Ab, length = 3, |dst, src| {
        load_reg!(src => t0);
        store_reg!(dst, t0);
        dispatch!();
    }
}
```

The lowerer substitutes operand idents `dst` -> 9, `src` -> 10 and
internal scratch `t0` -> 11 before splicing into `naked_asm!`.

## Current asm (AArch64)

See `reports/lyng/dsl-asm-baseline-aarch64/op_move.asm`.

Effective instruction sequence (after assembler strips comments):

```text
ldrb w9, [x19, #1]              ; decode dst index
ldrb w10, [x19, #2]             ; decode src index
ldr  x11, [x20, x10, lsl #3]    ; load REGS[src]
str  x11, [x20, x9, lsl #3]     ; store REGS[dst]
add  x19, x19, #3               ; advance PC
ldrb w8, [x19]                  ; load next opcode
ldr  x9, [x23, x8, lsl #3]      ; load next handler
br   x9                         ; tail-jump
```

8 instructions total. No spills, no slow-path bridge — the asm shape
matches the design's "hot move ≈ 4 ops + 4-instr dispatch tail" target.

## LLInt reference

See `reports/lyng/llint-reference/op_mov.md`.

JSC's `op_mov` body:

```text
get(m_src, t1)
loadConstantOrVariable(size, t1, t2)
return(t2)   # store to m_dst + dispatch
```

JSC emits load-from-register-or-constant-table (one cmovne/branch for
the constant-table check) plus the return macro (which writes the
destination + dispatches). Lyng's DSL build is slightly tighter because:

1. Lyng has no constant table for move — the source is always a
   register, so the constant-vs-register branch from JSC is elided.
2. Lyng's dispatch is 4 instructions (advance, load opcode, table
   lookup, branch); JSC's is similar but uses a different scratch
   register convention.

Within a 1-2 instruction tolerance of LLInt — meets the design's
"hot opcode ≈ LLInt size" goal.

## Side-by-side diff

| Step          | DSL (lyng)                        | LLInt (JSC)                          |
| ------------- | ------------------------------------ | ------------------------------------ |
| Decode src    | `ldrb w10, [x19, #2]`                | (inline in `get(m_src, t1)`)         |
| Decode dst    | `ldrb w9,  [x19, #1]`                | (inline in `m_dst` decode)           |
| Read source   | `ldr x11, [x20, x10, lsl #3]`        | `loadConstantOrVariable(size, t1, t2)` (multi-instr, includes const-table check) |
| Write dest    | `str x11, [x20, x9,  lsl #3]`        | `return(t2)` (writes + falls into dispatch) |
| Advance PC    | `add x19, x19, #3`                   | implicit in `dispatch()`             |
| Load opcode   | `ldrb w8, [x19]`                     | implicit in `dispatch()`             |
| Look up addr  | `ldr x9, [x23, x8, lsl #3]`          | implicit in `dispatch()`             |
| Tail-jump     | `br x9`                              | implicit in `dispatch()`             |

## Microbench

Not yet captured. The handler is dead code from a runtime perspective
in DSL-0b (the alpha dispatch is still active); microbenches need the
Phase-C dispatch flip to be meaningful. Tracked at DSL-0c.

## Behavioral tests

- `tests/dsl_validation_empty.rs` continues to pass (the load-bearing
  proc-macro integration test from B30 isn't broken by the new
  lowerer behavior).
- Lyng's existing `op_move` semantic tests in `crates/lyng/vm/tests/`
  continue to pass — the alpha dispatch is still active.

## Lowerer changes that landed alongside this port

To get `op_move` to compile correctly the proc-macro lowerer
(`crates/lyng/vm-dsl/src/`) gained:

1. **Operand-decode prologue emission.** `Layout::decode_prologue_tokens`
   now emits `decode_a!(...)`, `decode_ab!(...)`, ..., `decode_ax!(...)`
   token-tree macro invocations as the first template entry in
   `naked_asm!`. Replaces the placeholder `// decode_<layout>` string
   that Batch 1 emitted.
2. **Scratch-register substitution.** `lower::substitute_idents`
   rewrites every operand binding (`dst`, `src`, ...) and every
   reserved internal scratch ident (`t0..t6`) into a literal register
   number (`9..15`) before splicing the body into `naked_asm!`. The
   backend macros' `stringify!($idx)` calls then produce real
   register names like `w9` / `x10`.
3. **Standard named bindings.** Every emitted handler now carries
   `length`, `state_pc`, `state_pb`, `state_regs`, `state_fv`,
   `state_prefix`, `vm_poll`, `entry_stride_shift`, `entry_observed`,
   and `exit` named args, plus one `<name> = sym <name>` per
   `call_slow!(...)` reference discovered in the body.
4. **Backend macro args switched from `:ident` to `:tt`.** The
   `decode_*!` / `load_reg!` / `store_reg!` / `load_acc!` / `store_acc!`
   macros now accept literal register-number args, so the lowerer's
   substitution pass can feed them numbers.
5. **`Layout::operand_arity()` fix.** `Ax` (extended u32 jump-target
   operand) was reporting 3 — corrected to 1 (the single u32 operand).
   Was masked previously because no real handler used the Ax form.

The `extern crate self as lyng_vm;` declaration in `lyng-vm`'s
`lib.rs` lets the proc-macro emit `::lyng_vm::...` absolute paths
that resolve from both inside the crate and external test crates.
