# Spec 2 Phase D — Re-home IC state machine + delete legacy storage

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task.

**Goal:** Move the IC state machine (Uninit→Mono→Poly→Mega transitions) off `FeedbackVector`/`FeedbackSiteState`/`NamedPropertyFeedback` and onto per-kind side-tables on `Vm`. Then delete `feedback_flat_storage`, `FeedbackEntry`, `mirror_flat_slot`, `mirror_metadata_slot`, the debug equivalence assertion, `FeedbackVector`, `FeedbackSiteState`, and friends.

**Architecture:** Per-kind side-table HashMaps on `Vm`, keyed by `(CodeRef, FeedbackSlotId)` — same pattern Phase B established for `Vm::polymorphic_chains`. Lazy: only allocates for slots that actually warm up. `PropertyMetadata` (32B) stays asm-canonical; the Rust-only state-machine fields move to `Vm::property_ic_states`.

**Spec:** `docs/superpowers/specs/2026-05-26-spec-2-ic-jsc-migration-design.md` §6.

**Exit criteria** (spec §6.4 + §1):
- `cargo test --workspace` green throughout.
- `feedback_flat_storage`, `mirror_flat_slot`, `FeedbackEntry`, `FeedbackVector`, `FeedbackSiteState`, `NamedPropertyFeedback`, `with_feedback_slot_mut`, `mirror_metadata_slot`, `debug_assert_metadata_matches_flat` all absent (grep returns no matches).
- D1-D8 tests pass.
- V8 microbench within ≤1% of pre-Phase-D baseline (and ideally recovers the Phase C regression).

---

## Context — post-Phase-C state

Per recon (2026-05-27):

| Concept | Location | Today (post-Phase-C) |
|---|---|---|
| `Vm::feedback_vectors` | `crates/vm/src/vm.rs:168` | `Vec<FeedbackVector>`, semantic state machine. |
| `FeedbackVector` | `crates/vm/src/vm/feedback.rs:1988` | `sites: Vec<Option<FeedbackSiteState>>`. |
| `FeedbackSiteState` | `crates/vm/src/vm/feedback.rs:902` | 6-variant enum: `Arithmetic`, `Comparison`, `NamedProperty`, `KeyedProperty`, `Call`, `Construct`. |
| `NamedPropertyFeedback` | `crates/vm/src/vm/feedback.rs:668` | ~104B: `execution_count`, `cache_state`, `entry_count`, `entries[POLY_LIMIT]`, `monomorphic_own_data_handler`, `monomorphic_proto_data_handler`, `polymorphic_own_data_handlers[POLY_LIMIT]`, `generation`. |
| `with_feedback_slot_mut` | `crates/vm/src/vm/feedback.rs:2122` | Canonical access; 16 direct callers + 1 in vm.rs. Triggers `mirror_flat_slot` + `mirror_metadata_slot` after every mutation. |
| `mirror_flat_slot` + `mirror_metadata_slot` | `feedback.rs:2153`, `2200` | 20 callsites total. Paired. |
| `debug_assert_metadata_matches_flat` | `feedback.rs:2277` | Property kind only; compares MetadataTable vs FeedbackEntry. |
| `feedback_flat_storage` | `crates/vm/src/vm.rs:180` | `Vec<Box<[FeedbackEntry]>>`. No Rust readers; only `mirror_flat_slot` writes (for debug assert). |
| `FeedbackEntry` | `crates/vm/src/dsl/feedback_flat.rs:74` | 64B repr-C. Asm-read until Phase C.4 flipped to MetadataTable. Now write-only scaffolding. |
| `Vm::polymorphic_chains` | `crates/vm/src/vm.rs:173` | Phase B: `HashMap<(CodeRef, FeedbackSlotId), PolymorphicChain>` for entries `[POLY_LIMIT..8]`. |
| `Vm::metadata_tables` | `crates/vm/src/vm.rs` (post-C.1) | Per-code `MetadataTable` buffers; asm reads via x21. |
| `drain_llint_scalar_feedback` | `feedback.rs:2493` | Drains `ArithMetadata.{observed_bits,execution_count}` after each script execution. Bridge to warmup tiering. Survives Phase D. |
| `FeedbackSiteState::Arithmetic` | `feedback.rs:658` | Vestigial: only `execution_count`. Asm writes ArithMetadata directly; the variant has no readers post-Phase-C. |
| `FeedbackSiteState::Comparison` | similar | Likely vestigial. Verify during D.1.0. |

### Key insight (saves work)

