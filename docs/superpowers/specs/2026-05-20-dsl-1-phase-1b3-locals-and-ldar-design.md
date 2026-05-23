# Design: DSL-1 Phase 1.B.3 — Locals + Ldar inline ports

**Date:** 2026-05-20
**Status:** Design draft; awaiting user review.
**Parent spec:** [`2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md`](2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md) — Phase 1.B umbrella.
**Predecessor:** Phase 1.B.2 closed at HEAD `7baf5846`; cleanup batch closed at `08727f92` (audit realignment).
**Mid-phase state:** [`reports/lyng/dsl-1/phase-1b-summary.md`](../../../reports/lyng/dsl-1/phase-1b-summary.md).

---

## 1. Goal, scope, exit criteria

### Goal

Inline-port the LoadLocal / StoreLocal / Ldar opcode family using existing Phase 1.B.1/1.B.2 substrate. Land cumulative V8 v7 ≥ +3% vs pre-DSL-0 HEAD `d850f261` (the umbrella §1 criterion 5 gate) with real margin.

### In scope

**6 top-30 anchors:**
- `op_load_local_0_dsl` (opcode 144, A, length 2)
- `op_load_local_1_dsl` (opcode 145, A, length 2)
- `op_load_local_2_dsl` (opcode 146, A, length 2)
- `op_load_local_3_dsl` (opcode 147, A, length 2)
- `op_store_local_3_dsl` (opcode 151, A, length 2)
- `op_ldar_dsl` (opcode 130, A, length 2)

**3 macro-shared symmetric pairs (qualify under the 15-min rule):**
- `op_store_local_0_dsl` (opcode 148, A, length 2)
- `op_store_local_1_dsl` (opcode 149, A, length 2)
- `op_store_local_2_dsl` (opcode 150, A, length 2)

These three share `op_store_local_3`'s macro shape exactly — same handler body parameterized by literal slot index. Port cost ≪ 15 min each.

**Combined dispatch share:** ~1.38B/V8 v7 run for the 6 anchors per the umbrella spec; the 3 pairs add unmeasured share (not in top-30) but trivial cost.

### Out of scope

- **`op_load_env_slot` (opcode 11) — DEFERRED.** Per the research findings, it requires walking `frame.lexical_env()` and a variable-depth `environment_at_depth` traversal, plus a loop-iteration-env scan even at depth 0. The lexical env is NOT mirrored on `LlIntState`. Inlining requires either:
  - New substrate: `frame_lexical_env` mirror field on `LlIntState` (Phase-1.B.1-style refactor), OR
  - Fast-path-only inlining for the trivial `depth==0` + no-loop-env case with bail-to-slow otherwise.
  
  Either way, this is **substrate work, not a port**. The umbrella spec underestimated this complexity. **Recommended:** defer to a dedicated substrate sub-phase (Phase 1.C.0 or a Phase 1.B.4) co-designed with any other env-related work.

- **`op_star_0..7` (Store Accumulator to Register).** Not in top-30. Layout differs from `op_ldar` (`layout = None, length = 1` — destination baked into opcode byte). Different macro shape; doesn't qualify under the 15-min rule. Defer.

- **`op_store_env_slot`.** Same complexity as `op_load_env_slot`; same deferral.

- **`op_load_local_4..7` / `op_store_local_4..7`.** Not in top-30. Same macro shape as the in-scope variants but per the umbrella's strict selection rule, deferred unless dispatch-share data shows otherwise.

### Strict selection rule honored

