# VM Dispatch — Next Round Handoff

Issue: `lyng-1o9z`. Follow-up to the work that landed on `main` through commit
`a4044e7c` (Tracks A + D + IC inline-hint salvage). This brief is written for
an agent picking up the work cold.

## Why You're Here

Lyng JS is still 1.9–4.1x slower than QuickJS on the V8 v7 suite after the
last round of dispatch fixes. That round landed real wins (+22% geomean,
Crypto +43%) but the existing status report
(`reports/js/lyng-js/vm-dispatch-fixup-status.md`) overclaims what was
fixed in dispatch. A fresh symbol+offset profile shows the dispatch
*infrastructure* is still ~30–40% of `run_dispatch_loop` self-time on
Richards. The previous status report's "dispatch infrastructure cost is
gone" claim is wrong, and the Phase 7 evaluation
(`reports/js/lyng-js/vm-dispatch-phase7-evaluation.md`) that recommended
deferring direct-threaded dispatch was based on an incomplete reading of
the release assembly.

**Your job: deliver another concrete round of dispatch-loop wins, with
honest measurement.**

## What You'll Find On Main

```
86670f1f Modernize VM dispatch encoding         (the original modernization)
92429445 Track A: inline dispatch decode, kill decode_dispatch_instruction
b8960cc2 Track D: inline SMI fast paths into Add/Sub/Mul/Mod/BitAnd arms
dbd79a9c VM dispatch fixup status (lyng-1o9z): Tracks A + D landed
20656d92 Inline-hint IC hot-path helpers (Track C salvage)
a4044e7c Update fixup status: Tracks A + D + IC inline hint, B/C deferred
```

`cargo build --release -p lyng-js-cli` is green. 1656 unit tests pass.
Test262 baseline: 49724/49729 runnable files. Don't regress these.

## The Mistake In The Previous Report

The previous report says:

> `decode_dispatch_instruction` and `dispatch_operand_form` are absent
> from the post profile. The dispatch *infrastructure* cost is gone;
> remaining time is in handler bodies + IC machinery.

The first sentence is correct (the symbols were inlined away). The
second is wrong — the *work* those symbols did got folded into
`run_dispatch_loop`, where it shows up under the parent symbol. A fresh
sample disagrees with the conclusion.

## The Actual Breakdown

Reproduce the sample:

```sh
cargo build --release -p lyng-js-cli
cat testdata/js-benchmarks/v8-v7/base.js \
    testdata/js-benchmarks/v8-v7/richards.js > /tmp/lj-rich.js
cat >> /tmp/lj-rich.js <<'EOF'

for (var i = 0; i < 40; i++) {
  BenchmarkSuite.RunSuites({
    NotifyResult: function(n,r){},
    NotifyError: function(n,e){print(n+" ERROR: "+e)},
    NotifyScore: function(s){print("Score: "+s)}
  });
}
EOF
./target/release/lyng-js --shell /tmp/lj-rich.js > /tmp/lj-rich.out 2>&1 &
LJ_PID=$!
sleep 1
sample $LJ_PID 15 1 -file /tmp/lyng-richards.sample.txt > /dev/null 2>&1
kill $LJ_PID 2>/dev/null
```

Then disassemble:

```sh
objdump --disassemble-symbols=__ZN10lyng_js_vm2vm8dispatch36_\$LT\$impl\$u20\$lyng_js_vm..vm..Vm\$GT\$17run_dispatch_loop17h3ae27891d3804b79E \
  target/release/lyng-js > /tmp/rdl-disasm.txt
```

(The symbol hash will be different on your build. Use `nm` to find the
right one — there are usually four `run_dispatch_loop` instantiations;
pick the one with the most samples in the call tree.)

Map sample offsets back to the disassembly. The three indirect branches
inside `run_dispatch_loop` you should see:

| Offset (mine) | Instruction | What it is |
| ---: | --- | --- |
| ~+628 | `br x0` | **Prefix opcode dispatch** — table-lookup for Wide/ExtraWide |
| ~+764 | `br x0` | **Main opcode dispatch** — the primary opcode jump table |
| ~+2092 | `br x13` | **Operand-form dispatch** — `dispatch_operand_form` inlined |

And these hot offsets in a Richards run (12429 main-thread samples):

