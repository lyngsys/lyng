# asm-DSL engine — state of the engine (2026-05-22)

**HEAD:** `4867393a` (Phase 1.C closed; cumulative A/B + summary + followups committed).
**Cumulative V8 v7 vs pre-DSL-0 `d850f261`:** **+13.66% geomean** (11-sample direct measurement at Phase 1.C close).
**Behavioral parity:** 418 `lyng-js-vm --lib` + 1209 `lyng-js-tests` + 4 new `dsl_increment_writeback` tests passing. Test262: **49729 files passing / 0 failing / 100.00% rate** on runnable.
**Parent design:** [`docs/lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md`](../../docs/lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md).
**DSL-1 epic spec:** [`docs/superpowers/specs/2026-05-18-dsl-1-hot-opcode-rollout-design.md`](../../docs/superpowers/specs/2026-05-18-dsl-1-hot-opcode-rollout-design.md).

---

## 1. What this engine is

The asm-DSL interpreter is a hand-shaped LLInt-style fast path for lyng-js's JavaScript bytecode dispatch. Per the parent design §3, the goal is to replicate JSC's `LowLevelInterpreter64.asm` discipline — direct asm handlers tail-jumping through a dispatch table — but expressed in a Rust proc-macro DSL so the substrate stays in-language and the slow-path semantic bodies are shared with the original Rust dispatcher.

The architecture uses:

- **Pinned registers** (AArch64: x19=PC, x20=REGS, x21=FV, x22=VM, x23=TABLE, x24=STATE) across the whole handler chain.
- **`#[repr(C)] LlIntState`** as the asm-visible state record — a fixed-layout struct read directly by handlers via offset constants.
- **`naked_asm!` handlers** built via `llint_handler!` proc-macro + `macro_rules!` backend ops in `crates/lyng-js/vm/src/dsl/backend/aarch64/`.
- **Slow-path bridge** (`crate::dsl::slow_path::LlIntDispatchState`) for opcodes that can't (or shouldn't) inline. Each bridge call goes through a uniform shim that snapshots PC + register window, runs the existing semantic body, and returns one of {Continue, Refresh, ExitDone, ExitError}.
- **Mirror discipline** for arena pointers (instruction bytes, constants array, register window, feedback slab): the `LlIntState` fields are pointers into GC-or-arena-allocated storage, refreshed by the Refresh arm of `translate_outcome` after any slow-path call.
- **Record-smi shim pattern** (since DSL-0, formalized in Phase 1.C): inline-ported opcodes with feedback slots invoke a per-opcode `op_xxx_record_smi_rs` shim via `call_slow!` from the fast-path tail, which calls `vm.record_feedback_slot(code, slot)` and returns `Continue { pc_advance: <length> }`. The shim approach is a workaround for the placeholder `entry_observed` offset binding in feedback.rs; inline `record_smi!` is structurally available but not yet wired.

The DSL is implemented across two crates:

- `crates/lyng-js-vm-dsl/` — the proc-macro lowerer (parse `llint_handler!`, emit `naked_asm!` with universal named bindings for offsets/scratch regs).
- `crates/lyng-js/vm/src/dsl/` — the runtime side: `LlIntState`, register-convention constants, `entry.rs` trampoline shim, `slow_path.rs` bridge, `backend/aarch64/` operation vocabulary, `handlers/{cold,warm,hot}.rs` opcode handlers.

---

## 2. What has landed (timeline)

### DSL-0 substrate (closed at `d850f261`; pre-engagement)

(Unchanged from prior snapshot.)

The substrate phases (DSL-0a, 0b, 0c) brought up the entire DSL infrastructure: handler factory + dispatch table + entry/exit shims (0a); feedback-vector flat refactor + warm/hot family split (0b); slow-path bridge with the `DispatchState` shape (0c). By DSL-0 close, every opcode existed as a `cold-stub` `call_slow!` shim — semantically correct, no inline asm.

### DSL-1 Phase 1.A (closed at `b680752e`)

