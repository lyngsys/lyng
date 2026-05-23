lyng_vm::vm::dispatch_handlers::names::op_load_global:
Lfunc_begin358:
		// crates/vm/src/vm/dispatch_handlers/names.rs:26
		pub extern "C" fn op_load_global(state: &mut DispatchState) -> Step {
	sub sp, sp, #336
	stp x28, x27, [sp, #240]
	stp x26, x25, [sp, #256]
	stp x24, x23, [sp, #272]
	stp x22, x21, [sp, #288]
	stp x20, x19, [sp, #304]
	stp x29, x30, [sp, #320]
	add x29, sp, #320
	mov x19, x8
		// crates/vm/src/frame.rs:420
		self.metadata.code
	ldr w22, [x0, #4]
		// crates/vm/src/frame.rs:425
		self.state.instruction_offset
	ldr w6, [x0, #56]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/mem/mod.rs:893
		let result = crate::intrinsics::read_via_copy(dest);
	ldrb w3, [x0, #148]
	mov w8, #152
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/mem/mod.rs:894
		crate::intrinsics::write_via_move(dest, src);
	strb w8, [x0, #148]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:445
		unsafe { &*self.as_ptr().cast_const() }
	ldr x8, [x0, #128]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1599
		unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
	ldr x1, [x8, #56]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:568
		if self.start > slice.len() {
	subs x2, x1, x6
	b.lo LBB358_57
	mov x20, x0
	mov x26, #33
	movk x26, #32768, lsl #48
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x8, [x8, #48]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:89
		let ptr = unsafe { crate::intrinsics::offset(ptr, offset) };
	add x1, x8, x6
		// crates/vm/src/vm/dispatch.rs:295
		if let Some(prefix) = prefix {
	cmp w3, #152
	b.ne LBB358_59
	and x8, x2, #0x7ffffffffffffffe
		// crates/vm/src/vm/dispatch.rs:298
		let [_, ra, bx_low, bx_high, ..] = bytes else {
	cmp x2, #4
	ccmp x8, #4, #4, hs
	b.eq LBB358_23
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:3923
		unsafe { mem::transmute(bytes) }
	ldrh w24, [x1, #4]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:1338
		match self {
	cbz w24, LBB358_23
		// crates/vm/src/vm/dispatch.rs:307
		u16::from(*ra),
	ldrb w10, [x1, #1]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:3923
		unsafe { mem::transmute(bytes) }
	ldrh w8, [x1, #2]
	mov w9, #6
	stp w9, w10, [sp, #72]
		// crates/vm/src/vm/dispatch_handlers/names.rs:37
		let atom = try_step!(state.vm.read_atom_constant(code, bx));
	ldr x13, [x20, #80]
	ldr x9, [x13, #80]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:499
		unsafe { intrinsics::transmute_unchecked(self) }
	sub w28, w22, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x9, x28
	b.ls LBB358_61
LBB358_5:
		// crates/vm/src/vm/dispatch_handlers/names.rs:37
		let atom = try_step!(state.vm.read_atom_constant(code, bx));
	ldr x9, [x13, #72]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:745
		match *self {
	ldr x10, [x9, x28, lsl #3]
	cbz x10, LBB358_61
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1599
		unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
	ldr x12, [x10, #80]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/convert/num.rs:264
		Ok(value as Self)
	mov w11, w8
	sub x9, x26, #3
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x12, x11
	b.ls LBB358_25
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x12, [x10, #72]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	add x11, x12, x11, lsl #4
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2059
		Some(&v) => Some(v),
	ldr w12, [x11]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:1338
		match self {
	cmp w12, #4
	b.eq LBB358_25
	ldr w14, [x11, #4]
	cmp w12, #2
	b.ne LBB358_26
	str x13, [sp, #64]
		// crates/vm/src/vm/values.rs:17
		.map_or(atom, |installed| installed.canonical_atom(atom))
	ldr x8, [x10, #464]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x8, x14
	b.ls LBB358_12
		// crates/vm/src/vm/values.rs:17
		.map_or(atom, |installed| installed.canonical_atom(atom))
	ldr x8, [x10, #456]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	add x8, x8, x14, lsl #3
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2059
		Some(&v) => Some(v),
	ldr w9, [x8, #16]!
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:1042
		match self {
	cmp w9, #1
	b.ne LBB358_12
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2059
		Some(&v) => Some(v),
	ldr w14, [x8, #4]
LBB358_12:
		// crates/vm/src/vm/dispatch_handlers/names.rs:48
		agent,
	ldp x23, x8, [x20, #88]
	stp x8, x14, [sp, #48]
		// crates/vm/src/vm/dispatch_handlers/names.rs:49
		*host,
	ldp x9, x8, [x20, #104]
	stp x8, x9, [sp, #32]
		// crates/vm/src/vm/dispatch_handlers/names.rs:50
		&mut **registry,
	ldr x8, [x20, #120]
	str x8, [sp, #24]
		// crates/vm/src/frame.rs:465
		self.metadata.variable_env
	ldr w1, [x20, #12]
	sub x25, x26, #27
	str w1, [sp, #20]
LBB358_13:
	mov x27, x1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1599
		unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
	ldr x8, [x23, #5168]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:499
		unsafe { intrinsics::transmute_unchecked(self) }
	sub w21, w1, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x8, x21
	b.ls LBB358_16
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x8, [x23, #5160]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	lsl x9, x21, #7
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:745
		match *self {
	ldr x8, [x8, x9]
	cmp x8, x25
	b.eq LBB358_16
		// crates/vm/src/vm/names.rs:1643
		if agent.environment_is_global(current) {
	tbz x8, #63, LBB358_27
LBB358_16:
		// crates/vm/src/vm/names.rs:1647
		.environment_outer(current)
	mov x0, x23
	mov x1, x27
	bl lyng_env::agent::environments::<impl lyng_env::agent::Agent>::environment_outer
	tbz w0, #0, LBB358_19
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:1338
		match self {
	cbnz w1, LBB358_13
LBB358_19:
	mov w9, #0
	sub x8, x26, #24
LBB358_20:
		// crates/vm/src/vm/dispatch.rs:556
		Err(error) => Err(error),
	lsr w10, w27, #8
	ldr q0, [sp, #80]
	str q0, [sp, #112]
	ldr x11, [sp, #96]
LBB358_21:
	str x11, [sp, #128]
LBB358_22:
		// crates/vm/src/vm/dispatch_state.rs:303
		Err(error) => return $crate::vm::dispatch_state::Step::Error(error),
	lsl w10, w10, #8
	bfxil x10, x27, #0, #8
	orr x9, x10, x9, lsl #32
	stp x8, x9, [x19]
	str x25, [x19, #16]
	ldr q0, [sp, #112]
	stur q0, [x19, #24]
	ldr x8, [sp, #128]
	str x8, [x19, #40]
	b LBB358_64
LBB358_23:
	sub x8, x26, #12
	stur x8, [x29, #-144]
	stp w22, w6, [x29, #-136]
LBB358_24:
	ldp q0, q1, [x29, #-144]
	stp q0, q1, [x19]
	ldur q0, [x29, #-112]
	str q0, [x19, #32]
	b LBB358_64
LBB358_25:
	mov x10, #0
	b LBB358_62
LBB358_26:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2059
		Some(&v) => Some(v),
	ldr x11, [x11, #8]
		// crates/vm/src/vm/values.rs:833
		_ => Err(VmError::InvalidAtomConstant {
	orr x10, x12, x14, lsl #32
	b LBB358_63
LBB358_27:
		// crates/vm/src/vm/names.rs:1670
		.global_lexical_binding(global, name)
	sub x8, x29, #144
	mov x0, x23
	mov x1, x27
	ldr x2, [sp, #56]
	bl lyng_env::agent::environments::<impl lyng_env::agent::Agent>::global_lexical_binding
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:1654
		match self {
	ldur w8, [x29, #-144]
	cbz w8, LBB358_37
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:1655
		x @ Some(_) => x,
	ldur w21, [x29, #-136]
	mov x27, x8
LBB358_30:
		// crates/vm/src/vm/names.rs:535
		self.environment_for_slot_access(agent, binding.environment(), 0, binding.slot())?;
	sub x0, x29, #144
	ldr x1, [sp, #64]
	mov x2, x23
	mov x3, x27
	mov w4, #0
	mov x5, x21
	bl lyng_vm::vm::loop_iteration::<impl lyng_vm::vm::Vm>::environment_for_slot_access
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2173
		match self {
	ldur x8, [x29, #-144]
	ldur w27, [x29, #-136]
	cmp x8, x26
	b.ne LBB358_44
		// crates/vm/src/vm/values.rs:599
		.environment_slot(environment, slot)
	mov x0, x23
	mov x1, x27
	mov x2, x21
	bl lyng_env::agent::environments::<impl lyng_env::agent::Agent>::environment_slot
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2173
		match self {
	tbz w0, #0, LBB358_19
	mov x25, x1
	mov x8, #2
	movk x8, #9, lsl #32
	movk x8, #32760, lsl #48
		// crates/vm/src/vm/values.rs:621
		if value == Value::uninitialized_lexical() {
	cmp x1, x8
	b.ne LBB358_78
		// crates/ops/src/errors.rs:141
		error_value(agent, ErrorKind::Reference)
	mov x0, x23
	mov w1, #3
	bl lyng_ops::errors::error_value
	mov x25, x0
	mov w27, #0
	mov x8, #-9223372036854775808
	b LBB358_45
LBB358_37:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1599
		unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
	ldr x8, [x23, #5168]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x8, x21
	b.ls LBB358_69
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x8, [x23, #5160]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	add x8, x8, x21, lsl #7
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:745
		match *self {
	ldr x9, [x8]
	cmp x9, x25
	b.eq LBB358_69
		// crates/env/src/agent/environments.rs:320
		match self.environment_metadata(id) {
	tbnz x9, #63, LBB358_69
		// crates/env/src/agent/environments.rs:321
		Some(EnvironmentMetadata::Global { layout, .. }) => Some(*layout),
	ldr w8, [x8, #120]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1599
		unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
	ldr x9, [x23, #5144]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:499
		unsafe { intrinsics::transmute_unchecked(self) }
	sub w8, w8, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x9, x8
	b.ls LBB358_69
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x9, [x23, #5136]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	add x8, x9, x8, lsl #5
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:745
		match *self {
	ldr x9, [x8]
	mov x10, #-9223372036854775808
	cmp x9, x10
	b.eq LBB358_69
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1599
		unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
	ldr x9, [x8, #16]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
		if ptr == crate::intrinsics::transmute::<$ptr, NonNull<T>>(end_or_len) {
	cbz x9, LBB358_69
	mov x21, #0
	ldr x8, [x8, #8]
	add x9, x8, x9, lsl #4
	b LBB358_66
LBB358_44:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2175
		Err(e) => ControlFlow::Break(Err(e)),
	ldur w9, [x29, #-132]
	ldur x25, [x29, #-128]
	ldur q0, [x29, #-120]
	str q0, [sp, #80]
	ldur x10, [x29, #-104]
	str x10, [sp, #96]
LBB358_45:
	mov x10, #-9223372036854775808
		// crates/vm/src/vm/dispatch.rs:544
		match result {
	cmp x8, x10
	b.ne LBB358_20
	cbnz w27, LBB358_20
		// crates/vm/src/vm/dispatch_state.rs:173
		vm.handle_dispatch_result(agent, *frame_depth, frame, result)
	ldr x22, [x20, #136]
	ldr x1, [sp, #64]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:878
		if self < rhs {
	cbz x22, LBB358_50
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:882
		Some(unsafe { intrinsics::unchecked_sub(self, rhs) })
	sub x8, x22, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1631
		unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
	ldr x9, [x1, #56]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:229
		if self < slice.len() {
	cmp x8, x9
	b.hs LBB358_50
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x9, [x1, #48]
	mov w10, #80
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:231
		unsafe { Some(slice_get_unchecked(slice, self)) }
	madd x8, x8, x10, x9
		// crates/vm/src/vm/dispatch.rs:509
		*slot = frame;
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB358_50:
		// crates/vm/src/vm/dispatch.rs:548
		if self.transfer_to_exception_handler(agent, value)? {
	sub x0, x29, #144
	mov x2, x23
	mov x3, x25
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2173
		match self {
	ldur x8, [x29, #-144]
	ldurb w27, [x29, #-136]
	cmp x8, x26
	b.ne LBB358_80
		// crates/vm/src/vm/dispatch.rs:548
		if self.transfer_to_exception_handler(agent, value)? {
	tbz w27, #0, LBB358_85
	ldr x10, [sp, #64]
		// crates/vm/src/vm/dispatch.rs:529
		self.dispatch_frame_check_epoch = self.dispatch_frame_check_epoch.wrapping_add(1);
	ldr w8, [x10, #1640]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2380
		intrinsics::wrapping_add(self, rhs)
	add w8, w8, #1
		// crates/vm/src/vm/dispatch.rs:529
		self.dispatch_frame_check_epoch = self.dispatch_frame_check_epoch.wrapping_add(1);
	str w8, [x10, #1640]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:878
		if self < rhs {
	cbz x22, LBB358_56
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:882
		Some(unsafe { intrinsics::unchecked_sub(self, rhs) })
	sub x8, x22, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1599
		unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
	ldr x9, [x10, #56]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x8, x9
	b.hs LBB358_56
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x9, [x10, #48]
	mov w10, #80
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	madd x8, x8, x10, x9
		// crates/vm/src/vm/dispatch.rs:523
		*frame = stacked;
	ldr q0, [x8]
	str q0, [x20]
	ldp q0, q1, [x8, #48]
	ldp q3, q2, [x8, #16]
	stp q0, q1, [x20, #48]
	stp q3, q2, [x20, #16]
LBB358_56:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:445
		unsafe { &*self.as_ptr().cast_const() }
	ldr x8, [x20, #128]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x8, [x8, #48]
		// crates/vm/src/frame.rs:425
		self.state.instruction_offset
	ldr w9, [x20, #56]
		// crates/vm/src/vm/dispatch_state.rs:100
		unsafe { *bytes.as_ptr().add(pc) }
	ldrb w8, [x8, x9]
	b LBB358_79
LBB358_57:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:569
		slice_index_fail(self.start, slice.len(), slice.len())
Lloh921:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
Lloh922:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov x0, x6
	mov x2, x1
	bl core::slice::index::slice_index_fail
	brk #0x1
LBB358_59:
		// crates/vm/src/vm/dispatch.rs:296
		return decode_abx_operands_wide(bytes, prefix, is_profiled, code, instruction_offset);
	sub x0, x29, #144
	mov w4, #1
	mov x5, x22
	bl lyng_vm::vm::dispatch::decode_abx_operands_wide
		// crates/vm/src/vm/dispatch_handlers/names.rs:30
		let (a, bx, feedback_slot, instruction_len) = try_step!(decode_abx_operands(
	ldur x8, [x29, #-144]
		// crates/vm/src/vm/dispatch_state.rs:301
		match $e {
	cmp x8, x26
	b.ne LBB358_24
		// crates/vm/src/vm/dispatch_state.rs:302
		Ok(v) => v,
	ldurh w10, [x29, #-128]
	ldp w8, w24, [x29, #-136]
	ldur w9, [x29, #-124]
	stp w9, w10, [sp, #72]
		// crates/vm/src/vm/dispatch_handlers/names.rs:37
		let atom = try_step!(state.vm.read_atom_constant(code, bx));
	ldr x13, [x20, #80]
	ldr x9, [x13, #80]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:499
		unsafe { intrinsics::transmute_unchecked(self) }
	sub w28, w22, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x9, x28
	b.hi LBB358_5
LBB358_61:
	sub x9, x26, #29
	mov x10, x22
LBB358_62:
LBB358_63:
		// crates/vm/src/vm/dispatch_state.rs:303
		Err(error) => return $crate::vm::dispatch_state::Step::Error(error),
	stp x9, x10, [x19]
	str x11, [x19, #16]
	stp w22, w8, [x19, #24]
LBB358_64:
		// crates/vm/src/vm/dispatch_handlers/names.rs:65
		}
	ldp x29, x30, [sp, #320]
	ldp x20, x19, [sp, #304]
	ldp x22, x21, [sp, #288]
	ldp x24, x23, [sp, #272]
	ldp x26, x25, [sp, #256]
	ldp x28, x27, [sp, #240]
	add sp, sp, #336
	ret
LBB358_65:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:660
		unsafe { NonNull { pointer: intrinsics::offset(self.as_ptr(), count) } }
	add x8, x8, #16
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:390
		i += 1;
	add x21, x21, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
		if ptr == crate::intrinsics::transmute::<$ptr, NonNull<T>>(end_or_len) {
	cmp x8, x9
	b.eq LBB358_69
LBB358_66:
		// crates/vm/src/vm/names.rs:1683
		.position(|binding| binding.name() == Some(name) && binding.flags().is_lexical())?;
	ldp w10, w11, [x8]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:384
		if predicate(x) {
	cmp w10, #1
	ldr x10, [sp, #56]
	ccmp w11, w10, #0, eq
	b.ne LBB358_65
	ldrb w10, [x8, #9]
	tbz w10, #0, LBB358_65
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/convert/num.rs:306
		if u > (Self::MAX as $source) {
	lsr x8, x21, #32
	cbz x8, LBB358_30
LBB358_69:
		// crates/vm/src/vm/names.rs:540
		.global_environment_object(global)
	mov x0, x23
	mov x1, x27
	bl lyng_env::agent::environments::<impl lyng_env::agent::Agent>::global_environment_object
	mov x21, x0
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:1338
		match self {
	cbz w0, LBB358_19
	ldr x14, [sp, #64]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2666
		match self {
	cbz w24, LBB358_75
		// crates/vm/src/vm/feedback.rs:2085
		let site = self.feedback_site_for_slot(code, slot?)?;
	ldr x8, [x14, #104]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x8, x28
	b.ls LBB358_75
		// crates/vm/src/vm/feedback.rs:2085
		let site = self.feedback_site_for_slot(code, slot?)?;
	ldr x8, [x14, #96]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	add x9, x8, x28, lsl #5
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1599
		unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
	ldr x10, [x9, #16]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2268
		intrinsics::saturating_sub(self, rhs)
	sub w8, w24, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x10, x8
	b.ls LBB358_75
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x9, [x9, #8]
	mov w10, #1160
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x0, w8, w10, x9
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:745
		match *self {
	ldr x8, [x0]
	cmp x8, #10
		// crates/vm/src/vm/feedback.rs:2086
		match site {
	ccmp x8, #6, #0, ne
	b.eq LBB358_89
LBB358_75:
		// crates/vm/src/vm/names.rs:580
		self.try_named_property_load_inline_cache_hit(agent, code, feedback_slot, global_object)
	mov x0, x14
	mov x1, x23
	mov x2, x22
	mov x3, x24
	mov x4, x21
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_named_property_load_inline_cache_hit
		// crates/vm/src/vm/names.rs:579
		if let Some(value) =
	tbz w0, #0, LBB358_81
	mov x25, x1
LBB358_78:
		// crates/vm/src/frame.rs:440
		self.metadata.registers
	ldr w8, [x20, #20]
		// crates/vm/src/vm/dispatch_handlers/names.rs:62
		state.vm.write_register_unchecked(registers, a, value);
	ldr x9, [x20, #80]
	ldr w10, [sp, #76]
		// crates/vm/src/vm/registers.rs:160
		let absolute = registers.base() + u32::from(register);
	add w8, w8, w10
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x9, [x9, #24]
		// crates/vm/src/vm/registers.rs:84
		*self.register_stack.get_unchecked_mut(absolute) = value;
	str x25, [x9, w8, uxtw #3]
		// crates/vm/src/frame.rs:425
		self.state.instruction_offset
	ldr w8, [x20, #56]
	ldr w9, [sp, #72]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2380
		intrinsics::wrapping_add(self, rhs)
	add w8, w8, w9
		// crates/vm/src/frame.rs:435
		self.state.instruction_offset = instruction_offset;
	str w8, [x20, #56]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:445
		unsafe { &*self.as_ptr().cast_const() }
	ldr x9, [x20, #128]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x9, [x9, #48]
		// crates/vm/src/vm/dispatch_state.rs:100
		unsafe { *bytes.as_ptr().add(pc) }
	ldrb w8, [x9, w8, uxtw]
LBB358_79:
		// crates/vm/src/vm/dispatch_state.rs:244
		$crate::vm::dispatch_state::DISPATCH_TABLE[byte as usize],
Lloh923:
	adrp x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
Lloh924:
	add x9, x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x8, [x9, x8, lsl #3]
		// crates/vm/src/vm/dispatch_state.rs:243
		return $crate::vm::dispatch_state::Step::Continue(
	stp x26, x8, [x19]
	b LBB358_64
LBB358_80:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2175
		Err(e) => ControlFlow::Break(Err(e)),
	ldurb w9, [x29, #-133]
	ldurh w10, [x29, #-135]
	orr w10, w10, w9, lsl #16
	ldur w9, [x29, #-132]
	ldur x25, [x29, #-128]
	ldur q0, [x29, #-120]
	str q0, [sp, #112]
	ldur x11, [x29, #-104]
	b LBB358_21
LBB358_81:
	ldp x9, x1, [sp, #56]
	ldr w8, [sp, #20]
		// crates/vm/src/vm/names.rs:586
		.get_global_property_binding_with_context(
	stp w8, w9, [sp]
	sub x0, x29, #144
	mov x2, x23
	ldp x4, x3, [sp, #40]
	ldp x6, x5, [sp, #24]
	mov x7, x20
	bl lyng_vm::vm::names::<impl lyng_vm::vm::Vm>::get_global_property_binding_with_context
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2173
		match self {
	ldp x8, x27, [x29, #-144]
	ldur x25, [x29, #-128]
	cmp x8, x26
	b.ne LBB358_86
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:1367
		match self {
	tbz w27, #0, LBB358_87
	ldp x5, x0, [sp, #56]
		// crates/vm/src/vm/names.rs:595
		self.observe_named_property_slow_path(
	mov x1, x23
	mov x2, x22
	mov x3, x24
	mov x4, x21
	mov w6, #0
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_named_property_slow_path
	b LBB358_78
LBB358_85:
	mov w27, #0
	mov w10, #0
	mov x8, #-9223372036854775808
		// crates/vm/src/vm/dispatch.rs:548
		if self.transfer_to_exception_handler(agent, value)? {
	b LBB358_22
LBB358_86:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2175
		Err(e) => ControlFlow::Break(Err(e)),
	ldur q0, [x29, #-120]
	str q0, [sp, #80]
	ldur x9, [x29, #-104]
	str x9, [sp, #96]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2189
		Err(e) => Err(From::from(e)),
	lsr x9, x27, #32
	b LBB358_45
LBB358_87:
		// crates/ops/src/errors.rs:141
		error_value(agent, ErrorKind::Reference)
	mov x0, x23
	mov w1, #3
	bl lyng_ops::errors::error_value
	mov x25, x0
	mov w9, #0
	mov w27, #0
	mov x8, #-9223372036854775808
	b LBB358_45
LBB358_89:
		// crates/vm/src/vm/feedback.rs:2087
		FeedbackSiteState::NamedProperty(feedback) if feedback.monomorphic_fast.is_valid() => {
	ldr x8, [x0, #968]
	cbz x8, LBB358_96
		// crates/gc/src/arena.rs:606
		self.objects.get_ref(id)
	ldr x11, [x23, #224]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:882
		Some(unsafe { intrinsics::unchecked_sub(self, rhs) })
	sub w9, w21, #1
		// crates/gc/src/arena/storage.rs:1020
		raw / PRIMITIVE_SLOTS_PER_PAGE,
	lsr x10, x9, #6
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x10, x11
	b.hs LBB358_96
		// crates/gc/src/arena.rs:606
		self.objects.get_ref(id)
	ldr x11, [x23, #216]
		// crates/gc/src/arena/storage.rs:1021
		raw % PRIMITIVE_SLOTS_PER_PAGE,
	and x9, x9, #0x3f
		// crates/gc/src/arena/storage.rs:239
		self.pages.get(page_index)?.get_ref(slot_index)
	ldr x10, [x11, x10, lsl #3]
	mov w11, #80
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x13, w9, w11, x10
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:745
		match *self {
	ldr w9, [x13]
	cmp w9, #1
	b.ne LBB358_96
	ldr x9, [x0, #976]
		// crates/vm/src/vm/names.rs:550
		if record.shape() == handler.receiver_shape()
	ldp w12, w10, [x13, #52]
	ldr x11, [x13, #40]
	ldur q0, [x13, #8]
	ldur q1, [x13, #24]
	stp q0, q1, [sp, #144]
		// crates/types/src/ids.rs:21
		match NonZeroU32::new(raw) {
	lsr x13, x8, #32
	cmp x13, #0
	csel w13, w13, wzr, ne
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2365
		match (self, other) {
	cbz w12, LBB358_94
	cmp w12, w13
	b LBB358_95
LBB358_94:
		// crates/vm/src/vm/names.rs:550
		if record.shape() == handler.receiver_shape()
	cmp w13, #0
LBB358_95:
	ccmp x11, x9, #0, eq
	b.eq LBB358_101
LBB358_96:
		// crates/vm/src/vm/feedback.rs:2113
		if feedback.monomorphic_proto_fast.is_valid() =>
	ldr x12, [x0, #984]
	ldr x14, [sp, #64]
	cbz x12, LBB358_75
		// crates/gc/src/arena.rs:606
		self.objects.get_ref(id)
	ldr x10, [x23, #224]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:882
		Some(unsafe { intrinsics::unchecked_sub(self, rhs) })
	sub w8, w21, #1
		// crates/gc/src/arena/storage.rs:1020
		raw / PRIMITIVE_SLOTS_PER_PAGE,
	lsr x9, x8, #6
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x9, x10
	b.hs LBB358_75
		// crates/gc/src/arena.rs:606
		self.objects.get_ref(id)
	ldr x11, [x23, #216]
		// crates/gc/src/arena/storage.rs:1021
		raw % PRIMITIVE_SLOTS_PER_PAGE,
	and x8, x8, #0x3f
		// crates/gc/src/arena/storage.rs:239
		self.pages.get(page_index)?.get_ref(slot_index)
	ldr x9, [x11, x9, lsl #3]
	mov w13, #80
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x15, w8, w13, x9
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:745
		match *self {
	ldr w8, [x15]
	cmp w8, #1
	b.ne LBB358_75
	ldr x8, [x0, #992]
	ldr x14, [x0, #1000]
	ldr x9, [x0, #1008]
		// crates/vm/src/vm/dispatch/property.rs:1508
		if record.shape() != handler.receiver_shape()
	ldp w13, w16, [x15, #48]
	ldr x15, [x15, #40]
	cmp w13, #0
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2365
		match (self, other) {
	cbz w16, LBB358_107
	ccmp w16, w12, #0, ne
	b LBB358_108
LBB358_101:
		// crates/objects/src/shapes.rs:263
		let offset = low & HANDLER_SLOT_OFFSET_MASK;
	and x9, x8, #0x3fffffff
		// crates/vm/src/vm/names.rs:553
		let fast_value = match handler.slot_location() {
	tbnz w8, #31, LBB358_113
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:1545
		match self {
	cbz w10, LBB358_96
		// crates/gc/src/arena.rs:812
		self.object_slots.get(id)
	ldr x11, [x23, #640]
		// crates/gc/src/arena/storage.rs:860
		let slot = self.slots.get((id.get() - 1) as usize)?;
	sub w8, w10, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x11, x8
	b.ls LBB358_96
		// crates/gc/src/arena.rs:812
		self.object_slots.get(id)
	ldr x10, [x23, #632]
	mov w11, #24
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x8, w8, w11, x10
		// crates/gc/src/arena/storage.rs:861
		if slot.occupied {
	ldrb w10, [x8, #19]
	cmp w10, #1
	b.ne LBB358_96
		// crates/gc/src/arena/storage.rs:862
		Some(&slot.values)
	ldr x10, [x8, #8]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x9, x10
	b.hs LBB358_96
LBB358_106:
	ldr x8, [x8]
	b LBB358_115
LBB358_107:
		// crates/vm/src/vm/dispatch/property.rs:1508
		if record.shape() != handler.receiver_shape()
	ccmp w12, #0, #0, ne
LBB358_108:
	ccmp x15, x14, #0, eq
	ldr x14, [sp, #64]
	b.ne LBB358_75
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:882
		Some(unsafe { intrinsics::unchecked_sub(self, rhs) })
	sub w12, w13, #1
		// crates/gc/src/arena/storage.rs:1020
		raw / PRIMITIVE_SLOTS_PER_PAGE,
	lsr x13, x12, #6
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x13, x10
	b.hs LBB358_75
		// crates/gc/src/arena/storage.rs:1021
		raw % PRIMITIVE_SLOTS_PER_PAGE,
	and x10, x12, #0x3f
		// crates/gc/src/arena/storage.rs:239
		self.pages.get(page_index)?.get_ref(slot_index)
	ldr x11, [x11, x13, lsl #3]
	mov w12, #80
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x13, w10, w12, x11
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:745
		match *self {
	ldr w10, [x13]
	cmp w10, #1
	b.ne LBB358_75
		// crates/vm/src/vm/dispatch/property.rs:1515
		if prototype_record.shape() != handler.prototype_shape()
	ldp w12, w10, [x13, #52]
	ldr x11, [x13, #40]
	ldur q0, [x13, #8]
	ldur q1, [x13, #24]
	stp q0, q1, [x29, #-144]
		// crates/types/src/ids.rs:21
		match NonZeroU32::new(raw) {
	lsr x13, x8, #32
	cmp x13, #0
	csel w13, w13, wzr, ne
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2365
		match (self, other) {
	cbz w12, LBB358_116
	cmp w12, w13
	ccmp x11, x9, #0, eq
	ldr x14, [sp, #64]
	b.ne LBB358_75
	b LBB358_118
LBB358_113:
		// crates/gc/src/arena/records.rs:396
		if index < RUNTIME_OBJECT_INLINE_SLOT_COUNT {
	cmp x9, #4
	b.hs LBB358_96
	add x8, sp, #144
LBB358_115:
	ldr x25, [x8, x9, lsl #3]
	bl lyng_vm::vm::feedback::FeedbackSiteState::record_execution
	ldr x8, [sp, #64]
	ldp x0, x1, [x8, #120]
	mov x2, x22
	bl lyng_vm::vm::tiering::<impl lyng_vm::vm::Vm>::observe_tier_feedback_event
	b LBB358_78
LBB358_116:
	ldr x14, [sp, #64]
		// crates/vm/src/vm/dispatch/property.rs:1515
		if prototype_record.shape() != handler.prototype_shape()
	cbnz w13, LBB358_75
	cmp x11, x9
	b.ne LBB358_75
LBB358_118:
		// crates/objects/src/shapes.rs:401
		let offset = low & HANDLER_SLOT_OFFSET_MASK;
	and x9, x8, #0x3fffffff
		// crates/vm/src/vm/dispatch/property.rs:1520
		let value = match handler.slot_location() {
	tbnz w8, #31, LBB358_123
	ldr x14, [sp, #64]
		// crates/vm/src/vm/dispatch/property.rs:1523
		.object_slots(prototype_record.named_slots()?)?
	cbz w10, LBB358_75
		// crates/gc/src/arena.rs:812
		self.object_slots.get(id)
	ldr x11, [x23, #640]
		// crates/gc/src/arena/storage.rs:860
		let slot = self.slots.get((id.get() - 1) as usize)?;
	sub w8, w10, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x11, x8
	b.ls LBB358_75
		// crates/gc/src/arena.rs:812
		self.object_slots.get(id)
	ldr x10, [x23, #632]
	mov w11, #24
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x8, w8, w11, x10
		// crates/gc/src/arena/storage.rs:861
		if slot.occupied {
	ldrb w10, [x8, #19]
	cmp w10, #1
	b.ne LBB358_75
		// crates/gc/src/arena/storage.rs:862
		Some(&slot.values)
	ldr x10, [x8, #8]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2058
		match self {
	cmp x9, x10
	b.hs LBB358_75
	b LBB358_106
LBB358_123:
		// crates/gc/src/arena/records.rs:396
		if index < RUNTIME_OBJECT_INLINE_SLOT_COUNT {
	cmp x9, #3
	ldr x14, [sp, #64]
	b.hi LBB358_75
	sub x8, x29, #144
	b LBB358_115
		// crates/vm/src/vm/dispatch_handlers/names.rs:26
		pub extern "C" fn op_load_global(state: &mut DispatchState) -> Step {
	bl core::panicking::panic_cannot_unwind
	bl core::panicking::panic_cannot_unwind
	bl core::panicking::panic_cannot_unwind
