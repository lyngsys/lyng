// DSL-1 Phase 1.C.3 op_decrement_dsl handler asm (AArch64), captured from
//   cargo rustc --release -p lyng-vm --lib -- --emit=asm
//
// Symbol: lyng_vm::dsl::handlers::cold::op_decrement_dsl
// Source: crates/vm/src/dsl/handlers/cold.rs
// Layout: AbcSlot (a, b, c byte operands + 16-bit feedback slot),
// length = 6 bytes. The `c` operand is decoded by the proc-macro
// lowerer but unused by the handler body (op_decrement is logically
// unary; the binary AbcSlot layout is shared with op_increment and
// keeps the dispatch table uniform).
//
// Register allocation by the lowerer:
//   a=x9 (dst), b=x10 (src), c=x11 (unused), slot=x12
//   t0=x13 (src value), t1=x14 (result)
//   Internal scratch: x16/x17 (used by macros)

op_decrement_dsl:
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
    b.ne    Lop_decrement_dsl__slow

    // untag_smi!(t0)
    sxtw    x13, w13

    // dec_smi_overflow!(t0 => t1, .slow)
    subs    w14, w13, #1                 ; t1 = t0 - 1, set NZCV
    b.vs    Lop_decrement_dsl__slow      ; branch on signed overflow (only at i32::MIN)
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
    // still performs the writeback via op_decrement_semantic.

    // call_slow!(op_decrement_record_smi_rs, args = [slot])
    ldr     x16, [x24, #8]               ; pb_base
    sub     x17, x19, x16                ; pc_offset = PC - pb_base
    str     w17, [x24]                   ; state.frame_pc_offset = pc_offset
    mov     x0, x24                      ; a0 = STATE
    mov     w1, w12                      ; a1 = slot
    bl      _op_decrement_record_smi_rs

    // dispatch_after_slow!()
    cbnz    x0, Ltmp97
    ldr     x16, [x24, #8]
    add     x19, x16, x1
    ldr     x20, [x24, #16]
    ldr     x21, [x24, #24]
    ldrb    w8, [x19]
    ldr     x17, [x23, x8, lsl #3]
    br      x17
Ltmp97:
    cmp     x0, #2
    b.eq    Ltmp98
    ldr     w16, [x24]
    ldr     x17, [x24, #8]
    add     x19, x17, x16
    ldr     x20, [x24, #16]
    ldr     x21, [x24, #24]
    ldrb    w8, [x19]
    ldr     x17, [x23, x8, lsl #3]
    br      x17
Ltmp98:
    b       __interpreter_exit

Lop_decrement_dsl__slow:
    // call_slow!(op_decrement_slow_rs, args = [a, b, c, slot])
    ldr     x16, [x24, #8]               ; pb_base
    sub     x17, x19, x16                ; pc_offset
    str     w17, [x24]
    mov     x0, x24                      ; a0 = STATE
    mov     w1, w9                       ; a1 = dst
    mov     w2, w10                      ; a2 = src
    mov     w3, w11                      ; a3 = c (unused but threaded)
    mov     w4, w12                      ; a4 = slot
    bl      _op_decrement_slow_rs

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

// Fast-path instruction count (from `op_decrement_dsl:` through
// `bl _op_decrement_record_smi_rs` inclusive): 27 instructions.
//
// Identical to op_increment's 27-instruction count — the two opcodes
// share the unary AbcSlot layout, the same one-load + one-check_smi +
// one-untag + 3-instruction overflow core + tag_smi + store_reg +
// call_slow sequence. The only opcode-level difference is the arith
// mnemonic: `subs` vs `adds`. Both macros use the 12-bit unsigned
// immediate `#1` form so no scratch-register dance is needed.
//
// Overflow case: `subs wD, wS, #1` sets the V flag only at i32::MIN
// (i.e. when the source is exactly -2147483648 and decrementing would
// produce -2147483649, not representable as i32). The `b.vs` branches
// to the slow path; the semantic body then handles the BigInt/Number
// promotion. This is much narrower than op_sub's overflow window (any
// `i32::MIN - positive_rhs` or `i32::MAX - negative_rhs`), so the SMI
// fast path stays armed essentially indefinitely for any workload that
// doesn't reach i32::MIN.
