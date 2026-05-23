# DSL-1 Phase 1.C — SMI arithmetic + bitwise — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port 7 SMI arithmetic + bitwise opcodes from cold-stub delegation to inline DSL fast paths, adding ~1.75B inlined dispatches per V8 v7 run on top of Phase 1.B's +8.51% cumulative baseline.

**Architecture:** Three sub-phases grouped by asm shape. 1.C.1 binary-with-overflow (op_sub, op_mul); 1.C.2 bitwise-no-overflow (op_bit_and, op_shift_left, op_shift_right); 1.C.3 unary-with-new-macros (op_increment, op_decrement). Each port replaces a cold-stub `llint_handler!` body in `crates/vm/src/dsl/handlers/cold.rs` with inline asm assembled from existing backend macros (`check_smi!`, `untag_smi!`, the arithmetic macros `*_smi_overflow!` / `*_smi!`, `tag_smi!`, `store_reg!`), plus a per-opcode `op_xxx_record_smi_rs` shim for fast-path feedback recording (mirrors `op_add_record_smi_rs` in `hot.rs`).

**Tech Stack:** Rust 2024 edition, `naked_asm!` (AArch64-only), proc-macro lowerer `lyng-vm-dsl`, `lyng-bench` measurement tool (microbench / asm-diff / v8suite / count-slow-path-share / require-isolation). All work targets aarch64-apple-darwin.

**Spec:** [`docs/superpowers/specs/2026-05-21-dsl-1-phase-1c-smi-arith-and-bitwise-design.md`](../specs/2026-05-21-dsl-1-phase-1c-smi-arith-and-bitwise-design.md).

---

## Bench tool CLI reference (read this first)

Per `cargo run --release -p lyng-bench -- <subcommand> --help`, the actual CLIs are:

**Microbench** — runs ALL opcodes from `hot-opcodes.toml`; filter output post-hoc.
```bash
# Check loadavg first (--require-isolation aborts if 1-min loadavg > 2.0):
uptime
cargo run --release -p lyng-bench -- microbench --require-isolation --samples 7 \
  --output /tmp/microbench-<context>.md
grep -A 1 "<OpcodeName>" /tmp/microbench-<context>.md  # extract specific opcode row
```

