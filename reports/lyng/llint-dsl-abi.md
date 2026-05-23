# Lyng-js LLInt DSL ABI — handler bridge prerequisite report

This report is the source of truth for the asm-DSL handler ABI: the
`#[repr(C)] LlIntState` record, the Rust-only `LlIntRustContext`, the
opaque marker used to bridge them, the pinned-register convention, the
slow-path return ABI, and the pre-slow-path sync protocol every shim
runs. It is the second of three R-0 evidence reports required before
DSL-0 begins (the other two cover the `Value` representation and the
safepoint / allocation model). The DSL backend compiles handler bodies
into `naked_asm!` blocks that read and write asm-visible state at fixed
field offsets; those offsets, plus the register-pin convention and the
return tags, are the contract handler authors will rely on. Anything
that changes here breaks every generated handler simultaneously, so the
shape has to land before DSL-0a starts.

## Source citations

- Authoritative design: [docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md](../../../docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md),
  §5 ("Register-pin convention, `LlIntState` ABI, and
  `LlIntRustContext`") and §6 ("Slow-path bridge protocol").
- Companion R-0 reports: [llint-dsl-value-layout.md](./llint-dsl-value-layout.md)
  (the `Value` encoding the DSL's `check_*!` / `tag_*!` / `untag_*!`
  macros compile against) and the forthcoming `llint-dsl-safepoints.md`
  (the poll-byte and `call_slow!` boundary GC model that this report's
  `Refresh` discipline assumes).
- Reference for comparison only: WebKit `Source/JavaScriptCore/llint/`
  (read-only; not vendored, not quoted verbatim).
- Phase-3 baseline this ABI replaces: the inline IC fast path in
  [crates/lyng/vm/src/dispatch.rs](../../../crates/lyng/vm/src/dispatch.rs)
  and the per-opcode handlers under
  [crates/lyng/vm/src/handlers/](../../../crates/lyng/vm/src/handlers/),
  which currently take `&mut Vm` directly and rely on Rust function-call
  conventions rather than register pinning.

## The two-record split (and why)

Asm-visible state must be `#[repr(C)]`, fixed-layout, and contain only
thin pointers, integers, and padding — anything else (Rust enums, trait
objects, lifetimes, `Arc`s) breaks the offset-stability contract that
`naked_asm!` blocks rely on. The DSL substrate also needs to reach a
full safe Rust frame (`&mut Vm`, `&mut Agent`, the installed function,
the realm, the canonical `FrameRecord`) from inside slow-path shims —
that state is the opposite of asm-stable.

The design resolves the tension by splitting interpreter state into two
records:

1. **`LlIntState`** — `#[repr(C)]`, asm-visible, fixed offsets.
   Synced from pinned registers immediately before every `call_slow!`,
   reloaded into pinned registers after `Refresh`. Naked asm reads and
   writes it directly at `const`-resolved offsets.

2. **`LlIntRustContext<'vm>`** — pure Rust, not `repr(C)`, carries
   lifetimes and `Arc`s. Naked asm **never** reads this directly; the
   slow-path Rust shim recovers it from the opaque pointer stored in
   `LlIntState.rust_context`.

The bridge between them is a single thin pointer (`rust_context`) typed
as `*mut LlIntRustContextOpaque`, where `LlIntRustContextOpaque` is a
zero-sized marker. The asm side passes the pointer through verbatim;
the cast back to the concrete `*mut LlIntRustContext<'vm>` is performed
only inside `LlIntDispatchState::from_raw` at the start of each shim.

## `LlIntState` — asm-visible, fixed layout

The proposed Rust definition (mirroring §5 of the design):

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
    pub _pad1:              u32,          // align frame_pb_base to 8 bytes
    pub frame_pb_base:      *const u8,    // installed function's bytecode base
    pub frame_regs_base:    *mut Value,   // register-stack base for active frame
    pub frame_fv_base:      *mut FeedbackEntry, // feedback array base; record_*!/value_profile! write through it
    pub frame_depth:        u32,          // vm.frames().len() at snapshot
    pub frame_check_epoch:  u32,          // mirror of vm.dispatch_frame_check_epoch()

    // --- Single erased thin pointer to Rust-only state (asm only ever passes it through) ---
    pub rust_context:       *mut LlIntRustContextOpaque,

    // --- Per-instruction prefix flag (Wide / ExtraWide) ---
    pub prefix:             u8,           // 0 = none, 1 = Wide, 2 = ExtraWide
    pub _pad2:              [u8; 7],
}
```

### Computed field offsets

The following offsets were produced by `core::mem::offset_of!` in a
standalone Rust binary (rustc stable, Apple Silicon target;
`size_of::<*const u8>() == size_of::<*mut _>() == 8`):

| Field               | Offset (bytes) | Size (bytes) | Notes                                                         |
| ------------------- | -------------: | -----------: | ------------------------------------------------------------- |
| `frame_pc_offset`   |              0 |            4 | `u32`; PC = `frame_pb_base + frame_pc_offset` at entry/Refresh |
| `_pad1`             |              4 |            4 | aligns `frame_pb_base` to 8                                   |
| `frame_pb_base`     |              8 |            8 | `*const u8`                                                   |
| `frame_regs_base`   |             16 |            8 | `*mut Value`                                                  |
| `frame_fv_base`     |             24 |            8 | `*mut FeedbackEntry` (mutable for `record_*!`)                |
| `frame_depth`       |             32 |            4 | `u32`                                                         |
| `frame_check_epoch` |             36 |            4 | `u32`                                                         |
| `rust_context`      |             40 |            8 | `*mut LlIntRustContextOpaque`                                 |
| `prefix`            |             48 |            1 | `u8` — 0 / 1=Wide / 2=ExtraWide                               |
| `_pad2`             |             49 |            7 | pads `LlIntState` to 56 bytes (8-byte alignment)              |

Total `size_of::<LlIntState>()` = **56 bytes**, `align_of` = **8 bytes**.

Every field is a thin pointer, integer, or padding. No Rust enum, no
trait object, no lifetime-bearing reference, no `Arc<T>` — these would
all break offset stability under future rustc versions. The
`#[repr(C)]` attribute pins layout to a deterministic C-like rule
(no field reordering, predictable alignment); the offset-generation
test (below) enforces these values as hard constants.

