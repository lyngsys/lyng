# Phase 3b — Store-side NamedProperty inline IC fast path: status report

**Issue:** `lyng-im37` — Phase 3b: Inline IC fast path for Set/Assign/StrictAssign NamedProperty
**Parent:** `lyng-2pgt` — Phase 3: Inline IC fast path (epic)
**Date:** 2026-05-15
**Toolchain:** rustc 1.93 (2025-12-15), aarch64-apple-darwin, `--release` profile
**Parent commit:** `c70911db` (Phase 3a 2/2)

## What landed

The store-side IC dispatch chain on the monomorphic OwnData hit path is
inlined for `SetNamedProperty`, `AssignNamedProperty`, and
`StrictAssignNamedProperty` (all three share
`Vm::execute_set_named_property_opcode`). Hit path now reads:

1. Packed handler word from `NamedPropertyFeedback.monomorphic_fast` (Phase 3a sidecar, reused).
2. Receiver record via `heap.object_ref(receiver)` (Phase 2a infra).
3. Shape compare against the packed handler's high half.
4. Invalidation-epoch compare against `monomorphic_fast_dependency_epoch` (Phase 3a sidecar, reused).
5. Writable check (new bit on `NamedPropertyHandler`).
6. Slot offset decode → `PrimitiveMutator::mut_store_value(target, value)` for the barrier-aware write.

No `bl` to `try_named_property_store_inline_cache`, `try_store`,
`store_to_named_property_cache`, or `validated_named_property_cache_holder`
on the hit path. Two `bl`s remain — `PrimitiveMutator::store_value` (the
actual barrier-aware heap write, unavoidable) and `record_feedback_slot`
(tier bookkeeping the slow chain also performs after IC store).

Polymorphic, PrototypeData, megamorphic, miss, proxy, and out-of-line
corrupt-state cases continue through the existing
`try_named_property_store_inline_cache → ordinary_set / set_property_on_value`
slow chain unchanged.

### Packed-handler representation change

`NamedPropertyHandler` grew a writable bit. Old layout:

```text
bits  0..31  slot_offset (SlotLocation::encode — bit 31 = inline tag, bits 0..30 = offset)
bits 32..64  receiver shape (NonZeroU32; whole word == 0 ⇒ NONE)
```

New layout:

```text
bit   31     inline_slot flag (unchanged)
bit   30     writable flag (NEW; 1 = writable, 0 = read-only)
bits  0..30  slot_offset (now 30 bits — 1B values, debug-assert at pack time)
bits 32..64  receiver shape (unchanged)
```

Slot offsets are tiny in practice (inline: 0..3 per `INLINE_NAMED_SLOT_COUNT`;
out-of-line: rarely above 2^16). 30 bits is vastly sufficient.
`NamedPropertyHandler::from_entry` returns `Self::NONE` if a slot offset
ever exceeds the 30-bit field (defensive — never seen). Loads ignore
the writable bit (semantics unchanged from Phase 3a); stores
short-circuit non-writable entries to `stored = false` without a heap
write — matching `store_to_named_property_cache → Ok(Some(false))`
semantics for read-only own-data properties (silent no-op in sloppy
mode, TypeError via `check_property_assignment_result` in strict /
explicit-assignment mode).

## Verification

### Tests

| Check | Phase 3a | Phase 3b | Δ |
|---|---:|---:|---|
| `cargo test -p lyng-js-gc -p lyng-js-objects -p lyng-js-vm -p lyng-js-tests` | 1707 passed | 1709 passed | +2 (new writable-bit packing tests) |
| `cargo clippy --workspace --all-targets` | 0 errors, 62 warnings | 0 errors, 64 warnings | +2 (both in `crates/html_parser`, unrelated to Phase 3b code) |

### V8 v7 sweep (11 samples per benchmark, isolated subprocesses)

| Benchmark | Phase 2a | Phase 3a | Phase 3b | Δ vs Phase 3a | Δ vs Phase 2a |
|---|---:|---:|---:|---:|---:|
| Richards | 244 | 282 | 290 | **+2.8%** | **+18.9%** |
| DeltaBlue | 283 | 310 | 312 | +0.6% | +10.2% |
| Crypto | 272 | 277 | 275 | −0.7% | +1.1% |
| RayTrace | 401 | 416 | 420 | +1.0% | +4.7% |
| NavierStokes | 455 | 458 | 457 | −0.2% | +0.4% |
| Splay | 1270 | 1266 | 1286 | **+1.6%** | +1.3% |
| **Geomean** | — | — | — | **≈ +0.8%** | **≈ +5.8%** |

