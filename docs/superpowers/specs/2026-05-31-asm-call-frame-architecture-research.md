# Asm Call-Frame Architecture — JSC Research Synthesis & Long-Term Direction

**Date:** 2026-05-31
**Status:** Research synthesis — informs SP-0 of the inline-asm call-path program
**Source:** WebKit JSC (`/Users/sondre/dev/WebKit/Source/JavaScriptCore`), studied via
three parallel research agents (CallFrame/context model; LLInt asm call/return;
scope/env + stack model).

## Why this exists

Stage 2 of the V8-v7 perf work is an inline-asm call path for environment-free
callees — the data-backed broad lever (Richards `Call0`+`TailCall`+`Return` ≈ 45%;
RayTrace `Call2`+`ReturnUndefined` ≈ 26%). Its enabler (SP-0) is making call-frame
setup doable in asm. Lyng today pushes **two** growable Rust structures per call —
`Vec<FrameRecord>` (Vm) + `Vec<ExecutionContext>` (Agent) — neither asm-addressable.
We studied how JSC does it to pick the correct long-term architecture rather than the
fastest patch.

## What JSC does (decisive findings, cited)

1. **No execution-context object/stack.** A `CallFrame` is `Register*` reinterpreted
   (`interpreter/CallFrame.h:189,217`) — a slice of one contiguous register-file stack.
   The "execution context" concept does not exist as a struct; the CallFrame *is* the
   activation record.

2. **Fixed asm-addressable header, then args, then locals.** `CallFrameSlot`
   (`CallFrame.h:176-182`): `[0..1] CallerFrameAndPC{callerFrame,returnPC}`, `[2] codeBlock`,
   `[3] callee`, `[4] argumentCountIncludingThis`, `[5] this`, `[6..] args`. Locals are
   negative `VirtualRegister` offsets from the frame pointer
   (`interpreter/CallFrameInlines.h:37-41`). Fixed displacements are *why* the LLInt can do
   `Callee[cfr]`, `CodeBlock[cfr]` as constant-offset loads.

3. **Only ~5 things are stored; the rest is derived.**
   - Stored header slots: codeBlock, callee, argCount, this/newTarget (newTarget reuses the
     this-slot, `CallFrameInlines.h:155`), + caller/returnPC.
   - **realm/globalObject: derived** — `jsCallee()->realm()` or `codeBlock->globalObject()`
     (`CallFrameInlines.h:73-78`), never stored per frame.
   - **scope/lexical/var/private env:** a heap `JSScope` chain (`m_next`,
     `runtime/JSScope.h:93`) pointed to from a *local register* chosen per code unit
     (`CodeBlock::scopeRegister()`), seeded at entry from the callee's captured scope.
   - **referrer/sourceOrigin: derived** by stack walk through `code → ownerExecutable`.

4. **Scope threading = cheap.** `op_get_scope` is `loadp Callee[cfr],t0; loadp
   JSCallee::m_scope[t0],t0` (`LowLevelInterpreter64.asm:871-875`). The callee carries its
   captured scope (`runtime/JSCallee.h:63-95`). A no-activation function never re-points the
   scope register — it stays equal to the captured outer scope; **zero allocation**.

5. **Environment-free / no-activation is a static compile-time bit.**
   `needsActivation() = hasCapturedVariables() || (Eval|With)` (`parser/Nodes.h:1985-1986`);
   bytecode-gen folds in arguments/rest/default/destructuring/generator/async/debug
   (`bytecompiler/BytecodeGenerator.cpp:466-480,529-531,590-600,607-717`). No-activation ⇒
   all locals live in registers, scope register = captured outer scope. **This is exactly
   Lyng's `!needs_environment()` plus the arguments/rest/generator exclusions.**

6. **Regular calls copy NO arguments (overlapping setup).** `callHelper`
   (`LowLevelInterpreter64.asm:2467-2505`): the callee frame is carved from the caller's
   window at a compiler-chosen base (`m_argv`) so the already-evaluated arg VRs physically
   sit where the callee expects them. The caller writes only **3 header words**
   (argCount, callee, codeBlock) + repoints `sp`; the callee prologue writes
   callerFrame+returnPC. Only **tail calls** slide args (`prepareForTailCall`, `.asm:1347-1404`).

7. **Linked-call cache (CallLinkInfo) removes Rust from the steady-state call.** Inline
   per-callsite `{expected callee, cached codeBlock, cached entrypoint}`
   (`bytecode/CallLinkInfo.h:310-312`). Asm compares actual vs cached callee; on hit, store
   cached codeBlock + indirect-jump to cached entry — **zero C++** (`...64.asm:2486-2517`).
   On miss, jump to a link thunk that calls C++ once to populate the cache. **Uniform
   `call <entrypoint-reg>`** for every callee kind (JS-eligible, ineligible, native) — only
   the cached entrypoint differs.