### How offsets reach `naked_asm!`

Offsets are exposed as `pub const` items in
`crates/lyng/vm/src/dsl/reg_convention.rs`, e.g.:

```rust
pub const LLINTSTATE_FRAME_PC_OFFSET_OFFSET:    usize = offset_of!(LlIntState, frame_pc_offset);
pub const LLINTSTATE_FRAME_PB_BASE_OFFSET:      usize = offset_of!(LlIntState, frame_pb_base);
pub const LLINTSTATE_FRAME_REGS_BASE_OFFSET:    usize = offset_of!(LlIntState, frame_regs_base);
pub const LLINTSTATE_FRAME_FV_BASE_OFFSET:      usize = offset_of!(LlIntState, frame_fv_base);
pub const LLINTSTATE_FRAME_DEPTH_OFFSET:        usize = offset_of!(LlIntState, frame_depth);
pub const LLINTSTATE_FRAME_CHECK_EPOCH_OFFSET:  usize = offset_of!(LlIntState, frame_check_epoch);
pub const LLINTSTATE_RUST_CONTEXT_OFFSET:       usize = offset_of!(LlIntState, rust_context);
pub const LLINTSTATE_PREFIX_OFFSET:             usize = offset_of!(LlIntState, prefix);
```

The DSL backend references these via `const` placeholders in
`naked_asm!`. There is **no textual offset literal** anywhere in the
generated asm — the asm shape always carries the symbolic name, and
the assembler resolves it at link time from the `const` value.

## `LlIntRustContext` — Rust-only, lifetime-bearing

```rust
// NOT #[repr(C)]. Naked asm NEVER reads this directly.
// Only slow-path Rust shims dereference it.
pub struct LlIntRustContext<'vm> {
    pub vm:        &'vm mut Vm,
    pub agent:     &'vm mut Agent,
    pub host:      &'vm dyn HostHooks,
    pub registry:  &'vm mut (dyn NativeFunctionRegistry + 'vm),
    pub installed: Arc<InstalledFunction>,
    pub frame:     FrameRecord,                 // FULL snapshot — every field: code, realm, env, etc.
    pub frame_depth: usize,
    pub exit:      LlIntExitSlot,               // exit-slot lives here, off the asm-visible record
}
```

Why this is **not** `#[repr(C)]`:

- `&'vm mut Vm` and `&'vm mut Agent` carry lifetimes; lifetime-bearing
  references have no defined ABI position.
- `&'vm dyn HostHooks` and `&'vm mut (dyn NativeFunctionRegistry + 'vm)`
  are trait-object fat pointers; their internal layout is unspecified.
- `Arc<InstalledFunction>` is a managed pointer with a custom drop.
- `FrameRecord` (the canonical owned frame snapshot) embeds Rust enums
  and managed handles whose layout is not C-stable.

All of these are deliberately fine here, because **no offset in this
struct is ever consumed by asm.** Every field is read only by Rust
code inside slow-path shims, which can use Rust's full type system.

### Lifetime contract

`LlIntRustContext<'vm>` is alive across the entire `Vm::run`
invocation. The entry shim (in `vm/src/dsl/entry.rs`) constructs it,
takes the `*mut LlIntRustContextOpaque` cast, stores that pointer in
`LlIntState.rust_context` once, and never moves the context for the
lifetime of `Vm::run`. The asm bridge passes the `*mut LlIntState` as
the first slow-path argument; the shim's `LlIntDispatchState::from_raw`
recovers `&'_ mut LlIntRustContext<'vm>` from the raw pointer under
the contract that the entry shim established the borrowed lifetime
before any handler ran.

The `'vm` parameter is erased at the asm-side boundary precisely
because asm cannot represent it. The cast is safe so long as
`LlIntDispatchState::from_raw` is the only function that performs it,
and so long as the entry shim's `&'vm mut Vm` outlives every shim call
(which it does by construction — `Vm::run` is the lifetime-defining
function).

## `LlIntRustContextOpaque` — the erased pointer marker

```rust
#[repr(C)]
pub struct LlIntRustContextOpaque {
    _private: [u8; 0],
}
```

A zero-sized, `repr(C)`, private-field type. Its sole job is to give
`LlIntState.rust_context` a type-stable pointer target that **cannot be
accidentally treated as carrying lifetime or trait information**. If
the field were typed `*mut LlIntRustContext<'static>` instead, two
problems would appear:

1. Lifetime laundering: writing `'static` to describe what is actually
   a `'vm` lifetime would invite future maintainers to dereference the
   pointer in asm-adjacent code that the borrow checker would mistakenly
   bless.
2. Type-system entanglement: any change to `LlIntRustContext`'s
   generic parameters or field types would visibly change the type of
   `LlIntState.rust_context`, even though the asm layer doesn't and
   cannot care.

Using a separate empty-marker type keeps `LlIntState` agnostic of the
concrete Rust context's identity, and concentrates the entire unsafe
contract — "this pointer is actually `*mut LlIntRustContext<'vm>` for
the duration of `Vm::run`" — in one function:

```rust
impl LlIntDispatchState<'_> {
    /// SAFETY: caller is the asm bridge, which holds the only valid
    /// `*mut LlIntState` for the lifetime of `Vm::run`. The `rust_context`
    /// field was populated by `Vm::run`'s entry shim and points to a
    /// pinned `LlIntRustContext<'vm>` whose lifetime strictly outlives
    /// every slow-path call.
    pub unsafe fn from_raw<'a>(state: *mut LlIntState) -> Self {
        let raw_ctx = (*state).rust_context as *mut LlIntRustContext<'a>;
        // ... assemble wrapper combining &mut *state and &mut *raw_ctx ...
    }
}
```

This is the **only** place in the codebase that performs the cast.
Audit boundary: any new use of `LlIntState.rust_context` outside
`LlIntDispatchState::from_raw` is a code-review block.

## Pinned-register convention

State that's accessed per-dispatch lives in callee-saved registers,
pinned across the entire interpreter from `Vm::run` entry through every
handler invocation. The pinned set is fixed at the per-arch level.

### AArch64 (initial target — DSL-0a/0b)

| Pin       | Register | Type                                                                              | Refreshed when                                                              |
| --------- | -------- | --------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| `PC`      | x19      | `*const u8` (pointer, not offset; `pb_base + frame_pc_offset` materialized at entry) | call / return / exception unwind                                            |
| `REGS`    | x20      | `*mut Value`                                                                      | call / return                                                               |
| `FV`      | x21      | `*mut FeedbackEntry`                                                              | call / return                                                               |
| `VM`      | x22      | `*mut Vm`                                                                         | once per `Vm::run`; used for `[VM, #VM_POLL_PENDING_OFFSET]` reads on warm handlers |
| `TABLE`   | x23      | `*const Handler`                                                                  | once per `Vm::run`                                                          |
| `STATE`   | x24      | `*mut LlIntState`                                                                 | once per `Vm::run`; also moved to x0 for `call_slow!`                       |
| `t0..t6`  | x9–x15   | scratch (caller-saved)                                                            | per-instruction                                                             |
| spare CSR | x25–x28  | available for handlers needing more pins                                          | as needed                                                                   |

Notes:

- **PC as pointer, not offset.** Dispatch is `ldrb [PC]; advance; load
  handler; jump` — 4 instructions total. The slow-path sync turns the
  pointer back into an offset (`PC - pb_base`) for `LlIntState.frame_pc_offset`.
- **PB is not pinned.** Reconstructable from `state.frame_pb_base` on
  the rare paths that need it (bounds checks at function boundaries,
  slow-path PC syncing).
- **`STATE` doubles as the slow-path arg-0 source.** Every `call_slow!`
  does `mov x0, x24` before the `bl {slow_rs}`.

### x86_64 (deferred, but specified for forward-compat — DSL-3)

| Pin       | Register      | Notes                                            |
| --------- | ------------- | ------------------------------------------------ |
| `PC`      | r12           | callee-saved                                     |
| `REGS`    | r13           | callee-saved                                     |
| `FV`      | r14           | callee-saved                                     |
| `VM`      | r15           | callee-saved                                     |
| `STATE`   | rbx           | callee-saved (`*mut LlIntState`)                 |
| `TABLE`   | RIP-relative  | no register pin (RIP-relative addressing is free) |
| scratch   | rax, rcx, rdx, rsi, r8, r9, r10, r11 | caller-saved                  |

The DSL surface is identical across arches: every operation
(`load_reg!`, `check_smi!`, `call_slow!`, `dispatch!`) exists on both
arches with the same name and semantics. Arch-specific instruction
counts diverge — that's covered by per-arch asm baselines under
[reports/lyng/dsl-asm-baseline-aarch64/](./dsl-asm-baseline-aarch64/),
not by branching DSL code.

## Slow-path return ABI

A slow path is a normal Rust `extern "C"` function. The asm bridge
calls it with `bl {slow_rs}` (a symbol operand bound via `sym
op_xxx_slow_rs` — see "Symbol mangling" below), passing the asm-visible
`*mut LlIntState` and up to five `u32` operand words. The shim returns
a fixed two-word struct:

```rust
#[repr(C)]
pub struct SlowPathReturn {
    pub tag: u64,      // SlowPathTag, returned in x0 / rax
    pub payload: u64,  // returned in x1 / rdx
}

#[repr(u64)]
pub enum SlowPathTag {
    Continue = 0,   // payload = new PC offset (u32-in-u64). Dispatch at new PC.
    Refresh  = 1,   // payload = unused. Reload PC/REGS/FV from state.frame_*, dispatch.
    Exit     = 2,   // payload = unused. Bridge jumps to _interpreter_exit;
                    //   exit shim reads rust_context.exit.
}
```

### Per-tag semantics

| Tag        | Asm bridge action                                                                                                             | Payload meaning                          | Triggers                                                                                          |
| ---------- | ----------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------- | ------------------------------------------------------------------------------------------------- |
| `Continue` | `PC ← pb_base + payload`; load opcode; tail-dispatch                                                                          | new PC offset (32-bit value zero-extended in 64-bit return slot) | Common case for value-bearing slow paths (`op_get_named_property_slow_rs`, etc.) that don't alter frame |
| `Refresh`  | Reload `PC`, `REGS`, `FV` from `state.frame_pc_offset / frame_regs_base / frame_fv_base`; load opcode; tail-dispatch          | unused (0)                               | `op_call_*`, `op_return`, caught exceptions (`transfer_to_exception_handler`)                     |
| `Exit`     | Branch to `{interpreter_exit}` symbol; exit shim reads `rust_context.exit` to discriminate `Done` vs `Error`                  | unused (0)                               | `op_return` from top frame; uncaught guest exceptions; VM-internal errors                         |

The three *dispatch* tags do not encode success-vs-error. That
information lives in the exit slot (next section). A handler that
fails returns `Exit` with `rust_context.exit.kind == Error`; a
handler that succeeds-and-returns-from-top-frame returns `Exit` with
`rust_context.exit.kind == Done`. The bridge doesn't care; the exit
shim reconstructs the appropriate `VmResult<Value>`.

`Refresh` is required for any slow path that mutates frame topology
(`op_call_*`, `op_return`) because pinned registers `PC` / `REGS` / `FV`
must be reloaded to the new frame's bases. Caught guest exceptions also
take `Refresh`: the slow path runs `Vm::transfer_to_exception_handler`,
updates `rust_context.frame` and `state.frame_pc_offset` to the catch
handler's PC, and returns `Refresh` so the bridge resumes there.

## Exit slot — out-of-band success/error channel

The exit slot is **not** in `LlIntState`. It lives in the Rust-only
context:

```rust
pub struct LlIntExitSlot {
    pub kind:       ExitKind,
    pub done_value: Value,                       // set when kind == Done
    pub error:      Option<Box<VmError>>,        // set when kind == Error; bridge takes ownership
}

pub enum ExitKind {
    None,                                         // bridge has not yet observed Exit
    Done,                                         // Vm::run returns Ok(done_value)
    Error,                                        // Vm::run returns Err(*error.take().unwrap())
}
```

The asm-side `Exit` tag is just a signal; the bridge branches to the
exit shim and never inspects the slot. The exit shim (in Rust) reads
`rust_context.exit`:

- `kind == Done`: returns `Ok(self.done_value)`.
- `kind == Error`: returns `Err(*self.error.take().unwrap())`.
- `kind == None`: programming error (bridge observed `Exit` but slot
  was not populated); debug builds assert.

`VmError` is heap-allocated by the slow path on the error path. This
is fine because errors are rare; allocation cost is invisible at the
hot-path level. `done_value` is the `Value` returned by a successful
exit. Putting the exit slot off the asm-visible record means
`LlIntState` only has to carry per-dispatch state — the asm shape never
reads exit fields and never needs to know their layout.

## Pre-slow-path sync protocol — the load-bearing 5-step sequence

Before any semantic body reads from `state.frame.*`, the shim copies
asm-visible mirrors into the Rust-side snapshot. Without this, a
semantic body asking for `state.frame.instruction_offset()` would see
stale data from the last `Refresh` rather than the post-dispatch PC the
handler just set up.

Every slow-path shim follows this fixed sequence (verbatim from §6):

```text
1. Acquire LlIntState raw pointer (arg 0 / x0 / rdi).
2. Construct LlIntDispatchState<'_> via from_raw (casts rust_context back
   to the concrete type).
3. sync_from_asm():
   - rust_context.frame.instruction_offset  ← state.frame_pc_offset
   - rust_context.frame.registers_base       ← state.frame_regs_base
                                              (via id translation if needed)
   - rust_context.frame.feedback_vector_base ← state.frame_fv_base
                                              (similarly)
   - rust_context.frame.code_pb_base         ← state.frame_pb_base
   - rust_context.frame.depth_snapshot       ← state.frame_depth
4. Run op_xxx_semantic(&mut dispatch, args).
5. translate_outcome(outcome):
   For Continue { new_pc_offset }:
     SlowPathReturn { tag: Continue, payload: new_pc_offset as u64 }
     PLUS sync_to_asm(new_pc_offset):
       state.frame_pc_offset ← new_pc_offset
   For Refresh:
     sync_to_asm_full(): mirror all frame_* fields out of rust_context.frame
     SlowPathReturn { tag: Refresh, payload: 0 }
   For Exit { kind, payload }:
     rust_context.exit ← { kind, payload }
     SlowPathReturn { tag: Exit, payload: 0 }
```

Clarifying notes:

- **Step 2 is where the unsafe cast lives.** `from_raw` reads
  `(*state).rust_context`, casts it from `*mut LlIntRustContextOpaque`
  to `*mut LlIntRustContext<'_>`, and assembles a `LlIntDispatchState`
  wrapper combining `&mut LlIntState` + `&mut LlIntRustContext`. This
  is the entire unsafe surface of the slow-path bridge in normal
  execution.
- **Step 3 is one-way.** It populates the Rust snapshot from the asm
  mirrors; it does **not** push values back. The Rust snapshot's
  `frame.instruction_offset` will diverge from `state.frame_pc_offset`
  during semantic execution (e.g. when the slow path advances PC), and
  that's fine — step 5 reconciles.
- **Step 4 is regular safe Rust.** No `unsafe` inside semantic bodies;
  they take a `&mut LlIntDispatchState<'_>` and return a
  `SemanticOutcome`. The same semantic function backs both the DSL cold
  stub (via the shim) and, transitionally during DSL-0a, the alpha
  handler.
- **Step 5's `Continue` branch performs a single PC sync.** The slow
  path may have computed a new PC offset (e.g. `op_call_*` advancing
  past the call); `sync_to_asm(new_pc_offset)` writes it back to
  `state.frame_pc_offset` so the bridge's post-call dispatch can
  reconstruct `PC = pb_base + new_pc_offset`.
- **Step 5's `Refresh` branch performs a full sync.** Frame-altering
  paths (`op_call_*`, `op_return`) must mirror all `frame_*` fields
  out of `rust_context.frame` so the bridge can reload pinned
  registers for the new active frame.
- **Step 5's `Exit` branch writes the exit slot.** Both `Done` and
  `Error` populate `rust_context.exit` before returning the `Exit`
  tag. The asm bridge does not inspect the slot.

Operands pass as `u32` (not `u8`/`u16`) to avoid sign-/zero-extension
dance in ABI registers. Up to five operands fit in the four-or-five
arg-registers available across both ABIs (AArch64: x1–x5; x86_64 SysV:
rsi/rdx/rcx/r8/r9).

## Four-layer state-sync rules

There are four locations where interpreter state lives. The DSL ABI is
the rule-set that says when each is true:

| Layer                                          | Truth-at              | Synced from                              | Synced when                                                                  |
| ---------------------------------------------- | --------------------- | ---------------------------------------- | ---------------------------------------------------------------------------- |
| Pinned registers (`PC`, `REGS`, `FV`)          | hot path              | `LlIntState.frame_*`                     | only on entry and `Refresh` — otherwise pinned regs are truth                |
| `LlIntState.frame_*` (asm-visible mirrors)     | slow-path boundary    | pinned registers via pre-`call_slow!` sync | written before every `call_slow!`; read on `Refresh`                         |
| `LlIntRustContext.frame` (full `FrameRecord`)  | slow-path Rust code   | pinned regs + `vm.frames.last()`         | written by entry shim and by frame-changing slow paths (Call/Return)         |
| `vm.frames` (canonical)                        | always                | slow paths that mutate                   | call/return slow paths push/pop here; bridge mirrors into `rust_context.frame` |

Reading these rules forward (hot path):

1. Asm reads pinned registers (`PC`, `REGS`, `FV`). These are truth.
2. Before `call_slow!`, asm writes pinned registers → `state.frame_*`.
3. Slow-path shim reads `state.frame_*` → `rust_context.frame`
   (`sync_from_asm`). `rust_context.frame` is now consistent with the
   pre-call asm state.
4. Semantic body reads/writes `rust_context.frame` (and possibly
   `rust_context.vm.frames`, e.g. for Call/Return).
5. Shim writes outcome → `state.frame_*` (`sync_to_asm` or
   `sync_to_asm_full`) → bridge reloads pinned registers on `Refresh`.

Reading them backward (entry):

1. `Vm::run`'s entry shim builds `vm.frames` to hold the initial
   frame.
2. Entry shim mirrors the initial frame into `rust_context.frame`
   (the canonical full snapshot).
3. Entry shim copies the mirror into `LlIntState.frame_*`.
4. Entry shim loads `LlIntState.frame_*` into pinned registers.
5. Dispatch begins; pinned registers are now the source of truth until
   the next `call_slow!`.

The invariant binding all four layers: at every `call_slow!` boundary,
the layers must be **convergent** — every layer's reading of "the
current frame" must produce the same observable answer. Slow-path
shims own this. Bridge code is mechanical; the shim's
`sync_from_asm` / `translate_outcome` pair is the load-bearing
discipline.

## Symbol mangling and the `sym` operand

Rust's `naked_asm!` macro forbids `in/out/inout` operand constraints;
it supports only `sym` (compile-time symbols) and `const` (compile-time
integers). The slow-path call is therefore **not** a textual
`bl op_xxx_slow_rs` — that would bake the unmangled name into the asm
and break on any platform that uses symbol prefixing (macOS prefixes
external symbols with an underscore; Windows uses different
conventions). Instead:

```rust
core::arch::naked_asm!(
    // ...
    "bl {slow_rs}",                          // operand-bound call
    // ...
    slow_rs        = sym op_xxx_slow_rs,
    state_pb_off   = const offset_of!(LlIntState, frame_pb_base),
    state_pc_off   = const offset_of!(LlIntState, frame_pc_offset),
    state_regs_off = const offset_of!(LlIntState, frame_regs_base),
    state_fv_off   = const offset_of!(LlIntState, frame_fv_base),
)
```

- `slow_rs = sym op_xxx_slow_rs` — rustc resolves the symbol through
  the platform's name-mangling pipeline. The assembler sees the right
  external name on each target.
- `state_pb_off = const ...` — the field offset is substituted as an
  integer literal. There is no textual literal `8` anywhere in the
  source; if `LlIntState`'s layout shifts, the substituted constant
  shifts with it.
- The DSL backend emits these bindings programmatically: every
  per-operation macro that needs a field offset declares the
  corresponding `const` binding, and the proc-macro collects them into
  the single `naked_asm!` block.

This is also why the bridge sketch in §6 of the design uses
`{slow_rs}` / `{state_pb_off}` / `{state_pc_off}` / `{state_regs_off}`
/ `{state_fv_off}` / `{interpreter_exit}` rather than textual symbols
or numeric offsets. The `naked_asm!` block looks like this on AArch64
(simplified from §6):

```asm
; Pre-call: sync PC offset into state.frame_pc_offset
ldr  x9,  [x24, {state_pb_off}]
sub  x10, x19, x9                        ; PC pointer → offset
str  w10, [x24, {state_pc_off}]

; Move pinned STATE into arg 0, operands into a1..aN
mov  x0,  x24
mov  w1,  <operand a>
mov  w2,  <operand b>

bl   {slow_rs}                            ; sym-bound call; platform symbol mangling handled by rustc

; Post-call dispatch — Continue is the common case
cbnz x0,  .unusual                       ; tag != Continue
ldr  x9,  [x24, {state_pb_off}]
add  x19, x9,  x1                        ; PC = pb_base + new_offset
ldrb w8,  [x19]
ldr  x10, [x23, x8, lsl #3]
br   x10                                  ; tail-dispatch

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
b    {interpreter_exit}                   ; sym-bound exit shim
```

Common-case bridge cost (`Continue`): roughly 10–15 instructions
including the pre-call sync. Comparable to JSC LLInt's
`callSlowPath` + `dispatch()`.

## No-unwind policy

Slow paths are `extern "C"`, which under modern Rust aborts on panic
(the panic-abort contract). Slow paths must not panic, and the
controlled error path is `SlowPathTag::Exit` + `LlIntExitSlot`, never
stack unwinding.

Layered defense:

1. **Release builds.** `extern "C"`'s built-in panic-abort behavior is
   the floor. A panic inside a slow-path semantic body aborts the
   process cleanly. (This is the same contract every panicking Rust
   FFI boundary lives under.)
2. **Debug builds.** Each shim wraps the semantic body in
   `std::panic::catch_unwind`. If a panic escapes, the shim logs a
   diagnostic and `std::process::abort`s. This is strictly stricter
   than the release-build floor — `catch_unwind` would normally
   convert the panic into a returned error, but the DSL bridge does
   not honor that; the abort is intentional, to make the policy
   violation visible in tests.
3. **Static check.** Slow-path semantic bodies live in
   `vm/src/dsl/slow_path.rs` and per-opcode `*_slow_rs` modules.
   These are normal Rust, and they may use `?` and return
   `SemanticOutcome::Error(...)` — but they must never `panic!`,
   `unwrap` an `Err`, or `unreachable!`. Lints / review enforce this.

The reason the policy is this strict: **no path of control flow returns
from a slow-path call via stack unwinding**. The asm bridge has no
unwind tables in the `naked_asm!` body, and unwinding through the
bridge would corrupt the interpreter's invariants (pinned registers
not restored, `LlIntState.frame_*` left mid-sync). All error propagation
travels through the `Exit` tag and the exit slot; that is the only
error channel.

