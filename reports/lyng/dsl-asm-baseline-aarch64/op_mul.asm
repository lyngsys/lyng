// DSL-1 Phase 1.C.1 op_mul_dsl handler asm (AArch64), captured from
//   cargo rustc --release -p lyng-js-vm --lib -- --emit=asm
//
// Symbol: lyng_js_vm::dsl::handlers::cold::op_mul_dsl
// Source: crates/lyng-js/vm/src/dsl/handlers/cold.rs
// Layout: AbcSlot (a, b, c byte operands + 16-bit feedback slot),
// length = 6 bytes.
//
// Register allocation by the lowerer:
//   a=x9 (dst), b=x10 (lhs), c=x11 (rhs), slot=x12
//   t0=x13 (lhs value), t1=x14 (rhs value), t2=x15 (result)
//   Internal scratch: x16/x17 (used by macros)

op_mul_dsl:
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
    b.ne    Lop_mul_dsl__slow

    // load_reg!(c => t1)
    ldr     x14, [x20, x11, lsl #3]      ; t1 = REGS[rhs]

    // check_smi!(t1, .slow)
    mov     x16, #281470681743360
    movk    x16, #32760, lsl #48
    and     x16, x14, x16
    mov     x17, #17179869184
    movk    x17, #32760, lsl #48
    cmp     x16, x17
    b.ne    Lop_mul_dsl__slow

    // untag_smi!(t0); untag_smi!(t1)
    sxtw    x13, w13
    sxtw    x14, w14

    // mul_smi_overflow!(t0, t1 => t2, .slow)
    smull   x15, w13, w14                ; x_dst = (i64) lhs * (i64) rhs
    sxtw    x16, w15                     ; x16 = sxtw(low 32 of product)
    cmp     x15, x16                     ; overflow if sxtw != full product
    b.ne    Lop_mul_dsl__slow
    cbnz    w15, Ltmp14                  ; if result != 0, skip neg-zero check
    orr     w16, w13, w14                ; combine sign bits of lhs/rhs
    tbnz    w16, #31, Lop_mul_dsl__slow  ; either negative ⇒ ECMAScript -0
Ltmp14:                                  ; neg-zero check passed

    // tag_smi!(t2)
    mov     x16, #17179869184
    movk    x16, #32760, lsl #48
    ubfx    x15, x15, #0, #32
    orr     x15, x16, x15

    // store_reg!(a, t2)
    str     x15, [x20, x9, lsl #3]       ; REGS[dst] = t2

    // call_slow!(op_mul_record_smi_rs, args = [slot])
    ldr     x16, [x24, #8]               ; pb_base
    sub     x17, x19, x16                ; pc_offset = PC - pb_base
    str     w17, [x24]                   ; state.frame_pc_offset = pc_offset
    mov     x0, x24                      ; a0 = STATE
    mov     w1, w12                      ; a1 = slot
    bl      _op_mul_record_smi_rs

    // dispatch_after_slow!()
    cbnz    x0, Ltmp15
    ldr     x16, [x24, #8]
    add     x19, x16, x1
    ldr     x20, [x24, #16]
    ldr     x21, [x24, #24]
    ldrb    w8, [x19]
    ldr     x17, [x23, x8, lsl #3]
    br      x17
Ltmp15:
    cmp     x0, #2
    b.eq    Ltmp16
    ldr     w16, [x24]
    ldr     x17, [x24, #8]
    add     x19, x17, x16
    ldr     x20, [x24, #16]
    ldr     x21, [x24, #24]
    ldrb    w8, [x19]
    ldr     x17, [x23, x8, lsl #3]
    br      x17
Ltmp16:
    b       __interpreter_exit

Lop_mul_dsl__slow:
    // call_slow!(op_mul_slow_rs, args = [a, b, c, slot])
    ldr     x16, [x24, #8]               ; pb_base
    sub     x17, x19, x16                ; pc_offset
    str     w17, [x24]
    mov     x0, x24                      ; a0 = STATE
    mov     w1, w9                       ; a1 = dst
    mov     w2, w10                      ; a2 = lhs
    mov     w3, w11                      ; a3 = rhs
    mov     w4, w12                      ; a4 = slot
    bl      _op_mul_slow_rs

    // dispatch_after_slow!()
    cbnz    x0, Ltmp17
    ldr     x16, [x24, #8]
    add     x19, x16, x1
    ldr     x20, [x24, #16]
    ldr     x21, [x24, #24]
    ldrb    w8, [x19]
    ldr     x17, [x23, x8, lsl #3]
    br      x17
Ltmp17:
    cmp     x0, #2
    b.eq    Ltmp18
    ldr     w16, [x24]
    ldr     x17, [x24, #8]
    add     x19, x17, x16
    ldr     x20, [x24, #16]
    ldr     x21, [x24, #24]
    ldrb    w8, [x19]
    ldr     x17, [x23, x8, lsl #3]
    br      x17
Ltmp18:
    b       __interpreter_exit

// Fast-path instruction count (from `op_mul_dsl:` through
// `bl _op_mul_record_smi_rs` inclusive): 40 instructions.
//
// Compared to op_sub (36 insns): +4 for mul-specific work —
//   • mul_smi_overflow! is 7 insns (smull + sxtw + cmp + b.ne +
//     cbnz + orr + tbnz) vs sub_smi_overflow! 3 insns
//     (subs + b.vs + sxtw)                                    → +4
//
// The +4 delta breaks down as +1 for the multiply-overflow shape
// (smull+sxtw+cmp+b.ne vs adds+b.vs+sxtw) and +3 for the
// negative-zero deferral (cbnz+orr+tbnz) so the SMI fast path
// bails to slow on (-1)*0 / 0*(-1) / etc. — matching
// `smi_mul_result` in vm/dispatch/arithmetic.rs which returns
// `None` for the same cases.
//
// The decode / check_smi / untag / tag / store / record_smi /
// dispatch fragments are byte-for-byte identical to op_sub and
// op_add. Only the arithmetic-with-overflow-and-neg-zero block
// differs.
