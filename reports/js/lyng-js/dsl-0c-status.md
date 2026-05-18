# DSL-0c Status Report

DSL-0c is the third sub-phase of the DSL-0 milestone. Its design-time
scope was: switch `Vm::run` from `run_via_trampoline` (α) to
`run_via_dsl` (DSL), delete the α trampoline + `dispatch_handlers/` +
`dispatch_state.rs` + tier-accounting machinery, and verify the
single-implementation invariant via manifest Tests 3, 5, 6, 7.

## 1. Executed deliverables

| # | Deliverable | Task | Status |
|--:|---|---|---|
| 1 | Real `run_dsl_trampoline` (entry asm + `_interpreter_exit`) | C1 | DONE (`3f3b3474`) |
| 2 | Switch `Vm::run` from α to DSL | C1 | DONE (`3f3b3474`) |
| 3 | Op_move length bug fix (3 → 4) | (post-C1 surgical) | DONE (`8ce047e7`) |
| 4 | Length audit + consistency test for all 152 opcodes | (post-C1 surgical) | DONE (`f2a1ee4b`, `6890578a`) |
| 5 | Refresh arm unconditional `vm.frames().last()` reload | (post-C1 surgical) | DONE (`fcc58609`) |
| 6 | Continue arm epoch check for cross-frame catch parity with α | (post-C1 surgical) | DONE (`05684376`) |
| 7 | Call-semantics + wide-form bridge fix | (post-C1 surgical) | DONE (`83768883`) |
| 8 | C2: Delete `dispatch_handlers/` | C2 | **DEFERRED** — α stays as documented fallback |
| 9 | C3: Delete `dispatch_state.rs` | C3 | **DEFERRED** |
| 10 | C4: Delete `dispatch/` α-only helpers | C4 | **DEFERRED** |
| 11 | C5: Delete remaining α-trampoline references | C5 | **DEFERRED** |
| 12 | C6: Delete tier-accounting calls on backedges | C6 | **DEFERRED** |
| 13 | C7-C8: Post-flip microbench + V8 v7 + Test262 captures | C7-C8 | DONE |
| 14 | C9: Manifest Test 3 + 5 (DSL fn-ptr linker resolution) | C9 | **DEFERRED** — requires α deletion to enable |
| 15 | C10: Manifest Test 6 (`dispatch_handlers` absent grep) | C10 | **DEFERRED** — `dispatch_handlers` still exists |
| 16 | C11: Manifest Test 7 (opcode-counter parity) | C11 | **DEFERRED** — α + opcode-counters out of scope after deletion deferral |
| 17 | C12: DSL-0 decision document | C12 | DONE — `reports/js/lyng-js/dsl-0-decision.md` |
| 18 | C13: DSL-0 exit gate | C13 | DONE — this report |

## 2. The α-deletion deferral

The user directed at the Phase C abort gate (after the C1 subagent
reported the substrate active but with ~30 test failures) to:
- Skip C2-C6 α deletion
- Document remaining failures as known limitations
- Capture benches + decision doc

Rationale: the 92 Test262 regressions + 12 vm test failures are
addressable with focused engineering (DSL-1's Week 0 work), but
deleting α now would remove the fallback option mid-stride. α stays
as **documented fallback** until DSL-1 closes the regressions, at
which point α deletion + manifest invariants (Tests 3, 5, 6, 7)
become the first DSL-1 milestone.

The dead-code warnings on `run_trampoline`, `still_active`, and
`run_via_trampoline` document α's non-active status. The build is
clean despite α being dead code.

## 3. Test results

`cargo test -p lyng-js-vm --lib`: **408 passed / 6 failed**.

Failing tests:
- `tests::debugger::debugger_pauses_at_requested_loop_header_and_reads_frame_state` — `Vm::dsl_poll_pending` is a stub; debugger pause integration deferred per design §6.
- `tests::debugger::debugger_step_commands_pause_at_frame_depth_boundaries` — same.
- `tests::feedback::closures_sharing_one_code_ref_share_feedback_warmup_and_vector_state` — possible DSL feedback dual-write coherence bug.
- `tests::feedback::closures_sharing_one_code_ref_share_tiering_hotness` — α-specific tier counter assertions.
- `tests::feedback::feedback_vector_snapshot_reports_scalar_sites_for_tier_decisions` — α-shape state assertion.
- `tests::metadata_and_tail_calls::debug_deopt_assertion_reports_register_window_mismatch` — `#[should_panic]` test; assertion path differs under DSL.

Cross-crate (`lyng-js-bytecode`, `lyng-js-objects`, `lyng-js-compiler`, `lyng-js-tests`): 1180+196 = 1376 pass with ~6 additional failures in `lyng-js-tests` (similar categories).

**Test262: 49636/49729 files passing.** 92-file regression vs Pre-flight 7 baseline (49728/49729). 173 variant failures in `language` + 10 in `staging`. All `built-ins`, `annexB`, `harness` 100%.

## 4. V8 v7 evidence (post-flip)

| Benchmark | Pre-DSL-0 | DSL-0c (DSL active) | Δ |
|---|--:|--:|--:|
| Richards | 317 | 323 | +1.9% |
| DeltaBlue | 360 | 370 | +2.8% |
| Crypto | 256 | 277 | +8.2% |
| RayTrace | 417 | 430 | +3.1% |
| NavierStokes | 457 | 464 | +1.5% |
| Splay | 1342 | 1361 | +1.4% |

**Geomean +3.1%** vs the +20% target. Discussion in `dsl-0-decision.md` §3.

## 5. Surgical fixes made during Phase C

Six commits past C1 (`3f3b3474`):
- `8ce047e7` — op_move length 3→4
- `f2a1ee4b` — length audit across all DSL handlers
- `6890578a` — length consistency test
- `fcc58609` — Refresh arm unconditional vm.frames().last() reload
- `05684376` — Continue arm epoch check
- `83768883` — Call-semantics + wide-form bridge fix

Each was investigated, hypothesized, applied, and verified. Two subagent investigations identified the bugs precisely; the third confirmed the call-semantics fix worked.

## 6. Hand-off to DSL-1

DSL-1's revised entry conditions:

1. **Week 0 — Test262 + vm-test triage** (NEW PREREQUISITE). Bisect and fix the 92 Test262 + 12 vm regressions. Estimated 1-2 weeks.
2. **Week 0b — α-deletion strategy decision** (NEW). Either delete α after all parity gaps close (preferred), or formalize α-as-fallback as permanent architecture.
3. **Week 1+** — Continue per original plan: port 25 more hot opcodes + IC mode-byte refactor.

The substrate's cold-stub overhead (~10-12 instructions per dispatch) means perf wins require more hot opcodes ported. DSL-1's plan should prioritize by dispatch share from `tools/lyng-js-bench/hot-opcodes.toml`.

## 7. Status

**Overall: DONE_WITH_CONCERNS.**

The DSL substrate is active and structurally sound. 99.8% of Test262 passes; vm + cross-crate suites pass ~99% of cases. The remaining 12+92 failures cluster in well-characterized categories (debugger poll integration, tier-accounting, feedback dual-write coherence, language-spec edge cases) and define DSL-1's entry conditions.

α stays as documented fallback. Decision document: `reports/js/lyng-js/dsl-0-decision.md`.

DSL-0c dcat sub-epic `lyng-4cdz` and 13 task tickets are `in_review` awaiting user approval. Per `crates/lyng-js/AGENTS.md`, tickets NEVER close without explicit user approval.
