# VM Dispatch Phase 4/5 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Inline the hottest VM dispatch handlers and tighten checked register access without violating Lyng JS's no-unsafe policy.

**Architecture:** Keep the Phase 3 local dispatch frame model. Add small, checked register-window helpers for hot arms, then flatten arithmetic handling so dispatch arms do not rematch opcode families through generic helper layers.

**Tech Stack:** Rust, Lyng JS VM bytecode dispatch, `cargo test`, strict Clippy, Test262 filtered slices, `cargo asm`.

---

### Task 1: Structural RED Tests For Hot Dispatch Shape

**Files:**
- Modify: `crates/lyng-js/vm/src/vm/dispatch.rs`

- [x] **Step 1: Add failing structural tests**

Add tests in the existing `#[cfg(test)] mod tests` in `dispatch.rs`:

```rust
#[test]
fn dispatch_loop_inlines_hot_abc_handlers() {
    let source = include_str!("dispatch.rs");
    let run_loop = source
        .split("\n    fn run_dispatch_loop")
        .nth(1)
        .expect("dispatch loop should stay in this module");

    assert!(
        !run_loop.contains(".execute_abc_value_opcode("),
        "hot ABC dispatch should not rematch opcode families through execute_abc_value_opcode"
    );
}

#[test]
fn move_arm_uses_direct_register_window_access() {
    let source = include_str!("dispatch.rs");
    let move_arm = source
        .split("Opcode::Move => {")
        .nth(1)
        .and_then(|tail| tail.split("Opcode::Add").next())
        .expect("Move arm should stay directly before Add-family arms");

    assert!(
        !move_arm.contains("self.read_register(") && !move_arm.contains("self.write_register("),
        "Move should use direct checked register-window access in the hot dispatch arm"
    );
}
```

- [x] **Step 2: Verify RED**

Run: `cargo test -p lyng-js-vm dispatch_loop_inlines_hot_abc_handlers move_arm_uses_direct_register_window_access`

Expected: Cargo only accepts one filter, so if this exact command errors, run each test by name. Each test should fail against the current code for the expected structural reason.

### Task 2: Direct Checked Register Access For Move

**Files:**
- Modify: `crates/lyng-js/vm/src/vm/dispatch.rs`
- Modify: `crates/lyng-js/vm/src/vm/registers.rs`

- [x] **Step 1: Add checked window helper**

Add a VM helper that returns the absolute checked index for an active frame register. Keep it private to VM internals and use normal checked indexing, not unsafe.

- [x] **Step 2: Inline Move**

Replace the Move arm with direct checked indexing over `self.register_stack`, using the current frame register base once.

- [x] **Step 3: Verify GREEN**

Run: `cargo test -p lyng-js-vm move_arm_uses_direct_register_window_access`.

Expected: The structural Move test passes.

### Task 3: Inline SMI-Safe Arithmetic Arms

**Files:**
- Modify: `crates/lyng-js/vm/src/vm/dispatch.rs`
- Modify: `crates/lyng-js/vm/src/vm/dispatch/arithmetic.rs`

- [x] **Step 1: Flatten hot ABC handler dispatch**

Move the opcode-family match currently inside `execute_abc_value_opcode` into the central dispatch arms for the hot arithmetic opcodes. Keep cold/spec helpers in `arithmetic.rs` for coercive paths.

- [x] **Step 2: Add integer fast paths**

Inline safe SMI fast paths for `Add`, `Sub`, `BitAnd`, `AddSmi`, `SubSmi`, and `BitAndSmi`. For `Mul`/`MulSmi` and `Mod`/`ModSmi`, only use a SMI result when the ECMA-262 negative-zero cases cannot occur; otherwise fall through to existing number semantics.

- [x] **Step 3: Verify GREEN**

Run: `cargo test -p lyng-js-vm dispatch_loop_inlines_hot_abc_handlers`.

Expected: The structural ABC test passes.

### Task 4: Correctness And Negative-Zero Coverage

**Files:**
- Modify: `crates/lyng-js/vm/src/tests/core.rs` or the closest existing numeric VM test module after inspection.

- [x] **Step 1: Add behavior regression test**

Add a VM test that evaluates arithmetic cases preserving `-0` for multiplication and modulo fallbacks, plus integer fast-path sanity checks for add/sub/bit-and.

- [x] **Step 2: Verify focused behavior**

Run: `cargo test -p lyng-js-vm <new_test_name>`.

Expected: The new behavior test passes.

### Task 5: Verification And Issue Handoff

**Files:**
- Modify: `reports/js/lyng-js/vm-dispatch-phase4-5-status.md`
- Update: `lyng-1z4u` dcat issue

- [x] **Step 1: Run verification**

Run:

```bash
cargo test -p lyng-js-vm
cargo test -p lyng-js-tests
cargo clippy -p lyng-js-vm --all-targets -- -W clippy::pedantic -W clippy::nursery
cargo run --release -p lyng-js-test262 -- --filter built-ins/Math/sign --report /tmp/lyng-js-test262-math-sign.md -j 4
cargo run --release -p lyng-js-test262 -- --filter built-ins/Object/is --report /tmp/lyng-js-test262-object-is.md -j 4
cargo run --release -p lyng-js-test262 -- --filter language/expressions/multiplication --report /tmp/lyng-js-test262-mul.md -j 4
cargo run --release -p lyng-js-test262 -- --filter language/expressions/remainder --report /tmp/lyng-js-test262-rem.md -j 4
cargo asm --lib --build-type release "lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::run_dispatch_loop"
git diff --check
```

Expected: Tests and lint pass; Test262 slices have 0 failures/panics; cargo-asm evidence shows the hot `Move` path no longer calls generic register helpers and the run loop no longer calls `execute_abc_value_opcode`.

- [x] **Step 2: Record and mark review**

Add a dcat comment with implementation and verification evidence, then run:

```bash
dcat update lyng-1z4u --status in_review
```
