# SP-0a — Eliminate the Separate ExecutionContext (Design)

**Date:** 2026-05-31
**Status:** Design — approved for planning
**Phase:** SP-0a of the inline-asm call-path program (Stage 2 of V8-v7 perf work)
**Prerequisite reading:** `docs/superpowers/specs/2026-05-31-asm-call-frame-architecture-research.md`
(JSC register-file synthesis). Project memory: `project_asm_call_frame_rearchitecture`.

## Why this exists

Stage 2's broad perf lever is an inline-asm call path; its enabler (SP-0) is making
call-frame setup doable in asm. Lyng today pushes **two** growable Rust structures per
call — `Vm` frame stack (`FrameRecord`) and `Agent.execution_contexts: Vec<ExecutionContext>`
— neither asm-addressable. JSC has no execution-context object at all: the `CallFrame`
*is* the activation record, and realm/referrer/scope are derived or held in frame-local
slots.

SP-0a is the first, **behavior-preserving** sub-phase: make the frame stack the single
source of truth and **delete `ExecutionContext` entirely**, with no asm and no change to
observable behavior (Test262 stays 100%). It de-risks SP-0b (the pre-reserved bump stack
with an asm-addressable pinned header) by removing the second stack first.

## Current state (as explored 2026-05-31)

The two stacks are already **near-1:1**. Every bytecode entry pushes a `FrameRecord` *and*
an `ExecutionContext` together and pops them together (`vm.rs` entry, `bytecode_calls.rs`
×2, generator/async suspend+restore). `FrameRecord` already carries `realm`, `variable_env`,
`lexical_env`, `new_target`, `kind`, `code`, `callee`, `this_value`, `construct_this`,
flags. The depth-based unwind loops already run a `frames.len()` loop right next to a
redundant `execution_contexts().len()` loop (`vm.rs:1746`/`1767`, `internal_calls.rs`).

Fields unique to `ExecutionContext` (everything else is duplicated on the frame):

| Field | Readers | Disposition under SP-0a |
| --- | --- | --- |
| `this_state` (`Lexical`/`Uninitialized`/`Value`) | `super_ops`, arrow `dynamic_compilation`, `generators` | **Store** on `FrameState` (mutable; `super()` updates it) |
| `private_env: Option<EnvironmentRef>` | eval (1), generator state (1), GC (4) | **Store** on `FrameMetadata` |
| `script_or_module_referrer: Option<AtomId>` | `import()` (live), `.then()` capture, generator serialization | **Derive by walk** via a `Vm` establishment side-stack; **never on the frame** |
| `executable: ExecutableId` | 1 test | **Derive** from `kind` + `code` |
| `realm`, `variable_env`, `lexical_env`, `new_target`, `kind` | many | Already on the frame; unchanged |

Two structurally hard spots (the only ones):

1. **Jobs are the single non-1:1 case.** `jobs.rs:132` pushes a job `ExecutionContext` with
   **no** frame; it builds a *local* `synthetic_job_caller_frame` (`jobs.rs:728` — empty
   register window, realm, global envs, `kind=Job`, `entry` flag) that never goes on the
   stack. Everything it holds is derivable from the job (realm → global env; referrer from
   payload).
2. **One whole-stack scan.** `set_execution_context_this_state_for_lexical_env`
   (`super_ops.rs:220`) updates `this_state` on *all* contexts sharing a derived
   constructor's lexical env, so arrow closures see `this` after `super()`. Everything
   else is `.last()` / `.len()`.

The `ops` crate (`errors.rs::current_realm`, `promise.rs::create_promise_reaction`) reads
the current context through **`Agent`**, which cannot see `Vm.frames` — handled by §C.

Referrer provenance: **inherited from the caller** at push time
(`bytecode_calls.rs:269,449` read `current_execution_context().script_or_module_referrer()`),
established fresh only at roots (script entry passes it in; module entry derives it from
`module_key_for_environment(lexical_env)`; jobs from payload). It is *not* statically tied
to `code`.

