# Lyng-JS: JSC-Aligned Engine Roadmap

**Date:** 2026-05-14
**Supersedes (in spirit):** `vm-dispatch-modernization-plan.md` — that plan
targeted QuickJS parity and prescribed dispatch-loop tweaks. This roadmap
targets a higher ceiling (JSC LLInt + Baseline JIT) and a different
architectural shape.

## Goal

Reach **JSC LLInt-class interpreter performance** (~3–5× past QuickJS on
Octane-style workloads) followed by **JSC Baseline-class JIT performance**
(~10–15× past QuickJS). The route is the JSC playbook: a metadata-driven
inline-IC interpreter with threaded dispatch as the substrate, then a
template-style Baseline JIT consuming the same bytecode and IC state.

This roadmap explicitly rejects two prior framings:

1. **"Match QuickJS"** (the modernization-plan goal). The ceiling is too
   low for the work involved. JSC LLInt is achievable on the same
   substrate.
2. **"Adopt V8 Ignition shape piecemeal"** (Track H's pattern). V8's
   design is a coherent package that requires CSA-generated native stubs
   to amortize per-callsite IC overhead. Without the stubs the overhead
   is naked cost. JSC achieves Ignition-class perf through handwritten-asm
   handlers, which has safe-Rust analogues that V8's emitter-based
   approach does not.

## What's Already Aligned With JSC

Substantial parts of lyng-js already match JSC's data design. This
roadmap is finishing the alignment, not starting from scratch:

- **NaN-boxed `Value`** ([crates/lyng-js/types/src/value.rs:73-345](../../../crates/lyng-js/types/src/value.rs)).
  Matches `JSCJSValue.h`.
- **Per-callsite FeedbackVector with Monomorphic/Polymorphic/Megamorphic
  state machine** ([crates/lyng-js/vm/src/vm/feedback.rs:770-797](../../../crates/lyng-js/vm/src/vm/feedback.rs)).
  Matches JSC's `StructureStubInfo` / `InlineCacheHandler`.
- **Shape transition tree with inline + out-of-object slots**
  ([crates/lyng-js/objects/src/object_metadata.rs:333-397](../../../crates/lyng-js/objects/src/object_metadata.rs)).
  Matches `Structure.h`.
- **32-bit ShapeId** (already u32 by `define_runtime_id!`). Conceptually
  matches JSC's compressed `StructureID`.
- **Always-allocate slot per IC-shaped opcode** (Track H, just landed).
  Matches JSC's `op_get_by_id` metadata-table model.

## What Doesn't Match — The Roadmap's Workstreams

The pieces blocking JSC-class perf:

1. **Dispatch infrastructure.** JSC handlers end with their own
   `nextInstruction()` macro — per-handler indirect-branch site, with
   register-pinned ABI across handler boundaries. Lyng-js uses a flat
   Rust `match` with the central indirect branch.

2. **Cell access has one extra indirection.** `ObjectRef = u32` →
   `heap.object(id) -> &ObjectRecord`. JSC reads `[m_structureID]`
   directly from the cell pointer.

3. **IC fast path is a 4-deep function call chain.** JSC's `op_get_by_id`
   hit path is 5–7 inline asm operations. Ours is `try_named_property_load_inline_cache_hit`
   → `try_load` → `load_from_named_property_cache` →
   `named_property_cache_entry_valid`.

4. **Compiler-side argument marshaling.** Pre-Track-H, ~27–50% of
   dispatches were `Move` opcodes from argument shuffling. Real Track C
   (direct argument lowering) was deferred.

5. **No JIT.** `TierStatus::ReadyForNative` is dead scaffolding.

The phases below tackle these in dependency order.

## Guiding Principles

These should hold across every phase.

- **Profile before each phase, profile after each phase.** Use `sample`
  on Richards, Crypto, and one property-heavy workload. Capture
  `cargo asm` of `run_dispatch_loop` (or its successor) before/after.
  Track H's regression was caught late because the first measurement
  was contaminated by concurrent CPU work; insist on isolated
  measurement.

- **Each phase must deliver a measurable interpreter win on its own.**
  No more "the package will pay off." If a phase doesn't move
  benchmarks, the package theory is wrong and we adjust.

- **Test262 + V8 v7 sweeps are the verification floor**, not the
  ceiling. Pre-existing failure count must not grow; per-workload
  regression > 2% triggers a hold-and-investigate.

- **Subtractive changes preferred where possible.** Removing code
  (auto-allocator hacks, redundant function calls, dead scaffolding) is
  faster to land and easier to verify than additive structural change.

- **No half-finished phases on `main`.** Each phase is a complete
  workstream landed as a series of commits + status report. Mid-phase
  state goes on a branch.

## Open Decision Before Phase 1: `unsafe` / Nightly Policy

JSC's interpreter design exists because **offlineasm gives you proper
threaded dispatch with register pinning**. Safe stable Rust does not
offer a clean equivalent. The choices, ranked by perf upside × invasiveness:

**Option α — Stable safe Rust only.** Use a central trampoline that
dispatches to `extern "C" fn` handlers via a function-pointer table.
Each handler is its own function (independent register allocation,
independent inlining decisions, doesn't pollute neighbor handlers'
icache). One indirect branch per dispatch, same as today, but the
handlers are no longer one giant function. *Conservative; estimated
~10–15% gain from per-function optimization alone.*

**Option β — Nightly `become` (tail calls).** Each handler ends with
`become handler[next_opcode](state)`. Per-handler tail-call dispatch,
matching JSC's structural pattern exactly. Per-handler indirect-branch
site gives the ~2× branch-prediction win. *The actual JSC LLInt
shape. Requires accepting nightly Rust for the dispatch core.*

**Option γ — Localized `unsafe` for raw pointers and/or inline asm.**
Drop `ObjectRef = u32` indirection. Optionally use `asm!` for
register-pinned dispatch. *Maximum perf but largest cultural change.*

The roadmap is **written assuming Option β** — nightly `become` —
because that is what cashes out JSC's interpreter design in Rust. If
the team rejects nightly, fall back to Option α and accept the smaller
ceiling. Option γ should be revisited in Phase 2 (cell access) where
it's most contained.

This decision blocks Phase 1. Make it before starting.

---

## Phase 1 — Threaded Dispatch + Per-Handler ABI

**Workstream:** convert the central `match opcode` dispatch into per-handler
functions with tail-call dispatch. This is the structural foundation;
every other phase assumes it.

**Files:**
- `crates/lyng-js/vm/src/vm/dispatch.rs` — restructure top-level loop.
- `crates/lyng-js/vm/src/vm/dispatch_handlers/` — new module hierarchy,
  one file per opcode family (`arithmetic.rs`, `property.rs`, `calls.rs`,
  `control_flow.rs`, etc.). Each opcode is an `extern "C" fn` handler.
- `crates/lyng-js/vm/src/vm/dispatch_state.rs` — new struct carrying
  the pinned-register state (`pc`, `bytecode_base`, `register_stack_top`,
  `frame_ptr`, `vm: &mut Vm`). Same role as JSC's `cfr` / `t0`.

**Shape per handler (Option β):**

```rust
extern "C" fn op_add(state: &mut DispatchState) -> DispatchOutcome {
    // operand decode inline (no helper call)
    let bytes = &state.code()[state.pc..];
    let a = u16::from(bytes[1]);
    let b = u16::from(bytes[2]);
    let c = u16::from(bytes[3]);
    let slot = u16::from_le_bytes([bytes[4], bytes[5]]);

    // SMI fast path inline
    let left = state.read_register(b);
    let right = state.read_register(c);
    if let (Some(l), Some(r)) = (left.as_smi(), right.as_smi()) {
        if let Some(v) = l.checked_add(r) {
            state.record_feedback_smi(slot);
            state.write_register(a, Value::from_smi(v));
            state.advance(6);
            // Tail-call dispatch to next handler.
            let next = state.next_opcode_byte();
            become DISPATCH_TABLE[next as usize](state);
        }
    }

    // Cold path — function call OK here
    op_add_slow(state, a, b, c, slot)
}
```

**Shape per handler (Option α fallback):**

```rust
extern "C" fn op_add(state: &mut DispatchState) -> Option<extern "C" fn(&mut DispatchState)> {
    // ... same body ...
    let next = state.next_opcode_byte();
    Some(DISPATCH_TABLE[next as usize])
}

// Central trampoline:
fn run(state: &mut DispatchState) -> VmResult<Value> {
    let mut handler: extern "C" fn(&mut DispatchState) = DISPATCH_TABLE[state.first_opcode() as usize];
    while let Some(next) = (handler)(state) {
        handler = next;
    }
    state.take_result()
}
```

**Verification:**

- `cargo build --release -p lyng-js-cli` — green.
- `cargo test -p lyng-js-vm -p lyng-js-bytecode -p lyng-js-objects -p lyng-js-tests -p lyng-js-compiler` — full pass.
- `cargo asm` on each opcode function (`op_add`, `op_move`, `op_get_named_property`, `op_call_0`):
  - Body is one function in the symbol table per handler.
  - For Option β: each handler ends with `br x<N>` directly into the
    dispatch table (a tail-call). One indirect branch per dispatch.
  - For Option α: each handler returns a function pointer; one
    indirect call in the central trampoline.
- Per-handler binary size: hot opcodes like `op_add` should be < 200
  bytes; cold ones can be larger. Track total dispatch code size; the
  goal is for the *hot subset* to fit in L1i.

**Benchmarks:** 5-sample median via `lyng-js-bench compare`, in
isolation (no concurrent build or Test262):

| Bench | Pre-phase target | Post-phase target |
| --- | ---: | ---: |
| Richards | 234 | ≥ 280 (+20%) |
| DeltaBlue | 277 | ≥ 330 (+19%) |
| Crypto | 236 | ≥ 280 (+19%) |
| RayTrace | 387 | ≥ 460 (+19%) |
| NavierStokes | 424 | ≥ 500 (+18%) |
| Splay | 1198 | ≥ 1400 (+17%) |

**Exit criteria:**

- Per-handler dispatch shape verified by `cargo asm` on every IC-shaped
  opcode + Move/Jump/Load.
- Benchmark targets hit or within noise (within 5% on either side); no
  workload regresses > 2%.
- Test262 baseline preserved (49722/49729 or better).
- Per-handler structural regression tests added that fail if any opcode
  reverts to flat-`match` shape.

**Risks:**

- **Nightly `become` rejection.** If Option β is rejected, fall to
  Option α. Expected gain drops from +20% to +10–15%.
- **Code size explosion** — 152 per-handler functions vs one flat
  dispatch. Cold handlers should be `#[cold]`. If hot-handler icache
  pressure measurably increases over the current shape, consider
  splitting non-IC arms into a separate "common" handler that does
  generic operand decode and dispatches further.
- **Refactor blast radius** — `dispatch.rs` is 3300 lines today; the
  rewrite touches every line. Plan for 2-3 weeks of focused work and a
  rebase against any concurrent main changes.

**Estimated effort:** 2–3 weeks for one engineer.

---

## Phase 2 — Cell Access Flattening

**Workstream:** eliminate the second indirection in property access.
Today: `Value → ObjectRef(u32) → heap.object(id) → &ObjectRecord →
.shape() / .inline_slot()`. Target: `Value → cell_ptr → shape / slot`
in 1–2 loads.

Two sub-options ordered by safety:

**Phase 2a — Safe single-load handle.** Make `heap.object(id)` resolve
via `base + id * stride` in a single load, no `Vec::get` / `Option`
overhead. Replace `Vec<ObjectRecord>` with `Box<[ObjectRecord]>` of fixed
or grow-with-known-base layout. Object records become same-size for the
fast path (variable-size payloads moved to side tables).

- `crates/lyng-js/gc/src/object_records.rs` (or wherever the pool
  lives) — replace `Vec<ObjectRecord>` with a slab allocator.
- `crates/lyng-js/objects/src/internal_methods/property_cache.rs:113-280`
  — rewrite so `heap.object(holder_id)` is a single load (no
  `Option` on the hot path).

Expected gain: **2–4%** on property-heavy workloads (RayTrace, Splay,
DeltaBlue).

**Phase 2b — Pointer-identity cells (gated on `unsafe` decision).**
Replace `ObjectRef(u32)` with `*mut ObjectHeader` packed into the
NaN-boxed `Value`. Drop the side-table entirely for non-moving GC. If
the GC is or becomes moving/compacting, use a remembered set / read
barrier strategy.

This is the P0-1 / Option C from the state-of-the-engine report. It's
gated on accepting localized `unsafe` for the deref boundary.
Estimated additional gain over Phase 2a: **3–7%** on property-heavy
workloads.

**Phase 2 ordering:** Land 2a unconditionally. Land 2b if and only if
profiles after Phase 3 still show `heap.object(id)` as a hot
single-load (vs being free / cached). 2b might turn out unnecessary.

**Verification:**

- `cargo asm` on `op_get_named_property` shows one load from cell
  before the shape compare, not two.
- Test262 + V8 v7 sweep — no regressions.
- Spot profile on Splay should show reduced `heap.object` /
  `object_header` time.

**Risks:**

- Heap-pool restructure affects GC. Coordinate with `lyng-js-gc`.
- Variable-size object records currently exist; the slab needs a fixed
  fast-path size with overflow handling.

**Estimated effort:** 1–2 weeks for 2a; 3–4 weeks for 2b (gated).

---

## Phase 3 — Inline IC Fast Path

**Workstream:** collapse the 4-deep function call chain in IC dispatch
into a flat block inside each IC-shaped opcode handler. This is what
makes Track H's always-allocate machinery actually pay back.

**Today's chain** (`crates/lyng-js/vm/src/vm/dispatch/property.rs:68-127`
→ `feedback.rs:1859` → `feedback.rs:779` →
`property_cache.rs:113-140` → `property_cache.rs:228-280`):

```text
op_get_named_property handler
  → try_named_property_load_inline_cache_hit (vector lookup + execution_count++)
  → try_load (state check + find entry by shape)
  → load_from_named_property_cache (validity recheck + slot decode)
  → named_property_cache_entry_valid (re-fetch object header + re-compare shape)
```

Five function calls, two redundant shape compares, one execution-count
mutation, one polymorphic-entry binary search.

**Target shape** (matching JSC `op_get_by_id`):

```rust
extern "C" fn op_get_named_property(state: &mut DispatchState) -> DispatchOutcome {
    let (target_reg, recv_reg, atom_const, slot) = decode_with_slot(state);
    let receiver = state.read_register(recv_reg);

    if let Some(cell) = receiver.as_cell() {
        let cell_shape = cell.shape_id();              // 1 load
        let entry = state.feedback_entry(slot);        // 1 load (sidecar)
        if entry.cached_shape == cell_shape {          // monomorphic hit
            let offset = entry.slot_offset;            // load
            let value = if entry.is_inline_slot {
                cell.inline_slot(offset)               // 1 load
            } else {
                cell.out_of_line_slots()[offset]       // 1 indirect + 1 load
            };
            state.write_register(target_reg, value);
            state.advance(slot_instruction_len);
            become DISPATCH_TABLE[state.next_opcode_byte() as usize](state);
        }
    }

    // Polymorphic / megamorphic / miss path → call out
    op_get_named_property_slow(state, target_reg, recv_reg, atom_const, slot)
}
```

