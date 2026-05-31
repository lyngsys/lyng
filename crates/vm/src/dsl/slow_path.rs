//! Slow-path bridge: semantic-outcome type + per-opcode argument structs
//! + (DSL-0b) the `LlIntDispatchState` wrapper and `SlowPathReturn` ABI.
//!
//! During DSL-0a only `SemanticOutcome`, the `OpXxxArgs` structs, and the
//! transitional `LlIntDispatchState` alias are populated. The asm-facing
//! shim layer and `SlowPathReturn`/`SlowPathTag` lands in DSL-0b.

use lyng_types::{CodeRef, FeedbackSlotId, Value};

use crate::error::{VmError, VmResult};

/// Logical outcome of a semantic-body invocation. The α handler maps
/// this to `Step`; the DSL cold-stub shim maps it to `SlowPathReturn`.
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

/// Safe wrapper around a per-frame dispatch state.
///
/// During DSL-0a this holds a `&mut DispatchState<'vm>` directly — the
/// asm bridge does not exist yet, so semantic bodies reach VM state via
/// the legacy `DispatchState` accessors re-exposed here. In DSL-0b the
/// wrapper is also reachable through `LlIntDispatchState::from_raw`,
/// which reconstructs it from a `*mut LlIntState` passed by the asm
/// shim. The semantic body sees identical method signatures in both
/// paths — that's the single-implementation invariant in action.
pub struct LlIntDispatchState<'vm, 'borrow> {
    pub(crate) inner: LlIntDispatchInner<'vm, 'borrow>,
}

pub(crate) enum LlIntDispatchInner<'vm, 'borrow> {
    /// Borrowed from a live `DispatchState` (alpha path, transitional).
    Alpha(&'borrow mut DispatchState<'vm>),
    /// Reconstructed from a raw `*mut LlIntState` passed by the asm
    /// trampoline + an opaque-cast `&mut LlIntRustContext<'vm>`. The
    /// asm side never reads through `state.rust_context`; the slow-path
    /// shim casts it back to the Rust context borrow.
    Asm {
        state: *mut LlIntState,
        rust: &'borrow mut LlIntRustContext<'vm>,
    },
}

impl<'vm, 'borrow> LlIntDispatchState<'vm, 'borrow> {
    /// Construct from a live α `DispatchState`. The α handler in
    /// `dispatch_handlers/` calls this to forward into `op_xxx_semantic`.
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

