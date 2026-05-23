# Lyng JS Bytecode And VM

The compiler and VM communicate through immutable bytecode templates, runtime-installed
code references, register windows, call frames, feedback vectors, and metadata tables.

## Execution Pipeline

1. Parser and sema produce AST roots plus scope and binding metadata.
2. The compiler assigns frame registers, environment slots, constants, child functions,
   exception regions, source locations, and feedback sites.
3. The compiler emits `CompiledScriptUnit` and `CompiledFunctionUnit` values containing
   immutable `BytecodeFunction` templates.
4. The VM installs templates into runtime code storage and obtains `CodeRef` handles.
5. The VM executes installed code through call frames over a register stack.
6. Feedback vectors and inline-cache state are updated during interpretation.

The engine does not execute ASTs and does not use stack bytecode.

## Bytecode Templates

`lyng-bytecode` owns:

- `BytecodeFunction`
- `BytecodeFunctionHeader`
- instruction records and opcodes
- constant values
- child function units
- exception handlers
- source map entries
- feedback site descriptors
- safepoint descriptors
- deoptimization snapshots
- direct-eval lexical site metadata
- environment layout references
- builders, decoders, and disassembly helpers

Templates are immutable after compilation. Runtime feedback lives outside templates so
closures sharing a `CodeRef` share feedback keyed by feedback-site identity.

Instruction templates store the instruction stream as encoded bytes. The stream is
variable-width: hot short forms such as `LoadSmi8`, `LoadConst8`, `Jump8`,
`JumpIfFalse8`, `LoadLocal0..3`, and `StoreLocal0..3` use fewer bytes than the full
operand forms when their operands fit. Final jump operands, exception regions, feedback
sites, source locations, safepoints, wide operands, and VM frame PCs use byte offsets
into that stream. The builder may use logical instruction labels while emitting and
optimizing, but finalization lowers every label to a byte offset before producing a
`BytecodeFunction`.

