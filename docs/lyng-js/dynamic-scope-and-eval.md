# Lyng JS Dynamic Scope and Eval

This document describes the JS3 contract for dynamic scope, `eval`, and the shared
dynamic-compilation entrypoints. The same ownership and lowering boundaries are used by the
current compiler, VM, builtin bootstrap, and conformance work.

This note is intentionally narrow. It is about the interaction between otherwise-fast
register- or slot-based execution and the slow paths introduced by:

- direct `eval`
- indirect `eval`
- `with`
- dynamic compilation services reused by the `Function` constructor, direct eval, indirect
  eval, and harness script evaluation

Related notes:

- [Architecture Overview](architecture.md)
- [Runtime Model](runtime-model.md)
- [Runtime Substrate](runtime-substrate.md)
- [Bytecode and VM](bytecode-and-vm.md)
- [ECMA-262 Completion](ecma262-completion.md)

## Goals

- freeze one poisoning model for dynamic-scope features before Phase 4 implementation starts
- define when runtime environments must be materialized even though guest-visible `eval`
  semantics land later
- freeze one shared dynamic-compilation service and cache contract so late `eval` work does
  not invent a second compiler pipeline
- preserve the fast path for normal code while making dynamic paths explicit and correct

## Non-Negotiable Rules

- normal lexical access remains register- or slot-based
- direct `eval` and `with` are explicit slow paths
- dynamic-scope features do not create a second environment or name-resolution runtime
- host code does not own eval semantics or compilation caching
- any function affected by dynamic-scope features is marked by sema and compiler metadata;
  the VM does not rediscover poisoning from source names at runtime

## Shared Dynamic-Compilation Service

Take 3 uses one engine-owned dynamic-compilation service for all late dynamic source
evaluation entrypoints.

Consumers include:

- the Phase 5 `Function` constructor compilation hook
- indirect `eval`
- direct `eval`
- harness `evalScript` / script-source evaluation

Rules:

- `lyng_js_compiler::dynamic` owns parse-goal selection, source wrapping, sema mode,
  diagnostics, compile plumbing, and cache-key policy
- `vm/dynamic_compilation.rs` owns VM state: installed-code caching, caller frame
  inspection, caller lexical/private environment discovery, direct-eval declaration
  validation, and execution
- builtin dispatch calls the VM dynamic-compilation service; it does not inline parse,
  analysis, or compile policy
- the service reuses the normal parse -> sema -> compile -> install pipeline
- the service returns installable compiled units through one typed API family; it does not
  bypass bytecode ownership
- parse goal, target realm, strictness mode, and dynamic-scope mode are explicit inputs
- host code may supply source text or resolution context, but it does not own compilation
  caching or semantic policy

## Compilation Cache Contract

The dynamic-compilation service includes a source-keyed compilation cache from the start.

This is frozen now so late `eval` work does not treat caching as an optional optimization.

Cache-key requirements:

- source text or a content hash of the source text
- parse goal:
  - direct eval
  - indirect eval
  - `Function`-constructor body or equivalent dynamic-function goal
- target realm identity
- strictness mode when it affects semantics
- dynamic-scope mode:
  - global-only
  - direct-eval environment-sensitive
  - other later-sensitive modes if private environments or equivalent features affect visibility

Implementation rule:

- compiler dynamic APIs define the key and compile result shape; the VM owns the installed-code
  cache because `CodeRef` allocation, realm execution, and frame-sensitive state are VM-owned

Additional direct-eval rule:

- direct-eval cache entries must not be reused across incompatible outer-environment layouts,
  private-environment states, or other caller-sensitive binding conditions

Additional indirect-eval rule:

- indirect `eval` may reuse code more broadly because it executes in the target realm's
  global environment rather than the caller's lexical environment

## Poisoning Model

Dynamic-scope features are expressed as explicit sema or compiler metadata, not as ambient
runtime guesses.

Required function- or scope-level flags include:

- contains direct `eval`
- contains `with`
- requires dynamic name resolution in this region
- requires activation-environment materialization for observably aliased bindings

The exact Rust type names may differ, but the semantic split is frozen.

### Clean Code

Code not marked by the dynamic-scope flags keeps the normal Phase 4 fast path:

- uncaptured locals stay in frame registers
- captured bindings stay in environment slots
- global lookup uses atom-based global paths
- no dynamic name-probe opcodes are emitted on the normal path

### Direct Eval

Direct `eval` is the caller-sensitive path.

Rules:

- direct `eval` does not force a second compiler pipeline
- when sema marks a function or region as direct-eval-sensitive, the compiler materializes
  activation environments for the bindings whose observability requires heap-backed identity
- unaffected temporaries and provably unobservable locals may remain in registers
- the compiler emits explicit dynamic name-resolution paths only where direct `eval` can
  observe or introduce bindings that invalidate the normal slot path

### Indirect Eval

Indirect `eval` is a global-environment compilation path.

Rules:

- indirect `eval` does not poison the caller's local register layout
- it executes against the chosen realm's global lexical or variable environment
- its compilation cache can therefore be broader than direct eval's cache

### With

`with` is the object-environment slow path.

Rules:

- `with` introduces an object environment layered into the existing environment runtime
- name resolution inside affected regions uses explicit dynamic probes rather than pretending
  the normal lexical fast path still applies
- unaffected outer or sibling regions remain on the fast path
- `@@unscopables` extends the same object-environment machinery; it does not create a second
  `with` runtime path

## Name-Resolution Lowering

The compiler owns the lowering distinction between fast and dynamic name access.

Required lowering categories:

- direct register access for uncaptured locals
- direct environment-slot access for captured bindings
- atom-based global access
- dynamic `ResolveName`-style fallback only for regions marked as poisoned by direct `eval`
  or `with`

Rules:

- `typeof IdentifierReference` and sloppy `delete IdentifierReference` keep their specialized
  lowering rules even in the presence of dynamic-scope features
- label resolution remains compiler-owned and is unaffected by dynamic name resolution
- the VM executes the emitted resolution category; it does not infer lexical structure from names

## Environment Materialization

The dynamic-scope contract is not "materialize everything just in case."

Rules:

- activation environments are created only when the compiler metadata says the function or
  scope needs them for captures, mapped arguments, direct-eval observability, or `with`
- dynamic-scope poisoning is region-aware where the compiler can express that safely
- once a binding is forced into an environment for observability, all later accesses to that
  binding use the environment-backed path

## Current Status

- the `Function` constructor, indirect eval, direct eval, and harness script evaluation enter
  through the shared compiler dynamic API and VM dynamic-compilation service
- compiler and VM lowering already rely on this poisoning and environment-materialization model
- eval and `with` conformance work should extend this contract rather than adding another
  parse/sema/compile flow in builtin dispatch or VM call sites

## Invariants

- direct `eval` and `with` are slow paths by explicit design
- dynamic compilation reuses the normal compiler pipeline
- dynamic compilation owns a cache contract from the start
- direct eval cache entries are caller-sensitive
- indirect eval cache entries are realm-global-sensitive, not caller-local-sensitive
- dynamic-scope semantics do not move ownership of environments or name resolution into the VM
