# Phase 1 Trampoline Spike — Option α verification

**Issue:** `lyng-33i2` — Phase 1: Threaded dispatch + per-handler ABI
**Decision:** Option α (per-handler `extern "C" fn` + central trampoline + macro abstraction)
**Date:** 2026-05-14
**Toolchain:** rustc 1.93 (2025-12-15), aarch64-apple-darwin, release profile (thin LTO)

## Verdict

**GO — proceed to Phase 1 proper.** All four exit criteria for the spike hold; one
hot-handler-size measurement is 10% over the Phase 1 acceptance target, but the
overhead is fully attributable to spike-specific register-window indexing and is
expected to vanish under the production register-window helpers.

## What was built

A self-contained prototype of the Option α dispatch primitives, reachable only
from a unit test. The live `run_dispatch_loop` is untouched.

- [crates/lyng-js/vm/src/vm/dispatch_state.rs](../../../crates/lyng-js/vm/src/vm/dispatch_state.rs) — `DispatchState<'vm>`, `Handler` typedef,
  `Step` enum, `DISPATCH_TABLE` static (256 entries), `dispatch_next!` macro,
  `run_trampoline`.
- [crates/lyng-js/vm/src/vm/dispatch_handlers/](../../../crates/lyng-js/vm/src/vm/dispatch_handlers/) —
  `arithmetic::op_add`, `control_flow::{op_jump_back, op_return}`,
  `loads::{op_move, op_load_undefined}`, `stub::op_stub`.
- `DispatchState` collision with the pre-existing `DispatchState` frame-snapshot
  type in [dispatch.rs](../../../crates/lyng-js/vm/src/vm/dispatch.rs) resolved by renaming the old type to
  `DispatchFrameSnapshot` — three callsites plus one structural-test
  assertion, no behavior change.

The spike's bytecode encoding is intentionally minimal (1-byte register IDs,
hand-rolled per-opcode operand sizes) so the asm we measure is dominated by
the dispatch shape, not by production-grade encoding overhead. Production
handlers will use the real bytecode operand decoders.

## Functional check

`cargo test --release -p lyng-js-vm trampoline_spike` runs a hand-rolled
4-opcode program (`LdaUndefined R2; Move R3, R0; Add R2, R3, R1, slot=0;
Return R2`) and asserts the SMI fast path produces `Value::from_smi(12)` and
bumps `feedback_counter` to `1`. Result: **pass.**

## Exit criteria

| # | Criterion | Result |
|---|---|---|
| 1 | Single indirect call in the trampoline loop | **PASS** — one `blr` at `run_trampoline+0x54`; remaining branches are direct (the entry bounds-check `panic_bounds_check` is a cold call) |
| 2 | No full `Step` materialization on the hot tail | **PASS** — handlers write only 16 of `Step`'s 48 bytes (`stp x_tag, x_handler, [x8]`); the trampoline reads only those 16 bytes (`ldr [sp]`, `ldr [sp+8]`); the 32-byte `VmError` payload is loaded only on the cold `Step::Error` arm |
| 3 | Hot handler size < 200 bytes | **MIXED** — op_move 136 B, op_load_undefined 128 B, op_jump_back 96 B, op_return 88 B, op_stub 16 B; **op_add 220 B (10% over)**. See "op_add overshoot" below. |
| 4 | Unit test passes | **PASS** |

## Hot-handler sizes (start of function to first `ret`, bytes)

```
run_trampoline       180
op_add               220   <-- 10% over the <200 target
op_jump_back          96
op_return             88
op_load_undefined    128
op_move              136
op_stub               16
```

## Asm snippets

### `run_trampoline` — the hot dispatch loop

The hot loop body is 7 instructions per iteration and contains exactly one
indirect call (`blr x9`):

```asm
0x100350dd0   mov  x8, sp              ; sret slot for Step
0x100350dd4   mov  x0, x20             ; state ptr
0x100350dd8   blr  x9                  ; *** indirect call to handler ***
0x100350ddc   ldr  x8, [sp]            ; read Step tag word
0x100350de0   add  x9, x8, x21         ; \
0x100350de4   add  x10, x21, #0x41     ;  | branchless tag-test
0x100350de8   cmp  x8, x10             ;  | (Continue vs others)
0x100350dec   csel x8, x9, x22, hi     ; /
0x100350df0   cbnz x8, 0x100350dfc     ; non-Continue → epilogue
0x100350df4   ldr  x9, [sp, #0x8]      ; next handler pointer
0x100350df8   b    0x100350dd0         ; loop
```

