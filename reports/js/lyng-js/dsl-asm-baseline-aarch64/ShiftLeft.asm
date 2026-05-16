lyng_js_vm::vm::dispatch_handlers::arithmetic::op_shift_left:
L0:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	mov x1, x0
	mov x0, x8
L1:
	adrp x2, lyng_js_vm::vm::dispatch::arithmetic::<impl lyng_js_vm::vm::Vm>::execute_shift_left_opcode@PAGE
L2:
	add x2, x2, lyng_js_vm::vm::dispatch::arithmetic::<impl lyng_js_vm::vm::Vm>::execute_shift_left_opcode@PAGEOFF
	bl lyng_js_vm::vm::dispatch_handlers::arithmetic::op_binary_general
	ldp x29, x30, [sp], #16
	ret
	bl core::panicking::panic_cannot_unwind
	.loh AdrpAdd	L1, L2