## Required invariant tests for DSL-0b

The asm-DSL substrate cannot be exercised until DSL-0b lands; DSL-0a
proves the macros compile and produce structurally correct asm, and
DSL-0b proves the contract above holds at runtime. The following
tests must exist before any DSL-0c work begins:

1. **Offset stability test** — `vm/src/dsl/reg_convention.rs` includes
   a `#[test]` that calls `offset_of!` on each `LlIntState` field and
   asserts the expected literal value. The expected values are the
   computed offsets above:

   ```rust
   #[test]
   fn llint_state_field_offsets_are_stable() {
       use core::mem::{offset_of, size_of};
       assert_eq!(offset_of!(LlIntState, frame_pc_offset),    0);
       assert_eq!(offset_of!(LlIntState, frame_pb_base),      8);
       assert_eq!(offset_of!(LlIntState, frame_regs_base),   16);
       assert_eq!(offset_of!(LlIntState, frame_fv_base),     24);
       assert_eq!(offset_of!(LlIntState, frame_depth),       32);
       assert_eq!(offset_of!(LlIntState, frame_check_epoch), 36);
       assert_eq!(offset_of!(LlIntState, rust_context),      40);
       assert_eq!(offset_of!(LlIntState, prefix),            48);
       assert_eq!(size_of::<LlIntState>(),                   56);
   }
   ```

   If a future rustc, target triple, or unrelated field-set change
   perturbs any offset, this test fires before the generated `naked_asm!`
   blocks compile against the wrong literal.

