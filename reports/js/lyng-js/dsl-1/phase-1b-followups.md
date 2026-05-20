# Phase 1.B follow-ups (tracking)

Pinned 2026-05-20 (Phase 1.B cleanup batch 1) to track outstanding
gaps surfaced during Phase 1.B.0–1.B.2 that should not be forgotten.
Each item names the document that surfaced it and proposes a
concrete next step.

## 1. JS-level test for `ThisState::Uninitialized` (op_load_this)

**Surfaced in:**
[`reports/js/lyng-js/dsl-handlers/op_load_this.md`](../dsl-handlers/op_load_this.md)
§ "Coverage gap (documented)", paragraph beginning "the
ThisState::Uninitialized arm (derived constructor pre-super()
access throwing ReferenceError) is NOT directly tested through JS".

**Current coverage state:**
- The structural sentinel mechanism is exercised end-to-end via the
  arrow-function tests in
  `crates/lyng-js-tests/tests/op_load_this_inline.rs` (which trigger
  the Lexical arm) and via the structural validation handler in
  `crates/lyng-js/vm/tests/dsl_validation_frame_context.rs`
  (`load_uninit_lex_sentinel_handler_compiles_and_links`, opcode 213).
- The Uninitialized arm itself is exercised via the
  `op_load_this_semantic` slow path which existing language tests
  indirectly cover.
- What's missing: a JS-level integration test that triggers the
  Uninitialized arm directly via canonical
  derived-constructor-before-super() syntax.

**Blocker:**
Class-inheritance + super() flow support in the lyng-js
compiler/runtime is not yet fully exercised by the integration test
suite. Constructing the TDZ scenario reliably requires careful
class-syntax setup the parser/compiler should support but isn't
covered end-to-end yet.

**Proposed next step:**
Once parser/compiler support for class-inheritance + super() flow
is reliable (likely tracked in a separate epic), add a single
integration test in `crates/lyng-js-tests/tests/op_load_this_inline.rs`
that constructs a derived constructor accessing `this` before
`super()`, expects a ReferenceError, and confirms the
`op_load_this_slow_rs` slow path fired (e.g., via opcode-counters
slow-path-share telemetry). Estimated scope: ~30 minutes once the
class+super() prerequisite is in place.

## 2. asm-diff registry extension for `dsl::handlers::cold::*`

**Surfaced in:**
- [`reports/js/lyng-js/dsl-handlers/op_load_const8.md`](../dsl-handlers/op_load_const8.md)
  § "Current asm (AArch64)" — "Captured from
  `target/release/deps/lyng_js_vm-*.s` after a `cargo rustc --release
  -p lyng-js-vm --lib -- --emit=asm -C debuginfo=0` build."
- [`reports/js/lyng-js/dsl-handlers/op_load_this.md`](../dsl-handlers/op_load_this.md)
  § "Current asm (AArch64)" — same manual-capture mechanism.
- [`reports/js/lyng-js/dsl-1/phase-1b2-summary.md`](phase-1b2-summary.md)
  § "Per-handler asm baseline approach diverged from Phase 1.A" —
  "The `asm-diff --check` tool does not yet support the
  `dsl::handlers::cold::*` namespace."
- [`reports/js/lyng-js/dsl-1/phase-1b2-summary.md`](phase-1b2-summary.md)
  recommended next steps item 3.

**Current state:**
- The `lyng-js-bench asm-diff` tool walks a registry of handler
  symbols and compares emitted asm against checked-in baselines.
- Phase 1.A used the registry-driven flow.
- Phase 1.B.2's two ports (`op_load_const8`, `op_load_this`) live
  under `crates/lyng-js/vm/src/dsl/handlers/cold.rs` and are NOT in
  the asm-diff registry. The asm baselines under
  `reports/js/lyng-js/dsl-asm-baseline-aarch64/` were captured
  manually.

**Proposed next step:**
Extend the asm-diff tool to auto-discover handlers in
`dsl::handlers::cold::*` (and likely `dsl::handlers::hot::*` too if
that namespace ever expands). The discovery mechanism likely involves
either: (a) registering DSL handlers via a per-handler proc-macro
attribute that publishes the symbol into a `linkme`-style
distributed-slice, or (b) emitting a generated registry file from
the DSL lowerer alongside the asm output. Estimated scope: ~1-2 hours
once the discovery approach is chosen. Should land before Phase 1.B.3
closure so the locals + Ldar + LoadEnvSlot ports use the structured
tool rather than continuing the manual-capture flow.

## 3. Snippets-coverage audit before each sub-phase

**Surfaced in:**
[`reports/js/lyng-js/dsl-1/phase-1b2-summary.md`](phase-1b2-summary.md)
§ "Microbench snippets gap" — "The Phase 1.B.0 microbench Task 7
'added 14 snippets' commit (`ad240f50`) was inspected at planning
time and appeared to include `LoadConst8` + `LoadThis` based on the
referenced Phase 1.B.0 summary table. It does not."

**Current state:**
- Cleanup batch 1 (this batch) closed the immediate gap for
  `LoadConst8` + `LoadThis` (commit `922ff5f2`).
- The underlying process gap — "summary tables can be wrong; trust
  `grep`" — is documented inline in both the Phase 1.B.0 correction
  note and the Phase 1.B.2 lessons section.
- No automated guard currently fails the build / report generation
  if a hot-30 opcode lacks a snippet (the existing
  `snippets_cover_hot_opcodes_or_emit_warning` test only prints
  warnings).

**Proposed next step:**
Before each port sub-phase (currently Phase 1.B.3 will be the next
one), explicitly run
```bash
grep -E 'opcode: "(LoadLocal0|LoadLocal1|.../*the in-scope set*/)"'
  tools/lyng-js-bench/src/microbench/snippets.rs
```
to confirm every in-scope opcode has a snippet. Cheap step (~2 min)
that would have caught the Phase 1.B.0 → Phase 1.B.2 gap at planning
time rather than at report-writing time. Optional follow-up: harden
the existing `snippets_cover_hot_opcodes_or_emit_warning` test to
fail (not warn) for the configured-as-hot opcode set, with an
explicit allow-list of opcodes intentionally missing snippets.

## 4. Mandatory-reviewer prompt: substrate validation runtime check

**Surfaced in:**
[`reports/js/lyng-js/dsl-1/phase-1b1-summary.md`](phase-1b1-summary.md)
§ "Retrospective: structural-only validation tests insufficient for
substrate macros".

**Current state:**
The Phase 1.B.1 mandatory reviewer dispatch (Task 9, commit
`4ff25b9b`) approved the sub-phase as "APPROVED, 0 high/medium
findings, 2 low addressed inline" without flagging that the Task 6
validation tests are structural-only. The x22→x24 register-pin bug
in `load_constant!` / `load_state_value!` was latent through that
approval.

**Proposed next step:**
Extend the `feature-dev:code-reviewer` prompt template (or whatever
the in-house reviewer skill uses) for substrate-only sub-phases to
include the explicit question: "Do the validation tests
runtime-dispatch through the new substrate macros? If not, has
substrate validation been explicitly deferred to a named follow-up
sub-phase?" This is a process change, not a code change. Scope:
~10 minutes once the reviewer prompt template is identified.
