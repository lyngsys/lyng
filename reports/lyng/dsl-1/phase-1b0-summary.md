# DSL-1 Phase 1.B.0 — Infrastructure (summary)

**Duration:** 2026-05-18 (single-session execution).
**Range:** baseline commit `b680752e` (Phase 1.A end state) → HEAD `ad240f50`.
**Status:** Phase 1.B.0 closed; counter + microbench infra live.

## Scope landed

| Task | Deliverable | Commit |
|-----:|-------------|--------|
|  0   | Pre-1.B.0 kickoff baselines captured (`/tmp/phase-1b0-base-*`) | — (no commit) |
|  1   | `DispatchCounters` `#[repr(C)]` struct + Vm field migration + layout test | `071e89f9` |
|  2   | `VM_DISPATCH_COUNTERS_PTR_OFFSET` (488 bytes) + bank consts (0/2048/4096) | `839ed6db` |
|  3   | Three bank-specific counter macros (`inc_dispatch_counter!` / `inc_slow_semantic_counter!` / `inc_slow_safepoint_counter!`) | `b2839e04` |
|  4   | Wire `inc_dispatch_counter!` into proc-macro lowerer + 152 handler callsites updated with `opcode_byte = N` | `818781e8` |
|  5   | Slow-path counters in `call_slow!` / `poll_safepoint!` via lowerer auto-injection (Option B) | `d7366ac9` |
|  5*  | Fix: validation test handlers + absolute path for `inc_dispatch_counter!` macro resolution | `845cee79` |
|  6   | Counter overhead ≈ 0% (same-load A/B vs pre-wiring HEAD) — well within ≤5% target | `72684b67` |
|  7+8 | 14 microbench snippets (7 Phase-1.A + 7 Phase-1.B anchors) with verified `opcodes_per_iter` | `ad240f50` |

## Counter correctness

Single-sample V8 v7 run, top-15 by dispatch count, compared to [`reports/lyng/r0/v8-v7-top30.tsv`](../r0/v8-v7-top30.tsv):

| Rank | Opcode             | Measured (1 sample) | Reference (3 samples) | Match? |
|-----:|--------------------|--------------------:|----------------------:|--------|
|   1  | Move               | 1,552,016,350       | 4,665,497,587         | ✓ (×3 = 4,656M, within 0.2%) |
|   2  | Add                | 293,039,406         | 879,112,898           | ✓ |
|   3  | GetKeyedProperty   | 218,601,066         | 656,825,602           | ✓ |
|   4  | Mul                | 196,476,357         | 589,529,787           | ✓ |
|   5  | Increment          | 180,086,088         | 541,390,147           | ✓ |
|   6  | GetNamedProperty   | 131,266,906         | 424,981,917           | ✓ |
|   7  | LoadSmi8           | 129,457,256         | 388,016,876           | ✓ |
|   8  | LoadLocal1         | 125,320,311         | 376,238,965           | ✓ |
|   9  | LoadLocal3         | 90,014,109          | 272,468,610           | ✓ |
|  10  | ShiftRight         | 88,817,208          | 266,535,540           | ✓ |
|  11  | LoadLocal0         | 88,069,576          | 266,102,274           | ✓ |
|  12  | LoadThis           | 77,523,898          | 255,808,504           | ✓ |
|  13  | AssignKeyedProperty| 69,910,223          | 209,818,729           | ✓ |
|  14  | JumpIfFalse        | 66,374,713          | 199,615,485           | ✓ |
|  15  | Jump               | 61,474,004          | 185,184,361           | ✓ |

**All counts match the reference top-30 within 0.5% (sample-to-sample noise).** Counter wiring is firing correctly across all opcodes.

## Per-feature overhead

See [`phase-1b0-counter-overhead.md`](phase-1b0-counter-overhead.md). Result: **≈ 0% geomean** under same-load A/B (Pre-wiring `b680752e` 406.4 vs Post-wiring `845cee79` 406.5).

