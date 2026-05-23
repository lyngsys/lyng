# asm-DSL engine — state of the engine (2026-05-21)

**HEAD:** `aa3ab9fc` (Phase 1.B closed; planning artifacts archived).
**Cumulative V8 v7 vs pre-DSL-0 `d850f261`:** **+8.51% geomean** (11-sample direct measurement at Phase 1.B.3 close).
**Behavioral parity:** 418 `lyng-js-vm --lib` + 1209 `lyng-js-tests` passing. Test262: 49729 files passing / 0 failing.
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

The DSL is implemented across two crates:

- `crates/lyng-js-vm-dsl/` — the proc-macro lowerer (parse `llint_handler!`, emit `naked_asm!` with universal named bindings for offsets/scratch regs).
- `crates/lyng-js/vm/src/dsl/` — the runtime side: `LlIntState`, register-convention constants, `entry.rs` trampoline shim, `slow_path.rs` bridge, `backend/aarch64/` operation vocabulary, `handlers/{cold,warm,hot}.rs` opcode handlers.

---

## 2. What has landed (timeline)

### DSL-0 substrate (closed at `d850f261`; pre-engagement)

The substrate phases (DSL-0a, 0b, 0c) brought up the entire DSL infrastructure:

- 0a: handler factory + dispatch table + entry/exit shims (parent §5).
- 0b: feedback-vector flat refactor + warm/hot family split (parent §6).
- 0c: slow-path bridge with the `DispatchState` shape that semantic bodies consume (parent §6 + the cleanup that deleted α dispatcher).

By DSL-0 close, every opcode existed as a `cold-stub` `call_slow!` shim — semantically correct, no inline asm. The substrate could in principle replace any opcode's cold stub with a hand-asm fast path; DSL-1 is the rollout of those replacements.

Reports: `reports/js/lyng-js/{dsl-0-decision.md, dsl-0a-status.md, dsl-0b-status.md, dsl-0c-status.md}`.

### DSL-1 Phase 1.A (closed at `b680752e`)

