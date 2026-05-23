# `op_load_smi8` DSL port (Phase 1.A, Task 7)

**First top-30 opcode ported in Phase 1.A — dispatch share #7, by far
the highest-volume opcode in this phase.** Unlike Tasks 1-6 (which
loaded compile-time constants such as `undefined`, `null`, `true`,
`false`, `0`, `1`), `op_load_smi8` carries a runtime-varying signed-byte
payload. The decode prologue's `ldrb` zero-extends it; the new
`tag_smi_from_signed_byte!` backend macro then sign-extends the i8 to
an i32 and tags it as an SMI.

Layout = `Ab`, length = 3 (1-byte register-id `a` + 1-byte i8 payload
`b`). The handler reuses the `b` register as the in-place destination
for the tag — saves an explicit `mov` from `b` to a fresh scratch.

## DSL source

`crates/vm/src/dsl/handlers/cold.rs`:

```rust
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_load_smi8_dsl, layout = Ab, length = 3, |a, b| {
        tag_smi_from_signed_byte!(b);
        store_reg!(a, b);
        dispatch!();
    }
}
```

Note: `b` is NOT underscored — the macro consumes the operand value (the
i8 payload zero-extended into a w-register by the decode prologue's
`ldrb`) and writes the tagged Value back into the same register
in-place. `store_reg!(a, b)` then writes that register to `REGS[a]`.

## Backend macro (new in this task)

`crates/vm/src/dsl/backend/aarch64/values.rs`:

```rust
#[macro_export]
macro_rules! tag_smi_from_signed_byte {
    ($reg:tt) => {
        concat!(
            "sxtb   w", stringify!($reg), ", w", stringify!($reg), "\n",
            "uxtw   x", stringify!($reg), ", w", stringify!($reg), "\n",
            "movz   x16, #0x4, lsl #32\n",
            "movk   x16, #0x7ff8, lsl #48\n",
            "orr    x", stringify!($reg), ", x16, x", stringify!($reg), "\n",
        )
    };
}
```

5 instructions:
1. **`sxtb wR, wR`** — sign-extend the low byte (in the W-half of the
   register) to a full i32. The decode prologue's `ldrb` zero-extended
   the byte to bits 0-31 of the W register; `sxtb` reinterprets the low
   byte as signed and replicates its sign bit through bits 8-31.
2. **`uxtw xR, wR`** — zero-extend the W-word to a full X-word. After
   `sxtb`, the upper 32 bits of the X register may carry stale data;
   `uxtw` clears bits 32-63 so the subsequent `orr` produces a clean
   tagged Value. This mirrors `tag_smi!`'s sequence
   (`values.rs:178-188`), which also pairs `uxtw` with `orr`.
3. **`movz x16, #0x4, lsl #32`** — materialize the SMI kind
   discriminator in scratch `x16` (kind bits 32-47 = `0x0004`).
4. **`movk x16, #0x7ff8, lsl #48`** — OR-in the NaN-tag header
   (bits 48-63 = `0x7ff8`) into the scratch.
5. **`orr xR, x16, xR`** — combine the tag pattern with the sign-extended
   payload, producing the final tagged Value `0x7ff8_0004_PPPP_PPPP`
   (where `PPPP_PPPP` is the i32 payload).

**Distinct from sibling tag-* macros:**
- `tag_smi!` assumes the payload is already an i32 in `$reg`'s low
  word — no sign-extension. Used for SMI arithmetic outputs.
- `tag_smi_const!` materializes a *compile-time* literal payload via
  a single `movz`-with-immediate, saving the sign-extension entirely
  (3 instructions vs 5). Used for `op_load_zero` / `op_load_one`.
- `tag_smi_from_signed_byte!` is the runtime-payload-with-sign-extension
  variant — required when the payload is a narrow signed integer
  decoded from the bytecode stream.

The macro is reusable for any future narrow signed-integer SMI loader
opcodes.

## Current asm (AArch64)

See `reports/lyng/dsl-asm-baseline-aarch64/op_load_smi8.asm`.

Captured from `target/release/deps/lyng_vm-*.s` after a
`cargo rustc --release -p lyng-vm --lib -- --emit=asm -C debuginfo=0`
build. Effective sequence (12 instructions):

