lyng_vm::vm::dispatch_handlers::property::op_get_keyed_property:
Lfunc_begin389:
	.cfi_startproc
	.cfi_personality 155, _rust_eh_personality
	.cfi_lsda 16, Lexception191
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
	sub sp, sp, #560
	mov x19, x8
	ldr w2, [x0, #4]
	ldr w5, [x0, #56]
	ldrb w9, [x0, #148]
	mov w8, #152
	strb w8, [x0, #148]
	ldr x10, [x0, #128]
	ldr x1, [x10, #56]
	subs x8, x1, x5
	b.lo LBB389_99
	mov x20, x0
	sub x28, x29, #200
	mov x22, #33
	movk x22, #32768, lsl #48
	ldr x10, [x10, #48]
	add x1, x10, x5
	cmp w9, #152
	b.ne LBB389_100
	and x9, x8, #0x7ffffffffffffffe
	cmp x8, #4
	ccmp x9, #4, #4, hs
	b.eq LBB389_14
	ldrh w23, [x1, #4]
	cbz w23, LBB389_14
	ldrb w12, [x1, #1]
	ldrb w8, [x1, #2]
	mov w9, #6
	str w9, [sp, #60]
	ldrb w9, [x1, #3]
	ldr x21, [x20, #80]
	ldr w10, [x20, #20]
	ldr x1, [x21, #32]
	add w0, w10, w8
	cmp x1, x0
	b.ls LBB389_102
LBB389_5:
	add w8, w10, w9
	cmp x1, x8
	b.ls LBB389_107
	ldr x13, [x20, #88]
	ldr x9, [x20, #136]
	str x9, [sp, #72]
	ldr x9, [x21, #24]
	ldr x25, [x9, x0, lsl #3]
	and x22, x25, #0x7ff8000000000000
	ubfx x24, x25, #32, #16
	sub w11, w24, #1
	mov x10, #9221120237041090560
	cmp x22, x10
	ccmp w11, #1, #2, eq
	b.ls LBB389_16
	str x13, [sp, #64]
	str w12, [sp, #20]
	ldr x11, [x20, #96]
	str x11, [sp, #24]
	ldr x11, [x20, #104]
	str x11, [sp, #32]
	ldr x11, [x20, #112]
	str x11, [sp, #40]
	ldr x11, [x20, #120]
	str x11, [sp, #48]
	ldr x27, [x9, x8, lsl #3]
	and x8, x27, #0x7fffffff00000000
	and x8, x8, #0xfff8ffffffffffff
	cmp x22, x10
	ccmp w24, #5, #0, eq
	ccmp w25, #0, #4, eq
	mov x9, #17179869184
	movk x9, #32760, lsl #48
	ccmp x8, x9, #0, ne
	b.ne LBB389_26
	tbnz w27, #31, LBB389_27
	mov x0, x21
	ldr x1, [sp, #64]
	mov x3, x23
	mov x4, x25
	mov x5, x27
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_keyed_dense_index_load_inline_cache_hit
	tbz w0, #0, LBB389_57
LBB389_11:
	mov x8, x1
	ldr w9, [x20, #20]
	ldr x1, [x21, #32]
	ldr w10, [sp, #20]
	add w0, w9, w10
	cmp x1, x0
	b.ls LBB389_110
	ldr x9, [x21, #24]
	str x8, [x9, x0, lsl #3]
	ldr w8, [x20, #56]
	ldr w9, [sp, #60]
	adds w8, w8, w9
	b.hs LBB389_83
LBB389_13:
	str w8, [x20, #56]
	b LBB389_93
LBB389_14:
	sub x8, x22, #12
	stur x8, [x29, #-152]
	stp w2, w5, [x29, #-144]
LBB389_15:
	ldp q0, q1, [x28, #48]
	stp q0, q1, [x19]
	ldr q0, [x28, #80]
	str q0, [x19, #32]
	b LBB389_94
LBB389_16:
	mov x22, x13
	mov x0, x13
	mov w1, #5
	bl lyng_ops::errors::error_value
	mov x26, x0
	ldr x8, [sp, #72]
	cbz x8, LBB389_20
	ldr x8, [sp, #72]
	sub x8, x8, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs LBB389_20
	ldr x9, [x21, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB389_20:
	sub x0, x29, #152
	mov x1, x21
	mov x2, x22
	mov x3, x26
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_48
	tbz w5, #0, LBB389_47
LBB389_23:
	ldr w8, [x21, #1640]
	add w8, w8, #1
	str w8, [x21, #1640]
	ldr x8, [sp, #72]
	cbz x8, LBB389_93
	ldr x8, [sp, #72]
	sub x8, x8, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs LBB389_93
	ldr x9, [x21, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x8]
	str q0, [x20]
	ldp q0, q1, [x8, #48]
	ldp q3, q2, [x8, #16]
	stp q0, q1, [x20, #48]
	stp q3, q2, [x20, #16]
	b LBB389_93
LBB389_26:
	mov x9, #30064771072
	movk x9, #32760, lsl #48
	cmp x8, x9
	ccmp w27, #0, #4, eq
	b.ne LBB389_32
LBB389_27:
	ldr x8, [sp, #64]
	stp x21, x8, [x29, #-152]
	ldp x9, x8, [sp, #24]
	stp x9, x8, [x29, #-136]
	ldp x9, x8, [sp, #40]
	stp x9, x8, [x29, #-120]
	stur x20, [x29, #-104]
	sub x0, x29, #200
	sub x1, x29, #152
	mov x2, x27
	mov w3, #1
	bl lyng_ops::object::conversions::to_primitive
	ldp x8, x5, [x29, #-200]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_38
	ldr w3, [x20, #4]
	ldr w4, [x20, #56]
	add x0, sp, #128
	mov x1, x21
	ldr x2, [sp, #64]
	bl lyng_vm::vm::values::<impl lyng_vm::vm::Vm>::value_to_property_key
	ldr x8, [sp, #128]
	ldr w5, [sp, #136]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_39
	ldr w27, [sp, #140]
	b LBB389_33
LBB389_32:
	mov w5, #2
LBB389_33:
	mov x8, #9221120237041090560
	cmp x22, x8
	ccmp w25, #0, #4, eq
	ccmp w24, #5, #0, ne
	b.eq LBB389_49
	stp w5, w27, [sp, #8]
	sub x0, x29, #248
	str x25, [sp]
	mov x1, x21
	ldr x2, [sp, #64]
	ldp x3, x4, [sp, #24]
	ldp x5, x6, [sp, #40]
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::get_property_from_value
	sub x0, x29, #152
	sub x5, x29, #248
	mov x1, x21
	ldp x2, x3, [sp, #64]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldp x8, x5, [x29, #-152]
	ldur x26, [x29, #-136]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_106
	tbnz w5, #0, LBB389_81
	b LBB389_93
LBB389_38:
	ldp q0, q1, [x28, #16]
	stp q0, q1, [sp, #144]
	str x5, [sp, #136]
LBB389_39:
	mov x9, #-9223372036854775808
	cmp x8, x9
	b.ne LBB389_54
	cbnz w5, LBB389_54
	ldr x26, [sp, #144]
	ldr x8, [sp, #72]
	cbz x8, LBB389_44
	ldr x8, [sp, #72]
	sub x8, x8, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs LBB389_44
	ldr x9, [x21, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB389_44:
	sub x0, x29, #152
	mov x1, x21
	ldr x2, [sp, #64]
	mov x3, x26
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-152]
	ldurb w5, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_48
	tbnz w5, #0, LBB389_23
LBB389_47:
	mov w9, #0
	mov w5, #0
	mov x8, #-9223372036854775808
	b LBB389_56
LBB389_48:
	ldurb w9, [x29, #-141]
	ldurh w10, [x28, #57]
	orr w9, w10, w9, lsl #16
	ldur w10, [x29, #-140]
	ldur q0, [x28, #72]
	str q0, [sp, #176]
	ldur x26, [x29, #-136]
	ldur x11, [x29, #-112]
	b LBB389_55
LBB389_49:
	cbz w5, LBB389_65
	cmp w5, #1
	b.ne LBB389_72
	ldr w3, [x20, #4]
	ldp x0, x1, [x21, #96]
	ldr x2, [sp, #64]
	mov x4, x23
	mov x5, x25
	mov x6, x27
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_keyed_property_load_inline_cache
	tbz w0, #0, LBB389_85
	mov x26, x1
	ldr w1, [x20, #4]
	mov x0, x21
	mov x2, x23
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
	b LBB389_81
LBB389_54:
	lsr w9, w5, #8
	ldr w10, [sp, #140]
	ldr x26, [sp, #144]
	ldur q0, [sp, #152]
	str q0, [sp, #176]
	ldr x11, [sp, #168]
LBB389_55:
	str x11, [sp, #192]
LBB389_56:
	lsl w9, w9, #8
	bfxil x9, x5, #0, #8
	orr x9, x9, x10, lsl #32
	stp x8, x9, [x19]
	str x26, [x19, #16]
	ldr q0, [sp, #176]
	stur q0, [x19, #24]
	ldr x8, [sp, #192]
	str x8, [x19, #40]
	b LBB389_94
LBB389_57:
	add x0, sp, #80
	mov x1, x21
	ldr x2, [sp, #64]
	mov x3, x25
	mov x4, x27
	bl lyng_vm::vm::values::<impl lyng_vm::vm::Vm>::mapped_arguments_get
	ldr x8, [sp, #80]
	mov x9, #33
	movk x9, #32768, lsl #48
	add x9, x9, #1
	cmp x8, x9
	b.ne LBB389_62
	ldr x0, [sp, #64]
	mov x1, x25
	mov x2, x27
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_typed_array_index_value
	tbz w0, #0, LBB389_77
	mov x26, x1
	b LBB389_80
LBB389_62:
	sub x0, x29, #152
	add x5, sp, #80
	mov x1, x21
	ldp x2, x3, [sp, #64]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldp x8, x5, [x29, #-152]
	ldur x26, [x29, #-136]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_84
	tbnz w5, #0, LBB389_80
	b LBB389_93
LBB389_65:
	ldr w2, [x20, #4]
	mov x0, x21
	ldr x1, [sp, #64]
	mov x3, x23
	mov x4, x25
	mov x5, x27
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_keyed_dense_index_load_inline_cache_hit
	tbnz w0, #0, LBB389_11
	add x0, sp, #200
	mov x1, x21
	ldr x2, [sp, #64]
	mov x3, x25
	mov x4, x27
	bl lyng_vm::vm::values::<impl lyng_vm::vm::Vm>::mapped_arguments_get
	ldr x8, [sp, #200]
	mov x9, #33
	movk x9, #32768, lsl #48
	add x9, x9, #1
	cmp x8, x9
	b.ne LBB389_90
	ldr x0, [sp, #64]
	mov x1, x25
	mov x2, x27
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_typed_array_index_value
	tbz w0, #0, LBB389_95
	mov x26, x1
	b LBB389_98
LBB389_72:
	stp w5, w27, [sp, #8]
	add x0, sp, #344
	str x25, [sp]
	mov x1, x21
	ldr x2, [sp, #64]
	ldp x3, x4, [sp, #24]
	ldp x5, x6, [sp, #40]
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::get_property_from_value
	sub x0, x29, #152
	add x5, sp, #344
	mov x1, x21
	ldp x2, x3, [sp, #64]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldp x8, x5, [x29, #-152]
	ldur x26, [x29, #-136]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_106
	tbz w5, #0, LBB389_93
	ldr w1, [x20, #4]
	mov x0, x21
	mov x2, x23
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_keyed_generic_slow_path
	b LBB389_81
LBB389_77:
	sub x0, x29, #152
	ldr x1, [sp, #64]
	mov x2, x25
	mov x3, x27
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_own_index_value
	ldp x8, x5, [x29, #-152]
	ldur x26, [x29, #-136]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_106
	tbz w5, #0, LBB389_27
LBB389_80:
	ldr w2, [x20, #4]
	mov x0, x21
	ldr x1, [sp, #64]
	mov x3, x23
	mov x4, x25
	mov x5, x27
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_keyed_index_access
LBB389_81:
	ldr w8, [x20, #20]
	ldr x1, [x21, #32]
	ldr w9, [sp, #20]
	add w0, w8, w9
	cmp x1, x0
	b.ls LBB389_110
	ldr x8, [x21, #24]
	str x26, [x8, x0, lsl #3]
	ldr w8, [x20, #56]
	ldr w9, [sp, #60]
	adds w8, w8, w9
	b.lo LBB389_13
LBB389_83:
Lloh1063:
	adrp x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGE
Lloh1064:
	add x0, x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGEOFF
Lloh1065:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGE
Lloh1066:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGEOFF
	mov w1, #41
	bl core::option::expect_failed
	b LBB389_109
LBB389_84:
	ldur q0, [x28, #72]
	str q0, [sp, #176]
	ldur x9, [x29, #-112]
	str x9, [sp, #192]
	lsr w9, w5, #8
	lsr x10, x5, #32
	b LBB389_56
LBB389_85:
	mov w8, #1
	stp w8, w27, [sp, #8]
	str x25, [sp]
	add x0, sp, #296
	mov x1, x21
	ldr x2, [sp, #64]
	ldp x3, x4, [sp, #24]
	ldp x5, x6, [sp, #40]
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::get_property_from_value
	sub x0, x29, #152
	add x5, sp, #296
	mov x1, x21
	ldp x2, x3, [sp, #64]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldp x8, x5, [x29, #-152]
	ldur x26, [x29, #-136]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_106
	tbz w5, #0, LBB389_93
	ldr w2, [x20, #4]
	mov x0, x21
	ldr x1, [sp, #64]
	mov x3, x23
	mov x4, x25
	mov x5, x27
	mov w6, #0
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_keyed_atom_slow_path
	b LBB389_81
LBB389_90:
	sub x0, x29, #152
	add x5, sp, #200
	mov x1, x21
	ldp x2, x3, [sp, #64]
	mov x4, x20
	bl lyng_vm::vm::dispatch::<impl lyng_vm::vm::Vm>::handle_dispatch_result
	ldp x8, x5, [x29, #-152]
	ldur x26, [x29, #-136]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_106
LBB389_92:
	tbnz w5, #0, LBB389_98
LBB389_93:
	ldr x8, [x20, #128]
	ldr x8, [x8, #48]
	ldr w9, [x20, #56]
	ldrb w8, [x8, x9]
Lloh1067:
	adrp x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
Lloh1068:
	add x9, x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x10, [x9, x8, lsl #3]
	mov x8, #33
	movk x8, #32768, lsl #48
	stp x8, x10, [x19]
LBB389_94:
	add sp, sp, #560
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
LBB389_95:
	.cfi_restore_state
	sub x0, x29, #152
	ldr x1, [sp, #64]
	mov x2, x25
	mov x3, x27
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::try_fast_own_index_value
	ldp x8, x5, [x29, #-152]
	ldur x26, [x29, #-136]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_106
	tbz w5, #0, LBB389_103
LBB389_98:
	ldr w2, [x20, #4]
	mov x0, x21
	ldr x1, [sp, #64]
	mov x3, x23
	mov x4, x25
