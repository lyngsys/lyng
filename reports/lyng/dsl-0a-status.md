# DSL-0a Status Report

DSL-0a is the first sub-phase of the DSL-0 milestone documented in
[`docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md`](../../../docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md).
Its scope is semantic extraction — moving every opcode's semantic
body out of its α `extern "C"` handler into a free `op_xxx_semantic`
function reachable through a transitional `LlIntDispatchState`
wrapper, while the α path remains the active dispatch substrate. No
runtime substrate change.

This report records what landed, the exit-criterion verification,
and concerns / observations from execution.

## 1. Deliverables

| # | Deliverable | Task | Status |
|--:|---|---|---|
| 1 | `vm/src/dsl/` scaffold + `opcode_manifest.rs` skeleton | A2 | DONE |
| 2 | `SemanticOutcome` enum + `slow_path.rs` | A3 | DONE |
| 3 | Transitional `LlIntDispatchState` wrapper | A4 | DONE |
| 4 | `vm/src/vm/semantics/` module skeleton | A5 | DONE |
| 5 | Manifest Test 1 (exhaustive coverage), `#[ignore]`d → enabled in A18 | A6, A18 | DONE |
| 6 | Manifest Test 4 (source-grep smoke), `#[ignore]`d → enabled in A18 | A7, A18 | DONE |
| 7 | `loads` family extraction (37 opcodes) | A8 | DONE |
| 8 | `arithmetic` family extraction (29 opcodes) | A9 | DONE |
| 9 | `control_flow` family extraction (10 opcodes) | A10 | DONE |
| 10 | `property` family extraction (21 opcodes) | A11 | DONE |
| 11 | `names` family extraction (17 opcodes) | A12 | DONE |
| 12 | `scope` family extraction (10 opcodes) | A13 | DONE |
| 13 | `calls` family extraction (8 opcodes; `CallMethod` deferred) | A14 | DONE_WITH_CONCERNS |
| 14 | `iterators` family extraction (6 opcodes) | A15 | DONE |
| 15 | `generators` family extraction (6 opcodes) | A16 | DONE |
| 16 | `exceptions` family extraction (4 opcodes) | A17 | DONE |
| 17 | `prefix` family extraction (2 opcodes) + `misc` orphans (2: `InstanceOf`, `CallMethod` stubs) + enable Tests 1, 4 | A18 | DONE_WITH_CONCERNS |
| 18 | Manifest Test 2 (semantic fn-ptr linker resolution) | A19 | DONE |

**Total opcodes covered: 152 / 152 (`OPCODE_COUNT`).**

Real semantic extractions: 148. Stubs (delegating to `op_unimplemented`): 4 (`InstanceOf`, `CallMethod`, `Wide`, `ExtraWide` — though `Wide` and `ExtraWide` have real semantic bodies; only `InstanceOf` and `CallMethod` are stubs because their α handlers are themselves stubs).

Files added under `crates/lyng/vm/src/vm/semantics/`: 12 (`loads.rs`, `arithmetic.rs`, `control_flow.rs`, `property.rs`, `names.rs`, `scope.rs`, `calls.rs`, `iterators.rs`, `generators.rs`, `exceptions.rs`, `prefix.rs`, `misc.rs`) plus `mod.rs`.

α handler files in `crates/lyng/vm/src/vm/dispatch_handlers/` thinned to operand-decode + `LlIntDispatchState::from_alpha(...)` + call-semantic + `translate_outcome_to_step(...)` — each handler is now 5-15 lines.

## 2. Exit-criterion verification (§10 DSL-0a)

The design lists five exit criteria:

| # | Criterion | Status | Evidence |
|--:|---|---|---|
| 1 | Every `Opcode` variant appears in `OPCODES` (Manifest Test 1) | ✓ | `crates/lyng/vm/src/dsl/opcode_manifest.rs::manifest_tests::opcodes_manifest_is_exhaustive` passes; `OPCODES.len() == OPCODE_COUNT == 152` |
| 2 | Every `semantic_symbol` resolves to a real function (Manifest Test 2) | ✓ | `crates/lyng/vm/src/dsl/opcode_manifest.rs::manifest_tests::semantic_fn_ptrs_resolve` passes; `SEMANTIC_FN_PTRS.len() == 152`, all non-null |
| 3 | Source-grep smoke test passes (Manifest Test 4) | ✓ | `crates/lyng/vm/tests/dsl_manifest_grep.rs::no_op_functions_outside_semantics_and_handlers` passes |
| 4 | `cargo test -p lyng-vm -p lyng-bytecode -p lyng-objects -p lyng-tests -p lyng-compiler` passes at same count as Pre-flight 4 | ✓ | 1796 passed (was 1793 at Pre-flight 4; +3 from new manifest tests), 1 ignored (was 1; the un-ignored gains are accounted by `+2`, and the remaining ignored is an unrelated doctest) |
| 5 | Test262 pass count ≥ Pre-flight 7 baseline (49728/49729) | ✓ — **gained** | 49729/49729 passing — 100% pass rate, **gained 1 test vs Pre-flight baseline.** See `reports/lyng/dsl-0a-test262.md`. |

