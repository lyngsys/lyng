# Lyng JS Native Backend Research

This note records the backend recommendation for a future Lyng JS JIT as of 2026-05-03. It does
not add or approve any native backend dependency.

## Context

Lyng JS is interpreter-first, with immutable bytecode templates, compiler-owned feedback-site
metadata, stable `CodeRef` handles, shared feedback vectors, and explicit safepoint categories.
The architecture already requires later native tiers to respect bytecode identity, frame layout,
GC/root maps, bailout metadata, host hooks, and crate ownership boundaries.

The immediate goal is therefore not "pick a compiler and start emitting code". The first goal is
to preserve the current interpreter quality while making native execution a small, reversible
extension behind explicit feature gates and platform checks.

## Constraints

- Dependency growth is a design decision. No backend dependency should enter the default build
  until the tiering, safepoint, callout, and reporting contracts are documented and tested.
- Native execution must be platform-aware. The active development target is macOS/aarch64, where
  JIT execution interacts with `MAP_JIT`, hardened runtime entitlements, and per-thread write
  protection. x86_64 is less restrictive on developer machines, but still needs W^X discipline and
  unwind/call ABI validation.
- The JIT must not create a second semantics owner. Native code may accelerate bytecode-shaped
  paths, but semantics stay in `lyng-js-ops`, `lyng-js-objects`, `lyng-js-env`, and
  `lyng-js-builtins`.
- GC safety is mandatory. Native frames need precise stack maps or a conservative, documented
  no-GC region policy for the first experiment.
- Callouts must go through a narrow ABI. Native code should call VM/runtime helpers rather than
  reaching across crate internals.

## Option Matrix

| Option | Dependency cost | Platform support | Codegen control | Safepoints and stack maps | Callout integration | Compile latency | Maintenance risk | Fit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Interpreter tiering plus bytecode specialization | None | All current Rust targets | Bytecode-level only | Reuses interpreter frames and existing metadata | Existing VM helpers | Lowest | Low | Best immediate step |
| Cranelift baseline backend | Medium to high, but Rust-native and modular | Officially supports x86-64 and aarch64, among others | Medium: CLIF IR, register allocation, instruction selection owned by Cranelift | Has user-defined stack-map support, but Lyng must produce exact maps | `cranelift-module` / `cranelift-jit` support functions and symbols | Good for JIT use | Medium | Best first native spike |
| dynasm-rs direct assembly | Medium, smaller than Cranelift | x64/x86 and aarch64 support | High: direct instruction selection | Fully Lyng-owned; every safepoint and map must be authored manually | Direct calls are possible, but ABI discipline is fully ours | Very good | High | Useful only after tiering ABI is settled |
| Custom machine-code emitter | None to low external dependency | Only what Lyng implements | Maximum | Fully Lyng-owned | Fully Lyng-owned | Best possible for narrow stubs | Very high | Too early; likely to become architecture debt |
| LLVM ORC | Very high and likely C++/toolchain-heavy | Broad, mature native target support | Medium through LLVM IR and ORC layers | Mature ecosystem, but integration complexity is high | Strong linking and symbol model | Worse for small hot functions | High | Not a first Lyng path |

## Recommendation

Do not add a native backend dependency now.

The next implementation work should finish the JIT-readiness substrate:

1. Define the tiering state and hotness contract in `lyng-js-vm`.
2. Harden feedback vectors and inline-cache data so they are stable inputs to tier decisions.
3. Validate safepoint, root-map, and bailout metadata completeness against bytecode offsets.
4. Add benchmark/report rows that can show when native work is justified.
5. Add feature-gating and execution-memory policy before any backend is wired in.

After those pieces exist, the first native experiment should be a non-default Cranelift spike in
a separate follow-up issue. Cranelift is the best first backend because it is Rust-native,
actively maintained for JIT use, supports the active aarch64 and x86-64 targets, and already has
concepts for user-defined stack maps. The spike should lower only a tiny bytecode subset, call out
to existing VM/runtime helpers for everything else, and prove stack-map/callout/executable-memory
policy before chasing speed.

dynasm-rs and a custom emitter remain plausible later choices for ultra-small stubs or a mature
baseline tier, but using them first would force Lyng to own register allocation, patching,
safepoints, relocation policy, and architecture-specific correctness before the tiering contract is
even validated. LLVM ORC is too heavy for the current dependency and compile-latency policy.

## First Follow-Up Scope

Create a future child issue for a Cranelift baseline-backend spike with these limits:

- non-default Cargo feature or isolated prototype crate
- no default workspace dependency on Cranelift
- macOS/aarch64 and x86_64 feasibility notes before executable code lands
- one tiny straight-line bytecode subset only, such as Smi arithmetic and return
- mandatory callout ABI sketch for slow paths
- mandatory safepoint/root-map story, even if the first subset forbids GC-triggering calls
- benchmark row comparing interpreter, specialized bytecode, and native-spike compile/run costs

## Source Notes

- [Cranelift](https://cranelift.dev/) describes itself as a Rust compiler backend intended for
  embedders and JIT/AOT use, with x86-64 and aarch64 support.
- [cranelift-jit](https://docs.rs/crate/cranelift-jit/latest) provides a JIT library backed by
  Cranelift. The latest docs.rs release checked for this note was 0.131.0, and the crate page
  still labels the crate experimental.
- [Cranelift user stack maps](https://docs.rs/cranelift-codegen/latest/src/cranelift_codegen/ir/user_stack_maps.rs.html)
  show that stack maps are user-defined and must be supplied by the frontend at safepoints.
- [dynasm-rs](https://docs.rs/dynasm/latest/dynasm/) provides Rust procedural macros for dynamic
  assembly, and its runtime docs include [aarch64 support](https://censoredusername.github.io/dynasm-rs/runtime/dynasmrt/aarch64/index.html).
- [LLVM ORC](https://llvm.org/docs/ORCv2.html) provides modular JIT APIs for LLVM IR compilation,
  object linking, eager/lazy compilation, and custom compilers.
- Apple's [JIT entitlement](https://developer.apple.com/documentation/BundleResources/Entitlements/com.apple.security.cs.allow-jit)
  and [Apple silicon JIT porting](https://developer.apple.com/documentation/apple-silicon/porting-just-in-time-compilers-to-apple-silicon)
  docs require explicit `MAP_JIT` handling and note Apple silicon write/execute protection.
