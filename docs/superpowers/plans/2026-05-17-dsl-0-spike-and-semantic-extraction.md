# DSL-0: Spike, Semantic Extraction, and Full Opcode Coverage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the asm-DSL substrate behind every opcode — semantic-extracted alpha handlers (DSL-0a), DSL proc-macro + AArch64 backend + 5 hot + 5 warm + ~140 cold ports (DSL-0b), alpha trampoline deletion with single-implementation invariant verified (DSL-0c) — meeting the seven exit criteria in §10 of [docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md](../lyng/2026-05-16-asm-dsl-llint-interpreter-design.md).

**Architecture:** Three sequential sub-phases. **DSL-0a** extracts every opcode's semantic body out of its α `extern "C"` handler into a free `op_xxx_semantic` function reachable through a transitional `LlIntDispatchState` wrapper that aliases today's `DispatchState`. **DSL-0b** lands the `lyng-vm-dsl` proc-macro crate, the `vm/src/dsl/` runtime support (`LlIntState`, `LlIntRustContext`, slow-path bridge, entry/exit shims, FeedbackVector flat-array refactor, AArch64 backend ops), 9 validation cases, 5 hot ports (`op_move`, `op_add`, `op_jump`, `op_return`, `op_loop_header`), 5 warm ports (`op_jump8`, `op_jump_if_true(/8)`, `op_jump_if_false(/8)`, `op_wide`, `op_extra_wide`), and the ~140 cold-stub bulk generation. **DSL-0c** switches active dispatch to the DSL table, deletes the alpha trampoline / `Step` / `DispatchState` / tier-accounting machinery, and verifies single-implementation invariants via the structural manifest.

**Tech Stack:** Rust stable ≥ 1.88 (needed for `naked_asm!`), `syn` 2.x + `proc-macro2` + `quote` (proc-macro crate), `core::arch::naked_asm!` + `#[unsafe(naked)]` (AArch64 backend), existing `lyng-bench` subcommands `microbench` / `asm-diff` / `v8suite` (R-0 deliverables). Target arch is AArch64 only; x86_64 is deferred per design §10 DSL-2.

---

## Pre-flight check

Verify R-0 is landed and the worktree is ready before starting Task A1.

- [ ] **Pre-flight 1: Confirm worktree branch and clean tree**

  Run:
  ```sh
  git status -sb
  git rev-parse --abbrev-ref HEAD
  ```
  Expected: branch `claude/epic-saha-8f0b96` (or a fresh branch off it); working tree clean.

- [ ] **Pre-flight 2: Confirm R-0 deliverables exist**

  Run:
  ```sh
  ls reports/lyng/llint-dsl-value-layout.md \
     reports/lyng/llint-dsl-abi.md \
     reports/lyng/llint-dsl-safepoints.md \
     reports/lyng/dsl-asm-baseline-aarch64/NORMALIZATION.md \
     reports/lyng/microbench-baseline.md \
     reports/lyng/r0/status.md \
     tools/lyng-bench/hot-opcodes.toml
  ```
  Expected: every file exists. If any are missing, R-0 is incomplete — stop and finish R-0 before starting DSL-0.

- [ ] **Pre-flight 3: Confirm Rust toolchain ≥ 1.88**

  Run:
  ```sh
  rustc --version
  ```
  Expected: `rustc 1.88.x` or later. `naked_asm!` is stable as of 1.88; earlier versions will fail at the DSL-0b proc-macro step.

  If older: `rustup update stable && rustup default stable`.

- [ ] **Pre-flight 4: Confirm test baseline**

  Run:
  ```sh
  cargo test -p lyng-vm -p lyng-bytecode -p lyng-objects -p lyng-tests -p lyng-compiler 2>&1 | tail -30
  ```
  Expected: all tests pass. Record the total pass count — DSL-0a must not regress it.

- [ ] **Pre-flight 5: Confirm clean release build**

  Run:
  ```sh
  cargo build --release -p lyng-vm -p lyng-bench
  ```
  Expected: clean build, no warnings. Note compile time as a baseline for the proc-macro crate's impact later.

- [ ] **Pre-flight 6: Capture pre-DSL-0 baseline benches**

  These become the regression-reference for DSL-0b and DSL-0c exit gates.

  ```sh
  cargo run --release -p lyng-bench -- v8suite --report /tmp/pre-dsl-0-v8.md --json /tmp/pre-dsl-0-v8.json
  cargo run --release -p lyng-bench -- microbench --report /tmp/pre-dsl-0-microbench.md --samples 7 --iters 5000000 || echo "WARN: microbench needs isolated machine — re-run if loadavg gate fails"
  cp /tmp/pre-dsl-0-v8.md reports/lyng/pre-dsl-0-v8-baseline.md
  cp /tmp/pre-dsl-0-v8.json reports/lyng/pre-dsl-0-v8-baseline.json
  cp /tmp/pre-dsl-0-microbench.md reports/lyng/pre-dsl-0-microbench-baseline.md 2>/dev/null || true
  git add reports/lyng/pre-dsl-0-*
  git commit -m "DSL-0 pre-flight: capture pre-DSL-0 V8 v7 + microbench baselines"
  ```
  Expected: V8 v7 report committed. If microbench fails the isolation gate, re-run on a quiesced machine before relying on the numbers (still commit the run output for record).

- [ ] **Pre-flight 7: Capture pre-DSL-0 Test262 baseline**

  ```sh
  cargo run --release -p lyng-test262 -- --report /tmp/pre-dsl-0-test262.md -j 4
  cp /tmp/pre-dsl-0-test262.md reports/lyng/pre-dsl-0-test262-baseline.md
  git add reports/lyng/pre-dsl-0-test262-baseline.md
  git commit -m "DSL-0 pre-flight: capture pre-DSL-0 Test262 baseline"
  ```
  Expected: Test262 ≥ 49711/49729 (per R-0 `test262-after-r0.md`). Record exact pass count — this is the floor for DSL-0a / DSL-0b / DSL-0c exit gates.

---

## File structure

DSL-0 lands the following new directories and files. References are forward-looking — actual content is filled in by the relevant tasks.

### New: `crates/vm-dsl/` (proc-macro crate, DSL-0b)

```
crates/vm-dsl/
├── Cargo.toml                  # proc-macro = true; deps: syn, quote, proc-macro2
└── src/
    ├── lib.rs                  # #[proc_macro] llint_handler! entry
    ├── parse.rs                # syn-based handler-body parser
    ├── layouts.rs              # operand-layout descriptors (Abc, AbcSlot, Abx, Ax, ...)
    ├── scratch.rs              # compile-time scratch-register allocator
    └── lower.rs                # AST → naked_asm! string assembly
```

### New: `crates/vm/src/dsl/` (DSL runtime support, DSL-0a + DSL-0b)

```
crates/vm/src/dsl/
├── mod.rs                      # re-exports, backend cfg-dispatch
├── opcode_manifest.rs          # OpcodeEntry + OPCODES + structural Tests 1, 2, 3, 4, 5, 6, 7
├── slow_path.rs                # SemanticOutcome, OpXxxArgs, LlIntDispatchState, SlowPathReturn, SlowPathTag, shim helpers
├── reg_convention.rs           # pinned-register docs + const offsets via offset_of!
├── llint_state.rs              # LlIntState repr(C), LlIntRustContext, LlIntExitSlot, ExitKind, LlIntRustContextOpaque
├── entry.rs                    # Vm::run_via_dsl + _interpreter_exit
├── feedback_flat.rs            # FeedbackEntry flat-array layout (FV-pin storage)
├── ops.md                      # DSL vocabulary documentation
├── handlers/
│   ├── mod.rs                  # DSL_DISPATCH_TABLE assembly
│   ├── hot.rs                  # op_move, op_add, op_jump, op_return, op_loop_header
│   ├── warm.rs                 # op_jump8, op_jump_if_true(/8), op_jump_if_false(/8), op_wide, op_extra_wide
│   └── cold.rs                 # ~140 cold stubs (codegen-produced)
└── backend/
    ├── mod.rs                  # #[cfg(target_arch = "aarch64")] dispatch
    └── aarch64/
        ├── mod.rs              # macro re-exports
        ├── prelude.rs          # tag masks, exit-slot offsets, layout-decode helpers
        ├── operands.rs         # load_reg!, store_reg!, load_acc!, store_acc!, decode_abc!, ...
        ├── values.rs           # check_smi!, check_object_ref!, untag_smi!, tag_smi!, tag_undefined!, ...
        ├── objects.rs          # load_object_record!, load_record_shape!, load_inline_slot!, ...
        ├── arithmetic.rs       # add_smi_overflow!, sub_smi_overflow!, mul_smi_overflow!, bit_*_smi!, shift_*!
        ├── control.rs          # dispatch!, dispatch_after_slow!, call_slow!, branch_*!, dispatch_prefixed!
        ├── feedback.rs         # load_feedback_site!, value_profile!, record_smi!, ...
        ├── memory.rs           # load_byte!, load_word!, load_quad!, store_byte!, store_word!, store_quad!
        ├── counters.rs         # inc_counter! (gated by --features diagnostic-counters)
        └── safepoint.rs        # poll_safepoint!
```

### New: `crates/vm/src/vm/semantics/` (DSL-0a — semantic free functions)

```
crates/vm/src/vm/semantics/
├── mod.rs                      # re-exports + OpXxxArgs structs
├── loads.rs                    # op_move_semantic, op_load_*_semantic, op_lda_*_semantic, op_star_*_semantic, op_ldar_semantic, op_load_local_*_semantic, op_store_local_*_semantic
├── arithmetic.rs               # op_add_semantic, op_sub_semantic, ..., op_equal_semantic, op_strict_equal_semantic, op_less_than_semantic, ...
├── control_flow.rs             # op_jump_semantic, op_jump8_semantic, op_jump_if_true_semantic, op_jump_if_false_semantic, op_jump_if_true8_semantic, op_jump_if_false8_semantic, op_loop_header_semantic, op_return_semantic, op_return_undefined_semantic, op_nop_semantic
├── property.rs                 # op_get_named_property_semantic, op_set_named_property_semantic, op_assign_named_property_semantic, ... (21 opcodes)
├── names.rs                    # op_load_global_semantic, op_store_global_semantic, ..., op_load_this_semantic, op_load_callee_semantic, op_load_new_target_semantic (17 opcodes)
├── scope.rs                    # op_load_env_slot_semantic, op_store_env_slot_semantic, op_enter_env_scope_semantic, op_push_closure_env_semantic, op_push_with_env_semantic, op_type_of_semantic (10 opcodes)
├── calls.rs                    # op_call0_semantic, ..., op_call_semantic, op_tail_call_semantic, op_construct_semantic, op_create_closure_semantic, op_call_method_semantic (9 opcodes)
├── iterators.rs                # op_create_for_in_semantic, op_advance_for_in_semantic, op_close_for_in_semantic, op_create_iterator_semantic, op_advance_iterator_semantic, op_close_iterator_semantic (6 opcodes)
├── generators.rs               # op_suspend_generator_start_semantic, op_yield_semantic, op_delegate_yield_semantic, op_await_semantic, op_load_resume_kind_semantic, op_load_resume_value_semantic (6 opcodes)
├── exceptions.rs               # op_throw_semantic, op_enter_handler_semantic, op_leave_handler_semantic, op_load_exception_semantic (4 opcodes)
└── prefix.rs                   # op_wide_semantic, op_extra_wide_semantic (2 opcodes)
```

### Modified during DSL-0a (then deleted in DSL-0c)

```
crates/vm/src/vm/dispatch_handlers/
├── arithmetic.rs               # thinned: each α handler is decode → call op_xxx_semantic → translate Step
├── calls.rs                    # likewise
├── control_flow.rs             # likewise
├── exceptions.rs               # likewise
├── generators.rs               # likewise
├── iterators.rs                # likewise
├── loads.rs                    # likewise
├── names.rs                    # likewise
├── prefix.rs                   # likewise
├── property.rs                 # likewise
└── scope.rs                    # likewise
```

### Deleted in DSL-0c

```
crates/vm/src/vm/dispatch_state.rs   # DispatchState, Step, DISPATCH_TABLE, run_trampoline
crates/vm/src/vm/dispatch_handlers/  # α handlers (all of them)
crates/vm/src/vm/dispatch/           # α-only execute_*_opcode helpers (moved into semantics/)
crates/vm/src/vm/tiering.rs          # tier-accounting on backedges
```

### Modified for FV-pin / dispatch-switch

```
crates/vm/src/vm/feedback.rs         # FV flat-array refactor (eager alloc, Box<[FeedbackEntry]>)
crates/vm/src/vm/install.rs          # eager FV allocation at code install
crates/vm/src/vm.rs                  # Vm::run routes to run_via_dsl after switch
```

---

## Subagent dispatch strategy

Tasks in this plan are sized for subagent dispatch via `superpowers:subagent-driven-development`. Notes for the orchestrator:

- **Parallel within phase, sequential across phases.** DSL-0a tasks A8–A18 (opcode-family extractions) are independent and can run concurrently once A1–A7 (manifest scaffolding + transitional `LlIntDispatchState`) are in. DSL-0b has a strict dependency chain through Task B14 (entry/exit shims); after that, validation cases B30–B38 and the FV refactor B15–B19 can run in parallel; hot ports B39–B45 depend on B30 (first validation case passing). DSL-0c is fully sequential.

- **Worktree discipline.** Every subagent dispatch starts with a worktree-verification preamble matching R-0's pattern. Subagents must not commit to `main`. If a subagent reports finishing without committing, the orchestrator commits on the subagent's behalf with the agreed message format.

- **Verification rigor.** After every task, the orchestrator runs `cargo build`, `cargo test -p lyng-vm`, and any task-specific acceptance command before marking the task complete. Subagents may produce partial work; the orchestrator is the verifier of record.

- **dcat alignment.** Phase boundaries (DSL-0a → DSL-0b → DSL-0c) create dcat tickets and gate them on user approval before close, per the project's "never close issues without explicit user approval" rule (AGENTS.md).

- **Ticket creation up front.** Task A1 creates dcat tickets for every numbered task in the plan, so subagents have a referenceable ticket ID at dispatch time.

---

## Phase A — DSL-0a: Semantic extraction (Tasks A1–A20)

**Goal:** Every one of the 152 opcodes has its semantic body extracted into a free `op_xxx_semantic` function reachable from both the α handler (during DSL-0a/b) and the DSL cold-stub shim (starting DSL-0b). The α handler in `dispatch_handlers/` is thinned to operand-decode + call-semantic + translate-Step.

**Estimated duration:** 3–4 weeks.

**Exit criteria** (verified by Task A20):
1. Every `Opcode` variant has a corresponding entry in `OPCODES` (Manifest Test 1).
2. Every `semantic_symbol` resolves to a real function (Manifest Test 2).
3. Source-grep smoke test (Manifest Test 4) finds no handler-shaped logic outside `op_xxx_semantic` functions or their narrow operand-decode wrappers.
4. `cargo test -p lyng-vm -p lyng-bytecode -p lyng-objects -p lyng-tests -p lyng-compiler` passes (same count as Pre-flight 4).
5. Test262 pass count ≥ Pre-flight 7 baseline.

---

### Task A1: Create DSL-0 dcat tickets

**Files:** none in repo; uses dcat issue tracker.

- [ ] **Step 1: Read dcat workflow**

  Run:
  ```sh
  dcat prime --opinionated
  ```
  Expected: prints dcat workflow guide.

- [ ] **Step 2: Look up the parent epic and R-0 ticket**

  ```sh
  dcat show lyng-49qk
  dcat list --parent lyng-49qk --status in_review
  ```
  Expected: parent epic + the R-0 ticket and its sub-tickets in `in_review`.

- [ ] **Step 3: Create DSL-0 parent ticket**

  ```sh
  dcat create "DSL-0: spike + semantic extraction + full opcode coverage" \
    --type epic --priority 1 \
    --parent lyng-49qk \
    --labels js,performance,vm,roadmap,dsl \
    -d "DSL-0 milestone per §10 of docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md. Plan: docs/superpowers/plans/2026-05-17-dsl-0-spike-and-semantic-extraction.md."
  ```
  Record the returned ID as `<DSL0_PARENT>`.

- [ ] **Step 4: Create DSL-0a, DSL-0b, DSL-0c sub-epics**

  ```sh
  dcat create "DSL-0a: semantic extraction across all 152 opcodes" --type epic --priority 1 --parent <DSL0_PARENT> --labels js,vm,dsl
  dcat create "DSL-0b: DSL infrastructure + hot/warm/cold ports + FV flat-array refactor" --type epic --priority 1 --parent <DSL0_PARENT> --labels js,vm,dsl,performance
  dcat create "DSL-0c: delete alpha trampoline + verify single-implementation invariant" --type epic --priority 1 --parent <DSL0_PARENT> --labels js,vm,dsl
  ```
  Record returned IDs as `<DSL0A>`, `<DSL0B>`, `<DSL0C>`.

- [ ] **Step 5: Create task tickets under each sub-epic**

  Under `<DSL0A>`: one ticket per Task A2 through A20 (`Task A2: vm/src/dsl/ scaffold`, etc.).
  Under `<DSL0B>`: one ticket per Task B1 through B50.
  Under `<DSL0C>`: one ticket per Task C1 through C13.

  Use the dcat batch script:
  ```sh
  # Example for one ticket; repeat per task.
  dcat create "Task A2: vm/src/dsl/ scaffold + opcode_manifest skeleton" --type task --priority 1 --parent <DSL0A> --labels js,vm,dsl
  ```

  Persist the ticket IDs to `reports/lyng/dsl-0-ticket-map.md` so subagents can resolve task→ticket:
  ```markdown
  # DSL-0 ticket map

  | Task | Ticket |
  | --- | --- |
  | A1 | <ticket id> |
  | A2 | <ticket id> |
  | ... | ... |
  ```

- [ ] **Step 6: Commit the ticket map**

  ```sh
  git add reports/lyng/dsl-0-ticket-map.md
  git commit -m "DSL-0: create dcat ticket hierarchy + map"
  ```

---

### Task A2: Create `vm/src/dsl/` scaffold + opcode_manifest skeleton

**Files:**
- Create: `crates/vm/src/dsl/mod.rs`
- Create: `crates/vm/src/dsl/opcode_manifest.rs`
- Modify: `crates/vm/src/lib.rs`

- [ ] **Step 1: Create the `dsl` module entry**

  Create `crates/vm/src/dsl/mod.rs`:
  ```rust
  //! asm-DSL substrate per docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md.
  //!
  //! This module hosts the DSL runtime support — opcode manifest, slow-path
  //! bridge types, register-pin convention, `LlIntState` ABI, entry/exit
  //! shims, and the per-arch DSL operation backend. The proc-macro that
  //! consumes these lives in the separate `lyng-vm-dsl` crate.
  //!
  //! During DSL-0a the only module populated is `opcode_manifest` plus the
  //! transitional `LlIntDispatchState` wrapper in `slow_path`. DSL-0b adds
  //! every other module.

  pub mod opcode_manifest;
  ```

- [ ] **Step 2: Create the opcode_manifest skeleton**

  Create `crates/vm/src/dsl/opcode_manifest.rs`:
  ```rust
  //! Single-implementation invariant manifest per design §10.
  //!
  //! `OPCODES` enumerates every `Opcode` variant exactly once with the
  //! resolvable symbol names for its semantic body and (post-DSL-0b) its
  //! DSL handler. Seven structural tests use this manifest to verify the
  //! invariant — see the `manifest_tests` module.

  use lyng_bytecode::Opcode;

  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub enum OpcodeCategory {
      /// Full DSL body with inline fast paths (5 opcodes from DSL-0b plus
      /// 25 more in DSL-1).
      Hot,
      /// Full DSL body that includes a safepoint poll on its backedge
      /// (loop header + backward-jump variants + prefix opcodes).
      Warm,
      /// Three-line DSL stub delegating to a slow-path Rust shim.
      Cold,
  }

  #[derive(Clone, Copy, Debug)]
  pub struct OpcodeEntry {
      pub opcode: Opcode,
      pub semantic_symbol: &'static str,
      pub dsl_handler_symbol: &'static str,
      pub category: OpcodeCategory,
  }

  /// The single source of truth for the single-implementation invariant.
  ///
  /// Tests A6 / A19 / C9 / C10 / C11 walk this slice to verify exhaustive
  /// coverage and symbol resolution. Adding an `Opcode` variant without
  /// extending this slice fails Test 1 (exhaustive coverage).
  pub const OPCODES: &[OpcodeEntry] = &[
      // Populated by family-extraction tasks A8–A18.
  ];

  /// Subset filter for the DSL_DISPATCH_TABLE assembly in DSL-0b.
  pub fn by_category(category: OpcodeCategory) -> impl Iterator<Item = &'static OpcodeEntry> {
      OPCODES.iter().filter(move |entry| entry.category == category)
  }
  ```

- [ ] **Step 3: Wire the module into `lib.rs`**

  Open `crates/vm/src/lib.rs`. Find the existing top-level `pub mod` block (near the `pub mod vm;` line). Add:
  ```rust
  pub mod dsl;
  ```
  Place it alphabetically — between `pub mod activation` and `pub mod enumeration` (or wherever the existing imports sit).

- [ ] **Step 4: Verify it compiles**

  ```sh
  cargo build -p lyng-vm
  ```
  Expected: clean build. `OPCODES` is empty and `unused`-warning is suppressed via the `pub` visibility.

- [ ] **Step 5: Commit**

  ```sh
  git add crates/vm/src/dsl/mod.rs \
          crates/vm/src/dsl/opcode_manifest.rs \
          crates/vm/src/lib.rs
  git commit -m "DSL-0a: dsl/ module scaffold + opcode_manifest skeleton"
  ```

---

### Task A3: Define `SemanticOutcome` and per-opcode argument structs

**Files:**
- Create: `crates/vm/src/dsl/slow_path.rs`
- Modify: `crates/vm/src/dsl/mod.rs`

`SemanticOutcome` is the return type of every `op_xxx_semantic` function. It encodes the four dispatch decisions a semantic body can produce: Continue (advance PC), Refresh (frame changed, reload pinned regs), Exit-Done, Exit-Error. The α handler in `dispatch_handlers/` translates this into `Step::Continue / Done / Error`; the DSL cold-stub shim translates it into `SlowPathReturn`.

- [ ] **Step 1: Create the slow_path module**

  Create `crates/vm/src/dsl/slow_path.rs`:
  ```rust
  //! Slow-path bridge: semantic-outcome type + per-opcode argument structs
  //! + (DSL-0b) the `LlIntDispatchState` wrapper and `SlowPathReturn` ABI.
  //!
  //! During DSL-0a only `SemanticOutcome`, the `OpXxxArgs` structs, and the
  //! transitional `LlIntDispatchState` alias are populated. The asm-facing
  //! shim layer and `SlowPathReturn`/`SlowPathTag` lands in DSL-0b.

  use lyng_types::Value;

  use crate::error::VmError;

  /// Logical outcome of a semantic-body invocation. The α handler maps
  /// this to `Step`; the DSL cold-stub shim maps it to `SlowPathReturn`.
  pub enum SemanticOutcome {
      /// Dispatch continues at the post-instruction PC. `pc_advance` is
      /// the number of bytes the semantic body consumed (i.e. the
      /// instruction length when execution did not branch, or the absolute
      /// target offset minus the entry PC when the body performed a jump).
      Continue { pc_advance: u32 },
      /// Frame changed (call / return / cross-frame catch). The dispatcher
      /// must reload pinned PC/REGS/FV from the canonical frame state.
      Refresh,
      /// Successful program completion; `Vm::run` returns `Ok(value)`.
      ExitDone { value: Value },
      /// Abrupt completion that escapes the current `Vm::run`; the bridge
      /// returns `Err(error)`.
      ExitError { error: VmError },
  }
  ```

- [ ] **Step 2: Wire the module into `dsl/mod.rs`**

  Edit `crates/vm/src/dsl/mod.rs` and add:
  ```rust
  pub mod slow_path;
  ```

- [ ] **Step 3: Verify it compiles**

  ```sh
  cargo build -p lyng-vm
  ```
  Expected: clean build.

- [ ] **Step 4: Commit**

  ```sh
  git add crates/vm/src/dsl/slow_path.rs crates/vm/src/dsl/mod.rs
  git commit -m "DSL-0a: SemanticOutcome scaffolding"
  ```

---

### Task A4: Define transitional `LlIntDispatchState` wrapper

**Files:**
- Modify: `crates/vm/src/dsl/slow_path.rs`

The DSL-0a wrapper aliases today's `DispatchState`. The same wrapper type is reconstructed via `from_raw` in DSL-0b. Existing semantic logic uses the wrapper through the same API in both phases — that's the contract that lets the alpha path and the DSL cold-stub shim share one semantic body.

- [ ] **Step 1: Add the wrapper to `slow_path.rs`**

  Append to `crates/vm/src/dsl/slow_path.rs`:
  ```rust
  use crate::vm::dispatch_state::DispatchState;

  /// Safe wrapper around a per-frame dispatch state.
  ///
  /// During DSL-0a this holds a `&mut DispatchState<'vm>` directly — the
  /// asm bridge does not exist yet, so semantic bodies reach VM state via
  /// the legacy `DispatchState` accessors re-exposed here. In DSL-0b the
  /// wrapper is also reachable through `LlIntDispatchState::from_raw`,
  /// which reconstructs it from a `*mut LlIntState` passed by the asm
  /// shim. The semantic body sees identical method signatures in both
  /// paths — that's the single-implementation invariant in action.
  pub struct LlIntDispatchState<'vm, 'borrow> {
      pub(crate) inner: LlIntDispatchInner<'vm, 'borrow>,
  }

  pub(crate) enum LlIntDispatchInner<'vm, 'borrow> {
      /// Borrowed from a live `DispatchState` (alpha path, transitional).
      Alpha(&'borrow mut DispatchState<'vm>),
      // Asm(...) variant lands in DSL-0b.
  }

  impl<'vm, 'borrow> LlIntDispatchState<'vm, 'borrow> {
      /// Construct from a live α `DispatchState`. The α handler in
      /// `dispatch_handlers/` calls this to forward into `op_xxx_semantic`.
      pub fn from_alpha(state: &'borrow mut DispatchState<'vm>) -> Self {
          Self { inner: LlIntDispatchInner::Alpha(state) }
      }

      /// Mutable access to the underlying `DispatchState`. Semantic
      /// bodies use this for now; the DSL-0b refactor replaces this with
      /// typed accessors that operate uniformly across α and asm paths.
      pub fn dispatch_state(&mut self) -> &mut DispatchState<'vm> {
          match &mut self.inner {
              LlIntDispatchInner::Alpha(state) => *state,
          }
      }
  }
  ```

- [ ] **Step 2: Make `DispatchState` reachable from `dsl::slow_path`**

  Open `crates/vm/src/vm/dispatch_state.rs`. The `DispatchState` fields are `pub(crate)`. Add `pub(crate)` to the struct itself if not already (it already is — `pub struct DispatchState<'vm>` at line ~45). No change needed; the import in slow_path.rs already works because both modules live in the `lyng-vm` crate.

- [ ] **Step 3: Verify it compiles**

  ```sh
  cargo build -p lyng-vm
  ```
  Expected: clean build. The wrapper compiles because the only variant is `Alpha`, which trivially uses the existing `DispatchState` type.

- [ ] **Step 4: Commit**

  ```sh
  git add crates/vm/src/dsl/slow_path.rs
  git commit -m "DSL-0a: transitional LlIntDispatchState wrapper"
  ```

---

### Task A5: Define `semantics/` module skeleton

**Files:**
- Create: `crates/vm/src/vm/semantics/mod.rs`
- Modify: `crates/vm/src/vm.rs` (or `vm/mod.rs` if applicable)

- [ ] **Step 1: Check the vm module layout**

  Run:
  ```sh
  ls crates/vm/src/vm/
  head -50 crates/vm/src/lib.rs
  ```
  Expected: confirm whether `vm.rs` or `vm/mod.rs` is the module root.

- [ ] **Step 2: Create the semantics module skeleton**

  Create `crates/vm/src/vm/semantics/mod.rs`:
  ```rust
  //! Free-function semantic bodies per design §10 DSL-0a.
  //!
  //! Each `op_xxx_semantic` function implements the semantic effect of
  //! one bytecode opcode. The α handler in `dispatch_handlers/` decodes
  //! operands and calls into one of these; in DSL-0b the same function
  //! is also reachable from the DSL cold-stub shim in
  //! `crates/vm/src/dsl/slow_path.rs`.
  //!
  //! Per-family submodules are added by family-extraction tasks A8–A18.
  //! `OpXxxArgs` structs live alongside their semantic body.

  // Family submodules are added by tasks A8–A18.
  ```

- [ ] **Step 3: Wire the module into the vm tree**

  Open `crates/vm/src/vm.rs` (or `vm/mod.rs`). Find the existing `mod` block near the top. Add `pub(crate) mod semantics;` alphabetically.

- [ ] **Step 4: Verify it compiles**

  ```sh
  cargo build -p lyng-vm
  ```
  Expected: clean build.

- [ ] **Step 5: Commit**

  ```sh
  git add crates/vm/src/vm/semantics/mod.rs crates/vm/src/vm.rs
  git commit -m "DSL-0a: semantics/ module skeleton"
  ```

---

### Task A6: Manifest Test 1 — exhaustive `Opcode` coverage

**Files:**
- Modify: `crates/vm/src/dsl/opcode_manifest.rs`

This test fails today (`OPCODES` is empty). It stays failing through tasks A8–A18 and turns green once every family extraction has registered its opcodes in the manifest. We land the test first as the structural guardrail.

- [ ] **Step 1: Append the manifest_tests module**

  Append to `crates/vm/src/dsl/opcode_manifest.rs`:
  ```rust
  #[cfg(test)]
  mod manifest_tests {
      use super::*;
      use lyng_bytecode::{Opcode, OPCODE_COUNT};
      use std::collections::HashSet;

      /// Test 1 from design §10 DSL-0a: every `Opcode` variant appears in
      /// `OPCODES` exactly once.
      #[test]
      fn opcodes_manifest_is_exhaustive() {
          let count = OPCODE_COUNT as usize;
          assert_eq!(
              OPCODES.len(),
              count,
              "OPCODES has {} entries, expected {} (OPCODE_COUNT)",
              OPCODES.len(),
              count,
          );

          let mut seen: HashSet<u8> = HashSet::new();
          for entry in OPCODES {
              let byte = entry.opcode as u8;
              assert!(
                  byte < OPCODE_COUNT,
                  "OPCODES entry for {:?} has byte {} outside [0, {})",
                  entry.opcode,
                  byte,
                  OPCODE_COUNT,
              );
              assert!(
                  seen.insert(byte),
                  "OPCODES has duplicate entry for opcode byte {} ({:?})",
                  byte,
                  entry.opcode,
              );
          }

          for byte in 0..OPCODE_COUNT {
              assert!(
                  seen.contains(&byte),
                  "OPCODES missing entry for opcode byte {}: {:?}",
                  byte,
                  Opcode::from_byte(byte),
              );
          }
      }
  }
  ```

  Note: if `Opcode::from_byte` doesn't exist, replace its use with `byte` formatting only — the diagnostic just needs to be readable.

- [ ] **Step 2: Run the failing test**

  ```sh
  cargo test -p lyng-vm dsl::opcode_manifest::manifest_tests::opcodes_manifest_is_exhaustive 2>&1 | tail -20
  ```
  Expected: FAIL. Message names a specific opcode byte that's missing. Record the failure — this is the regression check that drives tasks A8–A18.

- [ ] **Step 3: Mark the test as `#[ignore]` until A18 completes**

  Change the `#[test]` attribute to:
  ```rust
  #[test]
  #[ignore = "Enabled by Task A18 once all family extractions are complete"]
  ```

- [ ] **Step 4: Verify the ignored test still compiles**

  ```sh
  cargo test -p lyng-vm dsl::opcode_manifest 2>&1 | tail -10
  ```
  Expected: 0 tests passed, 1 ignored.

- [ ] **Step 5: Commit**

  ```sh
  git add crates/vm/src/dsl/opcode_manifest.rs
  git commit -m "DSL-0a: manifest Test 1 (exhaustive Opcode coverage), ignored until A18"
  ```

---

### Task A7: Manifest Test 4 — source-grep smoke test

**Files:**
- Create: `crates/vm/tests/dsl_manifest_grep.rs`

Test 4 is defense-in-depth: ensures no opcode-shaped semantic logic sneaks back into helper modules outside the `semantics/` directory. Runs as a normal `#[test]` because it operates on source files.

- [ ] **Step 1: Create the smoke test**

  Create `crates/vm/tests/dsl_manifest_grep.rs`:
  ```rust
  //! Test 4 from design §10 DSL-0a: opcode-shaped semantic logic lives only in
  //! `crates/vm/src/vm/semantics/` and (transitionally) in
  //! `crates/vm/src/vm/dispatch_handlers/` as decode-and-call thunks.
  //!
  //! This test reads source files and rejects function names matching
  //! `^pub(\(.*\))?\s*fn\s+op_[a-z0-9_]+\s*\(` (i.e. `op_xxx` functions)
  //! in any module other than `semantics/` and `dispatch_handlers/`.

  use std::fs;
  use std::path::{Path, PathBuf};

  const VM_SRC: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src");

  fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
      for entry in fs::read_dir(dir).expect("read_dir") {
          let entry = entry.expect("dir entry");
          let path = entry.path();
          if path.is_dir() {
              collect_rs(&path, out);
          } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
              out.push(path);
          }
      }
  }

  #[test]
  #[ignore = "Enabled by Task A18 once all family extractions are complete"]
  fn no_op_functions_outside_semantics_and_handlers() {
      let mut files = Vec::new();
      collect_rs(Path::new(VM_SRC), &mut files);

      let allowlist_prefixes = [
          format!("{VM_SRC}/vm/semantics/"),
          format!("{VM_SRC}/vm/dispatch_handlers/"),
          format!("{VM_SRC}/dsl/handlers/"),  // DSL-0b host
      ];

      let mut offenders: Vec<(PathBuf, usize, String)> = Vec::new();
      for path in &files {
          let path_str = path.to_string_lossy();
          if allowlist_prefixes.iter().any(|p| path_str.starts_with(p)) {
              continue;
          }
          let body = fs::read_to_string(path).expect("read source");
          for (line_no, line) in body.lines().enumerate() {
              let trimmed = line.trim_start();
              if (trimmed.starts_with("pub fn op_")
                  || trimmed.starts_with("pub(crate) fn op_")
                  || trimmed.starts_with("pub(super) fn op_")
                  || trimmed.starts_with("fn op_"))
                  && trimmed.contains("(")
                  // Skip `op_xxx_slow` helper functions — they're allowed in
                  // dispatch_handlers/ today and will move to dsl/handlers/
                  // in DSL-0b.
                  && !trimmed.contains("op_") .clone().to_string().contains("_slow(")
              {
                  offenders.push((path.clone(), line_no + 1, trimmed.to_string()));
              }
          }
      }

      if !offenders.is_empty() {
          let report: Vec<String> = offenders
              .iter()
              .map(|(p, n, l)| format!("{}:{}: {}", p.display(), n, l))
              .collect();
          panic!(
              "Found op_* function(s) outside semantics/, dispatch_handlers/, or dsl/handlers/:\n{}",
              report.join("\n"),
          );
      }
  }
  ```

  Note: the `_slow` exclusion is approximate. Tighten in Task A18 if it produces false negatives or false positives.

- [ ] **Step 2: Verify the test compiles and is correctly ignored**

  ```sh
  cargo test -p lyng-vm --test dsl_manifest_grep 2>&1 | tail -10
  ```
  Expected: 0 tests passed, 1 ignored.

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/tests/dsl_manifest_grep.rs
  git commit -m "DSL-0a: manifest Test 4 (source-grep smoke), ignored until A18"
  ```

---

### Task A8: Extract `loads` family (35 opcodes)

**Files:**
- Create: `crates/vm/src/vm/semantics/loads.rs`
- Modify: `crates/vm/src/vm/semantics/mod.rs`
- Modify: `crates/vm/src/vm/dispatch_handlers/loads.rs`
- Modify: `crates/vm/src/dsl/opcode_manifest.rs`

**Opcodes in scope:** `Move`, `LoadUndefined`, `LoadUninitializedLexical`, `LoadNull`, `LoadTrue`, `LoadFalse`, `LoadZero`, `LoadOne`, `LoadSmi`, `LoadConst`, `Wide` and `ExtraWide` are not here (they're in `prefix` family — Task A18). `LdaUndefined`, `LdaNull`, `LdaTrue`, `LdaFalse`, `LdaZero`, `LdaOne`, `LdaSmi8`, `LdaConst8`, `Ldar`, `Star0..7`, `LoadSmi8`, `LoadConst8`, `LoadLocal0..3`, `StoreLocal0..3` — full list per `dispatch_handlers/loads.rs` `pub use` line.

- [ ] **Step 1: Read the existing α handlers**

  ```sh
  wc -l crates/vm/src/vm/dispatch_handlers/loads.rs
  rg -n "^pub extern" crates/vm/src/vm/dispatch_handlers/loads.rs
  ```
  Expected: lists ~35 `pub extern "C" fn op_xxx` entries matching the family.

- [ ] **Step 2: Create the loads semantics module**

  Create `crates/vm/src/vm/semantics/loads.rs`. For each α handler in `dispatch_handlers/loads.rs`, port its semantic body into:
  ```rust
  use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};

  pub struct OpMoveArgs {
      pub dst: u16,
      pub src: u16,
      pub instruction_len: u32,
  }

  pub(crate) fn op_move_semantic(
      state: &mut LlIntDispatchState<'_, '_>,
      args: OpMoveArgs,
  ) -> SemanticOutcome {
      let inner = state.dispatch_state();
      let registers = inner.frame.registers();
      let value = inner.vm.read_register_unchecked(registers, args.src);
      inner.vm.write_register_unchecked(registers, args.dst, value);
      SemanticOutcome::Continue { pc_advance: args.instruction_len }
  }
  ```

  Each opcode follows the same shape: an `OpXxxArgs` struct holding the decoded operands, an `op_xxx_semantic` function returning `SemanticOutcome`.

  For control-flow-free loads (e.g. `op_load_undefined`, `op_lda_zero`), the body is a register write + `Continue`. For `Star*` opcodes, the body writes the accumulator to the indexed register slot. For `Ldar`, it loads the named register into the accumulator. Match the existing α handler logic; do not add abstractions.

- [ ] **Step 3: Wire `loads` into `semantics/mod.rs`**

  Append to `crates/vm/src/vm/semantics/mod.rs`:
  ```rust
  pub(crate) mod loads;
  ```

- [ ] **Step 4: Thin each α handler in `dispatch_handlers/loads.rs`**

  Replace each `pub extern "C" fn op_xxx(state: &mut DispatchState) -> Step` body with:
  ```rust
  pub extern "C" fn op_move(state: &mut DispatchState) -> Step {
      // 1. Decode operands (unchanged from prior implementation).
      let prefix = state.prefix.take();
      let code = state.code();
      let pc = state.frame.instruction_offset();
      let (dst, src, instruction_len) = try_step!(
          decode_ab_operands(state.current_bytes(), prefix, code, pc),
      );

      // 2. Call semantic body via the transitional LlIntDispatchState.
      let mut ll_state = LlIntDispatchState::from_alpha(state);
      let outcome = crate::vm::semantics::loads::op_move_semantic(
          &mut ll_state,
          crate::vm::semantics::loads::OpMoveArgs { dst, src, instruction_len },
      );

      // 3. Translate SemanticOutcome → Step.
      translate_outcome_to_step(state, outcome)
  }
  ```

  Add a translation helper at the top of `dispatch_handlers/mod.rs` (or a sibling module):
  ```rust
  use crate::dsl::slow_path::SemanticOutcome;

  pub(crate) fn translate_outcome_to_step(
      state: &mut DispatchState,
      outcome: SemanticOutcome,
  ) -> Step {
      match outcome {
          SemanticOutcome::Continue { pc_advance } => {
              state.advance(pc_advance);
              dispatch_next!(state)
          }
          SemanticOutcome::Refresh => {
              if let Err(err) = state.refresh_from_active_frame() {
                  return Step::Error(err);
              }
              dispatch_next!(state)
          }
          SemanticOutcome::ExitDone { value } => Step::Done(value),
          SemanticOutcome::ExitError { error } => Step::Error(error),
      }
  }
  ```

  Note: `dispatch_next!` is a macro that already `return`s `Step::Continue`. The `match` arms above for Continue and Refresh use `dispatch_next!(state)` directly — the macro `return`s, so the `translate_outcome_to_step` body has a divergent type. If borrow-checker complaints arise, split into explicit `match` + early return rather than expression-style. Verify the helper's signature matches what each arm produces.

- [ ] **Step 5: Register loads opcodes in the manifest**

  In `crates/vm/src/dsl/opcode_manifest.rs`, append entries to `OPCODES` for each loads opcode:
  ```rust
  pub const OPCODES: &[OpcodeEntry] = &[
      OpcodeEntry {
          opcode: Opcode::Move,
          semantic_symbol: "lyng_vm::vm::semantics::loads::op_move_semantic",
          dsl_handler_symbol: "lyng_vm::dsl::handlers::cold::op_move_dsl",
          category: OpcodeCategory::Cold,  // refined to Hot in Task B39
      },
      // ... one entry per loads opcode
  ];
  ```

  Note: `dsl_handler_symbol` references symbols that will exist in DSL-0b. Manifest Test 3 (linker resolution for `dsl_handler_symbol`) is gated to run after DSL-0b; Test 2 (semantic_symbol resolution) runs at A19.

- [ ] **Step 6: Verify the family compiles and behavior tests pass**

  ```sh
  cargo build -p lyng-vm
  cargo test -p lyng-vm -p lyng-bytecode -p lyng-tests 2>&1 | tail -20
  ```
  Expected: clean build; all tests pass (no regression vs Pre-flight 4).

- [ ] **Step 7: Commit**

  ```sh
  git add crates/vm/src/vm/semantics/loads.rs \
          crates/vm/src/vm/semantics/mod.rs \
          crates/vm/src/vm/dispatch_handlers/loads.rs \
          crates/vm/src/vm/dispatch_handlers/mod.rs \
          crates/vm/src/dsl/opcode_manifest.rs
  git commit -m "DSL-0a: extract loads family (35 opcodes) into semantics::loads"
  ```

---

### Task A9: Extract `arithmetic` family (~30 opcodes)

**Files:**
- Create: `crates/vm/src/vm/semantics/arithmetic.rs`
- Modify: `crates/vm/src/vm/semantics/mod.rs`
- Modify: `crates/vm/src/vm/dispatch_handlers/arithmetic.rs`
- Modify: `crates/vm/src/dsl/opcode_manifest.rs`

**Opcodes in scope:** `Add`, `AddSmi`, `Sub`, `SubSmi`, `Mul`, `MulSmi`, `Div`, `Mod`, `DivSmi`, `ModSmi`, `Exp`, `BitOr`, `BitXor`, `BitAnd`, `BitAndSmi`, `BitNot`, `ShiftLeft`, `ShiftRight`, `UnsignedShiftRight`, `Negate`, `Increment`, `Decrement`, `Equal`, `StrictEqual`, `EqualZero`, `LessThan`, `LessEqual`, `GreaterThan`, `GreaterEqual`. Per the `pub use arithmetic::{...}` re-export in `dispatch_handlers/mod.rs`.

- [ ] **Step 1: Inspect existing `execute_*_opcode` helpers**

  ```sh
  rg -n "execute_(add|sub|mul|div|mod|exp|bit|shift|equal|less|greater|negate|increment|decrement)" \
     crates/vm/src/vm/dispatch/arithmetic.rs | head -40
  ```
  Expected: ~30 helper methods on `Vm`. Most of the work for this family is renaming them to `op_xxx_semantic` free functions.

- [ ] **Step 2: Create `semantics/arithmetic.rs`**

  For each opcode, port the existing α handler body in `dispatch_handlers/arithmetic.rs` (which already follows the "fast path + slow helper" shape — see `op_add` lines 39–108). The semantic body is the full handler logic up to the `dispatch_next!` call, replaced with `SemanticOutcome::Continue { pc_advance: instruction_len }`.

  Example for `op_add`:
  ```rust
  use lyng_types::{FeedbackSlotId, Value};
  use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};

  pub struct OpAddArgs {
      pub dst: u16,
      pub lhs: u16,
      pub rhs: u16,
      pub feedback_slot: Option<FeedbackSlotId>,
      pub instruction_len: u32,
  }

  pub(crate) fn op_add_semantic(
      state: &mut LlIntDispatchState<'_, '_>,
      args: OpAddArgs,
  ) -> SemanticOutcome {
      let inner = state.dispatch_state();
      let code = inner.code();
      let registers = inner.frame.registers();
      let left = inner.vm.read_register_unchecked(registers, args.lhs);
      let right = inner.vm.read_register_unchecked(registers, args.rhs);
      if let (Some(l), Some(r)) = (left.as_smi(), right.as_smi())
          && let Some(v) = l.checked_add(r)
      {
          inner.vm.record_feedback_slot(code, args.feedback_slot);
          inner.vm.write_register_unchecked(registers, args.dst, Value::from_smi(v));
          return SemanticOutcome::Continue { pc_advance: args.instruction_len };
      }
      op_add_slow_path(inner, args)
  }

  fn op_add_slow_path(state: &mut crate::vm::dispatch_state::DispatchState<'_>, args: OpAddArgs) -> SemanticOutcome {
      let result = {
          let crate::vm::dispatch_state::DispatchState { vm, agent, host, registry, frame, .. } = &mut *state;
          vm.execute_add_opcode(agent, *host, &mut **registry, frame, args.lhs, args.rhs)
      };
      let finish = {
          let crate::vm::dispatch_state::DispatchState { vm, agent, frame, frame_depth, .. } = &mut *state;
          vm.finish_abc_value_result(agent, *frame_depth, frame, args.instruction_len, args.feedback_slot, args.dst, result)
      };
      match finish {
          Ok(Some(_)) => SemanticOutcome::Continue { pc_advance: 0 }, // PC already updated by finish
          Ok(None) => SemanticOutcome::Refresh,  // catch transferred to a handler
          Err(error) => SemanticOutcome::ExitError { error },
      }
  }
  ```

  Note: `finish_abc_value_result` already advances PC and writes the destination register on success; the resulting `SemanticOutcome::Continue { pc_advance: 0 }` reflects that. Verify against the existing α handler — if PC isn't actually advanced inside `finish_abc_value_result`, return `Continue { pc_advance: args.instruction_len }` instead and adjust the helper to not double-advance.

  Repeat the same template for each of the ~30 arithmetic opcodes. For `*Smi` variants, the second operand is a decoded `i16` immediate, not a register. For unary opcodes (`Negate`, `Increment`, `Decrement`, `BitNot`, `EqualZero`), the args carry only `src` + `dst` + slot + len.

- [ ] **Step 3: Wire `arithmetic` into `semantics/mod.rs`**

  Append:
  ```rust
  pub(crate) mod arithmetic;
  ```

- [ ] **Step 4: Thin each α handler in `dispatch_handlers/arithmetic.rs`**

  Replace each handler's body with:
  ```rust
  pub extern "C" fn op_add(state: &mut DispatchState) -> Step {
      let code = state.code();
      let pc = state.frame.instruction_offset();
      let prefix = state.prefix.take();
      let (dst, lhs, rhs, feedback_slot, instruction_len) = try_step!(decode_abc_operands(
          state.current_bytes(), prefix, true, code, pc,
      ));
      let mut ll_state = crate::dsl::slow_path::LlIntDispatchState::from_alpha(state);
      let outcome = crate::vm::semantics::arithmetic::op_add_semantic(
          &mut ll_state,
          crate::vm::semantics::arithmetic::OpAddArgs {
              dst, lhs, rhs, feedback_slot, instruction_len,
          },
      );
      translate_outcome_to_step(state, outcome)
  }
  ```

- [ ] **Step 5: Register arithmetic opcodes in the manifest**

  Append entries to `OPCODES` in `opcode_manifest.rs` for each arithmetic opcode following the loads pattern.

- [ ] **Step 6: Verify build + tests**

  ```sh
  cargo build -p lyng-vm
  cargo test -p lyng-vm -p lyng-bytecode -p lyng-tests 2>&1 | tail -20
  ```
  Expected: clean build; no regressions.

- [ ] **Step 7: Commit**

  ```sh
  git add crates/vm/src/vm/semantics/arithmetic.rs \
          crates/vm/src/vm/semantics/mod.rs \
          crates/vm/src/vm/dispatch_handlers/arithmetic.rs \
          crates/vm/src/dsl/opcode_manifest.rs
  git commit -m "DSL-0a: extract arithmetic family (~30 opcodes) into semantics::arithmetic"
  ```

---

### Task A10: Extract `control_flow` family (10 opcodes)

**Files:**
- Create: `crates/vm/src/vm/semantics/control_flow.rs`
- Modify: `crates/vm/src/vm/semantics/mod.rs`
- Modify: `crates/vm/src/vm/dispatch_handlers/control_flow.rs`
- Modify: `crates/vm/src/dsl/opcode_manifest.rs`

**Opcodes in scope:** `Jump`, `Jump8`, `JumpIfTrue`, `JumpIfTrue8`, `JumpIfFalse`, `JumpIfFalse8`, `LoopHeader`, `Return`, `ReturnUndefined`, `Nop`.

- [ ] **Step 1: Create `semantics/control_flow.rs`**

  For unconditional jumps, the semantic body computes the target PC and returns `Continue { pc_advance: <new_pc - entry_pc> }`. For `op_return`, the body calls `state.finish_active_frame(value)` and returns either `ExitDone` (top-frame return) or `Refresh` (return-to-caller).

  Example for `op_jump`:
  ```rust
  use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};

  pub struct OpJumpArgs {
      pub offset: i32,
      pub instruction_len: u32,
  }

  pub(crate) fn op_jump_semantic(
      _state: &mut LlIntDispatchState<'_, '_>,
      args: OpJumpArgs,
  ) -> SemanticOutcome {
      // PC at handler entry already points to the start of the instruction.
      // Continue with pc_advance = (instruction_len + offset).
      let advance = (args.instruction_len as i64) + (args.offset as i64);
      SemanticOutcome::Continue { pc_advance: advance as u32 }
  }
  ```

  Confirm against the existing α `op_jump` implementation — the offset's sign convention (relative to instruction start vs. instruction end) matters.

  Example for `op_return`:
  ```rust
  pub struct OpReturnArgs {
      pub src: u16,
  }

  pub(crate) fn op_return_semantic(
      state: &mut LlIntDispatchState<'_, '_>,
      args: OpReturnArgs,
  ) -> SemanticOutcome {
      let inner = state.dispatch_state();
      let registers = inner.frame.registers();
      let value = inner.vm.read_register_unchecked(registers, args.src);
      match inner.finish_active_frame(value) {
          Ok(Some(top_value)) => SemanticOutcome::ExitDone { value: top_value },
          Ok(None) => SemanticOutcome::Refresh,
          Err(error) => SemanticOutcome::ExitError { error },
      }
  }
  ```

  For `op_loop_header`: it's a marker that advances by encoded length. The transitional alpha body still runs `observe_tier_backedge_event` — keep that call inside the semantic body for DSL-0a; it gets removed in DSL-0c.

- [ ] **Step 2: Wire into `semantics/mod.rs`**

  Append:
  ```rust
  pub(crate) mod control_flow;
  ```

- [ ] **Step 3: Thin each α handler**

  Replace each `pub extern "C" fn op_xxx` body with the decode + call-semantic + translate pattern from A8/A9.

- [ ] **Step 4: Register control_flow opcodes in the manifest**

- [ ] **Step 5: Verify build + tests**

  ```sh
  cargo build -p lyng-vm
  cargo test -p lyng-vm -p lyng-bytecode -p lyng-tests 2>&1 | tail -20
  ```

- [ ] **Step 6: Commit**

  ```sh
  git add crates/vm/src/vm/semantics/control_flow.rs \
          crates/vm/src/vm/semantics/mod.rs \
          crates/vm/src/vm/dispatch_handlers/control_flow.rs \
          crates/vm/src/dsl/opcode_manifest.rs
  git commit -m "DSL-0a: extract control_flow family (10 opcodes) into semantics::control_flow"
  ```

---

### Task A11: Extract `property` family (~21 opcodes)

**Files:**
- Create: `crates/vm/src/vm/semantics/property.rs`
- Modify: `crates/vm/src/vm/semantics/mod.rs`
- Modify: `crates/vm/src/vm/dispatch_handlers/property.rs`
- Modify: `crates/vm/src/dsl/opcode_manifest.rs`

**Opcodes in scope:** `GetNamedProperty`, `SetNamedProperty`, `AssignNamedProperty`, `StrictAssignNamedProperty`, `GetKeyedProperty`, `SetKeyedProperty`, `AssignKeyedProperty`, `StrictAssignKeyedProperty`, `DefineNamedProperty`, `DefineKeyedProperty`, `CreateObject`, `CreateArray`, `StoreDenseElement`, `LoadDenseElement`, `DeleteProperty`, `In`, `ToPropertyKey`, `CopyDataProperties`, `SetFunctionName`, `CheckObjectCoercible`, `ThrowIfUninitialized`.

This family has the most complex IC machinery. Many of these already have semi-extracted `execute_*_opcode` methods in `crates/vm/src/vm/dispatch/property.rs` (per `wc -l`: 81 KB file). The extraction is mostly mechanical reshape of those.

- [ ] **Step 1: Inspect existing helpers**

  ```sh
  rg -n "pub.*fn execute_" crates/vm/src/vm/dispatch/property.rs | head -30
  ```

- [ ] **Step 2: Create `semantics/property.rs`**

  For each opcode follow the A8/A9 pattern. For IC-heavy opcodes (`op_get_named_property`, `op_set_named_property`, `op_get_keyed_property`, `op_set_keyed_property`), preserve the existing fast-path/slow-path split exactly — DSL-0b will replace these with DSL bodies, but DSL-0a's job is only to lift them out of the α handler. Do not refactor IC layout here.

  Each `OpXxxArgs` carries the decoded operands. The semantic body returns the same outcome shape (`Continue { pc_advance }`, `Refresh`, or `ExitError`).

- [ ] **Step 3: Wire `property` into `semantics/mod.rs`**

- [ ] **Step 4: Thin α handlers in `dispatch_handlers/property.rs`**

- [ ] **Step 5: Register property opcodes in the manifest**

- [ ] **Step 6: Verify build + tests**

  ```sh
  cargo build -p lyng-vm
  cargo test -p lyng-vm -p lyng-bytecode -p lyng-tests 2>&1 | tail -20
  ```
  Expected: clean build; all tests pass. Property-access tests are the most likely to surface extraction errors — investigate any failure before proceeding.

- [ ] **Step 7: Commit**

  ```sh
  git add crates/vm/src/vm/semantics/property.rs \
          crates/vm/src/vm/semantics/mod.rs \
          crates/vm/src/vm/dispatch_handlers/property.rs \
          crates/vm/src/dsl/opcode_manifest.rs
  git commit -m "DSL-0a: extract property family (~21 opcodes) into semantics::property"
  ```

---

### Task A12: Extract `names` family (17 opcodes)

**Files:**
- Create: `crates/vm/src/vm/semantics/names.rs`
- Modify: `crates/vm/src/vm/semantics/mod.rs`
- Modify: `crates/vm/src/vm/dispatch_handlers/names.rs`
- Modify: `crates/vm/src/dsl/opcode_manifest.rs`

**Opcodes in scope:** `LoadGlobal`, `StoreGlobal`, `AssignGlobal`, `DeleteGlobal`, `LoadName`, `ResolveName`, `ResolveGlobal`, `AssignName`, `AssignVariableName`, `DeleteName`, `CaptureName`, `LoadCapturedName`, `LoadCapturedNameThis`, `AssignCapturedName`, `LoadThis`, `LoadCallee`, `LoadNewTarget`.

- [ ] **Step 1: Inspect α handlers**

  ```sh
  rg -n "^pub extern" crates/vm/src/vm/dispatch_handlers/names.rs | head -20
  ```

- [ ] **Step 2: Create `semantics/names.rs`**

  Each opcode: `OpXxxArgs` + `op_xxx_semantic` returning `SemanticOutcome`. Reuse existing helper functions from `crates/vm/src/vm/names.rs` (the 63 KB file containing the semantic implementations).

- [ ] **Step 3: Wire into `semantics/mod.rs`**

- [ ] **Step 4: Thin α handlers**

- [ ] **Step 5: Register in manifest**

- [ ] **Step 6: Verify build + tests**

- [ ] **Step 7: Commit**

  ```sh
  git add crates/vm/src/vm/semantics/names.rs \
          crates/vm/src/vm/semantics/mod.rs \
          crates/vm/src/vm/dispatch_handlers/names.rs \
          crates/vm/src/dsl/opcode_manifest.rs
  git commit -m "DSL-0a: extract names family (17 opcodes) into semantics::names"
  ```

---

### Task A13: Extract `scope` family (10 opcodes)

**Files:**
- Create: `crates/vm/src/vm/semantics/scope.rs`
- Modify: `crates/vm/src/vm/semantics/mod.rs`
- Modify: `crates/vm/src/vm/dispatch_handlers/scope.rs`
- Modify: `crates/vm/src/dsl/opcode_manifest.rs`

**Opcodes in scope:** `LoadEnvSlot`, `StoreEnvSlot`, `AssignEnvSlot`, `EnterEnvScope`, `LeaveEnvScope`, `PushClosureEnv`, `PopClosureEnv`, `PushWithEnv`, `PopWithEnv`, `TypeOf`.

- [ ] **Step 1: Inspect α handlers**

  ```sh
  rg -n "^pub extern" crates/vm/src/vm/dispatch_handlers/scope.rs
  ```

- [ ] **Step 2: Create `semantics/scope.rs`**

  Follow the A8 pattern. `EnterEnvScope` / `LeaveEnvScope` / `Push*Env` / `Pop*Env` may mutate the environment chain — preserve the existing semantics exactly.

- [ ] **Step 3: Wire into `semantics/mod.rs`**

- [ ] **Step 4: Thin α handlers**

- [ ] **Step 5: Register in manifest**

- [ ] **Step 6: Verify build + tests**

- [ ] **Step 7: Commit**

  ```sh
  git add crates/vm/src/vm/semantics/scope.rs \
          crates/vm/src/vm/semantics/mod.rs \
          crates/vm/src/vm/dispatch_handlers/scope.rs \
          crates/vm/src/dsl/opcode_manifest.rs
  git commit -m "DSL-0a: extract scope family (10 opcodes) into semantics::scope"
  ```

---

### Task A14: Extract `calls` family (9 opcodes)

**Files:**
- Create: `crates/vm/src/vm/semantics/calls.rs`
- Modify: `crates/vm/src/vm/semantics/mod.rs`
- Modify: `crates/vm/src/vm/dispatch_handlers/calls.rs`
- Modify: `crates/vm/src/dsl/opcode_manifest.rs`

**Opcodes in scope:** `Call0`, `Call1`, `Call2`, `Call3`, `Call`, `CallMethod`, `TailCall`, `Construct`, `CreateClosure`.

Calls are frame-transitioning. Every successful call's semantic body returns `Refresh` so the dispatcher reloads PC/REGS/FV for the callee frame. `CallMethod` may not exist as a separate α handler today (it's in the Opcode enum but check `dispatch_handlers/mod.rs`'s re-export list); if it's a `op_unimplemented`-only entry, mark it as Cold and provide a stub semantic that returns `ExitError { error: VmError::Unimplemented(Opcode::CallMethod) }` — or extend an existing call handler to cover it.

- [ ] **Step 1: Inspect existing call helpers and α handlers**

  ```sh
  rg -n "^pub extern" crates/vm/src/vm/dispatch_handlers/calls.rs
  rg -n "CallMethod" crates/vm/src/vm/dispatch_handlers/mod.rs
  ```

- [ ] **Step 2: Create `semantics/calls.rs`**

  Each call's semantic body:
  1. Resolves the callee and the argument range.
  2. Validates callability.
  3. Pushes the new frame.
  4. Returns `SemanticOutcome::Refresh` (so the dispatcher picks up the callee's PC/REGS/FV).

  On call failure: `SemanticOutcome::ExitError`. On tail-call: reuse the existing tail-call mechanics from `crates/vm/src/vm/call.rs`.

- [ ] **Step 3: Wire into `semantics/mod.rs`**

- [ ] **Step 4: Thin α handlers**

- [ ] **Step 5: Register in manifest**

- [ ] **Step 6: Verify build + tests**

- [ ] **Step 7: Commit**

  ```sh
  git add crates/vm/src/vm/semantics/calls.rs \
          crates/vm/src/vm/semantics/mod.rs \
          crates/vm/src/vm/dispatch_handlers/calls.rs \
          crates/vm/src/dsl/opcode_manifest.rs
  git commit -m "DSL-0a: extract calls family (9 opcodes) into semantics::calls"
  ```

---

### Task A15: Extract `iterators` family (6 opcodes)

**Files:**
- Create: `crates/vm/src/vm/semantics/iterators.rs`
- Modify: `crates/vm/src/vm/semantics/mod.rs`
- Modify: `crates/vm/src/vm/dispatch_handlers/iterators.rs`
- Modify: `crates/vm/src/dsl/opcode_manifest.rs`

**Opcodes in scope:** `CreateForIn`, `AdvanceForIn`, `CloseForIn`, `CreateIterator`, `AdvanceIterator`, `CloseIterator`.

- [ ] **Step 1: Inspect α handlers**

- [ ] **Step 2: Create `semantics/iterators.rs`**

  Each iterator opcode's semantic body wraps the iterator-protocol calls from `crates/vm/src/vm/loop_iteration.rs`. Preserve the abrupt-completion routing.

- [ ] **Step 3: Wire into `semantics/mod.rs`**

- [ ] **Step 4: Thin α handlers**

- [ ] **Step 5: Register in manifest**

- [ ] **Step 6: Verify build + tests**

- [ ] **Step 7: Commit**

  ```sh
  git add crates/vm/src/vm/semantics/iterators.rs \
          crates/vm/src/vm/semantics/mod.rs \
          crates/vm/src/vm/dispatch_handlers/iterators.rs \
          crates/vm/src/dsl/opcode_manifest.rs
  git commit -m "DSL-0a: extract iterators family (6 opcodes) into semantics::iterators"
  ```

---

### Task A16: Extract `generators` family (6 opcodes)

**Files:**
- Create: `crates/vm/src/vm/semantics/generators.rs`
- Modify: `crates/vm/src/vm/semantics/mod.rs`
- Modify: `crates/vm/src/vm/dispatch_handlers/generators.rs`
- Modify: `crates/vm/src/dsl/opcode_manifest.rs`

**Opcodes in scope:** `SuspendGeneratorStart`, `Yield`, `Await`, `DelegateYield`, `LoadResumeKind`, `LoadResumeValue`.

Generator suspension is the most subtle extraction. `Yield` / `Await` may suspend the current frame and unwind back to the caller — that path uses `SemanticOutcome::ExitDone` or a dedicated variant. Check the existing α handlers carefully: the semantics must preserve coroutine state transitions.

- [ ] **Step 1: Inspect α handlers and existing async support**

  ```sh
  rg -n "^pub extern" crates/vm/src/vm/dispatch_handlers/generators.rs
  wc -l crates/vm/src/vm/async_functions.rs crates/vm/src/vm/generators.rs
  ```

- [ ] **Step 2: Create `semantics/generators.rs`**

  If suspension produces a state shape the current `SemanticOutcome` enum can't express cleanly, *do not extend the enum*. Instead, route suspension through `ExitDone { value: suspension_marker_value }` (preserving today's contract) and let the caller distinguish via the marker. If that doesn't work, add a `SemanticOutcome::Suspended { state }` variant in this task, threading it through `translate_outcome_to_step` to produce the same `Step` shape today's α handlers produce.

- [ ] **Step 3: Wire into `semantics/mod.rs`**

- [ ] **Step 4: Thin α handlers**

- [ ] **Step 5: Register in manifest**

- [ ] **Step 6: Verify build + tests**

  ```sh
  cargo build -p lyng-vm
  cargo test -p lyng-vm -p lyng-bytecode -p lyng-tests 2>&1 | tail -20
  # Plus the focused generator/async test slice:
  cargo run --release -p lyng-test262 -- --filter built-ins/Generator --report /tmp/dsl-0a-generators.md -j 4
  ```
  Expected: no regressions in the Generator subset.

- [ ] **Step 7: Commit**

  ```sh
  git add crates/vm/src/vm/semantics/generators.rs \
          crates/vm/src/vm/semantics/mod.rs \
          crates/vm/src/vm/dispatch_handlers/generators.rs \
          crates/vm/src/dsl/opcode_manifest.rs \
          crates/vm/src/dsl/slow_path.rs
  git commit -m "DSL-0a: extract generators family (6 opcodes) into semantics::generators"
  ```

---

### Task A17: Extract `exceptions` family (4 opcodes)

**Files:**
- Create: `crates/vm/src/vm/semantics/exceptions.rs`
- Modify: `crates/vm/src/vm/semantics/mod.rs`
- Modify: `crates/vm/src/vm/dispatch_handlers/exceptions.rs`
- Modify: `crates/vm/src/dsl/opcode_manifest.rs`

**Opcodes in scope:** `Throw`, `EnterHandler`, `LeaveHandler`, `LoadException`.

`Throw` semantic body either returns `ExitError` (uncaught) or `Refresh` (transferred to a handler in the same frame) or `ExitError` again (cross-frame uncaught after unwind). Use `Vm::transfer_to_exception_handler` (already in `crates/vm/src/vm/exceptions.rs`) to do the routing.

- [ ] **Step 1: Inspect α handlers and exception support**

- [ ] **Step 2: Create `semantics/exceptions.rs`**

- [ ] **Step 3: Wire into `semantics/mod.rs`**

- [ ] **Step 4: Thin α handlers**

- [ ] **Step 5: Register in manifest**

- [ ] **Step 6: Verify build + tests**

- [ ] **Step 7: Commit**

  ```sh
  git add crates/vm/src/vm/semantics/exceptions.rs \
          crates/vm/src/vm/semantics/mod.rs \
          crates/vm/src/vm/dispatch_handlers/exceptions.rs \
          crates/vm/src/dsl/opcode_manifest.rs
  git commit -m "DSL-0a: extract exceptions family (4 opcodes) into semantics::exceptions"
  ```

---

### Task A18: Extract `prefix` family (2 opcodes) + close out remaining

**Files:**
- Create: `crates/vm/src/vm/semantics/prefix.rs`
- Create: `crates/vm/src/vm/semantics/misc.rs` (for `InstanceOf` and any orphan opcodes)
- Modify: `crates/vm/src/vm/semantics/mod.rs`
- Modify: `crates/vm/src/vm/dispatch_handlers/prefix.rs`
- Modify: `crates/vm/src/dsl/opcode_manifest.rs`

**Opcodes in scope:** `Wide`, `ExtraWide`, plus a coverage sweep for any opcode that hasn't landed in a prior family (likely `InstanceOf` from the `arithmetic`-family naming convention — re-check).

- [ ] **Step 1: Identify any opcodes still missing from the manifest**

  ```sh
  cargo test -p lyng-vm dsl::opcode_manifest::manifest_tests::opcodes_manifest_is_exhaustive -- --ignored --nocapture 2>&1 | tail -30
  ```
  Expected: lists any opcodes still missing. Likely: `InstanceOf`, possibly `CallMethod` (depending on A14 disposition).

- [ ] **Step 2: Create `semantics/prefix.rs`**

  ```rust
  use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};

  pub struct OpWideArgs;
  pub struct OpExtraWideArgs;

  pub(crate) fn op_wide_semantic(
      state: &mut LlIntDispatchState<'_, '_>,
      _args: OpWideArgs,
  ) -> SemanticOutcome {
      let inner = state.dispatch_state();
      if inner.prefix.is_some() {
          return SemanticOutcome::ExitError {
              error: crate::error::VmError::DoublePrefix,
          };
      }
      inner.prefix = Some(lyng_bytecode::Opcode::Wide);
      SemanticOutcome::Continue { pc_advance: 1 }
  }

  pub(crate) fn op_extra_wide_semantic(
      state: &mut LlIntDispatchState<'_, '_>,
      _args: OpExtraWideArgs,
  ) -> SemanticOutcome {
      let inner = state.dispatch_state();
      if inner.prefix.is_some() {
          return SemanticOutcome::ExitError {
              error: crate::error::VmError::DoublePrefix,
          };
      }
      inner.prefix = Some(lyng_bytecode::Opcode::ExtraWide);
      SemanticOutcome::Continue { pc_advance: 1 }
  }
  ```

  If `VmError::DoublePrefix` doesn't yet exist, add it as a new variant in `crates/vm/src/error.rs` (and update the existing α handler's error type accordingly).

- [ ] **Step 3: Create `semantics/misc.rs` for any orphans**

  Implement `op_instance_of_semantic` (and `op_call_method_semantic` if needed) similarly. If the existing α handler is `op_unimplemented`, the semantic body returns `ExitError { error: VmError::Unimplemented(Opcode::InstanceOf) }`.

- [ ] **Step 4: Wire into `semantics/mod.rs`**

  ```rust
  pub(crate) mod prefix;
  pub(crate) mod misc;
  ```

- [ ] **Step 5: Thin α prefix handlers and any orphan handlers**

- [ ] **Step 6: Register prefix + misc opcodes in the manifest**

- [ ] **Step 7: Enable Manifest Tests 1 and 4**

  In `crates/vm/src/dsl/opcode_manifest.rs`, remove the `#[ignore]` attribute from the exhaustive-coverage test.
  In `crates/vm/tests/dsl_manifest_grep.rs`, remove the `#[ignore]` attribute.

- [ ] **Step 8: Run both tests**

  ```sh
  cargo test -p lyng-vm dsl::opcode_manifest::manifest_tests::opcodes_manifest_is_exhaustive
  cargo test -p lyng-vm --test dsl_manifest_grep
  ```
  Expected: both PASS. If Test 4 catches an `op_*` function outside the allowlist, either move it to `semantics/` or extend the allowlist with a documented exception.

- [ ] **Step 9: Full test suite + Test262**

  ```sh
  cargo test -p lyng-vm -p lyng-bytecode -p lyng-objects -p lyng-tests -p lyng-compiler
  cargo run --release -p lyng-test262 -- --report /tmp/dsl-0a-test262.md -j 4
  ```
  Expected: focused suites pass count matches Pre-flight 4. Test262 pass count ≥ Pre-flight 7 baseline.

- [ ] **Step 10: Commit**

  ```sh
  git add crates/vm/src/vm/semantics/prefix.rs \
          crates/vm/src/vm/semantics/misc.rs \
          crates/vm/src/vm/semantics/mod.rs \
          crates/vm/src/vm/dispatch_handlers/prefix.rs \
          crates/vm/src/dsl/opcode_manifest.rs \
          crates/vm/tests/dsl_manifest_grep.rs \
          crates/vm/src/error.rs
  git commit -m "DSL-0a: extract prefix + misc, close manifest coverage, enable Tests 1 + 4"
  ```

---

### Task A19: Manifest Test 2 — `semantic_symbol` linker resolution

**Files:**
- Modify: `crates/vm/src/dsl/opcode_manifest.rs`

This test verifies every `OpcodeEntry.semantic_symbol` actually resolves to a function at link time. The simplest way is to call each function via a generated `fn ptr` (with synthetic dummy arguments via `unreachable!()` branches that never run) — but that's overkill. A cleaner approach: a static `&[(&str, fn(&mut LlIntDispatchState<'_, '_>, /* boxed args */ ()) -> ())]`-shaped registry where every semantic body is type-erased to a dummy signature.

Pragmatic alternative used by JSC's offlineasm output: just emit a `#[used]` static const reference to each function name's address, forcing the linker to keep them. Walking `OPCODES` and matching strings against `std::any::type_name_of_val` isn't easy in Rust — instead, build the manifest registration so that the symbol name comes from `type_name_of_val(&op_xxx_semantic)` at compile time.

Easier: a direct second const slice that holds the function pointers themselves, with one entry per opcode. The exhaustive-coverage test already checks the length of `OPCODES`; the new test verifies the function-pointer slice's length matches and each pointer is non-null.

