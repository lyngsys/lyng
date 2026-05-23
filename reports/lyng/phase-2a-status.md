# Phase 2a — Storage & accessor infrastructure: status report

**Issue:** `lyng-3foz` — Phase 2: Cell access flattening, sub-phase 2a
**Parent epic:** `lyng-49qk` — JSC-aligned engine roadmap
**Date:** 2026-05-15
**Toolchain:** rustc 1.93 (2025-12-15), aarch64-apple-darwin, `--release` profile
**Branch:** `claude/hardcore-yalow-020120`

## What landed

Two changes in `crates/lyng/gc/`, scoped to durable infrastructure that
Phase 3 (`lyng-2pgt`) will consume:

1. **`Vec<SlotPage<Record>>` → `Vec<Box<SlotPage<Record>>>`** in
   `crates/lyng/gc/src/arena/storage.rs:176`. Each `SlotPage` is ~5KB; a
   `Vec::push` that grew the slab previously moved all pages inline. Boxing
   makes page bodies pointer-stable, which is the precondition for handing out
   borrow-based record accessors.
2. **New `object_ref(id) -> Option<&RuntimeObjectRecord>`** accessor on
   `PrimitiveHeap` (`arena.rs:605`) and `PrimitiveHeapView`
   (`mutator.rs:198`), backed by `SlotArena::get_ref` /
   `SlotPage::get_ref`. The existing by-value `object(id) ->
   Option<RuntimeObjectRecord>` accessor is kept; callers that depend on
   `Copy` are unchanged.

No consumers adopt the new accessor in this PR. The IC chain, the
`property_cache.rs` flow, the dispatch handlers, and `RuntimeObjectRecord`'s
layout are all untouched.

The change is generic across all 14 `SlotArena` domains (objects, strings,
symbols, bigints, function payloads, value cells, suspended executions,
environments, codes, realms, shapes). Box deref coercion handles every
existing call site without code changes elsewhere. Handle encoding
(`make_handle::<Handle>(page_index, slot_index)` / `locate::<Handle>`),
card-table math (`page_index * PRIMITIVE_SLOTS_PER_PAGE + slot_index`), and
per-page free-list reuse are preserved verbatim.

## What's deferred to Phase 3 and why

The roadmap's stated 2a acceptance criterion — *"`cargo asm` on
`op_get_named_property` shows one load from cell before the shape compare,
not two"* — is **not** satisfied by this PR. The two loads come from an
algorithmic redundancy in the IC chain: `feedback::try_load` reads the
receiver record once to look up the cached shape, and
`validated_named_property_cache_holder` reads it again to verify the cache
dependency. Both lookups go through `PrimitiveHeap::object(id)`.

Phase 3 (`lyng-2pgt`) plans to *"collapse the 4-deep function call chain in
IC dispatch into a flat block inside each IC-shaped opcode handler"* and to
*"drop the redundant second shape compare."* The same code Phase 2a would
have refactored to dedup the receiver lookup will be rewritten in Phase 3
as part of handler inlining. Doing the dedup here would be throwaway work.

Phase 2a as planned and landed therefore covers **only the durable
infrastructure** — pointer-stable pages plus the borrow-based accessor —
which Phase 3 will adopt when it inlines `op_get_named_property` and the
nine other IC-shaped property opcodes.

Out of scope (consistent with the plan):

- The algorithmic IC chain collapse and the *one load before shape compare*
  asm criterion → **Phase 3** (`lyng-2pgt`).
- Phase 2b (pointer-identity cells, `*mut ObjectHeader` packed into
  NaN-boxed `Value`) → gated on profile evidence after Phase 3.
- Flattening to a single contiguous `Box<[Record]>` (`base + id * stride`)
  → reserved for Phase 2b.
- Shrinking `RuntimeObjectRecord` (e.g. moving `last_invalidation_epoch`)
  → verified: epoch is read inline on every monomorphic validation; moving
  it adds an indirection that costs more than the 8 bytes saved.

## Verification

### Tests

