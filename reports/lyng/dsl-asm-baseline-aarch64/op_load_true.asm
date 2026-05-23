lyng_vm::dsl::handlers::cold::op_load_true_dsl:
	; len=4 pc=0 pb=8 regs=16 fv=24 prefix=48 poll=1680 fb_stride=6 fb_observed=0 exit=__interpreter_exit
	ldrb	w9, [x19, #1]              ; decode a (byte at PC+1)
	ldrh	w10, [x19, #2]             ; decode _bx (unused; LLVM kept the load)
	mov	x11, #1                        ; tag_bool_const!(t0, 1) — movz x11, #0x1 (payload)
	movk	x11, #3, lsl #32           ; tag_bool_const!(t0, 1) — movk x11, #0x3, lsl #32 (Bool kind)
	movk	x11, #32760, lsl #48       ; tag_bool_const!(t0, 1) — movk x11, #0x7ff8, lsl #48 (32760 == 0x7ff8)
	str	x11, [x20, x9, lsl #3]         ; store_reg!(a, t0) — REGS[a] := true
	add	x19, x19, #4                   ; dispatch!() — advance PC by length=4
	ldrb	w8, [x19]                    ; dispatch!() — load next opcode byte
	ldr	x16, [x23, x8, lsl #3]         ; dispatch!() — look up next handler
	br	x16                            ; dispatch!() — tail-jump