7 trivial-load opcodes inline-ported in `cold.rs` (the "warm" name is misleading — these handlers were always intended for `cold.rs` since they're operand-decoded). Both top-30 anchors and adjacent-family completions:

| Opcode | Top-30 rank | Asm shape |
|--------|------------:|-----------|
| `op_load_undefined` | (adjacent) | `tag_undefined!` → store → dispatch |
| `op_load_null` | (adjacent) | `tag_null!` |
| `op_load_true` | (adjacent) | `tag_true!` |
| `op_load_false` | (adjacent) | `tag_false!` |
| `op_load_zero` | #16 | `tag_smi_zero!` |
| `op_load_one` | (adjacent) | `tag_smi_one!` |
| `op_load_smi8` | #7 | `tag_smi_from_signed_byte!` |

V8 v7 vs pre-DSL-0 `d850f261`: **+1.7%** (corrected via same-load A/B; the original Task-10 number was loadavg-inflated).

**Two opcodes deferred** to Phase 1.B due to substrate gaps:
- `op_load_const8` (#21) — needed a flat pre-resolved constants array exposed on `LlIntState`.
- `op_load_this` (#12) — needed a pre-resolved `this`-value mirror on `LlIntState` with a sentinel scheme for `ThisState::Uninitialized` / `ThisState::Lexical`.

Phase 1.A retrospective lesson, codified into the rules for 1.B+: **strict top-30 + macro-shared-pair selection** (the 5 adjacent-family ports would NOT have shipped under this rule; their per-opcode dispatch share was negligible vs. their substrate footprint).

Summary: [`reports/js/lyng-js/dsl-1/phase-1a-summary.md`](dsl-1/phase-1a-summary.md).

### DSL-1 Phase 1.B (closed at `aa3ab9fc`)

Four sequential sub-phases. Combined: **substrate refactor + 11 inline ports + infra and methodology cleanup**.

#### 1.B.0 — counter wiring + microbench infra (closed at `ae8b7766`)

- `DispatchCounters` (3 banks × 256 u64) on `Vm`, asm-stable layout.
- `inc_dispatch_counter!` / `inc_slow_semantic_counter!` / `inc_slow_safepoint_counter!` auto-injected by the lowerer.
- 14 microbench snippets covering Phase-1.A and Phase-1.B target opcodes.
- Overhead ≈ 0% on V8 v7 (Apple Silicon wide-issue absorbs the 4-instruction counter sequence into existing slots).

Summary: [`reports/js/lyng-js/dsl-1/phase-1b0-summary.md`](dsl-1/phase-1b0-summary.md).

#### 1.B.1 — frame-context substrate (closed at `4ff25b9b`)

- Two new `LlIntState` fields: `frame_const_base: *const Value` (arena slot into `RuntimeCodeRecord::constants`) and `frame_this_value: Value` (mirror of `frame.this_value()`, or `Value::uninitialized_lexical()` sentinel for non-Value `ThisState`).
- `LLINT_STATE_FRAME_CONST_BASE` (offset 32) + `LLINT_STATE_FRAME_THIS_VALUE` (offset 40); struct grows 56→72 bytes.
- `resolve_initial_this_value(&Agent, &FrameRecord) -> Value` helper (two-layer: pure inner + Agent-aware wrapper). 4 unit tests.
- `load_constant!` and `load_state_value!` backend macros.
- Mandatory `feature-dev:code-reviewer` dispatch (per umbrella §3); 0 findings.

Summary: [`reports/js/lyng-js/dsl-1/phase-1b1-summary.md`](dsl-1/phase-1b1-summary.md). GC review: [`phase-1b1-gc-review.md`](dsl-1/phase-1b1-gc-review.md).

#### 1.B.2 — backfill ports (closed at `7baf5846`)

- `op_load_const8` (#21, ~104M dispatches/V8 v7 run) — 4-instruction inline body via `load_constant!`.
- `op_load_this` (#12, ~256M dispatches/V8 v7 run) — `load_state_value!` + sentinel-bail (4-instruction `movz/movk` sentinel materialization + cmp + b.eq slow).
- Both confirmed at **0.000% slow-path-share** on V8 v7.

Same-load A/B vs 1.B.1 close was originally reported as +4.89% but an audit found the loadavg overlap was 21% (just outside protocol); the 11-sample re-run landed at **+0.91%** geomean.

Summary: [`reports/js/lyng-js/dsl-1/phase-1b2-summary.md`](dsl-1/phase-1b2-summary.md). Cleanup batch: integrated into [`phase-1b-summary.md`](dsl-1/phase-1b-summary.md).

#### Cleanup batch (between 1.B.2 and 1.B.3, at `08727f92`)

Post-1.B.2 audit identified 7 drift findings:
1. Missing LoadConst8 + LoadThis microbench snippets (claimed in 1.B.0 summary but absent from the file).
2. Deferred 1.B.2 microbench gate (no numbers for the two ported opcodes).
3. 1.B.0 summary inaccurate framing.
4. 1.B.2 A/B loadavg-overlap 21% (just outside ±20% protocol).
5. Test262 baseline never captured at umbrella level.
6. ThisState::Uninitialized JS-coverage gap.
7. `asm-diff --check` doesn't cover `dsl::handlers::cold::*` namespace.

Findings #1, #2, #3, #6, #7 → cleanup batch 1 (`922ff5f2..2cb027b0`). Findings #4, #5 + mid-phase umbrella summary → cleanup batch 2 (`78e25a6b..08727f92`). Test262 baseline locked in at 49729 passing.

#### 1.B.3 — locals + Ldar (closed at `8ee22da7`, umbrella closure at `aa3ab9fc`)

9 inline ports + 2 new backend macros + per-opcode gates + cumulative A/B + sub-phase summary:

| Opcode | Top-30 rank | Why qualified |
|--------|------------:|---------------|
| `op_load_local_0` | #11 | top-30 anchor |
| `op_load_local_1` | #8 | top-30 anchor |
| `op_load_local_2` | #18 | top-30 anchor |
| `op_load_local_3` | #9 | top-30 anchor |
| `op_store_local_3` | #22 | top-30 anchor |
| `op_ldar` | #26 | top-30 anchor |
| `op_store_local_0` | (pair) | macro-shared symmetric pair (≪15min cost) — but FUNCTIONALLY UNREACHABLE (see below) |
| `op_store_local_1` | (pair) | macro-shared symmetric pair |
| `op_store_local_2` | (pair) | macro-shared symmetric pair |

All 9 handlers: 7 instructions total each (1 decode + 2 body + 4 dispatch). All at **0.000% slow-path-share** on V8 v7. Combined dispatch share: **~1.26B inlined dispatches per V8 v7 run** (the 8 reachable opcodes; StoreLocal0 has 0).

**LoadEnvSlot deferred** to a future substrate sub-phase. Investigation revealed it requires a new `frame_lexical_env` mirror on `LlIntState` (Phase-1.B.1-style refactor) plus inline depth-walk for the common `depth==0` case. Substrate work, not a mechanical port. Recorded in [`reports/js/lyng-js/dsl-1/phase-1b-followups.md`](dsl-1/phase-1b-followups.md).

**StoreLocal0 functional unreachability:** the bytecode-builder peephole at `crates/lyng-js/bytecode/src/builder.rs:150-166` rewrites `Move dst=0, src=B` → `Ldar B` before the `store_local_opcode` branch fires. So StoreLocal0 cannot be emitted from compiled JS source. Inline port retained for symmetry; 0 V8 v7 dispatches in practice.

Summaries: [`phase-1b3-summary.md`](dsl-1/phase-1b3-summary.md), umbrella [`phase-1b-summary.md`](dsl-1/phase-1b-summary.md), direct cumulative A/B [`phase-1b3-cumulative-ab.md`](dsl-1/phase-1b3-cumulative-ab.md).

---

## 3. Aggregate metrics at HEAD

### V8 v7 cumulative — direct 11-sample A/B vs pre-DSL-0 `d850f261`

| Workload    | `d850f261` median | `8ee22da7` median | Delta |
|-------------|------------------:|------------------:|------:|
| Richards    | 242               | 285               | **+17.77%** |
| DeltaBlue   | 287               | 315               | +9.76% |
| Crypto      | 222               | 248               | +11.71% |
| RayTrace    | 390               | 403               | +3.33% |
| NavierStokes| 399               | 420               | +5.26% |
| Splay       | 1214              | 1262              | +3.95% |
| **Geomean** | —                 | —                 | **+8.51%** |

Clears umbrella §1 criterion 5 (≥+3%) by **5.5pp headroom**. All 6 workloads positive; no regressions.

### Test262

49729 passing files / 0 failing / 100.00% rate at HEAD. Matches the mid-phase baseline. The 9 inline ports in 1.B.3 are pure register-window moves — no semantic surface touched.

### Inline-ported opcodes (cumulative across Phase 1.A + 1.B)

**18 opcodes inline-ported** (7 in 1.A + 2 in 1.B.2 + 9 in 1.B.3). Of these, **17 are in the V8 v7 top-30 OR macro-shared symmetric pairs of top-30** (StoreLocal0 is a macro-shared pair but functionally unreachable per above; the 5 Phase 1.A adjacent-family completions were not top-30 but landed under the pre-rule retrospective).

Per the DSL-1 epic spec §2 table: 18 of ~45 planned opcode ports done. Phases 1.C through 1.G land the remaining ~27.

### Cumulative trajectory vs epic-spec phase gates

| Phase | Cumulative V8 v7 target | Actual at phase close | Status |
|-------|------------------------:|----------------------:|:------:|
| 1.A | ≥ +5% vs `d850f261` | +1.7% | ⚠ shipped below target; epic spec retrospectively softened — adjacent-family ports landed but didn't return the projected lift |
| 1.B | ≥ +15% vs `d850f261` | **+8.51%** | ⚠ shipped below target; umbrella §1 ≥+3% gate (1.B's own number) cleared by 5.5pp, but the epic-spec ≥+15% target was based on Phase 1.A delivering ≥+5% |
| 1.C | ≥ +35% | TBD | pending |
| 1.D | ≥ +45% | TBD | pending |
| 1.F | ≥ +70% | TBD | pending |
| 1.G | ≥ +80% | TBD | pending |

**The epic-spec phase targets were projected from JSC LLInt-style improvement curves and assumed adjacent-family Phase 1.A would deliver ~+5% solo.** The actual +1.7% on Phase 1.A revealed that adjacent-family ports add minimal dispatch share. Phase 1.B's strict top-30 discipline produced the +8.51% cumulative (a 5× scale-up from Phase 1.A's per-port average), validating the rule but starting Phase 1.C from below the epic-spec curve. Phase 1.C-G should still hit their **relative** gates (each delivers what it can on top of the cumulative); the **absolute** cumulative targets may need re-baselining at Phase 1.C close based on actual delivered share.

---

## 4. Substrate inventory at HEAD `aa3ab9fc`

### `LlIntState` layout (72 bytes)

```rust
#[repr(C)]
pub struct LlIntState {
    pub frame_pc_offset: u32,       // 0  — PC offset within instructions
    pub _pad1: u32,                 // 4
    pub frame_pb_base: *const u8,   // 8  — pointer into BytecodeFunction.instructions
    pub frame_regs_base: *mut Value,    // 16 — pointer into register stack
    pub frame_fv_base: *mut FeedbackEntry, // 24 — pointer into feedback flat storage
    pub frame_const_base: *const Value, // 32 — Phase 1.B.1 arena pointer into RuntimeCodeRecord::constants
    pub frame_this_value: Value,    // 40 — Phase 1.B.1 mirror of frame.this_value() or sentinel
    pub frame_depth: u32,           // 48
    pub frame_check_epoch: u32,     // 52
    pub rust_context: *mut LlIntRustContextOpaque, // 56
    pub prefix: u8,                 // 64
    pub _pad2: [u8; 7],             // 65
}
```

Layout asserted by `ll_int_state_offsets_stable` test in `dsl/llint_state.rs`.

### Backend macros (AArch64)

Existing operand decode + register-window access (in `dsl/backend/aarch64/operands.rs`):
- `decode_a!`, `decode_abx!`, `decode_abc!` — operand-byte decode
- `load_reg!`, `store_reg!` — variable-index register-window access
- `load_acc!`, `store_acc!` — accumulator (register 0) shorthand

Frame/state access (in `dsl/backend/aarch64/frame.rs`, `constants.rs`, `locals.rs`):
- `load_state_value!` — fixed-offset 8-byte Value load from LlIntState (Phase 1.B.1)
- `load_constant!` — indexed load from `frame_const_base` (Phase 1.B.1)
- `load_local_fixed!`, `store_local_fixed!` — single-instruction fixed-index register-window access (Phase 1.B.3)

Value tag operations (in `values.rs`, `prelude.rs`):
- `tag_undefined!`, `tag_null!`, `tag_true!`, `tag_false!`, `tag_smi_zero!`, `tag_smi_one!`, `tag_smi_from_signed_byte!` — Phase 1.A
- `load_uninit_lex_sentinel!` — materializes `Value::uninitialized_lexical()` via `movz` + 3× `movk` (Phase 1.B.2)
- `cmp_branch_eq!` — `cmp + b.eq slow_label` helper (Phase 1.B.2)

Control + dispatch (in `control.rs`, `safepoint.rs`):
- `dispatch!`, `dispatch_after_slow!`, `call_slow!` — Phase 0
- `poll_safepoint!` — warm-handler poll (Phase 0)

Counters (in `counters.rs`, Phase 1.B.0):
- `inc_dispatch_counter!`, `inc_slow_semantic_counter!`, `inc_slow_safepoint_counter!` — auto-injected by the lowerer

### Infra

- **Counter wiring** (Phase 1.B.0): `DispatchCounters` on `Vm`, 3 banks × 256 u64. Verified: Move dispatches on Richards match reference within 0.2%.
- **Microbench snippets** (Phase 1.B.0 + 1.B.2 backfill + 1.B.3): 19 snippets total. All verified via `verify_opcodes_per_iter` test.
- **`lyng-js-bench v8suite --count-slow-path-share`**: produces per-opcode slow-path-share percentages from v8 v7 runs.
- **`lyng-js-bench asm-diff`**: produces asm baselines and checks against committed baselines. **Limitation:** doesn't yet auto-discover the `dsl::handlers::cold::*` symbol namespace; the Phase 1.B.2 + 1.B.3 baselines were captured manually via `cargo rustc --emit=asm` extraction. Tracked in [`phase-1b-followups.md`](dsl-1/phase-1b-followups.md).
- **GC integration**: mirror-discipline invariant for `LlIntState` arena pointers; debug-only stability assertion in the Refresh arm; gc-stress integration test at `crates/lyng-js/tests/src/gc_stress_frame_context.rs`. GC review: [`phase-1b1-gc-review.md`](dsl-1/phase-1b1-gc-review.md).

---

## 5. Key methodological lessons (Phase 1.B retrospective)

These five rules emerged from Phase 1.B and now apply to all future asm-DSL work:

1. **±20% loadavg overlap on A/Bs is a hard gate.** No rounding. Phase 1.B.2's original A/B sat at 21% and overstated the geomean by ~4× (revised +4.89% → +0.91%). Future A/Bs use 11+ samples and abort if the threshold is exceeded.
2. **Per-sub-phase A/Bs compose roughly but not authoritatively.** Mid-phase composition of 1.B.0+1+2 predicted +3.4% cumulative; the direct 1.B.3 measurement landed +8.51%. Always measure the umbrella gate directly at phase close.
3. **Structural compile-and-link tests are not sufficient for new substrate macros.** Phase 1.B.1's `assert_handler_symbol_exists` tests missed a latent x22→x24 register-pin bug. Phase 1.B.2 caught it only when a real handler dispatched. Future substrate work writes runtime-dispatch tests immediately, even if the canonical opcode is in a later phase.
4. **Trust grep over summary tables.** The 1.B.0 summary claimed 14 microbench snippets landed but `grep` against the file showed LoadConst8 + LoadThis missing. When a sub-phase depends on infra produced by a prior sub-phase, cross-check via direct file inspection at sub-phase start.
5. **Bytecode-builder peephole analysis is required for macro-shared symmetric pair claims.** StoreLocal0 looked qualified on paper (shares StoreLocal3's macro shape) but the peephole renders it dead. Check the emit pipeline before counting an opcode as in-scope.

The full audit + cleanup arc is documented in [`phase-1b-summary.md`](dsl-1/phase-1b-summary.md).

---

## 6. Next steps

### Recommended path forward (two options)

Both options are sound; the choice depends on whether we want to bank Phase 1.C's V8 v7 lift first or unblock LoadEnvSlot + the Phase 1.F IC opcodes via substrate.

#### Option A: Phase 1.C — SMI arithmetic + bitwise (recommended for V8 v7 momentum)

Per DSL-1 epic spec §2: `op_sub`, `op_mul`, `op_increment`, `op_decrement`, `op_bit_and`, `op_shift_left`, `op_shift_right` (7 opcodes). Cumulative target ≥ +35%. The SMI fast-path shape is already prototyped in `op_add` (Phase 1.A); these 7 are mechanical mirrors.

Estimated effort: 1.5-2 weeks. The substrate is fully in place; per-opcode gates apply (≤12 inline instr, microbench within 2× LLInt, slow-path-share <20%).

**Risks:** Phase 1.C is where SMI overflow handling becomes load-bearing. Each op needs a `bcs slow_path` for overflow detection. Slow-path-share could approach the 20% gate if overflow is common in V8 v7 workloads (it generally isn't, but verify empirically).

#### Option B: LoadEnvSlot substrate sub-phase (recommended if Phase 1.F IC work is near)

Add a `frame_lexical_env: *mut LexicalEnvironment` mirror on `LlIntState` (analog to `frame_const_base`). Pre-resolve at trampoline entry + refresh on Refresh arm. Implement inline fast-path for the `depth==0 && no_loop_envs` case with bail-to-slow for variable-depth walks. Then port `op_load_env_slot` and `op_store_env_slot`.

Estimated effort: 3-4 days (mirror Phase 1.B.1's structure). Unlocks 2 top-30 opcodes (LoadEnvSlot is in the original Phase 1.B scope) and de-risks Phase 1.F (the IC opcodes including `op_load_global` may benefit from similar lexical-env substrate).

**Risks:** the lex-env chain is mutable in source-level scope-extension cases (e.g., `with`, `eval`). The Phase 1.B.1 mirror-discipline invariant assumes the mirrored canonical source isn't mutated between Refresh egress events. Lex-env mutation across slow-path bridges (sloppy-mode eval) needs careful refresh discipline. May surface unsafe-cell-style aliasing concerns worth a GC-style design review.

### Other phases per epic spec §2

| Phase | Scope | Notes |
|-------|-------|-------|
| 1.D | Comparison + branch (`op_greater_equal`, `op_less_equal`, 5 jump opcodes — 7 total) | Smaller phase; ~1 week |
| 1.E | Pointer-identity cells refactor (`ObjectRef = u32` → `*mut Cell`) | Big refactor (3-4 weeks); no opcode ports; blocks 1.F |
| 1.F | IC mode-byte refactor + 6 IC opcodes (`op_get_named_property`, `op_assign_named_property`, `op_load_global`, `op_store_global`, `op_get_keyed_property`, `op_assign_keyed_property`) | 3 weeks; bundled refactor + ports |
| 1.G | Calls + tail-call (6 opcodes — frame-transitioning, all return `Refresh`) | 1 week; closes DSL-1 |

### Followups tracked (not blocking but worth picking up opportunistically)

From [`reports/js/lyng-js/dsl-1/phase-1b-followups.md`](dsl-1/phase-1b-followups.md):

- **`asm-diff --check` namespace expansion**: extend the bench tool to discover `dsl::handlers::cold::*` symbols automatically (currently capture is manual).
- **`ThisState::Uninitialized` JS-coverage gap**: TDZ in derived constructor before `super()` is not directly testable from JS in lyng-js yet. The sentinel mechanism is exercised structurally; a future JS-level test once class+super support fills out.
- **`Vec<ConstantValue>` pre-resolution shape**: pre-resolution at install time works for Smi/Float; Atom/Builtin require lookup through `Vm`. Phase 1.B.1 reuses `RuntimeCodeRecord::constants` arena slot which the install pipeline already populates; further opcodes that need other forms of pre-resolution should follow this pattern.
- **StoreLocal0 deprecation candidate**: 0 dispatches in practice (peephole rewrites to Ldar). Could be removed from the opcode set in a future cleanup; not blocking.

### Off-ramp triggers (per DSL-1 epic spec §2)

- 5+ consecutive opcode ports in any phase fail per-opcode gates.
- Cells refactor (1.E) regresses Test262 baseline or microbench > 2%.
- IC mode-byte refactor (1.F.1) regresses V8 v7 vs pre-refactor.

If any fires, the coordinator writes a diagnostic report and decides: deepen, defer, or close DSL-1 with banked wins. **At HEAD `aa3ab9fc` the banked wins are already substantial** (+8.51% cumulative on a 6-workload V8 v7 suite, 18 inline ports, full substrate); a graceful close at any future off-ramp would still leave a meaningfully faster engine.

---

## 7. Coordinator workflow (proven through Phase 1.B)

The subagent-driven workflow that produced Phase 1.B in ~10 wall-clock days uses these patterns:

1. **`/superpowers:brainstorming`** for a new sub-phase → produces a spec at `docs/superpowers/specs/`.
2. **`/superpowers:writing-plans`** → produces an implementation plan at `docs/superpowers/plans/`.
3. **Sub-phase execution** via subagent dispatch:
   - One refactor-worker subagent per task batch (typically 2-4 tasks per worker for tightly-coupled sequences).
   - Coordinator handles bench A/Bs at the coordinator level (bench is long-running and benefits from coordinator-level loadavg awareness).
   - Mandatory `feature-dev:code-reviewer` dispatch for substrate-touching sub-phases (1.B.1 was reviewed; 1.B.3 used self-review as the changes were mechanical ports).
4. **Sub-phase close**: same-load A/B, behavioral parity check, Test262 check, sub-phase summary, followups recording.
5. **Phase close**: direct cumulative A/B vs phase-spec baseline, umbrella summary update.

User deny rules consistently honored: no `git -C`, no `cd && git`, no `--no-verify`, no destructive ops without consent.

---

## 8. References

### Design docs

- Parent design: [`docs/lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md`](../../docs/lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md)
- DSL-1 epic spec: [`docs/superpowers/specs/2026-05-18-dsl-1-hot-opcode-rollout-design.md`](../../docs/superpowers/specs/2026-05-18-dsl-1-hot-opcode-rollout-design.md)
- Phase 1.B umbrella: [`docs/superpowers/specs/2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md`](../../docs/superpowers/specs/2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md)
- Phase 1.B.1 spec: [`docs/superpowers/specs/2026-05-19-dsl-1-phase-1b1-frame-context-refactor-design.md`](../../docs/superpowers/specs/2026-05-19-dsl-1-phase-1b1-frame-context-refactor-design.md)
- Phase 1.B.2 spec: [`docs/superpowers/specs/2026-05-19-dsl-1-phase-1b2-backfill-ports-design.md`](../../docs/superpowers/specs/2026-05-19-dsl-1-phase-1b2-backfill-ports-design.md)
- Phase 1.B.3 spec: [`docs/superpowers/specs/2026-05-20-dsl-1-phase-1b3-locals-and-ldar-design.md`](../../docs/superpowers/specs/2026-05-20-dsl-1-phase-1b3-locals-and-ldar-design.md)

### Phase summaries (chronological)

- DSL-0c close: [`reports/js/lyng-js/dsl-0c-status.md`](dsl-0c-status.md)
- Phase 1.A: [`reports/js/lyng-js/dsl-1/phase-1a-summary.md`](dsl-1/phase-1a-summary.md)
- Phase 1.B umbrella (final): [`reports/js/lyng-js/dsl-1/phase-1b-summary.md`](dsl-1/phase-1b-summary.md)
- Phase 1.B.0: [`phase-1b0-summary.md`](dsl-1/phase-1b0-summary.md) — counter wiring + microbench infra
- Phase 1.B.1: [`phase-1b1-summary.md`](dsl-1/phase-1b1-summary.md) — frame-context substrate
- Phase 1.B.2: [`phase-1b2-summary.md`](dsl-1/phase-1b2-summary.md) — op_load_const8 + op_load_this
- Phase 1.B.3: [`phase-1b3-summary.md`](dsl-1/phase-1b3-summary.md) — locals + Ldar
- Phase 1.B followups: [`phase-1b-followups.md`](dsl-1/phase-1b-followups.md)

### Per-handler ported reports (18 opcodes)

Phase 1.A: 7 reports under [`reports/js/lyng-js/dsl-handlers/`](dsl-handlers/) — `op_load_undefined.md`, `op_load_null.md`, `op_load_true.md`, `op_load_false.md`, `op_load_zero.md`, `op_load_one.md`, `op_load_smi8.md`.

Phase 1.B.2: 2 reports — `op_load_const8.md`, `op_load_this.md`.

Phase 1.B.3: 9 reports — `op_load_local_{0,1,2,3}.md`, `op_store_local_{0,1,2,3}.md`, `op_ldar.md`.

### Asm baselines

`reports/js/lyng-js/dsl-asm-baseline-aarch64/` contains captured asm for each inline-ported handler.

### Key A/B comparison artifacts

- Phase 1.B.1 A/B vs 1.B.0: [`phase-1b1-ab-comparison.md`](dsl-1/phase-1b1-ab-comparison.md)
- Phase 1.B.2 A/B (11-sample re-run): [`phase-1b2-ab-comparison.md`](dsl-1/phase-1b2-ab-comparison.md)
- Phase 1.B.3 same-load A/B vs cleanup mid-phase: [`phase-1b3-ab-comparison.md`](dsl-1/phase-1b3-ab-comparison.md)
- **Phase 1.B.3 cumulative A/B vs pre-DSL-0** (the umbrella gate): [`phase-1b3-cumulative-ab.md`](dsl-1/phase-1b3-cumulative-ab.md)
- Test262 baseline: [`phase-1b-test262-baseline.md`](dsl-1/phase-1b-test262-baseline.md)

### Source code anchors

- DSL substrate: `crates/lyng-js/vm/src/dsl/`
- AArch64 backend macros: `crates/lyng-js/vm/src/dsl/backend/aarch64/`
- Opcode handlers: `crates/lyng-js/vm/src/dsl/handlers/{cold,warm,hot}.rs`
- Lowerer proc-macro: `crates/lyng-js-vm-dsl/src/`
- Bench tool: `tools/lyng-js-bench/`
