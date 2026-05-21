# DSL-1 Phase 1.B.2 — Backfill ports — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Inline-port `op_load_const8` (opcode 140, #21 in top-30) and `op_load_this` (opcode 28, #12 in top-30) using the frame-context substrate landed in Phase 1.B.1.

**Architecture:** Both opcodes currently use `call_slow!` shims. Phase 1.B.2 replaces those with inline asm handlers that read the new `LlIntState` fields (`frame_const_base`, `frame_this_value`). `op_load_const8` is a straight ~4-instruction inline read. `op_load_this` is ~9 instructions including a sentinel-bail to the existing slow path on `Value::uninitialized_lexical()`. One tiny new backend macro (`load_uninit_lex_sentinel!`) for sentinel materialization.

**Tech Stack:** Rust + AArch64 `naked_asm!`, `#[repr(C)]`, cargo workspace.

**Spec:** [`docs/superpowers/specs/2026-05-19-dsl-1-phase-1b2-backfill-ports-design.md`](../specs/2026-05-19-dsl-1-phase-1b2-backfill-ports-design.md).
**Baseline HEAD:** `68dd5e89` (Phase 1.B.1 closed).

---

## File structure overview

### Created
- `reports/js/lyng-js/dsl-handlers/op_load_const8.md` — per-handler ported report
- `reports/js/lyng-js/dsl-handlers/op_load_this.md` — per-handler ported report
- `reports/js/lyng-js/dsl-1/phase-1b2-ab-comparison.md` — same-load V8 v7 A/B
- `reports/js/lyng-js/dsl-1/phase-1b2-microbench.md` — microbench results vs LLInt reference + slow-path-share gate
- `reports/js/lyng-js/dsl-1/phase-1b2-summary.md` — sub-phase summary

### Modified
- `crates/lyng-js/vm/src/dsl/backend/aarch64/prelude.rs` — add `VALUE_UNINIT_LEX_BITS` const
- `crates/lyng-js-vm-dsl/src/lower.rs` — add `value_uninit_lex_bits` universal binding (mirroring `state_this_value` precedent from 1.B.1)
- `crates/lyng-js/vm/src/dsl/backend/aarch64/values.rs` (or `frame.rs` if more natural) — add `load_uninit_lex_sentinel!` macro
- `crates/lyng-js/vm/src/dsl/handlers/cold.rs:879-902` — replace `op_load_this_dsl` cold stub with inline port (keep `op_load_this_slow_rs` as bail target)
- `crates/lyng-js/vm/src/dsl/handlers/cold.rs:4184-4210` — replace `op_load_const8_dsl` cold stub with inline port (delete `op_load_const8_slow_rs` if no longer reachable)
- `crates/lyng-js/vm/tests/dsl_validation_frame_context.rs` — delete the 3 `#[ignore]`-d forward-pointer tests; keep the 3 structural compiles-and-links tests
- New integration tests in `crates/lyng-js-tests/` per the per-opcode-gate convention

---

## Conventions for this plan

- **User deny rules:** NEVER use `git -C <path>` or `cd <path> && git ...`. Always run git from the worktree's working directory (you're already there).
- **Commits:** Each task ends with a self-contained commit. Use the HEREDOC format with `Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>` footer.
- **`reports/js/lyng-js/bench-v8.md`** is a bench-tool side-effect; leave unstaged throughout.
- **Untracked planning docs** (`docs/superpowers/{plans,specs}/*.md`): leave untouched.
- **Behavioral parity at every commit:** `cargo test -p lyng-js-vm --lib --release` (≥417) AND `cargo test -p lyng-js-tests --release` (≥1187). 2 pre-existing `feedback_flat_consistency` failures stay unrelated.
- **TDD discipline** for the macro (Task 1) and per-opcode integration tests (Tasks 2 + 3). The asm-handler bodies themselves are exercised via the integration tests + microbench + V8 v7 A/B.

---

## Task 1: Add `load_uninit_lex_sentinel!` backend macro

**Files:**
- Modify: `crates/lyng-js/vm/src/dsl/backend/aarch64/prelude.rs` (add `VALUE_UNINIT_LEX_BITS` const)
- Modify: `crates/lyng-js-vm-dsl/src/lower.rs` (add `value_uninit_lex_bits` universal binding)
- Modify: `crates/lyng-js/vm/src/dsl/backend/aarch64/values.rs` (add macro)

- [ ] **Step 1: Add the sentinel-bits const to prelude.rs**

In `crates/lyng-js/vm/src/dsl/backend/aarch64/prelude.rs`, find the section with other Value-related constants (look for `VALUE_TAG_HEADER`, `VALUE_PAYLOAD_MASK`, etc. — around lines 40-50). Add after the existing kind constants:

```rust
/// 64-bit bit pattern of `Value::uninitialized_lexical()`. Used by
/// the `load_uninit_lex_sentinel!` backend macro to materialize the
/// sentinel for sentinel-bail comparisons in `op_load_this` and
/// any future opcode that needs to compare against this sentinel.
pub const VALUE_UNINIT_LEX_BITS: u64 = Value::uninitialized_lexical().bits();
```

Note: `Value::uninitialized_lexical()` is a `const fn` (per `crates/lyng-js/types/src/value.rs:186-188`), so this works at const-eval time. If `.bits()` is not const, expose it via a helper `pub const fn bits_const(self) -> u64` on `Value` in `types/src/value.rs` and use that. The refactor worker investigates at impl time.

