# Move Op Reduction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce unnecessary compiler-emitted Move-family dispatches without changing JavaScript semantics.

**Architecture:** Keep the optimization in `lyng-compiler` lowering. Do not change VM opcode semantics or bytecode encoding; the builder already compacts `Move` where possible. Add narrow helpers for safe source-register reuse and effect-only expression lowering, then verify with compiler bytecode-shape tests and V8-v7 opcode counters.

**Tech Stack:** Rust 2024 workspace, `lyng-compiler`, `lyng-bytecode`, `lyng-bench v8suite --count-opcodes`.

---

### Task 1: Compiler Shape Tests

**Files:**
- Modify: `crates/compiler/src/script/tests.rs`

- [x] **Step 1: Add tests for the first-pass patterns**

Add tests that assert:
- arithmetic using frame-local `var` operands does not first copy those operands into temporary registers;
- unused expression statements do not emit a Move into a throwaway result register;
- `for` update expressions such as `i++` do not copy the update result into an unused temporary.

- [x] **Step 2: Run the focused tests and confirm RED**

Run: `cargo test -p lyng-compiler move_reduction -- --nocapture`

Expected: at least one new test fails before implementation.

### Task 2: First-Pass Lowering Changes

**Files:**
- Modify: `crates/compiler/src/script/expr.rs`
- Modify: `crates/compiler/src/script/variables.rs`
- Modify: `crates/compiler/src/script/stmt.rs`
- Modify: `crates/compiler/src/script/loops.rs`
- Modify: `crates/compiler/src/script/property_exprs.rs`

- [x] **Step 1: Add safe source-register reuse**

Add a compiler helper that returns an existing frame-local register only for identifier reads that do not require TDZ checks and are not dynamic, captured-environment, global, or arguments-aliasing reads. Use it from operand-lowering paths where later evaluation cannot invalidate the source register.

- [x] **Step 2: Add effect-only expression lowering**

Add `lower_expr_for_effect` for contexts where the expression value is ignored. Start with update expressions so `i++` and `++i` still perform their read, write, and TDZ behavior, but skip the final copy into a destination register.

- [x] **Step 3: Use effect-only lowering in statement and loop update contexts**

Route expression statements without a script result register and `for` update expressions through `lower_expr_for_effect`.

### Task 3: Verification

**Files:**
- No source edits expected.

- [x] **Step 1: Run compiler tests**

Run: `cargo test -p lyng-compiler`

Expected: all compiler tests pass.

- [x] **Step 2: Run current V8-v7 opcode counts**

Run: `cargo run --release -p lyng-bench -- v8suite --count-opcodes --samples 1 --counts-json /tmp/lyng-v8-v7-opcode-counts-after-move-pass.json`

Expected: the report is written and `Move` count/share is no higher than the pre-change baseline from `/tmp/lyng-v8-v7-opcode-counts-current.json`.