| Offset (mine) | Samples | What's there |
| ---: | ---: | --- |
| +740 / +404 (collapsed) | ~5156 | Back-edge frame check, register-zero between the two `br x0` jump tables, jump-table setup |
| +9740 | ~2589 | Post-call epilogue after `execute_get_named_property_opcode` (`ldr [sp,#0xda0]; cmp; b.ne; b loop_top`) |
| +2256 | ~1027 | Same shape, post-call epilogue after `call_value_small` |
| +2904 | ~874 | Same shape, another helper call site |

About 30–40% of `run_dispatch_loop` self-time is in those infrastructure
regions, not in handler bodies. **Confirm this on your build before
proceeding.** Don't trust my numbers — capture your own.

## What Track A Actually Bought Us

For honest framing:

- Removed the function-call boundary at decode (no register-spill /
  reload, no prologue/epilogue across the decode → handler edge).
- Enabled Track D's inline SMI fast paths (they couldn't have happened
  while decode was a separate function).
- Combined value: +22% Richards, +43% Crypto.

What Track A did **not** do: eliminate decode work itself. The three
jump tables, the back-edge cookie, and the helper return-handling
sequence are all still in the loop body.

## Three Workstreams, In Priority Order

### 1. Collapse the 3-layer jump table (Track E)

**Highest impact. Most invasive.** This is the work Phase 7 deferred,
re-justified.

Today: prefix → main opcode → operand form = three indirect branches
per opcode dispatched. Per the Phase 7 evaluation, the central match
"already lowers to a jump table" — that's true but misses that there
are two more.

Goal: one indirect branch per dispatch. Approach options to compare on
a side branch:

- **(a) Fused dispatch table** indexed by a single byte that already
  encodes prefix+opcode. Re-encode bytecode so prefixed forms get
  distinct opcode bytes. Requires opcode-space audit (current high
  watermark from
  `reports/js/lyng-js/vm-dispatch-phase6-status.md` is 203/256; the
  Wide/ExtraWide-prefixed variants would expand this).
- **(b) Direct-threaded dispatch** via macro-replicated next-opcode
  match. The Phase 7 doc dismissed this because `become` is
  experimental and unsafe-rust is out. A safe-Rust replicated-match
  approach is still feasible — Phase 7 named the cost (code size,
  duplicated handler structure) without prototyping it. Time-box a
  prototype on one workload (Richards) and compare runtime, code
  size, `cargo asm`, and Test262 before deciding.
- **(c) Operand form fold-in.** The third jump table at +2092 is
  `dispatch_operand_form` inlined. If each opcode arm read its own
  operands directly (without the form-table lookup), this third
  branch disappears. Track A *almost* did this — operand reading is
  inline now — but the form-table lookup at the loop top is still
  there. Track this down in `dispatch.rs`. If `dispatch_operand_form`
  is still being called or its result is still being used to pick a
  per-opcode operand-read shape, fix that.

Start with (c) — it's the cheapest of the three and may eliminate one
jump table on its own. Then prototype (a) or (b) on a branch.

**Files:** `crates/lyng-js/vm/src/vm/dispatch.rs` (primary). For (a):
also `crates/lyng-js/bytecode/src/opcode.rs`,
`crates/lyng-js/bytecode/src/builder.rs`,
`crates/lyng-js/compiler/src/**`.

**Verification:**
- `cargo asm --lib --build-type release 'lyng_js_vm::vm::dispatch::<impl lyng_js_vm::vm::Vm>::run_dispatch_loop'`
  should show **one** indirect-branch `br x<N>` per dispatch path,
  not three.
- Re-run the sample workflow above. The "+740/+404 collapsed" cluster
  should shrink dramatically. If it doesn't, the change didn't help.
- Test262 baseline: 49724/49729 runnable files. Same V8 v7 score
  improvements documented in
  `reports/js/lyng-js/vm-dispatch-fixup-status.md`.

### 2. Drop the per-iteration back-edge cookie check (Track F)

**Medium impact. Small change.**

Today: every loop iteration loads `frame_depth`, loads stored cookie,
compares, branches on mismatch. The cookie only changes after
frame-mutating opcodes (Call*, Return*, Throw, Yield, Await,
Suspend, exception transfer, debug safepoint).

Goal: those boundary opcodes set a "frame dirty" flag, and the back
edge only re-validates when the flag is set. Or: each frame-changing
arm explicitly re-loads, and the back edge does no validation.

