# Lyng JS Docs Index

This directory documents the Lyng JS engine architecture and current implementation shape.

Start with the crate overview in
[../../crates/README.md](../../crates/README.md), then read the
top-level architecture note and the subsystem note for the area being changed.

## Read First

1. [Engine Overview](../../crates/README.md)
2. [Architecture](architecture.md)
3. [Engineering Standards](engineering-standards.md)
4. [asm-DSL LLInt-style Interpreter Design](2026-05-16-asm-dsl-llint-interpreter-design.md) — parent design for the current dispatch substrate.
5. [LLInt-Parity State Of The Engine](../../reports/lyng/llint-parity-state-of-engine.md) — live performance target, evidence, and optimization direction.

## Architecture Notes

- [Frontend Architecture](frontend-architecture.md)
- [Runtime Model](runtime-model.md)
- [Runtime Primitives](runtime-primitives.md)
- [Runtime Substrate](runtime-substrate.md)
- [Garbage Collection](gc.md)
- [Shared Memory And Backing Stores](shared-memory-and-backing-stores.md)
- [Bytecode And VM](bytecode-and-vm.md)
- [asm-DSL LLInt-style Interpreter Design](2026-05-16-asm-dsl-llint-interpreter-design.md)
- [Builtin Bootstrap](builtin-bootstrap.md)
- [Dynamic Scope And Eval](dynamic-scope-and-eval.md)
- [Performance Workflow](performance-workflow.md)
- [V8 And Octane Benchmark Plan](v8-octane-benchmark-plan.md)

## Reports

Generated Test262 and benchmark reports live under
[`../../reports/lyng/`](../../reports/lyng/). Those reports are evidence
from verification runs; this directory is the architecture reference.
