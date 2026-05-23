lyng_js_vm::dsl::handlers::cold::op_load_const8_dsl:
	; len=3 pc=0 pb=8 regs=16 fv=24 prefix=64 poll=1680 fb_stride=6 fb_observed=0 ctr=0 const_base=32 this_value=40 uninit_lex=9221120275695796226 exit=__interpreter_exit
	ldrb	w9, [x19, #1]              ; decode_ab!(a, b) — byte at PC+1 → w9 (a = dest reg id)
	ldrb	w10, [x19, #2]             ; decode_ab!(a, b) — byte at PC+2 → w10 (b = constant pool index, u8)
	ldr	x16, [x24, #32]                ; load_constant!(b => 10) — x16 = LlIntState.frame_const_base (*const Value)
	ldr	x10, [x16, x10, lsl #3]        ; load_constant!(b => 10) — x10 = frame_const_base[b] (Value is 8B → lsl #3)
	str	x10, [x20, x9, lsl #3]         ; store_reg!(a, 10) — REGS[a] := loaded Value
	add	x19, x19, #3                   ; dispatch!() — advance PC by length=3
	ldrb	w8, [x19]                    ; dispatch!() — load next opcode byte
	ldr	x16, [x23, x8, lsl #3]         ; dispatch!() — look up next handler
	br	x16                            ; dispatch!() — tail-jump to next handler
