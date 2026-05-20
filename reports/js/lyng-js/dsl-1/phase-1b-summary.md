# DSL-1 Phase 1.B — mid-phase umbrella summary

**Date:** 2026-05-20
**Phase status:** IN PROGRESS — 3 of 4 sub-phases closed; Phase 1.B.3 pending.
**Range:** baseline commit `b680752e` (Phase 1.A end state) → current HEAD `db2d05db` (cleanup batch 2 close).
**Predecessor (pre-Phase 1.B):** Phase 1.A end state at `b680752e` (per [`phase-1a-summary.md`](phase-1a-summary.md)).
**Pre-DSL-0 baseline (epic-level reference):** `d850f261` (per [`pre-phase-1a-baseline.md`](pre-phase-1a-baseline.md)).

> **Why this doc exists.** Sub-phase summaries (1b0, 1b1, 1b2) live
> separately, each scoped to a single sub-phase. Post-Phase-1.B.2
> audit (2026-05-20) flagged the absence of an umbrella-level
> summary as a drift finding: cumulative state had never been
> measured or documented; the umbrella §1 criterion 5 (V8 v7
> cumulative ≥ +3% vs pre-DSL-0 `d850f261`) had only been computed
> per-sub-phase, never cumulatively. This doc closes that gap mid-
> phase so Phase 1.B.3 isn't the first time cumulative trajectory
> is computed.

## Sub-phase progress

