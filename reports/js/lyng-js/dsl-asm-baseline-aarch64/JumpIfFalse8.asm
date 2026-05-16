lyng_js_vm::vm::dispatch_handlers::control_flow::op_jump_if_false8:
L0:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	mov x1, x0
	ldr w0, [x0, #56]
	ldr x9, [x1, #128]
	ldr x2, [x9, #56]
	subs x10, x2, x0
	b.lo L1
	cmp x10, #2
	b.ls L2
	ldr x9, [x9, #48]
	add x9, x9, x0
	ldrb w2, [x9, #1]
	ldrsb w3, [x9, #2]
	mov x0, x8
	mov w4, #3
	mov w5, #0
	bl lyng_js_vm::vm::dispatch_handlers::control_flow::op_jump_if_impl
	b L3
L2:
	ldr w9, [x1, #4]
	mov x10, #21
	movk x10, #32768, lsl #48
	str x10, [x8]
	stp w9, w0, [x8, #8]
	mov w9, #3
	str w9, [x8, #20]
L3:
	ldp x29, x30, [sp], #16
	ret
L1:
L4:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
L5:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov x1, x2
	bl core::slice::index::slice_index_fail
	brk #0x1
	bl core::panicking::panic_cannot_unwind
	.loh AdrpAdd	L4, L5
