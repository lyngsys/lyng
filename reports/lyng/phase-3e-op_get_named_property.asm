lyng_vm::vm::dispatch_handlers::property::op_get_named_property:
Lfunc_begin390:
		// crates/lyng/vm/src/vm/dispatch_handlers/property.rs:31
		pub extern "C" fn op_get_named_property(state: &mut DispatchState) -> Step {
	sub sp, sp, #384
	stp x28, x27, [sp, #288]
	stp x26, x25, [sp, #304]
	stp x24, x23, [sp, #320]
	stp x22, x21, [sp, #336]
	stp x20, x19, [sp, #352]
	stp x29, x30, [sp, #368]
	add x29, sp, #368
	mov x19, x8
		// crates/lyng/vm/src/frame.rs:420
		self.metadata.code
	ldr w21, [x0, #4]
		// crates/lyng/vm/src/frame.rs:425
		self.state.instruction_offset
	ldr w23, [x0, #56]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/mem/mod.rs:893
		let result = crate::intrinsics::read_via_copy(dest);
	ldrb w8, [x0, #148]
	mov w9, #152
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/mem/mod.rs:894
		crate::intrinsics::write_via_move(dest, src);
	strb w9, [x0, #148]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:445
		unsafe { &*self.as_ptr().cast_const() }
	ldr x9, [x0, #128]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1599
		unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
	ldr x1, [x9, #56]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:568
		if self.start > slice.len() {
	subs x2, x1, x23
	b.lo LBB390_66
	mov x20, x0
	mov x28, #33
	movk x28, #32768, lsl #48
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x9, [x9, #48]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:89
		let ptr = unsafe { crate::intrinsics::offset(ptr, offset) };
	add x1, x9, x23
		// crates/lyng/vm/src/vm/dispatch.rs:236
		if prefix.is_some() {
	cmp w8, #152
	b.ne LBB390_67
	and x8, x2, #0x7ffffffffffffffe
		// crates/lyng/vm/src/vm/dispatch.rs:239
		let [_, ra, rb, rc, ..] = bytes else {
	cmp x2, #4
	ccmp x8, #4, #4, hs
	b.eq LBB390_19
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:3923
		unsafe { mem::transmute(bytes) }
	ldrh w25, [x1, #4]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:1338
		match self {
	cbz w25, LBB390_19
		// crates/lyng/vm/src/vm/dispatch.rs:248
		u16::from(*ra),
	ldrb w12, [x1, #1]
		// crates/lyng/vm/src/vm/dispatch.rs:249
		u16::from(*rb),
	ldrb w8, [x1, #2]
	mov w24, #6
		// crates/lyng/vm/src/vm/dispatch.rs:250
		u16::from(*rc),
	ldrb w9, [x1, #3]
		// crates/lyng/vm/src/vm/dispatch_handlers/property.rs:52
		vm.execute_get_named_property_opcode(
	ldr x22, [x20, #80]
		// crates/lyng/vm/src/frame.rs:440
		self.metadata.registers
	ldr w13, [x20, #20]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1599
		unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
	ldr x10, [x22, #32]
		// crates/lyng/vm/src/vm/registers.rs:160
		let absolute = registers.base() + u32::from(register);
	add w0, w13, w8
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:272
		&(*slice)[self]
	cmp x10, x0
	b.ls LBB390_69
LBB390_5:
		// crates/lyng/vm/src/vm/dispatch/property.rs:87
		let atom = self.read_atom_constant(frame.code(), u32::from(atom_operand))?;
	ldr x8, [x22, #80]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:499
		unsafe { intrinsics::transmute_unchecked(self) }
	sub w11, w21, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x8, x11
	b.ls LBB390_21
		// crates/lyng/vm/src/vm/dispatch/property.rs:87
		let atom = self.read_atom_constant(frame.code(), u32::from(atom_operand))?;
	ldr x8, [x22, #72]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:745
		match *self {
	ldr x14, [x8, x11, lsl #3]
	cbz x14, LBB390_21
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1599
		unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
	ldr x16, [x14, #80]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/convert/num.rs:264
		Ok(value as Self)
	mov w15, w9
	sub x8, x28, #3
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x16, x15
	b.ls LBB390_22
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x16, [x14, #72]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	add x16, x16, x15, lsl #4
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2059
		Some(&v) => Some(v),
	ldr w15, [x16]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:1338
		match self {
	cmp w15, #4
	b.eq LBB390_22
	ldr w26, [x16, #4]
	cmp w15, #2
	b.ne LBB390_26
	ldp x16, x9, [x20, #88]
	ldr x8, [x20, #104]
	stp x9, x8, [sp, #40]
	ldp x9, x8, [x20, #112]
	stp x9, x8, [sp, #56]
	ldr x8, [x20, #136]
	str x8, [sp, #32]
	add w8, w13, w12
	str x8, [sp, #72]
	ldr x17, [x22, #24]
	ldr x27, [x17, x0, lsl #3]
		// crates/lyng/vm/src/vm/values.rs:17
		.map_or(atom, |installed| installed.canonical_atom(atom))
	ldr x8, [x14, #464]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x8, x26
	b.ls LBB390_13
		// crates/lyng/vm/src/vm/values.rs:17
		.map_or(atom, |installed| installed.canonical_atom(atom))
	ldr x8, [x14, #456]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	add x8, x8, x26, lsl #3
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2059
		Some(&v) => Some(v),
	ldr w9, [x8, #16]!
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:1042
		match self {
	cmp w9, #1
	b.ne LBB390_13
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2059
		Some(&v) => Some(v),
	ldr w26, [x8, #4]
LBB390_13:
		// crates/lyng/types/src/value.rs:95
		if Self::is_tagged_bits(self.0) {
	and x8, x27, #0x7fffffff00000000
	and x8, x8, #0xfff8ffffffffffff
	mov x9, #21474836480
	movk x9, #32760, lsl #48
	cmp x8, x9
		// crates/lyng/vm/src/vm/dispatch/property.rs:89
		let value = if let Some(object) = receiver.as_object_ref() {
	ccmp w27, #0, #4, eq
	b.ne LBB390_38
	mov w8, #1
		// crates/lyng/vm/src/vm/dispatch/property.rs:177
		self.get_property_from_value(agent, host, registry, frame, receiver, key);
	stp w8, w26, [sp, #8]
	str x27, [sp]
	add x0, sp, #128
	mov x1, x22
	mov x21, x16
	mov x2, x16
	ldp x3, x4, [sp, #40]
	ldp x5, x6, [sp, #56]
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::get_property_from_value
	ldr x23, [sp, #72]
		// crates/lyng/vm/src/vm/dispatch.rs:544
		match result {
	ldr x8, [sp, #128]
	cmp x8, x28
	b.ne LBB390_27
		// crates/lyng/vm/src/vm/dispatch.rs:545
		Ok(value) => Ok(Some(value)),
	ldr x21, [sp, #136]
LBB390_17:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1631
		unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
	ldr x8, [x22, #32]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:278
		&mut (*slice)[self]
	cmp x8, x23
	b.ls LBB390_113
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x8, [x22, #24]
		// crates/lyng/vm/src/vm/dispatch/property.rs:185
		self.register_stack[target_index] = value;
	str x21, [x8, x23, lsl #3]
		// crates/lyng/vm/src/frame.rs:425
		self.state.instruction_offset
	b LBB390_46
LBB390_19:
	sub x8, x28, #12
	stur x8, [x29, #-144]
	stp w21, w23, [x29, #-136]
LBB390_20:
		// crates/lyng/vm/src/vm/dispatch_state.rs:303
		Err(error) => return $crate::vm::dispatch_state::Step::Error(error),
	ldp q0, q1, [x29, #-144]
	stp q0, q1, [x19]
	ldur q0, [x29, #-112]
	b LBB390_24
LBB390_21:
	sub x8, x28, #29
	mov x12, x21
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2175
		Err(e) => ControlFlow::Break(Err(e)),
	lsr x10, x21, #32
	b LBB390_23
LBB390_22:
	mov x12, #0
	lsr x10, xzr, #32
LBB390_23:
		// crates/lyng/vm/src/vm/dispatch_state.rs:303
		Err(error) => return $crate::vm::dispatch_state::Step::Error(error),
	bfi x12, x10, #32, #32
	stp x8, x12, [x19]
	str x23, [x19, #16]
	stp w21, w9, [x19, #24]
	ldr q0, [sp, #176]
LBB390_24:
		// crates/lyng/vm/src/vm/dispatch_handlers/property.rs:34
		let prefix = state.prefix.take();
	str q0, [x19, #32]
LBB390_25:
		// crates/lyng/vm/src/vm/dispatch_handlers/property.rs:67
		}
	ldp x29, x30, [sp, #368]
	ldp x20, x19, [sp, #352]
	ldp x22, x21, [sp, #336]
	ldp x24, x23, [sp, #320]
	ldp x26, x25, [sp, #304]
	ldp x28, x27, [sp, #288]
	add sp, sp, #384
	ret
LBB390_26:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2059
		Some(&v) => Some(v),
	ldr x23, [x16, #8]
		// crates/lyng/vm/src/vm/values.rs:833
		_ => Err(VmError::InvalidAtomConstant {
	orr x12, x15, x26, lsl #32
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2175
		Err(e) => ControlFlow::Break(Err(e)),
	lsr x10, x12, #32
	b LBB390_23
LBB390_27:
	mov x9, #-9223372036854775808
		// crates/lyng/vm/src/vm/dispatch.rs:544
		match result {
	cmp x8, x9
	b.ne LBB390_49
	ldr w9, [sp, #136]
	cbnz w9, LBB390_49
		// crates/lyng/vm/src/vm/dispatch.rs:546
		Err(VmError::Abrupt(AbruptCompletion::Throw(value))) => {
	ldr x23, [sp, #144]
	ldr x8, [sp, #32]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:878
		if self < rhs {
	cbz x8, LBB390_32
	ldr x8, [sp, #32]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:882
		Some(unsafe { intrinsics::unchecked_sub(self, rhs) })
	sub x8, x8, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1631
		unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
	ldr x9, [x22, #56]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:229
		if self < slice.len() {
	cmp x8, x9
	b.hs LBB390_32
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x9, [x22, #48]
	mov w10, #80
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:231
		unsafe { Some(slice_get_unchecked(slice, self)) }
	madd x8, x8, x10, x9
		// crates/lyng/vm/src/vm/dispatch.rs:509
		*slot = frame;
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB390_32:
		// crates/lyng/vm/src/vm/dispatch.rs:548
		if self.transfer_to_exception_handler(agent, value)? {
	sub x0, x29, #144
	mov x1, x22
	mov x2, x21
	mov x3, x23
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2173
		match self {
	ldur x8, [x29, #-144]
	ldurb w11, [x29, #-136]
	cmp x8, x28
	b.ne LBB390_62
		// crates/lyng/vm/src/vm/dispatch.rs:548
		if self.transfer_to_exception_handler(agent, value)? {
	tbz w11, #0, LBB390_61
LBB390_35:
		// crates/lyng/vm/src/vm/dispatch.rs:529
		self.dispatch_frame_check_epoch = self.dispatch_frame_check_epoch.wrapping_add(1);
	ldr w8, [x22, #1640]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2380
		intrinsics::wrapping_add(self, rhs)
	add w8, w8, #1
		// crates/lyng/vm/src/vm/dispatch.rs:529
		self.dispatch_frame_check_epoch = self.dispatch_frame_check_epoch.wrapping_add(1);
	str w8, [x22, #1640]
	ldr x8, [sp, #32]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:878
		if self < rhs {
	cbz x8, LBB390_48
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:882
		Some(unsafe { intrinsics::unchecked_sub(self, rhs) })
	sub x8, x8, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1599
		unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
	ldr x9, [x22, #56]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x8, x9
	b.hs LBB390_48
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x9, [x22, #48]
	mov w10, #80
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	madd x8, x8, x10, x9
		// crates/lyng/vm/src/vm/dispatch.rs:523
		*frame = stacked;
	ldr q0, [x8]
	str q0, [x20]
	ldp q0, q1, [x8, #48]
	ldp q3, q2, [x8, #16]
	stp q0, q1, [x20, #48]
	stp q3, q2, [x20, #16]
	b LBB390_48
LBB390_38:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2666
		match self {
	cbz w25, LBB390_42
	ldr x8, [x22, #104]
	cmp x8, x11
	b.ls LBB390_42
		// crates/lyng/vm/src/vm/feedback.rs:2085
		let site = self.feedback_site_for_slot(code, slot?)?;
	ldr x8, [x22, #96]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	add x9, x8, x11, lsl #5
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1599
		unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
	ldr x11, [x9, #16]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2268
		intrinsics::saturating_sub(self, rhs)
	sub w8, w25, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x11, x8
	b.ls LBB390_42
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x9, [x9, #8]
	mov w11, #1160
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x0, w8, w11, x9
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:745
		match *self {
	ldr x8, [x0]
	cmp x8, #10
		// crates/lyng/vm/src/vm/feedback.rs:2086
		match site {
	ccmp x8, #6, #0, ne
	b.eq LBB390_71
LBB390_42:
		// crates/lyng/vm/src/vm/dispatch/property.rs:149
		if let Some(value) = self.try_named_property_load_inline_cache_hit(
	mov x0, x22
	str x16, [sp, #24]
	mov x1, x16
	mov x2, x21
	mov x3, x25
	mov x4, x27
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::try_named_property_load_inline_cache_hit
	ldr x23, [sp, #72]
	tbz w0, #0, LBB390_50
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1631
		unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
	ldr x8, [x22, #32]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:278
		&mut (*slice)[self]
	cmp x8, x23
	b.ls LBB390_114
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x8, [x22, #24]
		// crates/lyng/vm/src/vm/dispatch/property.rs:155
		self.register_stack[target_index] = value;
	str x1, [x8, x23, lsl #3]
LBB390_46:
		// crates/lyng/vm/src/frame.rs:425
		self.state.instruction_offset
	ldr w8, [x20, #56]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:724
		if intrinsics::unlikely(intrinsics::add_with_overflow(self, rhs).1) {
	adds w8, w8, w24
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:457
		if b {
	b.hs LBB390_88
LBB390_47:
		// crates/lyng/vm/src/frame.rs:435
		self.state.instruction_offset = instruction_offset;
	str w8, [x20, #56]
LBB390_48:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:445
		unsafe { &*self.as_ptr().cast_const() }
	ldr x8, [x20, #128]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x8, [x8, #48]
		// crates/lyng/vm/src/frame.rs:425
		self.state.instruction_offset
	ldr w9, [x20, #56]
		// crates/lyng/vm/src/vm/dispatch_state.rs:100
		unsafe { *bytes.as_ptr().add(pc) }
	ldrb w8, [x8, x9]
		// crates/lyng/vm/src/vm/dispatch_state.rs:244
		$crate::vm::dispatch_state::DISPATCH_TABLE[byte as usize],
Lloh1097:
	adrp x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
Lloh1098:
	add x9, x9, lyng_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x8, [x9, x8, lsl #3]
		// crates/lyng/vm/src/vm/dispatch_state.rs:243
		return $crate::vm::dispatch_state::Step::Continue(
	stp x28, x8, [x19]
	b LBB390_25
LBB390_49:
		// crates/lyng/vm/src/vm/dispatch.rs:556
		Err(error) => Err(error),
	ldp w11, w10, [sp, #136]
	lsr w12, w11, #8
	ldr x23, [sp, #144]
	ldp w21, w9, [sp, #152]
	ldr q0, [sp, #160]
	b LBB390_64
LBB390_50:
	mov w8, #1
		// crates/lyng/vm/src/vm/dispatch/property.rs:160
		self.get_property_from_value(agent, host, registry, frame, receiver, key);
	stp w8, w26, [sp, #8]
	str x27, [sp]
	add x0, sp, #80
	mov x1, x22
	ldr x2, [sp, #24]
	ldp x3, x4, [sp, #40]
	ldp x5, x6, [sp, #56]
	mov x7, x20
	bl lyng_vm::vm::property_access::<impl lyng_vm::vm::Vm>::get_property_from_value
		// crates/lyng/vm/src/vm/dispatch.rs:544
		match result {
	ldr x8, [sp, #80]
	cmp x8, x28
	b.ne LBB390_53
		// crates/lyng/vm/src/vm/dispatch.rs:545
		Ok(value) => Ok(Some(value)),
	ldr x21, [sp, #88]
		// crates/lyng/vm/src/frame.rs:420
		self.metadata.code
	ldr w2, [x20, #4]
		// crates/lyng/vm/src/vm/dispatch/property.rs:166
		self.observe_named_property_slow_path(
	mov x0, x22
	ldr x1, [sp, #24]
	mov x3, x25
	mov x4, x27
	mov x5, x26
	mov w6, #0
	bl lyng_vm::vm::feedback::<impl lyng_vm::vm::Vm>::observe_named_property_slow_path
	b LBB390_17
LBB390_53:
	mov x9, #-9223372036854775808
		// crates/lyng/vm/src/vm/dispatch.rs:544
		match result {
	cmp x8, x9
	b.ne LBB390_63
	ldr w9, [sp, #88]
	cbnz w9, LBB390_63
		// crates/lyng/vm/src/vm/dispatch.rs:546
		Err(VmError::Abrupt(AbruptCompletion::Throw(value))) => {
	ldr x23, [sp, #96]
	ldr x8, [sp, #32]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:878
		if self < rhs {
	cbz x8, LBB390_58
	ldr x8, [sp, #32]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:882
		Some(unsafe { intrinsics::unchecked_sub(self, rhs) })
	sub x8, x8, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1631
		unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
	ldr x9, [x22, #56]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:229
		if self < slice.len() {
	cmp x8, x9
	b.hs LBB390_58
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x9, [x22, #48]
	mov w10, #80
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:231
		unsafe { Some(slice_get_unchecked(slice, self)) }
	madd x8, x8, x10, x9
		// crates/lyng/vm/src/vm/dispatch.rs:509
		*slot = frame;
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB390_58:
		// crates/lyng/vm/src/vm/dispatch.rs:548
		if self.transfer_to_exception_handler(agent, value)? {
	sub x0, x29, #144
	mov x1, x22
	ldr x2, [sp, #24]
	mov x3, x23
	bl lyng_vm::vm::exceptions::<impl lyng_vm::vm::Vm>::transfer_to_exception_handler
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2173
		match self {
	ldur x8, [x29, #-144]
	ldurb w11, [x29, #-136]
	cmp x8, x28
	b.ne LBB390_62
		// crates/lyng/vm/src/vm/dispatch.rs:548
		if self.transfer_to_exception_handler(agent, value)? {
	tbnz w11, #0, LBB390_35
LBB390_61:
	mov w12, #0
	mov w11, #0
	mov x8, #-9223372036854775808
	b LBB390_65
LBB390_62:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2175
		Err(e) => ControlFlow::Break(Err(e)),
	ldurb w9, [x29, #-133]
	sub x10, x29, #144
	ldurh w10, [x10, #9]
	orr w12, w10, w9, lsl #16
	ldur w10, [x29, #-132]
	ldur x23, [x29, #-128]
	ldp w21, w9, [x29, #-120]
	ldur q0, [x29, #-112]
	b LBB390_64
LBB390_63:
		// crates/lyng/vm/src/vm/dispatch.rs:556
		Err(error) => Err(error),
	ldp w11, w10, [sp, #88]
	lsr w12, w11, #8
	ldr x23, [sp, #96]
	ldp w21, w9, [sp, #104]
	ldr q0, [sp, #112]
LBB390_64:
	str q0, [sp, #176]
LBB390_65:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2175
		Err(e) => ControlFlow::Break(Err(e)),
	lsl w12, w12, #8
	bfxil x12, x11, #0, #8
	b LBB390_23
LBB390_66:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:569
		slice_index_fail(self.start, slice.len(), slice.len())
Lloh1099:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
Lloh1100:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov x0, x23
	mov x2, x1
	bl core::slice::index::slice_index_fail
	b LBB390_89
LBB390_67:
		// crates/lyng/vm/src/vm/dispatch.rs:237
		return decode_abc_operands_wide(bytes, is_profiled, code, instruction_offset);
	sub x0, x29, #144
	mov w3, #1
	mov x4, x21
	mov x5, x23
	bl lyng_vm::vm::dispatch::decode_abc_operands_wide
		// crates/lyng/vm/src/vm/dispatch_handlers/property.rs:35
		let (a, b, c, feedback_slot, instruction_len) = try_step!(decode_abc_operands(
	ldur x8, [x29, #-144]
		// crates/lyng/vm/src/vm/dispatch_state.rs:301
		match $e {
	cmp x8, x28
	b.ne LBB390_20
		// crates/lyng/vm/src/vm/dispatch_state.rs:302
		Ok(v) => v,
	ldurh w12, [x29, #-132]
	ldurh w8, [x29, #-130]
	ldurh w9, [x29, #-128]
	ldur w25, [x29, #-136]
	ldur w24, [x29, #-124]
		// crates/lyng/vm/src/vm/dispatch_handlers/property.rs:52
		vm.execute_get_named_property_opcode(
	ldr x22, [x20, #80]
		// crates/lyng/vm/src/frame.rs:440
		self.metadata.registers
	ldr w13, [x20, #20]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1599
		unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
	ldr x10, [x22, #32]
		// crates/lyng/vm/src/vm/registers.rs:160
		let absolute = registers.base() + u32::from(register);
	add w0, w13, w8
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:272
		&(*slice)[self]
	cmp x10, x0
	b.hi LBB390_5
LBB390_69:
	str x10, [sp, #24]
Lloh1101:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.759@PAGE
Lloh1102:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.759@PAGEOFF
LBB390_70:
	ldr x1, [sp, #24]
	bl core::panicking::panic_bounds_check
	b LBB390_89
LBB390_71:
		// crates/lyng/vm/src/vm/feedback.rs:2087
		FeedbackSiteState::NamedProperty(feedback) if feedback.monomorphic_fast.is_valid() => {
	ldr x8, [x0, #968]
	cbz x8, LBB390_90
		// crates/lyng/gc/src/arena.rs:606
		self.objects.get_ref(id)
	ldr x12, [x16, #224]
	mov w9, #-1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:882
		Some(unsafe { intrinsics::unchecked_sub(self, rhs) })
	add x9, x27, x9
		// crates/lyng/gc/src/arena/storage.rs:1020
		raw / PRIMITIVE_SLOTS_PER_PAGE,
	lsr w11, w9, #6
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x11, x12
	b.hs LBB390_90
		// crates/lyng/gc/src/arena.rs:606
		self.objects.get_ref(id)
	ldr x12, [x16, #216]
		// crates/lyng/gc/src/arena/storage.rs:1021
		raw % PRIMITIVE_SLOTS_PER_PAGE,
	and x9, x9, #0x3f
		// crates/lyng/gc/src/arena/storage.rs:239
		self.pages.get(page_index)?.get_ref(slot_index)
	ldr x11, [x12, x11, lsl #3]
	mov w12, #80
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x13, w9, w12, x11
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:745
		match *self {
	ldr w9, [x13]
	cmp w9, #1
	b.ne LBB390_90
	stp x17, x10, [sp, #16]
	mov x14, x16
	ldr x9, [x0, #976]
		// crates/lyng/vm/src/vm/dispatch/property.rs:113
		if record.shape() == handler.receiver_shape()
	ldp w10, w11, [x13, #52]
	ldr x12, [x13, #40]
	ldur q0, [x13, #8]
	ldur q1, [x13, #24]
	stp q0, q1, [x29, #-176]
		// crates/lyng/types/src/ids.rs:21
		match NonZeroU32::new(raw) {
	lsr x13, x8, #32
	cmp x13, #0
	csel w13, w13, wzr, ne
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2365
		match (self, other) {
	cbz w10, LBB390_76
	cmp w10, w13
	ccmp x12, x9, #0, eq
	ldp x17, x10, [sp, #16]
	mov x16, x14
	b.eq LBB390_78
	b LBB390_90
LBB390_76:
	ldp x17, x10, [sp, #16]
	mov x16, x14
		// crates/lyng/vm/src/vm/dispatch/property.rs:113
		if record.shape() == handler.receiver_shape()
	cbnz w13, LBB390_90
	cmp x12, x9
	b.ne LBB390_90
LBB390_78:
		// crates/lyng/objects/src/shapes.rs:263
		let offset = low & HANDLER_SLOT_OFFSET_MASK;
	and x9, x8, #0x3fffffff
		// crates/lyng/vm/src/vm/dispatch/property.rs:116
		let fast_value = match handler.slot_location() {
	tbnz w8, #31, LBB390_84
	ldp x17, x10, [sp, #16]
	mov x16, x14
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:1545
		match self {
	cbz w11, LBB390_90
		// crates/lyng/gc/src/arena.rs:812
		self.object_slots.get(id)
	ldr x12, [x16, #640]
		// crates/lyng/gc/src/arena/storage.rs:860
		let slot = self.slots.get((id.get() - 1) as usize)?;
	sub w8, w11, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x12, x8
	b.ls LBB390_90
		// crates/lyng/gc/src/arena.rs:812
		self.object_slots.get(id)
	ldr x11, [x16, #632]
	mov w12, #24
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x8, w8, w12, x11
		// crates/lyng/gc/src/arena/storage.rs:861
		if slot.occupied {
	ldrb w11, [x8, #19]
	cmp w11, #1
	b.ne LBB390_90
		// crates/lyng/gc/src/arena/storage.rs:862
		Some(&slot.values)
	ldr x11, [x8, #8]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x9, x11
	b.hs LBB390_90
		// crates/lyng/gc/src/arena/storage.rs:862
		Some(&slot.values)
	ldr x8, [x8]
	b LBB390_86
LBB390_84:
		// crates/lyng/gc/src/arena/records.rs:396
		if index < RUNTIME_OBJECT_INLINE_SLOT_COUNT {
	cmp x9, #4
	ldp x17, x10, [sp, #16]
	mov x16, x14
	b.hs LBB390_90
	sub x8, x29, #176
LBB390_86:
	ldr x25, [x8, x9, lsl #3]
		// crates/lyng/vm/src/vm/feedback.rs:2133
		site.record_execution();
	bl lyng_vm::vm::feedback::FeedbackSiteState::record_execution
		// crates/lyng/vm/src/vm/feedback.rs:2136
		self.observe_tier_feedback_event(code);
	ldp x0, x1, [x22, #120]
	mov x2, x21
	bl lyng_vm::vm::tiering::<impl lyng_vm::vm::Vm>::observe_tier_feedback_event
	ldr x8, [sp, #24]
	ldr x9, [sp, #72]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:278
		&mut (*slice)[self]
	cmp x8, x9
	b.ls LBB390_115
LBB390_87:
	ldr x8, [sp, #72]
	ldr x9, [sp, #16]
	str x25, [x9, x8, lsl #3]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:724
		if intrinsics::unlikely(intrinsics::add_with_overflow(self, rhs).1) {
	adds w8, w24, w23
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:457
		if b {
	b.lo LBB390_47
LBB390_88:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:971
		None => expect_failed(msg),
Lloh1103:
	adrp x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGE
Lloh1104:
	add x0, x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGEOFF
Lloh1105:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGE
Lloh1106:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGEOFF
	mov w1, #41
	bl core::option::expect_failed
LBB390_89:
	brk #0x1
LBB390_90:
		// crates/lyng/vm/src/vm/feedback.rs:2113
		if feedback.monomorphic_proto_fast.is_valid() =>
	ldr x13, [x0, #984]
	cbz x13, LBB390_42
		// crates/lyng/gc/src/arena.rs:606
		self.objects.get_ref(id)
	ldr x11, [x16, #224]
	mov w8, #-1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:882
		Some(unsafe { intrinsics::unchecked_sub(self, rhs) })
	add x8, x27, x8
		// crates/lyng/gc/src/arena/storage.rs:1020
		raw / PRIMITIVE_SLOTS_PER_PAGE,
	lsr w9, w8, #6
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x9, x11
	b.hs LBB390_42
		// crates/lyng/gc/src/arena.rs:606
		self.objects.get_ref(id)
	ldr x12, [x16, #216]
		// crates/lyng/gc/src/arena/storage.rs:1021
		raw % PRIMITIVE_SLOTS_PER_PAGE,
	and x8, x8, #0x3f
		// crates/lyng/gc/src/arena/storage.rs:239
		self.pages.get(page_index)?.get_ref(slot_index)
	ldr x9, [x12, x9, lsl #3]
	mov w14, #80
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x15, w8, w14, x9
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:745
		match *self {
	ldr w8, [x15]
	cmp w8, #1
	b.ne LBB390_42
	stp x17, x10, [sp, #16]
	mov x17, x16
	ldr x8, [x0, #992]
	ldr x14, [x0, #1000]
	ldr x9, [x0, #1008]
		// crates/lyng/vm/src/vm/dispatch/property.rs:1508
		if record.shape() != handler.receiver_shape()
	ldp w10, w16, [x15, #48]
	ldr x15, [x15, #40]
	cmp w10, #0
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2365
		match (self, other) {
	cbz w16, LBB390_95
	ccmp w16, w13, #0, ne
	b LBB390_96
LBB390_95:
		// crates/lyng/vm/src/vm/dispatch/property.rs:1508
		if record.shape() != handler.receiver_shape()
	ccmp w13, #0, #0, ne
LBB390_96:
	ccmp x15, x14, #0, eq
	mov x16, x17
	b.ne LBB390_42
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:882
		Some(unsafe { intrinsics::unchecked_sub(self, rhs) })
	sub w10, w10, #1
		// crates/lyng/gc/src/arena/storage.rs:1020
		raw / PRIMITIVE_SLOTS_PER_PAGE,
	lsr x13, x10, #6
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x13, x11
	b.hs LBB390_42
		// crates/lyng/gc/src/arena/storage.rs:1021
		raw % PRIMITIVE_SLOTS_PER_PAGE,
	and x10, x10, #0x3f
		// crates/lyng/gc/src/arena/storage.rs:239
		self.pages.get(page_index)?.get_ref(slot_index)
	ldr x11, [x12, x13, lsl #3]
	mov w12, #80
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x13, w10, w12, x11
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:745
		match *self {
	ldr w10, [x13]
	cmp w10, #1
	b.ne LBB390_42
		// crates/lyng/vm/src/vm/dispatch/property.rs:1515
		if prototype_record.shape() != handler.prototype_shape()
	ldp w12, w10, [x13, #52]
	ldr x11, [x13, #40]
	ldur q0, [x13, #8]
	ldur q1, [x13, #24]
	stp q0, q1, [x29, #-144]
		// crates/lyng/types/src/ids.rs:21
		match NonZeroU32::new(raw) {
	lsr x13, x8, #32
	cmp x13, #0
	csel w13, w13, wzr, ne
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2365
		match (self, other) {
	cbz w12, LBB390_101
	cmp w12, w13
	ccmp x11, x9, #0, eq
	mov x16, x17
	b.ne LBB390_42
	b LBB390_103
LBB390_101:
	mov x16, x17
		// crates/lyng/vm/src/vm/dispatch/property.rs:1515
		if prototype_record.shape() != handler.prototype_shape()
	cbnz w13, LBB390_42
	cmp x11, x9
	b.ne LBB390_42
LBB390_103:
		// crates/lyng/objects/src/shapes.rs:401
		let offset = low & HANDLER_SLOT_OFFSET_MASK;
	and x9, x8, #0x3fffffff
		// crates/lyng/vm/src/vm/dispatch/property.rs:1520
		let value = match handler.slot_location() {
	tbnz w8, #31, LBB390_109
	mov x16, x17
		// crates/lyng/vm/src/vm/dispatch/property.rs:1523
		.object_slots(prototype_record.named_slots()?)?
	cbz w10, LBB390_42
		// crates/lyng/gc/src/arena.rs:812
		self.object_slots.get(id)
	ldr x11, [x16, #640]
		// crates/lyng/gc/src/arena/storage.rs:860
		let slot = self.slots.get((id.get() - 1) as usize)?;
	sub w8, w10, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x11, x8
	b.ls LBB390_42
		// crates/lyng/gc/src/arena.rs:812
		self.object_slots.get(id)
	ldr x10, [x16, #632]
	mov w11, #24
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x8, w8, w11, x10
		// crates/lyng/gc/src/arena/storage.rs:861
		if slot.occupied {
	ldrb w10, [x8, #19]
	cmp w10, #1
	b.ne LBB390_42
		// crates/lyng/gc/src/arena/storage.rs:862
		Some(&slot.values)
	ldr x10, [x8, #8]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2058
		match self {
	cmp x9, x10
	b.hs LBB390_42
		// crates/lyng/gc/src/arena/storage.rs:862
		Some(&slot.values)
	ldr x8, [x8]
	b LBB390_111
LBB390_109:
		// crates/lyng/gc/src/arena/records.rs:396
		if index < RUNTIME_OBJECT_INLINE_SLOT_COUNT {
	cmp x9, #3
	mov x16, x17
	b.hi LBB390_42
	sub x8, x29, #144
LBB390_111:
	ldr x25, [x8, x9, lsl #3]
		// crates/lyng/vm/src/vm/feedback.rs:2133
		site.record_execution();
	bl lyng_vm::vm::feedback::FeedbackSiteState::record_execution
		// crates/lyng/vm/src/vm/feedback.rs:2136
		self.observe_tier_feedback_event(code);
	ldp x0, x1, [x22, #120]
	mov x2, x21
	bl lyng_vm::vm::tiering::<impl lyng_vm::vm::Vm>::observe_tier_feedback_event
	ldr x8, [sp, #24]
	ldr x9, [sp, #72]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:278
		&mut (*slice)[self]
	cmp x8, x9
	b.hi LBB390_87
Lloh1107:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.761@PAGE
Lloh1108:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.761@PAGEOFF
	ldr x0, [sp, #72]
	b LBB390_70
LBB390_113:
	str x8, [sp, #24]
Lloh1109:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.763@PAGE
Lloh1110:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.763@PAGEOFF
	mov x0, x23
	b LBB390_70
LBB390_114:
	str x8, [sp, #24]
Lloh1111:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.762@PAGE
Lloh1112:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.762@PAGEOFF
	mov x0, x23
	b LBB390_70
LBB390_115:
Lloh1113:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.760@PAGE
Lloh1114:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.760@PAGEOFF
	ldr x0, [sp, #72]
	b LBB390_70
		// crates/lyng/vm/src/vm/dispatch_handlers/property.rs:31
		pub extern "C" fn op_get_named_property(state: &mut DispatchState) -> Step {
	bl core::panicking::panic_cannot_unwind
