# Lyng JS Bytecode and VM

This document freezes the execution contract between the compiler and the interpreter.

## Goals

- make the interpreter fast, direct, and easy to profile
- keep bytecode compact enough for instruction-cache efficiency
- support later JIT work without redesigning the bytecode format
- avoid rethinking frame layout, feedback metadata, or slot access after implementation begins

## Execution Pipeline

The execution pipeline is fixed:

1. parser and sema produce AST plus scope metadata
2. compiler assigns frame-local registers, environment slots, constants, and feedback sites
3. compiler emits immutable bytecode function templates
4. VM executes templates through call frames over a shared register stack
5. feedback and inline-cache state are updated during interpretation

The engine does not use AST interpretation or a stack bytecode format as an intermediate step.

## Compiler Model

Phase 4 uses a direct structured lowering pipeline, not a second optimizing IR.

The compiler contract is:

- sema-owned binding and scope metadata is treated as authoritative
- lowering walks one function at a time and emits final bytecode through patchable builders
- control-flow fixups, exception regions, and child-function records are resolved during
  bytecode construction rather than by a later SSA or graph optimizer
- small local peepholes and constant folding are allowed only when they do not duplicate or
  obscure semantic ownership

This keeps implementation readable and avoids introducing an intermediate representation that
would later need its own ownership, verifier, and rewrite cycle.

## Bytecode Function Template

Each compiled function is represented by an immutable bytecode template.

Required template contents:

- function flags
- name atom
- parameter counts
- register count
- instruction array
- constant pool
- child function references
- scope and capture metadata
- exception table
- feedback site descriptors
- source mapping or source-location table
- optional wide-operand side table

Required header metadata includes:

- function kind and strictness flags
- `this` mode
- `arguments` mode
- formal parameter count and minimum argument count
- optional environment-layout reference for functions that allocate activation environments
- register count, hidden-register count, and child-function count

Bytecode templates are immutable after compilation. Runtime feedback lives outside the
template so compiled code can be shared safely.

Feedback is attached per template, not per closure instance. Closures that share the same
`CodeRef` share feedback vectors and inline-cache state keyed by feedback-site identity. This
is intentional: it keeps runtime footprint lower and keeps optimization signals aligned
with shared compiled code.

## Compilation Units and Code Installation

The compiler owns bytecode templates as plain Rust data before they become live runtime code.

Phase 4 freezes this split:

- `lyng-js-compiler` produces installable compiled units containing bytecode templates and
  child-template graphs
- `lyng-js-vm` or runtime-facing evaluation entrypoints install those templates into the
  code storage domain and obtain `CodeRef` handles
- execution contexts, call frames, and feedback vectors only reference installed `CodeRef`
  values, never transient compiler-owned objects

This keeps the compiler from depending on runtime-owned heap installation details while still
preserving the stable `CodeRef` runtime contract.

## Public Entry Pipeline

Phase 4 public APIs should preserve the compile/install/execute split even if convenience
wrappers compose those steps.

Conceptually:

- `compile_script` parses or consumes frontend artifacts and returns an installable compiled unit
- runtime-facing installation entrypoints turn that unit into live code storage and `CodeRef`
  handles
- evaluation entrypoints execute installed top-level code in a chosen realm or runtime context
- disassembly and inspection APIs operate on compiled units and installed templates without
  requiring execution

The exact Rust type names may evolve, but the ownership split must not.

## Instruction Encoding

The initial default base instruction size is 4 bytes.

This is the planned Phase 4 starting point, not a density-blind commitment that ignores
measurement. Take 3 is freezing:

- register-based bytecode
- explicit operand-width escape hatches
- stable metadata attachment points and instruction identity, with the initial default using
  aligned 4-byte instruction words

Take 3 is not freezing the exact base-word encoding until the Phase 4 density gate closes.
If the default 4-byte base words produce unacceptable bytecode density or instruction-cache
behavior, compact short forms for ultra-common operations may still be introduced without
changing the register-based execution model or the surrounding metadata contracts.

Take 3 uses a fixed-width wordcode with opcode-selected operand layouts rather than a
single monolithic instruction struct.

Conceptual base forms:

```rust
struct InstrABC {
    op: u8,
    a: u8,
    b: u8,
    c: u8,
}

struct InstrABx {
    op: u8,
    a: u8,
    bx: u16,
}

struct InstrAx {
    op: u8,
    ax: i24,
}
```

Meaning:

- `InstrABC` is the common three-register form used for arithmetic, comparisons, moves, and
  other hot register-local operations
- `InstrABx` covers one register plus a 16-bit operand such as a constant-pool index, atom
  table index, slot-table index, or compact descriptor reference
- `InstrAx` covers 24-bit signed immediates such as relative jump deltas
- the opcode selects which decode form applies; handlers do not reinterpret one format as another

For relative control-flow opcodes, signed jump deltas are measured in 4-byte instruction
words rather than raw bytes.

This keeps the common case dense while preserving fixed-width decode and easy instruction
indexing by 4-byte words.

This is a deliberate Phase 0 default. The engine is choosing decode simplicity and aligned
instruction boundaries, but without paying the steady-state density cost of making every
instruction 8 bytes wide. Phase 4 benchmarking must include explicit bytecode-density and
instruction-cache pressure measurements on both x86_64 and aarch64, and the base encoding
is not considered closed until those numbers are captured and accepted.

## Wide Operands

Wide operands are supported from day one without abandoning fixed-width base words.

The plan is:

- common instructions use one 4-byte word in one of the base layouts above
- when an instruction needs both register operands and a full-width immediate, the compiler
  emits a dedicated `Wide` prefix plus one following 4-byte payload word
- wide instructions therefore consume 8 bytes total as two aligned words, with the prefix
  selecting a widened decode path for the following opcode
- the second word's layout is defined per widened opcode family rather than by one universal
  packed struct

Conceptually:

```rust
struct WidePrefix {
    wide_op: u8,
    op: u8,
    a: u8,
    b: u8,
}
```

This keeps:

- the common case at 4 bytes
- wide instructions aligned on the same 4-byte word grid
- jump targets and instruction indexing word-aligned
- rare large-function or large-operand cases supported without redesign
- the second 4-byte word is interpreted per widened opcode family rather than by one universal
  `d: u32` layout; some families use one 32-bit immediate while others may split the word into
  additional register fields, flags, ranges, or signed deltas
- if the Phase 4 density gate fails, short-form encodings for ultra-common instructions may
  still be added as a compatible refinement as long as instruction indexing, metadata
  ownership, and register-bytecode semantics remain unchanged

## Preferred Density Fallback

If Phase 4B rejects the default 4-byte base words, the first refinement is a narrow/wide
opcode family rather than a wholesale second VM design.

Preferred fallback rules:

- ultra-common register-local operations may gain 2-byte narrow forms while general-purpose
  operations retain the existing 4-byte or wide-prefixed families
- likely first narrow candidates are `Move`, `LoadUndefined`, `LoadNull`, `LoadTrue`,
  `LoadFalse`, small constant loads, and `Return`
- 1-byte encodings are reserved for only the most compelling zero-operand cases if the
  density measurements justify their added decode complexity
- instructions that carry feedback, safepoint, exception, or other rich metadata may remain
  in the full-width family even if narrow forms are introduced elsewhere
- metadata tables attach to compiler-owned decoded instruction identity or canonical
  instruction offsets emitted by the bytecode builder, not to ad hoc raw-byte pointer math

This makes the likely escape hatch explicit before Phase 4 implementation starts while still
letting measurements decide whether it is needed.

## Operand Model

The compiler assigns several operand spaces:

- frame register indices
- environment slots
- constant-pool indices
- exception-handler indices
- feedback-site descriptors keyed by instruction offset or descriptor ID

Operand classes:

- register operands
- environment-depth and environment-slot operands
- constant-pool operands
- atom operands
- compact descriptor operands
- relative jump operands

## Register Layout Contract

Every bytecode function owns one fixed frame-relative register space.

That space is partitioned logically into:

- parameter registers
- named local binding registers for uncaptured frame-local bindings
- compiler temporary registers
- hidden compiler-managed registers used for control-flow cleanup, exception state, and
  similar lowering needs

