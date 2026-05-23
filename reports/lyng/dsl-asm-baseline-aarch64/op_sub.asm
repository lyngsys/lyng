// DSL-1 Phase 1.C.1 op_sub_dsl handler asm (AArch64), captured from
//   cargo rustc --release -p lyng-vm --lib -- --emit=asm
//
// Symbol: lyng_vm::dsl::handlers::cold::op_sub_dsl
// Source: crates/vm/src/dsl/handlers/cold.rs
// Layout: AbcSlot (a, b, c byte operands + 16-bit feedback slot),
// length = 6 bytes.
//
// Register allocation by the lowerer:
//   a=x9 (dst), b=x10 (lhs), c=x11 (rhs), slot=x12
//   t0=x13 (lhs value), t1=x14 (rhs value), t2=x15 (result)
//   Internal scratch: x16/x17 (used by macros)

op_sub_dsl:
    // decode_abc_slot! prologue
    ldrb    w9,  [x19, #1]               ; w9  = a (dst reg id)
    ldrb    w10, [x19, #2]               ; w10 = b (lhs reg id)
    ldrb    w11, [x19, #3]               ; w11 = c (rhs reg id)
    ldrh    w12, [x19, #4]               ; w12 = slot (feedback slot id)

    // load_reg!(b => t0)
    ldr     x13, [x20, x10, lsl #3]      ; t0 = REGS[lhs]

    // check_smi!(t0, .slow)
    mov     x16, #281470681743360
    movk    x16, #32760, lsl #48
    and     x16, x13, x16
    mov     x17, #17179869184
    movk    x17, #32760, lsl #48
    cmp     x16, x17
    b.ne    Lop_sub_dsl__slow

    // load_reg!(c => t1)
    ldr     x14, [x20, x11, lsl #3]      ; t1 = REGS[rhs]

    // check_smi!(t1, .slow)
    mov     x16, #281470681743360
    movk    x16, #32760, lsl #48
    and     x16, x14, x16
    mov     x17, #17179869184
    movk    x17, #32760, lsl #48
    cmp     x16, x17
    b.ne    Lop_sub_dsl__slow

    // untag_smi!(t0); untag_smi!(t1)
    sxtw    x13, w13
    sxtw    x14, w14

    // sub_smi_overflow!(t0, t1 => t2, .slow)
    subs    w15, w13, w14
    b.vs    Lop_sub_dsl__slow
    sxtw    x15, w15

    // tag_smi!(t2)
    mov     x16, #17179869184
    movk    x16, #32760, lsl #48
    ubfx    x15, x15, #0, #32
    orr     x15, x16, x15

    // store_reg!(a, t2)
    str     x15, [x20, x9, lsl #3]       ; REGS[dst] = t2

    // call_slow!(op_sub_record_smi_rs, args = [slot])
    ldr     x16, [x24, #8]               ; pb_base
    sub     x17, x19, x16                ; pc_offset = PC - pb_base
    str     w17, [x24]                   ; state.frame_pc_offset = pc_offset
    mov     x0, x24                      ; a0 = STATE
    mov     w1, w12                      ; a1 = slot
    bl      _op_sub_record_smi_rs

    // dispatch_after_slow!()
    cbnz    x0, Ltmp18
    ldr     x16, [x24, #8]
    add     x19, x16, x1
    ldr     x20, [x24, #16]
    ldr     x21, [x24, #24]
    ldrb    w8, [x19]
    ldr     x17, [x23, x8, lsl #3]
    br      x17
Ltmp18:
    cmp     x0, #2
    b.eq    Ltmp19
    ldr     w16, [x24]
    ldr     x17, [x24, #8]
    add     x19, x17, x16
    ldr     x20, [x24, #16]
    ldr     x21, [x24, #24]
    ldrb    w8, [x19]
    ldr     x17, [x23, x8, lsl #3]
    br      x17
Ltmp19:
    b       __interpreter_exit

Lop_sub_dsl__slow:
    // call_slow!(op_sub_slow_rs, args = [a, b, c, slot])
    ldr     x16, [x24, #8]               ; pb_base
    sub     x17, x19, x16                ; pc_offset
    str     w17, [x24]
    mov     x0, x24                      ; a0 = STATE
    mov     w1, w9                       ; a1 = dst
    mov     w2, w10                      ; a2 = lhs
    mov     w3, w11                      ; a3 = rhs
    mov     w4, w12                      ; a4 = slot
    bl      _op_sub_slow_rs

    // dispatch_after_slow!()
    cbnz    x0, Ltmp20
    ldr     x16, [x24, #8]
    add     x19, x16, x1
    ldr     x20, [x24, #16]
    ldr     x21, [x24, #24]
    ldrb    w8, [x19]
    ldr     x17, [x23, x8, lsl #3]
    br      x17
Ltmp20:
    cmp     x0, #2
    b.eq    Ltmp21
    ldr     w16, [x24]
    ldr     x17, [x24, #8]
    add     x19, x17, x16
    ldr     x20, [x24, #16]
    ldr     x21, [x24, #24]
    ldrb    w8, [x19]
    ldr     x17, [x23, x8, lsl #3]
    br      x17
Ltmp21:
    b       __interpreter_exit

// Fast-path instruction count (from `op_sub_dsl:` through
// `bl _op_sub_record_smi_rs` inclusive): 36 instructions.
//
// Mirrors op_add's shape byte-for-byte except `subs` (was `adds`)
// for the overflow-detecting subtract. The SMI-tag-check / untag /
// retag / store / record-shim / dispatch fragments are identical.
