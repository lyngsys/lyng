# Handoff — SP-0b FINAL session: delete the dispatch snapshot (recover the −15%) + tidy up

> Read this first, then `docs/superpowers/plans/2026-06-01-sp0b-eliminate-dispatch-snapshot.md`
> ("SESSION #10 EXECUTION" is the live pointer; Phase 3/4 = the original Task-level detail for
> the bridge rewrite + deletion). This file is the focused plan for the LAST session.

## Goal & directive (locked)

Delete `DispatchState.frame: FrameRecord` — the per-frame snapshot — so the interpreter reads
the `FrameHeader` overlay + cold table + the thin `FrameView` (`DispatchState.pc/cfr/code_ref/
regs_len`) directly. This removes the per-call/return `reconstruct_frame_from_header` that is
the **~15% SP-0b call/return regression**. Correctness-first; every step Test262-baseline.

**THE −15% IS ALL-OR-NOTHING AT DELETION — this was MEASURED this session, do not re-litigate it.**
A 7-sample v8 A/B (baseline `1b39a0ec` vs the current branch HEAD, which already has the
Refresh-arm reconstruct DEFERRED) still shows the full regression:

| Bench | base | branch | | Bench | base | branch |
|---|---|---|---|---|---|---|
| Richards | 508 | 439 | | RayTrace | 459 | 379 |
| DeltaBlue | 386 | 329 | | NavierStokes | 545 | 460 |
| Crypto | 432 | 387 | | Splay | 1479 | 1324 |

Deferring the Refresh reconstruct only MOVED it into `sync_from_asm` (every call-heavy frame runs
a slow stub → `sync_from_asm` → lazy reconstruct, same count). The win lands ONLY when the field
is gone so `sync_from_asm` becomes `dispatch.pc = frame_pc_offset` (no reconstruct) AND
`finish_frame` stops reconstructing the popped frame. **Reader migrations alone yield ZERO perf** —
do not expect partial wins; budget for the whole deletion landing as one validated unit.

## Branch / state (verified at handoff)

- Repo `/Users/sondre/dev/lyng` · branch `feat/sp0b-unified-register-file-frame-arena` · HEAD `bf3eaa77`. Working tree CLEAN.
- Build green; `cargo test -p lyng-vm --all-features` = **609/0** (sum of the 7 `test result: ok` lines: 571+3+10+2+1+22+0). Clippy = **7 PRE-EXISTING** warnings (3×"too many arguments (10/7)", 1×(9/7), 1×(8/7), 2×"unnecessary closure … Option::None" — at NON-edited lines; match against these, ignore them).
- Whole-corpus Test262 = **49729 passed / 0 failed / 0 panicked / 3324 skip**, variants **95205/0/0** (6648 intl402 skip). Re-validate at every step.
- **28** `&FrameRecord`/`&mut FrameRecord`/`frame: FrameRecord` param sites remain (most are EXCLUDE/construction; see the inventory below).
- Crate dir is `crates/vm/` (package `lyng-vm`). Test262 tool: `tools/lyng-test262` (package `lyng-test262`). Bench: `tools/lyng-bench` (package `lyng-bench`).
- **A/B is pre-staged:** baseline binary built at `/tmp/lyng-base/target/release/lyng` (worktree at `1b39a0ec`); `target/release/lyng-bench` built. `git worktree remove /tmp/lyng-base` at the end (tidy-up).

## What's already done (this lineage — do NOT redo)

- Sessions #1–#9: the entire call/construct/tail core, runtime_objects iterators, async cluster, the `yield*` delegate generator core (7 methods), and the 2 static super/this resolvers are all FrameView/CallerContext. `call.rs`/`runtime_objects.rs`/`async_functions.rs`/`generators.rs` are FrameRecord-free outside `#[cfg(test)]` + the generator suspend/restore ENCODE (`FrameRecord::new(...).with_*()`/`with_resume`, which STAYS — it's the push/restore construction type).
- Session #10 (this session): the assign-named **rust-probe** reads the thin view (removed the sole `sync_from_asm`-bypassing snapshot reader); the **Refresh arm** defers the reconstruct (`frame_dirty = true`); all `dsl/handlers/` reads → thin view. `dsl/handlers/` has ZERO direct snapshot reads.