2. **Miri pass on the shim layer** — `vm/src/dsl/slow_path.rs`'s
   shim helpers (`LlIntDispatchState::from_raw`, `sync_from_asm`,
   `sync_to_asm`, `sync_to_asm_full`, `translate_outcome`) are
   exercised by unit tests that construct a `LlIntState` and a
   `LlIntRustContext`, perform the pointer dance manually, and check
   round-trip equality. Tests run under `cargo +nightly miri test
   -p lyng-vm slow_path` to catch UB in the pointer casts.

3. **Post-dispatch PC-sync invariant test** — a cold-stub opcode whose
   semantic body reads `state.frame.instruction_offset()` and asserts
   it equals the byte offset immediately past the dispatching opcode.
   This verifies that step 3 of the pre-slow-path sync sequence
   actually runs and that the asm bridge writes `state.frame_pc_offset`
   correctly before `bl {slow_rs}`. Lands as one of the DSL-0b
   validation cases (called out explicitly in §6 of the design).

4. **`LlIntRustContextOpaque` round-trip test** — construct a
   `LlIntRustContext<'_>`, cast its pointer to
   `*mut LlIntRustContextOpaque`, store it in
   `LlIntState.rust_context`, recover it via `from_raw`, and confirm
   the recovered reference points to the same allocation as the
   original (`std::ptr::eq` on the underlying address). Runs under
   Miri as well, since the cast is the load-bearing unsafe step.

