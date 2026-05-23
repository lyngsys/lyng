# `op_load_one` DSL port (Phase 1.A, Task 6)

Sixth Phase-1.A port — symmetric mirror of `op_load_zero` (Task 5). The
handler writes `Value::from_smi(1)` to register `a`; the `bx` operand is
unused (layout reserves the slot for forward compat). This task reuses
the `tag_smi_const!` macro introduced in Task 5 — no new backend
infrastructure required.

The SMI tag carries an explicit payload in the low 32 bits (the i32
integer value) and the SMI kind discriminator (`#0x4`) in bits 32-47,
so `tag_smi_const!` produces **3 instructions** — same shape as
`tag_bool_const!` (with kind `#0x3` instead of `#0x4`). For SMI(1) the
full tagged Value is `0x7ff8_0004_0000_0001`.

## DSL source

`crates/lyng-js/vm/src/dsl/handlers/cold.rs`:

```rust
llint_handler! {
    op_load_one_dsl, layout = Abx, length = 4, |a, _bx| {
        tag_smi_const!(t0, 1);
        store_reg!(a, t0);
        dispatch!();
    }
}
```

## Backend macro (reused from Task 5)

`crates/lyng-js/vm/src/dsl/backend/aarch64/values.rs`:

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

Already present from Task 5 — no source change to `values.rs` in this
task. The payload (`$payload:literal`) is substituted into the initial
`movz` immediate at macro-expansion time, so `tag_smi_const!(t0, 1)`
emits `movz x11, #1` where `tag_smi_const!(t0, 0)` emits `movz x11, #0`.
All other expanded instructions (kind `movk` and header `movk`) are
byte-for-byte identical between the two call sites.

## Current asm (AArch64)

See `reports/js/lyng-js/dsl-asm-baseline-aarch64/op_load_one.asm`.

Captured from `target/release/deps/lyng_js_vm-*.s` after a
`cargo rustc --release -p lyng-js-vm --lib -- --emit=asm -C debuginfo=0`
build. Effective sequence:

```asm
op_load_one_dsl:
    ldrb    w9,  [x19, #1]              ; decode a (byte at PC+1)
    ldrh    w10, [x19, #2]              ; decode _bx (unused; LLVM kept the load)
    mov     x11, #1                     ; tag_smi_const!(t0, 1) — movz x11, #0x1 (payload)
    movk    x11, #4, lsl #32            ; tag_smi_const!(t0, 1) — movk x11, #0x4, lsl #32 (SMI kind)
    movk    x11, #32760, lsl #48        ; tag_smi_const!(t0, 1) — movk x11, #0x7ff8, lsl #48
    str     x11, [x20, x9, lsl #3]      ; store_reg!(a, t0) — REGS[a] := SMI(1)
    add     x19, x19, #4                ; dispatch!() — advance PC by length=4
    ldrb    w8,  [x19]                  ; dispatch!() — load next opcode byte
    ldr     x16, [x23, x8, lsl #3]      ; dispatch!() — look up next handler
    br      x16                         ; dispatch!() — tail-jump
```

**10 instructions** total — identical shape to `op_load_zero` (10 each),
`op_load_true` (10), `op_load_false` (10); one more than `op_load_undefined`/
`op_load_null` (9 each). The +1 comes from `tag_smi_const!`'s 3-instruction
expansion: the SMI tag needs both an explicit payload (`#0x1` for one)
and an explicit kind (`#0x4`) materialized in different bit ranges. The
only byte-level difference from `op_load_zero`'s baseline asm is the
payload immediate in the first `movz` (`#1` vs `#0`) and the symbol name.

Note: the LLVM assembler rewrote the canonical `tag_smi_const!` form
(`movz x11, #0x1 ; movk x11, #0x4, lsl #32 ; movk x11, #0x7ff8, lsl #48`)
to the equivalent (`mov x11, #1 ; movk x11, #4, lsl #32 ; movk x11, #32760, lsl #48`).
The first `movz` and the rewritten `mov` produce identical machine encoding;
the `movk` immediates are decimal renderings of the same hex values
(`4 == 0x4`, `32760 == 0x7ff8`). No observable difference at runtime.

## LLInt reference

`capture-llint` in `auto` mode could not locate the LLInt symbol
(the JSC binary lacks an exact `op_load_one` analog; the closest
counterpart is `op_load_int_constant`, which is internal to JSC's
constant-pool resolution and not a standalone bytecode op). In
`excerpt` mode `op_load_one` did not match the offlineasm bare-symbol
heuristic. LLInt reference taken directly from the offlineasm source at
`/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter64.asm`.