- 6 anchors are top-30 ✓
- 3 pairs are macro-shared symmetric (StoreLocal0/1/2 share StoreLocal3's shape) and port cost ≪ 15 min each ✓
- Total: **9 opcodes**

### Exit criteria

1. **All 9 opcodes inline-ported** with per-handler ported reports + asm baselines.
2. **Per-opcode gates** for each:
   - ≤ 12 inline instructions
   - Microbench within 2× LLInt reference
   - Slow-path-share < 20% on V8 v7
   - Behavioral parity
3. **Same-load A/B vs `08727f92`** (pre-Phase-1.B.3 HEAD): aggregate V8 v7 regression ≤ 2%; per-workload ≤ 5%; expected positive delta on the dispatch share (~1.38B for the anchors).
4. **Cumulative V8 v7 ≥ +3% vs `d850f261`** (umbrella §1 criterion 5) — directly measured at phase close with 11+ samples and verified ≤ 20% loadavg overlap.
5. **Test262 ≥ 49729 passing** (umbrella mid-phase baseline at `08727f92`).
6. **Behavioral parity:** `cargo test -p lyng-vm --lib --release` (≥418), `cargo test -p lyng-tests --release` (≥1198). 2 pre-existing `feedback_flat_consistency` failures stay unrelated.
7. **A/B methodology rigor:** hard ±20% loadavg overlap (no rounding); ≥11 samples for any gate-bearing measurement.
8. **Sub-phase summary** at `reports/lyng/dsl-1/phase-1b3-summary.md`.
9. **Mandatory `feature-dev:code-reviewer` dispatch** over the full sub-phase commit range.

---

## 2. Background: substrate inventory

Substrate is fully in place; no new substrate work in 1.B.3:

| Substrate piece | Location | Status |
|------------------|----------|--------|
| `load_acc!($dst)` / `store_acc!($src)` macros | `dsl/backend/aarch64/operands.rs:126/136` | ✅ |
| `load_reg!($idx => $dst)` / `store_reg!($idx, $src)` | `dsl/backend/aarch64/operands.rs:106/116` | ✅ |
| `decode_a!($a)` (1-operand decode) | `dsl/backend/aarch64/operands.rs:34` | ✅ |
| Pinned regs: x19=PC, x20=REGS, x21=FV, x24=STATE | `dsl/reg_convention.rs` | ✅ |

One small new backend macro is justified:
- **`load_local_fixed!(N => $dst)`** and **`store_local_fixed!($src, N)`** — fixed-immediate-index load/store from the register window, avoiding a `movz` to materialize the index in a scratch. Emits `ldr x{dst}, [x20, #(N*8)]` (1 instruction). Used by `op_load_local_1/2/3` and `op_store_local_0/1/2/3`. `op_load_local_0` already maps to `load_acc!` directly.

Without these new macros, each LoadLocalN handler needs ~3 instructions (`movz x_scratch, #N; load_reg!(scratch => dst); ...`) instead of 2 (`load_local_fixed!(N => dst); store_reg!(a, dst); ...`). Worth the ~10 lines of macro code.

---

## 3. Per-opcode designs

### 3.1 `op_load_local_0_dsl` (opcode 144, A, length 2)

**Semantics** (loads.rs:436-448): `dst = registers[0]` (i.e. `args.a := accumulator`).

**Current (cold stub):** `call_slow!(op_load_local_0_slow_rs, args=[a])` + `dispatch_after_slow!()`.

**Target (inline, 2 body instructions):**
```rust
llint_handler! {
    op_load_local_0_dsl, opcode_byte = 144, layout = A, length = 2, |a| {
        load_acc!(10);       // ldr x10, [x20]
        store_reg!(a, 10);   // str x10, [x20, x_a, lsl #3]
        dispatch!();         // 4 instr tail
    }
}
```

**Asm shape (~7 total: 1 decode + 2 body + 4 dispatch).**

### 3.2 `op_load_local_1/2/3_dsl` (opcodes 145/146/147, A, length 2)

**Semantics:** `dst = registers[N]` for N=1,2,3.

**Target (inline, 2 body instructions):**
```rust
llint_handler! {
    op_load_local_N_dsl, opcode_byte = 144+N, layout = A, length = 2, |a| {
        load_local_fixed!(N => 10);  // ldr x10, [x20, #(N*8)]
        store_reg!(a, 10);            // str x10, [x20, x_a, lsl #3]
        dispatch!();                  // 4 instr tail
    }
}
```

**Asm shape (~7 total).**

### 3.3 `op_store_local_3_dsl` (opcode 151, A, length 2) — anchor

**Semantics** (loads.rs:483-495): `registers[3] = registers[args.a]`.

**Target (inline, 2 body instructions):**
```rust
llint_handler! {
    op_store_local_3_dsl, opcode_byte = 151, layout = A, length = 2, |a| {
        load_reg!(a => 10);              // ldr x10, [x20, x_a, lsl #3]
        store_local_fixed!(10, 3);       // str x10, [x20, #24]
        dispatch!();
    }
}
```

**Asm shape (~7 total).**

### 3.4 `op_store_local_0/1/2_dsl` (opcodes 148/149/150) — macro-shared pairs

Same body as 3.3 with literal slot index 0/1/2. ≪ 15 min each.

### 3.5 `op_ldar_dsl` (opcode 130, A, length 2)

**Semantics** (loads.rs:322-333): `accumulator (registers[0]) = registers[args.a]`.

**Target (inline, 2 body instructions):**
```rust
llint_handler! {
    op_ldar_dsl, opcode_byte = 130, layout = A, length = 2, |a| {
        load_reg!(a => 10);     // ldr x10, [x20, x_a, lsl #3]
        store_acc!(10);          // str x10, [x20]
        dispatch!();
    }
}
```

**Asm shape (~7 total).**

### 3.6 Slow-path-share expectations

All 9 opcodes are pure register-window moves with no bail conditions. Expected slow-path-share: **0.00%** for all 9, like the Phase 1.B.2 ports. Inline path handles 100% of cases.

---

## 4. Sub-phase phasing

Single refactor worker, 5 tasks. Revised wall-clock estimate: **3-4 days** (vs umbrella's 1.5-2 weeks — the umbrella assumed LoadEnvSlot in scope; without it, the work is mechanical).

| Task | Deliverable | Estimated time |
|-----:|-------------|---------------:|
| 1 | Add `load_local_fixed!` + `store_local_fixed!` backend macros + structural compiles-and-links test | ~1 hour |
| 2 | Inline-port 4 `op_load_local_N` handlers (one commit) + integration tests | ~2 hours |
| 3 | Inline-port 4 `op_store_local_N` handlers + `op_ldar` (one commit) + integration tests | ~2 hours |
| 4 | Microbench snippets (LoadLocal0/1/2/3, StoreLocal0/1/2/3, Ldar) + verify_opcodes_per_iter pass + per-handler ported reports + asm baselines | ~2 hours |
| 5 | Same-load A/B vs `08727f92` + cumulative A/B vs `d850f261` (umbrella gate) + Test262 confirmation + sub-phase summary + reviewer dispatch | ~3 hours (mostly bench wall-clock) |

Some snippets already exist from Phase 1.B.0 (LoadLocal0..3, StoreLocal3, Ldar per the 1.B.0 summary table). Verify presence; add missing ones (e.g., StoreLocal0/1/2 likely absent).

---

## 5. Test plan

### 5.1 Backend macro tests (Task 1)

- Structural compiles-and-links test for `load_local_fixed!` and `store_local_fixed!` in `dsl_validation_frame_context.rs` (or a new validation test file). Following the **post-audit pattern** documented in the 1.B.1 retrospective: structural is acceptable HERE because real handlers will dispatch through the macros in Tasks 2-3.

### 5.2 Per-opcode integration tests (Tasks 2-3)

For each opcode, at least one JS-level integration test in `crates/lyng-tests/tests/` (or extend existing files). Examples:

- **LoadLocalN family:** `(function(a, b, c, d) { return a + b + c + d; })(1, 2, 3, 4)` — exercises LoadLocal0..3 via parameter access. Assert returns 10.
- **StoreLocalN family:** loops with local-variable updates: `(function() { var x = 0; for (var i = 0; i < 100; i++) { x += i; } return x; })()` — exercises StoreLocal0..3 via variable updates. Assert returns 4950.
- **Ldar:** any expression that uses an intermediate register: `(function(a, b) { var c = a + b; return c * 2; })(1, 2)` — Ldar fires when reading the temporary into the accumulator. Assert returns 6.

The existing `lyng-tests` suite likely already exercises these opcodes via implicit coverage; the new tests are explicit assertions documenting the inline-port contract.

### 5.3 Per-opcode microbench (Task 4)

Verify each new snippet via `verify_opcodes_per_iter`. Run microbench, capture ns/dispatch + CI95. Compare to LLInt reference. Both must be within 2× LLInt.

### 5.4 Slow-path-share (Task 5)

Run `v8suite --count-slow-path-share` (or equivalent — discover via `--help`). Expected 0.00% for all 9 opcodes; gate is < 20%.

### 5.5 Behavioral parity at every commit

`cargo test -p lyng-vm --lib --release` (≥418), `cargo test -p lyng-tests --release` (≥1198).

### 5.6 Test262 (Task 5)

Re-run Test262 at phase-end HEAD. Assert ≥ 49729 passing (the umbrella mid-phase baseline). Capture pass count in the sub-phase summary.

### 5.7 Mandatory reviewer (Task 5)

`feature-dev:code-reviewer` over the full sub-phase commit range. Per the Phase 1.B.1 retrospective lesson, **the reviewer's task brief must explicitly include**: "Verify that runtime-dispatch coverage exists for any new backend macros — specifically, that the new `load_local_fixed!` and `store_local_fixed!` macros are exercised by REAL handlers (not just structural compiles-and-links tests)."

---

## 6. Same-load A/B methodology (post-audit lessons applied)

Per the cleanup batch lessons learned:

### Hard requirements (no rounding, no exceptions)

1. **≥ 11 samples** for any gate-bearing A/B (umbrella criteria 5 or 6).
2. **±20% loadavg overlap** at the changeover — strict, no rounding. If 21% measured, re-run.
3. **Both arms built fresh** at their respective HEAD (no incremental build cross-contamination).
4. **Both arms run within a single window** (< 60 min between start of first and end of second).

### Measurements required

1. **Same-load A/B vs immediate predecessor `08727f92`** — verifies no regression vs the cleanup-corrected baseline. Target: aggregate regression ≤ 2%; per-workload ≤ 5%.
2. **Direct cumulative A/B vs pre-DSL-0 `d850f261`** — verifies umbrella §1 criterion 5 (the +3% gate) at the cumulative level, not by composition. Target: ≥ +3% geomean.

The cumulative A/B doubles the bench time (4 builds + 2× 11-sample runs) but it's the definitive gate measurement. Better to run it once at phase close than rely on composed predictions.

### What's logged

Both A/B reports include:
- Per-workload medians (11 samples each)
- Per-workload deltas + geomean
- Loadavg at start + end of each arm
- Verdict (PASS / FAIL) against the explicit gate

---

## 7. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------:|-------:|-----------|
| Cumulative V8 v7 < +3% vs `d850f261` (gate fails) | medium | high | Mid-phase prediction is +3.4% (thin margin). 1.B.3 adds ~1.38B inlined dispatches vs 1.B.2's 360M (3.8× more) — should land +3-4% on this sub-phase alone. If sub-phase delta < +1%, investigate before declaring failure (could be loadavg artifact again). |
| LoadEnvSlot deferral changes the opcode mix from the umbrella's original 7 anchors | medium | low | The umbrella criterion 1 wording is "All 9 (up to 12 with pairs) opcodes ported" — 9 is the floor. We land 9 (6 anchors + 3 pairs), satisfying the floor. LoadEnvSlot becomes the focus of a follow-up substrate sub-phase, formally recorded in `phase-1b-followups.md`. The umbrella criterion is met; the *mix* is just different than originally enumerated. |
| New `*_fixed!` macros introduce a register-pin bug (analog to 1.B.1's x22→x24 issue) | low | medium | All 9 new handlers are runtime-dispatched via integration tests in Tasks 2-3. The cleanup-batch retrospective lesson is heeded: no structural-only validation. |
| Microbench snippets gap (some opcodes lack snippets) | medium | low | Task 4 includes "add missing snippets" as an explicit step. Trust `grep`, not summary tables (post-audit lesson). |
| Per-opcode microbench > 2× LLInt | low | medium | Inline path is 7 instructions total; LLInt's hand-written stub is similar. The 2× target is generous. If a specific opcode fails, investigate before declaring sub-phase failure. |
| 11-sample A/B still shows > 20% loadavg overlap (machine state is noisy) | medium | low | Schedule for overnight or quiet window; 15-sample contingency option. Document explicitly per cleanup-batch protocol. |
| Reviewer dispatches more findings than expected | low | medium | Phase 1.B.1's reviewer missed the x22→x24 bug — sub-phase reviewer brief explicitly calls out the need to verify runtime-dispatch coverage. |

---

## 8. Decisions made

1. **Defer `op_load_env_slot` to a future sub-phase.** Needs new `frame_lexical_env` substrate; does not fit the 1.B.3 mechanical-port scope. Documented in `phase-1b-followups.md`.
2. **Scope: 6 anchors + 3 macro-shared pairs = 9 opcodes.** No Star opcodes (different layout), no LoadLocal4-7 / StoreLocal4-7 (not top-30 and would require updated dispatch share data).
3. **Add two new tiny backend macros:** `load_local_fixed!(N => $dst)` and `store_local_fixed!($src, N)`. Single-instruction fixed-offset load/store from the register window. Eliminates need for `movz` to materialize fixed indices.
4. **Use existing macros for `op_load_local_0` (load_acc!) and `op_ldar` (load_reg!/store_acc!).** No new macro needed for these two.
5. **Hard methodology gates from audit lessons:** ±20% loadavg (no rounding), ≥11 samples, direct cumulative A/B at phase close (not composed prediction).
6. **Sub-phase phasing: 5 tasks, single refactor worker, 3-4 days.** Revised down from umbrella's 1.5-2 weeks (LoadEnvSlot deferral removes the substrate complexity).
7. **Reviewer brief explicitly demands runtime-dispatch coverage verification** for new backend macros. Lesson from Phase 1.B.1's structural-test miss.

---

## 9. References

- **Parent design:** [`docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md`](../../lyng/2026-05-16-asm-dsl-llint-interpreter-design.md) §10 DSL-1.
- **Phase 1.B umbrella spec:** [`2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md`](2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md).
- **Phase 1.B.2 spec (precedent):** [`2026-05-19-dsl-1-phase-1b2-backfill-ports-design.md`](2026-05-19-dsl-1-phase-1b2-backfill-ports-design.md).
- **Mid-phase umbrella summary:** [`reports/lyng/dsl-1/phase-1b-summary.md`](../../../reports/lyng/dsl-1/phase-1b-summary.md).
- **Test262 baseline:** [`reports/lyng/dsl-1/phase-1b-test262-baseline.md`](../../../reports/lyng/dsl-1/phase-1b-test262-baseline.md).
- **Followups doc:** [`reports/lyng/dsl-1/phase-1b-followups.md`](../../../reports/lyng/dsl-1/phase-1b-followups.md).
- **Phase 1.B.1 retrospective (structural-test lesson):** [`reports/lyng/dsl-1/phase-1b1-summary.md`](../../../reports/lyng/dsl-1/phase-1b1-summary.md) — "Retrospective: structural-only validation tests insufficient for substrate macros."
- **Top-30:** [`reports/lyng/r0/v8-v7-top30.tsv`](../../../reports/lyng/r0/v8-v7-top30.tsv).
- **Existing backend macros:** `crates/lyng/vm/src/dsl/backend/aarch64/operands.rs` (lines 34, 84, 106, 116, 126, 136).
- **Current cold stubs:** `crates/lyng/vm/src/dsl/handlers/cold.rs` (lines 342, 3959, 4255, 4284, 4313, 4342, 4371, 4400, 4429, 4458).
- **Semantic bodies:** `crates/lyng/vm/src/vm/semantics/loads.rs` (lines 322-333, 431-448, 483-495).
