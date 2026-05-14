# VM Dispatch Fixup Status (lyng-1o9z follow-up)

## Scope Landed

Three commits on this branch, in order:

- **Track A** (`92429445`): inline opcode + operand decode in the dispatch
  loop. Removes `decode_dispatch_instruction`, `decode_unprofiled_operands`,
  `DispatchDecode`, `DispatchOperands`, and the free-function wrappers
  (`instruction_bytes_at`, `opcode_from_byte`, `feedback_slot_from_bytes`,
  `feedback_slot_at`, `reject_dispatch_prefix`). Marks
  `dispatch_operand_form` `#[inline(always)]` so LLVM folds the form match
  into each opcode arm of the central dispatch match.

- **Track D** (`b8960cc2`): inline SMI fast paths into `Add`, `Sub`,
  `Mul`, `Mod`, `BitAnd` and their `*Smi` immediate forms. Each arm now
  reads both register values, runs `i32::checked_*` (or `smi_mul_result`
  / `smi_mod_result` for -0-safe paths), writes the result, records the
  feedback slot, advances `pc`, and continues — with no helper call on
  the SMI fast path. Misses fall through to the existing
  `execute_*_opcode` helpers, preserving f64 / ToPrimitive / BigInt
  semantics including -0.

- **IC inline-hint salvage** (`20656d92`): add `#[inline]` to
  `load_from_named_property_cache` and `named_property_cache_entry_valid`
  on the named-property IC hit path. LLVM now folds the four-call
  chain through `Vm::execute_get_named_property_opcode` into one
  tight sequence on the hot path.

## V8 v7 Suite (5-sample median, Apple ARM)

The user's V8 v7 corpus (`testdata/js-benchmarks/v8-v7/`):

| Bench | Pre (n=1) | Post (n=5 median) | QuickJS | Pre ratio | Post ratio | Improvement |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| richards | 193 | **236** | 969 | 5.02x | **4.11x** | **+22.3%** |
| deltablue | 239 | **278** | 1051 | 4.40x | **3.78x** | **+16.3%** |
| crypto | 168 | **240** | 803 | 4.78x | **3.35x** | **+42.9%** |
| raytrace | 311 | **394** | 1011 | 3.25x | **2.57x** | **+26.7%** |
| navier-stokes | 332 | **408** | 1328 | 4.00x | **3.25x** | **+22.9%** |
| splay | 1002 | **1220** | 2298 | 2.29x | **1.88x** | **+21.8%** |

Geomean improvement across the suite: **~25%**.

The QuickJS gap narrowed from 2.3-6x slower to **1.9-4.1x slower**.
Crypto in particular dropped from 6x slower to 3.4x slower.

## Correctness Evidence

- `cargo test -p lyng-js-vm -p lyng-js-bytecode -p lyng-js-objects -p lyng-js-tests`:
  **1656 passed**, 0 failed.
- `cargo clippy -p lyng-js-vm -p lyng-js-objects --all-targets -- -W
  clippy::pedantic -W clippy::nursery`: clean.
- `cargo run --release -p lyng-js-test262 -- --report ...`:
  - Pass: **49724 / 49729** runnable files (99.99%).
  - Fail: **5** — same pre-existing failures from
    `vm-dispatch-phase6-status.md` (`language/import/import-defer/evaluation-triggers`,
    `language/module-code` MissingModuleEnvironment, one namespace
    Test262Error). Plus 1 intermittent RegExp staging timeout.
- Targeted Test262 slices for arithmetic / -0 semantics, all 100% pass:
  - `language/expressions/{addition, subtraction, multiplication,
    modulus, less-than, strict-equals}`
  - `built-ins/Math/sign`, `built-ins/Object/is`
  - 21164 variants in `language/expressions` overall.

## Profiler Evidence

Before (postmortem profile of Richards):

| Frame | Share |
| --- | ---: |
| `decode_dispatch_instruction` | 26% |
| `dispatch_operand_form` | 13% |
| `execute_get_named_property_opcode` | 13% |
| `call_value_small` + IC | 12% |

After Tracks A + D + IC inline hint (Richards):

| Frame | Samples / 1470 | Share |
| --- | ---: | ---: |
| `run_dispatch_loop` body | 608 | 41% |
| `execute_get_named_property_opcode` | 194 | 13% |
| `call_value_small` + invoke | 179 | 12% |
| `try_named_property_load_inline_cache_hit` | 79 | 5% |
| Other handler bodies | rest | — |