## Decisions (locked)

1. **Delete `ExecutionContext` entirely** — no permanent shim, **no surviving ambient
   state**. `ExecutionContextKind` and `ExecutableId` survive (the frame and the job system
   use them independently).
2. **Jobs:** push a synthetic job root frame onto `self.frames` for the job's duration.
3. **Referrer:** derive by stack walk via a `Vm` establishment side-stack; the referrer
   **never touches the frame** (keeps the future asm-addressable header minimal). Never
   inherited/copied per call.
4. **Ops realm/referrer reads use a single `Agent` running-context scalar** —
   `{ realm, referrer }`, refreshed by the `Vm` at the existing context-push/pop sites.
   This is JSC's model: `vm.topCallFrame` is one mutable pointer and the current
   realm/global is *derived from it*; JSC keeps exactly this minimal ambient state. It is
   **not** a per-call stack. (The 443 `throw_*_error(agent)` sites all funnel through a
   single `errors::current_realm` function, so this changes one function body, not the call
   sites; threading the realm down into the frameless `types`/`env` layers that legitimately
   have no running-realm concept would be a cascading layering violation, so it is
   rejected.)
5. **Strategy:** bridge-first incremental — every commit compiles and keeps Test262 at
   100%; the `ExecutionContext` type is deleted in the final commit, not deferred.
6. **Realm stays stored** on the frame (deriving it is SP-0b).

