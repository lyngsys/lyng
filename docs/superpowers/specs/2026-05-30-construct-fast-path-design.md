# Rust `Construct` Fast Path — Design

**Date:** 2026-05-30
**Status:** Design approved; pending implementation plan
**Author:** brainstormed with Claude
**Target:** JSC LLInt parity on the V8 v7 suite — attack RayTrace's single
largest cost center (`Construct`).

## Problem

A fresh in-process time-attribution profile (`lyng-bench profile`, 3 samples,
200µs sampler) confirms that the call/return/construct cluster dominates
wall-time on both gate workloads, and that **`Construct` is the single biggest
individual lever on RayTrace**:

RayTrace — 85,977 samples, 546M dispatches:

| Opcode | Time share | Slow share |
| --- | ---: | ---: |
| `Construct` | **41.70%** | 99.97% |
| `Call2` | 13.32% | 99.83% |
| `ReturnUndefined` | 12.70% | 99.69% |
| (`Call1`/`Return`/`Call0`/`Call3`/`TailCall`/`Call`) | ~5.3% combined | ~99% |

→ call/return/construct cluster ≈ **72.6%** of RayTrace.

Richards — 23,375 samples, 727M dispatches: the cluster is ≈ **49.8%**, but
there it is `Call0` (21.41%) + `TailCall` (17.16%); `Construct` is negligible
(0.15%). So Construct-first maximizes the *RayTrace* win specifically; the
general call win (and most of Richards) belongs to a later inline-asm call path.

### Why `Construct` is ~100% slow and has no fast path

`Call0/1/2`, `TailCall`, and `Construct` are all **cold shims**
(`crates/vm/src/dsl/handlers/cold.rs`) — they always trampoline out of the asm
interpreter loop into a Rust handler. The profiler's "slow share" counts samples
that landed in the Rust handler, i.e. "exited asm to Rust," **not** "took the
slowest Rust sub-path." The eligible `Call*` opcodes already use a Vec-free
caller-register-window Rust fast path (lyng-4pvk), yet still read ~99% slow
because the trampoline + frame-setup cost is the floor a Rust fast path cannot
remove.

`Construct` is worse than `Call*`: it has **no Rust fast path at all**. Every
`new` in `construct_value` (`crates/vm/src/vm/call.rs:644`) pays, in order:

1. `std::mem::take(&mut self.argument_scratch)` + `collect_arguments_into` — a
   `Vec<Value>` materialization of the arguments.
2. `resolve_bound_construct_chain` — bound-chain walk.
3. `is_constructor` / `is_proxy_object` checks.
4. `create_construct_this` (`crates/vm/src/vm/bytecode_calls.rs:653`) — a full
   generic `get_property_from_object(new_target, "prototype")` on **every**
   construction, then an object allocation.
5. `enter_bytecode_call` — frame setup.

A `Construct` inline cache *already exists* but is **write-only scaffolding**:
`ConstructCacheEntry` (`crates/vm/src/vm/feedback.rs:194`) records `constructor`,
`constructor_shape`, `realm`, and `created_shape`; `observe_construct_target`
(`:496`) populates it; **nothing on the construct path ever reads it**, and it
does not cache the resolved prototype.

## Goal & Scope

Give `Construct` the Rust fast-path treatment `Call*` has, **plus** eliminate
the per-construction `.prototype` get via a watchpointed allocation profile.

**In scope (Stage 1):** a monomorphic Rust fast path inside `construct_value`
for ordinary bytecode constructors that (a) skips `argument_scratch`
materialization via caller-register-window copy, and (b) skips the `.prototype`
get entirely using a cached, watchpoint-guarded prototype.

**Explicitly out of scope** (each its own later effort): the Stage 2 inline-asm
call/construct path that removes the asm→Rust trampoline floor; pre-sizing the
created object from a cached `created_shape`; and fast-pathing bound / proxy /
derived-class constructors.

## Key Decisions (locked during brainstorming)

1. **Spec covers Stage 1 only** (Rust Construct fast path). The inline-asm call
   path is a sequenced follow-up.
2. **`.prototype` skip = watchpoint-based** (cache the resolved prototype value
   and do *no* read on the hit path), not a re-read-each-time slot load. Chosen
   for best steady-state cost.
3. **Invalidation granularity = per-constructor** (JSC FunctionRareData style),
   not per-shape over-invalidation — precise, and avoids permanently poisoning
   construct caching for all functions sharing a shape.

## Architecture

The fast path is a monomorphic branch at the top of `construct_value`, mirroring
how `call_value_small` branches to `call_value_small_bytecode_direct`. It splits
into the units below; each is independently testable.

### Component 1 — Construct eligibility gate

`ordinary_bytecode_construct_eligibility(agent, callee) -> Option<CodeRef>`,
mirroring `ordinary_bytecode_call_eligibility`
(`crates/vm/src/vm/call.rs:479`). Returns `None` (→ unchanged slow path) for:

- **bound** functions (would need `resolve_bound_construct_chain`),
- **proxies**,
- **derived class constructors** (`this` = TDZ + `super()` machinery),
- **generator / async** bodies (not constructible),
- callees needing an **`arguments` object** or a **rest parameter**,
- a **spread** argument list (`spread_mask.is_some()`).

`new_target == callee` always holds on the `Construct` opcode (`Reflect.construct`
routes through a builtin, not this opcode), so new-target divergence is out of
scope by construction.

### Component 2 — Consume the Construct IC

Add a monomorphic lookup of `ConstructCacheEntry` by feedback slot at the top of
`construct_value`, guarded on **constructor identity + `constructor_shape`**. On
a guard miss, or a polymorphic / megamorphic construct slot, take the existing
slow path (which continues to populate the IC via `observe_construct_target`).

### Component 3 — Watchpointed `.prototype` skip

- Extend `ConstructCacheEntry` (`crates/vm/src/vm/feedback.rs:194`) with the
  **resolved prototype** `ObjectRef`. "Resolved" folds in the
  realm-`Object.prototype` fallback that `create_construct_this` applies when a
  constructor's `.prototype` is not an object, so the cached value is exactly
  what the allocation needs.
- On an IC hit, allocate `construct_this` directly as
  `ObjectAllocation::ordinary(root_shape).with_prototype(cached_prototype)` —
  **no `get_property_from_object(new_target, "prototype")`**.
- The empty-object allocation itself is retained (it is not the measured cost);
  `created_shape`-based pre-sizing is a deliberate later optimization.

### Component 4 — Per-constructor `.prototype` invalidation

A function's `.prototype` is a writable own data property; reassigning it
(`F.prototype = X`) overwrites an existing own slot and **keeps the same shape**
(`crates/objects/src/shapes.rs:430`), so it fires **no** ShapeInvalidation
watchpoint. The existing `AdaptiveOwnWrite` watchpoint
(`crates/objects/src/watchpoint.rs`, `register_own_write_watchpoints` at
`crates/vm/src/vm/feedback.rs:922`) keys on shape invalidation and therefore
cannot catch this. A new, dedicated signal is required:

- **Per-constructor watchpoint set** (function rare-data style, keyed by
  constructor `ObjectRef`). The construct IC registers as an observer when it
  caches a prototype; firing clears only that constructor's construct IC.
- **Close the asm hole (load-bearing soundness step):** gate the
  AssignNamedProperty **inline-write handler install off for a function's
  `prototype` slot**, so every `F.prototype = …` write is forced through Rust.
  `.prototype` reassignment is rare, so the forced-slow write costs effectively
  nothing, and it guarantees the choke points below observe every write.
- **Fire** the watchpoint from the Rust write choke points — ordinary `[[Set]]`
  (`crates/objects/src/internal_methods.rs:282`) and `define_own_property`
  (`:175`) — when the target is a watched constructor's prototype slot.
- After firing, the next `new` re-runs the slow path, re-resolves `.prototype`,
  and re-caches with a fresh watchpoint. No permanent poisoning.

### Component 5 — Argument-window reuse

The fast path skips `argument_scratch` materialization by copying arguments
directly from the caller's register window, mirroring
`invoke_bytecode_call_from_caller_arg_window` (`crates/vm/src/vm/call.rs:547`)
and its `copy_within`-based `copy_arguments_from_caller_registers`
(`crates/vm/src/vm/bytecode_calls.rs:475`). This requires no spread (Component 1).

