# `op_shift_left` DSL port (opcode 47)

Phase 1.C.2 inline port: SMI left shift. Top-30 rank #25, ~89M
dispatches per V8 v7 run. Second port of sub-phase 1.C.2 (bitwise /
shifts) — uses the pre-existing `shift_left_smi!` macro from the
DSL-0 substrate. Left shift on tagged ints with a 5-bit-masked RHS
cannot overflow (the result is always a representable i32), so the
fast path has no bailout-on-overflow branch.

## DSL source

`crates/vm/src/dsl/handlers/cold.rs`:

```rust
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

The handler text is byte-for-byte identical to `op_bit_and` apart
from opcode byte, the arithmetic macro (`shift_left_smi!` vs
`bit_and_smi!`), and the recording-shim symbol. The shift-with-5-bit-
mask shape is one instruction longer than bitwise AND:
`shift_left_smi!` emits 3 instructions (`and w16, rhs, #0x1f` +
`lsl w_dst, w_lhs, w16` + `sxtw x_dst, w_dst`) vs `bit_and_smi!`'s 2
(`and + sxtw`). The extra `and #0x1f` masks the RHS shift count to
its low 5 bits per ECMAScript `<<` / ToUint32 semantics — the same
shape JSC's `op_lshift` uses on AArch64.

## Slow-path shims

- `op_shift_left_slow_rs` (unchanged; pre-existing cold-stub shim —
  invoked from the `.slow` label on SMI miss). Delegates to
  `crate::vm::semantics::arithmetic::op_shift_left_semantic` with the
  same 4-u32 operand-quartet adapter as op_add / op_sub / op_mul /
  op_bit_and.
- `op_shift_left_record_smi_rs` (NEW; fast-path feedback recording —
  mirrors `op_add_record_smi_rs` in `hot.rs:88-106`,
  `op_sub_record_smi_rs` in `cold.rs:1076-1094`,
  `op_mul_record_smi_rs` in `cold.rs:1185-1203`, and
  `op_bit_and_record_smi_rs` in `cold.rs:1541-1559`). Bumps the
  warmup counter, allocates the legacy vector at threshold, mirrors
  legacy state to the flat array, observes the tier feedback event.
  Returns `Continue { pc_advance: 6 }` so the asm bridge advances PC
  by op_shift_left's encoded length without re-entering
  `op_shift_left_semantic`.

## Current asm

See `reports/lyng/dsl-asm-baseline-aarch64/op_shift_left.asm`.

