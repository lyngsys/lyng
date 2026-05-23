# Design: DSL-1 — Hot-opcode rollout

**Date:** 2026-05-18
**Status:** Design approved; ready for implementation planning.
**Parent design:** [`docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md`](../../lyng/2026-05-16-asm-dsl-llint-interpreter-design.md) — §10 DSL-1 phase.
**Predecessor work:** DSL-0c complete (α dispatch deleted, all 152 opcodes on DSL substrate, 12 opcodes shipped — 3 with true inline fast paths: `op_move`, `op_add`, `op_loop_header`; 9 as cold-stub delegators in `hot.rs`/`warm.rs`).

---

## 1. Goal, scope, and exit criteria

### Goal

Complete the substrate win started by DSL-0. Port the remaining 28 top-30 hot opcodes from cold-stub delegation to full inline DSL fast paths, plus the two data-layout refactors required for the IC family's inline asm to be competitive with LLInt.

### In scope

- **All 30 top-30 hot opcodes ported to full inline DSL fast paths.** 2 are already done from DSL-0 (`op_move`, `op_add`); 28 remain to port in DSL-1.
- **Adjacent family members rounded out for completeness** (~15-17 additional opcodes that fall outside the measured top-30 but belong to families being ported — e.g., `op_store_local_0..2` to match `op_load_local_0..3`, `op_jump_if_true8` to match `op_jump_if_false8`, `op_call_*` family, the trivial constant-loaders `op_load_undefined`/`_null`/`_true`/`_false`/`_one`). Total port count: **~45 opcodes**.
- **IC mode-byte refactor** — replaces today's Phase 3a/3e/3f layered fast paths with LLInt-style mode-byte dispatch (parent §9 + §10 weeks 8-9).
- **Pointer-identity cells refactor** — `ObjectRef = u32` → `*mut Cell`, eliminating the side-table indirection on every object-record access (parent §9, ~3-4 weeks).
- Per-handler ported reports under [`reports/lyng/dsl-handlers/`](../../../reports/lyng/dsl-handlers/).
- Per-handler asm baselines under [`reports/lyng/dsl-asm-baseline-aarch64/`](../../../reports/lyng/dsl-asm-baseline-aarch64/).
- New DSL ops added to [`crates/lyng/vm/src/dsl/backend/aarch64/`](../../../crates/lyng/vm/src/dsl/backend/aarch64/) as ports demand them, documented in `ops.md`.
- Phase summary reports under `reports/lyng/dsl-1/`.
- Final DSL-1 decision document at `reports/lyng/dsl-1/dsl-1-completion.md`.

### Out of scope

- JIT (parent §2 non-goal — Baseline JIT remains deferred behind interpreter completion).
- Test262 100% conformance — parallel workstream, not gated on DSL-1.
- x86_64 backend (DSL-2 deferred per parent §2).
- `Cell` 8-byte header layout refactor beyond what the pointer-identity refactor strictly requires.
- `Shape` transition representation — evidence-driven from parent §9; defer unless an IC port surfaces the need with measurement evidence.
- Cold opcodes outside the top-30 — they stay as cold-stub delegators.

### Exit criteria

From parent §10 DSL-1:

1. All 30 hot opcodes have full DSL inline implementations with committed ported reports.
2. **Cumulative V8 v7 geomean ≥ +80% over pre-DSL-0 baseline.** Richards specifically ≥ ~570 against today's 318.
3. No workload regresses > 5% vs pre-DSL-0 baseline.
4. Test262 pass count ≥ pre-DSL-1 baseline (no regression from DSL-0c state).
5. Every per-handler asm-diff report exists in [`reports/lyng/dsl-handlers/`](../../../reports/lyng/dsl-handlers/).
6. **Slow-path-share < 20% per hot opcode** on V8 v7 workloads. Per-opcode waivers allowed; each must be justified in the ported report against an LLInt-on-same-workload baseline.