Rules:

- ordinary locals are registers, not a second local-slot abstraction layered above registers
- instructions read and write frame-relative register indices directly
- there is no requirement for dedicated `LoadLocal` or `StoreLocal` opcodes on the normal path
- environment-backed bindings, global-name access, and dynamic fallback paths remain explicit
  opcodes because they cross semantic storage boundaries
- each template records total register count plus hidden-register count so the VM can size
  frames without reverse-engineering compiler policy

The compiler may still use register classes internally, but the runtime contract is one
contiguous register file per frame.

## Constant Pool

The constant pool is typed. It exists to keep instruction streams compact, not to replace
the atom table or the heap.

Likely constant classes:

- numbers that are not inline Smi immediates
- string literals
- bigint literals
- regexp templates
- object or array literal templates where appropriate
- atom references when not encoded directly through specialized opcodes

The compiler should prefer specialized instructions for extremely common cases such as:

- `undefined`
- `null`
- booleans
- small integers

## Opcode Families

Exact opcode names may change, but the family structure is frozen.

### Data Movement

- `Move`
- `LoadUndefined`
- `LoadNull`
- `LoadTrue`
- `LoadFalse`
- `LoadSmi`
- `LoadConst`

### Local, Environment, and Global Access

- `LoadEnvSlot`
- `StoreEnvSlot`
- `LoadGlobal`
- `StoreGlobal`
- `DeleteGlobal`
- `ResolveName` only for dynamic-scope fallback paths

Normal frame-local bindings are addressed directly as registers through the other opcode
families. Explicit load or store opcodes are reserved for environment, global, or dynamic
name access.

### Arithmetic and Comparison

- numeric unary and binary operations
- bitwise operations
- equality and relational operations
- `TypeOf`, `InstanceOf`, and `In`

These opcodes carry feedback-site metadata when profiling is useful.

### Control Flow

- unconditional jumps
- conditional jumps
- switch and jump-table support
- loop-header markers for profiling and future OSR triggers
- return and abrupt completion support

### Object and Array Construction

- create ordinary object
- create array
- define named property
- define keyed property
- store dense element
- load dense element fast path

Array construction relies on the dedicated array object contract rather than ordinary-object
emulation:

- bytecode-created arrays use dedicated array-kind objects with dense element storage
- dense indexed writes performed by bytecode update array length metadata on the fast path
- this is enough for array literals, rest-parameter materialization, and engine-created arrays
- full array exotic behavior layers on top of the same array-kind and element-storage model

### Property Access

- get named property
- set named property
- get keyed property
- set keyed property
- delete property

Named property opcodes must use atom operands plus feedback-site metadata.

### Calls and Construction

- call
- call method
- explicit tail-call form or tail-call mode within the call family
- construct
- create closure
- bind `this` and `new.target` where needed

The call and construct family carries iterator-driven spread, class-specific `super`
entrypoints, and tail-call metadata through the same opcode family. New work should extend
these paths rather than creating parallel call machinery.

### Enumeration

- create `for-in` enumerator state
- advance `for-in` enumerator and branch on completion
- close or discard enumerator state when loop control exits early

The baseline `for-in` path remains part of the execution model, and iterator protocol plus
`for-of` work use the same lowering and VM machinery rather than replacing it.

### Exceptions

- throw
- begin protected region marker or handler metadata reference
- end protected region marker when needed by the lowering strategy
- load current exception into a target register for catch paths

### Later Features on the Same Execution Model

Iteration, classes, modules, generators, async execution, proxies, and shared-memory work all
extend the same execution model described here. They do not justify a second bytecode or VM
contract.

## Scope Lowering Strategy

The compiler lowers bindings according to semantic metadata, not ad hoc runtime lookup.

Rules:

- function-local uncaptured bindings become fixed frame-local registers
- captured bindings become environment slots
- block scopes only allocate runtime environments when sema marks them as environment-backed
- direct `eval` and `with` route through slower dynamic name-resolution strategies in the
  affected function or region

This must be visible in bytecode. The VM should not rediscover lexical structure from names.