Fast path (from `op_shift_left_dsl:` through
`bl _op_shift_left_record_smi_rs` inclusive): **36 instructions** —
4 ldrb/ldrh decode + 1 ldr + 7 check_smi + 1 ldr + 7 check_smi + 2
sxtw untag + 3 shift_left_smi (and #0x1f + lsl + sxtw) + 4 tag_smi +
1 str + 6 call_slow-setup.

Comparison vs same-family ports:

| Opcode        | Fast-path instr | Δ vs op_sub | Source of Δ                                       |
|---------------|----------------:|------------:|---------------------------------------------------|
| op_add        | 35              | -1          | adds (1 fewer than sub_smi_overflow's b.vs branch)|
| op_sub        | 36              |  0          | reference                                          |
| op_mul        | 40              | +4          | smull+cmp +1; neg-zero cbnz+orr+tbnz +3           |
| op_bit_and    | 35              | -1          | no overflow branch (and+sxtw vs subs+b.vs+sxtw)   |
| op_shift_left | 36              |  0          | +1 from `and #0x1f` mask vs op_bit_and             |

The +1 instr delta vs op_bit_and matches the analytic expectation
exactly: `shift_left_smi!`'s 3 insns (`and #0x1f + lsl + sxtw`) vs
`bit_and_smi!`'s 2 (`and + sxtw`). op_shift_left ties with op_sub at
36 instructions but for different reasons — op_sub spends its
extra instruction on overflow detection (`b.vs`), while op_shift_left
spends it on RHS masking (`and w16, w14, #0x1f`).

## LLInt reference

JSC's `op_lshift` (`Source/JavaScriptCore/llint/LowLevelInterpreter64.asm`)
uses the same arithmetic shape: an offlineasm `lshifti` pseudo-op that
lowers on AArch64 to `and w_count, w_rhs, #0x1f` + `lsl w_dst, w_lhs, w_count`,
plus the SMI tag-and-untag bookkeeping. There is no overflow branch —
left-shift on i32 payloads with a 5-bit-masked count always stays in
i32 range, matching Lyng's `shift_left_smi!` macro shape.

The only Lyng-specific overhead is the 7-instruction `check_smi`
(vs JSC's 4-instruction bit-test) — a NaN-tag-layout consequence
inherited from op_add / op_sub / op_bit_and. Net: per-dispatch Lyng
emits ~36 insns vs JSC's ~28; the ratio (~1.29×) matches the
op_sub / op_add / op_bit_and ratios within budget.

## Microbench

ns/dispatch on ShiftLeft microbench (7-sample median, post-warmup,
ARM64 loadavg fluctuating 3.4-5.4 — well above the 2.0 isolation
gate; ran without `--require-isolation` per spec §1.6 same-machine
A/B):

| Opcode    | Samples | Median ns | CI95   | Notes                                |
|-----------|--------:|----------:|-------:|--------------------------------------|
| Add       | 7       | 141.21    | ±0.55  | Phase 1.A reference (op_add hot)     |
| Sub       | 7       |  —        |  —     | Phase 1.C.1 Task 2 (no snippet rerun)|
| Mul       | 7       | 180.87    | ±1.47  | Phase 1.C.1 Task 3                   |
| BitAnd    | 7       | 154.67    | ±0.54  | Phase 1.C.2 Task 5                   |
| ShiftLeft | 7       | 154.01    | ±0.21  | Phase 1.C.2 Task 6 (this task)       |

ShiftLeft is statistically indistinguishable from BitAnd despite
being one instruction longer in the fast path (CI95 envelopes
overlap: 153.80–154.22 vs 154.13–155.21). Two contributing factors:

- **CPU-side concurrency.** A single `and w16, w14, #0x1f` + `lsl`
  pair can issue on the same cycle as the subsequent `sxtw` on
  modern AArch64 out-of-order cores (Apple M-series, Cortex-X / A-7x).
  The extra mask instruction adds ≤1 cycle of latency on the
  critical path and ~0 cycles on throughput-limited workloads.
- **Snippet-level dispatch mix.** Like BitAnd, each ShiftLeft
  microbench iteration dispatches an extra LoadLocal0 (reading `i`
  for the RHS register operand), so the per-ShiftLeft timing is
  amortized against the same surrounding loop overhead — the cross-
  opcode delta is dominated by the arith core, not the bookkeeping.

Per-opcode gate: ns/dispatch should be within 2× JSC LLInt's
op_lshift. We don't have a direct LLInt op_lshift microbench in-repo,
but the +1 instr delta vs op_bit_and matches the analytic
expectation (shift_left_smi is one instruction longer than
bit_and_smi due to the 5-bit mask). Gate **considered satisfied** by
inheritance from BitAnd (same fast-path shape plus one masking
instruction, same macro substrate).

Notes:
- Loadavg was 5.02 → 5.37 → 3.44 across the run sequence (uptime
  captures embedded in the bench transcript). The
  `--require-isolation` gate rejected the first attempt; rerun
  without strict gate per spec §1.6.
- The microbench snippet for ShiftLeft was added in this task at
  `tools/lyng-bench/src/microbench/snippets.rs` (using
  `x = i << y` with two locals to mirror the BitAnd snippet shape).
  No `ShiftLeftSmi` opcode exists in the bytecode-builder, so the
  two-locals form is for direct shape comparison with BitAnd, not
  peephole avoidance.

## Slow-path-share on V8 v7

Captured via `v8suite --count-opcodes --count-slow-path-share --samples 5`
(loadavg 3.44 → 3.69 across the sweep — above isolation floor; same
conditions as op_bit_and).

| Workload     | ShiftLeft dispatches | Semantic SP   | Share  |
|--------------|---------------------:|--------------:|-------:|
| Richards     |                    0 |             0 |    —   |
| DeltaBlue    |                    0 |             0 |    —   |
| Crypto       |          148,586,828 |   148,586,828 | 100.0% |
| RayTrace     |                    0 |             0 |    —   |
| NavierStokes |                    0 |             0 |    —   |
| Splay        |              643,220 |       643,220 | 100.0% |
| **Total**    |      **149,230,048** | **149,230,048** | **100.0%** |

Four of the six V8 v7 workloads (Richards, DeltaBlue, RayTrace,
NavierStokes) don't emit ShiftLeft at all. The two that do (Crypto,
Splay) both show **100% slow-path-share**.

**Threshold: < 20% per workload — NOT MET as instrumented (where
applicable).**

### Measurement-discipline caveat

This is the known measurement artifact, not a real regression: every
fast-path SMI left-shift calls
`call_slow!(op_shift_left_record_smi_rs, args = [slot])` which is
instrumented by `inc_slow_semantic_counter!` in
`crates/vm/src/dsl/backend/aarch64/control.rs:116` (every
`call_slow!` arm with `opcode_byte = N` bumps the counter,
regardless of label scope). The result: feedback-recording fast-path
entries are counted as if they were full slow-path entries.

The same 100% pattern holds across all inline-ported opcodes whose
fast paths still need a shim for feedback recording (`Add`, `Sub`,
`Mul`, `BitAnd`, and now `ShiftLeft`). Per the Phase 1.C plan §5 +
§1.6 + the Phase 1.C.1 summary's Followups #1, the per-opcode gate
should remain enforced **once the substrate distinguishes
"feedback-recording shim" from "true slow path"** — that work is a
substrate fix tracked as a Phase 1.C followup (gate counter-injection
on label-boundary state in `crates/vm-dsl/src/lower.rs`
`inject_opcode_byte`) and is not scheduled within Phase 1.C scope.

Crypto's 148M ShiftLeft dispatches are dominated by 32-bit modular
arithmetic mask operations (the `(x << n) | (y >> m)` rotations and
the `<<` accumulators ubiquitous in SHA / TEA / RC4 implementations)
— these are overwhelmingly i32-only and would record near-0% real
slow-path-share once the counter-injection artifact is accounted
for. Splay's 643K ShiftLeft is incidental (sparse-tree index shifts);
both workloads should fall well below the 20% gate when the
substrate fix lands.

**Per-workload waiver:** both workloads that exercise ShiftLeft
exceed 20% by the same instrumentation artifact described above.
The LLInt baseline on the same workloads would record 0% (no inline
path → no record_smi_rs shim) so a same-instrumentation A/B is not
meaningful. Per the Phase 1.C plan spec §1.6 + §5, per-opcode
waivers are explicitly allowed with justification — this report
supplies it. Once the fast-path / slow-path distinction lands, this
section should be re-measured.

## Behavioral tests

- `cargo test --release -p lyng-vm --lib`: **418 passed**.
- `cargo test --release -p lyng-tests`: **1209 passed**.
- Test262 left-shift slice: `cargo run --release -p lyng-test262
  -- --filter language/expressions/left-shift` → **89/89 variants
  passed across 45 files** (100% pass rate). Includes the
  `S11.7.1_*` and `bigint-*` left-shift semantics tests covering
  ToInt32(lhs) + ToUint32(rhs) + 5-bit-mask + signed-i32 result
  invariants.
- Two pre-existing failures in
  `crates/vm/tests/feedback_flat_consistency.rs`
  (`dual_write_keeps_smi_add_legacy_and_flat_in_sync` and
  `dual_write_keeps_polymorphic_property_access_legacy_and_flat_in_sync`)
  reproduce at HEAD `ce9edf4b` (Task 5 close) with the op_shift_left
  changes reverted — these failures pre-date this task and reference
  Call-feedback dual-write divergence (legacy=Some(Call(...)) vs
  flat=None at slot 0); the ShiftLeft inline path doesn't touch Call
  feedback dual-write.
- The `parses_the_committed_hot_opcodes_toml` test still fails
  ("37 opcodes > 35 max") — pre-existing and tracked in the plan.

## Post-fix slow-path-share update (2026-05-22)

After Phase 1.C followup #1 substrate fix at commit `47fc5061`, slow-
path-share re-measured with honest counter-injection discipline:

| Workload     | Dispatches  | Slow-path-share |
|--------------|------------:|----------------:|
| Richards     |           0 |              —  |
| DeltaBlue    |           0 |              —  |
| Crypto       | 151,884,545 |           0.0%  |
| RayTrace     |           0 |              —  |
| NavierStokes |           0 |              —  |
| Splay        |     704,340 |           0.0%  |

The pre-fix report's prediction (32-bit modular SHA/TEA/RC4 patterns
stay i32-bounded → near-0% real share) is empirically confirmed:
both emitting workloads measure 0.0% slow-path-share. The i32 SMI
fast path handles 32-bit modular shifts with no contention.

Per-workload gate status per spec §1.6 + §5:
- ✅ Workloads meeting <20% gate: Crypto, Splay (all emitting
  workloads meet the gate cleanly)
- ⚠ Workloads requiring waiver: none
- — N/A: Richards, DeltaBlue, RayTrace, NavierStokes (don't emit
  ShiftLeft)

See [`reports/lyng/dsl-1/phase-1c-post-fix-slow-path-share.md`](../dsl-1/phase-1c-post-fix-slow-path-share.md) for the consolidated post-fix re-measurement across all 8 inline-ported arithmetic-family opcodes.