# Lyng JS Frontend Architecture

This document describes the current JS3 frontend architecture: shared frontend types,
lexer/token model, AST storage, parser shape, semantic-analysis side tables, and the
parser/sema API contract.

This note is intentionally frontend-scoped. Runtime layout, bytecode, and VM execution are
specified elsewhere:

- [Architecture Overview](architecture.md)
- [Runtime Model](runtime-model.md)
- [Bytecode and VM](bytecode-and-vm.md)
- [Engineering Standards](engineering-standards.md)

## Goals

- keep the syntax pipeline fast and memory-efficient on large inputs
- produce compiler-oriented data structures rather than tooling-oriented trees
- keep parser and sema output stable enough that the compiler does not need to reinvent frontend analysis
- freeze the cross-phase interface between the frontend and the runtime/compiler layers

## Crate Ownership and Dependency DAG

The frontend stack is:

```text
lyng-js-common
  -> lyng-js-lexer
  -> lyng-js-ast

lyng-js-parser
  depends on lyng-js-common, lyng-js-lexer, lyng-js-ast

lyng-js-sema
  depends on lyng-js-common, lyng-js-ast
```

Cross-phase interface:

```text
lyng-js-common
  -> lyng-js-types
  -> lyng-js-compiler
  -> lyng-js-vm
  -> lyng-js-builtins
```

This is the frozen answer to the shared-atom question:

- `AtomId` and `AtomTable` live in `lyng-js-common`
- the frontend depends on `lyng-js-common`, not on runtime crates
- later runtime and compiler crates also depend on `lyng-js-common` for atom identity
- the shared atom namespace has one `AtomId` space with both permanent atoms and
  runtime-collectible atoms
- frontend-created atoms are permanent; runtime atomization may allocate collectible atoms
- `lyng-js-sema` consumes AST-owned parse roots and does not depend on `lyng-js-parser`

`lyng-js-types` does not own the atom ID type. It consumes it.

## Shared Frontend Types (`lyng-js-common`)

`lyng-js-common` owns the data structures shared across the full engine that are not
runtime values:

- `SourceId`
- `TextOffset`
- `TextRange`
- `Span`
- `Diagnostic`
- `Severity`
- `AtomId`
- `AtomTable`

The plan is:

- `SourceId` is a compact copyable identifier
- `TextOffset` and `TextRange` use 32-bit offsets
- `Span` is `(SourceId, TextRange)`
- `AtomId` is a compact copyable ID
- `AtomTable` is the shared atom namespace used by frontend, compiler, and runtime, with
  explicit permanent versus collectible entry classes managed by the caller

`lyng-js-common` must not grow guest runtime semantics. It is shared infrastructure, not a
miscellaneous dumping ground.

## Token Model (`lyng-js-lexer`)

The lexer is streaming. The parser consumes tokens on demand and does not materialize a
full token vector during normal parsing.

### Token Shape

Conceptually:

```rust
struct Token {
    kind: TokenKind,
    span: Span,
    flags: TokenFlags,
    payload: TokenPayload,
}
```

Rules:

- punctuation and keywords have no payload
- identifiers and private identifiers carry `AtomId`
- non-trivial literal payloads are referenced through compact lexer-local IDs rather than
  boxed payloads embedded in the token itself

The token record must remain compact and copyable.

### Token Flags

The token flags must include at least:

- `PRECEDED_BY_LINE_TERMINATOR`
- `CONTAINS_ESCAPE`
- any additional compact flags needed for grammar or early-error correctness

These flags exist for correctness, not convenience:

- ASI depends on line terminator tracking
- escaped identifier and keyword forms affect strict-mode and grammar decisions

### Token Payload Policy

Payload categories:

- `None`
- `Atom(AtomId)`
- `Literal(LiteralTokenId)`

The lexer-local literal side tables are temporary and parser-facing. They are used for:

- string literal payloads
- bigint literal payloads
- regexp literal payloads
- template chunk payloads
- numeric-literal auxiliary flags when needed beyond the token flags

The parser copies canonicalized literal data into the AST or AST literal tables. The AST
does not borrow from lexer-local side tables.

### Lexing Modes

The lexer/parser handshake must support:

- regular expression literal versus division disambiguation
- template literal continuation mode
- private identifier lexing
- contextual keyword handling without heap allocation

The parser controls the lexer mode through explicit state rather than through global magic.

