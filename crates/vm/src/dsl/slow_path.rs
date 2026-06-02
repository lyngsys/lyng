//! Slow-path bridge: semantic-outcome type, `LlIntDispatchState` wrapper,
//! and `SlowPathReturn` ABI.

use lyng_types::{CodeRef, FeedbackSlotId, Value};

use crate::error::{VmError, VmResult};

/// Logical outcome of a semantic-body invocation. Cold-stub shims map
/// this to `SlowPathReturn`; the alpha path maps it to `Step`.
#[derive(Debug)]
pub enum SemanticOutcome {
    /// Dispatch continues at the post-instruction PC. `pc_advance` is
    /// the number of bytes the semantic body consumed (i.e. the
    /// instruction length when execution did not branch, or the absolute
    /// target offset minus the entry PC when the body performed a jump).
    Continue { pc_advance: u32 },
    /// Frame changed (call / return / cross-frame catch). The dispatcher
    /// must reload pinned PC/REGS/FV from the canonical frame state.
    Refresh,
    /// Successful program completion; `Vm::run` returns `Ok(value)`.
    ExitDone { value: Value },
    /// Abrupt completion that escapes the current `Vm::run`; the bridge
    /// returns `Err(error)`.
    ExitError { error: VmError },
}

use crate::dsl::llint_state::{LlIntRustContext, LlIntState};
use crate::vm::dispatch_state::DispatchState;

/// Safe wrapper around a per-frame dispatch state. Semantic bodies see
/// the same method signatures regardless of whether the caller is the
/// alpha path or the asm shim path.
pub struct LlIntDispatchState<'vm, 'borrow> {
    pub(crate) inner: LlIntDispatchInner<'vm, 'borrow>,
}

pub(crate) enum LlIntDispatchInner<'vm, 'borrow> {
    /// Borrowed from a live `DispatchState` (alpha path).
    Alpha(&'borrow mut DispatchState<'vm>),
    /// Reconstructed from a `*mut LlIntState` passed by the asm trampoline.
    Asm {
        state: *mut LlIntState,
        rust: &'borrow mut LlIntRustContext<'vm>,
    },
}

impl<'vm, 'borrow> LlIntDispatchState<'vm, 'borrow> {
    /// Construct from an alpha `DispatchState`.
    pub const fn from_alpha(state: &'borrow mut DispatchState<'vm>) -> Self {
        Self {
            inner: LlIntDispatchInner::Alpha(state),
        }
    }

    /// Construct from a raw `*mut LlIntState` passed by the asm shim.
    ///
    /// # Safety
    ///
    /// Caller (the asm bridge) guarantees:
    /// - `state` is a valid `*mut LlIntState` for the lifetime of the
    ///   slow-path call.
    /// - `state.rust_context` was established by the entry shim
    ///   (`Vm::run_via_dsl`) and points to a valid
    ///   `LlIntRustContext<'vm>` whose `'vm` outlives `'borrow`.
    /// - No other live `&mut LlIntRustContext` aliases the same
    ///   pointer for the duration of the returned wrapper.
    pub unsafe fn from_raw(state: *mut LlIntState) -> Self {
        let rust = unsafe { &mut *(*state).rust_context.cast::<LlIntRustContext<'vm>>() };
        Self {
            inner: LlIntDispatchInner::Asm { state, rust },
        }
    }

    /// Mutable access to the underlying `DispatchState`.
    pub const fn dispatch_state(&mut self) -> &mut DispatchState<'vm> {
        match &mut self.inner {
            LlIntDispatchInner::Alpha(state) => state,
            LlIntDispatchInner::Asm { rust, .. } => &mut rust.dispatch,
        }
    }

    /// Current instruction offset. After `sync_from_asm`, `state.frame_pc_offset`
    /// and `rust.dispatch.pc` are in sync so reading either is correct.
    #[inline]
    pub const fn current_instruction_offset(&self) -> u32 {
        match &self.inner {
            LlIntDispatchInner::Alpha(state) => state.pc,
            LlIntDispatchInner::Asm { rust, .. } => rust.dispatch.pc,
        }
    }

    /// Copy asm-side mirrors into the Rust-side state before a semantic body
    /// runs. Idempotent on the alpha variant.
    pub fn sync_from_asm(&mut self) {
        if let LlIntDispatchInner::Asm { state, rust } = &mut self.inner {
            // Refresh the live PC from the asm-side mirror before any semantic
            // body observes it. Header fields come straight from the overlay.
            // SAFETY: `state` is valid by `from_raw`'s contract; scalar read.
            rust.dispatch.pc = unsafe { (**state).frame_pc_offset };
        }
    }

