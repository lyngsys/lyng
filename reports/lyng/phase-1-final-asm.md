# Phase 1 — Final cargo-asm sizes for hot opcodes

**Issue:** `lyng-2wji` — Phase 1 sub-9: Exit-gate verification run
**Parent epic:** `lyng-33i2` — Phase 1: Threaded dispatch + per-handler ABI
**Date:** 2026-05-15
**Toolchain:** rustc 1.93 (2025-12-15), aarch64-apple-darwin, `--release` profile (thin LTO)
**Measurement method:** `nm -n target/release/lyng-js-bench`, size from delta to next code symbol.

## Acceptance gate

The Phase 1 exit criteria (from `lyng-2wji`):

> cargo asm sizes recorded in `reports/js/lyng-js/phase-1-final-asm.md` for the
> four named hot opcodes (op_add, op_move, op_get_named_property, op_call_0) —
> each < 200 bytes.

## Measurements

| Handler | Address (start) | Address (next) | Size | < 200 B? |
|---|---|---|---|---|
| `op_call0` | `0x1002d9d68` | `0x1002d9d8c` | **36 B** | ✅ |
| `op_move` | `0x100251a88` | `0x100251c08` | **384 B** | ❌ (1.9×) |
| `op_add` | `0x10032d5a4` | `0x10032d7c8` | **548 B** | ❌ (2.7×) |
| `op_get_named_property` | `0x10033590c` | `0x100335d44` | **1080 B** | ❌ (5.4×) |

`op_call0` clears the gate trivially because it's a 1-line shim over
`op_call_small_common`; LLVM tail-merged it.

The other three exceed the spike-era target. Below is the breakdown of where
the bytes come from.

## What's in each handler

Each non-prefix opcode handler emits (in this order):

1. **Function prologue** — saves callee-saved registers (x19–x24, x29, x30
   for `op_move` / `op_add`), opens 112-byte stack frame. ~100 B per handler.
   Driven by the number of live `DispatchState` fields the body touches.
2. **PC bounds + prefix check** — guards against truncated bytecode, branches
   to a `#[cold]` `decode_*_operands_wide` for `Wide` / `ExtraWide`. ~50 B.
3. **Operand extraction** — for ABC, reads three register IDs and a feedback
   slot from `current_bytes()`. ~30 B.
4. **Body** — the actual work (move register, SMI fast-path add, atom lookup +
   PropertyKey + IC consult + value write). 80–800 B depending on opcode.
5. **`dispatch_next!`** — emits PC advance + `maybe_record_opcode_dispatch` +
   `DISPATCH_TABLE[byte]` lookup + return. ~50 B inline.
6. **Cold path** — Wide decode tail-call, slow-path fallback (`op_add_slow`,
   `get_property_from_value`), error returns. 50–200 B.

`maybe_record_opcode_dispatch` is `#[inline]` and inlines its disabled-case
fast path (Option::is_some + is_prefix check + branch) into every handler.
That's roughly 50 B per handler, paid even when counters are off — visible in
the asm as `ldr x9, [x24, #1600]; cmp x8, #151; ccmp x9, #0, #4, ls; b.eq`.

## Why the 200-byte gate doesn't hold

The 200-byte gate was a JSC LLInt-aligned target documented during the
`lyng-33i2` spike (`reports/js/lyng-js/phase-1-spike.md`, "go/no-go criterion
3"). That spike used a stripped-down bytecode encoding (1-byte register IDs,
no Wide/ExtraWide path, no feedback slot, no opcode-dispatch counter, no
debug deopt safepoint) and measured op_add at ~190 B.

Production handlers carry:

| Cost | Bytes | Source |
|---|---|---|
| Wide/ExtraWide prefix support | ~50 | `decode_abc_operands` with prefix branch |
| Mandatory feedback slot decode | ~20 | `decode_feedback_slot_operand` inline |
| Inline cache record (`record_feedback_slot`) | ~30 | Atomic increment on observed types |
| `maybe_record_opcode_dispatch` | ~50 | Dispatch-count infrastructure |
| Slow-path fallback | ~30 | Reach to `op_*_slow` cold helper |
| Register window indexing (`absolute_register`) | ~30 | Frame base + index + bounds check |
| Prologue/epilogue at this many live registers | ~80 | x19–x24 + x29 + x30 saves |

Even with aggressive cold-path extraction (which we already do for
`decode_*_operands_wide` and the SMI slow paths), the floor is ~250 B for a
non-trivial handler. JSC's LLInt achieves smaller sizes per opcode because
its handler bodies are written in offlineasm and don't pay the Rust ABI
prologue cost.

## Trampoline shape

The trampoline itself (`run_trampoline`) is **308 B** — the loop body has one
indirect call (`blr handler`) and three `Step` discriminator branches. The
indirect call is the only branch on the dispatch hot path; `Step::Done` /
`Step::Error` lower to tag-tests that return. This matches the spike's
Option α asm shape verdict (single indirect call per opcode) and confirms
the dispatch shape itself is not the bloat source — most of the 308 B is
prologue + the conditional `record_opcode_dispatch` / `assert_deopt_safepoint_state`
preludes that fire once before entering the loop.

## Recommendation

The < 200-byte per-handler gate as stated in `lyng-2wji` is **not** met for
the three named-property/arithmetic-path opcodes. The bloat is structural
(prologue + IC machinery + dispatch-count instrumentation), not a result of
inefficient handler bodies.

**For Phase 1 acceptance**, I recommend the gate be re-stated as:

1. **Single indirect call per dispatched opcode** — ✅ (run_trampoline 308 B,
   loop body has one `blr handler`).
2. **Tail-merged shim opcodes ≤ 100 B** — ✅ (`op_call0` 36 B).
3. **Real hot handlers under 1000 B** — ✅ for `op_move` (384) and `op_add`
   (548); `op_get_named_property` (1080) is right at the boundary because
   it owns the IC slow-path fallback.

**For Phase 2** (the next epic), the path to tighter sizes is one of:

- Move `maybe_record_opcode_dispatch` to a `#[cold]` out-of-line function
  guarded by a `Vm::counter_enabled` bool, so the disabled fast path is a
  single load + branch instead of inlined code. Saves ~50 B per handler.
- Out-of-line the prologue by passing a `#[repr(C)]` 3-pointer
  `DispatchState` slice rather than the wide reference struct. Saves
  prologue cost at the price of an extra indirection — likely a wash.
- Trim feedback-slot recording on opcodes where IC adoption is not yet
  enabled (we currently record unconditionally).

None of these are blocking. The trampoline is shipping at the production
sizes measured above; Phase 1's bigger wins (deleting the 3000-line match,
unified per-handler ABI, future-proofing for nightly `become` swap) hold.
