// DSL-0b op_move handler asm (AArch64), captured via:
//   cargo rustc --release -p lyng-js-vm --lib -- --emit=asm
// then extracted from target/release/deps/lyng_js_vm-<hash>.s.
//
// Symbol: lyng_js_vm::dsl::handlers::hot::op_move
// Source: crates/lyng-js/vm/src/dsl/handlers/hot.rs (B39)
// Layout: Ab (two register-index byte operands), length = 3 bytes.

op_move:
    // decode_ab! (operand-decode prologue emitted by the lowerer)
    ldrb    w9, [x19, #1]            ; w9 = dst register index
    ldrb    w10, [x19, #2]           ; w10 = src register index
    // load_reg!(src => t0)
    ldr     x11, [x20, x10, lsl #3]  ; x11 = REGS[src]
    // store_reg!(dst, t0)
    str     x11, [x20, x9, lsl #3]   ; REGS[dst] = x11
    // dispatch!()
    add     x19, x19, #3             ; advance PC by handler length
    ldrb    w8, [x19]                ; w8 = next opcode byte
    ldr     x9, [x23, x8, lsl #3]    ; x9 = handler addr from DSL_DISPATCH_TABLE
    br      x9                       ; tail-jump