    /// Translate a [`SemanticOutcome`] into the asm-facing [`SlowPathReturn`].
    /// On the alpha variant the return value is unused (the alpha path reads
    /// the outcome via `translate_outcome_to_step` instead).
    #[allow(
        clippy::too_many_lines,
        reason = "outcome translation is one state machine over all egress modes; splitting would hide the shared refresh/exit invariants"
    )]
    pub fn translate_outcome(&mut self, outcome: SemanticOutcome) -> SlowPathReturn {
        // Refresh the global-IC generation mirror on every slow-stub egress.
        // Any global structural mutation inside a slow path bumps the agent's
        // `global_structure_generation`; the asm `LoadGlobal` mode-7 hit guards
        // against a stale mirror, so it must be current before asm resumes.
        // Refreshing unconditionally here covers all bump sites at one choke point.
        if let LlIntDispatchInner::Asm { state: _, rust } = &mut self.inner {
            // Use the active frame's realm, not the cached entry-realm: after a
            // cross-realm Call the callee frame is already on `vm.frames()`, so
            // the mirror must track that frame's realm's generation.
            if let Some(realm) = rust.dispatch.vm.current_realm_of(rust.dispatch.agent) {
                rust.dispatch
                    .vm
                    .refresh_global_ic_generation_for_realm(rust.dispatch.agent, realm);
            }
        }
        match outcome {
            SemanticOutcome::Continue { pc_advance } => {
                // Cross-frame catch check: if a throw was caught in a higher frame,
                // `request_dispatch_frame_check` bumps the epoch. Promote `Continue`
                // to `Refresh` so the bridge reloads PC/REGS/MT for the catch frame.
                // (The alpha path checks this between every handler call.)
                if let LlIntDispatchInner::Asm { state: _, rust } = &mut self.inner {
                    let vm_epoch = rust.dispatch.vm.dispatch_frame_check_epoch_for_dsl();
                    if rust.dispatch.frame_check_epoch != vm_epoch
                        && rust.dispatch.vm.frame_depth() != rust.dispatch.frame_depth
                    {
                        // Cross-frame catch — reload frame state via Refresh.
                        return self.translate_outcome(SemanticOutcome::Refresh);
                    }
                    rust.dispatch.frame_check_epoch = vm_epoch;
                }
                // Refresh REGS from the live storage pointer on every Continue
                // egress so the bridge re-derives the active frame's window base
                // after any frame-depth change. The arena never reallocates, so
                // the pointer is stable; we recompute to pick up any window shift.
                // PC stays sourced from `rust.dispatch.pc` (handler-local advances
                // are not yet synced to the overlay).
                let mut new_offset_u64: u64 = 0;
                if let LlIntDispatchInner::Asm { state, rust } = &mut self.inner {
                    // Same-frame egress: source PC from the thin view, which is kept
                    // current by all Continue-returning paths. Advance by `pc_advance`.
                    let new_offset = rust.dispatch.pc.wrapping_add(pc_advance);
                    new_offset_u64 = u64::from(new_offset);
                    rust.dispatch.pc = new_offset;
                    let regs_base_ptr = {
                        // Window base = cfr + HEADER_SLOTS.
                        let base =
                            (rust.dispatch.cfr + crate::frame_header::HEADER_SLOTS as u32) as usize;
                        // SAFETY: window is reserved on the arena; one-past-the-end is valid.
                        unsafe { rust.dispatch.vm.register_stack_storage_mut_ptr().add(base) }
                    };
                    let mt_base: *mut u8 = {
                        let index = crate::vm::code_index_for_dsl(rust.dispatch.code_ref);
                        rust.dispatch
                            .vm
                            .metadata_tables
                            .get(index)
                            .and_then(|t| t.as_ref())
                            .map_or(std::ptr::null_mut(), |t| t.buffer_ptr().cast_mut())
                    };
                    let object_records_base =
                        rust.dispatch.agent.heap().view().object_record_ptr_table();
                    let object_slots_base =
                        rust.dispatch.agent.heap().view().object_slots_ptr_table();
                    let value_cells_base = rust
                        .dispatch
                        .agent
                        .heap()
                        .view()
                        .value_cell_ptr_table_base();
                    // SAFETY: state is valid by `from_raw`'s contract; unique borrow via `self`.
                    // Mirror the new PC into `state.frame_pc_offset` and refresh REGS/MT.
                    unsafe {
                        (**state).frame_pc_offset = new_offset;
                        (**state).frame_regs_base = regs_base_ptr;
                        (**state).frame_metadata_table_base = mt_base;
                        (**state).object_records_base = object_records_base;
                        (**state).object_slots_base = object_slots_base;
                        (**state).value_cells_base = value_cells_base;
                    }
                    rust.dispatch.refresh_dsl_poll_pending();
                }
                // `dispatch_after_slow!` reads the new PC from `payload` directly
                // to avoid a memory round-trip on the Continue arm.
                SlowPathReturn {
                    tag: SlowPathTag::Continue as u64,
                    payload: new_offset_u64,
                }
            }
            SemanticOutcome::Refresh => {
                if let LlIntDispatchInner::Asm { state, rust } = &mut self.inner {
                    // Frame-switch reload: source all asm-visible fields from the
                    // per-frame header overlay. Covers call/return (depth change),
                    // cross-frame catch (depth decreased; saved_pc rewritten by
                    // `transfer_to_exception_handler`), and same-frame catch
                    // (saved_pc rewritten in place). Reading `saved_pc` here is
                    // correct — unlike Continue which uses the live thin-view PC.
                    let current_depth = rust.dispatch.vm.frame_depth();
                    if let Some(cfr) = rust.dispatch.vm.current_cfr_opt() {
                        let code = rust.dispatch.vm.frame_header(cfr).code();
                        let regs_len = rust.dispatch.vm.frame_window_len(cfr);
                        let saved_pc = rust.dispatch.vm.frame_header(cfr).saved_pc();
                        let installed = rust
                            .dispatch
                            .vm
                            .installed_for_dsl_runtime(code)
                            .unwrap_or_else(|| rust.dispatch.installed.clone());
                        // Update the thin view.
                        rust.dispatch.cfr = cfr;
                        rust.dispatch.pc = saved_pc;
                        rust.dispatch.code_ref = code;
                        rust.dispatch.regs_len = regs_len;
                        rust.dispatch.frame_depth = current_depth;
                        rust.dispatch.installed = installed;
                        // Sync the frame-check epoch.
                        rust.dispatch.frame_check_epoch =
                            rust.dispatch.vm.dispatch_frame_check_epoch_for_dsl();
                        // Populate LlIntState mirrors from the overlay; window base = cfr + HEADER_SLOTS.
                        let regs_base_ptr = {
                            let base = (cfr + crate::frame_header::HEADER_SLOTS as u32) as usize;
                            // SAFETY: window is reserved on the active frame.
                            unsafe { rust.dispatch.vm.register_stack_storage_mut_ptr().add(base) }
                        };
                        let pb_base = rust
                            .dispatch
                            .installed
                            .function()
                            .instruction_bytes()
                            .as_ptr();
                        let mt_base: *mut u8 = {
                            let index = crate::vm::code_index_for_dsl(code);
                            rust.dispatch
                                .vm
                                .metadata_tables
                                .get(index)
                                .and_then(|t| t.as_ref())
                                .map_or(std::ptr::null_mut(), |t| t.buffer_ptr().cast_mut())
                        };
                        let object_records_base =
                            rust.dispatch.agent.heap().view().object_record_ptr_table();
                        let object_slots_base =
                            rust.dispatch.agent.heap().view().object_slots_ptr_table();
                        let value_cells_base = rust
                            .dispatch
                            .agent
                            .heap()
                            .view()
                            .value_cell_ptr_table_base();
                        // Derive per-frame fields; mirrors the entry shim in entry.rs::run_via_dsl.
                        let const_base: *const lyng_types::Value = rust
                            .dispatch
                            .agent
                            .heap()
                            .view()
                            .code(code)
                            .and_then(lyng_gc::RuntimeCodeRecord::constants)
                            .and_then(|slots| rust.dispatch.agent.heap().view().code_slots(slots))
                            .map_or(std::ptr::null(), <[lyng_types::Value]>::as_ptr);
                        // Refresh the `this` mirror from the overlay.
                        let this_value =
                            crate::dsl::llint_state::resolve_initial_this_value_from_header(
                                rust.dispatch.vm.frame_header(cfr),
                            );
                        // SAFETY: state is valid by `from_raw`'s contract.
                        unsafe {
                            (**state).frame_pc_offset = saved_pc;
                            (**state).frame_pb_base = pb_base;
                            (**state).frame_regs_base = regs_base_ptr;
                            (**state).frame_metadata_table_base = mt_base;
                            (**state).object_records_base = object_records_base;
                            (**state).object_slots_base = object_slots_base;
                            (**state).value_cells_base = value_cells_base;
                            (**state).frame_const_base = const_base;
                            (**state).frame_this_value = this_value;
                        }
                        // Debug: assert `frame_const_base` is stable across the
                        // slow-path call. If this fires, the arena moved unexpectedly.
                        #[cfg(debug_assertions)]
                        {
                            let recomputed: *const lyng_types::Value = rust
                                .dispatch
                                .agent
                                .heap()
                                .view()
                                .code(code)
                                .and_then(lyng_gc::RuntimeCodeRecord::constants)
                                .and_then(|slots| {
                                    rust.dispatch.agent.heap().view().code_slots(slots)
                                })
                                .map_or(std::ptr::null(), <[lyng_types::Value]>::as_ptr);
                            debug_assert_eq!(
                                const_base, recomputed,
                                "frame_const_base unstable across Refresh"
                            );
                        }
                        rust.dispatch.refresh_dsl_poll_pending();
                    }
                }
                SlowPathReturn {
                    tag: SlowPathTag::Refresh as u64,
                    payload: 0,
                }
            }
            SemanticOutcome::ExitDone { value } => {
                if let LlIntDispatchInner::Asm { state: _, rust } = &mut self.inner {
                    rust.exit.kind = crate::dsl::llint_state::ExitKind::Done;
                    rust.exit.done_value = value;
                }
                SlowPathReturn {
                    tag: SlowPathTag::Exit as u64,
                    payload: 0,
                }
            }
            SemanticOutcome::ExitError { error } => {
                if let LlIntDispatchInner::Asm { state: _, rust } = &mut self.inner {
                    rust.exit.kind = crate::dsl::llint_state::ExitKind::Error;
                    rust.exit.error = Some(Box::new(error));
                }
                SlowPathReturn {
                    tag: SlowPathTag::Exit as u64,
                    payload: 0,
                }
            }
        }
    }
}