- [ ] **Step 2: Add a unit test for the const**

At the bottom of `prelude.rs`, in a `#[cfg(test)] mod tests`, add:

```rust
#[test]
fn value_uninit_lex_bits_matches_runtime() {
    assert_eq!(VALUE_UNINIT_LEX_BITS, Value::uninitialized_lexical().bits());
    // Sanity: the sentinel must be distinguishable from common Values.
    assert_ne!(VALUE_UNINIT_LEX_BITS, Value::undefined().bits());
    assert_ne!(VALUE_UNINIT_LEX_BITS, Value::from_smi(0).bits());
}
```

If the `tests` mod doesn't already exist in prelude.rs, add it. If `Value::bits()` is not public, replace it with whatever public accessor exposes the u64 (`.to_bits()`, `.raw()`, etc.) — the refactor worker discovers via `grep "pub fn.*Value.*u64" crates/lyng-js/types/src/value.rs`.

- [ ] **Step 3: Run the unit test to verify it passes**

Run: `cargo test -p lyng-js-vm --lib value_uninit_lex_bits_matches_runtime`
Expected: PASS (or "no tests" if rust-analyzer is stale — run `cargo test -p lyng-js-vm --lib --release 2>&1 | tail -10` to verify).

- [ ] **Step 4: Add `value_uninit_lex_bits` to the lowerer's universal binding set**

Open `crates/lyng-js-vm-dsl/src/lower.rs`. Find the section that injects universal `naked_asm!` named bindings (look for `state_this_value`, `state_pb`, `state_fv`, `state_regs`, `state_prefix` — added during Phase 1.B.1 Task 5). Add an analogous binding:

```rust
// Phase 1.B.2: sentinel bit pattern for op_load_this's bail comparison.
// Mirrors the state_this_value pattern.
value_uninit_lex_bits = const ::lyng_js_vm::dsl::backend::aarch64::prelude::VALUE_UNINIT_LEX_BITS,
```

(Adjust the exact syntax to match the existing binding-list style.)

- [ ] **Step 5: Build to verify it compiles**

Run: `cargo build -p lyng-js-vm --release`
Expected: clean. The new const is exposed; the new binding is wired through the lowerer.

- [ ] **Step 6: Add the `load_uninit_lex_sentinel!` macro**

In `crates/lyng-js/vm/src/dsl/backend/aarch64/values.rs` (this is where Value-related backend macros live; if a different file is more conventional for sentinel/constant-materialization, the refactor worker picks based on the existing module organization — `prelude.rs` is also reasonable), add:

```rust
/// Materialize the `Value::uninitialized_lexical()` 64-bit sentinel
/// into the destination register. Used by `op_load_this` to compare
/// against the pre-resolved `frame_this_value` mirror; on match,
/// the handler bails to the slow path which resolves the actual
/// ThisState (Uninitialized → throw ReferenceError; Lexical → walk
/// lex-env).
///
/// Tries `ldr {dst}, =literal` first (assembler-managed literal
/// pool, 1 instruction). If `naked_asm!` rejects that form, falls
/// back to a `movz` + 3× `movk` sequence (4 instructions).
///
/// ## Emitted shape (literal-pool form, 1 instruction)
///
/// ```text
///     ldr x{dst}, ={value_uninit_lex_bits}
/// ```
///
/// ## Argument conventions
///
/// - `$dst_reg` is the scratch register number (the lowerer's
///   `t0..t6` slots have already been substituted by macro
///   expansion time).
/// - `value_uninit_lex_bits` is a `naked_asm!`-supplied named
///   binding (added to the lowerer's universal binding set in
///   Task 1 Step 4).
///
/// See spec §3.2.
#[macro_export]
macro_rules! load_uninit_lex_sentinel {
    ($dst_reg:tt) => {
        concat!(
            "ldr    x", stringify!($dst_reg), ", ={value_uninit_lex_bits}\n",
        )
    };
}
```

**Note on the macro shape:** mirror the existing `load_state_value!` pattern from `frame.rs`. The exact syntax — whether it's `macro_rules! load_uninit_lex_sentinel!` with `#[macro_export]` or some other variant — should match the existing convention in `aarch64/` modules. If the `ldr =literal` form is rejected at compile time (Step 7), update the macro to emit the movz/movk fallback:

```rust
#[macro_export]
macro_rules! load_uninit_lex_sentinel {
    ($dst_reg:tt) => {
        concat!(
            "movz   x", stringify!($dst_reg), ", #({value_uninit_lex_bits} & 0xffff)\n",
            "movk   x", stringify!($dst_reg), ", #(({value_uninit_lex_bits} >> 16) & 0xffff), lsl #16\n",
            "movk   x", stringify!($dst_reg), ", #(({value_uninit_lex_bits} >> 32) & 0xffff), lsl #32\n",
            "movk   x", stringify!($dst_reg), ", #(({value_uninit_lex_bits} >> 48) & 0xffff), lsl #48\n",
        )
    };
}
```

The refactor worker writes one form, tries to compile a test handler in Step 7, and switches to the other form if needed.

- [ ] **Step 7: Add a structural validation test in `dsl_validation_frame_context.rs`**

Open `crates/lyng-js/vm/tests/dsl_validation_frame_context.rs`. Alongside the existing three structural "compiles-and-links" handlers, add a fourth:

```rust
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_test_load_uninit_lex_sentinel_dsl,
    opcode_byte = 213, layout = Abx, length = 4, |a, _bx| {
        // Materialize the sentinel into x10, then store it to the
        // destination register. Slot 213 is unused (extends the
        // 210/211/212 range used by Phase 1.B.1 synthetic tests).
        load_uninit_lex_sentinel!(10);
        store_reg!(a, 10);
        dispatch!();
    }
}

#[cfg(target_arch = "aarch64")]
#[test]
fn load_uninit_lex_sentinel_handler_compiles_and_links() {
    let ptr = op_test_load_uninit_lex_sentinel_dsl as *const ();
    assert!(!ptr.is_null());
}
```

This catches macro-emit and lowerer-binding issues without requiring runtime execution.

- [ ] **Step 8: Run the test to verify it compiles + symbol exists**

Run: `cargo test -p lyng-js-vm --test dsl_validation_frame_context --release`
Expected: 4 tests passing (3 existing + 1 new), 3 ignored (the existing forward-pointer tests; deleted in Task 4). If the macro doesn't compile, switch to the movz/movk form (Step 6 alternate) and retry.

- [ ] **Step 9: Run the full vm + tests suites for parity**

Run in parallel: `cargo test -p lyng-js-vm --lib --release` and `cargo test -p lyng-js-tests --release`
Expected: 417+ vm, 1187+ tests (unchanged from Phase 1.B.1 close + 1 from the const unit test = 418 vm).

- [ ] **Step 10: Commit**

```bash
git add crates/lyng-js/vm/src/dsl/backend/aarch64/prelude.rs crates/lyng-js/vm/src/dsl/backend/aarch64/values.rs crates/lyng-js-vm-dsl/src/lower.rs crates/lyng-js/vm/tests/dsl_validation_frame_context.rs
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.2 Task 1: load_uninit_lex_sentinel! backend macro

Adds a 1-instruction (literal-pool `ldr =literal`) backend macro for
materializing the `Value::uninitialized_lexical()` sentinel into a
register. Used by Task 3's op_load_this inline port to compare against
the frame_this_value mirror and bail to the slow path on match.

The const `VALUE_UNINIT_LEX_BITS` is exposed in
`aarch64/prelude.rs`; the lowerer adds it to the universal binding
set as `value_uninit_lex_bits`, mirroring Phase 1.B.1's
`state_this_value` pattern.

If the literal-pool form is rejected by the rustc inline-asm parser,
the macro falls back to a 4-instruction movz/movk sequence
(documented in macro source).

Substrate-only commit; no opcode handler uses the macro yet
(Task 3 wires it). A structural compiles-and-links test in
`dsl_validation_frame_context.rs` exercises the macro end-to-end
through the lowerer + `naked_asm!`.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Inline-port `op_load_const8`

**Files:**
- Modify: `crates/lyng-js/vm/src/dsl/handlers/cold.rs:4179-4188` (handler) and `:4190-4210` (slow_rs)
- Create: `reports/js/lyng-js/dsl-handlers/op_load_const8.md`
- Test: extend an existing integration test file in `crates/lyng-js-tests/tests/` or add a new one

- [ ] **Step 1: Add integration tests for the inline port (TDD, written before the port)**

Find the existing integration test file that covers numeric literal evaluation. If unclear, run:
```bash
grep -rln "fn.*const8\|evaluate_script.*42\|LoadConst8" crates/lyng-js-tests/ | head -5
```

If a single canonical "load_const8_basics" test file doesn't exist, create `crates/lyng-js-tests/tests/op_load_const8_inline.rs`:

```rust
//! Phase 1.B.2 Task 2: integration tests for the inline op_load_const8 port.
//!
//! Exercises each ConstantValue variant that the pre-resolution
//! pipeline produces in the active code's flat constants array.

use lyng_js_tests::run_script_returning_value;  // or whatever the helper is

#[test]
fn op_load_const8_smi_constant() {
    let value = run_script_returning_value("42").expect("script ran");
    assert_eq!(value.to_number(), 42.0);
}

#[test]
fn op_load_const8_float_constant() {
    let value = run_script_returning_value("3.14").expect("script ran");
    assert!((value.to_number() - 3.14).abs() < 1e-10);
}

#[test]
fn op_load_const8_atom_constant() {
    let value = run_script_returning_value("'hello'").expect("script ran");
    assert_eq!(value.as_string(), Some("hello"));
}

