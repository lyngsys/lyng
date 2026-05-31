# Rust `Construct` Fast Path Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the `Construct` opcode a monomorphic Rust fast path that skips `argument_scratch` materialization (caller-register-window copy) and skips the per-`new` `.prototype` get via a cached, watchpoint-guarded prototype — attacking RayTrace's 41.7% single largest cost center.

**Architecture:** Mirror the existing `Call*` Rust fast path. Consume the dormant `ConstructCacheEntry` IC; extend it with the resolved prototype; guard it with a new per-constructor `.prototype`-reassignment watchpoint (the asm inline-write handler is gated off for the function `prototype` slot so every write is observed in Rust). On a guard miss / any excluded callee, fall back to the unchanged slow path.

**Tech Stack:** Rust workspace. Crates touched: `lyng-vm` (call/construct path, IC, watchpoint dispatch), `lyng-objects` (watchpoint storage, write choke points). Tests: `lyng-vm` (low-level IC/watchpoint via `Runtime`+`BytecodeBuilder`), `lyng-tests` (behavior via `compile_and_run`).

**Spec:** `docs/superpowers/specs/2026-05-30-construct-fast-path-design.md`

**Branch:** `feat/construct-fast-path` (already created; spec committed there as `82c88926`).

---

## Orientation (read before Task 1)

Anchor code the tasks mirror or modify:

- Construct slow path: `crates/vm/src/vm/call.rs:644` (`construct_value`).
- `create_construct_this` (the `.prototype` get to remove): `crates/vm/src/vm/bytecode_calls.rs:653`.
- Call fast-path template: `crates/vm/src/vm/call.rs:404` (`call_value_small`), `:479` (`ordinary_bytecode_call_eligibility`), `:547` (`invoke_bytecode_call_from_caller_arg_window`).
- Register-window frame entry: `crates/vm/src/vm/bytecode_calls.rs:382` (`enter_bytecode_call_from_caller_registers`), `:414` (`install_prepared_bytecode_call_from_registers`), `:475` (`copy_arguments_from_caller_registers`).
- Slow-path construct frame entry to mirror for this/new_target: `crates/vm/src/vm/bytecode_calls.rs:19` (`enter_bytecode_call`, threads `new_target` + `construct_call` → `install_prepared_bytecode_call(... construct_this, construct_call)`).
- Construct IC: `crates/vm/src/vm/feedback.rs:194` (`ConstructCacheEntry`), `:203` (`from_constructor`), `:248` (`ConstructCacheStorage`), `:496` (`observe_construct_target` — the write template; shows the `construct_ic_states[index]` + `construct_cache_entries` indexing), `:2647` (`observe_construct_target_on_state`).
- Construct IC slabs on `Vm`: `crates/vm/src/vm.rs:159` (`construct_ic_states: Vec<Option<Box<[Option<CallIcState>]>>>`), `:165` (`construct_cache_entries`).
- Property IC watchpoint pattern to mirror: `crates/vm/src/vm/feedback.rs:922` (`register_own_write_watchpoints`), and the agent dispatch that routes a fired observer to `Vm::clear_ic_slot_if_generation_matches`.
- Watchpoint types: `crates/objects/src/watchpoint.rs:118` (`Watchpoint`), the `ShapeInvalidationObserver` enum and `WatchpointSet` (`:122`).
- Watchpoint storage + fire: `crates/objects/src/runtime.rs:60` (`watchpoint_sets: HashMap<ShapeId, WatchpointSet>`), `:1257` (`watchpoint_set_mut`), `:1284` (`sweep_invalidated_watchpoint_sets`); `crates/objects/src/watchpoint.rs:182` (`Agent::fire_watchpoints_for_shape` reference).
- Write choke points: `crates/objects/src/internal_methods.rs:282` (`set`), `:175` (`define_own_property`).
- Function `.prototype` install + flags: `crates/builtins/src/public.rs:1233`; `crates/objects/src/functions.rs:997` (`has_prototype_property`), and `derived_class_constructor()` flag used at `crates/vm/src/vm/call.rs:720`.

