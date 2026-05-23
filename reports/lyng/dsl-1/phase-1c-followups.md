# DSL-1 Phase 1.C — Followups

Items surfaced during Phase 1.C that don't block phase close but warrant
tracking. Pick up opportunistically or schedule into Phase 1.D / 1.E / 1.F or a substrate sub-phase.

---

## High priority

### 1. Substrate fix: `inject_opcode_byte` counter-injection discipline

**Surfaced in:** Task 2 (op_sub) spec review (commit `e7a6cfab`+follow-ups).
**Affects:** All Phase 1.C ports (slow-path-share readings); will affect Phase 1.D comparison opcodes which use the same record-smi shim pattern.

**Problem.** The proc-macro `llint_handler!` lowerer auto-injects `inc_slow_semantic_counter!` on every `call_slow!` call site via `crates/lyng-js-vm-dsl/src/lower.rs::inject_opcode_byte`. The injection has no awareness of label-scope state: when a handler has both a fast-path `call_slow!(op_xxx_record_smi_rs, args = [slot])` (for feedback recording) AND a `.slow:` label scope with `call_slow!(op_xxx_slow_rs, ...)` (for the actual slow path), the lowerer instruments BOTH call sites. The fast-path invocation's counter bump incorrectly attributes successful inline executions as slow-path semantic entries.

**Consequence.** All `record-smi-shim-pattern` opcodes report ~100% slow-path-share regardless of actual fast-path hit rate. Per-opcode `<20%` gate (spec §1.6 + §5) cannot be honestly enforced. Phase 1.C documented per-workload waivers for each ported opcode.

**Fix (estimated 10-30 lines).** In `crates/lyng-js-vm-dsl/src/lower.rs`, the body parser already separates `BodyStmt::MacroCall` from `BodyStmt::Label` entries. Track a `seen_label: bool` flag during the body-token walk in `inject_opcode_byte`; flip to `true` when the first `BodyStmt::Label` is encountered. Skip the `opcode_byte = N` (or equivalent counter-injection) parameter when emitting `call_slow!` invocations *before* the first label boundary — those are fast-path tail invocations, not slow-path entries.

**Verification.** Add a regression test that exercises a handler with both fast-path `call_slow!(record_shim)` and slow-path `.slow: call_slow!(slow_rs)`, measures `inc_slow_semantic_counter!` on both code paths, and asserts only the slow-path invocation bumped the counter. The existing op_add / op_sub / op_mul handlers can serve as the test fixture after the fix lands.

**Effort.** ~10 lines + one regression test. Estimated 2-3 hours including code review.

**Unblocks.** Honest slow-path-share enforcement for op_add (since DSL-0), all 7 Phase 1.C ports, and all future opcodes that use the record-smi shim pattern (Phase 1.D comparisons, Phase 1.F IC opcodes, etc.).

**Recommended timing.** Land BEFORE or DURING Phase 1.D. The slow-path-share gate cannot be enforced for Phase 1.D comparison opcodes until this lands.

---

### 2. `asm-diff --check` namespace expansion (carryover from Phase 1.B)

**Surfaced in:** Phase 1.B followups; persisted through Phase 1.C.
**Affects:** Asm baseline maintenance.

**Problem.** `lyng-js-bench asm-diff --check` does not auto-discover the `dsl::handlers::cold::*` symbol namespace. Asm baselines under `reports/js/lyng-js/dsl-asm-baseline-aarch64/` for Phase 1.B and Phase 1.C ports were captured manually via `cargo rustc --release -p lyng-js-vm -- --emit=asm` + `awk` extraction.

**Fix.** Extend the bench tool's `asm-diff` symbol discovery to include the `cold` handler symbols. The current logic appears to only enumerate `hot.rs` / `warm.rs` (the original Phase 0 design pre-1.A).

**Effort.** ~30-60 minutes (extend symbol enumeration + add cold-namespace tests).

**Recommended timing.** Low priority; manual capture is working. Could be picked up as a maintenance task between phases.

---

## Medium priority

### 3. op_mul float-workload trade-off

**Surfaced in:** Phase 1.C.1 A/B (commit `e1c45c0b`).
**Affects:** op_mul performance on float-heavy workloads.

**Problem.** Phase 1.C.1's same-load A/B showed RayTrace -2.56%, NavierStokes -0.87%, Splay -1.00% — workloads where op_mul frequently sees double-precision operands. The SMI fast-path attempt costs ~7-8 instructions of `check_smi` (movz+movk+and+movz+movk+cmp+b.ne) before bailing to slow; on workloads where the bail happens nearly every dispatch, the wasted check is net-negative vs the pre-Phase-1.C.1 cold-stub baseline (which went directly to `call_slow!(op_mul_slow_rs)`).

The Phase 1.C cumulative A/B shows the float workloads recovered to net-positive (RayTrace +5.19% cumulative, NavierStokes +18.75% cumulative) because the rest of Phase 1.C's gains (op_increment/op_decrement particularly on NavierStokes) compensate. But the per-port trade-off is real.

