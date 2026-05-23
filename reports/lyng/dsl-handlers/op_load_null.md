# `op_load_null` DSL port (Phase 1.A, Task 2)

Second Phase-1.A port — mirrors Task 1 (`op_load_undefined`) exactly,
swapping only the tag-payload immediate. The handler writes
`Value::null()` to register `a`; the `bx` operand is unused (layout
reserves the slot for forward compat).

## DSL source

`crates/lyng-js/vm/src/dsl/handlers/cold.rs`:

```rust
llint_handler! {
    op_load_null_dsl, layout = Abx, length = 4, |a, _bx| {
        tag_null!(t0);
        store_reg!(a, t0);
        dispatch!();
    }
}
```

## Current asm (AArch64)

See `reports/js/lyng-js/dsl-asm-baseline-aarch64/op_load_null.asm`.

Captured from `target/release/deps/lyng_js_vm-*.s` after a
`cargo rustc --release -p lyng-js-vm --lib -- --emit=asm -C debuginfo=0`
build. Effective sequence:

```asm
op_load_null_dsl:
    ldrb    w9,  [x19, #1]              ; decode a (byte at PC+1)
    ldrh    w10, [x19, #2]              ; decode _bx (unused; LLVM kept the load)
    mov     x11, #8589934592            ; tag_null!(t0) — movz x11, #0x2, lsl #32
    movk    x11, #32760, lsl #48        ; tag_null!(t0) — movk x11, #0x7ff8, lsl #48
    str     x11, [x20, x9, lsl #3]      ; store_reg!(a, t0) — REGS[a] := null
    add     x19, x19, #4                ; dispatch!() — advance PC by length=4
    ldrb    w8,  [x19]                  ; dispatch!() — load next opcode byte
    ldr     x16, [x23, x8, lsl #3]      ; dispatch!() — look up next handler
    br      x16                         ; dispatch!() — tail-jump
```

**9 instructions** total. Per design budget — matches the expected pattern,
identical in shape to `op_load_undefined` save the tag-payload immediate.
LLVM did not elide the unused `ldrh` decode for `_bx`; the encoding is
small (4 bytes) and proving the load is dead requires alias analysis on
the bytecode buffer that LLVM doesn't perform here.

Note: the LLVM assembler rewrote the canonical `tag_null!` shifted-
immediate form
(`movz x11, #0x2, lsl #32 ; movk x11, #0x7ff8, lsl #48`) to the
mathematically-equivalent `mov x11, #8589934592 ; movk x11, #32760, lsl #48`
(where 8589934592 = 0x2_0000_0000 and 32760 = 0x7ff8). The encoding is
two instructions in both cases — no observable difference at runtime.

## LLInt reference

`capture-llint` in `auto` mode could not locate the LLInt symbol
(`_llint_op_mov` not found in the system JSC — binary stripped); in
`excerpt` mode `op_mov` did not match the offlineasm bare-symbol
heuristic. LLInt reference taken directly from the offlineasm source at
`/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter64.asm`.

The closest LLInt analog to "write `null` to a register" is the
`move ValueNull, t0` + `storeq t0, [...]` idiom used inline
within several handlers. The closest *opcode* analog is `op_mov` at
line 906:

```
llintOpWithReturn(op_mov, OpMov, macro (size, get, dispatch, return)
    get(m_src, t1)
    loadConstantOrVariable(size, t1, t2)
    return(t2)
end)
```

JSC's `op_mov` is more general — it moves a constant-table entry or
register into the destination, going through `loadConstantOrVariable`
(which dispatches on the operand encoding). The `null` value lives
in the constant table for most JSC bytecode streams, so the equivalent
sequence is "decode src operand, load constant table[src], store to
dest, dispatch". Lyng's `op_load_null` short-circuits the constant-
table indirection by materializing `Value::null()` directly with a
2-instruction `movz/movk` sequence — no memory access for the value.

A more directly-comparable LLInt fragment is the `move ValueNull,
t0; storeq t0, [...]` pattern used at LowLevelInterpreter64.asm —
that idiom is two instructions to materialize + one to store, matching
lyng's tag-and-store fast path exactly.

LLInt reference capture mode: **excerpt (manual)** — `capture-llint`
binary path failed; reference taken from local WebKit checkout at
`/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter64.asm`.

## Side-by-side diff

| Step | Lyng DSL                                         | LLInt (`op_mov`-via-null-constant)                  |
|------|--------------------------------------------------|-----------------------------------------------------|
| 1    | `ldrb w9, [x19, #1]` — decode `a`                | `get(m_dst, t0)` — decode dest reg                  |
| 2    | `ldrh w10, [x19, #2]` — decode `_bx` (unused)    | `get(m_src, t1)` — decode src constant-pool idx     |
| 3    | `movz x11, #0x2, lsl #32` (tag_null low)         | `loadConstantOrVariable(size, t1, t2)` — table load |
| 4    | `movk x11, #0x7ff8, lsl #48` (tag_null high)     | (rolled into step 3 as a single 64-bit `loadq`)     |
| 5    | `str x11, [x20, x9, lsl #3]` — store_reg!        | `return(t2)` → `storeq t2, [cfr, dst, 8]`           |
| 6-9  | `add/ldrb/ldr/br` — dispatch tail (4 instr)      | `dispatch()` — equivalent 4-instr tail              |

