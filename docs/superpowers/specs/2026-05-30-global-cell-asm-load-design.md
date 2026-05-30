# Asm-Inline Global Cell Load (mode 7) — Design

**Date:** 2026-05-30
**Status:** Design approved; pending implementation plan
**Author:** brainstormed with Claude
**Parent:** continues `docs/superpowers/specs/2026-05-29-global-property-cells-design.md`
(Global Property Cells M1). This is the asm-inline hit-path slice of that work.

## Problem

The time-attribution profiler (`lyng-bench profile`) measured `LoadGlobal` at
**15.78% of wall-time on isolated Richards** (5-sample run; 51.1M dispatches;
**121 samples/Mdispatch** — ~50× a plain `LoadLocal0` at 2.6, ~4×
`GetNamedProperty` at 32.7) with a **0.03% slow share**. So LoadGlobal is
expensive on its *hit* path, not because it bails.

The Global Property Cells work ("Phase 3") already cell-backs global bindings and
caches, per `(code, feedback_slot)`, *where* a `LoadGlobal` resolves
(`GlobalCellIcState` in `crates/vm/src/vm/ic_state/global_cell.rs`:
`GlobalCellTarget::Cell(PrimitiveValueCellRef)` or `EnvSlot(env, slot)`), guarded
by a coarse `global_structure_generation`. But the hit still runs **entirely in
Rust behind a per-dispatch C-ABI call**: `op_load_global_dsl`
(`crates/vm/src/dsl/handlers/cold.rs:476`) does `call_rust_probe!` →
`try_load_global_rust_probe_for_dsl` (`crates/vm/src/vm/names.rs:663`), which,
**before** even checking the IC, unconditionally does `read_atom_constant`
(name canonicalize, `names.rs:672`) and `find_global_environment_ref`
(env-chain walk, `names.rs:675`), then a `HashMap` lookup
(`global_cell_ic_state`, `feedback.rs:2758`), a generation compare, and finally
the cell deref. The 0.03% slow share confirms the IC almost always hits — so
essentially all 15.78% is fixed per-hit overhead behind a non-inlined call.

The 2026-05-29 M1 design specifies the fix: project the cached resolution into
the asm-readable `PropertyMetadata` (modes `6 = GlobalCellConstant`,
`7 = GlobalCellLoad`) so the handler serves the hit inline. The constant-fold
side (mode 6 + constness lattice + cell-watchpoints/deopt) is the larger,
higher-risk half. **This design lands only the mutable one-load half (mode 7),
for `Cell` targets** — the smallest slice that gets the hit out of Rust and
captures essentially all of Richards' LoadGlobal time.

## Goal

`LoadGlobal` against a cell-backed global resolves, on the steady-state hit, to:
validity-check → one inline cell load → write register → dispatch — with no
C-ABI call, no name re-read, no env-chain walk, no HashMap lookup. Target:
`LoadGlobal` samples/Mdispatch drops from ~121 toward `GetNamedProperty`'s range
(~33) or below, with no correctness change and no `v8suite` regression.

## Scope

**In scope:**
- A new asm `PropertyMetadata` mode `7 = GlobalCellLoad` and the asm hit path in
  `op_load_global_dsl` for it.
- Cold-path projection of mode 7 (+ cached cell ref + cached generation) into the
  site `PropertyMetadata` whenever it resolves to a `GlobalCellTarget::Cell`.
- Asm-reachability of the two reads the hit needs: the live global generation and
  the cell's `stored_value` (new pinned-offset bindings).

**Out of scope (unchanged; stays on the Rust cold path):**
- Constant folding (`mode 6 = GlobalCellConstant`), the constness lattice, and
  cell-watchpoint/deopt machinery. (Deferred follow-up; the bigger win.)
- `EnvSlot` (lexical `let`/`const`/`class`) inline — needs an in-asm TDZ-sentinel
  check and a Rust bail to throw anyway; Richards' 0.03% slow share shows almost
  nothing resolves to `EnvSlot` here, so it stays cold.
- `StoreGlobal` / `AssignGlobal` asm paths (global writes are rare in hot loops).

## Architecture

### Metadata projection & cold-path install

Reuse the existing `PropertyMetadata` triplet already read by the asm shape ICs
(`PROPERTY_METADATA_MODE_OFFSET` / `_HANDLER_BITS_OFFSET` / `_AUX_BITS_OFFSET` in
`crates/vm/src/dsl/reg_convention.rs`). When the cold path
(`try_load_global_rust_probe_for_dsl` / `load_global_with_feedback`) resolves a
site to a `GlobalCellTarget::Cell(cell)`, in addition to today's
`install_global_cell_ic` it projects, into that site's `PropertyMetadata`:

- `mode = 7` (GlobalCellLoad).
- `handler_bits = cell` (the `PrimitiveValueCellRef`).
- `aux_bits = structure_gen` (the generation captured at resolution).

`EnvSlot` resolutions and any non-cell outcome do **not** set mode 7 (the site
keeps whatever non-global-cell mode it had, or stays cold). Mode 7 is therefore a
positive assertion: "this site has a valid cell ref + the generation it was valid
at."

> **Open item (planning):** confirm `7` (and the reserved `6`) do not collide
> with the asm fast-path modes 1–5 already in use, and that a
> `PrimitiveValueCellRef` round-trips losslessly through the `handler_bits` width
> and `structure_gen` (a `u32`) through `aux_bits`. (M1 design open item #2.)

### The asm hit handler

Replace the unconditional `call_rust_probe!` body of `op_load_global_dsl`
(`crates/vm/src/dsl/handlers/cold.rs:476`) with an inline path that mirrors the
existing shape-IC handlers (modes 1–5):

```
op_load_global_dsl(a = target, bx = atom):
    load mode from PropertyMetadata
    if mode != 7: goto .slow
    load live_gen   (§ asm-reachability)
    load cached_gen = aux_bits
    if live_gen != cached_gen: goto .slow
    load cell_ref = handler_bits
    load value = cell_ref.stored_value   (§ asm-reachability)
    write_register(target, value)
    advance_dispatch_frame(instruction_len)
    tail-dispatch            (no Refresh — pure value load)
.slow:
    decode_abx!(a, bx)
    call_slow!(op_load_global_slow_rs, ...)   // existing cold path, unchanged
    dispatch_after_slow!()
```

The cold path is unchanged and remains the fallback for: mode ≠ 7 (never
resolved, or resolved to `EnvSlot`), generation mismatch (structural change),
first execution of a site, and everything the asm path does not handle. The cold
path re-resolves and re-projects mode 7 on the next miss. Because the asm path is
a pure value load that cannot change frames, the hit uses the non-refresh
dispatch (like other pure loads), not `dispatch_after_slow!`.

### Asm-reachability (primary implementation risk)

The hit needs two reads currently only reachable from Rust. Both follow the
existing precedent of pinning agent/heap bases into `LLINT_STATE` for asm
(`object_records` / `object_slots` bases + `RUNTIME_OBJECT_*` offsets, wired in
`crates/vm/src/dsl/backend/aarch64/` and `reg_convention.rs`).

1. **Live global generation.** `global_structure_generation` is a per-env `u32`
   in agent storage (`crates/env/src/agent/environments.rs:562`, bumped at `:577`
   by `bump_global_structure_generation`). Mirror it to a single stable,
   asm-pinned word (a `Vm`/realm field or an `LLINT_STATE` slot) kept in sync
   whenever the bump runs. Executing bytecode is monomorphic per realm (a code
   object resolves globals in exactly one realm), so one pinned word is correct
   for all sites in a run. The asm reads it with one load.

2. **Cell value.** Cells live in a `SlotArena<PrimitiveValueCellRecord,
   PrimitiveValueCellRef>` (`crates/gc/src/arena.rs`); `value_cell(ref)` returns
   the record (`crates/gc/src/mutator.rs:189`), whose `stored_value` is the
   `Value`. The asm needs the value-cell arena base + slot stride + `stored_value`
   field offset to turn a cached `PrimitiveValueCellRef` into a loaded `Value` —
   a new offset binding / DSL primitive analogous to `object_slots`.

> **Fallback if the inline cell load is infeasible/unsafe in asm:** keep the
> *load itself* in a thin Rust probe but eliminate the dominant overhead — the
> `read_atom_constant` name-canonicalize, the `find_global_environment_ref`
> env-walk, and the `HashMap` lookup — by serving mode-7 sites directly from the
> projected `handler_bits`/`aux_bits`. That is a partial win (still one C-ABI
> call) but removes the bulk of the per-hit cost. The plan should treat the fully
> inline path as the target and this as the documented fallback.

### Invalidation

Unchanged from Phase 3 — no new machinery:

- **Value writes** to a mutable global do **not** bump the generation; the cell
  is read live, so cached mode-7 sites stay valid and observe new values.
- **Structural changes** (delete of a configurable global, data↔accessor
  redefine, a new lexical shadowing a var, reconfigure) already call
  `bump_global_structure_generation`. A mode-7 site then fails the asm
  generation compare on its next execution and bails to the cold path, which
  re-resolves (or falls to the semantic path) and re-projects. A freed cell is
  therefore never dereferenced: the generation moved, so the asm load is not
  taken.

This is exactly the guard the Phase-3 Rust probe already performs
(`names.rs:683-686`); this design moves the compare into asm by making the live
generation asm-readable.

## Testing & success criteria

**Correctness (must not regress).** The cold path is unchanged, so the new risk
is confined to (a) the asm hit returning the correct value and (b) bailing in
every case it must. Existing global test262 must stay green: global
`var`/`let`/`const`/`class`, TDZ, `globalThis` reflection (`Object.keys`,
`getOwnPropertyDescriptor`, for-in), `delete` of configurable globals,
`Object.defineProperty` data↔accessor/attr toggles, sloppy-mode implicit globals,
eval-created globals, redeclaration, function hoisting.

**New unit tests:**
- Mode-7 projection on cold-path cell resolution; subsequent asm hit returns the
  same value as the semantic path.
- Mutable global: write a new value, next `LoadGlobal` (asm hit) observes it
  (generation unchanged, cell read live).
- Structural change while a site is mode-7 (`delete`, data→accessor redefine, new
  lexical shadowing a var): generation bumps → asm bails → cold path re-resolves
  → re-projects (or correctly falls to semantic); no stale/dangling read.
- An `EnvSlot` (lexical) global never takes mode 7.
- Layout test pinning the new pinned-generation word offset and the value-cell
  arena base/stride/`stored_value` offsets (mirroring
  `crates/vm/tests/dispatch_counters_layout.rs`).

**Performance acceptance:**
- `lyng-bench profile --filter Richards`: `LoadGlobal` samples/Mdispatch drops
  from ~121 toward ~33 (GetNamedProperty range) or below; its time share falls
  correspondingly; slow share stays ~0.
- Isolated `v8suite` scores do not regress.
- Bundled-suite behavior (the M1 O(n) motivation) is unaffected — this slice only
  changes the hit-path cost, not resolution/caching.

## Key source locations

- `crates/vm/src/dsl/handlers/cold.rs:476` — `op_load_global_dsl` (the handler to
  rewrite; currently `call_rust_probe!`).
- `crates/vm/src/vm/names.rs:663` — `try_load_global_rust_probe_for_dsl` (cold
  path; add mode-7 projection on `Cell` resolution).
- `crates/vm/src/vm/ic_state/global_cell.rs` — `GlobalCellTarget`,
  `GlobalCellIcState` (the cached resolution this projects into asm metadata).
- `crates/vm/src/vm/feedback.rs:2758` — `global_cell_ic_state` /
  `install_global_cell_ic` / `clear_global_cell_ic`; `PropertyMetadata`
  projection (add mode 7).
- `crates/vm/src/dsl/reg_convention.rs` — `PROPERTY_METADATA_*` offsets;
  `LLINT_STATE` base bindings (add pinned generation + value-cell arena bindings).
- `crates/vm/src/dsl/backend/aarch64/` — feedback/object backends to mirror for
  the mode-7 hit and the cell-load primitive.
- `crates/env/src/agent/environments.rs:562,577` —
  `global_structure_generation` / `bump_global_structure_generation` (mirror to
  the pinned word).
- `crates/gc/src/arena.rs`, `crates/gc/src/mutator.rs:189` — `value_cells`
  arena / `value_cell(ref)` / `PrimitiveValueCellRecord.stored_value` (the cell
  read to express in asm).
- `crates/vm/src/dsl/backend/aarch64/control.rs:360` — `call_rust_probe!` (the
  per-dispatch call being replaced on the hit path).

## Open items to resolve during planning

1. **Metadata encoding.** Confirm mode `7` is free vs asm modes 1–5; confirm a
   `PrimitiveValueCellRef` fits `handler_bits` and `u32` generation fits
   `aux_bits` losslessly. (Reserve `6` for the deferred constant-fold mode.)
2. **Pinned-generation home & sync.** Decide where the mirrored generation lives
   (a `Vm`/realm field vs an `LLINT_STATE` slot) and ensure every
   `bump_global_structure_generation` (and realm/env switch) updates it. Verify
   the monomorphic-per-realm assumption holds for every path that runs bytecode.
3. **Value-cell arena asm binding.** Pin the arena base / slot stride /
   `stored_value` offset and decide whether a generic "load Value from cell ref"
   DSL primitive is warranted (it would also serve the deferred mode 6 and
   `StoreGlobal`). Confirm the GC cannot move a cell's slot under a live mode-7
   site without bumping the generation (compaction/sweep interaction).
4. **Fallback decision.** If the fully-inline cell load is infeasible/unsafe in
   asm, fall back to the thin-probe variant (§ asm-reachability) and record the
   partial win; do not block the slice on a perfect inline load.
