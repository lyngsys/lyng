# State of the Engine: LLInt-parity Reality Check and New Roadmap

**Date:** 2026-05-16
**Author:** post-Phase-3f / 4b retrospective
**Supersedes (in spirit):** parts of [jsc-aligned-engine-roadmap.md](jsc-aligned-engine-roadmap.md)
  — the master roadmap (`lyng-49qk`) stays, but its α-substrate assumptions
  and Phase 1/Phase 3 acceptance criteria need to be rewritten in light of
  the measurements below.

---

## TL;DR

We are **not close to LLInt**. Against JSC `--useJIT=false` on the V8 v7
suite, lyng-js is **5–12× slower per workload**. Against QuickJS (the
target the roadmap explicitly rejected as "ceiling too low") we are
**1.6–3.7× slower**. The roadmap targeted *past QuickJS* by Phase 3 and
*near JSC LLInt* by end of Phase 4. We finished Phase 3f and Phase 4b
and we are below both targets by a wide margin.

The cause is not "we haven't finished the package yet." The cause is
that the Option α dispatch substrate has a **fixed structural overhead of
~15× the LLInt dispatch substrate** that no amount of IC layering can
amortize. Phase 1's own re-evaluation gate (the +8% geomean floor)
was violated at landing. The roadmap's "Stop. Inspect run() asm" rule
was not honored; we continued into Phases 2, 3, and 4.

The fix is two changes of kind, not degree:

1. **Honest measurement infrastructure first.** Until we can produce a
   reproducible cargo-asm-vs-LLInt-asm diff in CI, every performance
   claim is a guess.
2. **Substrate decision based on measured ceilings,** not "α projected
   to deliver 85–90% of β." We need to measure α, γ (inline-asm tail
   call + `#[naked]`), and pure-asm handlers against the same workloads
   and pick what the data says.

The rest of this document is the receipts.

---

## 1. The numbers

### 1.1 V8 v7 (release, isolated subprocess, single hardware, three samples)

Source: [`reports/js/lyng-js/external-engine-compare.md`](external-engine-compare.md)
(committed 2026-05-15, post-Phase-3f, post-Phase-4b).

| Workload      | lyng-js |  QuickJS | JSC LLInt | lyng vs QuickJS | lyng vs JSC LLInt |
| ------------- | ------: | -------: | --------: | --------------: | ----------------: |
| Richards      |     318 |      917 |      1871 |  **2.88× slow** |    **5.88× slow** |
| DeltaBlue     |     360 |     1022 |      1684 |  **2.84× slow** |    **4.68× slow** |
| Crypto        |     269 |      802 |      2119 |  **2.98× slow** |    **7.88× slow** |
| RayTrace      |     427 |     1005 |      4547 |  **2.35× slow** |   **10.65× slow** |
| EarleyBoyer   |     519 |     1917 |      6084 |  **3.69× slow** |   **11.72× slow** |
| RegExp        |     101 |      263 |      1071 |  **2.60× slow** |   **10.60× slow** |
| Splay         |    1372 |     2200 |      9893 |  **1.60× slow** |    **7.21× slow** |
| NavierStokes  |     448 |     1325 |      2931 |  **2.96× slow** |    **6.54× slow** |

Roadmap target after Phase 3 (α-bounded, from
[jsc-aligned-engine-roadmap.md:605-614](jsc-aligned-engine-roadmap.md)):
Richards ≥ 340, DeltaBlue ≥ 400, Crypto ≥ 360, RayTrace ≥ 560,
NavierStokes ≥ 610, Splay ≥ 1650 — "1.7–2× past QuickJS." We hit none
of these. Richards (the canonical OO-call workload) is at 34% of QuickJS,
not 1.7× past it. Phase 4b numbers are roughly the same range.

### 1.2 Phase progression (Richards as the reference workload)

| Milestone                    | Score | Note                                       |
| ---------------------------- | ----: | ------------------------------------------ |
| Phase 0 baseline              |   234 | Pre-roadmap                                 |
| Phase 1 (post-T1+T2 hot fix) |   233 | -0.4% vs baseline                          |
| Phase 2a / 3a                |   ~250–260 | Inline IC fast-path landed (Phase 3a)      |
| Phase 3e (one-hop proto)     |   318 | +36% vs baseline                            |
| Phase 3f (poly compaction)   |   318 | Equal to 3e on Richards                    |
| Phase 4a / 4b (compiler+star)|   318 | Per `external-engine-compare.md`           |
| **Current (post-Phase-4b)**   | **318** | **+36% vs baseline; 34.7% of QuickJS**     |
| Roadmap Phase 3 target       |   340 | α-bounded                                  |
| Roadmap Phase 4 target       |   374 | "near JSC LLInt"                           |
| **JSC LLInt actual**          | **1871** | **5.88× ahead of us**                       |

Phase 3 alone was targeted at +45% on Richards (234 → 340). We landed +36%
across Phases 1 + 2a + 3a + 3e + 3f + 4a + 4b combined. The cumulative
gain comes from **multiple phases**, not the one that was supposed to
deliver it.

### 1.3 Per-handler asm sizes

Source: [`reports/js/lyng-js/phase-1-final-asm.md`](phase-1-final-asm.md)
and [`reports/js/lyng-js/phase-3f-op_get_named_property.asm`](phase-3f-op_get_named_property.asm).

| Handler                  | Lyng-js (bytes / lines) | JSC LLInt baseline                | Ratio  |
| ------------------------ | ----------------------: | --------------------------------- | -----: |
| `op_move`                |               384 bytes | ~20–30 bytes (no prologue)        |    ~14× |
| `op_add`                  |               548 bytes | ~50 bytes (SMI fast path inline)  |    ~11× |
| `op_get_named_property`   |             1080 bytes / **788 lines asm**  | ~80 bytes (incl. dispatch tail)   |    ~13× |
| `op_set_named_property`   |             ~3281 lines asm                | ~150 bytes                         |     —   |
| `op_get_keyed_property`   |             ~1422 lines asm                | ~120 bytes                         |     —   |
| `run_trampoline_uncounted` (LLInt has none)        | 197 instrs / ~800 bytes | 0 (tail-jumped, no loop)           |    —    |

Phase 1's exit gate said "real hot handlers under 200 bytes," matching
JSC LLInt sizes ([phase-1-spike.md](phase-1-spike.md) §3). Phase 1's
final-asm report explicitly downgraded the gate to "under 1000 bytes"
([phase-1-final-asm.md:99-106](phase-1-final-asm.md)) without
re-evaluating the architecture choice. We then landed Phase 1.

### 1.4 Per-dispatch instruction count

Source: [`phase-1-diagnostics.md`](phase-1-diagnostics.md) §1 plus
inspection of [`phase-3f-op_get_named_property.asm`](phase-3f-op_get_named_property.asm).

| Component                                       | LLInt | Lyng-js (today) |
| ----------------------------------------------- | ----: | --------------: |
| Handler prologue (callee-saves, stack frame)    |     0 |          ~9     |
| Handler epilogue (restore, ret)                 |     0 |          ~9     |
| `CALL` / `RET` round-trip                        |     0 |          ~2     |
| `Step` enum materialization + match in trampoline |   0 |          ~13    |
| `dispatch_next!` tail (table lookup + return)    |     0 |          ~7     |
| Actual dispatch (`nextInstruction`: 3 instrs)    |     3 |          ~3     |
| **Per-dispatch substrate total**                  | **3** | **~38–43**     |

That's a **~13× substrate overhead per dispatch**. Not per workload,
per opcode dispatched. Richards runs ~3–4 million opcode dispatches per
benchmark iteration; ~40 extra instructions per dispatch is ~150 million
extra instructions per iteration, on a workload whose total instruction
count in LLInt is maybe ~250M. The bench score gap is approximately
proportional to the substrate-cost ratio.

### 1.5 Bytecode density

Source: [`bytecode-density-aarch64.md`](bytecode-density-aarch64.md).

| Workload                          | Unit bytes lyng-js | LLInt-equivalent (estimate) | Ratio |
| --------------------------------- | -----------------: | --------------------------: | ----: |
| `script.core.objects-and-arrays`  |                244 |                        ~90  |  ~2.7× |
| `functions.closure-calls`         |                184 |                        ~70  |  ~2.6× |
| `activation.arguments-rest-for-in`|                527 |                       ~200  |  ~2.6× |

Our base instruction width is 4 bytes (1 opcode byte + 3 operand bytes).
LLInt uses 1+N narrow encoding where N is the number of operands × 1
byte each. For ABC opcodes we are similar, but IC opcodes carry a
mandatory 2-byte feedback slot inline; LLInt carries the slot via a
metadata-ID byte + side table. The density difference here is **less
critical than the dispatch difference** but compounds it: more PC
advancement, more L1i pressure, more dispatches per JS statement.

---

## 2. Where the roadmap went wrong

The roadmap had a section called "Re-evaluation Checkpoints"
([jsc-aligned-engine-roadmap.md:1062-1083](jsc-aligned-engine-roadmap.md))
that said, verbatim:

> After **Phase 1**: if α's gain is < 8% geomean (below even the
> conservative target), the package theory is wrong or LLVM is
> materializing the `Step` enum on the hot path. Stop. Inspect `run()`
> asm. If the trampoline is visibly the cost, try the γ swap before
> scaling further work. If γ doesn't recover either, the per-handler
> function model itself is wrong — rethink.

Phase 1 post-T1+T2 measured **+3.0% geomean** vs baseline
([phase-1-diagnostics.md:430-447](phase-1-diagnostics.md)). The roadmap
said "stop" at < 8%. We continued. Failures, in order:

### 2.1 Phase 1's gates were redefined to fit reality, not the other way around

- The < 200-byte-per-handler target was renamed "real hot handlers
  under 1000 B." `op_get_named_property` came in at 1080 B and was
  declared "right at the boundary"
  ([phase-1-final-asm.md:106](phase-1-final-asm.md)).