Hot-path total: ~5–7 loads + 1 compare + 1 branch + write + dispatch.
Matches JSC's `performGetByIDHelper`.

**Concrete refactor:**

1. **Compact the IC handler representation.** Pack `(cached_shape,
   slot_offset, is_inline, is_double)` into a single 64-bit word (or a
   `repr(packed)` 8-byte struct). Match V8 `LoadHandler` bit-field
   pattern. Currently `NamedPropertyCacheEntry` is multi-field; the
   bit-packed form should be the fast-path lookup target.

2. **Inline the IC check into each property-load arm.** Delete the
   four-call chain on the hit path. Slow path remains a function call.

3. **Drop the redundant second shape compare.** `named_property_cache_entry_valid`
   re-reads the object header and re-compares the shape; the only thing
   that can change between `try_load` and `load_from_named_property_cache`
   is the prototype chain validity (relevant for `PrototypeData`
   entries). Move that check to the slow path or to a watchpoint /
   invalidation event.

4. **Apply to all 10 IC-shaped property/global opcodes**, plus the
   inverse path for stores (`SetNamedProperty` / `StoreGlobal` /
   `AssignNamedProperty` / strict variants).

5. **Same treatment for keyed property access** (`GetKeyedProperty` /
   `SetKeyedProperty`), where the IC additionally caches the index→atom
   mapping.

