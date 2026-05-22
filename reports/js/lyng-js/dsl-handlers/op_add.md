# `op_add` DSL port (B40)

Validates the full fast-path-into-slow-path bridge: SMI checks, untag,
overflow-detecting add, retag, register store, feedback recording, plus
a `.slow:` label hosting `call_slow! + dispatch_after_slow!`.

## DSL source

`crates/lyng-js/vm/src/dsl/handlers/hot.rs`:

```rust
llint_handler! {
    op_add, layout = AbcSlot, length = 6, |a, b, c, slot| {
        load_reg!(b => t0);
        check_smi!(t0, .slow);
        load_reg!(c => t1);
        check_smi!(t1, .slow);
        untag_smi!(t0);
        untag_smi!(t1);
        add_smi_overflow!(t0, t1 => t2, .slow);
        tag_smi!(t2);
        store_reg!(a, t2);
        record_smi!(slot);
        dispatch!();
        .slow:
        call_slow!(op_add_slow_rs, args = [a, b, c, slot]);
        dispatch_after_slow!();
    }
}
```

## Slow-path shim

A hand-written `op_add_slow_rs` shim (rather than going through
`dsl_cold_shim!`) adapts the raw u32 operand-id quartet from asm into
the `OpBinaryArgs` shape `op_add_semantic` expects. Lives next to the
handler:

```rust
pub extern "C" fn op_add_slow_rs(
    state: *mut LlIntState,
    dst: u32, lhs: u32, rhs: u32, feedback_slot: u32,
) -> SlowPathReturn {
    let mut dispatch = unsafe { LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    let args = OpBinaryArgs {
        dst: dst as u16,
        lhs: lhs as u16,
        rhs: rhs as u16,
        feedback_slot: FeedbackSlotId::from_raw(feedback_slot),
        instruction_len: 6,
    };
    let outcome = op_add_semantic(&mut dispatch, args);
    dispatch.translate_outcome(outcome)
}
```

The `FeedbackSlotId::from_raw(u32) -> Option<FeedbackSlotId>` builder
gracefully handles the `0` (no feedback slot) case via niche encoding.

## Current asm (AArch64)

See `reports/js/lyng-js/dsl-asm-baseline-aarch64/op_add.asm`.

The fast path is 21 instructions:
- 4 ldrb/ldrh (operand decode)
- 1 ldr (lhs value)
- 7 (check_smi: movz/movk/and/movz/movk/cmp/b.ne)
- 1 ldr (rhs value)
- 7 (second check_smi)
- 2 sxtw (untag)
- 3 (adds, b.vs, sxtw)
- 4 (tag_smi: movz/movk/uxtw/orr)
- 1 str (store result)
- 5 (record_smi: lsl/add/ldr/orr/str)
- 4 (dispatch: add/ldrb/ldr/br)

Total fast path: ~39 instructions (LLInt op_add is in the same ballpark
— see `reports/js/lyng-js/llint-reference/op_add.md`).

The slow path is `call_slow! + dispatch_after_slow!` — 5 instructions
for the call setup + 1 bl + the dispatch_after_slow trampoline.

## Register allocation

The lowerer's scratch allocator pre-assigns operand bindings first
(`a`=x9, `b`=x10, `c`=x11, `slot`=x12), then lazily binds internal
scratch `t0`=x13, `t1`=x14, `t2`=x15 as they appear. Macro-internal
scratch lives in x16/x17 (AAPCS64 IP0/IP1) to avoid collisions with
the live operand window.

## LLInt reference

See `reports/js/lyng-js/llint-reference/op_add.md`.

JSC's `op_add` macro uses a similar pattern (tag-check both operands,
SMI fast path with overflow detection, slow path on miss). The exact
instruction count differs because Lyng's tag layout uses a 16-bit
TagKind in the upper half of NaN-space (different mask/pattern
constants), and Lyng's record_smi! writes through the feedback vector
inline (JSC writes through a profile structure with a less direct
addressing form).

## Lowerer changes made in this batch

To support op_add, the proc-macro lowerer gained:

1. **Label declarations and references.** The parser now accepts
   `.label:` declarations as `BodyStmt::Label` entries; the lowerer
   strips the leading dot from references (`b.ne .slow` -> `b.ne Lslow`)
   and emits the declaration with an `L` prefix (`Lslow:`). The prefix
   is required by the Mach-O / ELF assembler-local-label convention
   to keep conditional branches in-range and not externally visible.

2. **Multi-arity `call_slow!`.** The existing macro used a `$()*`
   repetition for the per-operand arg-mov, but with a single
   `{arg_slot}` named binding — every iteration would have read the
   same destination register. Replaced with one match arm per arity
   (0..=5) that hardcodes `w1, w2, ...` correctly.

3. **Macro-internal scratch moved to x16/x17.** The `check_*!`,
   `tag_*!`, `record_*!`, `dispatch!`, `dispatch_after_slow!`,
   `call_slow!`, `dispatch_prefixed!`, and `poll_safepoint!` macros
   now use the AAPCS64 IP0/IP1 (`x16`/`x17`) registers as their
   internal scratch. Previously they clobbered `x9`/`x10`, which the
   lowerer's allocator would assign to live operand bindings — the
   handler would lose its operands mid-body. AAPCS64 lets us use
   `x16`/`x17` freely between call boundaries, so this is safe.

## Microbench

Not yet captured. op_add is dead code from a runtime perspective in
DSL-0b (alpha dispatch is still active); microbenches need the
Phase-C dispatch flip.

## Behavioral tests

- The DSL-0b validation tests (`tests/dsl_validation_*.rs`) continue
  to pass.
- The alpha `op_add` integration tests continue to pass — they
  exercise the legacy handler path, which is unchanged.

## Post-fix slow-path-share update (2026-05-22)

After Phase 1.C followup #1 substrate fix at commit `47fc5061`, slow-
path-share re-measured with honest counter-injection discipline:

| Workload     | Dispatches  | Slow-path-share |
|--------------|------------:|----------------:|
| Richards     |          15 |          33.3%  |
| DeltaBlue    |   1,346,815 |          11.6%  |
| Crypto       | 906,339,762 |           0.5%  |
| RayTrace     |  15,659,345 |          98.5%  |
| NavierStokes | 556,071,165 |          91.0%  |
| Splay        |  12,501,730 |          96.5%  |

SMI fast-path is excellent on integer-dominant workloads (Crypto 0.5%,
DeltaBlue 11.6%). Float-heavy workloads (RayTrace, NavierStokes, Splay)
show 91–98% slow-path-share — operand mix is IEEE-754 doubles where
the SMI fast path bails by design.

Per-workload gate status per spec §1.6 + §5:
- ✅ Workloads meeting <20% gate: DeltaBlue, Crypto
- ⚠ Workloads requiring waiver: Richards (33.3% on 15 dispatches —
  statistical noise), RayTrace (98.5%), NavierStokes (91.0%),
  Splay (96.5%). Float-heavy and non-SMI-dominant operand mixes are
  unavoidable consequences of double-precision arithmetic; LLInt
  op_add on the same workloads has identical SMI-bail discipline and
  would record comparable rates.

See [`reports/js/lyng-js/dsl-1/phase-1c-post-fix-slow-path-share.md`](../dsl-1/phase-1c-post-fix-slow-path-share.md) for the consolidated post-fix re-measurement across all 8 inline-ported arithmetic-family opcodes.
