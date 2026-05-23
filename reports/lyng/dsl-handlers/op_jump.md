# `op_jump` DSL port (B41)

## DSL source

`crates/lyng/vm/src/dsl/handlers/hot.rs`:

```rust
llint_handler! {
    op_jump, layout = Ax, length = 4, |offset| {
        call_slow!(op_jump_slow_rs, args = [offset]);
        dispatch_after_slow!();
    }
}
```

Note: `Jump` is the 4-byte form (opcode + i24 sign-extended delta).
The DSL's `decode_ax!` reads a 4-byte word at PC+1 which pulls in
one trailing byte from the next opcode; the slow-path shim masks
to the low 24 bits and sign-extends explicitly.

The plan's optimized version inlines PC computation and a backward-poll
fast path, but `dispatch!(jump_to = ...)` isn't supported by the
backend yet (the asm would need to compute `pb_base + entry_offset +
instruction_len + delta` inline, with an embedded `tbnz` branch on the
sign bit of `offset`). For DSL-0b the slow-path-only form lets us link
the handler and run validation tests; B-series follow-ups will inline
the forward path.

## Slow-path shim

A hand-written `op_jump_slow_rs` adapts the u32 raw operand (an i32
sign-extended through the SDK) into `OpJumpArgs { delta, instruction_len }`
and calls into the existing `op_jump_semantic`. The semantic body
already polls the incremental-mark safepoint on `delta < 0`, so the
"backward-edge poll" gate is honored.

## Current asm (AArch64)

See `reports/lyng/dsl-asm-baseline-aarch64/op_jump.asm`.

13 instructions on the fast path through the slow-path call:
- 1 `ldr` (operand decode)
- 5 (call_slow setup + bl)
- 4 (dispatch_after_slow Continue path: cbnz/ldr/add/ldrb/ldr/br)
- 7 (Refresh / Exit handling, lifted out of the hot path)

## LLInt reference

See `reports/lyng/llint-reference/op_jmp.md`.

JSC's `op_jmp` is a single inline branch — `dispatch(targetOffset)`.
For DSL-0b we're paying ~10 extra instructions for the slow-path call;
this is acceptable for the dead-code stage but should be inlined in
DSL-1's hot-path optimization pass.

## Microbench

Not yet captured (handler is dead code; alpha dispatch active).

## Behavioral tests

- `tests/dsl_validation_safepoint_backward_jump.rs` continues to pass.
- Alpha-dispatch `op_jump` integration tests continue to pass.
