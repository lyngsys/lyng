# SP-0b — Unified Register-File Frame Arena (Design)

**Date:** 2026-05-31
**Status:** Design — approved for planning
**Phase:** SP-0b of the inline-asm call-path program (Stage 2 of V8-v7 perf work)
**Prerequisite reading:**
`docs/superpowers/specs/2026-05-31-asm-call-frame-architecture-research.md` (JSC register-file
synthesis) and `docs/superpowers/specs/2026-05-31-sp0a-eliminate-execution-context-design.md`
(the merged predecessor). Project memory: `project_asm_call_frame_rearchitecture`.

## Why this exists

Stage 2's broad perf lever is an inline-asm call path for environment-free callees; its enabler
(SP-0) is making call-frame setup doable in asm. SP-0a deleted the separate `ExecutionContext`
stack so the **frame is the single source of truth**. Lyng today still pushes **two growable
Rust structures** per activation — `frames: Vec<FrameRecord>` and `register_stack: Vec<Value>` —
and **neither is asm-addressable**: a `Vec` realloc relocates the backing buffer, which would
invalidate any live frame/register pointer an asm call path holds in a register across a push.

SP-0b is the **storage + invariants** sub-phase. It collapses both Vecs into **one
pre-reserved, never-realloc bump arena** in JSC's register-file shape: each frame is a fixed
`repr(C)` header at `offset_of!`-pinned offsets, immediately followed by that frame's value
window (its registers). Realm stops being stored and is **derived**. A **soft-limit prologue
check** sized for the whole callee frame is the safety property that will later let asm push
frames branch-free. **No asm executes in SP-0b** — it is pure Rust, behavior-preserving, with
Test262 staying 100% after every commit.

## Locked decisions (from the SP-0b brainstorm)

1. **Storage model = "A": one typed arena.** A single never-realloc bump arena of 64-bit
   slots. Each frame is `[repr(C) FrameHeader][value window]`. `cfr` points at the header;
   `regs_base = cfr + header_slots`. GC walks the `caller_cfr` chain and traces each header's
   **typed** ref fields plus that frame's window as Values. **Rejected:** the literal JSC
   type-punned `Register` union (Lyng's header fields are `NonZeroU32` handles, not 64-bit
   pointers; a punned `u64` array would either waste half each slot or pack two handles per
   slot — destroying the uniformity that motivates punning — and would force GC to trace raw
   bits via an external slot-kind map with no compiler-checked field types). **Rejected:**
   Value-tag-encoded handles (pollutes the NaN-box tag space with engine-internal kinds that
   must never leak to JS-visible slots). The typed `repr(C)` header is continuous with Lyng's
   existing asm contract (`LlIntState` is already a pinned `repr(C)` struct, not a punned
   array) and is the right long-term fit for Lyng's handle-based heap.

2. **Field mapping = tiered header + derive + cold side-table.** The `repr(C)` header is
   internally tiered into an **asm-hot** cluster (offsets the asm path depends on) and an
   **interpreter-warm** cluster (the asm fast path never reads it). **Derived, not stored:**
   realm, referrer, executable, register-window geometry. **Cold Rust-only side-table:** the
   truly-rare per-activation state (generator resume, tail-caller, handler cursor, parameter
   initializer end offset). **Rejected:** putting everything in the header (carries a
   Value-sized resume slot + rare state on every frame, bloats the arg-overlap gap, keeps no
   derivation discipline). **Rejected:** maximal JSC minimalism with scope moved into
   value-window registers (`scopeRegister`) — env-free callees never read scope on the fast
   path, so VR-punning the envs buys the asm nothing and re-architects every scope-resolution
   opcode + the compiler against Lyng's field-based env model.

3. **Realm = fully derived, nothing stored.** No `realm` field anywhere on the frame.
   Current-frame reads use the `Agent.running_context` scalar (added in SP-0a). Function
   frames derive from the callee's `[[Realm]]`; root frames carry realm in the establishment
   side-stack (see §D). **Rejected:** a cold-table realm cache (keeps a redundant word the
   model is meant to derive) and deferring derivation to a follow-up (leaves the SP-0a
   deferral open).