---

## 2. Phase structure and schedule

DSL-1 splits into **7 sequential phases**. Single-dev wall-clock estimate: **~14-15 weeks** (parent §10 said 8-10; the +3-5 weeks is the cells refactor in-scope per user decision).

| Phase | Name | Duration | Opcodes / deliverables | Gate to next |
|-------|------|---------:|------------------------|--------------|
| **1.A** | Trivial loads | 2 weeks | `op_load_undefined`, `op_load_null`, `op_load_true`, `op_load_false`, `op_load_zero`, `op_load_one`, `op_load_smi8`, `op_load_const8`, `op_load_this` (~9) | V8 v7 cumulative ≥ +5%; all 9 within 5 instr of LLInt; slow-path-share <20% |
| **1.B** | Local register access | 2 weeks | `op_load_local_0..3`, `op_store_local_0..3` (top-30 subset), `op_ldar`, `op_load_env_slot` (~10) | V8 v7 cumulative ≥ +15% |
| **1.C** | SMI arithmetic + bitwise | 2 weeks | `op_sub`, `op_mul`, `op_increment`, `op_decrement`, `op_bit_and`, `op_shift_left`, `op_shift_right` (7) | V8 v7 cumulative ≥ +35%; SMI fast-path inline like `op_add` |
| **1.D** | Comparison + branch | 1 week | `op_greater_equal`, `op_less_equal`, plus inline branches for the currently-cold-stub `op_jump`, `op_jump_if_true`, `op_jump_if_false`, `op_jump_if_true8`, `op_jump_if_false8` (7) | V8 v7 cumulative ≥ +45%; warm-handler poll branches verified intact |
| **1.E** | Pointer-identity cells refactor | 3-4 weeks | `ObjectRef = u32` → `*mut Cell`; touches every object access path; no opcode ports during this phase | All behavioral tests pass; Test262 ≥ baseline; aggregate V8 v7 regresses no more than 2% |
| **1.F** | IC mode-byte refactor + IC opcodes | 3 weeks | 1.F.1 mode-byte refactor (~1 wk); 1.F.2 port `op_get_named_property`, `op_assign_named_property`, `op_load_global`, `op_store_global`, `op_get_keyed_property`, `op_assign_keyed_property` (6) | V8 v7 cumulative ≥ +70%; IC slow-path-share <20% with documented per-opcode waivers |
| **1.G** | Calls + tail-call | 1 week | `op_call_0..3`, `op_call`, `op_tail_call` (6) — frame-transitioning, all return `Refresh` | Final V8 v7 ≥ +80%; all exit criteria from §1 |

Phase 1.A and 1.B together cover roughly 19 of the top-30 by dispatch share — high-volume, mechanically simple ports validate the workflow before harder phases. Phase 1.E (cells) is the biggest single unit; it blocks 1.F because IC opcodes need fast inline cell access. Phase 1.F bundles its own internal refactor with 6 ports because the refactor is a prerequisite and the ports validate it.

**Approximate phase counts:** 1.A=9, 1.B=10, 1.C=7, 1.D=7, 1.E=0 (refactor), 1.F=6 (plus refactor), 1.G=6. Total ≈ 45 opcode ports.

### Off-ramp protocol

**Triggers:**
- 5+ consecutive opcode ports in any phase fail per-opcode gates (asm shape or slow-path-share).
- Cells refactor (1.E) regresses Test262 baseline or microbench worse than pre-refactor by >2%.
- IC mode-byte refactor (1.F.1) regresses V8 v7 vs pre-refactor.

**Response:**
- Pause the phase; do not start the next opcode.
- Coordinator writes a diagnostic report to `reports/lyng/dsl-1/off-ramp-<date>-<phase>.md` documenting the failure pattern.
- Decision options: (a) deepen scope into a sub-investigation; (b) defer affected opcodes to DSL-3 or beyond; (c) abort DSL-1 with the wins already banked. Every committed handler stays in production regardless.