The end product chosen here favors the clean architecture over minimal churn: a single
ambient scalar (JSC's `topCallFrame` analog) rather than a parallel per-call stack, and no
cold referrer word on the asm-addressable header. Extra mechanical work now is accepted.

## Architecture

### A. The frame becomes the single source of truth

Add to `FrameRecord` the fields that today live only on `ExecutionContext`:

- **`FrameState.this_state: ThisState`** — the `Lexical`/`Uninitialized`/`Value` 3-way, in
  the *mutable* half because `super()` updates it. `this_value` stays as a separate field
  (the resolved `this` binding); the two stay consistent at exactly the points they do
  today, both on the frame now.
- **`FrameMetadata.private_env: Option<EnvironmentRef>`** — stable per activation, seeded
  at entry.
- **`realm`** — stays *stored* (already is). Deriving deferred to SP-0b.
- **`executable`** — *derived* from `kind` + `code` (`Bytecode(code)` for function frames,
  markers otherwise); one reader (a test).
- **`script_or_module_referrer`** — *derived*; see §D.

`ExecutionContextKind`/`ExecutableId` types are retained. Only the `ExecutionContext`
struct, the `push_*/pop_*_context` API, and `Agent.execution_contexts` are deleted.

### B. The bridge (temporary scaffolding)

`agent.current_execution_context()` cannot survive — `Agent` can't see `Vm.frames`.
Replace it with a `Vm` method `self.current_execution_context(agent)` that builds an
`ExecutionContext` on the fly from `self.frames.last()`. This keeps reader diffs tiny
during migration; in the final commit the remaining callers switch to direct frame
accessors and the method + type are deleted together.

### C. A single `Agent` running-context scalar (the JSC `topCallFrame` analog)

Two `ops`-crate hubs hold only `Agent` and read the current context today:
`errors::current_realm` (the funnel reached by all 443 `throw_*_error` sites) and
`promise::create_promise_reaction` (referrer at `.then()` time). `Agent` cannot see
`Vm.frames`, so the "current realm/referrer" has to arrive some other way.

**The chosen mechanism is a single ambient scalar on `Agent`:**
`running_context: Option<RunningContext>` where `RunningContext { realm: RealmRef,
referrer: Option<AtomId> }`. The `Vm` refreshes it at the **existing context-push/pop
sites** — exactly the frame-transition points this refactor already edits: each
`push_*_context` becomes "refresh running-context from the new active frame", each
`pop_*_context` becomes "restore running-context from the new `frames.last()`" (realm =
`frames.last().realm()`; referrer = the §D walk). `errors::current_realm` and the ops
referrer read consume this scalar; the 443 throw sites are untouched.

This is **not** the per-call stack SP-0a removes — it is a single scalar, and it is
precisely JSC's model: `vm.topCallFrame` is one mutable pointer from which the current
realm/global is derived. JSC keeps exactly this minimal ambient state; it does **not**
thread the realm into every error site. (Threading was considered and rejected: the throw
sites funnel through one function, and pushing the realm down into the frameless
`types`/`env` layers would be a cascading layering violation.) Drift is mitigated by the
single refresh chokepoint plus a debug assert that the scalar equals
`derive(frames.last())`.

### D. Referrer derivation

Referrer is inherited from the caller today and established fresh only at roots. **The
referrer never lives on the frame** — it is cold and derived, and the frame header is what
SP-0b will pin for asm addressing. The seed instead lives in a small **`Vm`
`referrer_scopes` side-stack**: an entry `(base_frame_depth, referrer)` is pushed at each of
the four establishment points and unwound with its frames; `referrer(depth)` returns the
top scope whose `base_frame_depth <= depth`.

- **Establishment points** push a scope: script root (host-provided value), module root
  (computed from `lexical_env → module_key`), job root (payload value), restored generator
  frame (value captured at suspend).
- **Plain call frames** push nothing; the walk to the nearest covering scope reproduces
  today's inherited value.
- **Generators** capture the derived referrer at *suspend* into side-state and **re-push**
  the establishment scope on *resume*. This is the key subtlety: a pure re-walk at resume
  would read the resume-time stack (potentially a different realm) and be wrong —
  capture-and-restore preserves the suspend-time value. The live reader (`import()`,
  `dynamic_import.rs:293`) and the `.then()` capture (`promise.rs:38`) both read through the
  walk.

The side-stack is shallow (typically depth 1–2), cold, never on the asm path, and
categorically different from the per-call `ExecutionContext` stack being removed — it holds
only establishment scopes, not one entry per call. It is **not** a denormalized cache: each
entry has an explicit push/pop tied to an establishment frame's lifetime, hooked into the
existing frame-unwind loops. A `None`-mostly `Option<AtomId>` slot on `FrameMetadata` was
**rejected**: it would put a cold word on the `Copy` hot path and the future asm header for
no benefit.

**Separate follow-up (NOT SP-0a):** Lyng inherits referrer from the *caller*
(`bytecode_calls.rs:269`), whereas the spec sets a function activation's `[[ScriptOrModule]]`
from the *callee's* defining script/module (PrepareForOrdinaryCall). These coincide within a
single unit but diverge for a function defined in module A and called from module B that
runs a relative `import()`. SP-0a **reproduces today's inherit-from-caller behavior exactly**
(the side-stack walk does this); the spec-correctness question is tracked as its own item,
since fixing it is a behavior change, not a refactor.

### E. Job root frame & the whole-stack scan

- **Jobs:** push `synthetic_job_caller_frame` onto `self.frames` for the job's duration
  (it already has realm + global envs + `kind=Job`; add the payload referrer seed), and
  drop the `push_execution_context(job)` call. The job frame sits at depth 0 (jobs run with
  an empty stack) and is a passive holder — `self.run` is never invoked on it (the job
  helpers push their own bytecode frames or call natives), so dispatch never executes it
  as bytecode.
- **Whole-stack scan:** `set_execution_context_this_state_for_lexical_env` moves from
  `Agent` to `Vm` and iterates `self.frames` filtered by `lexical_env`, setting each
  match's `this_state`. `set_current_execution_context_this_state` becomes "set the current
  frame's `this_state`."

## Commit sequence (each commit compiles + Test262 100%)

1. **Frame carries everything + side-stack + running-context scalar stand up** *(pure
   additions, no readers changed)*: add `this_state`/`private_env` + `with_*` builders;
   populate at all push sites (entry, `bytecode_calls` ×2, generator restore) *alongside* the
   existing context push; also set the frame's `this_state` at the `super()` mutation; add
   the `Vm` `referrer_scopes` side-stack + `referrer()` walk, pushed/popped at the four
   establishment points; add the `Agent` `running_context` scalar + `Vm` refresh at the
   existing context-push/pop sites. The context stack is now redundant but still
   authoritative for readers.
2. **Migrate reader clusters** *(one commit each)*: `dynamic_import` (referrer/realm via
   walk + active frame) → `errors`/`promise` ops (point `current_realm` + the referrer read
   at the new `Agent` running-context scalar) → `super_ops` this_state →
   `dynamic_compilation` (arrow/eval) → `generators` serialization (frame fields + captured
   referrer) → residual `.is_some()` guards. After each, that cluster no longer touches the
   context stack.
3. **Job root frame**: push `synthetic_job_caller_frame` onto `self.frames`; drop the job
   context push. Verify frame-depth invariants.
4. **Whole-stack scan**: relocate `set_*_for_lexical_env` to `Vm`, scanning `self.frames`.
5. **Delete the old path**: remove `push_*/pop_*_context`, `Agent.execution_contexts`, the
   redundant `execution_contexts().len()` unwind loops (`vm.rs:1767`,
   `internal_calls.rs:40`), switch the last bridge callers to direct accessors, delete the
   `ExecutionContext` struct.

## Testing & verification

- **Primary gate:** Test262 stays **100% after every commit** (bisectable). Run the full
  suite per commit.
- **Unit tests:** the `env/src/tests.rs` assertions on `execution_contexts` (≈ lines 90,
  1056–1066) get rewritten to assert frame state, or deleted with the API.
- **Targeted regressions to add/confirm** — one per risky spot:
  - Multi-realm: `throw` inside a non-default realm selects *that* realm's
    `Error.prototype` (exercises the §C running-context scalar).
  - `super()` initializes `this` for arrow closures defined in the constructor (exercises
    the frame-stack scan, §E).
  - Generator created in module A, resumed from script B, runs `import()` → resolves
    against **A's** referrer (exercises capture-and-restore, §D).
  - `import()` inside a promise-reaction job → referrer from the job root frame (exercises
    §E + §D).
- **Perf gate:** the V8 suite (Richards/RayTrace) must not regress. `FrameRecord` is `Copy`
  and grows by `this_state` (small enum) + `private_env` (referrer stays *off* the frame);
  measure, and pack into existing flag bits if the hot copy path moves.

## Risks & out of scope

**Risks / mitigations:**
- Frame-size growth on the `Copy` path → measure on the gate; pack if needed.
- Job-frame depth invariants (`internal_completion_targets`, `note_frame_depth`,
  `MAX_BYTECODE_CALL_DEPTH`, register base 0) → covered by Test262 promise/async + the
  targeted job test.
- `referrer_scopes` side-stack unwind correctness → every establishment scope must be
  popped exactly when its base frame unwinds (including the abrupt/exception and
  suspend/resume paths); hook the push/pop into the existing frame-unwind loops and assert
  in debug that `referrer_scopes` is empty whenever the frame stack is empty.
- Establishment-point set must exactly match today's `with_script_or_module_referrer` sites
  (script entry, module entry, job, generator restore) → enumerate and unit-test the walk.
- Running-context scalar drift → refresh at the single context-push/pop chokepoint and
  assert in debug that `running_context == derive(frames.last())` (realm + referrer) after
  every transition; the multi-realm test guards the realm value.

**Explicitly out of scope** (→ later SP phases): any asm; the pre-reserved bump frame stack
and `repr(C)`/`offset_of!` pinned header (SP-0b); realm derivation (SP-0b); compiler
arg-VR layout (SP-0c). SP-0a is pure Rust, behavior-preserving.
