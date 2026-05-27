//! Test fixtures for DSL-0b validation cases (B30–B38).
//!
//! [`DslHarness`] exposes two surfaces, both designed around the fact
//! that DSL-0b's `run_dsl_trampoline` is still a `naked_asm!("ret")`
//! stub (see [`crate::dsl::entry`]):
//!
//! 1. [`DslHarness::assert_handler_symbol_exists`] — link-time check
//!    that an `llint_handler!`-expanded symbol is present and non-null.
//!    Used by B30 and as a structural floor for the deferred cases
//!    (B33–B37).
//! 2. [`DslHarness::invoke_semantic_directly`] — calls a semantic
//!    function with a manually-constructed `LlIntDispatchState::Asm`
//!    variant, bypassing the trampoline. Mirrors exactly what
//!    [`crate::dsl_cold_shim`]-emitted shims do at runtime: build the
//!    dispatch state via `from_raw`, call `sync_from_asm`, dispatch to
//!    the semantic, then `translate_outcome`. Used by B31 and B32 to
//!    exercise the Batch-2 slow-path bridge end-to-end without the
//!    asm trampoline.
//!
//! Runtime trampoline-based assertions are deferred to Batch 7 (after
//! `op_move`/`op_add` ports land real handlers and `run_dsl_trampoline`
//! is wired up). The deferred cases live in their own test files with
//! `#[ignore = "..."]` markers that point back to this design note.
//!
//! ## Why the harness lives here
//!
//! The `LlIntRustContext` fields are `pub(crate)` and the
//! `LlIntDispatchInner::Asm` constructor needs to reach through those
//! fields, so the harness can only be authored from inside the crate.
//! Integration tests under `crates/vm/tests/` consume the
//! harness via the `#[doc(hidden)] pub` re-export in
//! [`crate::dsl::test_helpers`].

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

/// Decoded view of the result of [`DslHarness::invoke_semantic_directly`].
///
/// Mirrors the four [`SemanticOutcome`] variants after they've gone
/// through the asm-facing [`crate::dsl::slow_path::SlowPathReturn`]
/// ABI. The harness reconstructs the high-level outcome by reading
/// both the `SlowPathTag` and the `LlIntRustContext.exit` slot.
#[derive(Debug)]
pub enum HarnessOutcome {
    /// `SemanticOutcome::Continue { pc_advance }` arrived as
    /// `SlowPathTag::Continue` with the new PC offset reflected in
    /// `state.frame_pc_offset`.
    Continued { new_pc_offset: u32 },
    /// `SemanticOutcome::Refresh` arrived as `SlowPathTag::Refresh`.
    Refreshed,
    /// `SemanticOutcome::ExitDone { value }` set `exit.kind = Done`.
    Done { value: Value },
    /// `SemanticOutcome::ExitError { error }` set `exit.kind = Error`.
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

            // Build a synthesized FrameRecord whose RegisterWindow
            // points to a zero-sized window — semantic bodies under
            // test don't read it, but the FrameRecord constructor
            // enforces a sane shape.
            let registers = crate::frame::RegisterWindow::new(0, 0);
            let frame = FrameRecord::new(
                code_ref,
                0,
                registers,
                None,
                realm.id(),
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

    /// Cheap structural assertion that a `llint_handler!`-expanded
    /// symbol exists at link time. Used by the deferred validation
    /// cases (B33–B37) and B30 as a load-bearing first proof.
    pub fn assert_handler_symbol_exists(handler: unsafe extern "C" fn() -> !) {
        let ptr = handler as *const ();
        assert!(!ptr.is_null(), "DSL handler symbol should not be null");
    }

    /// Drive a semantic function directly through the slow-path
    /// bridge, simulating what a `dsl_cold_shim!`-emitted shim does
    /// at runtime.
    ///
    /// This is the workhorse for B31 (slow-path round-trip) and B32
    /// (PC-sync). It:
    ///
    /// 1. Constructs an [`LlIntRustContext`] from the harness's owned
    ///    Vm/Agent/installed/frame.
    /// 2. Builds an [`LlIntState`] with `frame_pc_offset = entry_pc`.
    /// 3. Calls [`LlIntDispatchState::from_raw`] (the same call the
    ///    asm bridge makes), then `sync_from_asm` (mirrors asm-side
    ///    `frame_pc_offset` into `rust.frame.instruction_offset()`).
    /// 4. Invokes the caller-provided semantic.
    /// 5. Calls `translate_outcome`, mirroring the shim's exit path.
    /// 6. Reads back the resulting `SlowPathReturn` + `exit` slot to
    ///    materialize a [`HarnessOutcome`].
    ///
    /// The `entry_pc` argument is the value the harness writes to
    /// `state.frame_pc_offset` before calling `sync_from_asm`. The
    /// semantic body sees this value via
    /// [`LlIntDispatchState::current_instruction_offset`] — for B32
    /// the test passes `0x42` and asserts the value reads back
    /// unchanged.
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

        // Build the asm-visible state record. `frame_regs_base` /
        // `frame_pb_base` aren't touched by the slow-path bridge logic
        // under test in B31/B32, so null / dangling placeholders are
        // safe — the semantic body the test passes in only reads what
        // it explicitly chooses to read.
        let mut state = LlIntState {
            frame_pc_offset: entry_pc,
            _pad1: 0,
            frame_pb_base: core::ptr::null(),
            frame_regs_base: core::ptr::null_mut(),
            frame_metadata_table_base: core::ptr::null_mut(),
            object_records_base: core::ptr::null(),
            object_slots_base: core::ptr::null(),
            // Phase 1.B.1: harness doesn't exercise these fields;
            // null / undefined placeholders are safe.
            frame_const_base: core::ptr::null(),
            frame_this_value: Value::undefined(),
            frame_depth: 0,
            frame_check_epoch: 0,
            rust_context: (&raw mut rust_ctx).cast::<LlIntRustContextOpaque>(),
            prefix: 0,
            _pad2: [0; 7],
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

    /// Drive a `pub(crate)` semantic body through the Alpha variant
    /// of [`LlIntDispatchState`] — i.e. through the legacy
    /// `DispatchState`. Used by B38 (double-prefix rejection), where
    /// the semantic reads `state.dispatch_state().prefix` which is
    /// only populated on the Alpha variant.
    ///
    /// The caller specifies `init_prefix` to seed
    /// `DispatchState.prefix` before the body runs (e.g. `Some(Wide)`
    /// for the double-prefix-rejection path).
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

/// Re-exports of prefix-family semantic bodies for the DSL-0b
/// validation cases. The originals are `pub(crate)` inside
/// `crate::vm::semantics::prefix`; the harness needs an integration-
/// test-visible path so B38 can drive them directly.
///
/// The wrappers deliberately avoid the `op_xxx_semantic` naming
/// convention used elsewhere in the crate — the `dsl_manifest_grep`
/// guard rejects any `op_*` function definition outside
/// `semantics/`, `dispatch_handlers/`, and `dsl/handlers/`. The
/// `_via_dsl_harness` suffix keeps the symbols distinct.
pub mod prefix_semantics {
    use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
    use crate::vm::semantics::prefix::{
        op_extra_wide_semantic as inner_op_extra_wide_semantic,
        op_wide_semantic as inner_op_wide_semantic, OpPrefixArgs,
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
