# Phase 4b — Star fusion lookahead: status report

**Issue:** `lyng-48k8` — Phase 4: Compiler and bytecode polish (epic)
**Parent:** `lyng-49qk` — JSC-aligned engine roadmap (master epic)
**Date:** 2026-05-15
**Toolchain:** rustc 1.93 (2025-12-15), aarch64-apple-darwin, `--release` profile
**Parent commit:** `835c19f6` (Phase 4a close)

## What landed

Writer-side Star fusion for the accumulator-load (`Lda*`) handler family,
matching V8 Ignition's `src/interpreter/interpreter-assembler.cc` peephole:
when a `Lda*` handler is followed by a `StarN` byte, the handler stores
the value into register `N` inline and advances past the `Star` byte
before dispatching to the *next-next* opcode. The pair runs as one
dispatch instead of two.

### Handler shape

`dispatch_next_with_value!($state, $value)` replaces `dispatch_next!`
in nine accumulator-load handlers:

| Handler | Opcode | Source |
|---|---|---|
| `op_lda_undefined` | `LdaUndefined` | constant `Value::undefined()` |
| `op_lda_null` | `LdaNull` | constant `Value::null()` |
| `op_lda_true` | `LdaTrue` | constant `Value::from_bool(true)` |
| `op_lda_false` | `LdaFalse` | constant `Value::from_bool(false)` |
| `op_lda_zero` | `LdaZero` | constant `Value::from_smi(0)` |
| `op_lda_one` | `LdaOne` | constant `Value::from_smi(1)` |
| `op_lda_smi8` | `LdaSmi8` | embedded `i8` → SMI |
| `op_lda_const8` | `LdaConst8` | constant pool index |
| `op_ldar` | `Ldar` | register read |

These are the only opcodes that *always* write to register 0 — every
other value-producing handler takes an explicit destination operand, so
fusing there would require an additional `a == 0` runtime branch per
dispatch. That extension is deferred to a Phase 4c follow-up if measurement
justifies it.

### The macro

```rust
macro_rules! dispatch_next_with_value {
    ($state:expr, $value:expr) => {{
        let byte = $state.next_opcode_byte();
        if let Some(target) =
            ::lyng_js_bytecode::Opcode::accumulator_store_index_for_byte(byte)
        {
            let registers = $state.frame.registers();
            $state.vm.write_register_unchecked(registers, target, $value);
            $state.advance(1);
            let next_byte = $state.next_opcode_byte();
            return Step::Continue(DISPATCH_TABLE[next_byte as usize]);
        }
        return Step::Continue(DISPATCH_TABLE[byte as usize]);
    }};
}
```

The fast path writes both `r0` (the original `Lda*` semantics) *and*
`rN`, then advances past the `Star` byte and dispatches the instruction
*after* the `Star`. The semantic equivalence is preserved (subsequent
code reading `r0` still sees the loaded value).

### Bytecode-emitter invariant

`next_opcode_byte()`'s SAFETY contract — `PC < instruction_bytes.len()`
— is preserved transitively: every `StarN` is followed by another valid
opcode (the bytecode emitter never terminates a function with a
`Star*`), so the second `next_opcode_byte()` after `advance(1)` is also
in-bounds. The contract is documented at the new
`Opcode::accumulator_store_index_for_byte` helper.

## Verification

### Tests

| Check | Before 4b | After 4b | Δ |
|---|---:|---:|---|
| `cargo test -p lyng-js-compiler` | 71 passed | 71 passed | unchanged |
| `cargo test -p lyng-js-bytecode` | 45 passed | 45 passed | unchanged |
| `cargo test -p lyng-js-vm` | 401 passed | 401 passed | unchanged |
| `cargo test -p lyng-js-vm --features opcode-counters` | 402 passed | **403 passed** | +1 (new Star fusion regression) |
| `cargo test -p lyng-js-tests` | 1186 passed | 1186 passed | unchanged |
| `cargo clippy -p lyng-js-vm --all-features --tests -- -W clippy::pedantic -W clippy::nursery` | clean on changed files | clean on changed files | unchanged |

The new regression test `vm_star_fusion_elides_star_dispatch_after_lda`
builds a 3-instruction script that compacts to `LdaOne; Star2; Return r2`,
enables opcode dispatch counters, and asserts:

- `LdaOne` dispatched once (the fused producer)
- `Star2` dispatched **zero times** (folded into the `LdaOne` handler tail)
- `Return` dispatched once
- Total dispatches == 2 (was 3 before fusion)

This pins the fusion shape so future changes can't silently regress it.

### Runtime benchmark suite (lyng-js-bench runtime, 7 samples / 9 timed runs)

Δ ns/work-unit, vs the prior bench checkpoint (Phase 4a):