    /// Mutable access to the underlying `DispatchState`. Works on both
    /// dispatch variants: α handlers borrow the existing
    /// `DispatchState`; asm-path shims unpack the same shape from
    /// `LlIntRustContext::dispatch`. Semantic bodies under
    /// `crate::vm::semantics::` consume this uniformly.
    pub const fn dispatch_state(&mut self) -> &mut DispatchState<'vm> {
        match &mut self.inner {
            LlIntDispatchInner::Alpha(state) => state,
            LlIntDispatchInner::Asm { rust, .. } => &mut rust.dispatch,
        }
    }

    /// Typed accessor for the current instruction offset. Works on
    /// both α (legacy `DispatchState`) and asm-constructed dispatch
    /// states — design §6 invariant: after [`Self::sync_from_asm`]
    /// the asm side's `state.frame_pc_offset` and the Rust side's
    /// `rust.dispatch.frame.instruction_offset()` are in sync, so
    /// reading either is correct.
    ///
    /// Used by the DSL-0b validation cases (B32 PC-sync) and by
    /// callers that need PC inspection without going through
    /// `dispatch_state()`.
    #[inline]
    pub const fn current_instruction_offset(&self) -> u32 {
        match &self.inner {
            LlIntDispatchInner::Alpha(state) => state.frame.instruction_offset(),
            LlIntDispatchInner::Asm { rust, .. } => rust.dispatch.frame.instruction_offset(),
        }
    }

    /// Pre-slow-path sync — copy asm-side mirrors into the Rust-side
    /// snapshot before semantic code observes the frame. See design §6.
    ///
    /// Idempotent on the α variant (asm mirrors do not exist there;
    /// `rust.dispatch.frame` is already authoritative).
    pub fn sync_from_asm(&mut self) {
        if let LlIntDispatchInner::Asm { state, rust } = &mut self.inner {
            // SAFETY: `state` is valid by `from_raw`'s contract; we
            // only read scalar fields here.
            unsafe {
                rust.dispatch
                    .frame
                    .set_instruction_offset((**state).frame_pc_offset);
            }
            // registers_base is mirrored back via the `Refresh` path
            // in `translate_outcome`; semantic bodies read through
            // the register window, which is still authoritative on
            // entry (the asm side has not relocated it).
        }
    }

    /// Translate a [`SemanticOutcome`] into the asm-facing
    /// [`SlowPathReturn`]. Used by every asm-facing cold-stub shim.
    ///
    /// On the α variant this is a no-op (returns `Continue, 0`) — the
    /// alpha path uses `translate_outcome_to_step` in
    /// `dispatch_handlers/` and never calls this translator. Hitting
    /// the no-op branch is not an error; callers may invoke it
    /// uniformly across both variants.
    #[allow(
        clippy::too_many_lines,
        reason = "outcome translation is one state machine over all egress modes; splitting would hide the shared refresh/exit invariants"
    )]
    pub fn translate_outcome(&mut self, outcome: SemanticOutcome) -> SlowPathReturn {
        // Re-sync the asm-read global-IC generation mirror on EVERY slow-stub
        // egress, before asm dispatch can resume. A global structural mutation
        // (`delete globalThis.x`, `Object.defineProperty(globalThis, ...)`, sloppy
        // global creation) runs inside a slow path and bumps the agent's
        // `global_structure_generation`; the asm `LoadGlobal` mode-7 hit guards a
        // cached value-cell ref on `metadata.generation == this mirror`, so the
        // mirror MUST be current before the next possible hit. Refreshing here —
        // the single choke point all slow returns pass through before asm resumes
        // — covers all four bump sites uniformly. Cheap: one Vec index + store on
        // the cached global env (no env-chain walk), only on the cold path. We
        // refresh on all outcomes (incl. Exit) rather than only the
        // dispatch-resuming Continue/Refresh arms, so no bump path can slip
        // through — correctness over saving one branch.
        if let LlIntDispatchInner::Asm { state: _, rust } = &mut self.inner {
            // Derive the env from the ACTIVE frame's realm, not the cached
            // entry-realm env: a cross-realm Call egresses here with the callee
            // frame already pushed onto `vm.frames()`, so re-priming from that
            // frame's realm makes the mirror track the realm whose code is about
            // to resume — a mode-7 hit then compares against the correct realm's
            // generation. No active frame (program exit) → leave the mirror as-is.
            if let Some(realm) = rust.dispatch.vm.frames().last().map(|f| f.realm()) {
                rust.dispatch
                    .vm
                    .refresh_global_ic_generation_for_realm(rust.dispatch.agent, realm);
            }
        }
        match outcome {
            SemanticOutcome::Continue { pc_advance } => {
                // Epoch check — mirrors α's `run_trampoline` loop body
                // (see `vm::dispatch_state::run_trampoline`). When a
                // semantic body calls `handle_dispatch_result` and a
                // throw was caught in a higher frame (cross-frame
                // catch), the helper unwinds frames and bumps the
                // dispatch-frame-check epoch via
                // `request_dispatch_frame_check`. The caller frame's
                // `rust.dispatch.frame` is now stale (it's still the
                // pre-throw frame), but the active frame is whichever
                // frame caught the throw — its PC was rewritten by
                // `transfer_to_exception_handler`. We must promote
                // this `Continue` into a `Refresh` so the bridge
                // reloads PC/REGS/MT from `vm.frames().last()`.
                //
                // Under α, the trampoline loop does this check between
                // every handler call. Under DSL, the asm bridge does
                // NOT — so we do it here in the slow-path egress.
                if let LlIntDispatchInner::Asm { state: _, rust } = &mut self.inner {
                    let vm_epoch = rust.dispatch.vm.dispatch_frame_check_epoch_for_dsl();
                    if rust.dispatch.frame_check_epoch != vm_epoch
                        && rust.dispatch.vm.frames().len() != rust.dispatch.frame_depth
                    {
                        // Cross-frame catch — recurse into the Refresh
                        // arm so we share the frame-reload logic.
                        return self.translate_outcome(SemanticOutcome::Refresh);
                    }
                    rust.dispatch.frame_check_epoch = vm_epoch;
                }
                // REGS-pin refresh — the asm-side `x20` register pins
                // the active frame's register-stack base. When a slow
                // path triggers a nested call (e.g. `op_add_semantic`
                // → ToPrimitive → valueOf bytecode) and the nested
                // call's `reserve_register_window` reallocates the
                // underlying `Vec<Value>`, the old base pointer in
                // `x20` is freed — even when frame depth is unchanged
                // after the nested call returns. Recompute REGS from
                // the live `Vm::register_stack_storage_mut_ptr` on
                // every Continue egress so the asm bridge picks up
                // the post-reallocation base. PC stays sourced from
                // `rust.dispatch.frame` so handler-local PC advances
                // (which haven't been synced to `vm.frames`) are
                // preserved — matching α's `still_active` policy that
                // never clobbers PC on a same-frame epoch bump.
                let mut new_offset_u64: u64 = 0;
                if let LlIntDispatchInner::Asm { state, rust } = &mut self.inner {
                    let new_offset = rust
                        .dispatch
                        .frame
                        .instruction_offset()
                        .wrapping_add(pc_advance);
                    new_offset_u64 = u64::from(new_offset);
                    let active_frame = rust.dispatch.frame;
                    let regs_base_ptr = {
                        let base = active_frame.registers().base() as usize;
                        // SAFETY: register window is reserved on the
                        // active frame; one-past-the-end is well-defined.
                        unsafe { rust.dispatch.vm.register_stack_storage_mut_ptr().add(base) }
                    };
                    let mt_base: *mut u8 = {
                        let index = crate::vm::code_index_for_dsl(active_frame.code());
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
                    // SAFETY: state is valid by from_raw's contract;
                    // we hold a unique borrow through `self`. Mirror
                    // the new PC back into `state.frame_pc_offset` so
                    // a subsequent slow-path Refresh — or the test
                    // harness, which reads via state — sees the
                    // authoritative value. Likewise refresh REGS/MT
                    // so the asm bridge's next dispatch picks up any
                    // reallocation that happened during nested calls.
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
                // The asm bridge's `dispatch_after_slow!` Continue
                // arm reads the new pc_offset from `x1` (`payload`)
                // directly to skip the memory round-trip on the
                // Continue arm. Mirror it here for the asm side. (The
                // α variant ignores `payload`.)
                SlowPathReturn {
                    tag: SlowPathTag::Continue as u64,
                    payload: new_offset_u64,
                }
            }
            SemanticOutcome::Refresh => {
                if let LlIntDispatchInner::Asm { state, rust } = &mut self.inner {
                    // Always pull the active frame from `vm.frames().last()`
                    // — mirrors α's `refresh_from_active_frame()`. This
                    // covers all three Refresh callers uniformly:
                    //   - call/return: frame stack depth changed, the new
                    //     top frame is the callee/caller.
                    //   - cross-frame catch: depth decreased, top frame
                    //     was rewritten to the handler PC.
                    //   - same-frame catch: depth unchanged, but
                    //     `transfer_to_exception_handler` rewrote
                    //     `vm.frames.last_mut().instruction_offset` to the
                    //     handler target. `rust.dispatch.frame` is a
                    //     `Copy` snapshot of the pre-throw frame and is
                    //     stale; only `vm.frames().last()` has the
                    //     authoritative post-catch PC.
                    let current_depth = rust.dispatch.vm.frames().len();
                    if let Some(active) = rust.dispatch.vm.frames().last().copied() {
                        rust.dispatch.frame = active;
                    }
                    rust.dispatch.frame_depth = current_depth;
                    // Refresh `installed` unconditionally as well — even
                    // for same-frame catch the code identity is the same,
                    // but `installed_for_dsl_runtime` is a cheap lookup
                    // and matches α's `refresh_from_active_frame()`
                    // unconditional reinstall.
                    let installed = rust
                        .dispatch
                        .vm
                        .installed_for_dsl_runtime(rust.dispatch.frame.code())
                        .unwrap_or_else(|| rust.dispatch.installed.clone());
                    rust.dispatch.installed = installed;
                    // Sync the frame-check epoch — α does this in
                    // `refresh_from_active_frame`. Keeps the DSL Refresh
                    // path observationally identical to α.
                    rust.dispatch.frame_check_epoch =
                        rust.dispatch.vm.dispatch_frame_check_epoch_for_dsl();
                    let active_frame = rust.dispatch.frame;
                    let regs_base_ptr = {
                        let base = active_frame.registers().base() as usize;
                        // SAFETY: register window is reserved on the
                        // active frame; one-past-the-end is well-defined.
                        unsafe { rust.dispatch.vm.register_stack_storage_mut_ptr().add(base) }
                    };
                    let pb_base = rust
                        .dispatch
                        .installed
                        .function()
                        .instruction_bytes()
                        .as_ptr();
                    let mt_base: *mut u8 = {
                        let index = crate::vm::code_index_for_dsl(active_frame.code());
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
                    // Phase 1.B.1: derive the new fields for the
                    // active frame. Identical chain to the entry shim
                    // in entry.rs::run_via_dsl. See spec §3.4.
                    let const_base: *const lyng_types::Value = rust
                        .dispatch
                        .agent
                        .heap()
                        .view()
                        .code(active_frame.code())
                        .and_then(lyng_gc::RuntimeCodeRecord::constants)
                        .and_then(|slots| rust.dispatch.agent.heap().view().code_slots(slots))
                        .map_or(std::ptr::null(), <[lyng_types::Value]>::as_ptr);

                    // Phase 1.B.1: refresh the `this` mirror. Captures
                    // super() mutations and any other slow-path
                    // changes to frame.this_value().
                    let this_value =
                        crate::dsl::llint_state::resolve_initial_this_value(&active_frame);
                    // SAFETY: state is valid by from_raw's contract.
                    unsafe {
                        (**state).frame_pc_offset = active_frame.instruction_offset();
                        (**state).frame_pb_base = pb_base;
                        (**state).frame_regs_base = regs_base_ptr;
                        (**state).frame_metadata_table_base = mt_base;
                        (**state).object_records_base = object_records_base;
                        (**state).object_slots_base = object_slots_base;
                        (**state).value_cells_base = value_cells_base;
                        // Phase 1.B.1: refresh the new fields.
                        (**state).frame_const_base = const_base;
                        (**state).frame_this_value = this_value;
                    }
                    // Phase 1.B.1: debug-only stability assertion.
                    // The arena slot's data pointer must be stable
                    // across the slow-path call. If this fires, the
                    // arena moved under us — investigate before
                    // disabling. Matches the implicit invariant
                    // `frame_pb_base` already relies on. See spec §3.6.
                    #[cfg(debug_assertions)]
                    {
                        let recomputed: *const lyng_types::Value = rust
                            .dispatch
                            .agent
                            .heap()
                            .view()
                            .code(active_frame.code())
                            .and_then(lyng_gc::RuntimeCodeRecord::constants)
                            .and_then(|slots| rust.dispatch.agent.heap().view().code_slots(slots))
                            .map_or(std::ptr::null(), <[lyng_types::Value]>::as_ptr);
                        debug_assert_eq!(
                            const_base, recomputed,
                            "frame_const_base unstable across Refresh"
                        );
                    }
                    rust.dispatch.refresh_dsl_poll_pending();
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

/// asm-facing return ABI for cold-stub shims. The asm bridge reads
/// `tag` and dispatches on it (`Continue` / `Refresh` / `Exit`).
/// `payload` is reserved for future single-word returns (e.g. a packed
/// PC delta); DSL-0b leaves it zero.
#[repr(C)]
pub struct SlowPathReturn {
    pub tag: u64,
    pub payload: u64,
}

/// Tag values used by [`SlowPathReturn::tag`]. The integers are part of
/// the asm-DSL ABI — backend code may hard-code the constants.
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
        let pc = inner.frame.instruction_offset();
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
        let pc = inner.frame.instruction_offset();
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
        let pc = inner.frame.instruction_offset();
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

/// Generate an asm-facing cold-stub shim from a semantic body. Keeps
/// every cold stub's wrapper to one declaration site. Emits a
/// `#[no_mangle] pub extern "C" fn` that reconstructs an
/// `LlIntDispatchState` from the raw `*mut LlIntState`, mirrors asm
/// state into the Rust context, dispatches to the semantic body, and
/// translates the outcome back into a `SlowPathReturn`.
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
            // SAFETY: `state` is a valid `*mut LlIntState` for the
            // duration of this call; the asm bridge upholds the
            // contract documented on `LlIntDispatchState::from_raw`.
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
