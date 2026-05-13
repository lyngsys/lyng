# VM Dispatch Modernization Plan

Date: 2026-05-13. Persisted revision after dispatch-plan review.

This plan supersedes the ad hoc r1-r10 review drafts. It keeps the r10
architecture, with two final clarifications:

- Phase 2b measures total installed-code footprint using the actual side
  metadata representation present before 2b lands. It must not assume a
  fixed 4-byte side-table payload, because today's `WideOperand` is an
  8-byte `{ instruction_offset: u32, payload: u32 }`, and Phase 2a may
  replace that with bitset/rank/payload metadata before 2b.
- Phase 2b "retires" the side table and profiled wrappers from emitted
  bytecode and hot dispatch. Phase 2c deletes legacy compatibility
  structures and disassembler/decoder paths that may remain solely for
  transition and cleanup.

## Goal

- Minimum bar: match QuickJS interpreter performance on the V8 v7 suite
  (Richards, DeltaBlue, Crypto, RayTrace, EarleyBoyer, RegExp, Splay,
  NavierStokes). QuickJS is jitless and is the fair near-peer target.
- Stretch: approach V8 Ignition performance with `--jitless`. V8 Ignition
  is the gold standard for tiered-but-interpreter-only execution and
  contains density and inline-cache patterns we should adopt.

JIT/native code generation is out of scope until interpreter parity is in
hand. We are not targeting V8 with TurboFan enabled.

## Measured Baseline

Local baseline from the current worktree. Higher score means more work done
within the benchmark time budget; where Lyng also exceeds the wall-time budget,
the real slowdown is larger than the score ratio alone.

| Benchmark | Lyng score | QuickJS score | Real slowdown |
| --- | ---: | ---: | ---: |
| Crypto | 133 | 792 | ~24x (4x wall * 6x score) |
| NavierStokes | 305 | 1304 | ~14x |
| Richards | 183 | 912 | ~5.0x |
| DeltaBlue | 217 | 1031 | ~4.8x |
| Splay | 909 | 2082 | ~2.3x |

The synthetic micro-loop suite in `external-engine-compare.md` is near parity
because it sits on warmed ICs and exercises one builtin at a time. It is not
predictive of real workload performance and should not be the primary signal
until the V8 suite gap closes.

## Diagnosis

The dispatch deep dive and external-engine comparison converged on the same
root cause: the hot loop pays too much fixed work before it reaches a semantic
handler. The opcode-mix data is the clearest signal: `Move` is 27-50% of
dispatches on many workloads, and today's `Move` pays the full per-opcode tax
before doing one register copy.

Current fixed per-opcode costs:

1. Per-opcode `Instruction` enum materialization. Every iteration calls
   `decode_instruction_bytes`, validates and classifies the opcode, constructs
   an `Instruction`, then repeatedly pattern-matches it for length, feedback,
   unprofiled form, and operand destructuring.
2. Wide-operand side-table lookup on the hot path. `InstalledFunction::wide_payload`
   currently binary-searches by instruction offset for ABC/ABx decode.
3. Per-opcode `FrameRecord` copy. `*self.frames.last()` copies frame state every
   iteration instead of keeping frame state in local dispatch registers.
4. VM-global PC state. `current_instruction_len`, `advance_instruction`, and
   `jump_by` force handlers to write/read mutable VM state instead of advancing a
   local `pc`.
5. Register access through VM/frame plumbing with bounds checks on every operand.
6. Arithmetic helpers re-match opcode families after dispatch already knows the
   opcode. The SMI add path also round-trips through f64 instead of using checked
   integer arithmetic.
7. Bytecode density. Fixed 4-byte ABC encoding plus no accumulator creates more
   byte stream and more dispatches than V8 Ignition for equivalent work.

QuickJS, V8 Ignition, and JSC LLInt use different implementation techniques, but
they share a shape Lyng currently lacks: the bytecode pointer is local, the next
opcode selects the semantic handler directly, operand decode is handler-local, and
frame state is synced only at calls, returns, throws, suspension, or other
observable boundaries.

## Strategy

Do not start with threaded dispatch. Rust's dense `#[repr(u8)]` opcode match is
not the first bottleneck. First remove the per-opcode tax around the match:

1. Read one opcode byte and dispatch.
2. Move operand decode into the opcode arm.
3. Localize PC, frame, lexical, and register-window state.
4. Refine hot handlers so fast paths are inline and single-purpose.
5. Densify bytecode with accumulator-style short forms.
6. Re-evaluate threaded dispatch only after profiling shows central dispatch is
   the remaining bottleneck.

