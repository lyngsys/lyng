# asm-DSL safepoints, polling, and prefix dispatch — R-0 evidence report

This report locks in the same-thread polling model, the warm-handler
safepoint coverage, the prefix dispatch semantics, and the explicit
deferral of tier accounting from DSL-0. It is the third of three R-0
evidence reports required before DSL-0 begins (the other two cover
the `Value` layout and the handler ABI). The producer/consumer model,
the asm-level poll shape, and the six DSL-0b invariant tests captured
here are normative — they define the surface that DSL-0a's manifest
hands to DSL-0b for porting.

## Source

Design: [docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md](../../../docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md)

- §6, "GC safepoints and the warm-opcode poll model" — lines 389-407
- §6, "Prefix dispatch (`op_wide`, `op_extra_wide`)" — lines 409-421
- §10 DSL-0c, "Delete tier-accounting machinery on backedges" — line 750

Current code (cited line numbers below):

- `crates/vm/src/vm/dispatch_handlers/control_flow.rs` — `op_jump`,
  `op_jump8`, `op_loop_header`, conditional jump handlers
- `crates/vm/src/vm/dispatch_handlers/prefix.rs` — `op_wide`,
  `op_extra_wide`, `dispatch_prefixed`
- `crates/vm/src/vm/tiering.rs` — `observe_tier_backedge_event`
- `crates/vm/src/vm.rs` — `request_debug_pause`,
  `request_debug_pause_at`, `debug_poll_enabled`, `poll_debug_safepoint`,
  `poll_incremental_mark_safepoint`
- `crates/vm/src/vm/debugger.rs` — `should_poll`, `consume_pause`

## Today's safepoint surface (alpha path)

The alpha trampoline already runs the polling work this report describes;
DSL-0 changes the dispatch shape, not the semantics. Today's poll sites
fire at backedge-bearing handlers and at the explicit loop-header marker.
Each site performs three pieces of work, in order: a debug poll (only
the loop-header site, gated on `debug_poll_enabled`), a tier-backedge
observation, and an incremental-mark step. The DSL substrate inherits
the same coverage with two material changes: the bitmap-style
`poll_pending` byte replaces the always-call shape, and the tier-backedge
observation goes away with DSL-0c (see "Deferred work" below).

### `op_loop_header`

[crates/vm/src/vm/dispatch_handlers/control_flow.rs:75-93]

```rust
pub extern "C" fn op_loop_header(state: &mut DispatchState) -> Step {
    // ... decode ...
    if state.vm.debug_poll_enabled() {
        state.sync_active_frame();
        {
            let DispatchState { vm, agent, .. } = &mut *state;
            vm.poll_debug_safepoint(agent, VmDebugSafepointKind::LoopHeader);
        }
        try_step!(state.refresh_from_active_frame());
    }
    state.vm.observe_tier_backedge_event(code);
    Vm::poll_incremental_mark_safepoint(state.agent);
    advance_dispatch_frame(&mut state.frame, instruction_len);
    dispatch_next!(state);
}
```

This is the only handler that runs the debug poll today. The debug poll
goes through `state.sync_active_frame()` and a refresh because the hook
is allowed to mutate frame state.

### `op_jump`, `op_jump8` (backward only)

[crates/vm/src/vm/dispatch_handlers/control_flow.rs:39-73]

Both jump handlers branch on `ax < 0` and run the tier-backedge plus
incremental-mark poll only on negative offsets:

```rust
if ax < 0 {
    state.vm.observe_tier_backedge_event(code);
    Vm::poll_incremental_mark_safepoint(state.agent);
}
```

Forward jumps do no polling. No debug poll fires at `op_jump`/`op_jump8`
today.

### `op_jump_if_true` / `op_jump_if_false` (both i24 and i8 variants)

[crates/vm/src/vm/dispatch_handlers/control_flow.rs:117-200]

The shared `op_jump_if_impl` handler polls only when the jump is both
taken and negative:

```rust
if should_jump {
    if delta < 0 {
        Vm::poll_incremental_mark_safepoint(state.agent);
    }
    // ...
}
```