**Test commands:** low-level `cargo test -p lyng-vm <filter>`; behavior `cargo test -p lyng-tests <filter>`. Build gate: `cargo build -p lyng-vm`. Format: `cargo fmt`. Lint: `cargo clippy -p lyng-vm -p lyng-objects --tests`.

**Behavior-test harness** (`crates/tests/src/execution_semantics/`): `use super::support::{compile_and_run, compile_and_run_string};` — `compile_and_run(src) -> lyng_types::Value`. See `crates/tests/src/execution_semantics/classes.rs:8` for the canonical pattern.

---

## Task 1: Cache the resolved prototype in `ConstructCacheEntry`

**Files:**
- Modify: `crates/vm/src/vm/feedback.rs:194` (struct `ConstructCacheEntry`) and `:203` (`from_constructor`)
- Test: `crates/vm/src/tests/feedback.rs`

The created object's `[[Prototype]]` already *is* the resolved prototype (it is what `create_construct_this` installed, including the `Object.prototype` fallback). Read it from `created` rather than re-resolving.

- [ ] **Step 1: Write the failing test**

In `crates/vm/src/tests/feedback.rs`, add a test that runs a constructor twice through the slow path and asserts the recorded `ConstructCacheEntry.prototype` equals the created instance's `[[Prototype]]`. Mirror an existing construct-observing test in this file (search for `observe_construct` / `ConstructCacheEntry`); if none exists, use the `compile_unit` + `Vm::run` setup from a neighboring test and then read the entry via a test-only accessor.

```rust
#[test]
fn construct_cache_entry_records_resolved_prototype() {
    // Run `function F(){}; new F(); new F();` so the construct slot caches.
    // After the run, fetch the monomorphic ConstructCacheEntry for F's
    // construct site and assert entry.prototype == F.prototype object.
    // (Use the same harness pattern as the existing construct IC tests in
    // this file; assert entry.prototype.is_some() and equals the instance proto.)
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p lyng-vm construct_cache_entry_records_resolved_prototype`
Expected: FAIL — field `prototype` does not exist on `ConstructCacheEntry`.

- [ ] **Step 3: Add the field and populate it**

In the struct (`feedback.rs:194`) add `pub(super) prototype: Option<ObjectRef>,`. In `from_constructor` (`:203`), after computing `created_shape`, derive the prototype from the created object's header:

```rust
let prototype = created.and_then(|object| {
    agent
        .objects()
        .object_header(agent.heap().view(), object)
        .and_then(|header| header.prototype()) // resolved proto, incl. Object.prototype fallback
});
```

Add `prototype` to the returned `Self { .. }`. (Confirm the header accessor name against `crates/objects/src/object_records.rs:190` `prototype()`; if the header exposes it differently, use that accessor.)

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p lyng-vm construct_cache_entry_records_resolved_prototype`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm/feedback.rs crates/vm/src/tests/feedback.rs
git commit -m "feat(vm): cache resolved prototype in ConstructCacheEntry"
```

---

## Task 2: Per-constructor `.prototype` watchpoint — storage + observer variant

**Files:**
- Modify: `crates/objects/src/watchpoint.rs:118` (add observer variant), `crates/objects/src/runtime.rs:60` (storage) + register/fire API near `:1257`
- Test: `crates/objects/src/tests.rs`

Add a per-constructor watchpoint set (keyed by the constructor `ObjectRef`) and a new observer payload identifying a construct IC slot to clear. This task is data + objects-side API only; VM-side clearing is Task 3.

- [ ] **Step 1: Write the failing test**

In `crates/objects/src/tests.rs`:

```rust
#[test]
fn construct_prototype_watchpoint_fires_and_collects_observer() {
    // Build a RuntimeObjects, pick any ObjectRef `ctor`.
    // Register a ConstructIcClear observer on ctor's construct-prototype set.
    // Assert the set state is Watched.
    // Fire it, collecting observers into a sink; assert the sink contains the
    // observer payload and the set is now Invalidated.
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p lyng-objects construct_prototype_watchpoint_fires_and_collects_observer`
Expected: FAIL — API/variant absent.

- [ ] **Step 3: Add the observer variant**

In `crates/objects/src/watchpoint.rs`, add to `ShapeInvalidationObserver` a variant:

```rust
ConstructIcClear { code: CodeRef, slot: FeedbackSlotId, generation: u32 },
```

Add its arm to any `match` over the observer enum (the dispatcher arm at `watchpoint.rs:112`-style site — leave the actual VM clear as a no-op routed via the agent, matching how `AdaptiveProtoLoad`/`AdaptiveOwnWrite` are handled). Ensure `fire_into`/collection includes it.

- [ ] **Step 4: Add per-constructor storage + API**

In `crates/objects/src/runtime.rs` next to `watchpoint_sets` (`:60`):

```rust
pub(crate) construct_prototype_watchpoints: HashMap<ObjectRef, WatchpointSet>,
```

Add accessors mirroring `watchpoint_set_mut` (`:1257`):

```rust
pub fn construct_prototype_watchpoint_mut(&mut self, ctor: ObjectRef) -> &mut WatchpointSet {
    self.construct_prototype_watchpoints.entry(ctor).or_default()
}
pub fn construct_prototype_watchpoint_inspect(&self, ctor: ObjectRef) -> Option<&WatchpointSet> {
    self.construct_prototype_watchpoints.get(&ctor)
}
```

Add `construct_prototype_watchpoints` to `RuntimeObjects::new`/`Default` initialization, and add it to `sweep_invalidated_watchpoint_sets` (`:1284`) so invalidated entries are reaped.

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p lyng-objects construct_prototype_watchpoint_fires_and_collects_observer`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/objects/src/watchpoint.rs crates/objects/src/runtime.rs crates/objects/src/tests.rs
git commit -m "feat(objects): per-constructor .prototype watchpoint storage + ConstructIcClear observer"
```

---

## Task 3: Fire the watchpoint from the Rust write choke points + clear the construct IC

**Files:**
- Modify: `crates/objects/src/internal_methods.rs:282` (`set`), `:175` (`define_own_property`)
- Modify: VM-side dispatch — add `Vm::clear_construct_ic_slot_if_generation_matches` and route the `ConstructIcClear` observer to it (mirror `clear_ic_slot_if_generation_matches`; find its agent-dispatch site via `grep -rn clear_ic_slot_if_generation_matches crates/`)
- Test: `crates/tests/src/execution_semantics/classes.rs` (behavior) and `crates/vm/src/tests/inline_caches.rs` (IC cleared)

- [ ] **Step 1: Write the failing behavior test**

In `crates/tests/src/execution_semantics/classes.rs`:

```rust
#[test]
fn reassigning_function_prototype_is_observed_by_construct() {
    let result = compile_and_run_string(
        r"
        function F() {}
        F.prototype = { tag: 'first' };
        let a = new F();
        F.prototype = { tag: 'second' };   // must invalidate the construct IC
        let b = new F();
        a.__proto__.tag + ',' + b.__proto__.tag;
        ",
    );
    assert_eq!(result, "first,second");
}

#[test]
fn define_property_on_function_prototype_is_observed_by_construct() {
    let result = compile_and_run_string(
        r"
        function F() {}
        let a = new F();
        Object.defineProperty(F, 'prototype', { value: { tag: 'redef' } });
        let b = new F();
        (Object.getPrototypeOf(a) === Object.getPrototypeOf(b)) + '';
        ",
    );
    assert_eq!(result, "false");
}
```

- [ ] **Step 2: Run tests to verify they fail (or pass trivially pre-fast-path)**

Run: `cargo test -p lyng-tests reassigning_function_prototype_is_observed_by_construct define_property_on_function_prototype_is_observed_by_construct`
Expected: PASS now (slow path re-reads `.prototype` every time). These are **regression guards** that must STILL pass after Task 7 introduces the cache. Keep them; they fail only if the fast path skips a write it shouldn't.

- [ ] **Step 3: Add `Vm::clear_construct_ic_slot_if_generation_matches`**

Mirror `clear_ic_slot_if_generation_matches` but target the construct slabs (`construct_ic_states` / `construct_cache_entries`, `crates/vm/src/vm.rs:159`/`:165`): if the slot's `CallIcState.generation == generation`, reset that construct slot to empty. Route the `ConstructIcClear` observer to it at the same agent-dispatch site that handles `AdaptiveOwnWrite`.