`Arithmetic` and `Comparison` variants are already vestigial after Phase C.4 — the asm `record_*` writes go directly to `ArithMetadata`/`ComparisonMetadata` (the latter has no asm callers yet, but the variant pattern is the same). Phase D deletes them outright, no state-machine rehoming needed.

The substantive work is for `NamedProperty`, `Call`, `Construct`, `KeyedProperty` — each gets a side-table.

---

## File map

| File | New / existing | Phase D role |
|---|---|---|
| `crates/vm/src/vm/ic_state/mod.rs` | NEW | Module root + re-exports. |
| `crates/vm/src/vm/ic_state/property.rs` | NEW | `PropertyIcState` (cache_state, entry_count, entries, monomorphic handlers, polymorphic_own_data_handlers, generation). Methods: `install_*`, `transition_to_mega`, `clear`, `bump_generation`. |
| `crates/vm/src/vm/ic_state/call.rs` | NEW | `CallIcState` (callee tracking, mono→mega for calls). |
| `crates/vm/src/vm/ic_state/construct.rs` | NEW (or fold into call.rs) | `ConstructIcState`. |
| `crates/vm/src/vm/ic_state/keyed_property.rs` | NEW | `KeyedPropertyIcState`. |
| `crates/vm/src/vm.rs` | existing | Add `property_ic_states`, `call_ic_states`, `construct_ic_states`, `keyed_property_ic_states` HashMaps. Accessors. GC sweep helpers mirroring `prune_dead_code_polymorphic_chains`. |
| `crates/vm/src/vm/feedback.rs` | existing | Replace slow-path callers' `with_feedback_slot_mut(...)` invocations with direct side-table access. Delete `FeedbackVector`, `FeedbackSiteState`, `NamedPropertyFeedback`, etc. |
| `crates/vm/src/dsl/feedback_flat.rs` | existing | DELETED entirely. |
| `crates/vm/src/vm/install.rs` | existing | Stop allocating `feedback_vectors` and `feedback_flat_storage`. Keep `metadata_tables` allocation. |
| `crates/vm/src/vm/feedback/snapshot.rs` (if exists) | existing | Stub `FeedbackVectorSnapshot` returning empty/defaults for the Phase D window; Phase E re-implements via per-kind status. |
| `crates/vm/src/tests/feedback.rs` + `tests/inline_caches.rs` | existing | Adapt tests that read `vm.feedback_vector(code)` to either (a) read from the new side-tables, or (b) `#[ignore]` with `// TODO(Phase E): port to status API`. |

---

## Implementation order — bottom-up by kind

The plan executes **one kind at a time**, then deletes legacy storage last. This way each migration is validated independently and only the smallest blast radius is at risk per task.

```
D.1 — Re-home state machine, kind by kind:
  D.1.0 Delete vestigial Arithmetic + Comparison variants from FeedbackSiteState
  D.1.1 PropertyIcState side-table (the big one)
  D.1.2 CallIcState + ConstructIcState
  D.1.3 KeyedPropertyIcState

D.2 — Delete legacy storage:
  D.2.1 Delete debug_assert_metadata_matches_flat
  D.2.2 Delete mirror_flat_slot + mirror_metadata_slot callsites (and the helpers)
  D.2.3 Delete feedback_flat_storage + FeedbackEntry + dsl/feedback_flat.rs
  D.2.4 Delete FeedbackVector + FeedbackSiteState + remaining payloads
  D.2.5 Stub FeedbackVectorSnapshot / Footprint for Phase E

D.3 — Verification:
  D.3.1 D1-D4 state-machine tests
  D.3.2 D5/D6 grep checks
  D.3.3 D8 microbench
```

---

## Tasks

### Task D.1.0 — Delete vestigial `Arithmetic` + `Comparison` variants

**Recon to verify before starting:**
- `rg 'FeedbackSiteState::Arithmetic\b' crates/` — confirm no production code reads it (writes via slow-path helpers are part of the dead path).
- `rg 'FeedbackSiteState::Comparison\b' crates/` — same.
- If grep shows readers, plan to handle them (likely just `execution_count` projections — those feed `drain_llint_scalar_feedback`, which now reads `ArithMetadata` directly per Phase C).

**Work:**
1. Remove the `Arithmetic` and `Comparison` variants from `FeedbackSiteState`.
2. Remove `ArithmeticFeedback` and `ComparisonFeedback` structs (file:line ~658).
3. Update any `match` statements that exhaustively match `FeedbackSiteState` to drop the arms.
4. Update `FeedbackVector::site_mut(...)` if it eagerly creates these variants — switch to allocating only for the remaining kinds.
5. Confirm `drain_llint_scalar_feedback` no longer touches these variants (it should already source from `ArithMetadata`).

