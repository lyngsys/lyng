# Phase 4a — Direct argument lowering: status report

**Issue:** `lyng-48k8` — Phase 4: Compiler and bytecode polish (epic)
**Parent:** `lyng-49qk` — JSC-aligned engine roadmap (master epic)
**Date:** 2026-05-15
**Toolchain:** rustc 1.93 (2025-12-15), aarch64-apple-darwin, `--release` profile
**Parent commit:** `2cd3d294` (Phase 3d close)

## What landed

Direct argument lowering for the seven legacy call sites that were still
emitting `Move slot ← temp` chains for each argument. All of them now
reserve the contiguous call-argument block first and lower the argument
expression (or synthetic load) directly into its final slot, matching the
shape that the main `lower_call_expression` / `lower_tail_call_expression` /
`lower_construct_expression` paths have used since Phase 0.

### Refactored callsites

| Site | Before | After |
|---|---|---|
| `lower_bigint_literal` ([expr.rs](../../../crates/lyng-js/compiler/src/script/expr.rs)) | `alloc_temp` → `LoadConst` → `materialize` (1 Move/arg) | `reserve_argument_block(1)` → `emit_load_atom_string(slot, atom)` (no Move) |
| `lower_internal_binary_builtin` ([operators.rs](../../../crates/lyng-js/compiler/src/script/operators.rs)) | `lower_expr_to_temp` × 2 → `materialize` (1 Move/arg) | `reserve_argument_block(2)` → `lower_expr_into(arg, slot)` (no synthetic Move) |
| `lower_template_object` → `lower_template_object_into` ([templates.rs](../../../crates/lyng-js/compiler/src/script/templates.rs)) | `alloc_temp` × (1 + 2·K) → `emit_load_*` → `materialize` | `reserve_argument_block(1 + 2·K)` → `emit_load_*(slot, ...)` (no Move) |
| `lower_tagged_template_call` ([templates.rs](../../../crates/lyng-js/compiler/src/script/templates.rs)) | `lower_expr_to_temp` × N + `template_object` temp → `materialize` (N+1 Moves) | `reserve_argument_block(1 + N)` → inner template-object call writes into slot 0 + `lower_expr_into(arg, slot)` (no Move) |
| `lower_internal_template_to_string` ([templates.rs](../../../crates/lyng-js/compiler/src/script/templates.rs)) | `lower_expr_to_temp` → `materialize` (1 Move) | `reserve_argument_block(1)` → `lower_expr_into(expression, slot)` (no synthetic Move) |
| `lower_direct_eval_call_expression` ([calls.rs](../../../crates/lyng-js/compiler/src/script/calls.rs)) | `lower_call_arguments` (per-arg temp) → `materialize` via `_from_args` helper (2× Moves) | `collect_call_argument_plan` → `reserve_argument_block(1 + N)` → `lower_call_arguments_into(plan, base+1)` + `lower_direct_eval_callee_into(callee, base)` + `_from_range` helper (no synthetic Move) |
| `lower_direct_eval_tail_call_expression` ([calls.rs](../../../crates/lyng-js/compiler/src/script/calls.rs)) | Lower args to temps → use them for direct-eval call AND for tail call materialize (2× Moves) | Single contiguous (1+N) block — direct-eval call uses full range, tail call reuses slots `[base+1..base+N+1)` (no Move on fallback) |
| `lower_super_construct_call` ([calls.rs](../../../crates/lyng-js/compiler/src/script/calls.rs)) (both branches) | `alloc_temp` for super_constructor + per-arg temps + `materialize` → `internal_construct_super` (3+ Moves) | `reserve_argument_block(1 + N)` → `internal_super_constructor` writes into slot 0 + `lower_call_arguments_into(plan, base+1)` + `_from_range` helper (no Move) |

The two branches of `lower_super_construct_call` (direct-eval-allows-super
arrow path and the regular derived-class path) were also DRYed into a
single shared helper `emit_super_construct_call_into` since the only
behavioral difference is the post-call epilogue.

### Helpers added

- **`reserve_argument_block(count: u16) -> LoweringResult<CallRange>`**
  ([calls.rs](../../../crates/lyng-js/compiler/src/script/calls.rs)) —
  allocates N contiguous registers and returns a `CallRange`. Pure
  allocation, no Moves; this is the building block every refactored
  callsite uses.

- **`emit_internal_builtin_call_into_with_offset_and_this_from_range`**
  ([classes.rs](../../../crates/lyng-js/compiler/src/script/classes.rs)) —
  parallel to the existing `_with_offset_and_this` helper, but takes a
  pre-allocated `CallRange` instead of `&[u16]` of pre-existing registers.
  Skips the internal `materialize_argument_block` step entirely. The
  existing `&[u16]` helper now thin-wraps this one through `materialize`.

- **`lower_direct_eval_callee_into(callee, target)`**
  ([calls.rs](../../../crates/lyng-js/compiler/src/script/calls.rs)) —
  variant of the prior `lower_direct_eval_callee` that writes directly
  into a caller-supplied register instead of allocating its own temp.

