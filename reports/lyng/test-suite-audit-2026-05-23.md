# Lyng Test Suite Audit and Consolidation Plan

**Date:** 2026-05-23
**Scope:** All `#[test]` functions and integration tests across `crates/` and `tools/`. Excludes the test262 corpus itself (run separately via the `lyng-test262` binary).

---

## TL;DR

The suite is **2,783 `#[test]` functions across 190 files (~141k LOC of test code)** and a warm `cargo test --workspace` takes **~32s wall**. The "tests are slow because we have too many" framing is only half-right: ~half of those 32s is cargo's per-binary link/launch overhead, not test execution. Of the actual execution time, **two crates dominate** (`lyng-vm` 13.4s wall, `lyng-tests` 9.8s wall) and **one single test (`script_core_named_eval_recursion_uses_bytecode_call_guard`) burns 4.6s by itself** — ~15% of `lyng-tests`'s inner runtime.

There is genuine, large-scale duplication — but **80% removable is too aggressive**. A defensible target is **~30–40% of test functions removable (~800–1,000 tests)** plus substantial line-count reductions from parameterization, with no loss of meaningful coverage. Aggressive removal beyond that would gut engine-internal tests that test262 cannot replace (`crates/objects`, `crates/vm/src/tests/inline_caches.rs`, `crates/vm/src/tests/feedback.rs`, `crates/compiler/src/script/tests.rs`).

---

## 1. Current State (Inventory)

| Metric | Value |
|---|---|
| Total `#[test]` functions | **2,783** |
| Files containing `#[test]` | 190 |
| Total LOC in test files | ~141,253 |
| Parameterized tests (`rstest`, `test_case`, …) | **0** |
| Snapshot tests (`insta`, `expect-test`) | **0** |
| `#[ignore]` tests | 6 (5 intentional forward-pointers + 1 slow bench) |
| `TODO`/`FIXME` in crate test code | 1 |
| Largest test crates by `#[test]` count | `lyng-tests` (1,213) · `lyng-vm` (487) · `lyng-parser` (184) · `lyng-objects` (80) · `lyng-ops` (80) |
| Largest single test file by count | `crates/tests/src/parser_coverage.rs` — 193 tests, 834 LOC |

**Where the LOC is concentrated:** `crates/vm` test files alone are ~31k LOC (~22% of test LOC); `crates/tests` + `crates/test262-harness` + `tools/lyng-test262` together account for ~43k LOC.

---

## 2. Where Test Time Actually Goes

Warm-cache numbers (workstation, parallel test runner):

| Slice | Wall | Inner | Notes |
|---|---|---|---|
| Full workspace `cargo test` | **31.7s** | sum ≈17s | ~half of wall time is cargo per-binary overhead, not test logic |
| `lyng-vm` | **13.4s** | 1.2s | **12s of pure cargo/link overhead** from 15 separate `tests/dsl_validation_*.rs` integration files |
| `lyng-tests` | **9.8s** | 7.4s | The heaviest "real" test runtime |
| `tools/lyng-test262` | ~4.5s | 3.45s | 34 subprocess forks in `tests/harness.rs` |
| Every other crate | <0.55s | <0.25s | The frontend (lexer/parser/ast/sema/types) finishes in <0.13s each |

**Slowest single tests inside `lyng-tests`:**

| Test | File | Time |
|---|---|---|
| `script_core_named_eval_recursion_uses_bytecode_call_guard` | [regexp_and_annex_b.rs](crates/tests/src/execution_semantics/script_core/regexp_and_annex_b.rs) | **4.64s** |
| `frame_context_survives_gc_pressure_in_closure_loop` | [gc_stress_frame_context.rs](crates/tests/src/gc_stress_frame_context.rs) | 1.29s |
| `script_core_string_substr_matches_annex_b_numeric_edges` | regexp_and_annex_b.rs | 0.67s |
| `large_script_100k_lines_under_5s` | [smoke.rs](crates/tests/src/smoke.rs) | ~0.6s |
| All other ~1,200 tests in `lyng-tests` | — | ≤0.1s, mostly ~30–50ms |

The shape of the slow-test distribution is **long-tailed**: a handful of tests cost orders of magnitude more than the median. Removing those four tests would cut `lyng-tests` runtime by ~30% without removing any other test.

