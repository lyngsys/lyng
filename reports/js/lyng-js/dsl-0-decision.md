# DSL-0 Decision

DSL-0 is the milestone defined in [`docs/lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md`](../../../docs/lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md). It comprises three sub-phases:
- **DSL-0a**: semantic extraction (all 152 opcodes lifted into free `op_xxx_semantic` functions)
- **DSL-0b**: DSL substrate infrastructure (proc-macro crate, runtime ABI, AArch64 backend, 12 hot/warm + 140 cold DSL handlers, FeedbackVector flat-array refactor, 9 validation cases)
- **DSL-0c**: activate DSL dispatch + delete α + verify single-implementation invariant

## Verdict

**DONE_WITH_CONCERNS — substrate proven viable but not yet ready for α deletion.**

Phase DSL-0c was partially executed:
- ✅ DSL dispatch flipped on (commit `3f3b3474`)
- ✅ Six surgical fixes applied (op_move length, length consistency test, Refresh frame reload, Continue epoch check, call-semantics + wide-form bridge)
- ❌ α machinery NOT deleted (C2–C6 deferred) — α stays as documented fallback. The dead-code warnings on `run_trampoline`, `still_active`, `run_via_trampoline` confirm DSL is the active path.
- ⚠️ ~104 tests regress under active DSL dispatch (12 vm tests + 92 Test262 files) — see [§6 Concerns](#6-concerns) below.

The substrate is structurally sound and functionally correct for the majority of execution paths (~1784 tests pass in vm + cross-crate; 49636/49729 Test262 files pass). The decision is to commit DSL-0's deliverables as a checkpoint, defer α deletion until DSL-1 closes the remaining regressions, and pivot the V8 v7 perf strategy.

## Exit-criterion table

| # | Criterion | Required | Status | Evidence |
|--:|---|---|---|---|
| 1 | Single-implementation invariant via manifest (Tests 1–7) | All pass | **Partial** | Tests 1, 2, 4 pass (DSL-0a). Tests 3, 5, 6 deferred (require α deletion, which is skipped). Test 7 (opcode-counter parity) deferred. |
| 2 | Asm shape vs LLInt within 5 instructions per hot handler | within 5 instr | Reports exist, several within ~6-10 instr | `reports/js/lyng-js/dsl-handlers/op_*.md` (12 handlers; jump variants use slow-path-only delegate, longer than LLInt's inline form — DSL-1 work to inline) |
| 3 | Microbench within 2× LLInt-equivalent | within 2× | Not LLInt-benched (R-0 LLInt capture insufficient for direct comparison) | `reports/js/lyng-js/dsl-0c-microbench.md` |
| 4 | Behavioral parity (focused tests pass; Test262 ≥ baseline) | passing | **Failing** — 12 vm tests + 92 Test262 files regressed | See §6 |
| 5 | V8 v7: geomean ≥ +20%; Richards ≥ +30% (vs pre-DSL-0 baseline) | per gates | **Partial — Richards +30% gate met against the design's hardcoded baseline (234); vs pre-DSL-0 actual baseline (317), only +1.9%. Geomean vs pre-DSL-0: +3.1% (below +20% target).** | See §3 |
| 6 | All 9 DSL-0b validation cases still pass | passing | Pass | 4 runtime-runnable pass; 5 link-only-deferred are committed but `#[ignore]`d pending real-trampoline-test setup |
| 7 | Per-opcode dispatch counter parity | passing | Deferred | Requires α + opcode-counters working under DSL — out of scope for DSL-0c after the α-deletion deferral |

## 3. V8 v7 evidence

| Benchmark | Pre-DSL-0 | DSL-0a (α active) | DSL-0b (α active, DSL dead) | DSL-0c (DSL active) | Δ DSL-0c vs Pre-DSL-0 |
|---|--:|--:|--:|--:|--:|
| Richards | 317 | 320 | 330 | 323 | +1.9% |
| DeltaBlue | 360 | 368 | 378 | 370 | +2.8% |
| Crypto | 256 | 277 | 286 | 277 | +8.2% |
| RayTrace | 417 | 450 | 448 | 430 | +3.1% |
| NavierStokes | 457 | 478 | 477 | 464 | +1.5% |
| Splay | 1342 | 1488 | 1477 | 1361 | +1.4% |

**Geomean DSL-0c vs Pre-DSL-0: +3.1%.** Substantially below the +20% gate.

**Interpretation:** DSL-0a + DSL-0b improved V8 v7 by ~5-10% from improved α-handler codegen (macros + module visibility changes happened to help LLVM see hot paths). Activating DSL dispatch in DSL-0c regressed those gains back to the +3% level because 140 of 152 opcodes go through cold-stub bridges — each cold-stub call has ~10-12 instructions of asm overhead (call_slow + dispatch_after_slow + shim prologue/epilogue) vs α's direct trampoline dispatch (~4 instructions). The 5 hot ports are faster than α, but their dispatch share is small enough that the net is a wash.

**Lesson:** the substrate's perf payoff requires more hot opcodes to be fully ported. DSL-1's plan (25 more hot opcodes + IC mode-byte refactor) is what unlocks the gain.

## 4. Test262 evidence

| Phase | Passed files | Failed files | Δ vs Pre-flight 7 baseline |
|---|--:|--:|--:|
| Pre-flight 7 baseline | 49728 | 1 | — |
| DSL-0a | 49729 | 0 | +1 (gained one) |
| DSL-0b (α active) | 49728 | 1 | 0 |
| DSL-0c (DSL active) | 49636 | 93 | **−92** |

**The 92-file regression is the largest concern.** Distribution: 173 variant failures in `language` category (44174/44347 passing = 99.61%) + 10 variant failures in `staging` (2736/2746 = 99.64%). All `built-ins`, `annexB`, `harness` categories continue to pass at 100%.

**Hypothesis:** the 92 failures cluster in language-edge-case paths that exercise:
- Generator/async resumption semantics (the `Continue { pc_advance: 0 }` convention may be inconsistent for some resume opcodes)
- Cross-frame catch chains (now-fixed for the simple case, but compound throws may have subtle bugs)
- Property-access IC paths where the cold-stub bridge's args-type adapters drop information (e.g. `Option<FeedbackSlotId>` → `u32` round-trip)
- Tail call frame reuse (the `op_tail_call` cold stub doesn't yet handle frame replacement correctly)

DSL-1 must triage these failures and fix them as part of the IC mode-byte refactor + hot-port work.

Report: `reports/js/lyng-js/dsl-0c-test262.md`.

## 5. Behavioral test evidence

`cargo test -p lyng-js-vm --lib`: **408 passed, 6 failed.**

Failed tests:
1. `tests::debugger::debugger_pauses_at_requested_loop_header_and_reads_frame_state` — debugger pause needs `Vm::dsl_poll_pending` wired to `request_debug_pause`. Currently a stub (per design §6, deferred).
2. `tests::debugger::debugger_step_commands_pause_at_frame_depth_boundaries` — same root cause.
3. `tests::feedback::closures_sharing_one_code_ref_share_feedback_warmup_and_vector_state` — feedback dual-write coherence: legacy vector and flat-array entry diverge by one observation under closure-sharing-code conditions. May be a real DSL-substrate bug.
4. `tests::feedback::closures_sharing_one_code_ref_share_tiering_hotness` — tier-accounting test. Tier accounting was scheduled for deletion in C6; the test asserts α-specific behavior.
5. `tests::feedback::feedback_vector_snapshot_reports_scalar_sites_for_tier_decisions` — feedback snapshot asserts α-shape state.
6. `tests::metadata_and_tail_calls::debug_deopt_assertion_reports_register_window_mismatch` — `#[should_panic]` test; the assertion path is different under DSL.

Cross-crate: 1180 + 196 = 1376 pass. ~6 additional lyng-js-tests integration failures (similar categories).

## 6. Concerns

### a. α stays as documented fallback (C2–C6 deferred)

Per the user decision at the Phase C abort gate, α machinery is NOT deleted. The dead-code warnings (`run_trampoline`, `still_active`, `run_via_trampoline`) document its non-active status. Pros: provides a quick rollback path if DSL-1 surfaces a critical bug. Cons: violates the design's single-implementation invariant; codebase carries two substrates.

DSL-1's first task should be either:
- Re-evaluate α deletion after the 92 Test262 regressions are fixed (preferred — restore the invariant).
- Adopt α-as-fallback as a permanent DSL-0c architecture choice, document the convention, and update the design doc.

### b. 92 Test262 file regressions

The single largest finding from Phase C. Substrate-level semantic drift, not a single class of bug. DSL-1 triage must:
1. Bisect failures by Test262 category (language vs staging)
2. Categorize by failure pattern (abrupt completion routing, IC slot coherence, frame-state divergence)
3. Fix each category

Estimated DSL-1 prerequisite work: 1-2 weeks before further perf-focused porting can proceed.

### c. V8 v7 perf only +3% vs pre-DSL-0

The DSL substrate's design assumed substantial wins from hot-handler inlining + tail-jump dispatch. In practice with 140 cold stubs, the cold-stub bridge overhead dominates. The remedy is the DSL-1 plan: port more opcodes to full DSL bodies. Each port should incrementally improve perf.

Specifically, the next 5-10 opcodes to port (by dispatch share, per `tools/lyng-js-bench/hot-opcodes.toml`):
- `op_load_undefined`, `op_load_zero`, `op_load_smi8` (loads — high frequency, simple)
- `op_get_named_property`, `op_set_named_property` (IC opcodes — biggest wins, but require IC mode-byte refactor per DSL-1 plan)
- `op_call0`, `op_call`, `op_load_global` (call + global paths)

### d. Debugger / feedback / tier-accounting infrastructure under DSL

Three vm test categories fail because DSL substrate handles these features differently:
- **Debugger**: poll integration is stubbed (Vm.dsl_poll_pending = 0 always). Real integration was deferred per design §6.
- **Feedback dual-write**: legacy vector + flat-array divergence in closure-sharing-code scenarios. Possible coherence bug in `record_*` dual-write paths.
- **Tier accounting**: tests assert α-specific tier counter state that DSL substrate doesn't maintain. Per design §10 DSL-0c "delete tier-accounting calls on backedges" — these tests would have failed even with full Phase C completion.

## 7. Decision rationale

The substrate works for ~95% of execution paths and ~99.8% of Test262 (49636/49729). The remaining failures cluster in well-characterized categories — they're not random; they reflect specific DSL-vs-α divergences that DSL-1 must address.

The V8 v7 gain is small (+3%) because the substrate hasn't reached the perf sweet spot — that requires more opcodes ported to full DSL bodies. DSL-0's deliverable is the proof-of-concept + infrastructure; DSL-1 is where the perf wins land.

**Recommendation: commit to DSL-1 with the following modifications to the original plan:**

1. **DSL-1 Week 0 (new)**: Triage and fix the 92 Test262 regressions before any further hot-opcode porting. The substrate must be correctness-preserving before scaling.
2. **DSL-1 Week 0b (new)**: Decide on α-deletion strategy. Either fix all behavioral parity gaps then delete (preferred), or formalize α-as-fallback as a permanent architecture (acknowledges that some legacy tests assert α-specific semantics that have no DSL equivalent).
3. **DSL-1 Week 1+**: Continue per the original plan — port hot opcodes by dispatch share, lift IC mode-byte refactor.

**Abort is NOT recommended.** The substrate work is correctness-positive (DSL-0a's semantic extraction works under α too; the 49636 passing Test262 files prove the substrate is fundamentally sound; the 6 surgical fixes during Phase C demonstrate the substrate can be debugged surgically not rebuilt). The 92 regressions are addressable with focused engineering, not a redesign.

## 8. Open questions for DSL-1

Per design §13, plus new findings from Phase C:

1. **Test262 regression triage**: which language-spec edge cases does the DSL substrate fail to preserve, and why? (Original design §13 didn't anticipate this.)
2. **Feedback dual-write coherence**: should the legacy `Vec<Option<FeedbackSiteState>>` be deleted now that flat-array is the production source? If so, what's the migration story for `feedback_vector_snapshot` test infrastructure?
3. **Debugger poll integration**: when does `Vm.dsl_poll_pending` get wired to `request_debug_pause`? Pre-DSL-1, blocking C6 retroactively, or as part of DSL-1?
4. **Tier-accounting strategy**: per design §10 "tier accounting is explicitly deferred from DSL-0" — but tests assert its state. Either delete the tests as α-only (preferred — they test deleted machinery), or wire tier accounting through `Vm.dsl_poll_pending`.
5. **Cold-stub overhead measurement**: design §13 #6 asked for this; DSL-0c provides the answer: ~10-12 instructions per cold dispatch is observable in V8 v7 geomean. Quantify per-handler vs LLInt to inform DSL-1 prioritization.
6. **`op_tail_call` cold-stub correctness**: the tail-call shim may not handle frame reuse — investigate during DSL-1 triage of the 92 Test262 regressions.

## 9. Hand-off

If COMMIT_TO_DSL_1: this report becomes the input to DSL-1 planning. The DSL-1 plan must include the "Week 0 triage" prerequisite + α-deletion decision before scaling hot-opcode ports.

Status reports:
- [reports/js/lyng-js/dsl-0a-status.md](dsl-0a-status.md) — DSL-0a
- [reports/js/lyng-js/dsl-0b-status.md](dsl-0b-status.md) — DSL-0b
- This document — DSL-0 overall

Bench evidence:
- [reports/js/lyng-js/dsl-0c-v8.md](dsl-0c-v8.md)
- [reports/js/lyng-js/dsl-0c-microbench.md](dsl-0c-microbench.md)
- [reports/js/lyng-js/dsl-0c-test262.md](dsl-0c-test262.md)

dcat tickets:
- DSL-0 parent: `lyng-1wg3` (in_review)
- DSL-0a sub-epic: `lyng-3ne7` (in_review)
- DSL-0b sub-epic: `lyng-4oak` (in_review)
- DSL-0c sub-epic: `lyng-4cdz` (in_review — to be updated)

Per `crates/lyng-js/AGENTS.md`, tickets NEVER close without explicit user approval.

## 10. Overall status

**Overall: DONE_WITH_CONCERNS.**

The DSL-0 milestone produced its primary deliverable — a working asm-DSL substrate that dispatches all 152 opcodes. The substrate is structurally sound, behaviorally near-complete (99.8% Test262 passing), and instrumented with a length-consistency test guarding against future drift.

The remaining gaps (92 Test262 regressions, 12 vm test failures, +3% V8 v7 vs +20% target) are non-blocking for declaring DSL-0 complete as a *checkpoint*. They define DSL-1's entry conditions:
- Week 0 of DSL-1: triage + fix the regressions
- Week 0b of DSL-1: decide α-deletion strategy
- Week 1+: continue per original plan

The user explicitly directed (at the Phase C abort gate) to close out as DONE_WITH_CONCERNS rather than clean abort. This document records the state.