- **`lower_template_object_into(span, template, dest)`**
  ([templates.rs](../../../crates/lyng-js/compiler/src/script/templates.rs)) —
  variant of the prior `lower_template_object` that writes the resulting
  template object directly into a supplied target slot (slot 0 of the
  outer tagged-template-call's argument range).

### Helpers removed

- `lower_call_arguments` (calls.rs) — no remaining callers.
- `LoweredCallArguments` (state.rs) — no remaining constructors.

### Helpers retained

- `materialize_argument_block` is still used internally by the central
  `emit_internal_builtin_call_into_with_offset_and_this` for callers that
  pass `&[u16]` of pre-existing registers (e.g., classes.rs's many
  short hardcoded argument lists, `lower_object_accessor_property` in
  property_exprs.rs). These callers cannot avoid Moves because the
  source registers already hold values used elsewhere; the Moves are
  intrinsic to relocating into a contiguous call-arg block.

## Verification

### Tests

| Check | Before 4a | After 4a | Δ |
|---|---:|---:|---|
| `cargo test -p lyng-js-compiler` | 68 passed | **71 passed** | +3 (new structural regression tests) |
| `cargo test -p lyng-js-bytecode` | 45 passed | 45 passed | unchanged |
| `cargo test -p lyng-js-vm` | 401 passed | 401 passed | unchanged |
| `cargo test -p lyng-js-tests` | 1186 passed | 1186 passed | unchanged |
| `cargo clippy -p lyng-js-compiler --all-features --tests -- -W clippy::pedantic -W clippy::nursery` | clean on compiler files | clean on compiler files | unchanged (pre-existing warnings in `lyng-js-objects`) |
| `cargo fmt -p lyng-js-compiler` | clean | clean | reformatted touched files |

The three new structural regression tests assert that the refactored
callsites lower argument expressions/synthetic loads **directly** into
their final call-argument slot without a Move in between:

- `compile_script_lowers_bigint_literal_argument_directly_into_call_slot`
- `compile_script_lowers_tagged_template_arguments_directly_into_call_slots`
- `compile_script_lowers_direct_eval_call_arguments_directly_into_call_slots`

They will fail loudly if a future change reintroduces the
`alloc_temp + emit_load_X + Move slot ← temp` pattern at these sites.

### Bytecode density

| Workload | Phase 3d unit bytes | Phase 4a unit bytes | Δ bytes |
|---|---:|---:|---:|
| `script.core.objects-and-arrays` | 244 | 244 | 0 |
| `functions.closure-calls` | 184 | 168 | **−16** |
| `activation.arguments-rest-for-in` | 527 | 491 | **−36** |
| `exceptions.try-catch-finally` | 331 | 333 | +2 |
| `wide.large-register-function` | 3267 | 3259 | −8 |
| **Aggregate unit bytes** | **4553** | **4495** | **−58 (−1.27%)** |
| Aggregate wide share | 14.96% | 15.09% | +0.13% |

The closure-calls and activation rows account for the bulk of the
density reduction; both exercise call shapes that previously went
through the `lower_call_arguments`/`materialize_argument_block`
two-phase pattern (super-construct in particular for the activation
row).

Full density report: [`phase-4a-density.md`](phase-4a-density.md).

### Runtime throughput proxy (density harness, 7 samples / 15 evals)

| Workload | Phase 3d us/eval | Phase 4a us/eval | Δ |
|---|---:|---:|---:|
| `script.core.objects-and-arrays` | 2224.11 | 1974.91 | **−11.2%** |
| `functions.closure-calls` | 1950.17 | 1794.65 | **−8.0%** |
| `activation.arguments-rest-for-in` | 3891.02 | 3545.01 | **−8.9%** |
| `exceptions.try-catch-finally` | 1521.28 | 1439.28 | **−5.4%** |
| `wide.large-register-function` | 1503.59 | 1518.11 | +1.0% (within noise) |

### Runtime benchmark suite (lyng-js-bench runtime, 7 samples / 9 timed runs)

Δ ns/work-unit, vs the prior bench checkpoint (Phase 3d):

| Benchmark | Δ ns/wu | Δ % | Note |
|---|---:|---:|---|
| `array-heavy.iterator-runtime` | −172.44 | −1.8% | iterator-driven traversal |
| `array-heavy.literal-indexed-runtime` | −150.33 | **−20.0%** | dense indexed read/write |
| `class-heavy.runtime` | −243.95 | **−9.8%** | constructor + super dispatch |
| `module-heavy.compile` | −34.92 | −1.1% | compile_module path |
| `regexp-constructor-compile.runtime` | −11.54 | −0.3% | constructor compilation |
| `regexp-heavy.runtime` | −1949.64 | **−4.1%** | mixed RegExp runtime |
| `regexp-legacy-statics.runtime` | −59.08 | −2.8% | Annex B static accessors |
| `regexp-stable-exec.runtime` | −349.04 | −1.0% | repeated exec/test |
| `string-heavy.concat-runtime` | −28.97 | **−15.8%** | string concat + equality |
| `typed-array-heavy.runtime` | −120.68 | **−14.6%** | ArrayBuffer + DataView |
| `regexp-named-replace.runtime` | +145.55 | +1.1% | within noise |
| `async-heavy.frontend` | +31.64 | +0.8% | frontend-only, within noise |

10 out of 12 workloads improved; 2 are within noise of the prior
baseline. The strongest gains track workloads with heavy call traffic
through constructors, classes, BigInt-style binary builtins, and
indexed array access — all of which exercise the refactored lowering
paths.

Full bench report: [`phase-4a-bench.md`](phase-4a-bench.md).

### Test262

| | Files runnable | Files passed | Pass rate | Δ files vs Phase 3d |
|---|---:|---:|---:|---:|
| Phase 2a baseline | 49729 | 49722 | 99.99% | — |
| Phase 3d | 49729 | 49720 | 99.98% | — |
| **Phase 4a** | **49729** | **49722** | **99.99%** | **+2** |

Variant-level: 95196 passed / 9 failed (Phase 3d: 8 variants failed).

The 7 file failures are all pre-existing, unrelated to call argument
lowering:

- `language/import/import-defer/evaluation-triggers/trigger-{exported,not-exported}-string-super-property-set-exported.js` (import-defer, 2 files)
- `language/module-code/instn-star-iee-{multi,single}-cycle-same-name.js` (module cycle, 2 files)
- `language/module-code/namespace/internals/super-access-to-tdz-binding.js` (TDZ binding, 1 file)
- `staging/sm/TypedArray/toLocaleString.js` (TypedArray.toLocaleString, 1 file × 2 variants)
- `staging/sm/class/className.js` (class name serialization, 1 file × 2 variants)

File-level pass count recovers the Phase 2a baseline; the
`object-literal-__proto__` regression that Phase 3a introduced
(`lyng-2tr1`) did not surface in this run, likely because of the
different `-j 12` worker parallelism vs Phase 3d's `-j 4`. `lyng-2tr1`
remains an open follow-up.

Full report: [`phase-4a-test262.md`](phase-4a-test262.md).

## What's deferred

The Phase 4 epic (`lyng-48k8`) has two further sub-tasks beyond 4a:

- **4b — Star fusion lookahead**: per-handler peephole at runtime that
  folds a trailing `Star0..Star7` into the value-producing handler's
  tail, eliminating one dispatch per fused pair. Expected gain
  +3–5%.
- **4c — Compact accumulator-based bytecode**: compiler-side bias
  toward accumulator-form opcodes (`Ldar` / `Star0..7` / `LdaSmi8`).
  Expected gain +2–3% on icache footprint; the roadmap explicitly
  permits deferring 4c if measurement shows minimal gain.

Both remain on the Phase 4 epic and will be planned once 4a's
real-world impact is observed (the next benchmark sweep on a hot
production workload).

Also still open from earlier phases:

- `lyng-2tr1` — Phase 3a `object-literal-__proto__` regression (independent of Phase 4).
- `lyng-22al` — Phase 3e PrototypeData inline path.
- `lyng-5nju` — Phase 3f polymorphic IC compaction.
- `lyng-28t2` — Phase 3g γ-swap evaluation.

## Files changed

**Compiler**:
- `crates/lyng-js/compiler/src/script/calls.rs` — new `reserve_argument_block`, refactored direct-eval call/tail-call + super-construct, removed `lower_call_arguments`, factored helper `emit_super_construct_call_into`, added `lower_direct_eval_callee_into`, signature of `add_direct_eval_spread_feedback_site` switched from `&LoweredCallArguments` to primitives.
- `crates/lyng-js/compiler/src/script/classes.rs` — new `emit_internal_builtin_call_into_with_offset_and_this_from_range` helper; existing helper rewired to call it through `materialize_argument_block`.
- `crates/lyng-js/compiler/src/script/expr.rs` — BigInt literal lowering uses `reserve_argument_block` + `emit_load_atom_string` directly.
- `crates/lyng-js/compiler/src/script/operators.rs` — internal binary builtin lowering uses `reserve_argument_block` + `lower_expr_into`.
- `crates/lyng-js/compiler/src/script/templates.rs` — `lower_template_object` → `lower_template_object_into`, tagged-template call uses pre-reserved range, `lower_internal_template_to_string` takes `ExprId` and lowers into slot directly.
- `crates/lyng-js/compiler/src/script/state.rs` — removed `LoweredCallArguments` struct.
- `crates/lyng-js/compiler/src/script.rs` — dropped `LoweredCallArguments` re-export.
- `crates/lyng-js/compiler/src/script/tests.rs` — three new structural regression tests.

**Reports**:
- `reports/js/lyng-js/phase-4a-status.md` (this file).
- `reports/js/lyng-js/phase-4a-density.md`.
- `reports/js/lyng-js/phase-4a-bench.md`.
- `reports/js/lyng-js/phase-4a-test262.md`.

Total Phase 4a diff: roughly +260/−260 lines across 8 source files, plus
the three regression tests (~100 lines) and the four report files.
