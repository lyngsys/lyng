# DSL-1 Phase 1.B.3 — Locals + Ldar — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Inline-port 9 top-30 opcodes (LoadLocal0/1/2/3, StoreLocal0/1/2/3, Ldar — combined ~1.38B+ dispatches/V8 v7 run) using existing Phase 1.B.1/1.B.2 substrate, and land cumulative V8 v7 ≥ +3% vs pre-DSL-0 HEAD `d850f261`.

**Architecture:** 9 opcodes are pure register-window moves with no bail conditions. Each handler body is 2 instructions: a load via `load_acc!`/`load_local_fixed!`/`load_reg!` and a store via `store_reg!`/`store_local_fixed!`/`store_acc!`. Two new tiny backend macros (`load_local_fixed!`, `store_local_fixed!`) emit single-instruction fixed-offset accesses for compile-time-known slot indices. No new substrate; no GC integration changes.

**Tech Stack:** Rust + AArch64 `naked_asm!`, `#[repr(C)]`, cargo workspace.

**Spec:** [`docs/superpowers/specs/2026-05-20-dsl-1-phase-1b3-locals-and-ldar-design.md`](../specs/2026-05-20-dsl-1-phase-1b3-locals-and-ldar-design.md).
**Baseline HEAD:** `08727f92` (post-cleanup mid-phase).
**Pre-DSL-0 reference HEAD (for cumulative A/B):** `d850f261`.

---

## File structure overview

### Created
- `crates/lyng-js/vm/src/dsl/backend/aarch64/locals.rs` — new `load_local_fixed!` + `store_local_fixed!` macros
- `crates/lyng-js-tests/tests/op_locals_inline.rs` — JS-level integration tests for the 8 LoadLocal/StoreLocal opcodes
- `crates/lyng-js-tests/tests/op_ldar_inline.rs` — JS-level integration tests for Ldar
- `reports/js/lyng-js/dsl-handlers/op_load_local_0.md` (+ similar for 1, 2, 3) — per-handler ported reports
- `reports/js/lyng-js/dsl-handlers/op_store_local_0.md` (+ similar for 1, 2, 3) — per-handler ported reports
- `reports/js/lyng-js/dsl-handlers/op_ldar.md` — per-handler ported report
- `reports/js/lyng-js/dsl-asm-baseline-aarch64/op_load_local_{0,1,2,3}.asm` — asm baselines
- `reports/js/lyng-js/dsl-asm-baseline-aarch64/op_store_local_{0,1,2,3}.asm` — asm baselines
- `reports/js/lyng-js/dsl-asm-baseline-aarch64/op_ldar.asm` — asm baseline
- `reports/js/lyng-js/dsl-1/phase-1b3-ab-comparison.md` — same-load A/B vs `08727f92`
- `reports/js/lyng-js/dsl-1/phase-1b3-cumulative-ab.md` — cumulative A/B vs `d850f261`
- `reports/js/lyng-js/dsl-1/phase-1b3-microbench.md` — microbench results + slow-path-share
- `reports/js/lyng-js/dsl-1/phase-1b3-summary.md` — sub-phase summary

### Modified
- `crates/lyng-js/vm/src/dsl/backend/aarch64/mod.rs` — declare new `locals` submodule
- `crates/lyng-js/vm/src/dsl/handlers/cold.rs` — replace 9 `call_slow!` shims with inline ports (and optionally delete dead slow-path stubs)
- `crates/lyng-js/vm/tests/dsl_validation_frame_context.rs` — add 2 structural compiles-and-links tests for the new macros
- `tools/lyng-js-bench/src/microbench/snippets.rs` — add 3 missing snippets (StoreLocal0, StoreLocal1, StoreLocal2)
- `reports/js/lyng-js/dsl-1/phase-1b-followups.md` — record op_load_env_slot deferral formally

### Untouched
- All `LlIntState` fields, lowerer bindings, GC code, entry/slow-path infrastructure.

---

## Conventions for this plan

- **User deny rules:** NEVER use `git -C <path>` or `cd <path> && git ...`. Run git from the worktree's working directory.
- **NEVER skip hooks** (`--no-verify`), force-push, or destructive ops without explicit user consent.
- **Commits:** Each task ends with a self-contained commit. HEREDOC + `Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>` footer.
- **Untracked planning docs** (`docs/superpowers/{plans,specs}/*.md`): leave untouched per user discipline.
- **`reports/js/lyng-js/bench-v8.md`** is a bench-tool side-effect. May be modified or clean depending on entry state; never stage it.
- **Behavioral parity at every commit:** `cargo test -p lyng-js-vm --lib --release` (≥418), `cargo test -p lyng-js-tests --release` (≥1198). 2 pre-existing `feedback_flat_consistency` failures stay unrelated.
- **TDD discipline** for the macros (Task 1) and per-opcode integration tests (Tasks 2 + 3). Write tests first, verify they pass with the cold stub, then replace the stub — the same tests must continue passing.
- **Post-audit lesson honored:** every new backend macro must be exercised by REAL handler dispatch within this sub-phase, not just structural compiles-and-links.

---

## Task 1: Add `load_local_fixed!` + `store_local_fixed!` backend macros

**Files:**
- Create: `crates/lyng-js/vm/src/dsl/backend/aarch64/locals.rs`
- Modify: `crates/lyng-js/vm/src/dsl/backend/aarch64/mod.rs` (declare new submodule)
- Modify: `crates/lyng-js/vm/tests/dsl_validation_frame_context.rs` (add 2 structural tests)

- [ ] **Step 1: Read the existing operands macros for template shape**

```bash
cat crates/lyng-js/vm/src/dsl/backend/aarch64/operands.rs | head -150
```

Pay attention to `load_acc!` (operands.rs:126), `store_acc!` (operands.rs:136), and `load_reg!` (operands.rs:106). Your new `load_local_fixed!` is the fixed-immediate-index sibling of `load_reg!` (`load_reg!` takes an x-register index; yours takes a u8 compile-time literal).

- [ ] **Step 2: Create `crates/lyng-js/vm/src/dsl/backend/aarch64/locals.rs`**