**Files:**
- All `op_*` handlers introduced in Phase 1's `dispatch_handlers/property.rs`.
- `crates/lyng-js/vm/src/vm/feedback.rs` — collapse the
  `try_load`/`load_from_named_property_cache`/`entry_valid` chain.
- `crates/lyng-js/objects/src/internal_methods/property_cache.rs` —
  rewrite slot decode as a `const fn` that takes the bit-packed
  handler.

**Verification:**

- `cargo asm` on `op_get_named_property` shows the hit-path as a
  straight-line sequence of loads + one compare + one branch — no `bl`
  / call instructions on the hit path.
- Profile shows `try_named_property_load_inline_cache_hit` and
  `try_load` either inlined fully (no entry in the symbol table) or at
  most 1 entry each (the slow path).
- Crypto regression from Track H should fully recover and then some
  (Phase 3 is what makes the bookkeeping cash out).

**Benchmarks (post-Phase-3, cumulative from Phases 1+2+3):**

| Bench | Phase 0 (today) | Phase 3 target |
| --- | ---: | ---: |
| Richards | 234 | ≥ 380 (+62%) |
| DeltaBlue | 277 | ≥ 460 (+66%) |
| Crypto | 236 | ≥ 400 (+69%) |
| RayTrace | 387 | ≥ 640 (+65%) |
| NavierStokes | 424 | ≥ 700 (+65%) |
| Splay | 1198 | ≥ 1800 (+50%) |

