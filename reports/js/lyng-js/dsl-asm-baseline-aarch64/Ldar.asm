lyng_js_vm::vm::dispatch_handlers::loads::op_ldar:
L0:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	mov x9, x0
	ldr w0, [x0, #56]
	ldr x11, [x9, #128]
	ldr x1, [x11, #56]
	subs x12, x1, x0
	b.lo L1
	mov x10, #33
	movk x10, #32768, lsl #48
	cmp x12, #1
	b.ls L2
	ldr x12, [x11, #48]
	add x12, x12, x0
	ldrb w14, [x12, #1]
	ldr w12, [x9, #20]
	ldr x13, [x9, #80]
	add w14, w12, w14
	ldr x15, [x13, #24]
	ldr x14, [x15, w14, uxtw #3]
	str x14, [x15, x12, lsl #3]
	add w15, w0, #2
	str w15, [x9, #56]
	ldr x16, [x11, #48]
	ldrb w15, [x16, w15, uxtw]
	cmp x15, #151
	b.hi L3
L4:
	adrp x16, l_anon.10973c97f4c1e8e1c8050bb28bd48097.622@PAGE
L5:
	add x16, x16, l_anon.10973c97f4c1e8e1c8050bb28bd48097.622@PAGEOFF
	ldrb w16, [x16, x15]
	add w16, w16, #125
	and w17, w16, #0xff
	cmp w17, #7
	b.hi L3
	add w12, w12, w16, uxtb
	ldr x13, [x13, #24]
	str x14, [x13, w12, uxtw #3]
	add w12, w0, #3
	str w12, [x9, #56]
	ldr x9, [x11, #48]
	ldrb w9, [x9, w12, uxtw]
L6:
	adrp x11, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
L7:
	add x11, x11, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x9, [x11, x9, lsl #3]
	b L8
L2:
	ldr w9, [x9, #4]
	lsr w11, w0, #16
	sub x10, x10, #12
	str x10, [x8]
	str w9, [x8, #8]
	strh w0, [x8, #12]
	strh w11, [x8, #14]
	mov w9, #2
	str w9, [x8, #16]
	b L9
L3:
L10:
	adrp x9, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
L11:
	add x9, x9, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x9, [x9, x15, lsl #3]
L8:
	stp x10, x9, [x8]
L9:
	ldp x29, x30, [sp], #16
	ret
L1:
L12:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
L13:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov x2, x1
	bl core::slice::index::slice_index_fail
	brk #0x1
	bl core::panicking::panic_cannot_unwind
	.loh AdrpAdd	L4, L5
	.loh AdrpAdd	L6, L7
	.loh AdrpAdd	L10, L11
	.loh AdrpAdd	L12, L13
