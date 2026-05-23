# DSL-1 Phase 1.A — Trivial Loads Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port 9 trivial constant-loader opcodes from cold-stub delegation to full inline DSL fast paths, producing the first measurable V8 v7 win on the new substrate.

**Architecture:** Each opcode's `_dsl` handler in [`crates/vm/src/dsl/handlers/cold.rs`](../../../crates/vm/src/dsl/handlers/cold.rs) currently delegates 100% to a slow path. We replace each handler body with inline asm that writes the appropriate tagged `Value` to register `a` and tail-dispatches — bypassing the slow path entirely. The slow-path shim stays as the on-error fallback (where applicable — these opcodes have no fail mode, so the slow path becomes dead code and can be deleted).

**Tech Stack:** Rust 2024 edition (stable, ≥1.88 for `naked_asm!`), `lyng-vm-dsl` proc-macro (existing), AArch64 backend macros in [`crates/vm/src/dsl/backend/aarch64/`](../../../crates/vm/src/dsl/backend/aarch64/) (existing).

**Parent spec:** [`docs/superpowers/specs/2026-05-18-dsl-1-hot-opcode-rollout-design.md`](../specs/2026-05-18-dsl-1-hot-opcode-rollout-design.md) — Phase 1.A.

---

## Scope

This plan covers Phase 1.A only. Phases 1.B through 1.G will each get their own plan invoked at the phase boundary. At the end of this plan, V8 v7 should be cumulatively ≥ +5% over the pre-DSL-0 baseline, and 9 new ported reports + asm baselines should be committed.

**Opcodes in this phase (port order — simplest first):**

1. `op_load_undefined` (canonical exemplar; uses `tag_undefined!`)
2. `op_load_null`
3. `op_load_true`
4. `op_load_false`
5. `op_load_zero` (SMI constant, value 0)
6. `op_load_one` (SMI constant, value 1)
7. `op_load_smi8` (SMI with sign-extended i8 operand) — in top-30 at #7
8. `op_load_const8` (constant pool indirection) — in top-30 at #21
9. `op_load_this` (frame-context access) — in top-30 at #12

Five of these are outside the measured top-30 (load_undefined, _null, _true, _false, _one) but ship in the same phase for completeness because they share the inline-tag-write pattern. Four are in top-30 and drive Phase 1.A's measurable V8 v7 win.

**Off-ramp:** if Task 8 (`op_load_const8`) or Task 9 (`op_load_this`) surfaces a data-layout refactor we can't absorb in-task (e.g., constant-pool access or frame-context offsets that need a new DSL op), the worker **aborts and reports**. The coordinator decides whether to schedule the refactor or skip the opcode (keeping its cold stub through Phase 1.A and revisiting in 1.B).

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| [`crates/vm/src/dsl/handlers/cold.rs`](../../../crates/vm/src/dsl/handlers/cold.rs) | Modify | Replace cold-stub bodies for the 9 opcodes with inline DSL handlers. The `op_*_slow_rs` shims become dead code; delete those whose opcodes can never fail (all 6 const loaders), keep for `op_load_smi8`/`op_load_const8`/`op_load_this` only if they have a failure mode. |
| [`crates/vm/src/dsl/backend/aarch64/values.rs`](../../../crates/vm/src/dsl/backend/aarch64/values.rs) | Possibly modify | Add `tag_smi_const!($reg, $payload)` if not present (one-shot tagged-SMI for `op_load_zero`/`op_load_one`/`op_load_smi8`). |
| [`crates/vm/src/dsl/backend/aarch64/operands.rs`](../../../crates/vm/src/dsl/backend/aarch64/operands.rs) | Possibly modify | May need `decode_ab_signed!` if the lowerer doesn't sign-extend `b` for `op_load_smi8`. Inspect during Task 7. |
| [`crates/vm/src/dsl/backend/aarch64/mod.rs`](../../../crates/vm/src/dsl/backend/aarch64/mod.rs) | Possibly modify | Re-export any new macros. |
| [`crates/vm/src/dsl/ops.md`](../../../crates/vm/src/dsl/ops.md) | Modify | Document any new ops added. |
| [`crates/vm/src/dsl/handlers/cold.rs`](../../../crates/vm/src/dsl/handlers/cold.rs) imports section (top of file) | Modify | Add new macros to the import list as needed. |
| [`reports/lyng/dsl-handlers/op_load_undefined.md`](../../../reports/lyng/dsl-handlers/) (and 8 more) | Create | One ported report per opcode with DSL source, current asm, LLInt reference, side-by-side diff, microbench data. |
| [`reports/lyng/dsl-asm-baseline-aarch64/op_load_undefined.asm`](../../../reports/lyng/dsl-asm-baseline-aarch64/) (and 8 more) | Create | Captured asm baseline per opcode. |
| [`tools/lyng-bench/hot-opcodes.toml`](../../../tools/lyng-bench/hot-opcodes.toml) | Modify | Calibrate `aarch64_max_instructions` budgets for the 4 top-30 opcodes (LoadSmi8, LoadThis, LoadZero, LoadConst8) from real measurements. |
| [`reports/lyng/dsl-1/phase-1a-summary.md`](../../../reports/lyng/dsl-1/) | Create | Phase summary with aggregate V8 v7 delta + per-opcode rollup. |

---

## Task 0: Kickoff verification

**Files:**
- Read: [`reports/lyng/r0/v8-v7-top30.tsv`](../../../reports/lyng/r0/v8-v7-top30.tsv)
- Read: [`tools/lyng-bench/hot-opcodes.toml`](../../../tools/lyng-bench/hot-opcodes.toml)
- Read: [`reports/lyng/microbench-baseline.md`](../../../reports/lyng/microbench-baseline.md)
- Create: `reports/lyng/dsl-1/pre-phase-1a-baseline.md`

- [ ] **Step 1: Verify tooling availability**

```bash
cargo run --release -p lyng-bench -- --help
```

Expected: subcommands list includes `microbench`, `asm-diff`, `capture-llint`, `v8suite`. If any are missing, abort and report — R-0 tooling is expected to be complete pre-DSL-1.

- [ ] **Step 2: Capture pre-Phase-1.A V8 v7 baseline**

```bash
cargo run --release -p lyng-bench -- v8suite \
  --require-isolation \
  --samples 7 \
  --output /tmp/pre-phase-1a-v8.json
```

Expected: command exits 0; output file has 7 sample sets per workload.

If loadavg gate fires, run `uptime` and wait for loadavg < 2.0 before retrying.

- [ ] **Step 3: Capture pre-Phase-1.A microbench baseline for affected opcodes**

```bash
cargo run --release -p lyng-bench -- microbench \
  --opcodes-config tools/lyng-bench/hot-opcodes.toml \
  --opcodes LoadUndefined,LoadNull,LoadTrue,LoadFalse,LoadZero,LoadOne,LoadSmi8,LoadConst8,LoadThis \
  --samples 7 \
  --output /tmp/pre-phase-1a-microbench.json
```

Expected: 9 opcodes × 7 samples; output has ns/dispatch with confidence intervals.

- [ ] **Step 4: Capture pre-Phase-1.A slow-path-share for affected opcodes**

```bash
cargo run --release -p lyng-bench --features lyng-vm/opcode-counters -- v8suite \
  --require-isolation \
  --count-slow-path-share \
  --output /tmp/pre-phase-1a-slowshare.json
```

