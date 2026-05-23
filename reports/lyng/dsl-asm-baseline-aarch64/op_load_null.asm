lyng_vm::dsl::handlers::cold::op_load_null_dsl:
	; len=4 pc=0 pb=8 regs=16 fv=24 prefix=48 poll=1680 fb_stride=6 fb_observed=0 exit=__interpreter_exit
	ldrb	w9, [x19, #1]              ; decode a (byte at PC+1)
	ldrh	w10, [x19, #2]             ; decode _bx (unused; LLVM kept the load)
	mov	x11, #8589934592               ; tag_null!(t0) — movz x11, #0x2, lsl #32
	movk	x11, #32760, lsl #48       ; tag_null!(t0) — movk x11, #0x7ff8, lsl #48 (32760 == 0x7ff8)
	str	x11, [x20, x9, lsl #3]         ; store_reg!(a, t0) — REGS[a] := null
	add	x19, x19, #4                   ; dispatch!() — advance PC by length=4
	ldrb	w8, [x19]                    ; dispatch!() — load next opcode byte
	ldr	x16, [x23, x8, lsl #3]         ; dispatch!() — look up next handler
	br	x16                            ; dispatch!() — tail-jump