**Files:** `crates/lyng-js/vm/src/vm/dispatch.rs` —
`advance_dispatch_frame`, the outer loop preamble, and every arm that
sets `frame_depth` or syncs back to `self.frames`. Confirm boundaries
with `git log --oneline crates/lyng-js/vm/src/vm/dispatch.rs` —
`fa99ed14` and predecessors documented the existing sync points.

**Verification:**
- Sample again. The +404 cluster should shrink.
- The structural regression tests at the top of `dispatch.rs` (lines
  14–129 currently) test for "no per-op frame copies" — keep that
  invariant. Add a new test asserting no per-op frame-depth load if
  you can find a stable assembly signature for it.
- Test262 + V8 v7 baselines unchanged.

### 3. Inline helper return-handling (Track G)

**Lower-impact-per-byte, but symmetric to the IC inline-hint salvage.**

Today: every non-leaf opcode arm has a 4-instruction epilogue:

```
bl  <helper>
ldr x8, [sp, #0xda0]      ; load DispatchResult success marker
cmp x8, x9
b.ne <error_path>
b   <loop_top>
```

Each `bl` site contributes ~30–80 samples to `run_dispatch_loop` self-time
just for this epilogue. Hot helpers worth inlining the fast-return path of:

- `execute_get_named_property_opcode` (already had its IC inlined in
  commit `20656d92`; the wrapping call itself isn't inlined yet).
- `execute_set_named_property_opcode` (symmetric to load — needs the
  same treatment, including inlining `try_named_property_store_inline_cache`).
- `call_value_small` / `invoke_collected_call_value` /
  `enter_bytecode_call` — the call hot path.
- `execute_get_keyed_property_opcode` — visible in the sample under
  Richards even though Richards is mostly named-property work.

Approach: add `#[inline]` (not `#[inline(always)]`) to the helper's
fast-return path, splitting cold paths into a separate non-inlined
helper if needed. The IC salvage commit `20656d92` is a 2-line
demonstration.

**Files:**
- `crates/lyng-js/vm/src/vm/dispatch/property.rs`
- `crates/lyng-js/vm/src/vm/call.rs`
- `crates/lyng-js/vm/src/vm/bytecode_calls.rs`
- `crates/lyng-js/objects/src/internal_methods/property_cache.rs`
  (already partially done in `20656d92`; check what's left)

**Verification:**
- After each helper inline-hint, sample again and check the
  corresponding post-call offset (e.g., +9740 for
  `execute_get_named_property_opcode`). If the cluster doesn't
  shrink, the inline didn't take — examine `cargo asm` of the arm.
- Watch for regressions: I tried inlining `record_feedback_slot` and
  `record_allocated_feedback_slot` last round and got -1% Richards,
  -2.5% Crypto, -2% NavierStokes (likely icache pressure). Test each
  inline hint on the full V8 v7 sweep before committing. Don't trust
  a single workload.

## Constraints

- **No `unsafe` Rust.** `crates/lyng-js/AGENTS.md:139` documents this.
  The user has explicitly turned down `unsafe`-based options for
  dispatch in earlier rounds. Macro-replicated next-opcode matches are
  the safe-Rust alternative if you want to prototype direct-threading.
- **No regressions on Test262.** Baseline is 49724/49729 runnable
  files (5 pre-existing module-loading failures documented in
  `reports/js/lyng-js/vm-dispatch-phase6-status.md`). Run
  `cargo run --release -p lyng-js-test262 -- --report /tmp/t262.md -j 12`
  on every commit you intend to land.
- **No regressions on V8 v7.** Use the reproduction script below.
- **Don't overclaim.** Capture a real profile (sample + offset-to-
  source mapping) before writing a status report. The previous round's
  "infrastructure cost is gone" claim was based on the absence of a
  symbol, not on actual time distribution. Don't make the same mistake.
  If you can't directly attribute samples to a region of source code,
  say so.

## Required Reading Before Touching Code

In order:

1. `reports/js/lyng-js/vm-dispatch-fixup-status.md` — the report whose
   claims you're correcting. Note Tracks A + D as landed, Tracks B + C
   as deferred (the deferral writeups there explain why a 50-helper
   refactor and a peephole pass both failed to deliver).
2. `reports/js/lyng-js/vm-dispatch-phase7-evaluation.md` — the Phase 7
   evaluation. Read it critically. It correctly notes the central
   match lowers to a jump table, but it does not enumerate that there
   are *three* jump tables, and it does not prototype the safe-Rust
   replicated-match alternative it dismisses. Your Track E may revisit
   this conclusion.
3. `reports/js/lyng-js/vm-dispatch-phase6-status.md` — current opcode
   space, encoding rules, and test baselines.
4. `crates/lyng-js/vm/src/vm/dispatch.rs` — the file you'll be editing.
   The structural regression tests at the top of the file (currently
   lines 14–129) are your safety net against accidentally
   re-introducing helpers, frame copies, or generic rematches.
5. `crates/lyng-js/AGENTS.md` — repo-wide policies, especially the
   no-`unsafe` line.

## Reproduction Commands

Build:

```sh
cargo build --release -p lyng-js-cli
```

Correctness:

```sh
cargo test -p lyng-js-vm -p lyng-js-bytecode -p lyng-js-objects -p lyng-js-tests
cargo clippy -p lyng-js-vm -p lyng-js-objects --all-targets \
  -- -W clippy::pedantic -W clippy::nursery
cargo run --release -p lyng-js-test262 -- --report /tmp/t262.md -j 12
```

V8 v7 sweep (5-sample median is what the previous report used; do at
least 3 samples to filter noise):

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
  qjs /tmp/lj-$b.js | grep '^Score: '
done
```

Targets (from the existing report; don't regress, aim to improve):

| Bench | Current | QuickJS | Goal |
| --- | ---: | ---: | ---: |
| richards | 236 | 969 | ≥ 700 |
| deltablue | 278 | 1051 | ≥ 750 |
| crypto | 240 | 803 | ≥ 500 |
| raytrace | 394 | 1011 | ≥ 600 |
| navier-stokes | 408 | 1328 | ≥ 700 |
| splay | 1220 | 2298 | ≥ 1200 (no regression) |

Profile (Richards example, see "The Actual Breakdown" section above
for the full workflow):

```sh
./target/release/lyng-js --shell /tmp/lj-rich.js &
PID=$!
sleep 1
sample $PID 15 1 -file /tmp/lyng-richards.sample.txt
kill $PID
```

`atos -o target/release/lyng-js -l 0x100000000 <addr>` resolves
addresses back to symbols. For line-resolution add
`-Cdebuginfo=2` to release builds (this is expensive — don't make it
default, just use a separate target dir for profiling).

Disassembly:

```sh
nm target/release/lyng-js | grep run_dispatch_loop
# Pick the hash that matches your sample's call tree.
objdump --disassemble-symbols=<symbol> target/release/lyng-js \
  > /tmp/rdl-disasm.txt
```

## Suggested Commit Cadence

Each track is independent. Land each as its own commit (the user
preferred a series of commits on `main` over multiple PRs last round):

- Track E preflight: get clean before/after sample numbers on
  Richards. Write to `reports/js/lyng-js/vm-dispatch-track-e-baseline.md`.
- Track E (c) — operand-form fold-in. Smallest of the three (c)
  options; do this first to see if it eliminates the third jump table.
- Track E (a) or (b) — fused dispatch table or direct-threading
  prototype, depending on what (c) reveals. Prototype on a separate
  branch; merge into the working branch only after measuring.
- Track F — back-edge cookie check.
- Track G — helper return-handling, one helper per commit so you can
  bisect regressions.
- Final: update `reports/js/lyng-js/vm-dispatch-fixup-status.md` with
  honest before/after, **and** correct its overclaim about the
  previous round.

## What Success Looks Like

- `run_dispatch_loop` shows one indirect branch per dispatch path in
  `cargo asm`, not three.
- The "+740/+404 collapsed" sample cluster shrinks meaningfully on
  Richards (current ~5156 samples → target ≤ 2000).
- V8 v7 geomean improves another ~15–25% over current.
- Test262 baseline preserved.
- The status report cleanly distinguishes "symbol absent" from "work
  eliminated" and provides offset-level evidence for any claim of
  removed work.

## What Failure Looks Like (Avoid)

- Updating the status report without re-capturing the profile.
- Reporting V8 v7 numbers from a single sample (the previous round's
  pre-numbers were single-sample and noise made some of them look
  worse than reality).
- Inlining helpers without measuring icache impact — the previous
  round's feedback inline attempt regressed three benchmarks.
- Reaching for `unsafe` or `become` — they're off-limits. If you find
  yourself there, step back to a safe-Rust alternative.

## Issue Tracking

All under existing issue `lyng-1o9z` ("Implement VM dispatch
modernization"). Don't open new top-level issues; this is the same
workstream the previous rounds belonged to.