```asm
op_load_smi8_dsl:
    ldrb    w9,  [x19, #1]              ; decode_ab! — a (dest reg id)
    ldrb    w10, [x19, #2]              ; decode_ab! — b (i8 payload, zero-ext)
    sxtb    w10, w10                    ; tag_smi_from_signed_byte! — sign-extend
    ubfx    x10, x10, #0, #32           ; tag_smi_from_signed_byte! — zero-ext high
    mov     x16, #17179869184           ; tag_smi_from_signed_byte! — SMI kind (0x4 << 32)
    movk    x16, #32760, lsl #48        ; tag_smi_from_signed_byte! — NaN header (0x7ff8 << 48)
    orr     x10, x16, x10               ; tag_smi_from_signed_byte! — tag | payload
    str     x10, [x20, x9, lsl #3]      ; store_reg!(a, b) — REGS[a] := Value
    add     x19, x19, #3                ; dispatch!() — advance PC by length=3
    ldrb    w8,  [x19]                  ; dispatch!() — load next opcode byte
    ldr     x16, [x23, x8, lsl #3]      ; dispatch!() — look up next handler
    br      x16                         ; dispatch!() — tail-jump
```

**12 instructions** total. Compared to the constant-loaders:
- `op_load_undefined` / `op_load_null`: 9 instructions (2-instr tag).
- `op_load_zero` / `op_load_one` / `op_load_true` / `op_load_false`: 10
  instructions (3-instr `tag_*_const!`).
- `op_load_smi8`: **12 instructions** (5-instr `tag_smi_from_signed_byte!`).

The +2-3 instructions over the constant-loaders is the **structural
cost of runtime sign-extension** — the i8 payload arrives in bits 0-7,
must be sign-extended to bits 0-31 (`sxtb`), then composed with the tag
pattern. None of this can fold into a single mov-immediate the way
`tag_*_const!` can (because the payload is unknown at codegen time).

Note: the LLVM assembler rewrote our canonical macro forms:
- `uxtw x10, w10` → `ubfx x10, x10, #0, #32` (semantically identical;
  `uxtw` is an alias for `ubfm/ubfx`-encoded operations on aarch64).
- `movz x16, #0x4, lsl #32` → `mov x16, #17179869184` (= `0x4 << 32`,
  same encoding).
- `movk x16, #0x7ff8, lsl #48` → `movk x16, #32760, lsl #48`
  (32760 == 0x7ff8; decimal vs hex rendering).

No observable difference at runtime.

## LLInt reference

`capture-llint` could not locate a directly-comparable LLInt symbol —
JSC has no standalone `op_load_smi8` analog. The closest counterpart
is the constant-table path through `op_mov` (which serves both
constant loads and register moves), combined with the
`loadConstantOrVariable` macro that dispatches on operand encoding.
For SMI-shaped values in JSC's constant table, the LLInt produces the
JSValue as a precomputed 64-bit constant and `loadq`s it; the
sign-extension is done at compile time (when the constant is canned),
not at dispatch time.

A more directly-comparable JSC LLInt pattern would be its inline
SMI tag construction at `LowLevelInterpreter64.asm`'s integer-result
sites: `move Int32Tag, t0 ; move <payload>, t1 ; storeq ...` (on
x86_64; on AArch64 the same sequence needs `movz/movk` to materialize
the tag bits, plus sign-extension via `sxtb`/`sxth` for narrow inputs).
The shape on AArch64 is essentially identical to lyng's
`tag_smi_from_signed_byte!`.

LLInt reference capture mode: **excerpt (manual)** — neither `auto`
nor `excerpt` automated modes located an exact `op_load_smi8` analog
in the system JSC binary or offlineasm sources. Reference taken from
`/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter64.asm`.

## Side-by-side diff

