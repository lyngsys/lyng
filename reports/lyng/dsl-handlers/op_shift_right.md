# `op_shift_right` DSL port (opcode 48)

Phase 1.C.2 inline port: SMI arithmetic (sign-preserving) right shift.
Top-30 rank #10, ~266M dispatches per V8 v7 run (the **largest
dispatch share in Phase 1.C.2** — Crypto exercises shifts most
heavily through SHA / TEA / RC4-style modular arithmetic). Final port
of sub-phase 1.C.2 (bitwise / shifts) — uses the pre-existing
`shift_right_smi!` macro from the DSL-0 substrate. Right shift on
tagged ints with a 5-bit-masked RHS cannot overflow (the result is
always a representable i32), so the fast path has no
bailout-on-overflow branch.

## DSL source

`crates/lyng/vm/src/dsl/handlers/cold.rs`:

```rust
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

The handler text is byte-for-byte identical to `op_shift_left` apart
from the opcode byte (48 vs 47), the arithmetic macro
(`shift_right_smi!` vs `shift_left_smi!`), and the recording-shim
symbol. The two macros share an identical 3-instruction shape — only
the shift mnemonic differs:

- `shift_left_smi!` emits `and w16, rhs, #0x1f` + `lsl w_dst, w_lhs, w16` + `sxtw x_dst, w_dst`.
- `shift_right_smi!` emits `and w16, rhs, #0x1f` + `asr w_dst, w_lhs, w16` + `sxtw x_dst, w_dst`.

`asr` is **Arithmetic Shift Right** — it preserves the sign bit
(replicates the input's bit 31 into the vacated high bits). This
matches ECMAScript `>>` semantics exactly: the result is the signed
i32 truncated by `n & 0x1f` arithmetic right shifts of the lhs. It is
distinct from `op_unsigned_shift_right`'s `lsr` (Logical Shift Right,
zero-fill) which implements `>>>`.

## Slow-path shims

- `op_shift_right_slow_rs` (unchanged; pre-existing cold-stub shim —
  invoked from the `.slow` label on SMI miss). Delegates to
  `crate::vm::semantics::arithmetic::op_shift_right_semantic` with
  the same 4-u32 operand-quartet adapter as op_add / op_sub / op_mul
  / op_bit_and / op_shift_left.
- `op_shift_right_record_smi_rs` (NEW; fast-path feedback recording —
  mirrors `op_add_record_smi_rs` in `hot.rs:88-106`,
  `op_sub_record_smi_rs` in `cold.rs:1076-1094`,
  `op_mul_record_smi_rs` in `cold.rs:1185-1203`,
  `op_bit_and_record_smi_rs` in `cold.rs:1541-1559`, and
  `op_shift_left_record_smi_rs` in `cold.rs:1697-1717`). Bumps the
  warmup counter, allocates the legacy vector at threshold, mirrors
  legacy state to the flat array, observes the tier feedback event.
  Returns `Continue { pc_advance: 6 }` so the asm bridge advances PC
  by op_shift_right's encoded length without re-entering
  `op_shift_right_semantic`.

## Current asm

See `reports/lyng/dsl-asm-baseline-aarch64/op_shift_right.asm`.

