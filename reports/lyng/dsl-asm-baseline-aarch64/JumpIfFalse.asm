__ZN10lyng_vm2vm17dispatch_handlers12control_flow16op_jump_if_false17hba91908de013b17cE:
L0:
	.loc	185 169 0
	sub	sp, sp, #80
	stp	x20, x19, [sp, #48]
	stp	x29, x30, [sp, #64]
	add	x29, sp, #64
L1:
	.loc	19 420 9 prologue_end
	ldr	w5, [x0, #4]
L2:
	.loc	19 425 9
	ldr	w6, [x0, #56]
L3:
	.loc	54 893 22
	ldrb	w3, [x0, #148]
	mov	w9, #152
L4:
	.loc	54 894 9
	strb	w9, [x0, #148]
L5:
	.loc	9 445 20
	ldr	x9, [x0, #128]
L6:
	.loc	51 1599 55
	ldr	x1, [x9, #56]
L7:
	.loc	52 568 12
	subs	x2, x1, x6
	b.lo	L8
L9:
	.loc	52 0 12 is_stmt 0
	mov	x19, x0
L10:
	.loc	53 504 9 is_stmt 1
	ldr	x9, [x9, #48]
L11:
	.loc	52 89 24
	add	x1, x9, x6
L12:
	.loc	88 295 12
	cmp	w3, #152
	b.ne	L13
L14:
	.loc	88 298 9
	cmp	x2, #3
	b.ls	L15
L16:
	.loc	88 307 19
	ldrb	w2, [x1, #1]
L17:
	.loc	49 3923 22
	ldrh	w3, [x1, #2]
L18:
	.loc	49 0 22 is_stmt 0
	mov	w4, #4
L19:
L20:
L21:
	.loc	185 181 5 is_stmt 1
	mov	x0, x8
	mov	x1, x19
	mov	w5, #0
	bl	__ZN10lyng_vm2vm17dispatch_handlers12control_flow15op_jump_if_impl17hd46dfef41a6e59a2E
L22:
L23:
	.loc	185 0 5 is_stmt 0
	b	L24
L25:
L15:
	mov	x9, #21
	movk	x9, #32768, lsl #48
L26:
	.loc	88 299 16 is_stmt 1
	str	x9, [sp]
	stp	w5, w6, [sp, #8]
L27:
L28:
	.loc	1 305 17
	ldp	q0, q1, [sp]
	stp	q0, q1, [x8]
	ldr	q0, [sp, #32]
	str	q0, [x8, #32]
L29:
L24:
	.loc	185 182 2 epilogue_begin
	ldp	x29, x30, [sp, #64]
	ldp	x20, x19, [sp, #48]
L30:
	add	sp, sp, #80
	ret
L31:
L8:
L32:
	.loc	52 569 13
L33:
	adrp	x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
L34:
	add	x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov	x0, x6
L35:
	mov	x2, x1
	bl	__ZN4core5slice5index16slice_index_fail17h88b945c1204cb3a0E
L36:
L37:
	.loc	52 0 13 is_stmt 0
	brk	#0x1
L38:
L13:
	.loc	88 296 16 is_stmt 1
	mov	x0, sp
	mov	w4, #0
	mov	x20, x8
	bl	__ZN10lyng_vm2vm8dispatch24decode_abx_operands_wide17hac6b4f3cbbb3a75fE
L39:
	.loc	88 0 16 is_stmt 0
	mov	x8, x20
L40:
	.loc	185 173 62 is_stmt 1
	ldr	x9, [sp]
	mov	x10, #21
	movk	x10, #32768, lsl #48
	add	x10, x10, #12
	.loc	1 303 9
	cmp	x9, x10
	b.ne	L28
L41:
	.loc	1 304 16
	ldrh	w2, [sp, #16]
	ldr	w3, [sp, #8]
	ldr	w4, [sp, #20]
	.loc	1 303 9
	b	L20
L42:
L43:
L44:
	.loc	185 169 1
	bl	__ZN4core9panicking19panic_cannot_unwind17hf1ae4d338f0b538fE
L45:
	.loh AdrpAdd	L33, L34
L46:
