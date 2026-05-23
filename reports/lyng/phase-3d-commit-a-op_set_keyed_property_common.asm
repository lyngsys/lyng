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
	sub sp, sp, #1280
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
	b.lo LBB402_130
	mov x19, x0
	add x11, sp, #1040
	mov x26, #33
	movk x26, #32768, lsl #48
	ldr x10, [x10, #48]
	add x1, x10, x5
	cmp w9, #152
	b.ne LBB402_131
	and x9, x8, #0x7ffffffffffffffe
	cmp x8, #4
	ccmp x9, #4, #4, hs
	b.eq LBB402_25
	ldrh w3, [x1, #4]
	cbz w3, LBB402_25
	ldrb w9, [x1, #1]
	ldrb w10, [x1, #2]
	mov w12, #6
	ldrb w8, [x1, #3]
	ldr x21, [x20, #80]
	ldr w11, [x20, #20]
	ldr x1, [x21, #32]
	add w9, w11, w9
	cmp x1, x9
	b.ls LBB402_133
LBB402_5:
	add w0, w11, w10
	cmp x1, x0
	b.ls LBB402_177
	add w8, w11, w8
	cmp x1, x8
	b.ls LBB402_178
	ldr x23, [x20, #88]
	ldr x22, [x20, #136]
	ldr x10, [x21, #24]
	ldr x25, [x10, x9, lsl #3]
	and x24, x25, #0x7ff8000000000000
	ubfx x27, x25, #32, #16
	sub w11, w27, #1
	mov x9, #9221120237041090560
	cmp x24, x9
	ccmp w11, #1, #2, eq
	b.ls LBB402_27
	stp x22, x23, [sp, #112]
	str w12, [sp, #108]
	and w11, w2, #0xff
	ldp x26, x4, [x20, #96]
	ldp x12, x6, [x20, #112]
	str w11, [sp, #84]
	sub w23, w11, #83
	ldr x11, [x10, x0, lsl #3]
	ldr x10, [x10, x8, lsl #3]
	and x8, x10, #0x7fffffff00000000
	and x8, x8, #0xfff8ffffffffffff
	cmp x24, x9
	ccmp w27, #5, #0, eq
	ccmp w25, #0, #4, eq
	mov x9, #17179869184
	movk x9, #32760, lsl #48
	ccmp x8, x9, #0, ne
	str w3, [sp, #100]
	b.ne LBB402_35
	tbnz w10, #31, LBB402_36
	str x26, [sp, #88]
	stp x4, x6, [sp, #48]
	str x12, [sp, #64]
	str w23, [sp, #104]
	mov x0, x21
	ldr x26, [sp, #120]
	mov x1, x26
	mov x2, x28
	mov x4, x25
	mov x5, x10
	mov x22, x10
	mov x6, x11
	mov x23, x11
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_keyed_dense_index_store_inline_cache_hit
	and w8, w0, #0xff
	cmp w8, #2
	b.ne LBB402_58
	add x0, sp, #176
	mov x1, x21
	mov x2, x26
	mov x3, x25
	mov x4, x22
	mov x5, x23
	bl lyng_vm::vm::values::<impl lyng_vm::vm::Vm>::mapped_arguments_set
	ldr x8, [sp, #176]
	mov x28, #33
	movk x28, #32768, lsl #48
	add x9, x28, #1
	stp x9, x8, [sp, #24]
	cmp x8, x9
	str x22, [sp, #40]
	b.ne LBB402_66
	str x23, [sp, #72]
	str x23, [sp, #8]
	add x0, sp, #224
	stp w25, w22, [sp]
	mov x1, x21
	mov x2, x26
	ldr x3, [sp, #88]
	ldp x4, x6, [sp, #48]
	ldr x5, [sp, #64]
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_set_typed_array_index
	sub x0, x29, #152
	add x5, sp, #224
	mov x1, x21
	mov x2, x26
	ldr x3, [sp, #112]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x28
	b.ne LBB402_171
	cmp w5, #3
	b.eq LBB402_116
	cmp w5, #2
	ldr x3, [sp, #40]
	ldr w23, [sp, #104]
	ldr x4, [sp, #72]
	b.ne LBB402_21
	add x0, sp, #272
	ldr x26, [sp, #120]
	mov x1, x26
	mov x2, x25
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_set_engine_array_index
	sub x0, x29, #152
	add x5, sp, #272
	mov x1, x21
	mov x2, x26
	ldr x3, [sp, #112]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_171
	cmp w5, #3
	b.eq LBB402_116
	cmp w5, #2
	ldr x3, [sp, #40]
	ldr w23, [sp, #104]
	ldr x4, [sp, #72]
	b.ne LBB402_21
	add x0, sp, #320
	ldr x26, [sp, #120]
	mov x1, x26
	mov x2, x25
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_set_ordinary_index_data_property
	sub x0, x29, #152
	add x5, sp, #320
	mov x1, x21
	mov x2, x26
	ldr x3, [sp, #112]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_171
	cmp w5, #3
	b.eq LBB402_116
	cmp w5, #2
	ldp x10, x4, [sp, #40]
	ldr w23, [sp, #104]
	ldp x6, x12, [sp, #56]
	ldr x26, [sp, #88]
	ldr x11, [sp, #72]
	b.eq LBB402_36
LBB402_21:
	cmp w23, #2
	ldr x27, [sp, #120]
	b.hs LBB402_74
	tbnz w5, #0, LBB402_69
	ldr w8, [sp, #84]
	cmp w8, #84
	b.ne LBB402_155
LBB402_24:
	mov x0, x27
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #368]
	str wzr, [sp, #376]
	str x0, [sp, #384]
	mov x23, #33
	movk x23, #32768, lsl #48
	b LBB402_70
LBB402_25:
	sub x8, x26, #12
	stur x8, [x29, #-152]
	stp w28, w5, [x29, #-144]
LBB402_26:
	ldur q0, [x11, #168]
	ldur q1, [x11, #184]
	stp q0, q1, [x19]
	ldur q0, [x11, #200]
	str q0, [x19, #32]
	b LBB402_175
LBB402_27:
	mov x0, x23
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x24, x0
	cbz x22, LBB402_30
	sub x8, x22, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs LBB402_30
	ldr x9, [x21, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB402_30:
	sub x0, x29, #152
	mov x1, x21
	mov x2, x23
	mov x3, x24
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x26
	b.ne LBB402_171
	tbz w5, #0, LBB402_83
	ldr w8, [x21, #1640]
	add w8, w8, #1
	str w8, [x21, #1640]
	cbz x22, LBB402_116
	sub x8, x22, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs LBB402_116
LBB402_34:
	ldr x9, [x21, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x8]
	str q0, [x20]
	ldp q0, q1, [x8, #48]
	ldp q3, q2, [x8, #16]
	stp q0, q1, [x20, #48]
	stp q3, q2, [x20, #16]
	b LBB402_116
LBB402_35:
	mov x9, #30064771072
	movk x9, #32760, lsl #48
	cmp x8, x9
	ccmp w10, #0, #4, eq
	b.ne LBB402_61
LBB402_36:
	stp x25, x11, [sp, #64]
	str w23, [sp, #104]
	ldr x23, [sp, #120]
	stp x21, x23, [x29, #-152]
	str x26, [sp, #88]
	stp x26, x4, [x29, #-136]
	mov x26, x4
	mov x22, x12
	stp x12, x6, [x29, #-120]
	mov x28, x21
	mov x25, x6
	stur x20, [x29, #-104]
	sub x0, x29, #200
	sub x1, x29, #152
	mov x2, x10
	mov w3, #1
	bl lyng_ops::object::conversions::to_primitive
	ldp x8, x5, [x29, #-200]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_47
	ldr w3, [x20, #4]
	ldr w4, [x20, #56]
	add x0, sp, #416
	mov x21, x28
	mov x1, x28
	mov x28, x23
	mov x2, x23
	bl lyng_vm::vm::values::<impl lyng_vm::vm::Vm>::value_to_property_key
	mov x9, #33
	movk x9, #32768, lsl #48
	ldr x8, [sp, #416]
	ldr w5, [sp, #424]
	cmp x8, x9
	b.ne LBB402_48
	ldr w10, [sp, #428]
	ldp w3, w23, [sp, #100]
	mov x6, x25
	ldp x25, x11, [sp, #64]
	mov x12, x22
	mov x4, x26
	ldr x26, [sp, #88]
	mov x8, #9221120237041090560
	cmp x24, x8
	ccmp w25, #0, #4, eq
	ccmp w27, #5, #0, ne
	b.ne LBB402_62
LBB402_39:
	cbz w5, LBB402_84
	cmp w5, #1
	b.ne LBB402_99
	mov x27, x4
	stp x6, x12, [sp, #56]
	mov x4, x3
	ldr w3, [x20, #4]
	ldp x0, x1, [x21, #96]
	mov x2, x28
	mov x5, x25
	mov x22, x10
	mov x6, x22
	mov x24, x11
	mov x7, x11
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_keyed_property_store_inline_cache
	and w8, w0, #0xff
	cmp w8, #2
	b.ne LBB402_105
	str x24, [sp, #16]
	mov w8, #1
	str x22, [sp, #40]
	stp w8, w22, [sp, #8]
	str x25, [sp]
	add x0, sp, #848
	mov x1, x21
	mov x2, x28
	mov x3, x26
	mov x4, x27
	ldp x6, x5, [sp, #56]
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::set_property_on_value
	sub x0, x29, #152
	add x5, sp, #848
	mov x1, x21
	mov x2, x28
	ldr x3, [sp, #112]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_171
	cmp w5, #2
	b.eq LBB402_116
	cmp w23, #2
	b.hs LBB402_151
	tbz w5, #0, LBB402_147
LBB402_46:
	mov x22, #33
	movk x22, #32768, lsl #48
	str x22, [sp, #896]
	b LBB402_149
LBB402_47:
	mov x21, x28
	mov x28, x23
	add x9, sp, #1040
	ldur q0, [x9, #136]
	ldur q1, [x9, #152]
	stp q0, q1, [sp, #432]
	str x5, [sp, #424]
LBB402_48:
	mov x9, #-9223372036854775808
	cmp x8, x9
	ldr x22, [sp, #112]
	b.ne LBB402_57
	cbnz w5, LBB402_57
	mov x23, #33
	movk x23, #32768, lsl #48
	ldr x24, [sp, #432]
	cbz x22, LBB402_53
	sub x8, x22, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs LBB402_53
	ldr x9, [x21, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB402_53:
	sub x0, x29, #152
	mov x1, x21
	mov x2, x28
	mov x3, x24
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x23
	b.ne LBB402_171
	tbz w5, #0, LBB402_83
	ldr w8, [x21, #1640]
	add w8, w8, #1
	str w8, [x21, #1640]
	cbz x22, LBB402_116
LBB402_56:
	ldr x8, [sp, #112]
	sub x8, x8, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.lo LBB402_34
	b LBB402_116
LBB402_57:
	lsr w9, w5, #8
	ldr w10, [sp, #428]
	ldr x24, [sp, #432]
	add x11, sp, #185
	ldur q0, [x11, #255]
	str q0, [sp, #1088]
	ldr x11, [sp, #456]
	b LBB402_173
LBB402_58:
	ldr w8, [sp, #104]
	cmp w8, #2
	b.hs LBB402_114
	mov x22, #33
	movk x22, #32768, lsl #48
	str x22, [sp, #128]
	sub x0, x29, #152
	add x5, sp, #128
	mov x1, x21
	mov x2, x26
LBB402_60:
	ldr x3, [sp, #112]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_171
	b LBB402_113
LBB402_61:
	mov w5, #2
	str w5, [sp, #424]
	str w10, [sp, #428]
	mov x8, #33
	movk x8, #32768, lsl #48
	str x8, [sp, #416]
	ldr x28, [sp, #120]
	mov x8, #9221120237041090560
	cmp x24, x8
	ccmp w25, #0, #4, eq
	ccmp w27, #5, #0, ne
	b.eq LBB402_39
LBB402_62:
	str x11, [sp, #16]
	add x0, sp, #1040
	stp w5, w10, [sp, #8]
	str x25, [sp]
	mov x1, x21
	mov x2, x28
	mov x3, x26
	mov x5, x12
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::set_property_on_value
	ldr x8, [sp, #1040]
	mov x22, #33
	movk x22, #32768, lsl #48
	cmp x8, x22
	b.ne LBB402_75
	cmp w23, #1
	b.hi LBB402_114
	ldrb w8, [sp, #1048]
	tbz w8, #0, LBB402_110
LBB402_65:
	stur x22, [x29, #-248]
	b LBB402_112
LBB402_66:
	sub x0, x29, #152
	add x5, sp, #176
	mov x1, x21
	mov x2, x26
	ldr x3, [sp, #112]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x28
	b.ne LBB402_171
	ldr x27, [sp, #120]
	tbz w5, #0, LBB402_116
	ldr w8, [sp, #104]
	cmp w8, #2
	b.hs LBB402_73
LBB402_69:
	mov x23, #33
	movk x23, #32768, lsl #48
	str x23, [sp, #368]
LBB402_70:
	sub x0, x29, #152
	add x5, sp, #368
	mov x1, x21
	mov x2, x27
	ldr x3, [sp, #112]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x23
	b.ne LBB402_171
	tbz w5, #0, LBB402_116
	ldp x9, x8, [sp, #24]
	cmp x8, x9
	b.eq LBB402_74
LBB402_73:
	sub x0, x29, #152
	mov x1, x27
	mov x2, x25
	bl lyng_vm::vm::activation_objects::<impl lyng_vm::vm::Vm>::sync_engine_array_length
	ldur x8, [x29, #-152]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_176
LBB402_74:
	ldr w2, [x20, #4]
	mov x0, x21
	mov x1, x27
	ldr w3, [sp, #100]
	mov x4, x25
	ldr x5, [sp, #40]
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_keyed_index_access
	ldr w8, [x20, #56]
	ldr w9, [sp, #108]
	adds w8, w8, w9
	b.lo LBB402_115
	b LBB402_129
LBB402_75:
	ldr w5, [sp, #1048]
	mov x9, #-9223372036854775808
	cmp x8, x9
	ldr x23, [sp, #112]
	b.ne LBB402_104
	cbnz w5, LBB402_104
	ldr x24, [sp, #1056]
	cbz x23, LBB402_80
	sub x8, x23, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs LBB402_80
	ldr x9, [x21, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB402_80:
	sub x0, x29, #152
	mov x1, x21
	mov x2, x28
	mov x3, x24
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_171
	tbz w5, #0, LBB402_83
	ldr w8, [x21, #1640]
	add w8, w8, #1
	str w8, [x21, #1640]
	cbnz x23, LBB402_56
	b LBB402_116
LBB402_83:
	mov w9, #0
	mov w5, #0
	mov x8, #-9223372036854775808
	b LBB402_174
LBB402_84:
	mov x22, x4
	stp x6, x12, [sp, #56]
	ldr w2, [x20, #4]
	mov x0, x21
	mov x1, x28
	mov x4, x25
	mov x5, x10
	mov x27, x10
	mov x6, x11
	mov x24, x11
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_keyed_dense_index_store_inline_cache_hit
	and w8, w0, #0xff
	cmp w8, #2
	b.ne LBB402_108
	str x22, [sp, #48]
	add x0, sp, #512
	mov x1, x21
	mov x2, x28
	mov x3, x25
	mov x4, x27
	mov x5, x24
	bl lyng_vm::vm::values::<impl lyng_vm::vm::Vm>::mapped_arguments_set
	ldr x8, [sp, #512]
	mov x22, #33
	movk x22, #32768, lsl #48
	add x9, x22, #1
	cmp x8, x9
	str x27, [sp, #40]
	b.ne LBB402_117
	str x24, [sp, #72]
	str x24, [sp, #8]
	add x0, sp, #560
	stp w25, w27, [sp]
	mov x1, x21
	mov x2, x28
	mov x3, x26
	ldp x4, x6, [sp, #48]
	ldr x5, [sp, #64]
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_set_typed_array_index
	sub x0, x29, #152
	add x5, sp, #560
	mov x1, x21
	mov x2, x28
	ldr x3, [sp, #112]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_171
	cmp w5, #3
	b.eq LBB402_116
	cmp w5, #2
	b.ne LBB402_142
	add x0, sp, #608
	mov x1, x28
	mov x2, x25
	ldr x3, [sp, #40]
	ldr x4, [sp, #72]
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_set_engine_array_index
	sub x0, x29, #152
	add x5, sp, #608
	mov x1, x21
	mov x2, x28
	ldr x3, [sp, #112]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_171
	cmp w5, #3
	b.eq LBB402_116
	cmp w5, #2
	b.ne LBB402_142
	add x0, sp, #656
	mov x1, x28
	mov x27, x25
	mov x2, x25
	ldr x3, [sp, #40]
	ldr x4, [sp, #72]
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_set_ordinary_index_data_property
	sub x0, x29, #152
	add x5, sp, #656
	mov x22, x21
	mov x1, x21
	mov x2, x28
	ldr x3, [sp, #112]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_171
	cmp w5, #3
	ldr x28, [sp, #120]
	b.eq LBB402_116
	mov x21, x22
	mov x25, x27
	cmp w5, #2
	b.ne LBB402_142
	ldp x5, x8, [sp, #64]
	str x8, [sp, #16]
	add x0, sp, #704
	ldp x8, x4, [sp, #40]
	stp wzr, w8, [sp, #8]
	str x25, [sp]
	mov x1, x21
	mov x2, x28
	mov x3, x26
	ldr x6, [sp, #56]
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::set_property_on_value
	sub x0, x29, #152
	add x5, sp, #704
	mov x1, x21
	mov x2, x28
	ldr x3, [sp, #112]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_171
	cmp w5, #2
	ldr x28, [sp, #120]
	b.eq LBB402_116
	mov x21, x22
	mov x25, x27
	cmp w23, #2
	b.hs LBB402_169
	mov w22, #0
	b LBB402_144
LBB402_99:
	mov x24, x3
	str x11, [sp, #16]
	add x0, sp, #944
	stp w5, w10, [sp, #8]
	str x25, [sp]
	mov x1, x21
	mov x2, x28
	mov x3, x26
	mov x5, x12
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::set_property_on_value
	sub x0, x29, #152
	add x5, sp, #944
	mov x1, x21
	mov x2, x28
	ldr x3, [sp, #112]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_171
	cmp w5, #2
	b.eq LBB402_116
	cmp w23, #2
	b.hs LBB402_141
	tbz w5, #0, LBB402_137
LBB402_103:
	mov x22, #33
	movk x22, #32768, lsl #48
	str x22, [sp, #992]
	b LBB402_139
LBB402_104:
	lsr w9, w5, #8
	ldr w10, [sp, #1052]
	add x11, sp, #1040
	ldur q0, [x11, #24]
	str q0, [sp, #1088]
	ldr x24, [sp, #1056]
	ldr x11, [sp, #1080]
	b LBB402_173
LBB402_105:
	cmp w23, #2
	b.hs LBB402_128
	tbz w8, #0, LBB402_124
LBB402_107:
	mov x22, #33
	movk x22, #32768, lsl #48
	str x22, [sp, #800]
	b LBB402_126
LBB402_108:
	cmp w23, #2
	b.hs LBB402_114
	mov x22, #33
	movk x22, #32768, lsl #48
	str x22, [sp, #464]
	sub x0, x29, #152
	add x5, sp, #464
	mov x1, x21
	mov x2, x28
	b LBB402_60
LBB402_110:
	ldr w8, [sp, #84]
	cmp w8, #84
	b.ne LBB402_121
LBB402_111:
	mov x0, x28
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	stur x8, [x29, #-248]
	stur wzr, [x29, #-240]
	stur x0, [x29, #-232]
LBB402_112:
	ldr x3, [sp, #112]
	sub x0, x29, #152
	sub x5, x29, #248
	mov x1, x21
	mov x2, x28
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_171
LBB402_113:
	tbz w5, #0, LBB402_116
LBB402_114:
	ldr w8, [x20, #56]
	ldr w9, [sp, #108]
	adds w8, w8, w9
	b.hs LBB402_129
LBB402_115:
	str w8, [x20, #56]
LBB402_116:
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
	b LBB402_175
LBB402_117:
	sub x0, x29, #152
	add x5, sp, #512
	mov x1, x21
	mov x2, x28
	ldr x3, [sp, #112]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_171
	tbz w5, #0, LBB402_116
	cmp w23, #2
	b.hs LBB402_169
	mov w22, #0
	b LBB402_165
LBB402_121:
	ldr x9, [x21, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB402_65
	ldr x9, [x21, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB402_65
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB402_65
	b LBB402_111
LBB402_124:
	ldr w8, [sp, #84]
	cmp w8, #84
	b.ne LBB402_134
LBB402_125:
	mov x0, x28
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #800]
	str wzr, [sp, #808]
	str x0, [sp, #816]
	mov x22, #33
	movk x22, #32768, lsl #48
LBB402_126:
	ldr x3, [sp, #112]
	sub x0, x29, #152
	add x5, sp, #800
	mov x1, x21
	mov x2, x28
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_171
	tbz w5, #0, LBB402_116
LBB402_128:
	ldr w1, [x20, #4]
	mov x0, x21
	ldr w2, [sp, #100]
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
	ldr w8, [x20, #56]
	ldr w9, [sp, #108]
	adds w8, w8, w9
	b.lo LBB402_115
LBB402_129:
Lloh1159:
	adrp x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGE
Lloh1160:
	add x0, x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGEOFF
Lloh1161:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGE
Lloh1162:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGEOFF
	mov w1, #41
	bl core::option::expect_failed
LBB402_130:
Lloh1163:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.45@PAGE
Lloh1164:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.45@PAGEOFF
	mov x0, x5
	mov x2, x1
	bl core::slice::index::slice_index_fail
LBB402_131:
	mov x21, x2
	sub x0, x29, #152
	mov x2, x8
	mov w3, #1
	mov x4, x28
	bl lyng_vm::vm::dispatch::decode_abc_operands_wide
	add x11, sp, #1040
	ldur x8, [x29, #-152]
	cmp x8, x26
	b.ne LBB402_26
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
LBB402_133:
Lloh1165:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.59@PAGE
Lloh1166:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.59@PAGEOFF
	mov x0, x9
	bl core::panicking::panic_bounds_check
LBB402_134:
	ldr x9, [x21, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB402_107
	ldr x9, [x21, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB402_107
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB402_107
	b LBB402_125
LBB402_137:
	ldr w8, [sp, #84]
	cmp w8, #84
	b.ne LBB402_152
LBB402_138:
	mov x0, x28
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #992]
	str wzr, [sp, #1000]
	str x0, [sp, #1008]
	mov x22, #33
	movk x22, #32768, lsl #48
LBB402_139:
	ldr x3, [sp, #112]
	sub x0, x29, #152
	add x5, sp, #992
	mov x1, x21
	mov x2, x28
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_171
	tbz w5, #0, LBB402_116
LBB402_141:
	ldr w1, [x20, #4]
	mov x0, x21
	mov x2, x24
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_keyed_generic_slow_path
	b LBB402_114
LBB402_142:
	cmp w23, #2
	b.hs LBB402_170
	mov w22, #1
LBB402_144:
	tbnz w5, #0, LBB402_165
	ldr w8, [sp, #84]
	cmp w8, #84
	b.ne LBB402_161
LBB402_146:
	mov x0, x28
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #752]
	str wzr, [sp, #760]
	str x0, [sp, #768]
	mov x27, #33
	movk x27, #32768, lsl #48
	b LBB402_166
LBB402_147:
	ldr w8, [sp, #84]
	cmp w8, #84
	b.ne LBB402_158
LBB402_148:
	mov x0, x28
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #896]
	str wzr, [sp, #904]
	str x0, [sp, #912]
	mov x22, #33
	movk x22, #32768, lsl #48
LBB402_149:
	ldr x3, [sp, #112]
	sub x0, x29, #152
	add x5, sp, #896
	mov x1, x21
	mov x2, x28
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x22
	b.ne LBB402_171
	tbz w5, #0, LBB402_116
LBB402_151:
	ldr w2, [x20, #4]
	mov x0, x21
	mov x1, x28
	ldr w3, [sp, #100]
	mov x4, x25
	ldr x5, [sp, #40]
	mov w6, #1
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_keyed_atom_slow_path
	b LBB402_114
LBB402_152:
	ldr x9, [x21, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB402_103
	ldr x9, [x21, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB402_103
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB402_103
	b LBB402_138
LBB402_155:
	ldr x9, [x21, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB402_69
	ldr x9, [x21, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB402_69
	ldrh w8, [x8, #340]
	tbnz w8, #0, LBB402_24
	b LBB402_69
LBB402_158:
	ldr x9, [x21, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB402_46
	ldr x9, [x21, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB402_46
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB402_46
	b LBB402_148
LBB402_161:
	mov x10, x21
	ldr x9, [x21, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB402_164
	ldr x9, [x10, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB402_164
	ldrh w8, [x8, #340]
	ldr x28, [sp, #120]
	mov x21, x10
	tbnz w8, #0, LBB402_146
	b LBB402_165
LBB402_164:
	ldr x28, [sp, #120]
	mov x21, x10
LBB402_165:
	mov x27, #33
	movk x27, #32768, lsl #48
	str x27, [sp, #752]
LBB402_166:
	ldr x3, [sp, #112]
	sub x0, x29, #152
	add x5, sp, #752
	mov x1, x21
	mov x2, x28
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	cmp x8, x27
	b.ne LBB402_171
	tbz w5, #0, LBB402_116
	tbnz w22, #0, LBB402_170
LBB402_169:
	sub x0, x29, #152
	mov x1, x28
	mov x2, x25
	bl lyng_vm::vm::activation_objects::<impl lyng_vm::vm::Vm>::sync_engine_array_length
	ldur x8, [x29, #-152]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB402_176
LBB402_170:
	ldr w2, [x20, #4]
	mov x0, x21
	mov x1, x28
	ldr w3, [sp, #100]
	mov x4, x25
	ldr x5, [sp, #40]
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_keyed_index_access
	b LBB402_114
LBB402_171:
	ldurb w9, [x29, #-141]
	add x11, sp, #1040
	ldurh w10, [x11, #177]
	orr w9, w10, w9, lsl #16
	ldur w10, [x29, #-140]
LBB402_172:
	ldur q0, [x11, #192]
	str q0, [sp, #1088]
	ldur x24, [x29, #-136]
	ldur x11, [x29, #-112]
LBB402_173:
	str x11, [sp, #1104]
LBB402_174:
	bfi w5, w9, #8, #24
	str x8, [x19]
	stp w5, w10, [x19, #8]
	str x24, [x19, #16]
	ldr q0, [sp, #1088]
	stur q0, [x19, #24]
	ldr x8, [sp, #1104]
	str x8, [x19, #40]
LBB402_175:
	add sp, sp, #1280
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
LBB402_176:
	.cfi_restore_state
	ldp w5, w10, [x29, #-144]
	lsr w9, w5, #8
	add x11, sp, #1040
	b LBB402_172
LBB402_177:
Lloh1167:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.59@PAGE
Lloh1168:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.59@PAGEOFF
	bl core::panicking::panic_bounds_check
LBB402_178:
Lloh1169:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.59@PAGE
Lloh1170:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.59@PAGEOFF
	mov x0, x8
	bl core::panicking::panic_bounds_check
	.loh AdrpAdd	Lloh1157, Lloh1158
	.loh AdrpAdd	Lloh1161, Lloh1162
	.loh AdrpAdd	Lloh1159, Lloh1160
	.loh AdrpAdd	Lloh1163, Lloh1164
	.loh AdrpAdd	Lloh1165, Lloh1166
	.loh AdrpAdd	Lloh1167, Lloh1168
	.loh AdrpAdd	Lloh1169, Lloh1170
