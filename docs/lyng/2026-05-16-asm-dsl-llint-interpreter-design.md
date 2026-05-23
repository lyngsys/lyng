# Design: asm-DSL LLInt-class interpreter (stable Rust, contained unsafe)

**Date:** 2026-05-16
**Status:** Design approved; ready for implementation planning.
**Supersedes (in spirit):** parts of [`reports/lyng/jsc-aligned-engine-roadmap.md`](../../reports/lyng/jsc-aligned-engine-roadmap.md) — specifically its Option α substrate commitment and Phase 1/Phase 3 acceptance criteria.
**Companion document:** [`reports/lyng/llint-parity-state-of-engine.md`](../../reports/lyng/llint-parity-state-of-engine.md) — the measurement-driven retrospective that motivated this design.

---

## 1. Context

The previous JSC-aligned roadmap (`lyng-49qk`) committed to Option α — a Rust `extern "C" fn` per-handler dispatch table with a central trampoline returning a `Step` enum. After landing Phase 1 through Phase 3f + Phase 4a-b, isolated bench numbers (`reports/lyng/external-engine-compare.md`) show lyng at:

- **5-12× slower than JSC LLInt** across V8 v7 (Richards 318 vs LLInt 1871).
- **1.6-3.7× slower than QuickJS** — the engine the original roadmap called "ceiling too low."

The retrospective established that α has a structural ~13× substrate overhead vs LLInt that no amount of IC layering can amortize. The substrate decision itself is the rate-limiting step.

This design specifies a new substrate path that targets **near-LLInt interpreter performance on stable Rust with contained unsafe.** It draws on JSC's LLInt as a reference architecture — its source is the gold-standard implementation of this design class — but lyng retains complete control over data layout, dispatch macros, and runtime decisions. No JSC code is vendored; we read JSC, understand the patterns, and re-implement them in our own Rust DSL.

The user's stance on unsafe: **anything stable Rust supports, if contained behind a clean macro layer.** The user's stance on time: **not a constraint** — the project has the resources to do this right.

## 2. Goals & non-goals

### Goals

