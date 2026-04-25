# Lyng JS Engineering Standards

This document defines the implementation standards for take 3. These are not optional
style preferences. They are part of the architecture because they directly affect
maintainability, performance, and the risk of future rewrites.

## Principles

- spec fidelity before cleverness
- clear ownership before convenience
- measured performance before folklore
- readable systems code before framework-style abstraction
- explicit invariants before "it should be fine"

## Layering Rules

Crate boundaries are architectural boundaries.

- `lyng-js-ops` is the only owner of abstract operations
- `lyng-js-objects` is the owner of internal method behavior
- `lyng-js-env` owns realms, environments, execution contexts, and jobs
- `lyng-js-vm` executes bytecode; it does not become the backup home for semantics that
  belong elsewhere
- `lyng-js-compiler::dynamic` owns dynamic source parse/sema/compile policy; VM dynamic
  compilation owns installed-code caching, caller-sensitive frame/environment discovery, and
  execution
- `lyng-js-builtins` installs and implements builtin objects, but should call shared
  abstract operations and object helpers rather than duplicating semantics
- public builtin bootstrap allocation should live in family installers under
  `public/families/`; `public.rs` should remain orchestration plus shared bootstrap helpers
- `lyng-js-ops::object` is the public semantic surface for proxy-observable object
  operations such as `Get`, `Set`, `HasProperty`, `GetOwnProperty`,
  `DefineOwnProperty`, prototype operations, and own-key collection
- conformance embeddings such as `lyng-js-test262` own `$262`, helper catalogs, and
  runner/report policy; engine crates only expose generic embedding hooks

Rejected patterns:

- duplicating coercion logic in builtins and VM handlers
- hiding environment logic inside compiler or VM code
- property semantics implemented one way in objects and another in reflect or builtin helpers
- choosing ordinary-object and proxy-object paths at VM or builtin call sites when an
  `ObjectOpsContext` can select the right path inside `lyng-js-ops`
- implementing Function-constructor or eval parse/sema/compile flows inside builtin dispatch
  instead of the shared dynamic-compilation service

## API Ownership

Public APIs should be stable and intentional.

- use typed handles and typed IDs across crate boundaries
- avoid `usize` as a cross-crate semantic type when a dedicated typed ID should exist
- prefer explicit structs over loose tuples in public APIs
- make ownership obvious in type names and function signatures
- keep raw object-storage/internal-method helpers private or clearly named as
  ordinary-only/bootstrap-only when bypassing the proxy-aware object operation surface

If an API is intentionally temporary, that must be documented in the owning phase file.

## Hot Path Rules

The engine must treat hot-path discipline as a code quality requirement.

- no `Rc`, `Arc`, `RefCell`, `Mutex`, or trait-object dispatch in interpreter hot paths
- no string-keyed lookup in normal lexical access or normal named-property access
- no per-access heap allocation in:
  - arithmetic
  - local variable access
  - environment-slot access
  - shape-based property reads or writes
- no hidden cloning of large metadata structures in parser, compiler, or VM paths

Allowed tradeoffs:

- explicit slow paths for dynamic cases
- specialized data-oriented code in hot modules
- benchmark-proven complexity when the performance gain is real and the invariants remain clear

## Data Structure Rules

- prefer compact, contiguous storage for hot runtime data
- split hot state from cold metadata
- use side tables when they keep the hot structure small
- use fallback dictionary modes rather than making every normal object or array pay for highly dynamic cases
- avoid general-purpose hash maps in the fast path for properties or lexical bindings

## Spec Citation and Documentation Rules

- normative implementations cite the owning ECMA-262 section in module docs or function comments
- comments explain invariants, ownership, and non-obvious tradeoffs
- comments do not narrate obvious Rust syntax
- every major runtime structure should have a short module-level comment explaining:
  - what it owns
  - why it has its current shape
  - what the hot path is

## Safety Rules

- `unsafe` is allowed only when:
  - there is a measurable or strongly justified reason
  - the invariants are documented locally
  - a safe alternative was considered and rejected for a clear reason
- guest-triggered execution paths must not panic
- use typed error or completion paths for guest-visible failures
- `panic!` is reserved for:
  - impossible internal states
  - debug-time invariant failures
  - process setup failures outside guest semantics

Debug assertions are encouraged. Release-mode guest behavior must remain explicit and typed.

## Testing Rules

Every owned semantic area needs direct tests.

- unit tests for abstract operations, object semantics, environment semantics, compiler lowering, and VM behavior
- integration tests for multi-crate behavior such as compile-plus-execute and bootstrap-plus-execute
- targeted test262 slices per phase
- negative tests for parser and early-error behavior
- any phase that claims whole-engine ECMA-262 completion must also produce a whole-suite
  test262 report plus a checked-in exclusion manifest for intentionally out-of-scope cases
- fuzzing for:
  - lexer
  - parser
  - bytecode decoder or disassembler

A passing end-to-end test is not a substitute for direct unit tests of the owning layer.

## Benchmark and Memory Rules

Performance claims require measurements.

- changes in hot crates should come with benchmark coverage or justification for why that is not yet practical
- any change to a frozen data structure requires benchmark impact review
- benchmark groups should include:
  - frontend latency
  - interpreter throughput
  - bytecode density and instruction-cache pressure
  - property access
  - function call overhead
  - allocation and GC behavior
  - memory footprint, including explicit accounting for atoms, feedback, shapes, environments,
    and code templates
- initial memory budgeting targets should be tracked explicitly:
  - hot object header at or below 32 bytes on supported 64-bit builds
  - declarative environment record at or below 32 bytes before slot storage
  - default-realm bootstrap live heap within the current documented budget
- barrier-ready store helpers in barrier-free collectors are expected to compile to direct
  stores in optimized builds; benchmark or generated-code inspection is an acceptable proof
- reports belong under `reports/js/lyng-js/`

Known regressions may be accepted only if they are documented and the tradeoff is explicit.

## Readability Rules

- large semantic domains must be split across multiple modules
- names should follow spec concepts unless doing so would materially damage readability or performance
- helper functions should reduce complexity, not hide it
- avoid deep abstraction stacks in hot runtime modules
- prefer straightforward control flow over metaprogramming

The target style is rigorous systems code, not clever library code.

## Review Checklist

Every meaningful change should be reviewable against these questions:

- does this code live in the right crate
- does it duplicate existing semantic ownership
- does it add allocation or string lookup to a hot path
- does it preserve the frozen structure decisions
- does it write a `Value` or typed handle into traced heap storage without going through the
  barrier-ready helper API
- are the invariants documented
- are tests at the owning layer present
- if hot-path behavior changed, are benchmarks or memory notes included

If the answer is unclear, the change is not ready.

## Change Control for Frozen Structures

The following changes require plan and architecture updates in the same patch series:

- changing `Value` shape or size
- changing handle widths or handle stability assumptions
- changing object header or property storage structure
- changing environment layout strategy
- changing bytecode encoding or frame layout
- changing the host boundary
- changing generic embedding-extension surfaces that external tools depend on

Required justification:

- why the previous choice is no longer viable
- what code and docs must change
- expected correctness, performance, and memory impact
- migration risk for already-built phases

## Done Criteria

Work is not done when it compiles. It is done when:

- the owning semantics are implemented in the correct layer
- unit tests and targeted integration tests exist
- the relevant plan or architecture docs are updated if ownership changed
- whole-engine conformance claims come with zero unexplained failing test262 cases outside
  the checked-in exclusion manifest
- any material hot-path effect is measured or explicitly deferred with a reason
- the resulting code is still readable enough that the next engineer does not need to reverse-engineer intent