## 3. V8 v7 evidence

Although the DSL-0a exit criterion only requires "geomean within ±2% of pre-DSL-0", every benchmark improved:

| Benchmark | Pre-DSL-0 | Post-DSL-0a | Δ |
|---|--:|--:|--:|
| Richards | 317 | 320 | +0.9% |
| DeltaBlue | 360 | 368 | +2.2% |
| Crypto | 256 | 277 | +8.2% |
| RayTrace | 417 | 450 | +7.9% |
| NavierStokes | 457 | 478 | +4.6% |
| Splay | 1342 | 1488 | +10.9% |

Reports: `reports/lyng/dsl-0a-v8.md` + `dsl-0a-v8.json`.

Hypothesis: the macro-driven thinning in several α families (arithmetic, names, scope, iterators) produces tighter codegen than the inline-everything α form had — LLVM gets a clearer view of which paths are hot vs cold. This is a happy side-effect of DSL-0a's structural refactor and not a designed-for outcome; the win is preserved into DSL-0b regardless of whether it survives the alpha-deletion in DSL-0c.

## 4. Known concerns / DONE_WITH_CONCERNS

### a. `CallMethod` extraction deferred (A14)

The `pub use calls::{...}` re-export in `dispatch_handlers/mod.rs` does NOT include `CallMethod` — the opcode is `op_unimplemented` in the dispatch table. A14 followed the task's explicit guidance ("If `CallMethod` is in the `pub use` line, extract it. If it's not, skip it.") and did not extract.

A18 registered `CallMethod` in `OPCODES` with a stub semantic (`op_call_method_semantic` returning `VmError::UnsupportedOpcode { code, instruction_offset, opcode: CallMethod }`). The stub is reachable via `SEMANTIC_FN_PTRS` (Test 2 passes). When `CallMethod` gets a real implementation, the stub is replaced and the manifest entry remains stable.

### b. Prefix α handler doesn't route through `translate_outcome_to_step` (A18)

The α handlers for `op_wide` / `op_extra_wide` deliberately do NOT advance PC past the prefix byte — the immediately-following semantic opcode reads its widened operands from the bytecode stream starting at the prefix position. Routing through `translate_outcome_to_step` would `state.advance(1)` and break the widened decode.

The α prefix handlers match `SemanticOutcome` outcomes directly:
- `Continue { pc_advance: 0 }` → dispatch to `DISPATCH_TABLE[bytes[pc+1]]` without advancing PC
- `ExitError` → return `Step::Error`

The semantic bodies return `pc_advance: 0` as a marker for the α handler. Behavior is equivalent (Test262 unaffected). This deviation from the family template is necessary; document is preserved in the prefix handler's source.

### c. Synchronous `Refresh` vs trampoline-epoch lazy refresh (A14)

The pre-A14 α handlers for `Call0..3` / `Call` / `Construct` did NOT call `refresh_from_active_frame()` on the caught-abrupt path — they relied on the trampoline's `frame_check_epoch` + `still_active` check to refresh lazily on the next loop iteration. The new semantic bodies return `SemanticOutcome::Refresh`, which causes `translate_outcome_to_step` to call `refresh_from_active_frame()` synchronously.

Both paths end up at the same active-frame PC before the next dispatch. The differences are micro:
- Synchronous refresh updates `state.frame_check_epoch` to match the VM's epoch inside the semantic; the trampoline's epoch check fires once more (no-op) and `still_active` returns true.
- One extra `still_active(state)` call per caught-abrupt-after-call. Cold path, no measurable cost.

Endorsed by the A14 task instructions and verified by Test262 + the focused suite (no regressions).

### d. `VmError::DoublePrefix` added (A18)

The pre-A18 α handler rejected `Wide; Wide; ...` with `VmError::InstructionOutOfBounds` (a workaround for not having a dedicated variant). A18 added `VmError::DoublePrefix` per the plan. Functionally equivalent (both abrupt-completion-exits from `Vm::run`); the named variant is more diagnostic.

## 5. Files / commits

**Commits on `claude/epic-saha-8f0b96` since the DSL-0 plan was committed (`a2271382`):** 20.

- A1: `b1ac9df9` ticket map + `027a0883` dcat state
- Pre-flight: `7c3991cb` V8+microbench baselines, `8bb6f826` Test262 baseline
- A2: `743ef4f7` dsl/ scaffold + manifest skeleton
- A3: `b7edc2b7` SemanticOutcome
- A4: `642e2d0f` transitional LlIntDispatchState
- A5: `369b49cc` semantics/ skeleton
- A6: `878e0b93` Manifest Test 1 (`#[ignore]`d)
- A7: `9a49155e` Manifest Test 4 (`#[ignore]`d)
- A8: `69d4ca94` loads family (37 opcodes)
- A9: `69a2292c` arithmetic family (29 opcodes)
- A10: `b2de2361` control_flow family (10 opcodes)
- A11: `313dab9d` property family (21 opcodes)
- A12: `8eb234bb` names family (17 opcodes)
- A13: `328d5cd6` scope family (10 opcodes)
- A14: `0661c461` calls family (8 opcodes)
- A15: `5668afdd` iterators family (6 opcodes)
- A16: `2ce6692a` generators family (6 opcodes)
- A17: `cd809f23` exceptions family (4 opcodes)
- A18: `1ae62ebe` prefix + misc + enable Tests 1, 4
- A19: `0368d27f` Manifest Test 2 (fn-ptr resolution)

**Major new paths:**

- `crates/lyng/vm/src/dsl/mod.rs`
- `crates/lyng/vm/src/dsl/opcode_manifest.rs` (~1500 lines — OpcodeEntry + OPCODES + SEMANTIC_FN_PTRS + Tests 1, 2)
- `crates/lyng/vm/src/dsl/slow_path.rs` (~70 lines — SemanticOutcome, LlIntDispatchState)
- `crates/lyng/vm/src/vm/semantics/{mod.rs, loads.rs, arithmetic.rs, control_flow.rs, property.rs, names.rs, scope.rs, calls.rs, iterators.rs, generators.rs, exceptions.rs, prefix.rs, misc.rs}` (~5000 lines total)
- `crates/lyng/vm/tests/dsl_manifest_grep.rs` (~70 lines)

**Touched policy:**

- `crates/lyng/vm/src/error.rs` — added `VmError::DoublePrefix` variant.
- `crates/lyng/vm/src/lib.rs` — `pub mod dsl;` and `pub(crate) mod vm;` (visibility widening for cross-module access).
- `crates/lyng/vm/src/vm.rs` — `pub(crate) mod dispatch_state;`, `pub(crate) mod semantics;`.
- `crates/lyng/vm/src/vm/install.rs` — `pub(crate) struct InstalledFunction` (transitive visibility from `DispatchState::installed`).

## 6. Hand-off to DSL-0b

DSL-0b lands the asm-DSL substrate:

- `lyng-vm-dsl` proc-macro crate
- `vm/src/dsl/` runtime ABI (`LlIntState`, `LlIntRustContext`, slow-path bridge types)
- FeedbackVector flat-array refactor (eager allocation, `Box<[FeedbackEntry]>` storage)
- AArch64 backend operation macros (~25 ops: operand decode, value-tag checks, object access, SMI arithmetic, dispatch, slow-path bridge, feedback, safepoint poll, opcode counter)
- 9 validation cases (empty handler, slow-path round-trip, PC-sync, 3 safepoint cases, 3 prefix cases)
- 5 hot DSL ports: `op_move`, `op_add`, `op_jump`, `op_return`, `op_loop_header`
- 5 warm DSL ports: `op_jump8`, `op_jump_if_true(/8)`, `op_jump_if_false(/8)`, `op_wide`, `op_extra_wide`
- ~140 cold DSL stubs (codegen-generated)
- DSL_DISPATCH_TABLE populated but inactive (alpha remains the dispatch path)

The α handlers retain their thin shape during DSL-0b. DSL-0c flips the dispatch path and deletes alpha + tier-accounting machinery.

DSL-0a dcat ticket (`lyng-3ne7`) and all 19 sub-tickets are in `in_review` awaiting user approval to close. **Per `crates/lyng/AGENTS.md`: tickets are NEVER closed without explicit user approval.**

## 7. Status

**Overall: DONE_WITH_CONCERNS.** All exit criteria met or exceeded:

- Manifest Tests 1, 2, 4 pass.
- 1796 focused-suite tests pass (vs 1793 Pre-flight baseline; the +3 are the new manifest tests).
- Test262 100% pass rate on runnable files (49729/49729) — gained one test vs Pre-flight 7 baseline.
- V8 v7 geomean improved across all 6 benchmarks; Crypto/RayTrace/Splay each +8% or more.

The DONE_WITH_CONCERNS qualifier reflects the four documented concerns above (CallMethod deferred, prefix handler deviation, synchronous-Refresh choice, VmError::DoublePrefix added) — each is intentional, audited, and preserves behavior.