**Tests:**
- `cargo test --workspace` green. Any test that depended on Arithmetic/Comparison FeedbackSiteState shapes will need updating — likely just snapshot tests; if they're status-API style, they should already use `ArithMetadata`.

**Commit:** `vm/feedback: delete vestigial Arithmetic + Comparison FeedbackSiteState variants`

---

### Task D.1.1 — `PropertyIcState` side-table

**The big one. Estimate 3-5 hours of subagent work.**

**Design:**

```rust
// crates/vm/src/vm/ic_state/property.rs

pub struct PropertyIcState {
    pub cache_state: InlineCacheState,
    pub entry_count: u8,
    pub entries: [Option<NamedPropertyCacheEntry>; POLY_LIMIT],
    pub monomorphic_own_data_handler: NamedPropertyHandler,
    pub monomorphic_proto_data_handler: NamedPropertyProtoHandler,
    pub polymorphic_own_data_handlers: [NamedPropertyHandler; POLY_LIMIT],
    // execution_count + generation live on PropertyMetadata (asm-readable).
}

impl PropertyIcState {
    pub fn new() -> Self { ... }
    pub fn install_monomorphic_own_data(...) -> InstallOutcome { ... }
    pub fn install_proto_data(...) -> InstallOutcome { ... }
    pub fn transition_to_mega(&mut self) { ... }
    pub fn clear(&mut self) { ... }
    // …port all the relevant methods from NamedPropertyFeedback
}
```

**On `Vm`:**

```rust
pub(crate) property_ic_states: HashMap<(CodeRef, FeedbackSlotId), PropertyIcState>,
```

**Migration steps:**

1. Define `PropertyIcState` with the same fields as the Rust-only part of `NamedPropertyFeedback` (everything except `execution_count` and `generation` which already live on `PropertyMetadata`).

2. Port `NamedPropertyFeedback`'s methods to `PropertyIcState`. Read carefully — there are state-transition methods at lines ~1078, ~1090, ~942, etc. Each one needs to:
   - Move its body to `PropertyIcState`.
   - Update the *caller* (in the slow path) to access `vm.property_ic_states.entry((code, slot)).or_insert_with(PropertyIcState::new)` instead of `with_feedback_slot_mut(...)`.

3. **`execution_count` and `generation` already live on PropertyMetadata** — methods that write them should write through `vm.metadata_table_mut(code).property_mut(slot.get())`.

4. The slow path also needs to write the `mode` byte (the InlineCacheState as a packed u8) to `PropertyMetadata.mode` — the asm reads this to decide hit vs miss. Find where `mode` is currently written (`set_named_own_inline_mode` or similar in `feedback_flat.rs`) and reproduce that write directly on `PropertyMetadata`.

5. Replace every `with_feedback_slot_mut(code, slot, |site| { ... site.as_named_property_mut()? })` with direct side-table access.

6. Add GC sweep `Vm::prune_dead_code_property_ic_states(&self_predicate)` mirroring `prune_dead_code_polymorphic_chains`. Wire it into the same call site (`force_collect_with_active_roots` or whatever the Phase B Polymorphic sweep used).

**Tests:**
- All existing Property IC tests in `tests/inline_caches.rs` must continue to pass. They typically install, run, and inspect via `vm.feedback_vector_snapshot()` — those snapshot APIs are stubbed in D.2.5. For the duration of D.1.1, the snapshot can still read from `FeedbackVector` (untouched until D.2.4).
- Add D1: Uninit→Mono via single install on `PropertyIcState`.
- Add D2: Mono→Poly via second install with distinct shape.
- Add D3: Poly→Mega via 9th install (POLY_LIMIT inline + chain = 8 entries; the 9th transitions Mega).
- Add D4: re-install on Mono after `AdaptiveProtoLoad` clear → re-caches.

**Commit:** `vm/ic_state: rehome NamedProperty IC state machine onto PropertyIcState`

---

### Task D.1.2 — `CallIcState` + `ConstructIcState`

Same shape as D.1.1 but for Call/Construct kinds. These are simpler (mono→mega only, no polymorphic chain).

**Design:**

```rust
pub struct CallIcState {
    pub cache_state: InlineCacheState,
    pub mode: u8,
    pub callee_bits: u64,
    // execution_count + generation on CallMetadata.
}
```

**Same migration steps as D.1.1.** Port methods from `CallFeedback` / `ConstructFeedback`. Add HashMap fields on Vm. Wire GC sweep.

