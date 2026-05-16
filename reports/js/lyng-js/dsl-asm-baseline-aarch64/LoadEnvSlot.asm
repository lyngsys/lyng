lyng_js_vm::vm::dispatch_handlers::scope::op_load_env_slot:
L0:
	sub sp, sp, #176
	stp x28, x27, [sp, #80]
	stp x26, x25, [sp, #96]
	stp x24, x23, [sp, #112]
	stp x22, x21, [sp, #128]
	stp x20, x19, [sp, #144]
	stp x29, x30, [sp, #160]
	add x29, sp, #160
	mov x19, x8
	ldr w5, [x0, #4]
	ldr w6, [x0, #56]
	ldrb w3, [x0, #148]
	mov w8, #152
	strb w8, [x0, #148]
	ldr x26, [x0, #128]
	ldr x1, [x26, #56]
	subs x2, x1, x6
	b.lo L1
	mov x20, x0
	mov x25, #33
	movk x25, #32768, lsl #48
	ldr x8, [x26, #48]
	add x1, x8, x6
	cmp w3, #152
	b.ne L2
	cmp x2, #3
	b.ls L3
	ldrb w22, [x1, #1]
	ldrh w28, [x1, #2]
	mov w27, #4
L4:
	lsr w4, w28, #24
	ldr w3, [x20, #68]
	ldp x21, x23, [x20, #80]
	add x0, sp, #32
	and w5, w28, #0xffffff
	mov x1, x21
	mov x2, x23
	bl lyng_js_vm::vm::loop_iteration::<impl lyng_js_vm::vm::Vm>::environment_for_slot_access
	ldr x8, [sp, #32]
	cmp x8, x25
	b.ne L5
	ldr w24, [sp, #40]
	and w2, w28, #0xffffff
	mov x0, x23
	mov x1, x24
	bl lyng_js_env::agent::environments::<impl lyng_js_env::agent::Agent>::environment_slot
	tbz w0, #0, L6
	mov x8, #2
	movk x8, #9, lsl #32
	movk x8, #32760, lsl #48
	cmp x1, x8
	b.ne L7
	mov x0, x23
	mov w1, #3
	bl lyng_js_ops::errors::error_value
	mov x22, x0
	ldr x27, [x20, #136]
	cbz x27, L8
	sub x8, x27, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs L8
	ldr x9, [x21, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldp q0, q1, [x20, #32]
	stp q0, q1, [x8, #32]
	ldr q0, [x20, #64]
	str q0, [x8, #64]
	ldp q1, q0, [x20]
	stp q1, q0, [x8]
L8:
	add x0, sp, #32
	mov x1, x21
	mov x2, x23
	mov x3, x22
	bl lyng_js_vm::vm::exceptions::<impl lyng_js_vm::vm::Vm>::transfer_to_exception_handler
	ldr x8, [sp, #32]
	ldrb w24, [sp, #40]
	cmp x8, x25
	b.ne L9
	tbz w24, #0, L10
	ldr w8, [x21, #1640]
	add w8, w8, #1
	str w8, [x21, #1640]
	cbz x27, L11
	sub x8, x27, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs L11
	ldr x9, [x21, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldp q0, q1, [x8, #32]
	stp q0, q1, [x20, #32]
	ldr q0, [x8, #64]
	str q0, [x20, #64]
	ldp q1, q0, [x8]
	stp q1, q0, [x20]
L11:
	ldr x8, [x26, #48]
	ldr w9, [x20, #56]
	ldrb w8, [x8, x9]
	b L12
L3:
	sub x8, x25, #12
	str x8, [sp, #32]
	stp w5, w6, [sp, #40]
L5:
	ldp q0, q1, [sp, #32]
	stp q0, q1, [x19]
	ldr q0, [sp, #64]
	str q0, [x19, #32]
L13:
	ldp x29, x30, [sp, #160]
	ldp x20, x19, [sp, #144]
	ldp x22, x21, [sp, #128]
	ldp x24, x23, [sp, #112]
	ldp x26, x25, [sp, #96]
	ldp x28, x27, [sp, #80]
	add sp, sp, #176
	ret
L6:
	mov x10, #0
	lsr w9, w24, #8
	sub x8, x25, #24
L14:
	lsl w9, w9, #8
	bfxil x9, x24, #0, #8
	orr x9, x9, x10
	stp x8, x9, [x19]
	str x22, [x19, #16]
	ldr q0, [sp]
	stur q0, [x19, #24]
	ldr x8, [sp, #16]
	str x8, [x19, #40]
	b L13
L7:
	ldr w8, [x20, #20]
	add w8, w8, w22
	ldr x9, [x21, #24]
	str x1, [x9, w8, uxtw #3]
	ldr w8, [x20, #56]
	add w8, w8, w27
	str w8, [x20, #56]
	ldr x9, [x26, #48]
	ldrb w8, [x9, w8, uxtw]
L12:
L15:
	adrp x9, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
L16:
	add x9, x9, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x8, [x9, x8, lsl #3]
	stp x25, x8, [x19]
	b L13
L9:
	ldrb w9, [sp, #43]
	ldurh w10, [sp, #41]
	orr w9, w10, w9, lsl #16
	ldr x22, [sp, #48]
	ldur q0, [sp, #56]
	str q0, [sp]
	ldr x10, [sp, #72]
	str x10, [sp, #16]
	ldr w10, [sp, #44]
	lsl x10, x10, #32
	b L14
L10:
	mov w9, #0
	mov w24, #0
	mov x10, #0
	mov x8, #-9223372036854775808
	b L14
L1:
L17:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
L18:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov x0, x6
	mov x2, x1
	bl core::slice::index::slice_index_fail
	brk #0x1
L2:
	add x0, sp, #32
	mov w4, #0
	bl lyng_js_vm::vm::dispatch::decode_abx_operands_wide
	ldr x8, [sp, #32]
	cmp x8, x25
	b.ne L5
	ldrh w22, [sp, #48]
	ldr w28, [sp, #40]
	ldr w27, [sp, #52]
	b L4
	bl core::panicking::panic_cannot_unwind
	bl core::panicking::panic_cannot_unwind
	.loh AdrpAdd	L15, L16
	.loh AdrpAdd	L17, L18