**Test262 is not part of `cargo test`.** The corpus is gated behind `cargo run -p lyng-test262`. So when test262 ships a passing slice, *any* `#[test]` whose only job is to reproduce that slice is automatically redundant.

---

## 3. Findings — Where the Bloat Actually Lives

Each finding lists a concrete diagnosis, evidence (file paths + counts), and how much of the suite it accounts for.

### F1. The same JS behavior is tested at up to seven layers

For most observable language constructs, coverage exists at:

| Layer | Location |
|---|---|
| 1. Lexer | `crates/lexer/src/tests.rs` |
| 2. Parser (structural) | `crates/parser/src/tests/{expressions,statements,declarations,…}.rs` (184 tests) |
| 3. Parser ("does it parse") | `crates/tests/src/parser_coverage.rs` (193 tests, 834 LOC) |
| 4. Sema (AST-by-hand) | `crates/sema/src/tests.rs` (49 tests, 2,423 LOC) |
| 5. Sema (real source) | `crates/tests/src/sema_integration.rs` (32 tests) + `end_to_end.rs` (14 tests) |
| 6. VM execution | `crates/vm/src/tests/{classes,eval_and_with,promises,…}.rs` (~340 tests) |
| 7. End-to-end JS | `crates/tests/src/execution_semantics/**` (~540 tests) + `temporal/**` (~220) |
| 8. Conformance | `tools/lyng-test262` (off-by-default) |

For "`let x = 1; let x = 2;` is a `SyntaxError`", layers 1–5 likely all touch it. For "`new Box(5).read()` returns the expected value", layers 6–8 all touch it.

**Concrete pairs already verified:**

| Behavior | First test | Second test (~duplicate) |
|---|---|---|
| `+` operator | `parser/src/tests/expressions.rs:332 parse_addition` | `tests/src/parser_coverage.rs:216 expr_add` |
| `+=` operator | `expressions.rs:576 parse_compound_assignment` | `parser_coverage.rs:324 expr_add_assign` |
| `while` | `parser/src/tests/statements.rs:151 parse_while_statement` | `parser_coverage.rs:443 stmt_while` |
| function creates scope | `sema/src/tests.rs:100 function_creates_scope` | `sema_integration.rs:59 function_creates_scope` |
| `let` is block-scoped | `sema/src/tests.rs:207 let_binding_is_block_scoped` | `sema_integration.rs:99 let_is_block_scoped` |
| direct eval semantics | `vm/src/tests/eval_and_with.rs:223 evaluate_script_direct_eval_matches_test262_global_env_rec` | `tests/src/execution_semantics/eval.rs:653 direct_eval_in_only_strict_script_matches_test262_assert_shape` |

**Estimated bloat:** ~400–500 redundant tests across layers 3 and 5–6.

### F2. Two parallel end-to-end suites with no rule about which goes where

`crates/vm/src/tests/` (≈340 of its 487 tests) and `crates/tests/src/execution_semantics/` (≈540 tests) **both** call `compile_and_run_string(...)` and assert on the stringified result. They are the same kind of test, organized along different axes (`vm/` by VM subsystem, `tests/` by JS feature). There is no documented split rule — and the actual overlap is large:

- Classes: 38 tests in `vm/src/tests/classes.rs` (692 LOC) vs. 73 in `tests/src/execution_semantics/classes.rs` (2,210 LOC).
- Eval/with: 52 in `vm/src/tests/eval_and_with.rs` vs. 62 in `tests/src/execution_semantics/eval.rs`.
- Promises/async/generators: 66 + 68 + 32 in `vm/src/tests/{async_and_generators,promises,generators}.rs` — three separate VM files all exercising the same `await`/promise/generator pipeline that test262 already covers comprehensively.

There are **831 calls** to `compile_and_run_string` across the workspace. Most of those calls describe a *language feature*, not a VM subsystem, and belong in one place.

**Estimated bloat:** ~200–300 tests where the VM-local and execution-semantics copies are testing the same surface behavior.

### F3. Hand-rolled clones, zero parameterization

The codebase has **zero** uses of `rstest`, `test_case`, `paste`, `seq_macro`, or any table-driven harness. Every test is a separate function. Several test files are essentially data tables disguised as code:

- [lexer/src/tests.rs:238–330](crates/lexer/src/tests.rs) — `decimal_integers`, `decimal_float`, `decimal_exponent`, `hex_literal`, `octal_literal`, `binary_literal`, `numeric_separators`, `hex_with_separators` — eight near-identical tests differing only in input string + expected float.
- [objects/src/tests.rs:307–520](crates/objects/src/tests.rs) — three clusters of near-clones (`named_property_handler_packs_*` ×5, `named_property_handler_none_*` ×5, `named_property_proto_handler_*` ×6).
- [parser/src/tests/statements.rs:151–197](crates/parser/src/tests/statements.rs) — `parse_while_statement`, `parse_do_while_statement`, `parse_for_statement`, `parse_for_in`, `parse_for_of` — each is one parse + one `matches!`.

In LOC terms, parameterizing these clusters wouldn't reduce the *test count* dramatically, but would cut **~3,000–5,000 LOC** of test code and make additions a one-line table row.

### F4. Tests whose stated purpose is "match test262"

Many tests in `crates/tests/src/execution_semantics/` have names ending in `_matches_test262_rows`, `_matches_test262_assert_shape`, `_matches_test262_symbol_rows`, etc. — i.e., they exist to reproduce a known test262 fixture inside `cargo test`.

Once test262 itself runs green on those fixtures (which is the standalone-tool path), these reproductions add zero new information and only cost runtime + maintenance. Sample:

- `tests/src/execution_semantics/classes.rs:phase6_method_and_accessor_name_descriptors_match_test262_symbol_rows`
- `tests/src/execution_semantics/classes.rs:phase6_private_destructuring_target_evaluation_matches_test262_rows`
- `tests/src/execution_semantics/eval.rs:653 direct_eval_in_only_strict_script_matches_test262_assert_shape`

**There are 538 phase-tagged tests** (`phase4_`/`phase5_`/`phase6_`/`script_core_`) in `crates/tests/src/` — these were checkpoints against a development plan, not minimized. Many are now redundant with test262.

**Estimated bloat:** ~150–250 tests that are pure test262 reproductions.

### F5. Re-export smoke tests

A handful of tests do nothing but assert that types compile and re-export — work `cargo build` already verifies for free.

- [crates/tests/src/runtime_primitives.rs:11](crates/tests/src/runtime_primitives.rs) `property_key_and_descriptor_surface_is_reexported`
- [crates/tests/src/runtime_primitives.rs:50](crates/tests/src/runtime_primitives.rs) `completion_surface_is_reexported`

A grep for `_surface_is_reexported` and `_surface_is_*` would find more. Small in count (~10–20) but pure noise.

### F6. Integration-test fragmentation in `crates/vm/tests/`

`crates/vm/tests/` has **15 separate `.rs` files** (`dsl_validation_*.rs`). Each file becomes its own separate test binary, separately linked. This is responsible for **~12 seconds of pure cargo/link overhead per `cargo test -p lyng-vm`** — more than 10x the actual test runtime in that crate.

Consolidating those 15 files into 1–2 binaries would save ~10s of wall time on `lyng-vm` without removing a single test. This is structural, not duplication.

### F7. Outlier tests

Four tests dominate `lyng-tests` runtime:

| Test | Time | Crate |
|---|---|---|
| `script_core_named_eval_recursion_uses_bytecode_call_guard` | 4.64s | lyng-tests |
| `frame_context_survives_gc_pressure_in_closure_loop` | 1.29s | lyng-tests |
| `script_core_string_substr_matches_annex_b_numeric_edges` | 0.67s | lyng-tests |
| `large_script_100k_lines_under_5s` | ~0.6s | lyng-tests |

These aren't bloat — they exist because something needs to be stressed at scale. But they should be **clearly identified as long-running tests** (either gated behind a feature flag, or moved out of the default `cargo test` run into a `cargo test --features=stress` lane).

---

## 4. What We Should NOT Remove

A consolidation push can over-correct. Things that look like "obvious bloat" but are actually load-bearing:

- **`crates/objects/src/tests.rs`** — exercises `ObjectRuntime`, `PrimitiveHeap`, `ObjectAllocation`, inline storage transitions, IC entry encoding. Engine-internal data structures that test262 cannot observe. Parameterize the obvious clusters (F3) but keep the coverage.
- **`crates/vm/src/tests/{inline_caches,feedback}.rs`** — exercise IC entry mutation and feedback-vector invariants. Black-box JS tests cannot reach these.
- **`crates/compiler/src/script/tests.rs`** (59 tests) — each asserts on emitted bytecode shape. Test262 only sees JS-visible results; if we don't assert here, bytecode regressions are silent.
- **`crates/gc/`** tests — rooting and allocator invariants. test262 doesn't probe these directly.
- **`crates/bytecode/src/builder.rs`** tests — encoding/decoding round-trips.
- **`crates/parser/src/tests/`** (the structural ones, not the "does it parse" duplicates in `tests/src/parser_coverage.rs`) — these assert on AST node shape, which is the parser's contract.