**Possible mitigations.**
- **Fast-bail check.** Before the full `check_smi` (which is a 7-instruction sequence to verify the exact SMI tag pattern), do a single-instruction tag-prefix check that fast-rejects all non-SMI tags (`lsr x16, x_value, #48; cmp w16, #0x7ff8` — 2 instructions) — bail to slow on non-NaN-tagged values. Then continue with the existing check_smi for the remaining NaN-tagged tags.
- **Type-feedback-guided slow-path-first.** If feedback indicates the operand is consistently non-SMI, skip the SMI fast path entirely. This would need a feedback-state-aware DSL handler shape — substantial substrate work.
- **Accept the trade-off.** The cumulative A/B is strongly positive (+13.66%) and Phase 1.D's comparison opcodes might add their own float-workload pressure. Worth re-evaluating at Phase 1.D close.

**Recommended timing.** Investigate during Phase 1.D if op_greater_equal / op_less_equal show similar float-workload regressions. Otherwise revisit at Phase 1.E or as a Phase 1.G post-port optimization.

---

### 4. Shared `op_record_smi_arith_6_rs` consolidation

**Surfaced in:** Implicit pattern across Tasks 2-10.
**Affects:** Code duplication in `cold.rs` and `hot.rs`.

**Problem.** Phase 1.C added 7 per-opcode `op_xxx_record_smi_rs` shims, each ~19 lines and structurally identical apart from the symbol name. Plus the original `op_add_record_smi_rs` in `hot.rs`. Total: 8 nearly-identical shims, ~152 lines of repetition.

**Fix.** Introduce a single shared shim `op_record_smi_arith_6_rs(state, slot)` in a new file `crates/lyng-js/vm/src/dsl/handlers/shared.rs` (or as a module of `cold.rs`). It returns `Continue { pc_advance: 6 }`. All 8 record-shim call sites in Phase 1.A-1.C ports use the same `pc_advance: 6` (AbcSlot length), so a single shared shim is sufficient.

**Effort.** ~1 hour (write shim, migrate 8 call sites, update asm baselines if the symbol name changes).

**Recommended timing.** Optional cleanup. Could be picked up as a Phase 1.D preamble (cleaner code before adding 7 more comparison/branch shims) OR deferred indefinitely (the duplication is harmless).

**Trade-off.** A shared shim couples all callers — if op_add's `pc_advance: 6` ever changes (e.g., Wide-prefix decoding lands), all callers would need to update at once. The current per-opcode shims are more locally adjustable.

---

### 5. `verify_opcodes_per_iter` coverage

**Surfaced in:** Tasks 2, 5, 6, 7, 9, 10 code-quality reviews.
**Affects:** Microbench self-test coverage.

**Problem.** The `#[cfg(test)] mod verify_counts` test in `tools/lyng-js-bench/src/microbench/snippets.rs` hard-codes a `names` array of opcodes to verify the `opcodes_per_iter` declaration against real opcode-count data. The list excludes Phase 1.A's Add/Move/Jump, Phase 1.C's Sub/Mul/BitAnd/ShiftLeft/ShiftRight/Increment/Decrement (and historically all pre-Phase-1.B-anchor opcodes).

**Fix.** Add the missing names to the `verify_opcodes_per_iter` list and confirm each snippet's `opcodes_per_iter` claim empirically via `--count-opcodes`. The Mul snippet has a `| 0` co-dispatch (BitOr per iter) that needs accounting; either include the BitOr count in the snippet's declaration or document why it's excluded.

**Effort.** ~30 minutes per opcode × 7-8 opcodes = ~4 hours.

