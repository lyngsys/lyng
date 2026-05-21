# Design: DSL-1 Phase 1.B — Locals + frame-context refactor

**Date:** 2026-05-18
**Status:** Design approved; ready for implementation planning.
**Parent design:** [`docs/lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md`](../../lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md) §10 DSL-1 phase.
**Sibling spec:** [`2026-05-18-dsl-1-hot-opcode-rollout-design.md`](2026-05-18-dsl-1-hot-opcode-rollout-design.md) — the DSL-1 epic spec.
**Predecessor:** Phase 1.A summary at [`reports/js/lyng-js/dsl-1/phase-1a-summary.md`](../../../reports/js/lyng-js/dsl-1/phase-1a-summary.md) (HEAD `b680752e`: 7 inline ports + 2 deferrals + corrected +1.7% V8 v7 result).

---

## 1. Goal, scope, exit criteria

### Goal

Complete the Phase 1.A backfill (frame-context refactor + the 2 deferred opcodes), land the infrastructure that makes per-opcode gates enforceable, and port the Phase 1.B target opcodes under strict top-30 + macro-shared-symmetric-pair discipline.

### In scope

- **Infra (10.A + 10.B):** Counter wiring into the DSL `dispatch!` tail (and `call_slow!`/`poll_safepoint!` for slow-path-share banks); microbench snippets for the 14 in-scope opcodes (7 Phase-1.A + 7 Phase-1.B anchors).
- **Frame-context refactor:** Add `frame_const_base: *const Value` and `frame_this_value: Value` to [`LlIntState`](../../../crates/lyng-js/vm/src/dsl/llint_state.rs); pre-resolve at activation entry; refresh on frame transitions; GC root-scanning design review.
- **Phase 1.A backfill:** Inline ports for `op_load_const8` (#21) and `op_load_this` (#12) using the new fields.
- **Phase 1.B opcode ports (top-30 anchors):** `op_load_local_0/1/2/3` (#11/8/18/9), `op_store_local_3` (#22), `op_load_env_slot` (#19), `op_ldar` (#26) — 7 opcodes, ~1.38B combined dispatches/run.
- **Macro-shared symmetric pairs:** `op_store_local_0/1/2` if they share `op_store_local_3`'s macro at <15 min port cost each.

### Out of scope

- `op_load_global` (#30) — semantic IC opcode, defers to Phase 1.F where the IC mode-byte refactor lives.
- All non-top-30 opcodes outside the macro-shared-pair criterion (no adjacent-family completions — Phase 1.A taught the lesson).
- Speculative backend macros; only add when an in-scope opcode demands one.
- Tier accounting (still deferred per parent §10 DSL-0c).

### Strict selection rule (recorded as Phase 1.A lesson)

A port is justified in Phase 1.B if and only if:

1. The opcode is in measured top-30 ([`reports/js/lyng-js/r0/v8-v7-top30.tsv`](../../../reports/js/lyng-js/r0/v8-v7-top30.tsv)), **OR**
2. The opcode is the macro-shared symmetric pair of a top-30 anchor in this phase, with port cost <15 min and no new backend macro required.

Phase 1.A's adjacent-family completions (op_load_undefined/null/true/false/one) would NOT have shipped under this rule. Phase 1.B inherits the rule.

### Exit criteria

1. **All 9 (up to 12 with pairs) opcodes ported** with inline DSL fast paths + ported reports + asm baselines.
2. **Counter infrastructure (10.A)** produces sane per-opcode dispatch counts on a Richards run (matches Move ≈ 4.66B target within 5%); slow-path-share counter produces non-zero values for newly-ported opcodes.
3. **Microbench infrastructure (10.B)** produces ns/dispatch with 95% CI for all 14 in-scope opcodes — no "no snippet" entries.
4. **Frame-context refactor:** behavioral parity (413+ / 1186+); Test262 ≥ pre-refactor baseline; `gc-stress` mode shows no use-after-free or missed roots.
5. **V8 v7 cumulative ≥ +3% vs pre-DSL-0 HEAD `d850f261`** under same-load A/B. Realistic dispatch-share-scaled estimate from Phase 1.A's +1.7%.
6. **No workload regresses > 2%** vs pre-Phase-1.B HEAD under same-load A/B.
7. **Per-opcode slow-path-share < 20%** on V8 v7 (now enforceable thanks to 10.A).
8. **Per-opcode microbench within 2× of LLInt reference** (now enforceable thanks to 10.B).

---

## 2. Sub-phase structure

DSL-1 Phase 1.B splits into **4 sequential sub-phases** with hard gates between. Single-dev wall-clock estimate: **~3-4 weeks** (vs parent spec's 2-week 1.B estimate; the +1-2 weeks is the frame-context refactor + GC review).

| Sub-phase | Name | Duration | Deliverables | Gate to next |
|-----------|------|---------:|--------------|--------------|
| **1.B.0** | Infrastructure | 2-3 days | Counter wiring + microbench snippets | Counter records Move dispatches within 5% of expected; microbench produces CI95 for all 14 in-scope opcodes |
| **1.B.1** | Frame-context refactor | 3-4 days + GC review | `LlIntState` extended; pre-resolution at activation entry; refresh discipline; new backend macros; behavioral + Test262 pass | All behavioral tests pass; Test262 ≥ baseline; `gc-stress` clean; same-load A/B aggregate V8 v7 regression ≤ 2% |
| **1.B.2** | Phase 1.A backfill | 1-2 days | `op_load_const8` + `op_load_this` inline ports | Per-opcode gates green (≤12 instr, slow-path-share <20%, microbench within 2× LLInt, behavioral pass) |
| **1.B.3** | Phase 1.B opcode ports | 1.5-2 weeks | 7 top-30 ports + macro-shared symmetric pairs | Each port: per-opcode gates green; phase-end same-load A/B shows ≥ +3% V8 v7 cumulative vs pre-DSL-0 |

### Why 1.B.0 first

Without 10.A, the < 20% slow-path-share invariant in Phase 1.B's exit criteria is unmeasurable. Without 10.B, per-opcode microbench is unmeasurable for any new port. Doing infra first means every subsequent sub-phase ships with full gate visibility — and back-fills the missing Phase-1.A microbench data as a bonus.

### Sub-phase off-ramps

- **1.B.0 fails:** if 10.A's counter wiring proves harder than estimated (>3 days), defer 10.A to a focused refactor effort and proceed with V8 v7 + behavioral only as Phase 1.B's gates. 10.B is independent; must land regardless.
- **1.B.1 fails:** if frame-context refactor regresses Test262 unfixably or GC integration shows heisenbugs, **abort and reset to 1.B.0 HEAD**. The 2 deferred opcodes stay deferred. 1.B.3 opcode ports proceed independently (they don't depend on frame-context).
- **1.B.2 fails:** if a backfill port doesn't beat its cold stub, that specific port reverts; the other ships if green.
- **1.B.3 fails:** 5+ consecutive port failures (per-opcode gates) → pause phase. Same off-ramp protocol as Phase 1.A.

---

## 3. Subagent dispatch model

Same coordinator/worker model as Phase 1.A (scaled cleanly across 7 mechanical ports + 2 documented deferrals). Differences for Phase 1.B:

### Roles

**Coordinator (main session).** Owns sub-phase gating + dispatch ordering. New responsibility: **runs the same-load A/B at each sub-phase gate** (see §4).

**Implementer worker subagents** — one per work unit:
- **Infra workers (1.B.0):** 10.A counter wiring (1-2 days, multi-file) + 10.B microbench snippets (~half day). Each subagent owns its internal task breakdown via TodoWrite.
- **Refactor worker (1.B.1):** frame-context refactor (3-4 days). Subagent's own task list. Returns when behavioral parity + Test262 ≥ baseline + GC review done.
- **Backfill workers (1.B.2):** one per deferred opcode (`op_load_const8`, `op_load_this`). Unblocked by 1.B.1.
- **Opcode port workers (1.B.3):** one per opcode, standard 8-step workflow. Plus the new gate measurements (microbench within 2× LLInt, slow-path-share <20%).

**Reviewer subagent.** Mandatory for 1.B.1 refactor — too many failure modes (GC roots, layout offsets, pre-resolution semantics) to self-review. Optional elsewhere. `feature-dev:code-reviewer` against the refactor worker's commit range.

### Differences from Phase 1.A's dispatch

1. **Mandatory reviewer for refactor (1.B.1).** Phase 1.A's mechanical ports could be self-reviewed; the frame-context refactor cannot.
2. **Worker briefs include the corrected V8 v7 methodology** — "for gate verification, use same-load A/B; don't trust absolute baselines from prior commits".
3. **The 8-step per-opcode workflow gains step 5.5:** "Run microbench (1.B.0 enables this); verify ns/dispatch within 2× of LLInt reference. If not within 2×, investigate before committing."
4. **Single batch review at Phase 1.B end** covers the opcode ports (Phase 1.A showed per-task reviewer dispatches are wasted overhead on mirror ports).

### Refactor worker dispatch brief (1.B.1)

This is the highest-stakes worker in Phase 1.B. Brief includes:
- Both deferral notes ([`phase-1a-load-const8-deferred.md`](../../../reports/js/lyng-js/dsl-1/phase-1a-load-const8-deferred.md), [`phase-1a-load-this-deferred.md`](../../../reports/js/lyng-js/dsl-1/phase-1a-load-this-deferred.md)) as the requirements spec.
- Pointer to current `LlIntState` layout + offset-of test (`crates/lyng-js/vm/src/dsl/llint_state.rs`).
- Pointer to entry shim (`entry.rs`) and slow-path bridges (`slow_path.rs`) — must update both for refresh discipline.
- Pointer to GC scan code; reviewer must verify the new fields are scanned alongside REGS and frame roots.
- Explicit guidance on `Atom`/`Builtin` constant pre-resolution (const8 deferral note covers this).
- Sentinel scheme for `ThisState::Uninitialized` / `Lexical` (this deferral note covers this).
- Hard requirement: `cargo test` passes at every commit during the refactor; Test262 sweep at the end.

### What's the same as Phase 1.A

- Conservative parallelism: one worker in flight at a time.
- Workers use `git` without `-C` (user deny rule).
- Each commit is self-contained.
- `bench-v8.md` side-effect remains unstaged throughout.

---

## 4. Gates, measurement, and the same-load A/B protocol

### Same-load A/B protocol (mandatory for every V8 v7 gate measurement)

**Trigger:** any sub-phase gate that asserts a V8 v7 delta vs a prior commit.

**Steps:**
1. Stash current working-tree state (`git stash --include-untracked`).
2. Checkout the comparison-base HEAD (`git checkout <base-sha>`).
3. Build release: `cargo build --release -p lyng-js-bench`.
4. Run `cargo run --release -p lyng-js-bench -- v8suite --samples 7 --json /tmp/a-b-base.json`. Note current `uptime`.
5. Checkout the post-change HEAD (`git checkout <post-sha>`).
6. Build release.
7. Immediately re-run v8suite to `/tmp/a-b-post.json`. Verify `uptime` is within ±20% of step-4 loadavg (if not, abort — machine state shifted too much; re-run).
8. Restore working tree: `git checkout <feature-branch>` then `git stash pop`.
9. Compute per-workload deltas and geomean delta from the two JSONs.
10. Commit the A/B data as `reports/js/lyng-js/dsl-1/phase-1b-N-ab-comparison.md`.

**What NOT to do:**
- Don't compare against Task-0 pre-DSL-0 absolute baseline directly (loadavg-shifted numbers are misleading).
- Don't run base and post with other CPU-intensive work between them.
- Don't trust a single A/B run if `uptime` differs more than 20%.

### Per-sub-phase gates

**1.B.0 — Infra works:**
- 10.A: counter records ~4.66B Move dispatches on Richards run (matches pre-DSL-0c counter output within 5%).
- 10.A: per-feature-gated overhead: `v8suite` with vs without `--features opcode-counters` shows ≤5% Richards delta.
- 10.B: `microbench --samples 7` reports ns/dispatch with CI95 for all 14 in-scope opcodes — no "no snippet" entries.

**1.B.1 — Refactor green:**
- `cargo test -p lyng-js-vm --lib --release` (≥413).
- `cargo test -p lyng-js-tests --release` (≥1186).
- Test262 pass count ≥ pre-refactor baseline (run at 1.B.0 closure).
- Same-load A/B against 1.B.0-end HEAD: aggregate V8 v7 regression ≤ 2%.
- GC root-scanning review documented at `reports/js/lyng-js/dsl-1/phase-1b1-gc-review.md`.

**1.B.2 — Backfill ports inline:**
- Per-opcode: ≤12 inline instr; slow-path-share <20%; microbench within 2× LLInt; behavioral tests pass; ported report + asm baseline.

**1.B.3 — Phase-end:**
- Per-opcode: same checks as 1.B.2 for each of the 7 + symmetric pairs.
- Same-load A/B against pre-DSL-0 HEAD `d850f261`: V8 v7 geomean ≥ +3%.
- Same-load A/B against pre-1.B HEAD: no workload regresses > 2%.
- Phase summary at `reports/js/lyng-js/dsl-1/phase-1b-summary.md` with A/B data inline.

### Per-opcode gates (now fully enforceable thanks to 1.B.0)

| Gate | Criterion | Source |
|------|-----------|--------|
| Behavioral | `cargo test -p lyng-js-vm -p lyng-js-tests` passes | Existing suite |
| Asm shape | Within 5 instr of LLInt + documented Value-layout delta | Per-opcode ported report |
| Microbench | ns/dispatch within 2× of LLInt reference, 7-sample median, 95% CI | `lyng-js-bench microbench` (enabled by 1.B.0) |
| Slow-path-share | <20% on V8 v7 | `lyng-js-bench v8suite --count-slow-path-share` (enabled by 1.B.0) |
| Asm baseline | Captured, passes `asm-diff --check` | `lyng-js-bench asm-diff` |
| Ported report | All sections present | `reports/js/lyng-js/dsl-handlers/` |

---

## 5. Risks (deltas from Phase 1.A)

| Risk | Likelihood | Impact | Mitigation |
|------|-----------:|-------:|-----------|
| Frame-context refactor regresses Test262 unfixably | medium | high | 1.B.1 off-ramp: abort, reset to 1.B.0 HEAD, defer const8/this to Phase 1.C+. 1.B.3 ports proceed independently. |
| Frame-context refactor exposes GC bugs (missed roots, use-after-free) | medium | high | Mandatory `gc-stress` test pass before commit. GC root-scanning review as separate reviewer dispatch. Miri tests on the slow-path bridge if applicable. |
| `ThisState::Uninitialized`/`Lexical` sentinel scheme is wrong | medium | high | Refactor brief must include explicit semantics; pre-resolve `Value`; sentinel that the inline path's tag-check treats as bail-to-slow-path; cache `Lexical` on first observation, invalidate on lex-env mutation. Test262 covers throw-at-correct-PC semantics. |
| 10.A counter wiring adds >5% per-dispatch overhead | medium | medium | 1.B.0 gate measures empirically. If >5%, consider sparse-counter strategy. Parent §13.12 flagged this; 1.B.0 resolves with real data. |
| `LlIntState` size growth breaks offset-stable invariant | low | high | `ll_int_state_offsets_stable` test passes at every commit during 1.B.1. New fields well-aligned; tests updated to expect new size. |
| 1.B.3 opcode ports regress per-bench V8 v7 noise | low | medium | Same-load A/B catches this. Per-workload tolerance: "no workload > 2% regression"; per-bench tolerance = empirical CI95 from 1.B.0 microbench infra. |
| Microbench snippets (10.B) unrealistic, gate doesn't reflect real workloads | medium | medium | Snippet design: tight loop with realistic operand mix. Reviewer pass on 10.B snippets before they become gate criteria. |
| Same-load A/B fails because loadavg shifts mid-comparison | medium | low | Verify `uptime` within ±20% before accepting; re-run if shifted. If repeatedly impossible, use 11-sample medians to widen CI. |
| Macro-shared symmetric pairs cost > 15 min each (rule violation) | low | low | Rule says skip if cost exceeds. Document skip in phase summary; anchor ships regardless. |
| Subagent runs interrupted mid-work (as in Phase 1.A Task 10) | medium | medium | Coordinator verifies each subagent's commit landed before marking task complete. If subagent disappears, coordinator inspects working tree, either commits the staged work if green or reverts and re-dispatches. |

Risks retired from Phase 1.A: workflow ambiguity, mechanical port repeatability, unused-import rust-analyzer false-positive.

---

## 6. Deliverables checklist

### Code
- 9-12 new inline DSL handlers in [`cold.rs`](../../../crates/lyng-js/vm/src/dsl/handlers/cold.rs)
- Counter wiring in [`backend/aarch64/control.rs`](../../../crates/lyng-js/vm/src/dsl/backend/aarch64/control.rs) (`dispatch!` tail) and `call_slow!`/`poll_safepoint!` macros
- Counter array field on `Vm` struct (asm-stable `[u64; 256]` for opcodes; similar for slow-path-semantic + slow-path-safepoint banks)
- `LlIntState` extended with `frame_const_base` + `frame_this_value`; offset consts in [`reg_convention.rs`](../../../crates/lyng-js/vm/src/dsl/reg_convention.rs)
- New backend macros as opcode ports surface them (`load_local_const!`, `load_constant!`, `load_value_at_offset!`, others)
- Flat constant-pool resolution in [`entry.rs`](../../../crates/lyng-js/vm/src/dsl/entry.rs) activation entry; refresh discipline in [`slow_path.rs`](../../../crates/lyng-js/vm/src/dsl/slow_path.rs)
- Microbench snippets in [`tools/lyng-js-bench/src/microbench/snippets.rs`](../../../tools/lyng-js-bench/src/microbench/snippets.rs) for 14 opcodes

### Reports
- Per-handler ported reports + asm baselines for the 9-12 new ports
- Updated Phase-1.A ported reports (back-fill microbench data section once 10.B lands)
- 4 sub-phase summaries at `reports/js/lyng-js/dsl-1/phase-1b-N-summary.md`
- Frame-context refactor design doc at `docs/lyng-js/YYYY-MM-DD-frame-context-refactor.md`
- GC root-scanning review at `reports/js/lyng-js/dsl-1/phase-1b1-gc-review.md`
- A/B comparison data per sub-phase
- Final Phase 1.B summary at `reports/js/lyng-js/dsl-1/phase-1b-summary.md`

### Config
- `tools/lyng-js-bench/hot-opcodes.toml`: `aarch64_max_instructions` calibrated for the 7 top-30 Phase-1.B opcodes

---

## 7. Open questions to revisit during execution

1. **Counter overhead vs sparse counters** (parent §13.12). 1.B.0 measures empirically. If >5%, consider sparse-counter strategy. Decision lands in 1.B.0 summary.

2. **GC scan timing for new `LlIntState` fields.** `frame_this_value` is a `Value` root; `frame_const_base` points at heap-allocated flat array (entries are roots). Question: scan via `LlIntState` or via `LlIntRustContext.installed.constants`? Likely answer: canonical source for GC; `LlIntState` is the fast read path. Confirm during 1.B.1.

3. **Compiler invariant for sentinel `this` value.** If pre-resolution sentinel uses (e.g.) `Value::tag_uninitialized()` for `ThisState::Uninitialized`, every inline `op_load_this` must check and bail. Adds ~3 instructions. Measure at 1.B.2.

4. **op_store_local_0/1/2 macro sharing.** Whether the symmetric pair rule applies depends on `op_store_local_3`'s DSL handler structure. If it parameterizes slot index cleanly, port the 0/1/2 pairs <15 min each. If the handler hard-codes slot 3, the pair rule doesn't fire.

5. **Test262 sweep cadence.** Re-run at Phase 1.B kickoff and at 1.B.1 gate. Re-run at 1.B.3 phase-end too? Test262 takes ~hours; recommend yes at phase-end as final regression net. If slow, sample subset of categories at sub-phase gates and full sweep at phase-end only.

6. **Same-load A/B during low-load windows.** If dev machine loadavg constantly shifts, A/B may be impossible. Workaround: schedule A/B for quiet windows (overnight). Document conditions in each A/B file.

---

## 8. Policy alignment

Per parent §12: policy updates landed in DSL-0. Phase 1.B's frame-context refactor stays within the existing scoped-unsafe boundary (`crates/lyng-js/vm/src/dsl/`); no policy changes expected. If the refactor surfaces a new unsafe scope question (e.g., direct `*const Value` dereferencing in entry shim), coordinator audits before merging.

---

## 9. References

- **Parent design:** [`docs/lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md`](../../lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md)
- **DSL-1 epic spec:** [`2026-05-18-dsl-1-hot-opcode-rollout-design.md`](2026-05-18-dsl-1-hot-opcode-rollout-design.md)
- **Phase 1.A summary:** [`reports/js/lyng-js/dsl-1/phase-1a-summary.md`](../../../reports/js/lyng-js/dsl-1/phase-1a-summary.md)
- **Phase 1.A deferral notes:**
  - [`reports/js/lyng-js/dsl-1/phase-1a-load-const8-deferred.md`](../../../reports/js/lyng-js/dsl-1/phase-1a-load-const8-deferred.md)
  - [`reports/js/lyng-js/dsl-1/phase-1a-load-this-deferred.md`](../../../reports/js/lyng-js/dsl-1/phase-1a-load-this-deferred.md)
- **Measured top-30:** [`reports/js/lyng-js/r0/v8-v7-top30.tsv`](../../../reports/js/lyng-js/r0/v8-v7-top30.tsv)
- **Hot-opcodes config:** [`tools/lyng-js-bench/hot-opcodes.toml`](../../../tools/lyng-js-bench/hot-opcodes.toml)
- **DSL substrate:** [`crates/lyng-js/vm/src/dsl/`](../../../crates/lyng-js/vm/src/dsl/)
- **inc_counter! macro (already exists, needs wiring):** [`crates/lyng-js/vm/src/dsl/backend/aarch64/counters.rs`](../../../crates/lyng-js/vm/src/dsl/backend/aarch64/counters.rs)