| Benchmark | Δ ns/wu | Δ % | Note |
|---|---:|---:|---|
| `array-heavy.iterator-runtime` | −308.62 | **−3.2%** | iterator-driven traversal |
| `array-heavy.literal-indexed-runtime` | −63.63 | **−9.7%** | dense indexed read/write |
| `async-heavy.frontend` | +89.12 | +2.4% | frontend-only, within noise |
| `class-heavy.runtime` | −93.03 | **−3.9%** | constructor + super dispatch |
| `module-heavy.compile` | +81.98 | +2.6% | compile-only, within noise |
| `regexp-constructor-compile.runtime` | −53.93 | −1.4% | |
| `regexp-heavy.runtime` | −1138.57 | **−2.4%** | mixed RegExp |
| `regexp-legacy-statics.runtime` | +18.07 | +0.9% | within noise |
| `regexp-named-replace.runtime` | −552.91 | **−4.2%** | named-capture replace |
| `regexp-stable-exec.runtime` | +137.05 | +0.4% | within noise |
| `string-heavy.concat-runtime` | −17.47 | **−10.1%** | string concat |
| `typed-array-heavy.runtime` | −39.02 | **−5.2%** | DataView + typed-array |

**10/12 workloads improved**, 2 within noise. The largest gains track
workloads with hot Lda+Star pairs in their bytecode (string-heavy,
array-indexed, typed-array, class-heavy). The class-heavy workload is
especially interesting — it has visible Star6 dispatches in the opcode
count baseline (258,048 per run), which Phase 4b now elides entirely.

Full bench report: [`phase-4b-bench.md`](phase-4b-bench.md).

### Bytecode density

Static density is **unchanged** — Star fusion is a runtime-only
optimization, not a bytecode reshape. The aggregate stays at 4495
unit bytes; the static-density table is identical to Phase 4a.

The runtime throughput proxy column (a smaller, faster, noisier
sub-bench than the main bench suite) shows ±5% variance run-to-run and
is not a reliable signal for changes this size. The main runtime
benchmark (above) is the load-bearing measurement.

Full density report: [`phase-4b-density.md`](phase-4b-density.md).

### Test262

| | Files runnable | Files passed | Pass rate | Δ files vs Phase 4a |
|---|---:|---:|---:|---:|
| Phase 4a | 49729 | 49722 | 99.99% | — |
| **Phase 4b** | **49729** | **49721** | **99.99%** | **−1 (flaky timeout)** |

The one new failure is `staging/sm/RegExp/unicode-class-braced.js
[non-strict]` — `timeout after 1.0s`. This test has been timing out
intermittently since Phase 2a (it ran at 0.986s then, 0.801s in Phase
4a, and over 1.0s in this run). The strict variant still passes. The
failure is timing noise, not a semantics regression.

All 7 file failures from Phase 4a are still present and still
pre-existing (import-defer modules, module cycles, TypedArray
toLocaleString, class className, TDZ binding); no new deterministic
failures from Phase 4b.

Full report: [`phase-4b-test262.md`](phase-4b-test262.md).

## Why the gain is bounded — and what unlocks more

The Phase 4b fusion only fires when `LdaX` is followed by `StarN` in
the encoded bytecode. The post-emission peephole produces this pattern
only for the narrow case `LoadX r0 + Move rN, r0`, both of which
require the compiler to have *targeted register 0* in the first place.

The current compiler doesn't bias toward `r0` — `lower_expr_to_temp`
allocates fresh registers from the high end of the window, so `Lda*`
appears mostly only for intentional accumulator routing (which is
rare today). This is the **Phase 4c** workstream: bias the compiler
to lower single-use intermediate values into `r0` so the post-emission
peephole produces more `LdaX; StarN` pairs, which Phase 4b's fusion
then collapses into single dispatches.

The current `−3.2%` to `−10.1%` gains came mostly from incidental
accumulator routing (class-heavy.runtime's Star6 traffic + a handful
of inner-loop patterns). With Phase 4c, the fusion fires more often.

## What's deferred

- **4c — Compact accumulator-based bytecode**: compiler-side bias
  toward accumulator-form opcodes. Next sub-task on this epic.
- **Fusion for non-Lda value producers**: opcodes like `Add`,
  `GetNamedProperty` etc. that *could* write to `r0` if the compiler
  routed them there. Adding fusion checks here costs an `a == 0`
  branch per dispatch; gated on whether 4c produces enough such
  patterns to justify it.

## Files changed

**VM**:
- `crates/lyng-js/vm/src/vm/dispatch_state.rs` — new
  `dispatch_next_with_value!` macro alongside `dispatch_next!`.
- `crates/lyng-js/vm/src/vm/dispatch_handlers/loads.rs` — nine `Lda*`
  handlers switched to `dispatch_next_with_value!`.
- `crates/lyng-js/vm/src/tests/core.rs` — new
  `vm_star_fusion_elides_star_dispatch_after_lda` regression test
  (`#[cfg(feature = "opcode-counters")]`).

**Bytecode**:
- `crates/lyng-js/bytecode/src/opcode.rs` — new
  `Opcode::accumulator_store_index_for_byte(byte)` helper for the
  dispatch-loop peek (avoids materializing a full `Opcode` enum value
  on the hot path).

**Reports**:
- `reports/js/lyng-js/phase-4b-status.md` (this file).
- `reports/js/lyng-js/phase-4b-density.md`.
- `reports/js/lyng-js/phase-4b-bench.md`.
- `reports/js/lyng-js/phase-4b-test262.md`.

Total Phase 4b diff: roughly +90 added lines / −20 modified across 4
source files, plus the regression test (~60 lines) and the four
report files.