- [ ] **Step 4: Fire from the write choke points**

In `internal_methods.rs` `set` (`:282`) and `define_own_property` (`:175`), after a value write that targets an existing own slot, add:

```rust
// Function `.prototype` reassignment invalidates any construct allocation
// profile keyed on this constructor (the construct IC caches the prototype).
if key == PropertyKey::from_atom(WellKnownAtom::prototype.id())
    && self.construct_prototype_watchpoint_inspect(id).is_some()
{
    self.fire_construct_prototype_watchpoint(id); // marks Invalidated + collects observers for agent dispatch
}
```

Implement `fire_construct_prototype_watchpoint` to mirror the shape-keyed fire path (`Agent::fire_watchpoints_for_shape`, `watchpoint.rs:182`): take the set, mark invalidated, hand the collected `ConstructIcClear` observers to the VM dispatch after the borrow is released.

- [ ] **Step 5: Add the IC-cleared low-level test**

In `crates/vm/src/tests/inline_caches.rs`, mirror `adaptive_own_write_watchpoint_clears_ic_on_dictionary_transition` (`:4408`): construct `F` twice (cache populated), write `F.prototype`, assert the construct IC slot is reset.

- [ ] **Step 6: Run tests**

Run: `cargo test -p lyng-vm clear_construct && cargo test -p lyng-tests function_prototype`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/objects/src/internal_methods.rs crates/vm/ crates/tests/
git commit -m "feat: fire per-constructor .prototype watchpoint from write choke points; clear construct IC"
```

---

## Task 4: Gate the asm inline-write handler off for a function `prototype` slot

**Files:**
- Modify: `crates/vm/src/vm/feedback.rs` write-IC install (where `monomorphic_own_inline_write_handler` is set — see `:890`, `:1004`, and `named_property_install_slow_path` ~`:970`)
- Test: `crates/vm/src/tests/inline_caches.rs`

If a function's `prototype` slot could get an asm inline-write handler, a hot `F.prototype = …` site would mutate the slot without passing through the Rust choke points in Task 3, silently invalidating the soundness of Task 7's cache. Force such writes slow.

- [ ] **Step 1: Write the failing test**

In `crates/vm/src/tests/inline_caches.rs`:

```rust
#[test]
fn hot_function_prototype_write_does_not_install_inline_write_handler() {
    // Run a loop that assigns F.prototype = {...} many times (cross the IC
    // warmup threshold). Inspect the write site's PropertyIcState and assert
    // monomorphic_own_inline_write_handler is None (forced onto the Rust path).
    // Mirror the IC-state inspection used by existing inline_caches.rs tests.
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p lyng-vm hot_function_prototype_write_does_not_install_inline_write_handler`
Expected: FAIL — handler is installed.

- [ ] **Step 3: Add the gate**

At the point where the write IC decides to set `monomorphic_own_inline_write_handler`, refuse when the receiver is a function object and the written key is `prototype`:

```rust
let is_function_prototype_slot = agent
    .objects()
    .function_data(receiver)
    .is_some()
    && written_key == PropertyKey::from_atom(WellKnownAtom::prototype.id());
if is_function_prototype_slot {
    // Keep writes on the Rust slow path so the construct-IC watchpoint observes them.
    state.monomorphic_own_inline_write_handler = None;
}
```

(Use the receiver/key already available at the install site; if only the shape is available, gate on "shape is a function shape AND slot == the function prototype slot.")

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p lyng-vm hot_function_prototype_write_does_not_install_inline_write_handler`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm/feedback.rs crates/vm/src/tests/inline_caches.rs
git commit -m "feat(vm): force function .prototype writes off the asm inline-write fast path"
```

---

## Task 5: Construct eligibility gate

**Files:**
- Modify: `crates/vm/src/vm/call.rs` (add `ordinary_bytecode_construct_eligibility`, near `:479`)
- Test: `crates/vm/src/tests/metadata_and_tail_calls.rs` or `inline_caches.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn construct_eligibility_accepts_plain_ctor_rejects_special_callees() {
    // Build a plain bytecode function F (no rest/arguments/derived) -> Some(code).
    // Build: a bound function, a generator function, an async function, a
    // function with a rest param -> each None.
    // (Mirror how ordinary_bytecode_call_eligibility is exercised; if no such
    // test exists, assert at least the plain-Some and rest-param-None cases.)
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p lyng-vm construct_eligibility_accepts_plain_ctor`
Expected: FAIL — function not defined.

- [ ] **Step 3: Implement the gate**

Mirror `ordinary_bytecode_call_eligibility` (`call.rs:479`):

```rust
#[inline]
pub(in crate::vm) fn ordinary_bytecode_construct_eligibility(
    &self,
    agent: &Agent,
    callee_object: ObjectRef,
) -> Option<CodeRef> {
    if Self::bound_function_record(agent, callee_object).is_some() {
        return None;
    }
    if agent.objects().is_proxy_object(callee_object) {
        return None;
    }
    let code = Self::bytecode_entry(agent, callee_object)?;
    let function = self.installed_function(code)?;
    let flags = function.flags();
    if flags.generator() || flags.async_function() || flags.derived_class_constructor() {
        return None;
    }
    if function.arguments_mode() != ArgumentsMode::None || function.has_rest_parameter() {
        return None;
    }
    Some(code)
}
```

(Note: base-class constructors are constructible and stay eligible; only `derived_class_constructor()` is excluded. `new_target == callee` always holds on the `Construct` opcode.)

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p lyng-vm construct_eligibility_accepts_plain_ctor`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm/call.rs crates/vm/src/tests/
git commit -m "feat(vm): add ordinary_bytecode_construct_eligibility gate"
```

---

## Task 6: Thread `new_target` + `construct_this` + `construct_call` through the register-window entry

**Files:**
- Modify: `crates/vm/src/vm/bytecode_calls.rs:382` (`enter_bytecode_call_from_caller_registers`), `:414` (`install_prepared_bytecode_call_from_registers`)
- Test: `crates/vm/src/tests/core.rs` (regression: existing call path still works)

The register-window entry currently passes `new_target = None` and no construct flags. Generalize it so the construct fast path (Task 7) can reuse it, exactly as `install_prepared_bytecode_call` already supports for the slow path (`:65`).

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn register_window_construct_entry_sets_this_and_new_target() {
    // Drive enter_bytecode_call_from_caller_registers with construct params for
    // a plain bytecode constructor; assert the pushed frame's this_value is the
    // construct_this object and new_target == callee. (Use the BytecodeBuilder
    // setup from neighboring core.rs tests.)
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p lyng-vm register_window_construct_entry_sets_this_and_new_target`
Expected: FAIL — signature lacks construct params.

- [ ] **Step 3: Extend the signatures**

Add to both functions: `new_target: Option<ObjectRef>`, `construct_this: Option<ObjectRef>`, `construct_call: bool`. In `enter_bytecode_call_from_caller_registers`, pass `new_target` into `prepare_bytecode_call` (replacing the hardcoded `None` at `:393`). In `install_prepared_bytecode_call_from_registers`, set the frame `this_value`/`new_target`/construct flag the same way `install_prepared_bytecode_call` does (`with_this_value`, `with_new_target`, the `FrameFlags::construct()` path; see `:451`-`:465` and the slow-path analogue). Update the existing call-path caller (`invoke_bytecode_call_from_caller_arg_window`, `call.rs:567`) to pass `None, None, false`.

- [ ] **Step 4: Run tests to verify pass (new + regression)**

Run: `cargo test -p lyng-vm register_window_construct_entry_sets_this_and_new_target && cargo test -p lyng-vm call_`
Expected: PASS (new test + existing call regression suite).

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/vm/bytecode_calls.rs crates/vm/src/vm/call.rs crates/vm/src/tests/core.rs
git commit -m "refactor(vm): thread new_target/construct_this/construct_call through register-window entry"
```

---

## Task 7: Wire the fast path into `construct_value`

**Files:**
- Modify: `crates/vm/src/vm/call.rs:644` (`construct_value`)
- Test: `crates/tests/src/execution_semantics/classes.rs`, `crates/tests/src/execution_semantics/functions.rs`

The integration. Add a monomorphic fast branch at the top of `construct_value`, before the `argument_scratch` take. Everything not on the fast path falls through to the existing slow code unchanged.

- [ ] **Step 1: Write the failing/guard behavior tests**

```rust
#[test]
fn construct_fast_path_produces_correct_instances() {
    let result = compile_and_run_string(
        r"
        function Vec(x, y) { this.x = x; this.y = y; }
        let s = 0;
        for (let i = 0; i < 1000; i++) { let v = new Vec(i, i + 1); s += v.x + v.y; }
        s + '';
        ",
    );
    assert_eq!(result, "1000000"); // sum_{i<1000}(2i+1) = 1000^2
}

#[test]
fn construct_fast_path_excludes_fall_back_correctly() {
    let result = compile_and_run_string(
        r"
        function Base(v){ this.v = v; }
        class Derived extends Base { constructor(v){ super(v); this.d = v*2; } }
        let bound = Base.bind(null, 7);
        let p = new Proxy(Base, {});
        let a = new Derived(3);
        let b = new bound();          // bound -> slow path
        let c = new p(9);             // proxy  -> slow path
        let d = new Base(...[4]);     // spread -> slow path
        a.v + ',' + a.d + ',' + b.v + ',' + c.v + ',' + d.v;
        ",
    );
    assert_eq!(result, "3,6,7,9,4");
}
```

- [ ] **Step 2: Run tests to verify they fail or pass-via-slow-path**

Run: `cargo test -p lyng-tests construct_fast_path`
Expected: PASS via slow path before the change; they must STILL pass after — they assert correctness, the fast path must not change results.

- [ ] **Step 3: Implement the fast branch**

At the top of `construct_value` (after `let callee_value = self.read_register(...)`), insert:

```rust
if spread_mask.is_none()
    && let Some(callee) = callee_value.as_object_ref()
    && let Some(code) = self.ordinary_bytecode_construct_eligibility(agent, callee)
    && let Some(prototype) = self.monomorphic_construct_prototype(agent, frame.code(), feedback_slot, callee)
{
    // Skip the `.prototype` get: allocate construct_this with the cached prototype.
    let root_shape = agent.realm(frame.realm()).and_then(|r| r.root_shape())
        .ok_or(VmError::MissingRootShape(frame.realm()))?;
    let construct_this = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape).with_prototype(prototype),
            AllocationLifetime::Default,
        )
    });
    // Register the per-constructor .prototype watchpoint observer (idempotent;
    // skip if already Watched for this (code, slot, generation)).
    self.register_construct_prototype_observer(agent, frame.code(), feedback_slot, callee);
    advance_dispatch_frame(frame, instruction_len);
    self.sync_dispatch_frame(frame_depth, *frame);
    return self.invoke_bytecode_construct_from_caller_arg_window(
        agent, frame_depth, frame, result_register, callee, code,
        construct_this, arguments,
    );
}
// ...existing slow path unchanged...
```

Add the two new helpers in `call.rs`:

- `monomorphic_construct_prototype(&self, agent, code, slot, callee) -> Option<Option<ObjectRef>>` — read the construct IC slab for `(code, slot)`; return the cached `entry.prototype` **only if** the slot is monomorphic, `entry.constructor == callee`, and `entry.constructor_shape == callee`'s current shape. (Mirror the indexing in `observe_construct_target`, `feedback.rs:496`.) The outer `Option` is hit/miss; the inner `Option<ObjectRef>` is the (possibly null-proto) prototype.
- `register_construct_prototype_observer(&mut self, agent, code, slot, callee)` — `agent.objects_mut().construct_prototype_watchpoint_mut(callee).register(Watchpoint::ShapeInvalidation { observer: ShapeInvalidationObserver::ConstructIcClear { code, slot, generation } })`, ignoring `Err(Invalidated)` (a fired set means a write is racing; the slot will re-cache next time). Read `generation` from the construct slot's `CallIcState`.
- `invoke_bytecode_construct_from_caller_arg_window(...)` — mirror `invoke_bytecode_call_from_caller_arg_window` (`call.rs:547`) but pass construct params to `enter_bytecode_call_from_caller_registers` (Task 6): `new_target = Some(callee)`, `construct_this = Some(construct_this)`, `construct_call = true`, `this_value = Value::from_object_ref(construct_this)`. Call `observe_construct_target` afterward to keep the generation/observation fresh (mirror `observe_call_target` in the call analogue).

- [ ] **Step 4: Run the full behavior + construct suites**

Run: `cargo test -p lyng-tests construct && cargo test -p lyng-tests -- execution_semantics::classes execution_semantics::functions`
Expected: PASS (correctness preserved on fast and fallback paths).

- [ ] **Step 5: Re-run the Task 3 invalidation guards**

Run: `cargo test -p lyng-tests function_prototype reassigning_function_prototype`
Expected: PASS — the cache honors `.prototype` reassignment.

- [ ] **Step 6: Commit**

```bash
git add crates/vm/src/vm/call.rs crates/tests/
git commit -m "feat(vm): Construct fast path — skip .prototype get + caller-arg-window copy"
```

---

## Task 8: Verification — correctness, lint, and performance deltas

**Files:**
- Create: `reports/lyng/construct-fast-path-2026-05-30.md`

- [ ] **Step 1: Format + lint**

Run: `cargo fmt && cargo clippy -p lyng-vm -p lyng-objects --tests`
Expected: clean (no new warnings in touched files).

- [ ] **Step 2: Full VM + behavior suites**

Run: `cargo test -p lyng-vm && cargo test -p lyng-tests`
Expected: PASS.

- [ ] **Step 3: Test262 (no regression from 100%)**

Run the project's Test262 harness over `built-ins`, `language`, `staging`, `annexB` (see `reports/lyng/test262.md` for the canonical invocation, e.g. `cargo run --release -p lyng-bench -- test262 ...` / the test262 runner). Expected: same pass counts as baseline (100% in the started categories).

- [ ] **Step 4: Re-profile RayTrace + Richards**

Run:
```bash
cargo run --release -p lyng-bench -- profile --filter RayTrace --samples 3 \
  --report /tmp/profile-raytrace-after.md --json /tmp/profile-raytrace-after.json
