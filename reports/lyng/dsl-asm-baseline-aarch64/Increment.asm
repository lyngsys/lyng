lyng_js_vm::vm::dispatch_handlers::arithmetic::op_increment:
L0:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	mov x1, x0
	mov x0, x8
	mov w2, #1
	bl lyng_js_vm::vm::dispatch_handlers::arithmetic::op_update_register
	ldp x29, x30, [sp], #16
	ret
	bl core::panicking::panic_cannot_unwind