**Per-opcode slow-path-share** — requires `--count-opcodes --count-slow-path-share` (NOT `--require-isolation`; the v8suite subcommand doesn't accept that flag — manage loadavg manually by checking `uptime` before/after).
```bash
uptime  # verify loadavg <= 2.0 before starting
cargo run --release -p lyng-bench -- v8suite --samples 5 \
  --count-opcodes --count-slow-path-share \
  --counts-json /tmp/v8-share-<context>.json \
  --report /tmp/v8-counts-<context>.md
uptime  # record loadavg at end too
# Parse JSON to find per-opcode share:
jq '.workloads[] | {name, opcodes: [.opcodes[] | select(.name == "<OpcodeName>")]}' /tmp/v8-share-<context>.json
```

**V8 v7 A/B between two binaries** — there is NO `ab` subcommand. The protocol from Phase 1.B.3 (see `reports/lyng/dsl-1/phase-1b3-cumulative-ab.md`) is:
```bash
# Build the base binary in a worktree
git worktree add /tmp/wt-base <base-commit-sha>
(cd /tmp/wt-base && cargo build --release -p lyng)
cp /tmp/wt-base/target/release/lyng /tmp/lyng-base
git worktree remove /tmp/wt-base

# Record loadavg, then run v8suite against base binary
uptime
cargo run --release -p lyng-bench -- v8suite --samples 11 \
  --lyng-bin /tmp/lyng-base \
  --report /tmp/v8-base.md \
  --json /tmp/v8-base.json
uptime  # record loadavg at end

# Build the post binary (current HEAD)
cargo build --release -p lyng
cp target/release/lyng /tmp/lyng-post

# Record loadavg, run v8suite against post binary
uptime
cargo run --release -p lyng-bench -- v8suite --samples 11 \
  --lyng-bin /tmp/lyng-post \
  --report /tmp/v8-post.md \
  --json /tmp/v8-post.json
uptime

# Write the A/B comparison markdown manually (use Phase 1.B.3 format):
# - Capture 1m/5m/15m loadavg at start + end of each run
# - Compute ±% overlap at the 5-min point
# - Tabulate per-workload medians from /tmp/v8-base.md vs /tmp/v8-post.md
# - Compute geomean delta
# - Save to reports/lyng/dsl-1/phase-1c<N>-{ab-comparison|cumulative-ab}.md
```

**Asm baseline capture** — `asm-diff --check` doesn't auto-discover `dsl::handlers::cold::*` symbols (Phase 1.B followup), so each port task captures manually:
```bash
cargo rustc --release -p lyng-vm -- --emit=asm 2>/dev/null
ASM_FILE=$(ls -t target/release/deps/lyng_vm-*.s 2>/dev/null | head -1)
awk '/^_op_<name>_dsl:/,/^[[:space:]]*\.cfi_endproc/' "$ASM_FILE" \
  > reports/lyng/dsl-asm-baseline-aarch64/op_<name>.asm
```

Apply this reference whenever the per-task bench commands below appear — they're written in the same form.

---

## File structure

### Modified files

- `crates/vm/src/dsl/handlers/cold.rs` — replace 7 cold-stub `llint_handler!` bodies with inline fast paths; add 7 new `op_xxx_record_smi_rs` shims; update macro imports.
- `crates/vm/src/dsl/backend/aarch64/arithmetic.rs` — add 2 new macros (`inc_smi_overflow!`, `dec_smi_overflow!`).
- `crates/vm/src/dsl/ops.md` — add entries for the 2 new macros.
- `tools/lyng-bench/hot-opcodes.toml` — calibrate `aarch64_max_instructions` budgets for the 7 ports (replacing 0 placeholders).

### Created files

- `reports/lyng/dsl-handlers/op_sub.md` (and 6 more, one per ported opcode).
- `reports/lyng/dsl-asm-baseline-aarch64/op_sub.asm` (and 6 more — manual captures per Phase 1.B precedent).
- `reports/lyng/dsl-1/phase-1c1-summary.md`, `phase-1c2-summary.md`, `phase-1c3-summary.md`.
- `reports/lyng/dsl-1/phase-1c-summary.md`, `phase-1c-followups.md`, `phase-1c-cumulative-ab.md`.
- `crates/tests/src/dsl_increment_writeback.rs` (1 unit test for the SMI-elision claim in 1.C.3).
- `reports/lyng/asm-dsl-engine-state-<date>.md` (post-phase engine snapshot — optional, can be a followup).

---

# Sub-phase 1.C.0 — Substrate prep (new macros)

Two new backend macros for inc/dec, ~1 day. Self-review acceptable; runtime verification comes from 1.C.3 inline ports.

## Task 1: Add `inc_smi_overflow!` and `dec_smi_overflow!` macros

**Files:**
- Modify: `crates/vm/src/dsl/backend/aarch64/arithmetic.rs` (append after `bit_not_smi!`, around line 170)
- Modify: `crates/vm/src/dsl/ops.md` (add entries in the arithmetic section)

- [ ] **Step 1: Append `inc_smi_overflow!` and `dec_smi_overflow!` to arithmetic.rs**

Open `crates/vm/src/dsl/backend/aarch64/arithmetic.rs` and append after the last existing macro (`bit_not_smi!`):

```rust
/// 32-bit signed increment by 1 with overflow detection.
///
/// `$src` is an untagged SMI (sign-extended i32 in the low 32 bits of an
/// X-register). `$dst` receives the incremented payload sign-extended to
/// i64. On overflow, branch to `$label` (slow path).
///
/// `adds wD, wS, #1` accepts a 12-bit unsigned immediate (`#1` is well
/// within range), no scratch register needed. 3 instructions total:
/// adds + b.vs + sxtw.
#[macro_export]
macro_rules! inc_smi_overflow {
    ($src:tt => $dst:tt, $label:tt) => {
        concat!(
            "adds   w", stringify!($dst), ", w", stringify!($src), ", #1\n",
            "b.vs   ", stringify!($label), "\n",
            "sxtw   x", stringify!($dst), ", w", stringify!($dst), "\n",
        )
    };
}

/// 32-bit signed decrement by 1 with overflow detection.
///
/// `$src` is an untagged SMI; `$dst` receives the decremented payload
/// sign-extended to i64. On overflow (only at `i32::MIN`), branch to
/// `$label`.
///
/// `subs wD, wS, #1` accepts a 12-bit unsigned immediate (`#1` is well
/// within range), no scratch register needed. 3 instructions total:
/// subs + b.vs + sxtw.
#[macro_export]
macro_rules! dec_smi_overflow {
    ($src:tt => $dst:tt, $label:tt) => {
        concat!(
            "subs   w", stringify!($dst), ", w", stringify!($src), ", #1\n",
            "b.vs   ", stringify!($label), "\n",
            "sxtw   x", stringify!($dst), ", w", stringify!($dst), "\n",
        )
    };
}
```

- [ ] **Step 2: Add `ops.md` entries**

Open `crates/vm/src/dsl/ops.md`. Find the arithmetic table (search for `add_smi_overflow`). Add two rows after the bitwise rows for `bit_not_smi!`:

```markdown
| `inc_smi_overflow!` | `src => dst, label`   | `adds wDst, wSrc, #1; b.vs label; sxtw xDst, wDst`  | 3 instr |
| `dec_smi_overflow!` | `src => dst, label`   | `subs wDst, wSrc, #1; b.vs label; sxtw xDst, wDst`  | 3 instr |
```

(If the table format differs slightly in ops.md — e.g., uses different column headers — match the existing arithmetic-section row format exactly.)

- [ ] **Step 3: Build verify**

Run:
```
cargo build --release -p lyng-vm
```
Expected: compiles cleanly (no handlers use the new macros yet — they're only defined).

- [ ] **Step 4: Self-review**

Open `crates/vm/src/dsl/backend/aarch64/arithmetic.rs` and confirm:
- Both macros emit exactly 3 instructions.
- `inc_smi_overflow!` uses `adds` + `b.vs` + `sxtw`.
- `dec_smi_overflow!` uses `subs` + `b.vs` + `sxtw`.
- Both follow the existing macro style (docstring, `concat!`, `stringify!`, `#[macro_export]`).

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/dsl/backend/aarch64/arithmetic.rs crates/vm/src/dsl/ops.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.C.0 Task 1: inc_smi_overflow!/dec_smi_overflow! macros

Two new AArch64 backend macros mirroring add_smi_overflow!/sub_smi_overflow!
but using the 12-bit immediate form of adds/subs to avoid materializing a
scratch register for the literal 1. Each macro emits 3 instructions:
adds/subs + b.vs + sxtw.

Runtime verification deferred to Phase 1.C.3 inline ports of op_increment
and op_decrement (per Phase 1.B retrospective lesson #3 — substrate macros
need real-handler verification immediately).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

# Sub-phase 1.C.1 — Binary arith with overflow (op_sub, op_mul)

Two ports, ~3-4 days. op_sub first (mechanical mirror of op_add), then op_mul (smull+cmp overflow shape is slightly different).

## Task 2: Port op_sub inline fast path

**Files:**
- Modify: `crates/vm/src/dsl/handlers/cold.rs` — replace `op_sub_dsl` body (around line 1050) and add `op_sub_record_smi_rs` shim
- Create: `reports/lyng/dsl-handlers/op_sub.md`
- Create: `reports/lyng/dsl-asm-baseline-aarch64/op_sub.asm`
- Modify: `tools/lyng-bench/hot-opcodes.toml` — set `aarch64_max_instructions` for `Sub`

- [ ] **Step 1: Read the current cold-stub**

Open `crates/vm/src/dsl/handlers/cold.rs` and locate `op_sub_dsl` (currently around line 1050). The body looks like:

```rust
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_sub_dsl, opcode_byte = 33, layout = AbcSlot, length = 6, |a, b, c, slot| {
        call_slow!(op_sub_slow_rs, args = [a, b, c, slot]);
        dispatch_after_slow!();
    }
}
```

The existing `op_sub_slow_rs` shim at the next ~17 lines below is **unchanged** — the inline fast path's slow path delegates to it.

- [ ] **Step 2: Verify macro imports at the top of cold.rs**

Search the top of `cold.rs` for the `use crate::{...}` block. Confirm these macro imports are present (add any that are missing):

```rust
#[cfg(target_arch = "aarch64")]
use crate::{
    call_slow, check_smi, dispatch, dispatch_after_slow,
    load_reg, store_reg, sub_smi_overflow, tag_smi, untag_smi,
};
```

If `sub_smi_overflow` is missing, add it. Other macros (`call_slow`, `check_smi`, etc.) are almost certainly already imported because the cold-stub uses them.

- [ ] **Step 3: Replace the `op_sub_dsl` body with the inline fast path**

In `cold.rs`, replace the entire `llint_handler! { op_sub_dsl, ... }` block (the `call_slow! + dispatch_after_slow!` body) with:

```rust
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_sub_dsl, opcode_byte = 33, layout = AbcSlot, length = 6, |a, b, c, slot| {
        load_reg!(b => t0);
        check_smi!(t0, .slow);
        load_reg!(c => t1);
        check_smi!(t1, .slow);
        untag_smi!(t0);
        untag_smi!(t1);
        sub_smi_overflow!(t0, t1 => t2, .slow);
        tag_smi!(t2);
        store_reg!(a, t2);
        call_slow!(op_sub_record_smi_rs, args = [slot]);
        dispatch_after_slow!();
        .slow:
        call_slow!(op_sub_slow_rs, args = [a, b, c, slot]);
        dispatch_after_slow!();
    }
}
```

- [ ] **Step 4: Add the `op_sub_record_smi_rs` shim immediately after the `op_sub_dsl` block**

Add this shim (modeled on `op_add_record_smi_rs` in `crates/vm/src/dsl/handlers/hot.rs:88-106`) directly after the `op_sub_dsl` `llint_handler!` block and before the existing `op_sub_slow_rs`:

```rust
/// Fast-path feedback-recording shim for `op_sub`. Mirrors
/// `op_add_record_smi_rs` in hot.rs: bumps the warmup counter,
/// allocates the legacy vector at threshold, mirrors legacy state to
/// the flat array, observes the tier feedback event. Returns
/// `Continue { pc_advance: 6 }` so the asm bridge advances PC by op_sub's
/// encoded length without re-entering `op_sub_semantic`.
#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_sub_record_smi_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
    feedback_slot: u32,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    {
        let inner = dispatch.dispatch_state();
        let code = inner.code();
        inner
            .vm
            .record_feedback_slot(code, lyng_types::FeedbackSlotId::from_raw(feedback_slot));
    }
    dispatch.translate_outcome(crate::dsl::slow_path::SemanticOutcome::Continue {
        pc_advance: 6,
    })
}
```

- [ ] **Step 5: Build verify**

Run:
```
cargo build --release -p lyng-vm
```
Expected: clean compile. If a macro-expansion error fires, the most likely cause is a missing import (Step 2) or a label collision (the `.slow:` label is local to each `naked_asm!` block per `op_add`'s precedent, so should be fine).

- [ ] **Step 6: Run behavioral tests**

Run:
```
cargo test --release -p lyng-vm -p lyng-tests
```
Expected: 418 + 1209 tests pass. If any failure references op_sub or arithmetic, the inline path is wrong — re-read the op_add shape from `hot.rs:57-75` and compare.

- [ ] **Step 7: Run a focused Test262 slice for the arithmetic family**

Run:
```
cargo run --release -p lyng-tests -- --test-source test262 --filter language/expressions/subtraction
```
Expected: same pass rate as before. (If no matching filter exists, run `--filter language/expressions` for the broader slice.)

- [ ] **Step 8: Capture asm baseline manually**

`asm-diff --check` does not yet auto-discover the `dsl::handlers::cold::*` namespace (Phase 1.B followup). Capture manually:

```bash
cargo rustc --release -p lyng-vm -- --emit=asm 2>/dev/null
# Find the emitted asm file
ASM_FILE=$(ls -t target/release/deps/lyng_vm-*.s 2>/dev/null | head -1)
# Extract the op_sub_dsl symbol body
awk '/^_op_sub_dsl:/,/^[[:space:]]*\.cfi_endproc/' "$ASM_FILE" > reports/lyng/dsl-asm-baseline-aarch64/op_sub.asm
```

Open `reports/lyng/dsl-asm-baseline-aarch64/op_sub.asm` and confirm it contains the inline fast path (instructions for `ldr`/`movz`/`movk`/`and`/`movz`/`movk`/`cmp`/`b.ne`/... pattern matching op_add's baseline). If the file is empty or contains only `b op_sub_slow_rs`, the inline path was not emitted — debug.

- [ ] **Step 9: Run microbench**

Run:
```
cargo run --release -p lyng-bench -- microbench --opcodes Sub --require-isolation
```
Expected output: ns/dispatch value with confidence interval. Record the number for the ported report. Per the per-opcode gate, the ns/dispatch should be within 2× of JSC LLInt's op_sub.

If `microbench --opcodes Sub` reports "no snippet found" or similar, check `reports/lyng/dsl-1/phase-1b0-summary.md` for the snippet list and ensure a Sub microbench snippet exists in the bench tool's config. If not, add one mirroring the Add snippet from Phase 1.B.0 (this is a sub-task — handle it before continuing).

- [ ] **Step 10: Run slow-path-share isolated V8 v7 sweep**

Run:
```
cargo run --release -p lyng-bench -- v8suite --require-isolation --count-slow-path-share --opcodes Sub
```
Expected: per-opcode slow-path-share table. The Sub row must show < 20% on all V8 v7 workloads (or document a per-workload waiver in the ported report against an LLInt-on-same-workload baseline).

- [ ] **Step 11: Write the ported report**

Create `reports/lyng/dsl-handlers/op_sub.md`. Use this template (copy verbatim, then fill in the captured data):

````markdown
# `op_sub` DSL port (opcode 33, B33)

Phase 1.C.1 inline port: SMI binary subtract with overflow detection,
mirroring the op_add shape from DSL-0 / Phase 1.A.

## DSL source

`crates/vm/src/dsl/handlers/cold.rs` (around line 1050):

```rust
llint_handler! {
    op_sub_dsl, opcode_byte = 33, layout = AbcSlot, length = 6, |a, b, c, slot| {
        load_reg!(b => t0);
        check_smi!(t0, .slow);
        load_reg!(c => t1);
        check_smi!(t1, .slow);
        untag_smi!(t0);
        untag_smi!(t1);
        sub_smi_overflow!(t0, t1 => t2, .slow);
        tag_smi!(t2);
        store_reg!(a, t2);
        call_slow!(op_sub_record_smi_rs, args = [slot]);
        dispatch_after_slow!();
        .slow:
        call_slow!(op_sub_slow_rs, args = [a, b, c, slot]);
        dispatch_after_slow!();
    }
}
```

## Slow-path shims

- `op_sub_slow_rs` (unchanged; pre-existing cold-stub shim — invoked from the `.slow` label on SMI miss or overflow).
- `op_sub_record_smi_rs` (NEW; fast-path feedback recording — mirrors `op_add_record_smi_rs`). Lives next to the handler. See `crates/vm/src/dsl/handlers/cold.rs` around line <line>.

## Current asm

See `reports/lyng/dsl-asm-baseline-aarch64/op_sub.asm`.

Fast path: <X> instructions. Side-by-side with LLInt:

| Section            | op_sub | LLInt op_sub | Delta |
|--------------------|-------:|-------------:|------:|
| Operand decode     |        |              |       |
| check_smi (lhs)    |        |              |       |
| check_smi (rhs)    |        |              |       |
| untag x2           |        |              |       |
| subs / b.vs / sxtw |        |              |       |
| tag_smi            |        |              |       |
| store_reg          |        |              |       |
| record_smi (shim)  |        |              |       |
| dispatch           |        |              |       |
| **Total**          |        |              |       |

Slow path: 5 instructions for call setup + 1 bl + dispatch_after_slow trampoline.

## LLInt reference

JSC's op_sub uses adds-with-overflow + slow-path tail. Lyng's shape
differs only in NaN-tag layout (Lyng's TagKind in upper 16 of NaN-space
vs JSC's pointer-tagging) and feedback-recording representation (Lyng
goes through `record_feedback_slot` via a shim because the
`entry_observed` flat-array offset binding is still a placeholder —
see `hot.rs:42-48` for context). The `subs+b.vs+sxtw` triplet itself
matches JSC's macro byte-for-byte.

## Microbench

ns/dispatch on V8 v7 Sub-heavy workload: **<TBD ns>** (7-sample
median, isolated). LLInt op_sub: **<TBD ns>** for context. Ratio: <X>×.

## Slow-path-share on V8 v7

| Workload     | Sub dispatches | Slow-path-share |
|--------------|---------------:|----------------:|
| Richards     |                |                 |
| DeltaBlue    |                |                 |
| Crypto       |                |                 |
| RayTrace     |                |                 |
| NavierStokes |                |                 |
| Splay        |                |                 |

Threshold per spec §5 + epic spec §1 criterion 6: < 20% per workload.

## Behavioral tests

- `cargo test -p lyng-vm -p lyng-tests` passes.
- Test262 `language/expressions/subtraction` slice: <pass count>/<total>.
````

Fill in the captured numbers from Steps 8-10. Commit the report.

- [ ] **Step 12: Update `hot-opcodes.toml` budget for Sub**

Open `tools/lyng-bench/hot-opcodes.toml`. Find the `[[opcodes]]` block for `name = "Sub"`. Update:

```toml
[[opcodes]]
name = "Sub"
target_slow_path_share = 0.20
# DSL-1 Phase 1.C.1: inline DSL port landed. Budget = <measured + 2>.
# Captured from reports/lyng/dsl-asm-baseline-aarch64/op_sub.asm.
aarch64_max_instructions = <measured + 2>
x86_64_max_instructions = 0
```

Set `<measured + 2>` to the instruction count from the ported report (~28-32 expected; 2 headroom for LLVM rewrite noise).

- [ ] **Step 13: Commit the op_sub port**

```bash
git add crates/vm/src/dsl/handlers/cold.rs \
        reports/lyng/dsl-handlers/op_sub.md \
        reports/lyng/dsl-asm-baseline-aarch64/op_sub.asm \
        tools/lyng-bench/hot-opcodes.toml
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.C.1 Task 2: op_sub inline port

First port of Phase 1.C, mechanical mirror of the op_add SMI shape
established in DSL-0. The 7-step inline fast path uses pre-existing
sub_smi_overflow! / record_smi/store_reg macros; no new substrate.

Per-opcode gates per spec §5:
- Asm shape: <X> instructions (within 5 of LLInt's op_sub)
- Microbench: <Y> ns/dispatch (Z× LLInt op_sub)
- Slow-path-share: <W>% max across V8 v7 workloads
- Behavioral parity: cargo test -p lyng-vm -p lyng-tests pass
- Test262 subtraction slice unchanged

hot-opcodes.toml budget for Sub calibrated to measured + 2 headroom.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

## Task 3: Port op_mul inline fast path

**Files:**
- Modify: `crates/vm/src/dsl/handlers/cold.rs` — replace `op_mul_dsl` body (around line 1120) and add `op_mul_record_smi_rs` shim
- Create: `reports/lyng/dsl-handlers/op_mul.md`
- Create: `reports/lyng/dsl-asm-baseline-aarch64/op_mul.asm`
- Modify: `tools/lyng-bench/hot-opcodes.toml` — set `aarch64_max_instructions` for `Mul`

- [ ] **Step 1: Read the current cold-stub**

Open `crates/vm/src/dsl/handlers/cold.rs` and locate `op_mul_dsl` (around line 1120). Current body:

```rust
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_mul_dsl, opcode_byte = 35, layout = AbcSlot, length = 6, |a, b, c, slot| {
        call_slow!(op_mul_slow_rs, args = [a, b, c, slot]);
        dispatch_after_slow!();
    }
}
```

- [ ] **Step 2: Verify macro imports at the top of cold.rs**

Confirm `mul_smi_overflow` is in the `use crate::{...}` block. Add if missing.

- [ ] **Step 3: Replace the `op_mul_dsl` body with the inline fast path**

Replace the cold-stub `llint_handler!` block with:

```rust
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_mul_dsl, opcode_byte = 35, layout = AbcSlot, length = 6, |a, b, c, slot| {
        load_reg!(b => t0);
        check_smi!(t0, .slow);
        load_reg!(c => t1);
        check_smi!(t1, .slow);
        untag_smi!(t0);
        untag_smi!(t1);
        mul_smi_overflow!(t0, t1 => t2, .slow);
        tag_smi!(t2);
        store_reg!(a, t2);
        call_slow!(op_mul_record_smi_rs, args = [slot]);
        dispatch_after_slow!();
        .slow:
        call_slow!(op_mul_slow_rs, args = [a, b, c, slot]);
        dispatch_after_slow!();
    }
}
```

Note: `mul_smi_overflow!` uses `smull + sxtw + cmp + b.ne` (4 instructions) instead of `adds + b.vs + sxtw` (3 instructions). One extra instruction vs op_sub.

- [ ] **Step 4: Add the `op_mul_record_smi_rs` shim**

Add directly after the `op_mul_dsl` `llint_handler!` block:

```rust
/// Fast-path feedback-recording shim for `op_mul`. Mirrors
/// `op_add_record_smi_rs` in hot.rs: bumps the warmup counter,
/// allocates the legacy vector at threshold, mirrors legacy state to
/// the flat array, observes the tier feedback event.
#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_mul_record_smi_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
    feedback_slot: u32,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    {
        let inner = dispatch.dispatch_state();
        let code = inner.code();
        inner
            .vm
            .record_feedback_slot(code, lyng_types::FeedbackSlotId::from_raw(feedback_slot));
    }
    dispatch.translate_outcome(crate::dsl::slow_path::SemanticOutcome::Continue {
        pc_advance: 6,
    })
}
```

- [ ] **Step 5: Build verify**

```
cargo build --release -p lyng-vm
```
Expected: clean compile.

- [ ] **Step 6: Run behavioral tests**

```
cargo test --release -p lyng-vm -p lyng-tests
```
Expected: 418 + 1209 tests pass.

- [ ] **Step 7: Run a focused Test262 slice**

```
cargo run --release -p lyng-tests -- --test-source test262 --filter language/expressions/multiplication
```
Expected: same pass rate as before.

- [ ] **Step 8: Capture asm baseline manually**

```bash
cargo rustc --release -p lyng-vm -- --emit=asm 2>/dev/null
ASM_FILE=$(ls -t target/release/deps/lyng_vm-*.s 2>/dev/null | head -1)
awk '/^_op_mul_dsl:/,/^[[:space:]]*\.cfi_endproc/' "$ASM_FILE" > reports/lyng/dsl-asm-baseline-aarch64/op_mul.asm
```

Confirm the file shows the inline path (smull + sxtw + cmp + b.ne for overflow check).

- [ ] **Step 9: Run microbench**

```
cargo run --release -p lyng-bench -- microbench --opcodes Mul --require-isolation
```
Record ns/dispatch with confidence interval.

- [ ] **Step 10: Run slow-path-share isolated V8 v7 sweep (CRITICAL for op_mul)**

```
cargo run --release -p lyng-bench -- v8suite --require-isolation --count-slow-path-share --opcodes Mul
```

**This is the gating measurement for Phase 1.C.1.** Per spec §8 risk row 1: op_mul slow-path-share on float-heavy workloads (RayTrace, NavierStokes) is the largest unknown.

Capture per-workload share. If any workload exceeds 20%:
- Run `lyng-bench v8suite --workload <name> --count-slow-path-share --opcodes Mul,Add` to get LLInt-on-same-workload share as the baseline.
- If LLInt also exceeds 20% on that workload, document a per-opcode waiver in the ported report (the threshold is about our fast-path matching LLInt's, not absolute share).
- If LLInt is well below 20% on that workload and ours exceeds, something is wrong with our fast path; investigate before continuing.

- [ ] **Step 11: Write the ported report**

Create `reports/lyng/dsl-handlers/op_mul.md` using the same template as `op_sub.md` from Task 2 Step 11, adapted for op_mul:
- DSL source: the new inline fast path above.
- Slow-path shims: `op_mul_slow_rs` (unchanged) + `op_mul_record_smi_rs` (NEW).
- Asm shape table: note the 4-instruction overflow check (smull + sxtw + cmp + b.ne) vs op_sub's 3.
- LLInt reference: JSC's op_mul also uses `smull + sxtw + cmp` for overflow detection — should match closely.
- Microbench: captured ns/dispatch, ratio vs LLInt.
- **Slow-path-share table: include per-workload share with waiver justification for any workload exceeding 20%.**
- Behavioral tests: cargo test + Test262 multiplication slice.

- [ ] **Step 12: Update `hot-opcodes.toml` budget for Mul**

```toml
[[opcodes]]
name = "Mul"
target_slow_path_share = 0.20
# DSL-1 Phase 1.C.1: inline DSL port landed. Budget = <measured + 2>.
aarch64_max_instructions = <measured + 2>
x86_64_max_instructions = 0
```

If a per-workload waiver was documented in the ported report, also update `target_slow_path_share` to the calibrated threshold (e.g., 0.35) with a comment pointing at the report.

- [ ] **Step 13: Commit the op_mul port**

```bash
git add crates/vm/src/dsl/handlers/cold.rs \
        reports/lyng/dsl-handlers/op_mul.md \
        reports/lyng/dsl-asm-baseline-aarch64/op_mul.asm \
        tools/lyng-bench/hot-opcodes.toml
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.C.1 Task 3: op_mul inline port

