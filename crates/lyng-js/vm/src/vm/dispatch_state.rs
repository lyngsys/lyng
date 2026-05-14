#![allow(
    improper_ctypes_definitions,
    reason = "extern \"C\" handlers carry Rust enums by value as an ABI-stability choice, not as a real FFI boundary"
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
//! Post sub-8 cutover (`lyng-9gyk`), `run_trampoline` is the only dispatch
//! path — `Vm::run` routes here via `run_via_trampoline`.

use std::sync::Arc;

use lyng_js_bytecode::Opcode;
use lyng_js_env::Agent;
use lyng_js_host::HostHooks;
use lyng_js_objects::NativeFunctionRegistry;
use lyng_js_types::{CodeRef, Value};

use crate::error::{VmError, VmResult};
use crate::FrameRecord;
use lyng_js_types::AbruptCompletion;

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
    /// Set by `op_wide` / `op_extra_wide` to widen the next handler's
    /// operand decoding. The semantic handler consumes the prefix via
    /// `state.prefix.take()` so subsequent handlers see `None`.
    pub(crate) prefix: Option<Opcode>,
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

    /// Hot-path read of the byte at the current `pc`, with the slice
    /// bounds check elided. Mirrors JSC LLInt's `loadb [PB, PC, 1], t0`
    /// pattern: the bytecode validator guarantees that any opcode
    /// reachable via `dispatch_next!` is followed by another valid
    /// opcode byte (every script-completion path ends in `Return` /
    /// `ReturnUndefined`, which exit via `Step::Done` rather than
    /// `dispatch_next!`).
    ///
    /// # Safety
    ///
    /// Caller must guarantee `self.frame.instruction_offset() <
    /// self.installed.function.instruction_bytes().len()`. The dispatch
    /// path satisfies this via the bytecode-emitter invariant and
    /// terminal-opcode semantics described above.
    #[inline]
    pub(crate) fn next_opcode_byte(&self) -> u8 {
        let bytes = self.installed.function.instruction_bytes();
        let pc = self.frame.instruction_offset() as usize;
        debug_assert!(
            pc < bytes.len(),
            "dispatch_next! reached past end of bytecode — terminal opcode invariant violated"
        );
        // SAFETY: contract above — every dispatched opcode is followed
        // by another opcode byte; terminal opcodes (Return /
        // ReturnUndefined) exit via Step::Done, not dispatch_next!.
        unsafe { *bytes.as_ptr().add(pc) }
    }

    /// Hot-path PC advance, with the u32-overflow check elided.
    /// Validated bytecode is bounded far below `u32::MAX`, so
    /// `wrapping_add` is functionally equivalent to `checked_add` for
    /// any in-spec bytecode. Mirrors JSC LLInt's `addp Imm, PC` pattern
    /// (no overflow trap).
    #[inline]
    pub(crate) fn advance(&mut self, n: u32) {
        let next = self.frame.instruction_offset().wrapping_add(n);
        self.frame.set_instruction_offset(next);
    }

    #[inline]
    pub(crate) fn code(&self) -> CodeRef {
        self.frame.code()
    }

    /// Write `self.frame` back to `vm.frames[frame_depth - 1]`. Used before
    /// any handler operation that may inspect the live frame stack
    /// (return-from-frame, debugger safepoints, etc.).
    #[inline]
    pub(crate) fn sync_active_frame(&mut self) {
        let frame_depth = self.frame_depth;
        let frame = self.frame;
        self.vm.sync_dispatch_frame(frame_depth, frame);
    }

    /// Pop the agent's top execution context. Mirror of the
    /// `let _ = agent.pop_execution_context();` line in the legacy match.
    #[inline]
    pub(crate) fn pop_execution_context(&mut self) {
        let _ = self.agent.pop_execution_context();
    }

    /// Wrap `Vm::finish_frame` with the split borrow of `vm` and `agent` that
    /// the borrow checker requires through `&mut DispatchState`.
    #[inline]
    pub(crate) fn finish_active_frame(&mut self, value: Value) -> VmResult<Option<Value>> {
        let DispatchState { vm, agent, .. } = self;
        vm.finish_frame(agent, value)
    }

    /// Read constant `bx` from the active function's constant pool. Splits
    /// the `&mut vm` + `&mut agent` borrow that `Vm::read_constant`
    /// requires.
    #[inline]
    pub(crate) fn read_constant(&mut self, bx: u32) -> VmResult<Value> {
        let code = self.frame.code();
        let DispatchState { vm, agent, .. } = self;
        vm.read_constant(agent, code, bx)
    }

    /// Route a possibly-abrupt operation result through the exception
    /// transfer machinery. Returns `Ok(Some(value))` for success,
    /// `Ok(None)` if the abrupt completion was caught by an active handler
    /// (the handler should `dispatch_next!` to continue at the new PC), or
    /// `Err(error)` if the abrupt completion escapes the current code.
    ///
    /// When a throw is caught and `transfer_to_exception_handler` unwound
    /// frames across the trampoline boundary (callee → caller handler), the
    /// snapshot fields in `self` (`frame`, `frame_depth`, `installed`) point
    /// at the dead callee. Detect that case and refresh from the now-active
    /// caller frame so the subsequent `dispatch_next!` reads from the right
    /// bytecode. Legacy `run_dispatch_loop` got this for free via the
    /// outer-loop re-read after `request_dispatch_frame_check`.
    #[inline]
    pub(crate) fn handle_dispatch_result<T>(&mut self, result: VmResult<T>) -> VmResult<Option<T>> {
        let was_throw = matches!(&result, Err(VmError::Abrupt(AbruptCompletion::Throw(_))));
        let outcome = {
            let DispatchState {
                vm,
                agent,
                frame,
                frame_depth,
                ..
            } = &mut *self;
            vm.handle_dispatch_result(agent, *frame_depth, frame, result)?
        };
        if was_throw && outcome.is_none() {
            self.refresh_from_active_frame()?;
        }
        Ok(outcome)
    }

    /// Re-snapshot frame/depth/installed/epoch after a frame-changing
    /// operation. Required after a return that didn't terminate the script
    /// (caller frame is now active) or after a call (callee frame is now
    /// active).
    pub(crate) fn refresh_from_active_frame(&mut self) -> VmResult<()> {
        self.frame_depth = self.vm.frames().len();
        let frame = self
            .vm
            .frames()
            .last()
            .copied()
            .ok_or(VmError::MissingActiveFrame)?;
        self.frame = frame;
        let code = frame.code();
        let installed = self
            .vm
            .installed_for_code(code)
            .ok_or(VmError::MissingInstalledCode(code))?;
        self.installed = installed;
        self.frame_check_epoch = self.vm.dispatch_frame_check_epoch();
        Ok(())
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
/// **Prefix handling invariant:** this macro does NOT clear `state.prefix`.
/// Handlers that consult the prefix (`op_move`, `op_load_*`, `op_jump_if_*`)
/// must consume it with `state.prefix.take()` to leave `None` for the next
/// handler. Narrow-only handlers ignore the field entirely; the bytecode
/// emitter guarantees they never run with a stale prefix set (every Wide /
/// ExtraWide is immediately followed by a prefix-aware semantic opcode).
/// This keeps the narrow hot path free of a per-dispatch store.
///
/// `dispatch_next!` is the *only* place in any handler body that should
/// reference `DISPATCH_TABLE` — Phase 1's acceptance criteria grep for this
/// invariant.
#[macro_export]
macro_rules! dispatch_next {
    ($state:expr) => {{
        let byte = $state.next_opcode_byte();
        $state.vm.maybe_record_opcode_dispatch(byte);
        #[cfg(debug_assertions)]
        $state
            .vm
            .assert_deopt_safepoint_state($state.agent, &$state.frame, &$state.installed);
        return $crate::vm::dispatch_state::Step::Continue(
            $crate::vm::dispatch_state::DISPATCH_TABLE[byte as usize],
        );
    }};
}

/// `?`-like early-return for handlers. `Result<T, VmError>` → `T` on Ok, or
/// `return Step::Error(e)` on Err.
#[macro_export]
macro_rules! try_step {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(error) => return $crate::vm::dispatch_state::Step::Error(error),
        }
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
    let first_byte = state.first_opcode_byte();
    state.vm.maybe_record_opcode_dispatch(first_byte);
    #[cfg(debug_assertions)]
    state
        .vm
        .assert_deopt_safepoint_state(state.agent, &state.frame, &state.installed);
    let mut handler = DISPATCH_TABLE[first_byte as usize];
    loop {
        match (handler)(state) {
            Step::Continue(next) => handler = next,
            Step::Done(value) => return Ok(value),
            Step::Error(error) => return Err(error),
        }
    }
}