`decode_dispatch_instruction` and `dispatch_operand_form` are
**absent as separate symbols** from the post profile — but the *work*
they did was inlined into `run_dispatch_loop`, not eliminated. Earlier
revisions of this doc claimed "dispatch infrastructure cost is gone" /
"the profile confirms decode is gone." Both overstate what Track A
delivered.

### What Track A actually bought

- Removed the **function-call boundary** around decode — no
  register-spill/reload, no prologue/epilogue. That's worth a
  meaningful chunk on its own.
- Opened up **Track D's inline SMI fast paths**. Without Track A, the
  decode return type forced a uniform calling convention that prevented
  per-arm SMI continuation; once decode was inline the SMI fast paths
  could `continue;` the loop directly.
- The +22% on Richards is consistent with **"removed call overhead +
  inlined SMI arithmetic,"** not "removed decode."

### Symbol-level fresh sample (Richards on current main, 12429 samples)

Decoded offsets inside `run_dispatch_loop`:

| Offset | Samples | What it is |
| --- | ---: | --- |
| +740 / +404 (collapsed) | 5156 | Back-edge frame-validity check + register-zero between the two `br x0` jump tables |
| +9740 | 2589 | Post-call return-handling for `execute_get_named_property_opcode` (the `ldr [sp,#0xda0]; cmp; b.ne; b loop_top` epilogue after every helper call) |
| +2256 | 1027 | Post-call return-handling for `call_value_small` |
| +2904 | 874 | Same shape, another helper call site |

There are **three indirect branches** still threaded into the loop
body:

1. **Offset +628** (`br x0`) — prefix opcode dispatch (Wide / ExtraWide).
2. **Offset +764** (`br x0`) — main opcode dispatch.
3. **Offset +2092** (`br x13`) — operand-form dispatch.

Plus a per-iteration back-edge that re-validates the active frame
against a cookie before reading the next instruction (the +404 region),
and a chain of `mov w<N>, #0` register-zeroing instructions between the
dispatch layers.

**Roughly 30-40% of the `run_dispatch_loop` self-time is still dispatch
infrastructure** — three sequential jump tables, the back-edge
frame-validity check, and the post-helper return-handling sequence. The
decode *work* didn't disappear; the function-call boundary around it
did.

## Tracks B And C — Attempted and Documented

### Track B — Slim DispatchState

The plan called for hoisting `FrameMetadata` out of the per-iteration
`DispatchState` and only carrying the mutable `FrameState` subset.

**Three approaches tried, none delivered measurable wins:**

1. Hoist `registers_window` as a bare outer-loop local and rewrite the
   fast-path arms to read it directly. Reverted: -0.4% on richards,
   -0.7% on raytrace, no other movement. LLVM was already caching
   `frame.registers()` effectively through the `&mut FrameRecord`
   borrow.
2. Split `DispatchState` into `FrameMetadata` (hoisted) + `FrameState`
   (mutable subset). Aborted before implementation: the 50+ helpers
   that take `&FrameRecord` would all need new signatures, and the
   profile already showed handler bodies dominate, not frame-field
   access.
3. Inline `record_feedback_slot` and `record_allocated_feedback_slot`
   in `feedback.rs` to shave per-arm function-call overhead. Reverted
   after measuring: -1% richards, -2.5% crypto, -2% navier-stokes —
   likely icache pressure from a now-larger dispatch loop.

The per-opcode `FrameRecord` copy that the postmortem highlighted is
already paid only once per activation (since `fa99ed14`). With that
fixed, the remaining hot path is in handler bodies and the IC machinery
— not in frame access. Track B's planned refactor doesn't address the
real bottleneck.

### Track C — Compiler-side Move elimination

Move dispatches are 27-50% of opcodes on most workloads. Two peephole
implementations tried, neither caught enough patterns:

1. **Conservative** (`LoadX Rtemp; Move Rdst, Rtemp` → `LoadX Rdst`
   when Rtemp is dead): the compiler's `lower_expr_into(expr, dest)`
   already emits direct loads in most cases, and register reuse
   defeats the conservative "dead after Move" check. Reverted with
   +0-2% movement, opcode mix unchanged.
2. **Aggressive** (any rewritable writer — Add/GetNamedProperty/etc.
   — with smarter dead-check that allows reassignment-before-read):
   diagnostic counters showed every Move falling through `no_pred`
   because the predecessor at position i-1 is itself a `Move`. The
   compiler emits Move-Move chains for argument marshaling, not
   single-Load + single-Move pairs. Reverted with +0.4-0.9%
   movement, opcode mix essentially unchanged.

