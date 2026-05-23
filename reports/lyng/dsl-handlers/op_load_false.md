# `op_load_false` DSL port (Phase 1.A, Task 4)

Fourth Phase-1.A port — identical shape to Task 3 (`op_load_true`) with
payload=0. The handler writes `Value::from_bool(false)` to register `a`;
the `bx` operand is unused (layout reserves the slot for forward compat).

Bool tag carries an explicit payload bit (`#0x1` for `true`, `#0x0` for
`false`), so `tag_bool_const!` produces **3 instructions** rather than
the 2 instructions used by `tag_undefined!`/`tag_null!` (whose payloads
are folded into the same `movz` as their kind tag).

## DSL source

`crates/lyng/vm/src/dsl/handlers/cold.rs`:

```rust
llint_handler! {
    op_load_false_dsl, layout = Abx, length = 4, |a, _bx| {
        tag_bool_const!(t0, 0);
        store_reg!(a, t0);
        dispatch!();
    }
}
```

## Current asm (AArch64)

See `reports/lyng/dsl-asm-baseline-aarch64/op_load_false.asm`.

Captured from `target/release/deps/lyng_vm-*.s` after a
`cargo rustc --release -p lyng-vm --lib -- --emit=asm -C debuginfo=0`
build. Effective sequence:

```asm
op_load_false_dsl:
    ldrb    w9,  [x19, #1]              ; decode a (byte at PC+1)
    ldrh    w10, [x19, #2]              ; decode _bx (unused; LLVM kept the load)
    mov     x11, #0                     ; tag_bool_const!(t0, 0) — movz x11, #0x0 (payload)
    movk    x11, #3, lsl #32            ; tag_bool_const!(t0, 0) — movk x11, #0x3, lsl #32 (Bool kind)
    movk    x11, #32760, lsl #48        ; tag_bool_const!(t0, 0) — movk x11, #0x7ff8, lsl #48
    str     x11, [x20, x9, lsl #3]      ; store_reg!(a, t0) — REGS[a] := false
    add     x19, x19, #4                ; dispatch!() — advance PC by length=4
    ldrb    w8,  [x19]                  ; dispatch!() — load next opcode byte
    ldr     x16, [x23, x8, lsl #3]      ; dispatch!() — look up next handler
    br      x16                         ; dispatch!() — tail-jump
```

**10 instructions** total — identical shape to `op_load_true` (10 instr),
one more than `op_load_undefined`/`op_load_null` (9 each). The +1 comes
from `tag_bool_const!`'s 3-instruction expansion: the Bool tag needs
both an explicit payload (`#0x0` for `false`) and an explicit kind
(`#0x3`) materialized in different bit ranges. For `false` specifically
the payload `movz` writes `#0` — the only difference from `op_load_true`'s
asm is the immediate in that single instruction. LLVM did not elide the
unused `ldrh` decode for `_bx`; the encoding is small (4 bytes) and
proving the load is dead requires alias analysis on the bytecode buffer
that LLVM doesn't perform here.

Note: the LLVM assembler rewrote the canonical `tag_bool_const!` form
(`movz x11, #0x0 ; movk x11, #0x3, lsl #32 ; movk x11, #0x7ff8, lsl #48`)
to the equivalent (`mov x11, #0 ; movk x11, #3, lsl #32 ; movk x11, #32760, lsl #48`).
The first `movz` and the rewritten `mov` produce identical machine encoding;
the `movk` immediates are decimal renderings of the same hex values
(`3 == 0x3`, `32760 == 0x7ff8`). No observable difference at runtime.

## LLInt reference

`capture-llint` in `auto` mode could not locate the LLInt symbol
(`_llint_op_mov` not found in the system JSC — binary stripped); in
`excerpt` mode `op_mov` did not match the offlineasm bare-symbol
heuristic. LLInt reference taken directly from the offlineasm source at
`/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter64.asm`.

The closest LLInt analog to "write `false` to a register" is the
`move ValueFalse, t0` + `storeq t0, [...]` idiom used inline within
several handlers. The closest *opcode* analog is `op_mov` at line 906:

```
llintOpWithReturn(op_mov, OpMov, macro (size, get, dispatch, return)
    get(m_src, t1)
    loadConstantOrVariable(size, t1, t2)
    return(t2)
end)
```