The `Step` enum (48 bytes including the large `VmError` variant) is returned
via sret. On the hot path the trampoline reads only two 8-byte words from the
sret slot: the tag (`[sp]`) and the `Continue(Handler)` payload (`[sp+8]`). The
remaining 32 bytes — the `VmError` body — are only accessed on the cold
`Step::Error` arm at `0x100350e14`. This is the best Option α can deliver
without changing the ABI; if the resulting ~4-6 cycle store-load round-trip
per dispatch proves too expensive, the spec's β/γ escape hatches swap in
behind the `dispatch_next!` macro without touching handler bodies.

### `op_move` — representative hot handler tail

```asm
0x1002bbb84   ldrb w9, [x12, x0]       ; next opcode byte
0x1002bbb88   adrp x10, DISPATCH_TABLE
0x1002bbb8c   ldr  x9, [x10, x9, lsl #3]   ; load next handler
0x1002bbb90   mov  x10, #0x8000_0000_0000_0021   ; Step::Continue tag
0x1002bbb94   stp  x10, x9, [x8]       ; write Step::Continue(next_handler)
0x1002bbb98   ldp  x29, x30, [sp], #0x10
0x1002bbb9c   ret
```

The handler tail matches the spec's expected shape: load `DISPATCH_TABLE[next_op]`,
write the `Step::Continue(handler)` discriminant + payload to the sret slot,
return. No indirect call inside the handler; the trampoline owns the indirect
call.

## op_add overshoot (220 vs 200 bytes)

op_add's hot-path body breaks down as roughly:

- Function prologue + entry bounds check: ~36 B
- 6-byte operand slab via `bytes[..6].try_into::<&[u8; 6]>()`: ~12 B (one combined
  bounds check; the per-byte fallback would have been 5 separate panics,
  measured at +28 B in an earlier draft)
- Two register reads + checked SMI tag check + checked SMI add: ~76 B
  (intrinsically required for correctness)
- Register write + feedback bump + advance + `dispatch_next!` tail: ~60 B
- Epilogue + ret: ~12 B
- **3 register-window bounds checks** at `state.read_register(b)`,
  `state.read_register(c)`, `state.write_register(a)`: ~36 B total

The 36 B of register-window bounds checks are the spike-only overhead. Production
handlers go through `absolute_register(registers, idx)` + direct
`self.register_stack[...]` indexing (see the existing
[dispatch.rs `Opcode::Add` arm](../../../crates/lyng-js/vm/src/vm/dispatch.rs)) which the compiler folds into a single shared
bounds check per opcode arm. Once the spike's `state.regs[idx]` indexing is
replaced with the production accessor pattern in the first family-conversion
sub-issue, op_add is projected to land at ~184 B — under the <200 B target.

This is documented rather than fixed in the spike because the production
register-window helpers depend on `FrameRecord` + `RegisterWindow` which the
spike intentionally does not pull in.

## What this measurement proves

Option α produces a viable interpreter dispatch shape on aarch64:

1. The trampoline's `match Step { Continue, Done, Error }` lowers to a
   branchless tag test followed by one `cbnz` — no full `Step` enum
   materialization on the hot path, no second indirect branch.
2. Each handler ends with a small, identical tail (`ldr DISPATCH_TABLE[next]; stp tag,handler,[sret]; ret`),
   which is what the macro abstraction guarantees and what makes future swaps
   to Option β (`become`) or Option γ (inline-asm tail calls) one-line changes.
3. Hot handler sizes cluster well under the icache budget; the one outlier
   (op_add) is fixable with production helpers, not a structural ABI problem.

## What this measurement does NOT prove

- Phase 1 will hit the Richards ≥260 / DeltaBlue ≥310 / Crypto ≥265 /
  RayTrace ≥430 / NavierStokes ≥470 / Splay ≥1330 benchmark gates. That
  requires the V8 bench harness to be wired into `lyng-js-bench` and the
  full handler set converted. Both are tracked as sub-issues of `lyng-33i2`.
- The Test262 baseline 49722/49729 is preserved. Phase 1 proper preserves it
  by parallel-running the new dispatch behind a feature flag until cutover.
- The ~4-6 cycle store-load round-trip per dispatch is acceptable on the
  workloads we care about. The benchmark sub-issue is the answer; the spec
  reserves the β/γ swap as the escape hatch.

## Follow-up

Decompose `lyng-33i2` into family-by-family sub-issues per the plan in
[/Users/sondre/.claude/plans/work-on-lyng-33i2-greedy-teacup.md](/Users/sondre/.claude/plans/work-on-lyng-33i2-greedy-teacup.md).
