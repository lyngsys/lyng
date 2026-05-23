lyng_vm::dsl::handlers::cold::op_load_zero_dsl:
	; len=4 pc=0 pb=8 regs=16 fv=24 prefix=48 poll=1680 fb_stride=6 fb_observed=0 exit=__interpreter_exit
	ldrb	w9, [x19, #1]              ; decode a (byte at PC+1)
	ldrh	w10, [x19, #2]             ; decode _bx (unused; LLVM kept the load)
	mov	x11, #0                        ; tag_smi_const!(t0, 0) — movz x11, #0x0 (payload)
	movk	x11, #4, lsl #32           ; tag_smi_const!(t0, 0) — movk x11, #0x4, lsl #32 (SMI kind)
	movk	x11, #32760, lsl #48       ; tag_smi_const!(t0, 0) — movk x11, #0x7ff8, lsl #48 (32760 == 0x7ff8)
	str	x11, [x20, x9, lsl #3]         ; store_reg!(a, t0) — REGS[a] := SMI(0)
	add	x19, x19, #4                   ; dispatch!() — advance PC by length=4
	ldrb	w8, [x19]                    ; dispatch!() — load next opcode byte
	ldr	x16, [x23, x8, lsl #3]         ; dispatch!() — look up next handler
	br	x16                            ; dispatch!() — tail-jump
