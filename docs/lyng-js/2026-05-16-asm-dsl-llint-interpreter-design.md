# Design: asm-DSL LLInt-class interpreter (stable Rust, contained unsafe)

**Date:** 2026-05-16
**Status:** Design approved; ready for implementation planning.
**Supersedes (in spirit):** parts of [`reports/js/lyng-js/jsc-aligned-engine-roadmap.md`](../../reports/js/lyng-js/jsc-aligned-engine-roadmap.md) — specifically its Option α substrate commitment and Phase 1/Phase 3 acceptance criteria.
**Companion document:** [`reports/js/lyng-js/llint-parity-state-of-engine.md`](../../reports/js/lyng-js/llint-parity-state-of-engine.md) — the measurement-driven retrospective that motivated this design.

---

## 1. Context

The previous JSC-aligned roadmap (`lyng-49qk`) committed to Option α — a Rust `extern "C" fn` per-handler dispatch table with a central trampoline returning a `Step` enum. After landing Phase 1 through Phase 3f + Phase 4a-b, isolated bench numbers (`reports/js/lyng-js/external-engine-compare.md`) show lyng-js at:

- **5-12× slower than JSC LLInt** across V8 v7 (Richards 318 vs LLInt 1871).
- **1.6-3.7× slower than QuickJS** — the engine the original roadmap called "ceiling too low."

The retrospective established that α has a structural ~13× substrate overhead vs LLInt that no amount of IC layering can amortize. The substrate decision itself is the rate-limiting step.

This design specifies a new substrate path that targets **near-LLInt interpreter performance on stable Rust with contained unsafe.** It draws on JSC's LLInt as a reference architecture — its source is the gold-standard implementation of this design class — but lyng-js retains complete control over data layout, dispatch macros, and runtime decisions. No JSC code is vendored; we read JSC, understand the patterns, and re-implement them in our own Rust DSL.

The user's stance on unsafe: **anything stable Rust supports, if contained behind a clean macro layer.** The user's stance on time: **not a constraint** — the project has the resources to do this right.

## 2. Goals & non-goals

### Goals

