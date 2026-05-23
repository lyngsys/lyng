# `op_load_zero` DSL port (Phase 1.A, Task 5)

Fifth Phase-1.A port — first SMI constant-loader. The handler writes
`Value::from_smi(0)` to register `a`; the `bx` operand is unused (layout
reserves the slot for forward compat). This task introduces a new
backend macro, `tag_smi_const!`, which materializes a tagged SMI from a
compile-time literal payload. Reusable by `op_load_one` (Task 6) and
similar SMI constant-loaders.

The SMI tag carries an explicit payload in the low 32 bits (the i32
integer value) and the SMI kind discriminator (`#0x4`) in bits 32-47,
so `tag_smi_const!` produces **3 instructions** — same shape as
`tag_bool_const!` (with kind `#0x3` instead of `#0x4`). For SMI(0) the
full tagged Value is `0x7ff8_0004_0000_0000`.

## DSL source

`crates/vm/src/dsl/handlers/cold.rs`:

```rust
llint_handler! {
    op_load_zero_dsl, layout = Abx, length = 4, |a, _bx| {
        tag_smi_const!(t0, 0);
        store_reg!(a, t0);
        dispatch!();
    }
}
```

## New backend macro

`crates/vm/src/dsl/backend/aarch64/values.rs`:

```rust
#[macro_export]
macro_rules! tag_smi_const {
    ($reg:tt, $payload:literal) => {
        concat!(
            "movz   x", stringify!($reg), ", #", stringify!($payload), "\n",
            "movk   x", stringify!($reg), ", #0x4, lsl #32\n",
            "movk   x", stringify!($reg), ", #0x7ff8, lsl #48\n",
        )
    };
}
```

Mirrors `tag_bool_const!` (kind = `#0x3`) but with the SMI kind tag
(`#0x4`). The payload occupies the low 16 bits of the encoding —
`movz` accepts a 16-bit immediate, so this form only supports literals
that fit in `[0, 0xffff]`. For Phase 1.A the only callers will be
`op_load_zero` (payload = 0) and `op_load_one` (payload = 1), both
well within range. Larger SMI payloads (op_load_smi8 sign-extension,
op_load_smi 16-bit) need different handling (see Task 7).

Distinct from `tag_smi!`, which expects the payload to already be in
the register's low word (used by SMI arithmetic results, where the
payload is the runtime output of an `add`/`sub`/etc).

## Current asm (AArch64)

See `reports/lyng/dsl-asm-baseline-aarch64/op_load_zero.asm`.

Captured from `target/release/deps/lyng_vm-*.s` after a
`cargo rustc --release -p lyng-vm --lib -- --emit=asm -C debuginfo=0`
build. Effective sequence:

```asm
op_load_zero_dsl:
    ldrb    w9,  [x19, #1]              ; decode a (byte at PC+1)
    ldrh    w10, [x19, #2]              ; decode _bx (unused; LLVM kept the load)
    mov     x11, #0                     ; tag_smi_const!(t0, 0) — movz x11, #0x0 (payload)
    movk    x11, #4, lsl #32            ; tag_smi_const!(t0, 0) — movk x11, #0x4, lsl #32 (SMI kind)
    movk    x11, #32760, lsl #48        ; tag_smi_const!(t0, 0) — movk x11, #0x7ff8, lsl #48
    str     x11, [x20, x9, lsl #3]      ; store_reg!(a, t0) — REGS[a] := SMI(0)
    add     x19, x19, #4                ; dispatch!() — advance PC by length=4
    ldrb    w8,  [x19]                  ; dispatch!() — load next opcode byte
    ldr     x16, [x23, x8, lsl #3]      ; dispatch!() — look up next handler
    br      x16                         ; dispatch!() — tail-jump
```

**10 instructions** total — identical shape to `op_load_true` and
`op_load_false` (10 each), one more than `op_load_undefined`/`op_load_null`
(9 each). The +1 comes from `tag_smi_const!`'s 3-instruction expansion:
the SMI tag needs both an explicit payload (`#0x0` for zero) and an
explicit kind (`#0x4`) materialized in different bit ranges. For SMI(0)
specifically the payload `movz` writes `#0` — analogous to `op_load_false`'s
payload `movz`. LLVM did not elide the unused `ldrh` decode for `_bx`;
the encoding is small (4 bytes) and proving the load is dead requires
alias analysis on the bytecode buffer that LLVM doesn't perform here.

Note: the LLVM assembler rewrote the canonical `tag_smi_const!` form
(`movz x11, #0x0 ; movk x11, #0x4, lsl #32 ; movk x11, #0x7ff8, lsl #48`)
to the equivalent (`mov x11, #0 ; movk x11, #4, lsl #32 ; movk x11, #32760, lsl #48`).
The first `movz` and the rewritten `mov` produce identical machine encoding;
the `movk` immediates are decimal renderings of the same hex values
(`4 == 0x4`, `32760 == 0x7ff8`). No observable difference at runtime.

## LLInt reference

