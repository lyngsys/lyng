// DSL-0b op_jump_if_true8 handler asm (AArch64).
// Symbol: lyng_vm::dsl::handlers::warm::op_jump_if_true8
// Layout: Ab in the DSL (1-byte condition register + 1-byte i8 delta),
// length = 3.

op_jump_if_true8:
    ldrb    w9,  [x19, #1]
    ldrb    w10, [x19, #2]
    ldr     x16, [x24, #8]
    sub     x17, x19, x16
    str     w17, [x24]
    mov     x0, x24
    mov     w1, w9
    mov     w2, w10
    bl      _op_jump_if_true8_slow_rs
    cbnz    x0, 1f
    ldr     x16, [x24, #8]
    add     x19, x16, x1
    ldrb    w8, [x19]
    ldr     x17, [x23, x8, lsl #3]
    br      x17
1:
    cmp     x0, #2
    b.eq    2f
    ldr     w16, [x24]
    ldr     x17, [x24, #8]
    add     x19, x17, x16
    ldr     x20, [x24, #16]
    ldr     x21, [x24, #24]
    ldrb    w8, [x19]
    ldr     x17, [x23, x8, lsl #3]
    br      x17
2:
    b       __interpreter_exit
