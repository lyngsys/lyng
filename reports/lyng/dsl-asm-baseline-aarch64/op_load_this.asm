lyng_vm::dsl::handlers::cold::op_load_this_dsl:
	; len=4 pc=0 pb=8 regs=16 fv=24 prefix=64 poll=1680 fb_stride=6 fb_observed=0 ctr=0 const_base=32 this_value=40 uninit_lex=9221120275695796226 exit=__interpreter_exit
	ldrb	w9, [x19, #1]                ; decode_abx!(a, bx) — byte at PC+1 → w9 (a = dest reg id)
	ldrh	w10, [x19, #2]               ; decode_abx!(a, bx) — bytes PC+2..3 → w10 (bx, unused at runtime)
	ldr	x10, [x24, #40]                  ; load_state_value!(10, vm_state_offset = state_this_value) — x10 = LlIntState.frame_this_value
	mov	x11, #2                          ; load_uninit_lex_sentinel!(11) — movz x11, #2 (low 16 bits of VALUE_UNINIT_LEX_BITS = 0x7ff8_0009_0000_0002)
	movk	x11, #0, lsl #16             ; load_uninit_lex_sentinel!(11) — movk x11, #0, lsl #16 (bits 16-31; LLVM kept this even though it OR's in nothing)
	movk	x11, #9, lsl #32             ; load_uninit_lex_sentinel!(11) — movk x11, #9, lsl #32 (TagKind::Sentinel discriminator = 9)
	movk	x11, #32760, lsl #48         ; load_uninit_lex_sentinel!(11) — movk x11, #0x7ff8, lsl #48 (NaN-tag header; 32760 = 0x7ff8)
	cmp	x10, x11                         ; cmp_branch_eq!(10, 11, .slow) — compare mirror vs sentinel
	b.eq	Lop_load_this_dsl__slow      ; cmp_branch_eq!(10, 11, .slow) — bail to slow on sentinel match
	str	x10, [x20, x9, lsl #3]           ; store_reg!(a, 10) — REGS[a] := loaded `this` Value
	add	x19, x19, #4                     ; dispatch!() — advance PC by length=4
	ldrb	w8, [x19]                      ; dispatch!() — load next opcode byte
	ldr	x16, [x23, x8, lsl #3]           ; dispatch!() — look up next handler
	br	x16                              ; dispatch!() — tail-jump to next handler
Lop_load_this_dsl__slow:
	; call_slow!(op_load_this_slow_rs, args = [a, bx])
	ldr	x16, [x24, #8]                   ; load pb_base
	sub	x17, x19, x16                    ; compute pc_offset = PC - pb_base
	str	w17, [x24]                       ; store pc_offset in state.frame_pc_offset
	mov	x0, x24                          ; arg0 = STATE
	mov	w1, w9                           ; arg1 = a
	mov	w2, w10                          ; arg2 = bx
	bl	_op_load_this_slow_rs            ; call slow path
	; dispatch_after_slow!() tail
	cbnz	x0, <unusual>                ; tag != Continue → unusual handling
	ldr	x16, [x24, #8]                   ; reload pb_base
	add	x19, x16, x1                     ; reconstruct PC from new pc_offset
	ldr	x20, [x24, #16]                  ; reload REGS
	ldr	x21, [x24, #24]                  ; reload FV
	ldrb	w8, [x19]                      ; load next opcode byte
	ldr	x17, [x23, x8, lsl #3]           ; look up next handler
	br	x17                              ; tail-jump
	; (Refresh + Exit arms elided — handled by dispatch_after_slow! tail)
