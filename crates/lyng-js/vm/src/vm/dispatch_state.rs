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
    /// Frame-state refresh after a cross-frame catch happens in
    /// `run_trampoline`'s epoch check, not here — `Vm::handle_dispatch_result`
    /// bumps the dispatch-frame-check epoch via `request_dispatch_frame_check`,
    /// so the trampoline picks up the unwind on the next iteration.
    #[inline]
    pub(crate) fn handle_dispatch_result<T>(&mut self, result: VmResult<T>) -> VmResult<Option<T>> {
        let DispatchState {
            vm,
            agent,
            frame,
            frame_depth,
            ..
        } = self;
        vm.handle_dispatch_result(agent, *frame_depth, frame, result)
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
/// **Counter invariant (lyng-3uem T1):** opcode dispatch counting lives in
/// `run_trampoline_counted`, never in this macro. The hot dispatch path stays
/// free of the `state.vm.opcode_dispatch_counts.is_some()` check that used
/// to inline here. `run_trampoline` branches once per script entry to pick
/// between `run_trampoline_uncounted` (hot) and `run_trampoline_counted`
/// (instrumented).
///
/// `dispatch_next!` is the *only* place in any handler body that should
/// reference `DISPATCH_TABLE` — Phase 1's acceptance criteria grep for this
/// invariant.
#[macro_export]
macro_rules! dispatch_next {
    ($state:expr) => {{
        let byte = $state.next_opcode_byte();
        #[cfg(debug_assertions)]
        $state
            .vm
            .assert_deopt_safepoint_state($state.agent, &$state.frame, &$state.installed);
        return $crate::vm::dispatch_state::Step::Continue(
            $crate::vm::dispatch_state::DISPATCH_TABLE[byte as usize],
        );
    }};
}

/// Star-fusion variant of `dispatch_next!` for value-producing handlers
/// whose target register is the accumulator (`r0`).
///
/// Matches V8 Ignition's writer-side Star-fusion peephole
/// (`src/interpreter/interpreter-assembler.cc`): if the next byte is a
/// `StarN` opcode, the handler writes the just-produced value to register
/// `N` inline and advances past the `Star` byte before dispatching to the
/// instruction *after* it — eliminating one dispatch per fused pair.
///
/// Callers must have already written `value` to register 0; this macro
/// only performs the extra write to register `N`. Pass the value in a
/// local variable (not a side-effecting expression) — the macro evaluates
/// `$value` once on the fast path and not at all on the no-fusion path.
///
/// The SAFETY contract on `next_opcode_byte()` is satisfied transitively:
/// every `StarN` is followed by another valid opcode (Stars never appear
/// as a terminal instruction), so the second `next_opcode_byte()` after
/// `advance(1)` is also in-bounds.
#[macro_export]
macro_rules! dispatch_next_with_value {
    ($state:expr, $value:expr) => {{
        let byte = $state.next_opcode_byte();
        if let Some(target) =
            ::lyng_js_bytecode::Opcode::accumulator_store_index_for_byte(byte)
        {
            let registers = $state.frame.registers();
            $state.vm.write_register_unchecked(registers, target, $value);
            $state.advance(1);
            let next_byte = $state.next_opcode_byte();
            #[cfg(debug_assertions)]
            $state.vm.assert_deopt_safepoint_state(
                $state.agent,
                &$state.frame,
                &$state.installed,
            );
            return $crate::vm::dispatch_state::Step::Continue(
                $crate::vm::dispatch_state::DISPATCH_TABLE[next_byte as usize],
            );
        }
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

/// Central trampoline — the production dispatch loop. One indirect call
/// per opcode. The hot path is the `Step::Continue(next) => handler = next`
/// arm; `Done` and `Error` are taken once per script.
///
/// `state.vm` is hoisted into a raw pointer kept in a callee-saved register
/// across the loop (lyng-3uem T2). Without this, LLVM re-loads `state.vm`
/// every iteration because the indirect `blr handler` can clobber any state
/// field under the extern "C" ABI; manually pinning it saves one load per
/// dispatch. `local_epoch` mirrors `state.frame_check_epoch` in the same
/// way — synced to `state` only on the cold (epoch-changed) arm.
///
/// The opcode dispatch counter lives on a sibling path: when the
/// `opcode-counters` feature is enabled AND counters are turned on at
/// runtime, `Vm::run_via_trampoline` routes to `run_trampoline_counted`
/// instead. The hot path here never checks for it (lyng-3lqp).
///
/// Per the post-spike asm audit (`lyng-3uem`,
/// `reports/js/lyng-js/phase-1-diagnostics.md`), an earlier design inlined
/// `maybe_record_opcode_dispatch` into every `dispatch_next!` tail and cost
/// ~4 instructions per dispatch even when counters were `None`. The
/// current shape has zero counter cost on this path.
#[inline(never)]
pub fn run_trampoline(state: &mut DispatchState) -> VmResult<Value> {
    #[cfg(debug_assertions)]
    state
        .vm
        .assert_deopt_safepoint_state(state.agent, &state.frame, &state.installed);

    // T2 hoist: cache vm address + frame_check_epoch in locals so LLVM keeps
    // them in callee-saved registers across the indirect call. `vm_ptr`
    // aliases `state.vm` for the lifetime of this function call; no handler
    // reassigns `state.vm` (handlers receive `&mut DispatchState` only).
    let vm_ptr: *mut Vm = &raw mut *state.vm;
    let mut local_epoch = state.frame_check_epoch;

    let mut handler = DISPATCH_TABLE[state.first_opcode_byte() as usize];
    loop {
        match (handler)(state) {
            Step::Continue(next) => {
                // SAFETY: `vm_ptr` is stable for the lifetime of this call —
                // it was derived from `state.vm` at function entry and
                // handlers cannot reassign `state.vm` under Rust's borrow
                // rules. The handler call's extern "C" ABI clobbers caller-
                // saved registers but cannot mutate the local `vm_ptr` /
                // `local_epoch` slots.
                let vm_epoch = unsafe { (*vm_ptr).dispatch_frame_check_epoch() };
                if local_epoch != vm_epoch {
                    local_epoch = vm_epoch;
                    state.frame_check_epoch = vm_epoch;
                    if !still_active(state) {
                        state.refresh_from_active_frame()?;
                        local_epoch = state.frame_check_epoch;
                        handler = DISPATCH_TABLE[state.first_opcode_byte() as usize];
                        continue;
                    }
                }
                handler = next;
            }
            Step::Done(value) => return Ok(value),
            Step::Error(error) => return Err(error),
        }
    }
}

/// Instrumented dispatch loop — counters enabled. Records every dispatched
/// opcode by calling `maybe_record_opcode_dispatch` between handler
/// returns. The cost lives here so the hot path (`run_trampoline`) stays
/// clean. Not size-budgeted; only run when something has explicitly
/// enabled counters.
///
/// Gated behind the `opcode-counters` Cargo feature; absent from the
/// production binary entirely (`lyng-3lqp`). The runtime routing happens
/// in `Vm::run_via_trampoline`, not in a wrapper next to `run_trampoline`.
#[cfg(feature = "opcode-counters")]
#[inline(never)]
fn run_trampoline_counted(state: &mut DispatchState) -> VmResult<Value> {
    let first_byte = state.first_opcode_byte();
    state.vm.maybe_record_opcode_dispatch(first_byte);
    #[cfg(debug_assertions)]
    state
        .vm
        .assert_deopt_safepoint_state(state.agent, &state.frame, &state.installed);
    let mut handler = DISPATCH_TABLE[first_byte as usize];
    loop {
        match (handler)(state) {
            Step::Continue(next) => {
                // After the handler returns Continue, pc has been advanced
                // to the next opcode byte. Record that byte before
                // dispatching to its handler — same semantics as the
                // pre-split macro that recorded inside `dispatch_next!`.
                let next_byte = state.next_opcode_byte();
                state.vm.maybe_record_opcode_dispatch(next_byte);

                if state.frame_check_epoch != state.vm.dispatch_frame_check_epoch() {
                    state.frame_check_epoch = state.vm.dispatch_frame_check_epoch();
                    if !still_active(state) {
                        state.refresh_from_active_frame()?;
                        let byte = state.first_opcode_byte();
                        state.vm.maybe_record_opcode_dispatch(byte);
                        handler = DISPATCH_TABLE[byte as usize];
                        continue;
                    }
                }
                handler = next;
            }
            Step::Done(value) => return Ok(value),
            Step::Error(error) => return Err(error),
        }
    }
}

/// Frame-stack identity check shared by both trampoline variants.
///
/// The Vm bumps the epoch via `request_dispatch_frame_check` whenever frame
/// state changes (function call/return, exception transfer). When that
/// fires we have to decide: is `state.frame` still the active frame, or
/// did the underlying frame stack change?
///
/// - Property getter calls, builtin invocations, etc. bump the epoch but
///   leave `self.frames.last()` pointing at the same caller frame. The
///   handler-local pc advance in `state.frame` is the source of truth (the
///   legacy helpers don't write back to `self.frames` on success). DON'T
///   refresh — clobbering pc with `self.frames.last()` would revert the
///   advance and re-dispatch the same opcode (the bug behind a 30 GB OOM
///   in nested-call hot paths fixed in `fbace3dd`).
/// - Cross-frame exception transfer or a call's manual refresh (handlers
///   in `calls.rs`) bumps the epoch AND changes which frame is on top.
///   Frame depth / top-frame code differs from `state`'s snapshot —
///   caller refreshes and re-dispatches from the new active frame's pc.
#[inline]
fn still_active(state: &DispatchState) -> bool {
    state.frame_depth == state.vm.frames().len()
        && state
            .vm
            .frames()
            .last()
            .is_some_and(|f| f.code() == state.frame.code())
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
    /// frame, then hands control to `run_trampoline` (the production hot
    /// path) — or, when the `opcode-counters` feature is enabled AND the
    /// runtime counter is on, to `run_trampoline_counted` instead. The
    /// branch happens here, once per script invocation, so the hot dispatch
    /// loop never re-checks (lyng-3lqp).
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

        #[cfg(feature = "opcode-counters")]
        if state.vm.opcode_counter_enabled() {
            return run_trampoline_counted(&mut state);
        }
        run_trampoline(&mut state)
    }
}
