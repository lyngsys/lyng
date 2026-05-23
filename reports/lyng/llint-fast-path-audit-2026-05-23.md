# LLInt Fast-Path Audit — 2026-05-23

## Definition

For Lyng, "fast path" is reserved for a hot hit path that stays inside the
LLInt handler emitted from the asm DSL. If the hit path calls Rust before
dispatching, it is a Rust probe, Rust bridge, or Rust feedback shim, not an
LLInt fast path.

## Corrections Landed In This Slice

- Renamed the LLInt Rust bridge macro from `call_fast!` to
  `call_rust_probe!`.
- Renamed the remaining DSL-side Rust IC bridge helpers from `*_fast_rs` /
  `*_fast_for_dsl` to `*_rust_probe_rs` / `*_rust_probe_for_dsl`.
- Removed the Rust bridge helpers for `Equal` and `StrictEqual`.
  - `StrictEqual` now handles raw-equal primitives, `NaN`, and most
    raw-unequal primitive false cases inside DSL asm.
  - `Equal` now handles SMI-vs-SMI equality inside DSL asm.
- Removed the `GetNamedProperty` Rust probe bridge for the monomorphic
  OwnData inline-slot case.
  - The LLInt handler now reads the flat IC header, validates receiver
    shape and invalidation epoch, loads the object record through the
    asm-visible object-record table, and reads the inline named slot in
    DSL asm.
  - Polymorphic, prototype, out-of-line, non-object, invalidated, and
    uncached cases intentionally fall back to the counted semantic slow path.
- Added VM architecture tests that reject legacy fast-bridge terminology and
  pin the known Rust-probe bridge count.
- Renamed the Rust-dispatch IC/cache helpers and direct object probes so they
  no longer claim LLInt fast-path status:
  - `monomorphic_fast` / `polymorphic_fast` sidecars are now named
    `*_own_data_handler`, `*_proto_data_handler`, or
    `*_own_data_handlers`.
  - Rust IC helper entry points are now named as cache/direct paths, for
    example `named_property_own_data_handler`,
    `try_named_property_polymorphic_own_data_load`, and
    `record_named_property_cache_hit`.
  - Object/builtin shortcuts are now named as direct/specialized probes, for
    example `try_direct_get_named_data_property`,
    `direct_set_engine_array_index`, and `try_specialized_apply_builtin`.
  - The RegExp recognized-pattern shortcuts no longer use fast-path names.
- Added a VM architecture test that scans the Rust VM hot-path files and fails
  if they reintroduce `fast path`, `fast_`, or `_fast` terminology outside the
  DSL/LLInt code.
- Updated the live architecture doc so it reserves fast-path terminology for
  LLInt asm and describes Rust hit paths as probes, shortcuts, cache hits, or
  direct/specialized paths.
- Cleaned tool/test wording for Rust helper shortcuts and hybrid SMI
  arithmetic hit paths so they no longer use fast-path labels.

## Remaining Rust Probes From LLInt

These are still not LLInt fast paths. They call Rust on the hit probe and then
dispatch from the returned payload.

| Opcode | LLInt bridge | Rust entry point | Required replacement |
| --- | --- | --- | --- |
| `LoadGlobal` | `call_rust_probe!(op_load_global_rust_probe_rs, ...)` | `Vm::try_load_global_rust_probe_for_dsl` | LLInt-readable global/property IC metadata and inline load-global checks |
| `AssignNamedProperty` | `call_rust_probe!(op_assign_named_property_rust_probe_rs, ...)` | `Vm::try_assign_named_property_rust_probe_for_dsl` | LLInt mode-byte property IC probe and inline store |

## Hybrid SMI Paths Removed

These handlers previously computed the SMI result in DSL asm, but the hit side
still called Rust to record feedback. That made them hybrid inline hit paths,
not true LLInt fast paths. The live implementation now records pending SMI
feedback through the asm-visible flat feedback sidecar with `record_smi!`, then
drains that sidecar at explicit VM run boundaries.

- `op_add` in `crates/vm/src/dsl/handlers/hot.rs`
- `op_sub`
- `op_mul`
- `op_bit_and`
- `op_shift_left`
- `op_shift_right`
- `op_increment`
- `op_decrement`

The fix was to add scalar feedback words to the LLInt-readable `FeedbackEntry`
header: `scalar_observed_bits` and `scalar_execution_count`. The legacy
`Option<FeedbackSiteState>` remains a Rust-side semantic mirror; LLInt no longer
writes into that enum layout on hit-side scalar arithmetic.

## Terminology Sweep

Source audit command:

```sh
rg -n "fast path|fast-path|Fast path|Fast-path|proto-fast|fast_|_fast|\\bfast\\b|Fast" crates tools --glob '*.rs' --glob '!target'
```

Current result: the remaining source hits are confined to:

- `crates/vm/src/dsl/**`, where the term describes real DSL-emitted LLInt
  hit paths or DSL helper comments.
- DSL-specific tests that assert the behavior of those handlers.
- `crates/vm/src/tests/llint_architecture.rs`, which contains the guard
  strings for the terminology test itself.
- Test262 helper names such as `checkToTemporalPlainDateTimeFastPath`, which
  are external harness API names rather than Lyng path labels.
- Generic non-path wording such as Cargo's `--no-fail-fast`.

The Rust VM dispatch layer, feedback layer, object direct probes, builtin
specializations, and RegExp recognized-pattern shortcuts no longer use fast-path
terminology.

## Current Richards Evidence

The current counted Richards run after removing the scalar feedback hybrids
shows:

| Opcode | Dispatches | Semantic slow-path hits | Semantic share |
| --- | ---: | ---: | ---: |
| `GetNamedProperty` | 38,759,508 | 25,486,423 | 65.76% |
| `Equal` | 12,289,161 | 6,383,433 | 51.94% |
| `AssignNamedProperty` | 9,731,961 | 18,913 | 0.19% |
| `LoadGlobal` | 7,932,735 | 131 | 0.00% |
| `StrictEqual` | 5,537,671 | 0 | 0.00% |
| `BitAnd` | 2,240,640 | 0 | 0.00% |
| `Increment` | 983,619 | 0 | 0.00% |
| `Decrement` | 192,000 | 0 | 0.00% |
| `ShiftRight` | 191,808 | 0 | 0.00% |
| `AddSmi` | 178,752 | 178,752 | 100.00% |

`GetNamedProperty` is now a real LLInt hit path for monomorphic OwnData inline
slots, but Richards still has a large semantic slow share because prototype,
polymorphic, out-of-line, non-object, and cold/miss cases are intentionally
still semantic slow-path entries. `StrictEqual` is clean on this workload.
`Equal` is not: the SMI path is inline, but non-SMI equality still falls
through to Rust. The hot former scalar feedback-bridge handlers visible in this
run (`BitAnd`, `Increment`, `Decrement`, `ShiftRight`) now show zero semantic
slow-path entries. `AddSmi` is a separate missing LLInt opcode, not a feedback
hybrid: it is still a cold stub and should be tracked as missing opcode
coverage.

The current release samples for this tree are 287, 286, and 286 on Richards:

```sh
cargo run --release -p lyng-bench -- v8suite --filter Richards --timeout-secs 120
```

The `baseline=234` and `target=260` numbers are the local bench harness gate
values printed by `lyng-bench`; they are not QJS parity targets.

## Suggested Next Steps

1. Port `AddSmi` to a real LLInt handler. It is the remaining high-confidence
   scalar cold stub in Richards (`178,752` dispatches, all semantic slow).
2. Extend the LLInt-readable feedback/IC header to cover store and global-load
   modes. The Rust enum-backed feedback state should remain the semantic source
   of truth, but the LLInt needs compact mode-specific header words.
3. Port `LoadGlobal` and `AssignNamedProperty` from Rust probes to DSL asm in
   that order.
4. Continue the same audit discipline for any remaining claimed fast path:
   the hit side must dispatch from DSL asm without entering Rust. The known
   remaining Rust-probe cleanup candidates are still `LoadGlobal` and
   `AssignNamedProperty`.
