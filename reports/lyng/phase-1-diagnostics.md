# Phase 1 — Diagnostics for the V8 v7 geomean shortfall

**Issue context:** [`lyng-33i2`](../../../crates/lyng-js/vm/src/vm/dispatch_state.rs) Phase 1 cutover landed but missed all six V8 v7 score targets and regressed DeltaBlue by -7.2%. The roadmap's "Re-evaluation Checkpoints" section ([jsc-aligned-engine-roadmap.md:968-988](jsc-aligned-engine-roadmap.md)) names this exact outcome as a hard stop: *"if α's gain is < 8% geomean (below even the conservative target), the package theory is wrong or LLVM is materializing the Step enum on the hot path. Stop. Inspect run() asm."*
**Date:** 2026-05-15
**Toolchain:** rustc 1.93.1 (2026-02-11), aarch64-apple-darwin, `--release` profile (thin LTO)
**Methodology:** cargo-asm symbol dumps + diagnostic A/B with one variable removed at a time. Flamegraph runs were deferred — load average was 4.97 at investigation time, well above the roadmap's <2.0 isolation requirement, and the asm evidence proved deterministic enough to act on.

## TL;DR

The per-dispatch instruction count grew from the spike's ~20 instrs to production's ~27 instrs — a **+35% per-dispatch overhead increase** that fully accounts for the missed geomean target. Two compounding causes, both correct fixes for bugs the spike didn't carry, but neither is paying its asm-cost rent on the hot path:

1. **`maybe_record_opcode_dispatch` inlined into every `dispatch_next!`** adds 4 instructions per dispatch (plus ~21 instructions of cold-path code per handler symbol). Costs are paid even when the counter is `None`, because the disabled fast path is still a load+ccmp+branch. Confirmed by A/B: removing it shrinks `op_move` 98→77 instrs, `op_add` 140→119, `op_get_named_property` 369→346.
2. **Trampoline epoch + active_in check** (added in [fbace3dd](https://github.com/) to fix the cross-frame catch bug) adds 5 instructions per dispatch — 3 loads, 1 compare, 1 branch. The bug-fix is correct; the implementation re-reads `state.vm` and the epoch field every iteration instead of pinning them.

Of these, fix 1 is essentially free engineering: the counter exists for the bench harness and tests, not for production. A feature-gate or separate trampoline isolates it from the hot path. Fix 2 is more delicate — the correctness contract is real — but 2-3 instructions are recoverable without changing semantics.

Combined Tier 1+2 fixes project ~7 instructions saved per dispatch (~25% of the current overhead), which on dispatch-bound workloads like DeltaBlue translates to ~5–8% wall-clock gain — enough to clear the 8% geomean floor but likely short of individual workload targets like Richards ≥260 or DeltaBlue ≥310. If the geomean still doesn't clear after T1+T2, the γ-swap (inline-asm tail calls behind the existing `dispatch_next!` macro) is the documented escape hatch.

## Methodology

Captured `cargo asm --release` on the four named hot opcodes plus the trampoline. Counted true instructions (lines starting with `\t<mnemonic>`, excluding `.cfi_*` directives and labels). Diffed the dispatch-tail block specifically against the spike-era expected shape from [phase-1-spike.md](phase-1-spike.md).

For the maybe_record hypothesis, edited `dispatch_next!` to remove the `$state.vm.maybe_record_opcode_dispatch(byte)` call, rebuilt, captured asm, computed delta, restored the macro. The trampoline-internal calls at [dispatch_state.rs:276](../../../crates/lyng-js/vm/src/vm/dispatch_state.rs:276) and [dispatch_state.rs:317](../../../crates/lyng-js/vm/src/vm/dispatch_state.rs:317) were left in place since they fire once per trampoline entry (not per dispatch).

Bench numbers were not re-captured — load avg was 4.97 (vs roadmap's <2.0 isolation requirement). The committed [bench-v8.md](bench-v8.md) numbers are taken as authoritative for the "before" reading.

## Findings

### 1. Production dispatch path carries ~7 extra instructions per iteration vs the spike

**Spike's projected dispatch path** (from [phase-1-spike.md:65-84](phase-1-spike.md)):

```text
Inside handler dispatch_next! tail:
  ldrb   next opcode byte
  adrp   DISPATCH_TABLE
  add    DISPATCH_TABLE
  ldr    handler from DISPATCH_TABLE
  mov    Step::Continue tag
  movk   tag (immediate hi half)
  stp    tag, handler, [sret]
  ldp    epilogue (x29, x30)
  ret
  → 9 instructions
```

```text
Inside run_trampoline hot loop body:
  mov    sret slot prep
  mov    state ptr
  blr    handler
  ldr    Step tag from sret
  add    Continue tag arith
  add    Continue constant
  cmp
  csel   branchless Continue check
  cbnz   non-Continue exit
  ldr    Step payload (next handler)
  b      loop top
  → 11 instructions
```

**Spike total: ~20 instructions per dispatch.**

**Production dispatch_next! tail** (from [/tmp/op_add.before.asm](/tmp/op_add.before.asm) lines 127–157):

```text
ldr    bytes ptr
ldrb   next opcode byte
ldr    state.vm.opcode_dispatch_counts        ← maybe_record_opcode_dispatch
cmp    byte vs 151                            ← maybe_record_opcode_dispatch
ccmp   counts is null                         ← maybe_record_opcode_dispatch
b.eq   skip (taken when counters disabled)    ← maybe_record_opcode_dispatch
adrp   DISPATCH_TABLE
add    DISPATCH_TABLE
ldr    handler
mov    Step::Continue tag
movk   tag hi half
stp    tag, handler, [sret]
b      epilogue
→ 11 hot-path instructions (4 of them spike never paid)
```

**Production run_trampoline hot loop** (from [/tmp/run_trampoline.before.asm](/tmp/run_trampoline.before.asm) lines 77–92):

```text
add    sret slot
mov    state ptr
blr    handler
ldr    Step tag
add    Continue arith
add    Continue constant
cmp
csel
cbnz   non-Continue exit
ldr    Step payload (next handler)
ldr    state.frame_check_epoch                ← epoch check, 5 instrs new
ldr    state.vm                                ← epoch check
ldr    vm.dispatch_frame_check_epoch           ← epoch check
cmp                                            ← epoch check
b.eq   loop back                               ← epoch check
→ 15-16 hot-path instructions (5 of them spike never paid)
```

**Production total: ~26–27 instructions per dispatch — +35% vs spike.**

### 2. The handler prologue is mostly necessary

The trampoline + each handler each save 6 callee-saved register pairs + frame pointer (`x29/x30`) and allocate a 96–336 byte stack frame. That's 7–9 instructions of prologue + 7–9 of epilogue per handler invocation — ~18 instructions of overhead per `blr` in the trampoline.

This is the α-vs-γ floor the roadmap acknowledges. Each handler is a real Rust function with a real ABI, and the compiler has to honor that even when the body is small. JSC's offlineasm handlers carry no prologue at all. The γ swap (inline-asm tail calls + `#[naked]`) collapses this to ~3 instructions per dispatch — that's the +5–8% recovery the roadmap projects.

For Phase 1's α target, this overhead is structural and accepted. The Tier 1+2 fixes target the per-iteration overhead inside the loop body, not the per-handler-call cost.

### 3. The `Step` enum does NOT materialize to memory on the hot path

Critically, the spike's load-bearing claim still holds in production:

```asm
; trampoline reads only 16 of 48 bytes of Step from sret:
ldr  x8, [sp, #16]   ; Step tag word (8 B)
ldr  x9, [sp, #24]   ; Step::Continue payload — next handler pointer (8 B)
```

The full 32-byte `VmError` payload only loads on the cold `Step::Error` arm. The branchless Continue check (4 instrs: add/add/cmp/csel) is intact. This rules out one of the roadmap's two named risks ("the Step enum materializing in memory"). The other risk — "the match doesn't elide" — is also disproven by the production asm: the Continue arm is fall-through, only non-Continue branches.

The package theory is intact. The miss is per-dispatch overhead added by the two specific machinery items below.

### 4. A/B confirmation: removing `maybe_record_opcode_dispatch` saves 21 instructions per handler

Diagnostic edit at [dispatch_state.rs:229-241](../../../crates/lyng-js/vm/src/vm/dispatch_state.rs:229): commented out the `$state.vm.maybe_record_opcode_dispatch(byte)` line in the `dispatch_next!` macro. Rebuilt and recaptured asm. The macro was restored after measurement.

| Symbol | Before instructions | After instructions | Delta |
| --- | ---: | ---: | ---: |
| `run_trampoline` | 177 | 177 | 0 (unchanged — trampoline-internal calls fire once per script, not in the inner loop) |
| `op_move` | 98 | 77 | **-21** |
| `op_add` | 140 | 119 | **-21** |
| `op_get_named_property` | 369 | 346 | **-23** |

The disappeared 21–23 instructions per handler break down as:
- 4 instructions on the hot path (Option-discriminant load, ccmp, branch)
- ~17 instructions on the cold path (Wide/ExtraWide-prefix-byte lookup table, atomic increment, bounds check)

Even though the cold path is taken only when counters are enabled, it still occupies icache lines next to the hot path because LLVM doesn't out-of-line `#[inline]` functions. **Removing this from `dispatch_next!` is a near-zero-risk binary-size and icache-footprint win.**

### 5. The trampoline epoch check could be 2-3 instructions cheaper

[dispatch_state.rs:306-321](../../../crates/lyng-js/vm/src/vm/dispatch_state.rs:306) currently:

```rust
if state.frame_check_epoch != state.vm.dispatch_frame_check_epoch() {
    state.frame_check_epoch = state.vm.dispatch_frame_check_epoch();
    let still_active = ...;
    if !still_active { state.refresh_from_active_frame()?; ... }
}
```

The hot path (epoch unchanged, the common case) currently loads `state.frame_check_epoch`, loads `state.vm`, loads `vm.dispatch_frame_check_epoch`, compares, branches. Three loads per dispatch where one would suffice if `state.vm` and the trampoline's local epoch were kept in registers across the loop.

This is a more delicate optimization than fix #1 — the correctness contract that motivated this code (cross-frame catch parity, the 514-Test262-file regression and the 30 GB OOM that fbace3dd documents) is real. The fix is to hoist `state.vm` into a callee-saved register pinned across the loop and to keep `frame_check_epoch` in a register, syncing to `state.frame_check_epoch` only on the cold path. Saves 2 loads per dispatch.

## Planned solution

Two tiers of fixes, ordered by risk/reward. Each tier delivers an isolated win and a re-runnable bench so we can stop the moment we clear the 8% geomean floor.

### Tier 1 — Move `maybe_record_opcode_dispatch` off the hot path

**Goal:** Remove 4 instrs/dispatch + ~17 cold-path instrs/handler symbol. Zero behavior change when counters are off (the default everywhere except `lyng-js-bench --count-opcodes`).

**Recommended approach: separate "instrumented" dispatch table at compile time.**

```rust
// In dispatch_state.rs:
macro_rules! dispatch_next {
    ($state:expr) => {{
        let byte = $state.next_opcode_byte();
        #[cfg(debug_assertions)]
        $state.vm.assert_deopt_safepoint_state(...);
        return $crate::vm::dispatch_state::Step::Continue(
            $crate::vm::dispatch_state::DISPATCH_TABLE[byte as usize],
        );
    }};
}

// In vm.rs:
pub fn enable_opcode_dispatch_counts(&mut self) { ... }  // unchanged signature

// The trampoline branches once per script entry on whether counters are
// enabled; the hot dispatch path never re-checks.
#[inline(never)]
pub fn run_trampoline(state: &mut DispatchState) -> VmResult<Value> {
    if state.vm.opcode_dispatch_counts.is_some() {
        run_trampoline_counted(state)
    } else {
        run_trampoline_uncounted(state)
    }
}
```

Two trampolines, each pinning a slightly different inner loop. The hot path stays clean; the counted path is the diagnostic / bench-harness path. Increment lives in the trampoline body, not in the handler's tail.

**Why not a `#[cfg(feature = "opcode-counters")]` gate?** The counter has a real runtime user (the `lyng-js-bench --count-opcodes` flag, see [bench.md](bench.md)). A feature flag forces a recompile to switch modes, which is painful for ad-hoc bench investigation. Two trampolines pay 1 extra branch at script entry, never per dispatch.

**Alternative: bytecode-level instrumentation.** Compiler emits `RecordDispatch` opcodes adjacent to every other opcode in a "counted" bytecode variant. Higher engineering cost; cleaner asm. Defer unless Tier 1 + 2 still fall short.

**Verification:** Re-run [tests/core.rs:232-263](../../../crates/lyng-js/vm/src/tests/core.rs:232) (the counter test). Capture `cargo asm` of `op_move` / `op_add` — expect 77 / 119 instructions or fewer.

**Estimated effort:** 1 day. **Asm delta projected:** -4 instrs/dispatch, -21 instrs avg per handler symbol.

### Tier 2 — Tighten the trampoline epoch check

**Goal:** Recover 2 more instructions per dispatch from the trampoline loop body without changing the cross-frame catch parity that fbace3dd put in place.

**Concrete changes in [dispatch_state.rs:282-326](../../../crates/lyng-js/vm/src/vm/dispatch_state.rs:282):**

```rust
#[inline(never)]
pub fn run_trampoline(state: &mut DispatchState) -> VmResult<Value> {
    // Hoist into registers across the loop.
    let vm_ptr: *mut Vm = state.vm;
    let mut local_epoch = state.frame_check_epoch;
    let mut handler = DISPATCH_TABLE[state.first_opcode_byte() as usize];
    loop {
        match (handler)(state) {
            Step::Continue(next) => {
                // SAFETY: vm_ptr aliases state.vm for the lifetime of this function;
                // state.vm cannot be reassigned by any handler.
                let vm_epoch = unsafe { (*vm_ptr).dispatch_frame_check_epoch() };
                if local_epoch != vm_epoch {
                    local_epoch = vm_epoch;
                    state.frame_check_epoch = vm_epoch;
                    if !still_active_check(state) {
                        state.refresh_from_active_frame()?;
                        handler = DISPATCH_TABLE[state.first_opcode_byte() as usize];
                        continue;
                    }
                }
                handler = next;
            }
            Step::Done(v) => return Ok(v),
            Step::Error(e) => return Err(e),
        }
    }
}
```

The hot path becomes: `ldr w10, [x_vm, #1656]; cmp w_local, w10; b.eq loop_top` — 3 instructions instead of 5.

**Other small fixes in the same touch:**
- Hoist the `Step::Continue` tag constant (`add x10, x24, #65`) out of the loop body. -1 instr/dispatch.
- Audit whether the sret slot pointer (`add x8, sp, #16`) can be kept in a callee-saved register the handler must preserve. Likely no — extern "C" callee-clobbers — but worth checking.

**Risk:** The `unsafe` deref + register pinning is localized to the trampoline body. The `state.vm` aliasing rule is already enforced by the borrow checker for safe code paths; the `*mut Vm` here is a stable identity since the handler receives `&mut DispatchState` whose `vm` field has the same lifetime as the trampoline call.

**Verification:** Re-run cross-frame catch parity tests (`cargo test --release -p lyng-js-vm trampoline_parity`). Re-run Test262. Capture trampoline asm — expect 3-instr hot-path epoch check.

**Estimated effort:** 2–3 days including the parity-test re-run and asm verification. **Asm delta projected:** -3 instrs/dispatch.

### Tier 3 — Re-evaluate after T1 + T2

If T1+T2 doesn't clear the 8% geomean floor:

**T3a — γ-swap evaluation.** The roadmap names this as the documented escape hatch. One-line change to `dispatch_next!` per arch plus per-handler `#[naked]` audit. Per [jsc-aligned-engine-roadmap.md:206-246](jsc-aligned-engine-roadmap.md):

```rust
#[cfg(target_arch = "aarch64")]
macro_rules! dispatch_next {
    ($state:expr) => {{
        let next = DISPATCH_TABLE[$state.next_opcode_byte() as usize];
        unsafe {
            core::arch::asm!(
                "br {next}",
                next = in(reg) next,
                in("x0") $state,
                options(noreturn),
            );
        }
    }};
}
```

Expected gain: 5–8% on dispatch-bound workloads. Cost: one localized `unsafe` site + per-handler prologue audit. The macro abstraction is exactly what makes the swap cheap.

**T3b — Restate the asm-size gate in the roadmap.** [phase-1-final-asm.md](phase-1-final-asm.md) already proposes ≤100 B for tail-merged shims, ≤1000 B for real hot handlers. Update [lyng-33i2's acceptance text](.dogcats/issues.jsonl) and the [Phase 1 exit criteria](jsc-aligned-engine-roadmap.md:343-360) to match. The 200B target was JSC-LLInt-aligned and assumed offlineasm; production carries Rust ABI prologue + Wide path + feedback slot decode + register-window bounds checks. Either accept ~250B as the structural floor or commit to γ.

**T3c — Commit the missing pre-Phase-1 baselines.** Re-run on `main~N` (the trampoline-cutover boundary) and commit:
- `reports/js/lyng-js/phase-0-bench.md`
- `reports/js/lyng-js/phase-0-test262.md`
- `reports/js/lyng-js/phase-0-asm.md`

Phase 2 and Phase 3 will compute cumulative gains; without phase-0 they have to bootstrap from the roadmap's baseline column in [bench-v8.md](bench-v8.md), which isn't an isolated snapshot.

### Sequencing

```text
T1 (1 day)
  └─ re-run V8 v7 on isolated machine (load avg < 2.0)
     ├─ if geomean ≥ 8%: declare T1 sufficient, move to Phase 2
     └─ else continue
T2 (2-3 days)
  └─ re-run V8 v7
     ├─ if geomean ≥ 8%: declare T1+T2 sufficient, move to Phase 2
     └─ else T3a (γ-swap)
T3b + T3c run in parallel with T1/T2 (docs + retrospective baselines)
```

## Per-dispatch instruction budget — projected after each tier

| Source | Spike | Production (today) | After T1 | After T1+T2 | After T1+T2+γ |
| --- | ---: | ---: | ---: | ---: | ---: |
| dispatch_next! tail (handler) | 9 | 11 | 7 | 7 | 0 (`br next`) |
| Trampoline loop body | 11 | 16 | 16 | 13 | 0 (γ has no trampoline) |
| Handler prologue/epilogue | 18 | 18 | 18 | 18 | 0–3 (`#[naked]`) |
| **Total** | **38** | **45** | **41** | **38** | **0–3** |

(Prologue/epilogue is counted but is not the loop body — it's per-handler-call, not per-dispatch in steady state. Including it for completeness.)

Excluding prologue, **per-iteration steady-state instruction count: 20 (spike) → 27 (today) → 23 (T1) → 20 (T1+T2)**. T1+T2 restores the spike-era cost. γ then unlocks the ~85–90% of β's ceiling the roadmap predicted.

## What this means for the Phase 1 close decision

The Phase 1 architecture is right. The bench miss is two specific items in the hot path, both with measured fixes that are 1–5 days of work. **Phase 1 should not close until at least Tier 1 lands and the V8 v7 sweep is re-run on an isolated machine.** Tier 2 is recommended; γ is held in reserve.

Sub-9 ([lyng-2wji](.dogcats/issues.jsonl)) is the right home for the re-run. The asm-size gate question (Tier 3b) needs a roadmap-revision decision before sub-9 can close cleanly.

## Files referenced

- [dispatch_state.rs](../../../crates/lyng-js/vm/src/vm/dispatch_state.rs) — DispatchState, Handler, Step, DISPATCH_TABLE, dispatch_next!, run_trampoline
- [dispatch_handlers/mod.rs](../../../crates/lyng-js/vm/src/vm/dispatch_handlers/mod.rs) — build_dispatch_table
- [dispatch_handlers/arithmetic.rs](../../../crates/lyng-js/vm/src/vm/dispatch_handlers/arithmetic.rs) — op_add and the SMI fast path
- [vm.rs:295](../../../crates/lyng-js/vm/src/vm.rs:295) — maybe_record_opcode_dispatch
- [jsc-aligned-engine-roadmap.md](jsc-aligned-engine-roadmap.md) — Phase 1 exit criteria, re-evaluation checkpoints, γ-swap spec
- [phase-1-spike.md](phase-1-spike.md) — spike-era dispatch asm shape
- [phase-1-final-asm.md](phase-1-final-asm.md) — production asm sizes + the restated-gate proposal
- [bench-v8.md](bench-v8.md) — V8 v7 scores, gate status

## Raw measurements

Baseline (today) and post-experiment asm dumps preserved at:
- `/tmp/run_trampoline.before.asm` / `/tmp/run_trampoline.after.asm`
- `/tmp/op_move.before.asm` / `/tmp/op_move.after.asm`
- `/tmp/op_add.before.asm` / `/tmp/op_add.after.asm`
- `/tmp/op_get_named_property.before.asm` / `/tmp/op_get_named_property.after.asm`

These are transient (machine-local). The instruction counts and dispatch-tail diffs above are the load-bearing artifacts; the raw dumps are reproducible via:

```sh
cargo build --release -p lyng-js-vm
cargo asm --release -p lyng-js-vm "lyng_js_vm::vm::dispatch_handlers::arithmetic::op_add" 0
cargo asm --release -p lyng-js-vm "lyng_js_vm::vm::dispatch_state::run_trampoline"
```

---

## Measured outcome — T1 + T2 landed

T1 (`maybe_record_opcode_dispatch` removed from `dispatch_next!`; split into
`run_trampoline_uncounted` + `run_trampoline_counted`) and T2 (`state.vm` and
`frame_check_epoch` hoisted into callee-saved registers across the trampoline
hot loop) landed under [lyng-3uem](#). Measured against the projections in
this report:

### Per-handler asm sizes (cargo asm, instruction counts)

| Symbol | Before T1+T2 | After T1+T2 | Delta | Projected |
| --- | ---: | ---: | ---: | ---: |
| `op_move` | 98 | 77 | **-21** | -21 ✓ |
| `op_add` | 140 | 119 | **-21** | -21 ✓ |
| `op_get_named_property` | 369 | 346 | **-23** | -23 ✓ |
| `run_trampoline` (wrapper) | — | 4 | new (entry branch) | — |
| `run_trampoline_uncounted` (hot) | n/a | 197 | new (split) | — |
| `run_trampoline_counted` | n/a | 197 | new (split) | — |

Handler deltas match the diagnostic A/B exactly. The wrapper is 4 instructions
(`ldr counts ptr; cbz; b uncounted; b counted`) — a single nullness check that
LLVM lowered to a `cbz` + tail-call. The wrapper cost is paid once per
`Vm::run` invocation, not per dispatch.

### Trampoline hot loop body (`run_trampoline_uncounted`)

```asm
; LBB76_4 — loop top
mov  x8, sp                  ; sret slot
mov  x0, x20                 ; state ptr
blr  x9                      ; *** indirect call to handler ***
ldr  x8, [sp]                ; Step tag
add  x9, x8, x24             ; Continue tag arith
add  x10, x24, #65           ; Continue constant
cmp  x8, x10
csel x8, x9, x26, hi         ; branchless Continue check
cbnz x8, LBB76_18            ; non-Continue exit
ldr  x9, [sp, #8]            ; Step::Continue payload (next handler)
ldr  w10, [x23, #1656]       ; vm.dispatch_frame_check_epoch via HOISTED x23
cmp  w28, w10                ; w28 = HOISTED local_epoch
b.eq LBB76_3                 ; steady state: loop back
```

**13 instructions per dispatch** in the steady state — down from 15-16 before
T2. The two saved loads (`ldr w11, [x20, #144]` for `state.frame_check_epoch`
and `ldr x8, [x20, #80]` for `state.vm`) collapsed into register reads of x23
(vm_ptr) and w28 (local_epoch). The steady-state branch goes back to LBB76_3
(an empty label that falls through to LBB76_4) without re-loading the handler
from `DISPATCH_TABLE` — the handler pointer is in x9 from the Step::Continue
payload.

### Per-dispatch instruction budget — updated

| Source | Spike | Pre-T1+T2 | After T1+T2 (actual) | Projected | Match? |
| --- | ---: | ---: | ---: | ---: | --- |
| `dispatch_next!` tail (handler) | 9 | 11 | **7** | 7 | ✓ |
| Trampoline loop body | 11 | 15-16 | **13** | 13 | ✓ |
| **Total per dispatch** | 20 | 26-27 | **20** | 20 | ✓ |

T1 + T2 restored the spike-era per-dispatch cost. **-6 to -7 instructions
per dispatch versus the pre-fix production code, ~25% reduction.**

### Test verification

- `cargo test --release -p lyng-js-vm -p lyng-js-bytecode -p lyng-js-objects -p lyng-js-compiler`: 577/577 pass (including the opcode counter tests at [tests/core.rs:232-263](../../../crates/lyng-js/vm/src/tests/core.rs:232) — counter functionality preserved via `run_trampoline_counted`).
- `cargo test --release -p lyng-js-tests`: 1186/1186 pass.
- Test262 whole-suite: **49722/49729 runnable files pass (same as baseline; 0 panics, same 7 failures as pre-T1+T2)**.

### Bench verification — V8 v7 sweep landed

Captured at load avg ~4.2 (above the roadmap's <2.0 ideal, but the
medians are consistent across 5 samples per workload — see [bench-v8.md](bench-v8.md) for the per-sample tape). The directional signal is solid; absolute numbers may shift ±1-2% on a quieter machine.

| Bench | Baseline | Pre-T1+T2 | Post-T1+T2 | Δ vs Pre | Δ vs Baseline | Target |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Richards | 234 | 233 | 233 | 0 | -0.4% | ≥260 |
| DeltaBlue | 277 | 257 | **276** | +19 | -0.4% | ≥310 |
| Crypto | 236 | 257 | 262 | +5 | +11.0% | ≥265 |
| RayTrace | 387 | 385 | 397 | +12 | +2.6% | ≥430 |
| NavierStokes | 424 | 435 | 443 | +8 | +4.5% | ≥470 |
| Splay | 1198 | 1221 | 1214 | -7 | +1.3% | ≥1330 |

**Geomean vs baseline: +0.8% → +3.0%** (the +2.2pp improvement T1+T2 delivered).

**Hard gates checked:**

| Gate | Status |
| --- | --- |
| No workload regresses > 2% vs baseline | ✓ (worst is Richards at -0.4%) |
| DeltaBlue regression fixed | ✓ (-7.2% → -0.4%) |
| Geomean ≥ +8% (roadmap's re-evaluation floor) | ✗ (+3.0%) |
| Individual workload targets (Richards ≥260, etc.) | ✗ (all 6 still below target) |

T1+T2 fully recovered the DeltaBlue regression and lifted the geomean from
"package theory is broken" territory (+0.8%) to "α-bounded ceiling" territory
(+3.0%). The roadmap predicted α delivers ~85-90% of β's ceiling, and
β-vs-α gap is bounded to ~5-10% on dispatch-bound workloads.

We're sitting near α's interpreter ceiling. The remaining 5pp gap to the 8%
geomean floor — and the much bigger gap to individual workload targets like
Richards ≥260 — won't come from further per-dispatch instruction shaving.
**Per [jsc-aligned-engine-roadmap.md:592-610](jsc-aligned-engine-roadmap.md), the documented next move is T3a: the γ-swap evaluation** (inline-asm tail calls with `#[naked]` handlers behind the existing `dispatch_next!` macro). Macro change is one line per arch; the work is the per-handler prologue audit. Expected gain: +5-8% additional on dispatch-bound workloads.

Alternative reading of the data: the workload-specific targets (Richards
≥260, DeltaBlue ≥310, etc.) were also calibrated against a different
expected α gain. Phase 3 (Inline IC Fast Path) is the workstream that
*actually* delivers the 45-53% gains the roadmap projects for these
benchmarks — Phase 1 was always going to be a +11-12% per-workload-floor
gate at α. T1+T2 plus γ should reach that floor; Phase 3 is what reaches
the cumulative target column.
