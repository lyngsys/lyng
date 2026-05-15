lyng_js_vm::vm::dispatch_handlers::property::op_set_named_property_common:
Lfunc_begin403:
	.cfi_startproc
	sub sp, sp, #448
	.cfi_def_cfa_offset 448
	stp x28, x27, [sp, #352]
	stp x26, x25, [sp, #368]
	stp x24, x23, [sp, #384]
	stp x22, x21, [sp, #400]
	stp x20, x19, [sp, #416]
	stp x29, x30, [sp, #432]
	add x29, sp, #432
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
	ldr w23, [x1, #4]
	ldr w5, [x1, #56]
	ldrb w10, [x1, #148]
	mov w8, #152
	strb w8, [x1, #148]
	ldr x11, [x1, #128]
	ldr x8, [x11, #56]
	subs x9, x8, x5
	b.lo LBB403_71
	sub x28, x29, #152
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
	ldrh w24, [x8, #4]
	cbz w24, LBB403_17
	ldrb w9, [x8, #1]
	ldrb w12, [x8, #2]
	mov w20, #6
	ldrb w11, [x8, #3]
	ldr x19, [x1, #80]
	ldr w8, [x1, #20]
	ldr x10, [x19, #32]
	add w9, w8, w9
	cmp x10, x9
	b.ls LBB403_74
LBB403_5:
	add w8, w8, w12
	cmp x10, x8
	b.ls LBB403_157
	ldr x10, [x19, #80]
	sub w13, w23, #1
	cmp x10, x13
	b.ls LBB403_19
	ldr x12, [x19, #72]
	ldr x14, [x12, x13, lsl #3]
	cbz x14, LBB403_19
	ldr x16, [x14, #80]
	mov w15, w11
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
	ldr x15, [x1, #88]
	ldr x21, [x1, #136]
	ldr x10, [x19, #24]
	ldr x5, [x10, x9, lsl #3]
	ldr x9, [x14, #464]
	cmp x9, x26
	b.ls LBB403_14
	ldr x9, [x14, #456]
	add x9, x9, x26, lsl #3
	ldr w11, [x9, #16]!
	cmp w11, #1
	b.ne LBB403_14
	ldr w26, [x9, #4]
LBB403_14:
	ldp x25, x3, [x1, #96]
	ldp x17, x16, [x1, #112]
	ldr x6, [x10, x8, lsl #3]
	and x8, x5, #0x7ff8000000000000
	lsr x11, x5, #32
	sub w10, w11, #1
	mov x9, #9221120237041090560
	cmp x8, x9
	and w10, w10, #0xffff
	ccmp w10, #9, #2, eq
	stp x15, x1, [sp, #80]
	b.ls LBB403_27
LBB403_15:
	cmp x8, x9
	ccmp w10, #2, #2, eq
	b.hs LBB403_28
	mov x20, x0
	mov x0, x15
	mov w1, #5
	bl lyng_js_ops::errors::error_value
	mov x8, x0
	mov x0, x20
	mov w9, #0
	str x8, [sp, #224]
	mov x10, #-9223372036854775808
	b LBB403_37
LBB403_17:
	sub x8, x27, #12
	stur x8, [x29, #-152]
	stp w23, w5, [x29, #-144]
LBB403_18:
	ldp q0, q1, [x28]
	stp q0, q1, [x0]
	ldr q0, [x28, #32]
	b LBB403_24
LBB403_19:
	sub x10, x27, #29
	mov x9, x23
	b LBB403_21
LBB403_20:
	mov x9, #0
LBB403_21:
LBB403_22:
	lsr x12, x9, #32
	lsr w8, w9, #8
LBB403_23:
	bfi w9, w8, #8, #24
	str x10, [x0]
	stp w9, w12, [x0, #8]
	str x24, [x0, #16]
	stp w23, w11, [x0, #24]
	ldur q0, [x29, #-176]
LBB403_24:
	str q0, [x0, #32]
LBB403_25:
	.cfi_def_cfa wsp, 448
	ldp x29, x30, [sp, #432]
	ldp x20, x19, [sp, #416]
	ldp x22, x21, [sp, #400]
	ldp x24, x23, [sp, #384]
	ldp x26, x25, [sp, #368]
	ldp x28, x27, [sp, #352]
	add sp, sp, #448
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
	ldr x24, [x16, #8]
	orr x9, x15, x26, lsl #32
	b LBB403_22
LBB403_27:
	and w11, w11, #0xffff
	mov w14, #1
	lsl w11, w14, w11
	mov w14, #2008
	tst w11, w14
	b.eq LBB403_50
LBB403_28:
	mov x23, x3
	mov x24, x17
	stp x26, x6, [sp, #48]
	mov x26, x16
	stp x21, x0, [sp, #64]
	str w2, [sp, #44]
	ldr w1, [x1, #8]
	sub x8, x29, #152
	mov x0, x15
	mov x2, x5
	mov x22, x15
	mov x21, x5
	bl lyng_js_ops::object::primitive_wrappers::to_object
	ldur w9, [x29, #-152]
	cmp w9, #4
	b.ne LBB403_34
	ldur w2, [x29, #-148]
	stp x19, x22, [x29, #-152]
	stp x25, x23, [x29, #-136]
	stp x24, x26, [x29, #-120]
	ldr x8, [sp, #88]
	stur x8, [x29, #-104]
	add x0, sp, #208
	sub x1, x29, #152
	mov w3, #1
	ldp x4, x5, [sp, #48]
	mov x6, x21
	bl lyng_js_ops::proxy::set
	ldr x10, [sp, #208]
	cmp x10, x27
	b.ne LBB403_35
	ldr w23, [sp, #44]
	and w8, w23, #0xff
	cmp w8, #79
	ldr x0, [sp, #72]
	b.lo LBB403_32
	ldrb w8, [sp, #216]
	tbz w8, #0, LBB403_62
LBB403_32:
	ldr x8, [sp, #88]
	ldr w9, [x8, #56]
	adds w9, w9, w20
	b.hs LBB403_156
	str w9, [x8, #56]
	b LBB403_93
LBB403_34:
	sub x8, x29, #152
	orr x8, x8, #0x4
	ldr x10, [x8]
	stur x10, [sp, #220]
	ldr w8, [x8, #8]
	str w8, [sp, #228]
	mov x10, #-9223372036854775808
	b LBB403_36
LBB403_35:
	ldr w9, [sp, #216]
LBB403_36:
	ldp x21, x0, [sp, #64]
LBB403_37:
	mov x8, #-9223372036854775808
	cmp x10, x8
	b.ne LBB403_48
	cbnz w9, LBB403_48
	mov x20, x0
	ldr x24, [sp, #224]
	cbz x21, LBB403_42
	sub x8, x21, #1
	ldr x9, [x19, #56]
	cmp x8, x9
	b.hs LBB403_42
	ldr x9, [x19, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr x9, [sp, #88]
	ldr q0, [x9]
	str q0, [x8]
	ldp q0, q1, [x9, #48]
	ldp q3, q2, [x9, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB403_42:
	sub x0, x29, #152
	mov x1, x19
	ldr x2, [sp, #80]
	mov x3, x24
	bl lyng_js_vm::vm::exceptions::<impl lyng_js_vm::vm::Vm>::transfer_to_exception_handler
	ldur x10, [x29, #-152]
	ldurb w9, [x29, #-144]
	cmp x10, x27
	b.ne LBB403_49
	mov x0, x20
	tbz w9, #0, LBB403_70
	ldr w8, [x19, #1640]
	add w8, w8, #1
	str w8, [x19, #1640]
	ldr x8, [sp, #88]
	cbz x21, LBB403_93
	sub x9, x21, #1
LBB403_46:
	ldr x10, [x19, #56]
	cmp x9, x10
	b.hs LBB403_93
	ldr x10, [x19, #48]
	mov w11, #80
	madd x9, x9, x11, x10
	ldr q0, [x9]
	str q0, [x8]
	ldp q0, q1, [x9, #48]
	ldp q3, q2, [x9, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
	b LBB403_93
LBB403_48:
	lsr w8, w9, #8
	ldr w12, [sp, #220]
	ldr x24, [sp, #224]
	ldp w23, w11, [sp, #232]
	ldr q0, [sp, #240]
	stur q0, [x29, #-176]
	b LBB403_23
LBB403_49:
	ldurb w8, [x29, #-141]
	ldurh w11, [x28, #9]
	orr w8, w11, w8, lsl #16
	ldur w12, [x29, #-140]
	ldur x24, [x29, #-136]
	ldp w23, w11, [x29, #-128]
	ldr q0, [x28, #32]
	stur q0, [x29, #-176]
	mov x0, x20
	b LBB403_23
LBB403_50:
	tst w11, #0x6
	b.ne LBB403_15
	cbz w5, LBB403_28
	stp x3, x17, [sp, #24]
	str x16, [sp, #48]
	str w2, [sp, #44]
	str x0, [sp, #72]
	ldp x0, x1, [x19, #96]
	cbz w24, LBB403_85
	cmp x1, x13
	b.ls LBB403_85
	add x9, x0, x13, lsl #5
	ldr x10, [x9, #16]
	sub w8, w24, #1
	cmp x10, x8
	b.ls LBB403_85
	ldr x9, [x9, #8]
	mov w10, #1096
	umaddl x9, w8, w10, x9
	ldr x8, [x9]
	cmp x8, #10
	b.eq LBB403_85
	cmp x8, #6
	b.ne LBB403_85
	ldr x8, [x9, #968]
	cbz x8, LBB403_85
	ldr x13, [x15, #224]
	mov w10, #-1
	add x10, x5, x10
	lsr w11, w10, #6
	cmp x11, x13
	b.hs LBB403_85
	ldr x13, [x15, #216]
	and x10, x10, #0x3f
	ldr x11, [x13, x11, lsl #3]
	mov w13, #80
	umaddl x11, w10, w13, x11
	ldr w10, [x11]
	cmp w10, #1
	b.ne LBB403_85
	mov x3, x25
	mov x17, x26
	mov x16, x5
	mov x2, x6
	mov x22, x15
	str x21, [sp, #64]
	ldr x10, [x9, #976]
	ldp w13, w9, [x11, #52]
	ldr x11, [x11, #40]
	lsr x14, x8, #32
	cmp x14, #0
	csel w14, w14, wzr, ne
	cbz w13, LBB403_78
	cmp w13, w14
	ccmp x11, x10, #0, eq
	ldr x21, [sp, #64]
	mov x15, x22
	mov x6, x2
	mov x5, x16
	mov x26, x17
	mov x25, x3
	b.eq LBB403_80
	b LBB403_85
LBB403_62:
	and w8, w23, #0xff
	cmp w8, #80
	b.ne LBB403_75
LBB403_63:
	ldr x0, [sp, #80]
	mov w1, #5
	bl lyng_js_ops::errors::error_value
	mov x24, x0
	ldr x8, [sp, #64]
	cbz x8, LBB403_66
	ldr x8, [sp, #64]
	sub x8, x8, #1
	ldr x9, [x19, #56]
	cmp x8, x9
	b.hs LBB403_66
	ldr x9, [x19, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr x9, [sp, #88]
	ldr q0, [x9]
	str q0, [x8]
	ldp q0, q1, [x9, #48]
	ldp q3, q2, [x9, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB403_66:
	sub x0, x29, #152
	mov x1, x19
	ldr x2, [sp, #80]
	mov x3, x24
	bl lyng_js_vm::vm::exceptions::<impl lyng_js_vm::vm::Vm>::transfer_to_exception_handler
	ldur x10, [x29, #-152]
	ldurb w9, [x29, #-144]
	cmp x10, x27
	b.ne LBB403_146
	ldr x0, [sp, #72]
	tbz w9, #0, LBB403_70
	ldr w8, [x19, #1640]
	add w8, w8, #1
	str w8, [x19, #1640]
	ldr x8, [sp, #88]
	ldr x22, [sp, #64]
	cbz x22, LBB403_93
	sub x9, x22, #1
	b LBB403_46
LBB403_70:
	mov w9, #0
	mov w8, #0
	mov x10, #-9223372036854775808
	b LBB403_23
LBB403_71:
Lloh1173:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.45@PAGE
Lloh1174:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.45@PAGEOFF
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
	mov x4, x23
	bl lyng_js_vm::vm::dispatch::decode_abc_operands_wide
	mov x0, x20
	ldur x8, [x29, #-152]
	cmp x8, x27
	b.ne LBB403_18
	ldurh w9, [x29, #-140]
	ldurh w12, [x29, #-138]
	ldurh w11, [x29, #-136]
	ldur w24, [x29, #-144]
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
Lloh1175:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.760@PAGE
Lloh1176:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.760@PAGEOFF
	mov x0, x9
	mov x1, x10
	bl core::panicking::panic_bounds_check
LBB403_75:
	ldr x9, [x19, #80]
	ldr x8, [sp, #88]
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
	ldr x21, [sp, #64]
	mov x15, x22
	mov x6, x2
	mov x5, x16
	mov x26, x17
	mov x25, x3
	cbnz w14, LBB403_85
	cmp x11, x10
	b.ne LBB403_85
LBB403_80:
	tbnz w8, #30, LBB403_82
	ldr w8, [sp, #44]
	and w8, w8, #0xff
	cmp w8, #78
	b.hi LBB403_138
	b LBB403_154
LBB403_82:
	tbnz w8, #31, LBB403_134
	ldr x21, [sp, #64]
	mov x15, x22
	mov x6, x2
	mov x5, x16
	mov x26, x17
	mov x25, x3
	cbz w9, LBB403_85
	mov w10, #2
	mov x16, x9
	b LBB403_135
LBB403_85:
	mov x2, x15
	mov x3, x23
	mov x4, x24
	str x6, [sp, #56]
	mov x23, x5
	mov x22, x15
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::try_named_property_store_inline_cache
	and w8, w0, #0xff
	cmp w8, #2
	b.ne LBB403_89
	str x21, [sp, #64]
	mov x0, x22
	mov x1, x23
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::prototype_chain_has_proxy
	tbz w0, #0, LBB403_94
	ldr x8, [sp, #88]
	ldr w1, [x8, #8]
	sub x8, x29, #152
	mov x21, x22
	mov x0, x22
	mov x2, x23
	bl lyng_js_ops::object::primitive_wrappers::to_object
	ldur w9, [x29, #-152]
	cmp w9, #4
	b.ne LBB403_96
	ldur w2, [x29, #-148]
	stp x19, x21, [x29, #-152]
	ldp x9, x10, [sp, #24]
	stp x25, x9, [x29, #-136]
	ldp x8, x5, [sp, #48]
	stp x10, x8, [x29, #-120]
	ldr x8, [sp, #88]
	stur x8, [x29, #-104]
	add x0, sp, #96
	sub x1, x29, #152
	mov w3, #1
	mov x4, x26
	mov x22, x23
	mov x6, x23
	bl lyng_js_ops::proxy::set
	b LBB403_106
LBB403_89:
	ldr w9, [sp, #44]
	and w11, w9, #0xff
	ldr x10, [sp, #88]
	ldr w1, [x10, #4]
	cmp w11, #79
	b.lo LBB403_91
	tbz w8, #0, LBB403_97
LBB403_91:
	mov x0, x19
	mov x2, x24
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::record_feedback_slot
	ldr x8, [sp, #88]
	ldr w9, [x8, #56]
	adds w9, w9, w20
	b.hs LBB403_156
LBB403_92:
	str w9, [x8, #56]
	ldr x0, [sp, #72]
LBB403_93:
	ldr x9, [x8, #128]
	ldr x9, [x9, #48]
	ldr w8, [x8, #56]
	ldrb w8, [x9, x8]
Lloh1177:
	adrp x9, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
Lloh1178:
	add x9, x9, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x8, [x9, x8, lsl #3]
	stp x27, x8, [x0]
	b LBB403_25
LBB403_94:
	sub x8, x29, #152
	mov x21, x22
	mov x0, x22
	mov x22, x23
	mov x1, x23
	mov w2, #1
	mov x3, x26
	ldr x23, [sp, #56]
	mov x4, x23
	mov w5, #0
	bl lyng_js_ops::object::ordinary::ordinary_set
	ldur w8, [x29, #-152]
	cmp w8, #4
	b.ne LBB403_105
	ldurb w8, [x29, #-148]
	strb w8, [sp, #104]
	ldr w8, [sp, #44]
	b LBB403_107
LBB403_96:
	sub x8, x29, #152
	orr x8, x8, #0x4
	ldr x10, [x8]
	stur x10, [sp, #108]
	ldr w8, [x8, #8]
	str w8, [sp, #116]
	mov x10, #-9223372036854775808
	b LBB403_110
LBB403_97:
	and w8, w9, #0xff
	cmp w8, #80
	b.ne LBB403_122
LBB403_98:
	mov x0, x22
	mov w1, #5
	bl lyng_js_ops::errors::error_value
	mov x24, x0
	mov x20, x21
	cbz x21, LBB403_101
	sub x8, x20, #1
	ldr x9, [x19, #56]
	cmp x8, x9
	b.hs LBB403_101
	ldr x9, [x19, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr x9, [sp, #88]
	ldr q0, [x9]
	str q0, [x8]
	ldp q0, q1, [x9, #48]
	ldp q3, q2, [x9, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB403_101:
	sub x0, x29, #152
	mov x1, x19
	mov x2, x22
	mov x3, x24
	bl lyng_js_vm::vm::exceptions::<impl lyng_js_vm::vm::Vm>::transfer_to_exception_handler
	ldur x10, [x29, #-152]
	ldurb w9, [x29, #-144]
	cmp x10, x27
	b.ne LBB403_146
	tbz w9, #0, LBB403_149
	ldr w8, [x19, #1640]
	add w8, w8, #1
	str w8, [x19, #1640]
	ldr x0, [sp, #72]
	ldr x8, [sp, #88]
	cbz x20, LBB403_93
	sub x9, x20, #1
	b LBB403_46
LBB403_105:
	str x23, [sp, #16]
	add x0, sp, #96
	mov w8, #1
	stp w8, w26, [sp, #8]
	str x22, [sp]
	mov x1, x19
	mov x2, x21
	mov x3, x25
	ldp x4, x5, [sp, #24]
	ldr x6, [sp, #48]
	ldr x7, [sp, #88]
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::set_property_on_value
LBB403_106:
	ldr x10, [sp, #96]
	cmp x10, x27
	ldr w8, [sp, #44]
	b.ne LBB403_109
LBB403_107:
	and w8, w8, #0xff
	cmp w8, #78
	ldr x8, [sp, #88]
	b.hi LBB403_120
LBB403_108:
	ldr w2, [x8, #4]
	mov x0, x19
	mov x1, x21
	mov x3, x24
	mov x4, x22
	mov x5, x26
	mov w6, #1
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::observe_named_property_slow_path
	ldr x0, [sp, #72]
	b LBB403_32
LBB403_109:
	ldr w9, [sp, #104]
LBB403_110:
	mov x8, #-9223372036854775808
	cmp x10, x8
	b.ne LBB403_119
	cbnz w9, LBB403_119
	ldr x24, [sp, #112]
	ldr x8, [sp, #64]
	cbz x8, LBB403_115
	sub x8, x8, #1
	ldr x9, [x19, #56]
	cmp x8, x9
	b.hs LBB403_115
	ldr x9, [x19, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr x9, [sp, #88]
	ldr q0, [x9]
	str q0, [x8]
	ldp q0, q1, [x9, #48]
	ldp q3, q2, [x9, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB403_115:
	sub x0, x29, #152
	mov x1, x19
	mov x2, x21
	mov x3, x24
	bl lyng_js_vm::vm::exceptions::<impl lyng_js_vm::vm::Vm>::transfer_to_exception_handler
	ldur x10, [x29, #-152]
	ldurb w9, [x29, #-144]
	cmp x10, x27
	b.ne LBB403_129
	tbz w9, #0, LBB403_133
	ldr w8, [x19, #1640]
	add w8, w8, #1
	str w8, [x19, #1640]
	ldp x23, x0, [sp, #64]
	ldr x8, [sp, #88]
	cbz x23, LBB403_93
	sub x9, x23, #1
	b LBB403_46
LBB403_119:
	lsr w8, w9, #8
	ldr w12, [sp, #108]
	ldr x24, [sp, #112]
	ldp w23, w11, [sp, #120]
	ldr q0, [sp, #128]
	b LBB403_147
LBB403_120:
	ldrb w8, [sp, #104]
	tbz w8, #0, LBB403_125
LBB403_121:
	str x27, [sp, #160]
	b LBB403_127
LBB403_122:
	ldr x9, [x19, #80]
	sub w8, w1, #1
	cmp x9, x8
	b.ls LBB403_91
	ldr x9, [x19, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB403_91
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB403_91
	b LBB403_98
LBB403_125:
	ldr w8, [sp, #44]
	and w8, w8, #0xff
	cmp w8, #80
	b.ne LBB403_130
LBB403_126:
	mov x0, x21
	mov w1, #5
	bl lyng_js_ops::errors::error_value
	mov x8, #-9223372036854775808
	str x8, [sp, #160]
	str wzr, [sp, #168]
	str x0, [sp, #176]
LBB403_127:
	sub x0, x29, #152
	add x5, sp, #160
	mov x1, x19
	mov x2, x21
	ldr x3, [sp, #64]
	ldr x4, [sp, #88]
	bl lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::handle_dispatch_result
	ldur x10, [x29, #-152]
	ldurb w9, [x29, #-144]
	cmp x10, x27
	b.ne LBB403_146
	ldr x0, [sp, #72]
	ldr x8, [sp, #88]
	tbz w9, #0, LBB403_93
	b LBB403_108
LBB403_129:
	ldurb w8, [x29, #-141]
	ldurh w11, [x28, #9]
	orr w8, w11, w8, lsl #16
	ldur w12, [x29, #-140]
	ldur x24, [x29, #-136]
	ldp w23, w11, [x29, #-128]
	ldr q0, [x28, #32]
	str q0, [sp, #144]
	ldr q0, [sp, #144]
	b LBB403_147
LBB403_130:
	ldr x9, [x19, #80]
	ldr x8, [sp, #88]
	ldr w8, [x8, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB403_121
	ldr x9, [x19, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB403_121
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB403_121
	b LBB403_126
LBB403_133:
	mov w8, #0
	mov w9, #0
	mov x10, #-9223372036854775808
	ldr q0, [sp, #144]
	b LBB403_147
LBB403_134:
	mov w10, #5
LBB403_135:
	and x9, x8, #0x3fffffff
	ldr w8, [sp, #44]
	and w21, w8, #0xff
	orr x8, x9, x16, lsl #32
	stur w10, [x29, #-152]
	stur x8, [x28, #4]
	sub x1, x29, #152
	mov x0, x22
	bl lyng_js_gc::mutator::PrimitiveMutator::store_value
	cmp w21, #79
	b.lo LBB403_153
	cbnz w0, LBB403_153
	ldr x12, [x19, #72]
LBB403_138:
	ldr w8, [sp, #44]
	and w8, w8, #0xff
	cmp w8, #80
	b.ne LBB403_150
LBB403_139:
	mov x0, x22
	mov w1, #5
	bl lyng_js_ops::errors::error_value
	mov x24, x0
	ldr x8, [sp, #64]
	cbz x8, LBB403_142
	ldr x8, [sp, #64]
	sub x8, x8, #1
	ldr x9, [x19, #56]
	cmp x8, x9
	b.hs LBB403_142
	ldr x9, [x19, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr x9, [sp, #88]
	ldr q0, [x9]
	str q0, [x8]
	ldp q0, q1, [x9, #48]
	ldp q3, q2, [x9, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB403_142:
	sub x0, x29, #152
	mov x1, x19
	mov x2, x22
	mov x3, x24
	bl lyng_js_vm::vm::exceptions::<impl lyng_js_vm::vm::Vm>::transfer_to_exception_handler
	ldur x10, [x29, #-152]
	ldurb w9, [x29, #-144]
	cmp x10, x27
	b.ne LBB403_146
	tbz w9, #0, LBB403_149
	ldr w8, [x19, #1640]
	add w8, w8, #1
	str w8, [x19, #1640]
	ldp x9, x0, [sp, #64]
	ldr x8, [sp, #88]
	cbz x9, LBB403_93
	sub x9, x9, #1
	b LBB403_46
LBB403_146:
	ldurb w8, [x29, #-141]
	ldurh w11, [x28, #9]
	orr w8, w11, w8, lsl #16
	ldur w12, [x29, #-140]
	ldur x24, [x29, #-136]
	ldp w23, w11, [x29, #-128]
	ldr q0, [x28, #32]
LBB403_147:
	stur q0, [x29, #-176]
LBB403_148:
	ldr x0, [sp, #72]
	b LBB403_23
LBB403_149:
	mov w9, #0
	mov w8, #0
	mov x10, #-9223372036854775808
	b LBB403_148
LBB403_150:
	ldr x9, [x19, #80]
	ldr x8, [sp, #88]
	ldr w23, [x8, #4]
	sub w8, w23, #1
	cmp x9, x8
	b.ls LBB403_154
	ldr x8, [x12, x8, lsl #3]
	cbz x8, LBB403_154
	ldrh w8, [x8, #340]
	tbnz w8, #0, LBB403_139
	b LBB403_154
LBB403_153:
	ldr x8, [sp, #88]
	ldr w23, [x8, #4]
LBB403_154:
	mov x0, x19
	mov x1, x23
	mov x2, x24
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::record_feedback_slot
	ldr x8, [sp, #88]
	ldr w8, [x8, #56]
	adds w9, w8, w20
	b.hs LBB403_156
	ldr x8, [sp, #88]
	b LBB403_92
LBB403_156:
Lloh1179:
	adrp x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGE
Lloh1180:
	add x0, x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGEOFF
Lloh1181:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGE
Lloh1182:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGEOFF
	mov w1, #41
	bl core::option::expect_failed
LBB403_157:
Lloh1183:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.761@PAGE
Lloh1184:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.761@PAGEOFF
	mov x0, x8
	mov x1, x10
	bl core::panicking::panic_bounds_check
	.loh AdrpAdd	Lloh1173, Lloh1174
	.loh AdrpAdd	Lloh1175, Lloh1176
	.loh AdrpAdd	Lloh1177, Lloh1178
	.loh AdrpAdd	Lloh1181, Lloh1182
	.loh AdrpAdd	Lloh1179, Lloh1180
	.loh AdrpAdd	Lloh1183, Lloh1184
