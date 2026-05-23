__ZN10lyng_vm2vm17dispatch_handlers10arithmetic6op_sub17h3141c6fabb0b3e73E:
L0:
	.loc	178 110 0
	sub	sp, sp, #144
	stp	x28, x27, [sp, #48]
	stp	x26, x25, [sp, #64]
	stp	x24, x23, [sp, #80]
	stp	x22, x21, [sp, #96]
	stp	x20, x19, [sp, #112]
	stp	x29, x30, [sp, #128]
	add	x29, sp, #128
	mov	x19, x8
L1:
	.loc	19 420 9 prologue_end
	ldr	w1, [x0, #4]
L2:
	.loc	19 425 9
	ldr	w20, [x0, #56]
L3:
	.loc	54 893 22
	ldrb	w9, [x0, #148]
	mov	w8, #152
L4:
	.loc	54 894 9
	strb	w8, [x0, #148]
L5:
	.loc	9 445 20
	ldr	x25, [x0, #128]
L6:
	.loc	51 1599 55
	ldr	x8, [x25, #56]
L7:
	.loc	52 568 12
	subs	x2, x8, x20
	b.lo	L8
L9:
	.loc	52 0 12 is_stmt 0
	mov	x21, x0
L10:
	.loc	53 504 9 is_stmt 1
	ldr	x8, [x25, #48]
L11:
	.loc	52 89 24
	add	x8, x8, x20
L12:
	.loc	88 236 8
	cmp	w9, #152
	b.ne	L13
L14:
	.loc	88 0 8 is_stmt 0
	and	x9, x2, #0x7ffffffffffffffe
L15:
	.loc	88 239 9 is_stmt 1
	cmp	x2, #4
	ccmp	x9, #4, #4, hs
	b.eq	L16
L17:
	.loc	49 3923 22
	ldrh	w5, [x8, #4]
L18:
	.loc	5 1338 9
	cbz	w5, L16
L19:
	.loc	88 248 19
	ldrb	w22, [x8, #1]
L20:
	.loc	88 249 19
	ldrb	w3, [x8, #2]
L21:
	.loc	88 0 19 is_stmt 0
	mov	w23, #6
L22:
	.loc	88 250 19 is_stmt 1
	ldrb	w4, [x8, #3]
L23:
L24:
	.loc	19 440 9
	ldr	w26, [x21, #20]
L25:
	.loc	178 123 16
	ldr	x24, [x21, #80]
L26:
	.loc	91 160 20
	add	w8, w26, w3
L27:
	.loc	53 504 9
	ldr	x9, [x24, #24]
L28:
	.loc	91 61 18
	ldr	x8, [x9, w8, uxtw #3]
L29:
	.loc	91 160 20
	add	w10, w26, w4
L30:
	.loc	91 61 18
	ldr	x9, [x9, w10, uxtw #3]
L31:
	.loc	91 0 18 is_stmt 0
	mov	x27, #17179869184
	movk	x27, #32760, lsl #48
L32:
	mov	x10, #281470681743360
L33:
	movk	x10, #32760, lsl #48
L34:
	.loc	16 95 12 is_stmt 1
	and	x11, x8, x10
L35:
	.loc	16 95 12 is_stmt 0
	and	x10, x9, x10
	cmp	x10, x27
	ccmp	x11, x27, #0, eq
	b.eq	L36
L37:
L38:
L39:
	.loc	178 135 5 is_stmt 1
	mov	x0, x19
	mov	x1, x21
	mov	x2, x22
	mov	x6, x23
	bl	__ZN10lyng_vm2vm17dispatch_handlers10arithmetic11op_sub_slow17h409cc67e5d0a258fE
L40:
L41:
	b	L42
L43:
L16:
	.loc	178 0 5 is_stmt 0
	mov	x8, #33
	movk	x8, #32768, lsl #48
L44:
	sub	x8, x8, #12
	str	x8, [sp]
	stp	w1, w20, [sp, #8]
L45:
L46:
	.loc	1 305 17 is_stmt 1
	ldp	q0, q1, [sp]
	stp	q0, q1, [x19]
	ldr	q0, [sp, #32]
	str	q0, [x19, #32]
L47:
L42:
	.loc	178 136 2 epilogue_begin
	ldp	x29, x30, [sp, #128]
	ldp	x20, x19, [sp, #112]
	ldp	x22, x21, [sp, #96]
L48:
	ldp	x24, x23, [sp, #80]
	ldp	x26, x25, [sp, #64]
	ldp	x28, x27, [sp, #48]
	add	sp, sp, #144
	ret
L49:
L36:
	.loc	179 2566 26
	subs	w28, w8, w9
L50:
	.loc	178 126 16
	b.vs	L38
L51:
L52:
	.loc	178 128 18
	mov	x0, x24
	mov	x2, x5
	bl	__ZN10lyng_vm2vm8feedback36_$LT$impl$u20$lyng_vm..vm..Vm$GT$20record_feedback_slot17h9062948f4dbd3d29E
L53:
L54:
	.loc	178 0 0 is_stmt 0
	orr	x8, x28, x27
L55:
	.loc	91 160 20 is_stmt 1
	add	w9, w26, w22
L56:
	.loc	53 504 9
	ldr	x10, [x24, #24]
L57:
	.loc	91 84 13
	str	x8, [x10, w9, uxtw #3]
L58:
	.loc	49 2380 13
	add	w8, w20, w23
L59:
	.loc	19 435 9
	str	w8, [x21, #56]
L60:
	.loc	53 504 9
	ldr	x9, [x25, #48]
L61:
	.loc	1 100 18
	ldrb	w8, [x9, w8, uxtw]
L62:
	.loc	1 244 13
L63:
	adrp	x9, __ZN10lyng_vm2vm14dispatch_state14DISPATCH_TABL64@PAGE
L65:
L66:
	add	x9, x9, __ZN10lyng_vm2vm14dispatch_state14DISPATCH_TABL64@PAGEOFF
	ldr	x8, [x9, x8, lsl #3]
	mov	x9, #33
	movk	x9, #32768, lsl #48
	.loc	1 243 16
	stp	x9, x8, [x19]
	b	L42
L67:
L8:
L68:
	.loc	52 569 13
L69:
	adrp	x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
L70:
	add	x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov	x0, x20
L71:
	mov	x1, x8
L72:
	mov	x2, x8
	bl	__ZN4core5slice5index16slice_index_fail17h88b945c1204cb3a0E
L73:
L74:
	.loc	52 0 13 is_stmt 0
	brk	#0x1
L75:
L13:
	.loc	88 237 16 is_stmt 1
	mov	x0, sp
	mov	x4, x1
	mov	x1, x8
L76:
	mov	w3, #1
	mov	x24, x4
	mov	x5, x20
	bl	__ZN10lyng_vm2vm8dispatch24decode_abc_operands_wide17he457cb4163a94953E
L77:
	.loc	178 114 63
	ldr	x8, [sp]
	mov	x9, #33
	movk	x9, #32768, lsl #48
	.loc	1 303 9
	cmp	x8, x9
	b.ne	L46
L78:
	.loc	1 304 16
	ldrh	w22, [sp, #12]
	ldrh	w3, [sp, #14]
	ldrh	w4, [sp, #16]
	ldr	w5, [sp, #8]
	ldr	w23, [sp, #20]
	mov	x1, x24
	b	L24
L79:
L80:
L81:
	.loc	178 110 1
	bl	__ZN4core9panicking19panic_cannot_unwind17hf1ae4d338f0b538fE
L82:
	.loh AdrpAdd	L63, L66
	.loh AdrpAdd	L69, L70
L83:
