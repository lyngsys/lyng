lyng_vm::vm::dispatch_handlers::loads::op_store_local_3:
L0:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	mov x9, x0
	ldr w0, [x0, #56]
	ldr x11, [x9, #128]
	ldr x1, [x11, #56]
	subs x12, x1, x0
	b.lo L1
	mov x10, #21
	movk x10, #32768, lsl #48
	cmp x12, #1
	b.ls L2
	ldr x12, [x11, #48]
	add x12, x12, x0
	ldrb w12, [x12, #1]
	ldr w13, [x9, #20]
	ldr x14, [x9, #80]
	add w12, w13, w12
	ldr x14, [x14, #24]
	ldr x12, [x14, w12, uxtw #3]
	add w13, w13, #3
	str x12, [x14, w13, uxtw #3]
	add w12, w0, #2
	str w12, [x9, #56]
	ldr x9, [x11, #48]
	ldrb w9, [x9, w12, uxtw]
L3:
	adrp x11, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
L4:
	add x11, x11, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x9, [x11, x9, lsl #3]
	str x9, [x8, #8]
	add x10, x10, #12
	b L5
L2:
	ldr w9, [x9, #4]
	lsr w11, w0, #16
	str w9, [x8, #8]
	strh w0, [x8, #12]
	strh w11, [x8, #14]
	mov w9, #2
	str w9, [x8, #16]
L5:
	str x10, [x8]
	ldp x29, x30, [sp], #16
	ret
L1:
L6:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
L7:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov x2, x1
	bl core::slice::index::slice_index_fail
	brk #0x1
	bl core::panicking::panic_cannot_unwind
	.loh AdrpAdd	L3, L4
	.loh AdrpAdd	L6, L7
