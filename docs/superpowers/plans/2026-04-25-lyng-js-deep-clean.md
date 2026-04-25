# Lyng JS Deep Clean Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Recover Lyng JS architecture and readability without slowing ECMA-262 conformance work into unsafe, unreviewable patches.

**Architecture:** Work in small ownership-oriented slices under `lyng-2gmp`. Builtin metadata/bootstrap/dispatch, object abstract operations, and dynamic compilation each get a canonical owner and tests at that owner boundary. Broad conformance work in a touched area should wait until the relevant cleanup guardrails are in place.

**Tech Stack:** Rust workspace crates under `crates/lyng-js/*`, dcat issue tracking, Cargo unit/integration tests, targeted Test262 slices.

---

## Task 1: Finish Public Builtin Metadata Extraction

**Files:**
- Modify: `crates/lyng-js/builtins/src/public.rs`
- Create: `crates/lyng-js/builtins/src/public/metadata.rs`

- [ ] Create one guard test per remaining builtin family that asserts the family table length, family lookup, and `public_builtin_metadata` agree.
- [ ] Move remaining metadata rows from the monolithic chain into family tables without changing builtin IDs, names, lengths, constructibility, or prototype flags.
- [ ] Move all metadata table types and lookup helpers into `public/metadata.rs`; re-export only `public_builtin_metadata`.
- [ ] Verify with `cargo test -p lyng-js-builtins --lib`, `cargo fmt --all --check`, and `cargo clippy -p lyng-js-builtins --lib --all-features -- -W clippy::pedantic`.
- [ ] Set dcat task `lyng-4xqj` to `in_review` with verification output and commit.

## Task 2: Split Public Builtin Bootstrap by Family

**Files:**
- Modify: `crates/lyng-js/builtins/src/public.rs`
- Create: `crates/lyng-js/builtins/src/public/families/*.rs`

- [ ] Extract family bootstrap descriptor data and installation calls into family modules.
- [ ] Keep `ensure_public_realm_builtins` as orchestration over typed family installers.
- [ ] Preserve the existing `PublicRealmBuiltins` fields and initialization order unless a test proves order is irrelevant.
- [ ] Verify with `cargo test -p lyng-js-builtins --lib` and targeted `lyng-js-tests` bootstrap/runtime coverage.
- [ ] Add or update docs in `docs/lyng-js/builtin-bootstrap.md` when the module ownership changes.

## Task 3: Make Object Ops Proxy-Aware by Default

**Files:**
- Modify: `crates/lyng-js/ops/src/object.rs`
- Modify: `crates/lyng-js/ops/src/proxy.rs`
- Modify callers in `crates/lyng-js/builtins` and `crates/lyng-js/vm`

- [ ] Add owner-layer tests showing `Get`, `GetOwnProperty`, `HasProperty`, `Set`, `DefineOwnProperty`, prototype operations, and own-key operations route through proxy traps from the canonical `lyng-js-ops::object` surface.
- [ ] Introduce `ObjectOpsContext` only where call/coercion/error behavior is needed for proxy traps.
- [ ] Keep raw storage/internal-method helpers private or explicitly named as internal/bootstrap-only.
- [ ] Migrate builtin and VM call sites away from direct `agent.objects()` semantic access where an ops function should own the behavior.
- [ ] Verify with `cargo test -p lyng-js-ops`, `cargo test -p lyng-js-objects`, `cargo test -p lyng-js-vm`, and targeted `lyng-js-test262 --filter built-ins/Proxy`.

## Task 4: Extract Shared Dynamic Compilation

**Files:**
- Modify: `crates/lyng-js/compiler/Cargo.toml`
- Create: `crates/lyng-js/compiler/src/dynamic.rs`
- Create: `crates/lyng-js/vm/src/vm/dynamic_compilation.rs`
- Modify: `crates/lyng-js/vm/src/vm/builtin_dispatch.rs`

- [ ] Add `lyng-js-parser` as a normal compiler dependency.
- [ ] Add `lyng_js_compiler::dynamic` request/key/result types for Function constructor source wrapping, script/eval parse goals, sema mode, compile, diagnostics, and cache-key policy.
- [ ] Move VM installed-code cache and evaluation glue into `vm/dynamic_compilation.rs`.
- [ ] Leave VM frame inspection and caller environment/private environment discovery in VM.
- [ ] Verify Function constructor, indirect eval, direct eval, and evalScript with `cargo test -p lyng-js-compiler`, `cargo test -p lyng-js-vm`, and targeted Test262 eval/function filters.
- [ ] Update `docs/lyng-js/dynamic-scope-and-eval.md` when the shared service API exists.

## Task 5: Split Public Builtin Dispatch by Semantic Family

**Files:**
- Modify: `crates/lyng-js/builtins/src/public/dispatch.rs`
- Create or extend: `crates/lyng-js/builtins/src/public/dispatch/*.rs`

- [ ] Move cohesive dispatch groups into family modules using the existing Temporal split as the pattern.
- [ ] Keep the public dispatch ABI and `BuiltinFunctionId` identity unchanged.
- [ ] Add compile-time or unit-test coverage that a family-owned dispatch table rejects duplicate builtin IDs where practical.
- [ ] Verify with `cargo test -p lyng-js-builtins --lib` and family-specific `lyng-js-tests`/Test262 slices for moved dispatch groups.

## Task 6: Burn Down Builtins Clippy Warning Backlog

**Files:**
- Modify warning hotspots in `crates/lyng-js/builtins/src/public.rs`, `public/dispatch.rs`, `public/temporal.rs`, and `public/dispatch/temporal.rs`

- [ ] Treat each warning cluster as a small semantic-preserving cleanup task.
- [ ] Add tests first when changing observable control flow, conversions, or error behavior.
- [ ] Prefer targeted `#[allow]` only when the spec algorithm or hot-path shape justifies keeping the warning.
- [ ] Verify with the documented clippy command and the nearest unit/integration tests.

## Operating Rules

- Preserve builtin IDs, public realm behavior, handle layouts, bytecode encoding, and VM frame layout.
- Do not add new third-party dependencies.
- Do not close dcat issues without explicit user approval.
- Move each child issue to `in_review` only after verification and a dcat comment with the commands run.
- Commit each coherent slice separately.
