# VM Dispatch Phase 6 Status

Issue: `lyng-5xr4`

## Summary

Phase 6 replaces the bytecode side-channel widening scheme with a
self-describing byte stream. Instructions now carry at most one `Wide` or
`ExtraWide` prefix, call ranges are inline in the call instruction layouts, and
feedback-capable instructions use semantic profiled opcodes instead of generic
`ProfiledAbc` / `ProfiledAbx` envelopes.

The builder and VM also emit and execute accumulator-oriented short forms for
common register-zero loads and stores. Installed bytecode no longer carries a
wide-payload map, and dispatch decodes the instruction stream directly from the
encoded bytes.

During verification, the full Test262 run exposed one branch-only regression in
derived constructors that close a sync iterator while `return()` calls an arrow
capturing `super()`. The fix synchronizes the active dispatch frame around the
sync iterator-close bridge so nested user code can initialize the derived
constructor receiver without being overwritten by the resumed dispatch frame.

## Opcode Space

- Highest assigned opcode: `ConstructProfiled` at discriminant `203`.
- `OPCODE_COUNT`: `204`.
- Remaining `u8` opcode slots: `52`.
- Prefix opcodes: `Wide`, `ExtraWide`.
- Legacy envelope/storage terms absent from active source: `WideOperand`,
  `wide_operands`, `wide_payload`, `WidePayload`, `ProfiledAbc`,
  `ProfiledAbx`, `MissingWideOperand`.

## Reports

- Density: `reports/js/lyng-js/bytecode-density-aarch64.md`
  - Aggregate unit bytes: `4553`.
  - Aggregate wide share: `14.96%`.
  - Non-stress workloads: `0.00%` wide.
  - Large-register stress workload: `3267` unit bytes, `20.49%` wide.
- Runtime: `reports/js/lyng-js/bench.md`
  - Opcode dispatch counters: enabled.
  - Slowest row: `regexp-heavy.runtime` at `47117.67` ns/work-unit.
  - Largest template footprint: `class-heavy.runtime` at `9174` bytes.
  - Heaviest runtime snapshot: `runtime.regexp-literal-cache` at `256750`
    live bytes.
- External comparison: `reports/js/lyng-js/external-engine-compare.md`
  - Corpus: `v8-v7`.
  - Filter: `richards`.
  - Lyng JS: completed, score median `199.000`.
  - QuickJS: completed, score median `971.000`.
  - Boa: failed during external engine startup; the report records the crash.

## Verification

- `cargo fmt --all`: passed.
- `CARGO_TARGET_DIR=/private/tmp/lyng-target-phase6-current cargo test -p lyng-js-bytecode -p lyng-js-vm -p lyng-js-compiler -p lyng-js-tests`: passed.
- `CARGO_TARGET_DIR=/private/tmp/lyng-target-phase6-current cargo clippy -p lyng-js-bytecode -p lyng-js-vm -p lyng-js-compiler -p lyng-js-tests -p lyng-js-test262 -p lyng-js-bench --all-targets -- -W clippy::pedantic -W clippy::nursery`: passed.
- `CARGO_TARGET_DIR=/private/tmp/lyng-target-phase6-current cargo test -p lyng-js-tests phase6_derived_constructor_iterator_close_can_initialize_this_before_return`: passed.
- `CARGO_TARGET_DIR=/private/tmp/lyng-target-phase6-current cargo run --release -p lyng-js-test262 -- --filter language/statements/class/subclass/derived-class-return-override-for-of-arrow.js --report /tmp/lyng-js-current-derived-class-fixed2.md --list-failures -j 1`: passed, `2` variants.
- `CARGO_TARGET_DIR=/private/tmp/lyng-target-phase6-current cargo run --release -p lyng-js-test262 -- --filter language/statements/for-await-of/async-func-decl-dstr-array-elem-init-assignment.js --report /tmp/lyng-js-for-await-smoke.md --list-failures -j 1`: passed, `2` variants.
- `CARGO_TARGET_DIR=/private/tmp/lyng-target-phase6-current cargo run --release -p lyng-js-test262 -- --report reports/js/lyng-js/test262.md -j 12`: passed with `49724 / 49729` runnable files and `95200 / 95205` runnable variants passing. Remaining failures are the existing import-defer/module-environment/module-namespace clusters recorded in the report.
- `CARGO_TARGET_DIR=/private/tmp/lyng-target-phase6-current cargo run --release -p lyng-js-bench -- density --report reports/js/lyng-js/bytecode-density-aarch64.md --json reports/js/lyng-js/bytecode-density-aarch64.json`: passed.
- `CARGO_TARGET_DIR=/private/tmp/lyng-target-phase6-current cargo run --release -p lyng-js-bench -- runtime --count-opcodes --report reports/js/lyng-js/bench.md --json reports/js/lyng-js/bench.json`: passed.
- `CARGO_TARGET_DIR=/private/tmp/lyng-target-phase6-current cargo run --release -p lyng-js-bench -- compare --corpus v8-v7 --filter richards --preset smoke --timeout-ms 5000 --report reports/js/lyng-js/external-engine-compare.md --json reports/js/lyng-js/external-engine-compare.json`: completed and wrote the report. Earlier unfiltered `v8-v7` compare attempts were stopped after long-running Lyng JS children.
- `CARGO_TARGET_DIR=/private/tmp/lyng-target-phase6-current cargo asm --lib --build-type release --no-color 'lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::run_dispatch_loop'`: passed. The assembly contains the `LJTI455_0` jump table and indirect `br x11` dispatch branch after `decode_dispatch_instruction`.