---

## 3. Subagent dispatch model

User selection: **conservative — one worker subagent at a time**. The main session is the coordinator; one worker is in flight per work unit; sequential gating between units.

### Roles

**Coordinator (main session).**
- Dispatches one worker subagent at a time per opcode or refactor.
- Reviews each worker's output (asm-diff, ported report, microbench, behavioral tests) before dispatching the next.
- Owns phase-level gating: runs aggregate V8 v7 sweep at phase boundaries, decides go/no-go, fires off-ramp if criteria fail.
- Owns the running phase plan and the final DSL-1 decision document.

**Opcode worker subagent.** One per opcode port. Follows the 8-step workflow below. Reports back rather than commits when it can't satisfy a gate.

**Refactor worker subagent.** One for cells; one for IC mode-byte. Larger scope; owns its own internal task breakdown via its own TodoWrite. Returns when behavioral parity restored, microbench expected, no opcode ports broken.

**Reviewer subagent.** Optional, fired by the coordinator **sequentially** after the worker returns and before dispatching the next worker. `feature-dev:code-reviewer` against the worker's commit. High-confidence findings only. Coordinator decides whether to send back for fixes. Fired primarily for ports that surface new DSL ops, new layout interactions, or off-pattern asm; mechanical ports can be self-reviewed. The "one worker at a time" rule covers all subagent kinds — reviewer doesn't overlap with the next worker.

### Per-opcode workflow (8 steps, parent §10)

The worker subagent's brief follows the parent design's canonical workflow:

1. Read JSC's matching handler in `LowLevelInterpreter64.asm` (or the captured reference at `reports/lyng/llint-reference/`). Understand the fast-path shape.
2. Identify any data-layout dependency. If a refactor is surfaced that isn't already done in DSL-1, **abort and report**. Coordinator decides whether to schedule the refactor or skip the opcode.
3. Replace the cold-stub body in `dsl/handlers/cold.rs` (or `warm.rs` / `hot.rs` if previously cold-stubbed there) with a full inline DSL fast path. Add new DSL ops to `backend/aarch64/` if needed (with `ops.md` entry + `mod.rs` re-export).
4. Run `cargo run --release -p lyng-bench -- asm-diff --check`; inspect output; iterate until shape is within budget.
5. Run microbench for the opcode; capture ns/dispatch with confidence interval; compare to LLInt reference.
6. Run isolated V8 v7 sweep with `--require-isolation`; capture slow-path-share; verify < 20% (or document a justified waiver).
7. Write `reports/lyng/dsl-handlers/op_xxx.md` with: DSL source excerpt, current asm output, LLInt reference asm, side-by-side annotated diff, microbench results, behavioral test references.
8. Commit handler source + new asm baseline + ported report + any new DSL op as one cohesive change.

### What the worker gets in its dispatch prompt

Each worker subagent is briefed with the standard self-contained context:

- Opcode name and its top-30 dispatch share (from [`v8-v7-top30.tsv`](../../../reports/lyng/r0/v8-v7-top30.tsv)).
- Path to its JSC reference asm (or instructions to capture via `lyng-bench capture-llint`).
- Current cold-stub location (file path and approximate line).
- Path to the matching semantic body in `vm/src/vm/semantics/`.
- Pointers to one or two previously-completed similar opcodes as exemplars.
- The 8-step workflow above.
- The slow-path-share invariant and asm-diff budget.
- Explicit instructions to report back rather than escalate scope if a refactor is surfaced.

---

## 4. Gates, measurement, and verification cadence

### Per-opcode gates (enforced by the worker before it commits)

| Gate | Criterion | Source |
|------|-----------|--------|
| Behavioral | `cargo test -p lyng-vm -p lyng-tests` passes | Existing test suite |
| Asm shape | Within 5 instructions of LLInt's matching handler, plus any documented `Value`/`ObjectRef` delta from the R-0 value-layout report | Per-opcode ported report quantifies the delta |
| Microbench | ns/dispatch within 2× of JSC LLInt's matching opcode, isolated, 7-sample median | `lyng-bench microbench` |
| Slow-path-share | <20% on V8 v7 (per-opcode waivers allowed; must be justified in ported report against an LLInt-on-same-workload baseline) | `lyng-bench v8suite --count-slow-path-share` |
| Asm baseline | Updated and committed; passes `asm-diff --check` | `lyng-bench asm-diff` |
| Ported report | Exists with DSL source, current asm, LLInt reference, side-by-side diff, microbench data | `reports/lyng/dsl-handlers/` |

If a worker can't satisfy any gate, it reports back rather than commits.

### Per-phase gates (coordinator enforces at phase boundary)

After every opcode in a phase lands:

1. Run full V8 v7 sweep with `--require-isolation` (loadavg < 2.0); capture geomean delta vs pre-DSL-0 baseline.
2. Run Test262 slice for affected opcode families; verify no regression.
3. Inspect per-opcode slow-path-share table across the phase.
4. Compare aggregate microbench data against the phase's targeted V8 v7 cumulative threshold (table in §2).
5. If passed, write a phase-completion note to `reports/lyng/dsl-1/phase-1X-summary.md` and proceed.
6. If failed, off-ramp protocol fires (§2 above).

### Per-opcode verification cadence (worker)

Per parent §8 cadence:

1. `cargo build --release` — builds with new handler.
2. `cargo run --release -p lyng-bench -- asm-diff --check` — verifies asm baselines.
3. `cargo run --release -p lyng-bench -- microbench --opcodes <name>` — captures ns/dispatch.
4. `cargo run --release -p lyng-bench -- v8suite --require-isolation --count-slow-path-share` — captures slow-path-share if affected.
5. `cargo test -p lyng-vm -p lyng-tests` — behavioral tests.
6. Focused Test262 slice for affected opcode family.
7. Commit asm baseline + ported report + handler source as one change.

No CI; developer-driven discipline. The coordinator checks each worker's commit before dispatching the next.

### Measurement infrastructure (already exists from R-0)

- `lyng-bench microbench` ✓
- `lyng-bench asm-diff` ✓
- `lyng-bench capture-llint` ✓
- `lyng-bench v8suite --count-slow-path-share` ✓
- `--count-opcodes` for per-opcode dispatch counts ✓
- `--require-isolation` ✓
- Hot-opcodes config ✓ ([`tools/lyng-bench/hot-opcodes.toml`](../../../tools/lyng-bench/hot-opcodes.toml))

If a worker discovers a gap (e.g., missing microbench scaffold for a specific opcode shape), it reports back; the coordinator schedules an infra task before resuming.

---

## 5. Data-layout refactor sequencing (Phases 1.E and 1.F)

Both refactors are the highest-risk pieces of DSL-1. Each gets its own implementation plan via writing-plans during execution; this spec sets the boundaries only.

### Phase 1.E — Pointer-identity cells (~3-4 weeks)

**Current state.** `ObjectRef = u32` is a handle into a heap-resident `ObjectPool` side table. Every object-record access goes: tag check → `ObjectRef` → pool indirection → `ObjectRecord` → field. Two loads per object-record read where LLInt does one.

**Target state.** Object Values directly carry `*mut Cell` payloads (NaN-tagged). The cell carries an 8-byte header (kind tag + shape pointer + flags + GC bits) followed by inline slots. Object access becomes: tag check → `*mut Cell` → header → field — one load.

