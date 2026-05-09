# Lyng JS Docs Index

This directory documents the Lyng JS engine architecture and current implementation shape.

Start with the crate overview in
[../../crates/lyng-js/README.md](../../crates/lyng-js/README.md), then read the
top-level architecture note and the subsystem note for the area being changed.

## Read First

1. [Engine Overview](../../crates/lyng-js/README.md)
2. [Architecture](architecture.md)
3. [Engineering Standards](engineering-standards.md)

## Architecture Notes

- [Frontend Architecture](frontend-architecture.md)
- [Runtime Model](runtime-model.md)
- [Runtime Primitives](runtime-primitives.md)
- [Runtime Substrate](runtime-substrate.md)
- [Shared Memory And Backing Stores](shared-memory-and-backing-stores.md)
- [Bytecode And VM](bytecode-and-vm.md)
- [Builtin Bootstrap](builtin-bootstrap.md)
- [Dynamic Scope And Eval](dynamic-scope-and-eval.md)
- [Performance Workflow](performance-workflow.md)

## Reports

Generated Test262 and benchmark reports live under
[`../../reports/js/lyng-js/`](../../reports/js/lyng-js/). Those reports are evidence
from verification runs; this directory is the architecture reference.
