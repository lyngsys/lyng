//! Test fixtures for DSL handler and slow-path bridge validation.
//!
//! [`DslHarness`] exposes two surfaces:
//!
//! 1. [`DslHarness::assert_handler_symbol_exists`] — link-time check
//!    that an `llint_handler!`-expanded symbol is present and non-null.
//! 2. [`DslHarness::invoke_semantic_directly`] — calls a semantic
//!    function with a manually-constructed `LlIntDispatchState::Asm`
//!    variant, bypassing the trampoline. Mirrors what
//!    `dsl_cold_shim!`-emitted shims do at runtime: build the dispatch
//!    state via `from_raw`, call `sync_from_asm`, dispatch to the
//!    semantic, then `translate_outcome`.
//!
//! The `LlIntRustContext` fields are `pub(crate)`, so the harness lives
//! here and is re-exported via `#[doc(hidden)] pub` for integration tests.

#![allow(
    dead_code,
    clippy::missing_const_for_fn,
    reason = "test harness wrappers intentionally stay ordinary functions so integration tests can call semantic bodies uniformly"
)]

use std::sync::Arc;

use lyng_common::{AtomTable, SourceId};
use lyng_compiler::compile_script;
use lyng_env::Runtime;
use lyng_host::{HostHooks, NoopHostHooks};
use lyng_objects::{
    InternalMethodResult, NativeCallRequest, NativeConstructRequest, NativeFunctionRegistry,
    ObjectRuntime,
};
use lyng_parser::parse_script;
use lyng_sema::analyze_script;
use lyng_types::Value;

use crate::dsl::llint_state::{
    ExitKind, LlIntExitSlot, LlIntRustContext, LlIntRustContextOpaque, LlIntState,
};
use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome, SlowPathTag};
use crate::error::VmError;
use crate::vm::install::InstalledFunction;
use crate::{FrameRecord, Vm};

/// Decoded result of [`DslHarness::invoke_semantic_directly`].
///
/// Reconstructed from the `SlowPathTag` and the `LlIntRustContext.exit` slot.
#[derive(Debug)]
pub enum HarnessOutcome {
    /// `SlowPathTag::Continue`; new PC offset in `state.frame_pc_offset`.
    Continued { new_pc_offset: u32 },
    /// `SlowPathTag::Refresh`.
    Refreshed,
    /// `SlowPathTag::Exit` with `exit.kind = Done`.
    Done { value: Value },
    /// `SlowPathTag::Exit` with `exit.kind = Error`.
    Error { error: Box<VmError> },
}

/// Rejects every native call — the harness never invokes JS host code.
#[derive(Default)]
pub(crate) struct RejectingRegistry;

impl NativeFunctionRegistry for RejectingRegistry {
    fn call(
        &mut self,
        _runtime: &mut ObjectRuntime,
        _heap: &mut lyng_gc::PrimitiveMutator<'_>,
        _request: NativeCallRequest<'_>,
    ) -> InternalMethodResult<Value> {
        panic!("DslHarness should not invoke native calls");
    }

    fn construct(
        &mut self,
        _runtime: &mut ObjectRuntime,
        _heap: &mut lyng_gc::PrimitiveMutator<'_>,
        _request: NativeConstructRequest<'_>,
    ) -> InternalMethodResult<lyng_types::ObjectRef> {
        panic!("DslHarness should not invoke native constructs");
    }
}

/// Owns the long-lived state needed to drive a DSL handler / semantic
/// body in isolation.
///
/// The harness compiles a one-statement script (`var __dsl_harness = 0;`)
/// solely to obtain a real `Arc<InstalledFunction>` + `FrameRecord` for
/// the dispatch state. Semantic bodies under test never read the
/// installed code — they only need the dispatch wrapper to exist.
pub struct DslHarness {
    runtime: Runtime,
    vm: Vm,
    installed: Arc<InstalledFunction>,
    frame: FrameRecord,
    registry: RejectingRegistry,
}