4. **Arena backing = eager fixed `Box<[u64]>` behind a swap-ready interface.** Never realloc;
   `soft_limit = capacity − slack`; overflow throws `RangeError`. The arena is an opaque type
   whose only public surface is `base()`, a bump/release cursor, the soft-limit check, and a
   `try_grow` hook (a no-op for the eager backing). **Rejected for now:** lazy-commit
   reservation (`mmap`/`VirtualAlloc` MEM_RESERVE + commit-on-soft-limit + guard page). It is
   the better RSS end-state, but it produces a **byte-identical asm contract** and drops in
   behind this exact interface, so doing it now would only inject platform-divergent `unsafe`
   into a behavior-preserving Test262-parity refactor. Deferred to its own focused phase
   (naturally alongside SP-1/SP-2, sooner if many Vms coexist per process).

## Current state (as explored 2026-05-31, post-SP-0a merge)

- **`frames: Vec<FrameRecord>`** (`vm.rs:139`) — growable; `FrameRecord` ≈ 120 B, all `Copy`,
  **not** `repr(C)`, split into `FrameMetadata` (stable: `code`, `registers:RegisterWindow`,
  `return_register`, `realm`, `variable_env`, `private_env`, `new_target`, `callee`, `kind`,
  `parameter_initializer_end_offset`) and `FrameState` (mutable: `instruction_offset`,
  `lexical_env`, `this_value`, `this_state`, `construct_this`, `tail_caller`(+strict),
  `handler_cursor`, `flags`, `resume_kind`/`resume_value`/`resume_active`). Cap
  `MAX_BYTECODE_CALL_DEPTH = 8_192` (`bytecode_calls.rs:12`), checked at call entry →
  `RangeError`.
- **`register_stack: Vec<Value>` + `register_stack_top: usize`** (`vm.rs:134-135`) — the JS
  value stack; grows via `Vec::resize` at `reserve_register_window` (frame entry, on the Rust
  slow path); cursor moves on release, buffer never truncated. Already asm-addressed: its base
  is loaded into `LlIntState.frame_regs_base` and pinned into `x20` at trampoline entry
  (`dsl/entry.rs`), **re-derived per `run_via_dsl` entry** — which is the only reason today's
  reallocs are harmless (no live asm pointer survives a push).
- **`LlIntState`** (`dsl/llint_state.rs:30`) — the existing `#[repr(C)]`, 96 B, asm-addressable
  contract; offsets via `offset_of!` (`dsl/reg_convention.rs`) and **locked by
  `ll_int_state_offsets_stable()`** (`llint_state.rs:166`). This is the pattern SP-0b's frame
  header mirrors.
- **`Agent.running_context: Option<RunningContext>`** (`env/src/agent.rs`,
  `env/src/execution.rs:42`) — `RunningContext{ realm: RealmRef, referrer: Option<AtomId> }`,
  refreshed by `Vm::refresh_running_context` (`vm.rs:1371`) at every frame transition (~15
  sites), **not per-opcode**. Public `const`, readable by any `&Agent` holder.
- **Realm provenance:** function frames use `function_data(callee).realm()`
  (`objects/functions.rs:967`, pure — needs neither heap view nor agent;
  `unwrap_or_else(caller_frame.realm())` at `bytecode_calls.rs:165`); root frames
  (`synthetic_job_caller_frame` `jobs.rs:749`; module entry `modules.rs:530`; script entry)
  set realm from a `RealmRecord` in hand. `function_data(callee).realm()` is `Some` for every
  production callee kind (ordinary/native/embedding/bound/class/arrow/generator/async); Proxy
  callees never reach frame construction (the `apply` trap re-enters with a different callee).
- **`.realm()` readers** ≈ 98 sites: ~70–75 % read the **current** frame at opcode dispatch /
  single-frame helpers (`runtime_objects.rs`, `property_access.rs`, `call.rs`,
  `async_functions.rs`); ~10 % read **arbitrary**/suspended frames (mostly from
  `SuspendedExecutionRef` snapshots, not live frames); the rest are at frame boundaries.
