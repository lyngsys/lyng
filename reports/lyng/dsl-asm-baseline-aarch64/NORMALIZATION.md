# asm-diff normalization rules

This document is the single source of truth for the normalization
rules applied by `lyng-bench asm-diff` before comparing handler
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
rustc, run `lyng-bench asm-diff --mode update` to refresh
baselines; commit message must include `[asm-baseline-refresh: rustc <old>→<new>]`.
