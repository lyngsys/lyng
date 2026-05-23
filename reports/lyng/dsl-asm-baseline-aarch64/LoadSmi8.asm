lyng_js_vm::vm::dispatch_handlers::loads::op_load_smi8:
L0:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	mov x9, x0
	ldr w0, [x0, #56]
	ldr x11, [x9, #128]
	ldr x1, [x11, #56]
	subs x10, x1, x0
	b.lo L1
	cmp x10, #2
	b.ls L2
	ldr x10, [x11, #48]
	add x12, x10, x0
	ldrb w10, [x12, #1]
	ldr w14, [x9, #20]
	ldr x13, [x9, #80]
	ldr x1, [x13, #32]
	add w10, w14, w10
	cmp x1, x10
	b.ls L3
	ldrsb x12, [x12, #2]
	ldr x13, [x13, #24]
	mov x14, #17179869184
	movk x14, #32760, lsl #48
	bfxil x14, x12, #0, #32
	str x14, [x13, x10, lsl #3]
	add w10, w0, #3
	str w10, [x9, #56]
	ldr x9, [x11, #48]
	ldrb w9, [x9, w10, uxtw]
L4:
	adrp x10, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
L5:
	add x10, x10, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x9, [x10, x9, lsl #3]
	mov x10, #21
	movk x10, #32768, lsl #48
	add x10, x10, #12
	stp x10, x9, [x8]
	b L6
L2:
	ldr w9, [x9, #4]
	mov x10, #21
	movk x10, #32768, lsl #48
	str x10, [x8]
	stp w9, w0, [x8, #8]
	mov w9, #3
	str w9, [x8, #20]
L6:
	ldp x29, x30, [sp], #16
	ret
L1:
L7:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
L8:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov x2, x1
	bl core::slice::index::slice_index_fail
	b L9
L3:
L10:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.38@PAGE
L11:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.38@PAGEOFF
	mov x0, x10
	bl core::panicking::panic_bounds_check
L9:
	brk #0x1
	bl core::panicking::panic_cannot_unwind
	.loh AdrpAdd	L4, L5
	.loh AdrpAdd	L7, L8
	.loh AdrpAdd	L10, L11