- **GC frame tracing** (`vm/state.rs:204-305`): `ActiveVmRoots::trace_heap_edges` traces the
  **whole** `register_stack` Vec, then iterates `vm.frames` calling `trace_frame_record`, which
  marks `code, realm, lexical_env, variable_env, private_env, this_value, construct_this,
  new_target, callee, tail_caller, resume_value`. `RealmRef`/`EnvironmentRef` are real GC roots
  (`gc/rooting.rs:1564,1576` → `mark_environment`/`mark_realm`). Register stack is traced in its
  **entirety, not windowed**.

## Architecture

### A. The arena (`register_stack` is absorbed)

There is no separate value stack after SP-0b. One arena (`Box<[u64]>`, default ≈ 4 MB /
512 K slots) holds interleaved frames:

```
cfr →  [ FrameHeader : H × u64  (repr(C), typed) ]
       [ value window : M × u64  (this, args…, locals…, temps — Values) ]
regs_base = cfr + H        next_cfr = cfr + H + M
```

- The Rust register accessors (`read_register`/`write_register`/`register_stack[..]`) change
  from indexing the `register_stack` Vec to indexing the arena at `regs_base + idx`. Asm
  register addressing is **unchanged in form** — `[regs_base + idx*8]`; only the computation of
  `regs_base` changes (`cfr + H` instead of `register_stack_ptr + window_base`).
- `frames: Vec<FrameRecord>` and `register_stack: Vec<Value>` are both **deleted**, replaced by:
  the arena, a current `cfr`, and the existing `LlIntState.frame_depth` counter. A
  `caller_cfr` link in each header makes the activation chain explicit (the arena is not a Vec,
  so `.last()`/`frames[len-2]` no longer exist). `caller_cfr` is a **slot offset from the arena
  base** (not a raw pointer): realloc-irrelevant since the arena never moves, 32-bit-compact, and
  asm/GC resolve it as `base + caller_cfr`.
- **Opaque `FrameArena`** type. Public surface only: `base() -> *mut u64`, a bump/release
  cursor, the soft-limit check (§E), and a `try_grow` hook (no-op for the eager backing). The
  base is published into `LlIntState` so it is asm-addressable; lazy-commit later swaps the
  backing behind this interface with nothing above it changing.
- **Magnitude:** absorbing `register_stack` touches every register read/write and every
  `frames.last()/.len()/.pop()` site. The migration is mechanical and staged behind bridge
  accessors (§ Commit sequence), green per commit.

### B. The frame header (`repr(C)`, tiered, offset-pinned)

Tentative layout (exact packing finalized during implementation; offsets locked by test):

```
slot0:  caller_cfr:u32 | saved_pc:u32                                   ── asm-hot ──
slot1:  code:u32(CodeRef) | callee:u32(0 = root/none)
slot2:  this_value : Value (8 B)
slot3:  arg_count:u16 | return_reg:u16 | flags:u8 | this_state:u8 | kind:u8 | pad
slot4:  variable_env:u32 | lexical_env:u32                ── interpreter-warm (asm ignores) ──
slot5:  private_env:u32(0 = None) | new_target:u32(0 = None)
slot6:  construct_this:u32(0 = None) | pad:u32
```

- ≈ 56 B (7 slots). The **asm-hot** cluster (slots 0–3) carries exactly what a call prologue /
  in-frame fast path touches: caller link, saved caller PC, code, callee, `this`, arg count,
  flags. The **interpreter-warm** cluster (slots 4–6) is read by the interpreter but never by
  the env-free asm fast path.
- `saved_pc` is the **caller's** resume offset (JSC's `returnPC`); the *current* frame's live PC
  stays in `LlIntState.frame_pc_offset` / the pinned `x19` as today.
- `offset_of!` constants for each field + a `frame_header_offsets_stable()` lock-in test,
  mirroring `ll_int_state_offsets_stable()`.
- **Derived, never stored:** realm (§D), referrer (§D), executable (`kind`+`code`), `regs_base`
  (`= cfr + H`), window length (from `code`).
- **Cold side-table** (§C): `handler_cursor`, `tail_caller`(+strict), generator
  `resume_kind`/`resume_active`/`resume_value`, `parameter_initializer_end_offset`.
- `new_target` keeps its own warm slot (no JSC-style aliasing into the `this` slot — that
  micro-opt is out of scope).

