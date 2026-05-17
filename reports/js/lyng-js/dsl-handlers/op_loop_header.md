# `op_loop_header` DSL port (B43)

First warm-category handler — validates `poll_safepoint!` + the
fall-through `dispatch!(advance = 4)` shape against a real backedge
poll site.

## DSL source

`crates/lyng-js/vm/src/dsl/handlers/warm.rs`:

```rust
llint_handler! {
    op_loop_header, layout = Ax, length = 4, |_unused_target_offset| {
        poll_safepoint!(.poll_pending);
        dispatch!(advance = 4);
        .poll_pending:
        call_slow!(op_loop_header_poll_rs, args = []);
        dispatch_after_slow!();
    }
}
```

Fast path: 2-instruction poll (`ldrb` + `cbnz`) + 4-instruction
dispatch (advance / load / lookup / br). Total 6 instructions when
the safepoint is clear. With the operand decode that the lowerer
emits unconditionally, the actual count is 7 (the operand load is
dead in this handler since we never reference `_unused_target_offset`;
LLVM's optimizer should hoist it later if it ever matters).

Slow path: `call_slow!(op_loop_header_poll_rs)` + `dispatch_after_slow!`.

## Slow-path shim

A hand-written `op_loop_header_poll_rs` calls into the new
`crate::dsl::poll::run_poll(PollArgs)` consumer. The consumer is a
stub today — the Vm doesn't yet expose `poll_pending`, `run_incremental_gc_step`,
or `handle_debug_pause_request`. Real implementations land when the
GC + debugger integrations arrive.

## Current asm (AArch64)

See `reports/js/lyng-js/dsl-asm-baseline-aarch64/op_loop_header.asm`.

## LLInt reference

See `reports/js/lyng-js/llint-reference/op_loop_hint.md`.

JSC's `op_loop_hint` shape:

```text
loadi VM::m_traps[t0], t1
btiz t1, .opLoopHintEntry
callSlowPath(_llint_loop_osr)
...
```

Same two-step shape: check trap flag, dispatch or call into slow path.
The Lyng shape is structurally identical.

## Microbench

Not yet captured.

## Behavioral tests

- `tests/dsl_validation_safepoint_loop_header.rs` continues to pass.