impl DslHarness {
    /// Build a fresh harness. Sets up a `Runtime` + `Vm`, installs a
    /// trivial script, and captures the resulting installed function
    /// and a synthesized `FrameRecord` pointing at offset 0.
    pub fn new() -> Self {
        let mut runtime = Runtime::new(NoopHostHooks);
        let mut vm = Vm::new();
        let (installed, frame) = {
            let agent = runtime.root_agent_mut();
            let realm = agent
                .default_realm()
                .expect("default realm should exist for harness bootstrap");

            // Compile a one-liner so the harness has an installed
            // code record + corresponding feedback storage. Semantic
            // bodies under test never inspect the installed bytecode,
            // so the contents are arbitrary as long as compilation
            // succeeds.
            let mut atoms = AtomTable::new();
            let parsed = parse_script(&mut atoms, SourceId::new(0xD51), "var __dsl_harness = 0;");
            assert!(
                !parsed.diagnostics.has_errors(),
                "DslHarness bootstrap script should parse",
            );
            let sema = analyze_script(&parsed, &atoms);
            assert!(
                !sema.diagnostics.has_errors(),
                "DslHarness bootstrap script should pass sema",
            );
            let unit =
                compile_script(&parsed, &sema, &mut atoms).expect("DslHarness bootstrap compile");

            let installed_code = vm
                .install_script(agent, realm.id(), &unit)
                .expect("DslHarness install_script");

            let code_ref = installed_code.code();
            let installed = vm
                .installed_for_dsl_harness(code_ref)
                .expect("installed function should be present after install_script");

            // Build a synthesized FrameRecord whose RegisterWindow is a
            // zero-width window placed at the base a real `cfr == 0` frame
            // would use: a frame's window starts at `cfr + HEADER_SLOTS`, so
            // window base `HEADER_SLOTS` represents the valid frame at cfr 0.
            // Semantic bodies under test don't read this window, but the
            // `DispatchState` thin-view seed derives `cfr` via `Vm::cfr_of`
            // (`window_base - HEADER_SLOTS`); a base-0 window would underflow.
            let registers =
                crate::frame::RegisterWindow::new(crate::frame_header::HEADER_SLOTS as u32, 0);
            let frame = FrameRecord::new(
                code_ref,
                0,
                registers,
                None,
                realm.global_env(),
                realm.global_env(),
                lyng_env::ExecutionContextKind::Script,
            );

            (installed, frame)
        };

        Self {
            runtime,
            vm,
            installed,
            frame,
            registry: RejectingRegistry,
        }
    }

    /// Link-time check that a `llint_handler!`-expanded symbol is present.
    pub fn assert_handler_symbol_exists(handler: unsafe extern "C" fn() -> !) {
        let ptr = handler as *const ();
        assert!(!ptr.is_null(), "DSL handler symbol should not be null");
    }