`capture-llint` in `auto` mode could not locate the LLInt symbol
(the JSC binary lacks an exact `op_load_zero` analog; the closest
counterpart is `op_load_int_constant`, which is internal to JSC's
constant-pool resolution and not a standalone bytecode op). In
`excerpt` mode `op_load_zero` did not match the offlineasm bare-symbol
heuristic. LLInt reference taken directly from the offlineasm source at
`/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter64.asm`.

The closest LLInt analog is the same `op_mov` constant-pool fetch
pattern noted in `op_load_false.md`:

```
llintOpWithReturn(op_mov, OpMov, macro (size, get, dispatch, return)
    get(m_src, t1)
    loadConstantOrVariable(size, t1, t2)
    return(t2)
end)
```

JSC's `op_mov` is more general — it moves a constant-table entry or
register into the destination, going through `loadConstantOrVariable`
(which dispatches on the operand encoding). The integer `0` lives in
the constant table for most JSC bytecode streams (as a JSValue with
the SMI tag pattern `0xffff_0000_0000_0000` on x86_64 or
`0x7ffe_0000_0000_0000` on AArch64 with `Int32Tag` discriminator), so
the equivalent sequence is "decode src operand, load constant table[src],
store to dest, dispatch". Lyng's `op_load_zero` short-circuits the
constant-table indirection by materializing `Value::from_smi(0)`
directly with a 3-instruction `movz/movk/movk` sequence — no memory
access for the value.

A more directly-comparable LLInt fragment is the `move Int32Tag, t0;
move 0, t1; storeq t0, [...]; storeq t1, [...]` pattern (or the
`storeInt32` macro on 64-bit) used inline within several handlers — that
idiom is one instruction per tag/payload write on x86_64 (because
`Int32Tag` fits in a single immediate) + the stores. On AArch64 the
LLInt equivalent would also need a multi-instruction `movz/movk`
sequence to build the tagged JSValue, similar to lyng's tag-and-store
fast path.

LLInt reference capture mode: **excerpt (manual)** — `capture-llint`
binary path failed; reference taken from local WebKit checkout at
`/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter64.asm`.

## Side-by-side diff

| Step | Lyng DSL                                         | LLInt (`op_mov`-via-zero-constant)                  |
|------|--------------------------------------------------|-----------------------------------------------------|
| 1    | `ldrb w9, [x19, #1]` — decode `a`                | `get(m_dst, t0)` — decode dest reg                  |
| 2    | `ldrh w10, [x19, #2]` — decode `_bx` (unused)    | `get(m_src, t1)` — decode src constant-pool idx     |
| 3    | `movz x11, #0x0` (SMI payload)                   | `loadConstantOrVariable(size, t1, t2)` — table load |
| 4    | `movk x11, #0x4, lsl #32` (SMI kind tag)         | (rolled into step 3 as a single 64-bit `loadq`)     |
| 5    | `movk x11, #0x7ff8, lsl #48` (NaN-box header)    | (rolled into step 3)                                |
| 6    | `str x11, [x20, x9, lsl #3]` — store_reg!        | `return(t2)` → `storeq t2, [cfr, dst, 8]`           |
| 7-10 | `add/ldrb/ldr/br` — dispatch tail (4 instr)      | `dispatch()` — equivalent 4-instr tail              |

**Irreducible deltas vs LLInt:**

- **Value layout.** Lyng uses NaN-tagged 64-bit `Value` (high 16 bits = tag,
  low 32 bits = payload, canonical-NaN prefix in bits 48-60). SMI(0) is the
  constant `0x7ff8_0004_0000_0000` (payload `#0x0`, kind `#0x4` for SMI),
  requiring a 3-instruction `movz/movk/movk` to materialize. Unlike
  `undefined` (`0x7ff8_0001_0000_0000`) and `null` (`0x7ff8_0002_0000_0000`)
  whose low 32 bits are zero — so their payload byte rides in the same
  shifted-`movz` as the kind tag — `SMI` has a payload (the integer
  value) that must be materialized separately. For SMI(0) the payload is
  `#0`, so the initial `movz` is technically writing a zero; LLVM still
  emits the 3-instruction sequence because the macro expansion is
  uniform across `tag_smi_const!(t0, 0)` and `tag_smi_const!(t0, 1)`.
  JSC's JSValue is also NaN-boxed in 64-bit mode; the AArch64 LLInt has
  to build the same 3-instruction tag-and-payload sequence (or load a
  cached JSValue from the constant table — parity, not a delta).
- **No constant-table indirection.** Lyng has a dedicated `LoadZero`
  opcode so the value is in-line in the handler. JSC reuses `op_mov`
  and reads from the constant table, adding a `loadq` per dispatch. This
  is a strict win for lyng on `undefined`/`null`/`true`/`false`/`zero`/
  `one` — the canonical Phase 1.A constant-loaders.
- **Unused `bx` decode.** Lyng's `Abx` layout reads a 16-bit operand at
  `[PC+2]` even though `op_load_zero` doesn't use it. This `ldrh` is the
  one un-LLInt-like instruction; it costs ~1 cycle and survives because
  LLVM can't prove the load is dead in a naked handler. A future codegen
  tweak could narrow the layout to `A` (length=2) and drop the unused
  operand entirely, saving 1 instruction + 2 bytes per call site. Out of
  scope for Phase 1.A.

