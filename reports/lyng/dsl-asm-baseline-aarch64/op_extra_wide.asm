// DSL-0b op_extra_wide handler asm (AArch64).
// Symbol: lyng_vm::dsl::handlers::warm::op_extra_wide
// Source: crates/vm/src/dsl/handlers/warm.rs (B45)
// Layout: None (no operands), length = 1 byte.

op_extra_wide:
    // dispatch_prefixed!(kind = 2)
    ldrb    w16, [x24, #48]          ; w16 = state.prefix
    cbnz    w16, 1f                  ; double-prefix slow path
    mov     w16, #2                  ; ExtraWide = 2
    strb    w16, [x24, #48]          ; state.prefix = ExtraWide
    add     x19, x19, #1             ; advance PC past prefix byte
    ldrb    w8, [x19]                ; next opcode
    ldr     x17, [x23, x8, lsl #3]   ; look up handler
    br      x17                      ; tail-jump
1:
    brk     #0
