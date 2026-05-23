    Finished `release` profile [optimized] target(s) in 0.01s

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
	b.lo LBB370_127
	mov x19, x0
	add x27, sp, #192
	mov x25, #33
	movk x25, #32768, lsl #48
	ldr x9, [x9, #48]
	add x1, x9, x6
	cmp w3, #152
	b.ne LBB370_128
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
	b.ls LBB370_130
LBB370_5:
	ldr x17, [x15, #72]
	ldr x11, [x17, x24, lsl #3]
	cbz x11, LBB370_130
	ldr x13, [x11, #80]
	mov w12, w8
	sub x10, x25, #3
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
	ldp x0, x9, [x20, #88]
	ldp x11, x8, [x20, #104]
	stp x9, x11, [sp, #40]
	str x8, [sp, #56]
	ldr x8, [x20, #120]
	stp x8, x10, [sp, #64]
	str x0, [sp, #104]
	str w14, [sp, #100]
	tbz w2, #0, LBB370_21
	str x17, [sp, #24]
	ldr w28, [x20, #12]
	add x25, sp, #392
	sub x22, x29, #200
	mov x8, #33
	movk x8, #32768, lsl #48
	sub x26, x8, #27
	str w28, [sp, #32]
LBB370_14:
	add x8, sp, #392
	mov x1, x28
	bl lyng_env::agent::environments::<impl lyng_env::agent::Agent>::environment
	ldr x16, [sp, #392]
	cmp x16, x26
	b.eq LBB370_26
	ldur q0, [x25, #88]
	ldur q1, [x25, #104]
	stp q0, q1, [sp, #336]
	ldur q0, [x25, #120]
	str q0, [sp, #368]
	ldur q0, [x25, #56]
	ldur q1, [x25, #72]
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
	ldur x13, [x25, #45]
	stur x13, [x29, #-200]
	ldur w13, [x25, #52]
	add x14, sp, #392
	stur w13, [x14, #151]
	ldur x13, [x25, #136]
	str x13, [sp, #384]
	stur x13, [x22, #91]
	stur q1, [x22, #27]
	tbz x16, #63, LBB370_49
	ldr x0, [sp, #104]
	cbnz w28, LBB370_14
	b LBB370_25
LBB370_17:
	sub x8, x25, #12
	str x8, [sp, #392]
	str w21, [sp, #400]
	str w6, [sp, #404]
LBB370_18:
	ldur q0, [x27, #200]
	ldur q1, [x27, #216]
	stp q0, q1, [x19]
	ldur q0, [x27, #232]
	str q0, [x19, #32]
	b LBB370_133
LBB370_19:
	mov x9, #0
	b LBB370_131
LBB370_20:
	ldr x11, [x13, #8]
	orr x9, x12, x16, lsl #32
	b LBB370_132
LBB370_21:
	ldr w28, [x20, #12]
	add x25, sp, #392
	sub x22, x29, #200
	mov x8, #33
	movk x8, #32768, lsl #48
	sub x26, x8, #27
	str w28, [sp, #32]
LBB370_22:
	add x8, sp, #392
	mov x1, x28
	bl lyng_env::agent::environments::<impl lyng_env::agent::Agent>::environment
	ldr x16, [sp, #392]
	cmp x16, x26
	b.eq LBB370_26
	ldur q0, [x25, #88]
	ldur q1, [x25, #104]
	stp q0, q1, [sp, #336]
	ldur q0, [x25, #120]
	str q0, [sp, #368]
	ldur q0, [x25, #56]
	ldur q1, [x25, #72]
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
	ldur x13, [x25, #45]
	stur x13, [x29, #-200]
	ldur w13, [x25, #52]
	add x14, sp, #392
	stur w13, [x14, #151]
	ldur x13, [x25, #136]
	str x13, [sp, #384]
	stur x13, [x22, #91]
	stur q1, [x22, #27]
	tbz x16, #63, LBB370_42
	ldr x0, [sp, #104]
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
Lloh957:
	adrp x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
Lloh958:
	add x9, x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x8, [x9, x8, lsl #3]
	stp x25, x8, [x19]
	b LBB370_133
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
	tbz w8, #0, LBB370_115
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
	stur w13, [x27, #95]
	ldp q0, q1, [x15, #144]
	stp q0, q1, [sp, #192]
	ldp q1, q0, [x15, #176]
	stp q1, q0, [sp, #224]
	ldr x13, [sp, #192]
	ldur w14, [sp, #199]
	str w14, [sp, #444]
	stur x13, [x27, #245]
	ldur q0, [sp, #203]
	ldur q1, [sp, #219]
	stur q1, [x15, #72]
	stur q0, [x15, #56]
	ldur q0, [sp, #235]
	ldur q1, [x27, #59]
	ldur q2, [x27, #75]
	ldur x13, [x27, #91]
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
	ldr x2, [sp, #104]
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
	mov w4, #0
	mov x5, x28
	bl lyng_vm::vm::loop_iteration::<impl lyng_vm::vm::Vm>::environment_for_slot_access
	ldur x8, [x29, #-200]
	ldur w3, [x29, #-192]
	cmp x8, x25
	b.ne LBB370_116
	add x0, sp, #112
	mov x1, x22
	ldr x2, [sp, #104]
	mov x4, x28
	ldr x5, [sp, #72]
	bl lyng_vm::vm::values::<impl lyng_vm::vm::Vm>::write_environment_slot
	b LBB370_137
LBB370_49:
	add x15, sp, #392
	ldp q0, q1, [x15, #208]
	stp q0, q1, [sp, #256]
	ldur w13, [x15, #239]
	stur w13, [x27, #95]
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
	ldur q1, [x27, #59]
	ldur q2, [x27, #75]
	ldur x13, [x27, #91]
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
	ldr x2, [sp, #104]
	mov x27, x16
LBB370_50:
	cbz x9, LBB370_86
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
	mov w4, #0
	mov x5, x28
	bl lyng_vm::vm::loop_iteration::<impl lyng_vm::vm::Vm>::environment_for_slot_access
	ldur x8, [x29, #-200]
	ldur w3, [x29, #-192]
	cmp x8, x25
	b.ne LBB370_117
	ldr x8, [x22, #80]
	cmp x8, x24
	b.ls LBB370_134
	ldr x8, [x22, #72]
	ldr x8, [x8, x24, lsl #3]
	ldr x2, [sp, #104]
	cbz x8, LBB370_135
	ldrh w8, [x8, #340]
	b LBB370_135
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
	b LBB370_133
LBB370_61:
	ldr w8, [sp, #516]
	ldr x9, [x2, #5144]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB370_70
	ldr x9, [x2, #5136]
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
	cbz w23, LBB370_74
	ldr x8, [x22, #104]
	cmp x8, x24
	b.ls LBB370_74
	ldr x8, [x22, #96]
	add x9, x8, x24, lsl #5
	ldr x10, [x9, #16]
	sub w8, w23, #1
	cmp x10, x8
	b.ls LBB370_74
	ldr x9, [x9, #8]
	mov w10, #1216
	umaddl x9, w8, w10, x9
	ldr x8, [x9]
	cmp x8, #10
	ccmp x8, #6, #0, ne
	b.eq LBB370_159
LBB370_74:
	ldr x10, [x2, #224]
	sub w8, w28, #1
	lsr x9, x8, #6
	cmp x9, x10
	b.hs LBB370_81
	ldr x10, [x2, #216]
	and x8, x8, #0x3f
	ldr x9, [x10, x9, lsl #3]
	mov w10, #80
	umaddl x8, w8, w10, x9
	ldr w9, [x8]
	cmp w9, #1
	b.ne LBB370_81
	cbz w23, LBB370_81
	ldr w10, [x8, #52]
	cbz w10, LBB370_81
	ldr x9, [x22, #104]
	cmp x9, x24
	b.ls LBB370_81
	ldr x9, [x22, #96]
	add x11, x9, x24, lsl #5
	ldr x12, [x11, #16]
	sub w9, w23, #1
	cmp x12, x9
	b.ls LBB370_81
	ldr x11, [x11, #8]
	mov w12, #1216
	umaddl x11, w9, w12, x11
	ldr x9, [x11]
	cmp x9, #10
	ccmp x9, #6, #0, ne
	b.eq LBB370_169
LBB370_81:
	ldp x0, x1, [x22, #96]
	mov x3, x21
	mov x4, x23
	mov x5, x28
	ldr x6, [sp, #72]
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_named_property_store_inline_cache
	and w8, w0, #0xff
	cmp w8, #2
	b.ne LBB370_118
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
	b.ne LBB370_136
	mov x0, x22
	ldr x1, [sp, #104]
	mov x2, x21
	mov x3, x23
	mov x4, x28
	mov x5, x26
	mov w6, #1
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_named_property_slow_path
	b LBB370_158
LBB370_86:
	ldr w8, [sp, #516]
	ldr x9, [x2, #5144]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB370_95
	ldr x9, [x2, #5136]
	add x8, x9, x8, lsl #5
	ldr x9, [x8]
	mov x10, #-9223372036854775808
	cmp x9, x10
	b.eq LBB370_95
	ldr x9, [x8, #16]
	cbz x9, LBB370_95
	mov x28, #0
	ldr w3, [sp, #512]
	ldr x8, [x8, #8]
	add x9, x8, x9, lsl #4
	b LBB370_91
LBB370_90:
	add x8, x8, #16
	add x28, x28, #1
	cmp x8, x9
	b.eq LBB370_95
LBB370_91:
	ldp w10, w11, [x8]
	cmp w10, #1
	ccmp w11, w26, #0, eq
	b.ne LBB370_90
	ldrb w10, [x8, #9]
	tbz w10, #0, LBB370_90
	lsr x8, x28, #32
	cbnz x8, LBB370_95
	cbnz w3, LBB370_53
LBB370_95:
	ldr w28, [sp, #520]
	cbz w23, LBB370_99
	ldr x8, [x22, #104]
	cmp x8, x24
	b.ls LBB370_99
	ldr x8, [x22, #96]
	add x9, x8, x24, lsl #5
	ldr x10, [x9, #16]
	sub w8, w23, #1
	cmp x10, x8
	b.ls LBB370_99
	ldr x9, [x9, #8]
	mov w10, #1216
	umaddl x9, w8, w10, x9
	ldr x8, [x9]
	cmp x8, #10
	ccmp x8, #6, #0, ne
	b.eq LBB370_164
LBB370_99:
	ldr x10, [x2, #224]
	sub w8, w28, #1
	lsr x9, x8, #6
	cmp x9, x10
	b.hs LBB370_106
	ldr x10, [x2, #216]
	and x8, x8, #0x3f
	ldr x9, [x10, x9, lsl #3]
	mov w10, #80
	umaddl x8, w8, w10, x9
	ldr w9, [x8]
	cmp w9, #1
	b.ne LBB370_106
	cbz w23, LBB370_106
	ldr w10, [x8, #52]
	cbz w10, LBB370_106
	ldr x9, [x22, #104]
	cmp x9, x24
	b.ls LBB370_106
	ldr x9, [x22, #96]
	add x11, x9, x24, lsl #5
	ldr x12, [x11, #16]
	sub w9, w23, #1
	cmp x12, x9
	b.ls LBB370_106
	ldr x11, [x11, #8]
	mov w12, #1216
	umaddl x11, w9, w12, x11
	ldr x9, [x11]
	cmp x9, #10
	ccmp x9, #6, #0, ne
	b.eq LBB370_172
LBB370_106:
	ldp x0, x1, [x22, #96]
	mov x3, x21
	mov x4, x23
	mov x5, x28
	ldr x6, [sp, #72]
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_named_property_store_inline_cache
	and w8, w0, #0xff
	cmp w8, #2
	b.ne LBB370_120
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
	b.ne LBB370_147
	tbnz w8, #0, LBB370_140
	ldr x9, [x22, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB370_140
	ldr x9, [x22, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB370_140
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB370_140
	ldr x0, [sp, #104]
	mov w1, #3
	bl lyng_ops::errors::error_value
	b LBB370_124
LBB370_115:
	mov w8, #0
	mov w10, #0
	mov x9, #-9223372036854775808
	b LBB370_60
LBB370_116:
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
	cbnz x8, LBB370_138
	b LBB370_152
LBB370_117:
	add x9, sp, #392
	ldur q0, [x9, #156]
	stur q0, [sp, #124]
	ldur q0, [x9, #172]
	stur q0, [sp, #140]
	ldur w9, [x29, #-156]
	str w9, [sp, #156]
	str x8, [sp, #112]
	str w3, [sp, #120]
	b LBB370_148
LBB370_118:
	mov x0, x22
	mov x1, x21
	mov x2, x23
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
LBB370_119:
	str x25, [sp, #112]
	ldr x8, [sp, #424]
	cbnz x8, LBB370_138
	b LBB370_152
LBB370_120:
	tbnz w8, #0, LBB370_125
	ldr x8, [sp, #24]
	ldr x8, [x8, x24, lsl #3]
	cbz x8, LBB370_125
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB370_125
	ldr x0, [sp, #104]
	mov w1, #5
	bl lyng_ops::errors::error_value
LBB370_124:
	mov x8, #-9223372036854775808
	str x8, [sp, #112]
	str wzr, [sp, #120]
	str x0, [sp, #128]
	b LBB370_148
LBB370_125:
	mov x0, x22
	mov x1, x21
	mov x2, x23
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
LBB370_126:
	str x25, [sp, #112]
	b LBB370_148
LBB370_127:
Lloh959:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
Lloh960:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov x0, x6
	mov x2, x1
	bl core::slice::index::slice_index_fail
LBB370_128:
	mov x22, x2
	add x0, sp, #392
	mov x2, x8
	mov w4, #1
	mov x5, x21
	bl lyng_vm::vm::dispatch::decode_abx_operands_wide
	ldr x8, [sp, #392]
	cmp x8, x25
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
LBB370_130:
	sub x10, x25, #29
	mov x9, x21
LBB370_131:
LBB370_132:
	stp x10, x9, [x19]
	str x11, [x19, #16]
	stp w21, w8, [x19, #24]
LBB370_133:
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
LBB370_134:
	.cfi_restore_state
	mov w8, #0
	ldr x2, [sp, #104]
LBB370_135:
	add x0, sp, #112
	and w6, w8, #0x1
	mov x1, x22
	mov x4, x28
	ldr x5, [sp, #72]
	bl lyng_vm::vm::values::<impl lyng_vm::vm::Vm>::assign_environment_slot
	b LBB370_148
LBB370_136:
	ldrb w9, [sp, #200]
	ldur q0, [sp, #201]
	stur q0, [sp, #121]
	ldur q0, [sp, #217]
	stur q0, [sp, #137]
	ldr x10, [sp, #232]
	str x10, [sp, #152]
	str x8, [sp, #112]
	strb w9, [sp, #120]
LBB370_137:
	ldr x8, [sp, #424]
	cbz x8, LBB370_152
LBB370_138:
	lsl x9, x8, #2
	add x9, x9, #11
	and x9, x9, #0xfffffffffffffff8
	add x8, x8, x9
	adds x1, x8, #9
	b.eq LBB370_152
	ldr x8, [sp, #24]
	b LBB370_151
LBB370_140:
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
	b.ne LBB370_147
	tbnz w8, #0, LBB370_157
	ldr x9, [x22, #80]
	ldr w8, [x20, #4]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB370_157
	ldr x9, [x22, #72]
	ldr x8, [x9, x8, lsl #3]
	cbz x8, LBB370_157
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB370_157
	ldr x0, [sp, #104]
	mov w1, #5
	bl lyng_ops::errors::error_value
	b LBB370_124
LBB370_147:
	ldur q0, [sp, #201]
	stur q0, [sp, #121]
	ldur q0, [sp, #217]
	stur q0, [sp, #137]
	ldr x10, [sp, #232]
	str x10, [sp, #152]
	str x9, [sp, #112]
	strb w8, [sp, #120]
LBB370_148:
	ldr x8, [sp, #424]
	cbz x8, LBB370_152
	lsl x9, x8, #2
	add x9, x9, #11
	and x9, x9, #0xfffffffffffffff8
	add x8, x8, x9
	adds x1, x8, #9
	b.eq LBB370_152
	ldr x8, [sp, #16]
LBB370_151:
	sub x0, x8, x9
	mov w2, #8
	bl __rustc::__rust_dealloc
LBB370_152:
	ldr x0, [sp, #32]
	cbz x27, LBB370_154
	add x8, x27, x27, lsl #1
	lsl x1, x8, #2
	mov w2, #4
	bl __rustc::__rust_dealloc
LBB370_154:
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
LBB370_157:
	mov x0, x22
	ldr x1, [sp, #104]
	mov x2, x21
	mov x3, x23
	mov x4, x28
	mov x5, x26
	mov w6, #1
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_named_property_slow_path
LBB370_158:
	str x25, [sp, #112]
	add x0, sp, #392
	bl core::ptr::drop_in_place<lyng_env::environment_records::GlobalEnvironmentRecord>
	b LBB370_27
LBB370_159:
	ldr x8, [x9, #968]
	cbz x8, LBB370_74
	ldr x12, [x2, #224]
	sub w10, w28, #1
	lsr x11, x10, #6
	cmp x11, x12
	b.hs LBB370_74
	ldr x12, [x2, #216]
	and x10, x10, #0x3f
	ldr x11, [x12, x11, lsl #3]
	mov w12, #80
	umaddl x11, w10, w12, x11
	ldr w10, [x11]
	cmp w10, #1
	b.ne LBB370_74
	str x27, [sp, #16]
	ldr x10, [x9, #976]
	ldp w12, w9, [x11, #52]
	ldr x11, [x11, #40]
	lsr x13, x8, #32
	cmp x13, #0
	csel w13, w13, wzr, ne
	cbz w12, LBB370_187
	cmp w12, w13
	b LBB370_188
LBB370_164:
	ldr x8, [x9, #968]
	cbz x8, LBB370_99
	ldr x12, [x2, #224]
	sub w10, w28, #1
	lsr x11, x10, #6
	cmp x11, x12
	b.hs LBB370_99
	ldr x12, [x2, #216]
	and x10, x10, #0x3f
	ldr x11, [x12, x11, lsl #3]
	mov w12, #80
	umaddl x11, w10, w12, x11
	ldr w10, [x11]
	cmp w10, #1
	b.ne LBB370_99
	str x27, [sp, #8]
	ldr x10, [x9, #976]
	ldp w12, w9, [x11, #52]
	ldr x11, [x11, #40]
	lsr x13, x8, #32
	cmp x13, #0
	csel w13, w13, wzr, ne
	cbz w12, LBB370_193
	cmp w12, w13
	b LBB370_194
LBB370_169:
	mov x24, x27
	ldr x9, [x11, #1016]
	lsr x12, x9, #32
	cmp x12, #0
	ccmp w10, w12, #0, ne
	b.eq LBB370_175
	ldr x9, [x11, #1024]
	lsr x12, x9, #32
	cmp x12, #0
	ccmp w10, w12, #0, ne
	mov x25, #33
	movk x25, #32768, lsl #48
	ldp x26, x22, [sp, #80]
	ldr x2, [sp, #104]
	mov x27, x24
	b.ne LBB370_81
	mov w10, #1
	b LBB370_176
LBB370_172:
	str x27, [sp, #8]
	ldr x9, [x11, #1016]
	lsr x12, x9, #32
	cmp x12, #0
	ccmp w10, w12, #0, ne
	b.eq LBB370_181
	ldr x9, [x11, #1024]
	lsr x12, x9, #32
	cmp x12, #0
	ccmp w10, w12, #0, ne
	mov x25, #33
	movk x25, #32768, lsl #48
	ldp x26, x22, [sp, #80]
	ldr x2, [sp, #104]
	ldr x27, [sp, #8]
	b.ne LBB370_106
	mov w10, #1
	b LBB370_182
LBB370_175:
	mov x10, #0
LBB370_176:
	add x10, x11, x10, lsl #3
	ldr x10, [x10, #1032]
	ldr x11, [x8, #40]
	cmp x11, x10
	mov x25, #33
	movk x25, #32768, lsl #48
	ldp x26, x22, [sp, #80]
	ldr x2, [sp, #104]
	mov x27, x24
	b.ne LBB370_81
	tbz w9, #30, LBB370_211
	tbnz w9, #31, LBB370_209
	ldr w10, [x8, #56]
	ldp x26, x22, [sp, #80]
	ldr x2, [sp, #104]
	mov x27, x24
	cbz w10, LBB370_81
	mov w8, #2
	mov x28, x10
	b LBB370_210
LBB370_181:
	mov x10, #0
LBB370_182:
	add x10, x11, x10, lsl #3
	ldr x10, [x10, #1032]
	ldr x11, [x8, #40]
	cmp x11, x10
	mov x25, #33
	movk x25, #32768, lsl #48
	ldp x26, x22, [sp, #80]
	ldr x2, [sp, #104]
	ldr x27, [sp, #8]
	b.ne LBB370_106
	tbz w9, #30, LBB370_216
	tbnz w9, #31, LBB370_213
	ldr w10, [x8, #56]
	ldp x26, x22, [sp, #80]
	ldr x2, [sp, #104]
	ldr x27, [sp, #8]
	cbz w10, LBB370_106
	mov w8, #2
	mov x28, x10
	b LBB370_214
LBB370_187:
	cmp w13, #0
LBB370_188:
	ccmp x11, x10, #0, eq
	mov x25, #33
	movk x25, #32768, lsl #48
	ldp x26, x22, [sp, #80]
	ldr x2, [sp, #104]
	ldr x27, [sp, #16]
	b.ne LBB370_74
	tbz w8, #30, LBB370_201
	tbnz w8, #31, LBB370_199
	ldp x26, x22, [sp, #80]
	ldr x2, [sp, #104]
	ldr x27, [sp, #16]
	cbz w9, LBB370_74
	mov w10, #2
	mov x28, x9
	b LBB370_200
LBB370_193:
	cmp w13, #0
LBB370_194:
	ccmp x11, x10, #0, eq
	mov x25, #33
	movk x25, #32768, lsl #48
	ldp x26, x22, [sp, #80]
	ldr x2, [sp, #104]
	ldr x27, [sp, #8]
	b.ne LBB370_99
	tbz w8, #30, LBB370_205
	tbnz w8, #31, LBB370_202
	ldp x26, x22, [sp, #80]
	ldr x2, [sp, #104]
	ldr x27, [sp, #8]
	cbz w9, LBB370_99
	mov w10, #2
	mov x28, x9
	b LBB370_203
LBB370_199:
	mov w10, #5
LBB370_200:
	and x8, x8, #0x3fffffff
	orr x8, x8, x28, lsl #32
	stur w10, [x29, #-200]
	stur x8, [x14, #148]
	sub x1, x29, #200
	ldr x0, [sp, #104]
	ldr x2, [sp, #72]
	bl lyng_gc::mutator::PrimitiveMutator::store_value
LBB370_201:
	ldr x0, [sp, #88]
	mov x1, x21
	mov x2, x23
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
	ldr x27, [sp, #16]
	b LBB370_119
LBB370_202:
	mov w10, #5
LBB370_203:
	and x8, x8, #0x3fffffff
	orr x8, x8, x28, lsl #32
	stur w10, [x29, #-200]
	stur x8, [x14, #148]
	sub x1, x29, #200
	ldr x0, [sp, #104]
	ldr x2, [sp, #72]
	bl lyng_gc::mutator::PrimitiveMutator::store_value
	tbnz w0, #0, LBB370_208
LBB370_205:
	ldr x8, [sp, #24]
	ldr x8, [x8, x24, lsl #3]
	cbz x8, LBB370_208
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB370_208
	ldr x0, [sp, #104]
	mov w1, #5
	bl lyng_ops::errors::error_value
	ldr x27, [sp, #8]
	b LBB370_124
LBB370_208:
	ldr x0, [sp, #88]
	mov x1, x21
	mov x2, x23
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
	ldr x27, [sp, #8]
	b LBB370_126
LBB370_209:
	mov w8, #5
LBB370_210:
	and w9, w9, #0x3fffffff
	stp w8, w9, [x29, #-200]
	stur w28, [x29, #-192]
	sub x1, x29, #200
	ldr x0, [sp, #104]
	ldr x2, [sp, #72]
	bl lyng_gc::mutator::PrimitiveMutator::store_value
LBB370_211:
	ldr x0, [sp, #88]
	mov x1, x21
	mov x2, x23
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
	mov x27, x24
	str x25, [sp, #112]
	ldr x8, [sp, #424]
	cbnz x8, LBB370_138
	b LBB370_152
LBB370_213:
	mov w8, #5
LBB370_214:
	and w9, w9, #0x3fffffff
	stp w8, w9, [x29, #-200]
	stur w28, [x29, #-192]
	sub x1, x29, #200
	ldr x0, [sp, #104]
	ldr x2, [sp, #72]
	bl lyng_gc::mutator::PrimitiveMutator::store_value
	tbnz w0, #0, LBB370_219
LBB370_216:
	ldr x8, [sp, #24]
	ldr x8, [x8, x24, lsl #3]
	cbz x8, LBB370_219
	ldrh w8, [x8, #340]
	tbz w8, #0, LBB370_219
	ldr x0, [sp, #104]
	mov w1, #5
	bl lyng_ops::errors::error_value
	ldr x27, [sp, #8]
	b LBB370_124
LBB370_219:
	ldr x0, [sp, #88]
	mov x1, x21
	mov x2, x23
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::record_feedback_slot
	ldr x27, [sp, #8]
	b LBB370_126
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
	.loh AdrpAdd	Lloh957, Lloh958
	.loh AdrpAdd	Lloh959, Lloh960
