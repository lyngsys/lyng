lyng_js_vm::vm::dispatch_handlers::arithmetic::op_sub_smi_slow:
L0:
	sub sp, sp, #272
	stp x28, x27, [sp, #176]
	stp x26, x25, [sp, #192]
	stp x24, x23, [sp, #208]
	stp x22, x21, [sp, #224]
	stp x20, x19, [sp, #240]
	stp x29, x30, [sp, #256]
	add x29, sp, #256
	mov x20, x1
	mov x19, x0
	ldr x1, [x1, #80]
	ldr x8, [x1, #32]
	ldr w9, [x20, #20]
	add w0, w9, w3, uxth
	cmp x8, x0
	b.ls L1
	mov x21, x5
	mov x22, x2
	ldp x2, x10, [x20, #88]
	ldp x11, x12, [x20, #104]
	ldr x13, [x20, #120]
	mov x24, #33
	movk x24, #32768, lsl #48
	mov x27, #281470681743360
	movk x27, #32760, lsl #48
	mov x9, #17179869184
	movk x9, #32760, lsl #48
	ldr x8, [x1, #24]
	ldr x8, [x8, x0, lsl #3]
	and x14, x8, x27
	cmp x14, x9
	b.ne L2
	subs w14, w8, w4, sxth
	b.vc L3
L2:
	mov x14, #9221120237041090560
	bic x15, x14, x8
	ubfx x14, x8, #32, #16
	sub w16, w14, #1
	cmp x15, #0
	ccmp w16, #9, #2, eq
	b.ls L4
	fmov d0, x8
	sxth w8, w4
	scvtf d1, w8
	fsub d0, d0, d1
	fmov x8, d0
	fcmp d0, #0.0
	ccmp x8, #0, #4, eq
	b.ne L5
	fmov x10, d0
	and x10, x10, #0x7fffffffffffffff
	mov x11, #9218868437227405311
	cmp x10, x11
	frintz d1, d0
	mov x10, #281474972516352
	movk x10, #16863, lsl #48
	fmov d2, x10
	fccmp d0, d2, #2, le
	mov x10, #-4476578029606273024
	fmov d2, x10
	fccmp d0, d2, #8, ls
	fccmp d0, d1, #0, ge
	b.eq L6
	mov x9, #9221120237041090560
	fcmp d0, d0
	csel x8, x9, x8, vs
	b L5
L4:
	cmp w14, #4
	b.ne L7
	scvtf d0, w8
	sxth w8, w4
	scvtf d1, w8
	fsub d0, d0, d1
	fcmp d0, #0.0
	fmov x8, d0
	ccmp x8, #0, #4, eq
	cset w10, ne
	and x11, x8, #0x7fffffffffffffff
	mov x12, #9218868437227405311
	cmp x11, x12
	b.gt L5
	tbnz w10, #0, L5
	frintz d1, d0
	mov x10, #281474972516352
	movk x10, #16863, lsl #48
	fmov d2, x10
	fcmp d0, d2
	mov x10, #-4476578029606273024
	fmov d2, x10
	fccmp d0, d2, #8, ls
	fccmp d0, d1, #0, ge
	b.ne L5
L6:
	fcvtzs w8, d0
	orr x8, x8, x9
L5:
	stp x24, x8, [sp, #16]
L8:
	ldp x1, x2, [x20, #80]
	b L9
L3:
	orr x8, x14, x9
	stp x24, x8, [sp, #16]
L9:
	ldr x3, [x20, #136]
	add x8, sp, #16
	str x8, [sp]
	add x0, sp, #112
	mov x4, x20
	mov x5, x6
	mov x6, x21
	mov x7, x22
	bl lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::finish_abc_value_result
	ldr x8, [sp, #112]
	cmp x8, x24
	b.ne L10
	ldr x8, [x20, #128]
	ldr x8, [x8, #48]
	ldr w9, [x20, #56]
	ldrb w8, [x8, x9]
L11:
	adrp x9, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
L12:
	add x9, x9, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x8, [x9, x8, lsl #3]
	stp x24, x8, [x19]
	b L13
L10:
	ldp q0, q1, [sp, #112]
	stp q0, q1, [x19]
	ldr q0, [sp, #144]
	str q0, [x19, #32]
L13:
	ldp x29, x30, [sp, #256]
	ldp x20, x19, [sp, #240]
	ldp x22, x21, [sp, #224]
	ldp x24, x23, [sp, #208]
	ldp x26, x25, [sp, #192]
	ldp x28, x27, [sp, #176]
	add sp, sp, #272
	ret
L7:
	mov x26, x4
	mov x25, x6
	stp x1, x2, [sp, #112]
	mov x23, x2
	stp x10, x11, [sp, #128]
	stp x12, x13, [sp, #144]
	str x20, [sp, #160]
	add x0, sp, #64
	add x1, sp, #112
	mov x2, x8
	mov w3, #2
	bl lyng_js_ops::object::conversions::to_primitive
	ldp x8, x1, [sp, #64]
	cmp x8, x24
	b.ne L14
	add x8, sp, #112
	mov x0, x23
	bl lyng_js_ops::read::to_numeric
	ldr w28, [sp, #112]
	cmp w28, #4
	b.ne L15
	ldr x1, [sp, #120]
	and x9, x1, #0x7ff8000000000000
	ubfx x8, x1, #32, #16
	mov x10, #9221120237041090560
	cmp x9, x10
	b.ne L16
	sub w11, w8, #1
	cmp w11, #9
	b.hi L16
	mov w11, #1
	lsl w11, w11, w8
	mov w12, #1662
	tst w11, w12
	b.ne L17
	cmp w8, #7
	b.ne L18
L16:
	cmp x9, x10
	b.ne L17
	sub w8, w8, #7
	cmp w8, #2
	b.hs L17
	mov x0, x23
	mov w1, #5
	bl lyng_js_ops::errors::error_value
	movi d0, #0000000000000000
	b L19
L14:
	ldr x0, [sp, #80]
	ldur q0, [sp, #88]
	str q0, [sp, #112]
	ldr x9, [sp, #104]
	str x9, [sp, #128]
	b L20
L15:
	ldr w8, [sp, #116]
	ldr x0, [sp, #120]
	cbnz w28, L21
	and x9, x0, x27
	mov x10, #4294967296
	movk x10, #32760, lsl #48
	cmp x9, x10
	b.ne L21
	mov x0, x23
	mov w1, #5
	bl lyng_js_ops::errors::error_value
L21:
	orr x1, x28, x8, lsl #32
	mov x8, #-9223372036854775808
L20:
	mov x6, x25
	stp x8, x1, [sp, #16]
	str x0, [sp, #32]
	ldr q0, [sp, #112]
	stur q0, [sp, #40]
	ldr x8, [sp, #128]
	str x8, [sp, #56]
	b L8
L17:
	add x8, sp, #112
	mov x0, x23
	bl lyng_js_ops::read::to_number
	ldr w8, [sp, #112]
	cmp w8, #4
	b.ne L22
	ldr x8, [sp, #120]
	mov x9, #9221120237041090560
	bic x9, x9, x8
	cbnz x9, L23
	ubfx x9, x8, #32, #16
	sub w10, w9, #1
	cmp w10, #9
	b.hi L23
	cmp w9, #4
	b.ne L24
	scvtf d0, w8
	b L25
L22:
	ldr d0, [sp, #112]
	ldr x0, [sp, #120]
L19:
	mov x6, x25
	mov x8, #-9223372036854775808
	str x8, [sp, #16]
	str d0, [sp, #24]
	str x0, [sp, #32]
	b L8
L23:
	fmov d0, x8
L25:
	sxth w8, w26
	scvtf d1, w8
	fsub d0, d0, d1
	bl lyng_js_vm::vm::values::encode_number
	stp x24, x0, [sp, #16]
	mov x6, x25
	b L8
L18:
	mov x0, x23
	mov w1, #5
	bl lyng_js_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #16]
	str wzr, [sp, #24]
	str x0, [sp, #32]
	mov x6, x25
	b L8
L1:
L26:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.61@PAGE
L27:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.61@PAGEOFF
	mov x1, x8
	bl core::panicking::panic_bounds_check
L24:
L28:
	adrp x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.74@PAGE
L29:
	add x0, x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.74@PAGEOFF
L30:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.681@PAGE
L31:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.681@PAGEOFF
	mov w1, #44
	bl core::option::expect_failed
	.loh AdrpAdd	L11, L12
	.loh AdrpAdd	L26, L27
	.loh AdrpAdd	L30, L31
	.loh AdrpAdd	L28, L29
L32:
lyng_js_vm::vm::dispatch_handlers::arithmetic::op_greater_equal:
L33:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	mov x1, x0
	mov x0, x8
L34:
	adrp x2, lyng_js_vm::vm::dispatch::arithmetic::<impl lyng_js_vm::vm::Vm>::execute_greater_equal_opcode@PAGE
L35:
	add x2, x2, lyng_js_vm::vm::dispatch::arithmetic::<impl lyng_js_vm::vm::Vm>::execute_greater_equal_opcode@PAGEOFF
	bl lyng_js_vm::vm::dispatch_handlers::arithmetic::op_binary_general
	ldp x29, x30, [sp], #16
	ret
	bl core::panicking::panic_cannot_unwind
	.loh AdrpAdd	L34, L35
