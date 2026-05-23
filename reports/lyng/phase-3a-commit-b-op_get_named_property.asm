
.section __TEXT,__text,regular,pure_instructions
	.p2align	2
lyng_js_vm::vm::dispatch_handlers::property::op_get_named_property:
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
	b.lo LBB390_66
	mov x20, x0
	mov x28, #33
	movk x28, #32768, lsl #48
	ldr x9, [x9, #48]
	add x1, x9, x23
	cmp w8, #152
	b.ne LBB390_67
	and x8, x2, #0x7ffffffffffffffe
	cmp x2, #4
	ccmp x8, #4, #4, hs
	b.eq LBB390_19
	ldrh w25, [x1, #4]
	cbz w25, LBB390_19
	ldrb w12, [x1, #1]
	ldrb w8, [x1, #2]
	mov w24, #6
	ldrb w9, [x1, #3]
	ldr x22, [x20, #80]
	ldr w13, [x20, #20]
	ldr x10, [x22, #32]
	add w0, w13, w8
	cmp x10, x0
	b.ls LBB390_69
LBB390_5:
	ldr x8, [x22, #80]
	sub w11, w21, #1
	cmp x8, x11
	b.ls LBB390_21
	ldr x8, [x22, #72]
	ldr x14, [x8, x11, lsl #3]
	cbz x14, LBB390_21
	ldr x16, [x14, #80]
	mov w15, w9
	sub x8, x28, #3
	cmp x16, x15
	b.ls LBB390_22
	ldr x16, [x14, #72]
	add x16, x16, x15, lsl #4
	ldr w15, [x16]
	cmp w15, #4
	b.eq LBB390_22
	ldr w26, [x16, #4]
	cmp w15, #2
	b.ne LBB390_26
	ldp x15, x9, [x20, #88]
	ldr x8, [x20, #104]
	stp x9, x8, [sp, #40]
	ldp x9, x8, [x20, #112]
	stp x9, x8, [sp, #56]
	ldr x8, [x20, #136]
	str x8, [sp, #32]
	add w8, w13, w12
	str x8, [sp, #72]
	ldr x13, [x22, #24]
	ldr x27, [x13, x0, lsl #3]
	ldr x8, [x14, #464]
	cmp x8, x26
	b.ls LBB390_13
	ldr x8, [x14, #456]
	add x8, x8, x26, lsl #3
	ldr w9, [x8, #16]!
	cmp w9, #1
	b.ne LBB390_13
	ldr w26, [x8, #4]
LBB390_13:
	and x8, x27, #0x7fffffff00000000
	and x8, x8, #0xfff8ffffffffffff
	mov x9, #21474836480
	movk x9, #32760, lsl #48
	cmp x8, x9
	ccmp w27, #0, #4, eq
	b.ne LBB390_38
	mov w8, #1
	stp w8, w26, [sp, #8]
	str x27, [sp]
	add x0, sp, #128
	mov x1, x22
	mov x21, x15
	mov x2, x15
	ldp x3, x4, [sp, #40]
	ldp x5, x6, [sp, #56]
	mov x7, x20
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::get_property_from_value
	ldr x23, [sp, #72]
	ldr x8, [sp, #128]
	cmp x8, x28
	b.ne LBB390_27
	ldr x21, [sp, #136]
LBB390_17:
	ldr x8, [x22, #32]
	cmp x8, x23
	b.ls LBB390_89
	ldr x8, [x22, #24]
	str x21, [x8, x23, lsl #3]
	b LBB390_46
LBB390_19:
	sub x8, x28, #12
	stur x8, [x29, #-144]
	stp w21, w23, [x29, #-136]
LBB390_20:
	ldp q0, q1, [x29, #-144]
	stp q0, q1, [x19]
	ldur q0, [x29, #-112]
	b LBB390_24
LBB390_21:
	sub x8, x28, #29
	mov x12, x21
	lsr x10, x21, #32
	b LBB390_23
LBB390_22:
	mov x12, #0
	lsr x10, xzr, #32
LBB390_23:
	bfi x12, x10, #32, #32
	stp x8, x12, [x19]
	str x23, [x19, #16]
	stp w21, w9, [x19, #24]
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
	ldr x23, [x16, #8]
	orr x12, x15, x26, lsl #32
	lsr x10, x12, #32
	b LBB390_23
LBB390_27:
	mov x9, #-9223372036854775808
	cmp x8, x9
	b.ne LBB390_49
	ldr w9, [sp, #136]
	cbnz w9, LBB390_49
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
	bl lyng_js_vm::vm::exceptions::<impl lyng_js_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-144]
	ldurb w11, [x29, #-136]
	cmp x8, x28
	b.ne LBB390_62
	tbz w11, #0, LBB390_61
LBB390_35:
	ldr w8, [x22, #1640]
	add w8, w8, #1
	str w8, [x22, #1640]
	ldr x8, [sp, #32]
	cbz x8, LBB390_48
	sub x8, x8, #1
	ldr x9, [x22, #56]
	cmp x8, x9
	b.hs LBB390_48
	ldr x9, [x22, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x8]
	str q0, [x20]
	ldp q0, q1, [x8, #48]
	ldp q3, q2, [x8, #16]
	stp q0, q1, [x20, #48]
	stp q3, q2, [x20, #16]
	b LBB390_48
LBB390_38:
	cbz w25, LBB390_42
	ldr x8, [x22, #104]
	cmp x8, x11
	b.ls LBB390_42
	ldr x8, [x22, #96]
	add x9, x8, x11, lsl #5
	ldr x11, [x9, #16]
	sub w8, w25, #1
	cmp x11, x8
	b.ls LBB390_42
	ldr x9, [x9, #8]
	mov w11, #1096
	umaddl x0, w8, w11, x9
	ldr x8, [x0]
	cmp x8, #10
	ccmp x8, #6, #0, ne
	b.eq LBB390_71
LBB390_42:
	mov x0, x22
	str x15, [sp, #24]
	mov x1, x15
	mov x2, x21
	mov x3, x25
	mov x4, x27
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::try_named_property_load_inline_cache_hit
	ldr x23, [sp, #72]
	tbz w0, #0, LBB390_50
	ldr x8, [x22, #32]
	cmp x8, x23
	b.ls LBB390_90
	ldr x8, [x22, #24]
	str x1, [x8, x23, lsl #3]
LBB390_46:
	ldr w8, [x20, #56]
	adds w8, w8, w24
	b.hs LBB390_87
LBB390_47:
	str w8, [x20, #56]
LBB390_48:
	ldr x8, [x20, #128]
	ldr x8, [x8, #48]
	ldr w9, [x20, #56]
	ldrb w8, [x8, x9]
Lloh1079:
	adrp x9, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
Lloh1080:
	add x9, x9, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x8, [x9, x8, lsl #3]
	stp x28, x8, [x19]
	b LBB390_25
LBB390_49:
	ldp w11, w10, [sp, #136]
	lsr w12, w11, #8
	ldr x23, [sp, #144]
	ldp w21, w9, [sp, #152]
	ldr q0, [sp, #160]
	b LBB390_64
LBB390_50:
	mov w8, #1
	stp w8, w26, [sp, #8]
	str x27, [sp]
	add x0, sp, #80
	mov x1, x22
	ldr x2, [sp, #24]
	ldp x3, x4, [sp, #40]
	ldp x5, x6, [sp, #56]
	mov x7, x20
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::get_property_from_value
	ldr x8, [sp, #80]
	cmp x8, x28
	b.ne LBB390_53
	ldr x21, [sp, #88]
	ldr w2, [x20, #4]
	mov x0, x22
	ldr x1, [sp, #24]
	mov x3, x25
	mov x4, x27
	mov x5, x26
	mov w6, #0
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::observe_named_property_slow_path
	b LBB390_17
LBB390_53:
	mov x9, #-9223372036854775808
	cmp x8, x9
	b.ne LBB390_63
	ldr w9, [sp, #88]
	cbnz w9, LBB390_63
	ldr x23, [sp, #96]
	ldr x8, [sp, #32]
	cbz x8, LBB390_58
	ldr x8, [sp, #32]
	sub x8, x8, #1
	ldr x9, [x22, #56]
	cmp x8, x9
	b.hs LBB390_58
	ldr x9, [x22, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB390_58:
	sub x0, x29, #144
	mov x1, x22
	ldr x2, [sp, #24]
	mov x3, x23
	bl lyng_js_vm::vm::exceptions::<impl lyng_js_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-144]
	ldurb w11, [x29, #-136]
	cmp x8, x28
	b.ne LBB390_62
	tbnz w11, #0, LBB390_35
LBB390_61:
	mov w12, #0
	mov w11, #0
	mov x8, #-9223372036854775808
	b LBB390_65
LBB390_62:
	ldurb w9, [x29, #-133]
	sub x10, x29, #144
	ldurh w10, [x10, #9]
	orr w12, w10, w9, lsl #16
	ldur w10, [x29, #-132]
	ldur x23, [x29, #-128]
	ldp w21, w9, [x29, #-120]
	ldur q0, [x29, #-112]
	b LBB390_64
LBB390_63:
	ldp w11, w10, [sp, #88]
	lsr w12, w11, #8
	ldr x23, [sp, #96]
	ldp w21, w9, [sp, #104]
	ldr q0, [sp, #112]
LBB390_64:
	str q0, [sp, #176]
LBB390_65:
	lsl w12, w12, #8
	bfxil x12, x11, #0, #8
	b LBB390_23
LBB390_66:
Lloh1081:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.45@PAGE
Lloh1082:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.45@PAGEOFF
	mov x0, x23
	mov x2, x1
	bl core::slice::index::slice_index_fail
	b LBB390_88
LBB390_67:
	sub x0, x29, #144
	mov w3, #1
	mov x4, x21
	mov x5, x23
	bl lyng_js_vm::vm::dispatch::decode_abc_operands_wide
	ldur x8, [x29, #-144]
	cmp x8, x28
	b.ne LBB390_20
	ldurh w12, [x29, #-132]
	ldurh w8, [x29, #-130]
	ldurh w9, [x29, #-128]
	ldur w25, [x29, #-136]
	ldur w24, [x29, #-124]
	ldr x22, [x20, #80]
	ldr w13, [x20, #20]
	ldr x10, [x22, #32]
	add w0, w13, w8
	cmp x10, x0
	b.hi LBB390_5
LBB390_69:
	str x10, [sp, #24]
Lloh1083:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.756@PAGE
Lloh1084:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.756@PAGEOFF
LBB390_70:
	ldr x1, [sp, #24]
	bl core::panicking::panic_bounds_check
	b LBB390_88
LBB390_71:
	ldr x8, [x0, #968]
	cbz x8, LBB390_42
	ldr x12, [x15, #224]
	mov w9, #-1
	add x9, x27, x9
	lsr w11, w9, #6
	cmp x11, x12
	b.hs LBB390_42
	ldr x12, [x15, #216]
	and x9, x9, #0x3f
	ldr x11, [x12, x11, lsl #3]
	mov w12, #80
	umaddl x11, w9, w12, x11
	ldr w9, [x11]
	cmp w9, #1
	b.ne LBB390_42
	stp x13, x10, [sp, #16]
	mov x12, x15
	ldp w10, w9, [x11, #52]
	ldur q0, [x11, #8]
	ldur q1, [x11, #24]
	stp q0, q1, [x29, #-176]
	lsr x11, x8, #32
	cmp x11, #0
	csel w11, w11, wzr, ne
	cbz w10, LBB390_76
	cmp w10, w11
	mov x15, x12
	b.ne LBB390_42
	b LBB390_77
LBB390_76:
	mov x15, x12
	cbnz w11, LBB390_42
LBB390_77:
	tbnz w8, #31, LBB390_83
	mov x15, x12
	cbz w9, LBB390_42
	ldr x10, [x15, #640]
	sub w9, w9, #1
	cmp x10, x9
	b.ls LBB390_42
	ldr x10, [x15, #632]
	mov w11, #24
	umaddl x9, w9, w11, x10
	ldrb w10, [x9, #19]
	cmp w10, #1
	b.ne LBB390_42
	ldr x10, [x9, #8]
	and x8, x8, #0x7fffffff
	cmp x10, x8
	b.ls LBB390_42
	ldr x9, [x9]
	b LBB390_85
LBB390_83:
	tst w8, #0x7ffffffc
	mov x15, x12
	b.ne LBB390_42
	and x8, x8, #0x7fffffff
	sub x9, x29, #176
LBB390_85:
	add x8, x9, x8, lsl #3
	ldr x25, [x8]
	bl lyng_js_vm::vm::feedback::FeedbackSiteState::record_execution
	ldp x0, x1, [x22, #120]
	mov x2, x21
	bl lyng_js_vm::vm::tiering::<impl lyng_js_vm::vm::Vm>::observe_tier_feedback_event
	ldr x8, [sp, #24]
	ldr x9, [sp, #72]
	cmp x8, x9
	b.ls LBB390_91
	ldr x8, [sp, #72]
	ldr x9, [sp, #16]
	str x25, [x9, x8, lsl #3]
	adds w8, w24, w23
	b.lo LBB390_47
LBB390_87:
Lloh1085:
	adrp x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGE
Lloh1086:
	add x0, x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGEOFF
Lloh1087:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGE
Lloh1088:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGEOFF
	mov w1, #41
	bl core::option::expect_failed
LBB390_88:
	brk #0x1
LBB390_89:
	str x8, [sp, #24]
Lloh1089:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.759@PAGE
Lloh1090:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.759@PAGEOFF
	mov x0, x23
	b LBB390_70
LBB390_90:
	str x8, [sp, #24]
Lloh1091:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.758@PAGE
Lloh1092:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.758@PAGEOFF
	mov x0, x23
	b LBB390_70
LBB390_91:
Lloh1093:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.757@PAGE
Lloh1094:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.757@PAGEOFF
	ldr x0, [sp, #72]
	b LBB390_70
	bl core::panicking::panic_cannot_unwind
	.loh AdrpAdd	Lloh1079, Lloh1080
	.loh AdrpAdd	Lloh1081, Lloh1082
	.loh AdrpAdd	Lloh1083, Lloh1084
	.loh AdrpAdd	Lloh1087, Lloh1088
	.loh AdrpAdd	Lloh1085, Lloh1086
	.loh AdrpAdd	Lloh1089, Lloh1090
	.loh AdrpAdd	Lloh1091, Lloh1092
	.loh AdrpAdd	Lloh1093, Lloh1094
