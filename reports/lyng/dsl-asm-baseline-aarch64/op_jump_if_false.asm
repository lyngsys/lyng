// DSL-0b op_jump_if_false handler asm (AArch64).
// Symbol: lyng_vm::dsl::handlers::warm::op_jump_if_false
// Layout: Abx (1-byte condition register + 2-byte i16 delta), length = 4.
//
// Structurally identical to op_jump_if_true.asm; the only difference
// is the shim symbol (_op_jump_if_false_slow_rs) which routes into
// op_jump_if_false_semantic rather than op_jump_if_true_semantic.

op_jump_if_false:
    ldrb    w9,  [x19, #1]
    ldrh    w10, [x19, #2]
    ldr     x16, [x24, #8]
    sub     x17, x19, x16
    str     w17, [x24]
    mov     x0, x24
    mov     w1, w9
    mov     w2, w10
    bl      _op_jump_if_false_slow_rs
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
