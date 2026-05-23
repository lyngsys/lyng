// DSL-0b op_jump handler asm (AArch64).
// Symbol: lyng_vm::dsl::handlers::hot::op_jump
// Source: crates/lyng/vm/src/dsl/handlers/hot.rs (B41)
// Layout: Ax (single u32 operand at PC+1), length = 5 bytes.
//
// For DSL-0b op_jump is a thin call_slow + dispatch_after_slow. The
// real PC arithmetic + backward-edge poll happens inside
// op_jump_semantic (already a clean function pointer in Rust). A
// future iteration can inline the forward-jump fast path; see
// op_jump_shared_semantic in vm/semantics/control_flow.rs.

op_jump:
    // decode_ax! prologue: load 4-byte offset at [PC+1] into w9.
    ldr     w9, [x19, #1]

    // call_slow!(op_jump_slow_rs, args = [offset])
    ldr     x16, [x24, #8]        ; pb_base
    sub     x17, x19, x16          ; pc_offset
    str     w17, [x24]             ; state.frame_pc_offset
    mov     x0, x24                ; a0 = STATE
    mov     w1, w9                 ; a1 = offset
    bl      _op_jump_slow_rs

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