## Special Identifier-Operator Lowering

Some syntax-directed operators cannot reuse the ordinary identifier-load path.

Phase 4 freezes these rules:

- `typeof IdentifierReference` lowers through a non-throwing name-probe path rather than an
  ordinary global or dynamic load followed by generic `TypeOf`
- `delete IdentifierReference` is lowered by reference kind:
  - strict-mode early errors are handled before bytecode generation
  - known local, parameter, and environment-backed bindings compile to constant `false`
  - unresolved global-name probes compile to a path that returns `true` when no binding exists
  - actual global-object property deletion uses `DeleteGlobal`
- labeled `break` and `continue` targets are resolved entirely by the compiler; the VM sees
  jumps and structured `finally` unwinding rather than label lookup

This avoids common correctness bugs around undeclared-name `typeof`, sloppy `delete`, and
label handling.

## Function Activation Metadata

Every bytecode template records the activation policy the VM must follow before executing its
first instruction.

Required activation metadata includes:

- strict or sloppy mode
- `ThisMode`
- `ArgumentsMode`
- whether a function environment must be created at entry
- whether parameter initialization and body execution share one environment or require the
  later non-simple-parameter split

`ArgumentsMode` is frozen conceptually as:

- `None`
- `Unmapped`
- `Mapped`

The exact Rust enum names may differ, but the semantic split is fixed now.

## Arguments Object Strategy

Phase 4 must support the existence and aliasing behavior of `arguments` without forcing a
future frame-layout rewrite.

Rules:

- functions that do not need `arguments` do not allocate one
- strict functions and functions with non-simple parameters use `Unmapped`
- sloppy functions with simple parameter lists may use `Mapped` when sema and compiler metadata
  say the `arguments` object is observably needed
- mapped `arguments` must not alias raw frame registers through ad hoc object tricks
- when mapped aliasing is required, the aliased formal parameters are promoted into an
  activation environment or equivalent stable indirection owned by the runtime substrate

This means the rare mapped-arguments path pays an explicit activation cost, while ordinary
functions keep direct frame-register access.

## Rest Parameter Materialization

Rest parameters are activation-time array creation, not iterator-protocol work.

Rules:

- a function with a rest parameter materializes one fresh phase-4 array object per activation
- trailing actual arguments are copied into that array in source order
- rest arrays use the same minimal array-length bookkeeping as array literals and engine-created
  arrays
- rest parameter materialization does not imply support for spread in call or construct
  positions

## Script and Function Entry Shapes

Scripts execute through the same general code-template machinery as functions.

Phase 4 freezes these rules:

- top-level scripts compile to a dedicated top-level bytecode template
- script execution pushes an execution context with script-global lexical and variable
  environments already chosen by the runtime substrate
- source-defined functions compile to child bytecode templates referenced by their parent
- `CreateClosure` binds a child template to the current realm and environment handle rather
  than copying arbitrary local values into closure objects

## Call and Construct Convention

Call-like bytecode must use one explicit calling convention across bytecode and builtin
functions.

Required model:

- call sites identify the callee register, argument range, argument count, result register,
  and the compiler-owned feedback-site descriptor when profiling applies
- method calls additionally carry or produce the receiver value explicitly
- construct sites carry `new.target` semantics explicitly rather than smuggling them through
  ambient VM state
- argument values are passed in contiguous caller registers and copied or materialized into
  the callee frame according to template metadata
- return values write to an explicit destination register in the caller frame
- when the caller register layout already matches the callee parameter prefix, the VM may
  adopt or slide that contiguous range directly into the callee window instead of eagerly
  copying every argument, but this is an implementation optimization on top of the same
  explicit range-call contract

The VM may optimize copies later, but the semantic contract is explicit register-range calls,
not hidden stack machine state.

## Proper Tail Call Contract

Proper tail calls are closed later as a conformance milestone, but Phase 4 freezes the
execution contract now so they do not force a frame-model rewrite.

Rules:

- the compiler performs tail-position analysis and encodes tail-call eligibility explicitly in
  bytecode rather than requiring VM-side semantic rediscovery
