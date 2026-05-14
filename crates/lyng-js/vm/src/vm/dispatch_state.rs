#![allow(
    improper_ctypes_definitions,
    reason = "extern \"C\" handlers carry Rust enums by value as an ABI-stability choice, not as a real FFI boundary"
)]
#![allow(
    dead_code,
    reason = "Phase 1 sub-1 in progress: scaffolding is in place ahead of run_trampoline wiring into evaluate_entry behind cfg(feature = \"trampoline-dispatch\")"
)]

//! Phase 1 Option α dispatch primitives — the per-handler ABI specified in
//! `reports/js/lyng-js/jsc-aligned-engine-roadmap.md` and verified by the
//! `lyng-33i2` trampoline spike (see `reports/js/lyng-js/phase-1-spike.md`).
//!
//! `DispatchState<'vm>` bundles every reference a handler needs — the live
//! `Vm`, `Agent`, host hooks, native-function registry, the active
//! `FrameRecord`, and the `Arc<InstalledFunction>` whose `instruction_bytes()`
//! the handler is decoding. Handlers are `extern "C" fn`s receiving
//! `&mut DispatchState` and returning `Step`; the trampoline does the indirect
//! call and loops on `Step::Continue(handler)`.
//!
//! `run_trampoline` is the alternative dispatch entry point. It is not yet
//! reachable from `evaluate_entry_with_registry_from_offset` — that wiring
//! lands behind a `trampoline-dispatch` feature flag in a follow-up commit.

use std::sync::Arc;

use lyng_js_env::Agent;
use lyng_js_host::HostHooks;
use lyng_js_objects::NativeFunctionRegistry;
use lyng_js_types::{CodeRef, Value};

use crate::error::{VmError, VmResult};
use crate::FrameRecord;

use super::dispatch_handlers;
use super::install::InstalledFunction;
use super::{code_index, Vm};

/// Per-frame execution state threaded through every handler call.
///
/// All references share the `'vm` lifetime — the state exists only for one
/// `run_trampoline` invocation. Handlers split-borrow the fields when they
/// need both `&mut vm` and another `&mut` field at once:
///
/// ```ignore
/// let DispatchState { vm, agent, host, registry, frame, .. } = &mut *state;
/// let result = vm.execute_add_opcode(agent, host, registry, frame, b, c);
/// ```
pub struct DispatchState<'vm> {
    pub(crate) vm: &'vm mut Vm,
    pub(crate) agent: &'vm mut Agent,
    pub(crate) host: &'vm dyn HostHooks,
    pub(crate) registry: &'vm mut (dyn NativeFunctionRegistry + 'vm),
    pub(crate) installed: Arc<InstalledFunction>,
    pub(crate) frame: FrameRecord,
    pub(crate) frame_depth: usize,
    pub(crate) frame_check_epoch: u32,
}

impl<'vm> DispatchState<'vm> {
    /// Bytes from the active PC to the end of the function's instruction
    /// stream. Handlers slice this to decode their operands and look up the
    /// next opcode byte for `dispatch_next!`.
    #[inline]
    pub(crate) fn current_bytes(&self) -> &[u8] {
        let pc = self.frame.instruction_offset() as usize;
        &self.installed.function.instruction_bytes()[pc..]
    }

    #[inline]
    pub(crate) fn first_opcode_byte(&self) -> u8 {
        self.current_bytes()[0]
    }

    #[inline]
    pub(crate) fn next_opcode_byte(&self) -> u8 {
        self.current_bytes()[0]
    }

    #[inline]
    pub(crate) fn read_register(&self, idx: u16) -> Value {
        self.vm.read_register(self.frame.registers(), idx)
    }

    #[inline]
    pub(crate) fn write_register(&mut self, idx: u16, value: Value) {
        let registers = self.frame.registers();
        self.vm.write_register(registers, idx, value);
    }

    #[inline]
    pub(crate) fn advance(&mut self, n: u32) {
        let next = self
            .frame
            .instruction_offset()
            .checked_add(n)
            .expect("instruction offset should stay within u32");
        self.frame.set_instruction_offset(next);
    }

    #[inline]
    pub(crate) fn code(&self) -> CodeRef {
        self.frame.code()
    }