JSC's `op_mov` is more general — it moves a constant-table entry or
register into the destination, going through `loadConstantOrVariable`
(which dispatches on the operand encoding). The `false` value lives in
the constant table for most JSC bytecode streams, so the equivalent
sequence is "decode src operand, load constant table[src], store to
dest, dispatch". Lyng's `op_load_false` short-circuits the constant-
table indirection by materializing `Value::from_bool(false)` directly
with a 3-instruction `movz/movk/movk` sequence — no memory access for
the value.

A more directly-comparable LLInt fragment is the `move ValueFalse, t0;
storeq t0, [...]` pattern used inline within several handlers — that
idiom is one instruction to materialize (because `ValueFalse` fits in a
single immediate on x86_64) + one to store. On AArch64 the LLInt
equivalent would also need a multi-instruction `movz/movk` sequence to
build the tagged JSValue, similar to lyng's tag-and-store fast path.

LLInt reference capture mode: **excerpt (manual)** — `capture-llint`
binary path failed; reference taken from local WebKit checkout at
`/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter64.asm`.

## Side-by-side diff

| Step | Lyng DSL                                         | LLInt (`op_mov`-via-false-constant)                 |
|------|--------------------------------------------------|-----------------------------------------------------|
| 1    | `ldrb w9, [x19, #1]` — decode `a`                | `get(m_dst, t0)` — decode dest reg                  |
| 2    | `ldrh w10, [x19, #2]` — decode `_bx` (unused)    | `get(m_src, t1)` — decode src constant-pool idx     |
| 3    | `movz x11, #0x0` (Bool payload)                  | `loadConstantOrVariable(size, t1, t2)` — table load |
| 4    | `movk x11, #0x3, lsl #32` (Bool kind tag)        | (rolled into step 3 as a single 64-bit `loadq`)     |
| 5    | `movk x11, #0x7ff8, lsl #48` (NaN-box header)    | (rolled into step 3)                                |
| 6    | `str x11, [x20, x9, lsl #3]` — store_reg!        | `return(t2)` → `storeq t2, [cfr, dst, 8]`           |
| 7-10 | `add/ldrb/ldr/br` — dispatch tail (4 instr)      | `dispatch()` — equivalent 4-instr tail              |

**Irreducible deltas vs LLInt:**

- **Value layout.** Lyng uses NaN-tagged 64-bit `Value` (high 16 bits = tag,
  low 32 bits = payload, canonical-NaN prefix in bits 48-60). `false` is the
  constant `0x7ff8_0003_0000_0000` (payload `#0x0`, kind `#0x3` for Bool),
  requiring a 3-instruction `movz/movk/movk` to materialize. Unlike
  `undefined` (`0x7ff8_0001_0000_0000`) and `null` (`0x7ff8_0002_0000_0000`)
  whose low 32 bits are zero — so their payload byte rides in the same
  shifted-`movz` as the kind tag — `Bool` has a payload byte that
  must be materialized separately. For `false` the payload is `#0` so the
  initial `movz` is technically writing a zero; LLVM still emits the
  3-instruction sequence because the macro expansion is uniform across
  `tag_bool_const!(t0, 0)` and `tag_bool_const!(t0, 1)`. JSC's JSValue
  is also NaN-boxed in 64-bit mode and uses `ValueFalse` as a single-
  instruction constant move on x86_64 (`movabsq $imm, t0`) and a
  3-instruction `movz/movk/movk` on AArch64 — parity, not a delta.
- **No constant-table indirection.** Lyng has a dedicated `LoadFalse`
  opcode so the value is in-line in the handler. JSC reuses `op_mov`
  and reads from the constant table, adding a `loadq` per dispatch. This
  is a strict win for lyng on `undefined`/`null`/`true`/`false`/`zero`/
  `one` — the canonical Phase 1.A constant-loaders.
- **Unused `bx` decode.** Lyng's `Abx` layout reads a 16-bit operand at
  `[PC+2]` even though `op_load_false` doesn't use it. This `ldrh` is the
  one un-LLInt-like instruction; it costs ~1 cycle and survives because
  LLVM can't prove the load is dead in a naked handler. A future codegen
  tweak could narrow the layout to `A` (length=2) and drop the unused
  operand entirely, saving 1 instruction + 2 bytes per call site. Out of
  scope for Phase 1.A.

## Microbench

