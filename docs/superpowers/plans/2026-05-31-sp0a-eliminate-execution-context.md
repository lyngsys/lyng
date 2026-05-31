# SP-0a — Eliminate the Separate ExecutionContext Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the `Vm` frame stack the single source of truth and delete `ExecutionContext` entirely, with zero observable behavior change (Test262 stays 100%).

**Architecture:** `FrameRecord` absorbs the two fields it lacks (`this_state`, `private_env`); `script_or_module_referrer` is derived via a small `Vm` `referrer_scopes` establishment side-stack (never on the frame); the `ops` crate reads a single `Agent` `running_context` scalar (the JSC `topCallFrame` analog) refreshed by the `Vm` at frame transitions; the one non-1:1 case (promise/microtask jobs) gets a real synthetic root frame on the stack. Migration is bridge-free and incremental — each task is a commit that compiles and keeps Test262 at 100%, and `ExecutionContext` is deleted only in the final task.

**Tech Stack:** Rust (workspace crates `lyng_env`, `lyng_vm`, `lyng_ops`); test runner `cargo test` + the Test262 harness (`crates/test262-harness`); benches via the V8 suite.

**Spec:** `docs/superpowers/specs/2026-05-31-sp0a-eliminate-execution-context-design.md`. Read it first.

**Conventions for every commit message in this plan:** end with
```
Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
```
Branch is `feat/sp0a-eliminate-execution-context` (already created). Do not merge to `main` until the whole plan is done and the perf gate passes.

---

## File Structure

**Modified — `lyng_env`:**
- `crates/env/src/execution.rs` — add `RunningContext` struct (and its `TraceHeapEdges`); `ExecutionContext` itself is deleted in the final task.
- `crates/env/src/agent.rs` — add `running_context: Option<RunningContext>` field + accessor/setter; drop `execution_contexts` field and its `AgentCollectionSnapshot` clone+trace in the final task.
- `crates/env/src/agent/execution_contexts.rs` — deleted in the final task.
- `crates/env/src/tests.rs` — rewrite/remove the assertions that read `execution_contexts`.

**Modified — `lyng_vm`:**
- `crates/vm/src/frame.rs` — add `this_state`/`private_env` fields, builders, accessors, `set_this_state`.
- `crates/vm/src/vm.rs` — add `referrer_scopes` field + the side-stack/walk helpers + `refresh_running_context`; update the frame GC trace to cover `private_env`; entry push site; final-task unwind-loop cleanup.
- `crates/vm/src/vm/bytecode_calls.rs`, `vm/generators.rs`, `vm/async_functions.rs`, `vm/jobs.rs`, `vm/internal_calls.rs` — push-site population, job root frame, reader migration, cleanup-loop edits.
- `crates/vm/src/vm/builtin_dispatch/class_helpers/super_ops.rs` + `class_helpers.rs` + `class_helpers/private_fields.rs` — this_state read/write migration; relocate the whole-stack scan.
- `crates/vm/src/vm/dynamic_compilation.rs`, `vm/builtin_dispatch/dynamic_import.rs`, `vm/semantics/names.rs` — reader migration.

**Modified — `lyng_ops`:**
- `crates/ops/src/errors.rs` — point `current_realm` at the `running_context` scalar.
- `crates/ops/src/promise.rs` — point the referrer read at the `running_context` scalar.

---

## Task 1: Add `this_state` to the frame

**Files:**
- Modify: `crates/vm/src/frame.rs` (`FrameState` struct ~199-212; `FrameRecord::new` ~287-323; builders ~325-413; accessors ~214-269; setters ~474-490)
- Test: `crates/vm/src/frame.rs` (existing `#[cfg(test)]` module near line 620)

- [ ] **Step 1: Write the failing test**

Add to the test module in `crates/vm/src/frame.rs`:

```rust
    #[test]
    fn frame_round_trips_this_state() {
        use lyng_env::ThisState;
        let frame = sample_frame() // reuse the test helper used by existing frame tests
            .with_this_state(ThisState::Lexical);
        assert_eq!(frame.this_state(), ThisState::Lexical);

        let mut frame = frame;
        frame.set_this_state(ThisState::Uninitialized);
        assert_eq!(frame.this_state(), ThisState::Uninitialized);
    }
```

If there is no `sample_frame()` helper, build the frame inline with `FrameRecord::new(...)` mirroring the existing frame test at the bottom of the file (the one that calls `set_this_value(Value::from_smi(7))`).

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p lyng_vm --lib frame::tests::frame_round_trips_this_state`
Expected: FAIL — `no method named with_this_state`.

- [ ] **Step 3: Add the field, builder, accessor, setter**

In the `use` line at the top of `frame.rs` add `ThisState`:
```rust
use lyng_env::{ExecutionContextKind, ThisState};
```

Add the field to `FrameState` (the mutable half — `super()` updates it), after `this_value`:
```rust
    this_value: Value,
    this_state: ThisState,
```

In `FrameRecord::new`, initialize it in the `FrameState { … }` literal (default matches `ExecutionContext::new`, which defaults to `Uninitialized`):
```rust
                this_value: Value::undefined(),
                this_state: ThisState::Uninitialized,
```

Add a builder next to `with_this_value`:
```rust
    #[inline]
    pub const fn with_this_state(mut self, this_state: ThisState) -> Self {
        self.state.this_state = this_state;
        self
    }
```

Add an accessor next to `this_value` (in the `FrameState` impl AND mirror on `FrameRecord` if `FrameRecord` exposes per-field accessors; follow how `this_value` is exposed — `FrameRecord::this_value` delegates to `self.state.this_value`):
```rust
    #[inline]
    pub const fn this_state(&self) -> ThisState {
        self.state.this_state
    }