- [ ] **Step 1: Add the function-pointer registry**

  At the bottom of `crates/vm/src/dsl/opcode_manifest.rs`:
  ```rust
  /// Type-erased semantic function pointer. Each opcode has a unique
  /// concrete signature but the linker-resolution test only needs to
  /// know the pointer is non-null.
  pub type SemanticFnPtr = *const ();

  /// Parallel slice to `OPCODES` holding the type-erased function pointer
  /// for each `op_xxx_semantic`. Maintained by family-extraction tasks.
  ///
  /// SAFETY: each pointer is derived from a real Rust function via `as
  /// *const ()`; the linker resolves it at build time.
  pub static SEMANTIC_FN_PTRS: &[SemanticFnPtr] = &[
      // Populated by family-extraction tasks A8–A18.
  ];
  ```

  In each family extraction (re-touched in this task), add a function-pointer cast to `SEMANTIC_FN_PTRS`. E.g. for `op_move`:
  ```rust
  crate::vm::semantics::loads::op_move_semantic
      as fn(&mut LlIntDispatchState<'_, '_>, crate::vm::semantics::loads::OpMoveArgs) -> SemanticOutcome
      as *const (),
  ```

- [ ] **Step 2: Add the Test 2 body**

  In `manifest_tests`:
  ```rust
  #[test]
  fn semantic_fn_ptrs_resolve() {
      assert_eq!(
          SEMANTIC_FN_PTRS.len(),
          OPCODES.len(),
          "SEMANTIC_FN_PTRS has {} entries, OPCODES has {}",
          SEMANTIC_FN_PTRS.len(),
          OPCODES.len(),
      );
      for (idx, ptr) in SEMANTIC_FN_PTRS.iter().enumerate() {
          assert!(
              !ptr.is_null(),
              "SEMANTIC_FN_PTRS[{idx}] is null (opcode = {:?})",
              OPCODES[idx].opcode,
          );
      }
  }
  ```

- [ ] **Step 3: Run the test**

  ```sh
  cargo test -p lyng-vm dsl::opcode_manifest::manifest_tests::semantic_fn_ptrs_resolve 2>&1 | tail -10
  ```
  Expected: PASS.

- [ ] **Step 4: Commit**

  ```sh
  git add crates/vm/src/dsl/opcode_manifest.rs
  git commit -m "DSL-0a: manifest Test 2 (semantic fn-ptr linker resolution)"
  ```

---

### Task A20: DSL-0a exit gate

**Files:** none modified; produces evidence.

- [ ] **Step 1: Run all manifest tests**

  ```sh
  cargo test -p lyng-vm dsl::opcode_manifest
  cargo test -p lyng-vm --test dsl_manifest_grep
  ```
  Expected: all pass.

- [ ] **Step 2: Run full focused tests + clippy**

  ```sh
  cargo test -p lyng-vm -p lyng-bytecode -p lyng-objects -p lyng-tests -p lyng-compiler
  cargo clippy --all-targets --all-features -- -W clippy::pedantic -W clippy::nursery 2>&1 | tail -40
  ```
  Expected: focused suites pass; clippy warnings reviewed and either fixed or `allow`-justified inline.

- [ ] **Step 3: Run Test262 whole-corpus**

  ```sh
  cargo run --release -p lyng-test262 -- --report /tmp/dsl-0a-test262.md -j 4
  cp /tmp/dsl-0a-test262.md reports/lyng/dsl-0a-test262.md
  ```
  Expected: pass count ≥ Pre-flight 7 baseline.

- [ ] **Step 4: Run V8 v7 to confirm no regression**

  ```sh
  cargo run --release -p lyng-bench -- v8suite --report /tmp/dsl-0a-v8.md --json /tmp/dsl-0a-v8.json
  cp /tmp/dsl-0a-v8.md reports/lyng/dsl-0a-v8.md
  cp /tmp/dsl-0a-v8.json reports/lyng/dsl-0a-v8.json
  ```
  Expected: geomean within ±2% of pre-DSL-0 baseline (acceptable — semantic extraction adds one indirection through `translate_outcome_to_step` per opcode).

- [ ] **Step 5: Write DSL-0a status report**

  Create `reports/lyng/dsl-0a-status.md`:
  ```markdown
  # DSL-0a status

  ## Deliverables

  | Deliverable | Status | Path |
  | --- | --- | --- |
  | Semantics family: loads | done | crates/vm/src/vm/semantics/loads.rs |
  | Semantics family: arithmetic | done | crates/vm/src/vm/semantics/arithmetic.rs |
  | Semantics family: control_flow | done | crates/vm/src/vm/semantics/control_flow.rs |
  | Semantics family: property | done | crates/vm/src/vm/semantics/property.rs |
  | Semantics family: names | done | crates/vm/src/vm/semantics/names.rs |
  | Semantics family: scope | done | crates/vm/src/vm/semantics/scope.rs |
  | Semantics family: calls | done | crates/vm/src/vm/semantics/calls.rs |
  | Semantics family: iterators | done | crates/vm/src/vm/semantics/iterators.rs |
  | Semantics family: generators | done | crates/vm/src/vm/semantics/generators.rs |
  | Semantics family: exceptions | done | crates/vm/src/vm/semantics/exceptions.rs |
  | Semantics family: prefix | done | crates/vm/src/vm/semantics/prefix.rs |
  | Semantics family: misc | done | crates/vm/src/vm/semantics/misc.rs |
  | Manifest Test 1 (exhaustive coverage) | passing | crates/vm/src/dsl/opcode_manifest.rs |
  | Manifest Test 2 (semantic fn-ptr resolution) | passing | crates/vm/src/dsl/opcode_manifest.rs |
  | Manifest Test 4 (source-grep) | passing | crates/vm/tests/dsl_manifest_grep.rs |
  | Transitional LlIntDispatchState wrapper | done | crates/vm/src/dsl/slow_path.rs |

  ## Test262 evidence

  - Pre-DSL-0a baseline: <N> passing.
  - Post-DSL-0a: <M> passing. Δ: <delta>.

  Report: reports/lyng/dsl-0a-test262.md.

  ## V8 v7 evidence

  - Pre-DSL-0a baseline geomean: <X>.
  - Post-DSL-0a geomean: <Y>. Δ: <delta>%.

  Report: reports/lyng/dsl-0a-v8.md.

  ## Hand-off to DSL-0b

  DSL-0b lands the `lyng-vm-dsl` proc-macro, the `vm/src/dsl/` runtime
  ABI (`LlIntState`, `LlIntRustContext`, slow-path bridge), the AArch64
  backend, the FeedbackVector flat-array refactor, the 9 validation
  cases, the 5 hot ports, the 5 warm ports, and ~140 cold stubs.

  Plan: docs/superpowers/plans/2026-05-17-dsl-0-spike-and-semantic-extraction.md
  ```

  Fill in the `<N>` / `<M>` / `<X>` / `<Y>` placeholders with the actual numbers from steps 3–4.

- [ ] **Step 6: Update the dcat parent ticket**

  ```sh
  dcat update <DSL0A> --status in_review
  ```

- [ ] **Step 7: Commit**

  ```sh
  git add reports/lyng/dsl-0a-status.md \
          reports/lyng/dsl-0a-test262.md \
          reports/lyng/dsl-0a-v8.md \
          reports/lyng/dsl-0a-v8.json
  git commit -m "DSL-0a: exit-gate status report + post-extraction evidence"
  ```

- [ ] **Step 8: Notify the user**

  Surface message: "DSL-0a complete. All 152 opcodes extracted; manifest Tests 1, 2, and 4 passing; Test262 not regressed; V8 v7 within ±2% of pre-DSL-0 baseline. Status report at `reports/lyng/dsl-0a-status.md`. May I proceed to DSL-0b? (DSL-0a dcat ticket is in_review; close gating on your approval.)"

---

## Phase B — DSL-0b: DSL infrastructure + hot/warm/cold ports (Tasks B1–B50)

**Goal:** Land the proc-macro crate, the `vm/src/dsl/` runtime ABI, the AArch64 backend, the FeedbackVector flat-array refactor (prerequisite for the `FV` pin), all 9 validation cases as committed tests, 5 hot DSL handler ports, 5 warm handler ports, and ~140 cold stubs covering the remaining opcodes. The DSL dispatch table is fully populated by end of DSL-0b but **not yet active** — alpha remains the dispatch path until DSL-0c flips the switch.

**Estimated duration:** 4–5 weeks.

**Exit criteria** (verified by Task B50):
1. `lyng-vm-dsl` crate compiles; `llint_handler!` expands real opcodes.
2. All 9 DSL-0b validation cases pass as committed tests.
3. 5 hot handlers + 5 warm handlers + ~140 cold stubs exist and route through the DSL dispatch table (which is populated but inactive).
4. FeedbackVector flat-array refactor lands without regressing IC fast-path behavior.
5. Manifest entries' `dsl_handler_symbol` strings name real DSL symbols.
6. `cargo build --release -p lyng-vm` is clean.
7. `cargo test -p lyng-vm` passes (alpha is still the active path; DSL handlers are dead code per the still-default dispatch table assembly).

---

### Task B1: Create `lyng-vm-dsl` proc-macro crate

**Files:**
- Create: `crates/vm-dsl/Cargo.toml`
- Create: `crates/vm-dsl/src/lib.rs`
- Modify: `Cargo.toml` (workspace)

- [ ] **Step 1: Create crate directory and Cargo.toml**

  Create `crates/vm-dsl/Cargo.toml`:
  ```toml
  [package]
  name = "lyng-vm-dsl"
  version = "0.1.0"
  edition = "2021"
  description = "Proc-macro crate for the lyng asm-DSL interpreter substrate"
  license = "MIT"

  [lib]
  proc-macro = true

  [dependencies]
  proc-macro2 = "1"
  syn = { version = "2", features = ["full", "extra-traits"] }
  quote = "1"
  ```

- [ ] **Step 2: Create initial lib.rs**

  Create `crates/vm-dsl/src/lib.rs`:
  ```rust
  //! Proc-macro crate emitting #[unsafe(naked)] extern "C" fn DSL handlers.
  //!
  //! `llint_handler!` parses an offlineasm-flavored handler body (see
  //! `docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md` §4) and
  //! lowers it to a single `core::arch::naked_asm!` block.
  //!
  //! Submodules:
  //! - `parse`: syn-based AST for handler signatures + bodies.
  //! - `layouts`: operand-layout descriptors (Abc, AbcSlot, Abx, Ax, ...).
  //! - `scratch`: compile-time scratch-register allocator.
  //! - `lower`: AST → naked_asm! string assembly.

  use proc_macro::TokenStream;

  mod layouts;
  mod lower;
  mod parse;
  mod scratch;

  /// Define a DSL handler.
  ///
  /// Syntax (see docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md §4):
  ///
  /// ```ignore
  /// llint_handler! {
  ///     op_add, layout = AbcSlot, length = 6, |a, b, c, slot| {
  ///         load_reg!(b => t0);
  ///         check_smi!(t0, .slow);
  ///         load_reg!(c => t1);
  ///         check_smi!(t1, .slow);
  ///         add_smi_overflow!(t0, t1 => t2, .slow);
  ///         store_reg!(a, t2);
  ///         record_smi!(slot);
  ///         dispatch!();
  ///
  ///       .slow:
  ///         call_slow!(op_add_slow_rs, args = [a, b, c, slot]);
  ///         dispatch_after_slow!();
  ///     }
  /// }
  /// ```
  #[proc_macro]
  pub fn llint_handler(input: TokenStream) -> TokenStream {
      match parse::parse_handler(input.into()).and_then(lower::lower_handler) {
          Ok(tokens) => tokens.into(),
          Err(err) => err.to_compile_error().into(),
      }
  }
  ```

- [ ] **Step 3: Add the crate to the workspace**

  Open the workspace root `Cargo.toml`. Add `"crates/vm-dsl"` to the `members` array in the `[workspace]` section, sorted alphabetically.

- [ ] **Step 4: Create empty submodule files**

  Create:
  - `crates/vm-dsl/src/parse.rs` with a TODO stub:
    ```rust
    use proc_macro2::TokenStream;
    use syn::Result;

    pub(crate) struct HandlerAst {
        // Populated by Task B2.
        pub(crate) _placeholder: (),
    }

    pub(crate) fn parse_handler(_input: TokenStream) -> Result<HandlerAst> {
        Err(syn::Error::new(proc_macro2::Span::call_site(), "llint_handler! parser stub — Task B2"))
    }
    ```
  - `crates/vm-dsl/src/layouts.rs` with a TODO stub.
  - `crates/vm-dsl/src/scratch.rs` with a TODO stub.
  - `crates/vm-dsl/src/lower.rs`:
    ```rust
    use proc_macro2::TokenStream;
    use syn::Result;

    use crate::parse::HandlerAst;

    pub(crate) fn lower_handler(_ast: HandlerAst) -> Result<TokenStream> {
        Err(syn::Error::new(proc_macro2::Span::call_site(), "llint_handler! lowerer stub — Task B5"))
    }
    ```

- [ ] **Step 5: Verify the crate compiles**

  ```sh
  cargo build -p lyng-vm-dsl
  ```
  Expected: clean build of the (no-op) proc-macro crate.

- [ ] **Step 6: Commit**

  ```sh
  git add crates/vm-dsl/ Cargo.toml
  git commit -m "DSL-0b: lyng-vm-dsl proc-macro crate scaffold"
  ```

---

### Task B2: Proc-macro — handler-body parser

**Files:**
- Modify: `crates/vm-dsl/src/parse.rs`

Parses the input of `llint_handler! { name, layout = X, length = N, |args| { body } }` into an AST containing: handler name, layout descriptor, instruction length, named operand bindings, body statements (each statement is one DSL operation invocation or a label).

- [ ] **Step 1: Define the AST**

  Replace `crates/vm-dsl/src/parse.rs`:
  ```rust
  use proc_macro2::{Span, TokenStream};
  use syn::{
      braced, parenthesized, parse::{Parse, ParseStream}, punctuated::Punctuated,
      Expr, Ident, LitInt, Result, Token,
  };

  pub(crate) struct HandlerAst {
      pub(crate) name: Ident,
      pub(crate) layout: Ident,
      pub(crate) length: LitInt,
      pub(crate) operand_idents: Punctuated<Ident, Token![,]>,
      pub(crate) body: TokenStream,
  }

  impl Parse for HandlerAst {
      fn parse(input: ParseStream) -> Result<Self> {
          let name: Ident = input.parse()?;
          input.parse::<Token![,]>()?;
          let layout_ident: Ident = input.parse()?;  // `layout`
          if layout_ident != "layout" {
              return Err(syn::Error::new(layout_ident.span(), "expected `layout = ...`"));
          }
          input.parse::<Token![=]>()?;
          let layout: Ident = input.parse()?;
          input.parse::<Token![,]>()?;
          let length_ident: Ident = input.parse()?;
          if length_ident != "length" {
              return Err(syn::Error::new(length_ident.span(), "expected `length = ...`"));
          }
          input.parse::<Token![=]>()?;
          let length: LitInt = input.parse()?;
          input.parse::<Token![,]>()?;
          // |a, b, c| { ... }
          input.parse::<Token![|]>()?;
          let operand_idents = Punctuated::parse_separated_nonempty(input)?;
          input.parse::<Token![|]>()?;
          let body_content;
          braced!(body_content in input);
          let body: TokenStream = body_content.parse()?;
          Ok(HandlerAst { name, layout, length, operand_idents, body })
      }
  }

  pub(crate) fn parse_handler(input: TokenStream) -> Result<HandlerAst> {
      syn::parse2(input)
  }
  ```

- [ ] **Step 2: Verify**

  ```sh
  cargo build -p lyng-vm-dsl
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm-dsl/src/parse.rs
  git commit -m "DSL-0b: proc-macro handler-body parser"
  ```

---

### Task B3: Proc-macro — operand-layout descriptors

**Files:**
- Modify: `crates/vm-dsl/src/layouts.rs`

The layout enum drives the operand-decode prologue. Each variant maps to one of the existing operand layouts in `crates/bytecode/src/instruction.rs`.

- [ ] **Step 1: Implement layout enum and decode-prologue emitter**

  ```rust
  use proc_macro2::Span;
  use syn::{Error, Ident, Result};

  #[derive(Clone, Copy)]
  pub(crate) enum Layout {
      Abc,
      AbcSlot,
      Abx,
      Ax,
      Ab,
      A,
      None,
      // ... extend per existing OperandLayout in lyng-bytecode
  }

  impl Layout {
      pub(crate) fn from_ident(ident: &Ident) -> Result<Self> {
          match ident.to_string().as_str() {
              "Abc" => Ok(Self::Abc),
              "AbcSlot" => Ok(Self::AbcSlot),
              "Abx" => Ok(Self::Abx),
              "Ax" => Ok(Self::Ax),
              "Ab" => Ok(Self::Ab),
              "A" => Ok(Self::A),
              "None" => Ok(Self::None),
              other => Err(Error::new(
                  ident.span(),
                  format!("unknown layout `{other}`"),
              )),
          }
      }

      pub(crate) fn operand_arity(self) -> usize {
          match self {
              Self::None => 0,
              Self::A => 1,
              Self::Ab => 2,
              Self::Abc | Self::Abx | Self::Ax => 3,
              Self::AbcSlot => 4,
          }
      }

      pub(crate) fn decode_prologue_asm(self, _operands: &[Ident]) -> String {
          // Emits the operand-decode asm fragment (no prefix path; that is
          // a separate prefix-dispatch routine). Operand names are mapped
          // to scratch regs by the scratch allocator (Task B4).
          match self {
              Self::Abc => "// decode_abc prologue placeholder\n".to_string(),
              Self::AbcSlot => "// decode_abc_slot prologue placeholder\n".to_string(),
              // ... real fragments filled in by Task B5 lowering pass.
              _ => "// decode prologue placeholder\n".to_string(),
          }
      }
  }
  ```

- [ ] **Step 2: Verify**

  ```sh
  cargo build -p lyng-vm-dsl
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm-dsl/src/layouts.rs
  git commit -m "DSL-0b: proc-macro layout enum + decode-prologue stubs"
  ```

---

### Task B4: Proc-macro — scratch-register allocator

**Files:**
- Modify: `crates/vm-dsl/src/scratch.rs`

Maps named operand identifiers and DSL-internal scratch variables (e.g. `t0..t6`) to AArch64 register numbers. Errors at expand time if a handler asks for more scratch than the per-arch budget.

- [ ] **Step 1: Implement allocator**

  ```rust
  use std::collections::HashMap;
  use syn::{Error, Ident, Result};

  pub(crate) struct ScratchAllocator {
      // Operand names (a, b, c, slot) and internal scratch (t0..t6) all
      // map to AArch64 caller-saved regs x9..x15. Budget: 7 scratch regs.
      map: HashMap<String, u8>,
      next: u8,
  }

  impl ScratchAllocator {
      const BUDGET: u8 = 7;
      const FIRST: u8 = 9;

      pub(crate) fn new() -> Self {
          Self { map: HashMap::new(), next: 0 }
      }

      pub(crate) fn assign(&mut self, name: &Ident) -> Result<u8> {
          if let Some(&reg) = self.map.get(&name.to_string()) {
              return Ok(reg);
          }
          if self.next >= Self::BUDGET {
              return Err(Error::new(
                  name.span(),
                  format!(
                      "DSL handler exceeded scratch-register budget of {} (would assign x{} to `{}`)",
                      Self::BUDGET,
                      Self::FIRST + self.next,
                      name,
                  ),
              ));
          }
          let reg = Self::FIRST + self.next;
          self.next += 1;
          self.map.insert(name.to_string(), reg);
          Ok(reg)
      }

      pub(crate) fn lookup(&self, name: &str) -> Option<u8> {
          self.map.get(name).copied()
      }
  }
  ```

- [ ] **Step 2: Verify**

  ```sh
  cargo build -p lyng-vm-dsl
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm-dsl/src/scratch.rs
  git commit -m "DSL-0b: proc-macro scratch-register allocator"
  ```

---

### Task B5: Proc-macro — lowerer (AST → `naked_asm!` body)

**Files:**
- Modify: `crates/vm-dsl/src/lower.rs`

The lowerer is where the design's "DSL surface ≈ asm shape" decision pays out: each body statement is one DSL-op invocation, and the lowerer concatenates its asm fragment into a single string passed to `naked_asm!`.

- [ ] **Step 1: Implement the lowerer skeleton**

  ```rust
  use proc_macro2::{Span, TokenStream};
  use quote::quote;
  use syn::Result;

  use crate::layouts::Layout;
  use crate::parse::HandlerAst;
  use crate::scratch::ScratchAllocator;

  pub(crate) fn lower_handler(ast: HandlerAst) -> Result<TokenStream> {
      let layout = Layout::from_ident(&ast.layout)?;
      let operands: Vec<_> = ast.operand_idents.iter().cloned().collect();
      if operands.len() != layout.operand_arity() {
          return Err(syn::Error::new(
              ast.layout.span(),
              format!(
                  "layout {} has arity {}, got {} operand bindings",
                  ast.layout,
                  layout.operand_arity(),
                  operands.len(),
              ),
          ));
      }

      let mut scratch = ScratchAllocator::new();
      for name in &operands {
          scratch.assign(name)?;
      }

      // The body is currently raw TokenStream — the proc-macro doesn't
      // fully expand DSL operation macros itself. Instead it wraps the
      // body in `naked_asm!` and lets the per-arch macros (defined as
      // `macro_rules!` in crates/vm/src/dsl/backend/aarch64/)
      // expand their asm fragments via `concat!`.
      //
      // The lowerer's job here:
      //   1. Emit `#[unsafe(naked)] pub extern "C" fn <name>() -> ! { ... }`.
      //   2. Inside, emit the decode prologue (per layout).
      //   3. Then the user-provided body (raw — backend macros expand it).
      //   4. Auto-append dispatch trailer if the body doesn't end in `dispatch!`.
      //
      // For DSL-0b the lowerer emits a minimal viable handler that runs
      // through `naked_asm!` with the body tokens unchanged. The proc-macro
      // does NOT manually resolve macro_rules — naked_asm! itself
      // tolerates string literal concatenation through `concat!`.

      let name = &ast.name;
      let length = &ast.length;
      let prologue = layout.decode_prologue_asm(&operands);
      let body = &ast.body;

      Ok(quote! {
          #[unsafe(naked)]
          pub extern "C" fn #name() -> ! {
              ::core::arch::naked_asm!(
                  #prologue,
                  // The body is interpolated as a sequence of `concat!`-string
                  // expressions. Each backend macro_rules! macro returns a
                  // string literal usable inside `naked_asm!`.
                  #body
                  options(noreturn),
                  length = const #length as u32,
              )
          }
      })
  }
  ```

  **Risk acknowledged.** Whether `naked_asm!` accepts `concat!`-generated string-literal templates from interpolated macros is the load-bearing question of the spike. Validation case B30 will catch this. If `naked_asm!` rejects the macro-string composition, the lowerer must instead expand DSL macros into a single string at proc-macro time and emit that as the `naked_asm!` template. That's a refactor inside `lower.rs`; the public DSL surface stays the same.

- [ ] **Step 2: Verify the proc-macro crate compiles**

  ```sh
  cargo build -p lyng-vm-dsl
  ```

- [ ] **Step 3: Smoke-test with a trivial use site**

  In `crates/vm/src/dsl/`, add a temporary file `_smoke_test.rs` (excluded from `mod.rs`):
  ```rust
  use lyng_vm_dsl::llint_handler;

  llint_handler! {
      op_smoke, layout = None, length = 1, || {
          // No body — just dispatch.
      }
  }
  ```
  Add `_smoke_test.rs` to a test target or behind `#[cfg(any())]` so it doesn't interfere. Confirm:
  ```sh
  cargo build -p lyng-vm
  ```

- [ ] **Step 4: Remove the smoke-test file**

  ```sh
  rm crates/vm/src/dsl/_smoke_test.rs
  ```

- [ ] **Step 5: Commit**

  ```sh
  git add crates/vm-dsl/src/lower.rs
  git commit -m "DSL-0b: proc-macro lowerer (AST → naked_asm! body)"
  ```

---

### Task B6: `reg_convention.rs` — pinned-register documentation

**Files:**
- Create: `crates/vm/src/dsl/reg_convention.rs`
- Modify: `crates/vm/src/dsl/mod.rs`

- [ ] **Step 1: Author the convention doc + const stubs**

  Create `crates/vm/src/dsl/reg_convention.rs`:
  ```rust
  //! Pinned-register convention for the asm-DSL substrate.
  //!
  //! Authoritative source: design §5 of
  //! docs/lyng/2026-05-16-asm-dsl-llint-interpreter-design.md
  //! and reports/lyng/llint-dsl-abi.md.
  //!
  //! AArch64 mapping:
  //!
  //! | Pin     | Reg | Type                       |
  //! | ------- | --- | -------------------------- |
  //! | PC      | x19 | *const u8                  |
  //! | REGS    | x20 | *mut Value                 |
  //! | FV      | x21 | *mut FeedbackEntry         |
  //! | VM      | x22 | *mut Vm                    |
  //! | TABLE   | x23 | *const DslHandler          |
  //! | STATE   | x24 | *mut LlIntState            |
  //! | t0..t6  | x9..x15 | scratch (caller-saved) |
  //!
  //! Refresh discipline (slow-path call):
  //!   PRE:   state.frame_pc_offset ← PC - pb_base
  //!   POST:  if Refresh: PC/REGS/FV reloaded from state.frame_*
  //!
  //! Const offsets below are populated by Task B7 using offset_of!.

  // Placeholders; resolved to concrete values in Task B7.
  pub const LLINT_STATE_FRAME_PC_OFFSET: usize = 0;
  pub const LLINT_STATE_FRAME_PB_BASE: usize = 0;
  pub const LLINT_STATE_FRAME_REGS_BASE: usize = 0;
  pub const LLINT_STATE_FRAME_FV_BASE: usize = 0;
  pub const LLINT_STATE_PREFIX: usize = 0;
  pub const VM_POLL_PENDING_OFFSET: usize = 0;
  pub const VM_OPCODE_COUNTER_OFFSET: usize = 0;
  ```

- [ ] **Step 2: Wire into `dsl/mod.rs`**

  ```rust
  pub mod reg_convention;
  ```

- [ ] **Step 3: Verify**

  ```sh
  cargo build -p lyng-vm
  ```

- [ ] **Step 4: Commit**

  ```sh
  git add crates/vm/src/dsl/reg_convention.rs \
          crates/vm/src/dsl/mod.rs
  git commit -m "DSL-0b: pinned-register convention documentation + const offset stubs"
  ```

---

### Task B7: `llint_state.rs` — `LlIntState` repr(C)

**Files:**
- Create: `crates/vm/src/dsl/llint_state.rs`
- Modify: `crates/vm/src/dsl/mod.rs`
- Modify: `crates/vm/src/dsl/reg_convention.rs`

- [ ] **Step 1: Define `LlIntState` per design §5**

  Create `crates/vm/src/dsl/llint_state.rs`:
  ```rust
  //! asm-visible state record + Rust-only context per design §5.

  use lyng_types::Value;
  use crate::dsl::feedback_flat::FeedbackEntry;

  /// Opaque marker for the Rust-side context pointer in `LlIntState`.
  /// The asm layer never reads through this pointer.
  #[repr(C)]
  pub struct LlIntRustContextOpaque {
      _private: [u8; 0],
  }

  /// asm-visible per-frame state. Stable across rustc versions because
  /// it contains only thin pointers + integers.
  #[repr(C)]
  pub struct LlIntState {
      pub frame_pc_offset:   u32,
      pub _pad1:             u32,
      pub frame_pb_base:     *const u8,
      pub frame_regs_base:   *mut Value,
      pub frame_fv_base:     *mut FeedbackEntry,
      pub frame_depth:       u32,
      pub frame_check_epoch: u32,
      pub rust_context:      *mut LlIntRustContextOpaque,
      pub prefix:            u8,
      pub _pad2:             [u8; 7],
  }
  ```

- [ ] **Step 2: Wire into `dsl/mod.rs`**

  ```rust
  pub mod llint_state;
  ```

- [ ] **Step 3: Update `reg_convention.rs` to use offset_of!**

  Replace the placeholders:
  ```rust
  use core::mem::offset_of;
  use crate::dsl::llint_state::LlIntState;

  pub const LLINT_STATE_FRAME_PC_OFFSET: usize  = offset_of!(LlIntState, frame_pc_offset);
  pub const LLINT_STATE_FRAME_PB_BASE: usize    = offset_of!(LlIntState, frame_pb_base);
  pub const LLINT_STATE_FRAME_REGS_BASE: usize  = offset_of!(LlIntState, frame_regs_base);
  pub const LLINT_STATE_FRAME_FV_BASE: usize    = offset_of!(LlIntState, frame_fv_base);
  pub const LLINT_STATE_PREFIX: usize           = offset_of!(LlIntState, prefix);
  // VM offsets stay placeholder until Vm gains explicit fields for poll
  // and counter base (Tasks B17 / B27 ensure those exist).
  ```

- [ ] **Step 4: Add offset-generation test**

  Append to `llint_state.rs`:
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      use crate::dsl::reg_convention as r;

      #[test]
      fn ll_int_state_offsets_stable() {
          // Confirm the offset_of! values match the documented contract.
          // Values referenced in reports/lyng/llint-dsl-abi.md §X.
          assert_eq!(r::LLINT_STATE_FRAME_PC_OFFSET, 0);
          assert_eq!(r::LLINT_STATE_FRAME_PB_BASE, 8);
          assert_eq!(r::LLINT_STATE_FRAME_REGS_BASE, 16);
          assert_eq!(r::LLINT_STATE_FRAME_FV_BASE, 24);
          // ... up through `prefix`.
          assert_eq!(std::mem::size_of::<LlIntState>(), 56);
      }
  }
  ```

  Update the expected values if `offset_of!` reports different numbers — the test's purpose is to catch drift, so the first-run values become the contract.

- [ ] **Step 5: Verify**

  ```sh
  cargo build -p lyng-vm
  cargo test -p lyng-vm dsl::llint_state::tests
  ```

- [ ] **Step 6: Commit**

  ```sh
  git add crates/vm/src/dsl/llint_state.rs \
          crates/vm/src/dsl/reg_convention.rs \
          crates/vm/src/dsl/mod.rs
  git commit -m "DSL-0b: LlIntState repr(C) + offset-generation test"
  ```

---

### Task B8: `llint_state.rs` — `LlIntRustContext` + `LlIntExitSlot`

**Files:**
- Modify: `crates/vm/src/dsl/llint_state.rs`

- [ ] **Step 1: Add the Rust-only context types**

  Append:
  ```rust
  use std::sync::Arc;
  use lyng_env::Agent;
  use lyng_host::HostHooks;
  use lyng_objects::NativeFunctionRegistry;
  use crate::vm::install::InstalledFunction;
  use crate::FrameRecord;
  use crate::error::VmError;

  pub struct LlIntRustContext<'vm> {
      pub vm:          &'vm mut crate::vm::Vm,
      pub agent:       &'vm mut Agent,
      pub host:        &'vm dyn HostHooks,
      pub registry:    &'vm mut (dyn NativeFunctionRegistry + 'vm),
      pub installed:   Arc<InstalledFunction>,
      pub frame:       FrameRecord,
      pub frame_depth: usize,
      pub exit:        LlIntExitSlot,
  }

  pub struct LlIntExitSlot {
      pub kind:       ExitKind,
      pub done_value: lyng_types::Value,
      pub error:      Option<Box<VmError>>,
  }

  #[derive(Clone, Copy, PartialEq, Eq, Debug)]
  pub enum ExitKind {
      None,
      Done,
      Error,
  }

  impl Default for LlIntExitSlot {
      fn default() -> Self {
          Self {
              kind:       ExitKind::None,
              done_value: lyng_types::Value::undefined(),
              error:      None,
          }
      }
  }
  ```

- [ ] **Step 2: Verify**

  ```sh
  cargo build -p lyng-vm
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/src/dsl/llint_state.rs
  git commit -m "DSL-0b: LlIntRustContext + LlIntExitSlot + ExitKind"
  ```

---

### Task B9: `slow_path.rs` — `SlowPathReturn` + `SlowPathTag`

**Files:**
- Modify: `crates/vm/src/dsl/slow_path.rs`

- [ ] **Step 1: Add the asm-facing return ABI**

  Append:
  ```rust
  #[repr(C)]
  pub struct SlowPathReturn {
      pub tag:     u64,
      pub payload: u64,
  }

  #[repr(u64)]
  pub enum SlowPathTag {
      Continue = 0,
      Refresh  = 1,
      Exit     = 2,
  }
  ```

- [ ] **Step 2: Verify**

  ```sh
  cargo build -p lyng-vm
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/src/dsl/slow_path.rs
  git commit -m "DSL-0b: SlowPathReturn + SlowPathTag asm-facing ABI"
  ```

---

### Task B10: `slow_path.rs` — `LlIntDispatchState::from_raw` (asm path)

**Files:**
- Modify: `crates/vm/src/dsl/slow_path.rs`

- [ ] **Step 1: Add the Asm variant + `from_raw`**

  Update the `LlIntDispatchInner` enum and add the asm-path constructor:
  ```rust
  use crate::dsl::llint_state::{LlIntState, LlIntRustContext};

  pub(crate) enum LlIntDispatchInner<'vm, 'borrow> {
      Alpha(&'borrow mut crate::vm::dispatch_state::DispatchState<'vm>),
      Asm {
          state: *mut LlIntState,
          rust:  &'borrow mut LlIntRustContext<'vm>,
      },
  }

  impl<'vm, 'borrow> LlIntDispatchState<'vm, 'borrow> {
      /// Construct from a raw `*mut LlIntState` passed by the asm shim.
      ///
      /// # Safety
      ///
      /// Caller (the asm bridge) guarantees:
      /// - `state` is a valid `*mut LlIntState` for the lifetime of the
      ///   slow-path call.
      /// - `state.rust_context` was established by the entry shim
      ///   (`Vm::run_via_dsl`) and points to a `LlIntRustContext<'vm>`
      ///   whose `'vm` outlives `'borrow`.
      pub unsafe fn from_raw(state: *mut LlIntState) -> Self {
          let rust = unsafe {
              &mut *((*state).rust_context as *mut LlIntRustContext<'vm>)
          };
          Self {
              inner: LlIntDispatchInner::Asm { state, rust },
          }
      }

      /// Pre-slow-path sync — copy asm mirrors into the Rust-side
      /// snapshot before semantic code observes the frame. See design §6.
      pub fn sync_from_asm(&mut self) {
          if let LlIntDispatchInner::Asm { state, rust } = &mut self.inner {
              unsafe {
                  rust.frame.set_instruction_offset((**state).frame_pc_offset);
                  // Mirror registers_base / feedback_vector_base via the
                  // accessors on FrameRecord. Field renaming may apply.
              }
          }
      }
  }
  ```

- [ ] **Step 2: Verify**

  ```sh
  cargo build -p lyng-vm
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/src/dsl/slow_path.rs
  git commit -m "DSL-0b: LlIntDispatchState::from_raw + sync_from_asm (asm path)"
  ```

---

### Task B11: `slow_path.rs` — `translate_outcome` shim helper

**Files:**
- Modify: `crates/vm/src/dsl/slow_path.rs`

- [ ] **Step 1: Add the outcome translator**

  ```rust
  impl<'vm, 'borrow> LlIntDispatchState<'vm, 'borrow> {
      /// Translate `SemanticOutcome` to `SlowPathReturn`. Used by every
      /// asm-facing cold-stub shim.
      pub fn translate_outcome(&mut self, outcome: SemanticOutcome) -> SlowPathReturn {
          match outcome {
              SemanticOutcome::Continue { pc_advance } => {
                  // Sync new PC offset back to LlIntState.
                  if let LlIntDispatchInner::Asm { state, rust } = &mut self.inner {
                      let new_offset = rust.frame.instruction_offset()
                          .wrapping_add(pc_advance);
                      unsafe { (**state).frame_pc_offset = new_offset; }
                  }
                  SlowPathReturn { tag: SlowPathTag::Continue as u64, payload: 0 }
              }
              SemanticOutcome::Refresh => {
                  // Mirror full frame from rust_context.frame into state.frame_*.
                  if let LlIntDispatchInner::Asm { state, rust } = &mut self.inner {
                      unsafe {
                          (**state).frame_pc_offset   = rust.frame.instruction_offset();
                          (**state).frame_regs_base   = rust.frame.registers_base_ptr();
                          // ... fv_base, depth, etc.
                      }
                  }
                  SlowPathReturn { tag: SlowPathTag::Refresh as u64, payload: 0 }
              }
              SemanticOutcome::ExitDone { value } => {
                  if let LlIntDispatchInner::Asm { state: _, rust } = &mut self.inner {
                      rust.exit.kind = crate::dsl::llint_state::ExitKind::Done;
                      rust.exit.done_value = value;
                  }
                  SlowPathReturn { tag: SlowPathTag::Exit as u64, payload: 0 }
              }
              SemanticOutcome::ExitError { error } => {
                  if let LlIntDispatchInner::Asm { state: _, rust } = &mut self.inner {
                      rust.exit.kind = crate::dsl::llint_state::ExitKind::Error;
                      rust.exit.error = Some(Box::new(error));
                  }
                  SlowPathReturn { tag: SlowPathTag::Exit as u64, payload: 0 }
              }
          }
      }
  }
  ```

  `FrameRecord` may not have a `registers_base_ptr` accessor today; add one if needed (it already stores `*mut Value` somewhere internally).

- [ ] **Step 2: Verify**

  ```sh
  cargo build -p lyng-vm
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/src/dsl/slow_path.rs \
          crates/vm/src/vm/frame.rs
  git commit -m "DSL-0b: translate_outcome shim helper + FrameRecord accessors"
  ```

---

### Task B12: Cold-stub shim convenience macro

**Files:**
- Modify: `crates/vm/src/dsl/slow_path.rs`

- [ ] **Step 1: Define the shim convenience macro**

  Append:
  ```rust
  /// Macro-helper for generating an asm-facing cold-stub shim from a
  /// semantic body. Keeps every cold stub's shim wrapper to one
  /// declaration site. Generates `extern "C" fn op_xxx_slow_rs(...)`.
  #[macro_export]
  macro_rules! dsl_cold_shim {
      (
          $shim_name:ident
          for $semantic:path
          with $args_ty:ty
          { $($field:ident: $field_ty:ty),* $(,)? }
      ) => {
          #[no_mangle]
          pub extern "C" fn $shim_name(
              state: *mut $crate::dsl::llint_state::LlIntState,
              $($field: $field_ty),*
          ) -> $crate::dsl::slow_path::SlowPathReturn {
              // SAFETY: state is a valid LlIntState pointer; caller is
              // the asm bridge.
              let mut dispatch = unsafe {
                  $crate::dsl::slow_path::LlIntDispatchState::from_raw(state)
              };
              dispatch.sync_from_asm();
              let args = <$args_ty> { $($field),* };
              let outcome = $semantic(&mut dispatch, args);
              dispatch.translate_outcome(outcome)
          }
      };
  }
  ```

- [ ] **Step 2: Verify**

  ```sh
  cargo build -p lyng-vm
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/src/dsl/slow_path.rs
  git commit -m "DSL-0b: dsl_cold_shim! convenience macro"
  ```

---

### Task B13: Entry shim + `_interpreter_exit`

**Files:**
- Create: `crates/vm/src/dsl/entry.rs`
- Create: `crates/vm/src/dsl/handlers/mod.rs` (skeleton)
- Modify: `crates/vm/src/dsl/mod.rs`
- Modify: `crates/vm/src/vm.rs`

- [ ] **Step 1: Create entry.rs**

  Create `crates/vm/src/dsl/entry.rs`:
  ```rust
  //! Entry shim and exit shim per design §5 / §6.

  use std::sync::Arc;
  use lyng_env::Agent;
  use lyng_host::HostHooks;
  use lyng_objects::NativeFunctionRegistry;
  use lyng_types::Value;

  use crate::dsl::llint_state::{ExitKind, LlIntExitSlot, LlIntRustContext, LlIntRustContextOpaque, LlIntState};
  use crate::error::{VmError, VmResult};
  use crate::vm::install::InstalledFunction;
  use crate::FrameRecord;
  use crate::Vm;

  /// New entry point used after DSL-0c flips dispatch.
  ///
  /// During DSL-0b this is callable but not the default — `Vm::run`
  /// continues to route through the α trampoline. Task C1 swaps the
  /// route.
  pub fn run_via_dsl(
      vm: &mut Vm,
      agent: &mut Agent,
      host: &dyn HostHooks,
      registry: &mut dyn NativeFunctionRegistry,
      installed: Arc<InstalledFunction>,
      frame: FrameRecord,
  ) -> VmResult<Value> {
      let frame_depth = vm.frames().len();
      let pb_base = installed.function.instruction_bytes().as_ptr();
      let regs_base = frame.registers_base_ptr();
      let fv_base = installed.feedback_flat.as_ptr() as *mut crate::dsl::feedback_flat::FeedbackEntry;

      let mut rust_ctx = LlIntRustContext {
          vm,
          agent,
          host,
          registry,
          installed,
          frame,
          frame_depth,
          exit: LlIntExitSlot::default(),
      };

      let mut state = LlIntState {
          frame_pc_offset:   frame.instruction_offset(),
          _pad1:             0,
          frame_pb_base:     pb_base,
          frame_regs_base:   regs_base,
          frame_fv_base:     fv_base,
          frame_depth:       frame_depth as u32,
          frame_check_epoch: 0,
          rust_context:      (&mut rust_ctx) as *mut _ as *mut LlIntRustContextOpaque,
          prefix:            0,
          _pad2:             [0; 7],
      };

      // SAFETY: state is a valid mutable pointer to a stack-local
      // LlIntState; the asm trampoline only reads through it on the
      // current thread for the duration of this call.
      unsafe { run_dsl_trampoline(&mut state as *mut LlIntState) };

      match rust_ctx.exit.kind {
          ExitKind::Done => Ok(rust_ctx.exit.done_value),
          ExitKind::Error => Err(*rust_ctx.exit.error.take().unwrap()),
          ExitKind::None => Err(VmError::TrampolineExitedWithoutSetting),
      }
  }

  /// Asm-side trampoline entry. Loads pinned registers + tail-jumps to
  /// the first handler. The handler chain runs until `_interpreter_exit`
  /// is hit.
  ///
  /// Implemented as `#[unsafe(naked)] extern "C"` in DSL-0b once the
  /// proc-macro + backend are wired. Placeholder for now.
  #[unsafe(naked)]
  pub unsafe extern "C" fn run_dsl_trampoline(_state: *mut LlIntState) {
      ::core::arch::naked_asm!(
          // x0 = state. Setup pinned regs from state.frame_* fields, load
          // VM, TABLE, then tail-jump to first handler.
          //
          // Stub: just return (re-routed once handlers exist). This panics
          // at runtime if invoked before Tasks B30+ land real handlers.
          "ret",
          options(noreturn),
      );
  }

  /// `_interpreter_exit` is the symbolic target the slow-path bridge
  /// uses to escape the trampoline. The asm `b {exit}` branches here;
  /// this function reads `rust_context.exit` and returns to the caller
  /// of `run_via_dsl` via a normal Rust return.
  ///
  /// The actual interaction with the trampoline is via the stack frame:
  /// `run_dsl_trampoline` sets up a normal stack frame, so `_interpreter_exit`
  /// can be a normal `extern "C"` that pops back to `run_via_dsl`.
  #[no_mangle]
  pub extern "C" fn _interpreter_exit() {
      // No body — the asm "b" jumps here and the function returns
      // immediately (single `ret` instruction generated by rustc).
  }
  ```

  `VmError::TrampolineExitedWithoutSetting` is a new error variant — add it to `crates/vm/src/error.rs`.

- [ ] **Step 2: Wire into `dsl/mod.rs`**

  ```rust
  pub mod entry;
  pub mod handlers;
  ```

- [ ] **Step 3: Create handlers/mod.rs skeleton**

  Create `crates/vm/src/dsl/handlers/mod.rs`:
  ```rust
  //! DSL handler functions per design §10 DSL-0b.

  // Placeholder dispatch table; populated in Task B29.
  pub static DSL_DISPATCH_TABLE: [unsafe extern "C" fn() -> !; 256] = [
      unimplemented_dsl_handler; 256
  ];

  unsafe extern "C" fn unimplemented_dsl_handler() -> ! {
      loop {} // SAFETY: never reachable until DSL-0c flips dispatch
  }
  ```

- [ ] **Step 4: Add `Vm::run_via_dsl` thin wrapper**

  In `crates/vm/src/vm.rs`, add:
  ```rust
  impl Vm {
      pub(crate) fn run_via_dsl(
          &mut self,
          agent: &mut Agent,
          host: &dyn HostHooks,
          registry: &mut dyn NativeFunctionRegistry,
          installed: Arc<InstalledFunction>,
          frame: FrameRecord,
      ) -> VmResult<Value> {
          crate::dsl::entry::run_via_dsl(self, agent, host, registry, installed, frame)
      }
  }
  ```

  Note: not called yet — `Vm::run` still routes to `run_via_trampoline`.

- [ ] **Step 5: Verify**

  ```sh
  cargo build -p lyng-vm
  ```

- [ ] **Step 6: Commit**

  ```sh
  git add crates/vm/src/dsl/entry.rs \
          crates/vm/src/dsl/llint_state.rs \
          crates/vm/src/dsl/reg_convention.rs \
          crates/vm/src/dsl/handlers/mod.rs \
          crates/vm/src/dsl/mod.rs \
          crates/vm/src/vm.rs \
          crates/vm/src/error.rs
  git commit -m "DSL-0b: entry trampoline + _interpreter_exit + placeholder dispatch table"
  ```

---

### Task B14: FeedbackVector flat-array — define `FeedbackEntry` layout

**Files:**
- Create: `crates/vm/src/dsl/feedback_flat.rs`
- Modify: `crates/vm/src/dsl/mod.rs`

Per design §9, the DSL needs the `FV` pin to point at a flat array of fixed-size entries. The refactor preserves per-entry packed monomorphic/proto/polymorphic state (Phase 3f); only vector storage changes from `Vec<Option<FeedbackSiteState>>` to `Box<[FeedbackEntry]>`.

- [ ] **Step 1: Inspect current `FeedbackVector` and `FeedbackSiteState`**

  ```sh
  rg -n "pub.*struct.*Feedback(Vector|SiteState|EntryFootprint)" crates/vm/src/vm/feedback.rs | head -20
  ```

- [ ] **Step 2: Design the flat entry**

  Create `crates/vm/src/dsl/feedback_flat.rs`:
  ```rust
  //! Flat-array feedback storage for the DSL `FV` pin per design §9.
  //!
  //! Each `FeedbackEntry` is a fixed-size, pointer-stable IC slot. The
  //! field-by-field content mirrors today's `FeedbackSiteState` packed
  //! sidecars; only the *vector storage* changes from `Vec<Option<...>>`
  //! to `Box<[FeedbackEntry]>` so the asm `FV` pin can be a single
  //! pointer with computed offset.

  use lyng_types::Value;
  // Re-export today's per-entry state shape so the rest of the engine
  // continues to use a single source of truth for per-site content.
  pub use crate::vm::feedback::FeedbackSiteState;

  /// Single feedback entry, fixed-size, pointer-stable for the lifetime
  /// of the `InstalledFunction`.
  #[repr(C)]
  pub struct FeedbackEntry {
      pub state: FeedbackSiteState,
  }

  impl Default for FeedbackEntry {
      fn default() -> Self {
          Self { state: FeedbackSiteState::default() }
      }
  }
  ```

- [ ] **Step 3: Wire into `dsl/mod.rs`**

  ```rust
  pub mod feedback_flat;
  ```

- [ ] **Step 4: Verify**

  ```sh
  cargo build -p lyng-vm
  ```
  Expected: may fail if `FeedbackSiteState` is not `pub`. Promote its visibility if needed.

- [ ] **Step 5: Commit**

  ```sh
  git add crates/vm/src/dsl/feedback_flat.rs \
          crates/vm/src/dsl/mod.rs \
          crates/vm/src/vm/feedback.rs
  git commit -m "DSL-0b: FeedbackEntry flat-array layout"
  ```

---

### Task B15: FeedbackVector flat-array — eager allocation at install

**Files:**
- Modify: `crates/vm/src/vm/install.rs`
- Modify: `crates/vm/src/vm/installed.rs`

- [ ] **Step 1: Add `feedback_flat` field to `InstalledFunction`**

  Inspect `crates/vm/src/vm/installed.rs`. Identify `InstalledFunction`. Add a field:
  ```rust
  pub struct InstalledFunction {
      // ... existing fields ...
      /// Flat IC-entry storage pinned by the DSL substrate's `FV`
      /// register. Allocated to `function.feedback_slot_count()` at
      /// install; never grown. See `crates/vm/src/dsl/feedback_flat.rs`.
      pub feedback_flat: Box<[crate::dsl::feedback_flat::FeedbackEntry]>,
  }
  ```

  Add an accessor:
  ```rust
  impl crate::bytecode::Function /* or wherever feedback_slot_count() lives */ {
      pub fn feedback_flat(&self) -> &[crate::dsl::feedback_flat::FeedbackEntry] {
          // Routed through InstalledFunction at runtime; this stub may
          // not be needed depending on accessor chain.
      }
  }
  ```

- [ ] **Step 2: Populate it in install**

  In `crates/vm/src/vm/install.rs`'s install path, after computing the feedback slot count:
  ```rust
  let feedback_slot_count = function.feedback_slot_count();
  let feedback_flat: Box<[crate::dsl::feedback_flat::FeedbackEntry]> =
      (0..feedback_slot_count)
          .map(|_| crate::dsl::feedback_flat::FeedbackEntry::default())
          .collect::<Vec<_>>()
          .into_boxed_slice();
  let installed = InstalledFunction {
      // ... existing field initializations ...
      feedback_flat,
  };
  ```

- [ ] **Step 3: Verify**

  ```sh
  cargo build -p lyng-vm
  ```
  Expected: clean build.

- [ ] **Step 4: Commit**

  ```sh
  git add crates/vm/src/vm/install.rs \
          crates/vm/src/vm/installed.rs
  git commit -m "DSL-0b: eager FeedbackEntry flat allocation at install"
  ```

---

### Task B16: FeedbackVector flat-array — wire `FV` pin from install

**Files:**
- Modify: `crates/vm/src/dsl/entry.rs`

- [ ] **Step 1: Update `run_via_dsl` to use the flat array**

  In the entry shim, replace the `fv_base` line with:
  ```rust
  let fv_base = rust_ctx
      .installed
      .feedback_flat
      .as_ptr() as *mut crate::dsl::feedback_flat::FeedbackEntry;
  ```

- [ ] **Step 2: Verify**

  ```sh
  cargo build -p lyng-vm
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/src/dsl/entry.rs
  git commit -m "DSL-0b: wire FV pin to InstalledFunction.feedback_flat"
  ```

---

### Task B17: FeedbackVector flat-array — dual-write from existing record paths

**Files:**
- Modify: `crates/vm/src/vm/feedback.rs`

During DSL-0b alpha is still the active path. Alpha's existing `record_feedback_slot` writes into the old `Vec<Option<FeedbackSiteState>>`. To keep the flat array consistent, dual-write: every record path also writes into `InstalledFunction.feedback_flat[slot_id]`.

- [ ] **Step 1: Find the existing record paths**

  ```sh
  rg -n "fn record_feedback_slot\|fn record_smi\|fn value_profile" crates/vm/src/vm/feedback.rs | head -10
  ```

- [ ] **Step 2: Add the dual-write**

  For each `pub(super) fn record_*` method on `Vm`, after writing to the legacy `FeedbackVector`, write the same content to the flat array. Resolve the `InstalledFunction` via `vm.installed_for_code(code)` and index into `installed.feedback_flat[slot.0 as usize].state`.

- [ ] **Step 3: Add an invariant test**

  Create `crates/vm/tests/feedback_flat_consistency.rs`:
  ```rust
  //! After alpha runs a simple SMI-add hot loop, both the legacy
  //! `FeedbackVector` slot and the new `FeedbackEntry` slot should
  //! reflect the same observed type (SMI).

  use lyng_vm::test_helpers::{run_script, snapshot_feedback};  // or whatever the existing fixture is

  #[test]
  fn dual_write_keeps_legacy_and_flat_in_sync() {
      let snapshot = snapshot_feedback("for (let i = 0; i < 100; i += 1) { i + i }");
      assert_eq!(snapshot.legacy_smi_slots, snapshot.flat_smi_slots);
  }
  ```

  If the fixture helpers don't exist, create them inline in the test. The point is to verify dual-write parity.

- [ ] **Step 4: Verify**

  ```sh
  cargo test -p lyng-vm --test feedback_flat_consistency
  ```

- [ ] **Step 5: Commit**

  ```sh
  git add crates/vm/src/vm/feedback.rs \
          crates/vm/tests/feedback_flat_consistency.rs
  git commit -m "DSL-0b: dual-write FeedbackEntry from existing record paths"
  ```

---

### Task B18: FeedbackVector flat-array — phase 3f sidecar parity

**Files:**
- Modify: `crates/vm/src/dsl/feedback_flat.rs`
- Modify: `crates/vm/src/vm/feedback.rs`

Phase 3f's packed monomorphic/proto/polymorphic sidecars must continue to work through the flat array.

- [ ] **Step 1: Audit Phase 3f sidecars**

  ```sh
  rg -n "phase[-_]3f\|sidecar\|packed_mono\|packed_proto" crates/vm/src/vm/feedback.rs | head -20
  ```

- [ ] **Step 2: If sidecars are inside `FeedbackSiteState`, the refactor is free**

  Per the design (§9): "the flattening is about vector storage, not entry content — Phase 3f's packed sidecars stay inside each entry." If `FeedbackSiteState` already contains the sidecars, the wrap into `FeedbackEntry { state: ... }` automatically carries them. Verify by reading the struct definition.

- [ ] **Step 3: Add a parity test**

  Create or extend `tests/feedback_flat_consistency.rs` to verify a polymorphic property-access hot loop produces identical sidecars in both the legacy vector and the flat entry.

- [ ] **Step 4: Verify**

  ```sh
  cargo test -p lyng-vm --test feedback_flat_consistency
  ```

- [ ] **Step 5: Commit**

  ```sh
  git add crates/vm/src/dsl/feedback_flat.rs \
          crates/vm/src/vm/feedback.rs \
          crates/vm/tests/feedback_flat_consistency.rs
  git commit -m "DSL-0b: verify Phase 3f sidecar parity through flat array"
  ```

---

### Task B19: FeedbackVector flat-array — V8 v7 regression check

**Files:** none modified; produces evidence.

- [ ] **Step 1: Run V8 v7 with flat-array dual-write enabled**

  ```sh
  cargo run --release -p lyng-bench -- v8suite --report /tmp/dsl-0b-fv-refactor-v8.md --json /tmp/dsl-0b-fv-refactor-v8.json
  cp /tmp/dsl-0b-fv-refactor-v8.md reports/lyng/dsl-0b-fv-refactor-v8.md
  ```

- [ ] **Step 2: Compare to pre-DSL-0 baseline**

  Geomean shift should be small (the dual-write adds one indirect store per record-site write; cold). If geomean regresses > 5% vs the pre-DSL-0 baseline, the dual-write is too hot and needs to be batched or simplified. Document the regression in the report header.

- [ ] **Step 3: Commit evidence**

  ```sh
  git add reports/lyng/dsl-0b-fv-refactor-v8.md
  git commit -m "DSL-0b: FV refactor V8 v7 regression check evidence"
  ```

---

### Task B20: AArch64 backend prelude

**Files:**
- Create: `crates/vm/src/dsl/backend/mod.rs`
- Create: `crates/vm/src/dsl/backend/aarch64/mod.rs`
- Create: `crates/vm/src/dsl/backend/aarch64/prelude.rs`
- Modify: `crates/vm/src/dsl/mod.rs`

- [ ] **Step 1: Backend dispatch module**

  Create `crates/vm/src/dsl/backend/mod.rs`:
  ```rust
  //! Per-arch DSL backend dispatch. Today: AArch64 only.

  #[cfg(target_arch = "aarch64")]
  pub mod aarch64;

  // Re-export the proc-macro-facing entry point.
  #[cfg(target_arch = "aarch64")]
  pub use aarch64::__llint_handler_body;

  // raw_asm! shim used by the proc-macro to emit a label.
  #[cfg(target_arch = "aarch64")]
  #[macro_export]
  macro_rules! raw_asm {
      ($body:literal) => {
          ::core::arch::asm!($body);
      };
  }
  ```

- [ ] **Step 2: AArch64 backend root**

  Create `crates/vm/src/dsl/backend/aarch64/mod.rs`:
  ```rust
  pub mod arithmetic;
  pub mod control;
  pub mod counters;
  pub mod feedback;
  pub mod memory;
  pub mod objects;
  pub mod operands;
  pub mod prelude;
  pub mod safepoint;
  pub mod values;

  /// Top-level body builder invoked by the proc-macro. Concatenates the
  /// operand-decode prologue, the body fragments, and the dispatch
  /// trailer into a single `core::arch::naked_asm!` block.
  #[macro_export]
  macro_rules! __llint_handler_body {
      (
          layout = $layout:ident,
          operands = [$($op:ident),*],
          length = $length:literal,
          body = { $($body:tt)* }
      ) => {
          // Single naked_asm! block containing:
          //   1. Operand-decode prologue (per layout).
          //   2. Body fragments.
          //   3. Dispatch trailer (auto-appended if not present in body).
          ::core::arch::naked_asm!(
              // Prologue placeholder; replaced per layout in Task B22.
              "// prologue: layout = ", stringify!($layout),
              // Body fragments (each DSL op produces a string literal):
              $($body)*
              options(noreturn),
          )
      };
  }
  pub use __llint_handler_body;
  ```

- [ ] **Step 3: AArch64 prelude constants**

  Create `crates/vm/src/dsl/backend/aarch64/prelude.rs`:
  ```rust
  //! AArch64-specific constants referenced by DSL operation macros:
  //! NaN-tag masks, exit-slot offsets, layout-decode helpers.
  //!
  //! Values authoritative per reports/lyng/llint-dsl-value-layout.md
  //! and reports/lyng/llint-dsl-abi.md.

  use lyng_types::Value;

  // SMI tag bits per llint-dsl-value-layout.md §2.
  pub const VALUE_TAG_SMI_MASK: u64 = 0xFFFF_0000_0000_0000;
  pub const VALUE_TAG_SMI_PATTERN: u64 = 0xFFFE_0000_0000_0000;
  pub const VALUE_SMI_PAYLOAD_BITS: u32 = 32;

  // Per the same report — object-ref / undefined / null / bool patterns.
  // Fill in with the exact masks from the report; placeholders here.
  pub const VALUE_TAG_OBJECT_REF_MASK: u64 = 0xFFFF_FFFF_0000_0000;
  pub const VALUE_TAG_OBJECT_REF_PATTERN: u64 = 0xFFFC_0000_0000_0000;
  pub const VALUE_UNDEFINED_BITS: u64 = 0xFFF8_0000_0000_0000;
  pub const VALUE_NULL_BITS: u64 = 0xFFF9_0000_0000_0000;
  pub const VALUE_TRUE_BITS: u64 = 0xFFFA_0000_0000_0001;
  pub const VALUE_FALSE_BITS: u64 = 0xFFFA_0000_0000_0000;

  // Compile-time test: confirm these match the Value layout.
  const _: () = {
      assert!(Value::from_smi(0).into_raw() == VALUE_TAG_SMI_PATTERN);
      // Additional assertions per the value-layout report.
  };
  ```

  Reconcile placeholder masks against the actual values documented in `reports/lyng/llint-dsl-value-layout.md`. The compile-time `assert!`s will catch mismatches.

- [ ] **Step 4: Wire into `dsl/mod.rs`**

  ```rust
  pub mod backend;
  ```

- [ ] **Step 5: Verify**

  ```sh
  cargo build -p lyng-vm 2>&1 | tail -20
  ```
  Expected: may fail on the const-assert. Update masks to match the value-layout report.

- [ ] **Step 6: Commit**

  ```sh
  git add crates/vm/src/dsl/backend/ crates/vm/src/dsl/mod.rs
  git commit -m "DSL-0b: AArch64 backend scaffold + prelude constants"
  ```

---

### Task B21: AArch64 backend — operand decoding

**Files:**
- Create: `crates/vm/src/dsl/backend/aarch64/operands.rs`

Operand-decode macros emit asm sequences that read operand bytes from the bytecode stream (relative to PC) into named scratch registers. One macro per layout.

- [ ] **Step 1: Implement operand macros**

  ```rust
  //! Operand-decoding asm fragments for AArch64.
  //!
  //! These macros produce asm string fragments interpolated into the
  //! per-handler `naked_asm!` block by the proc-macro lowerer.

  // `decode_abc!`: reads three byte operands (narrow) into named regs.
  // Wide / ExtraWide variants are produced by the `dispatch_prefixed!`
  // path — narrow handlers always decode at byte width.
  #[macro_export]
  macro_rules! decode_abc {
      ($a:ident, $b:ident, $c:ident) => {
          concat!(
              "ldrb   w", stringify!($a), ", [x19, #1]\n",
              "ldrb   w", stringify!($b), ", [x19, #2]\n",
              "ldrb   w", stringify!($c), ", [x19, #3]\n",
          )
      };
  }

  // `decode_abc_slot!`: three byte operands + 16-bit feedback slot.
  #[macro_export]
  macro_rules! decode_abc_slot {
      ($a:ident, $b:ident, $c:ident, $slot:ident) => {
          concat!(
              "ldrb   w", stringify!($a), ", [x19, #1]\n",
              "ldrb   w", stringify!($b), ", [x19, #2]\n",
              "ldrb   w", stringify!($c), ", [x19, #3]\n",
              "ldrh   w", stringify!($slot), ", [x19, #4]\n",
          )
      };
  }

  // Register-file access.
  #[macro_export]
  macro_rules! load_reg {
      ($idx:ident => $dst:ident) => {
          concat!(
              "ldr    x", stringify!($dst), ", [x20, x", stringify!($idx), ", lsl #3]\n",
          )
      };
  }

  #[macro_export]
  macro_rules! store_reg {
      ($idx:ident, $src:ident) => {
          concat!(
              "str    x", stringify!($src), ", [x20, x", stringify!($idx), ", lsl #3]\n",
          )
      };
  }
  ```

  Note: the macros use `stringify!` to splice the scratch-register name (e.g. `t0`) into the asm string. The proc-macro's lowerer maps named operands to `t0..t6` and substitutes those into the macro invocations. Verify the integration once `decode_abc!` is exercised in Task B30.

- [ ] **Step 2: Verify the module compiles**

  ```sh
  cargo build -p lyng-vm
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/src/dsl/backend/aarch64/operands.rs
  git commit -m "DSL-0b: AArch64 operand-decode + register-file macros"
  ```

---

### Task B22: AArch64 backend — value-tag checks and tag manipulation

**Files:**
- Create: `crates/vm/src/dsl/backend/aarch64/values.rs`

- [ ] **Step 1: Implement value macros**

  ```rust
  //! Value-tag check and tag-manipulation asm fragments for AArch64.
  //!
  //! Per llint-dsl-value-layout.md, NaN-tagged Value with SMI variants
  //! and ObjectRef (u32) handles. SMI tag = upper 16 bits == 0xFFFE.

  /// Check `reg` is an SMI; branch to `label` on miss.
  #[macro_export]
  macro_rules! check_smi {
      ($reg:ident, $label:tt) => {
          concat!(
              "lsr    x9, x", stringify!($reg), ", #48\n",
              "cmp    w9, #0xfffe\n",
              "b.ne   ", stringify!($label), "\n",
          )
      };
  }

  /// Untag SMI: load lower 32 bits into the same reg, sign-extended.
  #[macro_export]
  macro_rules! untag_smi {
      ($reg:ident) => {
          concat!(
              "sxtw   x", stringify!($reg), ", w", stringify!($reg), "\n",
          )
      };
  }

  /// Tag an i32 in `reg` as an SMI Value (in-place).
  #[macro_export]
  macro_rules! tag_smi {
      ($reg:ident) => {
          concat!(
              "mov    x9, #0xfffe000000000000\n",
              "orr    x", stringify!($reg), ", x9, x", stringify!($reg), ", uxtw\n",
          )
      };
  }

  // Additional macros per ops.md vocabulary list (check_object_ref!,
  // check_undefined!, check_null!, check_bool!, check_double!, etc.)
  // follow the same shape. Each lowers to 1–3 instructions and
  // documents the irreducible delta vs LLInt where applicable.
  ```

- [ ] **Step 2: Verify**

  ```sh
  cargo build -p lyng-vm
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/src/dsl/backend/aarch64/values.rs
  git commit -m "DSL-0b: AArch64 value-tag check / manipulation macros"
  ```

---

### Task B23: AArch64 backend — object-record access macros

**Files:**
- Create: `crates/vm/src/dsl/backend/aarch64/objects.rs`

- [ ] **Step 1: Implement object-access macros**

  ```rust
  //! Object-record access via ObjectRef handles per llint-dsl-value-layout.md.
  //! When pointer-identity cells land as a later refactor (design §9 DSL-3),
  //! these macros are renamed `load_cell_*!` and emit fewer instructions.

  #[macro_export]
  macro_rules! load_object_record {
      ($ref:ident => $dst:ident) => {
          // ObjectRef (u32) → *const ObjectRecord via heap-pool indirection.
          // VM holds heap_pool_base; resolved here through Vm.
          concat!(
              "ldr    x9, [x22, {vm_heap_pool}]\n",
              "ldr    x", stringify!($dst), ", [x9, x", stringify!($ref), ", lsl #3]\n",
          )
      };
  }

  // Additional macros: load_record_shape!, load_record_inline_slot!,
  // load_record_outline_slots!, load_outline_slot!. Fill against the
  // ObjectRecord layout in crates/objects/.
  ```

  Note: `{vm_heap_pool}` references `offset_of!(Vm, heap_pool_base)`. Resolve to a real offset by adding a `pub const VM_HEAP_POOL_OFFSET` in `reg_convention.rs` (similar to `LLINT_STATE_*` constants).

- [ ] **Step 2: Verify**

  ```sh
  cargo build -p lyng-vm
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/src/dsl/backend/aarch64/objects.rs
  git commit -m "DSL-0b: AArch64 object-record access macros"
  ```

---

### Task B24: AArch64 backend — arithmetic macros

**Files:**
- Create: `crates/vm/src/dsl/backend/aarch64/arithmetic.rs`

- [ ] **Step 1: Implement arithmetic macros**

  ```rust
  //! SMI fast-path arithmetic asm fragments for AArch64.

  /// 32-bit add with overflow detection; branch to `label` on overflow
  /// (slow path).
  #[macro_export]
  macro_rules! add_smi_overflow {
      ($lhs:ident, $rhs:ident => $dst:ident, $label:tt) => {
          concat!(
              "adds   w", stringify!($dst), ", w", stringify!($lhs), ", w", stringify!($rhs), "\n",
              "b.vs   ", stringify!($label), "\n",
          )
      };
  }

  // sub_smi_overflow!, mul_smi_overflow! follow the same shape.
  // bit_and_smi!, bit_or_smi!, bit_xor_smi! — overflow-free.
  // Shifts: shift_left_smi!, shift_right_smi! — check shift count first.
  ```

- [ ] **Step 2: Verify**

  ```sh
  cargo build -p lyng-vm
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/src/dsl/backend/aarch64/arithmetic.rs
  git commit -m "DSL-0b: AArch64 SMI arithmetic macros"
  ```

---

### Task B25: AArch64 backend — control flow + slow-path bridge

**Files:**
- Create: `crates/vm/src/dsl/backend/aarch64/control.rs`

This is the most-used backend module. `dispatch!()` is the tail-jump at the end of every fast-path handler; `call_slow!` + `dispatch_after_slow!` is the bridge to a Rust shim.

- [ ] **Step 1: Implement control macros**

  ```rust
  //! Dispatch, branches, and slow-path bridge fragments for AArch64.

  /// Tail-jump dispatch — auto-advance PC by handler's encoded length
  /// (provided by the proc-macro via the `length =` attribute).
  #[macro_export]
  macro_rules! dispatch {
      () => {
          // Length placeholder; substituted by the proc-macro.
          concat!(
              "add    x19, x19, {length}\n",
              "ldrb   w8, [x19]\n",
              "ldr    x9, [x23, x8, lsl #3]\n",
              "br     x9\n",
          )
      };
      (advance = $n:literal) => {
          concat!(
              "add    x19, x19, #", stringify!($n), "\n",
              "ldrb   w8, [x19]\n",
              "ldr    x9, [x23, x8, lsl #3]\n",
              "br     x9\n",
          )
      };
  }

  /// Call into a Rust slow-path shim. The shim signature is
  /// `extern "C" fn(state: *mut LlIntState, operand_0: u32, ...)`.
  /// Operands pass in a1..a5.
  #[macro_export]
  macro_rules! call_slow {
      ($shim:ident, args = [$($arg:ident),*]) => {
          concat!(
              // Pre-call sync: state.frame_pc_offset ← PC - pb_base
              "ldr    x9, [x24, {state_pb}]\n",
              "sub    x10, x19, x9\n",
              "str    w10, [x24, {state_pc}]\n",
              // Move STATE to a0, operands to a1..aN
              "mov    x0, x24\n",
              // Each $arg slot expands to "mov w<n>, w<arg>" — generated
              // by the proc-macro at lower time, not here, because the
              // operand list is variadic.
              $("mov    w<arg_slot>, w", stringify!($arg), "\n",)*
              // Call shim
              "bl     {shim}\n",
              shim = sym $shim,
              state_pb = const crate::dsl::reg_convention::LLINT_STATE_FRAME_PB_BASE,
              state_pc = const crate::dsl::reg_convention::LLINT_STATE_FRAME_PC_OFFSET,
          )
      };
  }

  /// Post-`call_slow!` dispatch: branch on tag, common case is Continue.
  #[macro_export]
  macro_rules! dispatch_after_slow {
      () => {
          concat!(
              "cbnz   x0, .Lunusual\n",         // tag != Continue
              "ldr    x9, [x24, {state_pb}]\n",
              "add    x19, x9, x1\n",            // PC = pb_base + new_offset
              "ldrb   w8, [x19]\n",
              "ldr    x10, [x23, x8, lsl #3]\n",
              "br     x10\n",
              ".Lunusual:\n",
              "cmp    x0, #2\n",
              "b.eq   .Lexit\n",
              // Refresh: reload from state.frame_*
              "ldr    w11, [x24, {state_pc}]\n",
              "ldr    x9,  [x24, {state_pb}]\n",
              "add    x19, x9, x11\n",
              "ldr    x20, [x24, {state_regs}]\n",
              "ldr    x21, [x24, {state_fv}]\n",
              "ldrb   w8,  [x19]\n",
              "ldr    x10, [x23, x8, lsl #3]\n",
              "br     x10\n",
              ".Lexit:\n",
              "b      {exit}\n",
              state_pb = const crate::dsl::reg_convention::LLINT_STATE_FRAME_PB_BASE,
              state_pc = const crate::dsl::reg_convention::LLINT_STATE_FRAME_PC_OFFSET,
              state_regs = const crate::dsl::reg_convention::LLINT_STATE_FRAME_REGS_BASE,
              state_fv = const crate::dsl::reg_convention::LLINT_STATE_FRAME_FV_BASE,
              exit = sym crate::dsl::entry::_interpreter_exit,
          )
      };
  }

  /// Branch macros.
  #[macro_export]
  macro_rules! branch_zero {
      ($reg:ident, $label:tt) => {
          concat!("cbz    x", stringify!($reg), ", ", stringify!($label), "\n",)
      };
  }
  #[macro_export]
  macro_rules! branch_nonzero {
      ($reg:ident, $label:tt) => {
          concat!("cbnz   x", stringify!($reg), ", ", stringify!($label), "\n",)
      };
  }

  /// Prefix dispatch — see design §6.
  #[macro_export]
  macro_rules! dispatch_prefixed {
      (kind = $kind:ident) => {
          // ... per design §6: reject doubled prefix, store prefix byte,
          // advance 1, dispatch on next byte.
          concat!(
              "ldrb   w9, [x24, {state_prefix}]\n",
              "cbnz   w9, .Ldouble_prefix\n",
              "mov    w9, #", stringify!($kind), "\n",
              "strb   w9, [x24, {state_prefix}]\n",
              "add    x19, x19, #1\n",
              "ldrb   w8, [x19]\n",
              "ldr    x10, [x23, x8, lsl #3]\n",
              "br     x10\n",
              ".Ldouble_prefix:\n",
              // ... call into op_double_prefix_slow_rs ...
              "brk    #0\n",
              state_prefix = const crate::dsl::reg_convention::LLINT_STATE_PREFIX,
          )
      };
  }
  ```

  These macros are intricate. The `{length}`, `{shim}`, `{state_pb}`, etc. placeholders must be resolved by the proc-macro at handler-emission time — the actual `naked_asm!` block built by the `__llint_handler_body!` macro substitutes them. The variadic `args = [...]` in `call_slow!` requires the proc-macro to expand it into a fixed sequence of `mov` instructions (one per operand). Until the proc-macro can do this, the `call_slow!` invocations are emitted as-is and the lowerer in Task B5 must process them.

  This is a known integration risk. **The validation cases B30–B38 are the gate for whether this design works.** If the placeholder/variadic interaction proves untenable, fall back to: the proc-macro builds the full `naked_asm!` string itself (in `lower.rs`), bypassing per-arch `macro_rules!`. That's a refactor of B5 + B20–B28, not a structural change.

- [ ] **Step 2: Verify**

  ```sh
  cargo build -p lyng-vm
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/src/dsl/backend/aarch64/control.rs
  git commit -m "DSL-0b: AArch64 dispatch + slow-path-bridge + prefix macros"
  ```

---

### Task B26: AArch64 backend — feedback + safepoint + memory macros

**Files:**
- Create: `crates/vm/src/dsl/backend/aarch64/feedback.rs`
- Create: `crates/vm/src/dsl/backend/aarch64/safepoint.rs`
- Create: `crates/vm/src/dsl/backend/aarch64/memory.rs`

- [ ] **Step 1: Feedback macros**

  ```rust
  // feedback.rs
  #[macro_export]
  macro_rules! load_feedback_site {
      ($slot:ident => $dst:ident) => {
          concat!(
              // FeedbackEntry stride is sizeof::<FeedbackEntry>() — assume
              // documented in feedback_flat.rs (e.g. 64 bytes).
              "lsl    x9, x", stringify!($slot), ", #6\n",
              "add    x", stringify!($dst), ", x21, x9\n",
          )
      };
  }

  #[macro_export]
  macro_rules! record_smi {
      ($slot:ident) => {
          // Set the entry's observed-type bit for SMI in place.
          concat!(
              "lsl    x9, x", stringify!($slot), ", #6\n",
              "add    x9, x21, x9\n",
              "ldr    w10, [x9, {entry_observed}]\n",
              "orr    w10, w10, #1\n",
              "str    w10, [x9, {entry_observed}]\n",
              entry_observed = const 0,  // offset_of!(FeedbackEntry, state) + observed_types offset
          )
      };
  }
  ```

- [ ] **Step 2: Safepoint macro**

  ```rust
  // safepoint.rs
  #[macro_export]
  macro_rules! poll_safepoint {
      ($label_pending:tt) => {
          concat!(
              "ldrb   w9, [x22, {vm_poll}]\n",
              "cbnz   w9, ", stringify!($label_pending), "\n",
              vm_poll = const crate::dsl::reg_convention::VM_POLL_PENDING_OFFSET,
          )
      };
  }
  ```

  Add `VM_POLL_PENDING_OFFSET` in `reg_convention.rs`.

- [ ] **Step 3: Memory macros**

  ```rust
  // memory.rs
  #[macro_export]
  macro_rules! load_byte {
      ($base:expr, $offset:literal => $dst:ident) => {
          concat!(
              "ldrb   w", stringify!($dst), ", [x", stringify!($base), ", #", stringify!($offset), "]\n",
          )
      };
  }
  // load_word!, load_quad!, store_byte!, etc. follow the same shape.
  ```

- [ ] **Step 4: Verify**

  ```sh
  cargo build -p lyng-vm
  ```

- [ ] **Step 5: Commit**

  ```sh
  git add crates/vm/src/dsl/backend/aarch64/feedback.rs \
          crates/vm/src/dsl/backend/aarch64/safepoint.rs \
          crates/vm/src/dsl/backend/aarch64/memory.rs \
          crates/vm/src/dsl/reg_convention.rs
  git commit -m "DSL-0b: AArch64 feedback / safepoint / memory macros"
  ```

---

### Task B27: AArch64 backend — `inc_counter!` for opcode counters

**Files:**
- Create: `crates/vm/src/dsl/backend/aarch64/counters.rs`

- [ ] **Step 1: Implement `inc_counter!`**

  ```rust
  //! Opcode-counter increment, gated by `--features diagnostic-counters`.
  //!
  //! When the feature is off, the macro expands to an empty string — zero
  //! per-dispatch cost. When on, emits 3 instructions:
  //!   ldr  x9, [x22, {vm_counter_base}]
  //!   add  x9, x9, x<opcode_id>, lsl #3
  //!   ldr  x10, [x9]
  //!   add  x10, x10, #1
  //!   str  x10, [x9]

  #[cfg(feature = "diagnostic-counters")]
  #[macro_export]
  macro_rules! inc_counter {
      ($opcode_byte:literal) => {
          concat!(
              "ldr    x9, [x22, {vm_counter_base}]\n",
              "ldr    x10, [x9, #", stringify!($opcode_byte * 8), "]\n",
              "add    x10, x10, #1\n",
              "str    x10, [x9, #", stringify!($opcode_byte * 8), "]\n",
              vm_counter_base = const crate::dsl::reg_convention::VM_OPCODE_COUNTER_OFFSET,
          )
      };
  }

  #[cfg(not(feature = "diagnostic-counters"))]
  #[macro_export]
  macro_rules! inc_counter {
      ($opcode_byte:literal) => { "" };
  }
  ```

  Add `VM_OPCODE_COUNTER_OFFSET` in `reg_convention.rs`.

- [ ] **Step 2: Verify**

  ```sh
  cargo build -p lyng-vm
  cargo build -p lyng-vm --features diagnostic-counters
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/src/dsl/backend/aarch64/counters.rs \
          crates/vm/src/dsl/reg_convention.rs
  git commit -m "DSL-0b: AArch64 inc_counter! (feature-gated)"
  ```

---

### Task B28: Document the DSL vocabulary

**Files:**
- Create: `crates/vm/src/dsl/ops.md`

- [ ] **Step 1: Author the vocabulary doc**

  Create `crates/vm/src/dsl/ops.md`:
  ```markdown
  # DSL operation vocabulary (AArch64)

  All operations are `macro_rules!` macros in
  `crates/vm/src/dsl/backend/aarch64/*.rs` that produce string
  literals via `concat!`. The proc-macro lowerer interpolates them into
  the `naked_asm!` body.

  ## Operand decoding

  | Macro             | Layout    | Output regs                |
  | ----------------- | --------- | -------------------------- |
  | `decode_abc!`     | Abc       | a, b, c (3 byte operands)  |
  | `decode_abc_slot!`| AbcSlot   | a, b, c, slot (4 operands) |
  | `decode_abx!`     | Abx       | a, bx (1 byte + 1 u16)     |
  | `decode_ax!`      | Ax        | ax (1 u32)                 |
  | ...               | ...       | ...                        |

  ## Register file

  | Macro          | Description                            |
  | -------------- | -------------------------------------- |
  | `load_reg!`    | Read register `[REGS + idx*8]`         |
  | `store_reg!`   | Write register `[REGS + idx*8]`        |
  | `load_acc!`    | Read accumulator (register 0)          |
  | `store_acc!`   | Write accumulator (register 0)         |

  ## Value tag checks (NaN-tagged Value)

  | Macro                 | Effect                                          |
  | --------------------- | ----------------------------------------------- |
  | `check_smi!`          | Branch to label if `reg` is not an SMI          |
  | `check_object_ref!`   | Branch to label if `reg` is not an ObjectRef    |
  | `check_undefined!`    | Branch to label if `reg` is not undefined       |
  | ...                   | ...                                             |

  ## Arithmetic

  | Macro                 | Effect                                          |
  | --------------------- | ----------------------------------------------- |
  | `add_smi_overflow!`   | 32-bit add with branch-on-overflow              |
  | ...                   | ...                                             |

  ## Dispatch & slow-path bridge

  | Macro                  | Effect                                       |
  | ---------------------- | -------------------------------------------- |
  | `dispatch!()`          | Tail-jump with auto-advance (handler length) |
  | `dispatch!(advance=N)` | Tail-jump after explicit advance             |
  | `call_slow!(fn, args)` | Bridge to Rust slow-path shim                |
  | `dispatch_after_slow!()`| Post-call dispatch on Continue/Refresh/Exit |
  | `dispatch_prefixed!`   | Prefix-opcode tail-jump                      |
  ```

  Fill in every operation defined in B21–B27 with its AArch64 fragment cost. The doc lives next to the source for ease of audit.

- [ ] **Step 2: Commit**

  ```sh
  git add crates/vm/src/dsl/ops.md
  git commit -m "DSL-0b: DSL vocabulary documentation"
  ```

---

### Task B29: DSL_DISPATCH_TABLE assembly

**Files:**
- Modify: `crates/vm/src/dsl/handlers/mod.rs`

- [ ] **Step 1: Populate the table from `OPCODES` manifest**

  Replace the placeholder table in `handlers/mod.rs`:
  ```rust
  use lyng_bytecode::OPCODE_COUNT;

  pub type DslHandler = unsafe extern "C" fn() -> !;

  /// Built at compile time from the `OPCODES` manifest. Each slot points
  /// to a real DSL handler (hot / warm / cold). Slots beyond OPCODE_COUNT
  /// point to a panic stub.
  pub static DSL_DISPATCH_TABLE: [DslHandler; 256] = build_dsl_dispatch_table();

  const fn build_dsl_dispatch_table() -> [DslHandler; 256] {
      // Const-evaluable initialization is constrained — at minimum, walk
      // OPCODES and assign each entry's `dsl_handler_symbol` resolved
      // function. Since `OPCODES` contains string symbols (not fn ptrs),
      // we need a parallel `DSL_HANDLER_FN_PTRS` slice, similar to the
      // SEMANTIC_FN_PTRS slice from Task A19.
      let mut table: [DslHandler; 256] = [unimplemented_handler; 256];
      // Populated as hot / warm / cold ports land in B39–B48.
      table
  }

  unsafe extern "C" fn unimplemented_handler() -> ! {
      loop {} // SAFETY: never reachable until DSL-0c flips dispatch
  }
  ```

- [ ] **Step 2: Add submodules for handler families**

  ```rust
  pub mod hot;
  pub mod warm;
  pub mod cold;
  ```

- [ ] **Step 3: Verify**

  ```sh
  cargo build -p lyng-vm
  ```

- [ ] **Step 4: Commit**

  ```sh
  git add crates/vm/src/dsl/handlers/
  git commit -m "DSL-0b: DSL_DISPATCH_TABLE skeleton + handler family modules"
  ```

---

### Task B30: Validation case 1 — empty naked handler compiles

**Files:**
- Create: `crates/vm/tests/dsl_validation_empty.rs`

This is the load-bearing first proof. If a trivial `llint_handler!` invocation can be expanded to `#[unsafe(naked)] extern "C" fn { naked_asm!(...) }` and compile cleanly, the proc-macro + backend integration is viable. If this fails, the DSL design needs revision before scaling.

- [ ] **Step 1: Author the test**

  Create `crates/vm/tests/dsl_validation_empty.rs`:
  ```rust
  //! DSL-0b validation case 1 (design §10): an empty naked handler
  //! compiles and is callable.

  use lyng_vm_dsl::llint_handler;

  llint_handler! {
      op_validation_empty, layout = None, length = 1, || {
          dispatch!(advance = 0);
      }
  }

  #[test]
  fn empty_handler_symbol_exists() {
      // Take the function's address to force the linker to keep it.
      let ptr = op_validation_empty as *const ();
      assert!(!ptr.is_null());
  }
  ```

- [ ] **Step 2: Run**

  ```sh
  cargo test -p lyng-vm --test dsl_validation_empty 2>&1 | tail -20
  ```
  Expected: PASS. If it fails, the proc-macro/backend integration is the cause; investigate before proceeding.

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/tests/dsl_validation_empty.rs
  git commit -m "DSL-0b: validation case 1 (empty naked handler compiles)"
  ```

---

### Task B31: Validation case 2 — slow-path round-trip (4 outcomes)

**Files:**
- Create: `crates/vm/tests/dsl_validation_slow_roundtrip.rs`

Drives a contrived handler through each of the four slow-path outcomes (Continue, Refresh, Exit-Done, Exit-Error) and confirms the bridge dispatches correctly.

- [ ] **Step 1: Author the test**

  Create `crates/vm/tests/dsl_validation_slow_roundtrip.rs`:
  ```rust
  //! DSL-0b validation case 2 (design §10): each slow-path tag dispatches
  //! correctly.
  //!
  //! Constructs four trivial DSL handlers, each with a slow-path body
  //! that produces a different `SemanticOutcome`. Drives one byte of
  //! bytecode through each and verifies the post-call dispatch state.

  // Full implementation requires:
  //   - 4 cold-stub-style llint_handler! invocations
  //   - A test fixture that calls run_via_dsl with each as the only handler
  //   - Assertions on rust_context.exit and PC offset
  //
  // The fixture interface lives in crates/vm/src/dsl/test_helpers.rs
  // (created in this task).

  // ... (test body following the fixture API in test_helpers.rs)
  ```

  Create `crates/vm/src/dsl/test_helpers.rs` (or behind `#[cfg(test)]`):
  ```rust
  //! Minimal harness for driving a DSL handler through the trampoline.
  //!
  //! Used by validation cases 2–9.

  pub struct DslHarness {
      // ... constructs a single-handler dispatch table + state ...
  }

  impl DslHarness {
      pub fn run_one_handler(handler: unsafe extern "C" fn() -> !) -> HarnessOutcome {
          // ... builds LlIntState, calls trampoline-equivalent, reads exit ...
      }
  }

  pub enum HarnessOutcome {
      Continued { new_pc_offset: u32 },
      Refreshed,
      Done { value: lyng_types::Value },
      Error,
  }
  ```

  Concrete fixture implementation is in scope for this task. The handler-emission lines for each outcome are normal `llint_handler!` blocks with `call_slow!` to four distinct Rust shims that produce the four outcomes.

- [ ] **Step 2: Run**

  ```sh
  cargo test -p lyng-vm --test dsl_validation_slow_roundtrip
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/tests/dsl_validation_slow_roundtrip.rs \
          crates/vm/src/dsl/test_helpers.rs
  git commit -m "DSL-0b: validation case 2 (slow-path 4-outcome round-trip)"
  ```

---

### Task B32: Validation case 3 — PC-sync correctness

**Files:**
- Create: `crates/vm/tests/dsl_validation_pc_sync.rs`

Verifies the pre-slow-path PC sync from design §6: a semantic body reading `state.frame.instruction_offset()` sees the post-dispatch PC, not stale data.

- [ ] **Step 1: Author the test**

  ```rust
  //! DSL-0b validation case 3: pre-slow-path PC sync correctness.

  use lyng_vm::dsl::test_helpers::DslHarness;
  use lyng_vm_dsl::llint_handler;

  // A cold-stub-style handler that calls a semantic body asserting the
  // observed instruction_offset matches the entry-PC.
  fn op_pc_sync_semantic(state: &mut lyng_vm::dsl::slow_path::LlIntDispatchState<'_, '_>, _args: ()) -> lyng_vm::dsl::slow_path::SemanticOutcome {
      // Read the PC offset via sync_from_asm'd FrameRecord.
      // The harness configures the entry PC to 0x00 — we assert the
      // semantic body sees 0x00, not whatever stale value from before.
      // ... assertion here ...
      lyng_vm::dsl::slow_path::SemanticOutcome::Continue { pc_advance: 4 }
  }

  // dsl_cold_shim! generates op_pc_sync_slow_rs.

  llint_handler! {
      op_pc_sync, layout = None, length = 4, || {
          call_slow!(op_pc_sync_slow_rs, args = []);
          dispatch_after_slow!();
      }
  }

  #[test]
  fn semantic_body_sees_post_dispatch_pc() {
      let outcome = DslHarness::run_one_handler(op_pc_sync);
      // The harness asserts on completion that no assertion-fail
      // occurred inside op_pc_sync_semantic.
      assert!(matches!(outcome, lyng_vm::dsl::test_helpers::HarnessOutcome::Continued { .. }));
  }
  ```

- [ ] **Step 2: Run**

  ```sh
  cargo test -p lyng-vm --test dsl_validation_pc_sync
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/tests/dsl_validation_pc_sync.rs
  git commit -m "DSL-0b: validation case 3 (PC-sync correctness)"
  ```

---

### Task B33: Validation case 4 — safepoint on `op_loop_header`

**Files:**
- Create: `crates/vm/tests/dsl_validation_safepoint_loop_header.rs`

- [ ] **Step 1: Author the test**

  Tight loop of `op_add` + `op_loop_header` under contrived poll-flag-always-set conditions: confirm the GC poll fires at least once.

  ```rust
  //! DSL-0b validation case 4 — see design §6 / §10.
  use lyng_vm::test_helpers::compile_and_run_with_poll_forced;

  #[test]
  fn loop_header_warm_path_polls_when_pending_set() {
      // Compile JS that produces `op_add` + `op_loop_header` in a tight loop.
      // Force VM.poll_pending = GC_PENDING before each dispatch.
      // Confirm the slow-path poll counter increments ≥ 1.
      let result = compile_and_run_with_poll_forced(
          "let i = 0; for (let j = 0; j < 100; j += 1) { i += j }",
      );
      assert!(result.poll_fired_count >= 1);
  }
  ```

  `compile_and_run_with_poll_forced` is a test helper to be added under `crates/vm/src/test_helpers.rs` if not present.

- [ ] **Step 2: Run**

  ```sh
  cargo test -p lyng-vm --test dsl_validation_safepoint_loop_header
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/tests/dsl_validation_safepoint_loop_header.rs \
          crates/vm/src/test_helpers.rs
  git commit -m "DSL-0b: validation case 4 (safepoint on op_loop_header)"
  ```

---

### Task B34: Validation case 5 — safepoint on backward unconditional jump

**Files:**
- Create: `crates/vm/tests/dsl_validation_safepoint_backward_jump.rs`

Tight loop using `op_add` + negative `op_jump` (no `op_loop_header`): confirm the GC poll fires.

- [ ] **Step 1: Author the test**

  Similar to B33; emit a loop that uses backward `op_jump` rather than `op_loop_header` (compiler may need a hint to emit this shape; otherwise write a manual bytecode sequence via the existing bytecode builder).

  ```rust
  #[test]
  fn backward_jump_warm_path_polls_when_pending_set() {
      // Build bytecode by hand: op_load_zero r0 ; op_loop_header ; op_add r0,r0,r1 ; op_jump -N
      // (or use the BytecodeBuilder API to emit this without op_loop_header).
      // Force poll_pending=GC_PENDING. Assert poll fires.
      // ...
  }
  ```

- [ ] **Step 2: Run**

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/tests/dsl_validation_safepoint_backward_jump.rs
  git commit -m "DSL-0b: validation case 5 (safepoint on backward op_jump)"
  ```

---

### Task B35: Validation case 6 — safepoint on backward conditional jump

**Files:**
- Create: `crates/vm/tests/dsl_validation_safepoint_backward_cond_jump.rs`

Tight loop using a conditional backward `op_jump_if_true(/8)` or `op_jump_if_false(/8)`: confirm the GC poll fires when the taken branch is backward.

- [ ] **Step 1: Author the test**

- [ ] **Step 2: Run**

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/tests/dsl_validation_safepoint_backward_cond_jump.rs
  git commit -m "DSL-0b: validation case 6 (safepoint on backward conditional jump)"
  ```

---

### Task B36: Validation case 7 — Wide prefix decode

**Files:**
- Create: `crates/vm/tests/dsl_validation_prefix_wide.rs`

Drives `op_wide` + `op_move` (wide-form operands) through the DSL trampoline and asserts wide register operands decode correctly.

- [ ] **Step 1: Author the test**

  ```rust
  use lyng_vm::test_helpers::run_dsl_handlers;

  #[test]
  fn wide_prefix_decodes_wide_op_move() {
      // bytes = [Wide, Move (wide form), u16 dst, u16 src, ...]
      let bytes = [
          /* Wide */ 116, /* Move */ 1,
          /* dst */ 0x12, 0x34,
          /* src */ 0x56, 0x78,
      ];
      // ... run via DSL harness ...
      // assert dst register == read src register (u16-width operand)
  }
  ```

- [ ] **Step 2: Run**

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/tests/dsl_validation_prefix_wide.rs
  git commit -m "DSL-0b: validation case 7 (Wide prefix decode)"
  ```

---

### Task B37: Validation case 8 — ExtraWide prefix decode

**Files:**
- Create: `crates/vm/tests/dsl_validation_prefix_extra_wide.rs`

Same shape as B36 but with `op_extra_wide` and u32-width operands.

- [ ] **Step 1: Author the test**

- [ ] **Step 2: Run**

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/tests/dsl_validation_prefix_extra_wide.rs
  git commit -m "DSL-0b: validation case 8 (ExtraWide prefix decode)"
  ```

---

### Task B38: Validation case 9 — Double-prefix rejection

**Files:**
- Create: `crates/vm/tests/dsl_validation_prefix_double.rs`

`op_wide` + `op_wide` raises the expected `VmError::DoublePrefix` (added in Task A18) via the `op_double_prefix_slow_rs` path documented in the design.

- [ ] **Step 1: Author the test**

  ```rust
  #[test]
  fn double_prefix_raises_error() {
      let bytes = [/* Wide */ 116, /* Wide */ 116, /* arbitrary */ 0, 0, 0, 0];
      let result = run_dsl_harness_until_exit(bytes);
      assert!(matches!(result, HarnessOutcome::Error));
      // Optionally inspect the captured error to assert it's DoublePrefix.
  }
  ```

- [ ] **Step 2: Run**

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/tests/dsl_validation_prefix_double.rs
  git commit -m "DSL-0b: validation case 9 (double-prefix rejection)"
  ```

---

### Task B39: Hot port — `op_move` DSL body

**Files:**
- Modify: `crates/vm/src/dsl/handlers/hot.rs`
- Modify: `crates/vm/src/dsl/opcode_manifest.rs`
- Create: `reports/lyng/dsl-handlers/op_move.md`
- Modify: `reports/lyng/dsl-asm-baseline-aarch64/op_move.asm`

Port the simplest hot opcode first — `op_move` is just `load_reg → store_reg → dispatch`. Confirms the DSL emits an asm shape close to LLInt's `mov` handler.

- [ ] **Step 1: Author the handler**

  In `crates/vm/src/dsl/handlers/hot.rs`:
  ```rust
  use lyng_vm_dsl::llint_handler;

  llint_handler! {
      op_move, layout = Ab, length = 3, |dst, src| {
          load_reg!(src => t0);
          store_reg!(dst, t0);
          dispatch!();
      }
  }
  ```

  The handler is a tail-jump body with two memory ops + dispatch. LLInt's `op_mov` for comparison emits the same three operations.

- [ ] **Step 2: Update manifest**

  Change `op_move`'s `OpcodeEntry.category` from `Cold` (the default during A8 registration) to `Hot`, and update `dsl_handler_symbol` to `"lyng_vm::dsl::handlers::hot::op_move"`.

- [ ] **Step 3: Capture handler asm + diff against LLInt reference**

  ```sh
  cargo run --release -p lyng-bench -- asm-diff --opcodes op_move --output /tmp/asm/ --mode update
  cp /tmp/asm/op_move.asm reports/lyng/dsl-asm-baseline-aarch64/op_move.asm
  ```

- [ ] **Step 4: Write per-handler report**

  Create `reports/lyng/dsl-handlers/op_move.md`:
  ```markdown
  # `op_move` DSL port

  ## DSL source

  See `crates/vm/src/dsl/handlers/hot.rs`.

  ## Current asm (AArch64)

  See `reports/lyng/dsl-asm-baseline-aarch64/op_move.asm`.

  ## LLInt reference

  See `reports/lyng/llint-reference/op_mov.asm`.

  ## Side-by-side diff

  | Line | DSL                     | LLInt                  | Notes                        |
  | ---- | ----------------------- | ---------------------- | ---------------------------- |
  | ...  | ...                     | ...                    | ...                          |

  ## Microbench

  | Metric         | Pre-DSL-0 baseline | Post-port | Δ      |
  | -------------- | ------------------ | --------- | ------ |
  | ns/dispatch    | <X>                | <Y>       | <Δ>%   |

  ## Behavioral tests

  - tests/dsl_validation_empty.rs covers basic register-file read/write.
  - existing `op_move` tests in `crates/vm/tests/` continue to pass.
  ```

- [ ] **Step 5: Verify**

  ```sh
  cargo build -p lyng-vm
  cargo test -p lyng-vm
  ```

- [ ] **Step 6: Commit**

  ```sh
  git add crates/vm/src/dsl/handlers/hot.rs \
          crates/vm/src/dsl/opcode_manifest.rs \
          reports/lyng/dsl-handlers/op_move.md \
          reports/lyng/dsl-asm-baseline-aarch64/op_move.asm
  git commit -m "DSL-0b: hot port op_move + ported report + asm baseline"
  ```

---

### Task B40: Hot port — `op_add` DSL body

**Files:**
- Modify: `crates/vm/src/dsl/handlers/hot.rs`
- Modify: `crates/vm/src/dsl/opcode_manifest.rs`
- Create: `reports/lyng/dsl-handlers/op_add.md`
- Create: `reports/lyng/dsl-asm-baseline-aarch64/op_add.asm`

- [ ] **Step 1: Author the handler**

  ```rust
  llint_handler! {
      op_add, layout = AbcSlot, length = 6, |a, b, c, slot| {
          load_reg!(b => t0);
          check_smi!(t0, .slow);
          load_reg!(c => t1);
          check_smi!(t1, .slow);
          untag_smi!(t0);
          untag_smi!(t1);
          add_smi_overflow!(t0, t1 => t2, .slow);
          tag_smi!(t2);
          store_reg!(a, t2);
          record_smi!(slot);
          dispatch!();

        .slow:
          call_slow!(op_add_slow_rs, args = [a, b, c, slot]);
          dispatch_after_slow!();
      }
  }
  ```

  The shim `op_add_slow_rs` is generated by `dsl_cold_shim!` over `crate::vm::semantics::arithmetic::op_add_semantic`.

- [ ] **Step 2: Generate the shim**

  In `crates/vm/src/dsl/handlers/hot.rs`, append:
  ```rust
  use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};
  use crate::vm::semantics::arithmetic::{op_add_semantic, OpAddArgs};

  crate::dsl_cold_shim! {
      op_add_slow_rs
      for crate::vm::semantics::arithmetic::op_add_semantic
      with OpAddArgs
      { a: u32, b: u32, c: u32, slot: u32 }
  }
  ```

  Note: `OpAddArgs` fields' types may need `From<u32>` adapters if they're `Option<FeedbackSlotId>` rather than `u32`. Adjust `OpAddArgs` field types or add a conversion shim.

- [ ] **Step 3: Update manifest, capture asm, write report**

  Same shape as B39.

- [ ] **Step 4: Verify**

  ```sh
  cargo build -p lyng-vm
  cargo test -p lyng-vm
  cargo run --release -p lyng-bench -- microbench --opcodes op_add --samples 5 --iters 1000000 --report /tmp/op_add_micro.md
  ```

- [ ] **Step 5: Commit**

  ```sh
  git add crates/vm/src/dsl/handlers/hot.rs \
          crates/vm/src/dsl/opcode_manifest.rs \
          reports/lyng/dsl-handlers/op_add.md \
          reports/lyng/dsl-asm-baseline-aarch64/op_add.asm
  git commit -m "DSL-0b: hot port op_add + slow-path shim + ported report"
  ```

---

### Task B41: Hot port — `op_jump` DSL body (with backward poll)

**Files:**
- Modify: `crates/vm/src/dsl/handlers/hot.rs`
- Modify: `crates/vm/src/dsl/opcode_manifest.rs`
- Create: `reports/lyng/dsl-handlers/op_jump.md`
- Create: `reports/lyng/dsl-asm-baseline-aarch64/op_jump.asm`

- [ ] **Step 1: Author the handler**

  ```rust
  llint_handler! {
      op_jump, layout = Ax, length = 5, |offset| {
          // Forward jump: just advance and dispatch.
          // Backward jump (sign of offset): poll first.
          // Sign check: top bit of offset register.
          // Layout: offset is i32 (Ax = 4-byte operand after opcode).
          load_byte!(PC, #1 => t0);           // low byte of offset
          // Composite read: load full 32-bit offset
          // (omitted — verify layout decoder emits the right asm).
          // Simplified: check signum via signed compare to 0.
          // Backward jump = offset < 0; check via tbnz on sign bit.
          // For now treat unconditional jumps as warm regardless.
          poll_safepoint!(.poll_pending);
          dispatch!(jump_to = compute_target);

        .poll_pending:
          call_slow!(op_jump_poll_rs, args = []);
          dispatch_after_slow!();
      }
  }
  ```

  Note: the full `op_jump` DSL body interacts with the offset decode. Substitute the exact prelude/macros required to compute the target PC. If the design's `dispatch!(jump_to=...)` form is not yet supported by the lowerer, extend it in this task or fall back to `call_slow!` for the offset computation.

  This task is the first one where the `dispatch!(jump_to = ...)` form is exercised. If it's not in the backend, add it in `backend/aarch64/control.rs`.

- [ ] **Step 2: Generate the slow-path shim for the poll**

  ```rust
  crate::dsl_cold_shim! {
      op_jump_poll_rs
      for crate::dsl::poll::run_poll
      with crate::dsl::poll::PollArgs
      { }
  }
  ```

  `crate::dsl::poll` is a new module hosting the consumed-bit polling logic: read `vm.poll_pending`, run GC step / debugger pause for set bits, clear consumed bits.

- [ ] **Step 3: Create the poll module**

  Create `crates/vm/src/dsl/poll.rs`:
  ```rust
  //! Same-thread safepoint poll consumer per design §6.

  use crate::dsl::slow_path::{LlIntDispatchState, SemanticOutcome};

  pub struct PollArgs;

  pub fn run_poll(state: &mut LlIntDispatchState<'_, '_>, _args: PollArgs) -> SemanticOutcome {
      // Read vm.poll_pending; for each set bit, run the relevant work.
      let vm = match &mut state.inner {
          crate::dsl::slow_path::LlIntDispatchInner::Asm { rust, .. } => &mut *rust.vm,
          crate::dsl::slow_path::LlIntDispatchInner::Alpha(ds) => &mut *ds.vm,
      };
      let bits = vm.poll_pending;
      if bits & 0x01 != 0 {
          vm.run_incremental_gc_step();
          vm.poll_pending &= !0x01;
      }
      if bits & 0x02 != 0 {
          vm.handle_debug_pause_request();
          vm.poll_pending &= !0x02;
      }
      SemanticOutcome::Continue { pc_advance: 0 }
  }
  ```

  `Vm::poll_pending`, `Vm::run_incremental_gc_step`, and `Vm::handle_debug_pause_request` must exist or be added (often stubs in DSL-0b — they're populated by GC and debugger work outside this plan).

- [ ] **Step 4: Update manifest, capture asm, write report**

- [ ] **Step 5: Verify**

  ```sh
  cargo build -p lyng-vm
  cargo test -p lyng-vm --test dsl_validation_safepoint_backward_jump
  ```

- [ ] **Step 6: Commit**

  ```sh
  git add crates/vm/src/dsl/handlers/hot.rs \
          crates/vm/src/dsl/poll.rs \
          crates/vm/src/dsl/opcode_manifest.rs \
          reports/lyng/dsl-handlers/op_jump.md \
          reports/lyng/dsl-asm-baseline-aarch64/op_jump.asm \
          crates/vm/src/vm.rs
  git commit -m "DSL-0b: hot port op_jump + safepoint poll module + report"
  ```

---

### Task B42: Hot port — `op_return` DSL body

**Files:**
- Modify: `crates/vm/src/dsl/handlers/hot.rs`
- Modify: `crates/vm/src/dsl/opcode_manifest.rs`
- Create: `reports/lyng/dsl-handlers/op_return.md`
- Create: `reports/lyng/dsl-asm-baseline-aarch64/op_return.asm`

`op_return` is frame-transitioning — always returns `Refresh` or `ExitDone`. The DSL body is short: `load_reg → call_slow → dispatch_after_slow` with the slow-path doing the actual frame pop.

- [ ] **Step 1: Author the handler**

  ```rust
  llint_handler! {
      op_return, layout = A, length = 2, |src| {
          call_slow!(op_return_slow_rs, args = [src]);
          dispatch_after_slow!();
      }
  }

  crate::dsl_cold_shim! {
      op_return_slow_rs
      for crate::vm::semantics::control_flow::op_return_semantic
      with crate::vm::semantics::control_flow::OpReturnArgs
      { src: u32 }
  }
  ```

  Note: this is essentially a cold-stub shape, but `op_return` is hot per dispatch share. The optimization-to-evaluate from §6 ("op_return Refresh overhead vs Continue: 5 extra instructions × ~1-2% dispatch share") gets quantified in step 5.

- [ ] **Step 2: Capture asm + write report**

- [ ] **Step 3: Manifest update**

- [ ] **Step 4: Verify**

- [ ] **Step 5: Microbench**

  ```sh
  cargo run --release -p lyng-bench -- microbench --opcodes op_return --samples 7 --iters 5000000 --report /tmp/op_return_micro.md
  ```

  Compare to pre-DSL-0 baseline. If `op_return` is more than 5% slower than the alpha path, file a follow-up to introduce the same-code-unit fast-return shortcut from design §6 — but do NOT block DSL-0b on it.

- [ ] **Step 6: Commit**

  ```sh
  git add crates/vm/src/dsl/handlers/hot.rs \
          crates/vm/src/dsl/opcode_manifest.rs \
          reports/lyng/dsl-handlers/op_return.md \
          reports/lyng/dsl-asm-baseline-aarch64/op_return.asm
  git commit -m "DSL-0b: hot port op_return + ported report + microbench"
  ```

---

### Task B43: Warm port — `op_loop_header` DSL body

**Files:**
- Modify: `crates/vm/src/dsl/handlers/warm.rs`
- Modify: `crates/vm/src/dsl/opcode_manifest.rs`
- Create: `reports/lyng/dsl-handlers/op_loop_header.md`
- Create: `reports/lyng/dsl-asm-baseline-aarch64/op_loop_header.asm`

- [ ] **Step 1: Author the handler**

  ```rust
  llint_handler! {
      op_loop_header, layout = Ax, length = 4, |_unused_target_offset| {
          poll_safepoint!(.poll_pending);
          dispatch!(advance = 4);

        .poll_pending:
          call_slow!(op_loop_header_poll_rs, args = []);
          dispatch_after_slow!();
      }
  }

  crate::dsl_cold_shim! {
      op_loop_header_poll_rs
      for crate::dsl::poll::run_poll
      with crate::dsl::poll::PollArgs
      { }
  }
  ```

- [ ] **Step 2: Manifest update**

- [ ] **Step 3: Verify against validation case 4**

  ```sh
  cargo test -p lyng-vm --test dsl_validation_safepoint_loop_header
  ```

- [ ] **Step 4: Capture asm + report**

- [ ] **Step 5: Commit**

  ```sh
  git add crates/vm/src/dsl/handlers/warm.rs \
          crates/vm/src/dsl/opcode_manifest.rs \
          reports/lyng/dsl-handlers/op_loop_header.md \
          reports/lyng/dsl-asm-baseline-aarch64/op_loop_header.asm
  git commit -m "DSL-0b: warm port op_loop_header + safepoint coverage"
  ```

---

### Task B44: Warm ports — `op_jump8` + conditional jumps with backward poll

**Files:**
- Modify: `crates/vm/src/dsl/handlers/warm.rs`
- Modify: `crates/vm/src/dsl/opcode_manifest.rs`
- Create: `reports/lyng/dsl-handlers/op_jump8.md`, `op_jump_if_true.md`, `op_jump_if_true8.md`, `op_jump_if_false.md`, `op_jump_if_false8.md`
- Create per-opcode asm baseline files

- [ ] **Step 1: Port each warm jump handler**

  For each of `op_jump8`, `op_jump_if_true`, `op_jump_if_true8`, `op_jump_if_false`, `op_jump_if_false8`, author a DSL handler that:
  1. Decodes the offset (i8 for `*8` variants, i32 for full-width).
  2. For conditional jumps: tests the condition register.
  3. If branch is taken and offset is negative: poll first.
  4. Then advance PC and dispatch.

  ```rust
  llint_handler! {
      op_jump8, layout = A, length = 2, |offset| {
          // offset is i8.
          // Check sign:
          // (a) negative → backward → poll first.
          // (b) positive → forward → no poll.
          tbnz_signed!(offset, .backward);
          dispatch!(advance = 2);  // forward (but jump_to handling per backend)

        .backward:
          poll_safepoint!(.poll_pending);
          dispatch!(advance_signed = offset);

        .poll_pending:
          call_slow!(op_jump8_poll_rs, args = []);
          dispatch_after_slow!();
      }
  }
  ```

  `tbnz_signed!` and `dispatch!(advance_signed = ...)` are new backend ops to add to `control.rs` if not present.

- [ ] **Step 2: Manifest entries (warm category for all 5)**

- [ ] **Step 3: Capture asm + per-opcode reports**

- [ ] **Step 4: Verify against validation cases 5 + 6**

- [ ] **Step 5: Commit**

  ```sh
  git add crates/vm/src/dsl/handlers/warm.rs \
          crates/vm/src/dsl/opcode_manifest.rs \
          reports/lyng/dsl-handlers/op_jump8.md \
          reports/lyng/dsl-handlers/op_jump_if_true.md \
          reports/lyng/dsl-handlers/op_jump_if_true8.md \
          reports/lyng/dsl-handlers/op_jump_if_false.md \
          reports/lyng/dsl-handlers/op_jump_if_false8.md \
          reports/lyng/dsl-asm-baseline-aarch64/op_jump8.asm \
          reports/lyng/dsl-asm-baseline-aarch64/op_jump_if_true.asm \
          reports/lyng/dsl-asm-baseline-aarch64/op_jump_if_true8.asm \
          reports/lyng/dsl-asm-baseline-aarch64/op_jump_if_false.asm \
          reports/lyng/dsl-asm-baseline-aarch64/op_jump_if_false8.asm
  git commit -m "DSL-0b: warm ports for backward-jump variants (op_jump8, op_jump_if_*)"
  ```

---

### Task B45: Warm ports — `op_wide` + `op_extra_wide`

**Files:**
- Modify: `crates/vm/src/dsl/handlers/warm.rs`
- Modify: `crates/vm/src/dsl/opcode_manifest.rs`
- Create: `reports/lyng/dsl-handlers/op_wide.md`, `op_extra_wide.md`

- [ ] **Step 1: Author handlers**

  ```rust
  llint_handler! {
      op_wide, layout = None, length = 1, || {
          dispatch_prefixed!(kind = Wide);
      }
  }

  llint_handler! {
      op_extra_wide, layout = None, length = 1, || {
          dispatch_prefixed!(kind = ExtraWide);
      }
  }
  ```

  `dispatch_prefixed!` was defined in B25. Verify it emits the expected double-prefix-rejection path (verified by validation case 9 in B38).

- [ ] **Step 2: Manifest + asm capture + reports**

- [ ] **Step 3: Verify against validation cases 7, 8, 9**

- [ ] **Step 4: Commit**

  ```sh
  git add crates/vm/src/dsl/handlers/warm.rs \
          crates/vm/src/dsl/opcode_manifest.rs \
          reports/lyng/dsl-handlers/op_wide.md \
          reports/lyng/dsl-handlers/op_extra_wide.md \
          reports/lyng/dsl-asm-baseline-aarch64/op_wide.asm \
          reports/lyng/dsl-asm-baseline-aarch64/op_extra_wide.asm
  git commit -m "DSL-0b: warm ports for op_wide + op_extra_wide"
  ```

---

### Task B46: Cold-stub codegen tool

**Files:**
- Create: `tools/lyng-dsl-codegen/Cargo.toml`
- Create: `tools/lyng-dsl-codegen/src/main.rs`

Generates the `~140` cold-stub `llint_handler!` invocations + their `dsl_cold_shim!` invocations from the `OPCODES` manifest. Run once at DSL-0b end; output committed to `crates/vm/src/dsl/handlers/cold.rs`.

- [ ] **Step 1: Create the tool crate**

  Create `tools/lyng-dsl-codegen/Cargo.toml`:
  ```toml
  [package]
  name = "lyng-dsl-codegen"
  version = "0.1.0"
  edition = "2021"

  [dependencies]
  lyng-bytecode = { path = "../../crates/bytecode" }
  lyng-vm = { path = "../../crates/vm" }
  ```

- [ ] **Step 2: Create the generator**

  Create `tools/lyng-dsl-codegen/src/main.rs`:
  ```rust
  //! Generates cold-stub DSL handlers + shim wrappers from OPCODES manifest.
  //!
  //! Output: crates/vm/src/dsl/handlers/cold.rs

  use lyng_vm::dsl::opcode_manifest::{OPCODES, OpcodeCategory};

  fn main() {
      let mut out = String::new();
      out.push_str("//! Auto-generated cold-stub DSL handlers. Do not edit by hand.\n");
      out.push_str("//! Re-generate via `cargo run -p lyng-dsl-codegen`.\n\n");
      out.push_str("use lyng_vm_dsl::llint_handler;\n");
      out.push_str("use crate::dsl_cold_shim;\n\n");

      for entry in OPCODES {
          if entry.category != OpcodeCategory::Cold {
              continue;
          }
          // Resolve the opcode's operand layout via lookup in lyng-bytecode.
          // For each: emit
          //   llint_handler! { op_xxx, layout = ..., length = ..., |args| {
          //       call_slow!(op_xxx_slow_rs, args = [...]);
          //       dispatch_after_slow!();
          //   }}
          //   crate::dsl_cold_shim! {
          //       op_xxx_slow_rs for crate::vm::semantics::<family>::op_xxx_semantic
          //       with crate::vm::semantics::<family>::OpXxxArgs { ... }
          //   }
          //
          // Layout/family resolution lives in this generator using:
          //   - lyng_bytecode::OperandLayout (existing)
          //   - the family name extracted from semantic_symbol string
          out.push_str(&format!("// TODO codegen for {:?}\n", entry.opcode));
      }

      let target = std::path::Path::new("crates/vm/src/dsl/handlers/cold.rs");
      std::fs::write(target, out).expect("write cold.rs");
  }
  ```

  Fill in the body to produce the actual `llint_handler!` invocations. Use `lyng_bytecode::Opcode::layout()` (or whatever the existing API is) to get the operand-layout descriptor per opcode.

- [ ] **Step 3: Verify the tool builds**

  ```sh
  cargo build -p lyng-dsl-codegen
  ```

- [ ] **Step 4: Commit**

  ```sh
  git add tools/lyng-dsl-codegen/ Cargo.toml
  git commit -m "DSL-0b: cold-stub codegen tool"
  ```

---

### Task B47: Run cold-stub codegen for all remaining opcodes

**Files:**
- Modify: `crates/vm/src/dsl/handlers/cold.rs`

- [ ] **Step 1: Run the generator**

  ```sh
  cargo run -p lyng-dsl-codegen
  ```
  Expected: `crates/vm/src/dsl/handlers/cold.rs` populated with ~140 `llint_handler!` + `dsl_cold_shim!` pairs.

- [ ] **Step 2: Verify the output compiles**

  ```sh
  cargo build -p lyng-vm 2>&1 | tail -30
  ```
  Expected: clean build. If any cold stub fails to compile (e.g. layout mismatch, missing semantic body), fix in the generator and re-run, or fix the generated file directly and update the generator to match.

- [ ] **Step 3: Update `DSL_DISPATCH_TABLE` in `handlers/mod.rs`**

  Replace the placeholder fill with the real DSL handlers from `hot.rs`, `warm.rs`, `cold.rs`. Walk `OPCODES` and assign each slot.

- [ ] **Step 4: Commit**

  ```sh
  git add crates/vm/src/dsl/handlers/cold.rs \
          crates/vm/src/dsl/handlers/mod.rs
  git commit -m "DSL-0b: generate ~140 cold stubs + populate DSL_DISPATCH_TABLE"
  ```

---

### Task B48: Spot-validate 10 representative cold stubs

**Files:**
- Create: `crates/vm/tests/dsl_cold_stub_spot_check.rs`

- [ ] **Step 1: Author spot-check tests**

  Pick 10 opcodes across families. For each, drive the DSL handler through the harness with bytecode that exercises the opcode and assert the result matches the equivalent alpha-path computation.

  Recommended picks: `op_load_undefined`, `op_load_smi`, `op_get_named_property`, `op_load_global`, `op_call0`, `op_create_object`, `op_typeof`, `op_throw`, `op_yield`, `op_close_iterator`.

  ```rust
  use lyng_vm::test_helpers::dsl_runs_equivalent_to_alpha;

  #[test]
  fn cold_stub_op_load_undefined_matches_alpha() {
      assert!(dsl_runs_equivalent_to_alpha("undefined"));
  }
  // ... 9 more.
  ```

  `dsl_runs_equivalent_to_alpha` compiles the snippet, runs it via both paths (alpha and DSL via `Vm::run_via_dsl`), and asserts the final returned value + frame state match.

- [ ] **Step 2: Run**

  ```sh
  cargo test -p lyng-vm --test dsl_cold_stub_spot_check
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/tests/dsl_cold_stub_spot_check.rs \
          crates/vm/src/test_helpers.rs
  git commit -m "DSL-0b: spot-check 10 representative cold stubs"
  ```

---

### Task B49: Capture pre-/post-DSL-0b microbench + V8 v7

**Files:** none modified beyond evidence directory.

- [ ] **Step 1: Run V8 v7 (DSL handlers compiled but dispatch still alpha)**

  ```sh
  cargo run --release -p lyng-bench -- v8suite --report /tmp/dsl-0b-v8.md --json /tmp/dsl-0b-v8.json
  cp /tmp/dsl-0b-v8.md reports/lyng/dsl-0b-v8.md
  cp /tmp/dsl-0b-v8.json reports/lyng/dsl-0b-v8.json
  ```

  Expected: geomean ≈ DSL-0a baseline. DSL handlers are dead code; FV refactor's dual-write may be visible.

- [ ] **Step 2: Run microbench for the 5 hot ports**

  ```sh
  cargo run --release -p lyng-bench -- microbench --opcodes op_move,op_add,op_jump,op_return,op_loop_header --samples 7 --iters 5000000 --report /tmp/dsl-0b-hot-microbench.md --require-isolation
  cp /tmp/dsl-0b-hot-microbench.md reports/lyng/dsl-0b-hot-microbench.md
  ```

  Note: these are measuring DSL handlers in isolation (the bench harness calls them directly). The numbers establish per-handler ns/dispatch independent of the surrounding dispatch substrate.

- [ ] **Step 3: Commit evidence**

  ```sh
  git add reports/lyng/dsl-0b-v8.md \
          reports/lyng/dsl-0b-v8.json \
          reports/lyng/dsl-0b-hot-microbench.md
  git commit -m "DSL-0b: capture V8 v7 + hot-port microbench evidence"
  ```

---

### Task B50: DSL-0b exit gate

**Files:** none modified; produces evidence.

- [ ] **Step 1: Run full validation case suite**

  ```sh
  cargo test -p lyng-vm --test dsl_validation_empty \
                          --test dsl_validation_slow_roundtrip \
                          --test dsl_validation_pc_sync \
                          --test dsl_validation_safepoint_loop_header \
                          --test dsl_validation_safepoint_backward_jump \
                          --test dsl_validation_safepoint_backward_cond_jump \
                          --test dsl_validation_prefix_wide \
                          --test dsl_validation_prefix_extra_wide \
                          --test dsl_validation_prefix_double
  ```
  Expected: all 9 PASS.

- [ ] **Step 2: Run cold-stub spot-check + manifest tests**

  ```sh
  cargo test -p lyng-vm dsl::opcode_manifest
  cargo test -p lyng-vm --test dsl_cold_stub_spot_check
  cargo test -p lyng-vm --test feedback_flat_consistency
  ```

- [ ] **Step 3: Run full focused tests + Test262**

  ```sh
  cargo test -p lyng-vm -p lyng-bytecode -p lyng-objects -p lyng-tests -p lyng-compiler
  cargo run --release -p lyng-test262 -- --report /tmp/dsl-0b-test262.md -j 4
  cp /tmp/dsl-0b-test262.md reports/lyng/dsl-0b-test262.md
  ```

- [ ] **Step 4: Write DSL-0b status report**

  Create `reports/lyng/dsl-0b-status.md`:
  ```markdown
  # DSL-0b status

  ## Deliverables

  | Deliverable | Status | Path |
  | --- | --- | --- |
  | lyng-vm-dsl proc-macro crate | done | crates/vm-dsl/ |
  | vm/src/dsl/ runtime ABI | done | crates/vm/src/dsl/ |
  | LlIntState + LlIntRustContext + LlIntExitSlot | done | crates/vm/src/dsl/llint_state.rs |
  | Slow-path bridge + LlIntDispatchState::from_raw | done | crates/vm/src/dsl/slow_path.rs |
  | Entry shim + _interpreter_exit | done | crates/vm/src/dsl/entry.rs |
  | FeedbackVector flat-array refactor | done | crates/vm/src/dsl/feedback_flat.rs |
  | AArch64 backend (operands/values/objects/arithmetic/control/feedback/memory/counters/safepoint) | done | crates/vm/src/dsl/backend/aarch64/ |
  | 9 validation cases | all passing | crates/vm/tests/dsl_validation_*.rs |
  | 5 hot ports (op_move, op_add, op_jump, op_return, op_loop_header) | done | crates/vm/src/dsl/handlers/hot.rs |
  | 5 warm ports (op_jump8, op_jump_if_*, op_wide, op_extra_wide) | done | crates/vm/src/dsl/handlers/warm.rs |
  | ~140 cold stubs | done | crates/vm/src/dsl/handlers/cold.rs |
  | Per-handler ported reports | done | reports/lyng/dsl-handlers/ |
  | Asm baselines for hot + warm handlers | done | reports/lyng/dsl-asm-baseline-aarch64/ |

  ## Exit criterion verification

  1. lyng-vm-dsl proc-macro compiles + expands real opcodes — ✓
  2. All 9 validation cases pass — ✓ (see tests/dsl_validation_*.rs)
  3. 5 hot + 5 warm + ~140 cold handlers exist + populate DSL_DISPATCH_TABLE — ✓
  4. FeedbackVector flat-array refactor lands without regressing IC fast paths — ✓ (see feedback_flat_consistency tests, dsl-0b-fv-refactor-v8.md)
  5. Manifest dsl_handler_symbol entries name real symbols — verified manually; Test 3 enabled in DSL-0c
  6. cargo build --release -p lyng-vm is clean — ✓
  7. cargo test -p lyng-vm passes; alpha is still the active dispatch — ✓

  ## V8 v7 evidence

  - Pre-DSL-0 baseline geomean: <X>
  - Post-DSL-0b geomean (alpha still active): <Y>. Δ: <delta>%.

  Report: reports/lyng/dsl-0b-v8.md.

  ## Test262 evidence

  - Pre-DSL-0 baseline: <N>
  - Post-DSL-0b: <M>. Δ: <delta>.

  Report: reports/lyng/dsl-0b-test262.md.

  ## Hand-off to DSL-0c

  DSL-0c switches `Vm::run` from `run_via_trampoline` to `run_via_dsl`,
  deletes alpha, deletes tier accounting, and verifies the single-
  implementation invariant via the structural manifest's remaining
  tests (5, 6, 7).
  ```

- [ ] **Step 5: Update dcat**

  ```sh
  dcat update <DSL0B> --status in_review
  ```

- [ ] **Step 6: Commit**

  ```sh
  git add reports/lyng/dsl-0b-status.md \
          reports/lyng/dsl-0b-test262.md
  git commit -m "DSL-0b: exit-gate status report + post-port evidence"
  ```

- [ ] **Step 7: Notify the user**

  "DSL-0b complete. All 9 validation cases pass; 5 hot + 5 warm + ~140 cold DSL handlers compiled; FeedbackVector flat-array refactor in place; alpha dispatch still active. Per-handler asm-diff reports under `reports/lyng/dsl-handlers/`. Status report at `reports/lyng/dsl-0b-status.md`. May I proceed to DSL-0c (switch dispatch + delete alpha)?"

---

## Phase C — DSL-0c: Delete alpha + verify single-implementation invariant (Tasks C1–C13)

**Goal:** Switch active dispatch from α to DSL, delete the alpha trampoline / `Step` / `DispatchState` / `DISPATCH_TABLE` / `dispatch_handlers/` / `dispatch/` machinery and tier-accounting on backedges, and verify the single-implementation invariant via manifest Tests 5, 6, and 7. Re-run all behavioral tests, microbench, and V8 v7. Produce the DSL-0 decision document.

**Estimated duration:** 1 week.

**Exit criteria** (the seven DSL-0 exit criteria from design §10):
1. Single-implementation invariant via manifest (Tests 1–7 all pass).
2. Asm shape vs LLInt within 5 instructions per hot handler (plus documented Value-layout deltas).
3. Microbench vs LLInt-equivalent within 2× per hot handler.
4. Behavioral parity (focused tests pass; Test262 ≥ Pre-flight 7 baseline).
5. V8 v7 directional check: geomean ≥ +20% vs pre-DSL-0; Richards ≥ +30%.
6. All 9 DSL-0b validation cases still pass.
7. Per-opcode dispatch-counter output differs only in legitimately changed opcodes.

---

### Task C1: Switch active dispatch path

**Files:**
- Modify: `crates/vm/src/vm.rs`

- [ ] **Step 1: Find and update `Vm::run` (or its callee `run_via_trampoline`)**

  Locate `pub fn run` on `Vm`. Replace the body that called `run_via_trampoline` with a call to `run_via_dsl`. Keep the trampoline function around for one commit so the rollback diff is small; delete in Task C2.

  ```rust
  pub fn run(...) -> VmResult<Value> {
      // ... entry checks ...
      self.run_via_dsl(agent, host, registry, installed, frame)
  }
  ```

- [ ] **Step 2: Run focused tests**

  ```sh
  cargo test -p lyng-vm -p lyng-bytecode -p lyng-objects -p lyng-tests -p lyng-compiler 2>&1 | tail -30
  ```
  Expected: all tests pass. **If anything fails, this is the gate** — debug and fix before continuing.

- [ ] **Step 3: Run Test262 sanity slice**

  ```sh
  cargo run --release -p lyng-test262 -- --filter built-ins/Array --report /tmp/dsl-0c-array.md -j 4
  ```
  Expected: pass count matches pre-DSL-0c baseline on the same filter.

- [ ] **Step 4: Commit**

  ```sh
  git add crates/vm/src/vm.rs
  git commit -m "DSL-0c: switch Vm::run to dispatch through DSL trampoline"
  ```

---

### Task C2: Delete `dispatch_handlers/` directory

**Files:**
- Delete: `crates/vm/src/vm/dispatch_handlers/` (entire directory)
- Modify: `crates/vm/src/vm.rs` (remove `pub mod dispatch_handlers;`)
- Modify: `crates/vm/src/vm/dispatch_state.rs` (the dispatch table built from dispatch_handlers no longer needs the family imports)

- [ ] **Step 1: Remove the module**

  ```sh
  rm -r crates/vm/src/vm/dispatch_handlers
  ```

- [ ] **Step 2: Remove its declaration**

  Open `crates/vm/src/vm.rs` (or wherever the module is declared) and remove the line `pub(crate) mod dispatch_handlers;` (or similar).

- [ ] **Step 3: Run focused tests**

  ```sh
  cargo build -p lyng-vm 2>&1 | tail -30
  ```
  Expected: clean build because `Vm::run` no longer routes through `run_trampoline`, which is the only consumer of `dispatch_handlers/`.

  If `run_trampoline` still references it, defer this task until C5 deletes the trampoline machinery. Re-order if necessary.

- [ ] **Step 4: Commit**

  ```sh
  git rm -r crates/vm/src/vm/dispatch_handlers/
  git add crates/vm/src/vm.rs
  git commit -m "DSL-0c: delete dispatch_handlers/ (152 thinned α handlers)"
  ```

---

### Task C3: Delete `dispatch_state.rs` (alpha types)

**Files:**
- Delete: `crates/vm/src/vm/dispatch_state.rs`
- Modify: `crates/vm/src/vm.rs`

- [ ] **Step 1: Remove the module**

  ```sh
  rm crates/vm/src/vm/dispatch_state.rs
  ```

- [ ] **Step 2: Remove its declaration + any imports**

  In `crates/vm/src/vm.rs` remove `pub(crate) mod dispatch_state;`. Search for `dispatch_state::` references and remove them (the `LlIntDispatchInner::Alpha` variant in `dsl/slow_path.rs` will also go away — replace it with a single non-enum `Asm` storage).

- [ ] **Step 3: Simplify `LlIntDispatchState`**

  In `crates/vm/src/dsl/slow_path.rs`:
  ```rust
  pub struct LlIntDispatchState<'vm, 'borrow> {
      pub(crate) state: *mut LlIntState,
      pub(crate) rust:  &'borrow mut LlIntRustContext<'vm>,
  }
  ```

  Drop the `LlIntDispatchInner` enum entirely. Update all consumers.

- [ ] **Step 4: Verify**

  ```sh
  cargo build -p lyng-vm
  cargo test -p lyng-vm
  ```

- [ ] **Step 5: Commit**

  ```sh
  git rm crates/vm/src/vm/dispatch_state.rs
  git add crates/vm/src/vm.rs \
          crates/vm/src/dsl/slow_path.rs
  git commit -m "DSL-0c: delete dispatch_state.rs + simplify LlIntDispatchState"
  ```

---

### Task C4: Delete `dispatch/` α-only helpers

**Files:**
- Delete: `crates/vm/src/vm/dispatch/` (if its contents are no longer used)
- Modify: `crates/vm/src/vm.rs`

Most `execute_*_opcode` methods on `Vm` should have been replaced by `semantics/*` free functions during DSL-0a. Audit and remove anything in `dispatch/` that's no longer referenced.

- [ ] **Step 1: Audit references**

  ```sh
  rg -n "vm/dispatch::\|self\.execute_.*_opcode" crates/vm/src/ | head -40
  ```

  For each remaining reference, decide: move into `semantics/`, or keep as a private helper if it's still shared.

- [ ] **Step 2: Delete unused files**

  If `dispatch/arithmetic.rs` and `dispatch/property.rs` are still referenced (e.g. their helper functions like `decode_smi_immediate`, `smi_mul_result`, `smi_mod_result` are referenced from `semantics/`), keep those helpers but move them out of `dispatch/`. Move to `crates/vm/src/vm/arithmetic_helpers.rs` (or similar) and delete the `dispatch/` directory.

- [ ] **Step 3: Verify**

  ```sh
  cargo build -p lyng-vm
  cargo test -p lyng-vm
  ```

- [ ] **Step 4: Commit**

  ```sh
  git rm -r crates/vm/src/vm/dispatch/
  git add crates/vm/src/vm.rs
  git commit -m "DSL-0c: delete vm/dispatch/ α-only helpers, relocate kept helpers"
  ```

---

### Task C5: Delete `run_trampoline_uncounted` + `Step` enum + `DISPATCH_TABLE`

**Files:**
- Already covered by C2 + C3 deletion. This task is the sanity check.

- [ ] **Step 1: Grep for remaining α-machinery references**

  ```sh
  rg -n "run_trampoline\|run_trampoline_uncounted\|run_trampoline_counted\|Step::\|DISPATCH_TABLE\|dispatch_handlers" crates/vm/src/
  ```
  Expected: no matches outside of test fixtures or migration helpers that should also be removed.

- [ ] **Step 2: Remove any stragglers found**

  Edit the files listed by the grep until the search returns nothing.

- [ ] **Step 3: Verify**

  ```sh
  cargo build -p lyng-vm
  cargo test -p lyng-vm
  ```

- [ ] **Step 4: Commit (if changes made)**

  ```sh
  git add -u
  git commit -m "DSL-0c: clean up remaining alpha-trampoline references"
  ```

---

### Task C6: Delete tier-accounting calls on backedges

**Files:**
- Delete: `crates/vm/src/vm/tiering.rs` (if it exists)
- Modify: any consumer

Per design §6 + §10: "tier-accounting machinery on backedges goes away with the alpha path. After DSL-0c, the interpreter has no tier-up accounting — this is intentional, per §2 (JIT is out of scope)."

- [ ] **Step 1: Audit `tiering.rs` references**

  ```sh
  rg -n "observe_tier_backedge_event\|tier_up_counter\|tiering::" crates/vm/src/
  ```

- [ ] **Step 2: Remove references**

  Strip every `observe_tier_backedge_event(...)` call and any helpers that no longer have callers. Where the call sits inside a semantic body in `semantics/control_flow.rs` (added in Task A10), remove it.

- [ ] **Step 3: Delete `tiering.rs`**

  ```sh
  rm crates/vm/src/vm/tiering.rs
  ```

- [ ] **Step 4: Verify**

  ```sh
  cargo build -p lyng-vm
  cargo test -p lyng-vm
  ```

- [ ] **Step 5: Commit**

  ```sh
  git rm crates/vm/src/vm/tiering.rs
  git add -u
  git commit -m "DSL-0c: delete tier-accounting machinery on backedges"
  ```

---

### Task C7: Run all behavioral tests after deletion

**Files:** none modified; produces evidence.

- [ ] **Step 1: Focused suites**

  ```sh
  cargo test -p lyng-vm -p lyng-bytecode -p lyng-objects -p lyng-tests -p lyng-compiler 2>&1 | tail -40
  ```
  Expected: same pass count as Pre-flight 4.

- [ ] **Step 2: Clippy**

  ```sh
  cargo clippy --all-targets --all-features -- -W clippy::pedantic -W clippy::nursery 2>&1 | tail -50
  ```
  Expected: no new warnings. Fix or document any new ones.

- [ ] **Step 3: Whole-corpus Test262**

  ```sh
  cargo run --release -p lyng-test262 -- --report /tmp/dsl-0c-test262.md -j 4
  cp /tmp/dsl-0c-test262.md reports/lyng/dsl-0c-test262.md
  ```
  Expected: pass count ≥ Pre-flight 7 baseline.

- [ ] **Step 4: Commit evidence**

  ```sh
  git add reports/lyng/dsl-0c-test262.md
  git commit -m "DSL-0c: post-deletion behavioral test evidence"
  ```

---

### Task C8: Run microbench + V8 v7 with DSL dispatch active

**Files:** none modified; produces evidence.

- [ ] **Step 1: V8 v7 run**

  ```sh
  cargo run --release -p lyng-bench -- v8suite --report /tmp/dsl-0c-v8.md --json /tmp/dsl-0c-v8.json
  cp /tmp/dsl-0c-v8.md reports/lyng/dsl-0c-v8.md
  cp /tmp/dsl-0c-v8.json reports/lyng/dsl-0c-v8.json
  ```

- [ ] **Step 2: Microbench (5 hot + 5 warm + a sample of cold opcodes)**

  ```sh
  cargo run --release -p lyng-bench -- microbench --opcodes op_move,op_add,op_jump,op_return,op_loop_header,op_jump8,op_jump_if_true,op_wide,op_extra_wide,op_get_named_property,op_load_global,op_call0 --samples 7 --iters 5000000 --report /tmp/dsl-0c-microbench.md --require-isolation
  cp /tmp/dsl-0c-microbench.md reports/lyng/dsl-0c-microbench.md
  ```

- [ ] **Step 3: Compare to gates**

  Open both reports and confirm:
  - **V8 v7 geomean ≥ +20% vs pre-DSL-0** (exit criterion 5).
  - **Richards ≥ +30% vs pre-DSL-0** (exit criterion 5).
  - **Each hot handler within 2× of LLInt-equivalent** (exit criterion 3).

  If any gate fails, document the failure in a follow-up note in `dsl-0c-microbench.md` and consider whether DSL-0 is aborted (per the criterion-1-through-6 abort clause).

- [ ] **Step 4: Commit evidence**

  ```sh
  git add reports/lyng/dsl-0c-v8.md \
          reports/lyng/dsl-0c-v8.json \
          reports/lyng/dsl-0c-microbench.md
  git commit -m "DSL-0c: microbench + V8 v7 evidence with DSL dispatch active"
  ```

---

### Task C9: Manifest Test 3 + 5 — `dsl_handler_symbol` linker resolution

**Files:**
- Modify: `crates/vm/src/dsl/opcode_manifest.rs`

Mirrors Task A19's `SEMANTIC_FN_PTRS` pattern but for DSL handlers.

- [ ] **Step 1: Add `DSL_HANDLER_FN_PTRS` parallel slice**

  ```rust
  pub static DSL_HANDLER_FN_PTRS: &[*const ()] = &[
      // One entry per opcode, populated by manifest-registration code in
      // hot.rs / warm.rs / cold.rs (or by codegen at build time).
      crate::dsl::handlers::hot::op_move as *const (),
      // ...
  ];
  ```

- [ ] **Step 2: Add the test**

  ```rust
  #[test]
  fn dsl_handler_fn_ptrs_resolve() {
      assert_eq!(DSL_HANDLER_FN_PTRS.len(), OPCODES.len());
      for (idx, ptr) in DSL_HANDLER_FN_PTRS.iter().enumerate() {
          assert!(!ptr.is_null(), "DSL_HANDLER_FN_PTRS[{idx}] is null (opcode {:?})", OPCODES[idx].opcode);
      }
  }
  ```

- [ ] **Step 3: Run**

  ```sh
  cargo test -p lyng-vm dsl::opcode_manifest::manifest_tests::dsl_handler_fn_ptrs_resolve
  ```

- [ ] **Step 4: Commit**

  ```sh
  git add crates/vm/src/dsl/opcode_manifest.rs
  git commit -m "DSL-0c: manifest Test 3/5 (DSL handler fn-ptr linker resolution)"
  ```

---

### Task C10: Manifest Test 6 — `dispatch_handlers` does not exist

**Files:**
- Modify: `crates/vm/tests/dsl_manifest_grep.rs`

- [ ] **Step 1: Add a complementary "module absent" assertion**

  Append to `dsl_manifest_grep.rs`:
  ```rust
  use std::path::Path;

  #[test]
  fn dispatch_handlers_module_does_not_exist() {
      let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/vm/dispatch_handlers"));
      assert!(!path.exists(), "vm/dispatch_handlers/ should be deleted after DSL-0c");
  }

  #[test]
  fn no_dispatch_next_step_dispatch_table_references_in_semantics() {
      let semantics = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/vm/semantics"));
      let mut files = Vec::new();
      // ... walk + grep for "dispatch_next!", "Step", "DISPATCH_TABLE" ...
      // (Implementation: walk_files(semantics), each file body must not contain those tokens.)
  }
  ```

- [ ] **Step 2: Run**

  ```sh
  cargo test -p lyng-vm --test dsl_manifest_grep
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/tests/dsl_manifest_grep.rs
  git commit -m "DSL-0c: manifest Test 6 (dispatch_handlers absent + no α references)"
  ```

---

### Task C11: Manifest Test 7 — opcode-counter mode preserves counts

**Files:**
- Create: `crates/vm/tests/dsl_opcode_counter_parity.rs`

- [ ] **Step 1: Author the test**

  Compares per-opcode dispatch counts produced under DSL dispatch with `--features diagnostic-counters` against a pre-recorded alpha baseline captured during Pre-flight or A20.

  ```rust
  #[test]
  fn dsl_opcode_counter_matches_alpha_within_tolerance() {
      // Run a fixed Richards iteration count under opcode-counter mode.
      // Compare per-opcode counts to a baseline checked into the test
      // fixtures (or re-derive from DSL-0a's opcode-count JSON).
      // Tolerance per design §10 DSL-0c Test 7:
      //   "per-opcode counts within a documented per-handler instrumentation delta".
      //
      // Specifically: 0% drift for opcodes that have no DSL-side advance
      // optimization; documented delta for op_return (potential same-code-
      // unit shortcut) and Star-fusion targets.
      // ...
  }
  ```

  Pre-record the alpha-baseline counts by running Richards under diagnostic-counters during Task A20 (refresh if needed; the alpha-baseline reference file goes in `reports/lyng/dsl-0a-opcode-counts-richards.json`).

- [ ] **Step 2: Run**

  ```sh
  cargo test -p lyng-vm --features diagnostic-counters --test dsl_opcode_counter_parity
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add crates/vm/tests/dsl_opcode_counter_parity.rs \
          reports/lyng/dsl-0a-opcode-counts-richards.json
  git commit -m "DSL-0c: manifest Test 7 (opcode-counter parity α ↔ DSL)"
  ```

---

### Task C12: Write DSL-0 decision document

**Files:**
- Create: `reports/lyng/dsl-0-decision.md`

- [ ] **Step 1: Author the decision doc**

  ```markdown
  # DSL-0 decision

  ## Verdict

  <COMMIT_TO_DSL_1 | ABORT_TO_GAMMA_HARD>

  ## Exit-criterion table

  | # | Criterion | Status | Evidence |
  | -: | --- | --- | --- |
  | 1 | Single-implementation invariant (manifest Tests 1–7) | ✓ | crates/vm/src/dsl/opcode_manifest.rs + tests/dsl_manifest_grep.rs |
  | 2 | Asm shape within 5 instructions of LLInt (per hot handler) | ✓/✗ | reports/lyng/dsl-handlers/op_*.md (5 hot ports) |
  | 3 | Microbench within 2× of LLInt-equivalent | ✓/✗ | reports/lyng/dsl-0c-microbench.md |
  | 4 | Behavioral parity | ✓ | reports/lyng/dsl-0c-test262.md |
  | 5 | V8 v7 geomean ≥ +20% vs pre-DSL-0 (Richards ≥ +30%) | ✓/✗ | reports/lyng/dsl-0c-v8.md |
  | 6 | All 9 DSL-0b validation cases still pass | ✓ | crates/vm/tests/dsl_validation_*.rs |
  | 7 | Per-opcode dispatch counter parity | ✓ | crates/vm/tests/dsl_opcode_counter_parity.rs |

  ## Observations

  - <Bullet list of notable findings.>

  ## Decision rationale

  <Free-form: why commit to DSL-1, or why abort to γ-hard.>

  ## Open questions to revisit in DSL-1

  - Per design §13 open questions (1–12); update with anything DSL-0
    surfaced.

  ## Hand-off

  If COMMIT_TO_DSL_1: next plan = TBD (writing-plans skill, separate
  document covering 25 more hot opcodes + IC mode-byte refactor).
  If ABORT_TO_GAMMA_HARD: preserve DSL-0a (semantics extraction +
  manifest) and revert DSL-0b/0c. The 8–10 weeks produced the
  evidence we needed.
  ```

  Fill the placeholders with real numbers and bullets.

- [ ] **Step 2: Commit**

  ```sh
  git add reports/lyng/dsl-0-decision.md
  git commit -m "DSL-0: decision document"
  ```

---

### Task C13: DSL-0 exit gate

**Files:** none modified; produces user-facing summary.

- [ ] **Step 1: Final sanity-check all seven exit criteria**

  Walk through `reports/lyng/dsl-0-decision.md` and confirm every row has evidence.

- [ ] **Step 2: Run every test target one more time**

  ```sh
  cargo test -p lyng-vm -p lyng-bytecode -p lyng-objects -p lyng-tests -p lyng-compiler
  cargo test -p lyng-vm --features diagnostic-counters
  cargo build --release -p lyng-vm -p lyng-bench
  cargo clippy --all-targets --all-features -- -W clippy::pedantic -W clippy::nursery 2>&1 | tail -30
  ```

- [ ] **Step 3: Update dcat**

  ```sh
  dcat update <DSL0C> --status in_review
  dcat update <DSL0_PARENT> --status in_review
  ```

- [ ] **Step 4: Notify the user**

  "DSL-0 complete. Decision: `<COMMIT_TO_DSL_1 | ABORT_TO_GAMMA_HARD>`. All seven exit criteria verified per `reports/lyng/dsl-0-decision.md`. The DSL-0 dcat parent + sub-epics are in_review; close gates on your approval. Next: <DSL-1 planning | revert and pivot>."

---

## Self-review checklist

After completing every task, verify:

- [ ] **Spec coverage:** every requirement in §10 of the design (R-0 exit criteria already met before DSL-0a; DSL-0a, DSL-0b, DSL-0c sub-phase exit criteria; the seven DSL-0 final exit criteria) is implemented or explicitly waived in `reports/lyng/dsl-0-decision.md`.

- [ ] **Manifest Tests 1–7 all pass:** running `cargo test -p lyng-vm dsl::opcode_manifest` and `cargo test -p lyng-vm --test dsl_manifest_grep` reports 0 failures.

- [ ] **All 9 DSL-0b validation cases pass:** `cargo test -p lyng-vm --test 'dsl_validation_*'`.

- [ ] **Per-handler ported reports exist** for the 5 hot + 5 warm DSL handlers in `reports/lyng/dsl-handlers/`.

- [ ] **Asm baselines committed** for every ported handler in `reports/lyng/dsl-asm-baseline-aarch64/`.

- [ ] **`reports/lyng/dsl-0a-status.md`, `dsl-0b-status.md`, `dsl-0-decision.md` all exist** with their placeholders filled.

- [ ] **No `unsafe` outside the policy-allowed modules** — confirm by re-running the policy lint test introduced in R-0.

- [ ] **`cargo build --release -p lyng-vm` is clean.**

- [ ] **Test262 pass count ≥ Pre-flight 7 baseline.**

- [ ] **V8 v7 geomean ≥ +20% vs pre-DSL-0 baseline (Richards ≥ +30%)** — or, if not, an explicit abort decision is recorded in `dsl-0-decision.md`.

- [ ] **dcat sub-epics `<DSL0A>`, `<DSL0B>`, `<DSL0C>` and parent `<DSL0_PARENT>` are all `in_review`** awaiting user approval. None closed.

If any of these fail, stop and resolve before declaring DSL-0 done.



