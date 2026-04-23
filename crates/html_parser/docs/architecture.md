# Architecture

This document explains how the parser is structured today and how data flows through the crate.

## High-Level Pipeline

Parsing follows the same broad pipeline as the HTML standard:

1. `input::stream::InputStream` preprocesses the input stream.
2. `tokenizer::Tokenizer` converts characters into HTML tokens.
3. `tree::builder::TreeBuilder` consumes tokens and builds a DOM tree.
4. `dom` stores the result in an arena-backed representation.
5. `error` collects parse errors without aborting parsing.

The public entry points in [`src/tree/builder.rs`](../src/tree/builder.rs) are:

- `parse_str`
- `parse_str_scripting`
- `parse_fragment`

## Module Responsibilities

### `input/`

[`src/input/stream.rs`](../src/input/stream.rs) owns WHATWG input preprocessing:

- CR and CRLF normalization to LF
- line and column tracking
- reconsume support
- reporting control-character and noncharacter input errors
- injecting additional HTML at the current read position for parser-driven insertion paths

The input stream stores data in segments so inserted markup can be handled without rebuilding the whole stream.

### `tokenizer/`

[`src/tokenizer/mod.rs`](../src/tokenizer/mod.rs) implements the tokenizer as a pull-based state machine:

- `next_token()` returns one token at a time
- the state machine is split across explicit methods per tokenizer state
- character references are resolved in-tokenizer
- start-tag attribute handling, duplicate detection, and parse errors all happen here

The tokenizer is intentionally controlled by the tree builder. That matters because tree construction can switch tokenizer states for elements like `script`, `style`, `title`, and `textarea`.

The token definitions live in [`src/tokenizer/tokens.rs`](../src/tokenizer/tokens.rs).

### `tree/`

[`src/tree/builder.rs`](../src/tree/builder.rs) contains the tree-construction algorithm and is the largest part of the crate.

It maintains the parser state required by the HTML algorithm:

- current insertion mode
- stack of open elements
- list of active formatting elements
- template insertion modes
- foster-parenting state
- scripting flag
- fragment-parsing context

Supporting structures are split out:

- [`src/tree/open.rs`](../src/tree/open.rs): open-elements stack and scope checks
- [`src/tree/active.rs`](../src/tree/active.rs): active formatting elements
- [`src/tree/insertion.rs`](../src/tree/insertion.rs): insertion-mode enum
- [`src/tree/foreign.rs`](../src/tree/foreign.rs): SVG and MathML adjustments

The tree builder is also where foreign-content handling, fragment parsing, foster parenting, and adoption-agency behavior are coordinated.

### `dom`

The shared DOM crate now lives alongside the parser at [`../../dom/src/node.rs`](../../dom/src/node.rs) and stores nodes in an arena:

- every node lives in `Arena.nodes`
- references are `NodeId`
- tree links are parent / child / sibling indices
- nodes can be detached, reparented, and inserted before siblings without `Rc<RefCell<_>>`

Other DOM modules split out payload types and serialization:

- [`../../dom/src/element.rs`](../../dom/src/element.rs)
- [`../../dom/src/document.rs`](../../dom/src/document.rs)
- [`../../dom/src/serialize.rs`](../../dom/src/serialize.rs)

### `error.rs`

[`src/error.rs`](../src/error.rs) defines the parser’s error type and the named parse-error codes. Errors are collected and returned to the caller instead of aborting parsing.

## Data Flow

For complete-document parsing:

1. `TreeBuilder::new` creates an `InputStream`, `Tokenizer`, and empty DOM arena.
2. The builder creates the document node.
3. `TreeBuilder::run` repeatedly asks the tokenizer for the next token.
4. The builder dispatches that token according to the current insertion mode.
5. The builder mutates the DOM arena and parser stacks.
6. On EOF, the builder normalizes fragment state if needed and returns `ParseResult`.

For fragment parsing:

1. The caller provides a `FragmentContext`.
2. The builder creates a synthetic fragment root and context chain.
3. The tokenizer state is adjusted to match the fragment context.
4. Tokens are processed through the normal tree-construction pipeline.
5. The returned `ParseResult` includes fragment metadata.

## Ownership Model

The main ownership boundaries are:

- input text is borrowed where possible by `InputStream`
- tokens own their tag names and attributes
- tree construction moves start-tag payloads into DOM nodes where possible
- DOM nodes own all final strings stored in the parsed tree

This keeps the API straightforward while avoiding some unnecessary cloning in hot paths.

## Testing Strategy

The project uses two layers of validation:

- focused unit tests for low-level DOM and input-stream behavior
- html5lib compatibility suites for tokenizer, tree construction, serializer, and encoding behavior

The html5lib coverage is what gives confidence that recovery behavior matches browser expectations, not just idealized well-formed HTML input.
