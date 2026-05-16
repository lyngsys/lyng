# R-0 Tooling and Evidence Reports Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the measurement infrastructure (three new `lyng-js-bench` subcommands, slow-path-share counter mode, opcode-share config) and three evidence reports (value-layout, ABI, safepoints) that gate the asm-DSL interpreter work specified in [docs/lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md](../../lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md).

**Architecture:** Extend the existing `tools/lyng-js-bench/` Rust crate with three new subcommand modules (`microbench`, `asm_diff`, `capture_llint`) plus a `hot_opcodes` config parser. Add a new `SlowPathCounterStore` to `crates/lyng-js/vm/` behind the existing `opcode-counters` Cargo feature. Write three multi-page Markdown evidence reports documenting the current Value layout, the `LlIntState`/`LlIntRustContext` ABI, and the safepoint/poll model. Update three policy docs (`crates/lyng-js/AGENTS.md`, `docs/lyng-js/engineering-standards.md`, `docs/lyng-js/architecture.md`) to permit DSL-scoped unsafe. Capture initial baselines under `reports/js/lyng-js/`.

**Tech Stack:** Rust stable (rustc ≥ 1.88 needed for DSL phase; R-0 itself uses any current stable), `serde_json` (already in `lyng-js-bench` deps), `toml` (new dep for `hot-opcodes.toml`), system tools (`otool` on macOS for LLInt capture), `cargo asm` (optional convenience; fallback to `cargo rustc --emit=asm`).

---

## Pre-flight check

Before starting, verify the prerequisites are met. Run from repo root.

- [ ] **Pre-flight 1: Confirm worktree is on the right branch**

  Run:
  ```sh
  git status -sb
  ```
  Expected: branch shows `claude/epic-saha-8f0b96` or a fresh branch off it; working tree clean.

- [ ] **Pre-flight 2: Confirm spec is committed**

  Run:
  ```sh
  ls -la docs/lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md
  ```
  Expected: file exists. If absent, abort — the plan presumes the spec is the source of truth.

