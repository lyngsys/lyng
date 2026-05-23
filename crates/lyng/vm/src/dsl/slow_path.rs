//! Slow-path bridge: semantic-outcome type + per-opcode argument structs
//! + (DSL-0b) the `LlIntDispatchState` wrapper and `SlowPathReturn` ABI.
//!
//! During DSL-0a only `SemanticOutcome`, the `OpXxxArgs` structs, and the
//! transitional `LlIntDispatchState` alias are populated. The asm-facing
//! shim layer and `SlowPathReturn`/`SlowPathTag` lands in DSL-0b.

use lyng_js_types::Value;

use crate::error::VmError;

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
    pub fn from_alpha(state: &'borrow mut DispatchState<'vm>) -> Self {
        Self { inner: LlIntDispatchInner::Alpha(state) }
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
        let rust = unsafe {
            &mut *((*state).rust_context as *mut LlIntRustContext<'vm>)
        };
        Self {
            inner: LlIntDispatchInner::Asm { state, rust },
        }
    }

    /// Mutable access to the underlying `DispatchState`. Works on both
    /// dispatch variants: α handlers borrow the existing
    /// `DispatchState`; asm-path shims unpack the same shape from
    /// `LlIntRustContext::dispatch`. Semantic bodies under
    /// `crate::vm::semantics::` consume this uniformly.
    pub fn dispatch_state(&mut self) -> &mut DispatchState<'vm> {
        match &mut self.inner {
            LlIntDispatchInner::Alpha(state) => *state,
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
    pub fn current_instruction_offset(&self) -> u32 {
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
            // registers_base / fv_base are mirrored back via the
            // `Refresh` path in `translate_outcome`; semantic bodies
            // read those through `rust.dispatch.installed.feedback_flat`
            // and the register window, both of which are still
            // authoritative on entry (the asm side has not relocated
            // them).
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
    pub fn translate_outcome(&mut self, outcome: SemanticOutcome) -> SlowPathReturn {
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
                // reloads PC/REGS/FV from `vm.frames().last()`.
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
                // after the nested call returns. Recompute REGS (and
                // FV) from the live `Vm::register_stack_storage_mut_ptr`
                // on every Continue egress so the asm bridge picks up
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
                    let fv_base = {
                        let index = crate::vm::code_index_for_dsl(active_frame.code());
                        rust.dispatch.vm.feedback_flat_storage[index].as_ptr()
                            as *mut crate::dsl::feedback_flat::FeedbackEntry
                    };
                    // SAFETY: state is valid by from_raw's contract;
                    // we hold a unique borrow through `self`. Mirror
                    // the new PC back into `state.frame_pc_offset` so
                    // a subsequent slow-path Refresh — or the test
                    // harness, which reads via state — sees the
                    // authoritative value. Likewise refresh REGS/FV
                    // so the asm bridge's next dispatch picks up any
                    // reallocation that happened during nested calls.
                    unsafe {
                        (**state).frame_pc_offset = new_offset;
                        (**state).frame_regs_base = regs_base_ptr;
                        (**state).frame_fv_base = fv_base;
                    }
                }
                // The asm bridge's `dispatch_after_slow!` Continue
                // arm reads the new pc_offset from `x1` (`payload`)
                // directly to skip the memory round-trip on the fast
                // path. Mirror it here for the asm side. (The α
                // variant ignores `payload`.)
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
                    let pb_base = rust.dispatch.installed.function().instruction_bytes().as_ptr();
                    let fv_base = {
                        let index =
                            crate::vm::code_index_for_dsl(active_frame.code());
                        rust.dispatch.vm.feedback_flat_storage[index].as_ptr()
                            as *mut crate::dsl::feedback_flat::FeedbackEntry
                    };
                    // Phase 1.B.1: derive the new fields for the
                    // active frame. Identical chain to the entry shim
                    // in entry.rs::run_via_dsl. See spec §3.4.
                    let const_base: *const lyng_js_types::Value = rust
                        .dispatch
                        .agent
                        .heap()
                        .view()
                        .code(active_frame.code())
                        .and_then(lyng_js_gc::RuntimeCodeRecord::constants)
                        .and_then(|slots| {
                            rust.dispatch.agent.heap().view().code_slots(slots)
                        })
                        .map(|s| s.as_ptr())
                        .unwrap_or(std::ptr::null());

                    // Phase 1.B.1: refresh the `this` mirror. Captures
                    // super() mutations and any other slow-path
                    // changes to frame.this_value().
                    let this_value = crate::dsl::llint_state::resolve_initial_this_value(
                        rust.dispatch.agent,
                        &active_frame,
                    );
                    // SAFETY: state is valid by from_raw's contract.
                    unsafe {
                        (**state).frame_pc_offset = active_frame.instruction_offset();
                        (**state).frame_pb_base = pb_base;
                        (**state).frame_regs_base = regs_base_ptr;
                        (**state).frame_fv_base = fv_base;
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
                        let recomputed: *const lyng_js_types::Value = rust
                            .dispatch
                            .agent
                            .heap()
                            .view()
                            .code(active_frame.code())
                            .and_then(lyng_js_gc::RuntimeCodeRecord::constants)
                            .and_then(|slots| {
                                rust.dispatch.agent.heap().view().code_slots(slots)
                            })
                            .map(|s| s.as_ptr())
                            .unwrap_or(std::ptr::null());
                        debug_assert_eq!(
                            const_base, recomputed,
                            "frame_const_base unstable across Refresh"
                        );
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