### C. The cold side-table

A depth-indexed, Rust-only structure (never asm-addressed) holding the rare per-activation
state listed above. Keyed by `frame_depth` (already tracked). **Reset to default on push.** The
reset is a handful of small writes — acceptable for SP-0b (no asm fast path yet); in SP-1 the
env-free eligibility bit guarantees no cold state, so the asm prologue skips the cold table
entirely. GC traces the cold table's ref fields (`tail_caller`, `resume_value`) from here, since
they no longer live on the header.

### D. Realm and referrer derivation

Neither realm nor referrer is stored on the frame header.

- **Establishment side-stack** generalizes SP-0a's `referrer_scopes` to entries
  `(base_frame_depth, realm, referrer)`. An entry is pushed at each of the **four establishment
  points** (script root, module root, job root, restored generator frame) and unwound with its
  frames. It must enumerate to **exactly** SP-0a's referrer establishment set.
- **`realm_of(agent, frame, depth)`** =
  `frame.callee.is_some() ? function_data(callee).realm() : establishment.covering(depth).realm`.
  Function frames derive from the callee (pure, cheap); root frames read the covering
  establishment entry.
- **`referrer(depth)`** = the covering establishment entry's referrer (the existing walk), for
  all frames — reproducing today's inherit-from-caller behavior exactly. Generators
  capture-and-restore the referrer across suspend/resume as in SP-0a (a pure re-walk at resume
  would read the resume-time stack and be wrong).
- **Current realm** (≈ 70 % of readers, at opcode dispatch) reads the
  `Agent.running_context.realm` scalar — correct because the current frame's realm only changes
  at transitions, where the scalar is refreshed. `refresh_running_context` computes realm via
  `realm_of` at each transition. **Arbitrary-frame readers** (≈ 10 %) call `realm_of` directly;
  suspended-frame realm continues to come from `SuspendedExecutionRef`.
- **GC:** the side-stack traces its `realm` entries (root-frame realms stay rooted);
  function-frame realms remain reachable via the traced `callee`. **No GC root is lost** by
  dropping the stored `realm`.

### E. Soft-limit prologue and reservation

- Eager `Box<[u64]>`; `soft_limit = capacity − slack`. `slack` is sized so the `RangeError`
  throw path — which itself needs a frame + window — runs **inside** the reserve.
- Each frame push checks `cfr + H + window_len ≥ soft_limit → throw RangeError`. `try_grow` is a
  no-op for the eager backing (it cannot grow, so soft-limit overflow always throws). The
  byte-budget soft limit replaces/augments the `MAX_BYTECODE_CALL_DEPTH` cap.
- `Box<[u64]>` zero-fill and (future) mmap commit-zero give identical zero-on-first-touch
  semantics, so no consumer can depend on a difference.

### F. GC over the interleaved arena

The wholesale `register_stack` trace is **removed** — header slots are not Values and must never
be traced as Values. Replace it with a **frame walk** over the arena (`frame_depth` /
`caller_cfr` chain). For each frame:

- Trace the typed header refs: `code, callee, new_target, construct_this, this_value,
  variable_env, lexical_env, private_env`.
- Trace that frame's window `[regs_base .. regs_base + window_len]` as Values.

Plus: trace the cold side-table's refs and the establishment side-stack's realm entries. The
synthetic entry `caller_frame` root is folded into the walk.

## Commit sequence (each commit compiles + Test262 100%)

1. **Stand up the arena + header behind bridge accessors** *(no behavior change)*: introduce
   `FrameArena` (eager `Box<[u64]>`), the `repr(C)` `FrameHeader` with `offset_of!` constants +
   `frame_header_offsets_stable()`, and the cold side-table — populated **alongside** the
   existing `frames`/`register_stack` so readers are untouched. Publish the arena base into
   `LlIntState`.
2. **Migrate register access**: point `read_register`/`write_register`/window reservation at the
   arena windows; retire `register_stack`/`register_stack_top` once no reader remains.
