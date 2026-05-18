lyng_js_vm::dsl::handlers::cold::op_load_smi8_dsl:
	; len=3 pc=0 pb=8 regs=16 fv=24 prefix=48 poll=1680 fb_stride=6 fb_observed=0 exit=__interpreter_exit
	ldrb	w9, [x19, #1]              ; decode_ab!(a, b) — byte at PC+1 → w9 (a = dest reg id)
	ldrb	w10, [x19, #2]             ; decode_ab!(a, b) — byte at PC+2 → w10 (b = i8 payload, zero-extended by ldrb)
	sxtb	w10, w10                   ; tag_smi_from_signed_byte!(b) — sign-extend low byte to i32 in w10
	ubfx	x10, x10, #0, #32          ; tag_smi_from_signed_byte!(b) — clear x10 bits 32-63 (LLVM rewrote `uxtw x10, w10` to the equivalent `ubfx x10, x10, #0, #32`)
	mov	x16, #17179869184              ; tag_smi_from_signed_byte!(b) — materialize SMI kind: movz x16, #0x4, lsl #32 (LLVM rewrote to `mov x16, #0x4_0000_0000` = 17179869184)
	movk	x16, #32760, lsl #48       ; tag_smi_from_signed_byte!(b) — OR in NaN-tag header: movk x16, #0x7ff8, lsl #48 (32760 == 0x7ff8)
	orr	x10, x16, x10                  ; tag_smi_from_signed_byte!(b) — combine tag pattern + sign-extended payload
	str	x10, [x20, x9, lsl #3]         ; store_reg!(a, b) — REGS[a] := tagged SMI Value
	add	x19, x19, #3                   ; dispatch!() — advance PC by length=3
	ldrb	w8, [x19]                    ; dispatch!() — load next opcode byte
	ldr	x16, [x23, x8, lsl #3]         ; dispatch!() — look up next handler
	br	x16                            ; dispatch!() — tail-jump to next handler