The closest LLInt analog is the same `op_mov` constant-pool fetch
pattern noted in `op_load_zero.md`:

```
llintOpWithReturn(op_mov, OpMov, macro (size, get, dispatch, return)
    get(m_src, t1)
    loadConstantOrVariable(size, t1, t2)
    return(t2)
end)
```

JSC's `op_mov` is more general — it moves a constant-table entry or
register into the destination, going through `loadConstantOrVariable`
(which dispatches on the operand encoding). The integer `1` lives in
the constant table for most JSC bytecode streams (as a JSValue with
the SMI tag pattern `0xffff_0000_0000_0001` on x86_64 or
`0x7ffe_0000_0000_0001` on AArch64 with `Int32Tag` discriminator), so
the equivalent sequence is "decode src operand, load constant table[src],
store to dest, dispatch". Lyng's `op_load_one` short-circuits the
constant-table indirection by materializing `Value::from_smi(1)`
directly with a 3-instruction `movz/movk/movk` sequence — no memory
access for the value.

A more directly-comparable LLInt fragment is the `move Int32Tag, t0;
move 1, t1; storeq t0, [...]; storeq t1, [...]` pattern (or the
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

| Step | Lyng DSL                                         | LLInt (`op_mov`-via-one-constant)                   |
|------|--------------------------------------------------|-----------------------------------------------------|
| 1    | `ldrb w9, [x19, #1]` — decode `a`                | `get(m_dst, t0)` — decode dest reg                  |
| 2    | `ldrh w10, [x19, #2]` — decode `_bx` (unused)    | `get(m_src, t1)` — decode src constant-pool idx     |
| 3    | `movz x11, #0x1` (SMI payload)                   | `loadConstantOrVariable(size, t1, t2)` — table load |
| 4    | `movk x11, #0x4, lsl #32` (SMI kind tag)         | (rolled into step 3 as a single 64-bit `loadq`)     |
| 5    | `movk x11, #0x7ff8, lsl #48` (NaN-box header)    | (rolled into step 3)                                |
| 6    | `str x11, [x20, x9, lsl #3]` — store_reg!        | `return(t2)` → `storeq t2, [cfr, dst, 8]`           |
| 7-10 | `add/ldrb/ldr/br` — dispatch tail (4 instr)      | `dispatch()` — equivalent 4-instr tail              |

**Irreducible deltas vs LLInt:**

- **Value layout.** Lyng uses NaN-tagged 64-bit `Value` (high 16 bits = tag,
  low 32 bits = payload, canonical-NaN prefix in bits 48-60). SMI(1) is the
  constant `0x7ff8_0004_0000_0001` (payload `#0x1`, kind `#0x4` for SMI),
  requiring a 3-instruction `movz/movk/movk` to materialize. Unlike
  `undefined` (`0x7ff8_0001_0000_0000`) and `null` (`0x7ff8_0002_0000_0000`)
  whose low 32 bits are zero — so their payload byte rides in the same
  shifted-`movz` as the kind tag — `SMI` has a payload (the integer
  value) that must be materialized separately. For SMI(1) the payload is
  `#1`. JSC's JSValue is also NaN-boxed in 64-bit mode; the AArch64
  LLInt has to build the same 3-instruction tag-and-payload sequence
  (or load a cached JSValue from the constant table — parity, not a delta).
- **No constant-table indirection.** Lyng has a dedicated `LoadOne`
  opcode so the value is in-line in the handler. JSC reuses `op_mov`
  and reads from the constant table, adding a `loadq` per dispatch. This
  is a strict win for lyng on `undefined`/`null`/`true`/`false`/`zero`/
  `one` — the canonical Phase 1.A constant-loaders.
- **Unused `bx` decode.** Lyng's `Abx` layout reads a 16-bit operand at
  `[PC+2]` even though `op_load_one` doesn't use it. This `ldrh` is the
  one un-LLInt-like instruction; it costs ~1 cycle and survives because
  LLVM can't prove the load is dead in a naked handler. A future codegen
  tweak could narrow the layout to `A` (length=2) and drop the unused
  operand entirely, saving 1 instruction + 2 bytes per call site. Out of
  scope for Phase 1.A.

## Microbench

Microbench snippet not yet present for `LoadOne`; deferred to
Task 10.B. The pre-phase baseline at
`reports/js/lyng-js/dsl-1/pre-phase-1a-baseline.md` confirms this for
all nine Phase 1.A opcodes.

## V8 v7

A single-opcode port is not expected to move the V8 v7 geomean
measurably — the LoadOne dispatch share is sub-percent on the
V8 v7 suite. Phase 1.A's aggregate impact will be measured at Task 10
against the pre-phase baseline geomean of **387.09** captured in
`reports/js/lyng-js/dsl-1/pre-phase-1a-baseline.md`.

## Slow-path-share

Slow-path-share counter is currently no-op (DSL-0c gap; tracked as Task
10.A to re-wire into DSL dispatch tail). Once re-wired, expected post-
port value: **0%** — the slow-path shim `op_load_one_slow_rs`
was deleted alongside this port; the opcode has no fail mode (it
unconditionally writes `Value::from_smi(1)` to register `a` and
dispatches).

## Behavioral tests

- `cargo test -p lyng-js-vm --lib --release` — **413 passed**.
- `cargo test -p lyng-js-tests --release` — **1186 passed (2 suites)**.

Both green; behavioral parity preserved.

## Notes

- **Symmetric pair to `op_load_zero`.** This task is a direct mirror of
  Task 5 with payload `1` substituted for `0` in the `tag_smi_const!`
  invocation. The asm baselines for `op_load_zero` and `op_load_one`
  are byte-for-byte identical except for (a) the symbol name and (b)
  the payload immediate in the first `movz` (`#1` vs `#0`). No new
  macros, no new imports, no codegen-side changes — pure handler-body
  swap + slow-shim deletion.
- **`tag_smi_const!` macro reused from Task 5.** No source change to
  `crates/lyng-js/vm/src/dsl/backend/aarch64/values.rs` in this task.
  The macro covers both `op_load_zero` (payload = 0) and `op_load_one`
  (payload = 1). Larger SMI payloads (op_load_smi8 sign-extension,
  op_load_smi 16-bit) need different handling — out of scope here;
  addressed in Task 7.
- **Distinct from `tag_smi!`.** The pre-existing `tag_smi!` macro
  (values.rs line 179) expects the payload to already be in the
  destination register's low word — used by SMI arithmetic results
  where the runtime output of an `add`/`sub`/etc lives in a register.
  `tag_smi_const!` is the compile-time-payload sibling: when the
  payload is known at codegen time, it can be folded into a `movz`
  immediate, saving the prior `mov` + `uxtw` + `orr` shape.
- **Identical asm shape to `op_load_zero`.** The only differences from
  `op_load_zero`'s baseline asm are (1) the payload immediate (`#1` vs
  `#0`) in the first instruction of the tag sequence and (2) the symbol
  name. All other 9 instructions are byte-for-byte identical.
