lyng_js_vm::vm::dispatch_handlers::property::op_get_keyed_property:
L0:
	stp x28, x27, [sp, #-96]!
	stp x26, x25, [sp, #16]
	stp x24, x23, [sp, #32]
	stp x22, x21, [sp, #48]
	stp x20, x19, [sp, #64]
	stp x29, x30, [sp, #80]
	add x29, sp, #80
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
	b.lo L1
	mov x20, x0
	mov x21, #33
	movk x21, #32768, lsl #48
	ldr x9, [x9, #48]
	add x1, x9, x5
	cmp w8, #152
	b.ne L2
	and x8, x2, #0x7ffffffffffffffe
	cmp x2, #4
	ccmp x8, #4, #4, hs
	b.eq L3
	ldrh w22, [x1, #4]
	cbz w22, L3
	ldrb w13, [x1, #1]
	ldrb w8, [x1, #2]
	mov w12, #6
	ldrb w9, [x1, #3]
	ldr x21, [x20, #80]
	ldr w10, [x20, #20]
	ldr x1, [x21, #32]
	add w0, w10, w8
	cmp x1, x0
	b.ls L4
L5:
	add w8, w10, w9
	cmp x1, x8
	b.ls L6
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
	b.ls L7
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
	b.ne L8
	mov x26, x11
	tbnz w11, #31, L9
	cbz w22, L10
	ldr x8, [x21, #104]
	sub w11, w27, #1
	cmp x8, x11
	b.ls L11
	ldr x8, [x21, #96]
	add x8, x8, x11, lsl #5
	ldr x9, [x8, #16]
	sub w10, w22, #1
	cmp x9, x10
	b.ls L11
	ldr x8, [x8, #8]
	mov w9, #1216
	umaddl x8, w10, w9, x8
	ldr x9, [x8]
	cmp x9, #10
	ccmp x9, #4, #2, ne
	b.lo L12
L11:
	ldr w10, [x20, #4]
	ldr x8, [x21, #104]
	sub w27, w10, #1
	cmp x8, x27
	b.ls L10
	ldr x8, [x21, #96]
	add x8, x8, x27, lsl #5
	ldr x9, [x8, #16]
	sub w11, w22, #1
	cmp x9, x11
	b.ls L10
	ldr x8, [x8, #8]
	mov w9, #1216
	umaddl x9, w11, w9, x8
	ldr x8, [x9]
	cmp x8, #10
	ccmp x8, #3, #2, ne
	b.ls L13
L10:
	ldr w2, [x20, #4]
	mov x0, x21
	mov x1, x23
	mov x3, x22
	mov x4, x25
	mov x5, x26
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::try_keyed_dense_index_load_inline_cache_hit
	tbz w0, #0, L14
L15:
	mov x8, x1
	ldr w9, [x20, #20]
	ldr x1, [x21, #32]
	ldr w10, [sp, #52]
	add w0, w9, w10
	cmp x1, x0
	b.ls L16
	ldr x9, [x21, #24]
	str x8, [x9, x0, lsl #3]
	ldr w8, [x20, #56]
	ldr w9, [sp, #48]
	adds w8, w8, w9
	b.hs L17
L18:
	str w8, [x20, #56]
	b L19
L3:
	sub x8, x21, #12
	stur x8, [x29, #-160]
	stp w27, w5, [x29, #-152]
L20:
	ldp q0, q1, [x29, #-160]
	stp q0, q1, [x19]
	ldur q0, [x29, #-128]
	str q0, [x19, #32]
	b L21
L7:
	mov x0, x23
	mov w1, #5
	bl lyng_js_ops::errors::error_value
	mov x27, x0
	ldr x8, [sp, #88]
	cbz x8, L22
	ldr x8, [sp, #88]
	sub x8, x8, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs L22
	ldr x9, [x21, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
L22:
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
	b.ne L23
	tbz w5, #0, L24
L25:
	ldr w8, [x21, #1640]
	add w8, w8, #1
	str w8, [x21, #1640]
	ldr x8, [sp, #88]
	cbz x8, L19
	ldr x8, [sp, #88]
	sub x8, x8, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs L19
	ldr x9, [x21, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x8]
	str q0, [x20]
	ldp q0, q1, [x8, #48]
	ldp q3, q2, [x8, #16]
	stp q0, q1, [x20, #48]
	stp q3, q2, [x20, #16]
	b L19
L8:
	mov x9, #30064771072
	movk x9, #32760, lsl #48
	cmp x8, x9
	mov x26, x11
	ccmp w11, #0, #4, eq
	b.ne L26
L9:
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
	b.ne L27
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
	b.ne L28
	ldr w26, [sp, #156]
	mov x8, #9221120237041090560
	cmp x24, x8
	ccmp w25, #0, #4, eq
	ccmp w28, #5, #0, ne
	b.ne L29
L30:
	cbz w5, L31
	cmp w5, #1
	b.ne L32
	ldr w3, [x20, #4]
	cbz w22, L33
	ldr x9, [x21, #104]
	sub w8, w3, #1
	cmp x9, x8
	b.ls L33
	ldr x9, [x21, #96]
	add x9, x9, x8, lsl #5
	ldr x10, [x9, #16]
	sub w8, w22, #1
	cmp x10, x8
	b.ls L33
	ldr x9, [x9, #8]
	mov w10, #1216
	umaddl x9, w8, w10, x9
	ldr x8, [x9]
	cmp x8, #10
	b.eq L33
	cmp x8, #3
	b.hi L33
	ldr w8, [x9, #1204]
	cmp w8, w26
	b.ne L33
	ldr x8, [x9, #1088]
	cbz x8, L33
	ldr x12, [x23, #224]
	mov w10, #-1
	add x10, x25, x10
	lsr w11, w10, #6
	cmp x11, x12
	b.hs L33
	ldr x12, [x23, #216]
	and x10, x10, #0x3f
	ldr x11, [x12, x11, lsl #3]
	mov w12, #80
	umaddl x13, w10, w12, x11
	ldr w10, [x13]
	cmp w10, #1
	b.ne L33
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
	cbz w12, L34
	cmp w12, w13
	ccmp x11, x9, #0, eq
	mov x26, x14
	b.eq L35
	b L33
L27:
	ldp q0, q1, [x29, #-192]
	stp q0, q1, [sp, #160]
	str x5, [sp, #152]
L28:
	mov x9, #-9223372036854775808
	cmp x8, x9
	b.ne L36
	cbnz w5, L36
	ldr x27, [sp, #160]
	ldr x8, [sp, #88]
	cbz x8, L37
	ldr x8, [sp, #88]
	sub x8, x8, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs L37
	ldr x9, [x21, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
L37:
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
	b.ne L23
	tbnz w5, #0, L25
L24:
	mov w9, #0
	mov w5, #0
	mov x8, #-9223372036854775808
	b L38
L23:
	ldurb w9, [x29, #-149]
	sub x11, x29, #160
	ldurh w10, [x11, #9]
	orr w9, w10, w9, lsl #16
	ldur w10, [x29, #-148]
	ldur q0, [x11, #24]
	str q0, [sp, #192]
	ldur x27, [x29, #-144]
	ldur x11, [x29, #-120]
	b L39
L36:
	lsr w9, w5, #8
	ldr w10, [sp, #156]
	ldr x27, [sp, #160]
	ldur q0, [sp, #168]
	str q0, [sp, #192]
	ldr x11, [sp, #184]
L39:
	str x11, [sp, #208]
L38:
	lsl w9, w9, #8
	bfxil x9, x5, #0, #8
	orr x9, x9, x10, lsl #32
	stp x8, x9, [x19]
	str x27, [x19, #16]
	ldr q0, [sp, #192]
	stur q0, [x19, #24]
	ldr x8, [sp, #208]
	str x8, [x19, #40]
	b L21
L26:
	mov w5, #2
	stp w5, w26, [sp, #152]
	mov x8, #33
	movk x8, #32768, lsl #48
	str x8, [sp, #144]
	mov x8, #9221120237041090560
	cmp x24, x8
	ccmp w25, #0, #4, eq
	ccmp w28, #5, #0, ne
	b.eq L30
L29:
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
	b.ne L40
	tbnz w5, #0, L41
	b L19
L14:
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
	b.ne L42
	mov x0, x23
	mov x1, x25
	mov x2, x26
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::try_fast_typed_array_index_value
	tbz w0, #0, L43
	mov x27, x1
	b L44
L42:
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
	b.ne L45
	tbnz w5, #0, L44
	b L19
L31:
	cbz w22, L46
	ldr w27, [x20, #4]
	ldr x8, [x21, #104]
	sub w28, w27, #1
	cmp x8, x28
	b.ls L47
	ldr x8, [x21, #96]
	add x8, x8, x28, lsl #5
	ldr x9, [x8, #16]
	sub w24, w22, #1
	cmp x9, x24
	b.ls L47
	ldr x8, [x8, #8]
	mov w9, #1216
	umaddl x8, w24, w9, x8
	ldr x9, [x8]
	cmp x9, #10
	ccmp x9, #4, #2, ne
	b.lo L48
L47:
	ldr w27, [x20, #4]
	ldr x8, [x21, #104]
	sub w28, w27, #1
	cmp x8, x28
	b.ls L46
	ldr x8, [x21, #96]
	add x8, x8, x28, lsl #5
	ldr x9, [x8, #16]
	sub w24, w22, #1
	cmp x9, x24
	b.ls L46
	ldr x8, [x8, #8]
	mov w9, #1216
	umaddl x9, w24, w9, x8
	ldr x8, [x9]
	cmp x8, #10
	b.eq L46
	cmp x8, #3
	b.hi L46
	ldrb w8, [x9, #1209]
	cmp w8, #2
	b.ne L46
	ldrb w8, [x9, #1208]
	cbnz w8, L46
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
	b.eq L46
	ldur w10, [x29, #-156]
	ldur w8, [x29, #-144]
	ldurh w9, [x29, #-140]
	ldr w11, [x11, #1184]
	cbz w11, L49
	cmp w10, w11
	b.ne L49
	ldr x11, [sp, #40]
	ldrh w11, [x11, #1188]
	cmp w9, w11
	b.eq L50
L49:
	ldr x12, [sp, #40]
	ldr w11, [x12, #1192]
	cbz w11, L46
	cmp w10, w11
	b.ne L46
	ldrh w10, [x12, #1196]
	cmp w9, w10
	b.ne L46
L50:
	cbz w8, L46
	ldr x9, [x23, #640]
	sub w8, w8, #1
	cmp x9, x8
	b.ls L46
	ldr x9, [x23, #632]
	mov w10, #24
	umaddl x8, w8, w10, x9
	ldrb w9, [x8, #19]
	cmp w9, #1
	b.ne L46
	ldr x10, [x8, #8]
	mov w9, w26
	cmp x10, x9
	b.ls L46
	ldr x8, [x8]
	ldr x9, [x8, x9, lsl #3]
	mov x8, #1
	movk x8, #9, lsl #32
	movk x8, #32760, lsl #48
	str x9, [sp, #40]
	cmp x9, x8
	b.ne L51
L46:
	ldr w2, [x20, #4]
	mov x0, x21
	mov x1, x23
	mov x3, x22
	mov x4, x25
	mov x5, x26
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::try_keyed_dense_index_load_inline_cache_hit
	tbnz w0, #0, L15
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
	b.ne L52
	mov x0, x23
	mov x1, x25
	mov x2, x26
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::try_fast_typed_array_index_value
	tbz w0, #0, L53
	mov x27, x1
	b L54
L32:
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
	b.ne L40
	tbz w5, #0, L19
	ldr w1, [x20, #4]
	mov x0, x21
	mov x2, x22
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::observe_keyed_generic_slow_path
	b L41
L43:
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
	b.ne L40
	tbz w5, #0, L9
L44:
	ldr w2, [x20, #4]
	mov x0, x21
	mov x1, x23
	mov x3, x22
	mov x4, x25
	mov x5, x26
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::observe_keyed_index_access
	b L41
L45:
	sub x9, x29, #160
	ldur q0, [x9, #24]
	str q0, [sp, #192]
	ldur x9, [x29, #-120]
	str x9, [sp, #208]
	lsr w9, w5, #8
	lsr x10, x5, #32
	b L38
L12:
	ldr x8, [x8, #1136]
	cbz w8, L11
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
	b.eq L11
	cbz w10, L11
	ldur w8, [x29, #-156]
	cmp w8, w10
	b.ne L11
	ldur w8, [x29, #-144]
	ldurh w9, [x29, #-140]
	lsr x10, x10, #32
	cmp w9, w10, uxth
	ccmp w8, #0, #4, eq
	b.eq L11
	ldr x9, [x23, #640]
	sub w8, w8, #1
	cmp x9, x8
	b.ls L11
	ldr x9, [x23, #632]
	mov w10, #24
	umaddl x8, w8, w10, x9
	ldrb w9, [x8, #19]
	cmp w9, #1
	b.ne L11
	ldr x10, [x8, #8]
	and x9, x26, #0x7fffffff
	cmp x10, x9
	b.ls L11
	ldr x8, [x8]
	ldr x9, [x8, x9, lsl #3]
	mov x8, #1
	movk x8, #9, lsl #32
	movk x8, #32760, lsl #48
	str x9, [sp, #32]
	cmp x9, x8
	b.eq L11
	ldr x8, [x21, #104]
	cmp x8, x11
	b.ls L55
	ldr x8, [x21, #96]
	ldr x9, [sp, #40]
	add x8, x8, x9, lsl #5
	ldr x9, [x8, #16]
	ldr x10, [sp, #24]
	cmp x9, x10
	b.ls L55
	ldr x8, [x8, #8]
	mov w9, #1216
	ldr x10, [sp, #24]
	umaddl x0, w10, w9, x8
	ldr x8, [x0]
	cmp x8, #10
	b.eq L55
	bl lyng_js_vm::vm::feedback::FeedbackSiteState::record_execution
L55:
	ldp x0, x1, [x21, #120]
	mov x2, x27
	bl lyng_js_vm::vm::tiering::<impl lyng_js_vm::vm::Vm>::observe_tier_feedback_event
	ldr w8, [x20, #20]
	ldr x1, [x21, #32]
	ldr w9, [sp, #52]
	add w0, w8, w9
	cmp x1, x0
	b.ls L16
	ldr x8, [x21, #24]
	ldr x9, [sp, #32]
	str x9, [x8, x0, lsl #3]
	ldr w8, [x20, #56]
	ldr w9, [sp, #48]
	adds w8, w8, w9
	b.lo L18
	b L17
L13:
	ldrb w8, [x9, #1209]
	cmp w8, #2
	b.ne L10
	ldrb w8, [x9, #1208]
	cbnz w8, L10
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
	b.eq L10
	ldur w10, [x29, #-156]
	ldur w8, [x29, #-144]
	ldurh w9, [x29, #-140]
	ldr w11, [x12, #1184]
	cbz w11, L56
	cmp w10, w11
	b.ne L56
	ldrh w11, [x12, #1188]
	cmp w9, w11
	b.eq L57
L56:
	ldr w11, [x12, #1192]
	cbz w11, L10
	cmp w10, w11
	b.ne L10
	ldrh w10, [x12, #1196]
	cmp w9, w10
	b.ne L10
L57:
	cbz w8, L10
	ldr x9, [x23, #640]
	sub w8, w8, #1
	cmp x9, x8
	b.ls L10
	ldr x9, [x23, #632]
	mov w10, #24
	umaddl x8, w8, w10, x9
	ldrb w9, [x8, #19]
	cmp w9, #1
	b.ne L10
	ldr x10, [x8, #8]
	and x9, x26, #0x7fffffff
	cmp x10, x9
	b.ls L10
	ldr x8, [x8]
	ldr x9, [x8, x9, lsl #3]
	mov x8, #1
	movk x8, #9, lsl #32
	movk x8, #32760, lsl #48
	str x9, [sp, #40]
	cmp x9, x8
	b.eq L10
	ldr x8, [x21, #104]
	cmp x8, x27
	b.ls L58
	ldr x8, [x21, #96]
	add x8, x8, x27, lsl #5
	ldr x9, [x8, #16]
	ldr x10, [sp, #24]
	cmp x9, x10
	b.ls L58
	ldr x8, [x8, #8]
	mov w9, #1216
	ldr x10, [sp, #24]
	umaddl x0, w10, w9, x8
	ldr x8, [x0]
	cmp x8, #10
	b.eq L58
	bl lyng_js_vm::vm::feedback::FeedbackSiteState::record_execution
L58:
	ldp x0, x1, [x21, #120]
	ldr w2, [sp, #32]
	b L59
L52:
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
	b.ne L40
L60:
	tbnz w5, #0, L54
	b L19
L53:
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
	b.ne L40
	tbz w5, #0, L61
L54:
	ldr w2, [x20, #4]
	mov x0, x21
	mov x1, x23
	mov x3, x22
	mov x4, x25
	mov x5, x26
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::observe_keyed_index_access
	b L41
L1:
L62:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
L63:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov x0, x5
	mov x2, x1
	bl core::slice::index::slice_index_fail
	b L64
L2:
	sub x0, x29, #160
	mov w3, #1
	mov x4, x27
	bl lyng_js_vm::vm::dispatch::decode_abc_operands_wide
	ldur x8, [x29, #-160]
	cmp x8, x21
	b.ne L20
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
	b.hi L5
L4:
L65:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.61@PAGE
L66:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.61@PAGEOFF
	b L67
L48:
	ldr x8, [x8, #1136]
	cbz w8, L47
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
	b.eq L47
	cbz w9, L47
	ldur w8, [x29, #-156]
	cmp w8, w9
	b.ne L47
	ldurh w8, [x29, #-140]
	lsr x9, x9, #32
	cmp w8, w9, uxth
	b.ne L47
	ldur w8, [x29, #-144]
	cbz w8, L47
	ldr x9, [x23, #640]
	sub w8, w8, #1
	cmp x9, x8
	b.ls L47
	ldr x9, [x23, #632]
	mov w10, #24
	umaddl x8, w8, w10, x9
	ldrb w9, [x8, #19]
	cmp w9, #1
	b.ne L47
	ldr x10, [x8, #8]
	mov w9, w26
	cmp x10, x9
	b.ls L47
	ldr x8, [x8]
	ldr x9, [x8, x9, lsl #3]
	mov x8, #1
	movk x8, #9, lsl #32
	movk x8, #32760, lsl #48
	str x9, [sp, #40]
	cmp x9, x8
	b.eq L47
L51:
	ldr x8, [x21, #104]
	cmp x8, x28
	b.ls L68
	ldr x8, [x21, #96]
	add x8, x8, x28, lsl #5
	ldr x9, [x8, #16]
	cmp x9, x24
	b.ls L68
	ldr x8, [x8, #8]
	mov w9, #1216
	umaddl x0, w24, w9, x8
	ldr x8, [x0]
	cmp x8, #10
	b.eq L68
	bl lyng_js_vm::vm::feedback::FeedbackSiteState::record_execution
L68:
	ldp x0, x1, [x21, #120]
	mov x2, x27
L59:
	bl lyng_js_vm::vm::tiering::<impl lyng_js_vm::vm::Vm>::observe_tier_feedback_event
	ldr w8, [x20, #20]
	ldr x1, [x21, #32]
	ldr w9, [sp, #52]
	add w0, w8, w9
	cmp x1, x0
	b.ls L16
	ldr x8, [x21, #24]
	ldr x9, [sp, #40]
	str x9, [x8, x0, lsl #3]
	ldr w8, [x20, #56]
	ldr w9, [sp, #48]
	adds w8, w8, w9
	b.lo L18
	b L17
L61:
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
	b.eq L60
	b L40
L34:
	mov x26, x14
	cbnz w13, L33
	cmp x11, x9
	b.ne L33
L35:
	and x9, x8, #0x3fffffff
	tbnz w8, #31, L69
	mov x26, x14
	cbz w10, L33
	ldr x11, [x23, #640]
	sub w8, w10, #1
	cmp x11, x8
	b.ls L33
	ldr x10, [x23, #632]
	mov w11, #24
	umaddl x8, w8, w11, x10
	ldrb w10, [x8, #19]
	cmp w10, #1
	b.ne L33
	ldr x10, [x8, #8]
	cmp x9, x10
	b.hs L33
	ldr x8, [x8]
	b L70
L69:
	cmp x9, #3
	mov x26, x14
	b.ls L71
L33:
	ldr x8, [x23, #224]
	mov w9, #-1
	add x9, x25, x9
	lsr w10, w9, #6
	cmp x10, x8
	b.hs L72
	ldr x11, [x23, #216]
	and x12, x9, #0x3f
	ldr x11, [x11, x10, lsl #3]
	mov w13, #80
	umaddl x14, w12, w13, x11
	mov x13, x14
	ldr w11, [x13], #8
	cmp w11, #1
	b.ne L72
	ldr w16, [x14, #52]
	cbz w16, L72
	cbz w22, L73
	ldr x12, [x21, #104]
	sub w11, w3, #1
	cmp x12, x11
	b.ls L74
	ldr x15, [x21, #96]
	add x17, x15, x11, lsl #5
	ldr x0, [x17, #16]
	sub w15, w22, #1
	cmp x0, x15
	b.ls L74
	ldr x17, [x17, #8]
	mov w0, #1216
	umaddl x17, w15, w0, x17
	ldr x15, [x17]
	cmp x15, #10
	b.eq L74
	cmp x15, #3
	b.hi L74
	ldr x15, [x17, #1144]
	cbz x15, L75
	lsr x0, x15, #32
	cbz x0, L75
	ldr w1, [x17, #1160]
	cmp w1, w26
	b.ne L75
	cmp w16, w0
	b.ne L75
	mov x16, #0
	b L76
L72:
	cbz w22, L73
	ldr x12, [x21, #104]
	sub w11, w3, #1
L74:
	cmp x12, x11
	b.ls L73
	ldr x12, [x21, #96]
	add x12, x12, x11, lsl #5
	ldr x13, [x12, #16]
	sub w11, w22, #1
	cmp x13, x11
	b.ls L73
	ldr x12, [x12, #8]
	mov w13, #1216
	umaddl x12, w11, w13, x12
	ldr x11, [x12]
	cmp x11, #10
	b.eq L73
	cmp x11, #3
	b.hi L73
	ldr w11, [x12, #1204]
	cmp w11, w26
	b.ne L73
	ldr x13, [x12, #1104]
	cbz x13, L73
	cmp x10, x8
	b.hs L73
	ldr x11, [x23, #216]
	and x9, x9, #0x3f
	ldr x10, [x11, x10, lsl #3]
	mov w14, #80
	umaddl x15, w9, w14, x10
	ldr w9, [x15]
	cmp w9, #1
	b.ne L73
	mov x17, x26
	ldr x9, [x12, #1112]
	ldr x14, [x12, #1120]
	ldr x10, [x12, #1128]
	ldp w12, w16, [x15, #48]
	ldr x15, [x15, #40]
	cmp w12, #0
	cbz w16, L77
	ccmp w16, w13, #0, ne
	b L78
L75:
	ldr x15, [x17, #1152]
	cbz x15, L74
	lsr x0, x15, #32
	cbz x0, L74
	ldr w1, [x17, #1164]
	cmp w1, w26
	b.ne L74
	cmp w16, w0
	b.ne L74
	mov w16, #1
L76:
	add x16, x17, x16, lsl #3
	ldr x17, [x16, #1168]
	ldr w16, [x14, #56]
	ldr x14, [x14, #40]
	ldr q0, [x13]
	stur q0, [x29, #-160]
	ldr q0, [x13, #16]
	stur q0, [x29, #-144]
	cmp x14, x17
	b.ne L74
	tbnz w15, #31, L79
	cbz w16, L74
	ldr x14, [x23, #640]
	sub w13, w16, #1
	cmp x14, x13
	b.ls L74
	ldr x14, [x23, #632]
	mov w16, #24
	umaddl x13, w13, w16, x14
	ldrb w14, [x13, #19]
	cmp w14, #1
	b.ne L74
	ldr x16, [x13, #8]
	and x14, x15, #0x3fffffff
	cmp x16, x14
	b.ls L74
	ldr x8, [x13]
	add x8, x8, x14, lsl #3
	b L80
L77:
	ccmp w13, #0, #0, ne
L78:
	ccmp x15, x14, #0, eq
	mov x26, x17
	b.ne L73
	sub w12, w12, #1
	lsr x13, x12, #6
	cmp x13, x8
	b.hs L73
	and x8, x12, #0x3f
	ldr x11, [x11, x13, lsl #3]
	mov w12, #80
	umaddl x13, w8, w12, x11
	ldr w8, [x13]
	cmp w8, #1
	b.ne L73
	ldp w12, w11, [x13, #52]
	ldr x8, [x13, #40]
	ldur q0, [x13, #8]
	stur q0, [x29, #-160]
	ldur q0, [x13, #24]
	stur q0, [x29, #-144]
	lsr x13, x9, #32
	cmp x13, #0
	csel w13, w13, wzr, ne
	cbz w12, L81
	cmp w12, w13
	ccmp x8, x10, #0, eq
	mov x26, x17
	b.eq L82
	b L73
L79:
	tst w15, #0x3ffffffc
	b.ne L74
	and x8, x15, #0x3fffffff
	sub x9, x29, #160
	add x8, x9, x8, lsl #3
L80:
	ldr x23, [x8]
	mov x0, x21
	mov x1, x3
	mov x2, x22
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::record_feedback_slot
	b L83
L71:
	sub x8, x29, #160
L70:
	ldr x23, [x8, x9, lsl #3]
	mov x0, x21
	mov x1, x3
	mov x2, x22
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::record_feedback_slot
L83:
	ldr w8, [x20, #20]
	ldr x1, [x21, #32]
	ldr w9, [sp, #52]
	add w0, w8, w9
	cmp x1, x0
	b.ls L16
	ldr x8, [x21, #24]
	str x23, [x8, x0, lsl #3]
	ldr w8, [x20, #56]
	ldr w9, [sp, #48]
	adds w8, w8, w9
	b.lo L18
	b L17
L81:
	mov x26, x17
	cbnz w13, L73
	cmp x8, x10
	b.ne L73
L82:
	and x8, x9, #0x3fffffff
	tbnz w9, #31, L84
	mov x26, x17
	cbz w11, L73
	ldr x10, [x23, #640]
	sub w9, w11, #1
	cmp x10, x9
	b.ls L73
	ldr x10, [x23, #632]
	mov w11, #24
	umaddl x9, w9, w11, x10
	ldrb w10, [x9, #19]
	cmp w10, #1
	b.ne L73
	ldr x10, [x9, #8]
	cmp x8, x10
	b.hs L73
	ldr x9, [x9]
	b L85
L84:
	cmp x8, #3
	mov x26, x17
	b.ls L86
L73:
	ldp x0, x1, [x21, #96]
	mov x2, x23
	mov x4, x22
	mov x5, x25
	mov x24, x26
	mov x6, x26
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::try_keyed_property_load_inline_cache
	tbz w0, #0, L87
	mov x27, x1
	ldr w1, [x20, #4]
	mov x0, x21
	mov x2, x22
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::record_feedback_slot
L41:
	ldr w8, [x20, #20]
	ldr x1, [x21, #32]
	ldr w9, [sp, #52]
	add w0, w8, w9
	cmp x1, x0
	b.ls L16
	ldr x8, [x21, #24]
	str x27, [x8, x0, lsl #3]
	ldr w8, [x20, #56]
	ldr w9, [sp, #48]
	adds w8, w8, w9
	b.lo L18
L17:
L88:
	adrp x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGE
L89:
	add x0, x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGEOFF
L90:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGE
L91:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGEOFF
	mov w1, #41
	bl core::option::expect_failed
	b L64
L87:
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
	b.ne L40
	tbz w5, #0, L19
	ldr w2, [x20, #4]
	mov x0, x21
	mov x1, x23
	mov x3, x22
	mov x4, x25
	mov x5, x24
	mov w6, #0
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::observe_keyed_atom_slow_path
	b L41
L19:
	ldr x8, [x20, #128]
	ldr x8, [x8, #48]
	ldr w9, [x20, #56]
	ldrb w8, [x8, x9]
L92:
	adrp x9, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
L93:
	add x9, x9, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x10, [x9, x8, lsl #3]
	mov x8, #33
	movk x8, #32768, lsl #48
	stp x8, x10, [x19]
L21:
	add sp, sp, #592
	ldp x29, x30, [sp, #80]
	ldp x20, x19, [sp, #64]
	ldp x22, x21, [sp, #48]
	ldp x24, x23, [sp, #32]
	ldp x26, x25, [sp, #16]
	ldp x28, x27, [sp], #96
	ret
L40:
	sub x9, x29, #160
	ldur q0, [x9, #24]
	str q0, [sp, #192]
	ldur x9, [x29, #-120]
	str x9, [sp, #208]
	lsr w9, w5, #8
	lsr x10, x5, #32
	b L38
L86:
	sub x9, x29, #160
L85:
	ldr x23, [x9, x8, lsl #3]
	mov x0, x21
	mov x1, x3
	mov x2, x22
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::record_feedback_slot
	b L83
L6:
L94:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.61@PAGE
L95:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.61@PAGEOFF
	mov x0, x8
L67:
	bl core::panicking::panic_bounds_check
L64:
	brk #0x1
L16:
L96:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.38@PAGE
L97:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.38@PAGEOFF
	b L67
	bl core::panicking::panic_cannot_unwind
	.loh AdrpAdd	L62, L63
	.loh AdrpAdd	L65, L66
	.loh AdrpAdd	L90, L91
	.loh AdrpAdd	L88, L89
	.loh AdrpAdd	L92, L93
	.loh AdrpAdd	L94, L95
	.loh AdrpAdd	L96, L97