7 trivial-load opcodes inline-ported in `cold.rs`. V8 v7 vs `d850f261`: +1.7%. Two opcodes (`op_load_const8`, `op_load_this`) deferred to Phase 1.B due to substrate gaps.

Summary: [`reports/js/lyng-js/dsl-1/phase-1a-summary.md`](dsl-1/phase-1a-summary.md).

### DSL-1 Phase 1.B (closed at `aa3ab9fc`)

Four sub-phases. Combined: substrate refactor (frame-context: `frame_const_base` + `frame_this_value` mirrors on `LlIntState`) + 11 inline ports + infra + methodology cleanup. V8 v7 cumulative: **+8.51%** vs `d850f261`. Methodology lessons #1-5 codified (loadavg overlap, sub-phase A/B composition, substrate runtime verification, grep over summary tables, bytecode-builder peephole analysis).

Summary: [`phase-1b-summary.md`](dsl-1/phase-1b-summary.md).

### DSL-1 Phase 1.C (closed at `4867393a`)

Three sub-phases. Combined: 7 inline ports + 2 new backend macros + 1 substrate macro extension + 1 unit test + methodology lessons #6-9 codified.

#### 1.C.0 — substrate prep (Task 1, commit `64e3e5cb`)

- New macros `inc_smi_overflow!` and `dec_smi_overflow!` in `arithmetic.rs` (3 instructions each: `adds`/`subs` with 12-bit immediate `#1` + `b.vs` + `sxtw`).
- Runtime verification deferred to 1.C.3 ports (per methodology lesson #3); no separate compile-only test.

#### 1.C.1 — binary arith with overflow (Tasks 2-4, closed at `e1c45c0b`)

Two inline ports:

| Opcode | Top-30 rank | Dispatches/run | Asm-shape instr |
|--------|------------:|---------------:|----------------:|
| op_sub | #29         | 65M            | 36              |
| op_mul | #4          | 589M           | 40              |

**Substrate refactor:** `mul_smi_overflow!` extended from 4 → 7 instructions to add ECMAScript -0 deferral (`cbnz + orr + tbnz`) — caught by pre-existing `script_core_specialized_smi_arithmetic_preserves_negative_zero` test at first compile.

Same-load mini A/B vs Phase 1.C.0 close: +0.31% geomean (mixed signal — integer workloads gained, float workloads regressed due to op_mul's SMI-attempt overhead on doubles).

Summaries: [`phase-1c1-summary.md`](dsl-1/phase-1c1-summary.md), [`phase-1c1-ab-comparison.md`](dsl-1/phase-1c1-ab-comparison.md).

#### 1.C.2 — bitwise / shifts (Tasks 5-8, closed at `521c35af`)

Three inline ports:

| Opcode          | Top-30 rank | Dispatches/run | Asm-shape instr |
|-----------------|------------:|---------------:|----------------:|
| op_bit_and      | #24         | 98M            | 35              |
| op_shift_left   | #25         | 89M            | 36              |
| op_shift_right  | #10         | 266M           | 36              |

No new substrate; all macros pre-existing from DSL-0. Same-load mini A/B vs 1.C.1 close: **+3.04% geomean (all positive)**, Crypto +6.92% / Splay +5.34% standouts. Per Task 6 cleanup, `cold.rs` module header updated to reflect the hybrid generated+hand-maintained model + `op_mul_record_smi_rs` docstring backfilled with the trailing `Returns Continue` sentence.

Summaries: [`phase-1c2-summary.md`](dsl-1/phase-1c2-summary.md), [`phase-1c2-ab-comparison.md`](dsl-1/phase-1c2-ab-comparison.md).

#### 1.C.3 — unary update (inc/dec) (Tasks 9-12, closed at `30fda09c`)

Two inline ports + 1 unit test:

| Opcode        | Top-30 rank | Dispatches/run | Asm-shape instr |
|---------------|------------:|---------------:|----------------:|
| op_increment  | #5          | 541M           | 27              |
| op_decrement  | #23         | 99M            | 27              |

The shortest inline paths in Phase 1.C — 27 instructions each, saving ~9 instructions vs binary ports thanks to unary single-source layout (no rhs operand decode/check_smi/untag). The new substrate macros `inc_smi_overflow!`/`dec_smi_overflow!` are now runtime-verified by these inline ports.

**SMI-elision pattern (correctness-load-bearing).** The inline fast paths SKIP the src register writeback that the semantic body normally performs. For SMI src, `ToNumeric(SMI) == SMI` is identity (verified by reading `to_primitive` + `to_number`), so the writeback is observationally a no-op. Non-SMI src bails to slow which performs the writeback via the semantic body. Verified by `crates/lyng-js/tests/src/dsl_increment_writeback.rs` (Task 11, 4 tests pass).

Same-load mini A/B vs 1.C.2 close: **+3.19% geomean**, NavierStokes +13.56% standout (588M Increments per benchmark cycle).

Summaries: [`phase-1c3-summary.md`](dsl-1/phase-1c3-summary.md), [`phase-1c3-ab-comparison.md`](dsl-1/phase-1c3-ab-comparison.md).

#### Phase 1.C close (Tasks 13-15, closed at `4867393a`)

Direct cumulative A/B vs pre-DSL-0 `d850f261` (11-sample, loadavg-overlap 1.8%, all 6 workloads positive): **+13.66% geomean**. Phase 1.C umbrella summary + 10 followups committed.

Umbrella artifact: [`phase-1c-cumulative-ab.md`](dsl-1/phase-1c-cumulative-ab.md). Summary: [`phase-1c-summary.md`](dsl-1/phase-1c-summary.md). Followups: [`phase-1c-followups.md`](dsl-1/phase-1c-followups.md).

---

## 3. Aggregate metrics at HEAD

### V8 v7 cumulative — direct 11-sample A/B vs pre-DSL-0 `d850f261`

| Workload    | `d850f261` median | `4867393a` median | Delta      |
|-------------|------------------:|------------------:|-----------:|
| Richards    |               249 |               295 | **+18.47%** |
| DeltaBlue   |               303 |               332 |   **+9.57%** |
| Crypto      |               240 |               298 | **+24.17%** |
| RayTrace    |               405 |               426 |   **+5.19%** |
| NavierStokes|               416 |               494 | **+18.75%** |
| Splay       |              1292 |              1383 |   **+7.04%** |
| **Geomean** |                 — |                 — | **+13.66%** |

Cleared the spec-§3 re-baselined target range (+13% to +16%). **All 6 workloads positive; no regressions.**

### Test262

49729 passing files / 0 failing / 100.00% rate on runnable at HEAD. Matches Phase 1.B baseline. The 7 Phase 1.C inline ports don't touch semantic surface.

### Inline-ported opcodes (cumulative across Phase 1.A + 1.B + 1.C)

**25 opcodes inline-ported** (7 in 1.A + 2 in 1.B.2 + 9 in 1.B.3 + 7 in 1.C). Of these, **24 are in the V8 v7 top-30 OR macro-shared symmetric pairs of top-30** (StoreLocal0 is a macro-shared pair but functionally unreachable per Phase 1.B.3 finding).

Per the DSL-1 epic spec §2 table: **25 of ~45 planned opcode ports done** (~55% of DSL-1 scope). Phases 1.D through 1.G land the remaining ~20.

**Combined inlined dispatches per V8 v7 run: ~3.0B** (1.26B from Phase 1.A + 1.B + 1.75B from Phase 1.C).

### Cumulative trajectory vs epic-spec phase gates

| Phase | Epic-spec absolute target | Re-baselined | Actual at close | Status |
|-------|---------------------------|--------------|----------------:|:------:|
| 1.A | ≥ +5% | — | +1.7% | ⚠ shipped below epic target |
| 1.B | ≥ +15% | — | +8.51% | ⚠ shipped below epic target |
| 1.C | ≥ +35% | **+13% to +16%** (spec §3) | **+13.66%** | ✓ within re-baselined target |
| 1.D | ≥ +45% | TBD | TBD | pending |
| 1.F | ≥ +70% | TBD | TBD | pending |
| 1.G | ≥ +80% | TBD | TBD | pending |

**The epic-spec absolute curve has consistently deviated from actual delivered share.** Phase 1.C's transparent re-baselining + empirical calibration is the model — Phase 1.D / 1.F / 1.G should re-baseline at their respective closes based on actual delivered share, not the original projection.

---

## 4. Substrate inventory at HEAD `4867393a`

### `LlIntState` layout (72 bytes)

Unchanged from Phase 1.B close. See prior snapshot for layout details.

### Backend macros (AArch64)

Existing from prior phases (operand decode, register-window access, frame/state access, value tags, control + dispatch, safepoint, counters, feedback).

**New in Phase 1.C** (Task 1, in `dsl/backend/aarch64/arithmetic.rs`):
- `inc_smi_overflow!` — `adds wD, wS, #1; b.vs label; sxtw xD, wD` (3 instr; 12-bit immediate form)
- `dec_smi_overflow!` — `subs wD, wS, #1; b.vs label; sxtw xD, wD` (3 instr; overflow only at i32::MIN)

**Modified in Phase 1.C** (Task 3, in `dsl/backend/aarch64/arithmetic.rs`):
- `mul_smi_overflow!` — extended from 4 → 7 instructions: original `smull + sxtw + cmp + b.ne` + new `cbnz + orr + tbnz` for ECMAScript -0 deferral.

Vocabulary documented in `crates/lyng-js/vm/src/dsl/ops.md`. AArch64-only; x86_64 backend deferred to DSL-2 per parent design §2.

### Per-opcode `op_xxx_record_smi_rs` shims (Phase 1.C addition pattern)

The record-smi shim pattern (originally introduced as `op_add_record_smi_rs` in DSL-0c) is now uniformly applied across all inline-ported opcodes with feedback slots. Each ~19-line shim calls `vm.record_feedback_slot(code, FeedbackSlotId::from_raw(slot))` and returns `Continue { pc_advance: 6 }`. Eight shims total (op_add + the 7 Phase 1.C ports). A consolidation followup is tracked.

### Infra

Unchanged from Phase 1.B close, plus:

- **Microbench snippets:** 6 new snippets in `tools/lyng-js-bench/src/microbench/snippets.rs` (Sub, Mul, BitAnd, ShiftLeft, ShiftRight, Increment, Decrement — 7 total counting the Increment loop-body shape). All use the two-locals pattern to avoid `*Smi` peephole optimizations.
- **`hot-opcodes.toml`:** 7 budgets calibrated from real measurements + 2 headroom (Sub=38, Mul=42, BitAnd=37, ShiftLeft=38, ShiftRight=38, Increment=29, Decrement=29).
- **Unit test:** `crates/lyng-js/tests/src/dsl_increment_writeback.rs` (4 tests, verifies the SMI-elision claim for non-SMI src reaching slow path).

### GC integration

Unchanged from Phase 1.B close. No new arena pointers on `LlIntState` in Phase 1.C; no new mirror-discipline invariants required.

---

## 5. Key methodological lessons (Phase 1.B + 1.C cumulative)

The 5 lessons from Phase 1.B remain in force:

1. ±20% loadavg overlap on A/Bs is a hard gate.
2. Per-sub-phase A/Bs compose roughly but not authoritatively.
3. Structural compile-and-link tests are not sufficient for new substrate macros.
4. Trust grep over summary tables.
5. Bytecode-builder peephole analysis is required for macro-shared symmetric pair claims.

**Phase 1.C added 4 more:**

6. **The `call_slow!` counter-injection artifact.** The DSL lowerer's `inject_opcode_byte` instruments ALL `call_slow!` call sites with `inc_slow_semantic_counter!`, regardless of label scope. Fast-path record-smi shim invocations therefore double-count as slow-path semantic entries, producing 100% slow-path-share readings for all record-smi-shim-pattern opcodes. Per-opcode `<20%` gate cannot be enforced until the substrate fix lands (tracked as Phase 1.C followup #1; recommended timing: before or during Phase 1.D). The +13.66% cumulative A/B confirms the artifact is purely instrumentation.

7. **Sub-phase A/B composition vs direct cumulative diverges in both directions.** Phase 1.B saw direct cumulative LARGER than composition (+8.51% vs predicted +3.4%). Phase 1.C saw the opposite: composition product +6.7% vs direct cumulative +13.66%. **Rule generalizes: per-sub-phase A/Bs are informational only; always measure the umbrella gate directly at phase close.**

8. **Substrate macros may have latent bugs not surfaced by earlier ports.** Task 3 (op_mul) found that `mul_smi_overflow!` was incorrect for ECMAScript -0 semantics — the pre-existing `script_core_specialized_smi_arithmetic_preserves_negative_zero` test failed at first compile. The macro had existed since DSL-0 but no port used it until Phase 1.C. **Correctness-driven substrate extension during a port is the right response (with code review for the substrate change); the 8-step workflow's "abort and report on surfaced refactor" applies to non-correctness-driven refactors.**

9. **SMI-elision patterns are correctness-load-bearing.** Phase 1.C.3 inc/dec elide the src register writeback that the semantic body normally performs, on the argument that `ToNumeric(SMI) == SMI` is identity. The argument MUST be verified by reading the runtime-primitives helpers + backed by a slow-path-complement test. Future similar opcodes (op_negate, op_bit_not) will need the same audit + test pair.

The full audit + cleanup arc is documented in [`phase-1c-summary.md`](dsl-1/phase-1c-summary.md) §9.

---

## 6. Next steps

### Recommended path forward

**Phase 1.D — comparison + branch** (epic spec §2 row 1.D).

7 opcodes: `op_greater_equal` (#20), `op_less_equal` (#27), plus the 5 currently-cold-stub jump opcodes (`op_jump`, `op_jump_if_true`, `op_jump_if_false`, `op_jump_if_true8`, `op_jump_if_false8`).

Estimated effort: ~1 week (mechanical-port shape similar to Phase 1.C; the jump opcodes have non-trivial branch-target logic but the substrate already supports labels).

**Recommended pre-1.D substrate task: land Phase 1.C followup #1** (`inject_opcode_byte` counter-injection discipline fix). ~10 lines + regression test, ~2-3 hours. Unblocks honest slow-path-share enforcement for all Phase 1.D ports.

Phase 1.D cumulative target per epic spec §2: ≥ +45%. Re-baselining decision will be made at Phase 1.D close based on actual delivered share — Phase 1.D adds ~7 opcodes with combined dispatch share around 800M-1B per V8 v7 run, likely yielding +3-5pp on top of Phase 1.C's +13.66%, for a Phase 1.D close around +17% to +19% cumulative. **The +45% epic-spec target is unlikely to be met without significant additional work; expect another re-baselining.**

### Alternative: LoadEnvSlot substrate sub-phase

Pre-Phase-1.D substrate work to add a `frame_lexical_env` mirror on `LlIntState` analogous to Phase 1.B.1's `frame_const_base`. Unlocks `op_load_env_slot` (top-30 #19) + de-risks Phase 1.F IC family. Estimated 3-4 days. See Phase 1.B followups + the prior engine state snapshot §6 Option B for details.

### Other phases per epic spec §2

| Phase | Scope | Notes |
|-------|-------|-------|
| 1.D | Comparison + branch (7 opcodes) | Next; ~1 week |
| 1.E | Pointer-identity cells refactor | 3-4 weeks; no opcode ports; blocks 1.F |
| 1.F | IC mode-byte refactor + 6 IC opcodes | 3 weeks; bundled refactor + ports |
| 1.G | Calls + tail-call (6 opcodes — frame-transitioning, all return `Refresh`) | 1 week; closes DSL-1 |

### Followups tracked

10 followups in [`phase-1c-followups.md`](dsl-1/phase-1c-followups.md). High-priority items affecting Phase 1.D:

- **#1 Substrate fix: `inject_opcode_byte` counter-injection discipline** (recommended pre-1.D)
- **#2 `asm-diff --check` namespace expansion** (carryover from Phase 1.B; manual capture continues to work)

Plus 4 medium-priority items (op_mul float-workload trade-off, shared record-smi shim consolidation, microbench verify coverage, ports of *Smi immediate variants) and 4 low-priority items.

### Off-ramp triggers (per DSL-1 epic spec §2)

Unchanged from Phase 1.B close. At HEAD `4867393a` the banked wins are now **+13.66% cumulative on a 6-workload V8 v7 suite, 25 inline ports, full substrate**; a graceful close at any future off-ramp would still leave a meaningfully faster engine.

---

## 7. Coordinator workflow (refined through Phase 1.C)

The subagent-driven workflow that produced Phase 1.C in ~6 wall-clock hours of execution time uses these patterns:

1. **`/superpowers:brainstorming`** for a new phase/sub-phase → produces a spec at `docs/superpowers/specs/`.
2. **`/superpowers:writing-plans`** → produces an implementation plan at `docs/superpowers/plans/`.
3. **Sub-phase execution** via subagent dispatch:
   - One refactor-worker subagent per task batch.
   - Two-stage review per task: spec compliance reviewer THEN code-quality reviewer (`feature-dev:code-reviewer`).
   - **Bench-heavy tasks (mini A/Bs, cumulative A/Bs) are run by the coordinator directly** — dispatched agents tend to stall on long-running v8suite runs.
   - Mandatory code-reviewer dispatch for substrate-touching sub-phases or non-mechanical refactors.
4. **Sub-phase close**: same-load A/B (informational), behavioral parity check, Test262 spot-check, sub-phase summary, followups recording.
5. **Phase close**: direct cumulative A/B vs pre-DSL-0 baseline, Test262 final check, umbrella summary, followups doc, engine state snapshot.

User-deny rules consistently honored: no `git -C`, no `cd && git`, no `--no-verify`, no destructive operations without consent.

---

## 8. References

### Design docs

- Parent design: [`docs/lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md`](../../docs/lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md)
- DSL-1 epic spec: [`docs/superpowers/specs/2026-05-18-dsl-1-hot-opcode-rollout-design.md`](../../docs/superpowers/specs/2026-05-18-dsl-1-hot-opcode-rollout-design.md)
- Phase 1.B umbrella: [`docs/superpowers/specs/2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md`](../../docs/superpowers/specs/2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md)
- Phase 1.B.1 spec: [`docs/superpowers/specs/2026-05-19-dsl-1-phase-1b1-frame-context-refactor-design.md`](../../docs/superpowers/specs/2026-05-19-dsl-1-phase-1b1-frame-context-refactor-design.md)
- Phase 1.B.2 spec: [`docs/superpowers/specs/2026-05-19-dsl-1-phase-1b2-backfill-ports-design.md`](../../docs/superpowers/specs/2026-05-19-dsl-1-phase-1b2-backfill-ports-design.md)
- Phase 1.B.3 spec: [`docs/superpowers/specs/2026-05-20-dsl-1-phase-1b3-locals-and-ldar-design.md`](../../docs/superpowers/specs/2026-05-20-dsl-1-phase-1b3-locals-and-ldar-design.md)
- Phase 1.C spec: [`docs/superpowers/specs/2026-05-21-dsl-1-phase-1c-smi-arith-and-bitwise-design.md`](../../docs/superpowers/specs/2026-05-21-dsl-1-phase-1c-smi-arith-and-bitwise-design.md)
- Phase 1.C plan: [`docs/superpowers/plans/2026-05-21-dsl-1-phase-1c-smi-arith-and-bitwise.md`](../../docs/superpowers/plans/2026-05-21-dsl-1-phase-1c-smi-arith-and-bitwise.md)

### Phase summaries (chronological)

- DSL-0c close: [`reports/js/lyng-js/dsl-0c-status.md`](dsl-0c-status.md)
- Phase 1.A: [`reports/js/lyng-js/dsl-1/phase-1a-summary.md`](dsl-1/phase-1a-summary.md)
- Phase 1.B umbrella: [`phase-1b-summary.md`](dsl-1/phase-1b-summary.md)
- Phase 1.C umbrella: [`phase-1c-summary.md`](dsl-1/phase-1c-summary.md)
- Phase 1.C sub-phases: [`phase-1c1-summary.md`](dsl-1/phase-1c1-summary.md), [`phase-1c2-summary.md`](dsl-1/phase-1c2-summary.md), [`phase-1c3-summary.md`](dsl-1/phase-1c3-summary.md)
- Phase 1.C followups: [`phase-1c-followups.md`](dsl-1/phase-1c-followups.md)

### Engine state snapshots

- 2026-05-21 (Phase 1.B close): [`asm-dsl-engine-state-2026-05-21.md`](asm-dsl-engine-state-2026-05-21.md)
- **2026-05-22 (Phase 1.C close): this file**

### Per-handler ported reports (25 opcodes cumulative)

Phase 1.A: 7 reports — `op_load_undefined.md`, `op_load_null.md`, `op_load_true.md`, `op_load_false.md`, `op_load_zero.md`, `op_load_one.md`, `op_load_smi8.md`.

Phase 1.B.2: 2 reports — `op_load_const8.md`, `op_load_this.md`.

Phase 1.B.3: 9 reports — `op_load_local_{0,1,2,3}.md`, `op_store_local_{0,1,2,3}.md`, `op_ldar.md`.

Phase 1.C: 7 reports — `op_sub.md`, `op_mul.md`, `op_bit_and.md`, `op_shift_left.md`, `op_shift_right.md`, `op_increment.md`, `op_decrement.md`.

All under [`reports/js/lyng-js/dsl-handlers/`](dsl-handlers/).

### Asm baselines

`reports/js/lyng-js/dsl-asm-baseline-aarch64/` contains captured asm for each inline-ported handler (25 opcodes).

### Key A/B comparison artifacts

- Phase 1.B.3 cumulative A/B (Phase 1.B umbrella gate): [`phase-1b3-cumulative-ab.md`](dsl-1/phase-1b3-cumulative-ab.md)
- **Phase 1.C cumulative A/B (Phase 1.C umbrella gate):** [`phase-1c-cumulative-ab.md`](dsl-1/phase-1c-cumulative-ab.md)
- Phase 1.C sub-phase A/Bs: [`phase-1c1-ab-comparison.md`](dsl-1/phase-1c1-ab-comparison.md), [`phase-1c2-ab-comparison.md`](dsl-1/phase-1c2-ab-comparison.md), [`phase-1c3-ab-comparison.md`](dsl-1/phase-1c3-ab-comparison.md)
- Test262 baselines: Phase 1.B at [`phase-1b-test262-baseline.md`](dsl-1/phase-1b-test262-baseline.md); Phase 1.C unchanged at 49729 (captured in `phase-1c-summary.md` and the cumulative A/B doc).

### Source code anchors

- DSL substrate: `crates/lyng-js/vm/src/dsl/`
- AArch64 backend macros: `crates/lyng-js/vm/src/dsl/backend/aarch64/`
- Opcode handlers: `crates/lyng-js/vm/src/dsl/handlers/{cold,warm,hot}.rs`
- Lowerer proc-macro: `crates/lyng-js-vm-dsl/src/`
- Bench tool: `tools/lyng-js-bench/`
- Test262 runner: `tools/lyng-js-test262/`
- Phase 1.C SMI-elision verification test: `crates/lyng-js/tests/src/dsl_increment_writeback.rs`
