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

`lyng-js-bytecode` owns:

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

Instruction templates store the base instruction stream as encoded bytes. The stream is
variable-width: hot short forms such as `LoadSmi8`, `LoadConst8`, `Jump8`,
`JumpIfFalse8`, `LoadLocal0..3`, and `StoreLocal0..3` use fewer bytes than the full
operand forms when their operands fit. Logical instruction offsets remain instruction
indexes, not byte offsets, while `instruction_bytes()` exposes the compact
representation and `instructions()` provides a decoded iterator for audit, disassembly,
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

`lyng-js-vm` exposes optional per-opcode dispatch counters for profiler and JIT bring-up
work. Counters are disabled by default; when enabled through `Vm`, the interpreter records
one dispatch count per executed bytecode opcode and exposes an immutable
`OpcodeDispatchCounts` snapshot to embedders. The snapshot is runtime observability state,
not bytecode-template metadata.

`lyng-js-bench runtime --count-opcodes` enables the VM counters for executable runtime
workload rows and renders the top 20 opcodes per row in Markdown and JSON reports. Leave
the flag off for normal throughput baselines.

## Inspector Safepoints

`lyng-js-vm` also exposes a minimal debugger hook for interpreter-level inspection. An
embedder installs a `VmDebugHook`, requests a pause globally or at one installed
`CodeRef` and bytecode offset, then receives a `VmDebugPauseContext` at the next matching
safepoint.

The initial safepoints are function entry and `LoopHeader`. The pause context exposes
top-frame-first frame enumeration, register reads, and lexical environment-slot reads.
Step commands are part of the hook return value: step-in pauses at the next observed
safepoint, step-over pauses when the observed frame depth is less than or equal to the
origin depth, and step-out pauses when the observed frame depth is less than the origin
depth.

The debugger path is disabled by default. The interpreter selects a dispatch variant with
debug polling only when a hook is installed and a pause or step request is active, so the
normal no-debugger path does not add an inspector check to every opcode.

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
- Native-code execution is absent from the current engine.
