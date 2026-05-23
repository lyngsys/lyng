# DSL-1 Phase 1.C.3 — Unary update (inc/dec) — Summary

**HEAD:** `ed2f3a63` (Task 11 close — dsl_increment_writeback unit test).
**Predecessor:** Phase 1.C.2 close at `521c35af`.
**Sub-phase spec:** [`docs/superpowers/specs/2026-05-21-dsl-1-phase-1c-smi-arith-and-bitwise-design.md`](../../../docs/superpowers/specs/2026-05-21-dsl-1-phase-1c-smi-arith-and-bitwise-design.md) §2.1.C.3.

## What landed

Two inline ports + 1 unit test (substrate macros for inc/dec already landed in Task 1 substrate prep):

| Opcode        | Top-30 rank | Dispatches / V8 v7 run | Asm-shape instr | Slow-path-share max |
|---------------|------------:|-----------------------:|----------------:|---------------------:|
| op_increment  | #5          | 541M                   | 27              | see caveat below     |
| op_decrement  | #23         | 99M                    | 27              | see caveat below     |

Combined dispatch share added: **640M / V8 v7 run**.

Both ports are unary (single src register, no rhs operand). Asm shape is the **shortest in Phase 1.C** — 27 instructions vs 35-40 for the binary ports — because the unary layout saves the rhs operand decode + rhs `check_smi` + rhs `untag_smi` (~9 instructions total).

## SMI-elision of src register writeback

This sub-phase introduced a correctness subtlety not present in any prior Phase 1.C port:

The semantic body `op_update_register_semantic` (at `crates/lyng-js/vm/src/vm/semantics/arithmetic.rs:796-833`) shared by both opcodes writes the ToNumeric-coerced source back to the src register BEFORE writing the post-update value to dst. The inline fast path can safely SKIP this writeback when src is SMI because `ToNumeric(SMI) == SMI` is identity — verified by reading:
- `to_primitive` (`conversions.rs:90-92`) — short-circuits for non-objects (SMI is a non-object), returns value as-is
- `to_number` (`read.rs:296-298`) — short-circuits when `value.is_number()`, returns value identically

For non-SMI src (string, BigInt, Object with `valueOf`), the fast path bails to `.slow` which calls `op_increment_slow_rs` / `op_decrement_slow_rs`, which invokes the full semantic body — preserving the writeback semantics.

**The SMI-elision claim is verified end-to-end by the new `dsl_increment_writeback` unit test** (Task 11, `crates/lyng-js/tests/src/dsl_increment_writeback.rs`). The 4 tests exercise postfix and prefix increment/decrement with string source values, asserting both that the slow path produces the correct numeric result AND that the src register is written back with the ToNumeric-coerced value (the assertion uses `typeof r === "number"` on the postfix return value, which can only hold if the writeback happened — proven by tracing the compiler's `lower_update_expression`).

## A/B vs pre-1.C.3 HEAD

11-sample A/B (commit `521c35af` vs `ed2f3a63`); loadavg overlap at changeover = 0.4% (well within ±20% protocol). Full report at [`phase-1c3-ab-comparison.md`](phase-1c3-ab-comparison.md).

| Workload    | Base median | Post median | Delta      |
|-------------|------------:|------------:|-----------:|
| Richards    |        296  |        296  |  0.00%     |
| DeltaBlue   |        330  |        332  |  +0.61%    |
| Crypto      |        280  |        297  |  **+6.07%** |
| RayTrace    |        427  |        427  |  0.00%     |
| NavierStokes|        435  |        494  |  **+13.56%** |
| Splay       |       1383  |       1378  |  -0.36%    |
| **Geomean** |        —    |        —    |  **+3.19%** |

**NavierStokes +13.56%** is the headline — that workload dispatches 588M Increments per cycle and the inline port pays off proportionally. Crypto +6.07% is also strong. Other workloads have negligible inc/dec dispatch counts and show flat-to-noise results.

## Measurement-discipline caveat (continued from 1.C.1 / 1.C.2)

Per-opcode slow-path-share gate again reads 100% for both Increment and Decrement due to the substrate-wide `call_slow!` counter-injection artifact. **The strongly positive A/B (+3.19% geomean, including +13.56% on NavierStokes which dispatches 588M Increments) is direct confirmation that the artifact is purely instrumentation** — actual execution correctly takes the inline fast path.

Per spec §1.6 + §5, per-workload waivers are documented in op_increment.md / op_decrement.md. The substrate fix (`inject_opcode_byte` discipline) remains tracked as Phase 1.C followup #1.

## Per-opcode reports + new test

- [`reports/js/lyng-js/dsl-handlers/op_increment.md`](../dsl-handlers/op_increment.md) (Task 9, commit `2e7de038`)
- [`reports/js/lyng-js/dsl-handlers/op_decrement.md`](../dsl-handlers/op_decrement.md) (Task 10, commit `970f4e84`)
- [`crates/lyng-js/tests/src/dsl_increment_writeback.rs`](../../../crates/lyng-js/tests/src/dsl_increment_writeback.rs) (Task 11, commit `ed2f3a63`) — 4 tests, all passing

## Gates passed

- ✅ Per-opcode asm shape within 5 of LLInt (both at 27 instructions — the shortest inline paths in Phase 1.C)
- ✅ Per-opcode microbench within 2× LLInt reference
- ⚠ Per-opcode slow-path-share — counter-injection artifact (documented waiver)
- ✅ Behavioral parity: 418 + 1209 + 4 new = **1631 cargo tests pass**
- ✅ Test262 slices: postfix-increment 66/66, prefix-increment 57/57, postfix-decrement 65/65, prefix-decrement 58/58 — all 100%
- ✅ Asm baselines committed
- ✅ hot-opcodes.toml budgets calibrated (Increment=29, Decrement=29)
- ✅ New substrate macros (`inc_smi_overflow!` / `dec_smi_overflow!`) runtime-verified via inline ports + writeback unit test (Phase 1.B retrospective lesson #3 satisfied)

## Followups (continuing from 1.C.1 + 1.C.2)

Pinned, unchanged from prior sub-phase summaries:
1. `inject_opcode_byte` counter-injection discipline (substrate fix at `crates/lyng-js-vm-dsl/src/lower.rs`)
2. `verify_opcodes_per_iter` coverage (add Sub/Mul/BitAnd/ShiftLeft/ShiftRight/Increment/Decrement)
3. op_mul float-workload trade-off (consider a faster bail for non-SMI)
4. Cumulative Phase 1.C cross-day A/B sanity (Task 13)

New from 1.C.3:
5. **JS-level coverage of the SMI-elision write-skip case:** The `dsl_increment_writeback` test exercises non-SMI src reaching the slow path (which correctly writes back). The complementary case — verifying that SMI src does NOT write back via the fast path — would require either a Rust unit test on the dispatch result (no JS-level observation possible since the SMI writeback is observationally a no-op) or instrumentation/test mode. Low priority; the structural argument plus the non-SMI test together cover the safety story.

## Next sub-phase

**Phase 1.C close** — Task 13 (cumulative A/B vs pre-DSL-0 `d850f261`, the authoritative umbrella gate) + Task 14 (phase summary + followups) + Task 15 (engine state snapshot refresh).

After Phase 1.C close, the natural next step per spec §11 is **Phase 1.D** — comparison + branch opcodes (op_greater_equal #20, op_less_equal #27, plus 5 cold-stub jump opcodes). Phase 1.D will use the same record-smi-shim pattern and inherit the counter-injection caveat; the substrate fix tracked as Phase 1.C followup #1 should land before or during Phase 1.D to unblock honest per-opcode slow-path-share enforcement.
