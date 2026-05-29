# Global Property Cells — Milestone 1 Design

**Date:** 2026-05-29
**Status:** Design approved; pending implementation plan
**Author:** brainstormed with Claude

## Problem

Global variable/property access degrades linearly with the number of global
declarations in a realm. In the external `js-engine-benchmark` suite (all nine
V8-v7 benchmarks bundled and run in one process), RayTrace scores **~55** versus
**~262** when run alone — a ~4.8x regression that does not reproduce in our
internal `v8suite` because that harness isolates each benchmark in its own
subprocess with a clean global scope.

### Root cause (verified)

1. Global `var`/`function` declarations become own properties of the global
   object (`crates/vm/src/vm/global_script.rs`).
2. Once ≥ `BULK_GLOBAL_BINDING_DICTIONARY_THRESHOLD` (64) are bulk-declared (or
   the object passes the 128-property incremental limit), the global object is
   forced into **dictionary mode** (`ensure_global_object_dictionary`).
3. Lyng's inline caches **refuse to cache dictionary-mode receivers** —
   `plan_named_property_cache_entry` returns `None` when
   `receiver_header.flags().uses_named_property_dictionary()`
   (`crates/objects/src/internal_methods/property_cache.rs:51`).
4. With no plan, the `LoadGlobal` site promotes to **megamorphic** and never
   recovers; every global read falls to the full slow path:
   `get_global_property_binding_with_context` → `get_own_property` → `HashMap`
   hash + probe, on every access.
5. RayTrace reads many distinct globals per pixel (`new Flog.RayTracer.Color(...)`,
   `Math`, constructors). As the bundled suite piles every benchmark's
   declarations into one dictionary, these scattered hashmap probes become
   cache-miss-bound; cost climbs with dictionary size.

Evidence: the slowdown reproduces by injecting N empty global declarations
before a clean RayTrace (3000 var+func ≈ score 55.9, matching the full suite);
the per-read cliff lands exactly at 64 globals (50 globals → 890 ms, 100 globals
→ 1977 ms for the same single-global hot loop); GC was ruled out by profiling
(GC self-time is *lower* in the slow run — 1.6% vs 11%); the extra time is
concentrated in the inlined dispatch slow-path hashmap lookup.

A separate but related O(n) exists for global **lexical** bindings:
`global_lexical_binding` linear-scans `lexical_bindings: Vec<...>`
(`crates/env/src/agent/environments.rs:489`), and that scan runs at the top of
`load_global_with_feedback` on every `LoadGlobal`.

## Goal

Make global reads fast and independent of global-scope size, with constant
folding for write-once globals. This is **Milestone 1** of a larger direction:
property cells as the cache primitive. M1 delivers the cell foundation and the
global IC; **M2** (out of scope here) generalizes the cell-aware IC to ordinary
`GetNamedProperty`/`SetNamedProperty` on slow-mode objects.

### Why cells, and why staged

- A **property cell** (a stable, separately-tracked box holding one `Value`) is
  the primitive real engines use for globals (V8 `PropertyCell`, JSC global
  `WatchpointSet`s). Caching the cell gives stable-address access that survives
  unrelated global declarations, and unlocks **constant-value watchpoints**:
  folding a write-once global into the IC as a constant, deopting on
  reassignment. Constant folding is the larger win (it removes the load
  entirely for `Math`, constructors, `const`, never-reassigned `var`).
- Cell-backing is **not** the right representation for *every* dictionary
  object. Objects that dictionary due to **churn** (the 8-add/delete threshold)
  mutate constantly; cell-backing them would allocate/free a heap cell per
  add/delete. So cell-backed-vs-plain is a **permanent per-object policy axis**,
  not throwaway scaffolding — the global object is simply the always-on case.
