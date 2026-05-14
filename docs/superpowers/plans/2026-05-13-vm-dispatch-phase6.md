# VM Dispatch Phase 6 Plan

Issue: `lyng-5xr4`

## Goal

Make the bytecode stream self-describing: a semantic opcode is read directly at
`pc` or after one `Wide` / `ExtraWide` prefix, feedback slots use semantic
profiled opcodes instead of generic envelopes, and call ranges / widened
operands no longer live in side payload storage. Then add the first accumulator
short forms and refresh density/runtime evidence.

## Work Items

- [x] Add red tests for durable prefix decoding, semantic profiled opcodes, and
      absence of legacy `ProfiledAbc` / `ProfiledAbx` envelopes.
- [x] Replace `WideOperand` side storage with inline `Wide` / `ExtraWide`
      operand bytes in the bytecode builder and decoder.
- [x] Inline call ranges into `Call`, `TailCall`, and `Construct` bytecode
      layouts and update VM validation / dispatch.
- [x] Add semantic profiled opcode variants for feedback-capable instructions
      and route `add_feedback_site` through those layouts.
- [x] Delete installed wide-payload maps and legacy dispatch-side merge paths.
- [x] Add accumulator-style short forms for common loads and stores, with VM
      support and builder/compiler emission where safe.
- [x] Update disassembly, density accounting, and opcode-space/report docs.
- [x] Run focused bytecode / VM / compiler / integration tests, targeted
      Test262 slices, density/runtime reports, and `cargo-asm` checks.

## Encoding Decisions

- `Wide` and `ExtraWide` are prefixes only. They are invalid as semantic
  opcodes and cannot stack.
- Narrow ABC / ABx stay compact. Wide ABC/ABx inline high bytes after the
  semantic opcode; ExtraWide is reserved for the same durable layout where the
  current logical operand ranges require it.
- Feedback-capable instructions use semantic `*Profiled` opcode variants with a
  trailing `u16` feedback slot. The same opcode byte never has two lengths.
- General call-like opcodes carry their `CallRange` inline after the base ABC
  operands. Small call opcodes remain range-free.