- **Near-LLInt interpreter performance** on the V8 v7 suite — operationally defined as Richards ≥ 900 in isolated bench (within 2× of JSC LLInt's 1871).
- **100% Test262 conformance for Stage 3+ proposals, excluding intl402** — tracked as a parallel workstream that this design must not regress.
- **Stable Rust toolchain.** No nightly. `rustup default stable` builds the engine.
- **Contained unsafe.** Inline asm and `#[unsafe(naked)]` are acceptable when encapsulated by the DSL macro layer; handler authors never write `unsafe` directly.
- **Pure Rust build.** `cargo build` is the build. No Ruby, no Python, no external code generators. Standard Rust toolchain only.
- **Complete control over data layout.** Layout refactors are in scope when evidence supports them; we do not freeze sub-par designs because LLInt has different ones.

### Non-goals

- **JIT.** Out of scope for this design. The Baseline JIT (original Phases 5-6) remains valid in shape and stays deferred behind the interpreter milestone. We will not start JIT work until the interpreter substrate is complete and measured.
- **x86_64 support in the initial implementation.** Deferred until lyng-js has a concrete x86_64 user. AArch64 (Apple Silicon dev hardware) is the only initial target.
- **CI enforcement.** No CI infrastructure exists today; this design assumes developer-driven discipline (committed artifacts, manual baseline refresh). CI is a follow-up.
- **Adopting JSC's runtime data layout.** We read JSC as reference, not as a layout source-of-truth. Our `Value`, `Cell`, `Shape`, `FeedbackVector` remain our designs (subject to evidence-driven refactor during this work).

## 3. High-level approach

The dispatch substrate becomes a **Rust-native asm-DSL** that compiles handler bodies into `#[unsafe(naked)] extern "C" fn`s. Each handler's body is one `core::arch::asm!` block. The DSL is a proc-macro that parses handler source in offlineasm-flavored syntax (Flavor B; see §4) and emits per-arch inline assembly. State (PC, register-stack base, feedback-vector base, etc.) lives in pinned callee-saved registers across the entire interpreter — set up once at `Vm::run` entry, never spilled per dispatch. Dispatch is tail-jump (`br` / `jmp`), not call-return. The Step enum and the central trampoline both delete.

Handlers fall into two categories:

- **Hot opcodes (~25-30, by dispatch counter).** Full DSL bodies with fast paths inline. Match LLInt's handler shapes within a documented per-handler asm budget.
- **Cold opcodes (~120).** Three-line DSL stubs that delegate to Rust slow-path functions. Same dispatch shape, but the entire body is `call_slow!` + `dispatch_after_slow!`.

Both kinds use the same `llint_handler!` macro and the same dispatch table. The trampoline is gone; entry to the interpreter sets up pinned registers and tail-jumps to the first handler.

JSC's LLInt source (`Source/JavaScriptCore/llint/LowLevelInterpreter*.asm`) is the reference architecture: every ported handler has a side-by-side asm diff against JSC's matching handler. We deviate only with documented reason.

## 4. The DSL — Flavor B syntax

The DSL syntax is offlineasm-flavored — asm-shaped with Rust delimiters. Each statement maps to a small number of asm instructions. Labels and branches are explicit. No Rust control flow.

```rust
llint_handler! {
    op_add, layout = AbcSlot, length = 6, |a, b, c, slot| {
        load_reg!(b => t0);
        check_int32!(t0, .slow);
        load_reg!(c => t1);
        check_int32!(t1, .slow);
        add_int32_overflow!(t0, t1 => t2, .slow);
        store_reg!(a, t2);
        record_int32!(slot);
        dispatch!();

      .slow:
        call_slow!(op_add_slow_rs, args = [a, b, c, slot]);
        dispatch_after_slow!();
    }
}
```

Slow paths are normal Rust:

```rust
#[no_mangle]
pub extern "C" fn op_add_slow_rs(
    state: *mut DispatchState,
    dst: u32, lhs: u32, rhs: u32, slot: u32,
) -> SlowPathReturn {
    // Full Rust body — allocator, GC, Value coercion, etc.
}
```

Cold handlers are three lines:

```rust
llint_handler! {
    op_create_class, layout = AbcSlot, length = 6, |a, b, c, slot| {
        call_slow!(op_create_class_rs, args = [a, b, c, slot]);
        dispatch_after_slow!();
    }
}
```

**Why this syntax over Rust-idiomatic.** Direct mechanical translation from JSC's `LowLevelInterpreter64.asm` references. Authors can read JSC's matching handler in one window and write our DSL in another, line-by-line. The DSL's surface area matches the underlying machine, so there are no "I wrote Rust but the macro can't translate it" surprises. Onboarding cost is fixed (one register convention page, one operation vocabulary page) and amortizes across all hot opcodes.

## 5. Register-pin convention

State that's accessed per-dispatch lives in callee-saved registers, pinned across the entire interpreter from `Vm::run` entry through every handler invocation. The corresponding `DispatchState.frame.*` fields are stale snapshots synced only at slow-path boundaries.

### AArch64 mapping (initial target)

| Pin       | Register | Type                           | Refreshed when                                 |
| --------- | -------- | ------------------------------ | ---------------------------------------------- |
| `PC`      | x19      | `*const u8` (pointer, not offset) | call / return / exception unwind             |
| `REGS`    | x20      | `*mut Value`                   | call / return                                  |
| `FV`      | x21      | `*const FeedbackEntry`         | call / return                                  |
| `VM`      | x22      | `*mut Vm`                      | once per `Vm::run`                             |
| `TABLE`   | x23      | `*const Handler`               | once per `Vm::run`                             |
| `STATE`   | x24      | `*mut DispatchState`           | once per `Vm::run`; also passed as arg 0       |
| `t0..t6`  | x9–x15   | scratch (caller-saved)         | per-instruction                                |
| spare CSR | x25–x28  | available for handlers needing more pins | as needed                          |

### x86_64 mapping (deferred, but specified for forward-compat)

| Pin       | Register | Notes                                                                  |
| --------- | -------- | ---------------------------------------------------------------------- |
| `PC`      | r12      | callee-saved                                                           |
| `REGS`    | r13      | callee-saved                                                           |
| `FV`      | r14      | callee-saved                                                           |
| `VM`      | r15      | callee-saved                                                           |
| `STATE`   | rbx      | callee-saved                                                           |
| `TABLE`   | RIP-relative | no register pin (RIP-relative is free on x86_64)                   |
| scratch   | rax, rcx, rdx, rsi, r8, r9, r10, r11 | caller-saved (rsi, r8, r9 also serve as arg regs for `call_slow!`) |

### Two load-bearing choices

- **PC as pointer, not offset.** `PC` is `PB + offset` materialized once and updated by addition. Dispatch is `ldrb [PC]; advance; load handler; jump` — 4 instrs total. PB doesn't need a register pin; it's reconstructable from `STATE.installed.function.instruction_bytes` for rare cases (bounds checks at function boundaries).
- **`FV` as base pointer of a flat IC-entry array.** Requires refactoring today's `NamedPropertyFeedback` (multiple fields + sidecars from Phase 3a/3e/3f) into a contiguous flat-array layout. This refactor is in scope (§9).

### Refresh discipline (the load-bearing invariant)

| Pin     | Truth lives in     | When `STATE.frame.*` is synced |
| ------- | ------------------ | ------------------------------ |
| `PC`    | x19 register       | before every `call_slow!`      |
| `REGS`  | x20 register       | before every `call_slow!`      |
| `FV`    | x21 register       | before every `call_slow!`      |

After a slow-path call, the bridge reloads pinned registers from `STATE.frame.*` if the slow path signaled `Refresh` (call/return/exception). This is exactly LLInt's protocol — `storePC` / `loadPC` macros around `cCall*`.

## 6. Slow-path bridge protocol

A slow path is a normal Rust `extern "C"` function — no `unsafe(naked)`, no inline asm. Standard signature:

```rust
extern "C" fn op_xxx_slow_rs(
    state: *mut DispatchState,   // a0 / rdi
    operand_0: u32,              // a1 / rsi
    operand_1: u32,              // a2 / rdx
    // ... up to operand_4 (a5 / r9)
) -> SlowPathReturn;
```

Operands pass as `u32` (not `u8`/`u16`) to avoid extension dance in ABI registers.

### Return ABI

```rust
#[repr(C)]
pub struct SlowPathReturn {
    pub tag: u64,      // SlowPathTag, returned in a0 / rax
    pub payload: u64,  // returned in a1 / rdx
}

#[repr(u64)]
pub enum SlowPathTag {
    Continue = 0,   // payload = new PC offset (u32-in-u64). Dispatch at new PC.
    Refresh  = 1,   // payload = unused. Reload PC/REGS/FV from STATE.frame, dispatch.
    Done     = 2,   // payload = encoded Value. Exit Vm::run with Ok(value).
}
```

Three tags. Exceptions don't get their own tag — the slow path handles unwinding (transferring to the catch handler, updating `STATE.frame`), then returns `Refresh`.

### Bridge protocol (AArch64 sketch)

```asm
; Pre-call: sync PC into state.frame.instruction_offset
ldr  x9,  [x24, #STATE_PB_OFFSET]
sub  x10, x19, x9                  ; PC pointer → offset
str  w10, [x24, #STATE_FRAME_PC_OFFSET]

; Move pinned STATE into arg 0, operands into a1..aN
mov  x0,  x24                      ; STATE → a0
mov  w1,  <operand a>
mov  w2,  <operand b>
; ...

; Call
bl   op_xxx_slow_rs

; Post-call dispatch (Continue is the common case)
cbnz x0,  .unusual                 ; tag != Continue
ldr  x9,  [x24, #STATE_PB_OFFSET]
add  x19, x9,  x1                  ; PC = PB + new_offset
ldrb w8,  [x19]
ldr  x10, [x23, x8, lsl #3]
br   x10                           ; tail-dispatch

.unusual:
cmp  x0,  #2
b.eq .done
; Refresh: reload everything from STATE.frame
ldr  x19, [x24, #STATE_FRAME_PC_PTR]
ldr  x20, [x24, #STATE_FRAME_REGS]
ldr  x21, [x24, #STATE_FRAME_FV]
ldrb w8,  [x19]
ldr  x10, [x23, x8, lsl #3]
br   x10

.done:
; Exit interpreter; payload (x1) is encoded Value
b    _interpreter_exit
```

Common-case bridge cost (Continue): **~10-15 instructions** including pre-call sync. Matches LLInt's `callSlowPath` + `dispatch()`.

### Frame transitions

`op_call_*` and `op_return` slow paths always return `Refresh` because they alter frames. Other slow paths return `Continue` (PC advanced) or `Done` (uncaught exception, script complete).

Exception unwinding folds into `Refresh`: slow path detects throw → calls existing `Vm::transfer_to_exception_handler` → updates `STATE.frame` to catch handler → returns `Refresh`. If exception escapes the script, `Done` with error-encoded Value.

### GC safepoints

GC runs **only at `call_slow!` boundaries**, never inside a handler body. Inside a handler, scratch registers may hold raw cell pointers, partial Values, intermediate computations — not roots. At a slow-path call, all live Values are in `REGS` (the register stack), reachable from `STATE.frame.registers()`. This is LLInt's safepoint model.

Implication: scratch registers carrying Values must not survive across `call_slow!`. This is true by construction (caller-saved registers are clobbered by the call).

### Optimization to surface in DSL-0

`op_return` Refresh overhead vs Continue: 5 extra instructions × ~1-2% dispatch share = ~25-100k extra instructions per workload. May be visible. If so: add a "fast return to same code unit" shortcut. Decide based on DSL-0 measurement.

## 7. Per-arch backend structure

### Crate and module layout

```
crates/lyng-js-vm-dsl/                  -- proc-macro crate, new
├── Cargo.toml                          -- proc-macro = true
└── src/
    ├── lib.rs                          -- llint_handler! entry point
    ├── parse.rs                        -- syn-based body parser
    ├── layouts.rs                      -- operand-layout descriptors
    └── lower.rs                        -- AST → asm string assembly

crates/lyng-js/vm/src/dsl/              -- runtime-side support
├── mod.rs                              -- re-exports llint_handler!, declares backend
├── reg_convention.rs                   -- pinned-register docs + entry-point setup
├── slow_path.rs                        -- SlowPathReturn, SlowPathTag, bridge ABI
├── ops/
│   ├── mod.rs                          -- DSL vocabulary (re-exports per-arch)
│   └── ops.md                          -- vocabulary documentation
└── backend/
    ├── mod.rs                          -- cfg-dispatches to aarch64
    └── aarch64/
        ├── mod.rs                      -- exports all op macros
        ├── prelude.rs                  -- shared constants (NOT_CELL_MASK, INT32_TAG, ...)
        ├── operands.rs                 -- load_reg!, store_reg!, ...
        ├── values.rs                   -- check_int32!, untag_int32!, tag_*, ...
        ├── cells.rs                    -- load_cell_shape!, load_inline_slot!, ...
        ├── arithmetic.rs               -- add_int32_overflow!, ...
        ├── control.rs                  -- dispatch!, call_slow!, branch_*, ...
        └── feedback.rs                 -- load_feedback_site!, value_profile!, ...
```

x86_64 directory added when DSL-3 activates.

### Macro implementation: proc-macro + macro_rules!

Two layers:

- **Top level (`llint_handler!`):** a proc-macro using `syn`. Parses the handler signature and body into structured AST. Generates operand-decoding prologue from layout. Walks body statements, asks per-operation macros for their asm fragments, concatenates into one `core::arch::asm!` block. Emits `#[unsafe(naked)] extern "C" fn`.
- **Per-operation macros:** `macro_rules!` per arch, gated by `#[cfg(target_arch = ...)]`. Produce asm string fragments via `concat!`.

The proc-macro internally has a compile-time scratch register allocator (~200 lines): allocates `t0..t6` to operand variables in declaration order, allocates additional scratch as operations request, errors if scratch demand exceeds per-arch budget.

### DSL operation vocabulary (~40 operations)

Grouped by category:

- **Operand decoding** (auto-generated by `layout =`): `decode_abc!`, `decode_abc_slot!`, `decode_abx!`, `decode_ax!`, `decode_call_range!`, `decode_accumulator!`.
- **Register-file access:** `load_reg!`, `store_reg!`, `load_acc!`, `store_acc!`.
- **Value tag checks/tag manipulation:** `check_int32!`, `check_cell!`, `check_undefined!`, `check_double!`, `untag_int32!`, `untag_cell_ptr!`, `tag_int32!`, `tag_cell_ptr!`, `tag_undefined!`.
- **Cell field access:** `load_cell_shape!`, `load_cell_inline_slot!`, `load_cell_outline_slots!`, `load_outline_slot!`.
- **Feedback site access:** `load_feedback_site!`, `load_site_field!` (typed accessor).
- **Arithmetic (SMI fast paths):** `add_int32_overflow!`, `sub_int32_overflow!`, `mul_int32_overflow!`, bitwise (`bit_and_int32!`, `bit_or_int32!`, `bit_xor_int32!`), shifts.
- **Branching:** `branch_eq!`, `branch_ne!`, `branch_zero!`, `branch_nonzero!`.
- **Dispatch:** `dispatch!()` (auto-advance), `dispatch!(advance = N)`, `dispatch!(jump_to = expr)`.
- **Slow-path bridge:** `call_slow!`, `dispatch_after_slow!`.
- **Feedback recording:** `record_int32!`, `record_double!`, `record_cell!`, `record_string!`, `value_profile!`.

### Escape hatch

```rust
raw_asm!("/* arch-specific asm */")  // requires #[cfg(target_arch)] siblings per arch
```

Each use requires a justification comment. Discipline: on the third occurrence of a `raw_asm!` pattern, promote it to a DSL macro in `backend/<arch>/`.

### Per-arch divergence policy

- **DSL surface is identical.** Every operation exists on every supported arch with the same name and semantics. Arch-specific behavior goes in `raw_asm!`, not in different operation sets.
- **Asm shape is informational.** A `check_int32!` may emit 1 instruction on AArch64 and 2 on x86_64. Per-arch baselines are committed separately. Behavioral tests cover both.

## 8. Build pipeline

The build is `cargo build`. No external tools, no code generators, no Ruby. The proc-macro expands at compile time; naked functions emit inline asm; rustc + LLVM assemble. The infrastructure that matters is *around* the build.

### `cargo asm` automation

A new subcommand of the existing `lyng-js-bench` tool:

```sh
cargo run --release -p lyng-js-bench -- asm-diff \
  --opcodes-config tools/lyng-js-bench/hot-opcodes.toml \
  --baseline       reports/js/lyng-js/dsl-asm-baseline-aarch64/ \
  --output         /tmp/asm-current/ \
  --mode           check    # or `update`
```

Reads the hot-opcode list, invokes `cargo asm` per opcode, normalizes (strip CFI directives, file/line comments, rename labels to positional aliases, collapse jump-table padding), diffs against the committed baseline. Per-opcode instruction-count budgets in the config file.

### Asm baselines: in-repo

```
reports/js/lyng-js/dsl-asm-baseline-aarch64/
├── op_add.asm
├── op_move.asm
├── op_get_named_property.asm
└── ...
```

Small text files (0.5-5 KB each). Diffable in git. Reviewers read them in change reviews. The single most important artifact for "more science-based" — every claim about asm shape becomes a file the reader can check.

### LLInt reference capture (one-shot tool)

```sh
cargo run --release -p lyng-js-bench -- capture-llint \
  --jsc /System/Library/Frameworks/JavaScriptCore.framework/.../jsc \
  --opcodes op_get_by_id,op_put_by_id,op_add,op_mov,op_jmp,op_call,op_ret \
  --output  reports/js/lyng-js/llint-reference/
```

Invokes `otool -tvV` (macOS) / `objdump -d` (Linux) on the JSC binary. Finds `_llint_op_*` symbols. Dumps asm into markdown files. Re-captured when JSC ships a major version. Reference material; not gated.

### Per-handler ported reports

For each DSL handler we author:

```
reports/js/lyng-js/dsl-handlers/
├── op_add.md
├── op_get_named_property.md
└── ...
```

Each contains:

- DSL source (excerpt with link).
- Current asm output (both arches when applicable).
- LLInt reference asm.
- Side-by-side annotated diff: which instructions match, which don't, why.
- Microbench results (ns/dispatch).
- Behavioral test references.

These are the audit artifact per handler. They replace today's `phase-N-*.asm` files.

### Microbench harness

```sh
cargo run --release -p lyng-js-bench -- microbench \
  --opcodes-config tools/lyng-js-bench/hot-opcodes.toml \
  --baseline       reports/js/lyng-js/microbench-baseline.md \
  --samples        7 \
  --iters          5000000 \
  --require-isolation
```

Each opcode has a hot-loop test case (a JS function compiled to bytecode exercising that opcode). 7-sample median ns/dispatch. `--require-isolation` checks loadavg before starting; aborts if > 2.0.

### Verification cadence (no CI)

Discipline is developer-driven, artifact-based. Per change:

1. Build with `cargo build --release`.
2. Run `cargo run -p lyng-js-bench -- asm-diff --check`.
3. If a hot opcode's asm changed, run `cargo run -p lyng-js-bench -- microbench` for the affected opcodes.
4. If touching IC / property / arithmetic: run isolated V8 v7 sweep.
5. Run `cargo test -p lyng-js-vm -p lyng-js-tests` for behavioral coverage.
6. Run focused Test262 slice; whole-corpus for substantive changes.
7. Commit asm baselines + ported reports + handler source + bench reports together.

Manual baseline refresh: when an asm change is unrelated (rustc upgrade shifts labels), `--mode update` regenerates baselines; commit message includes `[asm-baseline-refresh: <reason>]` so the change is visible in history.

CI is a follow-up. When it lands, it will automate steps 1-2 and 5-6 on every push; step 3-4 stays on developer's bench machine (or a dedicated bench runner) due to isolation requirements.

### Build dependencies

| Dependency               | Required for             | Notes                                          |
| ------------------------ | ------------------------ | ---------------------------------------------- |
| Rust stable toolchain    | Everything                | rustup                                          |
| `cargo asm` crate        | asm-diff subcommand       | Cargo.toml dep                                  |
| `otool` / `objdump`      | LLInt capture (one-shot)  | system tools, dev-only                          |
| `mach_absolute_time` / `perf_event_open` | Microbench timing | OS API                                |
| **No Ruby, no Python, no offlineasm, no external code generators.** | | |

## 9. Data-layout refactors in scope

Sub-par data layouts are not frozen. The dispatch refactor is the natural moment to fix layouts that have been wrong but weren't worth touching in isolation. Each refactor is its own ticket with its own evidence requirement (asm shows the problem; refactor solves it; microbench confirms). No batched "do all layout work at once" phase.

Expected surfaces (we'll learn more during DSL-0):

| Refactor                                                          | Surfaces during        | Estimated effort | Motivation                                                          |
| ----------------------------------------------------------------- | ---------------------- | ----------------:| ------------------------------------------------------------------- |
| `FeedbackVector` flat-array layout                                | DSL-0 (`op_add`)       | ~1 week          | Required for the `FV` register pin to work                          |
| `Value` tag layout verification                                    | Pre-DSL-0              | ~2-3 days        | Verify masks (NOT_CELL_MASK, NUMBER_TAG, OTHER_TAG) match expectations |
| IC packed-handler representation (mode-byte + flat blocks)        | DSL-1 (`op_get_named_property`) | ~1-2 weeks | Collapses Phase 3a/3e/3f layered fast paths into LLInt-style mode dispatch |
| Pointer-identity cells (`ObjectRef = u32` → `*mut Cell`)          | DSL-1 (IC opcodes)     | ~3-4 weeks       | Eliminates side-table indirection; one fewer load per cell access   |
| `Cell` 8-byte header layout (JSC-equivalent)                       | DSL-1 (if needed)      | ~2-3 weeks       | If asm-diff shows our cell access is multi-instruction where LLInt is one |
| `Shape` transition representation                                  | DSL-1 (if hot)         | TBD              | Audit during porting; refactor only if evidence supports             |

The pre-work item (verifying `Value` tag layout matches the claims in the original roadmap) is **non-negotiable for DSL-0.** If our tags don't match what the SMI check / cell check / undefined check expect, the DSL operations need different asm than the LLInt reference — and we should know that before we start.

## 10. Phasing & exit criteria

Five phases. AArch64-only throughout. Single-dev cadence. Developer-driven discipline (no CI gates).

### R-0: Tooling (~2-3 weeks)

The measurement infrastructure the original roadmap needed and lacked.

**Deliverables:** `lyng-js-bench microbench`, `lyng-js-bench asm-diff`, `lyng-js-bench capture-llint`. LLInt reference files in `reports/js/lyng-js/llint-reference/`. Microbench baseline in `reports/js/lyng-js/microbench-baseline.md`. `tools/lyng-js-bench/hot-opcodes.toml` with the top-30 by dispatch count.

**Exit criteria:** all three subcommands work end-to-end and produce deterministic reports across 5 consecutive runs. LLInt reference files captured. Microbench baseline numbers committed.

**No DSL work during R-0.** Tooling first.

### DSL-0: Spike (~4-6 weeks)

The single experiment that decides whether the asm-DSL path is viable.

**Scope:**

- `lyng-js-vm-dsl` proc-macro crate created.
- `vm/src/dsl/backend/aarch64/` populated with the ~15 operations the spike needs.
- Register-pin convention frozen; `Vm::run` entry sets up pinned registers.
- Slow-path bridge ABI implemented and verified.
- **Four opcodes ported: `op_move`, `op_add`, `op_jump`, `op_return`.**
- Per-handler ported reports under `reports/js/lyng-js/dsl-handlers/`.
- Asm baselines under `reports/js/lyng-js/dsl-asm-baseline-aarch64/`.
- Pre-spike and post-spike microbench + V8 v7 runs captured as `phase-DSL-0-bench.md`.

**Exit criteria (all four, no waivers):**

1. **Asm shape vs LLInt.** Each ported handler's main hit path is **within 5 instructions** of JSC's matching handler, documented in the per-handler ported report.
2. **Microbench vs LLInt-equivalent.** Each ported opcode's ns/dispatch is **within 2× of JSC LLInt's matching opcode** measured the same way.
3. **Behavioral parity.** `cargo test -p lyng-js-vm` passes; Test262 baseline preserved (49710-49729 range, no new failures).
4. **V8 v7 directional check.** Richards score moves by **≥ +30%** vs pre-spike baseline.

If any of the four fail and the cause isn't fixable in ~3 days of investigation, **DSL-0 is a clean abort.** Rip out the four ported handlers, revert to today's α substrate, pivot to γ-hard (`#[unsafe(naked)]` on top of α handlers as a less ambitious fallback). The six weeks produce concrete evidence for the pivot decision.

If all four pass, DSL-0 closes with a written decision-document committing to DSL-1.

### DSL-1: Hot-opcode rollout (~8-10 weeks)

Port the remaining ~26 hot opcodes.

**Per-opcode workflow (~1-3 days each):**

1. Read JSC's matching handler in `LowLevelInterpreter64.asm`; understand semantics.
2. Identify any data-layout refactor surfaced. If yes, that's its own ticket, done first.
3. Write the DSL handler. Add new DSL operations to `backend/aarch64/` if needed (with `ops.md` entry).
4. Run `cargo asm` on the handler; inspect; iterate until shape is right.
5. Run microbench; capture into ported report.
6. Run isolated V8 v7 sweep; capture into ported report.
7. Write `reports/js/lyng-js/dsl-handlers/op_xxx.md` with side-by-side LLInt diff + data.
8. Commit asm baseline + ported report + handler source as one cohesive change.

**Port order (priority by dispatch share + risk):**

- Week 1-2: `op_load_*` family, `op_star_0..7` — trivial mechanical ports.
- Week 3-4: `op_load_local_*`, `op_store_local_*`, `op_ldar`.
- Week 5-6: `op_sub`, `op_mul`, `op_bit_*`, remaining SMI arithmetic.
- Week 7: `op_jump_if_*`, comparison ops.
- Week 8-9: `op_get_named_property`, `op_set_named_property`, `op_load_global`, `op_store_global`, `op_get_keyed_property`, `op_set_keyed_property` — IC opcodes. **The IC mode-byte refactor lands during this stretch** (replaces today's Phase 3a/3e/3f layered fast paths).
- Week 10: `op_call_0..3`, `op_call`, `op_return`, `op_tail_call` — frame-transitioning opcodes.

**Exit criteria:**

1. All 30 hot opcodes have DSL implementations with committed ported reports.
2. Cumulative geomean on V8 v7 ≥ +80% over the pre-DSL-0 baseline (Richards ≥ ~570 against today's 318).
3. No workload regressed > 5% vs pre-DSL-0 baseline.
4. Test262 baseline preserved.
5. Every per-handler asm-diff report exists.

**Off-ramp during DSL-1:** if 5+ consecutive handlers fail the within-5-of-LLInt criterion without documented reason, pause. The DSL might have a structural problem invisible in DSL-0. Cheaper to discover at handler 10 than 25.

### DSL-2: Cold-opcode wrap (~4-6 weeks)

The remaining ~120 opcodes get DSL stubs that delegate to Rust. Mostly mechanical (~15-30 min per opcode once pattern is established).

**Exit criteria:**

1. All 152 opcodes go through `llint_handler!`.
2. Legacy α dispatch path (`run_trampoline_uncounted`, `Step` enum, `dispatch_handlers/`) deleted.
3. Test262 baseline preserved.
4. V8 v7 geomean does not regress vs end of DSL-1.
5. Code-size diff committed (expected: codebase shrinks).

### DSL-3: x86_64 backend (deferred)

Not in initial scope. Activates when:

- lyng-js has a concrete x86_64 user / deployment, **or**
- A contributor wants to work on x86_64 specifically.

When activated: ~6 weeks of porting `backend/aarch64/` to `backend/x86_64/`. DSL surface is identical; only the per-arch macros change. Hot handlers don't need rewriting — they recompile against the new backend.

### DSL-4: Data-layout refactors (interleaved, not scheduled)

Each refactor is its own ticket, fired when DSL-1 surfaces evidence for it. See §9 for the expected surfaces.

### Test262 100% (parallel workstream)

Orthogonal to the DSL work. Spec compliance is about *semantics* (does `Array.prototype.at` handle every spec edge case?), not dispatch shape. DSL changes don't break Test262 if slow paths preserve semantics. Pace this stream to your appetite; it gets its own brainstorm if needed.

### Total interpreter-substrate timeline

| Phase                                 | Duration       | Cumulative     |
| ------------------------------------- | -------------- | -------------- |
| R-0 (tooling)                         | 2-3 weeks      | 3 wk           |
| DSL-0 (spike, 4 opcodes)              | 4-6 weeks      | 9 wk           |
| **Decision point — abort or scale up** |               |                |
| DSL-1 (26 more hot opcodes)           | 8-10 weeks     | 19 wk          |
| DSL-2 (cold-opcode wrap)              | 4-6 weeks      | 25 wk          |
| DSL-4 (data-layout, interleaved)      | (in DSL-1)     | (overlapping)  |
| **Total: ~6 months single-dev**       |                |                |
| DSL-3 (x86_64) deferred                | +6 weeks when activated | n/a   |
| Test262 100% (parallel)                | ~3-6 months    | n/a            |

## 11. Risks

| Risk                                                                   | Likelihood | Impact | Mitigation                                                                                            |
| ---------------------------------------------------------------------- | ---------- | ------ | ----------------------------------------------------------------------------------------------------- |
| `#[unsafe(naked)]` ergonomics tighter than expected (e.g., LLVM codegen quirks) | medium | high | DSL-0 spike validates this on real opcodes before scaling. If unsolvable, pivot to γ-hard.            |
| Slow-path bridge performance bottlenecks (megamorphic IC, exception-heavy code) | medium | medium | Microbench megamorphic workloads in DSL-0. If slow-path frequency dominates, inline more IC states in DSL. |
| GC interaction with naked handlers                                      | low-medium | high   | GC safepoints only at `call_slow!` boundaries — same as LLInt. Documented invariant; testable.        |
| x86_64 register pressure when DSL-3 activates                           | medium     | medium | Verify x86_64 pin assignment in DSL-3 before porting handlers. Plan B: don't pin `META`/`FV`, load from STATE per IC. |
| DSL becoming a maintenance black hole (feature creep)                  | medium     | high   | DSL surface deliberately small (~40 ops). Reject feature creep. Adds require: third occurrence of pattern + documentation. |
| Per-arch divergence in handler behavior                                | low        | high   | Behavioral tests cover both arches. Per-arch asm baselines acknowledge shape differences but behavior is one source of truth. |
| Rust toolchain bugs in `#[unsafe(naked)]` / `core::arch::asm!`         | low        | medium | Asm baselines catch codegen changes immediately. Can pin known-good rustc if needed.                  |
| Recruiting / onboarding cost for DSL fluency                           | medium     | low    | DSL is small. Cold opcodes stay simple stubs. Documentation + JSC reference files lower the bar.      |
| Data-layout refactors stall on GC integration (specifically pointer-identity cells) | medium | medium | Each refactor is its own ticket with own evidence requirement. Refactor only when DSL-1 surfaces hot need. |
| Microbench / V8 v7 measurement noise on dev machine                    | high       | low    | `--require-isolation` enforces loadavg < 2.0. Document quiescing steps. Accept higher variance on multi-purpose machines. |

## 12. Open questions to revisit during DSL-0

These don't block the design but should be answered with data, not speculation:

1. **`op_return` Refresh vs fast-return.** Worth optimizing in DSL-0 or defer? Decide based on `op_return` microbench post-DSL-0.
2. **Sixth pinned register.** Do we need a `META` / metadata-table pin separate from `FV`? Decide if a hot handler in DSL-1 demands it.
3. **DISPATCH_TABLE on x86_64.** RIP-relative is free; confirm with x86_64 backend implementation in DSL-3.
4. **Pointer-identity cells in DSL-1 or DSL-4.** Refactor pre-DSL-1 (front-load) or interleave with DSL-1 IC port (evidence-driven)?
5. **Cold-opcode wrapping overhead.** ~8 instructions per cold dispatch. Acceptable for opcodes at <0.1% dispatch share; quantify in DSL-2.

## 13. References

- **Companion retrospective:** [`reports/js/lyng-js/llint-parity-state-of-engine.md`](../../reports/js/lyng-js/llint-parity-state-of-engine.md) — the measurement-driven analysis of why the original roadmap missed.
- **Original roadmap (superseded in scope):** [`reports/js/lyng-js/jsc-aligned-engine-roadmap.md`](../../reports/js/lyng-js/jsc-aligned-engine-roadmap.md) — Phase 1-6 plan. The interpreter-track substrate decision (Option α) is reversed by this design; the Baseline JIT track (Phases 5-6) remains valid in shape, deferred behind this work.
- **JSC LLInt source (reference, not vendored):** `/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/`
  - `LowLevelInterpreter.asm` — dispatch macros, wrapper macros (`llintOp*`).
  - `LowLevelInterpreter64.asm` — 64-bit handler bodies including `performGetByIDHelper`.
  - `offlineasm/` — Ruby DSL compiler (architectural reference for our DSL's design).
- **JSC IC metadata reference:** `Source/JavaScriptCore/bytecode/GetByIdMetadata.h` — the `GetByIdMode` enum design our `op_get_named_property` mode-byte refactor mirrors.
- **Our current dispatch substrate (to be replaced):**
  - [`crates/lyng-js/vm/src/vm/dispatch_state.rs`](../../crates/lyng-js/vm/src/vm/dispatch_state.rs)
  - [`crates/lyng-js/vm/src/vm/dispatch_handlers/`](../../crates/lyng-js/vm/src/vm/dispatch_handlers/)
  - [`crates/lyng-js/vm/src/vm/dispatch/`](../../crates/lyng-js/vm/src/vm/dispatch/)
  - [`crates/lyng-js/vm/src/vm/feedback.rs`](../../crates/lyng-js/vm/src/vm/feedback.rs)
- **Project standards:**
  - [`AGENTS.md`](../../AGENTS.md) — repo-level guide.
  - [`crates/lyng-js/AGENTS.md`](../../crates/lyng-js/AGENTS.md) — Lyng JS operating guide.
  - [`docs/lyng-js/engineering-standards.md`](engineering-standards.md) — code quality bar.
  - [`docs/lyng-js/performance-workflow.md`](performance-workflow.md) — existing perf measurement conventions.
