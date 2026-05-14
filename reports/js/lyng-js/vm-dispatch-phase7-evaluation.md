# VM Dispatch Phase 7 Evaluation

Issue: `lyng-10ws`

## Decision

Do not start direct-threaded dispatch work yet.

Post-Phase-6 evidence does not identify the central Rust `match` as the
remaining bottleneck. The release assembly already lowers the opcode match to a
jump table on aarch64, and the post-Phase-6 runtime sample points at decode,
call, RegExp, property, and allocation work before it points at the branch
mechanism itself.

## Evidence

Artifacts:

- Runtime profile-target report:
  `reports/js/lyng-js/vm-dispatch-phase7-profile-runtime.md`
- Runtime profile-target JSON:
  `reports/js/lyng-js/vm-dispatch-phase7-profile-runtime.json`
- macOS `sample` artifact:
  `reports/js/lyng-js/vm-dispatch-phase7-runtime.sample.txt`
- Phase 6 release assembly check:
  `cargo asm --lib --build-type release --no-color 'lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::run_dispatch_loop'`

Commands:

- `CARGO_TARGET_DIR=/private/tmp/lyng-target-phase6-current /private/tmp/lyng-target-phase6-current/release/lyng-js-bench runtime --preset profile-target --count-opcodes --report /tmp/lyng-js-phase7-profile-runtime.md --json /tmp/lyng-js-phase7-profile-runtime.json`
- `sample <pid> 5 -file reports/js/lyng-js/vm-dispatch-phase7-runtime.sample.txt`
- `CARGO_TARGET_DIR=/private/tmp/lyng-target-phase6-current cargo asm --lib --build-type release --no-color 'lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::run_dispatch_loop'`

Profile-target runtime settings:

- Samples per benchmark: `1`.
- Warmup runs per sample: `1`.
- Timed runs per sample: `1`.
- Runtime loop trips: `32768`.
- Opcode dispatch counters: enabled.
- Slowest row: `regexp-heavy.runtime` at `48398.85` ns/work-unit.
- Next slow RegExp row: `regexp-stable-exec.runtime` at `39580.14`
  ns/work-unit.

Sample highlights:

- Main sampled call tree: `3759` samples.
- `run_dispatch_loop` under script evaluation: `1045` samples.
- Most of that branch immediately goes through `call_value_small` and then
  RegExp builtin dispatch: `1035` samples under `call_value_small`, `829`
  samples under `dispatch_regexp_builtin`.
- Top-of-stack collapsed entries include:
  - `decode_dispatch_instruction`: `252` samples.
  - `run_dispatch_loop`: `131` samples for one dispatch-loop symbol and `80`
    samples for another.
  - `RegExpLegacyStaticState::record_match`: `46` samples.
  - `regexp_exec_state`: `37` samples.
  - `execute_get_named_property_opcode`: `29` samples.
  - `try_primitive_number_binary_opcode`: `28` samples.

Assembly highlights:

- The dispatch-loop assembly contains a jump table label: `LJTI455_0`.
- It loads a table entry and dispatches with indirect branch `br x11`.
- The decode helper still appears before the jump:
  `decode_dispatch_instruction`.

This means a direct-threading experiment would not be addressing a clearly
measured branch-dispatch bottleneck. The current profile points first at
instruction decode shape and semantic runtime work.

## Safe Rust Options

`become` / explicit tail calls:

- Local compiler: `rustc 1.93.1 (01f6ddf75 2026-02-11)`.
- Probe result: `become` is still experimental and rejected with `E0658`.
- This option is not available on stable Rust in this workspace.

Macro-replicated next-opcode matches:

- Feasible in safe Rust.
- Expected cost: substantial code size, harder review, and duplicated handler
  structure.
- Not justified by the current evidence because the central match already
  lowers to a jump table and the sample does not isolate it as the bottleneck.

Inline asm / computed goto:

- Still out of scope under the Lyng JS no-unsafe policy.

## Recommendation

Keep Phase 7 as an evaluation-only result. If dispatch is revisited later, first
capture a narrower profile where most samples stay inside simple bytecode
handlers and `decode_dispatch_instruction` or the jump-table branch dominates.
Only then prototype a safe-Rust macro-replicated next-opcode match on a separate
branch and compare runtime, code size, `cargo asm`, and Test262 correctness
before considering implementation.
