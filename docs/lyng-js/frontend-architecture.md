# Lyng JS Frontend Architecture

The frontend turns source text into parser and compiler inputs. It owns lexical facts,
arena ASTs, parse entrypoints, early errors, scope metadata, binding metadata, and source
locations. It does not allocate runtime values or execute guest code.

## Crate Ownership

- `lyng-js-common` owns shared frontend IDs, source spans, string interning identifiers, and
  small cross-crate helper types.
- `lyng-js-lexer` owns source scanning and token production.
- `lyng-js-ast` owns arena-backed AST node storage and typed node IDs.
- `lyng-js-parser` owns grammar recognition, cover grammar handling, recovery, and parse
  root construction.
- `lyng-js-sema` owns early errors, lexical scope construction, binding tables, capture
  analysis, private-name analysis, and layout metadata consumed by the compiler.

The frontend dependency direction is:

```text
common
  -> lexer
  -> ast
  -> parser
  -> sema
```

`parser` depends on `common`, `lexer`, and `ast`. `sema` depends on `common` and `ast`.
The compiler consumes parser and sema outputs; the frontend does not depend on compiler,
runtime, VM, or builtin crates.

## Tokens

The lexer produces compact token records with source spans and contextual flags. It tracks
line terminators for automatic semicolon insertion and differentiates lexical modes for
regular expressions, templates, strings, comments, numeric literals, identifiers, keywords,
punctuation, and private identifiers.

Token payloads stay compact:

- identifiers and keywords carry internable text information
- numeric and string literals preserve enough source information for parser and compiler use
- template components expose cooked/raw boundaries to the parser
- comments are not AST nodes

## AST

`lyng-js-ast` stores nodes in arenas and refers to them through typed IDs. The AST keeps
script roots, module roots, declarations, statements, expressions, patterns, literals,
functions, classes, template literals, private names, and source spans distinct.

Important AST rules:

- expression and pattern nodes are separate where the language needs different validation
- cover grammar nodes are resolved by parser and sema before compilation
- parse roots expose script and module shapes explicitly
- AST nodes contain syntax shape, not runtime storage decisions

## Parser

The parser is a hand-written recursive-descent parser with explicit grammar ownership.
It handles:

- scripts and modules
- expressions, statements, declarations, classes, functions, methods, and arrows
- destructuring patterns
- template literals
- optional chaining
- private identifiers
- automatic semicolon insertion
- syntax recovery that permits multiple diagnostics from one parse

The parser reports syntax errors with source locations and returns typed parse roots for
successful parses.

## Semantic Analysis

`lyng-js-sema` converts syntax into compiler-facing semantic facts:

- scope table
- binding table
- per-scope binding layout
- capture metadata
- function metadata
- private-name tables
- early-error diagnostics
- lexical, function, global, object, module, and private-environment requirements

Sema is the owner of lexical binding decisions. The compiler treats sema output as
authoritative and lowers bindings to frame registers, environment slots, global bindings,
or dynamic lookup paths according to that metadata.

## Compiler Contract

Frontend output is compile-ready when:

- parse succeeded for the selected script or module root
- sema produced binding and scope metadata
- early errors have been reported
- captured bindings and environment allocation requirements are known
- private names and class-related metadata are resolved
- source locations are available for diagnostics and source maps

The compiler does not rediscover lexical structure from strings. It consumes AST IDs,
scope IDs, binding IDs, atoms, spans, and layout data.

## Invariants

- Frontend crates do not allocate `Value`, `ObjectRef`, `EnvironmentRef`, or `CodeRef`.
- Source spans remain available through parser, sema, compiler, and bytecode metadata.
- Lexical access decisions are made before bytecode execution.
- Runtime semantics remain in `lyng-js-ops`, `lyng-js-env`, `lyng-js-objects`,
  `lyng-js-vm`, and `lyng-js-builtins`, not in the parser.