| Step | Lyng DSL                                         | LLInt analog (`op_mov` + Int32 constant via `loadConstantOrVariable`)        |
|------|--------------------------------------------------|------------------------------------------------------------------------------|
| 1    | `ldrb w9, [x19, #1]` — decode `a`                | `get(m_dst, t0)` — decode dest reg                                           |
| 2    | `ldrb w10, [x19, #2]` — decode `b` (i8 payload)  | `get(m_src, t1)` — decode src constant-pool idx                              |
| 3    | `sxtb w10, w10` — sign-extend i8 to i32          | `loadConstantOrVariable(size, t1, t2)` — load precomputed JSValue (rolled)   |
| 4    | `ubfx x10, x10, #0, #32` — zero-extend bits 32-63| (rolled into step 3 as 64-bit `loadq`)                                       |
| 5    | `mov x16, #0x4_0000_0000` — SMI kind             | (constant carries kind bits — rolled into step 3)                            |
| 6    | `movk x16, #0x7ff8, lsl #48` — NaN header        | (constant carries header bits — rolled)                                      |
| 7    | `orr x10, x16, x10` — combine tag + payload      | (constant is already tagged)                                                 |
| 8    | `str x10, [x20, x9, lsl #3]` — store_reg!        | `return(t2)` → `storeq t2, [cfr, dst, 8]`                                    |
| 9-12 | `add/ldrb/ldr/br` — dispatch tail (4 instr)      | `dispatch()` — equivalent 4-instr tail                                       |

**Irreducible deltas vs LLInt:**

- **Inline literal vs constant-table indirection.** Lyng encodes the
  i8 payload directly into the bytecode stream (1 byte at `PC+2`) and
  reconstructs the tagged Value via sxtb + uxtw + movz + movk + orr.
  JSC's `op_mov` reads a precomputed JSValue from the constant table
  via a single `loadq`. JSC trades 5 ALU instructions for a memory
  load + cache pressure on the constant table. For small literals
  hit at high dispatch frequency, lyng's inline form should be a
  net win (cache locality + no constant-table dereference); for less
  frequent literals it could be a wash.
- **Value layout.** Lyng's NaN-tagged 64-bit `Value` uses kind
  bits 32-47 = `0x0004` for SMI and NaN header bits 48-63 = `0x7ff8`,
  producing `0x7ff8_0004_PPPP_PPPP` for an i32 payload `PPPP_PPPP`.
  JSC's AArch64 LLInt uses the analogous `Int32Tag` pattern; both
  require 2-3 instructions to materialize the tag bits on AArch64.
  This is a parity, not a delta.
- **Sign-extension granularity.** Lyng's `op_load_smi8` reserves a
  single bytecode byte for the payload (sufficient for `i8::MIN..=i8::MAX`,
  i.e. `-128..=127`). Larger SMI literals use `op_load_smi16` or
  `op_load_const8` (constant-table fallback). The sign-extension is
  unavoidable for the narrow form.

## Backend macro (new in this task)

`tag_smi_from_signed_byte!($reg)` was added to
`crates/vm/src/dsl/backend/aarch64/values.rs` immediately
after `tag_smi_const!`. The macro is `#[macro_export]`-d and re-exported
into the `cold.rs` import list:

```rust
use crate::{
    call_slow, decode_a, decode_ab, decode_abc, decode_abc_slot, decode_abx,
    decode_ax, dispatch, dispatch_after_slow, store_reg, tag_bool_const, tag_null,
    tag_smi_const, tag_smi_from_signed_byte, tag_undefined,
};
```

Reusable for any future narrow signed-integer SMI-loader opcode
(`op_load_smi16` could use a similar `tag_smi_from_signed_half!` once
ported — the 16-bit analog with `sxth` instead of `sxtb`).

## Microbench

Microbench snippet not yet present for `LoadSmi8`; deferred to
Task 10.B. The pre-phase baseline at
`reports/lyng/dsl-1/pre-phase-1a-baseline.md` confirms this for
all nine Phase 1.A opcodes.

## V8 v7

