lyng_vm::vm::dispatch_handlers::arithmetic::op_less_equal:
L0:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	mov x1, x0
	mov x0, x8
L1:
	adrp x2, lyng_vm::vm::dispatch::arithmetic::<impl lyng_vm::vm::Vm>::execute_less_equal_opcode@PAGE
L2:
	add x2, x2, lyng_vm::vm::dispatch::arithmetic::<impl lyng_vm::vm::Vm>::execute_less_equal_opcode@PAGEOFF
	bl lyng_vm::vm::dispatch_handlers::arithmetic::op_binary_general
	ldp x29, x30, [sp], #16
	ret
	bl core::panicking::panic_cannot_unwind
	.loh AdrpAdd	L1, L2