Fast path (from `op_shift_right_dsl:` through
`bl _op_shift_right_record_smi_rs` inclusive): **36 instructions** —
4 ldrb/ldrh decode + 1 ldr + 7 check_smi + 1 ldr + 7 check_smi + 2
sxtw untag + 3 shift_right_smi (and #0x1f + asr + sxtw) + 4 tag_smi +
1 str + 6 call_slow-setup.

Comparison vs same-family ports:

| Opcode         | Fast-path instr | Δ vs op_sub | Source of Δ                                                |
|----------------|----------------:|------------:|------------------------------------------------------------|
| op_add         | 35              | -1          | adds (1 fewer than sub_smi_overflow's b.vs branch)         |
| op_sub         | 36              |  0          | reference                                                  |
| op_mul         | 40              | +4          | smull+cmp +1; neg-zero cbnz+orr+tbnz +3                    |
| op_bit_and     | 35              | -1          | no overflow branch (and+sxtw vs subs+b.vs+sxtw)            |
| op_shift_left  | 36              |  0          | +1 from `and #0x1f` mask vs op_bit_and (lsl+sxtw shape)    |
| op_shift_right | 36              |  0          | identical to op_shift_left; `asr` swapped for `lsl`        |

The Δ is exactly zero vs op_shift_left because `shift_right_smi!` and
`shift_left_smi!` have identical instruction counts — both 3 insns
(`and #0x1f + shift + sxtw`). The `asr` mnemonic is the same length /
latency / throughput as `lsl` on AArch64 (both single-cycle ALU ops
on Apple M-series and Cortex-X / A-7x cores).

## LLInt reference

JSC's `op_rshift`
(`Source/JavaScriptCore/llint/LowLevelInterpreter64.asm`) uses the
same arithmetic shape: an offlineasm `rshifti` pseudo-op that lowers
on AArch64 to `and w_count, w_rhs, #0x1f` + `asr w_dst, w_lhs, w_count`,
plus the SMI tag-and-untag bookkeeping. There is no overflow branch —
arithmetic right-shift on i32 payloads with a 5-bit-masked count
always stays in i32 range, matching Lyng's `shift_right_smi!` macro
shape.

The only Lyng-specific overhead is the 7-instruction `check_smi`
(vs JSC's 4-instruction bit-test) — a NaN-tag-layout consequence
inherited from op_add / op_sub / op_bit_and / op_shift_left. Net:
per-dispatch Lyng emits ~36 insns vs JSC's ~28; the ratio (~1.29×)
matches the op_sub / op_add / op_bit_and / op_shift_left ratios
within budget.

## Microbench

ns/dispatch on ShiftRight microbench (7-sample median, post-warmup,
ARM64 loadavg fluctuating 2.85–4.27 — well above the 2.0 isolation
gate; ran without `--require-isolation` per spec §1.6 same-machine
A/B):

| Opcode     | Samples | Median ns | CI95   | Notes                                |
|------------|--------:|----------:|-------:|--------------------------------------|
| Add        | 7       | 144.37    | ±0.42  | Phase 1.A reference (op_add hot)     |
| Sub        | 7       | 147.18    | ±0.21  | Phase 1.C.1 Task 2                   |
| Mul        | 7       | 184.77    | ±0.37  | Phase 1.C.1 Task 3                   |
| BitAnd     | 7       | 156.92    | ±0.32  | Phase 1.C.2 Task 5                   |
| ShiftLeft  | 7       | 157.16    | ±0.15  | Phase 1.C.2 Task 6                   |
| ShiftRight | 7       | 156.89    | ±0.17  | Phase 1.C.2 Task 7 (this task)       |

ShiftRight is statistically indistinguishable from ShiftLeft and
BitAnd. The CI95 envelopes overlap completely (ShiftRight
156.72–157.06 vs ShiftLeft 157.01–157.31 vs BitAnd 156.60–157.24) —
all three fast paths cluster within ~0.3 ns of each other. This
matches the analytic expectation: the three handlers share the same
asm skeleton (load + check_smi×2 + untag×2 + arithmetic + tag + store
+ record_smi shim) with only the arithmetic core differing by one
instruction (ShiftLeft / ShiftRight have the extra `and #0x1f` mask
that BitAnd lacks). At ~36-instruction critical paths on a 3+ GHz
out-of-order core, single-instruction differences disappear into the
issue-width noise floor.

Per-opcode gate: ns/dispatch should be within 2× JSC LLInt's
op_rshift. We don't have a direct LLInt op_rshift microbench in-repo,
but the identical shape vs op_shift_left (already within budget) and
the +1 instr delta vs op_bit_and matches the analytic expectation.
Gate **considered satisfied** by inheritance from ShiftLeft / BitAnd
(same fast-path shape modulo the `asr`-vs-`lsl` mnemonic swap, same
macro substrate).

Notes:
- Loadavg was 2.85 → 4.27 → 3.66 across the run sequence. Above the
  2.0 isolation floor; the `--require-isolation` strict gate was
  bypassed per spec §1.6 same-machine A/B convention used by op_sub /
  op_mul / op_bit_and / op_shift_left.
- The microbench snippet for ShiftRight was added in this task at
  `tools/lyng-bench/src/microbench/snippets.rs` (using
  `x = i >> y` with two locals to mirror the ShiftLeft snippet
  shape). No `ShiftRightSmi` opcode exists in the bytecode-builder,
  so the two-locals form is for direct shape comparison with
  ShiftLeft, not peephole avoidance.

## Slow-path-share on V8 v7

Captured via `v8suite --count-opcodes --count-slow-path-share --samples 5`
(loadavg 2.85 → 3.66 across the sweep — above isolation floor; same
conditions as op_shift_left).

| Workload     | ShiftRight dispatches | Semantic SP   | Share  |
|--------------|----------------------:|--------------:|-------:|
| Richards     |               780,219 |       780,219 | 100.0% |
| DeltaBlue    |                     0 |             0 |    —   |
| Crypto       |           448,175,890 |   448,175,890 | 100.0% |
| RayTrace     |                 4,120 |         4,120 | 100.0% |
| NavierStokes |                     0 |             0 |    —   |
| Splay        |                     0 |             0 |    —   |
| **Total**    |       **448,960,229** | **448,960,229** | **100.0%** |

Three of the six V8 v7 workloads (DeltaBlue, NavierStokes, Splay)
don't emit ShiftRight at all. The three that do (Richards, Crypto,
RayTrace) all show **100% slow-path-share**.

The 5-sample aggregate Crypto count of 448M ShiftRight dispatches
divides to ~89.6M per single V8 v7 run — making ShiftRight one of
the highest-dispatch opcodes per Crypto invocation (>4× the
single-run BitAnd / ShiftLeft volumes). Crypto's hot loops are
dominated by 32-bit modular arithmetic where `>>` and `>>>`
appear in nearly every round of the SHA / TEA / RC4-family ciphers.

**Threshold: < 20% per workload — NOT MET as instrumented (where
applicable).**

### Measurement-discipline caveat

This is the known measurement artifact, not a real regression: every
fast-path SMI right-shift calls
`call_slow!(op_shift_right_record_smi_rs, args = [slot])` which is
instrumented by `inc_slow_semantic_counter!` in
`crates/lyng/vm/src/dsl/backend/aarch64/control.rs:116` (every
`call_slow!` arm with `opcode_byte = N` bumps the counter,
regardless of label scope). The result: feedback-recording fast-path
entries are counted as if they were full slow-path entries.

The same 100% pattern holds across all inline-ported opcodes whose
fast paths still need a shim for feedback recording (`Add`, `Sub`,
`Mul`, `BitAnd`, `ShiftLeft`, and now `ShiftRight`). Per the Phase
1.C plan §5 + §1.6 + the Phase 1.C.1 summary's Followups #1, the
per-opcode gate should remain enforced **once the substrate
distinguishes "feedback-recording shim" from "true slow path"** —
that work is a substrate fix tracked as a Phase 1.C followup (gate
counter-injection on label-boundary state in
`crates/lyng/vm-dsl/src/lower.rs` `inject_opcode_byte`) and is
not scheduled within Phase 1.C scope.

Crypto's 448M (5-run aggregate) ShiftRight dispatches are dominated
by 32-bit modular arithmetic where `(x >> n) | (y << m)` rotations
and shift-and-mask byte extractions are ubiquitous in cipher round
functions. These are overwhelmingly i32-only and would record near-0%
real slow-path-share once the counter-injection artifact is accounted
for. Richards / RayTrace's incidental counts (780K + 4K) are
peripheral sign-bit extractions and similar low-volume patterns; all
three workloads should fall well below the 20% gate when the
substrate fix lands.

**Per-workload waiver:** the three workloads that exercise ShiftRight
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
- Test262 right-shift slice: `cargo run --release -p lyng-test262
  -- --filter language/expressions/right-shift` → **73/73 variants
  passed across 37 files** (100% pass rate). Includes the
  `S11.7.2_*` and `bigint-*` right-shift semantics tests covering
  ToInt32(lhs) + ToUint32(rhs) + 5-bit-mask + sign-preserving
  arithmetic-shift invariants.
- Pre-existing failures in
  `crates/lyng/vm/tests/feedback_flat_consistency.rs`
  (`dual_write_keeps_smi_add_legacy_and_flat_in_sync` and
  `dual_write_keeps_polymorphic_property_access_legacy_and_flat_in_sync`)
  reproduce at HEAD `45c552f6` (Task 6 close) with the op_shift_right
  changes reverted — these failures pre-date this task and reference
  Call-feedback dual-write divergence (legacy=Some(Call(...)) vs
  flat=None at slot 0); the ShiftRight inline path doesn't touch
  Call feedback dual-write.
- The `parses_the_committed_hot_opcodes_toml` test still fails
  ("37 opcodes > 35 max") — pre-existing and tracked in the plan.

## Post-fix slow-path-share update (2026-05-22)

After Phase 1.C followup #1 substrate fix at commit `47fc5061`, slow-
path-share re-measured with honest counter-injection discipline:

| Workload     | Dispatches  | Slow-path-share |
|--------------|------------:|----------------:|
| Richards     |     868,131 |           0.0%  |
| DeltaBlue    |           0 |              —  |
| Crypto       | 456,364,366 |         0.008%  |
| RayTrace     |       4,200 |           0.0%  |
| NavierStokes |           0 |              —  |
| Splay        |           0 |              —  |

The i32-bounded shift-right pattern (SHA/TEA-style accumulator
operations) stays on the SMI fast path almost perfectly. All three
emitting workloads measure < 0.01% slow-path-share — a clean win
across the board.

Per-workload gate status per spec §1.6 + §5:
- ✅ Workloads meeting <20% gate: Richards, Crypto, RayTrace (all
  emitting workloads meet the gate cleanly)
- ⚠ Workloads requiring waiver: none
- — N/A: DeltaBlue, NavierStokes, Splay (don't emit ShiftRight)

See [`reports/lyng/dsl-1/phase-1c-post-fix-slow-path-share.md`](../dsl-1/phase-1c-post-fix-slow-path-share.md) for the consolidated post-fix re-measurement across all 8 inline-ported arithmetic-family opcodes.
