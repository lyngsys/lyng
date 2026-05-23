# `op_return` DSL port (B42)

## DSL source

`crates/lyng/vm/src/dsl/handlers/hot.rs`:

```rust
llint_handler! {
    op_return, layout = Ax, length = 4, |src| {
        call_slow!(op_return_slow_rs, args = [src]);
        dispatch_after_slow!();
    }
}
```

Note: `Return` uses the 4-byte Ax form (opcode + 24-bit register
operand). The plan's original example specified `layout = A, length =
2`, but the actual bytecode encoding is `Ax`/`length = 4`. The slow-
path shim masks to the low 24 bits before passing to
`op_return_semantic`.

`op_return` is frame-transitioning — every invocation returns
`Refresh` (nested return), `ExitDone` (root frame returned), or
`ExitError` (abrupt completion). The fast path is exactly the slow
path: there's no SMI/inline win to fish for, so we go straight to
the slow-path bridge.

## Slow-path shim

`op_return_slow_rs` adapts the single u32 raw operand into
`OpReturnArgs { register: src as u16 }` and calls into
`op_return_semantic`, which handles the actual frame transition.

## Current asm (AArch64)

See `reports/lyng/dsl-asm-baseline-aarch64/op_return.asm`.

13 instructions, same shape as op_jump.

## LLInt reference

See `reports/lyng/llint-reference/op_ret.md` (or equivalent).

JSC's op_ret is short: load the return value, pop the frame, dispatch
into the caller. Lyng's slow-path bridge has the overhead of going
through Rust — about 5 extra instructions vs an inline return.

The plan flags this trade-off in B42 step 5: "If op_return is more
than 5% slower than the alpha path, file a follow-up to introduce
the same-code-unit fast-return shortcut from design §6 — but do NOT
block DSL-0b on it."

## Microbench

Not yet captured.

## Behavioral tests

- All existing op_return integration tests continue to pass via the
  alpha dispatch.