Note the asymmetry with the unconditional `op_jump`: conditional jumps
poll only the incremental-mark step, not `observe_tier_backedge_event`.
DSL-0a preserves this exact shape on the alpha path; DSL-0b adds the
`poll_pending` check to the warm DSL handlers.

### `poll_incremental_mark_safepoint`

[crates/vm/src/vm.rs:598-601]

```rust
pub(super) fn poll_incremental_mark_safepoint(agent: &mut Agent) {
    let _ = agent.heap_mut().poll_incremental_mark_step();
}
```

A free function — does not touch `&mut self` on the VM. The DSL slow
path inherits this signature: the poll slow-path is a Rust function
that takes `&mut Agent` plus whatever VM state it needs through the
`LlIntState`/`LlIntRustContext` pair.

### `observe_tier_backedge_event`

[crates/vm/src/vm/tiering.rs:211-219]

```rust
pub(super) fn observe_tier_backedge_event(&mut self, code: CodeRef) {
    if let Some(state) = self
        .tiering
        .get_mut(code_index(code))
        .and_then(Option::as_mut)
    {
        state.observe_backedge_event();
    }
}
```

Bumps the per-code-block `backedge_events` counter and the `hotness`
score; when `hotness >= TIER_READY_HOTNESS_THRESHOLD` (8, per
tiering.rs:4), the code block moves to `TierStatus::ReadyForNative`. The
counter has no current consumer beyond `tiering_snapshot` for tests —
no JIT exists yet. This is exactly the machinery that DSL-0c deletes.

## DSL-0 polling model (same-thread)

### Storage

`Vm.poll_pending: u8` — a plain byte (NOT `AtomicU8`). Same-thread
access only in DSL-0. The agent-thread VM state is not currently
`Send`/`Sync`; the debugger API is `&mut self` (see
`request_debug_pause` and `request_debug_pause_at` at vm.rs:439 and
vm.rs:443, both `const fn`); the DSL substrate inherits those
constraints. Design §6 lines 397, 403.

Read by the asm path via `ldrb [VM, #VM_POLL_PENDING_OFFSET]` directly
— `VM` is pinned in x22 (AArch64) / r15 (x86_64) per the ABI report,
so the read is one instruction with no `LlIntState` indirection. Design
§6 line 397.

### Producers (same-thread only)

| Producer | Sets bit | When |
|---|---|---|
| GC scheduler | `GC_PENDING (0x01)` | Major collection due or incremental mark needs progress. Set during slow-path execution. |
| Debugger | `DEBUG_PAUSE (0x02)` | `Vm::request_debug_pause` (vm.rs:439) or `Vm::request_debug_pause_at` (vm.rs:443) called. Set during slow-path execution or between `Vm::run` invocations. |

Cross-thread producers are EXPLICITLY OUT OF SCOPE for DSL-0. If a
cross-thread requirement appears, see design §6 lines 402-403 for the
separate ticket that addresses the full synchronization surface (hook
handoff, pause-request payload, atomic semantics, memory ordering, and
the asm-side load semantics under contention).

### Consumers

Warm-handler poll slow paths, one per backedge-bearing handler:

| Handler | Slow-path symbol | Today's equivalent work |
|---|---|---|
| `op_loop_header` | `op_loop_header_poll_rs` | debug poll + tier observe + incremental-mark step (control_flow.rs:75-93) |
| `op_jump` (negative) | `op_jump_poll_rs` | incremental-mark step (control_flow.rs:45-48) |
| `op_jump8` (negative) | `op_jump8_poll_rs` | incremental-mark step (control_flow.rs:63-66) |
| `op_jump_if_true` (taken negative) | `op_jump_if_true_poll_rs` | incremental-mark step (control_flow.rs:140-142) |
| `op_jump_if_false` (taken negative) | `op_jump_if_false_poll_rs` | incremental-mark step (control_flow.rs:140-142) |
| `op_jump_if_true8` (taken negative) | `op_jump_if_true8_poll_rs` | incremental-mark step (control_flow.rs:140-142) |
| `op_jump_if_false8` (taken negative) | `op_jump_if_false8_poll_rs` | incremental-mark step (control_flow.rs:140-142) |

