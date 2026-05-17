//! Entry shim and exit shim per design §5 / §6.
//!
//! The entry shim ([`run_via_dsl`]) builds an [`LlIntRustContext`] and
//! a stack-local [`LlIntState`], then jumps into the asm trampoline.
//! The trampoline runs the DSL handler chain until a cold stub writes
//! `rust_ctx.exit` and the chain unwinds back to [`_interpreter_exit`].
//!
//! For DSL-0b Batch 2 the trampoline + exit shim are stubs (single
//! `ret`), and `run_via_dsl` is not wired into `Vm::run`. Calling it at
//! runtime is currently UB — there's no real handler chain. The
//! symbols exist so backend code and the proc-macro can link against
//! them. Batch 4 fills in the asm body; C1 flips dispatch.

use std::sync::Arc;

use lyng_js_env::Agent;
use lyng_js_host::HostHooks;
use lyng_js_objects::NativeFunctionRegistry;
use lyng_js_types::Value;

use crate::dsl::llint_state::{
    ExitKind, LlIntExitSlot, LlIntRustContext, LlIntRustContextOpaque, LlIntState,
};
use crate::error::{VmError, VmResult};
use crate::vm::install::InstalledFunction;
use crate::{FrameRecord, Vm};

/// New entry point used after DSL-0c flips dispatch.
///
/// During DSL-0b this is callable but not the default — `Vm::run`
/// continues to route through the α trampoline. Task C1 swaps the
/// route. Calling it at runtime in DSL-0b is currently UB because
/// `run_dsl_trampoline` is a single-`ret` stub; the symbol exists so
/// the proc-macro and backend code can link.
#[allow(dead_code)]
pub(crate) fn run_via_dsl(
    vm: &mut Vm,
    agent: &mut Agent,
    host: &dyn HostHooks,
    registry: &mut dyn NativeFunctionRegistry,
    installed: Arc<InstalledFunction>,
    frame: FrameRecord,
) -> VmResult<Value> {
    let frame_depth = vm.frames().len();
    let pb_base = installed.function().instruction_bytes().as_ptr();
    let frame_pc_offset = frame.instruction_offset();
    // DSL-0b (B16): wire the `FV` pin to the eagerly-allocated flat
    // feedback storage on `Vm::feedback_flat_storage`. The slot is
    // keyed by `code_index(frame.code())` and was populated to
    // `function.feedback_slot_count()` default entries at install
    // (B15). Storage is pointer-stable for the lifetime of the
    // `InstalledFunction`, so the raw pointer captured here is safe
    // to hand to the asm trampoline until `run_via_dsl` returns. Cast
    // the `*const FeedbackEntry` from `.as_ptr()` to `*mut` because
    // the asm-DSL ABI types `frame_fv_base` as `*mut FeedbackEntry`;
    // the trampoline reads/writes through it on the current thread
    // only (no aliasing UB during the trampoline's single-threaded
    // execution).
    let fv_base = {
        let index = crate::vm::code_index_for_dsl(frame.code());
        // The slot is guaranteed to exist because `store_installed`
        // populates `feedback_flat_storage[index]` to the correct
        // length before any code at `code` can be invoked. An empty
        // boxed slice's `as_ptr()` is still a valid (non-dangling)
        // pointer; the asm trampoline never dereferences past the
        // slot count anyway.
        vm.feedback_flat_storage[index].as_ptr()
            as *mut crate::dsl::feedback_flat::FeedbackEntry
    };

    let mut rust_ctx = LlIntRustContext {
        vm,
        agent,
        host,
        registry,
        installed,
        frame,
        frame_depth,
        exit: LlIntExitSlot::default(),
    };

    // frame_regs_base is a placeholder null pointer in DSL-0b Batch 3.
    // The register-window pin lands later in DSL-0b once Batch 4+ wires
    // the handler chain; until then the asm trampoline is a stub and
    // never dereferences it.
    let mut state = LlIntState {
        frame_pc_offset,
        _pad1: 0,
        frame_pb_base: pb_base,
        frame_regs_base: core::ptr::null_mut::<Value>(),
        frame_fv_base: fv_base,
        frame_depth: frame_depth as u32,
        frame_check_epoch: 0,
        rust_context: (&mut rust_ctx) as *mut LlIntRustContext<'_>
            as *mut LlIntRustContextOpaque,
        prefix: 0,
        _pad2: [0; 7],
    };

    // SAFETY: `state` is a valid mutable pointer to a stack-local
    // LlIntState; the asm trampoline only reads through it on the
    // current thread for the duration of this call.
    unsafe { run_dsl_trampoline(&mut state as *mut LlIntState) };

    match rust_ctx.exit.kind {
        ExitKind::Done => Ok(rust_ctx.exit.done_value),
        ExitKind::Error => Err(*rust_ctx.exit.error.take().unwrap()),
        ExitKind::None => Err(VmError::TrampolineExitedWithoutSetting),
    }
}

/// Asm-side trampoline entry. Loads pinned registers + tail-jumps to
/// the first handler. The handler chain runs until `_interpreter_exit`
/// is hit.
///
/// In DSL-0b Batch 2 the body is a single `ret` stub. Batch 4 fills
/// the body in using the AArch64 backend macros once the handler chain
/// exists. The stub means `run_via_dsl` is currently UB at runtime —
/// it returns with `rust_ctx.exit.kind == None` and surfaces as
/// `VmError::TrampolineExitedWithoutSetting`. No caller exists yet.
#[unsafe(naked)]
pub unsafe extern "C" fn run_dsl_trampoline(_state: *mut LlIntState) {
    // x0 = state. The full version sets up pinned regs from state.frame_*
    // fields, loads VM/TABLE, then tail-jumps to the first handler.
    // Stub: just return; rust_ctx.exit.kind stays None and run_via_dsl
    // surfaces TrampolineExitedWithoutSetting.
    core::arch::naked_asm!("ret");
}

/// `_interpreter_exit` is the symbolic target the slow-path bridge
/// uses to escape the trampoline. The asm `b {exit}` branches here;
/// the function reads `rust_context.exit` and returns to the caller of
/// `run_via_dsl` via a normal Rust return.
///
/// `run_dsl_trampoline` sets up a normal stack frame, so
/// `_interpreter_exit` can be a normal `extern "C"` that pops back to
/// `run_via_dsl`. Empty body = single `ret` generated by rustc.
#[unsafe(no_mangle)]
pub extern "C" fn _interpreter_exit() {
    // Intentionally empty.
}
