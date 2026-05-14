# VM Dispatch Profiled-Merge Plan (Track H)

Issue: `lyng-1o9z` (continuing the dispatch modernization workstream).

Date: 2026-05-14.

## Goal

Collapse the dispatch loop to **one indirect branch per dispatched
opcode** by eliminating the `*Profiled` opcode mirror and making the
feedback slot a normal trailing operand of every IC-shaped opcode.

This is Option 3 from the design conversation. The other two options
(`is_profiled` as a range compare; splitting merged arms so
`is_profiled` is compile-time per arm) were considered and rejected as
local fixes that don't address the underlying design problem: every
IC-shaped opcode currently exists in two forms, and that doubling
propagates into the dispatch loop, the bytecode encoding, and any
future JIT's stub emission.

## Why This Approach

Lyng JS will get a baseline JIT later (per the `architecture.md`
roadmap is interpreter-only; per the `state-of-the-engine.md` strategic
analysis the Sparkplug-style minimum-viable JIT is the only path to
V8/JSC-class performance). The opcode shape this round commits us to
gets consumed by that JIT.

- V8 and JSC both **always allocate a feedback slot** for every
  IC-shaped opcode. There is no "unprofiled Add" in either engine.
- The JIT can then emit one stub per IC-shaped opcode that reads the
  slot unconditionally; if the slot's state is `Uninitialized`, the
  stub falls through to the slow/generic path. No branch on "is there
  a slot at all."
- The interpreter likewise loses one branch on the IC hot path
  (today: `feedback_slot.and_then(...)`; after: the slot is always
  there).
- The current `*Profiled` mirror is a per-opcode binary choice that
  would force the JIT to emit two stubs (or one with an internal
  branch) per IC-shaped op. Bad foundation.

The bytecode-size cost is real and accepted: ~+2 bytes per IC-shaped
instruction, estimated +10-15% bytecode overall. V8 Ignition bytecode
is famously larger than QuickJS's for exactly this reason. The
trade-off pays for itself the moment a JIT exists.

## Design

### Always-allocate, no sentinel

For every IC-shaped opcode (the 46 currently with a `*Profiled`
mirror), the compiler allocates a `FeedbackSlotId` at emission time
and the slot is encoded as a 2-byte trailing operand. There is no
"slot missing" representation. The `FeedbackSlotId(NonZeroU32)` type
is preserved; what changes is that every IC-shaped opcode site
carries a guaranteed-valid slot, rather than an `Option<FeedbackSlotId>`.

### Opcode count

199 → 153 (the 46 `*Profiled` variants are deleted).
`Opcode::ConstructProfiled` was the last variant at byte 198; after
this change, `Opcode::Construct` (or whatever the new last opcode is)
is the last. `OPCODE_COUNT` shrinks accordingly. The `OPCODES` lookup
table in `opcode.rs:208` shrinks to 153 entries. This frees ~46 bytes
of opcode space for future use.

### Encoded sizes (narrow / wide / extra-wide)

Today (4-byte ABC instruction with optional profiled tail):

| Form | Layout | Length |
| --- | --- | ---: |
| Add narrow | `[op][a][b][c]` | 4 |
| AddProfiled narrow | `[op][a][b][c][slot_lo][slot_hi]` | 6 |
| Add wide | `[Wide][op][a_lo][b_lo][c_lo][a_hi][b_hi][c_hi]` | 8 |
| AddProfiled wide | wide + 2 slot bytes | 10 |

After (every IC-shaped opcode has the slot inline; non-IC opcodes
unchanged):

| Form | Layout | Length |
| --- | --- | ---: |
| Add narrow | `[op][a][b][c][slot_lo][slot_hi]` | 6 |
| Add wide | `[Wide][op][a_lo][b_lo][c_lo][a_hi][b_hi][c_hi][slot_lo][slot_hi]` | 10 |
| Move narrow | `[op][a][b][c]` | 4 (unchanged) |
| Move wide | `[Wide][op][a_lo][b_lo][c_lo][a_hi][b_hi][c_hi]` | 8 (unchanged) |

`Move`, `LoadConst`, `Jump`, `Return`, exception ops, scope ops, and
the rest of the non-IC opcodes keep their current encoding.

### Instruction enum simplification

`crates/lyng-js/bytecode/src/instruction.rs:12-49` today has six
variants:

```rust
pub enum Instruction {
    Abc { opcode, a, b, c },
    Abx { opcode, a, bx },
    Ax  { opcode, ax },
    FeedbackAbc { opcode, a, b, c, slot },
    FeedbackAbx { opcode, a, bx, slot },
    CallRange { opcode, a, b, c, range, slot: Option<FeedbackSlotId> },
}
```

After:

```rust
pub enum Instruction {
    Abc      { opcode, a, b, c },                            // non-IC opcodes
    Abx      { opcode, a, bx },                              // non-IC opcodes
    Ax       { opcode, ax },                                 // jumps
    AbcSlot  { opcode, a, b, c, slot: FeedbackSlotId },      // IC-shaped ABC
    AbxSlot  { opcode, a, bx, slot: FeedbackSlotId },        // IC-shaped ABX
    CallRange{ opcode, a, b, c, range, slot: FeedbackSlotId },// always slot
}
```

The renaming `Feedback*` → `*Slot` is cosmetic but reads more naturally
("opcode with a slot operand" vs "opcode that uses feedback"). The
`Option<FeedbackSlotId>` on `CallRange` becomes unconditional
`FeedbackSlotId`. `with_feedback_slot` is deleted.

### Dispatch preamble simplification

Today (`dispatch.rs:743-744`):

```rust
let opcode = semantic_opcode.profiled_base_opcode();
let is_profiled = semantic_opcode.is_profiled();
```

After: both lines deleted. `decode_abc_operands` and friends drop the
`is_profiled: bool` parameter; each opcode arm calls the correct
decoder for its own operand layout (the per-opcode-arm operand
read is already inline post-Track-E, so this is a cleanup of
parameter passing only).

The merged arms `Opcode::Add | Opcode::AddProfiled => ...` collapse to
`Opcode::Add => ...`. Same logic, no merge, no dynamic profiled flag.

`Opcode::profiled_base_opcode()` and `Opcode::is_profiled()` are
deleted from the public API. Last callsites are in the dispatch
preamble being removed.

## Phased Execution

Each phase ends with a green build, full `cargo test`, and Test262
within the documented baseline (49722-49724 / 49729 — verify the live
report at the start of work; the handoff notes some variance from
intermittent RegExp staging timeouts).

### Phase 1 — Opcode enum and lookup table (bytecode crate, leaf)

Files: `crates/lyng-js/bytecode/src/opcode.rs`.

- Delete the 46 `*Profiled` variants from the `Opcode` enum (lines
  ~157-203).
- Delete `Opcode::profiled_base_opcode()` (lines 697-748).
- Delete `Opcode::is_profiled()` (lines 687-690).
- Trim the `OPCODES` lookup table from 199 to 153 entries.
- Update `encoded_len()` (lines ~423-457): every former `*Profiled`
  variant's length is folded into its base opcode's arm. E.g.
  `Opcode::Add => 6` (was 4), `Opcode::Move => 4` (unchanged).
- Update the `from_byte_round_trips_opcode_discriminants` test in
  `opcode.rs:887` to round-trip the new shrunken set.
- Add a const assert that `OPCODE_COUNT < 256` (trivial today;
  documents the invariant).

Verification:
- `cargo build -p lyng-js-bytecode` — green.
- `cargo test -p lyng-js-bytecode` — green (some tests will fail
  pending Phase 2-3; pin those to Phase 3).

### Phase 2 — Instruction enum, builder, decoder

Files: `crates/lyng-js/bytecode/src/instruction.rs`,
`crates/lyng-js/bytecode/src/builder.rs`,
`crates/lyng-js/bytecode/src/decoder.rs`.