#[test]
fn op_load_const8_handles_multiple_constants_in_pool() {
    // Verifies indexing is correct (not just always index 0).
    let value = run_script_returning_value("var a = 1; var b = 2; var c = 3; c").expect("script ran");
    assert_eq!(value.to_number(), 3.0);
}
```

**Note on helpers:** the exact name for `run_script_returning_value` is whatever the existing test crate uses. Search via:
```bash
grep -rn "fn.*-> Value\|pub fn.*Script\|evaluate_script\|run_script" crates/lyng-js-tests/src/ | head -10
```
Match the convention used by other tests in the same directory.

- [ ] **Step 2: Run the tests — they should pass even before the port (cold stub still works)**

Run: `cargo test -p lyng-js-tests --test op_load_const8_inline --release`
Expected: 4 passing (or however many tests you wrote). They pass because the COLD STUB also produces the correct values — Task 2 inline port doesn't change semantics, just speed.

This is the value: when Task 2 lands the inline port, the same tests must continue to pass. They guard against semantic regression.

- [ ] **Step 3: Replace the `op_load_const8_dsl` cold stub with the inline port**

In `crates/lyng-js/vm/src/dsl/handlers/cold.rs`, find the existing cold stub (around lines 4182-4188):

```rust
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_load_const8_dsl, opcode_byte = 140, layout = Ab, length = 3, |a, b| {
        call_slow!(op_load_const8_slow_rs, args = [a, b]);
        dispatch_after_slow!();
    }
}
```

Replace with the inline port:

```rust
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_load_const8_dsl, opcode_byte = 140, layout = Ab, length = 3, |a, b| {
        // Phase 1.B.2: inline read from the pre-resolved constants
        // array (RuntimeCodeRecord::constants arena slot) via the
        // frame_const_base mirror on LlIntState. `b` is the constant
        // pool index; `a` is the destination register.
        //
        // Substrate established by Phase 1.B.1:
        // - frame_const_base populated at trampoline entry
        // - refreshed in slow_path::translate_outcome's Refresh arm
        // - GC-safe per the mirror discipline invariant.
        //
        // Slow path retained as `op_load_const8_slow_rs` purely as
        // a reference / fallback for any future case the inline path
        // can't handle. Expected slow-path-share on V8 v7: ~0%.
        load_constant!(b => 10);     // 2 instr: ldr base + ldr value[b]
        store_reg!(a, 10);           // 1 instr: str
        dispatch!();                 // 4 instr: dispatch tail
    }
}
```

**Note on macro syntax:** the exact `load_constant!` signature is `($idx_reg:tt => $dst_reg:tt, vm_const_base = $offset_expr:tt)` (with the lowerer auto-injecting `vm_const_base`). If the call-site syntax differs from the simple `load_constant!(b => 10)` form because the lowerer requires explicit binding, match what the existing `load_state_value!` invocations look like — the refactor worker uses an existing call-site as a template. Phase 1.B.1 Task 5 commit `3d2bfccc` is the reference.

- [ ] **Step 4: Build to verify the inline port compiles**

Run: `cargo build -p lyng-js-vm --release`
Expected: clean. If `load_constant!` rejects the call syntax, adjust to match the actual macro signature (check `constants.rs` for the definition).

- [ ] **Step 5: Run the integration tests — they must still pass**

Run: `cargo test -p lyng-js-tests --test op_load_const8_inline --release`
Expected: same 4+ passing as in Step 2. Inline port produces the same values.

- [ ] **Step 6: Run full vm + tests suites for parity**

Run in parallel: `cargo test -p lyng-js-vm --lib --release` and `cargo test -p lyng-js-tests --release`
Expected: 417+/1187+ (parity maintained, plus the new tests from Step 1 added the per-opcode coverage).

- [ ] **Step 7: Verify the slow stub `op_load_const8_slow_rs` is still referenced (or remove it)**

The inline port doesn't `call_slow!` anymore. If `op_load_const8_slow_rs` has no callers, it's dead code. Check:
```bash
grep -rn "op_load_const8_slow_rs" crates/lyng-js/
```
If no callers remain outside the function definition itself, delete the function. If it's still referenced (e.g., the prefix-wide variant uses it), keep it.

- [ ] **Step 8: Capture asm baseline**

Run: `cargo run --release -p lyng-js-bench -- asm-diff --opcode op_load_const8_dsl`

Or whatever the equivalent command is — discover via `cargo run --release -p lyng-js-bench -- --help` and match the Phase 1.A precedent (look at Phase 1.A commits for the exact incantation).

Save the asm baseline to wherever the convention dictates (likely `reports/js/lyng-js/asm-baselines/op_load_const8.txt` or similar). Phase 1.A ported handlers (e.g., `op_load_smi8`) have asm baselines; mirror that pattern.

- [ ] **Step 9: Write the per-handler ported report**

Create `reports/js/lyng-js/dsl-handlers/op_load_const8.md`. Mirror an existing report — `reports/js/lyng-js/dsl-handlers/op_load_false.md` or `op_load_null.md` are good templates (Phase 1.A handlers).

Required sections:
- Opcode byte + layout + length
- Inline asm sequence (annotated)
- Slow-path conditions (none for this opcode)
- Microbench result (ns/dispatch + CI95)
- Slow-path-share on V8 v7
- Asm-baseline reference

Some sections (microbench, slow-path-share) are filled in during Task 4 once the bench runs. Mark those `TBD-Task-4` for now and update in Task 4.

- [ ] **Step 10: Commit**

```bash
git add crates/lyng-js/vm/src/dsl/handlers/cold.rs reports/js/lyng-js/dsl-handlers/op_load_const8.md crates/lyng-js-tests/tests/op_load_const8_inline.rs
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.2 Task 2: op_load_const8 inline port

Replaces the call_slow! shim with a 4-instruction inline read via
load_constant!(b => 10); store_reg!(a, 10); dispatch!().

The inline path handles all ConstantValue variants (Smi, Float64,
Atom, Builtin) because the pre-resolution pipeline (Vm::install_constants)
materializes them into a flat Value array at install time. The
flat array's data pointer is exposed via LlIntState::frame_const_base
(Phase 1.B.1 substrate). Slow path retained as fallback.