Net rule: **engine-internal tests stay; "does this JS source produce this string" tests should live in exactly one place.**

---

## 5. Honest Target

Per the inventory:

| Area | Realistic removable |
|---|---|
| `crates/tests/src/parser_coverage.rs` | 80%+ (193 → ~10–20) |
| `crates/tests/src/end_to_end.rs` | 100% (fold into `sema_integration.rs`) |
| `crates/sema/src/tests.rs` | ~50% (drop AST-by-hand variants where `sema_integration` covers it) |
| `crates/vm/src/tests/{classes,eval_and_with,promises,async_and_generators}.rs` | 30–50% (where they shadow `tests/src/execution_semantics/` or test262) |
| `crates/tests/src/execution_semantics/` | 20–30% (the `_matches_test262_*` reproductions) |
| `crates/objects/src/tests.rs` | 15–20% (parameterize handler clusters) |
| `crates/tests/src/runtime_primitives.rs` re-export smoke tests | 100% (small absolute count) |

Aggregate: ~800–1,000 of 2,783 tests realistically removable ⇒ **~30–40% of test count**, with another **~10–20k LOC reduction** from parameterizing what stays. **Going to 80% removed would require gutting engine-internal coverage** that test262 cannot replace.

---

## 6. Recommended Plan

Six phases, ordered from highest confidence/lowest risk to most structural.

### Phase 0 — Baseline & guardrails (½ day)

Before deleting anything, lock in a reproducible "before" picture so we can prove what was preserved:

1. Commit a captured baseline of `cargo test --workspace --no-fail-fast` output (test names + pass/fail), and the timings table from §2.
2. Add a `cargo test --workspace` to CI if it isn't already, with timing breakdown.
3. Run `cargo run --release -p lyng-test262 -- --report /tmp/lyng-test262-baseline.md -j 12` and commit the report alongside this audit so post-cleanup we can verify the conformance score is unchanged.

**Exit criteria:** baseline reports committed; we have a "what passed before" reference.

### Phase 1 — Quick wins (2–3 days, ~250 tests removed, ~10s wall-time saved)

Targeted deletions and consolidations with low judgement-call risk. Each step is its own PR so review is easy.

| # | Action | Removes | Wall-time impact |
|---|---|---|---|
| 1.1 | Delete `crates/tests/src/runtime_primitives.rs` re-export-only tests (and any `*_surface_is_reexported` siblings) | ~10–20 | trivial |
| 1.2 | Consolidate `crates/vm/tests/dsl_validation_*.rs` (15 files) into 1–2 files | 0 deleted, structural | **~10s wall on `lyng-vm`** |
| 1.3 | Gate the four slow outliers behind `cargo test --features=stress` or move to `cargo test --release` lane (`script_core_named_eval_recursion_uses_bytecode_call_guard`, `frame_context_survives_gc_pressure_in_closure_loop`, `script_core_string_substr_matches_annex_b_numeric_edges`, `large_script_100k_lines_under_5s`) | 0 from default run | ~7s on `lyng-tests` |
| 1.4 | Replace `tools/lyng-test262/tests/harness.rs` 34 subprocess invocations with a single batched runner call (or in-process invocation) | structural | ~3s on `tools/lyng-test262` |
| 1.5 | Delete `crates/tests/src/end_to_end.rs` after moving its 14 tests into `sema_integration.rs` | ~14 | trivial |

**Exit criteria:** `cargo test --workspace` wall time drops from ~32s to ~15s; ~250 tests removed; test262 conformance unchanged.

### Phase 2 — Collapse parser_coverage and sema duplication (3–5 days, ~250 tests removed)

This is the highest-leverage cleanup. Treat as one focused effort.