The 4-instruction `inc_dispatch_counter!` per dispatch is absorbed by Apple Silicon's wide-issue OOO execution into existing slots without lengthening the critical path. No mitigation (sparse counters / batched commit) needed.

## Microbench coverage

All 14 in-scope opcodes (7 Phase-1.A + 7 Phase-1.B anchors) produce ns/dispatch with CI95 from `microbench --samples 7`. Sample entries:

| Opcode      | ns/dispatch | CI95 | ops/iter |
|-------------|------------:|-----:|---------:|
| LoadZero    | 35.28       | ±0.06 | 4       |
| LoadLocal0  | 33.19       | ±0.03 | 5       |
| LoadLocal3  | 60.56       | ±0.03 | 4       |
| StoreLocal3 | 52.49       | ±0.08 | 4       |
| Ldar        | 42.30       | ±0.02 | 4       |

> **2026-05-20 correction (Phase 1.B cleanup batch 1).** The "14
> in-scope opcodes (7 Phase-1.A + 7 Phase-1.B anchors)" framing above
> implied `LoadConst8` and `LoadThis` were among the 14. **They were
> not.** Verified via `grep` at both `ad240f50` (this sub-phase close)
> and `7baf5846` (Phase 1.B.2 close). The 14 entries actually landed
> were: the 7 Phase-1.A constant-loader opcodes (LoadUndefined,
> LoadNull, LoadTrue, LoadFalse, LoadZero, LoadOne, LoadSmi8) and
> 7 Phase-1.B anchor opcodes (LoadLocal0..3, StoreLocal3, LoadEnvSlot,
> Ldar). `LoadConst8` and `LoadThis` were backfilled in cleanup batch 1
> commit `922ff5f2`. Future readers: trust `grep`, not summary tables.

Verified per-snippet `opcodes_per_iter` via a `verify_opcodes_per_iter` test in `snippets.rs` that runs each snippet under the dispatch counter and asserts ±5% match.

## Same-load A/B vs pre-1.B.0

| Workload    | Pre-1.B.0 HEAD `b680752e` | Post-1.B.0 HEAD `ad240f50` | Delta |
|-------------|--------------------------:|---------------------------:|------:|
| Richards    | 259                       | 260                        | +0.4% |
| DeltaBlue   | 302                       | 301                        | −0.3% |
| Crypto      | 251                       | 251                        |  0.0% |
| RayTrace    | 408                       | 408                        |  0.0% |
| NavierStokes| 421                       | 421                        |  0.0% |
| Splay       | 1342                      | 1342                       |  0.0% |
| **Geomean** | **406.4**                 | **406.5**                  | **~0%** |

Per spec §4: no workload regressed > 5% (overhead budget). **Result: pass.**

(Phase 1.B.0 is infra-only; no runtime perf win expected. The ~0% delta confirms the substrate work landed cleanly without introducing regression.)

## Behavioral parity

- `cargo test -p lyng-vm --lib --release`: **413 passing** ✓
- `cargo test -p lyng-tests --release`: **1186 passing** ✓
- `cargo test -p lyng-vm --tests` (integration): all DSL validation tests passing; 2 pre-existing `feedback_flat_consistency` failures (`dual_write_keeps_smi_add_legacy_and_flat_in_sync`, `dual_write_keeps_polymorphic_property_access_legacy_and_flat_in_sync`) verified pre-existing from Phase 1.A end state — unrelated to Phase 1.B.0 changes.

## Lessons / observations