Asm baseline: ~7 instructions inline + 4 dispatch tail = 11 total
(well within the ≤12 budget).

Integration tests cover Smi/Float/Atom constants and multi-constant
pool indexing.

Microbench + slow-path-share gates verified in Task 4.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Inline-port `op_load_this`

**Files:**
- Modify: `crates/lyng-js/vm/src/dsl/handlers/cold.rs:874-882` (handler) — keep `op_load_this_slow_rs` as bail target
- Create: `reports/js/lyng-js/dsl-handlers/op_load_this.md`
- Test: `crates/lyng-js-tests/tests/op_load_this_inline.rs` (new)

- [ ] **Step 1: Add integration tests for the inline port + sentinel bail**

Create `crates/lyng-js-tests/tests/op_load_this_inline.rs`:

```rust
//! Phase 1.B.2 Task 3: integration tests for the inline op_load_this port.
//!
//! Exercises each ThisState arm:
//! - Value(v): inline fast path returns v
//! - Uninitialized: bail to slow path → ReferenceError throw
//! - Lexical: bail to slow path → lex-env walk
//!
//! Plus the no-EC fallback (function called without an explicit
//! execution context push — relies on FrameRecord::this_value).

use lyng_js_tests::run_script_returning_value;  // match helper from Task 2

#[test]
fn op_load_this_value_state_returns_real_this() {
    let value = run_script_returning_value(
        "(function() { return this.x; }).call({x: 42})"
    ).expect("script ran");
    assert_eq!(value.to_number(), 42.0);
}

#[test]
fn op_load_this_arrow_function_captures_lexical_this() {
    // Arrow function captures `this` from the outer scope.
    // ThisState::Lexical → bail to slow path → walk lex-env.
    let value = run_script_returning_value(
        "(function() { return (() => this.y)(); }).call({y: 7})"
    ).expect("script ran");
    assert_eq!(value.to_number(), 7.0);
}

#[test]
fn op_load_this_uninitialized_in_derived_constructor_throws() {
    // Derived constructor before super(): ThisState::Uninitialized.
    // Reading `this` must throw ReferenceError.
    let value = run_script_returning_value(r#"
        class B { constructor() { return { ok: true }; } }
        class D extends B {
            constructor() {
                try { var t = this; return 'didnt throw'; }
                catch (e) { return e.name; }
                finally { super(); /* satisfy strict mode */ }
            }
        }
        new D().toString();
    "#).expect("script ran");
    assert_eq!(value.as_string(), Some("ReferenceError"));
}

#[test]
fn op_load_this_in_top_level_script_is_undefined_or_global() {
    // Depending on strict mode + script vs module, this varies.
    // Just assert it doesn't crash and returns *something*.
    let _value = run_script_returning_value("this").expect("script ran");
    // No specific value assertion — semantics depend on spec details.
}
```

The exact JS syntax for triggering each ThisState arm may need adjustment based on what the lyng-js parser supports (e.g., class syntax availability). The refactor worker adjusts to match. If `class` syntax isn't fully supported, find alternative ways to construct each state (e.g., `Object.create` + manual binding rituals for Uninitialized).

- [ ] **Step 2: Run the tests — they should pass with the cold stub**

Run: `cargo test -p lyng-js-tests --test op_load_this_inline --release`
Expected: all passing (cold stub correctly produces these values). If any test fails before the inline port, that's an unrelated bug — investigate before proceeding.

- [ ] **Step 3: Replace the `op_load_this_dsl` cold stub with the inline port**

In `crates/lyng-js/vm/src/dsl/handlers/cold.rs`, find the existing cold stub (around lines 877-882):

```rust
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_load_this_dsl, opcode_byte = 28, layout = Abx, length = 4, |a, bx| {
        call_slow!(op_load_this_slow_rs, args = [a, bx]);
        dispatch_after_slow!();
    }
}
```

Replace with the inline + sentinel bail port:

```rust
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_load_this_dsl, opcode_byte = 28, layout = Abx, length = 4, |a, _bx| {
        // Phase 1.B.2: inline read of the pre-resolved `this` mirror
        // via LlIntState::frame_this_value. The mirror holds either
        // the real `this` value (ThisState::Value(v)) or the sentinel
        // `Value::uninitialized_lexical()` (ThisState::Uninitialized
        // or ThisState::Lexical) — see `resolve_initial_this_value`
        // in `crate::dsl::llint_state`.
        //
        // The inline fast path:
        // - reads frame_this_value into a scratch register (1 instr)
        // - materializes the sentinel into a second scratch (1 instr)
        // - compares + bails to slow path on equality (2 instr)
        // - stores to dest reg (1 instr)
        // - dispatch tail (4 instr)
        //
        // Total: ~9 instr inline (within ≤12 budget). Slow path is
        // op_load_this_slow_rs which delegates to the semantic body
        // that throws (Uninitialized) or walks lex-env (Lexical).
        load_state_value!(10, vm_state_offset = state_this_value);     // 1 instr: ldr from frame_this_value
        load_uninit_lex_sentinel!(11);                                  // 1 instr: ldr sentinel literal
        // Inline cmp + b.eq to slow path. `bx` is unused but kept in
        // the args list so the slow-path bridge has consistent ABI.
        cmp_and_bail_eq!(10, 11, op_load_this_slow_rs, args = [a, _bx]);  // 2 instr: cmp + b.eq
        store_reg!(a, 10);                                              // 1 instr: str
        dispatch!();                                                    // 4 instr: dispatch tail
    }
}
```