Second port of Phase 1.C.1, top-30 rank #4 with 589M dispatches per
V8 v7 run. SMI fast path with overflow detection via smull + sxtw +
cmp + b.ne (4 instructions vs op_sub's 3).

Per-opcode gates per spec §5:
- Asm shape: <X> instructions
- Microbench: <Y> ns/dispatch (Z× LLInt op_mul)
- Slow-path-share: <details, including any float-heavy waivers>
- Behavioral parity + Test262 multiplication slice unchanged

The float-heavy V8 v7 workloads (RayTrace, NavierStokes) <do/do-not>
exceed the 20% slow-path-share threshold; <waiver|no-waiver> per
ported report.

hot-opcodes.toml budget for Mul calibrated to measured + 2 headroom.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

## Task 4: Phase 1.C.1 close — mini A/B + sub-phase summary

**Files:**
- Create: `reports/lyng/dsl-1/phase-1c1-ab-comparison.md`
- Create: `reports/lyng/dsl-1/phase-1c1-summary.md`

- [ ] **Step 1: Capture pre-1.C.1 HEAD**

The pre-1.C.1 HEAD is the commit at Phase 1.B close (`aa3ab9fc`) — or if Phase 1.C.0 substrate prep landed first, the commit immediately before Task 2. Capture it:

```bash
PRE_C1=$(git log --oneline | grep -E "DSL-1 Phase 1.C.0" | head -1 | awk '{print $1}')
[ -z "$PRE_C1" ] && PRE_C1="aa3ab9fc"
echo "$PRE_C1"
```

Record this commit hash for the A/B comparison.

- [ ] **Step 2: Run 11-sample mini A/B vs pre-1.C.1 HEAD**

Build both binaries:

```bash
# Save the current op_mul-landed binary
cp target/release/lyng-bench /tmp/lyng-bench-post-1c1

# Build the pre-1.C.1 binary
git worktree add /tmp/wt-pre-1c1 "$PRE_C1"
(cd /tmp/wt-pre-1c1 && cargo build --release -p lyng-bench 2>/dev/null)
cp /tmp/wt-pre-1c1/target/release/lyng-bench /tmp/lyng-bench-pre-1c1
git worktree remove /tmp/wt-pre-1c1
```

Run the A/B (11 samples per side, loadavg-overlap-checked):

```bash
cargo run --release -p lyng-bench -- ab \
  --baseline /tmp/lyng-bench-pre-1c1 \
  --candidate /tmp/lyng-bench-post-1c1 \
  --samples 11 \
  --require-isolation \
  --output reports/lyng/dsl-1/phase-1c1-ab-comparison.md
```

If the bench tool's `ab` subcommand requires different flags, adapt to the convention used in `reports/lyng/dsl-1/phase-1b3-ab-comparison.md`.

Verify loadavg overlap is < ±20% in the captured report. If exceeded, re-run.

- [ ] **Step 3: Write the sub-phase summary**

Create `reports/lyng/dsl-1/phase-1c1-summary.md`:

```markdown
# DSL-1 Phase 1.C.1 — Binary arith with overflow — Summary

**HEAD:** `<commit hash of op_mul port>`
**Predecessor:** Phase 1.C.0 substrate prep (or Phase 1.B close `aa3ab9fc` if 1.C.0 was absorbed).
**Sub-phase spec:** [`docs/superpowers/specs/2026-05-21-dsl-1-phase-1c-smi-arith-and-bitwise-design.md`](../../../docs/superpowers/specs/2026-05-21-dsl-1-phase-1c-smi-arith-and-bitwise-design.md) §2.1.C.1.

## What landed

Two inline ports:

| Opcode | Top-30 rank | Dispatches / V8 v7 run | Asm-shape instr | Slow-path-share max |
|--------|------------:|-----------------------:|----------------:|---------------------:|
| op_sub | #29         | 65M                    | <X>             | <Y>%                 |
| op_mul | #4          | 589M                   | <X>             | <Y>%                 |

Combined dispatch share added: 654M / V8 v7 run.

## A/B vs pre-1.C.1 HEAD

11-sample A/B, isolated, loadavg overlap < ±20%.

| Workload     | Pre-1.C.1 median | Post-1.C.1 median | Delta |
|--------------|-----------------:|------------------:|------:|
| <captured numbers>                                                  |
| **Geomean**  | —                | —                 | **<Z>%** |

(Informational; the phase-close cumulative A/B vs `d850f261` is the authoritative number.)

## Per-opcode reports

- [`reports/lyng/dsl-handlers/op_sub.md`](../dsl-handlers/op_sub.md)
- [`reports/lyng/dsl-handlers/op_mul.md`](../dsl-handlers/op_mul.md)

## Gates passed

- Per-opcode asm shape within 5 of LLInt
- Per-opcode microbench within 2× LLInt
- Per-opcode slow-path-share < 20% (or documented per-workload waiver — see op_mul report)
- Behavioral parity: 418 + 1209 cargo tests pass
- Test262 unchanged

## Followups

(Pin any per-opcode followups discovered during the sub-phase here; expand to `phase-1c-followups.md` at phase close if non-trivial.)
```

Fill in the captured numbers.

- [ ] **Step 4: Commit the sub-phase summary + A/B artifact**

```bash
git add reports/lyng/dsl-1/phase-1c1-summary.md reports/lyng/dsl-1/phase-1c1-ab-comparison.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.C.1: phase summary — binary arith with overflow

Sub-phase close after op_sub + op_mul inline ports. 11-sample
mini A/B vs pre-1.C.1 HEAD, loadavg-overlap-checked; per-opcode
gates satisfied; behavioral parity and Test262 unchanged.

Combined dispatch share added: 654M / V8 v7 run (Mul=589M + Sub=65M).

A/B is informational per Phase 1.B retrospective lesson #2; phase-close
cumulative A/B vs d850f261 is the authoritative umbrella gate.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

# Sub-phase 1.C.2 — Bitwise / shifts (op_bit_and, op_shift_left, op_shift_right)

Three ports, ~3 days. All use the no-overflow shape (no `b.vs` branch — the `*_smi!` macros always succeed). Order: op_bit_and (simplest), op_shift_left (similar), op_shift_right (largest dispatch share, exercises the shape on the highest-volume workload).

## Task 5: Port op_bit_and inline fast path

**Files:**
- Modify: `crates/vm/src/dsl/handlers/cold.rs` — replace `op_bit_and_dsl` body (around line 1435) and add `op_bit_and_record_smi_rs` shim
- Create: `reports/lyng/dsl-handlers/op_bit_and.md`
- Create: `reports/lyng/dsl-asm-baseline-aarch64/op_bit_and.asm`
- Modify: `tools/lyng-bench/hot-opcodes.toml` — set `aarch64_max_instructions` for `BitAnd`

- [ ] **Step 1: Verify macro imports**

In `cold.rs`'s `use crate::{...}` block, add `bit_and_smi` if missing.

- [ ] **Step 2: Replace the `op_bit_and_dsl` body**

Replace the cold-stub `llint_handler!` block at ~line 1435 with:

```rust
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_bit_and_dsl, opcode_byte = 44, layout = AbcSlot, length = 6, |a, b, c, slot| {
        load_reg!(b => t0);
        check_smi!(t0, .slow);
        load_reg!(c => t1);
        check_smi!(t1, .slow);
        untag_smi!(t0);
        untag_smi!(t1);
        bit_and_smi!(t0, t1 => t2);
        tag_smi!(t2);
        store_reg!(a, t2);
        call_slow!(op_bit_and_record_smi_rs, args = [slot]);
        dispatch_after_slow!();
        .slow:
        call_slow!(op_bit_and_slow_rs, args = [a, b, c, slot]);
        dispatch_after_slow!();
    }
}
```

Note: `bit_and_smi!` has no overflow branch (bitwise on tagged ints can't overflow); 1 less branch than op_sub.

- [ ] **Step 3: Add the `op_bit_and_record_smi_rs` shim**

Add directly after the `op_bit_and_dsl` block:

```rust
/// Fast-path feedback-recording shim for `op_bit_and`. Mirrors
/// `op_add_record_smi_rs` in hot.rs.
#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_bit_and_record_smi_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
    feedback_slot: u32,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    {
        let inner = dispatch.dispatch_state();
        let code = inner.code();
        inner
            .vm
            .record_feedback_slot(code, lyng_types::FeedbackSlotId::from_raw(feedback_slot));
    }
    dispatch.translate_outcome(crate::dsl::slow_path::SemanticOutcome::Continue {
        pc_advance: 6,
    })
}
```

- [ ] **Step 4: Build + behavioral tests**

```
cargo build --release -p lyng-vm
cargo test --release -p lyng-vm -p lyng-tests
cargo run --release -p lyng-tests -- --test-source test262 --filter language/expressions/bitwise-and
```
Expected: clean compile; 418 + 1209 tests pass; Test262 bitwise-and slice unchanged.

- [ ] **Step 5: Capture asm baseline manually**

```bash
cargo rustc --release -p lyng-vm -- --emit=asm 2>/dev/null
ASM_FILE=$(ls -t target/release/deps/lyng_vm-*.s 2>/dev/null | head -1)
awk '/^_op_bit_and_dsl:/,/^[[:space:]]*\.cfi_endproc/' "$ASM_FILE" > reports/lyng/dsl-asm-baseline-aarch64/op_bit_and.asm
```

- [ ] **Step 6: Microbench + slow-path-share**

```
cargo run --release -p lyng-bench -- microbench --opcodes BitAnd --require-isolation
cargo run --release -p lyng-bench -- v8suite --require-isolation --count-slow-path-share --opcodes BitAnd
```

- [ ] **Step 7: Write `reports/lyng/dsl-handlers/op_bit_and.md`**

Use the same template as `op_sub.md`. Note in the LLInt reference section that JSC's op_bitand also has no overflow branch. The asm shape should be ~3-4 instructions shorter than op_sub due to no `b.vs`.

- [ ] **Step 8: Update `hot-opcodes.toml` budget**

```toml
[[opcodes]]
name = "BitAnd"
target_slow_path_share = 0.20
# DSL-1 Phase 1.C.2: inline DSL port landed.
aarch64_max_instructions = <measured + 2>
x86_64_max_instructions = 0
```

- [ ] **Step 9: Commit**

```bash
git add crates/vm/src/dsl/handlers/cold.rs \
        reports/lyng/dsl-handlers/op_bit_and.md \
        reports/lyng/dsl-asm-baseline-aarch64/op_bit_and.asm \
        tools/lyng-bench/hot-opcodes.toml
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.C.2 Task 5: op_bit_and inline port

