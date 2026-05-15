.section __TEXT,__text,regular,pure_instructions
	.p2align	2
lyng_js_vm::vm::dispatch_handlers::property::op_get_named_property:
Lfunc_begin390:
	.cfi_startproc
	.cfi_personality 155, _rust_eh_personality
	.cfi_lsda 16, Lexception192
	sub sp, sp, #336
	.cfi_def_cfa_offset 336
	stp x28, x27, [sp, #240]
	stp x26, x25, [sp, #256]
	stp x24, x23, [sp, #272]
	stp x22, x21, [sp, #288]
	stp x20, x19, [sp, #304]
	stp x29, x30, [sp, #320]
	add x29, sp, #320
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
	ldr w5, [x0, #56]
	ldrb w8, [x0, #148]
	mov w9, #152
	strb w9, [x0, #148]
	ldr x9, [x0, #128]
	ldr x1, [x9, #56]
	subs x2, x1, x5
	b.lo LBB390_62
	mov x20, x0
	mov x28, #33
	movk x28, #32768, lsl #48
	ldr x9, [x9, #48]
	add x1, x9, x5
	cmp w8, #152
	b.ne LBB390_63
	and x8, x2, #0x7ffffffffffffffe
	cmp x2, #4
	ccmp x8, #4, #4, hs
	b.eq LBB390_19
	ldrh w23, [x1, #4]
	cbz w23, LBB390_19
	ldrb w10, [x1, #1]
	ldrb w8, [x1, #2]
	mov w24, #6
	ldrb w9, [x1, #3]
	ldr x22, [x20, #80]
	ldr w11, [x20, #20]
	ldr x1, [x22, #32]
	add w0, w11, w8
	cmp x1, x0
	b.ls LBB390_65
LBB390_5:
	ldr x12, [x22, #80]
	sub w8, w21, #1
	cmp x12, x8
	b.ls LBB390_21
	ldr x12, [x22, #72]
	ldr x12, [x12, x8, lsl #3]
	cbz x12, LBB390_21
	ldr x14, [x12, #80]
	mov w13, w9
	sub x8, x28, #3
	cmp x14, x13
	b.ls LBB390_22
	ldr x14, [x12, #72]
	add x14, x14, x13, lsl #4
	ldr w13, [x14]
	cmp w13, #4
	b.eq LBB390_22
	ldr w25, [x14, #4]
	cmp w13, #2
	b.ne LBB390_26
	ldp x13, x27, [x20, #88]
	ldp x9, x8, [x20, #104]
	stp x9, x8, [sp, #40]
	ldr x8, [x20, #120]
	str x8, [sp, #56]
	ldr x9, [x20, #136]
	add w8, w11, w10
	stp x9, x8, [sp, #24]
	ldr x8, [x22, #24]
	ldr x26, [x8, x0, lsl #3]
	ldr x8, [x12, #464]
	cmp x8, x25
	b.ls LBB390_13
	ldr x8, [x12, #456]
	add x8, x8, x25, lsl #3
	ldr w9, [x8, #16]!
	cmp w9, #1
	b.ne LBB390_13
	ldr w25, [x8, #4]
LBB390_13:
	and x8, x26, #0x7fffffff00000000
	and x8, x8, #0xfff8ffffffffffff
	mov x9, #21474836480
	movk x9, #32760, lsl #48
	cmp x8, x9
	ccmp w26, #0, #4, eq
	b.ne LBB390_38
	mov w8, #1
	stp w8, w25, [sp, #8]
	str x26, [sp]
	add x0, sp, #112
	mov x1, x22
	mov x21, x13
	mov x2, x13
	mov x3, x27
	ldp x4, x5, [sp, #40]
	ldr x6, [sp, #56]
	mov x7, x20
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::get_property_from_value
	ldr x8, [sp, #112]
	cmp x8, x28
	b.ne LBB390_27
	ldr x21, [sp, #120]
LBB390_17:
	ldr x1, [x22, #32]
	ldr x0, [sp, #32]
	cmp x1, x0
	b.ls LBB390_69
	ldr x8, [x22, #24]
	str x21, [x8, x0, lsl #3]
	b LBB390_42
LBB390_19:
	sub x8, x28, #12
	stur x8, [x29, #-136]
	stp w21, w5, [x29, #-128]
LBB390_20:
	ldur q0, [x29, #-136]
	ldur q1, [x29, #-120]
	stp q0, q1, [x19]
	ldur q0, [x29, #-104]
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
	ldr q0, [sp, #160]
LBB390_24:
	str q0, [x19, #32]
LBB390_25:
	.cfi_def_cfa wsp, 336
	ldp x29, x30, [sp, #320]
	ldp x20, x19, [sp, #304]
	ldp x22, x21, [sp, #288]
	ldp x24, x23, [sp, #272]
	ldp x26, x25, [sp, #256]
	ldp x28, x27, [sp, #240]
	add sp, sp, #336
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
	ldr x23, [x14, #8]
	orr x12, x13, x25, lsl #32
	lsr x10, x12, #32
	b LBB390_23
LBB390_27:
	mov x9, #-9223372036854775808
	cmp x8, x9
	b.ne LBB390_45
	ldr w9, [sp, #120]
	cbnz w9, LBB390_45
	ldr x23, [sp, #128]
	ldr x8, [sp, #24]
	cbz x8, LBB390_32
	ldr x8, [sp, #24]
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
	sub x0, x29, #136
	mov x1, x22
	mov x2, x21
	mov x3, x23
	bl lyng_js_vm::vm::exceptions::<impl lyng_js_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-136]
	ldurb w11, [x29, #-128]
	cmp x8, x28
	b.ne LBB390_58
	tbz w11, #0, LBB390_57
LBB390_35:
	ldr w8, [x22, #1640]
	add w8, w8, #1
	str w8, [x22, #1640]
	ldr x8, [sp, #24]
	cbz x8, LBB390_44
	sub x8, x8, #1
	ldr x9, [x22, #56]
	cmp x8, x9
	b.hs LBB390_44
	ldr x9, [x22, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x8]
	str q0, [x20]
	ldp q0, q1, [x8, #48]
	ldp q3, q2, [x8, #16]
	stp q0, q1, [x20, #48]
	stp q3, q2, [x20, #16]
	b LBB390_44
LBB390_38:
	mov x0, x22
	str x13, [sp, #16]
	mov x1, x13
	mov x2, x21
	mov x3, x23
	mov x4, x26
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::try_named_property_load_inline_cache_hit
	tbz w0, #0, LBB390_46
	mov x8, x1
	ldr x1, [x22, #32]
	ldr x0, [sp, #32]
	cmp x1, x0
	b.ls LBB390_70
	ldr x9, [x22, #24]
	str x8, [x9, x0, lsl #3]
LBB390_42:
	ldr w8, [x20, #56]
	adds w8, w8, w24
	b.hs LBB390_67
	str w8, [x20, #56]
LBB390_44:
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
LBB390_45:
	ldp w11, w10, [sp, #120]
	lsr w12, w11, #8
	ldr x23, [sp, #128]
	ldp w21, w9, [sp, #136]
	ldr q0, [sp, #144]
	b LBB390_60
LBB390_46:
	mov w8, #1
	stp w8, w25, [sp, #8]
	str x26, [sp]
	add x0, sp, #64
	mov x1, x22
	ldr x2, [sp, #16]
	mov x3, x27
	ldp x4, x5, [sp, #40]
	ldr x6, [sp, #56]
	mov x7, x20
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::get_property_from_value
	ldr x8, [sp, #64]
	cmp x8, x28
	b.ne LBB390_49
	ldr x21, [sp, #72]
	ldr w2, [x20, #4]
	mov x0, x22
	ldr x1, [sp, #16]
	mov x3, x23
	mov x4, x26
	mov x5, x25
	mov w6, #0
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::observe_named_property_slow_path
	b LBB390_17
LBB390_49:
	mov x9, #-9223372036854775808
	cmp x8, x9
	b.ne LBB390_59
	ldr w9, [sp, #72]
	cbnz w9, LBB390_59
	ldr x23, [sp, #80]
	ldr x8, [sp, #24]
	cbz x8, LBB390_54
	ldr x8, [sp, #24]
	sub x8, x8, #1
	ldr x9, [x22, #56]
	cmp x8, x9
	b.hs LBB390_54
	ldr x9, [x22, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB390_54:
	sub x0, x29, #136
	mov x1, x22
	ldr x2, [sp, #16]
	mov x3, x23
	bl lyng_js_vm::vm::exceptions::<impl lyng_js_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-136]
	ldurb w11, [x29, #-128]
	cmp x8, x28
	b.ne LBB390_58
	tbnz w11, #0, LBB390_35
LBB390_57:
	mov w12, #0
	mov w11, #0
	mov x8, #-9223372036854775808
	b LBB390_61
LBB390_58:
	ldurb w9, [x29, #-125]
	ldurh w10, [x29, #-127]
	orr w12, w10, w9, lsl #16
	ldur w10, [x29, #-124]
	ldur x23, [x29, #-120]
	ldp w21, w9, [x29, #-112]
	ldur q0, [x29, #-104]
	b LBB390_60
LBB390_59:
	ldp w11, w10, [sp, #72]
	lsr w12, w11, #8
	ldr x23, [sp, #80]
	ldp w21, w9, [sp, #88]
	ldr q0, [sp, #96]
LBB390_60:
	str q0, [sp, #160]
LBB390_61:
	lsl w12, w12, #8
	bfxil x12, x11, #0, #8
	b LBB390_23
LBB390_62:
Lloh1081:
	adrp x3, l_anon.3dc9f61581f81da213707151cc3b89a9.45@PAGE
Lloh1082:
	add x3, x3, l_anon.3dc9f61581f81da213707151cc3b89a9.45@PAGEOFF
	mov x0, x5
	mov x2, x1
	bl core::slice::index::slice_index_fail
	b LBB390_68
LBB390_63:
	sub x0, x29, #136
	mov w3, #1
	mov x4, x21
	bl lyng_js_vm::vm::dispatch::decode_abc_operands_wide
	ldur x8, [x29, #-136]
	cmp x8, x28
	b.ne LBB390_20
	ldurh w10, [x29, #-124]
	ldurh w8, [x29, #-122]
	ldurh w9, [x29, #-120]
	ldur w23, [x29, #-128]
	ldur w24, [x29, #-116]
	ldr x22, [x20, #80]
	ldr w11, [x20, #20]
	ldr x1, [x22, #32]
	add w0, w11, w8
	cmp x1, x0
	b.hi LBB390_5
LBB390_65:
Lloh1083:
	adrp x2, l_anon.3dc9f61581f81da213707151cc3b89a9.756@PAGE
Lloh1084:
	add x2, x2, l_anon.3dc9f61581f81da213707151cc3b89a9.756@PAGEOFF
LBB390_66:
	bl core::panicking::panic_bounds_check
	b LBB390_68
LBB390_67:
Lloh1085:
	adrp x0, l_anon.3dc9f61581f81da213707151cc3b89a9.36@PAGE
Lloh1086:
	add x0, x0, l_anon.3dc9f61581f81da213707151cc3b89a9.36@PAGEOFF
Lloh1087:
	adrp x2, l_anon.3dc9f61581f81da213707151cc3b89a9.40@PAGE
Lloh1088:
	add x2, x2, l_anon.3dc9f61581f81da213707151cc3b89a9.40@PAGEOFF
	mov w1, #41
	bl core::option::expect_failed
LBB390_68:
	brk #0x1
LBB390_69:
Lloh1089:
	adrp x2, l_anon.3dc9f61581f81da213707151cc3b89a9.758@PAGE
Lloh1090:
	add x2, x2, l_anon.3dc9f61581f81da213707151cc3b89a9.758@PAGEOFF
	b LBB390_66
LBB390_70:
Lloh1091:
	adrp x2, l_anon.3dc9f61581f81da213707151cc3b89a9.757@PAGE
Lloh1092:
	add x2, x2, l_anon.3dc9f61581f81da213707151cc3b89a9.757@PAGEOFF
	b LBB390_66
	bl core::panicking::panic_cannot_unwind
	.loh AdrpAdd	Lloh1079, Lloh1080
	.loh AdrpAdd	Lloh1081, Lloh1082
	.loh AdrpAdd	Lloh1083, Lloh1084
	.loh AdrpAdd	Lloh1087, Lloh1088
	.loh AdrpAdd	Lloh1085, Lloh1086
	.loh AdrpAdd	Lloh1089, Lloh1090
	.loh AdrpAdd	Lloh1091, Lloh1092