Smaller incremental gain than Phase 3a (3a was +4.97% over Phase 2a;
3b adds another ~+0.8%) — expected, since `GetNamedProperty` is much
hotter than the store variants. The biggest cumulative gains over
Phase 2a remain on Richards (+18.9%) and DeltaBlue (+10.2%), the two
property-mutation-heavy benchmarks. Splay (tree-node mutations)
benefits in 3b where it was flat in 3a. No benchmark regresses >1%.

Reports:
- `reports/js/lyng-js/phase-3b-bench.md`
- `reports/js/lyng-js/phase-3b-bench.json`

### `cargo asm` on `op_set_named_property_common`

`op_set_named_property` (and the two Assign variants) are thin trampolines
that all call `op_set_named_property_common`, which inlines
`execute_set_named_property_opcode`. The asm artifact captures the common
function.

| Variant | Hit-path `bl` targets (excluding panics/decoders/exceptions) |
|---|---|
| Phase 3a (slow chain on store hit) | `try_named_property_store_inline_cache` → `record_feedback_slot` |
| Phase 3b (inlined fast path on store hit) | `PrimitiveMutator::store_value` → `record_feedback_slot` |

The IC chain helper is gone from the hit path. The remaining
`store_value` is the actual heap write with GC write barrier — that's
work that has to happen, not IC dispatch. `record_feedback_slot` is
tier bookkeeping unchanged from the slow chain.

Slow-path `bl` targets remain: `try_named_property_store_inline_cache`
(polymorphic / PrototypeData / megamorphic), `set_property_on_value` +
`ordinary_set` (proxy / OOL fallback), `observe_named_property_slow_path`
(cache update on miss).

Report:
- `reports/js/lyng-js/phase-3b-op_set_named_property_common.asm`

### Test262

| | Files runnable | Files passed | Pass rate | Δ vs Phase 2a |
|---|---:|---:|---:|---:|
| Phase 2a baseline | 49729 | 49722 | 99.99% | — |
| Phase 3a | 49729 | 49720 | 99.98% | −2 (1 latent regression + 1 timeout flake) |
| **Phase 3b** | **49729** | **49721** | **99.98%** | **−1 (latent regression only; timeout flake did not reproduce)** |

The Phase 3a `RegExp/unicode-class-braced.js [non-strict]` timeout
flake (was 0.986s in Phase 2a, timed out at 1.0s in the 3a measurement)
did not reproduce in the 3b run — consistent with the flake hypothesis.

The deterministic `staging/sm/expressions/object-literal-__proto__.js`
regression introduced by Phase 3a (1/2)'s `NamedPropertyFeedback`
layout change continues to reproduce in 3b. Tracked as `lyng-2tr1`;
unrelated to the 3b code path (3b doesn't change the layout further).

Report:
- `reports/js/lyng-js/phase-3b-test262.md`

## What's deferred

- **`lyng-2tr1`**: the carry-over `object-literal-__proto__` Test262 regression — not addressed by 3b, separate investigation.
- **`lyng-5j2z`**: global opcode IC fast path (Phase 3c).
- **`lyng-guem`**: keyed property opcode IC fast path (Phase 3d).
- **`lyng-22al` / `lyng-5nju` / `lyng-28t2`**: PrototypeData / polymorphic compaction / γ-swap (Phase 3e/f/g).

## Files changed

- `crates/lyng-js/objects/src/shapes.rs` — `NamedPropertyHandler` writable bit (bit 30 of low half); `from_entry` extracts `entry.attrs().writable()`; new `writable()` accessor; `slot_location()` masks bit 30 off the offset.
- `crates/lyng-js/objects/src/tests.rs` — `synthesize_own_data_entry` takes a `writable` parameter; 2 new tests (`packs_writable_bit_for_read_only_entry`, `writable_bit_does_not_alias_slot_offset`); 6 existing tests updated to pass `writable: true`.
- `crates/lyng-js/vm/src/vm/dispatch/property.rs` — `ValueStoreTarget` import added; cache-hit branch of `execute_set_named_property_opcode` rewritten as the inlined fast path; slow chain (lines below the new block) unchanged.

No changes in `crates/lyng-js/vm/src/vm/feedback.rs` — the
`monomorphic_fast`, `monomorphic_fast_dependency_epoch`,
`named_property_fast_handler`, and `refresh_monomorphic_fast`
infrastructure from Phase 3a is reused unchanged.

Total Phase 3b diff: ~100 added lines + ~20 modified across 3 files.