- Rename `FeedbackAbc` → `AbcSlot`, `FeedbackAbx` → `AbxSlot` (consistent
  with V8's naming for "operand of kind slot"). Slot field becomes
  required (not `Option`).
- `CallRange::slot` becomes `FeedbackSlotId` (was `Option<FeedbackSlotId>`).
- Delete `Instruction::with_feedback_slot()` and the
  `instruction.with_feedback_slot(*slot)` call at `builder.rs:1291`.
- Builder methods for IC-shaped opcodes (`add()`, `sub()`,
  `get_named_property()`, etc.) take a `FeedbackSlotId` parameter
  directly. Non-IC builder methods (`move_()`, `load_const()`, etc.)
  unchanged.
- `decoder.rs:464,485,513,518` tests: drop `Opcode::*Profiled`
  references; replace with the unified opcode and assert the trailing
  slot bytes decode correctly.
- `instruction.rs:652` test: same.

Verification:
- `cargo build -p lyng-js-bytecode` — green.
- `cargo test -p lyng-js-bytecode` — green.
- Disassembler snapshot tests (if any) — regenerate.

### Phase 3 — VM dispatch loop

File: `crates/lyng-js/vm/src/vm/dispatch.rs`.

- Delete `let opcode = semantic_opcode.profiled_base_opcode();`
  (line 743).
- Delete `let is_profiled = semantic_opcode.is_profiled();` (line 744).
- Drop the `is_profiled: bool` parameter from `decode_abc_operands`,
  `decode_abx_operands`, `decode_abx8_operands`, `decode_ax_operands`,
  and `decode_feedback_slot_operand`. Replace with two variants:
  `decode_abc_operands_with_slot` (mandatory slot) and
  `decode_abc_operands` (no slot, for non-IC opcodes). Same for ABX.
- Drop the merged arms `Opcode::Add | Opcode::AddProfiled => ...`
  pattern. Each IC-shaped opcode becomes one arm, calling the
  `_with_slot` decoder.
- Update the structural regression tests (lines 14-129) to:
  - Add an assertion that `profiled_base_opcode` is not called from
    `run_dispatch_loop`.
  - Add an assertion that `is_profiled` is not called from
    `run_dispatch_loop`.
  - Keep the existing assertions intact.

Verification:
- `cargo build --release -p lyng-js-cli` — green.
- `cargo test -p lyng-js-vm dispatch_loop_` — all structural tests
  pass, including the two new ones.
- `cargo asm --release 'lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::run_dispatch_loop'`
  — exactly **one** `br x<N>` per dispatch path. Capture the new
  disassembly for the eventual status report.

### Phase 4 — Compiler

Files: `crates/lyng-js/compiler/src/script/*.rs`.

Grep currently shows zero `Profiled` references in the compiler crate,
so this is primarily an API-flow audit:

- Every emission site for an IC-shaped opcode should now thread a
  `FeedbackSlotId` through to the builder. Confirm via grep that no
  call site is still using a two-step "emit base, then attach slot"
  pattern.
- If any compiler site previously relied on
  `with_feedback_slot`-style optionality, refactor to always allocate.

Verification:
- `cargo test -p lyng-js-vm -p lyng-js-bytecode -p lyng-js-objects -p lyng-js-tests`:
  full pass.
- `cargo clippy -p lyng-js-vm -p lyng-js-objects --all-targets -- -W clippy::pedantic -W clippy::nursery`:
  clean.

### Phase 5 — Benchmarks and reports

- V8 v7 sweep, 3-sample minimum. Reproduce per the handoff doc's
  script. Target: ≥ previous Track-E baseline on every workload;
  expect +5-12% on dispatch-bound workloads (Richards, DeltaBlue,
  Crypto).
- Test262 full corpus (`-j 4` for the documented run):
  `cargo run --release -p lyng-js-test262 -- --report /tmp/t262.md -j 4`.
  Must hold at the current baseline.
- Bytecode-density sweep:
  `reports/js/lyng-js/bytecode-density-aarch64.md` exists — regenerate
  and document the +10-15% size increase honestly.
- Update `reports/js/lyng-js/vm-dispatch-fixup-status.md` with this
  round's numbers, the offset-level disassembly evidence that the
  second indirect branch is gone, and a "what this round bought us"
  paragraph that doesn't overclaim.
- Capture before/after `cargo asm` of `run_dispatch_loop` in the
  status report.

## Verification Summary

| Check | Phase | Criterion |
| --- | --- | --- |
| Build clean | every | `cargo build --release -p lyng-js-cli` green |
| Tests pass | every | `cargo test -p lyng-js-vm -p lyng-js-bytecode -p lyng-js-objects -p lyng-js-tests` 100% |
| Clippy clean | every | `cargo clippy ... -W clippy::pedantic -W clippy::nursery` clean |
| Test262 baseline | 3, 5 | 49722-49724 / 49729 maintained |
| Single indirect branch | 3, 5 | `cargo asm` shows one `br x<N>` per dispatch path |
| V8 v7 no regression | 5 | All 6 workloads ≥ current baseline |
| Bytecode density honest | 5 | Density report updated, +10-15% accepted and noted |
| Disassembly archived | 5 | Pre and post `run_dispatch_loop` disasm in status report |

## Risks

1. **icache regression on dispatch loop.** Each arm grows by ~2 bytes
   of operand decode (slot bytes). The total loop body grows. The
   same risk killed Track B's `record_feedback_slot` inlining attempt
   (−1% Richards, −2.5% Crypto, −2% NavierStokes).
   *Mitigation:* measure Splay and RayTrace specifically — they're
   the icache-sensitive workloads. If regression > 2% on any, hold
   and investigate; possibly mark cold non-IC arms `#[inline(never)]`
   to shrink the hot loop body again.

2. **Bytecode-size impact on cold code.** Functions that never tier up
   still pay the +2 bytes/instruction. For modules with thousands of
   one-shot init functions this could be visible in install time.
   *Mitigation:* the bytecode-density report quantifies this.
   Accepted up to +15%; reconsider only if it exceeds +20%.

3. **Test262 module-loading edges.** Pre-existing 5-7 failures are
   listed in `vm-dispatch-phase6-status.md`. If new ones appear after
   Phase 2 or 3, the encoding round-trip likely broke for a specific
   opcode family.
   *Mitigation:* run the language/expressions slice and
   built-ins slice separately after each phase to localize.

4. **Feedback-vector size growth.** Every IC-shaped emission now
   allocates a slot. For functions with lots of arithmetic but no
   real polymorphism, feedback-vector entries grow proportionally.
   *Mitigation:* feedback-vector entries are cheap and per-CodeRef
   shared. The cost is a one-time per-function allocation increase;
   not a per-call cost.

5. **The structural regression tests at `dispatch.rs:14-129` are
   tightly worded.** They lock specific helper names out of the hot
   loop. The Phase 3 changes need to extend (not replace) those
   assertions to lock out `profiled_base_opcode` and `is_profiled`.

## Reproduce

Build:

```sh
cargo build --release -p lyng-js-cli
```

Correctness:

```sh
cargo test -p lyng-js-vm -p lyng-js-bytecode -p lyng-js-objects -p lyng-js-tests
cargo clippy -p lyng-js-vm -p lyng-js-objects --all-targets \
  -- -W clippy::pedantic -W clippy::nursery
cargo run --release -p lyng-js-test262 -- --report /tmp/t262.md -j 4
```

Asm verification (substitute the actual symbol hash from `nm`):

```sh
nm target/release/lyng-js | grep run_dispatch_loop
objdump --disassemble-symbols=<symbol> target/release/lyng-js \
  > /tmp/rdl-disasm-track-h.txt
grep -E '^\s+[0-9a-f]+:\s+[0-9a-f ]+\s+br\s+x' /tmp/rdl-disasm-track-h.txt
# Expect exactly one match per dispatch path (the main opcode jump table).
```

V8 v7 sweep:

```sh
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
  for i in 1 2 3; do
    ./target/release/lyng-js --shell /tmp/lj-$b.js | grep '^Score: '
  done
done
```

## Success Criteria

- `run_dispatch_loop` disassembly has exactly **one** `br x<N>` per
  dispatch path. The second indirect branch at offset ~+584 from the
  current Track-E baseline is gone.
- The `*Profiled` opcode mirror is **fully deleted** from
  `Opcode`, `Instruction`, the builder, the decoder, and the dispatch
  loop. No legacy code paths preserved.
- Test262 baseline maintained at 49722-49724 / 49729.
- V8 v7 sweep shows no regressions; expected modest gains
  (+5-12% on dispatch-bound workloads).
- Bytecode density change documented (+10-15% expected; accepted).
- Status report updated honestly, with the offset-level evidence
  that distinguishes "second branch removed" from "second branch
  symbol absent." Don't repeat the previous round's overclaim.

## Out Of Scope

- Helper-return-handling inlining (Track G from the handoff). Separate
  workstream; symmetric pattern but orthogonal mechanism.
- Back-edge cookie removal (Track F). Already partially landed in
  Track E (epoch-gated check). Further reduction is a separate
  follow-up.
- Per-handler tail-call dispatch (Phase 7 / direct-threaded). The
  single remaining indirect branch is the main opcode table; full
  threaded dispatch would eliminate that too, but is a much larger
  refactor and probably blocked on `unsafe` Rust or nightly `become`.
- JIT scaffolding. This round lays the *foundation* for the eventual
  JIT (uniform opcode shape with mandatory slot operand). The JIT
  itself is a separate workstream.

## Required Reading Before Touching Code

1. `reports/js/lyng-js/vm-dispatch-fixup-status.md` — landed work
   through Track A/D/IC-inline, and the corrected framing of what
   those rounds actually delivered.
2. `reports/js/lyng-js/vm-dispatch-infra-followup.md` — landed Track E
   (operand-form fold-in) + epoch-gated frame check + helper
   `#[inline]` hints. The current two-branch state is documented here.
3. `reports/js/lyng-js/vm-dispatch-next-round-handoff.md` — the
   workstream brief that named the three remaining structural items.
4. `reports/js/lyng-js/vm-dispatch-phase6-status.md` — current opcode
   space, encoding rules, Test262 failure baseline.
5. `crates/lyng-js/bytecode/src/opcode.rs` — the file with the
   `*Profiled` variants being deleted.
6. `crates/lyng-js/bytecode/src/instruction.rs` — the Instruction
   enum collapse.
7. `crates/lyng-js/vm/src/vm/dispatch.rs` — the dispatch loop
   simplification. Structural regression tests at lines 14-129 are
   the guard rail.
8. `crates/lyng-js/AGENTS.md` — repo-wide policies; the no-`unsafe`
   line applies (this plan doesn't need `unsafe`).
