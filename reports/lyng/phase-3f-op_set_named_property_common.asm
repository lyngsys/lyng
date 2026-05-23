    Finished `release` profile [optimized] target(s) in 0.02s

.section __TEXT,__text,regular,pure_instructions
	.p2align	2
lyng_vm::vm::dispatch_handlers::property::op_set_keyed_property_common:
Lfunc_begin402:
	.cfi_startproc
	stp x28, x27, [sp, #-96]!
	.cfi_def_cfa_offset 96
	stp x26, x25, [sp, #16]
	stp x24, x23, [sp, #32]
	stp x22, x21, [sp, #48]
	stp x20, x19, [sp, #64]
	stp x29, x30, [sp, #80]
	add x29, sp, #80
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset w21, -40
	.cfi_offset w22, -48
	.cfi_offset w23, -56
	.cfi_offset w24, -64
	.cfi_offset w25, -72
	.cfi_offset w26, -80
	.cfi_offset w27, -88
	.cfi_offset w28, -96
	.cfi_remember_state
	sub sp, sp, #1664
	str xzr, [sp]
	mov x20, x1
	ldr w27, [x1, #4]
	ldr w5, [x1, #56]
	ldrb w9, [x1, #148]
	mov w8, #152
	strb w8, [x1, #148]
	ldr x10, [x1, #128]
	ldr x1, [x10, #56]
	subs x8, x1, x5
	b.lo LBB402_200
	mov x19, x0
	add x11, sp, #1440
	mov x26, #33
	movk x26, #32768, lsl #48
	ldr x10, [x10, #48]
	add x1, x10, x5
	cmp w9, #152
	b.ne LBB402_201
	and x9, x8, #0x7ffffffffffffffe
	cmp x8, #4
	ccmp x9, #4, #4, hs
	b.eq LBB402_33
	ldrh w3, [x1, #4]
	cbz w3, LBB402_33
	ldrb w9, [x1, #1]
	ldrb w10, [x1, #2]
	mov w12, #6
	ldrb w8, [x1, #3]
	ldr x21, [x20, #80]
	ldr w11, [x20, #20]
	ldr x1, [x21, #32]
	add w9, w11, w9
	cmp x1, x9
	b.ls LBB402_203
LBB402_5:
	add w0, w11, w10
	cmp x1, x0
	b.ls LBB402_361
	add w8, w11, w8
	cmp x1, x8
	b.ls LBB402_362
	ldr x28, [x20, #88]
	ldr x22, [x20, #136]
	ldr x10, [x21, #24]
	ldr x25, [x10, x9, lsl #3]
	and x24, x25, #0x7ff8000000000000
	ubfx x13, x25, #32, #16
	sub w11, w13, #1
	mov x9, #9221120237041090560
	cmp x24, x9
	ccmp w11, #1, #2, eq
	b.ls LBB402_35
	stp x22, x28, [sp, #128]
	str w12, [sp, #120]
	and w11, w2, #0xff
	ldp x26, x4, [x20, #96]
	ldp x12, x6, [x20, #112]
	str w11, [sp, #112]
	sub w11, w11, #83
	str w11, [sp, #124]
	ldr x23, [x10, x0, lsl #3]
	ldr x16, [x10, x8, lsl #3]
	and x8, x16, #0x7fffffff00000000
	and x8, x8, #0xfff8ffffffffffff
	cmp x24, x9
	ccmp w13, #5, #0, eq
	ccmp w25, #0, #4, eq
	mov x9, #17179869184
	movk x9, #32760, lsl #48
	ccmp x8, x9, #0, ne
	b.ne LBB402_45
	ldr x28, [sp, #128]
	tbnz w16, #31, LBB402_46
	stp x26, x12, [sp, #56]
	str x4, [sp, #72]
	str w3, [sp, #116]
	str x13, [sp, #104]
	cbz w3, LBB402_18
	mov x14, #1
	movk x14, #9, lsl #32
	movk x14, #32760, lsl #48
	cmp x23, x14
	b.eq LBB402_18
	ldr x8, [x21, #104]
	sub w11, w27, #1
	cmp x8, x11
	b.ls LBB402_15
	ldr x8, [x21, #96]
	add x8, x8, x11, lsl #5
	ldr x9, [x8, #16]
	sub w10, w3, #1
	cmp x9, x10
	b.ls LBB402_15
	ldr x8, [x8, #8]
	mov w9, #1216
	umaddl x8, w10, w9, x8
	ldr x9, [x8]
	cmp x9, #10
	ccmp x9, #4, #2, ne
	b.lo LBB402_154
LBB402_15:
	ldr w10, [x20, #4]
	ldr x8, [x21, #104]
	sub w27, w10, #1
	cmp x8, x27
	b.ls LBB402_18
	ldr x8, [x21, #96]
	add x8, x8, x27, lsl #5
	ldr x9, [x8, #16]
	sub w11, w3, #1
	cmp x9, x11
	b.ls LBB402_18
	ldr x8, [x8, #8]
	mov w9, #1216
	umaddl x22, w11, w9, x8
	ldr x8, [x22]
	cmp x8, #10
	ccmp x8, #3, #2, ne
	b.ls LBB402_171
LBB402_18:
	mov x22, x6
	ldr w2, [x20, #4]
	mov x0, x21
	ldr x27, [sp, #136]
	mov x1, x27
	mov x4, x25
	mov x5, x16
	mov x6, x23
	mov x26, x16
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_keyed_dense_index_store_inline_cache_hit
	and w8, w0, #0xff
	cmp w8, #2
	b.ne LBB402_81
	add x0, sp, #352
	mov x1, x21
	mov x2, x27
	mov x3, x25
	mov x4, x26
	mov x5, x23
	bl lyng_vm::vm::values::<impl lyng_vm::vm::Vm>::mapped_arguments_set
	ldr x8, [sp, #352]
	mov x28, #33
	movk x28, #32768, lsl #48
	add x9, x28, #1
	stp x9, x8, [sp, #32]
	cmp x8, x9
	str x26, [sp, #48]
	b.ne LBB402_86
	str x23, [sp, #8]
	add x0, sp, #400
	stp w25, w26, [sp]
	mov x1, x21
	mov x2, x27
	ldp x3, x5, [sp, #56]
	ldr x4, [sp, #72]
	mov x6, x22
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_set_typed_array_index
	sub x0, x29, #152
	add x5, sp, #400
	mov x1, x21
	mov x2, x27
	ldr x3, [sp, #128]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x28
	b.ne LBB402_328
	str x25, [sp, #96]
	cmp w5, #3
	ldr x28, [sp, #128]
	b.eq LBB402_252
	cmp w5, #2
	ldr x25, [sp, #96]
	ldr x3, [sp, #48]
	b.ne LBB402_29
	mov x4, x23
	add x0, sp, #448
	ldr x27, [sp, #136]
	mov x1, x27
	mov x2, x25
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_set_engine_array_index
	sub x0, x29, #152
	add x5, sp, #448
	mov x1, x21
	mov x2, x27
	mov x3, x28
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_328
	cmp w5, #3
	b.eq LBB402_252
	cmp w5, #2
	ldr x25, [sp, #96]
	ldr x3, [sp, #48]
	b.ne LBB402_29
	mov x4, x23
	add x0, sp, #496
	ldr x27, [sp, #136]
	mov x1, x27
	mov x2, x25
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_set_ordinary_index_data_property
	sub x0, x29, #152
	add x5, sp, #496
	mov x1, x21
	mov x2, x27
	mov x3, x28
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_328
	cmp w5, #3
	b.eq LBB402_252
	cmp w5, #2
	ldr w3, [sp, #116]
	mov x6, x22
	ldp x25, x13, [sp, #96]
	ldp x16, x26, [sp, #48]
	ldp x12, x4, [sp, #64]
	b.eq LBB402_46
LBB402_29:
	ldr w8, [sp, #124]
	cmp w8, #2
	ldr x23, [sp, #136]
	b.hs LBB402_94
	tbnz w5, #0, LBB402_89
	ldr w8, [sp, #112]
	cmp w8, #84
	b.ne LBB402_212
LBB402_32:
	mov x0, x23
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #544]
	str wzr, [sp, #552]
	str x0, [sp, #560]
	mov x24, #33
	movk x24, #32768, lsl #48
	b LBB402_90
LBB402_33:
	sub x8, x26, #12
	stur x8, [x29, #-152]
	stp w27, w5, [x29, #-144]
LBB402_34:
	ldur q0, [x11, #152]
	ldur q1, [x11, #168]
	stp q0, q1, [x19]
	ldur q0, [x11, #184]
	str q0, [x19, #32]
	b LBB402_332
LBB402_35:
	mov x0, x28
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x24, x0
	cbz x22, LBB402_38
	sub x8, x22, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs LBB402_38
	ldr x9, [x21, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB402_38:
	sub x0, x29, #152
	mov x1, x21
	mov x2, x28
	mov x3, x24
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x26
	b.ne LBB402_44
	tbz w5, #0, LBB402_73
	ldr w8, [x21, #1640]
	add w8, w8, #1
	str w8, [x21, #1640]
	cbz x22, LBB402_252
	sub x8, x22, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs LBB402_252
LBB402_42:
	ldr x9, [x21, #48]
LBB402_43:
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x8]
	str q0, [x20]
	ldp q0, q1, [x8, #48]
	ldp q3, q2, [x8, #16]
	stp q0, q1, [x20, #48]
	stp q3, q2, [x20, #16]
	b LBB402_252
LBB402_44:
	ldurb w9, [x29, #-141]
	add x11, sp, #1440
	ldurh w10, [x11, #161]
	orr w9, w10, w9, lsl #16
	ldur w10, [x29, #-140]
	ldur q0, [x11, #176]
	str q0, [sp, #176]
	ldur x24, [x29, #-136]
	ldur x11, [x29, #-112]
	str x11, [sp, #192]
	b LBB402_74
LBB402_45:
	mov x9, #30064771072
	movk x9, #32760, lsl #48
	cmp x8, x9
	ccmp w16, #0, #4, eq
	ldr x28, [sp, #128]
	b.ne LBB402_76
LBB402_46:
	str w3, [sp, #116]
	stp x25, x13, [sp, #96]
	stp x24, x23, [sp, #80]
	ldr x22, [sp, #136]
	stp x21, x22, [x29, #-152]
	mov x24, x26
	stp x26, x4, [x29, #-136]
	mov x23, x4
	mov x26, x12
	stp x12, x6, [x29, #-120]
	mov x25, x6
	stur x20, [x29, #-104]
	sub x0, x29, #208
	sub x1, x29, #152
	mov x2, x16
	mov w3, #1
	bl lyng_ops::object::conversions::to_primitive
	ldp x8, x5, [x29, #-208]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_62
	ldr w3, [x20, #4]
	ldr w4, [x20, #56]
	add x0, sp, #592
	mov x27, x21
	mov x1, x21
	mov x2, x22
	bl lyng_vm::vm::values::<impl lyng_vm::vm::Vm>::value_to_property_key
	mov x9, #33
	movk x9, #32768, lsl #48
	ldr x8, [sp, #592]
	ldr w5, [sp, #600]
	cmp x8, x9
	b.ne LBB402_63
	ldr w16, [sp, #604]
	ldr w3, [sp, #116]
	mov x6, x25
	mov x21, x27
	mov x12, x26
	mov x4, x23
	ldp x23, x25, [sp, #88]
	mov x26, x24
	ldr x24, [sp, #80]
	ldr x13, [sp, #104]
	ldr x27, [sp, #136]
	mov x8, #9221120237041090560
	cmp x24, x8
	ccmp w25, #0, #4, eq
	ccmp w13, #5, #0, ne
	b.ne LBB402_77
LBB402_49:
	cbz w5, LBB402_105
	mov x7, x23
	cmp w5, #1
	b.ne LBB402_131
	stp x12, x4, [sp, #64]
	str x6, [sp, #24]
	mov x8, x3
	ldr w3, [x20, #4]
	mov x14, x8
	cbz w8, LBB402_281
	ldr x9, [x21, #104]
	sub w8, w3, #1
	cmp x9, x8
	b.ls LBB402_281
	ldr x9, [x21, #96]
	add x9, x9, x8, lsl #5
	ldr x10, [x9, #16]
	sub w8, w14, #1
	cmp x10, x8
	b.ls LBB402_281
	ldr x9, [x9, #8]
	mov w10, #1216
	umaddl x9, w8, w10, x9
	ldr x8, [x9]
	cmp x8, #10
	b.eq LBB402_281
	cmp x8, #3
	b.hi LBB402_281
	ldr w8, [x9, #1204]
	cmp w8, w16
	b.ne LBB402_281
	ldr x8, [x9, #1088]
	cbz x8, LBB402_281
	ldr x10, [sp, #136]
	ldr x12, [x10, #224]
	mov w10, #-1
	add x10, x25, x10
	lsr w11, w10, #6
	cmp x11, x12
	b.hs LBB402_281
	ldr x12, [sp, #136]
	ldr x12, [x12, #216]
	and x10, x10, #0x3f
	ldr x11, [x12, x11, lsl #3]
	mov w12, #80
	umaddl x11, w10, w12, x11
	ldr w10, [x11]
	cmp w10, #1
	b.ne LBB402_281
	mov x0, x26
	str x19, [sp, #104]
	mov x17, x16
	mov x15, x25
	mov x24, x21
	mov x23, x14
	mov x27, x3
	ldr x10, [x9, #1096]
	ldp w12, w9, [x11, #52]
	ldr x11, [x11, #40]
	lsr x13, x8, #32
	cmp x13, #0
	csel w13, w13, wzr, ne
	cbz w12, LBB402_274
	cmp w12, w13
	ccmp x11, x10, #0, eq
	mov x3, x27
	ldr x28, [sp, #128]
	mov x14, x23
	mov x21, x24
	mov x25, x15
	mov x16, x17
	ldr x19, [sp, #104]
	mov x26, x0
	b.eq LBB402_276
	b LBB402_281
LBB402_62:
	mov x27, x21
	ldp q0, q1, [x29, #-192]
	stp q0, q1, [sp, #608]
	str x5, [sp, #600]
LBB402_63:
	mov x9, #-9223372036854775808
	cmp x8, x9
	b.ne LBB402_75
	cbnz w5, LBB402_75
	mov x22, #33
	movk x22, #32768, lsl #48
	ldr x24, [sp, #608]
	cbz x28, LBB402_68
	sub x8, x28, #1
	ldr x9, [x27, #56]
	cmp x8, x9
	b.hs LBB402_68
	ldr x9, [x27, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB402_68:
	sub x0, x29, #152
	mov x1, x27
	ldr x2, [sp, #136]
	mov x3, x24
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_83
	tbz w5, #0, LBB402_104
	ldr w8, [x27, #1640]
	add w8, w8, #1
	str w8, [x27, #1640]
	cbz x28, LBB402_252
	sub x8, x28, #1
	ldr x9, [x27, #56]
	cmp x8, x9
	b.hs LBB402_252
	ldr x9, [x27, #48]
	b LBB402_43
LBB402_73:
	mov w9, #0
	mov w5, #0
	mov x8, #-9223372036854775808
LBB402_74:
	ldr q0, [sp, #176]
	str q0, [sp, #144]
	ldr x11, [sp, #192]
	b LBB402_330
LBB402_75:
	lsr w9, w5, #8
	ldr w10, [sp, #604]
	ldr x24, [sp, #608]
	add x11, sp, #361
	ldur q0, [x11, #255]
	str q0, [sp, #640]
	ldr x11, [sp, #632]
	b LBB402_84
LBB402_76:
	mov w5, #2
	str w5, [sp, #600]
	str w16, [sp, #604]
	mov x8, #33
	movk x8, #32768, lsl #48
	str x8, [sp, #592]
	ldr x27, [sp, #136]
	mov x8, #9221120237041090560
	cmp x24, x8
	ccmp w25, #0, #4, eq
	ccmp w13, #5, #0, ne
	b.eq LBB402_49
LBB402_77:
	str x23, [sp, #16]
	add x0, sp, #1440
	stp w5, w16, [sp, #8]
	str x25, [sp]
	mov x1, x21
	mov x2, x27
	mov x3, x26
	mov x5, x12
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::set_property_on_value
	ldr x8, [sp, #1440]
	mov x22, #33
	movk x22, #32768, lsl #48
	cmp x8, x22
	b.ne LBB402_95
	ldr w8, [sp, #124]
	cmp w8, #2
	b.hs LBB402_250
	ldrb w8, [sp, #1448]
	tbz w8, #0, LBB402_148
LBB402_80:
	stur x22, [x29, #-256]
	b LBB402_150
LBB402_81:
	ldr w8, [sp, #124]
	cmp w8, #2
	b.hs LBB402_250
	mov x22, #33
	movk x22, #32768, lsl #48
	str x22, [sp, #304]
	sub x0, x29, #152
	add x5, sp, #304
	b LBB402_248
LBB402_83:
	ldurb w9, [x29, #-141]
	add x11, sp, #1440
	ldurh w10, [x11, #161]
	orr w9, w10, w9, lsl #16
	ldur w10, [x29, #-140]
	ldur q0, [x11, #176]
	str q0, [sp, #640]
	ldur x24, [x29, #-136]
	ldur x11, [x29, #-112]
LBB402_84:
	str x11, [sp, #656]
LBB402_85:
	ldr q0, [sp, #640]
	str q0, [sp, #144]
	ldr x11, [sp, #656]
	b LBB402_330
LBB402_86:
	sub x0, x29, #152
	add x5, sp, #352
	mov x1, x21
	mov x2, x27
	ldr x3, [sp, #128]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x28
	b.ne LBB402_328
	ldp x28, x23, [sp, #128]
	tbz w5, #0, LBB402_252
	ldr w8, [sp, #124]
	cmp w8, #2
	b.hs LBB402_93
LBB402_89:
	mov x24, #33
	movk x24, #32768, lsl #48
	str x24, [sp, #544]
LBB402_90:
	sub x0, x29, #152
	add x5, sp, #544
	mov x1, x21
	mov x2, x23
	mov x3, x28
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x24
	b.ne LBB402_328
	tbz w5, #0, LBB402_252
	ldp x9, x8, [sp, #32]
	cmp x8, x9
	b.eq LBB402_94
LBB402_93:
	sub x0, x29, #152
	mov x1, x23
	mov x2, x25
	bl lyng_vm::vm::activation_objects::<impl lyng_vm::vm::Vm>::sync_engine_array_length
	ldur x8, [x29, #-152]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_272
LBB402_94:
	ldr w2, [x20, #4]
	mov x0, x21
	mov x1, x23
	ldr w3, [sp, #116]
	mov x4, x25
	ldr x5, [sp, #48]
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_keyed_index_access
	ldr w8, [x20, #56]
	ldr w9, [sp, #120]
	adds w8, w8, w9
	b.lo LBB402_251
	b LBB402_319
LBB402_95:
	ldr w5, [sp, #1448]
	mov x9, #-9223372036854775808
	cmp x8, x9
	b.ne LBB402_136
	cbnz w5, LBB402_136
	ldr x24, [sp, #1456]
	cbz x28, LBB402_100
	sub x8, x28, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs LBB402_100
	ldr x9, [x21, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB402_100:
	sub x0, x29, #152
	mov x1, x21
	mov x2, x27
	mov x3, x24
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_151
	tbz w5, #0, LBB402_199
	ldr w8, [x21, #1640]
	add w8, w8, #1
	str w8, [x21, #1640]
	cbz x28, LBB402_252
	sub x8, x28, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.lo LBB402_42
	b LBB402_252
LBB402_104:
	mov w9, #0
	mov w5, #0
	mov x8, #-9223372036854775808
	b LBB402_85
LBB402_105:
	str x26, [sp, #56]
	stp x23, x25, [sp, #88]
	str x4, [sp, #72]
	str w3, [sp, #116]
	cbz w3, LBB402_231
	mov x13, #1
	movk x13, #9, lsl #32
	movk x13, #32760, lsl #48
	ldr x8, [sp, #88]
	cmp x8, x13
	b.eq LBB402_231
	ldr w27, [x20, #4]
	ldr x8, [x21, #104]
	sub w23, w27, #1
	cmp x8, x23
	b.ls LBB402_137
	ldr x8, [x21, #96]
	add x8, x8, x23, lsl #5
	ldr x9, [x8, #16]
	sub w22, w3, #1
	cmp x9, x22
	b.ls LBB402_137
	ldr x8, [x8, #8]
	mov w9, #1216
	umaddl x8, w22, w9, x8
	ldr x9, [x8]
	cmp x9, #10
	b.eq LBB402_137
	cmp x9, #4
	b.hs LBB402_137
	ldr x26, [x8, #1136]
	cbz w26, LBB402_137
	mov w9, #4992
	sub x8, x29, #152
	ldr x1, [sp, #136]
	add x0, x1, x9
	ldr x2, [sp, #96]
	mov x25, x6
	str x16, [sp, #48]
	mov x24, x12
	bl lyng_objects::runtime::ObjectRuntime::object_header
	mov x13, #1
	movk x13, #9, lsl #32
	movk x13, #32760, lsl #48
	mov x12, x24
	ldr x16, [sp, #48]
	mov x6, x25
	ldr w3, [sp, #116]
	ldurb w8, [x29, #-130]
	cmp w8, #3
	b.eq LBB402_137
	cbz w26, LBB402_137
	ldur w8, [x29, #-148]
	cmp w8, w26
	b.ne LBB402_137
	ldurh w8, [x29, #-132]
	lsr x9, x26, #32
	cmp w8, w9, uxth
	b.ne LBB402_137
	ldur w8, [x29, #-136]
	cbz w8, LBB402_137
	ldr x9, [sp, #136]
	ldr x10, [x9, #640]
	sub w9, w8, #1
	cmp x10, x9
	b.ls LBB402_137
	ldr x10, [sp, #136]
	ldr x10, [x10, #632]
	mov w11, #24
	umaddl x9, w9, w11, x10
	ldrb w10, [x9, #19]
	cmp w10, #1
	b.ne LBB402_137
	ldr x11, [x9, #8]
	mov w10, w16
	cmp x11, x10
	b.ls LBB402_137
	ldr x9, [x9]
	ldr x9, [x9, x10, lsl #3]
	cmp x9, x13
	b.eq LBB402_137
	stp w16, w8, [x29, #-148]
	mov w8, #2
	stur w8, [x29, #-152]
	sub x1, x29, #152
	ldr x0, [sp, #136]
	ldr x2, [sp, #88]
	bl lyng_gc::mutator::PrimitiveMutator::store_value
	mov x13, #1
	movk x13, #9, lsl #32
	movk x13, #32760, lsl #48
	mov x12, x24
	ldr x16, [sp, #48]
	mov x6, x25
	ldr w3, [sp, #116]
	cbz w0, LBB402_137
	mov x25, x19
	mov x24, x21
	ldr x8, [x21, #104]
	cmp x8, x23
	b.ls LBB402_126
	ldr x8, [x24, #96]
	add x8, x8, x23, lsl #5
	ldr x9, [x8, #16]
	cmp x9, x22
	b.ls LBB402_126
	ldr x8, [x8, #8]
	mov w9, #1216
	umaddl x0, w22, w9, x8
	ldr x8, [x0]
	cmp x8, #10
	b.eq LBB402_126
	bl lyng_vm::vm::feedback::FeedbackSiteState::record_execution
LBB402_126:
	mov x21, x24
	ldp x0, x1, [x24, #120]
	mov x2, x27
	bl lyng_vm::vm::tiering::<impl lyng_vm::vm::Vm>::observe_tier_feedback_event
	ldr w8, [sp, #124]
	cmp w8, #2
	ldp x3, x2, [sp, #128]
	b.hs LBB402_129
	mov x22, #33
	movk x22, #32768, lsl #48
	str x22, [sp, #672]
	sub x0, x29, #152
	add x5, sp, #672
	mov x1, x21
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_273
	mov x19, x25
	tbz w5, #0, LBB402_252
LBB402_129:
	ldr w8, [x20, #56]
	ldr w9, [sp, #120]
	adds w8, w8, w9
	b.hs LBB402_319
	str w8, [x20, #56]
	mov x19, x25
	b LBB402_252
LBB402_131:
	mov x23, x3
	str x7, [sp, #16]
	add x0, sp, #1344
	stp w5, w16, [sp, #8]
	str x25, [sp]
	mov x1, x21
	mov x2, x27
	mov x3, x26
	mov x5, x12
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::set_property_on_value
	sub x0, x29, #152
	add x5, sp, #1344
	mov x1, x21
	mov x2, x27
	mov x3, x28
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_328
	cmp w5, #2
	b.eq LBB402_252
	ldr w8, [sp, #124]
	cmp w8, #2
	b.hs LBB402_208
	tbz w5, #0, LBB402_204
LBB402_135:
	mov x22, #33
	movk x22, #32768, lsl #48
	str x22, [sp, #1392]
	b LBB402_206
LBB402_136:
	lsr w9, w5, #8
	ldr w10, [sp, #1452]
	add x11, sp, #1440
	ldur q0, [x11, #24]
	stur q0, [x29, #-208]
	ldr x24, [sp, #1456]
	ldr x11, [sp, #1480]
	b LBB402_152
LBB402_137:
	ldr w27, [x20, #4]
	ldr x8, [x21, #104]
	sub w22, w27, #1
	cmp x8, x22
	b.ls LBB402_231
	ldr x8, [x21, #96]
	add x8, x8, x22, lsl #5
	ldr x9, [x8, #16]
	sub w10, w3, #1
	cmp x9, x10
	b.ls LBB402_231
	ldr x8, [x8, #8]
	mov w9, #1216
	umaddl x23, w10, w9, x8
	ldr x8, [x23]
	cmp x8, #10
	b.eq LBB402_231
	cmp x8, #3
	b.hi LBB402_231
	ldrb w8, [x23, #1209]
	cmp w8, #2
	b.ne LBB402_231
	ldrb w8, [x23, #1208]
	cbnz w8, LBB402_231
	mov w9, #4992
	sub x8, x29, #152
	ldr x1, [sp, #136]
	add x0, x1, x9
	ldr x2, [sp, #96]
	mov x25, x6
	stp x10, x16, [sp, #40]
	mov x24, x12
	mov x26, x13
	bl lyng_objects::runtime::ObjectRuntime::object_header
	mov x12, x24
	ldr x16, [sp, #48]
	mov x6, x25
	ldr w3, [sp, #116]
	ldurb w8, [x29, #-130]
	cmp w8, #3
	b.eq LBB402_231
	str x19, [sp, #104]
	str x21, [sp, #80]
	ldur w10, [x29, #-148]
	ldur w8, [x29, #-136]
	ldurh w9, [x29, #-132]
	ldr w11, [x23, #1184]
	cbz w11, LBB402_215
	cmp w10, w11
	b.ne LBB402_215
	ldrh w11, [x23, #1188]
	cmp w9, w11
	b.ne LBB402_215
	ldr x28, [sp, #128]
	ldr w3, [sp, #116]
	mov x6, x25
	ldr x21, [sp, #80]
	ldr x16, [sp, #48]
	mov x12, x24
	ldr x19, [sp, #104]
	cbnz w8, LBB402_219
	b LBB402_231
LBB402_148:
	ldr w8, [sp, #112]
	cmp w8, #84
	b.ne LBB402_196
LBB402_149:
	mov x0, x27
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	stur x8, [x29, #-256]
	stur wzr, [x29, #-248]
	stur x0, [x29, #-240]
LBB402_150:
	sub x0, x29, #152
	sub x5, x29, #256
	b LBB402_248
LBB402_151:
	ldurb w9, [x29, #-141]
	add x11, sp, #1440
	ldurh w10, [x11, #161]
	orr w9, w10, w9, lsl #16
	ldur w10, [x29, #-140]
	ldur q0, [x11, #176]
	stur q0, [x29, #-208]
	ldur x24, [x29, #-136]
	ldur x11, [x29, #-112]
LBB402_152:
	stur x11, [x29, #-192]
LBB402_153:
	ldur q0, [x29, #-208]
	str q0, [sp, #144]
	ldur x11, [x29, #-192]
	b LBB402_330
LBB402_154:
	ldr x26, [x8, #1136]
	cbz w26, LBB402_15
	mov w9, #4992
	sub x8, x29, #152
	ldr x1, [sp, #136]
	add x0, x1, x9
	mov x2, x25
	mov x22, x6
	stp x11, x16, [sp, #40]
	str x10, [sp, #32]
	bl lyng_objects::runtime::ObjectRuntime::object_header
	mov x14, #1
	movk x14, #9, lsl #32
	movk x14, #32760, lsl #48
	ldr x16, [sp, #48]
	mov x6, x22
	ldr w3, [sp, #116]
	ldurb w8, [x29, #-130]
	cmp w8, #3
	b.eq LBB402_15
	cbz w26, LBB402_15
	ldur w8, [x29, #-148]
	cmp w8, w26
	b.ne LBB402_15
	ldurh w8, [x29, #-132]
	lsr x9, x26, #32
	cmp w8, w9, uxth
	b.ne LBB402_15
	ldur w8, [x29, #-136]
	cbz w8, LBB402_15
	ldr x9, [sp, #136]
	ldr x10, [x9, #640]
	sub w9, w8, #1
	cmp x10, x9
	b.ls LBB402_15
	ldr x10, [sp, #136]
	ldr x10, [x10, #632]
	mov w11, #24
	umaddl x9, w9, w11, x10
	ldrb w10, [x9, #19]
	cmp w10, #1
	b.ne LBB402_15
	ldr x11, [x9, #8]
	and x10, x16, #0x7fffffff
	cmp x11, x10
	b.ls LBB402_15
	ldr x9, [x9]
	ldr x9, [x9, x10, lsl #3]
	cmp x9, x14
	b.eq LBB402_15
	stp w16, w8, [x29, #-148]
	mov w8, #2
	stur w8, [x29, #-152]
	sub x1, x29, #152
	ldr x0, [sp, #136]
	mov x2, x23
	bl lyng_gc::mutator::PrimitiveMutator::store_value
	ldp x9, x16, [sp, #40]
	mov x14, #1
	movk x14, #9, lsl #32
	movk x14, #32760, lsl #48
	mov x6, x22
	ldr w3, [sp, #116]
	cbz w0, LBB402_15
	mov x23, x19
	mov x22, x21
	ldr x8, [x21, #104]
	cmp x8, x9
	b.ls LBB402_169
	ldr x8, [x22, #96]
	ldp x10, x9, [sp, #32]
	add x8, x8, x9, lsl #5
	ldr x9, [x8, #16]
	cmp x9, x10
	b.ls LBB402_169
	ldr x8, [x8, #8]
	mov w9, #1216
	ldr x10, [sp, #32]
	umaddl x0, w10, w9, x8
	ldr x8, [x0]
	cmp x8, #10
	b.eq LBB402_169
	bl lyng_vm::vm::feedback::FeedbackSiteState::record_execution
LBB402_169:
	mov x21, x22
	ldp x0, x1, [x22, #120]
	mov x2, x27
	bl lyng_vm::vm::tiering::<impl lyng_vm::vm::Vm>::observe_tier_feedback_event
	ldr w8, [sp, #124]
	cmp w8, #2
	ldp x3, x2, [sp, #128]
	b.hs LBB402_194
	mov x22, #33
	movk x22, #32768, lsl #48
	str x22, [sp, #208]
	sub x0, x29, #152
	add x5, sp, #208
	b LBB402_192
LBB402_171:
	ldrb w8, [x22, #1209]
	cmp w8, #2
	b.ne LBB402_18
	ldrb w8, [x22, #1208]
	cbnz w8, LBB402_18
	mov w9, #4992
	sub x8, x29, #152
	ldr x1, [sp, #136]
	add x0, x1, x9
	mov x2, x25
	stp x6, x11, [sp, #24]
	str x16, [sp, #48]
	str w10, [sp, #40]
	mov x26, x14
	bl lyng_objects::runtime::ObjectRuntime::object_header
	ldr x16, [sp, #48]
	ldr x6, [sp, #24]
	ldr w3, [sp, #116]
	ldurb w8, [x29, #-130]
	cmp w8, #3
	b.eq LBB402_18
	ldur w10, [x29, #-148]
	ldur w8, [x29, #-136]
	ldurh w9, [x29, #-132]
	ldr w11, [x22, #1184]
	cbz w11, LBB402_177
	cmp w10, w11
	b.ne LBB402_177
	ldrh w11, [x22, #1188]
	cmp w9, w11
	b.eq LBB402_180
LBB402_177:
	ldr w11, [x22, #1192]
	cbz w11, LBB402_18
	cmp w10, w11
	b.ne LBB402_18
	ldrh w10, [x22, #1196]
	cmp w9, w10
	b.ne LBB402_18
LBB402_180:
	cbz w8, LBB402_18
	ldr x9, [sp, #136]
	ldr x10, [x9, #640]
	sub w9, w8, #1
	cmp x10, x9
	b.ls LBB402_18
	ldr x10, [sp, #136]
	ldr x10, [x10, #632]
	mov w11, #24
	umaddl x9, w9, w11, x10
	ldrb w10, [x9, #19]
	cmp w10, #1
	b.ne LBB402_18
	ldr x11, [x9, #8]
	and x10, x16, #0x7fffffff
	cmp x11, x10
	b.ls LBB402_18
	ldr x9, [x9]
	ldr x9, [x9, x10, lsl #3]
	cmp x9, x26
	b.eq LBB402_18
	stp w16, w8, [x29, #-148]
	mov w8, #2
	stur w8, [x29, #-152]
	sub x1, x29, #152
	ldr x0, [sp, #136]
	mov x2, x23
	bl lyng_gc::mutator::PrimitiveMutator::store_value
	ldr x16, [sp, #48]
	ldr x6, [sp, #24]
	ldr w3, [sp, #116]
	cbz w0, LBB402_18
	mov x23, x19
	mov x22, x21
	ldr x8, [x21, #104]
	cmp x8, x27
	b.ls LBB402_190
	ldr x8, [x22, #96]
	add x8, x8, x27, lsl #5
	ldr x9, [x8, #16]
	ldr x10, [sp, #32]
	cmp x9, x10
	b.ls LBB402_190
	ldr x8, [x8, #8]
	mov w9, #1216
	ldr x10, [sp, #32]
	umaddl x0, w10, w9, x8
	ldr x8, [x0]
	cmp x8, #10
	b.eq LBB402_190
	bl lyng_vm::vm::feedback::FeedbackSiteState::record_execution
LBB402_190:
	mov x21, x22
	ldp x0, x1, [x22, #120]
	ldr w2, [sp, #40]
	bl lyng_vm::vm::tiering::<impl lyng_vm::vm::Vm>::observe_tier_feedback_event
	ldr w8, [sp, #124]
	cmp w8, #2
	ldp x3, x2, [sp, #128]
	b.hs LBB402_194
	mov x22, #33
	movk x22, #32768, lsl #48
	str x22, [sp, #256]
	sub x0, x29, #152
	add x5, sp, #256
LBB402_192:
	mov x1, x21
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_360
	mov x19, x23
	tbz w5, #0, LBB402_252
LBB402_194:
	ldr w8, [x20, #56]
	ldr w9, [sp, #120]
	adds w8, w8, w9
	b.hs LBB402_319
	str w8, [x20, #56]
	mov x19, x23
	b LBB402_252
LBB402_196:
	ldr x9, [x21, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB402_80
	ldr x9, [x21, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB402_80
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB402_80
	b LBB402_149
LBB402_199:
	mov w5, #0
	mov w9, #0
	mov x8, #-9223372036854775808
	b LBB402_153
LBB402_200:
Lloh1181:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
Lloh1182:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov x0, x5
	mov x2, x1
	bl core::slice::index::slice_index_fail
LBB402_201:
	mov x21, x2
	sub x0, x29, #152
	mov x2, x8
	mov w3, #1
	mov x4, x27
	bl lyng_vm::vm::dispatch::decode_abc_operands_wide
	add x11, sp, #1440
	ldur x8, [x29, #-152]
	cmp x8, x26
	b.ne LBB402_34
	ldurh w9, [x29, #-140]
	ldurh w10, [x29, #-138]
	ldurh w8, [x29, #-136]
	ldur w3, [x29, #-144]
	ldur w12, [x29, #-132]
	mov x2, x21
	ldr x21, [x20, #80]
	ldr w11, [x20, #20]
	ldr x1, [x21, #32]
	add w9, w11, w9
	cmp x1, x9
	b.hi LBB402_5
LBB402_203:
Lloh1183:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.61@PAGE
Lloh1184:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.61@PAGEOFF
	mov x0, x9
	bl core::panicking::panic_bounds_check
LBB402_204:
	ldr w8, [sp, #112]
	cmp w8, #84
	b.ne LBB402_209
LBB402_205:
	mov x0, x27
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #1392]
	str wzr, [sp, #1400]
	str x0, [sp, #1408]
	mov x22, #33
	movk x22, #32768, lsl #48
LBB402_206:
	sub x0, x29, #152
	add x5, sp, #1392
	mov x1, x21
	mov x2, x27
	mov x3, x28
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_328
	tbz w5, #0, LBB402_252
LBB402_208:
	ldr w1, [x20, #4]
	mov x0, x21
	mov x2, x23
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_keyed_generic_slow_path
	b LBB402_250
LBB402_209:
	ldr x9, [x21, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB402_135
	ldr x9, [x21, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB402_135
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB402_135
	b LBB402_205
LBB402_212:
	ldr x9, [x21, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB402_89
	ldr x9, [x21, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB402_89
	ldrh w8, [x8, #340]
	tbnz w8, #0, LBB402_32
	b LBB402_89
LBB402_215:
	ldr w11, [x23, #1192]
	ldr x28, [sp, #128]
	ldr w3, [sp, #116]
	mov x6, x25
	ldr x21, [sp, #80]
	ldr x16, [sp, #48]
	mov x12, x24
	ldr x19, [sp, #104]
	cbz w11, LBB402_231
	cmp w10, w11
	b.ne LBB402_231
	ldrh w10, [x23, #1196]
	cmp w9, w10
	b.ne LBB402_231
	cbz w8, LBB402_231
LBB402_219:
	ldr x9, [sp, #136]
	ldr x10, [x9, #640]
	sub w9, w8, #1
	cmp x10, x9
	b.ls LBB402_231
	ldr x10, [sp, #136]
	ldr x10, [x10, #632]
	mov w11, #24
	umaddl x9, w9, w11, x10
	ldrb w10, [x9, #19]
	cmp w10, #1
	b.ne LBB402_231
	ldr x11, [x9, #8]
	mov w10, w16
	cmp x11, x10
	b.ls LBB402_231
	ldr x9, [x9]
	ldr x9, [x9, x10, lsl #3]
	cmp x9, x26
	b.eq LBB402_231
	stp w16, w8, [x29, #-148]
	mov w8, #2
	stur w8, [x29, #-152]
	sub x1, x29, #152
	ldr x0, [sp, #136]
	ldr x2, [sp, #88]
	bl lyng_gc::mutator::PrimitiveMutator::store_value
	mov x12, x24
	ldr x16, [sp, #48]
	mov x6, x25
	ldr w3, [sp, #116]
	cbz w0, LBB402_231
	ldr x8, [x21, #104]
	cmp x8, x22
	b.ls LBB402_228
	ldr x8, [sp, #80]
	ldr x8, [x8, #96]
	add x8, x8, x22, lsl #5
	ldr x9, [x8, #16]
	ldr x10, [sp, #40]
	cmp x9, x10
	b.ls LBB402_228
	ldr x8, [x8, #8]
	mov w9, #1216
	ldr x10, [sp, #40]
	umaddl x0, w10, w9, x8
	ldr x8, [x0]
	cmp x8, #10
	b.eq LBB402_228
	bl lyng_vm::vm::feedback::FeedbackSiteState::record_execution
LBB402_228:
	ldr x21, [sp, #80]
	ldp x0, x1, [x21, #120]
	mov x2, x27
	bl lyng_vm::vm::tiering::<impl lyng_vm::vm::Vm>::observe_tier_feedback_event
	ldr w8, [sp, #124]
	cmp w8, #2
	ldp x3, x2, [sp, #128]
	b.hs LBB402_358
	mov x22, #33
	movk x22, #32768, lsl #48
	str x22, [sp, #720]
	sub x0, x29, #152
	add x5, sp, #720
	mov x1, x21
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.eq LBB402_357
LBB402_230:
	ldurb w9, [x29, #-141]
	add x11, sp, #1440
	ldurh w10, [x11, #161]
	orr w9, w10, w9, lsl #16
	ldur w10, [x29, #-140]
	ldur q0, [x11, #176]
	str q0, [sp, #144]
	ldur x24, [x29, #-136]
	ldur x11, [x29, #-112]
	str x11, [sp, #160]
	ldr x19, [sp, #104]
	b LBB402_331
LBB402_231:
	str x6, [sp, #24]
	mov x24, x12
	ldr w2, [x20, #4]
	mov x0, x21
	ldr x27, [sp, #136]
	mov x1, x27
	ldp x23, x25, [sp, #88]
	mov x4, x25
	mov x5, x16
	mov x6, x23
	mov x26, x16
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_keyed_dense_index_store_inline_cache_hit
	and w8, w0, #0xff
	cmp w8, #2
	b.ne LBB402_246
	add x0, sp, #816
	mov x1, x21
	mov x2, x27
	mov x3, x25
	mov x4, x26
	mov x5, x23
	bl lyng_vm::vm::values::<impl lyng_vm::vm::Vm>::mapped_arguments_set
	ldr x8, [sp, #816]
	mov x22, #33
	movk x22, #32768, lsl #48
	add x9, x22, #1
	cmp x8, x9
	str x26, [sp, #48]
	b.ne LBB402_253
	str x23, [sp, #8]
	add x0, sp, #864
	stp w25, w26, [sp]
	mov x1, x21
	mov x2, x27
	ldr x3, [sp, #56]
	ldr x4, [sp, #72]
	mov x5, x24
	ldr x6, [sp, #24]
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_set_typed_array_index
	sub x0, x29, #152
	add x5, sp, #864
	mov x1, x21
	mov x2, x27
	mov x3, x28
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_328
	cmp w5, #3
	b.eq LBB402_252
	cmp w5, #2
	b.ne LBB402_257
	add x0, sp, #912
	mov x1, x27
	ldp x4, x2, [sp, #88]
	ldr x3, [sp, #48]
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_set_engine_array_index
	sub x0, x29, #152
	add x5, sp, #912
	mov x1, x21
	mov x2, x27
	mov x3, x28
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_328
	cmp w5, #3
	b.eq LBB402_252
	cmp w5, #2
	b.ne LBB402_257
	mov x25, x19
	add x0, sp, #960
	mov x1, x27
	ldp x4, x2, [sp, #88]
	ldr x3, [sp, #48]
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_set_ordinary_index_data_property
	sub x0, x29, #152
	add x5, sp, #960
	mov x22, x21
	mov x1, x21
	mov x2, x27
	mov x3, x28
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_273
	cmp w5, #3
	ldp x28, x27, [sp, #128]
	mov x19, x25
	b.eq LBB402_252
	mov x21, x22
	cmp w5, #2
	b.ne LBB402_257
	ldr x8, [sp, #88]
	str x8, [sp, #16]
	add x0, sp, #1008
	ldp x8, x3, [sp, #48]
	stp wzr, w8, [sp, #8]
	ldr x8, [sp, #96]
	str x8, [sp]
	mov x1, x21
	mov x2, x27
	ldr x4, [sp, #72]
	mov x5, x24
	ldr x6, [sp, #24]
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::set_property_on_value
	sub x0, x29, #152
	add x5, sp, #1008
	mov x1, x21
	mov x2, x27
	mov x3, x28
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_273
	cmp w5, #2
	ldp x28, x27, [sp, #128]
	mov x19, x25
	b.eq LBB402_252
	mov x21, x22
	ldr w8, [sp, #124]
	cmp w8, #2
	ldr x23, [sp, #96]
	b.hs LBB402_270
	mov w22, #0
	b LBB402_259
LBB402_246:
	ldr w8, [sp, #124]
	cmp w8, #2
	b.hs LBB402_250
	mov x22, #33
	movk x22, #32768, lsl #48
	str x22, [sp, #768]
	sub x0, x29, #152
	add x5, sp, #768
LBB402_248:
	mov x1, x21
	mov x2, x27
	mov x3, x28
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_328
	tbz w5, #0, LBB402_252
LBB402_250:
	ldr w8, [x20, #56]
	ldr w9, [sp, #120]
	adds w8, w8, w9
	b.hs LBB402_319
LBB402_251:
	str w8, [x20, #56]
LBB402_252:
	ldr x8, [x20, #128]
	ldr x8, [x8, #48]
	ldr w9, [x20, #56]
	ldrb w8, [x8, x9]
Lloh1185:
	adrp x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
Lloh1186:
	add x9, x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x10, [x9, x8, lsl #3]
	mov x8, #33
	movk x8, #32768, lsl #48
	stp x8, x10, [x19]
	b LBB402_332
LBB402_253:
	sub x0, x29, #152
	add x5, sp, #816
	mov x1, x21
	mov x2, x27
	mov x3, x28
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_328
	tbz w5, #0, LBB402_252
	ldr w8, [sp, #124]
	cmp w8, #2
	ldr x23, [sp, #96]
	b.hs LBB402_270
	mov w22, #0
	b LBB402_266
LBB402_257:
	ldr w8, [sp, #124]
	cmp w8, #2
	ldr x23, [sp, #96]
	b.hs LBB402_271
	mov w22, #1
LBB402_259:
	tbnz w5, #0, LBB402_266
	ldr w8, [sp, #112]
	cmp w8, #84
	b.ne LBB402_262
LBB402_261:
	mov x0, x27
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #1056]
	str wzr, [sp, #1064]
	str x0, [sp, #1072]
	mov x23, #33
	movk x23, #32768, lsl #48
	b LBB402_267
LBB402_262:
	mov x10, x21
	ldr x9, [x21, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB402_265
	ldr x9, [x10, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB402_265
	ldrh w8, [x8, #340]
	ldp x28, x27, [sp, #128]
	mov x21, x10
	tbnz w8, #0, LBB402_261
	b LBB402_266
LBB402_265:
	ldp x28, x27, [sp, #128]
	mov x21, x10
LBB402_266:
	mov x23, #33
	movk x23, #32768, lsl #48
	str x23, [sp, #1056]
LBB402_267:
	sub x0, x29, #152
	add x5, sp, #1056
	mov x1, x21
	mov x2, x27
	mov x3, x28
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x23
	b.ne LBB402_328
	tbz w5, #0, LBB402_252
	ldr x23, [sp, #96]
	tbnz w22, #0, LBB402_271
LBB402_270:
	sub x0, x29, #152
	mov x1, x27
	mov x2, x23
	bl lyng_vm::vm::activation_objects::<impl lyng_vm::vm::Vm>::sync_engine_array_length
	ldur x8, [x29, #-152]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_272
LBB402_271:
	ldr w2, [x20, #4]
	mov x0, x21
	mov x1, x27
	ldr w3, [sp, #116]
	mov x4, x23
	ldr x5, [sp, #48]
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_keyed_index_access
	b LBB402_250
LBB402_272:
	ldp w5, w10, [x29, #-144]
	lsr w9, w5, #8
	add x11, sp, #1440
	b LBB402_329
LBB402_273:
	ldurb w9, [x29, #-141]
	add x11, sp, #1440
	ldurh w10, [x11, #161]
	orr w9, w10, w9, lsl #16
	ldur w10, [x29, #-140]
	ldur q0, [x11, #176]
	str q0, [sp, #144]
	ldur x24, [x29, #-136]
	ldur x11, [x29, #-112]
	str x11, [sp, #160]
	mov x19, x25
	b LBB402_331
LBB402_274:
	mov x3, x27
	ldr x28, [sp, #128]
	mov x14, x23
	mov x21, x24
	mov x25, x15
	mov x16, x17
	ldr x19, [sp, #104]
	mov x26, x0
	cbnz w13, LBB402_281
	cmp x11, x10
	b.ne LBB402_281
LBB402_276:
	tbnz w8, #30, LBB402_278
	mov x21, x24
	mov x0, x24
	mov x1, x27
	mov x2, x23
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
	ldr w8, [sp, #124]
	cmp w8, #2
	ldr x22, [sp, #136]
	b.lo LBB402_349
	b LBB402_358
LBB402_278:
	tbnz w8, #31, LBB402_346
	mov x3, x27
	ldr x28, [sp, #128]
	mov x14, x23
	mov x21, x24
	mov x25, x15
	mov x16, x17
	ldr x19, [sp, #104]
	mov x26, x0
	cbz w9, LBB402_281
	mov w10, #2
	mov x15, x9
	b LBB402_347
LBB402_281:
	mov x22, x26
	ldr x23, [sp, #136]
	ldr x10, [x23, #224]
	mov w8, #-1
	add x8, x25, x8
	lsr w9, w8, #6
	cmp x9, x10
	b.hs LBB402_305
	ldr x10, [x23, #216]
	and x8, x8, #0x3f
	ldr x9, [x10, x9, lsl #3]
	mov w10, #80
	umaddl x8, w8, w10, x9
	ldr w9, [x8]
	cmp w9, #1
	b.ne LBB402_305
	cbz w14, LBB402_305
	ldr w10, [x8, #52]
	cbz w10, LBB402_305
	ldr x11, [x21, #104]
	sub w9, w3, #1
	cmp x11, x9
	b.ls LBB402_305
	ldr x11, [x21, #96]
	add x11, x11, x9, lsl #5
	ldr x12, [x11, #16]
	sub w9, w14, #1
	cmp x12, x9
	b.ls LBB402_305
	ldr x11, [x11, #8]
	mov w12, #1216
	umaddl x11, w9, w12, x11
	ldr x9, [x11]
	cmp x9, #10
	b.eq LBB402_305
	cmp x9, #3
	b.hi LBB402_305
	str x19, [sp, #104]
	mov x15, x25
	str x21, [sp, #80]
	mov x24, x14
	mov x27, x3
	ldr x9, [x11, #1144]
	cbz x9, LBB402_294
	lsr x12, x9, #32
	cbz x12, LBB402_294
	ldr w13, [x11, #1160]
	cmp w13, w16
	b.ne LBB402_294
	cmp w10, w12
	b.ne LBB402_294
	mov x10, #0
	b LBB402_299
LBB402_294:
	ldr x9, [x11, #1152]
	mov x3, x27
	ldp x28, x23, [sp, #128]
	mov x14, x24
	ldr x21, [sp, #80]
	mov x25, x15
	ldr x19, [sp, #104]
	cbz x9, LBB402_305
	lsr x12, x9, #32
	cbz x12, LBB402_305
	ldr w13, [x11, #1164]
	cmp w13, w16
	b.ne LBB402_305
	cmp w10, w12
	b.ne LBB402_305
	mov w10, #1
LBB402_299:
	add x10, x11, x10, lsl #3
	ldr x10, [x10, #1168]
	ldr x11, [x8, #40]
	cmp x11, x10
	mov x3, x27
	ldp x28, x23, [sp, #128]
	mov x14, x24
	ldr x21, [sp, #80]
	mov x25, x15
	ldr x19, [sp, #104]
	b.ne LBB402_305
	tbnz w9, #30, LBB402_302
	ldr x21, [sp, #80]
	mov x0, x21
	mov x1, x27
	mov x2, x24
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
	ldr w8, [sp, #124]
	cmp w8, #2
	ldr x22, [sp, #136]
	b.lo LBB402_339
	b LBB402_358
LBB402_302:
	tbnz w9, #31, LBB402_336
	ldr w10, [x8, #56]
	mov x3, x27
	ldp x28, x23, [sp, #128]
	mov x14, x24
	ldr x21, [sp, #80]
	mov x25, x15
	ldr x19, [sp, #104]
	cbz w10, LBB402_305
	mov w8, #2
	mov x15, x10
	b LBB402_337
LBB402_305:
	ldp x0, x1, [x21, #96]
	mov x2, x23
	mov x4, x14
	mov x24, x14
	mov x5, x25
	mov x6, x16
	mov x26, x7
	mov x27, x16
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_keyed_property_store_inline_cache
	and w8, w0, #0xff
	cmp w8, #2
	b.ne LBB402_311
	str x26, [sp, #16]
	mov w8, #1
	str x27, [sp, #48]
	stp w8, w27, [sp, #8]
	str x25, [sp]
	add x0, sp, #1248
	mov x1, x21
	mov x2, x23
	mov x3, x22
	ldp x5, x4, [sp, #64]
	ldr x6, [sp, #24]
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::set_property_on_value
	sub x0, x29, #152
	add x5, sp, #1248
	mov x1, x21
	mov x2, x23
	mov x3, x28
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_328
	cmp w5, #2
	b.eq LBB402_252
	ldr w8, [sp, #124]
	cmp w8, #2
	b.hs LBB402_327
	tbz w5, #0, LBB402_323
LBB402_310:
	mov x22, #33
	movk x22, #32768, lsl #48
	str x22, [sp, #1296]
	b LBB402_325
LBB402_311:
	ldr w9, [sp, #124]
	cmp w9, #2
	b.hs LBB402_318
	tbz w8, #0, LBB402_314
LBB402_313:
	mov x22, #33
	movk x22, #32768, lsl #48
	str x22, [sp, #1200]
	b LBB402_316
LBB402_314:
	ldr w8, [sp, #112]
	cmp w8, #84
	b.ne LBB402_320
LBB402_315:
	mov x0, x23
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #1200]
	str wzr, [sp, #1208]
	str x0, [sp, #1216]
	mov x22, #33
	movk x22, #32768, lsl #48
LBB402_316:
	sub x0, x29, #152
	add x5, sp, #1200
	mov x1, x21
	mov x2, x23
	mov x3, x28
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_328
	tbz w5, #0, LBB402_252
LBB402_318:
	ldr w1, [x20, #4]
	mov x0, x21
	mov x2, x24
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
	ldr w8, [x20, #56]
	ldr w9, [sp, #120]
	adds w8, w8, w9
	b.lo LBB402_251
LBB402_319:
Lloh1187:
	adrp x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGE
Lloh1188:
	add x0, x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGEOFF
Lloh1189:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGE
Lloh1190:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGEOFF
	mov w1, #41
	bl core::option::expect_failed
LBB402_320:
	ldr x9, [x21, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB402_313
	ldr x9, [x21, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB402_313
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB402_313
	b LBB402_315
LBB402_323:
	ldr w8, [sp, #112]
	cmp w8, #84
	b.ne LBB402_333
LBB402_324:
	mov x0, x23
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #1296]
	str wzr, [sp, #1304]
	str x0, [sp, #1312]
	mov x22, #33
	movk x22, #32768, lsl #48
LBB402_325:
	sub x0, x29, #152
	add x5, sp, #1296
	mov x1, x21
	mov x2, x23
	mov x3, x28
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_328
	tbz w5, #0, LBB402_252
LBB402_327:
	ldr w2, [x20, #4]
	mov x0, x21
	mov x1, x23
	mov x3, x24
	mov x4, x25
	ldr x5, [sp, #48]
	mov w6, #1
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_keyed_atom_slow_path
	b LBB402_250
LBB402_328:
	ldurb w9, [x29, #-141]
	add x11, sp, #1440
	ldurh w10, [x11, #161]
	orr w9, w10, w9, lsl #16
	ldur w10, [x29, #-140]
LBB402_329:
	ldur q0, [x11, #176]
	str q0, [sp, #144]
	ldur x24, [x29, #-136]
	ldur x11, [x29, #-112]
LBB402_330:
	str x11, [sp, #160]
LBB402_331:
	bfi w5, w9, #8, #24
	str x8, [x19]
	stp w5, w10, [x19, #8]
	str x24, [x19, #16]
	ldr q0, [sp, #144]
	stur q0, [x19, #24]
	ldr x8, [sp, #160]
	str x8, [x19, #40]
LBB402_332:
	add sp, sp, #1664
	.cfi_def_cfa wsp, 96
	ldp x29, x30, [sp, #80]
	ldp x20, x19, [sp, #64]
	ldp x22, x21, [sp, #48]
	ldp x24, x23, [sp, #32]
	ldp x26, x25, [sp, #16]
	ldp x28, x27, [sp], #96
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore w23
	.cfi_restore w24
	.cfi_restore w25
	.cfi_restore w26
	.cfi_restore w27
	.cfi_restore w28
	ret
LBB402_333:
	.cfi_restore_state
	ldr x9, [x21, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB402_310
	ldr x9, [x21, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB402_310
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB402_310
	b LBB402_324
LBB402_336:
	mov w8, #5
LBB402_337:
	and w9, w9, #0x3fffffff
	stp w8, w9, [x29, #-152]
	stur w15, [x29, #-144]
	sub x1, x29, #152
	ldr x22, [sp, #136]
	mov x0, x22
	mov x2, x7
	bl lyng_gc::mutator::PrimitiveMutator::store_value
	mov x25, x0
	ldr x21, [sp, #80]
	mov x0, x21
	mov x1, x27
	mov x2, x24
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
	ldr w8, [sp, #124]
	cmp w8, #2
	b.hs LBB402_358
	tbnz w25, #0, LBB402_344
LBB402_339:
	ldr w8, [sp, #112]
	cmp w8, #84
	b.ne LBB402_341
LBB402_340:
	mov x0, x22
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #1152]
	str wzr, [sp, #1160]
	str x0, [sp, #1168]
	b LBB402_345
LBB402_341:
	ldr x9, [x21, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB402_344
	ldr x9, [x21, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB402_344
	ldrh w8, [x8, #340]
	tbnz w8, #0, LBB402_340
LBB402_344:
	mov x8, #33
	movk x8, #32768, lsl #48
	str x8, [sp, #1152]
LBB402_345:
	sub x0, x29, #152
	add x5, sp, #1152
	ldr x1, [sp, #80]
	b LBB402_356
LBB402_346:
	mov w10, #5
LBB402_347:
	and w8, w8, #0x3fffffff
	stp w10, w8, [x29, #-152]
	stur w15, [x29, #-144]
	sub x1, x29, #152
	ldr x22, [sp, #136]
	mov x0, x22
	mov x2, x7
	bl lyng_gc::mutator::PrimitiveMutator::store_value
	mov x25, x0
	mov x21, x24
	mov x0, x24
	mov x1, x27
	mov x2, x23
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
	ldr w8, [sp, #124]
	cmp w8, #2
	b.hs LBB402_358
	tbnz w25, #0, LBB402_354
LBB402_349:
	ldr w8, [sp, #112]
	cmp w8, #84
	b.ne LBB402_351
LBB402_350:
	mov x0, x22
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #1104]
	str wzr, [sp, #1112]
	str x0, [sp, #1120]
	b LBB402_355
LBB402_351:
	ldr x9, [x21, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB402_354
	ldr x9, [x21, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB402_354
	ldrh w8, [x8, #340]
	tbnz w8, #0, LBB402_350
LBB402_354:
	mov x8, #33
	movk x8, #32768, lsl #48
	str x8, [sp, #1104]
LBB402_355:
	sub x0, x29, #152
	add x5, sp, #1104
	mov x1, x24
LBB402_356:
	ldp x3, x2, [sp, #128]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_230
LBB402_357:
	ldr x19, [sp, #104]
	tbz w5, #0, LBB402_252
LBB402_358:
	ldr w8, [x20, #56]
	ldr w9, [sp, #120]
	adds w8, w8, w9
	b.hs LBB402_319
	str w8, [x20, #56]
	ldr x19, [sp, #104]
	b LBB402_252
LBB402_360:
	ldurb w9, [x29, #-141]
	add x11, sp, #1440
	ldurh w10, [x11, #161]
	orr w9, w10, w9, lsl #16
	ldur w10, [x29, #-140]
	ldur q0, [x11, #176]
	str q0, [sp, #144]
	ldur x24, [x29, #-136]
	ldur x11, [x29, #-112]
	str x11, [sp, #160]
	mov x19, x23
	b LBB402_331
LBB402_361:
Lloh1191:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.61@PAGE
Lloh1192:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.61@PAGEOFF
	bl core::panicking::panic_bounds_check
LBB402_362:
Lloh1193:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.61@PAGE
Lloh1194:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.61@PAGEOFF
	mov x0, x8
	bl core::panicking::panic_bounds_check
	.loh AdrpAdd	Lloh1181, Lloh1182
	.loh AdrpAdd	Lloh1183, Lloh1184
	.loh AdrpAdd	Lloh1185, Lloh1186
	.loh AdrpAdd	Lloh1189, Lloh1190
	.loh AdrpAdd	Lloh1187, Lloh1188
	.loh AdrpAdd	Lloh1191, Lloh1192
	.loh AdrpAdd	Lloh1193, Lloh1194
Lfunc_end402:
	.cfi_endproc

	.p2align	2
lyng_vm::vm::dispatch_handlers::property::op_set_named_property_common:
Lfunc_begin403:
	.cfi_startproc
	sub sp, sp, #496
	.cfi_def_cfa_offset 496
	stp x28, x27, [sp, #400]
	stp x26, x25, [sp, #416]
	stp x24, x23, [sp, #432]
	stp x22, x21, [sp, #448]
	stp x20, x19, [sp, #464]
	stp x29, x30, [sp, #480]
	add x29, sp, #480
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset w21, -40
	.cfi_offset w22, -48
	.cfi_offset w23, -56
	.cfi_offset w24, -64
	.cfi_offset w25, -72
	.cfi_offset w26, -80
	.cfi_offset w27, -88
	.cfi_offset w28, -96
	.cfi_remember_state
	ldr w24, [x1, #4]
	ldr w5, [x1, #56]
	ldrb w10, [x1, #148]
	mov w8, #152
	strb w8, [x1, #148]
	ldr x11, [x1, #128]
	ldr x8, [x11, #56]
	subs x9, x8, x5
	b.lo LBB403_71
	sub x28, x29, #224
	mov x27, #33
	movk x27, #32768, lsl #48
	ldr x8, [x11, #48]
	add x8, x8, x5
	cmp w10, #152
	b.ne LBB403_72
	and x10, x9, #0x7ffffffffffffffe
	cmp x9, #4
	ccmp x10, #4, #4, hs
	b.eq LBB403_17
	ldrh w22, [x8, #4]
	cbz w22, LBB403_17
	ldrb w9, [x8, #1]
	ldrb w11, [x8, #2]
	mov w20, #6
	ldrb w12, [x8, #3]
	ldr x19, [x1, #80]
	ldr w8, [x1, #20]
	ldr x10, [x19, #32]
	add w9, w8, w9
	cmp x10, x9
	b.ls LBB403_74
LBB403_5:
	add w8, w8, w11
	cmp x10, x8
	b.ls LBB403_170
	ldr x10, [x19, #80]
	sub w13, w24, #1
	cmp x10, x13
	b.ls LBB403_19
	ldr x11, [x19, #72]
	ldr x14, [x11, x13, lsl #3]
	cbz x14, LBB403_19
	ldr x16, [x14, #80]
	mov w15, w12
	sub x10, x27, #3
	cmp x16, x15
	b.ls LBB403_20
	ldr x16, [x14, #72]
	add x16, x16, x15, lsl #4
	ldr w15, [x16]
	cmp w15, #4
	b.eq LBB403_20
	ldr w26, [x16, #4]
	cmp w15, #2
	b.ne LBB403_26
	ldr x17, [x1, #88]
	ldr x23, [x1, #136]
	ldr x10, [x19, #24]
	ldr x5, [x10, x9, lsl #3]
	ldr x9, [x14, #464]
	cmp x9, x26
	b.ls LBB403_14
	ldr x9, [x14, #456]
	add x9, x9, x26, lsl #3
	ldr w12, [x9, #16]!
	cmp w12, #1
	b.ne LBB403_14
	ldr w26, [x9, #4]
LBB403_14:
	ldp x25, x3, [x1, #96]
	ldp x16, x15, [x1, #112]
	ldr x7, [x10, x8, lsl #3]
	and x8, x5, #0x7ff8000000000000
	lsr x12, x5, #32
	sub w10, w12, #1
	mov x9, #9221120237041090560
	cmp x8, x9
	and w10, w10, #0xffff
	ccmp w10, #9, #2, eq
	stp x17, x1, [sp, #96]
	b.ls LBB403_27
LBB403_15:
	cmp x8, x9
	ccmp w10, #2, #2, eq
	b.hs LBB403_28
	mov x20, x0
	mov x0, x17
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, x0
	mov x0, x20
	mov w9, #0
	stur x8, [x29, #-208]
	mov x10, #-9223372036854775808
	b LBB403_37
LBB403_17:
	sub x8, x27, #12
	stur x8, [x29, #-152]
	stp w24, w5, [x29, #-144]
LBB403_18:
	ldur q0, [x28, #72]
	ldur q1, [x28, #88]
	stp q0, q1, [x0]
	ldur q0, [x28, #104]
	b LBB403_24
LBB403_19:
	sub x10, x27, #29
	mov x9, x24
	b LBB403_21
LBB403_20:
	mov x9, #0
LBB403_21:
LBB403_22:
	lsr x11, x9, #32
	lsr w8, w9, #8
LBB403_23:
	bfi w9, w8, #8, #24
	str x10, [x0]
	stp w9, w11, [x0, #8]
	str x22, [x0, #16]
	stp w24, w12, [x0, #24]
	ldur q0, [x29, #-176]
LBB403_24:
	str q0, [x0, #32]
LBB403_25:
	.cfi_def_cfa wsp, 496
	ldp x29, x30, [sp, #480]
	ldp x20, x19, [sp, #464]
	ldp x22, x21, [sp, #448]
	ldp x24, x23, [sp, #432]
	ldp x26, x25, [sp, #416]
	ldp x28, x27, [sp, #400]
	add sp, sp, #496
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore w23
	.cfi_restore w24
	.cfi_restore w25
	.cfi_restore w26
	.cfi_restore w27
	.cfi_restore w28
	ret
LBB403_26:
	.cfi_restore_state
	ldr x22, [x16, #8]
	orr x9, x15, x26, lsl #32
	b LBB403_22
LBB403_27:
	and w12, w12, #0xffff
	mov w14, #1
	lsl w12, w14, w12
	mov w14, #2008
	tst w12, w14
	b.eq LBB403_50
LBB403_28:
	mov x22, x3
	mov x24, x16
	stp x26, x7, [sp, #64]
	mov x26, x15
	stp x23, x0, [sp, #80]
	str w2, [sp, #60]
	ldr w1, [x1, #8]
	sub x8, x29, #152
	mov x0, x17
	mov x2, x5
	mov x21, x17
	mov x23, x5
	bl lyng_ops::object::primitive_wrappers::to_object
	ldur w9, [x29, #-152]
	cmp w9, #4
	b.ne LBB403_34
	ldur w2, [x29, #-148]
	stp x19, x21, [x29, #-152]
	stp x25, x22, [x29, #-136]
	stp x24, x26, [x29, #-120]
	ldr x8, [sp, #104]
	stur x8, [x29, #-104]
	sub x0, x29, #224
	sub x1, x29, #152
	mov w3, #1
	ldp x4, x5, [sp, #64]
	mov x6, x23
	bl lyng_ops::proxy::set
	ldur x10, [x29, #-224]
	cmp x10, x27
	b.ne LBB403_35
	ldr w22, [sp, #60]
	and w8, w22, #0xff
	cmp w8, #79
	ldr x0, [sp, #88]
	b.lo LBB403_32
	ldurb w8, [x29, #-216]
	tbz w8, #0, LBB403_62
LBB403_32:
	ldr x8, [sp, #104]
	ldr w9, [x8, #56]
	adds w9, w9, w20
	b.hs LBB403_169
	str w9, [x8, #56]
	b LBB403_123
LBB403_34:
	sub x8, x29, #152
	orr x8, x8, #0x4
	ldr x10, [x8]
	stur x10, [x28, #12]
	ldr w8, [x8, #8]
	stur w8, [x29, #-204]
	mov x10, #-9223372036854775808
	b LBB403_36
LBB403_35:
	ldur w9, [x29, #-216]
LBB403_36:
	ldp x23, x0, [sp, #80]
LBB403_37:
	mov x8, #-9223372036854775808
	cmp x10, x8
	b.ne LBB403_48
	cbnz w9, LBB403_48
	mov x20, x0
	ldur x22, [x29, #-208]
	cbz x23, LBB403_42
	sub x8, x23, #1
	ldr x9, [x19, #56]
	cmp x8, x9
	b.hs LBB403_42
	ldr x9, [x19, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr x9, [sp, #104]
	ldr q0, [x9]
	str q0, [x8]
	ldp q0, q1, [x9, #48]
	ldp q3, q2, [x9, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB403_42:
	sub x0, x29, #152
	mov x1, x19
	ldr x2, [sp, #96]
	mov x3, x22
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
	ldur x10, [x29, #-152]
	ldurb w9, [x29, #-144]
	cmp x10, x27
	b.ne LBB403_49
	mov x0, x20
	tbz w9, #0, LBB403_70
	ldr w8, [x19, #1640]
	add w8, w8, #1
	str w8, [x19, #1640]
	ldr x8, [sp, #104]
	cbz x23, LBB403_123
	sub x9, x23, #1
LBB403_46:
	ldr x10, [x19, #56]
	cmp x9, x10
	b.hs LBB403_123
	ldr x10, [x19, #48]
	mov w11, #80
	madd x9, x9, x11, x10
	ldr q0, [x9]
	str q0, [x8]
	ldp q0, q1, [x9, #48]
	ldp q3, q2, [x9, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
	b LBB403_123
LBB403_48:
	lsr w8, w9, #8
	ldur w11, [x29, #-212]
	ldur x22, [x29, #-208]
	ldp w24, w12, [x29, #-200]
	ldur q0, [x29, #-192]
	stur q0, [x29, #-176]
	b LBB403_23
LBB403_49:
	ldurb w8, [x29, #-141]
	ldurh w11, [x28, #81]
	orr w8, w11, w8, lsl #16
	ldur w11, [x29, #-140]
	ldur x22, [x29, #-136]
	ldp w24, w12, [x29, #-128]
	ldur q0, [x28, #104]
	stur q0, [x29, #-176]
	mov x0, x20
	b LBB403_23
LBB403_50:
	tst w12, #0x6
	b.ne LBB403_15
	cbz w5, LBB403_28
	stp x25, x3, [sp, #40]
	stp x16, x15, [sp, #64]
	stp x23, x0, [sp, #80]
	ldr x8, [x19, #104]
	cbz w22, LBB403_85
	cmp x8, x13
	b.ls LBB403_85
	ldr x9, [x19, #96]
	add x10, x9, x13, lsl #5
	ldr x12, [x10, #16]
	sub w9, w22, #1
	cmp x12, x9
	b.ls LBB403_85
	ldr x10, [x10, #8]
	mov w12, #1216
	umaddl x10, w9, w12, x10
	ldr x9, [x10]
	cmp x9, #10
	b.eq LBB403_85
	cmp x9, #6
	b.ne LBB403_85
	ldr x9, [x10, #968]
	cbz x9, LBB403_85
	ldr x15, [x17, #224]
	mov w12, #-1
	add x12, x5, x12
	lsr w14, w12, #6
	cmp x14, x15
	b.hs LBB403_85
	ldr x15, [x17, #216]
	and x12, x12, #0x3f
	ldr x14, [x15, x14, lsl #3]
	mov w15, #80
	umaddl x14, w12, w15, x14
	ldr w12, [x14]
	cmp w12, #1
	b.ne LBB403_85
	mov x6, x26
	mov x4, x5
	mov x3, x7
	mov x21, x17
	mov x23, x2
	ldr x12, [x10, #976]
	ldp w15, w10, [x14, #52]
	ldr x14, [x14, #40]
	lsr x16, x9, #32
	cmp x16, #0
	csel w16, w16, wzr, ne
	cbz w15, LBB403_78
	cmp w15, w16
	ccmp x14, x12, #0, eq
	mov x2, x23
	mov x17, x21
	mov x7, x3
	mov x5, x4
	mov x26, x6
	b.eq LBB403_80
	b LBB403_85
LBB403_62:
	and w8, w22, #0xff
	cmp w8, #80
	b.ne LBB403_75
LBB403_63:
	ldr x0, [sp, #96]
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x22, x0
	ldr x8, [sp, #80]
	cbz x8, LBB403_66
	ldr x8, [sp, #80]
	sub x8, x8, #1
	ldr x9, [x19, #56]
	cmp x8, x9
	b.hs LBB403_66
	ldr x9, [x19, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr x9, [sp, #104]
	ldr q0, [x9]
	str q0, [x8]
	ldp q0, q1, [x9, #48]
	ldp q3, q2, [x9, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB403_66:
	sub x0, x29, #152
	mov x1, x19
	ldr x2, [sp, #96]
	mov x3, x22
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
	ldur x10, [x29, #-152]
	ldurb w9, [x29, #-144]
	cmp x10, x27
	b.ne LBB403_162
	ldr x0, [sp, #88]
	tbz w9, #0, LBB403_70
	ldr w8, [x19, #1640]
	add w8, w8, #1
	str w8, [x19, #1640]
	ldr x8, [sp, #104]
	ldr x21, [sp, #80]
	cbz x21, LBB403_123
	sub x9, x21, #1
	b LBB403_46
LBB403_70:
	mov w8, #0
	mov w9, #0
	mov x10, #-9223372036854775808
	b LBB403_23
LBB403_71:
Lloh1195:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
Lloh1196:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov x0, x5
	mov x1, x8
	mov x2, x8
	bl core::slice::index::slice_index_fail
LBB403_72:
	mov x19, x2
	mov x21, x1
	mov x20, x0
	sub x0, x29, #152
	mov x1, x8
	mov x2, x9
	mov w3, #1
	mov x4, x24
	bl lyng_vm::vm::dispatch::decode_abc_operands_wide
	mov x0, x20
	ldur x8, [x29, #-152]
	cmp x8, x27
	b.ne LBB403_18
	ldurh w9, [x29, #-140]
	ldurh w11, [x29, #-138]
	ldurh w12, [x29, #-136]
	ldur w22, [x29, #-144]
	ldur w20, [x29, #-132]
	mov x1, x21
	mov x2, x19
	ldr x19, [x21, #80]
	ldr w8, [x21, #20]
	ldr x10, [x19, #32]
	add w9, w8, w9
	cmp x10, x9
	b.hi LBB403_5
LBB403_74:
Lloh1197:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.765@PAGE
Lloh1198:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.765@PAGEOFF
	mov x0, x9
	mov x1, x10
	bl core::panicking::panic_bounds_check
LBB403_75:
	ldr x9, [x19, #80]
	ldr x8, [sp, #104]
	ldr w8, [x8, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB403_32
	ldr x9, [x19, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB403_32
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB403_32
	b LBB403_63
LBB403_78:
	mov x2, x23
	mov x17, x21
	mov x7, x3
	mov x5, x4
	mov x26, x6
	cbnz w16, LBB403_85
	cmp x14, x12
	b.ne LBB403_85
LBB403_80:
	tbnz w9, #30, LBB403_82
	and w8, w23, #0xff
	cmp w8, #78
	b.hi LBB403_155
	b LBB403_167
LBB403_82:
	tbnz w9, #31, LBB403_151
	mov x2, x23
	mov x17, x21
	mov x7, x3
	mov x5, x4
	mov x26, x6
	cbz w10, LBB403_85
	mov w8, #2
	mov x4, x10
	b LBB403_152
LBB403_85:
	str w2, [sp, #60]
	ldr x12, [x17, #224]
	mov w9, #-1
	add x9, x5, x9
	lsr w10, w9, #6
	cmp x10, x12
	b.hs LBB403_104
	ldr x12, [x17, #216]
	and x9, x9, #0x3f
	ldr x10, [x12, x10, lsl #3]
	mov w12, #80
	umaddl x9, w9, w12, x10
	ldr w10, [x9]
	cmp w10, #1
	b.ne LBB403_104
	cbz w22, LBB403_104
	ldr w12, [x9, #52]
	cmp w12, #0
	ccmp x8, x13, #0, ne
	b.ls LBB403_104
	ldr x10, [x19, #96]
	add x13, x10, x13, lsl #5
	ldr x14, [x13, #16]
	sub w10, w22, #1
	cmp x14, x10
	b.ls LBB403_104
	ldr x13, [x13, #8]
	mov w14, #1216
	umaddl x13, w10, w14, x13
	ldr x10, [x13]
	cmp x10, #10
	b.eq LBB403_104
	cmp x10, #6
	b.ne LBB403_104
	mov x16, x26
	mov x15, x5
	mov x2, x7
	mov x23, x17
	ldr x10, [x13, #1016]
	lsr x14, x10, #32
	cbz x14, LBB403_95
	cmp w12, w14
	b.ne LBB403_95
	mov x12, #0
	b LBB403_98
LBB403_95:
	ldr x10, [x13, #1024]
	lsr x14, x10, #32
	mov x17, x23
	mov x7, x2
	mov x5, x15
	mov x26, x16
	cbz x14, LBB403_104
	cmp w12, w14
	b.ne LBB403_104
	mov w12, #1
LBB403_98:
	add x12, x13, x12, lsl #3
	ldr x12, [x12, #1032]
	ldr x13, [x9, #40]
	cmp x13, x12
	mov x17, x23
	mov x7, x2
	mov x5, x15
	mov x26, x16
	b.ne LBB403_104
	tbnz w10, #30, LBB403_101
	ldr w8, [sp, #60]
	and w8, w8, #0xff
	cmp w8, #78
	b.hi LBB403_140
	b LBB403_167
LBB403_101:
	tbnz w10, #31, LBB403_136
	ldr w9, [x9, #56]
	mov x17, x23
	mov x7, x2
	mov x5, x15
	mov x26, x16
	cbz w9, LBB403_104
	mov w8, #2
	mov x15, x9
	b LBB403_137
LBB403_104:
	ldr x0, [x19, #96]
	mov x1, x8
	mov x2, x17
	mov x3, x24
	mov x4, x22
	mov x21, x7
	mov x23, x5
	mov x6, x7
	mov x24, x17
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_named_property_store_inline_cache
	and w9, w0, #0xff
	cmp w9, #2
	ldr x25, [sp, #80]
	b.ne LBB403_107
	mov x0, x24
	mov x1, x23
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::prototype_chain_has_proxy
	str x23, [sp, #32]
	tbz w0, #0, LBB403_110
	str x21, [sp, #16]
	mov w8, #1
	stp w8, w26, [sp, #8]
	str x23, [sp]
	add x0, sp, #160
	mov x1, x19
	mov x2, x24
	ldp x3, x4, [sp, #40]
	ldp x5, x6, [sp, #64]
	ldr x7, [sp, #104]
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::set_property_on_value
	b LBB403_115
LBB403_107:
	ldr w10, [sp, #60]
	and w8, w10, #0xff
	cmp w8, #79
	ldr x8, [sp, #104]
	b.hs LBB403_112
LBB403_108:
	ldr w1, [x8, #4]
	mov x0, x19
	mov x2, x22
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
	ldr x8, [sp, #104]
	ldr w9, [x8, #56]
	adds w9, w9, w20
	b.hs LBB403_169
LBB403_109:
	str w9, [x8, #56]
	ldr x0, [sp, #88]
	b LBB403_123
LBB403_110:
	mov w25, #1
	sub x8, x29, #152
	mov x0, x24
	mov x1, x23
	mov w2, #1
	mov x3, x26
	mov x4, x21
	mov w5, #0
	bl lyng_ops::object::ordinary::ordinary_set
	ldur w8, [x29, #-152]
	cmp w8, #4
	b.ne LBB403_114
	ldurb w8, [x29, #-148]
	strb w8, [sp, #168]
	str x27, [sp, #160]
	ldr x25, [sp, #80]
	ldr x24, [sp, #96]
	b LBB403_115
LBB403_112:
	tbz w9, #0, LBB403_119
LBB403_113:
	str x27, [sp, #112]
	b LBB403_121
LBB403_114:
	str x21, [sp, #16]
	add x0, sp, #160
	stp w25, w26, [sp, #8]
	ldp x8, x3, [sp, #32]
	str x8, [sp]
	mov x1, x19
	ldp x24, x7, [sp, #96]
	mov x2, x24
	ldr x4, [sp, #48]
	ldp x5, x6, [sp, #64]
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::set_property_on_value
	ldr x25, [sp, #80]
LBB403_115:
	sub x0, x29, #152
	add x5, sp, #160
	mov x1, x19
	mov x21, x24
	mov x2, x24
	mov x3, x25
	ldr x4, [sp, #104]
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x10, [x29, #-152]
	ldurb w9, [x29, #-144]
	cmp x10, x27
	b.ne LBB403_162
	cmp w9, #2
	ldr x0, [sp, #88]
	ldr x8, [sp, #104]
	ldr w10, [sp, #60]
	b.eq LBB403_123
	and w10, w10, #0xff
	cmp w10, #78
	b.hi LBB403_124
LBB403_118:
	ldr w2, [x8, #4]
	mov x0, x19
	mov x1, x21
	mov x3, x22
	ldr x4, [sp, #32]
	mov x5, x26
	mov w6, #1
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_named_property_slow_path
	ldr x0, [sp, #88]
	b LBB403_32
LBB403_119:
	and w9, w10, #0xff
	cmp w9, #80
	b.ne LBB403_126
LBB403_120:
	mov x0, x24
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #112]
	str wzr, [sp, #120]
	str x0, [sp, #128]
LBB403_121:
	sub x0, x29, #152
	add x5, sp, #112
	mov x1, x19
	mov x2, x24
	mov x3, x25
	ldr x4, [sp, #104]
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x10, [x29, #-152]
	ldurb w9, [x29, #-144]
	cmp x10, x27
	b.ne LBB403_162
	ldr x0, [sp, #88]
	ldr x8, [sp, #104]
	tbnz w9, #0, LBB403_108
LBB403_123:
	ldr x9, [x8, #128]
	ldr x9, [x9, #48]
	ldr w8, [x8, #56]
	ldrb w8, [x9, x8]
Lloh1199:
	adrp x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
Lloh1200:
	add x9, x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x8, [x9, x8, lsl #3]
	stp x27, x8, [x0]
	b LBB403_25
LBB403_124:
	tbz w9, #0, LBB403_129
LBB403_125:
	str x27, [sp, #208]
	b LBB403_131
LBB403_126:
	ldr x9, [x19, #80]
	ldr w8, [x8, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB403_113
	ldr x9, [x19, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB403_113
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB403_113
	b LBB403_120
LBB403_129:
	ldr w8, [sp, #60]
	and w8, w8, #0xff
	cmp w8, #80
	b.ne LBB403_133
LBB403_130:
	mov x0, x21
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #208]
	str wzr, [sp, #216]
	str x0, [sp, #224]
LBB403_131:
	sub x0, x29, #152
	add x5, sp, #208
	mov x1, x19
	mov x2, x21
	ldr x3, [sp, #80]
	ldr x4, [sp, #104]
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x10, [x29, #-152]
	ldurb w9, [x29, #-144]
	cmp x10, x27
	b.ne LBB403_162
	ldr x0, [sp, #88]
	ldr x8, [sp, #104]
	tbnz w9, #0, LBB403_118
	b LBB403_123
LBB403_133:
	ldr x9, [x19, #80]
	ldr x8, [sp, #104]
	ldr w8, [x8, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB403_125
	ldr x9, [x19, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB403_125
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB403_125
	b LBB403_130
LBB403_136:
	mov w8, #5
LBB403_137:
	and w10, w10, #0x3fffffff
	ldr w9, [sp, #60]
	and w21, w9, #0xff
	stp w8, w10, [x29, #-152]
	stur w15, [x29, #-144]
	sub x1, x29, #152
	mov x0, x23
	bl lyng_gc::mutator::PrimitiveMutator::store_value
	ldr x8, [sp, #104]
	ldr w24, [x8, #4]
	cmp w21, #79
	b.lo LBB403_167
	tbnz w0, #0, LBB403_167
	ldr x11, [x19, #72]
LBB403_140:
	ldr w8, [sp, #60]
	and w8, w8, #0xff
	cmp w8, #80
	b.ne LBB403_148
LBB403_141:
	mov x0, x23
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x22, x0
	ldr x8, [sp, #80]
	cbz x8, LBB403_144
	ldr x8, [sp, #80]
	sub x8, x8, #1
	ldr x9, [x19, #56]
	cmp x8, x9
	b.hs LBB403_144
	ldr x9, [x19, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr x9, [sp, #104]
	ldr q0, [x9]
	str q0, [x8]
	ldp q0, q1, [x9, #48]
	ldp q3, q2, [x9, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB403_144:
	sub x0, x29, #152
	mov x1, x19
	mov x2, x23
	mov x3, x22
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
	ldur x10, [x29, #-152]
	ldurb w9, [x29, #-144]
	cmp x10, x27
	b.ne LBB403_162
	tbz w9, #0, LBB403_161
LBB403_146:
	ldr w8, [x19, #1640]
	add w8, w8, #1
	str w8, [x19, #1640]
	ldp x9, x0, [sp, #80]
	ldr x8, [sp, #104]
	cbz x9, LBB403_123
	sub x9, x9, #1
	b LBB403_46
LBB403_148:
	ldr x9, [x19, #80]
	sub w8, w24, #1
	cmp x9, x8
	b.ls LBB403_167
	ldr x8, [x11, x8, lsl #3]
	cbz x8, LBB403_167
	ldrh w8, [x8, #340]
	tbnz w8, #0, LBB403_141
	b LBB403_167
LBB403_151:
	mov w8, #5
LBB403_152:
	and x9, x9, #0x3fffffff
	and w24, w23, #0xff
	orr x9, x9, x4, lsl #32
	stur w8, [x29, #-152]
	stur x9, [x28, #76]
	sub x1, x29, #152
	mov x0, x21
	mov x2, x3
	bl lyng_gc::mutator::PrimitiveMutator::store_value
	cmp w24, #79
	b.lo LBB403_166
	cbnz w0, LBB403_166
	ldr x11, [x19, #72]
LBB403_155:
	and w8, w23, #0xff
	cmp w8, #80
	b.ne LBB403_163
LBB403_156:
	mov x0, x21
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x22, x0
	ldr x8, [sp, #80]
	cbz x8, LBB403_159
	ldr x8, [sp, #80]
	sub x8, x8, #1
	ldr x9, [x19, #56]
	cmp x8, x9
	b.hs LBB403_159
	ldr x9, [x19, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr x9, [sp, #104]
	ldr q0, [x9]
	str q0, [x8]
	ldp q0, q1, [x9, #48]
	ldp q3, q2, [x9, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB403_159:
	sub x0, x29, #152
	mov x1, x19
	mov x2, x21
	mov x3, x22
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
	ldur x10, [x29, #-152]
	ldurb w9, [x29, #-144]
	cmp x10, x27
	b.ne LBB403_162
	tbnz w9, #0, LBB403_146
LBB403_161:
	mov w8, #0
	mov w9, #0
	mov x10, #-9223372036854775808
	ldr x0, [sp, #88]
	b LBB403_23
LBB403_162:
	ldurb w8, [x29, #-141]
	ldurh w11, [x28, #81]
	orr w8, w11, w8, lsl #16
	ldur w11, [x29, #-140]
	ldur x22, [x29, #-136]
	ldp w24, w12, [x29, #-128]
	ldur q0, [x28, #104]
	stur q0, [x29, #-176]
	ldr x0, [sp, #88]
	b LBB403_23
LBB403_163:
	ldr x9, [x19, #80]
	ldr x8, [sp, #104]
	ldr w24, [x8, #4]
	sub w8, w24, #1
	cmp x9, x8
	b.ls LBB403_167
	ldr x8, [x11, x8, lsl #3]
	cbz x8, LBB403_167
	ldrh w8, [x8, #340]
	tbnz w8, #0, LBB403_156
	b LBB403_167
LBB403_166:
	ldr x8, [sp, #104]
	ldr w24, [x8, #4]
LBB403_167:
	mov x0, x19
	mov x1, x24
	mov x2, x22
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
	ldr x8, [sp, #104]
	ldr w8, [x8, #56]
	adds w9, w8, w20
	b.hs LBB403_169
	ldr x8, [sp, #104]
	b LBB403_109
LBB403_169:
Lloh1201:
	adrp x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGE
Lloh1202:
	add x0, x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGEOFF
Lloh1203:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGE
Lloh1204:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGEOFF
	mov w1, #41
	bl core::option::expect_failed
LBB403_170:
Lloh1205:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.766@PAGE
Lloh1206:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.766@PAGEOFF
	mov x0, x8
	mov x1, x10
	bl core::panicking::panic_bounds_check
	.loh AdrpAdd	Lloh1195, Lloh1196
	.loh AdrpAdd	Lloh1197, Lloh1198
	.loh AdrpAdd	Lloh1199, Lloh1200
	.loh AdrpAdd	Lloh1203, Lloh1204
	.loh AdrpAdd	Lloh1201, Lloh1202
	.loh AdrpAdd	Lloh1205, Lloh1206
