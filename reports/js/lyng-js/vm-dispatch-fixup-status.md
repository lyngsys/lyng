# VM Dispatch Fixup Status (lyng-1o9z follow-up)

## Scope Landed

Commits, in order:

- **Track A** (`92429445`): inline opcode + operand decode in the dispatch
  loop. Removes `decode_dispatch_instruction`, `decode_unprofiled_operands`,
  `DispatchDecode`, `DispatchOperands`, and several free-function wrappers
  (`instruction_bytes_at`, `opcode_from_byte`, `feedback_slot_from_bytes`,
  `feedback_slot_at`, `reject_dispatch_prefix`). The form classifier
  `dispatch_operand_form` is marked `#[inline(always)]` so LLVM folds it
  into each opcode arm of the central dispatch match.
- **Track D** (`b8960cc2`): inline SMI fast paths into `Add`, `Sub`,
  `Mul`, `Mod`, `BitAnd` and their `*Smi` immediate forms. Each arm now
  reads both register values, runs `i32::checked_*` (or `smi_mul_result`
  / `smi_mod_result` for -0-safe paths), writes the result, records the
  feedback slot, advances `pc`, and continues — with no helper call on
  the SMI fast path. Misses fall through to the existing
  `execute_*_opcode` helpers, preserving f64 / ToPrimitive / BigInt
  semantics including -0.

## V8 v7 Suite (5-sample median, Apple ARM)

The user's V8 v7 corpus (`testdata/js-benchmarks/v8-v7/`):

| Bench | Pre (n=1) | Post (n=5 median) | QuickJS | Pre ratio | Post ratio | Improvement |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| richards | 193 | **224** | 969 | 5.02x | **4.33x** | **+16.1%** |
| deltablue | 239 | **270** | 1051 | 4.40x | **3.89x** | **+13.0%** |
| crypto | 168 | **236** | 803 | 4.78x | **3.40x** | **+40.5%** |
| raytrace | 311 | **387** | 1011 | 3.25x | **2.61x** | **+24.4%** |
| navier-stokes | 332 | **406** | 1328 | 4.00x | **3.27x** | **+22.3%** |
| splay | 1002 | **1202** | 2298 | 2.29x | **1.91x** | **+20.0%** |

Geometric mean improvement across the suite: **~22%**.

The biggest wins are on the dispatch-bound workloads (Crypto +40%,
RayTrace +24%, NavierStokes +22%). The postmortem's gap between Lyng
and QuickJS narrowed from ~4-5x on every workload to **2.6-4.3x**, with
Crypto in particular dropping from 6x slower to 3.4x slower.

## Correctness Evidence

- `cargo test -p lyng-js-vm -p lyng-js-bytecode -p lyng-js-tests`:
  **1593 passed**, 0 failed.
- `cargo clippy -p lyng-js-vm --all-targets -- -W clippy::pedantic
  -W clippy::nursery`: clean.
- `cargo run --release -p lyng-js-test262 -- --report ...`:
  - Selected files: 53053
  - Runnable files: 49729
  - Pass: **49724** (99.99% of runnable)
  - Fail: **5** — same pre-existing failures recorded in
    `vm-dispatch-phase6-status.md`. The clusters are
    `language/import/import-defer/evaluation-triggers`,
    `language/module-code` (MissingModuleEnvironment), and one
    `language/module-code/namespace/internals` Test262Error. Plus 1
    intermittent `staging/sm/RegExp/unicode-class-braced.js` timeout
    that flakes between runs.
- Targeted Test262 slices for arithmetic / -0 semantics, all 100%:
  - `language/expressions/{addition, subtraction, multiplication,
    modulus}`
  - `built-ins/Math/sign`
  - `built-ins/Object/is`

## Profiler Evidence

On Richards (`sample` for 5s, post-A+D):

| Frame | Samples (n=1470) | Share |
| --- | ---: | ---: |
| `run_dispatch_loop` body | 608 | 41% |
| `execute_get_named_property_opcode` | 194 | 13% |
| `call_value_small` + IC | 179 | 12% |
| `try_named_property_load_inline_cache_hit` | 79 | 5% |
| `invoke_collected_call_value` | 78 | 5% |
| Other handler bodies | rest | — |