- **Unused `bx` decode (`ldrh w10, [x19, #2]`).** Same as `op_load_zero`:
  LLVM did *not* elide the unused 16-bit load — the proc-macro lowerer's
  prologue emits `decode_abx!(a, _bx)` which expands unconditionally;
  LLVM has no visibility into the live-out set inside the naked-asm! block.
  Acceptable for Phase 1.A: 1 extra instruction, ~1 cycle. A future
  micro-optimization could re-shape the layout to drop the operand,
  but it requires bytecode-format coordination (opcode-length changes
  break ahead-of-time encoded streams) and is out of scope.
- **Slow-path shim deleted.** `op_load_one_slow_rs` was the cold-stub
  bridge to `op_load_one_semantic`; with the inline fast path it has
  no callers (grep confirmed only self-references) and was removed
  alongside the handler-body change. The semantic body
  (`crate::vm::semantics::loads::op_load_one_semantic`) is still
  reachable from `evaluator.rs` and `bytecode_lowering`-style code paths;
  it was *not* deleted.
- **Manifest entry unchanged.** The opcode manifest references
  `op_load_one_dsl` (the handler symbol, not the shim). The symbol
  identity is preserved; only the body changed.
- **Pre-existing infra gaps acknowledged (not fixed by this task):**
  1. Slow-path-share counter is no-op since DSL-0c (Task 10.A).
  2. Microbench snippets for the 9 Phase-1.A opcodes don't exist
     (Task 10.B).
- **LLInt reference capture mode:** *excerpt (manual)* — neither
  `system` nor `local` nor `excerpt` automated modes located an exact
  `op_load_one` analog in the system JSC binary or offlineasm sources
  (JSC routes one through `op_mov` + constant-table); manual inspection
  of `/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter64.asm`
  yielded the reference fragments above.