The **right** fix is in the compiler, not in a peephole: change
`materialize_argument_block` (in `calls.rs`) and `lower_call_target`
to allocate the call-arg block FIRST and emit arguments directly into
their final slots via `lower_expr_into(expr, slot)`. That would
eliminate one Move per call argument. It's a 200-300 line change
across `calls.rs`, `expr.rs`, and the spread-element path, plus
careful handling of the small-call optimization (which packs
`this_value` + args in a contiguous block of its own). Deferred —
too large for a focused fix-up, and a follow-up workstream rather
than a dispatch-modernization concern.

The IC inline-hint salvage (commit `20656d92`) was discovered while
investigating Track C's failure and ended up being the only piece of
the "fix the IC tax" side conversation that delivered real wins
(+5% Richards, +3% DeltaBlue).

## Where The Remaining Gap Lives

Post-fixup we are still 1.9-4.1x slower than QuickJS. The remaining
delta has two parts:

### Handler-body / IC / call work

- **Compiler Move elimination via direct argument lowering** (real
  Track C, deferred): 27-50% of dispatches are still Move. Each Move
  is fast post-Track-A but eliminating ~half of them outright would
  give a meaningful speedup.
- **Call path inlining** (real Track B-equivalent for calls):
  `call_value_small` + `invoke_collected_call_value` +
  `enter_bytecode_call` are still separate function calls. Inlining
  the warmed fast path could shave 5-10% on call-heavy benchmarks.
- **Property store IC fast path**: `try_named_property_store_inline_cache`
  is the symmetric IC path to the load we inlined. Same inline hints
  should help similarly.

### Remaining dispatch infrastructure (revised)

Earlier drafts of this doc said "threaded dispatch (Phase 7) doesn't
look needed — the central match is not the bottleneck." That conclusion
is **wrong** in the form it was stated. The central `match` *itself*
is fine — LLVM lowers it cleanly. But there are **three sequential
jump tables plus a back-edge cookie check plus per-helper return
epilogues** still inlined into the loop body, accounting for ~30-40%
of `run_dispatch_loop` self-time. That is dispatch infrastructure
even if it no longer shows up under a separate symbol name.

Three structural items to attack, in order of expected impact:

1. **Collapse the 3-layer jump table.** Today: prefix opcode → main
   opcode → operand form, each its own `br`. A single table indexed by
   `(prefix, opcode)` with the operand form folded into each arm
   eliminates two of the three indirect branches per iteration. This
   is essentially what direct-threaded dispatch buys, and is the same
   territory Phase 7 deferred. The Phase 7 evaluation cited "the
   central match already lowers to a jump table" — true at the source
   level but missing that there are **three** jump tables, not one.
2. **Drop the per-iteration back-edge cookie check.** Today every loop
   iteration re-validates `frame_depth == stored_depth` even on
   opcodes that can't change the frame. Re-validation only needs to
   happen after `Call` / `Return` / `Throw` / `Yield` / `Await` and
   handler entry/exit. The +404 region samples (~5k / 12.4k) are
   largely this check + register-zero in the back-edge.
3. **Inline helper return-handling.** Every non-leaf opcode arm has a
   4-instruction epilogue after the `bl` to the helper
   (`ldr [sp,#0xda0]; cmp; b.ne; b loop_top`). The +9740 / +2256 /
   +2904 hot offsets (~4.5k / 12.4k samples combined) are these
   epilogues for the three hottest helpers
   (`execute_get_named_property_opcode`, `call_value_small`, another
   helper at +2904). Marking the hot helpers `#[inline]` — or adding
   a fast-return shape so the helper returns the same shape the loop
   expects without the explicit re-check — should fold this back into
   the helper itself.

A second pass on items 1 + 2 + 3 plus the three handler-body items
above would plausibly deliver another **30-50% combined**. Phase 7
(direct-threaded dispatch) is the right *direction* for item 1; the
specific implementation should be revisited against the current
profile, not the old "single jump table" framing.

## Reproduce

```sh
cargo build --release -p lyng-js-cli

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
  printf "%-15s lyng=%s qjs=%s\n" "$b" \
    "$(./target/release/lyng-js --shell /tmp/lj-$b.js | grep '^Score: ')" \
    "$(qjs /tmp/lj-$b.js | grep '^Score: ')"
done

cargo run --release -p lyng-js-test262 -- --report /tmp/lj-t262.md -j 12
```