**Tests:** Existing call IC tests stay green. Add D-style state-transition tests (not in spec §6.4 but useful).

**Commit:** `vm/ic_state: rehome Call+Construct IC state machines`

---

### Task D.1.3 — `KeyedPropertyIcState`

Same shape as D.1.2 (mono→mega, no polymorphic chain) plus dense-index handling. Port `KeyedPropertyFeedback`.

**Commit:** `vm/ic_state: rehome KeyedProperty IC state machine`

---

### Task D.2.1 — Delete `debug_assert_metadata_matches_flat`

The equivalence assertion is no longer load-bearing once the state machine has migrated: `mirror_flat_slot` writes `FeedbackEntry` from `FeedbackSiteState`, but if the slow path no longer mutates `FeedbackSiteState`, there's nothing to assert against.

**Work:**
1. Remove the `debug_assert_metadata_matches_flat` method.
2. Remove its invocation from `mirror_metadata_slot`.

**Tests:** workspace green.

**Commit:** `vm/feedback: delete debug equivalence assertion`

---

### Task D.2.2 — Delete `mirror_flat_slot` + `mirror_metadata_slot`

By this point both mirrors write to storage no one reads (`PropertyIcState` is the source of truth, asm reads from `MetadataTable` directly via the slow path's writes).

**Work:**
1. Verify the slow path now writes directly to `PropertyMetadata`/`CallMetadata`/etc. for asm-readable fields (mode, generation, handler_bits, aux_bits, execution_count). This happened in D.1.1-D.1.3.
2. Delete every `self.mirror_flat_slot(code, slot)` callsite.
3. Delete every `self.mirror_metadata_slot(code, slot)` callsite.
4. Delete the `mirror_flat_slot` and `mirror_metadata_slot` function definitions.
5. Delete `with_feedback_slot_mut` (it was the wrapper that called both mirrors).

**Tests:** workspace green.

**Commit:** `vm/feedback: delete mirror_flat_slot + mirror_metadata_slot helpers`

---

### Task D.2.3 — Delete `feedback_flat_storage` + `FeedbackEntry` + `dsl/feedback_flat.rs`

**Work:**
1. Delete `Vm::feedback_flat_storage` field and its allocation in `install.rs`.
2. Delete `Vm::frame_fv_base` population sites (was already documented as legacy after Task 4.4).
3. Delete `LlIntState::frame_fv_base` field.
4. Delete `LLINT_STATE_FRAME_FV_BASE` constant.
5. Delete `crates/vm/src/dsl/feedback_flat.rs` module entirely.
6. Delete the `pub mod feedback_flat;` declaration from `crates/vm/src/dsl/mod.rs` (or wherever it's declared).
7. Update any imports.

**Tests:** workspace green. `rg 'feedback_flat\|FeedbackEntry\|frame_fv_base'` returns matches only inside Phase D's deletion commit.

**Commit:** `vm/dsl: delete feedback_flat module + frame_fv_base pin field`

---

### Task D.2.4 — Delete `FeedbackVector` + `FeedbackSiteState` + remaining payloads

**Work:**
1. Delete `FeedbackVector`, `FeedbackSiteState`, `NamedPropertyFeedback`, `CallFeedback`, `ConstructFeedback`, `KeyedPropertyFeedback` (and any remaining `*Feedback` types whose state has migrated).
2. Delete `Vm::feedback_vectors` field.
3. Delete the corresponding allocation in `install.rs`.
4. Delete `with_feedback_slot` (the immutable variant).

**Tests:** workspace green. `rg 'FeedbackVector\|FeedbackSiteState\|NamedPropertyFeedback'` returns no matches except in deleted commit diff.

**Commit:** `vm/feedback: delete FeedbackVector + FeedbackSiteState + per-kind payloads`

---

### Task D.2.5 — Stub `FeedbackVectorSnapshot` for Phase E

Per spec §6.3: tests/profiler callers of `FeedbackVectorSnapshot`/`FeedbackVectorFootprint` need a stub so the workspace compiles.

**Approach:**

```rust
// Empty stub — Phase E replaces with proper per-kind status API.
pub struct FeedbackVectorSnapshot;
pub struct FeedbackVectorFootprint;

impl FeedbackVectorSnapshot {
    pub fn sites(&self) -> &[FeedbackSiteSnapshot] { &[] }
    // …other no-op methods that existing tests call
}
```

Where existing tests use `snapshot.sites[3].state == Polymorphic`, either:
- Update the test to read from `vm.property_ic_states.get(&(code, slot)).cache_state` directly.
- Or mark the test `#[ignore]` with `// TODO(Phase E): port to status API`.

Spec §6.3 explicitly allows the temporary stub + `#[ignore]` pattern.

**Tests:** every `#[ignore]`'d test has a TODO comment referencing Phase E. `cargo test --workspace` green.

**Commit:** `vm/feedback: stub FeedbackVectorSnapshot for Phase E follow-up`

---

### Task D.3.1 — D1-D4 state-machine tests

These were sketched in D.1.1 but verified holistically here. Add to `tests/inline_caches.rs`:

- D1: install a Property handler at a single shape → `PropertyIcState.cache_state == Monomorphic`.
- D2: install at a second distinct shape → `Polymorphic`, `entry_count == 2`.
- D3: install at the 9th distinct shape (inline 2 + chain 6 + one more) → `Megamorphic`, chain removed.
- D4: install Mono → fire `AdaptiveProtoLoad` to clear → install again → `Monomorphic` (returns to Uninit then Mono).

**Commit:** `vm/tests: D1-D4 state-machine transitions on PropertyIcState`

---

### Task D.3.2 — D5/D6 grep checks

These aren't tests per se; they're verification commands. Add them to the plan's verification section (no code change needed — just run and confirm).

```bash
rg 'feedback_flat_storage' crates/      # expect: zero matches
rg 'FeedbackEntry' crates/                # expect: zero matches
rg 'mirror_flat_slot' crates/            # expect: zero matches
rg 'FeedbackVector' crates/               # expect: only matches inside FeedbackVectorSnapshot stub (Phase E)
rg 'NamedPropertyFeedback' crates/        # expect: zero matches
rg 'mirror_metadata_slot' crates/         # expect: zero matches
rg 'debug_assert_metadata_matches_flat' crates/  # expect: zero matches
```

If any of these return unexpected matches, the deletion is incomplete.

---

### Task D.3.3 — V8 microbench

Run `cargo run --release -p lyng-bench --quiet -- v8suite --samples 5` and compare to Phase C end baseline (committed at `940d407a` / `ca387992`):
- Pre-Phase-D baseline (= Phase C end): Richards 408, DeltaBlue 375, Crypto 328, RayTrace 220, NavierStokes 436, Splay 1314.
- Expected: substantial recovery toward the pre-Spec-2 baseline (484/421/393/291/541/1440) because mirror_flat_slot + mirror_metadata_slot deletion eliminates the per-IC-mutation dual-write overhead.
- Spec §6.4 D8 budget: ≤1% delta vs pre-Phase-D. Phase D should recover, not regress.

**Commit:** update `reports/lyng/bench-v8.md` with end-of-Phase-D numbers + delta vs both Phase C end and pre-Spec-2.

---

## Verification (end-to-end)

After D.3.3:

1. `cargo test --workspace` green (~2672+ tests; Phase E port may unignore some).
2. `cargo test -p lyng-vm --test inline_caches` green (~44+ IC tests).
3. `cargo clippy --workspace --all-targets -- -D warnings` clean.
4. `cargo fmt --check` clean.
5. Grep checks from D.3.2 return zero matches for legacy types.
6. V8 bench shows recovery vs Phase C end; Phase D end ≤ pre-Spec-2 baseline.

---

## Risk and rollback

The riskiest task is D.1.1 (NamedProperty state machine rehome). If state transitions break, IC tests will fail visibly. If they regress to slow path silently (wrong cache decisions), benchmarks will show regression.

**Mitigation:** D.1.1 keeps `FeedbackSiteState::NamedProperty` ALIVE in parallel until D.2.4 deletes it. During the D.1.1 → D.2.3 window, the slow path writes to BOTH `FeedbackSiteState::NamedProperty` AND `PropertyIcState`. The debug equivalence assertion (alive until D.2.1) can be extended to also compare `PropertyIcState` against `NamedPropertyFeedback` if regressions appear.

If Phase D shows >3% bench regression at the end, revisit the side-table HashMap choice — may need to switch to inline-on-PropertyMetadata or per-code Vec.

---

## Out of scope (Phase E)

- `FeedbackVectorSnapshot` real implementation (status API).
- `NamedPropertyStatus`, `CallStatus`, `ArithStatus`, `ComparisonStatus`, `KeyedPropertyStatus` types.
- `MetadataTableFootprint`.
- Updating the ~12 test consumers that the D.2.5 stub temporarily covers.
- `recording_watchpoint_fires` cleanup.
