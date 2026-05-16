lyng_js_vm::vm::dispatch_handlers::loads::op_load_const8:
L0:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	mov x9, x0
	ldr w0, [x0, #56]
	ldr x13, [x9, #128]
	ldr x1, [x13, #56]
	subs x12, x1, x0
	b.lo L1
	ldr w11, [x9, #4]
	mov x10, #29
	movk x10, #32768, lsl #48
	cmp x12, #2
	b.ls L2
	ldr x12, [x13, #48]
	add x16, x12, x0
	ldrb w12, [x16, #2]
	ldr x14, [x9, #80]
	ldr x17, [x14, #80]
	sub w1, w11, #1
	sub x15, x10, #25
	cmp x17, x1
	b.ls L3
	ldr x17, [x14, #72]
	ldr x17, [x17, x1, lsl #3]
	cbz x17, L3
	ldr x15, [x17, #80]
	cmp x15, x12
	b.ls L4
	ldr x15, [x17, #72]
	add x17, x15, x12, lsl #4
	ldr w15, [x17]
	cmp w15, #4
	b.ne L5
L4:
	mov w15, #0
	mov x9, #0
	b L6
L2:
	sub x9, x10, #8
	str x9, [x8]
	stp w11, w0, [x8, #8]
	mov w9, #3
	str w9, [x8, #20]
	b L7
L3:
	mov x9, #0
	mov x10, x15
	mov x15, x11
L6:
	mov w13, w15
	orr x9, x9, x13
	stp x10, x9, [x8]
	str x16, [x8, #16]
	stp w11, w12, [x8, #24]
L7:
	ldp x29, x30, [sp], #16
	ret
L5:
	ldr x3, [x9, #88]
	ldr x2, [x3, #416]
	lsr x4, x1, #6
	cmp x4, x2
	b.hs L8
	mov x2, #0
	ldr x5, [x3, #408]
	and x1, x1, #0x3f
	ldr x4, [x5, x4, lsl #3]
	add x1, x4, x1, lsl #4
	ldr w4, [x1, #24]
	tbz w4, #0, L9
	ldr w4, [x1, #36]
	cbz w4, L10
	ldr x2, [x3, #784]
	sub w1, w4, #1
	cmp x2, x1
	b.ls L8
	ldr x2, [x3, #776]
	mov w3, #24
	umaddl x1, w1, w3, x2
	ldrb w2, [x1, #19]
	cmp w2, #1
	b.ne L8
	ldr x2, [x1, #8]
	cmp x2, x12
	b.ls L8
	ldr x1, [x1]
	ldr x1, [x1, x12, lsl #3]
	mov x2, #3
	movk x2, #9, lsl #32
	movk x2, #32760, lsl #48
	cmp x1, x2
	cset w2, ne
	ldrb w3, [x16, #1]
	tbnz w2, #0, L11
	b L12
L8:
	mov x2, #0
L10:
	ldrb w3, [x16, #1]
	tbz w2, #0, L12
L11:
	ldr w11, [x9, #20]
	add w11, w11, w3
	ldr x12, [x14, #24]
	str x1, [x12, w11, uxtw #3]
	add w11, w0, #3
	ldr x12, [x13, #48]
	ldrb w12, [x12, w11, uxtw]
L13:
	adrp x13, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
L14:
	add x13, x13, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	str w11, [x9, #56]
	ldr x9, [x13, x12, lsl #3]
	add x10, x10, #4
	stp x10, x9, [x8]
	b L7
L9:
	ldrb w3, [x16, #1]
	tbnz w2, #0, L11
L12:
	ldr w1, [x17, #4]
	ldr x16, [x17, #8]
	sub w17, w15, #2
	cmp w17, #2
	b.hs L15
	lsl x9, x1, #32
	b L6
L15:
	cbnz w15, L16
	mov x11, #17179869184
	movk x11, #32760, lsl #48
	orr x1, x1, x11
	b L11
L16:
	fmov d0, x16
	fcmp d0, d0
	mov x11, #9221120237041090560
	csel x1, x11, x16, vs
	b L11
L1:
L17:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
L18:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov x2, x1
	bl core::slice::index::slice_index_fail
	brk #0x1
	bl core::panicking::panic_cannot_unwind
	.loh AdrpAdd	L13, L14
	.loh AdrpAdd	L17, L18