- the call family exposes tail-call intent through explicit opcode forms or an equivalent
  dedicated call-mode encoding that is visible in bytecode metadata and disassembly
- the VM tail-call path reuses, recycles, or trampolines the current frame rather than
  relying on unbounded frame growth
- builtins and helpers that perform `PrepareForTailCall` route through the same tail-call
  machinery rather than open-coding custom frame teardown
- full proper-tail-call conformance may land later, but call-site metadata, frame layout, and
  register-window rules are frozen here to admit it without redesign

## Closure and Environment Lowering

Closures capture runtime environments, not arbitrary snapshots of frame memory.

Rules:

- child functions that capture outer bindings receive the current environment handle when the
  closure object is created
- bytecode does not copy uncaptured locals into closure records
- functions that need activation environments for captures, mapped arguments, or dynamic-scope
  fallback create those environments explicitly in the prologue or at the required scope edge
- closure creation is explicit in bytecode and does not rely on later VM inference

## Register Stack and Call Frames

The VM uses one shared register stack and lightweight frame records.

The VM owns:

- `Vec<Value>` register stack
- frame stack
- current exception state
- pending job or suspend state as later phases require

Each call frame stores:

- `CodeRef`
- instruction pointer
- base register index into the shared register stack
- register span or register count for the active template
- return register
- current realm
- current lexical environment
- current variable environment
- `this` value
- `new.target` when relevant
- callee object reference when relevant
- exception-handler stack base or handler cursor
- frame flags

Frame rules:

- the callee sees one contiguous register window starting at the base register index
- the register prefix is reserved deterministically for parameters and frame-local bindings
- compiler temporaries and hidden cleanup registers live in higher indices within the same
  frame window
- frame setup never requires runtime name resolution to locate ordinary locals
- frame records must be recyclable by the tail-call path without a second frame model or
  hidden caller-owned metadata

Frames must also be suspendable by design even before generators and async functions land.
The ordinary frame layout therefore needs to support a later suspend/resume representation
carrying enough information to resume execution correctly, including:

- code identity and instruction position
- some representation of live register or resume state
- lexical and variable environment handles
- `this` and `new.target`
- protected-region state
- frame flags or equivalent resumption metadata

The exact saved-register encoding is intentionally not frozen in Phase 4. Later generator and
async work may choose full-frame snapshots, liveness-trimmed state, or an equivalent resume
encoding, but the active frame layout must not require a redesign to permit suspension.

The shared register stack avoids per-call register-vector allocation.

## Environment Materialization

The compiler and VM cooperate to avoid heap environments when unnecessary.

- frame-local bindings stay in fixed frame-local registers
- when a function or block needs a heap environment, the compiler emits explicit environment
  creation or environment-capture operations
- closures capture environment state through environment handles, not by copying arbitrary locals

This is a core performance requirement.

## Feedback Vectors and Inline Caches

Feedback sites are assigned at compile time.

Each bytecode template contains:

- a feedback-site descriptor list
- a kind for each site, such as:
  - arithmetic
  - comparison
  - property access
  - call target

Runtime feedback data is stored separately and keyed by `CodeRef` plus feedback-site identity.
Site assignment is compile-time and fixed even when runtime storage is allocated lazily.

Required descriptor fields include:

- site kind
- owning instruction offset
- auxiliary metadata needed to interpret the site, such as expected arity, named-property
  atom, or keyed-access classification
- optional compact site IDs when the runtime wants a dense feedback array that does not use
  raw instruction offsets directly

Phase 4 feedback kinds must cover at minimum:

- named property load
- named property store
- keyed property access
- call
- construct
- arithmetic or comparison sites where type specialization is later useful

Inline cache policy:

- `Uninitialized`
- `Monomorphic`
- `Polymorphic`
- `Megamorphic`

Concrete Phase 4 property-cache shape:

- monomorphic named-load and named-store entries cache receiver `ShapeId`, access kind,
  slot offset or accessor slot pair, and any prototype invalidation token needed by that path
- prototype hits also cache holder-shape identity or an equivalent holder record so the VM
  can stay substrate-driven rather than rediscovering lookup structure on every hit