5. **Refresh-discipline test** — synthesize a `LlIntState` with
   deliberately stale `frame_pc_offset`, populate
   `rust_context.frame.instruction_offset` with the "correct" value,
   call `translate_outcome(SemanticOutcome::Refresh)`, and confirm
   `state.frame_pc_offset` is now equal to the correct value (and
   `state.frame_regs_base` / `state.frame_fv_base` / `state.frame_pb_base`
   were similarly written back). Verifies that `sync_to_asm_full`
   covers all four frame mirrors.

6. **Exit-slot round-trip test** — call a slow-path semantic body that
   returns `SemanticOutcome::Exit { kind: Done, value }`, run
   `translate_outcome`, observe the returned `SlowPathReturn { tag:
   Exit, payload: 0 }` and the populated `rust_context.exit.{kind,
   done_value}`. Symmetric test for the `Error` path with an injected
   `VmError`.

These tests, together with the existing register-pin baseline produced
by DSL-0a's empty-body handlers, form the contract that DSL-0c (the
first batch of real opcodes ported into the DSL) can be checked
against. A failure in any of them is a hard stop on DSL-0c work — the
ABI is wrong, and shipping handlers over a wrong ABI guarantees subtle
correctness bugs at scale.

## Why the ABI has this exact shape (summary)