## CRITICAL FACTS established this session (don't rediscover)

1. **The Alpha dispatch variant is TEST-ONLY.** `LlIntDispatchState::from_alpha` is called only in `dsl/test_helpers.rs:345`. Production runs exclusively the Asm path. So `dsl/handlers/` snapshot reads were the *shared* handler fns reached post-`sync_from_asm` on the Asm path, and the deletion's α impact is bounded to test/reference code.
2. **`finish_frame` (registers.rs:106) has a SECOND per-return reconstruct** (`pop_current_frame`), independent of the Refresh arm. It's largely VESTIGIAL: finish_frame already reads `this_value`/`lexical_env`/`construct_this` from the overlay (`frame_header(cfr_of(&frame))`), and only uses the reconstructed `frame` for `cfr_of(&frame)` (== `self.current_cfr` before release), `frame.flags()` (overlay `flags_bits`), `frame.return_register()` (overlay), `frame.registers()` (geometry), and `refresh_running_context_to_caller(agent, &frame)`. All overlay/geometry-derivable → Task 11 is tractable.
3. **`reconstruct_frame_from_header` CANNOT be fully deleted** — it stays for the `Vm::frame()`/`frames()` accessors (vm.rs:1223/1236), which feed GC trace, the debugger, `property_access.rs:255/277`, and tests. Only its HOT-PATH callers go: `sync_from_asm` (slow_path.rs:138), the entry reconstruct (vm.rs:2767), and `finish_frame`'s `pop_current_frame` (Task 11). Verify each remaining caller is cold before/after.
4. **The F3 PC-mirror is equivalent to a conditional overlay read.** `dispatch_state.rs:292 self.pc = self.frame.instruction_offset()` (in the `handle_dispatch_result` wrapper) can become `if same-frame && handled.is_none() { self.pc = self.vm.frame_header(self.cfr).saved_pc() }`: on SUCCESS `self.pc` is already the entry PC (the body didn't advance it — wrapper-routed bodies pass their delta via `Continue.pc_advance`, and `op_delegate_yield`/`op_await` already override `self.pc` from the overlay themselves), so the success arm is a no-op; on a SAME-frame caught throw `transfer_to_exception_handler` parked the handler PC into `saved_pc` and `refresh_dispatch_frame` rebuilt the snapshot from it, so overlay `saved_pc` == the value the old reader produced. **Verify by code-read that no wrapper-routed body advances the snapshot on success before relying on this.**

## The deletion plan (ordered; each Phase-1 step gated independently)

### Phase 1 — close the last reader surface (behavior-preserving, NO perf yet, gate each)

Each is Test262-baseline; commit per green step. These do not change perf (snapshot still lazily
reconstructed) — they shrink the deletion to a clean compile-error-driven step.

1. **generators.rs reads.** `next_dispatch_instruction_offset(&inner.frame, args.instruction_len)` (semantics/generators.rs:144 op_suspend_generator_start, :175 op_yield) → `inner.pc().wrapping_add(args.instruction_len)`. Delete the DEAD `inner.frame.clear_resume()` (:340 op_load_resume_value) — the cold table is authoritative and `inner.clear_resume()` runs one line above; the Step-A cold-half tripwire that policed snapshot↔cold parity was removed in Session #8.
2. **op_yield / op_suspend_generator_start `sync_active_frame()`.** These call `inner.sync_active_frame()` (dispatch_state.rs:225 → reads `self.frame` → `sync_dispatch_frame`) to park the PC before suspend. VERIFY whether the park is load-bearing: `suspend_current_generator_frame`/`suspend_generator_start` capture `resume_offset` EXPLICITLY and release the frame, so the snapshot park is likely vestigial. If needed, replace with an overlay park (`frame_header_mut(cfr).set_saved_pc(pc)` via a small DispatchState helper); else remove. Gate + async/generator canary.
3. **F3 PC-mirror** (dispatch_state.rs:292): convert per CRITICAL FACT #4. Gate (this touches every wrapper-routed body — full Test262).
4. **exceptions (2)** — `select_exception_handler`/`suspended_call_instruction_offset` (exceptions.rs:59/86) read only `frame.code()`+`frame.instruction_offset()` → take `frame: FrameView`. Their sole caller `transfer_to_exception_handler` (exceptions.rs:5) builds the transient `reconstruct_frame_from_header(cfr, depth-1)` (line 20) PURELY to feed them — replace with `FrameView::new(cfr, self.frame_header(cfr).saved_pc(), self.frame_window_len(cfr), self.frame_header(cfr).code())` (the `instruction_offset` MUST be the parked `saved_pc` = the handler-search PC; these two fns never read `registers()` but pass the real `frame_window_len` not 0). DELETES one hot-ish reconstruct. Gate + full Test262 (exception-heavy + the `internal_completion_targets` guard at exceptions.rs:21-28 reads `frame_depth`, unaffected).
5. **with_env (2, write-side)** — `push_with_environment`/`pop_with_environment` (with_env.rs:7/39) write BOTH the snapshot `frame.set_lexical_env` AND the overlay. Drop the snapshot write + the `frame: &mut FrameRecord` param (operate on the active overlay via `current_cfr`); update the caller `semantics/scope.rs:315`. THEN drop the overlay-half **Step-A tripwire** (the `this_value`/`this_state`/`lexical_env`/`construct_this` `debug_assert`s in `sync_dispatch_frame`, vm/dispatch.rs:225-247) — it depends on `with_env` keeping `snapshot.lexical_env` in sync, which no longer happens. Gate + full Test262.

After Phase 1: `DispatchState.frame` is read only by the bridge (`sync_active_frame`, `refresh_from_active_frame`, `sync_from_asm` reconstruct, the F3 wrapper now-conditional) — i.e. nothing in a semantic body.

### Phase 2 — the bridge rewrite + field deletion (THE −15%, compile-error-driven, ONE validated unit)

6. **`finish_frame` overlay-direct (Task 11, registers.rs:106).** Replace `let frame = self.pop_current_frame();` with: read `cfr = self.current_cfr`, decrement `self.frame_depth`, and derive everything from the overlay/geometry — `flags` ← `self.frame_header(cfr).flags_bits()` (via `FrameFlags::from_raw`), `return_register` ← `self.frame_header(cfr).return_register()`, `registers()` ← `RegisterWindow::new(cfr + HEADER_SLOTS, self.frame_window_len(cfr))`, this/lexical/construct already overlay-read. Add a cfr-based `refresh_running_context_to_caller_cfr(agent, popped_cfr)` (reads `caller_cfr`/realm from the overlay) to replace `refresh_running_context_to_caller(agent, &frame)`. NO reconstruct on the return path. (`pop_current_frame` STAYS for its cold callers: exception unwind exceptions.rs:113, jobs.rs:229, internal_calls.rs:26, vm.rs:2582.) Gate + full Test262.
7. **`sync_from_asm` (slow_path.rs:123)** — once no slow-body reads the snapshot, drop the `frame_dirty` reconstruct block + `frame.set_instruction_offset(pc)`; keep `rust.dispatch.pc = pc`.
8. **Refresh arm (slow_path.rs:311)** — drop the `rust.dispatch.frame_dirty = true` line (field gone).
9. **`refresh_from_active_frame` (dispatch_state.rs:301, α-path)** — drop the `self.frame = reconstruct...` (thin-view fields already set from the overlay).
10. **`sync_active_frame` (dispatch_state.rs:225)** — delete (or rewrite to an overlay park) if any caller survives Phase 1.2.
11. **DELETE `DispatchState.frame` + `frame_dirty`** (dispatch_state.rs:48 + the field). Fix the two constructors (`new_for_dsl_entry` dispatch_state.rs:120, `new_for_dsl_harness` test_helpers.rs) to build the thin view from `(cfr, depth, code, pc)` (entry.rs already has these — entry.rs:49 currently passes a `FrameRecord`; thread the scalars or keep building a temporary record locally just to seed the thin view, then drop it).
12. **Compile-error-drive the removal:** `sync_dispatch_frame`+`write_snapshot_into_backing` (dispatch.rs:201), `refresh_dispatch_frame` (dispatch.rs:255), `advance_dispatch_frame` (dispatch.rs:12), `next_dispatch_instruction_offset` (dispatch.rs:22), `vm.handle_dispatch_result`'s `frame: &mut FrameRecord` param (dispatch.rs:289 — rewrite to take `cfr`/operate on overlay; the throw arm parks `saved_pc` then transfers), `finish_abc_value_result` (dispatch.rs:316), the dead defensive `clear_resume()`s, `FrameView::from_record` (frame.rs:640 — if no callers remain). KEEP: `reconstruct_frame_from_header` (frame()/frames()/GC/debugger), `cfr_of`, `write_header_from_record`/`push_frame_with_header` (push construction), `frame_record_realm`/`caller_context_from_record` (synthetic-frame job callers — verify still used), `trace_all_frame_edges` (GC), `debugger.rs`, `names::load_name` (#[cfg(test)]).
13. `cargo build --workspace --all-targets --all-features` + clippy + `cargo test -p lyng-vm --all-features` (609/0; watch for #[cfg(test)] breakage — `cargo build` does NOT compile test modules, run the vm suite). Then full Test262.

### Phase 3 — validation (the gate) + tidy-up

14. **Full Test262** = baseline: 49729/0/0/3324, variants 95205/0/0. Any delta blocks; bisect within the phase.
15. **A/B re-run (the proof):** rebuild `target/release/lyng` (`cargo build --release -p lyng-cli`), then:
    ```
    target/release/lyng-bench v8suite --samples 7 --lyng-bin /tmp/lyng-base/target/release/lyng --report /tmp/bench-base.md
    target/release/lyng-bench v8suite --samples 7 --lyng-bin target/release/lyng --report /tmp/bench-final.md
    ```
    Confirm Richards/RayTrace/DeltaBlue/NavierStokes recover to **≥ baseline** (≈508/459/386/545). NOTE: `v8suite` writes `reports/lyng/bench-v8.json` as a side-effect — `git restore reports/lyng/bench-v8.json` after.
16. **Tidy-up:** `git worktree remove /tmp/lyng-base`. Confirm `frame_header_offsets_stable` + the `LLINT_STATE_*` offset tests still pass (`cargo test -p lyng-vm --all-features frame_header_offsets_stable`). Remove any `#[expect(dead_code)]` that became reachable/unreachable. Whole-branch review (spec + quality reviewer subagents) vs merge-base `1b39a0ec`, then `superpowers:finishing-a-development-branch` for the merge/PR decision.

## Remaining `&FrameRecord` inventory (the 28, categorized)

- **Phase-1 swap (4):** exceptions.rs:59/86; with_env.rs:7/39.
- **Phase-2 delete/rewrite (bridge):** dispatch.rs:12/22/201/255/289/316; dispatch_state.rs:48(field)/91/126(ctors); the field readers dispatch_state.rs:227/292/311, slow_path.rs:137/144, scope.rs:315, generators.rs:144/175/340.
- **KEEP (construction / synthetic / GC / debug / test):** vm.rs:1404 `cfr_of`, :1654 `write_header_from_record`, :1694 `push_frame_with_header`, :1819 (caller_frame construction — verify), :2007 `frame_record_realm`, :2045 `caller_context_from_record`, :2146 `refresh_running_context_to_caller` (rewrite to add cfr variant; the &FrameRecord one may stay for cold callers); frame.rs:640 `FrameView::from_record` (delete if no callers); state.rs:366 `trace_all_frame_edges` (GC); debugger.rs:59; names.rs:1486 `load_name` (#[cfg(test)]); entry.rs:49 + test_helpers.rs:117 (ctor seeds).

## HAZARDS

- **COLD-FIELD GOTCHA** (cost a 153-file regression earlier): any reconstruct-elimination must preserve cold reads (`handler_cursor`/`resume_*`/`tail_caller`/`parameter_initializer_end_offset`). They live in `frame_cold.get(depth)` — NOT a view-derived record (`frame_record_for_view` zeroes cold). `finish_frame` (Task 11) does NOT read cold off the popped frame (verified — only flags/return_register/this/lexical/construct/geometry), so it's safe; the exceptions swap reads only code/pc; but re-verify if you touch any other reconstruct caller.
- **SYNTHETIC FRAMES:** `realm_of(cfr)`/`frame_header(cfr)`/`cfr_of`/`FrameView::from_record` UNDERFLOW on synthetic frames (`RegisterWindow::new(0,0)`). The async-generator return-completion jobs pass synthetic frames to `caller_context_from_record` — that bridge STAYS. Never route a synthetic-reachable method through `realm_of(cfr)`.
- **Alpha is test-only** (CRITICAL FACT #1) — but `dsl/test_helpers.rs` + the α `dispatch_handlers/` are still COMPILED and run in the vm suite, so deletion must keep them building (the LSP/`cargo test` catches test-module breakage that `cargo build` masks).
- **NEVER wide-parallel subagents on this coupled core** (prior hangs/OOMs). Driver work, sequential. A single SEQUENTIAL scoped subagent per file is OK for mechanical swaps; never for the bridge rewrite.

## OPERATIONAL RULES (16GB machine; a bug once OOM'd it)

- Heavy cmds under the watchdog: `MEMTEST_MAX_SEC=<s> MEMTEST_THRESH_KB=12582912 MEMTEST_LOG=/tmp/<n>.log /tmp/memtest.sh '<cmd>'`. TIGHT `MEMTEST_MAX_SEC=300` for the vm test run. Don't pipe `| tail` into the watchdog cmd (truncates the log) — run raw, then `grep "test result:" /tmp/<n>.log`.
- NEVER run two release builds concurrently (OOM). Build baseline, then branch, then bench — sequentially.
- Per-step gate: `cargo build -p lyng-vm --all-features` (the rtk proxy sometimes returns a STALE "Finished" — if a diagnostic looks off, `touch` the edited file and rebuild to force real cargo output) + `cargo clippy -p lyng-vm --all-features` (no NEW vs the 7) + vm suite 609/0 + the async/generator canary (in the vm suite).
- Test262 at each Phase-1 step + the Phase-2 unit, in the BACKGROUND (`run_in_background: true`): `MEMTEST_MAX_SEC=2700 … 'cargo run --release -p lyng-test262 -- --report /tmp/t262-X.md -j 8'`. ALWAYS `--report /tmp/...` (omitting clobbers `reports/lyng/test262.md`). Confirm Passed/Failed/Panicked = 49729/0/0, variants 95205/0/0. ~release rebuild + run.
- BSD grep has no `\|` alternation — use `rg`/`grep -E`/`grep -F`. The rtk shell-proxy mangles some `rg`/pipe output (collapses paths, garbles matched text, mangles `wc -l` of clippy) — read files directly when output looks off; for clippy count, `grep -iE "errors,"` the proxy's "0 errors, 7 warnings" summary line.
- Avoid compound `git … && …` (permission-denied via the rtk rewrite); run git commands individually. `git restore <path>` works; `git checkout -- <path>` was denied.
- Stage only `crates/` in code commits; commit the plan/handoff docs separately. Commit each green increment. End commit msgs with:
  `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`
- Do this with FRESH, focused context. It is the −15% and the last SP-0b step; after it, SP-0c/SP-1 (asm call entry) per `project_asm_call_frame_rearchitecture` memory.
