# `op_extra_wide` DSL port (B45)

Companion to `op_wide`. See [`op_wide.md`](./op_wide.md) for the
shared `dispatch_prefixed!` analysis — both handlers share the same
backend macro and emit nearly identical asm (kind discriminator is
the only meaningful difference).