Expected: per-opcode counts; the 9 Phase 1.A opcodes show ~100% slow-path-share (they're cold stubs).

- [ ] **Step 5: Write the pre-phase baseline summary**

Create `reports/lyng/dsl-1/pre-phase-1a-baseline.md` with:

```markdown
# Pre-Phase-1.A baseline

Captured 2026-MM-DD on <dev machine name> with loadavg < 2.0.

## V8 v7 (geomean across workloads)

| Workload  | Score  | 95% CI |
|-----------|--------|--------|
| Richards  | <num>  | ±<num> |
| DeltaBlue | <num>  | ±<num> |
| ...       | ...    | ...    |
| **Geomean** | <num> |        |

(Fill from /tmp/pre-phase-1a-v8.json.)

## Microbench (ns/dispatch, 7-sample median)

| Opcode         | ns/dispatch | LLInt reference |
|----------------|-------------|-----------------|
| LoadUndefined  | <num>       | <num>           |
| LoadNull       | <num>       | <num>           |
| LoadTrue       | <num>       | <num>           |
| LoadFalse      | <num>       | <num>           |
| LoadZero       | <num>       | <num>           |
| LoadOne        | <num>       | <num>           |
| LoadSmi8       | <num>       | <num>           |
| LoadConst8     | <num>       | <num>           |
| LoadThis       | <num>       | <num>           |

## Slow-path-share (V8 v7)

| Opcode         | Slow-path-share |
|----------------|-----------------|
| LoadUndefined  | ~100% (cold stub) |
| ...            | ~100%             |
```

- [ ] **Step 6: Commit the baseline**

```bash
git add reports/lyng/dsl-1/pre-phase-1a-baseline.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.A: capture pre-phase baseline

Pre-port V8 v7, microbench, and slow-path-share captures for the 9
opcodes in Phase 1.A. Establishes the comparison baseline for the
phase exit gate.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

Expected: clean commit; no other files staged.

---

## Task 1: Port `op_load_undefined` (canonical exemplar — full TDD detail)

**Files:**
- Modify: `crates/vm/src/dsl/handlers/cold.rs:140-168` (replace the `op_load_undefined_dsl` handler body; delete `op_load_undefined_slow_rs`)
- Modify: `crates/vm/src/dsl/handlers/cold.rs` (top of file imports section — add `tag_undefined`, `store_reg`, `dispatch`, `decode_abx` to the use list if not already present)
- Create: `reports/lyng/dsl-asm-baseline-aarch64/op_load_undefined.asm`
- Create: `reports/lyng/dsl-handlers/op_load_undefined.md`

- [ ] **Step 1: Read JSC's reference for the equivalent opcode**

```bash
# JSC has no exact equivalent to "LoadUndefined-to-explicit-register" —
# its closest is op_mov_to_undefined or the constant-table load via
# loadConstantOrVariable. Capture both for reference.
cargo run --release -p lyng-bench -- capture-llint \
  --source auto \
  --opcodes op_mov,op_mov_to_undefined \
  --output reports/lyng/llint-reference/
```

Expected: produces or updates `reports/lyng/llint-reference/op_mov.md` (and op_mov_to_undefined.md if present in this JSC version). Note which mode succeeded (`system`/`local`/`excerpt`) — record in the ported report.

If capture-llint fails on `auto`, retry with `--source excerpt` to use the offlineasm source fallback.

- [ ] **Step 2: Read the existing cold-stub to confirm the layout**

Inspect [crates/vm/src/dsl/handlers/cold.rs:140-168](../../../crates/vm/src/dsl/handlers/cold.rs). The current handler:

```rust
llint_handler! {
    op_load_undefined_dsl, layout = Abx, length = 4, |a, bx| {
        call_slow!(op_load_undefined_slow_rs, args = [a, bx]);
        dispatch_after_slow!();
    }
}
```

`bx` operand is unused — the layout has the 16-bit slot for forward compat but the opcode doesn't read it. Confirm by reading the slow-path shim at line 155: `_unused_variables` allowed because `bx` is consumed only by the slow path's args constructor and is irrelevant to the semantic outcome.

- [ ] **Step 3: Confirm the relevant DSL macros exist**

Check [`crates/vm/src/dsl/backend/aarch64/values.rs`](../../../crates/vm/src/dsl/backend/aarch64/values.rs) for `tag_undefined!`. It exists at line 203-210:

```rust
macro_rules! tag_undefined {
    ($reg:tt) => {
        concat!(
            "movz   x", stringify!($reg), ", #0x1, lsl #32\n",
            "movk   x", stringify!($reg), ", #0x7ff8, lsl #48\n",
        )
    };
}
```

That's 2 instructions to materialize `Value::undefined()` in a scratch register.

- [ ] **Step 4: Run the existing behavioral test slice to confirm starting state is green**

```bash
cargo test -p lyng-vm --lib --release -- --nocapture loads
```

Expected: all `loads`-prefixed tests pass. This is the regression net we must preserve.

- [ ] **Step 5: Write the new inline DSL handler**

In [`crates/vm/src/dsl/handlers/cold.rs`](../../../crates/vm/src/dsl/handlers/cold.rs), replace lines 140-168 (the `op_load_undefined_dsl` handler block AND the `op_load_undefined_slow_rs` shim — the latter becomes dead code) with:

```rust
// =====================================================================
// LoadUndefined — inline DSL fast path (DSL-1 Phase 1.A, Task 1).
//
// Writes Value::undefined() to register `a`. The `bx` operand is unused
// (layout reserves a 16-bit slot for forward compat that this opcode
// doesn't consume). No fail mode → no slow path.
// =====================================================================

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_load_undefined_dsl, layout = Abx, length = 4, |a, _bx| {
        tag_undefined!(t0);
        store_reg!(a, t0);
        dispatch!();
    }
}
```

If the file's top-of-file `use` block doesn't already import `tag_undefined`, add it:

```rust
#[cfg(target_arch = "aarch64")]
use crate::{
    call_slow, decode_ab, decode_abx, dispatch, dispatch_after_slow,
    load_reg, store_reg, tag_undefined,
};
```

(Adjust to match the current imports — preserve the rest of the list.)

- [ ] **Step 6: Build to verify compile**

```bash
cargo build --release -p lyng-vm
```

Expected: clean build. If `tag_undefined` is undefined, the macro isn't exported / imported — fix the `use` list. If the proc-macro lowerer rejects `_bx`, switch to `bx` and accept a dead-decoded-operand warning (the lowerer's substitute_idents pass tolerates this — the decode prologue emits the `ldrh` and the unused register slot stays live until clobbered).

- [ ] **Step 7: Run behavioral tests**

```bash
cargo test -p lyng-vm --lib --release
cargo test -p lyng-tests --release
```

Expected: all green. The new handler's semantics must match the deleted slow-path shim's semantics (write `Value::undefined()` to register `a`).

If any test fails, the inline DSL has wrong tag bits or wrong register index. Restore the cold-stub and investigate the `tag_undefined!` macro's tag bits vs the R-0 value-layout report.

- [ ] **Step 8: Capture the asm baseline**

```bash
cargo run --release -p lyng-bench -- asm-diff \
  --opcodes-config tools/lyng-bench/hot-opcodes.toml \
  --baseline reports/lyng/dsl-asm-baseline-aarch64/ \
  --output /tmp/asm-current/ \
  --mode update \
  --opcodes LoadUndefined
```

Expected: produces `reports/lyng/dsl-asm-baseline-aarch64/op_load_undefined.asm` with the new inline asm. Inspect manually: should look like:

```text
op_load_undefined_dsl:
    ldrb    w9, [x19, #1]           ; decode a
    ldrh    w10, [x19, #2]          ; decode bx (unused)
    movz    x11, #0x1, lsl #32      ; tag_undefined! t0 (low)
    movk    x11, #0x7ff8, lsl #48   ; tag_undefined! t0 (high)
    str     x11, [x20, x9, lsl #3]  ; store_reg!(a, t0)
    add     x19, x19, #4            ; dispatch! advance
    ldrb    w8, [x19]               ; load next opcode
    ldr     x12, [x23, x8, lsl #3]  ; load next handler
    br      x12                     ; tail-jump
```

9 instructions. Within the design's expected budget. The unused `bx` decode (line 2) is dead code that LLVM may or may not eliminate; if asm-diff shows it, accept it for now and note in the ported report.

- [ ] **Step 9: Run microbench**

```bash
cargo run --release -p lyng-bench -- microbench \
  --opcodes LoadUndefined \
  --samples 7 \
  --require-isolation \
  --output /tmp/post-task1-microbench.json
```

Expected: ns/dispatch for LoadUndefined is significantly lower than pre-phase baseline (cold stub goes through `call_slow!` + Rust + `dispatch_after_slow!`; inline DSL is ~9 instructions). Target: within 2× of LLInt's `op_mov_to_undefined` (or equivalent).

If the result isn't substantially better than pre-phase, the cold-stub may not have been the bottleneck OR the asm baseline contains unexpected spills. Inspect.

- [ ] **Step 10: Run V8 v7 with slow-path-share count**

```bash
cargo run --release -p lyng-bench --features lyng-vm/opcode-counters -- v8suite \
  --require-isolation \
  --count-slow-path-share \
  --opcodes LoadUndefined \
  --output /tmp/post-task1-slowshare.json
```

Expected: LoadUndefined slow-path-share drops to ~0% (the slow path is gone; every dispatch goes through inline asm). If non-zero, the dispatch table still points at the old symbol — verify the manifest is consistent.

- [ ] **Step 11: Write the ported report**

Create `reports/lyng/dsl-handlers/op_load_undefined.md`:

```markdown
# `op_load_undefined` DSL port (Phase 1.A, Task 1)

First Phase-1.A port — establishes the canonical pattern for constant-loader
opcodes. The handler writes `Value::undefined()` to register `a`; the `bx`
operand is unused (layout reserves the slot for forward compat).

## DSL source

`crates/vm/src/dsl/handlers/cold.rs`:

```rust
llint_handler! {
    op_load_undefined_dsl, layout = Abx, length = 4, |a, _bx| {
        tag_undefined!(t0);
        store_reg!(a, t0);
        dispatch!();
    }
}
```

## Current asm (AArch64)

See `reports/lyng/dsl-asm-baseline-aarch64/op_load_undefined.asm`.

Effective sequence:

```text
ldrb   w9, [x19, #1]            ; decode a
ldrh   w10, [x19, #2]           ; decode bx (unused — see note)
movz   x11, #0x1, lsl #32       ; tag_undefined!: low half
movk   x11, #0x7ff8, lsl #48    ; tag_undefined!: high half
str    x11, [x20, x9, lsl #3]   ; store_reg!(a, t0)
add    x19, x19, #4             ; advance PC
ldrb   w8,  [x19]               ; next opcode
ldr    x12, [x23, x8, lsl #3]   ; next handler
br     x12                      ; tail-jump
```

**9 instructions** (8 if LLVM elides the dead `ldrh` of `bx`). 5 are dispatch tail (advance + load + branch); 4 are the actual work (decode + tag + store).

## LLInt reference

JSC has no direct `op_load_undefined`. The closest is `op_mov` to a register containing `jsUndefined()`, which JSC's bytecode compiler emits as a single `op_mov` from a constant-table slot holding `undefined`. LLInt's `loadConstantOrVariable` macro materializes the value with one cmov-style branch:

```text
get(m_src, t1)
loadConstantOrVariable(size, t1, t2)   ; ~3 instrs incl. const-table check
return(t2)                             ; store + dispatch (~5 instrs)
```

Total ~8 instructions. Lyng's port is essentially the same shape, with `tag_undefined!` substituting for the constant-table fetch. **Within 1 instruction of LLInt.**

## Side-by-side diff

| Step           | Lyng DSL                          | LLInt                                      |
|----------------|-----------------------------------|--------------------------------------------|
| Decode dst     | `ldrb w9, [x19, #1]`              | inline in `get(m_dst, ...)`                |
| Decode src     | `ldrh w10, [x19, #2]` (unused)    | inline in `get(m_src, t1)`                 |
| Read source    | (none — constant inlined)         | `loadConstantOrVariable(size, t1, t2)`     |
| Materialize    | `movz/movk` (tag_undefined!)      | (part of loadConstantOrVariable)           |
| Write dest     | `str x11, [x20, x9, lsl #3]`      | `return(t2)` (also dispatches)             |
| Dispatch       | 4-instr tail                      | embedded in `return`                       |

## Microbench

| Variant           | ns/dispatch (7-sample median) | 95% CI |
|-------------------|------------------------------:|-------:|
| Pre-port (cold)   | <fill from pre-phase data>    |        |
| Post-port (DSL)   | <fill from /tmp/post-task1>   |        |
| LLInt (reference) | <fill from R-0 LLInt micro>   |        |

Lyng DSL is **<N>× of LLInt** post-port (target: within 2×).

## Slow-path-share

Pre-port: ~100% (cold stub).
Post-port: 0% (inline; no slow path).

## Behavioral tests

- `cargo test -p lyng-vm --lib --release` passes (full suite).
- `cargo test -p lyng-tests --release` passes.
- Test262 sweep deferred to phase gate (Task 10).

## Notes

- The `bx` operand (16-bit unused) ideally elides — LLVM may keep the
  `ldrh` if it can't prove the load is dead. Acceptable: the load
  hits the bytecode cache line that the next dispatch's `ldrb` would
  hit anyway.
- The slow-path shim `op_load_undefined_slow_rs` was deleted alongside
  this port. No remaining caller.
```

Fill in the `<fill from...>` placeholders with real numbers from the captured JSON output. Real data — do not commit with `<fill>` placeholders remaining.

- [ ] **Step 12: Commit the port**

```bash
git add \
  crates/vm/src/dsl/handlers/cold.rs \
  reports/lyng/dsl-asm-baseline-aarch64/op_load_undefined.asm \
  reports/lyng/dsl-handlers/op_load_undefined.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.A Task 1: op_load_undefined inline DSL port

Replace cold-stub delegation with inline asm: tag_undefined! + store_reg
+ dispatch. Slow-path shim deleted (no fail mode). 9 instructions
total; within 1 of LLInt's op_mov-via-constant-table.

Ported report and asm baseline committed.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

Expected: clean commit. `git status` shows clean.

---

## Task 2: Port `op_load_null`

**Files:**
- Modify: `crates/vm/src/dsl/handlers/cold.rs:200-228` (replace `op_load_null_dsl` body; delete `op_load_null_slow_rs`)
- Create: `reports/lyng/dsl-asm-baseline-aarch64/op_load_null.asm`
- Create: `reports/lyng/dsl-handlers/op_load_null.md`

- [ ] **Step 1: Confirm `tag_null!` macro exists**

Inspect [`crates/vm/src/dsl/backend/aarch64/values.rs`](../../../crates/vm/src/dsl/backend/aarch64/values.rs) — `tag_null!` is at lines 212-220, producing `movz/movk` against tag pattern 0x7ff8_0002_0000_0000. 2 instructions.

- [ ] **Step 2: Run behavioral tests to confirm starting green**

```bash
cargo test -p lyng-vm --lib --release -- loads
```

Expected: pass.

- [ ] **Step 3: Replace the handler body**

In `crates/vm/src/dsl/handlers/cold.rs:200-228`, replace with:

```rust
// =====================================================================
// LoadNull — inline DSL fast path (DSL-1 Phase 1.A, Task 2).
// =====================================================================

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_load_null_dsl, layout = Abx, length = 4, |a, _bx| {
        tag_null!(t0);
        store_reg!(a, t0);
        dispatch!();
    }
}
```

Add `tag_null` to the file's `use` import list if not already present.

- [ ] **Step 4: Build**

```bash
cargo build --release -p lyng-vm
```

Expected: clean build.

- [ ] **Step 5: Run behavioral tests**

```bash
cargo test -p lyng-vm --lib --release
cargo test -p lyng-tests --release
```

Expected: all green.

- [ ] **Step 6: Capture asm baseline**

```bash
cargo run --release -p lyng-bench -- asm-diff \
  --baseline reports/lyng/dsl-asm-baseline-aarch64/ \
  --output /tmp/asm-current/ \
  --mode update \
  --opcodes LoadNull
```

Expected: 9-instruction sequence identical to LoadUndefined except the `movz` immediate is `#0x2` (null) instead of `#0x1` (undefined).

- [ ] **Step 7: Run microbench**

```bash
cargo run --release -p lyng-bench -- microbench \
  --opcodes LoadNull \
  --samples 7 \
  --require-isolation \
  --output /tmp/post-task2-microbench.json
```

Expected: ns/dispatch nearly identical to LoadUndefined.

- [ ] **Step 8: Write ported report**

Create `reports/lyng/dsl-handlers/op_load_null.md` with the same structure as Task 1's report, swapping `undefined` for `null` and updating the tag immediate (`#0x2` for null vs `#0x1` for undefined). Fill measurement placeholders from the actual captured data.

- [ ] **Step 9: Commit**

```bash
git add \
  crates/vm/src/dsl/handlers/cold.rs \
  reports/lyng/dsl-asm-baseline-aarch64/op_load_null.asm \
  reports/lyng/dsl-handlers/op_load_null.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.A Task 2: op_load_null inline DSL port

Mirrors Task 1's pattern: tag_null! + store_reg + dispatch.
9 instructions; identical shape to op_load_undefined with #0x2 tag.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Port `op_load_true`

**Files:**
- Modify: `crates/vm/src/dsl/handlers/cold.rs:230-258` (replace `op_load_true_dsl` body; delete `op_load_true_slow_rs`)
- Create: `reports/lyng/dsl-asm-baseline-aarch64/op_load_true.asm`
- Create: `reports/lyng/dsl-handlers/op_load_true.md`

- [ ] **Step 1: Confirm `tag_bool_const!` macro exists**

Inspect [`crates/vm/src/dsl/backend/aarch64/values.rs:222-231`](../../../crates/vm/src/dsl/backend/aarch64/values.rs): `tag_bool_const!` takes `($reg:tt, $payload:literal)` and produces 3 instructions:

```rust
macro_rules! tag_bool_const {
    ($reg:tt, $payload:literal) => {
        concat!(
            "movz   x", stringify!($reg), ", #", stringify!($payload), "\n",
            "movk   x", stringify!($reg), ", #0x3, lsl #32\n",
            "movk   x", stringify!($reg), ", #0x7ff8, lsl #48\n",
        )
    };
}
```

That's 3 instructions to materialize a tagged Boolean.

- [ ] **Step 2: Run behavioral tests**

```bash
cargo test -p lyng-vm --lib --release -- loads
```

Expected: pass.

- [ ] **Step 3: Replace the handler body**

In `crates/vm/src/dsl/handlers/cold.rs:230-258`, replace with:

```rust
// =====================================================================
// LoadTrue — inline DSL fast path (DSL-1 Phase 1.A, Task 3).
// =====================================================================

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_load_true_dsl, layout = Abx, length = 4, |a, _bx| {
        tag_bool_const!(t0, 1);
        store_reg!(a, t0);
        dispatch!();
    }
}
```

Add `tag_bool_const` to the imports.

- [ ] **Step 4: Build**

```bash
cargo build --release -p lyng-vm
```

Expected: clean build.

- [ ] **Step 5: Run behavioral tests**

```bash
cargo test -p lyng-vm --lib --release
cargo test -p lyng-tests --release
```

Expected: all green.

- [ ] **Step 6: Capture asm baseline**

```bash
cargo run --release -p lyng-bench -- asm-diff \
  --baseline reports/lyng/dsl-asm-baseline-aarch64/ \
  --output /tmp/asm-current/ \
  --mode update \
  --opcodes LoadTrue
```

Expected: 10-instruction sequence (`tag_bool_const!` is 3 instructions, one more than `tag_undefined!`). One more than LoadUndefined.

- [ ] **Step 7: Run microbench**

```bash
cargo run --release -p lyng-bench -- microbench \
  --opcodes LoadTrue \
  --samples 7 \
  --require-isolation \
  --output /tmp/post-task3-microbench.json
```

Expected: ns/dispatch ~similar to LoadUndefined; one extra cycle for the third movk should be invisible.

- [ ] **Step 8: Write ported report**

Create `reports/lyng/dsl-handlers/op_load_true.md` mirroring Task 1's structure. Update tag pattern: 3 `movz/movk` (payload=1, kind=3, header=0x7ff8). Note the extra instruction compared to undefined/null in the side-by-side diff (acknowledge it's an unavoidable consequence of the Bool tag layout: payload + kind + header, where undefined/null have payload always 0 so only kind+header needed).

- [ ] **Step 9: Commit**

```bash
git add \
  crates/vm/src/dsl/handlers/cold.rs \
  reports/lyng/dsl-asm-baseline-aarch64/op_load_true.asm \
  reports/lyng/dsl-handlers/op_load_true.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.A Task 3: op_load_true inline DSL port

tag_bool_const!(t0, 1) + store_reg + dispatch. 10 instructions
(one more than undefined/null because Bool tag carries a payload bit).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Port `op_load_false`

**Files:**
- Modify: `crates/vm/src/dsl/handlers/cold.rs:260-288` (replace `op_load_false_dsl` body; delete `op_load_false_slow_rs`)
- Create: `reports/lyng/dsl-asm-baseline-aarch64/op_load_false.asm`
- Create: `reports/lyng/dsl-handlers/op_load_false.md`

- [ ] **Step 1: Run behavioral tests**

```bash
cargo test -p lyng-vm --lib --release -- loads
```

Expected: pass.

- [ ] **Step 2: Replace the handler body**

In `crates/vm/src/dsl/handlers/cold.rs:260-288`, replace with:

```rust
// =====================================================================
// LoadFalse — inline DSL fast path (DSL-1 Phase 1.A, Task 4).
// =====================================================================

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_load_false_dsl, layout = Abx, length = 4, |a, _bx| {
        tag_bool_const!(t0, 0);
        store_reg!(a, t0);
        dispatch!();
    }
}
```

`tag_bool_const` already imported from Task 3.

- [ ] **Step 3: Build**

```bash
cargo build --release -p lyng-vm
```

Expected: clean build.

- [ ] **Step 4: Run behavioral tests**

```bash
cargo test -p lyng-vm --lib --release
cargo test -p lyng-tests --release
```

Expected: all green.

- [ ] **Step 5: Capture asm baseline**

```bash
cargo run --release -p lyng-bench -- asm-diff \
  --baseline reports/lyng/dsl-asm-baseline-aarch64/ \
  --output /tmp/asm-current/ \
  --mode update \
  --opcodes LoadFalse
```

Expected: 10-instruction sequence identical to LoadTrue except payload = 0 instead of 1.

- [ ] **Step 6: Run microbench**

```bash
cargo run --release -p lyng-bench -- microbench \
  --opcodes LoadFalse \
  --samples 7 \
  --require-isolation \
  --output /tmp/post-task4-microbench.json
```

Expected: identical to LoadTrue within noise.

- [ ] **Step 7: Write ported report**

Create `reports/lyng/dsl-handlers/op_load_false.md` mirroring Task 3's report, swapping payload=1 for payload=0.

- [ ] **Step 8: Commit**

```bash
git add \
  crates/vm/src/dsl/handlers/cold.rs \
  reports/lyng/dsl-asm-baseline-aarch64/op_load_false.asm \
  reports/lyng/dsl-handlers/op_load_false.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.A Task 4: op_load_false inline DSL port

tag_bool_const!(t0, 0) + store_reg + dispatch.
Identical shape to op_load_true with payload=0.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Port `op_load_zero` (SMI constant)

**Files:**
- Modify: `crates/vm/src/dsl/handlers/cold.rs:290-318` (replace `op_load_zero_dsl` body; delete `op_load_zero_slow_rs`)
- Possibly modify: `crates/vm/src/dsl/backend/aarch64/values.rs` (add `tag_smi_const!` if not present)
- Create: `reports/lyng/dsl-asm-baseline-aarch64/op_load_zero.asm`
- Create: `reports/lyng/dsl-handlers/op_load_zero.md`

- [ ] **Step 1: Check whether `tag_smi_const!` exists**

```bash
grep -n "tag_smi_const" crates/vm/src/dsl/backend/aarch64/values.rs
```

If found, skip Step 2. If not, proceed to Step 2.

- [ ] **Step 2: Add `tag_smi_const!` macro (if missing)**

In [`crates/vm/src/dsl/backend/aarch64/values.rs`](../../../crates/vm/src/dsl/backend/aarch64/values.rs), after `tag_bool_const!` (around line 231), insert:

```rust
/// Tag a compile-time SMI literal payload into `$reg`. Produces a
/// fully tagged `Value` carrying the SMI variant + the literal payload.
/// 3 instructions: movz payload, movk kind, movk header.
#[macro_export]
macro_rules! tag_smi_const {
    ($reg:tt, $payload:literal) => {
        concat!(
            "movz   x", stringify!($reg), ", #", stringify!($payload), "\n",
            "movk   x", stringify!($reg), ", #0x4, lsl #32\n",
            "movk   x", stringify!($reg), ", #0x7ff8, lsl #48\n",
        )
    };
}
```

Also re-export from [`crates/vm/src/dsl/backend/aarch64/mod.rs`](../../../crates/vm/src/dsl/backend/aarch64/mod.rs) if it has a per-macro re-export pattern; otherwise the `#[macro_export]` is sufficient.

Add an entry to [`crates/vm/src/dsl/ops.md`](../../../crates/vm/src/dsl/ops.md) under the value-tag section:

```markdown
- `tag_smi_const!($reg, $payload)` — materialize a tagged SMI carrying a compile-time literal payload. 3 instructions. Used by `op_load_zero`, `op_load_one`, and similar constant-loaders.
```

- [ ] **Step 3: Run behavioral tests**

```bash
cargo test -p lyng-vm --lib --release -- loads
```

Expected: pass.

- [ ] **Step 4: Replace the handler body**

In `crates/vm/src/dsl/handlers/cold.rs:290-318`, replace with:

```rust
// =====================================================================
// LoadZero — inline DSL fast path (DSL-1 Phase 1.A, Task 5).
// =====================================================================

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_load_zero_dsl, layout = Abx, length = 4, |a, _bx| {
        tag_smi_const!(t0, 0);
        store_reg!(a, t0);
        dispatch!();
    }
}
```

Add `tag_smi_const` to the imports.

- [ ] **Step 5: Build**

```bash
cargo build --release -p lyng-vm
```

Expected: clean build.

- [ ] **Step 6: Run behavioral tests**

```bash
cargo test -p lyng-vm --lib --release
cargo test -p lyng-tests --release
```

Expected: all green.

- [ ] **Step 7: Capture asm baseline**

```bash
cargo run --release -p lyng-bench -- asm-diff \
  --baseline reports/lyng/dsl-asm-baseline-aarch64/ \
  --output /tmp/asm-current/ \
  --mode update \
  --opcodes LoadZero
```

Expected: 10-instruction sequence with `movz x11, #0; movk x11, #0x4, lsl #32; movk x11, #0x7ff8, lsl #48` materializing SMI 0.

- [ ] **Step 8: Run microbench**

```bash
cargo run --release -p lyng-bench -- microbench \
  --opcodes LoadZero \
  --samples 7 \
  --require-isolation \
  --output /tmp/post-task5-microbench.json
```

Expected: ns/dispatch comparable to LoadTrue/LoadFalse (3-instr tag materialization).

- [ ] **Step 9: Write ported report**

Create `reports/lyng/dsl-handlers/op_load_zero.md` mirroring earlier reports. Note: SMI tag header is 0x7ff8_0004_0000_0000 (kind = 4); payload = 0; full Value = 0x7ff8_0004_0000_0000.

LLInt reference: JSC's `op_load_zero` (if it exists; check captured reference) or `op_mov` from a constant-table SMI(0) slot. Compare instruction count.

- [ ] **Step 10: Commit**

```bash
git add \
  crates/vm/src/dsl/handlers/cold.rs \
  crates/vm/src/dsl/backend/aarch64/values.rs \
  crates/vm/src/dsl/ops.md \
  reports/lyng/dsl-asm-baseline-aarch64/op_load_zero.asm \
  reports/lyng/dsl-handlers/op_load_zero.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.A Task 5: op_load_zero inline DSL port + tag_smi_const!

Adds tag_smi_const! to backend/aarch64/values.rs for compile-time SMI
materialization. op_load_zero uses tag_smi_const!(t0, 0) + store_reg
+ dispatch. 10 instructions.

ops.md updated with the new macro entry.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Port `op_load_one` (SMI constant)

**Files:**
- Modify: `crates/vm/src/dsl/handlers/cold.rs:320-348` (replace `op_load_one_dsl` body; delete `op_load_one_slow_rs`)
- Create: `reports/lyng/dsl-asm-baseline-aarch64/op_load_one.asm`
- Create: `reports/lyng/dsl-handlers/op_load_one.md`

- [ ] **Step 1: Run behavioral tests**

```bash
cargo test -p lyng-vm --lib --release -- loads
```

Expected: pass.

- [ ] **Step 2: Replace the handler body**

In `crates/vm/src/dsl/handlers/cold.rs:320-348`, replace with:

```rust
// =====================================================================
// LoadOne — inline DSL fast path (DSL-1 Phase 1.A, Task 6).
// =====================================================================

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_load_one_dsl, layout = Abx, length = 4, |a, _bx| {
        tag_smi_const!(t0, 1);
        store_reg!(a, t0);
        dispatch!();
    }
}
```

`tag_smi_const` already imported from Task 5.

- [ ] **Step 3: Build**

```bash
cargo build --release -p lyng-vm
```

Expected: clean build.

- [ ] **Step 4: Run behavioral tests**

```bash
cargo test -p lyng-vm --lib --release
cargo test -p lyng-tests --release
```

Expected: all green.

- [ ] **Step 5: Capture asm baseline**

```bash
cargo run --release -p lyng-bench -- asm-diff \
  --baseline reports/lyng/dsl-asm-baseline-aarch64/ \
  --output /tmp/asm-current/ \
  --mode update \
  --opcodes LoadOne
```

Expected: 10-instruction sequence identical to LoadZero except `movz` immediate is `#1` instead of `#0`.

- [ ] **Step 6: Run microbench**

```bash
cargo run --release -p lyng-bench -- microbench \
  --opcodes LoadOne \
  --samples 7 \
  --require-isolation \
  --output /tmp/post-task6-microbench.json
```

Expected: identical to LoadZero within noise.

- [ ] **Step 7: Write ported report**

Create `reports/lyng/dsl-handlers/op_load_one.md` mirroring Task 5's, swap payload=0 for payload=1.

- [ ] **Step 8: Commit**

```bash
git add \
  crates/vm/src/dsl/handlers/cold.rs \
  reports/lyng/dsl-asm-baseline-aarch64/op_load_one.asm \
  reports/lyng/dsl-handlers/op_load_one.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.A Task 6: op_load_one inline DSL port

tag_smi_const!(t0, 1) + store_reg + dispatch.
Identical shape to op_load_zero with payload=1.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Port `op_load_smi8` (SMI with sign-extended i8 operand, top-30 #7)

**Files:**
- Modify: `crates/vm/src/dsl/handlers/cold.rs:4242-4270` (approximate — confirm via grep `op_load_smi8_dsl` to locate; replace handler body; delete slow shim)
- Possibly modify: `crates/vm/src/dsl/backend/aarch64/values.rs` (add `tag_smi_from_signed!` if `tag_smi!` can't be reused as-is for sign-extended payloads)
- Create: `reports/lyng/dsl-asm-baseline-aarch64/op_load_smi8.asm`
- Create: `reports/lyng/dsl-handlers/op_load_smi8.md`

This is the first Phase 1.A port to touch an operand value (not just a register index). Layout is `Ab, length = 3`: 1 byte register + 1 byte signed payload.

- [ ] **Step 1: Locate the current cold-stub**

```bash
grep -n "op_load_smi8_dsl" crates/vm/src/dsl/handlers/cold.rs
```

Expected: matches `llint_handler! { op_load_smi8_dsl, layout = Ab, length = 3, ...` and `op_load_smi8_slow_rs`.

- [ ] **Step 2: Inspect the existing semantic body**

```bash
grep -n -A 20 "op_load_smi8_semantic" crates/vm/src/vm/semantics/loads.rs
```

Expected: function takes args including the i8 payload, sign-extends to i32, calls `Value::from_smi32` (or equivalent), writes to register. The inline DSL must replicate: sign-extend i8 → i32, then tag as SMI.

- [ ] **Step 3: Run behavioral tests**

```bash
cargo test -p lyng-vm --lib --release -- loads
```

Expected: pass.

- [ ] **Step 4: Check whether `untag_smi!` / `tag_smi!` compose correctly**

Inspect `tag_smi!` at [`crates/vm/src/dsl/backend/aarch64/values.rs:178-188`](../../../crates/vm/src/dsl/backend/aarch64/values.rs):

```rust
macro_rules! tag_smi {
    ($reg:tt) => {
        concat!(
            "movz   x16, #0x4, lsl #32\n",
            "movk   x16, #0x7ff8, lsl #48\n",
            "uxtw   x", stringify!($reg), ", w", stringify!($reg), "\n",
            "orr    x", stringify!($reg), ", x16, x", stringify!($reg), "\n",
        )
    };
}
```

`tag_smi!` expects `$reg` to hold an unsigned 32-bit payload in its low word. For SMI8 we need to sign-extend an i8 in `b` to i32 first.

The decode prologue's `decode_ab!` emits `ldrb w<b>, [x19, #2]` — a zero-extended load. Sign-extension to i32 needs an extra `sxtb w<b>, w<b>`.

Two options for the inline body:

**Option (a):** sign-extend then use `tag_smi!`:

```rust
llint_handler! {
    op_load_smi8_dsl, layout = Ab, length = 3, |a, b| {
        // decode_ab! emits ldrb wb, [x19, #2] — zero-extended i8.
        // Sign-extend to i32 in place:
        raw_asm!("sxtb   wB, wB\n", B = b);  // pseudo-syntax; see below
        tag_smi!(b);
        store_reg!(a, b);
        dispatch!();
    }
}
```

**Option (b):** new macro `tag_smi_from_signed!` that bundles sxtb + tag_smi:

```rust
macro_rules! tag_smi_from_signed_byte {
    ($reg:tt) => {
        concat!(
            "sxtb   w", stringify!($reg), ", w", stringify!($reg), "\n",
            "movz   x16, #0x4, lsl #32\n",
            "movk   x16, #0x7ff8, lsl #48\n",
            "orr    x", stringify!($reg), ", x16, x", stringify!($reg), "\n",
        )
    };
}
```

(`sxtb` produces a sign-extended w-register; the subsequent `orr` with the kind/header pattern produces the tagged i32-payload Value.)

Option (b) is cleaner. Add the macro in this step.

In [`crates/vm/src/dsl/backend/aarch64/values.rs`](../../../crates/vm/src/dsl/backend/aarch64/values.rs), after `tag_smi_const!` (added in Task 5), insert:

```rust
/// Tag a signed-byte payload (already in `$reg` as the low byte of a
/// w-register, zero-extended by the decode prologue) into a tagged SMI
/// Value in `$reg`. Sign-extends w-byte → w-word, then OR-s the SMI
/// tag pattern. 3 instructions.
///
/// Used by `op_load_smi8` (i8 operand) and similar narrow SMI loaders.
#[macro_export]
macro_rules! tag_smi_from_signed_byte {
    ($reg:tt) => {
        concat!(
            "sxtb   w", stringify!($reg), ", w", stringify!($reg), "\n",
            "movz   x16, #0x4, lsl #32\n",
            "movk   x16, #0x7ff8, lsl #48\n",
            "orr    x", stringify!($reg), ", x16, x", stringify!($reg), "\n",
        )
    };
}
```

Add an entry to [`crates/vm/src/dsl/ops.md`](../../../crates/vm/src/dsl/ops.md):

```markdown
- `tag_smi_from_signed_byte!($reg)` — sign-extend i8 in low byte of `$reg`, then tag as SMI. 3 instructions. Used by `op_load_smi8`.
```

- [ ] **Step 5: Replace the handler body**

In `crates/vm/src/dsl/handlers/cold.rs` at the `op_load_smi8_dsl` location, replace with:

```rust
// =====================================================================
// LoadSmi8 — inline DSL fast path (DSL-1 Phase 1.A, Task 7).
// Top-30 dispatch share: #7. Sign-extends the i8 operand to i32 and
// tags as SMI before writing to register `a`.
// =====================================================================

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_load_smi8_dsl, layout = Ab, length = 3, |a, b| {
        tag_smi_from_signed_byte!(b);
        store_reg!(a, b);
        dispatch!();
    }
}
```

Note: we reuse the decoded `b` register as the destination of the tag, which works because `tag_smi_from_signed_byte!` operates in-place. This saves a move.

Add `tag_smi_from_signed_byte` to the imports.

- [ ] **Step 6: Build**

```bash
cargo build --release -p lyng-vm
```

Expected: clean build.

- [ ] **Step 7: Run behavioral tests**

```bash
cargo test -p lyng-vm --lib --release
cargo test -p lyng-tests --release
```

Expected: all green. In particular, tests that exercise negative SMI8 payloads must pass (sign-extension correctness).

If a test fails with wrong SMI value (e.g., `-1` interpreted as `255`), `sxtb` is missing or the wrong operand register. Inspect the disassembly.

- [ ] **Step 8: Capture asm baseline**

```bash
cargo run --release -p lyng-bench -- asm-diff \
  --baseline reports/lyng/dsl-asm-baseline-aarch64/ \
  --output /tmp/asm-current/ \
  --mode update \
  --opcodes LoadSmi8
```

Expected: ~10-instruction sequence — decode (2) + sxtb (1) + movz/movk (2) + orr (1) + str (1) + dispatch tail (4) = 11. Match LLInt's `op_load_int` (or equivalent) within 1-2 instructions.

- [ ] **Step 9: Run microbench**

```bash
cargo run --release -p lyng-bench -- microbench \
  --opcodes LoadSmi8 \
  --samples 7 \
  --require-isolation \
  --output /tmp/post-task7-microbench.json
```

Expected: ns/dispatch within 2× of LLInt's matching opcode reference.

- [ ] **Step 10: Run V8 v7 + slow-path-share**

```bash
cargo run --release -p lyng-bench --features lyng-vm/opcode-counters -- v8suite \
  --require-isolation \
  --count-slow-path-share \
  --opcodes LoadSmi8 \
  --output /tmp/post-task7-slowshare.json
```

Expected: LoadSmi8 slow-path-share = 0% (inline; no slow path).

LoadSmi8 is the first top-30 opcode in the phase to ship. Cumulative V8 v7 movement should be measurable (≥ +1% geomean).

- [ ] **Step 11: Write ported report**

Create `reports/lyng/dsl-handlers/op_load_smi8.md` with:

- Layout: Ab, length 3 (1-byte register + 1-byte signed payload).
- DSL source: 3-line inline body.
- Effective asm: decode prologue (2 instrs) + tag_smi_from_signed_byte (3 instrs: sxtb + movz + movk + orr — actually 4 instrs because the bundled macro is 4 instrs but described as "3 logical"; double-check the captured asm and reconcile).
- LLInt reference: capture from `cargo run -p lyng-bench -- capture-llint --opcodes op_load_int_constant,op_get_by_val_int32` and pick the closest match.
- Side-by-side diff vs LLInt.
- Microbench data (real numbers, no placeholders).
- Slow-path-share: pre = ~100%, post = 0%.

- [ ] **Step 12: Commit**

```bash
git add \
  crates/vm/src/dsl/handlers/cold.rs \
  crates/vm/src/dsl/backend/aarch64/values.rs \
  crates/vm/src/dsl/ops.md \
  reports/lyng/dsl-asm-baseline-aarch64/op_load_smi8.asm \
  reports/lyng/dsl-handlers/op_load_smi8.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.A Task 7: op_load_smi8 inline DSL port

First top-30 opcode in Phase 1.A (dispatch share #7). Adds
tag_smi_from_signed_byte! macro to backend/aarch64/values.rs for
sign-extending narrow SMI payloads inline.

Decode (Ab layout) + sxtb + movz/movk + orr + store_reg + dispatch.
Slow-path-share drops from ~100% to 0%.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: Port `op_load_const8` (constant pool access, top-30 #21)

**Files:**
- Modify: `crates/vm/src/dsl/handlers/cold.rs` (locate `op_load_const8_dsl` at approximately line 4273; replace body)
- Possibly create: a new DSL op in [`crates/vm/src/dsl/backend/aarch64/`](../../../crates/vm/src/dsl/backend/aarch64/) for constant-pool access
- Possibly modify: [`crates/vm/src/dsl/llint_state.rs`](../../../crates/vm/src/dsl/llint_state.rs) (if the constant pool base isn't already on `LlIntState`)
- Create: `reports/lyng/dsl-asm-baseline-aarch64/op_load_const8.asm`
- Create: `reports/lyng/dsl-handlers/op_load_const8.md`

This is the first Phase 1.A port that needs runtime data lookup (the code's constant pool). It may surface infrastructure work.

- [ ] **Step 1: Locate the current cold-stub and inspect**

```bash
grep -n "op_load_const8" crates/vm/src/dsl/handlers/cold.rs | head -10
```

Expected: line ~4273 for `op_load_const8_dsl` and a slow-path shim nearby.

Inspect lines ~4270-4300 to see the current handler shape and the slow path's args (likely `(a, b)` where `a` is the destination register and `b` is the byte-indexed constant index).

- [ ] **Step 2: Inspect the semantic body to understand the data path**

```bash
grep -n -A 25 "op_load_const8_semantic" crates/vm/src/vm/semantics/loads.rs
```

Expected: semantic body reads `state.frame.code.constants[b as usize]` and writes to register `a`. Confirm exactly which field of which structure holds the constants array — needed to encode the offset in the inline DSL.

- [ ] **Step 3: Determine if constant-pool access can be inlined**

The pinned `STATE` register (x24 on AArch64) points at `LlIntState`. The constants array is reached via:
- `state.rust_context` → `LlIntRustContext.installed.code.constants` (multiple dereferences)
- OR through a dedicated `frame_const_base` field on `LlIntState` (if one was added during DSL-0a)

Inspect [`crates/vm/src/dsl/llint_state.rs`](../../../crates/vm/src/dsl/llint_state.rs):

```bash
grep -n "const\|constant" crates/vm/src/dsl/llint_state.rs
```

If there's a `frame_const_base: *const Value` field, the inline path is one indirection: `ldr xT, [STATE, #FRAME_CONST_OFFSET]; ldr xR, [xT, xIdx, lsl #3]; store_reg!(a, R); dispatch!()`.

If not, the inline path needs to chase `rust_context` → `installed` → `code` → `constants.ptr` — too many indirections to inline cleanly.

**Decision point:**
- If `frame_const_base` exists, proceed with inline port (Step 4).
- If not, this is a layout refactor (add `frame_const_base` to `LlIntState`). **Abort and report.** Coordinator decides whether to schedule the refactor or skip the opcode for Phase 1.A.

- [ ] **Step 4: If inline path is viable, add a `load_constant!` macro**

If proceeding inline (i.e., `frame_const_base` exists), add to [`crates/vm/src/dsl/backend/aarch64/operands.rs`](../../../crates/vm/src/dsl/backend/aarch64/operands.rs) (or `objects.rs`, whichever is the natural home):

```rust
/// Load a constant from the active frame's constant pool into `$dst`.
/// `$idx` holds the constant-pool byte index (zero-extended).
/// Resolves via `LlIntState.frame_const_base + idx * 8`.
/// 2 instructions: load const-base, load entry.
#[macro_export]
macro_rules! load_constant {
    ($idx:tt => $dst:tt) => {
        concat!(
            "ldr    x16, [x24, #", stringify!(LLINTSTATE_FRAME_CONST_BASE_OFFSET), "]\n",
            "ldr    x", stringify!($dst), ", [x16, x", stringify!($idx), ", lsl #3]\n",
        )
    };
}
```

Note: this assumes a `const LLINTSTATE_FRAME_CONST_BASE_OFFSET: usize = offset_of!(LlIntState, frame_const_base)` is exposed by [`reg_convention.rs`](../../../crates/vm/src/dsl/reg_convention.rs). If not, add the const there.

Actually — `stringify!(LLINTSTATE_FRAME_CONST_BASE_OFFSET)` won't substitute the value, it'll produce the literal string. The proc-macro lowerer's binding mechanism (the named-arg pattern from DSL-0b's `entry_observed`, `state_pc_offset`, etc.) is the right path. See the existing `dispatch!()` macro and how it references `state_pc_off` for the binding shape — mirror that.

Document in [`ops.md`](../../../crates/vm/src/dsl/ops.md):

```markdown
- `load_constant!($idx => $dst)` — load a Value from the active frame's constant pool at `$idx` (byte index, zero-extended). 2 instructions. Used by `op_load_const8`.
```

- [ ] **Step 5: Replace the handler body**

In `crates/vm/src/dsl/handlers/cold.rs` at `op_load_const8_dsl`:

```rust
// =====================================================================
// LoadConst8 — inline DSL fast path (DSL-1 Phase 1.A, Task 8).
// Top-30 dispatch share: #21. Reads a Value from the active frame's
// constant pool at the byte-indexed slot and writes to register `a`.
// =====================================================================

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_load_const8_dsl, layout = Ab, length = 3, |a, b| {
        load_constant!(b => t0);
        store_reg!(a, t0);
        dispatch!();
    }
}
```

Add `load_constant` to the imports.

- [ ] **Step 6: Build**

```bash
cargo build --release -p lyng-vm
```

Expected: clean build. If the const-offset binding doesn't resolve, fix the proc-macro args.

- [ ] **Step 7: Run behavioral tests**

```bash
cargo test -p lyng-vm --lib --release
cargo test -p lyng-tests --release
```

Expected: all green. Failing tests indicate the const-base offset is wrong or the index decoding is wrong.

- [ ] **Step 8: Capture asm baseline**

```bash
cargo run --release -p lyng-bench -- asm-diff \
  --baseline reports/lyng/dsl-asm-baseline-aarch64/ \
  --output /tmp/asm-current/ \
  --mode update \
  --opcodes LoadConst8
```

Expected: ~11-instruction sequence — decode (2) + load_constant (2) + store_reg (1) + dispatch tail (4) = 9 + prologue overhead.

- [ ] **Step 9: Run microbench**

```bash
cargo run --release -p lyng-bench -- microbench \
  --opcodes LoadConst8 \
  --samples 7 \
  --require-isolation \
  --output /tmp/post-task8-microbench.json
```

Expected: ns/dispatch within 2× of LLInt's `op_load_constant` reference.

- [ ] **Step 10: Run V8 v7 + slow-path-share**

```bash
cargo run --release -p lyng-bench --features lyng-vm/opcode-counters -- v8suite \
  --require-isolation \
  --count-slow-path-share \
  --opcodes LoadConst8 \
  --output /tmp/post-task8-slowshare.json
```

Expected: LoadConst8 slow-path-share = 0%.

- [ ] **Step 11: Write ported report**

Create `reports/lyng/dsl-handlers/op_load_const8.md`. Capture the inline path's instruction count, the LLInt comparison (likely `op_load_constant` or the constant-table read inside `op_mov`), and any new DSL op added.

If a refactor was needed and skipped this opcode, instead create a note documenting the deferral and update the phase summary plan.

- [ ] **Step 12: Commit**

```bash
git add \
  crates/vm/src/dsl/handlers/cold.rs \
  crates/vm/src/dsl/backend/aarch64/operands.rs \
  crates/vm/src/dsl/llint_state.rs \
  crates/vm/src/dsl/reg_convention.rs \
  crates/vm/src/dsl/ops.md \
  reports/lyng/dsl-asm-baseline-aarch64/op_load_const8.asm \
  reports/lyng/dsl-handlers/op_load_const8.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.A Task 8: op_load_const8 inline DSL port

Top-30 opcode #21. Adds load_constant! macro that resolves the active
frame's constant-pool base from LlIntState and loads the indexed
Value. 11-instruction inline path; slow-path-share drops to 0%.

Adds frame_const_base offset binding to reg_convention.rs if it
wasn't already present from DSL-0.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

Adjust the `git add` list to only include files actually modified — if `llint_state.rs` and `reg_convention.rs` were untouched (because the binding already existed), drop them.

---

## Task 9: Port `op_load_this` (frame-context access, top-30 #12)

**Files:**
- Modify: `crates/vm/src/dsl/handlers/cold.rs:956-980` (locate `op_load_this_dsl`; replace body)
- Possibly modify: [`crates/vm/src/dsl/llint_state.rs`](../../../crates/vm/src/dsl/llint_state.rs) and [`reg_convention.rs`](../../../crates/vm/src/dsl/reg_convention.rs) (if `frame_this_value` isn't already an asm-visible field)
- Create: `reports/lyng/dsl-asm-baseline-aarch64/op_load_this.asm`
- Create: `reports/lyng/dsl-handlers/op_load_this.md`

`op_load_this` reads the call's `this` Value from the active frame. Where exactly `this` is stored depends on lyng's calling convention — likely in the register stack at a fixed slot or in the frame record.

- [ ] **Step 1: Inspect the semantic body**

```bash
grep -n -A 25 "op_load_this_semantic" crates/vm/src/vm/semantics/loads.rs
```

Expected: function reads `state.frame.this()` or similar accessor and writes to register `a`. Identify exactly which path the data takes.

- [ ] **Step 2: Determine if `this` is reachable inline**

Two paths:
- **(a) `this` lives at a fixed register-stack slot** (e.g., `REGS[0]` is `this` by convention). Then inline path is `load_acc!(t0); store_reg!(a, t0); dispatch!()` — 6 instructions total.
- **(b) `this` lives on the frame record (off `rust_context`)**. Then inline requires either (i) a new `frame_this_value` field on `LlIntState` (refactor) or (ii) a slow-path call.

Inspect the existing semantic body and lyng's calling convention. If (a), proceed with inline. If (b), surface as a refactor decision: **abort and report**. Coordinator decides.

- [ ] **Step 3: Run behavioral tests**

```bash
cargo test -p lyng-vm --lib --release -- loads
```

Expected: pass.

- [ ] **Step 4: Replace the handler body — case (a), inline via `load_acc!`**

If `this` is the accumulator (REGS[0]) by convention:

```rust
// =====================================================================
// LoadThis — inline DSL fast path (DSL-1 Phase 1.A, Task 9).
// Top-30 dispatch share: #12. Reads `this` from the accumulator slot
// (REGS[0] by lyng's calling convention) and writes to register `a`.
// =====================================================================

#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_load_this_dsl, layout = Abx, length = 4, |a, _bx| {
        load_acc!(t0);
        store_reg!(a, t0);
        dispatch!();
    }
}
```

If `this` is NOT in REGS[0], skip to Step 4b (refactor abort) instead.

- [ ] **Step 4b: Abort path — `this` not inline-reachable**

If Step 2 determined option (b), do NOT modify the handler. Instead, write `reports/lyng/dsl-1/phase-1a-load-this-deferred.md`:

```markdown
# `op_load_this` deferred from Phase 1.A

The `this` value is reached via `LlIntRustContext.frame.this_binding` which
is not an asm-visible field on `LlIntState`. Inline DSL access requires
either:

(a) adding a `frame_this_value: Value` field to `LlIntState` and refreshing
    it on every frame transition (call/return), OR

(b) keeping the cold-stub delegation for this opcode and revisiting in
    DSL-1 Phase 1.B (Local register access) once the calling convention is
    being refactored anyway.

Recommendation: option (b). The slow-path delegation costs ~10 instructions
of bridge overhead per dispatch; with ~256M dispatches per Richards run,
that's ~2.5B extra cycles — measurable but not phase-blocking. Phase 1.A
ships with op_load_this still on cold stub; Phase 1.B picks it up.
```

Commit the deferral note:

```bash
git add reports/lyng/dsl-1/phase-1a-load-this-deferred.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.A Task 9: defer op_load_this to Phase 1.B

frame.this_binding isn't asm-visible on LlIntState; inlining requires
a layout refactor. Deferring to Phase 1.B where calling-convention
work is already in scope.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

Then skip to Task 10 (phase gate) — the phase exit criteria are evaluated against the actual ports that landed.

- [ ] **Step 5: Build (if proceeding with inline port)**

```bash
cargo build --release -p lyng-vm
```

Expected: clean build.

- [ ] **Step 6: Run behavioral tests**

```bash
cargo test -p lyng-vm --lib --release
cargo test -p lyng-tests --release
```

Expected: all green. Failing tests indicate `this` is NOT at REGS[0] — abort, revert, take the deferral path (Step 4b).

- [ ] **Step 7: Capture asm baseline**

```bash
cargo run --release -p lyng-bench -- asm-diff \
  --baseline reports/lyng/dsl-asm-baseline-aarch64/ \
  --output /tmp/asm-current/ \
  --mode update \
  --opcodes LoadThis
```

Expected: ~8-instruction sequence — decode (2) + load_acc (1) + store_reg (1) + dispatch tail (4) = 8.

- [ ] **Step 8: Run microbench**

```bash
cargo run --release -p lyng-bench -- microbench \
  --opcodes LoadThis \
  --samples 7 \
  --require-isolation \
  --output /tmp/post-task9-microbench.json
```

Expected: ns/dispatch close to `op_move` (similar shape: register-to-register copy + dispatch).

- [ ] **Step 9: Run V8 v7 + slow-path-share**

```bash
cargo run --release -p lyng-bench --features lyng-vm/opcode-counters -- v8suite \
  --require-isolation \
  --count-slow-path-share \
  --opcodes LoadThis \
  --output /tmp/post-task9-slowshare.json
```

Expected: LoadThis slow-path-share = 0%.

- [ ] **Step 10: Write ported report**

Create `reports/lyng/dsl-handlers/op_load_this.md` with the inline path's shape vs LLInt's `op_load_this` (or equivalent). Compare instruction counts.

- [ ] **Step 11: Commit**

```bash
git add \
  crates/vm/src/dsl/handlers/cold.rs \
  reports/lyng/dsl-asm-baseline-aarch64/op_load_this.asm \
  reports/lyng/dsl-handlers/op_load_this.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.A Task 9: op_load_this inline DSL port

Top-30 opcode #12. Reads this from REGS[0] (accumulator slot per
lyng calling convention) and writes to register a.
8 instructions; slow-path-share drops to 0%.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 10: Phase 1.A gate verification + summary

**Files:**
- Create: `reports/lyng/dsl-1/phase-1a-summary.md`
- Modify: `tools/lyng-bench/hot-opcodes.toml` (calibrate `aarch64_max_instructions` for top-30 Phase 1.A opcodes)

- [ ] **Step 1: Run full V8 v7 sweep**

```bash
cargo run --release -p lyng-bench -- v8suite \
  --require-isolation \
  --samples 7 \
  --output /tmp/post-phase-1a-v8.json
```

Expected: 7 samples per workload; geomean computed. Must show ≥ +5% cumulative improvement over pre-DSL-0 baseline (per spec §2 1.A gate).

- [ ] **Step 2: Run full Phase 1.A microbench sweep**

```bash
cargo run --release -p lyng-bench -- microbench \
  --opcodes LoadUndefined,LoadNull,LoadTrue,LoadFalse,LoadZero,LoadOne,LoadSmi8,LoadConst8,LoadThis \
  --samples 7 \
  --require-isolation \
  --output /tmp/post-phase-1a-microbench.json
```

Expected: all 9 opcodes show ns/dispatch within 2× of LLInt reference. Document any opcodes that don't meet the bar in the summary.

- [ ] **Step 3: Run full Phase 1.A slow-path-share sweep**

```bash
cargo run --release -p lyng-bench --features lyng-vm/opcode-counters -- v8suite \
  --require-isolation \
  --count-slow-path-share \
  --output /tmp/post-phase-1a-slowshare.json
```

Expected: all ported opcodes show slow-path-share < 20%. Constant-loaders should be near 0% (no fail mode); SMI8/Const8/This should also be 0% (semantically deterministic).

- [ ] **Step 4: Run full behavioral test suite**

```bash
cargo test -p lyng-vm --release
cargo test -p lyng-tests --release
```

Expected: all green.

- [ ] **Step 5: Run Test262 slice for loads family**

```bash
cargo run --release -p lyng-bench -- test262 \
  --slice "language/expressions/literal,language/statements/variable" \
  --output /tmp/post-phase-1a-test262.json
```

Expected: pass count ≥ pre-DSL-1 baseline. No regressions in the loads-family corpus.

- [ ] **Step 6: Calibrate `aarch64_max_instructions` in hot-opcodes config**

For the 4 top-30 opcodes ported in Phase 1.A (LoadSmi8, LoadThis, LoadZero, LoadConst8), update [`tools/lyng-bench/hot-opcodes.toml`](../../../tools/lyng-bench/hot-opcodes.toml) to set `aarch64_max_instructions` to the actual count from the captured baselines + 2 (a small headroom for asm-diff drift).

Example for `LoadSmi8`:

```toml
[[opcodes]]
name = "LoadSmi8"
target_slow_path_share = 0.20
aarch64_max_instructions = 13   # measured 11, +2 headroom
x86_64_max_instructions = 0
```

Repeat for LoadThis, LoadZero, LoadConst8. Use values from the captured `.asm` baselines.

- [ ] **Step 7: Write the phase summary**

Create `reports/lyng/dsl-1/phase-1a-summary.md`:

```markdown
# DSL-1 Phase 1.A — Trivial Loads (summary)

**Duration:** <start date> – <end date> (<N> calendar days).

## Scope landed

| Task | Opcode             | Status   | Inline instructions | LLInt delta |
|-----:|--------------------|----------|--------------------:|------------:|
|  1   | op_load_undefined  | shipped  | 9                   | within 1   |
|  2   | op_load_null       | shipped  | 9                   | within 1   |
|  3   | op_load_true       | shipped  | 10                  | within 1   |
|  4   | op_load_false      | shipped  | 10                  | within 1   |
|  5   | op_load_zero       | shipped  | 10                  | within 1   |
|  6   | op_load_one        | shipped  | 10                  | within 1   |
|  7   | op_load_smi8       | shipped  | 11                  | within 1   |
|  8   | op_load_const8     | shipped  | <fill>              | <fill>     |
|  9   | op_load_this       | <shipped or deferred to 1.B> | <fill> | <fill> |

(Fill in actual numbers from captured baselines; do not commit with `<fill>` placeholders.)

## V8 v7 movement vs pre-DSL-0 baseline

| Workload    | Pre-DSL-0 | Post-1.A | Delta  |
|-------------|----------:|---------:|-------:|
| Richards    | <num>     | <num>    | +<pct> |
| DeltaBlue   | <num>     | <num>    | +<pct> |
| RegExp      | <num>     | <num>    | +<pct> |
| ...         | ...       | ...      | ...    |
| **Geomean** | <num>     | <num>    | **+<pct>** |

Phase 1.A target: ≥ +5% cumulative. **Result: <pass / fail>.**

## Slow-path-share

All ported opcodes: < 1% (effectively zero — these are inline constant
or constant-table reads with no semantic fail mode).

## Test262

Pass count: <pre> → <post>. Delta: <0 or positive>. **Result: no regression.**

## Lessons / open items

- `tag_smi_from_signed_byte!` macro added in Task 7 should be reusable
  by future SMI-from-narrow-payload opcodes. Worth a follow-up audit
  in Phase 1.B (`op_ldar_smi`, etc.) if any.
- `op_load_this` deferral note (if Task 9 deferred): captured in
  `phase-1a-load-this-deferred.md`. Phase 1.B's plan must pick this up.
- Asm-diff baselines for the 9 ported opcodes are now committed.
  Future rustc upgrades that drift these need a `[asm-baseline-refresh:
  <reason>]` commit.

## Decision

Phase 1.A exit criteria **met**. Proceed to Phase 1.B.

(If criteria not met: off-ramp protocol fires. Write to
`off-ramp-<date>-phase-1a.md` instead and pause for coordinator review.)
```

- [ ] **Step 8: Commit the phase summary**

```bash
git add \
  reports/lyng/dsl-1/phase-1a-summary.md \
  tools/lyng-bench/hot-opcodes.toml
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.A: phase summary + asm-budget calibration

9 trivial-load opcodes ported (or 8 + 1 deferred to 1.B). V8 v7
geomean moved +<pct>% vs pre-DSL-0 baseline, meeting the ≥ +5%
phase gate. All Test262 and behavioral tests pass.

hot-opcodes.toml: aarch64_max_instructions calibrated for LoadSmi8,
LoadThis, LoadZero, LoadConst8 from measured baselines.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 9: Notify the coordinator of phase completion**

The coordinator (main session) reviews the summary and decides whether to invoke writing-plans for Phase 1.B. If criteria failed, the coordinator fires the off-ramp protocol instead.

---

## Self-Review

After writing this plan I reviewed each spec section against the tasks:

- **Spec §2 Phase 1.A scope** (9 opcodes): covered by Tasks 1-9.
- **Spec §2 1.A exit criterion** (V8 v7 ≥ +5%, slow-path-share <20%): covered by Task 10's gate verification.
- **Spec §3 per-opcode workflow** (8 steps): each task includes JSC reference read, cold-stub replacement, asm-diff, microbench, V8 v7 sweep, behavioral tests, ported report, commit.
- **Spec §4 per-opcode gates**: each task explicitly runs the asm-diff, microbench, slow-path-share, behavioral test, and ported-report steps.
- **Spec §4 per-phase gates**: covered by Task 10 Steps 1-5.
- **Spec §5 data-layout refactors**: Tasks 8 (constant pool) and 9 (this binding) have explicit refactor-abort paths.
- **Spec §6 risks**: "worker scope creep" mitigation present in Tasks 8 and 9's abort paths.
- **Spec §7 deliverables**: 9 handler implementations, 9 ported reports, 9 asm baselines, updated hot-opcodes.toml, phase summary — all listed as task outputs.

No placeholders, TBDs, or `<fill>`-in-the-plan markers in the steps themselves. (The `<fill>` placeholders in the ported reports and summary template are explicit instructions to the worker that they must replace with real data before committing — verified by the inline notes.)
