// DSL-0b op_add handler asm (AArch64), captured from
//   cargo rustc --release -p lyng-vm --lib -- --emit=asm
//
// Symbol: lyng_vm::dsl::handlers::hot::op_add
// Source: crates/vm/src/dsl/handlers/hot.rs (B40)
// Layout: AbcSlot (a, b, c byte operands + 16-bit feedback slot),
// length = 6 bytes.
//
// Register allocation by the lowerer:
//   a=x9 (dst), b=x10 (lhs), c=x11 (rhs), slot=x12
//   t0=x13 (lhs value), t1=x14 (rhs value), t2=x15 (result)
//   Internal scratch: x16/x17 (used by macros)

op_add:
    // decode_abc_slot! prologue
    ldrb    w9, [x19, #1]                ; w9  = a (dst reg id)
    ldrb    w10, [x19, #2]               ; w10 = b (lhs reg id)
    ldrb    w11, [x19, #3]               ; w11 = c (rhs reg id)
    ldrh    w12, [x19, #4]               ; w12 = slot (feedback slot id)

    // load_reg!(b => t0)
    ldr     x13, [x20, x10, lsl #3]      ; t0 = REGS[lhs]

    // check_smi!(t0, .slow)
    movz    x16, #0xffff, lsl #32
    movk    x16, #0x7ff8, lsl #48
    and     x16, x13, x16
    movz    x17, #0x4, lsl #32
    movk    x17, #0x7ff8, lsl #48
    cmp     x16, x17
    b.ne    Lslow

    // load_reg!(c => t1)
    ldr     x14, [x20, x11, lsl #3]      ; t1 = REGS[rhs]

    // check_smi!(t1, .slow)
    movz    x16, #0xffff, lsl #32
    movk    x16, #0x7ff8, lsl #48
    and     x16, x14, x16
    movz    x17, #0x4, lsl #32
    movk    x17, #0x7ff8, lsl #48
    cmp     x16, x17
    b.ne    Lslow

    // untag_smi!(t0); untag_smi!(t1)
    sxtw    x13, w13
    sxtw    x14, w14

    // add_smi_overflow!(t0, t1 => t2, .slow)
    adds    w15, w13, w14
    b.vs    Lslow
    sxtw    x15, w15

    // tag_smi!(t2)
    movz    x16, #0x4, lsl #32
    movk    x16, #0x7ff8, lsl #48
    uxtw    x15, w15
    orr     x15, x16, x15

    // store_reg!(a, t2)
    str     x15, [x20, x9, lsl #3]       ; REGS[dst] = t2

    // record_smi!(slot)
    lsl     x16, x12, #6                  ; entry_stride_shift = 6
    add     x16, x21, x16
    ldr     w17, [x16]                    ; entry_observed = 0
    orr     w17, w17, #0x1
    str     w17, [x16]

    // dispatch!()
    add     x19, x19, #6
    ldrb    w8, [x19]
    ldr     x16, [x23, x8, lsl #3]
    br      x16

Lslow:
    // call_slow!(op_add_slow_rs, args = [a, b, c, slot])
    ldr     x16, [x24, #8]                ; pb_base
    sub     x17, x19, x16                 ; pc_offset = PC - pb_base
    str     w17, [x24]                    ; state.frame_pc_offset = pc_offset
    mov     x0, x24                       ; a0 = STATE
    mov     w1, w9                        ; a1 = dst
    mov     w2, w10                       ; a2 = lhs
    mov     w3, w11                       ; a3 = rhs
    mov     w4, w12                       ; a4 = slot
    bl      _op_add_slow_rs

    // dispatch_after_slow!()
    cbnz    x0, Lunusual
    ldr     x16, [x24, #8]
    add     x19, x16, x1
    ldrb    w8, [x19]
    ldr     x17, [x23, x8, lsl #3]
    br      x17
Lunusual:
    cmp     x0, #2
    b.eq    Lexit
    ldr     w16, [x24]
    ldr     x17, [x24, #8]
    add     x19, x17, x16
    ldr     x20, [x24, #16]
    ldr     x21, [x24, #24]
    ldrb    w8, [x19]
    ldr     x17, [x23, x8, lsl #3]
    br      x17
Lexit:
    b       __interpreter_exit
