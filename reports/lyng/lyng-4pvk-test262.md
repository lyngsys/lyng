# lyng-4pvk — Test262 verification

**Issue:** `lyng-4pvk` — Remove `argument_scratch` Vec materialization for ordinary VM calls
**Baseline commit:** `d9243123` (Phase 4c deferral)
**Command:** `cargo run --release -p lyng-test262 -- --report ... --list-failures`

## Result summary

| Configuration | Pass rate | Failing files | Notes |
|---|---|---:|---|
| Pre-fix baseline (`d9243123`, `-j 12`) | 49721/49729 | 8 | 1 of 8 is the flaky `unicode-class-braced.js` timeout |
| Post-fix (`-j 1`) | 49722/49729 | 7 | flaky test does not fire at single-thread load |
| Post-fix (`-j 12`, run 1) | 49722/49729 | 7 | flaky did not fire this run |
| Post-fix (`-j 12`, run 2) | 49721/49729 | 8 | flaky fired |
| Post-fix (`-j 12`, run 3) | 49722/49729 | 7 | flaky did not fire this run |

**No new failures introduced by this change.** The flaky `unicode-class-
braced.js` timeout is already documented by Phase 4b as load-dependent
timing noise.

## Failure set (stable across runs)

All also present on the pre-fix baseline:

1. `language/import/import-defer/evaluation-triggers/trigger-exported-string-super-property-set-exported.js`
2. `language/import/import-defer/evaluation-triggers/trigger-not-exported-string-super-property-set-exported.js`
3. `language/module-code/instn-star-iee-single-cycle-same-name.js` (`MissingModuleEnvironment`)
4. `language/module-code/instn-star-iee-multi-cycle-same-name.js` (`MissingModuleEnvironment`)
5. `language/module-code/namespace/internals/super-access-to-tdz-binding.js`
6. `staging/sm/TypedArray/toLocaleString.js` [strict + non-strict variants]
7. `staging/sm/class/className.js` [strict + non-strict variants]

Plus the flaky `staging/sm/RegExp/unicode-class-braced.js` timeout
under load.

## Investigation note: first-run flake

The very first post-fix `-j 12` run after rebuild (in a single-binary
warm cargo session that had just built three crates) reported 19
failures, including `harness/deepEqual-array.js`, `Promise/resolve/
resolve-non-obj.js`, and the TypedArray species tests. None reproduced
on subsequent runs or at `-j 1`. Filter runs for the same files passed
cleanly. These appeared to be machine-load-induced timing flakes from
the warm-build state and not semantic regressions from this patch —
all three confirmed via:

- `lyng-test262 --filter harness/deepEqual -j 12`: 14/14 pass × 3
  consecutive runs
- `lyng-test262 -j 1` against the full corpus: matches baseline
  exactly minus the flaky timeout

## Patch-time semantic bug found and fixed

One real bug was caught during this Test262 pass and fixed before the
stable run:

**Bug:** The fast path's FrameRecord builder omitted
`.with_new_target(prepared.new_target)`. For arrow functions invoked
inside `new`-constructed callers, `prepare_bytecode_call` sets
`prepared.new_target` to the enclosing function's `new.target`. Without
propagation, `new.target` inside the arrow read as `undefined` even when
the caller was constructed.

**Symptom:** `language/expressions/arrow-function/lexical-new.target.js`
and `lexical-new.target-closure-returned.js` failed with
`Test262Error` (variant count: 4).

**Fix:** Add `.with_new_target(prepared.new_target)` to the
`install_prepared_bytecode_call_from_registers` frame builder. Matches
the slow path.

**Verification:** Filter run of the four failing variants passes after
the fix. They also pass in the stable full-suite runs.