**On `cmp_and_bail_eq!`:** this macro may not exist yet. If not, write the cmp + b.eq inline. The mechanism:
- aarch64 `cmp x10, x11` compares the registers
- `b.eq L_slow` branches to a label that calls `op_load_this_slow_rs` and dispatches after slow

Look at existing DSL handlers that have similar conditional-bail patterns. For example, type-check macros likely have `b.ne fast_path; b.eq slow_path; ...` patterns; mirror that.

If no precedent exists, the cleanest approach is to introduce a minimal `cmp_and_bail_eq!($reg_a, $reg_b, $slow_fn, args = [$($a:tt)*])` macro in `crates/lyng-js/vm/src/dsl/backend/aarch64/control.rs` (where dispatch and branch macros live). The refactor worker either finds a precedent and uses it directly, or adds the macro as a tiny helper here.

- [ ] **Step 4: Build to verify the inline port compiles**

Run: `cargo build -p lyng-js-vm --release`
Expected: clean. If the macro signatures don't match, adjust per existing precedent.

- [ ] **Step 5: Run the op_load_this integration tests — they must still pass**

Run: `cargo test -p lyng-js-tests --test op_load_this_inline --release`
Expected: all passing. The Value(v) tests exercise the inline fast path; the Uninitialized + Lexical tests exercise the sentinel bail to slow path.

- [ ] **Step 6: Run full vm + tests suites for parity**

Run in parallel: `cargo test -p lyng-js-vm --lib --release` and `cargo test -p lyng-js-tests --release`
Expected: 417+/1187+ (plus the new tests from Step 1).

- [ ] **Step 7: Capture asm baseline + write per-handler ported report**

Mirror Task 2 Steps 8 + 9. Asm baseline → `reports/js/lyng-js/asm-baselines/op_load_this.txt` (or equivalent location). Ported report → `reports/js/lyng-js/dsl-handlers/op_load_this.md`. Mark microbench + slow-path-share `TBD-Task-4`.

- [ ] **Step 8: Commit**

```bash
git add crates/lyng-js/vm/src/dsl/handlers/cold.rs crates/lyng-js-tests/tests/op_load_this_inline.rs reports/js/lyng-js/dsl-handlers/op_load_this.md reports/js/lyng-js/asm-baselines/op_load_this.txt
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.2 Task 3: op_load_this inline port with sentinel bail

Replaces the call_slow! shim with a ~9-instruction inline read of
the frame_this_value mirror + sentinel-bail to the slow path.

The inline fast path:
- reads LlIntState::frame_this_value via load_state_value!
- materializes Value::uninitialized_lexical() via the Task-1 macro
- inline cmp + b.eq bails to op_load_this_slow_rs on sentinel match
- stores the real `this` to dest reg + dispatches

Slow path retained (op_load_this_slow_rs) as the bail target for
ThisState::Uninitialized (throws ReferenceError) and ThisState::Lexical
(walks lex-env). The mirror was populated at trampoline entry by
resolve_initial_this_value (Phase 1.B.1 helper) and is refreshed on
every Refresh egress.

Integration tests cover all three ThisState arms (Value, Uninitialized,
Lexical) plus the no-EC fallback.

Asm baseline: ~9 instr inline + 4 dispatch tail = 13 total. The
sentinel materialization currently uses the literal-pool ldr form
(1 instr); fall-back movz/movk would be 4 instr (total 16, still
within absolute budget for handlers with bail paths).

Microbench + slow-path-share gates verified in Task 4.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Cleanup, V8 v7 A/B, microbench, slow-path-share

**Files:**
- Modify: `crates/lyng-js/vm/tests/dsl_validation_frame_context.rs` (delete the 3 ignored forward-pointer tests)
- Modify: `reports/js/lyng-js/dsl-handlers/op_load_const8.md` (fill in TBDs from Task 2)
- Modify: `reports/js/lyng-js/dsl-handlers/op_load_this.md` (fill in TBDs from Task 3)
- Create: `reports/js/lyng-js/dsl-1/phase-1b2-microbench.md`
- Create: `reports/js/lyng-js/dsl-1/phase-1b2-ab-comparison.md`

- [ ] **Step 1: Delete the 3 ignored forward-pointer tests in dsl_validation_frame_context.rs**

Open `crates/lyng-js/vm/tests/dsl_validation_frame_context.rs`. Find the three tests marked `#[ignore]` (added in Phase 1.B.1 Task 6, commit `0605a407`):
- `load_constant_reads_pre_resolved_constants_array`
- `load_this_value_reads_real_this_binding`
- `load_this_value_reads_sentinel_for_uninitialized`

Delete them entirely. Keep the 3 "compiles-and-links" structural tests (`load_constant_handler_compiles_and_links`, `load_this_value_handler_compiles_and_links`, `load_this_sentinel_handler_compiles_and_links`) and the Task-1 addition (`load_uninit_lex_sentinel_handler_compiles_and_links`).

Add a note at the top of the file explaining the cleanup:

```rust
//! Phase 1.B.2 cleanup: the 3 forward-pointer #[ignore]-d tests that
//! were placeholders for Phase 1.B.2's canonical opcodes are removed.
//! End-to-end coverage of op_load_const8 and op_load_this now lives in
//! `crates/lyng-js-tests/tests/op_load_const8_inline.rs` and
//! `crates/lyng-js-tests/tests/op_load_this_inline.rs` respectively.
//!
//! The 4 structural compiles-and-links tests remain — they catch
//! macro-emit and lowerer-binding regressions.
```

- [ ] **Step 2: Run tests to confirm the cleanup compiles + all suites pass**

Run: `cargo test -p lyng-js-vm --test dsl_validation_frame_context --release`
Expected: 4 passing (the 4 structural tests), 0 ignored.

Run: `cargo test -p lyng-js-vm --lib --release` and `cargo test -p lyng-js-tests --release`
Expected: 417+/1187+ (parity).

- [ ] **Step 3: Run microbench**

```bash
cargo build --release -p lyng-js-bench
cargo run --release -p lyng-js-bench -- microbench --samples 7 --json /tmp/phase-1b2-microbench.json 2>&1 | tail -30
```

Verify the `LoadConst8` and `LoadThis` snippets report ns/dispatch with CI95. Compare against the LLInt reference (Phase 1.B.0 microbench tables for those snippets, or `tools/lyng-js-bench/hot-opcodes.toml` if a reference is listed there).

- [ ] **Step 4: Run V8 v7 same-load A/B vs `68dd5e89` (Phase 1.B.1 closed HEAD)**

Follow the protocol from Phase 1.B.1 Task 8 (which is the established pattern):

```bash
git stash --include-untracked
uptime
git checkout 68dd5e89
cargo build --release -p lyng-js-bench
cargo run --release -p lyng-js-bench -- v8suite --samples 7 --json /tmp/phase-1b2-base.json 2>&1 | tail -20
uptime
git restore reports/js/lyng-js/bench-v8.md
git checkout claude/epic-saha-8f0b96
git stash pop
cargo build --release -p lyng-js-bench
cargo run --release -p lyng-js-bench -- v8suite --samples 7 --json /tmp/phase-1b2-post.json 2>&1 | tail -20
uptime
```