Each slow-path entry reads `Vm.poll_pending`, runs the relevant work
(GC step via `poll_incremental_mark_safepoint`, debugger pause via
`poll_debug_safepoint` for `op_loop_header_poll_rs`), and clears the
consumed bits.

Note that DSL-0b extends the debug-poll surface to all backedge
handlers, not just `op_loop_header` — the bit is shared between
producers, so any consumer that reads `DEBUG_PAUSE` must handle it
correctly. See design §6 line 401.

### Memory model

Single-threaded access only. The asm-side `ldrb` and Rust-side `u8`
writes need no memory-ordering machinery. If DSL-0 ever needs to be
made thread-safe, the byte upgrades to `AtomicU8`, the asm-side load
is documented as a non-atomic but acquire-equivalent read of an atomic
location (architecturally fine on AArch64/x86_64 for byte loads but
explicitly justified in writing), and the cross-thread protocol gets
its own ticket. Design §6 line 403.

## Warm-handler asm shape

The DSL emits the following sequence at every poll-bearing site (after
the operation's fast-path semantic work, before `dispatch_next!`):

```asm
    ldrb    w_scratch, [VM, #VM_POLL_PENDING_OFFSET]
    cbz     w_scratch, .no_poll
    bl      {poll_slow_rs}
    ; The slow path completes, returns, and the warm handler
    ; performs dispatch_after_slow! to resume at the next opcode.
    b       .resume_after_slow
.no_poll:
    ; ... continue with the warm handler's fast-path tail ...
.resume_after_slow:
    ; ... fall through to dispatch_next! ...
```

Two instructions on the fast path (`ldrb` + `cbz`), zero work when the
flag is clear. `VM` is the pinned VM register (x22 on AArch64, r15 on
x86_64 — see [llint-dsl-abi.md](./llint-dsl-abi.md)). Design §6 line
393.

## Invariant tests required before DSL-0b

DSL-0a writes these as harness fixtures; DSL-0b's port of each handler
must keep them green. Tests 1-3 cover safepoint coverage; tests 4-6
cover prefix dispatch (next section).

1. **Loop-header poll fires.** A tight `op_add` + `op_loop_header` loop
   with `poll_pending = GC_PENDING` set externally reaches the GC slow
   path within ~K iterations (concrete K calibrated during DSL-0a; a
   single backedge is sufficient because `op_loop_header` is the
   loop's only backedge marker).
2. **Backward-jump poll fires.** Same shape with `op_jump`-back (no
   `op_loop_header`) — confirms the warm-handler poll branch in
   `op_jump`'s DSL body is reached.
3. **Conditional backward-jump poll fires.** Same shape with a taken
   negative `op_jump_if_true` — confirms the poll branch fires on the
   taken-backedge path of the conditional jump warm handler.

Each test is repeated for `op_jump8` and the i8-variant conditional
jumps, since they are independent handlers sharing the dispatch table.

## Prefix dispatch semantics

[crates/vm/src/vm/dispatch_handlers/prefix.rs:16-48]

The current alpha trampoline implements prefix dispatch as:

```rust
fn dispatch_prefixed(state: &mut DispatchState, prefix: Opcode) -> Step {
    let semantic_byte = match state.current_bytes().get(1).copied() {
        Some(b) => b,
        None => return Step::Error(VmError::InstructionOutOfBounds { ... }),
    };
    if semantic_byte == Opcode::Wide as u8 || semantic_byte == Opcode::ExtraWide as u8 {
        return Step::Error(VmError::InstructionOutOfBounds { ... });
    }
    state.prefix = Some(prefix);
    Step::Continue(DISPATCH_TABLE[semantic_byte as usize])
}
```

This is *almost* the DSL shape, with the alpha `Step::Continue`
fall-through replaced by an asm tail-dispatch and `state.prefix` going
from `Option<Opcode>` to a u8 byte at a known `LlIntState` offset.

`op_wide` and `op_extra_wide` are **warm** handlers (not cold stubs) —
they have small bodies that:

1. Read `pc[1]` (the semantic opcode byte).
2. Reject doubled prefixes: branch to error if
   `LlIntState.prefix != 0`, or if the semantic byte is itself a
   prefix opcode. (Alpha uses the latter check; the DSL substrate uses
   the `state.prefix != 0` check that design §6 specifies, which is
   strictly stronger and catches more corruption.)
3. Set `LlIntState.prefix` to 1 (`Wide`) or 2 (`ExtraWide`).
4. Advance PC by 1 (past the prefix byte).
5. Tail-dispatch to the semantic handler at the new PC.

Semantic handlers consume `state.prefix` via their layout decoders
(auto-generated by `llint_handler!` — design §6 line 419). They read
it, decode operands at the wider width, advance PC past the wider
body, and clear `state.prefix` to 0 before tail-dispatching. The
single-implementation invariant (§10) covers this: prefix decoding
logic does not split between alpha and DSL.

### Prefix invariant tests required before DSL-0b

4. **Wide-prefixed `op_move` decodes correctly.** Wide-prefixed
   `op_move r256, r257` reads the right registers (registers ≥ 256
   require the Wide prefix; this is the canonical "wide changes
   operand width" test).
5. **ExtraWide-prefixed `op_move` decodes correctly.** Same with
   Wide-32 (registers ≥ 65536 require ExtraWide).
6. **Double-prefix raises error.** `op_wide; op_wide; op_move ...`
   raises the expected `VmError::InstructionOutOfBounds` variant
   (matching the current alpha behavior at prefix.rs:31-36).

These are design §6 lines 419-421's "three prefix cases" promoted to
the DSL-0b validation manifest.

## Deferred work (out of DSL-0 scope)

### Tier accounting on backedges

The existing `observe_tier_backedge_event` (tiering.rs:211-219) stays
alive on the alpha path through DSL-0a and DSL-0b, and **deletes with
alpha in DSL-0c** (design §10 line 750). After DSL-0c, the interpreter
has no tier-up accounting. This is intentional: per §2, JIT is out of
scope for DSL-0; per §6's same-thread `poll_pending` design, there is
no `TIER_UP_PENDING` bit and no scheduled producer for one.

The DSL substrate does not add a tier counter, a tier-up slow-path
hook, or any reference to `TIER_READY_HOTNESS_THRESHOLD` in DSL-0b's
warm handlers. When the JIT track resumes, tier accounting comes back
as part of that effort with its own design — design §11 (line 899)
captures this as an explicit open question: "Tier accounting
re-introduction shape." Out of scope for DSL-0, but specified to
prevent accidental reintroduction during the rollout.

### Cross-thread debugger pause

Same-thread only in DSL-0. The current `request_debug_pause`
(vm.rs:439) and `request_debug_pause_at` (vm.rs:443) APIs are both
`pub const fn ... (&mut self, ...)`, which by Rust's borrow rules can
only be invoked when no other reference to `self` exists — i.e., not
from another thread observing a running interpreter. The DSL
substrate inherits this constraint exactly.

If a real cross-thread requirement appears, that's a separate design
ticket. The minimum surface it has to cover, per design §6 line 402:
hook handoff (who owns the `Box<dyn VmDebugHook>` during a pause
crossing thread boundaries), pause-request payload (does the request
carry frame-targeting metadata that has to survive a cross-thread
fence), atomic semantics (`AtomicU8` vs the bare `u8`), memory
ordering (the asm-side `ldrb` becomes a documented
acquire-equivalent), and the cross-thread protocol's failure modes
(what happens if the hook returns a continue command after the VM has
already torn down).

### Tier-up signal under DSL

Not in DSL-0 or DSL-0c. When the JIT track resumes, the design picks
between three shapes:

- A new `poll_pending` bit (`TIER_UP_PENDING (0x04)`) with same-thread
  or cross-thread semantics, depending on whether the tier scheduler
  runs on the VM thread or elsewhere.
- A separate per-`CodeBlock` counter accessed via
  `rust_context.installed` (no warm-handler asm change required).
- A different mechanism entirely (osr-on-return, profiling sample,
  scheduler-driven invalidation).

The choice is deferred. Design §11 line 899 records the question;
this report records that the answer does not constrain the DSL-0
warm-handler shape, because no current code path consumes any of
the candidate signals.