| Check | Pre-change | Post-change | Δ |
| --- | ---: | ---: | --- |
| `cargo test -p lyng-gc -p lyng-objects -p lyng-vm -p lyng-tests` | 1701 passed, 1 ignored | 1701 passed, 1 ignored | identical |
| `cargo clippy --workspace --all-targets` | 0 errors, 62 pre-existing warnings | 0 errors, 62 pre-existing warnings | no new warnings |

### V8 v7 sweep (11 samples per benchmark, isolated subprocesses)

| Benchmark | Pre-change median | Post-change median | Δ |
| --- | ---: | ---: | ---: |
| Richards | 245 | 244 | −0.4% |
| DeltaBlue | 287 | 283 | −1.4% |
| Crypto | 267 | 272 | +1.9% |
| RayTrace | 400 | 401 | +0.3% |
| NavierStokes | 452 | 455 | +0.7% |
| Splay | 1247 | 1270 | +1.8% |
| **Geomean** | **~401** | **~404** | **+0.7%** |

Verdict: **within noise floor**, consistent with the plan's prediction
("neutral to slightly positive; we are not adopting the new accessor in
any hot path yet"). No workload regresses > 2%. Splay's +1.8% is the most
suggestive of a real signal — Splay is property-heavy and may already
benefit from the slightly tighter slab allocator behavior post-`Box`
(less reallocation churn during nursery growth).

Reports:
- `reports/lyng/phase-2a-baseline-bench.md`
- `reports/lyng/phase-2a-bench.md`

### `cargo asm` on `op_get_named_property`

Pre- and post-change asm captures are **byte-identical** after stripping
the cargo build preamble:

```sh
$ diff reports/lyng/phase-2a-baseline-op_get_named_property.asm \
       reports/lyng/phase-2a-final-op_get_named_property.asm
$ echo $?
0
```

Same 438 lines, same 9233 bytes, same `bl`-target set:
`try_named_property_load_inline_cache_hit`, `observe_named_property_slow_path`,
`get_property_from_value`, `transfer_to_exception_handler`, plus
`decode_abc_operands_wide` and the standard panic/bounds shims.

This is the **expected** result for an infrastructure-only PR. Phase 3
will adopt `object_ref` inside the inlined handler bodies and diff against
this baseline to demonstrate the load-count reduction.

### Test262

| | Passed | Runnable | Rate |
| --- | ---: | ---: | ---: |
| Baseline (`8aaed590`, `reports/lyng/test262.md`) | 49722 | 49729 | 93.72% |
| Post-change (`reports/lyng/phase-2a-test262.md`) | 49722 | 49729 | 93.72% |

Identical pass count, identical runnable count, identical pass rate.
**No regression.** ✓

## Foundation for Phase 3

Phase 3's planned target shape for `op_get_named_property` (per the roadmap
`Phase 3 — Inline IC Fast Path` section) inlines the IC chain into the
handler:

```rust
extern "C" fn op_get_named_property(state: &mut DispatchState) -> DispatchOutcome {
    let (target_reg, recv_reg, atom_const, slot) = decode_with_slot(state);
    let receiver = state.read_register(recv_reg);
    if let Some(cell) = receiver.as_cell() {
        let cell_shape = cell.shape_id();              // 1 load
        ...
    }
}
```

`object_ref(receiver)` is the safe-Rust analogue of `receiver.as_cell()` in
that sketch: a borrow that resolves to a single addressable
`RuntimeObjectRecord` without an 80-byte by-value copy. The pointer-stable
pages guarantee the borrow is sound across any `SlotArena` growth that may
happen during the surrounding handler body (no growth occurs on the IC hit
path; the guarantee matters for slow-path fallback that calls back into
the mutator).

Phase 3 can begin without any further `gc` crate work.

## Files changed

- `crates/lyng/gc/src/arena/storage.rs` — `Vec<Box<SlotPage>>`, +
  `SlotArena::get_ref`, + `SlotPage::get_ref`.
- `crates/lyng/gc/src/arena.rs` — + `PrimitiveHeap::object_ref`.
- `crates/lyng/gc/src/mutator.rs` — + `PrimitiveHeapView::object_ref`.

Total diff: 18 added lines, 2 modified lines across 3 files.
