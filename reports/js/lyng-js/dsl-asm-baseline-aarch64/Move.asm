lyng_js_vm::vm::dispatch_handlers::loads::op_move:
L0:
	sub sp, sp, #112
	stp x24, x23, [sp, #48]
	stp x22, x21, [sp, #64]
	stp x20, x19, [sp, #80]
	stp x29, x30, [sp, #96]
	add x29, sp, #96
	ldr w4, [x0, #4]
	ldr w19, [x0, #56]
	ldrb w9, [x0, #148]
	mov w10, #152
	strb w10, [x0, #148]
	ldr x20, [x0, #128]
	ldr x1, [x20, #56]
	subs x2, x1, x19
	b.lo L1
	mov x21, #33
	movk x21, #32768, lsl #48
	ldr x10, [x20, #48]
	add x1, x10, x19
	cmp w9, #152
	b.ne L2
	cmp x2, #3
	b.ls L3
	ldrb w9, [x1, #1]
	ldrb w11, [x1, #2]
	mov w10, #4
L4:
	ldr w12, [x0, #20]
	ldr x13, [x0, #80]
	add w11, w12, w11
	ldr x13, [x13, #24]
	ldr x11, [x13, w11, uxtw #3]
	add w9, w12, w9
	str x11, [x13, w9, uxtw #3]
	add w9, w10, w19
	str w9, [x0, #56]
	ldr x10, [x20, #48]
	ldrb w9, [x10, w9, uxtw]
L5:
	adrp x10, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
L6:
	add x10, x10, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x9, [x10, x9, lsl #3]
	stp x21, x9, [x8]
	b L7
L3:
	sub x9, x21, #12
	str x9, [sp]
	stp w4, w19, [sp, #8]
L8:
	ldp q0, q1, [sp]
	stp q0, q1, [x8]
	ldr q0, [sp, #32]
	str q0, [x8, #32]
L7:
	ldp x29, x30, [sp, #96]
	ldp x20, x19, [sp, #80]
	ldp x22, x21, [sp, #64]
	ldp x24, x23, [sp, #48]
	add sp, sp, #112
	ret
L1:
L9:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
L10:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov x0, x19
	mov x2, x1
	bl core::slice::index::slice_index_fail
	brk #0x1
L2:
	mov x22, x0
	mov x0, sp
	mov w3, #0
	mov x5, x19
	mov x23, x8
	bl lyng_js_vm::vm::dispatch::decode_abc_operands_wide
	mov x8, x23
	ldr x9, [sp]
	cmp x9, x21
	b.ne L8
	ldrh w9, [sp, #12]
	ldrh w11, [sp, #14]
	ldr w10, [sp, #20]
	mov x0, x22
	b L4
	bl core::panicking::panic_cannot_unwind
	.loh AdrpAdd	L5, L6
	.loh AdrpAdd	L9, L10
