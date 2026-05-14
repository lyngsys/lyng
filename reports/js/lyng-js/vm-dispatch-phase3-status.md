# VM Dispatch Modernization Phase 3 Status

## Scope Landed

- Added explicit dispatch-local frame state via `DispatchState`, initialized once
  per active frame and refreshed only when the active frame changes.
- Moved dispatch PC advancement and jumps to local `FrameRecord` helpers:
  `advance_dispatch_frame`, `jump_dispatch_frame`, and
  `next_dispatch_instruction_offset`.
- Removed VM-global PC state and APIs:
  `Vm::current_instruction_len`, `Vm::advance_instruction`, and `Vm::jump_by`.
- Threaded frame-depth plus local frame state through call, construct, property,
  async, generator, iterator, with-environment, debugger, return, throw, and
  suspension boundaries.
- Synced local dispatch state back to `self.frames` only at observable
  boundaries: bytecode call entry, host/builtin/proxy calls, tail calls,
  constructor fallback, direct eval/dynamic name paths, returns, throws,
  exception transfer, await/yield suspension, async iterator close suspension,
  generator resumption, and debugger safepoints.
- Added structural regression coverage that keeps the hot dispatch loop from
  reintroducing per-op `FrameRecord` copies or VM-global PC helpers.

## Correctness Evidence

- `cargo test -p lyng-js-vm`
  - 363 unit tests passed.
  - 0 doctests passed/failed, as expected for the crate.
- `cargo clippy -p lyng-js-vm --all-targets -- -W clippy::pedantic -W clippy::nursery`
  - Finished cleanly with no warnings.
- `cargo run --release -p lyng-js-test262 -- --filter language/statements/async-generator --report /tmp/lyng-js-test262-async-generator.md -j 4`
  - 301 files selected, 590 variants passed, 0 failed, 0 panicked.
- `cargo run --release -p lyng-js-test262 -- --filter language/eval-code/direct --report /tmp/lyng-js-test262-direct-eval.md -j 4`
  - 286 files selected, 336 variants passed, 0 failed, 0 panicked.
- `cargo run --release -p lyng-js-test262 -- --filter language/statements/for-await-of --report /tmp/lyng-js-test262-for-await-of.md -j 4`
  - 1,234 files selected, 2,431 variants passed, 0 failed, 0 panicked.
- `git diff --check`
  - Clean.

## Performance Evidence

- `cargo run --release -p lyng-js-bench -- runtime --report /tmp/lyng-js-bench-phase3-runtime.md`
  - Report completed successfully.
  - Slowest row: `regexp-heavy.runtime` at `48367.91` ns/work-unit.
  - Largest template footprint: `class-heavy.runtime` at `9534` bytes.
  - Heaviest runtime snapshot: `runtime.regexp-literal-cache` at `256750`
    live bytes.
- `cargo asm --version`
  - `cargo-asm 0.1.16`.
- `cargo asm --lib --build-type release "lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::run_dispatch_loop" | rg -n "memcpy|frames|last"`
  - No matches, confirming the release dispatch-loop body no longer contains the
    old frame-copy chain.
- `cargo rustc --release -p lyng-js-vm --lib -- --emit=asm`
  - Emitted `target/release/deps/lyng_js_vm-6460d214ef84dee7.s`.
  - The four `run_dispatch_loop` release assembly bodies are at lines
    `124573`, `132481`, `140467`, and `148323`.
  - `rg -n "memcpy" target/release/deps/lyng_js_vm-6460d214ef84dee7.s`
    found no `memcpy` calls inside the dispatch-loop line range
    `124573..156737`; the next `memcpy` after the loop bodies is at line
    `157968`.

## Next Phase Entry Conditions

- Phase 4/5 can start from a dispatch loop where instruction decode and PC
  advancement are local to the active dispatch state.
- Register-window borrowing remains the next major constraint. Phase 3 did not
  hold a long-lived `&mut [Value]` across helper calls, preserving the current
  no-unsafe policy and leaving Phase 5 to tighten register access deliberately.
