// DSL-0b op_loop_header handler asm (AArch64).
// Symbol: lyng_vm::dsl::handlers::warm::op_loop_header
// Source: crates/lyng/vm/src/dsl/handlers/warm.rs (B43)
// Layout: Ax (single u32 operand at PC+1; unused for the loop header
// — it's a backward-jump target for tools, but the handler itself
// only polls the safepoint and advances), length = 4 bytes.
//
// poll_safepoint! reads a byte from vm.poll_pending (offset 0 for
// DSL-0b; real offset lands when Vm gains the field). If zero, falls
// through to a 4-instr dispatch. Otherwise calls op_loop_header_poll_rs
// to consume pending work.

op_loop_header:
    // decode_ax! prologue (unused operand still gets decoded for
    // uniformity; the load is dead, optimizer prunes it post-DSL-0b).
    ldr     w9, [x19, #1]

    // poll_safepoint!(.poll_pending)
    ldrb    w16, [x22]                 ; w16 = vm.poll_pending
    cbnz    w16, Lop_loop_header__poll_pending

    // dispatch!(advance = 4)
    add     x19, x19, #4
    ldrb    w8, [x19]
    ldr     x16, [x23, x8, lsl #3]
    br      x16

Lop_loop_header__poll_pending:
    // call_slow!(op_loop_header_poll_rs, args = [])
    ldr     x16, [x24, #8]
    sub     x17, x19, x16
    str     w17, [x24]
    mov     x0, x24
    bl      _op_loop_header_poll_rs

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