```
(If `this_value()` lives only on `FrameState`, add `this_state()` there too and add the `FrameRecord` delegator matching `this_value`'s delegator.)

Add a setter next to `set_this_value` (~474):
```rust
    #[inline]
    pub(crate) const fn set_this_state(&mut self, this_state: ThisState) {
        self.state.this_state = this_state;
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p lyng_vm --lib frame::tests::frame_round_trips_this_state`
Expected: PASS.

- [ ] **Step 5: Build the whole workspace to confirm no signature breakage**

Run: `cargo build -p lyng_vm`
Expected: success (the new field has a default in `new`, so existing call sites are unaffected).

- [ ] **Step 6: Commit**

```bash
git add crates/vm/src/frame.rs
git commit -m "feat(vm): add this_state to the call frame

Pure addition; defaults to Uninitialized (matching ExecutionContext::new).
No readers yet.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: Add `private_env` to the frame

**Files:**
- Modify: `crates/vm/src/frame.rs` (`FrameMetadata` struct ~130-140; `FrameRecord::new` ~287-323; builders; accessors ~142-187)
- Test: `crates/vm/src/frame.rs` test module

- [ ] **Step 1: Write the failing test**

```rust
    #[test]
    fn frame_round_trips_private_env() {
        use lyng_types::EnvironmentRef;
        let env = EnvironmentRef::from_raw(3).expect("non-zero env ref");
        let frame = sample_frame().with_private_env(Some(env));
        assert_eq!(frame.private_env(), Some(env));
        assert_eq!(sample_frame().private_env(), None);
    }
```
(Use the same `EnvironmentRef` constructor the existing tests use; if `from_raw` is not the right constructor, copy the pattern from another frame test that builds an `EnvironmentRef`.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p lyng_vm --lib frame::tests::frame_round_trips_private_env`
Expected: FAIL — `no method named with_private_env`.

- [ ] **Step 3: Add field, builder, accessor**

Add to `FrameMetadata`, after `variable_env`:
```rust
    variable_env: EnvironmentRef,
    private_env: Option<EnvironmentRef>,
```

In `FrameRecord::new`, in the `FrameMetadata { … }` literal:
```rust
                variable_env,
                private_env: None,
```

Builder on `FrameRecord` (next to `with_callee`):
```rust
    #[inline]
    pub const fn with_private_env(mut self, private_env: Option<EnvironmentRef>) -> Self {
        self.metadata.private_env = private_env;
        self
    }
```

Accessor (on `FrameMetadata`, mirrored on `FrameRecord` like `variable_env`):
```rust
    #[inline]
    pub const fn private_env(&self) -> Option<EnvironmentRef> {
        self.metadata.private_env
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p lyng_vm --lib frame::tests::frame_round_trips_private_env`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/frame.rs
git commit -m "feat(vm): add private_env to the call frame

Pure addition; defaults to None. No readers yet.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Populate `this_state` + `private_env` at every frame push site

This makes the frame carry the same values the parallel `ExecutionContext` already does, at the same points. The context push/pop stays in place (still authoritative); we only ADD frame population.

**Files:**
- Modify: `crates/vm/src/vm.rs` (entry, ~1690-1722)
- Modify: `crates/vm/src/vm/bytecode_calls.rs` (~272-300 and ~452-489)
- Modify: `crates/vm/src/vm/generators.rs` (restore, ~1555-1590)
- Modify: `crates/vm/src/vm/builtin_dispatch/class_helpers/super_ops.rs` (~220-256, the `super()` mutation)
- Test: `crates/vm/src/vm/bytecode_calls.rs` test module (or a vm integration test)

- [ ] **Step 1: Write the failing test**

Add an integration-style test (place it where other `bytecode_calls` behavior tests live; if none, add to `crates/vm/src/vm/tests.rs` or the nearest existing vm test module) asserting that after entering a bytecode call, the current frame's `this_state`/`private_env` equal the current execution context's:

```rust
    #[test]
    fn frame_mirrors_context_this_state_and_private_env() {
        // Run a tiny script that enters a function call, pause at a point where a
        // frame is live (e.g. via the existing test harness that single-steps or
        // inspects vm.frame()), then assert equality with the context top.
        // Use the same harness the other vm call tests use to drive a call.
        let (vm, agent) = run_to_first_call("function f(){ return 1 } f()");
        let frame = vm.frame().expect("a live frame");
        let ctx = agent.current_execution_context().expect("a live context");
        assert_eq!(frame.this_state(), ctx.this_state());
        assert_eq!(frame.private_env(), ctx.private_env());
    }
```
Adapt `run_to_first_call` to whatever the existing vm test harness exposes (look for how other tests obtain a `Vm` + `Agent` mid-execution; if the harness can only run to completion, instead assert inside a native breakpoint or reuse a generator-suspend test that exposes a live frame).

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p lyng_vm --lib frame_mirrors_context_this_state_and_private_env`
Expected: FAIL — frame has default `Uninitialized`/`None`, context has the real values.

- [ ] **Step 3: Populate at the entry site (`vm.rs` ~1712-1722)**

The entry currently builds `context` then `frame = FrameRecord::new(...)…`. Add the two `with_*` to the frame builder, mirroring the `context` construction immediately above it:

```rust
        let frame = FrameRecord::new(
            code,
            0,
            RegisterWindow::new(register_base, register_len),
            None,
            realm,
            lexical_env,
            variable_env,
            context.kind(),
        )
        .with_this_value(this_value)
        .with_this_state(if entry_lexical_this {
            ThisState::Lexical
        } else {
            ThisState::Value(this_value)
        })
        .with_private_env(entry_private_env)
        .with_new_target(new_target)
        .with_flags(FrameFlags::entry().with_flag(FrameFlags::suspendable(), true));
```
(The module branch at ~1707 sets `ThisState::Value(this_value)` and the same `entry_private_env`; mirror those onto its frame too if the module path builds a distinct frame — match whatever `context` it uses.)

- [ ] **Step 4: Populate at the two `bytecode_calls.rs` sites**

At ~278 and ~458 the `context` is built with `.with_private_env(prepared.private_env).with_this_state(prepared.execution_this_state)`. The `frame = FrameRecord::new(...)` right below each must gain the same two:
```rust
        let frame = FrameRecord::new(/* …existing args… */)
            // …existing .with_* …
            .with_this_state(prepared.execution_this_state)
            .with_private_env(prepared.private_env);
```

- [ ] **Step 5: Populate at the generator restore site (`generators.rs` ~1555-1590)**

The restore rebuilds both a context (with `.with_private_env(...).with_this_state(...)`) and a `FrameRecord`. Add the matching `.with_this_state(...)` and `.with_private_env(...)` to the rebuilt `FrameRecord`, reading from the same `SuspendedExecutionSideState` fields the context uses.

- [ ] **Step 6: Mirror the `super()` this_state write onto the frame (`super_ops.rs` ~220-256)**

Today after `super()` the code calls `agent.set_execution_context_this_state_for_lexical_env(...)` / `agent.set_current_execution_context_this_state(...)` (lines 220-228) and separately locates `frame_index` and calls `frame.set_this_value`/`set_construct_this` (lines 230-256). Add a `frame.set_this_state(ThisState::Value(this_value));` next to the existing `frame.set_this_value(this_value);` at ~256 so the located frame's this_state tracks the context. Also add the same scan on frames for the lexical-env case — but DO NOT remove the agent calls yet (still authoritative); this is additive:
```rust
        frame.set_this_value(this_value);
        frame.set_construct_this(Some(this_object));
        frame.set_this_state(ThisState::Value(this_value));
```

- [ ] **Step 7: Run the test + the vm suite**

Run: `cargo test -p lyng_vm --lib frame_mirrors_context_this_state_and_private_env`
Expected: PASS.
Run: `cargo test -p lyng_vm`
Expected: PASS (no regressions).

- [ ] **Step 8: Commit**

```bash
git add crates/vm/src/vm.rs crates/vm/src/vm/bytecode_calls.rs crates/vm/src/vm/generators.rs crates/vm/src/vm/builtin_dispatch/class_helpers/super_ops.rs
git commit -m "feat(vm): populate frame this_state/private_env at all push sites

Mirrors the parallel ExecutionContext values onto the frame at entry, both
bytecode-call sites, generator restore, and the super() this write. Context
stack unchanged and still authoritative; readers migrate in later tasks.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: Trace `private_env` in the frame GC root scan

`private_env` is a heap edge. The `Vm` already traces frame fields as GC roots; add `private_env` so a private env reachable only through the frame is kept alive.

**Files:**
- Modify: `crates/vm/src/vm.rs` (the frame GC trace; per the census the frame field tracing is around lines 1147/1179/1196/1203 — search for `record.lexical_env()` / `record.new_target()` in the trace/root function)
- Test: `crates/vm/src/vm.rs` test module (GC test alongside existing frame-rooting tests)

- [ ] **Step 1: Write the failing test**

Model it on the existing minor-GC frame-rooting tests (search for a test that allocates an env, stores it only on a live frame, runs a minor GC, asserts survival). Add:
```rust
    #[test]
    fn minor_gc_frame_private_env_survives() {
        // Build a live frame whose ONLY reference to a freshly-allocated environment
        // is via private_env; run a minor GC; assert the environment still resolves.
        // Mirror the existing `minor_gc_cell_backed_global_value_survives`-style test
        // structure used in this module.
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p lyng_vm --lib minor_gc_frame_private_env_survives`
Expected: FAIL — the private env is collected.

- [ ] **Step 3: Add the trace edge**

In the frame-rooting/trace function, next to the line that traces `record.new_target()` (or `record.private_env()` if the GC already reads a private env from elsewhere), add tracing of the frame's `private_env`:
```rust
        if let Some(private_env) = record.private_env() {
            private_env.trace_heap_edges(tracer); // match the surrounding trace style/API
        }
```
Match the exact tracer API used by the adjacent `lexical_env`/`new_target` trace calls in that function.

- [ ] **Step 4: Run test + GC suite**

Run: `cargo test -p lyng_vm --lib minor_gc_frame_private_env_survives`
Expected: PASS.
Run: `cargo test -p lyng_vm gc`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm.rs
git commit -m "feat(vm): trace frame private_env as a GC root

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: Add the `referrer_scopes` establishment side-stack + walk

Referrer never goes on the frame. The seed lives in a small `Vm` side-stack pushed at the four establishment points; the "walk" reduces to reading the top scope.

**Files:**
- Modify: `crates/vm/src/vm.rs` (`Vm` struct ~123-127; `new()` ~582; add helpers)
- Test: `crates/vm/src/vm.rs` test module

- [ ] **Step 1: Write the failing test**

```rust
    #[test]
    fn referrer_scopes_walk_returns_nearest_establishment() {
        use lyng_common::AtomId;
        let mut vm = Vm::new();
        assert_eq!(vm.current_referrer(), None);
        let a = AtomId::from_raw(10).unwrap();
        vm.push_referrer_scope(0, Some(a));
        assert_eq!(vm.current_referrer(), Some(a));
        // a plain call adds no scope; the nearest establishment still wins:
        assert_eq!(vm.current_referrer(), Some(a));
        let b = AtomId::from_raw(20).unwrap();
        vm.push_referrer_scope(2, Some(b));
        assert_eq!(vm.current_referrer(), Some(b));
        vm.unwind_referrer_scopes_to(1); // drops the depth-2 scope
        assert_eq!(vm.current_referrer(), Some(a));
    }
```
Use whatever `AtomId` constructor the codebase exposes (`from_raw`/`new`); if `Vm::new()` is not the right constructor for a unit test, copy the construction other vm unit tests use.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p lyng_vm --lib referrer_scopes_walk_returns_nearest_establishment`
Expected: FAIL — `no method named push_referrer_scope`.

- [ ] **Step 3: Add the struct, field, and helpers**

Near the top of `vm.rs` (with the other small `Vm` support types), add:
```rust
/// One referrer-establishment scope on the `Vm` side-stack. `base_depth` is the
/// frame depth at which the establishing frame sits; the scope covers all frames
/// at depth >= `base_depth` until that frame unwinds. `referrer` is the seed
/// (None is a valid establishment — e.g. a script with no host referrer).
#[derive(Clone, Copy, Debug)]
struct ReferrerScope {
    base_depth: usize,
    referrer: Option<lyng_common::AtomId>,
}
```

Add the field to `struct Vm` next to `frames`:
```rust
    frames: Vec<FrameRecord>,
    referrer_scopes: Vec<ReferrerScope>,
```

Initialize in `Vm::new()` next to `frames: Vec::new(),`:
```rust
            referrer_scopes: Vec::new(),
```

Add the helpers in an appropriate `impl Vm` block:
```rust
    pub(crate) fn push_referrer_scope(&mut self, base_depth: usize, referrer: Option<lyng_common::AtomId>) {
        self.referrer_scopes.push(ReferrerScope { base_depth, referrer });
    }

    /// Drop every establishment scope whose base frame has unwound (i.e. whose
    /// `base_depth >= target_frame_depth`).
    pub(crate) fn unwind_referrer_scopes_to(&mut self, target_frame_depth: usize) {
        while self
            .referrer_scopes
            .last()
            .is_some_and(|scope| scope.base_depth >= target_frame_depth)
        {
            self.referrer_scopes.pop();
        }
    }

    /// The referrer of the current establishment (the nearest one toward the base).
    pub(crate) fn current_referrer(&self) -> Option<lyng_common::AtomId> {
        self.referrer_scopes.last().and_then(|scope| scope.referrer)
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p lyng_vm --lib referrer_scopes_walk_returns_nearest_establishment`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm.rs
git commit -m "feat(vm): add referrer_scopes establishment side-stack + walk

Off-frame referrer source for the future asm-addressable header. Not yet wired
to any push site or reader.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 6: Wire establishment scopes at the four establishment points + unwind

Push a referrer scope wherever the current code establishes a fresh (non-inherited) referrer, and unwind scopes in the frame-unwind loops. After this task, `vm.current_referrer()` equals today's inherited `current_execution_context().script_or_module_referrer()` at every point.

**Files:**
- Modify: `crates/vm/src/vm.rs` (entry ~1700-1731; unwind loop ~1746-1760)
- Modify: `crates/vm/src/vm/internal_calls.rs` (unwind loop ~30-42)
- Modify: `crates/vm/src/vm/jobs.rs` (~132)
- Modify: `crates/vm/src/vm/generators.rs` (suspend ~773-816 and restore ~1555-1590)
- Test: a vm integration test driving `import()` indirectly, OR a focused unit test that pushes frames + scopes and checks `current_referrer()` matches `current_execution_context().script_or_module_referrer()`

- [ ] **Step 1: Write the failing test**

```rust
    #[test]
    fn current_referrer_matches_context_after_entry() {
        // Drive a script whose top-level referrer is a known atom (use the existing
        // entry API that passes script_or_module_referrer), enter a nested call, and
        // assert vm.current_referrer() == agent.current_execution_context()
        //   .and_then(ExecutionContext::script_or_module_referrer) at a live frame.
        // Reuse the harness from Task 3.
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p lyng_vm --lib current_referrer_matches_context_after_entry`
Expected: FAIL — `current_referrer()` is `None` (no scopes pushed yet).

- [ ] **Step 3: Push a scope at script/module entry (`vm.rs` ~1726-1731)**

Right where the entry currently does `agent.push_execution_context(context);` and `self.frames.push(frame);`, add (use the same `script_or_module_referrer`/`module_referrer` value the context was built with, and `prior_frame_depth` already computed at ~1726):
```rust
        self.push_referrer_scope(prior_frame_depth, script_or_module_referrer);
        agent.push_execution_context(context);
        self.note_executed_code(frame.code());
        self.frames.push(frame);
```
For the module branch, push the `module_referrer` instead.

- [ ] **Step 4: Unwind scopes in the entry unwind loop (`vm.rs` ~1746-1760)**

After the `while self.frames.len() > prior_frame_depth { … }` loop body (or immediately after the loop), add:
```rust
        self.unwind_referrer_scopes_to(prior_frame_depth);
```

- [ ] **Step 5: Unwind scopes in the `internal_calls.rs` unwind loop (~30-42)**

This loop pops frames back to `prior_frame_depth` (or its local baseline). Add the matching `self.unwind_referrer_scopes_to(<baseline>);` after it, using the same baseline the frame loop uses.

- [ ] **Step 6: Push a scope for jobs (`jobs.rs` ~132)**

Jobs establish a referrer from the payload. Right before `agent.push_execution_context(ExecutionContext::job(...)…)`, capture the depth and push the scope (the job root frame is added in Task 11; for now the scope coexists with the context):
```rust
        let job_base_depth = self.frames.len();
        self.push_referrer_scope(job_base_depth, script_or_module_referrer);
        agent.push_execution_context(
            lyng_env::ExecutionContext::job(realm, job.executable(), lexical_env, variable_env)
                .with_script_or_module_referrer(script_or_module_referrer),
        );
```
And before the `let _ = agent.pop_execution_context();` at ~193 add:
```rust
        self.unwind_referrer_scopes_to(job_base_depth);
```

- [ ] **Step 7: Generators — capture on suspend, re-push on restore**

At suspend (`generators.rs` ~773-816, where the context is popped) call `self.unwind_referrer_scopes_to(<frame baseline being suspended>)` alongside the frame pop, and ensure the suspended side-state already carries `script_or_module_referrer` (it does — `generators.rs:1701`). At restore (`generators.rs` ~1555-1590, where the context is rebuilt and the frame pushed) add `self.push_referrer_scope(<restore depth>, saved_referrer);` using the saved value, so resume re-establishes the suspend-time referrer rather than re-walking the resume-time stack.

- [ ] **Step 8: Run test + vm suite + the dynamic-import tests**

Run: `cargo test -p lyng_vm --lib current_referrer_matches_context_after_entry`
Expected: PASS.
Run: `cargo test -p lyng_vm`
Expected: PASS.

- [ ] **Step 9: Commit**

```bash
git add crates/vm/src/vm.rs crates/vm/src/vm/internal_calls.rs crates/vm/src/vm/jobs.rs crates/vm/src/vm/generators.rs
git commit -m "feat(vm): establish referrer scopes at script/module/job/generator roots

current_referrer() now reproduces the inherited context referrer at every point.
Context stack still authoritative; readers migrate next.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 7: Add the `Agent` `running_context` scalar + `Vm` refresh

The single ambient `{ realm, referrer }` the `ops` crate reads. Refreshed at the existing context push/pop sites.

**Files:**
- Modify: `crates/env/src/execution.rs` (add `RunningContext` + `TraceHeapEdges`)
- Modify: `crates/env/src/agent.rs` (field ~171, init ~214, accessor/setter)
- Modify: `crates/vm/src/vm.rs` (add `refresh_running_context`; call it at entry push + unwind)
- Modify: `crates/vm/src/vm/bytecode_calls.rs`, `vm/generators.rs`, `vm/internal_calls.rs`, `vm/jobs.rs` (call refresh at their context push/pop sites)
- Test: `crates/env/src/tests.rs` (scalar round-trip) + a vm test (refresh tracks the frame)

- [ ] **Step 1: Write the failing test (env)**

In `crates/env/src/tests.rs`:
```rust
    #[test]
    fn running_context_round_trips() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        assert!(agent.running_context().is_none());
        let realm = agent.default_realm_id().expect("default realm");
        agent.set_running_context(Some(lyng_env::RunningContext::new(realm, None)));
        assert_eq!(agent.running_context().map(|rc| rc.realm()), Some(realm));
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p lyng_env --lib running_context_round_trips`
Expected: FAIL — `RunningContext`/`running_context` undefined.

- [ ] **Step 3: Add `RunningContext` (`execution.rs`)**

```rust
/// The single ambient running-context snapshot (realm + script/module referrer),
/// the analog of JSC's `vm.topCallFrame`-derived realm. Refreshed by the VM from
/// the active frame; the only ambient execution state after ExecutionContext is
/// removed. Not a stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunningContext {
    realm: RealmRef,
    referrer: Option<AtomId>,
}

impl RunningContext {
    #[inline]
    pub const fn new(realm: RealmRef, referrer: Option<AtomId>) -> Self {
        Self { realm, referrer }
    }

    #[inline]
    pub const fn realm(self) -> RealmRef {
        self.realm
    }

    #[inline]
    pub const fn referrer(self) -> Option<AtomId> {
        self.referrer
    }
}

impl TraceHeapEdges for RunningContext {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.realm.trace_heap_edges(tracer);
        // referrer is an interned AtomId; matches ExecutionContext, which did not
        // trace script_or_module_referrer either.
    }
}
```
Export it from the crate (add `RunningContext` to the `pub use execution::{…}` line in `crates/env/src/lib.rs` or wherever `ExecutionContext` is re-exported).

- [ ] **Step 4: Add the field + accessor/setter (`agent.rs`)**

Field next to `execution_contexts` (~171) — keep both for now:
```rust
    execution_contexts: Vec<ExecutionContext>,
    running_context: Option<RunningContext>,
```
Init in `new()` (~214):
```rust
            execution_contexts: Vec::new(),
            running_context: None,
```
Accessor/setter (in an `impl Agent` block; import `RunningContext`):
```rust
    #[inline]
    pub const fn running_context(&self) -> Option<RunningContext> {
        self.running_context
    }

    #[inline]
    pub fn set_running_context(&mut self, running_context: Option<RunningContext>) {
        self.running_context = running_context;
    }
```
Add `self.running_context` to the `Agent` GC trace (find the `impl TraceHeapEdges for Agent`-equivalent — note `AgentCollectionSnapshot` at `agent.rs:87`; add the snapshot field + trace mirroring `execution_contexts`, OR if `Agent` traces directly, add there). Trace via:
```rust
        if let Some(running_context) = &self.running_context {
            running_context.trace_heap_edges(tracer);
        }
```

- [ ] **Step 5: Run the env test**

Run: `cargo test -p lyng_env --lib running_context_round_trips`
Expected: PASS.

- [ ] **Step 6: Add `Vm::refresh_running_context` + a debug assert**

In `vm.rs`:
```rust
    /// Refresh the Agent's ambient running-context from the active frame. Called
    /// at every frame transition (the former context push/pop sites).
    pub(crate) fn refresh_running_context(&self, agent: &mut Agent) {
        let running = self
            .frame()
            .map(|frame| lyng_env::RunningContext::new(frame.realm(), self.current_referrer()));
        agent.set_running_context(running);
    }
```

- [ ] **Step 7: Call refresh at every context push/pop site**

Immediately after each `agent.push_execution_context(…)` and after each `self.frames.push(frame)` / unwind loop and `pop_execution_context`, add `self.refresh_running_context(agent);`. Sites:
- `vm.rs` entry: after `self.frames.push(frame)` (~1731) and after the unwind loop + `unwind_referrer_scopes_to` (~1760).
- `bytecode_calls.rs`: after the context push at ~300 and ~489; after `pop_execution_context` at ~349.
- `internal_calls.rs`: after its unwind loop (~42).
- `generators.rs`: after restore push (~1590) and after suspend pop (~791/815).
- `jobs.rs`: after the job context push (~135) and after the pop (~193).
- `async_functions.rs`: after `pop_execution_context` (~359).

- [ ] **Step 8: Write the vm refresh test**

```rust
    #[test]
    fn running_context_tracks_active_frame_realm() {
        // Drive a call; at a live frame assert
        //   agent.running_context().unwrap().realm() == vm.frame().unwrap().realm()
        // and that after the call returns the running_context realm reverts.
    }
```
Run it; expect PASS after Step 7. Run `cargo test -p lyng_vm`; expect PASS.

- [ ] **Step 9: Commit**

```bash
git add crates/env/src/execution.rs crates/env/src/agent.rs crates/env/src/lib.rs crates/env/src/tests.rs crates/vm/src/vm.rs crates/vm/src/vm/bytecode_calls.rs crates/vm/src/vm/generators.rs crates/vm/src/vm/internal_calls.rs crates/vm/src/vm/jobs.rs crates/vm/src/vm/async_functions.rs
git commit -m "feat: add Agent running_context scalar + Vm refresh at frame transitions

The single ambient { realm, referrer } the ops crate will read (JSC topCallFrame
analog). Refreshed at the existing context push/pop sites. No readers yet.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 8: Migrate the `ops` crate readers to the scalar

**Files:**
- Modify: `crates/ops/src/errors.rs` (~23-29)
- Modify: `crates/ops/src/promise.rs` (~38-43)
- Test: `crates/ops/src/errors.rs` test module (multi-realm)

- [ ] **Step 1: Write the failing test (multi-realm error prototype)**

In `crates/ops/src/errors.rs` tests, add a test that sets `running_context` to a non-default realm and asserts `current_realm` (via `error_value`) selects that realm's prototype:
```rust
    #[test]
    fn error_uses_running_context_realm() {
        // Create a second realm with distinct error prototypes; set
        // agent.set_running_context(Some(RunningContext::new(second_realm, None)));
        // assert throw_type_error(agent) yields an object whose prototype is the
        // SECOND realm's type_error_prototype, not the default realm's.
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p lyng_ops --lib error_uses_running_context_realm`
Expected: FAIL — `current_realm` still reads `current_execution_context()` (which the test doesn't set), so it falls back to default.

- [ ] **Step 3: Point `current_realm` at the scalar (`errors.rs` ~23-29)**

```rust
fn current_realm(agent: &Agent) -> Option<RealmRecord> {
    let realm = agent
        .running_context()
        .map(lyng_env::RunningContext::realm)
        .or_else(|| agent.default_realm_id())?;
    agent.realm(realm)
}
```

- [ ] **Step 4: Point the promise referrer read at the scalar (`promise.rs` ~38-43)**

```rust
    let script_or_module_referrer = agent
        .running_context()
        .and_then(lyng_env::RunningContext::referrer);
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p lyng_ops`
Expected: PASS (incl. the new test).
Run: `cargo test -p lyng_vm`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/ops/src/errors.rs crates/ops/src/promise.rs
git commit -m "refactor(ops): read realm/referrer from the running_context scalar

errors::current_realm and promise reaction referrer no longer touch the
execution-context stack.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 9: Migrate the `dynamic_import` readers

**Files:**
- Modify: `crates/vm/src/vm/builtin_dispatch/dynamic_import.rs` (~293-296, ~307-309, and the realm guards ~295/308)
- Test: a Test262-style dynamic-import test if one is not already covered (otherwise rely on the suite)

- [ ] **Step 1: Replace `active_script_or_module_referrer` (~293-296)**

This is a `Vm` method (`&self`/has `self.frame()`), but the current body reads `agent.current_execution_context()`. Use the running-context scalar (already correct here) or `self.current_referrer()`:
```rust
    pub(crate) fn active_script_or_module_referrer(&self, agent: &Agent) -> Option<ModuleKey> {
        agent
            .running_context()
            .and_then(lyng_env::RunningContext::referrer)
            .map(|atom| ModuleKey::new(agent.atoms().resolve(atom).to_owned().into_boxed_str()))
    }
```
(If this fn is currently `fn active_script_or_module_referrer(agent: &Agent)` with no `self`, keep it agent-only and just swap the body to `running_context()` — do NOT add `self` unless callers have it.)

- [ ] **Step 2: Replace the settle-job referrer read (~307-309)**

```rust
        let script_or_module_referrer = agent
            .running_context()
            .and_then(lyng_env::RunningContext::referrer);
```

- [ ] **Step 3: Replace the realm guards (~295, ~308)**

These call `current_execution_context()` only for `.is_some()` presence. Replace with `agent.running_context().is_some()` (presence of an active context). Confirm the surrounding logic only needs presence, not a specific field.

- [ ] **Step 4: Run tests**

Run: `cargo test -p lyng_vm`
Expected: PASS.
Run the Test262 dynamic-import subset (see Task 17 for the harness invocation); expect 100%.

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm/builtin_dispatch/dynamic_import.rs
git commit -m "refactor(vm): dynamic_import reads referrer from running_context

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 10: Migrate the remaining `current_execution_context()` readers (vm crate)

Every remaining vm-crate site that reads a FIELD off `current_execution_context()` switches to the live frame (`self.frame()`, or an in-scope `caller`/`record: &FrameRecord`). Sites that only check `.is_some()` switch to `agent.running_context().is_some()`.

**Field → frame accessor mapping (identical names):**
- `.realm()` → `frame.realm()`
- `.lexical_env()` → `frame.lexical_env()`
- `.variable_env()` → `frame.variable_env()`
- `.private_env()` → `frame.private_env()`
- `.new_target()` → `frame.new_target()`
- `.this_state()` → `frame.this_state()`
- `.kind()` → `frame.kind()`
- `.script_or_module_referrer()` → `self.current_referrer()`
- `.is_some()` guard → `agent.running_context().is_some()`

**Files & exact sites** (from the census; read each for surrounding context):
- `crates/vm/src/vm/dynamic_compilation.rs:662` (`.private_env()`), `:679-682` (`.this_state()`, `.new_target()`)
- `crates/vm/src/vm/bytecode_calls.rs:270,450,818` (`.is_some()` guards)
- `crates/vm/src/vm/semantics/names.rs:606` (`this` resolve — `.this_state()`/`this_value`)
- `crates/vm/src/vm/builtin_dispatch/class_helpers.rs:157,166` (guards)
- `crates/vm/src/vm/builtin_dispatch/class_helpers/private_fields.rs:33,171` (guards)
- `crates/vm/src/vm/builtin_dispatch/class_helpers/super_ops.rs:169` (`.this_state()`)
- `crates/vm/src/vm/generators.rs:1630` (guard), `:1671` (`.private_env()`), `:1673` (`.this_state()`), `:1701` (`.script_or_module_referrer()`)
- `crates/vm/src/vm/jobs.rs:539` (guard)
- `crates/vm/src/dsl/llint_state.rs:157` (guard)

**Note on `bytecode_calls.rs:269,449`** (the referrer *inheritance* reads that seed the new context): these become dead once readers move off the context; they are removed in Task 14 when the context push is deleted. Leave them for now.

**Files:**
- Modify: the files listed above
- Test: `crates/vm/src/vm.rs` or nearest module — a `super()`-in-arrow this_state test (also covers the scan in Task 12)

- [ ] **Step 1: Write the failing test (super this_state via frame)**

```rust
    #[test]
    fn super_initializes_this_for_arrow_closures() {
        // class B { constructor(){ this.x = 1 } }
        // class D extends B { constructor(){ const f = () => this; super(); return f() } }
        // new D() must return the constructed object (this initialized), not throw.
        // Run via the script harness and assert no ReferenceError and identity holds.
    }
```
This passes today (context path); it must KEEP passing after migrating `super_ops.rs:169` and `names.rs:606` to the frame. Treat it as a guard test added before the migration.

- [ ] **Step 2: Run it (baseline green)**

Run: `cargo test -p lyng_vm --lib super_initializes_this_for_arrow_closures`
Expected: PASS (before migration — it's a guard).

- [ ] **Step 3: Migrate each site**

For each site above, replace `agent.current_execution_context().<map>` per the mapping table. Representative diffs:

`dynamic_compilation.rs:679-682` (extracting arrow call-state) — was:
```rust
        let this_state = context.this_state();
        let new_target = context.new_target();
```
becomes (using the live frame in scope; if the function already binds `caller`/`record: &FrameRecord`, use it; else `self.frame()`):
```rust
        let frame = self.frame().ok_or_else(|| /* same error as the old None path */)?;
        let this_state = frame.this_state();
        let new_target = frame.new_target();
```

`super_ops.rs:169-171` — was:
```rust
                    || agent
                        .current_execution_context()
                        .is_some_and(|context| context.this_state() != ThisState::Uninitialized)
```
becomes:
```rust
                    || self
                        .frame()
                        .is_some_and(|frame| frame.this_state() != ThisState::Uninitialized)
```

Guards like `class_helpers.rs:157` — was `agent.current_execution_context().is_some()` → `agent.running_context().is_some()`.

- [ ] **Step 4: Run the vm suite**

Run: `cargo test -p lyng_vm`
Expected: PASS (including the guard test).

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm/dynamic_compilation.rs crates/vm/src/vm/bytecode_calls.rs crates/vm/src/vm/semantics/names.rs crates/vm/src/vm/builtin_dispatch/class_helpers.rs crates/vm/src/vm/builtin_dispatch/class_helpers/private_fields.rs crates/vm/src/vm/builtin_dispatch/class_helpers/super_ops.rs crates/vm/src/vm/generators.rs crates/vm/src/vm/jobs.rs crates/vm/src/dsl/llint_state.rs
git commit -m "refactor(vm): read execution-context fields from the live frame

All vm-crate readers now use self.frame()/caller/record and self.current_referrer()
instead of agent.current_execution_context(). Context stack now unread except for
the lexical-env this_state scan (next task) and the job context.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 11: Relocate the whole-stack this_state scan to the frame stack

**Files:**
- Modify: `crates/vm/src/vm/builtin_dispatch/class_helpers/super_ops.rs` (~220-228)
- Modify: `crates/vm/src/frame.rs` — already has `set_this_state` (Task 1)
- Test: reuse `super_initializes_this_for_arrow_closures` (Task 10) — it covers this

- [ ] **Step 1: Replace the agent scan with a frame scan**

At `super_ops.rs` ~220-228, replace:
```rust
            if !agent.set_execution_context_this_state_for_lexical_env(
                function_env,
                ThisState::Value(this_value),
            ) {
                let _ =
                    agent.set_current_execution_context_this_state(ThisState::Value(this_value));
            }
        } else {
            let _ = agent.set_current_execution_context_this_state(ThisState::Value(this_value));
        }
```
with a frame-stack scan (this method has `&mut self`, so `self.frames` is available):
```rust
            let mut updated = false;
            for frame in self
                .frames
                .iter_mut()
                .filter(|frame| frame.lexical_env() == function_env)
            {
                frame.set_this_state(ThisState::Value(this_value));
                updated = true;
            }
            if !updated {
                if let Some(frame) = self.frames.last_mut() {
                    frame.set_this_state(ThisState::Value(this_value));
                }
            }
        } else if let Some(frame) = self.frames.last_mut() {
            frame.set_this_state(ThisState::Value(this_value));
        }
```
(The Task-3 `frame.set_this_state(...)` at ~256 on the located `frame_index` stays — it covers the constructed-frame case; this scan covers the arrow-closure-sharing-lexical-env case, matching the old dual update.)

- [ ] **Step 2: Run the guard test + vm suite**

Run: `cargo test -p lyng_vm --lib super_initializes_this_for_arrow_closures`
Expected: PASS.
Run: `cargo test -p lyng_vm`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add crates/vm/src/vm/builtin_dispatch/class_helpers/super_ops.rs
git commit -m "refactor(vm): scan frames (not execution_contexts) for super this_state

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 12: Push a synthetic job root frame; stop pushing the job context

**Files:**
- Modify: `crates/vm/src/vm/jobs.rs` (`execute_runtime_job` ~101-196; `synthetic_job_caller_frame` ~728-740)
- Test: a job-referrer / job-realm test (dynamic import inside a promise reaction)

- [ ] **Step 1: Write the failing test**

```rust
    #[test]
    fn import_inside_promise_reaction_uses_job_referrer() {
        // Enqueue a promise reaction whose handler runs import('x'); assert the
        // resolved referrer equals the job's payload referrer (set the payload
        // referrer to a known atom and check the import request carries it).
        // Reuse the jobs test harness.
    }
```
If a direct harness is hard, instead assert the lower-level invariant: after `execute_runtime_job` enters, `vm.frame()` is the job root frame (kind == Job, realm == job realm) and `agent.running_context().unwrap().realm()` equals the job realm.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p lyng_vm --lib import_inside_promise_reaction_uses_job_referrer`
Expected: FAIL — there is no job frame on the stack; `vm.frame()` is empty (or stale).

- [ ] **Step 3: Push the synthetic frame; drop the context push**

In `execute_runtime_job` (~132-193), replace the context push/pop with a frame push/pop. Build the synthetic frame from the existing helper and seed its `this_state` (Job frames have no `this`; leave default) — the referrer scope is already pushed in Task 6:
```rust
        let job_base_depth = self.frames.len();
        self.push_referrer_scope(job_base_depth, script_or_module_referrer);
        let job_frame = self.synthetic_job_caller_frame(&realm_record);
        self.frames.push(job_frame);
        self.refresh_running_context(agent);
        let result = match job.payload() { /* …unchanged… */ };
        // teardown:
        while self.frames.len() > job_base_depth {
            let _ = self.frames.pop();
        }
        self.unwind_referrer_scopes_to(job_base_depth);
        self.refresh_running_context(agent);
        agent.clear_kept_objects();
        result
```
Remove the `agent.push_execution_context(ExecutionContext::job(...))` and the `let _ = agent.pop_execution_context();` lines. (The job helpers that call `synthetic_job_caller_frame(realm)` as a local `caller` argument are unaffected — they keep building a local frame for the inner call; only the ROOT job frame now lives on `self.frames`.)

- [ ] **Step 4: Verify depth invariants**

Confirm `execute_runtime_job` is only invoked when `self.frames` is empty (microtask drain). Add a debug assert:
```rust
        debug_assert_eq!(self.frames.len(), job_base_depth);
```
and confirm the job's inner calls push/pop their own frames above `job_base_depth` and fully unwind before teardown.

- [ ] **Step 5: Run tests + full promise/async vm tests**

Run: `cargo test -p lyng_vm --lib import_inside_promise_reaction_uses_job_referrer`
Expected: PASS.
Run: `cargo test -p lyng_vm`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/vm/src/vm/jobs.rs
git commit -m "feat(vm): run jobs on a synthetic root frame, drop the job ExecutionContext

The one non-1:1 case now has a real frame at depth 0; current frame + running
context work uniformly during jobs.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 13: Test262 + full-suite checkpoint (no code change)

Before deleting the type, confirm everything is green with the context stack now fully unread (except its own push/pop bookkeeping).

- [ ] **Step 1: Run the entire workspace test suite**

Run: `cargo test --workspace`
Expected: PASS.

- [ ] **Step 2: Run Test262**

Run the harness the repo uses (check `crates/test262-harness` README / the existing CI invocation; typically something like):
`cargo run -p lyng-test262-harness --release -- --threads 8`
Expected: **100% pass rate, identical to the pre-SP-0a baseline.** If any test regresses, STOP and bisect against `main` — do not proceed to deletion.

- [ ] **Step 3: No commit** (verification only). Record the pass counts in the task notes.

---

## Task 14: Delete the old path

Now remove the execution-context stack, its API, the redundant unwind loops, and the `ExecutionContext` type.

**Files:**
- Modify: `crates/vm/src/vm.rs` (entry: drop `push_execution_context`/the `prior_context_depth` loop ~1718-1769), `crates/vm/src/vm/bytecode_calls.rs` (drop context push ~278-300/458-489 and the inherited-referrer reads ~269/449, drop `pop_execution_context` ~349), `crates/vm/src/vm/internal_calls.rs` (drop the context-depth loop ~40-42), `crates/vm/src/vm/generators.rs` (drop context rebuild/pop), `crates/vm/src/vm/async_functions.rs` (drop context len/pop ~44/86/218/359), `crates/vm/src/vm/jobs.rs` (already done), `crates/vm/src/vm/exceptions.rs` (~109), `crates/vm/src/vm/dispatch_state.rs` (~144), `crates/vm/src/vm/semantics/control_flow.rs` (~285), `crates/vm/src/vm/call.rs` (~284/295/322)
- Modify: `crates/env/src/agent.rs` (drop `execution_contexts` field + init + the `AgentCollectionSnapshot` clone ~65 and trace ~97-99)
- Delete: `crates/env/src/agent/execution_contexts.rs`; remove its `mod` line in `agent.rs` (~26)
- Modify: `crates/env/src/execution.rs` (delete `ExecutionContext` struct + its `impl` + its `TraceHeapEdges`; KEEP `ExecutionContextKind`, `ExecutableId`, `ThisState`, `RunningContext`)
- Modify: re-export site (`crates/env/src/lib.rs`) — drop `ExecutionContext` from `pub use`

- [ ] **Step 1: Remove the context push/pop calls and redundant unwind loops**

In each vm-crate file, delete the `agent.push_*_context(...)`, `agent.pop_execution_context()`, `agent.current_execution_context()` (any stragglers should be none after Tasks 8-12), and the `while agent.execution_contexts().len() > prior_context_depth { … }` loops plus their `prior_context_depth` bindings. The frame unwind loops and `unwind_referrer_scopes_to` + `refresh_running_context` already cover teardown.

In `bytecode_calls.rs` delete the now-dead `let script_or_module_referrer = agent.current_execution_context()...` inheritance reads at ~269/449 (the referrer is established via scopes now).

- [ ] **Step 2: Remove the `Agent` field + snapshot wiring**

Delete `execution_contexts: Vec<ExecutionContext>` (field + `new()` init + `AgentCollectionSnapshot` field/clone/trace). Confirm `running_context` is traced (Task 7) and the frame GC trace covers all heap edges (Task 4).

- [ ] **Step 3: Delete the API module + the type**

Delete `crates/env/src/agent/execution_contexts.rs` and its `mod execution_contexts;`. Delete the `ExecutionContext` struct/impl/trace in `execution.rs`. Remove `ExecutionContext` from re-exports.

- [ ] **Step 4: Fix compile errors iteratively**

Run: `cargo build --workspace`
Fix each error (leftover references). Expected end state: clean build. Any remaining `ExecutionContext` reference is a missed reader — migrate it per the Task 10 mapping.

- [ ] **Step 5: Rewrite/remove the env tests that asserted on the stack**

In `crates/env/src/tests.rs`, the assertions at ~90 (`execution_contexts().is_empty()`), ~1056-1066 (push_*_context + kind assertions), ~1060 (`.to_vec()`): delete the ones that test the removed API; convert any still-meaningful ones to assert on `running_context()`/frame state. Do the same for `crates/tests/src/runtime_substrate_surface.rs:40-42` and `crates/tests/src/execution_semantics/runtime_pipeline.rs:37` (they assert `context.kind()`/`executable()`), and `crates/env/src/tests.rs` job/eval/builtin context tests — these context kinds were dead-or-test-only; delete the dead ones.

- [ ] **Step 6: Run the full suite + Test262**

Run: `cargo test --workspace`
Expected: PASS.
Run Test262 (same command as Task 13).
Expected: **100%, identical to baseline.**

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "refactor: delete ExecutionContext and the execution-context stack

The Vm frame stack is now the single source of truth; realm/referrer reach the
ops crate via the running_context scalar; jobs run on a synthetic root frame.
ExecutionContextKind, ExecutableId, ThisState, RunningContext retained.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 15: Targeted regression tests for the risky spots

Add explicit coverage so the four hard cases are guarded independently of Test262.

**Files:**
- Test: `crates/vm/src/vm.rs` (or the nearest integration test module) + `crates/ops/src/errors.rs`

- [ ] **Step 1: Add the four tests** (some may already exist from earlier tasks — keep one canonical copy each):
  1. **Multi-realm error** — `error_uses_running_context_realm` (Task 8). Confirm present.
  2. **super() arrow this** — `super_initializes_this_for_arrow_closures` (Task 10). Confirm present.
  3. **Generator referrer across resume** — generator created with referrer A, resumed under a different running-context realm/referrer B, runs `import()` → resolves against A:
```rust
    #[test]
    fn generator_referrer_survives_cross_context_resume() {
        // Build a generator whose establishment referrer is atom A; drive a resume
        // from a context whose running referrer is B; assert the import request
        // inside the generator carries A (capture-and-restore), not B.
    }
```
  4. **import in job** — `import_inside_promise_reaction_uses_job_referrer` (Task 12). Confirm present.

- [ ] **Step 2: Run them**

Run: `cargo test -p lyng_vm; cargo test -p lyng_ops`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "test: guard SP-0a hard cases (multi-realm, super-arrow this, gen referrer, job import)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 16: Perf gate (no regression on the V8 suite)

**Files:** none (measurement).

- [ ] **Step 1: Capture the baseline** — check out `main`, run the V8 bench harness the repo uses (Richards + RayTrace; see how prior commits report bundled scores, e.g. the RayTrace A/B in project memory). Record scores.

- [ ] **Step 2: Run on the branch** — `git checkout feat/sp0a-eliminate-execution-context`, run the same bench.

- [ ] **Step 3: Compare** — `FrameRecord` grew by `this_state` (1-byte enum) + `private_env` (`Option<EnvironmentRef>`); referrer stayed off the frame. Expected: **flat** (within noise). If Richards/RayTrace regress beyond noise, investigate the `FrameRecord` `Copy` size: pack `this_state` into the existing `FrameFlags`/spare bits rather than a standalone field, re-measure.

- [ ] **Step 4: Record results** in the task notes. No commit unless packing was needed (then commit the packing change with a `perf(vm):` message).

---

## Self-Review (completed by plan author)

- **Spec coverage:** §A this_state (Tasks 1,3,11) + private_env (Tasks 2,3,4); §B realized as direct `self.frame()` migration (Tasks 9-10), no separate bridge — faithful to "incrementally migratable, deleted at end"; §C running_context scalar (Tasks 7,8); §D referrer side-stack + walk + generator capture (Tasks 5,6,15); §E job root frame (Task 12) + whole-stack scan relocation (Task 11); deletion (Task 14); testing §G (Tasks 13,15) + perf §H (Task 16). The referrer inherit-vs-spec follow-up is intentionally NOT a task (out of SP-0a scope).
- **Placeholders:** test-harness specifics ("reuse the harness from Task 3", "the script harness") are deliberate — the exact mid-execution inspection API must be read from the existing vm test module; every code change to non-test source is shown in full.
- **Type consistency:** `RunningContext::new/realm/referrer`, `with_this_state`/`this_state`/`set_this_state`, `with_private_env`/`private_env`, `push_referrer_scope`/`unwind_referrer_scopes_to`/`current_referrer`, `refresh_running_context` used consistently across tasks.
