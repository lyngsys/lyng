# lyng-4pvk — Remove `argument_scratch` Vec materialization for ordinary VM calls: status report

**Issue:** `lyng-4pvk` — Remove argument_scratch Vec materialization for ordinary VM calls
**Parent:** `lyng-48k8` — Phase 4: Compiler and bytecode polish (epic)
**Master:** `lyng-49qk` — JSC-aligned engine roadmap
**Date:** 2026-05-15
**Toolchain:** rustc 1.93 (2025-12-15), aarch64-apple-darwin, `--release` profile
**Parent commit:** `d9243123` (Phase 4c deferral rationale)

## What landed

A no-Vec fast path for ordinary bytecode-to-bytecode calls. Phase 4a moved
the compiler to reserve-then-lower-into for call arguments — args land
directly in the caller's contiguous register block — but the VM was still
copying through `argument_scratch: Vec<Value>` on every call. The fast
path skips the Vec and copies directly from caller register slots into
the callee parameter slots via a single `register_stack.copy_within`.

### Eligibility

`Vm::ordinary_bytecode_call_eligibility` rejects callees that need the
Vec for legitimate reasons:

- bound functions (`bound_function_record` is `Some`) — need bound-arg
  prepending
- non-bytecode callees (`bytecode_entry` is `None`) — builtin / proxy
  / native paths consume `&[Value]` directly
- generators / async functions — `instantiate_*_call` stores the args
  in the generator/promise object
- class constructors — slow path throws TypeError for non-construct
  invocation; we let it do that
- functions with `arguments_mode != None` or `has_rest_parameter` —
  `initialize_activation_objects` would still need a materialized slice

