# Lyng JS Dynamic Scope And Eval

Dynamic scope behavior is represented explicitly in frontend metadata, compiler lowering,
bytecode metadata, runtime environments, and VM execution. The normal path remains register
and slot based.

## Ownership

- `lyng-js-parser` parses direct eval call syntax, indirect eval expressions, `with`
  statements, dynamic import expressions, and function constructor source.
- `lyng-js-sema` marks lexical scopes affected by direct eval, object environments,
  unscopables, and dynamic lookup.
- `lyng-js-compiler` emits direct-eval metadata, dynamic lookup operations, and environment
  materialization requirements.
- `lyng-js-bytecode` stores direct-eval lexical site metadata and environment layout
  references.
- `lyng-js-env` owns object environments, declarative environments, global environments,
  and execution-context state.
- `lyng-js-vm` executes eval and dynamic lookup paths.
- `lyng-js-builtins` dispatches `eval`, `Function`, dynamic import helpers, and related
  native builtin entrypoints.

## Eval Forms

Direct eval uses the caller's lexical context according to ECMA-262 rules. The compiler
emits site metadata so the VM can compile/evaluate source against the correct realm,
strictness, lexical environment, variable environment, private environment, and source
location.

Indirect eval evaluates as global code in the target realm. It does not inherit the caller's
lexical environment.

Function constructor paths compile source through the dynamic compilation service and
produce callable function objects with realm and builtin semantics handled by the runtime
and builtins layers.

## With Environments

`with` creates an object environment in the environment chain. Name resolution through that
environment observes `@@unscopables`, object property lookup, proxy traps, and abrupt
completion.

The compiler marks code that can encounter dynamic object-environment lookup. Unaffected
lexical access remains lowered to frame registers and environment slots.

## Dynamic Lookup

Dynamic lookup is explicit in bytecode and runtime environment operations. It is used for:

- direct eval affected lexical scopes
- object environments introduced by `with`
- global lookup sites whose behavior is host or realm observable
- named operations that must consult unscopables or object internal methods

The VM delegates object and environment semantics to `lyng-js-ops`, `lyng-js-env`, and
`lyng-js-objects`.

## Invariants

- Clean lexical code stays on registers and environment slots.
- Dynamic scope behavior is visible in sema, bytecode metadata, and VM operations.
- Eval compilation uses shared parser, sema, compiler, and installation paths.
- Object-environment lookup observes proxy and unscopables behavior through shared
  operation APIs.
