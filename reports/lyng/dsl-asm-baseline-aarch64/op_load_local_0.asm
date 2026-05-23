lyng_vm::dsl::handlers::cold::op_load_local_0_dsl:
	; len=2 pc=0 pb=8 regs=16 fv=24 prefix=64 poll=1680 fb_stride=6 fb_observed=0 ctr=0 const_base=32 this_value=40 uninit_lex=9221120275695796226 exit=__interpreter_exit
	ldrb	w9, [x19, #1]              ; decode_a!(a) — byte at PC+1 → w9 (a = dest reg id)
	ldr	x10, [x20]                     ; load_acc!(10) — x10 = REGS[0] (slot 0 = accumulator)
	str	x10, [x20, x9, lsl #3]         ; store_reg!(a, 10) — REGS[a] := x10
	add	x19, x19, #2                   ; dispatch!() — advance PC by length=2
	ldrb	w8, [x19]                    ; dispatch!() — load next opcode byte
	ldr	x16, [x23, x8, lsl #3]         ; dispatch!() — look up next handler
	br	x16                            ; dispatch!() — tail-jump to next handler