Compute per-workload deltas and geomean (mirror Phase 1.B.1's `phase-1b1-ab-comparison.md` format).

- [ ] **Step 5: Run slow-path-share measurement on V8 v7**

```bash
cargo run --release -p lyng-js-bench -- v8suite --samples 3 --count-slow-path-share --json /tmp/phase-1b2-slowshare.json 2>&1 | tail -30
```

Or whatever the actual command-line flag is — discover via `--help` if the syntax above is wrong. The infra was added in Phase 1.B.0 Task 5.

Verify per-opcode slow-path-share for `op_load_const8` (expected ≈ 0%) and `op_load_this` (expected < 5%).

- [ ] **Step 6: Write the microbench summary**

Create `reports/js/lyng-js/dsl-1/phase-1b2-microbench.md`:

```markdown
# Phase 1.B.2 — Microbench + slow-path-share results

Measured 2026-05-19 after `op_load_const8` and `op_load_this` inline ports landed.

## Microbench (post-port ns/dispatch)

| Opcode      | ns/dispatch | CI95 | LLInt ref | Within 2×? |
|-------------|------------:|-----:|----------:|------------|
| LoadConst8  | <n>         | ±<n> | <n>       | ✅ / ❌    |
| LoadThis    | <n>         | ±<n> | <n>       | ✅ / ❌    |

(LLInt reference: from Phase 1.B.0 microbench tables or hot-opcodes.toml; see [`phase-1b0-summary.md`](phase-1b0-summary.md) microbench section.)

## Slow-path-share on V8 v7

| Opcode      | Slow-path-share | Within 20% gate? |
|-------------|----------------:|------------------|
| op_load_const8 | <n>%         | ✅ / ❌          |
| op_load_this   | <n>%         | ✅ / ❌          |

## Verdict

| Gate | op_load_const8 | op_load_this |
|------|----------------|--------------|
| ≤12 inline instr | ✅ <n> | ✅ <n> |
| Microbench within 2× LLInt | ✅ / ❌ | ✅ / ❌ |
| Slow-path-share < 20% on V8 v7 | ✅ / ❌ | ✅ / ❌ |
| Behavioral parity | ✅ | ✅ |
```

- [ ] **Step 7: Write the A/B comparison**

Create `reports/js/lyng-js/dsl-1/phase-1b2-ab-comparison.md` mirroring `phase-1b1-ab-comparison.md`'s structure. Include per-workload deltas, geomean, loadavg overlap, verdict against the ≤2% aggregate / ≤5% per-workload gates.

- [ ] **Step 8: Fill in the TBDs in the per-handler ported reports**

Open `reports/js/lyng-js/dsl-handlers/op_load_const8.md` and `reports/js/lyng-js/dsl-handlers/op_load_this.md`. Replace `TBD-Task-4` markers with the actual microbench + slow-path-share numbers from Steps 3 + 5.

- [ ] **Step 9: Commit**

```bash
git add crates/lyng-js/vm/tests/dsl_validation_frame_context.rs reports/js/lyng-js/dsl-1/phase-1b2-microbench.md reports/js/lyng-js/dsl-1/phase-1b2-ab-comparison.md reports/js/lyng-js/dsl-handlers/op_load_const8.md reports/js/lyng-js/dsl-handlers/op_load_this.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.2 Task 4: cleanup + microbench + V8 v7 A/B

- Deletes the 3 ignored forward-pointer tests in
  dsl_validation_frame_context.rs (forward-pointers from Phase 1.B.1
  to canonical opcodes that now exist). 4 structural compiles-and-links
  tests retained.
- Microbench: LoadConst8 + LoadThis report ns/dispatch within 2× LLInt
  reference. (See phase-1b2-microbench.md for numbers.)
- Slow-path-share: op_load_const8 ~<n>%; op_load_this <<n>% on V8 v7.
  Both well within the <20% gate.
- Same-load A/B vs `68dd5e89` (Phase 1.B.1 close): aggregate <delta>%
  geomean (PASS, within ≤2% gate; no workload regressed > 5%).
- Per-handler ported reports filled in with microbench + slow-path-share
  data.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Phase 1.B.2 sub-phase summary

**Files:**
- Create: `reports/js/lyng-js/dsl-1/phase-1b2-summary.md`

- [ ] **Step 1: Draft the summary**

Use `reports/js/lyng-js/dsl-1/phase-1b1-summary.md` as the template. Include:
- Date range, baseline-vs-HEAD SHAs
- Status: closed
- Scope landed table (5 tasks → commits)
- Test results (vm 417+ / tests 1187+ / per-opcode integration tests)
- Same-load A/B summary (link to `phase-1b2-ab-comparison.md`)
- Microbench + slow-path-share summary (link to `phase-1b2-microbench.md`)
- Per-handler ported reports (links to the two `dsl-handlers/op_load_*.md` files)
- Lessons / observations
- Phase 1.B.2 exit criteria assessment table
- Decision: closed; recommended next step is Phase 1.B.3 (locals + Ldar + LoadEnvSlot ports)
- Commits list

- [ ] **Step 2: Verify all gate criteria are met**

Cross-check against spec §1 exit criteria. Each gate must be ✅:
- Behavioral parity: 417+ / 1187+
- Per-opcode gates for each of op_load_const8 + op_load_this: ≤12 instr, microbench within 2× LLInt, slow-path-share < 20%, ported report present, asm baseline captured
- Same-load A/B: ≤ 2% aggregate regression, ≤ 5% per-workload, expected ≥ +0.3% V8 v7 improvement

If any gate is ❌, the sub-phase is NOT closed; back to the relevant task to fix.

- [ ] **Step 3: Commit the summary**

```bash
git add reports/js/lyng-js/dsl-1/phase-1b2-summary.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.B.2: phase summary — backfill ports complete

5 commits landing inline ports of op_load_const8 (#21) and
op_load_this (#12) using the Phase 1.B.1 frame-context substrate.
Together these handle ~360M combined dispatches per V8 v7 run.

All exit gates green at HEAD <SHA>:
- Behavioral parity: 417+ vm-lib, 1187+ lyng-js-tests + per-opcode
  integration tests for all ThisState arms
- Per-opcode gates: ≤12 inline instr, microbench within 2× LLInt,
  slow-path-share < 20% on V8 v7 for both opcodes
- Same-load A/B vs `68dd5e89`: <delta>% V8 v7 geomean (PASS)
- Per-handler ported reports + asm baselines complete

Phase 1.B.2 closed. Phase 1.B.3 (locals + Ldar + LoadEnvSlot ports
under strict top-30 + macro-shared-pair discipline) can proceed.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Self-review of this plan

**Spec coverage check:**

| Spec section | Task(s) implementing it |
|--------------|--------------------------|
| §3.1 op_load_const8 inline port | Task 2 |
| §3.2 op_load_this inline port + sentinel bail | Task 3 |
| §3.2 sentinel materialization macro | Task 1 |
| §3.3 cmp + b.eq inline (no new macro) | Task 3 Step 3 |
| §4 Cleanup ignored tests | Task 4 Step 1 |
| §5 Per-opcode tests | Task 2 Step 1 + Task 3 Step 1 |
| §5 Microbench | Task 4 Step 3 |
| §5 Slow-path-share | Task 4 Step 5 |
| §6.1 Backend macro task | Task 1 |
| §6.4 V8 v7 A/B | Task 4 Step 4 |
| §6.5 Sub-phase summary | Task 5 |
| §7 Risks | Mitigated via TDD discipline and inline option for sentinel materialization (literal-pool + movz/movk fallback) |
| §8 Decisions | All reflected in task structure |

No gaps.

**Placeholder scan:** The `TBD-Task-4` markers in Tasks 2 + 3 (microbench + slow-path-share sections of the ported reports) are explicit deferrals to Task 4 — Task 4 explicitly fills them in. Not a "TBD" anti-pattern. All step code is concrete. Some "if X doesn't exist, do Y" branches are paired with concrete fallbacks (e.g., literal-pool vs movz/movk for the sentinel materialization). No empty placeholders.

**Type consistency:** Macro names consistent: `load_uninit_lex_sentinel!` (Task 1, used in Task 3), `load_constant!` (Task 2, from 1.B.1 substrate), `load_state_value!` (Task 3, from 1.B.1 substrate). Const name `VALUE_UNINIT_LEX_BITS` consistent (Task 1 definition, Task 3 reference via the `value_uninit_lex_bits` lowerer binding). Function name `op_load_this_slow_rs` consistent (existing Phase 1.A precedent, retained in Task 3).