## Microbench

Microbench snippet not yet present for `LoadZero`; deferred to
Task 10.B. The pre-phase baseline at
`reports/lyng/dsl-1/pre-phase-1a-baseline.md` confirms this for
all nine Phase 1.A opcodes.

## V8 v7

A single-opcode port is not expected to move the V8 v7 geomean
measurably — the LoadZero dispatch share is sub-percent on the
V8 v7 suite. Phase 1.A's aggregate impact will be measured at Task 10
against the pre-phase baseline geomean of **387.09** captured in
`reports/lyng/dsl-1/pre-phase-1a-baseline.md`.

## Slow-path-share

Slow-path-share counter is currently no-op (DSL-0c gap; tracked as Task
10.A to re-wire into DSL dispatch tail). Once re-wired, expected post-
port value: **0%** — the slow-path shim `op_load_zero_slow_rs`
was deleted alongside this port; the opcode has no fail mode (it
unconditionally writes `Value::from_smi(0)` to register `a` and
dispatches).

## Behavioral tests

- `cargo test -p lyng-vm --lib --release` — **413 passed**.
- `cargo test -p lyng-tests --release` — **1186 passed (2 suites)**.

Both green; behavioral parity preserved.

## Notes

- **First SMI port — new `tag_smi_const!` macro.** This task introduces
  a new backend macro that's reusable by `op_load_one` (Task 6) and
  similar future SMI constant-loaders that fit a 16-bit literal payload.
  `tag_smi_const!` mirrors the shape of `tag_bool_const!` (added in
  Task 3): payload `movz` + kind `movk` + header `movk`. The kind tag
  differs (SMI = `#0x4` vs Bool = `#0x3`); the header is identical.
- **Distinct from `tag_smi!`.** The pre-existing `tag_smi!` macro
  (values.rs line 179) expects the payload to already be in the
  destination register's low word — used by SMI arithmetic results
  where the runtime output of an `add`/`sub`/etc lives in a register.
  `tag_smi_const!` is the compile-time-payload sibling: when the
  payload is known at codegen time, it can be folded into a `movz`
  immediate, saving the prior `mov` + `uxtw` + `orr` shape.
- **Identical asm shape to `op_load_true`/`op_load_false` with kind tag = `#0x4`.**
  The only differences from `op_load_false`'s baseline asm are (1)
  the kind-tag immediate (`#4` vs `#3`) in the second instruction of
  the tag sequence and (2) the symbol name. All other 9 instructions
  are byte-for-byte identical.
- **Payload range.** `tag_smi_const!` only supports literals fitting
  in `movz`'s 16-bit immediate (`[0, 0xffff]`). For Phase 1.A this
  covers `op_load_zero` (payload = 0) and `op_load_one` (payload = 1).
  Larger SMI payloads need different handling: `op_load_smi8` has a
  signed 8-bit payload (could be sign-extended into a 16-bit `movz`
  immediate for positive values, but negative values need a `movn` or
  prior `mov + sxtb`), and `op_load_smi` has a 16-bit payload that
  also needs sign extension. Out of scope for Task 5; addressed in
  Task 7.
- **Unused `bx` decode (`ldrh w10, [x19, #2]`).** Same as `op_load_false`:
  LLVM did *not* elide the unused 16-bit load — the proc-macro lowerer's
  prologue emits `decode_abx!(a, _bx)` which expands unconditionally;
  LLVM has no visibility into the live-out set inside the naked-asm! block.
  Acceptable for Phase 1.A: 1 extra instruction, ~1 cycle. A future
  micro-optimization could re-shape the layout to drop the operand,
  but it requires bytecode-format coordination (opcode-length changes
  break ahead-of-time encoded streams) and is out of scope.
- **Slow-path shim deleted.** `op_load_zero_slow_rs` was the cold-stub
  bridge to `op_load_zero_semantic`; with the inline fast path it has
  no callers (grep confirmed only self-references) and was removed
  alongside the handler-body change. The semantic body
  (`crate::vm::semantics::loads::op_load_zero_semantic`) is still
  reachable from `evaluator.rs` and `bytecode_lowering`-style code paths;
  it was *not* deleted.
- **Manifest entry unchanged.** The opcode manifest references
  `op_load_zero_dsl` (the handler symbol, not the shim). The symbol
  identity is preserved; only the body changed.
- **Pre-existing infra gaps acknowledged (not fixed by this task):**
  1. Slow-path-share counter is no-op since DSL-0c (Task 10.A).
  2. Microbench snippets for the 9 Phase-1.A opcodes don't exist
     (Task 10.B).
- **LLInt reference capture mode:** *excerpt (manual)* — neither
  `system` nor `local` nor `excerpt` automated modes located an exact
  `op_load_zero` analog in the system JSC binary or offlineasm sources
  (JSC routes zero through `op_mov` + constant-table); manual inspection
  of `/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter64.asm`
  yielded the reference fragments above.
