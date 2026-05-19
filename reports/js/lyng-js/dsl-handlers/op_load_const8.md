# `op_load_const8` DSL port (Phase 1.B.2, Task 2)

First Phase-1.B.2 inline port. Reads from the pre-resolved constants
array via `LlIntState::frame_const_base` (substrate established by
Phase 1.B.1). Top-30 dispatch share: **#21**, ~104M dispatches per V8 v7
run.

## DSL source

`crates/lyng-js/vm/src/dsl/handlers/cold.rs`:

```rust
llint_handler! {
    op_load_const8_dsl, opcode_byte = 140, layout = Ab, length = 3, |a, b| {
        load_constant!(b => 10);
        store_reg!(a, 10);
        dispatch!();
    }
}
```

- `a` (byte 1): destination register id.
- `b` (byte 2): constant-pool index (u8).
- No slow path: the inline body covers every `ConstantValue` variant
  the install-time pre-resolution (`Vm::install_constants`)
  materializes into the flat `Value` array.

## Current asm (AArch64)

See `reports/js/lyng-js/dsl-asm-baseline-aarch64/op_load_const8.asm`.

Captured from `target/release/deps/lyng_js_vm-*.s` after a
`cargo rustc --release -p lyng-js-vm --lib -- --emit=asm -C debuginfo=0`
build. Effective sequence:

```asm
op_load_const8_dsl:
    ldrb    w9,  [x19, #1]              ; decode a (byte at PC+1)
    ldrb    w10, [x19, #2]              ; decode b (byte at PC+2 — const pool index)
    ldr     x16, [x24, #32]             ; load_constant!(b => 10) — x16 = frame_const_base
    ldr     x10, [x16, x10, lsl #3]     ; load_constant!(b => 10) — x10 = frame_const_base[b]
    str     x10, [x20, x9, lsl #3]      ; store_reg!(a, 10) — REGS[a] := loaded Value
    add     x19, x19, #3                ; dispatch!() — advance PC by length=3
    ldrb    w8,  [x19]                  ; dispatch!() — load next opcode byte
    ldr     x16, [x23, x8, lsl #3]      ; dispatch!() — look up next handler
    br      x16                         ; dispatch!() — tail-jump
```

**9 instructions** total. Well within the ≤12 budget. The `#32` literal
is `offset_of!(LlIntState, frame_const_base)` (pinned by the
`LLINT_STATE_FRAME_CONST_BASE` const). x24 is the STATE pin per the
asm-DSL register convention (see `crates/lyng-js/vm/src/dsl/reg_convention.rs`).

LLVM did not rewrite the `load_constant!` body — the canonical 2-instr
indexed-load shape (`ldr base; ldr value[idx]`) appears verbatim.

## Slow path

**Deleted.** `op_load_const8_slow_rs` had no callers after this port
landed; the inline path handles all cases. The opcode's failure mode
is "constant pool index out of range" — an internal compiler-emit bug
that's caught at install time by `Vm::install_constants` (the flat
constants array has a known size, and the bytecode validator checks
operand bounds). No runtime bail condition.

## LLInt reference

The closest LLInt analog is `op_mov` (which goes through
`loadConstantOrVariable`), or any of the constant-loading idioms in
JSC's LLInt that read from the constant pool. The shape is:

```text
get(idx) → decode constant-pool operand
loadq [constant_pool + idx, lsl 3] → load Value
storeq → write to dest register
dispatch()
```

JSC's LLInt achieves the same 2-instruction constant-pool read shape
(once the pre-resolved constants array is in a known location). Lyng's
inline body matches this exactly — base pointer cached in a per-frame
LlIntState mirror, indexed load, store, dispatch.

LLInt reference capture mode: **excerpt (manual)** — JSC's
`op_mov` lowering at
`/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter64.asm`
line 906. Same 2-instruction indexed-load shape after the per-frame
constant-pool base pointer is materialized.

## Side-by-side diff

