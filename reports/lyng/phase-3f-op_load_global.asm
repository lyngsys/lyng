    Finished `release` profile [optimized] target(s) in 0.01s

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
	mov x26, #33
	movk x26, #32768, lsl #48
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
	sub w28, w22, #1
	cmp x9, x28
	b.ls LBB358_61
LBB358_5:
	ldr x9, [x13, #72]
	ldr x10, [x9, x28, lsl #3]
	cbz x10, LBB358_61
	ldr x12, [x10, #80]
	mov w11, w8
	sub x9, x26, #3
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
	sub x25, x26, #27
	str w1, [sp, #20]
LBB358_13:
	mov x27, x1
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
	mov x1, x27
	bl lyng_env::agent::environments::<impl lyng_env::agent::Agent>::environment_outer
	tbz w0, #0, LBB358_19
	cbnz w1, LBB358_13
LBB358_19:
	mov w9, #0
	sub x8, x26, #24
LBB358_20:
	lsr w10, w27, #8
	ldr q0, [sp, #80]
	str q0, [sp, #112]
	ldr x11, [sp, #96]
LBB358_21:
	str x11, [sp, #128]
LBB358_22:
	lsl w10, w10, #8
	bfxil x10, x27, #0, #8
	orr x9, x10, x9, lsl #32
	stp x8, x9, [x19]
	str x25, [x19, #16]
	ldr q0, [sp, #112]
	stur q0, [x19, #24]
	ldr x8, [sp, #128]
	str x8, [x19, #40]
	b LBB358_64
LBB358_23:
	sub x8, x26, #12
	stur x8, [x29, #-144]
	stp w22, w6, [x29, #-136]
LBB358_24:
	ldp q0, q1, [x29, #-144]
	stp q0, q1, [x19]
	ldur q0, [x29, #-112]
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
	sub x8, x29, #144
	mov x0, x23
	mov x1, x27
	ldr x2, [sp, #56]
	bl lyng_env::agent::environments::<impl lyng_env::agent::Agent>::global_lexical_binding
	ldur w8, [x29, #-144]
	cbz w8, LBB358_37
	ldur w21, [x29, #-136]
	mov x27, x8
LBB358_30:
	sub x0, x29, #144
	ldr x1, [sp, #64]
	mov x2, x23
	mov x3, x27
	mov w4, #0
	mov x5, x21
	bl lyng_vm::vm::loop_iteration::<impl lyng_vm::vm::Vm>::environment_for_slot_access
	ldur x8, [x29, #-144]
	ldur w27, [x29, #-136]
	cmp x8, x26
	b.ne LBB358_44
	mov x0, x23
	mov x1, x27
	mov x2, x21
	bl lyng_env::agent::environments::<impl lyng_env::agent::Agent>::environment_slot
	tbz w0, #0, LBB358_19
	mov x25, x1
	mov x8, #2
	movk x8, #9, lsl #32
	movk x8, #32760, lsl #48
	cmp x1, x8
	b.ne LBB358_93
	mov x0, x23
	mov w1, #3
	bl lyng_ops::errors::error_value
	mov x25, x0
	mov w27, #0
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
	ldur w9, [x29, #-132]
	ldur x25, [x29, #-128]
	ldur q0, [x29, #-120]
	str q0, [sp, #80]
	ldur x10, [x29, #-104]
	str x10, [sp, #96]
LBB358_45:
	mov x10, #-9223372036854775808
	cmp x8, x10
	b.ne LBB358_20
	cbnz w27, LBB358_20
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
	sub x0, x29, #144
	mov x2, x23
	mov x3, x25
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-144]
	ldurb w27, [x29, #-136]
	cmp x8, x26
	b.ne LBB358_95
	tbz w27, #0, LBB358_100
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
	b LBB358_94
LBB358_57:
Lloh921:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
Lloh922:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov x0, x6
	mov x2, x1
	bl core::slice::index::slice_index_fail
	brk #0x1
LBB358_59:
	sub x0, x29, #144
	mov w4, #1
	mov x5, x22
	bl lyng_vm::vm::dispatch::decode_abx_operands_wide
	ldur x8, [x29, #-144]
	cmp x8, x26
	b.ne LBB358_24
	ldurh w10, [x29, #-128]
	ldp w8, w24, [x29, #-136]
	ldur w9, [x29, #-124]
	stp w9, w10, [sp, #72]
	ldr x13, [x20, #80]
	ldr x9, [x13, #80]
	sub w28, w22, #1
	cmp x9, x28
	b.hi LBB358_5
LBB358_61:
	sub x9, x26, #29
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
	mov x1, x27
	bl lyng_env::agent::environments::<impl lyng_env::agent::Agent>::global_environment_object
	mov x21, x0
	cbz w0, LBB358_19
	ldr x1, [sp, #64]
	cbz w24, LBB358_75
	ldr x8, [x1, #104]
	cmp x8, x28
	b.ls LBB358_75
	ldr x8, [x1, #96]
	add x9, x8, x28, lsl #5
	ldr x10, [x9, #16]
	sub w8, w24, #1
	cmp x10, x8
	b.ls LBB358_75
	ldr x9, [x9, #8]
	mov w10, #1216
	umaddl x0, w8, w10, x9
	ldr x8, [x0]
	cmp x8, #10
	ccmp x8, #6, #0, ne
	b.eq LBB358_104
LBB358_75:
	ldr x8, [x23, #224]
	sub w9, w21, #1
	lsr x10, x9, #6
	cmp x10, x8
	b.hs LBB358_85
	ldr x11, [x23, #216]
	and x12, x9, #0x3f
	ldr x11, [x11, x10, lsl #3]
	mov w13, #80
	umaddl x13, w12, w13, x11
	mov x12, x13
	ldr w11, [x12], #8
	cmp w11, #1
	b.ne LBB358_85
	ldr w15, [x13, #52]
	cbz w15, LBB358_85
	cbz w24, LBB358_90
	ldr x11, [x1, #104]
	cmp x11, x28
	b.ls LBB358_87
	ldr x14, [x1, #96]
	add x16, x14, x28, lsl #5
	ldr x17, [x16, #16]
	sub w14, w24, #1
	cmp x17, x14
	b.ls LBB358_87
	ldr x16, [x16, #8]
	mov w17, #1216
	umaddl x0, w14, w17, x16
	ldr x14, [x0]
	cmp x14, #10
	ccmp x14, #6, #0, ne
	b.ne LBB358_87
	ldr x14, [x0, #1016]
	lsr x16, x14, #32
	cmp x16, #0
	ccmp w15, w16, #0, ne
	b.eq LBB358_114
	ldr x14, [x0, #1024]
	lsr x16, x14, #32
	cmp x16, #0
	ccmp w15, w16, #0, ne
	b.ne LBB358_140
	mov w15, #1
	b LBB358_115
LBB358_85:
	cbz w24, LBB358_90
	ldr x11, [x1, #104]
LBB358_87:
	cmp x11, x28
	b.ls LBB358_90
LBB358_88:
	ldr x11, [x1, #96]
	add x12, x11, x28, lsl #5
	ldr x13, [x12, #16]
	sub w11, w24, #1
	cmp x13, x11
	b.ls LBB358_90
	ldr x12, [x12, #8]
	mov w13, #1216
	umaddl x0, w11, w13, x12
	ldr x11, [x0]
	cmp x11, #10
	ccmp x11, #6, #0, ne
	b.eq LBB358_109
LBB358_90:
	mov x0, x1
	mov x1, x23
	mov x2, x22
	mov x3, x24
	mov x4, x21
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_named_property_load_inline_cache_hit
	tbz w0, #0, LBB358_96
	mov x25, x1
LBB358_93:
	ldr w8, [x20, #20]
	ldr x9, [x20, #80]
	ldr w10, [sp, #76]
	add w8, w8, w10
	ldr x9, [x9, #24]
	str x25, [x9, w8, uxtw #3]
	ldr w8, [x20, #56]
	ldr w9, [sp, #72]
	add w8, w8, w9
	str w8, [x20, #56]
	ldr x9, [x20, #128]
	ldr x9, [x9, #48]
	ldrb w8, [x9, w8, uxtw]
LBB358_94:
Lloh923:
	adrp x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
Lloh924:
	add x9, x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x8, [x9, x8, lsl #3]
	stp x26, x8, [x19]
	b LBB358_64
LBB358_95:
	ldurb w9, [x29, #-133]
	ldurh w10, [x29, #-135]
	orr w10, w10, w9, lsl #16
	ldur w9, [x29, #-132]
	ldur x25, [x29, #-128]
	ldur q0, [x29, #-120]
	str q0, [sp, #112]
	ldur x11, [x29, #-104]
	b LBB358_21
LBB358_96:
	ldp x9, x1, [sp, #56]
	ldr w8, [sp, #20]
	stp w8, w9, [sp]
	sub x0, x29, #144
	mov x2, x23
	ldp x4, x3, [sp, #40]
	ldp x6, x5, [sp, #24]
	mov x7, x20
	bl lyng_vm::vm::names::<impl lyng_vm::vm::Vm>::get_global_property_binding_with_context
	ldp x8, x27, [x29, #-144]
	ldur x25, [x29, #-128]
	cmp x8, x26
	b.ne LBB358_101
	tbz w27, #0, LBB358_102
	ldp x5, x0, [sp, #56]
	mov x1, x23
	mov x2, x22
	mov x3, x24
	mov x4, x21
	mov w6, #0
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_named_property_slow_path
	b LBB358_93
LBB358_100:
	mov w27, #0
	mov w10, #0
	mov x8, #-9223372036854775808
	b LBB358_22
LBB358_101:
	ldur q0, [x29, #-120]
	str q0, [sp, #80]
	ldur x9, [x29, #-104]
	str x9, [sp, #96]
	lsr x9, x27, #32
	b LBB358_45
LBB358_102:
	mov x0, x23
	mov w1, #3
	bl lyng_ops::errors::error_value
	mov x25, x0
	mov w9, #0
	mov w27, #0
	mov x8, #-9223372036854775808
	b LBB358_45
LBB358_104:
	ldr x8, [x0, #968]
	cbz x8, LBB358_75
	ldr x11, [x23, #224]
	sub w9, w21, #1
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
	cbz w12, LBB358_122
	cmp w12, w13
	b LBB358_123
LBB358_109:
	ldr x12, [x0, #984]
	cbz x12, LBB358_90
	cmp x10, x8
	b.hs LBB358_90
	ldr x11, [x23, #216]
	and x9, x9, #0x3f
	ldr x10, [x11, x10, lsl #3]
	mov w13, #80
	umaddl x15, w9, w13, x10
	ldr w9, [x15]
	cmp w9, #1
	b.ne LBB358_90
	ldr x9, [x0, #992]
	ldr x14, [x0, #1000]
	ldr x10, [x0, #1008]
	ldp w13, w16, [x15, #48]
	ldr x15, [x15, #40]
	cmp w13, #0
	cbz w16, LBB358_130
	ccmp w16, w12, #0, ne
	b LBB358_131
LBB358_114:
	mov x15, #0
LBB358_115:
	add x15, x0, x15, lsl #3
	ldr x16, [x15, #1032]
	ldr w15, [x13, #56]
	ldr x13, [x13, #40]
	ldp q0, q1, [x12]
	stp q0, q1, [x29, #-144]
	cmp x13, x16
	b.ne LBB358_140
	tbnz w14, #31, LBB358_139
	cbz w15, LBB358_140
	ldr x13, [x23, #640]
	sub w12, w15, #1
	cmp x13, x12
	b.ls LBB358_140
	ldr x13, [x23, #632]
	mov w15, #24
	umaddl x12, w12, w15, x13
	ldrb w13, [x12, #19]
	cmp w13, #1
	b.ne LBB358_140
	ldr x15, [x12, #8]
	and x13, x14, #0x3fffffff
	cmp x15, x13
	b.ls LBB358_140
	ldr x8, [x12]
	add x8, x8, x13, lsl #3
	ldr x25, [x8]
	b LBB358_153
LBB358_122:
	cmp w13, #0
LBB358_123:
	ccmp x11, x9, #0, eq
	ldr x1, [sp, #64]
	b.ne LBB358_75
	and x9, x8, #0x3fffffff
	tbnz w8, #31, LBB358_136
	ldr x1, [sp, #64]
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
	b LBB358_138
LBB358_130:
	ccmp w12, #0, #0, ne
LBB358_131:
	ccmp x15, x14, #0, eq
	ldr x1, [sp, #64]
	b.ne LBB358_90
	sub w12, w13, #1
	lsr x13, x12, #6
	cmp x13, x8
	b.hs LBB358_90
	and x8, x12, #0x3f
	ldr x11, [x11, x13, lsl #3]
	mov w12, #80
	umaddl x13, w8, w12, x11
	ldr w8, [x13]
	cmp w8, #1
	b.ne LBB358_90
	ldp w12, w11, [x13, #52]
	ldr x8, [x13, #40]
	ldur q0, [x13, #8]
	ldur q1, [x13, #24]
	stp q0, q1, [x29, #-144]
	lsr x13, x9, #32
	cmp x13, #0
	csel w13, w13, wzr, ne
	cbz w12, LBB358_142
	cmp w12, w13
	ccmp x8, x10, #0, eq
	ldr x1, [sp, #64]
	b.ne LBB358_90
	b LBB358_144
LBB358_136:
	cmp x9, #4
	ldr x1, [sp, #64]
	b.hs LBB358_75
	add x8, sp, #144
LBB358_138:
	ldr x25, [x8, x9, lsl #3]
	b LBB358_153
LBB358_139:
	tst w14, #0x3ffffffc
	b.eq LBB358_141
LBB358_140:
	ldr x1, [sp, #64]
	cmp x11, x28
	b.hi LBB358_88
	b LBB358_90
LBB358_141:
	and x8, x14, #0x3fffffff
	sub x9, x29, #144
	add x8, x9, x8, lsl #3
	ldr x25, [x8]
	b LBB358_153
LBB358_142:
	ldr x1, [sp, #64]
	cbnz w13, LBB358_90
	cmp x8, x10
	b.ne LBB358_90
LBB358_144:
	and x8, x9, #0x3fffffff
	tbnz w9, #31, LBB358_150
	ldr x1, [sp, #64]
	cbz w11, LBB358_90
	ldr x10, [x23, #640]
	sub w9, w11, #1
	cmp x10, x9
	b.ls LBB358_90
	ldr x10, [x23, #632]
	mov w11, #24
	umaddl x9, w9, w11, x10
	ldrb w10, [x9, #19]
	cmp w10, #1
	b.ne LBB358_90
	ldr x10, [x9, #8]
	cmp x8, x10
	b.hs LBB358_90
	ldr x9, [x9]
	b LBB358_152
LBB358_150:
	cmp x8, #3
	ldr x1, [sp, #64]
	b.hi LBB358_90
	sub x9, x29, #144
LBB358_152:
	ldr x25, [x9, x8, lsl #3]
LBB358_153:
	bl lyng_vm::vm::feedback::FeedbackSiteState::record_execution
	ldr x8, [sp, #64]
	ldp x0, x1, [x8, #120]
	mov x2, x22
	bl lyng_vm::vm::tiering::<impl lyng_vm::vm::Vm>::observe_tier_feedback_event
	b LBB358_93
	bl core::panicking::panic_cannot_unwind
	bl core::panicking::panic_cannot_unwind
	bl core::panicking::panic_cannot_unwind
	.loh AdrpAdd	Lloh921, Lloh922
	.loh AdrpAdd	Lloh923, Lloh924
