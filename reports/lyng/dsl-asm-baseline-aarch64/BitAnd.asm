__ZN10lyng_js_vm2vm17dispatch_handlers10arithmetic10op_bit_and17h5b310fbcff76e749E:
L0:
	.loc	178 673 0
	sub	sp, sp, #160
	stp	x28, x27, [sp, #64]
	stp	x26, x25, [sp, #80]
	stp	x24, x23, [sp, #96]
	stp	x22, x21, [sp, #112]
	stp	x20, x19, [sp, #128]
	stp	x29, x30, [sp, #144]
	add	x29, sp, #144
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
	ldrb	w9, [x8, #1]
L20:
	.loc	88 0 19 is_stmt 0
	str	w9, [sp, #12]
	.loc	88 249 19 is_stmt 1
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
	.loc	178 685 16
	ldr	x24, [x21, #80]
L26:
	.loc	91 160 20
	add	w8, w26, w3
L27:
	.loc	53 504 9
	ldr	x9, [x24, #24]
L28:
	.loc	91 61 18
	ldr	x27, [x9, w8, uxtw #3]
L29:
	.loc	91 160 20
	add	w8, w26, w4
L30:
	.loc	91 61 18
	ldr	x22, [x9, w8, uxtw #3]
L31:
	.loc	91 0 18 is_stmt 0
	mov	x28, #17179869184
	movk	x28, #32760, lsl #48
L32:
	mov	x8, #281470681743360
L33:
	movk	x8, #32760, lsl #48
L34:
	.loc	16 95 12 is_stmt 1
	and	x9, x27, x8
L35:
	.loc	16 95 12 is_stmt 0
	and	x8, x22, x8
	cmp	x8, x28
	ccmp	x9, x28, #0, eq
	b.eq	L36
L37:
L38:
	.loc	178 695 5 is_stmt 1
	mov	x0, x19
	mov	x1, x21
	ldr	w2, [sp, #12]
	mov	x6, x23
	bl	__ZN10lyng_js_vm2vm17dispatch_handlers10arithmetic15op_bit_and_slow17hf68b0ea063983066E
L39:
L40:
	b	L41
L42:
L16:
	.loc	178 0 5 is_stmt 0
	mov	x8, #33
	movk	x8, #32768, lsl #48
L43:
	sub	x8, x8, #12
	str	x8, [sp, #16]
	stp	w1, w20, [sp, #24]
L44:
L45:
	.loc	1 305 17 is_stmt 1
	ldp	q0, q1, [sp, #16]
	stp	q0, q1, [x19]
	ldr	q0, [sp, #48]
	str	q0, [x19, #32]
L46:
L41:
	.loc	178 696 2 epilogue_begin
	ldp	x29, x30, [sp, #144]
	ldp	x20, x19, [sp, #128]
	ldp	x22, x21, [sp, #112]
L47:
	ldp	x24, x23, [sp, #96]
	ldp	x26, x25, [sp, #80]
	ldp	x28, x27, [sp, #64]
	add	sp, sp, #160
	ret
L48:
L36:
L49:
	.loc	178 688 18
	mov	x0, x24
	mov	x2, x5
	bl	__ZN10lyng_js_vm2vm8feedback36_$LT$impl$u20$lyng_js_vm..vm..Vm$GT$20record_feedback_slot17h9062948f4dbd3d29E
L50:
L51:
	.loc	178 691 69
	and	w8, w27, w22
L52:
	.loc	16 80 14
	orr	x8, x8, x28
	ldr	w9, [sp, #12]
L53:
	.loc	91 160 20
	add	w9, w26, w9
L54:
	.loc	53 504 9
	ldr	x10, [x24, #24]
L55:
	.loc	91 84 13
	str	x8, [x10, w9, uxtw #3]
L56:
	.loc	49 2380 13
	add	w8, w20, w23
L57:
	.loc	19 435 9
	str	w8, [x21, #56]
L58:
	.loc	53 504 9
	ldr	x9, [x25, #48]
L59:
	.loc	1 100 18
	ldrb	w8, [x9, w8, uxtw]
L60:
	.loc	1 244 13
L61:
	adrp	x9, __ZN10lyng_js_vm2vm14dispatch_state14DISPATCH_TABL62@PAGE
L63:
L64:
	add	x9, x9, __ZN10lyng_js_vm2vm14dispatch_state14DISPATCH_TABL62@PAGEOFF
	ldr	x8, [x9, x8, lsl #3]
	mov	x9, #33
	movk	x9, #32768, lsl #48
	.loc	1 243 16
	stp	x9, x8, [x19]
	b	L41
L65:
L8:
L66:
	.loc	52 569 13
L67:
	adrp	x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
L68:
	add	x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov	x0, x20
L69:
	mov	x1, x8
L70:
	mov	x2, x8
	bl	__ZN4core5slice5index16slice_index_fail17h88b945c1204cb3a0E
L71:
L72:
	.loc	52 0 13 is_stmt 0
	brk	#0x1
L73:
L13:
	.loc	88 237 16 is_stmt 1
	add	x0, sp, #16
	mov	x4, x1
	mov	x1, x8
L74:
	mov	w3, #1
	mov	x22, x4
	mov	x5, x20
	bl	__ZN10lyng_js_vm2vm8dispatch24decode_abc_operands_wide17he457cb4163a94953E
L75:
	.loc	178 677 63
	ldr	x8, [sp, #16]
	mov	x9, #33
	movk	x9, #32768, lsl #48
	.loc	1 303 9
	cmp	x8, x9
	b.ne	L45
L76:
	.loc	1 304 16
	ldrh	w8, [sp, #28]
	str	w8, [sp, #12]
	ldrh	w3, [sp, #30]
	ldrh	w4, [sp, #32]
	ldr	w5, [sp, #24]
	ldr	w23, [sp, #36]
	mov	x1, x22
	b	L24
L77:
L78:
L79:
	.loc	178 673 1
	bl	__ZN4core9panicking19panic_cannot_unwind17hf1ae4d338f0b538fE
L80:
	.loh AdrpAdd	L61, L64
	.loh AdrpAdd	L67, L68
L81:
