# Design: DSL-1 Phase 1.B.2 — Backfill inline ports

**Date:** 2026-05-19
**Status:** Design draft; awaiting user review.
**Parent spec:** [`2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md`](2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md) — Phase 1.B umbrella.
**Predecessor:** Phase 1.B.1 closed at HEAD `68dd5e89` ([`reports/js/lyng-js/dsl-1/phase-1b1-summary.md`](../../../reports/js/lyng-js/dsl-1/phase-1b1-summary.md)).
**Deferral inputs:**
- [`reports/js/lyng-js/dsl-1/phase-1a-load-const8-deferred.md`](../../../reports/js/lyng-js/dsl-1/phase-1a-load-const8-deferred.md)
- [`reports/js/lyng-js/dsl-1/phase-1a-load-this-deferred.md`](../../../reports/js/lyng-js/dsl-1/phase-1a-load-this-deferred.md)

---

## 1. Goal, scope, exit criteria

### Goal

Inline-port `op_load_const8` (#21, ~104M dispatches/V8 v7 run) and `op_load_this` (#12, ~256M dispatches/V8 v7 run) using the frame-context substrate landed in Phase 1.B.1. These are the two opcodes deferred from Phase 1.A pending the substrate.

### In scope

- **`op_load_const8_dsl` inline port** (opcode 140, layout Ab, length 3). Replaces the current `call_slow!` shim with a 4-instruction inline load via `load_constant!`.
- **`op_load_this_dsl` inline port** (opcode 28, layout Abx, length 4). Replaces the current `call_slow!` shim with a 6-instruction inline load + sentinel-bail via `load_state_value!` + inline `cmp`+`b.eq` against `Value::uninitialized_lexical()`. Slow path retained for the bail (re-uses existing `op_load_this_slow_rs`).
- **Per-opcode ported reports** at `reports/js/lyng-js/dsl-handlers/op_load_const8.md` and `reports/js/lyng-js/dsl-handlers/op_load_this.md`.
- **Asm baselines** captured via `lyng-js-bench asm-diff`.
- **Microbench snippets verification**: Phase 1.B.0 added `LoadConst8` and `LoadThis` snippets to `tools/lyng-js-bench/src/microbench/snippets.rs`. Phase 1.B.2 verifies these now report ns/dispatch within 2× LLInt reference.
- **Slow-path-share gate**: confirm < 20% on V8 v7 via Phase 1.B.0's `--count-slow-path-share` infra.
- **Clean up `dsl_validation_frame_context.rs`**: the 3 `#[ignore]`-d forward-pointer tests added in Phase 1.B.1 are removed — replaced by per-opcode integration tests through normal JS evaluation (which now exercise the canonical opcodes).
- **Same-load V8 v7 A/B** vs `68dd5e89` (Phase 1.B.1 closed HEAD).

### Out of scope

- Phase 1.B.3 opcode ports (locals, Ldar, LoadEnvSlot) — separate sub-phase.
- New backend macros — the 1.B.1 substrate `load_constant!` and `load_state_value!` cover both ports.
- New `LlIntState` fields — substrate is complete.
- IC mode-byte refactor (Phase 1.F).

### Strict selection rule

Both opcodes are top-30: `op_load_this` is #12 (256M dispatches/run), `op_load_const8` is #21 (104M dispatches/run). Both well above the strict top-30 + macro-shared-pair bar from Phase 1.B's parent spec §3.

### Exit criteria

1. **Behavioral parity.** `cargo test -p lyng-js-vm --lib --release` ≥ 417 passing; `cargo test -p lyng-js-tests --release` ≥ 1187 passing. 2 pre-existing `feedback_flat_consistency` failures unchanged.
2. **Per-opcode gates** for each of `op_load_const8` + `op_load_this`:
   - ≤ 12 inline instructions (asm baseline)
   - Microbench within 2× LLInt reference
   - Slow-path-share < 20% on V8 v7
   - Per-handler ported report present
   - Asm baseline passes `asm-diff --check`
3. **Same-load A/B vs `68dd5e89`**: aggregate V8 v7 regression ≤ 2%; per-workload regression ≤ 5%; expected improvement ≥ +0.3% on V8 v7 cumulative (LoadThis is the largest single port in 1.B.2).
4. **Sub-phase summary** at `reports/js/lyng-js/dsl-1/phase-1b2-summary.md`.

---

## 2. Background: substrate inventory

Phase 1.B.1 landed the substrate. Recap what's available:

| Substrate piece | Location | Status |
|------------------|----------|--------|
| `frame_const_base: *const Value` field on `LlIntState` (offset 32) | `crates/lyng-js/vm/src/dsl/llint_state.rs:31` | ✅ |
| `frame_this_value: Value` field on `LlIntState` (offset 40) | `crates/lyng-js/vm/src/dsl/llint_state.rs:32` | ✅ |
| `LLINT_STATE_FRAME_CONST_BASE` + `LLINT_STATE_FRAME_THIS_VALUE` consts | `crates/lyng-js/vm/src/dsl/reg_convention.rs:43-44` | ✅ |
| `load_constant!` macro (2-instruction indexed load) | `crates/lyng-js/vm/src/dsl/backend/aarch64/constants.rs` | ✅ |
| `load_state_value!` macro (1-instruction fixed-offset load) | `crates/lyng-js/vm/src/dsl/backend/aarch64/frame.rs` | ✅ |
| `resolve_initial_this_value` helper (populates `frame_this_value`) | `crates/lyng-js/vm/src/dsl/llint_state.rs` | ✅ |
| Population at trampoline entry (`run_via_dsl`) | `crates/lyng-js/vm/src/dsl/entry.rs` | ✅ |
| Refresh in Refresh arm (`translate_outcome`) | `crates/lyng-js/vm/src/dsl/slow_path.rs:293-312` | ✅ |
| Microbench snippets for LoadConst8 + LoadThis | `tools/lyng-js-bench/src/microbench/snippets.rs` | ✅ |
| Sentinel `Value::uninitialized_lexical()` const | `crates/lyng-js/types/src/value.rs:186` | ✅ |
| Slow-path stubs for both opcodes (kept as bail targets for op_load_this) | `crates/lyng-js/vm/src/dsl/handlers/cold.rs:879-902, 4184-4210` | ✅ |

Phase 1.B.2 just consumes the substrate; no substrate additions.

---

## 3. Inline-port designs

### 3.1 `op_load_const8_dsl` (opcode 140, layout Ab, length 3)

**Current (cold stub):**
```rust
llint_handler! {
    op_load_const8_dsl, opcode_byte = 140, layout = Ab, length = 3, |a, b| {
        call_slow!(op_load_const8_slow_rs, args = [a, b]);
        dispatch_after_slow!();
    }
}
```

**Target (inline):**
```rust
llint_handler! {
    op_load_const8_dsl, opcode_byte = 140, layout = Ab, length = 3, |a, b| {
        // b = constant pool index (u8), a = dest register
        load_constant!(b => x10);    // 2 instructions: ldr base + ldr value[b]
        store_reg!(a, x10);          // 1 instruction
        dispatch!();                 // 4 instructions (standard tail)
    }
}
```

**Asm shape (~7 instructions inline + 4 dispatch tail):**
```asm
; decode prologue (a, b already in registers by lowerer)
ldr  x16, [x22, #LLINT_STATE_FRAME_CONST_BASE]   ; load base ptr (1)
ldr  x10, [x16, x_b, lsl #3]                     ; load Value at b (2)
str  x10, [x20, x_a, lsl #3]                     ; store to dest reg (3)
; dispatch tail (4 instructions)
```

**vs LLInt reference** (estimate ~6-8 instructions per LLInt operand-decode + read-from-flat-constants + register-store): well within 2× target.

**Slow-path-share expectation:** ~0%. The inline path handles all cases that the pre-resolved `frame_const_base` covers (Smi, Float64, Atom, Builtin — all resolved at install time per Phase 1.B.1 §2.1). No bail conditions in the inline path.

### 3.2 `op_load_this_dsl` (opcode 28, layout Abx, length 4)

**Current (cold stub):**
```rust
llint_handler! {
    op_load_this_dsl, opcode_byte = 28, layout = Abx, length = 4, |a, bx| {
        call_slow!(op_load_this_slow_rs, args = [a, bx]);
        dispatch_after_slow!();
    }
}
```

**Target (inline + sentinel bail):**
```rust
llint_handler! {
    op_load_this_dsl, opcode_byte = 28, layout = Abx, length = 4, |a, _bx| {
        // a = dest register; bx unused (layout consistency)
        load_state_value!(LLINT_STATE_FRAME_THIS_VALUE => x10);  // 1 instruction: ldr
        // Sentinel compare inline. The sentinel is a 64-bit constant
        // (Value::uninitialized_lexical().bits()); materialize via
        // either literal pool or movz/movk and compare.
        load_uninit_lex_sentinel!(x16);  // 1-2 instructions
        cmp_value!(x10, x16);            // 1 instruction
        bail_to_slow_on_eq!(op_load_this_slow_rs, args = [a, _bx]);  // 1 b.eq + slow-path tail
        store_reg!(a, x10);              // 1 instruction
        dispatch!();                     // 4 instructions
    }
}
```

**Asm shape (~8-9 instructions inline + 4 dispatch tail; total ~12-13):**
```asm
ldr  x10, [x22, #LLINT_STATE_FRAME_THIS_VALUE]   ; load this mirror (1)
ldr  x16, =UNINIT_LEX_BITS                       ; load sentinel const (2; literal pool entry)
cmp  x10, x16                                    ; compare (3)
b.eq L_slow                                       ; bail if sentinel (4)
str  x10, [x20, x_a, lsl #3]                     ; store to dest reg (5)
; dispatch tail (4 instructions: 6-9)
L_slow:
; call_slow + dispatch_after_slow tail (handled by existing primitive)
```

**On the sentinel materialization:** `Value::uninitialized_lexical()` returns a `Value` with `InternalSentinel::UninitializedLexical` discriminant. The exact u64 bit pattern is a compile-time constant. AArch64 has two options:
- `ldr x16, =literal` — assembler allocates a literal pool entry; load is one ldr at the call site. Pros: always 1 instruction. Cons: requires literal pool support in `naked_asm!`, which may or may not be available.
- `movz x16, #imm16; movk x16, #imm16, lsl #16; ...` — up to 4 instructions to materialize a full 64-bit constant. Pros: no literal pool dependency. Cons: more instructions if all bits are set.

**Decision:** prefer `ldr {dst}, =literal` (1 instruction, assembler-managed literal pool) if `naked_asm!` accepts it; otherwise fall back to `movz {dst}, #imm16; movk {dst}, #imm16, lsl #16` (up to 4 instructions for a full 64-bit constant). The Phase 1.B.0 counter macros use `ldr` against a fixed offset successfully; literal-pool `ldr` is the natural extension. The refactor worker tries `ldr =literal` first; if rejected by the rustc inline-asm parser, switches to `movz/movk` and notes the deviation in the commit message. Either way, total inline budget stays ≤ 12 instructions.

**Backend macro decision:** introduce a new tiny macro `load_uninit_lex_sentinel!($dst)` in `crates/lyng-js/vm/src/dsl/backend/aarch64/prelude.rs` (or `values.rs` — wherever Value-related constants live). This keeps the handler body clean and centralizes the sentinel materialization so future opcodes (e.g., `op_throw_uninitialized`) can reuse it. **One new backend macro is OK in 1.B.2** — it's a tiny utility (single ldr/movz sequence) tied directly to the in-scope opcodes.

**Slow-path-share expectation:** very low (< 5%) on V8 v7. The sentinel fires only for `ThisState::Uninitialized` (TDZ for derived constructors before `super()`) or `ThisState::Lexical` (arrow functions captured from a lexical environment). V8 v7 workloads (Richards, DeltaBlue, etc.) are written in pre-class style; they rarely hit either case.

### 3.3 `cmp_value!` and `bail_to_slow_on_eq!` macros

These are illustrative names in §3.2's pseudo-asm; the actual implementation can use plain inline `cmp` + `b.eq` followed by the existing `call_slow!` + `dispatch_after_slow!` primitives. No new backend macro needed beyond `load_uninit_lex_sentinel!`. The handler body inlines the compare + branch directly.

---

## 4. Cleanup: `dsl_validation_frame_context.rs`

Phase 1.B.1 added 3 `#[ignore]`-d forward-pointer tests citing "Phase 1.B.2 lands the canonical opcodes" as the unblocker. Now that Phase 1.B.2 lands them, the ignored tests become obsolete because the 3 canonical opcodes are exercised by:
- Existing integration tests in `lyng-js-tests` that compile JS programs with `42` literals, `this` in closures, etc.
- A new per-opcode integration test per the per-opcode-gate convention.

**Decision: DELETE the 3 ignored tests.** Keep the 3 structural "compiles-and-links" tests (they validate backend macro syntax + asm string formation, useful for catching macro regressions before they hit production handlers).

The fewer ignored tests in the codebase, the better — ignored tests bit-rot silently.

---

## 5. Per-opcode test plan

Per the per-opcode-gate convention (parent spec §4), each port gets:

### `op_load_const8`

| Test | Location | Asserts |
|------|----------|---------|
| Smi constant load | `lyng-js-tests/...` (new integration test) | `vm.evaluate_script("42")` returns Smi 42 (forces op_load_const8 for the integer literal) |
| Float constant load | same file | `vm.evaluate_script("3.14")` returns f64 3.14 |
| Atom constant load | same file | `vm.evaluate_script("'hello'")` returns string "hello" (atom pre-resolved at install time) |
| Builtin constant load | same file | A builtin reference test (e.g., `Math.PI` if that triggers a Builtin constant) |

Existing integration tests in `lyng-js-tests` likely cover most of these via parser/compiler tests. **Required minimum:** at least one new explicit integration test per opcode that asserts the inline-handler path produces the right result for each in-scope case (Smi, Float, Atom, Builtin for op_load_const8; Value, Uninitialized-bail, Lexical-bail for op_load_this). The refactor worker may dedupe against existing coverage if specific cases already have explicit assertions.

### `op_load_this`

| Test | Location | Asserts |
|------|----------|---------|
| Real `this` binding (ThisState::Value) | new integration test | `(function() { return this; }).call({x: 42})` returns `{x: 42}` |
| Sentinel bail: Uninitialized (derived ctor pre-super()) | new integration test | `class D extends class { } { constructor() { try { this; return 'ok'; } catch (e) { return 'threw'; } super(); } }; new D()` returns "threw" |
| Sentinel bail: Lexical (arrow function) | new integration test | `(function() { return (() => this)(); }).call({y: 7})` returns `{y: 7}` (arrow captures outer this) |

Again, much of this is already covered by existing JS-test infrastructure. Refactor worker dedupes.

### Microbench

Phase 1.B.0 added `LoadConst8` and `LoadThis` snippets. Run `cargo run --release -p lyng-js-bench -- microbench --samples 7 --json /tmp/phase-1b2-microbench.json` and verify each is within 2× LLInt reference. The Phase 1.B.0 microbench tables show pre-port (cold-stub) ns/dispatch for these snippets; the post-port number should be lower (we're replacing a slow-path call with inline asm). LLInt reference numbers come from the existing `tools/lyng-js-bench/hot-opcodes.toml` per-opcode configuration if listed there, otherwise from the parent design's reference table.

### Slow-path-share

Run `cargo run --release -p lyng-js-bench -- v8suite --count-slow-path-share`. Both opcodes should show < 20% slow-path-share. For op_load_const8, expected ≈ 0%; for op_load_this, expected < 5%.

---

## 6. Sub-phase phasing

Single refactor worker, ~1-2 days wall-clock. Task breakdown (5 tasks):

1. **Task 1: Add `load_uninit_lex_sentinel!` backend macro.** Tiny — single ldr or movz/movk sequence. Unit test verifies it emits valid asm. ~30 min.
2. **Task 2: Inline-port `op_load_const8`.** Replace cold stub with inline body. Run integration tests. Capture asm baseline + write ported report. ~1-2 hours.
3. **Task 3: Inline-port `op_load_this`.** Replace cold stub with inline + sentinel bail. Run integration tests covering the three ThisState arms. Capture asm baseline + write ported report. ~2-3 hours.
4. **Task 4: Cleanup + V8 v7 A/B.** Delete the 3 ignored tests in `dsl_validation_frame_context.rs`. Run same-load A/B vs `68dd5e89`. Write A/B comparison + microbench summary. ~1 hour bench time.
5. **Task 5: Sub-phase summary.** Write `reports/js/lyng-js/dsl-1/phase-1b2-summary.md` mirroring 1.B.0/1.B.1 format. ~30 min.

Each task is one commit. Behavioral parity at every commit (≥417 + ≥1187).

---

## 7. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------:|-------:|-----------|
| Sentinel materialization needs > 2 instructions, blows the ≤ 12 budget | low | medium | If movz/movk needs all 4 quarters (8 instructions), refactor: use a small in-VM "sentinel constants" struct that the asm can address via a pinned register. Investigate at impl time; budget room exists (current target ~9 instructions for op_load_this leaves 3 instructions of headroom). |
| Inline `op_load_const8` doesn't beat the slow-path call (microbench regression) | low | medium | The slow-path call alone is 4+ instructions (call + 2 args + return). Inline at 4 instructions is a strict win. |
| `frame_this_value` mirror is stale for some path we didn't anticipate | low | high | Phase 1.B.1's reviewer dispatch verified the super() path. The 3-arm integration test in §5 will catch any divergence. The sentinel design means a stale mirror in the most-likely failure case (Uninitialized not refreshed) just slowpaths — fail-safe. |
| Slow-path-share > 20% for op_load_this due to lexical-this in some real workload | medium | medium | This is a real risk for non-V8 workloads (e.g., Test262 arrow-heavy tests). If the gate fails on V8 v7, we PASS the sub-phase but document the lexical-this share for Phase 1.F (which will inline-handle Lexical via a fast-path read of the lexical env). |
| Asm-baseline drift due to rustc inline-asm formatting changes | low | low | Same risk as Phase 1.A; the `asm-diff --check` snapshot is regenerated when accepted. |
| Same-load A/B shows < +0.3% improvement (less than expected) | low | low | The opcodes are top-12 and top-21; combined dispatch share is ~360M/V8v7 run. Even modest per-dispatch wins should aggregate to measurable improvement. If actual < +0.3%, document and proceed; the per-opcode gates are the primary correctness signal. |

---

## 8. Decisions made

1. **Sentinel-compare lives inline in the `op_load_this` handler body**, not in a `bail_if_sentinel!` macro. The compare is only used by `op_load_this`; abstracting it adds indirection without payoff (YAGNI).

2. **One new backend macro: `load_uninit_lex_sentinel!`.** This is a single-use utility that lives in the aarch64 backend module. If future opcodes need other sentinel comparisons, a generic `load_sentinel!(InternalSentinel::X)` macro can be derived later — but not in 1.B.2.

3. **The 3 ignored tests in `dsl_validation_frame_context.rs` are DELETED, not unignored.** They were forward-pointer placeholders; integration tests through normal JS evaluation supersede them. Keep the 3 structural "compiles-and-links" tests.

4. **`asm-baseline` capture happens AFTER the inline port lands** (Tasks 2 and 3), not before. Phase 1.A precedent.

5. **No new `Vm` field, no new install-time work, no new GC plumbing.** All substrate complete from Phase 1.B.1.

6. **`op_load_this`'s `bx` operand stays unused** (matches the layout Abx convention used by other opcodes like `op_load_zero`). Future feedback-instrumentation extensions can populate `bx` (e.g., for IC site tracking) without changing the inline handler.

---

## 9. References

- **Parent design:** [`docs/lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md`](../../lyng-js/2026-05-16-asm-dsl-llint-interpreter-design.md) §10 DSL-1.
- **Phase 1.B umbrella:** [`2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md`](2026-05-18-dsl-1-phase-1b-locals-and-frame-context-design.md).
- **Phase 1.B.1 closure:** [`reports/js/lyng-js/dsl-1/phase-1b1-summary.md`](../../../reports/js/lyng-js/dsl-1/phase-1b1-summary.md).
- **Phase 1.B.1 spec (substrate design):** [`2026-05-19-dsl-1-phase-1b1-frame-context-refactor-design.md`](2026-05-19-dsl-1-phase-1b1-frame-context-refactor-design.md).
- **Top-30 reference:** [`reports/js/lyng-js/r0/v8-v7-top30.tsv`](../../../reports/js/lyng-js/r0/v8-v7-top30.tsv) (op_load_this is #12 at 256M dispatches; op_load_const8 is #21 at 104M dispatches).
- **Phase 1.A analog port (template):** `op_load_smi8_dsl` at `crates/lyng-js/vm/src/dsl/handlers/cold.rs:4171-4176`.
- **Existing slow-path stubs:** `crates/lyng-js/vm/src/dsl/handlers/cold.rs:879-902` (op_load_this), `4184-4210` (op_load_const8).