**Irreducible deltas vs LLInt:**

- **Value layout.** Lyng uses NaN-tagged 64-bit `Value` (high 16 bits = tag,
  low 32 bits = payload, canonical-NaN prefix in bits 48-60). `null`
  is the constant `0x7ff8_0002_0000_0000` (payload byte `#0x2`),
  requiring a 2-instruction `movz/movk` to materialize. JSC's JSValue
  is also NaN-boxed in 64-bit mode but uses `ValueNull` as a single-
  instruction constant move on x86_64 (`movabsq $imm, t0`) and a similar
  2-instruction `movz/movk` on AArch64 — parity, not a delta.
- **No constant-table indirection.** Lyng has a dedicated `LoadNull`
  opcode so the value is in-line in the handler. JSC reuses `op_mov`
  and reads from the constant table, adding a `loadq` per dispatch. This
  is a strict win for lyng on `undefined`/`null`/`true`/`false`/`zero`/
  `one` — the canonical Phase 1.A constant-loaders.
- **Unused `bx` decode.** Lyng's `Abx` layout reads a 16-bit operand at
  `[PC+2]` even though `op_load_null` doesn't use it. This `ldrh`
  is the one un-LLInt-like instruction; it costs ~1 cycle and survives
  because LLVM can't prove the load is dead in a naked handler. A
  future codegen tweak could narrow the layout to `A` (length=2) and
  drop the unused operand entirely, saving 1 instruction + 2 bytes per
  call site. Out of scope for Phase 1.A.

## Microbench

Microbench snippet not yet present for `LoadNull`; deferred to
Task 10.B. The pre-phase baseline at
`reports/js/lyng-js/dsl-1/pre-phase-1a-baseline.md` confirms this for
all nine Phase 1.A opcodes.

## V8 v7

A single-opcode port is not expected to move the V8 v7 geomean
measurably — the LoadNull dispatch share is sub-percent on the
V8 v7 suite. Phase 1.A's aggregate impact will be measured at Task 10
against the pre-phase baseline geomean of **387.09** captured in
`reports/js/lyng-js/dsl-1/pre-phase-1a-baseline.md`.

## Slow-path-share

Slow-path-share counter is currently no-op (DSL-0c gap; tracked as Task
10.A to re-wire into DSL dispatch tail). Once re-wired, expected post-
port value: **0%** — the slow-path shim `op_load_null_slow_rs`
was deleted alongside this port; the opcode has no fail mode (it
unconditionally writes `Value::null()` to register `a` and
dispatches).

## Behavioral tests

- `cargo test -p lyng-js-vm --lib --release` — **413 passed**.
- `cargo test -p lyng-js-tests --release` — **1186 passed (2 suites)**.

Both green; behavioral parity preserved.

## Notes

- **Unused `bx` decode (`ldrh w10, [x19, #2]`).** LLVM did *not* elide
  the unused 16-bit load — the proc-macro lowerer's prologue emits
  `decode_abx!(a, _bx)` which expands unconditionally; LLVM has no
  visibility into the live-out set inside the naked-asm! block.
  Acceptable for Phase 1.A: 1 extra instruction, ~1 cycle. A future
  micro-optimization could re-shape the layout to drop the operand,
  but it requires bytecode-format coordination (opcode-length changes
  break ahead-of-time encoded streams) and is out of scope.
- **Slow-path shim deleted.** `op_load_null_slow_rs` was the
  cold-stub bridge to `op_load_null_semantic`; with the inline
  fast path it has no callers (grep confirmed only self-references)
  and was removed alongside the handler-body change.
- **Manifest entry unchanged.** The opcode manifest references
  `op_load_null_dsl` (the handler symbol, not the shim). The symbol
  identity is preserved; only the body changed.
- **Identical shape to `op_load_undefined`.** Only the `movz` immediate
  differs: `#0x2` (null payload) vs `#0x1` (undefined payload). Same
  9-instruction sequence; same dispatch tail; same unused-bx decode.
  This regularity is by design — the Phase 1.A constant-loaders share
  a body template (`tag_X!(t0); store_reg!(a, t0); dispatch!()`).
- **Pre-existing infra gaps acknowledged (not fixed by this task):**
  1. Slow-path-share counter is no-op since DSL-0c (Task 10.A).
  2. Microbench snippets for the 9 Phase-1.A opcodes don't exist
     (Task 10.B).
- **LLInt reference capture mode:** *excerpt (manual)* — neither
  `system` nor `local` nor `excerpt` automated modes located `op_mov`
  in the system JSC binary or offlineasm sources; manual inspection
  of `/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter64.asm`
  yielded the reference fragments.