Dispatch reads instructions from `instruction_bytes()` at the active frame PC; see
[Dispatch Substrate](#dispatch-substrate) below for how handlers consume that stream.
Installation validates operands and stores runtime metadata in sparse byte-offset keyed
side tables; it does not keep a decoded instruction cache as an alternate runtime
representation. `instructions()` remains a decoded iterator for audit, disassembly,
validation, and tests.

## Instruction Model

The bytecode is register-based. Operand spaces include:

- frame registers
- environment depths and slots
- constant-pool indices
- atom indices
- feedback-site IDs
- exception-handler indices
- call ranges
- jump deltas

The instruction layer exposes opcode-selected operand layouts, decoder helpers, and
builder validation. Disassembly operates on compiled units and decoded instruction streams.

## Register And Frame Model

Each function owns a fixed register space. The compiler partitions it into:

- parameter registers
- local binding registers
- temporary registers
- hidden registers for exception state, control-flow cleanup, and lowering helpers

The VM executes with `FrameRecord` and `RegisterWindow` structures. Call and construct
entrypoints seed arguments, `this`, new-target state, callee metadata, and realm/context
state according to the installed function record.

## Dispatch Substrate

`lyng-vm` dispatches bytecode through an asm-DSL LLInt-style substrate inspired by
JSC's `LowLevelInterpreter*.asm`. The runtime substrate is in
[`crates/vm/src/dsl/`](../../crates/vm/src/dsl/); the proc-macro lowerer that parses
handler bodies is in [`crates/vm-dsl/`](../../crates/vm-dsl/). The parent design is
[`2026-05-16-asm-dsl-llint-interpreter-design.md`](2026-05-16-asm-dsl-llint-interpreter-design.md).

There is no central dispatch loop and no `Step` enum. Entry to the interpreter sets up
pinned registers and tail-jumps to the first handler; each handler tail-jumps to the
next through a single dispatch table.

### Pinned-register convention (AArch64)

| Register | Role                              |
|----------|-----------------------------------|
| `x19`    | PC                                |
| `x20`    | Register window base              |
| `x21`    | Feedback-vector base              |
| `x22`    | `Vm` pointer                      |
| `x23`    | Dispatch-table base               |
| `x24`    | `LlIntState` pointer              |

AArch64 is the only supported target today. The x86_64 backend is deferred until there
is a concrete user.

### `LlIntState`

The asm-visible per-frame state record is `#[repr(C)] struct LlIntState`, defined in
[`crates/vm/src/dsl/llint_state.rs`](../../crates/vm/src/dsl/llint_state.rs). It holds:

- frame PC offset and the instruction-bytes base pointer
- register-window base
- feedback-vector base
- constants base and the `this`-value mirror (asm-visible frame context)
- frame depth and the safepoint-check epoch
- an opaque pointer to the Rust-side per-call context
- the current prefix byte

Field offsets are part of the ABI and locked by an offset-stability test
(`ll_int_state_offsets_stable`).

### Handler categories

All handlers use one `llint_handler!` macro and share one dispatch table. They divide
into three categories:

- **Hot opcodes.** Full DSL bodies with the fast path expressed inline as `naked_asm!`.
  Per-handler asm-shape budgets live in
  [`tools/lyng-bench/hot-opcodes.toml`](../../tools/lyng-bench/hot-opcodes.toml);
  per-handler asm baselines live under
  [`reports/lyng/dsl-asm-baseline-aarch64/`](../../reports/lyng/dsl-asm-baseline-aarch64/).
- **Warm opcodes** (`op_loop_header`, optionally `op_jump_loop`). Hot path includes a
  mandatory safepoint poll for GC, debugger, and tier-up. The poll is a single byte
  load + branch off the pinned `Vm` register; the slow call runs only on the rare arm.
- **Cold opcodes.** Three-line DSL stubs that `call_slow!` into Rust semantic bodies,
  then `dispatch_after_slow!`. Same dispatch shape as hot handlers, no inline asm
  body.

### Slow-path bridge

`call_slow!` crosses from asm into Rust through a `#[no_mangle] extern "C"` shim that
reconstructs a `LlIntDispatchState` wrapper from the `LlIntState` + opaque Rust context
pair. The shim runs the Rust semantic body — the same semantic functions any in-Rust
dispatcher would call — and returns one of:

- `Continue` — advance PC by N bytes and resume dispatch.
- `Refresh` — semantic body may have triggered GC or moved frame state; refresh
  arena-pointer mirrors on `LlIntState` before resuming.
- `ExitDone` — interpreter exits cleanly with a value.
- `ExitError` — interpreter exits with a `VmError`.

Mirror discipline: `LlIntState` fields that point into GC-or-arena storage are valid
only between Refresh egress events. Any slow-path call may trigger GC; the Refresh arm
restores the mirrors before the next handler reads them.

### Inline-port progress

25 opcodes are inline-ported as of HEAD (DSL-1 Phase 1.A + 1.B + 1.C); remaining
opcodes are cold-stub `call_slow!` shims that delegate to Rust semantic bodies. Phase
1.D and beyond will inline-port the rest of the top-30 hot set. Test262 has remained
at 100% pass on runnable through every phase close. The freshest engine snapshot is
[`reports/lyng/asm-dsl-engine-state-2026-05-22.md`](../../reports/lyng/asm-dsl-engine-state-2026-05-22.md).

## Scope And Environment Lowering

Compiler output distinguishes:

- uncaptured frame-local bindings
- captured environment slots
- global lexical bindings
- object-environment dynamic lookup
- private environments
- direct-eval lexical sites
- module bindings

The VM uses the emitted metadata. It does not rediscover lexical structure from names.

## Feedback And Inline Caches

Feedback vectors are owned by installed code and keyed by feedback-site metadata emitted by
the compiler. The VM records named-property, keyed-property, call, construct, and related
site data through explicit feedback structures.

Inline-cache state is part of interpreter execution. The current tier status surface is
metadata and reporting only; native-code execution is not part of the engine.

## Opcode Dispatch Counters

`lyng-vm` exposes optional per-opcode dispatch counters for profiler and JIT bring-up
work. Counters are disabled by default; when enabled through `Vm`, the interpreter records
one dispatch count per executed bytecode opcode and exposes an immutable
`OpcodeDispatchCounts` snapshot to embedders. The snapshot is runtime observability state,
not bytecode-template metadata.

`lyng-bench runtime --count-opcodes` enables the VM counters for executable runtime
workload rows and renders the top 20 opcodes per row in Markdown and JSON reports. Leave
the flag off for normal throughput baselines.

## Inspector Safepoints

`lyng-vm` also exposes a minimal debugger hook for interpreter-level inspection. An
embedder installs a `VmDebugHook`, requests a pause globally or at one installed
`CodeRef` and bytecode offset, then receives a `VmDebugPauseContext` at the next matching
safepoint.

The initial safepoints are function entry and `LoopHeader`. The pause context exposes
top-frame-first frame enumeration, register reads, and lexical environment-slot reads.
Step commands are part of the hook return value: step-in pauses at the next observed
safepoint, step-over pauses when the observed frame depth is less than or equal to the
origin depth, and step-out pauses when the observed frame depth is less than the origin
depth.

The debugger path is disabled by default. The asm-DSL warm handlers carry a single-byte
poll check off the pinned `Vm` register; when no hook is active, the byte is zero and the
check is a no-op. When a hook is installed and a pause or step request is active, the
byte goes non-zero and the warm handler's slow arm runs `Vm::poll_debug_safepoint`, which
invokes the host hook. Hot opcodes do not carry an inspector check.

## Exceptions And Abrupt Completion

Exception handlers are bytecode metadata. VM operations propagate guest-visible abrupt
completion through engine completion values and helper APIs. Source map entries and
diagnostic metadata preserve source locations for runtime errors.

## Modules And Dynamic Evaluation

The VM owns runtime installation and execution entrypoints for scripts, modules, function
code, direct eval, indirect eval, dynamic import hooks, and embedding extensions. The
compiler owns lowering. The host crate owns host-provided module and dynamic import hooks.

## Invariants

- Bytecode templates are immutable after compilation.
- Runtime execution references installed `CodeRef` handles, not compiler-owned objects.
- Registers are the normal local-access path.
- Feedback is attached by feedback-site identity.
- Every IC-shaped opcode carries a mandatory trailing feedback slot operand
  (Track H, landed). The bytecode encoding is JIT-ready: a future Baseline JIT
  consumes the same bytes and `FeedbackVector` without reshape.
  See [`reports/lyng/jsc-aligned-engine-roadmap.md`](../../reports/lyng/jsc-aligned-engine-roadmap.md).
- Native-code execution is absent from the current engine; the JSC-aligned
  roadmap above plans to add a Sparkplug-style Baseline JIT as Phase 6.