**Recommended timing.** Phase 1.D preamble (alongside followup #4 if pursued).

---

## Low priority

### 6. SMI-elision JS-level coverage gap

**Surfaced in:** Task 11 (writeback test).
**Affects:** Test coverage of the fast-path SMI-elision skip path.

**Status.** The Task 11 test `dsl_increment_writeback` exercises the non-SMI src case which forces the slow path. It cannot test the SMI src case because the elision is observationally a no-op (the test couldn't distinguish "writeback happened" from "writeback skipped" when the value is unchanged).

**Possible verification approaches.**
- Add a Rust-level test in the DSL handler crate that inspects the dispatched register state with instrumentation. Cost: substrate work.
- Trust the structural argument: `ToNumeric(SMI) == SMI` is identity (verified by reading `to_primitive` + `to_number`), so writeback-or-not is unobservable for SMI src.

**Recommended timing.** Defer. The structural argument plus the non-SMI test together cover the safety story for inc/dec.

---

### 7. Phase 1.C cross-day measurement variance

**Surfaced in:** Sub-phase A/B numbers vs cumulative A/B (Phase 1.C.2 summary).
**Affects:** Trust in same-day A/B measurements.

**Status.** Different runs of the SAME commit at different times produced different absolute scores (e.g., Richards measured at 299, 306, 292, 296, 295 across the day). The relative A/B deltas held but the absolute values varied with loadavg + thermal state.

**Mitigation already in place.** Always measure base and post back-to-back within ~15-minute windows + loadavg-overlap-check at ±20%. The Phase 1.C cumulative A/B (Task 13) followed this rigorously and produced clean +13.66% with all-positive workload deltas.

**Recommended.** No action. The protocol works. Document in the Phase 1.D plan that same-day back-to-back is required.

---

### 8. `parses_the_committed_hot_opcodes_toml` pre-existing failure

**Surfaced in:** Tasks 2-7 build verifies.
**Affects:** `cargo test -p lyng-js-bench --lib`.

**Status.** The test `parses_the_committed_hot_opcodes_toml` asserts `expected at most 35 hot opcodes, got 37` (or similar — count is now 37 since Phase 1.B added entries beyond the pre-Phase-1.A baseline). Pre-existing failure, reproduces at all pre-Task-2 HEADs.

**Fix.** Either: (a) raise the test's upper bound to match the actual top-30 + macro-shared-pair budget (~37-40), OR (b) drop the test (the budget is enforced by code review, not test).

**Effort.** ~5 minutes.

**Recommended timing.** Phase 1.D preamble. Drives the assertion to reality so the test crate is green at HEAD.

---

### 9. `feedback_flat_consistency::dual_write_*` pre-existing failures

**Surfaced in:** Tasks 2-3 implementer reports.
**Affects:** `cargo test -p lyng-js-vm`.

**Status.** Two test failures in `crates/lyng-js/vm/tests/feedback_flat_consistency.rs` (`dual_write_keeps_smi_add_legacy_and_flat_in_sync`, `dual_write_keeps_polymorphic_property_access_legacy_and_flat_in_sync`) reproduce at all pre-Task-2 HEADs. They trace back to commit `fe631a70` (DSL-0b B17: dual-write FeedbackEntry from existing record paths). The legacy/flat divergence is on Call-feedback, not on the arithmetic or property paths Phase 1.C touched.

**Fix.** Investigate the dual-write divergence on Call-feedback; may need to align the call-recording bookkeeping between legacy `FeedbackVector` and the new flat `FeedbackEntry` array.

**Effort.** Unknown — depends on whether the divergence is a real correctness issue or just a stale test invariant. Estimated 2-8 hours.

**Recommended timing.** Phase 1.F (IC mode-byte refactor) preamble — the IC family touches Call-feedback heavily.

---

### 10. Pre-existing `test262.md` working-tree dirt

**Surfaced in:** Tasks 5, 7, 9, 10 implementer reports.
**Affects:** Working-tree state during Test262 slice runs.

**Status.** Running the Test262 corpus filter rewrites `reports/js/lyng-js/test262.md` as a side effect. The implementers correctly reverted the file before committing to avoid polluting Phase 1.C commits with stale Test262 output.

**Mitigation.** Add `reports/js/lyng-js/test262.md` to a per-run-output gitignore filter, OR make the test262 runner write to `/tmp/...` by default when no `--report` flag is given, OR have the runner check if the file is git-tracked + would be overwritten and refuse without `--force`.

**Effort.** ~30 minutes.

**Recommended timing.** Low priority. Workers know to revert.

---

## Closed during Phase 1.C

These were tracked from Phase 1.B and remain closed/active:

- ✅ Phase 1.B followup: Microbench snippet for LoadConst8 + LoadThis (closed in Phase 1.B cleanup batch 1; not re-opened in 1.C)
- ⚠ Phase 1.B followup: `ThisState::Uninitialized` JS-coverage gap (still open; not in 1.C scope)
- ⚠ Phase 1.B followup: `Vec<ConstantValue>` pre-resolution shape (still open; carries into 1.F)
- ⚠ Phase 1.B followup: StoreLocal0 deprecation candidate (still open; not blocking)
- ⚠ Phase 1.B followup: `asm-diff --check` namespace expansion (RE-OPENED in Phase 1.C as followup #2 — same issue, persists)

---

## Summary by priority

**High priority (block or significantly impact Phase 1.D):**
1. Substrate fix for `call_slow!` counter-injection
2. `asm-diff --check` namespace expansion (continuing P1.B)

**Medium priority (worth doing during Phase 1.D / 1.E):**
3. op_mul float-workload trade-off investigation
4. Shared `op_record_smi_arith_6_rs` consolidation
5. `verify_opcodes_per_iter` coverage

**Low priority (opportunistic):**
6. SMI-elision JS-level coverage gap
7. Phase 1.C cross-day measurement variance (no action; protocol works)
8. `parses_the_committed_hot_opcodes_toml` bound update
9. `feedback_flat_consistency::dual_write_*` investigation (likely Phase 1.F preamble)
10. `test262.md` working-tree dirt mitigation
