lyng_js_vm::vm::dispatch_handlers::names::op_load_global:
L0:
	sub sp, sp, #336
	stp x28, x27, [sp, #240]
	stp x26, x25, [sp, #256]
	stp x24, x23, [sp, #272]
	stp x22, x21, [sp, #288]
	stp x20, x19, [sp, #304]
	stp x29, x30, [sp, #320]
	add x29, sp, #320
	mov x19, x8
	ldr w22, [x0, #4]
	ldr w6, [x0, #56]
	ldrb w3, [x0, #148]
	mov w8, #152
	strb w8, [x0, #148]
	ldr x8, [x0, #128]
	ldr x1, [x8, #56]
	subs x2, x1, x6
	b.lo L1
	mov x20, x0
	mov x26, #33
	movk x26, #32768, lsl #48
	ldr x8, [x8, #48]
	add x1, x8, x6
	cmp w3, #152
	b.ne L2
	and x8, x2, #0x7ffffffffffffffe
	cmp x2, #4
	ccmp x8, #4, #4, hs
	b.eq L3
	ldrh w24, [x1, #4]
	cbz w24, L3
	ldrb w10, [x1, #1]
	ldrh w8, [x1, #2]
	mov w9, #6
	stp w9, w10, [sp, #72]
	ldr x13, [x20, #80]
	ldr x9, [x13, #80]
	sub w28, w22, #1
	cmp x9, x28
	b.ls L4
L5:
	ldr x9, [x13, #72]
	ldr x10, [x9, x28, lsl #3]
	cbz x10, L4
	ldr x12, [x10, #80]
	mov w11, w8
	sub x9, x26, #3
	cmp x12, x11
	b.ls L6
	ldr x12, [x10, #72]
	add x11, x12, x11, lsl #4
	ldr w12, [x11]
	cmp w12, #4
	b.eq L6
	ldr w14, [x11, #4]
	cmp w12, #2
	b.ne L7
	str x13, [sp, #64]
	ldr x8, [x10, #464]
	cmp x8, x14
	b.ls L8
	ldr x8, [x10, #456]
	add x8, x8, x14, lsl #3
	ldr w9, [x8, #16]!
	cmp w9, #1
	b.ne L8
	ldr w14, [x8, #4]
L8:
	ldp x23, x8, [x20, #88]
	stp x8, x14, [sp, #48]
	ldp x9, x8, [x20, #104]
	stp x8, x9, [sp, #32]
	ldr x8, [x20, #120]
	str x8, [sp, #24]
	ldr w1, [x20, #12]
	sub x25, x26, #27
	str w1, [sp, #20]
L9:
	mov x27, x1
	ldr x8, [x23, #5168]
	sub w21, w1, #1
	cmp x8, x21
	b.ls L10
	ldr x8, [x23, #5160]
	lsl x9, x21, #7
	ldr x8, [x8, x9]
	cmp x8, x25
	b.eq L10
	tbz x8, #63, L11
L10:
	mov x0, x23
	mov x1, x27
	bl lyng_js_env::agent::environments::<impl lyng_js_env::agent::Agent>::environment_outer
	tbz w0, #0, L12
	cbnz w1, L9
L12:
	mov w9, #0
	sub x8, x26, #24
L13:
	lsr w10, w27, #8
	ldr q0, [sp, #80]
	str q0, [sp, #112]
	ldr x11, [sp, #96]
L14:
	str x11, [sp, #128]
L15:
	lsl w10, w10, #8
	bfxil x10, x27, #0, #8
	orr x9, x10, x9, lsl #32
	stp x8, x9, [x19]
	str x25, [x19, #16]
	ldr q0, [sp, #112]
	stur q0, [x19, #24]
	ldr x8, [sp, #128]
	str x8, [x19, #40]
	b L16
L3:
	sub x8, x26, #12
	stur x8, [x29, #-144]
	stp w22, w6, [x29, #-136]
L17:
	ldp q0, q1, [x29, #-144]
	stp q0, q1, [x19]
	ldur q0, [x29, #-112]
	str q0, [x19, #32]
	b L16
L6:
	mov x10, #0
	b L18
L7:
	ldr x11, [x11, #8]
	orr x10, x12, x14, lsl #32
	b L19
L11:
	sub x8, x29, #144
	mov x0, x23
	mov x1, x27
	ldr x2, [sp, #56]
	bl lyng_js_env::agent::environments::<impl lyng_js_env::agent::Agent>::global_lexical_binding
	ldur w8, [x29, #-144]
	cbz w8, L20
	ldur w21, [x29, #-136]
	mov x27, x8
L21:
	sub x0, x29, #144
	ldr x1, [sp, #64]
	mov x2, x23
	mov x3, x27
	mov w4, #0
	mov x5, x21
	bl lyng_js_vm::vm::loop_iteration::<impl lyng_js_vm::vm::Vm>::environment_for_slot_access
	ldur x8, [x29, #-144]
	ldur w27, [x29, #-136]
	cmp x8, x26
	b.ne L22
	mov x0, x23
	mov x1, x27
	mov x2, x21
	bl lyng_js_env::agent::environments::<impl lyng_js_env::agent::Agent>::environment_slot
	tbz w0, #0, L12
	mov x25, x1
	mov x8, #2
	movk x8, #9, lsl #32
	movk x8, #32760, lsl #48
	cmp x1, x8
	b.ne L23
	mov x0, x23
	mov w1, #3
	bl lyng_js_ops::errors::error_value
	mov x25, x0
	mov w27, #0
	mov x8, #-9223372036854775808
	b L24
L20:
	ldr x8, [x23, #5168]
	cmp x8, x21
	b.ls L25
	ldr x8, [x23, #5160]
	add x8, x8, x21, lsl #7
	ldr x9, [x8]
	cmp x9, x25
	b.eq L25
	tbnz x9, #63, L25
	ldr w8, [x8, #120]
	ldr x9, [x23, #5144]
	sub w8, w8, #1
	cmp x9, x8
	b.ls L25
	ldr x9, [x23, #5136]
	add x8, x9, x8, lsl #5
	ldr x9, [x8]
	mov x10, #-9223372036854775808
	cmp x9, x10
	b.eq L25
	ldr x9, [x8, #16]
	cbz x9, L25
	mov x21, #0
	ldr x8, [x8, #8]
	add x9, x8, x9, lsl #4
	b L26
L22:
	ldur w9, [x29, #-132]
	ldur x25, [x29, #-128]
	ldur q0, [x29, #-120]
	str q0, [sp, #80]
	ldur x10, [x29, #-104]
	str x10, [sp, #96]
L24:
	mov x10, #-9223372036854775808
	cmp x8, x10
	b.ne L13
	cbnz w27, L13
	ldr x22, [x20, #136]
	ldr x1, [sp, #64]
	cbz x22, L27
	sub x8, x22, #1
	ldr x9, [x1, #56]
	cmp x8, x9
	b.hs L27
	ldr x9, [x1, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
L27:
	sub x0, x29, #144
	mov x2, x23
	mov x3, x25
	bl lyng_js_vm::vm::exceptions::<impl lyng_js_vm::vm::Vm>::transfer_to_exception_handler
	ldur x8, [x29, #-144]
	ldurb w27, [x29, #-136]
	cmp x8, x26
	b.ne L28
	tbz w27, #0, L29
	ldr x10, [sp, #64]
	ldr w8, [x10, #1640]
	add w8, w8, #1
	str w8, [x10, #1640]
	cbz x22, L30
	sub x8, x22, #1
	ldr x9, [x10, #56]
	cmp x8, x9
	b.hs L30
	ldr x9, [x10, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x8]
	str q0, [x20]
	ldp q0, q1, [x8, #48]
	ldp q3, q2, [x8, #16]
	stp q0, q1, [x20, #48]
	stp q3, q2, [x20, #16]
L30:
	ldr x8, [x20, #128]
	ldr x8, [x8, #48]
	ldr w9, [x20, #56]
	ldrb w8, [x8, x9]
	b L31
L1:
L32:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
L33:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov x0, x6
	mov x2, x1
	bl core::slice::index::slice_index_fail
	brk #0x1
L2:
	sub x0, x29, #144
	mov w4, #1
	mov x5, x22
	bl lyng_js_vm::vm::dispatch::decode_abx_operands_wide
	ldur x8, [x29, #-144]
	cmp x8, x26
	b.ne L17
	ldurh w10, [x29, #-128]
	ldp w8, w24, [x29, #-136]
	ldur w9, [x29, #-124]
	stp w9, w10, [sp, #72]
	ldr x13, [x20, #80]
	ldr x9, [x13, #80]
	sub w28, w22, #1
	cmp x9, x28
	b.hi L5
L4:
	sub x9, x26, #29
	mov x10, x22
L18:
L19:
	stp x9, x10, [x19]
	str x11, [x19, #16]
	stp w22, w8, [x19, #24]
L16:
	ldp x29, x30, [sp, #320]
	ldp x20, x19, [sp, #304]
	ldp x22, x21, [sp, #288]
	ldp x24, x23, [sp, #272]
	ldp x26, x25, [sp, #256]
	ldp x28, x27, [sp, #240]
	add sp, sp, #336
	ret
L34:
	add x8, x8, #16
	add x21, x21, #1
	cmp x8, x9
	b.eq L25
L26:
	ldp w10, w11, [x8]
	cmp w10, #1
	ldr x10, [sp, #56]
	ccmp w11, w10, #0, eq
	b.ne L34
	ldrb w10, [x8, #9]
	tbz w10, #0, L34
	lsr x8, x21, #32
	cbz x8, L21
L25:
	mov x0, x23
	mov x1, x27
	bl lyng_js_env::agent::environments::<impl lyng_js_env::agent::Agent>::global_environment_object
	mov x21, x0
	cbz w0, L12
	ldr x1, [sp, #64]
	cbz w24, L35
	ldr x8, [x1, #104]
	cmp x8, x28
	b.ls L35
	ldr x8, [x1, #96]
	add x9, x8, x28, lsl #5
	ldr x10, [x9, #16]
	sub w8, w24, #1
	cmp x10, x8
	b.ls L35
	ldr x9, [x9, #8]
	mov w10, #1216
	umaddl x0, w8, w10, x9
	ldr x8, [x0]
	cmp x8, #10
	ccmp x8, #6, #0, ne
	b.eq L36
L35:
	ldr x8, [x23, #224]
	sub w9, w21, #1
	lsr x10, x9, #6
	cmp x10, x8
	b.hs L37
	ldr x11, [x23, #216]
	and x12, x9, #0x3f
	ldr x11, [x11, x10, lsl #3]
	mov w13, #80
	umaddl x13, w12, w13, x11
	mov x12, x13
	ldr w11, [x12], #8
	cmp w11, #1
	b.ne L37
	ldr w15, [x13, #52]
	cbz w15, L37
	cbz w24, L38
	ldr x11, [x1, #104]
	cmp x11, x28
	b.ls L39
	ldr x14, [x1, #96]
	add x16, x14, x28, lsl #5
	ldr x17, [x16, #16]
	sub w14, w24, #1
	cmp x17, x14
	b.ls L39
	ldr x16, [x16, #8]
	mov w17, #1216
	umaddl x0, w14, w17, x16
	ldr x14, [x0]
	cmp x14, #10
	ccmp x14, #6, #0, ne
	b.ne L39
	ldr x14, [x0, #1016]
	lsr x16, x14, #32
	cmp x16, #0
	ccmp w15, w16, #0, ne
	b.eq L40
	ldr x14, [x0, #1024]
	lsr x16, x14, #32
	cmp x16, #0
	ccmp w15, w16, #0, ne
	b.ne L41
	mov w15, #1
	b L42
L37:
	cbz w24, L38
	ldr x11, [x1, #104]
L39:
	cmp x11, x28
	b.ls L38
L43:
	ldr x11, [x1, #96]
	add x12, x11, x28, lsl #5
	ldr x13, [x12, #16]
	sub w11, w24, #1
	cmp x13, x11
	b.ls L38
	ldr x12, [x12, #8]
	mov w13, #1216
	umaddl x0, w11, w13, x12
	ldr x11, [x0]
	cmp x11, #10
	ccmp x11, #6, #0, ne
	b.eq L44
L38:
	mov x0, x1
	mov x1, x23
	mov x2, x22
	mov x3, x24
	mov x4, x21
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::try_named_property_load_inline_cache_hit
	tbz w0, #0, L45
	mov x25, x1
L23:
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
L31:
L46:
	adrp x9, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
L47:
	add x9, x9, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x8, [x9, x8, lsl #3]
	stp x26, x8, [x19]
	b L16
L28:
	ldurb w9, [x29, #-133]
	ldurh w10, [x29, #-135]
	orr w10, w10, w9, lsl #16
	ldur w9, [x29, #-132]
	ldur x25, [x29, #-128]
	ldur q0, [x29, #-120]
	str q0, [sp, #112]
	ldur x11, [x29, #-104]
	b L14
L45:
	ldp x9, x1, [sp, #56]
	ldr w8, [sp, #20]
	stp w8, w9, [sp]
	sub x0, x29, #144
	mov x2, x23
	ldp x4, x3, [sp, #40]
	ldp x6, x5, [sp, #24]
	mov x7, x20
	bl lyng_js_vm::vm::names::<impl lyng_js_vm::vm::Vm>::get_global_property_binding_with_context
	ldp x8, x27, [x29, #-144]
	ldur x25, [x29, #-128]
	cmp x8, x26
	b.ne L48
	tbz w27, #0, L49
	ldp x5, x0, [sp, #56]
	mov x1, x23
	mov x2, x22
	mov x3, x24
	mov x4, x21
	mov w6, #0
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::observe_named_property_slow_path
	b L23
L29:
	mov w27, #0
	mov w10, #0
	mov x8, #-9223372036854775808
	b L15
L48:
	ldur q0, [x29, #-120]
	str q0, [sp, #80]
	ldur x9, [x29, #-104]
	str x9, [sp, #96]
	lsr x9, x27, #32
	b L24
L49:
	mov x0, x23
	mov w1, #3
	bl lyng_js_ops::errors::error_value
	mov x25, x0
	mov w9, #0
	mov w27, #0
	mov x8, #-9223372036854775808
	b L24
L36:
	ldr x8, [x0, #968]
	cbz x8, L35
	ldr x11, [x23, #224]
	sub w9, w21, #1
	lsr x10, x9, #6
	cmp x10, x11
	b.hs L35
	ldr x11, [x23, #216]
	and x9, x9, #0x3f
	ldr x10, [x11, x10, lsl #3]
	mov w11, #80
	umaddl x13, w9, w11, x10
	ldr w9, [x13]
	cmp w9, #1
	b.ne L35
	ldr x9, [x0, #976]
	ldp w12, w10, [x13, #52]
	ldr x11, [x13, #40]
	ldur q0, [x13, #8]
	ldur q1, [x13, #24]
	stp q0, q1, [sp, #144]
	lsr x13, x8, #32
	cmp x13, #0
	csel w13, w13, wzr, ne
	cbz w12, L50
	cmp w12, w13
	b L51
L44:
	ldr x12, [x0, #984]
	cbz x12, L38
	cmp x10, x8
	b.hs L38
	ldr x11, [x23, #216]
	and x9, x9, #0x3f
	ldr x10, [x11, x10, lsl #3]
	mov w13, #80
	umaddl x15, w9, w13, x10
	ldr w9, [x15]
	cmp w9, #1
	b.ne L38
	ldr x9, [x0, #992]
	ldr x14, [x0, #1000]
	ldr x10, [x0, #1008]
	ldp w13, w16, [x15, #48]
	ldr x15, [x15, #40]
	cmp w13, #0
	cbz w16, L52
	ccmp w16, w12, #0, ne
	b L53
L40:
	mov x15, #0
L42:
	add x15, x0, x15, lsl #3
	ldr x16, [x15, #1032]
	ldr w15, [x13, #56]
	ldr x13, [x13, #40]
	ldp q0, q1, [x12]
	stp q0, q1, [x29, #-144]
	cmp x13, x16
	b.ne L41
	tbnz w14, #31, L54
	cbz w15, L41
	ldr x13, [x23, #640]
	sub w12, w15, #1
	cmp x13, x12
	b.ls L41
	ldr x13, [x23, #632]
	mov w15, #24
	umaddl x12, w12, w15, x13
	ldrb w13, [x12, #19]
	cmp w13, #1
	b.ne L41
	ldr x15, [x12, #8]
	and x13, x14, #0x3fffffff
	cmp x15, x13
	b.ls L41
	ldr x8, [x12]
	add x8, x8, x13, lsl #3
	ldr x25, [x8]
	b L55
L50:
	cmp w13, #0
L51:
	ccmp x11, x9, #0, eq
	ldr x1, [sp, #64]
	b.ne L35
	and x9, x8, #0x3fffffff
	tbnz w8, #31, L56
	ldr x1, [sp, #64]
	cbz w10, L35
	ldr x11, [x23, #640]
	sub w8, w10, #1
	cmp x11, x8
	b.ls L35
	ldr x10, [x23, #632]
	mov w11, #24
	umaddl x8, w8, w11, x10
	ldrb w10, [x8, #19]
	cmp w10, #1
	b.ne L35
	ldr x10, [x8, #8]
	cmp x9, x10
	b.hs L35
	ldr x8, [x8]
	b L57
L52:
	ccmp w12, #0, #0, ne
L53:
	ccmp x15, x14, #0, eq
	ldr x1, [sp, #64]
	b.ne L38
	sub w12, w13, #1
	lsr x13, x12, #6
	cmp x13, x8
	b.hs L38
	and x8, x12, #0x3f
	ldr x11, [x11, x13, lsl #3]
	mov w12, #80
	umaddl x13, w8, w12, x11
	ldr w8, [x13]
	cmp w8, #1
	b.ne L38
	ldp w12, w11, [x13, #52]
	ldr x8, [x13, #40]
	ldur q0, [x13, #8]
	ldur q1, [x13, #24]
	stp q0, q1, [x29, #-144]
	lsr x13, x9, #32
	cmp x13, #0
	csel w13, w13, wzr, ne
	cbz w12, L58
	cmp w12, w13
	ccmp x8, x10, #0, eq
	ldr x1, [sp, #64]
	b.ne L38
	b L59
L56:
	cmp x9, #4
	ldr x1, [sp, #64]
	b.hs L35
	add x8, sp, #144
L57:
	ldr x25, [x8, x9, lsl #3]
	b L55
L54:
	tst w14, #0x3ffffffc
	b.eq L60
L41:
	ldr x1, [sp, #64]
	cmp x11, x28
	b.hi L43
	b L38
L60:
	and x8, x14, #0x3fffffff
	sub x9, x29, #144
	add x8, x9, x8, lsl #3
	ldr x25, [x8]
	b L55
L58:
	ldr x1, [sp, #64]
	cbnz w13, L38
	cmp x8, x10
	b.ne L38
L59:
	and x8, x9, #0x3fffffff
	tbnz w9, #31, L61
	ldr x1, [sp, #64]
	cbz w11, L38
	ldr x10, [x23, #640]
	sub w9, w11, #1
	cmp x10, x9
	b.ls L38
	ldr x10, [x23, #632]
	mov w11, #24
	umaddl x9, w9, w11, x10
	ldrb w10, [x9, #19]
	cmp w10, #1
	b.ne L38
	ldr x10, [x9, #8]
	cmp x8, x10
	b.hs L38
	ldr x9, [x9]
	b L62
L61:
	cmp x8, #3
	ldr x1, [sp, #64]
	b.hi L38
	sub x9, x29, #144
L62:
	ldr x25, [x9, x8, lsl #3]
L55:
	bl lyng_js_vm::vm::feedback::FeedbackSiteState::record_execution
	ldr x8, [sp, #64]
	ldp x0, x1, [x8, #120]
	mov x2, x22
	bl lyng_js_vm::vm::tiering::<impl lyng_js_vm::vm::Vm>::observe_tier_feedback_event
	b L23
	bl core::panicking::panic_cannot_unwind
	bl core::panicking::panic_cannot_unwind
	bl core::panicking::panic_cannot_unwind
	.loh AdrpAdd	L32, L33
	.loh AdrpAdd	L46, L47