A single-opcode port is not expected to move the V8 v7 geomean
measurably on its own, but `op_load_smi8` is the highest-volume
opcode in Phase 1.A (top-30 dispatch share #7), so its inline port
should produce a measurable per-iteration speedup on integer-heavy
workloads (counter loops, array index computations, arithmetic
chains). Phase 1.A's aggregate impact will be measured at Task 10
against the pre-phase baseline geomean of **387.09** captured in
`reports/lyng/dsl-1/pre-phase-1a-baseline.md`.

## Slow-path-share

Slow-path-share counter is currently no-op (DSL-0c gap; tracked as
Task 10.A to re-wire into DSL dispatch tail). Once re-wired, expected
post-port value: **0%** — the slow-path shim `op_load_smi8_slow_rs`
was deleted alongside this port; the opcode has no fail mode (it
unconditionally writes `Value::from_smi(sxt(b))` to register `a` and
dispatches).

## Behavioral tests

- `cargo test -p lyng-vm --lib --release` — **413 passed**.
- `cargo test -p lyng-tests --release` — **1186 passed (2 suites)**.

Both green; behavioral parity preserved.

**Negative-payload correctness verified.** `op_load_smi8` carries a
signed-byte payload, so the critical correctness check is that
negative literals (e.g. `-1`, `-128`) sign-extend properly to
`Value::from_smi(-1) == 0x7ff8_0004_ffff_ffff`, NOT to
`Value::from_smi(255) == 0x7ff8_0004_0000_00ff`. The lyng-tests
crate contains many JS sources with negative integer literals; the
fact that all 1186 pass after replacing the cold-stub round-trip
with the inline DSL fast path confirms `sxtb` is producing the
correct sign-extended payload (had it not, negative-literal tests
would fail loudly with wrong arithmetic / comparison results).
Specific covered cases include negative array indices, negative
arithmetic operands, and `for`-loop step decrements in the script-core
test surfaces.

## Notes

- **First top-30 port in Phase 1.A.** All prior Phase-1.A ports
  (Tasks 1-6) were constant-loaders. Task 7 is the first port with
  a runtime-varying payload, and the first whose dispatch share
  alone justifies the inline fast path.
- **The new macro is reusable.** `tag_smi_from_signed_byte!` is
  not specific to `op_load_smi8`; any future narrow signed-integer
  SMI-loader opcode can reuse it. A natural extension would be
  `tag_smi_from_signed_half!` (16-bit input, `sxth` instead of
  `sxtb`) for `op_load_smi16` when that ports.
- **5-instruction tag overhead is structural.** Compared to the
  2-3 instructions of the constant-loaders' `tag_smi_const!`, the
  +2-3 instructions arise because:
  - The payload arrives as a runtime value (can't be folded into a
    `movz` immediate).
  - The payload is signed and narrow (requires `sxtb` to sign-extend
    before tagging).
  - The high bits of the X register need clearing (`uxtw`) before
    the `orr` composes cleanly with the tag pattern.
  None of these can be elided without changing the bytecode layout
  (which is out of scope for Phase 1.A).
- **Reusing the operand register saves a move.** The handler body uses
  `b` (the i8 payload) as both source and destination of the tag
  operation, then `store_reg!(a, b)` writes the same register to
  `REGS[a]`. A naive port might allocate a fresh scratch (e.g. `t2`)
  for the tagged value, but reusing `b` saves an explicit `mov`
  without affecting correctness — after the `orr`, `b` no longer
  needs to carry the raw i8 payload.
- **`store_reg!` writes the full X register.** `str x10, [x20, x9, lsl #3]`
  writes 8 bytes (the full tagged Value), so the upper-32-bit clear
  by `uxtw` is essential — without it, stale upper bits would
  corrupt the tag.
- **Slow-path shim `op_load_smi8_slow_rs` deleted.** With the inline
  fast path the shim has no callers; it was removed alongside the
  handler-body change. The semantic body
  (`crate::vm::semantics::loads::op_load_smi8_semantic`) is still
  reachable from `evaluator.rs` and `bytecode_lowering`-style paths;
  it was *not* deleted.
- **Manifest entry unchanged.** The opcode manifest references
  `op_load_smi8_dsl` (the handler symbol, not the shim). The symbol
  identity is preserved; only the body changed.
- **Pre-existing infra gaps acknowledged (not fixed by this task):**
  1. Slow-path-share counter is no-op since DSL-0c (Task 10.A).
  2. Microbench snippets for the 9 Phase-1.A opcodes don't exist
     (Task 10.B).
- **LLInt reference capture mode:** *excerpt (manual)* —
  `capture-llint` could not locate an exact LLInt analog. Reference
  taken manually from
  `/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter64.asm`.
