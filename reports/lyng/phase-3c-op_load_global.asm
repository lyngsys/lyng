
.section __TEXT,__text,regular,pure_instructions
	.p2align	2
lyng_vm::vm::dispatch_handlers::names::op_load_global:
Lfunc_begin358:
	.cfi_startproc
	.cfi_personality 155, _rust_eh_personality
	.cfi_lsda 16, Lexception161
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
	ldr w22, [x0, #4]
	ldr w6, [x0, #56]
	ldrb w3, [x0, #148]
	mov w8, #152
	strb w8, [x0, #148]
	ldr x8, [x0, #128]
	ldr x1, [x8, #56]
	subs x2, x1, x6
	b.lo LBB358_57
	mov x20, x0
	mov x28, #33
	movk x28, #32768, lsl #48
	ldr x8, [x8, #48]
	add x1, x8, x6
	cmp w3, #152
	b.ne LBB358_59
	and x8, x2, #0x7ffffffffffffffe
	cmp x2, #4
	ccmp x8, #4, #4, hs
	b.eq LBB358_23
	ldrh w24, [x1, #4]
	cbz w24, LBB358_23
	ldrb w10, [x1, #1]
	ldrh w8, [x1, #2]
	mov w9, #6
	stp w9, w10, [sp, #72]
	ldr x13, [x20, #80]
	ldr x9, [x13, #80]
	sub w27, w22, #1
	cmp x9, x27
	b.ls LBB358_61
LBB358_5:
	ldr x9, [x13, #72]
	ldr x10, [x9, x27, lsl #3]
	cbz x10, LBB358_61
	ldr x12, [x10, #80]
	mov w11, w8
	sub x9, x28, #3
	cmp x12, x11
	b.ls LBB358_25
	ldr x12, [x10, #72]
	add x11, x12, x11, lsl #4
	ldr w12, [x11]
	cmp w12, #4
	b.eq LBB358_25
	ldr w14, [x11, #4]
	cmp w12, #2
	b.ne LBB358_26
	str x13, [sp, #64]
	ldr x8, [x10, #464]
	cmp x8, x14
	b.ls LBB358_12
	ldr x8, [x10, #456]
	add x8, x8, x14, lsl #3
	ldr w9, [x8, #16]!
	cmp w9, #1
	b.ne LBB358_12
	ldr w14, [x8, #4]
LBB358_12:
	ldp x23, x8, [x20, #88]
	stp x8, x14, [sp, #48]
	ldp x9, x8, [x20, #104]
	stp x8, x9, [sp, #32]
	ldr x8, [x20, #120]
	str x8, [sp, #24]
	ldr w1, [x20, #12]
	sub x25, x28, #27
	str w1, [sp, #20]
LBB358_13:
	mov x26, x1
	ldr x8, [x23, #5168]
	sub w21, w1, #1
	cmp x8, x21
	b.ls LBB358_16
	ldr x8, [x23, #5160]
	lsl x9, x21, #7
	ldr x8, [x8, x9]
	cmp x8, x25
	b.eq LBB358_16
	tbz x8, #63, LBB358_27
LBB358_16:
	mov x0, x23
	mov x1, x26
	bl lyng_env::agent::environments::<impl lyng_env::agent::Agent>::environment_outer
	tbz w0, #0, LBB358_19
	cbnz w1, LBB358_13
LBB358_19:
	mov w9, #0
	sub x8, x28, #24
LBB358_20:
	lsr w10, w26, #8
	ldr q0, [sp, #80]
	str q0, [sp, #112]
	ldr x11, [sp, #96]
LBB358_21:
	str x11, [sp, #128]
LBB358_22:
	lsl w10, w10, #8
	bfxil x10, x26, #0, #8
	orr x9, x10, x9, lsl #32
	stp x8, x9, [x19]
	str x21, [x19, #16]
	ldr q0, [sp, #112]
	stur q0, [x19, #24]
	ldr x8, [sp, #128]
	str x8, [x19, #40]
	b LBB358_64
LBB358_23:
	sub x8, x28, #12
	stur x8, [x29, #-136]
	stp w22, w6, [x29, #-128]
LBB358_24:
	ldur q0, [x29, #-136]
	ldur q1, [x29, #-120]
	stp q0, q1, [x19]
	ldur q0, [x29, #-104]
	str q0, [x19, #32]
	b LBB358_64
LBB358_25:
	mov x10, #0
	b LBB358_62
LBB358_26:
	ldr x11, [x11, #8]
	orr x10, x12, x14, lsl #32
	b LBB358_63
LBB358_27:
	sub x8, x29, #136
	mov x0, x23
	mov x1, x26
	ldr x2, [sp, #56]
	bl lyng_env::agent::environments::<impl lyng_env::agent::Agent>::global_lexical_binding
	ldur w8, [x29, #-136]
	cbz w8, LBB358_37
	ldur w21, [x29, #-128]
	mov x26, x8
LBB358_30:
	sub x0, x29, #136
	ldr x1, [sp, #64]
	mov x2, x23
	mov x3, x26
	mov w4, #0
	mov x5, x21
	bl lyng_vm::vm::loop_iteration::<impl lyng_vm::vm::Vm>::environment_for_slot_access
	ldur x8, [x29, #-136]
	ldur w26, [x29, #-128]
	cmp x8, x28
	b.ne LBB358_44
	mov x0, x23
	mov x1, x26
	mov x2, x21
	bl lyng_env::agent::environments::<impl lyng_env::agent::Agent>::environment_slot
	tbz w0, #0, LBB358_19
	mov x21, x1
	mov x8, #2
	movk x8, #9, lsl #32
	movk x8, #32760, lsl #48
	cmp x1, x8
	b.ne LBB358_78
	mov x0, x23
	mov w1, #3
	bl lyng_ops::errors::error_value
	mov x21, x0
	mov w26, #0
	mov x8, #-9223372036854775808
	b LBB358_45
LBB358_37:
	ldr x8, [x23, #5168]
	cmp x8, x21
	b.ls LBB358_69
	ldr x8, [x23, #5160]
	add x8, x8, x21, lsl #7
	ldr x9, [x8]
	cmp x9, x25
	b.eq LBB358_69
	tbnz x9, #63, LBB358_69
	ldr w8, [x8, #120]
	ldr x9, [x23, #5144]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB358_69
	ldr x9, [x23, #5136]
	add x8, x9, x8, lsl #5
	ldr x9, [x8]
	mov x10, #-9223372036854775808
	cmp x9, x10
	b.eq LBB358_69
	ldr x9, [x8, #16]
	cbz x9, LBB358_69
	mov x21, #0
	ldr x8, [x8, #8]
	add x9, x8, x9, lsl #4
	b LBB358_66
LBB358_44:
	ldur w9, [x29, #-124]
	ldur x21, [x29, #-120]
	ldur q0, [x29, #-112]
	str q0, [sp, #80]
	ldur x10, [x29, #-96]
	str x10, [sp, #96]
LBB358_45:
	mov x10, #-9223372036854775808
	cmp x8, x10
	b.ne LBB358_20
	cbnz w26, LBB358_20
	ldr x22, [x20, #136]
	ldr x1, [sp, #64]
	cbz x22, LBB358_50
	sub x8, x22, #1
	ldr x9, [x1, #56]
	cmp x8, x9
	b.hs LBB358_50
	ldr x9, [x1, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB358_50:
	sub x0, x29, #136
	mov x2, x23
	mov x3, x21
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-136]
	ldurb w26, [x29, #-128]
	cmp x8, x28
	b.ne LBB358_80
	tbz w26, #0, LBB358_85
	ldr x10, [sp, #64]
	ldr w8, [x10, #1640]
	add w8, w8, #1
	str w8, [x10, #1640]
	cbz x22, LBB358_56
	sub x8, x22, #1
	ldr x9, [x10, #56]
	cmp x8, x9
	b.hs LBB358_56
	ldr x9, [x10, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x8]
	str q0, [x20]
	ldp q0, q1, [x8, #48]
	ldp q3, q2, [x8, #16]
	stp q0, q1, [x20, #48]
	stp q3, q2, [x20, #16]
LBB358_56:
	ldr x8, [x20, #128]
	ldr x8, [x8, #48]
	ldr w9, [x20, #56]
	ldrb w8, [x8, x9]
	b LBB358_79
LBB358_57:
Lloh901:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.45@PAGE
Lloh902:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.45@PAGEOFF
	mov x0, x6
	mov x2, x1
	bl core::slice::index::slice_index_fail
	brk #0x1
LBB358_59:
	sub x0, x29, #136
	mov w4, #1
	mov x5, x22
	bl lyng_vm::vm::dispatch::decode_abx_operands_wide
	ldur x8, [x29, #-136]
	cmp x8, x28
	b.ne LBB358_24
	ldurh w10, [x29, #-120]
	ldp w8, w24, [x29, #-128]
	ldur w9, [x29, #-116]
	stp w9, w10, [sp, #72]
	ldr x13, [x20, #80]
	ldr x9, [x13, #80]
	sub w27, w22, #1
	cmp x9, x27
	b.hi LBB358_5
LBB358_61:
	sub x9, x28, #29
	mov x10, x22
LBB358_62:
LBB358_63:
	stp x9, x10, [x19]
	str x11, [x19, #16]
	stp w22, w8, [x19, #24]
LBB358_64:
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
LBB358_65:
	.cfi_restore_state
	add x8, x8, #16
	add x21, x21, #1
	cmp x8, x9
	b.eq LBB358_69
LBB358_66:
	ldp w10, w11, [x8]
	cmp w10, #1
	ldr x10, [sp, #56]
	ccmp w11, w10, #0, eq
	b.ne LBB358_65
	ldrb w10, [x8, #9]
	tbz w10, #0, LBB358_65
	lsr x8, x21, #32
	cbz x8, LBB358_30
LBB358_69:
	mov x0, x23
	mov x1, x26
	bl lyng_env::agent::environments::<impl lyng_env::agent::Agent>::global_environment_object
	mov x25, x0
	cbz w0, LBB358_19
	ldr x12, [sp, #64]
	cbz w24, LBB358_75
	ldr x8, [x12, #104]
	cmp x8, x27
	b.ls LBB358_75
	ldr x8, [x12, #96]
	add x9, x8, x27, lsl #5
	ldr x10, [x9, #16]
	sub w8, w24, #1
	cmp x10, x8
	b.ls LBB358_75
	ldr x9, [x9, #8]
	mov w10, #1096
	umaddl x0, w8, w10, x9
	ldr x8, [x0]
	cmp x8, #10
	ccmp x8, #6, #0, ne
	b.eq LBB358_89
LBB358_75:
	mov x0, x12
	mov x1, x23
	mov x2, x22
	mov x3, x24
	mov x4, x25
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_named_property_load_inline_cache_hit
	tbz w0, #0, LBB358_81
	mov x21, x1
LBB358_78:
	ldr w8, [x20, #20]
	ldr x9, [x20, #80]
	ldr w10, [sp, #76]
	add w8, w8, w10
	ldr x9, [x9, #24]
	str x21, [x9, w8, uxtw #3]
	ldr w8, [x20, #56]
	ldr w9, [sp, #72]
	add w8, w8, w9
	str w8, [x20, #56]
	ldr x9, [x20, #128]
	ldr x9, [x9, #48]
	ldrb w8, [x9, w8, uxtw]
LBB358_79:
Lloh903:
	adrp x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
Lloh904:
	add x9, x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x8, [x9, x8, lsl #3]
	stp x28, x8, [x19]
	b LBB358_64
LBB358_80:
	ldurb w9, [x29, #-125]
	ldurh w10, [x29, #-127]
	orr w10, w10, w9, lsl #16
	ldur w9, [x29, #-124]
	ldur x21, [x29, #-120]
	ldur q0, [x29, #-112]
	str q0, [sp, #112]
	ldur x11, [x29, #-96]
	b LBB358_21
LBB358_81:
	ldp x9, x1, [sp, #56]
	ldr w8, [sp, #20]
	stp w8, w9, [sp]
	sub x0, x29, #136
	mov x2, x23
	ldp x4, x3, [sp, #40]
	ldp x6, x5, [sp, #24]
	mov x7, x20
	bl lyng_vm::vm::names::<impl lyng_vm::vm::Vm>::get_global_property_binding_with_context
	ldp x8, x26, [x29, #-136]
	ldur x21, [x29, #-120]
	cmp x8, x28
	b.ne LBB358_86
	tbz w26, #0, LBB358_87
	ldp x5, x0, [sp, #56]
	mov x1, x23
	mov x2, x22
	mov x3, x24
	mov x4, x25
	mov w6, #0
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_named_property_slow_path
	b LBB358_78
LBB358_85:
	mov w26, #0
	mov w10, #0
	mov x8, #-9223372036854775808
	b LBB358_22
LBB358_86:
	ldur q0, [x29, #-112]
	str q0, [sp, #80]
	ldur x9, [x29, #-96]
	str x9, [sp, #96]
	lsr x9, x26, #32
	b LBB358_45
LBB358_87:
	mov x0, x23
	mov w1, #3
	bl lyng_ops::errors::error_value
	mov x21, x0
	mov w9, #0
	mov w26, #0
	mov x8, #-9223372036854775808
	b LBB358_45
LBB358_89:
	ldr x8, [x0, #968]
	cbz x8, LBB358_75
	ldr x11, [x23, #224]
	sub w9, w25, #1
	lsr x10, x9, #6
	cmp x10, x11
	b.hs LBB358_75
	ldr x11, [x23, #216]
	and x9, x9, #0x3f
	ldr x10, [x11, x10, lsl #3]
	mov w11, #80
	umaddl x13, w9, w11, x10
	ldr w9, [x13]
	cmp w9, #1
	b.ne LBB358_75
	ldr x9, [x0, #976]
	ldp w12, w10, [x13, #52]
	ldr x11, [x13, #40]
	ldur q0, [x13, #8]
	ldur q1, [x13, #24]
	stp q0, q1, [sp, #144]
	lsr x13, x8, #32
	cmp x13, #0
	csel w13, w13, wzr, ne
	cbz w12, LBB358_94
	cmp w12, w13
	b LBB358_95
LBB358_94:
	cmp w13, #0
LBB358_95:
	ccmp x11, x9, #0, eq
	ldr x12, [sp, #64]
	b.ne LBB358_75
	and x9, x8, #0x3fffffff
	tbnz w8, #31, LBB358_102
	ldr x12, [sp, #64]
	cbz w10, LBB358_75
	ldr x11, [x23, #640]
	sub w8, w10, #1
	cmp x11, x8
	b.ls LBB358_75
	ldr x10, [x23, #632]
	mov w11, #24
	umaddl x8, w8, w11, x10
	ldrb w10, [x8, #19]
	cmp w10, #1
	b.ne LBB358_75
	ldr x10, [x8, #8]
	cmp x9, x10
	b.hs LBB358_75
	ldr x8, [x8]
	b LBB358_104
LBB358_102:
	cmp x9, #4
	ldr x12, [sp, #64]
	b.hs LBB358_75
	add x8, sp, #144
LBB358_104:
	ldr x21, [x8, x9, lsl #3]
	bl lyng_vm::vm::feedback::FeedbackSiteState::record_execution
	ldr x8, [sp, #64]
	ldp x0, x1, [x8, #120]
	mov x2, x22
	bl lyng_vm::vm::tiering::<impl lyng_vm::vm::Vm>::observe_tier_feedback_event
	b LBB358_78
	bl core::panicking::panic_cannot_unwind
	bl core::panicking::panic_cannot_unwind
	bl core::panicking::panic_cannot_unwind
	.loh AdrpAdd	Lloh901, Lloh902
	.loh AdrpAdd	Lloh903, Lloh904