- **Near-LLInt interpreter performance** on the V8 v7 suite — operationally defined as Richards ≥ 900 in isolated bench (within 2× of JSC LLInt's 1871).
- **100% Test262 conformance for Stage 3+ proposals, excluding intl402** — tracked as a parallel workstream that this design must not regress.
- **Stable Rust toolchain.** No nightly. `rustup default stable` builds the engine.
- **Contained unsafe.** Inline asm and `#[unsafe(naked)]` are acceptable when encapsulated by the DSL macro layer; handler authors never write `unsafe` directly.
- **Pure Rust build.** `cargo build` is the build. No Ruby, no Python, no external code generators. Standard Rust toolchain only.
- **Complete control over data layout.** Layout refactors are in scope when evidence supports them; we do not freeze sub-par designs because LLInt has different ones.

### Non-goals

- **JIT.** Out of scope for this design. The Baseline JIT (original Phases 5-6) remains valid in shape and stays deferred behind the interpreter milestone. We will not start JIT work until the interpreter substrate is complete and measured.
- **x86_64 support in the initial implementation.** Deferred until lyng has a concrete x86_64 user. AArch64 (Apple Silicon dev hardware) is the only initial target.
- **CI enforcement.** No CI infrastructure exists today; this design assumes developer-driven discipline (committed artifacts, manual baseline refresh). CI is a follow-up.
- **Adopting JSC's runtime data layout.** We read JSC as reference, not as a layout source-of-truth. Our `Value`, `Cell`, `Shape`, `FeedbackVector` remain our designs (subject to evidence-driven refactor during this work).

## 3. High-level approach

The dispatch substrate becomes a **Rust-native asm-DSL** that compiles handler bodies into `#[unsafe(naked)] extern "C" fn`s. Each handler's body is a single `core::arch::naked_asm!` block (not `core::arch::asm!`; naked functions on stable Rust require the dedicated `naked_asm!` macro, which forgoes `in/out/inout` operand constraints and instead relies on fixed registers, `sym` references for slow-path symbols, and `const` for compile-time values — see §7 for backend details).

The DSL is a proc-macro that parses handler source in offlineasm-flavored syntax (Flavor B; see §4) and emits per-arch inline assembly. State (PC, register-stack base, feedback-vector base, etc.) lives in pinned callee-saved registers across the entire interpreter — set up once at `Vm::run` entry, never spilled per dispatch. Dispatch is tail-jump (`br` / `jmp`), not call-return. The Step enum and the central trampoline both delete.

Handlers fall into three categories:

- **Hot opcodes (~25-30, by dispatch counter).** Full DSL bodies with fast paths inline. Match LLInt's handler shapes within a documented per-handler asm budget.
- **Warm opcodes (~1-3 today: `op_loop_header`, possibly `op_jump_loop`).** Full DSL bodies whose hot path includes a *mandatory* safepoint poll (GC, debugger, tier-up). Matches JSC's `op_loop_hint` shape: the poll is a branch on a flag, slow-path call on the rare arm. Categorically hot for icache placement; categorically poll-bearing for safepoint coverage. See §6 for the safepoint contract.
- **Cold opcodes (~118-120).** Three-line DSL stubs that delegate to Rust slow-path functions. Same dispatch shape, but the entire body is `call_slow!` + `dispatch_after_slow!`.

All three kinds use the same `llint_handler!` macro and the same dispatch table. The trampoline is gone; entry to the interpreter sets up pinned registers and tail-jumps to the first handler.

JSC's LLInt source (`Source/JavaScriptCore/llint/LowLevelInterpreter*.asm`) is the reference architecture: every ported handler has a side-by-side asm diff against JSC's matching handler. We deviate only with documented reason; the irreducible deltas caused by our `Value` layout (NaN-tagged with `ObjectRef` handles rather than pointer-identity cells) are explicitly acknowledged per-handler in the ported report rather than masked.

## 4. The DSL — Flavor B syntax

The DSL syntax is offlineasm-flavored — asm-shaped with Rust delimiters. Each statement maps to a small number of asm instructions. Labels and branches are explicit. No Rust control flow.

```rust
llint_handler! {
    op_add, layout = AbcSlot, length = 6, |a, b, c, slot| {
        load_reg!(b => t0);
        check_smi!(t0, .slow);
        load_reg!(c => t1);
        check_smi!(t1, .slow);
        add_smi_overflow!(t0, t1 => t2, .slow);
        store_reg!(a, t2);
        record_smi!(slot);
        dispatch!();

      .slow:
        call_slow!(op_add_slow_rs, args = [a, b, c, slot]);
        dispatch_after_slow!();
    }
}
```

The vocabulary matches the current `Value` layout (NaN-tagged with SMI variants, `ObjectRef = u32` handles for objects) rather than LLInt's pointer-identity-cell vocabulary. SMI checks use the actual tag masks documented in the R-0 value-layout report. `check_object_ref!` and `load_object_record!` are the equivalents of LLInt's `check_cell!` and cell-pointer dereferences. Pointer-identity Cells remain an evidence-driven later refactor (§9).

Slow paths are normal Rust:

```rust
#[no_mangle]
pub extern "C" fn op_add_slow_rs(
    state: *mut LlIntState,
    dst: u32, lhs: u32, rhs: u32, slot: u32,
) -> SlowPathReturn {
    // Reconstructs a safe LlIntDispatchState wrapper from state + rust_context,
    // then runs the Rust body — allocator, GC, Value coercion, etc. See §6 for
    // the full bridge protocol and shim layer.
}
```

Cold handlers are three lines:

```rust
llint_handler! {
    op_create_class, layout = AbcSlot, length = 6, |a, b, c, slot| {
        call_slow!(op_create_class_rs, args = [a, b, c, slot]);
        dispatch_after_slow!();
    }
}
```

Warm handlers (loop headers and backward-jump opcodes) interleave a fast safepoint poll with a slow-path bridge when the poll fires. The poll itself is a flag check off the pinned `VM` register (one load, no `LlIntState` indirection); the slow call is taken on the rare arm. `op_loop_header` is a *marker* that advances by encoded length — its `Ax` operand is currently unused (reserved for forward compat), and the handler does not jump:

```rust
llint_handler! {
    op_loop_header, layout = Ax, length = 4, |_unused_target_offset| {
        // Cheap flag check: VM.poll_pending is a same-thread u8 written by
        // GC / debugger scheduling during slow-path work. Common case is
        // `cbz` past the call.
        load_byte!(VM, #VM_POLL_PENDING_OFFSET => t0);
        branch_zero!(t0, .no_poll);

        call_slow!(op_loop_header_poll_rs, args = []);
        dispatch_after_slow!();

      .no_poll:
        dispatch!(advance = 4);
    }
}
```

Backward-jump handlers (`op_jump`, `op_jump8`, conditional jumps with negative offsets) follow the same shape but check sign of the offset first; only negative offsets trigger the poll branch.

**Why this syntax over Rust-idiomatic.** Direct mechanical translation from JSC's `LowLevelInterpreter64.asm` references. Authors can read JSC's matching handler in one window and write our DSL in another, line-by-line. The DSL's surface area matches the underlying machine, so there are no "I wrote Rust but the macro can't translate it" surprises. Onboarding cost is fixed (one register convention page, one operation vocabulary page) and amortizes across all hot opcodes.

## 5. Register-pin convention, `LlIntState` ABI, and `LlIntRustContext`

State that's accessed per-dispatch lives in callee-saved registers, pinned across the entire interpreter from `Vm::run` entry through every handler invocation. Asm-visible state lives in a `#[repr(C)] LlIntState` record sized for stable field offsets; everything that isn't asm-stable (trait objects, lifetimes, `Arc`s, Rust enums) lives in a Rust-only `LlIntRustContext` reached through a single thin pointer from `LlIntState`.

### `LlIntState` — asm-visible, fixed layout

```rust
/// Opaque marker for the Rust-side context pointer in `LlIntState`.
/// The asm layer never reads through this pointer. The cast back to the
/// real `LlIntRustContext<'vm>` lives only in `LlIntDispatchState::from_raw`.
#[repr(C)]
pub struct LlIntRustContextOpaque {
    _private: [u8; 0],
}

#[repr(C)]
pub struct LlIntState {
    // --- Per-frame snapshot mirrors of pinned registers (synced at every call_slow!) ---
    pub frame_pc_offset:    u32,          // pc - pb_base; synced from PC register before call_slow!
    pub _pad1:              u32,          // align rust_context to 8 bytes
    pub frame_pb_base:      *const u8,    // installed function's bytecode base
    pub frame_regs_base:    *mut Value,   // register-stack base for active frame
    pub frame_fv_base:      *mut FeedbackEntry, // feedback array base for active code (mutable — record_*!/value_profile! write through it)
    pub frame_depth:        u32,          // vm.frames().len() at snapshot
    pub frame_check_epoch:  u32,          // mirror of vm.dispatch_frame_check_epoch()

    // --- Single erased thin pointer to Rust-only state (asm only ever passes it through) ---
    pub rust_context:       *mut LlIntRustContextOpaque,

    // --- Per-instruction prefix flag (Wide / ExtraWide) ---
    pub prefix:             u8,           // 0 = none, 1 = Wide, 2 = ExtraWide
    pub _pad2:              [u8; 7],
}
```

All fields are thin pointers, integers, or padding. The size and field offsets are stable across rustc versions because no Rust enums, trait objects, or lifetime-bearing types appear in the layout. `rust_context` is typed as `*mut LlIntRustContextOpaque` (a zero-sized marker) precisely so the type system cannot accidentally treat the pointer as carrying a real lifetime; only `LlIntDispatchState::from_raw` casts it back to the concrete `*mut LlIntRustContext<'vm>` after the entry shim has established the lifetime contract.

Offsets are produced by `core::mem::offset_of!` and exposed as `const` items in `vm/src/dsl/reg_convention.rs`; the DSL backend references them via `const N` placeholders in `naked_asm!`. An offset-generation test verifies they match across rustc versions.

### `LlIntRustContext` — Rust-only, lifetime-bearing

```rust
// NOT #[repr(C)]. Naked asm NEVER reads this directly. Only slow-path Rust shims dereference it.
pub struct LlIntRustContext<'vm> {
    pub vm:        &'vm mut Vm,
    pub agent:     &'vm mut Agent,
    pub host:      &'vm dyn HostHooks,
    pub registry:  &'vm mut (dyn NativeFunctionRegistry + 'vm),
    pub installed: Arc<InstalledFunction>,
    pub frame:     FrameRecord,                  // FULL snapshot — every field: code, realm, env, etc.
    pub frame_depth: usize,
    pub exit:      LlIntExitSlot,                // exit-slot lives here, off the asm-visible record
}

pub struct LlIntExitSlot {
    pub kind:       ExitKind,
    pub done_value: Value,                       // set when kind == Done
    pub error:      Option<Box<VmError>>,         // set when kind == Error; bridge takes ownership
}

pub enum ExitKind {
    None,                                         // bridge has not yet observed Exit
    Done,                                         // Vm::run returns Ok(done_value)
    Error,                                        // Vm::run returns Err(*error.take().unwrap())
}
```

`VmError` is heap-allocated by the slow path on the error path (rare; allocation cost is invisible). `done_value` is the `Value` returned by a successful exit. The asm-side `Exit` tag is just a signal; the exit shim (in Rust) reads `rust_context.exit` to discriminate `Done`/`Error` and reconstruct `VmResult<Value>`.

### AArch64 mapping (initial target)

| Pin       | Register | Type                           | Refreshed when                                 |
| --------- | -------- | ------------------------------ | ---------------------------------------------- |
| `PC`      | x19      | `*const u8` (pointer, not offset; `pb_base + frame_pc_offset` materialized at entry) | call / return / exception unwind |
| `REGS`    | x20      | `*mut Value`                   | call / return                                  |
| `FV`      | x21      | `*mut FeedbackEntry`           | call / return                                  |
| `VM`      | x22      | `*mut Vm`                      | once per `Vm::run`; used for `[VM, #VM_POLL_PENDING_OFFSET]` reads on warm handlers |
| `TABLE`   | x23      | `*const Handler`               | once per `Vm::run`                             |
| `STATE`   | x24      | `*mut LlIntState`              | once per `Vm::run`; also moved to x0 for `call_slow!` |
| `t0..t6`  | x9–x15   | scratch (caller-saved)         | per-instruction                                |
| spare CSR | x25–x28  | available for handlers needing more pins | as needed                          |

### x86_64 mapping (deferred, but specified for forward-compat)

| Pin       | Register | Notes                                                                  |
| --------- | -------- | ---------------------------------------------------------------------- |
| `PC`      | r12      | callee-saved                                                           |
| `REGS`    | r13      | callee-saved                                                           |
| `FV`      | r14      | callee-saved                                                           |
| `VM`      | r15      | callee-saved                                                           |
| `STATE`   | rbx      | callee-saved (`*mut LlIntState`)                                       |
| `TABLE`   | RIP-relative | no register pin (RIP-relative is free on x86_64)                   |
| scratch   | rax, rcx, rdx, rsi, r8, r9, r10, r11 | caller-saved                                          |

### Two load-bearing choices

- **PC as pointer, not offset.** Pinned register `PC` holds `frame_pb_base + frame_pc_offset` materialized at entry and on `Refresh`. Dispatch is `ldrb [PC]; advance; load handler; jump` — 4 instrs total. PB doesn't need a register pin; it's reconstructable from `state.frame_pb_base` on the rare paths that need it (bounds checks at function boundaries, slow-path PC syncing).
- **`FV` as base pointer of a flat IC-entry array.** Mutable (`record_*!` and `value_profile!` write through it). Requires refactoring today's `NamedPropertyFeedback` (multiple fields + sidecars from Phase 3a/3e/3f) into a contiguous flat array of fixed-size entries. The flattening is about vector storage, not entry content — Phase 3f's packed sidecars stay inside each entry. See §9 for lifecycle.

### Refresh discipline (the load-bearing invariant)

Four layers of state, with explicit sync rules:

| Layer                                              | Truth-at                  | Synced from                              | Synced when                                                   |
| -------------------------------------------------- | ------------------------- | ---------------------------------------- | ------------------------------------------------------------- |
| Pinned registers (PC, REGS, FV)                     | hot path                  | `LlIntState.frame_*`                     | only on entry and `Refresh` — otherwise pinned regs are truth |
| `LlIntState.frame_*` (asm-visible mirrors)         | slow-path boundary        | pinned registers via pre-`call_slow!` sync | written before every `call_slow!`; read on `Refresh`         |
| `LlIntRustContext.frame` (full `FrameRecord`)      | slow-path Rust code       | pinned regs + `vm.frames.last()`         | written by entry shim and by frame-changing slow paths (Call/Return) |
| `vm.frames` (canonical)                            | always                    | slow paths that mutate                   | call/return slow paths push/pop here; bridge mirrors into `rust_context.frame` |

Slow-path shims own this sync. The contract is specified in detail in `llint-dsl-abi.md` (R-0 deliverable) with explicit invariant tests for each transition.

## 6. Slow-path bridge protocol

A slow path is a normal Rust `extern "C"` function. The bridge passes the asm-visible `*mut LlIntState`; the Rust shim reconstructs a safe `LlIntDispatchState<'_>` wrapper (combining `LlIntState` + `LlIntRustContext`) for slow-path code to use.

### Slow-path signature and shim layer

```rust
// Asm-facing signature — called directly from naked_asm! via `bl {slow_rs}` with `slow_rs = sym op_xxx_slow_rs`.
#[no_mangle]
pub extern "C" fn op_xxx_slow_rs(
    state: *mut LlIntState,      // a0 / rdi
    operand_0: u32,              // a1 / rsi
    operand_1: u32,              // a2 / rdx
    // ... up to operand_4 (a5 / r9)
) -> SlowPathReturn {
    // SAFETY: caller is the asm bridge which preserves the LlIntState
    // pointer's validity for the lifetime of this call. The state's
    // rust_context was established by the entry shim before any handler
    // ran and outlives this call.
    let mut dispatch = unsafe { LlIntDispatchState::from_raw(state) };
    // Sync asm-visible frame fields into the Rust-side snapshot before
    // semantic code observes the frame. See "Pre-slow-path sync" below.
    dispatch.sync_from_asm();
    let outcome = op_xxx_semantic(&mut dispatch, OpXxxArgs { operand_0, operand_1 });
    dispatch.translate_outcome(outcome)
}

// The semantic body — pure Rust, takes a safe wrapper, returns a logical outcome.
fn op_xxx_semantic(
    state: &mut LlIntDispatchState<'_>,
    args: OpXxxArgs,
) -> SemanticOutcome { /* normal Rust */ }
```

Implementation pattern: the asm-facing function is a small `unsafe` shim that constructs the wrapper, syncs state, calls the safe semantic body, translates the outcome into a `SlowPathReturn`. Both the shim and the wrapper live in `vm/src/dsl/slow_path.rs`; the unsafe is concentrated there. Semantic bodies are normal Rust functions that any opcode can call — the same function backs both the DSL cold stub (via the shim) and, transitionally during DSL-0a, the alpha handler.

### Pre-slow-path sync (the load-bearing protocol)

Before any semantic body reads from `state.frame.*`, the shim copies asm-visible mirrors into the Rust-side snapshot. Without this, a semantic body asking for `state.frame.instruction_offset()` would see stale data from the last `Refresh` rather than the post-dispatch PC the handler just set up.

Every slow-path shim follows this fixed sequence:

```text
1. Acquire LlIntState raw pointer (arg 0 / x0 / rdi).
2. Construct LlIntDispatchState<'_> via from_raw (casts rust_context back to the concrete type).
3. sync_from_asm():
   - rust_context.frame.instruction_offset  ← state.frame_pc_offset
   - rust_context.frame.registers_base       ← state.frame_regs_base   (via id translation if needed)
   - rust_context.frame.feedback_vector_base ← state.frame_fv_base     (similarly)
   - rust_context.frame.code_pb_base         ← state.frame_pb_base
   - rust_context.frame.depth_snapshot       ← state.frame_depth
4. Run op_xxx_semantic(&mut dispatch, args).
5. translate_outcome(outcome):
   For Continue { new_pc_offset }:        SlowPathReturn { tag: Continue, payload: new_pc_offset as u64 }
     PLUS sync_to_asm(new_pc_offset):    state.frame_pc_offset ← new_pc_offset
   For Refresh:                            sync_to_asm_full(): mirror all frame_* fields out of rust_context.frame
                                          SlowPathReturn { tag: Refresh, payload: 0 }
   For Exit { kind, payload }:             rust_context.exit ← { kind, payload }
                                          SlowPathReturn { tag: Exit, payload: 0 }
```

Invariant test: a cold-stub opcode whose semantic body reads `state.frame.instruction_offset()` and asserts it equals the byte immediately past the dispatching opcode. Verifies that step 3 actually ran. Lands as one of the DSL-0b validation cases.

Operands pass as `u32` (not `u8`/`u16`) to avoid extension dance in ABI registers.

**Unwind policy:** slow paths are `extern "C"`, which under modern Rust aborts on panic (the panic-abort contract). Slow paths must not panic. In debug builds, each shim wraps the semantic body in `std::panic::catch_unwind` and aborts cleanly with a diagnostic if a panic escapes. In release builds, `extern "C"`'s built-in panic-abort behavior is the safety net. **No path of control flow returns from a slow-path call via stack unwinding;** errors travel through `SlowPathTag::Exit` + `LlIntExitSlot`.

### Return ABI

Three *dispatch* tags. Exit outcomes (success vs error) discriminate via `rust_context.exit` rather than via tag:

```rust
#[repr(C)]
pub struct SlowPathReturn {
    pub tag: u64,      // SlowPathTag, returned in a0 / rax
    pub payload: u64,  // returned in a1 / rdx
}

#[repr(u64)]
pub enum SlowPathTag {
    Continue = 0,   // payload = new PC offset (u32-in-u64). Dispatch at new PC.
    Refresh  = 1,   // payload = unused. Reload PC/REGS/FV from state.frame_*, dispatch.
    Exit     = 2,   // payload = unused. Bridge jumps to _interpreter_exit; exit shim reads rust_context.exit.
}
```

`Exit` covers both `Ok(Value)` and `Err(VmError)`. The slow path writes `rust_context.exit.kind` to `Done` (with `done_value` filled) or `Error` (with `error` filled with `Some(Box::new(err))`), then returns `Exit`. The exit shim reads `rust_context.exit` and constructs the appropriate `VmResult<Value>`.

Uncaught guest exceptions take the `Error` path with `VmError::Abrupt(AbruptCompletion::Throw(value))`. VM-internal errors take the same path with their own variant. Caught guest exceptions still use `Refresh`: the slow path runs `Vm::transfer_to_exception_handler`, updates `rust_context.frame` and `state.frame_pc_offset` to the catch handler's PC, and returns `Refresh` so the bridge resumes at the handler.

### Bridge protocol (AArch64 sketch — operand-bound)

The actual `naked_asm!` block uses operand bindings, not textual symbol names:

```text
; Inside naked_asm! with bindings:
;   slow_rs        = sym op_xxx_slow_rs
;   state_pb_off   = const offset_of!(LlIntState, frame_pb_base)
;   state_pc_off   = const offset_of!(LlIntState, frame_pc_offset)
;   state_regs_off = const offset_of!(LlIntState, frame_regs_base)
;   state_fv_off   = const offset_of!(LlIntState, frame_fv_base)

; Pre-call: sync PC offset into state.frame_pc_offset
ldr  x9,  [x24, {state_pb_off}]
sub  x10, x19, x9                                ; PC pointer → offset
str  w10, [x24, {state_pc_off}]

; Move pinned STATE into arg 0, operands into a1..aN
mov  x0,  x24                                    ; STATE → a0
mov  w1,  <operand a>
mov  w2,  <operand b>
; ...

bl   {slow_rs}                                    ; sym-bound call; platform symbol mangling handled by rustc

; Post-call dispatch — Continue is the common case
cbnz x0,  .unusual                               ; tag != Continue
ldr  x9,  [x24, {state_pb_off}]
add  x19, x9,  x1                                ; PC = pb_base + new_offset
ldrb w8,  [x19]
ldr  x10, [x23, x8, lsl #3]
br   x10                                          ; tail-dispatch

.unusual:
cmp  x0,  #2
b.eq .exit
; Refresh: reload everything from state.frame_*
ldr  w11, [x24, {state_pc_off}]
ldr  x9,  [x24, {state_pb_off}]
add  x19, x9,  x11
ldr  x20, [x24, {state_regs_off}]
ldr  x21, [x24, {state_fv_off}]
ldrb w8,  [x19]
ldr  x10, [x23, x8, lsl #3]
br   x10

.exit:
b    {interpreter_exit}                           ; sym-bound exit shim
```

Common-case bridge cost (Continue): **~10-15 instructions** including pre-call sync. Matches LLInt's `callSlowPath` + `dispatch()`.

### Frame transitions

`op_call_*` and `op_return` slow paths always return `Refresh` because they alter frames. They are responsible for updating `rust_context.frame` to the new active frame before returning (the bridge mirrors `rust_context.frame.*` into `state.frame_*` for the next dispatch's pinned-register reload). Other hot/warm slow paths return `Continue`; uncaught exception and `op_return`-from-top-frame return `Exit`.

### GC safepoints and the warm-opcode poll model

GC runs at `call_slow!` boundaries — never inside a handler body's straight-line asm. Inside a handler, scratch registers may hold raw cell pointers, partial Values, intermediate computations — not roots. At a slow-path call, all live Values are in `REGS` (the register stack), reachable from `rust_context.frame.registers_range()`. This is LLInt's safepoint model.

To prevent GC starvation in tight all-fast loops (e.g., `op_add` + `op_jump`-back), safepoint coverage is preserved by **all backedge-bearing handlers** — `op_loop_header`, `op_jump`/`op_jump8` (when the offset is negative), and conditional jump handlers (when the taken offset is negative). Each is warm: the DSL body emits a cheap poll check (`ldrb [VM, {VM_POLL_PENDING_OFFSET}]; cbz`) and branches to a `call_slow!` only when the flag is set.

**`poll_pending` lifecycle (same-thread, DSL-0 scope):**

- **Storage**: `Vm.poll_pending: u8` — a plain byte. The agent-thread VM state is not currently `Send`/`Sync`, the debugger API is `&mut self`, and the DSL substrate inherits those constraints. Read by the asm path via `ldrb [VM, {VM_POLL_PENDING_OFFSET}]` directly — `VM` is pinned in x22/r15, so the read is one instruction with no `LlIntState` indirection.
- **Producers (same-thread only; each sets a distinct bit):**
  - `GC_PENDING (0x01)` — incremental marking scheduler when work is due; major-collection scheduler when a collection should happen. Set during slow-path execution.
  - `DEBUG_PAUSE (0x02)` — debugger when a pause is requested via the existing `&mut self` APIs (`Vm::request_debug_pause`, `request_debug_pause_at`). Set during slow-path execution or between `Vm::run` invocations.
- **Consumers**: the slow-path entries for warm handlers (`op_loop_header_poll_rs` and equivalents for backward-jump handlers). The slow path reads the bitfield, runs the relevant work (GC step, debugger pause), and clears the consumed bits.
- **Out of scope for DSL-0:** cross-thread debugger pause requests, tier-up scheduling (the JIT track is deferred per §2; tier accounting goes away with the alpha path in DSL-0c — see §10), any other concurrency-bearing producer. If cross-thread debugger control is wanted later, that gets its own design that addresses the full synchronization surface (hook handoff, pause request payload, memory ordering, asm-side atomic load semantics).
- **Memory model:** because all access is single-threaded, the asm-side `ldrb` and the Rust-side `u8` writes need no memory-ordering machinery. If DSL-0 ever needs to be made thread-safe, the byte upgrades to `AtomicU8`, the asm-side load is documented as a non-atomic but acquire-equivalent read of an atomic location (architecturally fine on AArch64/x86_64 for byte loads but explicitly justified in writing), and the cross-thread protocol gets its own ticket.

The producer model is specified in detail in `llint-dsl-safepoints.md` (R-0 deliverable), including invariant tests that verify backward unconditional jumps, conditional backward jumps, and `op_jump8` all reach their poll under contrived poll-flag-always-set scenarios.

**Implication for handler authors:** scratch registers carrying Values must not survive across `call_slow!`. True by construction (caller-saved registers are clobbered).

### Prefix dispatch (`op_wide`, `op_extra_wide`)

Prefix opcodes are dispatch-shape exceptions, not ordinary cold stubs. They read the *semantic* opcode byte at `pc + 1`, reject doubled prefixes, set `LlIntState.prefix`, and tail-dispatch directly to the semantic handler without consuming the semantic opcode through normal layout decoding. The semantic handler observes `state.prefix != 0`, decodes its operands at the wider width, advances PC past both the prefix and the wider-operand instruction body, and clears `state.prefix` to 0 before tail-dispatching.

DSL provides a `dispatch_prefixed!(kind)` operation that emits:
- `branch_nonzero!(state.prefix, .double_prefix_error)` — reject doubled prefix.
- `store_byte!(state.prefix, #PREFIX_VALUE_FOR_KIND)`.
- `advance!(1)` — past the prefix byte itself.
- `ldrb [PC], t_scratch; load handler from TABLE; br` — direct dispatch to the semantic handler at the new PC.

`op_wide` and `op_extra_wide` are **warm** handlers (small bodies, not cold stubs). Three prefix cases land as DSL-0b validation tests: Wide-prefixed `op_move` decodes wide operands correctly; ExtraWide-prefixed `op_move` likewise; double-prefix rejection raises the expected `VmError`. Semantic handlers' layout decoders must explicitly read and clear `state.prefix`; this is enforced by the layout-decoding macros generated by `llint_handler!` so handler authors never write it by hand.

The single-implementation invariant (§10) includes prefix decoding: prefix logic must not split between alpha and DSL.

### Optimization to surface in DSL-0

`op_return` Refresh overhead vs Continue: 5 extra instructions × ~1-2% dispatch share = ~25-100k extra instructions per workload. May be visible. If so: add a "fast return to same code unit" shortcut. Decide based on DSL-0 measurement.

## 7. Per-arch backend structure

### Crate and module layout

```
crates/vm-dsl/                  -- proc-macro crate, new
├── Cargo.toml                          -- proc-macro = true
└── src/
    ├── lib.rs                          -- llint_handler! entry point
    ├── parse.rs                        -- syn-based body parser
    ├── layouts.rs                      -- operand-layout descriptors
    └── lower.rs                        -- AST → asm string assembly

crates/vm/src/dsl/              -- runtime-side support
├── mod.rs                              -- re-exports llint_handler!, declares backend
├── reg_convention.rs                   -- pinned-register docs + LlIntState const offsets
├── llint_state.rs                      -- LlIntState repr(C), LlIntRustContext, LlIntExitSlot, ExitKind
├── entry.rs                            -- Vm::run_via_trampoline shim, _interpreter_exit
├── slow_path.rs                        -- SlowPathReturn, SlowPathTag, LlIntDispatchState wrapper, shim helpers
├── ops/
│   ├── mod.rs                          -- DSL vocabulary (re-exports per-arch)
│   └── ops.md                          -- vocabulary documentation
└── backend/
    ├── mod.rs                          -- cfg-dispatches to aarch64
    └── aarch64/
        ├── mod.rs                      -- exports all op macros
        ├── prelude.rs                  -- shared constants (SMI tag masks, exit-slot offsets, ...)
        ├── operands.rs                 -- load_reg!, store_reg!, ...
        ├── values.rs                   -- check_smi!, check_object_ref!, untag_smi!, tag_*!, ...
        ├── objects.rs                  -- load_object_record!, load_shape!, load_inline_slot!, ...
        ├── arithmetic.rs               -- add_smi_overflow!, ...
        ├── control.rs                  -- dispatch!, call_slow!, branch_*!, ...
        └── feedback.rs                 -- load_feedback_site!, value_profile!, ...
```

x86_64 directory added when DSL-3 activates.

### Macro implementation: proc-macro + macro_rules!

Two layers:

- **Top level (`llint_handler!`):** a proc-macro using `syn`. Parses the handler signature and body into structured AST. Generates operand-decoding prologue from layout. Walks body statements, asks per-operation macros for their asm fragments, concatenates into one `core::arch::naked_asm!` block. Emits `#[unsafe(naked)] extern "C" fn`. The `naked_asm!` macro (distinct from `asm!`) is the only legal inline-asm form inside naked functions on stable Rust — it forbids `in/out/inout` operand constraints and supports only `sym` (for symbols like slow-path function names) and `const` (for compile-time integers like field offsets); operand and state passing happens via the pinned-register convention (§5).
- **Per-operation macros:** `macro_rules!` per arch, gated by `#[cfg(target_arch = ...)]`. Produce asm string fragments via `concat!`. Constants referenced in the fragments (field offsets, tag masks) are resolved via `const` items defined in `reg_convention.rs` and `backend/<arch>/prelude.rs`, then materialized inside the `naked_asm!` block as `const` placeholders.

The proc-macro internally has a compile-time scratch register allocator (~200 lines): allocates `t0..t6` to operand variables in declaration order, allocates additional scratch as operations request, errors if scratch demand exceeds per-arch budget.

### DSL operation vocabulary (~40 operations)

Grouped by category. Names reflect the current `Value` layout (SMI tags, `ObjectRef` handles) rather than LLInt's pointer-identity-cell vocabulary; pointer-identity cells become an evidence-driven later refactor (§9).

- **Operand decoding** (auto-generated by `layout =`): `decode_abc!`, `decode_abc_slot!`, `decode_abx!`, `decode_ax!`, `decode_call_range!`, `decode_accumulator!`.
- **Register-file access:** `load_reg!`, `store_reg!`, `load_acc!`, `store_acc!`.
- **Value tag checks / tag manipulation (NaN-tagged Value layout):** `check_smi!`, `check_object_ref!`, `check_undefined!`, `check_null!`, `check_bool!`, `check_double!`, `untag_smi!`, `untag_object_ref!`, `tag_smi!`, `tag_object_ref!`, `tag_undefined!`. Each is implemented against the exact masks documented in the R-0 value-layout report.
- **Object access via `ObjectRef`:** `load_object_record!` (resolves `ObjectRef` to `*const ObjectRecord` via the heap pool), `load_record_shape!`, `load_record_inline_slot!`, `load_record_outline_slots!`, `load_outline_slot!`. (When pointer-identity cells land as a later refactor, this group renames to `load_cell_*!` with simpler asm.)
- **Feedback site access:** `load_feedback_site!`, `load_site_field!` (typed accessor).
- **Arithmetic (SMI fast paths):** `add_smi_overflow!`, `sub_smi_overflow!`, `mul_smi_overflow!`, bitwise (`bit_and_smi!`, `bit_or_smi!`, `bit_xor_smi!`), shifts.
- **Branching:** `branch_eq!`, `branch_ne!`, `branch_zero!`, `branch_nonzero!`.
- **Dispatch:** `dispatch!()` (auto-advance), `dispatch!(advance = N)`, `dispatch!(jump_to = expr)`.
- **Slow-path bridge:** `call_slow!`, `dispatch_after_slow!`.
- **Feedback recording:** `record_smi!`, `record_double!`, `record_object_ref!`, `record_string!`, `value_profile!`.
- **Safepoint poll** (for warm handlers): `poll_safepoint!` — emits `load_byte!(STATE, #LLINTSTATE_POLL_PENDING => t_scratch); branch_nonzero!(t_scratch, .label)`.
- **Direct byte / field access** (compose with above for warm handlers and raw asm needs): `load_byte!`, `load_word!`, `load_quad!`, `store_byte!`, `store_word!`, `store_quad!`.

### Escape hatch

```rust
raw_asm!("/* arch-specific asm */")  // requires #[cfg(target_arch)] siblings per arch
```

Each use requires a justification comment. Discipline: on the third occurrence of a `raw_asm!` pattern, promote it to a DSL macro in `backend/<arch>/`.

### Per-arch divergence policy

- **DSL surface is identical.** Every operation exists on every supported arch with the same name and semantics. Arch-specific behavior goes in `raw_asm!`, not in different operation sets.
- **Asm shape is informational.** A `check_smi!` may emit 1 instruction on AArch64 and 2 on x86_64. Per-arch baselines are committed separately. Behavioral tests cover both.

## 8. Build pipeline

The build is `cargo build`. No external tools, no code generators, no Ruby. The proc-macro expands at compile time; naked functions emit inline asm via `naked_asm!`; rustc + LLVM assemble. The infrastructure that matters is *around* the build.

### `cargo asm` automation

A new subcommand of the existing `lyng-bench` tool:

```sh
cargo run --release -p lyng-bench -- asm-diff \
  --opcodes-config tools/lyng-bench/hot-opcodes.toml \
  --baseline       reports/lyng/dsl-asm-baseline-aarch64/ \
  --output         /tmp/asm-current/ \
  --mode           check    # or `update`
```

Reads the hot-opcode list, captures the per-handler asm, normalizes, diffs against the committed baseline. Per-opcode instruction-count budgets in the config file.

**Asm capture strategy.** `cargo-asm` (the third-party crate) is the convenient developer-facing tool but is not a Cargo build dependency. The subcommand shells out to whichever capture mechanism is available:

1. `cargo asm` if installed (`cargo install cargo-asm`).
2. Fallback: `cargo rustc --release -p lyng-vm -- --emit=asm` and parse the resulting `.s` files for the named symbols. This path has no external-crate dependency.

The subcommand prefers (1) for convenience, falls back to (2) automatically, and supports `--capture-mode=rustc` for the deterministic path.

**Normalization is spec'd up front.** Before any baseline is captured, the normalization rules are committed to `reports/lyng/dsl-asm-baseline-aarch64/NORMALIZATION.md`:

- Strip `.cfi_*` directives and other debug metadata.
- Strip file/line/column comments (`# /path/to/file.rs:N:C`).
- Rename label symbols to positional aliases: `Lfunc_begin390` → `L0`, `LBB123_4` → `L1`, in order of first appearance.
- Strip jump-table padding (`.p2align N` lines outside instruction blocks).
- Strip literal-pool comments (the `; <constant_value>` annotations on `ldr` immediates).
- Preserve: instruction mnemonics, operands, branch directions (forward/backward labels), label structure (relative ordering), CFG markers (entries / exits).

This rule set is the single source of truth for baseline diffs; changes to it require a separate explicit commit with rationale.

### Asm baselines: in-repo

```
reports/lyng/dsl-asm-baseline-aarch64/
├── op_add.asm
├── op_move.asm
├── op_get_named_property.asm
└── ...
```

Small text files (0.5-5 KB each). Diffable in git. Reviewers read them in change reviews. The single most important artifact for "more science-based" — every claim about asm shape becomes a file the reader can check.

### LLInt reference capture (one-shot tool)

```sh
cargo run --release -p lyng-bench -- capture-llint \
  --source [auto|system|local|excerpt] \
  --jsc-binary <path>                    # required for system/local
  --jsc-source <path>                    # required for excerpt mode
  --opcodes op_get_by_id,op_put_by_id,op_add,op_mov,op_jmp,op_call,op_ret \
  --output  reports/lyng/llint-reference/
```

Three source modes for resilience:

- **`system`**: invokes `otool -tvV` (macOS) / `objdump -d` (Linux) on the system JSC binary. Finds `_llint_op_*` symbols. **Falls through to `local` if symbols are stripped** (some macOS releases ship stripped JavaScriptCore).
- **`local`**: same approach but on a locally built `JavaScriptCore` (WebKit checkout at known path). Required for users on stripped systems. Documented setup steps in `docs/lyng/llint-reference-setup.md` produced as part of R-0.
- **`excerpt`**: extracts handler bodies from JSC's `LowLevelInterpreter*.asm` source files directly via offlineasm-aware parsing. Produces *source-level* reference (offlineasm pseudo-code) rather than concrete asm. Always available; used as a last-resort fallback.
- **`auto`** (default): tries `system`, falls back to `local`, falls back to `excerpt`. Reports which mode produced each opcode in the output.

Reference material; re-captured only when JSC ships a major version or when our capture tooling improves. Not gated on every dev run.

### Per-handler ported reports

For each DSL handler we author:

```
reports/lyng/dsl-handlers/
├── op_add.md
├── op_get_named_property.md
└── ...
```

Each contains:

- DSL source (excerpt with link).
- Current asm output (both arches when applicable).
- LLInt reference asm.
- Side-by-side annotated diff: which instructions match, which don't, why.
- Microbench results (ns/dispatch).
- Behavioral test references.

These are the audit artifact per handler. They replace today's `phase-N-*.asm` files.

### Microbench harness

```sh
cargo run --release -p lyng-bench -- microbench \
  --opcodes-config tools/lyng-bench/hot-opcodes.toml \
  --baseline       reports/lyng/microbench-baseline.md \
  --samples        7 \
  --iters          5000000 \
  --require-isolation
```

Each opcode has a hot-loop test case (a JS function compiled to bytecode exercising that opcode). 7-sample median ns/dispatch. `--require-isolation` checks loadavg before starting; aborts if > 2.0.

### Verification cadence (no CI)

Discipline is developer-driven, artifact-based. Per change:

1. Build with `cargo build --release`.
2. Run `cargo run -p lyng-bench -- asm-diff --check`.
3. If a hot opcode's asm changed, run `cargo run -p lyng-bench -- microbench` for the affected opcodes.
4. If touching IC / property / arithmetic: run isolated V8 v7 sweep.
5. Run `cargo test -p lyng-vm -p lyng-tests` for behavioral coverage.
6. Run focused Test262 slice; whole-corpus for substantive changes.
7. Commit asm baselines + ported reports + handler source + bench reports together.

Manual baseline refresh: when an asm change is unrelated (rustc upgrade shifts labels), `--mode update` regenerates baselines; commit message includes `[asm-baseline-refresh: <reason>]` so the change is visible in history.

CI is a follow-up. When it lands, it will automate steps 1-2 and 5-6 on every push; step 3-4 stays on developer's bench machine (or a dedicated bench runner) due to isolation requirements.

### Build dependencies

| Dependency               | Required for             | Notes                                          |
| ------------------------ | ------------------------ | ---------------------------------------------- |
| Rust stable toolchain    | Everything                | rustup; `naked_asm!` requires ≥ 1.88            |
| `cargo asm` (CLI tool)    | asm-diff convenience path | `cargo install cargo-asm`; **not a Cargo.toml dep**. The asm-diff subcommand falls back to `cargo rustc -- --emit=asm` if absent. |
| `otool` / `objdump`      | LLInt capture (one-shot)  | system tools, dev-only; `local` mode requires a WebKit checkout |
| `mach_absolute_time` / `perf_event_open` | Microbench timing | OS API                                |
| **No Ruby, no Python, no offlineasm, no external code generators.** | | The DSL is `macro_rules!` + proc-macro, expanded at compile time. |

## 9. Data-layout refactors in scope

Sub-par data layouts are not frozen. The dispatch refactor is the natural moment to fix layouts that have been wrong but weren't worth touching in isolation. Each refactor is its own ticket with its own evidence requirement (asm shows the problem; refactor solves it; microbench confirms). No batched "do all layout work at once" phase.

Expected surfaces (we'll learn more during DSL-0):

| Refactor                                                          | Surfaces during        | Estimated effort | Motivation                                                          |
| ----------------------------------------------------------------- | ---------------------- | ----------------:| ------------------------------------------------------------------- |
| `FeedbackVector` flat-array layout                                | DSL-0 (`op_add`)       | ~1 week          | Required for the `FV` register pin to work; see lifecycle below     |
| IC packed-handler representation (mode-byte + flat blocks)        | DSL-1 (`op_get_named_property`) | ~1-2 weeks | Collapses Phase 3a/3e/3f layered fast paths into LLInt-style mode dispatch |
| Pointer-identity cells (`ObjectRef = u32` → `*mut Cell`)          | DSL-1 (IC opcodes) or DSL-3 | ~3-4 weeks  | Eliminates side-table indirection; one fewer load per cell access. Evidence-driven — refactor only if DSL-0/1 asm-diff shows the indirection is the irreducible delta. |
| `Cell` 8-byte header layout (JSC-equivalent)                       | DSL-1 (if needed)      | ~2-3 weeks       | If asm-diff shows our object-record access is multi-instruction where LLInt is one |
| `Shape` transition representation                                  | DSL-1 (if hot)         | TBD; evaluate when porting `op_get_named_property` / IC opcodes | Audit during porting; refactor only if evidence supports |

The `Value` layout verification is **not a refactor** — it's an R-0 deliverable. R-0 produces `reports/lyng/llint-dsl-value-layout.md` documenting exact masks (`notCellMask`-equivalent, SMI tag bits, double encoding offset, undefined/null/bool encodings), expected asm sequences per check/tag operation on AArch64, and any irreducible deltas vs LLInt. The DSL backend's `check_*!` / `tag_*!` macros are implemented against this report. If the report uncovers a Value-layout problem severe enough to need a refactor, that becomes a separate ticket; DSL-0 does not start until the report exists.

### `FeedbackVector` flat-array lifecycle

Today's `FeedbackVector` is `Vec<Option<FeedbackSiteState>>` with lazy allocation, large enum variants, and Phase 3f packed sidecars on the per-site structures. The DSL substrate needs a *flat array* of fixed-size IC entries pinned in the `FV` register. Decisions for the refactor:

- **Eager allocation at code installation.** When `InstalledFunction` is constructed from compiled bytecode, the flat IC-entry array is allocated to the bytecode's feedback-slot count. Slow paths never grow it at runtime.
- **Pointer stability for the installed function's lifetime.** Storage is `Box<[FeedbackEntry]>` (or equivalent fixed-storage). The `FV` pin survives any slow-path call, including allocations and GC.
- **Per-entry content unchanged.** Phase 3f's packed monomorphic/proto/polymorphic sidecars stay inside each `FeedbackEntry`. The flattening is about *vector storage*, not *entry content*. Existing IC fast-path evidence (`reports/lyng/phase-3f-status.md`) carries forward.
- **Closures sharing code share `FeedbackEntry` storage.** Consistent with today's per-function-not-per-closure feedback. Pin pointer is per-active-frame, refreshed on call/return.
- **Compatibility with current rich state.** The flat array becomes the hot path. The legacy `FeedbackVector` may stay as a side-table for rare-but-rich state (e.g., recent observation types, tier-up counters that don't fit in the IC entry). Decide which fields move into the entry vs stay on the side-table based on which the DSL hot paths read.

The refactor lands as a DSL-0 prerequisite (the `FV` pin won't work without it). Estimated ~1 week of focused work.

## 10. Phasing & exit criteria

Four phases (DSL-2 absorbed into DSL-0; DSL-3 deferred). AArch64-only throughout. Single-dev cadence. Developer-driven discipline (no CI gates).

### R-0: Tooling and evidence reports (~3-4 weeks)

The measurement infrastructure the original roadmap needed and lacked, plus three evidence reports that have to exist before any DSL handler is written.

**Tooling deliverables:**

- `lyng-bench microbench` subcommand: per-opcode ns/dispatch with confidence interval, isolated execution, loadavg gate.
- `lyng-bench asm-diff` subcommand: cargo-asm/`rustc --emit=asm` capture, normalization, baseline comparison, budget enforcement.
- `lyng-bench capture-llint` subcommand: auto/system/local/excerpt source modes (§8).
- **Slow-path-share counter mode** (the DSL-1 invariant's leading indicator). `lyng-bench` subcommands gain a `--count-slow-path-share` flag behind the existing `opcode-counters` Cargo feature. Each `call_slow!` invocation increments a per-opcode counter; counters are separated into `slow_path_semantic` (for `op_xxx_slow_rs` shims invoked from cold stubs or hot-handler fall-back) and `slow_path_safepoint` (for warm-handler poll bridges like `op_loop_header_poll_rs`). The DSL-1 invariant uses only `slow_path_semantic`; safepoint polls fire when scheduled and don't reflect fast-path failure. Counter mode is feature-flagged so the un-instrumented binary runs benchmarks without perturbation. Output format compatible with `tools/lyng-bench/hot-opcodes.toml`'s per-opcode `target_slow_path_share` thresholds (default 0.20, per-opcode waiver supported).
- **Opcode-counter mode under naked dispatch.** Current opcode counter is updated by the trampoline; once dispatch is tail-jumped, the counter needs a new insertion point. Decision: build-time feature flag with conditional DSL emission. When `--features opcode-counters` is set, the `llint_handler!` proc-macro emits an extra `inc_counter!` op at the start of every handler (per-arch: ~3 instrs of `ldr/add/str` against a counter array keyed by opcode ID). When the feature is off, the proc-macro emits a no-op — zero per-dispatch overhead. Slow-path-share counters share the same feature flag with separate counter banks.

**Configuration and baselines committed:**

- `tools/lyng-bench/hot-opcodes.toml`: top-30 opcodes by *measured* dispatch count from running the opcode counter on V8 v7 + internal benchmarks. Not guessed. Per-opcode `target_slow_path_share` thresholds (default 0.20).
- `reports/lyng/llint-reference/`: LLInt reference asm/source for the top-30 opcodes (whichever capture mode succeeds per-opcode).
- `reports/lyng/microbench-baseline.md`: current ns/dispatch baseline before any DSL work.
- `reports/lyng/dsl-asm-baseline-aarch64/NORMALIZATION.md`: the normalization rule set committed before any baselines.

**Evidence reports (the three the review called out):**

- `reports/lyng/llint-dsl-value-layout.md`: exact current `Value` layout (NaN tag header, kind bits, payload encoding), masks for SMI/cell-ref/undefined/null/bool/double checks, expected AArch64 asm sequences for each `check_*!` and `tag_*!` operation, irreducible deltas vs LLInt's pointer-identity-cell vocabulary, decision: DSL-0 uses current tags. This report is the source of truth for the `values.rs` macros in the DSL backend.
- `reports/lyng/llint-dsl-abi.md`: `LlIntState` and `LlIntRustContext` layouts with field offsets, the `LlIntRustContextOpaque` erased pointer pattern, pinned-register convention with const expressions, slow-path return ABI, exit-slot protocol, entry shim (`Vm::run_via_trampoline` builds both records, points `state.rust_context` at the opaque-typed pointer), exit shim (`_interpreter_exit` reads `rust_context.exit`), pre-slow-path sync protocol (§6) with explicit per-shim sequence, four-layer state sync rules (pinned regs / `LlIntState.frame_*` / `LlIntRustContext.frame` / `vm.frames`), offset-generation tests, Miri tests on the slow-path shim layer, the PC-sync invariant test that confirms a semantic body sees the post-dispatch PC.
- `reports/lyng/llint-dsl-safepoints.md`: GC poll semantics under the DSL substrate (same-thread `Vm.poll_pending: u8`, `GC_PENDING | DEBUG_PAUSE` bits only — tier accounting explicitly deferred from DSL-0 per §10), debugger pause integration via existing `&mut self` APIs, prefix dispatch semantics (`op_wide` / `op_extra_wide`), warm-opcode safepoint coverage, invariant tests for: tight `op_add` + `op_loop_header` loop reaches poll; tight backward `op_jump` loop (no loop header) reaches poll; conditional backward-jump loop reaches poll; Wide-prefixed instruction decodes correctly; double-prefix raises expected error.

**Exit criteria:**

1. All listed subcommands work end-to-end and produce deterministic reports across 5 consecutive runs (output is byte-identical or differs only in timestamps).
2. The config/baseline files and three evidence reports are committed.
3. The hot-opcodes config reflects *measured* dispatch shares, not guesses.
4. Slow-path-share counter mode produces sane per-opcode counts on a Richards run (no NaN/overflow, all opcodes counted, semantic vs safepoint cleanly separated).

**No DSL handler work during R-0.** Tooling and evidence first.

### DSL-0: Spike + semantic extraction + full opcode coverage (~8-10 weeks)

The single experiment that decides whether the asm-DSL path is viable, executed against full opcode coverage so V8 v7 actually runs. DSL-0 is one milestone with three sequential sub-phases.

**DSL-0a — Semantic extraction (~3-4 weeks):** for each of the 152 opcodes, extract its semantic body out of the alpha handler into a free function:

```rust
fn op_xxx_semantic(
    state: &mut LlIntDispatchState<'_>,
    args: OpXxxArgs,
) -> SemanticOutcome;  // (pc_advance, refresh_required) | Exit { kind, payload }
```

During DSL-0a, the legacy alpha handler in `dispatch_handlers/` is thinned to operand-decode + call-semantic + translate-outcome-to-Step. Both the alpha path and (eventually) the DSL cold stubs route through the same semantic body — single implementation per opcode. Several opcode families already have this shape (the `execute_*_opcode` methods on `Vm` in `vm/src/vm/dispatch/`); for those, extraction is mostly renaming. For families that don't (loads, scope ops, generators, exceptions), it's real work.

`LlIntDispatchState<'_>` is a safe wrapper module living in `vm/src/dsl/slow_path.rs`. It exposes a borrow-checked API mirroring today's `DispatchState<'_>` so semantic bodies don't have to be rewritten — they get familiar accessors backed by `LlIntState` + `LlIntRustContext`.

**DSL-0a exit criterion:** every opcode has its semantic body extracted into a free function. Alpha handlers are wrappers only. The invariant is enforced by a **structural manifest**, not by source-grep alone:

- `crates/vm/src/dsl/opcode_manifest.rs` declares `const OPCODES: &[OpcodeEntry]` with one entry per `Opcode` variant (`opcode`, `semantic_symbol: &'static str`, `dsl_handler_symbol: &'static str`, `category: OpcodeCategory`).
- Test 1: `OPCODES` length equals `Opcode` variant count (exhaustive coverage).
- Test 2: each `semantic_symbol` resolves to a real Rust function in the binary (linker reference).
- Test 3: each `dsl_handler_symbol` resolves to a real handler (linker reference, applies after DSL-0b).
- Test 4 (post-DSL-0a): a source-grep smoke test confirms no opcode body lives outside its `op_xxx_semantic` function in `dispatch_handlers/`; helper modules don't reach into handler-shaped logic. Grep is defense-in-depth, not the structural source of truth.

**DSL-0b — DSL infrastructure + hot ports + cold stubs (~4-5 weeks):**

- `lyng-vm-dsl` proc-macro crate created.
- `vm/src/dsl/backend/aarch64/` populated with the operations the spike needs (~20-25 ops, including direct-memory ops for warm handlers, `dispatch_prefixed!` for prefix opcodes, and `inc_counter!` for opcode-counter mode).
- `LlIntState` `#[repr(C)]` + `LlIntRustContext` + `LlIntRustContextOpaque` defined per §5. Const field-offsets exposed via `offset_of!`. Offset-generation tests in place.
- Register-pin convention enforced; `Vm::run` entry shim builds `LlIntRustContext`, points `state.rust_context` at it (cast to `*mut LlIntRustContextOpaque`), loads pinned registers, tail-jumps into dispatch. `_interpreter_exit` consumes `rust_context.exit` and builds `VmResult<Value>`.
- Slow-path bridge ABI implemented for all three tags with both exit kinds. Pre-slow-path sync protocol (§6) is the contract every shim follows.
- **Validation cases (each is a committed test):**
  1. Empty naked handler compiles via `naked_asm!` and dispatches correctly.
  2. Slow-path round-trip — one handler exercises each of the four slow-path outcomes (Continue / Refresh / Exit-Done / Exit-Error).
  3. PC-sync correctness — a cold-stub opcode whose semantic body asserts `state.frame.instruction_offset()` equals post-dispatch PC.
  4. Safepoint coverage on `op_loop_header` — tight `op_add` + `op_loop_header` loop reaches the GC poll under contrived poll-flag-set conditions.
  5. Safepoint coverage on backward `op_jump` — tight loop using `op_add` + negative `op_jump` (no `op_loop_header`) also reaches the GC poll.
  6. Safepoint coverage on backward conditional jump — tight loop using a conditional negative jump reaches the GC poll.
  7. Wide prefix decode — `op_wide` + `op_move` decodes wide register operands correctly.
  8. ExtraWide prefix decode — `op_extra_wide` + `op_move` decodes wide-32 register operands correctly.
  9. Double-prefix rejection — `op_wide` + `op_wide` raises the expected `VmError`.
- **Hot handlers ported with full DSL bodies (5):** `op_move`, `op_add`, `op_jump`, `op_return`, plus the warm `op_loop_header`.
- **Warm handlers ported (additional):** `op_jump`, `op_jump8`, `op_jump_if_true`, `op_jump_if_false` get backward-jump poll branches alongside their forward-jump fast paths. `op_wide` and `op_extra_wide` are warm with double-prefix rejection bodies.
- **Cold handlers ported as DSL stubs (~140):** every remaining opcode gets a `call_slow!(op_xxx_slow_rs, args = [...]); dispatch_after_slow!()` stub. The `op_xxx_slow_rs` shim follows the pre-slow-path sync protocol, calls the DSL-0a-extracted `op_xxx_semantic`, and translates the outcome. ~15-30 minutes per stub × ~140 ≈ 1.5 weeks.
- The `FeedbackVector` flat-array refactor (§9) lands inside DSL-0b as a prerequisite for the `FV` pin to be useful.
- Pre-DSL-0a and post-DSL-0b microbench + V8 v7 runs captured as `phase-DSL-0-bench.md`. Slow-path-share counts captured for the 5 hot handlers as a leading indicator (target < 20% — though DSL-0 only validates that the counter works; the 30-handler threshold gates DSL-1, not DSL-0).

**DSL-0c — Delete alpha and verify single-implementation invariant (~1 week):**

- Switch the active dispatch path from the alpha trampoline to the DSL table.
- Delete `run_trampoline_uncounted`, `Step` enum, `dispatch_handlers/`, `dispatch_state.rs`, and related α-only machinery.
- **Delete tier-accounting machinery on backedges.** The existing `observe_tier_backedge_event` calls on `op_loop_header`/`op_jump`/`op_jump8` were preserved through DSL-0a/b on the alpha path; they go away with alpha. After DSL-0c, the interpreter has no tier-up accounting — this is intentional, per §2 (JIT is out of scope) and the §6 same-thread `poll_pending` design. When the JIT track resumes, tier accounting comes back as part of that effort with its own design.
- Re-run all behavioral tests; re-run microbench + V8 v7.
- **Verify the single-implementation invariant via the manifest (§10 DSL-0a):**
  - Test 5: every `OpcodeEntry.dsl_handler_symbol` resolves to a function in the binary (linker check).
  - Test 6: `dispatch_handlers` module path does not exist in `crates/vm/src/`; source-grep confirms no `dispatch_next!`, `Step`, or `DISPATCH_TABLE` references in opcode semantic modules.
  - Test 7: opcode-counter mode (when compiled with `--features opcode-counters`) produces correct per-opcode dispatch counts under the DSL substrate, matching alpha-path counts within a documented per-handler instrumentation delta.

**DSL-0 exit criteria (no waivers):**

1. **Single-implementation invariant via manifest.** `OpcodeEntry` manifest covers every `Opcode` variant (Test 1 exhaustive); every `semantic_symbol` and `dsl_handler_symbol` resolves (Tests 2, 3, 5); source-grep smoke test (Tests 4, 6) shows no handler-shaped logic outside the manifest-listed locations; opcode-counter mode preserves per-opcode counts under DSL dispatch (Test 7).
2. **Asm shape vs LLInt (softened).** Each ported hot handler's main hit path is within 5 instructions of LLInt's matching handler, plus any documented irreducible delta from the current `Value`/`ObjectRef` layout captured in `llint-dsl-value-layout.md`. The ported report quantifies the delta per opcode.
3. **Microbench vs LLInt-equivalent.** Each ported hot opcode's ns/dispatch is within 2× of JSC LLInt's matching opcode, measured on the same dev MacBook with `--require-isolation`, 7-sample medians, identical iteration counts.
4. **Behavioral parity.** `cargo test -p lyng-vm -p lyng-tests` passes. Test262 pass count ≥ the pre-DSL-0 pristine baseline captured at the start of R-0 (currently `49722/49729` per [test262.md](../../reports/lyng/test262.md), refresh during R-0).
5. **V8 v7 directional check (executable now).** Geomean across V8 v7 workloads moves by **≥ +20%** vs pre-DSL-0 baseline. Richards specifically ≥ +30%. The gate works because all 152 opcodes are on the new substrate; the 5 fully-ported hot handlers carry the win.
6. **All 9 DSL-0b validation cases pass** (slow-path round-trip, PC-sync, three safepoint coverage cases, three prefix cases — see DSL-0b scope).
7. **No mysterious regressions.** Per-opcode dispatch-counter output before and after DSL-0 differs only in opcodes that legitimately changed.

If any of 1-6 fail and the cause isn't fixable in ~5 days of investigation, **DSL-0 is a clean abort.** DSL-0a (extracted semantic bodies + manifest) is preserved regardless — it's correctness-positive for the alpha path too. Only DSL-0b and DSL-0c revert. The eight-ten weeks produce concrete evidence; we pivot to γ-hard with the semantic extraction done.

If all pass, DSL-0 closes with a written decision-document committing to DSL-1.

### DSL-1: Hot-opcode rollout (~8-10 weeks)

Port the remaining ~25 hot opcodes (top-30 minus the 5 from DSL-0) from cold stubs to full DSL bodies with fast paths inline.

**Per-opcode workflow (~1-3 days each):**

1. Read JSC's matching handler in `LowLevelInterpreter64.asm` (or the captured reference); understand semantics.
2. Identify any data-layout refactor surfaced. If yes, that's its own ticket, done first.
3. Replace the cold-stub body with a full DSL fast path. Add new DSL operations to `backend/aarch64/` if needed (with `ops.md` entry).
4. Run `cargo asm` on the handler; inspect; iterate until shape is right.
5. Run microbench; capture into ported report.
6. Run isolated V8 v7 sweep; capture into ported report.
7. Write `reports/lyng/dsl-handlers/op_xxx.md` with side-by-side LLInt diff + data.
8. Commit asm baseline + ported report + handler source as one cohesive change.

**Port order (priority by dispatch share + risk):**

- Week 1-2: `op_load_*` family, `op_star_0..7` — trivial mechanical ports.
- Week 3-4: `op_load_local_*`, `op_store_local_*`, `op_ldar`.
- Week 5-6: `op_sub`, `op_mul`, `op_bit_*`, remaining SMI arithmetic.
- Week 7: `op_jump_if_*`, comparison ops.
- Week 8-9: `op_get_named_property`, `op_set_named_property`, `op_load_global`, `op_store_global`, `op_get_keyed_property`, `op_set_keyed_property` — IC opcodes. **The IC mode-byte refactor lands during this stretch** (replaces today's Phase 3a/3e/3f layered fast paths).
- Week 10: `op_call_0..3`, `op_call`, `op_tail_call` — frame-transitioning opcodes (`op_return` already done in DSL-0).

**Exit criteria:**

1. All 30 hot opcodes have full DSL implementations with committed ported reports.
2. Cumulative geomean on V8 v7 ≥ **+80% over the pre-DSL-0 baseline** (Richards ≥ ~570 against today's 318).
3. No workload regressed > 5% vs pre-DSL-0 baseline.
4. Test262 baseline preserved.
5. Every per-handler asm-diff report exists.
6. **Slow-path-share invariant.** For each of the 30 hot opcodes, the dispatch counter (when run with `--count-slow-path-share` on V8 v7 workloads) reports < 20% slow-path share. This is the leading indicator that the DSL fast paths are actually doing the work; if a hot opcode is 80%+ slow-path, we've quietly rebuilt the alpha interpreter inside the slow path and the substrate isn't winning anywhere. The 20% threshold has per-opcode waivers (some opcodes are fundamentally polymorphic), but documented per-handler in the ported report.

**Off-ramp during DSL-1:** if 5+ consecutive handlers fail either the within-5-of-LLInt criterion (modulo documented Value-layout delta) or the < 20% slow-path-share criterion, pause. The DSL might have a structural problem invisible in DSL-0. Cheaper to discover at handler 10 than 25.

### DSL-2: x86_64 backend (deferred — was DSL-3)

Not in initial scope. Activates when:

- lyng has a concrete x86_64 user / deployment, **or**
- A contributor wants to work on x86_64 specifically.

When activated: ~6 weeks of porting `backend/aarch64/` to `backend/x86_64/`. DSL surface is identical; only the per-arch macros change. All handlers (hot and cold) recompile against the new backend.

### DSL-3: Data-layout refactors (interleaved during DSL-1; was DSL-4)

Each refactor is its own ticket, fired when DSL-1 surfaces evidence for it. See §9 for the expected surfaces. Notably, `FeedbackVector` flat-array layout lands in DSL-0 as a prerequisite, not here.

### Test262 100% (parallel workstream)

Orthogonal to the DSL work. Spec compliance is about *semantics* (does `Array.prototype.at` handle every spec edge case?), not dispatch shape. DSL changes don't break Test262 if slow paths preserve semantics. Pace this stream to your appetite; it gets its own brainstorm if needed.

### Total interpreter-substrate timeline

| Phase                                                                | Duration       | Cumulative     |
| -------------------------------------------------------------------- | -------------- | -------------- |
| R-0 (tooling + 3 evidence reports)                                    | 3-4 weeks      | 4 wk           |
| DSL-0a (semantic extraction; alpha still active)                     | 3-4 weeks      | 8 wk           |
| DSL-0b (DSL infra + hot ports + cold stubs + FV refactor)            | 4-5 weeks      | 13 wk          |
| DSL-0c (delete alpha; verify single-implementation invariant)        | 1 week         | 14 wk          |
| **Decision point — abort to γ-hard, or scale up to DSL-1**           |                |                |
| DSL-1 (25 more hot opcodes + IC mode-byte refactor)                  | 8-10 weeks     | 24 wk          |
| DSL-3 (data-layout, interleaved during DSL-1)                        | (in DSL-1)     | (overlapping)  |
| **Total: ~5.5-6 months single-dev**                                  |                |                |
| DSL-2 (x86_64) deferred                                               | +6 weeks when activated | n/a   |
| Test262 100% (parallel)                                               | ~3-6 months    | n/a            |

## 11. Risks

| Risk                                                                   | Likelihood | Impact | Mitigation                                                                                            |
| ---------------------------------------------------------------------- | ---------- | ------ | ----------------------------------------------------------------------------------------------------- |
| **DSL-0a semantic extraction expands beyond estimate**                 | **medium** | **high** | The reviewer flagged this as the largest hidden work item. Estimate is 3-4 weeks for 152 opcodes; if it blows out, every later phase slips. Mitigation: stage opcode families and measure per-family completion rate after week 2. If a family takes >2x its estimate, re-evaluate scope (could DSL-0 ship with some families still on alpha handlers as a transitional state?). |
| **DSL-1 slow-path-share regression — "alpha rebuilt inside slow path"** | **medium** | **high** | The reviewer's primary structural concern. Mitigation: explicit < 20% slow-path-share invariant per hot opcode in DSL-1 exit criteria. Dispatch-counter instrumentation in R-0 makes this measurable. If a hot opcode is 80%+ slow-path, its fast-path design is wrong and gets re-thought before merging. |
| `#[unsafe(naked)]` + `naked_asm!` ergonomics tighter than expected (LLVM codegen quirks, restricted operand syntax) | medium | high | DSL-0b spike validates this on real opcodes before scaling. Empty-handler validation case lands in week 1 of DSL-0b. If unsolvable, pivot to γ-hard with DSL-0a's extraction work preserved. |
| `LlIntState` `#[repr(C)]` layout instability across rustc versions     | low        | high   | `offset_of!` generated const offsets + offset-generation test in `llint-dsl-abi.md`. All asm-visible fields are thin pointers + integers; no Rust enums or trait objects in the asm record. |
| `LlIntRustContext` lifetime-safety bugs at slow-path shim boundary     | medium     | high   | `LlIntRustContextOpaque` erased pointer type prevents accidental misuse — the cast back to the concrete `LlIntRustContext<'vm>` happens in exactly one place (`LlIntDispatchState::from_raw`), with the safety contract documented inline. Wrapper `LlIntDispatchState<'_>` enforces borrow rules for callers. Miri tests on the shim layer. |
| Slow-path panic unwinding across asm boundary (UB / abort)             | low        | high   | `extern "C"` ABI aborts on panic. Debug builds wrap shims in `catch_unwind` for clean abort + diagnostic. No path of control flow returns from a slow-path call via stack unwinding; errors travel through `LlIntExitSlot`. |
| Slow-path bridge performance bottlenecks (megamorphic IC, exception-heavy code) | medium | medium | Microbench megamorphic workloads in DSL-0. If slow-path frequency dominates, inline more IC states in DSL. The < 20% slow-path-share invariant in DSL-1 forces this to be addressed before merging. |
| GC starvation on tight loops without `op_loop_header`                  | low        | high   | Backward-jump handlers (`op_jump`, `op_jump8`, conditional jumps with negative offsets) are warm. Three explicit invariant tests in `llint-dsl-safepoints.md` cover the cases. |
| `Value` layout mismatch with DSL vocabulary assumptions                | medium     | medium | R-0 `llint-dsl-value-layout.md` documents exact masks and expected asm per check/tag op *before* any DSL handler ships. DSL-0b vocabulary uses `check_smi!` / `check_object_ref!` aligned with current Value. |
| `FeedbackVector` flat-array refactor regresses Phase 3f's packed sidecars | low-medium | medium | Refactor preserves per-entry packed monomorphic/proto/polymorphic state; only vector storage changes. Existing IC tests + benchmarks gate the refactor. |
| `poll_pending` flag misses an event                                     | low        | low    | Same-thread access only in DSL-0. Producers run during slow-path execution or between `Vm::run` invocations; consumers read on the next backedge. No race conditions to design around. Cross-thread pause requests are explicitly out of scope. |
| Cross-thread debugger requirement appears mid-implementation            | low        | medium | If a real cross-thread debugger requirement surfaces, that's a separate design ticket per §6. Don't retrofit the existing same-thread design with `AtomicU8` and a partial-thread-safety story — design the full synchronization surface together (hook handoff, pause payload, asm-side atomic semantics). |
| Pre-slow-path PC sync forgotten in a shim                              | medium     | high   | Every shim follows the fixed sync sequence from §6. `LlIntDispatchState::sync_from_asm()` is the *only* sanctioned way to materialize the frame for a semantic body. PC-sync invariant test (DSL-0b validation case #3) catches drift; a per-shim audit during DSL-0b ensures every cold-stub shim calls `sync_from_asm()`. |
| Prefix dispatch incorrectly handled (operand width drift, double-prefix bug) | medium | high   | Three explicit prefix validation cases in DSL-0b (Wide decode, ExtraWide decode, double-prefix rejection). Prefix decoding is in the single-implementation invariant. Layout decoders generated by `llint_handler!` enforce `state.prefix` consume-and-clear. |
| Tier accounting regression after alpha deletion                        | low        | low    | Tier accounting is explicitly deferred from DSL-0 (§6, §10). Nothing in DSL-0 consumes tier-up signal; nothing produces it. When JIT track resumes, tier accounting gets its own design — DSL-0's clean substrate is what enables that. |
| x86_64 register pressure when DSL-2 (x86_64 backend) activates         | medium     | medium | Verify x86_64 pin assignment before porting handlers. Plan B: don't pin `FV`, load from STATE per IC. |
| DSL becoming a maintenance black hole (feature creep)                  | medium     | high   | DSL surface deliberately small (~40-45 ops). Reject feature creep. Adds require: third occurrence of pattern + documentation. |
| Cold-stub authoring overhead during DSL-0b                             | medium     | low    | ~15-30 min per stub × 145 ≈ 1.5 weeks. Mechanical work. If the per-stub time blows out, the slow-path-bridge protocol is fundamentally wrong and we should know in DSL-0b week 2. |
| Per-arch divergence in handler behavior                                | low        | high   | Behavioral tests cover both arches. Per-arch asm baselines acknowledge shape differences but behavior is one source of truth. |
| Rust toolchain bugs in `#[unsafe(naked)]` / `core::arch::naked_asm!`   | low        | medium | Asm baselines catch codegen changes immediately. Can pin known-good rustc if needed. `naked_asm!` is stable as of Rust 1.88; mature enough for production use. |
| AGENTS.md / engineering-standards.md current "no unsafe in lyng" policy contradicts the DSL | high (if not pre-fixed) | medium | Policy doc updates **must land in the same change as the DSL crate** to avoid contradictory instructions to future contributors. See §12. |
| Recruiting / onboarding cost for DSL fluency                           | medium     | low    | DSL is small. Cold opcodes stay simple stubs. Documentation + JSC reference files lower the bar.      |
| Data-layout refactors stall on GC integration (specifically pointer-identity cells) | medium | medium | Each refactor is its own ticket with own evidence requirement. Refactor only when DSL-0/1 surfaces hot need. |
| Microbench / V8 v7 measurement noise on dev machine                    | high       | low    | `--require-isolation` enforces loadavg < 2.0. Document quiescing steps. Accept higher variance on multi-purpose machines. |

## 12. Policy and issue-tracker alignment (must land with the design)

The current `crates/AGENTS.md` says **"Do not use `unsafe` code in Lyng JS crates"** verbatim. This contradicts the approved design. Documentation updates must land alongside the DSL crate's creation so future contributors see consistent guidance.

**Required updates:**

- **`crates/AGENTS.md`** — replace the blanket no-unsafe rule with a scoped exception: `unsafe` is allowed only in (a) the `lyng-vm-dsl` crate, (b) `crates/vm/src/dsl/` (the DSL backend, entry/exit shims, slow-path bridge module), and (c) bridge modules explicitly named in this design. Hand-written `unsafe` outside these locations remains forbidden.
- **`docs/lyng/engineering-standards.md`** — add a section on the DSL substrate boundary, the unsafe surface within it, and the audit/review expectations for changes to those modules.
- **`docs/lyng/architecture.md`** — reflect the new dispatch substrate (DSL handlers, tail-jump dispatch, `LlIntState` ABI) replacing the α trampoline description.
- **Lint or source-level test** — a `cargo test` that walks `crates/**/*.rs` and fails if `unsafe` appears outside the approved module paths. Mechanical enforcement of the policy.

**Issue-tracker alignment:**

- Create a new dcat epic for "asm-DSL LLInt-class interpreter (lyng-49qk successor)" as a child of `lyng-49qk` so the relationship is explicit in the tracker.
- Break R-0 into separate dcat issues for: `microbench` subcommand, `asm-diff` subcommand, `capture-llint` subcommand, hot-opcodes config, `llint-dsl-value-layout.md`, `llint-dsl-abi.md`, `llint-dsl-safepoints.md`.
- Mark `lyng-28t2` (γ-swap evaluation) as superseded/deferred — but only after explicit user approval (per the project's "never close issues without explicit user approval" rule in `AGENTS.md`).
- Mark Phase 5 / Phase 6 epics (Baseline JIT prerequisites + Baseline JIT) as deferred-behind-this-design, not superseded — they remain valid in shape per §2 non-goals.

## 13. Open questions to revisit during DSL-0

These don't block the design but should be answered with data, not speculation:

1. **DSL-0a extraction time per opcode family.** The estimate is 3-4 weeks for 152 opcodes total, but per-family velocity varies. Loads/arithmetic are mechanical; calls/generators/exceptions are more involved. Track per-family completion rate from week 1; if a family takes >2× its estimate, decide whether DSL-0 can ship with that family still on the alpha path transitionally, or whether the whole DSL-0 timeline grows.
2. **`op_return` Refresh vs fast-return.** Worth optimizing in DSL-0b or defer? Decide based on `op_return` microbench post-DSL-0b.
3. **Sixth pinned register.** Do we need a `META` / metadata-table pin separate from `FV`? Decide if a hot handler in DSL-1 demands it.
4. **DISPATCH_TABLE on x86_64.** RIP-relative is free; confirm with x86_64 backend implementation in DSL-2.
5. **Pointer-identity cells in DSL-1 or DSL-3.** Refactor inside DSL-1 (front-load) or interleave with DSL-1 IC port (evidence-driven)?
6. **Cold-stub overhead.** ~8 instructions per cold dispatch. Acceptable for opcodes at <0.1% dispatch share; quantify in DSL-0c when all cold stubs land.
7. **DSL ABI stability across rustc upgrades.** Does `naked_asm!` codegen drift between rustc 1.88 and later versions? If so, what's the cost of locking a rustc version vs absorbing drift?
8. **Slow-path-share threshold per opcode.** 20% is the default DSL-1 invariant, but some opcodes (e.g., `op_get_keyed_property` on polymorphic-keyed workloads) may legitimately have higher slow-path share. Document the per-opcode threshold derivation methodology in DSL-1 (likely: "threshold = the share we'd expect from LLInt on the same workload"). Waivers must justify against this baseline.
9. **`LlIntRustContext` debug-build vs release-build layout.** `FrameRecord` is sizable; keep it inline in `LlIntRustContext` or box it? Decide based on DSL-0b allocator behavior.
10. **Compiler invariant for backedges.** Eventually proving every backedge passes through `op_loop_header` would let us drop the backward-jump poll branches. Audit the compiler's emission after DSL-1; if the invariant already holds for all generated code, simplify the warm-handler set.
11. **Tier accounting re-introduction shape.** Out of scope for DSL-0, but when JIT work resumes the design has to decide whether tier-up signal lives in a new `poll_pending` bit (with same-thread or cross-thread semantics), a separate per-CodeBlock counter accessed via `rust_context.installed`, or a different mechanism entirely. Don't pre-commit; let JIT design inform.
12. **Opcode-counter mode asm overhead.** When `--features opcode-counters` is on, the proc-macro emits an extra ~3-instr counter-increment per handler. Quantify the per-bench overhead. If it's >5% on tight loops, consider a sparse counter that increments on a sampled subset of dispatches.

## 14. References

- **Companion retrospective:** [`reports/lyng/llint-parity-state-of-engine.md`](../../reports/lyng/llint-parity-state-of-engine.md) — the measurement-driven analysis of why the original roadmap missed.
- **Design review feedback (Codex, first pass, 2026-05-16):** the review that triggered the first revision. P0/P1 findings (`naked_asm!` vs `asm!`, slow-path ABI insufficient for `VmError`, `DispatchState` not C-ABI, safepoint coverage gap, Value-layout vocabulary mismatch, DSL-0 V8-v7 gate not executable, FV lifecycle underspecified) are addressed in §3, §4, §5, §6, §7, §9, and §10.
- **Design review feedback (Codex, second pass, 2026-05-16):** the review that triggered the second revision. P0/P1 findings (trait-object fat pointers can't erase to `c_void`, `LlIntState` missing full frame/install context, cold stubs need semantic-extraction subphase, `op_loop_header` is a marker not a jump, backward-jump poll coverage, `FV` mutability, `LlIntExitSlot` should be Rust-side, `poll_pending` producer model, `sym` operand specification, no-unwind policy) are addressed by introducing `LlIntRustContext` (§5), the DSL-0a semantic-extraction subphase (§10), the warm backward-jump handler category (§3, §6), and the slow-path-share invariant in DSL-1 (§10).
- **Design review feedback (Codex, third pass, 2026-05-16):** the review that triggered the third revision. Verdict: no P0 blockers; P1/P2 findings to fold in before R-0. Findings (`'static` lifetime stand-in is misleading, `AtomicU8` over-specifies cross-thread debugger, tier-up counter has circular producer problem, pre-slow-path PC sync underspecified, prefix dispatch needs explicit DSL design, slow-path-share instrumentation must be R-0 deliverable, source-grep too weak for invariant, opcode-counter mode under naked dispatch unspecified) are addressed by the `LlIntRustContextOpaque` erasure (§5), the same-thread `Vm.poll_pending: u8` design (§6), the deliberate deferral of tier accounting from DSL-0 (§6, §10), the pre-slow-path sync protocol (§6), the `dispatch_prefixed!` DSL operation and three prefix validation cases (§6, §10), slow-path-share counters as an R-0 deliverable (§10), the opcode manifest with seven structural invariant tests (§10), and feature-flagged `inc_counter!` for opcode-counter mode (§10).
- **Original roadmap (superseded in scope):** [`reports/lyng/jsc-aligned-engine-roadmap.md`](../../reports/lyng/jsc-aligned-engine-roadmap.md) — Phase 1-6 plan. The interpreter-track substrate decision (Option α) is reversed by this design; the Baseline JIT track (Phases 5-6) remains valid in shape, deferred behind this work.
- **JSC LLInt source (reference, not vendored):** `/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/`
  - `LowLevelInterpreter.asm` — dispatch macros, wrapper macros (`llintOp*`).
  - `LowLevelInterpreter64.asm` — 64-bit handler bodies including `performGetByIDHelper`.
  - `offlineasm/` — Ruby DSL compiler (architectural reference for our DSL's design).
- **JSC IC metadata reference:** `Source/JavaScriptCore/bytecode/GetByIdMetadata.h` — the `GetByIdMode` enum design our `op_get_named_property` mode-byte refactor mirrors.
- **JSC value layout reference:** `Source/JavaScriptCore/runtime/JSCJSValue.h` — for the R-0 `llint-dsl-value-layout.md` comparison.
- **Our current Value implementation:** [`crates/types/src/value.rs`](../../crates/types/src/value.rs) — the source of truth for the value-layout report.
- **Our current dispatch substrate (to be replaced):**
  - [`crates/vm/src/vm/dispatch_state.rs`](../../crates/vm/src/vm/dispatch_state.rs)
  - [`crates/vm/src/vm/dispatch_handlers/`](../../crates/vm/src/vm/dispatch_handlers/)
  - [`crates/vm/src/vm/dispatch/`](../../crates/vm/src/vm/dispatch/)
  - [`crates/vm/src/vm/feedback.rs`](../../crates/vm/src/vm/feedback.rs)
- **Existing bench tool to be extended:** [`tools/lyng-bench/src/cli.rs`](../../tools/lyng-bench/src/cli.rs) — currently has `runtime`, `density`, `test262`, `compare`, `v8suite`. R-0 adds `microbench`, `asm-diff`, `capture-llint`.
- **Project standards (subject to update per §12):**
  - [`AGENTS.md`](../../AGENTS.md) — repo-level guide.
  - [`crates/AGENTS.md`](../../crates/AGENTS.md) — Lyng JS operating guide. **Update required**: scope-allow `unsafe` in DSL modules.
  - [`docs/lyng/engineering-standards.md`](engineering-standards.md) — code quality bar. **Update required**: DSL boundary review expectations.
  - [`docs/lyng/architecture.md`](architecture.md) — system architecture. **Update required**: reflect new dispatch substrate.
  - [`docs/lyng/performance-workflow.md`](performance-workflow.md) — existing perf measurement conventions; the DSL workflow extends these.