1. **`crates/tests/src/parser_coverage.rs` (193 tests → ~10–20):**
   - For each test, find the equivalent assertion in `crates/parser/src/tests/`.
   - If equivalent exists: delete from `parser_coverage.rs`.
   - If no equivalent: move into the appropriate `crates/parser/src/tests/*.rs` as a one-line table row, then delete from `parser_coverage.rs`.
   - Target: file shrinks from 834 LOC to ~50 LOC of "smoke" coverage (or is deleted entirely).
2. **`crates/sema/src/tests.rs` (49 tests → ~25):**
   - Identify pairs where an AST-by-hand test in `sema/src/tests.rs` has a real-source equivalent in `tests/src/sema_integration.rs`.
   - Delete the AST-by-hand version unless it exercises something `sema_integration.rs` can't (e.g., scope-table internals not visible from outside).
3. **Establish a written rule** (one paragraph in `crates/AGENTS.md`): "Parser tests live in `crates/parser/src/tests/`. Sema tests live in `crates/sema/src/tests.rs` for unit-level invariants and `crates/tests/src/sema_integration.rs` for real-source coverage. `crates/tests/src/parser_coverage.rs` does not exist." This is the most important deliverable of the phase — without the rule, the duplication grows back.

**Exit criteria:** `parser_coverage.rs` gone or reduced to ~10 tests; `sema/src/tests.rs` ~halved; AGENTS.md rule committed.

### Phase 3 — Decide canonical home for end-to-end JS tests (1–2 weeks, ~250 tests removed)

This is the largest single chunk and the highest judgement-call density. Recommend treating as a multi-PR effort with focused review on each move.

**Decision needed (ask the user before starting):** for tests that compile JS source and assert on the stringified result, which crate is canonical — `crates/vm/src/tests/` or `crates/tests/src/execution_semantics/`?

Recommendation: **`crates/tests/src/execution_semantics/` is canonical**, because:
- Its organization (by JS feature) is closer to how users and test262 think about language behavior.
- `crates/vm/src/tests/` should be reserved for things that need VM-internals inspection (debugger hooks, IC state, feedback, metadata, dispatch validation).

Then:
1. For each `crates/vm/src/tests/*.rs` file, walk its tests and decide: is this *VM-internal* (keep here) or *JS-behavioral* (move to `execution_semantics/` if not already covered, delete if duplicate)?
2. Apply ruthlessly to: `classes.rs`, `eval_and_with.rs`, `promises.rs`, `async_and_generators.rs`, `generators.rs`, `dynamic_import.rs`, `disposables.rs`.
3. Within `crates/tests/src/execution_semantics/`, audit the `_matches_test262_*` tests against current test262 pass-list. Delete any whose test262 fixtures are already passing in the standalone harness.

**Exit criteria:** ~150–250 VM-local JS tests moved or deleted; `vm/src/tests/` shrinks; documented rule in AGENTS.md ("VM tests cover VM internals, JS-behavioral tests live in `crates/tests/`").

### Phase 4 — Parameterize hand-rolled clones (ongoing background work, mostly LOC reduction)

Pick a parameterization approach (recommend `rstest` for `#[case]`-style or a tiny in-house table helper — adding `rstest` is a 1-line dependency add and well-justified per the test-quality goals). Then convert the obvious clusters:

- `crates/lexer/src/tests.rs` numeric/keyword tests
- `crates/objects/src/tests.rs` `named_property_handler_*` clusters
- `crates/parser/src/tests/statements.rs` loop-parsing tests
- `crates/builtins/src/bootstrap/tests.rs` constructor-shape tests

Expected: ~3,000–5,000 LOC reduction, ~50–100 tests collapsed into ~10 table-driven tests.

### Phase 5 — Standing rules to prevent re-bloat

Add to `crates/AGENTS.md` (or wherever the engineering standards live):

1. **No new `_matches_test262_*` tests.** test262 is the source of truth for spec conformance. Bugs found in test262 mode get a *regression* test, not a reproduction.
2. **Canonical home per layer:**
   - Lexer assertions → `crates/lexer/src/tests.rs`
   - Parser AST-shape assertions → `crates/parser/src/tests/*.rs`
   - Sema unit invariants → `crates/sema/src/tests.rs`
   - Sema on real source → `crates/tests/src/sema_integration.rs`
   - Compiler bytecode-shape → `crates/compiler/src/script/tests.rs`
   - VM-internals (IC, feedback, dispatch, debugger) → `crates/vm/src/tests/`
   - JS behavioral coverage → `crates/tests/src/execution_semantics/`
   - Engine-data-structure tests → owning crate
