    Finished `release` profile [optimized] target(s) in 0.01s

.section __TEXT,__text,regular,pure_instructions
	.p2align	2
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
	b.lo LBB389_162
	mov x20, x0
	mov x21, #33
	movk x21, #32768, lsl #48
	ldr x9, [x9, #48]
	add x1, x9, x5
	cmp w8, #152
	b.ne LBB389_163
	and x8, x2, #0x7ffffffffffffffe
	cmp x2, #4
	ccmp x8, #4, #4, hs
	b.eq LBB389_21
	ldrh w22, [x1, #4]
	cbz w22, LBB389_21
	ldrb w13, [x1, #1]
	ldrb w8, [x1, #2]
	mov w12, #6
	ldrb w9, [x1, #3]
	ldr x21, [x20, #80]
	ldr w10, [x20, #20]
	ldr x1, [x21, #32]
	add w0, w10, w8
	cmp x1, x0
	b.ls LBB389_165
LBB389_5:
	add w8, w10, w9
	cmp x1, x8
	b.ls LBB389_271
	ldr x23, [x20, #88]
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
	b.ls LBB389_23
	stp w12, w13, [sp, #48]
	ldr x11, [x20, #96]
	str x11, [sp, #56]
	ldr x11, [x20, #104]
	str x11, [sp, #64]
	ldr x11, [x20, #112]
	str x11, [sp, #72]
	ldr x11, [x20, #120]
	str x11, [sp, #80]
	ldr x11, [x9, x8, lsl #3]
	and x8, x11, #0x7fffffff00000000
	and x8, x8, #0xfff8ffffffffffff
	cmp x24, x10
	ccmp w28, #5, #0, eq
	ccmp w25, #0, #4, eq
	mov x9, #17179869184
	movk x9, #32760, lsl #48
	ccmp x8, x9, #0, ne
	b.ne LBB389_33
	mov x26, x11
	tbnz w11, #31, LBB389_34
	cbz w22, LBB389_16
	ldr x8, [x21, #104]
	sub w11, w27, #1
	cmp x8, x11
	b.ls LBB389_13
	ldr x8, [x21, #96]
	add x8, x8, x11, lsl #5
	ldr x9, [x8, #16]
	sub w10, w22, #1
	cmp x9, x10
	b.ls LBB389_13
	ldr x8, [x8, #8]
	mov w9, #1216
	umaddl x8, w10, w9, x8
	ldr x9, [x8]
	cmp x9, #10
	ccmp x9, #4, #2, ne
	b.lo LBB389_119
LBB389_13:
	ldr w10, [x20, #4]
	ldr x8, [x21, #104]
	sub w27, w10, #1
	cmp x8, x27
	b.ls LBB389_16
	ldr x8, [x21, #96]
	add x8, x8, x27, lsl #5
	ldr x9, [x8, #16]
	sub w11, w22, #1
	cmp x9, x11
	b.ls LBB389_16
	ldr x8, [x8, #8]
	mov w9, #1216
	umaddl x9, w11, w9, x8
	ldr x8, [x9]
	cmp x8, #10
	ccmp x8, #3, #2, ne
	b.ls LBB389_135
LBB389_16:
	ldr w2, [x20, #4]
	mov x0, x21
	mov x1, x23
	mov x3, x22
	mov x4, x25
	mov x5, x26
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::try_keyed_dense_index_load_inline_cache_hit
	tbz w0, #0, LBB389_71
LBB389_18:
	mov x8, x1
	ldr w9, [x20, #20]
	ldr x1, [x21, #32]
	ldr w10, [sp, #52]
	add w0, w9, w10
	cmp x1, x0
	b.ls LBB389_274
	ldr x9, [x21, #24]
	str x8, [x9, x0, lsl #3]
	ldr w8, [x20, #56]
	ldr w9, [sp, #48]
	adds w8, w8, w9
	b.hs LBB389_260
LBB389_20:
	str w8, [x20, #56]
	b LBB389_266
LBB389_21:
	sub x8, x21, #12
	stur x8, [x29, #-160]
	stp w27, w5, [x29, #-152]
LBB389_22:
	ldp q0, q1, [x29, #-160]
	stp q0, q1, [x19]
	ldur q0, [x29, #-128]
	str q0, [x19, #32]
	b LBB389_267
LBB389_23:
	mov x0, x23
	mov w1, #5
	bl lyng_js_ops::errors::error_value
	mov x27, x0
	ldr x8, [sp, #88]
	cbz x8, LBB389_27
	ldr x8, [sp, #88]
	sub x8, x8, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs LBB389_27
	ldr x9, [x21, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB389_27:
	sub x0, x29, #160
	mov x1, x21
	mov x2, x23
	mov x3, x27
	bl lyng_js_vm::vm::exceptions::<impl lyng_js_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-160]
	ldurb w5, [x29, #-152]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_62
	tbz w5, #0, LBB389_61
LBB389_30:
	ldr w8, [x21, #1640]
	add w8, w8, #1
	str w8, [x21, #1640]
	ldr x8, [sp, #88]
	cbz x8, LBB389_266
	ldr x8, [sp, #88]
	sub x8, x8, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs LBB389_266
	ldr x9, [x21, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x8]
	str q0, [x20]
	ldp q0, q1, [x8, #48]
	ldp q3, q2, [x8, #16]
	stp q0, q1, [x20, #48]
	stp q3, q2, [x20, #16]
	b LBB389_266
LBB389_33:
	mov x9, #30064771072
	movk x9, #32760, lsl #48
	cmp x8, x9
	mov x26, x11
	ccmp w11, #0, #4, eq
	b.ne LBB389_66
LBB389_34:
	stp x21, x23, [x29, #-160]
	ldp x9, x8, [sp, #56]
	stp x9, x8, [x29, #-144]
	ldp x9, x8, [sp, #72]
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
	b.ne LBB389_52
	ldr w3, [x20, #4]
	ldr w4, [x20, #56]
	add x0, sp, #144
	mov x1, x21
	mov x2, x23
	bl lyng_js_vm::vm::values::<impl lyng_js_vm::vm::Vm>::value_to_property_key
	ldr x8, [sp, #144]
	ldr w5, [sp, #152]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_53
	ldr w26, [sp, #156]
	mov x8, #9221120237041090560
	cmp x24, x8
	ccmp w25, #0, #4, eq
	ccmp w28, #5, #0, ne
	b.ne LBB389_67
LBB389_39:
	cbz w5, LBB389_79
	cmp w5, #1
	b.ne LBB389_109
	ldr w3, [x20, #4]
	cbz w22, LBB389_196
	ldr x9, [x21, #104]
	sub w8, w3, #1
	cmp x9, x8
	b.ls LBB389_196
	ldr x9, [x21, #96]
	add x9, x9, x8, lsl #5
	ldr x10, [x9, #16]
	sub w8, w22, #1
	cmp x10, x8
	b.ls LBB389_196
	ldr x9, [x9, #8]
	mov w10, #1216
	umaddl x9, w8, w10, x9
	ldr x8, [x9]
	cmp x8, #10
	b.eq LBB389_196
	cmp x8, #3
	b.hi LBB389_196
	ldr w8, [x9, #1204]
	cmp w8, w26
	b.ne LBB389_196
	ldr x8, [x9, #1088]
	cbz x8, LBB389_196
	ldr x12, [x23, #224]
	mov w10, #-1
	add x10, x25, x10
	lsr w11, w10, #6
	cmp x11, x12
	b.hs LBB389_196
	ldr x12, [x23, #216]
	and x10, x10, #0x3f
	ldr x11, [x12, x11, lsl #3]
	mov w12, #80
	umaddl x13, w10, w12, x11
	ldr w10, [x13]
	cmp w10, #1
	b.ne LBB389_196
	mov x14, x26
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
	cbz w12, LBB389_187
	cmp w12, w13
	ccmp x11, x9, #0, eq
	mov x26, x14
	b.eq LBB389_189
	b LBB389_196
LBB389_52:
	ldp q0, q1, [x29, #-192]
	stp q0, q1, [sp, #160]
	str x5, [sp, #152]
LBB389_53:
	mov x9, #-9223372036854775808
	cmp x8, x9
	b.ne LBB389_63
	cbnz w5, LBB389_63
	ldr x27, [sp, #160]
	ldr x8, [sp, #88]
	cbz x8, LBB389_58
	ldr x8, [sp, #88]
	sub x8, x8, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs LBB389_58
	ldr x9, [x21, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB389_58:
	sub x0, x29, #160
	mov x1, x21
	mov x2, x23
	mov x3, x27
	bl lyng_js_vm::vm::exceptions::<impl lyng_js_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-160]
	ldurb w5, [x29, #-152]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_62
	tbnz w5, #0, LBB389_30
LBB389_61:
	mov w9, #0
	mov w5, #0
	mov x8, #-9223372036854775808
	b LBB389_65
LBB389_62:
	ldurb w9, [x29, #-149]
	sub x11, x29, #160
	ldurh w10, [x11, #9]
	orr w9, w10, w9, lsl #16
	ldur w10, [x29, #-148]
	ldur q0, [x11, #24]
	str q0, [sp, #192]
	ldur x27, [x29, #-144]
	ldur x11, [x29, #-120]
	b LBB389_64
LBB389_63:
	lsr w9, w5, #8
	ldr w10, [sp, #156]
	ldr x27, [sp, #160]
	ldur q0, [sp, #168]
	str q0, [sp, #192]
	ldr x11, [sp, #184]
LBB389_64:
	str x11, [sp, #208]
LBB389_65:
	lsl w9, w9, #8
	bfxil x9, x5, #0, #8
	orr x9, x9, x10, lsl #32
	stp x8, x9, [x19]
	str x27, [x19, #16]
	ldr q0, [sp, #192]
	stur q0, [x19, #24]
	ldr x8, [sp, #208]
	str x8, [x19, #40]
	b LBB389_267
LBB389_66:
	mov w5, #2
	stp w5, w26, [sp, #152]
	mov x8, #33
	movk x8, #32768, lsl #48
	str x8, [sp, #144]
	mov x8, #9221120237041090560
	cmp x24, x8
	ccmp w25, #0, #4, eq
	ccmp w28, #5, #0, ne
	b.eq LBB389_39
LBB389_67:
	stp w5, w26, [sp, #8]
	sub x0, x29, #256
	str x25, [sp]
	mov x1, x21
	mov x2, x23
	ldp x3, x4, [sp, #56]
	ldp x5, x6, [sp, #72]
	mov x7, x20
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::get_property_from_value
	sub x0, x29, #160
	sub x5, x29, #256
	mov x1, x21
	mov x2, x23
	ldr x3, [sp, #88]
	mov x4, x20
	bl lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::handle_dispatch_result
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_268
	tbnz w5, #0, LBB389_258
	b LBB389_266
LBB389_71:
	add x0, sp, #96
	mov x1, x21
	mov x2, x23
	mov x3, x25
	mov x4, x26
	bl lyng_js_vm::vm::values::<impl lyng_js_vm::vm::Vm>::mapped_arguments_get
	ldr x8, [sp, #96]
	mov x9, #33
	movk x9, #32768, lsl #48
	add x9, x9, #1
	cmp x8, x9
	b.ne LBB389_76
	mov x0, x23
	mov x1, x25
	mov x2, x26
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::try_fast_typed_array_index_value
	tbz w0, #0, LBB389_114
	mov x27, x1
	b LBB389_117
LBB389_76:
	sub x0, x29, #160
	add x5, sp, #96
	mov x1, x21
	mov x2, x23
	ldr x3, [sp, #88]
	mov x4, x20
	bl lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::handle_dispatch_result
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_118
	tbnz w5, #0, LBB389_117
	b LBB389_266
LBB389_79:
	cbz w22, LBB389_102
	ldr w27, [x20, #4]
	ldr x8, [x21, #104]
	sub w28, w27, #1
	cmp x8, x28
	b.ls LBB389_83
	ldr x8, [x21, #96]
	add x8, x8, x28, lsl #5
	ldr x9, [x8, #16]
	sub w24, w22, #1
	cmp x9, x24
	b.ls LBB389_83
	ldr x8, [x8, #8]
	mov w9, #1216
	umaddl x8, w24, w9, x8
	ldr x9, [x8]
	cmp x9, #10
	ccmp x9, #4, #2, ne
	b.lo LBB389_166
LBB389_83:
	ldr w27, [x20, #4]
	ldr x8, [x21, #104]
	sub w28, w27, #1
	cmp x8, x28
	b.ls LBB389_102
	ldr x8, [x21, #96]
	add x8, x8, x28, lsl #5
	ldr x9, [x8, #16]
	sub w24, w22, #1
	cmp x9, x24
	b.ls LBB389_102
	ldr x8, [x8, #8]
	mov w9, #1216
	umaddl x9, w24, w9, x8
	ldr x8, [x9]
	cmp x8, #10
	b.eq LBB389_102
	cmp x8, #3
	b.hi LBB389_102
	ldrb w8, [x9, #1209]
	cmp w8, #2
	b.ne LBB389_102
	ldrb w8, [x9, #1208]
	cbnz w8, LBB389_102
	str x9, [sp, #40]
	mov w9, #4992
	sub x8, x29, #160
	add x0, x23, x9
	mov x1, x23
	mov x2, x25
	bl lyng_js_objects::runtime::ObjectRuntime::object_header
	ldurb w8, [x29, #-138]
	cmp w8, #3
	ldr x11, [sp, #40]
	b.eq LBB389_102
	ldur w10, [x29, #-156]
	ldur w8, [x29, #-144]
	ldurh w9, [x29, #-140]
	ldr w11, [x11, #1184]
	cbz w11, LBB389_94
	cmp w10, w11
	b.ne LBB389_94
	ldr x11, [sp, #40]
	ldrh w11, [x11, #1188]
	cmp w9, w11
	b.eq LBB389_97
LBB389_94:
	ldr x12, [sp, #40]
	ldr w11, [x12, #1192]
	cbz w11, LBB389_102
	cmp w10, w11
	b.ne LBB389_102
	ldrh w10, [x12, #1196]
	cmp w9, w10
	b.ne LBB389_102
LBB389_97:
	cbz w8, LBB389_102
	ldr x9, [x23, #640]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB389_102
	ldr x9, [x23, #632]
	mov w10, #24
	umaddl x8, w8, w10, x9
	ldrb w9, [x8, #19]
	cmp w9, #1
	b.ne LBB389_102
	ldr x10, [x8, #8]
	mov w9, w26
	cmp x10, x9
	b.ls LBB389_102
	ldr x8, [x8]
	ldr x9, [x8, x9, lsl #3]
	mov x8, #1
	movk x8, #9, lsl #32
	movk x8, #32760, lsl #48
	str x9, [sp, #40]
	cmp x9, x8
	b.ne LBB389_177
LBB389_102:
	ldr w2, [x20, #4]
	mov x0, x21
	mov x1, x23
	mov x3, x22
	mov x4, x25
	mov x5, x26
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::try_keyed_dense_index_load_inline_cache_hit
	tbnz w0, #0, LBB389_18
	add x0, sp, #224
	mov x1, x21
	mov x2, x23
	mov x3, x25
	mov x4, x26
	bl lyng_js_vm::vm::values::<impl lyng_js_vm::vm::Vm>::mapped_arguments_get
	ldr x8, [sp, #224]
	mov x9, #33
	movk x9, #32768, lsl #48
	add x9, x9, #1
	cmp x8, x9
	b.ne LBB389_155
	mov x0, x23
	mov x1, x25
	mov x2, x26
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::try_fast_typed_array_index_value
	tbz w0, #0, LBB389_158
	mov x27, x1
	b LBB389_161
LBB389_109:
	stp w5, w26, [sp, #8]
	add x0, sp, #368
	str x25, [sp]
	mov x1, x21
	mov x2, x23
	ldp x3, x4, [sp, #56]
	ldp x5, x6, [sp, #72]
	mov x7, x20
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::get_property_from_value
	sub x0, x29, #160
	add x5, sp, #368
	mov x1, x21
	mov x2, x23
	ldr x3, [sp, #88]
	mov x4, x20
	bl lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::handle_dispatch_result
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_268
	tbz w5, #0, LBB389_266
	ldr w1, [x20, #4]
	mov x0, x21
	mov x2, x22
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::observe_keyed_generic_slow_path
	b LBB389_258
LBB389_114:
	sub x0, x29, #160
	mov x1, x23
	mov x2, x25
	mov x3, x26
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::try_fast_own_index_value
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_268
	tbz w5, #0, LBB389_34
LBB389_117:
	ldr w2, [x20, #4]
	mov x0, x21
	mov x1, x23
	mov x3, x22
	mov x4, x25
	mov x5, x26
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::observe_keyed_index_access
	b LBB389_258
LBB389_118:
	sub x9, x29, #160
	ldur q0, [x9, #24]
	str q0, [sp, #192]
	ldur x9, [x29, #-120]
	str x9, [sp, #208]
	lsr w9, w5, #8
	lsr x10, x5, #32
	b LBB389_65
LBB389_119:
	ldr x8, [x8, #1136]
	cbz w8, LBB389_13
	stp x8, x11, [sp, #32]
	str x10, [sp, #24]
	mov w9, #4992
	sub x8, x29, #160
	add x0, x23, x9
	mov x1, x23
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
	ldr x9, [x23, #640]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB389_13
	ldr x9, [x23, #632]
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
	b.ls LBB389_133
	ldr x8, [x21, #96]
	ldr x9, [sp, #40]
	add x8, x8, x9, lsl #5
	ldr x9, [x8, #16]
	ldr x10, [sp, #24]
	cmp x9, x10
	b.ls LBB389_133
	ldr x8, [x8, #8]
	mov w9, #1216
	ldr x10, [sp, #24]
	umaddl x0, w10, w9, x8
	ldr x8, [x0]
	cmp x8, #10
	b.eq LBB389_133
	bl lyng_js_vm::vm::feedback::FeedbackSiteState::record_execution
LBB389_133:
	ldp x0, x1, [x21, #120]
	mov x2, x27
	bl lyng_js_vm::vm::tiering::<impl lyng_js_vm::vm::Vm>::observe_tier_feedback_event
	ldr w8, [x20, #20]
	ldr x1, [x21, #32]
	ldr w9, [sp, #52]
	add w0, w8, w9
	cmp x1, x0
	b.ls LBB389_274
	ldr x8, [x21, #24]
	ldr x9, [sp, #32]
	str x9, [x8, x0, lsl #3]
	ldr w8, [x20, #56]
	ldr w9, [sp, #48]
	adds w8, w8, w9
	b.lo LBB389_20
	b LBB389_260
LBB389_135:
	ldrb w8, [x9, #1209]
	cmp w8, #2
	b.ne LBB389_16
	ldrb w8, [x9, #1208]
	cbnz w8, LBB389_16
	str x9, [sp, #40]
	str x11, [sp, #24]
	str w10, [sp, #32]
	mov w9, #4992
	sub x8, x29, #160
	add x0, x23, x9
	mov x1, x23
	mov x2, x25
	bl lyng_js_objects::runtime::ObjectRuntime::object_header
	ldurb w8, [x29, #-138]
	cmp w8, #3
	ldr x12, [sp, #40]
	b.eq LBB389_16
	ldur w10, [x29, #-156]
	ldur w8, [x29, #-144]
	ldurh w9, [x29, #-140]
	ldr w11, [x12, #1184]
	cbz w11, LBB389_142
	cmp w10, w11
	b.ne LBB389_142
	ldrh w11, [x12, #1188]
	cmp w9, w11
	b.eq LBB389_145
LBB389_142:
	ldr w11, [x12, #1192]
	cbz w11, LBB389_16
	cmp w10, w11
	b.ne LBB389_16
	ldrh w10, [x12, #1196]
	cmp w9, w10
	b.ne LBB389_16
LBB389_145:
	cbz w8, LBB389_16
	ldr x9, [x23, #640]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB389_16
	ldr x9, [x23, #632]
	mov w10, #24
	umaddl x8, w8, w10, x9
	ldrb w9, [x8, #19]
	cmp w9, #1
	b.ne LBB389_16
	ldr x10, [x8, #8]
	and x9, x26, #0x7fffffff
	cmp x10, x9
	b.ls LBB389_16
	ldr x8, [x8]
	ldr x9, [x8, x9, lsl #3]
	mov x8, #1
	movk x8, #9, lsl #32
	movk x8, #32760, lsl #48
	str x9, [sp, #40]
	cmp x9, x8
	b.eq LBB389_16
	ldr x8, [x21, #104]
	cmp x8, x27
	b.ls LBB389_154
	ldr x8, [x21, #96]
	add x8, x8, x27, lsl #5
	ldr x9, [x8, #16]
	ldr x10, [sp, #24]
	cmp x9, x10
	b.ls LBB389_154
	ldr x8, [x8, #8]
	mov w9, #1216
	ldr x10, [sp, #24]
	umaddl x0, w10, w9, x8
	ldr x8, [x0]
	cmp x8, #10
	b.eq LBB389_154
	bl lyng_js_vm::vm::feedback::FeedbackSiteState::record_execution
LBB389_154:
	ldp x0, x1, [x21, #120]
	ldr w2, [sp, #32]
	b LBB389_182
LBB389_155:
	sub x0, x29, #160
	add x5, sp, #224
	mov x1, x21
	mov x2, x23
	ldr x3, [sp, #88]
	mov x4, x20
	bl lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::handle_dispatch_result
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_268
LBB389_157:
	tbnz w5, #0, LBB389_161
	b LBB389_266
LBB389_158:
	sub x0, x29, #160
	mov x1, x23
	mov x2, x25
	mov x3, x26
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::try_fast_own_index_value
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_268
	tbz w5, #0, LBB389_184
LBB389_161:
	ldr w2, [x20, #4]
	mov x0, x21
	mov x1, x23
	mov x3, x22
	mov x4, x25
	mov x5, x26
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::observe_keyed_index_access
	b LBB389_258
LBB389_162:
Lloh1083:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
Lloh1084:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov x0, x5
	mov x2, x1
	bl core::slice::index::slice_index_fail
	b LBB389_273
LBB389_163:
	sub x0, x29, #160
	mov w3, #1
	mov x4, x27
	bl lyng_js_vm::vm::dispatch::decode_abc_operands_wide
	ldur x8, [x29, #-160]
	cmp x8, x21
	b.ne LBB389_22
	ldurh w13, [x29, #-148]
	ldurh w8, [x29, #-146]
	ldurh w9, [x29, #-144]
	ldur w22, [x29, #-152]
	ldur w12, [x29, #-140]
	ldr x21, [x20, #80]
	ldr w10, [x20, #20]
	ldr x1, [x21, #32]
	add w0, w10, w8
	cmp x1, x0
	b.hi LBB389_5
LBB389_165:
Lloh1085:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.61@PAGE
Lloh1086:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.61@PAGEOFF
	b LBB389_272
LBB389_166:
	ldr x8, [x8, #1136]
	cbz w8, LBB389_83
	str x8, [sp, #40]
	mov w9, #4992
	sub x8, x29, #160
	add x0, x23, x9
	mov x1, x23
	mov x2, x25
	bl lyng_js_objects::runtime::ObjectRuntime::object_header
	ldurb w8, [x29, #-138]
	cmp w8, #3
	ldr x9, [sp, #40]
	b.eq LBB389_83
	cbz w9, LBB389_83
	ldur w8, [x29, #-156]
	cmp w8, w9
	b.ne LBB389_83
	ldurh w8, [x29, #-140]
	lsr x9, x9, #32
	cmp w8, w9, uxth
	b.ne LBB389_83
	ldur w8, [x29, #-144]
	cbz w8, LBB389_83
	ldr x9, [x23, #640]
	sub w8, w8, #1
	cmp x9, x8
	b.ls LBB389_83
	ldr x9, [x23, #632]
	mov w10, #24
	umaddl x8, w8, w10, x9
	ldrb w9, [x8, #19]
	cmp w9, #1
	b.ne LBB389_83
	ldr x10, [x8, #8]
	mov w9, w26
	cmp x10, x9
	b.ls LBB389_83
	ldr x8, [x8]
	ldr x9, [x8, x9, lsl #3]
	mov x8, #1
	movk x8, #9, lsl #32
	movk x8, #32760, lsl #48
	str x9, [sp, #40]
	cmp x9, x8
	b.eq LBB389_83
LBB389_177:
	ldr x8, [x21, #104]
	cmp x8, x28
	b.ls LBB389_181
	ldr x8, [x21, #96]
	add x8, x8, x28, lsl #5
	ldr x9, [x8, #16]
	cmp x9, x24
	b.ls LBB389_181
	ldr x8, [x8, #8]
	mov w9, #1216
	umaddl x0, w24, w9, x8
	ldr x8, [x0]
	cmp x8, #10
	b.eq LBB389_181
	bl lyng_js_vm::vm::feedback::FeedbackSiteState::record_execution
LBB389_181:
	ldp x0, x1, [x21, #120]
	mov x2, x27
LBB389_182:
	bl lyng_js_vm::vm::tiering::<impl lyng_js_vm::vm::Vm>::observe_tier_feedback_event
	ldr w8, [x20, #20]
	ldr x1, [x21, #32]
	ldr w9, [sp, #52]
	add w0, w8, w9
	cmp x1, x0
	b.ls LBB389_274
	ldr x8, [x21, #24]
	ldr x9, [sp, #40]
	str x9, [x8, x0, lsl #3]
	ldr w8, [x20, #56]
	ldr w9, [sp, #48]
	adds w8, w8, w9
	b.lo LBB389_20
	b LBB389_260
LBB389_184:
	stp wzr, w26, [sp, #8]
	add x0, sp, #272
	str x25, [sp]
	mov x1, x21
	mov x2, x23
	ldp x3, x4, [sp, #56]
	ldp x5, x6, [sp, #72]
	mov x7, x20
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::get_property_from_value
	sub x0, x29, #160
	add x5, sp, #272
	mov x1, x21
	mov x2, x23
	ldr x3, [sp, #88]
	mov x4, x20
	bl lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::handle_dispatch_result
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.eq LBB389_157
	b LBB389_268
LBB389_187:
	mov x26, x14
	cbnz w13, LBB389_196
	cmp x11, x9
	b.ne LBB389_196
LBB389_189:
	and x9, x8, #0x3fffffff
	tbnz w8, #31, LBB389_195
	mov x26, x14
	cbz w10, LBB389_196
	ldr x11, [x23, #640]
	sub w8, w10, #1
	cmp x11, x8
	b.ls LBB389_196
	ldr x10, [x23, #632]
	mov w11, #24
	umaddl x8, w8, w11, x10
	ldrb w10, [x8, #19]
	cmp w10, #1
	b.ne LBB389_196
	ldr x10, [x8, #8]
	cmp x9, x10
	b.hs LBB389_196
	ldr x8, [x8]
	b LBB389_243
LBB389_195:
	cmp x9, #3
	mov x26, x14
	b.ls LBB389_242
LBB389_196:
	ldr x8, [x23, #224]
	mov w9, #-1
	add x9, x25, x9
	lsr w10, w9, #6
	cmp x10, x8
	b.hs LBB389_209
	ldr x11, [x23, #216]
	and x12, x9, #0x3f
	ldr x11, [x11, x10, lsl #3]
	mov w13, #80
	umaddl x14, w12, w13, x11
	mov x13, x14
	ldr w11, [x13], #8
	cmp w11, #1
	b.ne LBB389_209
	ldr w16, [x14, #52]
	cbz w16, LBB389_209
	cbz w22, LBB389_255
	ldr x12, [x21, #104]
	sub w11, w3, #1
	cmp x12, x11
	b.ls LBB389_211
	ldr x15, [x21, #96]
	add x17, x15, x11, lsl #5
	ldr x0, [x17, #16]
	sub w15, w22, #1
	cmp x0, x15
	b.ls LBB389_211
	ldr x17, [x17, #8]
	mov w0, #1216
	umaddl x17, w15, w0, x17
	ldr x15, [x17]
	cmp x15, #10
	b.eq LBB389_211
	cmp x15, #3
	b.hi LBB389_211
	ldr x15, [x17, #1144]
	cbz x15, LBB389_221
	lsr x0, x15, #32
	cbz x0, LBB389_221
	ldr w1, [x17, #1160]
	cmp w1, w26
	b.ne LBB389_221
	cmp w16, w0
	b.ne LBB389_221
	mov x16, #0
	b LBB389_226
LBB389_209:
	cbz w22, LBB389_255
	ldr x12, [x21, #104]
	sub w11, w3, #1
LBB389_211:
	cmp x12, x11
	b.ls LBB389_255
	ldr x12, [x21, #96]
	add x12, x12, x11, lsl #5
	ldr x13, [x12, #16]
	sub w11, w22, #1
	cmp x13, x11
	b.ls LBB389_255
	ldr x12, [x12, #8]
	mov w13, #1216
	umaddl x12, w11, w13, x12
	ldr x11, [x12]
	cmp x11, #10
	b.eq LBB389_255
	cmp x11, #3
	b.hi LBB389_255
	ldr w11, [x12, #1204]
	cmp w11, w26
	b.ne LBB389_255
	ldr x13, [x12, #1104]
	cbz x13, LBB389_255
	cmp x10, x8
	b.hs LBB389_255
	ldr x11, [x23, #216]
	and x9, x9, #0x3f
	ldr x10, [x11, x10, lsl #3]
	mov w14, #80
	umaddl x15, w9, w14, x10
	ldr w9, [x15]
	cmp w9, #1
	b.ne LBB389_255
	mov x17, x26
	ldr x9, [x12, #1112]
	ldr x14, [x12, #1120]
	ldr x10, [x12, #1128]
	ldp w12, w16, [x15, #48]
	ldr x15, [x15, #40]
	cmp w12, #0
	cbz w16, LBB389_233
	ccmp w16, w13, #0, ne
	b LBB389_234
LBB389_221:
	ldr x15, [x17, #1152]
	cbz x15, LBB389_211
	lsr x0, x15, #32
	cbz x0, LBB389_211
	ldr w1, [x17, #1164]
	cmp w1, w26
	b.ne LBB389_211
	cmp w16, w0
	b.ne LBB389_211
	mov w16, #1
LBB389_226:
	add x16, x17, x16, lsl #3
	ldr x17, [x16, #1168]
	ldr w16, [x14, #56]
	ldr x14, [x14, #40]
	ldr q0, [x13]
	stur q0, [x29, #-160]
	ldr q0, [x13, #16]
	stur q0, [x29, #-144]
	cmp x14, x17
	b.ne LBB389_211
	tbnz w15, #31, LBB389_239
	cbz w16, LBB389_211
	ldr x14, [x23, #640]
	sub w13, w16, #1
	cmp x14, x13
	b.ls LBB389_211
	ldr x14, [x23, #632]
	mov w16, #24
	umaddl x13, w13, w16, x14
	ldrb w14, [x13, #19]
	cmp w14, #1
	b.ne LBB389_211
	ldr x16, [x13, #8]
	and x14, x15, #0x3fffffff
	cmp x16, x14
	b.ls LBB389_211
	ldr x8, [x13]
	add x8, x8, x14, lsl #3
	b LBB389_241
LBB389_233:
	ccmp w13, #0, #0, ne
LBB389_234:
	ccmp x15, x14, #0, eq
	mov x26, x17
	b.ne LBB389_255
	sub w12, w12, #1
	lsr x13, x12, #6
	cmp x13, x8
	b.hs LBB389_255
	and x8, x12, #0x3f
	ldr x11, [x11, x13, lsl #3]
	mov w12, #80
	umaddl x13, w8, w12, x11
	ldr w8, [x13]
	cmp w8, #1
	b.ne LBB389_255
	ldp w12, w11, [x13, #52]
	ldr x8, [x13, #40]
	ldur q0, [x13, #8]
	stur q0, [x29, #-160]
	ldur q0, [x13, #24]
	stur q0, [x29, #-144]
	lsr x13, x9, #32
	cmp x13, #0
	csel w13, w13, wzr, ne
	cbz w12, LBB389_246
	cmp w12, w13
	ccmp x8, x10, #0, eq
	mov x26, x17
	b.eq LBB389_248
	b LBB389_255
LBB389_239:
	tst w15, #0x3ffffffc
	b.ne LBB389_211
	and x8, x15, #0x3fffffff
	sub x9, x29, #160
	add x8, x9, x8, lsl #3
LBB389_241:
	ldr x23, [x8]
	mov x0, x21
	mov x1, x3
	mov x2, x22
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::record_feedback_slot
	b LBB389_244
LBB389_242:
	sub x8, x29, #160
LBB389_243:
	ldr x23, [x8, x9, lsl #3]
	mov x0, x21
	mov x1, x3
	mov x2, x22
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::record_feedback_slot
LBB389_244:
	ldr w8, [x20, #20]
	ldr x1, [x21, #32]
	ldr w9, [sp, #52]
	add w0, w8, w9
	cmp x1, x0
	b.ls LBB389_274
	ldr x8, [x21, #24]
	str x23, [x8, x0, lsl #3]
	ldr w8, [x20, #56]
	ldr w9, [sp, #48]
	adds w8, w8, w9
	b.lo LBB389_20
	b LBB389_260
LBB389_246:
	mov x26, x17
	cbnz w13, LBB389_255
	cmp x8, x10
	b.ne LBB389_255
LBB389_248:
	and x8, x9, #0x3fffffff
	tbnz w9, #31, LBB389_254
	mov x26, x17
	cbz w11, LBB389_255
	ldr x10, [x23, #640]
	sub w9, w11, #1
	cmp x10, x9
	b.ls LBB389_255
	ldr x10, [x23, #632]
	mov w11, #24
	umaddl x9, w9, w11, x10
	ldrb w10, [x9, #19]
	cmp w10, #1
	b.ne LBB389_255
	ldr x10, [x9, #8]
	cmp x8, x10
	b.hs LBB389_255
	ldr x9, [x9]
	b LBB389_270
LBB389_254:
	cmp x8, #3
	mov x26, x17
	b.ls LBB389_269
LBB389_255:
	ldp x0, x1, [x21, #96]
	mov x2, x23
	mov x4, x22
	mov x5, x25
	mov x24, x26
	mov x6, x26
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::try_keyed_property_load_inline_cache
	tbz w0, #0, LBB389_261
	mov x27, x1
	ldr w1, [x20, #4]
	mov x0, x21
	mov x2, x22
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::record_feedback_slot
LBB389_258:
	ldr w8, [x20, #20]
	ldr x1, [x21, #32]
	ldr w9, [sp, #52]
	add w0, w8, w9
	cmp x1, x0
	b.ls LBB389_274
	ldr x8, [x21, #24]
	str x27, [x8, x0, lsl #3]
	ldr w8, [x20, #56]
	ldr w9, [sp, #48]
	adds w8, w8, w9
	b.lo LBB389_20
LBB389_260:
Lloh1087:
	adrp x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGE
Lloh1088:
	add x0, x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGEOFF
Lloh1089:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGE
Lloh1090:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGEOFF
	mov w1, #41
	bl core::option::expect_failed
	b LBB389_273
LBB389_261:
	mov w8, #1
	stp w8, w24, [sp, #8]
	str x25, [sp]
	add x0, sp, #320
	mov x1, x21
	mov x2, x23
	ldp x3, x4, [sp, #56]
	ldp x5, x6, [sp, #72]
	mov x7, x20
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::get_property_from_value
	sub x0, x29, #160
	add x5, sp, #320
	mov x1, x21
	mov x2, x23
	ldr x3, [sp, #88]
	mov x4, x20
	bl lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::handle_dispatch_result
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_268
	tbz w5, #0, LBB389_266
	ldr w2, [x20, #4]
	mov x0, x21
	mov x1, x23
	mov x3, x22
	mov x4, x25
	mov x5, x24
	mov w6, #0
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::observe_keyed_atom_slow_path
	b LBB389_258
LBB389_266:
	ldr x8, [x20, #128]
	ldr x8, [x8, #48]
	ldr w9, [x20, #56]
	ldrb w8, [x8, x9]
Lloh1091:
	adrp x9, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
Lloh1092:
	add x9, x9, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x10, [x9, x8, lsl #3]
	mov x8, #33
	movk x8, #32768, lsl #48
	stp x8, x10, [x19]
LBB389_267:
	add sp, sp, #592
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
LBB389_268:
	.cfi_restore_state
	sub x9, x29, #160
	ldur q0, [x9, #24]
	str q0, [sp, #192]
	ldur x9, [x29, #-120]
	str x9, [sp, #208]
	lsr w9, w5, #8
	lsr x10, x5, #32
	b LBB389_65
LBB389_269:
	sub x9, x29, #160
LBB389_270:
	ldr x23, [x9, x8, lsl #3]
	mov x0, x21
	mov x1, x3
	mov x2, x22
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::record_feedback_slot
	b LBB389_244
LBB389_271:
Lloh1093:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.61@PAGE
Lloh1094:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.61@PAGEOFF
	mov x0, x8
LBB389_272:
	bl core::panicking::panic_bounds_check
LBB389_273:
	brk #0x1
LBB389_274:
Lloh1095:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.38@PAGE
Lloh1096:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.38@PAGEOFF
	b LBB389_272
	bl core::panicking::panic_cannot_unwind
	.loh AdrpAdd	Lloh1083, Lloh1084
	.loh AdrpAdd	Lloh1085, Lloh1086
	.loh AdrpAdd	Lloh1089, Lloh1090
	.loh AdrpAdd	Lloh1087, Lloh1088
	.loh AdrpAdd	Lloh1091, Lloh1092
	.loh AdrpAdd	Lloh1093, Lloh1094
	.loh AdrpAdd	Lloh1095, Lloh1096