8. **Cheap prologue + return, no C++.** Prologue (`.asm:1636-1755`): push cfr/PC, save
   callee-saves, **soft-stack-limit check** sized for the whole frame, zero-fill locals,
   seed scope. Return (`op_ret`/`doReturn`, `64.asm:2606`, `.asm:1798`): value→reg, restore
   CSRs, collapse frame (`sp=cfr`, pop), `ret`; the caller's return site stores the result
   into its dst VR (`dispatchAfterRegularCall`, `64.asm:84-95`).

9. **Single pre-reserved bump stack — the safety foundation.** The JS stack is one
   pre-reserved region with a fixed base (native thread stack, or `CLoopStack`
   `PageReservation::reserve(maxPerThreadStackUsage)`, `interpreter/CLoopStack.cpp:55-72`);
   frames bump-allocate (`sp = cfr - frameSize`); a **soft limit** above the hard limit
   (`VM::softStackLimit`) is checked once per prologue, leaving slack for slow-path C++.
   **Never realloc** — that is what makes asm pointer-bump frame push safe. A growable
   `Vec<FrameRecord>` cannot provide this (relocation invalidates all live frame/scope ptrs).

10. **Native↔JS boundary:** adjacent `VM::topCallFrame` + `topEntryFrame` (`runtime/VM.h:391`),
    an `EntryFrame`/`VMEntryRecord` saving prev tops (`interpreter/VMEntryRecord.h:38-61`), and
    a `ProtoCallFrame` to stage entry (`interpreter/ProtoCallFrame.h:42-96`). Maps to Lyng's
    Rust↔asm boundary.

## Recommended long-term architecture for Lyng

**Adopt C-as-arena with B's derive-discipline inside it.** Concretely, SP-0 becomes:

- **One pre-reserved, never-realloc'd JS value/frame stack** (bump-pointer; fixed base).
  Replace the growable `Vec<FrameRecord>` as the frame backing. Reuse / extend the existing
  `register_stack` (already asm-addressed via `LlIntState.frame_const_base`) so frames live
  *in* the register file with a fixed header, rather than as fat structs in a side Vec.
- **Drop `Vec<ExecutionContext>` entirely.** Store only the irreducible header
  (caller link, return/resume state, code, callee, argCount, this/new_target) at pinned
  asm offsets (`repr(C)` / `offset_of!` constants, mirroring `VM_*_OFFSET`). **Derive**
  realm (from callee/code) and referrer (stack walk); keep the scope in a frame-local slot
  seeded from the callee's captured environment (Lyng's `function_data.environment()`).
- **Soft-limit prologue check** sized for the whole callee frame; one slow-path entry to
  grow/throw `RangeError`. This is the safety property that lets asm push frames branch-free.
- **Environment-free eligibility** stays a static compile-time bit = `!needs_environment()`
  + no arguments/rest/generator/async + strict/global this (lexical handled later).
- Later phases layer in the **linked-call cache** and **overlapping arg setup** (the latter
  needs compiler arg-VR layout changes) for the steady-state zero-Rust call.

## Revised SP-0 reality & decomposition

SP-0 is **not a small enabler** — it is a re-architecture of Lyng's frame/stack/context
model toward the JSC register-file shape. Recommended sub-phases:

- **SP-0a — Eliminate the separate ExecutionContext:** make the FrameRecord (or the new
  frame header) the single source of truth; derive realm/referrer; re-route the handful of
  `execution_contexts` readers through the current frame. Pure refactor, no asm yet; behavior-
  preserving; independently testable (Test262 must stay 100%).
- **SP-0b — Pre-reserved bump frame stack + pinned asm-addressable header:** move frames into
  a fixed-base, capacity-bounded region with a `repr(C)`/offset-pinned header; soft-limit
  prologue check in Rust first. Still no asm call; just the storage + invariants.
- **SP-0c — (optional pre-req) compiler arg-VR layout** for overlapping arg setup, if SP-1
  needs it.

Then SP-1 (asm call entry) and SP-2 (asm return) build on SP-0a+SP-0b, and SP-3 broadens.

**Magnitude:** this is a multi-spec, multi-phase architectural program touching vm (frames,
register stack, dispatch), env (Agent context stack removal), and compiler (scope register,
arg layout, env-free bit). Each sub-phase is its own spec → plan → implementation cycle.