- **`#[repr(C)]` everywhere asm reads.** No Rust enums, trait objects,
  or lifetimes in the asm-visible record, because their layout is not
  stable across rustc versions.
- **Single opaque pointer for everything else.** Trait objects,
  lifetimes, `Arc`s, full `FrameRecord` snapshots all live behind one
  thin `rust_context` pointer that asm only ever passes through, never
  dereferences.
- **PC as pointer, PC offset as mirror.** Pinned `PC` is one
  pointer-load away from the next opcode byte; `frame_pc_offset` is one
  subtraction away when the slow path needs to sync.
- **`FV` as a mutable base pointer.** Required by `record_*!` and
  `value_profile!` writes; matches the flat IC-entry array refactor
  scheduled with DSL.
- **Three dispatch tags, two exit kinds.** Hot-path tag testing
  collapses to a single nonzero branch (`Continue` = 0); the rare
  `Exit` path discriminates `Done` vs `Error` out of band via the
  exit slot.
- **Pre-call sync, then return-and-resync.** The 5-step protocol is
  identical for every shim; macroized so handler authors cannot
  accidentally skip a step.
- **No-unwind, panic-abort.** Errors travel only through the exit
  slot; no stack-unwind path crosses the asm bridge.
- **Symbol bindings via `sym`, offsets via `const`.** Naked asm uses
  zero textual symbols and zero numeric literals — every external
  reference is operand-bound, so platform mangling and offset
  evolution are handled by rustc, not by handler authors.

The contract is small (one `repr(C)` struct, six pinned registers, one
return ABI, one 5-step sync sequence) but every part of it is
load-bearing: violating any one piece produces UB at scale because
every generated handler shares the same bridge. The invariant tests
above exist precisely to catch a violation at the moment it lands,
not 50 opcodes later when handler debugging becomes intractable.