These targets put the interpreter at roughly 2× past QuickJS — the JSC
LLInt-only territory.

**Exit criteria:**

- All 10 IC-shaped opcodes have the flat hit-path verified in `cargo
  asm`.
- Crypto recovers to or beyond pre-Track-H level (≥ 260).
- Geomean gain over Phase 0 is ≥ 40%.
- Test262 baseline preserved.

**Risks:**

- **Polymorphic cache compaction** — bit-packed handler must
  accommodate the polymorphic case (multiple shapes per slot) or
  polymorphic falls to the slow path. JSC's `PolymorphicAccess`
  handler is more complex than the monomorphic case; we may need a
  layered structure where monomorphic is bit-packed inline and
  polymorphic is a pointer to a sibling array.
- **Watchpoint integration** — currently `named_property_cache_entry_valid`
  re-checks dependencies inline. Moving the dependency check to a
  watchpoint event requires a watchpoint system; if absent, the
  validity recheck stays on the hit path (cheaper than today but not
  free).

**Estimated effort:** 3–4 weeks.

---

## Phase 4 — Compiler / Bytecode Polish

**Workstream:** finish the deferred compiler-side work that's been
deferred for two rounds, plus a small set of structural improvements
that pair well with the new dispatch shape.

### 4a — Direct argument lowering (real Track C)

`crates/lyng-js/compiler/src/script/calls.rs` `materialize_argument_block` +
`crates/lyng-js/compiler/src/script/expr.rs` `lower_call_target`.

Compiler currently emits `LoadX Rtemp; Move Rdst, Rtemp` chains for
each call argument. ~27–50% of dispatches were Move opcodes
(`vm-dispatch-fixup-status.md`). The fix is to reserve the call-arg
register block first, then `lower_expr_into(arg, target_slot)` directly
into final slots.

Estimated gain: **+8–12%** by reducing Move dispatches by ~half.

### 4b — Star fusion lookahead

V8 Ignition pattern (`src/interpreter/interpreter-assembler.cc:1324-1380`):
when a handler that produces a value is followed by `StarN` in the
bytecode, the handler does the Star's write inline before dispatching
to the next-next opcode. Removes one dispatch per fused pair.