### Tokenization Policy

Take 3 is choosing a streaming parser front end with lightweight token state over a
pre-tokenized source model.

Consequences:

- lower memory footprint on large files
- no second copy of the full token stream in normal operation
- parser complexity is slightly higher, but the architecture avoids a full token-buffer rewrite later

Opt-in token materialization for tests or debugging is acceptable, but it must not become
the default parser path.

## AST Layout (`lyng-js-ast`)

The AST is compiler-oriented and arena-backed, using typed IDs instead of borrowed node references.

### AST Container

The owning container is an append-only `Ast` value with typed arenas per node family.

Required node-ID families include:

- `ScriptId`
- `ModuleId`
- `StmtId`
- `ExprId`
- `DeclId`
- `PatternId`
- `BindingId` for syntax-level binding nodes if needed
- `FunctionId`
- `ClassElementId`
- `TemplateLiteralId`

Node IDs are 32-bit and copyable.

`lyng-js-ast` also owns the parse-root wrapper types consumed by later phases:

- `ParsedScript`
- `ParsedModule`

These wrappers contain:

- owning `Ast`
- root ID
- source kind
- syntax diagnostics
- directive-prologue summary

The parser constructs these values, but they are defined in `lyng-js-ast` so sema and the
compiler can consume them without a dependency on `lyng-js-parser`.

### Arena Strategy

The AST uses an arena-of-vectors design:

- each node family has its own contiguous storage
- list children are stored in append-only side arenas and referenced by `(start, len)` or
  typed list IDs rather than per-node heap `Vec`s
- nodes store spans and child IDs only

This means:

- no public lifetime-heavy borrowed-node API
- no per-node heap allocation for child lists
- good cache locality for per-family traversals

### Node Shape

Each major node stores:

- node-specific payload
- `Span`

Node spans stay inline because:

- diagnostics need them constantly
- the compiler and sema need them often enough that a separate side table would add indirection everywhere

### Literal Storage

Literal storage is split by payload size:

- numeric literals store parsed numeric value inline as `NumericLiteralValue`
- `NumericLiteralValue::Int32(i32)` is used for exact integer literals that fit in `i32`
- `NumericLiteralValue::Number(f64)` is used for all other numeric literals
- string, bigint, regexp, and template literal payloads live in AST-owned literal tables
  referenced by small IDs

This keeps common expression nodes compact while still avoiding lexer-side borrowing.
It also means the compiler does not need to rediscover integer-ness from source text just
to choose between small-integer and generic-number constant loading.

### Template Literal Representation

Template literals are first-class AST shapes, not ad hoc string-literal special cases.

Rules:

- expression variants must cover both ordinary template literals and tagged templates
- template payloads live in AST-owned tables keyed by `TemplateLiteralId`
- each template record stores quasis in source order and the interleaved expression IDs
- each quasi stores both cooked and raw string payload IDs
- cooked payload may be absent for quasis whose escape processing is invalid but still
  observable through tagged-template semantics
- tagged templates reuse the same template payload with a separate tag `ExprId`

### Parse Recovery Nodes

The AST includes explicit invalid or error nodes for parser recovery.

Rules:

- the parser may emit invalid placeholder nodes to continue after syntax errors
- sema recognizes invalid nodes and avoids inventing bindings or semantic meaning for them
- the compiler does not run if syntax or sema diagnostics are fatal

Error nodes exist to improve diagnostics and recovery, not to make invalid programs executable.

### Expression and Pattern Separation

Patterns are their own node family, not "expressions interpreted later."

This is a frozen decision.

The parser resolves expression-versus-pattern shape at parse time for constructs where the
grammar requires it. The compiler and sema do not reinterpret the tree later.

## Parser Architecture (`lyng-js-parser`)

The parser is a recursive-descent parser over the streaming lexer.

### Parser State

Required parser state includes:

- current token
- one-token lookahead
- strict-mode state
- contextual parse flags such as:
  - no-`in`
  - allow-`yield`
  - allow-`await`
  - in-parameter-list
  - in-class-body
  - cover-form state for parenthesized expressions and async-arrow heads

Two-token lookahead is allowed only where the grammar truly requires it. It should not
become the generic parser model.

Known allowed cases include:

- async-arrow and `async function` disambiguation
- binding-pattern versus expression ambiguities in declaration and `for`-header positions
- narrow grammar edges such as `let`-led statement disambiguation where one token is not sufficient

