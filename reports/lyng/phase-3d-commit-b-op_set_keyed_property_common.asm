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
	sub sp, sp, #1440
	str xzr, [sp]
	mov x20, x1
	ldr w28, [x1, #4]
	ldr w5, [x1, #56]
	ldrb w9, [x1, #148]
	mov w8, #152
	strb w8, [x1, #148]
	ldr x10, [x1, #128]
	ldr x1, [x10, #56]
	subs x8, x1, x5
	b.lo LBB402_184
	mov x19, x0
	add x11, sp, #1216
	mov x26, #33
	movk x26, #32768, lsl #48
	ldr x10, [x10, #48]
	add x1, x10, x5
	cmp w9, #152
	b.ne LBB402_185
	and x9, x8, #0x7ffffffffffffffe
	cmp x8, #4
	ccmp x9, #4, #4, hs
	b.eq LBB402_30
	ldrh w3, [x1, #4]
	cbz w3, LBB402_30
	ldrb w9, [x1, #1]
	ldrb w10, [x1, #2]
	mov w12, #6
	ldrb w8, [x1, #3]
	ldr x25, [x20, #80]
	ldr w11, [x20, #20]
	ldr x1, [x25, #32]
	add w9, w11, w9
	cmp x1, x9
	b.ls LBB402_187
LBB402_5:
	add w0, w11, w10
	cmp x1, x0
	b.ls LBB402_268
	add w8, w11, w8
	cmp x1, x8
	b.ls LBB402_269
	ldr x27, [x20, #88]
	ldr x22, [x20, #136]
	ldr x10, [x25, #24]
	ldr x21, [x10, x9, lsl #3]
	and x24, x21, #0x7ff8000000000000
	ubfx x23, x21, #32, #16
	sub w11, w23, #1
	mov x9, #9221120237041090560
	cmp x24, x9
	ccmp w11, #1, #2, eq
	b.ls LBB402_32
	stp x27, x22, [sp, #112]
	stp w3, w12, [sp, #88]
	and w11, w2, #0xff
	ldp x22, x4, [x20, #96]
	ldp x12, x6, [x20, #112]
	str w11, [sp, #68]
	sub w11, w11, #83
	str w11, [sp, #108]
	ldr x17, [x10, x0, lsl #3]
	ldr x26, [x10, x8, lsl #3]
	and x8, x26, #0x7fffffff00000000
	and x8, x8, #0xfff8ffffffffffff
	cmp x24, x9
	ccmp w23, #5, #0, eq
	ccmp w21, #0, #4, eq
	mov x9, #17179869184
	movk x9, #32760, lsl #48
	ccmp x8, x9, #0, ne
	str x4, [sp, #96]
	b.ne LBB402_41
	ldr x27, [sp, #120]
	tbnz w26, #31, LBB402_42
	stp x22, x12, [sp, #72]
	str x6, [sp, #40]
	cbz w3, LBB402_15
	mov x8, #1
	movk x8, #9, lsl #32
	movk x8, #32760, lsl #48
	cmp x17, x8
	b.eq LBB402_15
	ldr x8, [x25, #104]
	sub w11, w28, #1
	cmp x8, x11
	b.ls LBB402_15
	ldr x8, [x25, #96]
	add x8, x8, x11, lsl #5
	ldr x9, [x8, #16]
	sub w10, w3, #1
	cmp x9, x10
	b.ls LBB402_15
	ldr x8, [x8, #8]
	mov w9, #1128
	umaddl x8, w10, w9, x8
	ldr x9, [x8]
	cmp x9, #10
	ccmp x9, #4, #2, ne
	b.lo LBB402_160
LBB402_15:
	ldr w2, [x20, #4]
	mov x0, x25
	ldr x28, [sp, #112]
	mov x1, x28
	mov x4, x21
	mov x5, x26
	mov x6, x17
	mov x22, x17
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_keyed_dense_index_store_inline_cache_hit
	and w8, w0, #0xff
	cmp w8, #2
	b.ne LBB402_76
	add x0, sp, #224
	mov x1, x25
	mov x2, x28
	mov x3, x21
	mov x4, x26
	mov x5, x22
	bl lyng_vm::vm::values::<impl lyng_vm::vm::Vm>::mapped_arguments_set
	ldr x8, [sp, #224]
	mov x27, #33
	movk x27, #32768, lsl #48
	add x9, x27, #1
	str x8, [sp, #48]
	str x9, [sp, #32]
	cmp x8, x9
	b.ne LBB402_78
	str x22, [sp, #56]
	str x22, [sp, #8]
	add x0, sp, #272
	stp w21, w26, [sp]
	mov x1, x25
	mov x2, x28
	ldp x3, x5, [sp, #72]
	ldr x4, [sp, #96]
	ldr x6, [sp, #40]
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_set_typed_array_index
	sub x0, x29, #152
	add x5, sp, #272
	mov x1, x25
	mov x2, x28
	ldr x3, [sp, #120]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x27
	b.ne LBB402_246
	cmp w5, #3
	ldr x27, [sp, #120]
	b.eq LBB402_152
	cmp w5, #2
	ldr x4, [sp, #56]
	b.ne LBB402_26
	add x0, sp, #320
	ldr x27, [sp, #112]
	mov x1, x27
	mov x2, x21
	mov x3, x26
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_set_engine_array_index
	sub x0, x29, #152
	add x5, sp, #320
	mov x1, x25
	mov x2, x27
	ldr x27, [sp, #120]
	mov x3, x27
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_246
	cmp w5, #3
	b.eq LBB402_152
	cmp w5, #2
	ldr x4, [sp, #56]
	b.ne LBB402_26
	add x0, sp, #368
	ldr x27, [sp, #112]
	mov x1, x27
	mov x2, x21
	mov x3, x26
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_set_ordinary_index_data_property
	sub x0, x29, #152
	add x5, sp, #368
	mov x1, x25
	mov x2, x27
	ldr x27, [sp, #120]
	mov x3, x27
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_246
	cmp w5, #3
	b.eq LBB402_152
	cmp w5, #2
	ldr x6, [sp, #40]
	ldp x22, x12, [sp, #72]
	ldr x4, [sp, #96]
	ldr x17, [sp, #56]
	b.eq LBB402_42
LBB402_26:
	ldr w8, [sp, #108]
	cmp w8, #2
	ldr x23, [sp, #112]
	b.hs LBB402_86
	tbnz w5, #0, LBB402_81
	ldr w8, [sp, #68]
	cmp w8, #84
	b.ne LBB402_201
LBB402_29:
	mov x0, x23
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #416]
	str wzr, [sp, #424]
	str x0, [sp, #432]
	mov x24, #33
	movk x24, #32768, lsl #48
	b LBB402_82
LBB402_30:
	sub x8, x26, #12
	stur x8, [x29, #-152]
	stp w28, w5, [x29, #-144]
LBB402_31:
	ldur q0, [x11, #152]
	ldur q1, [x11, #168]
	stp q0, q1, [x19]
	ldur q0, [x11, #184]
	str q0, [x19, #32]
	b LBB402_250
LBB402_32:
	mov x0, x27
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x24, x0
	cbz x22, LBB402_35
	sub x8, x22, #1
	ldr x9, [x25, #56]
	cmp x8, x9
	b.hs LBB402_35
	ldr x9, [x25, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB402_35:
	sub x0, x29, #152
	mov x1, x25
	mov x2, x27
	mov x3, x24
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x26
	b.ne LBB402_246
	tbz w5, #0, LBB402_69
	ldr w8, [x25, #1640]
	add w8, w8, #1
	str w8, [x25, #1640]
	cbz x22, LBB402_152
	sub x8, x22, #1
	ldr x9, [x25, #56]
	cmp x8, x9
	b.hs LBB402_152
LBB402_39:
	ldr x9, [x25, #48]
LBB402_40:
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x8]
	str q0, [x20]
	ldp q0, q1, [x8, #48]
	ldp q3, q2, [x8, #16]
	stp q0, q1, [x20, #48]
	stp q3, q2, [x20, #16]
	b LBB402_152
LBB402_41:
	mov x9, #30064771072
	movk x9, #32760, lsl #48
	cmp x8, x9
	ccmp w26, #0, #4, eq
	ldr x27, [sp, #120]
	b.ne LBB402_71
LBB402_42:
	mov x8, x25
	mov x25, x19
	stp x21, x17, [sp, #48]
	ldr x19, [sp, #112]
	stp x8, x19, [x29, #-152]
	stp x22, x12, [sp, #72]
	stp x22, x4, [x29, #-136]
	mov x21, x8
	stp x12, x6, [x29, #-120]
	mov x22, x6
	stur x20, [x29, #-104]
	sub x0, x29, #208
	sub x1, x29, #152
	mov x2, x26
	mov w3, #1
	bl lyng_ops::object::conversions::to_primitive
	ldp x8, x5, [x29, #-208]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_58
	ldr w3, [x20, #4]
	ldr w4, [x20, #56]
	add x0, sp, #464
	mov x26, x21
	mov x1, x21
	mov x28, x19
	mov x2, x19
	bl lyng_vm::vm::values::<impl lyng_vm::vm::Vm>::value_to_property_key
	mov x9, #33
	movk x9, #32768, lsl #48
	ldr x8, [sp, #464]
	ldr w5, [sp, #472]
	cmp x8, x9
	b.ne LBB402_59
	ldr w8, [sp, #476]
	ldr w3, [sp, #88]
	mov x6, x22
	ldp x22, x12, [sp, #72]
	mov x9, x26
	mov x26, x8
	ldp x21, x17, [sp, #48]
	ldr x4, [sp, #96]
	mov x19, x25
	mov x25, x9
	mov x8, #9221120237041090560
	cmp x24, x8
	ccmp w21, #0, #4, eq
	ccmp w23, #5, #0, ne
	b.ne LBB402_72
LBB402_45:
	cbz w5, LBB402_96
	cmp w5, #1
	b.ne LBB402_122
	mov x28, x12
	str x6, [sp, #40]
	mov x8, x3
	ldr w3, [x20, #4]
	ldr x1, [x25, #104]
	mov x14, x8
	cbz w8, LBB402_223
	sub w8, w3, #1
	cmp x1, x8
	b.ls LBB402_223
	ldr x9, [x25, #96]
	add x9, x9, x8, lsl #5
	ldr x10, [x9, #16]
	sub w8, w14, #1
	cmp x10, x8
	b.ls LBB402_223
	ldr x9, [x9, #8]
	mov w10, #1128
	umaddl x9, w8, w10, x9
	ldr x8, [x9]
	cmp x8, #10
	b.eq LBB402_223
	cmp x8, #3
	b.hi LBB402_223
	ldr w8, [x9, #1116]
	cmp w8, w26
	b.ne LBB402_223
	ldr x8, [x9, #1088]
	cbz x8, LBB402_223
	ldr x10, [sp, #112]
	ldr x12, [x10, #224]
	mov w10, #-1
	add x10, x21, x10
	lsr w11, w10, #6
	cmp x11, x12
	b.hs LBB402_223
	ldr x12, [sp, #112]
	ldr x12, [x12, #216]
	and x10, x10, #0x3f
	ldr x11, [x12, x11, lsl #3]
	mov w12, #80
	umaddl x11, w10, w12, x11
	ldr w10, [x11]
	cmp w10, #1
	b.ne LBB402_223
	stp x19, x25, [sp, #72]
	mov x2, x17
	mov x16, x26
	mov x15, x21
	mov x23, x3
	ldr x10, [x9, #1096]
	ldp w12, w9, [x11, #52]
	ldr x11, [x11, #40]
	lsr x13, x8, #32
	cmp x13, #0
	csel w13, w13, wzr, ne
	cbz w12, LBB402_216
	cmp w12, w13
	ccmp x11, x10, #0, eq
	mov x3, x23
	ldr x27, [sp, #120]
	ldr w14, [sp, #88]
	ldp x19, x25, [sp, #72]
	mov x21, x15
	mov x26, x16
	mov x17, x2
	b.eq LBB402_218
	b LBB402_223
LBB402_58:
	mov x28, x19
	mov x26, x21
	ldp q0, q1, [x29, #-192]
	stp q0, q1, [sp, #480]
	str x5, [sp, #472]
LBB402_59:
	mov x19, x25
	mov x9, #-9223372036854775808
	cmp x8, x9
	b.ne LBB402_70
	cbnz w5, LBB402_70
	mov x22, #33
	movk x22, #32768, lsl #48
	ldr x24, [sp, #480]
	cbz x27, LBB402_64
	sub x8, x27, #1
	ldr x9, [x26, #56]
	cmp x8, x9
	b.hs LBB402_64
	ldr x9, [x26, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB402_64:
	sub x0, x29, #152
	mov x1, x26
	mov x2, x28
	mov x3, x24
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_246
	tbz w5, #0, LBB402_69
	ldr w8, [x26, #1640]
	add w8, w8, #1
	str w8, [x26, #1640]
	cbz x27, LBB402_152
	sub x8, x27, #1
	ldr x9, [x26, #56]
	cmp x8, x9
	b.hs LBB402_152
	ldr x9, [x26, #48]
	b LBB402_40
LBB402_69:
	mov w9, #0
	mov w5, #0
	mov x8, #-9223372036854775808
	b LBB402_249
LBB402_70:
	lsr w9, w5, #8
	ldr w10, [sp, #476]
	ldr x24, [sp, #480]
	add x11, sp, #233
	ldur q0, [x11, #255]
	str q0, [sp, #512]
	ldr x11, [sp, #504]
	b LBB402_248
LBB402_71:
	mov w5, #2
	str w5, [sp, #472]
	str w26, [sp, #476]
	mov x8, #33
	movk x8, #32768, lsl #48
	str x8, [sp, #464]
	ldr x28, [sp, #112]
	mov x8, #9221120237041090560
	cmp x24, x8
	ccmp w21, #0, #4, eq
	ccmp w23, #5, #0, ne
	b.eq LBB402_45
LBB402_72:
	str x17, [sp, #16]
	add x0, sp, #1216
	stp w5, w26, [sp, #8]
	str x21, [sp]
	mov x1, x25
	mov x2, x28
	mov x3, x22
	mov x5, x12
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::set_property_on_value
	ldr x8, [sp, #1216]
	mov x22, #33
	movk x22, #32768, lsl #48
	cmp x8, x22
	b.ne LBB402_87
	ldr w8, [sp, #108]
	cmp w8, #2
	b.hs LBB402_150
	ldrb w8, [sp, #1224]
	tbz w8, #0, LBB402_145
LBB402_75:
	stur x22, [x29, #-256]
	b LBB402_147
LBB402_76:
	ldr w8, [sp, #108]
	cmp w8, #2
	b.hs LBB402_150
	mov x22, #33
	movk x22, #32768, lsl #48
	str x22, [sp, #176]
	sub x0, x29, #152
	add x5, sp, #176
	b LBB402_148
LBB402_78:
	sub x0, x29, #152
	add x5, sp, #224
	mov x1, x25
	mov x2, x28
	ldr x3, [sp, #120]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x27
	b.ne LBB402_246
	ldp x23, x27, [sp, #112]
	tbz w5, #0, LBB402_152
	ldr w8, [sp, #108]
	cmp w8, #2
	b.hs LBB402_85
LBB402_81:
	mov x24, #33
	movk x24, #32768, lsl #48
	str x24, [sp, #416]
LBB402_82:
	sub x0, x29, #152
	add x5, sp, #416
	mov x1, x25
	mov x2, x23
	mov x3, x27
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x24
	b.ne LBB402_246
	tbz w5, #0, LBB402_152
	ldr x8, [sp, #48]
	ldr x9, [sp, #32]
	cmp x8, x9
	b.eq LBB402_86
LBB402_85:
	sub x0, x29, #152
	mov x1, x23
	mov x2, x21
	bl lyng_vm::vm::activation_objects::<impl lyng_vm::vm::Vm>::sync_engine_array_length
	ldur x8, [x29, #-152]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_214
LBB402_86:
	ldr w2, [x20, #4]
	mov x0, x25
	mov x1, x23
	ldr w3, [sp, #88]
	mov x4, x21
	mov x5, x26
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_keyed_index_access
	ldr w8, [x20, #56]
	ldr w9, [sp, #92]
	adds w8, w8, w9
	b.lo LBB402_151
	b LBB402_237
LBB402_87:
	ldr w5, [sp, #1224]
	mov x9, #-9223372036854775808
	cmp x8, x9
	b.ne LBB402_142
	cbnz w5, LBB402_142
	ldr x24, [sp, #1232]
	cbz x27, LBB402_92
	sub x8, x27, #1
	ldr x9, [x25, #56]
	cmp x8, x9
	b.hs LBB402_92
	ldr x9, [x25, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB402_92:
	sub x0, x29, #152
	mov x1, x25
	mov x2, x28
	mov x3, x24
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_153
	tbz w5, #0, LBB402_183
	ldr w8, [x25, #1640]
	add w8, w8, #1
	str w8, [x25, #1640]
	cbz x27, LBB402_152
	sub x8, x27, #1
	ldr x9, [x25, #56]
	cmp x8, x9
	b.lo LBB402_39
	b LBB402_152
LBB402_96:
	mov x24, x22
	str x12, [sp, #80]
	str x6, [sp, #40]
	cbz w3, LBB402_127
	mov x22, #1
	movk x22, #9, lsl #32
	movk x22, #32760, lsl #48
	cmp x17, x22
	b.eq LBB402_127
	ldr w28, [x20, #4]
	ldr x8, [x25, #104]
	sub w23, w28, #1
	cmp x8, x23
	b.ls LBB402_127
	ldr x8, [x25, #96]
	add x8, x8, x23, lsl #5
	ldr x9, [x8, #16]
	sub w10, w3, #1
	cmp x9, x10
	b.ls LBB402_127
	ldr x8, [x8, #8]
	mov w9, #1128
	umaddl x8, w10, w9, x8
	ldr x9, [x8]
	cmp x9, #10
	b.eq LBB402_127
	cmp x9, #4
	b.hs LBB402_127
	ldr x8, [x8, #1104]
	cbz w8, LBB402_127
	str x8, [sp, #72]
	mov w9, #4992
	sub x8, x29, #152
	ldr x1, [sp, #112]
	add x0, x1, x9
	mov x2, x21
	stp x10, x17, [sp, #48]
	bl lyng_objects::runtime::ObjectRuntime::object_header
	ldr x9, [sp, #72]
	ldr x17, [sp, #56]
	ldr w3, [sp, #88]
	ldurb w8, [x29, #-130]
	cmp w8, #3
	b.eq LBB402_127
	cbz w9, LBB402_127
	ldur w8, [x29, #-148]
	cmp w8, w9
	b.ne LBB402_127
	ldurh w8, [x29, #-132]
	lsr x9, x9, #32
	cmp w8, w9, uxth
	b.ne LBB402_127
	ldur w8, [x29, #-136]
	cbz w8, LBB402_127
	ldr x9, [sp, #112]
	ldr x10, [x9, #640]
	sub w9, w8, #1
	cmp x10, x9
	b.ls LBB402_127
	ldr x10, [sp, #112]
	ldr x10, [x10, #632]
	mov w11, #24
	umaddl x9, w9, w11, x10
	ldrb w10, [x9, #19]
	cmp w10, #1
	b.ne LBB402_127
	ldr x11, [x9, #8]
	mov w10, w26
	cmp x11, x10
	b.ls LBB402_127
	ldr x9, [x9]
	ldr x9, [x9, x10, lsl #3]
	cmp x9, x22
	b.eq LBB402_127
	stp w26, w8, [x29, #-148]
	mov w8, #2
	stur w8, [x29, #-152]
	sub x1, x29, #152
	ldr x0, [sp, #112]
	mov x2, x17
	bl lyng_gc::mutator::PrimitiveMutator::store_value
	ldr x17, [sp, #56]
	ldr w3, [sp, #88]
	cbz w0, LBB402_127
	mov x8, x25
	mov x25, x19
	mov x21, x8
	ldr x8, [x8, #104]
	cmp x8, x23
	b.ls LBB402_117
	ldr x8, [x21, #96]
	add x8, x8, x23, lsl #5
	ldr x9, [x8, #16]
	ldr x10, [sp, #48]
	cmp x9, x10
	b.ls LBB402_117
	ldr x8, [x8, #8]
	mov w9, #1128
	ldr x10, [sp, #48]
	umaddl x0, w10, w9, x8
	ldr x8, [x0]
	cmp x8, #10
	b.eq LBB402_117
	bl lyng_vm::vm::feedback::FeedbackSiteState::record_execution
LBB402_117:
	ldp x0, x1, [x21, #120]
	mov x2, x28
	bl lyng_vm::vm::tiering::<impl lyng_vm::vm::Vm>::observe_tier_feedback_event
	ldr w8, [sp, #108]
	cmp w8, #2
	ldp x2, x3, [sp, #112]
	b.hs LBB402_120
	mov x22, #33
	movk x22, #32768, lsl #48
	str x22, [sp, #544]
	sub x0, x29, #152
	add x5, sp, #544
	mov x1, x21
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_270
	mov x19, x25
	tbz w5, #0, LBB402_152
LBB402_120:
	ldr w8, [x20, #56]
	ldr w9, [sp, #92]
	adds w8, w8, w9
	b.hs LBB402_237
	str w8, [x20, #56]
	mov x19, x25
	b LBB402_152
LBB402_122:
	mov x23, x3
	str x17, [sp, #16]
	add x0, sp, #1120
	stp w5, w26, [sp, #8]
	str x21, [sp]
	mov x1, x25
	mov x2, x28
	mov x3, x22
	mov x5, x12
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::set_property_on_value
	sub x0, x29, #152
	add x5, sp, #1120
	mov x1, x25
	mov x2, x28
	mov x3, x27
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_246
	cmp w5, #2
	b.eq LBB402_152
	ldr w8, [sp, #108]
	cmp w8, #2
	b.hs LBB402_192
	tbz w5, #0, LBB402_188
LBB402_126:
	mov x22, #33
	movk x22, #32768, lsl #48
	str x22, [sp, #1168]
	b LBB402_190
LBB402_127:
	ldr w2, [x20, #4]
	mov x0, x25
	ldr x28, [sp, #112]
	mov x1, x28
	mov x4, x21
	mov x5, x26
	mov x6, x17
	mov x23, x17
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_keyed_dense_index_store_inline_cache_hit
	and w8, w0, #0xff
	cmp w8, #2
	b.ne LBB402_143
	add x0, sp, #640
	mov x1, x25
	mov x2, x28
	mov x3, x21
	mov x4, x26
	mov x5, x23
	bl lyng_vm::vm::values::<impl lyng_vm::vm::Vm>::mapped_arguments_set
	ldr x8, [sp, #640]
	mov x22, #33
	movk x22, #32768, lsl #48
	add x9, x22, #1
	cmp x8, x9
	b.ne LBB402_156
	str x23, [sp, #56]
	str x23, [sp, #8]
	add x0, sp, #688
	stp w21, w26, [sp]
	mov x1, x25
	mov x2, x28
	mov x3, x24
	ldr x4, [sp, #96]
	ldr x5, [sp, #80]
	ldr x6, [sp, #40]
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_set_typed_array_index
	sub x0, x29, #152
	add x5, sp, #688
	mov x1, x25
	mov x2, x28
	mov x3, x27
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_246
	cmp w5, #3
	b.eq LBB402_152
	cmp w5, #2
	b.ne LBB402_193
	add x0, sp, #736
	mov x1, x28
	mov x2, x21
	mov x3, x26
	ldr x4, [sp, #56]
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_set_engine_array_index
	sub x0, x29, #152
	add x5, sp, #736
	mov x1, x25
	mov x2, x28
	mov x3, x27
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_246
	cmp w5, #3
	b.eq LBB402_152
	cmp w5, #2
	b.ne LBB402_193
	mov x23, x19
	add x0, sp, #784
	mov x1, x28
	str x21, [sp, #48]
	mov x2, x21
	str x26, [sp, #72]
	mov x3, x26
	ldr x4, [sp, #56]
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_set_ordinary_index_data_property
	sub x0, x29, #152
	add x5, sp, #784
	mov x22, x25
	mov x1, x25
	mov x2, x28
	mov x3, x27
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_215
	cmp w5, #3
	ldp x28, x27, [sp, #112]
	ldr x21, [sp, #48]
	ldr x26, [sp, #72]
	mov x19, x23
	b.eq LBB402_152
	mov x25, x22
	cmp w5, #2
	b.ne LBB402_193
	ldr x8, [sp, #56]
	str x8, [sp, #16]
	add x0, sp, #832
	stp wzr, w26, [sp, #8]
	str x21, [sp]
	mov x1, x25
	mov x2, x28
	mov x3, x24
	ldr x4, [sp, #96]
	ldr x5, [sp, #80]
	ldr x6, [sp, #40]
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::set_property_on_value
	sub x0, x29, #152
	add x5, sp, #832
	mov x1, x25
	mov x2, x28
	mov x3, x27
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_215
	cmp w5, #2
	ldp x28, x27, [sp, #112]
	ldr x21, [sp, #48]
	ldr x26, [sp, #72]
	mov x19, x23
	b.eq LBB402_152
	mov x25, x22
	ldr w8, [sp, #108]
	cmp w8, #2
	b.hs LBB402_212
	mov w22, #0
	b LBB402_195
LBB402_142:
	lsr w9, w5, #8
	ldr w10, [sp, #1228]
	add x11, sp, #1216
	ldur q0, [x11, #24]
	stur q0, [x29, #-208]
	ldr x24, [sp, #1232]
	ldr x11, [sp, #1256]
	b LBB402_154
LBB402_143:
	ldr w8, [sp, #108]
	cmp w8, #2
	b.hs LBB402_150
	mov x22, #33
	movk x22, #32768, lsl #48
	str x22, [sp, #592]
	sub x0, x29, #152
	add x5, sp, #592
	b LBB402_148
LBB402_145:
	ldr w8, [sp, #68]
	cmp w8, #84
	b.ne LBB402_180
LBB402_146:
	mov x0, x28
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	stur x8, [x29, #-256]
	stur wzr, [x29, #-248]
	stur x0, [x29, #-240]
LBB402_147:
	sub x0, x29, #152
	sub x5, x29, #256
LBB402_148:
	mov x1, x25
	mov x2, x28
	mov x3, x27
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_246
	tbz w5, #0, LBB402_152
LBB402_150:
	ldr w8, [x20, #56]
	ldr w9, [sp, #92]
	adds w8, w8, w9
	b.hs LBB402_237
LBB402_151:
	str w8, [x20, #56]
LBB402_152:
	ldr x8, [x20, #128]
	ldr x8, [x8, #48]
	ldr w9, [x20, #56]
	ldrb w8, [x8, x9]
Lloh1157:
	adrp x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
Lloh1158:
	add x9, x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x10, [x9, x8, lsl #3]
	mov x8, #33
	movk x8, #32768, lsl #48
	stp x8, x10, [x19]
	b LBB402_250
LBB402_153:
	ldurb w9, [x29, #-141]
	add x11, sp, #1216
	ldurh w10, [x11, #161]
	orr w9, w10, w9, lsl #16
	ldur w10, [x29, #-140]
	ldur q0, [x11, #176]
	stur q0, [x29, #-208]
	ldur x24, [x29, #-136]
	ldur x11, [x29, #-112]
LBB402_154:
	stur x11, [x29, #-192]
LBB402_155:
	ldur q0, [x29, #-208]
	str q0, [sp, #512]
	ldur x11, [x29, #-192]
	b LBB402_248
LBB402_156:
	sub x0, x29, #152
	add x5, sp, #640
	mov x1, x25
	mov x2, x28
	mov x3, x27
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_246
	tbz w5, #0, LBB402_152
	ldr w8, [sp, #108]
	cmp w8, #2
	b.hs LBB402_212
	mov w22, #0
	b LBB402_208
LBB402_160:
	ldr x22, [x8, #1104]
	cbz w22, LBB402_15
	mov w9, #4992
	sub x8, x29, #152
	ldr x1, [sp, #112]
	add x0, x1, x9
	mov x2, x21
	stp x11, x17, [sp, #48]
	str x10, [sp, #32]
	bl lyng_objects::runtime::ObjectRuntime::object_header
	mov x12, #1
	movk x12, #9, lsl #32
	movk x12, #32760, lsl #48
	ldr x17, [sp, #56]
	ldr w3, [sp, #88]
	ldurb w8, [x29, #-130]
	cmp w8, #3
	b.eq LBB402_15
	cbz w22, LBB402_15
	ldur w8, [x29, #-148]
	cmp w8, w22
	b.ne LBB402_15
	ldurh w8, [x29, #-132]
	lsr x9, x22, #32
	cmp w8, w9, uxth
	b.ne LBB402_15
	ldur w8, [x29, #-136]
	cbz w8, LBB402_15
	ldr x9, [sp, #112]
	ldr x10, [x9, #640]
	sub w9, w8, #1
	cmp x10, x9
	b.ls LBB402_15
	ldr x10, [sp, #112]
	ldr x10, [x10, #632]
	mov w11, #24
	umaddl x9, w9, w11, x10
	ldrb w10, [x9, #19]
	cmp w10, #1
	b.ne LBB402_15
	ldr x11, [x9, #8]
	and x10, x26, #0x7fffffff
	cmp x11, x10
	b.ls LBB402_15
	ldr x9, [x9]
	ldr x9, [x9, x10, lsl #3]
	cmp x9, x12
	b.eq LBB402_15
	stp w26, w8, [x29, #-148]
	mov w8, #2
	stur w8, [x29, #-152]
	sub x1, x29, #152
	ldr x0, [sp, #112]
	mov x2, x17
	bl lyng_gc::mutator::PrimitiveMutator::store_value
	ldp x9, x17, [sp, #48]
	ldr w3, [sp, #88]
	cbz w0, LBB402_15
	mov x23, x19
	mov x21, x25
	ldr x8, [x25, #104]
	cmp x8, x9
	b.ls LBB402_175
	ldr x8, [x21, #96]
	ldr x9, [sp, #48]
	add x8, x8, x9, lsl #5
	ldr x9, [x8, #16]
	ldr x10, [sp, #32]
	cmp x9, x10
	b.ls LBB402_175
	ldr x8, [x8, #8]
	mov w9, #1128
	ldr x10, [sp, #32]
	umaddl x0, w10, w9, x8
	ldr x8, [x0]
	cmp x8, #10
	b.eq LBB402_175
	bl lyng_vm::vm::feedback::FeedbackSiteState::record_execution
LBB402_175:
	ldp x0, x1, [x21, #120]
	mov x2, x28
	bl lyng_vm::vm::tiering::<impl lyng_vm::vm::Vm>::observe_tier_feedback_event
	ldr w8, [sp, #108]
	cmp w8, #2
	ldp x2, x3, [sp, #112]
	b.hs LBB402_178
	mov x22, #33
	movk x22, #32768, lsl #48
	str x22, [sp, #128]
	sub x0, x29, #152
	add x5, sp, #128
	mov x1, x21
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_215
	mov x19, x23
	tbz w5, #0, LBB402_152
LBB402_178:
	ldr w8, [x20, #56]
	ldr w9, [sp, #92]
	adds w8, w8, w9
	b.hs LBB402_237
	str w8, [x20, #56]
	mov x19, x23
	b LBB402_152
LBB402_180:
	ldr x9, [x25, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB402_75
	ldr x9, [x25, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB402_75
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB402_75
	b LBB402_146
LBB402_183:
	mov w5, #0
	mov w9, #0
	mov x8, #-9223372036854775808
	b LBB402_155
LBB402_184:
Lloh1159:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.45@PAGE
Lloh1160:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.45@PAGEOFF
	mov x0, x5
	mov x2, x1
	bl core::slice::index::slice_index_fail
LBB402_185:
	mov x21, x2
	sub x0, x29, #152
	mov x2, x8
	mov w3, #1
	mov x4, x28
	bl lyng_vm::vm::dispatch::decode_abc_operands_wide
	add x11, sp, #1216
	ldur x8, [x29, #-152]
	cmp x8, x26
	b.ne LBB402_31
	ldurh w9, [x29, #-140]
	ldurh w10, [x29, #-138]
	ldurh w8, [x29, #-136]
	ldur w3, [x29, #-144]
	ldur w12, [x29, #-132]
	mov x2, x21
	ldr x25, [x20, #80]
	ldr w11, [x20, #20]
	ldr x1, [x25, #32]
	add w9, w11, w9
	cmp x1, x9
	b.hi LBB402_5
LBB402_187:
Lloh1161:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.59@PAGE
Lloh1162:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.59@PAGEOFF
	mov x0, x9
	bl core::panicking::panic_bounds_check
LBB402_188:
	ldr w8, [sp, #68]
	cmp w8, #84
	b.ne LBB402_198
LBB402_189:
	mov x0, x28
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #1168]
	str wzr, [sp, #1176]
	str x0, [sp, #1184]
	mov x22, #33
	movk x22, #32768, lsl #48
LBB402_190:
	sub x0, x29, #152
	add x5, sp, #1168
	mov x1, x25
	mov x2, x28
	mov x3, x27
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_246
	tbz w5, #0, LBB402_152
LBB402_192:
	ldr w1, [x20, #4]
	mov x0, x25
	mov x2, x23
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_keyed_generic_slow_path
	b LBB402_150
LBB402_193:
	ldr w8, [sp, #108]
	cmp w8, #2
	b.hs LBB402_213
	mov w22, #1
LBB402_195:
	tbnz w5, #0, LBB402_208
	ldr w8, [sp, #68]
	cmp w8, #84
	b.ne LBB402_204
LBB402_197:
	mov x0, x28
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #880]
	str wzr, [sp, #888]
	str x0, [sp, #896]
	mov x23, #33
	movk x23, #32768, lsl #48
	b LBB402_209
LBB402_198:
	ldr x9, [x25, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB402_126
	ldr x9, [x25, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB402_126
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB402_126
	b LBB402_189
LBB402_201:
	ldr x9, [x25, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB402_81
	ldr x9, [x25, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB402_81
	ldrh w8, [x8, #340]
	tbnz w8, #0, LBB402_29
	b LBB402_81
LBB402_204:
	ldr x9, [x25, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB402_207
	ldr x9, [x25, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB402_207
	ldrh w8, [x8, #340]
	ldp x28, x27, [sp, #112]
	tbnz w8, #0, LBB402_197
	b LBB402_208
LBB402_207:
	ldp x28, x27, [sp, #112]
LBB402_208:
	mov x23, #33
	movk x23, #32768, lsl #48
	str x23, [sp, #880]
LBB402_209:
	sub x0, x29, #152
	add x5, sp, #880
	mov x1, x25
	mov x2, x28
	mov x3, x27
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x23
	b.ne LBB402_246
	tbz w5, #0, LBB402_152
	tbnz w22, #0, LBB402_213
LBB402_212:
	sub x0, x29, #152
	mov x1, x28
	mov x2, x21
	bl lyng_vm::vm::activation_objects::<impl lyng_vm::vm::Vm>::sync_engine_array_length
	ldur x8, [x29, #-152]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_214
LBB402_213:
	ldr w2, [x20, #4]
	mov x0, x25
	mov x1, x28
	ldr w3, [sp, #88]
	mov x4, x21
	mov x5, x26
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_keyed_index_access
	b LBB402_150
LBB402_214:
	ldp w5, w10, [x29, #-144]
	lsr w9, w5, #8
	add x11, sp, #1216
	b LBB402_247
LBB402_215:
	ldurb w9, [x29, #-141]
	add x11, sp, #1216
	ldurh w10, [x11, #161]
	orr w9, w10, w9, lsl #16
	ldur w10, [x29, #-140]
	ldur q0, [x11, #176]
	str q0, [sp, #512]
	ldur x24, [x29, #-136]
	ldur x11, [x29, #-112]
	str x11, [sp, #528]
	mov x19, x23
	b LBB402_249
LBB402_216:
	mov x3, x23
	ldr x27, [sp, #120]
	ldr w14, [sp, #88]
	ldp x19, x25, [sp, #72]
	mov x21, x15
	mov x26, x16
	mov x17, x2
	cbnz w13, LBB402_223
	cmp x11, x10
	b.ne LBB402_223
LBB402_218:
	tbnz w8, #30, LBB402_220
	ldr x21, [sp, #80]
	mov x0, x21
	mov x1, x23
	ldr w2, [sp, #88]
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
	ldr w8, [sp, #108]
	cmp w8, #2
	ldr x22, [sp, #112]
	b.lo LBB402_257
	b LBB402_265
LBB402_220:
	tbnz w8, #31, LBB402_254
	mov x3, x23
	ldr x27, [sp, #120]
	ldr w14, [sp, #88]
	ldp x19, x25, [sp, #72]
	mov x21, x15
	mov x26, x16
	mov x17, x2
	cbz w9, LBB402_223
	mov w10, #2
	mov x15, x9
	b LBB402_255
LBB402_223:
	ldr x0, [x25, #96]
	ldr x23, [sp, #112]
	mov x2, x23
	mov x4, x14
	mov x5, x21
	mov x6, x26
	mov x24, x17
	mov x7, x17
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_keyed_property_store_inline_cache
	and w8, w0, #0xff
	cmp w8, #2
	b.ne LBB402_229
	str x24, [sp, #16]
	mov w8, #1
	stp w8, w26, [sp, #8]
	str x21, [sp]
	add x0, sp, #1024
	mov x1, x25
	mov x2, x23
	mov x3, x22
	ldr x4, [sp, #96]
	mov x5, x28
	ldr x6, [sp, #40]
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::set_property_on_value
	sub x0, x29, #152
	add x5, sp, #1024
	mov x1, x25
	mov x2, x23
	mov x3, x27
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_246
	cmp w5, #2
	b.eq LBB402_152
	ldr w8, [sp, #108]
	cmp w8, #2
	b.hs LBB402_245
	tbz w5, #0, LBB402_241
LBB402_228:
	mov x22, #33
	movk x22, #32768, lsl #48
	str x22, [sp, #1072]
	b LBB402_243
LBB402_229:
	ldr w9, [sp, #108]
	cmp w9, #2
	b.hs LBB402_236
	tbz w8, #0, LBB402_232
LBB402_231:
	mov x22, #33
	movk x22, #32768, lsl #48
	str x22, [sp, #976]
	b LBB402_234
LBB402_232:
	ldr w8, [sp, #68]
	cmp w8, #84
	b.ne LBB402_238
LBB402_233:
	mov x0, x23
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #976]
	str wzr, [sp, #984]
	str x0, [sp, #992]
	mov x22, #33
	movk x22, #32768, lsl #48
LBB402_234:
	sub x0, x29, #152
	add x5, sp, #976
	mov x1, x25
	mov x2, x23
	mov x3, x27
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_246
	tbz w5, #0, LBB402_152
LBB402_236:
	ldr w1, [x20, #4]
	mov x0, x25
	ldr w2, [sp, #88]
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
	ldr w8, [x20, #56]
	ldr w9, [sp, #92]
	adds w8, w8, w9
	b.lo LBB402_151
LBB402_237:
Lloh1163:
	adrp x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGE
Lloh1164:
	add x0, x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGEOFF
Lloh1165:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGE
Lloh1166:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGEOFF
	mov w1, #41
	bl core::option::expect_failed
LBB402_238:
	ldr x9, [x25, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB402_231
	ldr x9, [x25, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB402_231
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB402_231
	b LBB402_233
LBB402_241:
	ldr w8, [sp, #68]
	cmp w8, #84
	b.ne LBB402_251
LBB402_242:
	mov x0, x23
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #1072]
	str wzr, [sp, #1080]
	str x0, [sp, #1088]
	mov x22, #33
	movk x22, #32768, lsl #48
LBB402_243:
	sub x0, x29, #152
	add x5, sp, #1072
	mov x1, x25
	mov x2, x23
	mov x3, x27
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_246
	tbz w5, #0, LBB402_152
LBB402_245:
	ldr w2, [x20, #4]
	mov x0, x25
	mov x1, x23
	ldr w3, [sp, #88]
	mov x4, x21
	mov x5, x26
	mov w6, #1
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_keyed_atom_slow_path
	b LBB402_150
LBB402_246:
	ldurb w9, [x29, #-141]
	add x11, sp, #1216
	ldurh w10, [x11, #161]
	orr w9, w10, w9, lsl #16
	ldur w10, [x29, #-140]
LBB402_247:
	ldur q0, [x11, #176]
	str q0, [sp, #512]
	ldur x24, [x29, #-136]
	ldur x11, [x29, #-112]
LBB402_248:
	str x11, [sp, #528]
LBB402_249:
	bfi w5, w9, #8, #24
	str x8, [x19]
	stp w5, w10, [x19, #8]
	str x24, [x19, #16]
	ldr q0, [sp, #512]
	stur q0, [x19, #24]
	ldr x8, [sp, #528]
	str x8, [x19, #40]
LBB402_250:
	add sp, sp, #1440
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
LBB402_251:
	.cfi_restore_state
	ldr x9, [x25, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB402_228
	ldr x9, [x25, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB402_228
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB402_228
	b LBB402_242
LBB402_254:
	mov w10, #5
LBB402_255:
	and w8, w8, #0x3fffffff
	stp w10, w8, [x29, #-152]
	stur w15, [x29, #-144]
	sub x1, x29, #152
	ldr x22, [sp, #112]
	mov x0, x22
	bl lyng_gc::mutator::PrimitiveMutator::store_value
	mov x25, x0
	ldr x21, [sp, #80]
	mov x0, x21
	mov x1, x23
	ldr w2, [sp, #88]
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
	ldr w8, [sp, #108]
	cmp w8, #2
	b.hs LBB402_265
	tbnz w25, #0, LBB402_262
LBB402_257:
	ldr w8, [sp, #68]
	cmp w8, #84
	b.ne LBB402_259
LBB402_258:
	mov x0, x22
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #928]
	str wzr, [sp, #936]
	str x0, [sp, #944]
	b LBB402_263
LBB402_259:
	ldr x9, [x21, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB402_262
	ldr x9, [x21, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB402_262
	ldrh w8, [x8, #340]
	tbnz w8, #0, LBB402_258
LBB402_262:
	mov x8, #33
	movk x8, #32768, lsl #48
	str x8, [sp, #928]
LBB402_263:
	sub x0, x29, #152
	add x5, sp, #928
	ldr x1, [sp, #80]
	ldp x2, x3, [sp, #112]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_267
	ldr x19, [sp, #72]
	tbz w5, #0, LBB402_152
LBB402_265:
	ldr w8, [x20, #56]
	ldr w9, [sp, #92]
	adds w8, w8, w9
	b.hs LBB402_237
	str w8, [x20, #56]
	ldr x19, [sp, #72]
	b LBB402_152
LBB402_267:
	ldurb w9, [x29, #-141]
	add x11, sp, #1216
	ldurh w10, [x11, #161]
	orr w9, w10, w9, lsl #16
	ldur w10, [x29, #-140]
	ldur q0, [x11, #176]
	str q0, [sp, #512]
	ldur x24, [x29, #-136]
	ldur x11, [x29, #-112]
	str x11, [sp, #528]
	ldr x19, [sp, #72]
	b LBB402_249
LBB402_268:
Lloh1167:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.59@PAGE
Lloh1168:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.59@PAGEOFF
	bl core::panicking::panic_bounds_check
LBB402_269:
Lloh1169:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.59@PAGE
Lloh1170:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.59@PAGEOFF
	mov x0, x8
	bl core::panicking::panic_bounds_check
LBB402_270:
	ldurb w9, [x29, #-141]
	add x11, sp, #1216
	ldurh w10, [x11, #161]
	orr w9, w10, w9, lsl #16
	ldur w10, [x29, #-140]
	ldur q0, [x11, #176]
	str q0, [sp, #512]
	ldur x24, [x29, #-136]
	ldur x11, [x29, #-112]
	str x11, [sp, #528]
	mov x19, x25
	b LBB402_249
	.loh AdrpAdd	Lloh1157, Lloh1158
	.loh AdrpAdd	Lloh1159, Lloh1160
	.loh AdrpAdd	Lloh1161, Lloh1162
	.loh AdrpAdd	Lloh1165, Lloh1166
	.loh AdrpAdd	Lloh1163, Lloh1164
	.loh AdrpAdd	Lloh1167, Lloh1168
	.loh AdrpAdd	Lloh1169, Lloh1170