- Staging isolates risk: the global IC sees an effectively **monomorphic**
  receiver (the realm's global object), so M1 validates the full
  cell + constness + watchpoint/deopt machinery end-to-end against RayTrace and
  the full test262 suite with a small blast radius, before M2 touches the
  hottest code path in the engine (general polymorphic/megamorphic property
  access and asm-handler-mode coexistence). The global IC remains a distinct
  fast path even in the end state (separate opcodes, as in V8/JSC), so little
  rework.

## Scope

**In scope (M1):**
- A general **cell-backed dictionary storage** mechanism (a new dictionary
  payload variant holding a `ValueCellRef`), built reusable but **enabled via an
  object flag only for the global object**.
- Cell-backing of **all** global bindings: `var`/`function` (on the global
  object) and `let`/`const`/`class` (in the global environment).
- A new **global IC path** for `LoadGlobal`/`StoreGlobal`/`AssignGlobal` that
  caches the cell and constant-folds write-once globals.
- Constness lattice + cell-keyed watchpoint/deopt machinery.
- Removal of the global lexical `Vec` linear scan in favor of a `name→cell` map.

**Out of scope (M2 and later):**
- Cell-aware IC for general `GetNamedProperty`/`SetNamedProperty`.
- Cell-backing ordinary (non-global) dictionary objects, and the heuristic for
  which dictionaries to cell-back.

## Architecture

### Data model — every global binding resolves to a heap `ValueCell`

A cell is the existing heap `PrimitiveValueCellRef` (`crates/gc`): a GC-traced,
write-barriered, rooted box holding one `Value`, currently used to box the
primitive inside wrapper objects. It is reused as the global property cell.

Two storage homes, matching where bindings already live:

- **`var`/`function` → the global object's dictionary.** Today a dictionary
  entry's payload is `NamedPropertyValue::Data(Value)`. We add a **cell-backed
  payload variant** so a cell-backed entry holds a `ValueCellRef` plus the
  existing `attrs`. The global object's `[[Get]]/[[Set]]/[[GetOwnProperty]]/
  [[Delete]]/[[DefineOwnProperty]]` dereference the cell. Cell-backing is gated
  by a new object flag (e.g. `CELL_BACKED_DICTIONARY`) set **only on the global
  object** in M1; ordinary dictionaries keep inline-`Value` payloads. This
  introduces two dictionary payload representations behind one enum (contained
  branching in the dictionary get/set/define/delete paths) — an intentional,
  permanent axis (see "Why cells, and why staged").

- **`let`/`const`/`class` → the global environment's lexical table.** Today
  `lexical_bindings: Vec<GlobalLexicalBindingRecord>` is linear-scanned. We
  change it so each binding owns a cell and lookups go through a `name→cell`
  map (eliminating the O(n) `global_lexical_binding` scan). TDZ is a sentinel in
  the cell until initialization.

**Resolution + shadowing:** a global name can resolve to a lexical cell *or* a
var cell (lexical shadows var). The cold path resolves which and caches that
specific cell; a later structural change (e.g. a `let` added shadowing a `var`)
invalidates via the watchpoint mechanism.

### The constness lattice

Constness lives **not** in the cell record (no spare field) but in a
**cell-keyed registry** mirroring the existing shape `watchpoint_sets:
HashMap<ShapeId, WatchpointSet>`:

```
cell_watchpoints: HashMap<ValueCellRef, CellWatchpointSet>
CellWatchpointSet {
    state: Constness,
    dependents: Vec<(CodeRef, FeedbackSlotId, generation)>,
}
```

This reuses the generation / `clear_ic_slot_if_generation_matches` machinery and
generalizes to M2. The lattice is monotonic (V8 `PropertyCell`-style):

- **Uninitialized** — `var` pre-assignment (value `undefined`) or `let`/`const`
  in TDZ (TDZ sentinel; `LoadGlobal` throws ReferenceError). Not foldable.
- **Constant(v)** — assigned exactly once; value `v`. Foldable.
- **Mutable** — assigned a second, *different* value. Reads load the cell.

Store transitions:

| From | store(v) | Result |
| --- | --- | --- |
| Uninitialized | any `v` | Constant(v) |
| Constant(v) | same `v` | Constant(v) (idempotent — e.g. re-run) |
| Constant(v) | `w ≠ v` | Mutable + **fire cell watchpoint** (deopt folded sites) |
| Mutable | any | Mutable |

Net: `const C=…`, builtins (`Math`), and `var x=5` never reassigned all fold;
reassigned `var`/`let` degrade once and then stay cheap.

### The global IC path

`LoadGlobal` is **monomorphic by construction** — a code object belongs to one
realm, so a site always resolves against the same global environment. The cell
IC is therefore *simpler and faster* than the shape-based IC: no receiver
polymorphism, and correctness comes from invalidation rather than a per-hit
shape compare.

- **Cold path (miss):** resolve the name once — lexical `name→cell` map first
  (lexical shadows var), else the global object's cell-backed dictionary entry.
  Read constness from the registry. Install a `GlobalCell` IC entry and register
  the site as a dependent of the cell (for deopt).
- **Hit, constant:** new metadata mode (e.g. `6 = GlobalCellConstant`);
  `handler_bits` = the folded `Value`. Hit = validity/generation check → return
  value. **Zero loads, no shape compare.**
- **Hit, mutable:** mode `7 = GlobalCellLoad`; `handler_bits` = the
  `ValueCellRef`. Hit = validity check → one load of `cell.stored_value()`.
- **Store side (`StoreGlobal`/`AssignGlobal`):** caches the cell ref; hit writes
  the cell (existing barrier). If already `Mutable`, the asm fast path just
  writes; `Constant`/`Uninitialized` falls to a helper that runs the lattice
  transition (and fires the watchpoint on degrade). Global stores are rare in
  hot loops, so this is not a steady-state cost.

This also retires the `lexical_bindings` linear scan and the unconditional
`lookup_global_lexical_binding_ref` at the top of `load_global_with_feedback`:
resolution happens once on the cold path; hits skip it entirely.

> **No-shape-compare safety:** a code object resolves globals in exactly one
> realm, so the global IC site is monomorphic. With every structural change
> invalidating dependents, a cached cell ref is always valid; the only per-hit
> guard is the standard IC mode/generation validity check. Cross-realm edge
> cases fall to the slow path.

### Invalidation

A cached cell ref stays valid as long as the binding keeps resolving to that
cell. **Value changes need no invalidation** (stable address; mutable reads load
the current value). Only two classes invalidate, both routed through the same
drain → `clear_ic_slot_if_generation_matches` path the shape watchpoints use,
keyed by cell:

1. **Constness degrade** (Constant→Mutable, 2nd distinct write) — fire the cell
   watchpoint, drain dependents, clear folded sites; they re-plan as
   `GlobalCellLoad`.
2. **Structural changes:**
   - **Delete** (configurable global) — drain dependents *before* freeing the
     cell (no dangling reads), remove the entry, free the cell.
   - **Redefine as accessor** (`Object.defineProperty(globalThis,'x',{get})`) —
     drain; re-plan falls to the normal accessor path.
   - **Reconfigure to non-writable** — drain store-side dependents so
     `StoreGlobal` stops fast-writing; loads still fine.
   - **New lexical shadowing an existing var** (`let x` where `var x` existed) —
     the name now resolves to a different cell; drain the old var cell's
     dependents. Declaration-time, rare.

Implementation: a `cell_watchpoints` registry + `fire_cell_watchpoints(cell)`
mirroring `fire_watchpoints_for_shape`, with a `CellInvalidationObserver
{ code, slot, generation }` reusing the existing clear call.

### Reflective access & spec semantics

Cell-backing must be invisible to the spec. **Contract:** any operation that
reads/writes a global's *value* goes through the cell; any operation that
changes the binding's *existence / kind / attrs* drains the cell's IC
dependents.

- `[[Get]]`/`[[GetOwnProperty]]` deref the cell; attrs come from the dictionary
  entry. `[[Set]]` writes the cell + runs the constness transition. `[[Delete]]`
  drains + frees. `[[DefineOwnProperty]]` updates attrs; data→accessor drops
  cell-backing (drain dependents).
- `[[OwnPropertyKeys]]`/`Object.keys`/for-in unchanged — iterate entries; the
  cell is just where the value lives.
- **Lexical bindings never appear on `globalThis`** (per spec) — they live in
  the global env, so reflection is unchanged. **TDZ** = sentinel in the lexical
  cell; `[[Get]]`/`LoadGlobal` throw ReferenceError on it.
- **Sloppy-mode implicit globals** (`x = 1`) and **eval-created globals** use
  the normal var-cell creation path. **`with`/dynamic scopes** force `LoadName`,
  already a separate non-cell slow path, unaffected.

### GC / rooting

Cells are heap `ValueCell`s. Edges to trace:

- The global object's cell-backed dictionary entries — each holds a
  `ValueCellRef` (a heap edge). The dictionary lives in agent-side
  `ObjectMetadata`; tracing must visit these refs. **Open item:** confirm and,
  if necessary, extend how agent-side dictionary payload values are traced today
  (current greps did not locate explicit tracing; cell-backing makes the edge
  explicit and must be correct).
- The global env's `name→cell` map — cell refs traced as part of the rooted
  global environment.
- Each cell's `stored_value` — already traced by existing `ValueCell` machinery.
- The `cell_watchpoints` registry holds `ValueCellRef` keys that must **not**
  keep cells alive — drop entries on cell free (mirror
  `sweep_invalidated_watchpoint_sets`).

## Testing & success criteria

**Correctness (must not regress):** full test262, focused on:
- global `var`/`let`/`const`/`class`, TDZ
- `globalThis` reflection (`Object.keys`, `getOwnPropertyDescriptor`, for-in)
- `delete` of configurable globals
- `Object.defineProperty` on `globalThis`: data↔accessor conversion, writable /
  configurable / enumerable toggles
- sloppy-mode implicit globals, eval-created globals, redeclaration, function
  declaration hoisting

**New unit tests:**
- cell IC hit/miss
- constant fold, then deopt on 2nd distinct write
- invalidation on delete / accessor-redefine / non-writable / lexical-shadowing
- TDZ throw
- `const` stays folded

**Performance acceptance:**
- bundled external suite RayTrace recovers from ~55 toward ~260
- the global-count sweep flattens (no O(n) per-global degradation)
- isolated `v8suite` numbers do not regress
- a write-once-global microbenchmark exercises the zero-load constant path

## Key source locations

- `crates/objects/src/internal_methods/property_cache.rs:51` — dictionary IC
  bail (the gate M2 lifts; M1 adds the global cell path around it).
- `crates/vm/src/vm/names.rs:523` — `load_global_with_feedback` (cold-path
  resolution + IC install; lexical scan to retire).
- `crates/vm/src/vm/global_script.rs` — global declaration instantiation
  (var/function → global object; bulk-dictionary threshold).
- `crates/env/src/agent/environments.rs:481` — `global_lexical_binding` linear
  scan (to replace with `name→cell` map).
- `crates/objects/src/object_metadata.rs:234` — `NamedPropertyStorage` /
  `NamedPropertyDictionary` (add cell-backed payload variant).
- `crates/objects/src/watchpoint.rs`, `crates/objects/src/runtime.rs:60`
  (`watchpoint_sets`) — watchpoint machinery to mirror for `cell_watchpoints`.
- `crates/vm/src/vm/feedback.rs` — IC state, `PropertyMetadata` projection
  (add `GlobalCellConstant`/`GlobalCellLoad` modes), `clear_ic_slot_if_generation_matches`.
- `crates/gc` — `alloc_value_cell`, `mark_value_cell`, `ValueStoreTarget::ValueCell`.

## Open items to resolve during planning

1. **Dictionary payload GC tracing** — pin down exactly how agent-side
   dictionary values are kept alive today; ensure cell refs in cell-backed
   entries are traced. (Load-bearing for correctness.)
2. **`PropertyMetadata` mode coexistence** — confirm free mode-byte values and
   that the new global-cell modes don't collide with the asm fast-path modes
   1–5 already in use.
3. **Constness storage for M2** — M1 keeps constness in the cell-keyed registry;
   confirm this generalizes cleanly when M2 cell-backs arbitrary dictionaries
   (vs. extending the cell record).