    /// Drive a semantic body through the slow-path bridge, bypassing
    /// the trampoline. Mirrors what a `dsl_cold_shim!`-emitted shim
    /// does at runtime: `from_raw` → `sync_from_asm` → semantic →
    /// `translate_outcome`. Returns a [`HarnessOutcome`] decoded from
    /// the resulting `SlowPathReturn` + `exit` slot.
    pub fn invoke_semantic_directly<F>(&mut self, entry_pc: u32, semantic: F) -> HarnessOutcome
    where
        F: for<'vm, 'borrow> FnOnce(&mut LlIntDispatchState<'vm, 'borrow>) -> SemanticOutcome,
    {
        let agent = self.runtime.root_agent_mut();
        let host: &dyn HostHooks = &NoopHostHooks;

        let installed = Arc::clone(&self.installed);
        let frame = self.frame;
        let frame_depth = 0usize;
        let dispatch = crate::vm::dispatch_state::DispatchState::new_for_dsl_entry(
            &mut self.vm,
            agent,
            host,
            &mut self.registry,
            installed,
            frame,
            frame_depth,
            0,
        );
        let mut rust_ctx = LlIntRustContext {
            dispatch,
            exit: LlIntExitSlot::default(),
        };

        // `frame_regs_base` / `frame_pb_base` are not read by the bridge
        // logic under test; null placeholders are safe.
        let mut state = LlIntState {
            frame_pc_offset: entry_pc,
            _pad1: 0,
            frame_pb_base: core::ptr::null(),
            frame_regs_base: core::ptr::null_mut(),
            frame_metadata_table_base: core::ptr::null_mut(),
            object_records_base: core::ptr::null(),
            object_slots_base: core::ptr::null(),
            // Not exercised by the harness; null/undefined placeholders are safe.
            frame_const_base: core::ptr::null(),
            frame_this_value: Value::undefined(),
            frame_depth: 0,
            frame_check_epoch: 0,
            rust_context: (&raw mut rust_ctx).cast::<LlIntRustContextOpaque>(),
            prefix: 0,
            _pad2: [0; 7],
            // Not exercised by the harness; null placeholder is safe.
            value_cells_base: core::ptr::null(),
        };

        // SAFETY: `state` lives on this stack frame for the duration
        // of the call; `state.rust_context` points at the
        // `rust_ctx` local above, also pinned on this stack frame.
        // `from_raw`'s contract requires both — satisfied.
        let return_value = unsafe {
            let mut dispatch = LlIntDispatchState::from_raw(&raw mut state);
            dispatch.sync_from_asm();
            let outcome = semantic(&mut dispatch);
            dispatch.translate_outcome(outcome)
        };

        // Decode `SlowPathReturn` + `exit` into a `HarnessOutcome`.
        if return_value.tag == SlowPathTag::Continue as u64 {
            HarnessOutcome::Continued {
                new_pc_offset: state.frame_pc_offset,
            }
        } else if return_value.tag == SlowPathTag::Refresh as u64 {
            HarnessOutcome::Refreshed
        } else {
            // SlowPathTag::Exit — the kind is in `rust_ctx.exit`.
            match rust_ctx.exit.kind {
                ExitKind::Done => HarnessOutcome::Done {
                    value: rust_ctx.exit.done_value,
                },
                ExitKind::Error => HarnessOutcome::Error {
                    error: rust_ctx
                        .exit
                        .error
                        .take()
                        .expect("Error variant must carry an error"),
                },
                ExitKind::None => panic!(
                    "DslHarness: SlowPathTag::Exit returned but exit.kind == None — \
                     bridge invariant violated"
                ),
            }
        }
    }

    /// Drive a semantic body through the Alpha variant of
    /// [`LlIntDispatchState`]. `init_prefix` seeds `DispatchState.prefix`
    /// before the body runs.
    pub fn with_alpha_dispatch<R>(
        &mut self,
        init_prefix: Option<lyng_bytecode::Opcode>,
        body: impl for<'vm, 'borrow> FnOnce(&mut LlIntDispatchState<'vm, 'borrow>) -> R,
    ) -> R {
        let agent = self.runtime.root_agent_mut();
        let host: &dyn HostHooks = &NoopHostHooks;
        let installed = Arc::clone(&self.installed);
        let frame = self.frame;

        let mut state = crate::vm::dispatch_state::DispatchState::new_for_dsl_harness(
            &mut self.vm,
            agent,
            host,
            &mut self.registry,
            installed,
            frame,
            init_prefix,
        );

        let mut dispatch = LlIntDispatchState::from_alpha(&mut state);
        body(&mut dispatch)
    }
}

impl Default for DslHarness {
    fn default() -> Self {
        Self::new()
    }
}

/// Re-exports of prefix-family semantic bodies for integration tests.
/// The `_via_dsl_harness` suffix avoids the `op_*` naming convention
/// which is guarded by `dsl_manifest_grep`.
pub mod prefix_semantics {
    use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
    use crate::vm::semantics::prefix::{
        OpPrefixArgs, op_extra_wide_semantic as inner_op_extra_wide_semantic,
        op_wide_semantic as inner_op_wide_semantic,
    };

    /// `op_wide_semantic` made visible to integration tests.
    pub fn invoke_wide_semantic_via_dsl_harness(
        state: &mut LlIntDispatchState<'_, '_>,
    ) -> SemanticOutcome {
        inner_op_wide_semantic(state, OpPrefixArgs)
    }

    /// `op_extra_wide_semantic` made visible to integration tests.
    pub fn invoke_extra_wide_semantic_via_dsl_harness(
        state: &mut LlIntDispatchState<'_, '_>,
    ) -> SemanticOutcome {
        inner_op_extra_wide_semantic(state, OpPrefixArgs)
    }
}
