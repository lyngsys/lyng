# Design: DSL-1 Phase 1.C — SMI arithmetic + bitwise

**Date:** 2026-05-21
**Status:** Design draft; awaiting user review.
**Parent design:** [`docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md`](../../lyng/2026-05-16-asm-dsl-llint-interpreter-design.md) §10 DSL-1.
**DSL-1 epic spec:** [`docs/superpowers/specs/2026-05-18-dsl-1-hot-opcode-rollout-design.md`](2026-05-18-dsl-1-hot-opcode-rollout-design.md) §2 row 1.C.
**Engine snapshot:** [`reports/lyng/asm-dsl-engine-state-2026-05-21.md`](../../../reports/lyng/asm-dsl-engine-state-2026-05-21.md).
**Predecessor:** Phase 1.B closed at `aa3ab9fc` with +8.51% V8 v7 cumulative vs pre-DSL-0 `d850f261`, 18 inline-ported opcodes, 49729 Test262 passing.

---

## 1. Goal, scope, exit criteria

### Goal

Port the seven SMI arithmetic + bitwise opcodes from cold-stub delegation to inline DSL fast paths, replicating the op_add shape proven in DSL-0 and validated in Phase 1.A. Lands the second-largest single chunk of dispatch share in DSL-1 (~1.75B inlined dispatches per V8 v7 run, vs Phase 1.B.3's 1.26B).

### Targets (7 opcodes)

| Opcode | Top-30 rank | Dispatches / V8 v7 run | Shape |
|--------|------------:|-----------------------:|-------|
| `op_mul` | #4 | 589M | Binary, overflow (smull+cmp) |
| `op_increment` | #5 | 541M | Unary, overflow, write-back to src reg |
| `op_shift_right` | #10 | 266M | Binary, no overflow |
| `op_decrement` | #23 | 99M | Unary, overflow, write-back to src reg |
| `op_bit_and` | #24 | 98M | Binary, no overflow |
| `op_shift_left` | #25 | 89M | Binary, no overflow |
| `op_sub` | #29 | 65M | Binary, overflow |

Combined: ~1.75B inlined dispatches per V8 v7 run.

### In scope

- Seven inline ports replacing cold stubs in `crates/vm/src/dsl/handlers/cold.rs`.
- Two new backend macros (`inc_smi_overflow!`, `dec_smi_overflow!`) under `crates/vm/src/dsl/backend/aarch64/arithmetic.rs` with `ops.md` entries.
- Per-opcode microbench, slow-path-share check, asm baseline, ported report under [`reports/lyng/dsl-handlers/`](../../../reports/lyng/dsl-handlers/) (per epic spec §3 8-step workflow).
- Per-sub-phase mini A/B (informational) and phase-close cumulative A/B vs pre-DSL-0 `d850f261` (umbrella gate).
- Updated `aarch64_max_instructions` budgets in `tools/lyng-bench/hot-opcodes.toml` for the 7 ports (currently 0 placeholders).
- Three sub-phase summaries + one phase summary + one followups doc under `reports/lyng/dsl-1/`.

### Out of scope

- `op_div`, `op_mod`, `op_exp` — no SMI fast path; always delegate. Not top-30.
- `op_bit_or`, `op_bit_xor`, `op_unsigned_shift_right` — macros exist (`bit_or_smi!`, `bit_xor_smi!`, `ushift_right_smi!`) but these opcodes are not top-30 and don't qualify under the strict top-30 + macro-shared-pair rule from Phase 1.B retrospective.
- SMI-immediate variants `op_sub_smi`, `op_mul_smi`, `op_bit_and_smi` — not top-30; defer to opportunistic future phase if their share rises.
- `op_negate`, `op_bit_not` — not top-30.
- LoadEnvSlot substrate sub-phase — deferred Phase 1.B.3 followup; pursued in its own sub-phase if scheduled before Phase 1.D.
- `asm-diff --check` namespace expansion to cover `dsl::handlers::cold::*` automatically — Phase 1.B followup; not blocking Phase 1.C.

### Exit criteria

1. All 7 opcodes have inline DSL fast paths with committed ported reports in [`reports/lyng/dsl-handlers/`](../../../reports/lyng/dsl-handlers/).
2. Asm baselines updated and committed; each handler within 5 instructions of LLInt's matching handler for its shape.
3. Per-opcode slow-path-share < 20% on V8 v7 (per-opcode waivers allowed; justified against LLInt-on-same-workload baseline in the ported report).
4. Behavioral parity: `cargo test -p lyng-vm -p lyng-tests` passes (currently 418 + 1209 tests); Test262 ≥ 49729 passing.
5. Cumulative V8 v7 geomean A/B vs `d850f261`: positive delta over Phase 1.B close (+8.51%). Re-baselined target: **+13% to +16% cumulative at Phase 1.C close**, explicitly documenting the gap vs the epic-spec ≥+35% target (which was projected from JSC LLInt scaling and assumed Phase 1.A would deliver ≥+5%; Phase 1.A actually delivered +1.7%). See §3 for re-baselining rationale.
6. Phase summary `reports/lyng/dsl-1/phase-1c-summary.md` + 3 sub-phase summaries + followups doc committed.

---

## 2. Sub-phase structure

Three sub-phases, grouped by asm shape. Each sub-phase has its own per-opcode gates and a mini A/B (informational, per Phase 1.B retrospective lesson #2). The phase-close cumulative A/B vs `d850f261` is the authoritative number.

### 1.C.0 — substrate prep (optional, ~1 day)

Add `inc_smi_overflow!` and `dec_smi_overflow!` macros under `arithmetic.rs`. Update `ops.md`. Self-review acceptable (mechanical addition; Phase 1.B retrospective lesson #3 requires runtime-dispatch verification, which 1.C.3 supplies).

May be absorbed into 1.C.3 if we want tighter sequencing — the macros are not needed before 1.C.1. The advantage of doing it as 1.C.0 is that 1.C.3 then becomes purely "use existing macros to land 2 ports", matching 1.C.1's mechanical shape.

### 1.C.1 — binary arith with overflow (~3-4 days)

Ports: `op_sub` (#29, 65M/run), `op_mul` (#4, 589M/run).

Inline shape (mirrors op_add):
```
decode operands a, b, c, slot
load_reg!(b => t0)              # lhs
check_smi!(t0, .slow)
load_reg!(c => t1)              # rhs
check_smi!(t1, .slow)
untag_smi!(t0)
untag_smi!(t1)
{sub,mul}_smi_overflow!(t0, t1 => t2, .slow)
tag_smi!(t2)
store_reg!(a, t2)               # dst
record_smi!(slot)
dispatch!()
.slow:
call_slow!(op_{sub,mul}_slow_rs, args = [a, b, c, slot])
dispatch_after_slow!()
```

**op_mul slow-path-share risk:** V8 v7 RayTrace and NavierStokes operate on doubles; the SMI fast path misses; slow path delegates to `vm.execute_mul_opcode`. Per epic spec §1 criterion 6 + per-opcode waiver protocol, the ported report documents per-workload share against an LLInt-on-same-workload baseline. If LLInt also shows >20% slow-path-share on float-heavy workloads, the waiver is justified.

Per-opcode order: **op_sub first** (mechanical mirror of op_add, lowest risk; validates the workflow for Phase 1.C), then **op_mul** (overflow-detection shape uses `smull+cmp` instead of `b.vs`; surfaces any LLVM rewrite-noise differences early).

Per-opcode gates per §5 below. Mini A/B vs Phase 1.B close at sub-phase close, informational.

### 1.C.2 — bitwise / shifts (~3 days)

Ports: `op_bit_and` (#24, 98M/run), `op_shift_left` (#25, 89M/run), `op_shift_right` (#10, 266M/run).

Inline shape (no overflow branch):
```
decode operands a, b, c, slot
load_reg!(b => t0)
check_smi!(t0, .slow)
load_reg!(c => t1)
check_smi!(t1, .slow)
untag_smi!(t0)
untag_smi!(t1)
{bit_and,shift_left,shift_right}_smi!(t0, t1 => t2)
tag_smi!(t2)
store_reg!(a, t2)
record_smi!(slot)
dispatch!()
.slow:
call_slow!(op_{bit_and,shift_left,shift_right}_slow_rs, args = [a, b, c, slot])
dispatch_after_slow!()
```

Shifts mask rhs to its low 5 bits per ECMAScript `<<` / `>>` semantics — the `shift_left_smi!` / `shift_right_smi!` macros already include the `and w16, wRhs, #0x1f` mask.

**Shift rhs SMI assumption:** ECMAScript shift coerces rhs to Uint32; if rhs is non-SMI at runtime, the fast path bails to slow which performs the coercion. V8 v7 shift sites typically use SMI literal rhs (Crypto's bit-twiddling, NavierStokes's array indexing) — expect low slow-path-share. Document per-opcode in the ported reports.

Per-opcode order: **op_bit_and first** (simplest, exercises bitwise-no-overflow shape), then **op_shift_left**, then **op_shift_right** (largest dispatch share — saves it for last to confirm the shape works on the highest-share workload).

### 1.C.3 — unary update (~3 days)

Ports: `op_increment` (#5, 541M/run), `op_decrement` (#23, 99M/run).

Inline shape (different from binary — single source operand, write-back semantics):
```
decode operands a (dst), b (src), c (unused), slot
load_reg!(b => t0)
check_smi!(t0, .slow)
untag_smi!(t0)
{inc,dec}_smi_overflow!(t0 => t1, .slow)
tag_smi!(t1)
store_reg!(a, t1)               # dst
# NOTE: src writeback skipped — when src is SMI, ToNumeric(src)==src,
# so the semantic's `write_register_unchecked(registers, args.src, numeric)`
# at vm/semantics/arithmetic.rs:825 is idempotent. The fast path elides it.
# The slow path handles non-SMI src and performs the writeback there.
record_smi!(slot)
dispatch!()
.slow:
call_slow!(op_{increment,decrement}_slow_rs, args = [a, b, c, slot])
dispatch_after_slow!()
```

**Fast-path SMI simplification — src writeback elided:** The semantic body (`op_update_register_semantic` in `vm/semantics/arithmetic.rs`) writes the ToNumeric-coerced source back to the src register before computing the result. For SMI src, the coercion is identity: `Value::from_smi(s).as_smi() == Some(s)`. The inline fast path can skip the src writeback because reading-then-writing the same SMI Value is observationally a no-op. The slow path still performs the writeback for non-SMI src (string, BigInt, Object with valueOf, etc.).

Per Phase 1.B retrospective lesson #3 ("structural compile-and-link tests are not sufficient for new substrate macros"), 1.C.3 includes a unit test that exercises a non-SMI src reaching the slow path, confirming the writeback still happens via the slow-path semantic. Spec for the test:

- Setup: function with `let s = "1"; let r = ++s;` (string src forces slow path).
- Assertion: after execution, `s === 1` (writeback) and `r === 2` (post-update value).
- Lives at `crates/tests/src/dsl_increment_writeback.rs` or similar.

If lyng doesn't yet support prefix `++` on a string lvalue, the test uses the most concise JS expression that compiles to `op_increment` with non-SMI src. The 1.C.3 worker confirms what compiles to op_increment before writing the test.

**Verification of the SMI elision claim:** the 1.C.3 worker reads `op_update_register_semantic` and confirms that the only effect of the writeback for SMI src is to write back an unchanged value. If the helper has any side effect we missed (e.g., feedback recording tied to the src register specifically), the elision is unsafe — the ported report documents the read of the helper and the conclusion.

Per-opcode order: **op_increment first** (largest share — confirms the shape), then **op_decrement** (mechanical mirror).

---

## 3. Re-baselining the epic-spec target

The DSL-1 epic spec §2 calls for cumulative ≥+35% V8 v7 geomean vs `d850f261` at Phase 1.C close. That curve was projected from JSC LLInt-style improvement scaling and assumed Phase 1.A would deliver ≥+5% solo. Phase 1.A actually delivered +1.7% (adjacent-family ports had negligible per-opcode dispatch share); Phase 1.B closed at +8.51% vs the epic-spec ≥+15% target.

The engine snapshot (`asm-dsl-engine-state-2026-05-21.md` §3) already flagged that the absolute targets may need re-baselining at Phase 1.C close based on actual delivered share. **Phase 1.C does that re-baselining now, not after the fact.**

**Re-baselined target for Phase 1.C close:** +13% to +16% cumulative geomean vs `d850f261`, proportional to the added dispatch share.

Rationale for the projection:
- Phase 1.B.3 added 1.26B inlined dispatches/run and lifted ~+4pp (from approx +4.5% at 1.B.2 close to +8.51% at 1.B.3 close).
- Phase 1.C adds ~1.75B inlined dispatches/run — proportional lift ≈ +5.5pp.
- Realistic cumulative: 8.51% + 4-7pp = **+12.5% to +15.5%** at Phase 1.C close.

Phase 1.C summary documents the actual number against both the epic-spec target (for visibility) and the re-baselined trajectory (for honest tracking). If the actual number lands above the re-baselined range, that's a positive surprise; if below, that's an off-ramp consideration (§7).

---

## 4. New substrate (1.C.0 or first 1.C.3 task)

Two new macros in `crates/vm/src/dsl/backend/aarch64/arithmetic.rs`. Each is 3 instructions:

```rust
/// 32-bit signed increment by 1 with overflow detection.
/// $src is an untagged SMI (sign-extended i32 in low 32 bits of an X-reg).
/// $dst receives the incremented value sign-extended to i64.
/// On overflow, branch to $label (slow path).
#[macro_export]
macro_rules! inc_smi_overflow {
    ($src:tt => $dst:tt, $label:tt) => {
        concat!(
            "adds   w", stringify!($dst), ", w", stringify!($src), ", #1\n",
            "b.vs   ", stringify!($label), "\n",
            "sxtw   x", stringify!($dst), ", w", stringify!($dst), "\n",
        )
    };
}

/// 32-bit signed decrement by 1 with overflow detection.
/// Overflow only at i32::MIN.
#[macro_export]
macro_rules! dec_smi_overflow {
    ($src:tt => $dst:tt, $label:tt) => {
        concat!(
            "subs   w", stringify!($dst), ", w", stringify!($src), ", #1\n",
            "b.vs   ", stringify!($label), "\n",
            "sxtw   x", stringify!($dst), ", w", stringify!($dst), "\n",
        )
    };
}
```

AArch64 `adds` / `subs` accept a 12-bit unsigned immediate (`#1` is well within range), no scratch register needed. Each macro emits exactly 3 instructions: `adds`/`subs` + `b.vs` + `sxtw`.

Alternative considered and rejected: compose from existing `add_smi_overflow!(src, w_scratch_with_1 => dst, slow)`. Costs 1 extra instruction per dispatch to materialize 1 in scratch (via `mov w17, #1` or similar) — on 640M dispatches/V8v7 run that's 640M wasted instructions. The 50-line macro pair is well worth it.

`ops.md` entries added under the arithmetic section.

Per Phase 1.B retrospective lesson #3, substrate macros need runtime-dispatch verification immediately. 1.C.3's inline ports of op_increment/op_decrement *are* the runtime verification — no separate "structural" compile-and-link test like 1.B.1's `assert_handler_symbol_exists` (which Phase 1.B.2 retrospectively found insufficient).

---

## 5. Per-opcode gates

Standard gates carried forward from Phase 1.B umbrella §3, applied to each of the 7 opcodes before the worker subagent commits:

| Gate | Criterion | Source |
|------|-----------|--------|
| Behavioral | `cargo test -p lyng-vm -p lyng-tests` passes | Existing suites (418 + 1209 tests) |
| Asm shape | Within 5 instructions of LLInt's matching handler for its shape; ≤12 inline instructions per handler (epic spec §4 budget table — current placeholders are 0 in `hot-opcodes.toml`) | Per-opcode ported report quantifies the delta |
| Microbench | ns/dispatch within 2× of JSC LLInt's matching opcode, isolated, 7-sample median | `lyng-bench microbench --opcodes <name>` |
| Slow-path-share | < 20% on V8 v7; per-opcode waivers allowed with workload-mix justification against LLInt-on-same-workload baseline (op_mul on float-heavy workloads is the most likely waiver candidate) | `lyng-bench v8suite --count-slow-path-share` |
| Asm baseline | Updated and committed; passes `asm-diff --check` (note: `asm-diff --check` currently doesn't auto-discover `dsl::handlers::cold::*` — manual capture per Phase 1.B.2/1.B.3 precedent until the followup lands) | `lyng-bench asm-diff` |
| Ported report | DSL source, current asm, LLInt reference, side-by-side diff, microbench, slow-path-share | `reports/lyng/dsl-handlers/op_<name>.md` |
| `hot-opcodes.toml` budget | Calibrated `aarch64_max_instructions` for the opcode (measured + 2 headroom) | `tools/lyng-bench/hot-opcodes.toml` |

If a worker can't satisfy any gate, it reports back rather than commits.

---

## 6. A/B protocol

Per Phase 1.B retrospective lessons:

1. **11+ samples per A/B** (lesson #1). Loadavg overlap < ±20% — abort and retry if exceeded.
2. **Per-sub-phase mini A/B is informational** (lesson #2). Sub-phase A/Bs compose roughly but not authoritatively; the phase-close cumulative A/B is the authoritative number.
3. **Phase-close cumulative A/B vs `d850f261`** is the umbrella gate. 11+ samples, loadavg-overlap-checked, all 6 workloads + geomean.

Sub-phase A/Bs use the same loadavg-overlap protocol but with fewer samples (7) acceptable since they're informational.

Phase-close A/B artifact lives at `reports/lyng/dsl-1/phase-1c-cumulative-ab.md`.

---

## 7. Off-ramp triggers

Per DSL-1 epic spec §2 + Phase 1.B umbrella §1:

1. **5+ consecutive opcode ports fail per-opcode gates** — pause Phase 1.C; coordinator writes diagnostic at `reports/lyng/dsl-1/off-ramp-2026-MM-DD-phase-1c.md`. Decision: deepen scope (e.g., new substrate to address the failure pattern), defer affected opcodes to Phase 1.D or later, or close DSL-1 with banked wins at Phase 1.B close + whatever 1.C delivered.
2. **op_mul slow-path-share > 20% on multiple V8 v7 workloads** — document per-workload waiver against LLInt baseline; do *not* abort if LLInt-on-same-workload also shows similar share (the threshold is about our fast path matching LLInt's, not absolute share).
3. **Microbench ratio > 2× LLInt on ≥3 opcodes** — pause; the substrate may be limiting more than the ports. Coordinator investigates: is it the `record_smi!` cost? The check_smi overhead? Surface findings to a substrate sub-phase or accept and move on with documented justification.
4. **Cumulative A/B at phase close lands below +9% (i.e., negative delta from Phase 1.B close)** — abort and investigate. Negative cumulative delta from a phase that adds 1.75B inlined dispatches/run signals a regression somewhere (likely substrate cost or recent rust upgrade noise).

Phase 1.C close decision options (per off-ramp protocol): proceed to Phase 1.D, defer remaining DSL-1 phases to a follow-up epic, or close DSL-1 here with banked wins documented.

---

## 8. Risks

Deltas from DSL-1 epic spec §6:

| Risk | Likelihood | Impact | Mitigation |
|------|-----------:|-------:|-----------|
| `op_mul` slow-path-share > 20% on float-heavy workloads (RayTrace, NavierStokes) | medium | medium | Per-workload share breakdown in ported report; document LLInt-on-same-workload baseline; waiver protocol per epic spec §1 criterion 6 |
| `op_increment`/`op_decrement` SMI-elision-of-src-writeback subtlety wrong | low | medium | 1.C.3 worker reads `op_update_register_semantic` (vm/semantics/arithmetic.rs:796-833) and documents the conclusion in the ported report. Unit test exercises non-SMI src reaching slow path — confirms writeback still happens |
| Shift rhs not SMI in V8 v7 workloads (esp. NavierStokes array indexing) | low | low | Fast path bails to slow; document per-workload share in ported report. Most V8 v7 shift sites use SMI literals |
| Cumulative A/B disappoints vs epic-spec ≥+35% | high | low | §3 re-baselining explicitly handles this. Document the gap openly in the phase summary |
| Two new substrate macros (`inc_smi_overflow!`, `dec_smi_overflow!`) introduce a latent register-pin bug like Phase 1.B.1's | low | medium | Per lesson #3, runtime verification immediately via 1.C.3 inline ports. Sub-phase 1.C.3 budget includes time to investigate any handler-dispatch failure |
| LLVM/rustc rewrite noise on `smull+cmp` overflow check (op_mul) | low | low | Phase 1.B.0 microbench budget headroom (`+2` instructions) covers this; if op_mul lands at >budget, investigate before committing |
| `asm-diff --check` namespace gap continues to mask drift (Phase 1.B followup) | medium | low | Manual baseline capture per Phase 1.B.2/1.B.3 precedent. Tracked in `phase-1b-followups.md` |

Pre-existing risks from epic spec §6 (cells refactor, IC mode-byte, call-frame transition) are out of Phase 1.C scope; remain in Phase 1.E/1.F/1.G.

---

## 9. Coordinator workflow

Per Phase 1.B umbrella §3 (proven workflow):

1. **Sub-phase brainstorm** (this spec) → produces sub-phase design specs only if 1.C.0/1.C.3 substrate adds surface a non-mechanical change. The three sub-phases under Phase 1.C are mechanical enough that an internal task list under this spec covers them; sub-phase specs are not required.
2. **`/superpowers:writing-plans`** → produces `docs/superpowers/plans/2026-MM-DD-dsl-1-phase-1c-smi-arith-and-bitwise-plan.md` covering all three sub-phases.
3. **Sub-phase execution** via subagent dispatch:
   - One refactor-worker subagent per port (or per 2-3 tightly-coupled ports in the case of 1.C.2's bitwise group, if shape coupling justifies).
   - Coordinator handles A/Bs at the coordinator level (bench is long-running; coordinator-level loadavg awareness).
   - `feature-dev:code-reviewer` dispatched optionally — Phase 1.B.3 used self-review for mechanical ports; Phase 1.C ports are likewise mechanical except for the inc/dec writeback subtlety which warrants reviewer dispatch on 1.C.3.
4. **Sub-phase close:** mini A/B (informational), Test262 spot-check (≥ baseline), behavioral parity check, sub-phase summary, followups recording.
5. **Phase close:** direct cumulative A/B vs `d850f261`, Test262 final check, phase summary update.

User deny rules continue: no `git -C`, no `cd && git`, no `--no-verify`, no destructive ops without consent.

---

## 10. Deliverables checklist

- [ ] 7 new inline DSL handler implementations replacing existing cold stubs in `crates/vm/src/dsl/handlers/cold.rs`.
- [ ] 2 new backend macros (`inc_smi_overflow!`, `dec_smi_overflow!`) in `crates/vm/src/dsl/backend/aarch64/arithmetic.rs` + `ops.md` entries.
- [ ] 7 ported reports in `reports/lyng/dsl-handlers/op_{sub,mul,bit_and,shift_left,shift_right,increment,decrement}.md`.
- [ ] 7 asm baselines in `reports/lyng/dsl-asm-baseline-aarch64/`.
- [ ] 1 unit test for inc/dec non-SMI-src writeback at `crates/tests/src/dsl_increment_writeback.rs` (or equivalent location).
- [ ] Updated `tools/lyng-bench/hot-opcodes.toml` with calibrated `aarch64_max_instructions` budgets for the 7 ports (replacing 0 placeholders).
- [ ] 3 sub-phase summaries at `reports/lyng/dsl-1/phase-1c{1,2,3}-summary.md`.
- [ ] 1 phase summary at `reports/lyng/dsl-1/phase-1c-summary.md` with re-baselining commentary per §3.
- [ ] 1 phase-close cumulative A/B artifact at `reports/lyng/dsl-1/phase-1c-cumulative-ab.md`.
- [ ] 1 followups doc at `reports/lyng/dsl-1/phase-1c-followups.md` (or appended to `phase-1b-followups.md` if items are small).
- [ ] Updated engine state snapshot at `reports/lyng/asm-dsl-engine-state-<date>.md` (post-Phase 1.C close).

---

## 11. References

### Design docs

- Parent design: [`docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md`](../../lyng/2026-05-16-asm-dsl-llint-interpreter-design.md) §10 DSL-1.
- DSL-1 epic spec: [`docs/superpowers/specs/2026-05-18-dsl-1-hot-opcode-rollout-design.md`](2026-05-18-dsl-1-hot-opcode-rollout-design.md) §2 row 1.C.
- Phase 1.B umbrella: [`docs/superpowers/specs/2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md`](2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md).

### Engine state at Phase 1.B close

- Engine snapshot: [`reports/lyng/asm-dsl-engine-state-2026-05-21.md`](../../../reports/lyng/asm-dsl-engine-state-2026-05-21.md).
- Phase 1.B umbrella summary: [`reports/lyng/dsl-1/phase-1b-summary.md`](../../../reports/lyng/dsl-1/phase-1b-summary.md).
- Phase 1.B followups: [`reports/lyng/dsl-1/phase-1b-followups.md`](../../../reports/lyng/dsl-1/phase-1b-followups.md).

### Predecessor port (the SMI shape prototype)

- op_add ported report: [`reports/lyng/dsl-handlers/op_add.md`](../../../reports/lyng/dsl-handlers/op_add.md).
- op_add handler source: [`crates/vm/src/dsl/handlers/hot.rs`](../../../crates/vm/src/dsl/handlers/hot.rs) lines 38-72.

### Substrate

- Arithmetic backend macros: [`crates/vm/src/dsl/backend/aarch64/arithmetic.rs`](../../../crates/vm/src/dsl/backend/aarch64/arithmetic.rs).
- Value tag macros: [`crates/vm/src/dsl/backend/aarch64/values.rs`](../../../crates/vm/src/dsl/backend/aarch64/values.rs).
- Feedback macros: [`crates/vm/src/dsl/backend/aarch64/feedback.rs`](../../../crates/vm/src/dsl/backend/aarch64/feedback.rs).
- Operand decode: [`crates/vm/src/dsl/backend/aarch64/operands.rs`](../../../crates/vm/src/dsl/backend/aarch64/operands.rs).
- Backend ops vocab: [`crates/vm/src/dsl/ops.md`](../../../crates/vm/src/dsl/ops.md).

### Semantic bodies

- Arithmetic semantics: [`crates/vm/src/vm/semantics/arithmetic.rs`](../../../crates/vm/src/vm/semantics/arithmetic.rs).
  - `op_sub_semantic` lines 249-281.
  - `op_mul_semantic` lines 283-313.
  - `op_bit_and_semantic` lines 537-567.
  - `op_shift_left_semantic`, `op_shift_right_semantic` lines 617-629 (delegate via `op_binary_general`).
  - `op_update_register_semantic` lines 796-833 (shared by Increment/Decrement; informs the SMI-elision claim in 1.C.3).

### Current cold-stub handlers (to replace)

- `op_sub_dsl` at `crates/vm/src/dsl/handlers/cold.rs:1050`.
- `op_mul_dsl` at `crates/vm/src/dsl/handlers/cold.rs:1120`.
- `op_bit_and_dsl` at `crates/vm/src/dsl/handlers/cold.rs:1435`.
- `op_shift_left_dsl`, `op_shift_right_dsl` (locate during 1.C.2 task 1).
- `op_increment_dsl` at `crates/vm/src/dsl/handlers/cold.rs:1678`.
- `op_decrement_dsl` at `crates/vm/src/dsl/handlers/cold.rs:1712`.

### Measurement infrastructure

- Top-30 dispatch shares: [`reports/lyng/r0/v8-v7-top30.tsv`](../../../reports/lyng/r0/v8-v7-top30.tsv).
- Hot-opcodes config: [`tools/lyng-bench/hot-opcodes.toml`](../../../tools/lyng-bench/hot-opcodes.toml).
- Bench tool: [`tools/lyng-bench/`](../../../tools/lyng-bench/) (microbench, asm-diff, v8suite, count-slow-path-share, require-isolation).

### JSC LLInt references

- `op_sub`, `op_mul`, `op_inc`, `op_dec`, `op_bitand`, `op_lshift`, `op_rshift` in `/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter64.asm`.
- Existing captures at `reports/lyng/llint-reference/` if present; capture via `lyng-bench capture-llint` if not.

### Engineering standards

- [`AGENTS.md`](../../../AGENTS.md), [`crates/AGENTS.md`](../../../crates/AGENTS.md), [`docs/lyng/engineering-standards.md`](../../lyng/engineering-standards.md).
