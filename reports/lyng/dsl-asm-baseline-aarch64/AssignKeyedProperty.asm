lyng_vm::vm::dispatch_handlers::property::op_assign_keyed_property:
L0:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	mov x1, x0
	mov x0, x8
	mov w2, #83
	bl lyng_vm::vm::dispatch_handlers::property::op_set_keyed_property_common
	ldp x29, x30, [sp], #16
	ret
	bl core::panicking::panic_cannot_unwind