3. **No new "does it parse" tests separate from structural parser tests.**
4. **One canonical end-to-end harness function**, used everywhere. Today `compile_and_run_string` exists in multiple variants; pick one and document it.
5. **Long-running tests must declare themselves.** Anything >250ms gets the `stress` feature gate.
6. **CI guard:** track total `#[test]` count over time. A PR that grows the count by >10 has to justify it in the description.

### Phase 6 — Optional follow-ups

- Investigate whether the slowest test (`script_core_named_eval_recursion_uses_bytecode_call_guard`, 4.6s) is actually doing 4.6s of useful work or is hitting a perf path that should itself be the test (i.e., a bench).
- Look into whether `lyng-test262 --filter ...` slices can be invoked from `cargo test` as a smoke-conformance run, replacing many of the `_matches_test262_*` tests with a single "the slice still passes" assertion.

---

## 7. Risks & Mitigations

| Risk | Mitigation |
|---|---|
| Deleting a "duplicate" test that actually exercises a subtle internal path test262 misses | Phase 0 baseline; require test262 conformance number unchanged after each phase; review deletions in small PRs |
| Parameterization changes test discoverability (less obvious which case failed) | Use `rstest`'s `#[case]` naming, which produces named test reports per case |
| Phase 3 stalls because "VM-internal vs JS-behavioral" judgement calls accumulate | Resolve ambiguity by *default*: leave the test where it is and only move when there's a clear duplicate elsewhere |
| Rules in §5 are written but ignored over time | Add a CI check on total test count; reviewers reject PRs that grow it without justification |

---

## 8. Appendix: Per-Crate Inventory

| Crate | Files | `#[test]` | Notes |
|---|---|---|---|
| lyng-common | 3 | 24 | Keep |
| lyng-lexer | 1 | 88 | Parameterize numeric/keyword clusters |
| lyng-ast | 5 | 49 | Keep |
| lyng-parser | 8 | 184 | Canonical home for parser assertions |
| lyng-sema | 1 | 49 | Cut ~50%; drop AST-by-hand duplicates |
| lyng-types | 4 | 26 | Keep |
| lyng-gc | 6 | 51 | Keep — engine-internal |
| lyng-ops | 13 | 80 | Keep |
| lyng-host | 2 | 11 | Keep |
| lyng-objects | 2 | 80 | Parameterize handler clusters (~15–20% reduction) |
| lyng-env | 4 | 41 | Keep |
| lyng-bytecode | 6 | 45 | Keep |
| lyng-compiler | 4 | 71 | Keep — bytecode-shape asserts |
| lyng-vm | 45 | **487** | Consolidate `tests/dsl_validation_*` (15→2); move ~150–250 JS-behavioral tests out |
| lyng-builtins | 10 | 36 | Parameterize constructor-shape tests |
| lyng-cli | 2 | 18 | Keep |
| lyng-test262-harness | 0 | 0 | Library only |
| **lyng-tests** | 50 | **1,213** | Biggest cleanup target: delete `parser_coverage.rs`, fold `end_to_end.rs`, prune `_matches_test262_*`, slim `execution_semantics/` |
| lyng-vm-dsl | 1 | 5 | Keep |
| tools/lyng-bench | 12 | 86 | Keep |
| tools/lyng-test262 | 11 | 139 | Replace 34 subprocess fixtures in `tests/harness.rs` with one batched call |

---

## 9. Estimated End State

| Metric | Before | After (target) |
|---|---|---|
| `#[test]` count | 2,783 | ~1,700–1,900 |
| Test LOC | ~141k | ~90–110k |
| `cargo test --workspace` wall | ~32s | ~12–15s |
| `cargo test -p lyng-vm` wall | ~13.4s | ~3s |
| `cargo test -p lyng-tests` wall | ~9.8s | ~5s |
| Parameterized tests | 0 | clusters that benefit from it |
| Layers testing the same JS behavior | up to 7 | 1 (with rule documented) |
| Outlier tests (>250ms) | 4 in default run | 0 in default run; gated behind `--features=stress` |

The combination of (a) deletion of layered duplication, (b) consolidation of integration-test files in `lyng-vm/tests/`, and (c) gating the slowest 4 outliers behind a feature flag is what produces most of the wall-time saving. Parameterization gives LOC and maintenance savings but only modest runtime savings.