**In scope:**
- New `Cell` repr; allocator and GC integration.
- Update `Value` payload encoding: object variant carries `*mut Cell` instead of `ObjectRef`.
- Update every consumer of `ObjectRef` to consume `*mut Cell`. `ObjectRef` type deletes when the last consumer migrates.
- DSL ops affected: `check_object_ref!` → `check_cell!`, `load_object_record!` → direct cell-pointer dereference. `objects.rs` macros rewrite.
- All semantic bodies that read object fields rewrite to use cell-pointer access.
- GC root scanning updates: cells are roots; the pool indirection goes away.

**Out of scope (deferred):**
- Cell 8-byte header layout decisions beyond what the pointer-identity refactor strictly requires. If asm-diff during IC opcode ports surfaces additional layout work, that's its own ticket.
- JIT integration — cells must work when JIT lands, but JIT integration is DSL-3+.

**Exit criteria:**
1. Behavioral parity — `cargo test -p lyng-vm -p lyng-tests` passes.
2. Test262 pass count ≥ pre-refactor baseline.
3. Aggregate V8 v7 sweep regresses no more than 2% (cells is a layout change; opcode-level wins come in 1.F).
4. `Value` and `Cell` type definitions documented in updated [`docs/lyng/runtime-primitives.md`](../../lyng/runtime-primitives.md).
5. Cells refactor design doc at `docs/lyng/2026-MM-DD-pointer-identity-cells-design.md` exists and is committed.

**Off-ramp:**
- If Test262 regresses unfixably; if GC integration produces use-after-free; if behavioral tests show heisenbugs — abort 1.E. Reset to pre-refactor branch. Re-evaluate whether cells refactor stays in DSL-1 or moves to its own epic.

### Phase 1.F — IC mode-byte refactor + 6 IC opcode ports (~3 weeks)

**1.F.1 — IC mode-byte refactor (~1 week).**

**Current state.** `NamedPropertyFeedback` has layered fast paths from Phase 3a (monomorphic), 3e (proto chain), 3f (polymorphic packed sidecars). Each fast path is decided by tag inspection on the feedback entry. This is N layered branches in the slow path; DSL inline asm can only easily encode one branch.

**Target state.** Each IC entry carries a **mode byte** as its first field. The DSL handler reads the mode byte once, branches on it to a dedicated fast path per mode (LLInt-style: monomorphic, proto, polymorphic, megamorphic). Inline asm is one byte load + one compare + one branch per fast path.

**In scope:**
- Add mode byte to `FeedbackEntry` layout for property ICs.
- Backfill the mode byte when the slow path transitions an IC entry to a new state.
- Update flat-array entry size if mode-byte widens an existing variant.
- New DSL ops: `load_ic_mode!`, `branch_ic_mode!` (or equivalent — exact shape decided when porting `op_get_named_property`).
- Preserve existing Phase 3f packed sidecar content (the refactor adds a discriminator, not removes data).

**Exit criteria for 1.F.1:**
- IC opcodes still pass all behavioral tests via slow path (no DSL inlining yet at this point).
- Microbench shows no regression on a sweep that exercises monomorphic IC sites.
- Mode-byte layout documented in updated `llint-dsl-abi.md` or a new `ic-mode-byte.md` companion.

**1.F.2 — IC opcode ports (~2 weeks).**

The 6 IC opcodes get full inline DSL fast paths reading the mode byte:
- `op_get_named_property` (highest IC dispatch share)
- `op_assign_named_property`
- `op_load_global`
- `op_store_global`
- `op_get_keyed_property`
- `op_assign_keyed_property`

Each port follows the standard 8-step workflow. Per-opcode slow-path-share waivers are expected (megamorphic IC sites will have >20% slow-path-share; the worker documents the workload mix and a justified threshold).

**Exit criteria for 1.F.2:**
- All 6 IC opcodes ported with inline DSL fast paths.
- Per-opcode slow-path-share < 20% on monomorphic-dominant V8 v7 workloads; documented waivers for polymorphic-dominant workloads.
- Aggregate V8 v7 cumulative ≥ +70% over pre-DSL-0 baseline (this is the phase target; the +80% final target is hit after 1.G calls).

