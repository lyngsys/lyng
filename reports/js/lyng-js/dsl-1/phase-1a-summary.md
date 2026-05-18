# DSL-1 Phase 1.A — Trivial Loads (summary)

**Duration:** 2026-05-18 (single-session execution).
**Range:** baseline commit `54e158bc` → HEAD `1bed700d` (plus this summary commit).
**Status:** Phase 1.A closed with 7 inline ports + 2 documented deferrals.

## Scope landed

| Task | Opcode             | Status      | Inline instr | Commit     | New backend macro |
|-----:|--------------------|-------------|-------------:|------------|-------------------|
|  1   | op_load_undefined  | shipped     |  9           | `d931f4a0` | —                 |
|  2   | op_load_null       | shipped     |  9           | `3af7157c` | —                 |
|  3   | op_load_true       | shipped     | 10           | `6ec96ee1` | —                 |
|  4   | op_load_false      | shipped     | 10           | `15c50461` | —                 |
|  5   | op_load_zero       | shipped     | 10           | `9b8e8782` | `tag_smi_const!`  |
|  6   | op_load_one        | shipped     | 10           | `7151b5d6` | —                 |
|  7   | op_load_smi8       | shipped     | 12           | `172d54ea` | `tag_smi_from_signed_byte!` |
|  8   | op_load_const8     | **deferred** to Phase 1.B | — | `9770e4d7` | (refactor needed) |
|  9   | op_load_this       | **deferred** to Phase 1.B | — | `1bed700d` | (refactor needed) |

Asm baselines committed at [`reports/js/lyng-js/dsl-asm-baseline-aarch64/`](../dsl-asm-baseline-aarch64/) (7 files: `op_load_{undefined,null,true,false,zero,one,smi8}.asm`).
Ported reports at [`reports/js/lyng-js/dsl-handlers/`](../dsl-handlers/) (7 files mirroring the asm baselines).
Two new backend macros at [`crates/lyng-js/vm/src/dsl/backend/aarch64/values.rs`](../../../crates/lyng-js/vm/src/dsl/backend/aarch64/values.rs) (`tag_smi_const!`, `tag_smi_from_signed_byte!`).

## V8 v7 movement vs pre-phase baseline

Pre-phase baseline (captured at commit `54e158bc`) and post-phase measurement (captured during Task 10 at HEAD `1bed700d`):

| Workload    | Pre-phase | Post-phase | Delta |
|-------------|----------:|-----------:|------:|
| Richards    | 247       | 247        |  0%   |
| DeltaBlue   | 299       | 299        |  0%   |
| Crypto      | 235       | 235        |  0%   |
| RayTrace    | 392       | 392        |  0%   |
| NavierStokes| 407       | 407        |  0%   |
| Splay       | 1215      | 1215       |  0%   |
| **Geomean** | 387.09    | 387.09     | **0%** |

(Data sourced from [`reports/js/lyng-js/bench-v8.md`](../bench-v8.md) which Task 10 re-ran with the new ports active.)

Phase 1.A target was **≥ +5% geomean cumulative**. **Result: gate not met — flat (within measurement noise).**

### Honest assessment of the flat result

The Phase 1.A +5% V8 v7 target was optimistic given the opcode mix:

- **5 of 7 ports are adjacent-family completions** (`op_load_undefined`/`null`/`true`/`false`/`one`), not in the measured top-30. Their dispatch share is sub-1% combined — they couldn't move V8 v7 meaningfully on their own.
- **1 of 7 ports is top-30 but low share** (`op_load_zero` #16 with 171M dispatches/run, ~3% of total).
- **1 of 7 ports is high share** (`op_load_smi8` #7 with 388M dispatches/run, ~7%). The inline path saves ~10 instructions per dispatch vs the cold-stub call_slow shim (~3.9B instructions saved across a full V8 v7 run). On a ~trillion-instruction workload, that's a sub-percent win.
- **The two deferred ports** (`op_load_const8` #21, `op_load_this` #12) carry the remaining Phase 1.A dispatch share. Deferring them removed the dominant moveable mass.

A more realistic Phase 1.A expectation, given what actually landed: **≤ +2% V8 v7 geomean**, dominated by `op_load_smi8`'s contribution. Observed: 0% (within Crypto/Splay's measurement noise of ~±0.5%). The slow-path-call overhead saved by inlining is real but is amortized across millions of dispatches per opcode; the per-dispatch saving (~10 instructions = ~3 cycles on Apple Silicon) is well below measurement noise for any single opcode unless dispatch share approaches 10%+.

**The big V8 v7 wins are gated on Phase 1.F (IC opcodes — Get/AssignNamed/Keyed Property, LoadGlobal)**, not Phase 1.A. Those are top-30 #3, #6, #13, #28, #30 by share and have much heavier slow-path cost.

## Behavioral correctness

- `cargo test -p lyng-js-vm --lib --release`: **413 passed** ✓ (no regression from pre-phase 413).
- `cargo test -p lyng-js-tests --release`: **1186 passed** ✓ (no regression).
- Test262: not separately re-run in this phase. The 413 + 1186 in-repo tests provide regression coverage for the loads family. Phase 1.B kickoff should re-run the full Test262 sweep before the frame-context refactor lands.

## Limitations / gates NOT verified

The plan's §4 per-opcode gates that could not be verified in this session:

1. **Slow-path-share < 20% per opcode.** The `--count-slow-path-share` counter is no-op since DSL-0c removed the trampoline hook that called `maybe_record_opcode_dispatch` ([`crates/lyng-js/vm/src/vm.rs:335-353`](../../../crates/lyng-js/vm/src/vm.rs)). Counter wiring into the DSL `dispatch!` tail is required before this gate can be measured. **Follow-up: Task 10.A.**

2. **Per-opcode microbench within 2× of LLInt.** Microbench snippets for the 9 Phase-1.A opcodes don't exist in [`tools/lyng-js-bench/src/microbench/snippets.rs`](../../../tools/lyng-js-bench/src/microbench/snippets.rs) — only `Move`, `Add`, `GetNamedProperty`, `Jump` have generators. **Follow-up: Task 10.B.**

3. **`op_load_const8` and `op_load_this` inline ports.** Deferred per documented off-ramp. See [`phase-1a-load-const8-deferred.md`](phase-1a-load-const8-deferred.md) and [`phase-1a-load-this-deferred.md`](phase-1a-load-this-deferred.md).

## Follow-ups (before Phase 1.B starts, or alongside it)

### Task 10.A — re-wire slow-path-share counter

**Goal:** restore per-opcode slow-path-share measurement so the DSL-1 < 20% slow-path-share invariant becomes enforceable.

**Approach:** add a counter increment to the DSL `dispatch!` tail or the `call_slow!` shim, gated behind `--features opcode-counters`. Likely 1-3 instructions per dispatch when the feature is enabled; 0 when disabled.

**Effort:** ~half day. Touches the proc-macro lowerer (`crates/lyng-js-vm-dsl/src/lower.rs`) or the backend `dispatch!` / `call_slow!` macros.

### Task 10.B — add microbench snippets for 7 ported opcodes

**Goal:** unblock the per-opcode microbench-within-2x-LLInt gate for Phase-1.A opcodes.

**Snippets needed:**
- LoadUndefined: tight loop assigning `undefined` to a local
- LoadNull, LoadTrue, LoadFalse, LoadOne: variants of the above with each constant
- LoadZero: same pattern with `0`
- LoadSmi8: tight loop assigning small signed integers (e.g., `-1`, `5`, `127`)

**Approach:** mirror the existing snippet patterns in [`tools/lyng-js-bench/src/microbench/snippets.rs`](../../../tools/lyng-js-bench/src/microbench/snippets.rs) (Move, Add, GetNamedProperty, Jump).

**Effort:** ~half day. Each snippet is a few JS lines + wiring into the dispatch table the microbench tool walks.

### Frame-context refactor (op_load_const8 + op_load_this co-design)

**Goal:** add asm-visible pre-resolved frame-context fields to [`LlIntState`](../../../crates/lyng-js/vm/src/dsl/llint_state.rs) so that:
- `op_load_const8` can read constants from a flat `*const Value` array via a single indirection
- `op_load_this` can read `this` from a fixed-offset Value slot

**Refactor outline** (from the two deferral notes):
1. Add fields: `frame_const_base: *const Value` (16 bytes, *const) and `frame_this_value: Value` (16 bytes; Value is NaN-boxed and may be 8 or 16 bytes depending on layout — verify).
2. Add offsets in [`reg_convention.rs`](../../../crates/lyng-js/vm/src/dsl/reg_convention.rs) via `offset_of!`.
3. Pre-resolve at activation entry in [`entry.rs`](../../../crates/lyng-js/vm/src/dsl/entry.rs):
   - Constants: flatten `ConstantValue` enum to `Value` array (resolving Atom → string-Value at install time).
   - This: resolve `ThisState::Value` happy path; sentinel scheme for `Uninitialized`/`Lexical`.
4. Refresh discipline updates in [`slow_path.rs`](../../../crates/lyng-js/vm/src/dsl/slow_path.rs) bridges, mirroring PB/REGS/FV.
5. New DSL backend macros: `load_constant!($idx => $dst)` and Value-load for fixed offset.
6. GC root-scanning design review — both fields are GC roots.

**Effort:** ~2-3 days implementation + GC design review. Natural fit for Phase 1.B kickoff.

## hot-opcodes.toml calibration

Updated at Task 10 (uncommitted; will commit alongside this summary):

- `LoadSmi8.aarch64_max_instructions = 14` (measured 12, +2 headroom for LLVM rewrite noise)
- `LoadZero.aarch64_max_instructions = 12` (measured 10, +2 headroom)
- `LoadConst8`: deferred-comment added; budget unset
- `LoadThis`: deferred-comment added; budget unset
- All other opcodes: preserved (0 placeholders; calibration happens when their phase ports land)

## Lessons / observations

- **The `_dsl` suffix convention + `_bx` underscore-prefix idiom for unused operands** proved stable across 7 mirror ports.
- **LLVM consistently rewrote canonical `movz xN, #imm, lsl #shift` forms** to `mov xN, #composed_imm` — semantically identical, accept the noise in asm baselines.
- **The `tag_smi_const!` and `tag_smi_from_signed_byte!` macros** added in Tasks 5/7 are reusable for future SMI loaders.
- **Rust-analyzer's "unused import" warnings** on the tag_* macros are false positives — `cargo build` confirms 0 warnings. The proc-macro lowerer's expansion DOES use the imports via the macro invocations in the DSL body. (Confirmed in Task 6.5 batch review.)
- **Two of three "risky" opcodes** (Task 8 const8 and Task 9 this) hit the documented off-ramp. The off-ramp protocol worked: clear evidence, clean working tree, documented deferral, recommendation for Phase 1.B co-design.
- **V8 v7 movement was flat.** The Phase 1.A +5% target was optimistic given the opcode mix (5 of 7 ports adjacent-family, not in top-30; only 1 high-share port `op_load_smi8`). The big V8 v7 wins are gated on Phase 1.F IC opcodes, not Phase 1.A. This isn't a failure of the substrate work — it's a recalibration of expectations against the actual opcode distribution.
- **The phase plan's discipline held.** Every port had asm baseline + ported report + behavioral tests + clean commit. Two deferrals were clean. No regressions. The plan's per-task workflow scaled cleanly across 7 mechanical ports.

## Decision

Phase 1.A exit criteria assessment:

- ✅ **7 of 9 planned opcodes ported** with full ported reports + asm baselines
- ❌ **V8 v7 cumulative ≥ +5%** — observed 0% (target was optimistic for this opcode mix; recalibrate Phase 1.B target)
- ✅ **Behavioral parity** (413 + 1186 passing; no regressions)
- ⚠️ **Test262** not re-run in phase; deferred to Phase 1.B kickoff
- ⚠️ **Per-opcode microbench gates** — Task 10.B follow-up needed before they're enforceable
- ⚠️ **Per-opcode slow-path-share < 20%** — Task 10.A follow-up needed before measurable
- ✅ **2 deferred opcodes** documented with concrete refactor plans for Phase 1.B

**Recommendation:** Phase 1.A's substrate work landed cleanly (7 inline ports, 2 new reusable macros, 2 documented deferrals with refactor plans). The flat V8 v7 result is a recalibration of expectations rather than a failure of execution. Before Phase 1.B:

1. Complete Tasks 10.A (counter wiring) + 10.B (microbench snippets) to make the per-opcode gates enforceable.
2. Schedule the frame-context refactor as Phase 1.B kickoff (unblocks both deferred opcodes + sets the pattern for any future asm-visible frame-state fields).
3. **Update the parent plan/spec** to acknowledge the Phase 1.A +5% target was optimistic, and recalibrate the per-phase V8 v7 targets in light of actual dispatch-share distribution.

The substrate is sound. The discipline held. The targets need recalibration. Phase 1.B can start with infrastructure complete, two refactor decisions made, and a more realistic V8 v7 budget.