cargo run --release -p lyng-bench -- profile --filter Richards --samples 3 \
  --report /tmp/profile-richards-after.md --json /tmp/profile-richards-after.json
```
Expected: `Construct` time share on RayTrace falls out of the #1 slot and its slow share / Samples-per-Mdispatch drop materially.

- [ ] **Step 5: v8suite throughput delta**

Run: `cargo run --release -p lyng-bench -- v8suite` and capture Richards/RayTrace scores vs the pre-change baseline.

- [ ] **Step 6: Write the results report + commit**

Record before/after `Construct` time share, v8suite deltas, and test262 parity in `reports/lyng/construct-fast-path-2026-05-30.md`.

```bash
git add reports/lyng/construct-fast-path-2026-05-30.md
git commit -m "docs(report): Construct fast path before/after profile + v8suite deltas"
```

---

## Self-review notes (author)

- **Spec coverage:** Component 1 → Task 5; Component 2 → Task 7 (`monomorphic_construct_prototype`); Component 3 → Task 1 + Task 7 alloc; Component 4 → Tasks 2, 3, 4; Component 5 → Tasks 6, 7; Verification → Task 8. All spec components mapped.
- **Soundness:** Task 4 (inline-write gating) lands before Task 7 so the cache is never live without the write being observable in Rust. Task 3's invalidation guards are written before the cache exists and re-run after (Task 7 Step 5).
- **Type consistency:** `ConstructIcClear { code, slot, generation }` is used identically in Tasks 2, 3, 7; `ordinary_bytecode_construct_eligibility -> Option<CodeRef>` in Tasks 5, 7; register-window entry construct params `(new_target, construct_this, construct_call)` consistent across Tasks 6, 7.
- **Known impl-time confirmations** (the implementing agent must verify against live code, not guess): exact `ObjectHeader::prototype()` accessor (Task 1); the agent-side observer dispatch site for routing `ConstructIcClear` (Task 3); whether the write-IC install site has the receiver `ObjectRef` or only the shape (Task 4); the construct slab `CallIcState.generation` field name (Tasks 3, 7).