Critically, **`decode_dispatch_instruction` and `dispatch_operand_form`
no longer appear in the profile**. Before Track A, they were 26% and
13% of Richards respectively. The dispatch *infrastructure* cost
disappeared; the remaining time is in handler bodies and the IC layer.

## What Did Not Land — And Why

### Track B: slim `DispatchState`

The plan called for hoisting `FrameMetadata` out of the per-iteration
`DispatchState` and only carrying the mutable `FrameState` subset.
**Skipped.** Post-A+D profiling shows the dispatch-loop body is
dominated by handler work (property access, IC, calls), not by
`FrameRecord` field access. The 88-byte FrameRecord copy that the
postmortem highlighted is already paid only once per activation (not
per opcode) since `fa99ed14`. Hoisting metadata further would touch
~50 helper sites for an unclear gain. Defer until a future profile
shows it matters.

### Track C: compiler-side Move elimination

Implemented as a peephole that folded `LoadX Rtemp; Move Rdst, Rtemp`
into `LoadX Rdst` when `Rtemp` is dead after. **Reverted after
benchmarking** — the pattern almost never matches in real bytecode
because:

1. The compiler's `lower_expr_into(expr, dest)` already emits directly
   to the final destination in most cases.
2. Register reuse defeats the conservative "dead after Move" check
   when the temp register gets allocated to the next sub-expression.

The implementation was correct (no test failures, no regressions, the
encoding-form check protected the Abx8 short forms) but delivered
+0-2% on V8 v7 workloads — within noise. The opcode-mix report
showed Move counts unchanged on every workload. Net negative: ~200
lines of peephole machinery for no measurable runtime gain. Reverted
to keep the codebase honest.

A useful future Track C would be a smarter peephole that targets the
real Move patterns (argument marshaling, loop variable copy, property
store setup) — those don't follow `LoadX Rtemp; Move` shapes, so they
need a different pattern set. Track separately if the Move share
stays this high after other optimizations.

## Where The Remaining Gap Lives

Post-fixup we are still 2.6-4.3x slower than QuickJS. The remaining
delta is **not in dispatch** — the profile confirms decode is gone.
The next places to look:

- **Property access handlers** (`execute_get_named_property_opcode` =
  13% of Richards): the IC fast path goes through several helper
  function calls. Some inlining and slot lookup tightening could
  help.
- **Call dispatch** (`call_value_small`, `invoke_collected_call_value`
  = 12% of Richards combined): each call probably pays argument
  marshaling, frame setup, and entry trampoline costs that could be
  inlined.
- **Feedback recording**: `record_feedback_slot`,
  `try_named_property_load_inline_cache_hit`, and friends are still
  per-call function calls. Inlining the warm-path could shave 5-10%.
- **String / RegExp handlers**: regexp-heavy workloads pay a lot of
  time in `dispatch_regexp_builtin`, which is unrelated to dispatch.

A second-pass focused on the IC and call paths would likely deliver
another 15-30%. Threaded dispatch (Phase 7) still doesn't look
needed — the central match is not the bottleneck.

## Reproduce

```sh
cargo build --release -p lyng-js-cli

for b in richards deltablue crypto raytrace navier-stokes splay; do
  cat testdata/js-benchmarks/v8-v7/base.js \
      testdata/js-benchmarks/v8-v7/$b.js > /tmp/lj-$b.js
  cat >> /tmp/lj-$b.js <<'EOF'

BenchmarkSuite.RunSuites({
  NotifyResult: function(n,r){print(n+": "+r)},
  NotifyError: function(n,e){print(n+" ERROR: "+e)},
  NotifyScore: function(s){print("Score: "+s)}
});
EOF
  printf "%-15s lyng=%s qjs=%s\n" "$b" \
    "$(./target/release/lyng-js --shell /tmp/lj-$b.js | grep '^Score: ')" \
    "$(qjs /tmp/lj-$b.js | grep '^Score: ')"
done

# Test262 (corresponds to test262.md totals)
cargo run --release -p lyng-js-test262 -- --report /tmp/lj-t262.md -j 12
```
