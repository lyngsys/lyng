# VM Dispatch Modernization Phase 4/5 Status

## Scope Landed

- Inlined the hot ABC arithmetic/comparison dispatch arms so the run loop no
  longer calls `execute_abc_value_opcode`.
- Removed the old opcode-family rematch helper after splitting the dispatch
  arms into opcode-specific helpers.
- Added checked direct register-window access for `Move`, `LoadConst`, ABC
  result writes, and named-property load/store cache entry points.
- Added SMI fast paths for `Add`, `Sub`, `BitAnd`, and their immediate forms.
- Added guarded SMI fast paths for `Mul` and `Mod` that fall back when an
  ECMA-262 negative-zero result is possible.
- Kept primitive-number fast paths for the non-SMI arithmetic, bitwise,
  equality, and relational cases through opcode-specific helpers.
- Added structural regression tests for hot dispatch shape and direct register
  access, plus negative-zero coverage for specialized SMI arithmetic.

## Correctness Evidence

- `cargo test -p lyng-js-vm`
  - 368 unit tests passed.
  - 0 doctests passed/failed, as expected for the crate.
- `cargo test -p lyng-js-tests`
  - 1,183 unit tests passed.
  - 0 doctests passed/failed, as expected for the crate.
- `cargo clippy -p lyng-js-vm --all-targets -- -W clippy::pedantic -W clippy::nursery`
  - Finished cleanly with no warnings.
- `cargo run --release -p lyng-js-test262 -- --filter built-ins/Object/is --report /tmp/lyng-js-test262-object-is.md -j 4`
  - 21 files selected, 42 variants passed, 0 failed, 0 panicked.
- `cargo run --release -p lyng-js-test262 -- --filter built-ins/Math/sign --report /tmp/lyng-js-test262-math-sign.md -j 4`
  - 5 files selected, 10 variants passed, 0 failed, 0 panicked.
- `cargo run --release -p lyng-js-test262 -- --filter language/expressions/multiplication --report /tmp/lyng-js-test262-mul.md -j 4`
  - 40 files selected, 79 variants passed, 0 failed, 0 panicked.
- `cargo run --release -p lyng-js-test262 -- --filter language/expressions/modulus --report /tmp/lyng-js-test262-modulus.md -j 4`
  - 40 files selected, 79 variants passed, 0 failed, 0 panicked.
  - The planned `language/expressions/remainder` filter is not present in this
    Test262 checkout; the corresponding directory is `modulus`.
- `git diff --check`
  - Clean.

## Assembly Evidence

- `cargo asm --version`
  - `cargo-asm 0.1.16`.
- `CARGO_TARGET_DIR=/private/tmp/lyng-asm-target cargo asm --lib --build-type release "lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::run_dispatch_loop" | wc -l`
  - Emitted 8,581 assembly lines for the release dispatch-loop body.
- `CARGO_TARGET_DIR=/private/tmp/lyng-asm-target cargo asm --lib --build-type release "lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::run_dispatch_loop" | rg -n "execute_abc_value_opcode|read_register|write_register|absolute_register|memcpy|frames|last"`
  - No matches in the freshly built temporary target directory, confirming the
    release dispatch-loop body no longer contains the removed opcode-family
    helper, generic register helper calls, or the old frame-copy chain.

## Next Phase Entry Conditions

- Phase 6 can start from a dispatch loop with local frame state, direct checked
  register access for the hottest value movement paths, and flattened hot
  arithmetic dispatch.
- The remaining work should focus on bytecode density and durable wide
  encoding without reintroducing shared VM program-counter state or generic
  opcode-family rematches in the hot loop.