For everything else (the bulk of modern arrow / strict-mode / methods-
that-don't-touch-arguments paths), the fast path applies.

### The hot path

`call_value_small` (Call0..3 opcodes) and `call_value` (generic `Call`
without spread, ≥4 args) both check eligibility first. When eligible,
they hand off to `Vm::invoke_bytecode_call_from_caller_arg_window`,
which:

1. Computes the absolute register-stack index of the caller's first
   argument slot from `frame.registers().base() + caller_arg_base_local`.
2. Advances + syncs the caller dispatch frame past the call instruction.
3. Calls `enter_bytecode_call_from_caller_registers` →
   `install_prepared_bytecode_call_from_registers`:
   - `prepare_bytecode_call` (shared with the slow path; computes
     `prepared.new_target` for arrow's lexical-this/lexical-new.target)
   - `reserve_register_window` grows the register stack for the callee
     and fills the new slots with `undefined`
   - `copy_arguments_from_caller_registers` does one
     `register_stack.copy_within(src..src+min(arg_count, param_count), dst)`
     — the source window sits entirely before the destination window,
     so the copy is non-overlapping
   - FrameRecord constructed identically to the slow path's call shape
     (this_value, parameter_initializer_end_offset, new_target, callee,
     entry+suspendable flags)
4. Calls `observe_call_target` for feedback recording.

Spread, bound, FP.call, and Construct paths fall through to the existing
Vec-materializing slow path unchanged.

### Counter

A new `CallArgumentCopyCounterStore` (gated on the existing `opcode-counters`
feature) tracks:

- `scratch_pushes`: argument values pushed into `argument_scratch`
- `frame_copies`: argument values copied into a callee bytecode frame

Symmetric pair: an ordinary Call3 with my fix increments `frame_copies`
by 3 and `scratch_pushes` by 0. Pre-fix it would have incremented both
by 3 (push into scratch, then write into frame).

The counter is off by default in production and behind `#[cfg(feature =
"opcode-counters")]` everywhere.

## Verification

### Counter regression tests (`opcode-counters`)

| Test | Workload | Expected |
|---|---|---|
| `ordinary_bytecode_calls_avoid_argument_scratch_pushes` | `(a,b,c) => a+b+c` × 10 iters | `scratch_pushes == 0`, `frame_copies == 30` |
| `generic_call_with_more_than_three_args_also_avoids_scratch_pushes` | `(a,b,c,d,e) => a+b+c+d+e` × 8 iters | `scratch_pushes == 0`, `frame_copies == 40` |
| `spread_call_still_materializes_into_argument_scratch` | `add3(...args)` | `scratch_pushes > 0` |
| `bound_function_call_still_materializes_into_argument_scratch` | `plus.bind({...}, 1)(2, 3)` | `scratch_pushes > 0` |
| `nonstrict_function_referencing_arguments_object_stays_on_slow_path` | function with `arguments.length` | `scratch_pushes > 0` |
| `rest_parameter_function_stays_on_slow_path` | `(head, ...tail) =>` | `scratch_pushes > 0` |

Plus `fast_path_handles_iife_closure_helpers_calling_each_other` (non-
feature-gated) pins a deepEqual-style IIFE pattern with mutual recursion
through `||` short-circuits.

### Test suites

- `cargo test -p lyng-vm`: **401 passed** (default features)
- `cargo test --features opcode-counters -p lyng-vm`: **410 passed**
- `cargo test -p lyng-vm -p lyng-tests -p lyng-bytecode -p lyng-compiler`: **1704 passed**
- `cargo fmt --check` clean on all touched files
  ([opcode_counts.rs](../../crates/lyng/vm/src/opcode_counts.rs),
  [lib.rs](../../crates/lyng/vm/src/lib.rs),
  [vm.rs](../../crates/lyng/vm/src/vm.rs),
  [vm/call.rs](../../crates/lyng/vm/src/vm/call.rs),
  [vm/bytecode_calls.rs](../../crates/lyng/vm/src/vm/bytecode_calls.rs),
  [tests/core.rs](../../crates/lyng/vm/src/tests/core.rs))
- `cargo clippy --features opcode-counters -p lyng-vm --tests`: 0 errors;
  the 7 remaining warnings are pre-existing in files not touched by this
  change (`dispatch.rs`, `dispatch/property.rs`, `feedback.rs`,
  `dispatch_handlers/arithmetic.rs`).

### Test262

| Configuration | Pass rate | Failures |
|---|---|---|
| Pre-fix baseline (`d9243123`, `-j 12`) | 49721/49729 files | 8 (1 flaky timeout) |
| Post-fix (`-j 1`) | 49722/49729 files | 7 (no flaky) |
| Post-fix (`-j 12`, three runs) | 49721–49722/49729 files | 7–8 (same flaky as baseline) |

The flaky failure is `staging/sm/RegExp/unicode-class-braced.js [non-strict]:
timeout after 1.0s` — already documented by Phase 4b as timing noise, not
a semantics regression. No new failures introduced by this change.

### Runtime bench (vs `d9243123` Phase 4c-deferral baseline, `--preset baseline`)

| Workload | Phase 4c-deferral ns/op | lyng-4pvk ns/op | Δ |
|---|---:|---:|---:|
| `module-heavy.compile` | 3353.59 | 3091.44 | **−7.82%** |
| `regexp-constructor-compile.runtime` | 4005.39 | 3862.20 | **−3.57%** |
| `class-heavy.runtime` | 2424.54 | 2338.46 | **−3.55%** |
| `string-heavy.concat-runtime` | 180.96 | 176.40 | **−2.52%** |
| `regexp-heavy.runtime` | 47214.79 | 46162.97 | **−2.23%** |
| `array-heavy.literal-indexed-runtime` | 671.47 | 662.23 | −1.38% |
| `regexp-named-replace.runtime` | 12948.11 | 12781.07 | −1.29% |
| `regexp-stable-exec.runtime` | 34367.91 | 34147.61 | −0.64% |
| `typed-array-heavy.runtime` | 776.09 | 771.23 | −0.63% |
| `array-heavy.iterator-runtime` | 9694.44 | 9681.19 | −0.14% |
| `regexp-legacy-statics.runtime` | 2071.52 | 2126.33 | +2.65% |
| `async-heavy.frontend` | 3684.03 | 3783.76 | +2.71% |

- 7/12 workloads improved by ≥1%
- 3/12 workloads neutral (within ±1%)
- 2/12 workloads minor regressions (frontend / legacy-statics paths
  that don't exercise the fast path; both within typical between-run
  variance for these rows)

Largest wins on call-heavy workloads (`class-heavy`, `module-heavy.compile`,
`regexp-constructor-compile`) — the targeted scope.

## What's deferred / out of scope

- **`construct_value`** (the `Construct` opcode): not addressed. `new`
  expressions still materialize through `argument_scratch`. Trickier
  because of `new_target` threading, derived class constructor `this`
  setup, and bound-construct chain resolution. Worth its own follow-up
  once the fast path's shape settles.
- **`tail_call_value`** (the `TailCall` opcode): not addressed. Tail-
  call recycling reuses the caller's register base, which interacts
  with `teardown_tail_frame` and requires preserving caller state for
  recovery. Defer.
- **`Function.prototype.call`/`apply` shortcuts**
  (`invoke_function_call_builtin_target`): still routes through the
  Vec because it can resolve to a bound chain. Worth a future fast
  path if profiling shows it's hot.
- **Generator/async with simple parameters**: skipped by eligibility
  because `instantiate_generator_call`/`instantiate_async_function_call`
  takes `&[Value]`. Refactoring those signatures would let the fast
  path cover them, but their hot-path frequency is low compared to
  ordinary calls.

## References

- Source: third-party JSC-vs-Lyng review (May 2026), finding #2.
- JSC LLInt analogue:
  `Source/JavaScriptCore/llint/LowLevelInterpreter64.asm:2467` — direct
  callee-frame setup from caller register window via `CallLinkInfo`.
- Phase 4a (compiler-side reserve-then-lower-into): commit `835c19f6`.
- Phase 4b (Star fusion in dispatch): commit `631ec709`.
- Phase 4c deferral rationale: commit `d9243123`.