| Sub-phase | Status | HEAD | Summary |
|-----------|--------|------|---------|
| 1.B.0 (counter wiring + microbench infra) | ✅ closed | `ae8b7766` | [`phase-1b0-summary.md`](phase-1b0-summary.md) |
| 1.B.1 (frame-context substrate) | ✅ closed | `4ff25b9b` | [`phase-1b1-summary.md`](phase-1b1-summary.md) |
| 1.B.2 (op_load_const8 + op_load_this inline ports) | ✅ closed | `7baf5846` | [`phase-1b2-summary.md`](phase-1b2-summary.md) |
| Cleanup batch 1 (audit drift findings #1-#3, #6, #7) | ✅ closed | `2cb027b0` | Commits 7baf5846..2cb027b0 |
| Cleanup batch 2 (audit drift findings #4, #5; umbrella doc) | ✅ closed | `db2d05db` | This doc + commits 2cb027b0..db2d05db |
| 1.B.3 (locals + Ldar + LoadEnvSlot inline ports) | ⏳ pending | — | Brainstorm not yet started |

## Phase 1.B umbrella §1 exit criteria — status

The Phase 1.B umbrella spec at
[`docs/superpowers/specs/2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md`](../../../docs/superpowers/specs/2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md)
§1 lists 8 phase-wide exit criteria. Current state:

| # | Criterion | Status | Reference |
|--:|-----------|:------:|-----------|
| 1 | All 9-12 opcodes ported | ⏳ 2 of 9 (op_load_const8 + op_load_this); 7+ pending 1.B.3 | [phase-1b2-summary.md](phase-1b2-summary.md) |
| 2 | Counter infra (10.A) sane (Move ≈ 4.66B on Richards) | ✅ verified in 1.B.0 (within 0.2%) | [phase-1b0-summary.md](phase-1b0-summary.md) |
| 3 | Microbench (10.B) produces CI95 for all 14 in-scope opcodes | ✅ 16 snippets verified (14 original + 2 added in cleanup batch 1) | [phase-1b0-summary.md](phase-1b0-summary.md) + cleanup commit `922ff5f2` |
| 4 | Frame-context refactor: behavioral parity, Test262 ≥ baseline, gc-stress clean | ✅ behavioral + gc-stress in 1.B.1; Test262 baseline captured in cleanup batch 2 (49729 passing) | [phase-1b1-summary.md](phase-1b1-summary.md) + [phase-1b-test262-baseline.md](phase-1b-test262-baseline.md) |
| 5 | V8 v7 cumulative ≥ +3% vs pre-DSL-0 HEAD `d850f261` | ⚠ predicted ~+3.4% (composed from per-sub-phase deltas; not directly measured against `d850f261`) — see §"Cumulative V8 v7 state" below | This doc + sub-phase A/Bs |
| 6 | No workload regresses > 2% vs pre-Phase-1.B HEAD `b680752e` | ⚠ predicted clean (composed); not directly measured — see §"Cumulative V8 v7 state" below | Sub-phase A/Bs |
| 7 | Per-opcode slow-path-share < 20% on V8 v7 | ✅ for the 2 ported (both 0.00%); 7+ pending 1.B.3 | [phase-1b2-microbench.md](phase-1b2-microbench.md) |
| 8 | Per-opcode microbench within 2× LLInt reference | ✅ for the 2 ported (LoadConst8 36.34 ns, LoadThis 36.52 ns post-cleanup; both within budget); 7+ pending 1.B.3 | [phase-1b2-microbench.md](phase-1b2-microbench.md) + cleanup commit `4c20e775` |

**Closed sub-phase summary:** 4 of 8 criteria ✅ for closed work;
2 ✅ for the 2 ported opcodes (slow-path-share + microbench);
2 ⚠ predicted from per-sub-phase composition (cumulative V8 v7
gates). Phase 1.B.3 closes the remaining work and produces the
direct cumulative measurement.

## Cumulative V8 v7 state

### Per-sub-phase A/Bs (all under same-load A/B protocol)

| Sub-phase | A/B against | Geomean delta | Notes |
|-----------|-------------|--------------:|-------|
| 1.B.0 close `ae8b7766` | Pre-1.B `b680752e` | ~0% (≈ +0.1%) | Infra-only; expected | 
| 1.B.1 close `4ff25b9b` | 1.B.0 close `ae8b7766` | +0.80% | Substrate-only; no handler exercise |
| 1.B.2 close (re-run, 11-sample) `2cb027b0` | 1.B.1 close `68dd5e89` | **+0.91%** (revised from original +4.89%) | 2 inline ports; original A/B had 21% loadavg overlap and substantially overstated the effect — see [`phase-1b2-ab-comparison.md`](phase-1b2-ab-comparison.md) |

### Composition vs pre-DSL-0 HEAD `d850f261`

The umbrella §1 criterion 5 says **V8 v7 cumulative ≥ +3% vs
pre-DSL-0 HEAD `d850f261`**. This has NOT been directly measured at
the cumulative level. The per-sub-phase deltas above can be composed
multiplicatively to predict the cumulative value at the current HEAD.

Composition chain (each link is a measured A/B):

1. Phase 1.A close `b680752e` vs pre-DSL-0 `d850f261`: **+1.7%** (from [`phase-1a-summary.md`](phase-1a-summary.md))
2. 1.B.0 close `ae8b7766` vs Phase 1.A close `b680752e`: **~0%** (infra-only — [`phase-1b0-summary.md`](phase-1b0-summary.md))
3. 1.B.1 close `4ff25b9b` vs 1.B.0 close `ae8b7766`: **+0.80%** ([`phase-1b1-ab-comparison.md`](phase-1b1-ab-comparison.md))
4. 1.B.2 close `7baf5846` vs 1.B.1 close `68dd5e89`: **+0.91%** (11-sample re-run; [`phase-1b2-ab-comparison.md`](phase-1b2-ab-comparison.md))

Multiplicative cumulative: `(1.017) × (1.000) × (1.0080) × (1.0091) = 1.0344`

**Predicted cumulative V8 v7 geomean improvement vs pre-DSL-0
`d850f261`: ~+3.4%.**

### How robust is this prediction?

**Caveats:**

1. The composition assumes per-sub-phase deltas multiply cleanly.
   They will to a first approximation because each is a geomean of
   the same 6 workloads, but **measurement noise can compound** —
   the 1.B.0 "~0%" actually has CI95 that could be ±0.5%; the 1.B.1
   +0.80% sits inside its own CI95 of similar width; the 1.B.2
   re-run +0.91% has CI95 of about ±2% (per-workload CIs visible in
   the A/B report). Cumulative CI95 by quadrature is ~±2.2 pp,
   giving a 95%-confidence range of roughly **+1.2% to +5.6%**.
2. The prediction does NOT include phase substrate side-effects
   that may appear only when measured cumulatively (e.g., I-cache
   interactions between newly-inlined handlers and the rest of the
   bytecode loop). The 11-sample re-run was the first 1.B.2 A/B at
   loadavg-overlap-within-protocol; a similar +0.91% may be optimistic
   or pessimistic if conditions have shifted on the actual cumulative
   measurement.
3. **Phase 1.B.2's original A/B revision is the load-bearing change.**
   The original +4.89% would have placed the cumulative prediction
   at ~+7.5%, comfortably clear of the +3% gate. The revised +0.91%
   places it at ~+3.4% — *just* clear of the gate. Phase 1.B.3's
   inline ports must contribute meaningfully to maintain headroom.

### What 1.B.3 should target

Per [`docs/superpowers/specs/2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md`](../../../docs/superpowers/specs/2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md) §1,
1.B.3 lands 7 top-30 opcode ports (`op_load_local_0/1/2/3`,
`op_store_local_3`, `op_load_env_slot`, `op_ldar`) representing
~1.38B dispatches/run combined — by far the biggest dispatch
contribution of any 1.B sub-phase.

Dispatch share scaling (anchored on Phase 1.A's measured +1.7% over
7 ports with ~388M LoadSmi8 + 6 sub-1% adjacent-family ports):
roughly 1% per ~100M dispatches for a high-share opcode in a
favorable position. 7 ports × ~200M average → ~13% theoretical
ceiling, with 30-50% capture = +4-6% actual.

If 1.B.3 lands +4% (conservative) on top of the current ~+3.4%
cumulative, **total cumulative would be ~+7.5% vs `d850f261`** —
comfortably above the +3% gate.

### Direct measurement to be performed at 1.B.3 close

Phase 1.B.3 closure must:
1. Run same-load A/B post-1.B.3 HEAD vs pre-DSL-0 `d850f261`.
2. Confirm cumulative ≥ +3% (umbrella criterion 5).
3. Run same-load A/B post-1.B.3 HEAD vs pre-Phase-1.B `b680752e`.
4. Confirm no workload regresses > 2% (umbrella criterion 6).

The composition above is the predicted target; the direct
measurement is the authoritative one.

## Cleanup batches (post-audit realignment)

The audit performed on 2026-05-20 identified seven drift findings,
addressed across two cleanup batches:

| # | Finding | Resolution | Commit |
|--:|---------|-----------|--------|
| 1 | 1.B.0 microbench snippets gap (LoadConst8 + LoadThis absent from the originally-stated 14) | Snippets backfilled | `922ff5f2` |
| 2 | Deferred 1.B.2 microbench gate (no numbers for LoadConst8 / LoadThis ports) | Filled in with real numbers (LoadConst8 36.34 ns, LoadThis 36.52 ns) | `4c20e775` |
| 3 | 1.B.0 summary's "14 opcodes" framing implied LoadConst8 + LoadThis were among them; 1.B.1 retrospective on structural-only validation tests insufficient for substrate macros | 1.B.0 summary corrected; 1.B.1 retrospective documented | `323dc4f0` |
| 6 | `ThisState::Uninitialized` JS coverage gap | Pinned to [`phase-1b-followups.md`](phase-1b-followups.md) | `2cb027b0` |
| 7 | asm-diff registry doesn't cover dsl::handlers::cold::* | Pinned to [`phase-1b-followups.md`](phase-1b-followups.md) | `2cb027b0` |
| 4 | 1.B.2 A/B loadavg overlap 21% (just outside ±20% protocol) | Re-ran with 11 samples + cleaner loadavg overlap (13%); **revised headline from +4.89% to +0.91% geomean** — original was load-base-depressed | `78e25a6b` |
| 5 | Test262 baseline never captured at umbrella level | Captured: 49729 passing / 0 failing | `db2d05db` |

**Cleanup batch 1 (4 commits, `922ff5f2..2cb027b0`):** addressed
findings #1, #2, #3, #6, #7 — all documentation / test-only changes
(no observable runtime behavior changes).

**Cleanup batch 2 (3 commits, `78e25a6b..db2d05db` including this
doc):** addressed findings #4, #5, and produced this umbrella summary.

## Behavioral parity at current HEAD

`cargo test -p lyng-js-vm --lib --release`: **418 passing** ✓ (matches Phase 1.B.2 close baseline)
`cargo test -p lyng-js-tests --release`: **1198 passing** ✓ (matches Phase 1.B.2 close baseline)

Test262 (cleanup batch 2): **49729 passing files / 0 failing / 100.00% rate** ✓
(see [`phase-1b-test262-baseline.md`](phase-1b-test262-baseline.md);
+1 file vs pre-DSL-0 `d850f261` baseline of 49728/1).

Per-handler reports:
- [`reports/js/lyng-js/dsl-handlers/op_load_const8.md`](../dsl-handlers/op_load_const8.md) ✓
- [`reports/js/lyng-js/dsl-handlers/op_load_this.md`](../dsl-handlers/op_load_this.md) ✓

Per-handler asm baselines:
- [`reports/js/lyng-js/dsl-asm-baseline-aarch64/op_load_const8.asm`](../dsl-asm-baseline-aarch64/op_load_const8.asm) ✓
- [`reports/js/lyng-js/dsl-asm-baseline-aarch64/op_load_this.asm`](../dsl-asm-baseline-aarch64/op_load_this.asm) ✓

## Lessons / observations (Phase 1.B umbrella level)

1. **The ±20% loadavg-overlap A/B protocol is a hard gate, not a
   soft one.** The original 1.B.2 A/B sat at 21% overlap — 1
   percentage point past the threshold — and the 11-sample re-run
   revealed the original A/B overstated the geomean delta by ~4×
   (revised from +4.89% to +0.91%). Future A/Bs should treat the
   ±20% threshold as failure-mode: re-run with more samples or
   wait for cleaner load conditions.
2. **Substrate-only sub-phases don't materially move V8 v7 numbers,
   but the framework SHOULDN'T claim they're moving them.** Phase
   1.B.1 reported +0.80% and called it "substrate noise"; Phase
   1.B.2 reported +4.89% and ascribed it to "the substrate was
   well-shaped" — the revised +0.91% reveals that the substrate
   shaping continued to dominate, with the inline ports adding only
   minor incremental gains on top. The substrate work's payoff
   appears more in *enabling future ports* than in immediate V8
   v7 movement.
3. **Microbench snippet coverage drift is silent.** The Phase 1.B.0
   summary table said "14 in-scope opcodes (7 Phase-1.A + 7 Phase-
   1.B anchors)" without specifying which 14 — and LoadConst8 +
   LoadThis were absent. The gap was caught only when Phase 1.B.2
   tried to use the gate. Lesson: **trust grep, not summary tables.**
   For sub-phases that depend on infra produced by prior sub-phases,
   the dependency should be cross-checked at the start of the
   dependent sub-phase, not at its end.
4. **Test262 deferral compounds across sub-phases.** Each sub-phase
   1.B.0/1.B.1/1.B.2 individually said "no semantic surface touched,
   defer Test262". That was reasonable in isolation, but the
   cumulative state was never measured until cleanup batch 2.
   For future epics: the umbrella gate's "≥ baseline" criterion
   should be measured at every sub-phase close OR explicitly
   batched into a mid-phase checkpoint when no semantic surface is
   touched. Don't compound deferral across sub-phases without an
   explicit checkpoint.
5. **Cumulative V8 v7 trajectory needs explicit composition, not
   per-sub-phase reporting alone.** Each sub-phase reported its
   own A/B; the cumulative-vs-d850f261 number was never composed
   until this doc. Phase 1.B.3 closure should perform the direct
   measurement and report it explicitly.

## Decision

**Phase 1.B is healthy but with thinner cumulative headroom than the
sub-phase A/Bs suggested.** The cleanup batches realigned all known
drift findings. The revised Phase 1.B.2 A/B reveals the cumulative
trajectory is roughly **+3.4% vs pre-DSL-0** — just above the +3%
gate, with the heaviest dispatch-share contribution (1.B.3's locals)
still ahead.

Phase 1.B.3 can proceed. Recommended next step:
`/superpowers:brainstorming` for Phase 1.B.3 (locals + Ldar +
LoadEnvSlot inline ports). Per the umbrella spec §1, 1.B.3 should
land 7 top-30 anchors (LoadLocal0/1/2/3, StoreLocal3, LoadEnvSlot,
Ldar) plus macro-shared symmetric pairs under the 15-min rule.

The Phase 1.B.3 brief should emphasize:

1. Direct cumulative A/B at phase close (vs pre-DSL-0 `d850f261`)
   — confirm +3% gate empirically, not just by composition.
2. Test262 at phase close (≥ 49729 passing files vs this baseline).
3. Tight loadavg-overlap discipline on every A/B (≤ 20% absolute,
   or larger sample sizes if not achievable).
4. Honest reporting if any port lands a smaller-than-projected
   improvement; that's a real finding, not a methodological failure.

## Commits in Phase 1.B (cumulative)

30 commits between `b680752e` (pre-Phase 1.B) and `db2d05db`
(current HEAD).

| Sub-phase | Commits |
|-----------|--------:|
| 1.B.0 | 8 commits + 1 summary commit (9 total) |
| 1.B.1 | 9 commits + 1 summary commit (10 total) |
| 1.B.2 | 4 task commits + 1 summary commit (5 total) |
| Cleanup batch 1 | 4 commits (snippets + microbench fill + summary corrections + followup pinning) |
| Cleanup batch 2 | 2 commits + this summary (3 total) |

Total: **30 commits over Phase 1.B so far.** Phase 1.B.3 will add
the locals + Ldar + LoadEnvSlot ports + summary + a phase-close
direct cumulative A/B.