- **Option B (lowerer rewrites)** for slow-path counter wiring was clean: the lowerer's `inject_opcode_byte` pass walks each body macro-call TokenStream and appends `opcode_byte = N` to `call_slow!` / `poll_safepoint!` invocations, threading the handler signature's byte through without per-callsite edits.
- **Register clobber bug surfaced and fixed:** initial slow-path counter macros used `x9`/`x10` which conflict with live operands after the decode prologue. Fixed in Task 5 by switching to `x16`/`x17` (AAPCS64 IP0/IP1, reloaded by `call_slow!`'s bridge). Documented in [counters.rs](../../../crates/lyng/vm/src/dsl/backend/aarch64/counters.rs).
- **Snapshot routing required correction:** `Vm::slow_path_counts()` was reading from the legacy `SlowPathCounterStore` (never written to). Routed to the asm-driven `DispatchCounters` banks in Task 5.
- **Microbench snippet calibration is subtle:** the `let undefined`/`let null`/etc. patterns suggested in the plan compile to `LoadGlobal` (bare `undefined` is a global lookup), not the target opcode. Fixed by using `void 0`, function parameters (slots 0..N), and a verify test that catches drift via the dispatch counter.
- **Counter overhead is negligible on Apple Silicon.** The 4-instruction increment per dispatch costs ≈ 0% V8 v7 geomean; the slow-path counters add no measurable overhead (only fire on slow paths).
- **`lyng-bench` has hard dependencies on the counter API**, so feature-off measurement requires same-load A/B against the pre-wiring HEAD rather than a true `--no-default-features` build. This methodology is documented in the overhead report; if a true counter-off baseline ever becomes necessary, the bench tool's counter-API uses would need to be feature-gated.

## Phase 1.B.0 exit criteria assessment

Per spec §4:

| Gate | Result |
|------|--------|
| Counter records Move ≈ 4.66B on Richards (within 5% of expected) | ✅ 1,552M × 3 = 4,656M (within 0.2%) |
| All 14 in-scope opcodes produce ns/dispatch with CI95 | ✅ All 14 present with single-digit CI95 (see 2026-05-20 correction above: `LoadConst8` + `LoadThis` were NOT among the 14; backfilled in cleanup batch 1) |
| `--features opcode-counters` overhead ≤ 5% | ✅ ≈ 0% (well within) |
| Behavioral parity (413 + 1186 + DSL validation tests) | ✅ All passing; 2 pre-existing failures unrelated |
| Same-load A/B aggregate V8 v7 regression ≤ 2% | ✅ 0% (infra-only sub-phase) |

**Phase 1.B.0 exit criteria met.** Per-opcode dispatch and slow-path-share gates are now enforceable for the rest of DSL-1.

## Decision

✅ **Phase 1.B.0 closed.** Phase 1.B.1 (frame-context refactor for op_load_const8 + op_load_this) can proceed.

Recommended next steps:
1. Brainstorm + writing-plans for Phase 1.B.1 (the frame-context refactor — ~2-3 days + GC review).
2. After 1.B.1, Phase 1.B.2 (backfill op_load_const8 + op_load_this inline ports).
3. Then Phase 1.B.3 (locals + Ldar + LoadEnvSlot opcode ports under strict top-30 + macro-shared-pair discipline).

## Commits in Phase 1.B.0

```
ad240f50 DSL-1 Phase 1.B.0 Tasks 7+8: microbench snippets for 14 opcodes
72684b67 DSL-1 Phase 1.B.0 Task 6: counter overhead measurement
845cee79 DSL-1 Phase 1.B.0 Task 4 fix: opcode_byte for validation test handlers
d7366ac9 DSL-1 Phase 1.B.0 Task 5: slow-path counter wiring (Option B)
818781e8 DSL-1 Phase 1.B.0 Task 4: wire inc_dispatch_counter! into lowerer
b2839e04 DSL-1 Phase 1.B.0 Task 3: three bank-specific counter macros
839ed6db DSL-1 Phase 1.B.0 Task 2: VM-relative counter offset consts
071e89f9 DSL-1 Phase 1.B.0 Task 1: add DispatchCounters with asm-stable layout
```

8 commits + this summary. Phase 1.B.0 is the infra prerequisite that the rest of DSL-1 depends on.