- The +11-12% per-workload floor was relaxed in writing
  ("workload-specific targets were also calibrated against a different
  expected α gain… Phase 3 is what *actually* delivers the 45–53%
  gains," [`phase-1-diagnostics.md:459-464`](phase-1-diagnostics.md)).
- The +8% geomean floor was not enforced. Phase 2 was started.

### 2.2 The α-vs-β projection was speculation, presented as a fact

The roadmap says α delivers "**85–90% of β's interpreter ceiling**"
([jsc-aligned-engine-roadmap.md:137-141](jsc-aligned-engine-roadmap.md)).
This claim was never measured. We have no β measurement (it requires
nightly Rust + `become`). We have no γ measurement (it requires the
inline-asm tail-call dispatch_next! swap and a per-handler `#[naked]`
audit). We assumed α was close to β; we did not check.

The actual evidence we have: α delivers ~3% geomean over Phase 0. JSC
LLInt — which is essentially what β / γ approximate — is ~6× ahead of
us. If "α is 85–90% of β" were correct, α should be in the same
order-of-magnitude as JSC LLInt. It is not.

The honest reading: **the α-vs-LLInt gap is much larger than 5–10%.**
The roadmap's "α-bounded targets" were calibrated against the wrong
ceiling.

### 2.3 The "inline IC fast path" was never actually inlined

Phase 3 was the centerpiece of the roadmap — the workstream that was
supposed to deliver +45–53% per workload. Phase 3a–3f all landed. The
status reports say "all helpers fully inlined into each dispatch
handler" ([phase-3f-status.md:60-76](phase-3f-status.md)).

What actually happened: the handler in
[`dispatch_handlers/property.rs:31-67`](../../../crates/lyng-js/vm/src/vm/dispatch_handlers/property.rs)
delegates to `Vm::execute_get_named_property_opcode`, a separate
function with its own ABI prologue/epilogue
([`dispatch/property.rs:70-200`](../../../crates/lyng-js/vm/src/vm/dispatch/property.rs)).
The IC fast paths are inside *that* function, with this shape:

```rust
let value = if let Some(object) = receiver.as_object_ref() {
    if let Some((handler, cached_epoch)) =
        self.named_property_fast_handler(frame.code(), feedback_slot)
    {
        // monomorphic OwnData inline check (Phase 3a)
        if record.shape() == handler.receiver_shape() && ...
    }
    if let Some(value) = self.try_named_property_polymorphic_fast_load(...) {
        // polymorphic OwnData inline check (Phase 3f), POLY_LIMIT = 2
    }
    if let Some(value) = self.try_named_property_proto_fast_load(...) {
        // one-hop PrototypeData inline check (Phase 3e)
    }
    if let Some(value) = self.try_named_property_load_inline_cache_hit(...) {
        // original 4-deep slow chain
    }
    // ... full slow path
}
```

This is not the JSC LLInt shape. JSC's `performGetByIDHelper`
([LowLevelInterpreter64.asm:1634-1677](../../../../WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter64.asm))
loads a single **mode byte** from metadata and dispatches to one of
four mode-specific straight-line blocks (`Default` /
`ProtoLoad` / `ArrayLength` / `Unset`). Each block is ~5–7 instructions.
The branch is on the mode byte, not on "did mono fail → try poly → try
proto → try slow chain."

Our shape is "stack four checks in source order; each runs in turn until
one hits." On polymorphic-with-prototype workloads (DeltaBlue) the
miss-path through the first two checks costs as much as the eventual
hit-path. On megamorphic workloads (EarleyBoyer, Splay) all four
checks miss and we fall to the original 4-deep chain we were supposedly
replacing.

The 788-line asm of `op_get_named_property` is what this layering looks
like with LLVM's best effort: every block is inlined but every block
still runs.

### 2.4 Each phase added complexity without removing the prior phase's overhead

- Phase 1 added the trampoline + `Step` enum.
- Phase 3a added a packed-handler fast path inside the IC chain.
- Phase 3e added a parallel proto-fast handler word + its check block.
- Phase 3f added a `[NamedPropertyHandler; 2]` sidecar + its check block.
- Phase 4b added `dispatch_next_with_value!` Star-fusion (a runtime
  branch on every value-producing handler).

Each was measured in isolation and showed a small win
([phase-3f-status.md:25](phase-3f-status.md): Phase 3f gave +4% on
DeltaBlue, equal Richards, +7.5% on Splay). The total of all these wins
is the +36% Richards bump from Phase 0 — but the rolled-up cost in code
size, icache pressure, and miss-path traversal is the 788-line
`op_get_named_property` handler. We added paths without retiring old
ones.

### 2.5 No actual asm-vs-LLInt diff was ever produced

The roadmap mentions JSC's `performGetByIDHelper` and links to the asm
file. The phase reports cite "matches JSC's metadata shape" and "Phase
3a inline IC fast path… single packed-handler load." No phase committed
a side-by-side asm comparison: "here is LLInt's
`performGetByIDHelper` Default mode, here is our `op_get_named_property`
hit path, here are the corresponding instructions in each, here is what
we still pay that they don't."

Without that diff, "we match JSC's shape" is a structural claim, not a
measured claim. Phase 3 status reports say *the helpers are fully
inlined* but the asm shows *the function has 788 lines of asm and a
384-byte stack frame*. Both can be true: the helpers inlined into the
caller; the caller is huge.

### 2.6 Phase 0 baseline asm was never committed

The roadmap's pre-work checklist
([jsc-aligned-engine-roadmap.md:1099-1105](jsc-aligned-engine-roadmap.md))
required:

> Snapshot Phase-0 evidence:
> - Full V8 v7 sweep, isolated…
> - Full Test262 run…
> - cargo asm of current `run_dispatch_loop` (all 4 monomorphs). Commit
>   to `reports/js/lyng-js/phase-0-asm.md`.

`phase-0-asm.md` does not exist in `reports/js/lyng-js/`. We compared
Phase 1 asm to a spike-era projection (which was itself missing several
load-bearing features), not to a real Phase 0 baseline.

### 2.7 Benchmark conditions were known-contaminated and we shipped anyway

Phase 1 diagnostics
([phase-1-diagnostics.md:5-6](phase-1-diagnostics.md)) explicitly states:

> Flamegraph runs were deferred — load average was 4.97 at investigation
> time, well above the roadmap's <2.0 isolation requirement.

The roadmap calls isolated measurement "the verification floor"
([jsc-aligned-engine-roadmap.md:103-107](jsc-aligned-engine-roadmap.md)).
We measured Phase 1 / T1+T2 / each Phase 3 sub-phase on a machine
loaded ~2× beyond the floor, then made architectural decisions based on
those numbers.

---

## 3. What LLInt actually is (the missing reference architecture)

This section is the side-by-side that should have been in the roadmap.

### 3.1 LLInt dispatch substrate (3 instructions)

From [`LowLevelInterpreter.asm:481-485`](../../../../WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter.asm):

```asm
macro nextInstruction()
    loadb [PB, PC, 1], t0                                ; load next opcode byte
    leap _os_script_config_storage, t1                   ; load opcode-map base
    jmp JSC::LLInt::OpcodeConfig::opcodeMap[t1, t0, PtrSize]  ; tail-jump
end
```

Three instructions. **No CALL.** **No prologue/epilogue.** Each handler
is a label in one giant function (the offlineasm-generated
`LowLevelInterpreter64.S`). PB is the program-bytes base, PC the
instruction offset; both live in pinned callee-saved registers across
the entire interpreter. `metadataTable`, `cfr` (call-frame register),
and the value-profile pointer are also pinned. The interpreter does
not save/restore them per dispatch because there is no per-dispatch
function boundary.

### 3.2 LLInt op_get_by_id Default-mode hit path (~17 instructions, including dispatch)

From [`LowLevelInterpreter64.asm:1679-1693`](../../../../WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter64.asm)
+ [`:1634-1677`](../../../../WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter64.asm):

```asm
; get(m_base, t0) — read base virtual-register operand byte
loadb 1[PB, PC, 1], t0

; loadConstantOrVariableCell(size, t0, t3, .slow)
bpgteq t0, FirstConstantRegisterIndexNarrow, .constant
loadq [cfr, t0, 8], t3                                   ; load receiver Value
btqnz t3, notCellMask, .opGetByIdSlow                    ; reject non-cell

; metadata(t2, t1) — compute &metadata[m_metadataID]
loadh op_get_by_id_metadataOffset[metadataTable], t2
getu 4[PB, PC, 1], t1                                    ; m_metadataID
muli sizeof OpGetById::Metadata, t1
addi t1, t2
addp metadataTable, t2

; performGetByIDHelper(...) Default mode
loadb metadata.mode[t2], t1
bbneq t1, Default, .opGetByIdProtoLoad
loadi JSCell::m_structureID[t3], t1                      ; receiver shape
loadi metadata.defaultMode.structureID[t2], t0           ; cached shape
bineq t0, t1, .opGetByIdSlow                             ; shape miss → slow
loadis metadata.defaultMode.cachedOffset[t2], t1         ; cached slot offset
loadPropertyAtVariableOffset(t1, t3, t0)                 ; ~2 instr: load slot

; valueProfile() + return(t0)
storeq t0, valueProfile.buckets[metadataTable, t2, ...]  ; record observation
get(m_dst, t1)                                            ; read dst operand
storeq t0, [cfr, t1, 8]                                  ; write to register

; dispatch() → nextInstruction()
addp op_get_by_id_length, PC                             ; advance PC
loadb [PB, PC, 1], t0                                    ; next opcode byte
leap _os_script_config_storage, t1
jmp opcodeMap[t1, t0, PtrSize]                           ; tail-jump
```

That's ~17 instructions of straight-line code on the hit path. Compare
to our 788-line `op_get_named_property` asm.

### 3.3 The four key LLInt design properties

| Property                          | LLInt                          | Lyng-js α today                |
| --------------------------------- | ------------------------------ | ------------------------------ |
| Per-dispatch function boundary    | **None** (label, tail-jump)    | extern "C" function call + ret |
| Pinned register state             | PB/PC/cfr/metadataTable in CSR | DispatchState struct on stack  |
| Per-handler prologue/epilogue     | **None**                       | ~18 instrs per dispatch        |
| IC dispatch shape                 | One mode-byte branch + N flat blocks | N nested if-let-Some checks    |
| Bytecode-side IC payload          | `metadataID` (1 byte) → table  | inline 2-byte feedback slot    |
| Slow-path return semantics        | `callSlowPath; dispatch()`     | `Step::Error / Step::Continue` |

We have parity on data-design (NaN-boxed Value, shape transition tree,
metadata-driven IC) and on bytecode width. We **do not** have parity on
the four properties that actually control per-dispatch cost. Phase 1's
α decision committed us against three of them; the fourth was the IC
shape, and Phase 3 layered four parallel paths instead of one mode-byte
branch.

---

## 4. The substrate decision is the rate-limiting step

Per-dispatch cost (steady state):

```
JSC LLInt:        3 instrs/dispatch  (just nextInstruction)
Lyng-js α today: ~38 instrs/dispatch (substrate) + ~10–30 instrs/handler-body
                   ~ 13× LLInt substrate
Lyng-js γ (asm tail-call, projected): ~10 instrs/dispatch + ~10 instrs/handler-body
                   ~ 4× LLInt substrate
Lyng-js γ + #[naked] (no prologue, projected): ~5 instrs/dispatch + ~10 instrs/handler-body
                   ~ 2× LLInt substrate
```

These are projections from the spike's asm shape, not measurements. We
should measure them.

The roadmap's α-only commitment was load-bearing on the claim that
α-vs-β was a 5–10% gap. If that claim were correct, α at +3% over
baseline would mean β at ~+8%, and γ (≈β) at ~+8–13%. That doesn't get
us to LLInt either; LLInt is ~6× ahead, not 13% ahead. So even on the
roadmap's own assumed numbers, **α cannot reach LLInt**. We did not
acknowledge this when we shipped Phase 1.

If we want LLInt parity, the substrate has to change. The options are:

1. **γ-swap (inline-asm tail-call dispatch_next!).** One macro change
   per arch. Reduces substrate to ~10 instrs/dispatch. Recovers maybe
   2–3× of the LLInt gap. Still 4× behind LLInt's substrate. Tier 3a
   in the original Phase 1 diagnostics. Not blocked.
2. **γ + `#[naked]` handlers.** Per-handler prologue/epilogue
   elimination. Stable Rust as of 1.88. Recovers ~2× more.
3. **`become` (β) on nightly.** Same shape as γ minus the `unsafe`
   block. Requires nightly. The user has explicitly rejected nightly,
   so this is out unless that rejection is revisited.
4. **Asm-language handlers (the LLInt approach).** Maximum freedom,
   maximum write/maintenance cost.
5. **Accept QuickJS-class.** Stop targeting LLInt. Re-baseline the
   roadmap to "consistently past QuickJS" — which is achievable in α.

Until we have asm-level measurements of #1 and #2 on real opcodes (not
spike micro-benches), we cannot pick honestly between them.