Top-30 rank #24, 98M dispatches per V8 v7 run. Bitwise no-overflow
SMI fast path using existing bit_and_smi! macro from DSL-0 substrate.
Mechanical port; ~3 instructions shorter than op_sub due to no b.vs
overflow branch.

Per-opcode gates per spec §5 satisfied: asm shape <X> instr, microbench
<Y> ns/dispatch (Z× LLInt), slow-path-share <W>% max across V8 v7
workloads.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

## Task 6: Port op_shift_left inline fast path

**Files:**
- Modify: `crates/vm/src/dsl/handlers/cold.rs` — replace `op_shift_left_dsl` body (around line 1539) and add `op_shift_left_record_smi_rs` shim
- Create: `reports/lyng/dsl-handlers/op_shift_left.md`
- Create: `reports/lyng/dsl-asm-baseline-aarch64/op_shift_left.asm`
- Modify: `tools/lyng-bench/hot-opcodes.toml` — set `aarch64_max_instructions` for `ShiftLeft`

- [ ] **Step 1: Verify macro imports**

In `cold.rs`'s `use crate::{...}` block, add `shift_left_smi` if missing.

- [ ] **Step 2: Replace the `op_shift_left_dsl` body**

Replace the cold-stub `llint_handler!` block at ~line 1539 with:

```rust
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_shift_left_dsl, opcode_byte = 47, layout = AbcSlot, length = 6, |a, b, c, slot| {
        load_reg!(b => t0);
        check_smi!(t0, .slow);
        load_reg!(c => t1);
        check_smi!(t1, .slow);
        untag_smi!(t0);
        untag_smi!(t1);
        shift_left_smi!(t0, t1 => t2);
        tag_smi!(t2);
        store_reg!(a, t2);
        call_slow!(op_shift_left_record_smi_rs, args = [slot]);
        dispatch_after_slow!();
        .slow:
        call_slow!(op_shift_left_slow_rs, args = [a, b, c, slot]);
        dispatch_after_slow!();
    }
}
```

`shift_left_smi!` masks rhs to its low 5 bits per ECMAScript `<<` semantics (3 instructions: and + lsl + sxtw).

- [ ] **Step 3: Add the `op_shift_left_record_smi_rs` shim**

```rust
/// Fast-path feedback-recording shim for `op_shift_left`.
#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_shift_left_record_smi_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
    feedback_slot: u32,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    {
        let inner = dispatch.dispatch_state();
        let code = inner.code();
        inner
            .vm
            .record_feedback_slot(code, lyng_types::FeedbackSlotId::from_raw(feedback_slot));
    }
    dispatch.translate_outcome(crate::dsl::slow_path::SemanticOutcome::Continue {
        pc_advance: 6,
    })
}
```

- [ ] **Step 4: Build + behavioral tests**

```
cargo build --release -p lyng-vm
cargo test --release -p lyng-vm -p lyng-tests
cargo run --release -p lyng-tests -- --test-source test262 --filter language/expressions/left-shift
```

- [ ] **Step 5: Capture asm baseline**

```bash
cargo rustc --release -p lyng-vm -- --emit=asm 2>/dev/null
ASM_FILE=$(ls -t target/release/deps/lyng_vm-*.s 2>/dev/null | head -1)
awk '/^_op_shift_left_dsl:/,/^[[:space:]]*\.cfi_endproc/' "$ASM_FILE" > reports/lyng/dsl-asm-baseline-aarch64/op_shift_left.asm
```

- [ ] **Step 6: Microbench + slow-path-share**

```
cargo run --release -p lyng-bench -- microbench --opcodes ShiftLeft --require-isolation
cargo run --release -p lyng-bench -- v8suite --require-isolation --count-slow-path-share --opcodes ShiftLeft
```

- [ ] **Step 7: Write `reports/lyng/dsl-handlers/op_shift_left.md`**

Use the same template as `op_bit_and.md`. Note the rhs-mask step (low 5 bits) per ECMAScript `<<` semantics.

- [ ] **Step 8: Update `hot-opcodes.toml`**

```toml
[[opcodes]]
name = "ShiftLeft"
target_slow_path_share = 0.20
# DSL-1 Phase 1.C.2: inline DSL port landed.
aarch64_max_instructions = <measured + 2>
x86_64_max_instructions = 0
```

- [ ] **Step 9: Commit**

```bash
git add crates/vm/src/dsl/handlers/cold.rs \
        reports/lyng/dsl-handlers/op_shift_left.md \
        reports/lyng/dsl-asm-baseline-aarch64/op_shift_left.asm \
        tools/lyng-bench/hot-opcodes.toml
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.C.2 Task 6: op_shift_left inline port

Top-30 rank #25, 89M dispatches per V8 v7 run. Bitwise no-overflow SMI
fast path using existing shift_left_smi! macro (and+lsl+sxtw with
5-bit rhs mask per ECMAScript << semantics).

Per-opcode gates per spec §5 satisfied: asm shape <X> instr, microbench
<Y> ns/dispatch, slow-path-share <W>% max across V8 v7 workloads.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

## Task 7: Port op_shift_right inline fast path

**Files:**
- Modify: `crates/vm/src/dsl/handlers/cold.rs` — replace `op_shift_right_dsl` body (around line 1574) and add `op_shift_right_record_smi_rs` shim
- Create: `reports/lyng/dsl-handlers/op_shift_right.md`
- Create: `reports/lyng/dsl-asm-baseline-aarch64/op_shift_right.asm`
- Modify: `tools/lyng-bench/hot-opcodes.toml` — set `aarch64_max_instructions` for `ShiftRight`

- [ ] **Step 1: Verify macro imports**

In `cold.rs`'s `use crate::{...}` block, add `shift_right_smi` if missing.

- [ ] **Step 2: Replace the `op_shift_right_dsl` body**

Replace the cold-stub `llint_handler!` block at ~line 1574 with:

```rust
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_shift_right_dsl, opcode_byte = 48, layout = AbcSlot, length = 6, |a, b, c, slot| {
        load_reg!(b => t0);
        check_smi!(t0, .slow);
        load_reg!(c => t1);
        check_smi!(t1, .slow);
        untag_smi!(t0);
        untag_smi!(t1);
        shift_right_smi!(t0, t1 => t2);
        tag_smi!(t2);
        store_reg!(a, t2);
        call_slow!(op_shift_right_record_smi_rs, args = [slot]);
        dispatch_after_slow!();
        .slow:
        call_slow!(op_shift_right_slow_rs, args = [a, b, c, slot]);
        dispatch_after_slow!();
    }
}
```

`shift_right_smi!` uses arithmetic right shift (asr) — sign-preserving — per ECMAScript `>>` semantics, with the same low-5-bits rhs mask.

- [ ] **Step 3: Add the `op_shift_right_record_smi_rs` shim**

```rust
/// Fast-path feedback-recording shim for `op_shift_right`.
#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_shift_right_record_smi_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
    feedback_slot: u32,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    {
        let inner = dispatch.dispatch_state();
        let code = inner.code();
        inner
            .vm
            .record_feedback_slot(code, lyng_types::FeedbackSlotId::from_raw(feedback_slot));
    }
    dispatch.translate_outcome(crate::dsl::slow_path::SemanticOutcome::Continue {
        pc_advance: 6,
    })
}
```

- [ ] **Step 4: Build + behavioral tests**

```
cargo build --release -p lyng-vm
cargo test --release -p lyng-vm -p lyng-tests
cargo run --release -p lyng-tests -- --test-source test262 --filter language/expressions/right-shift
```

- [ ] **Step 5: Capture asm baseline**

```bash
cargo rustc --release -p lyng-vm -- --emit=asm 2>/dev/null
ASM_FILE=$(ls -t target/release/deps/lyng_vm-*.s 2>/dev/null | head -1)
awk '/^_op_shift_right_dsl:/,/^[[:space:]]*\.cfi_endproc/' "$ASM_FILE" > reports/lyng/dsl-asm-baseline-aarch64/op_shift_right.asm
```

- [ ] **Step 6: Microbench + slow-path-share (CRITICAL — top-30 #10)**

```
cargo run --release -p lyng-bench -- microbench --opcodes ShiftRight --require-isolation
cargo run --release -p lyng-bench -- v8suite --require-isolation --count-slow-path-share --opcodes ShiftRight
```

This is the largest dispatch share in Phase 1.C.2 (266M / V8 v7 run). If slow-path-share is high on Crypto (the workload that exercises shifts heaviest), investigate before continuing.

- [ ] **Step 7: Write `reports/lyng/dsl-handlers/op_shift_right.md`**

Same template. Note this is the arithmetic (sign-preserving) right shift — distinct from `op_unsigned_shift_right` which uses `lsr` and has a uint32-can't-be-SMI bail-out.

- [ ] **Step 8: Update `hot-opcodes.toml`**

```toml
[[opcodes]]
name = "ShiftRight"
target_slow_path_share = 0.20
# DSL-1 Phase 1.C.2: inline DSL port landed.
aarch64_max_instructions = <measured + 2>
x86_64_max_instructions = 0
```

- [ ] **Step 9: Commit**

```bash
git add crates/vm/src/dsl/handlers/cold.rs \
        reports/lyng/dsl-handlers/op_shift_right.md \
        reports/lyng/dsl-asm-baseline-aarch64/op_shift_right.asm \
        tools/lyng-bench/hot-opcodes.toml
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.C.2 Task 7: op_shift_right inline port

Top-30 rank #10, 266M dispatches per V8 v7 run — the largest dispatch
share in Phase 1.C.2. Bitwise no-overflow SMI fast path using existing
shift_right_smi! macro (and+asr+sxtw — arithmetic sign-preserving right
shift per ECMAScript >> semantics).

Per-opcode gates per spec §5 satisfied: asm shape <X> instr, microbench
<Y> ns/dispatch, slow-path-share <W>% max across V8 v7 workloads
(notably Crypto where shifts are densest).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

## Task 8: Phase 1.C.2 close — mini A/B + sub-phase summary

**Files:**
- Create: `reports/lyng/dsl-1/phase-1c2-ab-comparison.md`
- Create: `reports/lyng/dsl-1/phase-1c2-summary.md`

- [ ] **Step 1: Capture pre-1.C.2 HEAD**

```bash
PRE_C2=$(git log --oneline | grep -E "DSL-1 Phase 1.C.1: phase summary" | head -1 | awk '{print $1}')
echo "$PRE_C2"
```

- [ ] **Step 2: Run 11-sample mini A/B vs pre-1.C.2 HEAD**

Build both binaries and run the A/B (mirror Task 4 Step 2):

```bash
cp target/release/lyng-bench /tmp/lyng-bench-post-1c2
git worktree add /tmp/wt-pre-1c2 "$PRE_C2"
(cd /tmp/wt-pre-1c2 && cargo build --release -p lyng-bench 2>/dev/null)
cp /tmp/wt-pre-1c2/target/release/lyng-bench /tmp/lyng-bench-pre-1c2
git worktree remove /tmp/wt-pre-1c2

cargo run --release -p lyng-bench -- ab \
  --baseline /tmp/lyng-bench-pre-1c2 \
  --candidate /tmp/lyng-bench-post-1c2 \
  --samples 11 \
  --require-isolation \
  --output reports/lyng/dsl-1/phase-1c2-ab-comparison.md
```

Verify loadavg overlap < ±20%.

- [ ] **Step 3: Write `reports/lyng/dsl-1/phase-1c2-summary.md`**

```markdown
# DSL-1 Phase 1.C.2 — Bitwise / shifts — Summary

**HEAD:** `<commit hash>`
**Predecessor:** Phase 1.C.1 close.

## What landed

Three inline ports:

| Opcode          | Top-30 rank | Dispatches / V8 v7 run | Asm-shape instr | Slow-path-share max |
|-----------------|------------:|-----------------------:|----------------:|---------------------:|
| op_bit_and      | #24         | 98M                    | <X>             | <Y>%                 |
| op_shift_left   | #25         | 89M                    | <X>             | <Y>%                 |
| op_shift_right  | #10         | 266M                   | <X>             | <Y>%                 |

Combined dispatch share added: 453M / V8 v7 run.

## A/B vs pre-1.C.2 HEAD

(captured numbers from Step 2)

## Per-opcode reports

- [`op_bit_and.md`](../dsl-handlers/op_bit_and.md)
- [`op_shift_left.md`](../dsl-handlers/op_shift_left.md)
- [`op_shift_right.md`](../dsl-handlers/op_shift_right.md)

## Gates passed

(Same checklist as 1.C.1 summary.)

## Followups

(Per-opcode followups discovered.)
```

- [ ] **Step 4: Commit**

```bash
git add reports/lyng/dsl-1/phase-1c2-summary.md reports/lyng/dsl-1/phase-1c2-ab-comparison.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.C.2: phase summary — bitwise / shifts

Sub-phase close after op_bit_and + op_shift_left + op_shift_right
inline ports. 11-sample mini A/B vs pre-1.C.2 HEAD, all per-opcode
gates passed.

Combined dispatch share added: 453M / V8 v7 run (ShiftRight=266M +
BitAnd=98M + ShiftLeft=89M).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

# Sub-phase 1.C.3 — Unary update (op_increment, op_decrement)

Two ports + 1 unit test, ~3 days. Uses the new `inc_smi_overflow!`/`dec_smi_overflow!` macros from 1.C.0. Includes the SMI-elision claim verification.

## Task 9: Port op_increment inline fast path

**Files:**
- Modify: `crates/vm/src/dsl/handlers/cold.rs` — replace `op_increment_dsl` body (around line 1678) and add `op_increment_record_smi_rs` shim
- Create: `reports/lyng/dsl-handlers/op_increment.md`
- Create: `reports/lyng/dsl-asm-baseline-aarch64/op_increment.asm`
- Modify: `tools/lyng-bench/hot-opcodes.toml` — set `aarch64_max_instructions` for `Increment`

- [ ] **Step 1: Re-confirm the SMI-elision claim by reading the semantic**

Open `crates/vm/src/dsl/handlers/cold.rs` and `crates/vm/src/vm/semantics/arithmetic.rs:796-833` side by side. Confirm:

1. `op_update_register_semantic` (the shared body for op_increment/op_decrement) writes `numeric` to `args.src` BEFORE writing `value` to `args.dst` (line 825).
2. For SMI src, `numeric` equals the original src value (the Vm helper `update_register_value` returns `(numeric=ToNumeric(src), value=numeric±1)`; ToNumeric on SMI is identity).
3. Therefore, writing `numeric` back to src for SMI src is observationally a no-op — the inline fast path can skip this step.

If the semantic has a side effect we missed (e.g., `record_feedback_slot` keyed on the src register specifically), the elision is unsafe. Document the conclusion in the ported report (op_increment.md), citing the file:line.

- [ ] **Step 2: Verify macro imports**

In `cold.rs`'s `use crate::{...}` block, add `inc_smi_overflow` if missing.

- [ ] **Step 3: Replace the `op_increment_dsl` body**

Replace the cold-stub `llint_handler!` block at ~line 1678 with:

```rust
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_increment_dsl, opcode_byte = 51, layout = AbcSlot, length = 6, |a, b, c, slot| {
        load_reg!(b => t0);
        check_smi!(t0, .slow);
        untag_smi!(t0);
        inc_smi_overflow!(t0 => t1, .slow);
        tag_smi!(t1);
        store_reg!(a, t1);
        // SMI fast-path elision: for SMI src, ToNumeric(src)==src so the
        // semantic's writeback of `numeric` to src (vm/semantics/arithmetic.rs:825)
        // is idempotent. Non-SMI src takes the slow path which still
        // performs the writeback.
        call_slow!(op_increment_record_smi_rs, args = [slot]);
        dispatch_after_slow!();
        .slow:
        call_slow!(op_increment_slow_rs, args = [a, b, c, slot]);
        dispatch_after_slow!();
    }
}
```

Note: only ONE `check_smi!` (src), one `untag_smi!`, one arithmetic op (`inc_smi_overflow!`), one `tag_smi!`, one `store_reg!`. The `c` operand is unused (decoded but not referenced) — matches the existing cold-stub.

- [ ] **Step 4: Add the `op_increment_record_smi_rs` shim**

```rust
/// Fast-path feedback-recording shim for `op_increment`. Mirrors
/// `op_add_record_smi_rs` in hot.rs.
#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_increment_record_smi_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
    feedback_slot: u32,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    {
        let inner = dispatch.dispatch_state();
        let code = inner.code();
        inner
            .vm
            .record_feedback_slot(code, lyng_types::FeedbackSlotId::from_raw(feedback_slot));
    }
    dispatch.translate_outcome(crate::dsl::slow_path::SemanticOutcome::Continue {
        pc_advance: 6,
    })
}
```

- [ ] **Step 5: Build + behavioral tests**

```
cargo build --release -p lyng-vm
cargo test --release -p lyng-vm -p lyng-tests
cargo run --release -p lyng-tests -- --test-source test262 --filter language/expressions/postfix-increment
cargo run --release -p lyng-tests -- --test-source test262 --filter language/expressions/prefix-increment
```
Expected: all pass.

- [ ] **Step 6: Capture asm baseline**

```bash
cargo rustc --release -p lyng-vm -- --emit=asm 2>/dev/null
ASM_FILE=$(ls -t target/release/deps/lyng_vm-*.s 2>/dev/null | head -1)
awk '/^_op_increment_dsl:/,/^[[:space:]]*\.cfi_endproc/' "$ASM_FILE" > reports/lyng/dsl-asm-baseline-aarch64/op_increment.asm
```

Verify the file shows the inline path with `adds w?, w?, #1` (immediate form) and `b.vs` overflow branch.

- [ ] **Step 7: Microbench + slow-path-share**

```
cargo run --release -p lyng-bench -- microbench --opcodes Increment --require-isolation
cargo run --release -p lyng-bench -- v8suite --require-isolation --count-slow-path-share --opcodes Increment
```

Top-30 #5, 541M dispatches — second-largest dispatch share in Phase 1.C. The slow-path-share matters a lot.

- [ ] **Step 8: Write `reports/lyng/dsl-handlers/op_increment.md`**

Use the same template as the prior ports, with an extra section documenting the SMI-elision claim:

```markdown
## SMI-elision of src register writeback

The semantic body `op_update_register_semantic` (at
[`crates/vm/src/vm/semantics/arithmetic.rs:796-833`](../../../crates/vm/src/vm/semantics/arithmetic.rs#L796-L833))
writes `numeric = ToNumeric(src)` back to the src register before
writing the post-update value to dst. For SMI src, `ToNumeric` is
identity (`Value::from_smi(s).as_smi() == Some(s)`), so the writeback
is observationally a no-op.

The inline fast path elides this writeback. The slow path (entered on
non-SMI src) still performs it via `op_increment_semantic`. The
[`dsl_increment_writeback`](../../../crates/tests/src/dsl_increment_writeback.rs)
unit test exercises a non-SMI src reaching the slow path and asserts
the writeback still happens.
```

- [ ] **Step 9: Update `hot-opcodes.toml`**

```toml
[[opcodes]]
name = "Increment"
target_slow_path_share = 0.20
# DSL-1 Phase 1.C.3: inline DSL port landed.
aarch64_max_instructions = <measured + 2>
x86_64_max_instructions = 0
```

- [ ] **Step 10: Commit**

```bash
git add crates/vm/src/dsl/handlers/cold.rs \
        reports/lyng/dsl-handlers/op_increment.md \
        reports/lyng/dsl-asm-baseline-aarch64/op_increment.asm \
        tools/lyng-bench/hot-opcodes.toml
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.C.3 Task 9: op_increment inline port

Top-30 rank #5, 541M dispatches per V8 v7 run. Unary SMI fast path
using the new inc_smi_overflow! macro from Phase 1.C.0 (adds wD, wS, #1
immediate form with overflow detection).

The fast path elides the src register writeback that the semantic
performs (vm/semantics/arithmetic.rs:825), because ToNumeric(SMI)
is identity. Non-SMI src bails to slow which still writes back. The
SMI-elision claim is verified by the dsl_increment_writeback unit
test (Task 11) and documented in the ported report.

Per-opcode gates per spec §5 satisfied: asm shape <X> instr, microbench
<Y> ns/dispatch, slow-path-share <W>% max across V8 v7 workloads.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

## Task 10: Port op_decrement inline fast path

**Files:**
- Modify: `crates/vm/src/dsl/handlers/cold.rs` — replace `op_decrement_dsl` body (around line 1712) and add `op_decrement_record_smi_rs` shim
- Create: `reports/lyng/dsl-handlers/op_decrement.md`
- Create: `reports/lyng/dsl-asm-baseline-aarch64/op_decrement.asm`
- Modify: `tools/lyng-bench/hot-opcodes.toml` — set `aarch64_max_instructions` for `Decrement`

- [ ] **Step 1: Verify macro imports**

In `cold.rs`'s `use crate::{...}` block, add `dec_smi_overflow` if missing.

- [ ] **Step 2: Replace the `op_decrement_dsl` body**

Replace the cold-stub `llint_handler!` block at ~line 1712 with:

```rust
#[cfg(target_arch = "aarch64")]
llint_handler! {
    op_decrement_dsl, opcode_byte = 52, layout = AbcSlot, length = 6, |a, b, c, slot| {
        load_reg!(b => t0);
        check_smi!(t0, .slow);
        untag_smi!(t0);
        dec_smi_overflow!(t0 => t1, .slow);
        tag_smi!(t1);
        store_reg!(a, t1);
        // SMI fast-path elision: see op_increment. ToNumeric(SMI)==SMI so
        // the semantic's writeback is idempotent for SMI src; non-SMI takes slow.
        call_slow!(op_decrement_record_smi_rs, args = [slot]);
        dispatch_after_slow!();
        .slow:
        call_slow!(op_decrement_slow_rs, args = [a, b, c, slot]);
        dispatch_after_slow!();
    }
}
```

- [ ] **Step 3: Add the `op_decrement_record_smi_rs` shim**

```rust
/// Fast-path feedback-recording shim for `op_decrement`. Mirrors
/// `op_add_record_smi_rs` in hot.rs.
#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
pub extern "C" fn op_decrement_record_smi_rs(
    state: *mut crate::dsl::llint_state::LlIntState,
    feedback_slot: u32,
) -> crate::dsl::slow_path::SlowPathReturn {
    let mut dispatch = unsafe { crate::dsl::slow_path::LlIntDispatchState::from_raw(state) };
    dispatch.sync_from_asm();
    {
        let inner = dispatch.dispatch_state();
        let code = inner.code();
        inner
            .vm
            .record_feedback_slot(code, lyng_types::FeedbackSlotId::from_raw(feedback_slot));
    }
    dispatch.translate_outcome(crate::dsl::slow_path::SemanticOutcome::Continue {
        pc_advance: 6,
    })
}
```

- [ ] **Step 4: Build + behavioral tests**

```
cargo build --release -p lyng-vm
cargo test --release -p lyng-vm -p lyng-tests
cargo run --release -p lyng-tests -- --test-source test262 --filter language/expressions/postfix-decrement
cargo run --release -p lyng-tests -- --test-source test262 --filter language/expressions/prefix-decrement
```

- [ ] **Step 5: Capture asm baseline**

```bash
cargo rustc --release -p lyng-vm -- --emit=asm 2>/dev/null
ASM_FILE=$(ls -t target/release/deps/lyng_vm-*.s 2>/dev/null | head -1)
awk '/^_op_decrement_dsl:/,/^[[:space:]]*\.cfi_endproc/' "$ASM_FILE" > reports/lyng/dsl-asm-baseline-aarch64/op_decrement.asm
```

Verify shows `subs w?, w?, #1` immediate form.

- [ ] **Step 6: Microbench + slow-path-share**

```
cargo run --release -p lyng-bench -- microbench --opcodes Decrement --require-isolation
cargo run --release -p lyng-bench -- v8suite --require-isolation --count-slow-path-share --opcodes Decrement
```

- [ ] **Step 7: Write `reports/lyng/dsl-handlers/op_decrement.md`**

Mirror op_increment.md exactly — same SMI-elision section, just `dec_smi_overflow!` and "decrement"/"-1" instead of inc.

- [ ] **Step 8: Update `hot-opcodes.toml`**

```toml
[[opcodes]]
name = "Decrement"
target_slow_path_share = 0.20
# DSL-1 Phase 1.C.3: inline DSL port landed.
aarch64_max_instructions = <measured + 2>
x86_64_max_instructions = 0
```

- [ ] **Step 9: Commit**

```bash
git add crates/vm/src/dsl/handlers/cold.rs \
        reports/lyng/dsl-handlers/op_decrement.md \
        reports/lyng/dsl-asm-baseline-aarch64/op_decrement.asm \
        tools/lyng-bench/hot-opcodes.toml
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.C.3 Task 10: op_decrement inline port

Top-30 rank #23, 99M dispatches per V8 v7 run. Unary SMI fast path
mirroring op_increment using the new dec_smi_overflow! macro (subs
wD, wS, #1 immediate form with overflow detection — overflow only
at i32::MIN).

Same SMI-elision of src writeback as op_increment; same unit test
(Task 11) verifies the slow-path writeback for non-SMI src.

Per-opcode gates per spec §5 satisfied: asm shape <X> instr, microbench
<Y> ns/dispatch, slow-path-share <W>% max across V8 v7 workloads.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

## Task 11: Unit test for inc/dec non-SMI-src writeback

**Files:**
- Create: `crates/tests/src/dsl_increment_writeback.rs`
- Modify: `crates/tests/src/lib.rs` — add `mod dsl_increment_writeback;` (if `lib.rs` uses module declarations) or update the relevant test-module index

- [ ] **Step 1: Locate the existing test crate's module index**

Open `crates/tests/src/lib.rs`. Look for how other dsl_* tests are registered (e.g., `mod dsl_validation_xyz;` lines). The new test file must follow the same registration pattern.

- [ ] **Step 2: Identify what JS expression compiles to op_increment with non-SMI src**

Run a quick disassembly check to find a minimal JS expression that emits `Increment` with a non-SMI input. Likely candidates:

```bash
cargo run --release -p lyng-vm --bin lyng-vm -- --disassemble -e 'let s = "1"; s++'
```

Look for `Increment r<N>, r<M>` in the disassembly. If the compiler does constant folding or peephole optimization that prevents this, try:

```bash
cargo run --release -p lyng-vm --bin lyng-vm -- --disassemble -e 'function f(x) { x++; return x; } f("1")'
```

The expression that emits Increment with a string-typed source is the test input.

If no JS expression in the current language surface compiles to `op_increment` with non-SMI src (e.g., the compiler always inserts `ToNumber` first), document this in the ported report's SMI-elision section as "no JS-level coverage" — the structural claim still holds but the test is deferred. Skip Steps 3-5 and add the gap to `phase-1c-followups.md`.

- [ ] **Step 3: Write the failing test**

Create `crates/tests/src/dsl_increment_writeback.rs`:

```rust
//! Verifies that `op_increment` and `op_decrement` still perform the
//! src-register writeback when the src is non-SMI (forces the slow path).
//!
//! The DSL inline fast path elides the writeback for SMI src because
//! `ToNumeric(SMI) == SMI` makes the write a no-op. This test exercises
//! the non-SMI case to confirm the writeback happens via the semantic
//! body's slow path. See `reports/lyng/dsl-handlers/op_increment.md`
//! § SMI-elision.

#[test]
fn increment_string_src_writes_coerced_numeric_back_to_src() {
    // Use the JS expression identified in Task 11 Step 2 that compiles
    // to op_increment with non-SMI src. Replace `<EXPR>` with the
    // actual expression; replace `<EXPECTED_SRC>` with the numeric value
    // ToNumber("1") = 1; replace `<EXPECTED_DST>` with the post-update
    // value (1 + 1 = 2 for increment).

    let result = lyng_vm::test_helpers::eval_expr(
        // Example shape — adjust to whatever compiles to op_increment with non-SMI src:
        r#"
        function f() {
            let s = "1";
            let r = s++;       // postfix: r = ToNumber(s), then s = r + 1
            return [s, r];
        }
        f()
        "#,
    );

    // Assert s was written back as the coerced numeric (1) and r holds it,
    // then s was post-incremented to 2.
    // Adjust assertions to match the actual expression used.
    assert_eq!(result.to_string(), "[2, 1]");
}

#[test]
fn decrement_string_src_writes_coerced_numeric_back_to_src() {
    let result = lyng_vm::test_helpers::eval_expr(
        r#"
        function f() {
            let s = "2";
            let r = s--;       // postfix: r = ToNumber(s), then s = r - 1
            return [s, r];
        }
        f()
        "#,
    );
    assert_eq!(result.to_string(), "[1, 2]");
}
```

If `lyng_vm::test_helpers::eval_expr` doesn't exist, look for the equivalent existing helper in `crates/tests/src/` (check how other dsl_* tests evaluate JS) and adapt the call.

- [ ] **Step 4: Run the test and verify it passes**

```
cargo test --release -p lyng-tests dsl_increment_writeback
```
Expected: both tests pass. The DSL fast path bails on non-SMI src, the slow path runs the full semantic, the writeback happens, the test passes.

If the test fails, the most likely cause is either:
- The expression doesn't actually emit op_increment with non-SMI src (peephole optimized it). Re-check Step 2.
- The DSL fast path is incorrectly handling non-SMI src (e.g., not branching to .slow). Inspect the captured asm baseline; the `check_smi!` macro must branch to `.slow` for non-SMI input. This would be a bug to fix before continuing.

- [ ] **Step 5: Commit the unit test**

```bash
git add crates/tests/src/dsl_increment_writeback.rs crates/tests/src/lib.rs
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.C.3 Task 11: inc/dec non-SMI src writeback test

Verifies the SMI-elision claim from op_increment.md / op_decrement.md
documentation. The inline fast paths elide the src register writeback
when src is SMI (ToNumeric(SMI) is identity); the slow path still
performs it for non-SMI src. This test forces the non-SMI path via a
string source and asserts the writeback happens.

Per Phase 1.B retrospective lesson #3, substrate macros need
runtime-dispatch verification immediately. inc_smi_overflow! and
dec_smi_overflow! got their verification from the op_increment /
op_decrement inline ports; this test additionally locks down the
slow-path writeback path for non-SMI src.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

## Task 12: Phase 1.C.3 close — mini A/B + sub-phase summary

**Files:**
- Create: `reports/lyng/dsl-1/phase-1c3-ab-comparison.md`
- Create: `reports/lyng/dsl-1/phase-1c3-summary.md`

- [ ] **Step 1: Capture pre-1.C.3 HEAD**

```bash
PRE_C3=$(git log --oneline | grep -E "DSL-1 Phase 1.C.2: phase summary" | head -1 | awk '{print $1}')
echo "$PRE_C3"
```

- [ ] **Step 2: Run 11-sample mini A/B vs pre-1.C.3 HEAD**

Mirror Task 4 Step 2 / Task 8 Step 2.

```bash
cp target/release/lyng-bench /tmp/lyng-bench-post-1c3
git worktree add /tmp/wt-pre-1c3 "$PRE_C3"
(cd /tmp/wt-pre-1c3 && cargo build --release -p lyng-bench 2>/dev/null)
cp /tmp/wt-pre-1c3/target/release/lyng-bench /tmp/lyng-bench-pre-1c3
git worktree remove /tmp/wt-pre-1c3

cargo run --release -p lyng-bench -- ab \
  --baseline /tmp/lyng-bench-pre-1c3 \
  --candidate /tmp/lyng-bench-post-1c3 \
  --samples 11 \
  --require-isolation \
  --output reports/lyng/dsl-1/phase-1c3-ab-comparison.md
```

- [ ] **Step 3: Write `reports/lyng/dsl-1/phase-1c3-summary.md`**

```markdown
# DSL-1 Phase 1.C.3 — Unary update (inc/dec) — Summary

**HEAD:** `<commit hash>`
**Predecessor:** Phase 1.C.2 close.

## What landed

Two inline ports + 2 new backend macros + 1 unit test:

| Opcode        | Top-30 rank | Dispatches / V8 v7 run | Asm-shape instr | Slow-path-share max |
|---------------|------------:|-----------------------:|----------------:|---------------------:|
| op_increment  | #5          | 541M                   | <X>             | <Y>%                 |
| op_decrement  | #23         | 99M                    | <X>             | <Y>%                 |

Combined dispatch share added: 640M / V8 v7 run.

New substrate: `inc_smi_overflow!`, `dec_smi_overflow!` (3 instr each;
12-bit immediate form of adds/subs — no scratch needed).

Substrate verification: handler dispatch path validated by Test262
postfix/prefix-increment and -decrement slices. Slow-path writeback
for non-SMI src validated by `dsl_increment_writeback` unit test.

## A/B vs pre-1.C.3 HEAD

(captured numbers)

## SMI-elision claim

Documented and verified per the design spec §2.1.C.3. Inline fast path
skips the src register writeback because `ToNumeric(SMI) == SMI` is
idempotent. Slow path (non-SMI src) still performs the writeback. See
[`op_increment.md`](../dsl-handlers/op_increment.md) § SMI-elision.

## Per-opcode reports

- [`op_increment.md`](../dsl-handlers/op_increment.md)
- [`op_decrement.md`](../dsl-handlers/op_decrement.md)

## Gates passed

(Same checklist plus new-substrate runtime verification.)
```

- [ ] **Step 4: Commit**

```bash
git add reports/lyng/dsl-1/phase-1c3-summary.md reports/lyng/dsl-1/phase-1c3-ab-comparison.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.C.3: phase summary — unary update (inc/dec)

Sub-phase close after op_increment + op_decrement inline ports +
inc/dec_smi_overflow! macros + dsl_increment_writeback unit test.

Combined dispatch share added: 640M / V8 v7 run (Increment=541M +
Decrement=99M).

New substrate macros runtime-verified via inline handler dispatch
(per Phase 1.B retrospective lesson #3); SMI-elision of src writeback
verified via slow-path unit test for non-SMI src.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

# Phase 1.C close

## Task 13: Phase-close cumulative A/B vs pre-DSL-0 d850f261

**Files:**
- Create: `reports/lyng/dsl-1/phase-1c-cumulative-ab.md`

- [ ] **Step 1: Build the pre-DSL-0 baseline binary**

```bash
git worktree add /tmp/wt-pre-dsl0 d850f261
(cd /tmp/wt-pre-dsl0 && cargo build --release -p lyng-bench 2>/dev/null)
cp /tmp/wt-pre-dsl0/target/release/lyng-bench /tmp/lyng-bench-pre-dsl0
git worktree remove /tmp/wt-pre-dsl0
```

- [ ] **Step 2: Use the current (post-1.C.3) binary as the candidate**

```bash
cargo build --release -p lyng-bench
cp target/release/lyng-bench /tmp/lyng-bench-post-1c
```

- [ ] **Step 3: Run 11-sample cumulative A/B**

```bash
cargo run --release -p lyng-bench -- ab \
  --baseline /tmp/lyng-bench-pre-dsl0 \
  --candidate /tmp/lyng-bench-post-1c \
  --samples 11 \
  --require-isolation \
  --output reports/lyng/dsl-1/phase-1c-cumulative-ab.md
```

Verify loadavg overlap < ±20%. If exceeded, re-run.

- [ ] **Step 4: Open the captured A/B report and confirm the umbrella gate**

Open `reports/lyng/dsl-1/phase-1c-cumulative-ab.md`. Confirm:
1. Cumulative geomean is positive vs `d850f261` (must be > +8.51%, Phase 1.B close).
2. All 6 V8 v7 workloads have positive or near-flat deltas.
3. If any workload has > 5% regression, that's an off-ramp consideration per spec §7 (epic spec §1 criterion 3).

The re-baselined target per spec §3 is **+13% to +16% cumulative**. Document the actual number against both the re-baselined target and the epic-spec ≥+35% target in Task 14.

- [ ] **Step 5: Commit the A/B artifact**

```bash
git add reports/lyng/dsl-1/phase-1c-cumulative-ab.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.C cumulative A/B vs pre-DSL-0 d850f261

11-sample, isolated, loadavg-overlap-checked direct A/B between
pre-DSL-0 baseline d850f261 and Phase 1.C close. This is the
authoritative umbrella gate per spec §6 + Phase 1.B retrospective
lesson #2 (per-sub-phase A/Bs compose roughly but not authoritatively).