Each phase advances on profiler artifacts and correctness gates. V8 v7 scores
are recorded as outcomes, not used as brittle per-PR gates.

## Phase 1: Byte-Stream Dispatch

Goal: the inner loop reads one opcode byte and dispatches; `Instruction` is no
longer constructed on the hot path.

### Wide Payload Compatibility

Phase 1 must remain compatible with today's split encoding: narrow operand bytes
live in the byte stream, high bits and call ranges live in `wide_payloads`.
Reading only raw stream bytes would silently truncate widened operands.

Classify opcode side-payload behavior by derivation, not by a hand-maintained
list:

```rust
enum WidePayloadKind {
    Never,       // always-narrow in today's side-table architecture
    Maybe,       // may have a wide payload at this pc
    AlwaysCall,  // CallRange payload is always in the side table
}

fn wide_payload_kind(opcode: Opcode) -> WidePayloadKind;
```

This name is intentionally Phase-1/2a-shaped. It describes the current side
table architecture. Phase 2b replaces it with `OperandWidthKind`.

Back the classifier with bytecode unit tests that walk all `Opcode` variants and
assert behavior against builder emission. `emit_ax` never adds wide operands,
short-form ABx/local opcodes are clamped by construction, and generic ABC/ABx
emission may add a wide operand per instruction offset.

### Tasks

- Add `InstalledFunction::read_opcode_byte(pc)` with safe hot-path mapping from
  opcode byte to `Opcode`. Install-time validation guarantees validity, but the
  runtime path stays safe: `debug_assert!` plus a cold `VmError::CorruptCode`
  branch. No `unsafe` or `unreachable_unchecked`.
- Remove `Instruction::{Abc, Abx, Ax, ...}` form destructuring from the central
  dispatch loop.
- For `WidePayloadKind::Never`, read operands directly from
  `function.instruction_bytes()`.
- For `WidePayloadKind::Maybe`, route through `read_abc_operands` /
  `read_abx_operands`, which consult Phase 2a's presence map and only merges a
  payload when this specific `pc` has one.
- For `WidePayloadKind::AlwaysCall`, fetch the `CallRange::encode()` payload
  directly from the presence map. Install-time validation guarantees it exists.
- Keep transitional `ProfiledAbc` / `ProfiledAbx` support. The profiled
  envelope is decoded without materializing `Instruction`; its inner opcode is
  classified with `wide_payload_kind(inner_opcode)`, and any side payload is
  looked up at the envelope's `pc`, not `pc + 1`.
- Make `wide_payload_kind(ProfiledAbc | ProfiledAbx)` reject or panic with
  "decode the inner opcode first"; profiled wrappers are not semantic opcodes.
- Introduce a local `pc: u32` in the dispatch loop. Advance with per-opcode
  length constants or jump deltas. Keep `current_instruction_len`,
  `advance_instruction`, and `jump_by` until Phase 3 because suspend/resume,
  direct eval, debugger, and call/throw helpers still observe PC through frame
  state.
- Keep `Instruction`, `decode_instruction_bytes`, and form classification for
  validation, disassembly, and tests.

### Ordering

Phase 2a-prereq must land before maybe-wide arms move to the new helpers.
Always-narrow, always-call, profiled-envelope decode, and local-PC plumbing can
land before or after 2a-prereq.

### Verification

- `cargo test -p lyng-js-vm`.
- Filtered Test262 slices for `built-ins/Math`, `language/expressions`, and
  `language/statements/for`.
- `lyng-js-bench runtime --count-opcodes` unchanged.
- Profiles for `richards.js` and `navier-stokes.js` show
  `decode_instruction_bytes`, `instruction_form`, `Instruction::encoded_len`,
  and `Instruction::without_feedback_slot` absent from the top hot path.
- `wide_payload` calls fall to actual widened instruction count. Always-narrow
  opcodes never query it.

## Phase 2: Wide Operand And Metadata Encoding

Goal: ABC/ABx dispatch never binary-searches for operands, and the durable
encoding no longer needs a side table or generic profiled wrappers.

Phase 2 has three sub-phases. 2a is the immediate performance step. 2b is a
larger encoding cleanup scheduled with Phase 6, not immediately after 2a. 2c is
delete-only cleanup.

### Phase 2a: Exact Byte-Offset Presence Map

Replace sorted `wide_payloads` binary search with exact byte-offset addressing.
`pc >> 2` is invalid because instruction widths are variable.

Use explicit presence, not a `0` sentinel. `CallRange::encode()` can be zero
when base and count are both zero.

Default shape:

```rust
struct WidePayloadMap {
    bitset: Box<[u64]>,
    rank: Box<[u32]>,
    payloads: Box<[u32]>,
}
```

Lookup:

```rust
let word = pc as usize / 64;
let bit = pc as usize % 64;
let mask = 1u64 << bit;
if bitset[word] & mask == 0 {
    return None;
}
let prefix_in_word = bitset[word] & (mask - 1);
let index = rank[word] + prefix_in_word.count_ones();
Some(payloads[index as usize])
```

This gives O(1) presence check and O(1) payload fetch. The fallback simple shape
is bitset plus dense `u32` array keyed directly by byte offset. Do not use
`Box<[Option<u32>]>`; `Option<u32>` has no niche and costs 8 bytes per entry.

2a-prereq lands under the existing dispatcher by switching
`InstalledFunction::wide_payload` to the new map. 2a-consume happens during
Phase 1c when the new operand helpers query the map directly.

### Phase 2b: Durable Encoding Change, Scheduled With Phase 6

Retire side-payload encoding and generic profiled wrappers from emitted bytecode
and hot dispatch. The byte stream must contain enough information for dispatch
to find the semantic opcode and compute the next instruction offset without any
side table.

Required invariants:

- Semantic-op locatability: every instruction has one 1-byte semantic opcode,
  optionally preceded by at most one prefix byte. The semantic opcode is at `pc`
  or `pc + 1`. The bounded prefix set is exactly `{Wide, ExtraWide}`. Prefixes
  do not stack. Profiling, feedback, and operand metadata are never prefixes.
- Length self-description: `pc_next = pc + encoded_len(prefix?, semantic_op)`
  using only the optional prefix and semantic opcode. Operand bytes, feedback
  slots, and trailing metadata are included in that length, but shape never
  depends on hidden per-instruction side metadata.

The wide-operand decision is `Wide` / `ExtraWide` prefix encoding. Per-opcode
wide variants would put too much pressure on the 1-byte opcode space once
profiled variants are added.

Feedback encoding options:

1. Semantic profiled variants, preferred if opcode-space audit confirms enough
   headroom. Examples: `AddProfiled`, `LoadGlobalProfiled`,
   `GetNamedPropertyProfiled`, `CallProfiled`. Plain variants carry no slot;
   profiled variants carry a `u16` slot at a fixed trailing offset.
2. Always-reserved feedback bytes, fallback if opcode space becomes too tight.
   Feedback-capable opcodes always reserve a `u16` slot, with
   `FeedbackSlotId::NONE` at unprofiled sites.

Forbidden shape: the same opcode byte sometimes carries feedback bytes and
sometimes does not without a variant distinguishing the layouts.

Classifier rename is atomic in 2b. The old side-table classifier is replaced:

```rust
enum OperandWidthKind {
    Fixed,
    WideCapable,
}

fn operand_width_kind(opcode: Opcode) -> OperandWidthKind;
```

Call layout becomes inline operand bytes plus a small static table keyed by
opcode where needed. `AlwaysCall` disappears because there is no side payload
left to pair with. `WidePayloadKind` / `wide_payload_kind` are deleted in the
same PR as the 2b encoding swap; no compatibility shim crosses PR boundaries.

Important measurement rule: 2b is architecture cleanup, not a byte-stream
density win by itself. Today's widened instruction is 4 stream bytes plus actual
installed side metadata; after 2b it becomes inline stream bytes with no side
metadata. The byte stream may grow on widened sites. The gate is installed-code
footprint, counting the actual side metadata representation that exists before
2b lands, not a fixed 4-byte assumption.

### Phase 2c: Delete Legacy Compatibility

After 2b stops emitting and hot-dispatching legacy encodings, remove any
remaining compatibility structures and tests:

- `wide_payloads` storage and `WideOperand`.
- `decode_abc_operands` / `decode_abx_operands` legacy side-payload merge.
- `Instruction::ProfiledAbc` / `Instruction::ProfiledAbx`.
- The profiled-wrapper dispatch arm.
- Disassembler and decoder special cases for the generic profiled envelope.
- Profiled-wrapper unit tests.

2c does not handle the classifier rename; 2b already deleted the old classifier.

### Verification

2a:

- Bytecode and disassembly are bytewise-identical to pre-2a output.
- Opcode mix is unchanged.
- The old binary search no longer appears in profiles.
- Test262 unchanged.

2b:

- Disassembler round-trip: compile -> emit bytes -> decode -> disassemble ->
  re-encode produces bytewise-identical byte streams.
- Validator rejects malformed prefix/opcode/feedback combinations.
- Positive validator/decoder tests cover `Wide AddProfiled`,
  `Wide LoadGlobalProfiled`, `Wide CallProfiled`, `ExtraWide AddProfiled`,
  `ExtraWide GetNamedPropertyProfiled`, and a case where all operands widen and
  the trailing feedback slot remains at the expected fixed offset. If the
  always-reserved feedback option is chosen, run the same composition tests
  against the non-profiled semantic opcodes with `FeedbackSlotId::NONE`.
- Before/after installed-footprint report counts byte stream plus actual
  pre-2b side metadata. Byte-stream-only density is tracked but not a hard
  2b gate.
- Opcode-space audit is committed with the change.
- Dispatch and validators consume `operand_width_kind`, not `wide_payload_kind`.
- Whole-corpus Test262, plus Phase 4 negative-zero slices if numeric fast paths
  have already landed.

2c:

- Compile-time absence of legacy data structures and envelope special cases.
- `cargo test`, `cargo clippy`, and Test262 unchanged.

## Phase 3: Localize Frame State

Goal: the inner loop holds local dispatch state and frame metadata. Per-opcode
`self.frames.last()` and `FrameRecord` copies disappear. VM-level PC APIs are
deleted once helpers no longer read PC from frame state.

### Register-Slice Borrow Strategy

Holding `&mut [Value]` across a dispatch arm conflicts with helpers needing
`&mut self`. `unsafe` is forbidden in Lyng JS crates, so raw-pointer register
access is out unless repository policy changes.

Prototype and choose between:

- Disjoint-state split, preferred: split register storage into a sibling
  `Registers` state and pass `&mut Registers` plus `&mut VmRest` to dispatch.
  Fast arms touch registers only; cold helpers re-acquire register access
  through a narrow accessor.
- Bounded-borrow fast arms: fast arms borrow the register slice only for the arm
  and never call helpers; cold arms keep today's `&mut self` helper path.

### Tasks

- Introduce `DispatchState { pc, lexical_env, this, construct_this,
  handler_cursor, flags, resume_kind, resume_value, resume_active }`.
- Hoist frame metadata once per activation: code, register window, parameter
  offset, realm, variable env, callee, new target, and frame kind.
- Implement the chosen safe register strategy and document invariants.
- Sync `DispatchState` back to frames only at calls, tail calls, constructs,
  returns, throws, yield/await/generator suspension, exception transfer, debug
  safepoints, deopt capture, and dynamic eval entry.
- Convert helpers from `&FrameRecord` to narrower inputs.
- Delete `Vm::current_instruction_len`, `Vm::advance_instruction`, and
  `Vm::jump_by`.

### Verification

- Whole-corpus Test262 unchanged.
- Runtime benchmarks show no compiler-density regressions.
- Profiles show `self.frames.last()` and register `Vec` indexing gone from
  per-opcode hot paths where expected.
- `cargo asm` confirms the `FrameRecord` copy chain is gone from the loop body.

## Phase 4: Per-Opcode Handler Refinement

Goal: hottest opcodes have inline fast paths and no redundant opcode matching.

### Tasks

- Inline `Move`: `regs[a] = regs[b]; pc += len; dispatch`.
- Inline SMI-safe `AddSmi`, `SubSmi`, `BitAndSmi`, `Add`, and `Sub`.
  Successful checked integer operations store `Value::from_smi`. Overflow falls
  through to the existing f64/spec path.
- Treat `MulSmi`, `Mul`, `ModSmi`, and `Mod` carefully for negative zero.
  Multiplication falls through to f64 on `prod == 0 && (a < 0 || b < 0)`.
  Modulo falls through on zero divisor, checked-rem overflow, or
  `result == 0 && a < 0`.
- Leave `DivSmi` / `Div` on the existing f64 path unless measurement and tests
  justify an exact-integer fast path.
- Flatten `execute_abc_value_opcode`; dispatch already knows the opcode.
- Inline `LoadConst` warm path when no GC slot is present.
- Inline warmed mono-IC fast paths for `LoadGlobal` and `GetNamedProperty`
  while keeping feedback data structures and warmup semantics unchanged.

### Verification

- Test262 unchanged, especially multiplication/modulo negative-zero tests,
  `built-ins/Math/sign`, `built-ins/Object/is`, and tests observing
  `1 / x === -Infinity`.
- Profiles no longer show `execute_abc_value_opcode`,
  `execute_smi_immediate_opcode`, or `try_primitive_number_binary_opcode` in
  the per-opcode hot path.
- `cargo asm` confirms SMI add fast path does not call `encode_number` or f64
  conversion.

## Phase 5: Register Access

Goal: hot register reads/writes compile to direct loads/stores with no per-access
branches.

### Tasks

- Hold a typed `&mut [Value]` register window within an activation via the safe
  Phase 3 strategy.
- Use checked indexing only; `get_unchecked` is forbidden by the Lyng JS
  `unsafe` policy.
- Hoist register base out of `RegisterWindow`; do not call
  `RegisterWindow::base()` per operand.
- Audit multi-register handlers (`Move`, `Add`, `GetKeyedProperty`) and combine
  reads where it helps LLVM eliminate bounds checks.

### Verification

- Test262 unchanged.
- `cargo asm` for the `Move` arm shows one load, one store, one PC advance, and
  one dispatch branch with no bounds-check branch in between.

## Phase 6: Bytecode Density

Goal: average byte stream per JavaScript operation drops by 30-40%, and `Move`
falls below 20% of dispatches by routing short-lived temporaries through an
accumulator-like path.

Phase 2b is scheduled with this phase because compiler, validator, disassembler,
and VM encoding all move together. Phase 2b itself is not the density win; the
accumulator and short-store work is.

### Tasks

- Add an implicit accumulator or fixed implicit register for short-form loads:
  `LdaUndefined`, `LdaSmi8`, `LdaConst8`, `Ldar`, and related forms.
- Add `Star0..StarN` short stores from accumulator to common registers.
- Update the compiler/register allocator to route short-lived temporaries
  through accumulator forms.
- Land the durable `Wide` / `ExtraWide` prefix encoding and profiled-variant
  cleanup from Phase 2b.
- Regenerate bytecode-density reports.

### Verification

- Test262 unchanged.
- `lyng-js-bench density` shows representative byte streams down 30-40%.
- `--count-opcodes` shows `Move` below 20%.
- V8 v7 scores and wall times recorded against QuickJS.

## Phase 7: Threaded Dispatch, Conditional

Only evaluate after Phase 6 if profiles show central `match` dispatch remains
the bottleneck.

Safe-Rust options:

- Wait for stable Rust `become` / explicit tail calls, then one handler function
  per opcode with a static dispatch table.
- Macro replication: each opcode arm ends with its own replicated next-opcode
  match, giving opcode-pair predictor entries while staying on stable Rust.

Inline asm and computed-goto shims are out of scope unless the no-`unsafe` policy
changes.

## Verification Stack

Correctness gates before merging phase work:

- `cargo test -p lyng-js-vm -p lyng-js-bytecode -p lyng-js-compiler -p lyng-js-tests`.
- `cargo clippy --all-targets --all-features -- -W clippy::pedantic -W clippy::nursery`.
- Filtered Test262 for the changed area; whole-corpus at phase boundaries.
- `lyng-js-bench runtime --report` for hot-path changes, with no unexplained
  unrelated workload regressions.

Performance evidence before phase exit:

- Named profiler artifact for the phase.
- `cargo asm` artifact for representative arms, especially `Move`.
- V8 v7 scores and wall times for Richards, DeltaBlue, Crypto, NavierStokes,
  and Splay through `target/release/lyng-js` and `qjs`.

## Reporting Cadence

Each phase exit writes a status report under `reports/js/lyng-js/` describing
what landed, measured deltas, profiler artifacts, and next-phase entry
conditions. Regenerate `bench.md` and `external-engine-compare.md` at phase
exits.

## Open Questions

- Does Rust stabilize `become` in time for Phase 7?
- Should Phase 1 land as one large PR or one PR per opcode family?
- How much of the mono-IC fast path should live in dispatch arms versus
  `feedback.rs` helpers? Decide by measurement.

## Bottom Line

The dispatch loop has the right central-match idea, but too much work happens
before and around the match. Strip instruction materialization, remove hot
side-table searches, localize state, then densify bytecode. Threaded dispatch is
a last-mile optimization, not the starting point.