Frame entry: `install_prepared_bytecode_call_from_registers`
(`crates/vm/src/vm/bytecode_calls.rs:414`) must be extended to thread
`new_target` + `construct_this` + `construct_call`, matching what
`install_prepared_bytecode_call` already passes on the slow path
(`enter_bytecode_call` → `install_prepared_bytecode_call(... construct_this,
construct_call)`, `crates/vm/src/vm/bytecode_calls.rs:62`).

## Data flow (fast-path hit)

```
Construct op → construct_value
  ├─ eligibility(callee)            [Component 1]  miss → slow path
  ├─ IC lookup by feedback slot     [Component 2]  miss/poly/mega → slow path
  │     guard: constructor identity + constructor_shape
  ├─ construct_this = alloc(root_shape, with_prototype(cached))   [Component 3]
  │     (watchpoint guarantees cached == current F.prototype)     [Component 4]
  ├─ copy args caller-window → callee frame (copy_within)         [Component 5]
  └─ enter callee frame: this = construct_this, new_target = callee, construct=true
```

Write side (`F.prototype = X`):

```
write to F.prototype  → forced onto Rust path (inline-write IC gated off) [C4]
  → [[Set]] / define_own_property choke point detects watched ctor prototype slot
  → fire F's per-constructor watchpoint → clear F's construct IC          [C4]
```

## GC safety

The cached prototype `ObjectRef` is kept reachable by the watchpoint invariant
itself: while the IC entry is live, the constructor's `prototype` slot still
references the cached object (any reassignment fires the watchpoint and clears
the entry), so the prototype remains reachable through the constructor and is
traced normally. The cached ref is handled exactly like the already-cached
`constructor` ref in `ConstructCacheEntry`, consistent with existing IC practice
and the global-cell tenuring discipline
(`docs/superpowers/specs/2026-05-30-global-cell-asm-load-design.md`).

## Error handling, edge cases, fallbacks

Every case below falls back to the existing, unchanged slow path:

- guard miss (different constructor or shape), polymorphic / megamorphic slot;
- bound / proxy / derived-class / generator / async / arguments-object /
  rest-parameter / spread (rejected by Component 1);
- a constructor whose `.prototype` was reassigned (watchpoint fired → IC cleared
  → slow path re-resolves and re-caches);
- non-object `.prototype` resolves to the realm `Object.prototype`, and that
  resolved value is what gets cached.

`Reflect.construct` is unaffected (it does not go through the `Construct`
opcode).

## Testing

- **Test262 must remain 100%** (`built-ins`, `language`, `staging`, `annexB`).
- Invalidation fires and a subsequent `new` observes the new prototype, for each
  write path: plain `F.prototype = X` assignment; `Object.defineProperty(F,
  "prototype", …)`; and a **hot** `F.prototype = …` write site (asserting the
  inline-write gating kept it on the Rust path).
- Fallback correctness: proxy constructor, bound constructor, derived class
  `extends`, spread args (`new F(...xs)`), and a megamorphic construct site all
  produce spec-correct objects.
- Non-object `.prototype` (`F.prototype = 5`) → instance proto is
  `Object.prototype`.
- **Performance:** re-run `lyng-bench profile --filter RayTrace` to quantify the
  `Construct` time-share drop (target: out of the #1 slot), and `lyng-bench
  v8suite` for the Richards/RayTrace throughput delta.

## References

- Profile evidence: `reports/lyng/v8-raytrace-profile-2026-05-30.md`,
  `reports/lyng/profile-raytrace.md`.
- Construct slow path: `crates/vm/src/vm/call.rs:644` (`construct_value`),
  `crates/vm/src/vm/bytecode_calls.rs:653` (`create_construct_this`).
- Call fast-path template: `crates/vm/src/vm/call.rs:404`/`:479`/`:547`,
  `crates/vm/src/vm/bytecode_calls.rs:382`/`:414`/`:475`.
- Construct IC scaffolding: `crates/vm/src/vm/feedback.rs:194` /
  `:496` / `:2647`; `crates/vm/src/vm.rs:159` (`construct_ic_states`).
- Watchpoints: `crates/objects/src/watchpoint.rs`,
  `crates/vm/src/vm/feedback.rs:922` (`register_own_write_watchpoints`).
- Write choke points: `crates/objects/src/internal_methods.rs:282` (`set`),
  `:175` (`define_own_property`).
- Prior call-path effort (Construct deferral rationale):
  `reports/lyng/lyng-4pvk-status.md`.