    /// Stub for feedback recording — Phase 1 sub-4 wires this through to
    /// `Vm::record_feedback_slot` against the live `FeedbackVector`. The
    /// arithmetic handlers call it on the SMI fast path.
    #[inline]
    pub(crate) fn record_feedback_arithmetic_smi(&mut self, _slot: u16) {
        // TODO(lyng-54em): integrate with FeedbackVector via
        // self.vm.record_feedback_slot(self.code(), Some(FeedbackSlotId::from_raw(slot)?)).
    }
}

/// Per-opcode handler ABI. Each handler returns a `Step` describing what the
/// trampoline should do next.
pub type Handler = extern "C" fn(&mut DispatchState) -> Step;

/// Trampoline control-flow value. The trampoline keeps the active handler in
/// a local variable and only inspects this enum's discriminant.
pub enum Step {
    Continue(Handler),
    Done(Value),
    Error(VmError),
}

/// Tail of every fast-path handler: pick the next handler from
/// `DISPATCH_TABLE` indexed by the byte at the current `pc`, and return it
/// inside `Step::Continue`. The trampoline turns this into one indirect call
/// per opcode.
///
/// `dispatch_next!` is the *only* place in any handler body that should
/// reference `DISPATCH_TABLE` — Phase 1's acceptance criteria grep for this
/// invariant.
#[macro_export]
macro_rules! dispatch_next {
    ($state:expr) => {
        return $crate::vm::dispatch_state::Step::Continue(
            $crate::vm::dispatch_state::DISPATCH_TABLE[$state.next_opcode_byte() as usize],
        )
    };
}

/// Sized to cover every byte that could land in `bytes[pc]`. The first
/// `lyng_js_bytecode::OPCODE_COUNT` slots map to real or stub handlers; the
/// rest are `op_stub`, so an invalid byte fails cleanly rather than indexing
/// past the table.
pub const DISPATCH_TABLE_LEN: usize = 256;

/// Static dispatch table — one `Handler` per opcode byte value.
pub static DISPATCH_TABLE: [Handler; DISPATCH_TABLE_LEN] =
    dispatch_handlers::build_dispatch_table();

/// Central trampoline. One indirect call per opcode. The hot path is the
/// `Step::Continue(next) => handler = next` arm; `Done` and `Error` are
/// taken once per script.
///
/// `#[inline(never)]` keeps this as a standalone symbol for `cargo asm`
/// inspection across the family-conversion sub-issues. Once Phase 1 cuts
/// over and the trampoline is the only dispatch path, this attribute can
/// be revisited.
#[inline(never)]
pub fn run_trampoline(state: &mut DispatchState) -> VmResult<Value> {
    let mut handler = DISPATCH_TABLE[state.first_opcode_byte() as usize];
    loop {
        match (handler)(state) {
            Step::Continue(next) => handler = next,
            Step::Done(value) => return Ok(value),
            Step::Error(error) => return Err(error),
        }
    }
}

impl Vm {
    /// Bridge from the live `Vm::run` entrypoint into the trampoline
    /// dispatch path. Constructs a `DispatchState` from the current active
    /// frame, then hands control to `run_trampoline`.
    ///
    /// Reachable only with `--features trampoline-dispatch`. Until
    /// sub-3..sub-7 land real handlers for every opcode family, most
    /// programs hit `op_stub` and return `Step::Error(VmError::MissingActiveFrame)`.
    ///
    /// Frame transitions (Call*, Construct, TailCall) are handled by the
    /// family handlers themselves in sub-6; this entry point only sets up
    /// the initial frame snapshot.
    pub(super) fn run_via_trampoline(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> VmResult<Value> {
        let frame_depth = self.frames.len();
        let frame = self
            .frames
            .last()
            .copied()
            .expect("evaluation should install one active frame");
        let code = frame.code();
        let installed = self
            .installed
            .get(code_index(code))
            .and_then(Option::as_ref)
            .cloned()
            .ok_or(VmError::MissingInstalledCode(code))?;
        let frame_check_epoch = self.dispatch_frame_check_epoch();

        let mut state = DispatchState {
            vm: self,
            agent,
            host,
            registry,
            installed,
            frame,
            frame_depth,
            frame_check_epoch,
        };

        run_trampoline(&mut state)
    }
}
