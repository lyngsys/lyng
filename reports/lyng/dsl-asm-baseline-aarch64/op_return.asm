// DSL-0b op_return handler asm (AArch64).
// Symbol: lyng_vm::dsl::handlers::hot::op_return
// Source: crates/lyng/vm/src/dsl/handlers/hot.rs (B42)
// Layout: A (single byte operand at PC+1), length = 2 bytes.
//
// op_return is frame-transitioning — always returns Refresh / ExitDone /
// ExitError. The DSL body is a thin slow-path bridge.

op_return:
    // decode_a! prologue
    ldrb    w9, [x19, #1]          ; src register id

    // call_slow!(op_return_slow_rs, args = [src])
    ldr     x16, [x24, #8]         ; pb_base
    sub     x17, x19, x16
    str     w17, [x24]             ; state.frame_pc_offset
    mov     x0, x24                ; a0 = STATE
    mov     w1, w9                 ; a1 = src
    bl      _op_return_slow_rs

    // dispatch_after_slow!()
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
