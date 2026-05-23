.section __TEXT,__text,regular,pure_instructions
	.p2align	2
lyng_vm::vm::dispatch_handlers::names::op_store_or_assign_global:
Lfunc_begin370:
	.cfi_startproc
	.cfi_personality 155, _rust_eh_personality
	.cfi_lsda 16, Lexception172
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
	sub sp, sp, #656
	mov x20, x1
	ldr w21, [x1, #4]
	ldr w6, [x1, #56]
	ldrb w3, [x1, #148]
	mov w8, #152
	strb w8, [x1, #148]
	ldr x9, [x1, #128]
	ldr x1, [x9, #56]
	subs x8, x1, x6
	b.lo LBB370_113
	mov x19, x0
	add x25, sp, #192
	mov x26, #33
	movk x26, #32768, lsl #48
	ldr x9, [x9, #48]
	add x1, x9, x6
	cmp w3, #152
	b.ne LBB370_114
	and x9, x8, #0x7ffffffffffffffe
	cmp x8, #4
	ccmp x9, #4, #4, hs
	b.eq LBB370_17
	ldrh w23, [x1, #4]
	cbz w23, LBB370_17
	ldrb w9, [x1, #1]
	ldrh w8, [x1, #2]
	mov w14, #6
	ldr x15, [x20, #80]
	ldr x10, [x15, #80]
	sub w24, w21, #1
	cmp x10, x24
	b.ls LBB370_116
LBB370_5:
	ldr x17, [x15, #72]
	ldr x11, [x17, x24, lsl #3]
	cbz x11, LBB370_116
	ldr x13, [x11, #80]
	mov w12, w8
	sub x10, x26, #3
	cmp x13, x12
	b.ls LBB370_19
	ldr x13, [x11, #72]
	add x13, x13, x12, lsl #4
	ldr w12, [x13]
	cmp w12, #4
	b.eq LBB370_19
	ldr w16, [x13, #4]
	cmp w12, #2
	b.ne LBB370_20
	ldr x8, [x11, #464]
	cmp x8, x16
	b.ls LBB370_12
	ldr x8, [x11, #456]
	add x8, x8, x16, lsl #3
	ldr w10, [x8, #16]!
	cmp w10, #1
	b.ne LBB370_12
	ldr w16, [x8, #4]
LBB370_12:
	stp x16, x15, [sp, #80]
	ldr w8, [x20, #20]
	add w8, w8, w9
	ldr x9, [x15, #24]
	ldr x10, [x9, w8, uxtw #3]
	ldp x9, x11, [x20, #88]
	str x9, [sp, #104]
	ldp x9, x8, [x20, #104]
	stp x11, x9, [sp, #40]
	str x8, [sp, #56]
	ldr x8, [x20, #120]
	stp x8, x10, [sp, #64]
	str w14, [sp, #100]
	tbz w2, #0, LBB370_21
	str x17, [sp, #24]
	ldr w28, [x20, #12]
	mov x8, #33
	movk x8, #32768, lsl #48
	add x26, sp, #392
	sub x22, x29, #200
	sub x27, x8, #27
	str w28, [sp, #32]
LBB370_14:
	add x8, sp, #392
	ldr x0, [sp, #104]
	mov x1, x28
	bl lyng_env::agent::environments::<impl lyng_env::agent::Agent>::environment
	ldr x16, [sp, #392]
	cmp x16, x27
	b.eq LBB370_26
	ldur q0, [x26, #88]
	ldur q1, [x26, #104]
	stp q0, q1, [sp, #336]
	ldur q0, [x26, #120]
	str q0, [sp, #368]
	ldur q0, [x26, #56]
	ldur q1, [x26, #72]
	stp q0, q1, [sp, #304]
	ldp q0, q1, [sp, #336]
	stur q0, [x22, #43]
	stur q1, [x22, #59]
	ldr q0, [sp, #368]
	stur q0, [x22, #75]
	ldp q0, q1, [sp, #304]
	stur q0, [x22, #11]
	ldr x17, [sp, #400]
	ldr w28, [sp, #408]
	ldr w12, [sp, #412]
	ldr x0, [sp, #416]
	ldr w9, [sp, #424]
	ldr w10, [sp, #428]
	ldr w11, [sp, #432]
	ldrb w8, [sp, #436]
	ldur x13, [x26, #45]
	stur x13, [x29, #-200]
	ldur w13, [x26, #52]
	add x14, sp, #392
	stur w13, [x14, #151]
	ldur x13, [x26, #136]
	str x13, [sp, #384]
	stur x13, [x22, #91]
	stur q1, [x22, #27]
	tbz x16, #63, LBB370_49
	cbnz w28, LBB370_14
	b LBB370_25
LBB370_17:
	sub x8, x26, #12
	str x8, [sp, #392]
	str w21, [sp, #400]
	str w6, [sp, #404]
LBB370_18:
	ldur q0, [x25, #200]
	ldur q1, [x25, #216]
	stp q0, q1, [x19]
	ldur q0, [x25, #232]
	str q0, [x19, #32]
	b LBB370_119
LBB370_19:
	mov x9, #0
	b LBB370_117
LBB370_20:
	ldr x11, [x13, #8]
	orr x9, x12, x16, lsl #32
	b LBB370_118
LBB370_21:
	ldr w28, [x20, #12]
	add x27, sp, #392
	sub x22, x29, #200
	mov x8, #33
	movk x8, #32768, lsl #48
	sub x26, x8, #27
	str w28, [sp, #32]
LBB370_22:
	add x8, sp, #392
	ldr x0, [sp, #104]
	mov x1, x28
	bl lyng_env::agent::environments::<impl lyng_env::agent::Agent>::environment
	ldr x16, [sp, #392]
	cmp x16, x26
	b.eq LBB370_26
	ldur q0, [x27, #88]
	ldur q1, [x27, #104]
	stp q0, q1, [sp, #336]
	ldur q0, [x27, #120]
	str q0, [sp, #368]
	ldur q0, [x27, #56]
	ldur q1, [x27, #72]
	stp q0, q1, [sp, #304]
	ldp q0, q1, [sp, #336]
	stur q0, [x22, #43]
	stur q1, [x22, #59]
	ldr q0, [sp, #368]
	stur q0, [x22, #75]
	ldp q0, q1, [sp, #304]
	stur q0, [x22, #11]
	ldr x17, [sp, #400]
	ldr w28, [sp, #408]
	ldr w12, [sp, #412]
	ldr x0, [sp, #416]
	ldr w9, [sp, #424]
	ldr w10, [sp, #428]
	ldr w11, [sp, #432]
	ldrb w8, [sp, #436]
	ldur x13, [x27, #45]
	stur x13, [x29, #-200]
	ldur w13, [x27, #52]
	add x14, sp, #392
	stur w13, [x14, #151]
	ldur x13, [x27, #136]
	str x13, [sp, #384]
	stur x13, [x22, #91]
	stur q1, [x22, #27]
	tbz x16, #63, LBB370_42
	cbnz w28, LBB370_22
LBB370_25:
	ldr w28, [sp, #32]
LBB370_26:
	mov x25, #33
	movk x25, #32768, lsl #48
	sub x8, x25, #24
	str x8, [sp, #112]
	stp w28, wzr, [sp, #120]
LBB370_27:
	ldr x9, [sp, #112]
	cmp x9, x25
	ldr w10, [sp, #100]
	b.ne LBB370_30
	ldr w8, [x20, #56]
	add w8, w8, w10
	str w8, [x20, #56]
	ldr x9, [x20, #128]
	ldr x9, [x9, #48]
	ldrb w8, [x9, w8, uxtw]
LBB370_29:
Lloh937:
	adrp x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
Lloh938:
	add x9, x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x8, [x9, x8, lsl #3]
	stp x25, x8, [x19]
	b LBB370_119
LBB370_30:
	ldr w8, [sp, #120]
	mov x10, #-9223372036854775808
	cmp x9, x10
	b.ne LBB370_41
	cbnz w8, LBB370_41
	ldp x22, x2, [x20, #80]
	ldr x23, [x20, #136]
	ldr x21, [sp, #128]
	cbz x23, LBB370_35
	sub x8, x23, #1
	ldr x9, [x22, #56]
	cmp x8, x9
	b.hs LBB370_35
	ldr x9, [x22, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB370_35:
	add x0, sp, #392
	mov x1, x22
	mov x3, x21
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
	ldr x9, [sp, #392]
	ldrb w8, [sp, #400]
	cmp x9, x25
	b.ne LBB370_58
	tbz w8, #0, LBB370_101
	ldr w8, [x22, #1640]
	add w8, w8, #1
	str w8, [x22, #1640]
	cbz x23, LBB370_40
	sub x8, x23, #1
	ldr x9, [x22, #56]
	cmp x8, x9
	b.hs LBB370_40
	ldr x9, [x22, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x8]
	str q0, [x20]
	ldp q0, q1, [x8, #48]
	ldp q3, q2, [x8, #16]
	stp q0, q1, [x20, #48]
	stp q3, q2, [x20, #16]
LBB370_40:
	ldr x8, [x20, #128]
	ldr x8, [x8, #48]
	ldr w9, [x20, #56]
	ldrb w8, [x8, x9]
	b LBB370_29
LBB370_41:
	lsr w10, w8, #8
	ldr w11, [sp, #124]
	ldr x21, [sp, #128]
	ldur q0, [sp, #136]
	str q0, [sp, #160]
	ldr x12, [sp, #152]
	b LBB370_59
LBB370_42:
	add x15, sp, #392
	ldp q0, q1, [x15, #208]
	stp q0, q1, [sp, #256]
	ldur w13, [x15, #239]
	stur w13, [x25, #95]
	ldp q0, q1, [x15, #144]
	stp q0, q1, [sp, #192]
	ldp q1, q0, [x15, #176]
	stp q1, q0, [sp, #224]
	ldr x13, [sp, #192]
	ldur w14, [sp, #199]
	str w14, [sp, #444]
	stur x13, [x25, #245]
	ldur q0, [sp, #203]
	ldur q1, [sp, #219]
	stur q1, [x15, #72]
	stur q0, [x15, #56]
	ldur q0, [sp, #235]
	ldur q1, [x25, #59]
	ldur q2, [x25, #75]
	ldur x13, [x25, #91]
	str x13, [sp, #528]
	stur q2, [x15, #120]
	stur q1, [x15, #104]
	add x14, sp, #392
	stur q0, [x15, #88]
	stp x16, x17, [sp, #392]
	str w28, [sp, #408]
	str w12, [sp, #412]
	stp x0, x17, [sp, #24]
	str x0, [sp, #416]
	sub x12, x17, #4
	str w9, [sp, #424]
	str w10, [sp, #428]
	str w11, [sp, #432]
	ldr x9, [sp, #408]
	add x9, x9, x9, lsl #1
	lsl x9, x9, #2
	mov x10, x17
	strb w8, [sp, #436]
	mov x25, #33
	movk x25, #32768, lsl #48
	ldp x26, x22, [sp, #80]
	mov x27, x16
LBB370_43:
	cbz x9, LBB370_61
	ldp w3, w8, [x10], #12
	add x12, x12, #12
	sub x9, x9, #12
	cmp w8, w26
	ccmp w3, #0, #4, eq
	b.eq LBB370_43
	ldr w28, [x12]
LBB370_46:
	sub x0, x29, #200
	mov x1, x22
	ldr x2, [sp, #104]
	mov w4, #0
	mov x5, x28
	bl lyng_vm::vm::loop_iteration::<impl lyng_vm::vm::Vm>::environment_for_slot_access
	ldur x8, [x29, #-200]
	ldur w3, [x29, #-192]
	cmp x8, x25
	b.ne LBB370_102
	add x0, sp, #112
	mov x1, x22
	ldr x2, [sp, #104]
	mov x4, x28
	ldr x5, [sp, #72]
	bl lyng_vm::vm::values::<impl lyng_vm::vm::Vm>::write_environment_slot
	b LBB370_123
LBB370_49:
	add x15, sp, #392
	ldp q0, q1, [x15, #208]
	stp q0, q1, [sp, #256]
	ldur w13, [x15, #239]
	stur w13, [x25, #95]
	ldp q0, q1, [x15, #144]
	stp q0, q1, [sp, #192]
	ldp q1, q0, [x15, #176]
	stp q1, q0, [sp, #224]
	ldr x13, [sp, #192]
	ldur w14, [sp, #199]
	str w14, [sp, #444]
	stur x13, [x15, #45]
	ldur q0, [sp, #203]
	ldur q1, [sp, #219]
	stur q1, [x15, #72]
	stur q0, [x15, #56]
	ldur q0, [sp, #235]
	ldur q1, [x25, #59]
	ldur q2, [x25, #75]
	ldur x13, [x25, #91]
	str x13, [sp, #528]
	stur q2, [x15, #120]
	stur q1, [x15, #104]
	add x14, sp, #392
	stur q0, [x15, #88]
	stp x16, x17, [sp, #392]
	str w28, [sp, #408]
	str w12, [sp, #412]
	str x0, [sp, #16]
	str x0, [sp, #416]
	sub x12, x17, #4
	str w9, [sp, #424]
	str w10, [sp, #428]
	str w11, [sp, #432]
	ldr x9, [sp, #408]
	add x9, x9, x9, lsl #1
	lsl x9, x9, #2
	str x17, [sp, #32]
	mov x10, x17
	strb w8, [sp, #436]
	mov x25, #33
	movk x25, #32768, lsl #48
	ldp x26, x22, [sp, #80]
	mov x27, x16
LBB370_50:
	cbz x9, LBB370_79
	ldp w3, w8, [x10], #12
	add x12, x12, #12
	sub x9, x9, #12
	cmp w8, w26
	ccmp w3, #0, #4, eq
	b.eq LBB370_50
	ldr w28, [x12]
LBB370_53:
	sub x0, x29, #200
	mov x1, x22
	ldr x2, [sp, #104]
	mov w4, #0
	mov x5, x28
	bl lyng_vm::vm::loop_iteration::<impl lyng_vm::vm::Vm>::environment_for_slot_access
	ldur x8, [x29, #-200]
	ldur w3, [x29, #-192]
	cmp x8, x25
	b.ne LBB370_103
	ldr x8, [x22, #80]
	cmp x8, x24
	b.ls LBB370_120
	ldr x8, [x22, #72]
	ldr x8, [x8, x24, lsl #3]
	cbz x8, LBB370_121
	ldrh w8, [x8, #340]
	b LBB370_121
LBB370_58:
	ldrb w10, [sp, #403]
	add x12, sp, #392
	ldurh w11, [x12, #9]
	orr w10, w11, w10, lsl #16
	ldr w11, [sp, #404]
	ldur q0, [x12, #24]
	str q0, [sp, #160]
	ldr x21, [sp, #408]
	ldr x12, [sp, #432]
LBB370_59:
	str x12, [sp, #176]
LBB370_60:
	bfi w8, w10, #8, #24
	str x9, [x19]
	stp w8, w11, [x19, #8]
	str x21, [x19, #16]
	ldr q0, [sp, #160]
	stur q0, [x19, #24]
	ldr x8, [sp, #176]
	str x8, [x19, #40]
	b LBB370_119
LBB370_61:
	ldr w8, [sp, #516]
	ldr x9, [sp, #104]
	ldr x9, [x9, #5144]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB370_70
	ldr x9, [sp, #104]
	ldr x9, [x9, #5136]
	add x8, x9, x8, lsl #5
	ldr x9, [x8]
	mov x10, #-9223372036854775808
	cmp x9, x10
	b.eq LBB370_70
	ldr x9, [x8, #16]
	cbz x9, LBB370_70
	mov x28, #0
	ldr w3, [sp, #512]
	ldr x8, [x8, #8]
	add x9, x8, x9, lsl #4
	b LBB370_66
LBB370_65:
	add x8, x8, #16
	add x28, x28, #1
	cmp x8, x9
	b.eq LBB370_70
LBB370_66:
	ldp w10, w11, [x8]
	cmp w10, #1
	ccmp w11, w26, #0, eq
	b.ne LBB370_65
	ldrb w10, [x8, #9]
	tbz w10, #0, LBB370_65
	lsr x8, x28, #32
	cbnz x8, LBB370_70
	cbnz w3, LBB370_46
LBB370_70:
	ldr w28, [sp, #520]
	ldp x0, x1, [x22, #96]
	cbz w23, LBB370_74
	cmp x1, x24
	b.ls LBB370_74
	add x9, x0, x24, lsl #5
	ldr x10, [x9, #16]
	sub w8, w23, #1
	cmp x10, x8
	b.ls LBB370_74
	ldr x9, [x9, #8]
	mov w10, #1096
	umaddl x9, w8, w10, x9
	ldr x8, [x9]
	cmp x8, #10
	ccmp x8, #6, #0, ne
	b.eq LBB370_145
LBB370_74:
	ldr x2, [sp, #104]
	mov x3, x21
	mov x4, x23
	mov x5, x28
	ldr x6, [sp, #72]
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_named_property_store_inline_cache
	and w8, w0, #0xff
	cmp w8, #2
	b.ne LBB370_104
	mov x8, #21474836480
	movk x8, #32760, lsl #48
	ldr x9, [sp, #104]
	stp x22, x9, [x29, #-200]
	ldp x10, x9, [sp, #40]
	stp x10, x9, [x29, #-184]
	ldp x10, x9, [sp, #56]
	stp x10, x9, [x29, #-168]
	stur x20, [x29, #-152]
	add x0, sp, #192
	sub x1, x29, #200
	orr x6, x28, x8
	mov x2, x28
	mov w3, #1
	mov x4, x26
	ldr x5, [sp, #72]
	bl lyng_ops::proxy::set
	ldr x8, [sp, #192]
	cmp x8, x25
	b.ne LBB370_122
	mov x0, x22
	ldr x1, [sp, #104]
	mov x2, x21
	mov x3, x23
	mov x4, x28
	mov x5, x26
	mov w6, #1
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_named_property_slow_path
	b LBB370_105
LBB370_79:
	ldr w8, [sp, #516]
	ldr x9, [sp, #104]
	ldr x9, [x9, #5144]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB370_88
	ldr x9, [sp, #104]
	ldr x9, [x9, #5136]
	add x8, x9, x8, lsl #5
	ldr x9, [x8]
	mov x10, #-9223372036854775808
	cmp x9, x10
	b.eq LBB370_88
	ldr x9, [x8, #16]
	cbz x9, LBB370_88
	mov x28, #0
	ldr w3, [sp, #512]
	ldr x8, [x8, #8]
	add x9, x8, x9, lsl #4
	b LBB370_84
LBB370_83:
	add x8, x8, #16
	add x28, x28, #1
	cmp x8, x9
	b.eq LBB370_88
LBB370_84:
	ldp w10, w11, [x8]
	cmp w10, #1
	ccmp w11, w26, #0, eq
	b.ne LBB370_83
	ldrb w10, [x8, #9]
	tbz w10, #0, LBB370_83
	lsr x8, x28, #32
	cbnz x8, LBB370_88
	cbnz w3, LBB370_53
LBB370_88:
	ldr w28, [sp, #520]
	ldp x0, x1, [x22, #96]
	cbz w23, LBB370_92
	cmp x1, x24
	b.ls LBB370_92
	add x9, x0, x24, lsl #5
	ldr x10, [x9, #16]
	sub w8, w23, #1
	cmp x10, x8
	b.ls LBB370_92
	ldr x9, [x9, #8]
	mov w10, #1096
	umaddl x9, w8, w10, x9
	ldr x8, [x9]
	cmp x8, #10
	ccmp x8, #6, #0, ne
	b.eq LBB370_150
LBB370_92:
	ldr x2, [sp, #104]
	mov x3, x21
	mov x4, x23
	mov x5, x28
	ldr x6, [sp, #72]
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_named_property_store_inline_cache
	and w8, w0, #0xff
	cmp w8, #2
	b.ne LBB370_106
	ldr x8, [sp, #104]
	stp x22, x8, [x29, #-200]
	ldp x9, x8, [sp, #40]
	stp x9, x8, [x29, #-184]
	ldp x9, x8, [sp, #56]
	stp x9, x8, [x29, #-168]
	stur x20, [x29, #-152]
	add x0, sp, #192
	sub x1, x29, #200
	mov x2, x28
	mov w3, #1
	mov x4, x26
	bl lyng_ops::proxy::has_property
	ldr x9, [sp, #192]
	ldrb w8, [sp, #200]
	cmp x9, x25
	b.ne LBB370_133
	tbnz w8, #0, LBB370_126
	ldr x9, [x22, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB370_126
	ldr x9, [x22, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB370_126
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB370_126
	ldr x0, [sp, #104]
	mov w1, #3
	bl lyng_ops::errors::error_value
	b LBB370_110
LBB370_101:
	mov w8, #0
	mov w10, #0
	mov x9, #-9223372036854775808
	b LBB370_60
LBB370_102:
	add x9, sp, #392
	ldur q0, [x9, #156]
	stur q0, [sp, #124]
	ldur q0, [x9, #172]
	stur q0, [sp, #140]
	ldur w9, [x29, #-156]
	str w9, [sp, #156]
	str x8, [sp, #112]
	str w3, [sp, #120]
	ldr x8, [sp, #424]
	cbnz x8, LBB370_124
	b LBB370_138
LBB370_103:
	add x9, sp, #392
	ldur q0, [x9, #156]
	stur q0, [sp, #124]
	ldur q0, [x9, #172]
	stur q0, [sp, #140]
	ldur w9, [x29, #-156]
	str w9, [sp, #156]
	str x8, [sp, #112]
	str w3, [sp, #120]
	ldr x8, [sp, #424]
	cbnz x8, LBB370_135
	b LBB370_138
LBB370_104:
	mov x0, x22
	mov x1, x21
	mov x2, x23
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
LBB370_105:
	str x25, [sp, #112]
	ldr x8, [sp, #424]
	cbnz x8, LBB370_124
	b LBB370_138
LBB370_106:
	tbnz w8, #0, LBB370_111
	ldr x8, [sp, #24]
	ldr x8, [x8, x24, lsl #3]
	cbz x8, LBB370_111
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB370_111
	ldr x0, [sp, #104]
	mov w1, #5
	bl lyng_ops::errors::error_value
LBB370_110:
	mov x8, #-9223372036854775808
	str x8, [sp, #112]
	str wzr, [sp, #120]
	str x0, [sp, #128]
	ldr x8, [sp, #424]
	cbnz x8, LBB370_135
	b LBB370_138
LBB370_111:
	mov x0, x22
	mov x1, x21
	mov x2, x23
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
LBB370_112:
	str x25, [sp, #112]
	ldr x8, [sp, #424]
	cbnz x8, LBB370_135
	b LBB370_138
LBB370_113:
Lloh939:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.45@PAGE
Lloh940:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.45@PAGEOFF
	mov x0, x6
	mov x2, x1
	bl core::slice::index::slice_index_fail
LBB370_114:
	mov x22, x2
	add x0, sp, #392
	mov x2, x8
	mov w4, #1
	mov x5, x21
	bl lyng_vm::vm::dispatch::decode_abx_operands_wide
	ldr x8, [sp, #392]
	cmp x8, x26
	b.ne LBB370_18
	ldrh w9, [sp, #408]
	ldr w8, [sp, #400]
	ldr w23, [sp, #404]
	ldr w14, [sp, #412]
	mov x2, x22
	ldr x15, [x20, #80]
	ldr x10, [x15, #80]
	sub w24, w21, #1
	cmp x10, x24
	b.hi LBB370_5
LBB370_116:
	sub x10, x26, #29
	mov x9, x21
LBB370_117:
LBB370_118:
	stp x10, x9, [x19]
	str x11, [x19, #16]
	stp w21, w8, [x19, #24]
LBB370_119:
	add sp, sp, #656
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
LBB370_120:
	.cfi_restore_state
	mov w8, #0
LBB370_121:
	add x0, sp, #112
	and w6, w8, #0x1
	mov x1, x22
	ldr x2, [sp, #104]
	mov x4, x28
	ldr x5, [sp, #72]
	bl lyng_vm::vm::values::<impl lyng_vm::vm::Vm>::assign_environment_slot
	b LBB370_134
LBB370_122:
	ldrb w9, [sp, #200]
	ldur q0, [sp, #201]
	stur q0, [sp, #121]
	ldur q0, [sp, #217]
	stur q0, [sp, #137]
	ldr x10, [sp, #232]
	str x10, [sp, #152]
	str x8, [sp, #112]
	strb w9, [sp, #120]
LBB370_123:
	ldr x8, [sp, #424]
	cbz x8, LBB370_138
LBB370_124:
	lsl x9, x8, #2
	add x9, x9, #11
	and x9, x9, #0xfffffffffffffff8
	add x8, x8, x9
	adds x1, x8, #9
	b.eq LBB370_138
	ldr x8, [sp, #24]
	b LBB370_137
LBB370_126:
	mov x8, #21474836480
	movk x8, #32760, lsl #48
	ldr x9, [sp, #104]
	stp x22, x9, [x29, #-200]
	ldp x10, x9, [sp, #40]
	stp x10, x9, [x29, #-184]
	ldp x10, x9, [sp, #56]
	stp x10, x9, [x29, #-168]
	stur x20, [x29, #-152]
	add x0, sp, #192
	sub x1, x29, #200
	orr x6, x28, x8
	mov x2, x28
	mov w3, #1
	mov x4, x26
	ldr x5, [sp, #72]
	bl lyng_ops::proxy::set
	ldr x9, [sp, #192]
	ldrb w8, [sp, #200]
	cmp x9, x25
	b.ne LBB370_133
	tbnz w8, #0, LBB370_143
	ldr x9, [x22, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB370_143
	ldr x9, [x22, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB370_143
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB370_143
	ldr x0, [sp, #104]
	mov w1, #5
	bl lyng_ops::errors::error_value
	b LBB370_110
LBB370_133:
	ldur q0, [sp, #201]
	stur q0, [sp, #121]
	ldur q0, [sp, #217]
	stur q0, [sp, #137]
	ldr x10, [sp, #232]
	str x10, [sp, #152]
	str x9, [sp, #112]
	strb w8, [sp, #120]
LBB370_134:
	ldr x8, [sp, #424]
	cbz x8, LBB370_138
LBB370_135:
	lsl x9, x8, #2
	add x9, x9, #11
	and x9, x9, #0xfffffffffffffff8
	add x8, x8, x9
	adds x1, x8, #9
	b.eq LBB370_138
	ldr x8, [sp, #16]
LBB370_137:
	sub x0, x8, x9
	mov w2, #8
	bl __rustc::__rust_dealloc
LBB370_138:
	cbz x27, LBB370_140
	add x8, x27, x27, lsl #1
	lsl x1, x8, #2
	ldr x0, [sp, #32]
	mov w2, #4
	bl __rustc::__rust_dealloc
LBB370_140:
	ldr x9, [sp, #472]
	cbz x9, LBB370_27
	lsl x8, x9, #2
	add x8, x8, #11
	and x8, x8, #0xfffffffffffffff8
	add x9, x9, x8
	adds x1, x9, #9
	b.eq LBB370_27
	ldr x9, [sp, #464]
	sub x0, x9, x8
	mov w2, #8
	bl __rustc::__rust_dealloc
	b LBB370_27
LBB370_143:
	mov x0, x22
	ldr x1, [sp, #104]
	mov x2, x21
	mov x3, x23
	mov x4, x28
	mov x5, x26
	mov w6, #1
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_named_property_slow_path
	str x25, [sp, #112]
	add x0, sp, #392
	bl core::ptr::drop_in_place<lyng_env::environment_records::GlobalEnvironmentRecord>
	b LBB370_27
LBB370_145:
	ldr x8, [x9, #968]
	cbz x8, LBB370_74
	ldr x10, [sp, #104]
	ldr x12, [x10, #224]
	sub w10, w28, #1
	lsr x11, x10, #6
	cmp x11, x12
	b.hs LBB370_74
	ldr x12, [sp, #104]
	ldr x12, [x12, #216]
	and x10, x10, #0x3f
	ldr x11, [x12, x11, lsl #3]
	mov w12, #80
	umaddl x11, w10, w12, x11
	ldr w10, [x11]
	cmp w10, #1
	b.ne LBB370_74
	mov x24, x27
	ldr x10, [x9, #976]
	ldp w12, w9, [x11, #52]
	ldr x11, [x11, #40]
	lsr x13, x8, #32
	cmp x13, #0
	csel w13, w13, wzr, ne
	cbz w12, LBB370_155
	cmp w12, w13
	b LBB370_156
LBB370_150:
	ldr x8, [x9, #968]
	cbz x8, LBB370_92
	ldr x10, [sp, #104]
	ldr x12, [x10, #224]
	sub w10, w28, #1
	lsr x11, x10, #6
	cmp x11, x12
	b.hs LBB370_92
	ldr x12, [sp, #104]
	ldr x12, [x12, #216]
	and x10, x10, #0x3f
	ldr x11, [x12, x11, lsl #3]
	mov w12, #80
	umaddl x11, w10, w12, x11
	ldr w10, [x11]
	cmp w10, #1
	b.ne LBB370_92
	str x27, [sp, #8]
	ldr x10, [x9, #976]
	ldp w12, w9, [x11, #52]
	ldr x11, [x11, #40]
	lsr x13, x8, #32
	cmp x13, #0
	csel w13, w13, wzr, ne
	cbz w12, LBB370_161
	cmp w12, w13
	b LBB370_162
LBB370_155:
	cmp w13, #0
LBB370_156:
	ccmp x11, x10, #0, eq
	mov x25, #33
	movk x25, #32768, lsl #48
	ldp x26, x22, [sp, #80]
	mov x27, x24
	b.ne LBB370_74
	tbz w8, #30, LBB370_169
	tbnz w8, #31, LBB370_167
	ldp x26, x22, [sp, #80]
	mov x27, x24
	cbz w9, LBB370_74
	mov w10, #2
	mov x28, x9
	b LBB370_168
LBB370_161:
	cmp w13, #0
LBB370_162:
	ccmp x11, x10, #0, eq
	mov x25, #33
	movk x25, #32768, lsl #48
	ldp x26, x22, [sp, #80]
	ldr x27, [sp, #8]
	b.ne LBB370_92
	tbz w8, #30, LBB370_174
	tbnz w8, #31, LBB370_171
	ldp x26, x22, [sp, #80]
	ldr x27, [sp, #8]
	cbz w9, LBB370_92
	mov w10, #2
	mov x28, x9
	b LBB370_172
LBB370_167:
	mov w10, #5
LBB370_168:
	and x8, x8, #0x3fffffff
	orr x8, x8, x28, lsl #32
	stur w10, [x29, #-200]
	stur x8, [x14, #148]
	sub x1, x29, #200
	ldr x0, [sp, #104]
	ldr x2, [sp, #72]
	bl lyng_gc::mutator::PrimitiveMutator::store_value
LBB370_169:
	ldr x0, [sp, #88]
	mov x1, x21
	mov x2, x23
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
	mov x27, x24
	str x25, [sp, #112]
	ldr x8, [sp, #424]
	cbnz x8, LBB370_124
	b LBB370_138
LBB370_171:
	mov w10, #5
LBB370_172:
	and x8, x8, #0x3fffffff
	orr x8, x8, x28, lsl #32
	stur w10, [x29, #-200]
	stur x8, [x14, #148]
	sub x1, x29, #200
	ldr x0, [sp, #104]
	ldr x2, [sp, #72]
	bl lyng_gc::mutator::PrimitiveMutator::store_value
	tbnz w0, #0, LBB370_177
LBB370_174:
	ldr x8, [sp, #24]
	ldr x8, [x8, x24, lsl #3]
	cbz x8, LBB370_177
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB370_177
	ldr x0, [sp, #104]
	mov w1, #5
	bl lyng_ops::errors::error_value
	ldr x27, [sp, #8]
	b LBB370_110
LBB370_177:
	ldr x0, [sp, #88]
	mov x1, x21
	mov x2, x23
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
	ldr x27, [sp, #8]
	b LBB370_112
	mov x19, x0
	add x0, sp, #392
	bl core::ptr::drop_in_place<lyng_env::environment_records::GlobalEnvironmentRecord>
	mov x0, x19
	bl __Unwind_Resume
	mov x19, x0
	add x0, sp, #392
	bl core::ptr::drop_in_place<lyng_env::environment_records::GlobalEnvironmentRecord>
	mov x0, x19
	bl __Unwind_Resume
	.loh AdrpAdd	Lloh937, Lloh938
	.loh AdrpAdd	Lloh939, Lloh940