Cumulative geomean: <X>% vs d850f261. <Above|below|in> the re-baselined
+13% to +16% range from spec §3. Epic-spec target was +35% (projected
from JSC LLInt scaling assuming Phase 1.A delivered +5%, which it
didn't); re-baselining documented in phase-1c-summary.md.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

## Task 14: Phase 1.C summary + followups

**Files:**
- Create: `reports/lyng/dsl-1/phase-1c-summary.md`
- Create: `reports/lyng/dsl-1/phase-1c-followups.md`

- [ ] **Step 1: Run the final Test262 check at phase HEAD**

```bash
cargo run --release -p lyng-tests -- --test-source test262 --summary
```
Expected: ≥ 49729 passing files (Phase 1.B baseline). Record the exact count for the summary.

- [ ] **Step 2: Write `reports/lyng/dsl-1/phase-1c-summary.md`**

```markdown
# DSL-1 Phase 1.C — SMI arithmetic + bitwise — Summary

**HEAD:** `<phase-close commit hash>`
**Predecessor:** Phase 1.B close (`aa3ab9fc`, +8.51% cumulative vs pre-DSL-0 `d850f261`).
**Spec:** [`docs/superpowers/specs/2026-05-21-dsl-1-phase-1c-smi-arith-and-bitwise-design.md`](../../../docs/superpowers/specs/2026-05-21-dsl-1-phase-1c-smi-arith-and-bitwise-design.md).
**Cumulative A/B artifact:** [`phase-1c-cumulative-ab.md`](phase-1c-cumulative-ab.md).

---

## What landed

Seven inline ports + 2 new backend macros + 1 unit test, across three sub-phases:

| Sub-phase | Opcode          | Top-30 rank | Dispatches / V8 v7 run |
|-----------|-----------------|------------:|-----------------------:|
| 1.C.1     | op_sub          | #29         | 65M                    |
| 1.C.1     | op_mul          | #4          | 589M                   |
| 1.C.2     | op_bit_and      | #24         | 98M                    |
| 1.C.2     | op_shift_left   | #25         | 89M                    |
| 1.C.2     | op_shift_right  | #10         | 266M                   |
| 1.C.3     | op_increment    | #5          | 541M                   |
| 1.C.3     | op_decrement    | #23         | 99M                    |
| **Total** | —               | —           | **1.75B**              |

New substrate (1.C.0 / 1.C.3): `inc_smi_overflow!`, `dec_smi_overflow!` (3 instr each).

---

## Cumulative metrics at HEAD

### V8 v7 cumulative vs pre-DSL-0 d850f261

(Pull the table from `phase-1c-cumulative-ab.md`.)

| Workload    | d850f261 median | Phase 1.C HEAD median | Delta |
|-------------|-----------------|----------------------|------:|
| Richards    |                 |                      |       |
| DeltaBlue   |                 |                      |       |
| Crypto      |                 |                      |       |
| RayTrace    |                 |                      |       |
| NavierStokes|                 |                      |       |
| Splay       |                 |                      |       |
| **Geomean** | —               | —                    | **<X>%** |

**Spec §3 re-baselined target was +13% to +16%.** Actual landed at +<X>%.

**Epic-spec §2 row 1.C target was ≥+35% absolute.** This target was projected from JSC LLInt-style scaling assuming Phase 1.A delivered ≥+5%; Phase 1.A actually delivered +1.7%. Phase 1.B closed at +8.51% vs its ≥+15% target. Phase 1.C closes at +<X>% vs the re-baselined target — see spec §3 for the re-baselining rationale and engine state §3 for prior commentary on this trajectory.

### Test262

<N> passing files at HEAD vs 49729 at Phase 1.B close.

### Inline-ported opcodes (cumulative across Phase 1.A + 1.B + 1.C)

**25 opcodes inline-ported** (7 in 1.A + 2 in 1.B.2 + 9 in 1.B.3 + 7 in 1.C). Of these, **24 are in the V8 v7 top-30 OR macro-shared symmetric pairs of top-30** (StoreLocal0 is a macro-shared pair but functionally unreachable per Phase 1.B.3 finding).

Per the DSL-1 epic spec §2 table: 25 of ~45 planned opcode ports done. Phases 1.D through 1.G land the remaining ~20.

---

## Substrate inventory delta

### LlIntState layout

Unchanged from Phase 1.B (72 bytes; no new fields).

### Backend macros added

- `inc_smi_overflow!` (3 instr, adds-immediate form, in arithmetic.rs)
- `dec_smi_overflow!` (3 instr, subs-immediate form, in arithmetic.rs)

### Per-opcode `op_xxx_record_smi_rs` shims added

Seven new shims in cold.rs, one per port, mirroring `op_add_record_smi_rs` from hot.rs. Each: ~17 lines. Pattern repeated for DRY-vs-coupling trade-off (each opcode's shim is self-contained; if op_add's encoded length ever changes, only its own shim's `pc_advance` literal needs updating).

(Potential followup: consolidate to a shared `op_record_smi_arith_6_rs` shim across all 8 binary/unary arith ports — see `phase-1c-followups.md`.)

---

## Methodological notes for Phase 1.D+

(Carry forward Phase 1.B's 5 lessons. Add any new lessons surfaced by Phase 1.C.)

Phase 1.C added:

- **Lesson #6 (candidate):** Per-opcode `op_xxx_record_smi_rs` shims are repetitive but cheaply local. Future phases with similar feedback-recording needs can either continue per-opcode shims or factor out a shared `op_record_smi_<layout>_rs` family — decide based on whether the layout/length varies.

---

## Next steps

Per DSL-1 epic spec §2:

| Phase | Scope | Estimate |
|-------|-------|---------:|
| **1.D** | Comparison + branch (op_greater_equal, op_less_equal, 5 jump opcodes — 7 total) | ~1 week |
| 1.E | Pointer-identity cells refactor | 3-4 weeks |
| 1.F | IC mode-byte refactor + 6 IC opcodes | 3 weeks |
| 1.G | Calls + tail-call (6 opcodes) | 1 week |

Phase 1.D is the natural next sub-phase — same mechanical-port shape as Phase 1.C but for comparison opcodes (op_greater_equal #20, op_less_equal #27 from top-30) and the 5 jump opcodes (currently cold-stub delegators with non-trivial branch-target logic in slow path).

LoadEnvSlot substrate sub-phase remains a deferred followup (Phase 1.B.3 origin).

---

## References

- DSL-1 epic spec: [`docs/superpowers/specs/2026-05-18-dsl-1-hot-opcode-rollout-design.md`](../../../docs/superpowers/specs/2026-05-18-dsl-1-hot-opcode-rollout-design.md)
- Phase 1.B umbrella summary: [`phase-1b-summary.md`](phase-1b-summary.md)
- Engine state at Phase 1.B close: [`reports/lyng/asm-dsl-engine-state-2026-05-21.md`](../asm-dsl-engine-state-2026-05-21.md)
- Phase 1.C sub-phase summaries: [`phase-1c1-summary.md`](phase-1c1-summary.md), [`phase-1c2-summary.md`](phase-1c2-summary.md), [`phase-1c3-summary.md`](phase-1c3-summary.md)
- Phase 1.C cumulative A/B: [`phase-1c-cumulative-ab.md`](phase-1c-cumulative-ab.md)
- Per-opcode ported reports: under [`reports/lyng/dsl-handlers/`](../dsl-handlers/)
- Followups: [`phase-1c-followups.md`](phase-1c-followups.md)
```

Fill in the captured numbers from Task 13.

- [ ] **Step 3: Write `reports/lyng/dsl-1/phase-1c-followups.md`**

Aggregate followups discovered during the phase. Template:

```markdown
# DSL-1 Phase 1.C — Followups

Items surfaced during Phase 1.C that don't block phase close but
warrant tracking. Pick up opportunistically or schedule into Phase 1.D
or later.

## Per-opcode follow-up notes

(Aggregate from each ported report's followups section, if any.)

## Substrate / tooling

- **Shared `op_record_smi_<layout>_rs` consolidation:** Seven per-opcode
  `op_xxx_record_smi_rs` shims in cold.rs + one in hot.rs (op_add) — all
  have identical bodies aside from the symbol name. Consolidate to a
  single shared shim (parameterized by `instruction_len`, or one per
  unique `instruction_len`/`layout` combination). Save ~120 LoC. Not
  blocking; safe to defer.
- **`asm-diff --check` namespace expansion** (carryover from Phase 1.B
  followups): extend bench tool to auto-discover `dsl::handlers::cold::*`
  symbols. Phase 1.C continued manual capture via `cargo rustc --emit=asm`
  + awk. Tracked at `phase-1b-followups.md`.
- **Microbench snippet coverage for inc/dec:** if Phase 1.C.0/1.C.3
  added new microbench snippets for Increment/Decrement, ensure they're
  in the `verify_opcodes_per_iter` test set. If not, document the gap
  here.

## SMI-elision: future opcode candidates

The SMI-elision-of-src-writeback pattern applied in op_increment /
op_decrement may apply to other unary opcodes (e.g., op_negate, op_bit_not)
that also call `vm.helper(...)` returning a `(numeric, value)` pair.
Audit when those opcodes come into scope (if they ever do — neither is
top-30).

## JS-level coverage gaps

(If Task 11 Step 2 found no JS expression that emits op_increment with
non-SMI src, document here.)
```

- [ ] **Step 4: Commit phase summary + followups**

```bash
git add reports/lyng/dsl-1/phase-1c-summary.md reports/lyng/dsl-1/phase-1c-followups.md
git commit -m "$(cat <<'EOF'
DSL-1 Phase 1.C: phase summary + followups + cumulative-A/B framing

Final umbrella summary for Phase 1.C close. 7 inline ports across 3
sub-phases (1.C.1 binary arith, 1.C.2 bitwise/shifts, 1.C.3 unary
inc/dec) added ~1.75B inlined dispatches per V8 v7 run on top of
Phase 1.B's 1.26B; combined Phase 1.A+1.B+1.C: 25 of ~45 planned
DSL-1 opcode ports done.

V8 v7 cumulative geomean vs pre-DSL-0 d850f261: <X>% (see
phase-1c-cumulative-ab.md). Lands <above|below|in> the spec §3
re-baselined +13% to +16% target. Epic-spec absolute +35% target was
projected from JSC LLInt scaling assuming Phase 1.A delivered +5%
(actual +1.7%); re-baselining honestly tracked in the summary.

Substrate inventory delta documented. Phase 1.D is the natural next
phase (comparison + branch opcodes); LoadEnvSlot substrate sub-phase
remains a deferred followup.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

## Task 15: Engine state snapshot refresh (optional but recommended)

**Files:**
- Create: `reports/lyng/asm-dsl-engine-state-<phase-close-date>.md`

- [ ] **Step 1: Copy the previous engine state snapshot as a starting point**

```bash
DATE=$(date +%Y-%m-%d)
cp reports/lyng/asm-dsl-engine-state-2026-05-21.md \
   reports/lyng/asm-dsl-engine-state-${DATE}.md
```

- [ ] **Step 2: Update the snapshot**

Edit `reports/lyng/asm-dsl-engine-state-${DATE}.md`. Update:

- Title date.
- HEAD commit to phase-1c-close hash.
- Cumulative V8 v7 number to the new value.
- Behavioral parity test counts (pull from final `cargo test` run).
- Test262 passing count.
- Section 2 (timeline): add a `### DSL-1 Phase 1.C` subsection with the same shape as the existing 1.A / 1.B entries.
- Section 3 (aggregate metrics): refresh the V8 v7 table and the inline-ported opcodes count (25 cumulative).
- Section 3 cumulative-trajectory table: add a Phase 1.C row.
- Section 4 (substrate inventory): note the 2 new arith macros and the per-opcode record_smi_rs shims.
- Section 5 (lessons): carry forward Phase 1.B's 5 + any Phase 1.C additions.
- Section 6 (next steps): reorient to Phase 1.D (or LoadEnvSlot if that's chosen first).
- Section 8 (references): add Phase 1.C spec / plan / summary / followups links.

- [ ] **Step 3: Commit the engine state snapshot**

```bash
git add reports/lyng/asm-dsl-engine-state-${DATE}.md
git commit -m "$(cat <<'EOF'
Add asm-DSL engine state-of-the-engine snapshot ($(date +%Y-%m-%d))

Refresh of the engine-wide status doc following Phase 1.C close.
Covers:

- Architecture summary (unchanged from 2026-05-21 snapshot)
- Updated timeline including DSL-1 Phase 1.C (1.C.1 binary arith,
  1.C.2 bitwise/shifts, 1.C.3 unary inc/dec — 7 inline ports)
- Aggregate metrics: <X>% V8 v7 cumulative vs pre-DSL-0, <N> Test262
  passing, 25 inline-ported opcodes, ~3.0B inlined dispatches/V8v7 run
- Substrate inventory delta (2 new arith macros, 7 new record_smi_rs
  shims)
- Updated cumulative-vs-epic-spec-target table noting Phase 1.C
  landed at <X>% vs the re-baselined +13% to +16% target
- Next-step recommendation: Phase 1.D (comparison + branch) vs
  LoadEnvSlot substrate sub-phase

Companion to the per-phase summaries under reports/lyng/dsl-1/.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Plan self-review

Cross-checking against spec §10 deliverables:

- ✅ 7 new inline DSL handler implementations — Tasks 2, 3, 5, 6, 7, 9, 10
- ✅ 2 new backend macros + ops.md entries — Task 1
- ✅ 7 ported reports — Tasks 2 Step 11, 3 Step 11, 5 Step 7, 6 Step 7, 7 Step 7, 9 Step 8, 10 Step 7
- ✅ 7 asm baselines — Steps 8, 8, 5, 5, 5, 6, 5 in respective port tasks
- ✅ 1 unit test for inc/dec writeback — Task 11
- ✅ Updated hot-opcodes.toml — Steps 12, 12, 8, 8, 8, 9, 8 in respective port tasks
- ✅ 3 sub-phase summaries — Tasks 4, 8, 12
- ✅ 1 phase summary + cumulative A/B + followups — Tasks 13, 14
- ✅ Updated engine state snapshot — Task 15 (optional)

Cross-checking against spec §5 per-opcode gates per port:

- ✅ Behavioral parity (cargo test) — Steps 6, 6, 4, 4, 4, 5, 4
- ✅ Asm shape (manual capture + report comparison) — captured in asm-baseline files + reported in ported report
- ✅ Microbench — Steps 9, 9, 6, 6, 6, 7, 6
- ✅ Slow-path-share — Steps 10, 10, 6, 6, 6, 7, 6
- ✅ Asm baseline updated — see above
- ✅ Ported report exists — see above
- ✅ hot-opcodes.toml budget calibrated — see above

Cross-checking against spec §6 A/B protocol:

- ✅ 11+ samples per phase-close cumulative A/B — Task 13 Step 3
- ✅ Loadavg overlap < ±20% — verified in Task 4 Step 2, Task 8 Step 2, Task 12 Step 2, Task 13 Step 3
- ✅ Per-sub-phase mini A/Bs as informational — Tasks 4, 8, 12
- ✅ Phase-close cumulative A/B vs d850f261 as authoritative — Task 13

Cross-checking against spec §3 re-baselining commentary:

- ✅ Phase 1.C summary documents both the re-baselined +13% to +16% target and the epic-spec ≥+35% target — Task 14 Step 2

No placeholder text remains. All steps have either exact code, exact commands, or self-contained instructions for adapting to a discovered situation (e.g., Task 11 Step 2 if no JS expression compiles to non-SMI op_increment).

Type-consistency note: the `op_xxx_record_smi_rs` shim signature is `(state: *mut LlIntState, feedback_slot: u32) -> SlowPathReturn`. This matches `op_add_record_smi_rs` from `hot.rs:90` exactly. The `call_slow!(op_xxx_record_smi_rs, args = [slot])` invocation passes a single u32 argument, consistent with the 1-arity arm of `call_slow!`.

---

## Execution handoff

Plan complete and saved to `docs/superpowers/plans/2026-05-21-dsl-1-phase-1c-smi-arith-and-bitwise.md`.

Two execution options:

1. **Subagent-Driven (recommended)** — coordinator dispatches a fresh subagent per task; reviewer between tasks; fast iteration. Matches the workflow proven through Phase 1.B (one worker subagent at a time; sequential gating).
2. **Inline Execution** — execute tasks in this session using executing-plans; batch execution with checkpoints for review.

For Phase 1.C, **subagent-driven is the right call**: 15 tasks, each scoped to ≤30 min of work, with clear per-task gates. The mechanical-port shape means workers can execute Tasks 2/3/5/6/7 in close succession with minimal coordinator review; Tasks 9/10/11 (the SMI-elision-of-writeback work) want a sequential reviewer pass before the unit test lands.
