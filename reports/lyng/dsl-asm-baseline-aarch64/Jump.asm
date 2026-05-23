__ZN10lyng_js_vm2vm17dispatch_handlers12control_flow7op_jump17h74722142eaf9e80dE:
L0:
	.loc	185 39 0
	sub	sp, sp, #112
	stp	x26, x25, [sp, #32]
	stp	x24, x23, [sp, #48]
	stp	x22, x21, [sp, #64]
	stp	x20, x19, [sp, #80]
	stp	x29, x30, [sp, #96]
	add	x29, sp, #96
L1:
	.loc	19 425 9 prologue_end
	ldr	w19, [x0, #56]
L2:
	.loc	9 445 20
	ldr	x21, [x0, #128]
L3:
	.loc	51 1599 55
	ldr	x1, [x21, #56]
L4:
	.loc	52 568 12
	subs	x9, x1, x19
	b.lo	L5
L6:
	.loc	185 0 0 is_stmt 0
	ldr	w20, [x0, #4]
L7:
	.loc	88 373 9 is_stmt 1
	cmp	x9, #3
	b.ls	L8
L9:
	.loc	53 504 9
	ldr	x9, [x21, #48]
L10:
	.loc	52 89 24
	add	x9, x9, x19
L11:
	.loc	88 382 48
	ldrb	w23, [x9, #3]
L12:
	.loc	88 130 60
	ldurh	w22, [x9, #1]
L13:
	.loc	185 45 8
	tbz	w23, #7, L14
L15:
	.loc	185 46 9
	ldr	x10, [x0, #80]
	.loc	185 46 18 is_stmt 0
	ldr	x11, [x10, #128]
L16:
	.loc	31 499 18 is_stmt 1
	sub	w9, w20, #1
L17:
	.loc	52 229 12
	cmp	x11, x9
	b.ls	L18
L19:
	.loc	185 46 18
	ldr	x10, [x10, #120]
	mov	w11, #24
L20:
	.loc	52 231 27
	umaddl	x9, w9, w11, x10
L21:
	.loc	5 767 15
	ldrb	w10, [x9, #21]
L22:
	.loc	5 767 9 is_stmt 0
	tbz	w10, #0, L18
L23:
	.loc	187 145 32 is_stmt 1
	ldr	w10, [x9, #8]
L24:
	.loc	49 2224 13
	adds	w10, w10, #1
L25:
	csinv	w10, w10, wzr, lo
L26:
	.loc	187 145 9
	str	w10, [x9, #8]
L27:
	.loc	187 8 30
	ldrb	w10, [x9, #20]
L28:
	.loc	187 151 12
	cmp	w10, #4
	b.ne	L29
L30:
	.loc	187 0 12 is_stmt 0
	mov	w10, #1
	.loc	187 152 13 is_stmt 1
	strb	w10, [x9, #20]
L31:
L29:
	.loc	187 154 24
	ldr	w11, [x9]
L32:
	.loc	49 2224 13
	adds	w11, w11, #2
L33:
	csinv	w11, w11, wzr, lo
L34:
	.loc	187 154 9
	str	w11, [x9]
	.loc	187 155 12
	cmp	w10, #1
	ccmp	w11, #7, #0, eq
	b.ls	L18
L35:
	.loc	187 0 12 is_stmt 0
	mov	w10, #2
	.loc	187 158 13 is_stmt 1
	strb	w10, [x9, #20]
L36:
L18:
	.loc	187 0 13 is_stmt 0
	mov	x24, x8
	mov	x25, x0
	.loc	185 47 45 is_stmt 1
	ldr	x0, [x0, #88]
L37:
L38:
	.loc	66 567 34
	mov	x8, sp
	bl	__ZN10lyng_js_gc7rooting50_$LT$impl$u20$lyng_js_gc..arena..PrimitiveHeap$GT$26poll_incremental_mark_step17h6c3f408ffd773f7eE
L39:
L40:
	.loc	66 0 34 is_stmt 0
	mov	x8, x24
	mov	x0, x25
L41:
L14:
	sxtb	w9, w23
	and	w9, w9, #0xff000000
	lsl	w10, w23, #16
L42:
	.loc	88 130 60 is_stmt 1
	orr	w9, w10, w9
L43:
	.loc	88 158 16
	add	w9, w22, w9
	add	w9, w9, #4
	adds	x9, x19, w9, sxtw
L44:
	.loc	88 159 8
	b.mi	L45
L46:
	.loc	23 331 31
	lsr	x10, x9, #32
	cbz	x10, L47
L48:
	.loc	23 0 31 is_stmt 0
	mov	x10, #4294967296
	.loc	23 331 31
	b	L49
L50:
L8:
	.loc	23 0 31
	mov	x9, #21
	movk	x9, #32768, lsl #48
L51:
	.loc	1 305 34 is_stmt 1
	str	x9, [x8]
	stp	w20, w19, [x8, #8]
	mov	w9, #4
	str	w9, [x8, #16]
L52:
	.loc	22 0 0 is_stmt 0
	b	L53
L54:
L45:
	mov	x10, #-4294967296
L55:
L49:
	.loc	1 305 17 is_stmt 1
	mov	w9, w9
	orr	x9, x10, x9
L56:
	.loc	1 0 17 is_stmt 0
	mov	x10, #21
	movk	x10, #32768, lsl #48
L57:
	.loc	1 305 34
	orr	x10, x10, #0x2
	stp	x10, x9, [x8]
	stp	w19, w20, [x8, #16]
L58:
	.loc	22 0 0
	b	L53
L59:
L47:
	.loc	19 435 9 is_stmt 1
	str	w9, [x0, #56]
L60:
	.loc	53 504 9
	ldr	x10, [x21, #48]
L61:
	.loc	1 100 18
	ldrb	w9, [x10, x9]
L62:
	.loc	1 244 13
L63:
	adrp	x10, __ZN10lyng_js_vm2vm14dispatch_state14DISPATCH_TABL64@PAGE
L65:
L66:
	add	x10, x10, __ZN10lyng_js_vm2vm14dispatch_state14DISPATCH_TABL64@PAGEOFF
	ldr	x9, [x10, x9, lsl #3]
	mov	x10, #21
	movk	x10, #32768, lsl #48
	.loc	1 243 16
	add	x10, x10, #12
	stp	x10, x9, [x8]
L67:
L53:
	.loc	185 55 2 epilogue_begin
	ldp	x29, x30, [sp, #96]
	ldp	x20, x19, [sp, #80]
	ldp	x22, x21, [sp, #64]
	ldp	x24, x23, [sp, #48]
	ldp	x26, x25, [sp, #32]
	add	sp, sp, #112
	ret
L5:
L68:
L69:
	.loc	52 569 13
L70:
	adrp	x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
L71:
	add	x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov	x0, x19
L72:
	mov	x2, x1
	bl	__ZN4core5slice5index16slice_index_fail17h88b945c1204cb3a0E
L73:
L74:
	.loc	52 0 13 is_stmt 0
	brk	#0x1
L75:
L76:
L77:
	.loc	185 39 1 is_stmt 1
	bl	__ZN4core9panicking19panic_cannot_unwind17hf1ae4d338f0b538fE
L78:
	.loh AdrpAdd	L63, L66
	.loh AdrpAdd	L70, L71
L79:
