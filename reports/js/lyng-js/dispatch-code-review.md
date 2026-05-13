# Lyng JS Dispatch Code Review

Date: 2026-05-13

Scope: `crates/lyng-js/vm` dispatch hot path, compared with checked-out QuickJS, V8 Ignition, and JavaScriptCore LLInt sources under `/Users/sondre/dev`.

## Executive Summary

Lyng has removed the old function-table fallback path and now has a single LLVM-jump-table-friendly opcode match, but the loop is still not close to the shape used by QuickJS, V8, or JSC. The current hot loop pays several fixed costs before most opcode bodies run:

- It materializes an `Instruction` enum from bytecode every dispatch.
- It strips profiled wrappers and re-matches the instruction form before opcode dispatch.
- It binary-searches the wide-operand side table for ordinary ABC/ABX instructions, including high-frequency `Move`.
- It copies/reads the active `FrameRecord` and writes `current_instruction_len`/frame instruction offset every opcode.
- It routes register access through `Vm` methods and frame windows rather than through localized frame/register state.

The most likely root cause is not that Rust `match` dispatch itself is poor. It is that Lyng's dispatch loop still has a decode-and-metadata architecture between opcode fetch and opcode execution. The peer engines encode the metadata needed by handlers in the bytecode stream or in direct indexed tables, and keep PC/frame/register state in interpreter locals.

## Local Benchmark Evidence

I ran:

```sh
cargo run --release -p lyng-js-bench -- runtime --preset smoke --count-opcodes --report /tmp/lyng-js-dispatch-review-runtime-opcodes.md --json /tmp/lyng-js-dispatch-review-runtime-opcodes.json
```

The run is a smoke run, so the throughput values are not final medians. The opcode mix is still useful because it shows which opcodes pay the fixed per-dispatch tax most often:

| Workload | ns/work-unit | Dispatches | Top opcode |
| --- | ---: | ---: | --- |
| `array-heavy.iterator-runtime` | 10169.27 | 10137 | `Move` 27% |
| `array-heavy.literal-indexed-runtime` | 1513.02 | 3799 | `Move` 37% |
| `class-heavy.runtime` | 3601.56 | 9138 | `Move` 30% |
| `regexp-constructor-compile.runtime` | 4653.66 | 2674 | `Move` 50% |
| `regexp-heavy.runtime` | 43768.23 | 15018 | `Move` 39% |
| `regexp-legacy-statics.runtime` | 2216.80 | 1952 | `Move` 33% |
| `regexp-named-replace.runtime` | 12019.53 | 1441 | `Move` 53% |
| `regexp-stable-exec.runtime` | 36106.77 | 48523 | `Move` 45% |
| `string-heavy.concat-runtime` | 298.83 | 1260 | `Move` 45% |
| `typed-array-heavy.runtime` | 1295.58 | 4395 | `Move` 44% |

This matters because the current `Move` path is not just a register copy. Before `Opcode::Move` reaches `read_register`/`write_register`, the loop has already decoded an `Instruction`, checked feedback, stripped profile wrappers, matched instruction form, and run `decode_abc_operands`.

## Peer Engine Shape

### QuickJS

QuickJS uses a local `pc`, local stack/register pointers, and computed-goto dispatch when enabled:

- `/Users/sondre/dev/quickjs/quickjs.c:17393` builds a `dispatch_table[256]` of labels.
- `/Users/sondre/dev/quickjs/quickjs.c:17403` dispatches with `goto *dispatch_table[opcode = *pc++]`.
- `/Users/sondre/dev/quickjs/quickjs.c:17500` enters the dispatch loop and handlers read operands directly from `pc`.
- `/Users/sondre/dev/quickjs/quickjs.c:18207` through `:18222` show local load/store short opcodes as direct `var_buf[...]` operations.
- `/Users/sondre/dev/quickjs/quickjs.c:18444` through `:18458` show jumps updating `pc` directly.
- `/Users/sondre/dev/quickjs/quickjs-opcode.h:65` onward defines opcode sizes as part of the opcode table.

QuickJS does not decode an intermediate instruction object in the hot loop. Operand size and stack effects are static opcode metadata.

### V8 Ignition

V8 Ignition dispatches through precompiled bytecode handlers:

- `/Users/sondre/dev/v8/src/interpreter/interpreter.h:108` through `:113` defines the dispatch table, including operand-scale variants.
- `/Users/sondre/dev/v8/src/interpreter/interpreter.cc:108` through `:115` stores each handler's instruction entry in that table.
- `/Users/sondre/dev/v8/src/interpreter/interpreter-assembler.cc:388` through `:405` read byte operands directly at compile-time-known offsets.
- `/Users/sondre/dev/v8/src/interpreter/interpreter-assembler.cc:1203` through `:1215` advances the bytecode offset as local assembler state.
- `/Users/sondre/dev/v8/src/interpreter/interpreter-assembler.cc:1382` through `:1410` loads the next bytecode, loads the target handler, and tail-calls it.
- `/Users/sondre/dev/v8/src/interpreter/interpreter-generator.cc:135` through `:174` show simple load/store/move handlers doing direct register operations then dispatching.

Ignition still has metadata and feedback, but handler code reads the specific operands it needs. It does not have a generic enum decode and form re-match in front of every handler.

### JavaScriptCore LLInt

JSC LLInt is similarly bytecode-layout-driven:

- `/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter.cpp:267` through `:285` initializes narrow/wide opcode maps.
- `/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter.cpp:409` through `:421` dispatches by computed goto.
- `/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter.asm:499` through `:542` computes dispatch advance from opcode length constants.
- `/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter.asm:554` through `:563` reads operands from the instruction layout.
- `/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter.asm:578` through `:592` uses an embedded metadata ID plus metadata offset tables, not a byte-offset binary search.
- `/Users/sondre/dev/WebKit/Source/JavaScriptCore/llint/LowLevelInterpreter64.asm:1194` through `:1247` show binary arithmetic fast paths that read operands, write the destination frame slot, and dispatch without a generic arithmetic-family rematch.

JSC's design keeps the interpreter state in registers/macros and makes bytecode layout carry the facts handlers need.

## Findings

### P1: Hot Dispatch Still Materializes `Instruction`

Lyng currently does:

- Fetch frame and instruction offset: `crates/lyng-js/vm/src/vm/dispatch.rs:91` through `:100`.
- Decode into `Instruction`: `crates/lyng-js/vm/src/vm/install.rs:70` through `:75`.
- Decode raw bytes via `decode_instruction_bytes`: `crates/lyng-js/bytecode/src/decoder.rs:91` through `:120`.
- Query `encoded_len`, `feedback_slot`, `without_feedback_slot`, and `opcode`: `crates/lyng-js/vm/src/vm/dispatch.rs:106` through `:115`.
- Match the stripped instruction form to populate operand locals: `crates/lyng-js/vm/src/vm/dispatch.rs:123` through `:159`.
- Only then enter the opcode match at `crates/lyng-js/vm/src/vm/dispatch.rs:169`.

The enum itself is defined at `crates/lyng-js/bytecode/src/instruction.rs:9` through `:38`. `Opcode::encoded_len()` already exists at `crates/lyng-js/bytecode/src/opcode.rs:295`, and its comment says it is for a byte-stream dispatcher, but the dispatcher still advances via `Instruction::encoded_len()` at `crates/lyng-js/bytecode/src/instruction.rs:148`.

This is likely the largest avoidable fixed dispatch cost. The peer engines dispatch from raw bytecode and let each handler decode only the operands it needs.

Recommended direction:

- Keep decoded `Instruction` only for disassembly, tests, validation, and debug surfaces.
- In the VM loop, read `bytes[pc]`, map to `Opcode`, and decode operands directly in the opcode arm or in form-specific inline helpers.
- Use the install-time validation guarantee to avoid repeated malformed-byte checks on every dispatch. If raw byte safety still needs a guard, keep it at frame entry or debug/assertion boundaries.
- Replace `current_instruction_len` writes with local `pc += opcode.encoded_len()` or arm-specific advance.

### P1: Wide Operand Lookup Is a Per-Opcode Binary Search

`InstalledFunction::wide_payload` is a binary search over a side table:

- `crates/lyng-js/vm/src/vm/install.rs:61` through `:68`.

The normal ABC/ABX decoders call it for ordinary instructions:

- `crates/lyng-js/vm/src/vm/values.rs:515` through `:535`.
- `crates/lyng-js/vm/src/vm/values.rs:537` through `:550`.

The only ABC fast-path exclusions are small calls and `Call`/`TailCall`/`Construct`. That means `Move`, `Add`, `GetNamedProperty`, `GetKeyedProperty`, `StrictEqual`, and many other hot instructions perform a byte-offset binary search before the handler body even runs.

Given the opcode counts above, this is especially bad for `Move`: in several workloads, 40-50% of dispatches are register moves, and each one goes through the wide-operand lookup path before copying a value.

Peer engines avoid this differently:

- QuickJS puts instruction size/operand form in the opcode table.
- V8 has operand-scale-specific handler entries and compile-time operand offsets.
- JSC has narrow/wide opcode maps and generated narrow/wide handlers.

Recommended direction:

- Stop storing wide operands as byte-offset side records for normal dispatch.
- Encode wide operands inline in bytecode, or introduce explicit wide-prefix/wide-opcode forms so the dispatch stream carries the width.
- If side tables must remain for compatibility, build a direct indexed lookup that is O(1) and cold for the common narrow path. A sorted vector binary search in the hot loop should be treated as a temporary diagnostic implementation, not the VM architecture.

### P1: Frame Hoisting Is Only Partial

The current frame types are split into metadata and state:

- `crates/lyng-js/vm/src/frame.rs:124` through `:140`.
- `crates/lyng-js/vm/src/frame.rs:189` through `:212`.
- `crates/lyng-js/vm/src/frame.rs:271` through `:281`.

But the hot loop still copies the frame each opcode:

- `crates/lyng-js/vm/src/vm/dispatch.rs:91` through `:99`.

And normal advance/jump mutates `self.frames.last_mut()`:

- `crates/lyng-js/vm/src/vm/registers.rs:31` through `:43`.
- `crates/lyng-js/vm/src/vm/registers.rs:53` through `:75`.

The comment in `frame.rs` says the metadata half is hoisted by the outer dispatch loop, but the loop copies `FrameRecord`, not just `FrameState`. Even if LLVM scalar-replaces some of this, the source architecture still forces most handlers and helpers to be written around `&FrameRecord`, which prevents a clean local-PC/local-register-window dispatch state.

Recommended direction:

- Keep `pc`/`instruction_offset`, register window/base, realm, lexical env, `this`, construct-this, and handler cursor in a local dispatch state.
- Write frame state back only at frame-changing or observable boundaries: calls, returns, exceptions, debugger/poll safepoints, generator/async suspension, and deopt/tiering capture.
- Replace broad `&FrameRecord` helper APIs in hot opcode families with narrower inputs. A small `DispatchFrameView` or local state struct is preferable to copying a full frame every dispatch.

### P2: Register Access Still Goes Through VM/Frame Plumbing

Simple register operations use:

- `read_register`: `crates/lyng-js/vm/src/vm/registers.rs:6` through `:14`.
- `write_register`: `crates/lyng-js/vm/src/vm/registers.rs:16` through `:29`.
- `Move`: `crates/lyng-js/vm/src/vm/dispatch.rs:170` through `:173`.

This computes the absolute register index and indexes `self.register_stack` on each read/write. Because install-time validation already reserves register windows, the hot loop should be able to operate from a localized base/window and let Rust eliminate most bounds checks in simple paths.

Peer examples:

- QuickJS local loads/stores are direct `var_buf[idx]` or `var_buf[0]` operations.
- V8 generated handlers load/store frame registers directly at operand-derived offsets.
- JSC arithmetic writes directly to `[cfr, dst, 8]` in the LLInt path.

Recommended direction:

- Bind register base/window once per activation or frame-state refresh.
- Restructure hot helpers to take `RegisterWindow`/base and operate on the register slice directly.
- Keep safe Rust indexing if desired, but shape the code so the optimizer sees one validated window rather than repeated calls through `self`.

### P2: `current_instruction_len` Is Global Mutable Dispatch State

The loop writes `self.current_instruction_len` every opcode at `crates/lyng-js/vm/src/vm/dispatch.rs:106`, and `advance_instruction`/`jump_by` read it from `Vm` at `crates/lyng-js/vm/src/vm/registers.rs:31` through `:43` and `:53` through `:75`.

This exists because handlers do not own local PC advancement. QuickJS, V8, and JSC all advance local PC/offset state directly and only synchronize frame state at boundaries.

Recommended direction:

- Make advance/jump local to the dispatch loop.
- Pass the next offset explicitly to suspend/resume helpers that need it.
- Remove `current_instruction_len` from `Vm` once helpers no longer require it.

### P2: Arithmetic/Comparison Dispatch Re-Matches Opcode Families

The top-level match groups arithmetic/comparison opcodes, then calls `execute_abc_value_opcode` with the opcode:

- `crates/lyng-js/vm/src/vm/dispatch.rs:175` onward.
- `crates/lyng-js/vm/src/vm/dispatch/arithmetic.rs:25` through `:98`.

`execute_abc_value_opcode` then calls `execute_smi_immediate_opcode`, `try_primitive_number_binary_opcode`, and does another `match opcode`. Thin LTO may fold some of this, so this is lower confidence than the decode/frame/side-table issues. Still, the peer engines tend to put the per-opcode fast path directly in the handler and call out only on slow cases.

Recommended direction:

- After fixing byte-stream decode and frame locals, split the hottest arithmetic/comparison opcodes into per-opcode arms or const-generic inline helpers.
- Verify with assembly/profiler before broad refactoring; do not optimize this before removing the guaranteed metadata/decode overhead.

### P3: Feedback/Spread Metadata Is Still Too Indirect

Call dispatch does a `wide_payload` lookup and then derives spread metadata through feedback descriptors:

- `crates/lyng-js/vm/src/vm/dispatch.rs:359` through `:381`.
- `crates/lyng-js/vm/src/vm/dispatch.rs:386` through `:407`.
- `crates/lyng-js/vm/src/vm/dispatch.rs:416` through `:435`.

Feedback recording also checks installed descriptors during warmup:

- `crates/lyng-js/vm/src/vm/feedback.rs:1721` through `:1779`.

This is less universal than the wide-operand issue, but it is the same architecture smell: hot handler metadata is discovered indirectly from side structures. V8 reads feedback slot operands in the handler; JSC embeds metadata IDs and uses direct metadata tables.

Recommended direction:

- Encode feedback slot IDs and call spread metadata in bytecode operands or direct metadata IDs.
- Make feedback allocation/warmup explicit and cheap on the allocated hot path.

## Non-Findings

The `Arc<InstalledFunction>` clone in `crates/lyng-js/vm/src/vm/dispatch.rs:84` through `:89` is once per activation of the outer loop, not once per opcode. It is worth removing eventually if a better ownership model falls out of dispatch-state work, but it is not the main dispatch tax.

`collect_arguments_into` clears/reserves a reusable `Vec<Value>` at `crates/lyng-js/vm/src/vm/call.rs:672` through `:695`. That is call-path work, not the fixed per-opcode dispatch problem. It may matter for call-heavy rows, but it should be profiled separately after the interpreter loop is cleaned up.

## Proposed Work Order

1. Build a byte-stream dispatch prototype for a narrow set of hot opcodes: `Move`, `LoadLocal0`-`LoadLocal3`, `LoadConst8`, `LoadSmi8`, `LoadZero`, `LoadOne`, `Add`, `AddSmi`, `Jump8`, `JumpIfFalse8`, and `Return`.
2. Remove `Instruction` materialization from the hot loop for those opcodes and advance via local PC.
3. Eliminate `wide_payload` binary search for narrow ABC/ABX operands. A first step can be "known narrow" fast paths for opcodes that cannot be wide in the current encoding, but the target should be inline wide encoding or wide opcode variants.
4. Localize frame state: register base/window, current offset, lexical env, `this`, construct-this, and handler cursor.
5. Update helper APIs only where needed by the hot opcodes first. Avoid broad VM refactors until measurements show the direction is correct.
6. Re-run `lyng-js-bench runtime --count-opcodes`, the arithmetic compare workload, and a targeted Test262 slice after each stage.

## Bottom Line

The current dispatch loop has the right high-level shape for stable Rust (`match Opcode` can become a jump table), but the hot path still looks like a decoder plus metadata resolver wrapped around a dispatch loop. QuickJS, V8, and JSC all make the bytecode stream and handler layout carry the data needed for the next opcode. Lyng should move in that direction before spending more time on alternative dispatch mechanisms.
