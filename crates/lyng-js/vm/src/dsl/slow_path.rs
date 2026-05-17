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

    /// Mutable access to the underlying `DispatchState`. Semantic
    /// bodies use this for now; the DSL-0b refactor replaces this with
    /// typed accessors that operate uniformly across α and asm paths.
    pub fn dispatch_state(&mut self) -> &mut DispatchState<'vm> {
        match &mut self.inner {
            LlIntDispatchInner::Alpha(state) => *state,
            LlIntDispatchInner::Asm { .. } => {
                // The asm path uses typed accessors landed in later
                // DSL-0b tasks. Hitting this branch means an α-only
                // call site mis-fired on an asm-constructed dispatch
                // state.
                panic!("LlIntDispatchState::dispatch_state called on asm variant");
            }
        }
    }

    /// Pre-slow-path sync — copy asm-side mirrors into the Rust-side
    /// snapshot before semantic code observes the frame. See design §6.
    ///
    /// Idempotent on the α variant (asm mirrors do not exist there;
    /// `rust.frame` is already authoritative).
    pub fn sync_from_asm(&mut self) {
        if let LlIntDispatchInner::Asm { state, rust } = &mut self.inner {
            // SAFETY: `state` is valid by `from_raw`'s contract; we
            // only read scalar fields here.
            unsafe {
                rust.frame.set_instruction_offset((**state).frame_pc_offset);
            }
            // registers_base / fv_base are mirrored back via the
            // `Refresh` path in `translate_outcome`; semantic bodies
            // read those through `rust.installed.feedback_flat` and
            // the register window, both of which are still authoritative
            // on entry (the asm side has not relocated them).
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
                if let LlIntDispatchInner::Asm { state, rust } = &mut self.inner {
                    let new_offset = rust
                        .frame
                        .instruction_offset()
                        .wrapping_add(pc_advance);
                    // SAFETY: state is valid by from_raw's contract;
                    // we hold a unique borrow through `self`.
                    unsafe {
                        (**state).frame_pc_offset = new_offset;
                    }
                }
                SlowPathReturn {
                    tag: SlowPathTag::Continue as u64,
                    payload: 0,
                }
            }
            SemanticOutcome::Refresh => {
                if let LlIntDispatchInner::Asm { state, rust } = &mut self.inner {
                    // SAFETY: state is valid by from_raw's contract.
                    unsafe {
                        (**state).frame_pc_offset = rust.frame.instruction_offset();
                        // frame_regs_base / frame_fv_base remain
                        // authoritative as established at entry —
                        // FrameRecord's RegisterWindow does not move
                        // during one trampoline call, and the FV pin
                        // is pinned to `installed.feedback_flat`.
                        // Batch 3 (Task B16) wires these to live
                        // pointer accessors on FrameRecord /
                        // InstalledFunction.
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