/// Return ABI for cold-stub shims. The asm bridge dispatches on `tag`
/// (`Continue` / `Refresh` / `Exit`); `payload` carries the new PC
/// offset on `Continue`.
#[repr(C)]
pub struct SlowPathReturn {
    pub tag: u64,
    pub payload: u64,
}

/// Tag values for [`SlowPathReturn::tag`]. Part of the asm-DSL ABI.
#[repr(u64)]
pub enum SlowPathTag {
    Continue = 0,
    Refresh = 1,
    Exit = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NoDecodeAbxOperands {
    pub a: u16,
    pub bx: u32,
    pub instruction_len: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NoDecodeAbcOperands {
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub instruction_len: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NoDecodeAbcSlotOperands {
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub feedback_slot: FeedbackSlotId,
    pub instruction_len: u32,
}

#[inline]
pub(crate) fn decode_no_decode_abx_operands(
    bytes: &[u8],
    code: CodeRef,
    instruction_offset: u32,
) -> VmResult<NoDecodeAbxOperands> {
    let (a, bx, _feedback_slot, instruction_len) =
        crate::vm::dispatch::decode_abx_operands(bytes, None, false, code, instruction_offset)?;
    Ok(NoDecodeAbxOperands {
        a,
        bx,
        instruction_len,
    })
}

#[inline]
pub(crate) fn decode_no_decode_abc_operands(
    bytes: &[u8],
    code: CodeRef,
    instruction_offset: u32,
) -> VmResult<NoDecodeAbcOperands> {
    let (a, b, c, feedback_slot, instruction_len) =
        crate::vm::dispatch::decode_abc_operands(bytes, None, false, code, instruction_offset)?;
    debug_assert_eq!(feedback_slot, None);
    Ok(NoDecodeAbcOperands {
        a,
        b,
        c,
        instruction_len,
    })
}

#[inline]
pub(crate) fn decode_no_decode_abc_slot_operands(
    bytes: &[u8],
    code: CodeRef,
    instruction_offset: u32,
) -> VmResult<NoDecodeAbcSlotOperands> {
    let (a, b, c, feedback_slot, instruction_len) =
        crate::vm::dispatch::decode_abc_operands(bytes, None, true, code, instruction_offset)?;
    let feedback_slot = feedback_slot.ok_or(VmError::InstructionOutOfBounds {
        code,
        instruction_offset,
    })?;
    Ok(NoDecodeAbcSlotOperands {
        a,
        b,
        c,
        feedback_slot,
        instruction_len,
    })
}

impl LlIntDispatchState<'_, '_> {
    #[inline]
    pub(crate) fn decode_current_abx_operands(&mut self) -> VmResult<NoDecodeAbxOperands> {
        let inner = self.dispatch_state();
        let pc = inner.pc();
        let code = inner.code();
        let bytes = inner
            .installed
            .function()
            .instruction_bytes()
            .get(pc as usize..)
            .ok_or(VmError::InstructionOutOfBounds {
                code,
                instruction_offset: pc,
            })?;
        decode_no_decode_abx_operands(bytes, code, pc)
    }

    #[inline]
    pub(crate) fn decode_current_abc_operands(&mut self) -> VmResult<NoDecodeAbcOperands> {
        let inner = self.dispatch_state();
        let pc = inner.pc();
        let code = inner.code();
        let bytes = inner
            .installed
            .function()
            .instruction_bytes()
            .get(pc as usize..)
            .ok_or(VmError::InstructionOutOfBounds {
                code,
                instruction_offset: pc,
            })?;
        decode_no_decode_abc_operands(bytes, code, pc)
    }

    #[inline]
    pub(crate) fn decode_current_abc_slot_operands(&mut self) -> VmResult<NoDecodeAbcSlotOperands> {
        let inner = self.dispatch_state();
        let pc = inner.pc();
        let code = inner.code();
        let bytes = inner
            .installed
            .function()
            .instruction_bytes()
            .get(pc as usize..)
            .ok_or(VmError::InstructionOutOfBounds {
                code,
                instruction_offset: pc,
            })?;
        decode_no_decode_abc_slot_operands(bytes, code, pc)
    }
}

/// Generate an asm-facing cold-stub shim. Emits a `#[no_mangle] pub extern "C" fn`
/// that reconstructs `LlIntDispatchState`, calls the semantic body, and
/// translates the outcome to `SlowPathReturn`.
///
/// Example:
/// ```ignore
/// dsl_cold_shim! {
///     op_load_constant_slow_rs,
///     semantic: op_load_constant_semantic,
///     args: OpLoadConstantArgs,
///     operands: { dst: u16, constant_index: u32 },
/// }
/// ```
#[macro_export]
macro_rules! dsl_cold_shim {
    (
        $shim_name:ident,
        semantic: $semantic:path,
        args: $args_ty:ty,
        operands: { $($field:ident: $field_ty:ty),* $(,)? } $(,)?
    ) => {
        #[no_mangle]
        pub extern "C" fn $shim_name(
            state: *mut $crate::dsl::llint_state::LlIntState,
            $($field: $field_ty),*
        ) -> $crate::dsl::slow_path::SlowPathReturn {
            // SAFETY: `state` is valid for this call; see `from_raw` contract.
            let mut dispatch = unsafe {
                $crate::dsl::slow_path::LlIntDispatchState::from_raw(state)
            };
            dispatch.sync_from_asm();
            let args = <$args_ty> { $($field),* };
            let outcome = $semantic(&mut dispatch, args);
            dispatch.translate_outcome(outcome)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_bytecode::Opcode;
    use lyng_types::{CodeRef, FeedbackSlotId};

    fn code_ref() -> CodeRef {
        CodeRef::from_raw(1).expect("non-zero code ref")
    }

    #[test]
    fn no_decode_abx_operands_decode_from_instruction_bytes() {
        let bytes = [Opcode::LoadSmi as u8, 7, 0x34, 0x12];

        let decoded = decode_no_decode_abx_operands(&bytes, code_ref(), 0)
            .expect("Abx no-decode operands should decode");

        assert_eq!(decoded.a, 7);
        assert_eq!(decoded.bx, 0x1234);
        assert_eq!(decoded.instruction_len, 4);
    }

    #[test]
    fn no_decode_abc_operands_decode_from_instruction_bytes() {
        let bytes = [Opcode::DefineNamedProperty as u8, 1, 2, 3];

        let decoded = decode_no_decode_abc_operands(&bytes, code_ref(), 0)
            .expect("Abc no-decode operands should decode");

        assert_eq!(decoded.a, 1);
        assert_eq!(decoded.b, 2);
        assert_eq!(decoded.c, 3);
        assert_eq!(decoded.instruction_len, 4);
    }

    #[test]
    fn no_decode_abc_slot_operands_decode_feedback_slot_from_instruction_bytes() {
        let slot = 11u16;
        let slot_bytes = slot.to_le_bytes();
        let bytes = [
            Opcode::SetNamedProperty as u8,
            4,
            5,
            6,
            slot_bytes[0],
            slot_bytes[1],
        ];

        let decoded = decode_no_decode_abc_slot_operands(&bytes, code_ref(), 0)
            .expect("AbcSlot no-decode operands should decode");

        assert_eq!(decoded.a, 4);
        assert_eq!(decoded.b, 5);
        assert_eq!(decoded.c, 6);
        assert_eq!(
            decoded.feedback_slot,
            FeedbackSlotId::from_raw(u32::from(slot)).unwrap()
        );
        assert_eq!(decoded.instruction_len, 6);
    }
}
