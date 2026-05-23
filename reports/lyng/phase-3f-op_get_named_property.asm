    Finished `release` profile [optimized] target(s) in 0.01s

.section __TEXT,__text,regular,pure_instructions
	.p2align	2
lyng_vm::vm::dispatch_handlers::property::op_get_named_property:
Lfunc_begin390:
	.cfi_startproc
	.cfi_personality 155, _rust_eh_personality
	.cfi_lsda 16, Lexception192
	sub sp, sp, #384
	.cfi_def_cfa_offset 384
	stp x28, x27, [sp, #288]
	stp x26, x25, [sp, #304]
	stp x24, x23, [sp, #320]
	stp x22, x21, [sp, #336]
	stp x20, x19, [sp, #352]
	stp x29, x30, [sp, #368]
	add x29, sp, #368
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
	mov x19, x8
	ldr w21, [x0, #4]
	ldr w23, [x0, #56]
	ldrb w8, [x0, #148]
	mov w9, #152
	strb w9, [x0, #148]
	ldr x9, [x0, #128]
	ldr x1, [x9, #56]
	subs x2, x1, x23
	b.lo LBB390_62
	mov x20, x0
	mov x25, #33
	movk x25, #32768, lsl #48
	ldr x9, [x9, #48]
	add x1, x9, x23
	cmp w8, #152
	b.ne LBB390_63
	and x8, x2, #0x7ffffffffffffffe
	cmp x2, #4
	ccmp x8, #4, #4, hs
	b.eq LBB390_19
	ldrh w26, [x1, #4]
	cbz w26, LBB390_19
	ldrb w11, [x1, #1]
	ldrb w8, [x1, #2]
	mov w28, #6
	ldrb w10, [x1, #3]
	ldr x22, [x20, #80]
	ldr w12, [x20, #20]
	ldr x24, [x22, #32]
	add w0, w12, w8
	cmp x24, x0
	b.ls LBB390_65
LBB390_5:
	ldr x9, [x22, #80]
	sub w8, w21, #1
	cmp x9, x8
	b.ls LBB390_21
	ldr x9, [x22, #72]
	ldr x13, [x9, x8, lsl #3]
	cbz x13, LBB390_21
	ldr x15, [x13, #80]
	mov w14, w10
	sub x9, x25, #3
	cmp x15, x14
	b.ls LBB390_22
	ldr x15, [x13, #72]
	add x15, x15, x14, lsl #4
	ldr w14, [x15]
	cmp w14, #4
	b.eq LBB390_22
	ldr w27, [x15, #4]
	cmp w14, #2
	b.ne LBB390_26
	ldp x2, x10, [x20, #88]
	ldr x9, [x20, #104]
	stp x10, x9, [sp, #40]
	ldp x10, x9, [x20, #112]
	stp x10, x9, [sp, #56]
	ldr x9, [x20, #136]
	str x9, [sp, #32]
	add w9, w12, w11
	str x9, [sp, #72]
	ldr x3, [x22, #24]
	ldr x4, [x3, x0, lsl #3]
	ldr x9, [x13, #464]
	cmp x9, x27
	b.ls LBB390_13
	ldr x9, [x13, #456]
	add x9, x9, x27, lsl #3
	ldr w10, [x9, #16]!
	cmp w10, #1
	b.ne LBB390_13
	ldr w27, [x9, #4]
LBB390_13:
	and x9, x4, #0x7fffffff00000000
	and x9, x9, #0xfff8ffffffffffff
	mov x10, #21474836480
	movk x10, #32760, lsl #48
	cmp x9, x10
	ccmp w4, #0, #4, eq
	b.ne LBB390_38
	mov w8, #1
	stp w8, w27, [sp, #8]
	str x4, [sp]
	add x0, sp, #128
	mov x1, x22
	mov x21, x2
	ldp x3, x4, [sp, #40]
	ldp x5, x6, [sp, #56]
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::get_property_from_value
	ldr x23, [sp, #72]
	ldr x9, [sp, #128]
	cmp x9, x25
	b.ne LBB390_27
	ldr x21, [sp, #136]
LBB390_17:
	ldr x24, [x22, #32]
	cmp x24, x23
	b.ls LBB390_142
	ldr x8, [x22, #24]
	str x21, [x8, x23, lsl #3]
	b LBB390_106
LBB390_19:
	sub x8, x25, #12
	stur x8, [x29, #-144]
	stp w21, w23, [x29, #-136]
LBB390_20:
	ldp q0, q1, [x29, #-144]
	stp q0, q1, [x19]
	ldur q0, [x29, #-112]
	b LBB390_24
LBB390_21:
	sub x9, x25, #29
	mov x12, x21
	lsr x8, x21, #32
	b LBB390_23
LBB390_22:
	mov x12, #0
	lsr x8, xzr, #32
LBB390_23:
	bfi x12, x8, #32, #32
	stp x9, x12, [x19]
	str x23, [x19, #16]
	stp w21, w10, [x19, #24]
	ldr q0, [sp, #176]
LBB390_24:
	str q0, [x19, #32]
LBB390_25:
	.cfi_def_cfa wsp, 384
	ldp x29, x30, [sp, #368]
	ldp x20, x19, [sp, #352]
	ldp x22, x21, [sp, #336]
	ldp x24, x23, [sp, #320]
	ldp x26, x25, [sp, #304]
	ldp x28, x27, [sp, #288]
	add sp, sp, #384
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
LBB390_26:
	.cfi_restore_state
	ldr x23, [x15, #8]
	orr x12, x14, x27, lsl #32
	lsr x8, x12, #32
	b LBB390_23
LBB390_27:
	mov x8, #-9223372036854775808
	cmp x9, x8
	b.ne LBB390_61
	ldr w8, [sp, #136]
	cbnz w8, LBB390_61
	ldr x23, [sp, #144]
	ldr x8, [sp, #32]
	cbz x8, LBB390_32
	ldr x8, [sp, #32]
	sub x8, x8, #1
	ldr x9, [x22, #56]
	cmp x8, x9
	b.hs LBB390_32
	ldr x9, [x22, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB390_32:
	sub x0, x29, #144
	mov x1, x22
	mov x2, x21
	mov x3, x23
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
	ldur x9, [x29, #-144]
	ldurb w11, [x29, #-136]
	cmp x9, x25
	b.ne LBB390_121
	tbz w11, #0, LBB390_120
LBB390_35:
	ldr w8, [x22, #1640]
	add w8, w8, #1
	str w8, [x22, #1640]
	ldr x8, [sp, #32]
	cbz x8, LBB390_108
	sub x8, x8, #1
	ldr x9, [x22, #56]
	cmp x8, x9
	b.hs LBB390_108
	ldr x9, [x22, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x8]
	str q0, [x20]
	ldp q0, q1, [x8, #48]
	ldp q3, q2, [x8, #16]
	stp q0, q1, [x20, #48]
	stp q3, q2, [x20, #16]
	b LBB390_108
LBB390_38:
	cmp w26, #0
	ldr x9, [x22, #104]
	ccmp x9, x8, #0, ne
	cset w12, hi
	cmp w12, #1
	b.ne LBB390_41
	ldr x9, [x22, #96]
	add x10, x9, x8, lsl #5
	ldr x11, [x10, #16]
	sub w9, w26, #1
	cmp x11, x9
	b.ls LBB390_41
	ldr x10, [x10, #8]
	mov w11, #1216
	umaddl x0, w9, w11, x10
	ldr x9, [x0]
	cmp x9, #10
	ccmp x9, #6, #0, ne
	b.eq LBB390_67
LBB390_41:
	ldr x9, [x2, #224]
	mov w10, #-1
	add x10, x4, x10
	lsr w11, w10, #6
	cmp x11, x9
	b.hs LBB390_51
	ldr x13, [x2, #216]
	and x14, x10, #0x3f
	ldr x13, [x13, x11, lsl #3]
	mov w15, #80
	umaddl x16, w14, w15, x13
	mov x15, x16
	ldr w13, [x15], #8
	cmp w13, #1
	b.ne LBB390_51
	ldr w1, [x16, #52]
	cbz w1, LBB390_51
	cbz w12, LBB390_102
	ldr x12, [x22, #96]
	add x17, x12, x8, lsl #5
	ldr x14, [x17, #16]
	sub w13, w26, #1
	cmp x14, x13
	b.ls LBB390_53
	ldr x17, [x17, #8]
	mov w0, #1216
	umaddl x0, w13, w0, x17
	ldr x17, [x0]
	cmp x17, #10
	b.eq LBB390_53
	cmp x17, #6
	b.ne LBB390_53
	str x3, [sp, #24]
	mov x3, x2
	ldr x17, [x0, #1016]
	lsr x2, x17, #32
	cbz x2, LBB390_72
	cmp w1, w2
	b.ne LBB390_72
	mov x1, #0
	b LBB390_75
LBB390_51:
	cbz w12, LBB390_102
	ldr x12, [x22, #96]
	add x13, x12, x8, lsl #5
	ldr x14, [x13, #16]
	sub w13, w26, #1
LBB390_53:
	cmp x14, x13
	b.ls LBB390_102
LBB390_54:
	add x8, x12, x8, lsl #5
	ldr x8, [x8, #8]
	mov w12, #1216
	umaddl x0, w13, w12, x8
	ldr x8, [x0]
	cmp x8, #10
	b.eq LBB390_102
	cmp x8, #6
	b.ne LBB390_102
	ldr x13, [x0, #984]
	cbz x13, LBB390_102
	cmp x11, x9
	b.hs LBB390_102
	ldr x12, [x2, #216]
	and x8, x10, #0x3f
	ldr x10, [x12, x11, lsl #3]
	mov w11, #80
	umaddl x15, w8, w11, x10
	ldr w8, [x15]
	cmp w8, #1
	b.ne LBB390_102
	mov x1, x4
	str x3, [sp, #24]
	mov x17, x2
	ldr x8, [x0, #992]
	ldr x14, [x0, #1000]
	ldr x10, [x0, #1008]
	ldp w11, w16, [x15, #48]
	ldr x15, [x15, #40]
	cmp w11, #0
	cbz w16, LBB390_90
	ccmp w16, w13, #0, ne
	b LBB390_91
LBB390_61:
	ldp w11, w8, [sp, #136]
	lsr w12, w11, #8
	ldr x23, [sp, #144]
	ldp w21, w10, [sp, #152]
	ldr q0, [sp, #160]
	b LBB390_123
LBB390_62:
Lloh1097:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
Lloh1098:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov x0, x23
	mov x2, x1
	bl core::slice::index::slice_index_fail
	b LBB390_141
LBB390_63:
	sub x0, x29, #144
	mov w3, #1
	mov x4, x21
	mov x5, x23
	bl lyng_vm::vm::dispatch::decode_abc_operands_wide
	ldur x8, [x29, #-144]
	cmp x8, x25
	b.ne LBB390_20
	ldurh w11, [x29, #-132]
	ldurh w8, [x29, #-130]
	ldurh w10, [x29, #-128]
	ldur w26, [x29, #-136]
	ldur w28, [x29, #-124]
	ldr x22, [x20, #80]
	ldr w12, [x20, #20]
	ldr x24, [x22, #32]
	add w0, w12, w8
	cmp x24, x0
	b.hi LBB390_5
LBB390_65:
Lloh1099:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.759@PAGE
Lloh1100:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.759@PAGEOFF
LBB390_66:
	mov x1, x24
	bl core::panicking::panic_bounds_check
	b LBB390_141
LBB390_67:
	ldr x9, [x0, #968]
	cbz x9, LBB390_41
	ldr x13, [x2, #224]
	mov w10, #-1
	add x10, x4, x10
	lsr w11, w10, #6
	cmp x11, x13
	b.hs LBB390_41
	ldr x13, [x2, #216]
	and x10, x10, #0x3f
	ldr x11, [x13, x11, lsl #3]
	mov w13, #80
	umaddl x15, w10, w13, x11
	ldr w10, [x15]
	cmp w10, #1
	b.ne LBB390_41
	mov x17, x4
	str x3, [sp, #24]
	mov x16, x2
	ldr x10, [x0, #976]
	ldp w14, w11, [x15, #52]
	ldr x13, [x15, #40]
	ldur q0, [x15, #8]
	ldur q1, [x15, #24]
	stp q0, q1, [x29, #-176]
	lsr x15, x9, #32
	cmp x15, #0
	csel w15, w15, wzr, ne
	cbz w14, LBB390_82
	cmp w14, w15
	ccmp x13, x10, #0, eq
	mov x2, x16
	ldr x3, [sp, #24]
	mov x4, x17
	b.ne LBB390_41
	b LBB390_84
LBB390_72:
	ldr x17, [x0, #1024]
	lsr x2, x17, #32
	cbz x2, LBB390_101
	cmp w1, w2
	b.ne LBB390_101
	mov w1, #1
LBB390_75:
	add x1, x0, x1, lsl #3
	ldr x2, [x1, #1032]
	ldr w1, [x16, #56]
	ldr x16, [x16, #40]
	ldp q0, q1, [x15]
	stp q0, q1, [x29, #-144]
	cmp x16, x2
	b.ne LBB390_101
	tbnz w17, #31, LBB390_100
	cbz w1, LBB390_101
	ldr x16, [x3, #640]
	sub w15, w1, #1
	cmp x16, x15
	b.ls LBB390_101
	ldr x16, [x3, #632]
	mov w1, #24
	umaddl x15, w15, w1, x16
	ldrb w16, [x15, #19]
	cmp w16, #1
	b.ne LBB390_101
	ldr x1, [x15, #8]
	and x16, x17, #0x3fffffff
	cmp x1, x16
	b.ls LBB390_101
	ldr x8, [x15]
	add x8, x8, x16, lsl #3
	b LBB390_126
LBB390_82:
	mov x2, x16
	ldr x3, [sp, #24]
	mov x4, x17
	cbnz w15, LBB390_41
	cmp x13, x10
	b.ne LBB390_41
LBB390_84:
	and x10, x9, #0x3fffffff
	tbnz w9, #31, LBB390_96
	mov x2, x16
	ldr x3, [sp, #24]
	mov x4, x17
	cbz w11, LBB390_41
	ldr x13, [x2, #640]
	sub w9, w11, #1
	cmp x13, x9
	b.ls LBB390_41
	ldr x11, [x2, #632]
	mov w13, #24
	umaddl x9, w9, w13, x11
	ldrb w11, [x9, #19]
	cmp w11, #1
	b.ne LBB390_41
	ldr x11, [x9, #8]
	cmp x10, x11
	b.hs LBB390_41
	ldr x8, [x9]
	b LBB390_98
LBB390_90:
	ccmp w13, #0, #0, ne
LBB390_91:
	ccmp x15, x14, #0, eq
	mov x2, x17
	mov x4, x1
	b.ne LBB390_102
	sub w11, w11, #1
	lsr x13, x11, #6
	cmp x13, x9
	b.hs LBB390_102
	and x9, x11, #0x3f
	ldr x11, [x12, x13, lsl #3]
	mov w12, #80
	umaddl x13, w9, w12, x11
	ldr w9, [x13]
	cmp w9, #1
	b.ne LBB390_102
	ldp w12, w11, [x13, #52]
	ldr x9, [x13, #40]
	ldur q0, [x13, #8]
	ldur q1, [x13, #24]
	stp q0, q1, [x29, #-144]
	lsr x13, x8, #32
	cmp x13, #0
	csel w13, w13, wzr, ne
	cbz w12, LBB390_128
	cmp w12, w13
	ccmp x9, x10, #0, eq
	mov x2, x17
	mov x4, x1
	b.ne LBB390_102
	b LBB390_130
LBB390_96:
	cmp x10, #4
	mov x2, x16
	ldr x3, [sp, #24]
	mov x4, x17
	b.hs LBB390_41
	sub x8, x29, #176
LBB390_98:
	ldr x26, [x8, x10, lsl #3]
	bl lyng_vm::vm::feedback::FeedbackSiteState::record_execution
	ldp x0, x1, [x22, #120]
	mov x2, x21
	bl lyng_vm::vm::tiering::<impl lyng_vm::vm::Vm>::observe_tier_feedback_event
	ldr x8, [sp, #72]
	cmp x24, x8
	b.hi LBB390_139
Lloh1101:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.760@PAGE
Lloh1102:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.760@PAGEOFF
	ldr x0, [sp, #72]
	b LBB390_66
LBB390_100:
	tst w17, #0x3ffffffc
	b.eq LBB390_125
LBB390_101:
	mov x2, x3
	ldr x3, [sp, #24]
	cmp x14, x13
	b.hi LBB390_54
LBB390_102:
	mov x0, x22
	str x2, [sp, #24]
	mov x1, x2
	mov x2, x21
	mov x3, x26
	mov x24, x4
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_named_property_load_inline_cache_hit
	ldr x23, [sp, #72]
	tbz w0, #0, LBB390_109
	ldr x24, [x22, #32]
	cmp x24, x23
	b.ls LBB390_143
	ldr x8, [x22, #24]
	str x1, [x8, x23, lsl #3]
LBB390_106:
	ldr w8, [x20, #56]
	adds w8, w8, w28
	b.hs LBB390_140
LBB390_107:
	str w8, [x20, #56]
LBB390_108:
	ldr x8, [x20, #128]
	ldr x8, [x8, #48]
	ldr w9, [x20, #56]
	ldrb w8, [x8, x9]
Lloh1103:
	adrp x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
Lloh1104:
	add x9, x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x8, [x9, x8, lsl #3]
	stp x25, x8, [x19]
	b LBB390_25
LBB390_109:
	mov w8, #1
	stp w8, w27, [sp, #8]
	str x24, [sp]
	add x0, sp, #80
	mov x1, x22
	ldr x2, [sp, #24]
	ldp x3, x4, [sp, #40]
	ldp x5, x6, [sp, #56]
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::get_property_from_value
	ldr x9, [sp, #80]
	cmp x9, x25
	b.ne LBB390_112
	ldr x21, [sp, #88]
	ldr w2, [x20, #4]
	mov x0, x22
	ldr x1, [sp, #24]
	mov x3, x26
	mov x4, x24
	mov x5, x27
	mov w6, #0
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_named_property_slow_path
	b LBB390_17
LBB390_112:
	mov x8, #-9223372036854775808
	cmp x9, x8
	b.ne LBB390_122
	ldr w8, [sp, #88]
	cbnz w8, LBB390_122
	ldr x23, [sp, #96]
	ldr x8, [sp, #32]
	cbz x8, LBB390_117
	ldr x8, [sp, #32]
	sub x8, x8, #1
	ldr x9, [x22, #56]
	cmp x8, x9
	b.hs LBB390_117
	ldr x9, [x22, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB390_117:
	sub x0, x29, #144
	mov x1, x22
	ldr x2, [sp, #24]
	mov x3, x23
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
	ldur x9, [x29, #-144]
	ldurb w11, [x29, #-136]
	cmp x9, x25
	b.ne LBB390_121
	tbnz w11, #0, LBB390_35
LBB390_120:
	mov w12, #0
	mov w11, #0
	mov x9, #-9223372036854775808
	b LBB390_124
LBB390_121:
	ldurb w8, [x29, #-133]
	sub x10, x29, #144
	ldurh w10, [x10, #9]
	orr w12, w10, w8, lsl #16
	ldur w8, [x29, #-132]
	ldur x23, [x29, #-128]
	ldp w21, w10, [x29, #-120]
	ldur q0, [x29, #-112]
	b LBB390_123
LBB390_122:
	ldp w11, w8, [sp, #88]
	lsr w12, w11, #8
	ldr x23, [sp, #96]
	ldp w21, w10, [sp, #104]
	ldr q0, [sp, #112]
LBB390_123:
	str q0, [sp, #176]
LBB390_124:
	lsl w12, w12, #8
	bfxil x12, x11, #0, #8
	b LBB390_23
LBB390_125:
	and x8, x17, #0x3fffffff
	sub x9, x29, #144
	add x8, x9, x8, lsl #3
LBB390_126:
	ldr x26, [x8]
	bl lyng_vm::vm::feedback::FeedbackSiteState::record_execution
	ldp x0, x1, [x22, #120]
	mov x2, x21
	bl lyng_vm::vm::tiering::<impl lyng_vm::vm::Vm>::observe_tier_feedback_event
	ldr x8, [sp, #72]
	cmp x24, x8
	b.hi LBB390_139
Lloh1105:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.761@PAGE
Lloh1106:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.761@PAGEOFF
	ldr x0, [sp, #72]
	b LBB390_66
LBB390_128:
	mov x2, x17
	mov x4, x1
	cbnz w13, LBB390_102
	cmp x9, x10
	b.ne LBB390_102
LBB390_130:
	and x9, x8, #0x3fffffff
	tbnz w8, #31, LBB390_136
	mov x2, x17
	mov x4, x1
	cbz w11, LBB390_102
	ldr x10, [x2, #640]
	sub w8, w11, #1
	cmp x10, x8
	b.ls LBB390_102
	ldr x10, [x2, #632]
	mov w11, #24
	umaddl x8, w8, w11, x10
	ldrb w10, [x8, #19]
	cmp w10, #1
	b.ne LBB390_102
	ldr x10, [x8, #8]
	cmp x9, x10
	b.hs LBB390_102
	ldr x8, [x8]
	b LBB390_138
LBB390_136:
	cmp x9, #3
	mov x2, x17
	mov x4, x1
	b.hi LBB390_102
	sub x8, x29, #144
LBB390_138:
	ldr x26, [x8, x9, lsl #3]
	bl lyng_vm::vm::feedback::FeedbackSiteState::record_execution
	ldp x0, x1, [x22, #120]
	mov x2, x21
	bl lyng_vm::vm::tiering::<impl lyng_vm::vm::Vm>::observe_tier_feedback_event
	ldr x8, [sp, #72]
	cmp x24, x8
	b.ls LBB390_144
LBB390_139:
	ldr x8, [sp, #72]
	ldr x9, [sp, #24]
	str x26, [x9, x8, lsl #3]
	adds w8, w28, w23
	b.lo LBB390_107
LBB390_140:
Lloh1107:
	adrp x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGE
Lloh1108:
	add x0, x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGEOFF
Lloh1109:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGE
Lloh1110:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGEOFF
	mov w1, #41
	bl core::option::expect_failed
LBB390_141:
	brk #0x1
LBB390_142:
Lloh1111:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.764@PAGE
Lloh1112:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.764@PAGEOFF
	mov x0, x23
	b LBB390_66
LBB390_143:
Lloh1113:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.763@PAGE
Lloh1114:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.763@PAGEOFF
	mov x0, x23
	b LBB390_66
LBB390_144:
Lloh1115:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.762@PAGE
Lloh1116:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.762@PAGEOFF
	ldr x0, [sp, #72]
	b LBB390_66
	bl core::panicking::panic_cannot_unwind
	.loh AdrpAdd	Lloh1097, Lloh1098
	.loh AdrpAdd	Lloh1099, Lloh1100
	.loh AdrpAdd	Lloh1101, Lloh1102
	.loh AdrpAdd	Lloh1103, Lloh1104
	.loh AdrpAdd	Lloh1105, Lloh1106
	.loh AdrpAdd	Lloh1109, Lloh1110
	.loh AdrpAdd	Lloh1107, Lloh1108
	.loh AdrpAdd	Lloh1111, Lloh1112
	.loh AdrpAdd	Lloh1113, Lloh1114
	.loh AdrpAdd	Lloh1115, Lloh1116
