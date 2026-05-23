lyng_vm::vm::dispatch_handlers::names::op_load_this:
L0:
	sub sp, sp, #144
	stp x26, x25, [sp, #64]
	stp x24, x23, [sp, #80]
	stp x22, x21, [sp, #96]
	stp x20, x19, [sp, #112]
	stp x29, x30, [sp, #128]
	add x29, sp, #128
	mov x19, x8
	ldr w5, [x0, #4]
	ldr w6, [x0, #56]
	ldrb w3, [x0, #148]
	mov w8, #152
	strb w8, [x0, #148]
	ldr x25, [x0, #128]
	ldr x1, [x25, #56]
	subs x2, x1, x6
	b.lo L1
	mov x20, x0
	mov x24, #33
	movk x24, #32768, lsl #48
	ldr x8, [x25, #48]
	add x1, x8, x6
	cmp w3, #152
	b.ne L2
	cmp x2, #3
	b.ls L3
	ldrb w23, [x1, #1]
	mov w22, #4
	ldr x21, [x20, #88]
	ldr x8, [x21, #5240]
	cbz x8, L4
L5:
	ldr x9, [x21, #5232]
	mov w10, #56
	madd x8, x8, x10, x9
	ldp x9, x8, [x8, #-56]
	ldr x26, [x20, #40]
	cmp x9, #3
	csel x8, x26, x8, eq
	mov w10, #2
	csel x9, x10, x9, eq
	cbz x9, L6
	mov x26, x8
	cmp x9, #1
	b.ne L7
	mov x0, x21
	mov w1, #3
	bl lyng_ops::errors::error_value
	mov w10, #0
	mov w8, #0
	lsr x17, x0, #32
	mov x9, #-9223372036854775808
	b L8
L3:
	sub x8, x24, #12
	str x8, [sp, #16]
	stp w5, w6, [sp, #24]
L9:
	ldp q0, q1, [sp, #16]
	stp q0, q1, [x19]
	ldr q0, [sp, #48]
	str q0, [x19, #32]
	b L10
L6:
	ldr w2, [x20, #68]
	add x0, sp, #16
	mov x1, x21
	bl lyng_vm::vm::bytecode_calls::<impl lyng_vm::vm::Vm>::this_environment_record
	ldr x9, [sp, #16]
	cmp x9, x24
	b.ne L11
	ldrb w8, [sp, #60]
	cmp w8, #1
	b.gt L12
	cbz w8, L7
	mov x0, x21
	mov w1, #3
	bl lyng_ops::errors::error_value
	mov w10, #0
	mov w8, #0
	lsr x17, x0, #32
	mov x9, #-9223372036854775808
	b L8
L11:
	ldp w8, w10, [sp, #24]
	ldp w0, w17, [sp, #32]
	ldr x15, [sp, #40]
	ldp w14, w13, [sp, #48]
	ldr w12, [sp, #56]
	ldrb w11, [sp, #60]
	ldurh w16, [sp, #61]
	strh w16, [sp, #8]
	ldrb w16, [sp, #63]
	strb w16, [sp, #10]
L8:
	mov x16, #-9223372036854775808
	cmp x9, x16
	b.ne L13
	cbnz w8, L13
	ldr x23, [x20, #80]
	ldr x26, [x20, #136]
	mov w8, w0
	orr x22, x8, x17, lsl #32
	cbz x26, L14
	sub x8, x26, #1
	ldr x9, [x23, #56]
	cmp x8, x9
	b.hs L14
	ldr x9, [x23, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldp q0, q1, [x20, #32]
	stp q0, q1, [x8, #32]
	ldr q0, [x20, #64]
	str q0, [x8, #64]
	ldp q1, q0, [x20]
	stp q1, q0, [x8]
L14:
	add x0, sp, #16
	mov x1, x23
	mov x2, x21
	mov x3, x22
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
	ldr x9, [sp, #16]
	ldrb w8, [sp, #24]
	cmp x9, x24
	b.ne L15
	tbz w8, #0, L16
	ldr w8, [x23, #1640]
	add w8, w8, #1
	str w8, [x23, #1640]
	cbz x26, L17
	sub x8, x26, #1
	ldr x9, [x23, #56]
	cmp x8, x9
	b.hs L17
	ldr x9, [x23, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldp q0, q1, [x8, #32]
	stp q0, q1, [x20, #32]
	ldr q0, [x8, #64]
	str q0, [x20, #64]
	ldp q1, q0, [x8]
	stp q1, q0, [x20]
L17:
	ldr x8, [x25, #48]
	ldr w9, [x20, #56]
	ldrb w8, [x8, x9]
	b L18
L13:
	lsr w16, w8, #8
	mov w0, w0
	orr x22, x0, x17, lsl #32
	ldrh w17, [sp, #8]
	strh w17, [sp, #12]
	ldrb w17, [sp, #10]
	b L19
L15:
	ldrb w10, [sp, #27]
	ldurh w11, [sp, #25]
	orr w16, w11, w10, lsl #16
	ldr w10, [sp, #28]
	ldp x22, x15, [sp, #32]
	ldp w14, w13, [sp, #48]
	ldr w12, [sp, #56]
	ldrb w11, [sp, #60]
	ldurh w17, [sp, #61]
	strh w17, [sp, #12]
	ldrb w17, [sp, #63]
L19:
	strb w17, [sp, #14]
L20:
	lsl w16, w16, #8
	bfxil x16, x8, #0, #8
	orr x8, x16, x10, lsl #32
	stp x9, x8, [x19]
	stp x22, x15, [x19, #16]
	stp w14, w13, [x19, #32]
	str w12, [x19, #40]
	strb w11, [x19, #44]
	ldrh w8, [sp, #12]
	sturh w8, [x19, #45]
	ldrb w8, [sp, #14]
	strb w8, [x19, #47]
	b L10
L12:
	cmp w8, #2
	b.ne L7
	ldr x26, [sp, #40]
	b L7
L16:
	mov w16, #0
	mov w8, #0
	mov x9, #-9223372036854775808
	b L20
L1:
L21:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
L22:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov x0, x6
	mov x2, x1
	bl core::slice::index::slice_index_fail
	brk #0x1
L2:
	add x0, sp, #16
	mov w4, #0
	bl lyng_vm::vm::dispatch::decode_abx_operands_wide
	ldr x8, [sp, #16]
	cmp x8, x24
	b.ne L9
	ldrh w23, [sp, #32]
	ldr w22, [sp, #36]
	ldr x21, [x20, #88]
	ldr x8, [x21, #5240]
	cbnz x8, L5
L4:
	ldr x26, [x20, #40]
L7:
	ldr x8, [x20, #80]
	ldr w9, [x20, #20]
	add w9, w9, w23
	ldr x8, [x8, #24]
	str x26, [x8, w9, uxtw #3]
	ldr w8, [x20, #56]
	add w8, w8, w22
	str w8, [x20, #56]
	ldr x9, [x25, #48]
	ldrb w8, [x9, w8, uxtw]
L18:
L23:
	adrp x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
L24:
	add x9, x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x8, [x9, x8, lsl #3]
	stp x24, x8, [x19]
L10:
	ldp x29, x30, [sp, #128]
	ldp x20, x19, [sp, #112]
	ldp x22, x21, [sp, #96]
	ldp x24, x23, [sp, #80]
	ldp x26, x25, [sp, #64]
	add sp, sp, #144
	ret
	bl core::panicking::panic_cannot_unwind
	bl core::panicking::panic_cannot_unwind
	.loh AdrpAdd	L21, L22
	.loh AdrpAdd	L23, L24
