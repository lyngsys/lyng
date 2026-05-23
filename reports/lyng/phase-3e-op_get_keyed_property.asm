lyng_js_vm::vm::dispatch_handlers::property::op_get_keyed_property:
Lfunc_begin389:
		// crates/lyng-js/vm/src/vm/dispatch_handlers/property.rs:121
		pub extern "C" fn op_get_keyed_property(state: &mut DispatchState) -> Step {
	stp x28, x27, [sp, #-96]!
	stp x26, x25, [sp, #16]
	stp x24, x23, [sp, #32]
	stp x22, x21, [sp, #48]
	stp x20, x19, [sp, #64]
	stp x29, x30, [sp, #80]
	add x29, sp, #80
	sub sp, sp, #592
	mov x19, x8
		// crates/lyng-js/vm/src/frame.rs:420
		self.metadata.code
	ldr w27, [x0, #4]
		// crates/lyng-js/vm/src/frame.rs:425
		self.state.instruction_offset
	ldr w5, [x0, #56]
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
	subs x2, x1, x5
	b.lo LBB389_134
	mov x20, x0
	mov x21, #33
	movk x21, #32768, lsl #48
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x9, [x9, #48]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:89
		let ptr = unsafe { crate::intrinsics::offset(ptr, offset) };
	add x1, x9, x5
		// crates/lyng-js/vm/src/vm/dispatch.rs:236
		if prefix.is_some() {
	cmp w8, #152
	b.ne LBB389_135
	and x8, x2, #0x7ffffffffffffffe
		// crates/lyng-js/vm/src/vm/dispatch.rs:239
		let [_, ra, rb, rc, ..] = bytes else {
	cmp x2, #4
	ccmp x8, #4, #4, hs
	b.eq LBB389_18
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:3923
		unsafe { mem::transmute(bytes) }
	ldrh w23, [x1, #4]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:1338
		match self {
	cbz w23, LBB389_18
		// crates/lyng-js/vm/src/vm/dispatch.rs:248
		u16::from(*ra),
	ldrb w10, [x1, #1]
		// crates/lyng-js/vm/src/vm/dispatch.rs:249
		u16::from(*rb),
	ldrb w8, [x1, #2]
	mov w9, #6
	stp w9, w10, [sp, #80]
		// crates/lyng-js/vm/src/vm/dispatch.rs:250
		u16::from(*rc),
	ldrb w9, [x1, #3]
		// crates/lyng-js/vm/src/vm/dispatch_handlers/property.rs:142
		vm.execute_get_keyed_property_opcode(
	ldr x21, [x20, #80]
		// crates/lyng-js/vm/src/frame.rs:440
		self.metadata.registers
	ldr w10, [x20, #20]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1599
		unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
	ldr x1, [x21, #32]
		// crates/lyng-js/vm/src/vm/registers.rs:160
		let absolute = registers.base() + u32::from(register);
	add w0, w10, w8
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:272
		&(*slice)[self]
	cmp x1, x0
	b.ls LBB389_137
LBB389_5:
		// crates/lyng-js/vm/src/vm/registers.rs:160
		let absolute = registers.base() + u32::from(register);
	add w8, w10, w9
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:272
		&(*slice)[self]
	cmp x1, x8
	b.ls LBB389_195
	ldr x22, [x20, #88]
	ldr x9, [x20, #136]
	str x9, [sp, #88]
		// crates/lyng-js/vm/src/vm/registers.rs:13
		self.register_stack[absolute]
	ldr x9, [x21, #24]
	ldr x25, [x9, x0, lsl #3]
		// crates/lyng-js/types/src/value.rs:85
		((bits & TAG_HEADER) == TAG_HEADER) && Self::tag_kind_bits(bits).is_some()
	and x24, x25, #0x7ff8000000000000
	ubfx x28, x25, #32, #16
		// crates/lyng-js/types/src/value.rs:95
		if Self::is_tagged_bits(self.0) {
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
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:1545
		match self {
	tbnz w26, #31, LBB389_31
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2666
		match self {
	cbz w23, LBB389_13
		// crates/lyng-js/vm/src/vm/feedback.rs:2205
		let site = self.feedback_site_for_slot(code, slot?)?;
	ldr x8, [x21, #104]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:499
		unsafe { intrinsics::transmute_unchecked(self) }
	sub w11, w27, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x8, x11
	b.ls LBB389_13
		// crates/lyng-js/vm/src/vm/feedback.rs:2205
		let site = self.feedback_site_for_slot(code, slot?)?;
	ldr x8, [x21, #96]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	add x8, x8, x11, lsl #5
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1599
		unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
	ldr x9, [x8, #16]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2268
		intrinsics::saturating_sub(self, rhs)
	sub w10, w23, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x9, x10
	b.ls LBB389_13
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x8, [x8, #8]
	mov w9, #1160
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x8, w10, w9, x8
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:745
		match *self {
	ldr x9, [x8]
	cmp x9, #10
		// crates/lyng-js/vm/src/vm/feedback.rs:2206
		match site {
	ccmp x9, #4, #2, ne
	b.lo LBB389_106
LBB389_13:
		// crates/lyng-js/vm/src/frame.rs:420
		self.metadata.code
	ldr w2, [x20, #4]
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:440
		if let Some(value) = self.try_keyed_dense_index_load_inline_cache_hit(
	mov x0, x21
	mov x1, x22
	mov x3, x23
	mov x4, x25
	mov x5, x26
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::try_keyed_dense_index_load_inline_cache_hit
	tbz w0, #0, LBB389_68
LBB389_15:
	mov x8, x1
		// crates/lyng-js/vm/src/frame.rs:440
		self.metadata.registers
	ldr w9, [x20, #20]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1631
		unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
	ldr x1, [x21, #32]
	ldr w10, [sp, #84]
		// crates/lyng-js/vm/src/vm/registers.rs:160
		let absolute = registers.base() + u32::from(register);
	add w0, w9, w10
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:278
		&mut (*slice)[self]
	cmp x1, x0
	b.ls LBB389_198
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x9, [x21, #24]
		// crates/lyng-js/vm/src/vm/registers.rs:28
		self.register_stack[absolute] = value;
	str x8, [x9, x0, lsl #3]
		// crates/lyng-js/vm/src/frame.rs:425
		self.state.instruction_offset
	ldr w8, [x20, #56]
	ldr w9, [sp, #80]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:724
		if intrinsics::unlikely(intrinsics::add_with_overflow(self, rhs).1) {
	adds w8, w8, w9
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:457
		if b {
	b.hs LBB389_133
LBB389_17:
		// crates/lyng-js/vm/src/frame.rs:435
		self.state.instruction_offset = instruction_offset;
	str w8, [x20, #56]
	b LBB389_125
LBB389_18:
	sub x8, x21, #12
	stur x8, [x29, #-160]
	stp w27, w5, [x29, #-152]
LBB389_19:
		// crates/lyng-js/vm/src/vm/dispatch_state.rs:303
		Err(error) => return $crate::vm::dispatch_state::Step::Error(error),
	ldp q0, q1, [x29, #-160]
	stp q0, q1, [x19]
	ldur q0, [x29, #-128]
	str q0, [x19, #32]
	b LBB389_126
LBB389_20:
		// crates/lyng-js/ops/src/errors.rs:135
		error_value(agent, ErrorKind::Type)
	mov x0, x22
	mov w1, #5
	bl lyng_js_ops::errors::error_value
	mov x27, x0
	ldr x8, [sp, #88]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:878
		if self < rhs {
	cbz x8, LBB389_24
	ldr x8, [sp, #88]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:882
		Some(unsafe { intrinsics::unchecked_sub(self, rhs) })
	sub x8, x8, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1631
		unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
	ldr x9, [x21, #56]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:229
		if self < slice.len() {
	cmp x8, x9
	b.hs LBB389_24
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x9, [x21, #48]
	mov w10, #80
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:231
		unsafe { Some(slice_get_unchecked(slice, self)) }
	madd x8, x8, x10, x9
		// crates/lyng-js/vm/src/vm/dispatch.rs:509
		*slot = frame;
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB389_24:
		// crates/lyng-js/vm/src/vm/dispatch.rs:548
		if self.transfer_to_exception_handler(agent, value)? {
	sub x0, x29, #160
	mov x1, x21
	mov x2, x22
	mov x3, x27
	bl lyng_js_vm::vm::exceptions::<impl lyng_js_vm::vm::Vm>::transfer_to_exception_handler
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2173
		match self {
	ldur x8, [x29, #-160]
	ldurb w5, [x29, #-152]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_59
		// crates/lyng-js/vm/src/vm/dispatch.rs:548
		if self.transfer_to_exception_handler(agent, value)? {
	tbz w5, #0, LBB389_58
LBB389_27:
	ldr w8, [x21, #1640]
	add w8, w8, #1
	str w8, [x21, #1640]
	ldr x8, [sp, #88]
	cbz x8, LBB389_125
	ldr x8, [sp, #88]
	sub x8, x8, #1
	ldr x9, [x21, #56]
	cmp x8, x9
	b.hs LBB389_125
	ldr x9, [x21, #48]
	mov w10, #80
	madd x8, x8, x10, x9
	ldr q0, [x8]
	str q0, [x20]
	ldp q0, q1, [x8, #48]
	ldp q3, q2, [x8, #16]
	stp q0, q1, [x20, #48]
	stp q3, q2, [x20, #16]
		// crates/lyng-js/vm/src/vm/dispatch.rs:522
		if let Some(stacked) = self.frames.get(index).copied() {
	b LBB389_125
LBB389_30:
	mov x9, #30064771072
	movk x9, #32760, lsl #48
		// crates/lyng-js/types/src/value.rs:95
		if Self::is_tagged_bits(self.0) {
	cmp x8, x9
		// crates/lyng-js/types/src/ids.rs:21
		match NonZeroU32::new(raw) {
	ccmp w26, #0, #4, eq
	b.ne LBB389_63
LBB389_31:
		// crates/lyng-js/vm/src/vm/property_access.rs:1585
		let mut bridge = VmToPrimitiveBridge {
	stp x21, x22, [x29, #-160]
	ldp x9, x8, [sp, #48]
	stp x9, x8, [x29, #-144]
	ldp x9, x8, [sp, #64]
	stp x9, x8, [x29, #-128]
	stur x20, [x29, #-112]
		// crates/lyng-js/vm/src/vm/property_access.rs:1592
		lyng_js_ops::object::to_primitive(&mut bridge, value, hint)
	sub x0, x29, #208
	sub x1, x29, #160
	mov x2, x26
	mov w3, #1
	bl lyng_js_ops::object::conversions::to_primitive
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2173
		match self {
	ldp x8, x5, [x29, #-208]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_49
		// crates/lyng-js/vm/src/frame.rs:420
		self.metadata.code
	ldr w3, [x20, #4]
		// crates/lyng-js/vm/src/frame.rs:425
		self.state.instruction_offset
	ldr w4, [x20, #56]
		// crates/lyng-js/vm/src/vm/property_access.rs:462
		self.value_to_property_key(
	add x0, sp, #144
	mov x1, x21
	mov x2, x22
	bl lyng_js_vm::vm::values::<impl lyng_js_vm::vm::Vm>::value_to_property_key
		// crates/lyng-js/vm/src/vm/dispatch.rs:544
		match result {
	ldr x8, [sp, #144]
	ldr w5, [sp, #152]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_50
		// crates/lyng-js/vm/src/vm/dispatch.rs:545
		Ok(value) => Ok(Some(value)),
	ldr w26, [sp, #156]
	mov x8, #9221120237041090560
		// crates/lyng-js/types/src/value.rs:95
		if Self::is_tagged_bits(self.0) {
	cmp x24, x8
	ccmp w25, #0, #4, eq
	ccmp w28, #5, #0, ne
	b.ne LBB389_64
LBB389_36:
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:475
		if let Some(index) = key.as_index() {
	cbz w5, LBB389_76
	cmp w5, #1
	b.ne LBB389_87
		// crates/lyng-js/vm/src/frame.rs:420
		self.metadata.code
	ldr w3, [x20, #4]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2666
		match self {
	cbz w23, LBB389_97
		// crates/lyng-js/vm/src/vm/feedback.rs:2151
		let site = self.feedback_site_for_slot(code, slot?)?;
	ldr x1, [x21, #104]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:499
		unsafe { intrinsics::transmute_unchecked(self) }
	sub w8, w3, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x1, x8
	b.ls LBB389_98
		// crates/lyng-js/vm/src/vm/feedback.rs:2151
		let site = self.feedback_site_for_slot(code, slot?)?;
	ldr x9, [x21, #96]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	add x9, x9, x8, lsl #5
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1599
		unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
	ldr x10, [x9, #16]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2268
		intrinsics::saturating_sub(self, rhs)
	sub w8, w23, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x10, x8
	b.ls LBB389_98
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x9, [x9, #8]
	mov w10, #1160
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x8, w8, w10, x9
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:745
		match *self {
	ldr x9, [x8]
	cmp x9, #10
	b.eq LBB389_98
		// crates/lyng-js/vm/src/vm/feedback.rs:2152
		match site {
	cmp x9, #3
	b.hi LBB389_98
		// crates/lyng-js/vm/src/vm/feedback.rs:2154
		if feedback.monomorphic_named_fast_atom == atom.raw()
	ldr w9, [x8, #1148]
	cmp w9, w26
	b.ne LBB389_168
		// crates/lyng-js/vm/src/vm/feedback.rs:2155
		&& feedback.monomorphic_named_fast.is_valid() =>
	ldr x9, [x8, #1088]
	cbz x9, LBB389_168
		// crates/lyng-js/gc/src/arena.rs:606
		self.objects.get_ref(id)
	ldr x12, [x22, #224]
	mov w10, #-1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:882
		Some(unsafe { intrinsics::unchecked_sub(self, rhs) })
	add x10, x25, x10
		// crates/lyng-js/gc/src/arena/storage.rs:1020
		raw / PRIMITIVE_SLOTS_PER_PAGE,
	lsr w11, w10, #6
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x11, x12
	b.hs LBB389_168
		// crates/lyng-js/gc/src/arena.rs:606
		self.objects.get_ref(id)
	ldr x12, [x22, #216]
		// crates/lyng-js/gc/src/arena/storage.rs:1021
		raw % PRIMITIVE_SLOTS_PER_PAGE,
	and x10, x10, #0x3f
		// crates/lyng-js/gc/src/arena/storage.rs:239
		self.pages.get(page_index)?.get_ref(slot_index)
	ldr x11, [x12, x11, lsl #3]
	mov w12, #80
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x14, w10, w12, x11
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:745
		match *self {
	ldr w10, [x14]
	cmp w10, #1
	b.ne LBB389_168
	ldr x10, [x8, #1096]
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1474
		if record.shape() != handler.receiver_shape()
	ldp w13, w11, [x14, #52]
	ldr x12, [x14, #40]
	ldur q0, [x14, #8]
	stur q0, [x29, #-160]
	ldur q0, [x14, #24]
	stur q0, [x29, #-144]
		// crates/lyng-js/types/src/ids.rs:21
		match NonZeroU32::new(raw) {
	lsr x14, x9, #32
	cmp x14, #0
	csel w14, w14, wzr, ne
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2365
		match (self, other) {
	cbz w13, LBB389_159
	cmp w13, w14
	ccmp x12, x10, #0, eq
	b.eq LBB389_161
	b LBB389_168
LBB389_49:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2175
		Err(e) => ControlFlow::Break(Err(e)),
	ldp q0, q1, [x29, #-192]
	stp q0, q1, [sp, #160]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2189
		Err(e) => Err(From::from(e)),
	str x5, [sp, #152]
LBB389_50:
	mov x9, #-9223372036854775808
		// crates/lyng-js/vm/src/vm/dispatch.rs:544
		match result {
	cmp x8, x9
	b.ne LBB389_60
	cbnz w5, LBB389_60
		// crates/lyng-js/vm/src/vm/dispatch.rs:546
		Err(VmError::Abrupt(AbruptCompletion::Throw(value))) => {
	ldr x27, [sp, #160]
	ldr x8, [sp, #88]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:878
		if self < rhs {
	cbz x8, LBB389_55
	ldr x8, [sp, #88]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:882
		Some(unsafe { intrinsics::unchecked_sub(self, rhs) })
	sub x8, x8, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1631
		unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
	ldr x9, [x21, #56]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:229
		if self < slice.len() {
	cmp x8, x9
	b.hs LBB389_55
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x9, [x21, #48]
	mov w10, #80
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:231
		unsafe { Some(slice_get_unchecked(slice, self)) }
	madd x8, x8, x10, x9
		// crates/lyng-js/vm/src/vm/dispatch.rs:509
		*slot = frame;
	ldr q0, [x20]
	str q0, [x8]
	ldp q0, q1, [x20, #48]
	ldp q3, q2, [x20, #16]
	stp q0, q1, [x8, #48]
	stp q3, q2, [x8, #16]
LBB389_55:
		// crates/lyng-js/vm/src/vm/dispatch.rs:548
		if self.transfer_to_exception_handler(agent, value)? {
	sub x0, x29, #160
	mov x1, x21
	mov x2, x22
	mov x3, x27
	bl lyng_js_vm::vm::exceptions::<impl lyng_js_vm::vm::Vm>::transfer_to_exception_handler
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2173
		match self {
	ldur x8, [x29, #-160]
	ldurb w5, [x29, #-152]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_59
		// crates/lyng-js/vm/src/vm/dispatch.rs:548
		if self.transfer_to_exception_handler(agent, value)? {
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
		// crates/lyng-js/vm/src/vm/dispatch.rs:556
		Err(error) => Err(error),
	lsr w9, w5, #8
	ldr w10, [sp, #156]
	ldr x27, [sp, #160]
	ldur q0, [sp, #168]
	str q0, [sp, #192]
	ldr x11, [sp, #184]
LBB389_61:
	str x11, [sp, #208]
LBB389_62:
		// crates/lyng-js/vm/src/vm/dispatch_state.rs:303
		Err(error) => return $crate::vm::dispatch_state::Step::Error(error),
	lsl w9, w9, #8
	bfxil x9, x5, #0, #8
	orr x9, x9, x10, lsl #32
	stp x8, x9, [x19]
	str x27, [x19, #16]
	ldr q0, [sp, #192]
	stur q0, [x19, #24]
	ldr x8, [sp, #208]
	str x8, [x19, #40]
	b LBB389_126
LBB389_63:
	mov w5, #2
		// crates/lyng-js/vm/src/vm/property_access.rs:458
		return Ok(PropertyKey::from_symbol(symbol));
	stp w5, w26, [sp, #152]
	mov x8, #33
	movk x8, #32768, lsl #48
	str x8, [sp, #144]
	mov x8, #9221120237041090560
		// crates/lyng-js/types/src/value.rs:95
		if Self::is_tagged_bits(self.0) {
	cmp x24, x8
	ccmp w25, #0, #4, eq
	ccmp w28, #5, #0, ne
	b.eq LBB389_36
LBB389_64:
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:588
		self.get_property_from_value(agent, host, registry, frame, receiver, key);
	stp w5, w26, [sp, #8]
	sub x0, x29, #256
	str x25, [sp]
	mov x1, x21
	mov x2, x22
	ldp x3, x4, [sp, #48]
	ldp x5, x6, [sp, #64]
	mov x7, x20
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::get_property_from_value
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:590
		self.handle_dispatch_result(agent, frame_depth, frame, property_result)?
	sub x0, x29, #160
	sub x5, x29, #256
	mov x1, x21
	mov x2, x22
	ldr x3, [sp, #88]
	mov x4, x20
	bl lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::handle_dispatch_result
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2173
		match self {
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_158
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:589
		let Some(value) =
	tbnz w5, #0, LBB389_131
	b LBB389_125
LBB389_68:
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:451
		let value = if let Some(result) = self.mapped_arguments_get(agent, object, index) {
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
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:457
		} else if let Some(value) = Self::try_fast_typed_array_index_value(agent, object, index)
	mov x0, x22
	mov x1, x25
	mov x2, x26
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::try_fast_typed_array_index_value
	tbz w0, #0, LBB389_92
	mov x27, x1
	b LBB389_95
LBB389_73:
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:452
		let Some(value) = self.handle_dispatch_result(agent, frame_depth, frame, result)?
	sub x0, x29, #160
	add x5, sp, #96
	mov x1, x21
	mov x2, x22
	ldr x3, [sp, #88]
	mov x4, x20
	bl lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::handle_dispatch_result
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2173
		match self {
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_96
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:452
		let Some(value) = self.handle_dispatch_result(agent, frame_depth, frame, result)?
	tbnz w5, #0, LBB389_95
	b LBB389_125
LBB389_76:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2666
		match self {
	cbz w23, LBB389_80
	ldr w27, [x20, #4]
		// crates/lyng-js/vm/src/vm/feedback.rs:2205
		let site = self.feedback_site_for_slot(code, slot?)?;
	ldr x8, [x21, #104]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:499
		unsafe { intrinsics::transmute_unchecked(self) }
	sub w28, w27, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x8, x28
	b.ls LBB389_80
		// crates/lyng-js/vm/src/vm/feedback.rs:2205
		let site = self.feedback_site_for_slot(code, slot?)?;
	ldr x8, [x21, #96]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	add x8, x8, x28, lsl #5
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1599
		unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
	ldr x9, [x8, #16]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2268
		intrinsics::saturating_sub(self, rhs)
	sub w24, w23, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x9, x24
	b.ls LBB389_80
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x8, [x8, #8]
	mov w9, #1160
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x8, w24, w9, x8
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:745
		match *self {
	ldr x9, [x8]
	cmp x9, #10
		// crates/lyng-js/vm/src/vm/feedback.rs:2206
		match site {
	ccmp x9, #4, #2, ne
	b.lo LBB389_138
LBB389_80:
		// crates/lyng-js/vm/src/frame.rs:420
		self.metadata.code
	ldr w2, [x20, #4]
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:489
		if let Some(value) = self.try_keyed_dense_index_load_inline_cache_hit(
	mov x0, x21
	mov x1, x22
	mov x3, x23
	mov x4, x25
	mov x5, x26
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::try_keyed_dense_index_load_inline_cache_hit
	tbnz w0, #0, LBB389_15
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:500
		let value = if let Some(result) = self.mapped_arguments_get(agent, object, index) {
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
	b.ne LBB389_122
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:508
		Self::try_fast_typed_array_index_value(agent, object, index)
	mov x0, x22
	mov x1, x25
	mov x2, x26
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::try_fast_typed_array_index_value
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:507
		} else if let Some(value) =
	tbz w0, #0, LBB389_127
	mov x27, x1
	b LBB389_130
LBB389_87:
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:577
		self.get_property_from_value(agent, host, registry, frame, receiver, key);
	stp w5, w26, [sp, #8]
	add x0, sp, #368
	str x25, [sp]
	mov x1, x21
	mov x2, x22
	ldp x3, x4, [sp, #48]
	ldp x5, x6, [sp, #64]
	mov x7, x20
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::get_property_from_value
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:579
		self.handle_dispatch_result(agent, frame_depth, frame, property_result)?
	sub x0, x29, #160
	add x5, sp, #368
	mov x1, x21
	mov x2, x22
	ldr x3, [sp, #88]
	mov x4, x20
	bl lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::handle_dispatch_result
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2173
		match self {
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_158
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:578
		let Some(value) =
	tbz w5, #0, LBB389_125
		// crates/lyng-js/vm/src/frame.rs:420
		self.metadata.code
	ldr w1, [x20, #4]
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:583
		self.observe_keyed_generic_slow_path(frame.code(), feedback_slot);
	mov x0, x21
	mov x2, x23
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::observe_keyed_generic_slow_path
	b LBB389_131
LBB389_92:
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:461
		Self::try_fast_own_index_value(agent, object, index)?
	sub x0, x29, #160
	mov x1, x22
	mov x2, x25
	mov x3, x26
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::try_fast_own_index_value
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2173
		match self {
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_158
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:463
		if let Some(value) = value {
	tbz w5, #0, LBB389_31
LBB389_95:
		// crates/lyng-js/vm/src/frame.rs:420
		self.metadata.code
	ldr w2, [x20, #4]
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:464
		self.observe_keyed_index_access(agent, frame.code(), feedback_slot, object, index);
	mov x0, x21
	mov x1, x22
	mov x3, x23
	mov x4, x25
	mov x5, x26
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::observe_keyed_index_access
	b LBB389_131
LBB389_96:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2175
		Err(e) => ControlFlow::Break(Err(e)),
	sub x9, x29, #160
	ldur q0, [x9, #24]
	str q0, [sp, #192]
	ldur x9, [x29, #-120]
	str x9, [sp, #208]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2189
		Err(e) => Err(From::from(e)),
	lsr w9, w5, #8
	lsr x10, x5, #32
	b LBB389_62
LBB389_97:
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:547
		if let Some(value) = self.try_keyed_property_load_inline_cache(
	ldr x1, [x21, #104]
LBB389_98:
	ldr x0, [x21, #96]
	mov x2, x22
	mov x4, x23
	mov x5, x25
	mov x6, x26
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::try_keyed_property_load_inline_cache
	tbz w0, #0, LBB389_101
	mov x27, x1
		// crates/lyng-js/vm/src/frame.rs:420
		self.metadata.code
	ldr w1, [x20, #4]
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:554
		self.record_feedback_slot(frame.code(), feedback_slot);
	mov x0, x21
	mov x2, x23
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::record_feedback_slot
	b LBB389_131
LBB389_101:
	mov w8, #1
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:560
		self.get_property_from_value(agent, host, registry, frame, receiver, key);
	stp w8, w26, [sp, #8]
	str x25, [sp]
	add x0, sp, #320
	mov x1, x21
	mov x2, x22
	ldp x3, x4, [sp, #48]
	ldp x5, x6, [sp, #64]
	mov x7, x20
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::get_property_from_value
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:562
		self.handle_dispatch_result(agent, frame_depth, frame, property_result)?
	sub x0, x29, #160
	add x5, sp, #320
	mov x1, x21
	mov x2, x22
	ldr x3, [sp, #88]
	mov x4, x20
	bl lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::handle_dispatch_result
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2173
		match self {
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_158
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:561
		let Some(value) =
	tbz w5, #0, LBB389_125
		// crates/lyng-js/vm/src/frame.rs:420
		self.metadata.code
	ldr w2, [x20, #4]
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:566
		self.observe_keyed_atom_slow_path(
	mov x0, x21
	mov x1, x22
	mov x3, x23
	mov x4, x25
	mov x5, x26
	mov w6, #0
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::observe_keyed_atom_slow_path
	b LBB389_131
LBB389_106:
		// crates/lyng-js/vm/src/vm/feedback.rs:2208
		if feedback.monomorphic_dense_fast.is_valid() =>
	ldr x8, [x8, #1136]
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1385
		let handler = self.keyed_property_dense_fast_handler(code, feedback_slot)?;
	cbz w8, LBB389_13
	stp x8, x11, [sp, #32]
	str x10, [sp, #24]
	mov w9, #4992
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1387
		let header = agent.objects().object_header(view, receiver)?;
	sub x8, x29, #160
		// crates/lyng-js/env/src/agent.rs:270
		&self.objects
	add x0, x22, x9
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1387
		let header = agent.objects().object_header(view, receiver)?;
	mov x1, x22
	mov x2, x25
	bl lyng_js_objects::runtime::ObjectRuntime::object_header
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2666
		match self {
	ldurb w8, [x29, #-138]
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1387
		let header = agent.objects().object_header(view, receiver)?;
	cmp w8, #3
	ldp x10, x11, [sp, #32]
	b.eq LBB389_13
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2365
		match (self, other) {
	cbz w10, LBB389_13
	ldur w8, [x29, #-156]
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1388
		if handler.receiver_shape() != Some(header.shape())
	cmp w8, w10
	b.ne LBB389_13
	ldur w8, [x29, #-144]
	ldurh w9, [x29, #-140]
	lsr x10, x10, #32
	cmp w9, w10, uxth
	ccmp w8, #0, #4, eq
	b.eq LBB389_13
		// crates/lyng-js/gc/src/arena.rs:812
		self.object_slots.get(id)
	ldr x9, [x22, #640]
		// crates/lyng-js/gc/src/arena/storage.rs:860
		let slot = self.slots.get((id.get() - 1) as usize)?;
	sub w8, w8, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x9, x8
	b.ls LBB389_13
		// crates/lyng-js/gc/src/arena.rs:812
		self.object_slots.get(id)
	ldr x9, [x22, #632]
	mov w10, #24
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x8, w8, w10, x9
		// crates/lyng-js/gc/src/arena/storage.rs:861
		if slot.occupied {
	ldrb w9, [x8, #19]
	cmp w9, #1
	b.ne LBB389_13
		// crates/lyng-js/gc/src/arena/storage.rs:862
		Some(&slot.values)
	ldr x10, [x8, #8]
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1396
		.get(index as usize)
	and x9, x26, #0x7fffffff
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2058
		match self {
	cmp x10, x9
	b.ls LBB389_13
		// crates/lyng-js/gc/src/arena/storage.rs:862
		Some(&slot.values)
	ldr x8, [x8]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2059
		Some(&v) => Some(v),
	ldr x9, [x8, x9, lsl #3]
	mov x8, #1
	movk x8, #9, lsl #32
	movk x8, #32760, lsl #48
	str x9, [sp, #32]
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1394
		let value = view
	cmp x9, x8
	b.eq LBB389_13
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1631
		unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
	ldr x8, [x21, #104]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:229
		if self < slice.len() {
	cmp x8, x11
	b.ls LBB389_120
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x8, [x21, #96]
	ldr x9, [sp, #40]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:231
		unsafe { Some(slice_get_unchecked(slice, self)) }
	add x8, x8, x9, lsl #5
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1631
		unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
	ldr x9, [x8, #16]
	ldr x10, [sp, #24]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:229
		if self < slice.len() {
	cmp x9, x10
	b.ls LBB389_120
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x8, [x8, #8]
	mov w9, #1160
	ldr x10, [sp, #24]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:231
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x0, w10, w9, x8
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:767
		match *self {
	ldr x8, [x0]
	cmp x8, #10
	b.eq LBB389_120
		// crates/lyng-js/vm/src/vm/feedback.rs:2133
		site.record_execution();
	bl lyng_js_vm::vm::feedback::FeedbackSiteState::record_execution
LBB389_120:
		// crates/lyng-js/vm/src/vm/feedback.rs:2136
		self.observe_tier_feedback_event(code);
	ldp x0, x1, [x21, #120]
	mov x2, x27
	bl lyng_js_vm::vm::tiering::<impl lyng_js_vm::vm::Vm>::observe_tier_feedback_event
		// crates/lyng-js/vm/src/frame.rs:440
		self.metadata.registers
	ldr w8, [x20, #20]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1631
		unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
	ldr x1, [x21, #32]
	ldr w9, [sp, #84]
		// crates/lyng-js/vm/src/vm/registers.rs:160
		let absolute = registers.base() + u32::from(register);
	add w0, w8, w9
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:278
		&mut (*slice)[self]
	cmp x1, x0
	b.ls LBB389_198
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x8, [x21, #24]
	ldr x9, [sp, #32]
		// crates/lyng-js/vm/src/vm/registers.rs:28
		self.register_stack[absolute] = value;
	str x9, [x8, x0, lsl #3]
		// crates/lyng-js/vm/src/frame.rs:425
		self.state.instruction_offset
	ldr w8, [x20, #56]
	ldr w9, [sp, #80]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:724
		if intrinsics::unlikely(intrinsics::add_with_overflow(self, rhs).1) {
	adds w8, w8, w9
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:457
		if b {
	b.lo LBB389_17
	b LBB389_133
LBB389_122:
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:502
		self.handle_dispatch_result(agent, frame_depth, frame, result)?
	sub x0, x29, #160
	add x5, sp, #224
	mov x1, x21
	mov x2, x22
	ldr x3, [sp, #88]
	mov x4, x20
	bl lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::handle_dispatch_result
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2173
		match self {
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_158
LBB389_124:
	tbnz w5, #0, LBB389_130
LBB389_125:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:445
		unsafe { &*self.as_ptr().cast_const() }
	ldr x8, [x20, #128]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x8, [x8, #48]
		// crates/lyng-js/vm/src/frame.rs:425
		self.state.instruction_offset
	ldr w9, [x20, #56]
		// crates/lyng-js/vm/src/vm/dispatch_state.rs:100
		unsafe { *bytes.as_ptr().add(pc) }
	ldrb w8, [x8, x9]
		// crates/lyng-js/vm/src/vm/dispatch_state.rs:244
		$crate::vm::dispatch_state::DISPATCH_TABLE[byte as usize],
Lloh1083:
	adrp x9, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGE
Lloh1084:
	add x9, x9, lyng_js_vm::vm::dispatch_state::DISPATCH_TABLE@PAGEOFF
	ldr x10, [x9, x8, lsl #3]
		// crates/lyng-js/vm/src/vm/dispatch_state.rs:243
		return $crate::vm::dispatch_state::Step::Continue(
	mov x8, #33
	movk x8, #32768, lsl #48
	stp x8, x10, [x19]
LBB389_126:
		// crates/lyng-js/vm/src/vm/dispatch_handlers/property.rs:157
		}
	add sp, sp, #592
	ldp x29, x30, [sp, #80]
	ldp x20, x19, [sp, #64]
	ldp x22, x21, [sp, #48]
	ldp x24, x23, [sp, #32]
	ldp x26, x25, [sp, #16]
	ldp x28, x27, [sp], #96
	ret
LBB389_127:
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:511
		} else if let Some(value) = Self::try_fast_own_index_value(agent, object, index)? {
	sub x0, x29, #160
	mov x1, x22
	mov x2, x25
	mov x3, x26
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::try_fast_own_index_value
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2173
		match self {
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.ne LBB389_158
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:511
		} else if let Some(value) = Self::try_fast_own_index_value(agent, object, index)? {
	tbz w5, #0, LBB389_155
LBB389_130:
		// crates/lyng-js/vm/src/frame.rs:420
		self.metadata.code
	ldr w2, [x20, #4]
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:523
		self.observe_keyed_index_access(agent, frame.code(), feedback_slot, object, index);
	mov x0, x21
	mov x1, x22
	mov x3, x23
	mov x4, x25
	mov x5, x26
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::observe_keyed_index_access
LBB389_131:
		// crates/lyng-js/vm/src/frame.rs:440
		self.metadata.registers
	ldr w8, [x20, #20]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1631
		unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
	ldr x1, [x21, #32]
	ldr w9, [sp, #84]
		// crates/lyng-js/vm/src/vm/registers.rs:160
		let absolute = registers.base() + u32::from(register);
	add w0, w8, w9
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:278
		&mut (*slice)[self]
	cmp x1, x0
	b.ls LBB389_198
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x8, [x21, #24]
		// crates/lyng-js/vm/src/vm/registers.rs:28
		self.register_stack[absolute] = value;
	str x27, [x8, x0, lsl #3]
		// crates/lyng-js/vm/src/frame.rs:425
		self.state.instruction_offset
	ldr w8, [x20, #56]
	ldr w9, [sp, #80]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:724
		if intrinsics::unlikely(intrinsics::add_with_overflow(self, rhs).1) {
	adds w8, w8, w9
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:457
		if b {
	b.lo LBB389_17
LBB389_133:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:971
		None => expect_failed(msg),
Lloh1085:
	adrp x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGE
Lloh1086:
	add x0, x0, l_anon.10973c97f4c1e8e1c8050bb28bd48097.36@PAGEOFF
Lloh1087:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGE
Lloh1088:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.40@PAGEOFF
	mov w1, #41
	bl core::option::expect_failed
	b LBB389_197
LBB389_134:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:569
		slice_index_fail(self.start, slice.len(), slice.len())
Lloh1089:
	adrp x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGE
Lloh1090:
	add x3, x3, l_anon.10973c97f4c1e8e1c8050bb28bd48097.47@PAGEOFF
	mov x0, x5
	mov x2, x1
	bl core::slice::index::slice_index_fail
	b LBB389_197
LBB389_135:
		// crates/lyng-js/vm/src/vm/dispatch.rs:237
		return decode_abc_operands_wide(bytes, is_profiled, code, instruction_offset);
	sub x0, x29, #160
	mov w3, #1
	mov x4, x27
	bl lyng_js_vm::vm::dispatch::decode_abc_operands_wide
		// crates/lyng-js/vm/src/vm/dispatch_handlers/property.rs:125
		let (a, b, c, feedback_slot, instruction_len) = try_step!(decode_abc_operands(
	ldur x8, [x29, #-160]
		// crates/lyng-js/vm/src/vm/dispatch_state.rs:301
		match $e {
	cmp x8, x21
	b.ne LBB389_19
		// crates/lyng-js/vm/src/vm/dispatch_state.rs:302
		Ok(v) => v,
	ldurh w11, [x29, #-148]
	ldurh w8, [x29, #-146]
	ldurh w9, [x29, #-144]
	ldur w23, [x29, #-152]
	ldur w10, [x29, #-140]
	stp w10, w11, [sp, #80]
		// crates/lyng-js/vm/src/vm/dispatch_handlers/property.rs:142
		vm.execute_get_keyed_property_opcode(
	ldr x21, [x20, #80]
		// crates/lyng-js/vm/src/frame.rs:440
		self.metadata.registers
	ldr w10, [x20, #20]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1599
		unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
	ldr x1, [x21, #32]
		// crates/lyng-js/vm/src/vm/registers.rs:160
		let absolute = registers.base() + u32::from(register);
	add w0, w10, w8
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:272
		&(*slice)[self]
	cmp x1, x0
	b.hi LBB389_5
LBB389_137:
Lloh1091:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.61@PAGE
Lloh1092:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.61@PAGEOFF
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:278
		&mut (*slice)[self]
	b LBB389_196
LBB389_138:
		// crates/lyng-js/vm/src/vm/feedback.rs:2208
		if feedback.monomorphic_dense_fast.is_valid() =>
	ldr x8, [x8, #1136]
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1385
		let handler = self.keyed_property_dense_fast_handler(code, feedback_slot)?;
	cbz w8, LBB389_80
	str x8, [sp, #40]
	mov w9, #4992
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1387
		let header = agent.objects().object_header(view, receiver)?;
	sub x8, x29, #160
		// crates/lyng-js/env/src/agent.rs:270
		&self.objects
	add x0, x22, x9
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1387
		let header = agent.objects().object_header(view, receiver)?;
	mov x1, x22
	mov x2, x25
	bl lyng_js_objects::runtime::ObjectRuntime::object_header
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2666
		match self {
	ldurb w8, [x29, #-138]
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1387
		let header = agent.objects().object_header(view, receiver)?;
	cmp w8, #3
	ldr x9, [sp, #40]
	b.eq LBB389_80
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2365
		match (self, other) {
	cbz w9, LBB389_80
	ldur w8, [x29, #-156]
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1388
		if handler.receiver_shape() != Some(header.shape())
	cmp w8, w9
	b.ne LBB389_80
	ldurh w8, [x29, #-140]
	lsr x9, x9, #32
	cmp w8, w9, uxth
	b.ne LBB389_80
	ldur w8, [x29, #-144]
	cbz w8, LBB389_80
		// crates/lyng-js/gc/src/arena.rs:812
		self.object_slots.get(id)
	ldr x9, [x22, #640]
		// crates/lyng-js/gc/src/arena/storage.rs:860
		let slot = self.slots.get((id.get() - 1) as usize)?;
	sub w8, w8, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x9, x8
	b.ls LBB389_80
		// crates/lyng-js/gc/src/arena.rs:812
		self.object_slots.get(id)
	ldr x9, [x22, #632]
	mov w10, #24
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x8, w8, w10, x9
		// crates/lyng-js/gc/src/arena/storage.rs:861
		if slot.occupied {
	ldrb w9, [x8, #19]
	cmp w9, #1
	b.ne LBB389_80
		// crates/lyng-js/gc/src/arena/storage.rs:862
		Some(&slot.values)
	ldr x10, [x8, #8]
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1396
		.get(index as usize)
	mov w9, w26
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2058
		match self {
	cmp x10, x9
	b.ls LBB389_80
		// crates/lyng-js/gc/src/arena/storage.rs:862
		Some(&slot.values)
	ldr x8, [x8]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2059
		Some(&v) => Some(v),
	ldr x9, [x8, x9, lsl #3]
	mov x8, #1
	movk x8, #9, lsl #32
	movk x8, #32760, lsl #48
	str x9, [sp, #40]
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1394
		let value = view
	cmp x9, x8
	b.eq LBB389_80
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1631
		unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
	ldr x8, [x21, #104]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:229
		if self < slice.len() {
	cmp x8, x28
	b.ls LBB389_153
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x8, [x21, #96]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:231
		unsafe { Some(slice_get_unchecked(slice, self)) }
	add x8, x8, x28, lsl #5
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1631
		unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
	ldr x9, [x8, #16]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:229
		if self < slice.len() {
	cmp x9, x24
	b.ls LBB389_153
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x8, [x8, #8]
	mov w9, #1160
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:231
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x0, w24, w9, x8
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:767
		match *self {
	ldr x8, [x0]
	cmp x8, #10
	b.eq LBB389_153
		// crates/lyng-js/vm/src/vm/feedback.rs:2133
		site.record_execution();
	bl lyng_js_vm::vm::feedback::FeedbackSiteState::record_execution
LBB389_153:
		// crates/lyng-js/vm/src/vm/feedback.rs:2136
		self.observe_tier_feedback_event(code);
	ldp x0, x1, [x21, #120]
	mov x2, x27
	bl lyng_js_vm::vm::tiering::<impl lyng_js_vm::vm::Vm>::observe_tier_feedback_event
		// crates/lyng-js/vm/src/frame.rs:440
		self.metadata.registers
	ldr w8, [x20, #20]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1631
		unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
	ldr x1, [x21, #32]
	ldr w9, [sp, #84]
		// crates/lyng-js/vm/src/vm/registers.rs:160
		let absolute = registers.base() + u32::from(register);
	add w0, w8, w9
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:278
		&mut (*slice)[self]
	cmp x1, x0
	b.ls LBB389_198
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x8, [x21, #24]
	ldr x9, [sp, #40]
		// crates/lyng-js/vm/src/vm/registers.rs:28
		self.register_stack[absolute] = value;
	str x9, [x8, x0, lsl #3]
		// crates/lyng-js/vm/src/frame.rs:425
		self.state.instruction_offset
	ldr w8, [x20, #56]
	ldr w9, [sp, #80]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:724
		if intrinsics::unlikely(intrinsics::add_with_overflow(self, rhs).1) {
	adds w8, w8, w9
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:457
		if b {
	b.lo LBB389_17
	b LBB389_133
LBB389_155:
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:515
		self.get_property_from_value(agent, host, registry, frame, receiver, key);
	stp wzr, w26, [sp, #8]
	add x0, sp, #272
	str x25, [sp]
	mov x1, x21
	mov x2, x22
	ldp x3, x4, [sp, #48]
	ldp x5, x6, [sp, #64]
	mov x7, x20
	bl lyng_js_vm::vm::property_access::<impl lyng_js_vm::vm::Vm>::get_property_from_value
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:517
		self.handle_dispatch_result(agent, frame_depth, frame, property_result)?
	sub x0, x29, #160
	add x5, sp, #272
	mov x1, x21
	mov x2, x22
	ldr x3, [sp, #88]
	mov x4, x20
	bl lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::handle_dispatch_result
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2173
		match self {
	ldp x8, x5, [x29, #-160]
	ldur x27, [x29, #-144]
	mov x9, #33
	movk x9, #32768, lsl #48
	cmp x8, x9
	b.eq LBB389_124
LBB389_158:
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2175
		Err(e) => ControlFlow::Break(Err(e)),
	sub x9, x29, #160
	ldur q0, [x9, #24]
	str q0, [sp, #192]
	ldur x9, [x29, #-120]
	str x9, [sp, #208]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:2189
		Err(e) => Err(From::from(e)),
	lsr w9, w5, #8
	lsr x10, x5, #32
	b LBB389_62
LBB389_159:
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1474
		if record.shape() != handler.receiver_shape()
	cbnz w14, LBB389_168
	cmp x12, x10
	b.ne LBB389_168
LBB389_161:
		// crates/lyng-js/objects/src/shapes.rs:263
		let offset = low & HANDLER_SLOT_OFFSET_MASK;
	and x10, x9, #0x3fffffff
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1479
		let value = match handler.slot_location() {
	tbnz w9, #31, LBB389_167
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1482
		.object_slots(record.named_slots()?)?
	cbz w11, LBB389_168
		// crates/lyng-js/gc/src/arena.rs:812
		self.object_slots.get(id)
	ldr x12, [x22, #640]
		// crates/lyng-js/gc/src/arena/storage.rs:860
		let slot = self.slots.get((id.get() - 1) as usize)?;
	sub w9, w11, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x12, x9
	b.ls LBB389_168
		// crates/lyng-js/gc/src/arena.rs:812
		self.object_slots.get(id)
	ldr x11, [x22, #632]
	mov w12, #24
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x9, w9, w12, x11
		// crates/lyng-js/gc/src/arena/storage.rs:861
		if slot.occupied {
	ldrb w11, [x9, #19]
	cmp w11, #1
	b.ne LBB389_168
		// crates/lyng-js/gc/src/arena/storage.rs:862
		Some(&slot.values)
	ldr x11, [x9, #8]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2058
		match self {
	cmp x10, x11
	b.hs LBB389_168
		// crates/lyng-js/gc/src/arena/storage.rs:862
		Some(&slot.values)
	ldr x8, [x9]
	b LBB389_181
LBB389_167:
		// crates/lyng-js/gc/src/arena/records.rs:396
		if index < RUNTIME_OBJECT_INLINE_SLOT_COUNT {
	cmp x10, #3
	b.ls LBB389_180
LBB389_168:
		// crates/lyng-js/vm/src/vm/feedback.rs:2183
		if feedback.monomorphic_named_fast_atom == atom.raw()
	ldr w9, [x8, #1148]
	cmp w9, w26
	b.ne LBB389_98
		// crates/lyng-js/vm/src/vm/feedback.rs:2184
		&& feedback.monomorphic_named_proto_fast.is_valid() =>
	ldr x12, [x8, #1104]
	cbz x12, LBB389_98
		// crates/lyng-js/gc/src/arena.rs:606
		self.objects.get_ref(id)
	ldr x10, [x22, #224]
	mov w9, #-1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:882
		Some(unsafe { intrinsics::unchecked_sub(self, rhs) })
	add x9, x25, x9
		// crates/lyng-js/gc/src/arena/storage.rs:1020
		raw / PRIMITIVE_SLOTS_PER_PAGE,
	lsr w13, w9, #6
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x13, x10
	b.hs LBB389_98
		// crates/lyng-js/gc/src/arena.rs:606
		self.objects.get_ref(id)
	ldr x11, [x22, #216]
		// crates/lyng-js/gc/src/arena/storage.rs:1021
		raw % PRIMITIVE_SLOTS_PER_PAGE,
	and x9, x9, #0x3f
		// crates/lyng-js/gc/src/arena/storage.rs:239
		self.pages.get(page_index)?.get_ref(slot_index)
	ldr x13, [x11, x13, lsl #3]
	mov w14, #80
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x15, w9, w14, x13
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:745
		match *self {
	ldr w9, [x15]
	cmp w9, #1
	b.ne LBB389_98
	ldr x9, [x8, #1112]
	ldr x14, [x8, #1120]
	ldr x8, [x8, #1128]
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1552
		if record.shape() != handler.receiver_shape()
	ldp w13, w16, [x15, #48]
	ldr x15, [x15, #40]
	cmp w13, #0
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2365
		match (self, other) {
	cbz w16, LBB389_174
	ccmp w16, w12, #0, ne
	b LBB389_175
LBB389_174:
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1552
		if record.shape() != handler.receiver_shape()
	ccmp w12, #0, #0, ne
LBB389_175:
	ccmp x15, x14, #0, eq
	b.ne LBB389_98
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:882
		Some(unsafe { intrinsics::unchecked_sub(self, rhs) })
	sub w12, w13, #1
		// crates/lyng-js/gc/src/arena/storage.rs:1020
		raw / PRIMITIVE_SLOTS_PER_PAGE,
	lsr x13, x12, #6
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x13, x10
	b.hs LBB389_98
		// crates/lyng-js/gc/src/arena/storage.rs:1021
		raw % PRIMITIVE_SLOTS_PER_PAGE,
	and x10, x12, #0x3f
		// crates/lyng-js/gc/src/arena/storage.rs:239
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
	b.ne LBB389_98
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1559
		if prototype_record.shape() != handler.prototype_shape()
	ldp w12, w10, [x13, #52]
	ldr x11, [x13, #40]
	ldur q0, [x13, #8]
	stur q0, [x29, #-160]
	ldur q0, [x13, #24]
	stur q0, [x29, #-144]
		// crates/lyng-js/types/src/ids.rs:21
		match NonZeroU32::new(raw) {
	lsr x13, x9, #32
	cmp x13, #0
	csel w13, w13, wzr, ne
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2365
		match (self, other) {
	cbz w12, LBB389_184
	cmp w12, w13
	ccmp x11, x8, #0, eq
	b.ne LBB389_98
	b LBB389_186
LBB389_180:
	sub x8, x29, #160
LBB389_181:
	ldr x22, [x8, x10, lsl #3]
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1486
		self.record_feedback_slot(code, feedback_slot);
	mov x0, x21
	mov x1, x3
	mov x2, x23
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::record_feedback_slot
LBB389_182:
		// crates/lyng-js/vm/src/frame.rs:440
		self.metadata.registers
	ldr w8, [x20, #20]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1631
		unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
	ldr x1, [x21, #32]
	ldr w9, [sp, #84]
		// crates/lyng-js/vm/src/vm/registers.rs:160
		let absolute = registers.base() + u32::from(register);
	add w0, w8, w9
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:278
		&mut (*slice)[self]
	cmp x1, x0
	b.ls LBB389_198
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:504
		self.ptr.cast().as_non_null_ptr()
	ldr x8, [x21, #24]
		// crates/lyng-js/vm/src/vm/registers.rs:28
		self.register_stack[absolute] = value;
	str x22, [x8, x0, lsl #3]
		// crates/lyng-js/vm/src/frame.rs:425
		self.state.instruction_offset
	ldr w8, [x20, #56]
	ldr w9, [sp, #80]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:724
		if intrinsics::unlikely(intrinsics::add_with_overflow(self, rhs).1) {
	adds w8, w8, w9
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:457
		if b {
	b.lo LBB389_17
	b LBB389_133
LBB389_184:
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1559
		if prototype_record.shape() != handler.prototype_shape()
	cbnz w13, LBB389_98
	cmp x11, x8
	b.ne LBB389_98
LBB389_186:
		// crates/lyng-js/objects/src/shapes.rs:401
		let offset = low & HANDLER_SLOT_OFFSET_MASK;
	and x8, x9, #0x3fffffff
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1564
		let value = match handler.slot_location() {
	tbnz w9, #31, LBB389_192
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1567
		.object_slots(prototype_record.named_slots()?)?
	cbz w10, LBB389_98
		// crates/lyng-js/gc/src/arena.rs:812
		self.object_slots.get(id)
	ldr x11, [x22, #640]
		// crates/lyng-js/gc/src/arena/storage.rs:860
		let slot = self.slots.get((id.get() - 1) as usize)?;
	sub w9, w10, #1
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:219
		if self < slice.len() {
	cmp x11, x9
	b.ls LBB389_98
		// crates/lyng-js/gc/src/arena.rs:812
		self.object_slots.get(id)
	ldr x10, [x22, #632]
	mov w11, #24
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:221
		unsafe { Some(slice_get_unchecked(slice, self)) }
	umaddl x9, w9, w11, x10
		// crates/lyng-js/gc/src/arena/storage.rs:861
		if slot.occupied {
	ldrb w10, [x9, #19]
	cmp w10, #1
	b.ne LBB389_98
		// crates/lyng-js/gc/src/arena/storage.rs:862
		Some(&slot.values)
	ldr x10, [x9, #8]
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:2058
		match self {
	cmp x8, x10
	b.hs LBB389_98
		// crates/lyng-js/gc/src/arena/storage.rs:862
		Some(&slot.values)
	ldr x9, [x9]
	b LBB389_194
LBB389_192:
		// crates/lyng-js/gc/src/arena/records.rs:396
		if index < RUNTIME_OBJECT_INLINE_SLOT_COUNT {
	cmp x8, #3
	b.hi LBB389_98
	sub x9, x29, #160
LBB389_194:
	ldr x22, [x9, x8, lsl #3]
		// crates/lyng-js/vm/src/vm/dispatch/property.rs:1571
		self.record_feedback_slot(code, feedback_slot);
	mov x0, x21
	mov x1, x3
	mov x2, x23
	bl lyng_js_vm::vm::feedback::<impl lyng_js_vm::vm::Vm>::record_feedback_slot
	b LBB389_182
LBB389_195:
Lloh1093:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.61@PAGE
Lloh1094:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.61@PAGEOFF
	mov x0, x8
LBB389_196:
	bl core::panicking::panic_bounds_check
LBB389_197:
	brk #0x1
LBB389_198:
Lloh1095:
	adrp x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.38@PAGE
Lloh1096:
	add x2, x2, l_anon.10973c97f4c1e8e1c8050bb28bd48097.38@PAGEOFF
		// ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:278
		&mut (*slice)[self]
	b LBB389_196
		// crates/lyng-js/vm/src/vm/dispatch_handlers/property.rs:121
		pub extern "C" fn op_get_keyed_property(state: &mut DispatchState) -> Step {
	bl core::panicking::panic_cannot_unwind