In threaded dispatch this becomes a per-handler peephole: at the end
of each value-producing handler, check if the next opcode byte is
`Star0..7`, write the value to the corresponding register if so, and
advance the PC past the Star before dispatching.

Estimated gain: **+3–5%** on benchmark code with many expression
statements.

### 4c — Compact accumulator-based bytecode

Audit which opcodes have an obvious accumulator-based variant. Today
we have `Ldar` / `Star0..7` / `LdaSmi8` etc. but the compiler uses
them inconsistently. Audit + bias compiler emission toward accumulator
forms where lifetime analysis allows.

Estimated gain: **+2–3%** on bytecode size / icache footprint.

**Files:**
- `crates/lyng-js/compiler/src/script/calls.rs`
- `crates/lyng-js/compiler/src/script/expr.rs`
- `crates/lyng-js/vm/src/vm/dispatch_handlers/*.rs` (Star fusion)
- `crates/lyng-js/compiler/src/script/script.rs` (Star/Ldar emission
  audit)

**Verification:**

- Opcode dispatch counts (via `--count-opcodes`) — Move share drops
  from 27–50% to < 20%.
- V8 v7 sweep — cumulative gain over Phase 3.

**Exit criteria:**

- Move share < 20% on Richards.
- Cumulative geomean over Phase 0 ≥ 60%.

**Estimated effort:** 2–3 weeks.

---

## Phase 5 — JIT Prerequisites

**Workstream:** lay groundwork for Baseline JIT. No native codegen yet;
this is plumbing.

### 5a — Cranelift integration

Add Cranelift as a dependency (already in the Rust ecosystem, used by
Wasmtime, well-supported). Wire up a tiny smoke test that compiles
`fn() -> i32 { 42 }` via Cranelift and runs it.

`crates/lyng-js-jit` (new crate) — backend interface.

### 5b — Tier-up counters

The `TierStatus` enum exists in `crates/lyng-js/vm/src/tiering.rs` but
is dead. Reanimate:

- Increment a per-function execution counter at each `LoopHeader` and
  function entry (similar to JSC's `op_loop_hint` / function prologue).
- When the counter crosses a threshold, transition `TierStatus` to
  `ReadyForNative` and queue the function for JIT compilation.
- Counter increment: branchless `addi` in the loop-header handler.

Files: `crates/lyng-js/vm/src/vm/dispatch_handlers/control_flow.rs`,
`crates/lyng-js/vm/src/tiering.rs`.

### 5c — JIT calling convention

Decide the ABI for JIT'd code. Recommended: the JIT'd version of a
function has the same signature as a fully-inlined interpreter run —
takes the receiver register, the argument range, a `&mut Agent`. The
JIT'd code can call back into interpreter helpers (slow paths) via the
same `extern "C"` ABI as interpreter handlers.

Document this in `docs/lyng-js/jit-abi.md`.

### 5d — Deopt / fallback path

JIT'd code must be able to fall back to the interpreter on
unanticipated cases (e.g., IC state goes megamorphic, watchpoint
fires). Define the fallback: JIT'd code calls a designated
`deopt_to_interpreter` runtime function with the current state; the
interpreter resumes at the appropriate bytecode offset.

This is the simplest possible deopt — no on-stack-replacement back to
JIT, no complex state reconstruction. Just bail out and let the
interpreter take over.

**Verification:**

- Smoke test: a manually-emitted "compiled" function (e.g., a Cranelift
  IR for one trivial bytecode) runs end-to-end and returns the right
  value.
- Tier-up counter test: hot function transitions to
  `ReadyForNative` after expected iteration count.

**Exit criteria:**

- Cranelift dependency green.
- Tier-up wiring tested end-to-end with a hand-rolled stub.
- JIT ABI documented.

**Estimated effort:** 3–4 weeks.

---

## Phase 6 — Baseline JIT

**Workstream:** the actual native codegen. One template per bytecode
opcode. Each template emits the native-code equivalent of the
interpreter handler.

### Codegen shape

For each opcode, the JIT emitter walks the bytecode and emits a
Cranelift IR block that does what the interpreter handler does, but
inlined:

```text
// For `op_get_named_property` in JIT'd code:
load   v0, [receiver_reg]            ; load receiver
test   v0, NOT_CELL_MASK             ; check it's a cell
brnz   v0, slow_path                  ; fall to slow path if not
load   v1, [v0]                       ; load cell.shape_id
load   v2, [feedback_vector + slot * 16]  ; load cached_shape
icmp   v1, v2                         ; compare
brne   slow_path                      ; miss
load   v3, [feedback_vector + slot * 16 + 8]  ; load handler word
// branch on handler bits → inline-slot vs out-of-line-slot
...
```

This is the same shape as the inline IC from Phase 3 — except emitted
as native code per call site, so there's no dispatch overhead per
opcode.

### Workstream sub-phases

**6a — Trivial opcodes first.** `Move`, `Load*`, `Jump`, `Return`. No
ICs, no calls. Smoke test that the JIT works end-to-end on toy
functions.

**6b — Arithmetic.** `Add`, `Sub`, `Mul`, `BitAnd`, etc. Per-op SMI
fast path, generic fall-back. This unlocks Crypto and the arithmetic
side of Richards.

**6c — Property access.** `GetNamedProperty`, `SetNamedProperty`, etc.
Inline the IC fast path. This is the biggest win — the bookkeeping
Track H paid for finally cashes out.

**6d — Calls.** `Call0..3`, `Call`, `TailCall`, `Construct`. Inline
the IC check on the callee shape; inline argument passing.

**6e — Generators, async, exceptions.** The long tail. Don't optimize
— just emit calls into the interpreter helpers for these. They're rare.

### Verification

After each sub-phase:

- The JIT'd function passes the same tests as the interpreter version.
- V8 v7 sweep — measure the cumulative gain. The JIT should give
  another 2–3× over the interpreter for arithmetic-heavy workloads,
  per V8 Sparkplug numbers.

**Cumulative benchmark targets (Phase 6 complete):**

| Bench | Phase 0 (today) | Phase 6 target |
| --- | ---: | ---: |
| Richards | 234 | ≥ 900 (~4×) |
| DeltaBlue | 277 | ≥ 1000 |
| Crypto | 236 | ≥ 1000 (~4×) |
| RayTrace | 387 | ≥ 1400 |
| NavierStokes | 424 | ≥ 1500 |
| Splay | 1198 | ≥ 2500 |

These targets put lyng-js into JSC-Baseline / V8-Sparkplug territory:
roughly half of full V8, several times QuickJS.

**Exit criteria:**

- Every IC-shaped opcode has a JIT template that emits the inline IC.
- Tier-up + deopt round-trip tested.
- V8 v7 sweep shows ~3–5× cumulative gain over Phase 0.
- Test262 baseline preserved.

**Estimated effort:** 3–4 months.

---

## Phase 7 (Optional, Future) — Speculative Tier

If JSC Baseline-class isn't enough — i.e., if there's pressure to
compete with V8 directly on web workloads — the next tier is JSC DFG /
V8 Maglev: speculative optimization with type inference, OSR, and
deoptimization.

This is genuinely a different project. Estimated 6–12 months on top of
Phase 6. Ceiling: approaching full V8 / full JSC.

The roadmap leaves Phase 7 unspecified intentionally. Don't plan it
until Phase 6 ships and you can measure where the remaining cost is.

---

## Verification Methodology (Across All Phases)

For each phase the same evidence is captured. Reports go in
`reports/js/lyng-js/`.

### Bench harness setup

Use `lyng-js-bench compare` with `--samples 5 --warmup-samples 2` per
workload, run individually (not full-suite — the harness's full-suite
mode currently produces one row).

**Critical:** run in isolation. No concurrent Test262, no concurrent
builds. Check `uptime` before starting (load avg < 2.0 ideally). If
the machine is shared, schedule bench runs at off-hours.

### `cargo asm` evidence

For each new phase's hot opcodes:

```sh
cargo asm --release 'lyng_js_vm::vm::dispatch_handlers::arithmetic::op_add'
cargo asm --release 'lyng_js_vm::vm::dispatch_handlers::property::op_get_named_property'
```

Capture before/after into `reports/js/lyng-js/phase-N-asm.md`.

### Test262

```sh
cargo run --release -p lyng-js-test262 -- --report /tmp/t262-phase-N.md -j 4
```

Pass count must not decrease vs. the phase-entry baseline. Cluster
deltas are documented in the phase status report.

### Structural regression tests

Each phase adds source-level structural assertions to
`crates/lyng-js/vm/src/vm/dispatch.rs` (or its successor) that fail
the build if the structural property the phase delivered gets
accidentally reverted. Examples:

- Phase 1: assert `dispatch.rs` does not contain a `match
  semantic_opcode` over more than 10 arms (catches accidental
  flat-match restoration).
- Phase 3: assert `op_get_named_property` source does not contain
  `try_named_property_load_inline_cache_hit` (catches IC chain
  reintroduction).

---

## Risks Across the Roadmap

- **Nightly `become` dependency.** Phase 1's biggest decision. Without
  it, ceiling drops from JSC-LLInt-class to a softer target.
- **Cranelift integration cost.** Cranelift is well-supported but
  non-trivial. Budget 1–2 weeks at the start of Phase 5 for plumbing
  alone.
- **Heap-pool restructure (Phase 2).** Affects GC. If the GC team has
  parallel work, coordinate explicitly.
- **Watchpoint system absence.** Phase 3 assumes a watchpoint /
  invalidation mechanism for prototype-chain dependencies. If absent,
  the IC fast path can't fully match JSC; the validity recheck stays
  in the hit path (cheaper but not free).
- **Each phase's gain estimate is calibrated against the cumulative
  package.** If Phase 1 underperforms, downstream phases may
  underperform proportionally. The estimates are not independent.

## Timeline (One Engineer, Sequential)

| Phase | Duration | Cumulative |
| --- | ---: | ---: |
| 1: Threaded dispatch | 2–3 weeks | 3 weeks |
| 2a: Cell flatten (safe) | 1–2 weeks | 5 weeks |
| 3: Inline IC | 3–4 weeks | 9 weeks |
| 4: Compiler / Star fusion | 2–3 weeks | 12 weeks |
| 5: JIT prerequisites | 3–4 weeks | 16 weeks |
| 6: Baseline JIT | 3–4 months | 28 weeks |

**Interpreter-only milestone (Phases 1-4):** ~3 months. Delivers
roughly 2× past QuickJS — JSC LLInt-class on the interpreter alone.

**Full Baseline JIT milestone (Phases 1-6):** ~7 months. Delivers
roughly JSC Baseline / V8 Sparkplug-class — several times QuickJS,
half of full V8.

With more engineers, Phase 2a/2b and parts of Phase 4 parallelize with
Phase 1. Phase 6 sub-phases (6b through 6d) parallelize once 6a is
green.

## Re-evaluation Checkpoints

After **Phase 1**: if the threaded-dispatch gain is < 10%, the
package theory is wrong. Stop. Reassess.

After **Phase 3**: if Crypto isn't recovered to at least pre-Track-H
levels, the inline IC isn't paying off in our substrate. Consider
rolling back Track H + Phase 3 and going QuickJS-shaped (drop
per-callsite ICs entirely).

After **Phase 6 sub-phase 6c**: if JIT'd property access isn't
showing measurable wins over the interpreted hot path, the IC handler
encoding is wrong or the dispatch isn't getting out of the way.
Investigate before continuing.

These are honest off-ramps, not just optimistic checkpoints.

---

## Pre-Work Before Phase 1

1. Resolve the `unsafe` / nightly decision (Option α / β / γ).
2. Lock the verification methodology in CI (isolated bench harness,
   automated `cargo asm` capture, Test262 gate).
3. Snapshot Phase-0 evidence: full V8 v7 sweep, full Test262, `cargo
   asm` of current `run_dispatch_loop`. This is the baseline every
   phase will be measured against.

Once those three are done, Phase 1 is unblocked.