- polymorphic sites store a small fixed array of monomorphic entries in site-local feedback
  storage rather than unbounded per-site chains or hash tables
- megamorphic sites fall back to the generic object-substrate path plus any later shared stub
  or classifier; they do not keep growing per-site descriptor history without bound
- keyed sites may first classify into dense-index, named-atom, or generic slow-path families
  before consulting the same feedback storage

The VM updates cache state during normal execution. Property opcodes never perform global
hash lookups first and only later "learn" to cache.

Property feedback must remain shape-oriented from the start. The fast path is expected to
cache shape-plus-offset or an equivalent stable shape-derived dispatch record rather than a
guest-string-keyed lookup artifact.

Runtime feedback allocation policy:

- feedback-site numbering is fixed at compile time
- feedback sites are identified by compiler-owned descriptors keyed by instruction offset or
  a compact derived site ID; they are not required to consume a dedicated operand lane in every
  profiled opcode form
- runtime feedback arrays may be allocated lazily once a template crosses a hotness threshold
- closures that share one `CodeRef` also share the same feedback storage once it exists
- lazy allocation must not change bytecode semantics or feedback-site identity

Inline-cache invalidation policy:

- shape-based property caches depend on invalidation state owned by `lyng-js-objects`
- caches whose correctness depends on prototype-chain stability must also depend on the
  corresponding prototype invalidation token or equivalent substrate-owned dependency record
- the initial implementation may invalidate conservatively, including flushing affected cache
  families, but it must not rely on unchecked stale shape assumptions after prototype or
  dictionary-mode mutations

## Safepoint and Deoptimization Contract

Take 3 stays interpreter-only in early phases, but Phase 4 freezes the metadata contract that
later JIT tiers need for precise GC and bailout.

Rules:

- allocation-capable calls, loop-backedge or OSR points, exception edges, and other compiled
  reentry boundaries are representable as explicit safepoints or are mappable to explicit
  safepoint descriptors in the bytecode template
- each safepoint descriptor is keyed by a stable safepoint ID or bytecode offset and records
  enough state to reconstruct the interpreter register window, lexical or variable environments,
  `this`, `new.target`, and active exception or cleanup state
- deoptimization snapshots map optimized values back to interpreter registers or environment
  slots owned by the active bytecode template
- later compiled tiers must publish precise stack maps or root maps keyed by safepoints;
  conservative scanning of compiled frames is not permitted
- source mapping, safepoint mapping, and deoptimization metadata may share tables or indices,
  but the ownership remains compiler-visible and explicit
- Phase 4 requires stable metadata shapes and durable attachment points for this information;
  it does not require implementing a full optimized-tier deoptimization pipeline before the
  first runnable interpreter milestone ships

## Exception Handling

Protected regions are represented by explicit exception tables stored per bytecode template.

Each entry records:

- protected instruction range
- handler target
- catch or finally mode
- stack-depth or frame-state restoration data required by the interpreter

The interpreter uses table-driven unwinding rather than scattered manual handler stacks in
unrelated opcode handlers.

`finally` lowering uses a structured cleanup path rather than mandatory body duplication.
The default model is:

- one logical `finally` body
- explicit completion-kind and completion-value state in compiler-generated temporaries or
  equivalent metadata
- re-entry into the correct continuation after cleanup

The compiler may duplicate trivially small cleanup blocks later as an optimization, but
the architectural default is shared cleanup code to control bytecode growth and keep
unwinding semantics explicit.

Phase 4 also freezes these unwinding rules:

- exception-table entries are attached to the owning bytecode template, not mutable VM side
  structures invented at runtime
- unwinding restores register-stack height and frame state from compiler-owned metadata
- catch entry paths load the active thrown value through explicit bytecode-visible state
- `return`, `throw`, `break`, and `continue` crossing a `finally` region all use the same
  lowered completion-kind mechanism rather than bespoke opcode-local control flow

## Minimal Error Helper Contract

Phase 4 introduces the first runtime helpers that create spec-shaped thrown error values
before the full builtin graph from Phase 5 exists.

Rules:

- bytecode handlers and object-aware `lyng-js-ops` wrappers do not manufacture ad hoc string
  throws for ordinary spec error cases
