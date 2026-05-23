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
- Added VM architecture tests that reject legacy fast-bridge terminology and
  pin the known Rust-probe bridge count.

## Remaining Rust Probes From LLInt

These are still not LLInt fast paths. They call Rust on the hit probe and then
dispatch from the returned payload.

| Opcode | LLInt bridge | Rust entry point | Required replacement |
| --- | --- | --- | --- |
| `LoadGlobal` | `call_rust_probe!(op_load_global_rust_probe_rs, ...)` | `Vm::try_load_global_rust_probe_for_dsl` | LLInt-readable global/property IC metadata and inline load-global checks |
| `GetNamedProperty` | `call_rust_probe!(op_get_named_property_rust_probe_rs, ...)` | `Vm::try_get_named_property_rust_probe_for_dsl` | LLInt mode-byte property IC probe and inline value load |
| `AssignNamedProperty` | `call_rust_probe!(op_assign_named_property_rust_probe_rs, ...)` | `Vm::try_assign_named_property_rust_probe_for_dsl` | LLInt mode-byte property IC probe and inline store |

## Hybrid SMI Paths That Still Call Rust

These handlers compute the SMI result in DSL asm, but the hit side still calls
Rust to record feedback. They should be treated as hybrid inline hit paths until
the feedback write is moved into the DSL or the feedback design changes.

- `op_add` in `crates/vm/src/dsl/handlers/hot.rs`
- `op_sub`
- `op_mul`
- `op_bit_and`
- `op_shift_left`
- `op_shift_right`
- `op_increment`
- `op_decrement`

The blocker is the feedback layout. The old inline `record_smi!` route cannot
be used while the `entry_observed` offset binding is still a placeholder;
writing at offset `0` would corrupt the legacy `Option<FeedbackSiteState>`
discriminant. This needs a real LLInt-readable feedback header or a deliberate
policy that the LLInt hit path does not record this feedback.

## Rust IC Cache Terminology To Clean Up

The Rust dispatch layer still uses "fast" names for inline-cache sidecars and
helper lookups. These are not LLInt fast paths:

- `NamedPropertyFeedback::monomorphic_fast`
- `NamedPropertyFeedback::monomorphic_proto_fast`
- `NamedPropertyFeedback::polymorphic_fast`
- `KeyedPropertyFeedback::monomorphic_named_proto_fast`
- `*_fast_handler`
- `try_*_fast_load` / `try_*_fast_store`
- `record_named_property_fast_hit`

These names came from the older Rust-dispatch IC work. They should either be
renamed to cache/sidecar terminology (`*_handler_cache`, `record_*_ic_hit`,
etc.) or retired as each IC family is ported to true LLInt mode-byte dispatch.

## Current Richards Evidence

The current counted Richards run after the equality changes shows:

| Opcode | Dispatches | Semantic slow-path hits | Semantic share |
| --- | ---: | ---: | ---: |
| `StrictEqual` | 31,408,973 | 0 | 0.00% |
| `Equal` | 69,702,579 | 36,206,028 | 51.94% |

`StrictEqual` is now clean on this workload. `Equal` is not: the SMI path is
inline, but non-SMI equality still falls through to Rust.

The release score for this build is 294 on Richards
(`cargo run --release -p lyng-bench -- v8suite --filter Richards --timeout-secs 120`).

## Suggested Next Steps

1. Land the terminology/test correction separately so Rust probes cannot be
   presented as LLInt fast paths again.
2. Add a real LLInt-readable feedback/IC header before porting the property IC
   bridges. The current Rust enum-backed feedback state is the main reason the
   LLInt handlers still need Rust probes.
3. Port `GetNamedProperty`, `LoadGlobal`, and `AssignNamedProperty` from Rust
   probes to DSL asm in that order.
4. Replace or remove the arithmetic `*_record_smi_rs` shims so the SMI hit
   paths no longer call Rust.
5. Rename the remaining Rust IC cache terminology once the mode-byte path is in
   place, or explicitly mark it as Rust-dispatch cache terminology while it
   exists.
