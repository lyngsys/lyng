// DSL-1 Phase 1.C.3 op_increment_dsl handler asm (AArch64), captured from
//   cargo rustc --release -p lyng-vm --lib -- --emit=asm
//
// Symbol: lyng_vm::dsl::handlers::cold::op_increment_dsl
// Source: crates/vm/src/dsl/handlers/cold.rs
// Layout: AbcSlot (a, b, c byte operands + 16-bit feedback slot),
// length = 6 bytes. The `c` operand is decoded by the proc-macro
// lowerer but unused by the handler body (op_increment is logically
// unary; the binary AbcSlot layout is shared with op_decrement and
// keeps the dispatch table uniform).
//
// Register allocation by the lowerer:
//   a=x9 (dst), b=x10 (src), c=x11 (unused), slot=x12
//   t0=x13 (src value), t1=x14 (result)
//   Internal scratch: x16/x17 (used by macros)

op_increment_dsl:
    // decode_abc_slot! prologue
    ldrb    w9,  [x19, #1]               ; w9  = a (dst reg id)
    ldrb    w10, [x19, #2]               ; w10 = b (src reg id)
    ldrb    w11, [x19, #3]               ; w11 = c (unused operand)
    ldrh    w12, [x19, #4]               ; w12 = slot (feedback slot id)

    // load_reg!(b => t0)
    ldr     x13, [x20, x10, lsl #3]      ; t0 = REGS[src]

    // check_smi!(t0, .slow)
    mov     x16, #281470681743360
    movk    x16, #32760, lsl #48
    and     x16, x13, x16
    mov     x17, #17179869184
    movk    x17, #32760, lsl #48
    cmp     x16, x17
    b.ne    Lop_increment_dsl__slow

    // untag_smi!(t0)
    sxtw    x13, w13

    // inc_smi_overflow!(t0 => t1, .slow)
    adds    w14, w13, #1                 ; t1 = t0 + 1, set NZCV
    b.vs    Lop_increment_dsl__slow      ; branch on signed overflow
    sxtw    x14, w14                     ; sign-extend 32-bit result to i64

    // tag_smi!(t1)
    mov     x16, #17179869184
    movk    x16, #32760, lsl #48
    ubfx    x14, x14, #0, #32
    orr     x14, x16, x14

    // store_reg!(a, t1)
    str     x14, [x20, x9, lsl #3]       ; REGS[dst] = t1

    // SMI fast-path elision: the semantic body writes ToNumeric(src)
    // back to the src register (vm/semantics/arithmetic.rs:825) before
    // writing the post-update value to dst. For SMI src, ToNumeric is
    // identity (Value::from_smi round-trips) so the writeback is a
    // no-op. The fast path skips it. Non-SMI src bails to .slow which
    // still performs the writeback via op_increment_semantic.

    // call_slow!(op_increment_record_smi_rs, args = [slot])
    ldr     x16, [x24, #8]               ; pb_base
    sub     x17, x19, x16                ; pc_offset = PC - pb_base
    str     w17, [x24]                   ; state.frame_pc_offset = pc_offset
    mov     x0, x24                      ; a0 = STATE
    mov     w1, w12                      ; a1 = slot
    bl      _op_increment_record_smi_rs

    // dispatch_after_slow!()
    cbnz    x0, Ltmp99
    ldr     x16, [x24, #8]
    add     x19, x16, x1
    ldr     x20, [x24, #16]
    ldr     x21, [x24, #24]
    ldrb    w8, [x19]
    ldr     x17, [x23, x8, lsl #3]
    br      x17
Ltmp99:
    cmp     x0, #2
    b.eq    Ltmp100
    ldr     w16, [x24]
    ldr     x17, [x24, #8]
    add     x19, x17, x16
    ldr     x20, [x24, #16]
    ldr     x21, [x24, #24]
    ldrb    w8, [x19]
    ldr     x17, [x23, x8, lsl #3]
    br      x17
Ltmp100:
    b       __interpreter_exit

Lop_increment_dsl__slow:
    // call_slow!(op_increment_slow_rs, args = [a, b, c, slot])
    ldr     x16, [x24, #8]               ; pb_base
    sub     x17, x19, x16                ; pc_offset
    str     w17, [x24]
    mov     x0, x24                      ; a0 = STATE
    mov     w1, w9                       ; a1 = dst
    mov     w2, w10                      ; a2 = src
    mov     w3, w11                      ; a3 = c (unused but threaded)
    mov     w4, w12                      ; a4 = slot
    bl      _op_increment_slow_rs

    // dispatch_after_slow!()
    cbnz    x0, Ltmp101
    ldr     x16, [x24, #8]
    add     x19, x16, x1
    ldr     x20, [x24, #16]
    ldr     x21, [x24, #24]
    ldrb    w8, [x19]
    ldr     x17, [x23, x8, lsl #3]
    br      x17
Ltmp101:
    cmp     x0, #2
    b.eq    Ltmp102
    ldr     w16, [x24]
    ldr     x17, [x24, #8]
    add     x19, x17, x16
    ldr     x20, [x24, #16]
    ldr     x21, [x24, #24]
    ldrb    w8, [x19]
    ldr     x17, [x23, x8, lsl #3]
    br      x17
Ltmp102:
    b       __interpreter_exit

// Fast-path instruction count (from `op_increment_dsl:` through
// `bl _op_increment_record_smi_rs` inclusive): 27 instructions.
//
// Substantially shorter than the binary inline ports (op_sub: 36,
// op_mul: 40, op_bit_and: 34, op_shift_left/right: 36) because
// op_increment is unary: a single source register means only one
// `load_reg!` (1 ldr) + one `check_smi!` (7 instructions) + one
// `untag_smi!` (1 sxtw) — saving 9 instructions vs. the binary
// shapes. The `adds wD, wS, #1` form (12-bit unsigned immediate)
// avoids the scratch-register dance used by `sub_smi_overflow!`
// (which threads its rhs as a register operand), keeping the
// increment core at 3 instructions (adds + b.vs + sxtw).