impl Vm {
    /// Look up the `Arc<InstalledFunction>` for a given `CodeRef`. Used by
    /// `DispatchState::refresh_from_active_frame` after a frame transition.
    #[inline]
    pub(in crate::vm) fn installed_for_code(&self, code: CodeRef) -> Option<Arc<InstalledFunction>> {
        self.installed
            .get(code_index(code))
            .and_then(Option::as_ref)
            .cloned()
    }

    /// Read the for-in enumerator slot off the side table. Mirrors the
    /// legacy `self.for_in_states.advance(agent, base, register)` direct
    /// access from a trampoline-safe wrapper.
    #[inline]
    pub(in crate::vm) fn for_in_advance(
        &mut self,
        agent: &mut lyng_js_env::Agent,
        base: u32,
        register: u16,
    ) -> VmResult<Option<lyng_js_types::PropertyKey>> {
        self.for_in_states.advance(agent, base, register)
    }

    /// Insert an iterator enumerator into the for-in side table.
    #[inline]
    pub(in crate::vm) fn for_in_insert(
        &mut self,
        base: u32,
        register: u16,
        enumerator: lyng_js_ops::enumeration::ForInEnumerator,
    ) {
        self.for_in_states.insert(base, register, enumerator);
    }

    /// Drop the for-in enumerator at `register`.
    #[inline]
    pub(in crate::vm) fn for_in_remove(&mut self, base: u32, register: u16) {
        let _ = self.for_in_states.remove(base, register);
    }

    /// Insert an iterator state into the iterator side table.
    #[inline]
    pub(in crate::vm) fn iterator_insert(
        &mut self,
        base: u32,
        register: u16,
        iterator: lyng_js_ops::iterator::IteratorRecord,
    ) {
        self.iterator_states.insert(base, register, iterator);
    }

    /// Read the current exception (used by `LoadException`).
    #[inline]
    pub(in crate::vm) fn current_exception_value(&self) -> Value {
        self.current_exception().unwrap_or_else(Value::undefined)
    }

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
            prefix: None,
        };

        run_trampoline(&mut state)
    }
}