---

## 6. Risks (deltas from parent §11)

| Risk | Likelihood | Impact | Mitigation |
|------|-----------:|-------:|-----------|
| Cells refactor regresses Test262 unfixably | medium | high | Off-ramp in Phase 1.E. Refactor lands on its own branch with full Test262 sweep gating merge. Coordinator runs Test262 mid-refactor to catch drift early. |
| Cells refactor exposes GC bugs (use-after-free, missed roots) | medium | high | Miri tests on cells allocation + scan paths. `gc-stress` mode during refactor. Behavioral tests under stress mode before merging. |
| IC mode-byte refactor invalidates Phase 3f packed sidecars | low | medium | Refactor preserves packed-sidecar content; only adds a discriminator byte at entry head. Phase 3f tests gate the refactor. |
| `call_*` opcodes (Phase 1.G) regress on frame-transition overhead | medium | medium | `Refresh` path is unavoidable for frame transitions. Microbench `op_call_0` on tight call/return loops; if >2× LLInt, surface the `op_return` fast-return shortcut from parent open question §13.2. |
| Worker subagent scope creep (refactor surfaced mid-port) | medium | medium | Worker briefs explicitly say "abort and report" on surfaced refactors. Coordinator decides whether to schedule refactor or skip opcode. |
| Per-opcode review backlog stalls dispatch | medium | low | Reviewer subagent is optional; coordinator self-reviews mechanical ports. Reviewer fired for ports surfacing new DSL ops, new layout interactions, or off-pattern asm. |
| Asm-diff churn from rustc upgrade mid-DSL-1 | low | medium | Pin a known-good rustc for the duration of DSL-1. Document the version in `rust-toolchain.toml`. Refresh deliberately at a phase boundary if needed. |
| DSL ops vocabulary grows unsustainably | medium | low | Per-arch directory budget: ~50 ops total (today ~25-30 in `backend/aarch64/`). New op added only on third occurrence of pattern. Coordinator audits at each phase boundary. |
| Per-opcode ported reports become busywork | low | low | Use a template generator (`lyng-bench gen-port-report --opcode <name>` — schedule into Phase 1.A if not already present). |

Pre-existing risks from parent §11 (DSL-0a expansion, `naked_asm` ergonomics, `LlIntState` layout instability, slow-path panic, GC starvation) are retired by DSL-0c being shipped.

---

## 7. Deliverables checklist (must exist at DSL-1 completion)