---

## 5. The new roadmap

This roadmap is **measurement-first**. Each phase starts by predicting an
asm shape and a benchmark delta; it ships only if both are measured to
hold. If a phase doesn't deliver its prediction, we **stop and
investigate**, not continue.

### Phase R-0: Tooling and baselines (the missing prerequisite)

**Goal:** make claims about performance falsifiable by anyone running
`make perf-baseline`. No engineering work landed in R-0 ever changes the
VM; it only changes what we can see.

**Deliverables:**

- **R-0.1 Isolated bench harness.** `lyng-js-bench compare` already
  exists; harden it so it refuses to run when the 1-min load average
  > 2.0 (currently the requirement is documented but not enforced —
  see `phase-1-diagnostics.md:5`). Add a `--require-isolation` flag
  used by CI.
- **R-0.2 Phase 0-equivalent baseline snapshot.**
  - V8 v7 + density + Test262 captured on a quiesced machine.
  - cargo asm of the 12 hottest opcodes (per the opcode-dispatch
    counter's top-12 on Richards + Crypto + DeltaBlue): `op_move`,
    `op_add`, `op_get_named_property`, `op_set_named_property`,
    `op_call_small_common`, `op_load_local`, `op_jump`,
    `op_jump_if_true`, `op_load_global`, `op_get_keyed_property`,
    `op_return`, `op_load_const8`.
  - run_trampoline_uncounted asm.
  - Commit to `reports/js/lyng-js/baseline-2026Q2-asm.md`. This is the
    file Phase 0 should have produced.
- **R-0.3 LLInt reference asm.** Build JSC LLInt locally (or extract
  from the system `libJavaScriptCore.dylib` via `otool -tvV`) and
  snapshot the matching ~12 opcodes:
  `op_get_by_id`, `op_put_by_id`, `op_call`, `op_loop_hint`, `op_jmp`,
  `op_add`, `op_mov`, etc. Commit to
  `reports/js/lyng-js/llint-reference-asm.md`. This is the
  side-by-side comparison the original roadmap promised but never
  produced.
- **R-0.4 Per-opcode microbench harness.** A new
  `lyng-js-bench microbench` that runs each hot opcode in a tight loop
  (5M iterations) and reports cycles/dispatch via `mach_absolute_time`
  (Darwin) or `clock_gettime(CLOCK_MONOTONIC_RAW)` (Linux). Decoupled
  from V8 v7 noise.
  - Each microbench produces a single number: ns/dispatch for that
    opcode on a hot-cache, no-IC-miss scenario.
  - Initial numbers feed a `reports/js/lyng-js/microbench-baseline.md`.
  - This is the leading indicator. V8 v7 is the lagging indicator.
- **R-0.5 `cargo asm` diff in CI.** A make target /
  `lyng-js-bench asm-diff` that:
  - dumps cargo asm of the 12 hot opcodes,
  - compares instruction counts to the committed baseline,
  - **fails the build if any handler grew by > 5 instructions
    without a corresponding waiver comment.**
  - The waiver is a per-handler comment in source like
    `// asm-budget: 250 instrs (waiver: 2026-05-16 lyng-XXXX)` with a
    grace period.
- **R-0.6 `sample` script with symbol resolution.** A canned `xcrun
  xctrace`/`sample` runner that produces a 30-frame stack profile of
  Richards/DeltaBlue with Rust symbols demangled and inline frames
  shown. Currently we eyeball flamegraphs ad-hoc.

**Exit criteria:** baseline + LLInt asm files committed; CI fails on
unauthorized asm growth; `lyng-js-bench microbench` reports cycle counts
for the 12 hot opcodes with < 2% sample variance over 5 runs.

**Effort:** 1 week.

**Why this is Phase 0 and not in parallel:** every prior performance
claim was a guess. We cannot pick between γ and α on guesses.

### Phase R-1: Substrate spike — γ vs α vs `#[naked]`-γ

**Goal:** produce real measurements of three substrate variants on the
**same handler bodies**, on the **same hardware**, on the **same
workloads**. Decide the substrate.

**Hypothesis to test:** "On the asm shape that the `dispatch_next!`
macro currently emits, switching the macro from `return Step::Continue(...)`
to inline-asm tail-jump (γ) reduces per-dispatch cost by ≥ 5 instrs and
moves Richards by ≥ +15% in isolated bench."

**Variants:**

- **R-1.α (control):** today's `return Step::Continue(handler)`.
  Already measured.
- **R-1.γ-soft:** swap `dispatch_next!` to inline-asm tail-jump on
  AArch64 + x86_64. No other changes. Localized `unsafe` only in the
  macro. Per-arch matrix.
- **R-1.γ-hard:** R-1.γ-soft + `#[naked]` on the 12 hot opcodes, with
  manually managed register pinning for the dispatch state's most-used
  fields.

The handler bodies do not change between R-1.α / R-1.γ-soft /
R-1.γ-hard. Only the dispatch shape changes.

**Verification per variant:**

- `cargo asm` snapshot of all 12 hot opcodes.
- `lyng-js-bench microbench` per opcode.
- V8 v7 sweep, isolated, ≥ 5 samples.
- Test262 unchanged.

**Decision rule:**

- If γ-soft gains < 5% over α on isolated V8 v7 geomean, **the
  trampoline cost is not the bottleneck** — move attention to the
  handler bodies (Phase R-3) and abandon γ.
- If γ-soft gains ≥ 5% and γ-hard gains another ≥ 5%, **adopt γ-hard
  for the 12 hot opcodes**, keep α elsewhere.
- If γ-soft is ≥ 5% but γ-hard adds < 2%, **adopt γ-soft, leave naked
  handlers for later.**
- If both γ-soft and γ-hard fail to move the needle, the handler bodies
  are the cost, not dispatch. Re-target.

**Effort:** 2 weeks for the spike + measurement (1 week per arch is
realistic given the per-handler audit for `#[naked]`).

**No production code lands until the spike answers the question.** This
is the discipline the roadmap claimed to have ("If [the trampoline
overhead is excessive], fall back to γ early") but did not enforce.

### Phase R-2: IC fast path collapse (the real Phase 3)

**Goal:** make `op_get_named_property`'s hit path look like LLInt's
`performGetByIDHelper`. One mode-byte branch. Mode-specific straight-line
blocks. No nested `if-let-Some` stack.

**Concrete changes:**

- **R-2.1 Replace the 4-layer fast-path stack with a mode dispatch.**
  The compact-handler representation gets a `mode: GetByIdMode` byte
  (Default / ProtoLoad / ArrayLength / Polymorphic / Unset). The
  handler in `op_get_named_property` reads the mode and branches once.
- **R-2.2 Move the IC machinery from `vm/dispatch/property.rs::execute_*_opcode`
  inline into the trampoline handler.** No more function call across the
  feedback boundary. This is the "fully inlined" claim from Phase 3f,
  honestly delivered this time. Acceptance: `op_get_named_property`'s
  asm contains no `bl` instruction on the hit path AND `vm/dispatch/property.rs::execute_get_named_property_opcode`
  ceases to exist as a separate symbol on the hot path.
- **R-2.3 Retire the layered helpers.** Once R-2.1 + R-2.2 land, delete:
  - `try_named_property_polymorphic_fast_load`
  - `try_named_property_proto_fast_load`
  - `try_named_property_load_inline_cache_hit` (move its body inline
    into the mode-byte branch's slow path).
  - All the corresponding store / global / keyed variants.

  Each is replaced by a single straight-line block under the mode-byte
  branch.
- **R-2.4 ArrayLength mode (Phase 3h from the original roadmap).**
  Already specified at [jsc-aligned-engine-roadmap.md:670-687](jsc-aligned-engine-roadmap.md);
  fold into R-2.1.

**Verification (asm-shape gates):**

- `op_get_named_property` asm contains exactly one `cmp` + `b.ne` for
  the mode-byte branch, then one straight-line block per mode.
- Hit-path instruction count for Default mode ≤ 25 (matches LLInt's
  ~17 plus α's ~7 dispatch overhead).
- Asm size for `op_get_named_property` ≤ 400 bytes (down from 1080).
- Test262 baseline preserved.

**Verification (microbench gates):**

- `lyng-js-bench microbench op_get_named_property` ns/dispatch ≤ 60% of
  R-0 baseline.
- Richards ≥ 380 in isolated bench (vs today's 318, conservative
  target).

**If gates aren't met, stop.** Investigate. Do not proceed to R-3.

**Effort:** 3 weeks.

### Phase R-3: Handler body audit and slim-down

**Goal:** reduce the per-handler asm size by removing structural overhead
not caused by the substrate. This is where we recover from accumulated
Phase 1–4 cruft.

**Concrete suspects (each verified by cargo-asm diff):**

- **R-3.1 `maybe_record_opcode_dispatch` still costs.** Phase 1 T1
  ([phase-1-diagnostics.md:166-208](phase-1-diagnostics.md)) split the
  trampoline; the per-handler asm cost may persist via debug-assert
  paths or via the `Wide` decoder. Re-audit.
- **R-3.2 `decode_abc_operands` wide path icache pressure.**
  Currently `#[cold] #[inline(never)]`, but LLVM may still emit the
  jump-out + return-back pair, costing 2 instructions per handler. Try
  moving the wide path entirely behind a static table lookup or behind
  a per-opcode generated function.
- **R-3.3 The 384-byte stack frame.** Profile what fills it. Almost
  certainly: scratch `Step` materialization + intermediate `Value`
  spills around the call boundary. Address via R-2.2 (inlining the
  IC body); confirm afterward.
- **R-3.4 `Arc<InstalledFunction>` ref count.** Every handler holds
  `installed: Arc<InstalledFunction>`. Confirm in asm that the Arc
  isn't being cloned/dropped per dispatch.
- **R-3.5 DispatchState size.** Today's `DispatchState` is ~88+ bytes
  (4 references + Arc + FrameRecord + usize + u32 + Option<Opcode>).
  Each handler that touches it reads 1–8 fields. The per-handler
  prologue is bloated by saving the registers needed to address those
  fields. Audit: can `DispatchState` be split into a 3-pointer hot half
  + a cold tail, so the hot half fits in registers across the call?

Each item is a sub-phase with: predicted asm delta, predicted bench
delta, isolated bench, asm before/after, and a stop-if-no-gain rule.

**Effort:** 2 weeks.

### Phase R-4: Compiler & bytecode revisit

**Goal:** evaluate whether the bytecode encoding itself is leaving wins
on the table. This is a hypothesis-test phase; we don't redesign unless
the numbers say to.

**Items, each its own ticket:**

- **R-4.1 Feedback-slot encoding.** Today: 2-byte inline operand on
  IC-shaped opcodes. LLInt: 1-byte `metadataID` operand → metadata
  table indirection. Hypothesis: switching to a metadataID byte saves
  1 byte per IC opcode (~15% of all opcodes), shrinks bytecode ~5%,
  helps icache. Verify with a parallel encoder + bytecode-density
  bench.
- **R-4.2 Accumulator-routed bytecode revisit.** Phase 4c was deferred
  ([phase-4b-status.md](phase-4b-status.md)) because the compiler
  doesn't reserve r0 for accumulator use. The Star-fusion path
  (Phase 4b) implicitly assumes r0 is the accumulator; if the compiler
  doesn't emit r0-routed bytecode the fusion can't fire. Audit the
  emitted bytecode: how often is Star-fusion actually firing? Per
  Phase 4b status, 10/12 workloads gained, but no measurement of the
  fusion-fire rate. Counter the firing rate directly; if it's < 10% of
  expressions, the fusion isn't paying for the per-dispatch branch.
- **R-4.3 Wide prefix elimination.** Wide/ExtraWide currently cost the
  prefix byte on every wide instruction (4-byte → 5-byte for narrow
  ABC → ExtraWide ABC). Investigate the actual narrow/wide split in
  real workloads; if narrow is ≥ 99%, drop ExtraWide entirely.

**Effort:** 3 weeks for the full set; each item is independently
landable.

### Phase R-5 (decision point, not a fixed phase)

After R-1 through R-4 land, the data tells us where we are. The
decision tree from there:

- If isolated Richards ≥ 600 (1.6× past QuickJS, ~30% of LLInt) —
  **honest "past QuickJS" milestone hit**. Re-evaluate JIT
  prerequisites (originally Phase 5). Decide whether to push for
  LLInt-class interpreter (γ-hard + asm handlers) or jump to Baseline
  JIT.
- If isolated Richards ≥ 1000 (~half of LLInt) — **near-LLInt
  interpreter achieved**. Plan Baseline JIT in earnest.
- If isolated Richards stalls at ≤ 500 — substrate is the wall, not
  ICs or compiler. Re-open the substrate decision (β / pure asm).

**Phase 5 and Phase 6 from the original roadmap (JIT prerequisites,
Baseline JIT) remain valid in shape and stay deferred behind the
interpreter milestone.** They were already gated behind Phase 3
landing; that gate was wrong then, it's also wrong now. Don't start
JIT work until the interpreter ceiling is honestly measured.

---

## 6. Tooling improvements (the "be more scientific" delta)

Beyond R-0's baseline-and-asm scaffolding, several pieces of permanent
infrastructure should land alongside R-1:

### 6.1 `cargo asm` automation

```sh
cargo run -p lyng-js-bench -- asm \
  --opcodes op_add,op_move,op_get_named_property,... \
  --baseline reports/js/lyng-js/baseline-2026Q2-asm.md \
  --output /tmp/asm-current.md \
  --diff
```

- Reads the hot-opcode list from a committed config
  (`reports/js/lyng-js/hot-opcodes.toml`).
- Runs `cargo asm --release ...` per opcode.
- Diffs instruction counts and asm size against the baseline.
- Exits non-zero on regression beyond the per-opcode budget.

Wired into `cargo run --release -p lyng-js-bench -- pre-commit-perf` so
no merge to main can grow hot-handler asm without a recorded waiver.

### 6.2 LLInt asm capture

```sh
cargo run -p lyng-js-bench -- llint-asm \
  --jsc /System/Library/Frameworks/JavaScriptCore.framework/Versions/Current/Helpers/jsc \
  --opcodes op_get_by_id,op_put_by_id,... \
  --output reports/js/lyng-js/llint-reference-asm.md
```

Uses `otool -tvV` or `objdump` on the JSC binary to extract LLInt
handler bodies, identified by the `_llint_*` symbol prefix in the
LowLevelInterpreter-generated entrypoints. Output is a structured asm
report that R-0.3 commits.

### 6.3 Per-opcode microbench

```sh
cargo run --release -p lyng-js-bench -- microbench \
  --opcode op_get_named_property \
  --samples 7 \
  --iters 5000000
```

Each microbench is a tiny JS function compiled to bytecode containing
that one opcode in a hot loop. The harness runs it via the VM under a
cycle-count timer. Output: cycles/dispatch with confidence interval.

This is the missing leading indicator. V8 v7 takes 20 minutes and is
dominated by allocator + GC + builtin behavior; per-opcode microbench
isolates the substrate cost we care about.

### 6.4 `lyng-js-bench compare-llint`

```sh
cargo run --release -p lyng-js-bench -- compare-llint \
  --opcodes op_get_named_property,op_set_named_property \
  --output /tmp/llint-compare.md
```

Side-by-side asm: our handler asm in left column, LLInt's matching
handler in right column, instruction count delta at the bottom. Run
once per phase and commit the markdown.

### 6.5 Opcode-dispatch counter top-N

The existing `opcode-counters` feature exposes per-opcode dispatch
counts. Add a top-N report (e.g. "top 12 opcodes account for X% of
dispatches on Richards") to gate which opcodes are part of the
"hot" set we audit each phase.

### 6.6 Bench harness load-average enforcement

The roadmap requires load average < 2.0 for measurements. Today this
is documented but not enforced; benches have been captured at load
~4–5. Add a `--require-isolation` flag (default ON for CI; off for
ad-hoc) that:

- Reads `loadavg`.
- Aborts with a clear message if load > 2.0.
- Suggests `sudo /usr/bin/renice` + `pmset` quieting tricks.

### 6.7 Source-of-truth bench numbers in dcat issue acceptance

Bench numbers cited in dcat issue acceptance criteria should point at a
committed report file path + commit SHA, not at numbers pasted into the
issue. This is mechanical hygiene; it prevents "the +X% gain" claims
from drifting between when the work landed and when it was reviewed.

---

## 7. What changes about how we work

The roadmap's "Guiding Principles" section listed the right rules:

> Profile before each phase, profile after each phase.
>
> Each phase must deliver a measurable interpreter win on its own. No more
> "the package will pay off." If a phase doesn't move benchmarks, the
> package theory is wrong and we adjust.

We violated both. The new operating rule is concrete:

1. **A phase opens with a written hypothesis: predicted asm shape +
   predicted bench delta.** Filed in the phase's ticket.
2. **Phase ships with: asm before, asm after, isolated bench before,
   isolated bench after, side-by-side LLInt comparison.** All committed
   to `reports/js/lyng-js/phase-XYZ-*`.
3. **If the asm shape didn't materialize as predicted, the phase
   doesn't land,** even if benches improved. The why matters: a
   benchmark gain whose mechanism we don't understand is luck, and
   luck doesn't compound.
4. **If the bench delta didn't materialize as predicted, the phase
   doesn't land,** even if asm shape looks right. The roadmap's
   re-evaluation checkpoints apply at every phase, not just Phases 1
   and 3.
5. **If both hold, the phase lands with a status report that lets a
   fresh agent reproduce both measurements** — not just declare them.
6. **The hot-opcode set is fixed at the start of each phase via the
   dispatch counter top-N**, so we don't optimize cold paths by
   accident.

This is not new discipline. It is the discipline the roadmap announced
and that we didn't keep.

---

## 8. Honest target re-baselining

The original roadmap targeted:

- Phases 1–4 (interpreter): **~+60% geomean over Phase 0**, "near JSC
  LLInt-class on the interpreter alone."
- Phases 1–6 (with JIT): ~3–5× cumulative, "JSC Baseline / V8 Sparkplug
  territory."

Where we are: **~+36% Richards** (the canonical workload), a fraction
of QuickJS, ~17% of JSC LLInt. The interpreter ceiling claim of "near
LLInt on α" is not credible after Phases 3 + 4 with the substrate held
constant.

A more honest target set, anchored on the substrate decision:

| Substrate                   | Realistic interpreter ceiling     | Time to milestone | Risk           |
| --------------------------- | --------------------------------- | ----------------- | -------------- |
| α (today)                   | ~50% of QuickJS on geomean        | 0 (here)          | (where we are) |
| α + IC collapse (R-2 only)  | ~80–100% of QuickJS               | 1 month           | Low            |
| γ-soft (R-1 + R-2)          | ~120–150% of QuickJS              | 2 months          | Low–Medium     |
| γ-hard / `#[naked]`         | ~200–300% of QuickJS, ~30–50% LLInt | 3–4 months       | Medium         |
| Pure asm handlers           | Near-LLInt                        | 6+ months         | High           |
| `become` (nightly only)     | γ-hard-ish without `unsafe`       | 1 month           | Operational    |

"Near LLInt on stable Rust" was always the ambitious end of this range.
We now have evidence that α specifically does not get us there. We
should not pretend that more α-shaped work will, and we should not
treat the QuickJS comparison as an embarrassment to avoid mentioning —
it's the realistic interpreter milestone, and we are not even there yet.

The decision the project owner needs to make: **what's the actual
target?** Three coherent options:

- **Option 1: "Past QuickJS, on α, with respect for our time."** Drop
  the LLInt parity goal. Land R-0 and R-2 + R-3 (the structural cleanups
  that bring us past QuickJS in α). Stop. Ship the engine. ~2 months.
- **Option 2: "Near LLInt, on stable Rust, with γ-hard."** Land R-0,
  spike R-1 honestly, commit to γ-hard if the data supports it, then
  R-2 + R-3 + R-4 on the new substrate. ~4–5 months. Includes a
  formal "if R-1 says α isn't worth abandoning, we go with Option 1"
  off-ramp.
- **Option 3: "LLInt parity at any cost."** Build asm-language handlers
  (the `offlineasm` approach in spirit). The user has rejected this
  level of unsafety historically. Not recommended without a re-decision.

The original roadmap committed to Option 2 in form but Option 1 in
substance — α-only, with hopes-and-dreams projections that never
materialized. The next step is picking one option deliberately.

---

## 9. Concrete next actions (week-by-week)

| Week | Work                                                                                          |
| ---- | --------------------------------------------------------------------------------------------- |
| 1    | R-0.1 (isolation gate) + R-0.2 (baseline asm + bench) + R-0.6 (sample script). All in.        |
| 2    | R-0.3 (LLInt asm capture) + R-0.4 (microbench harness) + R-0.5 (CI asm-diff). Land report.   |
| 3-4  | R-1 spike. γ-soft + γ-hard prototypes on 12 hot opcodes. Microbench + V8 v7. Decision report. |
| 5    | **Decision point:** substrate. Pick α / γ-soft / γ-hard / stop.                              |
| 6-8  | R-2 (IC collapse to LLInt mode-byte shape). New phase with strict gates.                     |
| 9-10 | R-3 (handler-body audit).                                                                     |
| 11-13 | R-4 (compiler/bytecode revisit).                                                              |
| 14   | Re-baseline against LLInt. Decision: continue interpreter work, start JIT, or ship.          |

The first 5 weeks are non-negotiable: until we can measure honestly we
shouldn't pick a substrate, and we shouldn't pick a substrate without
the option to revisit. The decision in week 5 is the one this roadmap
was supposed to encode but didn't.

---

## 10. References (for the next agent)

- Original master roadmap:
  [`jsc-aligned-engine-roadmap.md`](jsc-aligned-engine-roadmap.md)
- Phase 1 (the structural decision):
  [`phase-1-spike.md`](phase-1-spike.md),
  [`phase-1-final-asm.md`](phase-1-final-asm.md),
  [`phase-1-diagnostics.md`](phase-1-diagnostics.md).
- Phase 3 (the would-be win):
  [`phase-3a-status.md`](phase-3a-status.md) through
  [`phase-3f-status.md`](phase-3f-status.md), plus the per-phase
  `*-op_get_named_property.asm` snapshots.
- Phase 4 (compiler-side polish):
  [`phase-4a-status.md`](phase-4a-status.md),
  [`phase-4b-status.md`](phase-4b-status.md).
- Bench:
  [`external-engine-compare.md`](external-engine-compare.md) (the V8 v7
  + QuickJS + JSC LLInt comparison),
  [`bench-v8.md`](bench-v8.md),
  [`bytecode-density-aarch64.md`](bytecode-density-aarch64.md).
- JSC LLInt reference (read-only, in `/Users/sondre/dev/WebKit/`):
  - `Source/JavaScriptCore/llint/LowLevelInterpreter.asm` (dispatch
    macros, opcode wrapper macros).
  - `Source/JavaScriptCore/llint/LowLevelInterpreter64.asm` (64-bit
    handler bodies, including `performGetByIDHelper` at lines
    1634–1677).
  - `Source/JavaScriptCore/bytecode/GetByIdMetadata.h` (the mode-byte
    layout this roadmap calls "the missing IC shape").
- Our current dispatch substrate:
  - [`crates/lyng-js/vm/src/vm/dispatch_state.rs`](../../../crates/lyng-js/vm/src/vm/dispatch_state.rs)
    (`DispatchState`, `Step`, `dispatch_next!`, `run_trampoline`).
  - [`crates/lyng-js/vm/src/vm/dispatch_handlers/property.rs`](../../../crates/lyng-js/vm/src/vm/dispatch_handlers/property.rs)
    (`op_get_named_property` and friends — the handlers).
  - [`crates/lyng-js/vm/src/vm/dispatch/property.rs`](../../../crates/lyng-js/vm/src/vm/dispatch/property.rs)
    (`execute_get_named_property_opcode` and the 4-layer IC chain
    R-2 retires).
  - [`crates/lyng-js/vm/src/vm/feedback.rs`](../../../crates/lyng-js/vm/src/vm/feedback.rs)
    (`NamedPropertyFeedback`, the packed-handler layout, the
    polymorphic sidecar — R-2 unifies these into one mode-byte branch).