- [ ] **Pre-flight 3: Confirm Rust toolchain**

  Run:
  ```sh
  rustc --version
  cargo --version
  ```
  Expected: rustc 1.88 or later (needed downstream for `naked_asm!`; the plan itself runs on any current stable but you'll want to verify the toolchain matches what DSL-0 will require).

- [ ] **Pre-flight 4: Confirm bench tool builds clean**

  Run:
  ```sh
  cargo build --release -p lyng-js-bench
  ```
  Expected: clean build, no warnings.

- [ ] **Pre-flight 5: Confirm test suites pass at baseline**

  Run:
  ```sh
  cargo test -p lyng-js-vm -p lyng-js-bytecode -p lyng-js-objects -p lyng-js-tests -p lyng-js-compiler
  ```
  Expected: all tests pass. Note the count; you'll compare at the end.

---

## Phase 1 — Setup and policy updates (Tasks 1-3)

### Task 1: Create R-0 sub-tickets in dcat

**Files:** none in repo; uses dcat issue tracker.

- [ ] **Step 1: Read dcat workflow**

  Run:
  ```sh
  dcat prime --opinionated
  ```
  Expected: prints dcat workflow guide.

- [ ] **Step 2: Look up the parent epic**

  Run:
  ```sh
  dcat show lyng-49qk
  ```
  Expected: shows the JSC-aligned engine roadmap epic.

- [ ] **Step 3: Create R-0 parent ticket**

  Run:
  ```sh
  dcat create "R-0: asm-DSL tooling and evidence reports" --type epic --priority 1 \
    --parent lyng-49qk \
    --labels js,performance,tooling,vm,roadmap \
    -d "Tooling and evidence reports specified in §10 of docs/lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md. See plan at docs/superpowers/plans/2026-05-16-r0-tooling-and-evidence.md."
  ```
  Record the returned ID; this is the parent for the next sub-tickets.

- [ ] **Step 4: Create sub-tickets for major workstreams**

  Run six commands (substitute `<R0_PARENT_ID>` from Step 3):
  ```sh
  dcat create "Update unsafe policy docs for DSL boundary" --type task --priority 1 --parent <R0_PARENT_ID> --labels js,docs
  dcat create "Add hot-opcodes.toml from measured V8 v7 dispatch counts" --type task --priority 1 --parent <R0_PARENT_ID> --labels js,tooling,bench
  dcat create "lyng-js-bench asm-diff subcommand" --type task --priority 1 --parent <R0_PARENT_ID> --labels js,tooling,bench,perf
  dcat create "lyng-js-bench microbench subcommand" --type task --priority 1 --parent <R0_PARENT_ID> --labels js,tooling,bench,perf
  dcat create "lyng-js-bench capture-llint subcommand" --type task --priority 1 --parent <R0_PARENT_ID> --labels js,tooling,bench
  dcat create "Slow-path-share counter mode" --type task --priority 1 --parent <R0_PARENT_ID> --labels js,vm,counters
  dcat create "Evidence report: value layout, ABI, safepoints" --type task --priority 1 --parent <R0_PARENT_ID> --labels js,docs,vm
  dcat create "R-0 baselines and determinism verification" --type task --priority 1 --parent <R0_PARENT_ID> --labels js,bench,verification
  ```

- [ ] **Step 5: Verify tickets created**

  Run:
  ```sh
  dcat list <R0_PARENT_ID>
  ```
  Expected: 8 child tickets listed.

  No commit needed — dcat tracks its own state.

---

### Task 2: Update `crates/lyng-js/AGENTS.md` to scope-allow DSL unsafe

**Files:**
- Modify: `crates/lyng-js/AGENTS.md:142`

- [ ] **Step 1: Mark the ticket in progress**

  Run:
  ```sh
  dcat update <UNSAFE_POLICY_TICKET_ID> --status in_progress
  ```

- [ ] **Step 2: Read current policy line**

  Run:
  ```sh
  sed -n '139,148p' crates/lyng-js/AGENTS.md
  ```
  Expected: shows the "Memory And Safety" section including `- Do not use unsafe code in Lyng JS crates.`

- [ ] **Step 3: Replace the blanket prohibition with the scoped allowance**

  Use Edit on `crates/lyng-js/AGENTS.md` to replace:
  ```
  - Do not use `unsafe` code in Lyng JS crates.
  ```
  with:
  ```
  - `unsafe` code is permitted only in the DSL substrate modules listed below, and
    only behind macro-generated code with audited invariants:
    - `crates/lyng-js-vm-dsl/` (proc-macro crate; not yet created)
    - `crates/lyng-js/vm/src/dsl/` (DSL backend, entry/exit shims, slow-path bridge)
    - Existing narrow unsafe blocks in `crates/lyng-js/vm/src/vm/dispatch_state.rs` and
      `crates/lyng-js/types/src/value.rs` (bounds-check elision, NaN-box construction).
    Hand-written `unsafe` outside these locations is forbidden. Every `unsafe` block
    must carry a `// SAFETY:` comment naming the invariant the caller must uphold.
  ```

- [ ] **Step 4: Verify the file still parses as Markdown**

  Run:
  ```sh
  head -160 crates/lyng-js/AGENTS.md | tail -30
  ```
  Expected: section reads cleanly; bullet list structure intact.

- [ ] **Step 5: Commit**

  Run:
  ```sh
  git add crates/lyng-js/AGENTS.md
  git commit -m "$(cat <<'EOF'
  R-0: scope-allow unsafe in DSL substrate modules

  Replace the blanket no-unsafe rule in crates/lyng-js/AGENTS.md with a
  scoped allowance for the asm-DSL substrate modules. Per §12 of
  docs/lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md.

  Co-Authored-By: <your-name>
  EOF
  )"
  ```

- [ ] **Step 6: Mark ticket in review**

  Run:
  ```sh
  dcat update <UNSAFE_POLICY_TICKET_ID> --status in_review
  ```

---

### Task 3: Update `docs/lyng-js/engineering-standards.md` Safety Rules section

**Files:**
- Modify: `docs/lyng-js/engineering-standards.md:62-69`

- [ ] **Step 1: Read current Safety Rules**

  Run:
  ```sh
  sed -n '62,69p' docs/lyng-js/engineering-standards.md
  ```

- [ ] **Step 2: Add DSL-boundary text**

  Edit `docs/lyng-js/engineering-standards.md` to append after the existing Safety Rules bullets (before `## Testing Rules`):
  ```
  - DSL boundary: the asm-DSL substrate (`crates/lyng-js-vm-dsl/` and
    `crates/lyng-js/vm/src/dsl/`) is the audited home for inline assembly,
    `#[unsafe(naked)]` functions, and the slow-path bridge. Changes to those
    modules require: a `// SAFETY:` invariant comment per unsafe block, an asm
    snapshot diff via `lyng-js-bench asm-diff` (when the DSL handler set is
    populated), and Miri coverage for the shim layer.
  ```

- [ ] **Step 3: Commit**

  Run:
  ```sh
  git add docs/lyng-js/engineering-standards.md
  git commit -m "R-0: document DSL substrate audit expectations in engineering-standards"
  ```

---

## Phase 2 — Hot opcodes measurement (Tasks 4-6)

### Task 4: Measure top-30 opcodes by dispatch count on V8 v7

**Files:** none modified; gathers data.

- [ ] **Step 1: Mark ticket in progress**

  ```sh
  dcat update <HOT_OPCODES_TICKET_ID> --status in_progress
  ```

- [ ] **Step 2: Build the counter-enabled bench binary**

  Run:
  ```sh
  cargo build --release -p lyng-js-bench --features lyng-js-vm/opcode-counters
  ```
  Expected: clean build.

- [ ] **Step 3: Run V8 v7 with opcode counting**

  Run:
  ```sh
  cargo run --release -p lyng-js-bench -- v8suite --count-opcodes \
    --json /tmp/r0-opcode-counts.json --samples 3
  ```
  Expected: JSON file emitted with per-workload per-opcode dispatch counts.

- [ ] **Step 4: Verify the JSON has the expected shape**

  Run:
  ```sh
  jq 'keys' /tmp/r0-opcode-counts.json | head -20
  ```
  Expected: workloads (`Richards`, `DeltaBlue`, etc.) appear as top-level keys.

- [ ] **Step 5: Compute top-30 across the suite**

  Write a one-off shell script `/tmp/r0-top30.sh`:
  ```sh
  #!/usr/bin/env bash
  set -euo pipefail
  jq -r '
    [paths(numbers) as $p | {opcode: ($p[-1]|tostring), count: getpath($p)}]
    | group_by(.opcode)
    | map({opcode: .[0].opcode, total: (map(.count) | add)})
    | sort_by(-.total)
    | .[0:30]
    | .[] | "\(.opcode)\t\(.total)"
  ' /tmp/r0-opcode-counts.json
  ```
  Make executable and run:
  ```sh
  chmod +x /tmp/r0-top30.sh
  /tmp/r0-top30.sh > /tmp/r0-top30.tsv
  cat /tmp/r0-top30.tsv
  ```
  Expected: 30 lines, opcode name + dispatch count, descending. Record this list for Task 5.

  Note: if the JSON nesting differs from the assumed shape, adjust the `jq` filter. The goal is `<opcode>\t<total>` for the 30 most-dispatched opcodes across the V8 v7 suite.

- [ ] **Step 6: Save raw data for the report**

  Copy the JSON and TSV into the reports tree:
  ```sh
  mkdir -p reports/js/lyng-js/r0/
  cp /tmp/r0-opcode-counts.json reports/js/lyng-js/r0/v8-v7-opcode-counts.json
  cp /tmp/r0-top30.tsv reports/js/lyng-js/r0/v8-v7-top30.tsv
  ```

- [ ] **Step 7: Commit raw data**

  Run:
  ```sh
  git add reports/js/lyng-js/r0/v8-v7-opcode-counts.json reports/js/lyng-js/r0/v8-v7-top30.tsv
  git commit -m "R-0: measured V8 v7 opcode dispatch counts (raw data)"
  ```

---

### Task 5: Write `tools/lyng-js-bench/hot-opcodes.toml`

**Files:**
- Create: `tools/lyng-js-bench/hot-opcodes.toml`

- [ ] **Step 1: Draft the TOML using Task 4 data**

  Write `tools/lyng-js-bench/hot-opcodes.toml`:
  ```toml
  # Hot-opcode configuration for the asm-DSL substrate.
  #
  # The `hot` list is the top-30 opcodes by measured dispatch share on the
  # V8 v7 benchmark suite (see reports/js/lyng-js/r0/v8-v7-top30.tsv for
  # the source data; refresh when the compiler emits a major new pattern).
  #
  # `target_slow_path_share` is the DSL-1 invariant: each hot opcode must
  # measure < this fraction of slow-path-semantic share on V8 v7 workloads.
  # Default 0.20; per-opcode overrides are allowed with justification.
  #
  # `aarch64_max_instructions` and `x86_64_max_instructions` are the
  # asm-diff per-handler budgets (instructions, not bytes) used by
  # `lyng-js-bench asm-diff`. Calibrated in DSL-0b from real measurements.
  # Placeholder values here (0 = no enforcement); update during DSL-0b.

  [meta]
  source_data = "reports/js/lyng-js/r0/v8-v7-top30.tsv"
  refresh_command = "cargo run --release -p lyng-js-bench -- v8suite --count-opcodes --json /tmp/r0-opcode-counts.json"
  default_target_slow_path_share = 0.20

  [[opcodes]]
  name = "Move"
  target_slow_path_share = 0.20
  aarch64_max_instructions = 0  # set in DSL-0b
  x86_64_max_instructions = 0   # set in DSL-2

  # ... (repeat for each of the 30 opcodes from /tmp/r0-top30.tsv)
  ```
  Fill in one `[[opcodes]]` block per opcode from the top-30 list. Use the opcode names as they appear in `lyng_js_bytecode::Opcode` (PascalCase).

- [ ] **Step 2: Add a `toml` dev/build dep to the bench tool**

  Edit `tools/lyng-js-bench/Cargo.toml`. After `serde_json = "1"`, add:
  ```toml
  toml = "0.8"
  serde = { version = "1", features = ["derive"] }
  ```

- [ ] **Step 3: Build to verify deps resolve**

  Run:
  ```sh
  cargo build --release -p lyng-js-bench
  ```
  Expected: clean build with new deps.

- [ ] **Step 4: Commit**

  Run:
  ```sh
  git add tools/lyng-js-bench/hot-opcodes.toml tools/lyng-js-bench/Cargo.toml Cargo.lock
  git commit -m "R-0: hot-opcodes.toml with measured top-30 opcodes from V8 v7"
  ```

---

### Task 6: Add `hot_opcodes` config-parser module

**Files:**
- Create: `tools/lyng-js-bench/src/hot_opcodes.rs`
- Modify: `tools/lyng-js-bench/src/lib.rs`

- [ ] **Step 1: Write failing test for config parsing**

  Create `tools/lyng-js-bench/src/hot_opcodes.rs`:
  ```rust
  //! Parser for `tools/lyng-js-bench/hot-opcodes.toml`.
  //!
  //! Consumed by `asm-diff`, `microbench`, and `--count-slow-path-share`.
  //! The config is the single source of truth for which opcodes count
  //! as "hot" and what their per-opcode invariant thresholds are.

  use serde::Deserialize;
  use std::path::Path;

  #[derive(Debug, Deserialize, Clone, PartialEq)]
  pub struct HotOpcodesConfig {
      pub meta: Meta,
      pub opcodes: Vec<OpcodeEntry>,
  }

  #[derive(Debug, Deserialize, Clone, PartialEq)]
  pub struct Meta {
      pub source_data: String,
      pub refresh_command: String,
      pub default_target_slow_path_share: f64,
  }

  #[derive(Debug, Deserialize, Clone, PartialEq)]
  pub struct OpcodeEntry {
      pub name: String,
      #[serde(default)]
      pub target_slow_path_share: Option<f64>,
      #[serde(default)]
      pub aarch64_max_instructions: Option<u32>,
      #[serde(default)]
      pub x86_64_max_instructions: Option<u32>,
  }

  impl HotOpcodesConfig {
      /// Load from a file path.
      ///
      /// # Errors
      ///
      /// Returns an error if the file cannot be read or the TOML cannot be parsed.
      pub fn load(path: &Path) -> Result<Self, String> {
          let raw = std::fs::read_to_string(path)
              .map_err(|err| format!("read {}: {err}", path.display()))?;
          toml::from_str::<Self>(&raw)
              .map_err(|err| format!("parse {}: {err}", path.display()))
      }

      /// Effective slow-path share threshold for an opcode (override or default).
      #[must_use]
      pub fn target_slow_path_share(&self, opcode_name: &str) -> f64 {
          self.opcodes
              .iter()
              .find(|entry| entry.name == opcode_name)
              .and_then(|entry| entry.target_slow_path_share)
              .unwrap_or(self.meta.default_target_slow_path_share)
      }

      /// Hot-opcode name list, in config order.
      #[must_use]
      pub fn hot_opcode_names(&self) -> Vec<&str> {
          self.opcodes.iter().map(|entry| entry.name.as_str()).collect()
      }
  }

  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn parses_minimal_config() {
          let raw = r#"
              [meta]
              source_data = "x.tsv"
              refresh_command = "cmd"
              default_target_slow_path_share = 0.20

              [[opcodes]]
              name = "Move"
          "#;
          let config: HotOpcodesConfig = toml::from_str(raw).unwrap();
          assert_eq!(config.opcodes.len(), 1);
          assert_eq!(config.opcodes[0].name, "Move");
          assert!((config.target_slow_path_share("Move") - 0.20).abs() < 1e-9);
      }

      #[test]
      fn per_opcode_threshold_override_takes_precedence() {
          let raw = r#"
              [meta]
              source_data = "x"
              refresh_command = "y"
              default_target_slow_path_share = 0.20

              [[opcodes]]
              name = "GetNamedProperty"
              target_slow_path_share = 0.35
          "#;
          let config: HotOpcodesConfig = toml::from_str(raw).unwrap();
          assert!((config.target_slow_path_share("GetNamedProperty") - 0.35).abs() < 1e-9);
      }

      #[test]
      fn missing_opcode_falls_back_to_default() {
          let raw = r#"
              [meta]
              source_data = "x"
              refresh_command = "y"
              default_target_slow_path_share = 0.20
          "#;
          let config: HotOpcodesConfig = toml::from_str(raw).unwrap();
          assert!((config.target_slow_path_share("MissingOpcode") - 0.20).abs() < 1e-9);
      }
  }
  ```

- [ ] **Step 2: Wire the new module into lib.rs**

  Edit `tools/lyng-js-bench/src/lib.rs` to add `pub mod hot_opcodes;` after the existing `pub mod cli;` line.

- [ ] **Step 3: Run the new tests**

  Run:
  ```sh
  cargo test -p lyng-js-bench hot_opcodes::tests
  ```
  Expected: all 3 tests pass.

- [ ] **Step 4: Verify it parses the real committed config**

  Add a smoke-test at the bottom of `hot_opcodes.rs`:
  ```rust
  #[test]
  fn parses_the_committed_hot_opcodes_toml() {
      let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
          .join("hot-opcodes.toml");
      let config = HotOpcodesConfig::load(&path).expect("load");
      assert!(config.opcodes.len() >= 25, "expected at least 25 hot opcodes, got {}", config.opcodes.len());
      assert!(config.opcodes.len() <= 35, "expected at most 35 hot opcodes, got {}", config.opcodes.len());
  }
  ```

- [ ] **Step 5: Run smoke test**

  ```sh
  cargo test -p lyng-js-bench hot_opcodes::tests::parses_the_committed
  ```
  Expected: pass.

- [ ] **Step 6: Commit**

  ```sh
  git add tools/lyng-js-bench/src/hot_opcodes.rs tools/lyng-js-bench/src/lib.rs
  git commit -m "R-0: hot_opcodes config parser module with TOML round-trip tests"
  ```

- [ ] **Step 7: Mark ticket in review**

  ```sh
  dcat update <HOT_OPCODES_TICKET_ID> --status in_review
  ```

---

## Phase 3 — asm-diff subcommand (Tasks 7-12)

### Task 7: Write the asm normalization rule set

**Files:**
- Create: `reports/js/lyng-js/dsl-asm-baseline-aarch64/NORMALIZATION.md`

- [ ] **Step 1: Create the directory**

  ```sh
  mkdir -p reports/js/lyng-js/dsl-asm-baseline-aarch64
  ```

- [ ] **Step 2: Write the normalization spec**

  Create `reports/js/lyng-js/dsl-asm-baseline-aarch64/NORMALIZATION.md`:
  ```markdown
  # asm-diff normalization rules

  This document is the single source of truth for the normalization
  rules applied by `lyng-js-bench asm-diff` before comparing handler
  asm to committed baselines. Changes to these rules require a separate,
  explicitly-reasoned commit.

  ## Inputs

  - Raw output of `cargo asm --release -p <crate> <symbol>` OR
    `cargo rustc --release -p <crate> -- --emit=asm` for the matching `.s` file.
  - Target: `aarch64-apple-darwin` (initial), `x86_64-*` (future).

  ## Rules

  Applied in order:

  1. **Strip CFI directives.** Any line matching `^\s*\.cfi_` (with optional leading whitespace) is dropped.
  2. **Strip section/alignment metadata.** Lines starting with `.section`, `.p2align`, `.globl`, `.private_extern`, or `.subsections_via_symbols` are dropped.
  3. **Strip debug source comments.** Lines matching `^\s*#\s*/` (file-path comments emitted by `--emit=asm`) are dropped.
  4. **Strip literal-pool comments.** Inline `;` comments that begin with whitespace followed by `=` (literal-value annotations) are dropped.
  5. **Rename labels positionally.** Symbols matching `^L[A-Za-z_]*[0-9]+$` (compiler-generated labels) are renamed to `L0`, `L1`, ... in order of first appearance. Branches referring to renamed labels are rewritten with the same alias.
  6. **Strip blank lines.**
  7. **Trim trailing whitespace** on each remaining line.

  ## What is preserved

  - Instruction mnemonic and operands.
  - Branch direction (forward/backward labels).
  - Label structure (relative ordering).
  - Function-entry markers (`<name>:` at column 0).

  ## What is intentionally NOT normalized

  - Register names (a per-arch baseline is per-arch).
  - Immediate values (constants are part of the asm semantics).
  - Instruction selection (we WANT to detect a `ldp` → `ldr` regression).

  ## Stability

  Two builds of the same handler with the same rustc version MUST
  produce byte-identical normalized output. If they don't, the rules
  above are incomplete — file a bug.

  Cross-rustc-version stability is NOT guaranteed. When upgrading
  rustc, run `lyng-js-bench asm-diff --mode update` to refresh
  baselines; commit message must include `[asm-baseline-refresh: rustc <old>→<new>]`.
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add reports/js/lyng-js/dsl-asm-baseline-aarch64/NORMALIZATION.md
  git commit -m "R-0: asm-diff normalization rule set"
  ```

---

### Task 8: Add `asm-diff` subcommand skeleton + cli wiring

**Files:**
- Create: `tools/lyng-js-bench/src/asm_diff.rs`
- Modify: `tools/lyng-js-bench/src/cli.rs`
- Modify: `tools/lyng-js-bench/src/lib.rs`

- [ ] **Step 1: Mark ticket in progress**

  ```sh
  dcat update <ASM_DIFF_TICKET_ID> --status in_progress
  ```

- [ ] **Step 2: Create the skeleton module**

  Create `tools/lyng-js-bench/src/asm_diff.rs`:
  ```rust
  //! `lyng-js-bench asm-diff` — capture, normalize, and diff handler asm
  //! against committed per-arch baselines.

  use std::path::PathBuf;

  #[derive(Debug, Clone, PartialEq)]
  pub struct AsmDiffOptions {
      pub opcodes_config: PathBuf,
      pub baseline_dir: PathBuf,
      pub output_dir: PathBuf,
      pub mode: Mode,
      pub capture_mode: CaptureMode,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum Mode {
      Check,
      Update,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum CaptureMode {
      Auto,        // try cargo-asm first, fall back to rustc-emit-asm
      CargoAsm,    // force cargo-asm
      RustcEmit,   // force cargo rustc -- --emit=asm
  }

  /// Run the asm-diff subcommand.
  ///
  /// # Errors
  ///
  /// Returns Err with a user-facing message on parse failure, capture
  /// failure, or — in `Check` mode — any per-opcode regression.
  pub fn run(args: &[String]) -> Result<(), String> {
      let options = parse_args(args)?;
      // Placeholder: real implementation lands in later tasks.
      // For now, just print what we would do.
      println!("asm-diff: opcodes_config={}", options.opcodes_config.display());
      println!("asm-diff: baseline_dir={}", options.baseline_dir.display());
      println!("asm-diff: output_dir={}", options.output_dir.display());
      println!("asm-diff: mode={:?}", options.mode);
      println!("asm-diff: capture_mode={:?}", options.capture_mode);
      Err("asm-diff: not yet implemented (R-0 Task 9+)".into())
  }

  fn parse_args(args: &[String]) -> Result<AsmDiffOptions, String> {
      let mut opcodes_config = PathBuf::from("tools/lyng-js-bench/hot-opcodes.toml");
      let mut baseline_dir = PathBuf::from("reports/js/lyng-js/dsl-asm-baseline-aarch64");
      let mut output_dir = PathBuf::from("/tmp/asm-current");
      let mut mode = Mode::Check;
      let mut capture_mode = CaptureMode::Auto;

      let mut iter = args.iter().peekable();
      while let Some(arg) = iter.next() {
          match arg.as_str() {
              "--opcodes-config" => {
                  opcodes_config = iter
                      .next()
                      .ok_or("--opcodes-config requires a path")?
                      .into();
              }
              "--baseline" => {
                  baseline_dir = iter.next().ok_or("--baseline requires a path")?.into();
              }
              "--output" => {
                  output_dir = iter.next().ok_or("--output requires a path")?.into();
              }
              "--mode" => match iter.next().map(String::as_str) {
                  Some("check") => mode = Mode::Check,
                  Some("update") => mode = Mode::Update,
                  Some(other) => return Err(format!("--mode: unknown value {other}")),
                  None => return Err("--mode requires check|update".into()),
              },
              "--capture-mode" => match iter.next().map(String::as_str) {
                  Some("auto") => capture_mode = CaptureMode::Auto,
                  Some("cargo-asm") => capture_mode = CaptureMode::CargoAsm,
                  Some("rustc") => capture_mode = CaptureMode::RustcEmit,
                  Some(other) => return Err(format!("--capture-mode: unknown value {other}")),
                  None => return Err("--capture-mode requires auto|cargo-asm|rustc".into()),
              },
              "--help" | "-h" => {
                  return Err(help_text());
              }
              other => return Err(format!("asm-diff: unknown argument {other}\n\n{}", help_text())),
          }
      }

      Ok(AsmDiffOptions {
          opcodes_config,
          baseline_dir,
          output_dir,
          mode,
          capture_mode,
      })
  }

  fn help_text() -> String {
      [
          "Usage: lyng-js-bench asm-diff [options]",
          "",
          "Options:",
          "  --opcodes-config PATH   Path to hot-opcodes.toml (default: tools/lyng-js-bench/hot-opcodes.toml)",
          "  --baseline DIR          Directory containing committed baselines",
          "  --output DIR            Directory for current-build asm capture",
          "  --mode check|update     check: fail on diff; update: overwrite baselines (default: check)",
          "  --capture-mode auto|cargo-asm|rustc  Capture backend (default: auto)",
      ]
      .join("\n")
  }

  #[cfg(test)]
  mod tests {
      use super::*;

      fn args(parts: &[&str]) -> Vec<String> {
          parts.iter().map(|p| (*p).to_string()).collect()
      }

      #[test]
      fn parses_defaults() {
          let opts = parse_args(&args(&[])).unwrap();
          assert_eq!(opts.mode, Mode::Check);
          assert_eq!(opts.capture_mode, CaptureMode::Auto);
      }

      #[test]
      fn parses_mode_update() {
          let opts = parse_args(&args(&["--mode", "update"])).unwrap();
          assert_eq!(opts.mode, Mode::Update);
      }

      #[test]
      fn rejects_unknown_mode() {
          let err = parse_args(&args(&["--mode", "bogus"])).unwrap_err();
          assert!(err.contains("unknown value"));
      }
  }
  ```

- [ ] **Step 3: Wire into CLI**

  Edit `tools/lyng-js-bench/src/cli.rs`:

  Add to the `Command` enum:
  ```rust
  AsmDiff(Vec<String>),
  ```

  Add to `parse_command`:
  ```rust
  Some("asm-diff") => Ok(Command::AsmDiff(args[2..].to_vec())),
  ```

  Update `help_text` to include `asm-diff`.

- [ ] **Step 4: Wire into lib.rs**

  Edit `tools/lyng-js-bench/src/lib.rs`:
  ```rust
  pub mod asm_diff;
  ```

  In `run`:
  ```rust
  cli::Command::AsmDiff(command_args) => asm_diff::run(&command_args),
  ```

- [ ] **Step 5: Run tests**

  ```sh
  cargo test -p lyng-js-bench
  ```
  Expected: all tests pass; new asm_diff tests appear.

- [ ] **Step 6: Smoke-test the CLI wiring**

  ```sh
  cargo run --release -p lyng-js-bench -- asm-diff --help 2>&1 | head -10
  ```
  Expected: help text appears (returned as Err but printed).

- [ ] **Step 7: Commit**

  ```sh
  git add tools/lyng-js-bench/src/asm_diff.rs tools/lyng-js-bench/src/cli.rs tools/lyng-js-bench/src/lib.rs
  git commit -m "R-0: asm-diff subcommand skeleton with CLI wiring"
  ```

---

### Task 9: Implement `cargo asm` capture + `rustc --emit=asm` fallback

**Files:**
- Modify: `tools/lyng-js-bench/src/asm_diff.rs`

- [ ] **Step 1: Write failing test for capture**

  Add to `asm_diff.rs` tests module:
  ```rust
  #[test]
  fn capture_via_rustc_returns_asm_for_existing_symbol() {
      // Pick a symbol that definitely exists in lyng-js-vm.
      let result = capture_symbol(
          "lyng-js-vm",
          "lyng_js_vm::Vm::new",
          CaptureMode::RustcEmit,
      );
      let asm = result.expect("capture should succeed");
      assert!(asm.contains("lyng_js_vm::Vm::new"));
  }
  ```

- [ ] **Step 2: Run test to verify it fails**

  ```sh
  cargo test -p lyng-js-bench capture_via_rustc
  ```
  Expected: FAIL (function not yet defined).

- [ ] **Step 3: Implement `capture_symbol`**

  Add to `asm_diff.rs`:
  ```rust
  use std::process::Command;

  /// Capture the asm for a single symbol via the requested backend.
  ///
  /// Returns the raw, unnormalized asm text on success.
  ///
  /// # Errors
  ///
  /// Returns Err if the capture tool fails or produces no output.
  pub fn capture_symbol(
      crate_name: &str,
      symbol: &str,
      mode: CaptureMode,
  ) -> Result<String, String> {
      match mode {
          CaptureMode::CargoAsm => capture_via_cargo_asm(crate_name, symbol),
          CaptureMode::RustcEmit => capture_via_rustc_emit(crate_name, symbol),
          CaptureMode::Auto => capture_via_cargo_asm(crate_name, symbol)
              .or_else(|_| capture_via_rustc_emit(crate_name, symbol)),
      }
  }

  fn capture_via_cargo_asm(crate_name: &str, symbol: &str) -> Result<String, String> {
      let output = Command::new("cargo")
          .args(["asm", "--release", "-p", crate_name, symbol])
          .output()
          .map_err(|err| format!("cargo-asm not available: {err}"))?;
      if !output.status.success() {
          return Err(format!(
              "cargo asm exited {}: {}",
              output.status,
              String::from_utf8_lossy(&output.stderr)
          ));
      }
      let text = String::from_utf8_lossy(&output.stdout).into_owned();
      if text.trim().is_empty() {
          return Err(format!("cargo asm produced empty output for {symbol}"));
      }
      Ok(text)
  }

  fn capture_via_rustc_emit(crate_name: &str, symbol: &str) -> Result<String, String> {
      // Use the cargo target dir's .s files.
      let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());

      // 1. Build with --emit=asm
      let build = Command::new("cargo")
          .args([
              "rustc",
              "--release",
              "-p",
              crate_name,
              "--",
              "--emit=asm",
              "-C",
              "debuginfo=0",
          ])
          .output()
          .map_err(|err| format!("cargo rustc failed: {err}"))?;
      if !build.status.success() {
          return Err(format!(
              "cargo rustc exited {}: {}",
              build.status,
              String::from_utf8_lossy(&build.stderr)
          ));
      }

      // 2. Find the .s file for the crate.
      let deps_dir = std::path::Path::new(&target_dir)
          .join("release")
          .join("deps");
      let crate_stem = crate_name.replace('-', "_");
      let s_file = std::fs::read_dir(&deps_dir)
          .map_err(|err| format!("read deps dir {}: {err}", deps_dir.display()))?
          .filter_map(Result::ok)
          .find_map(|entry| {
              let name = entry.file_name().to_string_lossy().into_owned();
              if name.starts_with(&crate_stem) && name.ends_with(".s") {
                  Some(entry.path())
              } else {
                  None
              }
          })
          .ok_or_else(|| format!(".s file for {crate_name} not found in {}", deps_dir.display()))?;

      // 3. Extract the symbol's body.
      let text = std::fs::read_to_string(&s_file)
          .map_err(|err| format!("read {}: {err}", s_file.display()))?;
      extract_symbol_body(&text, symbol)
  }

  fn extract_symbol_body(asm: &str, symbol: &str) -> Result<String, String> {
      // Mach-O symbol names get a leading underscore in some compilers.
      let candidates = [symbol.to_string(), format!("_{symbol}")];
      let mut iter = asm.lines();
      let mut found = false;
      let mut body = Vec::new();
      let label_pattern: Vec<String> = candidates.iter().map(|c| format!("{c}:")).collect();
      while let Some(line) = iter.next() {
          if !found {
              if label_pattern.iter().any(|p| line.contains(p)) {
                  found = true;
                  body.push(line.to_string());
              }
          } else {
              // Stop at the next top-level symbol or end of file.
              if line.starts_with(|c: char| c.is_ascii_alphabetic())
                  && line.ends_with(':')
                  && !line.starts_with('L')
                  && !line.starts_with('.')
              {
                  break;
              }
              body.push(line.to_string());
          }
      }
      if !found {
          return Err(format!("symbol {symbol} not found in asm"));
      }
      Ok(body.join("\n"))
  }
  ```

- [ ] **Step 4: Run the test**

  ```sh
  cargo test -p lyng-js-bench capture_via_rustc
  ```
  Expected: PASS. Note: this test runs a real `cargo rustc` build under the hood; it will take ~1 minute.

- [ ] **Step 5: Commit**

  ```sh
  git add tools/lyng-js-bench/src/asm_diff.rs
  git commit -m "R-0: asm-diff capture via cargo-asm with rustc --emit=asm fallback"
  ```

---

### Task 10: Implement asm normalization

**Files:**
- Modify: `tools/lyng-js-bench/src/asm_diff.rs`

- [ ] **Step 1: Write failing tests for normalization rules**

  Add to the `tests` module:
  ```rust
  #[test]
  fn normalize_strips_cfi_directives() {
      let raw = "foo:\n\t.cfi_startproc\n\tret\n\t.cfi_endproc\n";
      let normalized = normalize(raw);
      assert!(!normalized.contains(".cfi_"));
      assert!(normalized.contains("ret"));
  }

  #[test]
  fn normalize_strips_section_metadata() {
      let raw = ".section __TEXT,__text\n\t.p2align 2\nfoo:\n\tret\n";
      let normalized = normalize(raw);
      assert!(!normalized.contains(".section"));
      assert!(!normalized.contains(".p2align"));
      assert!(normalized.contains("foo:"));
  }

  #[test]
  fn normalize_renames_labels_positionally() {
      let raw = "foo:\n\tb LBB42_3\nLBB42_3:\n\tret\n";
      let normalized = normalize(raw);
      assert!(!normalized.contains("LBB42_3"));
      assert!(normalized.contains("L0"));
      // The branch and the label both reference the same alias.
      let l0_count = normalized.matches("L0").count();
      assert!(l0_count >= 2, "expected branch + label, got: {normalized}");
  }

  #[test]
  fn normalize_is_deterministic() {
      let raw = "foo:\n\t.cfi_startproc\n\tldr x0, [x1]\n\tret\n";
      let first = normalize(raw);
      let second = normalize(raw);
      assert_eq!(first, second);
  }
  ```

- [ ] **Step 2: Run tests to verify they fail**

  ```sh
  cargo test -p lyng-js-bench asm_diff::tests::normalize
  ```
  Expected: 4 FAIL (normalize not defined).

- [ ] **Step 3: Implement `normalize`**

  Add to `asm_diff.rs`:
  ```rust
  use std::collections::HashMap;

  /// Normalize raw asm output per the rules in
  /// `reports/js/lyng-js/dsl-asm-baseline-aarch64/NORMALIZATION.md`.
  #[must_use]
  pub fn normalize(raw: &str) -> String {
      let mut label_map: HashMap<String, String> = HashMap::new();
      let mut next_label_idx = 0_usize;
      let mut out: Vec<String> = Vec::new();

      for line in raw.lines() {
          let trimmed = line.trim_end();
          let stripped = trimmed.trim_start();

          // Rule 1-2: drop CFI / section / alignment / debug-source-comment lines.
          if stripped.is_empty()
              || stripped.starts_with(".cfi_")
              || stripped.starts_with(".section")
              || stripped.starts_with(".p2align")
              || stripped.starts_with(".globl")
              || stripped.starts_with(".private_extern")
              || stripped.starts_with(".subsections_via_symbols")
              || stripped.starts_with("# /")
          {
              continue;
          }

          // Rule 5: rename positional labels.
          let renamed = rename_labels(trimmed, &mut label_map, &mut next_label_idx);
          out.push(renamed);
      }

      out.join("\n") + "\n"
  }

  fn rename_labels(
      line: &str,
      map: &mut HashMap<String, String>,
      next_idx: &mut usize,
  ) -> String {
      // Pattern: `LBB<digits>_<digits>` or `L<word>_<digits>` (compiler-generated).
      // Replace with sequential L0, L1, ...
      let mut result = String::with_capacity(line.len());
      let mut rest = line;
      while let Some(idx) = find_compiler_label(rest) {
          let (prefix, label_start) = rest.split_at(idx);
          result.push_str(prefix);
          let label_len = label_token_length(label_start);
          let label = &label_start[..label_len];
          let alias = map.entry(label.to_string()).or_insert_with(|| {
              let alias = format!("L{}", *next_idx);
              *next_idx += 1;
              alias
          });
          result.push_str(alias);
          rest = &label_start[label_len..];
      }
      result.push_str(rest);
      result
  }

  fn find_compiler_label(s: &str) -> Option<usize> {
      // Look for "L" followed by a letter/underscore, then digits.
      // Matches LBB123_4, Lfunc_end42, etc. — but NOT plain "Lvalue" without digits.
      let bytes = s.as_bytes();
      let mut i = 0;
      while i < bytes.len() {
          if bytes[i] == b'L' && i + 1 < bytes.len() {
              let token = &s[i..];
              let len = label_token_length(token);
              if len > 1 {
                  // Must contain at least one digit to be considered compiler-generated.
                  let mid = &token[1..len];
                  if mid.bytes().any(|b| b.is_ascii_digit()) {
                      return Some(i);
                  }
              }
          }
          i += 1;
      }
      None
  }

  fn label_token_length(s: &str) -> usize {
      s.bytes()
          .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
          .count()
  }
  ```

- [ ] **Step 4: Run normalization tests**

  ```sh
  cargo test -p lyng-js-bench asm_diff::tests::normalize
  ```
  Expected: 4 PASS.

- [ ] **Step 5: Commit**

  ```sh
  git add tools/lyng-js-bench/src/asm_diff.rs
  git commit -m "R-0: asm-diff normalization rules per NORMALIZATION.md"
  ```

---

### Task 11: Implement diff + per-opcode budget enforcement + `--mode check/update`

**Files:**
- Modify: `tools/lyng-js-bench/src/asm_diff.rs`

- [ ] **Step 1: Write failing test for budget enforcement**

  Add to the tests module:
  ```rust
  #[test]
  fn check_mode_fails_when_baseline_missing() {
      let result = check_one_symbol(
          "fake_op",
          /* current */ "fake_op:\n\tret\n",
          /* baseline_dir */ &std::path::PathBuf::from("/nonexistent"),
          /* max_instructions */ Some(100),
      );
      assert!(result.is_err());
  }

  #[test]
  fn check_mode_succeeds_when_within_budget() {
      let tmp = tempdir::TempDir::new("asm").expect("tmp");
      let baseline_path = tmp.path().join("fake_op.asm");
      std::fs::write(&baseline_path, "fake_op:\n\tret\n").unwrap();
      let result = check_one_symbol(
          "fake_op",
          "fake_op:\n\tret\n",
          tmp.path(),
          Some(100),
      );
      assert!(result.is_ok(), "{:?}", result);
  }
  ```

  Add `tempdir = "0.3"` to `tools/lyng-js-bench/Cargo.toml` under `[dev-dependencies]` if absent.

- [ ] **Step 2: Run to verify failure**

  ```sh
  cargo test -p lyng-js-bench asm_diff::tests::check_mode
  ```
  Expected: FAIL (function not defined).

- [ ] **Step 3: Implement `check_one_symbol` and the diff plumbing**

  Add to `asm_diff.rs`:
  ```rust
  use std::path::Path;

  /// Per-symbol outcome from a `--mode check` pass.
  #[derive(Debug, PartialEq)]
  pub enum CheckOutcome {
      Match,
      Differs { diff: String, current_instr_count: usize, baseline_instr_count: usize },
  }

  /// Check one symbol against its baseline. Returns Ok(outcome) on success;
  /// Err(message) if the baseline file is missing or the current asm exceeds
  /// the instruction budget.
  ///
  /// # Errors
  ///
  /// - Baseline file does not exist.
  /// - Current asm's instruction count exceeds `max_instructions` budget.
  pub fn check_one_symbol(
      symbol: &str,
      current_asm: &str,
      baseline_dir: &Path,
      max_instructions: Option<u32>,
  ) -> Result<CheckOutcome, String> {
      let baseline_path = baseline_dir.join(format!("{symbol}.asm"));
      let baseline = std::fs::read_to_string(&baseline_path)
          .map_err(|err| format!("baseline missing for {symbol}: {} ({err})", baseline_path.display()))?;

      let normalized_current = normalize(current_asm);
      let normalized_baseline = normalize(&baseline);

      let current_instr_count = count_instructions(&normalized_current);
      if let Some(budget) = max_instructions {
          if budget > 0 && current_instr_count > budget as usize {
              return Err(format!(
                  "{symbol}: {current_instr_count} instructions exceeds budget of {budget}"
              ));
          }
      }

      if normalized_current == normalized_baseline {
          Ok(CheckOutcome::Match)
      } else {
          let baseline_instr_count = count_instructions(&normalized_baseline);
          Ok(CheckOutcome::Differs {
              diff: textual_diff(&normalized_baseline, &normalized_current),
              current_instr_count,
              baseline_instr_count,
          })
      }
  }

  fn count_instructions(normalized: &str) -> usize {
      normalized
          .lines()
          .filter(|line| {
              let trimmed = line.trim();
              // Instruction lines start with whitespace + mnemonic.
              // Skip labels (end with :) and directives (start with .).
              !trimmed.is_empty()
                  && !trimmed.ends_with(':')
                  && !trimmed.starts_with('.')
                  && !trimmed.starts_with('#')
          })
          .count()
  }

  fn textual_diff(baseline: &str, current: &str) -> String {
      // Minimal line-by-line diff. Good enough for committed reports.
      use std::fmt::Write;
      let mut out = String::new();
      let baseline_lines: Vec<&str> = baseline.lines().collect();
      let current_lines: Vec<&str> = current.lines().collect();
      let max_len = baseline_lines.len().max(current_lines.len());
      for i in 0..max_len {
          let b = baseline_lines.get(i).copied().unwrap_or("");
          let c = current_lines.get(i).copied().unwrap_or("");
          if b == c {
              writeln!(out, "  {b}").ok();
          } else {
              writeln!(out, "- {b}").ok();
              writeln!(out, "+ {c}").ok();
          }
      }
      out
  }

  /// Update one baseline file in place.
  ///
  /// # Errors
  ///
  /// Returns Err if the baseline file cannot be written.
  pub fn update_one_baseline(
      symbol: &str,
      current_asm: &str,
      baseline_dir: &Path,
  ) -> Result<(), String> {
      std::fs::create_dir_all(baseline_dir)
          .map_err(|err| format!("create {}: {err}", baseline_dir.display()))?;
      let normalized = normalize(current_asm);
      let path = baseline_dir.join(format!("{symbol}.asm"));
      std::fs::write(&path, normalized)
          .map_err(|err| format!("write {}: {err}", path.display()))
  }
  ```

- [ ] **Step 4: Run tests**

  ```sh
  cargo test -p lyng-js-bench asm_diff::tests::check_mode
  ```
  Expected: PASS.

- [ ] **Step 5: Wire `check_one_symbol`/`update_one_baseline` into the top-level `run()`**

  Replace the placeholder body of `pub fn run` with logic that:
  1. Loads `hot_opcodes::HotOpcodesConfig` from `options.opcodes_config`.
  2. For each opcode in `config.opcodes`, computes the symbol name (e.g., `lyng_js_vm::vm::dispatch_handlers::{family}::op_{name}` — the existing handlers in `crates/lyng-js/vm/src/vm/dispatch_handlers/` follow this pattern; verify with `nm target/release/libl*.rlib` if uncertain).
  3. In Check mode: capture asm, call `check_one_symbol`, collect outcomes, report.
  4. In Update mode: capture asm, call `update_one_baseline`.

  ```rust
  pub fn run(args: &[String]) -> Result<(), String> {
      let options = parse_args(args)?;
      let config = crate::hot_opcodes::HotOpcodesConfig::load(&options.opcodes_config)?;

      let mut failures: Vec<String> = Vec::new();
      let mut matches = 0_usize;
      let mut diffs = 0_usize;

      for entry in &config.opcodes {
          let symbol = symbol_name_for(&entry.name);
          let asm = match capture_symbol("lyng-js-vm", &symbol, options.capture_mode) {
              Ok(text) => text,
              Err(err) => {
                  failures.push(format!("{}: capture failed: {err}", entry.name));
                  continue;
              }
          };

          match options.mode {
              Mode::Check => match check_one_symbol(
                  &entry.name,
                  &asm,
                  &options.baseline_dir,
                  entry.aarch64_max_instructions,
              ) {
                  Ok(CheckOutcome::Match) => matches += 1,
                  Ok(CheckOutcome::Differs { diff, current_instr_count, baseline_instr_count }) => {
                      diffs += 1;
                      println!("=== {} (instr count: baseline {} -> current {}) ===",
                          entry.name, baseline_instr_count, current_instr_count);
                      println!("{diff}");
                  }
                  Err(err) => failures.push(format!("{}: {err}", entry.name)),
              },
              Mode::Update => {
                  if let Err(err) = update_one_baseline(&entry.name, &asm, &options.baseline_dir) {
                      failures.push(format!("{}: {err}", entry.name));
                  }
              }
          }
      }

      println!("asm-diff: {matches} match, {diffs} differ, {} failures", failures.len());
      if !failures.is_empty() {
          return Err(failures.join("\n"));
      }
      if options.mode == Mode::Check && diffs > 0 {
          return Err(format!("{diffs} handlers differ from baseline"));
      }
      Ok(())
  }

  fn symbol_name_for(opcode_name: &str) -> String {
      // Map PascalCase opcode names to current handler symbol paths.
      // During R-0 the handlers still live in dispatch_handlers/; their family
      // can be derived heuristically or via a table. Use a hand-maintained
      // mapping for now and verify against nm output.
      let snake = pascal_to_snake(opcode_name);
      format!("lyng_js_vm::vm::dispatch_handlers::op_{snake}")
  }

  fn pascal_to_snake(s: &str) -> String {
      let mut out = String::with_capacity(s.len() + 4);
      for (i, c) in s.chars().enumerate() {
          if c.is_ascii_uppercase() {
              if i != 0 {
                  out.push('_');
              }
              out.push(c.to_ascii_lowercase());
          } else {
              out.push(c);
          }
      }
      out
  }
  ```

- [ ] **Step 6: Smoke-test against the real binary**

  ```sh
  cargo build --release -p lyng-js-bench
  cargo build --release -p lyng-js-vm
  cargo run --release -p lyng-js-bench -- asm-diff --mode update \
    --opcodes-config tools/lyng-js-bench/hot-opcodes.toml \
    --baseline reports/js/lyng-js/dsl-asm-baseline-aarch64
  ```
  Expected: writes one baseline file per opcode under `reports/js/lyng-js/dsl-asm-baseline-aarch64/`. Note: some symbol paths may not resolve under the current handler layout — for each failure, either fix `symbol_name_for` heuristic or accept that those opcodes have non-standard handler paths and document in the report.

- [ ] **Step 7: Run a check pass against the freshly-written baselines**

  ```sh
  cargo run --release -p lyng-js-bench -- asm-diff --mode check
  ```
  Expected: clean exit (all match).

- [ ] **Step 8: Commit**

  ```sh
  git add tools/lyng-js-bench/src/asm_diff.rs reports/js/lyng-js/dsl-asm-baseline-aarch64/
  git commit -m "R-0: asm-diff check/update with per-opcode budget enforcement + initial alpha baselines"
  ```

- [ ] **Step 9: Mark ticket in review**

  ```sh
  dcat update <ASM_DIFF_TICKET_ID> --status in_review
  ```

---

## Phase 4 — Microbench subcommand (Tasks 12-16)

### Task 12: Add microbench subcommand skeleton + isolation gate

**Files:**
- Create: `tools/lyng-js-bench/src/microbench.rs`
- Modify: `tools/lyng-js-bench/src/cli.rs`
- Modify: `tools/lyng-js-bench/src/lib.rs`

- [ ] **Step 1: Mark ticket in progress**

  ```sh
  dcat update <MICROBENCH_TICKET_ID> --status in_progress
  ```

- [ ] **Step 2: Write failing test for loadavg gate**

  Create `tools/lyng-js-bench/src/microbench.rs`:
  ```rust
  //! `lyng-js-bench microbench` — per-opcode ns/dispatch with confidence interval.
  //!
  //! Loop construction: each opcode has a hand-written JS source snippet that
  //! exercises the opcode in a tight inner loop; the harness compiles it,
  //! runs it for N iterations, and divides total time by dispatch count.

  use std::path::PathBuf;

  #[derive(Debug, Clone, PartialEq)]
  pub struct MicrobenchOptions {
      pub opcodes_config: PathBuf,
      pub baseline: Option<PathBuf>,
      pub samples: usize,
      pub iters: u64,
      pub require_isolation: bool,
      pub output: Option<PathBuf>,
  }

  /// Run the microbench subcommand.
  ///
  /// # Errors
  ///
  /// Returns Err on CLI parsing failure, isolation-gate failure, or any
  /// per-opcode microbench error.
  pub fn run(args: &[String]) -> Result<(), String> {
      let options = parse_args(args)?;
      if options.require_isolation {
          gate_on_loadavg()?;
      }
      // Real per-opcode loop landed in subsequent tasks.
      println!("microbench: options = {options:?}");
      Err("microbench: not yet implemented (R-0 Task 13+)".into())
  }

  /// Abort if 1-min loadavg > 2.0.
  ///
  /// # Errors
  ///
  /// Returns Err if loadavg cannot be read or exceeds 2.0.
  pub fn gate_on_loadavg() -> Result<(), String> {
      let avg = read_loadavg_one_min()?;
      if avg > 2.0 {
          return Err(format!(
              "isolation gate: 1-min load average {avg:.2} > 2.0; \
               run on a quiesced machine or pass without --require-isolation"
          ));
      }
      Ok(())
  }

  fn read_loadavg_one_min() -> Result<f64, String> {
      #[cfg(target_os = "linux")]
      {
          let text = std::fs::read_to_string("/proc/loadavg")
              .map_err(|err| format!("read /proc/loadavg: {err}"))?;
          let first = text
              .split_whitespace()
              .next()
              .ok_or("malformed /proc/loadavg")?;
          first
              .parse::<f64>()
              .map_err(|err| format!("parse loadavg: {err}"))
      }

      #[cfg(target_os = "macos")]
      {
          // macOS: read via `uptime` parsing (libc::getloadavg is also an option).
          let output = std::process::Command::new("uptime")
              .output()
              .map_err(|err| format!("run uptime: {err}"))?;
          let text = String::from_utf8_lossy(&output.stdout);
          // Format: "... load averages: 1.23 4.56 7.89"
          let after = text
              .split("load average")
              .nth(1)
              .ok_or("uptime: no load average")?;
          let nums: Vec<&str> = after
              .split(|c: char| !c.is_ascii_digit() && c != '.')
              .filter(|s| !s.is_empty())
              .collect();
          let first = nums.first().ok_or("uptime: no first loadavg number")?;
          first
              .parse::<f64>()
              .map_err(|err| format!("parse loadavg: {err}"))
      }

      #[cfg(not(any(target_os = "linux", target_os = "macos")))]
      {
          Err("loadavg gate not implemented on this platform".into())
      }
  }

  fn parse_args(args: &[String]) -> Result<MicrobenchOptions, String> {
      let mut opcodes_config = PathBuf::from("tools/lyng-js-bench/hot-opcodes.toml");
      let mut baseline: Option<PathBuf> = None;
      let mut samples = 7;
      let mut iters = 5_000_000;
      let mut require_isolation = false;
      let mut output: Option<PathBuf> = None;

      let mut iter = args.iter().peekable();
      while let Some(arg) = iter.next() {
          match arg.as_str() {
              "--opcodes-config" => opcodes_config = iter.next().ok_or("--opcodes-config requires a path")?.into(),
              "--baseline" => baseline = Some(iter.next().ok_or("--baseline requires a path")?.into()),
              "--samples" => samples = iter.next().and_then(|s| s.parse().ok()).ok_or("--samples requires a number")?,
              "--iters" => iters = iter.next().and_then(|s| s.parse().ok()).ok_or("--iters requires a number")?,
              "--require-isolation" => require_isolation = true,
              "--output" => output = Some(iter.next().ok_or("--output requires a path")?.into()),
              "--help" | "-h" => return Err(help_text()),
              other => return Err(format!("microbench: unknown arg {other}\n\n{}", help_text())),
          }
      }

      Ok(MicrobenchOptions {
          opcodes_config,
          baseline,
          samples,
          iters,
          require_isolation,
          output,
      })
  }

  fn help_text() -> String {
      [
          "Usage: lyng-js-bench microbench [options]",
          "",
          "Options:",
          "  --opcodes-config PATH    Path to hot-opcodes.toml",
          "  --baseline PATH          Path to microbench-baseline.md for comparison",
          "  --samples N              Samples per opcode (default 7)",
          "  --iters N                Inner-loop iterations per sample (default 5_000_000)",
          "  --require-isolation      Abort if 1-min loadavg > 2.0",
          "  --output PATH            Write report to PATH (default stdout)",
      ]
      .join("\n")
  }

  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn parses_defaults() {
          let opts = parse_args(&[]).unwrap();
          assert_eq!(opts.samples, 7);
          assert_eq!(opts.iters, 5_000_000);
          assert!(!opts.require_isolation);
      }

      #[test]
      fn rejects_missing_samples_value() {
          let err = parse_args(&["--samples".into()]).unwrap_err();
          assert!(err.contains("requires a number"));
      }
  }
  ```

- [ ] **Step 3: Wire into CLI**

  Edit `tools/lyng-js-bench/src/cli.rs`:

  Add to `Command` enum:
  ```rust
  Microbench(Vec<String>),
  ```

  Add to `parse_command`:
  ```rust
  Some("microbench") => Ok(Command::Microbench(args[2..].to_vec())),
  ```

  Update `help_text`.

  Edit `lib.rs` to add `pub mod microbench;` and the dispatch case.

- [ ] **Step 4: Run tests**

  ```sh
  cargo test -p lyng-js-bench microbench
  ```
  Expected: pass.

- [ ] **Step 5: Smoke test isolation gate (interactive)**

  ```sh
  cargo run --release -p lyng-js-bench -- microbench --require-isolation --help 2>&1 | tail -5
  ```
  Expected: help text shows (gate not triggered because --help returns first).

- [ ] **Step 6: Commit**

  ```sh
  git add tools/lyng-js-bench/src/microbench.rs tools/lyng-js-bench/src/cli.rs tools/lyng-js-bench/src/lib.rs
  git commit -m "R-0: microbench subcommand skeleton + cross-platform loadavg gate"
  ```

---

### Task 13: Implement per-opcode loop generator (JS snippet per opcode)

**Files:**
- Create: `tools/lyng-js-bench/src/microbench/snippets.rs` (sub-module)
- Modify: `tools/lyng-js-bench/src/microbench.rs`

- [ ] **Step 1: Convert microbench.rs to a module directory**

  ```sh
  mkdir -p tools/lyng-js-bench/src/microbench
  git mv tools/lyng-js-bench/src/microbench.rs tools/lyng-js-bench/src/microbench/mod.rs
  ```

- [ ] **Step 2: Create snippets.rs**

  Create `tools/lyng-js-bench/src/microbench/snippets.rs`:
  ```rust
  //! Per-opcode JS source snippets used to drive the microbench inner loop.
  //!
  //! Each entry is a JS function that exercises the named opcode in a hot
  //! `for` loop. The harness compiles the function, calls it with the
  //! iteration count, and measures wall time. ns/dispatch = wall_time_ns /
  //! (iters * opcodes_per_iter).

  use std::collections::HashMap;

  #[derive(Debug, Clone)]
  pub struct Snippet {
      /// Pascal-case opcode name from `lyng_js_bytecode::Opcode`.
      pub opcode: &'static str,
      /// JS source — a function named `bench` that takes `iters` and runs the loop.
      pub source: &'static str,
      /// Number of times the opcode dispatches per loop iteration. Used to
      /// convert wall time to ns/dispatch.
      pub opcodes_per_iter: u32,
  }

  /// Hand-maintained snippet table. Add entries as new opcodes need coverage.
  /// Snippets that need accurate per-iter counts can be verified by running
  /// the snippet under `lyng-js-bench runtime --count-opcodes`.
  #[must_use]
  pub fn all_snippets() -> HashMap<&'static str, Snippet> {
      let mut map = HashMap::new();

      // Move: a single register-to-register copy per loop body line.
      // The compiler is permitted to fuse Move with other ops; the
      // opcodes_per_iter is verified empirically.
      map.insert("Move", Snippet {
          opcode: "Move",
          source: r"
              function bench(iters) {
                  let x = 1;
                  for (let i = 0; i < iters; i++) {
                      let a = x;
                      let b = a;
                      let c = b;
                      let d = c;
                      x = d;
                  }
                  return x;
              }
          ",
          opcodes_per_iter: 4, // 4 Move ops in the loop body (calibrate with --count-opcodes)
      });

      // Add: SMI fast-path arithmetic.
      map.insert("Add", Snippet {
          opcode: "Add",
          source: r"
              function bench(iters) {
                  let x = 0;
                  for (let i = 0; i < iters; i++) {
                      x = x + 1;
                  }
                  return x;
              }
          ",
          opcodes_per_iter: 1,
      });

      // GetNamedProperty: monomorphic property read.
      map.insert("GetNamedProperty", Snippet {
          opcode: "GetNamedProperty",
          source: r"
              function bench(iters) {
                  let o = { x: 1, y: 2, z: 3 };
                  let s = 0;
                  for (let i = 0; i < iters; i++) {
                      s = o.x + o.y + o.z;
                  }
                  return s;
              }
          ",
          opcodes_per_iter: 3,
      });

      // Jump: pure-jump tight loop.
      map.insert("Jump", Snippet {
          opcode: "Jump",
          source: r"
              function bench(iters) {
                  for (let i = 0; i < iters; i++) {}
                  return iters;
              }
          ",
          opcodes_per_iter: 1,
      });

      // Add additional snippets as needed for the hot-30 set.
      // For opcodes not present here, the microbench skips with a warning
      // (and the report records "no snippet" for that opcode).

      map
  }

  /// Look up a snippet by opcode name.
  #[must_use]
  pub fn for_opcode(name: &str) -> Option<Snippet> {
      all_snippets().get(name).cloned()
  }
  ```

- [ ] **Step 3: Wire into mod.rs**

  Add to `tools/lyng-js-bench/src/microbench/mod.rs` near the top:
  ```rust
  mod snippets;
  pub use snippets::{Snippet, all_snippets, for_opcode};
  ```

- [ ] **Step 4: Add test that snippets cover all hot opcodes**

  Add to the tests module:
  ```rust
  #[test]
  fn snippets_cover_hot_opcodes_or_emit_warning() {
      let config = crate::hot_opcodes::HotOpcodesConfig::load(
          std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/hot-opcodes.toml")),
      ).expect("load");
      let snippets = snippets::all_snippets();
      let mut missing: Vec<&str> = Vec::new();
      for entry in &config.opcodes {
          if !snippets.contains_key(entry.name.as_str()) {
              missing.push(entry.name.as_str());
          }
      }
      // R-0 ships with snippets for the top ~10 by frequency; the rest
      // emit "no snippet" warnings until DSL-0b coverage. Track here.
      println!("opcodes without microbench snippet: {missing:?}");
      // No assertion failure: partial coverage is acceptable at R-0,
      // but the warning is committed via the test output.
  }
  ```

- [ ] **Step 5: Run tests**

  ```sh
  cargo test -p lyng-js-bench microbench
  ```
  Expected: PASS.

- [ ] **Step 6: Commit**

  ```sh
  git add tools/lyng-js-bench/src/microbench/
  git commit -m "R-0: per-opcode JS snippets for microbench inner loop"
  ```

---

### Task 14: Implement timing harness + sample collection

**Files:**
- Create: `tools/lyng-js-bench/src/microbench/timing.rs`
- Modify: `tools/lyng-js-bench/src/microbench/mod.rs`

- [ ] **Step 1: Write failing tests**

  Create `tools/lyng-js-bench/src/microbench/timing.rs`:
  ```rust
  //! Timing harness: high-resolution monotonic clock + sample aggregation.

  use std::time::{Duration, Instant};

  /// One sample: wall-clock duration plus the dispatch count it measures.
  #[derive(Debug, Clone, Copy)]
  pub struct Sample {
      pub elapsed: Duration,
      pub dispatches: u64,
  }

  impl Sample {
      #[must_use]
      pub fn ns_per_dispatch(&self) -> f64 {
          let ns = self.elapsed.as_nanos() as f64;
          ns / (self.dispatches as f64)
      }
  }

  /// Aggregate sample statistics.
  #[derive(Debug, Clone)]
  pub struct SampleStats {
      pub samples: Vec<Sample>,
      pub median_ns_per_dispatch: f64,
      pub min_ns_per_dispatch: f64,
      pub max_ns_per_dispatch: f64,
      /// Half-width of a 95% confidence interval around the median, in ns.
      /// Computed via the inter-quartile bootstrap as a robust approximation.
      pub ci95_half_width_ns: f64,
  }

  impl SampleStats {
      #[must_use]
      pub fn from_samples(mut samples: Vec<Sample>) -> Self {
          assert!(!samples.is_empty(), "need at least one sample");
          let mut rates: Vec<f64> = samples.iter().map(Sample::ns_per_dispatch).collect();
          rates.sort_by(|a, b| a.partial_cmp(b).unwrap());

          let median = rates[rates.len() / 2];
          let min = *rates.first().unwrap();
          let max = *rates.last().unwrap();

          // Conservative CI: half the IQR (75th - 25th percentile) is a
          // robust dispersion estimate that doesn't assume normality.
          let q1 = rates[rates.len() / 4];
          let q3 = rates[(rates.len() * 3) / 4];
          let ci = (q3 - q1) / 2.0;

          samples.sort_by(|a, b| {
              a.ns_per_dispatch()
                  .partial_cmp(&b.ns_per_dispatch())
                  .unwrap()
          });

          Self {
              samples,
              median_ns_per_dispatch: median,
              min_ns_per_dispatch: min,
              max_ns_per_dispatch: max,
              ci95_half_width_ns: ci,
          }
      }
  }

  /// Run `f` once, returning (elapsed, dispatches).
  ///
  /// The `dispatches` value must be passed in by the caller — it's the
  /// opcode count × inner iteration count.
  pub fn time_once<F: FnOnce() -> ()>(dispatches: u64, f: F) -> Sample {
      let start = Instant::now();
      f();
      let elapsed = start.elapsed();
      Sample { elapsed, dispatches }
  }

  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn ns_per_dispatch_computes_correctly() {
          let sample = Sample {
              elapsed: Duration::from_nanos(1_000),
              dispatches: 100,
          };
          assert!((sample.ns_per_dispatch() - 10.0).abs() < 1e-9);
      }

      #[test]
      fn stats_from_samples_basic() {
          let samples = vec![
              Sample { elapsed: Duration::from_nanos(100), dispatches: 10 },  // 10 ns
              Sample { elapsed: Duration::from_nanos(200), dispatches: 10 },  // 20 ns
              Sample { elapsed: Duration::from_nanos(300), dispatches: 10 },  // 30 ns
              Sample { elapsed: Duration::from_nanos(400), dispatches: 10 },  // 40 ns
              Sample { elapsed: Duration::from_nanos(500), dispatches: 10 },  // 50 ns
          ];
          let stats = SampleStats::from_samples(samples);
          assert!((stats.median_ns_per_dispatch - 30.0).abs() < 1e-9);
          assert!((stats.min_ns_per_dispatch - 10.0).abs() < 1e-9);
          assert!((stats.max_ns_per_dispatch - 50.0).abs() < 1e-9);
      }

      #[test]
      fn time_once_returns_positive_elapsed() {
          let sample = time_once(1000, || {
              std::hint::black_box((0..1000).fold(0_u64, |a, b| a.wrapping_add(b)));
          });
          assert!(sample.elapsed.as_nanos() > 0);
          assert_eq!(sample.dispatches, 1000);
      }
  }
  ```

- [ ] **Step 2: Wire into mod.rs**

  Add `mod timing;` near the top.

- [ ] **Step 3: Run tests**

  ```sh
  cargo test -p lyng-js-bench microbench::timing
  ```
  Expected: 3 PASS.

- [ ] **Step 4: Commit**

  ```sh
  git add tools/lyng-js-bench/src/microbench/timing.rs tools/lyng-js-bench/src/microbench/mod.rs
  git commit -m "R-0: microbench timing harness + sample stats with robust CI"
  ```

---

### Task 15: Wire end-to-end microbench runner

**Files:**
- Modify: `tools/lyng-js-bench/src/microbench/mod.rs`

- [ ] **Step 1: Implement the end-to-end runner**

  Replace the placeholder `pub fn run` body in `tools/lyng-js-bench/src/microbench/mod.rs`:
  ```rust
  pub fn run(args: &[String]) -> Result<(), String> {
      let options = parse_args(args)?;
      if options.require_isolation {
          gate_on_loadavg()?;
      }

      let config = crate::hot_opcodes::HotOpcodesConfig::load(&options.opcodes_config)?;
      let snippet_table = snippets::all_snippets();

      let mut report_lines: Vec<String> = Vec::new();
      report_lines.push("# Microbench Baseline".to_string());
      report_lines.push(String::new());
      report_lines.push(format!("Samples per opcode: {}", options.samples));
      report_lines.push(format!("Inner iters per sample: {}", options.iters));
      report_lines.push(String::new());
      report_lines.push("| Opcode | Samples | Median ns/dispatch | Min | Max | CI95 half-width | Snippet ratio |".to_string());
      report_lines.push("|---|---:|---:|---:|---:|---:|---|".to_string());

      for entry in &config.opcodes {
          let Some(snippet) = snippet_table.get(entry.name.as_str()) else {
              report_lines.push(format!("| {} | — | no snippet | — | — | — | — |", entry.name));
              continue;
          };

          let samples = match run_snippet(snippet, options.iters, options.samples) {
              Ok(s) => s,
              Err(err) => {
                  report_lines.push(format!("| {} | — | error: {err} | — | — | — | — |", entry.name));
                  continue;
              }
          };
          let stats = timing::SampleStats::from_samples(samples);

          report_lines.push(format!(
              "| `{}` | {} | {:.2} | {:.2} | {:.2} | ±{:.2} | {} ops/iter |",
              entry.name,
              stats.samples.len(),
              stats.median_ns_per_dispatch,
              stats.min_ns_per_dispatch,
              stats.max_ns_per_dispatch,
              stats.ci95_half_width_ns,
              snippet.opcodes_per_iter,
          ));
      }

      let body = report_lines.join("\n") + "\n";
      if let Some(out) = options.output.as_ref() {
          std::fs::write(out, &body)
              .map_err(|err| format!("write {}: {err}", out.display()))?;
          println!("microbench: wrote {}", out.display());
      } else {
          println!("{body}");
      }
      Ok(())
  }

  fn run_snippet(
      snippet: &snippets::Snippet,
      iters: u64,
      samples: usize,
  ) -> Result<Vec<timing::Sample>, String> {
      // Build a VM, compile the snippet's `bench` function, then call it
      // `samples` times with `iters` as its argument. Wrap each call in
      // `time_once`. Compute dispatches = iters * opcodes_per_iter.
      //
      // Real implementation depends on the existing
      // lyng_js_vm::Vm public API for compile+invoke; see
      // tools/lyng-js-bench/src/runtime.rs for the patterns used by
      // the runtime suite.
      use std::sync::Arc;
      // (Pseudocode — real wiring uses Vm::evaluate_script_*; consult
      // tools/lyng-js-bench/src/runtime.rs:480-560 for the working pattern.)
      let _ = (snippet, iters, samples);
      let mut out: Vec<timing::Sample> = Vec::new();
      // Warm-up: run once unmeasured.
      // Measured samples: run `samples` times.
      // For each measured run:
      //   timing::time_once(iters * snippet.opcodes_per_iter as u64, || { vm.invoke(...) })
      // Push to out.
      //
      // Until the wiring lands, return a synthetic sample so the
      // report-generation path is exercisable.
      out.push(timing::Sample {
          elapsed: std::time::Duration::from_nanos(100),
          dispatches: 10,
      });
      Ok(out)
  }
  ```

- [ ] **Step 2: Replace the placeholder `run_snippet` with real wiring**

  Read `tools/lyng-js-bench/src/runtime.rs` lines 480-560 to find the pattern for compiling + invoking a function. Adapt it for `run_snippet`. Typical shape:

  ```rust
  let agent = lyng_js_env::Agent::new();
  let mut vm = lyng_js_vm::Vm::new();
  // ... evaluate script that defines `bench` ...
  // ... find the `bench` global ...
  // ... for each sample: time_once { vm.invoke(bench, [iters_as_value]) } ...
  ```

  Inspect runtime.rs for the canonical pattern (it sets up agent + host + registry consistently). Use the same.

- [ ] **Step 3: Build & smoke test**

  ```sh
  cargo build --release -p lyng-js-bench
  cargo run --release -p lyng-js-bench -- microbench --samples 3 --iters 100000
  ```
  Expected: prints a markdown table to stdout with at least the 4 hand-written snippets (Move, Add, GetNamedProperty, Jump) showing ns/dispatch numbers.

- [ ] **Step 4: Commit**

  ```sh
  git add tools/lyng-js-bench/src/microbench/mod.rs
  git commit -m "R-0: microbench end-to-end runner with per-opcode snippets"
  ```

---

### Task 16: Write initial `microbench-baseline.md`

**Files:**
- Create: `reports/js/lyng-js/microbench-baseline.md`

- [ ] **Step 1: Run microbench in isolation**

  Quiesce the machine (close other apps, wait for loadavg < 2.0). Then:

  ```sh
  cargo run --release -p lyng-js-bench -- microbench \
    --require-isolation \
    --samples 7 \
    --iters 5000000 \
    --output reports/js/lyng-js/microbench-baseline.md
  ```
  Expected: writes baseline file.

- [ ] **Step 2: Verify the file is sensibly formatted**

  ```sh
  head -30 reports/js/lyng-js/microbench-baseline.md
  ```
  Expected: well-formed markdown table.

- [ ] **Step 3: Commit**

  ```sh
  git add reports/js/lyng-js/microbench-baseline.md
  git commit -m "R-0: initial microbench baseline (pre-DSL alpha substrate)"
  ```

- [ ] **Step 4: Mark microbench ticket in review**

  ```sh
  dcat update <MICROBENCH_TICKET_ID> --status in_review
  ```

---

## Phase 5 — capture-llint subcommand (Tasks 17-19)

### Task 17: capture-llint skeleton + system mode

**Files:**
- Create: `tools/lyng-js-bench/src/capture_llint.rs`
- Modify: `tools/lyng-js-bench/src/cli.rs`
- Modify: `tools/lyng-js-bench/src/lib.rs`

- [ ] **Step 1: Mark ticket in progress**

  ```sh
  dcat update <CAPTURE_LLINT_TICKET_ID> --status in_progress
  ```

- [ ] **Step 2: Create the module**

  Create `tools/lyng-js-bench/src/capture_llint.rs`:
  ```rust
  //! `lyng-js-bench capture-llint` — extract JSC LLInt handler asm/source.
  //!
  //! Source-mode strategy:
  //! - `auto`: try system → local → excerpt in order; report which mode produced each opcode.
  //! - `system`: `otool -tvV` on the system JSC binary; finds `_llint_op_*` symbols.
  //! - `local`: same approach but on a locally built JSC binary.
  //! - `excerpt`: parse JSC's offlineasm source files directly; produces
  //!   source-level reference instead of concrete asm.

  use std::path::PathBuf;

  #[derive(Debug, Clone, PartialEq)]
  pub struct CaptureLlintOptions {
      pub source: Source,
      pub jsc_binary: Option<PathBuf>,
      pub jsc_source: Option<PathBuf>,
      pub opcodes: Vec<String>,
      pub output_dir: PathBuf,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum Source {
      Auto,
      System,
      Local,
      Excerpt,
  }

  /// Run capture-llint.
  ///
  /// # Errors
  ///
  /// Returns Err on CLI failure or when no source mode succeeds for any
  /// requested opcode.
  pub fn run(args: &[String]) -> Result<(), String> {
      let options = parse_args(args)?;
      std::fs::create_dir_all(&options.output_dir)
          .map_err(|err| format!("create output dir: {err}"))?;

      let mut produced: Vec<(String, Source)> = Vec::new();
      let mut failures: Vec<String> = Vec::new();

      for opcode in &options.opcodes {
          match capture_one(opcode, &options) {
              Ok(mode) => produced.push((opcode.clone(), mode)),
              Err(err) => failures.push(format!("{opcode}: {err}")),
          }
      }

      // Write a summary report at output_dir/README.md.
      let mut summary = String::from("# JSC LLInt reference asm\n\nCaptured by `lyng-js-bench capture-llint`.\n\n| Opcode | Source mode |\n|---|---|\n");
      for (opcode, mode) in &produced {
          summary.push_str(&format!("| `{opcode}` | {mode:?} |\n"));
      }
      let summary_path = options.output_dir.join("README.md");
      std::fs::write(&summary_path, summary)
          .map_err(|err| format!("write {}: {err}", summary_path.display()))?;

      println!("captured {} opcodes, {} failures", produced.len(), failures.len());
      for failure in &failures {
          eprintln!("  {failure}");
      }
      if produced.is_empty() {
          Err("no opcodes captured".into())
      } else {
          Ok(())
      }
  }

  fn capture_one(opcode: &str, options: &CaptureLlintOptions) -> Result<Source, String> {
      let modes: Vec<Source> = match options.source {
          Source::Auto => vec![Source::System, Source::Local, Source::Excerpt],
          single => vec![single],
      };
      let mut errors = Vec::new();
      for mode in modes {
          match try_mode(mode, opcode, options) {
              Ok(asm) => {
                  let path = options.output_dir.join(format!("{opcode}.md"));
                  let body = format!(
                      "# JSC LLInt reference: `{opcode}`\n\nCapture mode: {mode:?}\n\n```asm\n{asm}\n```\n"
                  );
                  std::fs::write(&path, body)
                      .map_err(|err| format!("write {}: {err}", path.display()))?;
                  return Ok(mode);
              }
              Err(err) => errors.push(format!("{mode:?}: {err}")),
          }
      }
      Err(errors.join(" | "))
  }

  fn try_mode(mode: Source, opcode: &str, options: &CaptureLlintOptions) -> Result<String, String> {
      match mode {
          Source::System | Source::Local => {
              let binary = match mode {
                  Source::System => options.jsc_binary.clone()
                      .unwrap_or_else(|| PathBuf::from("/System/Library/Frameworks/JavaScriptCore.framework/Versions/Current/Helpers/jsc")),
                  Source::Local => options.jsc_binary.clone()
                      .ok_or("--jsc-binary required for local mode")?,
                  _ => unreachable!(),
              };
              capture_from_binary(&binary, opcode)
          }
          Source::Excerpt => {
              let source_root = options.jsc_source.clone()
                  .ok_or("--jsc-source required for excerpt mode")?;
              capture_from_source(&source_root, opcode)
          }
          Source::Auto => unreachable!("Auto is expanded earlier"),
      }
  }

  fn capture_from_binary(binary: &std::path::Path, opcode: &str) -> Result<String, String> {
      let symbol = format!("_llint_{opcode}");
      let tool = if cfg!(target_os = "macos") { "otool" } else { "objdump" };
      let args: Vec<String> = if cfg!(target_os = "macos") {
          vec!["-tvV".into(), binary.display().to_string()]
      } else {
          vec!["-d".into(), "--no-show-raw-insn".into(), binary.display().to_string()]
      };
      let output = std::process::Command::new(tool)
          .args(args)
          .output()
          .map_err(|err| format!("run {tool}: {err}"))?;
      if !output.status.success() {
          return Err(format!("{tool} exited {}", output.status));
      }
      let text = String::from_utf8_lossy(&output.stdout);
      extract_llint_symbol(&text, &symbol)
  }

  fn extract_llint_symbol(disasm: &str, symbol: &str) -> Result<String, String> {
      let mut iter = disasm.lines();
      let mut body: Vec<String> = Vec::new();
      let mut found = false;
      while let Some(line) = iter.next() {
          if !found {
              if line.contains(symbol) {
                  found = true;
                  body.push(line.to_string());
              }
          } else {
              // Stop at the next `_llint_op_*` symbol.
              if line.contains("_llint_op_") && !line.contains(symbol) {
                  break;
              }
              body.push(line.to_string());
              if body.len() > 200 {
                  // Cap per-symbol output to keep reports readable.
                  break;
              }
          }
      }
      if !found {
          return Err(format!("symbol {symbol} not found (binary may be stripped)"));
      }
      Ok(body.join("\n"))
  }

  fn capture_from_source(source_root: &std::path::Path, opcode: &str) -> Result<String, String> {
      // Search LowLevelInterpreter64.asm and LowLevelInterpreter.asm for
      // `llintOp(op_xxx, ...` or `llintOpWithMetadata(op_xxx, ...`.
      let candidates = [
          source_root.join("Source/JavaScriptCore/llint/LowLevelInterpreter64.asm"),
          source_root.join("Source/JavaScriptCore/llint/LowLevelInterpreter.asm"),
      ];
      let pattern = format!("llintOp(") + opcode + ",";
      let pattern_meta = format!("llintOpWithMetadata(") + opcode + ",";

      for file in &candidates {
          let Ok(text) = std::fs::read_to_string(file) else { continue; };
          for needle in [&pattern, &pattern_meta] {
              if let Some(start) = text.find(needle.as_str()) {
                  let body: String = text[start..].lines().take(80).collect::<Vec<_>>().join("\n");
                  return Ok(body);
              }
          }
      }
      Err("opcode not found in any offlineasm source file".into())
  }

  fn parse_args(args: &[String]) -> Result<CaptureLlintOptions, String> {
      let mut source = Source::Auto;
      let mut jsc_binary: Option<PathBuf> = None;
      let mut jsc_source: Option<PathBuf> = None;
      let mut opcodes: Vec<String> = Vec::new();
      let mut output_dir = PathBuf::from("reports/js/lyng-js/llint-reference");

      let mut iter = args.iter().peekable();
      while let Some(arg) = iter.next() {
          match arg.as_str() {
              "--source" => match iter.next().map(String::as_str) {
                  Some("auto") => source = Source::Auto,
                  Some("system") => source = Source::System,
                  Some("local") => source = Source::Local,
                  Some("excerpt") => source = Source::Excerpt,
                  Some(other) => return Err(format!("--source: unknown {other}")),
                  None => return Err("--source requires a value".into()),
              },
              "--jsc-binary" => jsc_binary = Some(iter.next().ok_or("--jsc-binary requires a path")?.into()),
              "--jsc-source" => jsc_source = Some(iter.next().ok_or("--jsc-source requires a path")?.into()),
              "--opcodes" => {
                  let list = iter.next().ok_or("--opcodes requires a comma-separated list")?;
                  opcodes.extend(list.split(',').map(str::trim).map(String::from));
              }
              "--output" => output_dir = iter.next().ok_or("--output requires a path")?.into(),
              "--help" | "-h" => return Err(help_text()),
              other => return Err(format!("capture-llint: unknown arg {other}\n\n{}", help_text())),
          }
      }

      if opcodes.is_empty() {
          return Err("--opcodes <comma-separated list> is required".into());
      }
      Ok(CaptureLlintOptions { source, jsc_binary, jsc_source, opcodes, output_dir })
  }

  fn help_text() -> String {
      [
          "Usage: lyng-js-bench capture-llint --opcodes <list> [options]",
          "",
          "Options:",
          "  --source auto|system|local|excerpt   Capture strategy (default auto)",
          "  --jsc-binary PATH                    JSC binary for system/local mode",
          "  --jsc-source PATH                    WebKit source root for excerpt mode",
          "  --opcodes a,b,c                      Comma-separated LLInt opcode names (without `_llint_` prefix)",
          "  --output PATH                        Output directory (default reports/js/lyng-js/llint-reference)",
      ]
      .join("\n")
  }

  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn parses_minimal_args() {
          let opts = parse_args(&[
              "--opcodes".into(),
              "op_add,op_mov".into(),
          ]).unwrap();
          assert_eq!(opts.source, Source::Auto);
          assert_eq!(opts.opcodes, vec!["op_add", "op_mov"]);
      }

      #[test]
      fn rejects_missing_opcodes() {
          let err = parse_args(&[]).unwrap_err();
          assert!(err.contains("--opcodes"));
      }
  }
  ```

- [ ] **Step 3: Wire into CLI**

  Add `CaptureLlint(Vec<String>)` to `Command` enum; add `Some("capture-llint") => Ok(Command::CaptureLlint(args[2..].to_vec()))` to parser; update help_text and lib.rs.

- [ ] **Step 4: Run tests**

  ```sh
  cargo test -p lyng-js-bench capture_llint
  ```
  Expected: pass.

- [ ] **Step 5: Commit**

  ```sh
  git add tools/lyng-js-bench/src/capture_llint.rs tools/lyng-js-bench/src/cli.rs tools/lyng-js-bench/src/lib.rs
  git commit -m "R-0: capture-llint subcommand with auto/system/local/excerpt modes"
  ```

---

### Task 18: Write `llint-reference-setup.md` for local mode

**Files:**
- Create: `docs/lyng-js/llint-reference-setup.md`

- [ ] **Step 1: Write the setup doc**

  Create `docs/lyng-js/llint-reference-setup.md`:
  ```markdown
  # JSC LLInt reference capture — local-build setup

  `lyng-js-bench capture-llint` uses three modes (`system`, `local`, `excerpt`).
  This doc covers the `local` mode: building JSC from source so the binary
  retains `_llint_op_*` symbols even when the system framework is stripped.

  ## When you need this

  - macOS ships `JavaScriptCore.framework` but the symbols may be stripped.
    Run `nm /System/Library/Frameworks/JavaScriptCore.framework/Versions/Current/Helpers/jsc | grep _llint_op_add`
    to test. If it returns nothing, the system binary won't work for capture.
  - Linux: most distributions don't ship `jsc`; you'll need a local build.

  ## Building WebKit/JSC

  Clone the WebKit repository and build JSC in debug mode (symbols retained):
  ```sh
  git clone https://github.com/WebKit/WebKit.git
  cd WebKit
  Tools/Scripts/build-jsc --debug
  ```

  Output binary: `WebKitBuild/Debug/bin/jsc` (Linux) or
  `WebKitBuild/Debug/JavaScriptCore.framework/Helpers/jsc` (macOS).

  Verify symbols are present:
  ```sh
  nm WebKitBuild/Debug/bin/jsc | grep _llint_op_add
  ```
  Expected: one or more matches.

  ## Running capture-llint in local mode

  ```sh
  cargo run --release -p lyng-js-bench -- capture-llint \
    --source local \
    --jsc-binary /path/to/WebKitBuild/Debug/bin/jsc \
    --opcodes op_add,op_mov,op_jmp,op_get_by_id,op_put_by_id,op_call,op_ret \
    --output reports/js/lyng-js/llint-reference
  ```

  ## Running capture-llint in excerpt mode (no build required)

  Excerpt mode reads the offlineasm source files directly from a WebKit
  source checkout — no compilation needed.

  ```sh
  cargo run --release -p lyng-js-bench -- capture-llint \
    --source excerpt \
    --jsc-source /Users/sondre/dev/WebKit \
    --opcodes op_add,op_mov,op_jmp,op_get_by_id \
    --output reports/js/lyng-js/llint-reference
  ```

  This produces source-level (offlineasm pseudo-code) reference rather than
  concrete asm. It's always available; the trade-off is one level removed
  from the actual machine code.
  ```

- [ ] **Step 2: Commit**

  ```sh
  git add docs/lyng-js/llint-reference-setup.md
  git commit -m "R-0: setup doc for capture-llint local-build mode"
  ```

---

### Task 19: Capture initial LLInt references

**Files:**
- Create: `reports/js/lyng-js/llint-reference/` (one file per opcode + README)

- [ ] **Step 1: Identify the top-30 opcodes' LLInt equivalents**

  Map each lyng-js opcode name to its JSC counterpart (e.g., `Add` → `op_add`, `GetNamedProperty` → `op_get_by_id`, `Move` → `op_mov`, `Jump` → `op_jmp`, `Return` → `op_ret`, `LoadLocal0` → handled by `op_mov` family, etc.). Some opcodes will have no direct equivalent — note those in the README.

- [ ] **Step 2: Run capture-llint in auto mode for the mappable subset**

  ```sh
  cargo run --release -p lyng-js-bench -- capture-llint \
    --source auto \
    --jsc-binary /System/Library/Frameworks/JavaScriptCore.framework/Versions/Current/Helpers/jsc \
    --jsc-source /Users/sondre/dev/WebKit \
    --opcodes op_add,op_mov,op_jmp,op_get_by_id,op_put_by_id,op_call,op_ret,op_loop_hint,op_jtrue,op_jfalse,op_jmp_short,op_get_array_length,op_sub,op_mul,op_negate,op_bitand,op_bitor,op_bitxor,op_lshift,op_rshift,op_to_property_key \
    --output reports/js/lyng-js/llint-reference
  ```
  Expected: per-opcode markdown files emitted under `reports/js/lyng-js/llint-reference/`, plus a `README.md` summary listing which mode produced each.

- [ ] **Step 3: Spot-check one output file**

  ```sh
  cat reports/js/lyng-js/llint-reference/op_add.md
  ```
  Expected: a clean reference file with asm or offlineasm content.

- [ ] **Step 4: Commit**

  ```sh
  git add reports/js/lyng-js/llint-reference/
  git commit -m "R-0: initial JSC LLInt reference asm for top-20 opcodes"
  ```

- [ ] **Step 5: Mark ticket in review**

  ```sh
  dcat update <CAPTURE_LLINT_TICKET_ID> --status in_review
  ```

---

## Phase 6 — Slow-path-share counter (Tasks 20-22)

### Task 20: Add `SlowPathCounterStore` to VM

**Files:**
- Create: `crates/lyng-js/vm/src/slow_path_counts.rs`
- Modify: `crates/lyng-js/vm/src/lib.rs`
- Modify: `crates/lyng-js/vm/src/vm.rs`
- Modify: `crates/lyng-js/vm/Cargo.toml` (if feature flag absent)

- [ ] **Step 1: Mark ticket in progress**

  ```sh
  dcat update <SLOW_PATH_SHARE_TICKET_ID> --status in_progress
  ```

- [ ] **Step 2: Create `slow_path_counts.rs`**

  Mirror the existing `opcode_counts.rs` shape:
  ```rust
  //! Per-opcode slow-path-entry counters, separated into "semantic" entries
  //! (called from cold stubs or hot-handler fall-back) and "safepoint"
  //! entries (called from warm-handler poll bridges).
  //!
  //! Gated behind the `opcode-counters` Cargo feature. Production builds
  //! carry no counter code.

  use std::cell::Cell;
  use lyng_js_bytecode::{Opcode, OPCODE_COUNT};

  const OPCODE_COUNT_LEN: usize = OPCODE_COUNT as usize;

  pub struct SlowPathCounterStore {
      semantic: Box<[Cell<u64>]>,
      safepoint: Box<[Cell<u64>]>,
  }

  impl SlowPathCounterStore {
      pub fn new() -> Self {
          Self {
              semantic: (0..OPCODE_COUNT_LEN).map(|_| Cell::new(0)).collect::<Vec<_>>().into_boxed_slice(),
              safepoint: (0..OPCODE_COUNT_LEN).map(|_| Cell::new(0)).collect::<Vec<_>>().into_boxed_slice(),
          }
      }

      #[inline]
      pub fn record_semantic(&self, opcode: Opcode) {
          let slot = &self.semantic[usize::from(opcode as u8)];
          slot.set(slot.get().saturating_add(1));
      }

      #[inline]
      pub fn record_safepoint(&self, opcode: Opcode) {
          let slot = &self.safepoint[usize::from(opcode as u8)];
          slot.set(slot.get().saturating_add(1));
      }

      pub fn reset(&self) {
          for slot in &self.semantic { slot.set(0); }
          for slot in &self.safepoint { slot.set(0); }
      }

      pub fn snapshot(&self) -> SlowPathCounts {
          SlowPathCounts {
              semantic: self.semantic.iter().map(Cell::get).collect(),
              safepoint: self.safepoint.iter().map(Cell::get).collect(),
          }
      }
  }

  impl Default for SlowPathCounterStore {
      fn default() -> Self {
          Self::new()
      }
  }

  #[derive(Clone, Debug, Eq, PartialEq, Default)]
  pub struct SlowPathCounts {
      semantic: Vec<u64>,
      safepoint: Vec<u64>,
  }

  impl SlowPathCounts {
      #[must_use]
      pub fn semantic(&self, opcode: Opcode) -> u64 {
          self.semantic.get(usize::from(opcode as u8)).copied().unwrap_or(0)
      }

      #[must_use]
      pub fn safepoint(&self, opcode: Opcode) -> u64 {
          self.safepoint.get(usize::from(opcode as u8)).copied().unwrap_or(0)
      }
  }

  #[cfg(test)]
  mod tests {
      use super::*;
      use lyng_js_bytecode::Opcode;

      #[test]
      fn records_semantic_independently_of_safepoint() {
          let store = SlowPathCounterStore::new();
          store.record_semantic(Opcode::Add);
          store.record_semantic(Opcode::Add);
          store.record_safepoint(Opcode::Add);
          let snap = store.snapshot();
          assert_eq!(snap.semantic(Opcode::Add), 2);
          assert_eq!(snap.safepoint(Opcode::Add), 1);
      }

      #[test]
      fn reset_clears_both_counters() {
          let store = SlowPathCounterStore::new();
          store.record_semantic(Opcode::Move);
          store.record_safepoint(Opcode::Move);
          store.reset();
          let snap = store.snapshot();
          assert_eq!(snap.semantic(Opcode::Move), 0);
          assert_eq!(snap.safepoint(Opcode::Move), 0);
      }
  }
  ```

- [ ] **Step 3: Wire into the VM crate**

  Edit `crates/lyng-js/vm/src/lib.rs` to add:
  ```rust
  #[cfg(feature = "opcode-counters")]
  pub mod slow_path_counts;
  ```

  In `crates/lyng-js/vm/src/vm.rs`, find the `opcode_dispatch_counts` field (around line 157) and add a sibling:
  ```rust
  #[cfg(feature = "opcode-counters")]
  slow_path_counts: Option<crate::slow_path_counts::SlowPathCounterStore>,
  ```

  In `Vm::new` (around line 224), initialize:
  ```rust
  #[cfg(feature = "opcode-counters")]
  slow_path_counts: None,
  ```

  Add accessor methods next to the existing `enable_opcode_dispatch_counts`:
  ```rust
  #[cfg(feature = "opcode-counters")]
  pub fn enable_slow_path_counts(&mut self) {
      if self.slow_path_counts.is_none() {
          self.slow_path_counts = Some(crate::slow_path_counts::SlowPathCounterStore::new());
      }
  }

  #[cfg(feature = "opcode-counters")]
  pub fn disable_slow_path_counts(&mut self) {
      self.slow_path_counts = None;
  }

  #[cfg(feature = "opcode-counters")]
  pub fn reset_slow_path_counts(&mut self) {
      if let Some(store) = &self.slow_path_counts {
          store.reset();
      }
  }

  #[cfg(feature = "opcode-counters")]
  pub fn slow_path_counts(&self) -> Option<crate::slow_path_counts::SlowPathCounts> {
      self.slow_path_counts.as_ref().map(|store| store.snapshot())
  }

  #[cfg(feature = "opcode-counters")]
  pub fn slow_path_counts_enabled(&self) -> bool {
      self.slow_path_counts.is_some()
  }
  ```

- [ ] **Step 4: Run tests**

  ```sh
  cargo test --features opcode-counters -p lyng-js-vm slow_path_counts
  ```
  Expected: 2 PASS.

- [ ] **Step 5: Verify Vm public API still compiles**

  ```sh
  cargo build --release -p lyng-js-vm --features opcode-counters
  cargo build --release -p lyng-js-vm   # without feature
  ```
  Expected: both succeed.

- [ ] **Step 6: Commit**

  ```sh
  git add crates/lyng-js/vm/src/slow_path_counts.rs crates/lyng-js/vm/src/lib.rs crates/lyng-js/vm/src/vm.rs
  git commit -m "R-0: SlowPathCounterStore for slow-path-semantic vs safepoint counts"
  ```

---

### Task 21: Wire `--count-slow-path-share` flag in bench tool

**Files:**
- Modify: `tools/lyng-js-bench/src/runtime.rs` (where `--count-opcodes` lives)
- Modify: `tools/lyng-js-bench/src/v8suite.rs` (same)

- [ ] **Step 1: Add the flag to runtime suite options**

  Find the `--count-opcodes` arg parsing in `tools/lyng-js-bench/src/runtime.rs:232` and add a sibling:
  ```rust
  "--count-slow-path-share" => {
      options.count_slow_path_share = true;
  }
  ```

  Add the `count_slow_path_share: bool` field to the options struct (search for `count_opcodes` to find it).

- [ ] **Step 2: Wire the field through to Vm enable/disable calls**

  Wherever `vm.enable_opcode_dispatch_counts()` is called (find via grep), add a sibling call when `options.count_slow_path_share` is set:
  ```rust
  if options.count_slow_path_share {
      vm.enable_slow_path_counts();
  }
  ```

- [ ] **Step 3: Emit slow-path counts in report output**

  After existing `--count-opcodes` reporting, add a "Slow-path share" table that joins each opcode's dispatch count with its `slow_path_counts().semantic(opcode)` value and computes the share percent.

  Skeleton:
  ```rust
  if options.count_slow_path_share {
      let Some(slow) = vm.slow_path_counts() else { /* error */ };
      let Some(dispatches) = vm.opcode_dispatch_counts() else { /* error */ };
      for entry in dispatches.iter().filter(|e| e.count() > 0) {
          let semantic = slow.semantic(entry.opcode());
          let safepoint = slow.safepoint(entry.opcode());
          let share = (semantic as f64) / (entry.count() as f64);
          // Append to report.
      }
  }
  ```

- [ ] **Step 4: Add the same flag to v8suite.rs**

  Mirror Step 1-3 in `tools/lyng-js-bench/src/v8suite.rs`.

- [ ] **Step 5: Run tests**

  ```sh
  cargo test --features lyng-js-vm/opcode-counters -p lyng-js-bench
  ```
  Expected: pass.

- [ ] **Step 6: Smoke test**

  ```sh
  cargo run --release -p lyng-js-bench --features lyng-js-vm/opcode-counters -- runtime --count-opcodes --count-slow-path-share --preset inner-loop
  ```
  Expected: report includes the slow-path-share table. Counts will all be 0 until DSL handlers actually call `record_semantic`/`record_safepoint` — that wiring lands in DSL-0a/b. The R-0 deliverable is the *infrastructure*, not yet the actual increments.

- [ ] **Step 7: Commit**

  ```sh
  git add tools/lyng-js-bench/src/runtime.rs tools/lyng-js-bench/src/v8suite.rs
  git commit -m "R-0: --count-slow-path-share flag and report integration"
  ```

---

### Task 22: Synthetic test that the flag wiring works end-to-end

**Files:**
- Modify: `tools/lyng-js-bench/src/runtime.rs` (or wherever the existing test lives)

- [ ] **Step 1: Write an integration test**

  Add to the existing tests module in `runtime.rs`:
  ```rust
  #[test]
  fn count_slow_path_share_flag_parses_and_initializes_counters() {
      // Parse args; verify count_slow_path_share is set.
      let options = parse_runtime_options(&["--count-slow-path-share".into()]).unwrap();
      assert!(options.count_slow_path_share);
  }
  ```

  (Adjust the function name and option type to match what's actually used in runtime.rs.)

- [ ] **Step 2: Run**

  ```sh
  cargo test -p lyng-js-bench count_slow_path_share_flag
  ```
  Expected: pass.

- [ ] **Step 3: Commit**

  ```sh
  git add tools/lyng-js-bench/src/runtime.rs
  git commit -m "R-0: integration test for --count-slow-path-share flag wiring"
  ```

- [ ] **Step 4: Mark ticket in review**

  ```sh
  dcat update <SLOW_PATH_SHARE_TICKET_ID> --status in_review
  ```

---

## Phase 7 — Evidence reports (Tasks 23-25)

### Task 23: Write `llint-dsl-value-layout.md`

**Files:**
- Create: `reports/js/lyng-js/llint-dsl-value-layout.md`

- [ ] **Step 1: Mark evidence-reports ticket in progress**

  ```sh
  dcat update <EVIDENCE_REPORTS_TICKET_ID> --status in_progress
  ```

- [ ] **Step 2: Read the current Value implementation**

  ```sh
  wc -l crates/lyng-js/types/src/value.rs
  ```
  Then read the file. Capture:
  - Value byte layout (size, alignment, internal `u64` encoding)
  - All tag bit masks (`is_smi`, `is_double`, `is_undefined`, `is_null`, `is_bool`, `is_object`, `is_string`, `is_symbol`)
  - The `not_cell_mask`-equivalent (whatever the code uses to tag a heap reference)
  - SMI encoding: bit positions for payload + tag
  - Double encoding: NaN-tag space offset
  - Sentinel encodings: `array_hole`, `uninitialized_lexical`, `empty_internal_slot`, `deleted_environment_binding`

- [ ] **Step 3: Capture the asm shape of each `check_*!`/`tag_*!` operation today**

  For each operation the DSL will eventually need, identify the equivalent existing Rust call site (e.g., `Value::is_smi` in dispatch_handlers/arithmetic.rs op_add). Use `cargo asm` to capture the codegen.

  ```sh
  cargo run --release -p lyng-js-bench -- asm-diff --mode update \
    --opcodes-config tools/lyng-js-bench/hot-opcodes.toml \
    --baseline reports/js/lyng-js/dsl-asm-baseline-aarch64
  ```
  This populates `reports/js/lyng-js/dsl-asm-baseline-aarch64/Add.asm` etc., which you can reference.

- [ ] **Step 4: Draft the report**

  Create `reports/js/lyng-js/llint-dsl-value-layout.md`. Structure:
  ```markdown
  # Lyng-js Value layout — DSL substrate prerequisite report

  This report documents the current `Value` representation in lyng-js as the
  source of truth for the DSL backend's `check_*!` / `tag_*!` macros. It is
  one of three R-0 evidence reports required before DSL-0 begins.

  ## Source

  Implementation: [crates/lyng-js/types/src/value.rs](../../crates/lyng-js/types/src/value.rs)
  Reference: JSC `Source/JavaScriptCore/runtime/JSCJSValue.h`

  ## Encoding overview

  - Total size: 8 bytes (`#[repr(transparent)] struct Value(u64)`).
  - Tag space: NaN-boxing in the upper 16 bits of the IEEE 754 double NaN
    payload. Tag distinguishes SMI, double, object reference, string,
    symbol, undefined, null, bool, and several internal sentinels.

  (Fill in exact tag bit positions, masks, and encoding rules from
  value.rs:75-345.)

  ## Per-operation asm sequences

  ### `check_smi!(value, fail_label)`

  Expected AArch64 sequence (from current `Value::is_smi` codegen):

  (Insert the asm here from the captured baseline. Note any differences
  from LLInt's equivalent `bqb value, numberTag, slow`.)

  ### `check_object_ref!(value, fail_label)`

  ...

  ### `untag_smi!(value, dst_i32)`

  ...

  (Continue for the full vocabulary listed in §7 of the design.)

  ## Irreducible deltas vs LLInt

  LLInt assumes pointer-identity cells (`*mut JSCell`). Lyng-js uses
  `ObjectRef = u32` with a side-table lookup. Per-operation delta:

  | Operation | LLInt instrs | Lyng instrs | Reason |
  |---|---:|---:|---|
  | `check_cell!` / `check_object_ref!` | 1 | 1 | Tag check is same shape |
  | `load_cell_shape!` / `load_record_shape!` | 1 load | 2 loads (resolve ObjectRef → ObjectRecord ptr) | Side-table indirection |
  | `load_inline_slot!` / `load_record_inline_slot!` | 1 load | 2 loads | Side-table indirection |

  These are the documented deltas DSL-0 accepts. The pointer-identity-cell
  refactor (Phase 2b / DSL-3) would close them; not in DSL-0 scope.

  ## Decision

  DSL-0 uses the current Value layout as-is. The DSL vocabulary names
  reflect this (`check_smi!`, `check_object_ref!`, `load_object_record!`)
  rather than LLInt's pointer-cell names. Pointer-identity cells become
  an evidence-driven later refactor.
  ```

- [ ] **Step 5: Cross-reference the actual code**

  Read `crates/lyng-js/types/src/value.rs` and fill in the exact masks and bit positions. Cite `value.rs:LINE` for each fact.

- [ ] **Step 6: Validate the report's asm snippets compile**

  For any asm snippet quoted in the report, run `cargo asm` on the matching Rust function and confirm the sequence appears. The report is evidence; it must be reproducible.

- [ ] **Step 7: Commit**

  ```sh
  git add reports/js/lyng-js/llint-dsl-value-layout.md
  git commit -m "R-0: llint-dsl-value-layout.md evidence report"
  ```

---

### Task 24: Write `llint-dsl-abi.md`

**Files:**
- Create: `reports/js/lyng-js/llint-dsl-abi.md`

- [ ] **Step 1: Read the relevant design sections**

  Re-read §5 (LlIntState + LlIntRustContext + LlIntRustContextOpaque) and §6 (slow-path bridge protocol, pre-slow-path sync, four-layer state sync rules) of the design.

- [ ] **Step 2: Draft the report**

  Create `reports/js/lyng-js/llint-dsl-abi.md`. Structure:
  ```markdown
  # asm-DSL ABI — DSL substrate prerequisite report

  This report fully specifies the `LlIntState` / `LlIntRustContext` ABI,
  the pinned-register convention, the slow-path return ABI, the exit-slot
  protocol, the pre-slow-path sync protocol, and the four-layer state sync
  rules from §5-§6 of the design. It is one of three R-0 evidence reports
  required before DSL-0 begins.

  ## Source

  Design: [docs/lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md](../../docs/lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md) §5-§6.

  ## `LlIntState` layout

  (Copy from §5, fill in actual offsets computed via `core::mem::offset_of!`
  on a placeholder struct. Verify with a Rust test that generates the
  layout and prints offsets. Commit the test.)

  | Field | Type | Offset | Size |
  |---|---|---:|---:|
  | frame_pc_offset | u32 | 0 | 4 |
  | _pad1 | u32 | 4 | 4 |
  | frame_pb_base | *const u8 | 8 | 8 |
  | frame_regs_base | *mut Value | 16 | 8 |
  | frame_fv_base | *mut FeedbackEntry | 24 | 8 |
  | frame_depth | u32 | 32 | 4 |
  | frame_check_epoch | u32 | 36 | 4 |
  | rust_context | *mut LlIntRustContextOpaque | 40 | 8 |
  | prefix | u8 | 48 | 1 |
  | _pad2 | [u8; 7] | 49 | 7 |

  Total: 56 bytes, 8-byte aligned.

  ## `LlIntRustContext` layout (Rust-only)

  Not `#[repr(C)]`. Fields:
  - `vm: &'vm mut Vm`
  - `agent: &'vm mut Agent`
  - `host: &'vm dyn HostHooks`
  - `registry: &'vm mut (dyn NativeFunctionRegistry + 'vm)`
  - `installed: Arc<InstalledFunction>`
  - `frame: FrameRecord` (full snapshot)
  - `frame_depth: usize`
  - `exit: LlIntExitSlot`

  Accessed only through `LlIntDispatchState::from_raw`'s unsafe cast.

  ## Pinned-register convention

  | Symbol | AArch64 | x86_64 | Type |
  |---|---|---|---|
  | PC | x19 | r12 | *const u8 |
  | REGS | x20 | r13 | *mut Value |
  | FV | x21 | r14 | *mut FeedbackEntry |
  | VM | x22 | r15 | *mut Vm |
  | TABLE | x23 | (RIP-relative) | *const Handler |
  | STATE | x24 | rbx | *mut LlIntState |

  ## Pre-slow-path sync protocol

  (Quote §6 directly: the five-step sequence each shim runs.)

  ## Four-layer state sync rules

  (Quote §5's table.)

  ## Slow-path return ABI

  ```rust
  #[repr(C)]
  pub struct SlowPathReturn { pub tag: u64, pub payload: u64 }

  #[repr(u64)]
  pub enum SlowPathTag {
      Continue = 0,
      Refresh = 1,
      Exit = 2,
  }
  ```

  ## Invariant tests required before DSL-0b

  - `offset_of!(LlIntState, frame_pc_offset)` equals committed const.
  - All field offsets stable across rustc 1.88, 1.89, 1.90 (re-run when a
    new rustc version ships).
  - Miri test: round-trip `LlIntRustContextOpaque` cast does not violate
    aliasing.
  - PC-sync invariant: a synthetic slow-path observes the post-dispatch
    PC, not the entry PC.

  ## Symbol-mangling / no-unwind policy

  All slow paths are `extern "C"` (panic-abort under modern Rust ABI).
  Debug builds wrap shims in `catch_unwind`. Cross-boundary unwinding
  is forbidden.
  ```

- [ ] **Step 3: Compute real offsets via a one-off test**

  Add a temporary unit test under `crates/lyng-js/vm/src/tests/` that builds a placeholder `LlIntState` struct (matching the design's spec exactly) and prints `offset_of!` for each field. Run it, capture the output, and update the table in the report.

  Delete the temporary test after the table is filled in; the real `LlIntState` lands in DSL-0b with proper tests.

- [ ] **Step 4: Commit**

  ```sh
  git add reports/js/lyng-js/llint-dsl-abi.md
  git commit -m "R-0: llint-dsl-abi.md evidence report with computed field offsets"
  ```

---

### Task 25: Write `llint-dsl-safepoints.md`

**Files:**
- Create: `reports/js/lyng-js/llint-dsl-safepoints.md`

- [ ] **Step 1: Read the relevant code paths today**

  Read each:
  - `crates/lyng-js/vm/src/vm/dispatch_handlers/control_flow.rs` (op_loop_header, op_jump, op_jump8)
  - `crates/lyng-js/vm/src/vm/dispatch_handlers/prefix.rs` (op_wide, op_extra_wide)
  - GC poll: search for `poll_incremental_mark_step`
  - Debugger pause: search for `debug_state.should_poll`, `request_debug_pause`
  - Tier-up: read `crates/lyng-js/vm/src/vm/tiering.rs`

- [ ] **Step 2: Draft the report**

  Create `reports/js/lyng-js/llint-dsl-safepoints.md`. Structure:
  ```markdown
  # asm-DSL safepoints, polling, and prefix dispatch — R-0 evidence report

  This report locks in the same-thread polling model, the warm-handler
  safepoint coverage, the prefix dispatch semantics, and the explicit
  deferral of tier accounting from DSL-0. It is one of three R-0
  evidence reports required before DSL-0 begins.

  ## Source

  Design: §6 (safepoints + prefix dispatch), §10 (tier accounting deferral).
  Current code:
  - [crates/lyng-js/vm/src/vm/dispatch_handlers/control_flow.rs](../../crates/lyng-js/vm/src/vm/dispatch_handlers/control_flow.rs) — op_loop_header, op_jump, op_jump8.
  - [crates/lyng-js/vm/src/vm/dispatch_handlers/prefix.rs](../../crates/lyng-js/vm/src/vm/dispatch_handlers/prefix.rs) — op_wide, op_extra_wide.
  - [crates/lyng-js/vm/src/vm/tiering.rs](../../crates/lyng-js/vm/src/vm/tiering.rs) — observe_tier_backedge_event.

  ## Today's safepoint surface

  Today's alpha path polls at:

  - `op_loop_header`: GC step + debugger pause + tier backedge.
  - Negative `op_jump`: tier backedge + (sometimes) GC.
  - Negative `op_jump8`: same.
  - Taken negative conditional jumps: same.

  Cite the relevant control_flow.rs lines.

  ## DSL-0 polling model

  Same-thread `Vm.poll_pending: u8` with two bits:
  - `GC_PENDING (0x01)`
  - `DEBUG_PAUSE (0x02)`

  No `TIER_UP_PENDING`. Tier accounting is deferred from DSL-0 (see
  "Deferred work" below).

  ### Producers (same-thread only)

  Producer | Sets bit | When
  --- | --- | ---
  GC scheduler | `GC_PENDING` | Major collection due or incremental mark needs progress (during slow-path execution)
  Debugger | `DEBUG_PAUSE` | `Vm::request_debug_pause` / `Vm::request_debug_pause_at` called (currently `&mut self`)

  Cross-thread producers are explicitly out of scope. If needed later, see
  design §6 "Cross-thread debugger" note.

  ### Consumers

  Warm handlers' poll slow paths:
  - `op_loop_header_poll_rs` — for `op_loop_header`.
  - `op_jump_poll_rs` — for backward `op_jump`/`op_jump8`.
  - `op_jump_if_*_poll_rs` — for backward conditional jumps.

  Each reads `Vm.poll_pending`, runs the relevant work (GC step, debugger
  pause), clears the consumed bits.

  ## Warm-handler asm shape

  Hot-path poll check (AArch64):
  ```asm
  ldrb w_scratch, [VM, #VM_POLL_PENDING_OFFSET]
  cbz  w_scratch, .no_poll
  bl   {poll_slow_rs}
  ; ... dispatch_after_slow ...
  .no_poll:
  ; ... continue with the warm handler's fast path ...
  ```

  ## Invariant tests required for DSL-0b

  1. **Loop-header poll fires.** A tight `op_add` + `op_loop_header` loop
     with `poll_pending = GC_PENDING` set externally reaches the GC slow
     path within ~K iterations.
  2. **Backward-jump poll fires.** Same with `op_jump`-back and no
     `op_loop_header`.
  3. **Conditional backward-jump poll fires.** Same with a taken
     negative `op_jump_if_true`.

  ## Prefix dispatch semantics

  `op_wide` and `op_extra_wide` are warm handlers (not cold stubs). They:
  1. Read pc[1] (the semantic opcode byte).
  2. Reject doubled prefixes: branch to error if `LlIntState.prefix != 0`.
  3. Set `LlIntState.prefix` to 1 (Wide) or 2 (ExtraWide).
  4. Advance PC by 1 (past the prefix byte).
  5. Tail-dispatch to the semantic handler at the new PC.

  Semantic handlers consume `state.prefix` via their layout decoders
  (auto-generated by `llint_handler!`) — they read it, decode operands at
  the wider width, advance PC past the wider body, and clear
  `state.prefix` to 0 before tail-dispatching.

  ### Prefix invariant tests required for DSL-0b

  4. **Wide-prefixed op_move decodes correctly.** Wide-prefixed `op_move r256, r257`
     reads the right registers.
  5. **ExtraWide-prefixed op_move decodes correctly.** Same with Wide-32.
  6. **Double-prefix raises error.** `op_wide; op_wide; op_move ...` raises
     the expected `VmError::Abrupt(...)` variant.

  ## Deferred work (out of DSL-0 scope)

  - **Tier accounting on backedges.** The existing
    `observe_tier_backedge_event` stays alive on the alpha path through
    DSL-0a/b and deletes with alpha in DSL-0c. After DSL-0c, the
    interpreter has no tier accounting until the JIT track resumes. See
    design §6 and §10 for the rationale.
  - **Cross-thread debugger pause.** Same-thread only in DSL-0. If a
    real cross-thread requirement appears, the design says it gets its
    own ticket addressing hook handoff, pause payload, atomic semantics,
    and memory ordering.
  ```

- [ ] **Step 3: Cite the actual code locations**

  Replace placeholder citations with real `file.rs:LINE` references after reading the matching files.

- [ ] **Step 4: Commit**

  ```sh
  git add reports/js/lyng-js/llint-dsl-safepoints.md
  git commit -m "R-0: llint-dsl-safepoints.md evidence report"
  ```

- [ ] **Step 5: Mark evidence-reports ticket in review**

  ```sh
  dcat update <EVIDENCE_REPORTS_TICKET_ID> --status in_review
  ```

---

## Phase 8 — Verification (Tasks 26-28)

### Task 26: Run determinism verification

**Files:** none modified; produces evidence.

- [ ] **Step 1: Mark verification ticket in progress**

  ```sh
  dcat update <BASELINES_VERIFICATION_TICKET_ID> --status in_progress
  ```

- [ ] **Step 2: Run each subcommand 5 times; verify byte-identical output**

  ```sh
  for i in 1 2 3 4 5; do
    cargo run --release -p lyng-js-bench -- asm-diff --mode check > /tmp/asm-diff-$i.txt
    cargo run --release -p lyng-js-bench -- microbench --samples 3 --iters 100000 > /tmp/microbench-$i.txt
  done
  diff /tmp/asm-diff-1.txt /tmp/asm-diff-2.txt /tmp/asm-diff-3.txt /tmp/asm-diff-4.txt /tmp/asm-diff-5.txt
  ```
  Expected: `diff` empty (asm-diff deterministic). Microbench output will not be byte-identical due to timing jitter — that's expected; verify the *structure* is identical (same opcodes, same column count) by comparing the headers and per-opcode counts.

- [ ] **Step 3: Document determinism evidence**

  Create `reports/js/lyng-js/r0/determinism.md`:
  ```markdown
  # R-0 subcommand determinism verification

  - `asm-diff --mode check`: 5 consecutive runs produce byte-identical
    output. Verified via `diff` (empty output).
  - `microbench`: timing values vary; structure (per-opcode rows, column
    layout) is deterministic. Verified by structural diff after sorting.
  - `capture-llint`: deterministic given a fixed JSC binary + source root;
    rerun produces byte-identical output.
  ```

- [ ] **Step 4: Commit**

  ```sh
  git add reports/js/lyng-js/r0/determinism.md
  git commit -m "R-0: determinism verification evidence"
  ```

---

### Task 27: Run full test suite, confirm Test262 not regressed

**Files:** none modified; produces evidence.

- [ ] **Step 1: Run focused test suites**

  ```sh
  cargo test -p lyng-js-vm -p lyng-js-bytecode -p lyng-js-objects -p lyng-js-tests -p lyng-js-compiler
  ```
  Expected: same pass count as Pre-flight 5.

- [ ] **Step 2: Run Test262 whole-corpus**

  ```sh
  cargo run --release -p lyng-js-test262 -- --report /tmp/r0-test262.md -j 4
  ```
  Expected: pass count ≥ 49722/49729 (per [reports/js/lyng-js/test262.md](../../reports/js/lyng-js/test262.md)).

- [ ] **Step 3: Commit the test262 report**

  ```sh
  cp /tmp/r0-test262.md reports/js/lyng-js/r0/test262-after-r0.md
  git add reports/js/lyng-js/r0/test262-after-r0.md
  git commit -m "R-0: Test262 pass-count evidence (no regression)"
  ```

---

### Task 28: Update architecture.md and final R-0 status report

**Files:**
- Modify: `docs/lyng-js/architecture.md`
- Create: `reports/js/lyng-js/r0/status.md`

- [ ] **Step 1: Add a forward-pointing note to architecture.md**

  Find a sensible location (after "Architecture Constraints") and append:
  ```markdown
  ## Upcoming substrate work

  The dispatch substrate is moving from today's α (extern "C" handlers
  returning `Step`) to an asm-DSL substrate documented in
  [docs/lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md](2026-05-16-asm-dsl-llint-interpreter-design.md).
  R-0 (tooling and evidence reports) is the first milestone; see
  [reports/js/lyng-js/r0/status.md](../../reports/js/lyng-js/r0/status.md)
  for current progress.
  ```

- [ ] **Step 2: Write R-0 status report**

  Create `reports/js/lyng-js/r0/status.md`:
  ```markdown
  # R-0 status

  ## Deliverables

  | Deliverable | Status | Path |
  |---|---|---|
  | `microbench` subcommand | done | tools/lyng-js-bench/src/microbench/ |
  | `asm-diff` subcommand | done | tools/lyng-js-bench/src/asm_diff.rs |
  | `capture-llint` subcommand | done | tools/lyng-js-bench/src/capture_llint.rs |
  | `--count-slow-path-share` infrastructure | done | crates/lyng-js/vm/src/slow_path_counts.rs |
  | hot-opcodes.toml from measured data | done | tools/lyng-js-bench/hot-opcodes.toml |
  | LLInt reference capture | done | reports/js/lyng-js/llint-reference/ |
  | microbench-baseline.md | done | reports/js/lyng-js/microbench-baseline.md |
  | dsl-asm-baseline-aarch64/NORMALIZATION.md | done | reports/js/lyng-js/dsl-asm-baseline-aarch64/ |
  | llint-dsl-value-layout.md | done | reports/js/lyng-js/llint-dsl-value-layout.md |
  | llint-dsl-abi.md | done | reports/js/lyng-js/llint-dsl-abi.md |
  | llint-dsl-safepoints.md | done | reports/js/lyng-js/llint-dsl-safepoints.md |
  | Policy doc updates | done | crates/lyng-js/AGENTS.md, docs/lyng-js/engineering-standards.md |
  | Determinism evidence | done | reports/js/lyng-js/r0/determinism.md |
  | Test262 no-regression | done | reports/js/lyng-js/r0/test262-after-r0.md |

  ## Exit-criterion verification

  1. ✓ All subcommands work end-to-end; deterministic across 5 runs.
  2. ✓ Config + baselines + three evidence reports committed.
  3. ✓ hot-opcodes.toml reflects measured dispatch shares from V8 v7 run.
  4. ✓ Slow-path-share counter mode produces sane (zero) per-opcode
     counts on a Richards run today; non-zero counts will appear once
     DSL handlers populate `record_semantic`/`record_safepoint` calls
     in DSL-0b.

  ## Hand-off to DSL-0a

  Next milestone is DSL-0a (semantic extraction). See design §10.
  The three R-0 evidence reports are the prerequisite reading.
  ```

- [ ] **Step 3: Commit**

  ```sh
  git add docs/lyng-js/architecture.md reports/js/lyng-js/r0/status.md
  git commit -m "R-0: status report + architecture.md forward-pointer to design"
  ```

- [ ] **Step 4: Mark all R-0 tickets in review**

  ```sh
  dcat update <R0_PARENT_ID> --status in_review
  ```

- [ ] **Step 5: Ask the user for final approval before closing**

  Notify the user: "R-0 complete. All deliverables landed; status report at `reports/js/lyng-js/r0/status.md`. May I close the R-0 ticket and its sub-tickets?"

  Wait for explicit approval. Only after the user confirms:
  ```sh
  dcat close <R0_PARENT_ID> --reason "R-0 done; see reports/js/lyng-js/r0/status.md"
  ```
  (Close sub-tickets similarly.)

---

## Self-review checklist

After completing all tasks, verify:

- [ ] All 8 dcat sub-tickets land as `in_review` or `closed`.
- [ ] `cargo build --release -p lyng-js-bench` is clean.
- [ ] `cargo test -p lyng-js-bench` passes.
- [ ] `cargo test -p lyng-js-vm --features opcode-counters` passes.
- [ ] `lyng-js-bench asm-diff --mode check` exits 0.
- [ ] `lyng-js-bench microbench --samples 3 --iters 100000` exits 0.
- [ ] `lyng-js-bench capture-llint --opcodes op_add --source excerpt --jsc-source /Users/sondre/dev/WebKit` exits 0.
- [ ] Test262 pass count ≥ 49722/49729.
- [ ] All three evidence reports exist and cite real `file.rs:LINE` references.
- [ ] `crates/lyng-js/AGENTS.md` line 142 area no longer contains a blanket no-unsafe rule.
- [ ] No new `unsafe` blocks outside the modules explicitly allowed by the updated policy.

If any of these fail, stop and resolve before declaring R-0 done.
