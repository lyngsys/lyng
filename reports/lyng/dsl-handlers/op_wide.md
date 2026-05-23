# `op_wide` / `op_extra_wide` DSL ports (B45)

Two warm prefix-opcode handlers. Both share the `dispatch_prefixed!`
backend macro — the only difference is the kind discriminator (1 for
Wide, 2 for ExtraWide).

## DSL source

`crates/vm/src/dsl/handlers/warm.rs`:

```rust
llint_handler! {
    op_wide, layout = None, length = 1, || {
        dispatch_prefixed!(kind = 1);
    }
}

llint_handler! {
    op_extra_wide, layout = None, length = 1, || {
        dispatch_prefixed!(kind = 2);
    }
}
```

`dispatch_prefixed!` was defined in B25 and already covers the
state-prefix store, single-byte PC advance, and the double-prefix
rejection guard (a `brk #0` placeholder until
`op_double_prefix_slow_rs` lands).

## Current asm (AArch64)

See:
- `reports/lyng/dsl-asm-baseline-aarch64/op_wide.asm`
- `reports/lyng/dsl-asm-baseline-aarch64/op_extra_wide.asm`

Each emits exactly 7 fast-path instructions:

```text
ldrb    w16, [x24, #48]    ; load state.prefix
cbnz    w16, double_prefix  ; reject doubled prefix
mov     w16, #<kind>        ; set discriminator
strb    w16, [x24, #48]     ; state.prefix = <kind>
add     x19, x19, #1        ; advance PC
ldrb    w8, [x19]           ; load next opcode
ldr     x17, [x23, x8, lsl #3] ; look up handler
br      x17                 ; tail-jump
```

## LLInt reference

JSC's `op_wide16` / `op_wide32`:

```text
add 1, PC
loadb [PC, kind, offset] ; dispatch to size-specific handler
```

Same shape — set the prefix metadata, advance past the prefix byte,
dispatch into the underlying opcode's handler. The Lyng DSL emits
one extra `cbnz` + a `brk #0`-guarded slow path to catch
double-prefix invariant violations (validation case 9 in B38).

## Validation cases

- Case 7: Wide prefix decode (`tests/dsl_validation_prefix_wide.rs`)
- Case 8: ExtraWide prefix decode (`tests/dsl_validation_prefix_extra_wide.rs`)
- Case 9: Double-prefix rejection (`tests/dsl_validation_prefix_double.rs`)

All three continue to pass with the new handler symbols.

## Microbench

Not yet captured.