```rust
//! Fixed-immediate-index register-window load/store macros for DSL-1
//! Phase 1.B.3.
//!
//! `op_load_local_N` and `op_store_local_N` (N in 0..3) hardcode the
//! source/destination local slot index. Materializing N into a scratch
//! register and using `load_reg!`/`store_reg!` would cost 2 instructions
//! (movz + ldr/str); the fixed-offset form below costs 1 (ldr/str with
//! immediate offset).
//!
//! Both macros emit a single instruction:
//!
//! ```text
//!     ldr  x{dst}, [x20, #{N*8}]      ; load_local_fixed!
//!     str  x{src}, [x20, #{N*8}]      ; store_local_fixed!
//! ```
//!
//! x20 is the REGS pin (register-window base). `N*8` is the byte offset
//! because each register slot is a 64-bit `Value`.
//!
//! Spec §2 (Phase 1.B.3 design).

/// Load a `Value` from the register-window slot at fixed index `$N`
/// into `$dst_reg`. `$N` must be a literal in `0..=255` (the asm
/// immediate-offset range for a `ldr` with positive offset is wider,
/// but bytecode register indices fit in u8).
///
/// One instruction: `ldr x{dst}, [x20, #(N*8)]`.
///
/// Usage from a handler body (the lowerer substitutes `dst` to a
/// register number literal before macro expansion):
///
/// ```ignore
/// load_local_fixed!(1 => dst);
/// ```
#[macro_export]
macro_rules! load_local_fixed {
    ($n:literal => $dst_reg:tt) => {
        concat!(
            "ldr    x", stringify!($dst_reg), ", [x20, #", stringify!($n), " * 8]\n",
        )
    };
}

/// Store the `Value` in `$src_reg` into the register-window slot at
/// fixed index `$N`. Mirror of `load_local_fixed!`.
///
/// One instruction: `str x{src}, [x20, #(N*8)]`.
#[macro_export]
macro_rules! store_local_fixed {
    ($src_reg:tt, $n:literal) => {
        concat!(
            "str    x", stringify!($src_reg), ", [x20, #", stringify!($n), " * 8]\n",
        )
    };
}
```

- [ ] **Step 3: Declare the new submodule in `aarch64/mod.rs`**

Open `crates/lyng-js/vm/src/dsl/backend/aarch64/mod.rs`. Add `pub mod locals;` alongside the other `pub mod ...;` declarations (e.g., `pub mod constants;`, `pub mod frame;`).

- [ ] **Step 4: Build to verify the macros compile**

Run: `cargo build -p lyng-js-vm --release`
Expected: clean. No handler uses the macros yet; this only confirms `macro_rules!` syntax is valid.

- [ ] **Step 5: Add 2 structural compiles-and-links tests in dsl_validation_frame_context.rs**

Open `crates/lyng-js/vm/tests/dsl_validation_frame_context.rs`. Alongside the existing 4 structural handlers, add 2 more:

```rust
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_test_load_local_fixed_dsl,
    opcode_byte = 214, layout = A, length = 2, |a| {
        // Loads register 1 via fixed-immediate-offset access; stores
        // to dest register `a`. Structural — symbol must compile.
        load_local_fixed!(1 => 10);
        store_reg!(a, 10);
        dispatch!();
    }
}

#[cfg(target_arch = "aarch64")]
#[test]
fn load_local_fixed_handler_compiles_and_links() {
    let ptr = op_test_load_local_fixed_dsl as *const ();
    assert!(!ptr.is_null());
}

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_test_store_local_fixed_dsl,
    opcode_byte = 215, layout = A, length = 2, |a| {
        // Reads dest register `a` value; stores to local slot 2 via
        // fixed-immediate-offset. Structural.
        load_reg!(a => 10);
        store_local_fixed!(10, 2);
        dispatch!();
    }
}

#[cfg(target_arch = "aarch64")]
#[test]
fn store_local_fixed_handler_compiles_and_links() {
    let ptr = op_test_store_local_fixed_dsl as *const ();
    assert!(!ptr.is_null());
}
```

**These structural tests are intentional first-line-defense; per the post-audit lesson, Tasks 2-3 add runtime-dispatch coverage via canonical opcodes that exercise the macros end-to-end.**

- [ ] **Step 6: Run the new tests**

Run: `cargo test -p lyng-js-vm --test dsl_validation_frame_context --release`
Expected: 6 tests passing (4 existing + 2 new). 0 ignored.

If the macros don't emit valid asm, the build fails — switch macro internals (e.g., adjust the `#(N*8)` literal-expression syntax) until rustc accepts it.

- [ ] **Step 7: Run full vm + tests suites for parity**

Run in parallel:
```bash
cargo test -p lyng-js-vm --lib --release
cargo test -p lyng-js-tests --release
```
Expected: 418+ vm, 1198+ lyng-js-tests (parity maintained).

- [ ] **Step 8: Commit**

