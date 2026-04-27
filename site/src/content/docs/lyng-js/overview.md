---
title: Lyng JS overview
description: A high-level introduction to the Lyng JavaScript engine.
---

Lyng JS is a JavaScript engine written in Rust. It targets the ECMA-262
specification and is structured as a classic pipeline: a lexer feeds a
parser, the parser produces an AST that the compiler lowers to bytecode,
and a stack-based virtual machine executes that bytecode against a
runtime substrate that owns objects, functions, and the garbage collector.

This page is the public entry point. More detailed design notes live with
the source under each crate.

## What's here today

- **Lexer and parser** for the full ECMA-262 grammar.
- **Bytecode compiler** that lowers the AST to a compact instruction set.
- **Virtual machine** with a tracing garbage collector and a runtime
  primitive layer (objects, arrays, functions, closures, iterators, …).
- **Builtin library** scaffolding for the global object and core
  prototypes.

## What's next

Coverage of the standard library, Temporal, shared memory, and the rest of
the long tail of ECMA-262 is in active development. Watch the
[blog](/blog/) for progress reports.