3. **Migrate frame access**: `.last()/.last_mut()` → current-`cfr` header access;
   `frames.len()` unwind loops → walks to a target `cfr`/depth (the `close_*_frames(depth)`
   cleanups keep working off the depth counter); `finish_frame` reads
   `caller_cfr`/`saved_pc`/`return_reg` from the header instead of `pop()`+`last()`. Generator
   snapshot/restore copies the window + header fields into `SuspendedExecutionRef`.
4. **GC per-window walk** (§F): replace the wholesale register trace + `frames` iteration with
   the arena frame walk; trace cold-table + establishment-side-stack refs.
5. **Realm + referrer derivation** (§D): merge realm into the establishment side-stack; add
   `realm_of`; route current-frame readers through `running_context`, arbitrary readers through
   `realm_of`; delete the stored `realm` field.
6. **Soft-limit prologue** (§E): add the byte-budget soft-limit check; reconcile with
   `MAX_BYTECODE_CALL_DEPTH`.
7. **Delete the old path**: remove `frames: Vec<FrameRecord>` (`register_stack`/
   `register_stack_top` were already retired in step 2) and any residual bridge accessors;
   confirm the arena is the sole frame/register backing.

(Ordering is indicative; the implementation plan finalizes staging. The arena must remain
never-realloc and the suite green at every commit.)

## Testing & verification

- **Primary gate:** Test262 stays **100 % after every commit** (bisectable; run the full suite
  per commit).
- **Offset lock-in:** `frame_header_offsets_stable()` asserts every `FrameHeader` field offset
  and the header size, mirroring `ll_int_state_offsets_stable()`.
- **GC (heaviest focus — the interleave hazard):** a sole-referenced object held only in a
  register window survives GC; all header refs are traced; no header slot is ever mistraced as a
  Value; deep nested frames trace correctly; cold-table-held refs (`tail_caller`,
  `resume_value`) survive.
- **Realm:** `throw` in a non-default realm selects that realm's `Error.prototype` (running
  context); a function defined in realm B and called from realm A runs with B's realm
  (callee-derived); script/module/job root realm is correct post-derivation; a generator created
  in realm A and resumed from realm B reads A's realm.
- **Soft limit:** deep recursion throws `RangeError` (not a crash) with the slack intact; the
  throw itself runs inside the reserve.
- **Perf gate:** the V8 suite (Richards/RayTrace) must not regress — arena window indexing must
  match flat-Vec indexing. Measure before/after.
- **Debug asserts:** `running_context == derive(current frame)`; arena cursor/`frame_depth`
  consistency; the `caller_cfr` chain is well-formed (every link resolves to a lower `cfr`,
  bottoming at the root); the establishment side-stack is empty **iff** the arena is empty.

## Risks & out of scope

**Risks / mitigations:**
- **Register-access perf parity** — arena window indexing must equal `Vec` indexing; measure on
  the V8 gate and micro-bench the hot accessors.
- **GC interleave correctness** — the header-not-Value boundary is the central hazard; covered
  by the GC tests above and the per-window walk (never a wholesale slot trace).
- **Establishment-point set** — the realm/referrer side-stack must push/pop at exactly SP-0a's
  referrer establishment points (script/module/job/generator-restore); enumerate and unit-test
  the walk; assert the side-stack empties with the arena.
- **Header offset stability** — locked by `frame_header_offsets_stable()`; any reorder fails the
  test.
- **RSS** — eager ≈ 4 MB/Vm; acceptable for modest Vm counts; revisit via the deferred
  lazy-commit phase if many Vms coexist.
- **Generator window snapshot/restore** — the window now lives in the arena; ensure
  snapshot copies the exact `[regs_base..+len]` slice and restore reconstructs it faithfully.
- **`register_stack` absorption blast radius** — pervasive but mechanical; staged behind bridge
  accessors with the suite green per commit.

**Explicitly out of scope** (→ later phases): any asm call/return (SP-1/SP-2); overlapping arg
setup + compiler arg-VR layout (SP-0c); lazy-commit reservation (`mmap`/`VirtualAlloc` +
guard page) — a backing swap behind the arena interface, its own follow-up; the referrer
inherit-from-caller → callee-`[[ScriptOrModule]]` correctness fix (a behavior change, separate
item carried over from SP-0a); newTarget-into-`this`-slot aliasing. SP-0b is pure Rust,
behavior-preserving.