- common throw sites route through narrow helpers such as type, reference, or range error
  creation
- those helpers are realm-aware so later intrinsic wiring can attach the correct prototypes
  without changing call sites or completion propagation
- Phase 5 may deepen the backing object construction, but Phase 4 freezes the helper call
  shape and the fact that `Throw(Value)` already carries guest-visible error objects
- error messages should be engine-consistent and descriptive, typically naming the operation
  plus the offending value category, even though exact message text is not treated as a
  cross-engine compatibility contract
- helper implementations may defer allocating the final `.message` runtime string until that
  property is observed, as long as the eventual visible text matches the helper-selected error payload

## Source Mapping

Bytecode templates include a source-location table sufficient for:

- diagnostics
- stack traces
- disassembly
- later deoptimization support

Source mapping should be compact and should not require storing a full span on every instruction.

Phase 4 also preserves enough source identity for Phase 5 `Function.prototype.toString`:

- source-defined bytecode templates retain a `SourceId` plus function-body or declaration range
  sufficient to recover the original source slice
- the runtime keeps source text reachable by `SourceId` while installed code that depends on it
  remains live
- builtin-native functions use the builtin bootstrap formatting path rather than pretending to
  have source slices

## VM Dispatch

Dispatch must be modular and organized by opcode family.

Rules:

- no monolithic 10,000-line dispatch files
- hot opcode families live in separate modules
- fast paths avoid heap allocation
- uncommon semantics are factored into helpers or slow-path routines

Computed goto or direct-threaded dispatch may be added later, but the bytecode format and
handler ownership must not depend on that choice.

## JIT-Readiness Requirements

Take 3 does not implement a JIT in early phases, but the interpreter must expose the data
one would need later:

- feedback-site metadata
- stable shape IDs
- explicit register-based local access and environment-slot access
- loop headers and hotness counters
- source mapping and frame-state recovery data
- explicit safepoint descriptors and deoptimization snapshots
- a precise stack-map or root-map contract for any future compiled frames

The goal is "JIT-ready without JIT-driven code quality debt."

## VM Tiering Contract

The interpreter owns tiering state per installed `CodeRef`. It does not attach this state to
closure objects or transient frames. Closures created from the same bytecode template therefore
share one hotness record, one feedback vector once allocated, one invalidation epoch, and one
future native-code attachment slot.

Tier states:

- `InterpreterOnly`: default state for installed code; no native tier is eligible
- `Collecting`: tiering is explicitly enabled and the interpreter is collecting hotness
- `ReadyForNative`: the template crossed the hotness threshold and may be considered by a later
  native backend
- `NativeAttached`: reserved future state for an attached native code generation
- `Invalidated`: dependency invalidation cleared hotness and any native attachment; the next
  eligible interpreter event restarts collection without changing guest-visible execution

Hotness inputs:

- feedback-site execution events add one hotness point
- loop backedges add two hotness points
- disabled templates keep their status at `InterpreterOnly`; enabling tiering is an explicit
  host/VM policy decision, not a guest-visible semantic effect

Transition rules:

- installation creates an `InterpreterOnly` state next to the installed function record
- `set_tier_eligible(code, true)` moves `InterpreterOnly` code to `Collecting`
- eligible code moves from `Collecting` to `ReadyForNative` when hotness reaches the VM threshold
- invalidation increments the per-code epoch, clears hotness counters, clears native attachment
  metadata, and moves the code to `Invalidated`
- interpreter execution remains the only execution path in these states until a separate backend
  issue adds native code generation and dispatch

## Invariants

- bytecode is immutable once compiled
- register-based execution and explicit operand-width escape hatches are frozen
- the default base encoding is 4-byte word-aligned until the Phase 4 density gate proves it
  acceptable or replaces it with a documented compact refinement
- wide-prefixed instructions consume two aligned words and do not break word indexing
- frame-local accesses are register-based, not name-based
- feedback-site ownership is decided by the compiler
- the VM does not invent new semantic ownership that belongs in `lyng-js-ops`,
  `lyng-js-env`, or `lyng-js-objects`