### Cover Grammar and Arrow Functions

The parser owns cover-grammar conversion. It is not deferred to sema or the compiler.

Rules:

- the parser may use temporary parser-local cover forms for parenthesized expressions and
  async-arrow candidates
- cover forms do not become public AST node kinds
- when `=>` is observed, the parser lowers the cover form into parameter-list and pattern nodes
- when `=>` is not observed, the same temporary form lowers into the ordinary expression shape
- arrow-parameter early errors are reported against the converted parameter form, not by
  asking later phases to reinterpret expression nodes

### Entry Points

Required entry points:

- `parse_script`
- `parse_module`

The parser produces:

- AST
- root ID
- syntax diagnostics
- directive-prologue info needed by sema

### Recovery Policy

The parser should recover well enough to report multiple syntax errors in one pass.

Recovery rules:

- recover to statement boundaries for statement-level failures
- recover to delimiter boundaries for list-like syntax
- emit invalid placeholder nodes instead of abandoning the AST entirely
- never fabricate bindings or directives that were not parsed
- invalid declaration subtrees poison only the local construct; sema still analyzes
  surrounding valid siblings and enclosing scopes

The parser is not trying to be a language server, but it should not stop at the first error either.

## Semantic Analysis (`lyng-js-sema`)

`lyng-js-sema` owns static semantics needed for early errors and later compilation.

### Sema Output Shape

Sema output is side-table driven. It does not mutate AST nodes in place.

Required ID families:

- `ScopeId`
- `SemanticBindingId`
- `FunctionSemaId`
- `UseSiteId`
- `PrivateNameId`

Required side tables:

- scope table
- binding table
- function table
- capture table
- directive and strictness metadata
- reference-resolution results
- private-name definition and resolution tables

Reference resolution is part of sema ownership. Each identifier, `super`, and similar
name-like use site that later compilation needs must have a stable `UseSiteId` and a
resolved classification in the sema output.

### Scope Table

Each scope record must include:

- parent scope
- scope kind
- owning function or module
- strictness
- dynamic-scope flags
- binding range or binding list reference
- child-scope range or list reference

Dynamic-scope flags are required for:

- direct `eval`
- `with`
- other constructs that weaken slot-based assumptions

Each function contributes the scope records required by static visibility rules. In
particular, sema must model a distinct parameter scope when ECMA-262 requires one, rather
than assuming every function has a single flat scope.

### Binding Table

Each binding record must include:

- name atom
- declaration kind
- declaring scope
- hoist and TDZ properties
- whether it is captured
- whether it requires environment storage
- whether it resolves to global, local, or dynamic lookup

Sema computes semantic storage class, not final register indices.

Required storage classes:

- `FrameLocal`
- `EnvironmentSlot`
- `GlobalName`
- `DynamicLookup`

The compiler later assigns actual register numbers and environment-slot numbers using
these classifications and the scope layout order.

### Binding Introduction and Parameter Scope

Sema owns flattening syntax-level binding forms into concrete bindings.

Rules:

- destructuring patterns introduce one binding record per bound identifier
- binding introduction order within a pattern is left to right in source order
- nested object and array patterns do not reorder bindings when flattened
- default parameter initializers are analyzed in parameter-scope visibility, not body-scope visibility
- earlier parameters are visible to later parameter initializers
- later parameters are not visible to earlier parameter initializers
- when a function has non-simple parameters, sema must model the parameter scope and body scope distinctly

### Function Sema

Each function-level semantic record must include:

- syntax function ID, with a 1:1 mapping from `FunctionId` to `FunctionSemaId`
- strictness
- scope root
- parameter-scope ID when distinct from the body scope
- capture set
- whether it needs an environment
- whether it contains direct `eval`
- whether it contains `with`
- whether it needs an `arguments` object
- whether it references `super`, `new.target`, `this`, `await`, or `yield`

This gives the compiler the control bits it needs without rediscovering semantics from the AST.

### Scope Layout Computation

Phase 1 sema computes deterministic binding order for environment-backed scopes.

Rules:

- lexical and var bindings are tracked distinctly
- sema decides which bindings require environment storage
- sema computes stable per-scope binding order for environment slots
- that order is left-to-right source declaration order within the owning scope after pattern flattening
- function parameters use left-to-right parameter introduction order
- lexical scopes, parameter scopes, and var scopes keep separate layout orders; sema does not merge them into one synthetic list
- sema records the function and block scopes that are environment-backed at runtime
- the compiler assigns final frame-register indices for `FrameLocal` bindings later
- initialization and hoisting order remain compiler/runtime concerns; the sema order is a deterministic layout order only

This keeps the environment layout stable without forcing the frontend to own bytecode register allocation.

### Early Errors

`lyng-js-sema` owns early errors such as:

- duplicate lexical bindings
- invalid `break` and `continue`
- invalid `return` outside functions
- strict-mode identifier restrictions
- module/script-only restrictions
- parameter-list and default-initializer visibility errors
- arrow-parameter errors after cover-form conversion
- private-name static semantic errors that do not require runtime knowledge

If a static semantic rule needs runtime object or environment behavior, it belongs to a
later phase rather than being approximated here.

Sema must treat invalid nodes as local holes while still preserving surrounding scope and
binding analysis for valid siblings.

### Private-Name Analysis

Private-name analysis is an explicit part of sema ownership, not an unspecified future add-on.

Rules:

- sema tracks private-name definitions per class body
- sema resolves private-name uses to the defining class-local private-name entry
- duplicate private-name definitions and undefined private-name uses are early errors
- private-name metadata lives in dedicated side tables rather than being shoehorned into
  ordinary lexical binding tables

## Parser/Sema API Contract

The low-level contract is explicit and shared-atom based.

Conceptually:

```rust
fn parse_script(atoms: &mut AtomTable, source_id: SourceId, source: &str) -> ParsedScript;
fn parse_module(atoms: &mut AtomTable, source_id: SourceId, source: &str) -> ParsedModule;

fn analyze_script(parsed: &ParsedScript, atoms: &AtomTable) -> ScriptSema;
fn analyze_module(parsed: &ParsedModule, atoms: &AtomTable) -> ModuleSema;
```

Required parse outputs:

- owning AST container
- root ID
- syntax diagnostics
- source kind
- directive-prologue summary

`ParsedScript` and `ParsedModule` are `lyng-js-ast` types. That keeps `lyng-js-sema`
independent of `lyng-js-parser` while still letting the parser return one stable frontend
artifact per source unit.

Required sema outputs:

- scope table
- binding table
- function table
- capture metadata
- use-site table and reference-resolution results
- early-error diagnostics
- scope layout metadata for later compilation

The exact Rust surface may later add convenience wrappers, but the architectural contract
is that atoms are caller-provided and AST plus sema are explicit values, not implicit
global state.

### Compile-Ready Contract

The compiler should be able to consume:

- AST
- root ID
- sema tables
- atom table

without performing its own name resolution, strictness inference, capture analysis, or
scope-layout discovery.

## Deferred Parsing Compatibility

Take 3 starts with eager full parse plus sema for each source unit. Lazy parsing or
syntax-only parsing is not a Phase 1 requirement.

Phase 1 still freezes the compatibility points that later deferred-parsing work must reuse:

- source text remains recoverable by `SourceId` plus stable function-body ranges; deferred
  work must be able to re-lex from source slices rather than depending on retained token buffers
- function-level frontend records keep enough source identity to re-enter parsing for one
  body without changing AST node IDs, atom ownership, or parse-root ownership
- sema may later add summary-only function metadata for deferred bodies, but the eager path
  remains the semantic reference and the public parse/sema APIs remain source-root oriented
- deferred compilation must plug into the existing compiler/runtime pipeline as "compile this
  source-backed function body now", not as a second frontend stack

This keeps startup-oriented future work open without distorting the initial frontend.

## Performance and Memory Invariants

Non-negotiable frontend invariants:

- no full token vector in the normal parse path
- no public borrowed-node graph APIs
- no per-node heap `Vec` for child lists
- no dependency from frontend crates to runtime-value or GC crates
- identifiers and property-like names are represented as `AtomId`
- parser and sema can handle large inputs without quadratic temporary allocation behavior

## Deferred Work

This document intentionally defers:

- comment retention beyond diagnostics needs
- tooling-oriented CSTs
- source maps for bytecode
- optimization-oriented AST rewrites
- lazy parsing or deferred function-body compilation for startup-heavy workloads
- language-server quality recovery

Those features should not distort the compiler-oriented frontend architecture.