| Step | Lyng DSL                                          | LLInt (`op_mov`-via-constant-pool)                  |
|------|---------------------------------------------------|------------------------------------------------------|
| 1    | `ldrb w9, [x19, #1]` — decode `a`                 | `get(m_dst, t0)` — decode dest reg                  |
| 2    | `ldrb w10, [x19, #2]` — decode `b` (pool idx)     | `get(m_src, t1)` — decode src constant-pool idx     |
| 3    | `ldr x16, [x24, #32]` — load frame_const_base     | `loadConstantOrVariable` — load constant-pool base  |
| 4    | `ldr x10, [x16, x10, lsl #3]` — indexed Value     | `loadq [cp + t1, lsl 3], t2` — same shape           |
| 5    | `str x10, [x20, x9, lsl #3]` — store to reg       | `storeq t2, [cfr, dst, 8]`                          |
| 6-9  | `add/ldrb/ldr/br` — dispatch tail (4 instr)       | `dispatch()` — equivalent 4-instr tail              |

**Irreducible deltas vs LLInt:**

- **Per-frame mirror.** Lyng's `frame_const_base` lives on `LlIntState`
  (the per-call state struct read through pinned x24). JSC's LLInt
  keeps the constant-pool base in `cfr` (call-frame register, x29) and
  reads it via a fixed `cfr + CodeBlock + constantsVectorOffset`
  sequence. Both are 1 load per access — parity.
- **No tag-check after load.** The pre-resolved constants array
  contains tagged `Value` u64s; the dispatch path reads them
  bit-for-bit without re-tagging. JSC's LLInt is the same.
- **No bounds check.** Bytecode validator guarantees `b < pool_size`
  at install time, so the inline path doesn't re-check. JSC's LLInt
  also relies on AOT validation.

## Microbench

`LoadConst8` microbench snippet present (added in Phase 1.B.0 Task 7).
ns/dispatch result: **TBD-Task-4** — Task 4 of Phase 1.B.2 runs the
microbench suite + slow-path-share counter, and fills in the number
here.

Expected: significantly lower than the cold-stub call-slow shim (4
instructions inline vs 7+ call-shim instructions). Target: within 2×
LLInt reference.

## V8 v7

A single-opcode port is not expected to move the V8 v7 geomean
measurably on its own, but combined with op_load_this (also in Phase
1.B.2) the aggregate effect should clear the +0.3% gate. Same-load
A/B comparison vs `68dd5e89` (Phase 1.B.1 close): **TBD-Task-4**.

## Slow-path-share

**TBD-Task-4** (microbench + slow-path-share gate). Expected: **~0%**
on V8 v7. The inline path has no bail condition; the cold-stub call-
slow shim is deleted. The slow-path-share counter for opcode 140
should report 0 dispatches falling through to a Rust-side bail.

## Behavioral tests

- `cargo test -p lyng-js-vm --lib --release` — **418 passed**.
- `cargo test -p lyng-js-tests --release` — **1192 passed** (1187
  baseline + 5 new `op_load_const8_inline.rs` integration tests).

Both green; behavioral parity preserved.

Integration tests cover:
1. Smi constant load (`42` literal).
2. Float64 constant load (`3.14` literal).
3. Atom constant load (`'hello'` literal).
4. Multi-constant pool indexing (3 named bindings, return the third).
5. Negative Smi constant load (`-7`).

## Notes

- **Slow-path shim deleted.** `op_load_const8_slow_rs` was the
  cold-stub bridge to `op_load_const8_semantic`; with the inline fast
  path it has no callers (grep confirmed only the in-source comment
  remains) and was removed alongside the handler-body change.
- **Substrate dependency.** This port consumes the
  `frame_const_base: *const Value` field on `LlIntState` (offset 32,
  populated at trampoline entry by `entry.rs::run_via_dsl`, refreshed
  on every slow-path Refresh egress in
  `slow_path.rs::translate_outcome`). Mirror discipline is verified
  by Phase 1.B.1 Task 7's gc-stress test
  (`gc_stress_frame_context.rs`).
- **x22 → x24 bug fix.** The `load_constant!` macro in
  `aarch64/constants.rs` initially emitted `ldr x16, [x22, ...]` (VM
  pin) but the offset is into `LlIntState` (STATE pin = x24). Bug was
  latent because no real opcode handler dispatched through the macro
  in Phase 1.B.1 — only the structural validation tests (opcode 213
  is never dispatched) compiled it. Fixed alongside this port. Same
  fix applied to `load_state_value!` in `aarch64/frame.rs`.