- **~45 new DSL handler implementations** with inline fast paths (28 top-30 + ~17 adjacent family members).
- One per-handler ported report per implementation in [`reports/lyng/dsl-handlers/`](../../../reports/lyng/dsl-handlers/).
- One asm baseline file per implementation in [`reports/lyng/dsl-asm-baseline-aarch64/`](../../../reports/lyng/dsl-asm-baseline-aarch64/) (updated; some legacy `CamelCase.asm` files from α era need refreshing or retiring).
- Updated [`tools/lyng-bench/hot-opcodes.toml`](../../../tools/lyng-bench/hot-opcodes.toml) with calibrated per-opcode `aarch64_max_instructions` budgets (currently 0 placeholders — DSL-1 calibrates them from real measurements).
- Cells refactor design doc at `docs/lyng/2026-MM-DD-pointer-identity-cells-design.md`.
- IC mode-byte refactor design doc at `docs/lyng/2026-MM-DD-ic-mode-byte-design.md` (or section in updated `llint-dsl-abi.md`).
- 7 phase summary reports at `reports/lyng/dsl-1/phase-1X-summary.md`.
- Final DSL-1 decision document at `reports/lyng/dsl-1/dsl-1-completion.md`.
- Updated [`docs/lyng/runtime-primitives.md`](../../lyng/runtime-primitives.md) reflecting cells layout.
- Updated [`docs/lyng/architecture.md`](../../lyng/architecture.md) only if dispatch substrate behavior changed (it shouldn't — that was DSL-0).

---

## 8. Policy and tracker alignment

Per parent §12, the policy updates landed in DSL-0. No further policy changes needed for DSL-1 unless cells refactor surfaces an unsafe scope question (e.g., does direct `*mut Cell` dereferencing need its own scoped exception?). Coordinator audits at the start of Phase 1.E.

Per parent §12, the dcat epic for DSL-1 (a child of `lyng-49qk`) needs to exist. Confirm at planning kickoff; create if absent. Each phase gets its own dcat issue child of the DSL-1 epic.

---

## 9. Open questions to revisit during execution

These don't block DSL-1 but should be answered with data during the phases:

1. **`op_return` Refresh vs fast-return** (parent §13.2) — revisit after Phase 1.A microbench shows `op_return` latency in context. Decide before Phase 1.G calls.
2. **Slow-path-share threshold per IC opcode** (parent §13.8) — derive baseline from LLInt on the same workload; document per-opcode threshold methodology in 1.F.2 ported reports.
3. **Compiler invariant for backedges** (parent §13.10) — audit after Phase 1.D; if all generated code passes through `op_loop_header`, simplify the warm-handler set (delete backward-jump poll branches from `op_jump*`).
4. **Cell 8-byte header layout** — settle exact layout (kind tag bits, shape pointer width, flag bits, GC bits) during Phase 1.E. Likely lands in the cells refactor design doc.
5. **Per-opcode `aarch64_max_instructions` budgets** — currently 0 placeholders in [`hot-opcodes.toml`](../../../tools/lyng-bench/hot-opcodes.toml). Calibrate during Phase 1.A from real ported handlers + LLInt baselines.

---

## 10. References

- **Parent design:** [`docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md`](../../lyng/2026-05-16-asm-dsl-llint-interpreter-design.md) — full substrate design; this spec is a faithful execution of its §10 DSL-1 phase.
- **Measured top-30 dispatch shares:** [`reports/lyng/r0/v8-v7-top30.tsv`](../../../reports/lyng/r0/v8-v7-top30.tsv).
- **Hot-opcodes config:** [`tools/lyng-bench/hot-opcodes.toml`](../../../tools/lyng-bench/hot-opcodes.toml).
- **DSL backend:** [`crates/lyng/vm/src/dsl/`](../../../crates/lyng/vm/src/dsl/) — substrate from DSL-0.
- **Existing DSL handlers (12 from DSL-0):** [`crates/lyng/vm/src/dsl/handlers/hot.rs`](../../../crates/lyng/vm/src/dsl/handlers/hot.rs), [`warm.rs`](../../../crates/lyng/vm/src/dsl/handlers/warm.rs), [`cold.rs`](../../../crates/lyng/vm/src/dsl/handlers/cold.rs).
- **Semantic bodies (all 152):** [`crates/lyng/vm/src/vm/semantics/`](../../../crates/lyng/vm/src/vm/semantics/).
- **DSL ops vocabulary:** [`crates/lyng/vm/src/dsl/ops.md`](../../../crates/lyng/vm/src/dsl/ops.md), [`backend/aarch64/`](../../../crates/lyng/vm/src/dsl/backend/aarch64/).
- **Existing ported reports (12 from DSL-0):** [`reports/lyng/dsl-handlers/`](../../../reports/lyng/dsl-handlers/).
- **Existing asm baselines:** [`reports/lyng/dsl-asm-baseline-aarch64/`](../../../reports/lyng/dsl-asm-baseline-aarch64/).
- **JSC LLInt reference:** `/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter64.asm` (read-only reference, not vendored).
- **Engineering standards:** [`docs/lyng/engineering-standards.md`](../../lyng/engineering-standards.md), [`AGENTS.md`](../../../AGENTS.md), [`crates/lyng/AGENTS.md`](../../../crates/lyng/AGENTS.md).
