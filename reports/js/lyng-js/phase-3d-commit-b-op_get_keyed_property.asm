lyng_js_vm::vm::dispatch_handlers::property::op_get_keyed_property:
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
	sub sp, sp, #592
	mov x19, x8
	ldr w27, [x0, #4]
	ldr w5, [x0, #56]
	ldrb w8, [x0, #148]
	mov w9, #152
	strb w9, [x0, #148]
	ldr x9, [x0, #128]
	ldr x1, [x9, #56]
	subs x2, x1, x5
	b.lo LBB389_120
	mov x20, x0
	mov x21, #33
	movk x21, #32768, lsl #48
	ldr x9, [x9, #48]
	add x1, x9, x5
	cmp w8, #152
	b.ne LBB389_121
	and x8, x2, #0x7ffffffffffffffe
	cmp x2, #4
	ccmp x8, #4, #4, hs
	b.eq LBB389_18
	ldrh w23, [x1, #4]
	cbz w23, LBB389_18
	ldrb w10, [x1, #1]
	ldrb w8, [x1, #2]
	mov w9, #6
	stp w9, w10, [sp, #80]
	ldrb w9, [x1, #3]
	ldr x21, [x20, #80]
	ldr w10, [x20, #20]
	ldr x1, [x21, #32]
	add w0, w10, w8
	cmp x1, x0
	b.ls LBB389_123
LBB389_5:
	add w8, w10, w9
	cmp x1, x8
	b.ls LBB389_171
	ldr x22, [x20, #88]
	ldr x9, [x20, #136]
	str x9, [sp, #88]
	ldr x9, [x21, #24]
	ldr x25, [x9, x0, lsl #3]
	and x24, x25, #0x7ff8000000000000
	ubfx x28, x25, #32, #16
	sub w11, w28, #1
	mov x10, #9221120237041090560
	cmp x24, x10
	ccmp w11, #1, #2, eq
	b.ls LBB389_20
	ldr x11, [x20, #96]
	str x11, [sp, #48]
	ldr x11, [x20, #104]
	str x11, [sp, #56]
	ldr x11, [x20, #112]
	str x11, [sp, #64]
	ldr x11, [x20, #120]
	str x11, [sp, #72]
	ldr x26, [x9, x8, lsl #3]
	and x8, x26, #0x7fffffff00000000
	and x8, x8, #0xfff8ffffffffffff
	cmp x24, x10
	ccmp w28, #5, #0, eq
	ccmp w25, #0, #4, eq
	mov x9, #17179869184
	movk x9, #32760, lsl #48
	ccmp x8, x9, #0, ne
	b.ne LBB389_30
	tbnz w26, #31, LBB389_31
	cbz w23, LBB389_13
	ldr x8, [x21, #104]
	sub w11, w27, #1
	cmp x8, x11
	b.ls LBB389_13
	ldr x8, [x21, #96]
	add x8, x8, x11, lsl #5
	ldr x9, [x8, #16]
	sub w10, w23, #1
	cmp x9, x10
	b.ls LBB389_13
	ldr x8, [x8, #8]
	mov w9, #1128
	umaddl x8, w10, w9, x8
	ldr x9, [x8]
	cmp x9, #10
	ccmp x9, #4, #2, ne
	b.lo LBB389_97
LBB389_13:
	ldr w2, [x20, #4]
	mov x0, x21
	mov x1, x22
	mov x3, x23
	mov x4, x25
	mov x5, x26
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::try_keyed_dense_index_load_inline_cache_hit
	tbz w0, #0, LBB389_68
LBB389_15:
	mov x8, x1
	ldr w9, [x20, #20]
	ldr x1, [x21, #32]
	ldr w10, [sp, #84]
	add w0, w9, w10
	cmp x1, x0
	b.ls LBB389_174
	ldr x9, [x21, #24]
	str x8, [x9, x0, lsl #3]
	ldr w8, [x20, #56]
	ldr w9, [sp, #80]
	adds w8, w8, w9
	b.hs LBB389_158
LBB389_17:
	str w8, [x20, #56]
	b LBB389_164
LBB389_18:
	sub x8, x21, #12
	stur x8, [x29, #-160]
	stp w27, w5, [x29, #-152]
LBB389_19:
	ldp q0, q1, [x29, #-160]
	stp q0, q1, [x19]
	ldur q0, [x29, #-128]
	str q0, [x19, #32]
	b LBB389_165
LBB389_20:
	mov x0, x22
	mov w1, #5
	bl lyng_js_ops::errors::error_value
	mov x27, x0
	ldr x8, [sp, #88]
	cbz x8, LBB389_24
	ldr x8, [sp, #88]
	sub x8, x8, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs LBB389_24
	ldr x9, [x21, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB389_24:
	sub x0, x29, #160
	mov x1, x21
	mov x2, x22
	mov x3, x27
	bl lyng_js_vm::vm::exceptions::<impl lyng_js_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-160]
	ldurb w5, [x29, #-152]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_59
	tbz w5, #0, LBB389_58
LBB389_27:
	ldr w8, [x21, #1640]
	add w8, w8, #1
	str w8, [x21, #1640]
	ldr x8, [sp, #88]
	cbz x8, LBB389_164
	ldr x8, [sp, #88]
	sub x8, x8, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs LBB389_164
	ldr x9, [x21, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x8]
	str q0, [x20]
	ldp q0, q1, [x8, #48]
	ldp q3, q2, [x8, #16]
	stp q0, q1, [x20, #48]
	stp q3, q2, [x20, #16]
	b LBB389_164
LBB389_30:
	mov x9, #30064771072
	movk x9, #32760, lsl #48
	cmp x8, x9
	ccmp w26, #0, #4, eq
	b.ne LBB389_63
LBB389_31:
	stp x21, x22, [x29, #-160]
	ldp x9, x8, [sp, #48]
	stp x9, x8, [x29, #-144]
	ldp x9, x8, [sp, #64]
	stp x9, x8, [x29, #-128]
	stur x20, [x29, #-112]
	sub x0, x29, #208
	sub x1, x29, #160
	mov x2, x26
	mov w3, #1
	bl lyng_js_ops::object::conversions::to_primitive
	ldp x8, x5, [x29, #-208]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_49
	ldr w3, [x20, #4]
	ldr w4, [x20, #56]
	add x0, sp, #144
	mov x1, x21
	mov x2, x22
	bl lyng_js_vm::vm::values::<impl lyng_js_vm::vm::Vm>::value_to_property_key
	ldr x8, [sp, #144]
	ldr w5, [sp, #152]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_50
	ldr w26, [sp, #156]
	mov x8, #9221120237041090560
	cmp x24, x8
	ccmp w25, #0, #4, eq
	ccmp w28, #5, #0, ne
	b.ne LBB389_64
LBB389_36:
	cbz w5, LBB389_76
	cmp w5, #1
	b.ne LBB389_87
	ldr w3, [x20, #4]
	ldr x1, [x21, #104]
	cbz w23, LBB389_153
	sub w8, w3, #1
	cmp x1, x8
	b.ls LBB389_153
	ldr x9, [x21, #96]
	add x9, x9, x8, lsl #5
	ldr x10, [x9, #16]
	sub w8, w23, #1
	cmp x10, x8
	b.ls LBB389_153
	ldr x9, [x9, #8]
	mov w10, #1128
	umaddl x9, w8, w10, x9
	ldr x8, [x9]
	cmp x8, #10
	b.eq LBB389_153
	cmp x8, #3
	b.hi LBB389_153
	ldr w8, [x9, #1116]
	cmp w8, w26
	b.ne LBB389_153
	ldr x8, [x9, #1088]
	cbz x8, LBB389_153
	ldr x12, [x22, #224]
	mov w10, #-1
	add x10, x25, x10
	lsr w11, w10, #6
	cmp x11, x12
	b.hs LBB389_153
	ldr x12, [x22, #216]
	and x10, x10, #0x3f
	ldr x11, [x12, x11, lsl #3]
	mov w12, #80
	umaddl x13, w10, w12, x11
	ldr w10, [x13]
	cmp w10, #1
	b.ne LBB389_153
	ldr x9, [x9, #1096]
	ldp w12, w10, [x13, #52]
	ldr x11, [x13, #40]
	ldur q0, [x13, #8]
	stur q0, [x29, #-160]
	ldur q0, [x13, #24]
	stur q0, [x29, #-144]
	lsr x13, x8, #32
	cmp x13, #0
	csel w13, w13, wzr, ne
	cbz w12, LBB389_144
	cmp w12, w13
	ccmp x11, x9, #0, eq
	b.eq LBB389_146
	b LBB389_153
LBB389_49:
	ldp q0, q1, [x29, #-192]
	stp q0, q1, [sp, #160]
	str x5, [sp, #152]
LBB389_50:
	mov x9, #-9223372036854775808
	cmp x8, x9
	b.ne LBB389_60
	cbnz w5, LBB389_60
	ldr x27, [sp, #160]
	ldr x8, [sp, #88]
	cbz x8, LBB389_55
	ldr x8, [sp, #88]
	sub x8, x8, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs LBB389_55
	ldr x9, [x21, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB389_55:
	sub x0, x29, #160
	mov x1, x21
	mov x2, x22
	mov x3, x27
	bl lyng_js_vm::vm::exceptions::<impl lyng_js_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-160]
	ldurb w5, [x29, #-152]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_59
	tbnz w5, #0, LBB389_27
LBB389_58:
	mov w9, #0
	mov w5, #0
	mov x8, #-9223372036854775808
	b LBB389_62
LBB389_59:
	ldurb w9, [x29, #-149]
	sub x11, x29, #160
	ldurh w10, [x11, #9]
	orr w9, w10, w9, lsl #16
	ldur w10, [x29, #-148]
	ldur q0, [x11, #24]
	str q0, [sp, #192]
	ldur x27, [x29, #-144]
	ldur x11, [x29, #-120]
	b LBB389_61
LBB389_60:
	lsr w9, w5, #8
	ldr w10, [sp, #156]
	ldr x27, [sp, #160]
	ldur q0, [sp, #168]
	str q0, [sp, #192]
	ldr x11, [sp, #184]
LBB389_61:
	str x11, [sp, #208]
LBB389_62:
	lsl w9, w9, #8
	bfxil x9, x5, #0, #8
	orr x9, x9, x10, lsl #32
	stp x8, x9, [x19]
	str x27, [x19, #16]
	ldr q0, [sp, #192]
	stur q0, [x19, #24]
	ldr x8, [sp, #208]
	str x8, [x19, #40]
	b LBB389_165
LBB389_63:
	mov w5, #2
	stp w5, w26, [sp, #152]
	mov x8, #33
	movk x8, #32768, lsl #48
	str x8, [sp, #144]
	mov x8, #9221120237041090560
	cmp x24, x8
	ccmp w25, #0, #4, eq
	ccmp w28, #5, #0, ne
	b.eq LBB389_36
LBB389_64:
	stp w5, w26, [sp, #8]
	sub x0, x29, #256
	str x25, [sp]
	mov x1, x21
	mov x2, x22
	ldp x3, x4, [sp, #48]
	ldp x5, x6, [sp, #64]
	mov x7, x20
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::get_property_from_value
	sub x0, x29, #160
	sub x5, x29, #256
	mov x1, x21
	mov x2, x22
	ldr x3, [sp, #88]
	mov x4, x20
	bl lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::handle_dispatch_result
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_166
	tbnz w5, #0, LBB389_156
	b LBB389_164
LBB389_68:
	add x0, sp, #96
	mov x1, x21
	mov x2, x22
	mov x3, x25
	mov x4, x26
	bl lyng_js_vm::vm::values::<impl lyng_js_vm::vm::Vm>::mapped_arguments_get
	ldr x8, [sp, #96]
	mov x9, #33
	movk x9, #32768, lsl #48
	add x9, x9, #1
	cmp x8, x9
	b.ne LBB389_73
	mov x0, x22
	mov x1, x25
	mov x2, x26
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::try_fast_typed_array_index_value
	tbz w0, #0, LBB389_92
	mov x27, x1
	b LBB389_95
LBB389_73:
	sub x0, x29, #160
	add x5, sp, #96
	mov x1, x21
	mov x2, x22
	ldr x3, [sp, #88]
	mov x4, x20
	bl lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::handle_dispatch_result
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_96
	tbnz w5, #0, LBB389_95
	b LBB389_164
LBB389_76:
	cbz w23, LBB389_80
	ldr w27, [x20, #4]
	ldr x8, [x21, #104]
	sub w28, w27, #1
	cmp x8, x28
	b.ls LBB389_80
	ldr x8, [x21, #96]
	add x8, x8, x28, lsl #5
	ldr x9, [x8, #16]
	sub w24, w23, #1
	cmp x9, x24
	b.ls LBB389_80
	ldr x8, [x8, #8]
	mov w9, #1128
	umaddl x8, w24, w9, x8
	ldr x9, [x8]
	cmp x9, #10
	ccmp x9, #4, #2, ne
	b.lo LBB389_124
LBB389_80:
	ldr w2, [x20, #4]
	mov x0, x21
	mov x1, x22
	mov x3, x23
	mov x4, x25
	mov x5, x26
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::try_keyed_dense_index_load_inline_cache_hit
	tbnz w0, #0, LBB389_15
	add x0, sp, #224
	mov x1, x21
	mov x2, x22
	mov x3, x25
	mov x4, x26
	bl lyng_js_vm::vm::values::<impl lyng_js_vm::vm::Vm>::mapped_arguments_get
	ldr x8, [sp, #224]
	mov x9, #33
	movk x9, #32768, lsl #48
	add x9, x9, #1
	cmp x8, x9
	b.ne LBB389_113
	mov x0, x22
	mov x1, x25
	mov x2, x26
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::try_fast_typed_array_index_value
	tbz w0, #0, LBB389_116
	mov x27, x1
	b LBB389_119
LBB389_87:
	stp w5, w26, [sp, #8]
	add x0, sp, #368
	str x25, [sp]
	mov x1, x21
	mov x2, x22
	ldp x3, x4, [sp, #48]
	ldp x5, x6, [sp, #64]
	mov x7, x20
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::get_property_from_value
	sub x0, x29, #160
	add x5, sp, #368
	mov x1, x21
	mov x2, x22
	ldr x3, [sp, #88]
	mov x4, x20
	bl lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::handle_dispatch_result
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_166
	tbz w5, #0, LBB389_164
	ldr w1, [x20, #4]
	mov x0, x21
	mov x2, x23
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::observe_keyed_generic_slow_path
	b LBB389_156
LBB389_92:
	sub x0, x29, #160
	mov x1, x22
	mov x2, x25
	mov x3, x26
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::try_fast_own_index_value
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_166
	tbz w5, #0, LBB389_31
LBB389_95:
	ldr w2, [x20, #4]
	mov x0, x21
	mov x1, x22
	mov x3, x23
	mov x4, x25
	mov x5, x26
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::observe_keyed_index_access
	b LBB389_156
LBB389_96:
	sub x9, x29, #160
	ldur q0, [x9, #24]
	str q0, [sp, #192]
	ldur x9, [x29, #-120]
	str x9, [sp, #208]
	lsr w9, w5, #8
	lsr x10, x5, #32
	b LBB389_62
LBB389_97:
	ldr x8, [x8, #1104]
	cbz w8, LBB389_13
	stp x8, x11, [sp, #32]
	str x10, [sp, #24]
	mov w9, #4992
	sub x8, x29, #160
	add x0, x22, x9
	mov x1, x22
	mov x2, x25
	bl lyng_js_objects::runtime::ObjectRuntime::object_header
	ldurb w8, [x29, #-138]
	cmp w8, #3
	ldp x10, x11, [sp, #32]
	b.eq LBB389_13
	cbz w10, LBB389_13
	ldur w8, [x29, #-156]
	cmp w8, w10
	b.ne LBB389_13
	ldur w8, [x29, #-144]
	ldurh w9, [x29, #-140]
	lsr x10, x10, #32
	cmp w9, w10, uxth
	ccmp w8, #0, #4, eq
	b.eq LBB389_13
	ldr x9, [x22, #640]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB389_13
	ldr x9, [x22, #632]
	mov w10, #24
	umaddl x8, w8, w10, x9
	ldrb w9, [x8, #19]
	cmp w9, #1
	b.ne LBB389_13
	ldr x10, [x8, #8]
	and x9, x26, #0x7fffffff
	cmp x10, x9
	b.ls LBB389_13
	ldr x8, [x8]
	ldr x9, [x8, x9, lsl #3]
	mov x8, #1
	movk x8, #9, lsl #32
	movk x8, #32760, lsl #48
	str x9, [sp, #32]
	cmp x9, x8
	b.eq LBB389_13
	ldr x8, [x21, #104]
	cmp x8, x11
	b.ls LBB389_111
	ldr x8, [x21, #96]
	ldr x9, [sp, #40]
	add x8, x8, x9, lsl #5
	ldr x9, [x8, #16]
	ldr x10, [sp, #24]
	cmp x9, x10
	b.ls LBB389_111
	ldr x8, [x8, #8]
	mov w9, #1128
	ldr x10, [sp, #24]
	umaddl x0, w10, w9, x8
	ldr x8, [x0]
	cmp x8, #10
	b.eq LBB389_111
	bl lyng_js_vm::vm::feedback::FeedbackSiteState::record_execution
LBB389_111:
	ldp x0, x1, [x21, #120]
	mov x2, x27
	bl lyng_js_vm::vm::tiering::<impl lyng_js_vm::vm::Vm>::observe_tier_feedback_event
	ldr w8, [x20, #20]
	ldr x1, [x21, #32]
	ldr w9, [sp, #84]
	add w0, w8, w9
	cmp x1, x0
	b.ls LBB389_174
	ldr x8, [x21, #24]
	ldr x9, [sp, #32]
	str x9, [x8, x0, lsl #3]
	ldr w8, [x20, #56]
	ldr w9, [sp, #80]
	adds w8, w8, w9
	b.lo LBB389_17
	b LBB389_158
LBB389_113:
	sub x0, x29, #160
	add x5, sp, #224
	mov x1, x21
	mov x2, x22
	ldr x3, [sp, #88]
	mov x4, x20
	bl lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::handle_dispatch_result
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_166
LBB389_115:
	tbnz w5, #0, LBB389_119
	b LBB389_164
LBB389_116:
	sub x0, x29, #160
	mov x1, x22
	mov x2, x25
	mov x3, x26
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::try_fast_own_index_value
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_166
	tbz w5, #0, LBB389_141
LBB389_119:
	ldr w2, [x20, #4]
	mov x0, x21
	mov x1, x22
	mov x3, x23
	mov x4, x25
	mov x5, x26
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::observe_keyed_index_access
	b LBB389_156
LBB389_120:
Lloh1063:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.45@PAGE
Lloh1064:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.45@PAGEOFF
	mov x0, x5
	mov x2, x1
	bl core::slice::index::slice_index_fail
	b LBB389_173
LBB389_121:
	sub x0, x29, #160
	mov w3, #1
	mov x4, x27
	bl lyng_js_vm::vm::dispatch::decode_abc_operands_wide
	ldur x8, [x29, #-160]
	cmp x8, x21
	b.ne LBB389_19
	ldurh w11, [x29, #-148]
	ldurh w8, [x29, #-146]
	ldurh w9, [x29, #-144]
	ldur w23, [x29, #-152]
	ldur w10, [x29, #-140]
	stp w10, w11, [sp, #80]
	ldr x21, [x20, #80]
	ldr w10, [x20, #20]
	ldr x1, [x21, #32]
	add w0, w10, w8
	cmp x1, x0
	b.hi LBB389_5
LBB389_123:
Lloh1065:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.59@PAGE
Lloh1066:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.59@PAGEOFF
	b LBB389_172
LBB389_124:
	ldr x8, [x8, #1104]
	cbz w8, LBB389_80
	str x8, [sp, #40]
	mov w9, #4992
	sub x8, x29, #160
	add x0, x22, x9
	mov x1, x22
	mov x2, x25
	bl lyng_js_objects::runtime::ObjectRuntime::object_header
	ldurb w8, [x29, #-138]
	cmp w8, #3
	ldr x9, [sp, #40]
	b.eq LBB389_80
	cbz w9, LBB389_80
	ldur w8, [x29, #-156]
	cmp w8, w9
	b.ne LBB389_80
	ldurh w8, [x29, #-140]
	lsr x9, x9, #32
	cmp w8, w9, uxth
	b.ne LBB389_80
	ldur w8, [x29, #-144]
	cbz w8, LBB389_80
	ldr x9, [x22, #640]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB389_80
	ldr x9, [x22, #632]
	mov w10, #24
	umaddl x8, w8, w10, x9
	ldrb w9, [x8, #19]
	cmp w9, #1
	b.ne LBB389_80
	ldr x10, [x8, #8]
	mov w9, w26
	cmp x10, x9
	b.ls LBB389_80
	ldr x8, [x8]
	ldr x9, [x8, x9, lsl #3]
	mov x8, #1
	movk x8, #9, lsl #32
	movk x8, #32760, lsl #48
	str x9, [sp, #40]
	cmp x9, x8
	b.eq LBB389_80
	ldr x8, [x21, #104]
	cmp x8, x28
	b.ls LBB389_139
	ldr x8, [x21, #96]
	add x8, x8, x28, lsl #5
	ldr x9, [x8, #16]
	cmp x9, x24
	b.ls LBB389_139
	ldr x8, [x8, #8]
	mov w9, #1128
	umaddl x0, w24, w9, x8
	ldr x8, [x0]
	cmp x8, #10
	b.eq LBB389_139
	bl lyng_js_vm::vm::feedback::FeedbackSiteState::record_execution
LBB389_139:
	ldp x0, x1, [x21, #120]
	mov x2, x27
	bl lyng_js_vm::vm::tiering::<impl lyng_js_vm::vm::Vm>::observe_tier_feedback_event
	ldr w8, [x20, #20]
	ldr x1, [x21, #32]
	ldr w9, [sp, #84]
	add w0, w8, w9
	cmp x1, x0
	b.ls LBB389_174
	ldr x8, [x21, #24]
	ldr x9, [sp, #40]
	str x9, [x8, x0, lsl #3]
	ldr w8, [x20, #56]
	ldr w9, [sp, #80]
	adds w8, w8, w9
	b.lo LBB389_17
	b LBB389_158
LBB389_141:
	stp wzr, w26, [sp, #8]
	add x0, sp, #272
	str x25, [sp]
