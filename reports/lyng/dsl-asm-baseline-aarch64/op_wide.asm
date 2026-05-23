// DSL-0b op_wide handler asm (AArch64).
// Symbol: lyng_js_vm::dsl::handlers::warm::op_wide
// Source: crates/lyng-js/vm/src/dsl/handlers/warm.rs (B45)
// Layout: None (no operands), length = 1 byte.

op_wide:
    // dispatch_prefixed!(kind = 1)
    ldrb    w16, [x24, #48]          ; w16 = state.prefix
    cbnz    w16, 1f                  ; if already prefixed, double-prefix
    mov     w16, #1                  ; Wide = 1
    strb    w16, [x24, #48]          ; state.prefix = Wide
    add     x19, x19, #1             ; advance PC past prefix byte
    ldrb    w8, [x19]                ; w8 = next opcode
    ldr     x17, [x23, x8, lsl #3]   ; look up handler
    br      x17                      ; tail-jump
1:
    // Double-prefix: brk #0 placeholder until
    // op_double_prefix_slow_rs lands.
    brk     #0