Microbench snippet not yet present for `LoadFalse`; deferred to
Task 10.B. The pre-phase baseline at
`reports/lyng/dsl-1/pre-phase-1a-baseline.md` confirms this for
all nine Phase 1.A opcodes.

## V8 v7

A single-opcode port is not expected to move the V8 v7 geomean
measurably — the LoadFalse dispatch share is sub-percent on the
V8 v7 suite. Phase 1.A's aggregate impact will be measured at Task 10
against the pre-phase baseline geomean of **387.09** captured in
`reports/lyng/dsl-1/pre-phase-1a-baseline.md`.

## Slow-path-share

Slow-path-share counter is currently no-op (DSL-0c gap; tracked as Task
10.A to re-wire into DSL dispatch tail). Once re-wired, expected post-
port value: **0%** — the slow-path shim `op_load_false_slow_rs`
was deleted alongside this port; the opcode has no fail mode (it
unconditionally writes `Value::from_bool(false)` to register `a` and
dispatches).

## Behavioral tests

- `cargo test -p lyng-vm --lib --release` — **413 passed**.
- `cargo test -p lyng-tests --release` — **1186 passed (2 suites)**.

Both green; behavioral parity preserved.

## Notes

- **Identical shape to `op_load_true` with payload=0.** The only
  difference in the emitted asm is the immediate in the first `mov`/
  `movz` (`#0` for `false`, `#1` for `true`). All other 9 instructions
  are byte-for-byte identical to Task 3's baseline. The 3-instruction
  `tag_bool_const!` expansion is uniform across both polarities; the
  Bool payload byte is written by a separate `movz` regardless of
  whether it's `#0` or `#1`.
- **+1 instruction vs `undefined`/`null` is structural, not waste.** The
  `tag_bool_const!` macro emits 3 instructions (movz payload, movk kind,
  movk header) because the Bool tag's payload (`#0x1` for `true`,
  `#0x0` for `false`) lives in the low 32 bits of the Value word and
  cannot be folded into the kind-tag `movk` at `lsl #32`. By contrast
  `tag_undefined!` and `tag_null!` set payload=0 so their kind-tag
  `movz` doubles as the payload write — a free byte. This is
  fundamental to the NaN-tagged encoding; the only way to remove the
  extra instruction would be to fold the Bool payload into the kind
  byte (e.g. use kind `#0x3` for `false` and `#0x4` for `true`),
  which would burn a kind slot and complicate every Bool type check.
  For `false` specifically, the payload `movz x11, #0` *could* be
  elided if the macro special-cased payload=0 (since the subsequent
  `movk` at `lsl #32` would write the kind tag into a register whose
  low 32 bits start as garbage — needing a prior `mov x11, xzr` or
  an explicit zero). Net: a payload=0 special-case would save 1
  instruction but complicates the macro; out of scope for Phase 1.A.
- **Unused `bx` decode (`ldrh w10, [x19, #2]`).** LLVM did *not* elide
  the unused 16-bit load — the proc-macro lowerer's prologue emits
  `decode_abx!(a, _bx)` which expands unconditionally; LLVM has no
  visibility into the live-out set inside the naked-asm! block.
  Acceptable for Phase 1.A: 1 extra instruction, ~1 cycle. A future
  micro-optimization could re-shape the layout to drop the operand,
  but it requires bytecode-format coordination (opcode-length changes
  break ahead-of-time encoded streams) and is out of scope.
- **Slow-path shim deleted.** `op_load_false_slow_rs` was the
  cold-stub bridge to `op_load_false_semantic`; with the inline fast
  path it has no callers (grep confirmed only self-references) and
  was removed alongside the handler-body change.
- **Manifest entry unchanged.** The opcode manifest references
  `op_load_false_dsl` (the handler symbol, not the shim). The symbol
  identity is preserved; only the body changed.
- **Pre-existing infra gaps acknowledged (not fixed by this task):**
  1. Slow-path-share counter is no-op since DSL-0c (Task 10.A).
  2. Microbench snippets for the 9 Phase-1.A opcodes don't exist
     (Task 10.B).
- **LLInt reference capture mode:** *excerpt (manual)* — neither
  `system` nor `local` nor `excerpt` automated modes located `op_mov`
  in the system JSC binary or offlineasm sources; manual inspection
  of `/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter64.asm`
  yielded the reference fragments.