```bash
git add crates/lyng-js/vm/src/dsl/backend/aarch64/locals.rs crates/lyng-js/vm/src/dsl/backend/aarch64/mod.rs crates/lyng-js/vm/tests/dsl_validation_frame_context.rs
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.3 Task 1: load_local_fixed! + store_local_fixed! backend macros

Adds two tiny single-instruction backend macros for fixed-immediate-index
register-window access:

- `load_local_fixed!(N => dst)` emits `ldr x{dst}, [x20, #(N*8)]`
- `store_local_fixed!(src, N)` emits `str x{src}, [x20, #(N*8)]`

Used in Tasks 2 + 3 by op_load_local_1/2/3 and op_store_local_0/1/2/3
to avoid materializing the fixed slot index in a scratch register.
op_load_local_0 maps to existing `load_acc!` (slot 0 = accumulator);
op_ldar uses existing `load_reg!`/`store_acc!`.

Structural compiles-and-links tests added in dsl_validation_frame_context.rs.
The new macros will be exercised by REAL handlers in Tasks 2 + 3 — heeding
the Phase 1.B.1 retrospective lesson that structural-only validation is
insufficient for substrate macros.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Inline-port the 4 `op_load_local_N` handlers

**Files:**
- Modify: `crates/lyng-js/vm/src/dsl/handlers/cold.rs:4253-4365` (replace 4 cold stubs)
- Create: `crates/lyng-js-tests/tests/op_locals_inline.rs` (integration tests covering all 8 LoadLocal/StoreLocal cases — written here, used by Task 3 too)

- [ ] **Step 1: Discover the lyng-js-tests script-running helper**

```bash
grep -rn "fn.*Value\b\|pub fn run_script\|pub fn evaluate_script\|fn execute_script" crates/lyng-js-tests/src/ 2>/dev/null | head -10
```

Find the existing helper that runs a JS source and returns a `Value` (or `VmResult<Value>`). Use that same helper in your new test file. Phase 1.B.2's `op_load_const8_inline.rs` and `op_load_this_inline.rs` are existing examples; read them:

```bash
head -30 crates/lyng-js-tests/tests/op_load_const8_inline.rs
```

Match the convention.

- [ ] **Step 2: Write the failing integration tests FIRST (they should pass with the cold stub)**

Create `crates/lyng-js-tests/tests/op_locals_inline.rs`:

```rust
//! Phase 1.B.3 Tasks 2 + 3: integration tests for the inline
//! op_load_local_N + op_store_local_N ports.
//!
//! Each test exercises one or more of the 4 LoadLocal opcodes
//! (slots 0..3, via parameter access) and the 4 StoreLocal opcodes
//! (slots 0..3, via local-variable update in a loop or assignment).
//! Tests pass with the cold-stub OR the inline port — the inline port
//! must produce the same observable semantics.

use lyng_js_tests::run_script_returning_value;  // adjust to match the actual helper name

#[test]
fn load_local_0_returns_first_parameter() {
    // First parameter sits at register 0 (accumulator); reading it
    // via parameter access triggers LoadLocal0 in the bytecode.
    let value = run_script_returning_value(
        "(function(a) { return a; })(42)"
    ).expect("script ran");
    assert_eq!(value.to_number(), 42.0);
}

#[test]
fn load_local_1_returns_second_parameter() {
    let value = run_script_returning_value(
        "(function(a, b) { return b; })(10, 20)"
    ).expect("script ran");
    assert_eq!(value.to_number(), 20.0);
}

#[test]
fn load_local_2_returns_third_parameter() {
    let value = run_script_returning_value(
        "(function(a, b, c) { return c; })(10, 20, 30)"
    ).expect("script ran");
    assert_eq!(value.to_number(), 30.0);
}

#[test]
fn load_local_3_returns_fourth_parameter() {
    let value = run_script_returning_value(
        "(function(a, b, c, d) { return d; })(10, 20, 30, 40)"
    ).expect("script ran");
    assert_eq!(value.to_number(), 40.0);
}

#[test]
fn load_locals_aggregate() {
    // Exercises LoadLocal0 + LoadLocal1 + LoadLocal2 + LoadLocal3 in
    // a single expression. Validates indexing is correct (not just
    // always slot 0).
    let value = run_script_returning_value(
        "(function(a, b, c, d) { return a + b + c + d; })(1, 2, 3, 4)"
    ).expect("script ran");
    assert_eq!(value.to_number(), 10.0);
}

#[test]
fn store_local_3_updates_local_var() {
    // A function with 3 declared params + 1 local var likely places
    // the local at slot 3 (after params). Updating it exercises
    // StoreLocal3.
    let value = run_script_returning_value(r#"
        (function(a, b, c) {
            var x = 100;
            x = a + b + c + x;
            return x;
        })(1, 2, 3)
    "#).expect("script ran");
    assert_eq!(value.to_number(), 106.0);
}

#[test]
fn store_local_0_1_2_via_assignments() {
    // A function with multiple parameters whose values are
    // overwritten exercises StoreLocal0/1/2.
    let value = run_script_returning_value(r#"
        (function(a, b, c) {
            a = a * 2;       // StoreLocal0
            b = b * 3;       // StoreLocal1
            c = c * 4;       // StoreLocal2
            return a + b + c;
        })(10, 20, 30)
    "#).expect("script ran");
    // a=20, b=60, c=120 → sum 200
    assert_eq!(value.to_number(), 200.0);
}

#[test]
fn locals_in_tight_loop_sum() {
    // Stress test: tight loop using StoreLocal3 (loop counter / accumulator)
    // and LoadLocal[N] reads. Exercises both opcode families heavily.
    let value = run_script_returning_value(r#"
        (function() {
            var i = 0;
            var sum = 0;
            for (i = 0; i < 100; i++) {
                sum = sum + i;
            }
            return sum;
        })()
    "#).expect("script ran");
    // 0+1+...+99 = 4950
    assert_eq!(value.to_number(), 4950.0);
}
```

**Note on the actual local-slot placement:** the lyng-js bytecode compiler decides which local variable goes to which slot. The tests above assume parameters take slots 0..N-1 and locals are placed after. If the compiler diverges (e.g., uses different slot ordering), the test SEMANTICS still hold (correct return value) even if the specific opcodes fired differ. The integration tests verify END-TO-END correctness, not opcode-specific dispatch — that's what microbench + asm baselines verify in Task 4.

- [ ] **Step 3: Run the tests — they should pass with cold stubs (TDD red-green-refactor: this is the "the spec is met before the change" baseline)**

Run: `cargo test -p lyng-js-tests --test op_locals_inline --release`
Expected: 8 passing (or however many you wrote). They pass because cold stubs produce correct semantics.

If any test fails BEFORE you touch the handlers, that's an unrelated bug — investigate before proceeding.

- [ ] **Step 4: Replace the 4 `op_load_local_N` cold stubs with inline ports**

In `crates/lyng-js/vm/src/dsl/handlers/cold.rs`, find the 4 `llint_handler!` invocations (around lines 4255, 4284, 4313, 4342 per the research). Replace each with the inline form.

For `op_load_local_0_dsl` (opcode 144, slot 0 = accumulator):

```rust
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_load_local_0_dsl, opcode_byte = 144, layout = A, length = 2, |a| {
        // Phase 1.B.3: inline read of accumulator (register 0) into
        // dest register `a`. Slot 0 is the accumulator by convention
        // (see `load_acc!` macro doc in operands.rs).
        load_acc!(10);       // ldr x10, [x20]
        store_reg!(a, 10);   // str x10, [x20, x_a, lsl #3]
        dispatch!();
    }
}
```

For `op_load_local_1_dsl` / `op_load_local_2_dsl` / `op_load_local_3_dsl` (opcodes 145/146/147):

```rust
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_load_local_N_dsl, opcode_byte = (144 + N), layout = A, length = 2, |a| {
        // Phase 1.B.3: inline read of register N into dest register `a`.
        load_local_fixed!(N => 10);  // ldr x10, [x20, #(N*8)]
        store_reg!(a, 10);            // str x10, [x20, x_a, lsl #3]
        dispatch!();
    }
}
```

Substitute N = 1, 2, 3 literally in each handler (the lowerer's macro substitution requires the literal to be visible at macro-expand time). DO NOT try to parameterize across handlers — write 3 separate copy-paste invocations.

**Important:** the macro `load_local_fixed!` was added in Task 1; if its actual call syntax differs from `load_local_fixed!(N => 10)`, check the Task 1 source for the real signature.

- [ ] **Step 5: Check whether the slow-path stubs are now dead**

```bash
grep -rn "op_load_local_[0-3]_slow_rs" crates/lyng-js/
```

If the 4 `op_load_local_N_slow_rs` functions have no callers outside their own definition, delete them. If something else calls them (e.g., a wide/extra-wide prefix variant), keep them. Document either way in the commit message.

- [ ] **Step 6: Build to verify the inline ports compile**

Run: `cargo build -p lyng-js-vm --release`
Expected: clean. If a macro signature mismatch arises, adjust.

- [ ] **Step 7: Run the integration tests — they must still pass**

Run: `cargo test -p lyng-js-tests --test op_locals_inline --release`
Expected: still passing. The inline ports produce the same observable semantics as the cold stubs.

- [ ] **Step 8: Run full vm + tests suites for parity**

Run in parallel:
```bash
cargo test -p lyng-js-vm --lib --release
cargo test -p lyng-js-tests --release
```
Expected: 418+ vm, 1198+ lyng-js-tests + 8 new from op_locals_inline.rs = 1206+ total.

- [ ] **Step 9: Commit**

```bash
git add crates/lyng-js/vm/src/dsl/handlers/cold.rs crates/lyng-js-tests/tests/op_locals_inline.rs
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.3 Task 2: op_load_local_0/1/2/3 inline ports

Replaces 4 call_slow! shims with 2-instruction inline reads from the
register window:

- op_load_local_0: load_acc! (slot 0 = accumulator)
- op_load_local_{1,2,3}: load_local_fixed!(N => x10) + store_reg!(a, 10)

All 4 are pure register-window moves with no bail conditions — slow-path
expected 0.00% on V8 v7.

Dead-code: op_load_local_{0,1,2,3}_slow_rs <KEPT/REMOVED> (refactor worker
confirms via grep at impl time).

Integration test op_locals_inline.rs (also covers StoreLocal — used by
Task 3) verifies semantic parity vs the cold stubs.

Microbench + asm baseline + slow-path-share gates: Task 4.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Inline-port the 4 `op_store_local_N` handlers + `op_ldar`

**Files:**
- Modify: `crates/lyng-js/vm/src/dsl/handlers/cold.rs` (lines 3957 area for op_ldar, 4369-4467 area for store_local 0/1/2/3)
- Create: `crates/lyng-js-tests/tests/op_ldar_inline.rs` (integration tests for Ldar)

- [ ] **Step 1: Write the Ldar integration tests FIRST**

Create `crates/lyng-js-tests/tests/op_ldar_inline.rs`:

```rust
//! Phase 1.B.3 Task 3: integration tests for the inline op_ldar port.
//!
//! Ldar = "Load Accumulator from Register" — copies registers[a] into
//! the accumulator (register 0). The bytecode compiler emits Ldar
//! after temporaries are computed when the next opcode expects the
//! value in the accumulator.

use lyng_js_tests::run_script_returning_value;  // match Task 2's helper

#[test]
fn ldar_via_intermediate_temporary() {
    // (a + b) * 2 — the compiler computes (a + b) into a temp, then
    // Ldar's it before the multiply.
    let value = run_script_returning_value(
        "(function(a, b) { var c = a + b; return c * 2; })(1, 2)"
    ).expect("script ran");
    assert_eq!(value.to_number(), 6.0);
}

#[test]
fn ldar_in_chained_arithmetic() {
    let value = run_script_returning_value(
        "(function(a, b, c) { var x = a + b; var y = x + c; return y * 10; })(1, 2, 3)"
    ).expect("script ran");
    // a+b = 3; (a+b)+c = 6; *10 = 60
    assert_eq!(value.to_number(), 60.0);
}

#[test]
fn ldar_with_function_call_result() {
    let value = run_script_returning_value(r#"
        (function() {
            function add(x, y) { return x + y; }
            var r = add(3, 4);
            return r + 1;
        })()
    "#).expect("script ran");
    assert_eq!(value.to_number(), 8.0);
}
```

- [ ] **Step 2: Run the tests — they should pass with the cold stub**

Run: `cargo test -p lyng-js-tests --test op_ldar_inline --release`
Expected: 3 passing. Cold stub produces correct semantics.

- [ ] **Step 3: Replace the 4 `op_store_local_N` cold stubs with inline ports**

In `crates/lyng-js/vm/src/dsl/handlers/cold.rs`, find the 4 store-local stubs (opcodes 148, 149, 150, 151 — around lines 4371, 4400, 4429, 4458). Replace each with:

```rust
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_store_local_N_dsl, opcode_byte = (148 + N), layout = A, length = 2, |a| {
        // Phase 1.B.3: inline read of source register `a` into x10;
        // write to local slot N.
        load_reg!(a => 10);              // ldr x10, [x20, x_a, lsl #3]
        store_local_fixed!(10, N);        // str x10, [x20, #(N*8)]
        dispatch!();
    }
}
```

Substitute N = 0, 1, 2, 3 literally in each of the 4 handlers.

- [ ] **Step 4: Replace the `op_ldar_dsl` cold stub with inline port**

In `crates/lyng-js/vm/src/dsl/handlers/cold.rs` around line 3959, replace:

```rust
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_ldar_dsl, opcode_byte = 130, layout = A, length = 2, |a| {
        // Phase 1.B.3: inline accumulator-load. Reads source register
        // `a` into x10; writes to register 0 (accumulator). The
        // accumulator convention is slot 0 (see store_acc! macro doc).
        load_reg!(a => 10);     // ldr x10, [x20, x_a, lsl #3]
        store_acc!(10);          // str x10, [x20]
        dispatch!();
    }
}
```

- [ ] **Step 5: Check whether store-local + Ldar slow stubs are now dead**

```bash
grep -rn "op_store_local_[0-3]_slow_rs\|op_ldar_slow_rs" crates/lyng-js/
```

Delete dead-code slow stubs; document in commit message.

- [ ] **Step 6: Build + run all tests**

Run: `cargo build -p lyng-js-vm --release`
Expected: clean.

Run in parallel:
```bash
cargo test -p lyng-js-vm --lib --release
cargo test -p lyng-js-tests --release
```
Expected: 418+ vm, 1198+ + 8 (Task 2) + 3 (Ldar) = 1209+ tests.

- [ ] **Step 7: Commit**

```bash
git add crates/lyng-js/vm/src/dsl/handlers/cold.rs crates/lyng-js-tests/tests/op_ldar_inline.rs
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.3 Task 3: op_store_local_0/1/2/3 + op_ldar inline ports

5 inline ports replacing call_slow! shims:

- op_store_local_{0,1,2,3}: load_reg!(a => x10) + store_local_fixed!(x10, N)
  StoreLocal3 is the top-30 anchor; StoreLocal0/1/2 are macro-shared
  symmetric pairs (≪ 15 min each per the umbrella rule).
- op_ldar: load_reg!(a => x10) + store_acc!(x10). Accumulator is slot 0.

All 5 are pure register-window moves with no bail conditions — slow-path
expected 0.00% on V8 v7.

Dead-code: <list> slow stubs removed; <list> retained for prefix variants.

Integration test op_ldar_inline.rs covers temporary materialization,
chained arithmetic, and function-call-result patterns.

Microbench + asm baseline + slow-path-share gates: Task 4.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Add missing snippets + capture asm baselines + microbench + per-handler ported reports

**Files:**
- Modify: `tools/lyng-js-bench/src/microbench/snippets.rs` (add 3 missing snippets)
- Create: `reports/js/lyng-js/dsl-asm-baseline-aarch64/op_{load,store}_local_{0,1,2,3}.asm` + `op_ldar.asm` (9 baselines)
- Create: `reports/js/lyng-js/dsl-handlers/op_{load,store}_local_{0,1,2,3}.md` + `op_ldar.md` (9 per-handler reports)
- Create: `reports/js/lyng-js/dsl-1/phase-1b3-microbench.md`

- [ ] **Step 1: Verify which snippets exist**

```bash
grep -n 'map.insert("' tools/lyng-js-bench/src/microbench/snippets.rs | head -25
```

Per current state (HEAD `08727f92`), these snippets exist for the in-scope opcodes:
- LoadLocal0, LoadLocal1, LoadLocal2, LoadLocal3 (Phase 1.B.0)
- StoreLocal3 (Phase 1.B.0)
- Ldar (Phase 1.B.0)

These are MISSING (need to add in this task):
- StoreLocal0
- StoreLocal1
- StoreLocal2

- [ ] **Step 2: Add the 3 missing snippets**

Open `tools/lyng-js-bench/src/microbench/snippets.rs`. Find the existing `StoreLocal3` entry (around line 374). Add 3 new entries immediately after it, modeled on its shape. Example for StoreLocal0:

```rust
// StoreLocal0: assign to first parameter inside a tight loop.
map.insert("StoreLocal0", Snippet {
    name: "StoreLocal0",
    source: r#"
        (function bench(iters) {
            var a = 0;
            for (var i = 0; i < iters; i++) {
                a = i;
            }
            return a;
        })(iters)
    "#,
    opcodes_per_iter: <count>,  // Tune empirically; verify via DUMP_SNIPPETS=1
});
```

(Same shape for StoreLocal1, StoreLocal2 — change variable names so the compiler places them in slots 1 and 2 respectively. The actual slot placement depends on the parser/compiler; verify via `DUMP_SNIPPETS=1 cargo test ... verify_opcodes_per_iter` if the choice of expression matters.)

**Important:** the parser+peephole may collapse simple `a = i` patterns. Look at the existing StoreLocal3 snippet for the exact JS source structure that successfully generates StoreLocal3 dispatches per iter. Mirror that structure for StoreLocal0/1/2.

- [ ] **Step 3: Verify `opcodes_per_iter` is correct**

Run:
```bash
cargo test -p lyng-js-bench --lib verify_opcodes_per_iter --release
```
Expected: ALL snippets pass (the existing 16 + your 3 new = 19). If the new ones fail, tune `opcodes_per_iter` based on the diagnostic output.

- [ ] **Step 4: Run microbench end-to-end**

```bash
cargo build --release -p lyng-js-bench
cargo run --release -p lyng-js-bench -- microbench --samples 7 --json /tmp/phase-1b3-microbench.json 2>&1 | tail -30
```

Capture ns/dispatch + CI95 for all 9 in-scope opcodes:
- LoadLocal0, LoadLocal1, LoadLocal2, LoadLocal3
- StoreLocal0, StoreLocal1, StoreLocal2, StoreLocal3
- Ldar

Compare against LLInt reference numbers (from `tools/lyng-js-bench/hot-opcodes.toml` if listed, else from Phase 1.B.0 microbench table). All 9 must be within 2× LLInt reference.

- [ ] **Step 5: Capture asm baselines**

The `lyng-js-bench asm-diff` tool may not yet support the `dsl::handlers::cold::*` namespace (per the Phase 1.B.2 finding). If so, fall back to manual extraction:

```bash
cargo rustc --release -p lyng-js-vm --lib -- --emit=asm 2>&1 | tail -10
```

Find the produced `.s` file under `target/release/deps/` and grep for each opcode symbol:

```bash
ls target/release/deps/lyng_js_vm-*.s | head -1  # find the file
SFILE=$(ls target/release/deps/lyng_js_vm-*.s | head -1)
# For each of the 9 opcodes:
for OP in op_load_local_0 op_load_local_1 op_load_local_2 op_load_local_3 \
          op_store_local_0 op_store_local_1 op_store_local_2 op_store_local_3 \
          op_ldar; do
    sed -n "/_${OP}_dsl:/,/\.section/p" "$SFILE" | head -30 \
      > "reports/js/lyng-js/dsl-asm-baseline-aarch64/${OP}.asm"
done
```

(Adjust the sed pattern if the symbol decoration differs in the rustc output.)

Verify each captured baseline file has the inline asm (not the cold-stub asm). Each should be ~7-10 instructions total.

- [ ] **Step 6: Write per-handler ported reports**

For each of the 9 opcodes, create `reports/js/lyng-js/dsl-handlers/op_<name>.md`. Use `reports/js/lyng-js/dsl-handlers/op_load_const8.md` (from Phase 1.B.2) as the template.

Each report contains:
- Opcode byte + layout + length
- Inline asm sequence (annotated; quote the 7-line asm from the baseline)
- Slow-path conditions (none for these — pure register moves)
- Microbench result (ns/dispatch + CI95 from Step 4)
- LLInt reference comparison (within 2×? Yes/No)
- Slow-path-share on V8 v7 (Step 7 below)
- Asm-baseline reference (link to the `.asm` file from Step 5)

- [ ] **Step 7: Capture slow-path-share on V8 v7**

```bash
cargo run --release -p lyng-js-bench -- v8suite --samples 3 --count-slow-path-share --json /tmp/phase-1b3-slowshare.json 2>&1 | tail -40
```

(Or whatever the actual flag is; Phase 1.B.2 used this successfully.)

Extract per-opcode slow-path-share for all 9. Expected: 0.000% for each. Gate: < 20%.

Add the slow-path-share numbers to each per-handler ported report (Step 6).

- [ ] **Step 8: Write phase-1b3-microbench.md**

Create `reports/js/lyng-js/dsl-1/phase-1b3-microbench.md` mirroring `phase-1b2-microbench.md`:

```markdown
# Phase 1.B.3 — Microbench + slow-path-share results

Measured 2026-05-20 after the 9 inline ports landed (HEAD `<sha>`).

## Microbench (post-port ns/dispatch)

| Opcode      | Post-port (ns) | CI95 | LLInt ref | Within 2×? |
|-------------|---------------:|-----:|----------:|------------|
| LoadLocal0  | <n>            | ±<n> | <n>       | ✅         |
| LoadLocal1  | <n>            | ±<n> | <n>       | ✅         |
| LoadLocal2  | <n>            | ±<n> | <n>       | ✅         |
| LoadLocal3  | <n>            | ±<n> | <n>       | ✅         |
| StoreLocal0 | <n>            | ±<n> | <n>       | ✅         |
| StoreLocal1 | <n>            | ±<n> | <n>       | ✅         |
| StoreLocal2 | <n>            | ±<n> | <n>       | ✅         |
| StoreLocal3 | <n>            | ±<n> | <n>       | ✅         |
| Ldar        | <n>            | ±<n> | <n>       | ✅         |

## Slow-path-share on V8 v7

| Opcode      | Dispatches | Slow-path | Share |
|-------------|-----------:|----------:|------:|
| op_load_local_0 | <n>    | 0         | 0.000% |
| ... (all 9)     |         |           |        |

All 9 within < 20% gate (expected 0.000% across all due to no bail conditions).

## Verdict

All per-opcode gates green.
```

- [ ] **Step 9: Verify behavioral parity unchanged**

Run in parallel:
```bash
cargo test -p lyng-js-vm --lib --release
cargo test -p lyng-js-tests --release
```
Expected: 418+ / 1209+.

- [ ] **Step 10: Commit**

```bash
git add tools/lyng-js-bench/src/microbench/snippets.rs reports/js/lyng-js/dsl-asm-baseline-aarch64/ reports/js/lyng-js/dsl-handlers/op_load_local_*.md reports/js/lyng-js/dsl-handlers/op_store_local_*.md reports/js/lyng-js/dsl-handlers/op_ldar.md reports/js/lyng-js/dsl-1/phase-1b3-microbench.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.3 Task 4: per-opcode gates + microbench + slow-path-share

- Added 3 missing microbench snippets (StoreLocal0/1/2) to close the
  snippets-coverage gap; total 19 snippets verified via
  verify_opcodes_per_iter.
- Captured asm baselines for all 9 inline-ported opcodes; each baseline
  contains the expected 2-instruction body + decode + dispatch tail.
- Microbench (7-sample medians): all 9 within 2× LLInt reference.
  Headroom varies; documented per opcode in
  reports/js/lyng-js/dsl-handlers/op_*.md.
- Slow-path-share on V8 v7: 0.000% for all 9 opcodes (no bail
  conditions; combined ~<n>M dispatches per V8 v7 run, all inline).
- Per-handler ported reports complete with asm + microbench +
  slow-path-share data.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Same-load A/B + cumulative A/B + Test262 + sub-phase summary + reviewer dispatch

**Files:**
- Create: `reports/js/lyng-js/dsl-1/phase-1b3-ab-comparison.md`
- Create: `reports/js/lyng-js/dsl-1/phase-1b3-cumulative-ab.md`
- Create: `reports/js/lyng-js/dsl-1/phase-1b3-summary.md`
- Modify: `reports/js/lyng-js/dsl-1/phase-1b-followups.md` (record op_load_env_slot deferral)

- [ ] **Step 1: Same-load A/B vs immediate predecessor `08727f92`**

Mirror Phase 1.B.2 cleanup A/B protocol (the one that used 11 samples; HEAD `78e25a6b` is the precedent):

```bash
uptime  # capture loadavg
git stash --include-untracked
git checkout 08727f92
cargo build --release -p lyng-js-bench
cargo run --release -p lyng-js-bench -- v8suite --samples 11 --json /tmp/phase-1b3-ab-base.json 2>&1 | tail -20
uptime
git restore reports/js/lyng-js/bench-v8.md 2>/dev/null  # if modified by bench
git checkout claude/epic-saha-8f0b96
git stash pop 2>/dev/null  # if anything was stashed
cargo build --release -p lyng-js-bench
cargo run --release -p lyng-js-bench -- v8suite --samples 11 --json /tmp/phase-1b3-ab-post.json 2>&1 | tail -20
uptime
```

**Loadavg overlap MUST be within ±20% at the changeover** (post-audit hard rule). If exceeds 20%, abort and re-run in a quieter window. NO ROUNDING.

Compute per-workload deltas + geomean. Target: aggregate ≤ 2% regression, ≤ 5% per-workload.

Write `reports/js/lyng-js/dsl-1/phase-1b3-ab-comparison.md` mirroring `phase-1b2-ab-comparison.md` (the cleanup re-run version, with 11 samples).

- [ ] **Step 2: Cumulative A/B vs pre-DSL-0 `d850f261`**

This is the umbrella §1 criterion 5 gate. Repeat the A/B protocol with base = `d850f261`:

```bash
git stash --include-untracked
git checkout d850f261
cargo build --release -p lyng-js-bench
cargo run --release -p lyng-js-bench -- v8suite --samples 11 --json /tmp/phase-1b3-cumulative-base.json 2>&1 | tail -20
uptime
git restore reports/js/lyng-js/bench-v8.md 2>/dev/null
git checkout claude/epic-saha-8f0b96
git stash pop 2>/dev/null
cargo build --release -p lyng-js-bench
cargo run --release -p lyng-js-bench -- v8suite --samples 11 --json /tmp/phase-1b3-cumulative-post.json 2>&1 | tail -20
uptime
```

**Target: aggregate ≥ +3% geomean.** This is the meaningful cumulative gate that the entire Phase 1.B was working toward.

Write `reports/js/lyng-js/dsl-1/phase-1b3-cumulative-ab.md`:

```markdown
# Phase 1.B.3 — Cumulative A/B vs pre-DSL-0 HEAD `d850f261`

Measured 2026-05-20 at Phase 1.B.3 phase-close HEAD.

## Methodology

Umbrella §1 criterion 5 gate: V8 v7 cumulative ≥ +3% vs pre-DSL-0 HEAD `d850f261`.

11-sample medians, same-load A/B protocol per umbrella §4. Loadavg overlap measured at changeover: <N>% — <within / outside> ±20% protocol.

## V8 v7 results

| Workload    | `d850f261` median | Phase 1.B.3 close median | Cumulative delta |
|-------------|------------------:|-------------------------:|-----------------:|
| Richards    | <n>               | <n>                      | <+n>%            |
| DeltaBlue   | <n>               | <n>                      | <+n>%            |
| Crypto      | <n>               | <n>                      | <+n>%            |
| RayTrace    | <n>               | <n>                      | <+n>%            |
| NavierStokes| <n>               | <n>                      | <+n>%            |
| Splay       | <n>               | <n>                      | <+n>%            |
| **Geomean** | **<n>**           | **<n>**                  | **<+n>%**        |

## Verdict

- Umbrella §1 criterion 5 target: ≥ +3% geomean.
- Observed: <n>% geomean.
- Result: **<PASS / FAIL>**.

This is the definitive umbrella gate measurement at the cumulative level; supersedes any composed-prediction value in `phase-1b-summary.md`.
```

- [ ] **Step 3: Capture Test262 pass count at HEAD**

```bash
cargo run --release -p lyng-js-test262 -- 2>&1 | tail -10
```

(Or whatever the actual command is; see `phase-1b-test262-baseline.md` for the precedent from cleanup batch 2.)

Expected: ≥ 49729 passing (the umbrella baseline). If less, investigate before declaring sub-phase failure.

Note the pass count for the sub-phase summary.

- [ ] **Step 4: Record op_load_env_slot deferral formally in followups doc**

Open `reports/js/lyng-js/dsl-1/phase-1b-followups.md`. Add a new entry for the LoadEnvSlot deferral with details:

```markdown
### op_load_env_slot — deferred to a substrate sub-phase

**Date deferred:** 2026-05-20 (Phase 1.B.3 brainstorming)
**Reason:** The semantic body in `vm/semantics/scope.rs:81-110` requires:
- Reading `frame.lexical_env()` (not mirrored on LlIntState)
- A variable-depth `environment_at_depth` walk
- A loop-iteration-env linear scan even at depth 0
- A slot read that can yield (`handle_dispatch_result`)

This is substrate work (Phase-1.B.1-style refactor with a new `frame_lexical_env`
mirror on `LlIntState`), not a mechanical port.

**Recommendation:** dedicated substrate sub-phase (proposed Phase 1.B.4 or
Phase 1.C.0) co-designed with op_store_env_slot and any other env-related work.
The umbrella §1 criterion 1 floor of "9 opcodes ported" is still met by
Phase 1.B.3 (6 anchors + 3 pairs); LoadEnvSlot's deferral changes the *mix*
of ports, not the count.

**Estimated effort:** 3-4 days (mirror the Phase 1.B.1 frame-context refactor
structure).
```

- [ ] **Step 5: Mandatory feature-dev:code-reviewer dispatch**

Dispatch a reviewer over the full sub-phase commit range. The brief MUST include:

> Review the Phase 1.B.3 commit range (`08727f92..HEAD`). Spec at `docs/superpowers/specs/2026-05-20-dsl-1-phase-1b3-locals-and-ldar-design.md`. Plan at `docs/superpowers/plans/2026-05-20-dsl-1-phase-1b3-locals-and-ldar.md`.
>
> **High-priority verification items:**
>
> 1. **Runtime-dispatch coverage of new backend macros.** The Phase 1.B.1 retrospective lesson (documented in `phase-1b1-summary.md`) is that structural-only validation tests missed the x22→x24 register-pin bug. Verify that the new `load_local_fixed!` and `store_local_fixed!` macros from Task 1 are exercised by REAL handlers in Task 2 + Task 3, not just by structural symbol-existence tests. Specifically check that the integration tests in `op_locals_inline.rs` and `op_ldar_inline.rs` actually run the inlined paths (not just the cold stubs).
>
> 2. **Asm correctness.** For each of the 9 ported handlers, verify the captured asm baseline at `reports/js/lyng-js/dsl-asm-baseline-aarch64/op_*.asm` matches the design's "2 body + 1 decode + 4 dispatch = 7 instr" target. Spot-check for any unexpected register pins (e.g., x22 used where x20 was intended — the analog to the 1.B.1 bug).
>
> 3. **Per-opcode gates.** All 9 must satisfy: ≤ 12 inline instr, microbench within 2× LLInt, slow-path-share < 20% on V8 v7, behavioral parity. Cross-check the microbench numbers from `phase-1b3-microbench.md` against the per-handler ported reports.
>
> 4. **Cumulative A/B vs `d850f261` ≥ +3%.** This is the meaningful umbrella gate. Verify the measurement methodology (11 samples, ±20% loadavg overlap), and verdict.
>
> 5. **Dead-code cleanup correctness.** If slow-path stubs were deleted, verify no remaining caller references them. If retained, verify the rationale (e.g., used by wide/extra-wide prefix variants).
>
> Report confidence-filtered findings only. Verdict: APPROVED / APPROVED WITH CONCERNS / CHANGES REQUESTED.

Address any high-severity findings before sub-phase close. Append reviewer sign-off section to a reviewer-specific file or to the sub-phase summary.

- [ ] **Step 6: Write Phase 1.B.3 sub-phase summary**

Create `reports/js/lyng-js/dsl-1/phase-1b3-summary.md` mirroring `phase-1b2-summary.md` (post-cleanup version). Include:
- Range, baseline-vs-HEAD SHAs
- Status: closed
- Scope landed (5 tasks → commits; 9 ports detailed)
- Test results (vm 418+ / tests 1209+)
- Same-load A/B vs `08727f92` (link to phase-1b3-ab-comparison.md)
- Cumulative A/B vs `d850f261` (link to phase-1b3-cumulative-ab.md) — this is the headline result
- Microbench + slow-path-share summary (link to phase-1b3-microbench.md)
- Per-handler ported reports (links to 9 dsl-handlers/op_*.md files)
- Test262 confirmation: ≥ 49729 passing
- Reviewer dispatch outcome
- Lessons / observations (especially: 1.B.3 wall-clock vs umbrella estimate; structural-test runtime-dispatch coverage worked correctly per the retrospective lesson)
- Phase 1.B.3 exit criteria assessment table
- Decision: closed; next step Phase 1.C (or LoadEnvSlot substrate sub-phase, if scoped first)
- Commits list

- [ ] **Step 7: Commit Task 5 + Push final HEAD**

```bash
git add reports/js/lyng-js/dsl-1/phase-1b3-ab-comparison.md reports/js/lyng-js/dsl-1/phase-1b3-cumulative-ab.md reports/js/lyng-js/dsl-1/phase-1b3-summary.md reports/js/lyng-js/dsl-1/phase-1b-followups.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.3 Task 5: same-load A/B + cumulative A/B + Test262 + reviewer + sub-phase summary

- Same-load A/B vs `08727f92`: aggregate <delta>% geomean, range
  <min>%-<max>%. PASS (≤ 2% regression gate).
- Cumulative A/B vs pre-DSL-0 `d850f261`: aggregate <delta>% geomean.
  Umbrella §1 criterion 5 target ≥ +3%: <PASS / FAIL>.
- Test262 at HEAD: <n> passing (vs 49729 mid-phase baseline). PASS.
- Reviewer dispatch: <APPROVED / WITH CONCERNS> — <n high>/<n medium>/<n low>
  findings; major findings <resolved / deferred>.
- op_load_env_slot deferral formally recorded in phase-1b-followups.md.
- Sub-phase summary at phase-1b3-summary.md.

Phase 1.B.3 closed. Phase 1.B status: 4 of 4 sub-phases done (plus
LoadEnvSlot deferred to a substrate sub-phase).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Self-review of this plan

**Spec coverage check:**

| Spec section | Task(s) implementing it |
|--------------|--------------------------|
| §1 Goal: 9 inline ports | Tasks 2 + 3 |
| §1 Goal: cumulative ≥ +3% vs `d850f261` | Task 5 Step 2 |
| §2 Substrate inventory + load_local_fixed!/store_local_fixed! macros | Task 1 |
| §3.1-3.6 per-opcode designs | Tasks 2 + 3 |
| §4 Sub-phase phasing (5 tasks) | Plan structure |
| §5.1 Backend macro structural tests | Task 1 Step 5 |
| §5.2 Per-opcode integration tests | Tasks 2 + 3 |
| §5.3 Per-opcode microbench | Task 4 |
| §5.4 Slow-path-share | Task 4 Step 7 |
| §5.5 Behavioral parity at every commit | Every task's verification |
| §5.6 Test262 ≥ 49729 | Task 5 Step 3 |
| §5.7 Mandatory reviewer with explicit runtime-dispatch coverage check | Task 5 Step 5 |
| §6 Hard ±20% loadavg + 11+ samples + direct cumulative A/B | Task 5 Steps 1-2 |
| §7 Risks | Mitigated via TDD discipline (Tasks 2-3 write tests first) and explicit reviewer brief items |
| §8 Decisions (LoadEnvSlot deferral, scope, macros, reviewer brief) | Reflected in plan structure + Task 5 Step 4 |

No gaps.

**Placeholder scan:** the `<n>` and `<sha>` markers in Task 4-5 templates are explicit fill-in points for the implementer (not vague "TBD"). Each step shows concrete code or commands; no "implement details" anti-patterns.

**Type consistency:** macro names `load_local_fixed!` / `store_local_fixed!` consistent (Task 1 definition; Tasks 2-3 usage). Opcode names consistent (`op_load_local_N_dsl` style throughout). Helper name `run_script_returning_value` consistent across Tasks 2 + 3 tests (real name discovered in Task 2 Step 1).
